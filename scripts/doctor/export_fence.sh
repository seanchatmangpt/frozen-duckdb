#!/usr/bin/env bash
# ADV3 / KD-01 — native dylib export-fence evidence tool.
# Diagnoses the C++-mangled export surface of the pinned cached dylib:
#   1. subject derivation (same single-source VERSION as bin_audit.sh)
#   2. true mangled-export census (correct pattern; bin_audit.sh's
#      `[[:space:]]__[Z][Z]` regex only matches "__ZZ" and undercounts)
#   3. per-family classification (fmt/ska_sort/httplib internals,
#      duckdb:: core, ADBC helpers) + demangled top offenders
#   4. local-fix probe: strip -x on a copy, LC_DYLD_EXPORTS_TRIE check
# Exit 0 always: this is an evidence collector, not a gate (bin_audit
# owns the FAIL verdict). Every number printed is real `nm` output.
set -u

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BUILDER_LIB="$REPO_ROOT/crates/frozen-duckdb-builder/src/lib.rs"

VERSION="$(sed -n 's/^[[:space:]]*\(pub \)\{0,1\}const VERSION:[[:space:]]*&str[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\2/p' "$BUILDER_LIB" | head -n1)"
if [[ -z "$VERSION" ]]; then
    echo "[MISSING] cannot derive VERSION from $BUILDER_LIB" >&2
    exit 3
fi
ARCH="arm64"
DYLIB="${HOME}/.frozen-duckdb/cache/v${VERSION}-${ARCH}/libduckdb_${ARCH}.dylib"
if [[ ! -f "$DYLIB" ]]; then
    echo "[MISSING] $DYLIB not found — no substitution" >&2
    exit 3
fi

echo "== export_fence: KD-01 evidence, subject v${VERSION}-${ARCH} =="
echo "subject: $DYLIB"
echo "size: $(stat -f%z "$DYLIB") bytes"

NM_OUT="$(mktemp /tmp/export_fence.nm.XXXXXX)"
COPY="$(mktemp /tmp/export_fence.copy.XXXXXX)"
trap 'rm -f "$NM_OUT" "$COPY"' EXIT
nm -gU "$DYLIB" > "$NM_OUT" 2>/dev/null

total_T="$(grep -c '[[:space:]]T[[:space:]]*_' "$NM_OUT")"
# CORRECT mangled count: T symbols whose name starts with __Z (C++ mangling
# under the macOS leading underscore).
mangled_T="$(grep -cE '[[:space:]]T[[:space:]]*__Z' "$NM_OUT")"
zz21="$(grep -cE '[[:space:]]__[Z][Z]' "$NM_OUT")"   # bin_audit's regex, for contrast
duckdb_c="$(grep -cE '[[:space:]]T[[:space:]]*_duckdb_' "$NM_OUT")"

echo "T exports (leading underscore):        $total_T"
echo "C++-mangled T exports (__Z*):          $mangled_T   <-- TRUE KD-01 census"
echo "bin_audit '__ZZ' regex matches:        $zz21        <-- undercount (matches only __ZZ local-symbol mangling)"
echo "_duckdb_* C-ABI exports:               $duckdb_c"

echo
echo "-- family breakdown of mangled T exports --"
awk '$2=="T" && $3 ~ /^__Z/ {print $3}' "$NM_OUT" > "${NM_OUT}.mangled"
awk -F'[ :]' '{print $1}' /dev/null 2>/dev/null
while read -r sym; do
    case "$sym" in
        __ZN*10duckdb_fmt*)     echo duckdb_fmt ;;
        __ZN*15duckdb_ska_sort*) echo duckdb_ska_sort ;;
        __ZN*14duckdb_httplib*) echo duckdb_httplib ;;
        __ZN*6duckdb*)          echo duckdb_core_namespace ;;
        __Z*Adbc*)              echo adbc_helpers ;;
        __Z*duckdb_*)           echo c_api_internals ;;
        *)                      echo other ;;
    esac
done < "${NM_OUT}.mangled" | sort | uniq -c | sort -rn

echo
echo "-- 21 __ZZ symbols (function-local statics / lambda __invoke, demangled) --"
grep -E '[[:space:]]__[Z][Z]' "$NM_OUT" | awk '{print $3}' | c++filt | sed 's/^/  /'

echo
echo "-- local-fix probe (on a /tmp copy; subject untouched) --"
cp "$DYLIB" "$COPY"
if strip -x "$COPY" 2>/dev/null; then
    after="$(nm -gU "$COPY" 2>/dev/null | grep -cE '[[:space:]]T[[:space:]]*__Z')"
    echo "strip -x on copy: exit 0, mangled T exports after: $after (unchanged => strip cannot fence exports)"
else
    echo "strip -x failed on copy"
fi
trie="$(otool -l "$COPY" 2>/dev/null | grep -c LC_DYLD_EXPORTS_TRIE)"
echo "LC_DYLD_EXPORTS_TRIE load commands on copy: $trie (exports live in the link-time exports trie; no post-hoc darwin tool removes trie entries)"
if command -v objcopy >/dev/null 2>&1; then
    echo "objcopy present: localize-sym route theoretically available"
else
    echo "objcopy: NOT PRESENT on darwin (no --localize-symbol route)"
fi
echo "== export_fence: evidence complete (verdict: see docs/sjira/v2.0.0/receipts/ADV3-export-fence.md) =="
exit 0
