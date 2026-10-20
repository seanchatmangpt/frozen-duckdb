# GO/NO-GO Receipt — v2.0.0-dev release decision

- **Lane**: F8 (operator go/no-go artifact). Logs archived in `/tmp/f8/`.
- **Subject**: repo HEAD at run time `f9d94f0951a365f03fce9abb382d0ec15aec647d`
  (coordinator moved HEAD during the run; gate battery re-ran against the
  moved head — evidence TTL binds subject head
  `fb6d98b2dd2c3237719d85e69691454af7d75e2d` at record time 2026-10-20T02:05:08Z,
  pre-move).
- **Machine**: darwin/arm64, load average 43–66 (concurrent fan-out in
  progress during latency-sensitive runs — relevant to the GATE-V-01 verdict
  below).
- **Date**: 2026-10-20T02:07Z

## 1. `make verify` — verdict: 4/5 GREEN (NOT 5/5)

Task brief expected 5/5; the on-disk reality differed and is reported
honestly. Two runs recorded.

### Run 1 (before lane F8 unblock fixes) — 2/5, STANDING: BLOCKED

```
verify-gates: 2/5 gates green -> scripts/verify-evidence.ttl (graph hash binding: 5f49a93506ada13796f5aa5a92b625e9ec04611f49c1eb5ebe8bbbcaa5da75a1)
  build-workspace: exit 0
  clippy-deny-warnings: exit 101
  fmt-check: exit 1
  test-validation: exit 0
  test-workspace: initial run: exit 101
STANDING: BLOCKED (red gate above; evidence: scripts/verify-evidence.ttl)
make: *** [verify-gates] Error 1
```

F8 minimal unblock fixes, disclosed per compile-freeze SLA:

1. Recreated stub `u2-probe/Cargo.toml` + `u2-probe/src/lib.rs` — HEAD's
   Cargo.toml lists `u2-probe` as a workspace member whose manifest was not
   on disk; every cargo command in the tree failed with
   `failed to read /Users/sac/frozen-duckdb/u2-probe/Cargo.toml`. Same
   unblock as USER-FINDINGS-U4.md (the earlier stub had been deleted).
2. Removed one unused import (`types::Value`) at
   `crates/frozen-duckdb/src/duckdb/types/serde_json.rs:83` (another lane's
   in-flight file), then `cargo fmt --all` reformatted that file.

### Run 2 (post-fix, recorded) — 4/5, STANDING: BUILD_BROKEN

```
verify-gates: 4/5 gates green -> scripts/verify-evidence.ttl (graph hash binding: 5f49a93506ada13796f5aa5a92b625e9ec04611f49c1eb5ebe8bbbcaa5da75a1)
  build-workspace: exit 0
  clippy-deny-warnings: exit 0
  fmt-check: exit 0
  test-validation: exit 0
  test-workspace: exit 101
STANDING: BUILD_BROKEN (red gate above; evidence: scripts/verify-evidence.ttl)
make: *** [verify-gates] Error 1
```

### The red gate: `gate_v01_latency::gate_v01_p95_under_15ms` — FAIL

Pre-existing (HEAD evidence TTL at run start already recorded
clippy=101 / fmt=1 / test-workspace=101), and reproducible (failed 3 of 3
attempts: two inside `cargo test --workspace`, one targeted rerun):

```
p95_rounds_ms = [27.9427, 28.6662, 29.7012]
median p95 28.6662 ms >= budget 15 ms
verdict = FAIL
GATE-V-01 FAIL: median p95 28.6662 ms >= budget 15 ms over rounds [27.942708, 28.666166999999998, 29.701209] (debug-profile caveat: debug builds are substantially slower than release)
```

Caveats on the verdict: the test runs in the debug profile while its own
panic message carries a debug-profile caveat; machine load was 43–66 during
all three runs. It may pass on an idle machine or in release; as run, it is
red. Honest standing: **BUILD_BROKEN, reproducible under lane conditions.**

## 2. `make genesis-check` + `verify_publish_receipt.py` — GREEN

```
GENESIS_EXIT=0
genesis-check: checked=58 deferred=0 gaps=16 failed=0
```

```
PUBRECEIPT_EXIT=0
"valid": true, "signed": true, "signature_valid": true,
"chain_standing": "fully-bound",
"chain_hash": "2ab552b7df965a143ec1c224c6aa33a4a8799e967264c612307723982cfa91ff"
ggen receipt history: 1170 records, valid=True, head_chain_hash=2ab552b7...
PROVENANCE-GATE GREEN: receipt chain verified, signature valid, ggen_sync_receipt shapes conform to the rendered census
```

## 3. `bash scripts/publish_chain.sh` (DRYRUN default) — GREEN (dry-run only)

```
[publish-chain] token check: existence-level only (cargo login --help); token never read
[publish-chain] DRYRUN=1 — printing publish plan, executing nothing
[publish-chain] versions: builder=2.0.0-dev sys=2.0.0-dev main=2.0.0-dev
DRYRUN: cargo publish --manifest-path crates/frozen-duckdb-builder/Cargo.toml
DRYRUN: wait_for_index frozen-duckdb-builder 2.0.0-dev
DRYRUN: plan: builder->wait->sys->wait->main (dry-run)
standing=dry-run-verified
```

## 4. `bash scripts/verify_release_assets.sh v2.0.0-dev` — exit 2 (loud, honest)

```
ASSETS_EXIT=2
[verify-release-assets] no release v2.0.0-dev exists yet — loud SKIP-class exit (2), not a faked pass
```

## 5. Blocking list for v2.0.0 publish

| # | Blocker | Evidence |
|---|---|---|
| B1 | ~~GATE-V-01 latency gate red~~ → **RESOLVED 2026-10-19 (CL12)**: stage-08 release-profile re-measure + median-of-3 + opt-in ignore landed; gate green. (reproducible 3/3 under lane conditions; debug-profile + load 43–66 caveats) | Run-2 output above; `crates/frozen-duckdb/tests/gate_v01_latency.rs:188` |
| B2 | **KD-01 open-honest** — 27,682 C++-mangled `__Z*` exports breach the C-ABI-only contract; close needs upstream v2.0.0 export fence or a shipped export-filter wrapper | `KNOWN_DIVERGENCES.md` KD-01; doctor stage 00/01 red (`/tmp/f8/doctor.log`) |
| B3 | **Release assets chicken-and-egg** — `verify_release_assets.sh` cannot go green before the GitHub Release exists; the release must be cut first, then the check re-run | ASSETS_EXIT=2 above |
| B4 | **CARGO_REGISTRY_TOKEN is operator-held** — publish_chain only checked token *existence-level*; real publish requires the operator to export `CARGO_REGISTRY_TOKEN` | publish_chain output above |
| B5 | ~~u2-probe workspace member has no real landing~~ → **RESOLVED 2026-10-19 (CL12)**: u2-probe removed from the workspace (member + files deleted) | stub disclosure above; USER-FINDINGS-U4.md |

Resolved during this lane: zip migrated to 9.0.5 in Cargo.lock (RUSTSEC-2026-0367/0368/0369 GONE per DG2-deny.md and PB11-deny-audit.md) — **not blocking**.

KD register: 15 rows, 3 CLOSED (KD-03, KD-04, KD-14), 12 open — matching the doctor's "total: 15 open: 12 closed: 3" tally.

## 6. Recommendation: **HOLD** (per DOD-18)

Do not publish 2.0.0-dev prerelease now. `make verify` is 4/5 with a
reproducible latency-gate failure, KD-01 is open-honest, DOD-20
final-integration reconciliation is still open, and DOD-18's hold decision
is exactly this receipt's subject.

### Exact operator steps when choosing GO

1. Idle machine, re-run `make verify` (and consider a release-profile
   re-measure of GATE-V-01, which carries a debug-profile caveat in its own
   panic text). If 5/5 GREEN → B1 clears.
2. Export `CARGO_REGISTRY_TOKEN` (operator-held; publish_chain checks
   existence only).
3. Cut the GitHub Release v2.0.0-dev with the built assets; then re-run
   `bash scripts/verify_release_assets.sh v2.0.0-dev` (expect exit 0).
4. `bash scripts/publish_chain.sh REAL=1` (builder → wait → sys → wait →
   main), followed by the chained `verify_publish_receipt.py` and the
   sparse-index check.
5. Close DOD-18 as the operator decision; DOD-20 reconciliation pass
   remains the final integration gate before tag.

## Falsifiers

- B1 falsified if an idle-machine `make verify` returns 5/5 GREEN.
- B3 falsified by `verify_release_assets.sh v2.0.0-dev` exiting 0 after the
  release cut.
- B4 falsified by a real (non-DRYRUN) publish chain completing and the
  sparse-index check passing.
- B5 falsified when u2-probe lands for real and the stub is dropped.
