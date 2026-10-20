# CL13 — KD-06 first slice: host-VFS local filesystem (80/20)

- Date: 2026-10-19
- Lane: CL13 (implement)
- Contract: `docs/design/host-range-vfs-contract.md` (ARD §3.2.A)
- Status: **ALIVE** (local slice) — ran + passed this session

## What landed

1. `crates/frozen-duckdb/src/duckdb/host_vfs.rs` (new): table function
   `read_local_range(file, offset, length)` backed by `std::fs::File`
   (`open` / `read_at` pread loop / `metadata` size), implementing the
   contract's open/read/size callback analog against a local file.
   Typed status vocabulary `HostRangeStatus::{InvalidInput, OutOfRange}`
   (closed set, no silent-zero). Semantics: offset<0 or length<=0 ->
   InvalidInput; offset >= EOF -> empty (0 rows); range crossing EOF ->
   clamped partial range; rows are exact-byte slices (<=2048 B/row) with
   `byte_offset`/`byte_length` columns forming a contiguous chain.
2. `crates/frozen-duckdb/tests/host_vfs_local.rs` (new): 6 integration
   tests over real temp files.
3. Shared-file edits (disclosed, beyond lane file list — required wiring):
   - `crates/frozen-duckdb/src/duckdb/mod.rs`: `pub mod host_vfs;` under
     the existing `vtab` cfg, + module doc for `pub mod vtab`.
   - `crates/frozen-duckdb/Cargo.toml`: declared the previously
     UNDECLARED feature gate `vtab = []`. Without it, `cfg(feature =
     "vtab")` was always-false and the entire vtab registration surface
     (function.rs, mod.rs trait, r2d2 vtab block) was unreachable dead
     code — `read_local_range` could not exist without this declaration.

## Verification (real output, this session)

- `cargo test -p frozen-duckdb --features vtab --test host_vfs_local`
  -> exit 0: `6 passed; 0 failed` (registers_and_queries_real_file,
  middle_range_exact_bytes, offset_past_eof_is_empty,
  range_crossing_eof_clamps, invalid_params_error, known_content_via_sql).
- `cargo build -p frozen-duckdb` (default features) -> exit 0 (slice is
  opt-in; default surface unchanged).
- `cargo clippy -p frozen-duckdb --features vtab --all-targets` -> no
  warnings attributable to host_vfs (only sys prebuilt-binary notice).
- `cargo fmt -p frozen-duckdb` applied; no diffs remain in lane files.
- Pre-existing, not this lane's: `tests/tpch_integration_test.rs` fails
  to compile ("unclosed delimiter") unmodified vs HEAD.

## Open (KD-06 remains open)

- HTTP host-callback delegation (real `host-range://`, TLS in host),
  GATE-V-05 <=5MB court, `write`/`NetworkDenied`/`QuotaExceeded` rows,
  network slice of the closed-set status table.
