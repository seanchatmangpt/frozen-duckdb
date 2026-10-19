//! TEST-VAL-001: 500k-event semi-structured storage audit (lane V2).
//!
//! Real engine (v2.0.0-dev cached dylib), no mocks. Measures:
//! - VARIANT columnar storage compression vs raw JSONL on disk
//! - selective filter latency (`json_extract_string` on the payload path)
//!
//! Run: CARGO_TARGET_DIR=target-laneV2 cargo test -p frozen-duckdb \
//!        --test value_audit_datadog -- --nocapture

use std::fs::File;
use std::io::{BufWriter, Write};
use std::time::Instant;

use frozen_duckdb::{params, Connection};

const EVENTS: usize = 500_000;
const DB_PATH: &str = "/tmp/value_audit/val001.duckdb";
const JSONL_PATH: &str = "/tmp/value_audit/events.jsonl";
const RECEIPT_PATH: &str = "/tmp/value_audit/val001.json";

/// The exact SQL filter that works on this engine (recorded per lane task).
const FILTER_SQL: &str =
    "SELECT count(*) FROM logs WHERE json_extract_string(payload::VARCHAR, '$.response.code') = '500'";

struct EngineIdentity {
    dylib: String,
}

fn engine_identity() -> EngineIdentity {
    // The test binary is what links the cached duckdb dylib; otool -L names it.
    let exe = std::env::current_exe().expect("current_exe");
    let out = std::process::Command::new("otool")
        .arg("-L")
        .arg(&exe)
        .output()
        .expect("otool");
    let text = String::from_utf8_lossy(&out.stdout);
    let dylib = text
        .lines()
        .map(str::trim)
        .find(|l| l.contains("duckdb") && l.contains(".dylib"))
        .map(str::to_string)
        .unwrap_or_else(|| format!("UNKNOWN ({})", text.lines().count()));
    EngineIdentity { dylib }
}

#[test]
fn value_audit_datadog_500k_variant_compression() {
    std::fs::create_dir_all("/tmp/value_audit").unwrap();
    for p in [DB_PATH, JSONL_PATH] {
        let _ = std::fs::remove_file(p);
    }

    let ident = engine_identity();
    println!("engine dylib: {}", ident.dylib);

    let conn = Connection::open(DB_PATH).expect("open db");
    conn.execute("CREATE TABLE logs(ts BIGINT, payload VARIANT)", [])
        .expect("VARIANT DDL accepted on this engine");

    // Generate, write raw JSONL, and insert through the VARIANT pipeline.
    let t_ins = Instant::now();
    let mut raw = BufWriter::new(File::create(JSONL_PATH).unwrap());
    let mut stmt = conn.prepare("INSERT INTO logs VALUES (?, ?)").unwrap();
    let mut buf = String::with_capacity(256);
    for i in 0..EVENTS {
        let code = if i % 50 == 0 { 500 } else { 200 };
        buf.clear();
        let service = ["api", "web", "worker", "db"][i % 4];
        let env = ["prod", "staging"][i % 2];
        // ~150 byte payload, datadog-log shaped.
        buf.push_str(&format!(
            r#"{{"service":"{service}","env":"{env}","host":"host-{}","message":"request completed","response":{{"code":{code},"duration_ms":{}}}}}"#,
            i % 64,
            (i * 7) % 900,
        ));
        raw.write_all(buf.as_bytes()).unwrap();
        raw.write_all(b"\n").unwrap();
        stmt.execute(params![i as i64, buf.as_str()])
            .expect("varchar bind -> VARIANT insert");
    }
    drop(raw);
    conn.execute("CHECKPOINT", []).expect("checkpoint");
    let insert_s = t_ins.elapsed().as_secs_f64();

    let bytes_raw = std::fs::metadata(JSONL_PATH).unwrap().len();
    let bytes_db = std::fs::metadata(DB_PATH).unwrap().len();
    let compression = 1.0 - (bytes_db as f64 / bytes_raw as f64);

    // Selective filter timing.
    let t_f = Instant::now();
    let hits: i64 = conn.query_row(FILTER_SQL, [], |r| r.get(0)).expect("filter");
    let filter_ms = t_f.elapsed().as_secs_f64() * 1000.0;
    assert_eq!(hits as usize, EVENTS / 50, "filter must select the 500s");

    // Receipt (single JSON line).
    let verdict = if compression >= 0.90 && ident.dylib.contains("duckdb") {
        "PASS"
    } else {
        "FAIL"
    };
    let line = format!(
        "{{\"id\":\"VALUE-VAL-001\",\"events\":{},\"bytes_raw\":{},\"bytes_db\":{},\"compression\":{:.4},\"filter_latency_ms\":{:.1},\"filter_sql\":\"{}\",\"insert_seconds\":{:.1},\"engine_dylib\":\"{}\",\"verdict\":\"{}\"}}",
        EVENTS, bytes_raw, bytes_db, compression, filter_ms, FILTER_SQL, insert_s, ident.dylib, verdict
    );
    println!("{}", line);
    std::fs::write(RECEIPT_PATH, line + "\n").unwrap();

    assert!(compression >= 0.90, "compression {compression:.4} < 0.90");
    assert_eq!(verdict, "PASS");
}
