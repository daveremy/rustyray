# RustyRay Phase 3: Final Implementation Plan

## Overview

Based on Gemini's positive assessment and recommendations, this is the final plan for implementing the macro system in Phase 3. The plan incorporates all feedback and prioritizes developer experience.

## Key Objectives

1. **Dramatically simplify the API** to match Ray Python's ergonomics
2. **Maintain full type safety** with compile-time validation
3. **Provide excellent error messages** for macro users
4. **Support both async and sync** functions
5. **Enable incremental adoption** with backward compatibility

## Implementation Schedule

### Week 1: Foundation & Infrastructure

#### Day 1-2: Project Setup
- Create workspace structure with `rustyray-macros` crate
- Move existing code to `rustyray-core`
- Setup dependencies (syn, quote, proc-macro2, linkme)
- Configure CI for workspace

#### Day 3-4: Basic Macro Framework
- Create error handling utilities with `syn::Error::new_spanned`
- Setup shared parsing utilities
- Implement basic attribute parsing
- Create test infrastructure with `trybuild`

#### Day 5: Testing Infrastructure
- Setup comprehensive `trybuild` test cases
- Create test fixtures for success/failure cases
- Document testing patterns for contributors

### Week 2: Remote Function Macro

#### Day 1-2: Basic #[remote] Implementation
```rust
#[rustyray::remote]
async fn add(x: i32, y: i32) -> i32 {
    x + y
}
```
- Parse async functions
- Generate module with `remote()` function
- Handle basic serializable types

#### Day 3: Sync Function Support
```rust
#[rustyray::remote]
fn compute(x: i32) -> i32 {  // Non-async
    x * 2
}
```
- Detect sync vs async functions
- Generate appropriate wrappers
- Test thread pool execution

#### Day 4: ObjectRef Arguments
```rust
#[rustyray::remote]
async fn chain(prev: ObjectRef<i32>, y: i32) -> i32 {
    prev.get().await? + y
}
```
- Detect ObjectRef<T> parameters
- Generate proper dependency handling
- Test chaining patterns

#### Day 5: Resource Requirements
```rust
#[rustyray::remote(num_cpus = 2.0, num_gpus = 0.5)]
async fn train_model(data: Vec<f64>) -> Model {
    // ...
}
```
- Parse macro attributes
- Store resource requirements
- Integration with task system

### Week 3: Actor Macros

#### Day 1-2: Basic #[actor] Implementation
```rust
#[rustyray::actor]
pub struct Counter {
    value: i32,
}
```
- Generate typed handle struct
- Add serialization derives
- Create message/response enums

#### Day 3-4: #[actor_methods] Implementation
```rust
#[rustyray::actor_methods]
impl Counter {
    pub fn new(initial: i32) -> Self {
        Counter { value: initial }
    }
    
    pub async fn increment(&mut self) -> i32 {
        self.value += 1;
        self.value
    }
}
```
- Parse impl blocks
- Generate typed handle methods
- Handle constructor pattern
- Support both sync and async methods

#### Day 5: Actor Resources & Error Handling
```rust
#[rustyray::actor(num_cpus = 2.0)]
pub struct HeavyActor { }

#[rustyray::actor_methods]
impl DataProcessor {
    pub async fn process(&mut self, data: Vec<i32>) -> Result<Output, MyError> {
        // ...
    }
}
```
- Actor resource requirements
- Custom error type handling
- Result<T, E> in ObjectRef

### Week 4: Runtime & Integration

#### Day 1-2: Global Runtime
```rust
// src/runtime.rs
pub struct Runtime {
    actor_system: Arc<ActorSystem>,
    task_system: Arc<TaskSystem>,
}

impl Runtime {
    pub fn init() -> Result<()>;
    pub fn global() -> Result<&'static Runtime>;
    pub fn shutdown() -> Result<()>;
}
```

#### Day 3: #[rustyray::main] Macro
```rust
#[rustyray::main]
async fn main() -> Result<()> {
    // Auto-initialize runtime
    // Register all functions
    // Run async main
    // Auto-shutdown
}
```

#### Day 4: Prelude & Exports
```rust
// rustyray::prelude
pub use crate::{remote, actor, actor_methods, main};
pub use crate::{ObjectRef, Result, RustyRayError};
pub use crate::{get_all, put};
```

#### Day 5: Integration Testing
- Update all examples to use macros
- Performance benchmarks
- Migration guide

### Week 5: Advanced Features & Polish

#### Day 1: Generic Support (Stretch Goal)
```rust
#[rustyray::remote]
async fn process<T: Serialize + DeserializeOwned>(data: T) -> T {
    data
}

#[rustyray::actor]
pub struct GenericActor<T> {
    items: Vec<T>,
}
```

#### Day 2-3: Error Message Excellence
- Comprehensive error messages with spans
- Helpful suggestions for common mistakes
- Clear documentation links

#### Day 4: Performance Optimization
- Minimize macro expansion time
- Optimize generated code
- Benchmark vs manual implementation

#### Day 5: Documentation
- Macro API documentation
- Migration guide from manual API
- Best practices guide

### Week 6: Testing & Release

#### Day 1-2: Comprehensive Testing
- All examples converted
- Edge case testing
- Backward compatibility verification

#### Day 3: Community Preview
- Beta release (0.3.0-beta.1)
- Gather feedback
- Address critical issues

#### Day 4: Final Polish
- Address beta feedback
- Update documentation
- Performance validation

#### Day 5: Release
- Version 0.3.0 release
- Announcement blog post
- Update Ray community

## Testing Strategy

### 1. Compile-Time Tests (trybuild)
```rust
// tests/ui/remote_pass.rs
#[rustyray::remote]
async fn valid_function(x: i32) -> i32 { x }

// tests/ui/remote_fail.rs
#[rustyray::remote]
async fn invalid_function(x: NonSerializable) -> i32 { 0 }
// Should fail with clear error about NonSerializable
```

### 2. Integration Tests
- Full actor lifecycle tests
- Task chaining tests
- Error propagation tests
- Resource requirement tests

### 3. Performance Tests
- Macro expansion time
- Runtime overhead vs manual
- Memory usage comparison

## Error Message Standards

Every macro error must:
1. Point to the exact problematic code with span
2. Explain what's wrong in simple terms
3. Suggest how to fix it
4. Link to relevant documentation

Example:
```
error: ObjectRef<T> arguments must come before regular arguments
  --> src/main.rs:4:34
   |
4  | async fn bad_order(x: i32, ref: ObjectRef<i32>) -> i32 {
   |                            ^^^ ObjectRef argument here
   |
   = help: move this argument before 'x'
   = note: ObjectRef arguments represent task dependencies and must be resolved first
   = see: https://rustyray.dev/docs/macros#objectref-arguments
```

## Success Metrics

1. **API Simplicity**: 70% reduction in boilerplate code
2. **Type Safety**: Zero runtime type errors
3. **Performance**: <5% overhead vs manual implementation
4. **Developer Experience**: 90% positive feedback in beta
5. **Documentation**: 100% macro coverage with examples

## Risk Management

### High Risk: Macro Complexity
- **Mitigation**: Start simple, add features incrementally
- **Mitigation**: Extensive testing with trybuild
- **Mitigation**: Code review by Rust macro experts

### Medium Risk: Compile Time Impact
- **Mitigation**: Profile macro expansion
- **Mitigation**: Optimize hot paths
- **Mitigation**: Provide precompiled macro crate

### Low Risk: Breaking Changes
- **Mitigation**: Feature flags for new API
- **Mitigation**: Deprecation warnings
- **Mitigation**: Migration tool (stretch goal)

## Next Steps

1. Create `rustyray-macros` crate
2. Setup workspace configuration
3. Implement basic #[remote] for async functions
4. Create first trybuild tests
5. Iterate based on test results

This plan incorporates all of Gemini's feedback while maintaining an aggressive but achievable timeline. The focus on developer experience, especially error messages, will be critical to adoption success.