//! Debug test to see task execution
#![allow(dead_code, unused_variables)]
use rustyray::prelude::*;

#[rustyray::remote]
async fn simple_task() -> i32 {
    println!(">>> Task executing!");
    42
}

#[rustyray::main]
async fn main() -> Result<()> {
    println!("=== Debug Test ===\n");

    // Get runtime info
    let runtime = rustyray_core::runtime::global()?;
    let task_system = runtime.task_system();

    println!("Submitting task...");
    let task_ref = simple_task_remote::remote().await?;
    println!("Task submitted with ID: {:?}", task_ref.id());

    // Give it a moment to execute
    println!("Waiting a bit...");
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    println!("Getting result...");
    let result = task_ref.get().await?;
    println!("Got result: {result}");

    Ok(())
}
