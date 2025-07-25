//! Core types used throughout RustyRay.
//!
//! This module defines the fundamental types that mirror Ray's concepts
//! but adapted for Rust's type system.

use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

/// Counter for generating unique actor IDs.
/// In a distributed system, this would need to be more sophisticated.
static ACTOR_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Counter for generating unique task IDs.
static TASK_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Counter for generating unique object IDs.
static OBJECT_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Unique identifier for an actor.
///
/// In Ray, actor IDs are unique across the cluster. For now, we'll use
/// a simple incrementing counter, but eventually this should be:
/// - Globally unique (across nodes)
/// - Include node/process information
/// - Support serialization for network transfer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ActorId(u64);

impl Default for ActorId {
    fn default() -> Self {
        Self::new()
    }
}

impl ActorId {
    /// Generate a new unique actor ID.
    pub fn new() -> Self {
        let id = ACTOR_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        ActorId(id)
    }

    /// Get the inner ID value.
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for ActorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Actor({})", self.0)
    }
}

/// Unique identifier for a task.
///
/// Tasks in Ray are stateless function executions. Each task has a unique ID
/// for tracking execution, dependencies, and results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct TaskId(u64);

impl Default for TaskId {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskId {
    /// Generate a new unique task ID.
    pub fn new() -> Self {
        let id = TASK_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        TaskId(id)
    }

    /// Get the inner ID value.
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for TaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Task({})", self.0)
    }
}

/// Unique identifier for an object in the object store.
///
/// Objects are immutable data that can be shared between tasks and actors.
/// In Ray, objects are stored in the distributed object store (Plasma).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ObjectId(u64);

impl ObjectId {
    /// Generate a new unique object ID.
    pub fn new() -> Self {
        let id = OBJECT_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        ObjectId(id)
    }

    /// Get the inner ID value.
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl Default for ObjectId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Object({})", self.0)
    }
}

impl From<u64> for ObjectId {
    fn from(value: u64) -> Self {
        ObjectId(value)
    }
}

impl From<ObjectId> for TaskId {
    fn from(object_id: ObjectId) -> Self {
        // Use the same underlying ID value
        TaskId(object_id.as_u64())
    }
}

/// Result type for actor operations.
///
/// Eventually this will handle serialization/deserialization for
/// sending results across the network.
pub type ActorResult<T> = Result<T, crate::error::RustyRayError>;
