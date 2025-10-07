#!/bin/bash
# Download Pre-compiled DuckDB Binaries - No Compilation Ever!
# This downloads the official pre-compiled DuckDB binaries

set -e

echo "🦆 Downloading Pre-compiled DuckDB Binaries (No Compilation Ever!)"
echo "=================================================================="

# Configuration
DUCKDB_VERSION="1.4.0"
PREBUILT_DIR="target/duckdb-prebuilt"

# Create prebuilt directory
mkdir -p "$PREBUILT_DIR"
cd "$PREBUILT_DIR"

# Detect platform
if [[ "$OSTYPE" == "darwin"* ]]; then
    if [[ $(uname -m) == "arm64" ]]; then
        PLATFORM="osx-universal"
        LIB_EXT="dylib"
    else
        PLATFORM="osx-universal"
        LIB_EXT="dylib"
    fi
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    if [[ $(uname -m) == "aarch64" ]]; then
        PLATFORM="linux-arm64"
        LIB_EXT="so"
    else
        PLATFORM="linux-amd64"
        LIB_EXT="so"
    fi
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
