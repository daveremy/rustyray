//! Comprehensive example showcasing all RustyRay macro features.
//!
//! This example demonstrates:
//! - #[remote] functions with various signatures
//! - #[actor] with multiple constructors
//! - Resource requirements (num_cpus, num_gpus)
//! - Error handling with Result types
//! - ObjectRef chaining
//! - Async and sync methods

#![allow(dead_code, unused_variables)]
use rustyray::prelude::*;
use std::time::Duration;

// ====== Remote Functions ======

/// Simple async remote function
#[rustyray::remote]
async fn compute_fibonacci(n: u32) -> u64 {
    println!("Computing fibonacci({n}) on worker");
    match n {
        0 => 0,
        1 => 1,
        _ => {
            let mut a = 0u64;
            let mut b = 1u64;
            for _ in 2..=n {
                let temp = a + b;
                a = b;
                b = temp;
            }
            b
        }
    }
}

/// Remote function with resource requirements
#[rustyray::remote(num_cpus = 2.0, num_gpus = 0.5)]
async fn train_model(data_size: usize, epochs: u32) -> ModelResult {
    println!("Training model with {data_size} data points for {epochs} epochs");
    println!("  Using 2 CPUs and 0.5 GPU");

    // Simulate training
    tokio::time::sleep(Duration::from_millis(100)).await;

    ModelResult {
        accuracy: 0.95,
        loss: 0.05,
        epochs_trained: epochs,
    }
}

/// Sync remote function (automatically wrapped in async)
#[rustyray::remote]
fn process_data(values: Vec<i32>) -> DataStats {
    println!("Processing {} values synchronously", values.len());

    let sum: i32 = values.iter().sum();
    let max = values.iter().max().copied().unwrap_or(0);
    let min = values.iter().min().copied().unwrap_or(0);

    DataStats {
        count: values.len(),
        sum,
        max,
        min,
        average: if values.is_empty() {
            0.0
        } else {
            sum as f64 / values.len() as f64
        },
    }
}

/// Remote function that returns a value with error handling done internally
#[rustyray::remote]
async fn divide(a: f64, b: f64) -> f64 {
    if b == 0.0 {
        println!("Warning: Division by zero, returning 0.0");
        0.0
    } else {
        a / b
    }
}

/// Remote function with ObjectRef parameters for chaining
///
/// NOTE: This currently requires the object store implementation (Phase 4)
/// to work properly when ObjectRefs are passed between tasks.
#[rustyray::remote]
async fn combine_results(
    stats: ObjectRef<DataStats>,
    model: ObjectRef<ModelResult>,
) -> Result<CombinedReport> {
    let stats_val = stats.get().await?;
    let model_val = model.get().await?;

    println!(
        "Combining results: {} data points, {:.2}% accuracy",
        stats_val.count,
        model_val.accuracy * 100.0
    );

    Ok(CombinedReport {
        data_count: stats_val.count,
        data_average: stats_val.average,
        model_accuracy: model_val.accuracy,
        summary: format!(
            "Processed {} items with {:.1}% model accuracy",
            stats_val.count,
            model_val.accuracy * 100.0
        ),
    })
}

// ====== Data Types ======

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ModelResult {
    accuracy: f64,
    loss: f64,
    epochs_trained: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct DataStats {
    count: usize,
    sum: i32,
    max: i32,
    min: i32,
    average: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct CombinedReport {
    data_count: usize,
    data_average: f64,
    model_accuracy: f64,
    summary: String,
}

// ====== Actor Definition ======

/// ML Pipeline coordinator actor
#[rustyray::actor]
pub struct MLPipeline {
    name: String,
    total_jobs: usize,
    successful_jobs: usize,
}

#[rustyray::actor_methods]
impl MLPipeline {
    /// Standard constructor
    pub fn new(name: String) -> Self {
        MLPipeline {
            name,
            total_jobs: 0,
            successful_jobs: 0,
        }
    }

    /// Constructor with initial stats
    pub fn with_stats(name: String, total: usize, successful: usize) -> Self {
        MLPipeline {
            name,
            total_jobs: total,
            successful_jobs: successful,
        }
    }

    /// Run a complete ML pipeline
    pub async fn run_pipeline(&mut self, data: Vec<i32>, epochs: u32) -> String {
        println!("\n{}: Starting ML pipeline", self.name);
        self.total_jobs += 1;

        // Step 1: Process data
        let stats_ref = process_data_remote::remote(data.clone()).await.unwrap();

        // Step 2: Train model
        let model_ref = train_model_remote::remote(data.len(), epochs)
            .await
            .unwrap();

        // Step 3: Combine results
        let report_ref = combine_results_remote::remote(stats_ref, model_ref)
            .await
            .unwrap();
        let report = report_ref.get().await.unwrap();

        self.successful_jobs += 1;
        println!("{}: Pipeline completed - {}", self.name, report.summary);

        report.summary
    }

    /// Get current statistics (sync method)
    pub fn get_stats(&self) -> (usize, usize) {
        println!(
            "{}: Jobs - Total: {}, Successful: {}",
            self.name, self.total_jobs, self.successful_jobs
        );
        (self.total_jobs, self.successful_jobs)
    }

    /// Compute success rate
    pub async fn success_rate(&self) -> f64 {
        if self.total_jobs == 0 {
            0.0
        } else {
            self.successful_jobs as f64 / self.total_jobs as f64
        }
    }
}

// ====== Main Function ======

#[rustyray::main]
async fn main() -> Result<()> {
    println!("=== Comprehensive RustyRay Macro Demo ===\n");

    // Example 1: Simple remote functions
    println!("1. Simple remote function calls:");
    let fib_future = compute_fibonacci_remote::remote(10).await?;
    let fib_result = fib_future.get().await?;
    println!("   fibonacci(10) = {fib_result}");

    // Example 2: Division with error handling
    println!("\n2. Division with built-in error handling:");
    let div1 = divide_remote::remote(10.0, 2.0).await?;
    let div2 = divide_remote::remote(10.0, 0.0).await?;

    let div1_result = div1.get().await?;
    println!("   10.0 / 2.0 = {div1_result}");

    let div2_result = div2.get().await?;
    println!("   10.0 / 0.0 = {div2_result} (handled internally)");

    // Example 3: Actor with multiple constructors
    println!("\n3. Actors with multiple constructors:");
    let pipeline1 = MLPipeline::remote("Pipeline1".to_string()).await?;
    let pipeline2 = MLPipeline::remote_with_stats("Pipeline2".to_string(), 5, 4).await?;

    let (total, successful) = pipeline2.get_stats().await?.get().await?;
    println!("   Pipeline2 started with {total} total, {successful} successful");

    // Example 4: Complete pipeline execution
    println!("\n4. Running ML pipeline:");
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let summary_ref = pipeline1.run_pipeline(data.clone(), 100).await?;
    let summary = summary_ref.get().await?;
    println!("   Result: {summary}");

    // Example 5: Parallel execution
    println!("\n5. Parallel task execution:");
    let results = [
        compute_fibonacci_remote::remote(15).await?,
        compute_fibonacci_remote::remote(20).await?,
        compute_fibonacci_remote::remote(25).await?,
    ];

    for (i, result_ref) in results.iter().enumerate() {
        let value = result_ref.get().await?;
        println!("   fibonacci({}) = {}", 15 + i * 5, value);
    }

    // Example 6: Check final stats
    println!("\n6. Final statistics:");
    let rate = pipeline1.success_rate().await?.get().await?;
    println!("   Pipeline1 success rate: {:.1}%", rate * 100.0);

    println!("\n✓ All demonstrations completed successfully!");
    Ok(())
}
