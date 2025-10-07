#!/bin/bash
# KCura Release Checklist Script
# Run this before creating a release to ensure all quality gates pass

set -euo pipefail

echo "🧪 Running KCura Release Checklist..."

# Clean start
echo "🧹 Cleaning workspace..."
cargo clean

# Update dependencies
echo "📦 Updating dependencies..."
cargo update -w

# Verify metadata is valid
echo "📋 Validating metadata..."
cargo metadata --format-version=1 > /dev/null

# Format check
echo "🎨 Checking formatting..."
cargo fmt --all -- --check

# Clippy with strict warnings
echo "🔍 Running clippy..."
cargo clippy --all-targets -- -D warnings

# Test suite
echo "🧪 Running test suite..."
cargo test --workspace

# Coverage check
echo "📊 Running coverage..."
cargo install cargo-llvm-cov --locked
RUSTFLAGS="-Cdebuginfo=2" cargo llvm-cov --workspace --lcov --output-path lcov.info

# Security audit
echo "🔒 Running security audit..."
cargo install cargo-deny --locked
cargo deny check advisories

# Check conventional commits compliance
echo "🔍 Checking conventional commits..."
if [ -f ".commitlintrc.json" ]; then
    echo "✅ Conventional commits configuration found"
else
    echo "❌ No conventional commits configuration found"
    exit 1
fi

# Generate changelog (if conventional commits is set up)
echo "📝 Checking changelog..."
if [ -f "scripts/generate-changelog.sh" ]; then
    scripts/generate-changelog.sh
else
    echo "ℹ️  No changelog generator found, skipping..."
fi

# Documentation build
echo "📚 Building documentation..."
cargo doc --workspace --no-deps

# Release build test
echo "🚀 Testing release build..."
cargo build --release --workspace

# Feature matrix check
echo "🔧 Checking feature matrix..."
cargo install cargo-hack --locked
cargo hack check --workspace --each-feature --no-dev-deps

# Public API diff check
echo "🔌 Checking public API..."
cargo install cargo-public-api --locked
cargo public-api --diff-git-checks

# Benchmark smoke test
echo "⚡ Running benchmark smoke test..."
cargo bench --bench kcura_benchmarks -- --warm-up-time 1 --measurement-time 2

# Examples validation
echo "💡 Running examples..."
if [ -f "examples/quick-start.sh" ]; then
    bash examples/quick-start.sh
else
    echo "ℹ️  No examples script found, skipping..."
fi

# Doctest compilation
echo "📖 Checking doctests..."
cargo test --workspace --doc

echo ""
echo "✅ All release checks passed!"
echo ""
echo "Next steps:"
echo "1. Review the generated documentation: cargo doc --open"
echo "2. Update CHANGELOG.md with release notes"
echo "3. Tag the release: git tag vX.Y.Z"
echo "4. Push the tag: git push origin vX.Y.Z"
echo "5. Create GitHub release with release notes"
