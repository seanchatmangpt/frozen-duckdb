#!/usr/bin/env bash
set -euo pipefail

echo "=== KCura CI Gates ==="

# Gate 1: Deny forbidden tokens
echo "Checking for forbidden tokens..."
if rg -n 'FAKE_OK|HARDCODED|DUMMY|STUB|PLACEHOLDER' | grep -q .; then
    echo "::error ::Forbidden tokens found in codebase"
    exit 1
fi
echo "✓ No forbidden tokens found"

# Gate 2: Forbid trivially-constant FFI returns
echo "Checking FFI functions for constant returns..."
if rg -n 'pub extern "C"\s+fn\s+kc_.*\{' crates/kcura-ffi/src/lib.rs | grep -q .; then
    if rg -n 'pub extern "C"\s+fn\s+kc_.*\{([^}]|\n)*return\s+Ok\([^)]*\);\s*\}' crates/kcura-ffi/src/lib.rs | grep -q .; then
        echo "::error ::FFI function returns constant Ok(...)"
        exit 1
    fi
fi
echo "✓ No constant FFI returns found"

# Gate 3: Check for unimplemented placeholders
echo "Checking for unimplemented placeholders..."
if rg -n 'unimplemented!|todo!|panic!.*TODO' | grep -q .; then
    echo "::error ::Unimplemented placeholders found"
    exit 1
fi
echo "✓ No unimplemented placeholders found"

# Gate 4: Check for fake constant returns
echo "Checking for fake constant returns..."
if rg -n 'return\s+Ok(\s+["\'][A-Za-z0-9_\-]\{1,\}["\']\s*);' | grep -q .; then
    echo "::error ::Fake constant Ok returns found"
    exit 1
fi
echo "✓ No fake constant returns found"

# Gate 5: Check for stubbed functions
echo "Checking for stubbed functions..."
if rg -n 'dummy|fake|stub|placeholder' | grep -q .; then
    echo "::error ::Stubbed functions found"
    exit 1
fi
echo "✓ No stubbed functions found"

echo "✓ All CI gates passed"
