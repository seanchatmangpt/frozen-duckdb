//! query_receipt — lane MAX4 Last-Mile detached receipt demo.
//!
//! Runs a real 5-query session against frozen-duckdb, mints one BLAKE3
//! receipt per query over (sql_text || input_hash || output_row_count ||
//! output_first_vector_bytes), writes a session manifest (AD4 mint-then-
//! verify), then proves the verify anchor non-vacuous: an edited receipt
//! FAILS verification, a re-mint is GREEN.
//!
//! Run: cargo run -p frozen-duckdb --example query_receipt
//! Hash: BLAKE3 via the `b3sum` CLI (no blake3 crate in the workspace; recorded).

use anyhow::{anyhow, Context, Result};
use frozen_duckdb::duckdb::types::ValueRef;
use frozen_duckdb::{params, Connection, ToSql};
use serde_json::json;
use std::io::Write;
use std::path::Path;
use std::process::Command;

fn b3_bytes(data: &[u8]) -> Result<String> {
    let mut child = Command::new("b3sum")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .context("b3sum CLI not available")?;
    child.stdin.take().unwrap().write_all(data)?;
    let out = child.wait_with_output()?;
    if !out.status.success() {
        return Err(anyhow!("b3sum exited nonzero"));
    }
    let line = String::from_utf8(out.stdout)?.trim().to_string();
    Ok(line.split_whitespace().next().unwrap_or("").to_string())
}

fn b3_file(p: &Path) -> Result<String> {
    let out = Command::new("b3sum").arg(p).output()?;
    if !out.status.success() {
        return Err(anyhow!("b3sum {} failed", p.display()));
    }
    let line = String::from_utf8(out.stdout)?.trim().to_string();
    Ok(line.split_whitespace().next().unwrap_or("").to_string())
}

fn frame(parts: &[&[u8]]) -> Vec<u8> {
    let mut v = Vec::new();
    for p in parts {
        v.extend_from_slice(&(p.len() as u64).to_le_bytes());
        v.extend_from_slice(p);
    }
    v
}

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

struct Receipt {
    index: usize,
    sql: String,
    input_hash: String,
    row_count: usize,
    first_vector: String,
    digest: String,
}

impl Receipt {
    fn to_json(&self) -> serde_json::Value {
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
    fn write(&self, dir: &Path) -> Result<std::path::PathBuf> {
        let p = dir.join(format!("qr{:03}.json", self.index));
        std::fs::write(&p, serde_json::to_string_pretty(&self.to_json())?)?;
        Ok(p)
    }
}

fn execute_and_mint(
    conn: &Connection,
    index: usize,
    sql: &str,
    params: &[&dyn ToSql],
) -> Result<Receipt> {
    let mut stmt = conn.prepare(sql)?;
    let mut rows = stmt.query(params)?;
    let mut row_count = 0usize;
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
    let input_render = params
        .iter()
        .map(|p| Ok(format!("{:?}", p.to_sql()?)))
        .collect::<frozen_duckdb::Result<Vec<_>>>()?
        .join("\u{1f}");
    let input_hash = b3_bytes(input_render.as_bytes())?;
    let rc = row_count.to_string();
    let digest = b3_bytes(&frame(&[
        sql.as_bytes(),
        input_hash.as_bytes(),
        rc.as_bytes(),
        first_vector.as_bytes(),
    ]))?;
    Ok(Receipt {
        index,
        sql: sql.to_string(),
        input_hash,
        row_count,
        first_vector,
        digest,
    })
}

fn mint_manifest(dir: &Path) -> Result<std::path::PathBuf> {
    let manifest = dir.join("MANIFEST.b3");
    let mut names: Vec<String> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with("qr") && n.ends_with(".json"))
        .collect();
    names.sort();
    if names.is_empty() {
        return Err(anyhow!("mint: refusing empty manifest"));
    }
    let lines: Result<Vec<String>> = names
        .iter()
        .map(|n| Ok(format!("{}  {n}", b3_file(&dir.join(n))?)))
        .collect();
    std::fs::write(&manifest, lines?.join("\n") + "\n")?;
    Ok(manifest)
}

fn verify_manifest(dir: &Path, manifest: &Path) -> Result<()> {
    let txt = std::fs::read_to_string(manifest)?;
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
            return Err(anyhow!("malformed manifest line {t:?}"));
        }
        entries.push((digest, name));
    }
    if entries.is_empty() {
        return Err(anyhow!("manifest has no entries"));
    }
    let manifested: std::collections::BTreeSet<String> =
        entries.iter().map(|(_, n)| n.clone()).collect();
    for (digest, name) in &entries {
        let actual = b3_file(&dir.join(name))
            .map_err(|_| anyhow!("TAMPER — receipt {name} missing or unreadable"))?;
        if &actual != digest {
            return Err(anyhow!(
                "TAMPER — receipt {name} b3 mismatch (manifest {digest} vs actual {actual})"
            ));
        }
    }
    let extra: Vec<String> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with("qr") && n.ends_with(".json") && !manifested.contains(n))
        .collect();
    if !extra.is_empty() {
        return Err(anyhow!(
            "TAMPER — unmanifested receipt(s): {}",
            extra.join(", ")
        ));
    }
    Ok(())
}

fn main() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let rdir = dir.path().join("receipts");
    std::fs::create_dir_all(&rdir)?;

    let conn = Connection::open_in_memory()?;
    conn.execute_batch(
        "CREATE TABLE receipt_items (id INTEGER, name TEXT, amount DECIMAL(18,2));\n
         INSERT INTO receipt_items VALUES (1,'alpha',10.50),(2,'beta',25.00),(3,'gamma',7.25),(4,'delta',99.99);",
    )?;

    let p_int = 2i32;
    let p_dec = "20.00";
    let runs: Vec<(&str, Vec<&dyn ToSql>)> = vec![
        ("SELECT version() AS v", vec![]),
        (
            "SELECT id, name, amount::VARCHAR AS amount FROM receipt_items WHERE id = ?",
            vec![&p_int],
        ),
        ("SELECT count(*)::VARCHAR AS n FROM receipt_items", vec![]),
        (
            "SELECT name FROM receipt_items WHERE amount > ? ORDER BY name",
            vec![&p_dec],
        ),
        (
            "SELECT sum(amount)::VARCHAR AS total FROM receipt_items",
            vec![],
        ),
    ];

    let mut receipts = Vec::new();
    for (i, (sql, ps)) in runs.into_iter().enumerate() {
        let r = execute_and_mint(&conn, i, sql, &ps)?;
        let path = r.write(&rdir)?;
        println!("RECEIPT {} -> {}", r.to_json(), path.display());
        receipts.push(r);
    }
    assert_eq!(receipts.len(), 5, "5 queries -> 5 receipts");

    let manifest = mint_manifest(&rdir)?;
    println!("MANIFEST minted: {}", manifest.display());
    print!("{}", std::fs::read_to_string(&manifest)?);
    verify_manifest(&rdir, &manifest).expect("fresh mint GREEN");
    println!("VERIFY GREEN (fresh mint)");

    // Tamper court: edit a receipt -> verify FAILS.
    let victim = rdir.join("qr002.json");
    let original = std::fs::read_to_string(&victim)?;
    std::fs::write(&victim, original.replace("4", "400"))?;
    match verify_manifest(&rdir, &manifest) {
        Err(e) => println!("TAMPER-COURT RED (as required): {e}"),
        Ok(()) => panic!("VACUOUS GATE: tampered receipt still verified"),
    }

    // Re-mint: green again.
    std::fs::write(&victim, &original)?;
    let manifest2 = mint_manifest(&rdir)?;
    verify_manifest(&rdir, &manifest2)?;
    println!("RE-MINT GREEN: {}", manifest2.display());

    // params! macro exercised so the drop-in surface stays imported/used.
    let _ = params![];
    Ok(())
}
