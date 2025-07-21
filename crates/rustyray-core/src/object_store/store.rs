//! In-memory implementation of the object store using CLRU cache.

use super::traits::ObjectStore;
use super::types::{
    ObjectId, ObjectMetadata, PendingObject, PutResult, StoreConfig, StoreStats, StoredObject,
};
use crate::{Result, RustyRayError};
use async_trait::async_trait;
use bytes::Bytes;
use clru::{CLruCache, CLruCacheConfig, WeightScale};
use dashmap::DashMap;
use serde::{de::DeserializeOwned, Serialize};
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Weight implementation for CLRU cache (zero-sized type)
struct ObjectWeight;

impl WeightScale<ObjectId, StoredObject> for ObjectWeight {
    fn weight(&self, _key: &ObjectId, value: &StoredObject) -> usize {
        value.metadata.size
    }
}

/// Statistics collector for the object store.
struct StatsCollector {
    hit_count: AtomicU64,
    miss_count: AtomicU64,
    put_count: AtomicU64,
    get_count: AtomicU64,
    eviction_count: AtomicU64,
    pinned_count: AtomicUsize,
    pinned_size: AtomicUsize,
}

impl StatsCollector {
    fn new() -> Self {
        Self {
            hit_count: AtomicU64::new(0),
            miss_count: AtomicU64::new(0),
            put_count: AtomicU64::new(0),
            get_count: AtomicU64::new(0),
            eviction_count: AtomicU64::new(0),
            pinned_count: AtomicUsize::new(0),
            pinned_size: AtomicUsize::new(0),
        }
    }
}

/// In-memory object store implementation using CLRU cache.
pub struct InMemoryStore {
    /// Main cache using CLRU for strict memory enforcement.
    cache: Arc<
        Mutex<
            CLruCache<
                ObjectId,
                StoredObject,
                std::collections::hash_map::RandomState,
                ObjectWeight,
            >,
        >,
    >,

    /// Pinned objects that cannot be evicted.
    pinned: Arc<DashMap<ObjectId, StoredObject>>,

    /// Pending objects being created (for future Create/Seal).
    #[allow(dead_code)]
    pending: Arc<DashMap<ObjectId, PendingObject>>,

    /// Store statistics.
    stats: Arc<StatsCollector>,

    /// Configuration.
    config: StoreConfig,
}

impl InMemoryStore {
    /// Create a new in-memory object store.
    pub fn new(config: StoreConfig) -> Self {
        let stats = Arc::new(StatsCollector::new());

        // Configure CLRU cache with strict memory limits
        let max_capacity =
            NonZeroUsize::new(config.max_memory).unwrap_or(NonZeroUsize::new(1).unwrap());
        let cache = Arc::new(Mutex::new(CLruCache::with_config(
            CLruCacheConfig::new(max_capacity).with_scale(ObjectWeight),
        )));

        Self {
            cache,
            pinned: Arc::new(DashMap::new()),
            pending: Arc::new(DashMap::new()),
            stats,
            config,
        }
    }

    /// Serialize an object to bytes.
    fn serialize<T: Serialize>(object: &T) -> Result<Bytes> {
        // Use our existing serialization utility
        crate::task::serde_utils::serialize(object).map(|vec| Bytes::from(vec))
    }

    /// Deserialize bytes to an object.
    fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> Result<T> {
        // Use our existing deserialization utility
        crate::task::serde_utils::deserialize(bytes)
    }

    /// Put raw bytes with a specific ID (used by task system)
    pub async fn put_with_id(&self, id: ObjectId, data: Vec<u8>) -> Result<PutResult> {
        let size = data.len();

        // Check if single object exceeds total capacity
        if size > self.config.max_memory {
            return Err(RustyRayError::Internal(format!(
                "Object size ({} bytes) exceeds max memory limit ({} bytes)",
                size, self.config.max_memory
            )));
        }

        // Check if we have enough space considering pinned objects
        let pinned_size = self.stats.pinned_size.load(Ordering::Relaxed);
        let available_space = self.config.max_memory.saturating_sub(pinned_size);

        if size > available_space {
            return Err(RustyRayError::Internal(format!(
                "Cannot store object: {} bytes needed but only {} bytes available",
                size, available_space
            )));
        }

        // Create metadata - we don't know the type, so use a generic type
        let metadata = Arc::new(ObjectMetadata {
            size,
            created_at: Instant::now(),
            ref_count: Arc::new(AtomicUsize::new(1)),
            type_id: std::any::TypeId::of::<Vec<u8>>(),
            type_name: "Vec<u8>".to_string(),
            is_pinned: Arc::new(AtomicBool::new(false)),
        });

        let stored = StoredObject {
            data: data.into(),
            metadata: metadata.clone(),
        };

        // Store in cache
        let mut cache = self.cache.lock().unwrap();
        let result = cache.put_with_weight(id, stored.clone());
        drop(cache);

        // Handle the result
        match result {
            Ok(evicted) => {
                if let Some(_evicted_obj) = evicted {
                    self.stats.eviction_count.fetch_add(1, Ordering::Relaxed);
                }
            }
            Err((_returned_id, returned_obj)) => {
                return Err(RustyRayError::Internal(format!(
                    "Object too large for cache: {} bytes",
                    returned_obj.metadata.size
                )));
            }
        }

        self.stats.put_count.fetch_add(1, Ordering::Relaxed);

        Ok(PutResult { id, size })
    }
}

#[async_trait]
impl ObjectStore for InMemoryStore {
    async fn put<T>(&self, object: T) -> Result<PutResult>
    where
        T: Serialize + Send + 'static,
    {
        let id = ObjectId::new();
        let type_id = std::any::TypeId::of::<T>();
        let type_name = std::any::type_name::<T>().to_string();

        // Serialize the object
        let data = Self::serialize(&object)?;
        let size = data.len();

        // Check if single object exceeds total capacity
        if size > self.config.max_memory {
            return Err(RustyRayError::Internal(format!(
                "Object size ({} bytes) exceeds max memory limit ({} bytes)",
                size, self.config.max_memory
            )));
        }

        // Check if we have enough space considering pinned objects
        let pinned_size = self.stats.pinned_size.load(Ordering::Relaxed);
        let available_space = self.config.max_memory.saturating_sub(pinned_size);

        if size > available_space {
            return Err(RustyRayError::Internal(format!(
                "Cannot store object: {} bytes needed but only {} bytes available (pinned objects consume {} bytes)",
                size, available_space, pinned_size
            )));
        }

        // Create metadata
        let metadata = Arc::new(ObjectMetadata {
            size,
            created_at: Instant::now(),
            ref_count: Arc::new(AtomicUsize::new(1)),
            type_id,
            type_name,
            is_pinned: Arc::new(AtomicBool::new(false)),
        });

        let stored = StoredObject {
            data,
            metadata: metadata.clone(),
        };

        // Store in cache - CLRU will automatically evict if needed
        let mut cache = self.cache.lock().unwrap();

        // Check what was evicted (if anything)
        let result = cache.put_with_weight(id, stored.clone());
        drop(cache);

        // Handle the result - if Err, the object was too big
        match result {
            Ok(evicted) => {
                // Update stats for evicted items
                if let Some(_evicted_obj) = evicted {
                    self.stats.eviction_count.fetch_add(1, Ordering::Relaxed);
                }
            }
            Err((_returned_id, returned_obj)) => {
                // This shouldn't happen as we check size before, but handle it
                return Err(RustyRayError::Internal(format!(
                    "Object too large for cache: {} bytes",
                    returned_obj.metadata.size
                )));
            }
        }

        // Update stats for new item
        self.stats.put_count.fetch_add(1, Ordering::Relaxed);

        Ok(PutResult { id, size })
    }

    async fn get<T>(&self, id: ObjectId) -> Result<Option<T>>
    where
        T: DeserializeOwned + 'static,
    {
        self.stats.get_count.fetch_add(1, Ordering::Relaxed);

        // Check pinned first
        if let Some(entry) = self.pinned.get(&id) {
            self.stats.hit_count.fetch_add(1, Ordering::Relaxed);

            // Type check
            let expected_type = std::any::TypeId::of::<T>();
            if entry.metadata.type_id != expected_type {
                return Err(RustyRayError::Internal(format!(
                    "Type mismatch for {}: expected {}, found {}",
                    id,
                    std::any::type_name::<T>(),
                    entry.metadata.type_name
                )));
            }

            return Self::deserialize(&entry.data).map(Some);
        }

        // Check cache
        let mut cache = self.cache.lock().unwrap();
        if let Some(entry) = cache.get(&id) {
            self.stats.hit_count.fetch_add(1, Ordering::Relaxed);

            // Type check
            let expected_type = std::any::TypeId::of::<T>();
            if entry.metadata.type_id != expected_type {
                return Err(RustyRayError::Internal(format!(
                    "Type mismatch for {}: expected {}, found {}",
                    id,
                    std::any::type_name::<T>(),
                    entry.metadata.type_name
                )));
            }

            Self::deserialize(&entry.data).map(Some)
        } else {
            self.stats.miss_count.fetch_add(1, Ordering::Relaxed);
            Ok(None)
        }
    }

    async fn get_raw(&self, id: ObjectId) -> Result<Option<Bytes>> {
        self.stats.get_count.fetch_add(1, Ordering::Relaxed);

        // Check pinned first
        if let Some(entry) = self.pinned.get(&id) {
            self.stats.hit_count.fetch_add(1, Ordering::Relaxed);
            return Ok(Some(entry.data.clone()));
        }

        // Check cache
        let mut cache = self.cache.lock().unwrap();
        if let Some(entry) = cache.get(&id) {
            self.stats.hit_count.fetch_add(1, Ordering::Relaxed);
            Ok(Some(entry.data.clone()))
        } else {
            self.stats.miss_count.fetch_add(1, Ordering::Relaxed);
            Ok(None)
        }
    }

    async fn exists(&self, id: ObjectId) -> Result<bool> {
        if self.pinned.contains_key(&id) {
            return Ok(true);
        }

        let cache = self.cache.lock().unwrap();
        Ok(cache.peek(&id).is_some())
    }

    async fn metadata(&self, id: ObjectId) -> Result<Option<ObjectMetadata>> {
        if let Some(entry) = self.pinned.get(&id) {
            // Clone the metadata
            return Ok(Some(entry.metadata.as_ref().clone()));
        }

        let cache = self.cache.lock().unwrap();
        if let Some(entry) = cache.peek(&id) {
            // Clone the metadata
            Ok(Some(entry.metadata.as_ref().clone()))
        } else {
            Ok(None)
        }
    }

    async fn pin(&self, id: ObjectId) -> Result<()> {
        // Check if already pinned
        if self.pinned.contains_key(&id) {
            return Ok(());
        }

        // Try to get from cache first (to ensure it exists)
        let mut cache = self.cache.lock().unwrap();
        if let Some(entry) = cache.pop(&id) {
            // Mark as pinned
            entry.metadata.is_pinned.store(true, Ordering::Relaxed);
            let size = entry.metadata.size;
            // Move to pinned storage
            self.pinned.insert(id, entry);
            // Update pinned count and size
            self.stats.pinned_count.fetch_add(1, Ordering::Relaxed);
            self.stats.pinned_size.fetch_add(size, Ordering::Relaxed);
            Ok(())
        } else {
            Err(RustyRayError::Internal(format!("Object {} not found", id)))
        }
    }

    async fn unpin(&self, id: ObjectId) -> Result<()> {
        // First check if the object is actually pinned
        let entry = self.pinned.get(&id).ok_or_else(|| {
            RustyRayError::Internal(format!("Object {} not found or not pinned", id))
        })?;

        // Clone the entry while we still have the reference
        let entry_clone = entry.clone();
        drop(entry); // Release the reference

        // Try to put it in the cache BEFORE removing from pinned
        // This ensures we don't lose the object if cache insertion fails
        let mut cache = self.cache.lock().unwrap();

        let result = cache.put_with_weight(id, entry_clone.clone());
        drop(cache);

        match result {
            Ok(evicted) => {
                // Successfully inserted into cache, now safe to remove from pinned
                if let Some((_, removed_entry)) = self.pinned.remove(&id) {
                    // Mark as unpinned
                    entry_clone
                        .metadata
                        .is_pinned
                        .store(false, Ordering::Relaxed);

                    // Update stats for any evicted object
                    if let Some(_evicted_obj) = evicted {
                        self.stats.eviction_count.fetch_add(1, Ordering::Relaxed);
                    }

                    // Update pinned count and size
                    self.stats.pinned_count.fetch_sub(1, Ordering::Relaxed);
                    self.stats
                        .pinned_size
                        .fetch_sub(removed_entry.metadata.size, Ordering::Relaxed);
                    Ok(())
                } else {
                    // Object was removed from pinned between our check and remove
                    // This is rare but possible - the object might have been deleted
                    Err(RustyRayError::Internal(format!(
                        "Object {} was removed during unpin",
                        id
                    )))
                }
            }
            Err((_id, _obj)) => {
                // Failed to insert into cache - object remains pinned
                // This prevents data loss
                Err(RustyRayError::Internal(format!(
                    "Cannot unpin object {}: insufficient space in cache. Object remains pinned.",
                    id
                )))
            }
        }
    }

    async fn stats(&self) -> StoreStats {
        // Get the actual weight and count from CLRU cache
        let (cache_weight, cache_count) = {
            let cache = self.cache.lock().unwrap();
            (cache.weight(), cache.len())
        };

        // Get pinned size and count
        let pinned_size = self.stats.pinned_size.load(Ordering::Relaxed);
        let pinned_count = self.stats.pinned_count.load(Ordering::Relaxed);

        // Total size is cache weight + pinned size
        let total_size = cache_weight + pinned_size;

        // Total objects is cache count + pinned count
        let object_count = cache_count + pinned_count;

        StoreStats {
            total_objects: object_count,
            total_size,
            hit_count: self.stats.hit_count.load(Ordering::Relaxed),
            miss_count: self.stats.miss_count.load(Ordering::Relaxed),
            eviction_count: self.stats.eviction_count.load(Ordering::Relaxed),
            put_count: self.stats.put_count.load(Ordering::Relaxed),
            get_count: self.stats.get_count.load(Ordering::Relaxed),
            pinned_count,
            size_histogram: vec![], // TODO: Implement histogram
        }
    }

    async fn delete(&self, id: ObjectId) -> Result<bool> {
        // To prevent race conditions with pin/unpin operations, we need to check
        // both stores atomically. We'll use a mark-and-sweep approach:
        // 1. Try to remove from pinned first (fast path for pinned objects)
        // 2. If not found, lock cache and check there
        // 3. If still not found after locking, check pinned again in case it was just pinned

        // First, try to remove from pinned (common case for pinned objects)
        if let Some((_, entry)) = self.pinned.remove(&id) {
            // Successfully removed from pinned
            self.stats.pinned_count.fetch_sub(1, Ordering::Relaxed);
            self.stats
                .pinned_size
                .fetch_sub(entry.metadata.size, Ordering::Relaxed);
            return Ok(true);
        }

        // Not in pinned, so lock the cache and check there
        let mut cache = self.cache.lock().unwrap();
        if let Some(_entry) = cache.pop(&id) {
            // Found in cache, remove it
            drop(cache); // Release lock early

            return Ok(true);
        }

        // Still holding cache lock - check pinned one more time
        // This handles the race where object was pinned after our first check
        drop(cache); // Release cache lock first

        if let Some((_, entry)) = self.pinned.remove(&id) {
            // It was just pinned during our cache check
            self.stats.pinned_count.fetch_sub(1, Ordering::Relaxed);
            self.stats
                .pinned_size
                .fetch_sub(entry.metadata.size, Ordering::Relaxed);
            return Ok(true);
        }

        // Object not found in either store
        Ok(false)
    }

    async fn clear(&self) -> Result<()> {
        self.pinned.clear();

        let mut cache = self.cache.lock().unwrap();
        cache.clear();
        drop(cache);

        // Reset counters
        self.stats.pinned_count.store(0, Ordering::Relaxed);
        self.stats.pinned_size.store(0, Ordering::Relaxed);

        Ok(())
    }
}
