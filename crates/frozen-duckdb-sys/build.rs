use std::{env, fs, path::Path, path::PathBuf};

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
    // docs.rs survival: docs.rs builds on Linux where no libduckdb .so asset
    // exists (the GitHub release download 404s pre-tag), so ensure_binary()
    // would either fail outright or fall into a 30+ minute source compile.
    // The standard DOCS_RS pattern instead: generate bindings from the
    // builder's vendored headers and emit no linking directives, so rustdoc
    // can typecheck without any library present.
    if env::var("DOCS_RS").is_ok() {
        docs_rs_main();
        return;
    }

    // Ensure the frozen DuckDB mega-library is available
    let binary_path =
        frozen_duckdb_builder::ensure_binary().expect("Failed to get frozen DuckDB binary");

    // Get the directory containing the binary and headers
    let lib_dir = binary_path
        .parent()
        .expect("Binary path has no parent directory");

    // Tell rustc where to find the library
    println!("cargo:rustc-link-search=native={}", lib_dir.display());

    // Link against the DuckDB library
    println!("cargo:rustc-link-lib=dylib=duckdb");

    // Runtime resolution too: link-time -L does not survive to load time. The
    // unscoped rustc-link-arg applies to this crate's own test harness as well
    // (its lib unittest otherwise aborts in the loader with
    // "Library not loaded: @rpath/libduckdb.dylib").
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_dir.display());

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
    // Declared in BOTH branches: whichever mode's output is cached, a mode
    // switch (DOCS_RS appearing or disappearing) must re-run this script.
    // Declaring it only in the docs branch lets a cached normal output mask
    // the switch, silently reusing non-docs state (and vice versa).
    println!("cargo:rerun-if-env-changed=DOCS_RS");

    println!(
        "cargo:warning=Using prebuilt DuckDB binary: {}",
        binary_path.display()
    );
}

/// docs.rs path: bindings from the vendored headers, zero linking directives.
///
/// Never calls `ensure_binary()`: on docs.rs there is no release asset (404
/// pre-tag), so that path would fail or trigger a 30+ minute source compile —
/// and even on success it writes `~/.frozen-duckdb`. The builder crate ships
/// the pinned headers inside its package (`vendored-headers/`, present both in
/// this workspace checkout and in its crates.io `.crate` archive), so bindings
/// are generated straight from those, staged under OUT_DIR because registry
/// sources are read-only. rustdoc typechecks the FFI declarations without any
/// library, which is exactly what a docs build needs.
fn docs_rs_main() {
    let vendored = find_vendored_headers()
        .expect("docs.rs build: cannot locate frozen-duckdb-builder vendored-headers");

    // Stage as <OUT_DIR>/headers/duckdb/duckdb.h, mirroring the cache layout
    // that wrapper.h's `#include "duckdb/duckdb.h"` expects via -I.
    let out_dir = env::var("OUT_DIR").unwrap();
    let include_dir = Path::new(&out_dir).join("headers");
    let staged = include_dir.join("duckdb");
    fs::create_dir_all(&staged).expect("Failed to create staged docs headers directory");
    for header_name in ["duckdb.h", "duckdb.hpp"] {
        let src = vendored.join(header_name);
        let contents = fs::read(&src).unwrap_or_else(|e| {
            panic!(
                "docs.rs build: missing vendored header {}: {}",
                src.display(),
                e
            )
        });
        fs::write(staged.join(header_name), contents)
            .unwrap_or_else(|e| panic!("docs.rs build: failed to stage {}: {}", header_name, e));
    }

    let out_path = Path::new(&out_dir).join("bindgen.rs");
    build_linked::main(&out_dir, &out_path, &include_dir);

    // Metadata for dependents: frozen-duckdb's build.rs hard-requires
    // DEP_DUCKDB_DUCKDB_LIB_DIR. rustdoc never links, so the rpath argument
    // dependents emit from this value is inert during a docs build.
    println!("cargo:DUCKDB_LIB_DIR={}", include_dir.display());
    println!("cargo:DUCKDB_INCLUDE_DIR={}", include_dir.display());

    // Deliberately NO rustc-link-search / rustc-link-lib / rustc-link-arg:
    // the docs build must succeed with no library present anywhere.

    println!("cargo:rerun-if-env-changed=DOCS_RS");
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed={}", vendored.display());

    println!(
        "cargo:warning=docs.rs build: bindings generated from vendored headers ({}), no library linked",
        vendored.display()
    );
}

/// Locate the builder crate's `vendored-headers/` directory without touching
/// `$HOME` or the network. Order: workspace/path checkout first, then the
/// crates.io registry extraction that docs.rs builds use.
fn find_vendored_headers() -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    // Workspace checkout: the builder crate is a sibling of this crate.
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        candidates.push(Path::new(&manifest_dir).join("../frozen-duckdb-builder/vendored-headers"));
    }

    // crates.io registry (docs.rs): build-dependencies are unpacked under
    // $CARGO_HOME/registry/src/<registry>/frozen-duckdb-builder-<version>/.
    let cargo_home = env::var_os("CARGO_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| Path::new(&home).join(".cargo")));
    if let Some(home) = cargo_home {
        if let Ok(registries) = fs::read_dir(home.join("registry").join("src")) {
            for registry in registries.flatten() {
                if let Ok(versions) = fs::read_dir(registry.path()) {
                    for version in versions.flatten() {
                        if version
                            .file_name()
                            .to_str()
                            .is_some_and(|name| name.starts_with("frozen-duckdb-builder-"))
                        {
                            candidates.push(version.path().join("vendored-headers"));
                        }
                    }
                }
            }
        }
    }

    candidates
        .into_iter()
        .find(|dir| dir.join("duckdb.h").exists())
}

// No bundled build path exists: this crate always links the frozen prebuilt
// dylib. The legacy `bundled` feature (still declared in Cargo.toml) must stay
// inert so `--all-features` remains a no-op — hence no `cfg(feature)` gate here.
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
            // wrapper.h already includes "duckdb/duckdb.h" via -I; adding the header
            // again here double-includes it and trips C redefinition errors
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
