# FR-04: View-Layer Expansion of Compressed Segments

Design for vectorized on-demand expansion of the compressed segment
representation (one row per maximal run, see
`docs/design/ocel2-compression.md`) back into dense per-event streams,
exposed as a view layer over frozen-duckdb.

## Grounding: what is already EXECUTED

All expansion SQL below descends from BP12's executed work
(`docs/design/rehydration-sql.md`), run for real against DuckDB CLI
`v1.4.1 (Andium) b390a7c376`:

- **Q2 generate_series expansion** — `Counts` fan-out with uniform
  cadence timestamps
  `t_k = TStart + k * (TEnd - TStart) / (Counts - 1)`; VERIFIED-SYNTAX.
- **Q3 Dt-chain window variant** — running `SUM(Dt) OVER (PARTITION BY
  ObjectID ORDER BY SegmentID)` chain when `TEnd` is untrusted;
  VERIFIED-SYNTAX.
- **Q4 OCEL 2.0 JSON export** — `ocel-json-2.0` shaped via
  `json_object`/`json_group_array`; VERIFIED-SYNTAX.
- **Q5 losslessness invariants** — event-count conservation, no
  duplicates, time floor: `true | true | true` on fixtures.

Standing: all queries are **VERIFIED-SYNTAX on the v1.4.1 CLI;
UNVERIFIED on the pinned v1.5.5/2.0.0 engine** (BP12's
`test-validation` run failed on a staged-dylib version mismatch).
Nothing here is claimed ALIVE on the repo subject until V-S1 passes
through the pinned engine. This doc inherits that standing verbatim;
no upgrade without a receipt.

## Timestamp law (fixed input to everything below)

Event `k` (0-based) of a `Counts=n` segment lands at
`t_k = TStart + k * Step` where `Step = (TEnd - TStart) / (n-1)`;
`n = 1` degenerates to `TStart`. The Dt-chain variant replaces
`TEnd`-derived step with `Dt / Counts` per event when segment end
brackets are untrusted. Residual lossiness (original per-event jitter)
is owned by the compression doc's lossiness table, not here.

## View layer: three mechanisms, one decision

The expansion must be consumable by analysts and by the equivalence
court. Three candidate mechanisms over the segment table:

### Option A — pure SQL views (chosen default)

```sql
CREATE OR REPLACE VIEW v_ocel2_events AS
WITH bounds AS (
  SELECT SegmentID,
         CASE WHEN Counts > 1
              THEN date_diff('microsecond', TStart, TEnd)
                   / (Counts - 1)
              ELSE 0 END AS StepUs
  FROM segment_vector
)
SELECT s.SegmentID, s.ObjectID, s.ActionID,
       s.FromStateID, s.ToStateID,
       gs.i AS EventIndex,
       s.TStart + INTERVAL (b.StepUs * gs.i) MICROSECOND AS EventTime
FROM segment_vector s
JOIN bounds b USING (SegmentID)
CROSS JOIN generate_series(0, CAST(s.Counts - 1 AS BIGINT)) gs(i);
```

Plus `v_ocel2_events_dtchain` (Q3 shape) and `v_ocel2_export_json`
(Q4 shape) as sibling views.

- **Pros**: zero new engine surface; the optimizer sees plain
  relational algebra and can push filters/projections into the
  segment scan before fan-out; replayable as text; exactly what BP12
  executed. DuckDB evaluates the view lazily — no expansion cost
  unless the consumer selects from it (on-demand by construction).
- **Cons**: no streaming/short-circuit — a `LIMIT 10` still materializes
  the full generate_series for segments reached before the limit is
  satisfied within a pipeline batch; a single huge segment
  (`Counts ~ 10^9`) blows the chunk before any bound applies.

### Option B — table function (parquet_scan-style)

A `segment_expand(segment_range)` scan function producing the same
columns via a custom table function (Rust side, FFI).

- **Pros**: true producer-side chunking — can emit row-group-at-a-time
  and respect a caller-supplied max-rows budget; composable with
  `LIMIT` for streaming consumers.
- **Cons**: new C API/FFI surface on the pinned engine (UNVERIFIED
  territory, and the exact surface is precisely what BP12 could not
  reach); must re-implement timestamp law outside SQL (drift risk vs
  the executed text); not declaratively replayable.

### Option C — scalar micronavigation UDF

`expand_event(segment_row, k) -> event_row` scalar function; callers
drive `k` themselves.

- **Pros**: smallest possible unit; trivially testable.
- **Cons**: pushes the `generate_series` driver to every call site
  (N copies of the fan-out logic = the N² drift the composition laws
  forbid); scalar UDFs defeat vectorized execution (per-row FFI
  crossing); worst memory profile of the three, not best.

### Decision

Ship **A now** (all of it is VERIFIED-SYNTAX and grounded in executed
SQL); reserve **B** as the FR-05 escape hatch only where A's
materialization bound is provably exceeded (see below); **C rejected**
— it decomposes vectorization for no bound win.

Views are the public surface; the Q2/Q3/Q4 SQL text is the single
source of the timestamp law. Any Rust-side reimplementation (future
Option B) must differential-test against the view output or it is a
drift defect by definition.

## Memory-bounded chunked expansion (ties FR-05)

The generate_series fan-out is `O(sum(Counts))` rows. The bound that
matters is per-segment, not total: a single `Counts ~ 10^8+` run
expands inside one pipeline chunk.

Laws for the view layer:

1. **Chunk by segment, not by row.** Consumers iterate
   `WHERE SegmentID BETWEEN lo AND hi` windows; each window's
   expansion is materializable. The view stays unchanged — bounding is
   a consumer discipline enforced by the expansion helper, not a view
   rewrite.
2. **Cap-aware helper.** A thin wrapper query (or, if A's bound is
   exceeded in practice, the Option B function) resolves a target
   byte/row budget to SegmentID windows using `sum(Counts) OVER
   (ORDER BY SegmentID)` running totals — the same window machinery
   Q3 already VERIFIED-SYNTAXes.
3. **Spill before fan-out.** Very long segments route through
   FR-05's chunked external storage: the expansion reads the segment
   row, but the *dense consumer* (export, court) writes windowed
   batches to durable storage before proceeding to the next window.
   No in-memory accumulation of the full dense stream exists anywhere
   in the plan.
4. **Loud cap.** An expansion exceeding the budget without an explicit
   window request is a refusal (typed error naming the SegmentID and
   its `Counts`), never a silent OOM.

## Equivalence court V-S1

Claim under test: *the rehydrated (expanded) stream is semantically
equal to the raw baseline event stream.*

Court design (Chicago: real collaborators, assert on final state):

1. **Subjects.** Raw baseline = pre-compression `ocel2_events` table
   retained on the fixture corpus. Rehydrated = `v_ocel2_events`
   (Q2 law) over the compressed segments.
2. **Semantic equality predicate** — not byte equality; the
   compression discards per-event jitter, so equality is defined as:
   - event multiset cardinality equal (Q5 conservation, re-asserted
     against the *baseline*, not just internally);
   - `(ObjectID, ActionID, EventIndex)` key set equal;
   - per-event timestamp within tolerance `<= Step/2` of baseline
     (uniform-cadence reconstruction guarantee) — hard zero-tolerance
     only on fixtures whose original cadence was uniform;
   - ordered event-type/object relationship projections equal
     (Q4 export compared against baseline-derived export).
3. **Mutation check (non-vacuity).** Corrupt one segment's `Counts`
   (+1) and re-run: the cardinality predicate must flip to false. A
   court that cannot flip is VACUOUS and the design is not landed.
4. **Engine gate.** The whole court runs through
   `cargo run -p test-validation` on the pinned engine. Current
   blocker: BP12's staged-dylib version mismatch (`v2.1.0-dev89402`
   vs `v2.0.0`). Until that run is green, V-S1 standing is
   **BLOCKED(engine)**, view standing is VERIFIED-SYNTAX only.

Pass output: the five predicate columns all `true` on fixture
corpus + mutation flip witnessed. Fail output names the predicate,
segment, and first divergent `(SegmentID, EventIndex)`.

## Standing summary

| Artifact | Standing |
|---|---|
| Expansion/view SQL (Q2/Q3/Q4-derived) | VERIFIED-SYNTAX (CLI v1.4.1) |
| Same SQL on pinned engine | UNVERIFIED (dylib mismatch) |
| V-S1 court | BLOCKED(engine) until test-validation green |
| Option B table function | UNSUPPORTED (not built; reserved for FR-05) |

## Falsifier

V-S1 running green on the pinned engine with any predicate `false`, or
the mutation check failing to flip, refutes this design. A
reimplementation of the timestamp law outside the executed SQL text
that drifts from view output refutes the single-source claim.

## See Also

`docs/design/rehydration-sql.md` · `docs/design/ocel2-compression.md` ·
`docs/design/memory-bounds.md` (FR-05) · `docs/design/verification-plan.md`
