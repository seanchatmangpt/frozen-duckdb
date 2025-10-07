use std::env;
use std::path::Path;

fn main() {
    // Check if we should use the prebuilt DuckDB
    if let Ok(lib_dir) = env::var("DUCKDB_LIB_DIR") {
        let lib_dir = Path::new(&lib_dir);
        let include_dir = env::var("DUCKDB_INCLUDE_DIR")
            .map(|p| Path::new(&p).to_path_buf())
            .unwrap_or_else(|_| lib_dir.join("include"));

        // Tell rustc where to find the DuckDB library and headers
        println!("cargo:rustc-link-search=native={}", lib_dir.display());
        println!("cargo:rustc-link-lib=dylib=duckdb");
        println!("cargo:include={}", include_dir.display());

        // This prevents the bundled build
        println!("cargo:rerun-if-env-changed=DUCKDB_LIB_DIR");
        println!("cargo:rerun-if-env-changed=DUCKDB_INCLUDE_DIR");
    } else {
        // Fall back to bundled if no prebuilt library is specified
        println!("cargo:warning=No DUCKDB_LIB_DIR specified, using bundled DuckDB");
    }
}