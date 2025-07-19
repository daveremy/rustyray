//! Task cancellation and timeout support
//!
//! This module provides mechanisms to cancel tasks and prevent memory leaks
//! from tasks that never complete.

use crate::error::{Result, RustyRayError};
use crate::types::TaskId;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{watch, RwLock};
use tokio::time::{timeout, Instant};

/// Token that can be used to cancel a task
#[derive(Clone)]
pub struct CancellationToken {
    receiver: watch::Receiver<bool>,
}

impl CancellationToken {
    /// Create a new cancellation token pair
    pub fn new() -> (Self, CancellationHandle) {
        let (tx, rx) = watch::channel(false);

        let token = CancellationToken { receiver: rx };

        let handle = CancellationHandle { sender: tx };

        (token, handle)
    }

    /// Check if cancellation has been requested
    pub fn is_cancelled(&self) -> bool {
        *self.receiver.borrow()
    }

    /// Wait for cancellation
    pub async fn cancelled(&mut self) {
        // Wait for the value to become true
        let _ = self.receiver.wait_for(|&cancelled| cancelled).await;
    }
}

/// Handle used to trigger cancellation
pub struct CancellationHandle {
    sender: watch::Sender<bool>,
}

impl CancellationHandle {
    /// Cancel the associated token
    pub fn cancel(&self) {
        let _ = self.sender.send(true);
    }
}

/// Tracks active tasks and their cancellation handles
pub struct TaskTracker {
    /// Map of task ID to cancellation handle and submission time
    tasks: Arc<RwLock<HashMap<TaskId, (CancellationHandle, Instant)>>>,
    /// Default timeout for tasks
    default_timeout: Duration,
}

impl TaskTracker {
    /// Create a new task tracker
    pub fn new(default_timeout: Duration) -> Self {
        TaskTracker {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            default_timeout,
        }
    }

    /// Register a new task and return its cancellation token
    pub async fn register_task(&self, task_id: TaskId) -> CancellationToken {
        let (token, handle) = CancellationToken::new();
        let mut tasks = self.tasks.write().await;
        tasks.insert(task_id, (handle, Instant::now()));
        token
    }

    /// Cancel a specific task
    pub async fn cancel_task(&self, task_id: &TaskId) -> Result<()> {
        let tasks = self.tasks.read().await;
        if let Some((handle, _)) = tasks.get(task_id) {
            handle.cancel();
            Ok(())
        } else {
            Err(RustyRayError::Internal(format!(
                "Task {} not found",
                task_id
            )))
        }
    }

    /// Remove a task from tracking (when it completes)
    pub async fn remove_task(&self, task_id: &TaskId) {
        let mut tasks = self.tasks.write().await;
        tasks.remove(task_id);
    }

    /// Cancel all tasks
    pub async fn cancel_all(&self) {
        let tasks = self.tasks.read().await;
        for (handle, _) in tasks.values() {
            handle.cancel();
        }
    }

    /// Get tasks that have exceeded the timeout
    pub async fn get_timed_out_tasks(&self, timeout_duration: Option<Duration>) -> Vec<TaskId> {
        let timeout = timeout_duration.unwrap_or(self.default_timeout);
        let now = Instant::now();
        let tasks = self.tasks.read().await;

        tasks
            .iter()
            .filter_map(|(id, (_, start_time))| {
                if now.duration_since(*start_time) > timeout {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Cancel tasks that have exceeded the timeout
    pub async fn cancel_timed_out_tasks(&self, timeout_duration: Option<Duration>) -> usize {
        let timed_out = self.get_timed_out_tasks(timeout_duration).await;
        let count = timed_out.len();

        for task_id in timed_out {
            let _ = self.cancel_task(&task_id).await;
        }

        count
    }
}

/// Extension trait for futures to add timeout and cancellation support
pub trait TaskExt: Sized {
    /// Add a timeout to this future
    async fn with_timeout(self, duration: Duration) -> Result<Self::Output>
    where
        Self: std::future::Future,
    {
        match timeout(duration, self).await {
            Ok(result) => Ok(result),
            Err(_) => Err(RustyRayError::Internal("Task timed out".to_string())),
        }
    }

    /// Add cancellation support to this future
    async fn with_cancellation(self, mut token: CancellationToken) -> Result<Self::Output>
    where
        Self: std::future::Future,
    {
        tokio::select! {
            result = self => Ok(result),
            _ = token.cancelled() => Err(RustyRayError::Internal("Task cancelled".to_string())),
        }
    }
}

impl<T: std::future::Future> TaskExt for T {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cancellation_token() {
        let (mut token, handle) = CancellationToken::new();

        // Should not be cancelled initially
        assert!(!token.is_cancelled());

        // Cancel in background
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(10)).await;
            handle.cancel();
        });

        // Wait for cancellation
        token.cancelled().await;
        assert!(token.is_cancelled());
    }

    #[tokio::test]
    async fn test_task_tracker() {
        let tracker = TaskTracker::new(Duration::from_secs(1));
        let task_id = TaskId::new();

        // Register task
        let token = tracker.register_task(task_id).await;
        assert!(!token.is_cancelled());

        // Cancel task
        tracker.cancel_task(&task_id).await.unwrap();
        assert!(token.is_cancelled());

        // Remove task
        tracker.remove_task(&task_id).await;

        // Cancelling removed task should fail
        assert!(tracker.cancel_task(&task_id).await.is_err());
    }

    #[tokio::test]
    async fn test_timeout_detection() {
        let tracker = TaskTracker::new(Duration::from_millis(50));
        let task_id = TaskId::new();

        // Register task
        let _token = tracker.register_task(task_id).await;

        // No timeouts initially
        let timed_out = tracker.get_timed_out_tasks(None).await;
        assert!(timed_out.is_empty());

        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(60)).await;

        // Should detect timeout
        let timed_out = tracker.get_timed_out_tasks(None).await;
        assert_eq!(timed_out.len(), 1);
        assert_eq!(timed_out[0], task_id);

        // Cancel timed out tasks
        let count = tracker.cancel_timed_out_tasks(None).await;
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_future_with_timeout() {
        use crate::task::cancellation::TaskExt;

        // Future that completes in time
        let future = async {
            tokio::time::sleep(Duration::from_millis(10)).await;
            42
        };

        let result = future.with_timeout(Duration::from_millis(50)).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);

        // Future that times out
        let slow_future = async {
            tokio::time::sleep(Duration::from_millis(100)).await;
            42
        };

        let result = slow_future.with_timeout(Duration::from_millis(50)).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("timed out"));
    }

    #[tokio::test]
    async fn test_future_with_cancellation() {
        use crate::task::cancellation::TaskExt;

        let (token, handle) = CancellationToken::new();

        // Cancel in background
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(20)).await;
            handle.cancel();
        });

        // Future that gets cancelled
        let future = async {
            tokio::time::sleep(Duration::from_millis(50)).await;
            42
        };

        let result = future.with_cancellation(token).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cancelled"));
    }
}
