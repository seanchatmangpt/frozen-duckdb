# Gap Detection Scripts

This directory contains scripts for systematically detecting implementation gaps, fake stubs, and missing functionality in the KCura codebase.

## Scripts Overview

### 1. `rg_sweep.sh` - Comprehensive Gap Analysis
**Purpose**: Provides a complete inventory of implementation gaps across the entire codebase.

**Usage**:
```bash
./scripts/rg_sweep.sh | tee sweep.log
```

**What it checks**:
- Stubs and placeholders (`unimplemented!`, `todo!`, `panic!`, `TODO`, `FIXME`, `XXX`, `dummy`, `fake`, `stub`, `placeholder`)
- Telemetry/OTEL integration
- Hooks runtime implementation
- Kernel/router implementation
- Receipts and cryptographic functions
- FFI surface completeness
- Benchmarks and test coverage
- Governance/audit scaffolding
- Compiler implementations (OWL RL, SHACL, SPARQL)
- Engine and runtime management
- Catalog and DDL management
- In-memory store implementation

**Output**: Comprehensive report with ✅ (implemented) and ⚠️ (potential gaps) indicators.

### 2. `ci_gate.sh` - CI Enforcement Gate
**Purpose**: Enforces "no unimplemented features" policy by failing the build if critical gaps are found.

**Usage**:
```bash
./scripts/ci_gate.sh
```

**What it enforces**:
- **Critical patterns**: No `unimplemented!`, `todo!`, `panic!`, `TODO`, `FIXME`, `XXX` in production code
- **No fake implementations**: No `dummy`, `fake`, `stub`, `placeholder` tokens
- **No hardcoded responses**: No `return "hardcoded"`, `return "mock"`, `return "fake"`, `return "dummy"`
- **Proper error handling**: No `unwrap()` or `expect()` in library code
- **Core API completeness**: Required functions must exist (`exec_sql`, `query_sparql`, `validate_shacl`, `on_commit`)
- **FFI completeness**: Must have FFI exports
- **Test coverage**: Must have FFI, hook, and kernel tests

**Exit codes**:
- `0`: All checks passed, code is production-ready
- `1`: Critical gaps found, build should fail

## Integration with CI/CD

### GitHub Actions Integration
Add to your CI workflow:

```yaml
- name: Check Implementation Completeness
  run: ./scripts/ci_gate.sh
```

### Pre-commit Hook
Add to `.git/hooks/pre-commit`:

```bash
#!/bin/bash
./scripts/ci_gate.sh
if [ $? -ne 0 ]; then
    echo "❌ Pre-commit failed: Critical implementation gaps detected"
    echo "Run './scripts/rg_sweep.sh' for detailed analysis"
    exit 1
fi
```

## Philosophy: Fail Fast ("Tables-or-Fail")

These scripts implement the KCura philosophy of **fail-fast implementation**:

- **No silent downgrades**: If OWL/SHACL cannot be lowered to catalog/DDL, return converter errors immediately
- **No placeholder implementations**: Functions must perform real work, not return hardcoded responses
- **No fake implementations**: All functions must have corresponding real implementations
- **No stubs in production**: Any function exposed via FFI/public API must prove real effects via tests

## Customization

### Adding New Gap Patterns
Edit `rg_sweep.sh` to add new patterns:

```bash
# Add new pattern check
echo "--- Checking new feature ---"
rg -n 'new_pattern' || echo "  ⚠️  No new_pattern found"
```

### Modifying CI Gate Strictness
Edit `ci_gate.sh` to adjust what constitutes a "critical" gap:

```bash
# Make pattern non-critical (warning only)
check_warning_patterns() {
    local pattern="$1"
    local description="$2"
    
    if rg -q "$pattern"; then
        echo "⚠️  WARNING: Found $description"
        rg -n "$pattern"
        # Don't set FAILED=true for warnings
    else
        echo "✅ No $description found"
    fi
}
```

## Troubleshooting

### False Positives
If the scripts report false positives:

1. **Documentation references**: Patterns in docs/README files are expected
2. **Test utilities**: `fake_*` functions in test utilities are acceptable
3. **Arrow patches**: Patterns in `patches/` directory are external code

### Missing Implementations
If gaps are reported:

1. **Check if feature is actually needed**: Some gaps may be intentional
2. **Implement missing functionality**: Follow the test-first approach
3. **Update scripts**: Add exceptions for legitimate cases

### Performance Impact
The scripts use `ripgrep` (rg) which is fast, but for very large codebases:

- Use `head_limit` parameter to limit output
- Run on specific directories instead of entire repo
- Cache results for repeated runs

## Related Tools

- `scripts/fake_guard.sh`: Legacy fake detection script
- `scripts/scan_fakes.sh`: Pattern-based fake detection
- `scripts/ci_gates.sh`: Comprehensive CI gate collection
- `scripts/redteam_probe.rs`: Runtime behavioral fake detection

## Contributing

When adding new gap detection patterns:

1. **Test thoroughly**: Ensure patterns catch real gaps without false positives
2. **Document clearly**: Explain what the pattern detects and why it's important
3. **Update both scripts**: Add to both `rg_sweep.sh` and `ci_gate.sh` as appropriate
4. **Consider impact**: Balance thoroughness with CI performance

## Examples

### Successful Run
```bash
$ ./scripts/ci_gate.sh
=== ✅ CI GATE PASSED ===
No critical implementation gaps found.
Code is ready for production deployment.
```

### Failed Run
```bash
$ ./scripts/ci_gate.sh
=== ❌ CI GATE FAILED ===
Critical implementation gaps found. Fix these issues before merging.

To see all gaps (including non-critical), run:
  ./scripts/rg_sweep.sh
```

### Comprehensive Analysis
```bash
$ ./scripts/rg_sweep.sh | tee sweep.log
=== 🔍 GAP & IMPLEMENTATION SWEEP ===
Scanning for unimplemented features, fake stubs, and missing functionality...
...
=== ✅ SWEEP COMPLETE ===
```
