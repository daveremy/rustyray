//! ObjectRef implementation - futures for task results.
//!
//! ObjectRefs are Ray's key innovation for async dataflow. They act as
//! type-safe futures that can be passed between tasks to build dependency
//! graphs. This implementation is backed by the object store for consistency.

use crate::error::{Result, RustyRayError};
use crate::object_store::{InMemoryStore, ObjectStore};
use crate::types::ObjectId;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

/// A reference to an object in the object store.
///
/// ObjectRefs are the primary way to handle task results in Ray. They act
/// as futures that can be:
/// - Returned immediately when a task is submitted
/// - Passed as arguments to other tasks (creating dependencies)
/// - Awaited to get the actual result
///
/// The type parameter T provides compile-time type safety, but is erased
/// at runtime when serialized.
///
/// All ObjectRefs are backed by the object store for consistency.
pub struct ObjectRef<T> {
    /// The unique ID of this object
    id: ObjectId,

    /// Reference to the object store
    store: Arc<InMemoryStore>,

    /// Phantom data to track the type parameter
    _phantom: PhantomData<T>,
}

impl<T> ObjectRef<T> {
    /// Create a new ObjectRef backed by the object store.
    pub(crate) fn new(id: ObjectId, store: Arc<InMemoryStore>) -> Self {
        ObjectRef {
            id,
            store,
            _phantom: PhantomData,
        }
    }

    /// Get the object ID
    pub fn id(&self) -> ObjectId {
        self.id
    }

    /// Get a reference to the object store
    pub(crate) fn store(&self) -> &Arc<InMemoryStore> {
        &self.store
    }
}

impl<T: DeserializeOwned + Send + 'static> ObjectRef<T> {
    /// Get the value of this ObjectRef, blocking until it's available.
    ///
    /// This can be called multiple times on clones of the same ObjectRef.
    /// In Ray, this is equivalent to `ray.get()`.
    pub async fn get(&self) -> Result<T> {
        // Poll the object store with exponential backoff
        let mut retries = 0;
        let max_retries = 100;
        let mut delay_ms = 1;

        loop {
            // Try to get raw bytes first
            match self.store.get_raw(self.id).await? {
                Some(bytes) => {
                    // Check if this is an error result
                    const ERROR_MARKER: &[u8] = b"__RUSTYRAY_ERROR__";
                    if bytes.starts_with(ERROR_MARKER) {
                        // Extract error message
                        let error_msg = String::from_utf8_lossy(&bytes[ERROR_MARKER.len()..]);
                        return Err(RustyRayError::Internal(error_msg.to_string()));
                    }

                    // Normal result - deserialize from raw bytes
                    return crate::task::serde_utils::deserialize::<T>(&bytes).map_err(|e| {
                        RustyRayError::Internal(format!(
                            "Failed to deserialize object {}: {}",
                            self.id, e
                        ))
                    });
                }
                None => {
                    if retries >= max_retries {
                        return Err(RustyRayError::Internal(format!(
                            "Object {} not found after {} retries",
                            self.id, max_retries
                        )));
                    }

                    tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                    retries += 1;
                    delay_ms = (delay_ms * 2).min(100); // Cap at 100ms
                }
            }
        }
    }
}

/// Future implementation for ObjectRef
impl<T: DeserializeOwned + Send + 'static> Future for ObjectRef<T> {
    type Output = Result<T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Create a future for get() and poll it
        let get_future = self.get();
        tokio::pin!(get_future);
        get_future.poll(cx)
    }
}

impl<T> Clone for ObjectRef<T> {
    fn clone(&self) -> Self {
        ObjectRef {
            id: self.id,
            store: self.store.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<T> fmt::Debug for ObjectRef<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ObjectRef").field("id", &self.id).finish()
    }
}

impl<T> fmt::Display for ObjectRef<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ObjectRef({})", self.id)
    }
}

/// Serialization support for passing ObjectRefs between tasks
impl<T> Serialize for ObjectRef<T> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Serialize both the ID and store reference
        #[derive(Serialize)]
        struct ObjectRefData {
            id: ObjectId,
            // In a distributed system, this would include store location
        }

        ObjectRefData { id: self.id }.serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for ObjectRef<T> {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct ObjectRefData {
            #[allow(dead_code)]
            id: ObjectId,
        }

        let _data = ObjectRefData::deserialize(deserializer)?;

        // For now, we'll need to attach the store after deserialization
        // In a real system, this would look up the store from runtime context
        use serde::de::Error;
        Err(D::Error::custom(
            "ObjectRef deserialization requires runtime context - use TaskSystem methods",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object_store::StoreConfig;

    #[tokio::test]
    async fn test_object_ref_store_backed() {
        let store = Arc::new(InMemoryStore::new(StoreConfig::default()));
        let value = 42i32;

        // Put value in store
        let result = store.put(value).await.unwrap();

        // Create ObjectRef
        let obj_ref = ObjectRef::<i32>::new(result.id, store.clone());

        // Get value
        let result = obj_ref.get().await.unwrap();
        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn test_object_ref_clone() {
        let store = Arc::new(InMemoryStore::new(StoreConfig::default()));
        let value = "hello".to_string();

        // Put value in store
        let result = store.put(value.clone()).await.unwrap();

        // Create ObjectRef
        let obj_ref = ObjectRef::<String>::new(result.id, store.clone());
        let obj_ref_clone = obj_ref.clone();

        // Both should get the same value
        assert_eq!(obj_ref.get().await.unwrap(), "hello");
        assert_eq!(obj_ref_clone.get().await.unwrap(), "hello");
    }

    #[tokio::test]
    async fn test_object_ref_future() {
        let store = Arc::new(InMemoryStore::new(StoreConfig::default()));
        let value = vec![1, 2, 3];

        // Put value in store
        let result = store.put(value.clone()).await.unwrap();

        // Create ObjectRef
        let obj_ref = ObjectRef::<Vec<i32>>::new(result.id, store.clone());

        // Await as future
        let result = obj_ref.await.unwrap();
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn test_object_ref_not_found() {
        let store = Arc::new(InMemoryStore::new(StoreConfig::default()));
        let id = ObjectId::new();

        // Create ObjectRef for non-existent object
        let obj_ref = ObjectRef::<i32>::new(id, store.clone());

        // Should error after retries
        let result = obj_ref.get().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }
}
