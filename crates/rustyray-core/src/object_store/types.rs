//! Core types for the object store.

use bytes::Bytes;
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::Arc;
use std::time::Instant;

// Re-export ObjectId from types module for convenience
pub use crate::types::ObjectId;

/// Metadata about stored objects.
#[derive(Debug, Clone)]
pub struct ObjectMetadata {
    /// Size of the object in bytes.
    pub size: usize,

    /// When the object was created.
    pub created_at: Instant,

    /// Reference count (managed by CLRU, but we track for stats).
    pub ref_count: Arc<AtomicUsize>,

    /// Type information for runtime type checking.
    pub type_id: std::any::TypeId,

    /// Type name for debugging.
    pub type_name: String,

    /// Whether this object is pinned (cannot be evicted).
    pub is_pinned: Arc<AtomicBool>,
}

/// Result of a put operation.
#[derive(Debug)]
pub struct PutResult {
    /// The ID assigned to the stored object.
    pub id: ObjectId,

    /// Size of the stored object in bytes.
    pub size: usize,
}

/// Configuration for the object store.
#[derive(Debug, Clone)]
pub struct StoreConfig {
    /// Maximum memory usage in bytes.
    pub max_memory: usize,

    /// Eviction policy to use when memory is full.
    pub eviction_policy: EvictionPolicy,

    /// Whether to collect detailed statistics.
    pub enable_stats: bool,
}

impl Default for StoreConfig {
    fn default() -> Self {
        Self {
            max_memory: 1_073_741_824, // 1GB default
            eviction_policy: EvictionPolicy::LRU,
            enable_stats: true,
        }
    }
}

/// Eviction policy for the cache.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvictionPolicy {
    /// Least Recently Used.
    LRU,
    /// First In First Out.
    FIFO,
    /// Least Frequently Used.
    LFU,
}

/// Store statistics.
#[derive(Debug, Clone, Default)]
pub struct StoreStats {
    /// Total number of objects in the store.
    pub total_objects: usize,

    /// Total size of all objects in bytes.
    pub total_size: usize,

    /// Number of successful get operations.
    pub hit_count: u64,

    /// Number of failed get operations.
    pub miss_count: u64,

    /// Number of objects evicted.
    pub eviction_count: u64,

    /// Number of put operations.
    pub put_count: u64,

    /// Number of get operations.
    pub get_count: u64,

    /// Number of pinned objects.
    pub pinned_count: usize,

    /// Size histogram: (size_bucket_kb, count).
    pub size_histogram: Vec<(usize, usize)>,
}

/// Internal structure for storing objects.
#[derive(Clone)]
pub(crate) struct StoredObject {
    /// The serialized object data.
    pub data: Bytes,

    /// Metadata about the object.
    pub metadata: Arc<ObjectMetadata>,
}

/// For future Create/Seal pattern support (internal).
#[allow(dead_code)]
pub(crate) struct PendingObject {
    /// Buffer being written to.
    pub buffer: bytes::BytesMut,

    /// Expected size (if known).
    pub expected_size: Option<usize>,

    /// Type information.
    pub type_id: std::any::TypeId,

    /// Type name for debugging.
    pub type_name: String,
}
