# RustyRay Continuation Prompt - Phase 4.5 Progress

## Project Context
I'm working on RustyRay, a Rust implementation of Ray Core (the distributed actor system). This is a greenfield project with no existing users, giving us freedom to make optimal design choices.

## Where We Started (Beginning of Phase 4.5)
- **Completed Phase 4**: Production-ready object store with CLRU cache, type-safe operations, and zero-copy access
- **Goal**: Integrate the object store with actors and tasks for efficient data sharing
- **Initial Problem**: ObjectRef was in the task module (implying task-specific), but Ray's architecture shows it should be universal

## What We've Accomplished

### 1. Architectural Refactoring
- **Moved ObjectRef from `task/object_ref.rs` to core level** (`src/object_ref.rs`)
- **Updated all imports** across the codebase
- **Simplified ObjectRef** to always be store-backed (removed dual channel/store modes)

### 2. Consulted Ray's Architecture (via Gemini)
Key findings:
- Ray uses `RayObject` with both data and metadata buffers for type preservation
- ObjectRef is universal, not task-specific
- Actors use same `ray.put()`/`ray.get()` as tasks (no special API)
- Task results are automatically stored in object store

### 3. Fixed Type Storage Issue
- **Problem**: Task results stored as `Vec<u8>` type, causing type mismatches on retrieval
- **Solution**: Implemented type-erased storage - store raw bytes, deserialize on retrieval
- **Result**: Tests went from 16 failures to 4 (56 passing)

### 4. Updated Runtime Architecture
- Added shared `object_store` to Runtime
- TaskSystem and ActorSystem share the same store instance
- Task results automatically stored via `put_with_id`

## Current State

### Code Structure
```
crates/rustyray-core/src/
├── object_ref.rs       # Universal object references (moved from task/)
├── object_store/       # Storage backend
├── actor/              # Actor system
├── task/               # Task system  
└── runtime.rs          # Contains shared object store
```

### Test Status
- 56 tests passing
- 4 tests failing (pre-existing issues):
  - `test_task_error_propagation` 
  - `test_error_with_multiple_args`
  - `test_long_running_task_cancellation`
  - `test_task_timeout`

These failures are related to error propagation and cancellation, not our refactoring.

## Outstanding Tasks (from TodoWrite)

1. ✅ Move ObjectRef from task module to core level
2. ✅ Update all imports after moving ObjectRef
3. ✅ Simplify ObjectRef to be always store-backed
4. ✅ Fix task result storage type mismatch issue
5. **[PENDING]** Create ray module with global put/get functions
6. ✅ Update task system to auto-store results
7. **[PENDING]** Update examples to use new API
8. **[PENDING]** Add actor examples using ray::put/get
9. **[PENDING]** Create integration tests for actor-task sharing
10. **[PENDING]** Document the new architecture

## Next Steps Plan

### 1. Create `ray` Module (Priority: High)
```rust
// In crates/rustyray-core/src/ray.rs
pub fn put<T: Send + Sync + 'static>(value: T) -> Result<ObjectRef<T>> {
    runtime::global()?.object_store().put(value)
}

pub async fn get<T: Send + Sync + 'static>(obj_ref: &ObjectRef<T>) -> Result<T> {
    obj_ref.get().await
}
```

### 2. Actor Integration Examples (Priority: Medium)
- Create examples showing actors using `ray::put()`/`ray::get()`
- No need for special ActorContext - actors use global functions
- Show actor-task data sharing patterns

### 3. Fix Pre-existing Test Failures (Priority: Low)
- Error propagation tests expect errors to be stored/retrieved
- Cancellation tests have timing issues
- These are separate from our refactoring work

## Key Design Decisions Made

1. **ObjectRef is Universal**: Not task-specific, used by all components
2. **Type-Erased Storage**: Store raw bytes, deserialize on retrieval (like Python Ray)
3. **Shared Object Store**: Single instance in Runtime, shared by all systems
4. **No Actor-Specific API**: Actors use same `put`/`get` as everyone else

## Important Files Changed

- `/crates/rustyray-core/src/object_ref.rs` - Moved and simplified
- `/crates/rustyray-core/src/runtime.rs` - Added shared object store
- `/crates/rustyray-core/src/task/system.rs` - Auto-stores task results
- `/crates/rustyray-core/src/lib.rs` - Updated exports
- `/crates/rustyray-core/src/task/mod.rs` - Removed ObjectRef export

## Commands to Continue

```bash
# Check current test status
cargo test --lib

# Run specific failing tests to see current behavior
cargo test --lib test_error_propagation

# Continue with next task (create ray module)
# See design in docs/phase4.5-design-v2.md
```

## Architecture Notes

The current implementation uses "Option 3" from our type fix plan:
- Store task results as raw bytes via `put_with_id`
- ObjectRef::get() deserializes from raw bytes
- Type checking happens at retrieval (like Python duck typing)

This is simpler than Ray's full metadata approach but works well for our current needs.