//! High-level task system API.
//!
//! This module provides the main interface for submitting and managing tasks.
//! It integrates with the actor system to provide unified execution.

use crate::actor::ActorSystem;
use crate::error::{Result, RustyRayError};
use crate::object_ref::ObjectRef;
use crate::object_store::{InMemoryStore, ObjectStore, StoreConfig};
#[cfg(test)]
use crate::task::TaskManagerConfig;
use crate::task::{FunctionId, FunctionRegistry, TaskArg, TaskManager, TaskSpec};
use crate::types::ObjectId;
use serde::{de::DeserializeOwned, Serialize};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

/// Represents the current state of the task system
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum ShutdownState {
    Running = 0,
    ShuttingDown = 1,
    Shutdown = 2,
}

impl From<u8> for ShutdownState {
    fn from(value: u8) -> Self {
        match value {
            0 => ShutdownState::Running,
            1 => ShutdownState::ShuttingDown,
            2 => ShutdownState::Shutdown,
            _ => ShutdownState::Shutdown,
        }
    }
}

/// The main task system that coordinates task execution.
///
/// This integrates with the actor system to provide a unified
/// execution environment for both tasks and actors.
pub struct TaskSystem {
    /// Function registry for task functions
    registry: Arc<FunctionRegistry>,

    /// Task manager for execution
    task_manager: Arc<TaskManager>,

    /// Reference to the actor system for integration
    #[allow(dead_code)]
    actor_system: Arc<ActorSystem>,

    /// Object store for sharing data between tasks
    object_store: Arc<InMemoryStore>,

    /// Shutdown state
    shutdown_state: Arc<AtomicU8>,
    
    /// Tracks spawned tasks for result storage
    result_storage_tasks: Arc<Mutex<Vec<JoinHandle<()>>>>,
}

impl TaskSystem {
    /// Create a new task system
    pub fn new(actor_system: Arc<ActorSystem>) -> Self {
        let registry = Arc::new(FunctionRegistry::new());
        let task_manager = Arc::new(TaskManager::new(registry.clone()));
        let object_store = Arc::new(InMemoryStore::new(StoreConfig::default()));

        TaskSystem {
            registry,
            task_manager,
            actor_system,
            object_store,
            shutdown_state: Arc::new(AtomicU8::new(ShutdownState::Running as u8)),
            result_storage_tasks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create a new task system with custom timeout for tasks
    pub fn with_timeout(actor_system: Arc<ActorSystem>, timeout: Duration) -> Self {
        let registry = Arc::new(FunctionRegistry::new());
        let task_manager = Arc::new(TaskManager::with_timeout(registry.clone(), timeout));
        let object_store = Arc::new(InMemoryStore::new(StoreConfig::default()));

        TaskSystem {
            registry,
            task_manager,
            actor_system,
            object_store,
            shutdown_state: Arc::new(AtomicU8::new(ShutdownState::Running as u8)),
            result_storage_tasks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create a new task system with a shared object store
    pub fn with_store(actor_system: Arc<ActorSystem>, object_store: Arc<InMemoryStore>) -> Self {
        let registry = Arc::new(FunctionRegistry::new());
        let task_manager = Arc::new(TaskManager::new(registry.clone()));

        TaskSystem {
            registry,
            task_manager,
            actor_system,
            object_store,
            shutdown_state: Arc::new(AtomicU8::new(ShutdownState::Running as u8)),
            result_storage_tasks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create a new task system for tests with aggressive timeouts
    #[cfg(test)]
    pub fn for_tests(actor_system: Arc<ActorSystem>) -> Self {
        let registry = Arc::new(FunctionRegistry::new());
        let task_manager = Arc::new(TaskManager::with_config(
            registry.clone(),
            TaskManagerConfig::for_tests(),
        ));
        let object_store = Arc::new(InMemoryStore::new(StoreConfig::default()));

        TaskSystem {
            registry,
            task_manager,
            actor_system,
            object_store,
            shutdown_state: Arc::new(AtomicU8::new(ShutdownState::Running as u8)),
            result_storage_tasks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Submit a task for execution and return an ObjectRef.
    ///
    /// This is the main entry point for task execution. The task is
    /// queued for execution and an ObjectRef is returned immediately.
    /// The actual execution happens asynchronously.
    pub async fn submit<T>(&self, mut spec: TaskSpec) -> Result<ObjectRef<T>>
    where
        T: DeserializeOwned + Send + 'static,
    {
        // Check if shutting down
        let state = ShutdownState::from(self.shutdown_state.load(Ordering::Acquire));
        if state != ShutdownState::Running {
            return Err(RustyRayError::Internal(
                "Task system is shutting down".to_string(),
            ));
        }

        // Create a channel for the result
        let (result_tx, result_rx) = tokio::sync::oneshot::channel();

        // Generate object ID for the result
        let object_id = ObjectId::new();
        spec.task_id = crate::types::TaskId::from(object_id);

        // Queue task for execution
        self.task_manager.queue_task(spec, result_tx, self.object_store.clone()).await?;

        // Create ObjectRef backed by store
        let object_ref = ObjectRef::with_store(object_id, self.object_store.clone());

        // Spawn a task to store the result when it's ready
        let store = self.object_store.clone();
        let task_tracker = self.result_storage_tasks.clone();
        let handle = tokio::spawn(async move {
            if let Ok(result) = result_rx.await {
                // Store both successful results and errors
                // We wrap the Result<Vec<u8>> to preserve error information
                let wrapped_result = match result {
                    Ok(bytes) => {
                        // Successful result - store the raw bytes
                        bytes
                    }
                    Err(e) => {
                        // Error result - serialize the error as a special error marker
                        // We'll use a simple encoding: prefix with error marker + error message
                        let error_marker = b"__RUSTYRAY_ERROR__";
                        let error_msg = e.to_string().into_bytes();
                        let mut error_bytes =
                            Vec::with_capacity(error_marker.len() + error_msg.len());
                        error_bytes.extend_from_slice(error_marker);
                        error_bytes.extend_from_slice(&error_msg);
                        error_bytes
                    }
                };

                // Store the result (success or error) in the object store
                let _ = store.put_with_id(object_id, wrapped_result).await;
            }
        });
        
        // Track the spawned task
        {
            let mut tasks = task_tracker.lock().await;
            // Clean up completed tasks
            tasks.retain(|handle| !handle.is_finished());
            // Add the new task
            tasks.push(handle);
        }

        Ok(object_ref)
    }

    /// Submit a task with specific arguments.
    ///
    /// This is a convenience method that creates a TaskSpec and submits it.
    pub async fn submit_task<T>(
        &self,
        function_id: crate::task::FunctionId,
        args: Vec<TaskArg>,
    ) -> Result<ObjectRef<T>>
    where
        T: DeserializeOwned + Send + 'static,
    {
        let spec = TaskSpec::normal(function_id, args);
        self.submit(spec).await
    }

    /// Put an object into the object store.
    ///
    /// This is used to make data available to tasks as dependencies.
    /// Returns an ObjectRef that can be passed to other tasks.
    pub async fn put<T>(&self, value: T) -> Result<ObjectRef<T>>
    where
        T: Serialize + DeserializeOwned + Send + 'static,
    {
        // Serialize the value for backward compatibility with task manager
        let bytes = crate::task::serde_utils::serialize(&value)
            .map_err(|e| RustyRayError::Internal(format!("Failed to serialize object: {}", e)))?;

        // Store in object store
        let result = self.object_store.put(value).await?;

        // Create ObjectRef with store reference
        let object_ref = ObjectRef::with_store(result.id, self.object_store.clone());

        // Also notify task manager for backward compatibility
        // The task manager needs the bytes for resolving dependencies
        self.task_manager
            .notify_object_ready(result.id, bytes)
            .await?;

        Ok(object_ref)
    }

    /// Register a function with the task system.
    ///
    /// Functions must be registered before they can be called as tasks.
    pub fn register_function<F>(&self, id: impl Into<FunctionId>, function: F) -> Result<()>
    where
        F: Fn(Vec<Vec<u8>>, crate::task::context::DeserializationContext) -> crate::task::BoxFuture<'static, Result<Vec<u8>>>
            + Send
            + Sync
            + 'static,
    {
        self.registry.register(id.into(), function)
    }

    /// Clear the function registry (for testing)
    pub fn clear_registry(&self) {
        self.registry.clear();
    }

    /// Check if the task system is shut down
    pub fn is_shutdown(&self) -> bool {
        ShutdownState::from(self.shutdown_state.load(Ordering::Acquire)) == ShutdownState::Shutdown
    }
    
    /// Wait for all spawned result storage tasks to complete
    pub async fn wait_for_tasks(&self) -> Result<()> {
        let tasks = {
            let mut tasks_guard = self.result_storage_tasks.lock().await;
            std::mem::take(&mut *tasks_guard)
        };
        
        for task in tasks {
            // Ignore errors from individual tasks
            let _ = task.await;
        }
        
        Ok(())
    }

    /// Shutdown the task system gracefully.
    ///
    /// This will:
    /// 1. Stop accepting new tasks
    /// 2. Wait for all pending tasks to complete
    /// 3. Clean up resources
    ///
    /// This method is idempotent - it can be called multiple times safely.
    pub async fn shutdown(&self) -> Result<()> {
        // Try to transition from Running to ShuttingDown
        let prev_state = self.shutdown_state.compare_exchange(
            ShutdownState::Running as u8,
            ShutdownState::ShuttingDown as u8,
            Ordering::AcqRel,
            Ordering::Acquire,
        );

        match ShutdownState::from(prev_state.unwrap_or_else(|e| e)) {
            ShutdownState::Running => {
                // We successfully initiated shutdown
                // Shutdown task manager
                self.task_manager.shutdown().await?;
                
                // Wait for all result storage tasks to complete
                let tasks = {
                    let mut tasks_guard = self.result_storage_tasks.lock().await;
                    std::mem::take(&mut *tasks_guard)
                };
                
                for task in tasks {
                    // Ignore errors from task completion
                    let _ = task.await;
                }

                // Mark as fully shutdown
                self.shutdown_state
                    .store(ShutdownState::Shutdown as u8, Ordering::Release);
                Ok(())
            }
            ShutdownState::ShuttingDown => {
                // Another thread is already shutting down
                // Wait for it to complete
                while ShutdownState::from(self.shutdown_state.load(Ordering::Acquire))
                    == ShutdownState::ShuttingDown
                {
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                }
                Ok(())
            }
            ShutdownState::Shutdown => {
                // Already shutdown
                Ok(())
            }
        }
    }
}

/// Builder pattern for creating tasks with a fluent API
pub struct TaskBuilder {
    function_id: crate::task::FunctionId,
    args: Vec<TaskArg>,
    resources: crate::task::spec::TaskResources,
    /// Stores any error that occurred during building
    error: Option<RustyRayError>,
}

impl TaskBuilder {
    /// Create a new task builder
    pub fn new(function_id: impl Into<crate::task::FunctionId>) -> Self {
        TaskBuilder {
            function_id: function_id.into(),
            args: Vec::new(),
            resources: Default::default(),
            error: None,
        }
    }

    /// Add a value argument
    ///
    /// If serialization fails, the error is stored and will be returned
    /// when submit() is called.
    pub fn arg<T: Serialize>(mut self, value: T) -> Self {
        if self.error.is_none() {
            match TaskArg::from_value(&value) {
                Ok(arg) => self.args.push(arg),
                Err(e) => self.error = Some(e),
            }
        }
        self
    }

    /// Add an ObjectRef argument
    pub fn arg_ref<T>(mut self, object_ref: &ObjectRef<T>) -> Self {
        self.args.push(TaskArg::from_object_ref(object_ref.id()));
        self
    }

    /// Set CPU requirements
    pub fn num_cpus(mut self, cpus: f64) -> Self {
        self.resources.num_cpus = cpus;
        self
    }

    /// Set GPU requirements
    pub fn num_gpus(mut self, gpus: f64) -> Self {
        self.resources.num_gpus = gpus;
        self
    }

    /// Build the TaskSpec
    ///
    /// Returns an error if any arguments failed to serialize.
    pub fn build(self) -> Result<TaskSpec> {
        if let Some(error) = self.error {
            return Err(error);
        }
        Ok(TaskSpec::normal(self.function_id, self.args).with_resources(self.resources))
    }

    /// Submit the task
    ///
    /// Returns an error if any arguments failed to serialize or if submission fails.
    pub async fn submit<T>(self, system: &TaskSystem) -> Result<ObjectRef<T>>
    where
        T: DeserializeOwned + Send + 'static,
    {
        let spec = self.build()?;
        system.submit(spec).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::with_test_runtime;

    #[tokio::test]
    async fn test_task_system_basic() {
        with_test_runtime(|| async {
            // Use runtime's task system
            let runtime = crate::runtime::global().unwrap();
            let task_system = runtime.task_system();

            // Register a function with unique name for this test
            task_system
                .register_function(
                    "test_basic_double",
                    crate::task_function!(|x: i32| async move {
                        Ok::<i32, RustyRayError>(x * 2)
                    }),
                )
                .unwrap();

            // Submit a task
            let args = vec![TaskArg::from_value(&21).unwrap()];
            let result_ref: ObjectRef<i32> = task_system
                .submit_task("test_basic_double".into(), args)
                .await
                .expect("Failed to submit task");

            // Get result
            let result = result_ref.get().await.unwrap();
            assert_eq!(result, 42);
        }).await;
    }

    #[tokio::test]
    async fn test_task_builder() {
        with_test_runtime(|| async {
            // Use runtime's task system
            let runtime = crate::runtime::global().unwrap();
            let task_system = runtime.task_system();

            // Register a function with unique name for this test
            task_system
                .register_function(
                    "test_builder_add",
                    crate::task_function!(|x: i32, y: i32| async move {
                        Ok::<i32, RustyRayError>(x + y)
                    }),
                )
                .unwrap();

            // Use builder pattern
            let result_ref: ObjectRef<i32> = TaskBuilder::new("test_builder_add")
                .arg(10)
                .arg(15)
                .num_cpus(1.0)
                .submit(&task_system)
                .await
                .unwrap();

            // Get result
            let result = result_ref.get().await.unwrap();
            assert_eq!(result, 25);
        }).await;
    }

    #[tokio::test]
    async fn test_object_put_and_dependency() {
        with_test_runtime(|| async {
            // Use runtime's task system
            let runtime = crate::runtime::global().unwrap();
            let task_system = runtime.task_system();

            // Register a function with unique name for this test
            task_system
                .register_function(
                    "test_dependency_use_ref",
                    crate::task_function!(|x: i32| async move {
                        Ok::<i32, RustyRayError>(x + 100)
                    }),
                )
                .unwrap();

            // Put an object
            let obj_ref = task_system.put(42i32).await.unwrap();

            // Submit task that uses the object
            let result_ref: ObjectRef<i32> = TaskBuilder::new("test_dependency_use_ref")
                .arg_ref(&obj_ref)
                .submit(&task_system)
                .await
                .unwrap();

            // Get result
            let result = result_ref.get().await.unwrap();
            assert_eq!(result, 142);
        }).await;
    }
}
