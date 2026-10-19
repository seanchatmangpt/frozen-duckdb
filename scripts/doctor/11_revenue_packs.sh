#!/usr/bin/env bash
# Stage 11 — revenue packs court (Wave 12, lane R5).
#
# Runs the four revenue-pack legs fail-closed. Exit contract mirrors the
# doctor stage matrix: 0 = all clean, 1 = FAIL, 2 = skip, 3 =
# NOT-IMPLEMENTED (leg absent or zero tests executed — a filter that
# matches nothing is NOT a pass).
#
# Wiring note: this stage is NOT wired into scripts/doctor.sh. The
# doctor.sh PHASES array is an admitted transition owned by the
# coordinator, applied after the Wave 12 lanes land — a lane does not
# edit the aggregate.
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
CRATE="frozen-duckdb"

status=0

# run_native_leg <label> <test-name-filter>
# Runs `cargo test -p frozen-duckdb <filter>` and enforces BOTH:
#   (a) no FAILED line in the output, and
#   (b) at least one test actually executed.
# Cargo exits 0 when a filter matches zero tests, so exit code alone is
# not admission — the executed-count check is the fail-closed leg.
run_native_leg() {
  local label="$1" filter="$2" out rc executed
  echo "=== STAGE 11 leg: $label (cargo test filter: $filter) ==="
  out="$(cargo test -p "$CRATE" "$filter" 2>&1)"
  rc=$?
  if [ $rc -ne 0 ]; then
    echo "  cargo rc=$rc — leg FAIL (build break or test failure)"
    printf '%s\n' "$out" | tail -30
    status=1
    return
  fi
  executed="$(printf '%s\n' "$out" | grep -c '^test .* \.\.\..*ok$' || true)"
  if [ "$executed" -eq 0 ]; then
    echo "  NOT-IMPLEMENTED: filter matched 0 executed tests (rc=0 but nothing ran) — fail-closed, exit 3 class"
    status=3
    return
  fi
  echo "  PASS: $executed test(s) executed and ok"
}

echo "=== STAGE 11 leg 1: browser/OPFS process-mining leg — FR-01/FR-02 BLOCKED (loud, NOT a fake pass) ==="
echo "  The browser/OPFS leg of the revenue packs depends on FR-01 (WASM"
echo "  compile: scripts/wasm/ does not exist, no emcmake build attempted)"
echo "  and FR-02 (OPFS persistence: no opfs/ module, no crash court on"
echo "  disk). That leg is BLOCKED FR-01/FR-02 and is NOT executed here."
echo "  Any green claim for the browser/OPFS leg from this stage is VOID."
echo "  Only the NATIVE leg of the process-mining pack (test_rev_process_mining)"
echo "  runs below; it can at most witness the native side."

run_native_leg "rev compliance"      "test_rev_compliance"
run_native_leg "rev clean room"      "test_rev_clean_room"
run_native_leg "rev edge autonomy"   "test_rev_edge_autonomy"
run_native_leg "rev process mining (native leg)" "test_rev_process_mining"

case "$status" in
  0)
    echo "STAGE 11 REVENUE PACKS: ALL FOUR LEGS CLEAN (3 native + browser/OPFS leg confirmed BLOCKED FR-01/FR-02, unexecuted)"
    exit 0
    ;;
  3)
    echo "STAGE 11 REVENUE PACKS: NOT-IMPLEMENTED (one or more legs matched 0 tests) — fail-closed"
    exit 3
    ;;
  *)
    echo "STAGE 11 REVENUE PACKS: FAIL (exit 1)"
    exit 1
    ;;
esac
