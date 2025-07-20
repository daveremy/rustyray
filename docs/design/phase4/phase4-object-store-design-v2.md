# Phase 4: Object Store Design (Updated)

## Overview

The RustyRay Object Store is a critical component that provides efficient in-memory storage and sharing of serialized objects between tasks and actors. It's inspired by Ray's Plasma store but adapted for Rust's ownership model and our incremental development approach.

### What is Plasma?

Plasma is Ray's distributed shared-memory object store that enables:
- **Zero-copy data sharing** between processes via memory mapping (mmap)
- **Language-agnostic** object sharing (Python, C++, Java)
- **Fault isolation** - survives individual worker crashes
- **Create/Seal pattern** for memory-efficient object creation

In RustyRay, we're building incrementally toward this vision:
- **Phase 4** (Current): In-process object store with `bytes::Bytes`
- **Phase 4.5** (Future): Shared memory with `memmap2`
- **Phase 5** (Future): Full distributed object store

**Key Updates Based on Review:**
- Use `moka` crate for proven LRU implementation
- Design for future Create/Seal pattern support
- Enhanced stats collection for debugging
- Explicit handling of contention concerns

## Goals

1. **Zero-copy data sharing** - Multiple readers without data duplication
2. **Type safety** - Compile-time guarantees for object types
3. **Memory efficiency** - Automatic garbage collection via reference counting
4. **Extensibility** - Support for shared memory and spilling in future phases
5. **Performance** - Minimal overhead for local object access

## Architecture

### Core Components

```
┌─────────────────────────────────────────────────────────────┐
│                        TaskSystem                           │
│  ┌─────────────┐    ┌──────────────┐    ┌──────────────┐  │
│  │ TaskManager │───▶│ ObjectStore  │◀───│ ActorSystem  │  │
│  └─────────────┘    └──────────────┘    └──────────────┘  │
│                             │                               │
│                    ┌────────┴────────┐                      │
│                    │                 │                      │
│              ┌─────▼─────┐    ┌─────▼─────┐               │
│              │InMemStore │    │SharedMem  │               │
│              │(Phase 4)  │    │(Future)   │               │
│              └───────────┘    └───────────┘               │
└─────────────────────────────────────────────────────────────┘
```

### Key Types

```rust
/// Unique identifier for objects in the store
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct ObjectId(u64);

/// Metadata about stored objects
#[derive(Debug)]
pub struct ObjectMetadata {
    pub size: usize,
    pub created_at: Instant,
    pub last_accessed: AtomicInstant, // For LRU tracking
    pub ref_count: Arc<AtomicUsize>,
    pub object_type: TypeId,
    pub is_pinned: AtomicBool,
}

/// Enhanced store statistics
#[derive(Debug, Clone)]
pub struct StoreStats {
    pub total_objects: usize,
    pub total_size: usize,
    pub hit_count: u64,
    pub miss_count: u64,
    pub eviction_count: u64,
    pub put_count: u64,
    pub get_count: u64,
    pub size_histogram: Vec<(usize, usize)>, // (size_bucket, count)
}
```

## Design Decisions

### 1. Storage Strategy

**Decision**: Use `bytes::Bytes` for storage
- **Rationale**: 
  - Zero-copy cloning via reference counting
  - Efficient slicing and sharing
  - Already used in our codebase
  - Compatible with future shared memory implementation

### 2. Cache Implementation

**Decision**: Use `moka` crate for LRU cache *(Updated based on feedback)*
- **Rationale**:
  - Battle-tested, high-performance concurrent cache
  - Built-in support for LRU, LFU, and time-based eviction
  - Avoids complex manual implementation
  - Reduces lock contention through internal sharding

### 3. Memory Management

**Decision**: Dual-layer approach with `moka` cache + pinned objects
- **Rationale**:
  - `moka` handles automatic eviction of unpinned objects
  - Separate `DashMap` for pinned objects that must not be evicted
  - Reduces contention on size counters through sharding

### 4. Future Create/Seal Support *(New)*

**Decision**: Design internal API to support future Create/Seal pattern
- **Rationale**:
  - Avoids memory spikes for large objects
  - Enables direct writing into store memory
  - Smooth migration path to shared memory

**What is Create/Seal?**
The Create/Seal pattern allows writing data directly into the object store's memory:
1. **Create**: Allocate buffer in store, get direct write access
2. **Write**: Stream data directly into the buffer (no intermediate copies)
3. **Seal**: Make object immutable and available to readers

This avoids the traditional pattern where objects must exist fully-formed in worker memory before copying to the store, reducing peak memory usage by ~50% for large objects.

## API Design

### ObjectStore Trait

```rust
#[async_trait]
pub trait ObjectStore: Send + Sync {
    /// Store an object and return its ID
    async fn put<T: Serialize + Send + 'static>(
        &self, 
        object: T
    ) -> Result<ObjectId>;
    
    /// Future API: Create a buffer for direct writing
    async fn create(&self, size: usize) -> Result<ObjectBuilder>;
    
    /// Retrieve an object by ID
    async fn get<T: DeserializeOwned + 'static>(
        &self, 
        id: ObjectId
    ) -> Result<Option<T>>;
    
    /// Get raw bytes without deserialization (zero-copy)
    async fn get_raw(&self, id: ObjectId) -> Result<Option<Bytes>>;
    
    /// Check if object exists
    async fn exists(&self, id: ObjectId) -> Result<bool>;
    
    /// Get object metadata
    async fn metadata(&self, id: ObjectId) -> Result<Option<ObjectMetadata>>;
    
    /// Pin object in memory (prevent eviction)
    async fn pin(&self, id: ObjectId) -> Result<()>;
    
    /// Unpin object
    async fn unpin(&self, id: ObjectId) -> Result<()>;
    
    /// Get comprehensive store statistics
    async fn stats(&self) -> StoreStats;
}

/// Future support for Create/Seal pattern
pub struct ObjectBuilder {
    id: ObjectId,
    buffer: BytesMut,
    store: Weak<InMemoryStore>,
}

impl ObjectBuilder {
    pub fn write(&mut self, data: &[u8]) -> Result<()> { ... }
    pub async fn seal(self) -> Result<ObjectId> { ... }
}
```

### InMemoryStore Implementation (Updated)

```rust
pub struct InMemoryStore {
    /// Main cache using moka for automatic eviction
    cache: moka::future::Cache<ObjectId, StoredObject>,
    
    /// Pinned objects that cannot be evicted
    pinned: Arc<DashMap<ObjectId, StoredObject>>,
    
    /// Pending objects being created (for future Create/Seal)
    pending: Arc<DashMap<ObjectId, PendingObject>>,
    
    /// Store statistics
    stats: Arc<StoreStatsCollector>,
    
    /// Configuration
    config: StoreConfig,
}

struct StoredObject {
    data: Bytes,
    metadata: Arc<ObjectMetadata>,
}

struct StoreStatsCollector {
    hit_count: AtomicU64,
    miss_count: AtomicU64,
    put_count: AtomicU64,
    get_count: AtomicU64,
    // Use sharded counters to reduce contention
    size_shards: Vec<AtomicUsize>,
}

impl InMemoryStore {
    pub fn new(config: StoreConfig) -> Self {
        // Configure moka cache
        let cache = moka::future::Cache::builder()
            .max_capacity(config.max_memory)
            .weigher(|_key, value: &StoredObject| value.data.len() as u32)
            .eviction_listener(|key, value, cause| {
                // Update eviction stats
                match cause {
                    RemovalCause::Size => { /* log size eviction */ }
                    RemovalCause::Expired => { /* log expiration */ }
                    _ => {}
                }
            })
            .build();
            
        Self {
            cache,
            pinned: Arc::new(DashMap::new()),
            pending: Arc::new(DashMap::new()),
            stats: Arc::new(StoreStatsCollector::new()),
            config,
        }
    }
}
```

## Integration Plan

### 1. Update ObjectRef<T>

```rust
pub struct ObjectRef<T> {
    id: ObjectId,
    store: Arc<dyn ObjectStore>,
    cached: OnceCell<T>, // Optional: cache deserialized value
    _phantom: PhantomData<T>,
}

impl<T: DeserializeOwned> ObjectRef<T> {
    pub async fn get(&self) -> Result<T> {
        // Check cached value first
        if let Some(cached) = self.cached.get() {
            return Ok(cached.clone());
        }
        
        // Load from store
        let value = self.store.get(self.id).await?
            .ok_or_else(|| RustyRayError::Internal("Object not found"))?;
            
        // Optionally cache for repeated access
        let _ = self.cached.set(value.clone());
        Ok(value)
    }
    
    /// Get raw bytes for zero-copy scenarios
    pub async fn get_raw(&self) -> Result<Bytes> {
        self.store.get_raw(self.id).await?
            .ok_or_else(|| RustyRayError::Internal("Object not found"))
    }
}
```

### 2. Memory Pressure Testing Strategy *(New)*

Create specific test scenarios:
- Concurrent puts that exceed memory limit
- Mixed pinned/unpinned objects under pressure
- Rapid put/get cycles to test eviction
- Large object handling (GB-scale)

## Implementation Phases (Updated)

### Phase 4.1: Core Implementation (Week 1)
- [x] ObjectStore trait definition
- [ ] InMemoryStore using moka cache
- [ ] Basic put/get operations
- [ ] Reference counting via moka

### Phase 4.2: Integration (Week 2)
- [ ] Update ObjectRef to use store
- [ ] Integrate with TaskSystem
- [ ] Update existing tests
- [ ] Add object store tests

### Phase 4.3: Advanced Features (Week 3)
- [ ] Pinning support
- [ ] Comprehensive statistics
- [ ] Memory pressure tests
- [ ] Performance benchmarks

### Phase 4.4: Prepare for Future (Week 4)
- [ ] Internal Create/Seal API design
- [ ] Documentation
- [ ] Examples
- [ ] Design shared memory migration path

## Key Improvements from Feedback

1. **Use Proven Cache Implementation**: Moka provides battle-tested caching with minimal effort
2. **Prepare for Create/Seal**: Internal API supports future zero-copy writes
3. **Enhanced Statistics**: Comprehensive metrics for debugging and optimization
4. **Contention Mitigation**: Sharded size counters and moka's internal sharding
5. **Memory Pressure Testing**: Explicit test scenarios for eviction behavior

## Migration Path to Shared Memory

When moving to shared memory in the future:
1. Implement `SharedMemoryStore` using memmap2
2. Use same `ObjectStore` trait interface
3. Create/Seal pattern becomes primary API
4. Maintain `InMemoryStore` for single-process scenarios

## Success Metrics (Updated)

1. **Performance**:
   - Put latency: <1ms for objects <1MB
   - Get latency: <100μs for cached objects
   - Memory overhead: <10% vs raw storage
   - **No deadlocks under memory pressure** *(new)*

2. **Reliability**:
   - No memory leaks under stress
   - Correct eviction behavior
   - Type safety maintained
   - **Graceful degradation at memory limit** *(new)*

3. **Usability**:
   - Seamless integration with existing API
   - Clear error messages
   - Comprehensive statistics for debugging
   - Good documentation

## Dependencies

```toml
[dependencies]
moka = { version = "0.12", features = ["future"] }
bytes = "1.5"
dashmap = "6.0"
bincode = "2.0"
tokio = { version = "1", features = ["full"] }
once_cell = "1.19"
```

## Conclusion

This updated design incorporates valuable feedback to create a more robust and production-ready object store. By leveraging proven libraries like `moka` and designing for future enhancements, we maintain momentum while building on solid foundations. The focus on testing memory pressure scenarios and comprehensive statistics will ensure reliability as we scale.