//! Integration tests for actor-task data sharing via object store

use async_trait::async_trait;
use rustyray_core::{
    actor::Actor, ray, runtime, task::TaskBuilder, task_function, ObjectRef, Result, RustyRayError,
};
use serde::{Deserialize, Serialize};
use std::any::Any;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct SharedData {
    id: u32,
    values: Vec<i32>,
}

/// Actor that creates data and stores it in object store
struct DataProducer {
    id: u32,
}

#[async_trait]
impl Actor for DataProducer {
    async fn handle(&mut self, msg: Box<dyn Any + Send>) -> Result<Box<dyn Any + Send>> {
        if msg
            .downcast_ref::<String>()
            .map(|s| s == "create")
            .unwrap_or(false)
        {
            // Create some data
            let data = SharedData {
                id: self.id,
                values: vec![1, 2, 3, 4, 5]
                    .into_iter()
                    .map(|x| x * self.id as i32)
                    .collect(),
            };

            // Store in object store
            let obj_ref = ray::put(data).await.unwrap();
            Ok(Box::new(obj_ref))
        } else {
            Err(RustyRayError::Internal("Unknown message".to_string()))
        }
    }
}

/// Actor that consumes data from object store
struct DataConsumer {
    processed_count: u32,
}

#[async_trait]
impl Actor for DataConsumer {
    async fn handle(&mut self, msg: Box<dyn Any + Send>) -> Result<Box<dyn Any + Send>> {
        if let Some(obj_ref) = msg.downcast_ref::<rustyray_core::ObjectRef<SharedData>>() {
            // Get data from object store
            let data = ray::get(obj_ref).await.unwrap();
            self.processed_count += 1;

            // Process data (sum values)
            let sum: i32 = data.values.iter().sum();
            Ok(Box::new(sum))
        } else {
            Err(RustyRayError::Internal(
                "Expected ObjectRef<SharedData>".to_string(),
            ))
        }
    }
}

#[tokio::test]
async fn test_actor_creates_task_consumes() {
    use rustyray_core::test_utils::with_test_runtime;

    with_test_runtime(|| async {
        let runtime = runtime::global().unwrap();
        let task_system = runtime.task_system();

        // First test: Simple task without actor involvement
        task_system
            .register_function(
                "simple_double",
                task_function!(|x: i32| async move { Ok::<i32, RustyRayError>(x * 2) }),
            )
            .unwrap();

        let simple_ref: ObjectRef<i32> = task_system
            .submit(TaskBuilder::new("simple_double").arg(42).build().unwrap())
            .await
            .unwrap();

        let simple_result = simple_ref.get().await.unwrap();
        assert_eq!(simple_result, 84);

        // Now the original test with actor
        let actor_system = runtime.actor_system();

        // Create actor
        let producer = actor_system
            .create_actor(DataProducer { id: 10 })
            .await
            .unwrap();

        // Actor creates data
        let response = producer.call(Box::new("create".to_string())).await.unwrap();
        let obj_ref = response
            .downcast_ref::<rustyray_core::ObjectRef<SharedData>>()
            .expect("Expected ObjectRef")
            .clone();

        // Register a task function that processes the data
        task_system
            .register_function(
                "process_data",
                task_function!(|data: SharedData| async move {
                    let doubled: Vec<i32> = data.values.iter().map(|x| x * 2).collect();
                    Ok::<Vec<i32>, RustyRayError>(doubled)
                }),
            )
            .unwrap();

        // Execute task with ObjectRef argument
        let task_ref: ObjectRef<Vec<i32>> = task_system
            .submit(
                TaskBuilder::new("process_data")
                    .arg_ref(&obj_ref)
                    .build()
                    .unwrap(),
            )
            .await
            .unwrap();

        // Get task result
        let doubled_values = task_ref.get().await.unwrap();
        assert_eq!(doubled_values, vec![20, 40, 60, 80, 100]);
    })
    .await;
}

#[tokio::test]
async fn test_task_creates_actor_consumes() {
    use rustyray_core::test_utils::with_test_runtime;

    with_test_runtime(|| async {
        let runtime = runtime::global().unwrap();
        let actor_system = runtime.actor_system();
        let task_system = runtime.task_system();

        // Register a task that creates data
        task_system
            .register_function(
                "create_data",
                task_function!(|| async move {
                    let data = SharedData {
                        id: 20,
                        values: vec![5, 10, 15, 20, 25],
                    };
                    Ok::<SharedData, RustyRayError>(data)
                }),
            )
            .unwrap();

        // Execute task
        let task_ref: ObjectRef<SharedData> = task_system
            .submit(TaskBuilder::new("create_data").build().unwrap())
            .await
            .unwrap();

        // Create consumer actor
        let consumer = actor_system
            .create_actor(DataConsumer { processed_count: 0 })
            .await
            .unwrap();

        // Actor consumes task result
        let response = consumer.call(Box::new(task_ref)).await.unwrap();
        let sum = *response.downcast_ref::<i32>().expect("Expected i32");
        assert_eq!(sum, 75); // 5 + 10 + 15 + 20 + 25
    })
    .await;
}

#[tokio::test]
async fn test_shared_object_store() {
    use rustyray_core::test_utils::with_test_runtime;

    with_test_runtime(|| async {
        let runtime = runtime::global().unwrap();
        let task_system = runtime.task_system();

        // Test 1: Put data using ray API and retrieve with task
        let data = SharedData {
            id: 30,
            values: vec![100, 200, 300],
        };
        let obj_ref = ray::put(data.clone()).await.unwrap();

        // Verify we can get it back directly
        let retrieved = ray::get(&obj_ref).await.unwrap();
        assert_eq!(retrieved.id, 30);

        // Register task to read the data
        task_system
            .register_function(
                "read_data_id",
                task_function!(|data: SharedData| async move { Ok::<u32, RustyRayError>(data.id) }),
            )
            .unwrap();

        let task_ref: ObjectRef<u32> = task_system
            .submit(
                TaskBuilder::new("read_data_id")
                    .arg_ref(&obj_ref)
                    .build()
                    .unwrap(),
            )
            .await
            .unwrap();

        let id = task_ref.get().await.unwrap();
        assert_eq!(id, 30);
    })
    .await;
}

#[tokio::test]
async fn test_batch_operations() {
    use rustyray_core::test_utils::with_test_runtime;

    with_test_runtime(|| async {
        // Put multiple values
        let values = vec![10, 20, 30, 40, 50];
        let refs = ray::put_batch(values.clone()).await.unwrap();
        assert_eq!(refs.len(), 5);

        // Get them back
        let retrieved = ray::get_batch(&refs).await.unwrap();
        assert_eq!(retrieved, values);

        // Wait for them (should be immediate since we just put them)
        ray::wait(&refs).await.unwrap();
    })
    .await;
}

#[tokio::test]
async fn test_concurrent_actors_and_tasks() {
    use rustyray_core::test_utils::with_test_runtime;

    with_test_runtime(|| async {
        let runtime = runtime::global().unwrap();
        let actor_system = runtime.actor_system();
        let task_system = runtime.task_system();

        // Create multiple actors concurrently
        let mut actor_handles = vec![];
        for i in 0..5 {
            let actor_system = actor_system.clone();
            let handle = tokio::spawn(async move {
                let producer = actor_system
                    .create_actor(DataProducer { id: i as u32 })
                    .await
                    .unwrap();

                // Each actor creates data
                let response = producer.call(Box::new("create".to_string())).await.unwrap();
                let obj_ref = response
                    .downcast_ref::<ObjectRef<SharedData>>()
                    .expect("Expected ObjectRef")
                    .clone();

                (i, obj_ref)
            });
            actor_handles.push(handle);
        }

        // Collect all ObjectRefs from actors
        let mut object_refs = vec![];
        for handle in actor_handles {
            let (idx, obj_ref) = handle.await.unwrap();
            object_refs.push((idx, obj_ref));
        }

        // Register a task function
        task_system
            .register_function(
                "sum_values",
                task_function!(|data: SharedData| async move {
                    let sum: i32 = data.values.iter().sum();
                    Ok::<i32, RustyRayError>(sum)
                }),
            )
            .unwrap();

        // Submit concurrent tasks to process each actor's data
        let mut task_handles = vec![];
        for (idx, obj_ref) in object_refs {
            let task_system = task_system.clone();
            let handle = tokio::spawn(async move {
                let task_ref: ObjectRef<i32> = task_system
                    .submit(
                        TaskBuilder::new("sum_values")
                            .arg_ref(&obj_ref)
                            .build()
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                let sum = task_ref.get().await.unwrap();
                (idx, sum)
            });
            task_handles.push(handle);
        }

        // Collect all results
        let mut results = vec![];
        for handle in task_handles {
            results.push(handle.await.unwrap());
        }

        // Verify results
        results.sort_by_key(|(idx, _)| *idx);
        assert_eq!(results.len(), 5);

        // Each actor with id i creates values [i, 2*i, 3*i, 4*i, 5*i]
        // So sum should be i * (1+2+3+4+5) = i * 15
        for (idx, sum) in results {
            assert_eq!(sum, (idx as i32) * 15);
        }
    })
    .await;
}
