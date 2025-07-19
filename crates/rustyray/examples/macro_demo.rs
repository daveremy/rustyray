//! Demo of the new macro-based API
//!
//! This example shows how the #[remote] macro simplifies
//! the RustyRay API.

use rustyray::prelude::*;

#[rustyray::remote]
async fn add(x: i32, y: i32) -> i32 {
    println!("Computing {} + {} on a remote task", x, y);
    x + y
}

#[rustyray::remote]
async fn hello(name: String) -> String {
    format!("Hello, {}!", name)
}

#[rustyray::remote(num_cpus = 2.0)]
async fn compute_heavy(n: u64) -> u64 {
    // Simulate some heavy computation
    let mut result = 0;
    for i in 0..n {
        result += i;
    }
    result
}

#[rustyray::main]
async fn main() -> Result<()> {
    println!("=== RustyRay Macro Demo ===\n");
    
    // Simple addition
    println!("Calling add(5, 3)...");
    let result = add_remote::remote(5, 3).await?;
    println!("Result: {}\n", result.get().await?);
    
    // String function
    println!("Calling hello(\"World\")...");
    let greeting = hello_remote::remote("World".to_string()).await?;
    println!("Result: {}\n", greeting.get().await?);
    
    // Function with resource requirements
    println!("Calling compute_heavy(1000) with 2 CPUs...");
    let computation = compute_heavy_remote::remote(1000).await?;
    println!("Result: {}\n", computation.get().await?);
    
    println!("All tasks completed successfully!");
    Ok(())
}