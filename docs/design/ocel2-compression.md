# OCEL 2.0 Segment-Vector Compression over DuckDB

## Scope

Design for real-time OCEL 2.0 (XES-successor, object-centric event log)
process-mining metamodel storage over frozen-duckdb. Conforming state
transitions collapse into run-length-encoded segment vectors; raw event
rows are reserved strictly for formal invariant escapes. Target: 90-99%
storage reduction vs naive per-event rows, with lossless on-demand
rehydration of a standards-compliant OCEL 2.0 event stream via DuckDB SQL
window functions.

All DuckDB SQL in this document is UNVERIFIED against the 1.5.5 / 2.0.0
engine until actually executed in this repo. Syntax is written to the
documented dialect but carries no ALIVE standing until a receipt exists.

## Metamodel mapping

OCEL 2.0 core: events, objects, object types, event types, and per-event
object relations (`ocel:oid`, `ocel:type`, qualifier attributes).

Mapping to this design:

| OCEL 2.0 concept | Storage representation |
|---|---|
| Event | Cell of a compressed segment (or escape row) |
| Event type | `EventTypeID` dimension in segment vector |
| Object | Row in `ocel2_objects` |
| State of an object | Last observed attribute-value tuple for that `oid` |
| State transition | `(ActionID, FromStateID, ToStateID)` triple |
| Repeated identical transitions | Run length in `Counts` |
| Inter-event time | `Dt` (delta-seconds), aggregated per run |

Conformance assumption: the input stream is a valid OCEL 2.0 log
(schema-validating against the OCEL 2.0 specification, v1.0). Non-conforming
inputs are the escape path, not a compression failure mode.

## Schema DDL (UNVERIFIED)

Segment-vector tables. Syntax not yet executed against the engine;
`UNVERIFIED` until a `cargo test` / CLI receipt lands.

```sql
-- Dimension tables (shared, uncompressed; small relative to events)
CREATE TABLE IF NOT EXISTS ocel2_object_types (
  ObjectTypeID   UINTEGER PRIMARY KEY,
  TypeName       VARCHAR NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS ocel2_objects (
  ObjectID       UBIGINT PRIMARY KEY,      -- internal surrogate
  OcelOid        VARCHAR NOT NULL,         -- ocel:oid from the log
  ObjectTypeID   UINTEGER NOT NULL REFERENCES ocel2_object_types
);

CREATE TABLE IF NOT EXISTS ocel2_event_types (
  EventTypeID    UINTEGER PRIMARY KEY,
  TypeName       VARCHAR NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS ocel2_states (
  StateID        UBIGINT PRIMARY KEY,
  ObjectTypeID   UINTEGER NOT NULL REFERENCES ocel2_object_types,
  AttrHash       UBIGINT NOT NULL,         -- hash of sorted attr tuple
  AttrJson       JSON NOT NULL             -- canonical attribute payload
);

CREATE TABLE IF NOT EXISTS ocel2_actions (
  ActionID       UINTEGER PRIMARY KEY,
  EventTypeID    UINTEGER NOT NULL REFERENCES ocel2_event_types,
  FromStateID    UBIGINT NOT NULL REFERENCES ocel2_states,
  ToStateID      UBIGINT NOT NULL REFERENCES ocel2_states
);

-- Compressed segment vectors: one row = one maximal run of identical
-- transitions for one object, in timestamp order.
CREATE TABLE IF NOT EXISTS ocel2_segments (
  SegmentID      UBIGINT PRIMARY KEY,
  ObjectID       UBIGINT NOT NULL REFERENCES ocel2_objects,
  ActionID       UINTEGER NOT NULL REFERENCES ocel2_actions,
  FromStateID    UBIGINT NOT NULL REFERENCES ocel2_states,
  ToStateID      UBIGINT NOT NULL REFERENCES ocel2_states,
  Counts         UBIGINT   NOT NULL,       -- events in this run
  Dt             DOUBLE    NOT NULL,       -- sum of inter-event seconds
  TStart         TIMESTAMP NOT NULL,       -- first event timestamp
  TEnd           TIMESTAMP NOT NULL        -- last event timestamp
);

-- Raw storage: ONLY formal invariant escapes land here. A conforming
-- event never gets a raw row.
CREATE TABLE IF NOT EXISTS ocel2_escapes (
  EscapeID       UBIGINT PRIMARY KEY,
  ObjectID       UBIGINT NOT NULL REFERENCES ocel2_objects,
  EscapeClass    VARCHAR NOT NULL,         -- see classification rule
  RawEventJson   JSON    NOT NULL,         -- verbatim OCEL 2.0 event
  IngestedAt     TIMESTAMP NOT NULL
);
```

Rationale for run key: a run is maximal only when the next event for the
same object has the same `(ActionID, FromStateID, ToStateID)` triple AND
contributes only to `Counts`/`Dt`. Any attribute change forces a new state
ID, hence a new action, hence a new segment — losslessness is carried by
the state table, not by the segment row.

## Ingestion path (real-time)

Streaming ingest is a two-step fold per arriving event:

1. Compute `(EventTypeID, FromStateID)` from the object's current state
   (materialized view or maintained `ocel2_object_current` table).
2. Compare against the open segment tail for that object:
   - Match: `UPDATE ocel2_segments SET Counts = Counts + 1, Dt = Dt + ?,
     TEnd = ? WHERE SegmentID = ?`.
   - No match: close the tail, insert a new segment.

Batch mode (bulk load of an existing OCEL 2.0 JSON/XML log) uses the same
RLE keying via a window function over the raw staging table:

```sql
-- UNVERIFIED
INSERT INTO ocel2_segments
SELECT
  row_number() OVER () AS SegmentID,
  ObjectID, ActionID, FromStateID, ToStateID,
  COUNT(*)                       AS Counts,
  SUM(DtSeconds)                 AS Dt,
  MIN(EventTime)                 AS TStart,
  MAX(EventTime)                 AS TEnd
FROM (
  SELECT
    e.ObjectID, a.ActionID, a.FromStateID, a.ToStateID,
    e.EventTime,
    COALESCE(
      date_diff('second',
        LAG(e.EventTime) OVER (PARTITION BY e.ObjectID ORDER BY e.EventTime),
        e.EventTime), 0) AS DtSeconds,
    CASE WHEN
      LAG(a.ActionID)    OVER w IS NOT DISTINCT FROM a.ActionID AND
      LAG(a.FromStateID) OVER w IS NOT DISTINCT FROM a.FromStateID AND
      LAG(a.ToStateID)   OVER w IS NOT DISTINCT FROM a.ToStateID
    THEN 0 ELSE 1 END AS RunBoundary
  FROM ocel2_staging_events e
  JOIN ocel2_actions a ON a.EventTypeID = e.EventTypeID
                     AND a.FromStateID  = e.FromStateID
  WINDOW w AS (PARTITION BY e.ObjectID ORDER BY e.EventTime)
)
WHERE RunBoundary = 1
   OR LAG(RunBoundary) OVER (
        PARTITION BY ObjectID ORDER BY TStart) IS NULL
GROUP BY
  ObjectID, ActionID, FromStateID, ToStateID,
  SUM(SUM(RunBoundary)) OVER (
    PARTITION BY ObjectID ORDER BY TStart
    ROWS UNBOUNDED PRECEDING);
```

The nested `SUM(SUM(...)) OVER` running-boundary grouping is the least
portable piece here and is the first thing to verify against the engine.

## Rehydration query skeleton (UNVERIFIED)

On-demand expansion of segment vectors back into a standards-compliant
OCEL 2.0 event stream, using `generate_series` to fan each run out to
`Counts` logical events:

```sql
-- UNVERIFIED
WITH expanded AS (
  SELECT
    s.ObjectID,
    s.ActionID,
    s.FromStateID,
    s.ToStateID,
    a.EventTypeID,
    et.TypeName          AS EventTypeName,
    o.OcelOid,
    ot.TypeName          AS ObjectTypeName,
    s.Counts,
    s.TStart,
    s.TEnd,
    gs.i                 AS RunIndex,
    date_add(
      s.TStart,
      INTERVAL (gs.i * (s.Dt / GREATEST(s.Counts - 1, 1))) SECOND
    )                    AS EventTime
  FROM ocel2_segments s
  JOIN ocel2_actions a    USING (ActionID)
  JOIN ocel2_event_types et ON et.EventTypeID = a.EventTypeID
  JOIN ocel2_objects o    USING (ObjectID)
  JOIN ocel2_object_types ot ON ot.ObjectTypeID = o.ObjectTypeID
  CROSS JOIN LATERAL generate_series(0, s.Counts - 1) AS gs(i)
)
SELECT
  EventTypeName, OcelOid, ObjectTypeName,
  states_from.AttrJson AS AttributesBefore,
  states_to.AttrJson   AS Attributes,
  EventTime
FROM expanded
JOIN ocel2_states states_from ON states_from.StateID = FromStateID
JOIN ocel2_states states_to   ON states_to.StateID   = ToStateID
UNION ALL
-- escapes rehydrate verbatim
SELECT
  json_extract_string(RawEventJson, '$.ocel:eventType'),
  json_extract_string(RawEventJson, '$.ocel:oid'),
  json_extract_string(RawEventJson, '$.ocel:type'),
  RawEventJson, -- attribute payload reassembled at serializer level
  RawEventJson,
  json_extract(RawEventJson, '$.ocel:timestamp')
FROM ocel2_escapes
ORDER BY EventTime;
```

Notes:

- `generate_series` over `Counts - 1` reproduces exactly `Counts` events;
  `Counts = 1` degenerates to a single event at `TStart`.
- Even-spacing of timestamps inside a run is a reconstruction choice, not
  information loss: per-event timestamps inside a run are summarized by
  `Dt` only. This is the one intentional semantic approximation — see
  falsifier F3 and the escape rule below. If exact per-event timestamps
  are required, the run does not conform and must be an escape.
- `UNION ALL` head/tail column counts must match the serializer contract;
  column alignment above is illustrative pending the real serializer.

## Conformance / invariant-escape classification rule

An event is compressed into segment vectors iff ALL of the following hold;
otherwise it is an escape row:

1. **Schema conformance**: the event validates against the OCEL 2.0
   metamodel (typed event, at least one related object, well-formed
   attributes).
2. **Action coverage**: `(EventTypeID, FromStateID)` resolves to exactly
   one registered row in `ocel2_actions`. Unknown actions are escapes of
   class `UNKNOWN_ACTION` (and register the action for the next ingest).
3. **Deterministic transition**: the object's resulting state hash maps to
   exactly one `ToStateID`. Ambiguous state resolution (`AMBIGUOUS_STATE`)
   is an escape.
4. **Timestamp summarizability**: the consumer accepts run-summarized
   inter-event time (`Dt`) with even rehydration spacing. If the stream
   carries per-event semantic timestamps inside a would-be run
   (e.g. `TIMESTAMP_EXACT` fidelity requirement), it is an escape of class
   `TIMESTAMP_EXACT`.
5. **Singleton relation**: the event relates to exactly one object. Events
   relating to N > 1 objects are stored once per object only if the
   transition is identical from each object's perspective; otherwise
   `MULTI_OBJECT` escape (raw row, emitted once at rehydration).

Escape classes are closed and typed: `UNKNOWN_ACTION`, `AMBIGUOUS_STATE`,
`TIMESTAMP_EXACT`, `MULTI_OBJECT`, `SCHEMA_INVALID`. Anything outside the
closure is a classifier bug, not a new class — extend the closure with a
receipt, never fall through to raw by default.

Raw storage is reserved strictly for these escapes. A compression pipeline
that writes raw rows for conforming events has violated the invariant and
fails F4.

## Compression ratio expectation

Target: 90-99% storage reduction vs naive per-event row storage
(event row ≈ 80-150 B; segment row ≈ 60-80 B amortized over `Counts`
events). Realistic runs in repeat-driven processes (order lifecycles,
approval chains, IoT state machines) have mean run length 10-100+, giving
10-100x row-count reduction.

Ratio is MEASURED, never asserted: `SUM(Counts)` events vs segment row
count, bytes from `duckdb_database_size()` or `parquet_file_size` on
export. Any claim of reduction in a receipt carries the actual measured
numbers on an actual log.

## Falsifiers

- **F1 (semantic byte-comparability)**: rehydration over a compressed log
  must produce an OCEL 2.0 event stream semantically comparable to the raw
  baseline: same event count, same (event type, oid, attribute payload,
  object relations) multiset, timestamps equal up to the documented
  even-spacing rule for summarized runs. Comparison is a deterministic
  diff script, not eyeballing. Fail = any multiset mismatch.
- **F2 (measured compression)**: compression ratio computed from real
  storage on a real log; target 90-99%. Fail = measured ratio below 90%
  on a run-length-heavy corpus, or any claim of ratio without a measured
  receipt.
- **F3 (escape completeness)**: every raw row in `ocel2_escapes` carries an
  escape class from the closed set, and every event not conforming to the
  classification rule has an escape row. Fail = a conforming event with a
  raw row (invariant violation) or a non-conforming event with none.
- **F4 (no silent raw fallback)**: ingesting a conforming log must produce
  zero `ocel2_escapes` rows. Any conforming event landing raw fails.
- **F5 (rehydration volume bound)**: rehydrated event count equals
  `SUM(Counts) + escape count`. Fail = any drift, which would mean
  `generate_series` fan-out is off by the boundary condition.

## Verification plan (UNVERIFIED until executed)

1. Execute the DDL against the vendored 1.5.5 engine; fix any dialect
   rejects and re-receipt.
2. Bulk-load a synthetic OCEL 2.0 log (deterministic generator, seeded) +
   one real-world log; run F1-F5 as a `cargo test` integration suite or a
   scripted SQL court with embedded output.
3. Measure storage: baseline raw table vs segment tables, same engine,
   same checkpoint.
4. Only after 1-3: flip this document's UNVERIFIED markers with receipts.
