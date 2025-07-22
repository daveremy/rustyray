#!/bin/bash
# Test runner script for RustyRay
# This ensures tests are run with proper configuration

set -e  # Exit on error

echo "=================================="
echo "RustyRay Test Runner"
echo "=================================="
echo ""
echo "Tests must run single-threaded due to global runtime."
echo "The runtime itself supports concurrent operations."
echo ""

# Run tests with single-threaded execution
exec cargo test "$@" -- --test-threads=1