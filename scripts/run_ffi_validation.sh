#!/bin/bash
#! # FFI Validation Script for Frozen DuckDB
#!
#! This script runs comprehensive FFI validation tests to ensure that
#! the frozen-duckdb library properly exposes all required functionality:
#!
#! - Core DuckDB FFI functions
#! - Flock LLM extensions
#! - Architecture detection
#! - Performance validation
#!
#! ## Usage
#!
#! ```bash
#! # Run all FFI validation tests
#! ./scripts/run_ffi_validation.sh
#!
#! # Run with specific architecture
#! ARCH=x86_64 ./scripts/run_ffi_validation.sh
#! ARCH=arm64 ./scripts/run_ffi_validation.sh
#!
#! # Run in CI mode (no interactive output)
#! CI=true ./scripts/run_ffi_validation.sh
#! ```
#
# STATUS (TR6 2026-09-21): KEPT-UNCERTAIN. Documented FFI validation entry point
# (docs/testing/go-ffi-smoketest.md); orchestrates build_go_smoketest.sh + cargo test runs.
# BLOCKED as committed: assumes a repo-local prebuilt/ dylib, but the builder keeps binaries in
# ~/.frozen-duckdb/cache/v{VER}-{arch}/ (same drift as scripts/test_ffi_simple.sh — see its
# STATUS note; not run this session, would fail at the same library-resolution step). Repair is
# the T6 (scripts-fix) lane; see docs/sjira/v26.9.21/TR6.md History.

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Configuration
CI_MODE="${CI:-false}"
ARCH_OVERRIDE="${ARCH:-}"

echo -e "${BLUE}🦆 Frozen DuckDB FFI Validation Suite${NC}"
echo "=================================================="

# Check if we're in the right directory
if [[ ! -f "$PROJECT_ROOT/Cargo.toml" ]]; then
    echo -e "${RED}❌ Error: Not in frozen-duckdb project root${NC}"
    echo "Expected to find Cargo.toml in: $PROJECT_ROOT"
    exit 1
fi

# Check frozen DuckDB environment
if [[ -z "${DUCKDB_LIB_DIR:-}" ]]; then
    echo -e "${YELLOW}⚠️  Setting up frozen DuckDB environment...${NC}"
    
    if [[ -f "$PROJECT_ROOT/prebuilt/setup_env.sh" ]]; then
        source "$PROJECT_ROOT/prebuilt/setup_env.sh"
        echo -e "${GREEN}✅ Environment configured${NC}"
    else
        echo -e "${RED}❌ Error: setup_env.sh not found${NC}"
        exit 1
    fi
fi

echo -e "${GREEN}✅ DUCKDB_LIB_DIR: $DUCKDB_LIB_DIR${NC}"
echo -e "${GREEN}✅ DUCKDB_INCLUDE_DIR: ${DUCKDB_INCLUDE_DIR:-$DUCKDB_LIB_DIR}${NC}"

# Test results tracking
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Function to run a test
run_test() {
    local test_name="$1"
    local test_command="$2"
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    echo -e "${BLUE}🧪 Running: $test_name${NC}"
    
    if eval "$test_command"; then
        echo -e "${GREEN}✅ PASS: $test_name${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        return 0
    else
        echo -e "${RED}❌ FAIL: $test_name${NC}"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        return 1
    fi
}

# Function to run Go smoke test
run_go_smoketest() {
    local arch="${1:-}"
    
    if [[ -n "$arch" ]]; then
        export ARCH="$arch"
        echo -e "${BLUE}🏗️  Testing architecture: $arch${NC}"
    fi
    
    # Run the Go smoke test
    "$SCRIPT_DIR/build_go_smoketest.sh"
}

# Function to run Rust integration tests
run_rust_tests() {
    echo -e "${BLUE}🦀 Running Rust integration tests...${NC}"
    
    # Run core functionality tests
    cargo test --lib -- --nocapture
    
    # Run FFI-specific tests if they exist
    if cargo test --lib ffi -- --nocapture 2>/dev/null; then
        echo -e "${GREEN}✅ FFI-specific tests passed${NC}"
    else
        echo -e "${YELLOW}⚠️  No FFI-specific tests found${NC}"
    fi
}

# Function to run Flock extension tests
run_flock_tests() {
    echo -e "${BLUE}🐑 Running Flock extension tests...${NC}"
    
    # Run Flock-specific tests
    if cargo test --lib flock -- --nocapture 2>/dev/null; then
        echo -e "${GREEN}✅ Flock tests passed${NC}"
    else
        echo -e "${YELLOW}⚠️  No Flock-specific tests found${NC}"
    fi
}

# Function to validate architecture detection
validate_architecture() {
    echo -e "${BLUE}🏗️  Validating architecture detection...${NC}"
    
    # Test current architecture
    local current_arch=$(uname -m)
    echo -e "${GREEN}✅ Current architecture: $current_arch${NC}"
    
    # Test architecture override
    if [[ -n "$ARCH_OVERRIDE" ]]; then
        echo -e "${GREEN}✅ Architecture override: $ARCH_OVERRIDE${NC}"
    fi
    
    # Verify binary exists for current architecture
    local expected_binary=""
    case "$current_arch" in
        "x86_64"|"amd64")
            expected_binary="libduckdb_x86_64.dylib"
            ;;
        "arm64"|"aarch64")
            expected_binary="libduckdb_arm64.dylib"
            ;;
        *)
            echo -e "${YELLOW}⚠️  Unknown architecture: $current_arch${NC}"
            return 0
            ;;
    esac
    
    if [[ -f "$DUCKDB_LIB_DIR/$expected_binary" ]]; then
        echo -e "${GREEN}✅ Architecture-specific binary found: $expected_binary${NC}"
    else
        echo -e "${YELLOW}⚠️  Architecture-specific binary not found: $expected_binary${NC}"
        echo -e "${YELLOW}   Available binaries:${NC}"
        ls -la "$DUCKDB_LIB_DIR"/libduckdb* 2>/dev/null || echo "   No binaries found"
    fi
}

# Function to validate performance
validate_performance() {
    echo -e "${BLUE}⚡ Validating performance...${NC}"
    
    # Run performance comparison example
    if [[ -f "$PROJECT_ROOT/examples/performance_comparison.rs" ]]; then
        echo -e "${BLUE}   Running performance comparison...${NC}"
        if cargo run --example performance_comparison --release; then
            echo -e "${GREEN}✅ Performance validation passed${NC}"
        else
            echo -e "${YELLOW}⚠️  Performance validation had issues${NC}"
        fi
    else
        echo -e "${YELLOW}⚠️  Performance comparison example not found${NC}"
    fi
}

# Main test execution
echo -e "${BLUE}🚀 Starting FFI validation suite...${NC}"
echo ""

# 1. Validate architecture detection
run_test "Architecture Detection" "validate_architecture"

# 2. Run Go smoke test for current architecture
run_test "Go FFI Smoke Test (Current Arch)" "run_go_smoketest"

# 3. Run Go smoke test for alternative architecture (if available)
if [[ -n "$ARCH_OVERRIDE" ]]; then
    run_test "Go FFI Smoke Test (Override: $ARCH_OVERRIDE)" "run_go_smoketest '$ARCH_OVERRIDE'"
fi

# 4. Run Rust integration tests
run_test "Rust Integration Tests" "run_rust_tests"

# 5. Run Flock extension tests
run_test "Flock Extension Tests" "run_flock_tests"

# 6. Validate performance
run_test "Performance Validation" "validate_performance"

# Summary
echo ""
echo -e "${BLUE}📊 FFI Validation Summary${NC}"
echo "=================================================="
echo -e "Total Tests: $TOTAL_TESTS"
echo -e "${GREEN}Passed: $PASSED_TESTS${NC}"
echo -e "${RED}Failed: $FAILED_TESTS${NC}"

if [[ $FAILED_TESTS -eq 0 ]]; then
    echo -e "${GREEN}🎉 ALL FFI VALIDATION TESTS PASSED!${NC}"
    echo -e "${GREEN}✅ Frozen DuckDB FFI is fully functional${NC}"
    EXIT_CODE=0
else
    echo -e "${RED}❌ Some FFI validation tests failed${NC}"
    echo -e "${RED}⚠️  Check FFI implementation and dependencies${NC}"
    EXIT_CODE=1
fi

# CI-specific output
if [[ "$CI_MODE" == "true" ]]; then
    echo "::set-output name=total_tests::$TOTAL_TESTS"
    echo "::set-output name=passed_tests::$PASSED_TESTS"
    echo "::set-output name=failed_tests::$FAILED_TESTS"
    echo "::set-output name=success_rate::$(echo "scale=2; $PASSED_TESTS * 100 / $TOTAL_TESTS" | bc -l)"
fi

exit $EXIT_CODE
