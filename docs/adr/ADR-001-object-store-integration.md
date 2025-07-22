# ADR-001: Phase 4.5 Object Store Integration Decisions

**Date:** February 2025  
**Status:** Accepted  
**Context:** Integrating the object store with actors and tasks

## Decision Record

### 1. Universal ObjectRef at Core Level

**Decision:** Move ObjectRef from `task/object_ref.rs` to `src/object_ref.rs`

**Rationale:**
- Ray's architecture shows ObjectRef is universal, not task-specific
- Actors and tasks both need to share data via ObjectRef
- Simplifies mental model - one type for all object references

**Consequences:**
- Breaking change to module structure
- All imports need updating
- Cleaner architecture going forward

### 2. Always Store-Backed ObjectRef

**Decision:** Remove dual channel/store mode, ObjectRef always uses object store

**Previous Design:**
```rust
enum ObjectRefInner<T> {
    Channel(Receiver<Result<T>>),  // For local task results
    Store(ObjectId, Arc<Store>),    // For stored objects
}
```

**New Design:**
```rust
pub struct ObjectRef<T> {
    id: ObjectId,
    store: Arc<InMemoryStore>,
    _phantom: PhantomData<T>,
}
```

**Rationale:**
- Eliminates complexity of dual modes
- Consistent behavior for all ObjectRefs
- Enables seamless actor-task sharing
- Matches Ray's unified approach

**Consequences:**
- Slight overhead for small task results
- Simpler implementation and testing
- Unified data sharing model

### 3. Type-Erased Storage

**Decision:** Store raw bytes in object store, deserialize on retrieval

**Implementation:**
```rust
// Storage: Always Vec<u8>
store.put_with_id(id, serialize(&value)?);

// Retrieval: Deserialize to requested type
let bytes = store.get_raw(id)?;
deserialize::<T>(&bytes)
```

**Rationale:**
- Avoids complex type registry
- Matches Python Ray's duck typing
- Enables cross-language compatibility later
- Simple and efficient

**Consequences:**
- Runtime type checking only
- Type errors caught at retrieval time
- Need clear error messages for type mismatches

### 4. Error Storage in Object Store

**Decision:** Store errors as objects with special marker prefix

**Implementation:**
```rust
const ERROR_MARKER: &[u8] = b"__RUSTYRAY_ERROR__";

// Store error
let error_bytes = [ERROR_MARKER, error_msg.as_bytes()].concat();
store.put_with_id(id, error_bytes);

// Detect error on retrieval
if bytes.starts_with(ERROR_MARKER) {
    return Err(parse_error(&bytes));
}
```

**Rationale:**
- Unifies error and success result handling
- Errors propagate through ObjectRef naturally
- No separate error channel needed
- Simple marker-based approach

**Consequences:**
- Errors take up object store space
- Need to be careful about error object lifecycle
- Simple but effective for current needs

### 5. Shared Object Store in Runtime

**Decision:** Single object store instance shared by all components

**Architecture:**
```rust
pub struct Runtime {
    actor_system: Arc<ActorSystem>,
    task_system: Arc<TaskSystem>,
    object_store: Arc<InMemoryStore>,  // Shared
}
```

**Rationale:**
- Single source of truth for all objects
- Enables actor-task data sharing
- Simplifies configuration and management
- Natural pathway to distributed store

**Consequences:**
- All components share same memory pool
- Need careful capacity planning
- Simplified architecture

### 6. Ray-Compatible API

**Decision:** Implement `ray::put()` and `ray::get()` global functions

**API Design:**
```rust
pub async fn put<T>(value: T) -> Result<ObjectRef<T>>
pub async fn get<T>(obj_ref: &ObjectRef<T>) -> Result<T>
```

**Rationale:**
- Familiar to Ray users
- Simple and intuitive
- Hides complexity of runtime access
- Enables future distributed implementation

**Consequences:**
- Requires global runtime state
- Need careful initialization ordering
- Clean API for users

## Rejected Alternatives

### 1. Task-Specific ObjectRef
- Would have required separate ActorObjectRef type
- Increased complexity with no benefit
- Goes against Ray's unified model

### 2. Typed Object Store
- Store with generic type parameter Store<T>
- Would prevent heterogeneous storage
- Incompatible with cross-language goals

### 3. Separate Error Channel
- Considered separate error propagation mechanism
- Added complexity
- Errors are just another type of result

### 4. Multiple Object Stores
- Considered per-actor or per-task stores
- Prevents data sharing
- Complex lifetime management

## Lessons Learned

1. **Unification Simplifies**: Moving to universal concepts reduces complexity
2. **Type Erasure Works**: Runtime typing is sufficient for distributed systems
3. **Errors Are Data**: Treating errors as stored objects unifies the model
4. **Global State Is OK**: For runtime infrastructure, global state simplifies API

## References

- [Ray's ObjectRef Implementation](https://github.com/ray-project/ray/blob/master/src/ray/core_worker/common.h)
- [Phase 4.5 Design Document](./phase4.5-design-v2.md)
- [Gemini Analysis of Ray Architecture](./gemini-phase4.5-review.md)