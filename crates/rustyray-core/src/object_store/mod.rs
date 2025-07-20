//! Object store for efficient in-memory storage and sharing of serialized objects.
//!
//! This module provides the core object storage functionality for RustyRay,
//! inspired by Ray's Plasma store but adapted for Rust's ownership model.

mod store;
mod traits;
mod types;

#[cfg(test)]
mod tests;

pub use store::InMemoryStore;
pub use traits::ObjectStore;
pub use types::{ObjectId, ObjectMetadata, PutResult, StoreConfig, StoreStats};

// Re-export for convenience
pub use types::EvictionPolicy;