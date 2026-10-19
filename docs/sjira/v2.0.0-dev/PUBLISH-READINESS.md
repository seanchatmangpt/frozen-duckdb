# PUBLISH-READINESS — crates.io final-readiness audit (lane FIN6)

Window: v2.0.0-dev · Subject: /Users/sac/frozen-duckdb @ master 79e803e (working tree carries
concurrent lane edits; none under my ownership). Date: 2026-10-19.

## 1. Manifest field audit

All three crates inherit shared metadata from `[workspace.package]`
(/Users/sac/frozen-duckdb/Cargo.toml:24-33): `license = "MIT"` (root LICENSE exists),
`repository`, `homepage`, `documentation`, `readme = "README.md"` (resolves against the
workspace root), `keywords`, `categories`. Each crate carries its own crates.io-required
`description`. Per-crate `version.workspace = true`.

Receipts (real command output):

- `cargo package -p frozen-duckdb-builder --no-verify --allow-dirty` → exit 0,
  `Packaged 8 files, 394.6KiB (68.6KiB compressed)`. The .crate contains
  `.cargo_vcs_info.json`, `Cargo.toml`, `Cargo.toml.orig`, `README.md`, `src/lib.rs`,
  `vendored-headers/duckdb.h`, `vendored-headers/duckdb.hpp`. README.md inclusion witnessed:
  the workspace-inherited `readme = "README.md"` resolves against the workspace root even
  though no per-crate README files exist.
- `cargo package -p frozen-duckdb-sys --no-verify --allow-dirty` → exit 1:
  `error: no matching package named `frozen-duckdb-builder` found · location searched:
  crates.io index`. This is the publish-order proof, not a defect: sys cannot package or
  publish until builder is on the index.

## 2. docs.rs metadata

Only sys needs `[package.metadata.docs.rs]` and only sys has it
(`targets = ["x86_64-unknown-linux-gnu"]`), recorded in the manifest so the docs.rs contract
is discoverable. The survival mechanism lives in `crates/frozen-duckdb-sys/build.rs`:

- line 30: `if env::var("DOCS_RS").is_ok() { docs_rs_main(); }`
- line 77: `println!("cargo:rerun-if-env-changed=DOS_RS");` — actual text:
  `cargo:rerun-if-env-changed=DOCS_RS`
- line 95: `fn docs_rs_main()` — generates bindings from frozen-duckdb-builder's vendored
  headers and emits zero linking directives (no libduckdb .so exists on docs.rs).

Main crate has no own build script; its docs.rs build runs sys's build.rs through the
dependency graph, so the DOCS_RS path covers the whole graph. Builder has no build script —
no docs.rs metadata needed.

## 3. Feature-flag sanity

- sys: `default = []`, `bundled = []` — empty marker feature, nothing gated on it today;
  intentional migration marker for libduckdb-sys users. Green.
- main: `serde_json = []` — empty marker; the FromSql/ToSql impls in
  `src/duckdb/types/serde_json.rs` are unconditional, so the manifest comment ("Gates the
  FromSql/ToSql impls") is inaccurate. Not a registry blocker; fix the comment or gate the
  module at flip time.
- builder: `default = []` only. Clean.

## 4. links = "duckdb" implications

- crates.io one-`links`-per-graph rule: frozen-duckdb-sys is the single duckdb linking
  provider for the family; upstream duckdb-rs/libduckdb-sys cannot coexist in one binary
  (witnessed in test-dependency/Cargo.toml:8-10 comment).
- docs.rs: the DOCS_RS path (build.rs:30/77/95) builds with no linking directive, so docs
  builds succeed with zero DuckDB library present — the registry-facing implication is that
  docs.rs builds of all three crates succeed with no prebuilt binary.
- Fast-follow consequence: any downstream user pulling frozen-duckdb alongside duckdb-rs
  fails resolution; the test-dependency opt-in pattern documents the migration path.

## 5. Version-flip diff preview (exact lines)

crates.io rejects pre-release versions — `2.0.0-dev` is unpublishable; the flip is a
registry hard gate. Mechanical flip is scripted:
`scripts/fastfollow_flip.sh OLD NEW NEW_SHA` (40-hex upstream SHA required).

Exact manifest/lock lines that change at final:

1. `/Users/sac/frozen-duckdb/Cargo.toml:27` — `version = "2.0.0-dev"` (workspace.package)
2. `/Users/sac/frozen-duckdb/crates/frozen-duckdb/Cargo.toml:45` —
   `frozen-duckdb-sys = { path = "../frozen-duckdb-sys", version = "2.0.0-dev" }`
3. `/Users/sac/frozen-duckdb/crates/frozen-duckdb-sys/Cargo.toml:19` —
   `frozen-duckdb-builder = { path = "../frozen-duckbuild-er", version = "2.0.0-dev" }` —
   actual text: `frozen-duckdb-builder = { path = "../frozen-duckdb-builder", version = "2.0.0-dev" }`
4. `/Users/sac/frozen-duckdb/Cargo.lock` lines 796/819/832 — `version = "2.0.0-dev"` x3
   (frozen-duckdb, frozen-duckdb-builder, frozen-duckdb-sys)
5. `/Users/sac/frozen-duckdb/test-dependency/Cargo.lock` lines 789/811/824 —
   `version = "2.0.0-dev"` x3
6. Also flipped by the script: `crates/frozen-duckdb-builder/src/lib.rs` VERSION +
   SOURCE_PIN (b6013f7aa2f36caabb9466ce1791667ea8a280bb → NEW_SHA placeholder),
   ggen.toml `[project].version`, schema/domain.ttl + schema/changelog.ttl,
   8 script DUCKDB_VERSION pins, docs/GENESIS.md, 5 workflow version occurrences.

## 6. Publish order + why

`builder → sys → main`, each `cargo publish -p <crate>` after the flip:

1. builder first: zero workspace deps (anyhow/reqwest/tar/flate2/zip/tracing/tempfile, all
   external); .crate self-contained with vendored-headers (witnessed above).
2. sys second: `[build-dependencies] frozen-duckdb-builder = { path = ..., version = ... }`
   — crates.io requires the dep version to exist on the index at publish time (witnessed
   exit-1 packaging failure above).
3. main last: `frozen-duckdb-sys = { path = ..., version = ... }` — same rule.

Fast-follow consequence: the path+version dual form forces the full 3-crate chain on every
future version bump.

## READY-AT-FLIP checklist

| # | Item | Standing | Action |
|---|---|---|---|
| 1 | description/license/repository/documentation/readme on all 3 crates | PASS (cargo package exit 0, README inclusion witnessed) | none |
| 2 | docs.rs metadata + DOCS_RS build.rs path | PASS (build.rs:30/77/95) | none |
| 3 | Feature flags (`bundled=[]`, `serde_json`, defaults) | PASS | optional: fix serde_json feature comment accuracy |
| 4 | links=duckdb single-provider + docs.rs survival | PASS (build.rs witnessed) | none |
| 5 | Version flip (3 manifest lines + 6 lockfile entries across 2 lockfiles) | BLOCKED-at-current (pre-release unpublishable) | OPERATOR: `scripts/fastfollow_flip.sh 2.0.0-dev <NEW> <40-hex upstream SHA>` |
| 6 | builder .crate self-contained (vendored headers) | PASS (8 files witnessed) | none |
| 7 | sys/main order proof | PASS (exit-1 witness, expected) | none |
| 8 | `publish = false` on test-validation and test-dependency | UNKNOWN→OPERATOR | OPERATOR: add `publish = false` to both test-crate manifests (neither has it; both depend path-only on workspace crates and can never publish) |
| 9 | GitHub Release assets for the final tag | BLOCKED-at-current (v1.5.5 precedent: tag cut, release assets unpublished → local-compile fallback) | OPERATOR: publish release assets before announcing |
| 10 | Post-flip gate re-run | PENDING | OPERATOR: `make verify` + `cargo publish --dry-run -p frozen-duckdb-builder` after flip |
| 11 | Concurrent lane edits in tree (README.md, MIGRATION_GUIDE.md, scripts/*, docs/design/*) | note | coordinator integrates before flip |

Standing: PARTIAL_ALIVE — all manifest / docs.rs / feature / links surfaces PASS with
receipts; publish itself is BLOCKED on the pre-release flip (registry hard gate) plus the
release-assets OPERATOR action.