# RustyRay Phase 5 Continuation Prompt

## Project Context

You are continuing work on RustyRay, a Rust implementation of Ray Core's distributed actor system. The project is at commit `86e7ad5` with Phase 4.5 successfully completed.

**GitHub Repository**: https://github.com/daveremy/rustyray

## Current State (Phase 4.5 Complete)

### What Was Just Accomplished
1. **Universal ObjectRef**: Moved from task-specific to core level
2. **Shared Object Store**: All components share single store in Runtime
3. **Ray-style API**: Global `ray::put()` and `ray::get()` functions
4. **Error Propagation**: Errors stored in object store with `__RUSTYRAY_ERROR__` markers
5. **Actor-Task Data Sharing**: Seamless object sharing between actors and tasks
6. **All Tests Passing**: 60 tests including new integration tests
7. **Grade A from Gemini**: Comprehensive review praised architecture and documentation

### Key Architectural Decisions
- ObjectRef is always store-backed (no dual mode)
- Type-erased storage (store bytes, deserialize on retrieval)
- Errors are just another type of stored object
- Single shared object store for entire runtime

## Phase 5: Reference Counting & Memory Management

### Design Already Complete
The design document is at `docs/design/phase5/phase5-reference-counting-design.md`. Key decisions:
- **Owner-based reference counting** (simpler than distributed consensus)
- **Integrate with ObjectRef lifecycle** (new/clone/drop)
- **Prevent premature eviction** (only evict objects with zero references)
- **Task reference tracking** (increment when passed to tasks)

### Implementation Plan

1. **Update ObjectRef Implementation**:
```rust
// In src/object_ref.rs
impl<T> ObjectRef<T> {
    pub(crate) fn new(id: ObjectId, store: Arc<InMemoryStore>) -> Self {
        store.increment_ref_count(id);  // Add this
        Self { id, store, _phantom: PhantomData }
    }
}

impl<T> Clone for ObjectRef<T> {
    fn clone(&self) -> Self {
        self.store.increment_ref_count(self.id);  // Add this
        Self { /* ... */ }
    }
}

impl<T> Drop for ObjectRef<T> {
    fn drop(&mut self) {
        self.store.decrement_ref_count(self.id);  // Add this
    }
}
```

2. **Update Object Store**:
```rust
// In src/object_store/types.rs
pub struct ObjectEntry {
    pub data: Vec<u8>,
    pub ref_count: AtomicU32,  // Add this
    pub created_at: SystemTime,
    pub last_accessed: AtomicU64,
    pub is_pinned: AtomicBool,
}

// In src/object_store/store.rs
impl InMemoryStore {
    pub(crate) fn increment_ref_count(&self, id: ObjectId) { /* ... */ }
    pub(crate) fn decrement_ref_count(&self, id: ObjectId) { /* ... */ }
    fn can_evict(&self, entry: &ObjectEntry) -> bool {
        !entry.is_pinned.load(Ordering::Acquire) &&
        entry.ref_count.load(Ordering::Acquire) == 0  // Check ref count
    }
}
```

3. **Task Reference Tracking**:
- Track ObjectRefs passed as task arguments
- Increment count when task submitted
- Decrement when task completes

4. **Testing**:
- Test ref count increment/decrement
- Test eviction respects ref counts
- Test circular reference scenarios
- Test memory leak detection

## Technical Environment

### Development Setup
```bash
# Clone if needed
git clone https://github.com/daveremy/rustyray
cd rustyray

# Common commands
cargo build
cargo test
cargo fmt
cargo clippy
```

### Using Gemini for Analysis
When you need to analyze Ray's implementation:
```bash
# Load API key and run analysis
export $(cat .env | grep GEMINI_API_KEY) && gemini -p "@~/code/ray/src/ray/core_worker/reference_count.cc How does Ray implement reference counting?"
```

### Key Files to Modify

1. **Core Implementation**:
   - `crates/rustyray-core/src/object_ref.rs` - Add ref counting hooks
   - `crates/rustyray-core/src/object_store/types.rs` - Add ref_count field
   - `crates/rustyray-core/src/object_store/store.rs` - Implement counting methods
   - `crates/rustyray-core/src/task/system.rs` - Track task references

2. **Tests**:
   - Create `crates/rustyray-core/src/object_store/ref_count_tests.rs`
   - Update integration tests in `crates/rustyray-core/tests/`

## Important Context

### What NOT to Change
- The universal ObjectRef design (it's working well)
- The error marker approach (deferred to Phase 6 for structured errors)
- The ray module API (it's clean and Ray-compatible)

### Performance Considerations
- Use atomic operations for thread safety
- Don't add overhead to the hot path
- Reference counting should be near zero-cost

### Future Phases Overview
- **Phase 6**: Metadata & Error Enhancement (structured errors, metadata)
- **Phase 7**: Performance Optimizations (notification instead of polling)
- **Phase 8**: Distributed Foundation (multi-node support)
- **Phase 9**: Production Features (fault tolerance, monitoring)

## Gemini Review Notes from Phase 4.5

Key recommendations for future work:
1. Replace polling in `ObjectRef::get()` with notification-based approach (Phase 7)
2. Implement structured errors instead of string errors (Phase 6)
3. Optimize batch operations for true parallelism (Phase 7)
4. Consider size-based routing for small objects (Phase 7)

## Documentation Standards

Continue the excellent documentation practices:
- Update ROADMAP.md when Phase 5 is complete
- Create ADR-002 if making significant architectural decisions
- Update DECISIONS.md with any new choices
- Keep examples updated with new patterns

## Success Criteria for Phase 5

1. All objects have accurate reference counts
2. No objects evicted while still referenced
3. All objects eventually evictable when unreferenced
4. <5% performance overhead
5. No memory leaks in stress tests
6. Clear debugging tools for reference count issues

## Starting Commands

```bash
# Get latest code
git pull origin main

# Create a new branch for Phase 5
git checkout -b phase5-reference-counting

# Run tests to ensure clean start
cargo test

# Start implementing!
```

## Questions for Design Decisions

As you implement, consider:
1. Should we add weak references for cache-friendly patterns?
2. How to handle reference count overflow (unlikely but possible)?
3. Should we add debug mode with detailed ref count logging?
4. How to detect and report potential memory leaks?

Good luck with Phase 5! The foundation from Phase 4.5 is solid, and the reference counting design is clear. This phase will make RustyRay's memory management production-ready.