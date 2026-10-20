# Control Plan — v2.0.0-dev campaign (lane LSS8, Control-Plan avatar)

Every capability surface gets exactly one primary control: the gate that
watches it, the detection signal, the reaction when it trips, and the
escalation path when the reaction does not clear it. All cited paths are real
files on this tree at HEAD 79e803e. Companion docs: `POKA-YOKE-REGISTER.md`
(devices), `KD-CLOSURE-PLAN.md` (open divergences), `TAG-REHEARSAL.md`
(publish), `_LANES.md` (authority).

## 1. Control matrix

Format: characteristic | control | detection | reaction | escalation.
Gate paths are executable; citations verified on disk this session.

- **C1 Engine pin** (DuckDB 2.0.0-dev, pinned v2.0-cyanoptera b6013f7)
  - Control: `cargo run -p test-validation` (Gate 6 in the GENERATED
    `scripts/verify-gates.sh` battery) + builder marker check in
    `crates/frozen-duckdb-builder/src/lib.rs` (ENGINE_VERSION marker,
    byte-exact label law, size pin).
  - Detection: builder demotes the cached lib to MISS when the marker
    != `v{VERSION}` or the size pin disagrees.
  - Reaction: demote-to-MISS restage — builder discards the cache entry and
    re-acquires (probe-first, atomic temp+rename). Automatic.
  - Escalation: re-acquire fails (upstream pin gone) -> surface BLOCKED,
    file against `scripts/download_duckdb_binaries.sh`, operator decides pin.
- **C2 Shared cache integrity**
  - Control: P1 marker+size pin and P5 atomic writes
    (`POKA-YOKE-REGISTER.md`); KD-14 closure is the witness.
  - Detection: marker/size mismatch or partial lib observed by a
    concurrent build; builder refuses.
  - Reaction: re-acquire; lanes block on the write, never link a
    half-written lib.
  - Escalation: repeat trips in one session -> freeze lane builds, audit
    the writer, coordinator.
- **C3 Performance SLA** (REV-001 wall)
  - Control: `scripts/doctor/08_perf_sla.sh` (doctor stage 08) running
    `cargo test -p frozen-duckdb --test gate_v01_latency --test
    gate_ingest_throughput` in release class.
  - Detection: latency/throughput cargo tests red; doctor exits nonzero
    (fail-closed, device P2).
  - Reaction: repair lane fixes, re-runs stage 08; KD-07 caps the accepted
    budget until closed.
  - Escalation: chronic red -> KD-07-class row in `KNOWN_DIVERGENCES.md`,
    operator triage.
- **C4 OCEL compress/roundtrip**
  - Control: `scripts/doctor/07_ocel_compress.sh` ->
    `cargo test -p frozen-duckdb --test doctor_ocel_tests`.
  - Detection: ratio assertion fails inside `doctor_ocel_tests.rs`; stage
    exit nonzero.
  - Reaction: repair lane fixes the compression path, re-runs stage 07.
  - Escalation: red after repair -> typed divergence row, operator spec
    decision.
- **C5 VARIANT surface**
  - Control: `crates/frozen-duckdb/tests/variant_tests.rs` under the
    `cargo test --workspace` gate (Gate 5 in verify-gates).
  - Detection: variant roundtrip/cast test red.
  - Reaction: local fix on the FFI/type path, rerun.
  - Escalation: red on clean revert -> upstream DuckDB behavior change;
    spec decision.
- **C6 Docs claims** (AD7 discipline)
  - Control: AD7-style falsification sweeps, periodic — run the claim's
    cited command and diff against the claim.
  - Detection: cited output != claim (stale count, wrong path, wrong
    exit code).
  - Reaction: edit the claim to match observed output, or mark it
    as-of-date.
  - Escalation: claim cannot be executed -> claim deleted or
    UNSUPPORTED-marked.
- **C7 Warehouse-sim determinism**
  - Control: `warehouse-sim/courts/determinism.sh` +
    `warehouse-sim/tests/chaos.test.ts`.
  - Detection: byte-diff of two render runs nonempty; chaos test red.
  - Reaction: fix lane repairs stateful route/ledger drift, re-run court.
  - Escalation: nondeterminism survives -> KD-03-class reopen, BLOCKED.
- **C8 Warehouse-sim receipt chain**
  - Control: `warehouse-sim/courts/receipt_chain.sh`.
  - Detection: chain hash mismatch or missing receipt.
  - Reaction: regenerate receipts from source; never hand-edit
    `economics_receipt.json`.
  - Escalation: regeneration diverges -> generator bug; fix generator,
    re-project.
- **C9 Value-audit**
  - Control: `scripts/value_audit.sh` (renders via
    `warehouse-sim/economics/render_value_audit.ts`) +
    `crates/frozen-duckdb/tests/value_audit_*.rs`.
  - Detection: audit render red or rust audit tests red.
  - Reaction: regenerate from collectors
    (`scripts/value_audit/collect_val003.sh`).
  - Escalation: collector-vs-render divergence -> fix forward, receipt
    the rerun.
- **C10 Generated surfaces** (Makefile, verify-gates.sh)
  - Control: `make gates` = `ggen sync run` + genesis-check (device P4).
  - Detection: genesis-check drift.
  - Reaction: `ggen sync run` re-render; edit `schema/verify.ttl`, never
    the output.
  - Escalation: drift after sync -> template bug; fix the ggen rule.
- **C11 Version lockstep**
  - Control: `scripts/check_workspace_deps.sh` + P6 BUILDER_VERSION
    cross-check across `.github/workflows/*`.
  - Detection: crate/builder/guard version triple disagrees.
  - Reaction: re-pin all three to one version, rerun cross-check.
  - Escalation: upstream dep break -> pin bump decided by operator.
- **C12 Publish** (tag cut)
  - Control: `docs/sjira/v2.0.0-dev/TAG-REHEARSAL.md` preconditions P0-P6
    + `scripts/verify_publish_receipt.py`.
  - Detection: hold rule fires — assets missing/wrong-named, provenance
    gate RED (exit 1), publishing-token identity doubt, red CI without a
    reproducible dylib.
  - Reaction: STOP per TAG-REHEARSAL abort doctrine. Before first
    `cargo publish`: fix forward. From step 4 on: preserve the verbatim
    error, milestone BLOCKED, operator decides fix-forward versioning
    (2.0.1-dev+).
  - Escalation: operator only; lanes never retry-over a permanent step.

## 2. Out-of-control plan — 11 open KDs

Reactor classes: **UPSTREAM-WATCH** (nothing local to fix; watch the
upstream surface), **LOCAL-FIX** (a lane repairs on this tree), **SPEC**
(operator decides: tighten, split, or descope the clause).

| KD | Divergence (open) | Reactor | Reaction plan |
|---|---|---|---|
| KD-01 | 21 `__Z*` exports breach C-ABI | LOCAL+SPEC | Stages 00/01 red; fix exports |
| KD-02 | flock 404 at dev pin | UPSTREAM | Watch upstream; un-ignore `flock_tests.rs` |
| KD-05 | ~~No wasm artifact (FR-01)~~ CLOSED (MINIMAL-SLICE, X3) | resolved | min wasm 24.69MiB, wire 6.00MiB, version gate ×3 (`X3-kd05-close.md`); full-threaded build vs SLA open as separate surface |
| KD-06 | `host-range://` VFS design-only | LOCAL-FIX | Build VFS; `courts/gate_v05.sh` flips |
| KD-07 | REV-001 wall 4.2-6.1s vs <2.5s | SPEC | C3 detects; optimize or split clause |
| KD-08 | Allocator-stats C API absent | LOCAL-FIX | Add in `-sys`; `courts/gate_v01.sh` covers |
| KD-09 | Class B cgroup needs Linux+root | UPSTREAM | Watch `docker_low_ram_runner.sh` |
| KD-10 | Class E BEAM leg lives in smalt | UPSTREAM | React in smalt; telco test holds boundary |
| KD-11 | Browser courts FR-01/02 blocked | UPSTREAM | Chained on KD-05; opens at wasm closure |
| KD-12 | Anomaly exact-match false | SPEC | C6: as-of-date till fix or descope |
| KD-13 | Registry corruption, cleanup actor | LOCAL-FIX | Harden cleanup; spare cargo registry |

Rule: no KD row flips open->closed without a receipt citing the exact
command and its real output; any standing flip additionally requires
`scripts/check_sjira_standing.sh` clean.

## See Also

`POKA-YOKE-REGISTER.md` · `KD-CLOSURE-PLAN.md` · `TAG-REHEARSAL.md` ·
`_LANES.md` · `scripts/verify-gates.sh` (GENERATED — edit
`schema/verify.ttl`)
