#!/usr/bin/env bash
# value_audit_sign.sh — mint the detached integrity manifest for the CFO
# value-audit render (court-mint time, over the REAL receipts).
# Repo: /Users/sac/frozen-duckdb
#
# INTEGRITY ANCHOR LAW: the b3sum block the renderer prints is computed over
# whatever files are on disk at render time, so it "verifies" any edited
# number (witnessed: val003 12090 -> 1209000 rendered GREEN, exit 0). The
# anchor is therefore NOT the render-time digest — it is this detached
# manifest, minted once at court-mint time and checked by
# `render_value_audit.ts --verify <manifest>` BEFORE any rendering.
#
# Usage: value_audit_sign.sh [--dir /tmp/value_audit] [-o manifest.b3]
#   Manifest format (b3sum-compatible, comment lines start with '#'):
#     # value-audit manifest minted <UTC timestamp>
#     # subject: <git rev-parse HEAD> (read-only binding; no git mutations)
#     <b3>  val001.json
#     ...
#
# Exit 0 only if every receipt exists and the manifest was written; exit 1
# otherwise (loud, never a faked mint).

set -euo pipefail

DIR=/tmp/value_audit
OUT=""
while [ $# -gt 0 ]; do
  case "$1" in
    --dir) DIR="$2"; shift 2 ;;
    -o|--out) OUT="$2"; shift 2 ;;
    *) echo "usage: value_audit_sign.sh [--dir DIR] [-o MANIFEST]" >&2; exit 1 ;;
  esac
done

OUT="${OUT:-$DIR/value-audit.manifest}"

RECEIPTS="val001.json val002.json val003.json val004.json"
for r in $RECEIPTS; do
  [ -s "$DIR/$r" ] || { echo "SIGN-FAIL: receipt $DIR/$r missing/empty" >&2; exit 1; }
done

command -v b3sum >/dev/null 2>&1 || { echo "SIGN-FAIL: b3sum not on PATH" >&2; exit 1; }
command -v git   >/dev/null 2>&1 || { echo "SIGN-FAIL: git not on PATH" >&2; exit 1; }

SUBJECT="$(git -C "$(cd "$(dirname "$0")/.." && pwd)" rev-parse HEAD 2>/dev/null || echo "no-git-subject")"

{
  echo "# value-audit manifest minted $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "# subject: $SUBJECT"
  (cd "$DIR" && b3sum $RECEIPTS)
} > "$OUT"

echo "[MINT] $OUT"
cat "$OUT"
