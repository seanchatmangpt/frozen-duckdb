# gate-v04-opfs-concurrency.md

GATE-V-04: concurrent read+write inside `chrome-headless-shell` against real
`FileSystemSyncAccessHandle` instances. Extends BP2 court 3
(`chrome-burnin-court.md`) with a Web Locks coordination layer: the browser
owns the mutex, the engine never blocks, and every conflict is a structured
backoff decided at the HOST level. All content UNVERIFIED until the
wasm+OPFS substrate exists; a receipt without a real burn-in log is SUSPECT.

## 1. Why Web Locks, given OPFS exclusivity

OPFS `createSyncAccessHandle({mode: 'in-place'})` is exclusive among handles
on the same file within the origin, but the strength of that exclusivity
across workers is UNVERIFIED (vfs-journal-design.md, UNVERIFIED register
item 2). GATE-V-04 therefore treats the substrate lock as belt, not as the
seam:

- `navigator.locks` is the browser-side mutex over the OPFS handle. The
  writer mount runs entirely inside
  `navigator.locks.request('fzd-<db-name>', ...)`.
- The lockfile (`db.db.lock` per vfs-journal-design.md) stays as the
  crash-recovery seam: Web Locks are released by the browser on worker
  death, so the lockfile carries the dead-holder evidence
  (`bootId`, `heldSince`, heartbeat) that the takeover path needs.
- Web Locks are per-origin and per-profile; two `chrome-headless-shell`
  instances with different profile dirs do NOT contend. Dual-tab
  concurrency (section 4) uses one profile, two workers.

## 2. The structured-backoff contract

The engine (WASM) never waits on a contended lock. Retry policy lives at
the HOST boundary (the JS/Rust driver around the worker RPC), matching
vfs-journal-design.md's reject-fast rule: no retry loops inside the VFS.

Error surface (extends the vfs-journal error table):

| Code | Trigger | Carrier |
|---|---|---|
| `DB_LOCKED` | `navigator.locks.request` queued behind a live holder | rejected RPC promise |
| `DB_LOCKED_STEALABLE` | lockfile present, heartbeat stale | rejected RPC + takeover offer |
| `DB_LOCKED_STALE` | dead-holder takeover already taken | informational on the winning mount |

Host backoff shape (typed retry-with-jitter, bounded):

```
# host-level backoff (pseudo, harness side)
fn backoff(attempt: u32) -> Duration {
  base = 25ms * 2^min(attempt, 5)  # cap ~800ms
  jitter = rand(0 ..= base / 2)
  base / 2 + jitter                # bounded
}
# attempts <= MAX, then DB_LOCKED surfaced to the caller verbatim.
# Every wait is a real lock-state observable (locks.query()), never sleep.
```

Laws:

- No fixed `sleep(N)` anywhere in the backoff path (no-sleep doctrine).
  Backoff is a retry *schedule*, entered only after a real `DB_LOCKED`
  rejection; each retry re-observes lock state via `navigator.locks.query()`
  before re-requesting.
- The rejection reaches the caller within one event-loop turn of the
  failed substrate call — never a hang, never a renderer crash.
- Retry budget exhaustion surfaces `DB_LOCKED` with the final
  `locks.query()` snapshot attached, so the caller can distinguish
  contended from wedged.

## 3. Court 3 extension: Web Locks coordination

Extends `chrome-burnin-court.md` court 3. The original court's assertions
(exactly one winner, loser reads winner's rows after release, winner data
uncorrupted) all remain in force. Added procedure:

1. Both workers request `navigator.locks.request('fzd-<db>', ...)`
   before touching the OPFS handle.
2. Winner proceeds to mount + write; loser receives `DB_LOCKED` on its
   RPC promise and runs the host backoff.
3. Loser's retry succeeds after winner's clean close; loser re-checksums
   all rows the winner wrote.
4. The loser's conflicting writes while locked are asserted to have
   never reached the OPFS handle (winner's post-run checksum unchanged
   through the contention window).

Added assertion: `navigator.locks.query()` observed from a third
(non-holding) worker shows exactly one exclusive holder for the db name
for the whole contention window.

Falsifiers (non-vacuity, additive to court 3's):

| Mutation | Expected court result |
|---|---|
| Web Locks request removed from open path | real OPFS lock error surfaces; still-PASS is VACUOUS |
| Holder ignores `locks.query()` in backoff | retry loop reported SKIP-LOUD, never PASS |

The first mutation is the load-bearing one: without Web Locks the raw
substrate error must be visible, which proves GATE-V-04 exercises the real
OPFS lock, not a harness fiction.

## 4. Dual-tab concurrency: two workers, one writer

Two workers in one origin, one profile dir:

- Worker W: writer mount, holds the Web Lock + OPFS handle + lockfile.
- Worker R: reader mount, non-exclusive read handle, snapshot read per
  vfs-journal-design.md section 6 (readers copy bytes, never share the
  live handle).

Procedure:

1. W opens, seeds a deterministic dataset, enters churn (checkpointing
   periodically). R opens concurrently and must succeed immediately —
   reader-open is never blocked by the writer's lock.
2. R snapshots and checksums while W writes; assert every R checksum
   equals a valid W checkpoint state (pre-or-post, never a third value —
   the same disjunctive law as court 1's WAL-replay checksums).
3. R attempts a write; it must receive `DB_LOCKED` (reader mounts are
   read-only by construction, so this asserts the mount rejects writes
   with the typed code, not that a crash is avoided).
4. W is killed via CDP (`Page.crash`) while holding the lock. R's next
   read must still succeed (snapshot reads survive writer death), and a
   fresh worker W2 must complete the takeover: stale-lockfile detection,
   `LOCK_TAKEOVER`, WAL replay, checksums pre-or-post.

## 5. Holder-death takeover path (vfs-journal F-stale)

The simulated holder-death in step 4 is the GATE-V-04 witness for
vfs-journal-design.md's F-stale falsifier in the browser substrate:

- Web Locks release on `Page.crash` (browser-owned, not application
  state), so the crash produces exactly the stale condition the takeover
  exists for: lock gone, lockfile present, heartbeat stale.
- W2's open must observe `DB_LOCKED_STEALABLE` -> take over with a
  logged `LOCK_TAKEOVER` event -> run recovery -> report
  `DB_LOCKED_STALE` informationally.
- Falsifier: if W2 opens without emitting `LOCK_TAKEOVER`, the takeover
  path is unwired — court FAILs even if the data reads back fine, because
  the assertion is on the takeover path, not only the bytes.

## 6. UNVERIFIED register

1. `Page.crash` releases `navigator.locks` promptly (assumed per spec:
   locks are browser-owned; the court asserts it, not assumes it).
2. Cross-worker exclusivity strength of `in-place` handles (inherited
   UNVERIFIED from vfs-journal-design.md).
3. `navigator.locks.query()` visibility from a third worker in
   chrome-headless-shell.
4. Whether reader snapshot-open under writer churn stays under the 256MB
   wasm budget at the configured DB size.
5. Heartbeat TTL value; until measured, the takeover threshold is a
   config surface, not a constant.

## 7. Standing

UNVERIFIED. This gate becomes PARTIAL_ALIVE on the first real
chrome-headless-shell run with the Web Locks layer attached (log receipt
per chrome-burnin-court.md wrap rules), and ALIVE only when all named
mutations produce their expected non-PASS results on the same build.

## See Also

- `docs/design/chrome-burnin-court.md` (BP2 courts, falsifier table)
- `docs/design/vfs-journal-design.md` (lockfile, error codes, F-stale)
- `docs/design/wasm-opfs-architecture.md` (256MB budget)
- `~/.claude/rules/no-sleep-testing.md`
