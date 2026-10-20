# PB10 — Feature Audit Receipt

Repo: /Users/sac/frozen-duckdb · lane PB10 · 2026-10-19 · no git ops performed.

## Subject

`crates/frozen-duckdb/Cargo.toml` [features],
`crates/frozen-duckdb/src/duckdb/types/mod.rs`,
`crates/frozen-duckdb/src/duckdb/types/serde_json.rs`,
`crates/frozen-duckdb-sys/Cargo.toml` + `build.rs`.

## Findings

### F1 — serde_json feature is load-bearing (FIN6's "impls unconditional" is FALSE)

`types/mod.rs:20-21` carries `#[cfg(feature = "serde_json")] mod serde_json;`.
The entire `ToSql`/`FromSql` impl module for `serde_json::Value` is compiled
only under the feature. There are no other impl sites: `row.rs:398` is a doc
comment only; `main.rs` (the CLI binary) uses serde_json directly and is not
feature-gated by design. Comment in Cargo.toml was already truthful; it was
tightened instead of the cfg changed.

Compile receipt (both green, real output, cargo 1.90-style `Finished` lines):

```
$ cargo check -p frozen-duckdb --lib                       -> exit 0, Finished 4.76s
$ cargo check -p frozen-duckdb --lib --features serde_json -> exit 0, Finished 1.83s
```

### F2 — serde_json dependency is unconditional (compile cost, unresolved)

`serde_json.workspace = true` sits in `[dependencies]`, not optional. A
consumer who never enables the feature still pays serde_json's compile. Making
it `optional = true` is the minimal-cost fix but touches dependency resolution
beyond this lane's one-line ownership; registered as divergence, not silently
skipped: **[DIVERGENCE] F2 — serde_json dep not optional; owner decision
required** (would become `serde_json = { workspace = true, optional = true }`
+ feature `serde_json = ["dep:serde_json"]`).

### F3 — bundled=[] empty marker: documented WHY in-tree, not in manifest

`crates/frozen-duckdb-sys/build.rs:179-181`: "No bundled build path exists:
this crate always links the frozen prebuilt dylib. The legacy `bundled` feature
(still declared in Cargo.toml) must stay inert so `--all-features` remains a
no-op — hence no cfg(feature) gate here." So `bundled = []` is a migration
marker (compat with upstream duckdb-rs's `bundled` flag) whose inertness is
load-bearing. **[DIVERGENCE] F3 — `bundled = []` in sys Cargo.toml has no
adjacent WHY comment; build.rs carries it.** One-line comment fix available
but outside this lane's file ownership.

### F4 — vendored=[] naming consistency: no such feature exists

Neither crate declares a `vendored` feature. Naming pairs in-tree:
- `bundled` (feature, sys) — legacy inert marker (F3)
- `vendored-headers/` (directory, builder crate) — offline DuckDB 1.5.5
  headers for bindgen
The feature/dir name split (bundled vs vendored-headers) is cosmetic drift,
not a functional inconsistency; no consumer greps one against the other.

### F5 — default features minimal for consumers: YES

`[features]` in the main crate declares only `serde_json` with no `default`
key, so default = empty set. Confirmed by `cargo check -p frozen-duckdb --lib`
(build succeeds with the serde_json module compiled out). Consumer compile
cost from this crate's own feature surface is minimal; see F2 for the dep-level
cost that remains.

## Diff (sole file edit)

`crates/frozen-duckdb/Cargo.toml` [features] comment — replaced one line with
three (marked PB10): states the cfg gate location (types/mod.rs:20), that the
feature is load-bearing, and that the dependency is unconditional.

## Standing

ALIVE for F1/F4/F5 (observed: cfg source read + two real compile receipts).
F2/F3 registered divergences (loud, not skipped).
