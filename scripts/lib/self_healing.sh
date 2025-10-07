#!/usr/bin/env bash
# KCura Self-Healing Configuration System
# Auto-repairs configuration issues and learns optimal settings
# Version: 1.0 - 80/20 innovation for maximum value

set -euo pipefail

# Self-healing configuration paths (initialize only if not already set)
if [[ -z "${HEALING_LOG_DIR+x}" ]]; then
    local healing_dir=""
    healing_dir="$(get_script_config "temp" "healing_dir" "$(get_config "global" "healing_dir" "/tmp")")"
    readonly HEALING_LOG_DIR="$healing_dir/kcura-healing"
fi
readonly PERFORMANCE_HISTORY="${HEALING_LOG_DIR}/performance_history.json"
readonly CONFIG_BACKUP_DIR="${HEALING_LOG_DIR}/config_backups"
readonly LEARNING_CACHE="${HEALING_LOG_DIR}/learning_cache.json"

# Performance learning data (initialize only if not already set)
if [[ -z "${PERFORMANCE_HISTORY_CACHE+a}" ]]; then
    declare -A PERFORMANCE_HISTORY_CACHE=()
fi
if [[ -z "${ADAPTIVE_SETTINGS+a}" ]]; then
    declare -A ADAPTIVE_SETTINGS=()
fi

# Initialize self-healing system
init_self_healing() {
    mkdir -p "$HEALING_LOG_DIR" "$CONFIG_BACKUP_DIR"

    # Load performance history if available
    if [[ -f "$PERFORMANCE_HISTORY" ]]; then
        while IFS='=' read -r key value; do
            PERFORMANCE_HISTORY_CACHE["$key"]="$value"
        done < <(jq -r 'to_entries[] | "\(.key)=\(.value)"' "$PERFORMANCE_HISTORY" 2>/dev/null || true)
    fi

    # Load adaptive settings cache
    if [[ -f "$LEARNING_CACHE" ]]; then
        while IFS='=' read -r key value; do
            ADAPTIVE_SETTINGS["$key"]="$value"
        done < <(jq -r 'to_entries[] | "\(.key)=\(.value)"' "$LEARNING_CACHE" 2>/dev/null || true)
    fi

    debug "Self-healing system initialized"
}

# Self-healing configuration repair
repair_configuration() {
    local config_file="$1"

    if [[ ! -f "$config_file" ]]; then
        warn "Configuration file missing: $config_file"

        # Generate default configuration
        if generate_optimal_config "$config_file"; then
            info "Generated optimal configuration: $config_file"
            return 0
        else
            error "Failed to generate configuration: $config_file"
            return 1
        fi
    fi

    # Validate existing configuration
    if ! validate_config_integrity "$config_file"; then
        warn "Configuration integrity issues detected: $config_file"

        # Backup current config
        local backup_file="$CONFIG_BACKUP_DIR/$(basename "$config_file").$(date +%Y%m%d_%H%M%S)"
        cp "$config_file" "$backup_file"
        info "Backed up configuration: $backup_file"

        # Attempt repair
        if repair_config_issues "$config_file"; then
            info "Configuration repaired successfully: $config_file"
            return 0
        else
            warn "Configuration repair failed, using backup"
            cp "$backup_file" "$config_file"
            return 1
        fi
    fi

    return 0
}

# Generate optimal configuration based on environment and history
generate_optimal_config() {
    local config_file="$1"

    # Detect environment characteristics
    local cpu_cores=$(nproc 2>/dev/null || echo "4")
    local total_memory=$(free -m 2>/dev/null | awk 'NR==2{print $2}' || echo "8192")
    local disk_space=$(df . 2>/dev/null | awk 'NR==2{print $4}' || echo "1000000")

    # Use performance history for optimal settings
    local optimal_timeout="${PERFORMANCE_HISTORY_CACHE[avg_timeout]:-30}"
    local optimal_workers="${PERFORMANCE_HISTORY_CACHE[optimal_workers]:-$((cpu_cores / 2))}"
    local optimal_cache_ttl="${PERFORMANCE_HISTORY_CACHE[optimal_cache_ttl]:-300}"

    # Generate optimized configuration
    cat > "$config_file" << EOF
# Auto-generated KCura configuration
# Generated on: $(date)
# Environment: $(detect_environment)
# System: $cpu_cores CPU cores, ${total_memory}MB RAM

global:
  log_level: "$(get_env_log_level)"
  log_format: "json"
  enable_metrics: true
  enable_audit: true
  timeout_default: $optimal_timeout
  max_retries: $(get_optimal_retries)
  retry_delay: 2

scan_fakes:
  enabled: true
  timeout: $optimal_timeout
  max_workers: $optimal_workers
  cache_results: true
  cache_ttl: $optimal_cache_ttl

ci_gate:
  enabled: true
  timeout: $((optimal_timeout * 2))
  failure_action: "$(get_env_failure_action)"

smoke_tests:
  enabled: true
  timeout_per_test: $optimal_timeout
  max_retries: $(get_optimal_retries)
  parallel_execution: $(get_parallel_execution_setting "$cpu_cores")

performance:
  parallel_processing: true
  caching_enabled: true
  max_memory_mb: $(get_optimal_memory "$total_memory")
  max_cpu_percent: 80

# Environment-specific overrides
$(get_environment_overrides)
EOF

    return $?
}

# Get optimal log level based on environment
get_env_log_level() {
    case "$(detect_environment)" in
        "production") echo "warn" ;;
        "staging") echo "info" ;;
        *) echo "debug" ;;
    esac
}

# Get optimal retry count based on environment
get_optimal_retries() {
    case "$(detect_environment)" in
        "production") echo "2" ;;
        "staging") echo "3" ;;
        *) echo "1" ;;
    esac
}

# Get failure action based on environment
get_env_failure_action() {
    case "$(detect_environment)" in
        "production") echo "fail_build" ;;
        "staging") echo "fail_build" ;;
        *) echo "warn" ;;
    esac
}

# Get parallel execution setting based on CPU cores
get_parallel_execution_setting() {
    local cpu_cores="$1"
    [[ $cpu_cores -gt 4 ]] && echo "true" || echo "false"
}

# Get optimal memory limit based on total memory
get_optimal_memory() {
    local total_memory="$1"
    local available_memory=$((total_memory * 80 / 100))  # 80% of total
    echo $((available_memory > 512 ? available_memory : 512))
}

# Get environment-specific overrides
get_environment_overrides() {
    local env="$(detect_environment)"

    case "$env" in
        "development")
            echo "development:
  global:
    log_level: \"debug\"
    enable_metrics: true
  scan_fakes:
    timeout: 15
  ci_gate:
    failure_action: \"warn\""
            ;;
        "production")
            echo "production:
  global:
    log_level: \"warn\"
    enable_metrics: true
  security:
    input_validation: true
    safe_execution: true
    audit_trail: true
  performance:
    parallel_processing: true
    caching_enabled: true"
            ;;
        *)
            echo "# No environment-specific overrides for $env"
            ;;
    esac
}

# Validate configuration integrity
validate_config_integrity() {
    local config_file="$1"

    # Check if file exists and is readable
    [[ ! -f "$config_file" ]] && return 1
    [[ ! -r "$config_file" ]] && return 1

    # Check YAML syntax (basic)
    if command -v yamllint >/dev/null 2>&1; then
        yamllint "$config_file" >/dev/null 2>&1 || return 1
    fi

    # Check for required sections
    local required_sections=("global" "scan_fakes" "ci_gate" "smoke_tests")
    for section in "${required_sections[@]}"; do
        grep -q "^$section:" "$config_file" || return 1
    done

    return 0
}

# Attempt to repair configuration issues
repair_config_issues() {
    local config_file="$1"

    # Create a temporary repaired version
    local temp_config=$(mktemp)

    # Basic repair: add missing required sections
    {
        # Copy original content
        cat "$config_file"

        # Add missing sections if needed
        local missing_sections=()

        grep -q "^global:" "$config_file" || missing_sections+=("global")
        grep -q "^scan_fakes:" "$config_file" || missing_sections+=("scan_fakes")
        grep -q "^ci_gate:" "$config_file" || missing_sections+=("ci_gate")
        grep -q "^smoke_tests:" "$config_file" || missing_sections+=("smoke_tests")

        for section in "${missing_sections[@]}"; do
            echo ""
            echo "$section:"
            echo "  enabled: true"
            case "$section" in
                "global")
                    echo "  log_level: \"info\""
                    echo "  timeout_default: 60"
                    ;;
                "scan_fakes"|"ci_gate"|"smoke_tests")
                    echo "  timeout: 60"
                    ;;
            esac
        done
    } > "$temp_config"

    # Validate repaired config
    if validate_config_integrity "$temp_config"; then
        mv "$temp_config" "$config_file"
        return 0
    else
        rm -f "$temp_config"
        return 1
    fi
}

# Learn from execution performance
learn_from_performance() {
    local operation="$1"
    local duration_ms="$2"
    local success="$3"

    # Update performance history
    local history_key="perf_${operation}_total"
    local count_key="perf_${operation}_count"
    local avg_key="perf_${operation}_avg"

    local total_duration=${PERFORMANCE_HISTORY_CACHE[$history_key]:-0}
    local execution_count=${PERFORMANCE_HISTORY_CACHE[$count_key]:-0}

    total_duration=$((total_duration + duration_ms))
    execution_count=$((execution_count + 1))

    PERFORMANCE_HISTORY_CACHE[$history_key]="$total_duration"
    PERFORMANCE_HISTORY_CACHE[$count_key]="$execution_count"
    PERFORMANCE_HISTORY_CACHE[$avg_key]=$((total_duration / execution_count))

    # Adapt settings based on performance
    adapt_settings "$operation" "$duration_ms" "$success"

    # Persist learning data periodically
    if [[ $((execution_count % 10)) -eq 0 ]]; then
        save_performance_history
    fi
}

# Adapt settings based on performance data
adapt_settings() {
    local operation="$1"
    local duration_ms="$2"
    local success="$3"

    # Adaptive timeout based on performance
    if [[ $duration_ms -gt 0 ]]; then
        local current_timeout=${ADAPTIVE_SETTINGS["timeout_${operation}"]:-30}

        if [[ $success == "true" ]] && [[ $duration_ms -lt $((current_timeout * 1000 * 80 / 100)) ]]; then
            # Performance is good, could potentially reduce timeout
            ADAPTIVE_SETTINGS["timeout_${operation}"]=$((current_timeout * 90 / 100))
        elif [[ $success != "true" ]] || [[ $duration_ms -gt $((current_timeout * 1000 * 120 / 100)) ]]; then
            # Performance issues, increase timeout
            ADAPTIVE_SETTINGS["timeout_${operation}"]=$((current_timeout * 110 / 100))
        fi
    fi

    # Adaptive worker count based on system performance
    local cpu_usage=$(get_cpu_usage 2>/dev/null || echo "50")
    if [[ $cpu_usage -lt 70 ]]; then
        ADAPTIVE_SETTINGS["max_workers"]="$(get_config "global" "max_workers" "2")"
    else
        ADAPTIVE_SETTINGS["max_workers"]="1"
    fi
}

# Get current CPU usage
get_cpu_usage() {
    # Linux
    if [[ -f /proc/stat ]]; then
        awk '/^cpu / {usage=($2+$4)*100/($2+$4+$5)} END {print int(usage)}' /proc/stat 2>/dev/null || echo "50"
    # macOS
    elif command -v top >/dev/null 2>&1; then
        top -l 1 | awk '/CPU usage/ {print int($3)}' 2>/dev/null || echo "50"
    else
        echo "50"
    fi
}

# Save performance history
save_performance_history() {
    # Convert associative array to JSON-like format
    {
        echo "{"
        local first=true
        for key in "${!PERFORMANCE_HISTORY_CACHE[@]}"; do
            [[ $first == false ]] && echo ","
            echo "  \"$key\": ${PERFORMANCE_HISTORY_CACHE[$key]}"
            first=false
        done
        echo "}"
    } > "$PERFORMANCE_HISTORY"

    # Save adaptive settings
    {
        echo "{"
        local first=true
        for key in "${!ADAPTIVE_SETTINGS[@]}"; do
            [[ $first == false ]] && echo ","
            echo "  \"$key\": \"${ADAPTIVE_SETTINGS[$key]}\""
            first=false
        done
        echo "}"
    } > "$LEARNING_CACHE"

    debug "Performance history and adaptive settings saved"
}

# Get adaptive setting with fallback
get_adaptive_setting() {
    local key="$1"
    local default_value="$2"

    echo "${ADAPTIVE_SETTINGS[$key]:-$default_value}"
}

# Health check for self-healing system
self_healing_health_check() {
    local issues=0

    # Check if performance history is writable
    if ! touch "$PERFORMANCE_HISTORY" 2>/dev/null; then
        warn "Cannot write to performance history: $PERFORMANCE_HISTORY"
        issues=$((issues + 1))
    fi

    # Check if learning cache is writable
    if ! touch "$LEARNING_CACHE" 2>/dev/null; then
        warn "Cannot write to learning cache: $LEARNING_CACHE"
        issues=$((issues + 1))
    fi

    if [[ $issues -eq 0 ]]; then
        health_check "self_healing_system" "ok" "Self-healing system operational"
        return 0
    else
        health_check "self_healing_system" "warning" "$issues configuration issues detected"
        return 1
    fi
}

# Export functions for use in other scripts
export -f init_self_healing repair_configuration generate_optimal_config
export -f learn_from_performance adapt_settings save_performance_history
export -f get_adaptive_setting self_healing_health_check

# Initialize if this script is run directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    echo "KCura Self-Healing System v1.0"
    echo "Usage: source this file in your scripts"
    echo ""
    echo "Functions available:"
    echo "  init_self_healing - Initialize self-healing system"
    echo "  repair_configuration <file> - Auto-repair configuration issues"
    echo "  generate_optimal_config <file> - Generate optimal configuration"
    echo "  learn_from_performance <op> <duration> <success> - Learn from execution"
    echo "  get_adaptive_setting <key> [default] - Get adaptive setting"
    echo "  self_healing_health_check - Check system health"
    echo ""
    echo "Environment variables:"
    echo "  KC_HEALING_LOG_DIR - Directory for healing logs (default: /tmp/kcura-healing)"
fi
