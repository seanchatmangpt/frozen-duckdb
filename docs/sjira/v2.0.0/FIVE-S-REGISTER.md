# FIVE-S-REGISTER.md — 5S pass over repo root + top level (lane LSS7, 2026-10-19)

Scope: repo root, `scripts/`, `docs/explanation/`, `attic/`. GENESIS row check
applied before every removal/move. Lane mandate: `FIVE-S-REGISTER.md` (this
file) + disclosed deletions/moves of clearly-dead clutter; everything else is
proposal-only.

## 1. Sort (inventory + dead/duplicate triage)

Executed this pass:

| item | finding | action |
|---|---|---|
| `libnull.rlib` (root) | Untracked stray linker output; already covered by `.gitignore` `*.rlib`; zero references. | DELETED (safe: untracked). |
| `.cursorrules` (root) | 19 KB Cursor-era config; zero references anywhere in the repo; superseded by `CLAUDE.md`. | MOVED to `attic/cursorrules` (`git mv`, reversible). |
| `scripts/setup_workspace_caching.sh` | Dead: scaffolds a `frozen-workspace` workspace with a `frozen-arrow` member — a crate that no longer exists in the workspace; zero live references (only `scripts/README.md` listing + historical COMMIT-PLAN). No GENESIS row. | MOVED to `attic/setup_workspace_caching.sh` (`git mv`). |

Checked and kept (GENESIS-pinned or alive):

- `attic/gen_sjira_tickets*.sh` — GENESIS sources for `docs/sjira/v26.9.21/T*|TR*|C*|G*` rows. Stay.
- `docs/explanation/related.md` — GENESIS `gap` row (`lineage:7f604e5`). Stays; not "corrections", it is reconciliation debt on the ledger, exactly where it belongs.
- `vendors/duckdb-rs` — empty on disk but it is the GENESIS-pinned gitlink (upstream pin rows check it); emptiness = uninitialized submodule, not dead.
- `scripts/use_prebuilt_duckdb.sh` — ALIVE: invoked by `scripts/build_frozen_duckdb.sh` (lines 417, 435). Not dead.
- `scripts/README_gap_detection.md`, `scripts/rg_sweep.sh`, `scripts/open_gaps.sh` — self-documenting kept/reconciled set (TR6/G5); no GENESIS rows but live references from `scripts/README.md`.
- No `__pycache__/` or `*.pyc` anywhere in the working tree (checked, not just trusted the ignore rule) — Shine is already clean.

## 2. Set-in-order (what belongs where)

- `attic/` = dead-but-historical scripts/configs with provenance value; candidates proposed below, not moved (shared surfaces / plausible reuse).
- `scripts/` = live build/gate/verify tooling only; gap-detection suite documented by its own README.
- `docs/archive/` = prose snapshots; no root files currently qualify (root docs — `CONTEXT.md`, `FORWARD_DEPLOYMENT.md`, `KNOWN_DIVERGENCES.md`, `CAMPAIGN-INDEX.md`, `HANDWRITTEN.md` — are all referenced by other docs; moves would break links).
- `scripts/smoke_go.go`, `scripts/smoke_go_simple.go`, `scripts/build_go_smoketest.sh`, `scripts/create_arrow_cache.sh`, `scripts/setup_flock_ollama.sh`, `scripts/test_comprehensive_flock.sh` — go/flock/arrow probes with no live caller; PROPOSED for `attic/` move (batch decision by coordinator).

## 3. Shine

- Clean: no `__pycache__`, no `*.pyc`, no stray `*.rlib` (after this pass), no empty untracked dirs beyond the gitlink.
- `target-gap8` etc. are not Shine — they are the Sort finding below.

## 4. Standardize — proposed clean-root allowlist

Everything else (tracked) is cargo/ggen/CI-essential or workspace config:

```
.actrc .cargo/ .claude/ .doc-surface.toml .ggen/ .ggen-v2/ .gitattributes
.github/ .gitignore .tool-versions
Cargo.toml Cargo.lock Makefile justfile ggen.toml ggen.lock config.toml
LICENSE LICENSE-ENTERPRISE-DRAFT.md
README.md CHANGELOG.md QUICKSTART.md TROUBLESHOOTING.md MIGRATION_GUIDE.md
CLAUDE.md CONTEXT.md FORWARD_DEPLOYMENT.md HANDWRITTEN.md
CAMPAIGN-INDEX.md KNOWN_DIVERGENCES.md
crates/ docs/ gates/ generated/ prebuilt/ schema/ scripts/ templates/
test-dependency/ test-validation/ tests/ vendors/ warehouse-sim/ attic/
```

Not on the allowlist: any `target-*/` build dir, `*.rlib`, lane leases
(`.cargo-laneT1/`, `target-lane*/`, `target-integration/`) — these are leases
deleted at integration per the same-checkout-fanout cleanup law.

## 5. Sustain (enforcement today + gaps)

Enforced today: `make genesis-check` parses GENESIS rows (existence/pin
checks); `.gitignore` covers `*.rlib`, `__pycache__/`, `*.pyc`, `target/`,
`target-lane*/`, `target-integration/`, `.cargo-laneT1/`, `.ggen*/`.

Gaps found (proposals, not acted on):

1. **Tracked build artifacts (largest finding):** `target-gap8/`,
   `target-cg2/`, `target-dt3/`, `target-fx1/`, `target-gap9/` contain ~1164
   tracked files (`git ls-files target-*` excluding `target-lane*/`), including
   cargo `.fingerprint` state that shows up as per-run noise in `git status`.
   Remedy: `git rm -r --cached target-gap8 target-cg2 target-dt3 target-fx1
   target-gap9` + add `target-*/` to `.gitignore` (one line replaces the
   current narrower patterns). Coordinator decision — touches many paths.
2. `.gitignore` names `target-lane*/` and `target-integration/` but not the
   general `target-*/` family — every future probe-lane build root will
   re-create gap 1 unless the pattern is generalized.
3. `scripts/README.md` line 12-13 still lists `setup_workspace_caching.sh`
   (now moved to `attic/`) — one-line stale-listing follow-up for the
   scripts/README owner.
4. Root `docs` sprawl is bounded but `CONTEXT.md`/`FORWARD_DEPLOYMENT.md` are
   referenced by historical tickets only; fine to keep, no action.
