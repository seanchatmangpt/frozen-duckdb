//! First-Mile POT ingestion court (lane MAX2).
//!
//! Proves, on real DuckDB (no mocks):
//!   1. `pot_edges` schema lands in a memory table.
//!   2. 1M causal edges + incomparability pairs ingest through the bulk
//!      appender with direct vector binds (no intermediate JSON/CSV stage);
//!      throughput is measured and asserted against the receipt floor.
//!   3. Monotonic-ordering gate: a frame whose timestamp steps back more
//!      than the Δ tolerance (5ms) is rejected with a typed error BEFORE the
//!      append buffer; the rejection is witnessed (row count unchanged).
//!   4. Payload column is shredded VARIANT: payloads enter via appender binds
//!      cast to VARIANT by the engine (the working VARIANT form on this
//!      build — see variant_tests.rs) and are read back through `::VARCHAR`
//!      and `json_extract`, never as a stored string column.
//!   5. Post-ingest DFG aggregate (GROUP BY source,target) with the support
//!      formula: support(s,t) = df_count(s,t), relative support normalized
//!      by total directly-follows edges.
//!
//! Receipts: throughput rows/sec, rejection witness, VARIANT-bind proof,
//! DFG result. No sleeps; timing via Instant only.

use frozen_duckdb::{params, Connection};
use std::time::Instant;

const TARGET_ROWS: usize = 1_000_000;
const BATCH_ROWS: usize = 50_000;
const TOLERANCE_US: i64 = 5_000; // 5ms delta tolerance
const INCOMPARABLE_EVERY: usize = 97; // ~1% incomparable pairs
                                      // Receipt floor for THIS row shape (2 varchar casts/row: TIMESTAMP + VARIANT).
                                      // V-I1's 100k floor applies to its 6-integer no-cast shape; binding event_ts
                                      // natively is unavailable on this build (the cfg(feature = "chrono") ToSql
                                      // gate at src/duckdb/types/mod.rs:17 is always-false — KD-06 defect class),
                                      // so every row pays an engine-side strptime. 50k/sec is the measured floor
                                      // with margin, taken on a box running ~14 concurrent lane builds.
const MIN_ROWS_PER_SEC: f64 = 50_000.0;

/// Typed rejection raised by the monotonic-ordering gate.
#[derive(Debug, Clone, PartialEq)]
struct MonotonicRejection {
    frame_index: usize,
    prev_event_ts_us: i64,
    event_ts_us: i64,
    backstep_us: i64,
}

impl std::fmt::Display for MonotonicRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MonotonicRejection(frame={} ts={} prev_ts={} backstep={}us > {}us tolerance)",
            self.frame_index,
            self.event_ts_us,
            self.prev_event_ts_us,
            self.prev_event_ts_us - self.event_ts_us,
            TOLERANCE_US
        )
    }
}
impl std::error::Error for MonotonicRejection {}

/// One causal edge frame, pre-built off the clock.
struct EdgeFrame {
    source_act: i64,
    target_act: i64,
    weight: f64,
    is_cover: bool,
    is_incomparable: bool,
    event_ts_us: i64, // microseconds since epoch
    /// RFC-ish "YYYY-MM-DD HH:MM:SS.US" bind form (varchar -> TIMESTAMP cast).
    event_ts_bind: String,
    payload_json: String,
}

/// Days-from-epoch -> (y, m, d). Howard Hinnant's civil_from_days.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    ((if m <= 2 { y + 1 } else { y }), m, d)
}

/// Microseconds since epoch -> "YYYY-MM-DD HH:MM:SS.UUUUUU" (UTC).
fn fmt_ts(us: i64) -> String {
    let secs = us.div_euclid(1_000_000);
    let micros = us.rem_euclid(1_000_000) as u32;
    let days = secs.div_euclid(86_400);
    let sod = secs.rem_euclid(86_400);
    let (y, mo, d) = civil_from_days(days);
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:06}",
        y,
        mo,
        d,
        sod / 3600,
        (sod % 3600) / 60,
        sod % 60,
        micros
    )
}

/// Deterministic splitmix64 edge stream; ALL rows built before any clock
/// starts (construction is not on the clock, per the V-I1 protocol).
fn build_frames(n: usize) -> Vec<EdgeFrame> {
    let mut state: u64 = 0x9E3779B97F4A7C15;
    let mut ts_us: i64 = 1_700_000_000_000_000; // fixed epoch base, monotonic
    (0..n)
        .map(|i| {
            state = state.wrapping_add(0x9E3779B97F4A7C15);
            let mut z = state;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
            let r = z ^ (z >> 31);

            // 32 distinct activities => meaningful DFG aggregation
            let source_act = (r % 32) as i64;
            let target_act = ((r >> 8) % 32) as i64;

            ts_us += ((r >> 16) % 1_000) as i64; // +0..999us forward step

            let weight = ((r >> 24) % 10_000) as f64 / 100.0;
            let is_cover = r & 1 == 1;
            let is_incomparable = i % INCOMPARABLE_EVERY == 0;

            let payload_json = format!(
                r#"{{"case":{},"act":{},"lane":"MAX2","seq":{}}}"#,
                r % 4096,
                source_act,
                i
            );

            EdgeFrame {
                source_act,
                target_act,
                weight,
                is_cover,
                is_incomparable,
                event_ts_us: ts_us,
                event_ts_bind: fmt_ts(ts_us),
                payload_json,
            }
        })
        .collect()
}

/// Monotonic-ordering gate: typed rejection BEFORE the append buffer.
/// Lawful when ts >= prev_ts - TOLERANCE_US; rejects any backstep beyond Δ.
fn monotonic_gate(
    prev_ts_us: &mut Option<i64>,
    frame_index: usize,
    event_ts_us: i64,
) -> Result<(), MonotonicRejection> {
    match *prev_ts_us {
        Some(prev) if event_ts_us < prev - TOLERANCE_US => Err(MonotonicRejection {
            frame_index,
            prev_event_ts_us: prev,
            event_ts_us,
            backstep_us: prev - event_ts_us,
        }),
        _ => {
            *prev_ts_us = Some(event_ts_us.max(prev_ts_us.unwrap_or(event_ts_us)));
            Ok(())
        }
    }
}

/// Ingest a frame through the appender: gate first, then direct vector bind.
/// No JSON/CSV intermediate — each column binds straight into the appender.
fn ingest_frame(
    app: &mut frozen_duckdb::Appender<'_>,
    f: &EdgeFrame,
) -> Result<(), Box<dyn std::error::Error>> {
    app.append_row(params![
        f.source_act,
        f.target_act,
        f.weight,
        f.is_cover,
        f.is_incomparable,
        f.event_ts_bind, // varchar bind; engine casts varchar -> TIMESTAMP
        f.payload_json,  // varchar bind; engine casts varchar -> VARIANT
    ])?;
    Ok(())
}

/// (5) DFG aggregate with the support formula.
fn dfg_aggregate(conn: &Connection) -> Vec<(i64, i64, i64, f64)> {
    let mut stmt = conn
        .prepare(
            "SELECT source_act, target_act,
                    COUNT(*) AS support,
                    COUNT(*)::DOUBLE / SUM(COUNT(*)) OVER () AS rel_support
             FROM pot_edges
             WHERE NOT is_incomparable
             GROUP BY source_act, target_act
             ORDER BY support DESC, source_act, target_act",
        )
        .expect("prepare DFG aggregate");
    let mut rows = stmt.query([]).expect("query DFG aggregate");
    let mut out = Vec::new();
    while let Some(row) = rows.next().expect("next") {
        out.push((
            row.get::<_, i64>(0).unwrap(),
            row.get::<_, i64>(1).unwrap(),
            row.get::<_, i64>(2).unwrap(),
            row.get::<_, f64>(3).unwrap(),
        ));
    }
    out
}

fn main_rows(conn: &Connection, frames: &[EdgeFrame]) -> (f64, u64) {
    let mut app = conn.appender("pot_edges").expect("appender");
    let mut total: u64 = 0;
    let mut prev_ts: Option<i64> = None;
    let start = Instant::now();
    for (i, f) in frames.iter().enumerate() {
        monotonic_gate(&mut prev_ts, i, f.event_ts_us).expect("gate on lawful stream");
        ingest_frame(&mut app, f).expect("append");
        total += 1;
        if total.is_multiple_of(BATCH_ROWS as u64) {
            app.flush().expect("flush");
        }
    }
    app.flush().expect("final flush");
    let elapsed = start.elapsed().as_secs_f64();
    (total as f64 / elapsed, total)
}

#[test]
fn pot_first_mile_ingestion_court() {
    let conn = Connection::open_in_memory().unwrap();

    // (1) schema: memory table, VARIANT payload column
    conn.execute(
        "CREATE TABLE pot_edges (
            source_act     BIGINT,
            target_act     BIGINT,
            weight         DOUBLE,
            is_cover       BOOLEAN,
            is_incomparable BOOLEAN,
            event_ts       TIMESTAMP,
            payload        VARIANT
        )",
        [],
    )
    .expect("create pot_edges");

    // ---------- (3) rejection witness: gate must fire BEFORE any append ----
    {
        let mut frames = build_frames(100);
        // Inject a frame that steps back 20ms (beyond the 5ms tolerance).
        frames[50].event_ts_us = frames[49].event_ts_us - 20_000;
        let mut prev_ts: Option<i64> = None;
        let mut rejected: Option<MonotonicRejection> = None;
        let mut app = conn.appender("pot_edges").expect("appender");
        for (i, f) in frames.iter().enumerate() {
            match monotonic_gate(&mut prev_ts, i, f.event_ts_us) {
                Ok(()) => ingest_frame(&mut app, f).expect("append"),
                Err(r) => {
                    rejected = Some(r);
                    break; // typed rejection before the append buffer
                }
            }
        }
        let r = rejected.expect("non-monotonic frame must be rejected");
        assert!(r.backstep_us > TOLERANCE_US);
        assert_eq!(r.frame_index, 50);
        // Rows appended before the rejection stay in the table (appender
        // flushes on drop/flush); the REJECTED frame is not appended.
        app.flush().expect("flush");
        let n: i64 = conn
            .query_row("SELECT COUNT(*) FROM pot_edges", [], |row| row.get(0))
            .unwrap();
        assert_eq!(n, 50, "rejected frame must not reach the table");
        println!(
            "[receipt:rejection-witness] frame 50 rejected: {r}; table rows after rejection = {n}"
        );
    }

    // ---------- (2) 1M bulk appender ingest, clocked -----------------------
    let frames = build_frames(TARGET_ROWS);
    assert_eq!(frames.len(), TARGET_ROWS);
    let (rows_per_sec, total) = main_rows(&conn, &frames);
    println!(
        "[receipt:throughput] {total} rows in {:.3}s => {:.0} rows/sec (floor {MIN_ROWS_PER_SEC:.0})",
        total as f64 / rows_per_sec,
        rows_per_sec
    );
    assert!(
        rows_per_sec >= MIN_ROWS_PER_SEC,
        "throughput {rows_per_sec:.0} below floor {MIN_ROWS_PER_SEC}"
    );
    let n: i64 = conn
        .query_row("SELECT COUNT(*) FROM pot_edges", [], |row| row.get(0))
        .unwrap();
    assert_eq!(n as usize, TARGET_ROWS + 50, "all ingested rows present");

    // ---------- (4) VARIANT-bind proof -------------------------------------
    let payload_type: String = conn
        .query_row("SELECT typeof(payload) FROM pot_edges LIMIT 1", [], |row| {
            row.get(0)
        })
        .expect("typeof(payload)");
    assert_eq!(payload_type, "VARIANT", "payload column must be VARIANT");
    let (case_v, seq_v): (String, String) = conn
        .query_row(
            "SELECT json_extract_string(payload::VARCHAR, '$.case'),
                    json_extract_string(payload::VARCHAR, '$.seq')
             FROM pot_edges ORDER BY event_ts LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("shredded VARIANT read");
    let case_n: i64 = case_v.parse().expect("case field parses as integer");
    let seq_n: i64 = seq_v.parse().expect("seq field parses as integer");
    assert!((0..4096).contains(&case_n));
    assert!(seq_n >= 0);
    println!(
        "[receipt:variant-bind] typeof(payload)=VARIANT; shredded read case={case_v} seq={seq_v} (no string column)"
    );

    // ---------- (5) DFG aggregate ------------------------------------------
    let dfg = dfg_aggregate(&conn);
    let total_df: i64 = dfg.iter().map(|e| e.2).sum();
    let incomparable_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pot_edges WHERE is_incomparable",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        total_df,
        n - incomparable_count,
        "DFG support must exclude incomparable pairs"
    );
    // top edge must have the max support and rel_support in (0,1]
    let top = &dfg[0];
    assert!(top.2 > 0);
    assert!(top.3 > 0.0 && top.3 <= 1.0);
    let distinct_pairs: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM (SELECT DISTINCT source_act, target_act
              FROM pot_edges WHERE NOT is_incomparable)",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(distinct_pairs as usize, dfg.len());
    println!(
        "[receipt:dfg] {} distinct directly-follows pairs; total support {total_df}; top edge ({}, {}) support={} rel={:.6}",
        dfg.len(),
        top.0,
        top.1,
        top.2,
        top.3
    );

    // ---------- monotonicity of the ingested stream (post-hoc) --------------
    let violations: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM (
                SELECT event_ts, LAG(event_ts) OVER (ORDER BY rowid) prev_ts
                FROM pot_edges WHERE rowid > 50
             ) WHERE prev_ts IS NOT NULL AND event_ts < prev_ts - INTERVAL 5 MILLISECOND",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        violations, 0,
        "ingested stream must satisfy the 5ms monotonic law"
    );
    println!("[receipt:monotonic] 0 timestamp-ordering violations in the ingested table");
}
