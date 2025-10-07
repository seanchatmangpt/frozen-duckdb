# LLM Setup Guide

## Overview

This guide shows how to set up **LLM-in-database capabilities** using the **Flock extension** with **Ollama** to enable **text completion**, **embedding generation**, **semantic search**, and **intelligent filtering** directly within DuckDB.

## Prerequisites

### 1. Install Ollama

Ollama is required to run the LLM models locally.

#### macOS Installation

```bash
# Install via Homebrew
brew install ollama

# Or download from official site
curl -fsSL https://ollama.ai/install.sh | sh
```

#### Linux Installation

```bash
# Install via official script
curl -fsSL https://ollama.ai/install.sh | sh
```

#### Windows Installation

Download and install from [ollama.ai](https://ollama.ai/download).

### 2. Start Ollama Server

```bash
# Start the Ollama server
ollama serve

# Verify server is running
curl -s http://localhost:11434/api/version
# Expected: {"version":"0.12.3"}
```

### 3. Pull Required Models

```bash
# Pull text generation model (30B parameters)
ollama pull qwen3-coder:30b

# Pull embedding model (8B parameters)
ollama pull qwen3-embedding:8b

# Verify models are available
curl -s http://localhost:11434/api/tags | grep -E "(qwen3-coder|qwen3-embedding)"
```

**Model Specifications:**
- **qwen3-coder:30b**: 30.5B parameters for text generation and code completion
- **qwen3-embedding:8b**: 7.6B parameters for generating 1024-dimensional embeddings

## Frozen DuckDB Setup

### 1. Configure Environment

```bash
# Set up frozen DuckDB (if not already done)
source ../frozen-duckdb/prebuilt/setup_env.sh
```

### 2. Install Flock Extension

```sql
-- Connect to DuckDB
duckdb

-- Install and load Flock extension
INSTALL flock FROM community;
LOAD flock;

-- Verify installation
SELECT extension_name FROM duckdb_extensions() WHERE extension_name = 'flock';
```

### 3. Configure Ollama Secret

```sql
-- Create secret for Ollama connection
CREATE SECRET ollama_secret (TYPE OLLAMA, API_URL 'http://localhost:11434');

-- Alternative syntax (without name)
CREATE SECRET (TYPE OLLAMA, API_URL 'http://localhost:11434');
```

### 4. Create Models

```sql
-- Text generation model
CREATE MODEL('coder', 'qwen3-coder:30b', 'ollama');

-- Embedding model
CREATE MODEL('embedder', 'qwen3-embedding:8b', 'ollama');

-- Verify models are created
GET MODELS;
```

### 5. Create Prompts

```sql
-- Basic completion prompt
CREATE PROMPT('complete', 'Complete this text: {{text}}');

-- Question answering prompt
CREATE PROMPT('qa', 'Answer this question: {{text}}');

-- Filter prompt
CREATE PROMPT('filter', 'Answer yes or no: {{text}}');

-- Verify prompts are created
GET PROMPTS;
```

## CLI Setup (Recommended)

### Automated Setup

```bash
# Use the CLI for automated setup
frozen-duckdb flock-setup

# Setup with custom Ollama URL
frozen-duckdb flock-setup --ollama-url http://192.168.1.100:11434

# Setup without verification (faster)
frozen-duckdb flock-setup --skip-verification
```

### Manual Verification

```bash
# Test basic LLM functionality
frozen-duckdb complete --prompt "Hello, how are you?"

# Generate embeddings
frozen-duckdb embed --text "machine learning"

# Verify system info includes Flock
frozen-duckdb info
```

## Testing the Setup

### 1. Basic Text Completion

```sql
-- Test text completion
SELECT llm_complete(
    {'model_name': 'coder'},
    {'prompt_name': 'complete', 'context_columns': [{'data': 'The Rust programming language'}]}
) as result;
```

Expected output:
```
The Rust programming language is a systems programming language that runs blazingly fast, prevents segfaults, and guarantees thread safety.
```

### 2. Embedding Generation

```sql
-- Test embedding generation
SELECT llm_embedding(
    {'model_name': 'embedder'},
    [{'data': 'artificial intelligence'}]
) as embedding;
```

Expected output:
```
[0.123456, 0.789012, ...]  # 1024-dimensional vector
```

### 3. Semantic Search

```sql
-- Create test corpus
CREATE TABLE documents AS
SELECT 'Machine learning is a subset of AI' as content
UNION ALL
SELECT 'Deep learning uses neural networks'
UNION ALL
SELECT 'Natural language processing is NLP';

-- Generate embeddings for corpus
CREATE TABLE document_embeddings AS
SELECT
    content,
    llm_embedding({'model_name': 'embedder'}, [{'data': content}]) as embedding
FROM documents;

-- Search for similar content
SELECT content, similarity
FROM (
    SELECT
        content,
        array_distance(
            embedding,
            (SELECT embedding FROM document_embeddings WHERE content = 'Machine learning is a subset of AI')
        ) as similarity
    FROM document_embeddings
) t
ORDER BY similarity
LIMIT 3;
```

## Troubleshooting Setup Issues

### Common Setup Problems

#### 1. Ollama Server Not Running

**Error:** `Failed to connect to Ollama`

**Solution:**
```bash
# Check if Ollama is running
curl -s http://localhost:11434/api/version

# Start Ollama server
ollama serve

# Run in background
ollama serve &
```

#### 2. Models Not Available

**Error:** `Model not found`

**Solution:**
```bash
# Check available models
ollama list

# Pull missing models
ollama pull qwen3-coder:30b
ollama pull qwen3-embedding:8b

# Verify models are loaded
curl -s http://localhost:11434/api/tags
```

#### 3. Flock Extension Issues

**Error:** `Flock extension not available`

**Solution:**
```sql
-- Check extension status
SELECT extension_name, loaded FROM duckdb_extensions();

-- Try reinstalling
INSTALL flock FROM community;
LOAD flock;

-- Check for version conflicts
SELECT version FROM duckdb_extensions() WHERE extension_name = 'flock';
```

#### 4. Network Connection Issues

**Error:** `Network connection failed`

**Solution:**
```bash
# Test basic connectivity
curl -s http://localhost:11434/api/version

# Check firewall settings
# Ensure no firewall blocking localhost:11434

# Try different port if needed
CREATE SECRET ollama_secret (TYPE OLLAMA, API_URL 'http://localhost:11435');
```

## Performance Optimization

### Memory Management

**System Requirements:**
- **Minimum RAM**: 8GB (4GB for models + 4GB for operations)
- **Recommended RAM**: 16GB+ (for optimal performance)
- **Storage**: 50GB+ (for model storage and caching)

**Memory Optimization:**
```bash
# Monitor memory usage
htop

# Check DuckDB memory usage
SELECT * FROM pragma_memory_usage();

# Optimize for available memory
# Use smaller models if memory constrained
```

### Network Optimization

**Local Setup (Recommended):**
- **Latency**: <1ms (localhost)
- **Bandwidth**: Unlimited (local)
- **Reliability**: High (no network dependencies)

**Remote Setup (Advanced):**
```bash
# Configure for remote Ollama server
frozen-duckdb flock-setup --ollama-url http://your-server:11434

# Or in SQL
CREATE SECRET ollama_secret (TYPE OLLAMA, API_URL 'http://your-server:11434');
```

### Batch Processing Optimization

```sql
-- Process multiple items efficiently
CREATE TABLE batch_results AS
SELECT
    item,
    llm_complete(
        {'model_name': 'coder'},
        {'prompt_name': 'complete', 'context_columns': [{'data': item}]}
    ) as completion
FROM input_items;
```

## Security Considerations

### Network Security

**Local Only (Recommended):**
- All operations happen on localhost
- No data sent over network
- Complete privacy and security

**Remote Ollama (Advanced):**
```bash
# Use HTTPS for remote connections
CREATE SECRET ollama_secret (TYPE OLLAMA, API_URL 'https://your-server:11434');

# Configure authentication if needed
# (Ollama supports basic auth)
```

### Data Privacy

- **Local processing**: All LLM operations happen locally
- **No data transmission**: Prompts and responses stay on your machine
- **Model isolation**: Each model runs in isolated environment
- **Audit trail**: All operations logged locally

## Advanced Configuration

### Custom Model Configuration

```sql
-- Use different models for different tasks
CREATE MODEL('fast_coder', 'qwen3-coder:7b', 'ollama');  -- Smaller, faster
CREATE MODEL('quality_coder', 'qwen3-coder:30b', 'ollama');  -- Larger, better quality

-- Switch between models as needed
SELECT llm_complete({'model_name': 'fast_coder'}, {...});  -- For quick responses
SELECT llm_complete({'model_name': 'quality_coder'}, {...});  -- For better quality
```

### Custom Prompt Engineering

```sql
-- Advanced prompt templates
CREATE PROMPT('code_review', 'Review this code for bugs and improvements: {{code}}');

CREATE PROMPT('explain_concept', 'Explain {{concept}} in simple terms for beginners');

CREATE PROMPT('translate_code', 'Translate {{code}} from {{from_lang}} to {{to_lang}}');

-- Usage with parameters
SELECT llm_complete(
    {'model_name': 'coder'},
    {
        'prompt_name': 'code_review',
        'context_columns': [{'code': 'fn main() { println!("Hello"); }'}]
    }
);
```

### Environment-Specific Setup

```bash
#!/bin/bash
# setup_llm_environment.sh

# Detect available resources
TOTAL_MEM=$(system_profiler SPHardwareDataType | grep "Memory:" | awk '{print $2}' | sed 's/GB//')

if (( $(echo "$TOTAL_MEM < 16" | bc -l) )); then
    echo "⚠️  Limited memory detected. Consider using smaller models."
    RECOMMENDED_MODEL="qwen3-coder:7b"
else
    echo "✅ Sufficient memory for full models."
    RECOMMENDED_MODEL="qwen3-coder:30b"
fi

# Setup with recommended model
CREATE MODEL('coder', '$RECOMMENDED_MODEL', 'ollama');
```

## Production Deployment

### Docker Deployment

```dockerfile
# Dockerfile with LLM capabilities
FROM rust:latest as builder

# Install Ollama
RUN curl -fsSL https://ollama.ai/install.sh | sh

# Pull models during build
RUN ollama pull qwen3-coder:30b
RUN ollama pull qwen3-embedding:8b

# Setup frozen DuckDB
COPY frozen-duckdb /frozen-duckdb
RUN cd /frozen-duckdb && source prebuilt/setup_env.sh

# Build application
WORKDIR /app
COPY . .
ENV DUCKDB_LIB_DIR="/frozen-duckdb/prebuilt"
ENV DUCKDB_INCLUDE_DIR="/frozen-duckdb/prebuilt"
RUN cargo build --release

# Runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y ca-certificates

# Copy Ollama binary and models
COPY --from=builder /usr/local/bin/ollama /usr/local/bin/
COPY --from=builder /root/.ollama /root/.ollama

# Copy application
COPY --from=builder /app/target/release/app /usr/local/bin/

# Start services
CMD ollama serve & sleep 5 && app
```

### Kubernetes Deployment

```yaml
# ollama-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: ollama
spec:
  replicas: 1
  selector:
    matchLabels:
      app: ollama
  template:
    metadata:
      labels:
        app: ollama
    spec:
      containers:
      - name: ollama
        image: ollama/ollama:latest
        ports:
        - containerPort: 11434
        env:
        - name: OLLAMA_HOST
          value: "0.0.0.0:11434"

---
# application-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: frozen-duckdb-app
spec:
  replicas: 3
  selector:
    matchLabels:
      app: frozen-duckdb
  template:
    metadata:
      labels:
        app: frozen-duckdb
    spec:
      containers:
      - name: app
        image: your-app:latest
        env:
        - name: DUCKDB_LIB_DIR
          value: "/frozen-duckdb/prebuilt"
        - name: DUCKDB_INCLUDE_DIR
          value: "/frozen-duckdb/prebuilt"
        - name: OLLAMA_URL
          value: "http://ollama-service:11434"
```

## Monitoring and Maintenance

### Health Checks

```bash
#!/bin/bash
# llm_health_check.sh

# Check Ollama status
if curl -s http://localhost:11434/api/version > /dev/null; then
    echo "✅ Ollama server running"
else
    echo "❌ Ollama server not responding"
    exit 1
fi

# Check model availability
MODELS=$(curl -s http://localhost:11434/api/tags | jq -r '.models[].name')
if echo "$MODELS" | grep -q "qwen3-coder"; then
    echo "✅ Required models available"
else
    echo "❌ Required models missing"
    exit 1
fi

# Test LLM functionality
if frozen-duckdb complete --prompt "test" > /dev/null 2>&1; then
    echo "✅ LLM functionality working"
else
    echo "❌ LLM functionality failed"
    exit 1
fi

echo "🎉 All LLM systems operational"
```

### Performance Monitoring

```sql
-- Monitor LLM operation performance
CREATE TABLE llm_metrics AS
SELECT
    'completion' as operation,
    COUNT(*) as total_operations,
    AVG(execution_time_ms) as avg_time_ms,
    MIN(execution_time_ms) as min_time_ms,
    MAX(execution_time_ms) as max_time_ms
FROM (
    -- This would require custom performance tracking
    SELECT 1 as execution_time_ms
) metrics;
```

### Log Analysis

```bash
# Monitor Ollama logs
tail -f ~/.ollama/logs/server.log

# Monitor application logs
journalctl -u your-app -f

# Check system resources during LLM operations
top -p $(pgrep -f ollama) -p $(pgrep -f your-app)
```

## Troubleshooting Common Issues

### 1. Model Loading Issues

**Problem:** Models fail to load or respond slowly

**Solutions:**
```bash
# Check model status
ollama list

# Restart Ollama server
ollama stop
ollama serve

# Clear model cache if corrupted
rm -rf ~/.ollama/models
ollama pull qwen3-coder:30b
```

### 2. Memory Issues

**Problem:** Out of memory errors during LLM operations

**Solutions:**
```bash
# Monitor memory usage
free -h

# Use smaller models for memory-constrained systems
CREATE MODEL('coder', 'qwen3-coder:7b', 'ollama');

# Restart Ollama with memory limits
OLLAMA_MAX_QUEUE=10 ollama serve
```

### 3. Network Issues

**Problem:** Connection timeouts or network errors

**Solutions:**
```bash
# Test basic connectivity
curl -v http://localhost:11434/api/version

# Check firewall settings
sudo ufw status

# Try different port
CREATE SECRET ollama_secret (TYPE OLLAMA, API_URL 'http://localhost:11435');
```

### 4. Permission Issues

**Problem:** Access denied errors

**Solutions:**
```bash
# Check Ollama permissions
ls -la /usr/local/bin/ollama

# Fix permissions if needed
sudo chmod +x /usr/local/bin/ollama

# Check model directory permissions
ls -la ~/.ollama/
```

## Best Practices

### 1. Resource Management

- **Monitor memory usage** during LLM operations
- **Use appropriate model sizes** for your hardware
- **Implement batch processing** for multiple operations
- **Clean up temporary data** after operations

### 2. Performance Optimization

- **Use local Ollama** for best performance and privacy
- **Cache frequently used embeddings** to avoid recomputation
- **Batch similar operations** together for efficiency
- **Monitor operation times** and optimize bottlenecks

### 3. Security

- **Keep Ollama updated** for security patches
- **Use local models only** unless remote access is required
- **Monitor access logs** for unauthorized usage
- **Secure model files** with appropriate permissions

### 4. Maintenance

- **Regularly update models** for improved performance
- **Monitor system resources** during LLM operations
- **Backup model files** for quick recovery
- **Document custom configurations** for team members

## Summary

Setting up LLM capabilities with Frozen DuckDB and Ollama provides **powerful AI integration** directly within your database operations. The setup process is **straightforward** but requires **careful configuration** of models, prompts, and system resources.

**Key Setup Steps:**
1. **Install Ollama** and pull required models
2. **Configure Flock extension** in DuckDB
3. **Create models and prompts** for your use cases
4. **Test functionality** with sample operations
5. **Optimize performance** for your specific needs

**Benefits Achieved:**
- **SQL-native LLM operations** (no external API calls needed)
- **High-performance local inference** (no network latency)
- **Complete data privacy** (all processing local)
- **Seamless database integration** (unified data + AI workflows)

**Next Steps:**
1. Complete the [Integration Guide](./integration.md) for project setup
2. Explore the [LLM Operations Guide](./llm-operations.md) for usage examples
3. Set up [Performance Monitoring](./performance-tuning.md) for optimization
4. Consider [RAG Pipelines](./rag-pipelines.md) for advanced use cases
