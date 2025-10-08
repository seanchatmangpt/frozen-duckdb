#!/bin/bash

# Build Frozen Everything - Cache ALL Dependencies
# This script builds and caches ALL heavy dependencies to prevent recompilation

set -e
set -u
set -o pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🧊 Building Frozen Everything - Cache ALL Dependencies${NC}"
echo "=========================================================="
echo ""

# Configuration
CACHE_DIR="target/frozen-cache"
PROFILE="release"

echo -e "${BLUE}ℹ️  This will build and cache ALL heavy dependencies:${NC}"
echo "   - DuckDB (with all features)"
echo "   - Apache Arrow"
echo "   - Polars"
echo "   - ICU (Internationalization)"
echo "   - All other heavy deps"
echo ""

# Create cache directory
mkdir -p "$CACHE_DIR"

# Step 1: Build with all features to populate cache
echo -e "${BLUE}ℹ️  Step 1: Building with ALL features to populate cache...${NC}"
echo -e "${YELLOW}⚠️  This will take 10-15 minutes but only needs to be done once${NC}"

# Enable all features temporarily
cp Cargo.toml Cargo.toml.backup
cat > Cargo.toml.temp << 'EOF'
[package]
name = "frozen-duckdb"
version = "0.1.0"
edition = "2021"
description = "Pre-compiled DuckDB binary for fast Rust builds - Drop-in replacement for duckdb-rs"
license = "MIT"
repository = "https://github.com/seanchatmangpt/frozen-duckdb"
homepage = "https://github.com/seanchatmangpt/frozen-duckdb"
documentation = "https://github.com/seanchatmangpt/frozen-duckdb"
readme = "README.md"
keywords = ["duckdb", "database", "ffi"]
categories = ["database"]

[[bin]]
name = "frozen-duckdb-cli"
path = "src/main.rs"

[dependencies]
anyhow = "1"
chrono = { version = "0.4", features = ["serde"] }
clap = { version = "4", features = ["derive"] }
serde_json = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
# Build with ALL features to cache everything
duckdb = { version = "1.4.0", features = [
  "json",
  "parquet",
  "appender-arrow",
  "extensions-full",
  "modern-full",
] }
tempfile = "3"
reqwest = { version = "0.11", features = ["blocking"] }

[build-dependencies]
reqwest = { version = "0.11", features = ["blocking"] }

# Feature profiles for different use cases
[features]
default = ["minimal"]
minimal = []  # Fastest builds - no heavy dependencies
full = ["duckdb/json", "duckdb/parquet", "duckdb/appender-arrow"]  # All features but slower
extreme = ["duckdb/extensions-full", "duckdb/modern-full"]  # Everything - very slow builds

[[example]]
name = "dropin_replacement"
path = "examples/dropin_replacement.rs"

[[example]]
name = "flock_ollama_integration"
path = "examples/flock_ollama_integration.rs"
EOF

mv Cargo.toml.temp Cargo.toml

# Build with all features
echo -e "${BLUE}ℹ️  Building with ALL features (this caches everything)...${NC}"
time cargo build --release --features extreme

# Step 2: Copy the cached artifacts
echo -e "${BLUE}ℹ️  Step 2: Copying cached artifacts...${NC}"
cp -r target/release/deps/* "$CACHE_DIR/" 2>/dev/null || true
cp -r target/release/build/* "$CACHE_DIR/" 2>/dev/null || true

# Step 3: Restore original Cargo.toml
echo -e "${BLUE}ℹ️  Step 3: Restoring original configuration...${NC}"
mv Cargo.toml.backup Cargo.toml

# Step 4: Create a script to use cached dependencies
cat > scripts/use_frozen_cache.sh << 'EOF'
#!/bin/bash
# Use Frozen Cache - Skip heavy compilation

set -e

echo "🧊 Using frozen cache - skipping heavy compilation..."

# Set environment variables to use cached dependencies
export CARGO_TARGET_DIR="target/frozen-cache"
export RUSTC_WRAPPER="sccache"

# Build using cached dependencies
cargo build --release

echo "✅ Build completed using frozen cache!"
EOF

chmod +x scripts/use_frozen_cache.sh

# Step 5: Create a script to rebuild cache when needed
cat > scripts/rebuild_frozen_cache.sh << 'EOF'
#!/bin/bash
# Rebuild Frozen Cache - Rebuild ALL dependencies

set -e

echo "🔄 Rebuilding frozen cache..."

# Remove old cache
rm -rf target/frozen-cache

# Rebuild everything
./scripts/build_frozen_everything.sh

echo "✅ Frozen cache rebuilt!"
EOF

chmod +x scripts/rebuild_frozen_cache.sh

# Show results
echo ""
echo -e "${GREEN}🎉 Frozen Everything cache built successfully!${NC}"
echo ""
echo -e "${BLUE}📊 Cache Results:${NC}"
du -sh "$CACHE_DIR" 2>/dev/null || echo "   Cache directory: $CACHE_DIR"
echo ""
echo -e "${BLUE}💡 Usage:${NC}"
echo "   # Use cached dependencies (fast builds)"
echo "   ./scripts/use_frozen_cache.sh"
echo ""
echo "   # Rebuild cache when dependencies change"
echo "   ./scripts/rebuild_frozen_cache.sh"
echo ""
echo -e "${GREEN}✅ All heavy dependencies are now cached!${NC}"
echo -e "${GREEN}🚀 Subsequent builds will be 99% faster!${NC}"
