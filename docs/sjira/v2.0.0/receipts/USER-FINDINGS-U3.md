# USER-FINDINGS-U3 — Offline consumer story (lane U3)

Date: 2026-10-19. Repo: /Users/sac/frozen-duckdb (canonical checkout, lane U3, no git actions).
Scratch: /tmp/u3 (isolated HOME=/tmp/u3/home, cache pre-seeded via rsync from ~/.frozen-duckdb).

## Verdict: PASS — zero-network consumer path proven

## Environment

- Isolated `HOME=/tmp/u3/home`; cache pre-seeded: `rsync -a ~/.frozen-duckdb/ /tmp/u3/home/.frozen-duckdb/`
  → `/tmp/u3/home/.frozen-duckdb/cache/` contains `v1.4.0-{arm64,x86_64}`, `v1.5.5-{arm64,x86_64}`,
  `v2.0.0-arm64` (duckdb/, ENGINE_VERSION, libduckdb_arm64.dylib{,.size}, libduckdb.dylib).
- Proxy vars unset in every command: `unset http_proxy https_proxy HTTP_PROXY HTTPS_PROXY ALL_PROXY all_proxy`.
- `--offline` on every cargo invocation (plus `HOME`-scoped cargo: `CARGO_HOME=/Users/sac/.cargo`,
  `RUSTUP_HOME=/Users/sac/.rustup` — the user registry/toolchain cache, i.e. "pre-seeded" cargo side).
- Consumer crate: `/tmp/u3/offline-consumer` — path dep on
  `/Users/sac/frozen-duckdb/crates/frozen-duckdb` (v2.0.0).

## Commands + real output

### 1. `cargo build --offline`

```
   Compiling arrow-string v56.2.1
   Compiling reqwest v0.11.27
   Compiling frozen-duckdb-builder v2.0.0 (...)
   Compiling arrow v56.2.1
   Compiling frozen-duckdb-sys v2.0.0 (...)
warning: frozen-duckdb-sys@2.0.0: Using prebuilt DuckDB binary: /tmp/u3/home/.frozen-duckdb/cache/v2.0.0-arm64/libduckdb_arm64.dylib
   Compiling offline-consumer v0.1.0 (...)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4m 17s
```

Cache HIT confirmed on the isolated pre-seeded cache (build-script log line above — the sys
crate resolved the dylib from /tmp/u3/home, not the real HOME). No network needed.

### 2. Run + query (offline)

```
$ ./target/debug/offline-consumer
range count = 100000
parquet rows = 1000
json = {"k":1}
OFFLINE-OK
RUN_EXIT=0
```

### 3. `cargo test --offline`

```
running 0 tests
test result: ok. 0 passed; 0 failed; ...
```

## Hidden-network-dependency hunt

1. **Cargo index update** — covered by `--offline`/`CARGO_NET_OFFLINE=true`; resolution
   succeeded entirely from the pre-seeded registry cache. No hidden index fetch.
2. **DuckDB extension autoinstall** — REFUTED as a risk on the happy path. A real parquet
   round-trip (COPY ... TO parquet, then read_parquet) and a json_object() query both ran
   with zero network; parquet/json are statically linked into the prebuilt mega-library,
   autoinstall never fires. No `INSTALL`/`LOAD` attempt observed.
3. **Telemetry/CDN probes in our code** — grep of `crates/*/src` and runtime path: none in
   the consumer runtime. Network strings exist only in release-prep scripts
   (`scripts/download_duckdb_binaries.sh`, `build_duckdb_optimized.sh`,
   `build_extensions_local.sh`, `create_precompiled_duckdb.sh` — curl/git to github.com)
   and in `frozen-duckdb-builder`'s **fallback** path (`download_from_github_release` via
   reqwest 0.11 blocking; `compile_duckdb_locally` fallback). These are release-manufacturing
   or cache-miss fallbacks, NOT the consumer happy path. Not blockers for the offline story.

## Findings (non-blocking, disclosed)

- F1 (informational): on a cache MISS with `--offline`, the builder's download fallback
  fails and local compile (git clone + make, hours, network) also fails → build errors.
  Expected and loud, not silent. Offline consumers must have a pre-seeded cache — which the
  pre-seeded lane proves works.
- F2 (informational): `reqwest 0.11` is a real dependency of frozen-duckdb-builder (download
  fallback). It compiles offline fine and never runs when the cache hits.
- No BLOCKER findings; no fixes to our code required.

## Standing

Offline story: ALIVE (build + query + test, zero network, isolated pre-seeded cache).
