#!/usr/bin/env bash
set -euo pipefail

echo "=== 🔍 GAP & IMPLEMENTATION SWEEP ==="
echo "Scanning for unimplemented features, fake stubs, and missing functionality..."
echo ""

# 1. Stub / Fake / TODO patterns
echo "--- Checking stubs and placeholders ---"
echo "Looking for unimplemented!, todo!, panic!, TODO, FIXME, XXX, dummy, fake, stub, placeholder..."
rg -n 'unimplemented!' || echo "  ✅ No unimplemented! found"
rg -n 'todo!' || echo "  ✅ No todo! found"
rg -n 'panic!' || echo "  ✅ No panic! found"
rg -n 'TODO' || echo "  ✅ No TODO found"
rg -n 'FIXME' || echo "  ✅ No FIXME found"
rg -n 'XXX' || echo "  ✅ No XXX found"
rg -n 'dummy' || echo "  ✅ No dummy found"
rg -n 'fake' || echo "  ✅ No fake found"
rg -n 'stub' || echo "  ✅ No stub found"
rg -n 'placeholder' || echo "  ✅ No placeholder found"
echo ""

# 2. Telemetry / OTEL usage
echo "--- Checking telemetry / OTEL integration ---"
echo "Looking for OpenTelemetry integration, tracing, metrics..."
rg -n 'use .*opentelemetry' || echo "  ⚠️  No OpenTelemetry imports found"
rg -n 'tracing_opentelemetry' || echo "  ⚠️  No tracing_opentelemetry found"
rg -n 'OpenTelemetryLayer' || echo "  ⚠️  No OpenTelemetryLayer found"
rg -n 'record_hook_metrics' || echo "  ⚠️  No record_hook_metrics found"
rg -n 'global::meter' || echo "  ⚠️  No global::meter found"
rg -n 'kc_set_traceparent' crates/kcura-ffi/src || echo "  ⚠️  No kc_set_traceparent FFI found"
echo ""

# 3. Hooks implementation
echo "--- Checking hooks runtime ---"
echo "Looking for hook evaluation functions..."
rg -n 'eval_guards' crates/kcura-hooks || echo "  ⚠️  No eval_guards found"
rg -n 'eval_on_commit' crates/kcura-hooks || echo "  ⚠️  No eval_on_commit found"
rg -n 'eval_thresholds' crates/kcura-hooks || echo "  ⚠️  No eval_thresholds found"
rg -n 'eval_timers' crates/kcura-hooks || echo "  ⚠️  No eval_timers found"
echo ""

# 4. Kernels / Router
echo "--- Checking kernel/router implementation ---"
echo "Looking for kernel execution and routing..."
rg -n 'execute_.*_kernel' crates/kcura-kernels || echo "  ⚠️  No kernel execution functions found"
rg -n 'execute_inner_eq_join' crates/kcura-kernels || echo "  ⚠️  No inner equi-join kernel found"
rg -n 'execute_left_eq_join' crates/kcura-kernels || echo "  ⚠️  No left equi-join kernel found"
rg -n 'ExecutionRoute' || echo "  ⚠️  No ExecutionRoute enum found"
rg -n 'choose_execution_route' || echo "  ⚠️  No choose_execution_route function found"
echo ""

# 5. Receipts / Crypto
echo "--- Checking receipts and cryptographic functions ---"
echo "Looking for receipt creation, verification, and crypto primitives..."
rg -n 'create_receipt' crates/kcura-receipts || echo "  ⚠️  No create_receipt found"
rg -n 'verify_receipt' crates/kcura-receipts || echo "  ⚠️  No verify_receipt found"
rg -n 'sha3' crates/kcura-receipts || echo "  ⚠️  No SHA3 implementation found"
rg -n 'ed25519' crates/kcura-receipts || echo "  ⚠️  No Ed25519 implementation found"
rg -n 'kc_receipt_verify' crates/kcura-ffi || echo "  ⚠️  No kc_receipt_verify FFI found"
rg -n 'CREATE TABLE.*kc_receipts' crates/kcura-engine || echo "  ⚠️  No kc_receipts table DDL found"
rg -n 'CREATE TABLE.*tx_log' crates/kcura-engine || echo "  ⚠️  No tx_log table DDL found"
echo ""

# 6. FFI surface
echo "--- Checking FFI functions ---"
echo "Looking for C ABI exports and FFI tests..."
rg -n 'pub extern "C"' crates/kcura-ffi/src || echo "  ⚠️  No FFI exports found"
rg -n 'ffi' crates/kcura-tests || echo "  ⚠️  No FFI tests found"
rg -n 'ffi_smoke' examples || echo "  ⚠️  No FFI smoke test found"
echo ""

# 7. Benchmarks and tests
echo "--- Checking benchmarks and test coverage ---"
echo "Looking for benchmarks and comprehensive tests..."
rg -n 'bench' benches || echo "  ⚠️  No benchmarks found"
rg -n 'determinism' crates/kcura-tests || echo "  ⚠️  No determinism tests found"
rg -n 'receipt' crates/kcura-tests || echo "  ⚠️  No receipt tests found"
rg -n 'hook' crates/kcura-tests || echo "  ⚠️  No hook tests found"
rg -n 'kernel' crates/kcura-tests || echo "  ⚠️  No kernel tests found"
echo ""

# 8. Governance / Audit (future readiness)
echo "--- Checking governance / audit scaffolding ---"
echo "Looking for enterprise governance features..."
rg -n 'AuditEvent' || echo "  ⚠️  No AuditEvent found"
rg -n 'PolicyDefinition' || echo "  ⚠️  No PolicyDefinition found"
rg -n 'ComplianceStatus' || echo "  ⚠️  No ComplianceStatus found"
echo ""

# 9. Compiler implementations
echo "--- Checking compiler implementations ---"
echo "Looking for OWL RL, SHACL, and SPARQL compilation..."
rg -n 'compile_owl_rl' crates/kcura-compiler-owl-rl || echo "  ⚠️  No OWL RL compilation found"
rg -n 'compile_shacl' crates/kcura-compiler-shacl || echo "  ⚠️  No SHACL compilation found"
rg -n 'compile_sparql' crates/kcura-compiler-sparql || echo "  ⚠️  No SPARQL compilation found"
echo ""

# 10. Engine and runtime
echo "--- Checking engine and runtime ---"
echo "Looking for engine orchestration and runtime management..."
rg -n 'Runtime' crates/kcura-engine || echo "  ⚠️  No Runtime struct found"
rg -n 'execute_sql' crates/kcura-duck || echo "  ⚠️  No SQL execution found"
rg -n 'attach_database' crates/kcura-duck || echo "  ⚠️  No database attachment found"
echo ""

# 11. Catalog and DDL
echo "--- Checking catalog and DDL ---"
echo "Looking for knowledge catalog and DDL management..."
rg -n 'Catalog' crates/kcura-catalog || echo "  ⚠️  No Catalog struct found"
rg -n 'CREATE TABLE' crates/kcura-catalog || echo "  ⚠️  No DDL statements found"
rg -n 'DDL' crates/kcura-catalog || echo "  ⚠️  No DDL management found"
echo ""

# 12. In-memory store
echo "--- Checking in-memory store ---"
echo "Looking for in-memory implementation..."
rg -n 'InMemStore' crates/kcura-inmem || echo "  ⚠️  No InMemStore found"
rg -n 'execute_query' crates/kcura-inmem || echo "  ⚠️  No in-memory query execution found"
echo ""

echo "=== ✅ SWEEP COMPLETE ==="
echo ""
echo "Legend:"
echo "  ✅ = Feature implemented or not needed"
echo "  ⚠️  = Potential gap or missing implementation"
echo ""
echo "Review any ⚠️  items above for implementation gaps."
echo "Run with: ./scripts/rg_sweep.sh | tee sweep.log"
