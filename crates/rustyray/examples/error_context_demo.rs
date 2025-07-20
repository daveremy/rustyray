//! Example demonstrating enhanced error propagation with context
//!
//! This example shows how RustyRay now adds helpful context to errors,
//! making debugging distributed applications much easier.

use rustyray::prelude::*;

// Remote function that might fail
#[rustyray::remote]
async fn risky_operation(value: i32) -> i32 {
    if value < 0 {
        panic!("Negative values not allowed!");
    }
    value * 2
}

// Actor that demonstrates error context
#[rustyray::actor]
struct ErrorDemoActor {
    name: String,
}

#[rustyray::actor_methods]
impl ErrorDemoActor {
    pub fn new(name: String) -> Self {
        ErrorDemoActor { name }
    }

    pub async fn try_operation(&self, value: i32) -> String {
        format!("{}: Trying operation with {}", self.name, value)
    }
}

#[rustyray::main]
async fn main() -> Result<()> {
    println!("=== RustyRay Error Context Demo ===\n");

    // Example 1: Runtime not initialized error (this won't happen with #[rustyray::main])
    println!("1. Enhanced runtime error messages are built-in");
    println!("   (Runtime is already initialized by #[rustyray::main])\n");

    // Example 2: Remote function with good input
    println!("2. Successful remote function call:");
    match risky_operation_remote::remote(5).await {
        Ok(result_ref) => {
            let value = result_ref.get().await?;
            println!("   risky_operation(5) = {}", value);
        }
        Err(e) => {
            println!("   Error with context: {}", e);
        }
    }

    // Example 3: Remote function that will panic
    println!("\n3. Remote function with error:");
    match risky_operation_remote::remote(-5).await {
        Ok(result_ref) => match result_ref.get().await {
            Ok(value) => println!("   risky_operation(-5) = {}", value),
            Err(e) => println!("   Task execution failed: {}", e),
        },
        Err(e) => {
            // The enhanced error will show context
            println!("   Error with context: {}", e);
        }
    }

    // Example 4: Actor method calls with context
    println!("\n4. Actor method calls:");
    match ErrorDemoActor::remote("TestActor".to_string()).await {
        Ok(actor) => match actor.try_operation(42).await {
            Ok(result_ref) => {
                let msg = result_ref.get().await?;
                println!("   {}", msg);
            }
            Err(e) => {
                println!("   Actor error with context: {}", e);
            }
        },
        Err(e) => {
            println!("   Failed to create actor with context: {}", e);
        }
    }

    println!("\n✓ Error context demonstration complete!");
    Ok(())
}
