# Phase 3 Technical Debt Tracker

This document tracks technical improvements and issues identified during Phase 3 development that should be addressed in future iterations.

## From Gemini Code Review (Week 1)

### High Priority
- [x] **Type Detection Enhancement** - Handle fully qualified paths for ObjectRef
  - Status: FIXED in Week 2
  - Added comprehensive type detection in utils.rs
  
- [x] **Test Infrastructure** - Add proper dev-dependencies
  - Status: FIXED in Week 2
  - Added rustyray and tokio-test to dev-dependencies

### Medium Priority  
- [ ] **API Design** - Consider alternatives to `function_remote::remote()` pattern
  - Current: `add_remote::remote(1, 2).await?`
  - Alternative proposals:
    - `add.remote(1, 2).await?` (method on function)
    - `remote!(add(1, 2)).await?` (macro wrapper)
    - `Ray::task(add).args(1, 2).submit().await?` (builder pattern)
  - Decision: Defer to Phase 4 for user feedback

- [ ] **Parameter Validation** - Add Send + 'static bounds checking
  - Ensure all remote function parameters implement required traits
  - Add compile-time validation in macro

- [ ] **Error Messages** - Enhance with better context
  - Add spans to error messages
  - Provide suggestions for common mistakes
  - Use syn::Error::new_spanned consistently

### Low Priority
- [ ] **Documentation** - Better explain the generated module pattern
  - Add rustdoc to generated modules
  - Create macro expansion examples
  - Document the linkme registration process

## New Issues Discovered (Week 2)

### Type System
- [ ] **Type Aliases** - ObjectRef type aliases are not detected
  - Example: `type MyRef = ObjectRef<String>;`
  - Requires semantic analysis or different approach

- [ ] **Nested Types** - Complex nested ObjectRef types
  - Example: `Box<ObjectRef<T>>`, `Vec<ObjectRef<T>>`
  - May need recursive type checking

### Actor System Design Decisions
- [ ] **Actor Handle Naming** - Decide on handle struct naming convention
  - Options: `{Actor}Handle`, `{Actor}Ref`, `Remote{Actor}`
  
- [ ] **Message Passing** - Choose between enum or trait-based messages
  - Enum: Simple, all methods in one place
  - Traits: More flexible, better for large actors

- [ ] **Constructor Handling** - How to handle multiple constructors
  - Current plan: Any method returning Self becomes a constructor
  - Alternative: Explicit #[constructor] attribute

## Testing Improvements
- [ ] **Macro Expansion Tests** - Add tests that verify expanded code
  - Use cargo-expand in tests
  - Compare against expected output

- [ ] **Integration Tests** - Full end-to-end tests with runtime
  - Test remote functions with actual task system
  - Test actor lifecycle with real message passing

- [ ] **Error Case Tests** - More comprehensive error testing
  - Invalid syntax combinations
  - Runtime registration failures
  - Type mismatch scenarios

## Performance Considerations
- [ ] **Registration Overhead** - Measure linkme registration impact
  - Current: All functions registered at startup
  - Alternative: Lazy registration on first use

- [ ] **Serialization Optimization** - Profile ObjectRef serialization
  - Current: Uses bincode
  - Consider: Custom serialization for common types

## Documentation Needs
- [ ] **Macro Book** - Comprehensive guide to all macros
  - Examples for each macro
  - Common patterns and best practices
  - Troubleshooting guide

- [ ] **Architecture Guide** - Explain the overall design
  - How macros integrate with runtime
  - Registration and execution flow
  - Extension points for users

## Future Features (Phase 4+)
- [ ] **Generic Functions** - Support for generic remote functions
  - Requires: Type erasure or monomorphization strategy
  
- [ ] **Lifetime Support** - Allow references with lifetimes
  - Requires: Careful design to ensure safety

- [ ] **Streaming Results** - Support for streaming/chunked returns
  - Use case: Large data processing
  - Requires: New ObjectRef variant

- [ ] **Method Attributes** - Additional attributes for methods
  - `#[timeout(30s)]` - Execution timeout
  - `#[retry(3)]` - Automatic retry on failure
  - `#[cache]` - Result caching

## Refactoring Opportunities
- [ ] **Shared Macro Utilities** - Extract common patterns
  - Function validation
  - Type extraction
  - Error generation

- [ ] **Macro Testing Framework** - Utilities for testing proc macros
  - Token stream comparison
  - Span-aware assertions
  - Mock expansion environment

## Notes
- Items marked [x] are completed
- Priority levels: High (blocking), Medium (important), Low (nice-to-have)
- This document should be updated as new issues are discovered
- Consider creating GitHub issues for high-priority items