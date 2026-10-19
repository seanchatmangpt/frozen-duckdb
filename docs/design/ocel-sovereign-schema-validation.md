# ocel-sovereign-schema-validation.md

Chicago-style execution validation of the sovereign spec's §3.3/§3.4 OCEL
storage SQL (`ocel_object_epochs`, `ocel_conforming_segments`,
`ocel_escapes`, and the `unnest(generate_series(...))` rehydration view)
against a real engine. Companion capture: `ex4pm-conformance-subsystem.md`
(Subsystem B) and `sovereign-spec-v26.10.19.md` §3.

## Engine identity (degraded substrate, loudly labeled)

| field | value |
|---|---|
| engine | `/opt/homebrew/bin/duckdb` CLI |
| version | `v1.4.1 (Andium) b390a7c376` |
| why degraded | the repo-linked frozen-duckdb native dylib is mid-rebuild; no `target/**/*.dylib` exists at run time (checked: `ls target/debug/*.dylib target/release/*.dylib` → no matches). Same degraded-real-substrate posture as BP12. |
| working db | `/tmp/x5_ocel.duckdb` (persistent; each bare CLI invocation is a fresh in-memory db — first attempt lost all DDL across statements, receipt below) |
| scratch script | `/tmp/x5_ocel.sql` (final validated form) |

Every finding below is from a real executed command with its actual output.
Nothing here is asserted from the spec text alone.

## Per-query status

| # | query | status | receipt |
|---|---|---|---|
| Q1 | `CREATE TABLE ocel_object_epochs (... base_attributes VARIANT)` | **FAIL (typed error — this IS the finding)** | `Not implemented Error: A table cannot be created from a VARIANT column yet` |
| Q1' | same DDL with `base_attributes JSON` | PASS | table created; `PRAGMA table_info` shows `base_attributes \| JSON` |
| Q2 | `CREATE TABLE ocel_conforming_segments` (RLE segments) | PASS | created, 3 rows inserted |
| Q3 | `CREATE TABLE ocel_escapes` (escape ledger) | PASS | created, 1 row inserted |
| Q4 | VARIANT insert `'{"k":1}'::VARIANT` | **BLOCKED upstream of cast** | unreachable: table with VARIANT column cannot exist on 1.4.1 (Q1). No VARIANT cast was executed; none is claimed. |
| Q4' | JSON insert `'{"k":1}'::JSON` | PASS | 2 rows; read-back `json_extract(base_attributes,'$.k')` → `1`, `2` |
| Q5 | rehydration view, spec-literal NULLIF guard | **FAIL — silent under-count (key finding)** | segment with `transition_count = 1` yields **0 rows**: `generate_series(t, t, NULL)` → `0 rows` (probe receipt below). Conservation: expected 16, actual 15, `conserved = false` |
| Q5' | hardened view: `to_microseconds(COALESCE((date_diff('microsecond', t_start, t_end) / NULLIF(transition_count - 1, 0))::BIGINT, 1))` | PASS | expected 16, actual 16, `conserved = true` |
| Q6 | rehydration SELECT (multi-segment) | PASS | segment 1 → 10 rows (09:00:00–09:09:00, 60 s spacing), segment 2 → 5 rows (10:00:00–10:04:00), segment 3 (singleton) → 1 row at 11:00:00 |
| Q7 | event-count conservation `sum(transition_count) == rehydrated count` | PASS (with Q5' view) | `16 = 16, true` |
| Q8 | §3.2 ledger conservation `rehydrated + escapes == observed` | PASS | `17 = 17` |

### Mid-run receipt worth keeping (toolchain behavior)

Piping a multi-statement script to a bare `duckdb` (no db file) ran each
statement in one session but the next invocation started empty —
`Catalog Error: Table with name ocel_conforming_segments does not exist!
Did you mean "sqlite_temp_master"?`. Fixed by pinning the persistent db
file `/tmp/x5_ocel.duckdb`. Heredoc-per-invocation is not a stateful
engine session; receipts must come from the persistent-db run.

## Findings (typed divergences, engine 1.4.1 vs spec)

### D1 — VARIANT column DDL rejected (BLOCKED, typed error)

Spec form: `base_attributes VARIANT`. Engine answer, verbatim:

```text
Not implemented Error: A table cannot be created from a VARIANT column yet
```

`CREATE TABLE ... (col VARIANT)` is not implementable on 1.4.1 — this is
a storage-layer refusal, not a cast problem; the `'{...}'::VARIANT` cast
(Q4) was never reached and no behavior is claimed for it. Validated
fallback: `JSON` type, same DDL shape otherwise, insert and
`json_extract` read-back both green. The sovereign spec's VARIANT pipeline
(see `variant-shredding-design.md`, `DUCKDB_TYPE_VARIANT=41`) remains
UNVERIFIED on this substrate; re-run Q1/Q4 against the repo-linked 1.5.5 /
2.0.0-dev engine when the dylib rebuild lands.

### D2 — spec-literal NULLIF guard silently drops singleton segments (spec defect, real)

The guard as specified — step computed as
`span / NULLIF(transition_count - 1, 0)` — is division-safe but not
conservation-safe. For `transition_count = 1` the step is NULL and
`generate_series(t_start, t_end, NULL)` returns **zero rows** (probe:
unnest over identical timestamps with NULL step → `0 rows`; with
`INTERVAL 1 SECOND` → exactly 1 row). A singleton run (an object whose
entire history is one event) silently vanishes from rehydration. Measured:
expected 16, actual 15, `conserved = false`, and the per-segment grouping
shows segment 3 absent.

Fix (validated, `conserved = true` at 16/16): heal the NULL step with
`COALESCE(..., to_microseconds(1))` — a degenerate span with a 1 µs step
yields exactly one event at `t_start`. Recommended spec amendment: the
§3.4 NULLIF guard MUST be paired with a COALESCE default step, and Q7
conservation MUST include at least one `transition_count = 1` segment —
without the singleton the vacuity is invisible (first run without it
showed `conserved = true` on 15/15 with the defective view shape that
ignored `Counts`; the earlier naive fixed-second step run measured 143 vs
15, also `false` — both failure modes are receipted).

### D3 — step derivation must be span-relative, not literal seconds

`generate_series(t_start, t_end, INTERVAL (n) SECOND)`-style literal steps
over-fan (measured 143 rows for 15 transitions before switching to
span-relative microsecond math). The lawful form is
`span / (transition_count - 1)` in microsecond BIGINT arithmetic with an
explicit `::BIGINT` cast — `to_microseconds(DOUBLE)` is a binder error
(`Candidate functions: to_microseconds(BIGINT) -> INTERVAL`), because
DuckDB `/` on integers yields DOUBLE.

## Standing

| item | standing |
|---|---|
| §3.3 DDL, JSON-attribute fallback form (Q1', Q2, Q3) | VERIFIED on 1.4.1 (real DDL + inserts + read-back receipts) |
| §3.3 DDL, VARIANT spec form (Q1, Q4) | BLOCKED — typed engine refusal, receipt above |
| §3.4 rehydration view, hardened COALESCE form (Q5', Q6, Q7, Q8) | VERIFIED on 1.4.1 (conservation 16/16 and 17/17) |
| §3.4 rehydration view, spec-literal NULLIF-only form | REFUTED — silent singleton drop, receipt above |
| all of the above on the repo-linked 1.5.5 / 2.0.0-dev engine | UNKNOWN — re-run required when the native dylib rebuild lands |

## Falsifier

Re-run `/tmp/x5_ocel.sql` against the repo-linked engine once available:
(1) Q1 must accept or produce a new typed error (either outcome updates
D1); (2) Q7 conservation with the singleton segment must still report
`conserved = true`; (3) deliberately dropping the COALESCE must flip Q7 to
`false` — if the mutated view still conserves, the court is vacuous and
this document's Q5' claim is FAIL.

## See Also

`ex4pm-conformance-subsystem.md` · `sovereign-spec-v26.10.19.md` ·
`variant-shredding-design.md` · `ocel2-compression.md`
