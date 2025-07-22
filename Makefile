# Makefile for RustyRay project

# Default test command that enforces single-threaded execution
.PHONY: test
test:
	@echo "Running tests with single-threaded execution..."
	@echo "This is required for test isolation with the global runtime."
	@echo ""
	cargo test -- --test-threads=1

# Run tests for a specific package
.PHONY: test-core
test-core:
	cargo test -p rustyray-core -- --test-threads=1

# Run tests with concurrent execution (will show warning)
.PHONY: test-concurrent
test-concurrent:
	@echo "WARNING: Running tests concurrently may cause failures!"
	@echo "The runtime supports concurrent operations, but tests need isolation."
	@echo ""
	cargo test

# Run a specific test
.PHONY: test-one
test-one:
	@if [ -z "$(TEST)" ]; then \
		echo "Usage: make test-one TEST=test_name"; \
		exit 1; \
	fi
	cargo test $(TEST) -- --test-threads=1

# Build the project
.PHONY: build
build:
	cargo build

# Build release
.PHONY: release
release:
	cargo build --release

# Run clippy
.PHONY: clippy
clippy:
	cargo clippy -- -D warnings

# Format code
.PHONY: fmt
fmt:
	cargo fmt

# Check formatting
.PHONY: fmt-check
fmt-check:
	cargo fmt -- --check

# Clean build artifacts
.PHONY: clean
clean:
	cargo clean

# Run all checks (format, clippy, test)
.PHONY: check
check: fmt-check clippy test

# Help
.PHONY: help
help:
	@echo "RustyRay Makefile commands:"
	@echo "  make test         - Run all tests (single-threaded)"
	@echo "  make test-core    - Run rustyray-core tests"
	@echo "  make test-one TEST=name - Run a specific test"
	@echo "  make build        - Build the project"
	@echo "  make release      - Build release version"
	@echo "  make clippy       - Run clippy linter"
	@echo "  make fmt          - Format code"
	@echo "  make fmt-check    - Check code formatting"
	@echo "  make clean        - Clean build artifacts"
	@echo "  make check        - Run all checks"
	@echo "  make help         - Show this help message"