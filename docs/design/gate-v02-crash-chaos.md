# GATE-V-02: crash-chaos court (continuous writer + ungraceful kill)

Standing: PROPOSED-RUNNABLE (native `kill -9` leg runs today).
Browser `Page.crash` via CDP leg is the ALIVE adjudicator per
`docs/design/chrome-burnin-court.md` Court 1; blocked on FR-02 until
`opfs_crash_court.mjs` exists.

Alias of record: AC-2 in `docs/design/acceptance-criteria-courts.md`;
falsifier table rows AC-2a/AC-2b in
`docs/design/vfs-journal-design.md` section 7. This doc is the GATE-V
alias: it binds those claims to concrete command contracts.

## Claim under test

A `kill -9` delivered DURING an open write transaction leaves the
database (1) mountable, (2) WAL-replayed to the last COMMITTED
transaction (no partial txn rows), (3) with zero page corruption —
checksums equal one of exactly two legal values (pre-kill or
post-commit), never a third.

## Legs

- **Native leg (PROPOSED-RUNNABLE now)**:
  `bash scripts/courts/crash_chaos_court.sh`
- **Browser leg (FR-02-blocked)**: `opfs_crash_court.mjs` driving
  CDP `Page.crash` mid-txn against real OPFS, per chrome-burnin
  Court 1. ALIVE requires its receipt; the native leg alone caps
  standing at PARTIAL_ALIVE.

## Native leg command contract

### Writer loop script

`scripts/courts/crash_chaos_writer.sh <db-path> <sentinel-file>`
( POSIX sh; runs as the victim process, in its own process group.

```
1. open (or init) <db-path>
2. CREATE TABLE IF NOT EXISTS events(
     pk BIGINT PRIMARY KEY, checksum_col BIGINT);
3. loop:
   a. BEGIN TRANSACTION
   b. INSERT batch of 1000 rows (seeded PRNG payload;
      checksum_col = f(pk) deterministic)
   c. UPDATE <sentinel-file> with committed row count (atomic
      write: tmp + rename)
   d. COMMIT
   e. record last committed count to <sentinel-file>.committed
      (tmp + rename) BEFORE next BEGIN
```

Law: `e` runs INSIDE the loop, after COMMIT, before the next
BEGIN — so the sentinel always names the last fully-committed
count. The gap between sentinel and next COMMIT is the exposure
window the court must catch.

### Kill timing: sentinel poll, never sleep

The driver polls a real observable with a deadline:

```
last=0
while [ "$count" -lt "$TARGET" ]; do
  count=$(cat sentinel.committed 2>/dev/null || echo 0)
  ...
done
```

The driver fires `kill -9` the moment the sentinel crosses a
PRE-CHOSEN TARGET value chosen so the kill lands between COMMIT
of txn N and COMMIT of txn N+1 (the exposure window). If the
poll exceeds its deadline (writer stalled), the court fails as
STALLED — a typed failure, not a timeout guess.

No-sleep doctrine: zero `sleep(N)` in the court. Every wait is a
bounded poll on the sentinel file with a wall-clock deadline.

Deliver `kill -9 -- -$PGID` (process-group kill) so no orphaned
writer child survives to complete the open txn after the court
proceeds.

### Reopen + assertion contract

Fresh process against the same file, no cleanup:

```
frozen-duckdb-cli <db-path> \
  "PRAGMA database_size;" \
  "PRAGMA storage_info;" \
  "SELECT count(*), sum(checksum_col) FROM events;" \
  "CHECKPOINT;"
```

Assertions (all must hold for PASS):

1. reopen exits 0 (mountable).
2. `PRAGMA database_size` and `PRAGMA storage_info` parse and
   exit 0 (page set is self-consistent; zero page corruption).
3. `count` and `sum(checksum_col)` equal exactly the values
   computed from the last committed sentinel — the count written
   to `<sentinel-file>.committed` before the kill:
   - `count == committed_count` (exact; WAL replayed the last
     committed txn, discarded nothing committed)
   - `checksum == f(committed_count)` (closed-form over the
     deterministic payload; a torn/partial txn produces a third
     value and FAILs)
4. `CHECKPOINT` exits 0 post-recovery.
5. Two legal post-states only: pre-txn (== committed sentinel)
   or... the committed sentinel IS the only legal value here —
   the alternative legal value (one txn ahead) is covered by the
   browser leg's two-injection-point design (chrome-burnin Court
   1) and by running this court at TARGET and TARGET+batch.
6. Run the court twice: TARGET and TARGET + 500 rows, so both
   kill points (mid-txn and just-post-commit) are exercised.

### Artifact

`target/court/ac2-receipt.json`:
`{kill_pid, signal, pgid, pre_kill_sentinel_count,
post_recovery_count, post_recovery_checksum, expected_checksum,
reopen_exit, pragmas_ok, verdict}`. Receipt embeds real command
output (raw counts, exit codes), never summaries.

## Non-vacuity (shim court, fail-closed)

1. **No-op kill shim**: replace `kill -9` with `kill -0` (signal
   0 probes, kills nothing; edit marked `SHIM-COURT`). The court
   MUST DETECT that no real crash fired — the shimmed run's
   writer process is still alive at the reopen step, and the
   post-run count exceeds the committed sentinel (the exposure
   window filled in) — and FAIL loudly. Detection rule: after
   the court's kill step, `kill -0 $PID` must FAIL (process gone)
   within the bounded poll deadline; if the victim survives, the
   court exits 1 as `VACUOUS GATE: no crash fired`.
2. **WAL-rot mutation** (AC-2b alias): after a real kill, truncate
   the WAL by one block (marked `SHIM-COURT`), then reopen — the
   court must FAIL on assertion 1 or 2, proving the checksum and
   pragma asserts can detect corruption at all. If the truncated
   run still passes, the assertions are vacuous and the court
   FAILs.
3. **Baseline/restore**: sha256 the writer script + court script
   before; every shim carries the literal `SHIM-COURT` marker;
   restore and require byte-equal sha256 or report the subject
   CORRUPTED and exit nonzero.
4. A court leg that cannot flip under mutation 1 or 2 is FAIL,
   not PASS.

## No-sleep compliance

Zero `Process.sleep`/`sleep N`. Sentinel polls are bounded
deadline loops; a stalled writer is STALLED, a survivor after
`kill -9` is VACUOUS. Both are typed exits, never retries.

## Standing rules

- PROPOSED now (this spec, no run receipt).
- PARTIAL_ALIVE: first green native receipt at both kill points
  (TARGET, TARGET+500) with its no-op-shim and WAL-rot flips
  witnessed on the same subject.
- ALIVE: browser-leg receipt (CDP `Page.crash`, real OPFS,
  two-injection-point checksum adjudication) per chrome-burnin
  Court 1. Native alone never reaches ALIVE.
- Regresses to BLOCKED if any replay diverges from the closed-form
  checksum or any reopen exits nonzero on an unshimmed subject.

## Loud skips

Missing toolchain, missing CLI/test binary, or unwritable
`target/court/` => `[SKIP] <reason>`, never a faked green cell.

## Falsifier table (GATE-V alias)

| ID | Falsifier | Expected on mutation |
|----|-----------|----------------------|
| V02-a | `kill -9` -> `kill -0` shim | FAIL: victim survives, VACUOUS GATE |
| V02-b | WAL truncated one block post-crash | FAIL on reopen/pragma assert |
| V02-c | writer loop reduced to single txn, no loop | court detects no exposure window, FAIL |
