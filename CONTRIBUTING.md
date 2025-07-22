# Contributing to RustyRay

Thank you for your interest in contributing to RustyRay! This document provides guidelines for contributing to the project.

## Getting Started

### Prerequisites
- Rust 1.70 or later
- Basic understanding of async Rust and Tokio
- Familiarity with Ray concepts is helpful but not required

### Setup
```bash
# Clone the repository
git clone https://github.com/daveremy/rustyray
cd rustyray

# Build the project
cargo build

# Run tests (must be single-threaded)
cargo test -- --test-threads=1
# or use the Makefile
make test
```

## How to Contribute

### Reporting Issues
- Check existing issues first
- Use issue templates when available
- Include:
  - Clear description of the problem
  - Steps to reproduce
  - Expected vs actual behavior
  - System information (OS, Rust version)
  - Relevant code snippets or error messages

### Submitting Pull Requests

1. **Fork and Clone**
   ```bash
   git clone https://github.com/yourusername/rustyray
   cd rustyray
   git remote add upstream https://github.com/daveremy/rustyray
   ```

2. **Create a Branch**
   ```bash
   git checkout -b feature/your-feature-name
   # or
   git checkout -b fix/issue-description
   ```

3. **Make Your Changes**
   - Follow the coding standards (see below)
   - Add tests for new functionality
   - Update documentation as needed
   - Ensure all tests pass

4. **Commit Your Changes**
   ```bash
   git add .
   git commit -m "feat: add new feature" 
   # or "fix: resolve issue with..."
   # or "docs: update..."
   # or "test: add tests for..."
   ```

5. **Push and Create PR**
   ```bash
   git push origin feature/your-feature-name
   ```
   Then create a pull request on GitHub.

## Coding Standards

### Rust Style
- Follow standard Rust conventions
- Use `cargo fmt` before committing
- Run `cargo clippy` and address warnings
- Prefer explicit types for public APIs
- Document public items with `///` comments

### Code Organization
- Keep modules focused and cohesive
- Use the module structure established in the codebase
- Put tests in the same file using `#[cfg(test)]`
- Integration tests go in the `tests/` directory

### Error Handling
- Use `Result<T, RustyRayError>` for fallible operations
- Add context to errors using `.context()` method
- Create specific error variants rather than using `Internal`

### Testing
- Write unit tests for new functionality
- Use `with_test_runtime()` fixture for tests needing runtime
- Integration tests should demonstrate real usage patterns
- Tests must be able to run with `--test-threads=1`

### Documentation
- Document all public APIs
- Include examples in doc comments
- Update README.md for user-facing changes
- Create ADRs for significant architectural decisions

## Architecture Decision Records (ADRs)

For significant changes, create an ADR:
1. Copy the template from `docs/DOCUMENTATION_STRATEGY.md`
2. Place in `docs/adr/ADR-XXX-title.md`
3. Include in your PR

## Development Workflow

### Running Tests
```bash
# All tests
make test

# Specific package
make test-core

# Single test
make test-one TEST=test_name
```

### Building Documentation
```bash
# Generate rustdoc
cargo doc --open

# Check documentation
cargo doc --no-deps
```

### Performance Considerations
- Benchmark significant changes
- Use `cargo bench` for micro-benchmarks
- Profile with `cargo flamegraph` if needed
- Avoid unnecessary allocations

## Communication

### Getting Help
- Open an issue for questions
- Check existing documentation and issues first
- Be specific about what you're trying to achieve

### Code Review Process
- PRs require at least one review
- Address all feedback or explain why not
- Keep PRs focused - one feature/fix per PR
- Update your PR rather than creating new ones

## What to Work On

### Good First Issues
Look for issues labeled `good first issue` - these are ideal for newcomers.

### Priority Areas
Check the ROADMAP.md for current phase priorities:
- Phase 6: Garbage Collection & Production Features (current)
- Performance optimizations
- Documentation improvements
- Test coverage

### Ideas Welcome
Have an idea not in the roadmap? Open an issue to discuss!

## Recognition

Contributors will be:
- Listed in release notes
- Acknowledged in relevant documentation
- Part of building the future of distributed computing in Rust!

## Questions?

If anything is unclear, please open an issue asking for clarification. We're happy to help!

Thank you for contributing to RustyRay! 🦀✨