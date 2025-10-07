#!/bin/bash
# Setup environment for prebuilt DuckDB binary
export DUCKDB_LIB_DIR="$(dirname "$(realpath "$0")")"
export DUCKDB_INCLUDE_DIR="$(dirname "$(realpath "$0")")"

# Set library path for runtime and build time
if [[ "$OSTYPE" == "darwin"* ]]; then
    export DYLD_FALLBACK_LIBRARY_PATH="${DUCKDB_LIB_DIR}:${DYLD_FALLBACK_LIBRARY_PATH:-}"
    export DYLD_LIBRARY_PATH="${DUCKDB_LIB_DIR}:${DYLD_LIBRARY_PATH:-}"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    export LD_LIBRARY_PATH="${DUCKDB_LIB_DIR}:${LD_LIBRARY_PATH:-}"
fi

echo "🦆 Prebuilt DuckDB environment configured:"
echo "  Library: $DUCKDB_LIB_DIR"
echo "  Headers: $DUCKDB_INCLUDE_DIR"
echo "  Binary: $(ls -1 libduckdb.dylib 2>/dev/null | head -1)"
echo ""
echo "To use in your project:"
echo "  source $DUCKDB_LIB_DIR/setup_env.sh"
echo "  cargo build -p kcura-duck"