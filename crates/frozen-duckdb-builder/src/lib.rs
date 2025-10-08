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

const VERSION: &str = "1.4.0";
const CACHE_DIR: &str = ".frozen-duckdb";
const BINARY_NAME: &str = "libduckdb";

/// Ensure the prebuilt DuckDB binary is available
/// 
/// This function:
/// 1. Checks for cached binary in ~/.frozen-duckdb/cache/v1.4.0-{arch}/
/// 2. If missing, tries to download from GitHub Release
/// 3. If download fails, compiles locally as fallback
/// 4. Returns path to the binary
pub fn ensure_binary() -> Result<PathBuf> {
    let arch = detect_architecture()?;
    let cache_dir = get_cache_dir()?;
    let versioned_cache = cache_dir.join(format!("v{}-{}", VERSION, arch));
    let binary_path = get_binary_path(&versioned_cache, &arch);

    // Check if we already have a cached binary
    if binary_path.exists() {
        info!("Using cached DuckDB binary: {}", binary_path.display());
        return Ok(binary_path);
    }

    // Check if prebuilt binary exists in project directory
    if let Ok(prebuilt_path) = check_prebuilt_binary(&arch) {
        info!("Found prebuilt binary, copying to cache: {}", prebuilt_path.display());
        copy_prebuilt_to_cache(&prebuilt_path, &binary_path)?;
        info!("Successfully set up prebuilt binary and headers");
        return Ok(binary_path);
    }

    info!("No cached binary found at: {}", binary_path.display());
    info!("Cache directory: {}", versioned_cache.display());

    // Debug: show what's in the cache directory
    if let Ok(entries) = fs::read_dir(&versioned_cache) {
        info!("Versioned cache directory contents:");
        for entry in entries {
            if let Ok(entry) = entry {
                info!("  {}", entry.path().display());
            }
        }
    }

    info!("Attempting to download...");
    
    // Try to download from GitHub Release
    match download_from_github_release(&versioned_cache, &arch) {
        Ok(path) => {
            info!("Successfully downloaded frozen DuckDB binary: {}", path.display());
            return Ok(path);
        }
        Err(e) => {
            warn!("Failed to download from GitHub Release: {}", e);
            info!("Falling back to local compilation...");
        }
    }
    
    // Fallback to local compilation
    let path = compile_duckdb_locally(&versioned_cache, &arch)
        .context("Failed to compile DuckDB locally")?;
    
    info!("Successfully compiled DuckDB binary: {}", path.display());
    Ok(path)
}

/// Check if prebuilt binary exists in project directory
fn check_prebuilt_binary(arch: &str) -> Result<PathBuf> {
    // Try to find the project root by looking for prebuilt directory
    let current_dir = env::current_dir()
        .context("Failed to get current directory")?;

    let prebuilt_dir = current_dir.join("prebuilt");
    if !prebuilt_dir.exists() {
        anyhow::bail!("Prebuilt directory not found: {}", prebuilt_dir.display());
    }

    let binary_name = format!("libduckdb_{}.dylib", arch);
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
        fs::create_dir_all(parent)
            .context("Failed to create cache directory")?;
    }

    // Copy the binary
    fs::copy(prebuilt_path, cache_path)
        .context("Failed to copy prebuilt binary to cache")?;

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

    info!("Copied prebuilt binary and headers to cache: {}", cache_path.display());
    Ok(())
}

/// Copy prebuilt headers to cache directory
fn copy_prebuilt_headers(cache_path: &Path) -> Result<()> {
    let current_dir = env::current_dir()
        .context("Failed to get current directory")?;

    let prebuilt_dir = current_dir.join("prebuilt");
    let headers_dest = cache_path.parent().unwrap();

    // Copy header files directly to cache directory (expected by bindgen)
    let header_files = ["duckdb.h", "duckdb.hpp"];
    for header_name in &header_files {
        let src_path = prebuilt_dir.join(header_name);
        if src_path.exists() {
            let dest_path = headers_dest.join(header_name);
            fs::copy(&src_path, &dest_path)
                .context("Failed to copy header file")?;
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
    let home = env::var("HOME")
        .context("HOME environment variable not set")?;
    
    let cache_dir = Path::new(&home).join(CACHE_DIR).join("cache");
    fs::create_dir_all(&cache_dir)
        .context("Failed to create cache directory")?;
    
    Ok(cache_dir)
}

/// Get the expected binary path for the given architecture
fn get_binary_path(cache_dir: &Path, arch: &str) -> PathBuf {
    let extension = if cfg!(target_os = "macos") {
        "dylib"
    } else if cfg!(target_os = "linux") {
        "so"
    } else if cfg!(target_os = "windows") {
        "dll"
    } else {
        "so" // Default fallback
    };
    
    cache_dir.join(format!("{}_{}.{}", BINARY_NAME, arch, extension))
}

/// Download prebuilt binary from GitHub Release
fn download_from_github_release(cache_dir: &Path, arch: &str) -> Result<PathBuf> {
    let binary_path = get_binary_path(cache_dir, arch);
    let url = format!(
        "https://github.com/seanchatmangpt/frozen-duckdb/releases/download/v{}/libduckdb_{}.dylib",
        VERSION, arch
    );
    
    info!("Downloading from: {}", url);
    
    // Create cache directory
    fs::create_dir_all(cache_dir)
        .context("Failed to create cache directory")?;
    
    // Download the binary
    let response = reqwest::blocking::get(&url)
        .context("Failed to download binary from GitHub Release")?;
    
    if !response.status().is_success() {
        anyhow::bail!("HTTP error: {}", response.status());
    }
    
    let content = response.bytes()
        .context("Failed to read response body")?;
    
    fs::write(&binary_path, content)
        .context("Failed to write downloaded binary")?;
    
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
    fs::create_dir_all(cache_dir)
        .context("Failed to create cache directory")?;

    // Create temporary directory for compilation
    let temp_dir = tempfile::tempdir()
        .context("Failed to create temporary directory")?;

    let temp_path = temp_dir.path();

    // Clone DuckDB source
    info!("Cloning DuckDB source...");
    let duckdb_dir = temp_path.join("duckdb");

    Command::new("git")
        .args(["clone", "--depth", "1", "--branch", "v1.4.0", "https://github.com/duckdb/duckdb.git"])
        .arg(&duckdb_dir)
        .current_dir(temp_path)
        .output()
        .context("Failed to clone DuckDB repository")?;

    // Build DuckDB with all features
    info!("Building DuckDB with all features...");
    let build_dir = duckdb_dir.join("build");
    fs::create_dir_all(&build_dir)
        .context("Failed to create build directory")?;

    // Configure with CMake - enable all extensions
    Command::new("cmake")
        .args([
            "..",
            "-DCMAKE_BUILD_TYPE=Release",
            "-DBUILD_EXTENSIONS=ON",
            "-DBUILD_PARQUET=ON",
            "-DBUILD_JSON=ON",
            "-DBUILD_ICU=ON",
            "-DBUILD_HTTPFS=ON",
            "-DBUILD_VISUALIZER=ON",
            "-DBUILD_TPCH=ON",
            "-DBUILD_TPCDS=ON",
            "-DBUILD_FTS=ON",
            "-DBUILD_INET=ON",
            "-DBUILD_EXCEL=ON",
            "-DBUILD_SQLSMITH=ON",
            "-DBUILD_TPCE=ON",
            "-DBUILD_JEMALLOC=ON",
            "-DBUILD_AUTOLOAD=ON",
            "-DBUILD_ARROW=ON",
            "-DBUILD_POLARS=ON",
        ])
        .current_dir(&build_dir)
        .output()
        .context("Failed to configure DuckDB with CMake")?;

    // Build with all available cores (use 4 as default)
    Command::new("make")
        .args(["-j4"])
        .current_dir(&build_dir)
        .output()
        .context("Failed to build DuckDB")?;

    // Find the built library
    let built_lib = find_built_library(&build_dir, arch)
        .context("Failed to find built library")?;

    // Copy library to cache directory with proper name
    let binary_path = get_binary_path(cache_dir, arch);
    fs::copy(&built_lib, &binary_path)
        .context("Failed to copy built library to cache")?;

    // Also copy header files for FFI bindings generation
    let headers_dir = cache_dir.join("include");
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

/// Ensure header files are available for FFI bindings

/// Find the built library in the build directory
fn find_built_library(build_dir: &Path, _arch: &str) -> Result<PathBuf> {
    // Look for the main DuckDB library - check multiple possible locations
    let possible_paths = [
        // Release build location (most common)
        build_dir.join("src").join("libduckdb.dylib"),
        build_dir.join("src").join("libduckdb.so"),
        build_dir.join("src").join("libduckdb.dll"),
        // Alternative locations
        build_dir.join("libduckdb.dylib"),
        build_dir.join("libduckdb.so"),
        build_dir.join("libduckdb.dll"),
        // Sometimes it's in a subdirectory
        build_dir.join("src").join("Release").join("libduckdb.dylib"),
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

    anyhow::bail!("Could not find built DuckDB library in {:?}. Tried: {:?}", build_dir, possible_paths);
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
}
