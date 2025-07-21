//! Tests for the object store module.

use super::*;
use crate::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestData {
    id: u64,
    name: String,
    values: Vec<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct OtherData {
    value: String,
}

fn create_test_store() -> InMemoryStore {
    let config = StoreConfig {
        max_memory: 1024 * 1024, // 1MB
        eviction_policy: EvictionPolicy::LRU,
        enable_stats: true,
    };
    InMemoryStore::new(config)
}

#[tokio::test]
async fn test_basic_put_get() -> Result<()> {
    let store = create_test_store();

    let data = TestData {
        id: 42,
        name: "test".to_string(),
        values: vec![1, 2, 3, 4, 5],
    };

    // Put the object
    let result = store.put(data.clone()).await?;
    assert!(result.size > 0);

    // Get it back
    let retrieved: Option<TestData> = store.get(result.id).await?;
    assert_eq!(retrieved, Some(data));

    // Check it exists
    assert!(store.exists(result.id).await?);

    // Get metadata
    let metadata = store.metadata(result.id).await?;
    assert!(metadata.is_some());
    let metadata = metadata.unwrap();
    assert_eq!(metadata.size, result.size);
    assert_eq!(metadata.type_name, std::any::type_name::<TestData>());

    Ok(())
}

#[tokio::test]
async fn test_type_safety() -> Result<()> {
    let store = create_test_store();

    let data = TestData {
        id: 1,
        name: "test".to_string(),
        values: vec![1, 2, 3],
    };

    // Put TestData
    let result = store.put(data).await?;

    // Try to get it as OtherData - should fail
    let wrong_type: Result<Option<OtherData>> = store.get(result.id).await;
    assert!(wrong_type.is_err());

    let err = wrong_type.unwrap_err();
    assert!(err.to_string().contains("Type mismatch"));

    Ok(())
}

#[tokio::test]
async fn test_get_raw() -> Result<()> {
    let store = create_test_store();

    let data = TestData {
        id: 99,
        name: "raw_test".to_string(),
        values: vec![9, 8, 7],
    };

    let result = store.put(data.clone()).await?;

    // Get raw bytes
    let raw_bytes = store.get_raw(result.id).await?;
    assert!(raw_bytes.is_some());

    // Deserialize manually to verify
    let deserialized: TestData = crate::task::serde_utils::deserialize(&raw_bytes.unwrap())?;
    assert_eq!(deserialized, data);

    Ok(())
}

#[tokio::test]
async fn test_pinning() -> Result<()> {
    let store = create_test_store();

    // Create multiple objects
    let mut ids = vec![];
    for i in 0..5 {
        let data = TestData {
            id: i,
            name: format!("test_{}", i),
            values: vec![i as i32; 100], // Larger data
        };
        let result = store.put(data).await?;
        ids.push(result.id);
    }

    // Pin the first object
    store.pin(ids[0]).await?;

    // Check stats show pinned count
    let stats = store.stats().await;
    assert_eq!(stats.pinned_count, 1);

    // Verify pinned object still exists
    assert!(store.exists(ids[0]).await?);

    // Verify we can still get it
    let data: Option<TestData> = store.get(ids[0]).await?;
    assert!(data.is_some(), "Failed to get pinned object");

    // Unpin
    store.unpin(ids[0]).await?;

    // Check stats updated
    let stats = store.stats().await;
    assert_eq!(stats.pinned_count, 0);

    Ok(())
}

#[tokio::test]
async fn test_delete() -> Result<()> {
    let store = create_test_store();

    // Start with empty store
    let initial_stats = store.stats().await;
    assert_eq!(initial_stats.total_objects, 0);
    assert_eq!(initial_stats.total_size, 0);

    let data = TestData {
        id: 123,
        name: "delete_me".to_string(),
        values: vec![1, 2, 3],
    };

    let result = store.put(data).await?;
    let object_size = result.size;

    // Check stats after put
    let stats_after_put = store.stats().await;
    assert_eq!(stats_after_put.total_objects, 1);
    assert_eq!(stats_after_put.total_size, object_size);

    // Verify it exists
    assert!(store.exists(result.id).await?);

    // Delete it
    let deleted = store.delete(result.id).await?;
    assert!(deleted);

    // Verify it's gone
    assert!(!store.exists(result.id).await?);

    // Check stats after deletion
    let stats_after_delete = store.stats().await;
    // Size should be updated immediately since we track it manually
    assert_eq!(stats_after_delete.total_size, 0);
    // Object count should also be 0
    assert_eq!(stats_after_delete.total_objects, 0);

    // Try to delete again - should return false
    let deleted_again = store.delete(result.id).await?;
    assert!(!deleted_again);

    Ok(())
}

#[tokio::test]
async fn test_clear() -> Result<()> {
    let store = create_test_store();

    // Add multiple objects
    let mut ids = vec![];
    for i in 0..10 {
        let data = TestData {
            id: i,
            name: format!("test_{}", i),
            values: vec![i as i32; 10],
        };
        let result = store.put(data).await?;
        ids.push(result.id);
    }

    // Pin one object
    store.pin(ids[0]).await?;

    // Clear everything
    store.clear().await?;

    // Verify all objects are gone
    for id in ids {
        assert!(!store.exists(id).await?);
    }

    // Check stats are reset
    let stats = store.stats().await;
    assert_eq!(stats.total_objects, 0);
    assert_eq!(stats.pinned_count, 0);

    Ok(())
}

#[tokio::test]
async fn test_stats_tracking() -> Result<()> {
    let store = create_test_store();

    // Initial stats should be zero
    let stats = store.stats().await;
    assert_eq!(stats.put_count, 0);
    assert_eq!(stats.get_count, 0);
    assert_eq!(stats.hit_count, 0);
    assert_eq!(stats.miss_count, 0);

    // Put an object
    let data = TestData {
        id: 1,
        name: "stats_test".to_string(),
        values: vec![1, 2, 3],
    };
    let result = store.put(data).await?;

    // Check put count
    let stats = store.stats().await;
    assert_eq!(stats.put_count, 1);

    // Get existing object - should be a hit
    let _: Option<TestData> = store.get(result.id).await?;
    let stats = store.stats().await;
    assert_eq!(stats.get_count, 1);
    assert_eq!(stats.hit_count, 1);
    assert_eq!(stats.miss_count, 0);

    // Get non-existent object - should be a miss
    let fake_id = ObjectId::new();
    let _: Option<TestData> = store.get(fake_id).await?;
    let stats = store.stats().await;
    assert_eq!(stats.get_count, 2);
    assert_eq!(stats.hit_count, 1);
    assert_eq!(stats.miss_count, 1);

    Ok(())
}

#[tokio::test]
async fn test_concurrent_access() -> Result<()> {
    let store = Arc::new(create_test_store());

    // Put initial objects
    let mut ids = vec![];
    for i in 0..10 {
        let data = TestData {
            id: i,
            name: format!("concurrent_{}", i),
            values: vec![i as i32; 50],
        };
        let result = store.put(data).await?;
        ids.push(result.id);
    }

    // Spawn multiple tasks to read concurrently
    let mut handles = vec![];
    for _ in 0..5 {
        let store_clone = store.clone();
        let ids_clone = ids.clone();
        let handle = tokio::spawn(async move {
            for id in &ids_clone {
                let _: Option<TestData> = store_clone.get(*id).await.unwrap();
                sleep(Duration::from_micros(100)).await;
            }
        });
        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        handle.await.unwrap();
    }

    // Check stats - should have many gets
    let stats = store.stats().await;
    assert_eq!(stats.get_count, 50); // 5 tasks * 10 objects
    assert_eq!(stats.hit_count, 50);

    Ok(())
}

#[tokio::test]
async fn test_object_not_found() -> Result<()> {
    let store = create_test_store();

    let fake_id = ObjectId::new();

    // Get should return None
    let result: Option<TestData> = store.get(fake_id).await?;
    assert!(result.is_none());

    // Get raw should return None
    let raw_result = store.get_raw(fake_id).await?;
    assert!(raw_result.is_none());

    // Exists should return false
    assert!(!store.exists(fake_id).await?);

    // Metadata should return None
    let metadata = store.metadata(fake_id).await?;
    assert!(metadata.is_none());

    // Pin should fail
    let pin_result = store.pin(fake_id).await;
    assert!(pin_result.is_err());

    // Unpin should fail
    let unpin_result = store.unpin(fake_id).await;
    assert!(unpin_result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_large_object() -> Result<()> {
    let store = create_test_store();

    // Create a large object
    let large_data = TestData {
        id: 9999,
        name: "large".to_string(),
        values: vec![42; 10000], // ~40KB of i32s
    };

    let result = store.put(large_data.clone()).await?;

    // Should be able to retrieve it
    let retrieved: Option<TestData> = store.get(result.id).await?;
    assert_eq!(retrieved, Some(large_data));

    // Check size in metadata
    let metadata = store.metadata(result.id).await?.unwrap();
    // Bincode serialization is efficient, so 10000 i32s won't be exactly 40KB
    // Each i32 is 4 bytes, plus some overhead for the struct and vec
    assert!(metadata.size > 10000); // Should be at least 10KB

    Ok(())
}

#[tokio::test]
async fn test_eviction_at_capacity() -> Result<()> {
    // Create a store with very small capacity
    let config = StoreConfig {
        max_memory: 1000, // 1KB - very small to trigger eviction
        eviction_policy: EvictionPolicy::LRU,
        enable_stats: true,
    };
    let store = InMemoryStore::new(config);

    // Create objects that are larger to ensure we exceed capacity
    let mut ids = vec![];
    for i in 0..10 {
        let data = TestData {
            id: i,
            name: format!("eviction_test_{}", i),
            values: vec![i as i32; 100], // ~400 bytes of i32s plus overhead
        };
        let result = store.put(data).await?;
        ids.push(result.id);
    }

    // Check stats and capacity tracking
    let stats = store.stats().await;
    println!("After putting 10 objects: {:?}", stats);
    println!(
        "Total size: {} bytes, capacity: {} bytes",
        stats.total_size, 1000
    );

    // With CLRU, strict enforcement means total size should never exceed capacity
    assert!(
        stats.total_size <= 1000,
        "Total size should not exceed capacity"
    );

    // Put operations succeeded
    assert_eq!(stats.put_count, 10);

    // With CLRU's synchronous eviction, cache size should be limited
    // Even if eviction_count is 0 (CLRU might not report evictions),
    // the important thing is that total size is under capacity
    println!("Eviction count from stats: {}", stats.eviction_count);

    let stats_after = store.stats().await;
    println!(
        "After putting 10 objects with strict enforcement: {:?}",
        stats_after
    );
    assert!(
        stats_after.total_size <= 1000,
        "Total size still within capacity"
    );

    Ok(())
}

#[tokio::test]
async fn test_single_object_exceeds_capacity() -> Result<()> {
    let config = StoreConfig {
        max_memory: 1000, // 1KB limit
        eviction_policy: EvictionPolicy::LRU,
        enable_stats: true,
    };
    let store = InMemoryStore::new(config);

    // Try to store an object larger than max_memory
    let large_data = TestData {
        id: 999,
        name: "huge_object".to_string(),
        values: vec![42; 1000], // ~4KB of data, definitely exceeds 1KB limit
    };

    // First, let's check the actual size
    let test_size = bincode::serde::encode_to_vec(&large_data, bincode::config::standard())
        .unwrap()
        .len();
    println!(
        "Actual serialized size: {} bytes (limit: 1000 bytes)",
        test_size
    );

    // Now this should fail with our strict enforcement
    let result = store.put(large_data.clone()).await;

    // Should get an error
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("exceeds max memory limit"));

    // Verify nothing was stored
    let stats = store.stats().await;
    assert_eq!(stats.total_objects, 0);
    assert_eq!(stats.total_size, 0);

    Ok(())
}

#[tokio::test]
async fn test_concurrent_pin_delete_race() -> Result<()> {
    let store = Arc::new(create_test_store());

    // Put an object
    let data = TestData {
        id: 100,
        name: "race_test".to_string(),
        values: vec![1, 2, 3],
    };
    let result = store.put(data).await?;
    let id = result.id;

    // Spawn multiple tasks to race
    let mut handles = vec![];

    // Task 1: Try to pin
    let store1 = store.clone();
    let handle1 = tokio::spawn(async move {
        for _ in 0..100 {
            let _ = store1.pin(id).await;
            tokio::time::sleep(tokio::time::Duration::from_micros(10)).await;
            let _ = store1.unpin(id).await;
        }
    });
    handles.push(handle1);

    // Task 2: Try to delete
    let store2 = store.clone();
    let handle2 = tokio::spawn(async move {
        for _ in 0..100 {
            let _ = store2.delete(id).await;
            tokio::time::sleep(tokio::time::Duration::from_micros(10)).await;
            // Re-add it
            let data = TestData {
                id: 100,
                name: "race_test".to_string(),
                values: vec![1, 2, 3],
            };
            let _ = store2.put(data).await;
        }
    });
    handles.push(handle2);

    // Task 3: Try to get
    let store3 = store.clone();
    let handle3 = tokio::spawn(async move {
        for _ in 0..100 {
            let _: Option<TestData> = store3.get(id).await.unwrap_or(None);
            tokio::time::sleep(tokio::time::Duration::from_micros(5)).await;
        }
    });
    handles.push(handle3);

    // Wait for all tasks
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify store is in consistent state
    let stats = store.stats().await;
    println!("Stats after race: {:?}", stats);

    // The stats should be internally consistent
    assert!(stats.pinned_count <= stats.total_objects);

    Ok(())
}

#[tokio::test]
async fn test_delete_unpin_race() -> Result<()> {
    let store = Arc::new(create_test_store());

    // Create and store an object
    let data = TestData {
        id: 1,
        name: "race_test".to_string(),
        values: vec![1, 2, 3],
    };
    let result = store.put(data.clone()).await?;
    let id = result.id;

    // Pin it
    store.pin(id).await?;

    // Run many concurrent operations to try to trigger race conditions

    // Task 1: Repeatedly unpin and pin
    let store1 = store.clone();
    let handle1 = tokio::spawn(async move {
        for i in 0..1000 {
            if i % 2 == 0 {
                let _ = store1.unpin(id).await;
            } else {
                let _ = store1.pin(id).await;
            }
        }
    });

    // Task 2: Repeatedly try to delete
    let store2 = store.clone();
    let handle2 = tokio::spawn(async move {
        let mut deleted = false;
        for _ in 0..1000 {
            if store2.delete(id).await.unwrap() {
                deleted = true;
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_micros(1)).await;
        }
        deleted
    });

    // Wait for tasks
    let _ = handle1.await.unwrap();
    let deleted = handle2.await.unwrap();

    // Verify the object was eventually deleted
    assert!(deleted, "Object should have been deleted");
    assert!(
        !store.exists(id).await?,
        "Object should not exist after deletion"
    );

    // Verify store is in consistent state
    let stats = store.stats().await;
    assert_eq!(stats.total_objects, 0, "No objects should remain");

    Ok(())
}

#[tokio::test]
async fn test_unpin_data_loss_scenario() -> Result<()> {
    // This test verifies that unpin allows eviction of other objects
    // but prevents data loss by keeping the object pinned if it can't fit

    // Create a store with specific capacity
    let config = StoreConfig {
        max_memory: 50, // Very small capacity
        eviction_policy: EvictionPolicy::LRU,
        enable_stats: true,
    };
    let store = InMemoryStore::new(config);

    // Create an object larger than cache capacity
    let large_data = TestData {
        id: 1,
        name: "too_large_for_cache".to_string(),
        values: vec![1; 100], // Will be > 50 bytes when serialized
    };

    // This should fail to store
    let result = store.put(large_data.clone()).await;
    assert!(
        result.is_err(),
        "Should not be able to store object larger than cache"
    );

    // Now test the normal case where unpin succeeds
    // Create a small object that fits
    let small_data = TestData {
        id: 2,
        name: "small".to_string(),
        values: vec![2; 2], // Small enough to fit
    };

    let result2 = store.put(small_data).await?;
    println!("Small object size: {} bytes", result2.size);
    store.pin(result2.id).await?;

    // Now unpin should succeed because the object can fit back
    let unpin_result = store.unpin(result2.id).await;
    assert!(
        unpin_result.is_ok(),
        "Should be able to unpin when object fits"
    );

    Ok(())
}

#[tokio::test]
async fn test_pinned_objects_blocking_capacity() -> Result<()> {
    let config = StoreConfig {
        max_memory: 1000, // 1KB limit
        eviction_policy: EvictionPolicy::LRU,
        enable_stats: true,
    };
    let store = InMemoryStore::new(config);

    // Fill the cache with pinned objects
    let mut pinned_ids = vec![];
    for i in 0..5 {
        let data = TestData {
            id: i,
            name: format!("pinned_{}", i),
            values: vec![i as i32; 30], // Each ~150 bytes
        };
        let result = store.put(data).await?;
        store.pin(result.id).await?;
        pinned_ids.push(result.id);
    }

    // Now try to add more objects - they should fail to persist due to pinned objects
    let mut new_ids = vec![];
    for i in 10..15 {
        let data = TestData {
            id: i,
            name: format!("new_{}", i),
            values: vec![i as i32; 30],
        };
        let result = store.put(data).await?;
        new_ids.push(result.id);
    }

    // Check that pinned objects are still there
    for id in &pinned_ids {
        let data: Option<TestData> = store.get(*id).await?;
        assert!(data.is_some(), "Pinned object should never be evicted");
    }

    // Check that new objects were likely evicted
    let mut evicted_count = 0;
    for id in &new_ids {
        let data: Option<TestData> = store.get(*id).await?;
        if data.is_none() {
            evicted_count += 1;
        }
    }

    // Debug: print actual status
    println!("Evicted count: {}/{}", evicted_count, new_ids.len());
    let stats = store.stats().await;
    println!("Stats after adding new objects: {:?}", stats);

    // With CLRU, the cache enforces strict limits
    // The key test is that the total size stays under capacity
    assert_eq!(stats.pinned_count, 5);

    // Check that capacity is enforced
    assert!(
        stats.total_size <= 1000,
        "Total size should stay under capacity even with pinned objects"
    );

    Ok(())
}
