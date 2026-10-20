//! query_receipt.rs — lane MAX4, Last-Mile detached receipt court.
//!
//! The maximization spec: for each executed query, a BLAKE3 receipt over
//! (sql_text || input_hash || output_row_count || output_first_vector_bytes),
//! a JSON receipt line per query, a session manifest signing all receipts
//! (the AD4 mint-then-verify pattern), and a tamper court: edit a receipt ->
//! verify FAILS; re-mint -> green.
//!
//! Reference pattern: warehouse-sim/economics/render_value_audit.ts
//! (--verify integrity anchor: every manifested receipt's b3 recomputed at
//! verify time; any mismatch, missing receipt, or unmanifested extra fails).
//!
//! Hash: BLAKE3 via the `b3sum` CLI (present on this workstation; the blake3
//! crate is not in the workspace deps and lane scope forbids touching
//! Cargo.toml — recorded here per spec: HASH=b3sum CLI, no in-tree crate).
//! Chicago discipline: b3sum is a real collaborator, not a mock; every digest
//! is recomputed from bytes on disk.

use anyhow::{anyhow, Context, Result};
use frozen_duckdb::duckdb::types::ValueRef;
use frozen_duckdb::Connection;
use serde_json::{json, Value as Json};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

// ---------------------------------------------------------------- b3sum CLI

/// BLAKE3 of raw bytes via the b3sum CLI reading stdin (real subprocess).
fn b3_bytes(data: &[u8]) -> Result<String> {
    let mut child = Command::new("b3sum")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .context("b3sum CLI not available — receipt court requires it")?;
    child
        .stdin
        .take()
        .unwrap()
        .write_all(data)
        .context("write to b3sum stdin")?;
    let out = child.wait_with_output()?;
    if !out.status.success() {
        return Err(anyhow!("b3sum exited nonzero: {:?}", out.status));
    }
    let line = String::from_utf8(out.stdout)?.trim().to_string();
    let digest = line.split_whitespace().next().unwrap_or("").to_string();
    if digest.len() != 64 || !digest.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(anyhow!("b3sum emitted malformed digest: {line:?}"));
    }
    Ok(digest)
}

/// BLAKE3 of a file on disk (first whitespace-separated field of b3sum output).
fn b3_file(p: &Path) -> Result<String> {
    let out = Command::new("b3sum")
        .arg(p)
        .output()
        .with_context(|| format!("b3sum {}", p.display()))?;
    if !out.status.success() {
        return Err(anyhow!("b3sum {} exited nonzero", p.display()));
    }
    let line = String::from_utf8(out.stdout)?.trim().to_string();
    Ok(line.split_whitespace().next().unwrap_or("").to_string())
}

// ------------------------------------------------------- receipt primitives

/// Length-prefixed frame so field boundaries are unambiguous in the digest
/// preimage (no separator-collision channel).
fn frame(parts: &[&[u8]]) -> Vec<u8> {
    let mut v = Vec::new();
    for p in parts {
        v.extend_from_slice(&(p.len() as u64).to_le_bytes());
        v.extend_from_slice(p);
    }
    v
}

/// The canonical digest preimage: sql_text || input_hash || row_count ||
/// first_vector_bytes — each length-prefixed.
fn receipt_digest(
    sql: &str,
    input_hash: &str,
    row_count: usize,
    first_vector_bytes: &[u8],
) -> Result<String> {
    let rc = row_count.to_string();
    b3_bytes(&frame(&[
        sql.as_bytes(),
        input_hash.as_bytes(),
        rc.as_bytes(),
        first_vector_bytes,
    ]))
}

/// Serialize the first row ("output vector head") deterministically from
/// real ValueRefs off the DuckDB cursor. Scalars rendered canonically;
/// complex types rendered via their Debug shape (stable within a run).
fn value_ref_to_string(v: ValueRef<'_>) -> String {
    match v {
        ValueRef::Null => "NULL".into(),
        ValueRef::Boolean(b) => format!("{b}"),
        ValueRef::TinyInt(x) => format!("{x}"),
        ValueRef::SmallInt(x) => format!("{x}"),
        ValueRef::Int(x) => format!("{x}"),
        ValueRef::BigInt(x) => format!("{x}"),
        ValueRef::HugeInt(x) => format!("{x}"),
        ValueRef::UTinyInt(x) => format!("{x}"),
        ValueRef::USmallInt(x) => format!("{x}"),
        ValueRef::UInt(x) => format!("{x}"),
        ValueRef::UBigInt(x) => format!("{x}"),
        ValueRef::Float(x) => format!("{x:?}"),
        ValueRef::Double(x) => format!("{x:?}"),
        ValueRef::Decimal(d) => format!("{d:?}"),
        ValueRef::Text(t) => String::from_utf8_lossy(t).into_owned(),
        ValueRef::Blob(b) => format!("blob:{}", b3_bytes(b).unwrap_or_else(|_| "?".into())),
        ValueRef::Timestamp(_, us) => format!("ts:{us}"),
        ValueRef::Date32(d) => format!("date:{d}"),
        ValueRef::Time64(_, v) => format!("time:{v}"),
        other => format!("complex:{other:?}"),
    }
}

struct QueryReceipt {
    index: usize,
    sql: String,
    input_hash: String,
    row_count: usize,
    first_vector: String,
    digest: String,
}

impl QueryReceipt {
    fn to_json(&self) -> Json {
        json!({
            "schema": "fdv.query-receipt/v1",
            "index": self.index,
            "sql": self.sql,
            "input_hash": self.input_hash,
            "output_row_count": self.row_count,
            "output_first_vector": self.first_vector,
            "digest": self.digest,
        })
    }

    fn write(&self, dir: &Path) -> Result<PathBuf> {
        let p = dir.join(format!("qr{:03}.json", self.index));
        std::fs::write(&p, serde_json::to_string_pretty(&self.to_json())?)?;
        Ok(p)
    }
}

/// Execute one real query with params, count rows, capture the first row
/// vector, and mint the receipt digest over the canonical preimage.
fn execute_and_mint(
    conn: &Connection,
    index: usize,
    sql: &str,
    params: &[&dyn frozen_duckdb::ToSql],
) -> Result<QueryReceipt> {
    let mut stmt = conn.prepare(sql)?;
    let mut rows = stmt.query(params)?;
    let mut row_count: usize = 0;
    let mut first_vector = String::new();
    while let Some(row) = rows.next()? {
        if row_count == 0 {
            // NOTE: column_count panics before execution in this vendored
            // layer, so it is read from the first executed row, not the
            // prepared statement.
            let ncols = row.as_ref().column_count();
            let cells: Vec<String> = (0..ncols)
                .map(|i| Ok(value_ref_to_string(row.get_ref(i)?)))
                .collect::<frozen_duckdb::Result<Vec<_>>>()?;
            first_vector = format!("[{}]", cells.join("\u{1f}"));
        }
        row_count += 1;
    }
    // input_hash: b3 over the caller-supplied canonical rendering of the
    // bound inputs (real input identity; `dyn ToSql` carries no Debug).
    let input_render = params
        .iter()
        .map(|p| Ok(format!("{:?}", p.to_sql()?)))
        .collect::<frozen_duckdb::Result<Vec<_>>>()?
        .join("\u{1f}");
    let input_hash = b3_bytes(input_render.as_bytes())?;
    let digest = receipt_digest(sql, &input_hash, row_count, first_vector.as_bytes())?;
    Ok(QueryReceipt {
        index,
        sql: sql.to_string(),
        input_hash,
        row_count,
        first_vector,
        digest,
    })
}

// ------------------------------------------------------- manifest (AD4 mint)

fn mint_manifest(dir: &Path) -> Result<PathBuf> {
    let manifest = dir.join("MANIFEST.b3");
    let mut entries: Vec<String> = Vec::new();
    let mut names: Vec<String> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with("qr") && n.ends_with(".json"))
        .collect();
    names.sort();
    if names.is_empty() {
        return Err(anyhow!(
            "mint: no receipts in {} — refusing empty manifest",
            dir.display()
        ));
    }
    for n in &names {
        entries.push(format!("{}  {n}", b3_file(&dir.join(n))?));
    }
    std::fs::write(&manifest, entries.join("\n") + "\n")?;
    Ok(manifest)
}

/// The verify anchor: recompute every manifested receipt's b3 from disk;
/// fail on mismatch, missing receipt, or unmanifested extra. Returns
/// Err (not Ok) on any tamper — the court asserts on this.
fn verify_manifest(dir: &Path, manifest: &Path) -> Result<()> {
    let txt = std::fs::read_to_string(manifest)
        .with_context(|| format!("verify: manifest {} unreadable", manifest.display()))?;
    let mut entries: Vec<(String, String)> = Vec::new();
    for line in txt.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        let mut it = t.splitn(2, char::is_whitespace);
        let digest = it.next().unwrap_or("").to_string();
        let name = it
            .next()
            .unwrap_or("")
            .trim()
            .trim_start_matches('*')
            .to_string();
        if digest.len() != 64 || name.is_empty() {
            return Err(anyhow!("verify: malformed manifest line {t:?}"));
        }
        entries.push((digest, name));
    }
    if entries.is_empty() {
        return Err(anyhow!("verify: manifest has no entries"));
    }
    let manifested: std::collections::BTreeSet<String> =
        entries.iter().map(|(_, n)| n.clone()).collect();
    // 1. every manifested receipt exists and matches
    for (digest, name) in &entries {
        let p = dir.join(name);
        let actual = b3_file(&p).map_err(|_| {
            anyhow!(
                "TAMPER — receipt {name} missing or unreadable (manifest {})",
                manifest.display()
            )
        })?;
        if &actual != digest {
            return Err(anyhow!(
                "TAMPER — receipt {name} b3 mismatch (manifest {}… vs actual {actual}…)",
                &digest[..12.min(digest.len())]
            ));
        }
    }
    // 2. no unmanifested receipt in the dir
    let extra: Vec<String> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| {
            let n = n.as_str();
            (n.starts_with("qr") && n.ends_with(".json")) && !manifested.contains(n)
        })
        .collect();
    if !extra.is_empty() {
        return Err(anyhow!(
            "TAMPER — unmanifested receipt(s): {}",
            extra.join(", ")
        ));
    }
    Ok(())
}

// ----------------------------------------------------------------- session

const QUERIES: &[&str] = &[
    "SELECT version() AS v",
    "SELECT id, name, amount::VARCHAR AS amount FROM receipt_items WHERE id = ?",
    "SELECT count(*)::VARCHAR AS n FROM receipt_items",
    "SELECT name FROM receipt_items WHERE amount > ? ORDER BY name",
    "SELECT sum(amount)::VARCHAR AS total FROM receipt_items",
];

fn run_session(conn: &Connection, dir: &Path) -> Result<Vec<QueryReceipt>> {
    conn.execute_batch(
        "CREATE TABLE receipt_items (id INTEGER, name TEXT, amount DECIMAL(18,2));\n
         INSERT INTO receipt_items VALUES (1,'alpha',10.50),(2,'beta',25.00),(3,'gamma',7.25),(4,'delta',99.99);",
    )?;
    let p_int = 2i32;
    let p_dec = "20.00";
    let runs: Vec<(&str, Vec<&dyn frozen_duckdb::ToSql>)> = vec![
        (QUERIES[0], vec![]),
        (QUERIES[1], vec![&p_int]),
        (QUERIES[2], vec![]),
        (QUERIES[3], vec![&p_dec]),
        (QUERIES[4], vec![]),
    ];
    let mut receipts = Vec::new();
    for (i, (sql, ps)) in runs.into_iter().enumerate() {
        let ps_ref: Vec<&dyn frozen_duckdb::ToSql> = ps;
        let r = execute_and_mint(conn, i, sql, &ps_ref)?;
        let path = r.write(dir)?;
        println!("RECEIPT {} -> {}", r.to_json(), path.display());
        receipts.push(r);
    }
    Ok(receipts)
}

// ------------------------------------------------------------------- court

#[test]
fn query_receipt_mint_verify_tamper_remit_court() -> Result<()> {
    // Session over a real DuckDB connection (in-memory engine).
    let conn = Connection::open_in_memory()?;
    let dir = tempfile::tempdir()?;
    let rdir = dir.path().join("receipts");
    std::fs::create_dir_all(&rdir)?;

    // 5 queries -> 5 receipts (real executions, real outputs).
    let receipts = run_session(&conn, &rdir)?;
    assert_eq!(receipts.len(), 5, "expected 5 receipted queries");
    assert_eq!(
        std::fs::read_dir(&rdir)?.count(),
        5,
        "5 receipt files on disk"
    );
    assert_eq!(receipts[1].row_count, 1, "point lookup returns one row");
    assert_eq!(
        receipts[2].first_vector, "[4]",
        "count(*) over 4 seeded rows (1 result row carrying the count)"
    );
    assert_eq!(receipts[3].row_count, 2, "amount > 20.00 -> beta+delta");
    assert_eq!(
        receipts[4].first_vector, "[142.74]",
        "sum matches seed data"
    );

    // Digest is a function of the whole preimage: same sql, different input
    // hash -> different digest (input_hash channel is load-bearing).
    let r_a = execute_and_mint(&conn, 9, QUERIES[1], &[&2i32])?;
    let r_b = execute_and_mint(&conn, 9, QUERIES[1], &[&3i32])?;
    assert_ne!(r_a.digest, r_b.digest, "input_hash must change the digest");

    // Mint the session manifest (AD4: mint-then-verify).
    let manifest = mint_manifest(&rdir)?;
    println!("MANIFEST {}", manifest.display());
    print!("{}", std::fs::read_to_string(&manifest)?);
    verify_manifest(&rdir, &manifest).expect("fresh mint must verify GREEN");

    // TAMPER COURT: edit a receipt on disk (real tamper, real bytes).
    let victim = rdir.join("qr002.json");
    let original = std::fs::read_to_string(&victim)?;
    let tampered = original.replace("4", "400");
    assert_ne!(original, tampered, "tamper must change the bytes");
    std::fs::write(&victim, &tampered)?;

    let verdict = verify_manifest(&rdir, &manifest);
    assert!(verdict.is_err(), "tampered receipt must FAIL verification");
    let msg = format!("{}", verdict.unwrap_err());
    assert!(msg.contains("TAMPER"), "typed tamper standing, got: {msg}");
    println!("TAMPER-COURT RED: {msg}");

    // RESTORE + re-mint: verify green again on the same engine/session.
    std::fs::write(&victim, &original)?;
    verify_manifest(&rdir, &manifest).expect("restored receipt must verify GREEN");
    let manifest2 = mint_manifest(&rdir)?;
    verify_manifest(&rdir, &manifest2).expect("re-mint must verify GREEN");
    println!("RE-MINT GREEN: {}", manifest2.display());

    Ok(())
}
