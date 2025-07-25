//! Tests for ObjectRef sharing between tasks

#[cfg(test)]
mod tests {
    use crate::task::TaskBuilder;
    use crate::task_function;
    use crate::test_utils::with_test_runtime;

    #[tokio::test]
    async fn test_objectref_sharing_between_tasks() {
        with_test_runtime(|| async {
            let runtime = crate::runtime::global().unwrap();
            let task_system = runtime.task_system();

            // Register functions
            task_system
                .register_function(
                    "test_sharing_produce_data",
                    task_function!(|| async move {
                        Ok::<Vec<i32>, crate::error::RustyRayError>(vec![1, 2, 3, 4, 5])
                    }),
                )
                .unwrap();

            task_system
                .register_function(
                    "test_sharing_sum",
                    task_function!(|data: Vec<i32>| async move {
                        Ok::<i32, crate::error::RustyRayError>(data.iter().sum())
                    }),
                )
                .unwrap();

            task_system
                .register_function(
                    "test_sharing_max",
                    task_function!(|data: Vec<i32>| async move {
                        Ok::<i32, crate::error::RustyRayError>(*data.iter().max().unwrap_or(&0))
                    }),
                )
                .unwrap();

            task_system
                .register_function(
                    "test_sharing_average",
                    task_function!(|data: Vec<i32>| async move {
                        let sum: i32 = data.iter().sum();
                        let avg = sum as f64 / data.len() as f64;
                        Ok::<f64, crate::error::RustyRayError>(avg)
                    }),
                )
                .unwrap();

            // Produce data once
            let data_ref = TaskBuilder::new("test_sharing_produce_data")
                .submit::<Vec<i32>>(task_system)
                .await
                .unwrap();

            // Clone the ObjectRef to share it between multiple tasks
            let data_ref_for_sum = data_ref.clone();
            let data_ref_for_max = data_ref.clone();
            let data_ref_for_avg = data_ref.clone();

            // Submit three tasks that all use the same data
            let sum_ref = TaskBuilder::new("test_sharing_sum")
                .arg_ref(&data_ref_for_sum)
                .submit::<i32>(task_system)
                .await
                .unwrap();

            let max_ref = TaskBuilder::new("test_sharing_max")
                .arg_ref(&data_ref_for_max)
                .submit::<i32>(task_system)
                .await
                .unwrap();

            let avg_ref = TaskBuilder::new("test_sharing_average")
                .arg_ref(&data_ref_for_avg)
                .submit::<f64>(task_system)
                .await
                .unwrap();

            // Get all results
            let sum = sum_ref.get().await.unwrap();
            let max = max_ref.get().await.unwrap();
            let avg = avg_ref.get().await.unwrap();

            assert_eq!(sum, 15); // 1+2+3+4+5
            assert_eq!(max, 5);
            assert_eq!(avg, 3.0);

            // We can also still get the original data
            let data = data_ref.get().await.unwrap();
            assert_eq!(data, vec![1, 2, 3, 4, 5]);
        })
        .await;
    }

    #[tokio::test]
    async fn test_objectref_concurrent_access() {
        with_test_runtime(|| async {
            let runtime = crate::runtime::global().unwrap();
            let task_system = runtime.task_system();

            // Register a slow function
            task_system
                .register_function(
                    "test_concurrent_slow_compute",
                    task_function!(|| async move {
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                        Ok::<i32, crate::error::RustyRayError>(42)
                    }),
                )
                .unwrap();

            // Submit the task
            let result_ref = TaskBuilder::new("test_concurrent_slow_compute")
                .submit::<i32>(task_system)
                .await
                .unwrap();

            // Clone multiple times
            let refs: Vec<_> = (0..10).map(|_| result_ref.clone()).collect();

            // Access all clones concurrently
            let mut handles = vec![];
            for (i, ref_) in refs.into_iter().enumerate() {
                handles.push(tokio::spawn(async move {
                    let result = ref_.get().await.unwrap();
                    (i, result)
                }));
            }

            // All should get the same result
            for handle in handles {
                let (i, result) = handle.await.unwrap();
                assert_eq!(result, 42, "Clone {i} got wrong result");
            }

            // Original should also work
            let original_result = result_ref.get().await.unwrap();
            assert_eq!(original_result, 42);
        })
        .await;
    }
}
