//! KD-06 local slice acceptance: `read_local_range` registers, queries a
//! real file, honors offset/length contract semantics (exact bytes,
//! EOF -> empty), and serves known content through SQL.

use frozen_duckdb::host_vfs;
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
