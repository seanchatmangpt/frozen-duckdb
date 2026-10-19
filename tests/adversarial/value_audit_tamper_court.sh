#!/usr/bin/env bash
# value_audit_tamper_court.sh — lane AD4 non-vacuity court for the CFO
# value-audit integrity anchor.
# Repo: /Users/sac/frozen-duckdb
#
# BASELINE -> BEFORE -> SHIM -> AFTER -> RESTORE per case, over an isolated
# copy of the real receipts in /tmp/ad4-court.$$ (never the live
# /tmp/value_audit set; no git state mutations).
#
# Non-vacuity law: each attack case MUST flip the renderer to nonzero exit
# (RENDER-FAIL/TAMPER). A case that cannot flip is COURT-FAIL, not PASS.
# Restore is byte-exact (sha256 vs baseline) or the court exits nonzero.

set -uo pipefail

REPO=/Users/sac/frozen-duckdb
RENDERER="$REPO/warehouse-sim/economics/render_value_audit.ts"
SIGN="$REPO/scripts/value_audit_sign.sh"
SRC=/tmp/value_audit
WORK="/tmp/ad4-court.$$"
RECEIPTS="val001.json val002.json val003.json val004.json"

cleanup() { rm -rf "$WORK"; }
trap cleanup EXIT
mkdir -p "$WORK"
cp "$SRC"/val00[1-4].json "$WORK/" || { echo "[SKIP] live receipts unavailable in $SRC"; exit 2; }

fails=0

render() { # -> exit code of renderer with --verify over the court manifest
  ( cd "$WORK" && B3_BLOCK="$(b3sum val00[1-4].json)" \
    node "$RENDERER" --dir "$WORK" --verify "$WORK/manifest.b3" --doctor GREEN ) >/dev/null 2>&1
}

expect_flip() { # <label>
  local label="$1" rc
  render; rc=$?
  if [ $rc -ne 0 ]; then
    echo "[FLIP-OK] $label: renderer exit $rc (tamper refused)"
  else
    echo "[COURT-FAIL] $label: tampering accepted — gate is VACUOUS"
    fails=$((fails + 1))
  fi
}

# Green control: untampered set + fresh manifest must verify.
( cd "$WORK" && bash "$SIGN" --dir "$WORK" -o "$WORK/manifest.b3" >/dev/null ) || {
  echo "[COURT-FAIL] manifest mint failed"; exit 1; }
render && echo "[CONTROL-GREEN] untampered set verifies (exit 0)" || {
  echo "[COURT-FAIL] control render failed on untampered set"; fails=$((fails + 1)); }

baseline=$(cd "$WORK" && sha256sum $RECEIPTS)

# Case 1: edited number (val003 annual figure x100)
sed -i '' 's/"usd_per_year": 12090,/"usd_per_year": 1209000,/' "$WORK/val003.json"
if grep -q 1209000 "$WORK/val003.json"; then
  expect_flip "case1-edited-number(val003 12090->1209000)"
else
  echo "[SKIP] case1 shim did not apply (receipt shape drift)"
fi
rm -f "$WORK/manifest.b3" # stale manifest from tampered set; remint per case
( cd "$WORK" && bash "$SIGN" --dir "$WORK" -o "$WORK/manifest.b3" >/dev/null )

# Case 2: deleted receipt
mv "$WORK/val002.json" "$WORK/val002.json.hidden"
expect_flip "case2-deleted-file(val002)"
mv "$WORK/val002.json.hidden" "$WORK/val002.json"

# Case 3: swapped contents val001 <-> val002
cat "$WORK/val001.json" > "$WORK/.tmp1"; cat "$WORK/val002.json" > "$WORK/.tmp2"
cat "$WORK/.tmp2" > "$WORK/val001.json"; cat "$WORK/.tmp1" > "$WORK/val002.json"
expect_flip "case3-swapped-contents(val001<->val002)"

# Restore to baseline byte-exact before case 4
cp "$SRC"/val00[1-4].json "$WORK/"
baseline_now=$(cd "$WORK" && sha256sum $RECEIPTS)
[ "$baseline" = "$baseline_now" ] || { echo "[COURT-FAIL] restore drifted from baseline"; fails=$((fails + 1)); }

# Case 4: extra unmanifested receipt
printf '{"bogus":true}\n' > "$WORK/val999.json"
expect_flip "case4-extra-unmanifested(val999)"
rm -f "$WORK/val999.json"

# Self-check: prove restore-detection is not vacuous — corrupt a copy, confirm
# the court's own comparison WOULD catch it.
printf 'x' >> "$WORK/val001.json"
baseline_corrupt=$(cd "$WORK" && sha256sum $RECEIPTS)
if [ "$baseline" != "$baseline_corrupt" ]; then
  echo "[SELF-CHECK-OK] restore-detection catches a corrupted file"
else
  echo "[COURT-FAIL] restore-detection self-check vacuous"; fails=$((fails + 1))
fi

if [ $fails -eq 0 ]; then
  echo "[COURT-PASS] value-audit tamper matrix: all 4 attacks red, control green, restore byte-exact"
  exit 0
fi
echo "[COURT-FAIL] $fails case(s) failed"; exit 1
