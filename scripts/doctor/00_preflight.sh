#!/usr/bin/env bash
# Phase 00 — preflight: TEST-001 hash attestation + TEST-002 strip contract.
# Real checks, inline (no external checks/ dir yet).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
fail=0

# TEST-001: hash attestation of the vendored DuckDB headers.
hdr="$ROOT/crates/frozen-duckdb-builder/vendored-headers/duckdb.h"
if [ ! -f "$hdr" ]; then
  echo "TEST-001 FAIL: missing $hdr"
  fail=1
else
  got="$(shasum -a 256 "$hdr" | awk '{print $1}')"
  echo "TEST-001 hash attestation: $got  $(basename "$hdr")"
  if [ -z "$got" ] || [ ${#got} -ne 64 ]; then
    echo "TEST-001 FAIL: attestation digest malformed"
    fail=1
  fi
fi

# TEST-002 (a): strip contract — structural check over the build script surface.
bs="$ROOT/scripts/build_duckdb_optimized.sh"
if [ ! -f "$bs" ]; then
  echo "TEST-002(a) FAIL: missing $bs"
  fail=1
elif ! grep -qE 'strip|postprocess' "$bs"; then
  echo "TEST-002(a) FAIL: strip contract absent from $bs"
  fail=1
else
  echo "TEST-002(a) strip contract: present in $(basename "$bs")"
fi

# TEST-002 (b): binary audit — the actual cached dylib (size ceiling, C-ABI-only
# census, arch). Verdict propagated from scripts/doctor/checks/bin_audit.sh.
# Exit 3 (missing subject) is a loud NOT-IMPLEMENTED, not a pass.
bin_audit="$ROOT/scripts/doctor/checks/bin_audit.sh"
if [ ! -x "$bin_audit" ] && [ ! -f "$bin_audit" ]; then
  echo "TEST-002(b) [SKIP] missing $bin_audit"
  fail=1
else
  if bash "$bin_audit"; then
    echo "TEST-002(b) bin_audit: PASS"
  else
    rc=$?
    case "$rc" in
      3) echo "TEST-002(b) bin_audit: NOT-IMPLEMENTED-LOUD (missing subject, rc=3)"
         fail=1 ;;
      *) echo "TEST-002(b) bin_audit: FAIL (rc=$rc)"
         fail=1 ;;
    esac
  fi
fi

exit "$fail"
