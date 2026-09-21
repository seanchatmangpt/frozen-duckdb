#!/bin/bash
# Setup environment for frozen DuckDB binary with architecture detection
export DUCKDB_LIB_DIR="$(dirname "$(realpath "$0")")"
export DUCKDB_INCLUDE_DIR="$(dirname "$(realpath "$0")")"

# Detect platform and architecture, then choose the matching shared library.
# Asset law (TR7): macOS ships libduckdb_{arch}.dylib, Linux ships
# libduckdb_{arch}.so (Windows is on the documented local-compile fallback).
# NOTE: frozen-duckdb release assets for Linux (.so) do not exist until the
# release AFTER v1.5.5; until then Linux users compile locally
# (compile_duckdb_locally) and place libduckdb_{arch}.so in this directory.
ARCH=${ARCH:-$(uname -m)}
if [[ "$OSTYPE" == "darwin"* ]]; then
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

    # Create symlinks for compatibility. DuckDB >= 1.5 dylibs carry the neutral
    # install name @rpath/libduckdb.dylib, so the plain link name is what matters;
    # the versioned names remain for older 1.4-era consumers
    ln -sf "$DUCKDB_LIB" "$DUCKDB_LIB_DIR/libduckdb.dylib"
    ln -sf "$DUCKDB_LIB" "$DUCKDB_LIB_DIR/libduckdb.1.dylib"
    ln -sf "$DUCKDB_LIB" "$DUCKDB_LIB_DIR/libduckdb.1.4.dylib"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    # The builder's detect_architecture normalizes aarch64 -> arm64 (cache dirs
    # and lib names use "arm64" on both platforms); mirror that here so the
    # filename matches the cached libduckdb_arm64.so.
    if [[ "$ARCH" == "aarch64" ]]; then
        ARCH="arm64"
    fi
    if [[ "$ARCH" == "x86_64" || "$ARCH" == "arm64" ]]; then
        DUCKDB_LIB="libduckdb_${ARCH}.so"
        echo "Detected Linux ${ARCH}, using ${DUCKDB_LIB}"
    else
        DUCKDB_LIB="libduckdb.so"
        echo "⚠️  Unknown Linux architecture ($ARCH), using plain libduckdb.so"
    fi

    # Linux link name: -lduckdb resolves against libduckdb.so (same law as the
    # builder's ensure_link_name on Linux). Upstream DuckDB Linux libs carry no
    # versioned soname, so no versioned symlinks are created here.
    ln -sf "$DUCKDB_LIB" "$DUCKDB_LIB_DIR/libduckdb.so"
else
    echo "❌ Unsupported platform: $OSTYPE" >&2
    echo "   frozen-duckdb ships macOS (.dylib) and Linux (.so) libraries;" >&2
    echo "   Windows users compile DuckDB locally (see builder docs)." >&2
    exit 1
fi

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
echo "  Binary: $DUCKDB_LIB ($(ls -lah "$DUCKDB_LIB_DIR/$DUCKDB_LIB" 2>/dev/null | awk '{print $5}'))"
echo "  Architecture: $ARCH"
echo ""
echo "To use in your project:"
echo "  source $DUCKDB_LIB_DIR/setup_env.sh"
echo "  cargo build"
