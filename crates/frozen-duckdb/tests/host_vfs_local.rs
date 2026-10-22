//! KD-06 local slice acceptance: `read_local_range` registers, queries a
//! real file, honors offset/length contract semantics (exact bytes,
//! EOF -> empty), and serves known content through SQL.

use frozen_duckdb::duckdb::host_vfs;
use frozen_duckdb::Connection;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

/// Fixture: a real temp file with known content, deleted on drop.
struct TempFile {
    path: PathBuf,
}

impl TempFile {
    fn with_bytes(bytes: &[u8]) -> TempFile {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "hostvfs-cl13-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut f = fs::File::create(&path).unwrap();
        f.write_all(bytes).unwrap();
        f.sync_all().unwrap();
        TempFile { path }
    }

    fn path(&self) -> &str {
        self.path.to_str().unwrap()
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn open_conn() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    host_vfs::register(&conn).unwrap();
    conn
}

/// Concatenate all data rows into one byte vector.
fn read_all(conn: &Connection, sql: &str, path: &str) -> Vec<u8> {
    let mut stmt = conn.prepare(sql).unwrap();
    let mut rows = stmt.query([path]).unwrap();
    let mut out = Vec::new();
    while let Some(row) = rows.next().unwrap() {
        let bytes: Vec<u8> = row.get(0).unwrap();
        out.extend_from_slice(&bytes);
    }
    out
}

/// Acceptance 1: function registers and queries a real file.
#[test]
fn registers_and_queries_real_file() {
    let tf = TempFile::with_bytes(b"0123456789");
    let conn = open_conn();
    let data = read_all(
        &conn,
        "select data from read_local_range(?, 0, 10)",
        tf.path(),
    );
    assert_eq!(data, b"0123456789");
}

/// Acceptance 2a: exact bytes — middle range.
#[test]
fn middle_range_exact_bytes() {
    let tf = TempFile::with_bytes(b"0123456789");
    let conn = open_conn();
    let data = read_all(
        &conn,
        "select data from read_local_range(?, 3, 4)",
        tf.path(),
    );
    assert_eq!(data, b"3456");
}

/// Acceptance 2b: EOF -> empty (offset past EOF yields zero rows).
#[test]
fn offset_past_eof_is_empty() {
    let tf = TempFile::with_bytes(b"0123456789");
    let conn = open_conn();
    let n: i64 = conn
        .query_row(
            "select count(*) from read_local_range(?, 10, 5)",
            [tf.path()],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(n, 0);
}

/// Acceptance 2c: range crossing EOF clamps to remaining bytes.
#[test]
fn range_crossing_eof_clamps() {
    let tf = TempFile::with_bytes(b"0123456789");
    let conn = open_conn();
    let data = read_all(
        &conn,
        "select data from read_local_range(?, 7, 100)",
        tf.path(),
    );
    assert_eq!(data, b"789");
}

/// Acceptance 2d: typed InvalidInput on bad parameters.
#[test]
fn invalid_params_error() {
    let tf = TempFile::with_bytes(b"0123456789");
    let conn = open_conn();
    let err = conn
        .query_row(
            "select data from read_local_range(?, -1, 4)",
            [tf.path()],
            |r| r.get::<_, Vec<u8>>(0),
        )
        .unwrap_err();
    assert!(
        err.to_string().contains("InvalidInput"),
        "expected typed InvalidInput, got: {err}"
    );
    let err = conn
        .query_row(
            "select data from read_local_range(?, 0, 0)",
            [tf.path()],
            |r| r.get::<_, Vec<u8>>(0),
        )
        .unwrap_err();
    assert!(
        err.to_string().contains("InvalidInput"),
        "expected typed InvalidInput, got: {err}"
    );
    let err = conn
        .query_row(
            "select data from read_local_range(?, 0, 4)",
            ["/nonexistent"],
            |r| r.get::<_, Vec<u8>>(0),
        )
        .unwrap_err();
    assert!(
        err.to_string().contains("InvalidInput"),
        "expected typed InvalidInput for unreadable file, got: {err}"
    );
}

/// Acceptance 3: known file content via SQL with offset/length columns.
#[test]
fn known_content_via_sql() {
    let payload: Vec<u8> = (0u8..=255).cycle().take(5000).collect();
    let tf = TempFile::with_bytes(&payload);
    let conn = open_conn();
    let mut stmt = conn
        .prepare("select byte_offset, byte_length, data from read_local_range(?, 100, 4500)")
        .unwrap();
    let mut rows = stmt.query([tf.path()]).unwrap();
    let mut assembled: Vec<u8> = Vec::new();
    let mut expect_offset: i64 = 100;
    while let Some(row) = rows.next().unwrap() {
        let off: i64 = row.get(0).unwrap();
        let len: i64 = row.get(1).unwrap();
        let bytes: Vec<u8> = row.get(2).unwrap();
        assert_eq!(off, expect_offset, "contiguous offset chain");
        assert_eq!(len, bytes.len() as i64, "length column matches bytes");
        assembled.extend_from_slice(&bytes);
        expect_offset = off + len;
    }
    assert_eq!(assembled.len(), 4500);
    assert_eq!(&assembled[..], &payload[100..4600]);
}

/// HTTP range: a local server answers 206 and SQL returns those bytes.
#[test]
fn http_range_reads_partial_body() {
    use std::io::{Read, Write};
    use std::net::TcpListener;

    let body = b"abcdefghij";
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        let (mut sock, _) = listener.accept().unwrap();
        let mut buf = [0u8; 2048];
        let _ = sock.read(&mut buf);
        let header = format!(
            "HTTP/1.1 206 Partial Content\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        );
        sock.write_all(header.as_bytes()).unwrap();
        sock.write_all(body).unwrap();
    });

    let url = format!("http://127.0.0.1:{port}/blob");
    let conn = open_conn();
    let mut stmt = conn
        .prepare("select data from read_http_range(?, 0, 10)")
        .unwrap();
    let mut rows = stmt.query([&url]).unwrap();
    let row = rows.next().unwrap().unwrap();
    let data: Vec<u8> = row.get(0).unwrap();
    assert_eq!(data, body);
}
