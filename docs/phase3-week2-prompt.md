# RustyRay Phase 3 Week 2 Development Prompt

Use this prompt to continue RustyRay Phase 3 development in a new session.

## Current Status

- **Repository**: https://github.com/daveremy/rustyray
- **Current Branch**: main
- **Last Commit**: Phase 3 Week 1 implementation complete
- **Current Version**: 0.3.0-dev

## Phase 3 Week 1 Accomplishments

### ✅ Completed (Exceeded Week 1 Goals):
1. Workspace restructure (rustyray-core, rustyray-macros, rustyray)
2. Full #[remote] macro implementation with:
   - Async function support
   - Sync function support (Week 2 goal achieved early!)
   - Resource requirements (num_cpus, num_gpus)
   - ObjectRef arguments (Week 2 goal achieved early!)
   - Result return type detection
   - Visibility preservation
   - Compile-time registration with linkme
3. Working examples (macro_demo.rs, objectref_demo.rs)
4. Basic trybuild tests
5. Comprehensive error handling improvements

### 📊 Code Review Feedback (Grade: A-)

Gemini's review identified these priorities:

#### Critical Issues to Fix First:
1. **Type Detection** - Handle fully qualified paths (e.g., `rustyray_core::ObjectRef`)
2. **Test Infrastructure** - Add proper dev-dependencies and expand coverage
3. **Documentation** - Explain the generated module pattern

#### Medium Priority:
1. API design consideration (function_remote::remote() pattern)
2. Parameter validation for Send + 'static bounds
3. Enhanced error messages with context

## Week 2 Goals

### Original Phase 3 Plan:
- Day 1-2: Basic #[actor] implementation
- Day 3-4: #[actor_methods] implementation
- Day 5: Actor resources & error handling

### Recommended Approach:
1. **Fix critical issues first** (2-3 hours)
2. **Create technical debt tracker** (30 min)
3. **Then proceed with actor macros**

## Critical Fixes Needed

### 1. Improve Type Detection (HIGH PRIORITY)

Current implementation only checks last path segment:
```rust
// File: crates/rustyray-macros/src/remote.rs
fn is_object_ref_type(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                segment.ident == "ObjectRef"
            } else {
                false
            }
        }
        syn::Type::Reference(type_ref) => is_object_ref_type(&type_ref.elem),
        _ => false,
    }
}
```

Should handle:
- Fully qualified paths: `rustyray_core::ObjectRef<T>`
- Type aliases: `type MyRef = ObjectRef<i32>`
- Nested types: `Box<ObjectRef<T>>`

### 2. Fix Test Infrastructure

Add to `crates/rustyray-macros/Cargo.toml`:
```toml
[dev-dependencies]
rustyray = { path = "../rustyray" }
tokio-test = "0.4"
```

### 3. Create Technical Debt Tracker

Create `docs/phase3-technical-debt.md` to track all improvements.

## Actor Macro Design (Week 2 Focus)

### Target API:
```rust
#[rustyray::actor]
pub struct Counter {
    value: i32,
}

#[rustyray::actor_methods]
impl Counter {
    pub fn new(initial: i32) -> Self {
        Counter { value: initial }
    }
    
    pub async fn increment(&mut self) -> i32 {
        self.value += 1;
        self.value
    }
    
    pub fn get_value(&self) -> i32 {
        self.value
    }
}

// Usage:
let counter = Counter::new(0).remote().await?;
let result = counter.increment().await?;
```

### Implementation Plan:
1. Generate typed handle struct
2. Create message/response enums
3. Implement handle methods
4. Support both sync and async methods
5. Handle constructors

## Key Files to Review

1. `/docs/phase3-final-plan.md` - Detailed implementation plan
2. `/crates/rustyray-macros/src/remote.rs` - Current remote macro (reference)
3. `/crates/rustyray-core/src/actor/mod.rs` - Actor system to integrate with
4. `/examples/future_api_vision.rs.txt` - Target API design

## Development Commands

```bash
# Run tests
cargo test -p rustyray-macros

# Run examples
cargo run -p rustyray --example macro_demo

# Check specific macro expansion
cargo expand -p rustyray --example macro_demo

# Run trybuild tests
cd crates/rustyray-macros && cargo test ui_tests
```

## Next Steps

1. **First Hour**: Fix type detection and test infrastructure
2. **Create Technical Debt Tracker**: Document all improvements
3. **Start Actor Macro**: Begin with #[actor] attribute
4. **Apply Lessons**: Use improved patterns from the start

## Important Context

- The `function_remote::remote()` API pattern has been questioned - consider alternatives
- ObjectRef serialization doesn't support distributed execution yet (core limitation)
- Registration uses linkme for compile-time function discovery
- All macros should maintain the same quality standards as #[remote]

## Success Criteria for Week 2

1. Fixed critical type detection issues
2. Basic #[actor] macro working
3. #[actor_methods] generating handle methods
4. Improved test coverage
5. Technical debt documented and tracked

Use this prompt to pick up development and continue building the actor macro system!