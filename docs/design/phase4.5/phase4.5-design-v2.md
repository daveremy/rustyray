# Phase 4.5: Actor-Object Store Integration Design (v2)

## Overview

This document outlines the design for integrating the object store with actors and tasks. Since RustyRay has no existing users, we can make optimal architectural decisions without backward compatibility constraints.

## Design Goals

1. **Unified Architecture**: Single object store shared by all components
2. **First-Class Integration**: Object store is a core part of actor/task APIs
3. **Zero-Copy Efficiency**: Large data sharing without serialization overhead
4. **Type Safety**: Leverage Rust's type system throughout
5. **Clean APIs**: Intuitive, idiomatic Rust interfaces

## Proposed Architecture

### 1. Runtime with Shared Object Store

```rust
pub struct Runtime {
    actor_system: Arc<ActorSystem>,
    task_system: Arc<TaskSystem>,
    object_store: Arc<InMemoryStore>,  // Shared by all systems
}

impl Runtime {
    pub fn new(config: RuntimeConfig) -> Self {
        let object_store = Arc::new(InMemoryStore::new(config.object_store_config));
        let actor_system = Arc::new(ActorSystem::new(object_store.clone()));
        let task_system = Arc::new(TaskSystem::new(
            actor_system.clone(),
            object_store.clone()
        ));
        
        Runtime {
            actor_system,
            task_system,
            object_store,
        }
    }
}
```

### 2. Actor with Built-in Context

```rust
#[async_trait]
pub trait Actor: Send + Sync + 'static {
    /// Called when actor starts
    async fn on_start(&mut self, ctx: &ActorContext) -> Result<()> {
        Ok(())
    }
    
    /// Handle incoming messages with context
    async fn handle(
        &mut self, 
        msg: Box<dyn Any + Send>, 
        ctx: &ActorContext
    ) -> Result<Box<dyn Any + Send>>;
    
    /// Called before actor stops
    async fn on_stop(&mut self, ctx: &ActorContext) -> Result<()> {
        Ok(())
    }
}

pub struct ActorContext {
    actor_id: ActorId,
    object_store: Arc<InMemoryStore>,
}

impl ActorContext {
    /// Store an object and get a reference
    pub fn put<T: Send + Sync + 'static>(&self, value: T) -> Result<ObjectRef<T>> {
        let bytes = serialize(&value)?;
        let id = self.object_store.put(bytes)?;
        Ok(ObjectRef::with_store(id, self.object_store.clone()))
    }
    
    /// Retrieve an object by reference
    pub async fn get<T: Send + Sync + 'static>(&self, obj_ref: &ObjectRef<T>) -> Result<T> {
        obj_ref.get().await
    }
}
```

### 3. Enhanced Task API

```rust
impl TaskSystem {
    /// Submit a task that can access the object store
    pub fn submit<F, T>(&self, f: F) -> ObjectRef<T>
    where
        F: FnOnce(TaskContext) -> T + Send + 'static,
        T: Send + Sync + 'static,
    {
        // Task gets its own context with object store access
    }
}

pub struct TaskContext {
    object_store: Arc<InMemoryStore>,
}

impl TaskContext {
    pub fn put<T: Send + Sync + 'static>(&self, value: T) -> Result<ObjectRef<T>>;
    pub async fn get<T: Send + Sync + 'static>(&self, obj_ref: &ObjectRef<T>) -> Result<T>;
}
```

### 4. Unified ObjectRef Implementation

```rust
pub struct ObjectRef<T> {
    id: ObjectId,
    store: Arc<InMemoryStore>,
    _phantom: PhantomData<T>,
}

impl<T: Send + Sync + 'static> ObjectRef<T> {
    /// Always store-backed, no local channels
    pub fn with_store(id: ObjectId, store: Arc<InMemoryStore>) -> Self {
        Self {
            id,
            store,
            _phantom: PhantomData,
        }
    }
    
    /// Get the value (async for consistency with distributed future)
    pub async fn get(&self) -> Result<T> {
        let bytes = self.store.get(self.id)?;
        deserialize(&bytes)
    }
}
```

### 5. Global Convenience Functions

```rust
/// Store a value in the global object store
pub fn put<T: Send + Sync + 'static>(value: T) -> Result<ObjectRef<T>> {
    runtime::global()?.object_store().put(value)
}

/// Retrieve a value from the global object store
pub async fn get<T: Send + Sync + 'static>(obj_ref: &ObjectRef<T>) -> Result<T> {
    obj_ref.get().await
}
```

## Key Design Decisions

### 1. Always Use Object Store for ObjectRefs
- Remove the dual local/store mode from current ObjectRef
- All ObjectRefs are backed by the object store
- Simpler, more consistent behavior

### 2. Context as First-Class Citizen
- Every actor gets a context in every method
- No optional parameters or separate traits
- Clean, consistent API

### 3. Unified Store Access Pattern
- Both actors and tasks use the same context pattern
- Same put/get methods everywhere
- Reduces cognitive load

### 4. Memory Management
- Single configurable memory limit for entire runtime
- LRU eviction with pinning support (already implemented)
- Future: per-actor memory quotas if needed

## Implementation Steps

### Step 1: Refactor Runtime and ObjectRef
1. Add object_store to Runtime
2. Simplify ObjectRef to always use store
3. Update TaskSystem to accept external store
4. Remove local channel mode from ObjectRef

### Step 2: Actor System Integration
1. Add object_store to ActorSystem
2. Create ActorContext
3. Update Actor trait with context parameters
4. Modify actor runtime to provide context

### Step 3: Task System Enhancement
1. Create TaskContext
2. Update task submission APIs
3. Ensure tasks can access object store
4. Update remote function handling

### Step 4: Examples and Testing
1. Actor-task data pipeline
2. Large object sharing
3. State persistence
4. Performance benchmarks

## Benefits of This Design

1. **Simplicity**: One way to create ObjectRefs, one way to access store
2. **Consistency**: Same patterns across actors and tasks
3. **Performance**: Zero-copy sharing, no duplicate storage
4. **Future-Proof**: Easy to extend to distributed object store

## Potential Enhancements

1. **Object Lifetime**: Add reference counting for automatic cleanup
2. **Streaming**: Support for large object streaming
3. **Persistence**: Optional disk backing for object store
4. **Monitoring**: Built-in metrics for object store usage

## Success Metrics

1. Clean API that feels natural in Rust
2. Efficient large object sharing between actors/tasks
3. No performance regression for small messages
4. Clear error messages for type mismatches
5. Comprehensive examples demonstrating patterns