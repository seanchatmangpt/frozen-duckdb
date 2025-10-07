use std::env;
use std::path::Path;

fn main() {
    // Set up frozen DuckDB binary for fast builds
    if let Err(e) = setup_duckdb_binary() {
        eprintln!("Warning: Failed to setup DuckDB binary: {}", e);
        eprintln!("Falling back to bundled DuckDB compilation");
    }
}

/// Setup DuckDB binary using architecture detection and environment setup
fn setup_duckdb_binary() -> Result<(), Box<dyn std::error::Error>> {
    // Check if environment is already configured (e.g., by setup_env.sh)
    if env::var("DUCKDB_LIB_DIR").is_ok() && env::var("DUCKDB_INCLUDE_DIR").is_ok() {
        let lib_dir = env::var("DUCKDB_LIB_DIR")?;
        let include_dir = env::var("DUCKDB_INCLUDE_DIR")?;

        // Use the configured paths
        println!("cargo:rustc-env=DUCKDB_LIB_DIR={}", lib_dir);
        println!("cargo:rustc-env=DUCKDB_INCLUDE_DIR={}", include_dir);

        // Tell rustc where to find the DuckDB library and headers
        println!("cargo:rustc-link-search=native={}", lib_dir);
        println!("cargo:rustc-link-lib=dylib=duckdb");
        println!("cargo:include={}", include_dir);

        // Set environment variables that persist for dependent crates
        println!("cargo:DUCKDB_LIB_DIR={}", lib_dir);
        println!("cargo:DUCKDB_INCLUDE_DIR={}", include_dir);

        println!("cargo:rerun-if-env-changed=DUCKDB_LIB_DIR");
        println!("cargo:rerun-if-env-changed=DUCKDB_INCLUDE_DIR");

        return Ok(());
    }

    // If not configured, try to find prebuilt binaries in the prebuilt directory
    let prebuilt_dir = Path::new("prebuilt");

    if prebuilt_dir.exists() {
        let lib_dir = prebuilt_dir;
        let include_dir = prebuilt_dir;

        // Set environment variables for this build and all dependent builds
        println!("cargo:rustc-env=DUCKDB_LIB_DIR={}", lib_dir.display());
        println!("cargo:rustc-env=DUCKDB_INCLUDE_DIR={}", include_dir.display());

        // Tell rustc where to find the DuckDB library and headers
        println!("cargo:rustc-link-search=native={}", lib_dir.display());
        println!("cargo:rustc-link-lib=dylib=duckdb");
        println!("cargo:include={}", include_dir.display());

        // Set environment variables that persist for dependent crates
        println!("cargo:DUCKDB_LIB_DIR={}", lib_dir.display());
        println!("cargo:DUCKDB_INCLUDE_DIR={}", include_dir.display());

        println!("cargo:rerun-if-env-changed=DUCKDB_LIB_DIR");
        println!("cargo:rerun-if-env-changed=DUCKDB_INCLUDE_DIR");

        return Ok(());
    }

    // If no prebuilt binaries found, let the dependent crates handle it
    // (they will fall back to bundled compilation)
    Ok(())
}
