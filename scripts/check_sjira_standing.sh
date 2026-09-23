#!/usr/bin/env bash
# check_sjira_standing.sh — FDDB-26922-03 gate.
#
# Checks every v26.9.21 ticket's frontmatter `aps:standing:` against the
# ticket's FINAL History standing (last `| 20…` row), so the frontmatter can
# never silently drift from the History record. CI_PROJECTION.md is excluded
# (it is a projection doc, not a ticket).
#
# Explicit non-aps final-row mappings (decided FDDB-26922-03; trailing 帳
# ledger / routing rows are annotations and do NOT demote the terminal-DoD
# standing; a ticket whose final standing ROW ITSELF is a standing value uses
# that value verbatim):
#   T2B  "UNSUPPORTED ledger"  -> ALIVE         (帳 row trailing; terminal DoD
#                                               row ALIVE @1062719, merged)
#   TR2  "UNSUPPORTED"         -> UNSUPPORTED   (final row IS the standing:
#                                               hand-hardened workflow lines)
#   TR3  "LEDGER (帳)"          -> ALIVE         (帳 row trailing; terminal
#                                               ALIVE @4e77b3f, merged)
#   TR7  "帳: UNSUPPORTED(...)" -> ALIVE         (NOTE + 帳 trailing; terminal
#                                               ALIVE, merged @1865954)
#   G9   "RECORDED"            -> ALIVE         (cross-pack routing row
#                                               trailing; terminal ALIVE,
#                                               merged @c47375c03)
set -uo pipefail
cd "$(dirname "$0")/.."

DIR="docs/sjira/v26.9.21"
mismatches=0
checked=0

# file:mapped-standing overrides for non-aps final rows (see header).
declare -a EXPLICIT=(
  "T2B.md:ALIVE"
  "TR2.md:UNSUPPORTED"
  "TR3.md:ALIVE"
  "TR7.md:ALIVE"
  "G9.md:ALIVE"
)
explicit_for() {
  local f="$1" e
  for e in "${EXPLICIT[@]}"; do
    if [ "${e%%:*}" = "$f" ]; then
      echo "${e#*:}"
      return 0
    fi
  done
  return 1
}

for f in "$DIR"/T*.md "$DIR"/TR*.md "$DIR"/C[0-9]*.md "$DIR"/G*.md "$DIR"/TPUB.md; do
  [ -f "$f" ] || continue
  base="$(basename "$f")"
  [ "$base" = "CI_PROJECTION.md" ] && continue
  checked=$((checked + 1))

  fm="$(grep -m1 '^aps:standing:' "$f" | sed 's|.*aps#||')"
  if [ -z "$fm" ]; then
    echo "MISMATCH $base: no frontmatter aps:standing line"
    mismatches=$((mismatches + 1))
    continue
  fi

  if expect="$(explicit_for "$base")"; then
    final="$expect"
  else
    # final History standing = last-row standing column, first token
    # (trailing parenthetical annotations are not part of the standing).
    final="$(grep '^| 20' "$f" | tail -1 | awk -F'|' '{print $3}' | xargs | awk '{print $1}')"
  fi

  if [ "$fm" != "$final" ]; then
    echo "MISMATCH $base: frontmatter=$fm final-history=$final"
    mismatches=$((mismatches + 1))
  fi
done

echo "checked=$checked mismatches=$mismatches"
[ "$mismatches" -eq 0 ]
