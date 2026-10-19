# wasm-opfs-architecture

In-browser frozen-duckdb: DuckDB compiled to WebAssembly, running in a Web
Worker, persisting through OPFS synchronous `FileSystemSyncAccessHandle`s.
Status vocabulary follows repo law: every untested claim in this document is
marked UNVERIFIED until a real build or court run proves it on an exact
subject.

## Motivation

Local-first analytics with zero cloud compute. The whole value proposition of
frozen-duckdb — a drop-in `duckdb` replacement with no compile step on the
consumer side — maps directly onto the browser: the WASM artifact ships the
engine, the user's browser is the runtime, and no server ever executes a
query. A data-heavy page that today must call an API to run SQL could instead
ship a single `.wasm` blob plus the frozen-duckdb builder/cache contract
(`~/.frozen-duckdb/cache/` becomes an OPFS-backed cache directory), query
locally, and persist durably across reloads via OPFS. Privacy follows from the
same shape: data never leaves the tab.

This extends the existing builder-sub-crate pattern rather than replacing it.
The native builder already knows how to acquire-or-compile the engine and
cache it; the WASM target is a second acquisition backend producing a
`duckdb_wasm` artifact instead of a `.a`/`.so`.

## Module layout

Worker isolation is the containment boundary. DuckDB runs in one dedicated
Web Worker; the main thread never links the engine and only sees a
message-port RPC surface (open/execute/fetch pairs, transferable
`ArrayBuffer`s for row payloads).

- `wasm-worker/` — the worker entry. Owns the DuckDB instance, the OPFS
  handles, and the module cache. A crash inside the worker (OOM kill of the
  wasm linear memory, uncaught in engine) tears down only the worker, not the
  page. UNVERIFIED: whether a wasm OOM aborts the worker cleanly or wedges
  the page's message channel; needs a real crash court.
- `rpc/` — main-thread client. Promise API mirroring the native
  frozen-duckdb surface; rejects on worker `error`/`messageerror` events so a
  dead worker surfaces as a typed error, not a hung promise.
- `opfs/` — block-I/O layer, see next section. Lives entirely inside the
  worker: `FileSystemSyncAccessHandle` is only reachable from a Worker (and
  only in a secure context), so the native-style sync filesystem API survives
  the port unchanged.
- `crates/frozen-duckdb-sys/` — unchanged FFI layer, compiled for
  `wasm32-...` targets. The vendored C API headers
  (`crates/frozen-duckdb-builder/vendored-headers/duckdb.h`, `duckdb.hpp`,
  staged from upstream v2.0-cyanoptera branch HEAD `b6013f7aa2f36caabb9466ce1791667ea8a280bb`,
  DuckDB `2.0.0-dev`) are the compile contract; bindgen over the same
  headers on wasm must yield a compatible bindings surface. UNVERIFIED.

Crash containment contract: main-thread state is reconstructible from OPFS
after any worker death (see falsifiers); the worker holds no authoritative
state that is not already in OPFS or derivable by re-opening the database.

## OPFS block-I/O layer design

DuckDB needs a `FileSystem` implementation over OPFS. The mapping:

- One `FileSystemFileHandle` per database file (`.db`, `.wal`, `.tmp`
  directory). On open, `createSyncAccessHandle()` yields a
  `FileSystemSyncAccessHandle` exposing `read(buf, {at})` /
  `write(buf, {at})` / `truncate(len)` / `getSize()` / `flush()` / `close()`
  — a byte-range API close enough to `pread`/`pwrite` that DuckDB's
  `FileSystem::Read`/`Write` map onto it directly. UNVERIFIED at real scale;
  per-call `at` offsets over OPFS have known throughput limits.
- Writes go through `flush()` at WAL commit boundaries only; relying on
  implicit flush is unspecified behavior per OPFS spec.
- **256MB memory cap.** The wasm linear memory is capped at 256MB
  (`MAXIMUM_MEMORY` / build-time `INITIAL_MEMORY` sizing under Emscripten;
  the equivalent knob under wasm32-wasi is `MALLOC`/`TOTAL_MEMORY` tuning —
  exact flag spelling UNVERIFIED). Buffer-manager blocks must be sized so the
  working set plus query intermediates fit under the cap; the block allocation
  size (`SET memory_limit='...'`) should be pinned to ~192MB, leaving headroom
  for the wasm stack, FFI buffers, and JS-side transfer buffers.
- **QuotaManager interactions.** OPFS quota is reported through the Storage
  QuotaManager (`navigator.storage.estimate()`), and `write()` on a sync
  access handle fails with `QuotaExceededError` once usage exceeds the
  origin's bucket quota. The layer must (a) query quota at open, (b) map
  `QuotaExceededError` to a typed out-of-disk error on the RPC surface (not a
  generic IO error), and (c) request persistent storage
  (`navigator.storage.persist()`) at open so eviction does not silently drop
  the database. UNVERIFIED: per-origin bucket quota sizes in current Chrome
  and whether `QuotaExceededError` is thrown or the write short-writes under
  quota pressure.
- WAL replay is the recovery path: on reopen after a crash, DuckDB replays
  the `.wal` through the same OPFS layer; no browser-specific recovery code
  is needed if the block layer is correct.

## Build matrix

The repo's native build knowledge (from `scripts/build_duckdb_optimized.sh`,
which pins `DUCKDB_VERSION="2.0.0-dev"` and pre-compiles heavy dependencies)
transfers to an Emscripten build:

| Target | Toolchain | Key flags | Status |
|---|---|---|---|
| Native (existing) | system CC | `scripts/build_duckdb_optimized.sh` | ALIVE |
| wasm32-wasi | wasi-sdk + DuckDB's own wasm support | DuckDB upstream has a `wasm` amal... platform | UNVERIFIED |
| Emscripten | emcmake/emmake | `DUCKDB_EXPLICIT_PLATFORM=wasm_eh` | UNVERIFIED |

`DUCKDB_EXPLICIT_PLATFORM=wasm_eh` is a *candidate* platform string observed
in upstream DuckDB's wasm build configuration; it is marked UNVERIFIED until a
real `emcmake` build against the vendored `b6013f7` headers produces a
`duckdb_wasm` artifact. Candidate flag set (all UNVERIFIED):

```bash
emcmake cmake -S duckdb -B build-wasm \
  -DCMAKE_BUILD_TYPE=Release \
  -DDUCKDB_EXPLICIT_PLATFORM=wasm_eh \
  -DDISABLE_UNITY=ON \
  -DBUILD_UNITTESTS=OFF \
  -DDUCKDB_BUILD_LIBRARY=ON \
  -DENABLE_SANITIZER=OFF \
  -DCMAKE_CXX_FLAGS="-sWASM_EH=1 -sALLOW_MEMORY_GROWTH=1 -sMAXIMUM_MEMORY=268435456"
```

The `wasm32-wasi` vs Emscripten choice: wasm32-wasi pairs with DuckDB's
upstream WASM support and keeps the sys-crate's wasi shims; Emscripten pairs
with `FileSystemSyncAccessHandle` via its own JS FS glue and `wasm_eh`
exception handling. Only one is chosen by the first real build; the matrix is
deliberately kept open until a build proves one path.

## Falsifiers

1. **Build falsifier.** A WASM build target must produce a `duckdb_wasm`
   artifact that passes the same `duckdb_library_version` gate the native
   build passes. Concretely: link the artifact into the sys-crate for a wasm
   target, call `duckdb_library_version()` from a wasm-side test, assert it
   reports the `2.0.0-dev` engine version matching the vendored `b6013f7`
   headers. A build that compiles but fails the version gate is FAIL.
2. **Persistence falsifier.** OPFS persistence must survive `Page.crash`
   plus reload via WAL replay. Concretely: via CDP (`Page.crash`), crash the
   tab after a committed write, reload, re-open the database from OPFS, and
   assert the committed row is present (replayed from the `.wal`).
   A reload test that only closes the tab cleanly does not exercise this
   falsifier; a `Page.crash` + reload that loses a committed row is FAIL.
3. **Quota falsifier.** (Supporting.) Fill OPFS to the origin's quota,
   attempt a write through the RPC surface, and assert a typed
   out-of-disk error — never a hang, silent short-write admitted as success,
   or worker wedge.

See Also: `docs/architecture/binary-management.md` ·
`scripts/build_duckdb_optimized.sh` ·
`crates/frozen-duckdb-builder/vendored-headers/duckdb.h` ·
`crates/frozen-duckdb-builder/src/lib.rs`
