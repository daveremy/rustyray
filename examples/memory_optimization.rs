//! Example demonstrating memory optimization in dependency resolution

use rustyray::actor::ActorSystem;
use rustyray::task::{TaskSystem, TaskBuilder};
use rustyray::task_function;
use rustyray::error::Result;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== RustyRay Memory Optimization Demo ===\n");
    
    // Create systems
    let actor_system = Arc::new(ActorSystem::new());
    let task_system = Arc::new(TaskSystem::new(actor_system.clone()));
    
    // Register functions
    task_system.register_function("create_data", task_function!(|size: usize| async move {
        // Create a large data vector
        let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        Ok::<Vec<u8>, rustyray::error::RustyRayError>(data)
    }))?;
    
    task_system.register_function("process_data", task_function!(|data: Vec<u8>| async move {
        // Simulate processing
        let sum: u64 = data.iter().map(|&b| b as u64).sum();
        Ok::<u64, rustyray::error::RustyRayError>(sum)
    }))?;
    
    println!("1. Creating large data object (10MB)...");
    let start = Instant::now();
    
    // Create a large object
    let data_ref = TaskBuilder::new("create_data")
        .arg(10_000_000usize)  // 10MB
        .submit::<Vec<u8>>(&task_system)
        .await?;
    
    let data = data_ref.get().await?;
    println!("   Created {} bytes in {:?}", data.len(), start.elapsed());
    
    println!("\n2. Sharing data across multiple tasks...");
    let start = Instant::now();
    
    // Put the data into the object store
    let shared_ref = task_system.put(data).await?;
    
    // Submit multiple tasks that use the same data
    let mut result_refs = vec![];
    for i in 0..5 {
        let result_ref = TaskBuilder::new("process_data")
            .arg_ref(&shared_ref)
            .submit::<u64>(&task_system)
            .await?;
        result_refs.push((i, result_ref));
    }
    
    // Get all results
    let mut results = vec![];
    for (i, ref_) in result_refs {
        let result = ref_.get().await?;
        results.push((i, result));
        println!("   Task {} result: {}", i, result);
    }
    
    println!("   Processed 5 tasks in {:?}", start.elapsed());
    
    println!("\n3. Memory optimization benefits:");
    println!("   - Object store uses Bytes for zero-copy cloning");
    println!("   - Multiple tasks share the same data without copying");
    println!("   - Pre-allocated vectors reduce allocations");
    println!("   - TaskArg::ObjectRef avoids data duplication");
    
    println!("\n4. Demonstrating dependency chain...");
    let start = Instant::now();
    
    // Create a chain of dependent tasks
    let step1 = TaskBuilder::new("create_data")
        .arg(1_000_000usize)  // 1MB
        .submit::<Vec<u8>>(&task_system)
        .await?;
    
    // Each task depends on the previous one
    let step1_ref = task_system.put(step1.get().await?).await?;
    
    let step2 = TaskBuilder::new("process_data")
        .arg_ref(&step1_ref)
        .submit::<u64>(&task_system)
        .await?;
    
    let final_result = step2.get().await?;
    println!("   Chain result: {} in {:?}", final_result, start.elapsed());
    
    // Shutdown
    println!("\n5. Shutting down...");
    task_system.shutdown().await?;
    actor_system.shutdown().await?;
    
    println!("\n✓ Memory optimization demo completed!");
    println!("  Key optimizations:");
    println!("  - Bytes type enables cheap cloning in object store");
    println!("  - Pre-allocated vectors avoid reallocation");
    println!("  - ObjectRef dependencies avoid data duplication");
    
    Ok(())
}