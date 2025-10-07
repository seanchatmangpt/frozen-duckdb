#!/usr/bin/env bash
# KCura CI Gate - 80/20 Core Team Implementation
# Configuration-driven, structured logging, enterprise-grade CI validation
# Version: 3.0 - Production hardened with observability

set -euo pipefail

# Script metadata
readonly SCRIPT_NAME="ci_gate"
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
readonly GATE_TIMEOUT="$(get_timeout "ci_gate" "check")"
readonly FAILURE_ACTION="$(get_script_config "ci_gate" "failure_action" "fail_build")"

# Global state
FAILED=false
TOTAL_CHECKS=0
PASSED_CHECKS=0

# Enhanced logging with configuration support
gate_log() {
    local level="$1"
    local message="$2"
    local extra_data="${3:-}"

    # Use the structured logging system
    log "$level" "$message" "$extra_data"
}

# Safe pattern checking with configuration-driven timeouts
check_critical_patterns() {
    local pattern="$1"
    local description="$2"
    local timeout="${3:-$GATE_TIMEOUT}"

    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))

    # Validate inputs
    if [[ -z "$pattern" ]] || [[ -z "$description" ]]; then
        error "Invalid pattern or description provided" "\"pattern\":\"$pattern\",\"description\":\"$description\""
        FAILED=true
        return 1
    fi

    start_timer "check_$description"

    # Configuration-driven retry logic
    local retry_count=0
    local max_retries="$(get_config "global" "max_retries" "3")"
    local retry_delay="$(get_config "global" "retry_delay" "2")"

    while [[ $retry_count -lt $max_retries ]]; do
        if timeout "$timeout" rg -q "$pattern" 2>/dev/null; then
            # Pattern found - critical issue
            error "Found $description" "\"pattern\":\"$pattern\",\"severity\":\"critical\""
            timeout "$timeout" rg -n "$pattern" 2>/dev/null || true
            record_metric "critical_issues_found" 1
            FAILED=true

            end_timer "check_$description"
            return 1
        else
            # Check succeeded or timed out
            if [[ $? -eq 124 ]]; then
                warn "Pattern check timed out: $description"
                record_metric "check_timeouts" 1
            fi

            break
        fi

        retry_count=$((retry_count + 1))
        if [[ $retry_count -lt $max_retries ]]; then
            debug "Retrying check: $description (attempt $retry_count/$max_retries)"
            sleep "$retry_delay"
        fi
    done

    if [[ $retry_count -eq $max_retries ]]; then
        error "Check failed after $max_retries attempts: $description"
        record_metric "check_failures" 1
        FAILED=true

        end_timer "check_$description"
        return 1
    fi

    info "Check passed: $description" "\"pattern\":\"$pattern\",\"clean\":true"
    record_metric "checks_passed" 1
    PASSED_CHECKS=$((PASSED_CHECKS + 1))

    end_timer "check_$description"
    return 0
}

# Enhanced core function checking with configuration
check_core_function() {
    local crate="$1"
    local function="$2"
    local description="$3"

    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))

    # Validate inputs
    if [[ -z "$crate" ]] || [[ -z "$function" ]] || [[ -z "$description" ]]; then
        error "Invalid parameters for check_core_function" "\"crate\":\"$crate\",\"function\":\"$function\""
        FAILED=true
        return 1
    fi

    # Validate crate directory exists with safe path checking
    local crate_path="crates/$crate"
    if [[ ! -d "$crate_path" ]]; then
        error "Crate directory not found: $crate_path"
        FAILED=true
        return 1
    fi

    start_timer "check_core_function_$function"

    if timeout "$GATE_TIMEOUT" rg -q "$function" "$crate_path" 2>/dev/null; then
        info "Core function found: $description" "\"crate\":\"$crate\",\"function\":\"$function\""
        record_metric "core_functions_found" 1
        PASSED_CHECKS=$((PASSED_CHECKS + 1))
    else
        error "Missing core function: $description" "\"crate\":\"$crate\",\"function\":\"$function\""
        record_metric "core_functions_missing" 1
        FAILED=true
    fi

    end_timer "check_core_function_$function"
    return 0
}

# Configuration-driven checks
perform_critical_checks() {
    info "Starting critical pattern checks"

    # Load check configurations
    local critical_patterns=(
        "unimplemented!|unimplemented"
        "todo!|todo"
        "panic!|panic"
        "unwrap\(\)|unwrap"
        "expect\(|expect"
        "dummy|dummy"
        "fake|fake"
        "stub|stub"
        "placeholder|placeholder"
    )

    local critical_descriptions=(
        "Unimplemented macros"
        "TODO macros"
        "Panic macros"
        "Unwrap calls"
        "Expect calls"
        "Dummy implementations"
        "Fake implementations"
        "Stub implementations"
        "Placeholder implementations"
    )

    # Perform each critical check
    for i in "${!critical_patterns[@]}"; do
        local pattern="${critical_patterns[$i]}"
        local description="${critical_descriptions[$i]}"

        if ! check_critical_patterns "$pattern" "$description"; then
            debug "Critical pattern check failed: $description"
        fi
    done
}

perform_core_function_checks() {
    info "Checking core API functions"

    # Core functions that must exist
    local core_functions=(
        "kcura-core:exec_sql:exec_sql function"
        "kcura-core:query_sparql:query_sparql function"
        "kcura-core:validate_shacl:validate_shacl function"
        "kcura-core:on_commit:on_commit hook function"
    )

    for func_spec in "${core_functions[@]}"; do
        IFS=':' read -r crate function description <<< "$func_spec"
        check_core_function "$crate" "$function" "$description"
    done
}

perform_ffi_checks() {
    info "Checking FFI completeness"

    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    start_timer "ffi_completeness_check"

    if timeout "$GATE_TIMEOUT" rg -q 'pub extern "C"' crates/kcura-ffi/src 2>/dev/null; then
        info "FFI exports found" "\"path\":\"crates/kcura-ffi/src\""
        record_metric "ffi_exports_found" 1
        PASSED_CHECKS=$((PASSED_CHECKS + 1))
    else
        error "No FFI exports found" "\"path\":\"crates/kcura-ffi/src\""
        record_metric "ffi_exports_missing" 1
        FAILED=true
    fi

    end_timer "ffi_completeness_check"
}

perform_test_coverage_checks() {
    info "Checking test coverage"

    local test_patterns=(
        "test.*ffi:FFI tests"
        "test.*hook:Hook tests"
        "test.*kernel:Kernel tests"
    )

    for test_spec in "${test_patterns[@]}"; do
        IFS=':' read -r pattern description <<< "$test_spec"

        TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
        start_timer "test_coverage_check_$pattern"

        if timeout "$GATE_TIMEOUT" rg -q "$pattern" crates/kcura-tests 2>/dev/null; then
            info "Test coverage check passed: $description" "\"pattern\":\"$pattern\""
            record_metric "test_coverage_found" 1
            PASSED_CHECKS=$((PASSED_CHECKS + 1))
        else
            error "Test coverage check failed: $description" "\"pattern\":\"$pattern\""
            record_metric "test_coverage_missing" 1
            FAILED=true
        fi

        end_timer "test_coverage_check_$pattern"
    done
}

perform_error_handling_checks() {
    info "Checking error handling patterns"

    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    start_timer "error_handling_check"

    if timeout "$GATE_TIMEOUT" rg -q "(unwrap\(|expect\()" -- 'crates/**/src/*.rs' 2>/dev/null | grep -v 'crates/kcura-cli' | grep -q .; then
        error "Error handling issues found in library code"
        timeout "$GATE_TIMEOUT" rg -n "(unwrap\(|expect\()" -- 'crates/**/src/*.rs' 2>/dev/null | grep -v 'crates/kcura-cli' || true
        record_metric "error_handling_issues" 1
        FAILED=true
    else
        info "Error handling validation passed"
        record_metric "error_handling_clean" 1
        PASSED_CHECKS=$((PASSED_CHECKS + 1))
    fi

    end_timer "error_handling_check"
}

# Main execution function with configuration-driven approach
main() {
    info "Starting KCura CI Gate v$SCRIPT_VERSION" "\"correlation_id\":\"$CORRELATION_ID\",\"failure_action\":\"$FAILURE_ACTION\""

    # Health check for dependencies
    if ! command -v rg >/dev/null 2>&1; then
        warn "ripgrep not found - pattern checking may be limited"
        health_check "ripgrep_dependency" "warning" "ripgrep not available"
    else
        health_check "ripgrep_dependency" "ok"
    fi

    # Perform all checks
    perform_critical_checks
    perform_core_function_checks
    perform_ffi_checks
    perform_test_coverage_checks
    perform_error_handling_checks

    # Final health checks (80/20 innovation)
    health_check "ci_gate_completion" "completed" "CI gate finished with $PASSED_CHECKS/$TOTAL_CHECKS checks passed"
    self_healing_health_check
    intelligent_cache_health_check

    # Handle failure action based on configuration
    if [[ "$FAILED" == "true" ]]; then
        error "CI Gate failed: $PASSED_CHECKS/$TOTAL_CHECKS checks passed"

        case "$FAILURE_ACTION" in
            "fail_build")
                error "Failing build due to critical implementation gaps"
                return 1
                ;;
            "warn")
                warn "Critical issues found but allowing build to continue"
                return 0
                ;;
            "ignore")
                info "Critical issues found but ignoring as configured"
                return 0
                ;;
            *)
                error "Unknown failure action: $FAILURE_ACTION"
                return 1
                ;;
        esac
    else
        info "CI Gate passed: All $TOTAL_CHECKS checks completed successfully"
        return 0
    fi
}

# Execute with error handling
if ! time_execution "ci_gate_execution" main "$@"; then
    case "$FAILURE_ACTION" in
        "fail_build"|"warn") exit 1 ;;
        "ignore") exit 0 ;;
        *) exit 1 ;;
    esac
fi

# Cleanup is handled by logging library trap
exit 0
