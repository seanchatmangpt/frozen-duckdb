//! Court V-I1 (native leg): appender batch ingest throughput.
//! Claim: sustained >= 100,000 rows/sec, single-threaded, no sleeps.
//! Protocol per docs/design/ingest-throughput-plan.md:
//!   - pre-built batches (construction not on the clock)
//!   - one uncounted warmup batch
//!   - >= 5s continuous ingest (Instant-based deadline check, no sleeps)
//!   - throughput = total rows flushed / elapsed wall time
//!
//! Second leg: BP3-shaped segment rows (SLA-3: 100k transitions/sec).

use frozen_duckdb::{Connection, ToSql};
use std::time::Instant;

const BATCH_ROWS: usize = 100_000;
const MIN_SECONDS: u64 = 5;
const MIN_ROWS_PER_SEC: f64 = 100_000.0;

/// Deterministic splitmix64 sequence; batches built BEFORE any clock starts.
fn seeded_batch() -> Vec<(i64, i64, i64, i64, i64, f64)> {
    let mut state: u64 = 0x9E3779B97F4A7C15;
    (0..BATCH_ROWS)
        .map(|_| {
            state = state.wrapping_add(0x9E3779B97F4A7C15);
            let mut z = state;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
            let r = z ^ (z >> 31);
            (
                r as i64,
                (r >> 32) as i64,
                (r >> 16) as i64,
                (r >> 8) as i64,
                (r >> 48) as i64,
                ((r >> 56) % 10_000) as f64 / 100.0,
            )
        })
        .collect()
}

fn machine_specs() -> String {
    let sh = |args: &[&str]| {
        std::process::Command::new("sysctl")
            .args(args)
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|| "unknown".into())
    };
    let cpu = sh(&["-n", "machdep.cpu.brand_string"]);
    let mem_gb = sh(&["-n", "hw.memsize"])
        .parse::<f64>()
        .map(|b| b / 1073741824.0)
        .unwrap_or(0.0);
    let cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(0);
    format!(
        "cpu='{}' cores={} mem={:.1}GB os={} target-cpu=host-default(test profile)",
        cpu,
        cores,
        mem_gb,
        std::env::consts::OS
    )
}

/// Sustained ingest: appender-append+flush of pre-built batches until the
/// Instant-based deadline has elapsed. No sleeps anywhere. Returns
/// (rows_per_sec, total_rows, elapsed_secs).
fn run_sustained(
    conn: &Connection,
    table: &str,
    batch: &[(i64, i64, i64, i64, i64, f64)],
) -> (f64, u64, f64) {
    let mut app = conn.appender(table).expect("appender");
    let mut total_rows: u64 = 0;
    let start = Instant::now();
    while start.elapsed().as_secs() < MIN_SECONDS {
        app.append_rows(
            batch
                .iter()
                .map(|r| -> [&dyn ToSql; 3] { [&r.0, &r.1, &r.5] }),
        )
        .expect("append batch");
        app.flush().expect("flush");
        total_rows += batch.len() as u64;
    }
    let elapsed = start.elapsed().as_secs_f64();
    let _ = app.flush();
    (total_rows as f64 / elapsed, total_rows, elapsed)
}

/// BP3 segment leg: (action_id, from_state, to_state, count, t_start, t_end).
fn run_segment_leg(conn: &Connection, batch: &[(i64, i64, i64, i64, i64, f64)]) -> (f64, u64, f64) {
    let mut app = conn.appender("bp3_segments").expect("appender");
    let mut total_rows: u64 = 0;
    let start = Instant::now();
    while start.elapsed().as_secs() < MIN_SECONDS {
        app.append_rows(
            batch
                .iter()
                .map(|r| -> [&dyn ToSql; 6] { [&r.0, &r.1, &r.2, &r.3, &r.4, &r.5] }),
        )
        .expect("append batch");
        app.flush().expect("flush");
        total_rows += batch.len() as u64;
    }
    let elapsed = start.elapsed().as_secs_f64();
    let _ = app.flush();
    (total_rows as f64 / elapsed, total_rows, elapsed)
}

fn assert_bound(name: &str, rows_per_sec: f64, total_rows: u64, elapsed: f64) {
    println!(
        "{}: {:.0} rows/sec over {:.2}s ({} rows)",
        name, rows_per_sec, elapsed, total_rows
    );
    assert!(
        rows_per_sec >= MIN_ROWS_PER_SEC,
        "{} sustained {:.0} rows/sec < required {:.0}",
        name,
        rows_per_sec,
        MIN_ROWS_PER_SEC
    );
}

#[test]
fn gate_ingest_throughput_native() {
    println!("machine specs: {}", machine_specs());
    let wide = seeded_batch();

    // Leg 1: packed 3-column fixed-width (i64, i64, f64).
    let conn = Connection::open_in_memory().expect("open db");
    conn.execute_batch("CREATE TABLE target(k BIGINT, v BIGINT, score DOUBLE);")
        .expect("ddl");
    // one uncounted warmup batch through the same path
    {
        let mut app = conn.appender("target").expect("appender");
        app.append_rows(
            wide.iter()
                .map(|r| -> [&dyn ToSql; 3] { [&r.0, &r.1, &r.5] }),
        )
        .expect("warmup");
        app.flush().expect("warmup flush");
    }
    conn.execute("DELETE FROM target", [])
        .expect("clear warmup");

    let (rps, rows, el) = run_sustained(&conn, "target", &wide);
    let n: u64 = conn
        .query_row("SELECT count(*) FROM target", [], |r| r.get(0))
        .expect("count");
    assert_eq!(n, rows, "SQL count(*) must equal ingested rows");
    assert_bound("leg1_packed_3col", rps, rows, el);

    // Leg 2: BP3-shaped segment rows (SLA-3: 100k transitions/sec).
    conn.execute_batch(
        "CREATE TABLE bp3_segments(action_id BIGINT, from_state BIGINT, to_state BIGINT, count BIGINT, t_start BIGINT, t_end DOUBLE);",
    )
    .expect("ddl bp3");
    {
        let mut app = conn.appender("bp3_segments").expect("appender");
        app.append_rows(
            wide.iter()
                .map(|r| -> [&dyn ToSql; 6] { [&r.0, &r.1, &r.2, &r.3, &r.4, &r.5] }),
        )
        .expect("bp3 warmup");
        app.flush().expect("bp3 warmup flush");
    }
    conn.execute("DELETE FROM bp3_segments", [])
        .expect("clear bp3 warmup");

    let (rps2, rows2, el2) = run_segment_leg(&conn, &wide);
    let n2: u64 = conn
        .query_row("SELECT count(*) FROM bp3_segments", [], |r| r.get(0))
        .expect("count bp3");
    assert_eq!(n2, rows2, "SQL count(*) must equal ingested bp3 rows");
    assert_bound("leg2_bp3_segments", rps2, rows2, el2);
}
