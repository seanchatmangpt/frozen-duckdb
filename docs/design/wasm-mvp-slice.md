# WASM MVP Slice — Minimal lib-only build + 20-line JS smoke

Design for the 80/20 alternative to GAP1b's full build. Status: design
(UNBUILT). Effort: S. Effort vs full build (GAP1b, in-flight): L.

Grounding (verified on disk 2026-10-19):

- `scripts/build_wasm_duckdb.sh` line 25: `SOURCE_PIN="d54cbbe3df3…"`
- `target/wasm-build/duckdb` HEAD = `d54cbbe3df17e8e5ce68312f483b4155836112aa`
- `git describe` in that tree: `v1.5.6-17220-gd54cbbe3df`
- GAP1b full build artifacts preserved at
  `target/wasm-build/build/` (`duckdb.wasm`, `duckdb.js`, CMakeCache).
  The minimal config reuses the same source checkout and its own build
  dir; incremental rebuild is possible but the flag delta is large
  enough (extension set, pthreads) that a fresh build dir is safer.
  Preserve `target/wasm-build/build` untouched (KD-05 evidence).

## 1. What the slice unlocks (the 80%)

One artifact — a minimal `duckdb.wasm` that instantiates and answers
`duckdb_library_version()` — flips four claims:

- FR-01: BLOCKED -> PARTIAL (artifact exists, instantiates in a real
  JS runtime, version gate green).
- KD-05: PARTIAL_ALIVE -> PARTIAL (pthread ABI risk removed: no
  pthreads, no sem_timedwait link hazard; size ceiling now judged
  against a min-config artifact, not the 39.2MiB full build).
- KD-11: unblocked to start (CDP/OPFS harness has a real wasm subject
  to load; still blocked at its OPFS legs until the full build).
- TEST-FLEET-001 wasm leg: runnable — zero-network + memory-cap
  assertions on real wasm instead of an FR-01-blocked skip.

What it does NOT claim: no OPFS (needs pthreads + full build), no full
SQL court (SELECT 42 / PRAGMA version are the ceiling here), no size
ceiling pass (KD-05's 25MB ceiling is judged on the full build; the
min build only shrinks the measured subject). Those remain with GAP1b.

## 2. Exact cmake flag diff vs scripts/build_wasm_duckdb.sh

Baseline: script lines 90-99 (emcmake configure block). Delta:

| flag | full build (script) | minimal slice |
|---|---|---|
| CMAKE_CXX_FLAGS | `-O3 -flto -sUSE_PTHREADS=1` | `-O3` |
| BUILD_EXTENSIONS | `parquet;json` | `""` (none) |
| BUILD_SHELL | default (on) | `OFF` |
| BUILD_UNITTESTS | default (on) | `OFF` |
| BUILD_HTTPFS | OFF (unchanged) | OFF |
| AUTOLOAD/AUTOINSTALL | OFF (unchanged) | OFF |
| DUCKDB_EXPLICIT_PLATFORM | wasm_eh (unchanged) | wasm_eh |
| EXPORTED_FUNCTIONS | not set | `["_duckdb_library_version","_duckdb_free","_duckdb_open","_duckdb_close"]` |
| build target | default (all) | `duckdb` (lib target only) |

Notes:

- Dropping `-sUSE_PTHREADS=1` is the load-bearing change: it removes
  the pthread ABI signature-mismatch hazard KD-05 diagnosed on the
  unittest/LTO link, and makes SharedArrayBuffer/COOP-COEP headers
  unnecessary for instantiation — the smoke runs on plain
  `file://`-free node or a bare page, no cross-origin isolation.
- `-flto` dropped only because the slice does not build unittests (the
  KD-05 failure was unittest LTO + pthreads); the lib link keeps -O3.
  Keep `-flto` if a size reduction is wanted; it is not load-bearing.
- EXPORTED_FUNCTIONS must be a single emcc-style list. Because the
  flags flow through `CMAKE_CXX_FLAGS`, the export list goes on the
  link step: `emcc` final-link flag
  `-sEXPORTED_FUNCTIONS=<list>` via
  `-DCMAKE_EXE_LINKER_FLAGS` / `-DCMAKE_SHARED_LINKER_FLAGS`, or
  (preferred) rely on DuckDB's own export list and add a 1-line
  `emsymbol()`-free extra: `--no-entry` is NOT set; exports are added
  via `CMAKE_CXX_FLAGS` append
  `-sEXPORTED_FUNCTIONS=[...]` — verify at build time that
  `WebAssembly.Module.exports()` lists `duckdb_library_version`, and
  fail the build receipt if absent (loud, not silent).
  VZT note: CMakeCXXFlags propagation of `-s*` to the final link is a
  known hazard; the smoke IS the court — if the export is missing the
  smoke fails loudly.
- DuckDB version string: derived from git describe in the checkout
  (v1.5.6-17220-gd54cbbe3df lineage) rendered as the dev version
  GAP1b measured as `v2.0.0` in /tmp/gap1b/witness.out.
  The smoke pins the exact expected string captured from the real
  build receipt, not a hand-written guess.

## 3. The smoke (acceptance criteria, ~20 lines)

A single `smoke.mjs` run under node (plain node, no flags needed
without pthreads) that:

1. Instantiates `duckdb.wasm` (raw `WebAssembly`, no JS glue needed
   for a version probe; duckdb.js glue optional). Raw WebAssembly
   instantiation of a 20MB+ lib requires explicit memory sizing —
   the module carries its own memory; no flags needed. Fail loud on
   instantiation throw.
2. Calls exported `duckdb_library_version()` and reads the returned
   heap pointer via `Memory` view.
3. Asserts the version string equals the string recorded in the build
   receipt for this exact artifact sha256.
4. Heap ceiling probe: records `WebAssembly.Memory.buffer.byteLength`
   before and after the version call plus a
   `duckdb_open`/`duckdb_close` cycle (no SQL executed). Asserts
   post-run heap <= ceiling (proposal: 64MB start, assert <= 128MB
   after — numbers pinned from the first real run, not invented).

Acceptance (all must hold, else exit 1):

- `WebAssembly.Module.exports()` includes `duckdb_library_version`
  (checked before instantiation; a missing export fails loudly).
- version string == receipt-pinned string (expected lineage
  `v2.0.0…` from b6013f7).
- heap ceiling: `buffer.byteLength <= 128 * 1024 * 1024` after the
  open/close cycle; also assert delta-vs-before <= 64MB so a silent
  growth regression trips the court.
- exit code 0 only if all four checks pass; any throw = exit 1.

## 4. Falsifier

The slice's claim is falsified if any of:

- The minimal configure+build fails where the full build succeeded
  (e.g. empty BUILD_EXTENSIONS unsupported upstream at b6013f7).
- The export list does not survive CMake flag propagation and the
  smoke cannot find `duckdb_library_version` in
  `WebAssembly.Module.exports()`.
- The version string mismatches the receipt-pinned value.
- Heap ceiling exceeded or delta regression.

Any of these = the slice did not land; KD-05 stays PARTIAL_ALIVE
(GAP1b route), TEST-FLEET-001 wasm leg stays skipped loud.

## 5. Effort + build mechanics

- Effort S: no new source, no new checkout — reuse
  `target/wasm-build/duckdb` (HEAD b6013f7, pin-verified).
- Fresh build dir `target/wasm-build/build-min` so the preserved GAP1b
  tree stays untouched as KD-05 evidence.
- One minimal-build script (or a MODE=min env branch in
  `scripts/build_wasm_duckdb.sh` guarded so the full-build path is
  byte-identical when unset — GAP1b owns that path).
- Receipt block mirrors the script's WASM BUILD RECEIPT (bytes,
  sha256, emcc version, flag set, smoke output embedded).

## 6. Boundary claims (no overclaim)

- The slice does NOT close FR-01 (that needs OPFS + full SQL courts
  on the full build; GAP1b route).
- The slice does NOT close KD-05 (size ceiling is judged on the full
  build; the min artifact only proves a smaller subject exists).
- The slice does NOT satisfy KD-11's OPFS legs (CDP harness can
  start; OPFS courts wait for the full build).
