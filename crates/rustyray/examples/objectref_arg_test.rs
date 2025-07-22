//! Test ObjectRef as function argument
use rustyray::prelude::*;

#[rustyray::remote]
async fn create_data() -> Vec<i32> {
    println!("Creating data...");
    vec![1, 2, 3, 4, 5]
}

#[rustyray::remote]
async fn process_data(data: Vec<i32>) -> Result<i32> {
    println!("Processing data...");
    println!("Got data: {:?}", data);
    Ok(data.iter().sum::<i32>())
}

#[rustyray::main]
async fn main() -> Result<()> {
    println!("=== ObjectRef Argument Test ===\n");
    
    // Create data with first task
    println!("Creating data with task 1...");
    let data_ref = create_data_remote::remote().await?;
    println!("Task 1 returned ObjectRef: {:?}", data_ref.id());
    
    // Pass ObjectRef to second task - Now using automatic resolution!
    println!("\nPassing ObjectRef to task 2...");
    // Using the new remote_ref variant that accepts ObjectRef directly
    let sum_ref = process_data_remote::remote_ref(data_ref).await?;
    println!("Task 2 returned ObjectRef: {:?}", sum_ref.id());
    
    // Get final result
    println!("\nGetting final result...");
    let sum = sum_ref.get().await?;
    println!("Sum: {}", sum);
    
    Ok(())
}