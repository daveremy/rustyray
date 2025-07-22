# Testing Guide for RustyRay

## Important: Single-Threaded Test Execution Required

RustyRay uses a global runtime pattern (similar to Ray) which requires tests to be run with single-threaded execution to ensure proper test isolation.

**The runtime itself FULLY supports concurrent actors and tasks** - this restriction only applies to test execution.

## Running Tests

### Using Make (Recommended)
```bash
# Run all tests
make test

# Run tests for a specific package
make test-core

# Run a specific test
make test-one TEST=test_name
```

### Using Cargo
Always include `-- --test-threads=1`:
```bash
# Run all tests
cargo test -- --test-threads=1

# Run tests for a specific package
cargo test -p rustyray-core -- --test-threads=1

# Run a specific test
cargo test test_name -- --test-threads=1
```

### Using the Test Script
```bash
./scripts/test.sh
# Or with additional cargo test arguments
./scripts/test.sh -p rustyray-core
```

### Setting Environment Variable
You can also set the environment variable:
```bash
export RUST_TEST_THREADS=1
cargo test
```

## Why Single-Threaded?

1. **Global Runtime**: RustyRay follows Ray's architecture with a global runtime per process
2. **Test Isolation**: Tests need to initialize/shutdown the runtime without interference
3. **NOT a Concurrency Limitation**: The runtime fully supports concurrent actors and tasks

## Concurrent Operations

The runtime supports full concurrency within each test. For example:
- Multiple actors can run concurrently
- Multiple tasks can execute in parallel
- Actors and tasks can interact concurrently

This is demonstrated in tests like:
- `test_concurrent_operations`
- `test_concurrent_actors_and_tasks`

## CI/CD Configuration

For CI pipelines, use one of these approaches:

### GitHub Actions
```yaml
- name: Run tests
  run: make test
```

### GitLab CI
```yaml
test:
  script:
    - cargo test -- --test-threads=1
```

### Jenkins
```groovy
sh 'make test'
```

## Troubleshooting

If you see the error:
```
CONCURRENT TEST EXECUTION DETECTED!
```

This means tests are running concurrently. Use one of the methods above to run tests single-threaded.

## Additional Resources

- Run `make help` to see all available commands
- The test fixtures in `test_utils.rs` automatically detect concurrent test execution
- See the concurrent operation tests for examples of parallel actor/task execution