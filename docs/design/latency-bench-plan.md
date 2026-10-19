# latency-bench-plan.md

NFR-01 court design: aggregation p95 latency over 100k in-memory rows
must be < 15 ms on the repo subject.

## Scope and standing

- Standing today: `PROPOSED` (native arm). The native arm is
  measurable with a criterion-based `cargo bench` harness (below).
  The wasm arm is `UNVERIFIED` until FR-01 lands (see
  `docs/design/wasm-opfs-architecture.md`).
- Dual-target note: one court, two arms. Native = `PROPOSED` and
  measurable today; wasm = `UNVERIFIED`, no numbers may be quoted
  until FR-01 lands.
- Fail-closed: missing toolchain, dataset, or harness is
  `[SKIP] <reason>`, never a faked pass.
- Authority: design document only (lane P5, write lane). It does not
  edit `scripts/verify-gates.sh`, `Makefile`, or any ggen projection.
  A future gate fact lives in `schema/verify.ttl`
  (`ver:RequiredCheck`), never in a projection.

## Workload definition (BP3/BP12 query classes)

Fixed dataset: 100,000 in-memory event rows, deterministically seeded
(fixed RNG seed recorded in every receipt), schema:

```sql
CREATE TABLE events (
  event_id BIGINT,
  case_id  BIGINT,
  activity VARCHAR,
  ts       TIMESTAMP,
  attrs    STRUCT(k VARCHAR, v VARCHAR)
);
```

Three query classes, each fixed once as a versioned SQL constant in
the harness source (any SQL change is a subject change and
re-baselines the court):

1. **Retention (BP3)** — cohort retention: date-bucketed group-by
   over event rows (cf. `docs/design/economic-model.md` reference
   workloads: "one cohort-retention query, one funnel query, one
   local-ML feature pass").
2. **Funnel (BP3)** — ordered step-conversion: count cases whose
   event sequence hits step-1 then step-2, per case.
3. **Rehydration (BP12)** — the lossless OCEL 2.0 segment-vector
   rehydration class from `docs/design/rehydration-sql.md`:
   unpivot/aggregate over `attrs` back into a flat event log.

All three queries run against the SAME 100k-row table per iteration.
No per-iteration DDL or connection setup inside the measured region.

## Measurement protocol

- Harness: `criterion` via a `[[bench]]` target
  (e.g. `crates/frozen-duckdb/benches/nfr01_latency.rs`), plus the
  court's own percentile pass over raw per-iteration samples (the
  pass rule is computed from raw samples, self-contained, not from
  criterion's summary alone).
- Fallback if criterion is unavailable: manual loop recording
  per-iteration wall clock via `std::time::Instant`, sorted samples,
  nearest-rank percentiles. No `Process.sleep`/`thread::sleep`
  anywhere; coordination is by iteration counts and bounded loops,
  never waits.
- Warmup: explicit and bounded — 50 unmeasured iterations per query
  class, declared in the receipt (`warmup_iters: 50`).
- Measured iterations: >= 1000 per arm. One latency sample = total
  wall clock of all three queries in one iteration (the pass rule
  tests the 3-query workload total, not one query).
- Percentile: p95 by nearest-rank over the sorted raw sample vector
  (index `ceil(0.95 * n)`).
- Machine specs per receipt: `cpu_model`, `cores`, `ram_gb`, `os`,
  `kernel`, `toolchain`, plus subject `git rev-parse HEAD` + tree
  hash (same binding the verify-gates runner uses). A receipt without
  machine specs is not a latency receipt.

## Pass rule

`PASS` iff p95 of the 3-query-total sample vector over >= 1000
measured iterations is < 15 ms. Anything else is `FAIL`; inability to
run is `[SKIP] <reason>` (registered divergence, not a pass).

## Non-vacuity mutation (shim-court shape)

BASELINE (sha256 of harness + SQL constants) -> BEFORE (expect PASS)
-> SHIM (marked `SHIM-COURT`) -> AFTER (expect FAIL) -> RESTORE
(sha256 byte-exact, else subject CORRUPTED, court exits nonzero).

- SHIM: deliberately oversized workload — raise the row count 100x
  (10,000,000 rows) in the harness source, single-line edit with
  `// SHIM-COURT: 100x rows to exceed 15ms p95`. If building 100x
  rows exceeds the budget, alternative shim: unroll the heaviest
  (retention) query 100x inside the measured region, marked
  `SHIM-COURT unrolled 100x`.
- AFTER must flip to FAIL (p95 >= 15 ms). If the 100x shim still
  lands < 15 ms, the court itself is `VACUOUS GATE`, exit 1: the pass
  rule is not actually testing the engine.
- Budget: if dataset build or the AFTER run exceeds a declared
  10-minute wall-clock budget, emit `[SKIP] <reason>` and exit 1 —
  fail-closed skip, never a silent pass.
- Restore self-check: the court corrupts a restore on a real temp
  copy and verifies the sha256 mismatch is detected; corrupt-restore
  detection failing = court FAIL.

## Receipt shape

Per run (JSON + a TTL evidence row):

```
{ subject_head, subject_tree, machine{...}, warmup_iters,
  measured_iters, n_rows, sql_constants_sha256, samples_sha256,
  raw_samples_count, p95_ms, standing }
```

`scripts/verify-evidence.ttl` rows carry `ver:exitCode` integers from
the real court execution in the same run (timestamp + head/tree
binding is the proof). A hand-edited TTL is not a receipt.

## Falsifiers

- F1: AFTER run on the 100x shim has p95 < 15 ms -> court is
  `VACUOUS GATE`, exit 1.
- F2: RESTORE sha256 != BASELINE sha256 -> subject CORRUPTED,
  exit nonzero.
- F3: budget exceeded on shim/AFTER -> `[SKIP]` + exit 1 (loud).
- F4: receipt missing machine specs or subject binding -> not
  evidence, court exit 1.
