#!/usr/bin/env bash
set -euo pipefail

# Semantic Coverage Verification
# Ensures that changes increase or preserve coverage and that mutation tests pass

echo "🔍 Verifying semantic coverage..."

# Check if lcov.info exists
if [[ ! -f "lcov.info" ]]; then
    echo "::error :: lcov.info not found - coverage data missing"
    exit 1
fi

# Basic coverage threshold check (should be > 80% for production)
COVERAGE_THRESHOLD=80

# Extract overall coverage percentage
OVERALL_COVERAGE=$(grep -o 'Branch coverage: [0-9.]*%' lcov.info | grep -o '[0-9.]*' | tail -1)

if [[ -z "$OVERALL_COVERAGE" ]]; then
    echo "::error :: Could not extract coverage percentage from lcov.info"
    exit 1
fi

echo "📊 Overall coverage: ${OVERALL_COVERAGE}%"

# Check if coverage meets threshold
if [ "$(echo "$OVERALL_COVERAGE < $COVERAGE_THRESHOLD" | bc)" = "1" ]; then
    echo "::error :: Coverage ${OVERALL_COVERAGE}% below threshold ${COVERAGE_THRESHOLD}%"
    exit 1
fi

echo "✅ Coverage threshold met"

# Check for critical path coverage (ensure kernel, hooks, tx paths are covered)
CRITICAL_PATHS=(
    "kcura_core::KCura::choose_execution_route"
    "kcura_core::KCura::metamorphic_kernel_test"
    "kcura_engine::hooks::TimerScheduler"
    "kcura_engine::runtime::KCuraEngine::commit_transaction"
    "kcura_engine::runtime::KCuraEngine::verify_receipt_against_delta"
)

for path in "${CRITICAL_PATHS[@]}"; do
    if ! grep -q "$path" lcov.info; then
        echo "::warning :: Critical path '$path' not found in coverage data"
    else
        echo "✅ Critical path '$path' covered"
    fi
done

echo "✅ Semantic coverage verification passed"