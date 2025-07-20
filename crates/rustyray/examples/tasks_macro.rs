//! Example demonstrating RustyRay's task execution with the macro API.
//!
//! This example shows:
//! - Using #[remote] for automatic function registration
//! - Using #[actor] and #[actor_methods] for typed actors
//! - Task dependencies through ObjectRef arguments
//! - Clean API with minimal boilerplate

use rustyray::prelude::*;

/// Remote function for computing squares
#[rustyray::remote]
async fn square(x: i32) -> i32 {
    println!("Computing square of {}", x);
    x * x
}

/// Remote function that takes ObjectRef arguments
///
/// NOTE: This currently fails with "ObjectRef has no state" error when called
/// because distributed object resolution is not yet implemented. Once Phase 4
/// implements the shared memory object store, this will work correctly.
#[rustyray::remote]
async fn sum_squares(a: ObjectRef<i32>, b: ObjectRef<i32>) -> Result<i32> {
    let val_a = a.get().await?;
    let val_b = b.get().await?;
    println!("Summing {} + {} = {}", val_a, val_b, val_a + val_b);
    Ok(val_a + val_b)
}

/// Coordinator actor using the macro API
#[rustyray::actor]
struct Coordinator {
    name: String,
}

#[rustyray::actor_methods]
impl Coordinator {
    pub fn new(name: String) -> Self {
        Coordinator { name }
    }

    pub async fn process_items(&mut self, count: usize) -> i32 {
        println!("{}: Processing {} items", self.name, count);

        // Submit parallel tasks using the macro-generated API
        let mut futures = Vec::new();
        for i in 0..count {
            let future = square_remote::remote(i as i32).await.unwrap();
            futures.push(future);
        }

        // Wait for all results and sum them
        let mut sum = 0;
        for future in futures {
            let result = future.get().await.unwrap();
            sum += result;
        }

        println!("{}: Sum of squares = {}", self.name, sum);
        sum
    }

    pub async fn chain_computation(&self) -> i32 {
        println!("{}: Chain computation not available yet", self.name);
        // NOTE: This method demonstrates ObjectRef chaining which requires
        // the object store implementation (Phase 4). For now, we return a dummy value.
        /*
        // This will work once object store is implemented:
        let square_5 = square_remote::remote(5).await.unwrap();
        let square_3 = square_remote::remote(3).await.unwrap();
        let result = sum_squares_remote::remote(square_5, square_3).await.unwrap();
        let final_value = result.get().await.unwrap();
        println!("{}: Chained result = {}", self.name, final_value);
        final_value
        */
        0 // Placeholder return
    }
}

#[rustyray::main]
async fn main() -> Result<()> {
    println!("=== RustyRay Tasks Example with Macros ===\n");

    // Example 1: Direct remote function calls
    println!("1. Direct remote function calls:");
    let result1 = square_remote::remote(7).await?;
    let value1 = result1.get().await?;
    println!("   square(7) = {}", value1);

    // Example 2: Chained computation with ObjectRefs
    // NOTE: This example currently fails due to ObjectRef serialization limitations.
    // It will work once the distributed object store is implemented in Phase 4.
    println!("\n2. Chained computation (currently limited):");
    println!("   Skipping ObjectRef chaining demo - requires object store (Phase 4)");
    /*
    let a = square_remote::remote(4).await?;
    let b = square_remote::remote(6).await?;
    let sum = sum_squares_remote::remote(a, b).await?;
    let total = sum.get().await?;
    println!("   square(4) + square(6) = {}", total);
    */

    // Example 3: Using an actor
    println!("\n3. Actor-based task coordination:");
    let coordinator = Coordinator::remote("MainCoordinator".to_string()).await?;

    // Process items through the actor
    let actor_result = coordinator.process_items(5).await?;
    let actor_sum = actor_result.get().await?;
    println!("   Actor computed sum: {}", actor_sum);

    // Chain computation through actor
    let chain_result = coordinator.chain_computation().await?;
    let chain_value = chain_result.get().await?;
    println!("   Actor chain result: {}", chain_value);

    println!("\n✓ All tasks completed successfully!");
    Ok(())
}
