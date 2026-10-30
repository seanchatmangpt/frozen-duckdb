---
dcterms:title: "TAG-REHEARSAL — frozen-duckdb 2.0.0 tag + release cut sequence (dry run)"
dcterms:created: "2026-10-19"
dcterms:isPartOf: "v2.0.0"
dcterms:provenance: "Lane FIN4 dry rehearsal, no git state commands executed; derives from v26.9.21/PUBLISH_RUNBOOK.md + build-binaries.yml@79e803e"
rdf:type: prov:Activity
aps:standing: PARTIAL_ALIVE
---

# TAG-REHEARSAL — frozen-duckdb 2.0.0 tag + release cut sequence (dry run)

Source of truth: `docs/sjira/v26.9.21/PUBLISH_RUNBOOK.md` (the 1.5.5 operator
cut sequence) + `.github/workflows/build-binaries.yml` + the README
known-open item (release assets BLOCKED on publication). No cut was executed —
this is a rehearsal. Per the runbook DRY-RUN-OVERCLAIM fence: gate-green means
"evidence bundle complete for a human go/no-go decision", never "shipped".

## 0. Mode drift from the 1.5.5 sequence

Two differences from v1.5.5, both verified on disk at HEAD 79e803e:

1. **Tag name.** Crate version is `2.0.0`
   (crates/frozen-duckdb-builder/src/lib.rs:15,
   `const VERSION: &str = "2.0.0";`). The release tag is therefore
   **`v2.0.0`**, not `v2.0.0`. The build-binaries workflow_dispatch guard
   hardcodes `BUILDER_VERSION="2.0.0"` (yml line 67) and its required
   input defaults to `v2.0.0` (yml line 12); any other input value fails
   the run loudly (yml lines 60-73).
2. **Asset publication is dispatch-capable, and the prior record is red.**
   Assets come from build-binaries.yml (fires on `push: tags: v*` and on
   workflow_dispatch). The last two runs of this workflow both FAILED:
   run 36239600176 (tag v1.5.5, 2026-09-26) and run 37940771273 (tag
   v26.10.8, 2026-10-09). No GitHub Release exists today
   (`gh release list` → empty). The 2.0.0 plan of record is
   `gh workflow run build-binaries.yml --ref v2.0.0` with
   `version=v2.0.0`, then HOLD the crates.io publishes until the release
   carries exactly two assets.

## 1. The 2.0.0 cut sequence (rehearsed, not executed)

Run all `git`/`cargo` commands from repo root on merged `master`. A
`no matching package named … found` during steps 6-8 is index lag, never a
manifest defect (proved by the v1.5.5 dry-run battery): wait and retry, do not
edit manifests.

### Step 1 — Pre-cut repo law

```
ggen sync run && ggen sync run            # two-pass, byte-identical
cargo build --workspace
python3 scripts/verify_publish_receipt.py  # exit 0 = provenance GREEN
make verify                                # full gate
```

Any red → fix forward on a branch, never cut on a red tree.

### Step 2 — Tag v2.0.0 on the merge HEAD and push

```
git fetch origin && git checkout master && git pull
git tag v2.0.0 <merge-sha>
git push origin v2.0.0
```

- `tag already exists` locally and unpushed → `git tag -d v2.0.0`, re-tag.
- Push rejected (exists remotely) → do NOT delete remote history; inspect
  `gh api repos/seanchatmangpt/frozen-duckdb/git/refs/tags/v2.0.0` and
  continue at step 3.

### Step 3 — Release assets via dispatch (GATE — hold everything until green)

```
gh workflow run build-binaries.yml --ref v2.0.0 -f version=v2.0.0
gh run list --repo seanchatmangpt/frozen-duckdb --workflow build-binaries.yml --limit 1
gh run watch <run-id> --repo seanchatmangpt/frozen-duckdb
gh api repos/seanchatmangpt/frozen-duckdb/releases/tags/v2.0.0 --jq '.assets[].name'
```

Expected: exactly two lines —

```
libduckdb_arm64.dylib
libduckdb_x86_64.dylib
```

(`ensure_binary()` downloads by exact name; see lib.rs:78
`libduckdb_{arch}.{dylib|so}` asset law.) Any other asset name
(`libfrozen_mega_*` leftovers count as wrong) → `gh run rerun <run-id>`.
**HOLD the publishes while assets are missing** — publishing first sends every
macOS first-build into a 404 download or a full source compile.

### Step 4 — Publish `frozen-duckdb-builder` (first permanent step)

```
cargo publish -p frozen-duckdb-builder
```

- HTTP 409 → already published; poll step 5 and continue.
- Token/auth failure → stop; identity must be seanchatmangpt.
- Validation rejection → read the API error verbatim, STOP, report. No
  improvised manifest edits mid-sequence.

### Step 5 — Index wait for builder (poll, ≤10 min)

```
while [ "$(curl -s -H 'User-Agent: frozen-duckdb-release (operator cut)' \
  https://crates.io/api/v1/crates/frozen-duckdb-builder \
  | python3 -c 'import json,sys; print(json.load(sys.stdin)["crate"]["max_version"])')" != "2.0.0" ]; do
  sleep 10
done
```

Still absent after ~10 min → check `https://status.rust-lang.org` and the
versions API before firing step 6.

### Step 6 — Publish `frozen-duckdb-sys`

```
cargo publish -p frozen-duckdb-sys
```

`no matching package named frozen-duckdb-builder found` → index lag; wait 60 s,
retry the exact same command (≤10 min). 409 → already published. Any other
failure → preserve the verbatim error, stop.

### Step 7 — Index wait for sys

Same poll as step 5 with `frozen-duckdb-sys` in the URL.

### Step 8 — Publish `frozen-duckdb`

```
cargo publish -p frozen-duckdb
```

Failure handling identical to step 6 (`no matching package named
frozen-duckdb-sys found` = index lag).

### Step 9 — docs.rs index/build wait (observe, not a gate)

Check `https://docs.rs/crate/frozen-duckdb-sys/2.0.0` and
`https://docs.rs/crate/frozen-duckdb/2.0.0`. A docs.rs build failure does
not break user builds — do not yank; record and fix forward in a patch.

### Step 10 — Post-publish verification (close-out)

- `curl -s https://crates.io/api/v1/crates/frozen-duckdb | python3 -c 'import json,sys; print(json.load(sys.stdin)["crate"]["max_version"])'` → `2.0.0`
- Sparse index 200 ×3: `index.crates.io/fr/oz/{frozen-duckdb,frozen-duckdb-sys,frozen-duckdb-builder}`
- Clean-machine smoke: `cargo new t && cargo add frozen-duckdb@2.0.0` +
  `Connection::open` query, no `DYLD_LIBRARY_PATH`,
  `~/.frozen-duckdb/cache/v2.0.0-{arch}/` auto-populated.
- Provenance gate re-run: `python3 scripts/verify_publish_receipt.py` → exit 0;
  final head `chain_hash` into the History row.
- **Fast-follow flip**: flip the README known-open item ("Prebuilt release
  assets for 2.0.0 — BLOCKED on tag/release publication") and any
  `aps:standing:` flips only after step 3's asset check is green; each flip
  cites this receipt + `scripts/check_sjira_standing.sh` clean.

## 2. Precondition table (dry-checked 2026-10-19, HEAD 79e803e)

| # | precondition | verification command | observed | verdict |
|---|---|---|---|---|
| P0 | tag/pin alignment: tag `v2.0.0`, builder `2.0.0`, guard `BUILDER_VERSION="2.0.0"` | `grep -n 'const VERSION' crates/frozen-duckdb-builder/src/lib.rs`; yml lines 12/67 | all three read `2.0.0` | **READY** |
| P1 | no `v2.0.0*` tag exists | `git tag -l 'v2.0.0*'` → empty; `git ls-remote --tags origin` → only `v1.5.5`, `v26.10.8` | confirmed, no re-tag hazard | **READY** |
| P2 | no existing 2.0.0 release | `gh release list --repo seanchatmangpt/frozen-duckdb` → empty, exit 0 | confirmed, clean slate | **READY** |
| P3 | `make verify` GREEN on merged master | `make verify` | not run in this rehearsal (read-only lane) | **OPERATOR** |
| P4 | assets published by build-binaries dispatch on the tag | `gh workflow run build-binaries.yml --ref v2.0.0 -f version=v2.0.0` + asset check | last two runs FAILED (36239600176, 37940771273); no release exists | **BLOCKED** until a green dispatch; inspect prior-failure logs before the cut |
| P5 | crates.io login identity = seanchatmangpt | no agent-readable check (`cargo login` is a write) | unverifiable from here | **OPERATOR** |
| P6 | fast-follow flip (README known-open item + standing flips) | `scripts/check_sjira_standing.sh` clean post-cut | gated on P4 green | **BLOCKED** (downstream of P4) |

## 3. Abort doctrine (carried from PUBLISH_RUNBOOK.md)

- **Before step 4 (first `cargo publish`), everything is recoverable**: re-tag
  an unpushed tag, re-run/re-dispatch workflows, fix forward. A red `make
  verify`, a red provenance gate (exit 1 = broken chain, exit 2 = unusable —
  run `ggen sync run` first), or a red CI halts before any tag push.
- **From step 4 on, each success is permanent** (crates.io versions can be
  yanked, never deleted or re-published). Any surprise in steps 4-8 → STOP,
  preserve the verbatim error, leave the milestone BLOCKED; the operator
  decides fix-forward versioning (2.0.1-dev+). Never retry-over a permanent
  step, never improvise manifest edits mid-sequence.
- **HOLD rules that halt the cut specifically**:
  - release assets missing or wrong-named (step 3) — the one irreversible
    step that can be wrongly ordered;
  - provenance gate RED (exit 1) — treat like red CI;
  - identity doubt on the publishing token (must be seanchatmangpt);
  - workflow red with no reproducible dylib — release is BLOCKED until fixed
    forward with a never-published tag.
- Never yank except genuine breakage; a yank deletes nothing and stops no
  already-locked builds.

## See Also

`docs/sjira/v26.9.21/PUBLISH_RUNBOOK.md` · `.github/workflows/build-binaries.yml` ·
`docs/sjira/v2.0.0/_LANES.md`
