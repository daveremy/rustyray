# Phase 5: Reference Counting Design

## Overview

This document outlines the design for implementing reference counting in RustyRay to enable safe automatic memory management and prevent premature object eviction.

## Goals

1. **Prevent Premature Eviction**: Objects with active references cannot be evicted
2. **Automatic Cleanup**: Objects with zero references can be safely evicted
3. **Memory Leak Prevention**: Detect and report objects that are never freed
4. **Minimal Performance Impact**: <5% overhead on object operations

## Design Decisions

### 1. Reference Counting Strategy

**Decision: Owner-Based Reference Counting** (like Ray)

**Rationale:**
- Simpler than fully distributed consensus
- Single source of truth for each object
- Easier to debug and reason about
- Natural fit with task/actor creation model

**Alternative Considered:** Fully distributed reference counting
- More complex, requires consensus
- Higher overhead for coordination
- Deferred to future enhancement

### 2. Implementation Approach

#### ObjectRef Lifecycle Integration

```rust
impl<T> ObjectRef<T> {
    pub(crate) fn new(id: ObjectId, store: Arc<InMemoryStore>) -> Self {
        // Increment reference count on creation
        store.increment_ref_count(id);
        
        Self {
            id,
            store,
            _phantom: PhantomData,
        }
    }
}

impl<T> Clone for ObjectRef<T> {
    fn clone(&self) -> Self {
        // Increment on clone
        self.store.increment_ref_count(self.id);
        
        Self {
            id: self.id,
            store: self.store.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<T> Drop for ObjectRef<T> {
    fn drop(&mut self) {
        // Decrement on drop
        self.store.decrement_ref_count(self.id);
    }
}
```

#### Object Store Integration

```rust
pub struct ObjectEntry {
    pub data: Vec<u8>,
    pub ref_count: AtomicU32,
    pub created_at: SystemTime,
    pub last_accessed: AtomicU64, // For LRU
    pub is_pinned: AtomicBool,
}

impl InMemoryStore {
    pub(crate) fn increment_ref_count(&self, id: ObjectId) {
        if let Some(entry) = self.objects.get(&id) {
            entry.ref_count.fetch_add(1, Ordering::AcqRel);
        }
    }
    
    pub(crate) fn decrement_ref_count(&self, id: ObjectId) {
        if let Some(entry) = self.objects.get(&id) {
            let prev = entry.ref_count.fetch_sub(1, Ordering::AcqRel);
            
            // Check for underflow
            if prev == 0 {
                log::error!("Reference count underflow for object {}", id);
            }
        }
    }
    
    fn can_evict(&self, entry: &ObjectEntry) -> bool {
        !entry.is_pinned.load(Ordering::Acquire) &&
        entry.ref_count.load(Ordering::Acquire) == 0
    }
}
```

### 3. Task Reference Tracking

When ObjectRefs are passed to tasks, we need to track them separately:

```rust
pub struct TaskRefCount {
    /// ObjectRefs passed as arguments to this task
    pub argument_refs: Vec<ObjectId>,
    /// ObjectRefs this task might create
    pub potential_outputs: Vec<ObjectId>,
}

impl TaskSystem {
    pub async fn submit_with_refs<T>(
        &self,
        spec: TaskSpec,
        refs: Vec<ObjectId>
    ) -> Result<ObjectRef<T>> {
        // Track refs for this task
        for id in &refs {
            self.object_store.increment_ref_count(*id);
        }
        
        // Store task ref info for cleanup on completion
        let task_id = spec.task_id;
        self.task_refs.insert(task_id, TaskRefCount {
            argument_refs: refs,
            potential_outputs: vec![output_id],
        });
        
        // Submit task...
    }
    
    fn on_task_complete(&self, task_id: TaskId) {
        if let Some(refs) = self.task_refs.remove(&task_id) {
            // Decrement refs for arguments
            for id in refs.argument_refs {
                self.object_store.decrement_ref_count(id);
            }
        }
    }
}
```

### 4. Eviction Safety

Modify the LRU eviction to respect reference counts:

```rust
impl InMemoryStore {
    fn evict_one(&self) -> Result<()> {
        let mut eviction_candidates = Vec::new();
        
        // Find objects with zero references
        for entry in self.objects.iter() {
            if self.can_evict(&entry) {
                eviction_candidates.push((
                    entry.key().clone(),
                    entry.last_accessed.load(Ordering::Relaxed)
                ));
            }
        }
        
        if eviction_candidates.is_empty() {
            return Err(RustyRayError::Internal(
                "No objects available for eviction".to_string()
            ));
        }
        
        // Sort by last accessed time (LRU)
        eviction_candidates.sort_by_key(|(_, last_accessed)| *last_accessed);
        
        // Evict the least recently used
        let (id, _) = eviction_candidates.first().unwrap();
        self.objects.remove(id);
        
        Ok(())
    }
}
```

### 5. Debugging and Metrics

Add comprehensive debugging support:

```rust
#[derive(Debug, Clone)]
pub struct RefCountStats {
    pub total_objects: usize,
    pub objects_with_refs: usize,
    pub total_ref_count: u64,
    pub potential_leaks: Vec<ObjectId>, // Objects with high ref counts
}

impl InMemoryStore {
    pub async fn ref_count_stats(&self) -> RefCountStats {
        let mut stats = RefCountStats::default();
        
        for entry in self.objects.iter() {
            stats.total_objects += 1;
            let ref_count = entry.ref_count.load(Ordering::Relaxed);
            
            if ref_count > 0 {
                stats.objects_with_refs += 1;
                stats.total_ref_count += ref_count as u64;
                
                // Flag potential leaks (arbitrary threshold)
                if ref_count > 1000 {
                    stats.potential_leaks.push(entry.key().clone());
                }
            }
        }
        
        stats
    }
    
    pub fn enable_ref_count_logging(&self) {
        // Add detailed logging for debugging
        self.debug_ref_counting.store(true, Ordering::Release);
    }
}
```

## Testing Strategy

### 1. Unit Tests
- Test increment/decrement operations
- Test eviction respects ref counts
- Test reference count underflow detection
- Test Clone and Drop behavior

### 2. Integration Tests
- Test task argument reference tracking
- Test circular reference scenarios
- Test high-concurrency reference counting
- Test memory leak detection

### 3. Stress Tests
- Create/destroy millions of ObjectRefs
- Verify no memory leaks
- Measure performance impact

## Migration Plan

1. **Add ref_count field**: Update ObjectEntry struct
2. **Update ObjectRef**: Add lifecycle hooks
3. **Update eviction**: Check ref counts
4. **Add task tracking**: Increment for task args
5. **Add metrics**: Debugging and monitoring

## Performance Considerations

- Use atomic operations for thread safety
- Batch reference count updates where possible
- Consider sharding for high-contention objects
- Profile impact on common operations

## Future Enhancements

1. **Distributed Reference Counting**: For multi-node operation
2. **Weak References**: For cache-friendly patterns
3. **Cycle Detection**: For circular reference cleanup
4. **Reference Count Compression**: For objects with many refs

## Success Criteria

1. No objects evicted while referenced
2. All objects eventually evictable when unreferenced  
3. <5% performance overhead
4. No memory leaks in stress tests
5. Clear debugging tools for production issues