#!/bin/bash
# Setup environment for frozen DuckDB binary with architecture detection
export DUCKDB_LIB_DIR="$(dirname "$(realpath "$0")")"
export DUCKDB_INCLUDE_DIR="$(dirname "$(realpath "$0")")"

# Detect architecture and choose appropriate binary
ARCH=${ARCH:-$(uname -m)}
if [[ "$ARCH" == "x86_64" ]]; then
    DUCKDB_LIB="libduckdb_x86_64.dylib"
    echo "🖥️  Detected x86_64 architecture, using 55MB binary"
elif [[ "$ARCH" == "arm64" ]]; then
    DUCKDB_LIB="libduckdb_arm64.dylib"
    echo "🍎 Detected Apple Silicon (arm64), using 50MB binary"
else
    DUCKDB_LIB="libduckdb.dylib"
    echo "⚠️  Unknown architecture ($ARCH), using universal binary (105MB)"
fi

# Create symlinks for compatibility
ln -sf "$DUCKDB_LIB" "$DUCKDB_LIB_DIR/libduckdb.dylib"
ln -sf "$DUCKDB_LIB" "$DUCKDB_LIB_DIR/libduckdb.1.dylib"
ln -sf "$DUCKDB_LIB" "$DUCKDB_LIB_DIR/libduckdb.1.4.dylib"

# Set library path for runtime and build time
if [[ "$OSTYPE" == "darwin"* ]]; then
    export DYLD_FALLBACK_LIBRARY_PATH="${DUCKDB_LIB_DIR}:${DYLD_FALLBACK_LIBRARY_PATH:-}"
    export DYLD_LIBRARY_PATH="${DUCKDB_LIB_DIR}:${DYLD_LIBRARY_PATH:-}"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    export LD_LIBRARY_PATH="${DUCKDB_LIB_DIR}:${LD_LIBRARY_PATH:-}"
fi

echo "🦆 Frozen DuckDB environment configured:"
echo "  Library: $DUCKDB_LIB_DIR"
echo "  Headers: $DUCKDB_INCLUDE_DIR"
echo "  Binary: $DUCKDB_LIB ($(ls -lah "$DUCKDB_LIB_DIR/$DUCKDB_LIB" | awk '{print $5}'))"
echo "  Architecture: $ARCH"
echo ""
echo "To use in your project:"
echo "  source $DUCKDB_LIB_DIR/setup_env.sh"
echo "  cargo build"