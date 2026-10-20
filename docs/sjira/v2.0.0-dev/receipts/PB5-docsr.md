# PB5 — docs.rs rehearsal, all three crates

- Subject: /Users/sac/frozen-duckdb @ master `adbd26014ad62d4f4a2da54a75b9d328dba053fb`
- Date: 2026-10-19
- Env: `DOCS_RS=1` (docs.rs parity), local toolchain, vendored headers
  (`crates/frozen-duckdb-builder/vendored-headers/`).

## Per-crate receipts

### frozen-duckdb-sys
- `DOCS_RS=1 cargo build -p frozen-duckdb-sys` → **exit 0**, 3.52s.
- build.rs DOCS_RS path fired for real: `docs.rs build: bindings generated
  from vendored headers (.../frozen-duckdb-builder/vendored-headers), no
  library linked`. No link errors — the vendored-header bindgen path is intact
  post-changes (re-receipt of the earlier witness).
- `DOCS_RS=1 cargo doc --no-deps -p frozen-duckdb-sys` → exit 0, **3 doc
  warnings**, all in generated `out/bindgen.rs` (2 unresolved links `types`,
  `entry_idx`; 1 unexpected-character). Bindgen-generated rustdoc noise only;
  not fixable in source.

### frozen-duckdb-builder
- `DOCS_RS=1 cargo build -p frozen-duckdb-builder` → **exit 0**.
- `cargo doc --no-deps` → exit 0, **0 warnings**. Clean.

### frozen-duckdb (main)
- `DOCS_RS=1 cargo doc --no-deps -p frozen-duckdb` → exit 0, **15 warnings,
  ALL 15 are `rustdoc::broken_intra_doc_links`** (`unresolved link`).
  Locations:
  - `crates/frozen-duckdb/src/duckdb/error.rs:51` — `[crate::duckdb::Connection::query_one]` (no such item)
  - `crates/frozen-duckdb/src/duckdb/appender_params.rs` — 8 links: `ParamsFromIter` (x4: 55,113,117,278), `Params` (181), `params_from_iter` (183), `Statement::exists` (217), `crate::duckdb::Connection::limit` (269)
  - `crates/fduckdb config.rs` — `AUTOMATIC` (74), `ASC` (86), `NULLS_FIRST` (92) — bare-word links rendered as intra-doc
  - `crates/frozen-duckdb/src/duckdb/params.rs:269` — `crate::duckdb::Connection::limit`
  - `crates/fduckdb statement.rs:228` — `params`
  - `crates/frozen-duckdb/src/duckdb/types/to_sql.rs:74` — `Error::ToSqlConversionFailure`
- 0 non-link doc warnings.

## docs.rs failure semantics (recorded, actual)

- docs.rs builds fail on **build errors**, not rustdoc warnings. docs.rs does
  NOT deny rustdoc warnings by default (the "-D warnings since 2024" premise
  is false; docs.rs builds with warnings shown, failing only on hard errors).
  No `[lints]` rustdoc deny in workspace or crate manifests (checked: zero
  `lints`/`rustdoc` entries).
- **Therefore no doc warning here would fail docs.rs.** All three crates:
  builds exit 0 under `DOCS_RS=1`; docs builds exit 0. Docs.rs rehearsal
  standing: **ALIVE** for all three; publish-blocked only by the known
  missing GitHub Release v1.5.5 assets item (outside this lane).

## Findings

1. **15 broken intra-doc links in frozen-duckdb** — would render as dead
   links on docs.rs. Not docs.rs-fatal, but a quality finding. Most are
   duckdb-rs inherited doc text referencing items this crate re-exports
   differently (`ParamsFromIter`, `params_from_iter`, `Params`,
   `Statement::exists`, `Connection::limit`, `Connection::query_one`) plus 3
   bare-word links in `config.rs` (`AUTOMATIC`/`ASC`/`NULLS_FIRST` — should
   be fenced as `` `AUTOMATIC` `` not `[]`-linked). Fix is small and
   lane-compatible: 8 sites in `appender_params.rs`, 1 each in
   `error.rs`/`params.rs`/`statement.rs`/`to_sql.rs`, 3 in `config.rs`.
2. **3 bindgen rustdoc warnings in frozen-duckdb-sys** — generated-code
   noise, acceptable; not docs.rs-fatal.
3. docs.rs parity env (`DOCS_RS=1`) verified to change build.rs behavior
   (rerun-if-env-changed wire observed; vendored-header bindgen path taken).
