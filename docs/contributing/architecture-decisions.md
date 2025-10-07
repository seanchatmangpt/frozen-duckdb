# Architecture Decisions Guide

## Overview

This guide documents the **key architecture decisions** made during the development of Frozen DuckDB, including **rationale**, **trade-offs**, **alternatives considered**, and **future considerations**. These decisions ensure **fast builds**, **production reliability**, and **maintainable code**.

## ADR 001: Architecture-Specific Binaries

### Decision

Split the universal DuckDB binary into **architecture-specific binaries** (x86_64 and arm64) instead of using a single universal binary.

### Rationale

**Problem**: Universal binaries were 105MB and provided suboptimal performance across different architectures.

**Solution**: Architecture-specific binaries with native optimization.

**Benefits**:
- **50% smaller downloads**: Users only get binaries for their architecture
- **Better performance**: Native architecture optimization
- **Faster setup**: No need to download unused architecture code
- **Storage efficiency**: 50-55MB per user vs 105MB universal

### Trade-offs

**Advantages**:
- Significantly smaller download sizes
- Better runtime performance
- Faster initial setup
- More efficient storage usage

**Disadvantages**:
- Slightly more complex binary management
- Requires architecture detection logic
- Multiple binaries to maintain and distribute

### Alternatives Considered

1. **Universal Binary Only**: Simpler but larger downloads and suboptimal performance
2. **Runtime Architecture Detection**: More complex but could optimize further
3. **Single Binary with Runtime Selection**: Would still be large and unoptimized

### Implementation

```rust
pub fn get_binary_name() -> String {
    let arch = detect();
    match arch.as_str() {
        "x86_64" => "libduckdb_x86_64.dylib".to_string(),
        "arm64" | "aarch64" => "libduckdb_arm64.dylib".to_string(),
        _ => "libduckdb.dylib".to_string(), // fallback
    }
}
```

### Future Considerations

- **Linux support**: Add Linux architecture-specific binaries
- **Windows support**: Add Windows DLL variants
- **Cross-compilation**: Build binaries for multiple platforms
- **Dynamic loading**: Load only required components at runtime

## ADR 002: Smart Environment Detection

### Decision

Implement **automatic architecture detection** with **manual override capability** via the `ARCH` environment variable.

### Rationale

**Problem**: Users needed to manually specify architecture, leading to configuration errors and poor user experience.

**Solution**: Auto-detect architecture with manual override for flexibility.

**Benefits**:
- **Seamless user experience**: No manual configuration required
- **Flexibility**: Manual override for CI/CD and testing
- **Backward compatibility**: Existing scripts continue to work
- **Error prevention**: Reduces configuration mistakes

### Trade-offs

**Advantages**:
- Excellent user experience
- Flexible for different environments
- Maintains backward compatibility
- Reduces support burden

**Disadvantages**:
- Additional logic for detection and override
- Potential for detection edge cases
- Environment variable dependency

### Alternatives Considered

1. **No Override**: Simpler but less flexible for CI/CD
2. **Config File**: More complex than environment variable
3. **Runtime Detection Only**: Less flexible for testing scenarios

### Implementation

```rust
pub fn detect() -> String {
    env::var("ARCH").unwrap_or_else(|_| std::env::consts::ARCH.to_string())
}
```

### Future Considerations

- **Platform-specific detection**: Handle edge cases on different systems
- **Container optimization**: Optimize for containerized environments
- **Cross-compilation support**: Better support for cross-compilation workflows
- **Detection caching**: Cache detection results for performance

## ADR 003: Flock Extension Integration

### Decision

Integrate the **Flock extension** for **LLM-in-database capabilities** using **Ollama** as the local LLM backend.

### Rationale

**Problem**: Users needed external tools for AI operations, creating integration complexity and privacy concerns.

**Solution**: Native LLM operations within DuckDB using local Ollama models.

**Benefits**:
- **SQL-native AI**: Use familiar SQL syntax for AI tasks
- **Local processing**: Complete privacy with no external API calls
- **Performance**: Fast local inference without network latency
- **Integration**: Seamless workflow with existing database operations

### Trade-offs

**Advantages**:
- Unique integration of database and AI operations
- Complete data privacy and security
- High performance with local inference
- Extensible for future AI capabilities

**Disadvantages**:
- Dependency on external extension ecosystem
- Current implementation limitations (27% test success rate)
- Additional setup complexity for users
- Maintenance overhead for extension compatibility

### Alternatives Considered

1. **External API Integration**: Would require network calls and reduce privacy
2. **Custom LLM Implementation**: Too complex and would duplicate existing work
3. **No LLM Integration**: Would miss market opportunity for AI-database integration

### Implementation

**Extension Setup:**
```sql
-- Install Flock extension
INSTALL flock FROM community;
LOAD flock;

-- Configure Ollama
CREATE SECRET ollama_secret (TYPE OLLAMA, API_URL 'http://localhost:11434');

-- Create models
CREATE MODEL('coder', 'qwen3-coder:30b', 'ollama');
CREATE MODEL('embedder', 'qwen3-embedding:8b', 'ollama');
```

**LLM Operations:**
```sql
-- Text completion
SELECT llm_complete(
    {'model_name': 'coder'},
    {'prompt_name': 'complete', 'context_columns': [{'data': 'Explain recursion'}]}
);

-- Embedding generation
SELECT llm_embedding(
    {'model_name': 'embedder'},
    [{'data': 'machine learning'}]
);
```

### Current Status

**Production Ready Features**:
- ✅ Extension loading and configuration
- ✅ Model creation and management
- ✅ Basic LLM operations (when Ollama is properly configured)
- ✅ Integration with Frozen DuckDB CLI

**Known Limitations**:
- ⚠️ 27% Flock test success rate (4/11 tests passing)
- ⚠️ Model resolution issues in some configurations
- ⚠️ Prompt management complexity
- ⚠️ Limited error handling for network issues

### Future Considerations

- **Improved reliability**: Address current implementation issues
- **Enhanced model support**: Support for additional Ollama models
- **Better error handling**: More robust error messages and recovery
- **Performance optimization**: Faster operations and better caching
- **Extended capabilities**: Additional AI operations and integrations

## ADR 004: Pre-compiled Binary Distribution

### Decision

Use **Git LFS** for distributing **large binary files** (>100MB) to handle GitHub's file size limitations while maintaining repository performance.

### Rationale

**Problem**: DuckDB binaries are >50MB each, and GitHub has a 100MB file size limit for regular files.

**Solution**: Use Git LFS for large binary files to enable efficient distribution.

**Benefits**:
- **GitHub compatibility**: Handle files >100MB limit
- **Efficient storage**: Only changed files transferred during clone
- **Bandwidth optimization**: Reduced clone times for contributors
- **Version control**: Proper tracking of binary file changes

### Trade-offs

**Advantages**:
- Enables distribution of large binary files
- Efficient for contributors (smaller clones)
- Proper version control for binaries
- GitHub-compatible solution

**Disadvantages**:
- Requires Git LFS setup for contributors
- Additional complexity for repository management
- Potential issues with LFS in some environments
- Storage costs for large files

### Alternatives Considered

1. **External Downloads**: Would require users to download binaries separately
2. **Compressed Archives**: Would still be large and not version controlled
3. **No Binaries in Repo**: Would require complex setup process for users

### Implementation

**Git LFS Configuration:**
```bash
# Track binary files with Git LFS
git lfs track "*.dylib"
git lfs track "*.so"
git lfs track "*.dll"

# Add to repository
git add .gitattributes
git add prebuilt/*.dylib
```

**Repository Structure:**
```
prebuilt/
├── libduckdb_x86_64.dylib    # Tracked with LFS (55MB)
├── libduckdb_arm64.dylib     # Tracked with LFS (50MB)
├── duckdb.h                  # Regular file (186KB)
├── duckdb.hpp                # Regular file (1.8MB)
└── setup_env.sh              # Regular file (script)
```

### Future Considerations

- **CDN distribution**: Faster downloads for users
- **Delta updates**: Only download changed binary components
- **Mirroring**: Multiple distribution points for reliability
- **Compression**: Reduce binary sizes further for distribution

## ADR 005: Minimal Dependency Strategy

### Decision

Use a **minimal set of well-tested dependencies** to reduce **build complexity**, **security surface**, and **maintenance burden**.

### Rationale

**Problem**: Complex dependency trees increase build times, introduce security vulnerabilities, and create maintenance overhead.

**Solution**: Carefully curated dependencies focused on core functionality.

**Benefits**:
- **Faster builds**: Fewer dependencies to compile
- **Security**: Smaller attack surface
- **Maintenance**: Less dependency update overhead
- **Reliability**: Fewer potential points of failure

### Trade-offs

**Advantages**:
- Significantly faster build times
- Reduced security vulnerabilities
- Simpler maintenance and updates
- More predictable build behavior

**Disadvantages**:
- Some functionality requires custom implementation
- Potential for code duplication
- May miss some ecosystem improvements
- Requires more careful evaluation of new dependencies

### Alternatives Considered

1. **Feature-Rich Dependencies**: Would provide more functionality but increase complexity
2. **Framework Dependencies**: Would simplify some aspects but add heavy dependencies
3. **Zero Dependencies**: Impractical for a database integration library

### Implementation

**Core Dependencies Only:**
```toml
[dependencies]
anyhow = "1"                    # Error handling
clap = { version = "4", features = ["derive"] }  # CLI parsing
tracing = "0.1"                # Logging
tracing-subscriber = "0.3"     # Log formatting
duckdb = "1.4.0"               # Database integration
tempfile = "3"                 # Temporary files
reqwest = "0.11"               # HTTP client (for LLM operations)
serde_json = "1"               # JSON handling
chrono = "0.4"                 # Date/time handling
```

**Dependency Evaluation Criteria:**
- **Essential functionality**: Must provide core required features
- **Security track record**: Well-maintained with good security history
- **Performance impact**: Minimal overhead on build and runtime performance
- **Maintenance burden**: Active maintenance and regular updates

### Future Considerations

- **Dependency audits**: Regular security and maintenance reviews
- **Alternative implementations**: Consider if custom implementations would be better
- **Feature additions**: Evaluate new dependencies against strict criteria
- **Build optimization**: Monitor dependency impact on build performance

## ADR 006: Comprehensive Testing Strategy

### Decision

Implement a **rigorous testing strategy** with **multiple test runs**, **property-based testing**, and **performance validation** to ensure **reliability** and **catch flaky behavior**.

### Rationale

**Problem**: Insufficient testing led to unreliable releases and performance regressions.

**Solution**: Comprehensive testing with multiple validation layers.

**Benefits**:
- **Reliability**: Catch flaky tests and edge cases
- **Performance assurance**: Validate SLO requirements consistently
- **Quality confidence**: High test coverage and validation
- **Regression prevention**: Early detection of issues

### Trade-offs

**Advantages**:
- High confidence in code quality and reliability
- Early detection of performance regressions
- Comprehensive edge case coverage
- Better debugging and troubleshooting

**Disadvantages**:
- Longer test execution times
- More complex test infrastructure
- Higher maintenance burden for test suite
- Potential for test-induced failures

### Alternatives Considered

1. **Basic Testing**: Simpler but insufficient for production quality
2. **CI-Only Testing**: Would miss local development issues
3. **No Performance Testing**: Would allow performance regressions

### Implementation

**Multiple Test Runs:**
```bash
# Core team requirement: Run tests 3+ times
cargo test --all && cargo test --all && cargo test --all
```

**Property-Based Testing:**
```rust
proptest! {
    #[test]
    fn test_architecture_detection_properties(arch in "x86_64|arm64|aarch64") {
        std::env::set_var("ARCH", arch);
        let detected = detect();
        prop_assert!(!detected.is_empty());
        std::env::remove_var("ARCH");
    }
}
```

**Performance Validation:**
```rust
#[test]
fn test_build_performance_slo() {
    let duration = benchmark::measure_build_time(|| Ok(()));

    assert!(
        duration.as_secs() < 10,
        "Build time exceeded SLO: {:?}",
        duration
    );
}
```

### Future Considerations

- **Test parallelization**: Run tests in parallel for faster execution
- **Test data generation**: Automated generation of diverse test data
- **Cross-platform testing**: Ensure compatibility across all supported platforms
- **Performance benchmarking**: Continuous performance regression detection

## ADR 007: Documentation-First Development

### Decision

Adopt a **documentation-first approach** with **comprehensive documentation** covering **architecture**, **API reference**, **integration guides**, and **troubleshooting** before implementation.

### Rationale

**Problem**: Insufficient documentation led to integration difficulties and maintenance issues.

**Solution**: Comprehensive documentation as part of the development process.

**Benefits**:
- **Better integration**: Clear guidance for users
- **Maintenance ease**: Well-documented codebase
- **Quality assurance**: Documentation validates understanding
- **Community contribution**: Lower barrier for contributors

### Trade-offs

**Advantages**:
- Significantly better user experience
- Easier maintenance and debugging
- Better contribution workflow
- Higher code quality through documentation

**Disadvantages**:
- Additional development time for documentation
- Documentation maintenance overhead
- Potential for documentation to become outdated
- Requires discipline to maintain

### Alternatives Considered

1. **Code-Only Documentation**: Insufficient for complex integration scenarios
2. **Minimal Documentation**: Would lead to support burden and integration issues
3. **External Documentation**: Harder to maintain and keep in sync

### Implementation

**Documentation Structure:**
```
docs/
├── README.md                    # Documentation index
├── architecture/
│   ├── overview.md             # System architecture
│   ├── binary-management.md    # Binary distribution
│   └── build-optimization.md   # Performance optimization
├── api/
│   ├── library.md              # Rust API reference
│   ├── cli.md                  # CLI commands
│   └── flock.md                # LLM API reference
├── guides/
│   ├── integration.md          # Project integration
│   ├── llm-setup.md            # Ollama configuration
│   ├── dataset-management.md   # Data operations
│   └── performance-tuning.md   # Performance optimization
└── troubleshooting/
    ├── build-issues.md         # Build problems
    └── faq.md                  # Common questions
```

**Documentation Standards:**
- **Working examples**: All code examples must compile and run
- **Complete coverage**: Document all public APIs and features
- **Real scenarios**: Use actual usage patterns, not mock examples
- **Maintenance**: Keep documentation in sync with code changes

### Future Considerations

- **Automated documentation**: Generate API docs from code annotations
- **Interactive examples**: Runnable code examples in documentation
- **Multilingual documentation**: Support for additional languages
- **Community contributions**: Process for community documentation improvements

## Summary

The architecture decisions documented here form the **foundation** of Frozen DuckDB's **success**, providing **fast builds**, **production reliability**, and **excellent user experience**. Each decision balances **competing concerns** to achieve **optimal outcomes** for the project's goals.

**Key Architecture Principles:**
- **Performance first**: Architecture-specific binaries for optimal speed
- **User experience**: Automatic detection with flexible override options
- **Integration focus**: Seamless compatibility with existing workflows
- **Quality assurance**: Comprehensive testing and documentation
- **Future-proofing**: Extensible design for evolving requirements

**Decision-Making Framework:**
1. **Identify the problem** clearly and comprehensively
2. **Consider multiple alternatives** with pros and cons
3. **Evaluate trade-offs** against project goals and constraints
4. **Implement with testing** and validation
5. **Document rationale** for future maintainers
6. **Monitor impact** and adjust as needed

**Architecture Evolution:**
- **Current state**: Solid foundation with room for enhancement
- **Growth areas**: Extended platform support and advanced features
- **Maintenance focus**: Performance monitoring and quality assurance
- **Innovation opportunities**: New integration patterns and capabilities

**Next Steps:**
1. Review these decisions when considering architectural changes
2. Document new decisions using the established ADR format
3. Monitor the impact of existing decisions on project success
4. Consider the documented alternatives when requirements evolve
