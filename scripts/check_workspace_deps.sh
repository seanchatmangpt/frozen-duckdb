#!/bin/bash
# KCura Workspace Dependency Management Script
# This script ensures all crates use workspace dependencies to prevent duplicate builds

set -euo pipefail

echo "🔍 Checking for dependency inconsistencies..."

# Find all Cargo.toml files in crates
CRATE_FILES=$(find crates -name "Cargo.toml")

# Dependencies that should always use workspace versions
WORKSPACE_DEPS=(
    "anyhow"
    "thiserror"
    "serde"
    "serde_json"
    "serde_yaml"
    "rand"
    "fake"
    "uuid"
    "time"
    "sha3"
    "blake3"
    "merkle-lite"
    "ed25519-dalek"
    "rayon"
    "moka"
    "parking_lot"
    "itertools"
    "regex"
    "dashmap"
    "tracing"
    "tracing-subscriber"
    "opentelemetry"
    "opentelemetry-otlp"
    "tracing-opentelemetry"
    "opentelemetry-metrics"
    "opentelemetry-sdk"
    "tokio"
    "duckdb"
    "proptest"
    "lazy_static"
    "percent-encoding"
    "criterion"
    "hdrhistogram"
    "tempfile"
    "csv"
    "cbindgen"
    "cc"
    "hex"
    "clap"
    "chrono"
    "reqwest"
    "arrow"
    "arrow-arith"
    "arrow-array"
    "arrow-buffer"
    "arrow-cast"
    "arrow-data"
    "arrow-ord"
    "arrow-row"
    "arrow-schema"
    "arrow-select"
    "arrow-string"
)

# Check for direct version specifications
echo "📋 Checking for direct version specifications..."
ISSUES_FOUND=0

for crate_file in $CRATE_FILES; do
    echo "Checking $crate_file..."
    
    for dep in "${WORKSPACE_DEPS[@]}"; do
        # Look for direct version specifications (not workspace = true)
        if grep -q "$dep.*version.*=" "$crate_file" && ! grep -q "$dep.*workspace.*true" "$crate_file"; then
            echo "❌ ISSUE: $crate_file has direct version for $dep"
            ISSUES_FOUND=1
        fi
    done
done

if [ $ISSUES_FOUND -eq 0 ]; then
    echo "✅ All dependencies are using workspace versions"
else
    echo "❌ Found dependency inconsistencies. Run 'cargo clean' and rebuild to consolidate."
    exit 1
fi

# Check for duplicate dependency builds
echo "🔍 Checking for duplicate builds..."
BUILD_DIRS=$(find target/debug/build -name "libduckdb-sys-*" 2>/dev/null | wc -l)
if [ "$BUILD_DIRS" -gt 1 ]; then
    echo "❌ Found $BUILD_DIRS DuckDB builds - this indicates dependency conflicts"
    echo "💡 Run 'cargo clean' and rebuild to consolidate"
    exit 1
else
    echo "✅ No duplicate DuckDB builds found"
fi

echo "🎉 Workspace dependency management check passed!"
