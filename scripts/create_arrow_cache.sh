#!/bin/bash
# KCura Arrow Cache Creation Script
# Creates a local cache of pre-compiled Arrow dependencies

set -e

echo "🎯 Creating Arrow dependency cache for faster builds..."

CACHE_DIR="target/arrow-cache"
ARROW_VERSION="56.2.0"

# Create cache directory
mkdir -p "$CACHE_DIR"
cd "$CACHE_DIR"

# Create a workspace for pre-compiling Arrow
cat > Cargo.toml << EOF
[workspace]
members = ["arrow-cache"]

[workspace.dependencies]
arrow = "56.2.0"
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

# Create the cache crate
mkdir -p arrow-cache/src
cat > arrow-cache/Cargo.toml << EOF
[package]
name = "arrow-cache"
version = "0.1.0"
edition = "2021"

[dependencies]
arrow = { workspace = true, features = ["default"] }
arrow-array = { workspace = true }
arrow-arith = { workspace = true }
arrow-buffer = { workspace = true }
arrow-cast = { workspace = true }
arrow-data = { workspace = true }
arrow-ord = { workspace = true }
arrow-row = { workspace = true }
arrow-schema = { workspace = true }
arrow-select = { workspace = true }
arrow-string = { workspace = true }
EOF

cat > arrow-cache/src/lib.rs << 'EOF'
// Arrow cache library - pre-compiled Arrow dependencies
pub use arrow::*;
pub use arrow_array::*;
pub use arrow_arith::*;
pub use arrow_buffer::*;
pub use arrow_cast::*;
pub use arrow_data::*;
pub use arrow_ord::*;
pub use arrow_row::*;
pub use arrow_schema::*;
pub use arrow_select::*;
pub use arrow_string::*;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_arrow_imports() {
        // Test that all Arrow modules can be imported
        let _schema = arrow_schema::Schema::empty();
        println!("Arrow cache working correctly");
    }
}
EOF

# Pre-compile with maximum optimizations
echo "⚡ Pre-compiling Arrow dependencies..."
export RUSTC_WRAPPER="sccache"
export RUSTFLAGS="-Ctarget-cpu=native -Ctarget-feature=+crt-static -Cembed-bitcode=yes"

cargo build --release
cargo build --release --tests

echo "✅ Arrow cache created successfully!"
echo "📁 Cache location: $CACHE_DIR"
echo "📦 Pre-compiled Arrow dependencies ready for use"
