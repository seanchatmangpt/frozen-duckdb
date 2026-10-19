# rehydration-sql.md

Design for lossless rehydration of OCEL 2.0 segment vectors back into
per-event streams over frozen-duckdb. Companion to
`docs/design/ocel2-compression.md` (ingest/compress direction); this doc
owns the decompress direction.

## Execution status (receipt-first)

Every query in this document was executed for real against the DuckDB CLI
`v1.4.1 (Andium) b390a7c376` (`/opt/homebrew/bin/duckdb`) in one run of the
scratch script `/tmp/bp12_rehydrate.sql`. That CLI is a **degraded
collaborator**, not the repo subject: the pinned frozen-dylib path
(`cargo run -p test-validation`, `CARGO_TARGET_DIR=target-laneBP12`) FAILED
this session with `version mismatch: staged dylib reports 'v2.1.0-dev89402'
but builder expects 'v2.0.0-dev'` — the dylib cache is mid-rebuild. Per the
loud-skip law this is recorded, not papered over: **all queries below are
VERIFIED-SYNTAX on duckdb v1.4.1 CLI; UNVERIFIED against the pinned
v1.5.5/2.0.0-dev engine** until the same script runs through
`test-validation`. No query is claimed ALIVE on the repo subject.

## Metamodel recap

Segment vector (compressed): one row per maximal run of identical
transitions per object — `(SegmentID, ObjectID, ActionID, FromStateID,
ToStateID, Counts, Dt, TStart, TEnd)` where `Dt` is the summed inter-event
seconds across the run and `TStart`/`TEnd` bracket it. Rehydration inverts
this: expand `Counts` back into one row per event, reconstruct per-event
timestamps, and emit OCEL 2.0 `ocel-json-2.0` shaped JSON.

Timestamp law used throughout: intra-segment cadence is uniform. Event
`k` (0-based) of a `Counts=n` segment lands at
`t_k = TStart + k * (TEnd - TStart) / (n-1)`; `n = 1` degenerates to
`TStart`. This is exact when original cadence was uniform and is the best
point estimate otherwise; the residual is a lossy dimension (original
per-event jitter is not stored), flagged in the lossiness table of the
compression doc.

## Segment-vector DDL (executed, Q1)

```sql
CREATE OR REPLACE TABLE segment_vector_ddl (
  SegmentID   UBIGINT PRIMARY KEY,
  ObjectID    UBIGINT NOT NULL,
  ActionID    UINTEGER NOT NULL,
  FromStateID UBIGINT NOT NULL,
  ToStateID   UBIGINT NOT NULL,
  Counts      UBIGINT NOT NULL,
  Dt          DOUBLE  NOT NULL,
  TStart      TIMESTAMPTZ NOT NULL,
  TEnd        TIMESTAMPTZ NOT NULL
);
```

Canonical DDL with full foreign keys lives in
`docs/design/ocel2-compression.md`; this thin variant is what the scratch
script executes.

## Rehydration via generate_series + window function (executed, Q2)

Timestamp from the segment's own `[TStart, TEnd]` bracket:

```sql
CREATE OR REPLACE TABLE ocel2_events AS
WITH bounds AS (
  SELECT
    SegmentID,
    CASE WHEN Counts > 1
         THEN date_diff('microsecond', TStart, TEnd)
              / (Counts - 1)
         ELSE 0 END AS StepUs
  FROM ocel2_segments
)
SELECT
  s.SegmentID,
  s.ObjectID,
  s.ActionID,
  s.FromStateID,
  s.ToStateID,
  gs.i AS EventIndex,
  s.TStart + INTERVAL (b.StepUs * gs.i) MICROSECOND AS EventTime,
  b.StepUs
FROM ocel2_segments s
JOIN bounds b USING (SegmentID)
CROSS JOIN generate_series(0, CAST(s.Counts - 1 AS BIGINT)) gs(i)
ORDER BY s.ObjectID, s.SegmentID, gs.i;
```

Executed result on fixtures (3 segments, 6 events, object `order-1`):
uniform hourly cadence, segment boundaries contiguous (seg 2 starts exactly
where seg 1 ends), `EventIndex 0..Counts-1` per segment.

## Alternative: Dt-chain timestamps (executed, Q3)

When `TEnd` is untrusted, chain segments via a running sum of `Dt` over
`(ObjectID, SegmentID)` and distribute intra-segment:

```sql
WITH chain AS (
  SELECT
    SegmentID,
    SUM(Dt) OVER (PARTITION BY ObjectID
                  ORDER BY SegmentID
                  ROWS BETWEEN UNBOUNDED PRECEDING
                       AND 1 PRECEDING) AS DtPrior
  FROM ocel2_segments
)
SELECT
  s.SegmentID,
  s.ObjectID,
  gs.i AS EventIndex,
  CASE WHEN gs.i = 0
       THEN s.TStart
       ELSE s.TStart + INTERVAL (CAST(
              s.Dt * gs.i / s.Counts AS BIGINT)) SECOND
  END AS EventTime
FROM ocel2_segments s
JOIN chain c USING (SegmentID)
CROSS JOIN generate_series(0, CAST(s.Counts - 1 AS BIGINT)) gs(i)
ORDER BY s.ObjectID, s.SegmentID, gs.i;
```

`DtPrior` is bound for downstream use (first-event-of-run offset when the
compression side ever decides to attribute the pre-run gap to the run's own
`Dt`); the expansion itself keys off `TStart` plus intra-segment fraction.

## OCEL 2.0 JSON export (executed, Q4)

```sql
CREATE OR REPLACE TABLE ocel2_export AS
SELECT
  json_object(
    'ocel:validation', 'ocel-json-2.0',
    'ocel:objectTypes', (SELECT coalesce(json_group_array(
        json_object('ocel:name', TypeName,
                    'ocel:oid-types', '{}'::JSON )), '{}'::JSON)
      FROM ocel2_object_types),
    'ocel:eventTypes', (SELECT coalesce(json_group_array(
        json_object('ocel:name', TypeName)), '{}'::JSON)
      FROM ocel2_event_types),
    'ocel:objects', (SELECT coalesce(json_group_array(
        json_object('ocel:oid', OcelOid,
                    'ocel:type', (SELECT t2.TypeName
                                  FROM ocel2_object_types t2
                                  WHERE t2.ObjectTypeID = o.ObjectTypeID)
        )), '{}'::JSON)
      FROM ocel2_objects o),
    'ocel:events', (
      SELECT coalesce(json_group_array(evt), '{}'::JSON)
      FROM (
        SELECT json_object(
          'ocel:id', CAST(e.SegmentID AS VARCHAR) || ':' ||
                     CAST(e.EventIndex AS VARCHAR),
          'ocel:timestamp', strftime(e.EventTime AT TIME ZONE 'UTC',
                                     '%Y-%m-%dT%H:%M:%SZ'),
          'ocel:attributes', '{}'::JSON,
          'ocel:type', et.TypeName,
          'ocel:relationships', json_array(json_object(
            'ocel:oid', o.OcelOid,
            'ocel:type', ot.TypeName
          ))
        ) AS evt
        FROM ocel2_events e
        JOIN ocel2_actions a ON a.ActionID = e.ActionID
        JOIN ocel2_event_types et ON et.EventTypeID = a.EventTypeID
        JOIN ocel2_objects o ON o.ObjectID = e.ObjectID
        JOIN ocel2_object_types ot ON ot.ObjectTypeID = o.ObjectTypeID
        ORDER BY e.EventTime
      )
    )
  ) AS ocel_json;
```

Executed output head on fixtures:

```
{"ocel:validation":"ocel-json-2.0","ocel:objectTypes":[{"ocel:name":"orders",
"ocel:oid-types":{}},{"ocel:name":"items","ocel:oid-types":{}}],
"ocel:eventTypes":[{"ocel:name":"create order"},...],
"ocel:objects":[{"ocel:oid":"order-1","ocel:type":"orders"},...],
"ocel:events ...}
```

DuckDB v1.4.1 notes: `json()` with zero args is not a macro (use
`'{}'::JSON`); `strftime(ts, '%Y-%m-%dT%H:%M:%SZ')` after
`AT TIME ZONE 'UTC'` renders RFC3339 `Z` timestamps.

## Losslessness invariants (executed, Q5)

```sql
SELECT
  (SELECT sum(Counts) FROM ocel2_segments) =
    (SELECT count(*) FROM ocel2_events) AS event_count_conserved,
  (SELECT count(DISTINCT (ObjectID, EventIndex, SegmentID))
     FROM ocel2_events) =
    (SELECT count(*) FROM ocel2_events) AS no_duplicate_events,
  (SELECT min(EventTime) FROM ocel2_events)
    >= (SELECT min(TStart) FROM ocel2_segments) AS time_monotone_floor;
```

Executed: `true | true | true`.

## Falsifier

Running `/tmp/bp12_rehydrate.sql` through the pinned engine
(`test-validation`) with any of Q2/Q5 returning an error or a `false`
invariant refutes this design's rehydration law on the repo subject.
