//! Example demonstrating RustyRay's task execution system.
//!
//! This example shows:
//! - Defining and registering remote functions
//! - Submitting tasks and getting ObjectRefs
//! - Task dependencies through ObjectRef arguments
//! - Integration with the actor system

use async_trait::async_trait;
use rustyray_core::actor::Actor;
use rustyray_core::error::Result;
use rustyray_core::object_ref::ObjectRef;
use rustyray_core::runtime;
use rustyray_core::task::{TaskBuilder, TaskSystem};
use rustyray_core::task_function;
use std::any::Any;
use std::sync::Arc;

/// A simple actor that can submit tasks
struct Coordinator {
    task_system: Arc<TaskSystem>,
}

#[async_trait]
impl Actor for Coordinator {
    async fn handle(&mut self, msg: Box<dyn Any + Send>) -> Result<Box<dyn Any + Send>> {
        if let Some(count) = msg.downcast_ref::<usize>() {
            println!("Coordinator: Processing {count} items");

            // Submit parallel tasks
            let mut futures = Vec::new();
            for i in 0..*count {
                let future = TaskBuilder::new("square")
                    .arg(i as i32)
                    .submit::<i32>(&self.task_system)
                    .await?;
                futures.push(future);
            }

            // Wait for all results
            let mut sum = 0;
            for future in futures {
                let result = future.get().await?;
                sum += result;
            }

            println!("Coordinator: Sum of squares = {sum}");
            Ok(Box::new(sum))
        } else {
            Err(rustyray_core::error::RustyRayError::InvalidMessage)
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== RustyRay Task System Example ===\n");

    // Initialize the runtime
    runtime::init()?;
    let rt = runtime::global()?;
    let actor_system = rt.actor_system();
    let task_system = rt.task_system();

    // Register task functions
    println!("1. Registering task functions...");

    // Square function
    task_system.register_function(
        "square",
        task_function!(|x: i32| async move {
            println!("   Task: Computing square of {x}");
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            Ok::<i32, rustyray_core::error::RustyRayError>(x * x)
        }),
    )?;

    // Add function
    task_system.register_function(
        "add",
        task_function!(|x: i32, y: i32| async move {
            println!("   Task: Adding {x} + {y}");
            Ok::<i32, rustyray_core::error::RustyRayError>(x + y)
        }),
    )?;

    // Multiply function
    task_system.register_function(
        "multiply",
        task_function!(|x: i32, y: i32| async move {
            println!("   Task: Multiplying {x} * {y}");
            Ok::<i32, rustyray_core::error::RustyRayError>(x * y)
        }),
    )?;

    println!("   ✓ Functions registered\n");

    // Example 1: Simple task execution
    println!("2. Simple task execution:");
    let result_ref: ObjectRef<i32> = TaskBuilder::new("square")
        .arg(5)
        .submit(task_system)
        .await?;
    let result = result_ref.get().await?;
    println!("   Square of 5 = {result}\n");

    // Example 2: Parallel tasks
    println!("3. Parallel task execution:");
    let nums = vec![1, 2, 3, 4, 5];
    let mut futures = Vec::new();

    for &num in &nums {
        let future = TaskBuilder::new("square")
            .arg(num)
            .submit::<i32>(task_system)
            .await?;
        futures.push(future);
    }

    print!("   Squares: ");
    for (i, future) in futures.into_iter().enumerate() {
        let result = future.get().await?;
        print!("{result}");
        if i < nums.len() - 1 {
            print!(", ");
        }
    }
    println!("\n");

    // Example 3: Task chaining with ObjectRef dependencies
    println!("4. Task chaining with ObjectRef dependencies:");

    // First compute 3 * 4
    let multiply_ref = TaskBuilder::new("multiply")
        .arg(3)
        .arg(4)
        .submit::<i32>(task_system)
        .await?;
    println!("   Submitted multiply(3, 4)");

    // Then add 5 to the result WITHOUT waiting for it to complete
    // This demonstrates true task chaining where the second task
    // automatically waits for the first to complete
    let add_ref = TaskBuilder::new("add")
        .arg_ref(&multiply_ref) // Pass the ObjectRef directly!
        .arg(5)
        .submit::<i32>(task_system)
        .await?;
    println!("   Submitted add(multiply_ref, 5) - chained without waiting!");

    // Get final result (this waits for both tasks to complete)
    let final_result = add_ref.get().await?;
    println!("   Final result: {final_result} (both tasks completed)\n");

    // Example 4: Actor submitting tasks
    println!("5. Actor submitting tasks:");
    let coordinator = Coordinator {
        task_system: task_system.clone(),
    };
    let coordinator_ref = actor_system.create_actor(coordinator).await?;

    // Ask coordinator to process 5 items
    let sum = coordinator_ref.call(Box::new(5usize)).await?;
    let sum_value = sum.downcast::<i32>().unwrap();
    println!("   Sum returned by coordinator: {}\n", *sum_value);

    // Example 5: Using put() to create ObjectRefs
    println!("6. Using put() to store values:");
    let stored_value = task_system.put(100i32).await?;
    println!("   Stored value 100");

    // Get the stored value
    let value = stored_value.get().await?;
    println!("   Retrieved stored value: {value}");

    // Use it in a task
    let result_ref = TaskBuilder::new("add")
        .arg(value)
        .arg(50)
        .submit::<i32>(task_system)
        .await?;

    let final_result = result_ref.get().await?;
    println!("   {value} + 50 = {final_result}\n");

    // Shutdown
    println!("7. Shutting down...");
    runtime::shutdown()?;

    println!("\n✓ Example completed successfully!");
    Ok(())
}
