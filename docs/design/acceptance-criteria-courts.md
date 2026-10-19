# Acceptance-Criteria Courts

Executable court specs for PRD §5 AC-1..4 — the AC layer above the
FR/NFR courts in `docs/design/verification-plan.md` and
`docs/design/requirements-traceability.md`. Each spec names: witness
command, artifact, standing-flip rule, non-vacuity mutation.

Standing vocabulary follows the traceability matrix: PROPOSED →
PARTIAL_ALIVE (native reality proves part) → ALIVE (receipt on exact
subject). A court that cannot flip is FAIL, not PASS.

| id | AC | native-runnable today | blocker |
|---|---|---|---|
| AC-1 | dual-target parity | no | FR-01 (no wasm artifact) |
| AC-2 | crash-chaos recovery | PROPOSED-RUNNABLE (native kill-9 variant) | wasm variant: FR-02 |
| AC-3 | OOM graceful abort | YES — native, now | wasm variant: FR-05-wasm |
| AC-4 | compression proof | delegated | P7 protocol owns it |

## AC-1: dual-target parity court

**Claim**: the same SQL workload file produces identical result digests
on the native engine and the WASM engine — the parity contract.

- **Witness command** (native leg, today):
  `cargo test -p frozen-duckdb --test parity_workload -- --nocapture`
  where `parity_workload` executes `tests/fixtures/parity_workload.sql`
  via the native engine and writes `target/parity/native.digest`
  (sha256 of the canonicalized result CSV).
- **Witness command** (wasm leg, FR-01-blocked):
  `node scripts/wasm/parity_court.mjs` — loads the built `duckdb-wasm`
  bundle, runs the same `parity_workload.sql`, writes
  `target/parity/wasm.digest`.
- **Adjudication**:
  `cmp target/parity/native.digest target/parity/wasm.digest`
  — byte-equal ⇒ AC-1 ALIVE; differ or either leg missing ⇒ BLOCKED
  with the differing digest pair attached.
- **Artifact**: `target/parity/{native,wasm}.digest` +
  `target/parity/receipt.json` (workload sha256, engine versions, both
  digests, verdict).
- **Standing-flip rule**: PROPOSED until both legs produce digests;
  ALIVE on one byte-equal adjudication; regresses to BLOCKED if any
  replay diverges.
- **Non-vacuity mutation**: edit `parity_workload.sql` (marked
  `SHIM-COURT`) to add `SELECT 1+1 AS drift;` — at least one leg's
  digest must change; if both digests are unchanged the court is
  vacuous and FAILs.
- **Runnability**: NOT runnable today — FR-01 gap (no `scripts/wasm/`,
  no emcmake build against vendored headers). The native leg alone is
  runnable and may mint `native.digest` receipts now; it cannot
  adjudicate parity until the wasm leg exists.

## AC-2: crash-chaos court (native kill-9 equivalent)

**Claim**: a mid-transaction kill leaves storage mountable with intact
checkpoint checksums. Browser instantiation of record: BP2 Court 1
(`docs/design/chrome-burnin-court.md` — `Page.crash` mid-transaction,
checksum asserts on state after the crash; its own falsifier is that a
no-op crash must FAIL the court).

- **Canonical court spec**: `docs/design/gate-v02-crash-chaos.md` —
  GATE-V-02 is the binding spec for this AC (writer loop, sentinel
  poll, reopen/assert contract, `target/court/ac2-receipt.json`
  artifact, kill-0 and WAL-rot non-vacuity shims). See that doc —
  canonical. This section no longer restates it (deduped 2026-10-19,
  lane ERRC-E3); `requirements-traceability.md` §"GATE-V-01..05"
  previously required the two docs "stay in sync" — the sync burden
  is removed by this pointer.
- **Standing-flip rule** (AC layer): PROPOSED until first real
  kill-9 run; PARTIAL_ALIVE on a native green receipt (wasm/OPFS
  leg still FR-02-blocked); ALIVE only via BP2 Court 1 receipt on
  the browser substrate.
- **Runnability**: native variant runnable today — needs only a CLI
  or test binary that runs an INSERT loop; nothing FR-01-blocked.
  Browser variant blocked on FR-02 (no `opfs_crash_court.mjs` on
  disk).

## AC-3: OOM graceful-abort court

**Claim**: exceeding the memory cap yields a typed `OutOfMemory`
error and engine state is preserved for subsequent use — never a
process kill or silent wrong answer.

- **Probe query** (exact, Cartesian join sized to blow the cap):
  `SELECT count(*) FROM range(100000) a, range(100000) b;`
  (10^10 intermediate rows against a small `memory_limit`).
- **Config call** (exact, native, today):
  ```rust
  let cfg = duckdb::Config::default()
      .with("memory_limit", "64MB")?;
  let conn = duckdb::Connection::open_in_memory_with_config(&cfg)?;
  ```
  (`Config::with` generic passthrough, `config.rs:129`; unknown keys
  fail at open.)
- **Witness command**:
  `cargo test -p frozen-duckdb --test oom_graceful_abort -- --nocapture`
  asserts: (1) the probe returns `Err` with DuckDB's out-of-memory
  error (typed, not a panic/abort), (2) the SAME connection afterwards
  runs `SELECT 42` and a fresh INSERT+SELECT round-trip successfully
  (state preserved).
- **Artifact**: `target/court/ac3-receipt.json` (probe query sha256,
  error string, post-OOM probe result, verdict).
- **Standing-flip rule**: PROPOSED until first native run;
  PARTIAL_ALIVE on a native green receipt; ALIVE requires the wasm
  variant (BP2 Court 2, 256MB linear-memory cap + CDP
  `Performance.getMetrics` peak poll, with its 10x-batch falsifier)
  to also run green.
- **Non-vacuity mutation**: set `memory_limit` to `"2GB"` (marked
  `SHIM-COURT`) so the probe fits comfortably — the
  probe must now SUCCEED; a court that still reports OutOfMemory at
  2GB is testing the query, not the cap, and FAILs.
- **Runnability**: runnable today on native — `Config::with` exists,
  the probe is plain SQL, no wasm needed.

## AC-4: compression proof court (delegated)

**Claim**: ≥90% compression on the 1M-event OCEL corpus.

This AC is **delegated to P7's protocol** — `docs/design/ocel2-compression.md`
owns the method, corpus fixture, and ratio math. The court is
V-S2 `scripts/compression_ratio_court.py --floor 0.90`, with the
ratio-fence law of `~/.claude/rules/no-sleep-testing.md` (relative
fence against an independent baseline, not absolute ms/bytes).

- **Witness command**: P7's, not restated here (delegation law:
  one owner per method — restating it here would fork the protocol).
- **Artifact**: P7's V-S2 receipt; this doc records only the
  delegation edge AC-4 → P7/V-S2.
- **Standing-flip rule**: follows V-S2's own flip rule; AC-4 standing
  is a read-through of V-S2's, never minted here.
- **Non-vacuity mutation**: P7's own (court must FAIL on a corpus
  where the ratio genuinely falls below floor, or on a fixture that
  compresses trivially).
- **Runnability**: delegated; corpus fixture is the P7-side gap
  (1M-event corpus not built).

## Runnability summary

- Runnable today (native): AC-2 native kill-9 variant
  (PROPOSED-RUNNABLE), AC-3 (runnable now via `Config::with` +
  `memory_limit`).
- FR-01/FR-02-blocked: AC-1 both legs' adjudication, AC-2 browser
  variant, AC-3 wasm variant (BP2 Court 2).
- Delegated: AC-4 → P7 / V-S2.

## See Also

`docs/design/verification-plan.md` ·
`docs/design/requirements-traceability.md` ·
`docs/design/chrome-burnin-court.md` ·
`docs/design/ocel2-compression.md` · `docs/design/memory-bounds.md`
