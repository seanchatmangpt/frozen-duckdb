---
window: v2.0.0
lane: PB18
subject: crates.io publish decision — publish-now vs hold-for-final
standing: RECOMMENDATION (operator decides)
date: 2026-10-19
---

# PUBLISH-DECISION — Option A (publish now) vs Option B (hold)

Consolidates the two publish strategies staged by this campaign into one
decision doc. Statuses below are read from disk at /Users/sac/frozen-duckdb
@ master (adbd260 + working tree), 2026-10-19.

## Option A — DEV-PRERELEASE PUBLISH NOW

Publish the 3-crate chain to crates.io today at a dev/prerelease version,
so consumers get a versioned index target instead of a path/git dep.

**Pro:**
- Consumers can `frozen-duckdb = "<ver>"` today; PB4 proved the full
  consumer path works out-of-tree (quickstart run, exit 0, 45.10s;
  `receipts/PB4-registry-rehearsal.md`).
- Exercising the publish chain for real retires the "first real publish"
  risk class early (order, index propagation, token).

**Con:**
- Permanent: crates.io has no delete; yank is not deletion (still
  downloadable via `cargo` with `=` pinning; breaks nothing already
  resolving, but the version remains in the index namespace forever).
- **Version-form precondition is DISPUTED on disk.** The tasking claims
  "crates.io accepts prereleases"; PUBLISH-READINESS.md section 5 (FIN6)
  states `2.0.0` is "unpublishable (registry hard gate)". Counter-
  evidence in-repo: PB4 packaged builder at `2.0.0` with exit 0
  (`cargo package --allow-dirty --offline -p frozen-duckdb-builder` →
  "Packaged 8 files, 437.1KiB"). `cargo package` success is necessary,
  not sufficient — `cargo publish --dry-run` against the live registry
  must be run BEFORE trusting either claim. No such live dry-run receipt
  exists on disk today.
- Assets still missing for the download path: the GitHub Release v1.5.5
  assets were never published (tag cut, release missing — README.md
  "Known open item"), so consumers hit the local-compile fallback:
  30-90 min first build (PB4 consumer used the local cache dylib; a
  fresh consumer has no cache).
- KD-01 mangled exports ship (21 upstream mangled symbols, DOD-15
  BLOCKED-upstream; `KD-CLOSURE-PLAN.md` KD-01 — "Blocks release: No").
- KD-15 rpath divergence reproduced out-of-tree (PB4 first-run loader
  failure `dyld: Library not loaded: @rpath/libduckdb.dylib`).

**Steps:**
1. Resolve the version-form question: run
   `cargo publish --dry-run -p frozen-duckdb-builder` (live registry).
   If `2.0.0` is refused, the A version becomes a flip
   (`fastfollow_flip.sh`) — at which point A collapses into B.
2. Add `publish = false` to test-dependency and test-validation
   (readiness checklist item 8, PUBLISH-READINESS.md).
3. Publish in order builder → sys → main
   (`scripts/publish_chain.sh`, DRYRUN=1 first, then PUBLISH_CONFIRM).
4. Announce with the compile-fallback caveat in RELEASE-DRAFT.

**Risks:** permanent index entry at a dev version; KD-01/KD-15 visible
to every consumer; first-build time reflects badly pre-assets.

**Reversibility:** yank only; never delete. Yank stops new resolution
but leaves the version addressable.

## Option B — HOLD FOR FINAL (current standing, DOD-18)

DOD-18 (`DEFINITION-OF-DONE.md`, "crates.io hold decision", PROPOSED,
evidence: operator decision pending + `gates/dry-run-publish/`) holds
all actuation until upstream ships v2.0.0 tag + release assets.
(Note: the tasking cited "DOD-17"; on disk the crates.io hold item is
DOD-18. DOD-17 is the warehouse-sim economics court. Cited as-found.)

Everything is already staged behind:

- `scripts/fastfollow_flip.sh OLD NEW NEW_SHA` — mechanical version
  flip (9-step edit set, fail-closed, proven in the 2026-10-21
  rehearsal; header receipt: /tmp/ff-rehearsal/RECEIPT.md).
- `scripts/publish_chain.sh` — order orchestrator builder → sys →
  main with bounded index polls (POLL-STEP classified, no bare
  sleeps); dry-run-verified terminal standing by construction.
- `.github/workflows/publish-crates.yml` — DISPATCH-ONLY,
  environment-gated ("release" env, CARGO_REGISTRY_TOKEN required,
  fail-closed REFUSED[NO_TOKEN]); dry-run input defaults true.
- `TAG-REHEARSAL.md` (FIN4): full step-by-step runbook; explicit
  instruction at line 36/89: "HOLD the publishes while assets are
  missing".
- `scripts/verify_publish_receipt.py` — provenance gate (exit 0 =
  GREEN).

**Preconditions (all operator-held):**
- Upstream duckdb v2.0.0 tag cut + Release assets published (fixes
  both the 30-90 min first build and the v1.5.5 asset precedent).
- Version flip to a final (or publishable) version via
  `fastfollow_flip.sh`, then `make verify` + `cargo publish
  --dry-run` re-run (readiness checklist items 5, 10).
- `publish = false` on both test crates (item 8).
- Coordinator integrates the current concurrent-lane working tree
  (item 11).

**Steps:** flip → gates → `publish_chain.sh` dry-run → operator
dispatch of publish-crates.yml with `confirm = publish-<ver>` →
verify_publish_receipt.py → release notes + announcement.

**Risks:** consumers stay on path/git deps longer; the staged chain
itself is unexercised against the live registry until then
(mitigated: chain dry-run modes exist and PB4 exercised the
package/consumer surfaces).

**Reversibility:** maximal — nothing permanent happens until the
operator actuates; the whole flip set is rehearsed and scripted.

## Comparison table

| Dimension | A: publish now | B: hold for final |
|---|---|---|
| Precondition | live-registry prerelease check (unproven) | upstream tag + assets (external) |
| Version form | 2.0.0 or flip-first (disputed) | final version via scripted flip |
| Consumer UX | versioned target today | path/git dep until final |
| First build | 30-90 min local compile (no assets) | asset download path |
| Known divergences | KD-01, KD-15 ship | closed or registered at final |
| Scripts | publish_chain.sh | flip + chain + workflow + runbook |
| Reversibility | yank-only, permanent entry | nothing actuated yet |
| Standing on disk | PARTIAL_ALIVE (no live dry-run receipt) | PROPOSED (DOD-18, staged) |

## RECOMMENDATION

**Recommend (B) HOLD FOR FINAL**, consistent with the standing DOD-18
hold and the TAG-REHEARSAL instruction to hold publishes while assets
are missing. This is a recommendation, not fiat: the operator owns
actuation (publish-crates.yml is dispatch-only by design, and both
A and B terminate in an operator-confirmed publish). If the operator
overrides toward A, step A.1 (live `cargo publish --dry-run`) is the
mandatory first move — the prerelease-acceptance premise has no
receipt on disk and one contrary claim (FIN6).

## Receipt index

- `receipts/PB4-registry-rehearsal.md` — package exit 0 / expected
  order failures / consumer out-of-tree run + KD-15 loader failure.
- `receipts/PB5-docsr.md` — docs.rs standing ALIVE for all three,
  publish-blocked only by the known hold.
- `PUBLISH-READINESS.md` (FIN6) — manifest/docs.rs/features/links
  PASS rows + prerelease-registry claim + READY-AT-FLIP checklist.
- `TAG-REHEARSAL.md` (FIN4) — hold-while-assets-missing runbook.
- `DEFINITION-OF-DONE.md` — DOD-15 (KD-01 BLOCKED-upstream), DOD-18
  (crates.io hold, PROPOSED, operator pending).
- `KD-CLOSURE-PLAN.md` — KD-01 "Blocks release: No"; KD register
  open/closed counts (11 open, KD-03/04/14 closed per
  CAMPAIGN-RECEIPT.md).
- `scripts/fastfollow_flip.sh`, `scripts/publish_chain.sh`,
  `scripts/verify_publish_receipt.py`,
  `.github/workflows/publish-crates.yml` — staged actuation set.
