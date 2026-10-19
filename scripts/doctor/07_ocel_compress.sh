#!/usr/bin/env bash
# Phase 07 — OCEL: event-log compression/roundtrip coverage.
# Owner test file per D8: crates/frozen-duckdb/tests/doctor_ocel_tests.rs
set -uo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
name="doctor_ocel_tests"
test_file="$ROOT/crates/frozen-duckdb/tests/${name}.rs"

if ! command -v cargo >/dev/null 2>&1; then
  echo "[SKIP] cargo not on PATH"
  exit 2
fi
if [ ! -f "$test_file" ]; then
  echo "PHASE NOT-IMPLEMENTED (design: docs/design/doctor-verification-blueprint.md#phase-6): missing test file $test_file"
  exit 3
fi
exec cargo test -p frozen-duckdb --test "$name" --quiet
