#!/usr/bin/env bash
# Disruption scorecard runner — lane S5. Same shape as economics/run_court.sh
# (reused, not forked): build the sovereign probe, run Path B for real, boot
# the sim server (deadline-based readiness), run Path A over real HTTP, emit
# disruption_scorecard.json + markdown, then the REQUIRED expected-fail
# selftest proving the assertion machinery can flip.
set -u
cd "$(dirname "$0")/.."   # warehouse-sim/

EV=economics/.evidence
mkdir -p "$EV"
LOG="$EV/scorecard.log"
: > "$LOG"

say() { echo "[run_scorecard] $*" | tee -a "$LOG"; }

# ---- 0. Toolchain prereqs (loud skips, never faked passes) -----------------
command -v node  >/dev/null || { say "[SKIP] node not found"; exit 2; }
command -v cargo >/dev/null || { say "[SKIP] cargo not found"; exit 2; }
command -v curl  >/dev/null || { say "[SKIP] curl not found"; exit 2; }
[ -d node_modules ]          || { say "[SKIP] npm install not done"; exit 2; }

# ---- 1. Sovereign leg (Path B receipt from the real probe, --full 5-class) -
LIBDIR=$(ls -d "$HOME"/.frozen-duckdb/cache/v2.0.0-arm64 2>/dev/null | head -1)
[ -n "$LIBDIR" ] || LIBDIR=$(find "$HOME/.frozen-duckdb/cache" -name 'libduckdb.dylib' 2>/dev/null | head -1 | xargs dirname)
[ -n "$LIBDIR" ] || { say "[SKIP] no cached libduckdb (run builder once)"; exit 2; }

say "building sovereign_probe"
( cd economics/sovereign_probe && CARGO_TARGET_DIR=target-g5 cargo build --release ) >>"$LOG" 2>&1 \
  || { say "FAIL: sovereign_probe build"; exit 1; }

say "running sovereign probe --full (Path B, native-baseline, 5 classes)"
DYLD_LIBRARY_PATH="$LIBDIR" ./economics/sovereign_probe/target-g5/release/sovereign_probe --full \
  > "$EV/sovereign-full.json" 2>>"$LOG" \
  || { say "FAIL: sovereign probe exited nonzero"; exit 1; }

# ---- 2. Warehouse leg: OWN the sim server (deadline-based readiness) -------
PORT=${PORT:-3301}
BASE="http://127.0.0.1:${PORT}"
say "starting next dev on :${PORT} (readiness deadline 120s, stability x2)"
( npm run dev -- -p "$PORT" > "$EV/scorecard-next-dev.log" 2>&1 & echo $! > "$EV/scorecard-next-dev.pid" )
SRV_PID=$(cat "$EV/scorecard-next-dev.pid")

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
  say "[SKIP] sim server did not become ready within 120s (see $EV/scorecard-next-dev.log)"
  exit 2
fi
say "sim ready (owned) at $BASE (pid $SRV_PID)"

cleanup() {
  kill "$SRV_PID" 2>/dev/null
  pkill -P "$SRV_PID" 2>/dev/null
}
trap cleanup EXIT

# ---- 3. Scorecard: real both-paths run --------------------------------------
NODE_ALIAS=(node --import ./economics/alias-register.mjs)

say "running disruption scorecard (Path A over HTTP, Path B receipt above)"
"${NODE_ALIAS[@]}" economics/scorecard.ts \
  --sovereign "$EV/sovereign-full.json" --base "$BASE" \
  --json "$EV/disruption_scorecard.json" --md "$EV/disruption_scorecard.md" \
  >> "$LOG" 2>&1
SC=$?

# ---- 4. Non-vacuity: the assertion machinery MUST be able to fail -----------
say "running selftest (doctored sovereign leg must be REFUSED)"
"${NODE_ALIAS[@]}" economics/scorecard.ts --selftest --sovereign "$EV/sovereign-full.json" >> "$LOG" 2>&1
SELF=$?
case $SELF in
  3) say "selftest flipped as required (exit 3)" ;;
  0) say "FAIL VACUOUS: selftest accepted a doctored PASS"; exit 1 ;;
  *) say "FAIL: selftest errored unexpectedly (exit $SELF)"; exit 1 ;;
esac

if [ $SC -eq 0 ]; then
  # Regenerate the table in the design doc between its markers (byte-exact
  # splice; prose outside the markers is untouched).
  DOC=../docs/design/disruption-scorecard.md
  if [ -f "$DOC" ] && grep -q 'SCORECARD:BEGIN' "$DOC"; then
    awk -v md="$EV/disruption_scorecard.md" '
      BEGIN{while((getline l < md)>0) tbl=tbl l "\n"}
      /SCORECARD:BEGIN/{print; print ""; system("cat " md); inblk=1; next}
      /SCORECARD:END/{inblk=0}
      !inblk && !/SCORECARD:BEGIN/{print}
    ' "$DOC" > "$DOC.tmp" && mv "$DOC.tmp" "$DOC"
    say "doc table regenerated at $DOC"
  fi
  say "VERDICT: PRODUCED — scorecard at $EV/disruption_scorecard.json (+ md); F1/F2 NEG rows are honest negatives, F3 held at \$0.00"
  exit 0
else
  say "VERDICT: FAIL (scorecard exit $SC)"
  exit $SC
fi
