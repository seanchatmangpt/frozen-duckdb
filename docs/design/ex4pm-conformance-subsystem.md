# ex4pm Conformance Subsystem (Subsystem B)

Cross-repo capture of Subsystem B (ex4pm) from the sovereign spec. ex4pm is a
**separate repository**; this document is the design capture in frozen-duckdb.
The SQL schema portion is validated in THIS repo — see
`docs/design/ocel-sovereign-schema-validation.md` when it exists.

## Role

Real-time conformance gatekeeper: every process-trace event is checked, at
ingest, against formal process models before it is admitted to the
conforming store. Non-conforming events take the escape path.

## Pipeline

```text
event --> α (abstraction) --> StateID check --> conforming | escape
```

## Abstraction Function α

α maps a raw event to a canonical state. Three stages:

1. **Binary pattern match** classifies the event against known process
   patterns.
2. **Unboxed state vector** packs the classification into a fixed-width
   bit-packed word:
   `<<Role::2, Term::16, Lock::1, QueueBucket::5, ...>>`
   (Erlang-style bit-syntax; widths are spec values, not derived).
3. **Canonical StateID** is the packed vector's identity in the model
   universe.

The gatekeeper never inspects raw payloads on the conforming path — it
compares packed vectors only.

## Formal Reachability Check

- `StateID ∈ V_formal` — state must be a vertex of the TLA+ model's state
  graph.
- `edge ∈ E_formal` — the observed transition must be an edge of the model's
  transition relation.
- Models are TLA+; the check is graph membership on `G_formal =
  (V_formal, E_formal)`.

## Conforming vs Escape

- **Conforming path**: the event's (StateID, edge) pair is admitted by
  setting a bit in an atomic bit-set ledger (`:atomics`, Erlang). Atomic
  per-instance, lock-free, O(1) admission.
- **Escape path**: full forensic capture — raw payload plus trace context —
  recorded in an uncompressed ledger, together with a **PQC-signed alarm
  receipt** (post-quantum signature over the capture).

## Ergodicity Math (spec formulas, transcribed not derived)

Coverage of the state space drives when enough evidence has been collected:

- `C_vol = |V_visited| / |V_formal|` — volume coverage ratio.
- `C_edge = |E_visited| / |E_formal|` — edge coverage ratio.
- **Schedule-diversity condition**: Shannon entropy of the observed action
  distribution must satisfy

  ```text
  H(A) >= 0.85 * log2(|A|)
  ```

  where `A` is the set of schedulable actions in the model.
- **Saturation stopping criterion**: stop exploring when

  ```text
  dC_vol/dt <= 1e-5 sustained for 24h, with zero escapes
  ```

  (coverage is no longer rising and the conforming path has held clean).

## Storage Split

Two stores, two retention postures:

- Conforming events -> **RLE-compressed segments in frozen-duckdb** (run-
  length encoded, matching the storage-format-v2 segment design).
- Escapes -> **uncompressed ledger** (forensic fidelity first).

### DDL — Three Tables

```sql
-- 1. Conforming RLE segments
CREATE TABLE conforming_segments (
  segment_id   UBIGINT PRIMARY KEY,
  state_id     UINTEGER NOT NULL,
  edge_id      UINTEGER NOT NULL,
  run_len      UINTEGER NOT NULL,
  span_start   TIMESTAMP NOT NULL,
  span_end     TIMESTAMP NOT NULL
);

-- 2. Escape ledger (uncompressed forensics)
CREATE TABLE escape_ledger (
  escape_id    UBIGINT PRIMARY KEY,
  state_id     UINTEGER,       -- NULL if state not in V_formal
  observed     BLOB NOT NULL,  -- raw forensic capture
  alarm_receipt BLOB NOT NULL, -- PQC-signed alarm receipt
  captured_at  TIMESTAMP NOT NULL
);

-- 3. Coverage checkpoints (ergodicity state)
CREATE TABLE coverage_checkpoints (
  checkpoint_id UBIGINT PRIMARY KEY,
  c_vol        DOUBLE NOT NULL,
  c_edge       DOUBLE NOT NULL,
  entropy_h    DOUBLE NOT NULL,    -- H(A) at checkpoint
  action_count UBIGINT NOT NULL,   -- |A|
  taken_at     TIMESTAMP NOT NULL
);

-- Rehydration view: conforming runs expanded back to per-event rows
CREATE VIEW conforming_rehydrated AS
SELECT s.segment_id, s.state_id, s.edge_id,
       unnest(generate_series(s.span_start, s.span_end,
                              INTERVAL (s.run_len) SECOND)) AS ts
FROM conforming_segments s;
```

Schema shapes are spec values; validation in this repo is tracked in
`docs/design/ocel-sovereign-schema-validation.md` when it exists.

## Standing

| Item | Standing |
|---|---|
| ex4pm repo (α, reachability, :atomics, escape) | UNKNOWN (separate repo; not observed here) |
| SQL schema portion (tables + rehydration view) | UNKNOWN until validated in this repo |
| Formulas (C_vol, C_edge, H(A), saturation) | Transcribed spec, not derived claims |
