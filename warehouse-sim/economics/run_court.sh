#!/usr/bin/env bash
# Empirical economic court runner — lane G5 (BASELINE→BEFORE→SHIM→AFTER shape
# applied as BEFORE=warehouse leg, SHIM=workload dispatch, AFTER=sovereign leg
# + fences; plus a REQUIRED expected-fail selftest proving the court can flip).
set -u
cd "$(dirname "$0")/.."   # warehouse-sim/

RUN_ID=$(date +%s)-$$
EV=economics/.evidence/run-$RUN_ID
LATEST=economics/.evidence/latest
mkdir -p "$EV"
LOG="$EV/run.log"
ln -sfn "$(pwd)/$EV" "$LATEST" 2>/dev/null
: > "$LOG"

say() { echo "[run_court] $*" | tee -a "$LOG"; }

# ---- 0. Toolchain prereqs (loud skips, never faked passes) -----------------
command -v node  >/dev/null || { say "[SKIP] node not found"; exit 2; }
command -v cargo >/dev/null || { say "[SKIP] cargo not found"; exit 2; }
command -v curl  >/dev/null || { say "[SKIP] curl not found"; exit 2; }
[ -d node_modules ]          || { say "[SKIP] npm install not done"; exit 2; }

# ---- 1. Sovereign leg (BASELINE receipt from the real probe) ---------------
LIBDIR=$(ls -d "$HOME"/.frozen-duckdb/cache/v2.0.0-arm64 2>/dev/null | head -1)
[ -n "$LIBDIR" ] || LIBDIR=$(find "$HOME/.frozen-duckdb/cache" -name 'libduckdb.dylib' 2>/dev/null | head -1 | xargs dirname)
[ -n "$LIBDIR" ] || { say "[SKIP] no cached libduckdb (run builder once)"; exit 2; }

say "building sovereign_probe"
( cd economics/sovereign_probe && CARGO_TARGET_DIR=target-g5 cargo build --release ) >>"$LOG" 2>&1 \
  || { say "FAIL: sovereign_probe build"; exit 1; }

say "running sovereign probe (Path B, native-baseline)"
DYLD_LIBRARY_PATH="$LIBDIR" ./economics/sovereign_probe/target-g5/release/sovereign_probe \
  > "$EV/sovereign.json" 2>>"$LOG" \
  || { say "FAIL: sovereign probe exited nonzero"; exit 1; }

# ---- 2. Warehouse leg: OWN the sim server (deadline-based readiness) -------
PORT=${PORT:-0}
[ "$PORT" = "0" ] && PORT=$(( 21000 + RANDOM % 20000 ))
BASE="http://127.0.0.1:${PORT}"
say "starting next dev on :${PORT} (readiness deadline 120s, stability x2)"
( npm run dev -- -p "$PORT" > "$EV/next-dev.log" 2>&1 & echo $! > "$EV/next-dev.pid" )
SRV_PID=$(cat "$EV/next-dev.pid")

ok=0; prev=0; ready=""
deadline=$(( $(date +%s) + 120 ))
while [ "$(date +%s)" -lt "$deadline" ]; do
  code=$(curl -s -o /dev/null -w '%{http_code}' --max-time 3 "$BASE/api/v2/statements" 2>/dev/null)
  curl_rc=$?
  [ "$curl_rc" -eq 0 ] || code=000
  if [ "$code" != "000" ]; then
    if [ "$prev" != "000" ]; then ready=1; break; fi   # two consecutive non-000
    prev=$code
  else
    prev=000
  fi
  sleep 1   # pacing inside the bounded readiness poll (audited)
done

if [ -z "$ready" ]; then
  kill "$SRV_PID" 2>/dev/null; pkill -P "$SRV_PID" 2>/dev/null
  say "[SKIP] sim server did not become ready within 120s (see $EV/next-dev.log)"
  exit 2
fi
say "sim ready (owned) at $BASE (pid $SRV_PID)"

cleanup() {
  kill "$SRV_PID" 2>/dev/null
  pkill -P "$SRV_PID" 2>/dev/null
}
trap cleanup EXIT

cleanup() {
  if [ "${already_up:-0}" = 0 ] && [ -f "$EV/next-dev.pid" ]; then
    kill "$(cat "$EV/next-dev.pid")" 2>/dev/null
    pkill -P "$(cat "$EV/next-dev.pid")" 2>/dev/null
  fi
}
trap cleanup EXIT

# ---- 3. Court: real both-paths run -----------------------------------------
say "running economic court (Path A over HTTP, Path B receipt above)"
node economics/court.ts --sovereign "$EV/sovereign.json" --base "$BASE" --out "$EV/receipt.json" \
  >> "$LOG" 2>&1
COURT=$?

# ---- 4. Non-vacuity: the court MUST be able to fail ------------------------
say "running selftest (doctored sovereign leg must be REFUSED)"
node economics/court.ts --selftest --sovereign "$EV/sovereign.json" >> "$LOG" 2>&1
SELF=$?
case $SELF in
  3) say "selftest flipped as required (exit 3)" ;;
  0) say "FAIL VACUOUS: selftest accepted a doctored PASS"; exit 1 ;;
  *) say "FAIL: selftest errored unexpectedly (exit $SELF)"; exit 1 ;;
esac

if [ $COURT -eq 0 ]; then
  say "VERDICT: PASS — sovereign <= 5% latency, 0 egress, USD0 invoice, sim burn nonzero"
  exit 0
else
  say "VERDICT: FAIL (court exit $COURT)"
  exit $COURT
fi
