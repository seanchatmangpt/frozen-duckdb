# Frozen DuckDB Architecture Overview

## System Architecture

The Frozen DuckDB project is designed as a **drop-in replacement** for bundled DuckDB compilation that provides **99% faster builds** while maintaining **100% compatibility** with existing Rust projects using `duckdb-rs`.

### Core Components

```
┌─────────────────────────────────────────────────────────────┐
│                    Frozen DuckDB Binary                     │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │ Architecture│  │ Environment │  │ Performance         │  │
│  │ Detection   │  │ Setup       │  │ Benchmarking        │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │ Build       │  │ CLI Tool    │  │ LLM Integration     │  │
│  │ Integration │  │ (Dataset    │  │ (Flock Extension)   │  │
│  │             │  │ Management) │  │                     │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### Key Design Principles

#### 1. Fast Builds Only
- **Pre-compiled binaries**: No DuckDB compilation during builds
- **Universal dylibs**: each v1.5.5 release asset contains arm64 + x86_64 slices
- **Minimal dependencies**: Reduce build complexity and download size

#### 2. Drop-in Compatibility
- **Zero breaking changes**: Seamless integration with existing projects
- **Zero configuration**: the builder acquires the binary automatically; `DUCKDB_LIB_DIR`/`DUCKDB_INCLUDE_DIR` are only used by the legacy prebuilt workflow
- **Fallback handling**: local compile pinned at upstream tag `v1.5.5` if no release asset is reachable

#### 3. Smart Architecture Detection
- **Automatic detection**: `uname -m` in the builder (`x86_64`, or `arm64`/`aarch64` → `arm64`); the `ARCH` override applies only to `prebuilt/setup_env.sh` and the `architecture` helper module
- **Binary selection**: appropriate versioned cache path per architecture
- **Runtime rpath**: the emitted `-Wl,-rpath` entry makes binaries and tests run without `DYLD_*` variables

## Module Architecture

### Core Library (`src/lib.rs`)

```rust
pub mod architecture;    // Architecture detection and binary selection
pub mod benchmark;       // Performance measurement utilities
pub mod env_setup;       // Environment validation and configuration
```

### CLI Tool (`src/main.rs`)

```rust
Commands::Download {     // Dataset management (Chinook, TPC-H)
Commands::Convert {      // Format conversion (CSV ↔ Parquet)
Commands::Info {         // System information display
Commands::FlockSetup {   // Ollama configuration for LLM
Commands::Complete {     // Text completion via LLM
Commands::Embed {        // Embedding generation
Commands::Search {       // Semantic search
Commands::Filter {       // LLM-based filtering
Commands::Summarize {    // Text summarization
```

### Build Integration (`frozen-duckdb-sys/build.rs`)

- **Binary acquisition**: calls `frozen-duckdb-builder::ensure_binary()` (cache → local prebuilt dir → GitHub Release download → local compile pinned at upstream tag `v1.5.5`)
- **Library linking**: `cargo:rustc-link-lib=dylib=duckdb` against the cached `libduckdb.dylib` link name
- **Header inclusion**: bindgen against the builder's headers (vendored 1.5.5 headers copied into the cache `duckdb/` subdir)
- **Runtime rpath**: `frozen-duckdb`'s build script consumes `DEP_DUCKDB_DUCKDB_LIB_DIR` and emits `-Wl,-rpath,{lib_dir}` for bins/tests/examples

## Data Flow Architecture

### Build Time
```
1. Binary Acquisition (ensure_binary)
   ├── Check cache: ~/.frozen-duckdb/cache/v1.5.5-{arch}/libduckdb_{arch}.dylib
   ├── Else copy local prebuilt/libduckdb_{arch}.dylib if present
   ├── Else download from GitHub Releases (libduckdb_{arch}.dylib)
   ├── Else clone upstream DuckDB at tag v1.5.5 and compile
   └── Normalize cache: duckdb/ headers + plain libduckdb.dylib link name

2. Build Integration
   ├── Set library search paths
   ├── Link DuckDB library (dylib)
   ├── Generate bindings from vendored/cached headers
   └── Emit runtime @rpath for bins/tests/examples
```

### Runtime
```
1. Library Loading
   ├── Load architecture-specific binary
   ├── Initialize DuckDB connection
   ├── Install required extensions
   └── Verify functionality

2. CLI Operations
   ├── Parse commands and arguments
   ├── Initialize DatasetManager/FlockManager
   ├── Execute requested operation
   └── Return results or save to files
```

## Performance Architecture

### Build Optimization Strategy

| Component | Before | After | Improvement |
|-----------|--------|-------|-------------|
| **First Build** | 1-2 minutes | 7-10 seconds | **85% faster** |
| **Incremental** | 30 seconds | 0.11 seconds | **99% faster** |
| **Release** | 1-2 minutes | 0.11 seconds | **99% faster** |

### Universal Binary Distribution

- **`libduckdb_arm64.dylib`** (~117MB): universal asset (arm64 + x86_64 slices)
- **`libduckdb_x86_64.dylib`** (~117MB): universal asset (arm64 + x86_64 slices)
- Either asset runs on Apple Silicon or Intel Macs

### Memory Usage
- **Binary size**: ~117MB per cached asset (universal)
- **Runtime memory**: ~50MB for typical operations
- **Build memory**: Minimal additional overhead

## Integration Architecture

### Zero-Configuration Path (primary)

No environment variables needed — `cargo build` triggers `frozen-duckdb-builder::ensure_binary()`, the cache is normalized (headers under `duckdb/`, plain `libduckdb.dylib` link name), and the emitted runtime `@rpath` loads the dylib at run time.

### Legacy Environment Variables (manual prebuilt workflow)

```bash
source prebuilt/setup_env.sh   # sets DUCKDB_LIB_DIR / DUCKDB_INCLUDE_DIR, ARCH-overridable
```

### CI/CD Integration
```yaml
# Zero setup: the builder downloads and caches the dylib on first build
- name: Build
  run: cargo build --release
```

## Security Architecture

- **No unsafe code**: All FFI interactions handled safely
- **Binary validation**: Environment and binary verification before use
- **Error handling**: Clear error messages with actionable guidance
- **Fallback behavior**: Graceful degradation if binaries unavailable

## Error Handling Architecture

### Exit Codes
- **0**: Success
- **1**: General error (invalid arguments, file not found)
- **2**: Environment not configured
- **3**: Binary validation failed
- **4**: Flock extension not available

### Error Types
- **Environment errors**: Missing variables, invalid paths
- **Binary errors**: Missing libraries, architecture mismatch
- **Network errors**: Ollama connectivity, model availability
- **LLM errors**: Model resolution, prompt configuration

## Future Architecture Considerations

### Potential Extensions
1. **Linux support**: Add Linux binaries for broader compatibility
2. **Windows support**: Add Windows binaries for cross-platform
3. **Version management**: Support multiple DuckDB versions
4. **Feature flags**: Optional extensions and capabilities
5. **Automated updates**: CI/CD for new DuckDB releases

### Scalability Considerations
1. **Binary distribution**: Git LFS for large files
2. **Architecture expansion**: Additional platforms and optimizations
3. **Performance monitoring**: Build time tracking and alerting
4. **User feedback**: Issue tracking and resolution mechanisms

## Architecture Decision Records (ADRs)

### ADR 001: Architecture-Specific Binaries
**Decision**: Split universal binary into architecture-specific versions

**Rationale**:
- Reduces download size by 50%
- Improves performance (native architecture)
- Better user experience (faster setup)

**Trade-offs**:
- Slightly more complex binary management
- Requires architecture detection logic
- Multiple binaries to maintain

### ADR 002: Smart Environment Detection
**Decision**: Auto-detect architecture with manual override capability

**Rationale**:
- Seamless user experience
- Flexibility for CI/CD environments
- Backward compatibility

**Trade-offs**:
- Additional logic for detection and override
- Potential for detection failures
- Environment variable dependency

### ADR 003: Flock Extension Integration
**Decision**: Integrate Flock extension for LLM-in-database capabilities

**Rationale**:
- Enables advanced RAG and AI operations
- Leverages existing DuckDB ecosystem
- Provides seamless LLM integration

**Trade-offs**:
- Dependency on external extension
- Current implementation limitations (27% test success)
- Additional setup complexity for users

## Summary

The Frozen DuckDB architecture is designed for **maximum developer productivity** through fast builds while maintaining **complete compatibility** and adding **advanced LLM capabilities**. The system prioritizes simplicity, performance, and reliability while providing a clear upgrade path for future enhancements.
