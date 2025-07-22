# ObjectRef Serialization Fix - Implementation Plan

## Problem Summary
ObjectRef currently contains `Arc<InMemoryStore>` which prevents it from being serialized when passed as task arguments. This breaks the fundamental Ray pattern of passing ObjectRefs between tasks.

## Proposed Solution: ID-Only ObjectRef

### Current Structure
```rust
pub struct ObjectRef<T> {
    id: ObjectId,
    store: Arc<InMemoryStore>,  // Cannot serialize!
    _phantom: PhantomData<T>,
}
```

### New Structure
```rust
pub struct ObjectRef<T> {
    id: ObjectId,
    _phantom: PhantomData<T>,
}
```

## Implementation Steps

### Phase 1: Core Changes
1. **Modify ObjectRef struct** (`object_ref.rs`)
   - Remove `store: Arc<InMemoryStore>` field
   - Keep only `id` and `_phantom` fields
   - Derive `Serialize` and `Deserialize`

2. **Update ObjectRef::new()**
   - Change signature from `new(id: ObjectId, store: Arc<InMemoryStore>)` to `new(id: ObjectId)`
   - Remove store parameter

3. **Update ObjectRef::get()**
   - Replace `self.store.get_raw()` with `runtime::global()?.object_store().get_raw()`
   - Handle runtime lookup errors gracefully

### Phase 2: Update Call Sites
1. **TaskSystem** (`task/system.rs`)
   - Update `submit()` to use `ObjectRef::new(object_id)` (no store)
   - Update `put()` to use new constructor

2. **Ray module** (`ray.rs`)
   - Update `put()` to use `ObjectRef::new(result.id)` 
   - Update `put_batch()` similarly

3. **Tests and Examples**
   - Remove store parameter from all ObjectRef::new() calls
   - No other changes needed (API remains the same)

### Phase 3: Optimization (Optional)
1. **Cache store reference** 
   - Use thread_local! for frequently accessed store
   - Benchmark to see if worthwhile

## Benefits
1. **Serializable**: ObjectRef can be passed as task arguments
2. **Simpler**: No need to pass store references around
3. **Ray-like**: Matches Ray's design where ObjectRef is just an ID
4. **Backwards compatible**: Public API (get/put) remains unchanged

## Risks & Mitigations
1. **Risk**: Small performance overhead from runtime lookup
   - **Mitigation**: Thread-local caching if needed
   
2. **Risk**: Runtime not initialized when calling get()
   - **Mitigation**: Clear error message guiding user to initialize runtime

## Testing Plan
1. Existing tests should continue to pass
2. Add serialization test for ObjectRef
3. Test passing ObjectRef as task argument
4. Test error handling when runtime not initialized

## Questions for Ray Architecture Review
1. How does Ray handle ObjectRef serialization internally?
2. Does Ray use any caching for object store lookups?
3. Are there other approaches we should consider?