//! Integration tests for actor-task data sharing using macros

use async_trait::async_trait;
use rustyray_core::{actor::Actor, ray, ObjectRef, Result, RustyRayError};
use serde::{Deserialize, Serialize};
use std::any::Any;

// Use the test utils instead of custom implementation
use rustyray_core::test_utils::with_test_runtime;

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
            let obj_ref = ray::put(data).await?;
            Ok(Box::new(obj_ref))
        } else {
            Err(RustyRayError::Internal("Unknown message".to_string()))
        }
    }
}

#[tokio::test]
async fn test_actor_creates_task_consumes() {
    with_test_runtime(|| async {
        let runtime = rustyray_core::runtime::global().unwrap();
        let actor_system = runtime.actor_system();
        let task_system = runtime.task_system();

        // Register the task function
        // Note: When using arg_ref, the ObjectRef is resolved to its value before the task runs
        task_system
            .register_function(
                "process_data",
                rustyray_core::task_function!(|data: SharedData| async move {
                    let doubled: Vec<i32> = data.values.iter().map(|x| x * 2).collect();
                    Ok::<Vec<i32>, RustyRayError>(doubled)
                }),
            )
            .unwrap();

        // Create actor
        let producer = actor_system
            .create_actor(DataProducer { id: 10 })
            .await
            .unwrap();

        // Actor creates data
        let response = producer.call(Box::new("create".to_string())).await.unwrap();
        let obj_ref = response
            .downcast_ref::<ObjectRef<SharedData>>()
            .expect("Expected ObjectRef")
            .clone();

        println!("Actor created object with ID: {:?}", obj_ref.id());

        // Task processes the data
        let task_ref = rustyray_core::TaskBuilder::new("process_data")
            .arg_ref(&obj_ref)
            .submit::<Vec<i32>>(&task_system)
            .await
            .unwrap();
        let doubled_values = task_ref.get().await.unwrap();
        assert_eq!(doubled_values, vec![20, 40, 60, 80, 100]);
    })
    .await;
}

#[tokio::test]
async fn test_task_creates_actor_consumes() {
    with_test_runtime(|| async {
        let runtime = rustyray_core::runtime::global().unwrap();
        let actor_system = runtime.actor_system();
        let task_system = runtime.task_system();

        // Register the task function
        task_system
            .register_function(
                "create_data",
                rustyray_core::task_function!(|id: u32| async move {
                    Ok::<SharedData, RustyRayError>(SharedData {
                        id,
                        values: vec![5, 10, 15, 20, 25],
                    })
                }),
            )
            .unwrap();

        // Task creates data
        let task_ref = rustyray_core::TaskBuilder::new("create_data")
            .arg(20u32)
            .submit::<SharedData>(&task_system)
            .await
            .unwrap();

        // Actor that consumes data from object store
        struct DataConsumer {
            processed_count: u32,
        }

        #[async_trait]
        impl Actor for DataConsumer {
            async fn handle(&mut self, msg: Box<dyn Any + Send>) -> Result<Box<dyn Any + Send>> {
                if let Some(obj_ref) = msg.downcast_ref::<ObjectRef<SharedData>>() {
                    // Get data from object store
                    let data = ray::get(obj_ref).await?;
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
async fn test_shared_via_ray_api() {
    with_test_runtime(|| async {
        // Put data using ray API
        let data = SharedData {
            id: 30,
            values: vec![100, 200, 300],
        };
        let obj_ref = ray::put(data.clone()).await.unwrap();

        // Both actors and tasks can access it
        let retrieved = ray::get(&obj_ref).await.unwrap();
        assert_eq!(retrieved, data);

        // Batch operations
        let values = vec![10, 20, 30, 40, 50];
        let refs = ray::put_batch(values.clone()).await.unwrap();
        assert_eq!(refs.len(), 5);

        let retrieved = ray::get_batch(&refs).await.unwrap();
        assert_eq!(retrieved, values);
    })
    .await;
}
