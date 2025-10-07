# KCura De-Fakery Protocol

This directory contains tools and scripts for systematically detecting and eliminating fake/mock/hardcoded implementations in the KCura codebase.

## 🎯 Prime Directive

**No stubs in production paths.** Any function exposed via FFI/public API must prove real effects via tests (state change, plan diff, cryptographic proof, OTEL trace).

## 📋 Protocol Components

### 1. Static Scanning (`scan_fakes.sh`)
Detects fake implementations at the code level using pattern matching.

```bash
./scripts/scan_fakes.sh .  # Scan entire repo for fake patterns
```

**Patterns Detected:**
- `unimplemented!` and `todo!` panics
- Trivial constant `Ok(...)` returns
- `dummy|fake|stub|placeholder` tokens
- FFI functions returning constants without dependency calls

### 2. Runtime Probes (`redteam_probe.rs`)
Systematically probes functions to detect if they're returning hardcoded responses.

```bash
cargo build --release --bin redteam_probe --manifest-path scripts/Cargo.toml
./target/release/redteam_probe
```

**Probes:**
- **Kernel Routing:** Forces different query patterns, checks if execution paths differ
- **Timer Hooks:** Advances deterministic clock, verifies firings occur at expected intervals
- **Transactions:** Attempts transactions, verifies database state actually changes
- **Receipts:** Tamper-tests cryptographic verification

### 3. CI Gates (`.github/workflows/ci.yml`)
Hard gates that prevent fake implementations from merging:

- `de-fakery-scan`: Static analysis before any tests
- `redteam-probe`: Runtime probing after compilation
- `semantic-coverage`: Ensures meaningful test coverage

## 🚨 Current Fake Implementations Detected

Run the scan script to see current issues:

```bash
./scripts/scan_fakes.sh .
```

**Known Issues:**
- `kc_last_exec_path`: Returns hardcoded JSON instead of tracking real execution paths
- `kc_tick_hooks`: Returns fake timer results instead of evaluating real hooks
- `kc_begin_tx`/`kc_commit_tx`/`kc_receipt_verify`: Mock transaction lifecycle
- `kc_metrics_snapshot`: Returns hardcoded metrics instead of live data

## 🔧 Implementation Queue

### A) Kernel Router (replace fake `kc_last_exec_path`)
- [ ] Route decision uses cost model (pattern match + stats)
- [ ] OTEL span `exec.kernel|exec.duckdb` with real data
- [ ] `EXPLAIN` snapshot saved for duckdb route
- [ ] Unit tests: route flips when adding regex/joins

### B) Timer Hooks (replace fake `kc_tick_hooks`)
- [ ] Deterministic clock injected; durable scheduled registry
- [ ] Idempotent fire with dedup key `(hook_id, scheduled_at)`
- [ ] Backoff + catch-up logic
- [ ] E2E tests with controlled time

### C) Transactions & Receipts
- [ ] Real DuckDB tx handle (BEGIN/COMMIT/ROLLBACK)
- [ ] SHA3-256 Merkle over normalized delta; Ed25519 signatures
- [ ] Tamper tests (any bit flip → invalid)
- [ ] OTEL spans `tx.*`, `receipts.verify`

### D) Metrics Snapshot
- [ ] Snapshots read from live meters/histograms
- [ ] Proven non-constant via workload deltas
- [ ] Prometheus scrape parity test

## 🧪 Testing Strategy

### Property Testing
```bash
cargo test proptest  # Test SPARQL subset with random inputs
```

### Mutation Testing
```bash
cargo mutants        # Inject faults, ensure tests catch them
```

### Fuzzing
```bash
cargo fuzz run fuzz_sparql_lowering  # Fuzz parsers/compilers
```

## 📊 Success Metrics

A capability is **DONE** only when:

1. **Spec → Tests → Code** traceability exists
2. **Golden trace** shows correct span set & attributes
3. **State proof** exists (DB diff, receipt verify, or metrics delta)
4. **Mutation tests** break with injected faults
5. **No static scan hits** (stubs/fake tokens)
6. **Docs** updated (USER_GUIDE + API reference + examples)

## 🚀 Quick Start

1. **Run the scan:** `./scripts/scan_fakes.sh .`
2. **Build the probe:** `cargo build --release --bin redteam_probe --manifest-path scripts/Cargo.toml`
3. **Run the probe:** `./target/release/redteam_probe`
4. **Check CI gates:** All de-fakery jobs should pass

## 🔍 Debugging Fake Implementations

Use the **Red-Team Playbook**:

* **Constant-output probe:** Call target function 100× with varied inputs; if output entropy ≈0, flag
* **Side-effect probe:** Wrap calls in DuckDB snapshot; diff affected tables; if no diff where expected, flag
* **Time-correlation probe:** For timer hooks, advance clock; if firings don't change, flag
* **Crypto probe:** Change one byte of payload; if verification still `true`, flag
* **Trace probe:** Assert missing expected spans/attrs → flag

This protocol ensures KCura maintains production-quality implementations that **cannot** pass tests unless they're genuinely real.

