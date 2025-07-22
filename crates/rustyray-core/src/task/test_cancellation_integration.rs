//! Integration tests for task cancellation

#[cfg(test)]
mod tests {
    use crate::actor::ActorSystem;
    use crate::task::{TaskBuilder, TaskSystem};
    use crate::task_function;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::time;

    #[tokio::test]
    async fn test_task_timeout() {
        let actor_system = Arc::new(ActorSystem::new());
        // Use test configuration with aggressive timeouts
        let task_system = Arc::new(TaskSystem::for_tests(actor_system.clone()));

        // Register a slow function
        task_system
            .register_function(
                "slow_task",
                task_function!(|delay_ms: u64| async move {
                    time::sleep(Duration::from_millis(delay_ms)).await;
                    Ok::<String, crate::error::RustyRayError>("completed".to_string())
                }),
            )
            .unwrap();

        // Task that completes in time
        let fast_ref = TaskBuilder::new("slow_task")
            .arg(50u64) // 50ms < 100ms timeout
            .submit::<String>(&task_system)
            .await
            .unwrap();

        let result = fast_ref.get().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "completed");

        // Task that times out
        let slow_ref = TaskBuilder::new("slow_task")
            .arg(200u64) // 200ms > 100ms timeout
            .submit::<String>(&task_system)
            .await
            .unwrap();

        // No sleep needed - get() will return when task is cancelled
        // Should get timeout error
        let result = slow_ref.get().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cancelled"));

        // Shutdown
        task_system.shutdown().await.unwrap();
        actor_system.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_long_running_task_cancellation() {
        let actor_system = Arc::new(ActorSystem::new());
        // Use test configuration with aggressive timeouts
        let task_system = Arc::new(TaskSystem::for_tests(actor_system.clone()));

        // Register a function that checks for cancellation
        task_system
            .register_function(
                "long_running",
                task_function!(|| async move {
                    // Simulate a long running task that periodically yields
                    for i in 0..10 {
                        time::sleep(Duration::from_millis(50)).await;
                        println!("Long running task iteration {i}");
                    }
                    Ok::<String, crate::error::RustyRayError>("Should not complete".to_string())
                }),
            )
            .unwrap();

        // Submit the long running task
        let task_ref = TaskBuilder::new("long_running")
            .submit::<String>(&task_system)
            .await
            .unwrap();

        // The task should be cancelled after timeout
        let result = task_ref.get().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cancelled"));

        // Shutdown
        task_system.shutdown().await.unwrap();
        actor_system.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_no_timeout_for_fast_tasks() {
        let actor_system = Arc::new(ActorSystem::new());
        // For this test, use a longer timeout to ensure fast tasks complete
        let task_system = Arc::new(TaskSystem::with_timeout(
            actor_system.clone(),
            Duration::from_secs(1),
        ));

        // Register many fast functions
        for i in 0..20 {
            let name = format!("fast_{i}");
            task_system
                .register_function(
                    name,
                    task_function!(|x: i32| async move {
                        // Very fast computation
                        Ok::<i32, crate::error::RustyRayError>(x * 2)
                    }),
                )
                .unwrap();
        }

        // Submit many fast tasks
        let mut refs = vec![];
        for i in 0..20 {
            let name = format!("fast_{i}");
            let ref_ = TaskBuilder::new(name)
                .arg(i)
                .submit::<i32>(&task_system)
                .await
                .unwrap();
            refs.push((i, ref_));
        }

        // All should complete successfully
        for (i, ref_) in refs {
            let result = ref_.get().await.unwrap();
            assert_eq!(result, i * 2);
        }

        // Shutdown
        task_system.shutdown().await.unwrap();
        actor_system.shutdown().await.unwrap();
    }
}
