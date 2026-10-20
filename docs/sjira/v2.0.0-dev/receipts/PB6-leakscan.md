# PB6 — Leak/Size Rescan on Final Tree (2.0.0-dev)

- Date: 2026-10-19
- Subject: master @ adbd260 (tree dirty: lane files in flight, incl.
  uncommitted `LICENSE-DUCKDB.md` x3)
- Precondition: no git state commands used; read-only scan + `cargo package`.
- Prior: `AD8-PACKAGE-AUDIT.md` (master @ 79e803e).

## Per-crate table (re-run of AD8 method on the final tree)

| crate | ships | unpacked bytes | `.crate` artifact | leak scan | offline `cargo package` | verdict |
|---|---|---|---|---|---|---|
| frozen-duckdb-sys | 13 files | 16,896 B | — (path-dep blocked) | clean | FAIL: `no matching package frozen-duckdb-builder found` (path-dep resolution, publish-order constraint) | READY (publish second) |
| frozen-duckdb-builder | 9 files | 389,294 B | 81,176 B (79.7 KiB compressed) | clean (test-only `/tmp` literals remain inside `#[cfg(test)]`, AD8 item 3, accepted) | PASS `Packaged 9 files, 438.3KiB` | READY (publish first) |
| frozen-duckdb | 80 files | 884,323 B | — (path-dep blocked) | clean | FAIL: `no matching package frozen-duckdb-builder found ... crates.io index` (path-dep resolution, publish-order constraint) | READY (publish last) |

All three crates are far under the 10 MB crates.io limit (largest unpacked
tree 0.88 MB, largest `.crate` 81 KB).

## Method (real executions, 2026-10-19)

- `cargo package --list --offline --allow-dirty -p <crate>` per crate;
  full grep sweep of every ship-list filename AND shipped-file content for
  `/Users/sac`, `/tmp/`, `target/`, `.ggen`, `.env`, `secret`, `attic/`,
  `/home/`, lane fixture paths.
- `cargo package --offline --no-verify --allow-dirty -p <crate>` per crate
  (recorded verdicts in table).
- Sizes: byte sums over ship lists resolved against crate dirs; `.crate`
  from `target/package/frozen-duckdb-builder-2.0.0-dev.crate`.

## Delta vs AD8 (new leaks since?)

**None.** Concretely:

1. **Ship-list growth is benign:**
   - `frozen-duckdb`: 77 → **80** files. Delta = `LICENSE-DUCKDB.md`
     (new since AD8, uncommitted, clean DuckDB license text) + 2 src files
     added by other lanes. The 13 machine-coupled test files AD8 excluded via
     `exclude = [...]` in `crates/frozen-duckdb/Cargo.toml` remain excluded —
     `tests/` has 32 files on disk, exactly 19 ship (the 13 exclusions hold).
   - `frozen-duckdb-builder`: 8 → **9** (+`LICENSE-DUCKDB.md`).
   - `frozen-duckdb-sys`: 12 → **13** (+`LICENSE-DUCKDB.md`).
2. **Content grep: zero real leaks.** All pattern hits are false positives:
   - `secret` hits = DuckDB `CREATE SECRET ollama_secret` SQL in
     `src/cli/flock_manager.rs`, `src/cli/commands.rs`, `tests/flock_tests.rs`
     (DuckDB syntax),
     doc comments; no credentials, no `.env`, no credentials on disk.
   - `lane` hits = lane bookkeeping comments only.
   - builder `/tmp` literals still inside `#[cfg(test)]` (AD8 item 3,
     unchanged, accepted).
3. **`quickstart_demo` and the cleanroom scripts do NOT ship** — absent from
   all three ship lists (they live outside the crate dirs). New test files
   that ship (`gate_ingest_throughput.rs`, `oom_graceful_abort.rs`,
   doctor_* court tests) are content-clean: no machine paths.
   `tests/doctor_leak_audit.rs` ships and is itself a leak-audit court.
4. `.cargo_vcs_info.json` still ships git SHA + `dirty: true` (normal cargo
   behavior; real publish requires clean tree — unchanged operator note).

## Verdict

**ALIVE** for 2.0.0-dev packaging on the final tree, with AD8's operator
requirements unchanged: publish order builder → sys → frozen-duckdb, from a
clean tree. Zero new leaks introduced by the tree changes since AD8.
