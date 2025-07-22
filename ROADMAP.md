# RustyRay Development Roadmap

This document outlines the development plan for RustyRay, a Rust implementation of Ray Core. Each phase builds on the previous one, allowing us to develop incrementally while learning Ray's architecture.

## Current Status

- **GitHub Repository**: https://github.com/daveremy/rustyray
- **Latest Version**: 0.5.0 (Phase 5 Complete)
- **Current Focus**: Phase 5.5 - Development Infrastructure & CI/CD
- **Next Release**: 0.5.5 (Development Infrastructure)

## Progress Summary

- **Phase 1: Local Actor System** ✅ Completed (v0.1.0)
  - Implemented core actor infrastructure with ActorId, ActorRef, and ActorSystem
  - Built async message passing using tokio channels
  - Added actor lifecycle management (start, stop, graceful shutdown)
  - Created working counter example demonstrating stateful actors
  - Achieved all primary deliverables for a working local actor system

- **Phase 2: Task Execution System** ✅ Completed (v0.2.0)
  - Implemented comprehensive task system with TaskManager and TaskSystem
  - Built ObjectRef with watch channels for multi-consumer futures
  - Added function registry with task_function! macro
  - Integrated task cancellation and timeout mechanisms
  - Fixed all critical issues from code review
  - Created multiple examples demonstrating task features

- **Phase 2.5: Code Review Improvements** ✅ Completed (v0.2.5)
  - Fixed TaskBuilder error propagation with proper Result handling
  - Implemented ObjectRef cloning with tokio::sync::watch for broadcast
  - Added comprehensive cancellation/timeout system with TaskTracker
  - Replaced sleep() in tests with proper synchronization primitives
  - Optimized memory allocations with bytes::Bytes for zero-copy
  - Received "Excellent" rating from Gemini comprehensive review
  - Published to GitHub with CI/CD pipeline

- **Phase 3: Macro System for API Ergonomics** ✅ Core Complete (Week 2 of 6)
  - Timeline: Started January 2025
  - Goal: Python-like simplicity with Rust's type safety
  - Status: 70% boilerplate reduction achieved! Moving to Phase 4 (remaining items deferred)

- **Phase 4: Local Shared Memory Object Store** ✅ Completed (v0.4.0)
  - Implemented production-ready object store with CLRU cache
  - Type-safe storage with runtime type checking
  - Zero-copy access patterns using bytes::Bytes
  - Comprehensive test coverage (16 tests)
  - Fixed all critical issues from Gemini review (Grade: A)
  - Ready for integration with actor system

- **Phase 4.5: Actor-Object Store Integration** ✅ Completed (v0.4.5)
  - Moved ObjectRef to core level (universal, not task-specific)
  - Implemented shared object store in Runtime
  - Fixed error propagation (errors stored in object store)
  - Created ray module with put/get API
  - Added actor examples using object store
  - All 60 tests passing (fixed 4 failing tests)
  - Comprehensive Ray comparison analysis completed

- **Phase 5: Runtime Architecture & Test Infrastructure** ✅ Completed (v0.5.0)
  - Refactored runtime from OnceCell to RwLock<Option<Arc<Runtime>>>
  - Made runtime re-initializable for proper test isolation
  - Created with_test_runtime() fixture with panic safety
  - Fixed ObjectRef resolution bug in ray::put()
  - Added graceful shutdown with explicit subsystem ordering
  - Implemented poisoned lock recovery for resilience
  - All 84 tests passing (up from 78)

- **Phase 5.5: Development Infrastructure & CI/CD** 🚀 In Progress
  - GitHub Actions CI/CD pipeline
  - Feature branch workflow with PR requirements
  - Issue templates and project management
  - Code quality gates (fmt, clippy, tests, coverage)
  - Release automation

## Overview

Based on our analysis of Ray's C++ implementation and comprehensive code review, we'll build RustyRay in phases:

1. **Local Actor System** ✅ (Completed)
2. **Task Execution System** ✅ (Completed) 
3. **Macro System for API Ergonomics** ✅ (Core Complete - 70% boilerplate reduction achieved!)
4. **Local Shared Memory Object Store** ✅ (Completed)
4.5. **Actor-Object Store Integration** ✅ (Completed)
5. **Runtime Architecture & Test Infrastructure** ✅ (Completed)
5.5. **Development Infrastructure & CI/CD** 🚀 (Current Focus)
6. **Garbage Collection & Production Features**
7. **Performance Optimizations**
8. **Distributed Foundation**
9. **Production Features**

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

## Phase 3: Macro System for API Ergonomics ✅ (Core Complete - Moving to Phase 4!)

Implement procedural macros to dramatically improve the developer experience. Goal: 70% reduction in boilerplate code.

### 3.1 Function Macros (Week 1-2) ✅ Week 1 Complete!
- [x] #[rustyray::remote] for automatic function registration
- [x] Support for both async and sync functions
- [x] Automatic serialization/deserialization handling
- [x] Type-safe function signatures with compile-time validation
- [x] ObjectRef<T> parameter support for task chaining
- [x] Resource requirements (num_cpus, num_gpus)
- [x] Result<T> return type detection and handling
- [x] Compile-time registration with linkme

### 3.2 Actor Macros (Week 2-3) ✅ Week 2 Complete! (Grade: A-)
- [x] #[rustyray::actor] for actor structs with resource requirements
- [x] #[rustyray::actor_methods] for typed method handles
- [x] Eliminate Box<dyn Any> boilerplate completely
- [x] Generate typed ActorHandle<T> automatically
- [x] Support for Result<T, E> return types with custom errors
- [x] Constructor pattern with Actor::remote(...)
- [x] Multiple constructor support (new→remote, others→remote_name)
- [x] Compile-time validations with clear error messages
- [x] Enhanced runtime error messages with context

### 3.3 Runtime Macros (Week 3-4) ✅ Core Complete
- [x] #[rustyray::main] for global runtime initialization (Week 2 bonus!)
- [x] Automatic TaskSystem/ActorSystem setup (via runtime::init)
- [x] Clean shutdown handling
- [x] Compile-time function registration with linkme (Week 1)
- [x] Enhanced error propagation with .context() (Week 2!)
- [ ] Global context improvements (deferred - not critical)

### 3.4 Polish & Release (Deferred to Post-Phase 4)
- [x] Update all examples to use macro API (Week 2!)
- [x] Enhanced error propagation with context (Week 2!)
- [ ] Performance benchmarks (<5% overhead target) - deferred
- [ ] Error message excellence with syn::Error::new_spanned - deferred
- [ ] Generic support (stretch goal) - deferred
- [ ] Beta release - after Phase 4
- [ ] Comprehensive rustdoc documentation - after API stabilization

**Success Metrics**:
- API simplicity: 70% less boilerplate
- Type safety: 100% compile-time validation
- Performance: <5% overhead vs manual
- Developer satisfaction: 90% positive feedback

**Deliverable**: Python-like simplicity with Rust's type safety (v0.3.0)

**Documentation Strategy**: Focus on examples and migration guide during development. 
Comprehensive rustdoc will be written after API stabilization based on user feedback.

---

## Phase 4: Local Shared Memory Object Store 📦 ✅ Completed (v0.4.0)

Implemented a production-ready object store inspired by Ray's Plasma store.

### 4.1 Object Management ✅
- [x] ObjectId generation with UUID v4
- [x] Object serialization/deserialization using bincode
- [x] Reference counting with Arc<AtomicUsize>
- [x] Garbage collection via LRU eviction with CLRU

### 4.2 Storage Backend ✅
- [x] In-memory store with CLRU cache (strict memory enforcement)
- [x] Zero-copy support using bytes::Bytes
- [x] LRU eviction policy with weight-based limits
- [x] Memory limits and quotas with atomic tracking
- [x] Pinning support to prevent eviction

### 4.3 Integration ✅
- [x] Type-safe put/get operations
- [x] Zero-copy reads with get_raw()
- [x] Thread-safe concurrent access
- [x] Comprehensive error handling

### 4.4 Production Features ✅
- [x] Atomic statistics collection
- [x] Race condition prevention
- [x] 16 comprehensive tests
- [x] Performance optimizations (O(1) operations)
- [x] Grade A from Gemini code review

**Key Achievements**:
- Replaced moka with CLRU for strict memory enforcement (no 75% overruns)
- Implemented atomic pinned size tracking (eliminated O(n) bottleneck)
- Type-safe storage with runtime type checking
- Zero-copy access patterns for efficiency
- Thread-safe with proper locking order
- Comprehensive test coverage including race conditions

**Deliverable**: Production-ready local object store with examples ✅

---

## Phase 4.5: Actor-Object Store Integration ✅ (Completed)

Successfully bridged the gap between actors, tasks, and the object store for seamless data sharing.

### 4.5.1 Shared Object Store ✅
- [x] Add object_store to Runtime as shared instance
- [x] Update TaskSystem to use external store
- [x] Remove local channel mode from ObjectRef
- [x] Ensure all ObjectRefs are store-backed

### 4.5.2 Architectural Refactoring ✅
- [x] Moved ObjectRef from task module to core level
- [x] Simplified ObjectRef to always be store-backed
- [x] Fixed type storage issue with type-erased approach
- [x] Task results automatically stored via put_with_id

### 4.5.3 Unified API ✅
- [x] Created ray module with global put/get functions
- [x] Implemented batch operations (put_batch/get_batch)
- [x] Added wait() for synchronization
- [x] Maintained type safety throughout

### 4.5.4 Error Handling Enhancement ✅
- [x] Fixed error propagation through ObjectRef
- [x] Errors stored in object store with special markers
- [x] All 60 tests passing (fixed 4 failing tests)

### 4.5.5 Examples & Testing ✅
- [x] Updated object_store_demo.rs with ray API
- [x] Created actor_object_sharing.rs example
- [x] Added comprehensive integration tests
- [x] Documented new architecture

**Key Achievement**: Universal ObjectRef with seamless actor-task data sharing
**Deliverable**: Unified object store accessible from both actors and tasks ✅

---

## Phase 5: Runtime Architecture & Test Infrastructure ✅ (Completed)

Major refactoring of the runtime architecture to fix concurrent test execution issues while maintaining Ray-compatible design.

### 5.1 Runtime Architecture Refactor ✅
- [x] Changed from OnceCell to RwLock<Option<Arc<Runtime>>>
- [x] Made runtime re-initializable after shutdown
- [x] Maintained thread safety with proper lifecycle
- [x] Added runtime::shutdown_async() for graceful shutdown

### 5.2 Test Infrastructure ✅
- [x] Created with_test_runtime() fixture with panic safety
- [x] Used futures::FutureExt::catch_unwind for isolation
- [x] Migrated all tests to new fixture pattern
- [x] Added concurrent test detection with clear errors

### 5.3 Critical Bug Fixes ✅
- [x] Fixed ray::put() to use TaskSystem for dependencies
- [x] Updated integration tests for current behavior
- [x] Added proper subsystem shutdown ordering
- [x] Implemented poisoned lock recovery

### 5.4 Code Quality ✅
- [x] Comprehensive code review with multiple agents
- [x] Fixed all high-priority issues
- [x] 84 tests passing (up from 78)
- [x] Documentation of architecture decisions

**Deliverable**: Robust runtime with proper test isolation

---

## Phase 5.5: Development Infrastructure & CI/CD 🔧 (In Progress)

Establish professional development practices before implementing production features.

### 5.5.1 GitHub Actions CI/CD ✅
- [x] Multi-platform test matrix (Linux, macOS, Windows)
- [x] Automated formatting and linting checks
- [x] Code coverage reporting
- [x] Security audit integration
- [x] Release automation workflow

### 5.5.2 Development Workflow
- [ ] Feature branch protection rules
- [ ] PR required for all changes
- [ ] Required status checks
- [x] Issue templates (bug, feature, task)
- [x] PR template with checklist

### 5.5.3 Project Management
- [ ] Convert roadmap items to GitHub issues
- [ ] Create project board for tracking
- [ ] Establish labels and milestones
- [ ] Link PRs to issues
- [ ] Set up automation (auto-assign, auto-label)

### 5.5.4 Developer Experience
- [ ] Pre-commit hooks for local checks
- [ ] Development setup scripts
- [ ] Benchmark baseline and tracking
- [ ] Documentation for new workflow
- [ ] Update CONTRIBUTING.md

**Deliverable**: Professional CI/CD pipeline with issue-driven development

---

## Phase 6: Garbage Collection & Production Features 🏭

Based on Phase 5 code review feedback, implement critical production features starting with garbage collection.

### 6.1 Reference Counting & Garbage Collection (High Priority)
- [ ] Add reference counting to ObjectRef lifecycle (new/drop)
- [ ] Implement atomic reference counters in object store
- [ ] Track ObjectRefs passed to tasks and actors
- [ ] Periodic cleanup of orphaned objects
- [ ] Memory pressure handling and eviction policies

### 6.2 Production Monitoring (High Priority)
- [ ] Metrics collection (operations, latency, memory usage)
- [ ] Health check endpoints for runtime status
- [ ] Performance monitoring with configurable thresholds
- [ ] Integration with standard monitoring tools (Prometheus)

### 6.3 Enhanced Error Handling (Medium Priority)
- [ ] Retry mechanisms for transient failures
- [ ] Richer error context (task ID, actor ID, stack traces)
- [ ] Error recovery strategies for different failure types
- [ ] Create RayErrorType enum matching Ray's error types

### 6.4 Resource Management (Medium Priority)
- [ ] Memory limits and backpressure mechanisms
- [ ] CPU/GPU resource tracking and limits
- [ ] Graceful degradation under resource pressure
- [ ] Work queue limits and overflow handling

### 6.5 Performance Optimizations (Low Priority)
- [ ] Replace std::sync with parking_lot for better performance
- [ ] Cache runtime references to reduce lock contention
- [ ] Profile and optimize hot paths
- [ ] Benchmark suite for regression testing

### 6.6 Deferred Phase 3 Items
- [ ] Performance benchmarks for macro overhead (<5% target)
- [ ] Enhanced error messages with syn::Error::new_spanned
- [ ] Generic support for remote functions
- [ ] Comprehensive rustdoc documentation

**Deliverable**: Production-ready runtime with GC, monitoring, and resource management

---

## Phase 7: Performance Optimizations ⚡

Implement key performance optimizations identified from Ray's architecture.

### 7.1 Size-Based Object Routing
- [ ] Implement threshold-based routing (100KB default)
- [ ] Inline storage for small objects
- [ ] Direct transport for medium objects
- [ ] Shared memory preparation for large objects

### 7.2 Zero-Copy Optimizations
- [ ] Implement zero-copy serialization where possible
- [ ] Use memory mapping for large objects
- [ ] Optimize actor-to-actor communication
- [ ] Reduce unnecessary copies in hot paths

### 7.3 Batch Operations
- [ ] Optimize batch put/get operations
- [ ] Implement vectorized operations
- [ ] Reduce lock contention for batch ops
- [ ] Pipeline network operations

### 7.4 Benchmarking Suite
- [ ] Create comprehensive benchmarks
- [ ] Compare with Ray performance
- [ ] Identify bottlenecks
- [ ] Track performance regressions

### 7.5 Early Performance Baseline
- [ ] Basic benchmark suite for Phases 5-6 changes
- [ ] Memory allocation profiling
- [ ] Lock contention analysis
- [ ] CI integration for regression detection

### 7.6 Configuration Management
- [ ] Design configuration system for memory limits
- [ ] Worker pool configuration
- [ ] Network settings for future phases
- [ ] Environment variable support

**Deliverable**: 10x performance improvement for common operations

---

## Phase 8: Distributed Foundation 🌐

Prepare RustyRay for distributed operation by implementing core distributed systems primitives.

### 8.1 Node Management
- [ ] Node ID generation and tracking
- [ ] Heartbeat mechanism
- [ ] Failure detection
- [ ] Node state management

### 8.2 Object Location Service
- [ ] Track object locations across nodes
- [ ] Implement object directory
- [ ] Support object migration
- [ ] Location-aware routing

### 8.3 Network Transport
- [ ] gRPC service definitions
- [ ] Efficient serialization for network
- [ ] Connection pooling
- [ ] Backpressure handling

### 8.4 Distributed Primitives
- [ ] Distributed reference counting protocol
- [ ] Cross-node object transfer
- [ ] Eventual consistency guarantees
- [ ] Basic fault tolerance

**Deliverable**: Foundation for multi-node operation

---

## Phase 9: Production Features 🏭

Complete RustyRay with production-ready features and polish.

### 9.1 Fault Tolerance
- [ ] Actor supervision trees
- [ ] Automatic actor restart with backoff
- [ ] Task retry mechanisms
- [ ] Cluster recovery from node failures
- [ ] Lineage-based reconstruction

### 9.2 Advanced Features
- [ ] Actor scheduling policies
- [ ] Resource requirements/constraints
- [ ] Streaming/generator tasks
- [ ] GPU support
- [ ] Autoscaling support

### 9.3 Observability
- [ ] Distributed tracing (OpenTelemetry)
- [ ] Metrics collection (Prometheus)
- [ ] Structured logging
- [ ] Dashboard and monitoring tools
- [ ] Performance profiling

### 9.4 Production Hardening
- [ ] Security features (TLS, auth)
- [ ] Rate limiting and quotas
- [ ] Multi-tenancy support
- [ ] Deployment tools and documentation
- [ ] Kubernetes integration

**Deliverable**: Production-ready distributed computing framework

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
- **Phase 2.5**: All code review issues addressed, "Excellent" rating achieved ✅
- **Phase 3**: API as simple as Ray's Python API (70% less boilerplate) ✅
- **Phase 4**: Can share large objects efficiently (<10ms latency) ✅
- **Phase 4.5**: Unified object store with actor-task sharing ✅
- **Phase 5**: Robust runtime with proper test isolation ✅
- **Phase 5.5**: CI/CD pipeline active, PR-based workflow established
- **Phase 6**: No memory leaks, production monitoring, resource limits
- **Phase 7**: 10x performance improvement on benchmarks
- **Phase 8**: Can transfer objects between nodes
- **Phase 9**: Production-ready with <1% failure rate

## Timeline Estimates

- **Phase 3**: 2 weeks (January 2025) ✅ Completed
- **Phase 4**: 2 weeks (January 2025) ✅ Completed  
- **Phase 4.5**: 1 week (February 2025) ✅ Completed
- **Phase 5**: 1 week (February 2025) ✅ Completed - Runtime Architecture
- **Phase 5.5**: 1 week (February 2025) 🚀 Current - Development Infrastructure
- **Phase 6**: 2 weeks (February-March 2025) - GC & Production Features
- **Phase 7**: 2 weeks (March 2025) - Performance
- **Phase 8**: 3 weeks (March-April 2025) - Distributed Foundation
- **Phase 9**: 4 weeks (April 2025) - Production Features
- **Version 1.0**: Q2 2025

## Technical Decisions Made

1. **Serialization**: Using bincode for efficiency, Arrow for future cross-language
2. **Async Runtime**: Committed to tokio for ecosystem compatibility
3. **Memory Model**: CLRU for strict memory limits, shared memory planned
4. **Error Handling**: Simple strings now, typed errors in Phase 6
5. **Testing**: Comprehensive unit and integration tests with CI/CD
6. **ObjectRef**: Universal type at core level, not task-specific
7. **Object Store**: Single shared instance in Runtime
8. **Type Safety**: Type parameter T in ObjectRef, runtime checking in store
9. **Error Storage**: Errors stored in object store with special markers
10. **API Design**: Ray-compatible put/get with Rust type safety
11. **Runtime Architecture**: Re-initializable with RwLock<Option<Arc<Runtime>>>
12. **Test Infrastructure**: with_test_runtime() fixture for isolation
13. **Lock Handling**: Graceful poisoned lock recovery

## Open Questions

1. **Reference Counting**: Owner-based (like Ray) or fully distributed?
2. **Shared Memory**: When to add Plasma-style shared memory store?
3. **Cross-Language**: Priority for Python interop via Arrow?
4. **Error Compatibility**: Match Ray's error types exactly or innovate?
5. **Performance Target**: Match Ray or leverage Rust for better performance?
6. **Protocol Compatibility**: Binary compatible with Ray or Rust-native?
7. **Cluster Management**: Kubernetes-native or custom orchestration?
8. **Monitoring Integration**: OpenTelemetry, Prometheus, or custom metrics?
9. **Lock Library**: Switch to parking_lot for performance?
10. **Test Concurrency**: Accept single-threaded tests or find alternative?

## Converting to GitHub Issues

Each numbered item in this roadmap can become a GitHub issue. For example:
- Issue #1: "Implement core types (ActorId, ActorHandle)"
- Issue #2: "Create Actor trait for user-defined actors"
- etc.

We can use GitHub Projects to track phases and milestones for releases.