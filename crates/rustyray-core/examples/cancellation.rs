//! Example demonstrating task cancellation and timeout features

use rustyray_core::error::Result;
use rustyray_core::runtime;
use rustyray_core::task::{TaskBuilder, TaskSystem};
use rustyray_core::task_function;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== RustyRay Task Cancellation Example ===\n");

    // Initialize the runtime
    runtime::init()?;
    let rt = runtime::global()?;
    let actor_system = rt.actor_system();

    // Create a custom task system with timeout for this example
    let task_system = Arc::new(TaskSystem::with_timeout(
        actor_system.clone(),
        Duration::from_millis(500),
    ));

    println!("1. Task system configured with 500ms timeout\n");

    // Register task functions
    task_system.register_function(
        "fast_task",
        task_function!(|x: i32| async move {
            println!("   Fast task starting...");
            time::sleep(Duration::from_millis(100)).await;
            println!("   Fast task completed!");
            Ok::<i32, rustyray_core::error::RustyRayError>(x * 2)
        }),
    )?;

    task_system.register_function(
        "slow_task",
        task_function!(|x: i32| async move {
            println!("   Slow task starting...");
            for i in 1..=10 {
                time::sleep(Duration::from_millis(100)).await;
                println!("   Slow task progress: {i}/10");
            }
            println!("   Slow task completed!");
            Ok::<i32, rustyray_core::error::RustyRayError>(x * 3)
        }),
    )?;

    task_system.register_function(
        "infinite_task",
        task_function!(|| async move {
            println!("   Infinite task starting...");
            let mut count = 0;
            loop {
                time::sleep(Duration::from_millis(200)).await;
                count += 1;
                println!("   Infinite task iteration {count}");
            }
            #[allow(unreachable_code)]
            Ok::<(), rustyray_core::error::RustyRayError>(())
        }),
    )?;

    println!("2. Running fast task (100ms < 500ms timeout):");
    let fast_ref = TaskBuilder::new("fast_task")
        .arg(21)
        .submit::<i32>(&task_system)
        .await?;

    match fast_ref.get().await {
        Ok(result) => println!("   ✓ Fast task result: {result}\n"),
        Err(e) => println!("   ✗ Fast task failed: {e}\n"),
    }

    println!("3. Running slow task (1000ms > 500ms timeout):");
    let slow_ref = TaskBuilder::new("slow_task")
        .arg(10)
        .submit::<i32>(&task_system)
        .await?;

    // Wait a bit to see some progress
    time::sleep(Duration::from_millis(600)).await;

    match slow_ref.get().await {
        Ok(result) => println!("   ✓ Slow task result: {result}\n"),
        Err(e) => println!("   ✗ Slow task cancelled: {e}\n"),
    }

    println!("4. Running infinite task:");
    let infinite_ref = TaskBuilder::new("infinite_task")
        .submit::<()>(&task_system)
        .await?;

    // Let it run for a bit
    time::sleep(Duration::from_millis(600)).await;
    println!("   Checking infinite task status...");

    match infinite_ref.get().await {
        Ok(_) => println!("   ✓ Infinite task completed (unexpected!)\n"),
        Err(e) => println!("   ✗ Infinite task cancelled: {e}\n"),
    }

    println!("5. Demonstrating memory leak prevention:");
    println!("   Submitting 100 tasks that would never complete...");

    let mut stuck_refs = vec![];
    for i in 0..100 {
        let name = if i % 2 == 0 {
            "infinite_task"
        } else {
            "slow_task"
        };
        let ref_ = if i % 2 == 0 {
            TaskBuilder::new(name).submit::<()>(&task_system).await?
        } else {
            TaskBuilder::new(name)
                .arg(i)
                .submit::<i32>(&task_system)
                .await
                .map(|r| {
                    // Convert to unit type for uniform storage
                    drop(r);
                    TaskBuilder::new("infinite_task").submit::<()>(&task_system)
                })?
                .await?
        };
        stuck_refs.push(ref_);
    }

    println!("   Waiting for timeout mechanism to clean them up...");
    time::sleep(Duration::from_millis(600)).await;

    // Check a few to verify they were cancelled
    let mut cancelled_count = 0;
    for (i, ref_) in stuck_refs.iter().take(5).enumerate() {
        if ref_.get().await.is_err() {
            cancelled_count += 1;
        }
        print!(".");
        if i == 4 {
            println!();
        }
    }

    println!("   ✓ Verified {cancelled_count}/5 tasks were cancelled\n");
    println!("   Without cancellation, these tasks would leak memory forever!");

    // Shutdown
    println!("\n6. Shutting down...");
    task_system.shutdown().await?;
    runtime::shutdown()?;

    println!("\n✓ Cancellation example completed successfully!");
    println!("  Key takeaways:");
    println!("  - Tasks that exceed timeout are automatically cancelled");
    println!("  - This prevents memory leaks from stuck tasks");
    println!("  - Timeout is configurable per TaskSystem");

    Ok(())
}
