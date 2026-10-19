#!/usr/bin/env bash
# value_audit.sh — lane V6 orchestrator: four value-audit courts + CFO render.
# Repo: /Users/sac/frozen-duckdb
#
# Contract:
#   Runs the four value-audit courts for real (cargo, default target dir —
#   post-freeze), waits on the /tmp/value_audit receipts with a bounded
#   deadline poll (POLL-STEP: `read -t` is the audited wait primitive; no
#   bare Process.sleep-style sleeps), then renders the operator's CFO
#   terminal format via warehouse-sim/economics/render_value_audit.ts.
#
# RATE CARD LAW: every dollar figure in the rendered output is derived from
# the ASSUMPTION-UNVERIFIED rate card (operator-supplied list prices,
# unaudited) declared in render_value_audit.ts's header and mirrored here:
#   - Datadog: $0.10/GB-mo ingestion + $1.70/M-events-mo indexing
#   - Celonis EMS: $0.004 per process-event segment mined
#   - Amplitude: $200/M-events-mo
#   - Cadence: 1 run/business day x 252 business days/yr
# Physical quantities are real receipt values; dollars are assumption math.
#
# Exit 0 only if all four courts ran green (or a missing court is a loud,
# registered [SKIP] with its reason printed and echoed as a divergence).

set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DIR=/tmp/value_audit
DEADLINE=${VALUE_AUDIT_DEADLINE:-900} # seconds, bounded receipt poll
mkdir -p "$DIR"

run_court() { # <name> <receipt-file> <cargo test args...>
  local name="$1" receipt="$2"; shift 2
  echo "== COURT $name"
  local out
  out=$(cargo test "$@" 2>&1)
  local rc=$?
  printf '%s\n' "$out" | tail -4
  if [ $rc -ne 0 ]; then
    echo "[FAIL] $name: cargo exit $rc"; printf '%s\n' "$out" | grep -E "panicked|FAILED|error" | head -5
    return 1
  fi
  # Bounded deadline poll on the real observable (the receipt file).
  local waited=0
  while [ ! -s "$DIR/$receipt" ]; do
    [ $waited -ge "$DEADLINE" ] && { echo "[SKIP] $name: receipt $DIR/$receipt absent after ${DEADLINE}s — registered divergence"; return 2; }
    read -t 2 < /dev/null || true # POLL-STEP: audited 2s wait primitive
    waited=$((waited + 2))
  done
  echo "[PASS] $name -> receipt $DIR/$receipt"
}

overall=0
run_court VAL-001/datadog val001.json \
  -p frozen-duckdb --test value_audit_datadog -- --nocapture || overall=1
run_court VAL-002/celonis val002.json \
  -p frozen-duckdb --test value_audit_celonis -- --nocapture || overall=1
run_court GATE-V-01/latency gate_v01_receipt.json \
  -p frozen-duckdb --test gate_v01_latency -- --nocapture || overall=1
# V4's collector chains gate_v01_receipt.json -> val003.json (value ledger).
bash "$ROOT/scripts/value_audit/collect_val003.sh" || overall=1
run_court VAL-004/amplitude val004.json \
  -p frozen-duckdb --test value_audit_amplitude -- --nocapture || overall=1

# Doctor pass gate for the Physical System State line.
# Preference order:
#   1. GAP15's shared log /tmp/gap15_doctor.log — poll up to 15 min for the
#      wrapper's DOCTOR_EXIT= verdict (POLL-STEP: read -t, no bare sleeps).
#   2. Run doctor.sh here — ONLY if no other lane is mid-run (pgrep cargo
#      must be empty; the doctor compiles the shared workspace).
#   3. Contended → render without --doctor and say so (loud divergence).
echo "== DOCTOR (verdict from GAP15 log, else local run, else contended-skip)"
GAP_LOG=/tmp/gap15_doctor.log
DOCTOR_LOG=""
DOCTOR=""
if [ -f "$GAP_LOG" ] || pgrep -f 'doctor\.sh' >/dev/null 2>&1; then
  waited=0
  while [ $waited -lt 900 ]; do
    verdict_line=$(grep -oE 'DOCTOR_EXIT=[0-9]+' "$GAP_LOG" 2>/dev/null | tail -1)
    [ -n "$verdict_line" ] && break
    # No wrapper verdict but the log is stale and no doctor is running: a
    # finished log still carries per-phase FAILs — treat as conclusive.
    if ! pgrep -f 'doctor\.sh' >/dev/null 2>&1 && [ -f "$GAP_LOG" ]; then
      waited=900; break
    fi
    read -t 5 < /dev/null || true # POLL-STEP: audited 5s wait primitive
    waited=$((waited + 5))
  done
  DOCTOR_LOG="$GAP_LOG"
  if [ "${verdict_line:-}" = "DOCTOR_EXIT=0" ]; then
    DOCTOR=GREEN
    echo "[PASS] doctor aggregate GREEN (GAP15 log)"
  else
    fails=$(grep -E '^\[FAIL\]' "$GAP_LOG" 2>/dev/null | sed 's/^/  /' | head -8)
    DOCTOR="NOT GREEN — doctor aggregate failed (see $GAP_LOG)"
    echo "[STANDING] doctor NOT green (GAP15 log)"; [ -n "$fails" ] && printf '%s\n' "$fails"
  fi
elif ! pgrep -x cargo >/dev/null 2>&1; then
  echo "== no shared doctor run; cargo uncontended — running doctor.sh here"
  if bash "$ROOT/scripts/doctor.sh" > "$DIR/doctor_out.txt" 2>&1; then
    DOCTOR=GREEN
    DOCTOR_LOG="$DIR/doctor_out.txt"
    echo "[PASS] doctor aggregate GREEN"
  else
    DOCTOR="NOT GREEN — doctor aggregate failed (see $DIR/doctor_out.txt)"
    DOCTOR_LOG="$DIR/doctor_out.txt"
    echo "[STANDING] doctor NOT green"
  fi
else
  echo "[SKIP] doctor verdict: cargo contended by another lane — rendering WITHOUT --doctor (Physical System State line stays honest not-green)"
fi

# Render — only if all four receipts exist; a missing receipt is a loud SKIP
# of the render, never a partial fake total.
missing=0
for r in val001.json val002.json val003.json val004.json; do
  [ -s "$DIR/$r" ] || { echo "[SKIP] render: receipt $r absent"; missing=1; }
done
if [ $missing -eq 0 ]; then
  B3_BLOCK=$(b3sum "$DIR/val001.json" "$DIR/val002.json" "$DIR/val003.json" "$DIR/val004.json")
  export B3_BLOCK
  render_args=(--dir "$DIR")
  if [ -n "$DOCTOR" ]; then
    render_args+=(--doctor "$DOCTOR")
    [ -n "$DOCTOR_LOG" ] && render_args+=(--doctor-log "$DOCTOR_LOG")
  fi
  # Mint-then-verify: sign the receipts immediately before rendering so no
  # tamper window opens between mint and render (AD4 integrity law).
  bash "$ROOT/scripts/value_audit_sign.sh" --dir "$VA_DIR"
  MANIFEST="$VA_DIR/manifest.b3"
  test -s "$MANIFEST" || { echo "RENDER-FAIL: manifest missing after sign" >&2; exit 1; }
  render_args+=("--verify" "$MANIFEST")
  node "$ROOT/warehouse-sim/economics/render_value_audit.ts" "${render_args[@]}"
  render_rc=$?
  [ $render_rc -ne 0 ] && overall=1
else
  overall=1
fi

exit $overall
