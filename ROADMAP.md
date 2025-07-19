# RustyRay Development Roadmap

This document outlines the development plan for RustyRay, a Rust implementation of Ray Core. Each phase builds on the previous one, allowing us to develop incrementally while learning Ray's architecture.

## Progress Summary

- **Phase 1: Local Actor System** ✅ Completed
  - Implemented core actor infrastructure with ActorId, ActorRef, and ActorSystem
  - Built async message passing using tokio channels
  - Added actor lifecycle management (start, stop, graceful shutdown)
  - Created working counter example demonstrating stateful actors
  - Achieved all primary deliverables for a working local actor system

- **Phase 2: Task Execution System** 🚀 Current Focus
  - Next step: Design and implement stateless task execution
  - Will integrate with existing actor system

## Overview

Based on our analysis of Ray's C++ implementation, we'll build RustyRay in phases:

1. **Local Actor System** ✅ (Completed)
2. **Task Execution System** (Current Focus)
3. **Object Store**
4. **Distributed Runtime**
5. **Production Features**

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
- [ ] Define Task and TaskId types
- [ ] Create TaskSpec for task definitions
- [ ] Implement TaskManager
- [ ] Add future/promise primitives

### 2.2 Task Scheduling
- [ ] Local task queue
- [ ] Worker pool for task execution
- [ ] Task dependency tracking
- [ ] Result handling and caching

### 2.3 Integration with Actors
- [ ] Tasks can create actors
- [ ] Tasks can call actor methods
- [ ] Actors can submit tasks
- [ ] Unified error handling

**Deliverable**: Tasks and actors working together locally

---

## Phase 3: Object Store 📦

Implement a simplified version of Plasma for efficient data sharing.

### 3.1 Object Management
- [ ] ObjectId generation
- [ ] Object serialization/deserialization
- [ ] Reference counting
- [ ] Basic garbage collection

### 3.2 Storage Backend
- [ ] In-memory store (HashMap-based)
- [ ] Shared memory support (using memmap2)
- [ ] Object eviction policies
- [ ] Memory limits and quotas

### 3.3 Integration
- [ ] Actors can put/get objects
- [ ] Tasks automatically store results
- [ ] Zero-copy reads where possible
- [ ] Object transfer between actors

**Deliverable**: Local object store with examples

---

## Phase 4: Distributed Runtime 🌐

Transform RustyRay into a true distributed system.

### 4.1 Networking Layer
- [ ] gRPC service definitions (using tonic)
- [ ] Node discovery and registration
- [ ] Heartbeat and failure detection
- [ ] Message routing between nodes

### 4.2 Global Control Store (GCS)
- [ ] Actor registry (global actor table)
- [ ] Node information table
- [ ] Object location table
- [ ] Distributed metadata management

### 4.3 Distributed Actors
- [ ] Remote actor creation
- [ ] Cross-node message passing
- [ ] Actor migration (stretch goal)
- [ ] Location transparency

### 4.4 Distributed Tasks
- [ ] Task scheduling across nodes
- [ ] Load balancing
- [ ] Data locality awareness
- [ ] Distributed dependency resolution

**Deliverable**: Multi-node RustyRay cluster

---

## Phase 5: Production Features 🏭

Add features needed for production use.

### 5.1 Fault Tolerance
- [ ] Actor supervision trees
- [ ] Automatic actor restart
- [ ] Task retry mechanisms
- [ ] Cluster recovery from node failures

### 5.2 Performance
- [ ] Performance benchmarks
- [ ] Optimization pass
- [ ] Profiling and metrics
- [ ] Resource management

### 5.3 Observability
- [ ] Distributed tracing
- [ ] Metrics collection
- [ ] Logging framework
- [ ] Dashboard (stretch goal)

### 5.4 Advanced Features
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

- **Phase 1**: Can create actors and send messages
- **Phase 2**: Can execute tasks with dependencies
- **Phase 3**: Can share large objects efficiently
- **Phase 4**: Can run actors across multiple nodes
- **Phase 5**: Comparable performance to Ray for basic operations

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