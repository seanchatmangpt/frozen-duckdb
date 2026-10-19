#!/usr/bin/env bash
# GATE-V aggregate runner — /Users/sac/frozen-duckdb/scripts/courts/run_all.sh
#
# Contract (fail-closed):
#   - Executes gate_v01..gate_v05 in order.
#   - Verdicts: PASS (exit 0) | SKIP (exit 2 with printed reason) |
#     NOT-IMPLEMENTED (exit 3) | FAIL (any other nonzero exit).
#   - Exit 0 ONLY when every court returned PASS or SKIP-with-reason.
#   - Any FAIL or NOT-IMPLEMENTED => exit 1 with the full verdict table.
#   - Loud skips are registered divergences; NOT-IMPLEMENTED is never
#     rendered as a green cell.
set -u
COURT_DIR="$(cd "$(dirname "$0")" && pwd)"
GATES="gate_v01 gate_v02 gate_v03 gate_v04 gate_v05"

rows=()
any_fail=0

for g in $GATES; do
  f="$COURT_DIR/${g}.sh"
  if [ ! -x "$f" ]; then
    rows+=("$g|NOT-IMPLEMENTED|missing or non-executable: $f")
    any_fail=1
    continue
  fi
  out="$("$f" 2>&1)"
  rc=$?
  case "$rc" in
    0) rows+=("$g|PASS|$out") ;;
    2) rows+=("$g|SKIP|${out:-<no reason printed>}") ;;
    3) rows+=("$g|NOT-IMPLEMENTED|$(echo "$out" | head -1)") ; any_fail=1 ;;
    *) rows+=("$g|FAIL|rc=$rc $(echo "$out" | head -1)"); any_fail=1 ;;
  esac
done

echo "=== GATE-V verdict table ==="
printf '%-10s %-16s %s\n' COURT VERDICT DETAIL
for r in "${rows[@]}"; do
  court="${r%%|*}"; rest="${r#*|}"
  verdict="${rest%%|*}"; detail="${rest#*|}"
  printf '%-10s %-16s %s\n' "$court" "$verdict" "$detail"
done

if [ "$any_fail" -ne 0 ]; then
  echo "GATE-V: FAIL (fail-closed: NOT-IMPLEMENTED and FAIL both block)"
  exit 1
fi
echo "GATE-V: PASS"
exit 0
