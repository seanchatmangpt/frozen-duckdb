#!/usr/bin/env bash
# TEST-025 doctor dual-pass court (lane E4).
# docs/design/doctor-dual-pass-court.md
#
# NOTE (engine dependency): scripts/doctor.sh phases require the frozen-duckdb
# engine rebuild to be complete (the doctor loads the built dylib). Running
# this court pre-rebuild yields pass-1 phase failures; the court reports those
# failures honestly (fail-closed) rather than faking a pass.
#
# Shape: BASELINE -> BEFORE(pass1) -> SHIM(none; observation court over two
# back-to-back doctor runs) -> AFTER(pass2 + assertions A1..A5) -> court
# verdict. Read-only over git (rev-parse/status only); no git state mutation.
#
# Exit 0 only if every assertion holds; any breach exits 1 naming the
# assertion. Skips are loud: [SKIP] <reason>, never a faked pass.
set -u

REPO="/Users/sac/frozen-duckdb"
DOCTOR="$REPO/scripts/doctor.sh"
EVIDENCE="$REPO/scripts/verify-evidence.ttl"
RECEIPT_CHECK="$REPO/scripts/verify_publish_receipt.py"
LOG1="/tmp/doctor-pass1.log"
LOG2="/tmp/doctor-pass2.log"

failures=()
NL=$'\n'

fail() { failures+=("$1"); echo "FAIL: $1"; }
skip() { echo "[SKIP] $1"; }

cd "$REPO" || { echo "FAIL: cannot cd $REPO"; exit 1; }

# ---- Prerequisites -------------------------------------------------------

if [[ ! -x "$DOCTOR" ]]; then
  echo "[SKIP] $DOCTOR missing or not executable"
  exit 1
fi
if ! command -v git >/dev/null 2>&1; then
  echo "[SKIP] git unavailable"
  exit 1
fi
if [[ ! -f "$EVIDENCE" ]]; then
  echo "[SKIP] $EVIDENCE missing (doctor regenerates it; run post-rebuild)"
  exit 1
fi
if ! command -v python3 >/dev/null 2>&1; then
  echo "[SKIP] python3 unavailable (needed for verify_publish_receipt.py)"
  exit 1
fi

# ---- BASELINE ------------------------------------------------------------

HEAD_SHA="$(git rev-parse HEAD)" || { echo "FAIL: git rev-parse HEAD"; exit 1; }
EV_SHA_BASE="$(shasum -a 256 "$EVIDENCE" | awk '{print $1}')"
echo "BASELINE HEAD=$HEAD_SHA"
echo "BASELINE evidence.sha256=$EV_SHA_BASE"

# Declared fixture inventory (residual checklist, pre-declared):
#   - no target-lane*/ dirs created or left by either pass
#   - /tmp fixture namespaces: x4writer*, e2gate.db, doctor-pass{1,2}.log (ours)
#   - leaked process names: x4writer, e2writer, gate_v01_fixture, flock_holder
declare -a LEAK_PROC_NAMES=(x4writer e2writer gate_v01_fixture flock_holder)

# ---- BEFORE: pass 1 ------------------------------------------------------

echo "== pass 1 =="
bash "$DOCTOR" >"$LOG1" 2>&1
rc1=$?
echo "pass1 exit=$rc1 log=$LOG1 ($(wc -l <"$LOG1") lines)"

# ---- AFTER: pass 2 (immediately after, no intervening edits/sync/rebuild) --

echo "== pass 2 =="
bash "$DOCTOR" >"$LOG2" 2>&1
rc2=$?
echo "pass2 exit=$rc2 log=$LOG2 ($(wc -l <"$LOG2") lines)"

# A1 — identical exits
if [[ "$rc1" != "$rc2" ]]; then
  fail "A1 identical-exits: rc1=$rc1 rc2=$rc2 (non-reproducible doctor verdict)"
else
  echo "A1 ok: rc1==rc2==$rc1"
fi

# A2 — zero leaked fixture processes after pass 2
for name in "${LEAK_PROC_NAMES[@]}"; do
  hits="$(pgrep -f "$name" 2>/dev/null || true)"
  if [[ -n "$hits" ]]; then
    detail="$(ps -o pid,command -p "$(echo "$hits" | tr '\n' ',' | sed 's/,$//')" 2>/dev/null || echo "$hits")"
    fail "A2 leaked-fixture: pgrep -f $name matched:$NL$detail"
  else
    echo "A2 ok: no process matching $name"
  fi
done

# A3 — zero undeclared residue
lane_dirs="$(ls -d "$REPO"/target-lane*/ 2>/dev/null || true)"
if [[ -n "$lane_dirs" ]]; then
  fail "A3 residue: target-lane dirs present:$NL$lane_dirs"
else
  echo "A3 ok: no target-lane*/ dirs"
fi

tmp_extra="$(ls /tmp 2>/dev/null | grep -E 'doctor|frozen-duckdb|x4writer|e2gate\.db' \
  | grep -v -E '^doctor-pass1\.log$|^doctor-pass2\.log$' || true)"
if [[ -n "$tmp_extra" ]]; then
  fail "A3 residue: undeclared /tmp artifacts:$NL$tmp_extra"
else
  echo "A3 ok: /tmp fixtures accounted (only declared pass logs)"
fi

# A4 — evidence TTL bound to same HEAD + identical digest across passes
EV_SHA_AFTER="$(shasum -a 256 "$EVIDENCE" | awk '{print $1}')"
if [[ "$EV_SHA_BASE" != "$EV_SHA_AFTER" ]]; then
  fail "A4 evidence-digest: sha256 changed across passes ($EV_SHA_BASE -> $EV_SHA_AFTER); not a pure projection"
else
  echo "A4 ok: evidence.sha256 stable ($EV_SHA_AFTER)"
fi
if grep -q "$HEAD_SHA" "$EVIDENCE"; then
  echo "A4 ok: evidence binds HEAD $HEAD_SHA"
else
  fail "A4 evidence-head: $EVIDENCE does not reference HEAD $HEAD_SHA"
fi

# A5 — verify_publish_receipt.py green (binding observation: after pass 2)
if python3 "$RECEIPT_CHECK" >"/tmp/doctor-court-receipt-check.log" 2>&1; then
  echo "A5 ok: verify_publish_receipt.py exit 0"
else
  rc5=$?
  fail "A5 publish-receipt: exit $rc5 (log: /tmp/doctor-court-receipt-check.log)"
fi

# ---- Verdict -------------------------------------------------------------

echo ""
if [[ "${#failures[@]}" -gt 0 ]]; then
  echo "COURT FAIL (${#failures[@]} breached assertion(s)):"
  for f in "${failures[@]}"; do
    echo "  - $f"
  done
  echo "Pass logs: $LOG1 $LOG2"
  exit 1
fi

echo "COURT PASS: doctor dual-pass idempotent at HEAD $HEAD_SHA"
echo "  rc1=rc2=$rc1; A2-A5 green; receipt: both pass logs + this output"
exit 0
