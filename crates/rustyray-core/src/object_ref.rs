//! ObjectRef implementation - futures for task results.
//!
//! ObjectRefs are Ray's key innovation for async dataflow. They act as
//! type-safe futures that can be passed between tasks to build dependency
//! graphs. This implementation is backed by the object store for consistency.

use crate::error::{Result, RustyRayError};
use crate::object_store::{InMemoryStore, ObjectStore};
use crate::task::context::{ContextualDeserialize, DeserializationContext};
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
/// ObjectRefs are serializable and can be passed between tasks. The object
/// store is accessed via the global runtime when needed.
#[derive(Clone, Serialize, Deserialize)]
pub struct ObjectRef<T> {
    /// The unique ID of this object
    id: ObjectId,

    /// Optional reference to the object store (not serialized)
    #[serde(skip)]
    store: Option<Arc<InMemoryStore>>,

    /// Phantom data to track the type parameter
    #[serde(skip)]
    _phantom: PhantomData<T>,
}

impl<T> ObjectRef<T> {
    /// Create a new ObjectRef from an object ID.
    #[allow(dead_code)]
    pub(crate) fn new(id: ObjectId) -> Self {
        ObjectRef {
            id,
            store: None,
            _phantom: PhantomData,
        }
    }

    /// Create a new ObjectRef with a specific store.
    pub(crate) fn with_store(id: ObjectId, store: Arc<InMemoryStore>) -> Self {
        ObjectRef {
            id,
            store: Some(store),
            _phantom: PhantomData,
        }
    }

    /// Get the object ID
    pub fn id(&self) -> ObjectId {
        self.id
    }
}

impl<T: DeserializeOwned + Send + 'static> ObjectRef<T> {
    /// Get the value of this ObjectRef, blocking until it's available.
    ///
    /// This can be called multiple times on clones of the same ObjectRef.
    /// In Ray, this is equivalent to `ray.get()`.
    pub async fn get(&self) -> Result<T> {
        // Use the attached store if available, otherwise try global runtime
        let store = if let Some(store) = &self.store {
            store.clone()
        } else {
            // Fallback to global runtime
            let runtime =
                crate::runtime::global().map_err(|_| RustyRayError::RuntimeNotInitialized)?;
            runtime.object_store().clone()
        };

        // Poll the object store with exponential backoff
        let mut retries = 0;
        let max_retries = 100;
        let mut delay_ms = 1;

        loop {
            // Try to get raw bytes first
            match store.get_raw(self.id).await? {
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

impl<T> fmt::Debug for ObjectRef<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ObjectRef")
            .field("id", &self.id)
            .field("has_store", &self.store.is_some())
            .finish()
    }
}

impl<T> fmt::Display for ObjectRef<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ObjectRef({})", self.id)
    }
}

/// Implementation of ContextualDeserialize for ObjectRef.
///
/// This allows ObjectRef to be properly rehydrated with an object store
/// reference when deserialized in a task context.
impl<T: Send + Sync + 'static> ContextualDeserialize for ObjectRef<T> {
    fn deserialize_with_context(bytes: &[u8], context: &DeserializationContext) -> Result<Self> {
        // First deserialize just the ID using a helper struct
        #[derive(Deserialize)]
        struct ObjectRefData {
            id: ObjectId,
        }

        let data: ObjectRefData = crate::task::serde_utils::deserialize(bytes).map_err(|e| {
            RustyRayError::Internal(format!("Failed to deserialize ObjectRef: {e}"))
        })?;

        // Then create the ObjectRef with the store from context
        Ok(ObjectRef {
            id: data.id,
            store: Some(context.object_store.clone()),
            _phantom: PhantomData,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_object_ref_store_backed() {
        // Initialize runtime for test
        let _ = crate::runtime::init(); // Ignore error if already initialized

        let runtime = crate::runtime::global().unwrap();
        let store = runtime.object_store();
        let value = 42i32;

        // Put value in store
        let result = store.put(value).await.unwrap();

        // Create ObjectRef
        let obj_ref = ObjectRef::<i32>::with_store(result.id, store.clone());

        // Get value
        let result = obj_ref.get().await.unwrap();
        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn test_object_ref_clone() {
        // Initialize runtime for test (or reuse existing)
        let _ = crate::runtime::init();

        let runtime = crate::runtime::global().unwrap();
        let store = runtime.object_store();
        let value = "hello".to_string();

        // Put value in store
        let result = store.put(value.clone()).await.unwrap();

        // Create ObjectRef
        let obj_ref = ObjectRef::<String>::with_store(result.id, store.clone());
        let obj_ref_clone = obj_ref.clone();

        // Both should get the same value
        assert_eq!(obj_ref.get().await.unwrap(), "hello");
        assert_eq!(obj_ref_clone.get().await.unwrap(), "hello");
    }

    #[tokio::test]
    async fn test_object_ref_future() {
        // Initialize runtime for test (or reuse existing)
        let _ = crate::runtime::init();

        let runtime = crate::runtime::global().unwrap();
        let store = runtime.object_store();
        let value = vec![1, 2, 3];

        // Put value in store
        let result = store.put(value.clone()).await.unwrap();

        // Create ObjectRef
        let obj_ref = ObjectRef::<Vec<i32>>::with_store(result.id, store.clone());

        // Await as future
        let result = obj_ref.await.unwrap();
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn test_object_ref_not_found() {
        // Initialize runtime for test (or reuse existing)
        let _ = crate::runtime::init();

        let id = ObjectId::new();

        // Create ObjectRef for non-existent object
        let runtime = crate::runtime::global().unwrap();
        let store = runtime.object_store();
        let obj_ref = ObjectRef::<i32>::with_store(id, store.clone());

        // Should error after retries
        let result = obj_ref.get().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_object_ref_serialization() {
        let obj_ref = ObjectRef::<i32>::new(ObjectId::new());

        // Test bincode serialization (what tasks actually use)
        let encoded = crate::task::serde_utils::serialize(&obj_ref).unwrap();
        let decoded: ObjectRef<i32> = crate::task::serde_utils::deserialize(&encoded).unwrap();
        assert_eq!(obj_ref.id(), decoded.id());
    }
}
