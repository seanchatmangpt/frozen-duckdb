# master-traceability.md

Section-7 master traceability for the sovereign spec
(`docs/design/sovereign-spec-v26.10.19.md`), re-derived as a
repo-grounded table. Each row: requirement, implementation locus in
THIS repo, court, current standing, witness command.

Standing vocabulary: PROPOSED (specified, no receipt on any subject),
PARTIAL_ALIVE (native reality proves part of the requirement), ALIVE
(receipt on exact subject), BLOCKED, UNSUPPORTED.

## Source-spec correction (noted, resolved in favor of §5)

The tasking spec's §7 table maps PRD-FR-01 to "GATE-V-01 binary
inspection". This is an internal inconsistency of that spec: its own
§5 court ladder defines GATE-V-01 as the **latency** court (p95 < 15
ms, stage 6), and the on-disk court docs agree
(`gate-v01-latency.md`; binary size is NFR-02, owned by
`binary-size-budget.md`). This matrix resolves the mapping in favor
of §5: PRD-FR-01's hermetic-bundle inspection is a **build gate**
(NATIVE `BUILD_EXTENSIONS` pin), not GATE-V-01. No GATE-V court is
re-assigned by this doc.

## Traceability matrix

### PRD-FR-01 Hermetic static bundling

- Locus: `crates/frozen-duckdb-builder/src/lib.rs:589-595` — cmake
  `-DBUILD_EXTENSIONS=` with the nine-extension list parquet, json,
  icu, httpfs, tpch, tpcds, fts, inet, sqlsmith (extension-NAME
  list, not a boolean, per the TR2 rehearsal),
  `-DBUILD_JEMALLOC=ON`, autoinstall/autoloading off.
- Court: native build gate. GATE-V-01 is NOT this row — see the
  correction above. Wasm leg: `scripts/build_wasm_duckdb.sh`.
- Standing: native-static ALIVE (nine-extension native build; TR2/TR7
  receipts, 22 passed / 0 failed —
  `docs/sjira/v26.10.8/CAMPAIGN-RECEIPT.md`); wasm leg PROPOSED.
- Witness: `cargo test -p frozen-duckdb-builder` (arch/extension
  court, 22/0); pin source `crates/frozen-duckdb-builder/src/lib.rs:594`.

### PRD-FR-02 Host-VFS virtualization

- Locus: `docs/design/host-range-vfs-contract.md` (CONTRACT layer,
  `host-range://` prefix, status UNVERIFIED) +
  `docs/design/host-call-vfs.md` (architecture).
- Court: GATE-V-05 (`gate-v05-hostvfs-transfer.md`; stub
  `scripts/courts/gate_v05.sh` exits 3, NOT-IMPLEMENTED).
- Standing: PROPOSED — contract only. No VFS callback registered;
  no `host-range://` code anywhere in `crates/`.
- Witness: `bash scripts/courts/gate_v05.sh; echo $?` -> exit 3
  (loud refusal, no fake pass).

### PRD-FR-03 VARIANT zero-panic API

- Locus: `crates/frozen-duckdb/src/duckdb/statement.rs:598` —
  VARIANT bind arm (JSON-text payload; varchar -> VARIANT cast);
  `crates/frozen-duckdb/tests/variant_tests.rs` (6 `#[test]` on
  disk) is the de facto roundtrip spec.
- Court: `cargo test -p frozen-duckdb --test variant_tests`
  (native battery); no wasm-side VARIANT court exists.
- Standing: PARTIAL_ALIVE — native battery green (5 passed /
  0 failed, live receipt below); wasm-side PROPOSED.
- Witness: live `cargo test -p frozen-duckdb --test variant_tests`
  run receipt below (this session).

### PRD-FR-04 OPFS sync persistence

- Locus: `docs/design/wasm-opfs-architecture.md` (OPFS block-I/O
  design; `FileSystemSyncAccessHandle`) +
  `docs/design/gate-v04-opfs-concurrency.md`.
- Court: GATE-V-04 (stub `scripts/courts/gate_v04.sh`, exit 3);
  BP2 Court 3, `chrome-burnin-court.md`.
- Standing: PROPOSED — no `opfs/` module, no burn-in log; the
  whole leg is UNVERIFIED in-source.
- Witness: `bash scripts/courts/gate_v04.sh; echo $?` -> exit 3.

### PRD-FR-05 Bounded memory, hard cap

- Locus: `crates/frozen-duckdb/src/duckdb/config.rs` —
  `Config::max_memory` (line 117) plus generic
  `Config::with(key, value)` accepting `memory_limit`; probe spec
  `docs/design/gate-v03-memory-ceiling.md` (allocator guard).
- Court: GATE-V-03 (stub `scripts/courts/gate_v03.sh`, exit 3);
  native leg consumed by R7's real OOM witness inside
  `gate-v03-memory-ceiling.md`.
- Standing: PARTIAL_ALIVE — native green receipt 2026-10-19
  (typed OOM error, post-OOM `SELECT 42` = 42, host exit 0);
  wasm 256MB-linear leg PROPOSED.
- Witness: R7 rig `/tmp/r7gate` cargo-run output quoted in
  `gate-v03-memory-ceiling.md` ("PROBE-ERROR typed err=Out of
  Memory ... POST-OOM SELECT 42 = 42"). No repo court script yet —
  named `[SKIP] scripted shim loop` in that doc.

### SLA-1 p95 < 15ms @ 100k rows

- Locus: `docs/design/gate-v01-latency.md` +
  `docs/design/latency-bench-plan.md` (lane P5). No bench harness
  in `crates/`.
- Court: GATE-V-01, the latency court (stub
  `scripts/courts/gate_v01.sh`, exit 3).
- Standing: PROPOSED — no harness, no baseline measurement.
- Witness: `bash scripts/courts/gate_v01.sh; echo $?` -> exit 3.

### SLA-2 <=25MB wasm / <=10MB wire

- Locus: `scripts/build_wasm_duckdb.sh:92` —
  `-DCMAKE_CXX_FLAGS="-O3 -flto"` (strip leg);
  `docs/design/binary-size-budget.md` (P6 binary audit; every
  budget cell BUDGET-ASSUMPTION).
- Court: binary-size court (P6 audit). Not a GATE-V court — see
  the correction above.
- Standing: PROPOSED — no wasm artifact exists to measure.
- Witness: `grep -n flto scripts/build_wasm_duckdb.sh` (flag
  present; artifact absent).

### SLA-3 >100k transitions/sec ingest

- Locus: `docs/design/ingest-throughput-plan.md` (P16 bench,
  court id V-I1, shim-court pattern).
- Court: V-I1. No court script in `scripts/`.
- Standing: PROPOSED — bench never run.
- Witness: none. A fabricated one would be SUSPECT; the plan doc
  is the only artifact.

### SLA-4 100% WAL recovery

- Locus: `docs/design/gate-v02-crash-chaos.md`
  (PROPOSED-RUNNABLE: native `kill -9` leg runs today; CDP
  `Page.crash` leg blocked on FR-02) +
  `docs/design/vfs-journal-design.md`.
- Court: GATE-V-02 (stub `scripts/courts/gate_v02.sh`, exit 3);
  BP2 Court 1.
- Standing: PROPOSED — native leg runnable, but no scripted
  court receipt has landed.
- Witness: `bash scripts/courts/gate_v02.sh; echo $?` -> exit 3.

## Receipts (real output, this session)

### variant_tests live run (PRD-FR-03 witness)

`cargo test -p frozen-duckdb --test variant_tests`, this session,
exit 0:

```
test variant_column_type_metadata_no_panic ... ok
test variant_raw_column_read_is_typed_error ... ok
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured;
  0 filtered out; finished in 0.05s
```

Delta resolved on inspection: the file carries 6 `#[test]`
attributes but the 6th is behind
`#[cfg(feature = "serde_json")]`
(`crates/frozen-duckdb/tests/variant_tests.rs:69-70`), so the
default-feature binary runs 5. The two named greens above are
the zero-panic and typed-error legs PRD-FR-03's confinement
claim rests on. 5 passed / 0 failed, exit 0, is the
PARTIAL_ALIVE witness; the wasm-side VARIANT leg remains
PROPOSED regardless.

### Gate stub census

Every GATE-V court script on disk is a loud NOT-IMPLEMENTED stub
(exit 3), which is the correct standing for PROPOSED rows:

- `scripts/courts/gate_v01.sh` .. `gate_v05.sh`: header comment
  "Loud NOT-IMPLEMENTED: exit 3, never a fake pass, never a silent
  skip"; runner `scripts/courts/run_all.sh`.
- Real run this session (2026-10-19): all five stubs exit 3;
  `bash scripts/courts/run_all.sh` prints the verdict table —
  `gate_v01..gate_v05  NOT-IMPLEMENTED`, final line
  `GATE-V: FAIL (fail-closed: NOT-IMPLEMENTED and FAIL both
  block)`, runner exit 1. Fail-closed confirmed: no GATE-V court
  can read as green while unimplemented.

## Falsifier for this document

Any row claiming ALIVE/PARTIAL_ALIVE without a court receipt on the
exact subject in this repo. Standing flips go through
receipt-gated tickets, never edits to this matrix.

## See Also

- `docs/design/sovereign-spec-v26.10.19.md` (source spec, §5/§7)
- `docs/design/requirements-traceability.md` (FR-01..05 alias map)
- `docs/design/prd-ard-v2.md` (PRD-FR-*/SLA-* source)
- `scripts/courts/README.md` (court inventory)
