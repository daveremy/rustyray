//! Example demonstrating error handling in RustyRay's task system.
//!
//! This example shows:
//! - How task errors are propagated through ObjectRef
//! - Different types of errors (validation, missing functions, etc.)
//! - How to handle errors gracefully

use rustyray_core::actor::ActorSystem;
use rustyray_core::error::{Result, RustyRayError};
use rustyray_core::task::{TaskBuilder, TaskSystem};
use rustyray_core::task_function;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== RustyRay Error Handling Example ===\n");

    // Create systems
    let actor_system = Arc::new(ActorSystem::new());
    let task_system = Arc::new(TaskSystem::new(actor_system.clone()));

    // Register functions with different error behaviors
    println!("1. Registering task functions...");

    // Function that validates input
    task_system.register_function(
        "divide",
        task_function!(|a: f64, b: f64| async move {
            if b == 0.0 {
                Err(RustyRayError::Internal("Division by zero".to_string()))
            } else {
                Ok::<f64, RustyRayError>(a / b)
            }
        }),
    )?;

    // Function that validates range
    task_system.register_function(
        "sqrt",
        task_function!(|x: f64| async move {
            if x < 0.0 {
                Err(RustyRayError::Internal(format!(
                    "Cannot take square root of negative number: {}",
                    x
                )))
            } else {
                Ok::<f64, RustyRayError>(x.sqrt())
            }
        }),
    )?;

    // Function that simulates network failure
    task_system.register_function(
        "fetch_data",
        task_function!(|url: String| async move {
            if url.starts_with("http://") {
                Err(RustyRayError::Internal(
                    "Insecure protocol not allowed".to_string(),
                ))
            } else if url.starts_with("https://") {
                Ok::<String, RustyRayError>(format!("Data from {}", url))
            } else {
                Err(RustyRayError::Internal("Invalid URL format".to_string()))
            }
        }),
    )?;

    println!("   ✓ Functions registered\n");

    // Example 1: Successful operations
    println!("2. Successful operations:");

    let result = TaskBuilder::new("divide")
        .arg(10.0)
        .arg(2.0)
        .submit::<f64>(&task_system)
        .await?
        .get()
        .await?;
    println!("   10.0 / 2.0 = {}", result);

    let result = TaskBuilder::new("sqrt")
        .arg(16.0)
        .submit::<f64>(&task_system)
        .await?
        .get()
        .await?;
    println!("   sqrt(16.0) = {}\n", result);

    // Example 2: Handling validation errors
    println!("3. Handling validation errors:");

    // Division by zero
    let div_by_zero = TaskBuilder::new("divide")
        .arg(5.0)
        .arg(0.0)
        .submit::<f64>(&task_system)
        .await?;

    match div_by_zero.get().await {
        Ok(_) => println!("   ERROR: Division by zero should have failed!"),
        Err(e) => println!("   ✓ Division by zero caught: {}", e),
    }

    // Negative square root
    let negative_sqrt = TaskBuilder::new("sqrt")
        .arg(-4.0)
        .submit::<f64>(&task_system)
        .await?;

    match negative_sqrt.get().await {
        Ok(_) => println!("   ERROR: Negative sqrt should have failed!"),
        Err(e) => println!("   ✓ Negative sqrt caught: {}", e),
    }

    // Invalid URL
    let bad_url = TaskBuilder::new("fetch_data")
        .arg("not-a-url")
        .submit::<String>(&task_system)
        .await?;

    match bad_url.get().await {
        Ok(_) => println!("   ERROR: Invalid URL should have failed!"),
        Err(e) => println!("   ✓ Invalid URL caught: {}", e),
    }

    // Example 3: Missing function error
    println!("\n4. Missing function error:");

    let missing_func = TaskBuilder::new("non_existent_function")
        .arg(42)
        .submit::<i32>(&task_system)
        .await?;

    match missing_func.get().await {
        Ok(_) => println!("   ERROR: Missing function should have failed!"),
        Err(e) => println!("   ✓ Missing function caught: {}", e),
    }

    // Example 4: Using Future trait for error handling
    println!("\n5. Using async/await for error handling:");

    let tasks = vec![
        TaskBuilder::new("divide")
            .arg(20.0)
            .arg(4.0)
            .submit::<f64>(&task_system)
            .await?,
        TaskBuilder::new("divide")
            .arg(15.0)
            .arg(3.0)
            .submit::<f64>(&task_system)
            .await?,
        TaskBuilder::new("divide")
            .arg(10.0)
            .arg(0.0)
            .submit::<f64>(&task_system)
            .await?, // This will fail
        TaskBuilder::new("divide")
            .arg(8.0)
            .arg(2.0)
            .submit::<f64>(&task_system)
            .await?,
    ];

    println!("   Processing {} division tasks...", tasks.len());
    for (i, task) in tasks.into_iter().enumerate() {
        match task.await {
            Ok(result) => println!("   Task {}: Success = {}", i + 1, result),
            Err(e) => println!("   Task {}: Failed = {}", i + 1, e),
        }
    }

    // Example 5: Error recovery pattern
    println!("\n6. Error recovery pattern:");

    // Try primary server, fallback to secondary
    let primary_url = "http://insecure.com/data";
    let secondary_url = "https://secure.com/data";

    println!("   Trying primary server: {}", primary_url);
    let primary_result = TaskBuilder::new("fetch_data")
        .arg(primary_url.to_string())
        .submit::<String>(&task_system)
        .await?;

    let data = match primary_result.get().await {
        Ok(data) => {
            println!("   Primary server succeeded: {}", data);
            data
        }
        Err(e) => {
            println!("   Primary server failed: {}", e);
            println!("   Trying secondary server: {}", secondary_url);

            let secondary_result = TaskBuilder::new("fetch_data")
                .arg(secondary_url.to_string())
                .submit::<String>(&task_system)
                .await?
                .get()
                .await?;

            println!("   Secondary server succeeded: {}", secondary_result);
            secondary_result
        }
    };

    println!("   Final data: {}", data);

    // Shutdown
    println!("\n7. Shutting down...");
    task_system.shutdown().await?;
    actor_system.shutdown().await?;

    println!("\n✓ Error handling example completed successfully!");
    Ok(())
}
