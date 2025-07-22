//! Demo of Ray-style put/get API
//!
//! This example shows how to use the global ray::put() and ray::get()
//! functions to share data between tasks and actors, similar to Python Ray.

use rustyray::prelude::*;
use rustyray_core::ray;
use std::collections::HashMap;

// A large dataset that we want to share
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Dataset {
    name: String,
    data: Vec<f64>,
    metadata: HashMap<String, String>,
}

impl Dataset {
    fn new(name: &str, size: usize) -> Self {
        let data: Vec<f64> = (0..size).map(|i| i as f64 * 0.1).collect();
        let mut metadata = HashMap::new();
        metadata.insert("size".to_string(), size.to_string());
        metadata.insert("type".to_string(), "synthetic".to_string());
        
        Self {
            name: name.to_string(),
            data,
            metadata,
        }
    }
    
    fn mean(&self) -> f64 {
        self.data.iter().sum::<f64>() / self.data.len() as f64
    }
    
    fn size_mb(&self) -> f64 {
        (self.data.len() * std::mem::size_of::<f64>()) as f64 / 1_000_000.0
    }
}

#[rustyray::remote]
async fn preprocess_data(dataset: Dataset) -> Result<Vec<f64>> {
    println!("Preprocessing: Got dataset '{}' ({:.2} MB)", dataset.name, dataset.size_mb());
    
    // Normalize the data
    let mean = dataset.mean();
    let normalized: Vec<f64> = dataset.data.iter().map(|x| x - mean).collect();
    
    println!("Preprocessing: Normalized {} values (mean was {:.2})", normalized.len(), mean);
    Ok(normalized)
}

#[rustyray::remote]
async fn analyze_data(data: Vec<f64>) -> Result<HashMap<String, f64>> {
    println!("Analysis: Got {} data points", data.len());
    
    let mut stats = HashMap::new();
    
    // Calculate statistics
    let sum: f64 = data.iter().sum();
    let mean = sum / data.len() as f64;
    let variance = data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / data.len() as f64;
    let std_dev = variance.sqrt();
    let min = data.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    
    stats.insert("count".to_string(), data.len() as f64);
    stats.insert("mean".to_string(), mean);
    stats.insert("std_dev".to_string(), std_dev);
    stats.insert("min".to_string(), min);
    stats.insert("max".to_string(), max);
    
    println!("Analysis: Computed statistics");
    Ok(stats)
}

#[rustyray::remote]
async fn combine_results(
    stats1: HashMap<String, f64>,
    stats2: HashMap<String, f64>,
) -> Result<String> {
    println!("Combining: Got both statistics...");
    
    // Combine the statistics
    let all_stats = vec![stats1, stats2];
    
    println!("Combining: Got {} result sets", all_stats.len());
    
    // Format results
    let mut report = String::from("=== Combined Analysis Report ===\n\n");
    
    for (i, stats) in all_stats.iter().enumerate() {
        report.push_str(&format!("Dataset {}:\n", i + 1));
        report.push_str(&format!("  Count: {:.0}\n", stats.get("count").unwrap_or(&0.0)));
        report.push_str(&format!("  Mean: {:.2}\n", stats.get("mean").unwrap_or(&0.0)));
        report.push_str(&format!("  Std Dev: {:.2}\n", stats.get("std_dev").unwrap_or(&0.0)));
        report.push_str(&format!("  Min: {:.2}\n", stats.get("min").unwrap_or(&0.0)));
        report.push_str(&format!("  Max: {:.2}\n", stats.get("max").unwrap_or(&0.0)));
        report.push_str("\n");
    }
    
    Ok(report)
}

#[rustyray::main]
async fn main() -> Result<()> {
    println!("=== RustyRay Ray API Demo ===\n");
    
    // Create large datasets
    println!("Creating datasets...");
    let dataset1 = Dataset::new("training_data", 100_000);
    let dataset2 = Dataset::new("validation_data", 50_000);
    
    println!("Dataset 1: {:.2} MB", dataset1.size_mb());
    println!("Dataset 2: {:.2} MB", dataset2.size_mb());
    
    // Store datasets in object store using ray::put
    println!("\nStoring datasets in object store...");
    let dataset1_ref = ray::put(dataset1).await?;
    let dataset2_ref = ray::put(dataset2).await?;
    println!("Stored datasets with IDs: {:?}, {:?}", dataset1_ref.id(), dataset2_ref.id());
    
    // Process datasets in parallel
    println!("\nLaunching preprocessing tasks...");
    // Using remote_ref for automatic ObjectRef resolution!
    let preproc1_ref = preprocess_data_remote::remote_ref(dataset1_ref).await?;
    let preproc2_ref = preprocess_data_remote::remote_ref(dataset2_ref).await?;
    
    // Pass the preprocessing results directly to analysis
    // (No need to get and re-put - ObjectRefs can be passed directly)
    
    // Analyze in parallel
    println!("\nLaunching analysis tasks...");
    // Using remote_ref for automatic ObjectRef resolution
    let stats1_ref = analyze_data_remote::remote_ref(preproc1_ref).await?;
    let stats2_ref = analyze_data_remote::remote_ref(preproc2_ref).await?;
    
    // Combine results
    println!("\nCombining results...");
    // Using remote_ref for automatic ObjectRef resolution
    let report_ref = combine_results_remote::remote_ref(stats1_ref, stats2_ref).await?;
    
    // Get final report
    let report = ray::get(&report_ref).await?;
    println!("\n{}", report);
    
    // Demonstrate batch operations
    println!("=== Batch Operations Demo ===\n");
    
    // Store multiple values at once
    let values = vec![10, 20, 30, 40, 50];
    let batch_refs = ray::put_batch(values.clone()).await?;
    println!("Stored {} values in batch", batch_refs.len());
    
    // Retrieve them all at once
    let retrieved_values = ray::get_batch(&batch_refs).await?;
    println!("Retrieved values: {:?}", retrieved_values);
    assert_eq!(values, retrieved_values);
    
    // Wait for objects to be ready (useful for synchronization)
    println!("\nWaiting for all batch objects...");
    ray::wait(&batch_refs).await?;
    println!("All objects are ready!");
    
    Ok(())
}