# gate-v03-memory-ceiling.md

GATE-V-03 court spec (Revision-2 numbers): adversarial memory
probe targeting ~2GB nominal under a 256MB configured ceiling
must return a graceful, typed error; host process and engine
state survive; subsequent queries succeed. Design doc only —
this lane edits no ggen projection (`scripts/verify-gates.sh`,
`Makefile`, `schema/verify.ttl` are sync outputs).

## Precedence and grounding

- Grounded in `docs/design/acceptance-criteria-courts.md` AC-3
  (OOM graceful-abort court) — same claim shape: typed error,
  never a process kill, state preserved for later use.
- **Revision-2 supersedes the AC-3 example numbers**: cap is
  `memory_limit="256MB"` (PRD-FR-05 says <=256MB; this spec uses
  the literal 256MB), probe scaled toward ~2GB nominal.
- PRD mapping: PRD-FR-05 / SLA "cap breach -> OutOfMemory
  status, never a crash" (`prd-ard-v2.md` GATE-V-03 row and
  GATE-V-05 row share this falsifier family; NFR-04 zero-panic
  boundary rides it — `requirements-traceability.md:92`).
- wasm variant (BP2 Court 2, 256MB linear memory) remains a
  separate future leg; this doc witnesses the native leg only.

## Probe definition (rev2 scale)

Config call (native, verified API — note the crate exposes
`open_in_memory_with_flags`, not `_with_config`):

```rust
let cfg = Config::default().with("memory_limit", "256MB")?;
let conn = Connection::open_in_memory_with_flags(cfg)?;
```

Probe SQL (exact constant; sha256 of the SQL string:
`32bd9787b41c44e1b6189972fd6153e658b53db595d2609718513abd85c09c73`):

```sql
CREATE TABLE probe_t AS SELECT a.range AS x, b.range AS y
FROM range(131072) a, range(1024) b;
```

Sizing math: 2^17 x 2^10 = 2^27 = 134,217,728 rows x 16 bytes
per side = 2,147,483,648 bytes = 2GiB nominal intermediate.
Materialized (CTAS) so the buffer manager must actually pin it.

Spill fence: `SET temp_directory=''` before the probe. Without
it the engine spills to disk and the cap is never hit (witnessed
below — the cap is only a *memory* cap, not a workload cap).

### Why count(*) is NOT an acceptable probe (witnessed)

The AC-3 example probe `SELECT count(*) FROM range(100000) a,
range(100000) b;` does **not** trip the cap: count(*) streams
the Cartesian without materializing. Real witness (same rig):

```
PROBE-COMPLETED count=134217728   (at 256MB cap, spill on)
```

Any court built on the streaming form is vacuous by
construction. The probe must be materializing (CTAS / ORDER BY
/ list aggregate).

## Witness receipt (native, first run — REAL)

- Date: 2026-10-19T20:12:46Z; host: Apple M3 Max, macOS 26.2,
  arm64; subject: `frozen-duckdb` HEAD
  `e4cfbd38f9efe55dde549c67d558da9769584fa0`, tree
  `8c6d4aff4d4ef1257186542581750f0b80e6bb6c`; engine: prebuilt
  `v2.0.0-arm64` dylib
  (`~/.frozen-duckdb/cache/v2.0.0-arm64/libduckdb_arm64.dylib`).
- Rig: `/tmp/r7gate` (scratch cargo bin, path-dep on
  `crates/frozen-duckdb`; probe source sha256
  `b714e729ba8be48977bb7890ff3c0610319cd101939b5c6694d5c2b009f0eac7`).
- Output (grep of `cargo run` under `CARGO_TARGET_DIR` scratch):

```
temp_directory disabled (no spill)
memory_limit=244.1 MiB
PROBE-ERROR typed err=Out of Memory Error: could not allocate
  block of size 256.0 KiB (243.9 MiB/244.1 MiB used)
POST-OOM SELECT 42 = 42
POST-OOM INSERT/SELECT ok
```

- Post-probe legs (same connection, after the OOM):
  `SELECT 42` -> 42; `CREATE TABLE t7 AS SELECT 1 AS x; INSERT
  INTO t7 VALUES (2); SELECT * FROM t7;` -> ok. Host process
  exit 0. No panic, no abort, no trap escape.

## Non-vacuity (two-sided, witnessed)

1. **Cap side**: at `memory_limit="2GB"` the same probe still
   OOMs (`1.8 GiB/1.8 GiB used`) — a cap merely above nominal
   with spill disabled still trips; the probe is genuinely
   memory-bound, not workload-bound.
2. **Completion side**: at a ceiling comfortably above nominal
   (`4GB`), the same probe COMPLETES (`PROBE-COMPLETED (cap not
   hit)`). If the probe OOMed at every cap the court would be
   testing the query, not the cap.
3. Standing-flip rule (from AC-3, adjusted): PROPOSED until
   first native run — **consumed**, the run above is it;
   PARTIAL_ALIVE on this native green receipt; ALIVE requires
   the wasm variant (BP2 Court 2). A court that cannot flip
   (probe OOMs at 4GB, or completes at 256MB) is FAIL.

## Non-vacuity shim-court (unrun legs, named honestly)

Run and witnessed above: cap-sweep flip (2GB/4GB legs).
Not yet run as a scripted loop with byte-exact restore:

- `[SKIP] scripted shim loop`: rig lives in /tmp scratch, not
  a repo court script; the cap-sweep legs above are real
  executions, but no `scripts/courts/` automation with
  sha256 BASELINE/RESTORE exists yet. A future court script
  must mutate the probe constant (marked `SHIM-COURT`) back to
  the streaming `count(*)` form and require the gate to REFUSE
  (detect the vacuous probe); a gate that accepts it exits 1
  as VACUOUS GATE.

## Planned scripted witness command (replay path)

For the durable court, port the rig into
`crates/frozen-duckdb/tests/oom_graceful_abort.rs` (AC-3's
named target), which `cargo test -p frozen-duckdb` picks up
natively — no dyld staging needed (cargo test links the dylib
via the sys build script's out-dir; the /tmp scratch bin
needed a manual `cp` of the prebuilt dylib, see below).

Exact replay (scratch form, verified working this session):

```
cp ~/.frozen-duckdb/cache/v2.0.0-arm64/libduckdb_arm64.dylib \
   <target_dir>/libduckdb.dylib
cargo run  # rig: /tmp/r7gate, sha256 b714e729...9f0eac7
```

Note for whoever lands the tests/ port: the scratch binary
needed `DYLD` staging (`cp libduckdb_arm64.dylib
<target>/debug/libduckdb.dylib`) because the loader wants the
name `libduckdb.dylib`; `cargo test` inside the workspace does
this via the sys build script (workspace test targets already
link correctly — `target/debug/*.rlib` exist from prior runs).

## Pass rule

PASS iff all hold in one run: (1) probe returns a typed
`Out of Memory Error` string (no panic/abort/host kill);
(2) `SELECT 42` on the same connection returns 42 afterwards;
(3) INSERT+SELECT round-trip on the same connection succeeds;
(4) completion-side counter-leg at 4GB completes. Else FAIL;
inability to run is `[SKIP] <reason>` + exit 1 — never a
faked pass.

## Standing

**PARTIAL_ALIVE (native leg witnessed 2026-10-19, receipt
above; ALIVE blocked on wasm BP2 Court 2 variant).**
