# RustyRay Phase 3: Macro System Implementation Plan

## Executive Summary

Phase 3 will introduce a procedural macro system that dramatically improves RustyRay's developer experience. The goal is to achieve Python-like simplicity while maintaining Rust's type safety and performance.

## Current State

After completing Phase 2.5, we have:
- ✅ Robust actor system with typed handles
- ✅ Task execution with ObjectRef futures
- ✅ Function registry with task_function! macro
- ✅ Cancellation and timeout support
- ✅ Clean separation of concerns

Current pain points:
- Verbose function registration
- Box<dyn Any> boilerplate in actors
- Manual task submission process
- No compile-time function name validation

## Phase 3 Goals

1. **Zero boilerplate** for remote functions
2. **Type-safe** actor methods with no Box<dyn Any>
3. **Compile-time** function registration
4. **Global runtime** management
5. **Python-like** API simplicity

## Implementation Plan

### Week 1: Project Structure & Foundation

#### 1.1 Create Macro Crate Structure
```bash
# New structure
rustyray/
├── rustyray-macros/        # Procedural macros
├── rustyray-core/          # Core implementation (current src/)
├── src/                    # Facade with prelude
└── examples/
    └── macro_examples/     # New macro-based examples
```

#### 1.2 Dependencies & Setup
- Add `syn`, `quote`, `proc-macro2` for macro development
- Add `linkme` for compile-time registration
- Add `once_cell` for global runtime
- Setup workspace configuration

#### 1.3 Basic Infrastructure
- Create macro entry points
- Setup error handling for macros
- Create shared utilities for code generation

### Week 2: Remote Function Macro (#[remote])

#### 2.1 Basic Function Support
```rust
#[rustyray::remote]
async fn add(x: i32, y: i32) -> i32 {
    x + y
}

// Generates:
// - add::remote(x, y) -> ObjectRef<i32>
// - Automatic registration with function registry
```

#### 2.2 Resource Requirements
```rust
#[rustyray::remote(num_cpus = 2.0, num_gpus = 1.0)]
async fn train_model(data: Vec<f64>) -> Model {
    // ...
}
```

#### 2.3 ObjectRef Arguments
- Support ObjectRef<T> parameters
- Automatic dependency tracking
- Type checking at compile time

#### 2.4 Testing
- Unit tests for macro parsing
- Integration tests with task system
- Compile-fail tests for invalid usage

### Week 3: Actor Macros (#[actor] & #[actor_methods])

#### 3.1 Actor Struct Transformation
```rust
#[rustyray::actor]
pub struct Counter {
    value: i32,
}

// Generates:
// - Serialization derives
// - CounterHandle type
// - Message/Response enums
```

#### 3.2 Method Registration
```rust
#[rustyray::actor_methods]
impl Counter {
    pub async fn increment(&mut self) -> i32 {
        self.value += 1;
        self.value
    }
}

// Generates typed handle methods:
// handle.increment() -> ObjectRef<i32>
```

#### 3.3 Constructor Support
- Special handling for `new` methods
- Actor::remote(...) creation pattern
- Builder pattern for complex actors

#### 3.4 Error Handling
- Support Result<T, E> returns
- Custom error types
- Proper error propagation

### Week 4: Runtime Management

#### 4.1 Global Runtime (#[rustyray::main])
```rust
#[rustyray::main]
async fn main() -> Result<()> {
    // Automatic runtime initialization
    // Function registration
    // Graceful shutdown
}
```

#### 4.2 Runtime Module
- Global TaskSystem/ActorSystem access
- Thread-safe initialization
- Resource cleanup on shutdown

#### 4.3 Prelude & Exports
- Common types and traits
- Convenience functions (get_all, put)
- Error types

### Week 5: Advanced Features & Polish

#### 5.1 Performance Optimizations
- Zero-cost abstractions validation
- Compile-time optimizations
- Benchmark suite

#### 5.2 Error Messages
- Clear macro error messages
- Helpful suggestions
- Span preservation

#### 5.3 Documentation
- Macro documentation
- Migration guide
- Best practices

#### 5.4 Backward Compatibility
- Feature flags for gradual adoption
- Deprecation warnings
- Migration tooling

### Week 6: Testing & Release

#### 6.1 Comprehensive Testing
- All examples using new API
- Performance benchmarks
- Edge case testing

#### 6.2 Documentation
- API documentation
- Tutorial series
- Video demonstrations

#### 6.3 Release Preparation
- Version 0.3.0 planning
- Breaking change documentation
- Community feedback

## Technical Decisions

### 1. Compile-time Registration
Use `linkme` for zero-runtime-cost function registration:
```rust
#[linkme::distributed_slice(REMOTE_FUNCTIONS)]
static ADD_FUNCTION: RemoteFunction = ...;
```

### 2. Type Safety
Generate strongly-typed code without Any:
- Actor message enums
- Typed handles
- Compile-time validation

### 3. Error Handling
- Preserve user error types
- Clear error messages
- No hidden panics

### 4. Performance
- No runtime overhead vs manual implementation
- Macro expansion caching
- Minimal dependencies

## Migration Strategy

### Phase 1: Parallel APIs (Week 4)
- New macro API alongside existing
- Examples showing both approaches
- Performance comparisons

### Phase 2: Deprecation (Week 5)
- Mark old APIs as deprecated
- Provide migration guide
- Automated migration tool (stretch)

### Phase 3: Removal (Future)
- Remove deprecated APIs in 0.4.0
- Clean up internal code
- Simplify documentation

## Success Metrics

1. **Code Reduction**: 50%+ less boilerplate
2. **Type Safety**: 100% compile-time checked
3. **Performance**: No regression vs manual
4. **Adoption**: Positive user feedback
5. **Documentation**: Complete coverage

## Risk Mitigation

### Complexity Risk
- Start with simple cases
- Incremental feature addition
- Extensive testing

### Performance Risk
- Continuous benchmarking
- Profile macro expansion
- Optimize hot paths

### Usability Risk
- User testing early
- Clear error messages
- Comprehensive docs

## Example Transformation

### Before (Current API):
```rust
// Function registration
let task_system = TaskSystem::new();
task_system.register_function(
    "add",
    task_function!(|x: i32, y: i32| async move {
        Ok::<i32, RustyRayError>(x + y)
    })
)?;

// Task submission
let result = TaskBuilder::new("add")
    .arg(5)
    .arg(3)
    .submit::<i32>(&task_system)
    .await?;

// Actor implementation
impl Actor for Counter {
    async fn handle(&mut self, msg: Box<dyn Any + Send>) -> Result<Box<dyn Any + Send>> {
        // Manual downcasting and dispatching
    }
}
```

### After (Phase 3 API):
```rust
// Function definition
#[rustyray::remote]
async fn add(x: i32, y: i32) -> i32 {
    x + y
}

// Task submission
let result = add::remote(5, 3).await?;

// Actor implementation
#[rustyray::actor_methods]
impl Counter {
    pub async fn increment(&mut self) -> i32 {
        self.value += 1;
        self.value
    }
}
```

## Next Steps

1. Create rustyray-macros crate
2. Implement basic #[remote] macro
3. Test with simple examples
4. Iterate based on usage

This plan provides a clear path to dramatically improving RustyRay's usability while maintaining its performance and safety guarantees.