//! Doctor OCEL tests (TEST-018/019/020) — scaled compression + rehydration
//! court.
//!
//! Spec sources:
//! - `docs/design/compression-proof-protocol.md` (NFR-03 / AC-4): ingest the
//!   same deterministic corpus two ways; S_raw = a real JSON-lines file on
//!   disk, S_fdb = the compressed segment db file post-CHECKPOINT; ratio is
//!   measured on-disk file bytes, never asserted.
//! - `docs/design/ocel-sovereign-schema-validation.md` (D2/D3): the
//!   spec-literal NULLIF rehydration guard silently drops singleton
//!   segments (generate_series with a NULL step returns 0 rows); this court
//!   uses the hardened COALESCE form:
//!   `to_microseconds(COALESCE((date_diff('microsecond', t_start, t_end)
//!   / NULLIF(transition_count - 1, 0))::BIGINT, 1))`.
//!
//! Real engine, real files, no mocks (Chicago school).

use frozen_duckdb::Connection;
use std::fs::{self, File};
use std::io::Write as IoWrite;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// Task default 100k; the compression-proof-protocol spec corpus is 1M. Env
/// override (FDOCEL_EVENTS) lets a scaled receipt run the same court at the
/// spec's contract size.
fn events() -> i64 {
    std::env::var("FDOCEL_EVENTS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(100_000)
}

const BLOCK: i64 = 100;
const ESCAPES: i64 = 5;
/// NFR-03 pass line: bytes(S_fdb) <= 0.10 * bytes(S_raw) — the compressed
/// db must be at least 90% smaller than the JSONL baseline.
const RATIO_PASS: f64 = 0.10;
/// The singleton-segment witness (D2): event n == 700 is pulled out of the
/// regular `n / BLOCK` grouping into a disjoint block so the corpus contains
/// exactly one transition_count = 1 segment. Without it the NULLIF vacuity
/// is invisible to conservation.
const SINGLETON_N: i64 = 700;
const SINGLETON_BLOCK: i64 = 1_000_001;

/// Cyclic steady-state shape: 5,000 objects, 4 actions, 12 channels, all
/// cycling at block granularity, uniform 1-second cadence so the segment
/// span-relative step reconstructs timestamps exactly (D3 law: span-relative
/// microsecond math, not literal-second steps).
const ACTIONS: [&str; 4] = ["create-order", "pick-item", "pack-item", "ship-order"];
const CHANNELS: [&str; 12] = [
    "web", "mobile", "pos", "partner", "edi", "phone", "kiosk", "email", "chat", "api", "store",
    "fax",
];

/// n -> segment block id. n == 700 is pulled into a disjoint singleton block.
fn block_of(n: i64) -> i64 {
    if n == SINGLETON_N {
        SINGLETON_BLOCK
    } else {
        n / BLOCK
    }
}

/// Deterministic corpus event for index n: (oid, action, channel, epoch_s).
fn event(n: i64) -> (String, &'static str, &'static str, i64) {
    let b = block_of(n);
    (
        format!("obj-{}", b % 5_000),
        ACTIONS[(b % 4) as usize],
        CHANNELS[(b % 12) as usize],
        1_767_225_600 + n, // 2026-01-01 00:00:00 UTC, 1 s cadence
    )
}

/// Epoch seconds -> "YYYY-MM-DD HH:MM:SS" (UTC, no external crates).
fn chrono_compat(ts: i64) -> String {
    let days = ts.div_euclid(86_400);
    let secs = ts.rem_euclid(86_400);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!(
        "{y:04}-{m:02}-{d:02} {:02}:{:02}:{:02}",
        secs / 3600,
        (secs % 3600) / 60,
        secs % 60
    )
}

fn temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before epoch")
        .as_nanos();
    let dir =
        std::env::temp_dir().join(format!("fdocel-{}-{}-{}", label, std::process::id(), nanos));
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

/// Fresh persistent database on the frozen engine.
fn open_db(path: &Path) -> Connection {
    if path.exists() {
        fs::remove_file(path).expect("remove stale db");
    }
    Connection::open(path).expect("open frozen-duckdb connection")
}

/// Scalar fetch (single row, single column, stringified).
fn scalar(conn: &Connection, sql: &str) -> String {
    let mut out = String::new();
    conn.query_row(sql, [], |r| {
        let v: String = r.get(0)?;
        out = v;
        Ok(())
    })
    .unwrap_or_else(|e| panic!("scalar `{sql}`: {e}"));
    out
}

/// Scalar fetch as i64 (BIGINT column, typed read).
fn scalar_i64(conn: &Connection, sql: &str) -> i64 {
    let mut out: i64 = 0;
    conn.query_row(sql, [], |r| {
        out = r.get(0)?;
        Ok(())
    })
    .unwrap_or_else(|e| panic!("scalar_i64 `{sql}`: {e}"));
    out
}

/// Write leg A's baseline: the deterministic corpus as a real JSON-lines
/// file on disk (S_raw). Returns bytes.
fn write_jsonl_baseline(dir: &Path) -> u64 {
    let path = dir.join("nfr03_baseline.jsonl");
    let mut f = File::create(&path).expect("create jsonl baseline");
    for n in 0..events() {
        let (oid, action, channel, ts) = event(n);
        let dt = chrono_compat(ts);
        writeln!(
            f,
            r#"{{"ocel:id":"e-{n}","ocel:timestamp":"{dt}","ocel:type":"{action}","ocel:attributes":{{"channel":"{channel}","epoch":{ts}}},"ocel:relationships":[{{"ocel:oid":"{oid}","ocel:type":"orders"}}]}}"#
        )
        .expect("write jsonl line");
    }
    fs::metadata(&path).expect("stat jsonl").len()
}

/// Probe whether the engine accepts VARIANT column DDL (spec form, Q1) or
/// refuses it (X5 finding D1 on 1.4.1) — the court records which form ran
/// and the escapes payload column follows it.
fn probe_ddl_form(conn: &Connection) -> &'static str {
    match conn
        .execute_batch("CREATE OR REPLACE TABLE _variant_probe(x VARIANT);")
        .map(|_| conn.execute_batch("DROP TABLE _variant_probe;"))
    {
        Ok(_) => {
            eprintln!("ocel-court: VARIANT DDL ACCEPTED (D1 updated on this engine)");
            "VARIANT"
        }
        Err(e) => {
            eprintln!("ocel-court: VARIANT DDL refused ({e}); falling back to JSON");
            "JSON"
        }
    }
}

/// Payload column DDL + insert cast for the probed form.
fn payload_ddl(form: &str) -> &'static str {
    if form == "VARIANT" {
        "VARIANT"
    } else {
        "JSON"
    }
}

fn payload_literal(form: &str, json: &str) -> String {
    if form == "VARIANT" {
        format!("'{json}'::VARIANT")
    } else {
        format!("'{json}'::JSON")
    }
}

/// Full-fidelity escape payload for row i.
fn escape_payload(i: i64) -> String {
    format!(
        r#"{{"ocel:id":"esc-{i}","ocel:type":"escape","ocel:attributes":{{"reason":"NON_UNIFORM_CADENCE","entropy":"{i}-{i}{i}{i}-payload-{}"}}}}"#,
        i * 7919
    )
}

/// The hardened rehydration view (Q5' form; D2/D3 law). Span-relative
/// microsecond step with COALESCE heal for the singleton segment.
const REHYDRATION_VIEW_SQL: &str = "
CREATE VIEW ocel_rehydrated AS
SELECT
    s.segment_id,
    s.oid,
    s.action,
    s.channel,
    unnest(generate_series(
        s.t_start,
        s.t_end,
        to_microseconds(COALESCE(
            (date_diff('microsecond', s.t_start, s.t_end)
             / NULLIF(s.transition_count - 1, 0))::BIGINT,
            1)))
    ) AS ts
FROM ocel_conforming_segments s;";

/// Chunked multi-row VALUES insert of the staging table.
/// Extension-independent (no read_json dependency). Returns ingest duration
/// for the events/sec receipt.
fn ingest_staging(conn: &Connection) -> std::time::Duration {
    let t0 = Instant::now();
    conn.execute_batch(
        "CREATE TABLE events_staging(
            n BIGINT, oid VARCHAR, action VARCHAR, channel VARCHAR, ts TIMESTAMP);",
    )
    .expect("create staging");
    const CHUNK: i64 = 1_000;
    let mut start = 0;
    while start < events() {
        let mut sql = String::with_capacity(1 << 17);
        sql.push_str("INSERT INTO events_staging VALUES ");
        let end = (start + CHUNK).min(events());
        for n in start..end {
            if n > start {
                sql.push(',');
            }
            let (oid, action, channel, ts) = event(n);
            sql.push_str(&format!(
                " ({}, '{}', '{}', '{}', '{}')",
                n,
                oid,
                action,
                channel,
                chrono_compat(ts)
            ));
        }
        sql.push(';');
        conn.execute_batch(&sql).expect("staging chunk insert");
        start = end;
    }
    t0.elapsed()
}

/// Build leg B (compressed segment db, S_fdb).
/// Returns (bytes_B, ddl_form, ingest_duration).
fn build_compressed_db(dir: &Path) -> (u64, &'static str, std::time::Duration) {
    let db_b = dir.join("nfr03_compressed.db");
    let conn = open_db(&db_b);
    let ddl_form = probe_ddl_form(&conn);
    eprintln!("ocel-court: DDL form used: {ddl_form}");

    conn.execute_batch(&format!(
        "CREATE TABLE ocel_conforming_segments(
            segment_id BIGINT,
            oid VARCHAR,
            action VARCHAR,
            channel VARCHAR,
            t_start TIMESTAMP,
            t_end TIMESTAMP,
            transition_count BIGINT);
         CREATE TABLE ocel_escapes(
            escape_id BIGINT,
            oid VARCHAR,
            payload {});",
        payload_ddl(ddl_form)
    ))
    .expect("create leg B DDL");

    // 5 escape rows: full-fidelity raw JSON, never folded.
    let mut esc = String::from("INSERT INTO ocel_escapes VALUES ");
    for i in 0..ESCAPES {
        if i > 0 {
            esc.push(',');
        }
        esc.push_str(&format!(
            " ('{}', 'esc-obj-{}', {})",
            i,
            i,
            payload_literal(ddl_form, &escape_payload(i))
        ));
    }
    esc.push(';');
    conn.execute_batch(&esc).expect("insert escapes");

    // Stage -> fold -> verify structurally -> drop raw -> checkpoint.
    let ingest_dur = ingest_staging(&conn);

    // Fold staging into RLE segments using the SAME grouping as block_of()
    // (n/BLOCK, singleton pulled out) — grouping drift here would make the
    // D2 singleton witness silently vanish (the vacuity this court guards).
    conn.execute_batch(&format!(
        "INSERT INTO ocel_conforming_segments
         SELECT row_number() OVER (ORDER BY b) AS segment_id,
                any_value(oid)    AS oid,
                any_value(action) AS action,
                any_value(channel) AS channel,
                min(ts) AS t_start,
                max(ts) AS t_end,
                count(*)::BIGINT AS transition_count
         FROM (SELECT *, CASE WHEN n = {SINGLETON_N} THEN {SINGLETON_BLOCK}
                              ELSE n / {BLOCK} END AS b
               FROM events_staging) q
         GROUP BY b;"
    ))
    .expect("fold staging into segments");

    // Segment-level structural check: rehydrated vs raw staging,
    // field-by-field, both EXCEPT directions, zero drift tolerated. The view
    // is created while staging still exists so the diff is over real rows.
    conn.execute_batch(REHYDRATION_VIEW_SQL)
        .expect("create view");
    let drift: i64 = scalar_i64(
        &conn,
        "SELECT
            (SELECT count(*) FROM (
                SELECT oid, action, channel, epoch_us(ts) AS t FROM ocel_rehydrated
                EXCEPT
                SELECT oid, action, channel, epoch_us(ts) FROM events_staging))
          + (SELECT count(*) FROM (
                SELECT oid, action, channel, epoch_us(ts) FROM events_staging
                EXCEPT
                SELECT oid, action, channel, epoch_us(ts) FROM ocel_rehydrated));",
    );
    assert_eq!(
        drift, 0,
        "TEST-018 structural drift: {drift} divergent rows"
    );

    conn.execute_batch("DROP VIEW ocel_rehydrated; DROP TABLE events_staging; CHECKPOINT;")
        .expect("drop raw + checkpoint B");
    drop(conn);

    (
        fs::metadata(&db_b).expect("stat B").len(),
        ddl_form,
        ingest_dur,
    )
}

/// Reopen the checkpointed leg B db and run the rehydration court.
fn rehydration_court(db_b: &Path) {
    let conn = Connection::open(db_b).expect("reopen leg B");
    conn.execute_batch(REHYDRATION_VIEW_SQL)
        .expect("recreate view after reopen");

    // F-C3 conservation: sum(transition_count) == events == rehydrated count.
    let total_transitions = scalar_i64(
        &conn,
        "SELECT sum(transition_count) FROM ocel_conforming_segments;",
    );
    let rehydrated = scalar_i64(&conn, "SELECT count(*) FROM ocel_rehydrated;");
    assert_eq!(
        total_transitions,
        events(),
        "conservation: sum(transition_count)"
    );
    assert_eq!(rehydrated, events(), "conservation: rehydrated count");

    // Singleton segment present and rehydrated exactly once (D2 witness).
    let singleton = scalar_i64(
        &conn,
        "SELECT transition_count FROM ocel_conforming_segments WHERE transition_count = 1;",
    );
    assert_eq!(singleton, 1, "singleton segment must exist (D2 witness)");

    // Timestamp bounds: every rehydrated ts within its segment's
    // [t_start, t_end].
    let out_of_bounds = scalar_i64(
        &conn,
        "SELECT count(*) FROM ocel_rehydrated r
         JOIN ocel_conforming_segments s USING (segment_id)
         WHERE r.ts < s.t_start OR r.ts > s.t_end;",
    );
    assert_eq!(out_of_bounds, 0, "rehydrated ts outside [t_start, t_end]");

    // Escapes: exactly ESCAPES rows, full fidelity, absent from segments.
    let esc_count = scalar_i64(&conn, "SELECT count(*) FROM ocel_escapes;");
    assert_eq!(esc_count, ESCAPES);

    // Full-fidelity read-back: each escape's entropy attribute round-trips
    // byte-exact through the payload column (JSON or VARIANT, per probe).
    for i in 0..ESCAPES {
        let want = format!("{}-{0}{0}{0}-payload-{}", i, i * 7919);
        let got = scalar(
            &conn,
            &format!(
                "SELECT json_extract_string(payload, '$.\"ocel:attributes\".entropy')
                 FROM ocel_escapes WHERE escape_id = {i};"
            ),
        );
        assert_eq!(got, want, "escape {i} entropy round-trip");
    }

    let overlap = scalar_i64(
        &conn,
        "SELECT count(*) FROM ocel_escapes e
         WHERE EXISTS (SELECT 1 FROM ocel_conforming_segments s WHERE s.oid = e.oid);",
    );
    assert_eq!(overlap, 0, "escape oids must be absent from segments");
}

/// TEST-018/019/020 battery — the whole court runs once; each test reads the
/// shared receipt. Build cost is dominated by the 100k-row ingest, so the
/// fixture is shared via OnceLock.
fn court() -> &'static CourtReceipt {
    static RECEIPT: std::sync::OnceLock<CourtReceipt> = std::sync::OnceLock::new();
    RECEIPT.get_or_init(|| {
        let dir = temp_dir("court");
        let t0 = Instant::now();

        // Leg A: raw JSON lines file (S_raw).
        let bytes_a = write_jsonl_baseline(&dir);

        // Leg B: compressed segment db (S_fdb), post-CHECKPOINT bytes.
        let (bytes_b, ddl_form, ingest_dur) = build_compressed_db(&dir);

        // Rehydration + escapes courts over the checkpointed db.
        let db_b = dir.join("nfr03_compressed.db");
        rehydration_court(&db_b);

        let ratio = bytes_b as f64 / bytes_a as f64;
        let events_per_sec = events() as f64 / ingest_dur.as_secs_f64();
        let receipt = CourtReceipt {
            events: events(),
            bytes_a,
            bytes_b,
            ratio,
            ddl_form,
            events_per_sec,
            wall: t0.elapsed(),
            _dir: dir,
        };
        println!(
            "OCEL-COURT events={} seed=20261019 ddl_form={} bytes_A(S_raw)={} bytes_B(S_fdb)={} ratio={:.5} events_per_sec={:.0} wall_ms={} verdict={}",
            receipt.events,
            receipt.ddl_form,
            receipt.bytes_a,
            receipt.bytes_b,
            receipt.ratio,
            receipt.events_per_sec,
            receipt.wall.as_millis(),
            if receipt.ratio <= RATIO_PASS { "PASS" } else { "FAIL" }
        );
        receipt
    })
}

struct CourtReceipt {
    events: i64,
    bytes_a: u64,
    bytes_b: u64,
    ratio: f64,
    ddl_form: &'static str,
    events_per_sec: f64,
    wall: std::time::Duration,
    _dir: PathBuf,
}

#[test]
fn test_018_rehydration_equivalence_and_conservation() {
    let r = court();
    assert_eq!(r.events, events());
    println!(
        "TEST-018 rehydration conservation + timestamp bounds: green (drift=0, ratio={:.5})",
        r.ratio
    );
}

#[test]
fn test_019_escape_rows_full_fidelity_and_absent() {
    let r = court();
    assert!(r.bytes_b > 0);
    println!(
        "TEST-019 escapes ledger: {} full-fidelity rows (ddl_form={}), entropy round-trip exact, absent from segments",
        ESCAPES, r.ddl_form
    );
}

#[test]
fn test_020_compression_ratio_measured() {
    let r = court();
    println!(
        "TEST-020 ratio={:.5} (bytes_B={} / bytes_A={}) pass-line={}",
        r.ratio, r.bytes_b, r.bytes_a, RATIO_PASS
    );
    assert!(
        r.ratio <= RATIO_PASS,
        "ratio {:.5} exceeds pass line {} — cyclic steady-state shape must compress >= 90%",
        r.ratio,
        RATIO_PASS
    );
}
