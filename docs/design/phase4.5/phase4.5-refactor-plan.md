# Phase 4.5 Refactoring Plan: ObjectRef Architecture

## Context

After analyzing Ray's architecture with Gemini, we discovered that our current design has ObjectRef in the wrong location (task module) and uses unnecessary complexity (channels). Ray's design is cleaner: ObjectRef is a universal primitive used by all components.

## Key Learnings from Ray

1. **ObjectRef is Universal**: Not task-specific - it's the standard way to reference any remote object
2. **No Special Actor API**: Actors use the same `ray.put()` and `ray.get()` as everyone else
3. **Automatic Task Storage**: Task results are automatically stored in the object store
4. **One Store per Node**: Single shared object store instance (in our case, per Runtime)
5. **Simple API**: Primary interface is global `put()`/`get()` functions

## Current Problems

1. ObjectRef is in `task/object_ref.rs` implying it's task-specific
2. Complex dual-mode ObjectRef (channels vs store)
3. Inconsistent APIs between actors and tasks
4. Compilation errors from our initial integration attempt

## Refactoring Plan

### Phase 1: Restructure Module Hierarchy (30 min)

**Move ObjectRef to core level:**
```
FROM: crates/rustyray-core/src/task/object_ref.rs
TO:   crates/rustyray-core/src/object_ref.rs
```

**Update module structure:**
```rust
// In lib.rs
pub mod object_ref;     // Universal object references
pub mod object_store;   // Storage backend
pub mod actor;          // Actor system
pub mod task;           // Task system
```

### Phase 2: Simplify ObjectRef Implementation (30 min)

1. **Remove channel-based mode** - All ObjectRefs are store-backed
2. **Simplify API** - Just `new()`, `get()`, and `id()`
3. **Better waiting** - Use exponential backoff for objects not yet available
4. **Clean serialization** - Handle store context properly

### Phase 3: Create Global API (30 min)

**Add `ray` module with global functions:**
```rust
// In crates/rustyray-core/src/ray.rs
pub fn put<T: Send + Sync + 'static>(value: T) -> Result<ObjectRef<T>> {
    runtime::global()?.put(value)
}

pub async fn get<T: Send + Sync + 'static>(obj_ref: &ObjectRef<T>) -> Result<T> {
    obj_ref.get().await
}
```

### Phase 4: Update Integration Points (30 min)

1. **Runtime**: Already has shared object store ✓
2. **Tasks**: Automatically store results in object store
3. **Actors**: Use global `ray::put()`/`ray::get()` - no special context
4. **Examples**: Update to show new patterns

## Success Criteria

1. All tests pass after refactoring
2. ObjectRef is in the correct location
3. Simpler, cleaner API matching Ray's design
4. No compilation errors
5. Clear separation of concerns

## Benefits

- **Correct Architecture**: Matches Ray's battle-tested design
- **Simplicity**: Fewer moving parts, easier to understand
- **Future-Proof**: Ready for distributed implementation
- **Clean API**: Intuitive for users familiar with Ray

## Risks & Mitigation

- **Risk**: Breaking existing functionality
  - **Mitigation**: Run tests after each phase
  
- **Risk**: Missing some imports
  - **Mitigation**: Use compiler errors as guide
  
- **Risk**: Over-engineering
  - **Mitigation**: Stay close to Ray's design

## Timeline

Total estimated time: 2 hours
- Phase 1: 30 minutes
- Phase 2: 30 minutes  
- Phase 3: 30 minutes
- Phase 4: 30 minutes

## Next Steps After Refactoring

Continue with Phase 4.5 integration:
1. Automatic task result storage
2. Actor examples using put/get
3. Comprehensive integration tests
4. Performance benchmarks