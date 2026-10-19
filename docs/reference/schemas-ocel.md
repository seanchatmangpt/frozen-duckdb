# schemas-ocel

Reference for the OCEL 2.0 conforming storage schema on the frozen
engine: three DDL tables, the hardened rehydration view, and the
escape ledger. Court of record:
`crates/frozen-duckdb/tests/doctor_ocel_tests.rs` (TEST-018/019/020) —
all SQL below is the real executed SQL from that file.

## DDL

### events_staging (transient)

```sql
CREATE TABLE events_staging(
    n BIGINT, oid VARCHAR, action VARCHAR,
    channel VARCHAR, ts TIMESTAMP);
```

Transient ingest surface; dropped after the fold is verified
(`DROP TABLE events_staging; CHECKPOINT;`).

### ocel_conforming_segments

```sql
CREATE TABLE ocel_conforming_segments(
    segment_id BIGINT,
    oid VARCHAR,
    action VARCHAR,
    channel VARCHAR,
    t_start TIMESTAMP,
    t_end TIMESTAMP,
    transition_count BIGINT);
```

RLE-folded event segments (block-granularity grouping, e.g.
`n / 100`); populated by the fold insert
(`row_number() OVER (ORDER BY b)`, `min/max(ts)`,
`count(*)::BIGINT AS transition_count`).

### ocel_escapes

```sql
CREATE TABLE ocel_escapes(
    escape_id BIGINT,
    oid VARCHAR,
    payload VARIANT);  -- or JSON, per DDL probe
```

Payload column form is decided at runtime by a probe:
`CREATE OR REPLACE TABLE _variant_probe(x VARIANT);` — accepted =>
`VARIANT` with `'{json}'::VARIANT` inserts; refused => `JSON` with
`'{json}'::JSON`. On the current engine the probe reports
`ocel-court: VARIANT DDL ACCEPTED (D1 updated on this engine)` — the
1.4.1-era refusal (X5 finding D1) is closed on this pin.

## Hardened rehydration view (executed verbatim)

The spec-literal NULLIF guard silently drops singleton segments
(`generate_series` with a NULL step returns 0 rows — D2). The court
uses the hardened COALESCE form:

```sql
CREATE VIEW ocel_rehydrated AS
SELECT
    s.segment_id,
    s.oid,
    s.action,
    s.channel,
    unnest(generate_series(
        s.t_start,
        s.t_end,
        to_microseconds(COALESCE(
            (date_diff('microsecond', s.t_start, s.t_end)
             / NULLIF(s.transition_count - 1, 0))::BIGINT,
            1)))
    ) AS ts
FROM ocel_conforming_segments s;
```

Span-relative microsecond math (D3 law), not literal-second steps.

## Rehydration courts (asserted by the test file)

- Conservation (F-C3): `sum(transition_count) == event count ==
  count(ocel_rehydrated)`.
- Singleton witness (D2): exactly one `transition_count = 1` segment
  exists and rehydrates exactly once; a corpus without it hides the
  NULLIF vacuity from conservation.
- Timestamp bounds: every rehydrated `ts` within
  `[t_start, t_end]` of its segment.
- Structural drift: bidirectional `EXCEPT` between rehydrated and
  raw staging, zero divergent rows tolerated.

## Escape ledger law

- Non-uniform-cadence events are pulled out as full-fidelity raw JSON
  rows in `ocel_escapes` — never folded into segments.
- Exactly `ESCAPES` rows; each `ocel:attributes.entropy` attribute
  round-trips byte-exact via `json_extract_string(payload,
  '$."ocel:attributes".entropy')`.
- Escape oids are disjoint from segment oids
  (`EXISTS` anti-join == 0 rows).

## See Also

`variant-types.md` ·
`docs/design/ocel-sovereign-schema-validation.md` ·
`docs/design/compression-proof-protocol.md` ·
`docs/design/rehydration-sql.md`
