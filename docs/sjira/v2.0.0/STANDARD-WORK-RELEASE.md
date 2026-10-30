# STANDARD-WORK-RELEASE — release flip + cut operator sheet

Window: v2.0.0. Subject: /Users/sac/frozen-duckdb on merged `master`.
Sources (grounded, no step from memory):
`scripts/fastfollow_flip.sh`, `docs/sjira/v2.0.0/TAG-REHEARSAL.md`,
`docs/sjira/v2.0.0/PUBLISH-READINESS.md`, `scripts/verify_release_assets.sh`,
`scripts/release_checklist.sh` (DRYRUN mode).

Standard-work rules:

- Any deviation from expected output/exit = **andon**: STOP, record the
  verbatim output, escalate to the operator. Never improvise a fix mid-sheet.
- No step from memory. Every command is copied from this sheet (cmd blocks)
  or the cited script; if sheet and script disagree, the script wins and
  the sheet itself is an andon.
- Halts H1-H3 (below) require operator judgment; the sequence does not
  resume without an explicit operator decision.

Total estimated cycle time: **55-95 min** (gates ~25-40 min dominate; asset
dispatch wait 10-20 min; index polls 2-10 min each).

## Phase 0 — Preconditions (5 min)

```text
C0.1: grep -n 'const VERSION' crates/frozen-duckdb-builder/src/lib.rs
C0.2: git tag -l 'v2.0.0*'; git ls-remote --tags origin
C0.3: gh release list --repo seanchatmangpt/frozen-duckdb
C0.4: RELEASE_CHECKLIST_DRYRUN=1 bash scripts/release_checklist.sh
```

| Step | Cmd | Expected / exit | Max | Halt | Auth [A](#notes) |
|---|---|---|---|---|---|
| 0.1 Tag/pin alignment | C0.1 | `2.0.0` at lib.rs:15 | 1m | other version: andon | TAG-REH P0 |
| 0.2 No v2.0.0* tag | C0.2 | only v1.5.5, v26.10.8; 0 | 1m | tag exists: andon | TAG-REH P1 |
| 0.3 No existing release | C0.3 | empty, exit 0 | 1m | release exists: andon | TAG-REH P2 |
| 0.4 Checklist preview | C0.4 | 12 `[n/12]`, pass, 0 [a] | 1m | step missing: andon | checklist |

## Phase 1 — Flip (version bump, reversible) (10 min)

`fastfollow_flip.sh` is fail-closed: missing curated file or failed sed
exits nonzero; one `FLIP` line per applied edit.

```text
C1.1: bash scripts/fastfollow_flip.sh 2.0.0 <NEW_VER> <40-hex-upstream-SHA>
C1.2: make sync
C1.3: make verify
C1.4: python3 scripts/verify_publish_receipt.py
```

| Step | Cmd | Expected / exit | Max | Halt | Auth |
|---|---|---|---|---|---|
| 1.1 Declarative flip | C1.1 | FLIP lines; done; exit 0 | 5m | `FATAL:`/!=0: andon | flip.sh |
| 1.2 Regen projections | C1.2 | Makefile re-rendered, 0 | 5m | drift: andon | CLAUDE.md |
| 1.3 Full gate | C1.3 | gates GREEN, exit 0 | 20m | red: andon [b] | TAG-REH 1 |
| 1.4 Provenance gate | C1.4 | 0; 2 = C1.2, retry once | 5m | 1 persists: andon | TAG-REH 1 |

## Phase 2 — Tag + asset dispatch (20-40 min, first irreversible boundary)

Recoverable through 2.2. Do NOT enter Phase 3 until 2.4 is green.

```text
C2.1: git fetch origin && git checkout master && git pull
      git tag v2.0.0 <merge-sha>
      git push origin v2.0.0
C2.2: gh workflow run build-binaries.yml --ref v2.0.0 \
        -f version=v2.0.0
C2.3: gh run list --workflow build-binaries.yml --limit 1
      gh run watch <run-id>
C2.4: bash scripts/verify_release_assets.sh v2.0.0
```

| Step | Cmd | Expected / exit | Max | Halt | Auth |
|---|---|---|---|---|---|
| 2.1 Tag HEAD, push | C2.1 | push accepted | 5m | remote reject: [c] | TAG-REH 2 |
| 2.2 Dispatch asset build | C2.2 | run queued | 2m | guard fail: andon [d] | TAG-REH 3 |
| 2.3 Watch run | C2.3 | completed success | 30m | FAILED: no publish; andon [d] | TAG-REH 3 |
| 2.4 Verify assets | C2.4 | `PASS — 2/2` + sha256; exit 0 | 10m | exit 1/2: **H1** | assets.sh |

## Phase 3 — Publish chain (irreversible from 3.1 on) (15-30 min)

Hard order builder -> sys -> main (witnessed: `cargo package -p
frozen-duckdb-sys` exits 1 until builder is on the index — PUBLISH-READINESS
sec 1).

```text
C3.1: cargo publish -p frozen-duckdb-builder
C3.2: curl -s -H 'User-Agent: frozen-duckdb-release (operator cut)' \
        https://crates.io/api/v1/crates/frozen-duckdb-builder \
      | python3 -c \
        'import json,sys; print(json.load(sys.stdin)["crate"]["max_version"])'
C3.3: cargo publish -p frozen-duckdb-sys
C3.4: C3.2 with frozen-duckdb-sys in the URL
C3.5: cargo publish -p frozen-duckdb
```

| Step | Cmd | Expected / exit | Max | Halt | Auth |
|---|---|---|---|---|---|
| 3.1 Publish builder | C3.1 | published; 409 = done | 10m | auth fail: **H2** [e] | TAG-REH 4 |
| 3.2 Index wait: builder | C3.2 | NEW_VER | 10m/10s | absent: status, andon | TAG-REH 5 |
| 3.3 Publish sys | C3.3 | published; 409 = done | 10m | no-pkg = lag: retry [f] | TAG-REH 6 |
| 3.4 Index wait: sys | C3.4 | prints NEW_VER | 10m/10s | as 3.2 | TAG-REH 7 |
| 3.5 Publish main | C3.5 | published; 409 = done | 10m | lag: retry [f]; other: andon | TAG-REH 8 |

## Phase 4 — Close-out (10 min)

```text
C4.1: curl -s https://crates.io/api/v1/crates/frozen-duckdb \
      | python3 -c \
        'import json,sys; print(json.load(sys.stdin)["crate"]["max_version"])'
C4.2: for c in frozen-duckdb frozen-duckdb-sys frozen-duckdb-builder; do
        curl -s -o /dev/null -w "$c %{http_code}\n" \
          "https://index.crates.io/fr/oz/$c"; done
C4.3: cargo new t && cd t && cargo add frozen-duckdb@<NEW_VER>
      # + Connection::open query; no DYLD_LIBRARY_PATH;
      # ~/.frozen-duckdb/cache/<NEW_VER>-<arch>/ must auto-populate
C4.4: open https://docs.rs/crate/frozen-duckdb-sys/<NEW_VER>
      open https://docs.rs/crate/frozen-duckdb/<NEW_VER>
C4.5: python3 scripts/verify_publish_receipt.py
C4.6: bash scripts/check_sjira_standing.sh
```

| Step | Cmd | Expected / exit | Max | Halt | Auth |
|---|---|---|---|---|---|
| 4.1 Registry max_version | C4.1 | NEW_VER | 2m | stale: poll, never re-publish | TAG-REH 10 |
| 4.2 Sparse index x3 | C4.2 | `200` each | 2m | 404 after 10m: andon | TAG-REH 10 |
| 4.3 Clean smoke | C4.3 | builds; cache populated | 15m | 404/compile: **H3** | README |
| 4.4 docs.rs observed | C4.4 | builds render | obs | failure: no yank, fix fwd [g] | TAG-REH 9 |
| 4.5 Provenance re-run | C4.5 | exit 0; chain -> History | 5m | !=0 post-pub: andon | TAG-REH 10 |
| 4.6 Standing flips | C4.6 | clean, exit 0 [h] | 2m | mismatch: flip not landed | standing |

## Operator-judgment halts (andon; sequence stops here)

| Halt | Trigger | Why operator decides | Doctrine |
|---|---|---|---|
| **H1** | 2.4 exit 1/2, bad assets, red run | misorder risk [i] | TAG-REH 1.3, 3 |
| **H2** | 3.1+ publish fail or surprise | success permanent [j] | TAG-REH 3 |
| **H3** | identity doubt, or 4.3 breaks | not agent-checkable [k] | TAG-REH P5, 3 |

### Footnotes

[a] DRYRUN prints each step, executes nothing (release_checklist.sh:10-17).
[b] Fix forward on a branch, never cut on a red tree (TAG-REH step 1).
[c] Inspect `gh api .../git/refs/tags/v2.0.0`; continue at 2.2.
[d] Compare logs vs BINARIES-FAILURE-ANALYSIS.md; two prior runs FAILED
(36239600176, 37940771273).
[e] Preserve verbatim error; STOP; no improvised manifest edits mid-sequence.
[f] `no matching package named ... found` = index lag, never a manifest
defect (v1.5.5 dry-run battery): wait 60s, retry the exact same command,
<=10 min. Never edit manifests.
[g] docs.rs failure does not break user builds; record and patch forward.
[h] Flips cite this sheet's receipts; README known-open item included.
[i] Publishing before assets 404s every macOS first-build (lib.rs:78 asset
law). Operator: re-dispatch (`gh run rerun`) vs BLOCKED.
[j] Operator picks fix-forward versioning (e.g. 2.0.1-dev+). Never
retry-over a permanent step.
[k] v1.5.5 precedent: tag cut, assets unpublished -> local-compile fallback.

## Notes

- Generated projections (Makefile, verify-gates.sh, verify-evidence.ttl,
  gates/dry-run-publish*) are never hand-edited; `make sync` re-renders.
- Before 3.1 everything is recoverable (re-tag unpushed, re-dispatch, fix
  forward); from 3.1 on, never retry-over a permanent step.
- Pre-release `2.0.0` itself is unpublishable (crates.io hard gate);
  this sheet assumes Phase 1 already flipped to a publishable NEW_VER.

## See Also

`docs/sjira/v2.0.0/TAG-REHEARSAL.md` ·
`docs/sjira/v2.0.0/PUBLISH-READINESS.md` ·
`docs/sjira/v26.9.21/PUBLISH_RUNBOOK.md` ·
`scripts/fastfollow_flip.sh` · `scripts/verify_release_assets.sh` ·
`scripts/release_checklist.sh`
