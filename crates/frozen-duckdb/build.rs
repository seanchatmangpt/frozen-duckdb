use std::env;

fn main() {
    // The frozen dylib's install name is @rpath/libduckdb.dylib. Link-time
    // rpath args cannot propagate from frozen-duckdb-sys's build script, so
    // this crate (which owns the bins/tests/examples that link the dylib)
    // reads the library directory from the sys crate's DEP_ metadata and
    // emits the rpath here.
    let lib_dir = env::var("DEP_DUCKDB_DUCKDB_LIB_DIR")
        .expect("frozen-duckdb-sys must expose DEP_DUCKDB_DUCKDB_LIB_DIR (links = \"duckdb\")");

    // wasm32-wasip1 links libduckdb.a. An rpath argument is for the host
    // dylib and is not a wasm-ld flag.
    let target = env::var("TARGET").unwrap_or_default();
    let static_link = env::var("DEP_DUCKDB_DUCKDB_LINK_KIND").ok().as_deref() == Some("static");
    if static_link || target.starts_with("wasm32-") {
        println!("cargo:rerun-if-env-changed=TARGET");
        return;
    }

    // The unscoped rustc-link-arg is deliberate: the scoped variants
    // (-bins/-tests/-examples) skip the lib's unit-test binary, which then
    // dies in dyld with "Library not loaded: @rpath/libduckdb.dylib".
    // Unconditional because -Wl,-rpath is valid for both Apple's and GNU's
    // linker, and Linux CI cannot rely on LD_LIBRARY_PATH propagation.
    println!("cargo:rustc-link-arg=-Wl,-rpath,{lib_dir}");
}
