//! Context-aware deserialization for task arguments.
//!
//! This module provides the infrastructure for deserializing types that need
//! access to runtime context (like ObjectRef needing the object store).

use crate::error::{Result, RustyRayError};
use crate::object_store::InMemoryStore;
use serde::de::DeserializeOwned;
use std::sync::Arc;

/// Runtime context needed during deserialization.
///
/// This context is passed through the task execution system and made available
/// during argument deserialization. It contains references to runtime resources
/// that certain types (like ObjectRef) need to function properly after deserialization.
#[derive(Clone)]
pub struct DeserializationContext {
    /// Reference to the object store for ObjectRef rehydration
    pub object_store: Arc<InMemoryStore>,
    // Future: Could add actor context, node info, etc.
}

impl DeserializationContext {
    /// Create a new deserialization context with the given object store.
    pub fn new(object_store: Arc<InMemoryStore>) -> Self {
        Self { object_store }
    }
}

/// Trait for types that need runtime context during deserialization.
///
/// Types implementing this trait can access the DeserializationContext
/// to properly initialize themselves after being deserialized from bytes.
pub trait ContextualDeserialize: Sized {
    /// Deserialize from bytes with access to runtime context.
    fn deserialize_with_context(bytes: &[u8], context: &DeserializationContext) -> Result<Self>;
}

/// Helper to provide contextual deserialization for standard types.
///
/// Since we can't use specialization on stable Rust, we'll handle this
/// in the macro by generating different code for ObjectRef vs other types.
pub fn deserialize_standard<T: DeserializeOwned>(
    bytes: &[u8],
    _context: &DeserializationContext,
) -> Result<T> {
    crate::task::serde_utils::deserialize(bytes)
        .map_err(|e| RustyRayError::Internal(format!("Deserialization failed: {e}")))
}

// We'll handle ObjectRef deserialization specially in the macro
// For now, just export the standard deserialization function
pub use deserialize_standard as deserialize_arg;
