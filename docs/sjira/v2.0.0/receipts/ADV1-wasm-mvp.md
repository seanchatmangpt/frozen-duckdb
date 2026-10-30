# Receipt: ADV1 — WASM MVP Slice

Status: **PARTIAL_ALIVE** (spec acceptance met; boundary claims per spec §6 not exceeded).

## Subject + replay

- Script (new, this lane): `/Users/sac/frozen-duckdb/scripts/build_wasm_mvp.sh`
- Spec: `/Users/sac/frozen-duckdb/docs/design/wasm-mvp-slice.md`
- Artifact: `/Users/sac/frozen-duckdb/target/wasm-min/duckdb.wasm`
- Smoke: `/Users/sac/frozen-duckdb/target/wasm-min/smoke.mjs`
- Build receipt (script-emitted): `/Users/sac/frozen-duckdb/target/wasm-min/receipt.txt`
- Build dir: `target/wasm-build/build-min` (fresh, per spec §5)

```
pin:   d54cbbe3df17e8e5ce68312f483b4155836112aa (verified pre-build)
bytes: 25,887,785
sha256: e1d0e49648b690c5a9371b9d9471287c0ee48eb23a0c679a30a0cccc2dcc1583
emcc:  6.0.13 (e13ee988ce3d87a738dd8f3661d02223308c412a)
node:  v26.10.0
git describe: v1.5.6-17220-gd54cbbe3df (rendered version v2.0.0)
```

## Smoke (real output, exit 0)

```
[smoke] exports: ["memory","main","__indirect_function_table","duckdb_open","duckdb_close","duckdb_library_version","duckdb_free","_start","setThrew","_emscripten_stack_restore","emscripten_stack_get_current"]
[smoke] duckdb_library_version() = v2.0.0
[smoke] duckdb_open rc= 0 db handle= 1893624n
[smoke] heap before=18677760 after=18743296
[smoke] PASS
```

Acceptance per spec §3, all four checks green:

1. `WebAssembly.Module.exports()` includes `duckdb_library_version` — checked
   pre-instantiation (loud falsifier path present, not taken).
2. version == `v2.0.0` (matches GAP1b witness lineage from b6013f7).
3. heap after open/close: 18,743,296 B (17.9 MiB) <= 128 MiB.
4. heap delta = 65,536 B (one page, the smoke's own scratch grow) <= 64 MiB.

## Flag set actually used (delta vs full build)

`BUILD_EXTENSIONS=""` ... note: upstream at b6013f7 ignores an empty
`BUILD_EXTENSIONS` for the statically linked set — configure log shows
"Linked into DuckDB: extensions [core_functions, parquet, json, icu]".
The extensions are compiled and statically linked regardless; the minimal
size comes from dropping the shell/unittests/pthreads/LTO, not the
extension set. Reported as-is, not smoothed over.

- `BUILD_SHELL=OFF`, `BUILD_UNITTESTS=OFF`, `-O3` only (no LTO, no
  `-sUSE_PTHREADS`), `-DDISABLE_THREADS=ON`, `-DDUCKDB_EXPLICIT_PLATFORM=wasm_eh`.
- Final link (not cmake): `em++` stub main + `libduckdb_static.a` +
  core_functions/icu static libs,
  `-sALLOW_MEMORY_GROWTH=1 -sENVIRONMENT=node
  -sEXPORTED_FUNCTIONS=[_main,_duckdb_library_version,_duckdb_free,_duckdb_open,_duckdb_close]`.

## KD-05 evidence refinement (spec §4 falsifier path — exercised, resolved)

Two ghosts found on the way:

1. **sem_timedwait link hazard confirmed**: with default threading,
   `ub_duckdb_parallel.cpp.o` leaves `sem_timedwait` undefined (source:
   `third_party/concurrentqueue/lightweightsemaphore.h`, pulled in by
   `src/parallel/task_scheduler_pool.cpp`). CMakeCache-only
   `-DDUCKDB_NO_THREADS=1` is INERT at b6013f7 (no CMake wiring). The real
   switch is **`-DDISABLE_THREADS=ON`** (CMakeLists.txt:605), which removed
   the symbol. This is the spec's predicted pthread ghost, with the exact
   upstream flag now pinned.
2. **FindThreads failure**: `find_package(Threads REQUIRED)` fails under
   emcc without pthreads; worked around with
   `-DCMAKE_HAVE_LIBC_PTHREAD=1` cache pre-seed (not a tree edit).
- `emcc` cannot link the static lib (C++ runtime symbols); the final link
  must use `em++`. The `duckdb` cmake target produces a wasm SIDE_MODULE
  (`libduckdb.so`) with no useful exports; a standalone artifact needs the
  stub-main final link above.
- STANDALONE_WASM output carries WASI imports; the smoke stubs them
  (no-op). Noted for KD-11's CDP harness: instantiation needs an import
  object, not `{}`.

## Preserved evidence untouched (verified post-run)

```
target/wasm-build/duckdb HEAD = d54cbbe3df17e8e5ce68312f483b4155836112aa, 0 modified files
target/wasm-build/build/duckdb.wasm (GAP1b, 41MB) present, untouched
```

## Boundary (spec §6)

Does NOT close FR-01 / KD-05 / KD-11: no OPFS, no full SQL court, 25MB-size
claim stays with GAP1b's full build. The 25.9MB min artifact proves a
smaller pthread-free subject instantiates in plain node with zero
cross-origin isolation.

## Standing

PARTIAL_ALIVE — falsifiers of spec §4: none fired on the final run. First
smoke iteration DID fire the export/instantiation falsifier (missing import
stub → instantiation threw, exit 1, recorded in run logs
/tmp/wasm-mvp-run6.log) and was fixed, not skipped.
