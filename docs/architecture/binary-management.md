# Binary Management Architecture

## Release Asset Strategy

Frozen DuckDB ships **prebuilt macOS dylibs as GitHub Release assets** and downloads them automatically on first build. As of DuckDB 1.5.5, each released asset is a **universal binary containing both arm64 and x86_64 slices**, so a single asset serves Apple Silicon and Intel Macs alike.

### Released Assets

| Asset | Size | Contains | Served By |
|-------|------|----------|-----------|
| `libduckdb_arm64.dylib` | ~117MB | universal (arm64 + x86_64) | Apple Silicon and Intel Macs |
| `libduckdb_x86_64.dylib` | ~117MB | universal (arm64 + x86_64) | Apple Silicon and Intel Macs |

### How the Cache Is Populated

`frozen-duckdb-builder::ensure_binary()` resolves the binary through a fixed sequence:

1. **Cache hit**: `~/.frozen-duckdb/cache/v1.5.5-{arch}/libduckdb_{arch}.dylib` (arch from `uname -m`)
2. **Local prebuilt dir**: copies `prebuilt/libduckdb_{arch}.dylib` into the cache if present
3. **GitHub Release download**: fetches `https://github.com/seanchatmangpt/frozen-duckdb/releases/download/v1.5.5/libduckdb_{arch}.dylib`
4. **Local-compile fallback**: clones upstream DuckDB at the pinned `v1.5.5` tag and builds with CMake

On **every** acquisition path, the builder then normalizes the cache layout: headers are
materialized under `duckdb/` (copied from the vendored 1.5.5 headers inside
`crates/frozen-duckdb-builder/vendored-headers/` when the asset carries none) and a plain
`libduckdb.dylib` symlink is created so `-lduckdb` resolves.

## Binary Selection

The builder detects the architecture with `uname -m` (`x86_64`, or `arm64`/`aarch64` mapped
to `arm64`) and selects the matching cache path. There is no environment override on this
path; because each 1.5.5 asset is a universal binary, either asset runs on any supported Mac.
The `ARCH` environment variable is honored only by `prebuilt/setup_env.sh` and the in-crate
`architecture` helper module (`frozen_duckdb::architecture::detect()`), not by the builder.

### Runtime Loading

The 1.5.5 dylibs carry the neutral install name `@rpath/libduckdb.dylib`. The
`frozen-duckdb` build script (consuming `DEP_DUCKDB_DUCKDB_LIB_DIR` metadata exposed by
`frozen-duckdb-sys`) emits `-Wl,-rpath,{cache_dir}` for binaries, tests, and examples, so
they run with **no `DYLD_LIBRARY_PATH` / `DYLD_FALLBACK_LIBRARY_PATH` configuration**.

## Compatibility Symlinks

The legacy prebuilt workflow (`prebuilt/setup_env.sh`) creates **compatibility symlinks** to maintain older build script compatibility:

```bash
ln -sf "$DUCKDB_LIB" "$DUCKDB_LIB_DIR/libduckdb.dylib"
ln -sf "$DUCKDB_LIB" "$DUCKDB_LIB_DIR/libduckdb.1.dylib"
ln -sf "$DUCKDB_LIB" "$DUCKDB_LIB_DIR/libduckdb.1.4.dylib"
```

### Symlink Notes

- **`libduckdb.dylib` is the name that matters**: DuckDB >= 1.5 dylibs carry the neutral install name `@rpath/libduckdb.dylib`, so the plain link name is what `-lduckdb` and `@rpath` resolution use.
- **`libduckdb.1.dylib` / `libduckdb.1.4.dylib` are legacy names** kept for 1.4-era consumers only.
- In the builder-managed cache, `libduckdb.dylib` is created as a symlink to the cached `libduckdb_{arch}.dylib` on every acquisition path.

## Cache Normalization (Builder)

`frozen-duckdb-builder` normalizes the cache after every acquisition path, so downstream builds always work:

```rust
// tail of ensure_binary() — runs unconditionally
ensure_headers(&versioned_cache)?;   // headers under duckdb/ for bindgen
ensure_link_name(&versioned_cache, &arch)?; // plain libduckdb.dylib for -lduckdb
```

### Normalization Guarantees

1. **Headers**: `duckdb/duckdb.h` and `duckdb/duckdb.hpp` exist in the versioned cache; when the acquisition path carried no headers (release downloads ship only the dylib), they are copied from the vendored 1.5.5 headers inside `crates/frozen-duckdb-builder/vendored-headers/`
2. **Link name**: `libduckdb.dylib` resolves (symlink on Unix) to the cached `libduckdb_{arch}.dylib`
3. **Idempotent**: existing artifacts are left untouched, so repeated builds do no extra work

## Binary Distribution Strategy

### GitHub Releases

Prebuilt dylibs are distributed as **GitHub Release assets**, not committed to the repository:

```bash
# Asset URL scheme consumed by frozen-duckdb-builder::ensure_binary()
https://github.com/seanchatmangpt/frozen-duckdb/releases/download/v1.5.5/libduckdb_{arch}.dylib
```

The `build-binaries.yml` workflow builds the dylibs on tag pushes (`v*`), stages them under the exact canonical download names, and attaches `libduckdb_x86_64.dylib` and `libduckdb_arm64.dylib` to the release.

## Version Management

### DuckDB Version Tracking

The crate version mirrors the bundled upstream DuckDB version (SemVer with that convention documented in the CHANGELOG):

```toml
# Cargo.toml (workspace)
[workspace.package]
version = "1.5.5"   # frozen-duckdb 1.5.5 bundles DuckDB 1.5.5
```

### Version Update Process

1. **Test new version**: Validate compatibility with existing features
2. **Update binaries**: CI (`build-binaries.yml`) builds and attaches `libduckdb_{arch}.dylib` assets for the new tag
3. **Update the pin**: bump `VERSION` in `frozen-duckdb-builder` and the fallback clone tag
4. **Update documentation**: Reflect version changes in docs
5. **Test integration**: Verify all features work with new version

## Binary Update Strategy

### Automated Updates

```bash
# Tag-driven: pushing a v* tag triggers build-binaries.yml
git tag v1.5.5
git push origin v1.5.5
```

### Manual Update Process

1. **Build binaries**: run `build-binaries.yml` (workflow_dispatch) for the target version
2. **Verification**: CI stages each artifact under the exact `libduckdb_{arch}.dylib` name the builder downloads
3. **Release**: assets attach to the tag's GitHub Release automatically
4. **Update documentation**: Reflect any changes in capabilities

## Performance Impact

### Binary Selection Performance

| Operation | Time | Impact |
|-----------|------|--------|
| **Architecture detection** | <1μs | Negligible |
| **Binary selection** | <1μs | Negligible |
| **Symlink creation** | <10ms | One-time setup |
| **Library loading** | <50ms | Runtime startup |

### Memory Usage

- **Binary footprint**: ~117MB per cached asset (universal: arm64 + x86_64)
- **Runtime memory**: Standard DuckDB memory usage
- **Build memory**: No additional overhead during builds

## Troubleshooting Binary Issues

### Common Issues

#### 1. Binary Not Found
```bash
# Check the builder-managed cache
ls -la ~/.frozen-duckdb/cache/

# Verify binary exists
find ~/.frozen-duckdb/cache -name 'libduckdb*'

# Force a fresh acquisition attempt
cargo clean && cargo build
```

#### 2. Architecture Mismatch
```bash
# Check current architecture
uname -m

# Builder uses uname -m; each v1.5.5 asset is universal
# (arm64 + x86_64), so either asset runs on this Mac
lipo -info ~/.frozen-duckdb/cache/v1.5.5-*/libduckdb_*.dylib
```

#### 3. Permission Issues
```bash
# Check file permissions
ls -la $DUCKDB_LIB_DIR/libduckdb*

# Fix permissions if needed
chmod 755 $DUCKDB_LIB_DIR/libduckdb*.dylib
```

### Debug Information

```bash
# Show system information (output only with -v)
frozen-duckdb-cli -v info

# Show cached binary details
ls -lah ~/.frozen-duckdb/cache/v*/
lipo -info ~/.frozen-duckdb/cache/v1.5.5-*/libduckdb_*.dylib

# Verbose build output (cargo's own flag — RUST_LOG is not read by the
# builder or the CLI)
cargo build -v
```

## Binary Security

### Validation Strategy

1. **Checksum verification**: Validate downloaded binaries (future enhancement)
2. **Signature checking**: Verify binary authenticity (future enhancement)
3. **Runtime validation**: Ensure binaries load and function correctly
4. **Fallback handling**: Graceful degradation if binaries corrupted

### Security Considerations

- **No code execution**: Binaries validated before use, not during execution
- **Runtime isolation**: library loading goes through the emitted `@rpath` entry, not a mutable global library path
- **Error boundaries**: Clear error messages for security-related failures
- **Audit trail**: Logging of binary selection and validation decisions

## Future Binary Enhancements

### Potential Improvements

1. **Compression**: Reduce binary sizes further
2. **Stripping**: Remove debug symbols for smaller production binaries
3. **Feature flags**: Optional components for smaller binaries
4. **CDN distribution**: Faster downloads for users
5. **Delta updates**: Only download changed binary components

### Cross-Platform Support

1. **Linux binaries**: Add support for Linux architectures (prebuilt assets are currently macOS-only; Linux builds use the local-compile fallback)
2. **Windows binaries**: Add Windows DLL support
3. **Cross-compilation**: Build binaries for multiple platforms
4. ~~**Universal binaries**~~: shipped as of v1.5.5 — each macOS asset already contains arm64 + x86_64 slices

## Summary

The binary management architecture provides **optimal performance** and **compatibility** while maintaining **simplicity** for end users. Release assets are universal macOS dylibs downloaded automatically from GitHub Releases, the cache is normalized on every acquisition path, and the emitted runtime `@rpath` means binaries and tests run with no environment configuration — seamless integration with existing Rust projects.
