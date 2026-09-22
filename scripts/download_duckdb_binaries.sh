#!/bin/bash
# Download Pre-compiled DuckDB Binaries - No Compilation Ever!
# This downloads the official pre-compiled DuckDB binaries

set -e

echo "🦆 Downloading Pre-compiled DuckDB Binaries (No Compilation Ever!)"
echo "=================================================================="

# Configuration
DUCKDB_VERSION="1.5.5"
PREBUILT_DIR="target/duckdb-prebuilt"

# Create prebuilt directory
mkdir -p "$PREBUILT_DIR"
cd "$PREBUILT_DIR"

# Detect platform
# NOTE: since DuckDB v1.5.x, macOS libduckdb is published ONLY as
# libduckdb-osx-universal.zip (universal arm64+x86_64); the per-arch
# libduckdb-osx-arm64.zip / libduckdb-osx-amd64.zip assets no longer exist.
# duckdb_cli-osx-universal.zip still exists for the CLI. The zip extracts a
# single universal libduckdb.dylib, which serves both arches.
#
# Asset law (TR7): within frozen-duckdb, library files are named
# libduckdb_{arch}.dylib (macOS) / libduckdb_{arch}.so (Linux), arch names
# x86_64 | arm64 (aarch64 normalized to arm64, matching the builder's
# detect_architecture). Upstream zip assets keep their upstream names; the
# extracted library below is additionally linked under the asset-law name.
ARCH=$(uname -m)
if [[ "$ARCH" == "aarch64" ]]; then
    ARCH="arm64"
fi
if [[ "$OSTYPE" == "darwin"* ]]; then
    PLATFORM="osx-universal"
    LIB_EXT="dylib"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    if [[ "$ARCH" == "arm64" ]]; then
        PLATFORM="linux-arm64"
    else
        PLATFORM="linux-amd64"
    fi
    LIB_EXT="so"
else
    echo "❌ Unsupported platform: $OSTYPE"
    exit 1
fi

echo "📦 Downloading DuckDB binaries for $PLATFORM..."

# Download DuckDB CLI and library
DUCKDB_CLI_URL="https://github.com/duckdb/duckdb/releases/download/v${DUCKDB_VERSION}/duckdb_cli-${PLATFORM}.zip"
DUCKDB_LIB_URL="https://github.com/duckdb/duckdb/releases/download/v${DUCKDB_VERSION}/libduckdb-${PLATFORM}.zip"

echo "📥 Downloading DuckDB CLI..."
curl -L "$DUCKDB_CLI_URL" -o "duckdb_cli-${PLATFORM}.zip"
unzip -q "duckdb_cli-${PLATFORM}.zip"

echo "📥 Downloading DuckDB Library..."
curl -L "$DUCKDB_LIB_URL" -o "libduckdb-${PLATFORM}.zip"
unzip -q "libduckdb-${PLATFORM}.zip"

# Align with the frozen-duckdb asset law (TR7): expose the extracted library
# under the asset-law name libduckdb_${ARCH}.{dylib,so} alongside the upstream
# plain name. On macOS the upstream asset is universal, so the per-arch name
# links to that single universal library (no per-arch assets exist upstream);
# on Linux the extracted libduckdb.so is linked as libduckdb_${ARCH}.so so the
# builder's cache/prebuilt naming and -lduckdb consumers agree. The -f guards
# fail loudly instead of leaving a dangling symlink (set -e alone would not
# catch the ln target error).
if [[ "$OSTYPE" == "darwin"* ]]; then
    if [[ ! -f libduckdb.dylib ]]; then
        echo "❌ Expected libduckdb.dylib inside libduckdb-${PLATFORM}.zip" >&2
        exit 1
    fi
    ln -sf libduckdb.dylib "libduckdb_${ARCH}.dylib"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    if [[ ! -f libduckdb.so ]]; then
        echo "❌ Expected libduckdb.so inside libduckdb-${PLATFORM}.zip" >&2
        exit 1
    fi
    ln -sf libduckdb.so "libduckdb_${ARCH}.so"
fi

# Download headers
echo "📥 Downloading DuckDB headers..."
HEADERS_URL="https://github.com/duckdb/duckdb/archive/refs/tags/v${DUCKDB_VERSION}.tar.gz"
curl -L "$HEADERS_URL" -o "duckdb-${DUCKDB_VERSION}.tar.gz"
tar -xzf "duckdb-${DUCKDB_VERSION}.tar.gz"
cp -r "duckdb-${DUCKDB_VERSION}/src/include" .

# Create environment setup script
cat > setup_env.sh << 'EOF'
#!/bin/bash
# Setup environment for prebuilt DuckDB
export DUCKDB_LIB_DIR="$(dirname "$(realpath "$0")")"
export DUCKDB_INCLUDE_DIR="$(dirname "$(realpath "$0")")/include"

# Set library path for runtime
if [[ "$OSTYPE" == "darwin"* ]]; then
    export DYLD_FALLBACK_LIBRARY_PATH="$DUCKDB_LIB_DIR:$DYLD_FALLBACK_LIBRARY_PATH"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    export LD_LIBRARY_PATH="$DUCKDB_LIB_DIR:$LD_LIBRARY_PATH"
fi

echo "🦆 DuckDB prebuilt environment configured:"
echo "  Library: $DUCKDB_LIB_DIR"
echo "  Headers: $DUCKDB_INCLUDE_DIR"
echo "  Binary: $(ls -1 *.dylib *.so 2>/dev/null | head -1)"
echo ""
echo "To use in your project:"
echo "  source $DUCKDB_LIB_DIR/setup_env.sh"
echo "  cargo build --no-default-features"
EOF

chmod +x setup_env.sh

# Clean up
rm -f *.zip *.tar.gz
rm -rf "duckdb-${DUCKDB_VERSION}"

cd ../..

echo "✅ Pre-compiled DuckDB binaries downloaded!"
echo ""
echo "📁 Prebuilt package location: $PREBUILT_DIR"
echo "📊 Package contents:"
ls -la "$PREBUILT_DIR"
echo ""
echo "🎯 Usage Instructions:"
echo "  1. Source the environment: source $PREBUILT_DIR/setup_env.sh"
echo "  2. Update kcura-duck Cargo.toml to use system DuckDB"
echo "  3. Build: cargo build -p kcura-duck"
echo "  4. Enjoy fast builds forever! 🚀"
echo ""
echo "💡 This DuckDB binary:"
echo "  ✅ Pre-compiled (no compilation needed)"
echo "  ✅ Official DuckDB release"
echo "  ✅ All extensions included"
echo "  ✅ Ready to use immediately"
echo ""
echo "🔄 Never needs compilation again!"
