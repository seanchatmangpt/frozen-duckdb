# Persona x Journey Map (PRD §2.2 x §2.3)

Source: `docs/design/prd-ard-v2.md` §2.2 (personas) and §2.3 (journeys).
Standings re-read from `docs/design/requirements-traceability.md`
(2026-10-19).

Standing vocabulary: PARTIAL_ALIVE = native reality proves part of the
requirement (court receipt on real run); PROPOSED = specified, no court
receipt on any subject.

## Master matrix

Personas: P1 edge & systems engineers; P2 enterprise app developers;
P3 data analysts & process practitioners.

| persona | journey | capability | reqs | court | standing |
|---|---|---|---|---|---|
| P1 | local query | drop-in `Connection`, local engine | FR-03/05 | V-G2 tests | PARTIAL_ALIVE |
| P1 | zero-cloud agg | `memory_limit` via `Config::with` | FR-05 | V-W3 (absent) | PARTIAL_ALIVE |
| P1 | crash recovery | local file persistence | FR-02 | V-W2 / BP2 C1 | PROPOSED |
| P2 | local query | browser worker + sync OPFS handle | FR-01/02 | V-W1 wasm build | PROPOSED |
| P2 | zero-cloud agg | on-device window agg, OPFS tables | FR-01/02/05 | V-W3 + BP2 C2 | PROPOSED |
| P2 | crash recovery | worker + OPFS + WAL replay | FR-01/02 | V-W2 / BP2 C1 | PROPOSED |
| P3 | local query | sub-15ms agg over 100k rows | NFR-01 | none (GAP: V-L1) | PROPOSED |
| P3 | zero-cloud agg | RLE/dict-compressed segments | NFR-03, AC-4 | V-S2 (never run) | PROPOSED |
| P3 | crash recovery | OCEL segment rehydration (BP12) | FR-04 | V-S1 (absent) | PROPOSED |

PARTIAL_ALIVE detail: V-G2 is `cargo test -p frozen-duckdb --test
variant_tests` (5 passed / 0 failed, exit 0, real run). P1 memory-cap
PARTIAL_ALIVE is plumbing only — `Config::with` accepts `memory_limit`
(`crates/frozen-duckdb/src/duckdb/config.rs:129`); no test exercises it
and the wasm memory court does not exist.

## Walkthroughs: today vs target

### P1 edge & systems engineer

- **Today**: only real path is the native Rust crate — drop-in
  `Connection` API, local engine, no browser. Real: VARIANT queries
  (5/5 native battery, exit 0); `memory_limit` caps via `Config::with`
  (plumbing, untested); OCEL segment storage via BP12-descended SQL
  shapes (`docs/design/view-expansion-design.md`); local file
  persistence. Not today: zero-panic trap boundary (NFR-04 not met on
  the native path and cannot be, per
  `docs/design/panic-confinement-protocol.md`), WASM sandbox (FR-01),
  OPFS (FR-02).
- **Target**: engine compiles to wasm32 (FR-01, V-W1), statically
  bound, zero host-process panic risk (NFR-04 via sandbox trap
  boundary, BP2 Courts 2/3). Ordering: FR-01 first; NFR-04 is
  FR-01-gated.
- **Blockers**: no `scripts/wasm/`; no emcmake build ever attempted
  against the vendored headers.

### P2 enterprise app developer

- **Today**: nothing on the browser path. The native crate is the only
  execution substrate; no worker, no `opfs/` module, no quota mapping.
  P2 has no artifact to integrate against at all — `scripts/wasm/` and
  `opfs/` do not exist on disk.
- **Target**: boot the engine in a dedicated browser worker, open a
  synchronous OPFS access handle, sub-second vectorized SQL over
  millions of rows; persistence survives `Page.crash` with WAL replay
  (FR-01 + FR-02, V-W2, BP2 Court 1); typed `QuotaExceededError`
  mapping (FR-02-quota, BP2 Court 3).
- **Blockers**: FR-01 artifact absent (V-W1 cannot run); OPFS
  block-I/O layer unimplemented; Chrome quota sizes UNVERIFIED.

### P3 data analyst / process practitioner

- **Today**: can store OCEL segments as compressed columnar tables via
  the native crate (BP12-descended SQL shapes) and query VARIANT
  columns; can set `memory_limit` caps. No daemons needed, but no
  sub-15ms guarantee — no latency court exists (NFR-01 GAP: no
  harness, no baseline). No browser/interactive surface; interface is
  Rust only.
- **Target**: interactive sub-15ms local queries across multi-object
  event streams (NFR-01, needs a V-L1 latency court); compressed
  storage >=90% vs raw JSON (NFR-03, V-S2, court never run); lossless
  segment rehydration (FR-04, V-S1, not implemented).
- **Blockers**: no V-L1 court defined (court-coverage GAP in
  `docs/design/verification-plan.md`); 1M-event corpus not built
  (AC-4); `ocel_rehydration_equivalence` test file does not exist.

## Honest column: what works TODAY (2.0.0-dev native engine)

| persona | real today (native only) | not today |
|---|---|---|
| P1 | VARIANT 5/5 exit 0; `memory_limit` plumbing; local files; BP12 | wasm; panic trap; OPFS |
| P2 | nothing — no browser artifact exists | everything browser-side |
| P3 | OCEL segs + VARIANT queries; memory caps | sub-15ms p95; rehydration; compress proof |

`memory_limit` is real plumbing (`Config::with`) but unexercised by
any test; V-W3 does not exist. FR-05 native standing is PARTIAL_ALIVE
on plumbing only.

## See Also

`docs/design/prd-ard-v2.md` ·
`docs/design/requirements-traceability.md` ·
`docs/design/verification-plan.md` ·
`docs/design/panic-confinement-protocol.md` ·
`docs/design/view-expansion-design.md`
