//! Ray-style global API for object storage and retrieval.
//!
//! This module provides the familiar `ray.put()` and `ray.get()` functions
//! that are used throughout Ray applications. These functions provide a
//! simple interface to the distributed object store.

use crate::error::Result;
use crate::object_ref::ObjectRef;
use crate::runtime;
use serde::{de::DeserializeOwned, Serialize};

/// Store an object in the global object store.
///
/// This is equivalent to `ray.put()` in Python Ray. The object is
/// serialized and stored, returning an ObjectRef that can be used
/// to retrieve it later or pass to tasks/actors.
///
/// # Example
/// ```no_run
/// # use rustyray_core::ray;
/// # async fn example() -> rustyray_core::Result<()> {
/// // Store a value
/// let data = vec![1, 2, 3, 4, 5];
/// let obj_ref = ray::put(data).await?;
///
/// // Pass the reference to tasks or actors
/// // task.remote(obj_ref).await?;
/// # Ok(())
/// # }
/// ```
pub async fn put<T>(value: T) -> Result<ObjectRef<T>>
where
    T: Serialize + DeserializeOwned + Send + Sync + 'static,
{
    let runtime = runtime::global()?;
    let task_system = runtime.task_system();
    
    // Use the task system's put method which handles both storage and notification
    task_system.put(value).await
}

/// Retrieve an object from the global object store.
///
/// This is equivalent to `ray.get()` in Python Ray. It takes an
/// ObjectRef and returns the deserialized value. This function
/// will block until the object is available.
///
/// # Example
/// ```no_run
/// # use rustyray_core::ray;
/// # async fn example() -> rustyray_core::Result<()> {
/// # let obj_ref = ray::put(vec![1, 2, 3]).await?;
/// // Retrieve a value
/// let data: Vec<i32> = ray::get(&obj_ref).await?;
/// assert_eq!(data, vec![1, 2, 3]);
/// # Ok(())
/// # }
/// ```
pub async fn get<T>(obj_ref: &ObjectRef<T>) -> Result<T>
where
    T: DeserializeOwned + Send + 'static,
{
    obj_ref.get().await
}

/// Store multiple objects in the global object store.
///
/// This is a batch version of `put()` that stores multiple objects
/// efficiently. Returns a vector of ObjectRefs in the same order
/// as the input values.
///
/// # Example
/// ```no_run
/// # use rustyray_core::ray;
/// # async fn example() -> rustyray_core::Result<()> {
/// let values = vec![1, 2, 3, 4, 5];
/// let refs = ray::put_batch(values).await?;
/// assert_eq!(refs.len(), 5);
/// # Ok(())
/// # }
/// ```
pub async fn put_batch<T>(values: Vec<T>) -> Result<Vec<ObjectRef<T>>>
where
    T: Serialize + DeserializeOwned + Send + Sync + 'static,
{
    let runtime = runtime::global()?;
    let task_system = runtime.task_system();
    
    let mut refs = Vec::with_capacity(values.len());
    
    // Use the task system's put method for each value
    for value in values {
        let obj_ref = task_system.put(value).await?;
        refs.push(obj_ref);
    }
    
    Ok(refs)
}

/// Retrieve multiple objects from the global object store.
///
/// This is a batch version of `get()` that retrieves multiple objects
/// efficiently. Returns a vector of values in the same order as the
/// input ObjectRefs.
///
/// # Example
/// ```no_run
/// # use rustyray_core::ray;
/// # async fn example() -> rustyray_core::Result<()> {
/// # let refs = ray::put_batch(vec![1, 2, 3]).await?;
/// let values: Vec<i32> = ray::get_batch(&refs).await?;
/// assert_eq!(values, vec![1, 2, 3]);
/// # Ok(())
/// # }
/// ```
pub async fn get_batch<T>(obj_refs: &[ObjectRef<T>]) -> Result<Vec<T>>
where
    T: DeserializeOwned + Send + 'static,
{
    let mut results = Vec::with_capacity(obj_refs.len());
    
    // In the future, we could optimize this with parallel fetching
    for obj_ref in obj_refs {
        results.push(obj_ref.get().await?);
    }
    
    Ok(results)
}

/// Wait for objects to be ready without retrieving them.
///
/// This is useful for synchronization - you can wait for tasks to
/// complete without retrieving their results immediately.
///
/// # Example
/// ```no_run
/// # use rustyray_core::ray;
/// # async fn example() -> rustyray_core::Result<()> {
/// # let refs = ray::put_batch(vec![1, 2, 3]).await?;
/// // Wait for all objects to be ready
/// ray::wait(&refs).await?;
/// 
/// // Now get() calls will return immediately
/// let values: Vec<i32> = ray::get_batch(&refs).await?;
/// # Ok(())
/// # }
/// ```
pub async fn wait<T>(obj_refs: &[ObjectRef<T>]) -> Result<()>
where
    T: DeserializeOwned + Send + 'static,
{
    // For now, we just check if objects exist
    // In the future, this could be optimized to avoid deserialization
    for obj_ref in obj_refs {
        let _ = obj_ref.get().await?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::with_test_runtime;
    
    #[tokio::test]
    async fn test_put_get() {
        with_test_runtime(|| async {
            // Test simple put/get
            let value = 42i32;
            let obj_ref = put(value).await.unwrap();
            let retrieved = get(&obj_ref).await.unwrap();
            assert_eq!(retrieved, 42);
            
            // Test with complex type
            let data = vec!["hello".to_string(), "world".to_string()];
            let obj_ref = put(data.clone()).await.unwrap();
            let retrieved: Vec<String> = get(&obj_ref).await.unwrap();
            assert_eq!(retrieved, data);
        }).await;
    }
    
    #[tokio::test]
    async fn test_batch_operations() {
        with_test_runtime(|| async {
            // Test batch put/get
            let values = vec![1, 2, 3, 4, 5];
            let refs = put_batch(values.clone()).await.unwrap();
            assert_eq!(refs.len(), 5);
            
            let retrieved = get_batch(&refs).await.unwrap();
            assert_eq!(retrieved, values);
        }).await;
    }
    
    #[tokio::test]
    async fn test_wait() {
        with_test_runtime(|| async {
            // Put some values
            let refs = put_batch(vec![10, 20, 30]).await.unwrap();
            
            // Wait for them (should complete immediately since we just put them)
            wait(&refs).await.unwrap();
            
            // Get should be instant
            let values = get_batch(&refs).await.unwrap();
            assert_eq!(values, vec![10, 20, 30]);
        }).await;
    }
    
    #[tokio::test]
    async fn test_error_propagation() {
        with_test_runtime(|| async {
            // Create a non-existent ObjectRef  
            let runtime = runtime::global().unwrap();
            let store = runtime.object_store();
            let fake_id = crate::types::ObjectId::new();
            let fake_ref = ObjectRef::<i32>::with_store(fake_id, store.clone());
            
            // Getting it should fail
            let result = get(&fake_ref).await;
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("not found"));
        }).await;
    }
    
    #[tokio::test]
    async fn test_concurrent_operations() {
        with_test_runtime(|| async {
            // This test verifies that the runtime supports concurrent operations
            let mut handles = vec![];
            
            // Spawn 10 concurrent tasks that all put and get objects
            for i in 0..10 {
                let handle = tokio::spawn(async move {
                    // Each task puts its own value
                    let obj_ref = put(i * 100).await.unwrap();
                    
                    // And retrieves it
                    let value = get(&obj_ref).await.unwrap();
                    assert_eq!(value, i * 100);
                    
                    // Return the value for verification
                    value
                });
                handles.push(handle);
            }
            
            // Wait for all tasks and verify results
            let mut results = vec![];
            for handle in handles {
                results.push(handle.await.unwrap());
            }
            
            // Sort results to check we got all values
            results.sort();
            assert_eq!(results, vec![0, 100, 200, 300, 400, 500, 600, 700, 800, 900]);
        }).await;
    }
}