---
dcterms:title: "PUBLISH_RUNBOOK — frozen-duckdb v1.5.5 operator cut sequence"
dcterms:created: "2026-09-21"
dcterms:isPartOf: "v26.9.21"
dcterms:provenance: "Rendered by TPUB (dry-run publish pre-flight) on feat/155-publish@7a271c7"
rdf:type: prov:Activity
aps:standing: https://w3id.org/chatman/aps#OPEN
---

# PUBLISH_RUNBOOK — frozen-duckdb v1.5.5 (operator cut sequence)

Every command below is an **operator cut** (権): agents hold no publish/merge/tag
authority. Execute in order; each step gates the next. crates.io publications
are **permanent** — a published version can be yanked but never deleted or
re-published — so any surprise between steps 4–8 means STOP and reassess, never
retry-over.

## Pre-flight verdict (TPUB, 2026-09-21, feat/155-publish@7a271c7)

### Name/ownership check — GO / GO / GO

| crate | crates.io API | sparse index | finding | verdict |
|-------|---------------|--------------|---------|---------|
| `frozen-duckdb` | 200 | `index.crates.io/fr/oz/frozen-duckdb` 200 | exists at **0.1.0** only (published 2025-10-07, sole owner **seanchatmangpt**, repository URL matches this repo, not yanked, 2,332 downloads; its dep on `duckdb ^1.4.0` is the old scheme this release replaces) | **GO** — legal same-owner version jump 0.1.0 → 1.5.5; no squatting, no takeover needed |
| `frozen-duckdb-sys` | 404 | 404 | name-free | **GO** |
| `frozen-duckdb-builder` | 404 | 404 | name-free | **GO** |

Identity gate: the publishing token (`cargo login`) must belong to
**seanchatmangpt** — it is the sole owner of `frozen-duckdb`, and sys/builder
will be created under whatever account runs step 4.

### Dry-run battery (evidence backing this runbook)

| gate | command | exit | outcome |
|------|---------|------|---------|
| builder dry-run | `cargo publish --dry-run --allow-dirty -p frozen-duckdb-builder` | 0 | PASS — full verification build from unpacked `target/package/frozen-duckdb-builder-1.5.5`, upload aborted by dry-run |
| sys dry-run | `cargo publish --dry-run --allow-dirty -p frozen-duckdb-sys` | 101 | `no matching package named frozen-duckdb-builder found` — ordering artifact (path+version dep resolves from registry during publish verification) |
| main dry-run | `cargo publish --dry-run --allow-dirty -p frozen-duckdb` | 101 | `no matching package named frozen-duckdb-sys found` — same class |
| sys/main packaging | `cargo package --allow-dirty --no-verify -p …` | 101 / 101 | even `--no-verify` packaging is index-blocked (resolution precedes packaging); `.crate` files for sys/main are **not producible pre-publication** |
| replica compensation | `cargo build` over publish-rewritten replicas + `[patch.crates-io]` | 0 | builder = real extracted packaged tree; sys/main = exact `cargo package --list` file sets with flattened manifests; all three compile together, replica CLI executes (`--help` exit 0) |
| docs.rs rehearsal | `HOME=/tmp/fd-docs-home-tpub DOCS_RS=1 RUSTUP_HOME=… CARGO_HOME=… cargo build -p frozen-duckdb-sys` | 0 | bindings generated from vendored headers, **no library linked**, isolated HOME left empty (zero `.frozen-duckdb` touches); plain build re-engages linking (exit 0) |
| package audits | `cargo package --list` ×3; sizes | 0 / 0 / 0 | see table below |

Package sizes (crates.io cap: **10 MiB** per `.crate`):

| crate | `.crate` | unpacked | largest files |
|-------|----------|----------|---------------|
| frozen-duckdb-builder | **426.9 KiB** (observed, 437,096 B) | 2.3 MiB | `vendored-headers/duckdb.hpp` 2.0 MiB, `duckdb.h` 244 KiB |
| frozen-duckdb-sys | not producible pre-publication | 748 KiB (replica-computed) | bundled bindgen modules 496 KiB + 209 KiB |
| frozen-duckdb | not producible pre-publication | 944 KiB (replica-computed) | `src/duckdb/vtab/arrow.rs` 80 KiB |

Readme gate: `README.md` present in all three package lists (sys/main resolve it
from the workspace root via `readme.workspace = true`). Vendored-headers gate:
`vendored-headers/duckdb.h` + `duckdb.hpp` in the builder list. No strays —
only `.rs/.h/.hpp/.md/.toml/.lock`, examples, and tests in the lists.

### Provenance gate (C5) — receipt-backed, run before every cut and at close-out

The pre-flight tables above are evidence, but the release path itself is now
receipt-backed: every `ggen sync run` appends a BLAKE3-chained, signed receipt
to `.ggen-v2/receipt-log.jsonl`, and the rendered receipt-contract census
(`generated/receipt_contract_matrix.json`, contract `ggen_sync_receipt` — facts:
pack `receipt-provenance-unification-pack` + `schema/domain.ttl`) types the
verdict. One command re-proves the chain and the shapes:

```
python3 scripts/verify_publish_receipt.py   # from repo root; exit 0 = GREEN
```

Expected (exit 0): `ggen receipt verify` verdict with `"valid": true` and
`"signature_valid": true` (chain + payload + graph hashes recomputed, Ed25519
signature checked), `ggen receipt history` replaying the full log, and all 7
`ggen_sync_receipt` fields conforming to the rendered census.

- **RED (exit 1)** → the provenance chain is broken (hash mismatch, bad
  signature, shape drift). Do not cut; treat like a red CI gate — fix forward
  on the feature branch, never on the release path.
- **Exit 2** → unusable, not broken: missing render (run `ggen sync run`) or
  ggen missing from PATH. `ggen sync run` must exit 0 before the gate can judge.
- Re-run at step 11 (close-out) so the post-publish History row carries the
  final head `chain_hash` — wave History entries then cite a verifiable
  receipt, not prose.

---

## Modeled dry-run gate (dry-run-publish-pack — 6 phases, C4)

The cut sequence below is a repeatable modeled gate: marketplace pack
`dry-run-publish-pack` v26.7.13 models this release's DoD as a 6-phase PDDL8
STRIPS domain (scope, generate, verify, manufacture, cleanroom, receipt).
`ggen sync run` (declared in `ggen.toml` `[ontology] imports` + bound in
`schema/domain.ttl`) renders the repo-side evidence templates:

| gate artifact | content |
|---------------|---------|
| `gates/dry-run-publish-gates.md` | phase × runbook-step alignment matrix + fence + per-phase evidence commands |
| `gates/dry-run-publish/dry-run-publish-domain.ttl` | merged 6-fragment PDDL8 domain + cycle problem (sole goal atom `dry-run-verified`) |
| `gates/dry-run-publish/dry-run-publish-shapes.ttl` | SHACL shapes for a filled dry-run evidence graph |

**DRY-RUN-OVERCLAIM FENCE** (pack law, binding for all release language in
this repo): the modeled gate covers LOCAL, REVERSIBLE dry-run verification
only. No atom anywhere in the modeled domain is named `published`,
`crates-io-uploaded`, or `release-complete`; the only terminal goal atom is
`dry-run-verified`. Gate-green means "evidence bundle complete for a human
go/no-go decision", never "release shipped" — the live cuts (steps 4–8) are
operator authority (権), outside the model.

Phase mapping (authoritative matrix with evidence commands:
`gates/dry-run-publish-gates.md`; re-render with `ggen sync run`):

| pack phase | gates these runbook steps |
|------------|---------------------------|
| 1 `DRY-RUN-SCOPE` | pre-flight name/ownership check; steps 1–2 (merge, tag) |
| 2 `DRY-RUN-GENERATE` | pre-cut repo law: `ggen sync run` exit 0 two-pass byte-identical; `cargo build --workspace` green; C5 provenance gate `python3 scripts/verify_publish_receipt.py` exit 0 |
| 3 `DRY-RUN-VERIFY` | step 3 (CI release assets green — hold the publish while missing) |
| 4 `DRY-RUN-MANUFACTURE` | TPUB dry-run battery; steps 4–8 packaging validation per member |
| 5 `DRY-RUN-CLEANROOM` | TPUB unpack-build + docs.rs rehearsal; step 11 clean-machine smoke |
| 6 `DRY-RUN-RECEIPT` | steps 9–11 (observe, reply, post-verify) + History/MILESTONE receipts; C5 provenance gate re-run (final head `chain_hash` into the History row); terminal: `dry-run-verified` |

Fence tripwire (must exit 0; the modeled domain carries the terminal atom and
none of the banned ones):

```
! grep -qE '\b(published|crates-io-uploaded|release-complete)\b' gates/dry-run-publish/dry-run-publish-domain.ttl && grep -q 'dry-run-verified' gates/dry-run-publish/dry-run-publish-domain.ttl
```

---

## The cut sequence

Repo: `seanchatmangpt/frozen-duckdb`. Run all `cargo`/`git` commands from the
repo root on `master` (post-merge). A `no matching package named … found` error
in steps 5–8 means **the crates.io index has not caught up** — it is never a
manifest defect; wait and retry (see step 5).

**Pre-cut repo law** (pack phase 2 — `DRY-RUN-GENERATE`; provenance gate is C5):
on the merged master, before step 2 where possible (tag verified content) and
always before step 4 (the first permanent cut): `ggen sync run` two-pass
byte-identical; `cargo build --workspace` green; `python3
scripts/verify_publish_receipt.py` exit 0. Any red → fix forward on a branch,
never cut (RED/exit-2 semantics: Provenance gate above).

### 1. Merge PR #3 — DONE

> **DONE** (wave-5 reconciliation, 2026-09-22): PR #3 is **MERGED** —
> `gh pr view 3 --repo seanchatmangpt/frozen-duckdb` → `state: MERGED`,
> `mergedAt: 2026-09-22T00:59:40Z`, `mergeCommit: a3a69e4`
> (feat/duckdb-1.5.5 → master; the branch tip c7b95ce "test: remove
> scheduler-dependent benchmark ordering" is inside the merge). Post-merge
> master carries 63070af (wave-5 ticket docs only — no product code).

```
gh pr view 3 --repo seanchatmangpt/frozen-duckdb   # MERGED: a3a69e4, 2026-09-22T00:59:40Z
```

- Pre-merge doctrine, retained for any re-merge scenario: **conflict at merge
  time** → resolve toward the branch whose ticket scope owns the file (scope
  table in each ticket), re-run the ci.yml gate suite locally, update the PR.
  **CI red on the PR** → do not merge; fix forward on the branch.
- The merge SHA is what step 2 tags: **a3a69e4**.

### 2. Tag v1.5.5 and push the tag

- **Tag target (wave-5 reconciliation): `a3a69e4`** — the PR #3 merge commit
  (step 1). Master tip has moved only by docs (63070af, sjira tickets), so the
  merge SHA remains the release-content boundary. v1.5.5 verified absent local
  + remote on 2026-09-22 (`git tag -l 'v1.5*'` → empty; `git ls-remote --tags
  origin` → empty) — no re-tag hazard exists yet.

```
git fetch origin && git checkout master && git pull
git tag v1.5.5 <merge-sha>
git push origin v1.5.5
```

- **`tag already exists`** → if it was never pushed and points at the wrong SHA:
  `git tag -d v1.5.5` and re-tag. If it is already on the remote: do not
  re-tag a pushed tag; move the workflow re-run route (step 3 failure) instead.
- **Push rejected** → the tag exists remotely from an earlier attempt: inspect
  `gh api repos/seanchatmangpt/frozen-duckdb/git/refs/tags/v1.5.5` and continue
  at step 3 rather than deleting remote history.

### 3. Verify CI release assets (gate — do not publish until green)

`build-binaries.yml` runs on the tag and uploads the two dylibs that
`ensure_binary()` downloads by exact name. Watch it:

```
gh run list --repo seanchatmangpt/frozen-duckdb --workflow build-binaries.yml --limit 3
gh run watch <run-id> --repo seanchatmangpt/frozen-duckdb
gh api repos/seanchatmangpt/frozen-duckdb/releases/tags/v1.5.5 --jq '.assets[].name'
```

Expected output, exactly two lines, exact names:

```
libduckdb_arm64.dylib
libduckdb_x86_64.dylib
```

- **Asset missing / wrong name** (`libfrozen_mega_*` leftovers count as wrong) →
  `gh run rerun <run-id>` (or `gh workflow run build-binaries.yml --ref v1.5.5`),
  wait for green, re-check. **HOLD the publish** while assets are missing: a
  crate published now would send every macOS first-build into a 404 download or
  a full source compile. Publishing before assets exist is the one irreversible
  step that can be wrongly ordered.
- **Workflow red** → inspect the log; if the runner cannot produce a dylib, the
  release is BLOCKED — fix forward with a new tag only if never published to
  crates.io yet (see step 2 note).

### 4. Publish `frozen-duckdb-builder`

```
cargo publish -p frozen-duckdb-builder
```

Expected: `Packaged 8 files, 2.3MiB (426.9KiB compressed)` then upload + a
crates.io validation pass, exit 0.

- **HTTP 409** (`crate version 1.5.5 is already uploaded`) → the step already
  succeeded (retried run). Verify with the poll in step 5 and continue. A
  version can never be re-published; if the *content* is wrong, yank
  (`cargo yank -p frozen-duckdb-builder --version 1.5.5` … only for genuine
  breakage) and cut 1.5.6 — never edit manifests to force a re-push.
- **`no matching package` error here** → not expected for builder (no internal
  deps); if seen, the token/auth is wrong — stop, check `cargo login` identity
  (must be seanchatmangpt).
- **Validation rejection** (e.g. size, metadata) → read the API error verbatim,
  stop, report. Do not improvise manifest edits mid-sequence.

### 5. Index wait for builder (poll, do not guess)

```
while [ "$(curl -s -H 'User-Agent: frozen-duckdb-release (operator cut)' \
  https://crates.io/api/v1/crates/frozen-duckdb-builder \
  | python3 -c 'import json,sys; print(json.load(sys.stdin)["crate"]["max_version"])')" != "1.5.5" ]; do
  sleep 10
done
```

Usually under a minute; give it up to ~10 minutes before treating it as
anomalous.

- **Still absent after ~10 min** → check
  `curl -s https://status.rust-lang.org` and the crates.io API
  (`https://crates.io/api/v1/crates/frozen-duckdb-builder/versions`) for a
  publishing-side delay; do not fire step 6 until the API shows 1.5.5.

### 6. Publish `frozen-duckdb-sys`

```
cargo publish -p frozen-duckdb-sys
```

- **`error: no matching package named frozen-duckdb-builder found`** → index
  lag; wait 60 s, retry the exact same command. Repeat up to ~10 min. This is
  the expected transient — the dry-run battery proved the manifest is correct.
- **409** → already published; verify via step-5-style poll for sys and continue.
- **Any other failure** → preserve the verbatim error, stop.

### 7. Index wait for sys

Same poll as step 5 with `frozen-duckdb-sys` in the URL.

### 8. Publish `frozen-duckdb`

```
cargo publish -p frozen-duckdb
```

Failure handling identical to step 6 (`no matching package named
frozen-duckdb-sys found` = index lag → wait + retry).

### 9. docs.rs builds (observe, do not gate the issue comment on it)

After sys and main land, docs.rs builds them on Linux. The DOCS_RS pattern is
rehearsed green on this exact head (see pre-flight table): sys generates
bindings from builder's vendored headers and emits no link directive, so no
`libduckdb.so` is needed.

- Check `https://docs.rs/crate/frozen-duckdb-sys/1.5.5` and
  `https://docs.rs/crate/frozen-duckdb/1.5.5` (build status + rendered docs).
- **docs.rs build failure** → user builds are unaffected (the failure is in the
  docs toolchain only; `docs.rs` metadata + rehearsal say the default config is
  correct). Do **not** yank. Read the docs.rs build log, record the failure in
  the milestone, fix forward in a patch release if it is ours.

### 10. Post the issue #1 reply

> **State note (wave-5 reconciliation, 2026-09-22):** issue #1 is **CLOSED** —
> auto-closed by the PR #3 merge (the PR title carries "closes #1"; verified
> `gh issue view 1` → `state: CLOSED`). The reply is still owed: commenting on
> a closed issue notifies the asker; reopening first is operator discretion.

```
gh issue view 1 --repo seanchatmangpt/frozen-duckdb   # CLOSED 2026-09-22 (auto via PR #3 merge): "Did you plan to release a version with DuckDB 1.5 ?"
# Post the BODY only (text after the `---` separator); the meta header + fence stay in the repo:
awk 'f; /^---$/{f=1}' docs/sjira/v26.9.21/ISSUE_1_COMMENT.md | sed 's/#{PR_NUMBER}/#3/g' > /tmp/issue-1-comment.md
gh issue comment 1 --repo seanchatmangpt/frozen-duckdb --body-file /tmp/issue-1-comment.md
```

- PR_NUMBER substitutes to **3** (the merge from step 1). Post as the repo
  owner account (the reply is written in that voice).

### 11. Post-publish verification (close-out)

- `curl -s https://crates.io/api/v1/crates/frozen-duckdb | python3 -c 'import json,sys; c=json.load(sys.stdin)["crate"]; print(c["max_version"])'` → `1.5.5`
- Sparse index shows all three: `index.crates.io/fr/oz/{frozen-duckdb,frozen-duckdb-sys,frozen-duckdb-builder}` → 200 each
- Clean-machine smoke (from PR_BODY.md checklist): `cargo new t && cargo add frozen-duckdb@1.5.5` + a `Connection::open` query, no `DYLD_LIBRARY_PATH`, `~/.frozen-duckdb/cache/v1.5.5-{arch}/` auto-populated, `ctypes` reports `v1.5.5`.
- Provenance gate re-run (C5): `python3 scripts/verify_publish_receipt.py` → exit 0; record the final head `chain_hash` in the History row (verifiable provenance, not prose).
- Update `docs/sjira/v26.9.21/MILESTONE.md` operator-cut checkboxes; append History rows.

## Abort doctrine

- Before step 4, everything is recoverable: re-merge, re-tag (unpushed),
  re-run workflows.
- From step 4 on, each success is permanent. If a later step is impossible
  (e.g. sys 409s with *different* content than intended — cannot happen for a
  first publish, but hold the rule), STOP: leave the milestone BLOCKED with the
  verbatim error; the operator decides fix-forward versioning (1.5.6+).
- Never yank except for genuine breakage; a yank does not delete and does not
  stop already-locked builds.
