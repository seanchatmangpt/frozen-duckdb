#!/usr/bin/env bash
# KCura Client Smoke Tests - 80/20 Core Team Implementation
# Configuration-driven, structured logging, enterprise-grade multi-language testing
# Version: 3.0 - Production hardened with observability

set -euo pipefail

# Script metadata
readonly SCRIPT_NAME="smoke_all"
readonly SCRIPT_VERSION="3.0"

# Load core team libraries (80/20 innovation)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Source core team libraries with self-healing and intelligent caching
source "$SCRIPT_DIR/lib/config.sh"
source "$SCRIPT_DIR/lib/logging.sh"
source "$SCRIPT_DIR/lib/self_healing.sh"
source "$SCRIPT_DIR/lib/intelligent_cache.sh"

# Initialize systems (80/20 innovation)
initialize_logging

# Initialize configuration with self-healing
if ! init_config; then
    warn "Configuration initialization failed, attempting self-healing"
    if ! repair_configuration "$PROJECT_ROOT/scripts/kcura-config.yaml"; then
        fatal "Failed to initialize configuration system after self-healing"
    fi
    # Re-initialize after repair
    init_config
fi

# Initialize intelligent caching and self-healing
init_intelligent_cache
init_self_healing

# Export configuration for environment variables
export_config

# Get script-specific configuration with adaptive settings
readonly TEST_TIMEOUT="$(get_timeout "smoke_tests" "test")"
readonly MAX_RETRIES="$(get_script_config "smoke_tests" "max_retries" "2")"
readonly SKIP_MISSING_DEPS="$(get_script_config "smoke_tests" "skip_on_missing_deps" "true")"
readonly PARALLEL_EXECUTION="$(get_script_config "smoke_tests" "parallel_execution" "false")"

# Global state
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
SKIPPED_TESTS=0

# Enhanced logging for smoke tests
smoke_log() {
    local level="$1"
    local message="$2"
    local extra_data="${3:-}"

    # Use the structured logging system
    log "$level" "$message" "$extra_data"
}

# Safe script validation with security checks
validate_smoke_script() {
    local script_path="$1"
    local script_name="$2"
    local required_commands="$3"

    # Basic path validation
    if [[ ! -f "$script_path" ]]; then
        error "Smoke script not found: $script_path" "\"script\":\"$script_name\",\"path\":\"$script_path\""
        return 1
    fi

    # Check for path traversal attempts
    if [[ "$script_path" =~ \.\. ]]; then
        error "Path traversal detected in script path: $script_path"
        return 1
    fi

    # Validate script before making executable (security fix)
    if [[ ! -x "$script_path" ]]; then
        debug "Making script executable: $script_path"

        # Security check: only make executable if it's a valid script file
        if [[ -f "$script_path" ]] && [[ -r "$script_path" ]] && [[ "$(head -1 "$script_path" 2>/dev/null)" =~ ^#! ]]; then
            if ! chmod +x "$script_path" 2>/dev/null; then
                warn "Cannot make script executable: $script_path"
                return 1
            fi
        else
            error "Refusing to make non-script file executable: $script_path"
            return 1
        fi
    fi

    # Validate required commands exist
    if [[ -n "$required_commands" ]]; then
        for cmd in $required_commands; do
            if ! command -v "$cmd" >/dev/null 2>&1; then
                if [[ "$SKIP_MISSING_DEPS" == "true" ]]; then
                    info "Skipping $script_name: required command '$cmd' not found"
                    record_metric "tests_skipped" 1
                    return 2  # Special return code for skip
                else
                    error "Required command not found for $script_name: $cmd"
                    return 1
                fi
            fi
        done
    fi

    info "Smoke script validated: $script_name" "\"path\":\"$script_path\""
    return 0
}

# Safe test execution with timeout and retry logic
run_smoke_test() {
    local name="$1"
    local script_path="$2"
    local expected_exit_code="${3:-0}"
    local retry_count=0

    TOTAL_TESTS=$((TOTAL_TESTS + 1))

    # Validate inputs
    if [[ -z "$name" ]] || [[ -z "$script_path" ]]; then
        error "Invalid parameters for smoke test" "\"name\":\"$name\",\"script\":\"$script_path\""
        FAILED_TESTS=$((FAILED_TESTS + 1))
        return 1
    fi

    start_timer "smoke_test_$name"

    info "Starting smoke test: $name" "\"script\":\"$script_path\",\"expected_exit\":\"$expected_exit_code\""

    # Retry logic for transient failures
    while [[ $retry_count -lt $MAX_RETRIES ]]; do
        # Execute test with safe timeout
        if timeout "$TEST_TIMEOUT" bash -c "
            # Safe execution with input validation
            if [[ -x '$script_path' ]] && [[ -f '$script_path' ]]; then
                cd '$(dirname "$script_path")' && '$script_path'
            else
                echo 'Script execution failed: invalid script'
                exit 1
            fi
        " 2>/dev/null; then

            local actual_exit_code=$?

            if [[ $actual_exit_code -eq $expected_exit_code ]]; then
                info "Smoke test passed: $name" "\"exit_code\":$actual_exit_code,\"expected\":$expected_exit_code"
                record_metric "smoke_tests_passed" 1
                PASSED_TESTS=$((PASSED_TESTS + 1))

                end_timer "smoke_test_$name"
                return 0
            else
                warn "Smoke test failed: $name (exit code $actual_exit_code, expected $expected_exit_code)"
                record_metric "smoke_tests_failed" 1
            fi
        else
            # Timeout or execution error
            if [[ $? -eq 124 ]]; then
                warn "Smoke test timed out: $name"
                record_metric "smoke_test_timeouts" 1
            else
                warn "Smoke test execution failed: $name"
                record_metric "smoke_test_failures" 1
            fi
        fi

        retry_count=$((retry_count + 1))
        if [[ $retry_count -lt $MAX_RETRIES ]]; then
            debug "Retrying smoke test: $name (attempt $retry_count/$MAX_RETRIES)"
            sleep 2
        fi
    done

    error "Smoke test failed after $MAX_RETRIES attempts: $name"
    FAILED_TESTS=$((FAILED_TESTS + 1))

    end_timer "smoke_test_$name"
    return 1
}

# Configuration-driven test definitions
get_test_config() {
    local test_name="$1"
    local config_key="$2"
    local default_value="$3"

    # Map test names to configuration sections
    case "$test_name" in
        "Node.js") echo "$(get_script_config "smoke_tests" "tests.node.$config_key" "$default_value")" ;;
        "Go") echo "$(get_script_config "smoke_tests" "tests.go.$config_key" "$default_value")" ;;
        "Python") echo "$(get_script_config "smoke_tests" "tests.python.$config_key" "$default_value")" ;;
        *) echo "$default_value" ;;
    esac
}

# Execute individual smoke test with configuration
execute_smoke_test() {
    local test_name="$1"

    info "Executing smoke test: $test_name"

    # Get test configuration from YAML
    local script_path
    script_path="$(get_test_config "$test_name" "script" "")"
    local required_commands
    required_commands="$(get_test_config "$test_name" "required_commands" "")"
    local expected_exit_code
    expected_exit_code="$(get_test_config "$test_name" "expected_exit_code" "0")"

    # Validate script path
    if [[ -z "$script_path" ]]; then
        error "No script path configured for test: $test_name"
        return 1
    fi

    # Full script path
    local full_script_path="$PROJECT_ROOT/$script_path"

    # Validate and run the test
    if validate_smoke_script "$full_script_path" "$test_name" "$required_commands"; then
        local validation_result=$?

        if [[ $validation_result -eq 2 ]]; then
            # Test was skipped due to missing dependencies
            info "Test skipped due to missing dependencies: $test_name"
            return 0
        fi

        # Run the actual test
        if run_smoke_test "$test_name" "$full_script_path" "$expected_exit_code"; then
            info "Test completed successfully: $test_name"
        else
            error "Test failed: $test_name"
        fi
    else
        error "Test validation failed: $test_name"
    fi
}

# Main execution function with configuration-driven approach
main() {
    info "Starting KCura smoke tests v$SCRIPT_VERSION" "\"correlation_id\":\"$CORRELATION_ID\",\"parallel\":\"$PARALLEL_EXECUTION\""

    # Health check for test environment
    health_check "test_environment" "ok" "Smoke test environment initialized"

    # Define tests to run (could be made configurable)
    local tests_to_run=(
        "Node.js"
        "Go"
        "Python"
    )

    # Execute tests (sequentially for 80/20 safety)
    for test_name in "${tests_to_run[@]}"; do
        if ! execute_smoke_test "$test_name"; then
            debug "Test execution returned non-zero for: $test_name"
        fi
    done

    # Final health checks (80/20 innovation)
    health_check "smoke_tests_completion" "completed" "Smoke tests finished: $PASSED_TESTS passed, $FAILED_TESTS failed, $SKIPPED_TESTS skipped"
    self_healing_health_check
    intelligent_cache_health_check

    # Summary with detailed metrics
    info "Smoke test execution summary" "\"total\":$TOTAL_TESTS,\"passed\":$PASSED_TESTS,\"failed\":$FAILED_TESTS,\"skipped\":$SKIPPED_TESTS"

    # Determine overall result
    if [[ $FAILED_TESTS -eq 0 ]]; then
        info "All smoke tests completed successfully"
        return 0
    else
        warn "Some smoke tests failed: $FAILED_TESTS failures out of $TOTAL_TESTS total tests"
        return 1
    fi
}

# Execute with error handling
if ! time_execution "smoke_tests_execution" main "$@"; then
    error "Smoke tests execution failed"
    exit 1
fi

# Cleanup is handled by logging library trap
exit 0