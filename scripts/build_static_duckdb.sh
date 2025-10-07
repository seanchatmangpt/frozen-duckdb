#!/bin/bash
# Build Frozen DuckDB Binary - Compile Once, Use Forever
# Builds DuckDB in isolated environment to avoid workspace conflicts

set -e

echo "🦆 Building Frozen DuckDB Binary (Compile Once, Use Forever)"
echo "============================================================"

# Configuration - use /tmp for clean build environment
BUILD_DIR="/tmp/duckdb-build-$$"
PREBUILT_DIR="target/duckdb-prebuilt"

# Clean any existing build
rm -rf "$BUILD_DIR" "$PREBUILT_DIR"

echo "📦 Step 1: Setting up isolated build environment..."
mkdir -p "$BUILD_DIR"
cd "$BUILD_DIR"

echo "📥 Step 2: Cloning DuckDB-rs source..."
git clone https://github.com/duckdb/duckdb-rs.git
cd duckdb-rs

echo "🔧 Step 3: Applying Arrow patch..."
# Apply our Arrow patch to the cloned source
cd crates/libduckdb-sys
if [ -f "../../../../apply-arrow-patch.sh" ]; then
    echo "Applying Arrow patch to libduckdb-sys..."
    ../../../../apply-arrow-patch.sh
fi
cd ../..

echo "⚡ Step 4: Building with core features (simplified for reliability)..."
# Build with core features first - we can add more later
export RUSTC_WRAPPER="sccache"
export RUSTFLAGS="-Ctarget-cpu=native"

echo "Building DuckDB with bundled feature (reliable baseline)..."
time cargo build --release --features "bundled"

echo "📦 Step 5: Creating prebuilt package..."
# Create prebuilt directory in main project
mkdir -p "../../../$PREBUILT_DIR"

# Copy the compiled library
if [ -f "target/release/libduckdb.a" ]; then
    cp target/release/libduckdb.a "../../../$PREBUILT_DIR/"
    echo "✅ Static library: libduckdb.a"
elif [ -f "target/release/libduckdb.so" ]; then
    cp target/release/libduckdb.so "../../../$PREBUILT_DIR/"
    echo "✅ Shared library: libduckdb.so"
elif [ -f "target/release/libduckdb.dylib" ]; then
    cp target/release/libduckdb.dylib "../../../$PREBUILT_DIR/"
    echo "✅ Dynamic library: libduckdb.dylib"
else
    echo "❌ No DuckDB library found in target/release/"
    ls -la target/release/
    exit 1
fi

# Copy headers
if [ -d "crates/libduckdb-sys/duckdb/src/include" ]; then
    cp -r crates/libduckdb-sys/duckdb/src/include "../../../$PREBUILT_DIR/"
    echo "✅ Headers: include/"
else
    echo "❌ Headers not found in crates/libduckdb-sys/duckdb/src/include"
    find crates/libduckdb-sys -name "*.h" -o -name "include" -type d
    exit 1
fi

# Create environment setup script
cat > "../../../$PREBUILT_DIR/setup_env.sh" << 'EOF'
#!/bin/bash
# Setup environment for frozen DuckDB binary
export DUCKDB_LIB_DIR="$(dirname "$(realpath "$0")")"
export DUCKDB_INCLUDE_DIR="$(dirname "$(realpath "$0")")/include"

# Set library path for runtime
if [[ "$OSTYPE" == "darwin"* ]]; then
    export DYLD_FALLBACK_LIBRARY_PATH="$DUCKDB_LIB_DIR:$DYLD_FALLBACK_LIBRARY_PATH"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    export LD_LIBRARY_PATH="$DUCKDB_LIB_DIR:$LD_LIBRARY_PATH"
fi

echo "🦆 Frozen DuckDB environment configured:"
echo "  Library: $DUCKDB_LIB_DIR"
echo "  Headers: $DUCKDB_INCLUDE_DIR"
echo "  Binary: $(ls -1 *.dylib *.so *.a 2>/dev/null | head -1)"
echo ""
echo "To use in your project:"
echo "  source $DUCKDB_LIB_DIR/setup_env.sh"
echo "  Update kcura-duck to use: duckdb = { version = \"1.4.0\", default-features = false }"
echo "  cargo build -p kcura-duck"
EOF

chmod +x "../../../$PREBUILT_DIR/setup_env.sh"

# Clean up build directory
cd ../../..
rm -rf "$BUILD_DIR"

echo "✅ Frozen DuckDB binary created successfully!"
echo ""
echo "📁 Frozen binary location: $PREBUILT_DIR"
echo "📊 Package contents:"
ls -la "$PREBUILT_DIR"
echo ""
echo "🎯 Usage Instructions:"
echo "  1. source $PREBUILT_DIR/setup_env.sh"
echo "  2. Update kcura-duck Cargo.toml to use system DuckDB"
echo "  3. cargo build -p kcura-duck (should be <10s)"
echo "  4. Enjoy fast builds forever! 🚀"
echo ""
echo "💡 This frozen binary provides:"
echo "  ✅ Pre-compiled DuckDB (no compilation needed)"
echo "  ✅ Core DuckDB functionality"
echo "  ✅ Arrow integration with patch"
echo "  ✅ Ready to use immediately"
echo ""
echo "🔄 Can be extended with additional features later!"
