# PB8 Receipt: Full flip + publish mechanical rehearsal #2 (lane PB8)

Date: 2026-10-19 · Repo: /Users/sac/frozen-duckdb (canonical, untouched;
`git rev-parse HEAD` = `adbd26014ad62d4f4a2da54a75b9d328dba053fb`) ·
Scratch subject: `git archive master` extracted to `/tmp/pb8/scratch`.
Placeholder upstream SHA used: `f000000000000000000000000000000000000000`.

## Step receipts (all commands run in /tmp/pb8/scratch)

| # | step | command | exit | evidence |
|---|---|---|---|---|
| 1 | BASELINE subject | `git archive master \| tar -x -C /tmp/pb8/scratch` | 0 | scratch matches `master` tree at adbd260 |
| 2 | version flip | `bash scripts/fastfollow_flip.sh 2.0.0-dev 2.0.0 f000…f000` | 0 | `/tmp/pb8/flip-output.txt`: 29 FLIP lines; builder `VERSION = "2.0.0"`, `SOURCE_PIN = f000…`; zero `2.0.0-dev` remaining in any manifest/lock |
| 3 | PB1 publish flags | `sed -i '' 's/^edition = /publish = false\nedition = /' test-dependency/Cargo.toml test-validation/Cargo.toml` | 0 | `publish = false` at line 4 of both test-crate manifests |
| 4 | PB2 license file | created `crates/{frozen-duckdb,frozen-duckdb-sys,frozen-duckdb-builder}/LICENSE-DUCKDB.md` (737 B each, MPL 1.1 notice, pinned tag v1.5.5) | — | see DEFECT below — PB2's original file is NOT in master |
| 5 | lockfile | `cargo generate-lockfile` | 0 | Cargo.lock: frozen-duckdb / -sys / -builder all `version = "2.0.0"` |
| 6 | package builder | `cargo package --offline -p frozen-duckdb-builder` | 0 | `Packaged 8 files, 437.2KiB (79.3KiB compressed)`; verification recompile finished 0.33s |
| 7 | builder battery | `cargo test -p frozen-duckdb-builder` | 0 | `35 passed; 0 failed` (0.52s) + 0 doc-tests |
| 8 | registry dry-run | `cargo publish --dry-run -p frozen-duckdb-builder` | 0 | index updated, packaged, verified, recompiled, `Uploading frozen-duckdb-builder v2.0.0` → `warning: aborting upload due to dry run` |

## Registry question: non-prerelease 2.0.0 with placeholder deps

**ACCEPTED at the local-verification layer.** `cargo publish --dry-run` ran the
full local pipeline at final version 2.0.0 — packaging, verification compile
from the packaged copy, and the upload staging (`Uploading … v2.0.0`) — and
aborted only at the explicit dry-run fence (`aborting upload due to dry run`).
Placeholder `SOURCE_PIN` is inert to the registry: it is a `const` inside
`src/lib.rs`, not a manifest field; the only manifest versions are the
external deps (anyhow/reqwest/tar/flate2/zip/tracing/tempfile) and they carry
normal semver requirements. The builder's deps are all external; the
**path+version dual-form deps live in sys (`build-dependencies`) and main**,
not builder, so this dry-run proves builder is fully self-contained at 2.0.0
(re-confirms PUBLISH-READINESS §6 publish order: builder → sys → main).

Note: `cargo publish --dry-run` DOES touch the network ("Updating crates.io
index" x2 — it resolves deps against the live index and checks auth-adjacent
metadata). The server-side version-collision / pre-release acceptance is the
one hop the dry-run cannot witness; that is operator authority on final day.

Unwitnessed hop: server-side acceptance of `2.0.0` itself.

## DEFECT found by this rehearsal: PB2's LICENSE-DUCKDB.md never landed in master

`git ls-files | grep -i license` on the canonical checkout returns only the
root `LICENSE` and `LICENSE-ENTERPRISE-DRAFT.md` — no `LICENSE-DUCKDB.md`
anywhere. PB2's receipt ("Verified by real `cargo package -l`", "coordinator
commits") describes an **uncommitted working-tree state**, and the file has
since been lost. Without it, the 2.0.0 packaged crate ships DuckDB's MPL 1.1
vendored headers with no attribution carrier (7 files, no license). This
rehearsal reconstructed the file in scratch from the PB2 receipt spec (MPL 1.1
notice naming pinned tag v1.5.5, scoped to vendored headers); the reconstruction
restores the 8-file package exactly (8 files, LICENSE-DUCKDB.md included) and its
byte count (737 B) differs from the receipt's 1187 B — the original text is
not recoverable from the receipt alone, only its structure.

**Operator action required**: re-add LICENSE-DUCKDB.md to all three crate dirs
and commit BEFORE the flip lands. Until then, publish readiness row 8
(PUBLISH-READINESS.md) and this license gap mean master is not final-day-ready.

## Standing

Mechanics proven at 2.0.0: flip → flags → lockfile → package → full builder
battery → registry dry-run, all exit 0 in one scratch subject. Remaining
unwitnessed hops: real engine rebuild at the real upstream v2.0.0 SHA,
server-side registry acceptance, GitHub Release assets, and the re-committed
license file. Overall: **ALIVE for mechanics; BLOCKED for final day on the
license re-commit + operator hops.**
