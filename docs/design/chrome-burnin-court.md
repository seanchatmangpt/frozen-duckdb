# chrome-burnin-court.md

Chicago-school verification protocol for the WASM/OPFS build of frozen-duckdb.
Continuous burn-in runs inside `chrome-headless-shell`, driven over the Chrome
DevTools Protocol (CDP). Every court observes the real browser substrate: real
OPFS storage, real QuotaManager, real worker isolate. No Node polyfills, no
synthetic file handles, no in-memory shims standing in for the filesystem.

## Governing law

- **No-sleep doctrine** (`~/.claude/rules/no-sleep-testing.md`): every
  wait is a bounded poll on a real observable with a deadline and a typed
  failure. Zero fixed `sleep(N)` anywhere in the harness.
- **Testing Chicago-style** (`~/.claude/rules/testing-chicago-style.md`):
  real collaborators by default. The browser, the OPFS mount, and the
  QuotaManager are the collaborators; mocking any of them invalidates the
  court. Assert on final observable state (checksums, CDP metrics), never on
  call counts.
- **Shim-court pattern** (fail-closed, non-vacuity): each court names the
  mutation that would make it fail. A court that still passes under its named
  mutation is VACUOUS and exits 1 as such.

## Burn-in loop shape

One CDP session per iteration against a fresh
`chrome-headless-shell --headless --user-data-dir=<tmp>` instance:

1. Launch browser, open target page hosting the worker.
2. Attach via CDP (`Target.attachToTarget`, flat session).
3. Load the burn-in workload (seed table, transactional churn, checkpoint).
4. Apply the court under test (crash injection, memory poll, lock race).
5. Re-attach or reload, assert post-state from real OPFS contents.
6. Tear down the browser process; the temp profile dir is deleted in the
   same iteration (lane-build-lease rule: cleanup in the same unit).

Iteration count and workload scale are harness config, not assertions.
Assertions are always on state after the court fires.

## Court 1: forced Page.crash mid-transaction, WAL replay integrity

**Procedure**: begin a transaction, write a known deterministic dataset
(seedable PRNG, fixed row count) across >= 2 tables, force `Page.crash` via
CDP before commit and again at a second injection point after commit but
before checkpoint. Relaunch a fresh browser against the same profile dir
(OPFS persists per-origin in the profile), reopen the database, and assert:

- WAL replay completes without error.
- Full-table checksums (ordered canonical serialization, e.g. BLAKE3 over
  `(ORDER BY pk)` row bytes) equal either the pre-transaction state or the
  post-commit state — never a third value.
- `PRAGMA integrity_check` equivalent (duckdb `CHECKPOINT` + full scan)
  reports zero corruption.
- No orphaned/zero-length files in the OPFS root beyond the WAL and the
  database file.

**Zero-mock rule**: crash comes from the real browser crash injection
(`Page.crash`), not from a JS exception thrown by the harness. Storage is
the real OPFS mount, read back through a real page reload.

**Falsifier (non-vacuity mutation)**: replace the crash injection with a
no-op (crash never fires) — the court must FAIL because the "mid-transaction"
precondition (uncommitted rows present) is not established; assert the
precondition with a pre-crash read showing uncommitted state is invisible to
a second connection. Second mutation: truncate the WAL by one block after
crash — the court must FAIL loudly on replay, proving the checksum asserts
can detect corruption at all.

## Court 2: bounded memory <= 256MB via CDP Performance metrics

**Procedure**: during peak workload (largest configured churn batch), poll
memory with `Performance.getMetrics` on a deadline loop: sample every
iteration of the burn-in loop, track `JSHeapUsedSize` and
`JSHeapTotalSize`; additionally read `Performance.getOverview` /
`processMemory` where available for the renderer process. The bound applies
to the observed peak across the whole run.

**Poll, never sleep**: the sampling loop is driven by workload progress
observables (rows committed per iteration, read from the worker via
`Runtime.evaluate` returning real counters) with a wall-clock deadline; a
run that makes no progress within the deadline fails as STALLED, not
retried on a timer.

**Assertion**: `peak(JSHeapUsedSize + wasmMemory) <= 256MB` over the entire
burn-in. Wasm linear memory is read from the instance itself
(`Memory.buffer.byteLength`) via `Runtime.evaluate` — this is a real
observable of the real instance, not a mock gauge.

**Falsifier**: inject a workload configuration that allocates > 256MB (e.g.
10x batch size). The court must FAIL with the offending sample attached.
A court that passes at 10x batch size is reading the wrong metric — that is
the vacuity check.

## Court 3: OPFS exclusive file-lock race under the real QuotaManager

**Procedure**: two workers (two `Worker` instances in the same origin,
sharing the real OPFS mount) attempt to open the same database file with an
exclusive lock simultaneously. Assertions:

- Exactly one open succeeds; the other receives the real typed lock
  contention error from the substrate (via the WASM layer surfacing the
  underlying OPFS/`duckdb` open failure), never a harness-synthesized error.
- The loser, on retry after the winner closes, opens successfully and reads
  all rows the winner wrote — proving the lock actually excluded and
  released.
- Concurrent write attempts from the loser while the winner holds the lock
  do not corrupt the winner's data (winner re-checksums its own rows after
  the loser's failed attempts).

**QuotaManager realism**: run under a constrained storage partition
(separate profile dir with a small disk budget) and force a near-quota
condition by pre-filling OPFS with real blobs until
`navigator.storage.estimate()` reports usage near quota; assert the open or
write path returns the real quota error rather than looping or silently
dropping writes.

**Zero-mock rule**: both lock contention and quota pressure come from the
browser's real storage machinery. No fake handle layer, no simulated
`FileSystemSyncAccessHandle`.

**Falsifier**: remove the exclusive-lock acquisition from the WASM open
path (open in read-write-shared mode). The court must FAIL because both
workers now open successfully — proving the lock assertion can detect the
absence of exclusivity. Second mutation: delete the quota pre-fill; the
quota branch becomes unexercised and its assertion must report SKIP-LOUD,
never PASS.

## Court 4: duckdb_library_version gate inside the worker

**Procedure**: after worker boot, call the version entry point exposed by
the WASM build inside the worker context (`Runtime.evaluate` against the
worker session) and assert it returns the exact crate-pinned DuckDB version
(2.0.0-dev, dev-suffix tolerant). The gate runs at the start of every burn-in iteration; a
mismatch aborts the iteration before any workload runs.

**Falsifier**: ship a mismatched `duckdb.wasm` artifact to the harness
(stage a wrong-version binary as a build input). The gate must refuse with
a typed version error. A gate that accepts any version is VACUOUS.

## CDP harness sketch

Status: UNVERIFIED — this is pseudo-shape for the implementer, not a
receipted artifact. It becomes verified only with a real run log attached
per the wrap rules.

```
# pseudo-Rust driving chrome-headless-shell over CDP (websocket)
struct CdpSession { ws: WebSocket, msg_id: AtomicU64 }

fn launch(profile: &Path) -> Browser {
  // spawn chrome-headless-shell --headless --remote-debugging-port=0
  // read "DevTools listening on ws://..." from stderr (bounded poll
  // on the real observable: stderr line, deadline 10s, else LAUNCH_FAILED)
}

async fn with_page(b: &Browser, f: impl Fn(Page)) {
  let target = b.call("Target.createTarget", url)?;
  let sess  = b.call("Target.attachToTarget", {targetId, flatten:true})?;
  f(Page { sess });
  b.call("Target.closeTarget", {targetId})?;
}

// Court 1 core (sketch):
with_page(&b, |p| {
  p.eval(worker_boot())?;                       // real OPFS mount
  p.eval(seed_and_begin_tx())?;                 // deterministic dataset
  assert!(!p.eval(visible_to_second_conn())?);  // precondition, real read
});
b.call("Page.crash", ())?;                        // real browser crash
let b2 = launch(&profile);                        // SAME profile => OPFS persists
with_page(&b2, |p| {
  p.eval(worker_boot())?;                        // WAL replay happens here
  assert_eq!(p.eval(replay_errors()), "[]");
  assert!(checksums_match_expected(p)?);         // pre OR post state
});

// Court 2 poll (no sleep):
loop {
  let progress = p.eval(rows_committed())?;      // real counter from worker
  if progress >= target { break; }
  assert!(start.elapsed() < DEADLINE, "STALLED");
  let m = p.sess.call("Performance.getMetrics", ())?;
  peak = max(peak, js_heap_used(&m) + wasm_memory_bytes(p)?);
}
assert!(peak <= MB256, "peak {peak} exceeds budget");

// Every poll reads a fresh observable; none waits a fixed interval.
```

## Falsifier table

| Court | Mutation proving non-vacuity | Expected court result |
|---|---|---|
| 1 WAL replay | Crash injection replaced by no-op | FAIL (precondition unmet) |
| 1 WAL replay | WAL truncated one block post-crash | FAIL (loud replay error) |
| 2 memory | Workload scaled 10x past 256MB | FAIL (offending sample) |
| 3 file lock | Exclusive lock removed from open path | FAIL (both opens succeed) |
| 3 quota | Quota pre-fill deleted | SKIP-LOUD, never PASS |
| 4 version gate | Mismatched duckdb.wasm staged | FAIL (typed version error) |

A burn-in wave is admitted only when every court passes against the real
substrate AND its named mutation, run against the same build, produces the
expected non-PASS result. Any mutation that leaves its court passing marks
the court VACUOUS and blocks the wave.

## See Also

- `~/.claude/rules/no-sleep-testing.md`
- `~/.claude/rules/testing-chicago-style.md`
- `docs/sjira/` active window (`_LANES.md`, work orders)
