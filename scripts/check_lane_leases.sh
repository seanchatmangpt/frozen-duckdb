#!/usr/bin/env bash
# P8 lane-lease guard (poka-yoke, lane LSS3) — fail-closed.
# Rejects the integration point (commit / --check) while any
# CARGO_TARGET_DIR=target-lane* build root exists under repo root.
# Law: same-checkout-fanout cleanup law — lane leases are leases, not assets.
set -u
cd "$(git rev-parse --show-toplevel 2>/dev/null || pwd)"

shopt -s nullglob
leases=(target-lane*/)
if [ ${#leases[@]} -eq 0 ]; then
  echo "[ok] no lane build roots present — integration lawful"
  exit 0
fi

total=0
echo "REFUSED: lane build roots present (leases, not assets):" >&2
for d in "${leases[@]}"; do
  sz=$(du -sh "$d" 2>/dev/null | cut -f1); sz=${sz:-?}
  echo "  ${d%/}  ${sz}" >&2
  total=$((total+1))
done
echo "total: ${total} lease(s). Coordinator deletes at integration, before commit." >&2
echo "Remediation: rm -rf target-lane*/  (operator action)." >&2
exit 1
