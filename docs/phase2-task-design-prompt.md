# Gemini Analysis Request: Ray Task System

## Context
We're implementing RustyRay, a Rust version of Ray Core. We've completed Phase 1 (local actor system) and are now planning Phase 2 (task execution system). We need to understand Ray's task implementation deeply to design our Rust version correctly.

## Files to Analyze
Please analyze these key Ray source files:

```bash
# Core task implementation
@~/code/ray/src/ray/core_worker/task_manager.cc
@~/code/ray/src/ray/core_worker/task_interface.h
@~/code/ray/src/ray/core_worker/task_execution.cc

# Task specification and arguments
@~/code/ray/src/ray/common/task/task_spec.h
@~/code/ray/src/ray/common/task/task_util.h
@~/code/ray/src/ray/core_worker/task_arg.h

# Python task API
@~/code/ray/python/ray/remote_function.py
@~/code/ray/python/ray/_private/worker.py

# Protobuf definitions
@~/code/ray/src/ray/protobuf/common.proto
@~/code/ray/src/ray/protobuf/core_worker.proto
```

## Questions to Answer

### 1. Core Concepts
- What exactly is a Ray task vs an actor method call?
- How are tasks represented internally (TaskSpec structure)?
- What's the difference between normal tasks and actor tasks?
- How do tasks relate to the object store?

### 2. Task Lifecycle
- How are tasks created and submitted?
- What's the flow from task submission to execution?
- How are task results returned and stored?
- How does task cancellation work?

### 3. Dependencies and Scheduling
- How does Ray track task dependencies?
- How are object dependencies resolved before task execution?
- What's the role of TaskManager vs Scheduler?
- How does locality-aware scheduling work?

### 4. Task Arguments
- How are task arguments passed (by value vs by reference)?
- How do ObjectRefs work as task arguments?
- What's the serialization strategy for task args?

### 5. Integration Points
- How do tasks interact with actors?
- Can tasks create actors? How?
- How do actors submit tasks?
- What's the relationship with the Global Control Store (GCS)?

### 6. Key Design Patterns
- What patterns does Ray use for async task execution?
- How does Ray handle task failures and retries?
- What's the threading model for task execution?
- How does Ray ensure at-most-once or at-least-once execution?

### 7. Rust Implementation Considerations
Based on Ray's design:
- Should we use async/await or threads for task execution?
- How should we handle the type erasure for generic task functions?
- What's the best way to implement ObjectRefs in Rust?
- Should we use tokio tasks or a custom worker pool?
- How can we make tasks zero-copy where possible?

## Current RustyRay Status
We have:
- Actor system with ActorId, ActorRef, ActorSystem
- Async message passing with tokio
- Type-erased actor messages using Any trait
- Graceful shutdown and lifecycle management

## Phase 2 Goals
We want to add:
- Stateless task execution
- Task dependencies
- Integration with existing actors
- Foundation for future object store integration

Please provide:
1. A clear explanation of Ray's task architecture
2. Key insights about the design decisions
3. Recommendations for our Rust implementation
4. Potential pitfalls to avoid
5. Which Ray features we should defer to later phases