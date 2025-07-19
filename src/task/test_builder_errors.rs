//! Tests for TaskBuilder error handling

#[cfg(test)]
mod tests {
    use crate::actor::ActorSystem;
    use crate::task::{TaskBuilder, TaskSystem};
    use crate::task_function;
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;

    // A type that implements Serialize but can fail during serialization
    #[derive(Serialize, Deserialize)]
    struct LargeData {
        data: Vec<u8>,
    }

    #[tokio::test]
    async fn test_task_builder_error_propagation() {
        let actor_system = Arc::new(ActorSystem::new());
        let task_system = Arc::new(TaskSystem::new(actor_system.clone()));

        // Register a function that takes multiple args
        task_system
            .register_function(
                "multi_arg",
                task_function!(|x: i32, y: String| async move {
                    Ok::<String, crate::error::RustyRayError>(format!("{}: {}", x, y))
                }),
            )
            .unwrap();

        // Test with valid arguments
        let result = TaskBuilder::new("multi_arg")
            .arg(42)
            .arg("test".to_string())
            .submit::<String>(&task_system)
            .await;

        assert!(result.is_ok());
        let value = result.unwrap().get().await.unwrap();
        assert_eq!(value, "42: test");

        // Shutdown
        task_system.shutdown().await.unwrap();
        actor_system.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_missing_function_error() {
        let actor_system = Arc::new(ActorSystem::new());
        let task_system = Arc::new(TaskSystem::new(actor_system.clone()));

        // Submit a task for a non-existent function
        let result_ref = TaskBuilder::new("non_existent")
            .arg(42)
            .submit::<i32>(&task_system)
            .await;

        // Submission succeeds (returns ObjectRef)
        assert!(result_ref.is_ok());

        // But getting the result should fail
        let get_result = result_ref.unwrap().get().await;
        assert!(get_result.is_err());
        let error_msg = get_result.unwrap_err().to_string();
        assert!(error_msg.contains("not found"));

        // Shutdown
        task_system.shutdown().await.unwrap();
        actor_system.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_task_builder_build_method() {
        // Test the build() method directly with valid data
        let builder = TaskBuilder::new("test")
            .arg(42)
            .arg("hello".to_string())
            .num_cpus(2.0);

        let result = builder.build();
        assert!(result.is_ok());
        let spec = result.unwrap();
        // Verify the spec was built correctly
        assert_eq!(spec.args.len(), 2);
        assert_eq!(spec.resources.num_cpus, 2.0);
    }
}
