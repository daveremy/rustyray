# Continuation Prompt: RustyRay Phase 4.5 - Runtime Architecture Fix

## Context

You are working on RustyRay, a Rust implementation of Ray Core (distributed actor system). Phase 4.5 fixed ObjectRef serialization but revealed fundamental issues with concurrent test execution.

## Current Status

### What Was Completed
1. **ObjectRef Serialization** - Fixed with optional store reference and context-aware deserialization
2. **Automatic ObjectRef Resolution** - Enhanced #[remote] macro for ObjectRef parameters
3. **Initial Test Fixes** - Made function names unique, added task tracking for spawned tasks

### Current Problem
Tests pass with `--test-threads=1` but fail when run concurrently due to shared global runtime. After extensive analysis, we've decided to adopt Ray's global runtime pattern with proper lifecycle management.

## Architecture Decision

We're moving from:
```rust
// Current: OnceCell that can't be reinitialized
static RUNTIME: OnceCell<Runtime> = OnceCell::new();
```

To:
```rust
// New: Explicit lifecycle management like Ray
static RUNTIME: RwLock<Option<Arc<Runtime>>> = RwLock::new(None);
```

**Key Decision Document**: `/docs/architecture/runtime-design-decision.md`

## Implementation Plan

### Phase 1: Core Runtime Changes
1. Replace `OnceCell<Runtime>` with `RwLock<Option<Arc<Runtime>>>`
2. Implement `init()`, `shutdown()`, `global()`, `is_initialized()`
3. Make runtime re-initializable after shutdown
4. Update all runtime access to handle `Option`

### Phase 2: Test Infrastructure
1. Create `with_test_runtime()` fixture in test_utils.rs
2. Ensure cleanup even on test panic
3. Add test-specific configuration helpers

### Phase 3: Migrate Tests
1. Update all tests to use the fixture instead of global runtime
2. Remove `init_test_runtime()` and `clear_runtime_state()`
3. Ensure each test has proper isolation

### Phase 4: Verification
1. Run tests with default concurrency (no --test-threads=1)
2. Verify all tests pass consistently
3. Update documentation and examples

## Key Files to Modify

1. `/crates/rustyray-core/src/runtime.rs` - Core runtime implementation
2. `/crates/rustyray-core/src/test_utils.rs` - Test fixtures
3. `/crates/rustyray-core/src/task/system.rs` - Remove task tracking added earlier
4. All test files using `runtime::global()`

## Important Context

- Ray uses this exact pattern successfully - we're following proven architecture
- The global runtime is NOT an anti-pattern for distributed systems
- Proper lifecycle management is the key to making it work
- Test isolation comes from fixtures, not from avoiding global state

## Next Steps

Start with Phase 1: Modify the runtime.rs file to implement the new architecture. The key is making the runtime re-initializable while maintaining thread safety.

## Success Criteria

1. All tests pass without `--test-threads=1`
2. Tests can run concurrently without interference
3. API remains compatible with Ray's patterns
4. No regression in functionality

## Notes

- We added task tracking to TaskSystem earlier - this can be removed as it was a band-aid
- The `replace_task_system` method in Runtime can also be removed
- Focus on making the lifecycle explicit and safe

Good luck! This architectural change will make RustyRay much more robust and maintainable.