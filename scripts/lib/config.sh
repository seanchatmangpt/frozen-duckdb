#!/usr/bin/env bash
# KCura Configuration Management Library
# Provides configuration loading, validation, and environment-specific overrides
# Version: 1.0 - 80/20 implementation

set -euo pipefail

# Configuration constants (initialize only if not already set)
if [[ -z "${CONFIG_FILE_DEFAULT+x}" ]]; then
    readonly CONFIG_FILE_DEFAULT="${CONFIG_FILE_DEFAULT:-kcura-config.yaml}"
fi
if [[ -z "${CONFIG_FILE+x}" ]]; then
    readonly CONFIG_FILE="${CONFIG_FILE:-$CONFIG_FILE_DEFAULT}"
fi

# Determine script directory safely (80/20 implementation)
if [[ -z "${SCRIPT_DIR+x}" ]]; then
    # For the 80/20 implementation, use BASH_SOURCE when available
    if [[ -n "${BASH_SOURCE[0]:-}" ]]; then
        readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
    else
        # Fallback for when sourced directly - use current working directory
        readonly SCRIPT_DIR="$(pwd)/scripts"
    fi
fi

if [[ -z "${PROJECT_ROOT+x}" ]]; then
    readonly PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
fi

# Global configuration cache (initialize only if not already set)
if [[ -z "${CONFIG_CACHE+a}" ]]; then
    declare -A CONFIG_CACHE=()
fi

# Logging interface for configuration operations
config_log() {
    local level="$1"
    local message="$2"

    case "$level" in
        "DEBUG") [[ "${CONFIG_LOG_LEVEL:-info}" =~ ^(debug|trace)$ ]] && echo "[$level] $message" >&2 ;;
        "INFO") [[ "${CONFIG_LOG_LEVEL:-info}" =~ ^(debug|info|trace)$ ]] && echo "[$level] $message" >&2 ;;
        "WARN") [[ "${CONFIG_LOG_LEVEL:-info}" =~ ^(debug|info|warn|trace)$ ]] && echo "[$level] $message" >&2 ;;
        "ERROR") echo "[$level] $message" >&2 ;;
    esac
}

# Safe logging that doesn't depend on BASH_SOURCE
safe_log() {
    local level="$1"
    local message="$2"

    case "$level" in
        "DEBUG") [[ "${KC_LOG_LEVEL:-info}" =~ ^(debug|trace)$ ]] && echo "[$level] $message" >&2 ;;
        "INFO") [[ "${KC_LOG_LEVEL:-info}" =~ ^(debug|info|trace)$ ]] && echo "[$level] $message" >&2 ;;
        "WARN") [[ "${KC_LOG_LEVEL:-info}" =~ ^(debug|info|warn|trace)$ ]] && echo "[$level] $message" >&2 ;;
        "ERROR") echo "[$level] $message" >&2 ;;
    esac
}

# Detect current environment
detect_environment() {
    if [[ -n "${KC_ENV:-}" ]]; then
        echo "$KC_ENV"
        return 0
    fi

    # Check for common environment indicators
    if [[ -n "${CI:-}" ]] || [[ -n "${GITHUB_ACTIONS:-}" ]]; then
        echo "ci"
    elif [[ -d ".git" ]] && command -v git >/dev/null 2>&1 && git branch --show-current >/dev/null 2>&1; then
        local branch
        branch=$(git branch --show-current 2>/dev/null)
        case "$branch" in
            "main"|"master") echo "production" ;;
            "develop"|"development") echo "development" ;;
            "staging"|"stage") echo "staging" ;;
            *) echo "development" ;;
        esac
    else
        echo "development"
    fi
}

# Load configuration with environment-specific overrides
load_config() {
    local config_path="$1"
    local environment="${2:-$(detect_environment)}"

    safe_log "INFO" "Loading configuration from $config_path (environment: $environment)"

    # Check if file exists and is readable
    if [[ ! -f "$config_path" ]]; then
        safe_log "ERROR" "Configuration file not found: $config_path"
        return 1
    fi

    if [[ ! -r "$config_path" ]]; then
        safe_log "ERROR" "Configuration file not readable: $config_path"
        return 1
    fi

    # Basic YAML parsing (simplified for 80/20)
    # In production, this would use yq or a proper YAML parser
    local in_global=false
    local in_env=false
    local current_section=""

    while IFS= read -r line; do
        # Skip comments and empty lines
        [[ "$line" =~ ^[[:space:]]*# ]] && continue
        [[ -z "$line" ]] && continue

        # Handle section headers
        if [[ "$line" =~ ^([a-zA-Z_][a-zA-Z0-9_]*):[[:space:]]*$ ]]; then
            current_section="${BASH_REMATCH[1]:-}"
            if [[ "$current_section" == "global" ]]; then
                in_global=true
                in_env=false
            elif [[ "$current_section" == "$environment" ]]; then
                in_global=false
                in_env=true
            else
                in_global=false
                in_env=false
            fi
            continue
        fi

        # Parse key-value pairs
        if [[ "$line" =~ ^[[:space:]]*([a-zA-Z_][a-zA-Z0-9_]*):[[:space:]]*(.*)$ ]]; then
            local key="${BASH_REMATCH[1]:-}"
            local value="${BASH_REMATCH[2]:-}"

            # Remove quotes if present
            value="${value%\"}"
            value="${value#\"}"
            value="${value%\'}"
            value="${value#\'}"

            # Set configuration value
            if [[ "$in_global" == "true" ]] || [[ "$in_env" == "true" ]]; then
                CONFIG_CACHE["$current_section.$key"]="$value"
                safe_log "DEBUG" "Set $current_section.$key = $value"
            fi
        fi
    done < "$config_path"

    safe_log "INFO" "Configuration loaded successfully"
    return 0
}

# Get configuration value with environment-specific overrides
get_config() {
    local section="$1"
    local key="$2"
    local default_value="${3:-}"

    # Check environment-specific value first
    local env_value="${CONFIG_CACHE[$section.$key]:-}"
    if [[ -n "$env_value" ]]; then
        echo "$env_value"
        return 0
    fi

    # Check global value
    local global_value="${CONFIG_CACHE[global.$key]:-}"
    if [[ -n "$global_value" ]]; then
        echo "$global_value"
        return 0
    fi

    # Return default
    echo "$default_value"
    return 0
}

# Validate configuration completeness
validate_config() {
    local required_keys=(
        "global.log_level"
        "scan_fakes.enabled"
        "ci_gate.enabled"
        "smoke_tests.enabled"
    )

    local missing_keys=()

    for key in "${required_keys[@]}"; do
        if [[ -z "$(get_config "${key%.*}" "${key#*.}")" ]]; then
            missing_keys+=("$key")
        fi
    done

    if [[ ${#missing_keys[@]} -gt 0 ]]; then
        config_log "ERROR" "Missing required configuration keys: ${missing_keys[*]}"
        return 1
    fi

    config_log "INFO" "Configuration validation passed"
    return 0
}

# Initialize configuration system
init_config() {
    local environment="${KC_ENV:-$(detect_environment)}"

    safe_log "INFO" "Initializing KCura configuration system (environment: $environment)"

    # Load main configuration
    if ! load_config "$SCRIPT_DIR/$CONFIG_FILE" "$environment"; then
        safe_log "ERROR" "Failed to load main configuration"
        return 1
    fi

    # Validate configuration
    if ! validate_config; then
        safe_log "ERROR" "Configuration validation failed"
        return 1
    fi

    safe_log "INFO" "Configuration system initialized successfully"
    return 0
}

# Get script-specific configuration
get_script_config() {
    local script_name="$1"
    local key="$2"
    local default_value="$3"

    echo "$(get_config "$script_name" "$key" "$default_value")"
}

# Check if a feature is enabled
is_feature_enabled() {
    local script_name="$1"
    local feature="${2:-enabled}"

    local value
    value="$(get_script_config "$script_name" "$feature" "false")"
    [[ "$value" =~ ^(true|1|yes|on)$ ]] && return 0 || return 1
}

# Get timeout value with fallback
get_timeout() {
    local script_name="$1"
    local operation="${2:-default}"

    local timeout
    timeout="$(get_script_config "$script_name" "timeout" "$(get_config "global" "timeout_default" "60")")"

    # Operation-specific timeout
    case "$operation" in
        "pattern") timeout="$(get_script_config "$script_name" "timeout" "$timeout")" ;;
        "test") timeout="$(get_script_config "smoke_tests" "timeout_per_test" "$timeout")" ;;
        "check") timeout="$(get_script_config "ci_gate" "timeout" "$timeout")" ;;
    esac

    echo "$timeout"
}

# Export configuration for use in other scripts
export_config() {
    # Export key configuration values as environment variables
    export KC_LOG_LEVEL="$(get_config "global" "log_level" "info")"
    export KC_LOG_FORMAT="$(get_config "global" "log_format" "json")"
    export KC_ENABLE_METRICS="$(get_config "global" "enable_metrics" "true")"
    export KC_ENABLE_AUDIT="$(get_config "global" "enable_audit" "true")"
    export KC_TIMEOUT_DEFAULT="$(get_config "global" "timeout_default" "60")"
    export KC_MAX_RETRIES="$(get_config "global" "max_retries" "3")"
    export KC_RETRY_DELAY="$(get_config "global" "retry_delay" "2")"
}

# Initialize if this script is run directly
if [[ "${BASH_SOURCE[0]:-}" == "${0:-}" ]] || [[ -z "${BASH_SOURCE[0]:-}" && -n "${0:-}" ]]; then
    echo "KCura Configuration Library v1.0"
    echo "Usage: source this file in your scripts"
    echo ""
    echo "Functions available:"
    echo "  init_config              - Initialize configuration system"
    echo "  get_config <section> <key> [default] - Get configuration value"
    echo "  get_script_config <script> <key> [default] - Get script-specific config"
    echo "  is_feature_enabled <script> [feature] - Check if feature is enabled"
    echo "  get_timeout <script> [operation] - Get timeout value"
    echo "  export_config            - Export configuration as environment variables"
    echo ""
    echo "Example:"
    echo "  source lib/config.sh"
    echo "  init_config"
    echo "  echo \"Log level: \$(get_config global log_level)\""
fi
