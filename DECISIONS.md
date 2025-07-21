# RustyRay Architectural Decisions

This document tracks key architectural decisions made throughout the RustyRay project. For detailed analysis of specific decisions, see the ADR (Architecture Decision Record) documents in the `docs/` folder.

## Phase 1: Actor System

### Actor ID Generation
**Decision:** Use UUID v4 for ActorId  
**Rationale:** Simple, no coordination needed, sufficient uniqueness  
**Status:** ✅ Proven successful

### Message Passing
**Decision:** Use tokio mpsc channels for actor mailboxes  
**Rationale:** Async-native, bounded for backpressure, good performance  
**Status:** ✅ Working well

### Actor Trait Design
**Decision:** Single `handle` method taking `Box<dyn Any>`  
**Rationale:** Maximum flexibility, follows Ray's pattern  
**Trade-off:** Type erasure requires downcasting  
**Status:** ⚠️ Considering typed actors in future

## Phase 2: Task System

### Task Future Design  
**Decision:** ObjectRef implements Future trait  
**Rationale:** Natural Rust pattern, enables `await` syntax  
**Status:** ✅ Excellent ergonomics

### Dependency Resolution
**Decision:** TaskManager handles dependency tracking  
**Rationale:** Centralized coordination, simpler than distributed  
**Status:** ✅ Sufficient for single-node

### Serialization
**Decision:** Use bincode for task arguments  
**Rationale:** Fast, compact, pure Rust  
**Trade-off:** Not cross-language compatible yet  
**Status:** ✅ Good for now, may revisit for Python interop

## Phase 3: Macro System

### Registration Strategy
**Decision:** Use linkme for compile-time registration  
**Rationale:** Zero runtime overhead, automatic discovery  
**Status:** ✅ Eliminates manual registration

### Error Handling in Macros
**Decision:** Support both Result<T, E> and T return types  
**Rationale:** Flexibility for users, good error propagation  
**Status:** ✅ Natural Rust patterns

### Actor Method Design
**Decision:** Generate typed handles, eliminate Box<dyn Any>  
**Rationale:** Type safety, better developer experience  
**Status:** ✅ 70% boilerplate reduction achieved

## Phase 4: Object Store

### Cache Implementation
**Decision:** Switch from moka to CLRU  
**Rationale:** Strict memory limits, no 75% overruns  
**Status:** ✅ Memory bounds respected

### Type Safety
**Decision:** Runtime type checking with TypeId  
**Rationale:** Allows heterogeneous storage, catches errors  
**Trade-off:** Runtime overhead vs compile-time safety  
**Status:** ✅ Good balance

### Pinning Design
**Decision:** Atomic pinned size tracking  
**Rationale:** O(1) operations, accurate memory accounting  
**Status:** ✅ Eliminated O(n) bottleneck

## Phase 4.5: Object Store Integration

### ObjectRef Location
**Decision:** Move to core level (`src/object_ref.rs`)  
**Rationale:** Universal type, not task-specific  
**Impact:** Breaking change but cleaner architecture  
**Status:** ✅ Completed successfully

### Storage Model
**Decision:** Always store-backed, no dual channel/store mode  
**Rationale:** Simplicity, consistency, enables sharing  
**Status:** ✅ Unified model working well

### Type Erasure
**Decision:** Store as Vec<u8>, deserialize on retrieval  
**Rationale:** Simple, flexible, cross-language potential  
**Trade-off:** Type errors at runtime, not compile time  
**Status:** ✅ Pragmatic solution

### Error Handling
**Decision:** Store errors in object store with marker prefix  
**Rationale:** Unifies error propagation with results  
**Innovation:** Errors are just another type of object  
**Status:** ✅ Elegant solution

### API Design
**Decision:** Global `ray::put()`/`ray::get()` functions  
**Rationale:** Familiar to Ray users, simple API  
**Status:** ✅ Clean and intuitive

## Phase 5: Reference Counting (Planned)

### Strategy
**Decision:** Owner-based reference counting  
**Rationale:** Simpler than distributed consensus  
**Alternative:** Fully distributed (rejected - too complex)  
**Status:** 📋 Designed, not implemented

### Implementation
**Decision:** Integrate with ObjectRef lifecycle  
**Rationale:** Automatic, can't be forgotten  
**Status:** 📋 Design complete

## Open Decisions

### Cross-Language Support
**Question:** When to add Apache Arrow support?  
**Options:**
1. Phase 6 - Soon, for Python interop
2. Phase 8 - With distributed features
3. Post-1.0 - Focus on Rust first

### Shared Memory
**Question:** When to implement Plasma-style shared memory?  
**Options:**
1. Phase 7 - With performance optimizations
2. Phase 8 - With distributed features
3. Optional feature - Not all users need it

### Error Type Design
**Question:** Match Ray's errors exactly or innovate?  
**Options:**
1. Exact match - Maximum compatibility
2. Rust-idiomatic - Better error handling
3. Hybrid - Core types match, extended with Rust features

### Protocol Compatibility
**Question:** Binary compatible with Ray or Rust-native?  
**Options:**
1. Full compatibility - Can join Ray clusters
2. Rust-native - Better performance, simpler
3. Bridge mode - Adapter for compatibility

## Decision-Making Principles

1. **Simplicity First**: Choose the simpler solution when possible
2. **Rust Idioms**: Leverage Rust's strengths (ownership, types, traits)
3. **Incremental**: Decisions can be revisited as we learn
4. **Performance**: But not premature optimization
5. **User Experience**: APIs should be intuitive and safe

## Lessons Learned

1. **Type erasure is acceptable** for distributed systems
2. **Global state is fine** for runtime infrastructure  
3. **Atomic operations** solve many concurrency issues
4. **Macros dramatically improve** developer experience
5. **Starting simple** and iterating works well

---

For detailed analysis of specific decisions, see:
- [ADR-001: Phase 4.5 Object Store Integration](docs/ADR-001-phase4.5-decisions.md)
- (More ADRs to be added as we progress)