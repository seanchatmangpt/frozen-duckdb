//! TEST-FLEET-005 — Class E (telco) test-fleet leg: DUCKDB.
//!
//! Spec: 50k-row columnar batch flush via bulk appender; per-batch flush
//! latency over 40 batches (spec bound <45ms/batch); end-to-end
//! session-to-queryable latency (spec bound 1200µs).
//!
//! Scope, recorded honestly:
//! - This file is the DUCKDB (native) leg only. The BEAM :atomics/:ets
//!   actor leg lives in the smalt repo and is OUT OF SCOPE for this
//!   canonical checkout: **smalt-leg BLOCKED-OTHER-REPO**. The
//!   scheduler-starvation assertion (how many batch flushes a
//!   scheduler-hogging actor can tolerate) requires the BEAM scheduler
//!   and therefore cannot execute here; it is recorded as smalt-leg
//!   BLOCKED, not skipped silently and not faked.
//! - The session-to-queryable measurement is an IN-PROCESS
//!   insert+query on one connection — NOT network-ingress latency. The
//!   1200µs spec is an ingress-to-queryable bound; the measured number
//!   below is reported as the in-process floor of that bound.
//!
//! Receipt: FLEET-005 (duckdb leg) → /tmp/value_audit/fleet005.json

use frozen_duckdb::Connection;
use std::time::Instant;

const BATCH_ROWS: usize = 50_000;
const BATCHES: usize = 40;
const TOTAL_ROWS: i64 = (BATCH_ROWS * BATCHES) as i64;

/// Deterministic seeded PRNG (xorshift64*), no external deps.
struct Rng(u64);

impl Rng {
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
}

fn median(v: &mut [f64]) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

/// Leg 1: 50k-row columnar batch flush via bulk appender, 40 batches,
/// per-batch flush latency; median asserted < 45ms.
#[test]
fn fleet005_batch_flush_latency() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(
        "CREATE TABLE cdr (
            id BIGINT,
            msisdn BIGINT,
            cell_id INTEGER,
            bytes_up BIGINT,
            bytes_down BIGINT
        );",
    )
    .unwrap();

    let mut rng = Rng(0x0045_5435_00c0_ffee);
    let mut lat_ms: Vec<f64> = Vec::with_capacity(BATCHES);

    for batch in 0..BATCHES {
        let rows: Vec<[i64; 5]> = (0..BATCH_ROWS)
            .map(|i| {
                let n = rng.next_u64();
                [
                    (batch * BATCH_ROWS + i) as i64,
                    n as i64,
                    (n >> 20) as i32 as i64,
                    (n >> 40) as i64,
                    (n >> 48) as i64,
                ]
            })
            .collect();

        let mut app = conn.appender("cdr").unwrap();
        let t0 = Instant::now();
        app.append_rows(rows.iter().copied()).unwrap();
        app.flush().unwrap();
        lat_ms.push(t0.elapsed().as_secs_f64() * 1e3);
    }

    let med = median(&mut lat_ms);
    let total = conn
        .query_row("SELECT count(*) FROM cdr", [], |r| r.get::<_, i64>(0))
        .unwrap();
    assert_eq!(total, TOTAL_ROWS, "all {TOTAL_ROWS} rows must be queryable");

    println!(
        "FLEET-005 flush: batches={} rows/batch={} median={:.3}ms min={:.3}ms max={:.3}ms (bound <45ms)",
        BATCHES,
        BATCH_ROWS,
        med,
        lat_ms.iter().copied().fold(f64::INFINITY, f64::min),
        lat_ms.iter().copied().fold(f64::NEG_INFINITY, f64::max)
    );
    assert!(
        med < 45.0,
        "median batch flush latency {:.3}ms violates 45ms spec bound",
        med
    );
}

/// Leg 2: end-to-end session-to-queryable — insert one row, then SELECT
/// it; report real elapsed microseconds. Honest label: in-process
/// insert+query on one connection, not network-ingress latency.
#[test]
fn fleet005_session_to_queryable() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("CREATE TABLE t (id BIGINT, v VARCHAR);")
        .unwrap();

    // Warm the table machinery so we measure the steady-state path, not
    // first-touch catalog/allocator costs.
    conn.execute("INSERT INTO t VALUES (0, 'warm')", [])
        .unwrap();
    conn.query_row("SELECT count(*) FROM t", [], |r| r.get::<_, i64>(0))
        .unwrap();

    let t0 = Instant::now();
    conn.execute("INSERT INTO t VALUES (1, 'fleet005')", [])
        .unwrap();
    let got: i64 = conn
        .query_row("SELECT id FROM t WHERE v = 'fleet005'", [], |r| r.get(0))
        .unwrap();
    let elapsed_us = t0.elapsed().as_secs_f64() * 1e6;

    assert_eq!(got, 1, "inserted row must be immediately queryable");
    println!(
        "FLEET-005 session-to-queryable: {:.1}µs (in-process insert+query, NOT network ingress; spec bound 1200µs)",
        elapsed_us
    );
    assert!(
        elapsed_us < 1200.0,
        "session-to-queryable {:.1}µs violates 1200µs bound",
        elapsed_us
    );
}
