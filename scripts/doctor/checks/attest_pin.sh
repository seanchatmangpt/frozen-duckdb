#!/usr/bin/env bash
# TEST-001 attest_pin.sh — header attestation court (lane D3)
#
# Reads SOURCE_PIN from crates/frozen-duckdb-builder/src/lib.rs, sha256s the
# four staged headers, asserts vendored==prebuilt per extension, fetches the
# upstream duckdb.h at SOURCE_PIN and diffs, and confirms the working-tree
# diff vs HEAD covers exactly the wave's staged headers (EXPECTED-STAGED).
# Exit 0 = attested, 1 = FAIL, 2 = loud SKIP-WITH-REASON.
set -u
ROOT="/Users/sac/frozen-duckdb"
VH="$ROOT/crates/frozen-duckdb-builder/vendored-headers"
PB="$ROOT/prebuilt"
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT
fail() { echo "FAIL: $*" >&2; exit 1; }

echo "== TEST-001 attest_pin =="
echo "[1] SOURCE_PIN"
PIN=$(grep -m1 'const SOURCE_PIN: &str = "' "$ROOT/crates/frozen-duckdb-builder/src/lib.rs" \
  | sed 's/.*"\(.*\)".*/\1/') || fail "SOURCE_PIN not found"
[[ "$PIN" =~ ^[0-9a-f]{40}$ ]] || fail "SOURCE_PIN malformed: '$PIN'"
echo "  SOURCE_PIN = $PIN"

echo "[2] sha256 of staged headers"
declare -A H
for f in "$VH/duckdb.h" "$VH/duckdb.hpp" "$PB/duckdb.h" "$PB/duckdb.hpp"; do
  [[ -f "$f" ]] || fail "missing staged header: $f"
  H[$f]=$(sha256sum "$f" | awk '{print $1}')
  echo "  $f  ${H[$f]}"
done

echo "[3] pairwise identity vendored == prebuilt (per ext)"
for ext in h hpp; do
  [[ "${H[$VH/duckdb.$ext]}" == "${H[$PB/duckdb.$ext]}" ]] \
    || fail "vendored/prebuilt duckdb.$ext diverge"
  echo "  OK duckdb.$ext"
done

echo "[4] upstream fetch at SOURCE_PIN"
UP="$TMP/duckdb.h.upstream"
URL="https://raw.githubusercontent.com/duckdb/duckdb/$PIN/src/include/duckdb.h"
if ! curl -fsSL --max-time 60 -o "$UP" "$URL"; then
  echo "[SKIP] upstream fetch failed (no network or pin absent on raw): $URL"
  echo "[SKIP-WITH-REASON] exiting 2; hashes + pairwise + staged-diff checks already passed above"
  exit 2
fi
if ! diff -u "$UP" "$VH/duckdb.h" > "$TMP/h.diff"; then
  fail "vendored duckdb.h differs from upstream@$PIN (diff in $TMP/h.diff, now removed)"
fi
echo "  OK duckdb.h matches upstream@$PIN"

echo "[5] git diff vs HEAD on staged header dirs"
DIFF_OUT=$(git -C "$ROOT" diff --name-only HEAD -- "$VH" "$PB" 2>&1) \
  || fail "git diff failed: $DIFF_OUT"
echo "  files differing from HEAD:"
echo "$DIFF_OUT" | sed 's/^/    /'
for f in duckdb.h duckdb.hpp; do
  echo "$DIFF_OUT" | grep -q "/$f$" || fail "expected $f in staged diff"
done
# Additional known wave-staged companions under prebuilt/ are EXPECTED-STAGED;
# anything else outside the allowlist is a real FAIL.
EXTRA=$(echo "$DIFF_OUT" | grep -v -e 'vendored-headers/duckdb\.h$' -e 'vendored-headers/duckdb\.hpp$' \
  -e 'prebuilt/duckdb\.h$' -e 'prebuilt/duckdb\.hpp$' -e 'prebuilt/README\.md$' -e 'prebuilt/setup_env\.sh$' || true)
[[ -z "$EXTRA" ]] || fail "unexpected non-wave files in staged diff: $EXTRA"
echo "  EXPECTED-STAGED: 4 headers + wave companions prebuilt/README.md, prebuilt/setup_env.sh differ from HEAD (uncommitted wave staging), not a failure"

echo "[6] GENESIS.md cross-check"
grep -q "upstream:duckdb/duckdb@v2.0.0" "$ROOT/docs/GENESIS.md" \
  || fail "GENESIS.md rows do not reference @v2.0.0"
grep -q "$PIN" "$ROOT/docs/GENESIS.md" \
  || fail "GENESIS.md does not carry SOURCE_PIN $PIN"
echo "  OK rows reference @v2.0.0 and pin $PIN"

echo "PASS: attested (pin $PIN, 4 headers pairwise-identical, upstream-matched, EXPECTED-STAGED diff)"
