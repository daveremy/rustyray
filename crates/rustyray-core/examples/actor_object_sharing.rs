//! Demonstrates actors using the object store via ray::put/get.
//!
//! This example shows how actors can:
//! - Store large data in the object store
//! - Share data efficiently between actors
//! - Pass ObjectRefs as actor method arguments
//! - Use the same ray API as tasks

use async_trait::async_trait;
use rustyray_core::{
    actor::{Actor, ActorRef},
    object_store::ObjectStore,
    ray, runtime, Result, RustyRayError,
};
use serde::{Deserialize, Serialize};
use std::any::Any;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Matrix {
    name: String,
    rows: usize,
    cols: usize,
    data: Vec<f64>,
}

impl Matrix {
    fn new(name: &str, rows: usize, cols: usize) -> Self {
        let size = rows * cols;
        let data = (0..size).map(|i| i as f64 * 0.1).collect();
        Self {
            name: name.to_string(),
            rows,
            cols,
            data,
        }
    }

    fn size_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.data.len() * std::mem::size_of::<f64>()
    }
}

/// A data producer actor that generates and stores large datasets
struct DataProducer {
    id: u32,
    matrices_created: usize,
}

#[async_trait]
impl Actor for DataProducer {
    async fn handle(
        &mut self,
        msg: Box<dyn Any + Send>,
    ) -> Result<Box<dyn Any + Send>> {
        if let Some(cmd) = msg.downcast_ref::<ProducerCommand>() {
            match cmd {
                ProducerCommand::CreateMatrix { name, rows, cols } => {
                    // Create a large matrix
                    let matrix = Matrix::new(name, *rows, *cols);
                    let size = matrix.size_bytes();
                    
                    // Store it in the object store using ray::put
                    let obj_ref = ray::put(matrix).await?;
                    
                    self.matrices_created += 1;
                    println!(
                        "Producer {}: Created matrix '{}' ({} bytes) -> {:?}",
                        self.id, name, size, obj_ref.id()
                    );
                    
                    // Return the ObjectRef
                    Ok(Box::new(ProducerResponse::MatrixCreated(obj_ref)))
                }
                ProducerCommand::GetStats => {
                    Ok(Box::new(ProducerResponse::Stats {
                        producer_id: self.id,
                        matrices_created: self.matrices_created,
                    }))
                }
            }
        } else {
            Err(RustyRayError::Internal("Unknown message type".to_string()))
        }
    }
}

/// A data consumer actor that processes matrices from the object store
struct DataConsumer {
    id: u32,
    matrices_processed: usize,
}

#[async_trait]
impl Actor for DataConsumer {
    async fn handle(
        &mut self,
        msg: Box<dyn Any + Send>,
    ) -> Result<Box<dyn Any + Send>> {
        if let Some(cmd) = msg.downcast_ref::<ConsumerCommand>() {
            match cmd {
                ConsumerCommand::ProcessMatrix(obj_ref) => {
                    // Retrieve the matrix from object store using ray::get
                    let matrix: Matrix = ray::get(obj_ref).await?;
                    
                    // Simulate processing
                    let sum: f64 = matrix.data.iter().sum();
                    let avg = sum / matrix.data.len() as f64;
                    
                    self.matrices_processed += 1;
                    println!(
                        "Consumer {}: Processed matrix '{}' ({}x{}) - avg value: {:.2}",
                        self.id, matrix.name, matrix.rows, matrix.cols, avg
                    );
                    
                    Ok(Box::new(ConsumerResponse::ProcessingComplete { 
                        average: avg 
                    }))
                }
                ConsumerCommand::GetStats => {
                    Ok(Box::new(ConsumerResponse::Stats {
                        consumer_id: self.id,
                        matrices_processed: self.matrices_processed,
                    }))
                }
            }
        } else {
            Err(RustyRayError::Internal("Unknown message type".to_string()))
        }
    }
}

// Message types
#[derive(Debug)]
enum ProducerCommand {
    CreateMatrix { name: String, rows: usize, cols: usize },
    GetStats,
}

#[derive(Debug)]
enum ProducerResponse {
    MatrixCreated(rustyray_core::ObjectRef<Matrix>),
    Stats { producer_id: u32, matrices_created: usize },
}

#[derive(Debug)]
enum ConsumerCommand {
    ProcessMatrix(rustyray_core::ObjectRef<Matrix>),
    GetStats,
}

#[derive(Debug)]
enum ConsumerResponse {
    ProcessingComplete { average: f64 },
    Stats { consumer_id: u32, matrices_processed: usize },
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== RustyRay Actor Object Sharing Demo ===\n");

    // Initialize runtime
    runtime::init()?;
    let runtime = runtime::global()?;
    let actor_system = runtime.actor_system();

    // Create producer actors
    println!("Creating producer actors...");
    let producer1 = actor_system
        .create_actor(DataProducer { id: 1, matrices_created: 0 })
        .await?;
    let producer2 = actor_system
        .create_actor(DataProducer { id: 2, matrices_created: 0 })
        .await?;

    // Create consumer actors  
    println!("Creating consumer actors...");
    let consumer1 = actor_system
        .create_actor(DataConsumer { id: 1, matrices_processed: 0 })
        .await?;
    let consumer2 = actor_system
        .create_actor(DataConsumer { id: 2, matrices_processed: 0 })
        .await?;

    println!("\n--- Phase 1: Producers create matrices ---");
    
    // Producers create matrices
    let matrices = vec![
        create_matrix(&producer1, "matrix_A", 100, 100).await?,
        create_matrix(&producer1, "matrix_B", 200, 200).await?,
        create_matrix(&producer2, "matrix_C", 150, 150).await?,
        create_matrix(&producer2, "matrix_D", 300, 300).await?,
    ];

    println!("\n--- Phase 2: Consumers process matrices ---");
    
    // Distribute matrices to consumers
    // Consumer 1 processes first two matrices
    process_matrix(&consumer1, &matrices[0]).await?;
    process_matrix(&consumer1, &matrices[1]).await?;
    
    // Consumer 2 processes last two matrices
    process_matrix(&consumer2, &matrices[2]).await?;
    process_matrix(&consumer2, &matrices[3]).await?;

    println!("\n--- Phase 3: Cross-processing ---");
    
    // Demonstrate that any consumer can process any matrix
    // Consumer 1 processes a matrix created by producer 2
    process_matrix(&consumer1, &matrices[2]).await?;
    
    // Consumer 2 processes a matrix created by producer 1  
    process_matrix(&consumer2, &matrices[0]).await?;

    println!("\n--- Final Statistics ---");
    
    // Get stats from all actors
    print_producer_stats(&producer1).await?;
    print_producer_stats(&producer2).await?;
    print_consumer_stats(&consumer1).await?;
    print_consumer_stats(&consumer2).await?;

    // Show object store statistics
    println!("\n--- Object Store Statistics ---");
    let store = runtime.object_store();
    let stats = store.stats().await;
    println!("Total objects: {}", stats.total_objects);
    println!("Total size: {} bytes", stats.total_size);
    println!("Cache hits: {}", stats.hit_count);
    println!("Cache misses: {}", stats.miss_count);

    // Shutdown
    runtime::shutdown()?;
    println!("\nDemo completed successfully!");
    Ok(())
}

// Helper functions
async fn create_matrix(
    producer: &ActorRef,
    name: &str,
    rows: usize,
    cols: usize,
) -> Result<rustyray_core::ObjectRef<Matrix>> {
    let response = producer
        .call(Box::new(ProducerCommand::CreateMatrix {
            name: name.to_string(),
            rows,
            cols,
        }))
        .await?;
    
    if let Some(resp) = response.downcast_ref::<ProducerResponse>() {
        match resp {
            ProducerResponse::MatrixCreated(obj_ref) => Ok(obj_ref.clone()),
            _ => Err(RustyRayError::Internal("Unexpected response".to_string())),
        }
    } else {
        Err(RustyRayError::Internal("Invalid response type".to_string()))
    }
}

async fn process_matrix(
    consumer: &ActorRef,
    matrix_ref: &rustyray_core::ObjectRef<Matrix>,
) -> Result<f64> {
    let response = consumer
        .call(Box::new(ConsumerCommand::ProcessMatrix(matrix_ref.clone())))
        .await?;
    
    if let Some(resp) = response.downcast_ref::<ConsumerResponse>() {
        match resp {
            ConsumerResponse::ProcessingComplete { average } => Ok(*average),
            _ => Err(RustyRayError::Internal("Unexpected response".to_string())),
        }
    } else {
        Err(RustyRayError::Internal("Invalid response type".to_string()))
    }
}

async fn print_producer_stats(producer: &ActorRef) -> Result<()> {
    let response = producer.call(Box::new(ProducerCommand::GetStats)).await?;
    
    if let Some(resp) = response.downcast_ref::<ProducerResponse>() {
        match resp {
            ProducerResponse::Stats { producer_id, matrices_created } => {
                println!(
                    "Producer {}: Created {} matrices",
                    producer_id, matrices_created
                );
            }
            _ => {}
        }
    }
    Ok(())
}

async fn print_consumer_stats(consumer: &ActorRef) -> Result<()> {
    let response = consumer.call(Box::new(ConsumerCommand::GetStats)).await?;
    
    if let Some(resp) = response.downcast_ref::<ConsumerResponse>() {
        match resp {
            ConsumerResponse::Stats { consumer_id, matrices_processed } => {
                println!(
                    "Consumer {}: Processed {} matrices",
                    consumer_id, matrices_processed
                );
            }
            _ => {}
        }
    }
    Ok(())
}