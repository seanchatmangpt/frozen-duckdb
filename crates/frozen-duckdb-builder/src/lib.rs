//! # Frozen DuckDB Builder
//!
//! This crate handles downloading prebuilt mega-libraries from GitHub Releases
//! or compiling them locally as a fallback. It manages caching under
//! `~/.frozen-duckdb/cache/` (one `v{version}-{arch}/` directory per build)
//! to ensure fast subsequent builds.

use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{debug, info, warn};

const VERSION: &str = "2.0.0-dev";
/// Pinned upstream duckdb/duckdb main commit this dev build stages/compiles
/// from (dev heads move — load-bearing pin).
const SOURCE_PIN: &str = "b6013f7aa2f36caabb9466ce1791667ea8a280bb";
const CACHE_DIR: &str = ".frozen-duckdb";
const BINARY_NAME: &str = "libduckdb";

/// Absolute path of this crate's vendored-headers directory, captured when this
/// crate is compiled (build-dependency scripts see their own CARGO_MANIFEST_DIR,
/// not this crate's, so it cannot be read at call time)
const VENDORED_HEADERS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/vendored-headers");

/// Name of the host target OS ("macos" | "linux" | "windows" | "other"),
/// separated from `library_extension_for` so the naming law is unit-testable
/// on every host.
fn target_os_name() -> &'static str {
    if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "other"
    }
}

/// Shared-library file extension for a target OS.
///
/// Asset law (TR7): `.dylib` on macOS, `.so` on Linux/other, `.dll` on
/// Windows. Windows is on the documented local-compile fallback — the
/// extension is kept only so `get_binary_path` stays consistent with the
/// local-compile output; no `.dll` release asset exists or is planned.
fn library_extension_for(os: &str) -> &'static str {
    match os {
        "macos" => "dylib",
        "linux" => "so",
        "windows" => "dll",
        _ => "so", // Default fallback
    }
}

/// Shared-library file extension for the host platform.
fn library_extension() -> &'static str {
    library_extension_for(target_os_name())
}

/// Name of the frozen-duckdb release asset carrying the shared library for
/// `arch` on `os` (parameterized form, unit-testable cross-platform).
///
/// Asset law (TR7): `libduckdb_{arch}.dylib` on macOS, `libduckdb_{arch}.so`
/// on Linux. This must stay in lockstep with `get_binary_path`, which names
/// the cached copy with the same platform extension.
///
/// NOTE: `.so` release assets do NOT exist in any published frozen-duckdb
/// release yet. As of 2026-09-21 no release (and no tag) has been published
/// at all — probed via the GitHub API and an anonymous asset-URL request
/// (HTTP 404) — so download attempts return HTTP 404 on EVERY platform and
/// `ensure_binary` falls back to `compile_duckdb_locally`, which is the
/// operative path today. The v2.0.0-dev workflow is planned to ship the macOS
/// asset set first; the Linux asset set lands with a later tag (TR2
/// coordinates the workflow side).
fn release_asset_name_for(os: &str, arch: &str) -> String {
    format!("libduckdb_{}.{}", arch, library_extension_for(os))
}

/// Name of the frozen-duckdb release asset for `arch` on the host platform.
fn release_asset_name(arch: &str) -> String {
    release_asset_name_for(target_os_name(), arch)
}

/// Ensure the prebuilt DuckDB binary is available
///
/// This function:
/// 1. Checks for cached binary in ~/.frozen-duckdb/cache/v{VERSION}-{arch}/
/// 2. If missing, tries to download from GitHub Release
/// 3. If download fails, compiles locally as fallback
/// 4. Returns path to the binary
pub fn ensure_binary() -> Result<PathBuf> {
    let arch = detect_architecture()?;
    let cache_dir = get_cache_dir()?;
    let versioned_cache = cache_dir.join(format!("v{}-{}", VERSION, arch));
    let binary_path = get_binary_path(&versioned_cache, &arch);

    // Structural anti-poison (GAP9 follow-on): a versioned cache dir is only
    // trusted when its ENGINE_VERSION marker carries exactly `v{VERSION}` —
    // the builder's own pin. A marker written by a different builder version
    // (or absent, e.g. the 13:22-style poisoning) demotes the cache entry to
    // a MISS so the download/compile path restages it atomically.
    let mut cache_hit = false;
    if binary_path.exists() {
        cache_hit = accept_cached_entry(
            &versioned_cache,
            &binary_path,
            &arch,
            MIN_ENGINE_DYLIB_BYTES,
        )?;
    }

    if cache_hit {
        // accepted above
    } else if let Ok(prebuilt_path) = check_prebuilt_binary(&arch) {
        verify_library_slice(&prebuilt_path, &arch)?;
        info!(
            "Found prebuilt binary, copying to cache: {}",
            prebuilt_path.display()
        );
        copy_prebuilt_to_cache(&prebuilt_path, &binary_path)?;
        info!("Successfully set up prebuilt binary and headers");
    } else {
        info!("No cached binary found at: {}", binary_path.display());
        info!("Cache directory: {}", versioned_cache.display());

        // Debug: show what's in the cache directory
        if let Ok(entries) = fs::read_dir(&versioned_cache) {
            info!("Versioned cache directory contents:");
            for entry in entries.flatten() {
                info!("  {}", entry.path().display());
            }
        }

        info!("Attempting to download...");

        // Try to download from GitHub Release
        match download_from_github_release(&versioned_cache, &arch) {
            Ok(path) => {
                info!(
                    "Successfully downloaded frozen DuckDB binary: {}",
                    path.display()
                );
            }
            Err(e) => {
                warn!("Failed to download from GitHub Release: {}", e);
                info!("Falling back to local compilation...");

                // Fallback to local compilation
                let path = compile_duckdb_locally(&versioned_cache, &arch)
                    .context("Failed to compile DuckDB locally")?;
                info!("Successfully compiled DuckDB binary: {}", path.display());
            }
        }
    }

    // Normalize the cache layout so downstream builds always work:
    // headers under duckdb/ for bindgen, plain libduckdb.dylib for -lduckdb
    ensure_headers(&versioned_cache)?;
    ensure_link_name(&versioned_cache, &arch)?;

    Ok(binary_path)
}

/// Trusted-hit law for a versioned cache entry (DT1 refactor of the
/// ensure_binary hit block): the ENGINE_VERSION marker must carry the
/// builder's own pin, the `.size` sidecar must match the dylib's actual
/// length (AD1: `lipo -archs` reads only the Mach-O header, so a truncated
/// dylib still slice-verifies), and the slice must verify.
///
/// `Ok(true)` = serve the cached dylib. `Ok(false)` = demote to a REAL MISS —
/// the pre-refactor code logged "restaging" and then `return Ok(binary_path)`,
/// serving the exact unverified dylib it had just declared untrusted.
/// `Err` = hard failure the caller propagates.
fn accept_cached_entry(
    versioned_cache: &Path,
    binary_path: &Path,
    arch: &str,
    min_bytes: u64,
) -> Result<bool> {
    // C2 strictness (AD6 siege): the label law is BYTE-EXACT. `stage_engine_marker`
    // writes the pin with no trailing bytes, so a marker whose raw payload is
    // `v{VERSION}\n` or `v{VERSION}\r\n` was NOT staged by this builder and
    // demotes to MISS like any foreign label. `read_engine_marker`'s trim stays
    // for display only. The DT1 regular-file law is preserved: symlinks and
    // directories demote before their contents are even read.
    let marker_path = engine_marker_path(versioned_cache);
    let is_regular = fs::symlink_metadata(&marker_path)
        .map(|m| m.is_file())
        .unwrap_or(false);
    let raw = is_regular.then(|| fs::read(&marker_path).ok()).flatten();
    let marker_label = raw
        .as_deref()
        .map(|bytes| String::from_utf8_lossy(bytes).trim().to_string())
        .unwrap_or_else(|| "<absent>".to_string());
    if raw.as_deref() != Some(expected_engine_marker().as_bytes()) {
        warn!(
            "cache entry labeled v{} carries marker '{}'; restaging",
            VERSION, marker_label
        );
        return Ok(false);
    }

    match read_library_size_pin(binary_path) {
        Some(pin) if pin == library_len(binary_path) => {}
        pin => {
            warn!(
                "cache binary {} length {} mismatches its size pin {:?}; demoting to MISS",
                binary_path.display(),
                library_len(binary_path),
                pin
            );
            // Fail-closed demotion: remove the untrusted dylib and its pin so
            // no concurrent or retried reader can be served them, and so the
            // restage starts from a genuinely empty entry.
            let _ = fs::remove_file(binary_path);
            let _ = fs::remove_file(library_size_pin_path(binary_path));
            return Ok(false);
        }
    }

    // A cache written before TR7 closed may hold the wrong slice under this
    // label (x86_64 bytes in v{VER}-arm64/libduckdb_arm64.dylib).
    verify_library_slice_for(target_os_name(), binary_path, arch, min_bytes)
        .map(|_| ())
        .with_context(|| {
            format!(
                "stale cached binary does not match its label; remove {} and rebuild",
                versioned_cache.display()
            )
        })?;
    Ok(true)
}

/// Copy the vendored headers into the cache when missing.
///
/// bindgen expects `{cache}/duckdb/duckdb.h`, but release downloads carry only
/// the dylib, so the headers for the pinned DuckDB version ship inside this
/// crate.
fn ensure_headers(versioned_cache: &Path) -> Result<()> {
    let headers_dir = versioned_cache.join("duckdb");
    // DT1: the early return requires BOTH headers. The previous duckdb.h-only
    // check let a run that crashed (or bailed on a missing vendored .hpp)
    // between the two copies leave a permanently half-staged headers dir that
    // every later call "normalized" by doing nothing.
    if headers_dir.join("duckdb.h").exists() && headers_dir.join("duckdb.hpp").exists() {
        return Ok(());
    }
    fs::create_dir_all(&headers_dir).context("Failed to create cache headers directory")?;
    for header_name in ["duckdb.h", "duckdb.hpp"] {
        let src = Path::new(VENDORED_HEADERS_DIR).join(header_name);
        if !src.exists() {
            anyhow::bail!("Vendored header missing: {}", src.display());
        }
        let dest = headers_dir.join(header_name);
        atomic_copy(&src, &dest)
            .with_context(|| format!("Failed to copy vendored header {}", src.display()))?;
        info!("Copied vendored header: {}", dest.display());
    }
    Ok(())
}

/// Create the plain `libduckdb.{dylib,so}` link name next to the arch-suffixed
/// cached binary when missing, so `-lduckdb` resolves. The link name carries
/// the platform extension (TR7): `libduckdb.dylib` on macOS, `libduckdb.so`
/// on Linux.
fn ensure_link_name(versioned_cache: &Path, arch: &str) -> Result<()> {
    let link_name = versioned_cache.join(format!("{}.{}", BINARY_NAME, library_extension()));
    if link_name.exists() {
        return Ok(());
    }
    let target = get_binary_path(versioned_cache, arch);
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&target, &link_name)
            .with_context(|| format!("Failed to symlink {}", link_name.display()))?;
    }
    #[cfg(not(unix))]
    {
        fs::copy(&target, &link_name).with_context(|| {
            format!("Failed to copy binary to link name {}", link_name.display())
        })?;
    }
    info!("Linked {} -> {}", link_name.display(), target.display());
    Ok(())
}

/// Check if prebuilt binary exists in project directory
fn check_prebuilt_binary(arch: &str) -> Result<PathBuf> {
    // Try to find the project root by looking for prebuilt directory
    let current_dir = env::current_dir().context("Failed to get current directory")?;

    let prebuilt_dir = current_dir.join("prebuilt");
    if !prebuilt_dir.exists() {
        anyhow::bail!("Prebuilt directory not found: {}", prebuilt_dir.display());
    }

    // prebuilt/ mirrors the release asset naming law (TR7):
    // libduckdb_{arch}.dylib on macOS, libduckdb_{arch}.so on Linux.
    let binary_name = release_asset_name(arch);
    let binary_path = prebuilt_dir.join(&binary_name);

    if binary_path.exists() {
        Ok(binary_path)
    } else {
        anyhow::bail!("Prebuilt binary not found: {}", binary_path.display());
    }
}

/// Copy prebuilt binary and headers to cache directory
fn copy_prebuilt_to_cache(prebuilt_path: &Path, cache_path: &Path) -> Result<()> {
    // Ensure cache directory exists
    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent).context("Failed to create cache directory")?;
    }

    // Copy the binary
    atomic_copy(prebuilt_path, cache_path).context("Failed to copy prebuilt binary to cache")?;

    // Make binary executable on Unix systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(cache_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(cache_path, perms)?;
    }

    // Copy headers as well
    copy_prebuilt_headers(cache_path)?;

    // DT1 persist-order law: dylib -> `.size` pin -> ENGINE_VERSION marker.
    // The marker is the trust root and must land LAST, so a crash anywhere
    // earlier leaves either no marker (clean MISS) or a marker whose pin
    // mismatches (demoted MISS) — never a valid marker over an unstaged pair.
    if let Some(versioned_cache) = cache_path.parent() {
        stage_library_size_pin(cache_path)?;
        stage_engine_marker(versioned_cache)?;
    }

    info!(
        "Copied prebuilt binary and headers to cache: {}",
        cache_path.display()
    );
    Ok(())
}

/// Copy prebuilt headers to cache directory
fn copy_prebuilt_headers(cache_path: &Path) -> Result<()> {
    let current_dir = env::current_dir().context("Failed to get current directory")?;

    let prebuilt_dir = current_dir.join("prebuilt");
    let headers_dest = cache_path.parent().unwrap().join("duckdb");
    fs::create_dir_all(&headers_dest).context("Failed to create cache headers directory")?;

    // Copy header files directly to cache directory (expected by bindgen)
    let header_files = ["duckdb.h", "duckdb.hpp"];
    for header_name in &header_files {
        let src_path = prebuilt_dir.join(header_name);
        if src_path.exists() {
            let dest_path = headers_dest.join(header_name);
            atomic_copy(&src_path, &dest_path).context("Failed to copy header file")?;
            info!("Copied header: {}", dest_path.display());
        }
    }

    Ok(())
}

/// Detect the architecture this build must target.
///
/// TR7: on an Apple-silicon CI host, `uname -m` reports the HOST arch even
/// when the workflow targets the other macOS slice — which is exactly how
/// the v1.5.5 x86_64 release leg (historical, TR7) built an x86_64 cmake slice but detected
/// itself as arm64, placed the artifact under `v{VERSION}-arm64/` with an
/// `libduckdb_arm64.dylib` name, and failed closed in the workflow's find
/// step. The build-binaries workflow exports `CMAKE_OSX_ARCHITECTURES` (and
/// `ARCH`) per matrix leg, and the same value is what gets handed to cmake,
/// so an explicit single-slice target in those variables is the authority
/// for detection, cache placement, and asset naming. Multi-arch lists and
/// unknown values fall through to host detection rather than adopting a
/// target the cmake invocation cannot be proven to match.
fn detect_architecture() -> Result<String> {
    let explicit = [
        env::var("CMAKE_OSX_ARCHITECTURES").ok(),
        env::var("ARCH").ok(),
    ];
    let host = host_architecture()?;
    resolve_target_architecture(&explicit, &host)
}

/// Raw `uname -m` answer for the build host (the machine running cmake).
fn host_architecture() -> Result<String> {
    let output = Command::new("uname")
        .arg("-m")
        .output()
        .context("Failed to run uname command")?;
    Ok(String::from_utf8(output.stdout)
        .context("Invalid UTF-8 in uname output")?
        .trim()
        .to_string())
}

/// Canonical single-slice name for `value`, or `None` for anything that is
/// not exactly one supported slice (multi-arch lists, unknown or mis-cased
/// names such as `amd64`/`X86_64`, empty strings).
fn single_slice(value: &str) -> Option<&'static str> {
    match value.trim() {
        "x86_64" => Some("x86_64"),
        "arm64" | "aarch64" => Some("arm64"),
        _ => None,
    }
}

/// Precedence law over the explicit target sources (TR7 hardening): the
/// FIRST source that names exactly one supported slice wins, in order
/// `CMAKE_OSX_ARCHITECTURES`, then `ARCH`; an unset, empty, multi-arch or
/// unrecognized earlier source does not shadow a valid later one. With no
/// valid explicit slice the host decides.
fn resolve_target_architecture(explicit: &[Option<String>], host: &str) -> Result<String> {
    let chosen = explicit
        .iter()
        .flatten()
        .map(|v| v.as_str())
        .find(|v| single_slice(v).is_some());
    resolve_architecture(chosen, host)
}

/// Pure decision core of [`detect_architecture`] (kept env-free so the
/// slice-targeting law is unit-testable without process-global mutation).
fn resolve_architecture(explicit: Option<&str>, host: &str) -> Result<String> {
    if let Some(slice) = explicit.and_then(single_slice) {
        return Ok(slice.to_string());
    }
    match single_slice(host) {
        Some(slice) => Ok(slice.to_string()),
        None => anyhow::bail!("Unsupported architecture: {}", host.trim()),
    }
}

/// Value handed to cmake as `-DCMAKE_OSX_ARCHITECTURES` for a local compile
/// labeled `arch`. An operator-set value passes through verbatim (it may be
/// a universal list); otherwise on macOS the LABEL itself is pinned, so a
/// label taken from `ARCH` can never diverge from the slice clang emits
/// (pre-hardening, `ARCH=x86_64` on an arm64 host labeled an arm64 build
/// x86_64). Off macOS cmake ignores the flag, so none is emitted.
fn cmake_osx_architectures_for(os: &str, env_value: Option<&str>, arch: &str) -> Option<String> {
    if let Some(v) = env_value.map(str::trim).filter(|v| !v.is_empty()) {
        return Some(v.to_string());
    }
    (os == "macos").then(|| arch.to_string())
}

/// Refuse a local compile whose label cannot be honored: off macOS there is
/// no slice-selection flag, so a cross-arch target would produce a
/// host-arch library under a foreign-arch name (the TR7 mislabel class).
fn check_local_compile_target(os: &str, arch: &str, host: &str) -> Result<()> {
    if os == "macos" || single_slice(host) == Some(arch) {
        return Ok(());
    }
    anyhow::bail!(
        "cross-architecture local compile unsupported on {}: target {} but host {} \
         (the built library would be mislabeled; TR7)",
        os,
        arch,
        host.trim()
    )
}

/// Outcome of a slice check that did not find a mismatch.
#[derive(Debug, PartialEq, Eq)]
enum SliceCheck {
    /// `lipo -archs` listed the labeled slice.
    Verified,
    /// No checker available on this host (non-macOS, or lipo absent); the
    /// label is UNVERIFIED, not admitted.
    Unverifiable(String),
}

/// True when whitespace-separated `lipo -archs` output lists exactly `arch`
/// (`arm64e` is not `arm64`).
fn lipo_archs_contain(lipo_output: &str, arch: &str) -> bool {
    lipo_output.split_whitespace().any(|a| a == arch)
}

/// Production size floor for a cached DuckDB engine dylib. The real cached
/// engine is ~66MB; anything under 10MiB cannot be a DuckDB engine, so a
/// truncated download (AD1: valid Mach-O header, missing payload) is
/// hard-refused by [`verify_library_slice_for`] even though `lipo -archs`
/// would accept it. Tests pass an explicit smaller floor so real tiny dylibs
/// still exercise the rest of the check.
const MIN_ENGINE_DYLIB_BYTES: u64 = 10 * 1024 * 1024;

/// Recognized Mach-O magics: 64-bit thin (both byte orders), fat/universal
/// (both byte orders). AD1 hardening: a payload without one of these in its
/// first 4 bytes is not a Mach-O library.
fn is_macho_magic(magic: &[u8; 4]) -> bool {
    matches!(
        magic,
        [0xcf, 0xfa, 0xed, 0xfe]
            | [0xfe, 0xed, 0xfa, 0xcf]
            | [0xca, 0xfe, 0xba, 0xbe]
            | [0xbe, 0xba, 0xfe, 0xca]
    )
}

/// Check that the Mach-O at `path` actually carries the slice it is labeled
/// with (TR7 falsifier made permanent: a label/slice mismatch fails closed
/// instead of shipping or caching a mislabeled library).
fn verify_library_slice_for(
    os: &str,
    path: &Path,
    arch: &str,
    min_bytes: u64,
) -> Result<SliceCheck> {
    if os != "macos" {
        return Ok(SliceCheck::Unverifiable(format!(
            "slice check is macOS-only (os={})",
            os
        )));
    }
    let out = match Command::new("lipo").arg("-archs").arg(path).output() {
        Ok(out) => out,
        Err(e) => return Ok(SliceCheck::Unverifiable(format!("lipo unavailable: {}", e))),
    };
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    if !out.status.success() {
        if stderr.contains("xcrun: error") {
            return Ok(SliceCheck::Unverifiable(format!(
                "lipo unavailable: {}",
                stderr.trim()
            )));
        }
        anyhow::bail!(
            "{} is not a readable Mach-O library (lipo -archs exit {:?}): {}",
            path.display(),
            out.status.code(),
            stderr.trim()
        );
    }
    if lipo_archs_contain(&stdout, arch) {
        // AD1 hardening: `lipo -archs` reads only the Mach-O header/load
        // commands, so a truncated dylib with a valid header still passes.
        // Two structural checks close that hole: (1) the file must be at
        // least `min_bytes` (a real DuckDB engine is ~66MB; the production
        // floor is MIN_ENGINE_DYLIB_BYTES = 10MiB), and (2) the first 4
        // bytes must be a Mach-O magic (thin arm64/x86_64, fat, or fat
        // byte-swapped). Either failure is a hard error, not Unverifiable.
        let len = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        if len < min_bytes {
            anyhow::bail!(
                "{} is {} bytes, below the {} byte engine size floor (truncated download? \
                 AD1): refusing to serve",
                path.display(),
                len,
                min_bytes
            );
        }
        let mut magic = [0u8; 4];
        fs::File::open(path)
            .and_then(|mut f| f.read_exact(&mut magic))
            .context("failed to read Mach-O magic")?;
        if !is_macho_magic(&magic) {
            anyhow::bail!(
                "{} lacks a Mach-O magic (first bytes {:02x?}); not a Mach-O library (AD1): \
                 refusing to serve",
                path.display(),
                magic
            );
        }
        Ok(SliceCheck::Verified)
    } else {
        anyhow::bail!(
            "TR7 slice mismatch: {} is labeled {} but carries [{}]",
            path.display(),
            arch,
            stdout.trim()
        )
    }
}

/// Host-platform form of [`verify_library_slice_for`]; an unverifiable
/// check is logged, a mismatch or unreadable library is an error.
fn verify_library_slice(path: &Path, arch: &str) -> Result<()> {
    match verify_library_slice_for(target_os_name(), path, arch, MIN_ENGINE_DYLIB_BYTES)? {
        SliceCheck::Verified => debug!("slice {} verified in {}", arch, path.display()),
        SliceCheck::Unverifiable(why) => {
            warn!("slice {} UNVERIFIED for {}: {}", arch, path.display(), why)
        }
    }
    Ok(())
}

/// Atomically write `bytes` to `dest`: write to a `.tmp-<pid>-<name>` sibling
/// in the same directory, then `fs::rename` over the final path. Concurrent
/// readers (X3) either see the old complete file or the new complete file —
/// never a partial mid-write binary that fails its Mach-O slice check (T2).
/// Renames within one directory are atomic on POSIX and Windows.
fn atomic_write_bytes(dest: &Path, bytes: &[u8]) -> Result<()> {
    let parent = dest
        .parent()
        .context("destination has no parent directory")?;
    let file_name = dest
        .file_name()
        .context("destination has no file name")?
        .to_string_lossy()
        .into_owned();
    let tmp = tmp_sibling(parent, &file_name);
    if let Err(e) = fs::write(&tmp, bytes) {
        let _ = fs::remove_file(&tmp); // never leak a partial temp sibling
        return Err(e).with_context(|| format!("Failed to write {}", tmp.display()));
    }
    if let Err(e) = fs::rename(&tmp, dest) {
        // DT1: a failed rename must not strand a complete-but-unpublished
        // temp sibling — the next audit's no-.tmp-residue check reads any
        // survivor as corruption evidence.
        let _ = fs::remove_file(&tmp);
        return Err(e)
            .with_context(|| format!("Failed to rename {} -> {}", tmp.display(), dest.display()));
    }
    Ok(())
}

/// Process-unique-per-call temp sibling: `.tmp-<pid>-<seq>-<name>`.
///
/// Concurrency court fix (lane AD1): the previous `.tmp-<pid>-<name>` form
/// collided across THREADS of one process (build scripts, parallel tests, a
/// multi-threaded persist), so thread A's `rename` could fire while thread B
/// was mid-write to the SAME tmp file — publishing B's partial bytes at the
/// final path under a valid ENGINE_VERSION marker. The sequence counter makes
/// every write's staging file private; renames within one directory remain
/// atomic.
fn tmp_sibling(parent: &Path, file_name: &str) -> PathBuf {
    static TMP_SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let seq = TMP_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    parent.join(format!(".tmp-{}-{}-{}", std::process::id(), seq, file_name))
}

/// Atomic variant of [`fs::copy`] with the same `.tmp-<pid>-<name>` sibling +
/// `fs::rename` discipline as [`atomic_write_bytes`].
fn atomic_copy(src: &Path, dest: &Path) -> Result<()> {
    let parent = dest
        .parent()
        .context("destination has no parent directory")?;
    let file_name = dest
        .file_name()
        .context("destination has no file name")?
        .to_string_lossy()
        .into_owned();
    let tmp = tmp_sibling(parent, &file_name);
    if let Err(e) = fs::copy(src, &tmp) {
        let _ = fs::remove_file(&tmp); // never leak a partial temp sibling
        return Err(e).with_context(|| format!("Failed to copy to {}", tmp.display()));
    }
    if let Err(e) = fs::rename(&tmp, dest) {
        // DT1: same no-residue law as atomic_write_bytes.
        let _ = fs::remove_file(&tmp);
        return Err(e)
            .with_context(|| format!("Failed to rename {} -> {}", tmp.display(), dest.display()));
    }
    Ok(())
}

/// Expected ENGINE_VERSION marker payload: exactly `v{VERSION}` (the
/// builder's own pin; no trailing newline).
fn expected_engine_marker() -> String {
    format!("v{}", VERSION)
}

/// Marker file path inside a versioned cache directory.
fn engine_marker_path(versioned_cache: &Path) -> PathBuf {
    versioned_cache.join("ENGINE_VERSION")
}

/// Write the ENGINE_VERSION marker (atomically) after a successful cache
/// persist. Called at every GAP9 atomic cache-persist site.
fn stage_engine_marker(versioned_cache: &Path) -> Result<()> {
    let bytes = expected_engine_marker().into_bytes();
    atomic_write_bytes(&engine_marker_path(versioned_cache), &bytes)
        .context("Failed to write ENGINE_VERSION cache marker")
}

/// Read the ENGINE_VERSION marker. Errors when absent or unreadable; a
/// present-but-different payload is returned as `Ok(other_label)`.
fn read_engine_marker(versioned_cache: &Path) -> Result<String> {
    let path = engine_marker_path(versioned_cache);
    // DT1 (AD6 hardening): the marker must be a regular file. A directory
    // fails fs::read anyway, but a SYMLINK would be followed to arbitrary
    // content outside the cache entry — demote any non-regular marker to
    // MISS instead of trusting where it points.
    let meta =
        fs::symlink_metadata(&path).context("ENGINE_VERSION marker missing from cache entry")?;
    if !meta.is_file() {
        anyhow::bail!(
            "ENGINE_VERSION marker at {} is not a regular file; demoting cache entry to MISS",
            path.display()
        );
    }
    let bytes = fs::read(&path).context("ENGINE_VERSION marker missing from cache entry")?;
    let label = String::from_utf8_lossy(&bytes).trim().to_string();
    Ok(label)
}

/// Length of the file at `path` (0 when unreadable; the size-pin comparison
/// then demotes the entry).
fn library_len(path: &Path) -> u64 {
    fs::metadata(path).map(|m| m.len()).unwrap_or(0)
}

/// Sidecar pin path for a cached dylib: `{dylib}.size`.
fn library_size_pin_path(binary_path: &Path) -> PathBuf {
    let mut name = binary_path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    name.push_str(".size");
    binary_path.with_file_name(name)
}

/// Record the persisted dylib's byte length in its `.size` sidecar (atomic).
/// AD1 truncation court: `lipo -archs` inspects only the Mach-O header, so a
/// truncated dylib still "verifies"; the length pin closes that hole.
fn stage_library_size_pin(binary_path: &Path) -> Result<()> {
    let pin = format!("{}\n", library_len(binary_path));
    atomic_write_bytes(&library_size_pin_path(binary_path), pin.as_bytes())
        .context("Failed to write library size pin")
}

/// `Some(recorded_len)` when a well-formed `.size` pin exists, else `None`.
fn read_library_size_pin(binary_path: &Path) -> Option<u64> {
    let bytes = fs::read(library_size_pin_path(binary_path)).ok()?;
    let text = String::from_utf8_lossy(&bytes);
    text.trim().parse::<u64>().ok()
}

/// Write `bytes` to `dest` only if they carry the labeled slice: a
/// mismatched or malformed payload is removed again, so a bad download can
/// never poison the versioned cache for later runs.
fn persist_verified_for(
    os: &str,
    bytes: &[u8],
    dest: &Path,
    arch: &str,
    min_bytes: u64,
) -> Result<()> {
    atomic_write_bytes(dest, bytes).context("Failed to write downloaded binary")?;
    if let Err(e) = verify_library_slice_for(os, dest, arch, min_bytes) {
        let _ = fs::remove_file(dest);
        return Err(e.context("refusing to cache downloaded binary"));
    }
    if let Some(versioned_cache) = dest.parent() {
        stage_library_size_pin(dest)?;
        stage_engine_marker(versioned_cache)?;
    }
    Ok(())
}

fn get_cache_dir() -> Result<PathBuf> {
    let home = env::var("HOME").context("HOME environment variable not set")?;

    let cache_dir = Path::new(&home).join(CACHE_DIR).join("cache");
    fs::create_dir_all(&cache_dir).context("Failed to create cache directory")?;

    Ok(cache_dir)
}

/// Get the expected binary path for the given architecture
fn get_binary_path(cache_dir: &Path, arch: &str) -> PathBuf {
    cache_dir.join(format!("{}_{}.{}", BINARY_NAME, arch, library_extension()))
}

/// Download prebuilt binary from GitHub Release
fn download_from_github_release(cache_dir: &Path, arch: &str) -> Result<PathBuf> {
    let binary_path = get_binary_path(cache_dir, arch);
    // Asset law (TR7): the URL must request the platform-correct asset name —
    // libduckdb_{arch}.dylib on macOS, libduckdb_{arch}.so on Linux — matching
    // both the published release assets and get_binary_path's cache name. See
    // release_asset_name_for: no release has been published yet, so ALL
    // downloads 404 today and the local-compile fallback runs.
    let url = format!(
        "https://github.com/seanchatmangpt/frozen-duckdb/releases/download/v{}/{}",
        VERSION,
        release_asset_name(arch)
    );

    info!("Downloading from: {}", url);

    // Create cache directory
    fs::create_dir_all(cache_dir).context("Failed to create cache directory")?;

    // Download the binary
    let response =
        reqwest::blocking::get(&url).context("Failed to download binary from GitHub Release")?;

    if !response.status().is_success() {
        anyhow::bail!("HTTP error: {}", response.status());
    }

    let content = response.bytes().context("Failed to read response body")?;

    persist_verified_for(
        target_os_name(),
        &content,
        &binary_path,
        arch,
        MIN_ENGINE_DYLIB_BYTES,
    )?;

    // Make binary executable on Unix systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&binary_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&binary_path, perms)?;
    }

    debug!("Downloaded binary to: {}", binary_path.display());
    Ok(binary_path)
}

/// Compile DuckDB locally as fallback
fn compile_duckdb_locally(cache_dir: &Path, arch: &str) -> Result<PathBuf> {
    info!("Compiling DuckDB locally for {}...", arch);
    check_local_compile_target(target_os_name(), arch, &host_architecture()?)?;

    // Create cache directory
    fs::create_dir_all(cache_dir).context("Failed to create cache directory")?;

    // Create temporary directory for compilation
    let temp_dir = tempfile::tempdir().context("Failed to create temporary directory")?;

    let temp_path = temp_dir.path();

    // Clone DuckDB source
    info!("Cloning DuckDB source...");
    let duckdb_dir = temp_path.join("duckdb");

    // Dev builds clone the default branch, then check out the load-bearing
    // SOURCE_PIN commit. Nothing here is derived from VERSION (which is
    // "2.0.0-dev"); the release-download path stays keyed to VERSION.
    let clone_out = Command::new("git")
        .args(["clone", "https://github.com/duckdb/duckdb.git"])
        .arg(&duckdb_dir)
        .current_dir(temp_path)
        .output()
        .context("Failed to clone DuckDB repository")?;
    if !clone_out.status.success() {
        anyhow::bail!(
            "DuckDB git clone failed (exit {:?}):\n{}",
            clone_out.status.code(),
            String::from_utf8_lossy(&clone_out.stderr)
        );
    }

    let checkout_out = Command::new("git")
        .args(["-C"])
        .arg(&duckdb_dir)
        .args(["checkout", SOURCE_PIN])
        .output()
        .context("Failed to run git checkout of pinned DuckDB commit")?;
    if !checkout_out.status.success() {
        anyhow::bail!(
            "Failed to check out pinned DuckDB commit {} (exit {:?}):\n{}",
            SOURCE_PIN,
            checkout_out.status.code(),
            String::from_utf8_lossy(&checkout_out.stderr)
        );
    }
    info!("Checked out DuckDB at pinned commit {}", SOURCE_PIN);

    // Build DuckDB with all features
    info!("Building DuckDB with all features...");
    let build_dir = duckdb_dir.join("build");
    fs::create_dir_all(&build_dir).context("Failed to create build directory")?;

    // Configure with CMake - enable all extensions.
    // TR7/TR2 coordination: the build-binaries workflow (TR2) exports
    // CMAKE_OSX_ARCHITECTURES per matrix arch; if the operator/workflow set
    // it in the environment, pass it through to cmake so a local compile can
    // target a specific macOS arch (or arch list) instead of the host
    // default. The flag is only meaningful for Apple targets; cmake ignores
    // it elsewhere.
    // CMake extension law for DuckDB v1.5.x, verified by the wave-3 release
    // rehearsal (ticket TR2): BUILD_EXTENSIONS is an extension-NAME list, not a
    // boolean — "-DBUILD_EXTENSIONS=ON" makes cmake try to load a literal "ON"
    // extension and configure fails. visualizer/tpce are out-of-tree in 1.5.x;
    // excel needs minizip-ng on the build host. Static in-tree set proven to
    // configure + build:
    let mut configure = Command::new("cmake");
    configure
        .args([
            "..",
            "-DCMAKE_BUILD_TYPE=Release",
            "-DBUILD_EXTENSIONS=parquet;json;icu;httpfs;tpch;tpcds;fts;inet;sqlsmith",
            "-DBUILD_JEMALLOC=ON",
            "-DBUILD_AUTOLOAD=ON",
        ])
        .current_dir(&build_dir);
    let osx_env = env::var("CMAKE_OSX_ARCHITECTURES").ok();
    if let Some(osx_archs) = cmake_osx_architectures_for(target_os_name(), osx_env.as_deref(), arch)
    {
        info!("Configuring with CMAKE_OSX_ARCHITECTURES={}", osx_archs);
        configure.arg(format!("-DCMAKE_OSX_ARCHITECTURES={}", osx_archs));
    }
    // Fail loudly with the captured tool output instead of discarding it and
    // dying later in find_built_library with no diagnostics.
    let configure_out = configure
        .output()
        .context("Failed to configure DuckDB with CMake")?;
    if !configure_out.status.success() {
        anyhow::bail!(
            "DuckDB cmake configure failed (exit {:?}):\n{}",
            configure_out.status.code(),
            String::from_utf8_lossy(&configure_out.stderr)
        );
    }

    // Build with all available cores (use 4 as default)
    let make_out = Command::new("make")
        .args(["-j4"])
        .current_dir(&build_dir)
        .output()
        .context("Failed to build DuckDB")?;
    if !make_out.status.success() {
        anyhow::bail!(
            "DuckDB make failed (exit {:?}):\n{}",
            make_out.status.code(),
            String::from_utf8_lossy(&make_out.stderr)
        );
    }

    // Find the built library
    let built_lib = find_built_library(&build_dir, arch).context("Failed to find built library")?;
    verify_library_slice(&built_lib, arch)?;

    // Copy library to cache directory with proper name
    let binary_path = get_binary_path(cache_dir, arch);
    atomic_copy(&built_lib, &binary_path).context("Failed to copy built library to cache")?;

    // Also copy header files for FFI bindings generation
    let headers_dir = cache_dir.join("duckdb");
    fs::create_dir_all(&headers_dir)?;

    // Copy DuckDB headers
    let duckdb_headers = [
        duckdb_dir.join("src").join("include").join("duckdb.h"),
        duckdb_dir.join("src").join("include").join("duckdb.hpp"),
    ];

    for header in &duckdb_headers {
        if header.exists() {
            let dest = headers_dir.join(header.file_name().unwrap());
            atomic_copy(header, &dest)?;
            info!("Copied header: {}", header.display());
        }
    }

    info!("Compiled DuckDB binary to: {}", binary_path.display());
    stage_library_size_pin(&binary_path)?;
    stage_engine_marker(cache_dir)?;
    Ok(binary_path)
}

/// Find the built library in the build directory
fn find_built_library(build_dir: &Path, _arch: &str) -> Result<PathBuf> {
    // Look for the main DuckDB library - check multiple possible locations
    let possible_paths = [
        // DuckDB v1.5.x cmake output (verified by the TR2 rehearsal: make -j4
        // places the artifact under build/duckdb/)
        build_dir.join("duckdb").join("libduckdb.dylib"),
        build_dir.join("duckdb").join("libduckdb.so"),
        build_dir.join("duckdb").join("libduckdb.dll"),
        // Release build location (older layouts)
        build_dir.join("src").join("libduckdb.dylib"),
        build_dir.join("src").join("libduckdb.so"),
        build_dir.join("src").join("libduckdb.dll"),
        // Alternative locations
        build_dir.join("libduckdb.dylib"),
        build_dir.join("libduckdb.so"),
        build_dir.join("libduckdb.dll"),
        // Sometimes it's in a subdirectory
        build_dir
            .join("src")
            .join("Release")
            .join("libduckdb.dylib"),
        build_dir.join("src").join("Release").join("libduckdb.so"),
        build_dir.join("src").join("Release").join("libduckdb.dll"),
    ];

    for path in &possible_paths {
        if path.exists() {
            info!("Found built library: {}", path.display());
            return Ok(path.clone());
        }
    }

    // If we can't find it, let's list what actually exists in the build directory
    if let Ok(entries) = fs::read_dir(build_dir) {
        let found_files: Vec<_> = entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .collect();

        if !found_files.is_empty() {
            info!("Files in build directory: {:?}", found_files);
        }
    }

    anyhow::bail!(
        "Could not find built DuckDB library in {:?}. Tried: {:?}",
        build_dir,
        possible_paths
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_architecture() {
        let arch = detect_architecture().unwrap();
        assert!(arch == "x86_64" || arch == "arm64");
    }

    // TR7 falsifier: reverting to uname-only detection makes this fail — the
    // pre-fix code answered "arm64" for the (Some("x86_64"), "arm64") host.
    #[test]
    fn tr7_explicit_slice_target_outranks_host() {
        // Apple-silicon CI host targeting the x86_64 slice (the exact TR7 case).
        assert_eq!(
            resolve_architecture(Some("x86_64"), "arm64").unwrap(),
            "x86_64"
        );
        // Symmetrically, an x86_64 host may target arm64.
        assert_eq!(
            resolve_architecture(Some("arm64"), "x86_64").unwrap(),
            "arm64"
        );
        assert_eq!(
            resolve_architecture(Some("aarch64"), "x86_64").unwrap(),
            "arm64"
        );
    }

    #[test]
    fn tr7_multi_arch_list_and_empty_fall_back_to_host() {
        // A universal-binary list is not a single-slice target: host decides.
        assert_eq!(
            resolve_architecture(Some("arm64;x86_64"), "arm64").unwrap(),
            "arm64"
        );
        assert_eq!(resolve_architecture(Some(""), "x86_64").unwrap(), "x86_64");
        assert_eq!(resolve_architecture(None, "arm64").unwrap(), "arm64");
        // Unset env + unresolvable host fails closed.
        assert!(resolve_architecture(None, "sparc").is_err());
    }

    #[test]
    fn test_get_cache_dir() {
        let cache_dir = get_cache_dir().unwrap();
        assert!(cache_dir.to_string_lossy().contains(CACHE_DIR));
    }

    #[test]
    fn test_get_binary_path() {
        let cache_dir = Path::new("/tmp/test");
        let arch = "x86_64";
        let path = get_binary_path(cache_dir, arch);

        if cfg!(target_os = "macos") {
            assert!(path.to_string_lossy().ends_with("libduckdb_x86_64.dylib"));
        } else if cfg!(target_os = "linux") {
            assert!(path.to_string_lossy().ends_with("libduckdb_x86_64.so"));
        }
    }

    #[test]
    fn test_library_extension_for_platforms() {
        // Asset law (TR7): dylib on macOS, so on Linux, dll on Windows
        // (local-compile fallback only), so for unknown platforms.
        assert_eq!(library_extension_for("macos"), "dylib");
        assert_eq!(library_extension_for("linux"), "so");
        assert_eq!(library_extension_for("windows"), "dll");
        assert_eq!(library_extension_for("other"), "so");
    }

    #[test]
    fn test_release_asset_name_per_platform() {
        assert_eq!(
            release_asset_name_for("macos", "arm64"),
            "libduckdb_arm64.dylib"
        );
        assert_eq!(
            release_asset_name_for("macos", "x86_64"),
            "libduckdb_x86_64.dylib"
        );
        assert_eq!(
            release_asset_name_for("linux", "arm64"),
            "libduckdb_arm64.so"
        );
        assert_eq!(
            release_asset_name_for("linux", "x86_64"),
            "libduckdb_x86_64.so"
        );
    }

    #[test]
    fn test_release_asset_name_matches_host_binary_path() {
        // The download URL asset and the cached binary name must carry the
        // same platform extension on the host, or download would write a file
        // get_binary_path would never find.
        for arch in ["x86_64", "arm64"] {
            let asset = release_asset_name(arch);
            let binary = get_binary_path(Path::new("/tmp/test"), arch);
            let binary_name = binary.file_name().unwrap().to_string_lossy();
            assert_eq!(asset, binary_name);
        }
    }

    #[test]
    fn test_ensure_link_name_platform_extension() {
        // Cache normalization must produce libduckdb.dylib on macOS and
        // libduckdb.so on Linux, and be idempotent.
        let tmp = tempfile::tempdir().unwrap();
        let cache = tmp.path();
        let arch = "arm64";
        let target = get_binary_path(cache, arch);
        fs::write(&target, b"fake dylib bytes").unwrap();

        ensure_link_name(cache, arch).unwrap();
        let link = cache.join(format!("{}.{}", BINARY_NAME, library_extension()));
        assert!(link.exists(), "link name missing: {}", link.display());

        ensure_link_name(cache, arch).unwrap(); // second call must not fail
    }

    // ---- TR7 hardening: boundary / adversarial / falsifier tests ----------

    fn some(v: &str) -> Option<String> {
        Some(v.to_string())
    }

    #[test]
    fn tr7_precedence_invalid_cmake_value_does_not_shadow_valid_arch() {
        // An unrecognized CMAKE_OSX_ARCHITECTURES must not hide ARCH (the
        // pre-hardening `.or_else` only fell through on UNSET).
        assert_eq!(
            resolve_target_architecture(&[some("ppc"), some("x86_64")], "arm64").unwrap(),
            "x86_64"
        );
        assert_eq!(
            resolve_target_architecture(&[some(""), some("arm64")], "x86_64").unwrap(),
            "arm64"
        );
        // Universal list + explicit single slice in ARCH: ARCH names the label.
        assert_eq!(
            resolve_target_architecture(&[some("arm64;x86_64"), some("x86_64")], "arm64").unwrap(),
            "x86_64"
        );
        // CMAKE_OSX_ARCHITECTURES outranks ARCH when both are valid slices.
        assert_eq!(
            resolve_target_architecture(&[some("arm64"), some("x86_64")], "x86_64").unwrap(),
            "arm64"
        );
        // Nothing explicit: host decides; unresolvable host fails closed.
        assert_eq!(
            resolve_target_architecture(&[None, None], "aarch64").unwrap(),
            "arm64"
        );
        assert!(resolve_target_architecture(&[None, some("amd64")], "riscv64").is_err());
    }

    #[test]
    fn tr7_malformed_explicit_targets_are_not_adopted() {
        // Whitespace is tolerated (GITHUB_ENV lines), everything else that is
        // not exactly one supported slice falls through to the host.
        assert_eq!(
            resolve_architecture(Some(" x86_64\n"), "arm64").unwrap(),
            "x86_64"
        );
        for bad in [
            "X86_64",
            "amd64",
            "x86-64",
            "arm64e",
            "arm64 x86_64",
            ";",
            "x86_64;",
        ] {
            assert_eq!(
                resolve_architecture(Some(bad), "arm64").unwrap(),
                "arm64",
                "malformed target {:?} must not be adopted",
                bad
            );
        }
        let err = resolve_architecture(None, " sparc\n")
            .unwrap_err()
            .to_string();
        assert!(err.contains("Unsupported architecture: sparc"), "{}", err);
    }

    #[test]
    fn tr7_cmake_flag_pins_the_label_on_macos() {
        // Label from ARCH only (no CMAKE_OSX_ARCHITECTURES): cmake must be
        // told the same slice, or clang emits the host slice under it.
        assert_eq!(
            cmake_osx_architectures_for("macos", None, "x86_64").as_deref(),
            Some("x86_64")
        );
        assert_eq!(
            cmake_osx_architectures_for("macos", Some("  "), "arm64").as_deref(),
            Some("arm64")
        );
        // Operator-set value (possibly universal) passes through verbatim.
        assert_eq!(
            cmake_osx_architectures_for("macos", Some(" arm64;x86_64 "), "arm64").as_deref(),
            Some("arm64;x86_64")
        );
        // Off macOS no flag is invented.
        assert_eq!(cmake_osx_architectures_for("linux", None, "x86_64"), None);
    }

    #[test]
    fn tr7_cross_arch_local_compile_refused_off_macos() {
        let err = check_local_compile_target("linux", "arm64", "x86_64\n")
            .unwrap_err()
            .to_string();
        assert!(err.contains("target arm64 but host x86_64"), "{}", err);
        assert!(check_local_compile_target("linux", "arm64", "aarch64").is_ok());
        assert!(check_local_compile_target("linux", "x86_64", "x86_64").is_ok());
        assert!(check_local_compile_target("macos", "x86_64", "arm64").is_ok());
    }

    #[test]
    fn tr7_lipo_output_parsing_is_token_exact() {
        assert!(lipo_archs_contain("x86_64 arm64\n", "arm64"));
        assert!(lipo_archs_contain("x86_64 arm64\n", "x86_64"));
        assert!(!lipo_archs_contain("arm64e\n", "arm64"));
        assert!(!lipo_archs_contain("x86_64h\n", "x86_64"));
        assert!(!lipo_archs_contain("", "arm64"));
    }

    #[test]
    fn tr7_slice_check_off_macos_is_unverifiable_not_admitted() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("libduckdb_x86_64.so");
        fs::write(&path, b"not inspected").unwrap();
        match verify_library_slice_for("linux", &path, "x86_64", 1).unwrap() {
            SliceCheck::Unverifiable(why) => assert!(why.contains("macOS-only"), "{}", why),
            SliceCheck::Verified => panic!("non-macOS check must not claim Verified"),
        }
    }

    /// Compile a real one-symbol Mach-O dylib for `archs` with the host C
    /// toolchain. `None` (printed as a visible SKIP by callers) when cc
    /// cannot target those slices on this machine.
    #[cfg(target_os = "macos")]
    fn real_dylib(dir: &Path, name: &str, archs: &[&str]) -> Option<PathBuf> {
        let src = dir.join("tr7_probe.c");
        fs::write(&src, "int tr7_probe(void) { return 7; }\n").ok()?;
        let out = dir.join(name);
        let mut cc = Command::new("cc");
        for a in archs {
            cc.args(["-arch", a]);
        }
        let status = cc
            .arg("-dynamiclib")
            .arg("-o")
            .arg(&out)
            .arg(&src)
            .status()
            .ok()?;
        status.success().then_some(out)
    }

    /// Test-local real dylib padded above MIN_ENGINE_DYLIB_BYTES (10MiB) via
    /// a retained data section, so it passes the production slice check —
    /// used by the DT1 trusted-hit court.
    #[cfg(target_os = "macos")]
    fn real_dylib_above_production_floor(dir: &Path, name: &str) -> Option<PathBuf> {
        let src = dir.join("dt1_pad.c");
        fs::write(
            &src,
            "__attribute__((used)) char dt1_pad[12 * 1024 * 1024] = {1};\n\
             int dt1_probe(void) { return dt1_pad[0]; }\n",
        )
        .ok()?;
        let out = dir.join(name);
        let status = Command::new("cc")
            .args(["-arch", "arm64", "-dynamiclib", "-o"])
            .arg(&out)
            .arg(&src)
            .status()
            .ok()?;
        status.success().then_some(out)
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn tr7_real_mach_o_slice_match_and_mismatch() {
        let tmp = tempfile::tempdir().unwrap();
        let Some(arm) = real_dylib(tmp.path(), "a.dylib", &["arm64"]) else {
            eprintln!("SKIP tr7_real_mach_o_slice_match_and_mismatch: cc cannot emit arm64");
            return;
        };
        assert_eq!(
            verify_library_slice_for("macos", &arm, "arm64", 1).unwrap(),
            SliceCheck::Verified
        );
        let err = verify_library_slice_for("macos", &arm, "x86_64", 1)
            .unwrap_err()
            .to_string();
        assert!(err.contains("TR7 slice mismatch"), "{}", err);
        assert!(
            err.contains("labeled x86_64") && err.contains("arm64"),
            "{}",
            err
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn tr7_real_universal_dylib_satisfies_either_label() {
        let tmp = tempfile::tempdir().unwrap();
        let Some(fat) = real_dylib(tmp.path(), "u.dylib", &["arm64", "x86_64"]) else {
            eprintln!("SKIP tr7_real_universal_dylib_satisfies_either_label: no x86_64 SDK slice");
            return;
        };
        for arch in ["arm64", "x86_64"] {
            assert_eq!(
                verify_library_slice_for("macos", &fat, arch, 1).unwrap(),
                SliceCheck::Verified
            );
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn tr7_malformed_libraries_fail_closed() {
        let tmp = tempfile::tempdir().unwrap();
        let text = tmp.path().join("libduckdb_arm64.dylib");
        fs::write(&text, b"<html>404 Not Found</html>").unwrap();
        let empty = tmp.path().join("empty.dylib");
        fs::write(&empty, b"").unwrap();
        let missing = tmp.path().join("missing.dylib");
        for p in [&text, &empty, &missing] {
            let r = verify_library_slice_for("macos", p, "arm64", 1);
            match r {
                Err(e) => assert!(e.to_string().contains("not a readable Mach-O"), "{}", e),
                Ok(SliceCheck::Unverifiable(why)) => {
                    eprintln!("SKIP malformed check for {}: {}", p.display(), why)
                }
                Ok(SliceCheck::Verified) => panic!("{} must not verify", p.display()),
            }
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn tr7_stale_pre_fix_cache_layout_is_detected() {
        // Reproduce the exact pre-fix TR7 artifact (historical v1.5.5 leg): x86_64 bytes placed at
        // v{VERSION}-arm64/libduckdb_arm64.dylib. The cached-path check must
        // refuse it rather than link a foreign slice.
        let tmp = tempfile::tempdir().unwrap();
        let versioned = tmp.path().join(format!("v{}-arm64", VERSION));
        fs::create_dir_all(&versioned).unwrap();
        let Some(x86) = real_dylib(tmp.path(), "x.dylib", &["x86_64"]) else {
            eprintln!("SKIP tr7_stale_pre_fix_cache_layout_is_detected: no x86_64 SDK slice");
            return;
        };
        let poisoned = get_binary_path(&versioned, "arm64");
        fs::copy(&x86, &poisoned).unwrap();
        assert!(verify_library_slice_for("macos", &poisoned, "arm64", 1).is_err());
        assert_eq!(
            verify_library_slice_for("macos", &poisoned, "x86_64", 1).unwrap(),
            SliceCheck::Verified
        );
    }

    #[test]
    fn gap9_atomic_persist_renames_through_tmp_sibling_without_residue() {
        // X3/T2 court: cache writes go through a .tmp-<pid>-<name> sibling and
        // an atomic fs::rename — the final path is only ever a complete file,
        // and no temp sibling survives the write.
        let tmp = tempfile::tempdir().unwrap();
        let dest = tmp.path().join("libduckdb_arm64.dylib");

        atomic_write_bytes(&dest, b"old-complete-payload").unwrap();
        assert_eq!(fs::read(&dest).unwrap(), b"old-complete-payload");

        atomic_write_bytes(&dest, b"new-complete-payload").unwrap();
        assert_eq!(fs::read(&dest).unwrap(), b"new-complete-payload");

        let src = tmp.path().join("src.dylib");
        fs::write(&src, b"copied-complete-payload").unwrap();
        atomic_copy(&src, &dest).unwrap();
        assert_eq!(fs::read(&dest).unwrap(), b"copied-complete-payload");

        let residue: Vec<_> = fs::read_dir(tmp.path())
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|n| n.starts_with(".tmp-"))
            .collect();
        assert!(residue.is_empty(), "leftover temp siblings: {:?}", residue);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn tr7_mismatched_download_never_reaches_the_cache() {
        let tmp = tempfile::tempdir().unwrap();
        let Some(arm) = real_dylib(tmp.path(), "a.dylib", &["arm64"]) else {
            eprintln!("SKIP tr7_mismatched_download_never_reaches_the_cache: no arm64 cc");
            return;
        };
        let bytes = fs::read(&arm).unwrap();
        let dest = get_binary_path(tmp.path(), "x86_64");
        let err = persist_verified_for("macos", &bytes, &dest, "x86_64", 1).unwrap_err();
        assert!(
            format!("{:#}", err).contains("TR7 slice mismatch"),
            "{:#}",
            err
        );
        assert!(!dest.exists(), "mislabeled download left in cache");
        // Duplicate delivery of a GOOD payload is idempotent.
        let good = get_binary_path(tmp.path(), "arm64");
        persist_verified_for("macos", &bytes, &good, "arm64", 1).unwrap();
        persist_verified_for("macos", &bytes, &good, "arm64", 1).unwrap();
        assert_eq!(fs::read(&good).unwrap(), bytes);
        // A 404 body delivered as the asset is refused and removed too.
        let html = get_binary_path(tmp.path(), "arm64-html");
        assert!(persist_verified_for("macos", b"<html>404</html>", &html, "arm64", 1).is_err());
        assert!(!html.exists());
    }

    /// Structural anti-poison: a cache entry whose ENGINE_VERSION marker is
    /// absent or carries a foreign label must read as a cache MISS (restage),
    /// and a successful persist stages exactly `v{VERSION}`.
    #[test]
    fn engine_marker_mismatch_or_absence_demotes_cache_to_miss() {
        let tmp = tempfile::tempdir().unwrap();
        let expected = expected_engine_marker();
        assert_eq!(expected, format!("v{}", VERSION));

        // Absent marker: reads as MISS (error), never as a trusted hit.
        let err = read_engine_marker(tmp.path()).unwrap_err();
        assert!(format!("{:#}", err).contains("missing from cache entry"));

        // Wrong label (a foreign builder version staged this cache): MISS,
        // and the demotion message carries the foreign marker.
        let foreign = format!("v{}.poison", VERSION);
        fs::write(engine_marker_path(tmp.path()), &foreign).unwrap();
        let found = read_engine_marker(tmp.path()).unwrap();
        assert_ne!(found, expected);
        assert_eq!(
            found, foreign,
            "mismatched marker must surface its foreign label for the restage warn"
        );

        // A successful persist stages the exact pinned marker.
        stage_engine_marker(tmp.path()).unwrap();
        let bytes = fs::read(engine_marker_path(tmp.path())).unwrap();
        assert_eq!(bytes, expected.clone().into_bytes());
        assert_eq!(read_engine_marker(tmp.path()).unwrap(), expected);

        // Round-trip through a real persist site too: the download persist
        // path writes the marker after the slice check passes.
        #[cfg(target_os = "macos")]
        if let Some(arm) = real_dylib(tmp.path(), "marker.dylib", &["arm64"]) {
            let bytes = fs::read(&arm).unwrap();
            let dest = get_binary_path(tmp.path(), "arm64");
            persist_verified_for("macos", &bytes, &dest, "arm64", 1).unwrap();
            assert_eq!(
                read_engine_marker(tmp.path()).unwrap(),
                expected,
                "persist_verified_for must stage ENGINE_VERSION after a verified persist"
            );
        } else {
            eprintln!("SKIP marker round-trip via real dylib: no arm64 cc");
        }
    }

    #[test]
    fn tr7_workflow_exports_both_target_vars_and_full_error_chain() {
        // Guard the workflow half of TR7: per-leg exports feed detection and
        // cmake; {:#} surfaces the captured cmake/make stderr.
        let wf = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../.github/workflows/build-binaries.yml");
        let Ok(text) = fs::read_to_string(&wf) else {
            eprintln!(
                "SKIP workflow guard: {} absent (packaged crate)",
                wf.display()
            );
            return;
        };
        assert!(text.contains(r#"echo "ARCH=${{ matrix.arch }}" >> "$GITHUB_ENV""#));
        assert!(
            text.contains(r#"echo "CMAKE_OSX_ARCHITECTURES=${{ matrix.arch }}" >> "$GITHUB_ENV""#)
        );
        assert!(text.contains(r#"eprintln!("Failed to build mega-library: {:#}", e);"#));
        assert!(!text.contains(r#"eprintln!("Failed to build mega-library: {}", e);"#));
    }

    /// Deterministic timing bench with committed regression bounds (debug
    /// profile, loaded CI host => bounds are >=10x the measured means;
    /// measured means are recorded in the TR7 hardening commit message).
    #[test]
    fn tr7_bench_regression_bounds() {
        use std::time::Instant;
        const N_RESOLVE: u32 = 200_000;
        let inputs = [some("arm64;x86_64"), some("x86_64")];
        let t = Instant::now();
        for _ in 0..N_RESOLVE {
            let a = resolve_target_architecture(std::hint::black_box(&inputs), "arm64").unwrap();
            std::hint::black_box(a);
        }
        let resolve_ns = t.elapsed().as_nanos() / u128::from(N_RESOLVE);

        const N_DETECT: u32 = 20;
        let t = Instant::now();
        for _ in 0..N_DETECT {
            std::hint::black_box(detect_architecture().unwrap());
        }
        let detect_us = t.elapsed().as_micros() / u128::from(N_DETECT);
        eprintln!(
            "BENCH tr7 resolve_target_architecture={}ns/op detect_architecture={}us/op",
            resolve_ns, detect_us
        );
        assert!(
            resolve_ns < 20_000,
            "resolve regression: {}ns/op",
            resolve_ns
        );
        assert!(detect_us < 500_000, "detect regression: {}us/op", detect_us);

        #[cfg(target_os = "macos")]
        {
            let tmp = tempfile::tempdir().unwrap();
            if let Some(lib) = real_dylib(tmp.path(), "b.dylib", &["arm64"]) {
                const N_LIPO: u32 = 20;
                let t = Instant::now();
                for _ in 0..N_LIPO {
                    verify_library_slice_for("macos", &lib, "arm64", 1).unwrap();
                }
                let lipo_us = t.elapsed().as_micros() / u128::from(N_LIPO);
                eprintln!("BENCH tr7 verify_library_slice={}us/op", lipo_us);
                assert!(
                    lipo_us < 1_000_000,
                    "slice-check regression: {}us/op",
                    lipo_us
                );
            } else {
                eprintln!("SKIP slice-check bench: no arm64 cc");
            }
        }
    }

    // ---- AD1 concurrency/tamper courts (post-GAP9/C2 cache layer) --------

    /// Synthetic Mach-O-shaped payload: `0xfeedfacf` magic + per-writer tag,
    /// so a torn read is detectable by BOTH magic and length tail.
    fn poison_probe_payload(tag: u64, len: usize) -> Vec<u8> {
        assert!(len > 16);
        let mut v = vec![0u8; len];
        v[..8].copy_from_slice(&0xfeedfacf_u64.to_le_bytes());
        v[8..16].copy_from_slice(&tag.to_le_bytes());
        v[len - 8..].copy_from_slice(&(len as u64).to_le_bytes());
        v
    }

    /// Looks like a complete payload to the reader court: magic intact, tag
    /// matches one of the live writers, length tail equals the observed len.
    fn payload_is_complete(bytes: &[u8], tags: &[u64]) -> Result<(), String> {
        if bytes.len() < 16 {
            return Err(format!("too short: {} bytes", bytes.len()));
        }
        let magic = u64::from_le_bytes(bytes[..8].try_into().unwrap());
        if magic != 0xfeedfacf_u64 {
            return Err(format!("torn magic: {magic:#x}"));
        }
        let tag = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
        if !tags.contains(&tag) {
            return Err(format!("foreign tag {tag}"));
        }
        let tail = u64::from_le_bytes(bytes[bytes.len() - 8..].try_into().unwrap());
        if tail as usize != bytes.len() {
            return Err(format!(
                "torn length: tail says {} but read {} bytes",
                tail,
                bytes.len()
            ));
        }
        Ok(())
    }

    /// AD1 attack 1: 3 writer threads racing the SAME cache destination with
    /// an always-on reader court. Falsifier for the pre-fix `.tmp-<pid>-<name>`
    /// staging file: two in-process threads shared one tmp path, so one
    /// thread's rename could publish the other's mid-write (partial) bytes at
    /// the final path — a poisoned serve under a valid marker.
    #[test]
    fn ad1_race_three_writers_reader_never_observes_a_partial_file() {
        let tmp = tempfile::tempdir().unwrap();
        let dest = tmp.path().join("libduckdb_arm64.dylib");
        let tags = [0xA_u64, 0xB_u64, 0xC_u64];
        let lens = [64 * 1024, 96 * 1024, 32 * 1024];
        let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let violations = std::sync::Arc::new(std::sync::Mutex::new(Vec::<String>::new()));

        // Reader court: continuously stat+read checking Mach-O magic + shape.
        let reader = {
            let dest = dest.clone();
            let stop = stop.clone();
            let violations = violations.clone();
            let tags = tags.to_vec();
            std::thread::spawn(move || {
                while !stop.load(std::sync::atomic::Ordering::Relaxed) {
                    if let Ok(bytes) = fs::read(&dest) {
                        if let Err(why) = payload_is_complete(&bytes, &tags) {
                            violations.lock().unwrap().push(why);
                        }
                    }
                }
            })
        };

        let mut writers = Vec::new();
        for (&tag, &len) in tags.iter().zip(&lens) {
            let dest = dest.clone();
            let bytes = poison_probe_payload(tag, len);
            writers.push(std::thread::spawn(move || {
                for _ in 0..40 {
                    atomic_write_bytes(&dest, &bytes).unwrap();
                    std::thread::yield_now();
                }
            }));
        }
        for w in writers {
            w.join().unwrap();
        }
        stop.store(true, std::sync::atomic::Ordering::Relaxed);
        reader.join().unwrap();

        let v = violations.lock().unwrap();
        assert!(
            v.is_empty(),
            "reader observed {} torn/partial files: {:?}",
            v.len(),
            &v[..v.len().min(5)]
        );
        // Final state: dest is exactly one writer's complete payload.
        let final_bytes = fs::read(&dest).unwrap();
        payload_is_complete(&final_bytes, &tags).unwrap();
        // Zero .tmp residue after the storm.
        let residue: Vec<_> = fs::read_dir(tmp.path())
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|n| n.starts_with(".tmp-"))
            .collect();
        assert!(residue.is_empty(), "leftover temp siblings: {residue:?}");
    }

    /// AD1 attack 1b: three marker+dylib persist sequences into ONE cache dir
    /// (the ensure_binary-style persist shape: dylib then marker) — the final
    /// marker must be exactly `v{VERSION}` and the dylib complete.
    #[test]
    fn ad1_three_concurrent_persist_sequences_leave_a_coherent_cache() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = tmp.path().to_path_buf();
        let dest = get_binary_path(&cache, "arm64");
        let tags = [1_u64, 2, 3];
        let mut handles = Vec::new();
        for &tag in &tags {
            let dest = dest.clone();
            let cache = cache.clone();
            handles.push(std::thread::spawn(move || {
                let bytes = poison_probe_payload(tag, 48 * 1024);
                atomic_write_bytes(&dest, &bytes).unwrap();
                stage_engine_marker(&cache).unwrap();
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
        assert_eq!(
            read_engine_marker(&cache).unwrap(),
            expected_engine_marker(),
            "final marker must be the builder's own pin"
        );
        payload_is_complete(&fs::read(&dest).unwrap(), &tags).unwrap();
        let residue: Vec<_> = fs::read_dir(&cache)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|n| n.starts_with(".tmp-"))
            .collect();
        assert!(residue.is_empty(), "leftover temp siblings: {residue:?}");
    }

    /// AD1 attack 2: tamper the ENGINE_VERSION marker in the window between
    /// the dylib write and the marker write. The (dylib, marker) pair may
    /// interleave any way it likes — a foreign or absent marker with a
    /// present dylib must demote to MISS, never a trusted serve.
    #[test]
    fn ad1_marker_tamper_midflight_demotes_to_miss_never_serves() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = tmp.path().to_path_buf();
        let dest = get_binary_path(&cache, "arm64");

        // Window state A: dylib written, marker not yet staged (crash/tamper).
        atomic_write_bytes(&dest, &poison_probe_payload(7, 8192)).unwrap();
        assert!(
            read_engine_marker(&cache).is_err(),
            "dylib without a marker must read as MISS, not a hit"
        );

        // Window state B: a tamperer swaps a foreign marker in.
        fs::write(engine_marker_path(&cache), b"v1.5.5.attacker").unwrap();
        let label = read_engine_marker(&cache).unwrap();
        assert_ne!(label, expected_engine_marker());
        assert_eq!(label, "v1.5.5.attacker");

        // The legitimate persist then completes: the tamper is overwritten
        // and the pair is coherent again.
        stage_engine_marker(&cache).unwrap();
        assert_eq!(
            read_engine_marker(&cache).unwrap(),
            expected_engine_marker()
        );

        // Persist-order law: a persist whose slice check FAILS never stages a
        // marker, so a poisoned download cannot mint a trusted pair.
        let unmarked = tmp.path().join("unmarked");
        fs::create_dir_all(&unmarked).unwrap();
        let bad_dest = get_binary_path(&unmarked, "arm64");
        assert!(persist_verified_for("macos", b"<html>404</html>", &bad_dest, "arm64", 1).is_err());
        assert!(!bad_dest.exists());
        assert!(
            read_engine_marker(&unmarked).is_err(),
            "no marker for a refused persist"
        );
    }

    /// AD1 attack 3 (TR7 hard): valid ENGINE_VERSION marker + truncated dylib
    /// must never be served as a cache hit — the slice check itself must
    /// HARD-FAIL, not rely on downstream size pins alone.
    ///
    /// COURT FINDING: `lipo -archs` reads only the Mach-O header/load
    /// commands, so a truncated dylib actually PASSES the slice check — the
    /// pre-fix code served a truncated library under a valid marker. The fix
    /// is structural, inside `verify_library_slice_for`: (1) a size floor
    /// (`min_bytes`; production = MIN_ENGINE_DYLIB_BYTES = 10MiB, real cached
    /// engine ~66MB) and (2) a Mach-O magic check on the first 4 bytes.
    /// A truncated real arm64 dylib carries a valid header, so only the size
    /// floor refuses it; a non-Mach-O payload is refused by the magic check.
    #[test]
    fn ad1_truncated_dylib_with_valid_marker_hard_fails() {
        let tmp = tempfile::tempdir().unwrap();

        // The hardened slice check must REFUSE a truncated real dylib whose
        // header is valid (pre-fix this verified via lipo -archs).
        #[cfg(target_os = "macos")]
        if let Some(arm) = real_dylib(tmp.path(), "t.dylib", &["arm64"]) {
            let full = fs::read(&arm).unwrap();
            assert!(full.len() > 16);
            let truncated = tmp.path().join("truncated.dylib");
            fs::write(&truncated, &full[..full.len() / 2]).unwrap();
            // Lipo still accepts the truncated header (the original finding),
            // so only a floor at the full size catches it: use the exact full
            // length as the floor and require a hard error.
            let err = verify_library_slice_for("macos", &truncated, "arm64", full.len() as u64)
                .unwrap_err();
            assert!(err.to_string().contains("engine size floor"), "{}", err);
            // The complete dylib passes the same floor.
            assert_eq!(
                verify_library_slice_for("macos", &arm, "arm64", full.len() as u64).unwrap(),
                SliceCheck::Verified
            );
            // Magic table: thin arm64/x86_64 (both byte orders) and fat
            // (both) are Mach-O; anything else is not.
            assert!(is_macho_magic(&[0xcf, 0xfa, 0xed, 0xfe]));
            assert!(is_macho_magic(&[0xfe, 0xed, 0xfa, 0xcf]));
            assert!(is_macho_magic(&[0xca, 0xfe, 0xba, 0xbe]));
            assert!(is_macho_magic(&[0xbe, 0xba, 0xfe, 0xca]));
            assert!(!is_macho_magic(b"PK\x03\x04"));
            assert!(!is_macho_magic(b"<htm"));
            assert!(!is_macho_magic(&[0x00, 0x00, 0x00, 0x00]));
            // A non-Mach-O payload still hard-fails the whole check even if
            // the size floor is met (defense in depth under lipo).
            let junk = tmp.path().join("junk.dylib");
            fs::write(&junk, b"MACHO-PAYLOAD-BUT-NOT-REALLY").unwrap();
            assert!(verify_library_slice_for("macos", &junk, "arm64", 1).is_err());
            // Malformed payloads (not truncations of a real dylib) still
            // hard-fail the slice check via lipo itself.
            let empty = tmp.path().join("empty.dylib");
            fs::write(&empty, b"").unwrap();
            assert!(verify_library_slice_for("macos", &empty, "arm64", 1).is_err());
        } else {
            eprintln!("SKIP ad1 slice-check half: no arm64 cc");
        }

        // The size-pin law: a persisted entry's pin matches; truncating the
        // dylib under the SAME valid marker breaks the pin pair -> None
        // pin / wrong pin, which the cache-hit path demotes to MISS.
        let cache = tmp.path().join("entry");
        fs::create_dir_all(&cache).unwrap();
        let dest = get_binary_path(&cache, "arm64");
        let full_payload = poison_probe_payload(9, 12_345);
        atomic_write_bytes(&dest, &full_payload).unwrap();
        stage_library_size_pin(&dest).unwrap();
        stage_engine_marker(&cache).unwrap();

        // Baseline: complete dylib + valid marker + matching pin = trusted.
        assert_eq!(
            read_library_size_pin(&dest),
            Some(library_len(&dest)),
            "complete persist must satisfy its own size pin"
        );

        // Truncation extreme 1: half the bytes (a real crash mid-write shape,
        // but through the FINAL path — e.g. a non-atomic writer landed here).
        let truncated_len = full_payload.len() / 2;
        fs::write(&dest, &full_payload[..truncated_len]).unwrap();
        assert_ne!(
            read_library_size_pin(&dest),
            Some(library_len(&dest)),
            "truncated dylib must fail its size pin (never served)"
        );

        // Truncation extreme 2: pin sidecar removed (crash before staging).
        let pin_path = library_size_pin_path(&dest);
        fs::remove_file(&pin_path).unwrap();
        assert_eq!(read_library_size_pin(&dest), None);
        fs::write(&dest, &full_payload).unwrap();
        assert_ne!(
            read_library_size_pin(&dest),
            Some(library_len(&dest)),
            "absent pin must demote even a complete dylib"
        );

        // Repersist restores the trusted pair.
        atomic_write_bytes(&dest, &full_payload).unwrap();
        stage_library_size_pin(&dest).unwrap();
        assert_eq!(read_library_size_pin(&dest), Some(library_len(&dest)));
        assert!(pin_path.exists());
    }

    // ---- DT1 adversarial review courts -----------------------------------

    /// DT1 fix court: a FAILED rename (here: destination path is a directory,
    /// so `fs::rename` gets EISDIR) must not strand the staged `.tmp-`
    /// sibling. The pre-fix code cleaned up on write failure only.
    #[test]
    fn dt1_failed_rename_leaves_no_tmp_sibling_residue() {
        let tmp = tempfile::tempdir().unwrap();
        // `dest` exists as a DIRECTORY: the tmp write succeeds, the rename
        // over it fails on every platform.
        let dest = tmp.path().join("blocked.dylib");
        fs::create_dir(&dest).unwrap();

        assert!(atomic_write_bytes(&dest, b"payload").is_err());
        assert!(atomic_copy(&tmp.path().join("missing-src"), &dest).is_err());

        let residue: Vec<_> = fs::read_dir(tmp.path())
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|n| n.starts_with(".tmp-"))
            .collect();
        assert!(
            residue.is_empty(),
            "rename failure left residue: {residue:?}"
        );
    }

    /// DT1 fix court: a size-pin mismatch (valid marker, truncated dylib) must
    /// demote to a REAL MISS — dylib and pin removed, `Ok(false)` — never
    /// `Ok(true)`/serve. Falsifier for the pre-fix `return Ok(binary_path)`
    /// that served the dylib it had just declared untrusted.
    #[cfg(target_os = "macos")]
    #[test]
    fn dt1_size_pin_mismatch_demotes_to_a_real_miss_never_serves() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = tmp.path().to_path_buf();
        let dest = get_binary_path(&cache, "arm64");
        let Some(arm) = real_dylib(tmp.path(), "p.dylib", &["arm64"]) else {
            eprintln!("SKIP dt1_size_pin_mismatch: no arm64 cc");
            return;
        };
        let full = fs::read(&arm).unwrap();

        // Trusted persist, then crash-shaped truncation of the dylib in place
        // (lipo still verifies the truncated header — the pin is the guard).
        atomic_write_bytes(&dest, &full).unwrap();
        stage_library_size_pin(&dest).unwrap();
        stage_engine_marker(&cache).unwrap();
        fs::write(&dest, &full[..full.len() / 2]).unwrap();

        let hit = accept_cached_entry(&cache, &dest, "arm64", 1).unwrap();
        assert!(!hit, "pin-mismatched entry must demote, never serve");
        assert!(
            !dest.exists(),
            "untrusted dylib must be removed, not served"
        );
        assert!(
            !library_size_pin_path(&dest).exists(),
            "untrusted pin must be removed with its dylib"
        );
    }

    /// DT1: the trusted-hit happy path through the refactored law — valid
    /// marker + matching pin + verifying slice = serve (`Ok(true)`), with the
    /// dylib left intact.
    #[cfg(target_os = "macos")]
    #[test]
    fn dt1_coherent_entry_is_a_trusted_hit() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = tmp.path().to_path_buf();
        let dest = get_binary_path(&cache, "arm64");
        let Some(arm) = real_dylib_above_production_floor(tmp.path(), "h.dylib") else {
            eprintln!("SKIP dt1_coherent_entry: no arm64 cc");
            return;
        };
        let full = fs::read(&arm).unwrap();
        atomic_write_bytes(&dest, &full).unwrap();
        stage_library_size_pin(&dest).unwrap();
        stage_engine_marker(&cache).unwrap();

        assert!(accept_cached_entry(&cache, &dest, "arm64", 1).unwrap());
        assert!(dest.exists());
    }

    /// DT1: absent marker with a present dylib (crash between the dylib
    /// rename and the marker write of the FIRST persist) is a MISS — the
    /// marker-after-dylib ordering means a valid marker can never pair with a
    /// dylib whose persist did not complete.
    #[test]
    fn dt1_crash_between_dylib_and_marker_is_a_miss() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = tmp.path().to_path_buf();
        let dest = get_binary_path(&cache, "arm64");
        atomic_write_bytes(&dest, &poison_probe_payload(9, 8192)).unwrap();
        // No marker, no pin staged.
        assert!(read_engine_marker(&cache).is_err());
        assert_eq!(read_library_size_pin(&dest), None);
        // Foreign marker (previous persist's marker tampered) also demotes.
        fs::write(engine_marker_path(&cache), b"v0.0.0-foreign").unwrap();
        assert_ne!(
            read_engine_marker(&cache).unwrap(),
            expected_engine_marker()
        );
    }

    /// DT1 (AD6 confirmation): CRLF and trailing-newline markers normalize;
    /// a marker that is a DIRECTORY or a SYMLINK demotes to MISS instead of
    /// being followed/trusted.
    #[cfg(unix)]
    #[test]
    fn dt1_marker_dir_and_symlink_demote_crlf_normalizes() {
        let tmp = tempfile::tempdir().unwrap();
        let expected = expected_engine_marker();

        // CRLF: trimmed to the exact pin.
        fs::write(engine_marker_path(tmp.path()), b"v2.0.0-dev\r\n").unwrap();
        assert_eq!(read_engine_marker(tmp.path()).unwrap(), expected);

        // Directory: unreadable as a file -> MISS.
        fs::remove_file(engine_marker_path(tmp.path())).unwrap();
        fs::create_dir(engine_marker_path(tmp.path())).unwrap();
        assert!(read_engine_marker(tmp.path()).is_err());

        // Symlink pointing at a file carrying the VALID pin: still demoted —
        // the marker must be a regular file inside the cache entry.
        let outside = tmp.path().join("outside.txt");
        fs::write(&outside, expected.as_bytes()).unwrap();
        fs::remove_dir_all(engine_marker_path(tmp.path())).unwrap();
        std::os::unix::fs::symlink(&outside, engine_marker_path(tmp.path())).unwrap();
        assert!(
            read_engine_marker(tmp.path()).is_err(),
            "symlinked marker must demote, not be followed"
        );
    }

    /// DT1: persist-order law across all persist sites — a marker must never
    /// be staged while its `.size` pin is absent (marker-last). Simulated
    /// crash window: dylib + marker present, pin missing => demote.
    #[test]
    fn dt1_valid_marker_with_missing_pin_demotes() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = tmp.path().to_path_buf();
        let dest = get_binary_path(&cache, "arm64");
        atomic_write_bytes(&dest, &poison_probe_payload(11, 8192)).unwrap();
        stage_engine_marker(&cache).unwrap();
        // pin never staged: the pre-AD1 cache shape. Must demote (real MISS
        // decision), not serve — and must NOT remove the dylib? No: the
        // demote law removes it; assert the decision is MISS.
        assert_eq!(
            accept_cached_entry(&cache, &dest, "arm64", 1).unwrap(),
            false,
            "marker without a pin is not a trusted hit"
        );
        assert!(!dest.exists(), "demoted entry is removed fail-closed");
    }

    // =====================================================================
    // AD6 full anti-poison siege court
    //
    // Drives the REAL decision layer (`accept_cached_entry`, the function
    // ensure_binary routes every cache hit through) against every poison
    // state the 13:22-class attacks can produce. Decision vocabulary:
    //   Ok(true)  = SERVE    (link the cached dylib)
    //   Ok(false) = RESTAGE  (demote to a real MISS; slot rebuilt)
    //   Err(_)    = FAIL     (fail-closed, build refused)
    //
    // Layering note (cases g2/h): the builder decision layer authenticates
    // LABEL/SIZE/SLICE — it cannot read a version string out of Mach-O
    // bytes. A dylib with the right slice and self-consistent pins but a
    // wrong engine version inside is SERVEd here by design; the version gate
    // lives downstream in the frozen-duckdb integration tests
    // (`SELECT version()`: crates/frozen-duckdb/tests/
    // core_functionality_tests.rs, dropin_compatibility_tests.rs). Case (h)
    // proves with a real dlopen probe that the version diverges while the
    // builder layers all pass, so the downstream gate is the operative one.
    // =====================================================================
    #[cfg(target_os = "macos")]
    #[test]
    fn ad6_siege_full_antipoison_stack_refuses_every_poison_state() {
        #[derive(Debug, PartialEq, Eq)]
        enum Decision {
            Serve,
            Restage,
            Fail,
        }
        fn decide(r: Result<bool>) -> Decision {
            match r {
                Ok(true) => Decision::Serve,
                Ok(false) => Decision::Restage,
                Err(_) => Decision::Fail,
            }
        }

        let tmp = tempfile::tempdir().unwrap();
        let scratch = tmp.path();
        let arch = "arm64";

        // Stage a genuinely lawful cache entry (real Mach-O, real marker,
        // real .size pin) through the production staging functions.
        let stage_lawful = |versioned: &Path, dylib: &Path| {
            fs::copy(dylib, get_binary_path(versioned, arch)).unwrap();
            stage_library_size_pin(&get_binary_path(versioned, arch)).unwrap();
            stage_engine_marker(versioned).unwrap();
        };

        // One real arm64 dylib stands in for "any dylib the labeled slot does
        // not belong to" (v1.5.5 bytes, wrong-version bytes: the marker/.size
        // law does not inspect versions, and the slice is right by design).
        let Some(good) = real_dylib(scratch, "good.dylib", &[arch]) else {
            eprintln!("[SKIP] ad6 siege: cc cannot emit arm64 dylib");
            return;
        };
        // A fat arm64+x86_64 binary (the universal-binary poison shape).
        let fat = real_dylib(scratch, "fat.dylib", &[arch, "x86_64"]);
        // A version-poison probe: valid arm64 Mach-O exporting
        // duckdb_library_version() == "v1.5.5".
        let probe_src = scratch.join("version_poison.c");
        fs::write(
            &probe_src,
            "const char *duckdb_library_version(void) { return \"v1.5.5\"; }\n",
        )
        .unwrap();
        let probe = Command::new("cc")
            .args(["-arch", arch, "-dynamiclib", "-o"])
            .arg(scratch.join("version_poison.dylib"))
            .arg(&probe_src)
            .status()
            .ok()
            .filter(|s| s.success())
            .map(|_| scratch.join("version_poison.dylib"));

        let versioned_for = |name: &str| {
            let v = scratch.join(name);
            fs::create_dir_all(&v).unwrap();
            v
        };

        // (name, expected decision, constructor). Constructors receive the
        // versioned cache dir to poison.
        let mut table: Vec<(&str, Decision, Box<dyn Fn(&Path)>)> = vec![
            (
                "control: real dylib + exact marker + pins -> SERVE",
                Decision::Serve,
                Box::new(|v: &Path| stage_lawful(v, &good)),
            ),
            // (a) the 13:22 attack: foreign dylib under a v2.0.0-dev label,
            // no AD1-era pins. Pre-siege this SERVED (the demote branch
            // returned Ok(binary_path)); the fix demotes to a real MISS.
            (
                "(a) v1.5.5 dylib + v2.0.0-dev marker (13:22 attack)",
                Decision::Restage,
                Box::new(|v: &Path| {
                    fs::copy(&good, get_binary_path(v, arch)).unwrap();
                    stage_engine_marker(v).unwrap();
                }),
            ),
            // (b) truncated dylib, valid marker, STALE pin (recorded before
            // truncation): the size-pin gate must fire.
            (
                "(b) truncated dylib + valid marker + stale .size",
                Decision::Restage,
                Box::new(|v: &Path| {
                    stage_lawful(v, &good);
                    let b = get_binary_path(v, arch);
                    let full = fs::read(&b).unwrap();
                    fs::write(&b, &full[..full.len() / 2]).unwrap();
                }),
            ),
            // (b2) truncation with an adversarially recomputed pin: pins pass,
            // the slice gate must FAIL closed (lipo on a mangled Mach-O).
            (
                "(b2) truncated dylib + recomputed pin -> slice FAIL",
                Decision::Fail,
                Box::new(|v: &Path| {
                    let b = get_binary_path(v, arch);
                    fs::copy(&good, &b).unwrap();
                    let full = fs::read(&b).unwrap();
                    fs::write(&b, &full[..16]).unwrap(); // mangle load commands
                    stage_library_size_pin(&b).unwrap();
                    stage_engine_marker(v).unwrap();
                }),
            ),
            // (c) valid dylib + MISSING marker.
            (
                "(c) valid dylib + missing marker",
                Decision::Restage,
                Box::new(|v: &Path| {
                    fs::copy(&good, get_binary_path(v, arch)).unwrap();
                    stage_library_size_pin(&get_binary_path(v, arch)).unwrap();
                    // marker intentionally absent
                }),
            ),
            // (d) foreign marker.
            (
                "(d) foreign marker v9.9.9",
                Decision::Restage,
                Box::new(|v: &Path| {
                    stage_lawful(v, &good);
                    fs::write(engine_marker_path(v), b"v9.9.9").unwrap();
                }),
            ),
            // (e) marker is a symlink to /dev/null.
            (
                "(e) marker symlink -> /dev/null",
                Decision::Restage,
                Box::new(|v: &Path| {
                    stage_lawful(v, &good);
                    fs::remove_file(engine_marker_path(v)).unwrap();
                    std::os::unix::fs::symlink("/dev/null", engine_marker_path(v)).unwrap();
                }),
            ),
            // (f) marker with trailing newline / CRLF: byte-exact label law.
            (
                "(f) marker `v{VERSION}\\n` (trailing newline)",
                Decision::Restage,
                Box::new(|v: &Path| {
                    stage_lawful(v, &good);
                    let mut m = expected_engine_marker().into_bytes();
                    m.push(b'\n');
                    fs::write(engine_marker_path(v), &m).unwrap();
                }),
            ),
            (
                "(f2) marker `v{VERSION}\\r\\n` (CRLF)",
                Decision::Restage,
                Box::new(|v: &Path| {
                    stage_lawful(v, &good);
                    let mut m = expected_engine_marker().into_bytes();
                    m.push(b'\r');
                    m.push(b'\n');
                    fs::write(engine_marker_path(v), &m).unwrap();
                }),
            ),
        ];

        // (g) universal-binary poison, two sub-cases when cc can emit fat.
        if let Some(fat_ref) = fat.as_ref() {
            let fat = fat_ref.clone();
            table.push((
                "(g1) universal fat dylib + dev marker, no pins",
                Decision::Restage,
                Box::new(move |v: &Path| {
                    fs::copy(&fat, get_binary_path(v, arch)).unwrap();
                    stage_engine_marker(v).unwrap();
                }),
            ));
            let fat = fat_ref.clone();
            table.push((
                "(g2) fat dylib + self-consistent pins (layering: version is downstream)",
                Decision::Serve,
                Box::new(move |v: &Path| stage_lawful(v, &fat)),
            ));
        } else {
            eprintln!("[SKIP] (g) rows: cc cannot emit arm64+x86_64 fat dylib");
        }

        // (h) valid Mach-O with the wrong engine version inside.
        if let Some(probe) = probe.as_ref() {
            table.push((
                "(h) wrong-version dylib, slice+pins valid (layering)",
                Decision::Serve,
                Box::new(move |v: &Path| stage_lawful(v, probe)),
            ));
        } else {
            eprintln!("[SKIP] (h) row: cc could not compile version-poison probe");
        }

        // (i) ENGINE_VERSION staged as a directory.
        table.push((
            "(i) ENGINE_VERSION is a directory",
            Decision::Restage,
            Box::new(|v: &Path| {
                stage_lawful(v, &good);
                fs::remove_file(engine_marker_path(v)).unwrap();
                fs::create_dir_all(engine_marker_path(v)).unwrap();
            }),
        ));

        let mut served = 0;
        let mut failed = false;
        for (name, expected, poison) in &table {
            let versioned = versioned_for(&format!(
                "ad6-siege-{}",
                name.split(|c: char| !c.is_ascii_alphanumeric())
                    .filter(|s| !s.is_empty())
                    .take(3)
                    .collect::<Vec<_>>()
                    .join("-")
            ));
            poison(&versioned);
            let binary = get_binary_path(&versioned, arch);
            let got = decide(accept_cached_entry(&versioned, &binary, arch, 1));
            let ok = got == *expected;
            failed |= !ok;
            if got == Decision::Serve {
                served += 1;
            }
            eprintln!(
                "{} {:<64} expected={:?} got={:?}",
                if ok { "PASS" } else { "FAIL" },
                name,
                expected,
                got
            );
            // SERVE on any (a)-(g) poison state is the critical bug class.
            if got == Decision::Serve && *expected != Decision::Serve {
                failed = true;
                eprintln!("CRITICAL: poison state SERVED: {}", name);
            }
        }

        // (h) layering proof: the version-poison dylib was SERVEd by the
        // builder layers, and a real dlopen/dlsym probe of the served path
        // returns "v1.5.5" — the version gate is downstream (integration
        // tests SELECT version()), exactly as the layering doc above states.
        if probe.is_some() && !failed {
            let served_probe = scratch.join("ad6-siege-h-probe");
            fs::create_dir_all(&served_probe).unwrap();
            let probe_path = scratch.join("version_poison.dylib");
            let binary = get_binary_path(&served_probe, arch);
            fs::copy(&probe_path, &binary).unwrap();
            stage_library_size_pin(&binary).unwrap();
            stage_engine_marker(&served_probe).unwrap();
            if !binary.exists() {
                fs::copy(&probe_path, &binary).unwrap();
            }
            stage_library_size_pin(&binary).unwrap();
            stage_engine_marker(&served_probe).unwrap();
            use std::ffi::{c_char, c_int, c_void, CStr, CString};
            extern "C" {
                fn dlopen(filename: *const c_char, flag: c_int) -> *mut c_void;
                fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
                fn dlclose(handle: *mut c_void) -> c_int;
            }
            let c = CString::new(binary.to_string_lossy().as_bytes()).unwrap();
            unsafe {
                let h = dlopen(c.as_ptr(), 2 /* RTLD_NOW */);
                assert!(
                    !h.is_null(),
                    "version-poison dylib must load (it slice-verifies) to prove layering"
                );
                let sym = dlsym(h, b"duckdb_library_version\0".as_ptr() as *const c_char);
                assert!(!sym.is_null(), "probe symbol missing");
                let f: extern "C" fn() -> *const c_char = std::mem::transmute(sym);
                let got = CStr::from_ptr(f()).to_string_lossy().into_owned();
                dlclose(h);
                eprintln!(
                    "LAYERING PROOF (h): builder layers SERVE this dylib; dlopen probe of the \
                     served path reports engine version '{}' (expected v{}) — the refusal must \
                     come from downstream test-validation",
                    got, VERSION
                );
                assert_eq!(got, "v1.5.5", "wrong-version dylib served as labeled");
            }
        }

        assert!(
            !failed,
            "siege matrix has failing rows (see PASS/FAIL lines above)"
        );
        assert!(
            served <= 3, // control + (g2) + (h): the only lawful SERVEs
            "unexpected SERVE count: {}",
            served
        );
    }
}
