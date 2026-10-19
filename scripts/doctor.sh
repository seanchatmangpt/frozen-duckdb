#!/usr/bin/env bash
# doctor.sh — aggregate verification runner (lane H2)
# Repo: /Users/sac/frozen-duckdb
#
# Contract:
#   set -euo pipefail; 13 phase banners (00..12); fail-closed — exit 0 only if
#   every phase exits 0. NOT-IMPLEMENTED phases (exit 3) are loud divergences,
#   never fake passes, and still fail the aggregate.
#
# NOTE (stages 11/12): 11_revenue_packs and 12_fleet_hardware include loud
# NOT-IMPLEMENTED/BLOCKED legs BY DESIGN (wasm, cgroup, BEAM legs are
# platform-gated). On this darwin host the aggregate therefore cannot reach
# ALL-CLEAN until KD-05/KD-09/KD-10 close. That standing is correct, not a
# defect — the fail-closed contract requires the aggregate to stay red while
# any leg is a loud divergence.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PHASES=(
  00_preflight
  01_sys_audit
  02_storage_wal
  03_types_variant
  04_host_vfs
  05_memory_guard
  06_browser_opfs
  07_ocel_compress
  08_perf_sla
  09_concurrency
  10_verify_economics
  11_revenue_packs
  12_fleet_hardware
)

any_fail=0
for p in "${PHASES[@]}"; do
  f="$ROOT/scripts/doctor/${p}.sh"
  echo "=================================================================="
  echo "== PHASE $p"
  echo "=================================================================="
  if [ ! -x "$f" ]; then
    echo "[FAIL] phase script missing or non-executable: $f"
    any_fail=1
    continue
  fi
  if bash "$f"; then
    echo "[PASS] $p"
  else
    rc=$?
    case "$rc" in
      2) echo "[SKIP] $p" ;;
      3) echo "[NOT-IMPLEMENTED] $p" ;;
      *) echo "[FAIL] $p (exit $rc)" ;;
    esac
    any_fail=1
  fi
done

echo "=================================================================="
if [ "$any_fail" -eq 0 ]; then
  verdict="ALL 13 PHASES VERIFIED CLEAN"
else
  verdict="DOCTOR VERDICT: FAIL-CLOSED — one or more phases not clean"
fi

# ---------------------------------------------------------------------------
# KNOWN DIVERGENCES summary (informational; parsed live from the register,
# never cached — the exit contract above is unchanged by this block).
# ---------------------------------------------------------------------------
KD_FILE="$ROOT/KNOWN_DIVERGENCES.md"
echo "== KNOWN DIVERGENCES =="
if [ ! -f "$KD_FILE" ]; then
  echo "[SKIP] register not found: $KD_FILE"
else
  # Data rows: table lines starting "| KD-"
  kd_total=$(grep -cE '^\| KD-[0-9]+' "$KD_FILE")
  kd_closed=$(grep -E '^\| KD-[0-9]+' "$KD_FILE" | grep -c 'CLOSED —')
  kd_open=$((kd_total - kd_closed))
  echo "total: $kd_total  open: $kd_open  closed: $kd_closed"
  if [ "$kd_closed" -gt 0 ]; then
    grep -E '^\| KD-[0-9]+' "$KD_FILE" | grep 'CLOSED —' \
      | sed -E 's/^\| (KD-[0-9]+).*/\1/' | tr '\n' ' ' | sed 's/ $//'
    echo
  fi
fi

echo "$verdict"
if [ "$any_fail" -eq 0 ]; then
  exit 0
fi
exit 1
