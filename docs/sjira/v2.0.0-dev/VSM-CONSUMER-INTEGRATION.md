# Value-Stream Map: Consumer Integration (cargo add -> first query)

Lane LSS6, v2.0.0-dev window. All timings measured this session on
`/Users/sac/frozen-duckdb` (master, arm64 macOS, load present).
Status: PARTIAL_ALIVE (cold-build step is receipt-cited, not re-run).

## Measured this session (real output)

```
$ time cargo build -p frozen-duckdb
  warning: frozen-duckdb-sys@2.0.0-dev: Using prebuilt DuckDB binary:
    /Users/sac/.frozen-duckdb/cache/v2.0.0-dev-arm64/libduckdb_arm64.dylib
  Finished `dev` profile ... in 3m 41s        # exit 0

$ time cargo run -q -p frozen-duckdb --example dropin_replacement
  Average posts per user: 1.25
  Example completed successfully!             # exit 0, 1.207s total
```

## Current-state steps and cycle times

| # | Step | Time | VA/WA/Wait | Note |
|---|------|------|-----------|------|
| 1 | `cargo add frozen-duckdb` + resolve | <1s | Wait | Index + lockfile update only |
| 1b | Engine acquisition (cached) | 7-10s | WA | builder downloads/links prebuilt dylib |
| 2 | First build | 3m41s warm / ~22min cold-under-load | WA | see decomposition |
| 3 | Link | ~seconds | WA | inside step 2 finish line |
| 4 | First query | ~1.2s incl. process start | VA | dropin_replacement example |

The only value-add step in the stream is step 4. Steps 1b-3 are pure wait.

## The 20% (where the time actually is)

- The engine is NOT the drag: cached acquisition is 7-10s and the build log
  confirms `Using prebuilt DuckDB binary`.
- The drag is the 168-crate Rust dep tree (arrow 56.2.1 stack, syn/
  proc-macro2/quote proc-macros: 23 tree rows reference them).
- Correction to the dispatching claim: `cargo tree -e normal` shows NO
  zstd-sys / lz4 / snap / brotli in frozen-duckdb's tree (grep count 0).
  The "zstd-sys etc." attribution belongs to DTS2's subject, not this one.
- DTS2's 22min figure is cold-under-load; the same tree warm is 3m41s.
- `--no-default-features` changes nothing (404 tree lines either way);
  the only feature is `serde_json`, which gates impls, not deps.

## Future-state proposal (ONE 80/20 experiment)

**Exp-FDDB-VSM1: precompiled dep-tree cache.** Since feature trimming is
proven inert, the lever is shipping/pre-warming compiled dep artifacts, not
a thinner feature set. Spec:

1. Fresh `cargo new` consumer project, dep on frozen-duckdb 2.0.0-dev.
2. Baseline: `cargo build` cold (`CARGO_TARGET_DIR` on empty dir).
   Record wall time + `cargo tree | wc -l`.
3. Treatment: same build with a populated shared `CARGO_TARGET_DIR`
   prewarmed by CI (e.g. a `dep-cache` release artifact per
   (rustc, triple, profile) key).
4. Falsifier: experiment fails if the prewarm saves <50% of cold wall
   time, or if the prewarm artifact for key (rustc, triple, profile)
   misses on a consumer's toolchain (measured miss rate >5%).
5. Success metric: cold first-build drops from ~22min to <5min.
6. Second-order lever (cheaper to land): publish a
   `frozen-duckdb-sys` `links`-based prewarm in CI on every commit, so
   the cache is warm before any consumer arrives.

Secondary spec, not run: per-profile sccache over the dep tree; expect
~same hit profile as (3) with less CI plumbing.

## Timeline shape (summary)

- Total stream (warm cache): ~3m45s, of which ~1.2s is value-add (0.5%).
- Total stream (cold, load): ~23min, of which ~1.2s is value-add (0.1%).
- 99%+ of stream time is wait/waste concentrated in step 2; attack step 2
  or accept it. Everything else is already fast.
