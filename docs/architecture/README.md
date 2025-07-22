# RustyRay Architecture

## Overview

RustyRay is a Rust implementation of Ray Core's distributed computing primitives. This document describes the architecture as of Phase 5 (v0.5.0).

## Core Components

### 1. Runtime
The global runtime manages all subsystems and provides a unified entry point.

```rust
pub struct Runtime {
    actor_system: Arc<ActorSystem>,
    task_system: Arc<TaskSystem>,
    object_store: Arc<InMemoryStore>,
}
```

**Key Features:**
- Global singleton pattern using `RwLock<Option<Arc<Runtime>>>`
- Re-initializable for test isolation
- Explicit lifecycle: `init()`, `shutdown()`, `global()`, `is_initialized()`
- Graceful shutdown with subsystem ordering
- Poisoned lock recovery

### 2. Actor System
Manages stateful actors with message passing.

```rust
pub struct ActorSystem {
    actors: Arc<DashMap<ActorId, ActorEntry>>,
    shutdown_state: Arc<AtomicU8>, // Running, ShuttingDown, Shutdown
}
```

**Key Features:**
- Actor lifecycle hooks: `on_start()`, `on_stop()`
- Typed message passing with generated enums
- Graceful shutdown with actor notification
- Thread-safe actor registry using DashMap

### 3. Task System
Executes stateless functions with dependency tracking.

```rust
pub struct TaskSystem {
    registry: Arc<FunctionRegistry>,
    manager: Arc<TaskManager>,
    object_store: Arc<InMemoryStore>,
    actor_system: Arc<ActorSystem>,
    shutdown_state: Arc<AtomicU8>,
}
```

**Key Features:**
- Dynamic function registration
- ObjectRef<T> for typed futures
- Automatic result storage in object store
- Task dependency resolution
- Cancellation and timeout support

### 4. Object Store
Shared memory store for data exchange between actors and tasks.

```rust
pub struct InMemoryStore {
    cache: Arc<Mutex<CLruCache<ObjectId, StoredObject>>>,
    config: StoreConfig,
    stats: Arc<ObjectStoreStats>,
}
```

**Key Features:**
- CLRU eviction with strict memory limits
- Zero-copy access using bytes::Bytes
- Type-safe storage with runtime checking
- Pinning support to prevent eviction
- Atomic statistics tracking

### 5. ObjectRef
Universal reference type for async values.

```rust
pub struct ObjectRef<T> {
    id: ObjectId,
    _phantom: PhantomData<T>,
}
```

**Key Features:**
- Always backed by object store
- Type-safe with phantom data
- Cloneable and shareable
- Automatic error propagation
- Future trait implementation

## Data Flow

### Task Execution Flow
1. User calls `#[remote]` function
2. TaskSystem creates TaskSpec with serialized args
3. TaskManager spawns async task
4. Function executes and returns result
5. Result automatically stored in object store
6. ObjectRef returned to user
7. User calls `.get()` to retrieve value

### Actor Message Flow
1. User calls actor method via generated handle
2. Message sent to actor's mpsc channel
3. Actor processes message in order
4. Result sent back via oneshot channel
5. For large data, stored in object store
6. ObjectRef returned to user

### Object Sharing Flow
1. User calls `ray::put(data)`
2. Data serialized and stored in object store
3. ObjectRef returned
4. ObjectRef passed to tasks/actors
5. Recipients call `ray::get(&ref)`
6. Data retrieved from shared store

## Memory Management

### Current State (Phase 5)
- CLRU eviction based on memory limits
- Pinning prevents eviction of active objects
- No reference counting (planned for Phase 6)
- Manual cleanup on shutdown

### Planned (Phase 6)
- Automatic reference counting
- GC for orphaned objects
- Memory pressure handling
- Leak detection tools

## Error Handling

### Error Propagation
- Errors stored in object store with special markers
- ObjectRef::get() returns Result<T, RustyRayError>
- Errors include context (task ID, function name)
- Graceful degradation on component failures

### Error Types
- `TaskNotFound`: Task function not registered
- `ActorNotFound`: Actor not in registry
- `ObjectNotFound`: Object evicted or missing
- `SerializationError`: Type mismatch or corruption
- `Internal`: System-level failures

## Concurrency Model

### Thread Safety
- All components use Arc for shared ownership
- DashMap for concurrent collections
- Mutex with poisoned lock recovery
- Atomic state transitions

### Async Runtime
- Built on Tokio
- All actor/task execution is async
- Blocking operations use spawn_blocking
- Careful handling of async drop

## Test Infrastructure

### Test Isolation
- `with_test_runtime()` fixture
- Panic safety with catch_unwind
- Automatic runtime cleanup
- Clear error on concurrent tests

### Requirements
- Tests must run with `--test-threads=1`
- Runtime supports concurrent operations
- Test isolation is for lifecycle only

## Macro System

### Remote Functions
```rust
#[rustyray::remote]
async fn compute(x: i32) -> i32 { x * 2 }
```
- Automatic registration via linkme
- Generated module with remote() function
- Type-safe invocation

### Actors
```rust
#[rustyray::actor]
struct Counter { value: i32 }

#[rustyray::actor_methods]
impl Counter {
    pub fn new() -> Self { Self { value: 0 } }
    pub async fn increment(&mut self) -> i32 {
        self.value += 1;
        self.value
    }
}
```
- Generated message enum
- Typed actor handle
- Automatic dispatch

### Runtime
```rust
#[rustyray::main]
async fn main() -> Result<()> {
    // Runtime automatically initialized
}
```

## Performance Considerations

### Current Optimizations
- Zero-copy with bytes::Bytes
- O(1) operations in object store
- Concurrent collections (DashMap)
- Atomic statistics

### Future Optimizations (Phase 7)
- Replace std locks with parking_lot
- Cache runtime references
- Batch operations
- Memory pooling

## Security Considerations

- No authentication/authorization yet
- Local-only, no network exposure
- Type safety prevents many errors
- Resource limits planned

## Comparison with Ray

### Similarities
- Global runtime pattern
- Actor model with messages
- Task execution with dependencies
- Shared object store
- ObjectRef futures

### Differences
- Rust type safety throughout
- No cross-language support yet
- Local-only (no distribution)
- Simpler error model
- Test isolation requirements