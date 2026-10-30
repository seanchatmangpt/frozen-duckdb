# host_range_vfs_test.md — KD-06 mechanism-slice receipt (lane X4)

Standing: **PARTIAL_ALIVE** (mechanism slice only). The full GATE-V-05
remote-parquet court is **OPEN** — no query has run through this VFS, no
callback has been invoked at runtime, and no wasm boundary leg exists.

## What was built

- `c_api/host_range_vfs.h` — `hostvfs::HostRangeFileSystem : duckdb::FileSystem`
  overriding OpenFile / Read (positional + sequential) / Write (both, always
  InvalidInput) / GetFileSize / FileExists / CanHandleFile / Glob / GetName /
  CanSeek, plus `hostvfs::HostRangeFileHandle : duckdb::FileHandle`.
  Dispatches every read to the contract callback
  `read_fn(void* ctx, const char* path, uint64_t offset, uint64_t length, uint8_t* out_buf) -> Status`
  with the closed-set status vocabulary from
  `docs/design/host-range-vfs-contract.md` (Success / InvalidInput /
  OutOfRange / NetworkDenied / QuotaExceeded), mapped to DuckDB exceptions in
  `DispatchRead` — never a silent-zero return.
- `c_api/host_range_vfs.cpp` — implementation +
  `RegisterHostRangeFactory` correction: the factory is
  `RegisterHostRangeFileSystem(duckdb::DuckDB&, unique_ptr<HostRangeFileSystem>)`,
  wiring the subsystem into `db.GetFileSystem().RegisterSubSystem(...)`.
- `c_api/host_range_vfs_compile_proof.cpp` — compile-proof TU that includes
  the vendored `duckdb.hpp` + the slice header and instantiates the class, so
  the whole override table is type-checked, not just parsed.

## Compile command (authoritative, exit 0, run this session)

Compiler: Apple clang 17.0.0 (clang-1700.6.3.2), macOS (arm64).

```bash
cd /Users/sac/frozen-duckdb/c_api
clang++ -std=c++17 -fsyntax-only \
  -I/Users/sac/frozen-duckdb/crates/frozen-duckdb-builder/vendored-headers \
  -I/Users/sac/frozen-duckdb/target/wasm-build/duckdb/src/include \
  -I. \
  host_range_vfs.cpp host_range_vfs_compile_proof.cpp
# exit 0 — both TUs
```

A real object compile also exited 0 for both TUs:

```bash
clang++ -std=c++17 -c -o /tmp/hrv.o -I<vendored-headers> -I<wasm-include> -I. host_range_vfs.cpp
clang++ -std=c++17 -c -o /tmp/hrv_proof.o -I<vendored-headers> -I<wasm-include> -I. host_range_vfs_compile_proof.cpp
```

## Non-vacuity (shim-court flip, this session)

Baseline sha256(host_range_vfs.h) = 92f11728b1bbbfe7e41f97615e0e9a16d7b50a5ed06e66d8e135637a9018533b.
SHIM-COURT mutation (break the Read override signature) made the compile
proof exit NONZERO — the gate flips, so it is not vacuous. Restore sha256
was byte-identical to baseline (fail-closed restore held). The header was
later comment-only edited (stale comment fix); current header sha256 =
6b2bd207552c4e7d6f790141be3fa842b805ca9ce5772ca0416ea46672ebd9f2, still
compiles GREEN. File hashes at receipt time:

```
6b2bd207552c4e7d6f790141be3fa842b805ca9ce5772ca0416ea46672ebd9f2  host_range_vfs.h
915941947793a002a2573f736e0a1bf0049358b067f8338bf14382a4b366d43e  host_range_vfs.cpp
3238c4601fe9fe1380f4a7bb5a67eb5f28cfab4bbc768ddbbe56d3d32ab11104  host_range_vfs_compile_proof.cpp
```

## What needs the runtime to prove (open boundary — honest limits)

- No runtime execution exists. OpenFile/Read/GetFileSize have never run.
- No query has been served through the registered subsystem (no
  `read_parquet('host-range://...')` witness). The scanner's footer-first /
  dict-page / lazy-data-page access pattern is assumed, not witnessed.
- The registration factory is type-checked only, never linked.
  `RegisterSubSystem` is a DUCKDB_API import; linking needs a native libduckdb
  1.5.5 archive. The wasm tree archive is wasm32 and cannot link a native
  object — the native receipt needs a native 1.5.5 build with identical
  headers.
- Idempotent re-registration (`InvalidInput` on duplicate prefix) is declared
  out of scope for the slice; the factory is register-once.
- `ResolveSize` with `size_fn == nullptr` throws loudly rather than faking a
  size. The contract's "1-byte range probe" size resolution needs the host to
  report object size through the probe response, which the bare `read_fn`
  signature cannot carry — contract revision needed (prefix bump to
  `host-range/v2://`) or `size_fn` becomes mandatory. Registered divergence,
  not a silent skip.
- QuotaExceeded maps to a prefixed IOException ("host-range: quota exceeded"),
  not a dedicated C++ exception type — DuckDB's exception set has no quota
  row; nearest-row mapping per the closed-set law, host pattern-matches the
  prefix.
- GATE-V-05 remote-parquet court: OPEN. The ≤5 MB budget court, the
  200-instead-of-206 shim flip, and the credential-leak falsifier are all
  unrun. This file is a mechanism slice receipt, not a gate receipt.

## See Also

- `docs/design/host-range-vfs-contract.md` (contract)
- `docs/design/host-call-vfs.md` (architecture)
- `docs/design/gate-v05-hostvfs-transfer.md` (acceptance court)
- `docs/sjira/v2.0.0/KD-CLOSURE-PLAN.md` (KD-06 row)
