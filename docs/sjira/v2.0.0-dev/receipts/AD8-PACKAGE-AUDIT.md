# AD8 — Package Chain Audit (2.0.0-dev)

- Date: 2026-10-19
- Subject: master @ 79e803e (tree dirty: docs + lane edits in flight)
- Method: `cargo package --allow-dirty --offline --list` per crate + full
  grep sweep of every ship-list entry + extracted `.crate` content inspection
  + real builds (`DOCS_RS=1 cargo build -p frozen-duckdb-sys`, isolated
  registry-scan binary compiled with rustc and run against a docs.rs-shaped
  fake `$CARGO_HOME`).

## Per-crate verdicts

| crate | ships | raw size | leak scan | verdict |
|---|---|---|---|---|
| frozen-duckdb-builder | 8 files (src/lib.rs, vendored-headers/duckdb.{h,hpp}, README, lock) | 345,566 B (`.crate` = 70,232 B) | clean | READY |
| frozen-duckdb-sys | 12 files (build.rs, wrapper*.h, src/*.rs) | 15,709 B | clean | READY (publish order: after builder) |
| frozen-duckdb | 77 files (after AD8 exclusion) | 970,309 B | clean | READY (publish order: after sys) |

All three are far under the 10 MB crates.io limit (largest raw tree 0.97 MB;
largest `.crate` observed 70 KB).

## Leaks found

1. **13 machine-coupled test files shipped in `frozen-duckdb`.**
   `tests/gate_v01_latency.rs`, `tests/test_fleet_class_{a..e}_*.rs` (5),
   `tests/test_rev_*.rs` (4), `tests/value_audit_*.rs` (3) contained
   hard-coded `/Users/sac/...` court paths (e.g. `COURT: &str =
   "/Users/sac/frozen-duckdb/scripts/courts/fleet_d_chaos.sh"` in
   `test_fleet_class_d_marine.rs:30`) and `/tmp/...` lane fixture paths.
   These are repo-side court/value-audit fixtures, not portable consumer
   tests. **FIXED**: excluded via `exclude = [...]` in
   `crates/frozen-duckdb/Cargo.toml` (manifest edit, disclosed per lane
   contract). Post-fix re-list + content grep: zero machine-path hits across
   all 77 shipped files; 19 portable `tests/` files still ship.
2. `.ggen-v2/` (receipt.json, receipt-log.jsonl) — gitignored and **absent
   from all three ship lists**. No fix needed. `.env` patterns also ignored.
3. builder `src/lib.rs` ships `/tmp/test` literals at lines 864/912 — inside
   `#[cfg(test)]` (module starts line 834). Test-only literals, inert in
   consumer builds, common on crates.io. Accepted, no edit.
4. builder `VENDORED_HEADERS_DIR` embeds `env!("CARGO_MANIFEST_DIR")` at
   compile time — on consumer machines this resolves to the registry
   extraction path, i.e. portable-by-construction. Not a leak.

## docs.rs / standalone-build verification (item 3)

- `cargo package --offline -p frozen-duckdb-sys` **fails at dependency
  resolution** (`frozen-duckdb-builder 2.0.0-dev` not on the registry) —
  this is the publish-order constraint, not a defect: verify-compile of a
  workspace member with a version-pinned path dep needs the registry
  version, so publish order is **builder → sys → frozen-duckdb**.
- `DOCS_RS=1 cargo build -p frozen-duckdb-sys` — PASS (4.89 s): docs branch
  generated bindings from vendored headers, emitted no linking directive.
- The registry-candidate branch of `find_vendored_headers()` was exercised
  in isolation: an exact copy of the scan logic compiled with `rustc` and
  run with `CARGO_MANIFEST_DIR` pointing nowhere and `CARGO_HOME` shaped as
  a docs.rs home (`registry/src/index.crates.io-6f14d713e4c8e5bd/
  frozen-duckdb-builder-2.0.0-dev/vendored-headers/` containing the
  extracted `.crate`'s headers) returned
  `FOUND .../frozen-duckdb-builder-2.0.0-dev/vendored-headers`. The
  candidate-2 lookup works against the real `.crate` layout.

## Licenses (item 5)

Workspace `license = "MIT"` (inherited by all three crates) satisfies the
crates.io license requirement via SPDX identifier. The `LICENSE` text file
itself is at workspace root and does not ship in any package (cargo only
auto-includes license files inside each crate dir). No violation; noted so
the release operator knows the SPDX field is the sole carrier.

## Adversarial scan summary (item 4)

- Zero `/Users/...` hits in any shipped file (post-fix).
- Zero `/tmp` hits outside `#[cfg(test)]` test literals.
- `.cargo_vcs_info.json` ships the git SHA + `dirty: true` — normal cargo
  behavior; a real `cargo publish` from a dirty tree requires
  `--allow-dirty`. **Readiness note**: publish from a clean tree.
- No `.ggen-v2`, receipt logs, or machine-cache layout paths in any package.

## Standing

ALIVE for 2.0.0-dev packaging, with the publish-order constraint
(builder → sys → frozen-duckdb) and clean-tree publish as operator
requirements. The 13-file exclusion is session-introduced (AD8) and
disclosed in the manifest comment.
