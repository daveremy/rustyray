# RustyRay Continuation Prompt - Phase 4.5 ObjectRef Serialization Fix

## Project Context
I'm working on RustyRay, a Rust implementation of Ray Core (the distributed actor system). We're in Phase 4.5, integrating the object store with actors and tasks.

## Current Status
- ✅ Object store is production-ready with CLRU cache
- ✅ ObjectRef moved to core level and simplified
- ✅ Ray module created with global put/get functions
- ✅ Task results automatically stored in object store
- ✅ All 64 library tests pass
- ❌ **CRITICAL ISSUE**: ObjectRef cannot be serialized when passed as task arguments

## The Problem
ObjectRef contains `Arc<InMemoryStore>` which prevents serialization. This breaks the fundamental Ray pattern:
```rust
// This should work but doesn't:
let data_ref = create_data_remote::remote().await?;
let result_ref = process_data_remote::remote(data_ref).await?; // FAILS!
// Error: "ObjectRef deserialization requires runtime context"
```

## The Solution (Approved)
Implement Phase 1 of the plan in `docs/objectref-serialization-plan-v2.md`:

1. **Make ObjectRef ID-only**:
```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObjectRef<T> {
    id: ObjectId,
    #[serde(skip)]
    _phantom: PhantomData<T>,
}
```

2. **Resolve store at runtime in get()**:
```rust
pub async fn get(&self) -> Result<T> {
    let runtime = runtime::global()?;
    let store = runtime.object_store();
    // ... rest of implementation
}
```

## Implementation Checklist
- [ ] Modify ObjectRef struct in `/crates/rustyray-core/src/object_ref.rs`
- [ ] Update ObjectRef::new() to only take ObjectId
- [ ] Update ObjectRef::get() to use runtime::global()
- [ ] Add RuntimeNotInitialized error variant
- [ ] Update all ObjectRef construction sites:
  - [ ] TaskSystem (`task/system.rs`)
  - [ ] Ray module (`ray.rs`)
  - [ ] Tests that create ObjectRef directly
- [ ] Add serialization test for ObjectRef
- [ ] Fix failing examples:
  - [ ] `ray_api_demo`
  - [ ] `objectref_arg_test`
  - [ ] Integration tests
- [ ] Verify all tests pass
- [ ] Test complex task chaining works

## Key Files to Modify
1. `/crates/rustyray-core/src/object_ref.rs` - Core changes
2. `/crates/rustyray-core/src/task/system.rs` - Update ObjectRef construction
3. `/crates/rustyray-core/src/ray.rs` - Update ObjectRef construction
4. `/crates/rustyray-core/src/error.rs` - Add new error variant

## Testing Commands
```bash
# Run all tests
cargo test --lib

# Test specific examples
cargo run --example objectref_arg_test
cargo run --example ray_api_demo

# Run integration tests
cargo test --test integration_test
```

## Success Criteria
- ObjectRef can be serialized/deserialized
- ObjectRef can be passed as task arguments
- All tests pass
- Examples demonstrating task chaining work correctly

## Context Documents
- `docs/objectref-serialization-plan-v2.md` - The implementation plan
- `docs/phase4.5-objectref-serialization-issue.md` - Problem analysis
- `docs/gemini-query-task-results.md` - Ray architecture insights

## Next Steps After This Fix
Once ObjectRef serialization is fixed, Phase 4.5 will be complete. The next phase will focus on:
1. Creating comprehensive documentation
2. Performance benchmarking
3. Planning Phase 5 (distributed features)