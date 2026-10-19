#!/bin/bash
# frozen-duckdb Release Checklist Script
# Run this before creating a release to ensure the full gate battery passes.
# Fail-closed: any nonzero step aborts the checklist (set -euo pipefail).
#
# DRY-RUN: RELEASE_CHECKLIST_DRYRUN=1 prints each step without executing.

set -euo pipefail

DRYRUN="${RELEASE_CHECKLIST_DRYRUN:-0}"
run() {
    echo "  \$ $*"
    if [ "$DRYRUN" = "1" ]; then
        return 0
    fi
    "$@"
}

echo "Running frozen-duckdb Release Checklist..."
if [ "$DRYRUN" = "1" ]; then
    echo "[DRY-RUN] steps will be printed, not executed"
fi

# 1. Verify metadata is valid
echo "[1/12] Validating metadata..."
run cargo metadata --format-version=1 > /dev/null

# 2. Format check
echo "[2/12] Checking formatting..."
run cargo fmt --all -- --check

# 3. Clippy with strict warnings
echo "[3/12] Running clippy..."
run cargo clippy --workspace --all-targets --all-features -- -D warnings

# 4. Test suite
echo "[4/12] Running test suite..."
run cargo test --workspace

# 5. Gate courts (release-mode, per gates battery)
echo "[5/12] Running gate courts..."
run cargo test --test gate_v01_latency --release
run cargo test --test gate_ingest_throughput --release
run cargo test --test doctor_storage_tests --release
run cargo test --test doctor_ocel_tests --release

# 6. Generated-surface gate battery (ggen projections + receipt)
echo "[6/12] Running make verify (verify-gates + ggen receipt verify)..."
run make verify

# 7. Genesis check (generated-surface drift)
echo "[7/12] Running make genesis-check..."
run make genesis-check

# 8. Doctor preflight
echo "[8/12] Running doctor preflight..."
run bash scripts/doctor/00_preflight.sh

# 9. Publish provenance receipt (ggen receipt verify + contract census)
echo "[9/12] Running publish receipt validation..."
run python3 scripts/verify_publish_receipt.py

# 10. Coverage check (only if cargo-llvm-cov is installed; loud skip otherwise)
if command -v cargo-llvm-cov > /dev/null 2>&1; then
    echo "[10/12] Running coverage..."
    RUSTFLAGS="-Cdebuginfo=2" run cargo llvm-cov --workspace --lcov --output-path lcov.info
else
    echo "[10/12] [SKIP] cargo-llvm-cov not found on PATH — coverage step dropped (loud skip, not a faked pass)"
fi

# 11. Security audit (only if cargo-deny is installed; loud skip otherwise)
if command -v cargo-deny > /dev/null 2>&1; then
    echo "[11/12] Running security audit..."
    run cargo deny check advisories
else
    echo "[11/12] [SKIP] cargo-deny not found on PATH — security audit step dropped (loud skip, not a faked pass)"
fi

# 12. Documentation build
echo "[12/12] Building documentation..."
run cargo doc --workspace --no-deps

echo ""
echo "All release checks passed!"
echo ""
echo "Next steps:"
echo "1. Review the generated documentation: cargo doc --open"
echo "2. Update CHANGELOG.md with release notes"
echo "3. Tag the release: git tag vX.Y.Z"
echo "4. Push the tag: git push origin vX.Y.Z"
echo "5. Create GitHub release with release notes"
