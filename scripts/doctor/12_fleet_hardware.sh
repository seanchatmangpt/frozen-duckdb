#!/usr/bin/env bash
# Stage 12 — fleet hardware-class battery (Wave 13, lane T6, wire+bookkeep).
#
# Five hardware classes, fail-closed:
#   A workstation (native leg)          — cargo test_fleet_class_a_workstation
#   B low-RAM runner (cgroup, Linux)    — cgroup_low_ram_runner.sh + proxy test
#   C retail edge (native leg)          — cargo test_fleet_class_c_retail
#   D marine/offline (chaos+compression)— fleet_d_chaos.sh + marine compression
#   E telco (duckdb leg)                — cargo test_fleet_class_e_telco
#
# Exit contract (mirrors the doctor stage matrix):
#   0 = every executable leg passed AND every blocked leg was echoed loud
#   1 = FAIL (leg ran and failed / compile abort)
#   2 = skip class, 3 = NOT-IMPLEMENTED (leg absent or 0 tests executed)
#
# Wiring note: this stage is NOT wired into scripts/doctor.sh — the
# aggregate PHASES array is the coordinator's admitted transition.
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT" || exit 1
CRATE="frozen-duckdb"
status=0
failed=0

# run_native_leg <label> <cargo-test-filter>
# Fail-closed: rc!=0 is FAIL; rc=0 with zero executed tests is
# NOT-IMPLEMENTED (a filter that matches nothing is never a pass).
run_native_leg() {
  local label="$1" filter="$2" out rc executed
  echo "--- native leg: $label (filter: $filter) ---"
  out="$(cargo test -p "$CRATE" --test "$filter" 2>&1)"
  rc=$?
  if [ $rc -ne 0 ]; then
    echo "  cargo rc=$rc — leg FAIL (build break or test failure)"
    printf '%s\n' "$out" | tail -30
    [ "$status" -lt 1 ] && status=1
    failed=1
    return
  fi
  executed="$(printf '%s\n' "$out" | grep -c '^test .* \.\.\. ok$' || true)"
  if [ "$executed" -eq 0 ]; then
    echo "  NOT-IMPLEMENTED-LOUD: filter matched 0 executed tests (rc=0)"
    echo "  — fail-closed, exit-3 class. A missing leg is never a pass."
    status=3
    return
  fi
  echo "  PASS: $executed test(s) executed and ok"
}

# run_exec_stage <label> <script-path> [args...]
# Runs an executable shell leg; rc=2 propagates as loud NOT-IMPLEMENTED
# (the cgroup runner's darwin contract), rc=3 likewise.
run_exec_stage() {
  local label="$1" script="$2"; shift 2
  local out rc
  echo "--- exec leg: $label ($script) ---"
  if [ ! -f "$script" ]; then
    echo "  NOT-IMPLEMENTED-LOUD: $script not on disk — leg absent, NOT a pass"
    status=3
    return
  fi
  out="$(bash "$script" "$@" 2>&1)"
  rc=$?
  printf '%s\n' "$out" | tail -15
  case "$rc" in
    0) echo "  PASS: $script rc=0" ;;
    2) echo "  NOT-IMPLEMENTED-LOUD: $script rc=2 (platform leg absent)"
       status=3 ;;
    3) echo "  NOT-IMPLEMENTED-LOUD: $script rc=3 (missing subject)"
       status=3 ;;
    *) echo "  FAIL: $script rc=$rc"
       [ "$status" -lt 1 ] && status=1
       failed=1 ;;
  esac
}

echo "=== STAGE 12 · CLASS A — workstation ==="
echo "  [BLOCKED-LOUD] wasm/OPFS leg of Class A: FR-01 not implemented"
echo "  (no scripts/wasm/, no emcmake build; no OPFS module). That leg is"
echo "  BLOCKED FR-01 and is NOT executed. Any green claim for it is VOID;"
echo "  only the NATIVE leg below can be witnessed by this stage."
if [ -f "crates/$CRATE/tests/test_fleet_class_a_workstation.rs" ]; then
  run_native_leg "class A workstation (native)" "test_fleet_class_a_workstation"
else
  echo "  NOT-IMPLEMENTED-LOUD: test_fleet_class_a_workstation.rs absent"
  status=3
fi

echo "=== STAGE 12 · CLASS B — low-RAM runner ==="
echo "  Class B's real gate is cgroup-based memory pressure on Linux."
run_exec_stage "class B cgroup runner" \
  "scripts/doctor/cgroup_low_ram_runner.sh"

echo "=== STAGE 12 · CLASS B proxy — low-RAM emulation test ==="
echo "  The proxy test (in-process memory-bound workload) is the only"
echo "  Class B witness runnable on this host; it emulates, it does NOT"
echo "  replace the cgroup gate (physical-reality: no cgroup on darwin)."
if [ -f "crates/$CRATE/tests/test_fleet_class_b_industrial.rs" ]; then
  run_native_leg "class B low-RAM proxy" "test_fleet_class_b_industrial"
else
  echo "  NOT-IMPLEMENTED-LOUD: test_fleet_class_b_industrial.rs absent"
  status=3
fi

echo "=== STAGE 12 · CLASS C — retail edge ==="
if [ -f "crates/$CRATE/tests/test_fleet_class_c_retail.rs" ]; then
  run_native_leg "class C retail edge" "test_fleet_class_c_retail"
else
  echo "  NOT-IMPLEMENTED-LOUD: test_fleet_class_c_retail.rs absent"
  status=3
fi

echo "=== STAGE 12 · CLASS D — marine/offline chaos court ==="
run_exec_stage "class D chaos court" "scripts/courts/fleet_d_chaos.sh"

echo "--- class D marine compression leg ---"
if [ -f "crates/$CRATE/tests/test_fleet_class_d_marine.rs" ]; then
  run_native_leg "class D marine compression" "test_fleet_class_d_marine"
else
  echo "  NOT-IMPLEMENTED-LOUD: test_fleet_class_d_marine.rs absent"
  status=3
fi

echo "=== STAGE 12 · CLASS E — telco (duckdb leg) ==="
echo "  [BLOCKED-LOUD] BEAM :atomics/:ets actor leg lives in smalt —"
echo "  BLOCKED-OTHER-REPO, out of scope for this checkout, NOT faked."
run_native_leg "class E telco (duckdb leg)" "test_fleet_class_e_telco"

case "$status" in
  0)
    echo "STAGE 12 FLEET HARDWARE: ALL EXECUTABLE LEGS PASS;"
    echo "blocked legs (A wasm/OPFS FR-01, B cgroup-on-darwin, E BEAM/smalt)"
    echo "echoed loud, unexecuted, unfaked."
    exit 0
    ;;
  3)
    if [ "$failed" -eq 1 ]; then
      echo "STAGE 12 FLEET HARDWARE: FAIL + NOT-IMPLEMENTED — fail-closed"
      exit 1
    fi
    echo "STAGE 12 FLEET HARDWARE: NOT-IMPLEMENTED (one or more legs absent"
    echo "or matched 0 tests) — fail-closed"
    exit 3
    ;;
  *)
    echo "STAGE 12 FLEET HARDWARE: FAIL (exit 1)"
    exit 1
    ;;
esac
