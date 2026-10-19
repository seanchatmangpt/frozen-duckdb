# revenue-packs.md

## Status

DESIGN — transcription of the operator's four revenue-pack specifications,
with each pack's executable witness being the Wave 12 `TEST-REV-*` court in
`crates/frozen-duckdb/tests/`. All price bands are ASSUMPTION-UNVERIFIED
operator claims; **no dollar figure in this repo is measured** (same policy
as `soc-valuation-model.md` and sovereign-spec §6). The verbatim operator
price-band text is not present in material available to the transcribing
lane — the transcription point is reserved and each band is marked
ASSUMPTION-UNVERIFIED rather than fabricated.

Owner: lane R5 (Wave 12). Court doctor stage:
`scripts/doctor/11_revenue_packs.sh` (fail-closed; not yet wired into
`scripts/doctor.sh` — that aggregate edit is a coordinator transition
after the lanes land).

## Pack 1 — Compliance pack (TEST-REV-001)

| field | value |
|---|---|
| product | 10M-row multi-currency journal trial-balance compliance on the frozen engine |
| court | `crates/frozen-duckdb/tests/test_rev_compliance.rs` (TEST-REV-001) |
| mechanism | physical db file + appender bulk ingest (10M rows, DECIMAL(18,4), 4 currencies); operator's `GROUP BY account_id HAVING sum(debit) <> sum(credit)` trial-balance query; reconciled balance dump hashed with BLAKE3 (b3sum; sha256 fallback, tool named in receipt) |
| assertions | `count(*) = 10_000_000`; variance rows = 0 (trial balance clears); global `sum(debit) = sum(credit)` (egress 0 — nothing leaves the system); wall < 2.5 s clause (recorded MET/UNMET-BY-MEASURED, honestly either way) |
| receipt | `/tmp/value_audit/rev001.json` |
| price band | ASSUMPTION-UNVERIFIED operator claim — transcription point reserved |

## Pack 2 — Process-mining pack (TEST-REV-002)

| field | value |
|---|---|
| product | OCEL-native process mining: operator bottleneck query over `ocel_conforming_segments` (250k segments, D8 schema pattern) |
| court | `crates/frozen-duckdb/tests/test_rev_process_mining.rs` (TEST-REV-002) |
| mechanism | operator query verbatim (`from`/`to`/`execution_volume`/`avg_dwell`/`p95_dwell` with `quantile_cont` or recorded `quantile_disc` fallback); median-of-20 latency; RSS-delta memory proxy (≤128 MB) |
| assertions | latency reported real vs the <10 ms spec line (comparison target, recorded, not asserted as gate); memory bounded-delta assert hard |
| browser/OPFS leg | **BLOCKED FR-01/FR-02** — the wasm/OPFS-in-Chrome leg has no harness in this repo (FR-01: no wasm build surface; FR-02: no OPFS adapter). Recorded as a loud registered divergence inside the test; the doctor stage echoes it loudly and never fakes it |
| receipt | `/tmp/value_audit/rev002*.json` per test module contract |
| price band | ASSUMPTION-UNVERIFIED operator claim — transcription point reserved |

## Pack 3 — Clean-room pack (TEST-REV-003)

| field | value |
|---|---|
| product | two-party clean-room DP-join: Party A (1M hashed profiles) x Party B (500k loyalty txns) in two isolated `ATTACH ':memory:'` schemas, join on `entity_hash` only |
| court | `crates/frozen-duckdb/tests/test_rev_clean_room.rs` (TEST-REV-003) |
| mechanism | isolated attached in-memory schemas; operator's `matched_cohort` query verbatim with illustrative DP-flavored `random()` masking (**NOT formal ε-DP** — the test module itself records this honestly); min-of-10 latency; `ps` RSS probes; cwd file-census privacy assert |
| assertions | one noisy row per cohort (64 rows); RSS delta < 256 MB; zero stray persistent `db/duckdb/wal/tmp` files (raw tables never touch disk); min latency ≤ 45 ms spec target |
| receipt | `/tmp/value_audit/rev003.json` |
| price band | ASSUMPTION-UNVERIFIED operator claim — transcription point reserved |

## Pack 4 — Edge-autonomy pack (TEST-REV-004)

| field | value |
|---|---|
| product | edge sensor-telemetry anomaly detection under a hard `memory_limit`: 1M-packet corpus, 100 sensors, windowed z-score `QUALIFY` query verbatim |
| court | `crates/frozen-duckdb/tests/test_rev_edge_autonomy.rs` (TEST-REV-004) |
| mechanism | 256 MB `memory_limit` config; seeded splitmix64 corpus with +7.5σ injected outliers (~1%) known a priori; windowed z-score QUALIFY with 3.0 threshold |
| assertions | ingest+query wall (<800 ms spec, recorded honestly); RSS flat across query (bounded-delta, 192 MB tolerance); detection recall ≥ 0.98 hard floor against the known injected set (exact id-set match) |
| receipt | `/tmp/value_audit/rev004.json` |
| price band | ASSUMPTION-UNVERIFIED operator claim — transcription point reserved |

## Status table

| pack | court file | doctor leg | standing at write time |
|---|---|---|---|
| Compliance (REV-001) | `test_rev_compliance.rs` | `cargo test -p frozen-duckdb test_rev_compliance` | IN-FLIGHT — file landed (R1-R4), stage run 2026-10-19 exited 1: compile error E0277 `Vec<Box<dyn ToSql>>: AppenderParams` at `test_rev_compliance.rs:81` (6 errors); no PASS witnessed |
| Process mining (REV-002, native) | `test_rev_process_mining.rs` | `cargo test -p frozen-duckdb test_rev_process_mining` | IN-FLIGHT — file landed; blocked on the shared compile break above; browser leg BLOCKED FR-01/FR-02 by design |
| Clean room (REV-003) | `test_rev_clean_room.rs` | `cargo test -p frozen-duckdb test_rev_clean_room` | IN-FLIGHT — file landed; blocked on the shared compile break above |
| Edge autonomy (REV-004) | `test_rev_edge_autonomy.rs` | `cargo test -p frozen-duckdb test_rev_edge_autonomy` | IN-FLIGHT — file landed; own compile error E0277 `(i64,i64,i64,f64): AppenderParams` at `test_rev_edge_autonomy.rs:131`; no PASS witnessed |
| browser/OPFS leg (all packs) | none in repo | stage 11 loud echo | BLOCKED FR-01/FR-02 — no wasm build surface (FR-01), no OPFS adapter (FR-02); never executed, never faked |

Falsifier for this document: any pack row claiming a PASS standing without
a court receipt on the exact subject (exit-0 stage run naming all four
legs). Standing flips only through receipted re-runs of
`scripts/doctor/11_revenue_packs.sh` after the owning lanes repair the
compile breaks.

## See Also

`docs/design/soc-valuation-model.md` ·
`docs/design/sovereign-spec-v26.10.19.md` (§6 legal ratchet) ·
`docs/design/doctor-implementation-map.md` ·
`crates/frozen-duckdb/tests/test_rev_{compliance,process_mining,clean_room,edge_autonomy}.rs`
