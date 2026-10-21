//! TEST-FLEET-004 native leg (lane T2) — marine container-telemetry fleet
//! class D battery over the frozen-duckdb engine.
//!
//! Legs:
//! 1. Chaos court (two-process): `scripts/courts/fleet_d_chaos.sh` kills -9
//!    the t2writer fixture process group mid-transaction at 200k/260k events
//!    in-flight; the checker must remount cleanly (PRAGMA database_size +
//!    storage_info), land in one of exactly two legal states (last-committed
//!    sentinel or in-txn sentinel), and satisfy the closed-form checksum —
//!    never a torn third state. Non-vacuity: a kill -0 SHIM-COURT leg must be
//!    DETECTED as a vacuous gate.
//! 2. Compression: over 1M completed marine events, measured on-disk bytes:
//!    (raw_jsonl - db) / raw >= 0.95.
//! 3. Satellite sync vector: the segment table serialized as a per-segment
//!    MD5 digest stream (32 hex chars x 10k segments) plus counts must fit
//!    <= 500 KB for delta-sync to a satellite node.
//!
//! Real engine, real files, real subprocesses, no mocks (Chicago school).

use frozen_duckdb::Connection;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const EVENTS: i64 = 1_000_000;
const BLOCK: i64 = 100;
const RATIO_PASS: f64 = 0.95;
const SATELLITE_BUDGET: usize = 500 * 1024;
const COURT: &str = "/Users/sac/frozen-duckdb/scripts/courts/fleet_d_chaos.sh";
const T2_BIN: &str = "/tmp/t2writer/target/release/t2writer";
const RECEIPT: &str = "/tmp/value_audit/fleet004.json";

// ---- corpus (identical shape to the t2writer fixture; one canonical law) --

const ACTIONS: [&str; 5] = [
    "container-gate-in",
    "vessel-load",
    "vessel-discharge",
    "container-gate-out",
    "yard-scan",
];
const CHANNELS: [&str; 6] = ["vessel", "terminal", "yard", "rail", "crane", "gate"];

fn marine_event(n: i64) -> (String, &'static str, &'static str, i64) {
    let b = n / BLOCK;
    (
        format!("cntr-{}", b % 5_000),
        ACTIONS[(b % 5) as usize],
        CHANNELS[(b % 6) as usize],
        1_767_225_600 + n,
    )
}

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

fn block_checksum(b: i64) -> i64 {
    let n0 = b * BLOCK;
    3 * (BLOCK * n0 + BLOCK * (BLOCK - 1) / 2) + 7 * BLOCK
}

// ---- helpers ---------------------------------------------------------------

fn temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("fleetd-{label}-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn scalar_i64(conn: &Connection, sql: &str) -> i64 {
    let mut out: i64 = 0;
    conn.query_row(sql, [], |r| {
        out = r.get(0)?;
        Ok(())
    })
    .unwrap_or_else(|e| panic!("scalar_i64 `{sql}`: {e}"));
    out
}

fn scalar_string(conn: &Connection, sql: &str) -> String {
    let mut out = String::new();
    conn.query_row(sql, [], |r| {
        let v: String = r.get(0)?;
        out = v;
        Ok(())
    })
    .unwrap_or_else(|e| panic!("scalar_string `{sql}`: {e}"));
    out
}

/// Build the 1M-event completed marine db (D8 schema + closed-form chksum).
fn build_marine_db(dir: &Path, events: i64) -> PathBuf {
    let db = dir.join("fleet004_marine.db");
    let conn = Connection::open(&db).expect("open marine db");
    conn.execute_batch(
        "CREATE TABLE ocel_conforming_segments(
            segment_id BIGINT,
            oid VARCHAR,
            action VARCHAR,
            channel VARCHAR,
            t_start TIMESTAMP,
            t_end TIMESTAMP,
            transition_count BIGINT,
            chksum BIGINT);",
    )
    .expect("create segments");
    let total_segments = events / BLOCK;
    const CHUNK: i64 = 500;
    let mut b: i64 = 0;
    while b < total_segments {
        let end = (b + CHUNK).min(total_segments);
        let mut sql = String::with_capacity(1 << 17);
        sql.push_str("INSERT INTO ocel_conforming_segments VALUES ");
        for seg in b..end {
            if seg > b {
                sql.push(',');
            }
            let (oid, action, channel, _) = marine_event(seg * BLOCK);
            let t_start = chrono_compat(1_767_225_600 + seg * BLOCK);
            let t_end = chrono_compat(1_767_225_600 + seg * BLOCK + BLOCK - 1);
            sql.push_str(&format!(
                " ({}, '{}', '{}', '{}', '{}', '{}', {}, {})",
                seg + 1,
                oid,
                action,
                channel,
                t_start,
                t_end,
                BLOCK,
                block_checksum(seg)
            ));
        }
        sql.push(';');
        conn.execute_batch(&sql).expect("segment chunk insert");
        b = end;
    }
    conn.execute_batch("CHECKPOINT;").expect("checkpoint");
    drop(conn);
    db
}

fn write_jsonl(dir: &Path, events: i64) -> PathBuf {
    let out = dir.join("fleet004_baseline.jsonl");
    let st = Command::new(T2_BIN)
        .args([
            "jsonl",
            out.to_str().expect("utf8 out"),
            &events.to_string(),
        ])
        .status()
        .expect("spawn t2writer jsonl");
    assert!(st.success(), "jsonl baseline writer exited {st}");
    out
}

fn engine_version(conn: &Connection) -> String {
    scalar_string(conn, "SELECT version()")
}

// ---- legs ------------------------------------------------------------------

/// Leg 1: the two-process kill -9 chaos court (shell driver).
#[test]
#[ignore = "UNSUPPORTED(fixture): requires external t2writer binary at /tmp/t2writer; run with --ignored where built"]
fn fleet004_leg1_chaos_court() {
    if !Path::new(T2_BIN).exists() {
        panic!(
            "[SKIP] no t2writer fixture binary at {T2_BIN} — build: cd /tmp/t2writer && \
             RUSTFLAGS='-C link-arg=-Wl,-rpath,$HOME/.frozen-duckdb/cache/v1.5.5-arm64' \
             cargo build --release"
        );
    }
    fs::create_dir_all("/tmp/value_audit").expect("value_audit dir");
    let out = Command::new("bash")
        .arg(COURT)
        .env("FLEET_D_RECEIPT", "/tmp/value_audit/fleet004_chaos.json")
        .output()
        .expect("spawn chaos court");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    println!("{stdout}{stderr}");
    assert!(
        out.status.success() && stdout.contains("VERDICT PASS"),
        "leg1 chaos court FAILED (exit {:?}):\n{stdout}\n{stderr}",
        out.status.code()
    );
    assert!(
        stdout.contains("NONVACUITY flip witnessed"),
        "leg1: non-vacuity flip not witnessed"
    );
}

/// Leg 2 + 3 share the 1M-event completed db via OnceLock.
#[allow(dead_code)] // dir retained for post-court inspection of the staged db
struct Built {
    dir: PathBuf,
    db: PathBuf,
    raw_bytes: u64,
    db_bytes: u64,
    engine: String,
}

fn built() -> &'static Built {
    static BUILT: std::sync::OnceLock<Built> = std::sync::OnceLock::new();
    BUILT.get_or_init(|| {
        let dir = temp_dir("build");
        let raw = write_jsonl(&dir, EVENTS);
        let db = build_marine_db(&dir, EVENTS);
        let raw_bytes = fs::metadata(&raw).expect("stat jsonl").len();
        let db_bytes = fs::metadata(&db).expect("stat db").len();
        let conn = Connection::open(&db).expect("reopen marine db");
        let engine = engine_version(&conn);
        drop(conn);
        Built {
            dir,
            db,
            raw_bytes,
            db_bytes,
            engine,
        }
    })
}

/// Leg 2: compression — (raw_jsonl - db)/raw >= 0.95 over 1M completed events.
#[test]
#[ignore = "UNSUPPORTED(fixture): requires external t2writer binary at /tmp/t2writer; run with --ignored where built"]
fn fleet004_leg2_compression_1m() {
    let b = built();
    let ratio = (b.raw_bytes - b.db_bytes) as f64 / b.raw_bytes as f64;
    println!(
        "fleet004 leg2: raw={}B db={}B ratio={ratio:.4} engine={}",
        b.raw_bytes, b.db_bytes, b.engine
    );
    assert!(
        ratio >= RATIO_PASS,
        "compression {ratio:.4} < {RATIO_PASS} (raw={} db={})",
        b.raw_bytes,
        b.db_bytes
    );
}

/// Leg 3: satellite sync vector — serialized digest stream + counts <= 500KB.
#[test]
#[ignore = "UNSUPPORTED(fixture): requires external t2writer binary at /tmp/t2writer; run with --ignored where built"]
fn fleet004_leg3_satellite_sync_vector() {
    let b = built();
    let conn = Connection::open(&b.db).expect("open for satellite leg");
    // conservation: segments fold exactly 1M events
    let total = scalar_i64(
        &conn,
        "SELECT sum(transition_count) FROM ocel_conforming_segments;",
    );
    assert_eq!(total, EVENTS, "conservation sum(transition_count)");
    // per-segment digest stream: 32 hex chars per segment, ordered, one blob
    let digest: String = scalar_string(
        &conn,
        "SELECT string_agg(md5(concat_ws('|', segment_id, oid, action, channel, \
         epoch_us(t_start), epoch_us(t_end), transition_count, chksum)), '' \
         ORDER BY segment_id) FROM ocel_conforming_segments;",
    );
    let n_segments = scalar_i64(&conn, "SELECT count(*) FROM ocel_conforming_segments;");
    let stream = format!(
        "FLEET004|v1|events={EVENTS}|segments={n_segments}|engine={}|digest=",
        b.engine
    );
    let total_bytes = stream.len() + digest.len();
    println!(
        "fleet004 leg3: segments={n_segments} digest={}B header={}B total={total_bytes}B budget={SATELLITE_BUDGET}B",
        digest.len(),
        stream.len()
    );
    assert_eq!(
        digest.len() as i64,
        n_segments * 32,
        "digest stream is exactly 32 hex chars per segment"
    );
    assert!(
        total_bytes <= SATELLITE_BUDGET,
        "satellite sync vector {total_bytes}B > {SATELLITE_BUDGET}B"
    );

    // receipt: real measured values only, written from this run
    let receipt = format!(
        "{{\n  \"receipt\": \"FLEET-004 (native)\",\n  \"lane\": \"T2\",\n  \
         \"gate\": \"TEST-FLEET-004\",\n  \"engine\": \"{}\",\n  \
         \"events\": {EVENTS},\n  \"segments\": {n_segments},\n  \
         \"leg2_compression\": {{\"raw_jsonl_bytes\": {}, \"db_bytes\": {}, \
         \"ratio\": {:.4}, \"pass_line\": {RATIO_PASS}}},\n  \
         \"leg3_satellite\": {{\"digest_bytes\": {}, \"header_bytes\": {}, \
         \"total_bytes\": {}, \"budget\": {}}},\n  \
         \"leg1_chaos\": \"see /tmp/value_audit/fleet004_chaos.json (court receipt)\",\n  \
         \"db_sha256\": \"{}\",\n  \"standing\": \"ALIVE\"\n}}\n",
        b.engine,
        b.raw_bytes,
        b.db_bytes,
        (b.raw_bytes - b.db_bytes) as f64 / b.raw_bytes as f64,
        digest.len(),
        stream.len(),
        total_bytes,
        SATELLITE_BUDGET,
        sha256_file(&b.db),
    );
    fs::create_dir_all("/tmp/value_audit").expect("value_audit dir");
    fs::write(RECEIPT, receipt).expect("write fleet004 receipt");
    println!("fleet004 receipt: {RECEIPT}");
}

fn sha256_file(p: &Path) -> String {
    let out = Command::new("shasum")
        .args(["-a", "256"])
        .arg(p)
        .output()
        .expect("run shasum");
    String::from_utf8_lossy(&out.stdout)
        .split_whitespace()
        .next()
        .unwrap_or("shasum-err")
        .to_string()
}
