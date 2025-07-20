# Phase 4: Object Store Design

## Overview

The RustyRay Object Store is a critical component that provides efficient in-memory storage and sharing of serialized objects between tasks and actors. It's inspired by Ray's Plasma store but adapted for Rust's ownership model and our incremental development approach.

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
    pub ref_count: Arc<AtomicUsize>,
    pub object_type: TypeId,
}

/// Result of put operation
pub struct PutResult {
    pub id: ObjectId,
    pub size: usize,
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

**Alternative considered**: Raw `Vec<u8>`
- **Rejected because**: No zero-copy sharing, higher memory usage

### 2. Serialization Format

**Decision**: Continue using bincode
- **Rationale**:
  - Fast and compact
  - Already integrated
  - Good Rust support
  
**Future consideration**: Support multiple formats (Arrow, Protobuf) for Ray compatibility

### 3. Memory Management

**Decision**: Reference counting with automatic cleanup
- **Rationale**:
  - Rust-idiomatic
  - Prevents memory leaks
  - No need for manual delete calls

**Implementation**:
- Track references via `Arc<AtomicUsize>`
- Clean up when ref_count reaches 0
- LRU eviction when memory limit reached

### 4. Type Safety

**Decision**: Store TypeId with objects, verify on get
- **Rationale**:
  - Catch type mismatches at runtime
  - Better than dynamic casting
  - Enables future type registry

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
    
    /// Get store statistics
    async fn stats(&self) -> StoreStats;
}
```

### InMemoryStore Implementation

```rust
pub struct InMemoryStore {
    /// Object storage: ObjectId -> (Bytes, Metadata)
    objects: Arc<DashMap<ObjectId, (Bytes, ObjectMetadata)>>,
    
    /// Pinned objects (won't be evicted)
    pinned: Arc<DashMap<ObjectId, ()>>,
    
    /// Memory usage tracking
    total_size: Arc<AtomicUsize>,
    
    /// Configuration
    config: StoreConfig,
}

pub struct StoreConfig {
    /// Maximum memory usage in bytes
    pub max_memory: usize,
    
    /// Eviction policy
    pub eviction_policy: EvictionPolicy,
}

pub enum EvictionPolicy {
    /// Least Recently Used
    LRU,
    /// First In First Out
    FIFO,
    /// Least Frequently Used
    LFU,
}
```

## Integration Plan

### 1. Update ObjectRef<T>

Current ObjectRef uses channels. We'll enhance it to:
- Store ObjectId reference
- Lazy load from object store
- Support zero-copy access via get_raw()

```rust
pub struct ObjectRef<T> {
    id: ObjectId,
    store: Arc<dyn ObjectStore>,
    _phantom: PhantomData<T>,
}

impl<T: DeserializeOwned> ObjectRef<T> {
    pub async fn get(&self) -> Result<T> {
        self.store.get(self.id).await?
            .ok_or_else(|| RustyRayError::Internal("Object not found"))
    }
}
```

### 2. TaskSystem Integration

Update TaskSystem::put to use object store:

```rust
impl TaskSystem {
    pub async fn put<T: Serialize + Send + 'static>(
        &self, 
        value: T
    ) -> Result<ObjectRef<T>> {
        let id = self.object_store.put(value).await?;
        Ok(ObjectRef::new(id, self.object_store.clone()))
    }
}
```

### 3. Actor Integration

Actors can store large state in object store:
- Reduces message passing overhead
- Enables shared state patterns
- Supports actor checkpointing

## Implementation Phases

### Phase 4.1: Core Implementation (Week 1)
- [ ] ObjectStore trait definition
- [ ] InMemoryStore with HashMap backend
- [ ] Basic put/get operations
- [ ] Reference counting

### Phase 4.2: Integration (Week 2)
- [ ] Update ObjectRef to use store
- [ ] Integrate with TaskSystem
- [ ] Update existing tests
- [ ] Add object store tests

### Phase 4.3: Memory Management (Week 3)
- [ ] LRU eviction policy
- [ ] Memory limit enforcement
- [ ] Pinning support
- [ ] Statistics and monitoring

### Phase 4.4: Performance & Polish (Week 4)
- [ ] Performance benchmarks
- [ ] Optimize serialization
- [ ] Documentation
- [ ] Examples

## Future Enhancements (Post-Phase 4)

### Shared Memory Support
- Use memmap2 for inter-process sharing
- Compatible with current API
- Enables multi-process actors

### Spilling to Disk
- Automatic spilling when memory full
- Transparent to users
- Configurable thresholds

### Distributed Object Store (Phase 5)
- Cross-node object sharing
- Consistent hashing for placement
- Replication for fault tolerance

## Testing Strategy

### Unit Tests
- Object lifecycle (put, get, evict)
- Reference counting correctness
- Type safety verification
- Memory limit enforcement

### Integration Tests
- Task→Object→Task flow
- Actor state persistence
- Concurrent access patterns
- Memory pressure scenarios

### Benchmarks
- Put/get latency
- Throughput under load
- Memory overhead
- Serialization performance

## Open Questions

1. **Object Versioning**: Should we support multiple versions of same object?
   - **Recommendation**: No, keep it simple for now

2. **Compression**: Should we compress large objects?
   - **Recommendation**: Add as optional feature later

3. **Custom Serializers**: Support formats beyond bincode?
   - **Recommendation**: Add trait for custom serializers in future

4. **Distributed Coherence**: How to handle in Phase 5?
   - **Recommendation**: Design for local-first, add coherence protocol later

## Success Metrics

1. **Performance**:
   - Put latency: <1ms for objects <1MB
   - Get latency: <100μs for cached objects
   - Memory overhead: <10% vs raw storage

2. **Reliability**:
   - No memory leaks under stress
   - Correct reference counting
   - Type safety maintained

3. **Usability**:
   - Seamless integration with existing API
   - Clear error messages
   - Good documentation

## Conclusion

This object store design provides a solid foundation for efficient data sharing in RustyRay while maintaining Rust's safety guarantees. The phased implementation approach allows us to deliver value incrementally while keeping the door open for future enhancements like shared memory and distributed storage.