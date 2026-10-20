//! MAX1 poison-rejection exit court.
//!
//! Contract under test: when a versioned cache entry fails ANY admission
//! invariant, `ensure_binary` aborts the process with the literal stderr
//! line `POISON_STATE_REFUSED: <cache path> (<invariant violated>)` and
//! exit code 1 — never a silently-restaged fallback.
//!
//! Chicago discipline: the child runs the REAL `ensure_binary` against a
//! REAL seeded cache under a temp `$HOME` (the builder's only cache-root
//! input); the parent asserts on the real exit code and real stderr of the
//! child process. No mocks.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// One poison scenario: (court name, invariant substring the typed stderr
/// line must carry, seeder). Seeders write into a temp versioned cache dir.
#[allow(clippy::type_complexity)]
const SCENARIOS: &[(&str, &str, fn(&Path))] = &[
    ("foreign marker", "foreign-marker", |versioned: &Path| {
        fs::write(binary(versioned), b"not-even-a-dylib").unwrap();
        fs::write(
            engine_marker(versioned),
            format!("v{}.poisoned", frozen_duckdb_builder::VERSION),
        )
        .unwrap();
    }),
    (
        "truncated dylib + valid marker",
        "truncated-dylib",
        |versioned: &Path| {
            // Valid v{VERSION} marker; dylib present but its .size pin is
            // absent/mismatched => the size-pin invariant fires.
            fs::write(binary(versioned), b"\xcf\xfa\xed\xfe_truncated_payload").unwrap();
            fs::write(
                engine_marker(versioned),
                format!("v{}", frozen_duckdb_builder::VERSION),
            )
            .unwrap();
        },
    ),
    ("missing marker", "missing-marker", |versioned: &Path| {
        fs::write(binary(versioned), b"dylib-bytes").unwrap();
        // no ENGINE_VERSION staged
    }),
    (
        "symlinked marker",
        "symlinked-marker",
        |versioned: &Path| {
            fs::write(binary(versioned), b"dylib-bytes").unwrap();
            let outside = versioned.parent().unwrap().join("outside-marker.txt");
            fs::write(&outside, format!("v{}", frozen_duckdb_builder::VERSION)).unwrap();
            #[cfg(unix)]
            std::os::unix::fs::symlink(&outside, engine_marker(versioned)).unwrap();
        },
    ),
];

fn engine_marker(versioned: &Path) -> PathBuf {
    versioned.join("ENGINE_VERSION")
}

fn binary(versioned: &Path) -> PathBuf {
    versioned.join(format!(
        "libduckdb_arm64.{}",
        if cfg!(target_os = "macos") {
            "dylib"
        } else {
            "so"
        }
    ))
}

#[test]
fn poison_state_refused_exits_1_with_typed_stderr() {
    // ---- child mode: this process was re-exec'd to run the real
    // ensure_binary against a seeded poison cache; its exit code IS the
    // assertion target of the parent court below.
    if let Ok(idx) = std::env::var("POISON_COURT_CHILD") {
        let scenario: usize = idx.parse().expect("POISON_COURT_CHILD=<index>");
        let (name, _invariant, seed) = &SCENARIOS[scenario];

        let home = std::env::var("POISON_COURT_HOME")
            .expect("child mode requires POISON_COURT_HOME=<temp home>");
        let cache_root = Path::new(&home).join(".frozen-duckdb").join("cache");
        let versioned = cache_root.join(format!("v{}-arm64", frozen_duckdb_builder::VERSION));
        fs::create_dir_all(&versioned).expect("seed versioned cache dir");
        seed(&versioned);
        eprintln!("[court] scenario: {}", name);

        match frozen_duckdb_builder::ensure_binary() {
            Ok(p) => {
                eprintln!(
                    "[court] FAIL: ensure_binary SERVED a poisoned entry: {}",
                    p.display()
                );
                std::process::exit(0); // served poison = court failure
            }
            Err(e) => {
                eprintln!(
                    "[court] FAIL: ensure_binary returned Err instead of the typed abort: {e:#}"
                );
                std::process::exit(2); // wrong failure mode
            }
        }
    }

    // ---- parent mode: re-exec this same test binary as the poison subject.
    let self_exe = std::env::current_exe().expect("current_exe");
    let mut failures = Vec::new();

    for (i, (name, invariant, _seed)) in SCENARIOS.iter().enumerate() {
        let home = tempfile::tempdir().expect("temp home");
        let out = Command::new(&self_exe)
            // --nocapture: libtest otherwise swallows the child's stderr
            // (the typed POISON_STATE_REFUSED line) on process::exit(1).
            .arg("--nocapture")
            .arg("--exact")
            .arg("poison_state_refused_exits_1_with_typed_stderr")
            .env("POISON_COURT_CHILD", i.to_string())
            .env("POISON_COURT_HOME", home.path())
            .env("HOME", home.path()) // builder cache root input
            .env("ARCH", "arm64") // deterministic detect_architecture
            .output()
            .expect("spawn court child");

        let stderr = String::from_utf8_lossy(&out.stderr);
        let ok_exit = out.status.code() == Some(1);
        let ok_line = stderr.contains("POISON_STATE_REFUSED:");
        let ok_inv = stderr.contains(&format!("({})", invariant));
        let ok = ok_exit && ok_line && ok_inv;

        eprintln!(
            "{} scenario {:<34} exit={:?} stderr-line={} invariant={}",
            if ok { "PASS" } else { "FAIL" },
            name,
            out.status.code(),
            ok_line,
            ok_inv
        );
        if !ok {
            failures.push(format!(
                "scenario '{}' exit={:?} (want 1)\nstderr:\n{}",
                name,
                out.status.code(),
                stderr
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "poison_state_exit court: {} of {} scenarios FAILED\n{}",
        failures.len(),
        SCENARIOS.len(),
        failures.join("\n---\n")
    );
    eprintln!(
        "poison_state_exit court: all {} poison scenarios refused with typed stderr + exit 1",
        SCENARIOS.len()
    );
}
