//! RustyRay - A Rust implementation of Ray Core
//! 
//! This library provides the core components for building a distributed
//! actor system inspired by Ray. We're starting with local actors and
//! will gradually add distributed features.
//!
//! # Architecture Overview
//! 
//! Based on Ray's C++ implementation, RustyRay consists of:
//! 
//! - **Actor System**: Stateful workers that process messages sequentially
//! - **Task System**: Stateless function execution with dependencies (future)
//! - **Object Store**: Distributed memory for sharing data (future)
//! - **GCS (Global Control Store)**: Cluster metadata management (future)
//!
//! # Current Focus
//! 
//! We're starting with a minimal actor system that supports:
//! - Creating actors locally
//! - Sending messages to actors
//! - Basic lifecycle management

/// The actor module implements Ray's actor concept in Rust.
/// 
/// In Ray, actors are:
/// - Stateful worker processes
/// - Process messages sequentially (by default)
/// - Have unique IDs for addressing
/// - Support method calls via handles
/// 
/// # Example
/// 
/// ```ignore
/// use rustyray::actor::{Actor, ActorSystem};
/// use async_trait::async_trait;
/// use std::any::Any;
/// 
/// struct Counter {
///     count: i32,
/// }
/// 
/// #[async_trait]
/// impl Actor for Counter {
///     async fn handle(&mut self, msg: Box<dyn Any + Send>) -> Result<Box<dyn Any + Send>> {
///         // Handle messages here
///         Ok(Box::new(()))
///     }
/// }
/// ```
pub mod actor;

/// Core types used throughout RustyRay.
/// 
/// This module defines fundamental types like:
/// - ActorId: Unique identifier for actors
/// - TaskId: Unique identifier for tasks
/// - ObjectId: Unique identifier for objects in the store
pub mod types;

/// Error types for RustyRay operations.
pub mod error;

// Future modules (commented out until we need them):
// pub mod task;      // Task execution system
// pub mod object;    // Object store for data sharing
// pub mod gcs;       // Global Control Store for cluster management
// pub mod rpc;       // RPC communication layer