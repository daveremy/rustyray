# ObjectRef Serialization Fix - Implementation Plan V2

## Problem Summary
ObjectRef currently contains `Arc<InMemoryStore>` which prevents serialization when passed as task arguments, breaking the fundamental Ray pattern of task chaining with data dependencies.

## Gemini's Insights on Ray's Architecture

Ray's ObjectRef is **not just an ID** - it contains:
- Object ID (unique identifier)
- Owner Address (worker that created it - for distributed GC)
- Task ID (creating task - for lineage/fault tolerance)
- Call Site (debug info)

However, Ray **doesn't use this metadata for object location**. Instead:
- Workers check local store first
- Query Global Control Store (GCS) if needed
- GCS maintains object ID → node location mapping

## Revised Solution: Phased Approach

### Phase 1: Minimal ID-Only ObjectRef (Current Priority)
Make ObjectRef serializable by storing only the ID, resolving store at runtime.

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObjectRef<T> {
    id: ObjectId,
    #[serde(skip)]
    _phantom: PhantomData<T>,
}
```

### Phase 2: Future Expansion (When Needed)
Add metadata for distributed features:

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObjectRef<T> {
    id: ObjectId,
    #[serde(skip_serializing_if = "Option::is_none")]
    owner_worker_id: Option<WorkerId>,  // For distributed GC
    #[serde(skip_serializing_if = "Option::is_none")]
    creating_task_id: Option<TaskId>,   // For fault tolerance
    #[serde(skip)]
    _phantom: PhantomData<T>,
}
```

## Implementation Steps (Phase 1)

### 1. Core ObjectRef Changes (`object_ref.rs`)
```rust
// Remove store field, add Serialize/Deserialize
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObjectRef<T> {
    id: ObjectId,
    #[serde(skip)]
    _phantom: PhantomData<T>,
}

impl<T> ObjectRef<T> {
    // Simplified constructor
    pub fn new(id: ObjectId) -> Self {
        ObjectRef {
            id,
            _phantom: PhantomData,
        }
    }
}

impl<T: DeserializeOwned + Send + 'static> ObjectRef<T> {
    pub async fn get(&self) -> Result<T> {
        // Resolve store at runtime
        let runtime = runtime::global()
            .map_err(|_| RustyRayError::RuntimeNotInitialized)?;
        let store = runtime.object_store();
        
        // Rest of get() implementation stays the same
        // (polling with exponential backoff)
    }
}
```

### 2. Update All Construction Sites

**TaskSystem** (`task/system.rs`):
```rust
// Before: ObjectRef::new(object_id, self.object_store.clone())
// After:  ObjectRef::new(object_id)
```

**Ray module** (`ray.rs`):
```rust
// Before: Ok(ObjectRef::new(result.id, object_store.clone()))
// After:  Ok(ObjectRef::new(result.id))
```

### 3. Add Runtime Error Handling
Create a new error variant for better user experience:
```rust
pub enum RustyRayError {
    // ... existing variants ...
    RuntimeNotInitialized,
}
```

### 4. Update Tests
Most tests should work unchanged. Add specific serialization test:
```rust
#[test]
fn test_objectref_serialization() {
    let obj_ref = ObjectRef::<i32>::new(ObjectId::new());
    let serialized = serde_json::to_string(&obj_ref).unwrap();
    let deserialized: ObjectRef<i32> = serde_json::from_str(&serialized).unwrap();
    assert_eq!(obj_ref.id(), deserialized.id());
}
```

## Benefits
1. **Immediate**: Fixes ObjectRef serialization for task arguments
2. **Simple**: Minimal changes to existing code
3. **Forward-compatible**: Can add metadata fields later without breaking changes
4. **Ray-aligned**: Follows Ray's pattern of runtime resolution

## Performance Considerations
- **Overhead**: One runtime lookup per `get()` call
- **Mitigation**: Could add thread-local caching if benchmarks show need
- **Reality**: Negligible compared to actual object retrieval time

## Migration Path
1. All existing code continues to work (API unchanged)
2. ObjectRef becomes serializable immediately
3. Future distributed features can add metadata fields
4. No breaking changes required

## Testing Strategy
1. **Unit tests**: ObjectRef serialization/deserialization
2. **Integration tests**: Pass ObjectRef between tasks
3. **Examples**: Update failing examples to verify fix
4. **Benchmarks**: Measure overhead of runtime lookup

## Success Criteria
- [ ] ObjectRef can be serialized/deserialized
- [ ] All existing tests pass
- [ ] ObjectRef can be passed as task argument
- [ ] Complex task chaining examples work
- [ ] Performance impact < 1% on benchmarks