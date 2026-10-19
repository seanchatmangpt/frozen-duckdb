# Ingest Throughput Plan

Responsibility matrix: Boundary Marshaller. Court id V-I1. Standing on
AFTER flip: `blocked`. Pattern: shim-court (BASELINE sha256 -> BEFORE green
-> SHIM marked `SHIM-COURT` -> AFTER nonzero -> RESTORE byte-exact,
fail-closed). Source pattern: smalt `scripts/shim_court/lib.sh`; repo-local
gate runner `scripts/verify-gates.sh` / `make verify-gates`.

## Claim under test

Packed columnar batch ingest through the native appender path sustains
>= 100,000 rows/sec, single-threaded, on the spec'd hardware class, with no
sleeps and a hard deadline bound.

## Court inventory row

| id | layer | subject | status |
|---|---|---|---|
| V-I1 | ingest throughput | appender batch ingest | PROPOSED-RUNNABLE |

Add after FR-01 lands:

| id | layer | subject | status |
| FR-01 | wasm shared-buffer ingest | wasm ingest court | UNVERIFIED |

## Subject

- Native (measurable NOW):
  `crates/frozen-duckdb/src/duckdb/appender/` (`mod.rs`, `arrow.rs`),
  entry point `Appender::append_rows` +
  `Appender::flush` (real
  `ffi::duckdb_appender_flush`).
- Wasm (proposed, FR-01 zero-copy shared-buffer path): the wasm ingest
  surface does not exist yet; its court cell is UNVERIFIED until FR-01
  lands. No sim pass, no faked green.

## Workload

Packed columnar batch: one in-memory columnar batch of fixed-width
integers (i64) + one f64 column, N rows per batch. Ingest via
`Appender::append_rows` over pre-built batches, close with `flush`, assert
row count via SQL `SELECT count(*)`. No sleeps anywhere — the harness
checks progress by rows-ingested counters, deadline-bounded, never
wait-and-hope.

## Measurement protocol

- Throughput = total rows flushed / elapsed wall time, single-threaded.
- Sustained: run >= 5 seconds of continuous ingest (deadline-checked, not
  a sleep-based window); report rows/sec.
- Hardware class recorded in the court receipt: CPU model, core count,
  memory, OS, from `sysctl`/`/proc` on the receiving machine.
- One warmup run, not counted.
- Batches pre-built BEFORE the clock starts; the clock covers ingest+flush
  only (batch construction is not ingestion).
- Deadline bound: if the 5s window cannot complete, the court fails
  loudly, not silently.

## Falsifier (non-vacuity)

An artificially throttled ingest must FAIL the >= 100k rows/sec bound.

- SHIM: mark the ingest loop `// SHIM-COURT`, insert a per-batch
  counter-checked throttling... no — a sleep is banned. Throttle by
  recomputing a checksum over the batch before each append, sized to force
  per-batch cost above the throughput budget. No sleeps, no spins on the
  clock; the throttle is real work, not a wait.
  or
- SHIM (alt): mutate the appender source (marked `SHIM-COURT`) so each
  `append_row` incurs a redundant full-batch pass before appending.

Either shim must drop sustained rows/sec below 100k and the court must
exit nonzero with standing `blocked`. A court that cannot flip is FAIL,
not PASS.

## Dual-target status

- Native: PROPOSED-RUNNABLE — subject, workload, protocol, and falsifier
  are fully spec'd; the appender path exists and is measurable now.
- Wasm: UNVERIFIED until FR-01 (zero-copy shared-buffer path) lands.
  Until then the wasm cell is loudly missing, never silently green.

## Receipt

The court writes into `scripts/verify-evidence.ttl` shape: real exit
codes from real runs, subject head/tree bound, hardware class, sustained
rows/sec, and standing. A hand-edited TTL is not a receipt.