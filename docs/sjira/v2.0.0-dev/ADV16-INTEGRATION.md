# ADV16 — Integration Prep (for coordinator)

Date: 2026-10-19 17:36 PDT. Census basis: `git status --porcelain`, `git diff
--stat`, mtime scans. Lane ADV16 owns this file and `/tmp/adv16_msg.txt` only.

## 1. Commit plan

Single integration commit, message already drafted at `/tmp/adv16_msg.txt`
(`git commit -F /tmp/adv16_msg.txt` — never `-m`).

### Commit (29 paths)

Modified (9):

| path | lane/classification |
|---|---|
| `Cargo.toml` | PB12 rust-version 1.87 + PB3 metadata (documentation field drop) |
| `crates/frozen-duckdb/Cargo.toml` | PB10-F2 serde_json feature + CLI required-features; rust-version.workspace |
| `crates/frozen-duckdb-builder/Cargo.toml` | PB12/PB3 rust-version.workspace, documentation drop |
| `crates/frozen-duckdb-sys/Cargo.toml` | PB12/PB3 + sys description rewrite |
| `test-dependency/Cargo.toml` | PB1 `publish = false` |
| `test-validation/Cargo.toml` | PB1 `publish = false` |
| `crates/frozen-duckdb/tests/gate_v01_latency.rs` | LSS5 median-of-3 (GAGE-RR-COURTS rule 3) + V4 cross-file JSON receipt |
| `crates/frozen-duckdb/tests/value_audit_datadog.rs` | CG1 per-PID scratch paths (file-lock-collision fix) |
| `scripts/verify-evidence.ttl` | GENERATED — regenerate, see 3a |

Untracked (20): `.github/workflows/publish-crates.yml`, `scripts/publish_chain.sh`,
`scripts/cleanroom/registry_consumer.sh`, `scripts/doctor/cdp_smoke.mjs`,
`scripts/build_wasm_mvp.sh`, 3x `LICENSE-DUCKDB.md`, `docs/sjira/v2.0.0-dev/PB3-metadata.md`,
`PUBLISH-DECISION.md`, `receipts/PB{2,4,5,6,8,9,10,11,12,13,14}-*.md`.

Diff scope sanity: `git diff --stat` = 9 files, +92/−47 — no lane touched files
outside its declared ownership; no deletions; no lane target dirs (`target-lane*`)
in status.

## 2. BLOCKERS — do not commit yet

> STATUS 2026-10-19 (CL12): B1 RESOLVED — `optional = true` landed on the
> serde_json dep, manifest parses, ADV8 witnessed. B2 SETTLED — all lanes
> reported handoff. B3 REGENERATED — `make verify` ran 5/5 green post-fixes,
> `scripts/verify-evidence.ttl` re-stamped from the green battery.

### B1. Workspace manifest is BROKEN (compile freeze)

`crates/frozen-duckdb/Cargo.toml` sets `serde_json = ["dep:serde_json"]` but the
`serde_json` dependency is NOT `optional = true`. Every cargo command in the
workspace fails manifest parse — `cargo fmt`, `build`, `test` are all down:

```
error: failed to parse manifest at crates/frozen-duckdb/Cargo.toml
  feature `serde_json` includes `dep:serde_json`, but `serde_json` is not an optional dependency
  A non-optional dependency of the same name is defined; consider adding `optional = true`
```

Note the PB10 comment block itself says "the serde_json DEPENDENCY is
unconditional" — the feature line contradicts its own comment. Owner (PB10 lane
or coordinator under compile-freeze SLA) must either add `optional = true` (and
re-verify CLI + cfg(test) sites) or revert the feature to `serde_json = []`.
Fix and rerun `cargo fmt --check` + `cargo build` before committing.

### B2. A lane is STILL WRITING

mtime evidence (checked twice, 17:35:15 → 17:36:24):

- `scripts/doctor/cdp_smoke.mjs` grew 7643 → 7800 bytes, mtime 17:35:47 (<45 s
  before second check) — kaizen-doctor lane in flight
- `scripts/cleanroom/registry_consumer.sh` mtime 17:34:37
- `scripts/build_wasm_mvp.sh` mtime 17:33:50

PB-lane outputs (PB2–PB14, PUBLISH-DECISION) have been quiet since 17:27 and
look stable. GAP1b/GAP11/GAP15/DOCKER1/ERRC-C5/AD3/AD6-owned surfaces above are
NOT confirmed quiescent. Coordinator: wait for ≥10 min mtime quiet on
`scripts/doctor/`, `scripts/cleanroom/`, `scripts/build_wasm_mvp.sh` (or explicit
lane handoff) before staging.

### B3. Last recorded gate battery FAILED

`scripts/verify-evidence.ttl` (regenerated 2026-10-20T00:34:30Z, head adbd260)
records `fmt-check exitCode 1` and `test-workspace exitCode 101` (build pass +
clippy pass + test-validation pass). The generated file as it stands receipts a
FAILING run — do not commit this version; regenerate after B1 is fixed and the
battery is green.

## 3. Must NOT be committed (scratch / regenerate)

- `/tmp/value_audit/` artifacts, `val001.json`/`val001.meta`,
  `gate_v01_receipt.*`, `fleet00*.json/meta`, `rev00*.json/meta` under
  `docs/sjira/v2.0.0-dev/receipts/` — these are runtime court outputs already
  tracked at 14:58; the new-run JSONL/db scratch stays in /tmp (the CG1 fix now
  writes per-PID). Nothing new in status maps to scratch, but do NOT `git add -A`
  blindly — add the 29 enumerated paths explicitly.
- `scripts/verify-evidence.ttl` only in regenerated (green-battery) form —
  never hand-edit.
- Coordinator deletes any lane `CARGO_TARGET_DIR=target-lane*` dirs before the
  final commit (none present in status now).

## 4. Sequence for coordinator

1. Wait for B2 quiescence (or lane handoff).
2. Fix B1 (owner: PB10/coordinator), rerun `cargo fmt --check` + `cargo build`.
3. Full battery green → `make verify` (regenerates verify-evidence.ttl, B3).
4. `git add` the explicit path list; confirm `git status --porcelain` clean of
   scratch; `git commit -F /tmp/adv16_msg.txt`.
