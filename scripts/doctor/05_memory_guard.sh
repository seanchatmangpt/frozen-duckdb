#!/usr/bin/env bash
# Phase 05 — memory guard: doctor_leak_audit + oom_graceful_abort.
set -uo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
names="doctor_leak_audit oom_graceful_abort"
fail=0

if ! command -v cargo >/dev/null 2>&1; then
  echo "[SKIP] cargo not on PATH"
  exit 2
fi

for name in $names; do
  test_file="$ROOT/crates/frozen-duckdb/tests/${name}.rs"
  if [ ! -f "$test_file" ]; then
    echo "PHASE NOT-IMPLEMENTED (design: docs/design/doctor-verification-blueprint.md#phase-4): missing test file $test_file"
    fail=1
    continue
  fi
  if ! cargo test -p frozen-duckdb --test "$name" --quiet; then
    fail=1
  fi
done
exit "$fail"
