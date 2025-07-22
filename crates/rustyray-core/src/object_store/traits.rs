//! Traits defining the object store interface.

use super::types::{ObjectId, ObjectMetadata, PutResult, StoreStats};
use crate::Result;
use async_trait::async_trait;
use bytes::Bytes;
use serde::{de::DeserializeOwned, Serialize};

/// The main trait for object storage implementations.
#[async_trait]
pub trait ObjectStore: Send + Sync {
    /// Store an object and return its ID.
    ///
    /// The object is serialized using bincode before storage.
    async fn put<T>(&self, object: T) -> Result<PutResult>
    where
        T: Serialize + Send + 'static;

    /// Retrieve an object by ID.
    ///
    /// Returns None if the object doesn't exist or has been evicted.
    /// The object is deserialized from bincode format.
    async fn get<T>(&self, id: ObjectId) -> Result<Option<T>>
    where
        T: DeserializeOwned + 'static;

    /// Get raw bytes without deserialization (zero-copy).
    ///
    /// This is useful when you need to pass the data along without
    /// deserializing it, or when implementing custom serialization.
    async fn get_raw(&self, id: ObjectId) -> Result<Option<Bytes>>;

    /// Check if an object exists in the store.
    async fn exists(&self, id: ObjectId) -> Result<bool>;

    /// Get metadata about an object.
    ///
    /// Returns None if the object doesn't exist.
    async fn metadata(&self, id: ObjectId) -> Result<Option<ObjectMetadata>>;

    /// Pin an object in memory to prevent eviction.
    ///
    /// Pinned objects will not be evicted even when the store is full.
    /// Returns an error if the object doesn't exist.
    async fn pin(&self, id: ObjectId) -> Result<()>;

    /// Unpin an object, allowing it to be evicted.
    ///
    /// When unpinning an object, it is moved back to the main cache.
    /// If the cache is at capacity, this operation may cause other objects
    /// to be evicted to make room for the unpinned object.
    ///
    /// Returns an error if the object doesn't exist or is not pinned.
    async fn unpin(&self, id: ObjectId) -> Result<()>;

    /// Get comprehensive store statistics.
    async fn stats(&self) -> StoreStats;

    /// Delete an object from the store.
    ///
    /// This immediately removes the object, regardless of reference count.
    /// Use with caution.
    async fn delete(&self, id: ObjectId) -> Result<bool>;

    /// Clear all objects from the store.
    ///
    /// This is a destructive operation that removes all stored objects.
    async fn clear(&self) -> Result<()>;
}

// Future support for Create/Seal pattern (internal until implemented)
#[async_trait]
#[allow(dead_code)]
pub(crate) trait CreateSealStore: ObjectStore {
    /// Create a buffer for direct writing.
    ///
    /// This allocates space in the store and returns a handle for writing.
    async fn create(&self, size: usize) -> Result<ObjectBuilder>;
}

/// Handle for building objects with the Create/Seal pattern.
#[allow(dead_code)]
pub(crate) struct ObjectBuilder {
    /// The ID that will be assigned to this object.
    pub id: ObjectId,

    /// Internal implementation details.
    _private: (),
}

#[allow(dead_code)]
impl ObjectBuilder {
    /// Write data to the object being built.
    pub fn write(&mut self, _data: &[u8]) -> Result<()> {
        // TODO: Implement when we add Create/Seal support
        unimplemented!("Create/Seal pattern will be implemented in a future phase")
    }

    /// Seal the object, making it immutable and available for reading.
    pub async fn seal(self) -> Result<ObjectId> {
        // TODO: Implement when we add Create/Seal support
        unimplemented!("Create/Seal pattern will be implemented in a future phase")
    }
}
