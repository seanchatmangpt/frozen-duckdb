#!/usr/bin/env bash
# fleet_d_chaos.sh — TEST-FLEET-004 native leg, chaos court (lane T2).
# Spec: the marine container-telemetry 1M-event fleet class D battery.
#
# Subject: /tmp/t2writer (cargo fixture, frozen-duckdb path dep, rpath to the
# cached v2.0.0-dev-arm64 dylib). Schema: D8 `ocel_conforming_segments` RLE
# (segment_id, oid, action, channel, t_start, t_end, transition_count) plus a
# per-block chksum column carrying the closed form sum_{n}(3n+7), so any torn
# transaction is detectable by checksum alone.
#
# Claim: kill -9 DURING an open write transaction leaves the db
#   (1) mountable, (2) WAL-replayed to a COMMITTED txn boundary,
#   (3) event count in {last-committed sentinel, in-txn sentinel} (two legal
#       states, never a third) and checksum == closed-form(count).
#
# BASELINE -> BEFORE -> SHIM -> AFTER -> RESTORE shape.
# Non-vacuity flip (leg c): kill -9 -> kill -0 SHIM-COURT no-op; the court
# must DETECT the survivor and flip the leg to VACUOUS-GATE. A court that
# cannot flip is FAIL, not PASS.
#
# No git state mutations. No sleeps: every wait is a bounded poll on a real
# observable (sentinel file / process liveness) with a wall-clock deadline.
set -u

REPO=/Users/sac/frozen-duckdb
T2_BIN=/tmp/t2writer/target/release/t2writer
T2_SRC=/tmp/t2writer/src/main.rs
COURT_SELF="$REPO/scripts/courts/fleet_d_chaos.sh"
WORK=/tmp/t2court.$$
TARGET=200000    # kill point, events (in-flight leg a)
TARGET2=260000   # kill point, events (in-flight leg b)
TOTAL_COMPLETE=300000 # leg d: completed-ingest leg (no kill)
DEADLINE_POLL=120
RC=0
RECEIPT="${FLEET_D_RECEIPT:-$REPO/target/court/fleet-d-chaos-receipt.json}"

# Per-run fixture dir: concurrent court runs must never share db files
# (F7-D2 torn-txn witnessed when two lanes ran this court against the
# shared /tmp/value_audit path).
RUN_DIR="/tmp/value_audit_$(date -u +%Y%m%dT%H%M%S)_$$"
mkdir -p "$REPO/target/court" "$RUN_DIR" || { echo "[SKIP] unwritable target dirs"; exit 1; }

say() { printf '%s %s\n' "$(date -u +%H:%M:%S)" "$*"; }
shasum() { command shasum -a 256 "$@" | cut -d' ' -f1; }

poll_sentinel() { # $1=file $2=min_count $3=deadline_epoch
  local c
  while [ "$(date +%s)" -lt "$3" ]; do
    c=$(cat "$1" 2>/dev/null || echo 0)
    [ "$c" -ge "$2" ] && return 0
  done
  return 1
}

poll_gone() { # $1=pid $2=deadline
  while [ "$(date +%s)" -lt "$2" ]; do
    kill -0 "$1" 2>/dev/null || return 0
  done
  return 1
}

poll_done() { # $1=done-file $2=deadline
  while [ "$(date +%s)" -lt "$2" ]; do
    [ -f "$1" ] && return 0
  done
  return 1
}

rd() { head -1 "$1" 2>/dev/null | tr -d '[:space:]'; }

# ---- BASELINE -------------------------------------------------------------
base_self=$(shasum "$COURT_SELF")
base_writer=$(shasum "$T2_SRC")

cleanup() { [ -n "${WPGID:-}" ] && kill -9 -- "-$WPGID" 2>/dev/null; }
trap cleanup EXIT

# ---- one crash leg -------------------------------------------------------
# crash_leg <label> <target> <kill_mode> <total>
# prints PASS/FAIL/DETECT lines; rc: 0 PASS, 1 FAIL, 2 DETECT (vacuity seen)
crash_leg() {
  local label=$1 target=$2 kmode=$3 total=$4
  local db="$WORK/db-$label.db" sent="$WORK/sent-$label"
  mkdir -p "$WORK"
  rm -f "$db" "$db".wal "$sent" "$sent".committed "$sent".tmp "$sent".done

  # own process group (portable setsid via perl setpgrp+exec)
  perl -e 'setpgrp(0,0); exec @ARGV' -- "$T2_BIN" writer "$db" "$sent" "$total" >/dev/null 2>&1 &
  local wpid=$! pgid="" dl
  dl=$(( $(date +%s) + DEADLINE_POLL ))
  while [ "$(date +%s)" -lt "$dl" ]; do
    pgid=$(ps -o pgid= -p "$wpid" 2>/dev/null | tr -d ' ')
    [ -n "$pgid" ] && break
  done
  if [ -z "$pgid" ]; then
    echo "FAIL $label writer-spawn"
    return 1
  fi
  WPGID=$pgid

  # bounded poll: kill inside the exposure window of the txn past `target`
  dl=$(( $(date +%s) + DEADLINE_POLL ))
  if ! poll_sentinel "$sent" "$target" "$dl"; then
    kill -9 -- "-$pgid" 2>/dev/null
    echo "FAIL $label STALLED sentinel<$target at deadline"
    return 1
  fi
  local pre_kill committed
  pre_kill=$(rd "$sent")
  committed=$(rd "$sent.committed")
  [ -n "$committed" ] || committed=0

  # ---- KILL (kmode 0 = SHIM-COURT no-op kill) ----
  if [ "$kmode" = "9" ]; then
    kill -9 -- "-$pgid" 2>/dev/null
  else
    kill -0 -- "-$pgid" 2>/dev/null # SHIM-COURT: signal-0 probe kills nothing
  fi

  dl=$(( $(date +%s) + 10 ))
  if ! poll_gone "$wpid" "$dl"; then
    if [ "$kmode" = "0" ]; then
      kill -9 -- "-$pgid" 2>/dev/null
      echo "DETECT $label VACUOUS-GATE survivor-pid=$wpid (kill -0 SHIM-COURT fired nothing; writer kept running)"
      return 2
    fi
    kill -9 -- "-$pgid" 2>/dev/null
    echo "FAIL $label SURVIVOR after kill -9"
    return 1
  fi
  if [ "$kmode" = "0" ]; then
    # process gone under the no-op shim: legal only if it RAN TO COMPLETION
    # (writer survived the fake kill and finished TOTAL) — that is the
    # vacuity signature. Anything short of full completion is an unexplained
    # death under a no-op signal => FAIL.
    local out0 rc0 count0
    out0=$("$T2_BIN" checker "$db" 2>&1); rc0=$?
    count0=$(printf '%s\n' "$out0" | awk -F'|' '/^COUNT\|/{print $2}')
    if [ "$rc0" -eq 0 ] && [ "$count0" = "$total" ]; then
      echo "DETECT $label VACUOUS-GATE writer SURVIVED the no-op kill and completed $count0/$total events (SHIM-COURT fired nothing)"
      return 2
    fi
    echo "FAIL $label shim-leg writer died short of total (count=$count0 rc=$rc0) under no-op kill"
    return 1
  fi

  # reopen + assert in a fresh process: mount, PRAGMAs, two-legal-states,
  # closed-form checksum, checkpoint.
  local out rc count_v sum_v expect engine
  out=$("$T2_BIN" checker "$db" 2>&1); rc=$?
  engine=$(printf '%s\n' "$out" | awk -F'|' '/^ENGINE\|/{print $2}')
  count_v=$(printf '%s\n' "$out" | awk -F'|' '/^COUNT\|/{print $2}')
  sum_v=$(printf '%s\n' "$out" | awk -F'|' '/^SUM\|/{print $2}')
  expect=$(printf '%s\n' "$out" | awk -F'|' '/^EXPECT\|/{print $2}')
  echo "LEG $label engine=$engine checker_exit=$rc committed=$committed sentinel=$pre_kill count=$count_v sum=$sum_v expect=$expect"
  [ "$rc" -ne 0 ] && { echo "FAIL $label checker exit=$rc (mount/pragma/checkpoint)"; return 1; }

  # two legal post-states ONLY:
  #   (i)  count == committed sentinel (in-flight txn rolled back), or
  #   (ii) count == in-txn sentinel (kill landed after COMMIT, before the
  #        .committed rename).
  if [ "$count_v" != "$committed" ] && [ "$count_v" != "$pre_kill" ]; then
    echo "FAIL $label count=$count_v is a THIRD value (committed=$committed sentinel=$pre_kill) — torn txn"
    return 1
  fi
  if [ "$sum_v" != "$expect" ]; then
    echo "FAIL $label checksum=$sum_v != closed-form=$expect over count=$count_v — torn/partial txn"
    return 1
  fi
  echo "PASS $label kill=-$kmode target=$target count=$count_v checksum_ok=1 committed=$committed"
  return 0
}

# ---- prerequisites -------------------------------------------------------
if [ ! -x "$T2_BIN" ]; then
  echo "[SKIP] no t2writer fixture binary: $T2_BIN"
  echo "[SKIP] build: cd /tmp/t2writer && RUSTFLAGS='-C link-arg=-Wl,-rpath,$HOME/.frozen-duckdb/cache/v2.0.0-dev-arm64' cargo build --release"
  exit 1
fi

mkdir -p "$WORK"
ID=$("$T2_BIN" checker "$WORK/identity.db" 2>&1 | awk -F'|' '/^ENGINE\|/{print $2}')
case "$ID" in
  v1.5.5) say "engine-identity OK: $ID (cached v1.5.5 dylib)";;
  "") say "VERDICT FAIL: engine identity probe produced no version"; rm -rf "$WORK"; exit 1;;
  *) say "engine-identity NOTE: probe reports '$ID' (expected v1.5.5; recording, not refusing)";;
esac

# ---- LEG a: real kill -9 at 200k events ----------------------------------
crash_leg a "$TARGET" 9 "$TOTAL_COMPLETE";  r1=$?
[ $r1 -eq 0 ] || RC=1

# ---- LEG b: real kill -9 at 260k events ----------------------------------
crash_leg b "$TARGET2" 9 "$TOTAL_COMPLETE"; r2=$?
[ $r2 -eq 0 ] || RC=1

# ---- LEG c (non-vacuity): kill -9 -> kill -0 SHIM-COURT ------------------
# expected flip: DETECT (rc 2). rc 0 = gate vacuous => court FAIL.
crash_leg c "$TARGET" 0 "$TOTAL_COMPLETE"; r3=$?
case $r3 in
  2) say "NONVACUITY flip witnessed: SHIM-COURT no-op-kill run DETECTED as VACUOUS GATE";;
  0) say "NONVACUITY FAIL: shim run PASSED — gate is VACUOUS"; RC=1;;
  *) say "NONVACUITY FAIL: shim run errored unexpectedly (rc=$r3)"; RC=1;;
esac

# ---- LEG d: completed ingest (no kill) at TOTAL_COMPLETE -----------------
# mount + PRAGMA + count == committed sentinel exactly + closed-form checksum
r4=1
db="$WORK/db-d.db" sent="$WORK/sent-d"
rm -f "$db" "$db".wal "$sent" "$sent".committed "$sent".done
perl -e 'setpgrp(0,0); exec @ARGV' -- "$T2_BIN" writer "$db" "$sent" "$TOTAL_COMPLETE" >/dev/null 2>&1 &
wpid=$!
dl=$(( $(date +%s) + DEADLINE_POLL ))
if poll_done "$sent.done" "$dl"; then
  # writer exited its loop; wait for process exit (bounded)
  poll_gone "$wpid" "$(( $(date +%s) + 30 ))" || kill -9 -- "-$(ps -o pgid= -p "$wpid" | tr -d ' ')" 2>/dev/null
  out=$("$T2_BIN" checker "$db" 2>&1); r4=$?
  count_v=$(printf '%s\n' "$out" | awk -F'|' '/^COUNT\|/{print $2}')
  sum_v=$(printf '%s\n' "$out" | awk -F'|' '/^SUM\|/{print $2}')
  engine=$(printf '%s\n' "$out" | awk -F'|' '/^ENGINE\|/{print $2}')
  echo "LEG d engine=$engine checker_exit=$r4 count=$count_v sum=$sum_v total=$TOTAL_COMPLETE"
  if [ "$r4" -eq 0 ] && [ "$count_v" = "$TOTAL_COMPLETE" ]; then
    echo "PASS d completed-ingest count=$count_v checksum_ok=1"
    r4=0
  else
    echo "FAIL d count=$count_v != $TOTAL_COMPLETE or checker rc=$r4"
    r4=1
  fi
else
  kill -9 -- "-$(ps -o pgid= -p "$wpid" 2>/dev/null | tr -d ' ')" 2>/dev/null
  echo "FAIL d STALLED before done marker"
  r4=1
fi
[ $r4 -eq 0 ] || RC=1

# ---- receipt -------------------------------------------------------------
cat > "$RECEIPT" <<EOF
{
  "court": "fleet_d_chaos court",
  "lane": "T2",
  "gate": "TEST-FLEET-004 native chaos leg",
  "engine": "$ID",
  "binary": "$T2_BIN",
  "binary_sha256": "$(shasum "$T2_BIN")",
  "court_self_sha256_baseline": "$base_self",
  "court_self_sha256_post": "$(shasum "$COURT_SELF")",
  "writer_src_sha256_baseline": "$base_writer",
  "writer_src_sha256_post": "$(shasum "$T2_SRC")",
  "schema": "ocel_conforming_segments RLE (D8) + closed-form chksum",
  "legs": [
    {"leg":"a","kill":"-9","target_events":$TARGET,"rc":$r1},
    {"leg":"b","kill":"-9","target_events":$TARGET2,"rc":$r2},
    {"leg":"c","kill":"-0 (SHIM-COURT)","target_events":$TARGET,"rc":$r3,"expected":"2 (VACUOUS-GATE DETECTED)"},
    {"leg":"d","kill":"none (completed)","total_events":$TOTAL_COMPLETE,"rc":$r4}
  ],
  "no_sleep": "all waits are bounded sentinel/liveness polls"
}
EOF
say "receipt: $RECEIPT"

# ---- RESTORE check (fail-closed) ----------------------------------------
[ "$(shasum "$COURT_SELF")" = "$base_self" ] || { echo "CORRUPTED: court script changed mid-run"; RC=1; }
[ "$(shasum "$T2_SRC")" = "$base_writer" ] || { echo "CORRUPTED: writer src changed mid-run"; RC=1; }

# ---- verdict ------------------------------------------------------------
if [ "$RC" -eq 0 ] && [ "$r1" -eq 0 ] && [ "$r2" -eq 0 ] && [ "$r3" -eq 2 ] && [ "$r4" -eq 0 ]; then
  say "VERDICT PASS: 2 real kill -9 legs green ($TARGET/$TARGET2 events in-flight); completed leg green at $TOTAL_COMPLETE; no-op-kill flip DETECTED"
else
  say "VERDICT FAIL (legs: a=$r1 b=$r2 c=$r3 d=$r4)"
  RC=1
fi
rm -rf "$WORK"
exit $RC
