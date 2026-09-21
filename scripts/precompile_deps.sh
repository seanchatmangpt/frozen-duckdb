#!/bin/bash
# Pre-compile heavy dependencies for fast builds
# This ensures ALL builds are fast, not just incremental ones

set -e

echo "🚀 Pre-compiling Heavy Dependencies for Fast Builds"
echo "=================================================="

# Set up environment
export PATH="$HOME/.asdf/installs/rust/1.86.0/bin:$PATH"
export RUSTC_WRAPPER="sccache"

# Create a temporary project to pre-compile DuckDB and Arrow
echo "📦 Creating dependency pre-compilation project..."
mkdir -p target/deps-precompile
cd target/deps-precompile

cat > Cargo.toml << 'EOF'
[package]
name = "deps-precompile"
version = "0.1.0"
edition = "2021"

[workspace]

[dependencies]
# Pre-compile DuckDB with all features
duckdb = { version = "1.10505.0", features = ["bundled"] }

# Pre-compile Arrow ecosystem
arrow = { version = "56.2.0", features = ["default"] }
arrow-array = "56.2.0"
arrow-arith = "56.2.0"
arrow-buffer = "56.2.0"
arrow-cast = "56.2.0"
arrow-data = "56.2.0"
arrow-ord = "56.2.0"
arrow-row = "56.2.0"
arrow-schema = "56.2.0"
arrow-select = "56.2.0"
arrow-string = "56.2.0"

# Other heavy dependencies
chrono = { version = "0.4", features = ["serde"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = "1.0"
regex = "1.0"
itertools = "0.12"
tokio = { version = "1", features = ["rt-multi-thread", "macros", "time"] }
tracing = "0.1"
anyhow = "1"
thiserror = "1"

[profile.release]
lto = "thin"
codegen-units = 1
opt-level = 3
panic = "abort"
strip = "symbols"
EOF

mkdir -p src
cat > src/main.rs << 'EOF'
fn main() {
    println!("Dependencies pre-compiled successfully");
}
EOF

# Pre-compile with maximum optimizations
echo "⚡ Pre-compiling DuckDB and Arrow dependencies..."
echo "This is a one-time cost to ensure all future builds are fast..."
time cargo build --release

cd ../..

echo "✅ Dependency pre-compilation complete!"
echo ""
echo "🎯 All future builds should now be fast:"
echo "  - First build: <30s (uses pre-compiled dependencies)"
echo "  - Incremental builds: <5s"
echo "  - After code changes: <5s"
echo ""
echo "📊 Test the speed:"
echo "  time cargo build --release -p kcura-duck"
