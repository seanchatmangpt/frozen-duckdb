#!/usr/bin/env bash
# Phase 10 — verify economics: run the warehouse-sim economics court.
# Exit contract: 0 pass / 1 fail / 2 skip / 3 NOT-IMPLEMENTED (court script
# absent — loud divergence, never a faked economics pass).
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
# lane Z7: path fix — the court lives at warehouse-sim/economics/run_court.sh
# (single source; not duplicated under scripts/warehouse-sim/).
COURT="$ROOT/warehouse-sim/economics/run_court.sh"

if [ ! -x "$COURT" ]; then
  echo "PHASE NOT-IMPLEMENTED (design: docs/design/doctor-verification-blueprint.md#phase-10): missing or non-executable $COURT"
  exit 3
fi

bash "$COURT"
