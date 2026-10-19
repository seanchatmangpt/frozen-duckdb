# gate-v01-latency.md

GATE-V-01 court spec: aggregation p95 latency over 100k in-memory
event rows must be < 15 ms. Reconciles with
`docs/design/latency-bench-plan.md` (lane P5). Design doc only —
this lane does not edit `scripts/verify-gates.sh`, `Makefile`,
`schema/verify.ttl`, or any ggen projection. A future gate fact
lives in `schema/verify.ttl` as `ver:RequiredCheck`, never in a
projection.

## Precedence: Revision-2 workload supersedes P5 workload

This spec adopts **Revision-2's** 3-query workload. Where it
conflicts with the earlier workload triple in
`latency-bench-plan.md` (retention / funnel / rehydration),
Revision-2 governs and the earlier triple is superseded.

Revision-2 workload (versioned SQL constants in harness source; any
SQL change is a subject change and re-baselines the court), against
one deterministic 100,000-row `events` table (fixed seed recorded
in every receipt):

1. **Point filter** — equality/range filter on `event_id`/`ts`
   returning a small scan.
2. **Multi-level aggregation** — group-by nested over a
   pre-aggregation level (e.g. per `case_id` rollup, then per
   activity bucket aggregate of that).
3. **Windowed rolling average** — `AVG(...) OVER (PARTITION BY ...
   ORDER BY ts ROWS BETWEEN n PRECEDING AND CURRENT ROW)`.

All three run against the SAME table per iteration; no per-iteration
DDL or connection setup inside the measured region. One latency
sample = wall clock of all three queries in one iteration (the pass
rule tests the 3-query workload total).

## Measurement protocol (P5 machinery, unchanged)

- Harness: `criterion` via `crates/frozen-duckdb/benches/` target,
  plus the court's own percentile pass over raw per-iteration
  samples (self-contained, not criterion's summary).
- Fallback: manual loop, `std::time::Instant`, sorted samples.
- **No sleeps anywhere** (no-sleep doctrine): coordination by
  iteration counts and bounded polls, never waits.
- Warmup: 50 unmeasured iterations per query (`warmup_iters: 50`).
- Measured iterations: **>= 1000** per arm.
- Percentile: **p95 nearest-rank** over sorted raw samples
  (index `ceil(0.95 * n)`).
- Machine-spec receipt binding (mandatory): `cpu_model`, `cores`,
  `ram_gb`, `os`, `kernel`, `toolchain`, plus `git rev-parse HEAD`
  + tree hash — same subject binding the verify-gates runner uses.
  A receipt without machine specs or subject binding is not a
  latency receipt (F4).
- Fail-closed: missing toolchain, dataset, harness, or budget is
  `[SKIP] <reason>` + exit 1 — never a faked pass.

## Pass rule

`PASS` iff p95 of the 3-query-total sample vector over >= 1000
measured iterations is **< 15.0 ms**. Else `FAIL`. Inability to run
is `[SKIP] <reason>` (registered divergence, not a pass).

## Leak clause (rev2 addition absent from P5)

Zero memory-allocation-leak detection, alongside the latency rule:
allocator counters before the measured loop and after teardown must
match **within epsilon** (declared constant in the harness; counters
capture retained allocations only, not transient ones).

- **Mechanism: PROPOSED.** Engine-side allocation-stats API on
  DuckDB 2.0-dev is **UNVERIFIED**. Candidate surfaces: engine task
  allocator stats, `duckdb_memory()`-class C API, profile output.
- If no usable counters surface exists on the pinned engine, this
  clause is recorded as
  `UNSUPPORTED-BY-CURRENT-SURFACE` in the receipt with the probe
  evidence (what was probed, what returned) — it is **not** faked,
  not silently dropped, and does not block the latency standing.
  The latency verdict and the leak verdict are reported separately;
  overall gate standing is the conjunction.

## Non-vacuity: shim-court with 100x flip

BASELINE (sha256 of harness + SQL constants) -> BEFORE (expect
PASS) -> SHIM (`SHIM-COURT`-marked single-line edit) -> AFTER
(expect FAIL) -> RESTORE (sha256 byte-exact vs BASELINE, else
subject CORRUPTED, court exits nonzero).

- SHIM: raise row count **100x** (10,000,000 rows) —
  `// SHIM-COURT: 100x rows to exceed 15ms p95`. Budget alternative:
  unroll the heaviest query 100x inside the measured region,
  marked `SHIM-COURT unrolled 100x`.
- AFTER must flip to FAIL (p95 >= 15 ms). A shim still < 15 ms is
  `VACUOUS GATE`, exit 1 — the pass rule is not testing the engine.
- Budget: dataset build or AFTER run exceeding the declared
  10-minute wall-clock budget -> `[SKIP] <reason>` + exit 1 (loud).
- **Corrupt-restore self-check**: the court corrupts a restore on a
  real temp copy and verifies the sha256 mismatch is detected;
  corrupt-restore detection failing = court FAIL.
- No git state mutations: the court reads `git rev-parse` only.

## Receipt shape

Per run (JSON + TTL evidence row):

```
{ subject_head, subject_tree, machine{...}, warmup_iters,
  measured_iters, n_rows, sql_constants_sha256, samples_sha256,
  raw_samples_count, p95_ms, leak{counters_before, counters_after,
  epsilon, verdict|unsupported}, standing }
```

`scripts/verify-evidence.ttl` rows carry `ver:exitCode` integers
from the real court execution in the same run; the header timestamp
+ head/tree binding is the proof. A hand-edited TTL is not a
receipt.

## Falsifiers

- F1: AFTER on the 100x shim has p95 < 15 ms -> `VACUOUS GATE`,
  exit 1.
- F2: RESTORE sha256 != BASELINE -> subject CORRUPTED, exit
  nonzero.
- F3: budget exceeded on shim/AFTER -> `[SKIP]` + exit 1 (loud).
- F4: receipt missing machine specs or subject binding -> not
  evidence, exit 1.
- F5: leak clause reported as satisfied without probe evidence on
  the pinned engine (faked counters) -> court FAIL, not pass.
