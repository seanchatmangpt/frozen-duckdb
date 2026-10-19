# compression-proof-protocol.md

NFR-03 / AC-4 court protocol: prove on-disk that BP3 segment-vector
storage is at least 90% smaller than a raw JSON-lines baseline over the
same 1,000,000-event corpus. Companion to `docs/design/ocel2-compression.md`
(BP3 shape) and `docs/design/rehydration-sql.md` (BP12 executed SQL).

## Claim under court (NFR-03 / AC-4)

Ingest the same 1,000,000 deterministic process events two ways:

- **A (baseline)**: raw JSON lines, one event per line, stored via a
  DuckDB table of one `JSON` column per line.
- **B (candidate)**: BP3 segment-vector tables
  (`ocel2_segments` + `ocel2_escapes` + dictionaries) with DuckDB
  columnar compression.

**Pass**: `bytes(B) <= 0.10 * bytes(A)` (>= 90% reduction), measured as
on-disk `.db` file bytes, same engine, same run. Ratio is MEASURED, never
asserted: the court reports whatever ratio it observes. Fail = ratio above
10% at the spec'd escape ceiling.

## Execution status (receipt-first)

All SQL below follows the storage shape of BP12's executed queries
(`docs/design/rehydration-sql.md` Q1/Q2: `generate_series` fan-out,
`date_diff` microsecond steps, `'{}'::JSON` literals, `AT TIME ZONE 'UTC'`
strftime). That batch is **VERIFIED-SYNTAX on duckdb v1.4.1 CLI
(`b390a7c376`)**; **UNVERIFIED against the pinned frozen engine** until the
harness runs through `cargo run -p test-validation` (dylib cache
mid-rebuild; see rehydration-sql.md execution status). No ratio in this
document is an executed receipt; the court mints the only real one.

## Generator contract (deterministic, seed-pinned)

`gen_corpus.rs` (bin in the test-validation crate) emits exactly
1,000,000 events as JSON lines to stdout. Determinism law: same
`--seed`, byte-identical corpus (sha256 recorded into the court receipt).

```
gen_corpus --seed 20261019 --events 1000000 \
  --escape-rate 0.02 > /tmp/nfr03_corpus.jsonl
```

Event shape (matches BP3 schema, one JSON object per line):

```json
{"ocel:id": "e-<n>", "ocel:timestamp": "...",
 "ocel:type": "create order",
 "ocel:attributes": {"channel": "web"},
 "ocel:relationships": [{"ocel:oid": "order-17",
                         "ocel:type": "orders"}]}
```

- 8 event types, 2 object types, 100,000 objects, Zipf action mix.
- Timestamps: uniform hourly cadence per object (uniform cadence is what
  segment `Dt`/`TStart`/`TEnd` encode losslessly).
- `ocel:attributes.channel` drawn from 12 values — RLE-friendly.
- **Escape rate**: fraction of events forced through the escape path
  (random attribute payload + non-uniform jitter, class
  `NON_UNIFORM_CADENCE` / `UNKNOWN_ATTRIBUTE`). Default spec'd ceiling:
  **2%** (`--escape-rate 0.02`). The court MUST report the ratio AT this
  ceiling, not at a friendlier rate; escape rows inflate B (raw JSON rows
  in `ocel_escapes`) and the ceiling is the honest operating point.

## Ingest harness contract

Two scratch databases, same engine, deleted before each run:

- A: `nfr03_baseline.db` —
  `CREATE TABLE raw_lines(line JSON); INSERT INTO raw_lines
  SELECT * FROM read_json_objects(...)` or the equivalent
  `INSERT ... VALUES` streaming path.
- B: `nfr03_compressed.db` — BP3 DDL from
  `docs/design/ocel2-compression.md`
  (`ocel2_object_types`, `ocel2_event_types`, `ocel2_objects`,
  `ocel2_states`, `ocel2_actions`, `ocel2_segments`, `ocel2_escapes`),
  then the batch RLE ingest (staging events + window-function
  run-boundary fold), escapes landing raw in `ocel2_escapes`.

## Measurement (on-disk bytes, not memory)

After `CHECKPOINT;` on each database, the court reads file bytes —
never `memory_usage`, never row estimates:

```bash
stat -f %z nfr03_baseline.db nfr03_compressed.db
```

```sql
CHECKPOINT;
```

Report line (emitted verbatim by the harness, this is the receipt):

```
NFR03-COURT events=1000000 seed=20261019 escape_rate=0.02 \
  bytes_A=<n> bytes_B=<n> ratio=<bytes_B/bytes_A> \
  verdict=PASS|FAIL   # PASS iff ratio <= 0.10
```

## Falsifiers (the court must be able to fail)

- **F-C1 padded-payload honesty**: rerun the harness with
  `--payload padded-random` — attribute values become unique random
  strings that defeat RLE and dictionary encoding. The court must still
  run and report the observed ratio whatever it is; suppression,
  skip-on-bad-news, or a faked verdict is court corruption. Expected:
  ratio > 10% (FAIL verdict) — that FAIL is the proof the measurement
  reads real bytes, not a rubber stamp.
- **F-C2 escape-ceiling sensitivity**: sweep `--escape-rate` over
  {0.0, 0.02, 0.05, 0.10} and report all four ratios. Monotone
  non-decreasing ratio in escape rate is the expected shape; a reported
  ratio at 0.02 that beats the 0.0 ratio is a measurement bug.
- **F-C3 conservation guard**: `SUM(Counts) + escape_count == 1,000,000`
  (BP12 Q5 invariant style, executed `true` on fixtures). Any drift
  fails the court regardless of ratio.
- **F-C4 vacuity guard**: a court run where path B crashes or skips
  ingest is FAIL (loud-skip law), never a green cell. The AFTER flip is
  the proof: if the court cannot produce a FAIL under F-C1's adversarial
  payload, the court is VACUOUS.

## Standing

UNVERIFIED until one full harness run on the pinned engine emits the
NFR03-COURT receipt line. VERIFIED-SYNTAX claims cover the SQL shapes
only (duckdb v1.4.1 CLI, BP12 batch). Live `published`/release claims are
out of scope: the terminal standing here is a measured ratio, not a
publish.
