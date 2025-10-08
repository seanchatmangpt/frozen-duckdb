#!/bin/bash
#! # Build Script for Go FFI Smoke Test
#!
#! This script builds and runs the Go smoke test that validates all FFI
#! functionality in the frozen-duckdb library, including:
#!
#! - Core DuckDB FFI functions
#! - Flock LLM extensions
#! - Architecture detection
#! - Performance validation
#!
#! ## Usage
#!
#! ```bash
#! # Build and run with frozen DuckDB environment
#! source prebuilt/setup_env.sh
#! ./scripts/build_go_smoketest.sh
#!
#! # Build and run with specific architecture
#! ARCH=x86_64 source prebuilt/setup_env.sh && ./scripts/build_go_smoketest.sh
#! ARCH=arm64 source prebuilt/setup_env.sh && ./scripts/build_go_smoketest.sh
#! ```

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
GO_VERSION="1.21"
CGO_ENABLED="1"

echo -e "${BLUE}🦆 Frozen DuckDB Go FFI Smoke Test Builder${NC}"
echo "=================================================="

# Check if we're in the right directory
if [[ ! -f "$PROJECT_ROOT/Cargo.toml" ]]; then
    echo -e "${RED}❌ Error: Not in frozen-duckdb project root${NC}"
    echo "Expected to find Cargo.toml in: $PROJECT_ROOT"
    exit 1
fi

# Check Go installation
if ! command -v go &> /dev/null; then
    echo -e "${RED}❌ Error: Go is not installed${NC}"
    echo "Please install Go $GO_VERSION or later"
    exit 1
fi

GO_VERSION_INSTALLED=$(go version | grep -o 'go[0-9]\+\.[0-9]\+' | sed 's/go//')
echo -e "${GREEN}✅ Go version: $GO_VERSION_INSTALLED${NC}"

# Check if frozen DuckDB environment is set up
if [[ -z "${DUCKDB_LIB_DIR:-}" ]]; then
    echo -e "${YELLOW}⚠️  Warning: DUCKDB_LIB_DIR not set${NC}"
    echo "Attempting to use prebuilt directory..."
    
    if [[ -d "$PROJECT_ROOT/prebuilt" ]]; then
        export DUCKDB_LIB_DIR="$PROJECT_ROOT/prebuilt"
        export DUCKDB_INCLUDE_DIR="$PROJECT_ROOT/prebuilt"
        echo -e "${GREEN}✅ Using prebuilt directory: $DUCKDB_LIB_DIR${NC}"
    else
        echo -e "${RED}❌ Error: No frozen DuckDB environment found${NC}"
        echo "Please run: source prebuilt/setup_env.sh"
        exit 1
    fi
else
    echo -e "${GREEN}✅ DUCKDB_LIB_DIR: $DUCKDB_LIB_DIR${NC}"
    echo -e "${GREEN}✅ DUCKDB_INCLUDE_DIR: ${DUCKDB_INCLUDE_DIR:-$DUCKDB_LIB_DIR}${NC}"
fi

# Check for required DuckDB files
if [[ ! -f "$DUCKDB_LIB_DIR/duckdb.h" ]]; then
    echo -e "${RED}❌ Error: duckdb.h not found in $DUCKDB_LIB_DIR${NC}"
    exit 1
fi

# Find the appropriate library file
LIB_FILE=""
if [[ -f "$DUCKDB_LIB_DIR/libduckdb.dylib" ]]; then
    LIB_FILE="$DUCKDB_LIB_DIR/libduckdb.dylib"
elif [[ -f "$DUCKDB_LIB_DIR/libduckdb.so" ]]; then
    LIB_FILE="$DUCKDB_LIB_DIR/libduckdb.so"
elif [[ -f "$DUCKDB_LIB_DIR/libduckdb.dll" ]]; then
    LIB_FILE="$DUCKDB_LIB_DIR/libduckdb.dll"
else
    echo -e "${RED}❌ Error: No DuckDB library file found in $DUCKDB_LIB_DIR${NC}"
    echo "Expected: libduckdb.dylib, libduckdb.so, or libduckdb.dll"
    exit 1
fi

echo -e "${GREEN}✅ DuckDB library: $LIB_FILE${NC}"

# Set up Go environment
export CGO_ENABLED=1
export CGO_CFLAGS="-I$DUCKDB_INCLUDE_DIR"
export CGO_LDFLAGS="-L$DUCKDB_LIB_DIR -lduckdb"

# Architecture detection
ARCH=$(uname -m)
if [[ -n "${ARCH_OVERRIDE:-}" ]]; then
    ARCH="$ARCH_OVERRIDE"
fi

echo -e "${GREEN}✅ Target architecture: $ARCH${NC}"

# Create temporary Go module for smoke test
TEMP_DIR=$(mktemp -d)
echo -e "${BLUE}📁 Working directory: $TEMP_DIR${NC}"

cd "$TEMP_DIR"

# Set up Go environment in temp directory
export PATH="$HOME/.asdf/shims:$PATH"
asdf local golang 1.22.5

# Initialize Go module
go mod init frozen-duckdb-smoketest

# Copy smoke test files
cp "$SCRIPT_DIR/smoke_go_simple.go" .
cp "$SCRIPT_DIR/duckdb_ffi.h" .

# Rename the simple smoke test to main.go
mv smoke_go_simple.go main.go

# Build the smoke test
echo -e "${BLUE}🔨 Building Go smoke test...${NC}"
if go build -o smoketest .; then
    echo -e "${GREEN}✅ Build successful${NC}"
else
    echo -e "${RED}❌ Build failed${NC}"
    exit 1
fi

# Run the smoke test
echo -e "${BLUE}🧪 Running FFI smoke test...${NC}"
echo "=================================================="

if ./smoketest; then
    echo -e "${GREEN}🎉 All FFI tests passed!${NC}"
    EXIT_CODE=0
else
    echo -e "${RED}❌ Some FFI tests failed${NC}"
    EXIT_CODE=1
fi

# Cleanup
cd "$PROJECT_ROOT"
rm -rf "$TEMP_DIR"

echo "=================================================="
if [[ $EXIT_CODE -eq 0 ]]; then
    echo -e "${GREEN}✅ Go FFI smoke test completed successfully${NC}"
else
    echo -e "${RED}❌ Go FFI smoke test failed${NC}"
fi

exit $EXIT_CODE
