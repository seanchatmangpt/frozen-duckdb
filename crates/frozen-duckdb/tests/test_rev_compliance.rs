//! TEST-REV-001: 10M-row journal trial-balance compliance against the real
//! engine (v2.0.0-dev88595, cached). Physical db file, appender bulk ingest,
//! DECIMAL(18,4) multi-account multi-currency journal, trial balance must
//! clear (variance 0) by construction. Receipt: /tmp/value_audit/rev001.json.

use frozen_duckdb::duckdb::appender_params_from_iter;
use frozen_duckdb::{Connection, ToSql};
use std::io::Write as _;
use std::time::Instant;

const ROWS: usize = 10_000_000;
const BATCH: usize = 100_000;

/// Deterministic splitmix64; debit == credit on every row, so every account's
/// sum(debit)==sum(credit) by construction (variance 0 guaranteed, not hoped).
/// account_id encodes currency: 1xxxx=USD, 2xxxx=EUR, 3xxxx=JPY, 4xxxx=GBP.
fn seeded_amount(i: u64) -> i128 {
    let mut z = i.wrapping_add(0x9E3779B97F4A7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    let r = z ^ (z >> 31);
    1 + (r % 100_000_000) as i128 // DECIMAL(18,4) units: 0.0001 .. 10_000.0000
}

fn account_id(i: u64) -> i64 {
    let prefix = 1 + (i % 4) as i64; // 4 currencies
    prefix * 10_000 + (i % 1_000) as i64
}

#[test]
fn rev001_trial_balance_10m() {
    let dir = std::env::temp_dir().join(format!("rev001-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let db_path = dir.join("rev001.duckdb");
    let _ = std::fs::remove_file(&db_path);

    let audit_dir = std::path::Path::new("/tmp/value_audit");
    std::fs::create_dir_all(audit_dir).unwrap();

    let conn = Connection::open(&db_path).expect("open physical db file");
    let version: String = conn
        .query_row("select version()", [], |r| r.get(0))
        .unwrap();
    eprintln!("engine version: {version}");

    conn.execute_batch(
        "CREATE TABLE journal (
            entry_id BIGINT,
            account_id BIGINT,
            currency TEXT,
            debit DECIMAL(18,4),
            credit DECIMAL(18,4)
        );",
    )
    .unwrap();

    let total_start = Instant::now();

    // Ingest 10M rows via appender in batches (bulk).
    let mut app = conn.appender("journal").expect("appender");
    for batch in 0..(ROWS / BATCH) {
        let rows: Vec<_> = (0..BATCH)
            .map(|k| {
                let i = (batch * BATCH + k) as u64;
                let amt = seeded_amount(i);
                let acct = account_id(i);
                let ccy: &'static str = match acct / 10_000 {
                    1 => "USD",
                    2 => "EUR",
                    3 => "JPY",
                    _ => "GBP",
                };
                let row: Vec<Box<dyn ToSql>> = vec![
                    Box::new(i as i64),
                    Box::new(acct),
                    Box::new(ccy),
                    Box::new(amt),
                    Box::new(amt),
                ];
                appender_params_from_iter(row)
            })
            .collect();
        app.append_rows(rows).expect("append batch");
    }
    drop(app);
    eprintln!("ingest done at {:?}", total_start.elapsed());

    // Operator's trial-balance query.
    let mut stmt = conn
        .prepare(
            "SELECT account_id, sum(debit) AS d, sum(credit) AS c
             FROM journal
             GROUP BY account_id
             HAVING sum(debit) <> sum(credit)",
        )
        .unwrap();
    let mut variance_rows: usize = 0;
    let mut rows = stmt.query([]).unwrap();
    while let Ok(Some(r)) = rows.next() {
        variance_rows += 1;
        let acct: i64 = r.get(0).unwrap();
        eprintln!("variance row: account {acct}");
    }
    drop(rows);
    drop(stmt);

    let wall = total_start.elapsed().as_secs_f64();

    // Reconciled balance table dump (ORDER BY account_id) -> digest input.
    let dump_path = audit_dir.join("rev001_balances.csv");
    {
        let mut stmt = conn
            .prepare(
                "SELECT account_id, sum(debit)::VARCHAR, sum(credit)::VARCHAR, currency
                 FROM journal GROUP BY account_id, currency ORDER BY account_id",
            )
            .unwrap();
        let mut rows = stmt.query([]).unwrap();
        let mut f = std::fs::File::create(&dump_path).unwrap();
        while let Ok(Some(r)) = rows.next() {
            let acct: i64 = r.get(0).unwrap();
            let d: String = r.get::<_, String>(1).unwrap();
            let c: String = r.get::<_, String>(2).unwrap();
            let ccy: String = r.get(3).unwrap();
            writeln!(f, "{acct},{d},{c},{ccy}").unwrap();
        }
    }

    // Digest: b3sum (BLAKE3) primary; sha256 fallback. State which was used.
    let out = std::process::Command::new("b3sum").arg(&dump_path).output();
    let (digest, hash_tool) = match out {
        Ok(o) if o.status.success() => (
            String::from_utf8_lossy(&o.stdout)
                .split_whitespace()
                .next()
                .unwrap()
                .to_string(),
            "blake3(b3sum)",
        ),
        _ => {
            let o = std::process::Command::new("shasum")
                .args(["-a", "256"])
                .arg(&dump_path)
                .output()
                .unwrap();
            (
                String::from_utf8_lossy(&o.stdout)
                    .split_whitespace()
                    .next()
                    .unwrap()
                    .to_string(),
                "sha256(shasum -a 256)",
            )
        }
    };

    // Egress/credits: total debits == total credits, nothing leaves the system.
    let (td, tc, n): (String, String, i64) = conn
        .query_row(
            "SELECT sum(debit)::VARCHAR, sum(credit)::VARCHAR, count(*) FROM journal",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();

    // Hard asserts (non-vacuous): row count, variance, balance, egress.
    assert_eq!(n, ROWS as i64, "row count");
    assert_eq!(variance_rows, 0, "trial balance must clear");
    assert_eq!(td, tc, "global debits == credits (egress 0)");

    let time_ok = wall < 2.5;
    let status = if time_ok { "MET" } else { "UNMET-BY-MEASURED" };

    let receipt = format!(
        "{{\n  \"test\": \"TEST-REV-001\",\n  \"engine\": \"{version}\",\n  \"rows\": {n},\n  \"variance\": {variance_rows},\n  \"debits_total\": \"{td}\",\n  \"credits_total\": \"{tc}\",\n  \"egress\": 0,\n  \"wall_s\": {wall:.3},\n  \"clause_lt_2_5s\": \"{status}\",\n  \"hash_tool\": \"{hash_tool}\",\n  \"digest\": \"{digest}\",\n  \"digest_input\": \"{}\",\n  \"rev_line\": \"REV-001 rows={n} variance={variance_rows} wall_s={wall:.1} digest={digest}\"\n}}\n",
        dump_path.display()
    );
    std::fs::write(audit_dir.join("rev001.json"), receipt).unwrap();

    eprintln!(
        "REV-001 rows={n} variance={variance_rows} wall_s={wall:.1} digest={digest} ({hash_tool}) clause_lt_2_5s={status}"
    );

    let _ = std::fs::remove_file(&db_path);
    let _ = std::fs::remove_dir_all(&dir);
}
