//! Integration tests for actor-task data sharing via object store

use async_trait::async_trait;
use rustyray_core::{
    actor::Actor, object_store::ObjectStore, ray, runtime, task_function, Result, RustyRayError,
    TaskBuilder,
};
use serde::{Deserialize, Serialize};
use std::any::Any;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct SharedData {
    id: u32,
    values: Vec<f64>,
}

impl SharedData {
    fn sum(&self) -> f64 {
        self.values.iter().sum()
    }
}

/// Actor that produces data and stores it in object store
struct Producer {
    id: u32,
}

#[async_trait]
impl Actor for Producer {
    async fn handle(&mut self, msg: Box<dyn Any + Send>) -> Result<Box<dyn Any + Send>> {
        if let Some(&size) = msg.downcast_ref::<usize>() {
            // Create data
            let data = SharedData {
                id: self.id,
                values: (0..size).map(|i| i as f64).collect(),
            };

            // Store in object store
            let obj_ref = ray::put(data).await?;
            Ok(Box::new(obj_ref))
        } else {
            Err(RustyRayError::Internal("Invalid message".to_string()))
        }
    }
}

/// Actor that consumes data from object store
struct Consumer {
    processed_count: usize,
}

#[async_trait]
impl Actor for Consumer {
    async fn handle(&mut self, msg: Box<dyn Any + Send>) -> Result<Box<dyn Any + Send>> {
        if let Some(obj_ref) = msg.downcast_ref::<rustyray_core::ObjectRef<SharedData>>() {
            // Get data from object store
            let data = ray::get(obj_ref).await?;
            let sum = data.sum();

            self.processed_count += 1;
            Ok(Box::new(sum))
        } else {
            Err(RustyRayError::Internal("Invalid message".to_string()))
        }
    }
}

#[tokio::test]
async fn test_actor_to_actor_sharing() {
    use rustyray_core::test_utils::with_test_runtime;

    with_test_runtime(|| async {
        let runtime = runtime::global().unwrap();
        let actor_system = runtime.actor_system();

        // Create actors
        let producer = actor_system.create_actor(Producer { id: 1 }).await.unwrap();
        let consumer = actor_system
            .create_actor(Consumer { processed_count: 0 })
            .await
            .unwrap();

        // Producer creates data
        let response = producer.call(Box::new(10usize)).await.unwrap();
        let obj_ref = response
            .downcast_ref::<rustyray_core::ObjectRef<SharedData>>()
            .unwrap()
            .clone();

        // Consumer processes data
        let response = consumer.call(Box::new(obj_ref.clone())).await.unwrap();
        let sum = *response.downcast_ref::<f64>().unwrap();
        assert_eq!(sum, 45.0); // Sum of 0..10

        // Another consumer can also access the same data
        let consumer2 = actor_system
            .create_actor(Consumer { processed_count: 0 })
            .await
            .unwrap();
        let response2 = consumer2.call(Box::new(obj_ref)).await.unwrap();
        let sum2 = *response2.downcast_ref::<f64>().unwrap();
        assert_eq!(sum2, 45.0);
    })
    .await;
}

#[tokio::test]
async fn test_task_to_actor_sharing() {
    use rustyray_core::test_utils::with_test_runtime;

    with_test_runtime(|| async {
        let runtime = runtime::global().unwrap();
        let actor_system = runtime.actor_system();
        let task_system = runtime.task_system();

        // Register a task that creates data
        task_system
            .register_function(
                "create_data",
                task_function!(|size: usize| async move {
                    let data = SharedData {
                        id: 100,
                        values: (0..size).map(|i| i as f64 * 2.0).collect(),
                    };
                    Ok::<SharedData, RustyRayError>(data)
                }),
            )
            .unwrap();

        // Task creates data
        let task_ref = TaskBuilder::new("create_data")
            .arg(5usize)
            .submit::<SharedData>(task_system)
            .await
            .unwrap();

        // Store task result in object store
        let data = task_ref.get().await.unwrap();
        let obj_ref = ray::put(data).await.unwrap();

        // Actor consumes data created by task
        let consumer = actor_system
            .create_actor(Consumer { processed_count: 0 })
            .await
            .unwrap();
        let response = consumer.call(Box::new(obj_ref)).await.unwrap();
        let sum = *response.downcast_ref::<f64>().unwrap();
        assert_eq!(sum, 20.0); // Sum of [0, 2, 4, 6, 8]
    })
    .await;
}

#[tokio::test]
async fn test_actor_to_task_sharing() {
    use rustyray_core::test_utils::with_test_runtime;

    with_test_runtime(|| async {
        let runtime = runtime::global().unwrap();
        let actor_system = runtime.actor_system();
        let task_system = runtime.task_system();

        // Register a task that processes data
        task_system
            .register_function(
                "process_data",
                task_function!(|data: SharedData| async move {
                    let sum = data.sum();
                    let avg = sum / data.values.len() as f64;
                    Ok::<f64, RustyRayError>(avg)
                }),
            )
            .unwrap();

        // Actor creates data
        let producer = actor_system.create_actor(Producer { id: 2 }).await.unwrap();
        let response = producer.call(Box::new(20usize)).await.unwrap();
        let obj_ref = response
            .downcast_ref::<rustyray_core::ObjectRef<SharedData>>()
            .unwrap()
            .clone();

        // Get data from object store to pass to task
        let data = ray::get(&obj_ref).await.unwrap();

        // Task processes data created by actor
        let task_ref = TaskBuilder::new("process_data")
            .arg(data)
            .submit::<f64>(task_system)
            .await
            .unwrap();

        let avg = task_ref.get().await.unwrap();
        assert_eq!(avg, 9.5); // Average of 0..20
    })
    .await;
}

#[tokio::test]
async fn test_batch_sharing() {
    use rustyray_core::test_utils::with_test_runtime;

    with_test_runtime(|| async {
        let runtime = runtime::global().unwrap();

        // Create multiple data objects
        let data_vec: Vec<SharedData> = (0..5)
            .map(|i| SharedData {
                id: i,
                values: vec![i as f64; 3],
            })
            .collect();

        // Store all in batch
        let refs = ray::put_batch(data_vec.clone()).await.unwrap();
        assert_eq!(refs.len(), 5);

        // Retrieve all in batch
        let retrieved = ray::get_batch(&refs).await.unwrap();
        assert_eq!(retrieved, data_vec);

        // Object store should have all objects
        let stats = runtime.object_store().stats().await;
        assert!(stats.total_objects >= 5);
    })
    .await;
}

#[tokio::test]
async fn test_cross_system_coordination() {
    use rustyray_core::test_utils::with_test_runtime;

    with_test_runtime(|| async {
        let runtime = runtime::global().unwrap();
        let actor_system = runtime.actor_system();
        let task_system = runtime.task_system();

        // Register a coordinator task
        task_system
            .register_function(
                "coordinate",
                task_function!(|count: usize| async move {
                    // Create multiple data objects
                    let mut refs = Vec::new();
                    for i in 0..count {
                        let data = SharedData {
                            id: i as u32,
                            values: vec![i as f64 * 10.0; 5],
                        };
                        refs.push(ray::put(data).await.unwrap());
                    }
                    Ok::<Vec<rustyray_core::ObjectRef<SharedData>>, RustyRayError>(refs)
                }),
            )
            .unwrap();

        // Task creates multiple objects
        let task_ref = TaskBuilder::new("coordinate")
            .arg(3usize)
            .submit::<Vec<rustyray_core::ObjectRef<SharedData>>>(task_system)
            .await
            .unwrap();

        let refs = task_ref.get().await.unwrap();
        assert_eq!(refs.len(), 3);

        // Multiple actors process the objects
        let mut consumers = Vec::new();
        for _ in 0..3 {
            let consumer = actor_system
                .create_actor(Consumer { processed_count: 0 })
                .await
                .unwrap();
            consumers.push(consumer);
        }

        // Each consumer processes one object
        let mut results = Vec::new();
        for (i, consumer) in consumers.iter().enumerate() {
            let response = consumer.call(Box::new(refs[i].clone())).await.unwrap();
            let sum = *response.downcast_ref::<f64>().unwrap();
            results.push(sum);
        }

        assert_eq!(results, vec![0.0, 50.0, 100.0]);
    })
    .await;
}
