use std::env;

fn main() {
    // The frozen dylib's install name is @rpath/libduckdb.dylib. Link-time
    // rpath args cannot propagate from frozen-duckdb-sys's build script, so
    // this crate (which owns the bins/tests/examples that link the dylib)
    // reads the library directory from the sys crate's DEP_ metadata and
    // emits the rpath here.
    let lib_dir = env::var("DEP_DUCKDB_DUCKDB_LIB_DIR")
        .expect("frozen-duckdb-sys must expose DEP_DUCKDB_DUCKDB_LIB_DIR (links = \"duckdb\")");

    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-arg-bins=-Wl,-rpath,{lib_dir}");
        println!("cargo:rustc-link-arg-tests=-Wl,-rpath,{lib_dir}");
        println!("cargo:rustc-link-arg-example=-Wl,-rpath,{lib_dir}");
        // Unqualified link-arg additionally covers the lib unittest harness
        // (`cargo test --lib`), which the bins/tests/example kinds all miss.
        println!("cargo:rustc-link-arg=-Wl,-rpath,{lib_dir}");
    }
}
