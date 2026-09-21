#!/bin/bash
#! # Simple FFI Test for Frozen DuckDB
#!
#! This script performs basic FFI validation by testing that the
#! frozen-duckdb library can be loaded and basic functions work.
#!
#! ## Usage
#!
#! ```bash
#! # Run simple FFI test
#! ./scripts/test_ffi_simple.sh
#! ```
#
# STATUS (TR6 2026-09-21): KEPT-UNCERTAIN. Frozen-duckdb-native smoke test; Tests 4-7 target real
# workspace targets (examples/basic_usage.rs and examples/flock_ollama_integration.rs exist).
# BLOCKED as committed: Tests 1-3 require a repo-local prebuilt/libduckdb*.dylib, but the builder
# now keeps binaries in ~/.frozen-duckdb/cache/v{VER}-{arch}/ (no dylib is committed; observed
# live: exit 1 "DuckDB library not found"), and prebuilt/setup_env.sh resolves DUCKDB_LIB_DIR
# from $0, which breaks when sourced from scripts/ callers. Library-resolution repair is the
# T6 (scripts-fix) lane; see docs/sjira/v26.9.21/TR6.md History.

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

echo -e "${BLUE}🦆 Frozen DuckDB Simple FFI Test${NC}"
echo "=================================================="

# Check if we're in the right directory
if [[ ! -f "$PROJECT_ROOT/Cargo.toml" ]]; then
    echo -e "${RED}❌ Error: Not in frozen-duckdb project root${NC}"
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

# Test 1: Check if DuckDB library exists and is loadable
echo -e "${BLUE}🧪 Test 1: Library Loading${NC}"
if [[ -f "$DUCKDB_LIB_DIR/libduckdb.dylib" ]]; then
    echo -e "${GREEN}✅ DuckDB library found: libduckdb.dylib${NC}"
    
    # Check library size
    LIB_SIZE=$(stat -f%z "$DUCKDB_LIB_DIR/libduckdb.dylib" 2>/dev/null || stat -c%s "$DUCKDB_LIB_DIR/libduckdb.dylib" 2>/dev/null || echo "unknown")
    echo -e "${GREEN}✅ Library size: $LIB_SIZE bytes${NC}"
else
    echo -e "${RED}❌ DuckDB library not found${NC}"
    exit 1
fi

# Test 2: Check if headers exist
echo -e "${BLUE}🧪 Test 2: Header Files${NC}"
if [[ -f "$DUCKDB_LIB_DIR/duckdb.h" ]]; then
    echo -e "${GREEN}✅ DuckDB header found: duckdb.h${NC}"
    
    # Check header size
    HEADER_SIZE=$(stat -f%z "$DUCKDB_LIB_DIR/duckdb.h" 2>/dev/null || stat -c%s "$DUCKDB_LIB_DIR/duckdb.h" 2>/dev/null || echo "unknown")
    echo -e "${GREEN}✅ Header size: $HEADER_SIZE bytes${NC}"
else
    echo -e "${RED}❌ DuckDB header not found${NC}"
    exit 1
fi

# Test 3: Check architecture-specific binaries
echo -e "${BLUE}🧪 Test 3: Architecture-Specific Binaries${NC}"
ARCH=$(uname -m)
case "$ARCH" in
    "x86_64"|"amd64")
        EXPECTED_BINARY="libduckdb_x86_64.dylib"
        ;;
    "arm64"|"aarch64")
        EXPECTED_BINARY="libduckdb_arm64.dylib"
        ;;
    *)
        echo -e "${YELLOW}⚠️  Unknown architecture: $ARCH${NC}"
        EXPECTED_BINARY=""
        ;;
esac

if [[ -n "$EXPECTED_BINARY" ]]; then
    if [[ -f "$DUCKDB_LIB_DIR/$EXPECTED_BINARY" ]]; then
        echo -e "${GREEN}✅ Architecture-specific binary found: $EXPECTED_BINARY${NC}"
        
        # Check binary size
        BINARY_SIZE=$(stat -f%z "$DUCKDB_LIB_DIR/$EXPECTED_BINARY" 2>/dev/null || stat -c%s "$DUCKDB_LIB_DIR/$EXPECTED_BINARY" 2>/dev/null || echo "unknown")
        echo -e "${GREEN}✅ Binary size: $BINARY_SIZE bytes${NC}"
    else
        echo -e "${YELLOW}⚠️  Architecture-specific binary not found: $EXPECTED_BINARY${NC}"
    fi
fi

# Test 4: Test Rust integration
echo -e "${BLUE}🧪 Test 4: Rust Integration${NC}"
cd "$PROJECT_ROOT"

# Test that the library can be built
if cargo build --lib --release; then
    echo -e "${GREEN}✅ Rust library builds successfully${NC}"
else
    echo -e "${RED}❌ Rust library build failed${NC}"
    exit 1
fi

# Test 5: Test basic DuckDB functionality through Rust
echo -e "${BLUE}🧪 Test 5: Basic DuckDB Functionality${NC}"
if cargo run --example basic_usage --release; then
    echo -e "${GREEN}✅ Basic DuckDB functionality works${NC}"
else
    echo -e "${RED}❌ Basic DuckDB functionality failed${NC}"
    exit 1
fi

# Test 6: Test Flock extension loading
echo -e "${BLUE}🧪 Test 6: Flock Extension${NC}"
if cargo run --example flock_ollama_integration --release 2>/dev/null; then
    echo -e "${GREEN}✅ Flock extension loads successfully${NC}"
else
    echo -e "${YELLOW}⚠️  Flock extension test had issues (may be expected without Ollama)${NC}"
fi

# Test 7: Run existing tests
echo -e "${BLUE}🧪 Test 7: Existing Test Suite${NC}"
if cargo test --lib --release; then
    echo -e "${GREEN}✅ Existing tests pass${NC}"
else
    echo -e "${RED}❌ Some existing tests failed${NC}"
    exit 1
fi

# Summary
echo ""
echo -e "${BLUE}📊 FFI Test Summary${NC}"
echo "=================================================="
echo -e "${GREEN}✅ All FFI tests passed!${NC}"
echo -e "${GREEN}✅ Frozen DuckDB library is properly configured${NC}"
echo -e "${GREEN}✅ Core functionality is working${NC}"
echo -e "${GREEN}✅ Architecture detection is working${NC}"
echo -e "${GREEN}✅ Rust integration is working${NC}"

echo ""
echo -e "${BLUE}🎉 FFI validation completed successfully!${NC}"
echo -e "${GREEN}The frozen-duckdb library is ready for use.${NC}"
