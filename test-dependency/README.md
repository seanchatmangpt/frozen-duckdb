# Test Dependency Crate

This crate exercises `frozen-duckdb` as a path dependency alongside the upstream
`duckdb = "1.5"` crate, and demonstrates the difference in setup requirements.

## Purpose

This test crate simulates two real-world scenarios:
1. A project depends on `frozen-duckdb` — the bundled builder acquires the prebuilt
   dylib automatically (no environment needed)
2. The same project also pulls the upstream `duckdb` crate, which compiles bundled
   DuckDB from source

Note: since frozen-duckdb 1.5.5, `frozen-duckdb` itself requires **no environment
setup** — `frozen-duckdb-sys`'s build script runs `ensure_binary()` (GitHub Release
download, vendored 1.5.5 headers, emitted runtime `@rpath`). The `DUCKDB_LIB_DIR` /
`DUCKDB_INCLUDE_DIR` checks in the sample program are informational diagnostics for
the legacy manual-prebuilt workflow, not a build requirement.

## Usage

### Build (upstream duckdb compiles from source; frozen-duckdb does not)
```bash
cd ../
cargo build
```

### Run the test application
```bash
cargo run
```

## Expected Behavior

- **frozen-duckdb dependency**: uses the prebuilt binary (fast, no env vars)
- **upstream `duckdb` dependency**: compiles bundled DuckDB from source (slow)

## Testing

```bash
cargo test
```

This exercises the frozen-duckdb path-dependency build end to end (binary acquisition, linking, and runtime loading via the emitted `@rpath`).
