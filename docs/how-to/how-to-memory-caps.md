# How To: Cap Engine Memory Without Crashing

Problem-oriented guide: bound frozen-duckdb's memory with a typed OOM
contract instead of process kills, and avoid the spill fence trap.

Grounded in `crates/frozen-duckdb/tests/oom_graceful_abort.rs` (GATE-V-03
court; X9 receipt legs green this campaign —
`docs/sjira/v2.0.0/CAMPAIGN-RECEIPT.md` line 323) and
`crates/frozen-duckdb/src/duckdb/config.rs:129`.

See Also: `docs/design/gate-v03-memory-ceiling.md`,
`docs/design/memory-bounds.md`, `docs/how-to/how-to-run-chicago-doctor.md`

## The pattern: Config::with at open time

```rust
use frozen_duckdb::{Config, Connection};

let config = Config::default()
    .with("memory_limit", "128MB")   // or 256MB / 4GB
    .expect("config with memory_limit");

let conn = Connection::open_in_memory_with_flags(config)
    .expect("open in-memory db");
```

`Config::with(key, value) -> Result<Self>` is the single setting seam
(`src/duckdb/config.rs:129`); every DuckDB setting reachable via
`SET` is reachable here at construction, before any query runs.

## The typed OOM contract

Under a cap too small for the workload, the engine must return a typed
error — never panic, never abort, never SIGKILL the process:

```rust
let err = conn
    .execute_batch(PROBE_SQL)   // 2 GiB nominal CTAS under 256MB cap
    .expect_err("probe must OOM under cap");
assert!(err.to_string().contains("Out of Memory"));
```

The witness: the test process is still alive to make that assert —
reaching it at all is the no-crash proof. And the same connection keeps
working after the OOM:

```rust
let v: i32 = conn.query_row("SELECT 42", [], |r| r.get(0))
    .expect("post-OOM SELECT 42");
assert_eq!(v, 42);
// small CTAS + INSERT + SELECT round-trip also succeeds
```

Production pattern: size `memory_limit` to your container's budget,
treat `Out of Memory` as a catchable, retryable query failure, and keep
the connection — engine state survives.

## The spill-fence caveat

`memory_limit` is a MEMORY cap, not a workload cap. Without a fence the
engine spills to disk under pressure and never hits the cap (witnessed
in the design doc) — your query completes and the cap silently never
fired. Fence spill when you actually want to test the cap:

```rust
conn.execute_batch("SET temp_directory=''")
    .expect("spill fence");
```

Choose deliberately:

- **Testing the cap** (court, OOM contract): fence with
  `temp_directory=''` so the cap is genuinely reachable.
- **Production under tight memory**: leave spill enabled and budget
  disk for it; the fence makes the cap real but also makes big
  workloads fail that would otherwise succeed slowly.

## Two-sided non-vacuity

The cap-side test is only meaningful with a completion side: the SAME
probe at `memory_limit=4GB` must COMPLETE. Otherwise you are testing
the query, not the cap. That leg is `#[ignore]`d (needs ~2 GiB+ real
RSS); run it manually:

```bash
cargo test -p frozen-duckdb --test oom_graceful_abort -- --ignored --nocapture
```

## Run the doctor stage

```bash
just doctor              # stage 05_memory_guard runs this + leak audit
# or stage-only:
bash scripts/doctor/05_memory_guard.sh
```

Stage standing: PARTIAL_ALIVE (the leak-audit leg's allocator-stats C
API is absent on the pinned engine surface — KD-08 in
`KNOWN_DIVERGENCES.md`; the D6 RSS-deceleration proxy covers memory).

See Also: `KNOWN_DIVERGENCES.md` (KD-08),
`docs/how-to/how-to-ocel2-compression.md`
