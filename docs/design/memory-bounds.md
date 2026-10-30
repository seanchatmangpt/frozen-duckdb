# Memory Bounds Design (<=256MB In-Browser Cap)

Status: DESIGN — nothing executed this wave. All runtime claims
UNVERIFIED until the CDP RSS-polling court runs.

## Config surface (verified against source, not runtime)

- The crate's generic passthrough is `Config::with(key, value)`
  (`crates/frozen-duckdb/src/duckdb/config.rs:129`), forwarding to the
  internal `set` and the duckdb config FFI. Unknown keys fail at open
  (`test_invalid_setting` in the same file covers the failure path).
- Typed builder: `Config::max_memory(&str)`
  (`crates/frozen-duckdb/src/duckdb/config.rs:117`), which sets the
  DuckDB key `"max_memory"` (config.rs:118).
- Key-name note: DuckDB's canonical knob is `memory_limit` — the
  string `memory_limit` and the hint "* Increasing the memory limit
  (SET memory_limit='...GB')" are both present in `strings` over
  `~/.frozen-duckdb/cache/v2.0.0-arm64/libduckdb.dylib`
  (extracted 2026-10-19). Upstream, `max_memory` and `memory_limit`
  are aliases; whether the v2.0.0 runtime accepts the
  `max_memory` spelling is UNVERIFIED. Safe form until probed:
  `Config::with("memory_limit", "256MB")`.
- `threads = 1` (`Config::with("threads", "1")`) is planned as part
  of the in-browser profile (fewer worker allocators, bounded buffer
  pools). UNVERIFIED as unexecuted.

## Planned in-browser profile

- At open, the OPFS persistence layer sets
  `memory_limit = "256MB"` and `threads = 1` via the
  `Config::with` passthrough (or `max_memory("256MB")` once the
  alias question is probed).
- The cap bounds DuckDB's buffer manager, not the whole tab: wasm
  heap overhead, JS glue, and OPFS staging buffers sit outside
  `memory_limit`. The 256MB budget must therefore be treated as the
  DuckDB-internal share of the tab, with the tab-level cap enforced
  by the CDP court below.

## CDP-observed RSS polling court (design)

- Launch Chrome `--headless=new --remote-debugging-port=0`, read the
  websocket endpoint from `http://127.0.0.1:<port>/json/version`,
  open a CDP websocket, `Performance.enable`, then poll
  `Performance.getMetrics` on an interval and extract
  `JSHeapUsedSize`, `JSHeapTotalSize`, and the process RSS metric.
- Bound: peak RSS over the whole burn-in run must stay <= 256MB
  (tab-level budget; the DuckDB-internal `memory_limit` share is
  capped by config at the same run). Peak is taken over all polls,
  not an average — one excursion fails the court.
- Polling, not sleeping: the court polls a real observable on a
  deadline (no-sleep discipline); a missed metric is a loud
  `[SKIP]`/FAIL, never a faked pass.
- Falsifier: a workload that allocates > 256MB inside DuckDB (e.g.
  a large sort/hash aggregate with `memory_limit` mis-applied or
  the alias key silently ignored) must FAIL the court via the CDP
  RSS peak exceeding the bound. If the court still passes with
  `memory_limit` deliberately unset, the court is vacuous and the
  config binding is not proven — that outcome exits 1 as VACUOUS.

## See Also

- `docs/design/chrome-burnin-court.md`
- `docs/design/wasm-opfs-architecture.md`
- `crates/frozen-duckdb/src/duckdb/config.rs` (`Config::with`,
  `Config::max_memory`)
