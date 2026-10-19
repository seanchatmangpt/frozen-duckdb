#!/usr/bin/env bash
# Phase 02 — storage/WAL: real cargo test when the engine is available;
# loud NOT-IMPLEMENTED while engine rebuild is in flight.
# Owner test file (D4): crates/frozen-duckdb/tests/doctor_storage_tests.rs
set -uo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
# Shared doctor build dir: one build paid across all doctor phases (kaizen LSS4).
export CARGO_TARGET_DIR="${DOCTOR_TARGET_DIR:-$ROOT/doctor-target}"
name="doctor_storage_tests"
test_file="$ROOT/crates/frozen-duckdb/tests/${name}.rs"

if ! command -v cargo >/dev/null 2>&1; then
  echo "[SKIP] cargo not on PATH"
  exit 2
fi

# Engine availability probe: a real (fast) build+link of the workspace lib.
if ! cargo build -p frozen-duckdb --quiet 2>&1 | tail -5; then
  echo "PHASE NOT-IMPLEMENTED (design: docs/design/doctor-verification-blueprint.md#phase-1): engine rebuild in flight"
  exit 3
fi

if [ ! -f "$test_file" ]; then
  echo "PHASE NOT-IMPLEMENTED (design: docs/design/doctor-verification-blueprint.md#phase-1): missing test file $test_file"
  exit 3
fi

exec cargo test -p frozen-duckdb --test "$name" --quiet
