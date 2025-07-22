# RustyRay Runtime Architecture Decision

**Date**: January 2025  
**Status**: Accepted  
**Context**: Phase 4.5 - Fixing concurrent test execution issues

## Executive Summary

RustyRay will adopt Ray's global runtime pattern with explicit lifecycle management, implemented using Rust's safety features. This provides API compatibility with Ray while solving our concurrent testing issues.

## Problem Statement

RustyRay currently uses a `OnceCell<Runtime>` global singleton that:
- Cannot be re-initialized after shutdown
- Causes test interference when running concurrently
- Leads to "Task manager shut down" errors
- Violates test isolation principles

Initial attempts to fix this with state cleanup (`clear_runtime_state`) created race conditions and didn't address the root cause.

## Decision

Adopt Ray's proven global runtime pattern with Rust-safe implementation:
- Use `RwLock<Option<Arc<Runtime>>>` for the global runtime
- Provide explicit `init()` and `shutdown()` lifecycle methods
- Runtime can be re-initialized after shutdown
- Tests use fixtures to ensure proper isolation

## Detailed Analysis

### Options Considered

#### Option 1: Pure Rust Pattern (Isolated Instances)
```rust
let runtime = Runtime::new()?;
let ref = runtime.put(data)?;
```
- **Pros**: True isolation, follows Rust ownership principles
- **Cons**: Breaks Ray API compatibility, poor ergonomics for distributed systems

#### Option 2: Thread-Local Runtime
```rust
thread_local! { static RUNTIME: RefCell<Option<Runtime>> = RefCell::new(None); }
```
- **Pros**: Some isolation between threads
- **Cons**: Doesn't work well with async, complex with work stealing

#### Option 3: Ray-Compatible Global Runtime (Chosen)
```rust
ray::init()?;
let ref = ray::put(data).await?;
ray::shutdown().await?;
```
- **Pros**: Ray API compatibility, proven pattern, good ergonomics
- **Cons**: Requires careful lifecycle management

### Why Ray's Pattern Works

1. **Distributed Systems Reality**: A process naturally has one connection to the distributed system
2. **API Ergonomics**: Passing runtime through every call is impractical
3. **Battle-Tested**: Ray has used this successfully in production for years
4. **User Expectations**: Ray users expect `ray.init()` / `ray.shutdown()` pattern

### How Ray Handles Testing

Ray faces identical challenges and solves them through:
- Explicit lifecycle control via fixtures
- Process-level isolation for true parallelism
- Careful cleanup between tests

```python
@pytest.fixture
def ray_start_regular_shared(request):
    ray.init()
    yield
    ray.shutdown()
```

## Implementation Design

### Core Runtime Module
```rust
// Global runtime storage
static RUNTIME: RwLock<Option<Arc<Runtime>>> = RwLock::new(None);

// Lifecycle management
pub fn init(config: Config) -> Result<()>
pub fn shutdown() -> Result<()>
pub fn global() -> Result<Arc<Runtime>>
pub fn is_initialized() -> bool

// User-facing API (delegates to global runtime)
pub async fn put<T>(value: T) -> Result<ObjectRef<T>>
pub async fn get<T>(obj_ref: ObjectRef<T>) -> Result<T>
```

### Test Infrastructure
```rust
pub async fn with_test_runtime<F, Fut>(test_body: F)
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = ()>
{
    // Ensure clean state
    if ray::is_initialized() {
        ray::shutdown().await.expect("Failed to cleanup");
    }
    
    ray::init(Config::test_default()).expect("Failed to init");
    
    // Run test with panic protection
    let result = std::panic::catch_unwind(/* test_body */);
    
    // Always cleanup
    ray::shutdown().await.expect("Failed to shutdown");
    
    // Re-panic if test failed
    result.unwrap();
}
```

### Safety Guarantees

1. **Thread Safety**: `RwLock` ensures safe concurrent access
2. **Initialization Safety**: `Option` represents uninitialized state
3. **Lifecycle Safety**: Explicit init/shutdown prevents use-after-free
4. **Error Handling**: All operations return `Result` for uninitialized access

## Migration Plan

### Phase 1: Core Implementation
1. Replace `OnceCell<Runtime>` with `RwLock<Option<Arc<Runtime>>>`
2. Implement init/shutdown/global functions
3. Add lifecycle tracking and safety checks

### Phase 2: Test Infrastructure
1. Create `with_test_runtime` fixture
2. Add test configuration helpers
3. Document testing patterns

### Phase 3: Test Migration
1. Update all tests to use fixtures
2. Remove `init_test_runtime` and `clear_runtime_state`
3. Verify concurrent execution works

### Phase 4: Cleanup
1. Remove old runtime implementation
2. Update all examples and documentation
3. Add migration guide for users

## Testing Strategy

### Default: Serial Execution
```bash
cargo test -- --test-threads=1
```

### Future: Process Isolation
```rust
// Potential future enhancement
#[test_process_isolated]
async fn test_with_own_process() {
    // Gets its own process like pytest-xdist
}
```

## Best Practices

1. **Always use test fixtures** - Never call `init()` directly in tests
2. **Handle initialization errors** - Check `is_initialized()` when uncertain
3. **Cleanup in production** - Call `shutdown()` before process exit
4. **Document lifecycle** - Make init/shutdown requirements clear

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Forgetting to shutdown in tests | Test fixtures with automatic cleanup |
| Double initialization | Return error, require explicit shutdown |
| Use after shutdown | Return `RuntimeNotInitialized` error |
| Concurrent init/shutdown | RwLock serializes access |

## Alternatives Rejected

### Static Singleton (Current)
- Cannot reinitialize after shutdown
- Causes test interference

### Per-Test Instances
- Breaks Ray API compatibility
- Poor ergonomics for users

### Thread-Local Storage
- Doesn't work well with async runtime
- Complex with work-stealing schedulers

## Conclusion

The Ray-compatible global runtime pattern provides the best balance of:
- API compatibility with Ray
- Ergonomic user experience
- Safe implementation in Rust
- Testability through fixtures

This decision aligns RustyRay with Ray's proven architecture while leveraging Rust's safety features.

## References

- [Ray Architecture Whitepaper](https://docs.ray.io/en/latest/ray-contribute/whitepaper.html)
- [Ray Testing Patterns](https://docs.ray.io/en/latest/ray-contribute/testing.html)
- Original analysis of concurrent test issues (this session)