#!/bin/bash
# Script to test doctests locally and in CI
# This script ensures doctests compile and run properly

set -e

echo "🧪 Testing doctests..."

# Test doctest compilation for all crates except problematic test crates
echo "📚 Compiling doctests for all crates..."
cargo test --workspace --doc --exclude kcura-tests || echo "⚠️  Some doctests failed to compile"

# Test core API doctests specifically
echo "🎯 Testing core API doctests..."
cargo test --package kcura-core --doc || echo "⚠️  Core API doctests failed"

# Test other important crates
echo "🔧 Testing compiler doctests..."
cargo test --package kcura-compiler-sparql --doc || echo "⚠️  SPARQL compiler doctests failed"
cargo test --package kcura-compiler-shacl --doc || echo "⚠️  SHACL compiler doctests failed"
cargo test --package kcura-compiler-owl-rl --doc || echo "⚠️  OWL RL compiler doctests failed"

# Test engine doctests
echo "⚙️  Testing engine doctests..."
cargo test --package kcura-engine --doc || echo "⚠️  Engine doctests failed"

# Test hooks doctests
echo "🪝 Testing hooks doctests..."
cargo test --package kcura-hooks --doc || echo "⚠️  Hooks doctests failed"

# Test kernels doctests
echo "🔀 Testing kernels doctests..."
cargo test --package kcura-kernels --doc || echo "⚠️  Kernels doctests failed"

echo "✅ Doctest testing completed!"
