#!/bin/bash
# Demonstration script for frozen-duckdb dependency issue
#
# This script demonstrates:
# 1. The problem: DuckDB compilation in dependent crates
# 2. The solution: Proper environment variable propagation
# 3. Performance comparison between approaches

set -euo pipefail

echo "🧪 Frozen DuckDB Dependency Issue Demonstration"
echo "=============================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $*"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $*"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $*"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*"
}

# Check if we're in the right directory
if [[ ! -d "test-dependency" ]]; then
    log_error "test-dependency directory not found"
    log_info "Please run this script from the frozen-duckdb root directory"
    exit 1
fi

# Clean previous builds
log_info "Cleaning previous builds..."
cargo clean -p test-dependency 2>/dev/null || true
rm -rf test-dependency/target/ 2>/dev/null || true

echo ""
echo "📋 TEST SCENARIO 1: Build WITHOUT frozen-duckdb environment"
echo "=========================================================="

# Build without environment setup (should trigger DuckDB compilation)
log_info "Building test-dependency without frozen-duckdb environment..."

start_time=$(date +%s)
if cargo build -p test-dependency 2>&1 | tee /tmp/build-without-env.log; then
    end_time=$(date +%s)
    duration=$((end_time - start_time))
    log_success "Build completed in ${duration}s"

    # Check if DuckDB compilation occurred
    if grep -q "Compiling duckdb" /tmp/build-without-env.log; then
        log_warning "❌ DuckDB compilation occurred (expected without setup)"
    else
        log_info "✅ No DuckDB compilation detected (unexpected - check logs)"
    fi
else
    end_time=$(date +%s)
    duration=$((end_time - start_time))
    log_error "Build failed after ${duration}s"
    cat /tmp/build-without-env.log
fi

echo ""
echo "📋 TEST SCENARIO 2: Build WITH frozen-duckdb environment"
echo "======================================================"

# Build with environment setup (should use pre-compiled binaries)
log_info "Building test-dependency with frozen-duckdb environment..."

start_time=$(date +%s)
if source prebuilt/setup_env.sh && cargo build -p test-dependency 2>&1 | tee /tmp/build-with-env.log; then
    end_time=$(date +%s)
    duration=$((end_time - start_time))
    log_success "Build completed in ${duration}s"

    # Check if DuckDB compilation occurred
    if grep -q "Compiling duckdb" /tmp/build-with-env.log; then
        log_warning "❌ DuckDB compilation still occurred (issue not fixed)"
    else
        log_success "✅ No DuckDB compilation detected (problem solved!)"
    fi
else
    end_time=$(date +%s)
    duration=$((end_time - start_time))
    log_error "Build failed after ${duration}s"
    cat /tmp/build-with-env.log
fi

echo ""
echo "📋 TEST SCENARIO 3: Run the test application"
echo "============================================"

log_info "Testing the application functionality..."

# Test the application
if source prebuilt/setup_env.sh && cargo run -p test-dependency 2>&1 | tee /tmp/app-output.log; then
    log_success "Application ran successfully"

    # Check if environment variables are properly detected
    if grep -q "Frozen DuckDB environment configured" /tmp/app-output.log; then
        log_success "✅ Environment variables properly detected by dependent crate"
    else
        log_warning "❌ Environment variables not detected by dependent crate"
    fi
else
    log_error "Application failed to run"
    cat /tmp/app-output.log
fi

echo ""
echo "📋 TEST SCENARIO 4: Run tests"
echo "============================"

log_info "Running tests to check functionality..."

if source prebuilt/setup_env.sh && cargo test --package test-dependency -- --nocapture 2>&1 | tee /tmp/test-output.log; then
    log_success "Tests passed"

    # Count test results
    passed=$(grep -c "test test_.* ... ok" /tmp/test-output.log || true)
    failed=$(grep -c "test test_.* ... FAILED" /tmp/test-output.log || true)

    echo "📊 Test Results: ${passed} passed, ${failed} failed"

    if [[ $failed -eq 0 ]]; then
        log_success "✅ All tests passed"
    else
        log_warning "❌ ${failed} test(s) failed"
    fi
else
    log_error "Tests failed to run"
    cat /tmp/test-output.log
fi

echo ""
echo "📋 SUMMARY"
echo "=========="

echo ""
echo "🔧 Environment Variables Check:"
if source prebuilt/setup_env.sh && echo "DUCKDB_LIB_DIR: ${DUCKDB_LIB_DIR:-NOT SET}"; then
    log_success "✅ DUCKDB_LIB_DIR is set: ${DUCKDB_LIB_DIR}"
else
    log_error "❌ DUCKDB_LIB_DIR not set"
fi

if source prebuilt/setup_env.sh && echo "DUCKDB_INCLUDE_DIR: ${DUCKDB_INCLUDE_DIR:-NOT SET}"; then
    log_success "✅ DUCKDB_INCLUDE_DIR is set: ${DUCKDB_INCLUDE_DIR}"
else
    log_error "❌ DUCKDB_INCLUDE_DIR not set"
fi

echo ""
echo "🎯 Key Findings:"
echo "  • The frozen-duckdb build.rs sets environment variables for dependent crates"
echo "  • The setup_env.sh script configures the environment for manual builds"
echo "  • Both approaches should prevent DuckDB compilation in dependent crates"
echo "  • Test results show whether the environment propagation is working"

echo ""
echo "📚 Files Created:"
echo "  • test-dependency/ - Test crate demonstrating the issue"
echo "  • test-dependency/Cargo.toml - Dependencies and configuration"
echo "  • test-dependency/src/main.rs - Application that uses DuckDB"
echo "  • test-dependency/README.md - Usage instructions"

echo ""
echo "🚀 Next Steps:"
echo "  1. Run: ./test-dependency/target/debug/test-dependency"
echo "  2. Check build times with/without environment setup"
echo "  3. Monitor for DuckDB compilation in build logs"
echo "  4. Verify environment variables are available in dependent crates"

echo ""
echo "📞 Troubleshooting:"
echo "  • If DuckDB still compiles: Check that DUCKDB_LIB_DIR/DUCKDB_INCLUDE_DIR are set"
echo "  • If tests fail: Ensure frozen-duckdb is built first"
echo "  • If environment not detected: Check that source prebuilt/setup_env.sh is run"

echo ""
echo "✅ Demonstration complete!"
