//! Test ObjectRef passing between tasks

#[cfg(test)]
mod tests {
    use crate::{
        object_ref::ObjectRef, runtime, task::TaskBuilder, task_function,
        test_utils::with_test_runtime,
    };

    #[tokio::test]
    async fn test_objectref_as_task_argument() {
        with_test_runtime(|| async {
            // Get runtime and task system
            let runtime = runtime::global().unwrap();
            let task_system = runtime.task_system();

            // Register a function that takes the resolved value (not the ObjectRef)
            // This matches Ray's behavior where ObjectRef arguments are resolved before the task runs
            task_system
                .register_function(
                    "test_passing_double_from_ref",
                    task_function!(|value: i32| async move {
                        Ok::<i32, crate::error::RustyRayError>(value * 2)
                    }),
                )
                .unwrap();

            // Put a value in the object store
            let input_ref = task_system.put(42i32).await.unwrap();

            // Submit task that uses the ObjectRef
            let result_ref: ObjectRef<i32> = TaskBuilder::new("test_passing_double_from_ref")
                .arg_ref(&input_ref)
                .submit(task_system)
                .await
                .unwrap();

            // Get result
            let result = result_ref.get().await.unwrap();
            assert_eq!(result, 84);
        })
        .await;
    }

    #[tokio::test]
    async fn test_chained_objectref_passing() {
        with_test_runtime(|| async {
            // Get runtime and task system
            let runtime = runtime::global().unwrap();
            let task_system = runtime.task_system();

            // Register functions that take resolved values
            task_system
                .register_function(
                    "test_chained_add_ten",
                    task_function!(|value: i32| async move {
                        Ok::<i32, crate::error::RustyRayError>(value + 10)
                    }),
                )
                .unwrap();

            task_system
                .register_function(
                    "test_chained_multiply_three",
                    task_function!(|value: i32| async move {
                        Ok::<i32, crate::error::RustyRayError>(value * 3)
                    }),
                )
                .unwrap();

            // Chain operations: 5 -> +10 -> *3 = 45
            let initial = task_system.put(5i32).await.unwrap();

            let added = TaskBuilder::new("test_chained_add_ten")
                .arg_ref(&initial)
                .submit::<i32>(task_system)
                .await
                .unwrap();

            let multiplied = TaskBuilder::new("test_chained_multiply_three")
                .arg_ref(&added)
                .submit::<i32>(task_system)
                .await
                .unwrap();

            let result = multiplied.get().await.unwrap();
            assert_eq!(result, 45);
        })
        .await;
    }
}
