#!/usr/bin/env bash
# KCura Structured Logging Library
# Provides structured JSON logging with correlation IDs and metrics
# Version: 1.0 - 80/20 implementation

set -euo pipefail

# Logging configuration (initialize only if not already set)
if [[ -z "${LOG_LEVEL+x}" ]]; then
    readonly LOG_LEVEL="${KC_LOG_LEVEL:-${LOG_LEVEL:-info}}"
fi
if [[ -z "${LOG_FORMAT+x}" ]]; then
    readonly LOG_FORMAT="${KC_LOG_FORMAT:-${LOG_FORMAT:-json}}"
fi

# Determine script name safely (go up from lib/ to scripts/)
if [[ -z "${SCRIPT_NAME:-}" ]]; then
    if [[ -n "${BASH_SOURCE[1]:-}" ]] && [[ "${BASH_SOURCE[1]}" != "${0:-}" ]]; then
        # Extract script name from the calling script path
        local calling_script="$(dirname "${BASH_SOURCE[1]}")"
        readonly SCRIPT_NAME="$(basename "$calling_script")"
    else
        readonly SCRIPT_NAME="unknown-script"
    fi
fi

readonly CORRELATION_ID="${KC_CORRELATION_ID:-$(uuidgen 2>/dev/null || echo "no-uuid-$(date +%s)-$$")}"

# Log level hierarchy (initialize only if not already set)
if [[ -z "${LOG_LEVELS+a}" ]]; then
    declare -A LOG_LEVELS=(
        ["TRACE"]=0
        ["DEBUG"]=1
        ["INFO"]=2
        ["WARN"]=3
        ["ERROR"]=4
        ["FATAL"]=5
    )
fi

# Global metrics collection (initialize only if not already set)
if [[ -z "${METRICS_START_TIME:-}" ]]; then
    if [[ -z "${METRICS+a}" ]]; then
        declare -A METRICS=()
    fi
    readonly METRICS_START_TIME=$(date '+%s%N')
fi

# Color codes for simple format (initialize only if not already set)
if [[ -z "${RED+x}" ]]; then
    readonly RED='\033[0;31m'
    readonly GREEN='\033[0;32m'
    readonly YELLOW='\033[1;33m'
    readonly BLUE='\033[0;34m'
    readonly PURPLE='\033[0;35m'
    readonly CYAN='\033[0;36m'
    readonly NC='\033[0m'
fi

# Should log function
should_log() {
    local level="$1"
    local current_level="${LOG_LEVELS[$LOG_LEVEL]:-2}"

    [[ "${LOG_LEVELS[$level]:-999}" -ge "$current_level" ]]
}

# Format timestamp
format_timestamp() {
    if [[ "$LOG_FORMAT" == "json" ]]; then
        date -u '+%Y-%m-%dT%H:%M:%SZ'
    else
        date '+%H:%M:%S'
    fi
}

# Structured logging function
log() {
    local level="$1"
    local message="$2"
    local extra_data="${3:-}"

    # Check if we should log this level
    if ! should_log "$level"; then
        return 0
    fi

    local timestamp
    timestamp=$(format_timestamp)

    # Collect additional context
    local context_data=""

    # Add metrics if available
    if [[ ${#METRICS[@]} -gt 0 ]]; then
        context_data="\"metrics\":{"
        local first=true
        for key in "${!METRICS[@]}"; do
            [[ $first == false ]] && context_data+=","
            context_data+="\"$key\":${METRICS[$key]}"
            first=false
        done
        context_data+="},"
    fi

    # Add extra data if provided
    if [[ -n "$extra_data" ]]; then
        context_data+="$extra_data"
    fi

    if [[ "$LOG_FORMAT" == "json" ]]; then
        # JSON structured logging
        local json_log
        printf -v json_log '{"timestamp":"%s","level":"%s","script":"%s","correlation_id":"%s","message":"%s"%s}' \
            "$timestamp" "$level" "$SCRIPT_NAME" "$CORRELATION_ID" "$message" \
            "${context_data:+,${context_data%,}}"

        echo "$json_log"
    else
        # Simple colored logging
        local color
        case "$level" in
            "TRACE"|"DEBUG") color="$CYAN" ;;
            "INFO") color="$GREEN" ;;
            "WARN") color="$YELLOW" ;;
            "ERROR"|"FATAL") color="$RED" ;;
            *) color="$NC" ;;
        esac

        printf "%s[%s] %s: %s%s\n" "$color" "$timestamp" "$level" "$message" "$NC"
    fi

    # Audit trail for important events
    if [[ "${KC_ENABLE_AUDIT:-false}" == "true" ]] && [[ "$level" =~ ^(ERROR|FATAL|WARN)$ ]]; then
        local audit_dir=""
        audit_dir="$(get_script_config "temp" "audit_dir" "$(get_config "global" "audit_dir" "/tmp")")"
        mkdir -p "$audit_dir" 2>/dev/null || true
        echo "$(date '+%Y-%m-%d %H:%M:%S') [$level] $SCRIPT_NAME($CORRELATION_ID): $message" >> "$audit_dir/kcura_audit.log"
    fi
}

# Convenience logging functions
trace() { log "TRACE" "${1:-}" "${2:-}"; }
debug() { log "DEBUG" "${1:-}" "${2:-}"; }
info() { log "INFO" "${1:-}" "${2:-}"; }
warn() { log "WARN" "${1:-}" "${2:-}"; }
error() { log "ERROR" "${1:-}" "${2:-}"; }
fatal() { log "FATAL" "${1:-}" "${2:-}"; exit 1; }

# Metrics collection
record_metric() {
    local metric_name="$1"
    local value="${2:-1}"
    local operation="${3:-inc}"

    case "$operation" in
        "inc")
            local current_value="${METRICS[$metric_name]:-0}"
            METRICS["$metric_name"]=$((current_value + value))
            ;;
        "set") METRICS["$metric_name"]="$value" ;;
        "add")
            local current_value="${METRICS[$metric_name]:-0}"
            METRICS["$metric_name"]=$((current_value + value))
            ;;
    esac

    debug "Metric recorded: $metric_name = ${METRICS[$metric_name]} ($operation)"
}

# Timing functions
start_timer() {
    local timer_name="$1"
    METRICS["timer_${timer_name}_start"]=$(date '+%s%N')
    debug "Timer started: $timer_name"
}

end_timer() {
    local timer_name="$1"
    local start_key="timer_${timer_name}_start"

    if [[ -z "${METRICS[$start_key]:-}" ]]; then
        warn "Timer '$timer_name' was not started"
        return 1
    fi

    local end_time
    end_time=$(date '+%s%N')
    local duration_ns=$((end_time - METRICS[$start_key]))
    local duration_ms=$((duration_ns / 1000000))

    METRICS["timer_${timer_name}_duration_ms"]="$duration_ms"
    unset "METRICS[$start_key]"

    info "Timer completed: $timer_name took ${duration_ms}ms"
    return 0
}

# Execution time decorator
time_execution() {
    local operation_name="$1"
    shift

    start_timer "$operation_name"
    local exit_code=0

    "$@" || exit_code=$?

    end_timer "$operation_name"
    return $exit_code
}

# Health check function
health_check() {
    local component="$1"
    local status="${2:-ok}"
    local message="${3:-}"

    if [[ "$status" == "ok" ]]; then
        record_metric "health_check_passed" 1
        info "Health check passed: $component" "$message"
    else
        record_metric "health_check_failed" 1
        warn "Health check failed: $component" "$message"
    fi

    if [[ "${KC_ENABLE_METRICS:-true}" == "true" ]]; then
        # Export metrics in Prometheus format
        echo "# HELP kcura_health_check_status Health check status (1=ok, 0=failed)"
        echo "# TYPE kcura_health_check_status gauge"
        echo "kcura_health_check_status{component=\"$component\"} $([[ $status == ok ]] && echo 1 || echo 0)"
    fi
}

# Cleanup function
cleanup_logging() {
    local end_time
    end_time=$(date '+%s%N')
    local total_duration_ns=$((end_time - METRICS_START_TIME))
    local total_duration_ms=$((total_duration_ns / 1000000))

    record_metric "execution_time_ms" "$total_duration_ms"

    debug "Script execution completed in ${total_duration_ms}ms"
    debug "Total metrics collected: ${#METRICS[@]}"

    # Export final metrics if enabled
    if [[ "${KC_ENABLE_METRICS:-true}" == "true" ]] && [[ ${#METRICS[@]} -gt 0 ]]; then
        echo ""
        echo "# KCura Script Metrics"
        echo "# Script: $SCRIPT_NAME"
        echo "# Correlation ID: $CORRELATION_ID"
        echo "# Execution time: ${total_duration_ms}ms"
        echo ""

        for metric in "${!METRICS[@]}"; do
            echo "kcura_script_${metric//[^a-zA-Z0-9_]/_} ${METRICS[$metric]}"
        done
    fi
}

# Set up cleanup trap
trap cleanup_logging EXIT

# Initialize logging system
initialize_logging() {
    info "Logging system initialized" "\"level\":\"$LOG_LEVEL\",\"format\":\"$LOG_FORMAT\",\"correlation_id\":\"$CORRELATION_ID\""

    # Set up audit directory if audit is enabled
    if [[ "${KC_ENABLE_AUDIT:-false}" == "true" ]]; then
        local audit_dir=""
        audit_dir="$(get_script_config "temp" "audit_dir" "$(get_config "global" "audit_dir" "/tmp")")"
        mkdir -p "$audit_dir" 2>/dev/null || true
        health_check "logging_system" "ok" "Audit logging enabled"
    fi
}

# Performance monitoring
monitor_performance() {
    local operation="$1"
    local duration_ms="$2"

    record_metric "operation_duration_ms" "$duration_ms"

    # Warn about slow operations
    if [[ $duration_ms -gt 5000 ]]; then
        warn "Slow operation detected: $operation took ${duration_ms}ms"
    fi
}

# Get timer duration (helper for performance learning)
get_timer_duration() {
    local timer_name="$1"
    local duration_key="timer_${timer_name}_duration_ms"

    # Check if timer exists in metrics
    if [[ -n "${METRICS[$duration_key]:-}" ]]; then
        echo "${METRICS[$duration_key]}"
        return 0
    fi

    # Fallback: estimate based on current time
    local current_time=$(date '+%s%N')
    local estimated_duration=$(( (current_time - METRICS_START_TIME) / 1000000 ))
    echo "$estimated_duration"
}

# Export logging functions for use in other scripts
export -f log trace debug info warn error fatal
export -f record_metric start_timer end_timer time_execution get_timer_duration
export -f health_check cleanup_logging initialize_logging
export -f monitor_performance

# Initialize if this script is run directly
if [[ "${BASH_SOURCE[0]:-}" == "${0:-}" ]] || [[ -z "${BASH_SOURCE[0]:-}" && -n "${0:-}" ]]; then
    echo "KCura Logging Library v1.0"
    echo "Usage: source this file in your scripts"
    echo ""
    echo "Functions available:"
    echo "  log <level> <message> [extra_json] - Core logging function"
    echo "  trace|debug|info|warn|error|fatal <message> [extra_json] - Convenience functions"
    echo "  record_metric <name> [value] [operation] - Record metrics"
    echo "  start_timer|end_timer <name> - Time operations"
    echo "  time_execution <name> <command...> - Time command execution"
    echo "  health_check <component> <status> [message] - Health checks"
    echo "  initialize_logging - Initialize logging system"
    echo ""
    echo "Environment variables:"
    echo "  KC_LOG_LEVEL - Log level (trace, debug, info, warn, error)"
    echo "  KC_LOG_FORMAT - Log format (json, simple)"
    echo "  KC_CORRELATION_ID - Request correlation ID"
    echo "  KC_ENABLE_METRICS - Enable metrics collection (true/false)"
    echo "  KC_ENABLE_AUDIT - Enable audit logging (true/false)"
fi
