# PB11 — Supply-Chain Rehearsal: cargo audit / cargo deny / minimal update set

- Subject: /Users/sac/frozen-duckdb @ master adbd260 (2.0.0-dev)
- Date: 2026-10-19
- Tooling: cargo-audit 0.22.2, cargo-deny 0.20.2 (both on PATH; advisory DB 1338 advisories)
- Lane discipline: NO updates applied; NO git state changes. Dry-runs only.

## 1. cargo audit (real output)

Exit nonzero ("error: 4 vulnerabilities found! / warning: 2 allowed warnings found").

| ID | Crate | Ver | Severity | Fix |
|---|---|---|---|---|
| RUSTSEC-2026-0258 | h2 0.3.27 | unbounded empty DATA frames | (no sev) | >=0.4.16 |
| RUSTSEC-2026-0367 | zip 0.6.6 | SUID/SGID preservation on extract | 7.3 high | >=9.0.0 |
| RUSTSEC-2026-0368 | zip 0.6.6 | symlink escape in ZipArchive::extract | — | >=9.0.0 |
| RUSTSEC-2026-0369 | zip 0.6.6 | permissions through symlinks | 9.1 critical | >=9.0.0 |
| RUSTSEC-2025-0134 | rustls-pemfile 1.0.4 | unmaintained (warning) | — | migrate to rustls-pki-types |
| (yanked) | yoke-derive 0.8.3 | yanked (warning) | — | 0.8.4 available |

## 2. cargo deny check (both tools real runs)

- `cargo deny check` → **advisories FAILED, bans ok, licenses FAILED, sources ok**.
- licenses FAILED is a **tool-config artifact, not a real supply-chain defect**: no
  deny.toml exists, so defaults reject every not-explicitly-allowed license
  (adler2, aes, ahash, ... — all standard OSI permissive). CLASSIFICATION: NON-BLOCKING
  config debt; add a deny.toml with allow-list before relying on cargo-deny as a gate.
- Advisories section independently FAILED (same 4 vulns as cargo audit).

## 3. cargo update --dry-run (full)

32 packages move on a blanket update. Highlights: cc 1.4.7→1.9.0, libc, serde_json,
syn 3.0.6→3.0.9, tokio 1.53.1→1.54.0, toml_edit/toml_parser, wasm-bindgen family,
yoke-derive 0.8.3→0.8.4, zerocopy 0.8.57→0.8.63, zstd-sys. Removes multiversion{,-macros}.
Blanket update is broader than needed — not proposed for this lane.

## 4. Minimal advisory-closing update set (proposed, NOT applied)

Verified by per-crate dry-run:

1. `cargo update -p yoke-derive` → 0.8.3→0.8.4 — closes the yanked warning. One-line,
   semver-compatible, zero risk. Do this.
2. **zip 0.6.6** — cannot be closed by `cargo update -p zip` (locks 0 packages:
   0.6 line is EOL for advisories; fixes land in >=9.0.0). Requires workspace
   `Cargo.toml:24` bump `zip = "0.6"` → `"9"` (or "2"+) + API migration in
   crates/frozen-duckdb-builder. Direct dep of frozen-duckdb-builder, used for
   release/publish packaging. **BLOCKING for 2.0.0 publish** (9.1 critical RUSTSEC-2026-0369
   in a direct dep that touches untrusted archives).
3. **h2 0.3.27** — `cargo update -p h2` locks 0 packages; fix >=0.4.16 requires
   reqwest 0.11→0.12 (workspace `Cargo.toml:19`) + hyper 0.14→1.x migration. Indirect
   (via reqwest, blocking feature). **NON-BLOCKING for publish** (no severity given,
   unbounded-empty-DATA-frame DoS class), but should ride the reqwest 0.12 wave.
4. rustls-pemfile unmaintained — resolves together with the reqwest 0.12 migration.
   NON-BLOCKING.

### Proposed minimal command set (for the owning lane, not this lane)

```bash
cargo update -p yoke-derive                                   # closes yanked warning
# separate commit/PR:
#   Cargo.toml: zip = "9" (+ builder API migration)           # closes 0367/0368/0369
#   Cargo.toml: reqwest = "0.12" (+ hyper 1.x migration)      # closes 0258 + 0134
```

## 5. BLOCKING vs NON-BLOCKING for 2.0.0 publish

**BLOCKING**
- zip 0.6.6 (RUSTSEC-2026-0367/0368/0369; one critical 9.1) — direct dep, fix is major-bump-only.

**NON-BLOCKING**
- yoke-derive yanked warning (one-line lockfile bump available).
- h2 0.3.27 RUSTSEC-2026-0258 (major-bump wave via reqwest 0.12).
- rustls-pemfile unmaintained (same wave).
- cargo-deny licenses failure (missing deny.toml defaults, not a real defect).

## Receipt

- Commands: `cargo audit`, `cargo deny check`, `cargo update --dry-run`,
  `cargo update -p {zip,h2,yoke-derive} --dry-run`, `cargo tree -i {zip,h2}`.
- All outputs above are real executions on adbd260. No lockfile or Cargo.toml was
  modified (dry-run verified: "not updating lockfile due to dry run").
