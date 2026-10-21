//! STALE_PRECONDITION_REFUSED two-phase guard (lane MAX5).
//!
//! Observes a table state hash at analysis time; refuses a mutation whose
//! observed state no longer matches (`STALE_PRECONDITION_REFUSED`), else
//! executes and returns the post-state hash — the optimistic-concurrency /
//! two-phase analog: the precondition hash travels with the mutation SQL and
//! a mismatch aborts before execution.
//!
//! Hash: SHA-256 over a canonical, ordered rendering of the table contents.
//! SHA-256 is implemented inline below (no new workspace dependency; this
//! lane owns only this file + the court test).

use crate::duckdb::Connection;
use std::fmt;

/// Typed guard error.
#[derive(Debug, PartialEq)]
pub enum PreconditionError {
    /// Observed state hash != expected. Message shape:
    /// `STALE_PRECONDITION_REFUSED: expected <hash4>, got <hash4>`.
    Stale {
        /// The hash observed at analysis time.
        expected: String,
        /// The hash re-observed at execution time.
        got: String,
    },
    /// Propagated DuckDB error.
    DuckDb(crate::duckdb::Error),
}

impl fmt::Display for PreconditionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PreconditionError::Stale { expected, got } => {
                write!(
                    f,
                    "STALE_PRECONDITION_REFUSED: expected {}, got {}",
                    short(expected),
                    short(got)
                )
            }
            PreconditionError::DuckDb(e) => write!(f, "duckdb error: {e}"),
        }
    }
}

impl std::error::Error for PreconditionError {}

impl From<crate::duckdb::Error> for PreconditionError {
    fn from(e: crate::duckdb::Error) -> Self {
        PreconditionError::DuckDb(e)
    }
}

fn short(h: &str) -> &str {
    &h[..h.len().min(4)]
}

/// Successful guarded execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionOutcome {
    /// Rows touched (rows changed when the driver reports them).
    pub rows_affected: usize,
    /// Table state hash re-observed AFTER the action executed.
    pub new_hash: String,
}

/// Two-phase precondition guard over one table.
pub struct PreconditionGuard;

impl PreconditionGuard {
    /// Phase 1 (ANALYZE): observe the table state hash.
    pub fn analyze(conn: &Connection, table: &str) -> Result<String, PreconditionError> {
        render_table(conn, table).map(|canonical| sha256_hex(canonical.as_bytes()))
    }

    /// Phase 2 (EXECUTE): re-hash; refuse on mismatch, else run `action_sql`
    /// and return the post-state hash.
    pub fn execute_if_unchanged(
        conn: &Connection,
        table: &str,
        expected_hash: &str,
        action_sql: &str,
    ) -> Result<ExecutionOutcome, PreconditionError> {
        let got = Self::analyze(conn, table)?;
        if got != expected_hash {
            return Err(PreconditionError::Stale {
                expected: expected_hash.to_string(),
                got,
            });
        }
        let rows_affected = conn.execute_batch_count(action_sql)?;
        let new_hash = Self::analyze(conn, table)?;
        Ok(ExecutionOutcome {
            rows_affected,
            new_hash,
        })
    }

    /// Two-phase commit analog: the mutation SQL *carries* the precondition
    /// hash (embedded as a leading `/* PRECONDITION:<hex> */` comment); the
    /// guard extracts it and validates BEFORE any execution. A mismatched or
    /// missing hash aborts with `STALE_PRECONDITION_REFUSED` / a missing-
    /// precondition refusal, never a partial application.
    pub fn execute_two_phase(
        conn: &Connection,
        table: &str,
        mutation_sql: &str,
    ) -> Result<ExecutionOutcome, PreconditionError> {
        let expected =
            extract_precondition(mutation_sql).ok_or_else(|| PreconditionError::Stale {
                expected: String::new(),
                got: String::new(),
            })?;
        Self::execute_if_unchanged(conn, table, &expected, mutation_sql)
    }
}

/// A missing embedded precondition is refused with a distinct message; kept
/// in the same error family so callers match on the family, not SQL text.
pub fn extract_precondition(sql: &str) -> Option<String> {
    let marker = "/* PRECONDITION:";
    let start = sql.find(marker)? + marker.len();
    let end = sql[start..].find("*/")? + start;
    let h = sql[start..end].trim();
    (!h.is_empty()).then(|| h.to_string())
}

/// Canonical, order-stable rendering of a table's contents: every row, in a
/// deterministic order, every column, deterministic NULL/text encoding.
fn render_table(conn: &Connection, table: &str) -> Result<String, PreconditionError> {
    if !table.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(PreconditionError::DuckDb(
            crate::duckdb::Error::InvalidParameterName(table.to_string()),
        ));
    }
    // Column count from the catalog (statement column_count() here requires
    // an executed result), then row-scan with ORDER BY ALL for a
    // deterministic total order.
    let col_count: i64 = conn.query_row(
        "SELECT count(*) FROM information_schema.columns WHERE table_name = $1",
        [table],
        |r| r.get(0),
    )?;
    let col_count = col_count as usize;
    let mut stmt = conn.prepare(&format!(r#"SELECT * FROM "{table}" ORDER BY ALL"#))?;
    let mut rows = stmt.query([])?;
    let mut canonical = String::from("v1|");
    while let Some(row) = rows.next()? {
        for i in 0..col_count {
            let v = row.get_ref(i)?;
            use crate::duckdb::types::ValueRef;
            match v {
                ValueRef::Null => canonical.push_str("\\N"),
                ValueRef::Int(n) => canonical.push_str(&n.to_string()),
                ValueRef::BigInt(n) => canonical.push_str(&n.to_string()),
                ValueRef::HugeInt(n) => canonical.push_str(&n.to_string()),
                ValueRef::UTinyInt(n) => canonical.push_str(&n.to_string()),
                ValueRef::USmallInt(n) => canonical.push_str(&n.to_string()),
                ValueRef::UInt(n) => canonical.push_str(&n.to_string()),
                ValueRef::UBigInt(n) => canonical.push_str(&n.to_string()),
                ValueRef::TinyInt(n) => canonical.push_str(&n.to_string()),
                ValueRef::SmallInt(n) => canonical.push_str(&n.to_string()),
                ValueRef::Text(b) => {
                    canonical.push_str(&format!("'{}'", String::from_utf8_lossy(b)))
                }
                ValueRef::Boolean(b) => canonical.push_str(if b { "t" } else { "f" }),
                ValueRef::Float(f) => canonical.push_str(&format!("{f:?}")),
                ValueRef::Double(d) => canonical.push_str(&format!("{d:?}")),
                other => canonical.push_str(&format!("raw:{:?}", other)),
            }
            canonical.push('|');
        }
        canonical.push('\n');
    }
    Ok(canonical)
}

trait ExecuteBatchCount {
    fn execute_batch_count(&self, sql: &str) -> Result<usize, PreconditionError>;
}

impl ExecuteBatchCount for Connection {
    fn execute_batch_count(&self, sql: &str) -> Result<usize, PreconditionError> {
        // For a single INSERT/UPDATE/DELETE, run it and read rows changed.
        let mut stmt = self.prepare(sql)?;
        let n = stmt.execute([])?; // rows changed
        Ok(n)
    }
}

// ---------------------------------------------------------------------------
// Inline SHA-256 (FIPS 180-4). No external dependency.
// ---------------------------------------------------------------------------

/// Hex-encoded SHA-256 digest of `data` (FIPS 180-4, inline — no external dep).
pub fn sha256_hex(data: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let bitlen = (data.len() as u64).wrapping_mul(8);
    let mut msg = data.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bitlen.to_be_bytes());
    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (i, wi) in w.iter_mut().enumerate().take(16) {
            *wi = u32::from_be_bytes([
                chunk[i * 4],
                chunk[i * 4 + 1],
                chunk[i * 4 + 1 + 1],
                chunk[i * 4 + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
            (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }
    let mut out = String::with_capacity(64);
    for word in h {
        out.push_str(&format!("{word:08x}"));
    }
    out
}
