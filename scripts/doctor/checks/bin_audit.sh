#!/usr/bin/env bash
# TEST-002 — dynamic library size ceiling + symbol contract (C ABI only)
# Subject: the cached dylib for the CURRENT builder VERSION.
# Exit codes: 0 pass / 1 breach / 3 missing-subject.
# Every skip/absence is loud; never substitutes another slot's dylib.
set -u

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
BUILDER_LIB="$REPO_ROOT/crates/frozen-duckdb-builder/src/lib.rs"

# --- derive VERSION from the builder source (single source of truth) ---
VERSION="$(sed -n 's/^[[:space:]]*\(pub \)\{0,1\}const VERSION:[[:space:]]*&str[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\2/p' "$BUILDER_LIB" | head -n1)"
if [[ -z "$VERSION" ]]; then
    echo "[FAIL] cannot derive VERSION const from $BUILDER_LIB" >&2
    exit 3
fi
ARCH="arm64"
CACHE_DIR="${HOME}/.frozen-duckdb/cache"
DYLIB="${CACHE_DIR}/v${VERSION}-${ARCH}/libduckdb_${ARCH}.dylib"

# --- 55.0 MiB ceiling (blueprint) ---
MAX_BYTES=57671680

fail=0

echo "== bin_audit: subject = v${VERSION}-${ARCH} dylib =="

if [[ ! -f "$DYLIB" ]]; then
    echo "[MISSING] engine artifact pending rebuild: $DYLIB not found" >&2
    echo "[MISSING] cache slot v${VERSION}-${ARCH} exists or not — never substituting another version's dylib" >&2
    exit 3
fi

echo "subject: $DYLIB"
real_size="$(stat -f%z "$DYLIB")"
echo "real size: ${real_size} bytes"

# --- (1) size ceiling on a stripped /tmp copy ---
TMP_COPY="$(mktemp -t bin_audit.XXXXXX)"
trap 'rm -f "$TMP_COPY"' EXIT
cp "$DYLIB" "$TMP_COPY"
if ! strip -x "$TMP_COPY" 2>/dev/null; then
    echo "[FAIL] strip -x failed on tmp copy" >&2
    exit 1
fi
stripped_size="$(stat -f%z "$TMP_COPY")"
echo "stripped size: ${stripped_size} bytes (ceiling ${MAX_BYTES} = 55.0 MiB)"
if (( stripped_size > MAX_BYTES )); then
    echo "[FAIL] stripped size ${stripped_size} exceeds ceiling ${MAX_BYTES}" >&2
    fail=1
else
    echo "[PASS] stripped size within ceiling"
fi

# --- (2) symbol contract: C ABI only, no C++-mangled exports ---
# ADV3 correction: the true breach is ALL `__Z*`-mangled T exports
# (~27k on the pinned dev dylib), not only `__ZZ` function-local statics
# (~21). Count mangled `^[0-9a-f]+ T __Z` vs C-ABI `^[0-9a-f]+ T _duckdb_`
# separately; breach = mangled_count > 0.
nm_out="$(nm -gU "$DYLIB" 2>/dev/null)"
if [[ -z "$nm_out" && $? -ne 0 ]]; then
    echo "[FAIL] nm -gU failed" >&2
    exit 1
fi
total_exports="$(printf '%s\n' "$nm_out" | grep -c '[[:space:]]T[[:space:]]*_')"
mangled="$(printf '%s\n' "$nm_out" | grep -cE '^[0-9a-f]+ T __Z')"
duckdb_exports="$(printf '%s\n' "$nm_out" | grep -cE '^[0-9a-f]+ T _duckdb_')"
c_abi_count="$duckdb_exports"

echo "exported symbols (T, leading underscore): ${total_exports}"
echo "C++-mangled (__Z...) exports: ${mangled}"
echo "duckdb_* C-ABI exports: ${duckdb_exports}"

if (( mangled != 0 )); then
    echo "[FAIL] ${mangled} C++-mangled exports present (C ABI contract breached; ADV3-corrected count)" >&2
    fail=1
else
    echo "[PASS] zero C++-mangled exports"
fi

# --- (3) arch contract ---
archs="$(lipo -archs "$DYLIB" 2>/dev/null)"
if [[ $? -ne 0 ]]; then
    echo "[FAIL] lipo -archs failed" >&2
    exit 1
fi
echo "archs: ${archs}"
if [[ " $archs " != *" arm64 "* ]]; then
    echo "[FAIL] arm64 not present in lipo -archs output" >&2
    fail=1
else
    echo "[PASS] arm64 present"
fi

if (( fail != 0 )); then
    echo "== bin_audit: BREACH =="
    exit 1
fi
echo "== bin_audit: PASS =="
exit 0
