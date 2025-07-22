# Changelog

All notable changes to RustyRay will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Phase 5.5: Development Infrastructure
  - GitHub Actions CI/CD pipeline with multi-platform testing
  - Code coverage reporting with llvm-cov
  - Security audit integration
  - Automated release workflow
  - Issue and PR templates
  - Comprehensive label strategy
  - Scripts for label and issue creation

### Fixed
- CI pipeline issues:
  - Cargo.toml version mismatches
  - Code formatting compliance with rustfmt
  - All clippy warnings resolved
  - Test isolation with --test-threads=1

## [0.5.0] - 2025-02-01

### Added
- Runtime architecture refactor for test isolation
- `with_test_runtime()` fixture for panic-safe test execution  
- `runtime::shutdown_async()` for graceful async shutdown
- Poisoned lock recovery throughout the codebase
- Comprehensive architecture documentation
- API reference documentation
- Architecture Decision Records (ADR) process
- Documentation strategy and organization

### Changed
- Runtime from `OnceCell<Runtime>` to `RwLock<Option<Arc<Runtime>>>`
- Runtime is now re-initializable after shutdown
- All tests updated to use `with_test_runtime()` fixture
- Documentation reorganized into clear categories

### Fixed
- `ray::put()` now correctly uses TaskSystem's put method
- ObjectRef resolution for task dependencies
- Concurrent test execution detection with clear errors
- Integration test to match current TaskManager behavior

### Performance
- 84 tests now passing (up from 78)
- No significant performance impact from refactor

## [0.4.5] - 2025-01-28

### Added
- Universal ObjectRef shared between actors and tasks
- Global `ray::put()` and `ray::get()` functions
- Batch operations: `ray::put_batch()` and `ray::get_batch()`
- `ray::wait()` for synchronization
- Actor examples using object store
- Comprehensive Ray comparison analysis

### Changed
- ObjectRef moved from task module to core level
- ObjectRef always backed by object store (removed channel mode)
- Task results automatically stored in object store
- Errors stored in object store with special markers

### Fixed
- Error propagation through ObjectRef
- Type serialization for object store
- 4 failing integration tests

## [0.4.0] - 2025-01-20

### Added
- Production-ready in-memory object store
- CLRU cache with strict memory limits
- Zero-copy access using `bytes::Bytes`
- Type-safe storage with runtime type checking
- Pinning support to prevent eviction
- Atomic statistics tracking
- 16 comprehensive object store tests

### Changed
- Replaced moka with CLRU for strict memory enforcement
- Object store integrated into Runtime

### Performance
- O(1) operations for all object store methods
- Eliminated O(n) bottleneck in pinned size tracking
- Zero-copy reads with `get_raw()`

## [0.3.0] - 2025-01-15

### Added
- `#[rustyray::remote]` macro for automatic function registration
- `#[rustyray::actor]` and `#[rustyray::actor_methods]` macros
- `#[rustyray::main]` for automatic runtime initialization
- Resource requirements (`num_cpus`, `num_gpus`) support
- Multiple actor constructor support
- Compile-time registration with linkme

### Changed
- 70% reduction in boilerplate code
- All examples updated to use macro API

### Performance
- Less than 5% overhead from macro usage
- Compile-time function registration

## [0.2.5] - 2025-01-10

### Fixed
- TaskBuilder error propagation with proper Result handling
- ObjectRef cloning using tokio::sync::watch
- Sleep() in tests replaced with proper synchronization

### Added
- Comprehensive cancellation/timeout system with TaskTracker
- Zero-copy optimization with bytes::Bytes
- Enhanced error context with `.context()` method

### Performance
- Optimized memory allocations
- Reduced serialization overhead

## [0.2.0] - 2025-01-05

### Added
- Complete task execution system
- ObjectRef<T> as typed futures
- Function registry with task_function! macro
- Task dependency tracking and resolution
- Integration between TaskSystem and ActorSystem
- Cancellation and timeout support

### Changed
- Unified error handling with Result types
- Comprehensive shutdown mechanisms

## [0.1.0] - 2024-12-20

### Added
- Initial actor system implementation
- Actor lifecycle management (start, stop, shutdown)
- Async message passing with request-response pattern
- Counter actor example
- Basic test suite

### Architecture
- ActorId and ActorRef types
- ActorSystem for managing actors
- Tokio-based async runtime

[Unreleased]: https://github.com/daveremy/rustyray/compare/v0.5.0...HEAD
[0.5.0]: https://github.com/daveremy/rustyray/compare/v0.4.5...v0.5.0
[0.4.5]: https://github.com/daveremy/rustyray/compare/v0.4.0...v0.4.5
[0.4.0]: https://github.com/daveremy/rustyray/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/daveremy/rustyray/compare/v0.2.5...v0.3.0
[0.2.5]: https://github.com/daveremy/rustyray/compare/v0.2.0...v0.2.5
[0.2.0]: https://github.com/daveremy/rustyray/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/daveremy/rustyray/releases/tag/v0.1.0