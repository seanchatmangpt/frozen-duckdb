# Verification Plan

Blueprint-lane (BP9) court battery. Every court must exist and flip (fail
under its mutation) BEFORE any layer claims ALIVE. Pattern: shim-court
(BASELINE sha256 → BEFORE green → SHIM marked `SHIM-COURT` → AFTER nonzero →
RESTORE byte-exact, fail-closed on hash mismatch). Source pattern: smalt
`scripts/shim_court/lib.sh`; repo-local application of the frozen-duckdb gate
runner (`scripts/verify-gates.sh`, `make verify-gates`).

## Court inventory

| id | layer | subject | standing on AFTER flip |
|---|---|---|---|
| V-W1 | WASM build | `duckdb_wasm` artifact | `blocked` |
| V-W2 | OPFS persistence | crash-recovered DB file | `build_broken` |
| V-W3 | memory budget | CDP-polled RSS/JS heap | `blocked` |
| V-S1 | OCEL rehydration | segment-vector vs raw log | `build_broken` |
| V-S2 | compression | reference corpus ratio | `blocked` |
| V-G1 | blueprint graph | `docs/design/*.ttl` | `blocked` |
| V-G2 | native gates | `make verify` | existing gate standing |

## V-W1 — WASM build produces duckdb_wasm

Witness (real command):

```
make wasm-build && \
  node scripts/wasm/wasm_smoke.mjs duckdb_library_version
```

Artifact: `target/wasm32-unknown-emscripten/release/duckdb_wasm.{js,wasm}`;
smoke prints the real version string and exits 0 only if it matches the
crated DuckDB version (1.5.5).

Non-vacuity shim: edit `scripts/wasm/wasm_smoke.mjs` (marker
`// SHIM-COURT`) to expect a wrong version literal → smoke exits nonzero.
Second shim: rename the `.wasm` artifact → build gate exits nonzero.

## V-W2 — OPFS persistence survives forced Page.crash

Witness (real command, Chrome via CDP, no mocks):

```
node scripts/wasm/opfs_crash_court.mjs
```

Canonical mechanism: `docs/design/chrome-burnin-court.md` Court 1 —
forced `Page.crash` mid-transaction, relaunch, WAL replay, checksum
equality against pre-crash digests. See that doc — canonical; this
row no longer restates the mechanism (deduped 2026-10-19, lane
ERRC-E3). Non-vacuity shim inherits Court 1's falsifier (a no-op
crash / skipped WAL flush must FAIL the court); any court that
recovers under the shim is recorded FAIL, not PASS.

## V-W3 — memory budget under CDP polling

Witness (real command):

```
node scripts/wasm/memory_budget_court.mjs --budget-mb 256
```

Bounded poll of `Performance.getMetrics` (JSHeapUsedSize) + OS RSS of the
renderer at load and query phases; exits nonzero on any sample > 256 MB.
Bounded deadline poll only — see `[[no-sleep-testing]]`: no `Process.sleep`,
no fixed waits; observation replaces time.

Non-vacuity shim: `// SHIM-COURT` raise the budget to 4096 in the court's
config while injecting a large allocation in the workload script → court
must still exit nonzero via the RSS channel; if it passes, the observer is
vacuous.

## V-S1 — OCEL segment-vector rehydration equivalence

Chicago law: real collaborators, real diffs, no mocks
(`[[testing-chicago-style]]`).

Witness (real command):

```
cargo test -p frozen-duckdb --test ocel_rehydration_equivalence -- --nocapture
```

The test: build a real OCEL segment vector from a real event log, rehydrate,
and diff the rehydrated log against the raw log field-by-field (typed
records, not string equality of a summary). Zero-diff or exit nonzero.

Non-vacuity shim: `// SHIM-COURT` mutate one segment's payload byte /
timestamp before rehydration → equivalence assert must FAIL. Also shim the
assert into comparing only counts → reviewer rejects: interaction/
summary-only asserts are banned; the diff must stay structural.

## V-S2 — compression ratio on reference corpus

Measured, not asserted:

```
python3 scripts/compression_ratio_court.py \
  --corpus test-fixtures/ocel-reference-corpus/ --floor 0.90
```

Computes compressed/original byte ratio over every corpus file; exits
nonzero below 0.90. Emits a per-file table (the receipt).

Non-vacuity shim: `// SHIM-COURT` replace one corpus file with
pre-compressed bytes (incompressible) → ratio drops below floor → exit
nonzero. A floor that never fires on this corpus has no bits.

## V-G1 — blueprint docs graph validation

BP7's graph must parse and enforce the status vocabulary
(`UNKNOWN|PARTIAL_ALIVE|ALIVE|BLOCKED|BUILD_BROKEN|UNSUPPORTED` only —
stored-standing / overclaim atoms are refusal-class).

Witness (real command):

```
ggen sync run && \
  python3 scripts/validate_blueprint_graph.py docs/design/
```

Validator: parses every `.ttl` under `docs/design/` (rdflib), runs the
SHACL shapes, exits nonzero on any parse error or off-vocabulary status.

Non-vacuity shim: append a `SHIM-COURT` triple claiming
`bp:layer10 ver:status "ALIVE"` with no receipt (unreceipted assertion) →
SHACL must refuse, exit nonzero. Second shim: insert a malformed Turtle
line → parse must fail.

## V-G2 — native gate battery stays green

The existing repo battery is never regressed by blueprint work:

```
make verify
```

`make verify` = `verify-gates.sh` (real exits recorded into
`scripts/verify-evidence.ttl`, bound to subjectHead/tree) +
`ggen receipt verify`. Receipts are only evidence when `ver:exitCode`
integers come from real executions in the same run.

Non-vacuity shim (existing, mirrored here): mutate a Rust source with
`// SHIM-COURT` so `cargo clippy --workspace --all-targets --all-features
-- -D warnings` fails → gate exits nonzero with standing `blocked`; restore
sha256-exact. Also: `SHIM-COURT` edit a
`gates/dry-run-publish/*.ttl` atom to `published` → the structural fence
tripwire must exit nonzero. `published`, `crates-io-uploaded`,
`release-complete` remain banned atoms; terminal standing is
`dry-run-verified` only.

## Cross-cutting bans (repo doctrine)

- **No `Process.sleep` / fixed waits** — bounded polls on observables only
  (`[[no-sleep-testing]]`). A court using sleep-to-hope is FAIL.
- **No mocks** — real subprocess, real browser, real files, real diffs;
  doubles only where genuinely infeasible, never by default
  (`[[testing-chicago-style]]`).
- **No tautological asserts** — an assert on what the court just
  constructed verifies the court, not the law (`[[no-overclaiming-rust]]`,
  `[[entropic-annealing-doctrine]]` mock-laundering channel).
- **A court that cannot flip is FAIL, not PASS.** Every shim above is
  mandatory, not illustrative.
- **No git state mutations** in any court; `git rev-parse` reads only.
- **Loud skips**: missing toolchain/network → `[SKIP] <reason>`, never a
  faked green cell.
- **Edit sources, not projections**: mutations go in schemas/sources/graph;
  `Makefile`, `scripts/verify-gates.sh`, `gates/dry-run-publish-gates.md`
  are `ggen sync run` outputs and only ever touched via RESTORE
  sha256-exact.

## Aggregate runner

```
make verify && bash scripts/courts/run_all.sh
```

Exit 1 if any gate or court FAILs or is CORRUPTED on restore. Courts live
under `scripts/courts/` as self-contained scripts; `run_all.sh` prints a
per-court verdict table (PASS|FAIL|SKIP|CORRUPTED) with real exit codes.

## Standing

All courts: UNSUPPORTED (this plan is authored, not yet executed). ALIVE per
layer is claimable only after its court's BEFORE green receipt and its AFTER
flip receipt exist on the exact subject head.
