# Test Dependency Crate

This crate exercises `frozen-duckdb` as a path dependency, and demonstrates the
difference in setup requirements versus the upstream `duckdb` crate.

## Purpose

This test crate simulates two real-world scenarios:
1. A project depends on `frozen-duckdb` — the bundled builder acquires the prebuilt
   dylib automatically (no environment needed)
2. A project migrating from the upstream `duckdb` crate compares the API side by
   side. The upstream dependency is available only behind the opt-in
   `upstream-compat` feature (`cargo build --features upstream-compat`); it is
   **default off**, because `frozen-duckdb-sys` and libduckdb-sys both declare
   `links = "duckdb"`, so dual-engine coexistence in one binary is structurally
   blocked.

Note: current `frozen-duckdb` requires **no environment setup** —
`frozen-duckdb-sys`'s build script runs `ensure_binary()` (GitHub Release
download, vendored headers, emitted runtime `@rpath`). The `DUCKDB_LIB_DIR` /
`DUCKDB_INCLUDE_DIR` checks in the sample program are informational diagnostics for
the legacy manual-prebuilt workflow, not a build requirement.

## Usage

### Build (default: frozen-duckdb only)
```bash
cd ../
cargo build
```

### Build with the upstream duckdb crate for migration testing
```bash
cargo build --features upstream-compat
```

### Run the test application
```bash
cargo run
```

## Expected Behavior

- **frozen-duckdb dependency**: uses the prebuilt binary (fast, no env vars)
- **upstream `duckdb` dependency** (`upstream-compat` feature): compiles bundled
  DuckDB from source (slow); never linkable alongside frozen-duckdb in one binary
  (`links = "duckdb"` collision)

## Testing

```bash
cargo test
```

This exercises the frozen-duckdb path-dependency build end to end (binary acquisition, linking, and runtime loading via the emitted `@rpath`).
