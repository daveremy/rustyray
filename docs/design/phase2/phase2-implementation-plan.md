# Phase 2 Implementation Plan: Task Execution System

## Overview

After analyzing Ray's architecture with Gemini, here's our refined plan for implementing RustyRay's task execution system.

## Key Insights from Ray Analysis

### 1. Unified Task Model
Ray uses a single `TaskSpec` structure for all execution types:
- **Normal tasks**: Stateless functions that can run anywhere
- **Actor tasks**: Method calls on specific actors
- **Actor creation tasks**: Special tasks that spawn actors

This unified model simplifies the scheduler and enables seamless task-actor integration.

### 2. ObjectRef as Futures
`ObjectRef`s are Ray's killer feature - they're type-erased futures that:
- Enable async dataflow programming
- Track dependencies automatically
- Decouple task submission from execution
- Allow building complex computation graphs

### 3. Function Registration Pattern
Ray doesn't serialize functions - it serializes function IDs:
- Functions are pre-registered with unique identifiers
- Only IDs and arguments are sent over the wire
- Workers look up functions in their local registry
- This avoids the "code mobility" problem

### 4. Event-Loop Based Execution
Ray uses async event loops (like Tokio), not traditional thread pools:
- Non-blocking dependency resolution
- CPU-bound tasks spawn to separate threads
- Efficient multiplexing of many tasks

## Revised Implementation Plan

### Week 1: Core Infrastructure
```
1. Define core types:
   - TaskSpec, TaskId, TaskType
   - ObjectRef<T> with local channel backing
   - TaskArg enum (Value vs ObjectRef)
   
2. Function registry:
   - Global DashMap for function storage
   - Registration macro (#[ray::remote])
   - Type-erased function wrapper
   
3. Basic TaskManager:
   - Task queue (tokio mpsc)
   - Dependency tracking
   - Result delivery via oneshot
```

### Week 2: Execution Engine
```
1. Task submission API:
   - submit_task() returns ObjectRef
   - Automatic dependency extraction
   - Integration with actor system
   
2. Task execution:
   - Dependency resolution (await ObjectRefs)
   - Function dispatch
   - spawn_blocking for CPU tasks
   
3. Error handling:
   - Task failures
   - Missing functions
   - Serialization errors
```

### Week 3: Integration & Polish
```
1. Actor-Task integration:
   - Tasks creating actors
   - Actors submitting tasks
   - Actor method calls as tasks
   
2. Examples:
   - Parallel map
   - Reduce/aggregate
   - Pipeline pattern
   - Actor + task hybrid
   
3. Testing & benchmarks:
   - Unit tests for each component
   - Integration tests
   - Performance baseline
```

## Implementation Priority

### Must Have (MVP)
1. ✅ TaskSpec and ObjectRef types
2. ✅ Function registration and dispatch
3. ✅ Local task execution with dependencies
4. ✅ Basic actor integration
5. ✅ Core examples working

### Nice to Have
1. ⏸️ Resource requirements
2. ⏸️ Task cancellation
3. ⏸️ Priority scheduling
4. ⏸️ Metrics/observability

### Defer to Later Phases
1. ❌ Distributed execution (Phase 4)
2. ❌ Object store integration (Phase 3)
3. ❌ Fault tolerance (Phase 5)
4. ❌ Dynamic code loading

## Key Design Decisions

### 1. Tokio-Native Design
```rust
// Use tokio throughout
tokio::spawn(async move {
    // Async dependency resolution
    let deps = resolve_dependencies(task.args).await?;
    
    // CPU-bound work in blocking pool
    tokio::task::spawn_blocking(move || {
        execute_function(func_id, deps)
    }).await?
})
```

### 2. Type-Safe ObjectRefs
```rust
// Phantom type for ergonomics
pub struct ObjectRef<T> {
    id: ObjectId,
    receiver: Option<oneshot::Receiver<Vec<u8>>>,
    _phantom: PhantomData<T>,
}

// But serialized as just ObjectId
```

### 3. Macro-Based Registration
```rust
#[ray::remote]
async fn my_task(x: i32) -> i32 {
    x * 2
}

// Expands to registration code
```

## Success Metrics

1. **Functionality**: Can execute tasks with dependencies
2. **Performance**: < 1ms overhead per task
3. **Integration**: Seamless with actor system
4. **Usability**: Clean API comparable to Ray

## Next Steps

1. Review and approve this plan
2. Create task module structure
3. Implement core types
4. Build working prototype
5. Iterate based on testing

## Questions to Resolve

1. Should we use bincode or another serialization format?
2. How much type safety vs flexibility in task arguments?
3. Should we support async tasks from the start?
4. What's the right abstraction for future object store?