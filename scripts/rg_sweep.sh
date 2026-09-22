#!/usr/bin/env bash
# STATUS (G5 2026-09-21): KEPT-FIXED. Informational gap sweep, never a gate
# (always exits 0; every probe falls back to an echo). G5 trimmed the kcura-era
# sections whose rg paths target crates/kcura-* — crates that do not exist in
# this repo — which made ~80% of the output permanent misleading warnings.
# Remaining sections probe generic patterns across the tree plus the real
# frozen-duckdb FFI surface (frozen-duckdb-sys bindgen output). Verified after
# edit: bash -n clean; live run exits 0. Re-verify on any further edit.

set -euo pipefail

echo "=== GAP & IMPLEMENTATION SWEEP ==="
echo "Scanning for unimplemented features, fake stubs, and missing functionality..."
echo ""

# 1. Stub / Fake / TODO patterns
echo "--- Checking stubs and placeholders ---"
echo "Looking for unimplemented!, todo!, panic!, TODO, FIXME, XXX, dummy, fake, stub, placeholder..."
rg -n 'unimplemented!' || echo "  OK: no unimplemented! found"
rg -n 'todo!' || echo "  OK: no todo! found"
rg -n 'panic!' || echo "  OK: no panic! found"
rg -n 'TODO' || echo "  OK: no TODO found"
rg -n 'FIXME' || echo "  OK: no FIXME found"
rg -n 'XXX' || echo "  OK: no XXX found"
rg -n 'dummy' || echo "  OK: no dummy found"
rg -n 'fake' || echo "  OK: no fake found"
rg -n 'stub' || echo "  OK: no stub found"
rg -n 'placeholder' || echo "  OK: no placeholder found"
echo ""

# 2. Telemetry / OTEL usage
echo "--- Checking telemetry / OTEL integration ---"
echo "Looking for OpenTelemetry integration, tracing, metrics..."
rg -n 'use .*opentelemetry' || echo "  GAP: no OpenTelemetry imports found"
rg -n 'tracing_opentelemetry' || echo "  GAP: no tracing_opentelemetry found"
rg -n 'OpenTelemetryLayer' || echo "  GAP: no OpenTelemetryLayer found"
rg -n 'global::meter' || echo "  GAP: no global::meter found"
echo ""

# 3. FFI surface (frozen-duckdb: the sys bindgen layer)
echo "--- Checking FFI surface ---"
echo "Looking for the bindgen-generated C ABI surface in frozen-duckdb-sys..."
rg -n 'extern "C"' crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs | head -5 || echo "  GAP: bindgen FFI surface not found"
echo "  ... (head of bindgen surface shown; full surface lives in the two bindgen_bundled_version*.rs files)"
rg -n 'extern "C"' crates/frozen-duckdb-sys/src --glob '!bindgen_bundled_version*' || echo "  OK: no hand-written extern \"C\" outside the bindgen surface"
echo ""

# 4. Governance / Audit (future readiness)
echo "--- Checking governance / audit scaffolding ---"
echo "Looking for enterprise governance features..."
rg -n 'AuditEvent' || echo "  GAP: no AuditEvent found"
rg -n 'PolicyDefinition' || echo "  GAP: no PolicyDefinition found"
rg -n 'ComplianceStatus' || echo "  GAP: no ComplianceStatus found"
echo ""

echo "=== SWEEP COMPLETE ==="
echo ""
echo "Legend:"
echo "  OK  = pattern absent (implemented or not needed)"
echo "  GAP = pattern absent where the section expects it"
echo "  hits = live matches to review (candidates, not verdicts)"
echo ""
echo "Run with: ./scripts/rg_sweep.sh | tee sweep.log"
