#!/usr/bin/env bash
# Non-vacuity court for the P8 lane-lease guard (shim-court pattern).
# BASELINE -> BEFORE -> SHIM -> AFTER -> RESTORE. A court that cannot flip is FAIL.
set -u
cd "$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
G=scripts/check_lane_leases.sh
LEASE=target-laneSHIMCOURT
rc=0

# BASELINE: no shim lease present
[ -e "$LEASE" ] && { echo "FAIL: $LEASE pre-exists"; exit 1; }

# BEFORE: guard must be green with no leases (note: repo has real leases, so
# this court runs the guard from a lease-free temp cwd via its own copy)
before=$(mktemp -d)
cp "$G" "$before/"
if (cd "$before" && bash check_lane_leases.sh >/dev/null 2>&1); then
  echo "BEFORE ok (exit 0, no leases)"
else
  echo "FAIL: guard red with zero leases"; rm -rf "$before"; exit 1
fi

# SHIM: mint a marked lease
mkdir -p "$before/$LEASE" && echo "SHIM-COURT" > "$before/$LEASE/marker"
after_rc=0
(cd "$before" && bash check_lane_leases.sh >/dev/null 2>&1) || after_rc=$?
if [ "$after_rc" -ne 0 ]; then
  echo "AFTER flipped (exit $after_rc) — non-vacuity proven"
else
  echo "FAIL VACUOUS GATE: guard accepted shim lease"; rc=1
fi

# RESTORE: shim lease removed, fail-closed byte-exact listing check
rm -rf "$before/$LEASE"
ls -A "$before" | grep -v check_lane_leases.sh | grep -q . && { echo "FAIL CORRUPTED: residue in court dir"; rc=1; }
rm -rf "$before"
[ "$rc" -eq 0 ] && echo "PASS" || echo "COURT FAIL"
exit "$rc"
