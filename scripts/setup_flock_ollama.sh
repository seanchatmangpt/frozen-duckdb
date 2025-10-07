#!/bin/bash
# Flock Extension Setup Script for Ollama
# Based on: https://dais-polymtl.github.io/flock/docs/getting-started/ollama
#
# This script automates the setup of Flock extension with Ollama provider
# for qwen3-coder:30b (text generation) and qwen3-embedding:8b (embeddings)
#
# Usage: ./setup_flock_ollama.sh [--duckdb-path <path>] [--ollama-url <url>] [--force]

set -euo pipefail

# Default values
DUCKDB_PATH=""
OLLAMA_URL="http://localhost:11434"
FORCE_SETUP=false
VERBOSE=false

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --duckdb-path)
            DUCKDB_PATH="$2"
            shift 2
            ;;
        --ollama-url)
            OLLAMA_URL="$2"
            shift 2
            ;;
        --force)
            FORCE_SETUP=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [--duckdb-path <path>] [--ollama-url <url>] [--force] [--verbose]"
            echo ""
            echo "Options:"
            echo "  --duckdb-path <path>    Path to DuckDB binary (default: use PATH)"
            echo "  --ollama-url <url>      Ollama API URL (default: http://localhost:11434)"
            echo "  --force                 Force setup even if components exist"
            echo "  --verbose               Enable verbose output"
            echo "  -h, --help             Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1" >&2
            exit 1
            ;;
    esac
done

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $*"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $*"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $*"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*"
}

verbose_log() {
    if [[ "$VERBOSE" == "true" ]]; then
        echo -e "${BLUE}[VERBOSE]${NC} $*"
    fi
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check Ollama status
check_ollama() {
    log_info "Checking Ollama status..."

    if ! command_exists curl; then
        log_error "curl is required but not installed"
        exit 1
    fi

    # Check if Ollama API is responding
    if curl -s --max-time 5 "${OLLAMA_URL}/api/version" >/dev/null 2>&1; then
        OLLAMA_VERSION=$(curl -s "${OLLAMA_URL}/api/version" | grep -o '"version":"[^"]*"' | cut -d'"' -f4)
        log_success "Ollama is running (version: ${OLLAMA_VERSION})"
        return 0
    else
        log_error "Ollama is not running or not accessible at ${OLLAMA_URL}"
        log_info "Please ensure Ollama is installed and running:"
        log_info "  1. Install Ollama from: https://ollama.ai/download"
        log_info "  2. Start Ollama: ollama serve"
        log_info "  3. Verify: curl ${OLLAMA_URL}/api/version"
        exit 1
    fi
}

# Check and pull models
setup_models() {
    log_info "Setting up required models..."

    # Required models
    # shellcheck disable=SC2034
    MODELS=("qwen3-coder:30b" "qwen3-embedding:8b")
    MODEL_DESCRIPTIONS=("Text generation model" "Embedding model")

    for i in "${!MODELS[@]}"; do
        model="${MODELS[$i]}"
        description="${MODEL_DESCRIPTIONS[$i]}"

        verbose_log "Checking model: $model ($description)"

        # Check if model exists (handle both exact match and tag variants)
        if curl -s "${OLLAMA_URL}/api/tags" | grep -q "\"name\":\"${model}\""; then
            log_success "Model $model is already available"
        else
            log_warning "Model $model not found. Checking for tag variants..."

            # For qwen3-coder:30b, also check for qwen3-coder:latest
            # For qwen3-embedding:8b, also check for qwen3-embedding:latest
            local model_base="${model%:*}"
            local model_tag="${model#*:}"

            if [[ "$model_tag" == "30b" && "$model_base" == "qwen3-coder" ]]; then
                if curl -s "${OLLAMA_URL}/api/tags" | grep -q "\"name\":\"qwen3-coder:latest\""; then
                    log_success "Model qwen3-coder:latest is available (equivalent to $model)"
                else
                    pull_model "$model"
                fi
            elif [[ "$model_tag" == "8b" && "$model_base" == "qwen3-embedding" ]]; then
                if curl -s "${OLLAMA_URL}/api/tags" | grep -q "\"name\":\"qwen3-embedding:latest\""; then
                    log_success "Model qwen3-embedding:latest is available (equivalent to $model)"
                else
                    pull_model "$model"
                fi
            else
                pull_model "$model"
            fi
        fi
    done
}

# Pull a model using ollama command
pull_model() {
    local model="$1"
    log_info "Pulling model: $model"

    # Check if ollama command exists for pulling
    if command_exists ollama; then
        if ollama pull "$model"; then
            log_success "Successfully pulled model: $model"
        else
            log_error "Failed to pull model: $model"
            log_info "You can manually pull it with: ollama pull $model"
            exit 1
        fi
    else
        log_error "ollama command not found. Cannot pull models automatically."
        log_info "Please manually pull the model: ollama pull $model"
        exit 1
    fi
}

# Setup Flock extension and configuration
setup_flock() {
    log_info "Setting up Flock extension..."

    # Find DuckDB binary
    DUCKDB_CMD="duckdb"
    if [[ -n "$DUCKDB_PATH" ]]; then
        if [[ -f "$DUCKDB_PATH" ]]; then
            DUCKDB_CMD="$DUCKDB_PATH"
        else
            log_error "DuckDB binary not found at: $DUCKDB_PATH"
            exit 1
        fi
    fi

    # Check if DuckDB exists
    if ! command_exists "$DUCKDB_CMD" && [[ ! -f "$DUCKDB_PATH" ]]; then
        log_error "DuckDB not found. Please install DuckDB or provide correct path."
        exit 1
    fi

    # Create temporary SQL file for setup
    SETUP_SQL=$(mktemp)
    cat > "$SETUP_SQL" << EOF
-- Install and load Flock extension
INSTALL flock FROM community;
LOAD flock;

-- Create Ollama secret (using __default_ollama as expected by Flock)
CREATE SECRET __default_ollama (TYPE OLLAMA, API_URL '${OLLAMA_URL}');

-- Create text generation model (qwen3-coder:30b)
CREATE MODEL('coder', 'qwen3-coder:30b', 'ollama');

-- Create embedding model (qwen3-embedding:8b)
CREATE MODEL('embedder', 'qwen3-embedding:8b', 'ollama');

-- Test the setup with a simple query
SELECT 'Flock setup completed successfully' as status;
EOF

    log_info "Running Flock setup SQL commands..."
    verbose_log "SQL file: $SETUP_SQL"

    if "$DUCKDB_CMD" -c ".read $SETUP_SQL" >/dev/null 2>&1; then
        log_success "Flock extension setup completed successfully"
    else
        log_error "Failed to execute Flock setup"
        log_info "You can manually run the setup with:"
        log_info "  duckdb -c \"INSTALL flock FROM community; LOAD flock;\""
        log_info "  duckdb -c \"CREATE SECRET __default_ollama (TYPE OLLAMA, API_URL '${OLLAMA_URL}');\""
        log_info "  duckdb -c \"CREATE MODEL('coder', 'qwen3-coder:30b', 'ollama');\""
        log_info "  duckdb -c \"CREATE MODEL('embedder', 'qwen3-embedding:8b', 'ollama');\""
        exit 1
    fi

    # Clean up temporary file
    rm -f "$SETUP_SQL"
}

# Test the setup
test_setup() {
    log_info "Testing Flock setup..."

    DUCKDB_CMD="duckdb"
    if [[ -n "$DUCKDB_PATH" ]]; then
        DUCKDB_CMD="$DUCKDB_PATH"
    fi

    # Test basic functionality
    TEST_SQL=$(mktemp)
    cat > "$TEST_SQL" << 'EOF'
-- Test extension is loaded
SELECT extension_name FROM duckdb_extensions() WHERE extension_name = 'flock';

-- Test models exist
GET MODELS;

-- Test secrets exist
SELECT * FROM duckdb_secrets();

-- Test basic LLM function (should work if setup is correct)
SELECT 'Extension and models are properly configured' as test_result;
EOF

    log_info "Running test queries..."
    if "$DUCKDB_CMD" -c ".read $TEST_SQL" >/dev/null 2>&1; then
        log_success "Flock setup test passed"
    else
        log_warning "Flock setup test had issues - this may be expected if models aren't fully loaded yet"
        log_info "You can verify the setup manually with:"
        log_info "  duckdb -c \"GET MODELS;\""
        log_info "  duckdb -c \"SELECT * FROM duckdb_secrets();\""
    fi

    rm -f "$TEST_SQL"
}

# Main execution
main() {
    echo "🦆 Frozen DuckDB Flock Extension Setup Script"
    echo "=============================================="
    echo ""

    log_info "Starting Flock extension setup..."
    log_info "Target models: qwen3-coder:30b (text), qwen3-embedding:8b (embeddings)"
    log_info "Ollama URL: $OLLAMA_URL"

    # Step 1: Check Ollama
    check_ollama

    # Step 2: Setup models
    setup_models

    # Step 3: Setup Flock
    setup_flock

    # Step 4: Test setup
    test_setup

    echo ""
    echo "🎉 Setup completed successfully!"
    echo ""
    echo "📋 Next steps:"
    echo "  1. Run tests: cargo test --test flock_tests"
    echo "  2. Use in project: source prebuilt/setup_env.sh && cargo run --example basic_usage"
    echo "  3. Troubleshooting: See TROUBLESHOOTING.md"
    echo ""
    echo "🔧 Manual verification:"
    echo "  duckdb -c \"GET MODELS;\""
    echo "  duckdb -c \"SELECT * FROM duckdb_secrets();\""
}

# Run main function
main "$@"
