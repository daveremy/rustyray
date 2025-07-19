//! Task manager for coordinating task execution.
//! 
//! The TaskManager is responsible for:
//! - Queuing tasks for execution
//! - Resolving task dependencies
//! - Dispatching tasks to workers
//! - Managing task results

use crate::error::{Result, RustyRayError};
use crate::task::{TaskSpec, TaskArg, ObjectId, FunctionRegistry};
use crate::task::cancellation::{TaskTracker, CancellationToken};
use dashmap::DashMap;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use bytes::Bytes;

/// Configuration for the task manager
#[derive(Clone)]
pub struct TaskManagerConfig {
    /// Timeout duration for tasks
    pub task_timeout: Duration,
    /// How often to check for timed out tasks
    pub timeout_check_interval: Duration,
}

impl Default for TaskManagerConfig {
    fn default() -> Self {
        TaskManagerConfig {
            task_timeout: Duration::from_secs(300), // 5 minutes
            timeout_check_interval: Duration::from_secs(1), // Check every second
        }
    }
}

impl TaskManagerConfig {
    /// Create a test configuration with short timeouts
    pub fn for_tests() -> Self {
        TaskManagerConfig {
            task_timeout: Duration::from_millis(100),
            timeout_check_interval: Duration::from_millis(10), // Check every 10ms for tests
        }
    }
}

/// Message sent to the task manager
enum TaskMessage {
    /// Submit a new task for execution
    Submit {
        spec: TaskSpec,
        result_tx: oneshot::Sender<Result<Vec<u8>>>,
    },
    /// Notify that an object is now available
    ObjectReady {
        id: ObjectId,
        data: Vec<u8>,
    },
    /// Shutdown the task manager
    Shutdown,
}

/// A task waiting for dependencies
struct PendingTask {
    spec: TaskSpec,
    result_tx: oneshot::Sender<Result<Vec<u8>>>,
    waiting_for: Vec<ObjectId>,
    /// Cancellation token for this task
    cancellation_token: CancellationToken,
}

/// The task manager coordinates task execution.
/// 
/// It maintains queues of pending tasks, tracks object availability,
/// and dispatches tasks to workers when their dependencies are ready.
pub struct TaskManager {
    /// Channel for receiving task messages
    sender: mpsc::Sender<TaskMessage>,
    /// Handle to the background task
    handle: Arc<tokio::sync::Mutex<Option<JoinHandle<()>>>>,
    /// Shutdown state
    is_shutdown: Arc<AtomicBool>,
}

impl TaskManager {
    /// Create a new task manager with default config
    pub fn new(registry: Arc<FunctionRegistry>) -> Self {
        Self::with_config(registry, TaskManagerConfig::default())
    }
    
    /// Create a new task manager with custom timeout
    pub fn with_timeout(registry: Arc<FunctionRegistry>, timeout: Duration) -> Self {
        let config = TaskManagerConfig {
            task_timeout: timeout,
            timeout_check_interval: Duration::from_millis(timeout.as_millis().min(1000) as u64 / 10), // 1/10th of timeout or 1 second
        };
        Self::with_config(registry, config)
    }
    
    /// Create a new task manager with custom config
    pub fn with_config(registry: Arc<FunctionRegistry>, config: TaskManagerConfig) -> Self {
        let (tx, rx) = mpsc::channel(1000);
        let tx_clone = tx.clone();
        
        let handle = tokio::spawn(async move {
            let mut manager = TaskManagerWorker::new(tx_clone, registry, config);
            manager.run(rx).await;
        });
        
        TaskManager {
            sender: tx,
            handle: Arc::new(tokio::sync::Mutex::new(Some(handle))),
            is_shutdown: Arc::new(AtomicBool::new(false)),
        }
    }
    
    /// Queue a task for execution
    pub async fn queue_task(
        &self,
        spec: TaskSpec,
        result_tx: oneshot::Sender<Result<Vec<u8>>>,
    ) -> Result<()> {
        self.sender
            .send(TaskMessage::Submit { spec, result_tx })
            .await
            .map_err(|_| RustyRayError::Internal("Task manager shut down".to_string()))
    }
    
    /// Notify that an object is ready
    pub async fn notify_object_ready(&self, id: ObjectId, data: Vec<u8>) -> Result<()> {
        self.sender
            .send(TaskMessage::ObjectReady { id, data })
            .await
            .map_err(|_| RustyRayError::Internal("Task manager shut down".to_string()))
    }
    
    /// Shutdown the task manager
    /// 
    /// This method is idempotent - it can be called multiple times safely.
    pub async fn shutdown(&self) -> Result<()> {
        // Check if already shutdown
        if self.is_shutdown.swap(true, Ordering::AcqRel) {
            // Already shut down
            return Ok(());
        }
        
        // Send shutdown message
        let _ = self.sender.send(TaskMessage::Shutdown).await;
        
        // Wait for worker to finish
        let mut handle_guard = self.handle.lock().await;
        if let Some(handle) = handle_guard.take() {
            handle.await
                .map_err(|e| RustyRayError::Internal(format!("Task manager panic: {:?}", e)))?;
        }
        
        Ok(())
    }
}

/// Internal worker for the task manager
struct TaskManagerWorker {
    /// Tasks waiting for dependencies
    pending_tasks: Vec<PendingTask>,
    /// Available objects - using Bytes for zero-copy cloning
    object_store: DashMap<ObjectId, Bytes>,
    /// Active task handles
    active_tasks: Vec<JoinHandle<()>>,
    /// Sender for internal messages
    sender: mpsc::Sender<TaskMessage>,
    /// Function registry
    registry: Arc<FunctionRegistry>,
    /// Task tracker for cancellation and timeouts
    task_tracker: Arc<TaskTracker>,
    /// How often to check for timeouts
    timeout_check_interval: Duration,
}

impl TaskManagerWorker {
    fn new(sender: mpsc::Sender<TaskMessage>, registry: Arc<FunctionRegistry>, config: TaskManagerConfig) -> Self {
        TaskManagerWorker {
            pending_tasks: Vec::new(),
            object_store: DashMap::new(),
            active_tasks: Vec::new(),
            sender,
            registry,
            task_tracker: Arc::new(TaskTracker::new(config.task_timeout)),
            timeout_check_interval: config.timeout_check_interval,
        }
    }
    
    async fn run(&mut self, mut rx: mpsc::Receiver<TaskMessage>) {
        // Spawn a background task to periodically check for timeouts
        let tracker = self.task_tracker.clone();
        let check_interval = self.timeout_check_interval;
        let timeout_checker = tokio::spawn(async move {
            let mut interval = tokio::time::interval(check_interval);
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                interval.tick().await;
                let cancelled = tracker.cancel_timed_out_tasks(None).await;
                if cancelled > 0 {
                    eprintln!("Cancelled {} timed out tasks", cancelled);
                }
            }
        });
        
        while let Some(msg) = rx.recv().await {
            match msg {
                TaskMessage::Submit { spec, result_tx } => {
                    self.handle_submit(spec, result_tx).await;
                }
                TaskMessage::ObjectReady { id, data } => {
                    self.handle_object_ready(id, data).await;
                }
                TaskMessage::Shutdown => {
                    break;
                }
            }
            
            // Clean up completed tasks
            self.active_tasks.retain(|handle| !handle.is_finished());
            
            // Clean up cancelled pending tasks
            self.pending_tasks.retain(|task| !task.cancellation_token.is_cancelled());
        }
        
        // Stop timeout checker
        timeout_checker.abort();
        
        // Wait for all active tasks to complete
        for handle in self.active_tasks.drain(..) {
            let _ = handle.await;
        }
    }
    
    async fn handle_submit(&mut self, spec: TaskSpec, result_tx: oneshot::Sender<Result<Vec<u8>>>) {
        // Register task with the tracker
        let cancellation_token = self.task_tracker.register_task(spec.task_id).await;
        
        // Check if task has dependencies
        let deps = spec.dependencies();
        if deps.is_empty() {
            // No dependencies, execute immediately
            self.execute_task(spec, result_tx, cancellation_token);
        } else {
            // Check which dependencies are already available
            let mut waiting_for = Vec::new();
            for dep_id in deps {
                if !self.object_store.contains_key(dep_id) {
                    waiting_for.push(*dep_id);
                }
            }
            
            if waiting_for.is_empty() {
                // All dependencies ready, execute now
                self.execute_task(spec, result_tx, cancellation_token);
            } else {
                // Queue task to wait for dependencies
                self.pending_tasks.push(PendingTask {
                    spec,
                    result_tx,
                    waiting_for,
                    cancellation_token,
                });
            }
        }
    }
    
    async fn handle_object_ready(&mut self, id: ObjectId, data: Vec<u8>) {
        // Store the object as Bytes for cheap cloning
        self.object_store.insert(id, Bytes::from(data));
        
        // Check pending tasks to see if any can now run
        let mut still_pending = Vec::new();
        let mut ready_tasks = Vec::new();
        
        for mut task in self.pending_tasks.drain(..) {
            // Remove this object from waiting list
            task.waiting_for.retain(|dep_id| *dep_id != id);
            
            if task.waiting_for.is_empty() {
                // All dependencies ready, queue for execution
                ready_tasks.push((task.spec, task.result_tx, task.cancellation_token));
            } else {
                // Still waiting for other dependencies
                still_pending.push(task);
            }
        }
        self.pending_tasks = still_pending;
        
        // Execute ready tasks
        for (spec, result_tx, cancellation_token) in ready_tasks {
            self.execute_task(spec, result_tx, cancellation_token);
        }
    }
    
    fn execute_task(&mut self, spec: TaskSpec, result_tx: oneshot::Sender<Result<Vec<u8>>>, cancellation_token: CancellationToken) {
        let object_store = self.object_store.clone();
        let sender = self.sender.clone();
        let task_id = spec.task_id;
        let registry = self.registry.clone();
        
        let tracker = self.task_tracker.clone();
        let handle = tokio::spawn(async move {
            // Use cancellation extension
            use crate::task::cancellation::TaskExt;
            
            // Check if already cancelled
            if cancellation_token.is_cancelled() {
                let _ = result_tx.send(Err(RustyRayError::Internal("Task cancelled".to_string())));
                return;
            }
            
            // Execute with cancellation support
            let execution = async {
                // Resolve dependencies
                let resolved_args = match Self::resolve_dependencies(&spec.args, &object_store).await {
                    Ok(args) => args,
                    Err(e) => {
                        eprintln!("Failed to resolve dependencies for task {}: {:?}", spec.task_id, e);
                        return Err(e);
                    }
                };
                
                // Look up function
                let function = match registry.get(&spec.function_id) {
                    Some(f) => f,
                    None => {
                        let error = RustyRayError::Internal(
                            format!("Function {} not found", spec.function_id)
                        );
                        eprintln!("Function {} not found for task {}", spec.function_id, spec.task_id);
                        return Err(error);
                    }
                };
                
                // Execute function with the vector of resolved arguments
                match function(resolved_args).await {
                    Ok(result) => {
                        // Also notify the task manager that this object is ready
                        // so other tasks waiting on it can proceed
                        // Convert TaskId to ObjectId by using the same underlying value
                        let object_id = ObjectId::from(task_id.as_u64());
                        let _ = sender.send(TaskMessage::ObjectReady { 
                            id: object_id, 
                            data: result.clone() 
                        }).await;
                        Ok(result)
                    }
                    Err(e) => {
                        // Note: We don't notify object_ready for failed tasks
                        // This prevents dependent tasks from trying to use invalid data
                        eprintln!("Task {} execution failed: {:?}", spec.task_id, e);
                        Err(e)
                    }
                }
            };
            
            // Run with cancellation and send result
            let result = match execution.with_cancellation(cancellation_token).await {
                Ok(Ok(data)) => Ok(data),  // Successful execution
                Ok(Err(e)) => Err(e),      // Task failed
                Err(e) => Err(e),          // Cancelled or timed out
            };
            let _ = result_tx.send(result);
            
            // Remove from tracker when done
            tracker.remove_task(&task_id).await;
        });
        
        self.active_tasks.push(handle);
    }
    
    async fn resolve_dependencies(
        args: &[TaskArg],
        object_store: &DashMap<ObjectId, Bytes>,
    ) -> Result<Vec<Vec<u8>>> {
        // Resolve each argument, preserving them as separate byte vectors
        // This matches Ray's model where arguments are handled individually
        
        // Pre-allocate with exact capacity to avoid reallocations
        let mut resolved_args = Vec::with_capacity(args.len());
        
        for arg in args {
            match arg {
                TaskArg::Value(bytes) => {
                    // Direct value, already serialized
                    resolved_args.push(bytes.clone());
                }
                TaskArg::ObjectRef(id) => {
                    // Look up object in store
                    match object_store.get(id) {
                        Some(data) => {
                            // Convert Bytes back to Vec<u8> - this is cheap if no other
                            // references exist, otherwise it clones
                            resolved_args.push(data.to_vec());
                        }
                        None => {
                            return Err(RustyRayError::Internal(
                                format!("Object {} not found in store", id)
                            ));
                        }
                    }
                }
            }
        }
        
        Ok(resolved_args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::FunctionId;
    
    #[tokio::test]
    async fn test_simple_task_execution() {
        // Create registry
        let registry = Arc::new(FunctionRegistry::new());
        
        // Register a test function
        let func_id = FunctionId::from("test_add");
        registry.register(func_id.clone(), |args| {
            Box::pin(async move {
                if args.len() != 2 {
                    return Err(RustyRayError::Internal("Expected 2 arguments".to_string()));
                }
                let x: i32 = crate::task::serde_utils::deserialize(&args[0])?;
                let y: i32 = crate::task::serde_utils::deserialize(&args[1])?;
                let result = x + y;
                Ok(crate::task::serde_utils::serialize(&result).unwrap())
            })
        }).unwrap();
        
        // Create task manager
        let manager = TaskManager::new(registry);
        
        // Create task spec
        let args = vec![
            TaskArg::from_value(&5i32).unwrap(),
            TaskArg::from_value(&7i32).unwrap(),
        ];
        let spec = TaskSpec::normal(func_id, args);
        
        // Submit task
        let (tx, rx) = oneshot::channel();
        manager.queue_task(spec, tx).await.unwrap();
        
        // Get result
        let result_bytes = rx.await.unwrap().unwrap(); // Unwrap the Result<Vec<u8>>
        let result: i32 = crate::task::serde_utils::deserialize(&result_bytes).unwrap();
        assert_eq!(result, 12);
        
        manager.shutdown().await.unwrap();
    }
}