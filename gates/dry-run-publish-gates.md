# Dry-Run Publish Gate — frozen-duckdb v1.5.5 (dry-run-publish-pack composition)

Rendered consequence of `schema/domain.ttl` (gate bindings) + the marketplace
pack `dry-run-publish-pack` v26.7.13 ontology (loaded via `ggen.toml`
[ontology] imports). EDIT THE SOURCES, NEVER THIS FILE. Join key across the
two graphs: `skos:notation` (`DRY-RUN-*`).

> **DRY-RUN-OVERCLAIM FENCE** (pack law — packs/dry-run-publish-pack/ontology.ttl):
> this gate models LOCAL, REVERSIBLE dry-run verification only. No atom in the
> modeled domain is named 'published', 'crates-io-uploaded', or
> 'release-complete'; the only terminal goal atom is 'dry-run-verified'.
> Gate-green means an evidence bundle is complete for a human go/no-go
> decision — never that a release shipped. The live cuts (merge, tag push,
> `cargo publish` without `--dry-run`) are operator authority (権) and stay
> outside the modeled domain.

| # | Pack phase | Repo gate binding | PUBLISH_RUNBOOK steps it gates | Evidence command (exit 0 required) |
|---|------------|-------------------|--------------------------------|-------------------------------------|
| 1 | `DRY-RUN-SCOPE` — 1: Release Scope & Identity | Gate 1 — Release Scope & Identity | TPUB pre-flight name/ownership check; runbook steps 1-2 (merge PR #3; tag v1.5.5) | `gh pr view 3 --repo seanchatmangpt/frozen-duckdb (OPEN); git tag --list v1.5.5 (bound); git status --porcelain (empty)` |
| 2 | `DRY-RUN-GENERATE` — 2: Deterministic Generation | Gate 2 — Deterministic Generation | pre-cut repo law: ggen sync two-pass idempotence + cargo build --workspace (this wave: C4 gates/ render) | `ggen sync run (exit 0, second pass reports unchanged: content identical); cargo build --workspace (exit 0)` |
| 3 | `DRY-RUN-VERIFY` — 3: Verification Ladder | Gate 3 — Verification Ladder | runbook step 3 (verify CI release assets — do not publish until green) | `gh run watch <run-id> --repo seanchatmangpt/frozen-duckdb; gh api repos/seanchatmangpt/frozen-duckdb/releases/tags/v1.5.5 --jq '.assets[].name' (exactly two expected names)` |
| 4 | `DRY-RUN-MANUFACTURE` — 4: Package Manufacture | Gate 4 — Package Manufacture | TPUB dry-run battery per member; runbook steps 4-8 (operator cuts builder -> index wait -> sys -> index wait -> main, packaging validation at each) | `cargo publish --dry-run --allow-dirty -p frozen-duckdb-builder (exit 0); cargo package --list -p frozen-duckdb-sys; cargo package --list -p frozen-duckdb (file-set audits)` |
| 5 | `DRY-RUN-CLEANROOM` — 5: Clean-Room Verification | Gate 5 — Clean-Room Verification | TPUB unpack-build + docs.rs rehearsal; runbook step 11 clean-machine smoke (cargo new + cargo add frozen-duckdb@1.5.5 + Connection::open) | `HOME=/tmp/fd-docs-home-tpub DOCS_RS=1 cargo build -p frozen-duckdb-sys (exit 0); cargo new /tmp/smoke + cargo add frozen-duckdb@1.5.5 (query exit 0)` |
| 6 | `DRY-RUN-RECEIPT` — 6: Receipt & Replay (terminal — dry-run verified; no external mutation ever occurs) | Gate 6 — Receipt & Replay (terminal: dry-run-verified) | runbook steps 9-11 (docs.rs builds — observe; post the issue #1 reply; post-publish verification) + History/MILESTONE receipt appends | `curl -s https://crates.io/api/v1/crates/frozen-duckdb (max_version 1.5.5); sparse index 200 x3; grep -c dry-run-verified gates/dry-run-publish/dry-run-publish-domain.ttl (>=1)` |

## The modeled cycle

PDDL8 STRIPS domain + merged cycle problem: `gates/dry-run-publish/dry-run-publish-domain.ttl`.
SHACL shapes for a filled dry-run evidence graph: `gates/dry-run-publish/dry-run-publish-shapes.ttl`.
Subject: `RC-FROZEN-DUCKDB-V155` (schema/domain.ttl fd:gateObject); publish set:
the three `PKG-FROZEN-DUCKDB-*` members in dependency order (fd:publishMember*).

Init atom: `scope-engaged` (phase 1).
Sole GOAL atom: `dry-run-verified` (phase 6 terminal) — the fence requires it to read unambiguously as a dry-run result, never as a release claim.

## Executing the modeled gate (repeatable)

```
ggen sync run    # render these artifacts (exit 0); a second pass must be byte-identical
```

Then walk phases 1-6 against PUBLISH_RUNBOOK.md with the evidence commands in
the table above, preserving command + exit for each (証). Fence tripwire
(structural, must exit 0 — the modeled domain carries the real terminal atom
and none of the banned ones):

```
! grep -qE '\b(published|crates-io-uploaded|release-complete)\b' gates/dry-run-publish/dry-run-publish-domain.ttl && grep -q 'dry-run-verified' gates/dry-run-publish/dry-run-publish-domain.ttl
```
