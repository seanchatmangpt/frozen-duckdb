#!/usr/bin/env bash
# GATE-V-02 / TEST-005 native crash-chaos court (lane Z2; re-dispatch of E2).
# Spec: docs/design/gate-v02-crash-chaos.md
#
# Claim: kill -9 DURING an open write transaction leaves the db
# (1) mountable, (2) WAL-replayed to a COMMITTED txn boundary,
# (3) checksum == closed-form(count) — one of exactly two legal
# values (last-committed sentinel, or one txn ahead at the in-txn
# sentinel), NEVER a third (a torn txn).
#
# Fixture: /tmp/z2writer (cargo, frozen-duckdb path dep, cached
# v2.0.0 dylib). Writer/checker subcommands, z2-suffixed
# names per lane Z2 ownership.
#
# BASELINE -> BEFORE -> SHIM -> AFTER -> RESTORE shape.
# Non-vacuity flip (V02-a): kill -9 -> kill -0 shim; the court must
# DETECT the survivor and fail the leg as VACUOUS GATE.
#
# No-sleep doctrine: every wait is a bounded poll on a real observable
# (sentinel file / process liveness) with a wall-clock deadline. No sleep N.
set -u

REPO=/Users/sac/frozen-duckdb
Z2_BIN=/tmp/z2writer/target/release/z2writer
Z2_SRC=/tmp/z2writer/src/main.rs
COURT_SELF="$REPO/scripts/courts/crash_chaos_native.sh"
WORK=/tmp/z2court.$$
TARGET=5000
TARGET2=5500
DEADLINE_POLL=120
RC=0
RECEIPT="${RECEIPT:-$REPO/target/court/ac2-receipt.json}"

mkdir -p "$REPO/target/court" || { echo "[SKIP] unwritable target/court"; exit 1; }

say() { printf '%s %s\n' "$(date -u +%H:%M:%S)" "$*"; }

shasum() { command shasum -a 256 "$@" | cut -d' ' -f1; }

# ---- bounded polls (no sleep N; spin on observables with deadline) -------
poll_sentinel() { # $1=file $2=min_count 3=deadline_epoch -> 0 ok / 1 STALLED
  local c
  while [ "$(date +%s)" -lt "$3" ]; do
    c=$(cat "$1" 2>/dev/null || echo 0)
    [ "$c" -ge "$2" ] && return 0
  done
  return 1
}

poll_gone() { # $1=pid $2=deadline -> 0 if process gone
  while [ "$(date +%s)" -lt "$2" ]; do
    kill -0 "$1" 2>/dev/null || return 0
  done
  return 1
}

rd() { head -1 "$1" 2>/dev/null | tr -d '[:space:]'; }

# ---- BASELINE -------------------------------------------------------------
base_self=$(shasum "$COURT_SELF")
base_writer=$(shasum "$Z2_SRC")

cleanup() {
  [ -n "${WPGID:-}" ] && kill -9 -- "-$WPGID" 2>/dev/null
}
trap cleanup EXIT

# ---- one crash leg -------------------------------------------------------
# crash_leg <label> <target> <kill_mode>   kill_mode: 9 | 0
# prints a PASS/FAIL/DETECT line; rc: 0 PASS, 1 FAIL, 2 DETECT (vacuity seen)
crash_leg() {
  local label=$1 target=$2 kmode=$3
  local db="$WORK/db-$label.db" sent="$WORK/sent-$label"
  mkdir -p "$WORK"
  rm -f "$db" "$db".wal "$sent" "$sent".committed "$sent".tmp

  # own process group (macOS has no setsid binary: perl setpgrp then exec
  # is the portable setsid)
  perl -e 'setpgrp(0,0); exec @ARGV' -- "$Z2_BIN" writer "$db" "$sent" >/dev/null 2>&1 &
  local wpid=$!
  local pgid="" dl
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

  # bounded poll until the in-txn sentinel crosses target (the kill fires
  # inside the exposure window of the txn carrying `target` rows)
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

  # ---- KILL (shim point: kmode 0 = SHIM-COURT no-op kill) ----
  if [ "$kmode" = "9" ]; then
    kill -9 -- "-$pgid" 2>/dev/null
  else
    kill -0 -- "-$pgid" 2>/dev/null # SHIM-COURT: no-op kill (signal 0 probe)
  fi

  # detection: victim must be GONE within a bounded window. Under the
  # shim, kill -0 kills nothing -> poll hits deadline -> DETECTED.
  dl=$(( $(date +%s) + 10 ))
  if ! poll_gone "$wpid" "$dl"; then
    if [ "$kmode" = "0" ]; then
      kill -9 -- "-$pgid" 2>/dev/null
      echo "DETECT $label VACUOUS-GATE survivor-pid=$wpid (kill -0 shim fired nothing; writer kept running)"
      return 2
    fi
    kill -9 -- "-$pgid" 2>/dev/null
    echo "FAIL $label SURVIVOR after kill -9"
    return 1
  fi

  # reopen + assert in a fresh process, no cleanup
  local out rc count_v sum_v expected engine
  out=$("$Z2_BIN" checker "$db" "$sent" 2>&1); rc=$?
  engine=$(printf '%s\n' "$out" | awk -F'|' '/^ENGINE\|/{print $2}')
  count_v=$(printf '%s\n' "$out" | awk -F'|' '/^COUNT\|/{print $2}')
  sum_v=$(printf '%s\n' "$out" | awk -F'|' '/^SUM\|/{print $2}')
  echo "LEG $label engine=$engine checker_exit=$rc committed=$committed sentinel=$pre_kill count=$count_v sum=$sum_v"
  [ "$rc" -ne 0 ] && { echo "FAIL $label checker exit=$rc (reopen/pragma/checkpoint)"; return 1; }

  # Two legal post-states ONLY (claim under test):
  #   (i)   count == committed sentinel (txn N+1 rolled back), or
  #   (ii)  count == in-txn sentinel (txn N+1 fully committed; the kill
  #         landed after COMMIT, before the .committed rename).
  # A third value, or sum != closed-form(count), is a torn txn => FAIL.
  if [ "$count_v" != "$committed" ] && [ "$count_v" != "$pre_kill" ]; then
    echo "FAIL $label count=$count_v is a THIRD value (committed=$committed sentinel=$pre_kill) — torn txn"
    return 1
  fi
  expected=$(( 3 * count_v * (count_v - 1) / 2 + 7 * count_v ))
  if [ "$sum_v" != "$expected" ]; then
    echo "FAIL $label checksum=$sum_v != closed-form=$expected over count=$count_v — torn/partial txn"
    return 1
  fi
  echo "PASS $label kill=-9 target=$target count=$count_v checksum_ok=1 committed=$committed"
  return 0
}

# ---- prerequisites -------------------------------------------------------
if [ ! -x "$Z2_BIN" ]; then
  echo "[SKIP] no z2writer fixture binary: $Z2_BIN"
  echo "[SKIP] build: cd /tmp/z2writer && RUSTFLAGS='-C link-arg=-Wl,-rpath,$HOME/.frozen-duckdb/cache/v2.0.0-arm64' cargo build --release"
  exit 1
fi

# engine identity gate: must be the cached verified dylib
mkdir -p "$WORK"
ID=$("$Z2_BIN" checker "$WORK/identity.db" /dev/null 2>&1 | awk -F'|' '/^ENGINE\|/{print $2}')
case "$ID" in
  v2.0.0) say "engine-identity OK: $ID (cached dylib)";;
  *) say "VERDICT FAIL: engine identity mismatch: got '$ID'"; rm -rf "$WORK"; exit 1;;
esac

# ---- LEG a: real kill -9 at TARGET ---------------------------------------
crash_leg a "$TARGET" 9;  r1=$?
[ $r1 -eq 0 ] || RC=1

# ---- LEG b: real kill -9 at TARGET+500 -----------------------------------
crash_leg b "$TARGET2" 9; r2=$?
[ $r2 -eq 0 ] || RC=1

# ---- LEG c (non-vacuity V02-a): kill -9 -> kill -0 shim ------------------
# expected flip: DETECT (rc 2). rc 0 = gate vacuous => court FAIL.
crash_leg c "$TARGET" 0; r3=$?
case $r3 in
  2) say "NONVACUITY flip witnessed: shim run DETECTED as VACUOUS GATE";;
  0) say "NONVACUITY FAIL: shim run PASSED — gate is VACUOUS"; RC=1;;
  *) say "NONVACUITY FAIL: shim run errored unexpectedly (rc=$r3)"; RC=1;;
esac

# ---- receipt -------------------------------------------------------------
cat > "$RECEIPT" <<EOF
{
  "court": "crash_chaos_native court",
  "lane": "Z2",
  "gate": "GATE-V-02/TEST-005 native leg",
  "spec": "docs/design/gate-v02-crash-chaos.md",
  "engine": "$ID",
  "binary": "$Z2_BIN",
  "binary_sha256": "$(shasum "$Z2_BIN")",
  "court_self_sha256_baseline": "$base_self",
  "court_self_sha256_post": "$(shasum "$COURT_SELF")",
  "writer_src_sha256_baseline": "$base_writer",
  "writer_src_sha256_post": "$(shasum "$Z2_SRC")",
  "legs": [
    {"leg":"a","kill":"-9","target":$TARGET,"rc":$r1},
    {"leg":"b","kill":"-9","target":$TARGET2,"rc":$r2},
    {"leg":"c","kill":"-0 (SHIM-COURT)","target":$TARGET,"rc":$r3,"expected":"2 (VACUOUS-GATE DETECTED)"}
  ],
  "no_sleep": "all waits are bounded sentinel/liveness polls"
}
EOF
say "receipt: $RECEIPT"

# ---- RESTORE check (fail-closed) ----------------------------------------
if [ "$(shasum "$COURT_SELF")" != "$base_self" ]; then
  echo "CORRUPTED: court script changed mid-run"; RC=1
fi
if [ "$(shasum "$Z2_SRC")" != "$base_writer" ]; then
  echo "CORRUPTED: writer src changed mid-run"; RC=1
fi

# ---- verdict ------------------------------------------------------------
if [ "$RC" -eq 0 ] && [ "$r1" -eq 0 ] && [ "$r2" -eq 0 ] && [ "$r3" -eq 2 ]; then
  say "VERDICT PASS: 2 real kill -9 legs green at TARGET and TARGET+500; no-op-kill flip DETECTED"
else
  say "VERDICT FAIL (legs: a=$r1 b=$r2 c=$r3)"
  RC=1
fi
rm -rf "$WORK"
exit $RC
