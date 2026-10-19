# How To: Compress and Rehydrate OCEL 2.0 Event Logs

Problem-oriented guide: fold a per-event OCEL 2.0 stream into run-length
segment vectors, measure the ratio against a real JSONL baseline, and
rehydrate losslessly back into per-event rows.

Standing note: the SQL in this guide is grounded in
`crates/frozen-duckdb/tests/doctor_ocel_tests.rs` (TEST-018/019/020), the
doctor stage `07_ocel_compress` court that ran 94.93% compress / 97.01%
query (receipt: `docs/sjira/v2.0.0-dev/CAMPAIGN-RECEIPT.md` lines 319,
373). The scalar DDL variant in `docs/design/rehydration-sql.md` is
VERIFIED-SYNTAX only (v1.4.1 CLI, degraded collaborator); this guide
follows the test file, which ran on the pinned engine.

See Also: `docs/design/ocel2-compression.md`,
`docs/design/rehydration-sql.md`,
`docs/design/compression-proof-protocol.md`

## Fold: stage, group, checkpoint

The corpus law: conforming transitions fold; raw JSON rows are reserved
for non-conforming "escape" events only. The court ingests 100k events
(`FDOCEL_EVENTS=1000000` scales to the spec contract size), keeps 5
escape rows, and folds the rest.

```sql
CREATE TABLE events_staging(
    n BIGINT, oid VARCHAR, action VARCHAR,
    channel VARCHAR, ts TIMESTAMP);

CREATE TABLE ocel_conforming_segments(
    segment_id BIGINT,
    oid VARCHAR,
    action VARCHAR,
    channel VARCHAR,
    t_start TIMESTAMP,
    t_end TIMESTAMP,
    transition_count BIGINT);

CREATE TABLE ocel_escapes(
    escape_id BIGINT,
    oid VARCHAR,
    payload JSON);   -- or VARIANT; probe first, see below
```

The fold (grouping key must match whatever produced the segments —
grouping drift silently kills the singleton witness):

```sql
INSERT INTO ocel_conforming_segments
SELECT row_number() OVER (ORDER BY b) AS segment_id,
       any_value(oid)    AS oid,
       any_value(action) AS action,
       any_value(channel) AS channel,
       min(ts) AS t_start,
       max(ts) AS t_end,
       count(*)::BIGINT AS transition_count
FROM (SELECT *, CASE WHEN n = 700 THEN 1000001
                     ELSE n / 100 END AS b
      FROM events_staging) q
GROUP BY b;
```

Then `DROP TABLE events_staging; CHECKPOINT;` — the ratio is measured on
the post-CHECKPOINT db file bytes, never asserted.

## VARIANT probe (engine-version fence)

On some engines VARIANT DDL is refused; probe, do not assume:

```rust
let ddl_form = match conn
    .execute_batch("CREATE OR REPLACE TABLE _variant_probe(x VARIANT);")
    .map(|_| conn.execute_batch("DROP TABLE _variant_probe;"))
{
    Ok(_) => "VARIANT",
    Err(_) => "JSON",
};
```

Cast escape payloads with the probed form: `'...'::VARIANT` or
`'...'::JSON`.

## Rehydrate: the hardened COALESCE view

The spec-literal `NULLIF` form silently drops singleton segments
(`generate_series` with a NULL step returns 0 rows). Use the hardened
span-relative microsecond form with a COALESCE heal:

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

Singleton law: `transition_count = 1` yields `NULL` step -> COALESCE
picks `1` microsecond, so the segment emits exactly one row at `t_start`
instead of zero rows. The court plants a deliberate singleton witness
(event n=700 pulled into a disjoint block) so this NULLIF vacuity is
visible to conservation counting.

Timestamp law: intra-segment cadence is assumed uniform;
`span / (n-1)` is the step. Original per-event jitter is NOT stored —
it is the one lossy dimension (see the lossiness table in
`docs/design/ocel2-compression.md`). Fully non-uniform runs belong in
`ocel_escapes`, never folded.

## Measure the ratio (NFR-03 contract)

Pass line: compressed db bytes <= 0.10 x raw JSONL bytes (>= 90%
smaller). Measured on disk, never asserted:

```sql
SELECT
  (SELECT count(*) FROM events_baseline)  AS raw_events,
  (SELECT sum(transition_count)
     FROM ocel_conforming_segments)       AS folded_events,
  (SELECT sum(transition_count)
     FROM ocel_conforming_segments)
    = (SELECT count(*) FROM events_baseline) AS count_conserved;
```

```bash
ls -l nfr03_baseline.jsonl nfr03_compressed.db
# ratio = db_bytes / jsonl_bytes; court requires <= 0.10
```

Measured this campaign: **94.93% reduction** (compress leg) and
**97.01%** (query leg) on the 100k-event court — both clear the 90%
pass line.

## Run the court

```bash
just doctor   # full battery; OCEL stage is 07_ocel_compress
# or directly:
cargo test -p frozen-duckdb --test doctor_ocel_tests
# spec contract size:
FDOCEL_EVENTS=1000000 cargo test -p frozen-duckdb \
  --test doctor_ocel_tests
```

Falsifier: any invariant `false`, any rehydration count mismatch, or a
singleton row missing refutes the fold/rehydration law on that engine.

See Also: `docs/how-to/how-to-run-chicago-doctor.md`
