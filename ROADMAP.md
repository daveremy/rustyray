# RustyRay Development Roadmap

This document outlines the development plan for RustyRay, a Rust implementation of Ray Core. Each phase builds on the previous one, allowing us to develop incrementally while learning Ray's architecture.

## Progress Summary

- **Phase 1: Local Actor System** ✅ Completed
  - Implemented core actor infrastructure with ActorId, ActorRef, and ActorSystem
  - Built async message passing using tokio channels
  - Added actor lifecycle management (start, stop, graceful shutdown)
  - Created working counter example demonstrating stateful actors
  - Achieved all primary deliverables for a working local actor system

- **Phase 2: Task Execution System** ✅ Completed
  - Implemented comprehensive task system with TaskManager and TaskSystem
  - Built ObjectRef with watch channels for multi-consumer futures
  - Added function registry with task_function! macro
  - Integrated task cancellation and timeout mechanisms
  - Fixed all critical issues from code review
  - Created multiple examples demonstrating task features

- **Phase 2.5: Code Review Improvements** ✅ Completed
  - Fixed TaskBuilder error propagation
  - Implemented ObjectRef cloning with tokio::sync::watch
  - Added comprehensive cancellation/timeout system
  - Replaced sleep() in tests with proper synchronization
  - Optimized memory allocations with Bytes type

- **Phase 3.0: Macro System for API Ergonomics** 🚀 Next Focus
  - Design and implement procedural macros for better developer experience

## Overview

Based on our analysis of Ray's C++ implementation and comprehensive code review, we'll build RustyRay in phases:

1. **Local Actor System** ✅ (Completed)
2. **Task Execution System** ✅ (Completed)
3. **Macro System for API Ergonomics** (Next Focus)
4. **Local Shared Memory Object Store**
5. **Distributed Runtime**
6. **Production Features**

---

## Phase 1: Local Actor System ✅ (Completed)

Build a single-node actor system that demonstrates Ray's actor model in Rust.

### 1.1 Basic Actor Infrastructure
- [x] Define core types (ActorId, ActorHandle)
- [x] Create Actor trait for user-defined actors
- [x] Implement ActorRef for sending messages
- [x] Build ActorSystem for managing actors

### 1.2 Message Passing
- [x] Define message types and serialization
- [x] Implement mailbox with tokio channels
- [x] Support async message handling
- [x] Add request-response pattern

### 1.3 Actor Lifecycle
- [x] Actor creation and registration
- [x] Graceful shutdown
- [x] Error handling and supervision
- [ ] Basic actor restart on failure (deferred to Phase 5)

### 1.4 Example Actors
- [x] Counter actor (stateful example)
- [x] Echo actor (stateless example)
- [ ] Calculator actor (request-response)
- [ ] Benchmark simple operations

**Deliverable**: Working local actor system with examples ✅

---

## Phase 2: Task Execution System ✅ (Completed)

Add Ray's task concept - stateless function execution with dependencies.

### 2.1 Task Infrastructure
- [x] Define Task and TaskId types
- [x] Create TaskSpec for task definitions
- [x] Implement TaskManager with worker pattern
- [x] Add ObjectRef as typed futures

### 2.2 Task Scheduling
- [x] Local task queue with mpsc channels
- [x] Async task execution with tokio
- [x] Task dependency tracking and resolution
- [x] Result handling with ObjectRef

### 2.3 Integration with Actors
- [x] Unified TaskSystem and ActorSystem
- [x] Shared execution environment
- [x] Unified error handling with Result types
- [x] Comprehensive shutdown mechanisms

### 2.4 Additional Features (Phase 2.5)
- [x] Task cancellation and timeouts
- [x] Error propagation through ObjectRef
- [x] Zero-copy object store with Bytes
- [x] Instance-based function registry

**Deliverable**: Tasks and actors working together locally ✅

---

## Phase 3: Macro System for API Ergonomics 🎯 (Next Focus)

Implement procedural macros to dramatically improve the developer experience.

### 3.1 Function Macros
- [ ] #[rustyray::remote] for automatic function registration
- [ ] Automatic serialization/deserialization handling
- [ ] Type-safe function signatures
- [ ] Support for async functions

### 3.2 Actor Macros
- [ ] #[rustyray::actor] for actor structs
- [ ] #[rustyray::actor_methods] for typed method handles
- [ ] Eliminate Box<dyn Any> boilerplate
- [ ] Generate typed ActorHandle<T> automatically

### 3.3 Runtime Macros
- [ ] #[rustyray::main] for global runtime initialization
- [ ] Automatic TaskSystem/ActorSystem setup
- [ ] Global context for task submission
- [ ] Clean shutdown handling

### 3.4 Examples and Documentation
- [ ] Update all examples to use macro API
- [ ] Comprehensive macro documentation
- [ ] Migration guide from manual API
- [ ] Performance benchmarks

**Deliverable**: Python-like simplicity with Rust's type safety

---

## Phase 4: Local Shared Memory Object Store 📦

Implement a simplified version of Plasma for efficient data sharing.

### 4.1 Object Management
- [ ] ObjectId generation
- [ ] Object serialization/deserialization
- [ ] Reference counting
- [ ] Basic garbage collection

### 4.2 Storage Backend
- [ ] In-memory store (HashMap-based)
- [ ] Shared memory support (using memmap2)
- [ ] Object eviction policies
- [ ] Memory limits and quotas

### 4.3 Integration
- [ ] Actors can put/get objects
- [ ] Tasks automatically store results
- [ ] Zero-copy reads where possible
- [ ] Object transfer between actors

**Deliverable**: Local object store with examples

---

## Phase 5: Distributed Runtime 🌐

Transform RustyRay into a true distributed system.

### 5.1 Networking Layer
- [ ] gRPC service definitions (using tonic)
- [ ] Node discovery and registration
- [ ] Heartbeat and failure detection
- [ ] Message routing between nodes

### 5.2 Global Control Store (GCS)
- [ ] Actor registry (global actor table)
- [ ] Node information table
- [ ] Object location table
- [ ] Distributed metadata management

### 5.3 Distributed Actors
- [ ] Remote actor creation
- [ ] Cross-node message passing
- [ ] Actor migration (stretch goal)
- [ ] Location transparency

### 5.4 Distributed Tasks
- [ ] Task scheduling across nodes
- [ ] Load balancing
- [ ] Data locality awareness
- [ ] Distributed dependency resolution

**Deliverable**: Multi-node RustyRay cluster

---

## Phase 6: Production Features 🏭

Add features needed for production use.

### 6.1 Fault Tolerance
- [ ] Actor supervision trees
- [ ] Automatic actor restart
- [ ] Task retry mechanisms
- [ ] Cluster recovery from node failures

### 6.2 Performance
- [ ] Performance benchmarks
- [ ] Optimization pass
- [ ] Profiling and metrics
- [ ] Resource management

### 6.3 Observability
- [ ] Distributed tracing
- [ ] Metrics collection
- [ ] Logging framework
- [ ] Dashboard (stretch goal)

### 6.4 Advanced Features
- [ ] Actor scheduling policies
- [ ] Resource requirements/constraints
- [ ] Streaming/generator tasks
- [ ] GPU support (stretch goal)

**Deliverable**: Production-ready RustyRay

---

## Design Principles

1. **Incremental Development**: Each phase produces working code
2. **Learn from Ray**: Study Ray's implementation but adapt to Rust idioms
3. **Performance**: Leverage Rust's zero-cost abstractions
4. **Safety**: Use Rust's type system to prevent errors
5. **Simplicity**: Start simple, add complexity only when needed

## Success Metrics

- **Phase 1**: Can create actors and send messages ✅
- **Phase 2**: Can execute tasks with dependencies ✅
- **Phase 3**: API as simple as Ray's Python API
- **Phase 4**: Can share large objects efficiently
- **Phase 5**: Can run actors across multiple nodes
- **Phase 6**: Comparable performance to Ray for basic operations

## Open Questions

1. **Serialization**: Serde vs custom? Compatibility with Ray's protocol?
2. **Async Runtime**: Tokio vs async-std vs custom?
3. **Memory Model**: How closely to follow Plasma's design?
4. **Protocol Compatibility**: Should we aim for Ray protocol compatibility?
5. **Language Bindings**: Python bindings for RustyRay?

## Converting to GitHub Issues

Each numbered item in this roadmap can become a GitHub issue. For example:
- Issue #1: "Implement core types (ActorId, ActorHandle)"
- Issue #2: "Create Actor trait for user-defined actors"
- etc.

We can use GitHub Projects to track phases and milestones for releases.