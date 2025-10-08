use std::{env, path::Path};

/// Tells whether we're building for Windows. This is more suitable than a plain
/// `cfg!(windows)`, since the latter does not properly handle cross-compilation
///
/// Note that there is no way to know at compile-time which system we'll be
/// targeting, and this test must be made at run-time (of the build script) See
/// https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-build-scripts
#[allow(dead_code)]
fn win_target() -> bool {
    std::env::var("CARGO_CFG_WINDOWS").is_ok()
}

/// Tells whether a given compiler will be used `compiler_name` is compared to
/// the content of `CARGO_CFG_TARGET_ENV` (and is always lowercase)
///
/// See [`win_target`]
#[allow(dead_code)]
fn is_compiler(compiler_name: &str) -> bool {
    std::env::var("CARGO_CFG_TARGET_ENV").is_ok_and(|v| v == compiler_name)
}

fn main() {
    // Ensure the frozen DuckDB mega-library is available
    let binary_path = frozen_duckdb_builder::ensure_binary()
        .expect("Failed to get frozen DuckDB binary");

    // Get the directory containing the binary and headers
    let lib_dir = binary_path.parent()
        .expect("Binary path has no parent directory");

    // Tell rustc where to find the library
    println!("cargo:rustc-link-search=native={}", lib_dir.display());

    // Link against the DuckDB library
    println!("cargo:rustc-link-lib=dylib=duckdb");

    // Set environment variables for dependent crates
    println!("cargo:DUCKDB_LIB_DIR={}", lib_dir.display());
    println!("cargo:DUCKDB_INCLUDE_DIR={}", lib_dir.display());

    // Generate bindings using the headers from the builder
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir).join("bindgen.rs");

    // Use the linked build approach with the headers from the builder
    build_linked::main(&out_dir, &out_path, lib_dir);

    // Re-run if the binary changes
    println!("cargo:rerun-if-changed={}", binary_path.display());

    // Re-run if environment variables change
    println!("cargo:rerun-if-env-changed=DUCKDB_LIB_DIR");
    println!("cargo:rerun-if-env-changed=DUCKDB_INCLUDE_DIR");

    println!("cargo:warning=Using prebuilt DuckDB binary: {}", binary_path.display());
}

#[cfg(not(feature = "bundled"))]
mod build_linked {
    use std::path::Path;

    use super::{bindings, HeaderLocation};

    pub fn main(_out_dir: &str, out_path: &Path, lib_dir: &Path) {
        // Use the frozen library directory as include directory
        let header = HeaderLocation::FromPath(lib_dir.to_string_lossy().to_string());
        bindings::write_to_out_dir(header, out_path);
    }
}

mod bindings {
    use std::path::Path;

    use super::HeaderLocation;

    pub fn write_to_out_dir(header: HeaderLocation, out_path: &Path) {
        let bindings = bindgen::Builder::default()
            .header("wrapper.h")
            // Skip wrapper_ext.h for now as it requires unstable extension API headers
            // .header("wrapper_ext.h")
            .header(header.path() + "/duckdb/duckdb.h")
            .clang_arg(format!("-I{}", header.path()))
            .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
            .generate()
            .expect("Unable to generate bindings");

        bindings
            .write_to_file(out_path)
            .expect("Couldn't write bindings!");
    }
}

enum HeaderLocation {
    FromPath(String),
}

impl HeaderLocation {
    fn path(&self) -> String {
        match self {
            HeaderLocation::FromPath(path) => path.clone(),
        }
    }
}
