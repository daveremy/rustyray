//! Demonstrates the object store functionality in RustyRay.
//!
//! This example shows how the object store enables:
//! - Efficient data sharing between tasks
//! - Memory management with LRU eviction
//! - Object pinning to prevent eviction
//! - Type-safe storage and retrieval
//! - Ray-style put/get API

use rustyray_core::{
    object_store::{EvictionPolicy, InMemoryStore, ObjectStore, StoreConfig},
    ray, runtime, ActorSystem, Result, TaskSystem,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DataSet {
    name: String,
    values: Vec<f64>,
}

impl DataSet {
    fn new(name: &str, size: usize) -> Self {
        let values = (0..size).map(|i| i as f64 * 1.1).collect();
        Self {
            name: name.to_string(),
            values,
        }
    }

    fn compute_mean(&self) -> f64 {
        if self.values.is_empty() {
            0.0
        } else {
            self.values.iter().sum::<f64>() / self.values.len() as f64
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== RustyRay Object Store Demo ===\n");

    // Initialize the runtime (required for ray module)
    runtime::init()?;

    // Demonstrate direct object store usage
    demonstrate_object_store().await?;

    // Demonstrate Ray-style API
    demonstrate_ray_api().await?;

    // Demonstrate object store with tasks
    let runtime = runtime::global()?;
    demonstrate_with_tasks(runtime.task_system()).await?;

    // Shutdown cleanly
    runtime::shutdown()?;

    println!("\nDemo completed successfully!");
    Ok(())
}

async fn demonstrate_object_store() -> Result<()> {
    println!("1. Direct Object Store Usage");
    println!("----------------------------");

    // Create a store with custom configuration
    let config = StoreConfig {
        max_memory: 1024 * 1024 * 10, // 10MB
        eviction_policy: EvictionPolicy::LRU,
        enable_stats: true,
    };
    let store = Arc::new(InMemoryStore::new(config));

    // Store some datasets
    println!("Storing datasets...");
    let dataset1 = DataSet::new("training_data", 1000);
    let dataset2 = DataSet::new("validation_data", 500);

    let result1 = store.put(dataset1.clone()).await?;
    let result2 = store.put(dataset2.clone()).await?;

    println!(
        "  - Stored {} (size: {} bytes)",
        dataset1.name, result1.size
    );
    println!(
        "  - Stored {} (size: {} bytes)",
        dataset2.name, result2.size
    );

    // Retrieve and process
    println!("\nRetrieving and processing...");
    if let Some(data) = store.get::<DataSet>(result1.id).await? {
        println!(
            "  - Retrieved: {}, mean value: {:.2}",
            data.name,
            data.compute_mean()
        );
    }

    // Pin important data
    println!("\nPinning important data...");
    store.pin(result1.id).await?;
    println!("  - Pinned {} to prevent eviction", dataset1.name);

    // Show statistics
    let stats = store.stats().await;
    println!("\nObject Store Statistics:");
    println!("  - Total objects: {}", stats.total_objects);
    println!("  - Total size: {} bytes", stats.total_size);
    println!("  - Cache hits: {}", stats.hit_count);
    println!("  - Cache misses: {}", stats.miss_count);
    println!("  - Pinned objects: {}", stats.pinned_count);

    Ok(())
}

async fn demonstrate_ray_api() -> Result<()> {
    println!("\n2. Ray-style API Usage");
    println!("----------------------");

    // Store data using ray.put()
    println!("Using ray::put() to store data...");
    let dataset = DataSet::new("ray_dataset", 2000);
    let obj_ref = ray::put(dataset.clone()).await?;
    println!("  - Stored dataset '{}' using ray::put()", dataset.name);

    // Retrieve data using ray.get()
    println!("\nUsing ray::get() to retrieve data...");
    let retrieved: DataSet = ray::get(&obj_ref).await?;
    println!(
        "  - Retrieved: {}, mean value: {:.2}",
        retrieved.name,
        retrieved.compute_mean()
    );

    // Batch operations
    println!("\nBatch operations:");
    let datasets = vec![
        DataSet::new("batch_1", 100),
        DataSet::new("batch_2", 200),
        DataSet::new("batch_3", 300),
    ];

    let refs = ray::put_batch(datasets.clone()).await?;
    println!("  - Stored {} datasets using ray::put_batch()", refs.len());

    let retrieved_batch: Vec<DataSet> = ray::get_batch(&refs).await?;
    println!(
        "  - Retrieved {} datasets using ray::get_batch()",
        retrieved_batch.len()
    );

    for (i, dataset) in retrieved_batch.iter().enumerate() {
        println!("    - {}: {} items", dataset.name, dataset.values.len());
    }

    // Using wait
    println!("\nUsing ray::wait():");
    ray::wait(&refs).await?;
    println!("  - All objects are ready!");

    Ok(())
}

async fn demonstrate_with_tasks(task_system: &TaskSystem) -> Result<()> {
    println!("\n3. Object Store with Task System");
    println!("--------------------------------");

    // Register a processing function
    task_system.register_function("process_dataset", |args, _context| {
        Box::pin(async move {
            if args.len() != 1 {
                return Err(rustyray_core::RustyRayError::Internal(
                    "Expected 1 argument".to_string(),
                ));
            }

            // For this example, we'll just return a simple result
            // In a real implementation, you would deserialize the dataset from args[0]
            let result = "Dataset processed successfully".to_string();

            // Serialize the result using bincode
            bincode::serde::encode_to_vec(&result, bincode::config::standard()).map_err(|e| {
                rustyray_core::RustyRayError::Internal(format!("Serialization failed: {:?}", e))
            })
        })
    })?;

    // Create large datasets using ray API
    println!("Creating and storing large datasets...");
    let large_dataset = DataSet::new("large_dataset", 10000);
    let dataset_ref = ray::put(large_dataset).await?;
    println!("  - Created ObjectRef for large dataset using ray::put()");

    // Submit processing task using TaskBuilder
    println!("\nSubmitting processing task...");
    use rustyray_core::TaskBuilder;

    // For now, we'll pass the dataset value directly since ObjectRef
    // deserialization in tasks is still being integrated
    let result_ref = TaskBuilder::new("process_dataset")
        .arg(DataSet::new("task_dataset", 100)) // Pass value for now
        .submit::<String>(&task_system)
        .await?;

    // The dataset can be used by multiple tasks without copying
    println!("  - Task submitted, waiting for result...");
    let result: String = result_ref.get().await?;
    println!("  - {}", result);

    // Demonstrate that the original dataset is still available
    println!("\nOriginal dataset still available:");
    let original_data: DataSet = ray::get(&dataset_ref).await?;
    println!(
        "  - Retrieved '{}' with {} items",
        original_data.name,
        original_data.values.len()
    );

    Ok(())
}
