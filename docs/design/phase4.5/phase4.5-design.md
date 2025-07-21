# Phase 4.5: Actor-Object Store Integration Design

## Overview

This document outlines the design for integrating the object store (completed in Phase 4) with the actor and task systems to enable efficient data sharing and state management.

## Current State

### What We Have
1. **Object Store** (`object_store/`): Production-ready in-memory store with:
   - CLRU cache with memory limits
   - Type-safe put/get operations
   - Pinning support
   - Zero-copy access via `bytes::Bytes`

2. **Task System** (`task/system.rs`): Already partially integrated:
   - Has its own `InMemoryStore` instance
   - `put()` method stores objects and returns `ObjectRef`
   - `ObjectRef` can resolve values from store

3. **Actor System** (`actor/`): Currently isolated:
   - Message passing via `Box<dyn Any + Send>`
   - No access to object store
   - Cannot share large data efficiently

## Design Goals

1. **Unified Object Store**: Single store instance shared across all systems
2. **Zero-Copy Data Sharing**: Actors and tasks share data without duplication
3. **Type Safety**: Maintain Rust's type guarantees
4. **Backward Compatibility**: Existing actor/task code continues to work
5. **Ergonomic API**: Simple `ray::put()` and `ray::get()` functions

## Proposed Architecture

### 1. Shared Object Store in Runtime

```rust
pub struct Runtime {
    actor_system: Arc<ActorSystem>,
    task_system: Arc<TaskSystem>,
    object_store: Arc<InMemoryStore>,  // NEW: Shared store
}
```

### 2. Actor Context with Store Access

```rust
pub struct ActorContext {
    handle: ActorHandle,
    object_store: Arc<InMemoryStore>,
}

impl ActorContext {
    pub fn put<T: Send + Sync + 'static>(&self, value: T) -> Result<ObjectRef<T>>;
    pub fn get<T: Send + Sync + 'static>(&self, ref: &ObjectRef<T>) -> Result<T>;
}
```

### 3. Enhanced Actor Trait

```rust
#[async_trait]
pub trait Actor: Send + Sync + 'static {
    async fn on_start(&mut self, ctx: &ActorContext) -> Result<()> {
        Ok(())
    }
    
    async fn on_message(&mut self, msg: Box<dyn Any + Send>, ctx: &ActorContext) -> Result<()>;
}
```

### 4. Global API Functions

```rust
// In rustyray::ray module
pub fn put<T: Send + Sync + 'static>(value: T) -> Result<ObjectRef<T>> {
    runtime::global()?.put(value)
}

pub fn get<T: Send + Sync + 'static>(object_ref: &ObjectRef<T>) -> Result<T> {
    runtime::global()?.get(object_ref)
}
```

## Implementation Plan

### Phase 1: Shared Object Store (Priority: High)
1. Add `object_store` field to `Runtime`
2. Create store with configurable memory limit
3. Pass store reference to both `ActorSystem` and `TaskSystem`
4. Update `TaskSystem::new()` to accept external store

### Phase 2: Actor Integration (Priority: High)
1. Add `object_store` reference to `ActorSystem`
2. Create `ActorContext` struct
3. Update `Actor` trait to include context parameter
4. Modify actor spawn to provide context

### Phase 3: API Design (Priority: Medium)
1. Implement `ray::put()` and `ray::get()` functions
2. Add store access methods to `Runtime`
3. Update macros to support object store operations
4. Ensure `ObjectRef` serialization works across boundaries

### Phase 4: Examples & Documentation (Priority: Low)
1. Actor sharing large dataset with tasks
2. Task pipeline with object passing
3. Actor state persistence example
4. Performance benchmarks

## Technical Considerations

### Memory Management
- Single memory limit shared across all systems
- Consider per-actor memory quotas
- Handle eviction gracefully

### Concurrency
- Object store is already thread-safe (uses `DashMap`)
- ObjectRefs are `Send + Sync`
- No additional synchronization needed

### Type Safety
- Maintain compile-time type checking where possible
- Runtime type verification in object store
- Clear error messages for type mismatches

### Performance
- Zero-copy access for large objects
- Minimal overhead for small objects
- Consider direct value passing for tiny objects

## Migration Strategy

1. **Backward Compatibility**: Existing actor code works unchanged
2. **Opt-in Enhancement**: New context parameter is optional
3. **Gradual Adoption**: Can migrate actors individually

## Risks and Mitigations

### Risk: Memory Pressure
**Mitigation**: Configurable memory limits, eviction policies, monitoring

### Risk: Type Confusion
**Mitigation**: Strong typing on ObjectRef, runtime checks, good errors

### Risk: Actor API Complexity
**Mitigation**: Keep simple message passing as default, context is advanced feature

## Success Criteria

1. Actors can share large objects with tasks efficiently
2. Tasks can return ObjectRefs that actors can resolve
3. Memory usage stays within configured limits
4. API is intuitive and well-documented
5. Performance is better than serialization for large objects

## Alternative Approaches Considered

1. **Separate Stores**: Keep isolated stores, add cross-store references
   - Rejected: Complex, inefficient, breaks zero-copy

2. **Actor-Only Store**: Give only actors access to store
   - Rejected: Tasks already use store, would break existing code

3. **Automatic Serialization**: Serialize all large messages to store
   - Rejected: Unexpected performance characteristics, magic behavior

## Next Steps

1. Review design with Gemini
2. Implement shared object store
3. Add actor context support
4. Create comprehensive examples
5. Performance testing