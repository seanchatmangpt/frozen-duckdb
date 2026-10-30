# USER-FINDINGS-U10 — real two-process concurrency probe (lane U10)

- Subject: `/Users/sac/frozen-duckdb` @ master 0a205b7 (workspace `frozen-duckdb` 2.0.0, prebuilt dylib `~/.frozen-duckdb/cache/v2.0.0-arm64/libduckdb_arm64.dylib`)
- Probe: `/tmp/u10` (`u10probe`, cargo release build linking `crates/frozen-duckdb`); two real OS processes per experiment, one db FILE (`/tmp/u10/work/db.duckdb`), no sleeps-as-sync (file-signal handshake only).
- Date: 2026-10-19

## Observed behavior (real executions, exact errors)

### EXP1 — P1 write txn open, P2 attempts write
P2's `Connection::open` itself fails IMMEDIATELY (~4 ms, measured 4–5 ms over
repeats; no block, no retry, no timeout):

```
OPEN-ERR 4ms IO Error: Could not set lock on file "/private/tmp/u10/work/db.duckdb": Conflicting lock is held in /private/tmp/u10/target/release/u10probe (PID 52909) by user sac. See also https://duckdb.org/docs/current/connect/concurrency
```

The in-process MVCC write-write conflict (doctor TEST-023, tuple-overlap
`TransactionContext: Conflicting...`) is NEVER reached cross-process: the
file lock gates first. After P1 commits and exits, P2 write succeeds
(WRITE-OK 17ms).

### EXP2 — P2 read while P1 writes: NOT MVCC, a hard file lock
- Default open: same immediate `Could not set lock` IO Error (EXP2, EXP2b ×3).
- READ_ONLY config open (`Config::default().access_mode(READ_ONLY)`): SAME
  failure while P1 holds the file (EXP2c, 5ms, same message). A reader
  cannot attach to a file a writer holds.

```
RO-OPEN-ERR 5ms IO Error: Could not set lock on file "...": Conflicting lock is held in ... (PID 59964) ...
```

- Two READ_ONLY processes with NO writer: both succeed concurrently
  (RO-READ-OK 30ms / 27ms, count=1 both).

### EXP3 — P1 killed -9 mid-txn (X9 pattern)
`kill -9` (exit 137). Next open by P2: clean recovery — OS releases flock on
process death, WAL replay, no stale lock, no corruption:

```
READ-OK 18ms count=1   # P1's uncommitted update gone (correct)
WRITE-OK 17ms
```

### EXP4 — both processes reopen under churn (20 cycles each)
14/40 opens succeed, 26/40 fail with the immediate lock error. No hangs, no
timeouts, no corruption, no spurious post-lock failures; every winner's
write+commit lands. Losses are the single-writer file lock, deterministic.

## Doc comparison — divergences

1. **WORST — `docs/reference/r2d2-pooling.md:36-38`**: claims "If you need
   multi-process access, use ... `AccessMode.READ_ONLY` via
   `file_with_flags`". FALSIFIED by EXP2c: a READ_ONLY process CANNOT open
   the file while any writer holds it — that recipe only works for
   multiple-readers-no-writer (EXP2d). A user following it gets the exact
   `Could not set lock` error above at ~4ms. FIXED this session (doc edit,
   not projection: `docs/` is hand-authored).
2. `docs/explanation/architectural-moat.md:30` "single-writer engine" —
   CONSISTENT with observations.
3. README / how-to / QUICKSTART / TROUBLESHOOTING: silent on cross-process
   file locking; no claim contradicted by observations (TROUBLESHOOTING
   "flock" hits are the Flock LLM extension, unrelated).

## Fix applied (doc correction, matches reality)

`docs/reference/r2d2-pooling.md` "When not to pool" section rewritten:
single-writer-per-file is absolute (reader excluded too, even READ_ONLY);
multi-reader-no-writer is the only multi-process mode; retry-loop recipe for
writer handoff, citing the witnessed error string.

## Falsifier / replay

```bash
cd /tmp/u10 && cargo build --release
P=/tmp/u10/target/release/u10probe; DB=/tmp/u10/work/db.duckdb
$P seed $DB
rm -f work/ready work/go
$P hold-txn $DB work/ready work/go commit &   # P1
while [ ! -f work/ready ]; do sleep 0.05; done
$P try-write $DB          # -> OPEN-ERR 4ms IO Error: Could not set lock ...
$P try-read-readonly $DB  # -> RO-OPEN-ERR 5ms (same lock error)
touch work/go; wait       # P1 commits; P2 write then succeeds
```
