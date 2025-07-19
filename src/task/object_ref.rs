//! ObjectRef implementation - futures for task results.
//! 
//! ObjectRefs are Ray's key innovation for async dataflow. They act as
//! type-safe futures that can be passed between tasks to build dependency
//! graphs. This implementation uses local channels for now but can be
//! extended to support distributed object stores.

use crate::error::{Result, RustyRayError};
use serde::{Serialize, Deserialize, de::DeserializeOwned};
use std::fmt;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::task::{Context, Poll};
use tokio::sync::{oneshot, watch};
use std::sync::Arc;

/// Type alias for the result transported through ObjectRef channels
type ObjectResult = Result<Vec<u8>>;

/// Counter for generating unique object IDs
static OBJECT_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Unique identifier for an object in the system.
/// 
/// In the distributed version, this would include node information
/// and be globally unique across the cluster.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ObjectId(u64);

impl ObjectId {
    /// Generate a new unique object ID
    pub fn new() -> Self {
        let id = OBJECT_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        ObjectId(id)
    }
    
    /// Get the inner ID value
    pub fn as_u64(&self) -> u64 {
        self.0
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

/// Internal state for an ObjectRef that can be shared among clones
struct ObjectRefState {
    /// Channel to receive the result (for local execution)
    receiver: watch::Receiver<Option<ObjectResult>>,
}

/// A reference to a future object that may not exist yet.
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
/// This implementation uses tokio::sync::watch to allow multiple clones
/// to await the same result.
pub struct ObjectRef<T> {
    /// The unique ID of this object
    id: ObjectId,
    
    /// Shared state containing the watch receiver
    /// None for deserialized ObjectRefs (not yet implemented)
    state: Option<Arc<ObjectRefState>>,
    
    /// Phantom type for compile-time type safety
    _phantom: PhantomData<T>,
}

impl<T> ObjectRef<T> {
    /// Create a new ObjectRef with a channel for receiving the result
    pub(crate) fn new() -> (Self, oneshot::Sender<ObjectResult>) {
        let id = ObjectId::new();
        
        // Create a watch channel with None as initial value
        let (watch_tx, watch_rx) = watch::channel(None);
        
        // Create a oneshot channel for the actual sender
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        
        // Spawn a task to bridge oneshot to watch
        tokio::spawn(async move {
            if let Ok(result) = oneshot_rx.await {
                let _ = watch_tx.send(Some(result));
            }
        });
        
        let state = Arc::new(ObjectRefState {
            receiver: watch_rx,
        });
        
        let object_ref = ObjectRef {
            id,
            state: Some(state),
            _phantom: PhantomData,
        };
        
        (object_ref, oneshot_tx)
    }
    
    /// Create an ObjectRef from just an ID (for deserialization)
    /// 
    /// This is used when an ObjectRef is passed as a task argument.
    /// The state will be None and must be resolved through the object store.
    pub(crate) fn from_id(id: ObjectId) -> Self {
        ObjectRef {
            id,
            state: None,
            _phantom: PhantomData,
        }
    }
    
    /// Get the object ID
    pub fn id(&self) -> ObjectId {
        self.id
    }
    
    /// Check if this ObjectRef has a local receiver
    pub fn is_local(&self) -> bool {
        self.state.is_some()
    }
}

impl<T: DeserializeOwned + Send + 'static> ObjectRef<T> {
    /// Get the value of this ObjectRef, blocking until it's available.
    /// 
    /// This can be called multiple times on clones of the same ObjectRef.
    /// In Ray, this is equivalent to `ray.get()`.
    pub async fn get(&self) -> Result<T> {
        // For now, we only support local execution
        let state = self.state.as_ref()
            .ok_or_else(|| RustyRayError::Internal(
                "ObjectRef has no state - distributed execution not yet implemented".to_string()
            ))?;
        
        // Clone the receiver so we can await without consuming self
        let mut receiver = state.receiver.clone();
        
        // Wait for the value to become Some
        receiver.wait_for(|value| value.is_some()).await
            .map_err(|_| RustyRayError::Internal(
                format!("ObjectRef {} watch channel closed", self.id)
            ))?;
        
        // Get the result
        let result = receiver.borrow().clone()
            .ok_or_else(|| RustyRayError::Internal(
                format!("ObjectRef {} has no result after wait", self.id)
            ))?;
        
        // Handle the inner Result - this propagates task execution errors
        let bytes = result?;
        
        // Deserialize the result
        crate::task::serde_utils::deserialize(&bytes)
            .map_err(|e| RustyRayError::Internal(
                format!("Failed to deserialize object result: {}", e)
            ))
    }
}

/// Future implementation for ObjectRef
impl<T: DeserializeOwned + Send + 'static> Future for ObjectRef<T> {
    type Output = Result<T>;
    
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let state = match &self.state {
            Some(state) => state,
            None => {
                return Poll::Ready(Err(RustyRayError::Internal(
                    "ObjectRef has no state - distributed execution not yet implemented".to_string()
                )));
            }
        };
        
        // Clone the receiver so we can poll it
        let mut receiver = state.receiver.clone();
        
        // Check if we have a value
        if receiver.borrow().is_some() {
            let result = receiver.borrow().clone().unwrap();
            
            // Handle the inner Result from task execution
            match result {
                Ok(bytes) => {
                    let deserialized = crate::task::serde_utils::deserialize(&bytes)
                        .map_err(|e| RustyRayError::Internal(
                            format!("Failed to deserialize object result: {}", e)
                        ));
                    Poll::Ready(deserialized)
                }
                Err(e) => {
                    // Task execution failed - propagate the error
                    Poll::Ready(Err(e))
                }
            }
        } else {
            // Register for wakeup when value changes
            let _ = receiver.mark_changed();
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

/// Clone creates a new handle to the same object
/// The cloned ObjectRef can be awaited independently
impl<T> Clone for ObjectRef<T> {
    fn clone(&self) -> Self {
        ObjectRef {
            id: self.id,
            state: self.state.clone(), // Arc is cheap to clone
            _phantom: PhantomData,
        }
    }
}

impl<T> fmt::Debug for ObjectRef<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ObjectRef")
            .field("id", &self.id)
            .field("is_local", &self.is_local())
            .finish()
    }
}

/// Serialize ObjectRef as just its ID
impl<T> Serialize for ObjectRef<T> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.id.serialize(serializer)
    }
}

/// Deserialize ObjectRef from just an ID
impl<'de, T> Deserialize<'de> for ObjectRef<T> {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let id = ObjectId::deserialize(deserializer)?;
        Ok(ObjectRef::from_id(id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_object_ref_local() {
        let (object_ref, sender) = ObjectRef::<i32>::new();
        let _id = object_ref.id();
        
        // Send result in background
        tokio::spawn(async move {
            let bytes = crate::task::serde_utils::serialize(&42).unwrap();
            sender.send(Ok(bytes)).unwrap();
        });
        
        // Get result
        let result = object_ref.get().await.unwrap();
        assert_eq!(result, 42);
    }
    
    #[tokio::test]
    async fn test_object_ref_future() {
        let (object_ref, sender) = ObjectRef::<String>::new();
        
        // Send result in background
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            let bytes = crate::task::serde_utils::serialize(&"hello".to_string()).unwrap();
            sender.send(Ok(bytes)).unwrap();
        });
        
        // Await as future
        let result = object_ref.await.unwrap();
        assert_eq!(result, "hello");
    }
    
    #[tokio::test]
    async fn test_object_ref_serialization() {
        let (object_ref, _) = ObjectRef::<i32>::new();
        let id = object_ref.id();
        
        // Serialize
        let serialized = crate::task::serde_utils::serialize(&object_ref).unwrap();
        
        // Deserialize
        let deserialized: ObjectRef<i32> = crate::task::serde_utils::deserialize(&serialized).unwrap();
        assert_eq!(deserialized.id(), id);
        assert!(!deserialized.is_local()); // Deserialized refs don't have receivers
    }
    
    #[tokio::test]
    async fn test_object_ref_dropped_sender() {
        let (object_ref, sender) = ObjectRef::<i32>::new();
        
        // Drop sender without sending
        drop(sender);
        
        // Should get error
        let result = object_ref.get().await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_object_ref_clone_multiple_await() {
        let (object_ref, sender) = ObjectRef::<i32>::new();
        
        // Clone the ObjectRef multiple times
        let clone1 = object_ref.clone();
        let clone2 = object_ref.clone();
        let clone3 = object_ref.clone();
        
        // Send result in background
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            let bytes = crate::task::serde_utils::serialize(&42).unwrap();
            sender.send(Ok(bytes)).unwrap();
        });
        
        // All clones should be able to get the result
        let result1 = clone1.get().await.unwrap();
        let result2 = clone2.get().await.unwrap();
        let result3 = clone3.get().await.unwrap();
        let original_result = object_ref.get().await.unwrap();
        
        assert_eq!(result1, 42);
        assert_eq!(result2, 42);
        assert_eq!(result3, 42);
        assert_eq!(original_result, 42);
    }
    
    #[tokio::test]
    async fn test_object_ref_clone_future() {
        let (object_ref, sender) = ObjectRef::<String>::new();
        let clone = object_ref.clone();
        
        // Send result in background
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            let bytes = crate::task::serde_utils::serialize(&"test".to_string()).unwrap();
            sender.send(Ok(bytes)).unwrap();
        });
        
        // Both original and clone can be awaited as futures
        let (result1, result2) = tokio::join!(object_ref, clone);
        assert_eq!(result1.unwrap(), "test");
        assert_eq!(result2.unwrap(), "test");
    }
}