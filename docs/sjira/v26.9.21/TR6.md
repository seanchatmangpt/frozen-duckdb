---
id: TR6
dcterms:title: "Repo hygiene + 帳 ledger"
dcterms:created: "2026-09-21"
dcterms:isPartOf: "v26.9.21"
rdf:type: oslc_cm:ChangeRequest, earl:TestRequirement
aps:standing: https://w3id.org/chatman/aps#OPEN
milestone: frozen-duckdb v1.5.5 wave 3 (hardening, pre-publish)
worktree: /Users/sac/frozen-duckdb-wt/r6
branch: feat/155-r6
---

# TR6 — Repo hygiene + 帳 ledger

## Scope (files you may change)

```
root rot files; scripts/{smoke*,fake_guard.sh,demo*}; HANDWRITTEN.md (new)
```

Anything outside scope: do not edit — record BLOCKED with the reason instead.

## Work order


Context: kcura-era rot at root (A6 audit): demo-dependency-issue.sh, test_architecture.rs, test_ffi_validation.rs, datasets/, test_datasets/, scripts/ smoke_go/smoke_node/smoke_py/fake_guard.sh family and ci_gate.sh/ci_gates.sh referencing non-existent crates/kcura-* paths. Triage each: still-useful → fix and keep; dead → git rm with the removal reasoned in the commit message; uncertain → keep + mark with a header comment. Then create HANDWRITTEN.md (帳): table path | semantic element | missing capability | intended owner pack | date — seed it with this wave's known agent-authored-by-hand rows (tickets T2A..TR6 non-generated repairs; the repo has no ggen pack projection).
DoD: [ ] every triaged file has a keep/fix/remove decision with evidence  [ ] repo still builds: cargo build --workspace exit 0  [ ] HANDWRITTEN.md exists with seeded rows  [ ] committed

## Contract (all tickets)

- Work ONLY in `/Users/sac/frozen-duckdb-wt/r6` on `feat/155-r6`. NEVER touch the main checkout /Users/sac/frozen-duckdb or sibling worktrees. Never push, never touch remotes, never publish.
- Atomic commits, conventional messages, on your branch only.
- Verify before claiming: preserve command + exit. A gate you did not run is UNKNOWN, not green.
- Append every transition to the History table below (the one sanctioned write outside your worktree), then STOP — the coordinator merges.

## History

| ts | standing | branch+SHA | gates+exits | remaining |
|----|----------|------------|-------------|-----------|
| 2026-09-21 | OPEN | feat/155-r6@c55d4f2 | not started | all of DoD |
| 2026-09-21 | ALIVE (DoD met on branch; STOP for coordinator merge) | feat/155-r6@2e0231c | cargo build --workspace → exit 0 (38.7s, links v1.5.5-arm64 cache dylib); bash -n on 5 edited scripts → 0; falsification probes kept as evidence: `cargo build -p test-dependency` → 101 "did not match any packages" (root demo script dead), scripts/ci_gates.sh live → exit 1 at own Gate 1 + fails bash -n:48, scripts/test_ffi_simple.sh live → exit 1 "DuckDB library not found" (central-cache drift + setup_env.sh $0 bug → kept-uncertain header). Decisions: REMOVED demo-dependency-issue.sh, test_architecture.rs, test_ffi_validation.rs, datasets/, test_datasets/, scripts/{ci_gate.sh,ci_gates.sh,fake_guard.sh,smoke_all.sh,smoke.py,smoke_py.py,smoke_node.mjs,demo_ffi_validation.sh}; KEPT+STATUS-HEADER test_ffi_simple.sh, run_ffi_validation.sh, smoke_go.go, smoke_go_simple.go, validate_prod_build.sh (stale CRATE_VERSION=0.1.0), validate_frozen_approach.sh, lib/config.sh (dir-wide note); NEW HANDWRITTEN.md 帳 ledger (8 seeded rows, TR3/TR4/T6/T7/wave-2/TR6). Cross-ref caution resolved: run_ffi_validation.sh does NOT reference root test_ffi_validation.rs. BLOCKED (outside scope table): scripts/scan_fakes.sh + scan_fakes_core_team.sh + kcura-config{,.example}.yaml — only remaining lib/ consumers, natural next paydown. Handoffs: T6 reconcile FFI-script lib resolution + setup_env.sh $0→BASH_SOURCE defect; T4 reconcile scripts/README_gap_detection.md (documents removed gates). 偽-residuals: smoke_go*.go not compiled (needs lib-resolution repair, T6); validate_* consumer flows not executed (crates.io-published-crate scope, TR8); smoke_go*.go gofmt misalignment pre-existing, untouched. Operator wrote: nothing — 5 agent commits (87,211 deletions; 41 ledgered header lines + HANDWRITTEN.md). | none in DoD |
