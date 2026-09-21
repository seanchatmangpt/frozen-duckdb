#!/bin/bash
# KCura Optimized DuckDB Build Script
# Pre-compiles Arrow and other heavy dependencies to speed up builds

set -e

echo "🚀 KCura Optimized DuckDB Build - Pre-compiling heavy dependencies..."

# Configuration
BUILD_DIR="target/duckdb-optimized"
ARROW_VERSION="56.2.0"
DUCKDB_VERSION="1.5.5"
# crates.io version of the `duckdb` crate that bundles DuckDB ${DUCKDB_VERSION};
# duckdb-rs encodes the engine version as 1.MAJOR_MINOR_PATCH.x since v1.5.0
DUCKDB_CRATE_VERSION="1.10505.0"

# Create build directory
mkdir -p "$BUILD_DIR"
cd "$BUILD_DIR"

# Function to download and extract a crate
download_crate() {
    local crate_name="$1"
    local version="$2"
    local url="https://crates.io/api/v1/crates/${crate_name}/${version}/download"
    
    echo "📦 Downloading ${crate_name} v${version}..."
    curl -L "$url" -o "${crate_name}-${version}.crate"
    tar -xf "${crate_name}-${version}.crate"
    mv "${crate_name}-${version}" "$crate_name"
}

# Pre-compile Arrow ecosystem crates
echo "🔧 Pre-compiling Arrow ecosystem..."

# Create a temporary Cargo.toml for Arrow compilation
cat > Cargo.toml << EOF
[package]
name = "arrow-precompile"
version = "0.1.0"
edition = "2021"

[dependencies]
arrow = { version = "$ARROW_VERSION", features = ["default"] }
arrow-array = "$ARROW_VERSION"
arrow-arith = "$ARROW_VERSION"
arrow-buffer = "$ARROW_VERSION"
arrow-cast = "$ARROW_VERSION"
arrow-data = "$ARROW_VERSION"
arrow-ord = "$ARROW_VERSION"
arrow-row = "$ARROW_VERSION"
arrow-schema = "$ARROW_VERSION"
arrow-select = "$ARROW_VERSION"
arrow-string = "$ARROW_VERSION"

[profile.release]
lto = "fat"
codegen-units = 1
opt-level = 3
panic = "abort"
strip = "symbols"
EOF

# Create a minimal main.rs
cat > src/main.rs << 'EOF'
fn main() {
    println!("Arrow pre-compilation complete");
}
EOF

mkdir -p src

# Pre-compile Arrow with all optimizations
echo "⚡ Compiling Arrow with maximum optimizations..."
export RUSTC_WRAPPER="sccache"
export RUSTFLAGS="-Ctarget-cpu=native -Ctarget-feature=+crt-static -Cembed-bitcode=yes"

cargo build --release --jobs $(nproc)

# Create optimized DuckDB build
echo "🦆 Creating optimized DuckDB build..."

# Create custom DuckDB Cargo.toml with all features
cat > ../duckdb-custom/Cargo.toml << EOF
[package]
name = "duckdb-custom"
version = "$DUCKDB_VERSION"
edition = "2021"

[dependencies]
duckdb = { version = "$DUCKDB_CRATE_VERSION", features = [
    "bundled",
    "json",
    "parquet", 
    "appender-arrow",
    "vtab-full",
    "extensions-full",
    "modern-full"
] }

# Use pre-compiled Arrow
arrow = { version = "$ARROW_VERSION", features = ["default"] }
arrow-array = "$ARROW_VERSION"
arrow-arith = "$ARROW_VERSION"
arrow-buffer = "$ARROW_VERSION"
arrow-cast = "$ARROW_VERSION"
arrow-data = "$ARROW_VERSION"
arrow-ord = "$ARROW_VERSION"
arrow-row = "$ARROW_VERSION"
arrow-schema = "$ARROW_VERSION"
arrow-select = "$ARROW_VERSION"
arrow-string = "$ARROW_VERSION"

[profile.release]
lto = "fat"
codegen-units = 1
opt-level = 3
panic = "abort"
strip = "symbols"
debug = "line-tables-only"

[profile.dev]
incremental = true
codegen-units = 4
debug = true
EOF

mkdir -p ../duckdb-custom/src
cat > ../duckdb-custom/src/main.rs << 'EOF'
fn main() {
    println!("DuckDB custom build ready");
}
EOF

echo "✅ Optimized DuckDB build configuration created!"
echo "📁 Build artifacts in: $BUILD_DIR"
echo "🦆 Custom DuckDB crate in: target/duckdb-custom"
echo ""
echo "To use the optimized build:"
echo "  cd target/duckdb-custom && cargo build --release"
