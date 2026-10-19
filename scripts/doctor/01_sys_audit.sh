#!/usr/bin/env bash
# Phase 01 — sys audit: run checks/bin_audit.sh and propagate its verdict.
# Exit contract: 0 pass / 1 fail / 2 skip / 3 NOT-IMPLEMENTED (missing subject
# or cannot derive VERSION — loud fail-closed, never a substituted dylib).
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
CHECK="$ROOT/scripts/doctor/checks/bin_audit.sh"

if [ ! -x "$CHECK" ]; then
  echo "PHASE NOT-IMPLEMENTED (design: docs/design/doctor-verification-blueprint.md#phase-1): missing or non-executable $CHECK"
  exit 3
fi

bash "$CHECK"
