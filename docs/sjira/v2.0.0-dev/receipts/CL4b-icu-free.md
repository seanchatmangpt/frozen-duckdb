# CL4b — icu-free DISABLE_THREADS full wasm (KD-05 size lever test)

Standing: **BUILD_ALIVE / SMOKE_ALIVE / SIZE_FLOOR_UNCHANGED** — the icu lever is dead:
removing icu does not shrink the artifact, because icu was never linked into it.
Lane CL4b, repo /Users/sac/frozen-duckdb, 2026-10-19. No git operations.

## Mechanism found (upstream)

`target/wasm-build/duckdb/extension/extension_config.cmake:17-18`:

```
duckdb_extension_load(icu)
duckdb_extension_statically_link(icu)
```

## Tree edit (pinned build artifact, edited in place per lane contract)

```
$ diff /tmp/extcfg.orig extension/extension_config.cmake
17,18d16
< duckdb_extension_load(icu)
< duckdb_extension_statically_link(icu)
```

(diff exit 1 = lines removed). The edit stands in
`target/wasm-build/duckdb/extension/extension_config.cmake`; original preserved at
`/tmp/extcfg.orig`. All other flags identical to CL4.

## Configure (emcmake) — exit 0

```
emcmake cmake -S target/wasm-build/duckdb -B target/wasm-full-nt-icu \
  -DCMAKE_CXX_STANDARD=17 -DCMAKE_CXX_FLAGS="-O3 -flto" \
  -DDUCKDB_EXPLICIT_PLATFORM=wasm_eh -DBUILD_EXTENSIONS="parquet;json" \
  -DENABLE_EXTENSION_AUTOLOADING=OFF -DENABLE_EXTENSION_AUTOINSTALL=OFF \
  -DBUILD_HTTPFS=OFF -DDISABLE_THREADS=ON -DBUILD_SHELL=OFF \
  -DBUILD_UNITTESTS=OFF -DCMAKE_BUILD_TYPE=Release
```

Configure log: `target/wasm-full-nt-icu-configure.log` (same two unused-var warnings
as CL4: ENABLE_EXTENSION_AUTOINSTALL/AUTOLOADING).

## Build

`cmake --build target/wasm-full-nt-icu -j` → **exit 0** (`build_exit.txt`), final line
`[100%] Built target duckdb_local_extension_repo` (`build.log`). Note: first run was
killed at the 600 s background cap mid-link ("Terminated: 15"); the incremental resume
completed clean. icu compile targets absent from the build graph (0 `icu` entries vs
CL4's `[51%] Building ... icu_loadable_extension` lines).

## Final link (same em++ line as CL4) — exit 0

```
em++ -O3 -flto -sWASM=1 -sSTANDALONE_WASM=1 -sALLOW_MEMORY_GROWTH=1 \
  -sEXPORTED_FUNCTIONS=_duckdb_library_version,_duckdb_open,_duckdb_close,_duckdb_query,_duckdb_connect,_duckdb_disconnect \
  -sENVIRONMENT=node -sTOTAL_MEMORY=268435456 --no-entry \
  -o duckdb_nt.js src/libduckdb_static.a tools/cpp/libduckdb_cpp_api.a third_party/*/*.a
```

## Smoke (node, CL4's smoke.mjs — raw WASI instantiation)

```
INSTANTIATE=OK
VERSION=v2.0.0-dev88595
```
smoke exit 0. Version exact-match PASS.

## Size delta table (the honest result)

| artifact | bytes | MiB | vs SLA 26,214,400 B (25 MiB) | node smoke |
|---|---|---|---|---|
| CL4 `duckdb_nt.wasm` (with icu in build graph) | 28,946,095 | 27.61 | +2,731,695 (+10.4%) | PASS |
| **CL4b `duckdb_nt.wasm` (icu-free build graph)** | **28,946,107** | **27.61** | **+2,731,707 (+10.4%)** | PASS |
| delta | **+12 bytes (noise)** | — | — | — |
| SLA | 26,214,400 | 25.00 | — | — |

sha256 `duckdb_nt.wasm`:
`dae5947ef6961f654789fda423983de4377bf2fc77b93e5d6773978fbdc2a10b`

## Why: icu never reached the artifact

- `ar t src/libduckdb_static.a | grep -ci icu` → **0** in BOTH CL4 and CL4b archives.
- The icu objects land in a separate archive
  `extension/icu/libicu_extension.a`, which the CL4/CL4b final em++ link line never
  passes (only `src/libduckdb_static.a`, `tools/cpp/libduckdb_cpp_api.a`,
  `third_party/*/*.a`).
- Symbol proof on the linked artifact: `emnm duckdb_nt.wasm | grep -ci icu` → **0**.
  (`strings` hits are only core metadata: the extension-catalog description string
  "Adds support for time zones and collations using the ICU library" and unrelated
  PROJ/CRS JSON fixtures — no icu code symbols.)

CL4's static-link of icu therefore described the compile graph, not the artifact: the
extension was built but never symbol-reachable in the standalone link.

## Verdict for KD-05

- icu is a **documented non-factor**, not a usable exclusion: the icu-free rebuild is
  byte-for-byte the same size class as CL4 (27.61 MiB). KD-05's full-build size SLA has
  **no witnessed path via this lever**; the honest icu-free floor equals CL4's:
  **28,946,107 bytes (27.61 MiB)**, over SLA by 10.4%.
- Real remaining levers (outside this lane): aggressive dead-code stripping of core
  (the mass is in core + third_party, not icu), or renegotiating the 25 MiB SLA
  against this artifact class.
