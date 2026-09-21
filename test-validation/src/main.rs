//! frozen-duckdb validation harness (TR5): a real gate, not decorative prints.
//!
//! Every check ASSERTS. Any failure is collected, reported to stderr, and the
//! process exits nonzero. The gate proves the builder-contract end to end:
//!
//! 1. `frozen_duckdb_builder::ensure_binary()` resolves the expected binary.
//! 2. That binary exists as a regular file.
//! 3. The DuckDB version is DERIVED FROM THE BUILDER — `ensure_binary()`
//!    returns `~/.frozen-duckdb/cache/v{VERSION}-{arch}/…`, a directory name
//!    the builder formats from its own `VERSION` constant. No version is
//!    hardcoded here.
//! 4. The dylib is loaded via FFI and `duckdb_library_version()` must equal
//!    `v{VERSION}` from step 3.
//! 5. Header layout holds: `{cache}/v{VERSION}-{arch}/duckdb/duckdb.h`.
//! 6. The replicated builder naming/layout rule agrees with the path the
//!    builder actually returned.

use frozen_duckdb_builder::ensure_binary;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    println!("🧪 frozen-duckdb validation harness (asserting gate)");
    let mut failures: Vec<String> = Vec::new();

    // Gate 1: builder crate resolves the binary via its public API.
    let binary_path: PathBuf = match ensure_binary() {
        Ok(p) => {
            println!("✅ builder resolved binary: {}", p.display());
            p
        }
        Err(e) => {
            failures.push(format!(
                "builder ensure_binary() failed: {e} (cannot locate staged DuckDB binary)"
            ));
            return report(&failures);
        }
    };

    // Gate 2: the binary exists and is a regular file.
    match std::fs::metadata(&binary_path) {
        Ok(m) if m.is_file() => {
            println!(
                "✅ binary exists ({} bytes): {}",
                m.len(),
                binary_path.display()
            );
        }
        Ok(_) => failures.push(format!(
            "builder path is not a regular file: {}",
            binary_path.display()
        )),
        Err(e) => failures.push(format!(
            "binary missing at builder path {}: {e}",
            binary_path.display()
        )),
    }

    // Gate 3: derive version + arch from the builder's own cache layout
    // ({cache}/v{VERSION}-{arch}/libduckdb_{arch}.{ext}).
    let (version, arch) = match parse_versioned_dir(binary_path.parent()) {
        Ok((v, a)) => {
            println!("✅ builder-derived DuckDB version: v{v} (arch: {a})");
            (v, a)
        }
        Err(e) => {
            failures.push(e);
            return report(&failures);
        }
    };

    // Gate 4: replicated builder rule (get_binary_path semantics) must agree
    // with the path the builder actually returned.
    match home_dir() {
        Ok(home) => {
            let expected = expected_binary_path(&home, &version, &arch);
            if binary_path == expected {
                println!("✅ cache layout matches builder rule: {}", expected.display());
            } else {
                failures.push(format!(
                    "cache layout drift: builder returned {}, replicated rule expects {}",
                    binary_path.display(),
                    expected.display()
                ));
            }
        }
        Err(e) => failures.push(e),
    }

    // Gate 5: header layout — {cache}/v{VERSION}-{arch}/duckdb/duckdb.h
    // (what bindgen in frozen-duckdb-sys consumes).
    let header = binary_path
        .parent()
        .map(|d| d.join("duckdb").join("duckdb.h"))
        .unwrap_or_else(|| PathBuf::from("<no parent>/duckdb/duckdb.h>"));
    match std::fs::metadata(&header) {
        Ok(m) if m.is_file() => {
            println!("✅ header layout present: {}", header.display());
        }
        Ok(_) => failures.push(format!("header path is not a regular file: {}", header.display())),
        Err(e) => failures.push(format!("header layout broken ({} missing): {e}", header.display())),
    }

    // Gate 6: load the staged dylib via FFI; its self-reported version must
    // equal the builder-derived expectation.
    match ffi_library_version(&binary_path, &format!("v{version}")) {
        Ok(reported) => println!("✅ dylib duckdb_library_version() = {reported}"),
        Err(e) => failures.push(e),
    }

    // Kept check (now asserting): system architecture must agree with the
    // arch encoded in the builder's cache layout.
    match detect_architecture() {
        Some(a) if a == arch || (a == "aarch64" && arch == "arm64") => {
            println!("✅ architecture detection agrees: {a}");
        }
        Some(a) => failures.push(format!(
            "architecture mismatch: uname reports '{a}' but builder cache is laid out for '{arch}'"
        )),
        None => failures.push("failed to detect architecture via uname -m".to_string()),
    }

    report(&failures)
}

/// Parse `v{VERSION}-{arch}` out of the versioned cache directory name the
/// builder itself constructs. This is how the harness reads the builder's
/// VERSION without a local hardcode and without touching the builder crate.
fn parse_versioned_dir(dir: Option<&Path>) -> Result<(String, String), String> {
    let dir = dir.ok_or("binary path has no parent directory".to_string())?;
    let name = dir
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| format!("unreadable cache directory name: {}", dir.display()))?;
    let rest = name
        .strip_prefix('v')
        .ok_or_else(|| format!("cache directory '{name}' does not start with 'v' (expected v{{VERSION}}-{{arch}})"))?;
    let (version, arch) = rest
        .rsplit_once('-')
        .ok_or_else(|| format!("cache directory '{name}' does not encode VERSION-arch"))?;
    if version.is_empty() || arch.is_empty() {
        return Err(format!("cache directory '{name}' has empty VERSION or arch"));
    }
    Ok((version.to_string(), arch.to_string()))
}

fn home_dir() -> Result<PathBuf, String> {
    std::env::var("HOME")
        .map(PathBuf::from)
        .map_err(|e| format!("HOME environment variable not usable: {e}"))
}

/// Extension mirror of the builder's `get_binary_path` (private), by target OS.
fn lib_extension() -> &'static str {
    if cfg!(target_os = "macos") {
        "dylib"
    } else if cfg!(target_os = "linux") {
        "so"
    } else if cfg!(target_os = "windows") {
        "dll"
    } else {
        "so"
    }
}

/// Replicated builder layout: $HOME/.frozen-duckdb/cache/v{V}-{arch}/libduckdb_{arch}.{ext}
fn expected_binary_path(home: &Path, version: &str, arch: &str) -> PathBuf {
    home.join(".frozen-duckdb")
        .join("cache")
        .join(format!("v{version}-{arch}"))
        .join(format!("libduckdb_{arch}.{}", lib_extension()))
}

/// Mirror of the builder's `detect_architecture` (private): `uname -m`.
fn detect_architecture() -> Option<String> {
    let output = std::process::Command::new("uname").arg("-m").output().ok()?;
    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// dlopen the staged dylib and call duckdb_library_version(), asserting it
/// equals `expected` (e.g. "v1.5.5").
fn ffi_library_version(binary_path: &Path, expected: &str) -> Result<String, String> {
    unsafe {
        let lib = libloading::Library::new(binary_path)
            .map_err(|e| format!("failed to load dylib {}: {e}", binary_path.display()))?;
        let sym: libloading::Symbol<unsafe extern "C" fn() -> *const c_char> = lib
            .get(b"duckdb_library_version\0")
            .map_err(|e| {
                format!(
                    "symbol duckdb_library_version not found in {}: {e}",
                    binary_path.display()
                )
            })?;
        let ptr = sym();
        if ptr.is_null() {
            return Err("duckdb_library_version() returned NULL".to_string());
        }
        let reported = CStr::from_ptr(ptr).to_string_lossy().into_owned();
        if reported != expected {
            return Err(format!(
                "version mismatch: staged dylib reports '{reported}' but builder expects '{expected}'"
            ));
        }
        Ok(reported)
    }
}

fn report(failures: &[String]) -> ExitCode {
    if failures.is_empty() {
        println!("\n🎉 ALL ASSERTIONS HELD: builder path, binary presence, cache");
        println!("   layout, header layout, and dylib FFI version are consistent.");
        ExitCode::SUCCESS
    } else {
        eprintln!("\n❌ frozen-duckdb validation FAILED: {} assertion(s) broke:", failures.len());
        for f in failures {
            eprintln!("   - {f}");
        }
        ExitCode::from(1)
    }
}
