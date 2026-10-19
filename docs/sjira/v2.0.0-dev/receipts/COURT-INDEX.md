# COURT-INDEX — materialized court JSON receipts (GAP14 closure)

Lane ERRC-C1 (Create), 2026-10-19. Source: `/tmp/value_audit/*.json` — copied
verbatim (byte-identical, verified with `cmp`, 15/15 exact). Provenance per
file lives in the sidecar `<name>.meta` (source path, court, engine, date,
sha256). JSON bodies are unmodified; headers are sidecar, not embedded, so the
files stay strict JSON. No fabricated entries; every row below cites a file
present on disk at write time.

| file | court | verdict | engine identity | date |
|---|---|---|---|---|
| val001.json | VALUE-VAL-001 (compression + filter, 500k events) | FAIL | duckdb native dylib (`@rpath/libduckdb.dylib`) | 2026-10-19 |
| val002.json | TEST-VAL-002 (DFG segment latency) | PASS | frozen-duckdb native | 2026-10-19 |
| val003.json | TEST-VAL-003 / GATE-V-01 counterfactual | ALIVE | frozen-duckdb native | 2026-10-19 |
| val004.json | VALUE-VAL-004 (funnel/egress) | PASS | frozen-duckdb native (wasm leg FR-01-blocked) | 2026-10-19 |
| rev001.json | TEST-REV-001 (10M-row reconciliation) | UNMET-BY-MEASURED (clause_lt_2_5s) | v2.0.0-dev88595 | 2026-10-19 |
| rev002.json | TEST-REV-002 (p95 + RSS) | within_spec=true (native leg) | frozen-duckdb native (browser leg BLOCKED FR-01) | 2026-10-19 |
| rev003.json | REV-003 (isolated-schema join) | ALIVE | frozen-duckdb native @ e4cfbd38 | 2026-10-19 |
| rev004.json | TEST-REV-004 (outlier recall) | OBSERVED | frozen-duckdb in-memory, memory_limit=256MB | 2026-10-19 |
| fleet001.json | FLEET-001 (quantile latency) | latency MET, RSS WITHIN-BUDGET (proxy leg) | `@rpath/libduckdb.dylib`; rustc 1.97.0 | 2026-10-19 |
| fleet002_proxy.json | TEST-FLEET-002-class-b-proxy | PARTIAL_ALIVE (proxy ceiling) | frozen-duckdb native (macos) | 2026-10-19 |
| fleet003.json | TEST-FLEET-003 (epoch ledger) | ALIVE | frozen-duckdb native | 2026-10-19 |
| fleet004.json | FLEET-004 (compression + satellite) | ALIVE | v2.0.0-dev88595 | 2026-10-19 |
| fleet004_chaos.json | fleet_d_chaos court (kill -9 + SHIM-COURT) | legs a/b/d rc=0; leg c rc=2 (VACUOUS-GATE detected, expected) | v1.5.5 (t2writer) | 2026-10-19 |
| fleet005.json | FLEET-005 class E telco | ALIVE (duckdb leg); BEAM leg BLOCKED-OTHER-REPO | duckdb native in-process @ e4cfbd38 | 2026-10-19 |
| gate_v01_receipt.json | GATE-V-01 (p95 budget) | p95 10.5088ms < 15ms budget | frozen-duckdb native | 2026-10-19 |

## Missing receipts

None. All candidates (`val001-004`, `rev001-004`, `fleet001-005`,
`fleet002_proxy`, `fleet004_chaos`, `gate_v01_receipt`) existed in
`/tmp/value_audit/` and were copied. No MISSING rows.

## Related non-JSON artifacts (not copied — out of scope for this lane)

`doctor_out.txt`, `events.jsonl`, `val001.duckdb(+wal)`,
`fleet001.duckdb`, `fleet003_state.bin`, `rev001_balances.csv` remain in
`/tmp/value_audit/`; digest anchors for them are embedded in the receipt JSONs
(e.g. rev001 blake3 digest, fleet004 db_sha256).
