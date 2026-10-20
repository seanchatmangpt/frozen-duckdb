#!/usr/bin/env bash
# Phase 08 — performance (SLA): gate_v01_latency + gate_ingest_throughput as real cargo tests.
set -uo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
# Shared doctor build dir: one build paid across all doctor phases (kaizen LSS4).
export CARGO_TARGET_DIR="${DOCTOR_TARGET_DIR:-$ROOT/doctor-target}"
names="gate_v01_latency gate_ingest_throughput"
fail=0

if ! command -v cargo >/dev/null 2>&1; then
  echo "[SKIP] cargo not on PATH"
  exit 2
fi

for name in $names; do
  test_file="$ROOT/crates/frozen-duckdb/tests/${name}.rs"
  if [ ! -f "$test_file" ]; then
    echo "PHASE NOT-IMPLEMENTED (design: docs/design/doctor-verification-blueprint.md#phase-7): missing test file $test_file"
    fail=1
    continue
  fi
  if ! cargo test --release -p frozen-duckdb --test "$name" --quiet; then
    fail=1
  fi
done
exit "$fail"
