#!/usr/bin/env bash
set -euo pipefail

# Fake-guard script: kills stubs/fakes on sight
# This script ensures no placeholder implementations make it to production

echo "🔍 Scanning for fake/stub code patterns..."

# Patterns that indicate fake/stub implementations
pat='(unimplemented!|todo!|panic!\("stub|fake\)|return\s+Ok\(true\);\s*//\s*TODO|dummy_|hardcoded_|Fake|AlwaysReturns|pass_through|NotYetImplemented)'

# Search for fake patterns in library code
hits=$(git grep -nE "$pat" -- 'crates/**/src/*.rs' || true)

if [ -n "$hits" ]; then
  echo "❌ Fake/stub code detected:"
  echo "$hits"
  echo ""
  echo "🚫 BLOCKING: Remove all fake implementations before merge"
  echo "   Required: Real functionality, not hardcoded responses"
  exit 1
fi

echo "✅ No fake patterns detected"

# Check for required symbols that must exist in core crates
echo "🔍 Verifying required symbols exist..."

req=(extract_owl_classes extract_owl_properties extract_shacl_shapes reason_rl verify_receipt materialize_full materialize_incremental)

missing_symbols=()
for sym in "${req[@]}"; do
  if ! git grep -n "$sym(" 'crates/**/src/*.rs' >/dev/null 2>&1; then
    missing_symbols+=("$sym")
  fi
done

if [ ${#missing_symbols[@]} -gt 0 ]; then
  echo "❌ Missing required symbols:"
  printf '   - %s\n' "${missing_symbols[@]}"
  echo ""
  echo "🚫 BLOCKING: All required symbols must be implemented"
  exit 1
fi

echo "✅ All required symbols found"

# Check for TODO/XXX in library code (not allowed)
echo "🔍 Checking for TODO/XXX in library code..."

todos=$(git grep -nE "(TODO|XXX|FIXME)" -- 'crates/**/src/*.rs' || true)

if [ -n "$todos" ]; then
  echo "❌ TODO/XXX found in library code:"
  echo "$todos"
  echo ""
  echo "🚫 BLOCKING: Remove all TODO/XXX from library code"
  exit 1
fi

echo "✅ No TODO/XXX in library code"

# Check for unwrap()/expect() in library code (not allowed)
echo "🔍 Checking for unwrap/expect in library code..."

# Only check library source files (not tests)
unwraps=$(git grep -nE "(\.unwrap\(|\.expect\()" -- 'crates/kcura-*/src/lib.rs' || true)

if [ -n "$unwraps" ]; then
  echo "❌ unwrap/expect found in library code:"
  echo "$unwraps"
  echo ""
  echo "🚫 BLOCKING: Use proper error handling, not unwrap/expect"
  exit 1
fi

echo "✅ No unwrap/expect in library code"

echo "🎉 Fake-guard passed: All code appears to be real implementations"
