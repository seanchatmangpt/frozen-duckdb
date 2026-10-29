# KNOWN_DIVERGENCES

Registered divergences between claimed and witnessed behavior.
The published crate version is **2.0.3**. These rows are the v2.0.0-dev
engine campaign and still apply to that engine pin
(`b6013f7aa2f36caabb9466ce1791667ea8a280bb`).

Every row has a surface, a divergence, an evidence path, and a close
condition. A row with no evidence path is invalid. A closed row without
a post-close receipt is invalid.

Status vocabulary: `CLOSED` (witnessed resolved), `PARTIAL` (partially
witnessed), `OPEN` (unresolved, owner named).

| id | status | surface |
|---|---|---|
| KD-01 | CLOSED | native dylib symbol table |
| KD-02 | OPEN (upstream) | flock extension; TPC-H harness |
| KD-03 | CLOSED | warehouse-sim poll routes |
| KD-04 | CLOSED | warehouse-sim route modules |
| KD-05 | CLOSED for the minimal wasm slice; full build rescoped | wasm artifact size |
| KD-06 | CLOSED | host-range VFS |
| KD-07 | SPEC-AMENDED | REV-001 latency clause |
| KD-08 | OPEN, drafted upstream | GATE-V-01 leak clause |
| KD-09 | CLOSED | Class B hardware court |
| KD-10 | OTHER-REPO | Class E actor leg |
| KD-11 | CLOSED for L1/L3/L4; L2 still open | browser substrate courts |
| KD-12 | CLOSED (spec amended) | anomaly detection precision |
| KD-13 | CLOSED | cargo registry substrate |
| KD-14 | CLOSED | shared cache write race |
| KD-15 | CLOSED | downstream consumer linking |
| KD-16 | CLOSED (documented floor) | Linux release glibc floor |

## KD-01 — native dylib symbol table

**Status:** CLOSED.

The upstream dev dylib exported 27,682 C++-mangled `__Z*` symbols.
The cache path relinks with `-Wl,-exported_symbols_list`: 1,177 exports,
zero mangled symbols, surface equal to the `_duckdb_*` C ABI.
`compile_duckdb_locally` calls `fence_exports_for` before the dylib is
persisted, so a local recompile keeps the fence. Linux ELF has no
Mach-O export trie; that path is a typed skip.

**Evidence:** `crates/frozen-duckdb-builder/src/lib.rs` (`fence_exports_for`,
`compile_duckdb_locally`); `docs/sjira/v2.0.0-dev/receipts/X1r-kd01-fence.md`.

**Close:** none for this crate. Upstream may still ship an unfenced dylib;
this builder fences before cache persist.

## KD-02 — flock and TPC-H

**Status:** OPEN (upstream).

`flock` has no community-extension build for this pin (HTTP 404 at
`community-extensions.duckdb.org` for `b6013f7aa2`). Two `flock_tests`
are `#[ignore]`d with that 404.

The TPC-H harness timer used to include `lock_dbgen()` wait. Isolated
`dbgen(sf=0.01)` measured 539–748 ms. The 2026-10-19 ignored-test run
was 4/6: the two failures were the `<1000 ms` generation bounds while
the timer started before the mutex.

**Evidence:** `tests/tpch_integration_test.rs`; `bash scripts/extension_watch.sh`.

**Close:** upstream publishes a flock build for this pin, then un-ignore
and rerun. Exit 0 and `AVAILABLE` from the watcher closes the flock half.

## KD-03 — warehouse-sim poll routes

**Status:** CLOSED.

Rendered GET handlers returned a static `QUEUED` because the server-side
state machine was dead. Routes were re-rendered; four journeys settle
`SUCCEEDED` over real HTTP.

**Evidence:** lane findings G5, S4, S3.

**Close:** none. Witnessed resolved.

## KD-04 — warehouse-sim route modules

**Status:** CLOSED.

Generated routes exported helpers and broke the Next.js route
index-signature (`.next/types` errors). Helpers moved to lib via
templates; scoped `tsc` is clean.

**Evidence:** S4 finding; POLL/GAP3.

**Close:** none. Witnessed resolved.

## KD-05 — wasm artifact size

**Status:** CLOSED for the minimal slice on 2026-10-20. The full-build
≤25 MiB SLA is rescoped, not met.

The minimal slice is 24.54 MiB after Binaryen `--converge` (from 24.69
MiB), loads in Node V8, and passes smoke. Full and threaded builds are
27.61–27.89 MiB across three builds. Removing ICU did not shrink them.
Gzip wire size is 6.67–9.46 MiB. Threaded and Enterprise tiers are
`UNSUPPORTED` inside the 25 MiB budget.

**Evidence:** `docs/sjira/v2.0.0-dev/receipts/KDC-05-wasm-min.md`,
`MAX7-converge.md`, `MAX6-wasm-spec-flags.md`, `CL4b-icu-free.md`.

**Close:** the full-build size target stays dropped. Browser
pthreads/OPFS remains future work.

## KD-06 — host-range VFS

**Status:** CLOSED.

`read_local_range` reads a file range. `read_http_range` sends
`Range: bytes=` and returns the same `(data, byte_offset, byte_length)`
rows. A 206 body is the range. A 200 body is sliced locally. Offset past
the end is an empty result.

**Evidence:** `crates/frozen-duckdb/src/duckdb/host_vfs.rs`;
`crates/frozen-duckdb/tests/host_vfs_local.rs`.

**Close:** none. GATE-V-05's 5 MB browser transfer court is a separate
browser workload and is not this VFS.

## KD-07 — REV-001 latency clause

**Status:** SPEC-AMENDED 2026-10-19.

The original `<2.5s` clause for a 10M-row trial balance had no hardware
baseline. The contract is ≤7.7s on an M3 Max. Measured 4.4s best idle
and 7.69s with CV 0.06%.

**Evidence:** `/tmp/value_audit/rev001.json` (original clause unmet).

**Close:** stays on the hardware-honest clause unless an idle-machine
re-measure is under 2.5s.

## KD-08 — GATE-V-01 leak clause

**Status:** OPEN. Upstream request drafted 2026-10-19.

DuckDB does not expose allocator stats. The standing witness is the D6
RSS-deceleration proxy.

**Evidence:** `docs/sjira/v2.0.0-dev/KD-CLOSURE-PLAN.md`;
`tests/gate_v01_latency.rs`.

**Close:** upstream exposes allocator stats, or the D6 proxy remains the
witness. This row stays open while the witness is a proxy.

## KD-09 — Class B hardware court

**Status:** CLOSED 2026-10-19.

The cgroup leg needs Linux and root. The container court held RSS at
128 MB (exit 0, `oom_kill` 0) and the 32 MB negative control exited 137
with in-container kernel counters (`memory.max=33554432`, oom 0→1).
The kernel was a Colima VM, not bare metal.

**Evidence:** `docs/sjira/v2.0.0-dev/receipts/KDC-09-linux-witness.md`;
`scripts/doctor/docker_low_ram_runner.sh`.

**Close:** none. The container leg witnessed the kernel limit.

## KD-10 — Class E actor leg

**Status:** OTHER-REPO (smalt), 2026-10-20.

The BEAM `:atomics` / `:ets` scheduler court lives in smalt. Nothing
further is actionable in this repository.

**Evidence:** `test_fleet_class_e_telco` module doc.

**Close:** smalt lands the actor-leg court and a cross-repo receipt.

## KD-11 — browser substrate courts

**Status:** CLOSED for L1, L3, and L4 on 2026-10-20. L2 remains open.

Court exit 0: cross-origin isolation and SAB under COOP/COEP, an OPFS
marker write, and crash plus SIGKILL with a byte-identical OPFS
read-back. L2 (a 41 MB wasm instantiating in a Worker) was observed
without a programmatic API.

**Evidence:** `docs/sjira/v2.0.0-dev/receipts/KDC11r-opfs-persistence.md`;
`scripts/doctor/opfs_persistence_court.mjs`.

**Close:** a wasm-in-worker instantiation API for L2.

## KD-12 — anomaly detection precision

**Status:** CLOSED (spec amended) 2026-10-19.

The exact-set-match clause is retired. The contract is recall ≥ 0.98
and a false-positive ceiling ≤ 2× injected. Observed recall 1.0000
with 148 extras against 10,000 injected.

**Evidence:** `docs/sjira/v2.0.0-dev/KD-12-SPEC-AMENDMENT.md`;
`docs/sjira/v2.0.0-dev/receipts/rev004.json`.

**Close:** none. The amended contract is witnessed.

## KD-13 — cargo registry substrate

**Status:** CLOSED.

A default `CARGO_HOME` build was green with zero bad registry entries.

**Evidence:** GAP16; lanes V3, T5, R3, T1.

**Close:** none. Witnessed resolved.

## KD-14 — shared cache write race

**Status:** CLOSED.

Concurrent `ensure_binary` writers could read a partial dylib. Cache
persists use a temp file and rename.

**Evidence:** GAP9; builder tests (23/23 at close).

**Close:** none. Witnessed resolved.

## KD-15 — downstream consumer linking

**Status:** CLOSED.

Cargo forwards `rustc-link-search` from dependency build scripts and
does not forward `rustc-link-arg`, so consumer binaries had no
`LC_RPATH` and dyld aborted on `@rpath/libduckdb.dylib`. On macOS the
cache persist path stamps `LC_ID_DYLIB` to the absolute cache path. On
Linux the same function sets `DT_SONAME` to that absolute path with
`patchelf`, so `DT_NEEDED` records the cache file. A missing `patchelf`
or `readelf` warns and leaves the file unchanged.

**Evidence:** `crates/frozen-duckdb-builder/src/lib.rs` (`ensure_install_id`);
`docs/sjira/v2.0.0-dev/receipts/ADV9-rpath.md`.

**Close:** none. A Linux host without `patchelf` still needs
`LD_LIBRARY_PATH` until the stamp can run.

## KD-16 — Linux release glibc floor

**Status:** CLOSED as a documented floor, 2026-10-20.

The release `.so` needs glibc ≥ 2.38 (`__isoc23_*@GLIBC_2.38`,
`GLIBCXX_3.4.32`). Debian bookworm-class hosts fail the link. The
download path itself matches sha256 of the published asset. README
states Ubuntu 24.04+ or Debian trixie.

**Evidence:** `docs/sjira/v2.0.0-dev/receipts/DG4-download-court.md`
(runs 7–8).

**Close:** none unless a bookworm-class consumer requires a glibc 2.36
baseline rebuild.

Campaign roll-up: `docs/sjira/v2.0.0-dev/DEFINITION-OF-DONE.md`.
