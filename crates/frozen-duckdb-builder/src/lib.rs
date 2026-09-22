//! # Frozen DuckDB Builder
//!
//! This crate handles downloading prebuilt mega-libraries from GitHub Releases
//! or compiling them locally as a fallback. It manages caching in `~/.frozen-duckdb/`
//! to ensure fast subsequent builds.

use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{debug, info, warn};

const VERSION: &str = "1.5.5";
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
/// release yet — the v1.5.5 workflow ships macOS assets only, and the Linux
/// asset set first lands with a tag AFTER v1.5.5 (TR2 coordinates the
/// workflow side). Until that tag exists, a Linux download attempt returns
/// HTTP 404 and `ensure_binary` falls back to `compile_duckdb_locally`,
/// which is the supported Linux path today.
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
/// 1. Checks for cached binary in ~/.frozen-duckdb/cache/v1.5.5-{arch}/
/// 2. If missing, tries to download from GitHub Release
/// 3. If download fails, compiles locally as fallback
/// 4. Returns path to the binary
pub fn ensure_binary() -> Result<PathBuf> {
    let arch = detect_architecture()?;
    let cache_dir = get_cache_dir()?;
    let versioned_cache = cache_dir.join(format!("v{}-{}", VERSION, arch));
    let binary_path = get_binary_path(&versioned_cache, &arch);

    if binary_path.exists() {
        info!("Using cached DuckDB binary: {}", binary_path.display());
    } else if let Ok(prebuilt_path) = check_prebuilt_binary(&arch) {
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

/// Copy the vendored headers into the cache when missing.
///
/// bindgen expects `{cache}/duckdb/duckdb.h`, but release downloads carry only
/// the dylib, so the headers for the pinned DuckDB version ship inside this
/// crate.
fn ensure_headers(versioned_cache: &Path) -> Result<()> {
    let headers_dir = versioned_cache.join("duckdb");
    if headers_dir.join("duckdb.h").exists() {
        return Ok(());
    }
    fs::create_dir_all(&headers_dir).context("Failed to create cache headers directory")?;
    for header_name in ["duckdb.h", "duckdb.hpp"] {
        let src = Path::new(VENDORED_HEADERS_DIR).join(header_name);
        if !src.exists() {
            anyhow::bail!("Vendored header missing: {}", src.display());
        }
        let dest = headers_dir.join(header_name);
        fs::copy(&src, &dest)
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
    fs::copy(prebuilt_path, cache_path).context("Failed to copy prebuilt binary to cache")?;

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
            fs::copy(&src_path, &dest_path).context("Failed to copy header file")?;
            info!("Copied header: {}", dest_path.display());
        }
    }

    Ok(())
}

/// Detect the current system architecture
fn detect_architecture() -> Result<String> {
    let output = Command::new("uname")
        .arg("-m")
        .output()
        .context("Failed to run uname command")?;

    let arch = String::from_utf8(output.stdout)
        .context("Invalid UTF-8 in uname output")?
        .trim()
        .to_string();

    match arch.as_str() {
        "x86_64" => Ok("x86_64".to_string()),
        "arm64" | "aarch64" => Ok("arm64".to_string()),
        _ => anyhow::bail!("Unsupported architecture: {}", arch),
    }
}

/// Get the cache directory (~/.frozen-duckdb)
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
    // release_asset_name_for: Linux .so assets only exist from the release
    // AFTER v1.5.5; until then Linux downloads 404 and local compile runs.
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

    fs::write(&binary_path, content).context("Failed to write downloaded binary")?;

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

    // Create cache directory
    fs::create_dir_all(cache_dir).context("Failed to create cache directory")?;

    // Create temporary directory for compilation
    let temp_dir = tempfile::tempdir().context("Failed to create temporary directory")?;

    let temp_path = temp_dir.path();

    // Clone DuckDB source
    info!("Cloning DuckDB source...");
    let duckdb_dir = temp_path.join("duckdb");

    Command::new("git")
        .args([
            "clone",
            "--depth",
            "1",
            "--branch",
            "v1.5.5",
            "https://github.com/duckdb/duckdb.git",
        ])
        .arg(&duckdb_dir)
        .current_dir(temp_path)
        .output()
        .context("Failed to clone DuckDB repository")?;

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
    if let Ok(osx_archs) = env::var("CMAKE_OSX_ARCHITECTURES") {
        let osx_archs = osx_archs.trim();
        if !osx_archs.is_empty() {
            info!("Configuring with CMAKE_OSX_ARCHITECTURES={}", osx_archs);
            configure.arg(format!("-DCMAKE_OSX_ARCHITECTURES={}", osx_archs));
        }
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

    // Copy library to cache directory with proper name
    let binary_path = get_binary_path(cache_dir, arch);
    fs::copy(&built_lib, &binary_path).context("Failed to copy built library to cache")?;

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
            fs::copy(header, dest)?;
            info!("Copied header: {}", header.display());
        }
    }

    info!("Compiled DuckDB binary to: {}", binary_path.display());
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
}
