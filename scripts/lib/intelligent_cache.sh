#!/usr/bin/env bash
# KCura Intelligent Caching System
# Adaptive caching that learns from usage patterns and optimizes performance
# Version: 1.0 - 80/20 innovation for smart caching

set -euo pipefail

# Cache configuration (initialize only if not already set)
if [[ -z "${CACHE_DIR+x}" ]]; then
    local cache_dir=""
    cache_dir="$(get_script_config "temp" "cache_dir" "$(get_config "global" "cache_dir" "/tmp")")"
    readonly CACHE_DIR="$cache_dir/kcura-cache"
fi
readonly CACHE_INDEX="${CACHE_DIR}/cache_index.json"
readonly CACHE_STATS="${CACHE_DIR}/cache_stats.json"
readonly DEFAULT_CACHE_TTL="${KC_CACHE_TTL:-300}"  # 5 minutes

# Cache data structures (initialize only if not already set)
if [[ -z "${CACHE_INDEX_CACHE+a}" ]]; then
    declare -A CACHE_INDEX_CACHE=()
fi
if [[ -z "${CACHE_STATS_CACHE+a}" ]]; then
    declare -A CACHE_STATS_CACHE=()
fi

# Initialize intelligent caching
init_intelligent_cache() {
    mkdir -p "$CACHE_DIR"

    # Load cache index if available
    if [[ -f "$CACHE_INDEX" ]]; then
        while IFS='=' read -r key value; do
            CACHE_INDEX_CACHE["$key"]="$value"
        done < <(jq -r 'to_entries[] | "\(.key)=\(.value)"' "$CACHE_INDEX" 2>/dev/null || true)
    fi

    # Load cache statistics if available
    if [[ -f "$CACHE_STATS" ]]; then
        while IFS='=' read -r key value; do
            CACHE_STATS_CACHE["$key"]="$value"
        done < <(jq -r 'to_entries[] | "\(.key)=\(.value)"' "$CACHE_STATS" 2>/dev/null || true)
    fi

    debug "Intelligent cache system initialized"
}

# Intelligent cache get with adaptive TTL
cache_get() {
    local cache_key="$1"

    # Check if item exists in cache
    local cache_entry="${CACHE_INDEX_CACHE[$cache_key]:-}"
    if [[ -z "$cache_entry" ]]; then
        debug "Cache miss: $cache_key"
        record_cache_stat "misses" 1
        return 1
    fi

    # Parse cache entry (format: "file_path:timestamp:ttl:hit_count")
    IFS=':' read -r cache_file cache_timestamp cache_ttl cache_hits <<< "$cache_entry"

    # Check if cache entry is still valid
    local current_time=$(date '+%s')
    local age=$((current_time - cache_timestamp))

    # Adaptive TTL based on usage patterns
    local adaptive_ttl=$(get_adaptive_ttl "$cache_key" "$cache_ttl")

    if [[ $age -gt $adaptive_ttl ]]; then
        debug "Cache expired: $cache_key (age: ${age}s, ttl: ${adaptive_ttl}s)"
        cache_delete "$cache_key"
        record_cache_stat "expirations" 1
        return 1
    fi

    # Read cached content
    if [[ -f "$cache_file" ]]; then
        cat "$cache_file"
        record_cache_stat "hits" 1

        # Update hit count and access time
        local new_hits=$((cache_hits + 1))
        local new_entry="$cache_file:$cache_timestamp:$adaptive_ttl:$new_hits"
        CACHE_INDEX_CACHE[$cache_key]="$new_entry"

        debug "Cache hit: $cache_key (hits: $new_hits, age: ${age}s)"
        return 0
    else
        # Cache file missing, remove from index
        unset "CACHE_INDEX_CACHE[$cache_key]"
        debug "Cache file missing, removed from index: $cache_key"
        record_cache_stat "corruptions" 1
        return 1
    fi
}

# Intelligent cache set with adaptive TTL calculation
cache_set() {
    local cache_key="$1"
    local content="$2"
    local custom_ttl="${3:-}"

    # Calculate adaptive TTL based on content characteristics and usage patterns
    local adaptive_ttl=$(calculate_adaptive_ttl "$cache_key" "$content" "$custom_ttl")

    # Create unique cache file
    local cache_file="$CACHE_DIR/$(echo "$cache_key" | sha256sum | cut -d' ' -f1)"
    local timestamp=$(date '+%s')

    # Write content to cache file
    echo "$content" > "$cache_file"

    # Update cache index
    local cache_entry="$cache_file:$timestamp:$adaptive_ttl:1"
    CACHE_INDEX_CACHE[$cache_key]="$cache_entry"

    # Record cache operation
    record_cache_stat "sets" 1
    debug "Cache set: $cache_key (ttl: ${adaptive_ttl}s, size: ${#content} bytes)"

    return 0
}

# Calculate adaptive TTL based on content and usage patterns
calculate_adaptive_ttl() {
    local cache_key="$1"
    local content="$2"
    local custom_ttl="$3"

    # Use custom TTL if provided
    if [[ -n "$custom_ttl" ]]; then
        echo "$custom_ttl"
        return 0
    fi

    # Get historical performance for this cache key
    local key_stats="${CACHE_STATS_CACHE[stats_${cache_key}]:-}"
    local base_ttl="$DEFAULT_CACHE_TTL"

    # Adjust TTL based on content size (larger content = longer TTL)
    local content_size=${#content}
    if [[ $content_size -gt 10000 ]]; then
        base_ttl=$((base_ttl * 150 / 100))  # 50% longer for large content
    elif [[ $content_size -lt 1000 ]]; then
        base_ttl=$((base_ttl * 80 / 100))   # 20% shorter for small content
    fi

    # Adjust TTL based on historical hit rate
    if [[ -n "$key_stats" ]]; then
        # Parse stats (format: "hits:misses:last_access")
        IFS=':' read -r hits misses last_access <<< "$key_stats"

        if [[ $hits -gt 0 ]]; then
            local hit_rate=$((hits * 100 / (hits + misses)))

            if [[ $hit_rate -gt 80 ]]; then
                base_ttl=$((base_ttl * 120 / 100))  # 20% longer for high hit rate
            elif [[ $hit_rate -lt 20 ]]; then
                base_ttl=$((base_ttl * 80 / 100))   # 20% shorter for low hit rate
            fi
        fi
    fi

    # Environment-based adjustments
    case "$(detect_environment)" in
        "production")
            # More conservative TTL in production
            base_ttl=$((base_ttl * 90 / 100))
            ;;
        "development")
            # Shorter TTL in development for faster feedback
            base_ttl=$((base_ttl * 70 / 100))
            ;;
    esac

    # Ensure minimum and maximum bounds
    local final_ttl=$base_ttl
    [[ $final_ttl -lt 60 ]] && final_ttl=60      # Minimum 1 minute
    [[ $final_ttl -gt 3600 ]] && final_ttl=3600  # Maximum 1 hour

    echo "$final_ttl"
}

# Get adaptive TTL for existing cache entry
get_adaptive_ttl() {
    local cache_key="$1"
    local current_ttl="$2"

    # Use same logic as calculate_adaptive_ttl but for existing entries
    local adaptive_ttl=$(calculate_adaptive_ttl "$cache_key" "" "$current_ttl")

    # Apply decay factor for old cache entries
    local age_factor=100
    if [[ -n "${CACHE_INDEX_CACHE[$cache_key]}" ]]; then
        IFS=':' read -r _ cache_timestamp _ cache_hits <<< "${CACHE_INDEX_CACHE[$cache_key]}"
        local age=$(( $(date '+%s') - cache_timestamp ))

        # Reduce TTL for very old entries
        if [[ $age -gt 1800 ]]; then  # Older than 30 minutes
            age_factor=80  # 20% reduction
        elif [[ $age -gt 900 ]]; then  # Older than 15 minutes
            age_factor=90  # 10% reduction
        fi
    fi

    echo $((adaptive_ttl * age_factor / 100))
}

# Delete cache entry
cache_delete() {
    local cache_key="$1"

    # Get cache entry
    local cache_entry="${CACHE_INDEX_CACHE[$cache_key]:-}"
    if [[ -z "$cache_entry" ]]; then
        return 0
    fi

    IFS=':' read -r cache_file _ _ _ <<< "$cache_entry"

    # Remove cache file
    [[ -f "$cache_file" ]] && rm -f "$cache_file"

    # Remove from index
    unset "CACHE_INDEX_CACHE[$cache_key]"

    record_cache_stat "deletions" 1
    debug "Cache deleted: $cache_key"
}

# Record cache statistics
record_cache_stat() {
    local stat_type="$1"
    local value="${2:-1}"

    CACHE_STATS_CACHE["$stat_type"]=$((CACHE_STATS_CACHE["$stat_type"]:-0 + value))

    # Persist stats periodically
    if [[ $((CACHE_STATS_CACHE["$stat_type"] % 10)) -eq 0 ]]; then
        save_cache_stats
    fi
}

# Save cache statistics
save_cache_stats() {
    # Save cache index
    {
        echo "{"
        local first=true
        for key in "${!CACHE_INDEX_CACHE[@]}"; do
            [[ $first == false ]] && echo ","
            echo "  \"$key\": \"${CACHE_INDEX_CACHE[$key]}\""
            first=false
        done
        echo "}"
    } > "$CACHE_INDEX"

    # Save cache statistics
    {
        echo "{"
        local first=true
        for key in "${!CACHE_STATS_CACHE[@]}"; do
            [[ $first == false ]] && echo ","
            echo "  \"$key\": ${CACHE_STATS_CACHE[$key]}"
            first=false
        done
        echo "}"
    } > "$CACHE_STATS"

    debug "Cache index and statistics saved"
}

# Get cache statistics
get_cache_stats() {
    local stat_key="$1"
    echo "${CACHE_STATS_CACHE[$stat_key]:-0}"
}

# Clean up expired cache entries
cache_cleanup() {
    local current_time=$(date '+%s')
    local cleaned_count=0

    for cache_key in "${!CACHE_INDEX_CACHE[@]}"; do
        IFS=':' read -r cache_file cache_timestamp cache_ttl _ <<< "${CACHE_INDEX_CACHE[$cache_key]}"

        local age=$((current_time - cache_timestamp))
        local adaptive_ttl=$(get_adaptive_ttl "$cache_key" "$cache_ttl")

        if [[ $age -gt $adaptive_ttl ]]; then
            cache_delete "$cache_key"
            cleaned_count=$((cleaned_count + 1))
        fi
    done

    if [[ $cleaned_count -gt 0 ]]; then
        debug "Cache cleanup completed: $cleaned_count entries removed"
        record_cache_stat "cleanup_runs" 1
    fi

    return $cleaned_count
}

# Get cache size information
get_cache_size() {
    local total_size=0
    local entry_count=0

    for cache_key in "${!CACHE_INDEX_CACHE[@]}"; do
        IFS=':' read -r cache_file _ _ _ <<< "${CACHE_INDEX_CACHE[$cache_key]}"

        if [[ -f "$cache_file" ]]; then
            local file_size=$(stat -f%z "$cache_file" 2>/dev/null || stat -c%s "$cache_file" 2>/dev/null || echo "0")
            total_size=$((total_size + file_size))
            entry_count=$((entry_count + 1))
        fi
    done

    echo "$entry_count:$total_size"
}

# Health check for intelligent cache
intelligent_cache_health_check() {
    local issues=0

    # Check if cache directory is writable
    if ! touch "$CACHE_DIR/test_write" 2>/dev/null; then
        warn "Cache directory not writable: $CACHE_DIR"
        issues=$((issues + 1))
    else
        rm -f "$CACHE_DIR/test_write"
    fi

    # Check cache size (warn if too large)
    IFS=':' read -r entry_count total_size <<< "$(get_cache_size)"
    local size_mb=$((total_size / 1024 / 1024))

    if [[ $size_mb -gt 100 ]]; then
        warn "Cache size is large: ${size_mb}MB"
        issues=$((issues + 1))
    fi

    # Check hit rate
    local hits=$(get_cache_stats "hits")
    local misses=$(get_cache_stats "misses")
    local total_requests=$((hits + misses))

    if [[ $total_requests -gt 0 ]]; then
        local hit_rate=$((hits * 100 / total_requests))
        if [[ $hit_rate -lt 30 ]]; then
            warn "Low cache hit rate: ${hit_rate}%"
            issues=$((issues + 1))
        fi
    fi

    if [[ $issues -eq 0 ]]; then
        health_check "intelligent_cache" "ok" "Cache system operational (entries: $entry_count, size: ${size_mb}MB)"
        return 0
    else
        health_check "intelligent_cache" "warning" "$issues cache issues detected"
        return 1
    fi
}

# Export functions for use in other scripts
export -f init_intelligent_cache cache_get cache_set cache_delete
export -f record_cache_stat save_cache_stats get_cache_stats
export -f cache_cleanup get_cache_size intelligent_cache_health_check

# Initialize if this script is run directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    echo "KCura Intelligent Cache System v1.0"
    echo "Usage: source this file in your scripts"
    echo ""
    echo "Functions available:"
    echo "  init_intelligent_cache - Initialize intelligent cache"
    echo "  cache_get <key> - Get cached value"
    echo "  cache_set <key> <content> [ttl] - Set cached value"
    echo "  cache_delete <key> - Delete cached value"
    echo "  cache_cleanup - Clean expired entries"
    echo "  get_cache_stats <type> - Get cache statistics"
    echo "  intelligent_cache_health_check - Check cache health"
    echo ""
    echo "Environment variables:"
    echo "  KC_CACHE_DIR - Cache directory (default: /tmp/kcura-cache)"
    echo "  KC_CACHE_TTL - Default cache TTL in seconds (default: 300)"
fi
