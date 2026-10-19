#!/usr/bin/env bash
# Phase 03 — types/variant: doctor_types_tests + variant_tests.
set -uo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
names="doctor_types_tests variant_tests"
fail=0

if ! command -v cargo >/dev/null 2>&1; then
  echo "[SKIP] cargo not on PATH"
  exit 2
fi

for name in $names; do
  test_file="$ROOT/crates/frozen-duckdb/tests/${name}.rs"
  if [ ! -f "$test_file" ]; then
    echo "PHASE NOT-IMPLEMENTED (design: docs/design/doctor-verification-blueprint.md#phase-2): missing test file $test_file"
    fail=1
    continue
  fi
  if ! cargo test -p frozen-duckdb --test "$name" --quiet; then
    fail=1
  fi
done
exit "$fail"
