# REL4r — Linux x86_64 linking court (HONEST PARTIAL — link not reached)

Standing: BLOCKED-PARTIAL (compile in progress, timebox expired)
Date: 2026-10-19
Lane: REL4r (replaces stalled REL4)
Pin: duckdb b6013f7aa2f36caabb9466ce1791667ea8a280bb (v2.0.0-dev88595)

## Inspection (prior-lane artifacts)
- `target/wasm-*` libduckdb_static.a — wasm32, not linux ELF. Not reusable.
- docker `focused_gates` (rel4.sh): linux/aarch64 build, /build/duckdb — wrong arch.
- docker `rel6-duckdb-build`: linux/aarch64, make -j1 in /out/src/build — wrong arch.
- NO linux x86_64 libduckdb artifact existed at lane start. Nothing to reuse.

## What was done
1. docker cp'd the pinned source out of rel6-duckdb-build:/out/src
   (host clone refused by topology guard; docker cp is not a git transition).
   Pruned .git from the copy. Path: /Users/sac/rel4r/duckdb-src (1.1 GB).
2. Launched docker amd64 container (Rosetta):
   docker run -d --name rel4r-amd64 --platform linux/amd64 \
     -v /Users/sac/rel4r:/work rust:1-bookworm bash -c
     'apt-get install cmake; bash /work/build.sh'
   build.sh: cmake -S /work/duckdb-src -B /tmp/build -DCMAKE_BUILD_TYPE=Release
     -DDISABLE_THREADS=1 -DBUILD_UNITTESTS=0 -DBUILD_BENCHMARKS=0 -DBUILD_SHELL=0
     -DJEMALLOC=1 && make -j$(nproc) duckdb
   Flags mirror the low-RAM config used by the aarch64 lanes.
3. Build in progress at timebox expiry. Last real log line (out/rel4r-build.log):
   [ 32%] Building CXX object src/function/window/CMakeFiles/duckdb_func_window.dir/ub_duckdb_func_window.cpp.o
   Progress: 156 translation units compiled, 38 min elapsed (Rosetta x86_64 emu).

## Link court NOT executed
No x86_64 libduckdb.a exists yet; duckdb_open/duckdb_library_version
(duckdb_library_version, expect v2.0.0-dev88595) was therefore not run. No
inference, no simulated pass. Expected version not observed.

## Resume
- Container `rel4r-amd64` left Up — make is incremental; rerun
  `docker start rel4r-amd64` if stopped, or exec the same make -j duckdb.
- Log: /Users/sac/rel4r/out/rel4r-build.log
- On completion: link C main against /tmp/build/libduckdb_static.a inside the
  amd64 container, run duckdb_open(":memory:"), print duckdb_library_version(),
  expect v2.0.0-dev88595; append output to this receipt.

## Falsifier
duckdb_library_version() inside the amd64 container prints v2.0.0-dev88595
and duckdb_open returns DUCKDB_SUCCESS on a binary linked against the
container-compiled libduckdb_static.a.
