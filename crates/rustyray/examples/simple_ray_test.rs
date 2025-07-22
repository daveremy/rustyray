//! Simple test of ray API
use rustyray::prelude::*;
use rustyray_core::ray;

#[rustyray::remote]
async fn double(x: i32) -> i32 {
    println!("Doubling {}", x);
    x * 2
}

#[rustyray::main]
async fn main() -> Result<()> {
    println!("=== Simple Ray Test ===\n");
    
    // Test direct put/get
    println!("Testing direct put/get...");
    let value = 42;
    let obj_ref = ray::put(value).await?;
    println!("Put value 42, got ref: {:?}", obj_ref.id());
    
    let retrieved = ray::get(&obj_ref).await?;
    println!("Retrieved: {}", retrieved);
    
    // Test with task
    println!("\nTesting with task...");
    let task_ref = double_remote::remote(10).await?;
    println!("Task submitted, ref: {:?}", task_ref.id());
    
    let result = ray::get(&task_ref).await?;
    println!("Task result: {}", result);
    
    Ok(())
}