//! Comprehensive tests for shutdown logic

use crate::actor::Actor;
use crate::error::Result;
use crate::task::TaskBuilder;
use crate::task_function;
use crate::test_utils::TestSync;
use async_trait::async_trait;
use std::any::Any;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::Duration;
use tokio::sync::Notify;
use tokio::time;

/// Test actor that tracks operations
struct TestActor {
    operations: Arc<AtomicUsize>,
    on_stop_notify: Option<Arc<Notify>>,
}

#[async_trait]
impl Actor for TestActor {
    async fn handle(&mut self, _msg: Box<dyn Any + Send>) -> Result<Box<dyn Any + Send>> {
        self.operations.fetch_add(1, Ordering::SeqCst);
        // Simulate some work
        time::sleep(Duration::from_millis(10)).await;
        Ok(Box::new(()))
    }

    async fn on_stop(&mut self) -> Result<()> {
        // Mark that on_stop was called
        self.operations.fetch_add(1000, Ordering::SeqCst);

        // Notify if someone is waiting
        if let Some(notify) = &self.on_stop_notify {
            notify.notify_one();
        }

        Ok(())
    }
}

#[tokio::test]
async fn test_task_system_idempotent_shutdown() {
    crate::test_utils::with_test_runtime(|| async {
        let runtime = crate::runtime::global().unwrap();
        let actor_system = runtime.actor_system();
        let task_system = runtime.task_system();

        // Register a function
        task_system
            .register_function(
                "test",
                task_function!(
                    |x: i32| async move { Ok::<i32, crate::error::RustyRayError>(x * 2) }
                ),
            )
            .unwrap();

        // Submit a task
        let result_ref = TaskBuilder::new("test")
            .arg(21)
            .submit::<i32>(task_system)
            .await
            .unwrap();

        // Get result
        let result = result_ref.get().await.unwrap();
        assert_eq!(result, 42);

        // First shutdown
        task_system.shutdown().await.unwrap();

        // Second shutdown should also succeed (idempotent)
        task_system.shutdown().await.unwrap();

        // Third shutdown should also succeed
        task_system.shutdown().await.unwrap();

        // Actor system shutdown should also be idempotent
        actor_system.shutdown().await.unwrap();
        actor_system.shutdown().await.unwrap();
    })
    .await;
}

#[tokio::test]
async fn test_concurrent_shutdown() {
    crate::test_utils::with_test_runtime(|| async {
        let runtime = crate::runtime::global().unwrap();
        let actor_system = runtime.actor_system();
        let task_system = runtime.task_system();

        // Register a function
        task_system
            .register_function(
                "sleep",
                task_function!(|ms: u64| async move {
                    time::sleep(Duration::from_millis(ms)).await;
                    Ok::<(), crate::error::RustyRayError>(())
                }),
            )
            .unwrap();

        // Submit some tasks
        for i in 0..5 {
            drop(
                TaskBuilder::new("sleep")
                    .arg(50u64 + i * 10)
                    .submit::<()>(task_system)
                    .await
                    .unwrap(),
            );
        }

        // Spawn multiple concurrent shutdown tasks
        let mut handles = vec![];
        for _ in 0..5 {
            let ts = task_system.clone();
            let as_ = actor_system.clone();
            handles.push(tokio::spawn(async move {
                ts.shutdown().await.unwrap();
                as_.shutdown().await.unwrap();
            }));
        }

        // All should complete successfully
        for handle in handles {
            handle.await.unwrap();
        }
    })
    .await;
}

#[tokio::test]
async fn test_no_new_tasks_after_shutdown() {
    crate::test_utils::with_test_runtime(|| async {
        let runtime = crate::runtime::global().unwrap();
        let task_system = runtime.task_system();

        // Register a function
        task_system
            .register_function(
                "test",
                task_function!(|x: i32| async move { Ok::<i32, crate::error::RustyRayError>(x) }),
            )
            .unwrap();

        // Shutdown
        task_system.shutdown().await.unwrap();

        // Try to submit a task - should fail
        let result = TaskBuilder::new("test")
            .arg(42)
            .submit::<i32>(task_system)
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("shutting down"));
    })
    .await;
}

#[tokio::test]
async fn test_no_new_actors_after_shutdown() {
    crate::test_utils::with_test_runtime(|| async {
        let runtime = crate::runtime::global().unwrap();
        let actor_system = runtime.actor_system();

        // Create an actor before shutdown
        let ops1 = Arc::new(AtomicUsize::new(0));
        let on_stop_notify = TestSync::notification();
        let actor1 = TestActor {
            operations: ops1.clone(),
            on_stop_notify: Some(on_stop_notify.clone()),
        };
        let actor_ref = actor_system.create_actor(actor1).await.unwrap();

        // Send a message - should work
        actor_ref.send(Box::new(())).await.unwrap();

        // Shutdown should wait for on_stop to complete
        actor_system.shutdown().await.unwrap();

        // No sleep needed - shutdown should have waited for on_stop
        // Check that on_stop was called (adds 1000)
        let final_ops = ops1.load(Ordering::SeqCst);
        assert!(final_ops >= 1001); // At least 1 operation + 1000 from on_stop

        // Try to create a new actor - should fail
        let ops2 = Arc::new(AtomicUsize::new(0));
        let actor2 = TestActor {
            operations: ops2,
            on_stop_notify: None,
        };
        let result = actor_system.create_actor(actor2).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("shutting down"));
    })
    .await;
}

#[tokio::test]
async fn test_graceful_actor_shutdown() {
    crate::test_utils::with_test_runtime(|| async {
        let runtime = crate::runtime::global().unwrap();
        let actor_system = runtime.actor_system();
        let operations = Arc::new(AtomicUsize::new(0));

        // Create multiple actors
        let mut refs = vec![];
        for _ in 0..3 {
            let actor = TestActor {
                operations: operations.clone(),
                on_stop_notify: None,
            };
            let actor_ref = actor_system.create_actor(actor).await.unwrap();
            refs.push(actor_ref);
        }

        // Send messages to all actors
        for actor_ref in &refs {
            for _ in 0..5 {
                actor_ref.send(Box::new(())).await.unwrap();
            }
        }

        // Shutdown should wait for all messages to be processed
        actor_system.shutdown().await.unwrap();

        // All actors should have processed their messages and called on_stop
        let final_ops = operations.load(Ordering::SeqCst);
        // 3 actors * (5 operations + 1000 from on_stop) = 3015
        assert_eq!(final_ops, 3015);
    })
    .await;
}

#[tokio::test]
async fn test_task_completion_before_shutdown() {
    crate::test_utils::with_test_runtime(|| async {
        let runtime = crate::runtime::global().unwrap();
        let actor_system = runtime.actor_system();
        let task_system = runtime.task_system();

        // Create a barrier to ensure all tasks have started
        let task_started = Arc::new(tokio::sync::Barrier::new(6)); // 5 tasks + 1 main thread

        // Register a slow function that signals when it starts
        let barrier_clone = task_started.clone();
        let slow_fn =
            move |args: Vec<Vec<u8>>, _context: crate::task::context::DeserializationContext| {
                let barrier = barrier_clone.clone();
                Box::pin(async move {
                    if args.len() != 1 {
                        return Err(crate::error::RustyRayError::Internal(
                            "Expected 1 argument".to_string(),
                        ));
                    }
                    let x: i32 = crate::task::serde_utils::deserialize(&args[0])?;

                    // Signal that task has started
                    barrier.wait().await;
                    time::sleep(Duration::from_millis(100)).await;
                    let result = x * 3;
                    Ok(crate::task::serde_utils::serialize(&result).unwrap())
                }) as crate::task::BoxFuture<'static, crate::error::Result<Vec<u8>>>
            };
        task_system.register_function("slow", slow_fn).unwrap();

        // Submit multiple tasks
        let mut refs = vec![];
        for i in 0..5 {
            let result_ref = TaskBuilder::new("slow")
                .arg(i)
                .submit::<i32>(task_system)
                .await
                .unwrap();
            refs.push((i, result_ref));
        }

        // Start shutdown after all tasks have started
        let shutdown_handle = tokio::spawn({
            let ts = task_system.clone();
            let as_ = actor_system.clone();
            let barrier = task_started.clone();
            async move {
                // Wait for all tasks to start
                barrier.wait().await;
                ts.shutdown().await.unwrap();
                as_.shutdown().await.unwrap();
            }
        });

        // All tasks should still complete successfully
        for (i, ref_) in refs {
            let result = ref_.get().await.unwrap();
            assert_eq!(result, i * 3);
        }

        // Shutdown should have completed
        shutdown_handle.await.unwrap();
    })
    .await;
}

#[tokio::test]
async fn test_shared_system_references() {
    crate::test_utils::with_test_runtime(|| async {
        // This test verifies that the new shutdown API works well with
        // multiple Arc references to the systems

        let runtime = crate::runtime::global().unwrap();
        let actor_system = runtime.actor_system();
        let task_system = runtime.task_system();

        // Create multiple references
        let ts_ref1 = task_system.clone();
        let ts_ref2 = task_system.clone();
        let as_ref1 = actor_system.clone();
        let as_ref2 = actor_system.clone();

        // Register function using one reference
        ts_ref1
            .register_function(
                "echo",
                task_function!(
                    |x: String| async move { Ok::<String, crate::error::RustyRayError>(x) }
                ),
            )
            .unwrap();

        // Submit task using another reference
        let result_ref = TaskBuilder::new("echo")
            .arg("Hello".to_string())
            .submit::<String>(&ts_ref2)
            .await
            .unwrap();

        let result = result_ref.get().await.unwrap();
        assert_eq!(result, "Hello");

        // Shutdown using yet another reference
        task_system.shutdown().await.unwrap();
        actor_system.shutdown().await.unwrap();

        // All references are still valid, but the systems are shut down
        // Trying to use them should fail gracefully
        let submit_result = TaskBuilder::new("echo")
            .arg("World".to_string())
            .submit::<String>(&ts_ref1)
            .await;
        assert!(submit_result.is_err());

        // Drop all references - no panic should occur
        drop(ts_ref1);
        drop(ts_ref2);
        drop(as_ref1);
        drop(as_ref2);
        let _ = task_system;
        let _ = actor_system;
    })
    .await;
}
