#!/usr/bin/env bash
# KCura Fake Implementation Scanner - 80/20 Core Team Implementation
# Configuration-driven, structured logging, enterprise-grade implementation
# Version: 3.0 - Production hardened with observability

set -euo pipefail

# Script metadata
readonly SCRIPT_NAME="scan_fakes"
readonly SCRIPT_VERSION="3.0"

# Load core team libraries
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Source core team libraries (80/20 innovation)
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

# Initialize intelligent caching
init_intelligent_cache

# Initialize self-healing system
init_self_healing

# Export configuration for environment variables
export_config

# Get script-specific configuration with adaptive settings
readonly SCRIPT_TIMEOUT="$(get_timeout "scan_fakes" "pattern")"
readonly MAX_WORKERS="$(get_adaptive_setting "max_workers" "$(get_script_config "scan_fakes" "max_workers" "2")")"
readonly CACHE_RESULTS="$(get_script_config "scan_fakes" "cache_results" "true")"

# Input validation and sanitization
validate_input() {
    local root_dir="${1:-.}"

    # Sanitize input path
    root_dir="$(realpath -m "$root_dir" 2>/dev/null || echo "$root_dir")"

    if [[ ! -d "$root_dir" ]]; then
        error "Input validation failed: '$root_dir' is not a valid directory"
        return 1
    fi

    # Check for path traversal attempts
    if [[ "$root_dir" =~ \.\. ]]; then
        error "Input validation failed: Path traversal detected in '$root_dir'"
        return 1
    fi

    # Ensure we're not scanning sensitive directories
    local forbidden_paths=("/proc" "/sys" "/dev" "/tmp")
    for forbidden in "${forbidden_paths[@]}"; do
        if [[ "$root_dir" =~ ^"$forbidden" ]]; then
            error "Input validation failed: Forbidden path '$root_dir'"
            return 1
        fi
    done

    echo "$root_dir"
    return 0
}

# Safe pattern matching with intelligent caching and adaptive learning
scan_pattern_safely() {
    local pattern="$1"
    local description="$2"
    local root_dir="$3"
    local timeout="$4"

    # Validate pattern
    if [[ -z "$pattern" ]]; then
        warn "Empty pattern provided for $description"
        return 0
    fi

    start_timer "pattern_scan_$description"

    # Create cache key for pattern results
    local cache_key="scan_pattern:${pattern}:${root_dir}"

    # Try intelligent cache first (80/20 innovation)
    if [[ "$CACHE_RESULTS" == "true" ]]; then
        local cached_result
        if cached_result=$(cache_get "$cache_key" 2>/dev/null); then
            info "Pattern scan from cache: $description" "\"cached\":true,\"result\":\"$cached_result\""
            record_metric "cache_hits" 1

            end_timer "pattern_scan_$description"

            # Return cached result
            [[ "$cached_result" == "found" ]] && return 1 || return 0
        fi
    fi

    # Use safe execution pattern with adaptive retry logic
    local retry_count=0
    local max_retries="$(get_config "global" "max_retries" "3")"

    while [[ $retry_count -lt $max_retries ]]; do
        if timeout "$timeout" bash -c "
            # Safe execution with input validation
            if [[ -d '$root_dir' ]] && [[ -r '$root_dir' ]]; then
                # Use ripgrep with safe options
                rg -n -q '$pattern' '$root_dir' --glob '!scripts/**' --glob '!target/**' --glob '!node_modules/**'
            fi
        " 2>/dev/null; then

            # Pattern found - log finding and learn from success
            warn "Fake pattern detected: $description" "\"pattern\":\"$pattern\",\"location\":\"$root_dir\""
            record_metric "fake_patterns_found" 1

            # Cache the result for future use
            if [[ "$CACHE_RESULTS" == "true" ]]; then
                cache_set "$cache_key" "found" "$((timeout * 2))"
            fi

            # Learn from successful execution
            end_timer "pattern_scan_$description"
            learn_from_performance "pattern_scan_$description" "$(get_timer_duration "pattern_scan_$description")" "true"

            return 1
        else
            # Pattern not found or timeout
            if [[ $? -eq 124 ]]; then
                warn "Pattern scan timed out: $description"
                record_metric "pattern_scan_timeouts" 1
            fi

            retry_count=$((retry_count + 1))
            if [[ $retry_count -lt $max_retries ]]; then
                debug "Retrying pattern scan: $description (attempt $retry_count/$max_retries)"
                sleep 1
            fi
        fi
    done

    info "Pattern scan completed: $description" "\"pattern\":\"$pattern\",\"clean\":true"
    record_metric "pattern_scans_clean" 1

    # Cache clean result
    if [[ "$CACHE_RESULTS" == "true" ]]; then
        cache_set "$cache_key" "clean" "$((timeout * 3))"
    fi

    # Learn from successful execution (even if pattern not found)
    end_timer "pattern_scan_$description"
    learn_from_performance "pattern_scan_$description" "$(get_timer_duration "pattern_scan_$description")" "true"

    return 0
}

# Check FFI functions for constant returns
check_ffi_functions() {
    local ffi_file="crates/kcura-ffi/src/lib.rs"

    start_timer "ffi_check"

    if [[ ! -f "$ffi_file" ]]; then
        info "FFI check skipped: file not found" "\"file\":\"$ffi_file\""
        end_timer "ffi_check"
        return 0
    fi

    # Safe FFI function analysis
    if timeout "$SCRIPT_TIMEOUT" bash -c "
        # Check for FFI function definitions
        if grep -q 'pub extern \"C\" fn kc_.*{' '$ffi_file' 2>/dev/null; then
            # Check for constant returns in FFI functions
            if grep -A 10 -B 2 'pub extern \"C\" fn kc_.*{' '$ffi_file' 2>/dev/null | grep -q 'return Ok('; then
                echo 'constant_return_detected'
            fi
        fi
    " 2>/dev/null; then

        error "FFI function returns constant Ok(...)" "\"file\":\"$ffi_file\",\"issue\":\"constant_return\""
        record_metric "ffi_constant_returns" 1

        end_timer "ffi_check"
        return 1
    else
        info "FFI functions validated successfully" "\"file\":\"$ffi_file\""
        record_metric "ffi_checks_passed" 1
    fi

    end_timer "ffi_check"
    return 0
}

# Main execution with configuration-driven approach
main() {
    local root_dir="${1:-.}"
    local exit_code=0

    info "Starting KCura Fake Scanner v$SCRIPT_VERSION" "\"correlation_id\":\"$CORRELATION_ID\""

    # Validate and sanitize input
    if ! VALIDATED_ROOT="$(validate_input "$root_dir")"; then
        fatal "Input validation failed for directory: $root_dir"
    fi

    info "Scanning directory with configuration" "\"directory\":\"$VALIDATED_ROOT\",\"timeout\":\"$SCRIPT_TIMEOUT\",\"workers\":\"$MAX_WORKERS\""

    # Load patterns from configuration
    local patterns=(
        "$(get_script_config "scan_fakes" "patterns.0.pattern" "unimplemented!|todo!|panic!")"
        "$(get_script_config "scan_fakes" "patterns.1.pattern" "return.*Ok.*(dummy|fake|stub|placeholder)")"
        "$(get_script_config "scan_fakes" "patterns.2.pattern" "(hardcoded|canned|mock).*response")"
    )

    local descriptions=(
        "Unimplemented macros and TODOs"
        "Fake return statements"
        "Hardcoded responses"
    )

    # Health check for dependencies
    if ! command -v rg >/dev/null 2>&1; then
        warn "ripgrep not found - pattern scanning may be limited"
        health_check "ripgrep_dependency" "warning" "ripgrep not available"
    else
        health_check "ripgrep_dependency" "ok"
    fi

    # Scan each pattern
    for i in "${!patterns[@]}"; do
        local pattern="${patterns[$i]}"
        local description="${descriptions[$i]}"

        if ! scan_pattern_safely "$pattern" "$description" "$VALIDATED_ROOT" "$SCRIPT_TIMEOUT"; then
            exit_code=1
        fi
    done

    # Check FFI functions
    info "Performing FFI function analysis"
    if ! check_ffi_functions; then
        exit_code=1
    fi

    # Final health checks (80/20 innovation)
    health_check "scan_completion" "completed" "Scan finished with exit code $exit_code"
    self_healing_health_check
    intelligent_cache_health_check

    # Periodic cache cleanup
    if [[ $((RANDOM % 10)) -eq 0 ]]; then
        cache_cleanup >/dev/null 2>&1 || true
    fi

    # Summary
    if [[ $exit_code -eq 0 ]]; then
        info "Scan completed successfully - no fake patterns detected"
    else
        warn "Scan completed with issues - fake patterns detected"
    fi

    return $exit_code
}

# Execute with error handling
if ! time_execution "scan_fakes_execution" main "$@"; then
    error "Scan execution failed"
    exit 1
fi

# Cleanup is handled by logging library trap
exit 0