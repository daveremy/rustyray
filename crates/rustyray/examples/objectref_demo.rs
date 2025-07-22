//! Demo of ObjectRef arguments in remote functions
//!
//! This example shows how the #[remote] macro handles
//! ObjectRef arguments for task chaining.

#![allow(dead_code, unused_variables)]
use rustyray::prelude::*;

#[rustyray::remote]
async fn stage1(x: i32) -> i32 {
    println!("Stage 1: Processing {x}");
    x * 2
}

#[rustyray::remote]
async fn stage2(prev: ObjectRef<i32>, y: i32) -> Result<i32> {
    println!("Stage 2: Waiting for previous result...");
    let prev_result = prev.get().await?;
    println!("Stage 2: Got {prev_result} from stage 1, adding {y}");
    Ok(prev_result + y)
}

#[rustyray::remote]
async fn stage3(prev1: ObjectRef<i32>, prev2: ObjectRef<i32>) -> Result<i32> {
    println!("Stage 3: Waiting for two previous results...");
    let result1 = prev1.get().await?;
    let result2 = prev2.get().await?;
    println!("Stage 3: Got {result1} and {result2}, multiplying");
    Ok(result1 * result2)
}

#[rustyray::main]
async fn main() -> Result<()> {
    println!("=== RustyRay ObjectRef Demo ===\n");

    // Stage 1: Initial computation
    println!("Starting pipeline...");
    let stage1_ref = stage1_remote::remote(10).await?;

    // Stage 2: Depends on stage 1
    let stage2_ref = stage2_remote::remote(stage1_ref.clone(), 5).await?;

    // Stage 3: Depends on both stage 1 and stage 2
    let stage3_ref = stage3_remote::remote(stage1_ref, stage2_ref).await?;

    // Get final result
    println!("\nGetting final result...");
    let final_result = stage3_ref.get().await?;
    println!("Final result: {final_result}");
    println!("\nExpected: (10 * 2) * ((10 * 2) + 5) = 20 * 25 = 500");

    Ok(())
}
