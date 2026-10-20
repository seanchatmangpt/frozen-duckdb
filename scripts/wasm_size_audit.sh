#!/usr/bin/env bash
# wasm_size_audit.sh — GAP1b close-condition court (lane ADV13)
# SLA-2 ladder: raw <=25MiB, wire (gzip -9) <=10MiB.
# Stages: llvm-strip/wasm-strip -> wasm-opt --all-features -Os -> gzip -9
#         -> wasm-objdump -h section census.
# Fail-closed: any tool missing/failed = loud [SKIP], never a faked pass.
set -u

WASM="${1:-target/wasm/duckdb.wasm}"
if [ ! -f "$WASM" ]; then
  echo "FATAL: artifact not found: $WASM" >&2
  exit 2
fi
WASM="$(cd "$(dirname "$WASM")" && pwd)/$(basename "$WASM")"

RAW_SLA=$((25 * 1024 * 1024))
WIRE_SLA=$((10 * 1024 * 1024))
WORK="$(mktemp -d /tmp/wasm-court.XXXXXX)"
trap 'rm -rf "$WORK"' EXIT
PASS=1
emit() { printf '%s\n' "$*"; }
sz() { stat -f%z "$1"; }
fmt_mib() { awk -v b="$1" 'BEGIN{printf "%.2fMiB", b/1048576}'; }

# find_tool: prefer emsdk's binary, fall back to PATH
find_tool() {
  local name="$1" p
  for p in "$HOME/emsdk/upstream/bin/$name" "$(command -v "$name" 2>/dev/null)"; do
    [ -n "$p" ] && [ -x "$p" ] && { printf '%s' "$p"; return 0; }
  done
  return 1
}

emit "=== WASM size court (GAP1b / SLA-2) — $(date -u +%FT%TZ) ==="
emit "subject: $WASM"
BASE=$(sz "$WASM")
emit "baseline: $BASE bytes ($(fmt_mib "$BASE"))"
emit "SLA ladder: raw <=25MiB ($RAW_SLA) / wire gzip -9 <=10MiB ($WIRE_SLA)"
emit ""

# --- Rung 1: strip ---
STRIP="$(find_tool llvm-strip || true)"
[ -z "$STRIP" ] && STRIP="$(find_tool wasm-strip || true)"
if [ -z "$STRIP" ]; then
  emit "[SKIP] no llvm-strip / wasm-strip — rung 1 skipped (loud)"
  PASS=0
  S1="$WASM"
else
  emit "--- rung 1: strip ($(basename "$STRIP")) ---"
  S1="$WORK/stripped.wasm"
  if "$STRIP" "$WASM" -o "$S1" 2>"$WORK/strip.err"; then
    B1=$(sz "$S1")
    V1="PASS"; [ "$B1" -gt "$RAW_SLA" ] && { V1="FAIL"; PASS=0; }
    emit "size: $B1 bytes ($(fmt_mib "$B1")) — rung 1 (raw <=25MiB): $V1"
  else
    emit "[SKIP] strip failed (exit $?): $(head -2 "$WORK/strip.err")"
    PASS=0
    S1="$WASM"
  fi
fi

# --- Rung 2: wasm-opt -Os ---
WOPT="$(find_tool wasm-opt || true)"
if [ -z "$WOPT" ]; then
  emit "[SKIP] no wasm-opt (binaryen) — rung 2 skipped (loud)"
  PASS=0
  S2="$S1"
else
  emit "--- rung 2: wasm-opt --all-features -Os ($(basename "$WOPT")) ---"
  S2="$WORK/opt.wasm"
  if "$WOPT" --all-features -Os "$S1" -o "$S2" 2>"$WORK/opt.err"; then
    B2=$(sz "$S2")
    V2="PASS"; [ "$B2" -gt "$RAW_SLA" ] && { V2="FAIL"; PASS=0; }
    emit "size: $B2 bytes ($(fmt_mib "$B2")) — rung 2 (raw <=25MiB): $V2"
  else
    emit "[SKIP] wasm-opt failed (exit $?): $(head -2 "$WORK/opt.err")"
    PASS=0
    S2="$S1"
  fi
fi

# --- Rung 3: gzip -9 wire size ---
emit "--- rung 3: gzip -9 wire ---"
if gzip -9 -c "$S2" >"$WORK/wire.gz" 2>"$WORK/gz.err"; then
  B3=$(sz "$WORK/wire.gz")
  V3="PASS"; [ "$B3" -gt "$WIRE_SLA" ] && { V3="FAIL"; PASS=0; }
  emit "size: $B3 bytes ($(fmt_mib "$B3")) — rung 3 (wire <=10MiB): $V3"
else
  emit "[SKIP] gzip failed: $(head -2 "$WORK/gz.err")"
  PASS=0
fi

# --- Rung 4: section census (wabt) ---
emit "--- rung 4: section census (wasm-objdump -h) ---"
OBJDUMP="$(find_tool wasm-objdump || true)"
if [ -z "$OBJDUMP" ]; then
  emit "[SKIP] no wasm-objdump (wabt) — section census skipped (loud)"
elif [ ! -f "$S2" ]; then
  emit "[SKIP] no optimized artifact to census"
else
  if "$OBJDUMP" -h "$S2" >"$WORK/sections.txt" 2>&1; then
    emit "top sections by size:"
    # wabt -h lines: "Code start=0x.. end=0x.. (size=0x..) count: N"
    grep -E '\(size=0x' "$WORK/sections.txt" \
      | sed -E 's/^ *([A-Za-z0-9_]+).*\(size=0x([0-9a-f]+)\).*/\2 \1/' \
      | while read -r hx n; do printf '%d %s\n' "0x$hx" "$n"; done \
      | sort -rn | head -5 \
      | while read -r b n; do emit "  $n: $b bytes ($(fmt_mib "$b"))"; done
  else
    emit "[SKIP] wasm-objdump -h failed"
    PASS=0
  fi
fi

emit ""
if [ "$PASS" -eq 1 ]; then
  emit "COURT VERDICT: PASS (all rungs within SLA)"
  exit 0
fi
emit "COURT VERDICT: FAIL (>=1 rung over SLA) — this FAIL is the receipt"
exit 1
