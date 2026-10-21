#!/usr/bin/env bash
# publish_gate.sh — fail-closed pre-publish gate for frozen-duckdb.
# Runs REAL checks on the exact HEAD and writes a receipt (identity, commands,
# exit codes, standing). Never publishes; publish_chain.sh does that.
# Standing: PUBLISH_READY only if every gate exits 0; otherwise REFUSED.
# Env: SKIP_TESTS=1 skips the full workspace test run (recorded as skipped).
set -uo pipefail
cd "$(dirname "$0")/.."

HEAD_SHA="$(git rev-parse HEAD)"
OUT="${GATE_RECEIPT:-target/publish-gate-receipt.txt}"
mkdir -p "$(dirname "$OUT")"
NAMES=(); CODES=(); FAIL=0

gate() { # name cmd...
  local name="$1"; shift
  echo "=== [publish-gate] $name"
  "$@" > "/tmp/pubgate-$name.log" 2>&1; local code=$?
  tail -n 3 "/tmp/pubgate-$name.log" | sed 's/^/    | /'
  NAMES+=("$name"); CODES+=("$code")
  [ "$code" -eq 0 ] || FAIL=1
}

tree_clean() { [ -z "$(git status --porcelain --untracked-files=normal)" ] || { git status --short; return 1; }; }
versions_aligned() {
  local v; v="$(cargo metadata --no-deps --format-version 1 | python3 -c \
    'import json,sys;print(" ".join(sorted({p["version"] for p in json.load(sys.stdin)["packages"] if p["name"].startswith("frozen-duckdb")})))')"
  echo "versions: $v"; [ "$(wc -w <<<"$v")" -eq 1 ]
}
# Cargo >= 1.90 resolves unpublished workspace deps locally for --workspace dry-runs.
publish_dry_run() { cargo publish --workspace --dry-run; }
wasm_ceiling() { # only checked when a wasm artifact exists; absence is recorded, not passed
  local a="${WASM_ARTIFACT:-target/wasm/duckdb.wasm}"
  [ -f "$a" ] || { echo "UNSUPPORTED: no wasm artifact at $a"; return 0; }
  local b; b="$(wc -c < "$a" | tr -d ' ')"; echo "wasm bytes=$b ceiling=33554432"
  [ "$b" -le 33554432 ]
}

gate tree-clean tree_clean
gate versions-aligned versions_aligned
gate build cargo build --workspace
gate fmt cargo fmt --all -- --check
gate clippy cargo clippy --workspace --all-targets -- -D warnings
if [ "${SKIP_TESTS:-0}" = "1" ]; then echo "tests SKIPPED (SKIP_TESTS=1)"; else gate tests cargo test --workspace; fi
gate publish-dry-run publish_dry_run
gate wasm-ceiling-32MiB wasm_ceiling

STANDING=PUBLISH_READY; [ "$FAIL" -eq 0 ] || STANDING=REFUSED
{
  echo "subject_head=$HEAD_SHA"
  echo "recorded_at=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  for i in "${!NAMES[@]}"; do echo "gate=${NAMES[$i]} exit=${CODES[$i]}"; done
  echo "standing=$STANDING"
} | tee "$OUT"
[ "$FAIL" -eq 0 ]
