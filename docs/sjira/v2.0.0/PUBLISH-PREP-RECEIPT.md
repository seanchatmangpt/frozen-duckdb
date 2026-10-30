# PUBLISH-PREP-RECEIPT — Wave 15 consolidation (lane PB19)

Date: 2026-10-19 · Repo: /Users/sac/frozen-duckdb @ master `adbd260`
(refreshed in place by lane CL8: the formerly-uncommitted PB1/PB2
artifacts and the MSRV resolution have since landed — see O2/O3/O8,
B2-B4).
Lane PB19 owns this file and the Wave 15 section of `_LANES.md`. Every
claim below cites its lane receipt; lanes without landed receipts are
marked UNKNOWN, never inferred. Consolidated after ~30 min polling;
12 of 18 lanes had landed (PB1/PB7/PB15-PB18 outstanding at write time).

## Per-lane status (receipt-cited)

| lane | status | receipt | one-line finding |
|---|---|---|---|
| PB1 | ALIVE | `receipts/PB8-flip-publish.md` step 3; committed at HEAD | RESOLVED since consolidation: `publish = false` landed and committed in `test-dependency/Cargo.toml:5` and `test-validation/Cargo.toml:5` (git-tracked; `git grep "publish = false" HEAD` exits 0). Effects witnessed: cargo publish refuses on both test crates. |
| PB2 | ALIVE | `receipts/PB2-license.md` | LICENSE-DUCKDB.md in all 3 crates, packaged + witnessed via `cargo package -l`; MPL 1.1 text corrected by PB13 (below) |
| PB3 | ALIVE | `PB3-metadata.md` | metadata audit; sys `description` diff ready for coordinator; MSRV floor 1.77 (c-string literal); no `rust_version` set yet |
| PB4 | ALIVE | `receipts/PB4-registry-rehearsal.md` | builder `.crate` + consumer quickstart ALIVE; KD-15 rpath reproduced out-of-tree; consumer link-arg fix proven |
| PB5 | ALIVE | `receipts/PB5-docsr.md` | `DOCS_RS=1` builds exit 0 all 3 crates; 15 main + 3 sys rustdoc warnings; none docs.rs-fatal (docs.rs fails on build errors, not warnings) |
| PB6 | ALIVE | `receipts/PB6-leakscan.md` | zero real leaks post-wave; ship-list growth benign; AD8 exclusions hold (19 of 32 tests ship) |
| PB7 | UNKNOWN | none | NO-RECEIPT-LANDED as of 2026-10-19 18:05 |
| PB8 | ALIVE | `receipts/PB8-flip-publish.md` | flip→flags→lockfile→package→35/35 builder tests→`publish --dry-run` all exit 0 at 2.0.0 in scratch. Former DEFECT (LICENSE-DUCKDB.md absent from master) RESOLVED: files landed and committed since. |
| PB9 | ALIVE | `receipts/PB9-docs-surface.md` | landing pages real, no stubs; 15+3+0 warnings; README 3 relative links 404 on docs.rs; no LICENSE/CHANGELOG packaged |
| PB10 | ALIVE | `receipts/PB10-features.md` | serde_json feature IS load-bearing (refutes FIN6 checklist item 3 note); F2/F3 registered divergences, loud |
| PB11 | ALIVE | `receipts/PB11-deny-audit.md` | zip 0.6.6 (RUSTSEC-2026-0369, CVSS 9.1 critical) BLOCKING for publish; yoke-derive one-line bump; h2/reqwest-0.12 wave NON-BLOCKING |
| PB12 | ALIVE (closed) | `receipts/PB12-msrv.md` + `receipts/ADV10-msrv-proof.md` | falsifier CLOSED via ADV10 scratch: first-party floor 1.87.0 confirmed (`cargo +1.87.0 check --workspace --all-targets` exit 0; builder release 35/35 on 1.87.0). Superseded by F4 resolution (§7 of PB12 receipt): `rust-version = "1.88"` landed at workspace `Cargo.toml` — every `time` ≥0.3.46 requires rustc 1.88 and zip ≥7.4 (incl. all 9.x, landed 9.0.5) requires `time ^0.3.47`, so the 1.87-compatible pin set is unsatisfiable under the committed graph. |
| PB13 | PASS (informational) | `receipts/PB13-mpl-compliance.md` | upstream DuckDB v1.5.5 LICENSE is MIT, not MPL 1.1 (live curl, exit 0); headers byte-exact vs build tree; LICENSE-DUCKDB.md content stale |
| PB14 | ALIVE | `receipts/PB14-ownership.md` | name ownership CLEAR: `frozen-duckdb` owned by seanchatmangpt (gh auth matches); `-builder`/`-sys` unregistered, first-publish wins |
| PB15-PB18 | UNKNOWN | none | NO-RECEIPT-LANDED as of 2026-10-19 18:05 |

## Consolidated READY-AT-FLIP checklist

Merges FIN6 (`PUBLISH-READINESS.md` READY-AT-FLIP table), the PB wave
receipts above, and the P-lane design findings (P3/P5/P6/P10/P12/P13/
P14 — all design-stage surfaces, none publish-blocking; see final
section). Standing per item, each cited to its receipt.

### READY (receipted, no action)

- R1 Manifest metadata complete on all 3 crates — FIN6 item 1, PB3.
- R2 docs.rs survival path (`DOCS_RS` in `build.rs:30/77/95`) — FIN6
  item 2, re-executed by PB5, exit 0 all crates.
- R3 docs.rs builds: all 3 crates build+doc exit 0 under `DOCS_RS=1`;
  warnings are non-fatal — PB5, PB9.
- R4 links=duckdb single-provider; builder `.crate` self-contained
  (vendored headers; 8-9 files) — FIN6 items 4/6, PB4, PB6, PB8.
- R5 Publish order proof builder→sys→main (sys/main packaging
  expected-fail on registry absence) — FIN6 item 7, PB4, PB8.
- R6 Zero secrets/paths in ship lists post-wave — PB6.
- R7 serde_json feature load-bearing and correctly commented — PB10
  (supersedes FIN6 item 3 "fix comment" action).
- R8 crates.io name ownership: all 3 names controlled/claimable by
  seanchatmangpt — PB14.
- R9 Landing pages render real crate docs (no stubs) — PB9.
- R10 docs.rs landing-page quality items (README 404 links, no
  LICENSE/CHANGELOG in package, 15 broken intra-doc links) — PB9:
  quality debt, NOT registry-blocking; listed under OPERATOR as
  fix-before-or-after items.
- R11 Consumer quickstart via `[patch.crates-io]` proven, with the
  KD-15 rpath consumer link-arg — PB4.
- R12 Flip mechanics proven end-to-end at 2.0.0 in scratch: flip →
  publish flags → lockfile → package → 35/35 builder tests →
  `publish --dry-run` exit 0 — PB8.
- R13 MSRV: `rust-version = "1.88"` LANDED at workspace
  `Cargo.toml:30`. First-party floor 1.87 confirmed by ADV10 scratch
  (`receipts/ADV10-msrv-proof.md`: `cargo +1.87.0 check --workspace
  --all-targets` exit 0, builder 35/35 on 1.87.0), then evidence-forced
  to 1.88 by F4 (PB12 receipt §7): zip ≥7.4 (landed 9.0.5 for B2)
  requires `time ^0.3.47`; every `time` ≥0.3.46 declares
  `rust_version = 1.88`, so 1.87 pins are unsatisfiable. Falsifier
  CLOSED. 1.88 verified: `cargo check --workspace` exit 0, builder
  46/46 on default toolchain.

### OPERATOR items (authority or action outside lane scope)

- O1 Version flip: `scripts/fastfollow_flip.sh 2.0.0 <NEW>
  <40-hex upstream SHA>` — FIN6 item 5; mechanics pre-proven PB8.
- O2 RESOLVED: PB1 `publish = false` flags and `crates/*/LICENSE-DUCKDB.md`
  are committed at HEAD (`git grep "publish = false" HEAD` and
  `git ls-files crates/*/LICENSE-DUCKDB.md` both exit 0). PB8's former
  DEFECT (license absent from master) is closed.
- O3 RESOLVED: LICENSE-DUCKDB.md x3 regenerated with corrected MIT
  attribution (PB13 refuted the MPL 1.1 premise via live curl of the
  upstream v1.5.5 LICENSE; MIT at pin
  `d54cbbe3df17e8e5ce68312f483b4155836112aa`), committed, and ships
  via `cargo package -l` — witnessed exit 0 in `receipts/PB2-license.md`.
- O4 Real publish: `cargo publish -p` builder → sys → main (server
  side: version acceptance, checksums, index publication — the
  unwitnessed hop PB8 names as operator authority).
- O5 GitHub Release assets for the final tag — FIN6 item 9; v1.5.5
  precedent is tag-cut/release-missing; also gates the builder
  download path (PB4 delta row 8).
- O6 Post-flip gate re-run: `make verify` + `cargo publish --dry-run
  -p frozen-duckdb-builder` on the flipped tree — FIN6 item 10.
- O7 Advisory hygiene: `cargo update -p yoke-derive` (one-liner);
  remaining waves: zip "0.6"→"9" RESOLVED (zip 9.0.5 in `Cargo.lock`,
  RUSTSEC-2026-0367/0368/0369 GONE per `receipts/DG2-deny.md` and
  GO-NOGO-RECEIPT line 119), reqwest "0.12" wave (NON-BLOCKING) — PB11.
- O8 RESOLVED: `rust-version = "1.88"` applied (workspace
  `Cargo.toml:30`), evidence-based per PB12/ADV10/F4 (see R13).
- O9 Docs-quality fixes at owner discretion (PB9 fixes-needed 1-6):
  `documentation` metadata → docs.rs URL; README absolute links;
  per-crate CHANGELOG/LICENSE packaging; 15 intra-doc link fixes;
  sys bindgen warning suppression at wrapper.h source.
- O10 PB10 registered divergences: optional serde_json dep (F2) and
  sys `bundled = []` WHY comment (F3) — owner decisions, not gates.
- O11 Publish-day confirmation itself (the human "yes") and crate
  token (`cargo login` state) — see authority statement below.

### BLOCKED items

- B1 `2.0.0` is unpublishable as-is — crates.io rejects
  pre-release `dev` versions; the flip is a registry hard gate.
  Receipt: FIN6 §5, witnessed by PB8 (dry-run passed only at
  flipped 2.0.0). Resolves via O1.
- B2 RESOLVED: zip migrated to 9.0.5 in `Cargo.lock` with builder API
  migration landed — RUSTSEC-2026-0367/0368/0369 GONE per
  `receipts/DG2-deny.md` re-scan; PB11's BLOCKING classification no
  longer applies (GO-NOGO-RECEIPT line 119). Side effect: the zip 9
  bump forced `time ^0.3.47`, which drove the rust-version 1.88
  decision (F4, see R13).
- B3 RESOLVED: `crates/*/LICENSE-DUCKDB.md` committed at HEAD with
  corrected MIT attribution (PB13 premise refuted; see O3) — the
  attribution carrier ships via `cargo package -l` (PB2 receipt).
- B4 RESOLVED for PB1 (flags landed + committed; effects witnessed
  via PB8 step 3 and live `cargo publish` refusal on both test
  crates). Remaining unlanded PB receipts (PB7, PB15-PB18) are
  UNKNOWN — no ALIVE claim is made for them.

## Publish-blocking list: 2.0.0 vs 2.0.0-final

- `2.0.0` (prerelease): former blockers O2+O3 (license) and B2
  (zip advisory) are all RESOLVED — license committed with corrected
  MIT text and ships in-package (PB2), zip 9.0.5 landed with RUSTSEC
  advisories cleared (DG2 re-scan). Name ownership (PB14), metadata,
  docs.rs, leak scan, flip mechanics, rust-version 1.88 are all
  receipted READY. No known 2.0.0 blocker remains from this
  consolidation.
- `2.0.0-final`: everything above plus O1 (flip), O5 (release
  assets), O6 (post-flip gate), O4 (real publishes), and the
  operator hops PB8 lists: real engine rebuild at the real upstream
  v2.0.0 SHA, server-side registry acceptance.

## Operator-authority statement

The following are exclusively operator-authority; no lane and no
receipt can discharge them:

1. crates.io API token state (`cargo login`) — never held by lanes.
2. The version-flip execution (O1) and its target version/SHA.
3. Publish dispatch and the publish confirmation itself (O4) — the
   server-side acceptance hop no dry-run can witness (PB8).
4. GitHub Release asset publication (O5).
5. The final go/no-go on B2 (zip) for the dev prerelease.

## P-lane design findings (not publish-blocking)

P3/P5/P6/P10/P12/P13/P14 (`docs/design/` Wave 4 PRD lanes) are
design-stage surfaces and do not gate publish: P3 boundary-abi
WASM exports PROPOSED/UNVERIFIED; P5 wasm bench arm UNVERIFIED
until FR-01; P6 binary-size-budget (no open flags found); P10
view-expansion BLOCKED(engine) until test-validation; P12
sync-digest DESIGN/UNVERIFIED, no implementation; P13 ingest
wasm leg UNVERIFIED until FR-01; P14 acceptance-criteria wasm
courts FR-01/FR-02-blocked. These carry their standing in their
own docs; cited here so the checklist is total, not to block.
