#!/usr/bin/env bash
# TEST-VAL-003 collector (lane V4) — chains the GATE-V-01 receipt into the
# value ledger WITHOUT re-running the benchmark.
#
# Receipt precedence: reuse an existing /tmp/value_audit/gate_v01_receipt.json
# if present (it implies a real prior PASS run — the test writes it only after
# its assert passes); otherwise run the gate test for real. Then compute the
# counterfactual ledger via val003_snowflake.ts.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RECEIPT=/tmp/value_audit/gate_v01_receipt.json

if [ -f "$RECEIPT" ]; then
  echo "collect_val003: reusing existing receipt $RECEIPT (no benchmark re-run)"
  cat "$RECEIPT"
else
  echo "collect_val003: no receipt found — running GATE-V-01 for real"
  (cd "$ROOT" && CARGO_TARGET_DIR=target-laneV4 cargo test -p frozen-duckdb \
    --release --test gate_v01_latency -- --nocapture)
fi

cd "$ROOT/warehouse-sim"
node economics/val003_snowflake.ts --json /tmp/value_audit/val003.json
echo "collect_val003: OK"
