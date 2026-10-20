# REL3 — API Layer Receipt (2.0.0-dev)

Lane: REL3 (verify+receipt). Date: 2026-10-19.
Subject: /Users/sac/frozen-duckdb @ master dbac0b4 (working tree; `scripts/verify-evidence.ttl` pre-modified by another lane — not mine).
Scope owned: this file only. No git commands run.

## 1. Public API census

`cargo doc --no-deps` — exit 0, `Generated target/doc/frozen_duckdb/index.html and 3 other files`
(3 doc warnings from frozen-duckdb-sys only).

Crate-root re-export list (`crates/frozen-duckdb/src/lib.rs` lines 163–196), verified on disk:

- Modules: `duckdb`, `architecture`, `benchmark`, `env_setup`, `cli`
- Re-exports: `params_from_iter`, `Appender`, `Config`, `Connection`, `Error as DuckDBError`,
  `Result as DuckDBResult`, `Row`, `Rows`, `Statement`, `ToSql`, `Transaction`
- Convenience re-exports: `FromSql`, `Type`, `Value` (from `duckdb::types`), `arrow` module,
  `pub type Result<T> = DuckDBResult<T>`
- `params!` is `#[macro_export]` at the crate root (exported from the vendored module), matching
  duckdb-rs; `params_from_iter` comes through the `duckdb` module.

Static census of the vendored API layer (`crates/frozen-duckdb/src/duckdb/`): 366 public items
(`pub fn|struct|enum|trait|type|mod|use|const`), of which 266 are `pub fn`.

Cross-reference: generated `docs/reference/api.md` re-export table matches the lib.rs root list
(Connection/Statement/Transaction/Appender/Config/Row/Rows/Result/Error/ToSql/FromSql/Type/Value/
arrow/params_from_iter). No drift.

## 2. Drop-in proof (real runs)

`cargo test -p frozen-duckdb --test dropin_compatibility_tests` → **7 passed; 0 failed** (0.37s).
Tests: connection, prepared statements, transaction, complex queries, error handling, performance,
data types — all duckdb-rs-shaped.

## 3. VARIANT documentation

`docs/reference/variant-types.md` exists and documents type id 41, `LogicalTypeId::Variant`
(`src/duckdb/types/mod.rs:105`), FFI + Arrow bridge + value layer, pointer to
`docs/design/variant-shredding-design.md` (FR-03). Real grep confirms `Variant` in the types layer.

Gap noted: `variant-types.md` is cross-linked only from `docs/reference/schemas-ocel.md`; the
generated `docs/reference/api.md` re-export table does not yet mention the VARIANT addition.
Projection, so fix belongs upstream in the pack, not hand-edited. Logged here as reconciliation
debt, not fixed in this lane.

## 4. Test suite counts (re-run)

| Suite | Command | Result |
|---|---|---|
| lib | `cargo test -p frozen-duckdb --lib` | **140 passed; 0 failed** (0.65s) |
| dropin | `--test dropin_compatibility_tests` | **7 passed; 0 failed** (0.37s) |
| core | `--test core_functionality_tests` | **10 passed; 0 failed** (0.40s) |

Note: earlier milestone prose says "141 lib tests"; the re-run observed **140** on this subject.
Reported count corrected to the on-disk receipt (140), per no-overclaiming law.

## Standing

ALIVE for the drop-in API surface at 2.0.0-dev: docs build, root surface matches duckdb-rs shape,
drop-in suite green 7/7, lib+dropin+core green (157 total across the three suites).
One registered gap: VARIANT not yet in the generated api.md cross-link graph.
