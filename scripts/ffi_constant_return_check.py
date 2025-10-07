#!/usr/bin/env python3
"""
FFI Constant Return Check
Scans for FFI functions that return constant Ok(...) without accessing dependencies.
"""

import re
import sys

def check_ffi_constant_returns():
    """Check for FFI functions that return constant values."""
    issues = []

    # Read the FFI lib file
    try:
        with open('crates/kcura-ffi/src/lib.rs', 'r') as f:
            content = f.read()
    except FileNotFoundError:
        print("FFI lib.rs not found - skipping check")
        return 0

    # Find all public extern "C" functions
    ffi_functions = re.findall(r'pub extern "C"\s+fn\s+kc_(\w+)\([^}]*\{([^}]*)\}', content, re.MULTILINE | re.DOTALL)

    for func_name, body in ffi_functions:
        # Check if function returns constant Ok(...)
        if re.search(r'return\s+Ok\([^)]+\);', body):
            # Check if it accesses any dependencies (kcura_*, duckdb, etc.)
            has_deps = re.search(r'(kcura_|duckdb|conn|db)', body)
            if not has_deps:
                issues.append(f"FFI function kc_{func_name} returns constant Ok(...) without accessing dependencies")

    # Report issues
    if issues:
        for issue in issues:
            print(f"::error :: {issue}")
        return 1

    return 0

if __name__ == "__main__":
    sys.exit(check_ffi_constant_returns())