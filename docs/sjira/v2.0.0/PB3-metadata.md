# PB3 — Manifest Metadata Audit (crates.io norms)

Lane: PB3. Read-only audit; no manifest edits applied (coordinator applies the
diffs in this file). Date: 2026-10-19. Repo: /Users/sac/frozen-duckdb.

## Verification (real command output)

```
$ rustc --version
rustc 1.97.0 (2d8144b78 2026-07-07)
```

```
$ cargo metadata --format-version 1 | python3 -c "import json,sys; d=json.load(sys.stdin); \
  [print(p['name'],p['version'],'rust_version=',p.get('rust_version'),'edition=',p.get('edition'),'authors=',p.get('authors')) for p in d['packages'] if p['name'].startswith('frozen-duckdb')]"
frozen-duckdb 2.0.0 rust_version= None edition= 2021 authors= ['Sean Chatman']
frozen-duckdb-builder 2.0.0 rust_version= None edition= 2021 authors= ['Sean Chatman']
frozen-duckdb-sys 2.0.0 rust_version= None edition= 2021 authors= ['Sean Chatman']
```

```
$ cargo package -l -p frozen-duckdb        2>/dev/null | grep -i readme
README.md
$ cargo package -l -p frozen-duckdb-builder 2>/dev/null | grep -i readme
README.md
$ cargo package -l -p frozen-duckdb-sys    2>/dev/null | grep -i readme
README.md
```

All three `cargo package -l` runs exited 0 and each packages `README.md`
(workspace-root README, 11,657 bytes, resolves via `readme.workspace = true`;
`cargo metadata` shows resolved path `../../README.md`).

URL check:

```
$ curl -s -o /dev/null -w "%{http_code}" https://github.com/seanchatmangpt/frozen-duckdb
200
```

`https://crates.io/categories/database` returned HTTP 403 to curl (bot-blocked);
`database` is nonetheless a real crates.io category slug — verified by slug, not
by curl.

## Findings

| # | Finding | Severity |
|---|---|---|
| 1 | `rust-version` absent (`None`) in all three crates | High — publish gate |
| 2 | `documentation` URL is the GitHub repo, not a docs.rs URL | Medium |
| 3 | `homepage` duplicates `repository` | Low (norm-permitted) |
| 4 | `c"..."` literal at `crates/frozen-duckdb-builder/src/lib.rs:2297` (`c"duckdb_library_version"`) stabilized 1.77 → measured MSRV floor | High — drives #1 |
| 5 | authors `["Sean Chatman"]` present, consistent; crates.io `updated_at` is server-side, not manifest-controlled | Info |
| 6 | keywords `["duckdb","database","ffi"]` — 3 ≤ 5, all lowercase | OK |
| 7 | categories `["database"]` — valid crates.io slug | OK |

## Per-crate tables

All package fields inherit from `[workspace.package]` (root `Cargo.toml`), so
each fix lands once and covers all three crates. Resolved values confirmed via
`cargo metadata`.

### frozen-duckdb

| field | current | proposal | rationale |
|---|---|---|---|
| description | present, specific ("Drop-in replacement for the duckdb (duckdb-rs) crate...") | keep | Publish-ready |
| keywords | ["duckdb","database","ffi"] | keep | ≤5, lowercase |
| categories | ["database"] | keep | Valid slug |
| repository | https://github.com/seanchatmangpt/frozen-duckdb (HTTP 200) | keep | Resolves |
| homepage | same as repository | keep | Norm-permitted |
| documentation | GitHub URL | remove field | docs.rs auto-links; a GitHub `documentation` misdirects |
| readme | workspace → packages `README.md` (`cargo package -l`) | keep | Verified packaged |
| rust-version | None | "1.77" | MSRV floor: `c""` literal (builder lib.rs:2297) stabilized 1.77; edition 2021 needs 1.56 |
| authors | ["Sean Chatman"] | keep | Consistent |

### frozen-duckdb-builder

| field | current | proposal | rationale |
|---|---|---|---|
| description | present, specific (build-time download/compile companion) | keep | Publish-ready |
| keywords | ["duckdb","database","ffi"] | keep | ≤5, lowercase |
| categories | ["database"] | keep | Valid slug |
| repository/homepage | GitHub (200) | keep | Resolves |
| documentation | GitHub URL | remove field | docs.rs auto-links |
| readme | packages `README.md` | keep | Verified |
| rust-version | None | "1.77" | `c""` literal at src/lib.rs:2297 |
| authors | ["Sean Chatman"] | keep | Consistent |

### frozen-duckdb-sys

| field | current | proposal | rationale |
|---|---|---|---|
| description | "FFI bindings for frozen-duckdb mega-library" | optional tighten: "Low-level FFI bindings to the DuckDB analytical database" | Terse but accurate |
| keywords | ["duckdb","database","ffi"] | keep | ≤5, lowercase |
| categories | ["database"] | keep | Valid slug |
| repository/homepage | GitHub (200) | keep | Resolves |
| documentation | GitHub URL | remove field | docs.rs auto-links |
| readme | packages `README.md` | keep | Verified |
| rust-version | None | "1.77" | workspace-inherited |
| authors | ["Sean Chatman"] | keep | Consistent |
| links = "duckdb" | present | keep | Single linking provider; correct |

## Exact TOML diffs for coordinator apply

### Diff 1 — MSRV (root `Cargo.toml`, `[workspace.package]`)

```diff
--- a/Cargo.toml
+++ b/Cargo.toml
@@ [workspace.package]
 edition = "2021"
+rust-version = "1.77"
 license = "MIT"
```

### Diff 2 — documentation field (root `Cargo.toml`, `[workspace.package]`)

```diff
--- a/Cargo.toml
+++ b/Cargo.toml
@@ [workspace.package]
 homepage = "https://github.com/seanchatmangpt/frozen-duckdb"
-documentation = "https://github.com/seanchatmangpt/frozen-duckdb"
 readme = "README.md"
```

Then per-crate `documentation.workspace = true` lines become inert (field no
longer exists in workspace.package); the coordinator should also delete the
three `documentation.workspace = true` lines:

```diff
--- a/crates/frozen-duckdb/Cargo.toml
+++ b/crates/frozen-duckdb/Cargo.toml
@@ -4
-documentation.workspace = true
```

Same deletion in `crates/frozen-duckdb-builder/Cargo.toml` and
`crates/frozen-duckdb-sys/Cargo.toml`.

### Optional Diff 3 — sys description tighten (crates/frozen-duckdb-sys/Cargo.toml)

```diff
--- a/crates/frozen-duckdb-sys/Cargo.toml
+++ b/crates/frozen-duckdb-sys/Cargo.toml
@@ [package]
-description = "FFI bindings for frozen-duckdb mega-library"
+description = "Low-level FFI bindings to the DuckDB analytical database for frozen-duckdb"
```

## Notes

- MSRV rationale: only one `c"..."` literal exists in shipped code
  (`crates/frozen-duckdb-builder/src/lib.rs:2297`). Edition 2021 alone would
  allow 1.56; the literal raises the floor to 1.77. Toolchain on this machine is
  1.97.0, so 1.77 is the measured code-level floor, not a build-tested minimum;
  if the coordinator wants a build-tested MSRV it needs an older-toolchain run.
- crates.io `updated` timestamp is server-side; nothing in the manifest maps to
  it. Authors field is correct as-is.
- After apply, gate: `cargo metadata` should show `rust_version= Some("1.77")`
  for all three and `documentation= None`.
