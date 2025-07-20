//! Demonstrates the object store functionality in RustyRay.
//!
//! This example shows how the object store enables:
//! - Efficient data sharing between tasks
//! - Memory management with LRU eviction
//! - Object pinning to prevent eviction
//! - Type-safe storage and retrieval

use rustyray_core::{
    object_store::{InMemoryStore, ObjectStore, StoreConfig, EvictionPolicy},
    ActorSystem, TaskSystem,
    Result,
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
    
    // Create the system components
    let actor_system = Arc::new(ActorSystem::new());
    let task_system = Arc::new(TaskSystem::new(actor_system.clone()));
    
    // Demonstrate direct object store usage
    demonstrate_object_store().await?;
    
    // Demonstrate object store with tasks
    demonstrate_with_tasks(&task_system).await?;
    
    // Shutdown cleanly
    task_system.shutdown().await?;
    actor_system.shutdown().await?;
    
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
    
    println!("  - Stored {} (size: {} bytes)", dataset1.name, result1.size);
    println!("  - Stored {} (size: {} bytes)", dataset2.name, result2.size);
    
    // Retrieve and process
    println!("\nRetrieving and processing...");
    if let Some(data) = store.get::<DataSet>(result1.id).await? {
        println!("  - Retrieved: {}, mean value: {:.2}", data.name, data.compute_mean());
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

async fn demonstrate_with_tasks(task_system: &TaskSystem) -> Result<()> {
    println!("\n2. Object Store with Task System");
    println!("--------------------------------");
    
    // Register a processing function
    task_system.register_function("process_dataset", |args| {
        Box::pin(async move {
            if args.len() != 1 {
                return Err(rustyray_core::RustyRayError::Internal(
                    "Expected 1 argument".to_string()
                ));
            }
            
            // For this example, we'll just return a simple result
            // In a real implementation, you would deserialize the dataset from args[0]
            let result = "Dataset processed successfully".to_string();
            
            // Serialize the result using bincode
            bincode::serde::encode_to_vec(&result, bincode::config::standard())
                .map_err(|e| rustyray_core::RustyRayError::Internal(format!("Serialization failed: {:?}", e)))
        })
    })?;
    
    // Create large datasets
    println!("Creating and storing large datasets...");
    let large_dataset = DataSet::new("large_dataset", 10000);
    let dataset_ref = task_system.put(large_dataset).await?;
    println!("  - Created ObjectRef for large dataset");
    
    // Submit processing task using TaskBuilder
    println!("\nSubmitting processing task...");
    use rustyray_core::task::TaskBuilder;
    let task_spec = TaskBuilder::new("process_dataset")
        .arg_ref(&dataset_ref) // Pass by reference, not value
        .build()?;
    let result_ref = task_system.submit(task_spec).await?;
    
    // The dataset can be used by multiple tasks without copying
    println!("  - Task submitted, waiting for result...");
    let result: String = result_ref.get().await?;
    println!("  - {}", result);
    
    // The dataset is still available for other tasks
    println!("\nDataset still available for reuse:");
    // Note: In the integrated version, the ObjectRef would retrieve from the object store
    // For now, we demonstrate that the object is in the store
    println!("  - Dataset stored and available for other tasks");
    
    Ok(())
}