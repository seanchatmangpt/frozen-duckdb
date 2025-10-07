#!/usr/bin/env bash
set -euo pipefail

# Spec/code sync check script
# Ensures symbols found in code are documented and vice versa

echo "🔍 Checking spec/code synchronization..."

# Check that all public errors are documented
echo "📋 Checking error code documentation..."

# Extract error codes from docs/specs/errors.yaml
if [ -f "docs/specs/errors.yaml" ]; then
  documented_errors=$(grep -E "^[A-Z][0-9]+:" docs/specs/errors.yaml | cut -d: -f1 | sort)
else
  echo "❌ docs/specs/errors.yaml not found"
  exit 1
fi

# Extract error codes from Rust code
code_errors=$(grep -rE "ErrorCode::[A-Z][0-9]+" crates/ | grep -oE "[A-Z][0-9]+" | sort -u)

# Check for undocumented errors
undocumented=$(comm -23 <(echo "$code_errors") <(echo "$documented_errors"))
if [ -n "$undocumented" ]; then
  echo "❌ Undocumented error codes found:"
  echo "$undocumented"
  echo "🚫 BLOCKING: All error codes must be documented in docs/specs/errors.yaml"
  exit 1
fi

# Check for unused documented errors
unused=$(comm -13 <(echo "$code_errors") <(echo "$documented_errors"))
if [ -n "$unused" ]; then
  echo "⚠️  Documented but unused error codes:"
  echo "$unused"
  echo "Consider removing from docs/specs/errors.yaml"
fi

echo "✅ Error code documentation is synchronized"

# Check that all public API functions are documented
echo "📚 Checking API documentation..."

# Extract public functions from kcura-core
if [ -f "crates/kcura-core/src/api.rs" ]; then
  api_functions=$(grep -E "^pub fn|^pub async fn" crates/kcura-core/src/api.rs | grep -oE "fn [a-zA-Z_][a-zA-Z0-9_]*" | cut -d' ' -f2 | sort)
else
  echo "❌ crates/kcura-core/src/api.rs not found"
  exit 1
fi

# Check if functions are documented in USER_GUIDE
if [ -f "docs/USER_GUIDE.md" ]; then
  for func in $api_functions; do
    if ! grep -q "$func" docs/USER_GUIDE.md; then
      echo "⚠️  Function $func not documented in USER_GUIDE.md"
    fi
  done
else
  echo "❌ docs/USER_GUIDE.md not found"
  exit 1
fi

echo "✅ API documentation check completed"

# Check that all FFI functions have corresponding Rust implementations
echo "🔗 Checking FFI/Rust implementation sync..."

if [ -f "include/kcura.h" ]; then
  ffi_functions=$(grep -E "^[a-zA-Z_][a-zA-Z0-9_]*\s+kc_" include/kcura.h | grep -oE "kc_[a-zA-Z_][a-zA-Z0-9_]*" | sort)
else
  echo "❌ include/kcura.h not found"
  exit 1
fi

# Check if FFI functions have Rust implementations
for ffi_func in $ffi_functions; do
  rust_func="${ffi_func#kc_}"  # Remove kc_ prefix
  if ! grep -r "fn $rust_func" crates/kcura-ffi/src/ >/dev/null 2>&1; then
    echo "❌ FFI function $ffi_func has no Rust implementation"
    exit 1
  fi
done

echo "✅ FFI/Rust implementation sync check passed"

# Check that all hook types are documented
echo "🎣 Checking hook documentation..."

if [ -f "crates/kcura-hooks/src/lib.rs" ]; then
  hook_types=$(grep -E "pub enum|pub struct" crates/kcura-hooks/src/lib.rs | grep -oE "[A-Z][a-zA-Z0-9_]*" | sort)
else
  echo "❌ crates/kcura-hooks/src/lib.rs not found"
  exit 1
fi

if [ -f "docs/HOOKS_RUNTIME_V1.md" ]; then
  for hook_type in $hook_types; do
    if ! grep -q "$hook_type" docs/HOOKS_RUNTIME_V1.md; then
      echo "⚠️  Hook type $hook_type not documented in HOOKS_RUNTIME_V1.md"
    fi
  done
else
  echo "❌ docs/HOOKS_RUNTIME_V1.md not found"
  exit 1
fi

echo "✅ Hook documentation check completed"

# Check that all compiler features are documented
echo "🔧 Checking compiler documentation..."

if [ -f "docs/COMPILER_V1.md" ]; then
  compiler_features=$(grep -rE "pub fn|pub struct|pub enum" crates/kcura-compiler*/src/ | grep -oE "[a-zA-Z_][a-zA-Z0-9_]*" | sort -u)
  
  for feature in $compiler_features; do
    if ! grep -q "$feature" docs/COMPILER_V1.md; then
      echo "⚠️  Compiler feature $feature not documented in COMPILER_V1.md"
    fi
  done
else
  echo "❌ docs/COMPILER_V1.md not found"
  exit 1
fi

echo "✅ Compiler documentation check completed"

echo "🎉 Spec/code sync check passed!"
