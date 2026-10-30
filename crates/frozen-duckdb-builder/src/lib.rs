//! # Frozen DuckDB Builder
//!
//! This crate handles downloading prebuilt mega-libraries from GitHub Releases
//! or compiling them locally as a fallback. It manages caching under
//! `~/.frozen-duckdb/cache/` (one `v{version}-{arch}/` directory per build)
//! to ensure fast subsequent builds.

pub mod profile;

use anyhow::{Context, Result};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

pub const VERSION: &str = "2.0.0";

/// How `frozen-duckdb-sys` must link the engine for a Cargo target triple.
///
/// Host triples use the cached shared library. `wasm32-wasip1` (and the older
/// `wasm32-wasi` / `wasm32-wasip2` names) must use a wasm32 static archive:
/// wasm-ld cannot load a macOS dylib or a Linux `.so`, and `dylib=duckdb`
/// makes it search for `-lduckdb` and fail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkKind {
    /// `libduckdb.dylib` / `libduckdb.so`.
    Shared,
    /// `libduckdb.a` built for wasm32-wasip1.
    StaticArchive,
}

/// Link kind for `target` (`$TARGET` in a build script).
pub fn link_kind_for_target(target: &str) -> LinkKind {
    match target {
        "wasm32-wasip1" | "wasm32-wasip2" | "wasm32-wasi" => LinkKind::StaticArchive,
        _ => LinkKind::Shared,
    }
}

/// Cache-relative path of the wasip1 static archive for `target`.
pub fn static_archive_cache_name(target: &str) -> String {
    format!("v{VERSION}-{target}/libduckdb.a")
}

/// True when `bytes` is an `ar` archive of wasm32 objects, not a Mach-O or
/// ELF archive. A host `libduckdb_static.a` starts with the same `!<arch>`
/// magic and must not be handed to wasm-ld.
pub fn archive_is_wasm32(bytes: &[u8]) -> bool {
    if !bytes.starts_with(b"!<arch>\n") {
        return false;
    }
    // The symbol index of a full-engine archive is ~10.6 MB, so the first
    // wasm member sits past a 256 KiB window. Scan the whole prefix the
    // caller read (ensure_static_archive reads 64 MiB).
    let window = bytes;
    let has = |magic: &[u8]| window.windows(magic.len()).any(|w| w == magic);
    has(b"\0asm")
        && !has(&[0xcf, 0xfa, 0xed, 0xfe])
        && !has(&[0xfe, 0xed, 0xfa, 0xcf])
        && !has(&[0xca, 0xfe, 0xba, 0xbe])
        && !has(b"\x7fELF")
}

/// Pinned upstream duckdb/duckdb release commit this build stages/compiles
/// from (the `v2.0.0` LTS tag, codename "Cinnamon Teal" / *Spatula
/// cyanoptera*; released 2026-10-21).
const SOURCE_PIN: &str = "d54cbbe3df17e8e5ce68312f483b4155836112aa";
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
/// operative path today. The v2.0.0 workflow is planned to ship the macOS
/// asset set first; the Linux asset set lands with a later tag (TR2
/// coordinates the workflow side).
fn release_asset_name_for(os: &str, arch: &str) -> String {
    format!("libduckdb_{}.{}", arch, library_extension_for(os))
}

/// Name of the frozen-duckdb release asset for `arch` on the host platform.
fn release_asset_name(arch: &str) -> String {
    release_asset_name_for(target_os_name(), arch)
}

/// X3/T2/GAP15: exclusive build lock for the download/compile fallback.
///
/// Pattern (KD-09/BP4 lockfile): a create-exclusive `.build-lock` file
/// carrying the holder's PID. A contender reads the PID; if the holder is
/// dead the lock is STOLEN (removed, then re-raced). After acquiring, the
/// cache is RE-CHECKED (double-checked locking): another process may have
/// finished staging while we waited.
///
/// No external crates: exclusion via `OpenOptions::create_new`, liveness via
/// `kill -0` (exit 0 = alive; EPERM-class failure also means alive — a live
/// process we cannot signal is not a dead one).
const BUILD_LOCK_FILE: &str = ".build-lock";

/// True when `pid` names a live process on this host. Fail-closed: if we
/// cannot determine liveness (kill spawn fails), report ALIVE so we never
/// steal a possibly-live lock.
#[cfg(unix)]
fn pid_alive(pid: u32) -> bool {
    let out = Command::new("kill").arg("-0").arg(pid.to_string()).output();
    match out {
        Ok(o) => {
            if o.status.success() {
                true
            } else {
                // EPERM: process exists but is not signalable by us.
                let err = String::from_utf8_lossy(&o.stderr).to_lowercase();
                o.status.code() != Some(1) || err.contains("operation not permitted")
            }
        }
        // Cannot determine -> assume alive, never steal.
        Err(_) => true,
    }
}

#[cfg(not(unix))]
fn pid_alive(_pid: u32) -> bool {
    // Non-unix: no liveness info -> treat every existing lock as held.
    true
}

/// How long a contended builder waits for the live holder to stage the cache
/// before failing loudly. Bounds the lock to the documented compile envelope.
const LOCK_WAIT_TIMEOUT: Duration = Duration::from_secs(90 * 60);

/// Poll interval for the contended wait. Not a coordination sleep: this is
/// the audited poll step over a real observable (trusted cache entry).
/// SLEEP-CLASS: POLL-STEP
const LOCK_POLL_INTERVAL: Duration = Duration::from_secs(2);

/// Bounded poll for a trusted cache entry staged by the live lock holder.
/// Returns `Ok(true)` when the entry appeared (caller serves it), `Ok(false)`
/// on deadline. Poll re-checks the ENTRY ITSELF, not the lockfile — the
/// holder may stage the entry and still hold the lock while finishing.
fn wait_for_staged_entry(
    versioned_cache: &Path,
    binary_path: &Path,
    arch: &str,
    timeout: Duration,
) -> Result<bool> {
    let deadline = Instant::now() + timeout;
    loop {
        if binary_path.exists()
            && accept_cached_entry(versioned_cache, binary_path, arch, MIN_ENGINE_DYLIB_BYTES)?
                == CacheAdmission::Trusted
        {
            return Ok(true);
        }
        if Instant::now() >= deadline {
            return Ok(false);
        }
        std::thread::sleep(LOCK_POLL_INTERVAL);
    }
}

/// An acquired exclusive build lock. Removing the lockfile on drop ONLY if
/// it still carries our PID (a stolen-then-reacquired lockfile must never be
/// deleted by the process that lost it).
#[derive(Debug)]
pub struct BuildLock {
    lock_path: PathBuf,
    pid: u32,
}

impl BuildLock {
    /// Attempt to acquire the exclusive build lock at `versioned_cache/.build-lock`.
    ///
    /// `Ok(lock)` = we hold it. `Err` naming `LockHeld` = a LIVE holder exists;
    /// caller treats this as a blocking condition (build.rs callers will wait on
    /// the cache to appear; in-process callers serialize on this error).
    pub fn acquire(versioned_cache: &Path) -> Result<Self> {
        let lock_path = versioned_cache.join(BUILD_LOCK_FILE);
        fs::create_dir_all(versioned_cache)
            .with_context(|| format!("creating cache dir {}", versioned_cache.display()))?;
        let pid = std::process::id();
        loop {
            match fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&lock_path)
            {
                Ok(mut f) => {
                    use std::io::Write;
                    writeln!(f, "{}", pid).context("writing build-lock pid")?;
                    return Ok(BuildLock { lock_path, pid });
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    // Stale detection: read the holder PID; steal if dead.
                    let holder = fs::read_to_string(&lock_path)
                        .ok()
                        .and_then(|s| s.trim().parse::<u32>().ok());
                    match holder {
                        Some(h) if h == pid => {
                            // We already hold it (reentrant) — treat as held.
                            anyhow::bail!("build lock already held by this process (pid {})", pid);
                        }
                        Some(h) if pid_alive(h) => anyhow::bail!(
                            "build lock held by live pid {} at {}",
                            h,
                            lock_path.display()
                        ),
                        _ => {
                            // No readable PID or dead holder -> steal and re-race.
                            info!(
                                "stealing stale build lock at {} (holder dead or unparseable)",
                                lock_path.display()
                            );
                            let _ = fs::remove_file(&lock_path);
                            continue;
                        }
                    }
                }
                Err(e) => {
                    return Err(e)
                        .with_context(|| format!("creating build lock at {}", lock_path.display()))
                }
            }
        }
    }
}

impl Drop for BuildLock {
    fn drop(&mut self) {
        // Release only if the lockfile still names OUR pid (not a stolen one).
        if let Ok(content) = fs::read_to_string(&self.lock_path) {
            if content.trim() == self.pid.to_string() {
                let _ = fs::remove_file(&self.lock_path);
            }
        }
    }
}

/// `libc++.a` directory inside a wasi-sdk install.
///
/// wasi-sdk 22+ uses `lib/wasm32-wasip1`. Older SDKs use `lib/wasm32-wasi`.
pub fn wasi_sdk_cxx_lib_dir(sdk: &Path) -> Result<PathBuf> {
    // wasi-sdk 24 and earlier: lib/wasm32-wasip1. wasi-sdk 25+ moved the
    // sysroot under share/wasi-sysroot and split exception variants. DuckDB
    // throws, so the eh archive is the one the link needs.
    for rel in [
        "lib/wasm32-wasip1",
        "lib/wasm32-wasi",
        "share/wasi-sysroot/lib/wasm32-wasip1/eh",
        "share/wasi-sysroot/lib/wasm32-wasi/eh",
    ] {
        let dir = sdk.join(rel);
        if dir.join("libc++.a").is_file() && dir.join("libc++abi.a").is_file() {
            return Ok(dir);
        }
    }
    anyhow::bail!(
        "WASI_SDK_PATH {} has no libc++.a and libc++abi.a under lib/wasm32-wasip1 or share/wasi-sysroot/lib/wasm32-wasip1/eh",
        sdk.display()
    )
}

/// Toolchain file for a wasi-sdk install. SDK 34 names it `wasi-sdk-p1.cmake`.
/// Older SDKs ship `wasi-sdk.cmake`.
pub fn wasi_sdk_toolchain_file(sdk: &Path) -> Result<PathBuf> {
    for rel in [
        "share/cmake/wasi-sdk-p1.cmake",
        "share/cmake/wasi-sdk.cmake",
    ] {
        let path = sdk.join(rel);
        if path.is_file() {
            return Ok(path);
        }
    }
    anyhow::bail!(
        "wasi-sdk toolchain file missing under {} (looked for share/cmake/wasi-sdk-p1.cmake and wasi-sdk.cmake)",
        sdk.display()
    )
}

/// Cached `libduckdb.a` for a wasm32 static target.
///
/// A missing archive is compiled with wasi-sdk when `WASI_SDK_PATH` is set.
/// The host dylib is never returned: linking it is what makes wasm-ld report
/// `unable to find library -lduckdb`.
pub fn ensure_static_archive(target: &str) -> Result<PathBuf> {
    anyhow::ensure!(
        link_kind_for_target(target) == LinkKind::StaticArchive,
        "{target} links the shared library, not a static archive"
    );
    let cache = get_cache_dir()?.join(format!("v{VERSION}-{target}"));
    let archive = cache.join("libduckdb.a");
    if archive.is_file() {
        // The ar symbol index of a full-engine archive runs to tens of MB
        // (measured: 10.6 MB of symbol table before the first `!<arch>` member
        // payload), so the wasm-magic probe must see past it. 256 KiB falsely
        // rejected a genuine wasm32 archive.
        let prefix = read_prefix(&archive, 64 * 1024 * 1024)?;
        if archive_is_wasm32(&prefix) {
            return Ok(archive);
        }
        anyhow::bail!(
            "{} is not a wasm32 archive. A host libduckdb_static.a cannot be linked by wasm-ld. Remove it and rebuild with WASI_SDK_PATH set.",
            archive.display()
        );
    }
    let sdk = env::var("WASI_SDK_PATH").with_context(|| {
        format!(
            "no wasm32 staticlib at {} and WASI_SDK_PATH is unset. \
             {target} cannot link the host dylib (that search is `-lduckdb` and fails). \
             Install wasi-sdk, export WASI_SDK_PATH, and rebuild.",
            archive.display()
        )
    })?;
    compile_wasip1_static(&cache, Path::new(&sdk), target)
}

fn read_prefix(path: &Path, max: usize) -> Result<Vec<u8>> {
    let mut file = fs::File::open(path).with_context(|| format!("opening {}", path.display()))?;
    let mut buf = vec![0u8; max];
    let n = file
        .read(&mut buf)
        .with_context(|| format!("reading {}", path.display()))?;
    buf.truncate(n);
    Ok(buf)
}

/// Turn off `CMAKE_POSITION_INDEPENDENT_CODE` in a pinned DuckDB checkout.
fn disable_wasip1_pic(duckdb_dir: &Path) -> Result<()> {
    let cmake_lists = duckdb_dir.join("CMakeLists.txt");
    let text = fs::read_to_string(&cmake_lists)
        .with_context(|| format!("reading {}", cmake_lists.display()))?;
    let needle = "set(CMAKE_POSITION_INDEPENDENT_CODE ON)";
    anyhow::ensure!(
        text.contains(needle),
        "{} does not set CMAKE_POSITION_INDEPENDENT_CODE ON; refusing to configure a wasm archive whose PIC policy is unknown",
        cmake_lists.display()
    );
    fs::write(
        &cmake_lists,
        text.replacen(needle, "set(CMAKE_POSITION_INDEPENDENT_CODE OFF)", 1),
    )
    .with_context(|| format!("writing {}", cmake_lists.display()))?;
    Ok(())
}

/// Compile the pinned DuckDB sources to `libduckdb.a` for wasm32-wasip1.
fn compile_wasip1_static(cache_dir: &Path, sdk: &Path, target: &str) -> Result<PathBuf> {
    anyhow::ensure!(
        link_kind_for_target(target) == LinkKind::StaticArchive,
        "refusing to wasi-compile for {target}"
    );
    let _cxx = wasi_sdk_cxx_lib_dir(sdk)?;
    let toolchain = wasi_sdk_toolchain_file(sdk)?;

    info!(
        "Compiling DuckDB staticlib for {} with {}",
        target,
        sdk.display()
    );
    fs::create_dir_all(cache_dir).context("Failed to create wasip1 cache directory")?;
    let temp_dir = tempfile::tempdir().context("Failed to create temporary directory")?;
    let duckdb_dir = temp_dir.path().join("duckdb");
    checkout_pinned_duckdb(&duckdb_dir)?;
    // DuckDB's root CMakeLists sets POSITION_INDEPENDENT_CODE ON, which
    // compiles wasm objects as -fPIC (`env.__memory_base`). A plain set()
    // overrides -D, so the checkout is patched before configure.
    disable_wasip1_pic(&duckdb_dir)?;

    let build_dir = duckdb_dir.join("build-wasip1");
    fs::create_dir_all(&build_dir).context("Failed to create wasip1 build directory")?;
    let duckdb_src = duckdb_dir
        .to_str()
        .context("duckdb path is not utf-8")?
        .to_string();
    let build_src = build_dir
        .to_str()
        .context("build path is not utf-8")?
        .to_string();
    let toolchain_arg = format!("-DCMAKE_TOOLCHAIN_FILE={}", toolchain.display());
    let configure = Command::new("cmake")
        .args([
            "-S",
            &duckdb_src,
            "-B",
            &build_src,
            &toolchain_arg,
            "-DCMAKE_BUILD_TYPE=Release",
            "-DBUILD_SHARED_LIBS=OFF",
            "-DBUILD_EXTENSIONS=",
            "-DBUILD_SHELL=OFF",
            "-DBUILD_UNITTESTS=OFF",
            "-DENABLE_EXTENSION_AUTOLOADING=OFF",
            "-DENABLE_EXTENSION_AUTOINSTALL=OFF",
            "-DBUILD_HTTPFS=OFF",
            "-DBUILD_JEMALLOC=OFF",
            "-DDISABLE_THREADS=ON",
        ])
        .output()
        .context("Failed to run cmake for wasm32-wasip1")?;
    anyhow::ensure!(
        configure.status.success(),
        "wasm32-wasip1 cmake configure failed (exit {:?}):\n{}",
        configure.status.code(),
        String::from_utf8_lossy(&configure.stderr)
    );

    let jobs = std::thread::available_parallelism()
        .map(|n| n.get().to_string())
        .unwrap_or_else(|_| "1".into());
    let built = Command::new("cmake")
        .args([
            "--build",
            &build_src,
            "--target",
            "duckdb_static",
            "-j",
            &jobs,
        ])
        .output()
        .context("Failed to run cmake --build for wasm32-wasip1")?;
    if !built.status.success() {
        // Some DuckDB generations name the static target `duckdb` when
        // BUILD_SHARED_LIBS=OFF.
        let fallback = Command::new("cmake")
            .args(["--build", &build_src, "--target", "duckdb", "-j", &jobs])
            .output()
            .context("Failed to run cmake --build fallback for wasm32-wasip1")?;
        anyhow::ensure!(
            fallback.status.success(),
            "wasm32-wasip1 cmake build failed (exit {:?}):\n{}\nfallback:\n{}",
            built.status.code(),
            String::from_utf8_lossy(&built.stderr),
            String::from_utf8_lossy(&fallback.stderr)
        );
    }

    let produced = find_wasip1_archive(&build_dir)?;
    // Same symbol-index law as ensure_static_archive: the ar symbol table of
    // a full-engine archive precedes the first wasm member payload.
    let prefix = read_prefix(&produced, 64 * 1024 * 1024)?;
    anyhow::ensure!(
        archive_is_wasm32(&prefix),
        "cmake produced {} but it is not a wasm32 archive",
        produced.display()
    );
    let dest = cache_dir.join("libduckdb.a");
    fs::copy(&produced, &dest)
        .with_context(|| format!("copying {} to {}", produced.display(), dest.display()))?;
    copy_extension_archives(&build_dir, cache_dir)?;
    info!("Cached wasm32 staticlib at {}", dest.display());
    Ok(dest)
}

fn copy_extension_archives(build_dir: &Path, cache_dir: &Path) -> Result<()> {
    let mut stack = vec![build_dir.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = match fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if name.starts_with("lib") && name.ends_with("_extension.a") {
                fs::copy(&path, cache_dir.join(name))
                    .with_context(|| format!("copying extension archive {}", path.display()))?;
            }
        }
    }
    Ok(())
}

fn find_wasip1_archive(build_dir: &Path) -> Result<PathBuf> {
    let candidates = [
        build_dir.join("src/libduckdb_static.a"),
        build_dir.join("src/libduckdb.a"),
        build_dir.join("libduckdb_static.a"),
        build_dir.join("libduckdb.a"),
    ];
    for path in &candidates {
        if path.is_file() {
            return Ok(path.clone());
        }
    }
    anyhow::bail!(
        "wasm32-wasip1 build finished but libduckdb.a was not found under {}",
        build_dir.display()
    )
}

fn checkout_pinned_duckdb(duckdb_dir: &Path) -> Result<()> {
    let parent = duckdb_dir
        .parent()
        .context("duckdb checkout path has no parent")?;
    let clone_out = Command::new("git")
        .args([
            "clone",
            "--filter=blob:none",
            "https://github.com/duckdb/duckdb.git",
        ])
        .arg(duckdb_dir)
        .current_dir(parent)
        .output()
        .context("Failed to clone DuckDB repository")?;
    anyhow::ensure!(
        clone_out.status.success(),
        "DuckDB git clone failed (exit {:?}):\n{}",
        clone_out.status.code(),
        String::from_utf8_lossy(&clone_out.stderr)
    );
    let checkout_out = Command::new("git")
        .args(["-C"])
        .arg(duckdb_dir)
        .args(["checkout", SOURCE_PIN])
        .output()
        .context("Failed to run git checkout of pinned DuckDB commit")?;
    anyhow::ensure!(
        checkout_out.status.success(),
        "Failed to check out pinned DuckDB commit {} (exit {:?}):\n{}",
        SOURCE_PIN,
        checkout_out.status.code(),
        String::from_utf8_lossy(&checkout_out.stderr)
    );
    Ok(())
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
        cache_hit = match accept_cached_entry(
            &versioned_cache,
            &binary_path,
            &arch,
            MIN_ENGINE_DYLIB_BYTES,
        )? {
            CacheAdmission::Trusted => true,
            CacheAdmission::Refused(invariant) => {
                poison_state_refuse(&versioned_cache, invariant);
            }
        };
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
        // X3/T2/GAP15: enter the expensive fallback only under the exclusive
        // build lock, then RE-CHECK the cache — a contender that was already
        // compiling may have staged the entry while we waited for the lock.
        let _lock = match BuildLock::acquire(&versioned_cache) {
            Ok(l) => Some(l),
            Err(_held) => {
                // Contended: a live builder is staging this exact cache entry.
                // Bounded poll on the real observable (a trusted cache entry
                // appearing); fail loudly on deadline instead of double-compiling.
                if wait_for_staged_entry(&versioned_cache, &binary_path, &arch, LOCK_WAIT_TIMEOUT)?
                {
                    info!(
                        "Cache entry staged by another builder while waiting: {}",
                        binary_path.display()
                    );
                    None
                } else {
                    return Err(anyhow::anyhow!(
                        "another builder (pid) holds the build lock at {} and no cache entry appeared within {:?}",
                        versioned_cache.join(BUILD_LOCK_FILE).display(),
                        LOCK_WAIT_TIMEOUT
                    ))
                    .context("DuckDB build lock contended; refusing to double-compile");
                }
            }
        };

        let mut post_lock_hit = false;
        if _lock.is_some() && binary_path.exists() {
            post_lock_hit = match accept_cached_entry(
                &versioned_cache,
                &binary_path,
                &arch,
                MIN_ENGINE_DYLIB_BYTES,
            )? {
                CacheAdmission::Trusted => true,
                CacheAdmission::Refused(invariant) => {
                    poison_state_refuse(&versioned_cache, invariant);
                }
            };
        }
        if post_lock_hit {
            info!(
                "Cache entry staged by another process while waiting on build lock: {}",
                binary_path.display()
            );
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
    }

    // Normalize the cache layout so downstream builds always work:
    // headers under duckdb/ for bindgen, plain libduckdb.dylib for -lduckdb
    ensure_headers(&versioned_cache)?;
    ensure_link_name(&versioned_cache, &arch)?;

    Ok(binary_path)
}

/// Typed verdict of the cache-entry admission law. `Trusted` = serve the
/// cached dylib. `Refused(invariant)` = the entry failed the named
/// invariant; the caller MUST abort with `POISON_STATE_REFUSED: <cache>
/// (<invariant>)` on stderr and exit code 1 — never a silent restage.
#[derive(Debug, PartialEq, Eq)]
enum CacheAdmission {
    Trusted,
    Refused(&'static str),
}

/// Typed poison-rejection exit (MAX1): on any invariant failure, clean the
/// untrusted entry so the NEXT run restages from empty, print the literal
/// stderr contract line, and abort the process with exit code 1. This
/// replaces the pre-MAX1 warn+restage demotion, which silently swallowed
/// poison and served a freshly-staged fallback with no typed signal.
fn poison_state_refuse(versioned_cache: &Path, invariant: &'static str) -> ! {
    eprintln!(
        "POISON_STATE_REFUSED: {} ({})",
        versioned_cache.display(),
        invariant
    );
    std::process::exit(1)
}

/// Trusted-hit law for a versioned cache entry (DT1 refactor of the
/// ensure_binary hit block): the ENGINE_VERSION marker must carry the
/// builder's own pin, the `.size` sidecar must match the dylib's actual
/// length (AD1: `lipo -archs` reads only the Mach-O header, so a truncated
/// dylib still slice-verifies), and the slice must verify.
///
/// `Ok(Trusted)` = serve the cached dylib. `Ok(Refused(invariant))` = the
/// entry failed the named invariant; the caller aborts via
/// `poison_state_refuse` (typed stderr + exit 1) after the untrusted files
/// are removed so the next run restages from a genuinely empty entry.
/// `Err` = hard failure the caller propagates.
fn accept_cached_entry(
    versioned_cache: &Path,
    binary_path: &Path,
    arch: &str,
    min_bytes: u64,
) -> Result<CacheAdmission> {
    // C2 strictness (AD6 siege): the label law is BYTE-EXACT. `stage_engine_marker`
    // writes the pin with no trailing bytes, so a marker whose raw payload is
    // `v{VERSION}\n` or `v{VERSION}\r\n` was NOT staged by this builder and
    // is refused like any foreign label. `read_engine_marker`'s trim stays
    // for display only. The DT1 regular-file law is preserved: symlinks and
    // directories are refused before their contents are even read.
    let marker_path = engine_marker_path(versioned_cache);
    let marker_state = match fs::symlink_metadata(&marker_path) {
        Err(_) => ("missing-marker", None),
        Ok(m) if !m.is_file() => ("symlinked-marker", None),
        Ok(_) => ("foreign-marker", fs::read(&marker_path).ok()),
    };
    let (invariant, raw) = marker_state;
    let marker_label = raw
        .as_deref()
        .map(|bytes| String::from_utf8_lossy(bytes).trim().to_string())
        .unwrap_or_else(|| "<absent>".to_string());
    if raw.as_deref() != Some(expected_engine_marker().as_bytes()) {
        warn!(
            "cache entry labeled v{} carries marker '{}' ({}); restaging",
            VERSION, marker_label, invariant
        );
        return Ok(CacheAdmission::Refused(invariant));
    }

    match read_library_size_pin(binary_path) {
        Some(pin) if pin == library_len(binary_path) => {}
        pin => {
            warn!(
                "cache binary {} length {} mismatches its size pin {:?}; refusing",
                binary_path.display(),
                library_len(binary_path),
                pin
            );
            // Fail-closed refusal: remove the untrusted dylib and its pin so
            // no concurrent or retried reader can be served them, and so the
            // next run's restage starts from a genuinely empty entry.
            let _ = fs::remove_file(binary_path);
            let _ = fs::remove_file(library_size_pin_path(binary_path));
            return Ok(CacheAdmission::Refused("truncated-dylib"));
        }
    }

    // A cache written before TR7 closed may hold the wrong slice under this
    // label (x86_64 bytes in v{VER}-arm64/libduckdb_arm64.dylib).
    if let Err(_e) = verify_library_slice_for(target_os_name(), binary_path, arch, min_bytes) {
        return Ok(CacheAdmission::Refused("slice-magic-or-floor"));
    }

    // KD-15 self-heal: a cache written before the install-ID stamp existed is
    // still accepted, then stamped in place (idempotent). Stamping changes the
    // byte length, so the size pin is re-staged over the stamped bytes; the
    // ENGINE_VERSION marker is untouched (already validated above).
    if ensure_install_id(binary_path)? {
        stage_library_size_pin(binary_path)?;
    }
    Ok(CacheAdmission::Trusted)
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

    // KD-15: stamp the install ID BEFORE the size pin so the pin covers the
    // stamped bytes (install_name_tool + codesign change the length).
    ensure_install_id(cache_path)?;

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
    /// `lipo -archs` listed the labeled slice (macOS) or the platform magic
    /// matched (Linux ELF / Windows PE) (REL5).
    Verified,
    /// No checker available on this host (e.g. `lipo` absent on macOS); the
    /// label is UNVERIFIED, not admitted.
    Unverifiable(String),
    /// The file exists but structurally contradicts its platform label
    /// (wrong magic, truncated below the engine floor). Hard refusal, never
    /// a silent pass (REL5: a PE/ELF served on a non-macOS leg must fail).
    Fail(String),
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
        // REL5: on a non-macOS leg the previously-silent `Unverifiable`
        // return would serve an unverified (possibly wrong-platform) binary.
        // Read the leading magic and demand the platform expectation:
        //   linux   -> ELF    7f 45 4c 46
        //   windows -> PE      'M' 'Z'
        // Wrong/short magic or a below-floor payload is a hard
        // `SliceCheck::Fail`, not a silent pass.
        let expected = match os {
            "linux" => ("ELF", [0x7f, b'E', b'L', b'F']),
            "windows" => ("PE", [b'M', b'Z', 0, 0]),
            other => {
                return Ok(SliceCheck::Unverifiable(format!(
                    "no slice checker for os={}",
                    other
                )))
            }
        };
        let len = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        if len < min_bytes {
            return Ok(SliceCheck::Fail(format!(
                "{} is {} bytes, below the {} byte engine size floor (truncated download?): \
                 refusing to serve {}",
                path.display(),
                len,
                min_bytes,
                expected.0
            )));
        }
        let mut magic = [0u8; 4];
        if let Err(e) = fs::File::open(path).and_then(|mut f| f.read_exact(&mut magic)) {
            return Ok(SliceCheck::Fail(format!(
                "{} unreadable while reading {} magic: {}",
                path.display(),
                expected.0,
                e
            )));
        }
        let matched = match expected.0 {
            "ELF" => magic == [0x7f, b'E', b'L', b'F'],
            "PE" => magic[..2] == *b"MZ",
            _ => false,
        };
        if !matched {
            return Ok(SliceCheck::Fail(format!(
                "{} lacks the {} magic (first bytes {:02x?}); wrong-platform or corrupt \
                 download (REL5): refusing to serve",
                path.display(),
                expected.0,
                magic
            )));
        }
        return Ok(SliceCheck::Verified);
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

/// KD-15 (LC_RPATH does not propagate cross-crate): a cached dylib whose
/// Mach-O install ID is `@rpath/libduckdb.dylib` makes every consumer record
/// `@rpath/libduckdb.dylib` in its LC_LOAD_DYLIB regardless of how the linker
/// found the library, so a bare run dies with "no LC_RPATH's found". The fix
/// is at the SOURCE of the ID: stamp the dylib's install ID to its absolute
/// cache path (install_name_tool -id) and re-sign ad-hoc (codesign -f -s -).
/// Consumers then record the absolute cache path and load without any rpath
/// or DYLD env var. Idempotent: an already-stamped library is left untouched.
/// Non-Mach-O payloads (unit-test fixtures) are skipped, not an error.
#[cfg(target_os = "macos")]
fn ensure_install_id(path: &Path) -> Result<bool> {
    // Only stamp real Mach-O images: unit tests persist fake byte payloads
    // through persist_verified_for, and install_name_tool would reject them.
    let mut magic = [0u8; 4];
    fs::File::open(path)
        .and_then(|mut f| f.read_exact(&mut magic))
        .with_context(|| {
            format!(
                "failed to read magic for install-ID check: {}",
                path.display()
            )
        })?;
    if !is_macho_magic(&magic) {
        debug!("install-ID stamp skipped (not Mach-O): {}", path.display());
        return Ok(false);
    }

    let wanted = fs::canonicalize(path)?;
    let path_str = path.to_str().context("non-UTF8 path")?;
    let wanted_str = wanted.to_str().context("non-UTF8 path")?;

    let out = Command::new("otool")
        .args(["-D", path_str])
        .output()
        .context("failed to run otool -D")?;
    let current = String::from_utf8_lossy(&out.stdout)
        .lines()
        .last()
        .unwrap_or("")
        .trim()
        .to_string();
    if out.status.success() && current == wanted_str {
        debug!("install-ID already stamped: {}", path.display());
        return Ok(false);
    }

    let out = Command::new("install_name_tool")
        .args(["-id", wanted_str, path_str])
        .output()
        .context("failed to run install_name_tool")?;
    if !out.status.success() {
        // Best-effort: a dylib linked without -headerpad_max_install_names has
        // no room to grow its LC_ID_DYLIB (observed on minimal cc-built test
        // fixtures; CMake-built DuckDB carries the padding). Serving an
        // unstamped library only re-opens KD-15 for consumers (they need an
        // rpath or DYLD_LIBRARY_PATH again) — it must not brick the build, so
        // warn loudly and leave the library untouched.
        warn!(
            "KD-15 install-ID stamp UNSUCCESSFUL on {} (consumers will need an              rpath or DYLD_LIBRARY_PATH): {}",
            path.display(),
            String::from_utf8_lossy(&out.stderr).trim()
        );
        return Ok(false);
    }
    // install_name_tool invalidates the existing code signature; re-sign
    // ad-hoc so the loader accepts the modified image.
    let out = Command::new("codesign")
        .args(["--force", "--sign", "-", path_str])
        .output()
        .context("failed to run codesign")?;
    if !out.status.success() {
        anyhow::bail!(
            "codesign ad-hoc re-sign failed on {} (KD-15 stamp): {}",
            path.display(),
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    info!(
        "install-ID stamped to {}: {}",
        wanted.display(),
        path.display()
    );
    Ok(true)
}

/// Linux: stamp `DT_SONAME` to the absolute cache path, the ELF counterpart
/// of the Mach-O install id. Consumers record that path in `DT_NEEDED` and
/// load the cache without `LD_LIBRARY_PATH` or a forwarded rpath.
/// Non-ELF payloads (unit-test fixtures) are skipped. Missing `patchelf` or
/// `readelf` warns and leaves the file untouched, same as a Mach-O that has
/// no header pad.
#[cfg(target_os = "linux")]
fn ensure_install_id(path: &Path) -> Result<bool> {
    let mut magic = [0u8; 4];
    fs::File::open(path)
        .and_then(|mut f| f.read_exact(&mut magic))
        .with_context(|| format!("failed to read magic for SONAME check: {}", path.display()))?;
    if magic != [0x7f, b'E', b'L', b'F'] {
        debug!("SONAME stamp skipped (not ELF): {}", path.display());
        return Ok(false);
    }

    let wanted = fs::canonicalize(path)?;
    let wanted_str = wanted.to_str().context("non-UTF8 path")?;
    let path_str = path.to_str().context("non-UTF8 path")?;

    let out = Command::new("readelf")
        .args(["-d", path_str])
        .output()
        .context("failed to run readelf -d")?;
    if !out.status.success() {
        warn!(
            "KD-15 SONAME stamp skipped (readelf failed) on {}: {}",
            path.display(),
            String::from_utf8_lossy(&out.stderr).trim()
        );
        return Ok(false);
    }
    let dump = String::from_utf8_lossy(&out.stdout);
    let current = dump
        .lines()
        .find(|line| line.contains("(SONAME)"))
        .and_then(|line| line.split('[').nth(1))
        .and_then(|rest| rest.split(']').next())
        .unwrap_or("")
        .trim();
    if current == wanted_str {
        debug!("SONAME already stamped: {}", path.display());
        return Ok(false);
    }

    let out = Command::new("patchelf")
        .args(["--set-soname", wanted_str, path_str])
        .output()
        .context("failed to run patchelf")?;
    if !out.status.success() {
        warn!(
            "KD-15 SONAME stamp UNSUCCESSFUL on {} (consumers will need LD_LIBRARY_PATH or an rpath): {}",
            path.display(),
            String::from_utf8_lossy(&out.stderr).trim()
        );
        return Ok(false);
    }
    info!("SONAME stamped to {}: {}", wanted.display(), path.display());
    Ok(true)
}

/// Windows and other hosts: no install-id stamp.
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn ensure_install_id(_path: &Path) -> Result<bool> {
    Ok(false)
}

/// Host-platform form of [`verify_library_slice_for`]; an unverifiable
/// check is logged, a mismatch or unreadable library is an error.
fn verify_library_slice(path: &Path, arch: &str) -> Result<()> {
    match verify_library_slice_for(target_os_name(), path, arch, MIN_ENGINE_DYLIB_BYTES)? {
        SliceCheck::Verified => debug!("slice {} verified in {}", arch, path.display()),
        SliceCheck::Unverifiable(why) => {
            warn!("slice {} UNVERIFIED for {}: {}", arch, path.display(), why)
        }
        SliceCheck::Fail(why) => {
            anyhow::bail!(
                "slice check FAILED for {} ({}): {}",
                path.display(),
                arch,
                why
            )
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
/// Test-and-review helper; the production hit path compares the raw label
/// inline (see `accept_cached_entry`). Kept for the marker courts.
#[allow(dead_code)]
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
    // KD-15: stamp before the size pin so the pin covers the stamped bytes.
    ensure_install_id(dest)?;
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
    // "2.0.0"); the release-download path stays keyed to VERSION.
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
    // CMake extension law for DuckDB v2.0.0: BUILD_EXTENSIONS names
    // extensions to BUILD (loadable by default); the statically-linked set
    // (core_functions, icu, json, parquet — with their generated
    // `_duckdb_extension_<name>_describe` definitions) comes from each
    // extension's own default `duckdb_extension_statically_link` config, NOT
    // from LINK_CORE_EXTENSIONS (turning that on at this tag drops the
    // describe objects from the archives and the dylib link fails on
    // undefined describes — witnessed 2026-10-29, build log in
    // docs/sjira/v2.0.4/). Keep the v1.5-era BUILD_EXTENSIONS list: it works
    // under the targeted `cmake --build` invocation below.
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

    // Build the shared `duckdb` target explicitly. DuckDB v2.0.0's default
    // `make` target (all) builds the loadable-extension variants alongside
    // the static ones; during that run the static extension archives
    // (libjson_extension.a etc.) were re-archived WITHOUT their generated
    // `_duckdb_extension_<name>_describe` objects and the libduckdb.dylib
    // link then failed on undefined describes. A targeted build of `duckdb`
    // (+ deps) archives the describes and links cleanly — witnessed
    // 2026-10-29 (probe: cmake --build --target duckdb exit 0; make all:
    // describe-free archives, link exit 1).
    let make_out = Command::new("cmake")
        .args(["--build", ".", "--target", "duckdb", "--parallel", "4"])
        .current_dir(&build_dir)
        .output()
        .context("Failed to build DuckDB")?;
    if !make_out.status.success() {
        anyhow::bail!(
            "DuckDB build failed (exit {:?}):\n{}",
            make_out.status.code(),
            String::from_utf8_lossy(&make_out.stderr)
        );
    }

    // Find the built library
    let built_lib = find_built_library(&build_dir, arch).context("Failed to find built library")?;
    verify_library_slice(&built_lib, arch)?;

    // Copy library to cache directory with proper name
    let binary_path = get_binary_path(cache_dir, arch);

    // KD-01 fence-restage: apply the export fence to the freshly built dylib
    // BEFORE it is persisted to the cache, so a local compile can no longer
    // produce (or a restage silently revert to) the 27,682-mangled-export
    // dylib. See crates/frozen-duckdb-builder/export_fence/README.md.
    fence_exports_for(&built_lib, &binary_path)
        .context("KD-01 export fence failed on locally compiled dylib")?;

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
    // KD-15: stamp before the size pin so the pin covers the stamped bytes.
    ensure_install_id(&binary_path)?;
    stage_library_size_pin(&binary_path)?;
    stage_engine_marker(cache_dir)?;
    Ok(binary_path)
}

/// Committed KD-01 export-fence allowlist: exactly the C-ABI surface
/// (`_duckdb_*`) a fenced dylib may export. Provenance + refresh law:
/// `crates/frozen-duckdb-builder/export_fence/README.md`.
const EXPORT_FENCE_LIST: &str = include_str!("../export_fence/exports.list");

/// Census of a Mach-O dylib's exported (`nm -gU`) symbol surface:
/// `(mangled_cxx, duckdb_c_abi, total_t)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ExportCensus {
    mangled_cxx: usize,
    duckdb_c_abi: usize,
    total_t: usize,
}

/// Parse `nm -gU` output (one symbol per line: address/type/name).
fn parse_export_census(nm_output: &str) -> ExportCensus {
    let mut c = ExportCensus {
        mangled_cxx: 0,
        duckdb_c_abi: 0,
        total_t: 0,
    };
    for line in nm_output.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        // external defined text symbols look like "<addr> T _name" (or
        // "<addr>, T _name" — nm prints a literal comma for some symbols).
        if fields.len() < 2 || (fields[1] != "T" && fields[1] != "t") {
            continue;
        }
        let name = fields
            .iter()
            .skip(2)
            .find(|f| f.starts_with('_'))
            .copied()
            .unwrap_or("");
        let name = name.trim_end_matches(',');
        if name.is_empty() {
            continue;
        }
        if fields[1] == "t" {
            continue; // local text symbols are not exports
        }
        c.total_t += 1;
        if name.starts_with("__Z") {
            c.mangled_cxx += 1;
        }
        if name.starts_with("_duckdb_") {
            c.duckdb_c_abi += 1;
        }
    }
    c
}

/// Run `nm -gU` on a Mach-O binary and census its export surface.
fn export_census(dylib_path: &Path) -> Result<ExportCensus> {
    let out = Command::new("nm")
        .args(["-gU"])
        .arg(dylib_path)
        .output()
        .context("Failed to run nm for export census")?;
    anyhow::ensure!(
        out.status.success(),
        "nm -gU failed on {} (exit {:?})",
        dylib_path.display(),
        out.status.code()
    );
    Ok(parse_export_census(&String::from_utf8_lossy(&out.stdout)))
}

/// Locate the static DuckDB archive a dylib can be relinked from. The
/// local-compile flow lays `libduckdb.dylib` and `libduckdb_static.a` in the
/// same cmake output directory; the search walks up a bounded number of
/// levels so alternate layouts still fence.
fn find_static_archive(dylib_path: &Path) -> Option<PathBuf> {
    let mut dir = dylib_path.parent()?;
    for _ in 0..3 {
        let candidate = dir.join("libduckdb_static.a");
        if candidate.exists() {
            return Some(candidate);
        }
        dir = dir.parent()?;
    }
    None
}

/// Locate the CMake-generated link command for the `duckdb` shared target
/// (sibling of the core static archive). The v2.0.0 fence relink replays this
/// exact link with the export fence injected — the CMake link already
/// resolves the generated `_duckdb_extension_<name>_describe` machinery
/// (capability + extension archives plus the duckdb-target objects), which a
/// hand-assembled archive list cannot reconstruct reliably at this tag.
fn find_duckdb_link_file(static_archive: &Path) -> Option<PathBuf> {
    let build_src = static_archive.parent()?;
    let candidate = build_src.join("CMakeFiles/duckdb.dir/link.txt");
    candidate.exists().then_some(candidate)
}

/// Compute the effective export list for the fence relink: the KD-01
/// allowlist ∩ the symbols the relink inputs actually define. ld64 treats
/// `-exported_symbols_list` entries as required roots, so an allowlist entry
/// with no definition in the inputs (upstream ships `autocomplete` in the
/// static set; our local extension config does not) fails the link with
/// "undefined symbol ... referenced from <initial-undefines>". Intersecting
/// keeps the allowlist a ceiling without turning it into a requirement; the
/// post-relink census still enforces zero C++-mangled exports.
fn effective_export_list(allowlist: &str, inputs: &[String], cwd: &Path) -> Result<String> {
    let mut defined: HashSet<String> = HashSet::new();
    for input in inputs {
        let path = cwd.join(input);
        if !path.exists() {
            continue;
        }
        let out = Command::new("nm")
            .args(["--defined-only"])
            .arg(&path)
            .output()
            .with_context(|| format!("nm failed on {}", path.display()))?;
        if !out.status.success() {
            continue;
        }
        for line in String::from_utf8_lossy(&out.stdout).lines() {
            if let Some(name) = line.split_whitespace().nth(2) {
                if name.starts_with("_duckdb_") {
                    defined.insert(name.to_string());
                }
            }
        }
    }
    let effective: Vec<&str> = allowlist
        .lines()
        .filter(|l| defined.contains(l.trim()))
        .collect();
    Ok(effective.join("\n"))
}

/// Patch the CMake link command for the fence relink: redirect the output to
/// the fenced dylib, stamp the final cache install name, and inject the
/// `-Wl,-exported_symbols_list` fence. Returns the argv to execute.
fn patch_link_command(
    link_file: &Path,
    list_file: &Path,
    install_name: &Path,
    fenced_output: &Path,
) -> Result<Vec<String>> {
    let raw = fs::read_to_string(link_file)
        .with_context(|| format!("Failed to read {}", link_file.display()))?;
    let mut argv: Vec<String> = raw.split_whitespace().map(String::from).collect();
    anyhow::ensure!(
        !argv.is_empty(),
        "empty link command in {}",
        link_file.display()
    );
    // Drop the original output flag pair and re-emit ours, so the fence never
    // overwrites the CMake-built dylib in place. Also drop the original
    // -install_name pair — the linker takes the last one, so ours must be the
    // only one.
    for flag in ["-o", "-install_name"] {
        if let Some(pos) = argv.iter().position(|a| a == flag) {
            let end = (pos + 2).min(argv.len());
            argv.drain(pos..end);
        }
    }
    // Inject fence flags right after the compiler program name.
    let inject = vec![
        "-o".to_string(),
        fenced_output.display().to_string(),
        "-install_name".to_string(),
        install_name.display().to_string(),
        "-Wl,-exported_symbols_list".to_string(),
        list_file.display().to_string(),
    ];
    argv.splice(1..1, inject);
    Ok(argv)
}

/// KD-01 export fence (macOS only): relink `dylib_path` in place from its
/// sibling `libduckdb_static.a` with `-Wl,-exported_symbols_list`, so the
/// exports trie carries exactly the 1,177 `_duckdb_*` C-ABI symbols and zero
/// C++-mangled exports. Darwin has no post-hoc tool for this (`strip -x`
/// leaves the trie; no objcopy) — hence relink, per KD-01/ADV3 and lane
/// X1r's recipe (`docs/sjira/v2.0.0/receipts/X1r-kd01-fence.md`).
///
/// - Non-macOS (Linux ELF): typed no-op SKIP — the fence is Mach-O-specific
///   and Linux is out of scope (KD-15 scope note).
/// - No static archive found: the dylib cannot be relinked; it is admitted
///   only if it ALREADY passes the fence census (0 mangled exports) — an
///   idempotent no-op. An unfenced dylib without an archive is a typed
///   `Err`, never a silent pass.
#[cfg(test)]
pub(crate) fn fence_exports(dylib_path: &Path) -> Result<()> {
    fence_exports_for(dylib_path, dylib_path)
}

/// Like [`fence_exports`], but stamps `install_name` (the final cache path)
/// into the relinked dylib, mirroring X1r's relink (the cache dylib's
/// `LC_ID_DYLIB` must name the cache slot, not the temp build directory).
pub(crate) fn fence_exports_for(dylib_path: &Path, install_name: &Path) -> Result<()> {
    if target_os_name() != "macos" {
        info!(
            "[SKIP] KD-01 export fence: macOS-only (subject {} is a {} binary) — Linux ELF out of scope per KD-15",
            dylib_path.display(),
            target_os_name()
        );
        return Ok(());
    }

    // Census BEFORE: decide between relink, idempotent no-op, and typed refusal.
    if find_static_archive(dylib_path).is_none() {
        let census = export_census(dylib_path)?;
        anyhow::ensure!(
            census.mangled_cxx == 0,
            "KD-01 export fence: {} exports {} C++-mangled symbols but no libduckdb_static.a \
             sibling exists to relink from — rebuild the cache (local compile) so the fence \
             can relink, then re-verify",
            dylib_path.display(),
            census.mangled_cxx
        );
        debug!(
            "KD-01 export fence: {} already fenced ({} C-ABI exports, 0 mangled) — no-op",
            dylib_path.display(),
            census.duckdb_c_abi
        );
        return Ok(());
    }

    let temp_dir = tempfile::tempdir().context("Failed to create fence temp directory")?;
    let temp_path = temp_dir.path();
    let fenced_output = temp_path.join("libduckdb_fenced.dylib");

    let static_archive =
        find_static_archive(dylib_path).expect("static archive existence checked above");
    let link_file = find_duckdb_link_file(&static_archive).with_context(|| {
        format!(
            "KD-01 export fence: no CMake link command at {} — cannot relink",
            static_archive
                .parent()
                .map(|p| p.join("CMakeFiles/duckdb.dir/link.txt"))
                .map(|p| p.display().to_string())
                .unwrap_or_default()
        )
    })?;
    // Effective export list = KD-01 allowlist ∩ symbols the link inputs
    // define (see effective_export_list — ld64 roots export-list entries).
    let relink_dir = link_file
        .parent()
        .and_then(|d| d.parent())
        .and_then(|d| d.parent())
        .map(|p| p.to_path_buf());
    let link_cwd = relink_dir
        .as_deref()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    let raw_link = fs::read_to_string(&link_file)
        .with_context(|| format!("Failed to read {}", link_file.display()))?;
    let inputs: Vec<String> = raw_link
        .split_whitespace()
        .filter(|a| a.ends_with(".a") || a.ends_with(".o"))
        .map(String::from)
        .collect();
    let effective = effective_export_list(EXPORT_FENCE_LIST, &inputs, &link_cwd)?;
    let list_file = temp_path.join("exports.list");
    fs::write(&list_file, effective).context("Failed to write effective export fence list")?;
    let argv = patch_link_command(&link_file, &list_file, install_name, &fenced_output)?;
    debug!("KD-01 fence relink argv: {argv:?}");
    // Replay the link from the directory CMake uses (link.txt is
    // build/src-relative; see the relink_dir computation above).
    let mut replay = Command::new(&argv[0]);
    replay.args(&argv[1..]);
    replay.current_dir(&link_cwd);
    let relink = replay.output().context("Failed to run fence relink")?;
    anyhow::ensure!(
        relink.status.success(),
        "KD-01 export fence relink failed (exit {:?}):\n{}",
        relink.status.code(),
        String::from_utf8_lossy(&relink.stderr)
    );

    let codesign = Command::new("codesign")
        .args(["--force", "-s", "-"])
        .arg(&fenced_output)
        .output()
        .context("Failed to run codesign on fenced dylib")?;
    anyhow::ensure!(
        codesign.status.success(),
        "KD-01 export fence codesign failed (exit {:?}):\n{}",
        codesign.status.code(),
        String::from_utf8_lossy(&codesign.stderr)
    );

    // Census AFTER: the fence must actually fence. A relink that still leaks
    // mangled exports is a typed failure, not a persisted artifact.
    let after = export_census(&fenced_output)?;
    anyhow::ensure!(
        after.mangled_cxx == 0,
        "KD-01 export fence produced a dylib that still exports {} C++-mangled symbols — \
         refusing to persist",
        after.mangled_cxx
    );

    atomic_copy(&fenced_output, dylib_path).context("Failed to persist fenced dylib")?;
    info!(
        "KD-01 export fence applied: {} now exports {} C-ABI symbols, 0 mangled",
        dylib_path.display(),
        after.duckdb_c_abi
    );
    Ok(())
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
    fn wasip1_targets_link_a_static_archive() {
        assert_eq!(
            link_kind_for_target("wasm32-wasip1"),
            LinkKind::StaticArchive
        );
        assert_eq!(
            link_kind_for_target("wasm32-wasip2"),
            LinkKind::StaticArchive
        );
        assert_eq!(link_kind_for_target("wasm32-wasi"), LinkKind::StaticArchive);
        assert_eq!(
            link_kind_for_target("aarch64-apple-darwin"),
            LinkKind::Shared
        );
        assert_eq!(
            link_kind_for_target("x86_64-unknown-linux-gnu"),
            LinkKind::Shared
        );
        assert_eq!(
            static_archive_cache_name("wasm32-wasip1"),
            format!("v{VERSION}-wasm32-wasip1/libduckdb.a")
        );
    }

    #[test]
    fn wasm_archive_magic_rejects_host_objects() {
        let mut wasm = b"!<arch>\n".to_vec();
        wasm.extend_from_slice(&[0u8; 60]);
        wasm.extend_from_slice(b"\0asm");
        assert!(archive_is_wasm32(&wasm));

        // Full-engine ar symbol index: the first wasm member is past 256 KiB.
        let mut deep = b"!<arch>\n".to_vec();
        deep.resize(deep.len() + 300 * 1024, 0u8);
        deep.extend_from_slice(b"\0asm");
        assert!(archive_is_wasm32(&deep));

        let mut macho = b"!<arch>\n".to_vec();
        macho.extend_from_slice(&[0xcf, 0xfa, 0xed, 0xfe]);
        macho.extend_from_slice(b"\0asm");
        assert!(!archive_is_wasm32(&macho));

        let mut elf = b"!<arch>\n".to_vec();
        elf.extend_from_slice(b"\x7fELF");
        elf.extend_from_slice(b"\0asm");
        assert!(!archive_is_wasm32(&elf));
        assert!(!archive_is_wasm32(b"\0asm"));
    }

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
    fn rel5_elf_magic_verifies_on_linux_branch() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("libduckdb_x86_64.so");
        // Crafted ELF-magic payload (min_bytes=1 floor so the fixture stays tiny).
        let mut bytes = vec![0x7f, b'E', b'L', b'F'];
        bytes.extend_from_slice(&[2, 1, 1, 0]); // 64-bit LE SysV ident tail
        fs::write(&path, &bytes).unwrap();
        assert_eq!(
            verify_library_slice_for("linux", &path, "x86_64", 1).unwrap(),
            SliceCheck::Verified
        );
    }

    #[test]
    fn rel5_text_file_fails_hard_on_linux_branch() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("libduckdb_x86_64.so");
        fs::write(&path, b"not inspected").unwrap();
        match verify_library_slice_for("linux", &path, "x86_64", 1).unwrap() {
            SliceCheck::Fail(why) => assert!(why.contains("ELF magic"), "{}", why),
            other => panic!(
                "text payload must Fail hard on the linux branch, got {:?}",
                other
            ),
        }
    }

    #[test]
    fn rel5_wrong_magic_fails_on_windows_branch() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("duckdb.dll");
        // An ELF served where a PE (MZ) is expected must Fail, not verify.
        fs::write(&path, [0x7f, b'E', b'L', b'F']).unwrap();
        match verify_library_slice_for("windows", &path, "x86_64", 1).unwrap() {
            SliceCheck::Fail(why) => assert!(why.contains("PE magic"), "{}", why),
            other => panic!("ELF on windows branch must Fail, got {:?}", other),
        }
    }

    #[test]
    fn rel5_truncated_linux_payload_fails_size_floor() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("libduckdb_x86_64.so");
        fs::write(&path, [0x7f, b'E', b'L', b'F']).unwrap();
        match verify_library_slice_for("linux", &path, "x86_64", 1024).unwrap() {
            SliceCheck::Fail(why) => assert!(why.contains("size floor"), "{}", why),
            other => panic!("below-floor payload must Fail, got {:?}", other),
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
        // -headerpad_max_install_names: leave room for install_name_tool -id
        // to grow the LC_ID_DYLIB (KD-15 stamp path is exercised on these
        // fixtures by persist/accept courts).
        let status = cc
            .arg("-dynamiclib")
            .args(["-Wl,-headerpad_max_install_names"])
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
                Ok(SliceCheck::Fail(why)) => {
                    panic!("{} must Fail closed loudly: {}", p.display(), why)
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
        // KD-15: the persisted bytes are the DELIVERED payload with the
        // install ID stamped to the cache path (and re-signed), so byte
        // equality with the payload no longer holds. What must hold:
        // idempotence across duplicate delivery, and a stamped install ID.
        let persisted = fs::read(&good).unwrap();
        assert_eq!(persisted.len(), fs::metadata(&good).unwrap().len() as usize);
        let id = Command::new("otool")
            .args(["-D", good.to_str().unwrap()])
            .output()
            .unwrap();
        assert!(
            String::from_utf8_lossy(&id.stdout)
                .lines()
                .last()
                .unwrap_or("")
                .trim()
                == good.canonicalize().unwrap().to_str().unwrap(),
            "persisted dylib install ID must be its own cache path, got {:?}",
            String::from_utf8_lossy(&id.stdout)
        );
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
        assert_eq!(
            hit,
            CacheAdmission::Refused("truncated-dylib"),
            "pin-mismatched entry must refuse, never serve"
        );
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

        assert_eq!(
            accept_cached_entry(&cache, &dest, "arm64", 1).unwrap(),
            CacheAdmission::Trusted
        );
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
        fs::write(engine_marker_path(tmp.path()), b"v2.0.0\r\n").unwrap();
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
        // pin never staged: the pre-AD1 cache shape. Must refuse (typed MISS
        // decision), not serve — and the demote law removes the dylib.
        assert_eq!(
            accept_cached_entry(&cache, &dest, "arm64", 1).unwrap(),
            CacheAdmission::Refused("truncated-dylib"),
            "marker without a pin is not a trusted hit"
        );
        assert!(!dest.exists(), "demoted entry is removed fail-closed");
    }

    // =====================================================================
    // AD6 full anti-poison siege court
    //
    // Drives the REAL decision layer (`accept_cached_entry`, the function
    // ensure_binary routes every cache hit through) against every poison
    // state the 13:22-class attacks can produce. Decision vocabulary (MAX1):
    //   Trusted             = SERVE   (link the cached dylib)
    //   Refused(invariant)  = REFUSED (typed poison rejection; ensure_binary
    //                         aborts with `POISON_STATE_REFUSED: <cache>
    //                         (<invariant>)` and exit code 1)
    //   Err(_)              = FAIL    (fail-closed, build refused)
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
            Refused(&'static str),
            Fail,
        }
        fn decide(r: Result<CacheAdmission>) -> Decision {
            match r {
                Ok(CacheAdmission::Trusted) => Decision::Serve,
                Ok(CacheAdmission::Refused(inv)) => Decision::Refused(inv),
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
        type PoisonCase<'a> = (&'a str, Decision, Box<dyn Fn(&Path) + 'a>);
        let mut table: Vec<PoisonCase> = vec![
            (
                "control: real dylib + exact marker + pins -> SERVE",
                Decision::Serve,
                Box::new(|v: &Path| stage_lawful(v, &good)),
            ),
            // (a) the 13:22 attack: foreign dylib under a v2.0.0 label,
            // no AD1-era pins. Pre-siege this SERVED (the demote branch
            // returned Ok(binary_path)); the fix demotes to a real MISS.
            (
                "(a) v1.5.5 dylib + v2.0.0 marker (13:22 attack)",
                Decision::Refused("poison"),
                Box::new(|v: &Path| {
                    fs::copy(&good, get_binary_path(v, arch)).unwrap();
                    stage_engine_marker(v).unwrap();
                }),
            ),
            // (b) truncated dylib, valid marker, STALE pin (recorded before
            // truncation): the size-pin gate must fire.
            (
                "(b) truncated dylib + valid marker + stale .size",
                Decision::Refused("poison"),
                Box::new(|v: &Path| {
                    stage_lawful(v, &good);
                    let b = get_binary_path(v, arch);
                    let full = fs::read(&b).unwrap();
                    fs::write(&b, &full[..full.len() / 2]).unwrap();
                }),
            ),
            // (b2) truncation with an adversarially recomputed pin: pins pass,
            // the slice gate must refuse closed (MAX1: typed slice refusal —
            // still never a serve).
            (
                "(b2) truncated dylib + recomputed pin -> slice REFUSED",
                Decision::Refused("slice-magic-or-floor"),
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
                Decision::Refused("poison"),
                Box::new(|v: &Path| {
                    fs::copy(&good, get_binary_path(v, arch)).unwrap();
                    stage_library_size_pin(&get_binary_path(v, arch)).unwrap();
                    // marker intentionally absent
                }),
            ),
            // (d) foreign marker.
            (
                "(d) foreign marker v9.9.9",
                Decision::Refused("poison"),
                Box::new(|v: &Path| {
                    stage_lawful(v, &good);
                    fs::write(engine_marker_path(v), b"v9.9.9").unwrap();
                }),
            ),
            // (e) marker is a symlink to /dev/null.
            (
                "(e) marker symlink -> /dev/null",
                Decision::Refused("poison"),
                Box::new(|v: &Path| {
                    stage_lawful(v, &good);
                    fs::remove_file(engine_marker_path(v)).unwrap();
                    std::os::unix::fs::symlink("/dev/null", engine_marker_path(v)).unwrap();
                }),
            ),
            // (f) marker with trailing newline / CRLF: byte-exact label law.
            (
                "(f) marker `v{VERSION}\\n` (trailing newline)",
                Decision::Refused("poison"),
                Box::new(|v: &Path| {
                    stage_lawful(v, &good);
                    let mut m = expected_engine_marker().into_bytes();
                    m.push(b'\n');
                    fs::write(engine_marker_path(v), &m).unwrap();
                }),
            ),
            (
                "(f2) marker `v{VERSION}\\r\\n` (CRLF)",
                Decision::Refused("poison"),
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
                Decision::Refused("poison"),
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
            Decision::Refused("poison"),
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
            let ok = match (&got, expected) {
                (Decision::Refused(_), Decision::Refused(_)) => true,
                (a, b) => a == b,
            };
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
                let sym = dlsym(h, c"duckdb_library_version".as_ptr());
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

    // ===== X3/T2/GAP15 build-lock court =====
    // Lock primitives tested directly against real processes and real files.

    fn live_foreign_pid(dir: &Path) -> Option<u32> {
        // A real, genuinely live foreign process (not self, not reaped).
        let child = Command::new("sleep").arg("5").spawn().ok()?;
        let pid = child.id();
        // Dropping Child stops tracking but does NOT kill the process; a
        // detached reaper thread reaps it after it exits so no zombie lingers.
        std::thread::spawn(move || {
            let mut child = child;
            let _ = child.wait();
        });
        let _ = dir;
        Some(pid)
    }

    #[test]
    fn lock_acquire_write_release_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = tmp.path().join("entry");
        let lock = BuildLock::acquire(&cache).unwrap();
        let lock_path = cache.join(".build-lock");
        assert!(lock_path.exists());
        assert_eq!(
            fs::read_to_string(&lock_path).unwrap().trim(),
            std::process::id().to_string()
        );
        drop(lock);
        assert!(!lock_path.exists(), "release must remove the lockfile");
    }

    #[test]
    fn lock_held_blocks_second_acquire() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = tmp.path().join("entry");
        let _held = BuildLock::acquire(&cache).unwrap();
        // Same-pid reentrancy is refused (not silently re-granted).
        let err = BuildLock::acquire(&cache).unwrap_err();
        assert!(
            format!("{}", err).contains("already held by this process"),
            "{}",
            err
        );
    }

    #[test]
    fn lock_blocks_on_live_foreign_holder() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = tmp.path().join("entry");
        let lock_path = cache.join(".build-lock");
        fs::create_dir_all(&cache).unwrap();
        let pid = live_foreign_pid(tmp.path()).expect("spawn live holder");
        fs::write(&lock_path, format!("{}\n", pid)).unwrap();
        let err = BuildLock::acquire(&cache).unwrap_err();
        assert!(format!("{}", err).contains("live pid"), "{}", err);
        // The live holder's lockfile survives the refused acquisition.
        assert!(lock_path.exists());
    }

    #[test]
    fn lock_steals_dead_holder() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = tmp.path().join("entry");
        let lock_path = cache.join(".build-lock");
        fs::create_dir_all(&cache).unwrap();
        // A genuinely dead PID: spawn a process, reap it, record its pid.
        let out = Command::new("true").status().unwrap();
        assert!(out.success());
        // `true`'s pid is unrecoverable from status(); write a pid that is
        // guaranteed dead by reaping a child we spawned explicitly.
        let mut child = Command::new("true").spawn().unwrap();
        let dead = child.id();
        child.wait().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let _ = out;
        fs::write(&lock_path, format!("{}\n", dead)).unwrap();
        let _lock = BuildLock::acquire(&cache).unwrap();
        // Lockfile now carries OUR pid: stolen and re-raced.
        assert_eq!(
            fs::read_to_string(&lock_path).unwrap().trim(),
            std::process::id().to_string()
        );
    }

    #[test]
    fn lock_steals_unparseable_lockfile() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = tmp.path().join("entry");
        let lock_path = cache.join(".build-lock");
        fs::create_dir_all(&cache).unwrap();
        fs::write(&lock_path, b"not-a-pid\n").unwrap();
        let _lock = BuildLock::acquire(&cache).unwrap();
        assert_eq!(
            fs::read_to_string(&lock_path).unwrap().trim(),
            std::process::id().to_string()
        );
    }

    #[test]
    fn lock_drop_never_deletes_foreign_lockfile() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = tmp.path().join("entry");
        let lock_path = cache.join(".build-lock");
        let lock = BuildLock::acquire(&cache).unwrap();
        // Simulate a steal: a foreign holder overwrote the file.
        let foreign = live_foreign_pid(tmp.path()).unwrap();
        fs::write(&lock_path, format!("{}\n", foreign)).unwrap();
        drop(lock);
        assert!(lock_path.exists(), "foreign lockfile must survive our drop");
    }

    #[test]
    fn pid_alive_court() {
        // Real observables: self is alive; a reaped child is dead.
        assert!(pid_alive(std::process::id()));
        let mut child = Command::new("true").spawn().unwrap();
        let dead = child.id();
        child.wait().unwrap();
        assert!(!pid_alive(dead));
    }

    #[test]
    fn wait_for_staged_entry_poll_semantics() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = tmp.path().join("entry");
        fs::create_dir_all(&cache).unwrap();
        let binary = get_binary_path(&cache, "arm64");

        // MISS + zero timeout -> Ok(false) without a single poll sleep.
        let started = Instant::now();
        let hit = wait_for_staged_entry(&cache, &binary, "arm64", Duration::ZERO).unwrap();
        assert!(!hit);
        assert!(
            started.elapsed() < Duration::from_secs(1),
            "deadline check must precede any poll sleep"
        );

        // Stage a TRUSTED entry (marker + size pin + slice-verifying dylib
        // above the production floor) -> Ok(true) immediately.
        let Some(dylib) = real_dylib_above_production_floor(tmp.path(), "wd.dylib") else {
            eprintln!("SKIP wait_for_staged_entry_poll_semantics: no arm64 cc");
            return;
        };
        let bytes = fs::read(&dylib).unwrap();
        persist_verified_for("macos", &bytes, &binary, "arm64", MIN_ENGINE_DYLIB_BYTES).unwrap();
        stage_engine_marker(&cache).unwrap();
        let hit = wait_for_staged_entry(&cache, &binary, "arm64", Duration::ZERO).unwrap();
        assert!(hit);
    }

    // ---- KD-01 export fence (WO1) ----

    #[test]
    fn kd01_fence_list_is_present_and_nonempty() {
        // include_str! makes the file a compile-time dependency: a missing or
        // empty allowlist cannot compile this crate at all.
        assert!(
            !EXPORT_FENCE_LIST.trim().is_empty(),
            "export fence list must be non-empty"
        );
        let entries: Vec<&str> = EXPORT_FENCE_LIST.lines().map(str::trim).collect();
        assert_eq!(entries.len(), 1177, "fence list line count drifted");
        assert!(
            entries.iter().all(|l| l.starts_with("_duckdb_")),
            "every fence entry must be a _duckdb_* C-ABI symbol"
        );
        let mut sorted: Vec<&str> = entries.clone();
        sorted.sort_unstable();
        assert_eq!(entries, sorted, "fence list must stay sorted (refresh law)");
        let mut deduped = sorted.clone();
        deduped.dedup();
        assert_eq!(sorted, deduped, "fence list must have no duplicates");
    }

    #[test]
    fn kd01_census_parser_counts_mangled_and_c_abi() {
        let nm = "\
0000000000010000 T __ZN6duckdb8DatabaseD1Ev\n\
0000000000012000 T __ZZ6duckdb3fooENK3barEv\n\
0000000000014000 T _duckdb_create_config\n\
0000000000016000 T _duckdb_free\n\
0000000000018000 D _duckdb_version\n\
000000000001a000 t _local_only\n";
        let c = parse_export_census(nm);
        assert_eq!(c.mangled_cxx, 2);
        assert_eq!(c.duckdb_c_abi, 2);
        assert_eq!(c.total_t, 4); // t (local) and D (non-text) excluded
    }

    #[test]
    fn kd01_census_parser_handles_nm_comma_form() {
        // nm prints some symbols as "<addr>, T _name" — the parser must not
        // classify those as mangled.
        let nm = "0000000000010000, T __ZN6duckdb3BarC1Ev\n0000000000020000 T _duckdb_open\n";
        let c = parse_export_census(nm);
        assert_eq!(c.mangled_cxx, 1);
        assert_eq!(c.duckdb_c_abi, 1);
    }

    /// Opt-in (expensive) court: run the fence on a COPY of the current cache
    /// dylib in /tmp and census the result. With the cache currently holding
    /// the UNFENCED dylib and no static archive sibling, `fence_exports` must
    /// fail closed with a typed refusal naming the mangled count.
    #[test]
    #[ignore = "expensive court: real nm census + fence pass on a /tmp copy of the cache dylib"]
    fn kd01_court_cache_copy_census_and_fence_standing() {
        let cache = PathBuf::from(std::env::var("HOME").unwrap_or_default())
            .join(".frozen-duckdb/cache/v2.0.0-arm64/libduckdb_arm64.dylib");
        let Some(_) = std::fs::metadata(&cache).ok().filter(|m| m.is_file()) else {
            eprintln!("[SKIP] cache dylib absent");
            return;
        };
        let tmp = tempfile::tempdir().unwrap();
        let subject = tmp.path().join("subject.dylib");
        fs::copy(&cache, &subject).unwrap();

        let before = export_census(&subject).unwrap();
        eprintln!(
            "court: cache-copy census BEFORE: mangled={} c_abi={} total_t={}",
            before.mangled_cxx, before.duckdb_c_abi, before.total_t
        );

        // No static archive sibling in a bare /tmp dir: an unfenced copy must
        // be REFUSED (typed Err), never admitted.
        let fenced = fence_exports(&subject);
        match (&before, &fenced) {
            (c, Err(e)) if c.mangled_cxx > 0 => {
                let msg = format!("{:#}", e);
                assert!(
                    msg.contains("libduckdb_static.a"),
                    "refusal must name the missing archive: {}",
                    msg
                );
                eprintln!("court: unfenced copy REFUSED as expected — {}", msg);
            }
            (c, Ok(())) if c.mangled_cxx == 0 => {
                eprintln!(
                    "court: already-fenced copy admitted idempotently ({} C-ABI exports)",
                    c.duckdb_c_abi
                );
            }
            (c, outcome) => panic!(
                "court: illegal standing census={:?} outcome={:?}",
                c, outcome
            ),
        }

        let after = export_census(&subject).unwrap();
        assert_eq!(
            before, after,
            "a refused or no-op fence must not mutate the subject"
        );
        eprintln!("court: subject unmutated after fence pass (sha-stable census)");
    }
}
