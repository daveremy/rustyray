//! Test to demonstrate the race condition issue
use rustyray::prelude::*;

#[rustyray::remote]
async fn slow_task() -> i32 {
    println!("Task starting...");
    // Simulate some work
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    println!("Task finishing...");
    42
}

#[rustyray::main]
async fn main() -> Result<()> {
    println!("=== Race Condition Test ===\n");

    // Submit task
    println!("Submitting task...");
    let task_ref = slow_task_remote::remote().await?;
    println!("Task submitted, immediately trying to get result...");

    // Try to get result immediately (might fail)
    match task_ref.get().await {
        Ok(result) => println!("Got result: {}", result),
        Err(e) => println!("Error (expected): {}", e),
    }

    // Wait a bit and try again
    println!("\nWaiting 100ms and trying again...");
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    match task_ref.get().await {
        Ok(result) => println!("Got result: {}", result),
        Err(e) => println!("Error (unexpected): {}", e),
    }

    Ok(())
}
