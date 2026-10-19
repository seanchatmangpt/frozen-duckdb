# VFS + Journal Design (OPFS Storage Adapter, ARD §3.2.B)

Status: DESIGN — nothing in this document has been executed. Every runtime
claim is UNVERIFIED until a real BP2 court run proves it on an exact
subject. Companion docs: `docs/design/storage-format-v2.md` (storage-version
gating, EXPORT/IMPORT migration) and `docs/design/chrome-burnin-court.md`
(courts 1-4, falsifier table).

## 0. Ownership boundary (read this first)

DuckDB's storage engine already implements the hard parts. This wave builds
the thin adapter beneath it, not a second storage engine.

| Property (ARD §3.2.B) | Owner | Mechanism |
|---|---|---|
| Columnar page format, 4KB-aligned | DuckDB-native | Its storage manager; we pin the grain (§2) |
| Write-ahead journal append | DuckDB-native | Engine `.wal`; we provide byte transport (§3) |
| Recovery-on-init at unclean marker | Replay=engine; trigger=ours | Engine replays; we mark (§4) |
| Exclusive single-writer locks, structured errors | OURS | Handle mode + typed error codes (§5) |
| Concurrent reads, independent memory contexts | OURS | Read-only snapshot mounts + 1 writer (§6) |

Law: if a property fails, we fix the adapter or file it upstream — we never
re-implement replay, checksums, or page eviction below the engine.

## 1. Architecture

DuckDB-WASM in a dedicated Web Worker. The VFS lives entirely inside the
worker (`FileSystemSyncAccessHandle` is Worker-only, secure-context-only).
Main thread sees a message-port RPC surface only
(`docs/design/wasm-opfs-architecture.md`).

Layer stack, top to bottom:

1. Engine storage manager (DuckDB): columnar pages, buffer manager,
   checkpoint, WAL write/trim/replay. Not touched by this wave.
2. `FileSystem` interface impl (OURS): `OpfsSyncVFS`, a
   pread/pwrite-shaped mapping of DuckDB's `FileSystem::Read/Write/
   FileExists/Remove/...` onto `handle.read(buf, {at}) /
   write(buf, {at}) / truncate / getSize / flush / close`.
3. Lock + error-map layer (OURS): open-mode computation, handle-mode
   negotiation, error translation to structured codes (§5).
4. OPFS substrate (browser): real quota, real exclusivity, real
   persistence.

## 2. Page layout and 4KB alignment

- DuckDB's on-disk format is columnar and block-structured natively; the
  block size is engine-configurable. ARD requires a 4KB-aligned grain at
  the VFS boundary.
- Design: pin the engine block size to 4096 via the config passthrough
  (`crates/frozen-duckdb/src/duckdb/config.rs`, `Config::with`,
  equivalent of `SET block_allocation_size`/`SET storage_block_size`).
  UNVERIFIED: exact key spelling and whether DuckDB 2.0.0-dev accepts
  4096 (engine minimum supported block size is UNVERIFIED; upstream
  default is larger). If 4096 is refused, fall back: keep the engine's
  default block size and make the VFS issue 4KB-aligned sub-block I/O
  (every `write(buf, {at})` has `at % 4096 == 0` and lengths padded to
  the 4KB grain; file sizes are multiples of 4096 via `truncate`).
  Alignment invariant is assertable in the VFS itself regardless of which
  fallback wins.
- Why alignment matters here: OPFS `write({at})` with arbitrary offsets
  is the known throughput cliff (wasm-opfs-architecture, "per-call `at`
  offsets ... known throughput limits"); aligned fixed-grain I/O keeps
  the surface simple to reason about for quota and crash-point analysis.
- Checksums: DuckDB-native (per-block checksums in its storage format).
  The VFS adds none, removes none. Checkpoint integrity is asserted by
  the BP2 court (Court 1 asserts zero corruption after replay).
- Storage version: OPFS-attached databases pass
  `STORAGE_VERSION 'v2.0.0'` per `storage-format-v2.md`; never combined
  with an explicit `ENCRYPTION_KEY` (dylib strings declare them
  incompatible).

## 3. Write-ahead journal (append path)

- The journal IS DuckDB's `.wal`. Engine appends WAL records at commit;
  `CHECKPOINT` folds the WAL into the main `.db` file and truncates the
  WAL. Our layer: byte transport + flush discipline.
- Flush discipline (ours, and the durability contract): `flush()` is
  called on the WAL handle at every WAL commit boundary. Relying on
  implicit OPFS flush is unspecified behavior per spec, so the VFS flush
  is explicit and synchronous. `handle.write()` on a sync access handle
  is already synchronous from the worker's perspective; `flush()` is
  what bounds the durability point.
- Durable point semantics (contract C1, storage-format-v2): a committed
  transaction survives `Page.crash` iff its WAL flush completed. The VFS
  exposes a counter of completed WAL flushes; the BP2 court reads it via
  `Runtime.evaluate` (real observable, no-sleep polling).
- Checkpoint policy (ours): trigger `CHECKPOINT` after N committed
  transactions or M WAL bytes (initial: N=1000, M=64MB; both are
  configuration, not assertions). UNVERIFIED: whether explicit
  `CHECKPOINT` over the OPFS VFS completes without wedging mid-size WALs
  — first real run decides.
- Main `.db` writes happen only inside `CHECKPOINT` (engine-driven);
  the VFS flushes the `.db` handle at checkpoint completion, before the
  engine truncates the WAL.

## 4. Recovery-on-init (unclean-shutdown marker)

- The unclean-shutdown marker is physical and ours to read at mount:
  a clean shutdown leaves (a) a zero-length-or-absent WAL (post-
  checkpoint) OR (b) a WAL whose every record is followed by the
  engine's commit/flush boundary. We do NOT parse WAL records (that is
  engine-native). Our mount-time marker is:
  - `wal getSize() > 0` at open => unclean (last checkpoint did not
    complete after final commit) => run engine recovery (normal open;
    engine replays the WAL).
  - Plus a sidecar sentinel `mount.state`: written `<boot-id>` at
    worker boot, deleted on clean `close()`. `mount.state` present at
    the next boot => previous session died uncleanly => log the
    recovery event, set `recovered_from_unclean=true` on the RPC open
    result, and proceed with engine replay. UNVERIFIED: worker kill
    semantics on `Page.crash` (whether the worker gets any cleanup
    chance — the design assumes none, the sentinel makes it observable
    rather than guessed).
- Recovery itself is DuckDB-native replay through the same OPFS VFS —
  no browser-specific recovery code (wasm-opfs-architecture). If replay
  errors, the VFS surfaces the engine error mapped to
  `RECOVERY_FAILED` (§5) and refuses to hand the DB to the RPC layer
  in a half-open state (fail-closed: no read-only degraded mount after
  a failed replay).
- AC-2 falsifier (Court 1): `Page.crash` mid-txn, relaunch on the same
  profile, mount must succeed and post-state checksums equal pre-txn OR
  post-commit — never a third value; checkpoint/full-scan integrity
  clean. A mount that fails or a third checksum value is FAIL.

## 5. Exclusive single-writer locks -> structured error codes

- OPFS gives no advisory byte-range locks. Exclusivity is carried by the
  sync access handle mode itself: a handle opened with
  `createSyncAccessHandle({mode: 'in-place'})` (read-write) is exclusive
  among handles on the same file within the origin for the handle's
  lifetime. UNVERIFIED: cross-worker exclusivity strength per current
  Chrome (the browser may permit a second non-exclusive handle); the
  design therefore does NOT rely on the substrate alone.
- Belt-and-suspenders: a lockfile `db.db.lock` carrying
  `{bootId, workerId, heldSince}` is created (exclusive create via
  `create({exclusive: true})` on a fresh file) before the writer mount,
  and deleted at clean close. Present lockfile with a live holder =>
  reject; present lockfile with a dead bootId (no heartbeat refresh
  within TTL, read via a separate non-exclusive read handle) => stale,
  take over with a logged `LOCK_TAKEOVER` event. UNVERIFIED: need for a
  heartbeat at all until Court 3 exercises two-worker contention.
- Open-mode mapping (OURS):

| Intended access | OPFS handle mode | On contention |
|---|---|---|
| Writer (read-write, exclusive) | `in-place`, lockfile first | Reject, structured code |
| Reader (read-only) | non-exclusive read handle | Succeeds alongside writer |

- Structured error codes (typed on the RPC surface, never a hang, never
  a generic IO error):

| Code | Trigger | Maps from |
|---|---|---|
| `DB_LOCKED` | lockfile held by a live session | `NoModificationAllowedError` / create fail |
| `DB_LOCKED_STALE` | dead-holder takeover taken | (informational, not fatal) |
| `QUOTA_EXCEEDED` | origin quota hit on write | `QuotaExceededError` |
| `RECOVERY_FAILED` | engine WAL replay errored | engine exception through VFS |
| `VERSION_UNSUPPORTED` | storage-version open refusal | engine open error (v2 fallback path) |
| `NOT_FOUND` | missing DB file on reader open | OPFS `NotFoundError` |
| `WORKER_DEAD` | reject on worker `error`/`messageerror` | worker teardown (rpc layer) |

- Rejection semantics: every lock conflict surfaces as a rejected
  promise carrying the code within one event-loop turn of the failed
  substrate call. No retry loops inside the VFS; retry policy belongs to
  the caller. A caller that waits on a contended lock is a bug — the
  correct shape is reject-fast.

## 6. Concurrent read sessions

- One writer worker holds the exclusive handle + lockfile. Reader
  sessions are separate mounts (separate workers or the main thread with
  a non-exclusive read-only handle) over a snapshot of the file taken at
  reader-open: readers copy `getSize()` bytes of `.db` into an in-memory
  context (`copy` via `read()` into transferable `ArrayBuffer`s) so a
  concurrent checkpoint cannot shift bytes under a reader.
- Snapshot cost is bounded by the 256MB wasm budget
  (wasm-opfs-architecture): DB files beyond the reader copy budget are
  rejected with `QUOTA_EXCEEDED` on the reader mount, loud not quiet.
  UNVERIFIED: whether a read-only handle over a live writer's file is
  safe for direct mmap-less reads mid-checkpoint — the snapshot rule is
  the conservative default until proven otherwise.
- Each read session runs in its own DuckDB in-memory instance
  (independent memory context by construction). Readers never write.
  Read-your-writes across sessions is re-mount, not live tailing —
  explicit non-goal for this wave.

## 7. Falsifiers

| ID | Falsifier | Expected on mutation |
|---|---|---|
| AC-2a | `Page.crash` mid-txn, remount: mountable, checksums pre-or-post | FAIL on 3rd value |
| AC-2b | WAL truncated one block post-crash | FAIL loud; proves checksums detect rot |
| F-lock | Lock step removed from open path (Court 3) | both opens succeed => FAIL |
| F-quota | Near-quota write through RPC | `QUOTA_EXCEEDED`; never hang/silent write |
| F-quota | Near-quota write through RPC | `QUOTA_EXCEEDED`; never hang or silent write |
| F-stale | Writer killed unclean, lockfile left | takeover, `LOCK_TAKEOVER`, recovery runs |
| F-vacuous | Any court passing under its mutation | VACUOUS, wave blocked (burnin-court law) |

AC-2 as stated in the work order (`Page.crash mid-txn -> mountable +
checkpoint checksums intact`) is AC-2a. Lock-conflict input must yield
a typed error, never a hang — that is F-lock2's reject-fast assertion.

## 8. UNVERIFIED register (consolidated)

1. Engine acceptance of a 4096 block size on 2.0.0-dev; exact config key.
2. Cross-worker strength of OPFS `in-place` handle exclusivity.
3. `Page.crash` worker cleanup semantics (sentinel design assumes none).
4. Explicit `CHECKPOINT` over OPFS at mid-size WALs.
5. Direct read-only reads mid-checkpoint (why readers snapshot instead).
6. Storage-version key/value spelling at open (inherited from
   storage-format-v2.md).

Each becomes verified only by a real BP2 court receipt on an exact
build. Nothing in §8 is admitted by this document.

## See Also

- `docs/design/storage-format-v2.md`
- `docs/design/chrome-burnin-court.md`
- `docs/design/wasm-opfs-architecture.md`
- `docs/design/verification-plan.md` (V-W2)
- `crates/frozen-duckdb/src/duckdb/config.rs` (`Config::with`)
