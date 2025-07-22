# ADR-002: Runtime Architecture Refactor

**Date:** 2025-02-01  
**Status:** Accepted  
**Context:** Phase 5 - Fixing concurrent test execution issues

## Context

The original runtime implementation used `OnceCell<Runtime>` which prevented re-initialization after shutdown. This caused issues with test isolation since tests need to initialize and shutdown the runtime independently.

## Decision

Refactor the runtime from `OnceCell<Runtime>` to `RwLock<Option<Arc<Runtime>>>`.

## Rationale

1. **Re-initializability**: Tests can shutdown and reinitialize the runtime
2. **Thread Safety**: RwLock provides safe concurrent access
3. **Explicit Lifecycle**: Clear init/shutdown/reinit pattern
4. **Ray Compatibility**: Maintains global runtime pattern like Ray

## Consequences

### Positive
- Tests can run in isolation with proper cleanup
- Runtime can be reinitialized after shutdown
- Graceful handling of poisoned locks
- Clear lifecycle management

### Negative
- Tests must run single-threaded (`--test-threads=1`)
- Slightly more complex implementation
- Small performance overhead from RwLock

### Neutral
- No API changes for users
- Runtime still global (by design)

## Alternatives Considered

1. **Thread-local Runtime**: Would allow concurrent tests but diverges from Ray's design
2. **Runtime Factory**: More complex, doesn't match Ray's global pattern
3. **Test-only Runtime**: Would create divergence between test and production code

## Implementation Details

- Added `runtime::shutdown_async()` for graceful async shutdown
- Implemented poisoned lock recovery
- Created `with_test_runtime()` fixture for test isolation
- All 84 tests updated to use new pattern