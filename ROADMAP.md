# RustyRay Development Roadmap

This document outlines the development plan for RustyRay, a Rust implementation of Ray Core. Each phase builds on the previous one, allowing us to develop incrementally while learning Ray's architecture.

## Current Status

- **GitHub Repository**: https://github.com/daveremy/rustyray
- **Latest Version**: 0.4.5 (Phase 4.5 Complete)
- **Current Focus**: Phase 5 - Reference Counting & Memory Management
- **Next Release**: 0.5.0 (Reference Counting)

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

## Overview

Based on our analysis of Ray's C++ implementation and comprehensive code review, we'll build RustyRay in phases:

1. **Local Actor System** ✅ (Completed)
2. **Task Execution System** ✅ (Completed) 
3. **Macro System for API Ergonomics** ✅ (Core Complete - 70% boilerplate reduction achieved!)
4. **Local Shared Memory Object Store** ✅ (Completed)
4.5. **Actor-Object Store Integration** ✅ (Completed)
5. **Reference Counting & Memory Management** 🚀 (Current Focus)
6. **Metadata & Error Enhancement**
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

## Phase 5: Reference Counting & Memory Management 🧮 (Current Focus)

Implement distributed reference counting to enable safe memory management and prevent premature object eviction.

### 5.1 Local Reference Counting
- [ ] Add reference counting to ObjectRef lifecycle (new/drop)
- [ ] Implement atomic reference counters in object store
- [ ] Track local references within a worker
- [ ] Update ObjectRef Clone to increment count

### 5.2 Task Reference Tracking
- [ ] Track ObjectRefs passed to tasks
- [ ] Increment count when task submitted
- [ ] Decrement when task completes
- [ ] Handle task failure cases

### 5.3 Eviction Safety
- [ ] Modify LRU eviction to check reference counts
- [ ] Only evict objects with zero references
- [ ] Add eviction callbacks for cleanup
- [ ] Implement memory pressure handling

### 5.4 Metrics & Debugging
- [ ] Add reference count to object store statistics
- [ ] Debug logging for reference count changes
- [ ] Memory leak detection tools
- [ ] Performance impact benchmarks

### 5.5 Documentation
- [ ] API documentation for reference counting behavior
- [ ] Migration guide for existing code
- [ ] Best practices for avoiding leaks

**Deliverable**: Safe automatic memory management with reference counting

---

## Phase 6: Metadata & Error Enhancement 📋

Enhance object metadata and error handling to match Ray's production capabilities.

### 6.1 Structured Metadata
- [ ] Separate data and metadata buffers in RayObject
- [ ] Add serialization format tracking
- [ ] Include original type information
- [ ] Support for cross-language metadata

### 6.2 Comprehensive Error Types
- [ ] Create RayErrorType enum matching Ray's 13 types
- [ ] Implement error objects with proper metadata
- [ ] Add stack trace capture for errors
- [ ] Improve error messages with context

### 6.3 Object Metadata
- [ ] Track object creation time
- [ ] Add object size metadata
- [ ] Include task/actor origin information
- [ ] Support nested object references

### 6.4 Testing & Migration
- [ ] Update all error handling code
- [ ] Migrate from string errors to typed errors
- [ ] Add error propagation tests
- [ ] Document error handling patterns

### 6.5 Deferred Phase 3 Items
- [ ] Performance benchmarks for macro overhead (<5% target)
- [ ] Enhanced error messages with syn::Error::new_spanned
- [ ] Generic support for remote functions
- [ ] Comprehensive rustdoc documentation

### 6.6 API Documentation
- [ ] Document metadata format and usage
- [ ] Error handling best practices
- [ ] Migration guide from Phase 4.5

**Deliverable**: Production-quality metadata and error handling

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
- **Phase 5**: No memory leaks, safe automatic eviction
- **Phase 6**: Rich error types and metadata support
- **Phase 7**: 10x performance improvement on benchmarks
- **Phase 8**: Can transfer objects between nodes
- **Phase 9**: Production-ready with <1% failure rate

## Timeline Estimates

- **Phase 3**: 2 weeks (January 2025) ✅ Completed
- **Phase 4**: 2 weeks (January 2025) ✅ Completed  
- **Phase 4.5**: 1 week (February 2025) ✅ Completed
- **Phase 5**: 1 week (February 2025) 🚀 Current - Reference Counting
- **Phase 6**: 1 week (February 2025) - Metadata Enhancement
- **Phase 7**: 2 weeks (February 2025) - Performance
- **Phase 8**: 3 weeks (March 2025) - Distributed Foundation
- **Phase 9**: 4 weeks (March-April 2025) - Production Features
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

## Open Questions

1. **Reference Counting**: Owner-based (like Ray) or fully distributed?
2. **Shared Memory**: When to add Plasma-style shared memory store?
3. **Cross-Language**: Priority for Python interop via Arrow?
4. **Error Compatibility**: Match Ray's error types exactly or innovate?
5. **Performance Target**: Match Ray or leverage Rust for better performance?
6. **Protocol Compatibility**: Binary compatible with Ray or Rust-native?
7. **Cluster Management**: Kubernetes-native or custom orchestration?

## Converting to GitHub Issues

Each numbered item in this roadmap can become a GitHub issue. For example:
- Issue #1: "Implement core types (ActorId, ActorHandle)"
- Issue #2: "Create Actor trait for user-defined actors"
- etc.

We can use GitHub Projects to track phases and milestones for releases.