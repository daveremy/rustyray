# RustyRay Development Bootstrap Prompt

Use this prompt to continue RustyRay development on another machine or session.

## Project Context

I'm working on RustyRay, a Rust implementation of Ray Core (the distributed actor system). The project is at a critical juncture, about to begin Phase 3 which will add a procedural macro system to dramatically improve API ergonomics.

## Current Status

- **Repository**: https://github.com/daveremy/rustyray
- **Current Version**: 0.2.5 (Phase 2.5 complete)
- **Next Target**: 0.3.0 (Phase 3 - Macro System)
- **Timeline**: 6-week implementation starting now

## Completed Phases

### Phase 1: Local Actor System ✅
- Implemented actor infrastructure with typed handles
- Async message passing with tokio
- Graceful shutdown mechanisms

### Phase 2: Task Execution System ✅
- Task system with ObjectRef futures
- Function registry with task_function! macro
- Integration with actor system

### Phase 2.5: Code Review Improvements ✅
- Fixed all critical issues from Gemini review
- Implemented proper cancellation/timeout
- Added test synchronization utilities
- Optimized with bytes::Bytes for zero-copy
- Received "Excellent" rating from comprehensive review

## Phase 3 Plan (Current Focus)

We just completed planning for Phase 3 with Gemini's feedback. The goal is to achieve Python-like API simplicity while maintaining Rust's type safety.

### Key Objectives:
- 70% reduction in boilerplate code
- Full compile-time type safety
- <5% performance overhead
- Support both async and sync functions

### Technical Approach:
- Procedural macros: `#[rustyray::remote]`, `#[rustyray::actor]`, `#[rustyray::actor_methods]`
- Compile-time registration with `linkme`
- Global runtime with `#[rustyray::main]`
- Focus on excellent error messages using `syn::Error::new_spanned`

### Timeline:
- Week 1: Foundation & Infrastructure
- Week 2: Remote Function Macro
- Week 3: Actor Macros
- Week 4: Runtime Management
- Week 5: Advanced Features & Polish
- Week 6: Testing & Release

## Key Technical Decisions

1. **Architecture**: Separate `rustyray-macros` crate for proc macros
2. **Registration**: Use `linkme` for zero-cost compile-time registration
3. **Error Handling**: `thiserror` for type-safe errors
4. **Async Runtime**: Committed to `tokio`
5. **Serialization**: `serde` for now

## Important Files to Review

1. `/docs/phase3-final-plan.md` - Detailed implementation plan
2. `/docs/phase3-gemini-feedback.md` - Critical feedback incorporated
3. `/examples/future_api_vision.rs.txt` - Target API design
4. `/ROADMAP.md` - Updated with current progress
5. `/src/task/system.rs` - Current task system implementation
6. `/src/actor/mod.rs` - Current actor implementation

## Next Steps

1. Create `rustyray-macros` crate structure
2. Setup workspace configuration
3. Implement basic `#[rustyray::remote]` macro
4. Start with async functions, add sync support
5. Use `trybuild` for comprehensive testing

## Development Guidelines

1. **Testing**: Use `trybuild` from day one for macro testing
2. **Errors**: Every macro error must point to specific code with helpful messages
3. **Incremental**: Get `#[remote]` perfect before moving to actors
4. **Performance**: Benchmark against manual implementation continuously

## Example of Current vs Future API

### Current (Verbose):
```rust
let task_system = TaskSystem::new();
task_system.register_function(
    "add",
    task_function!(|x: i32, y: i32| async move {
        Ok::<i32, RustyRayError>(x + y)
    })
)?;

let result = TaskBuilder::new("add")
    .arg(5)
    .arg(3)
    .submit::<i32>(&task_system)
    .await?;
```

### Future (Simple):
```rust
#[rustyray::remote]
async fn add(x: i32, y: i32) -> i32 {
    x + y
}

let result = add::remote(5, 3).await?;
```

## Commands to Get Started

```bash
# Clone the repository
git clone https://github.com/daveremy/rustyray.git
cd rustyray

# Review the current state
cargo test
cargo run --example tasks
cargo run --example actors

# Read the phase 3 plan
cat docs/phase3-final-plan.md

# Start Phase 3 implementation
# First: Create rustyray-macros crate...
```

## Context for AI Assistant

You're helping implement Phase 3 of RustyRay. The planning is complete and approved. Focus on:
1. Creating the macro crate structure
2. Implementing `#[rustyray::remote]` first
3. Ensuring excellent error messages
4. Following the 6-week timeline
5. Maintaining backward compatibility

The goal is to transform RustyRay's API from verbose to elegant, matching Ray's simplicity while keeping Rust's safety guarantees.