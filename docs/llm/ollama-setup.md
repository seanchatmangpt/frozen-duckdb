# Ollama Setup Guide

## Overview

**Ollama** is a **local LLM server** that enables running **large language models** on your own hardware. This guide covers **installation**, **configuration**, and **optimization** of Ollama for use with the **Flock extension** in Frozen DuckDB.

## System Requirements

### Hardware Requirements

| Component | Minimum | Recommended | Notes |
|-----------|---------|-------------|-------|
| **RAM** | 8GB | 16GB+ | Required for model loading |
| **Storage** | 50GB | 100GB+ | Model storage and caching |
| **CPU** | 4 cores | 8+ cores | Better for parallel operations |
| **GPU** | Optional | NVIDIA/AMD | Significant speedup for inference |

### Operating System Support

- **macOS**: 12.0+ (Intel and Apple Silicon)
- **Linux**: Ubuntu 18.04+, CentOS 7+, Arch Linux
- **Windows**: Windows 10+ (via WSL2 or native)

## Installation

### 1. Download and Install

#### macOS

```bash
# Install via Homebrew (recommended)
brew install ollama

# Or download from official site
curl -fsSL https://ollama.ai/install.sh | sh

# Verify installation
ollama --version
```

#### Linux

```bash
# Install via official script
curl -fsSL https://ollama.ai/install.sh | sh

# Or install from package manager (Ubuntu/Debian)
curl -fsSL https://ollama.ai/install.sh | sh -s -- --deb

# Verify installation
ollama --version
```

#### Windows

1. Download from [ollama.ai/download](https://ollama.ai/download)
2. Run the installer
3. Verify installation via PowerShell:
```powershell
ollama --version
```

### 2. Start Ollama Server

```bash
# Start the Ollama server
ollama serve

# Run in background
ollama serve &

# Or use systemd service (Linux)
sudo systemctl enable ollama
sudo systemctl start ollama
```

### 3. Verify Server Status

```bash
# Check if server is running
curl -s http://localhost:11434/api/version

# Expected response:
# {"version":"0.12.3"}
```

**Alternative verification:**
```bash
# Check server process
ps aux | grep ollama

# Check listening ports
netstat -tlnp | grep 11434
```

## Model Management

### Pull Required Models

```bash
# Pull text generation model (30B parameters)
ollama pull qwen3-coder:30b

# Pull embedding model (8B parameters)
ollama pull qwen3-embedding:8b

# Verify models are available
ollama list

# Expected output:
# NAME                    SIZE    MODIFIED
# qwen3-coder:30b        30.5GB  2 minutes ago
# qwen3-embedding:8b     7.6GB   1 minute ago
```

### Model Information

**qwen3-coder:30b:**
- **Size**: 30.5GB on disk
- **Parameters**: 30 billion
- **Use case**: High-quality text generation, code completion
- **Performance**: Excellent quality, moderate speed
- **Memory**: ~16GB RAM required for inference

**qwen3-embedding:8b:**
- **Size**: 7.6GB on disk
- **Parameters**: 8 billion
- **Use case**: Vector embeddings for similarity search
- **Performance**: Fast embedding generation, consistent vectors
- **Memory**: ~4GB RAM required for inference

### Alternative Models

For systems with limited resources, consider smaller models:

```bash
# Smaller text generation model (faster, lower quality)
ollama pull qwen3-coder:7b

# Alternative embedding model
ollama pull qwen3-embedding:4b
```

## Configuration

### 1. Server Configuration

**Default Configuration:**
- **Port**: 11434
- **Host**: localhost (127.0.0.1)
- **Max queue**: 512 requests
- **Max connections**: 1000

**Custom Configuration:**
```bash
# Start with custom port
OLLAMA_PORT=11435 ollama serve

# Start with custom host
OLLAMA_HOST=0.0.0.0 ollama serve

# Start with memory limits
OLLAMA_MAX_QUEUE=100 ollama serve
```

**Environment Variables:**
```bash
# Server configuration
export OLLAMA_HOST=0.0.0.0:11434
export OLLAMA_PORT=11434
export OLLAMA_MAX_QUEUE=512
export OLLAMA_MAX_CONNECTIONS=1000

# Model configuration
export OLLAMA_MODELS=/custom/model/path
export OLLAMA_KEEP_ALIVE=24h
```

### 2. Model Configuration

**Model-Specific Settings:**
```bash
# Configure model parameters
ollama create custom-coder -f ./Modelfile

# Example Modelfile for custom configuration
cat > Modelfile << 'EOF'
FROM qwen3-coder:30b
PARAMETER temperature 0.7
PARAMETER top_p 0.9
PARAMETER num_ctx 4096
EOF
```

## Integration with Flock Extension

### 1. DuckDB Configuration

```sql
-- Connect to DuckDB
duckdb

-- Install Flock extension
INSTALL flock FROM community;
LOAD flock;

-- Configure Ollama connection
CREATE SECRET ollama_secret (TYPE OLLAMA, API_URL 'http://localhost:11434');

-- Create models
CREATE MODEL('coder', 'qwen3-coder:30b', 'ollama');
CREATE MODEL('embedder', 'qwen3-embedding:8b', 'ollama');
```

### 2. CLI Integration

```bash
# Setup Ollama integration
frozen-duckdb flock-setup

# Setup with custom URL
frozen-duckdb flock-setup --ollama-url http://192.168.1.100:11434

# Verify setup
frozen-duckdb info
# Should show Flock extension available
```

### 3. Test Integration

```bash
# Test basic LLM functionality
frozen-duckdb complete --prompt "Hello, how are you?"

# Test embedding generation
frozen-duckdb embed --text "machine learning"

# Verify system information
frozen-duckdb info
```

## Performance Optimization

### 1. Memory Management

**Monitor Memory Usage:**
```bash
# Check system memory
free -h

# Monitor Ollama memory usage
htop | grep ollama

# Check model memory requirements
ollama show qwen3-coder:30b
```

**Memory Optimization:**
```bash
# Use smaller models for memory-constrained systems
ollama pull qwen3-coder:7b  # Uses less memory than 30b

# Configure memory limits
export OLLAMA_MAX_QUEUE=50  # Reduce concurrent operations

# Monitor memory during operations
watch -n 1 'free -h'
```

### 2. CPU Optimization

**Multi-threading:**
```bash
# Check CPU usage during operations
top -p $(pgrep ollama)

# Configure thread settings
export OLLAMA_NUM_THREADS=4  # Match CPU cores

# Monitor CPU usage
htop
```

**GPU Acceleration (if available):**
```bash
# Check GPU availability
nvidia-smi

# Configure GPU usage
export OLLAMA_USE_GPU=true

# Monitor GPU usage
nvidia-smi -l 1
```

### 3. Network Optimization

**Local Setup (Recommended):**
```bash
# Benefits: No network latency, complete privacy, reliable
ollama serve  # Local server

# Configure for local only
export OLLAMA_HOST=127.0.0.1:11434
```

**Remote Setup (Advanced):**
```bash
# Configure for remote server
export OLLAMA_HOST=your-server:11434

# Update Flock configuration
CREATE SECRET ollama_secret (TYPE OLLAMA, API_URL 'http://your-server:11434');
```

## Monitoring and Maintenance

### 1. Server Monitoring

**Health Checks:**
```bash
#!/bin/bash
# ollama_health_check.sh

echo "🔍 Checking Ollama server health..."

# Check server status
if curl -s http://localhost:11434/api/version > /dev/null; then
    echo "✅ Ollama server is running"
else
    echo "❌ Ollama server is not responding"
    exit 1
fi

# Check model availability
MODELS=$(curl -s http://localhost:11434/api/tags | jq -r '.models[].name')
REQUIRED_MODELS=("qwen3-coder:30b" "qwen3-embedding:8b")

for model in "${REQUIRED_MODELS[@]}"; do
    if echo "$MODELS" | grep -q "$model"; then
        echo "✅ Model $model is available"
    else
        echo "❌ Model $model is missing"
        exit 1
    fi
done

echo "🎉 Ollama server is healthy"
```

**Performance Monitoring:**
```bash
# Monitor server performance
curl -s http://localhost:11434/api/version

# Check active connections
netstat -tlnp | grep 11434

# Monitor resource usage
top -p $(pgrep ollama)
```

### 2. Log Management

**Server Logs:**
```bash
# Check server logs
tail -f ~/.ollama/logs/server.log

# Filter for errors
tail -f ~/.ollama/logs/server.log | grep -i error

# Monitor request patterns
tail -f ~/.ollama/logs/server.log | grep -E "(POST|GET|response_time)"
```

**Application Logs:**
```bash
# Monitor Frozen DuckDB operations
RUST_LOG=debug frozen-duckdb complete --prompt "test" 2>&1 | tee operation.log

# Analyze performance patterns
grep -E "(duration|time|memory)" operation.log
```

### 3. Model Maintenance

**Model Updates:**
```bash
# Check for model updates
ollama list

# Update specific model
ollama pull qwen3-coder:30b  # Will update if newer version available

# Remove unused models to free space
ollama rm old-model-name
```

**Storage Management:**
```bash
# Check model storage usage
du -sh ~/.ollama/models/

# Clean up old model versions
ollama prune  # Remove unused model layers

# Check disk usage
df -h ~/.ollama/
```

## Troubleshooting

### 1. Installation Issues

#### Permission Errors

**Problem:** Installation fails with permission errors

**Solutions:**
```bash
# Use sudo for system-wide installation
sudo curl -fsSL https://ollama.ai/install.sh | sudo sh

# Or install to user directory
curl -fsSL https://ollama.ai/install.sh | sh -s -- --user
```

#### Network Issues

**Problem:** Download fails due to network restrictions

**Solutions:**
```bash
# Use alternative installation method
wget https://ollama.ai/download/ollama-linux-amd64.tgz
tar -xzf ollama-linux-amd64.tgz
sudo mv ollama /usr/local/bin/

# Or use package manager
# Ubuntu/Debian
curl -fsSL https://ollama.ai/install.sh | sh -s -- --deb
```

### 2. Server Startup Issues

#### Port Already in Use

**Problem:** Port 11434 already occupied

**Solutions:**
```bash
# Check what's using the port
lsof -i :11434

# Kill conflicting process
sudo kill -9 $(lsof -ti :11434)

# Or use different port
OLLAMA_PORT=11435 ollama serve
```

#### Memory Issues

**Problem:** Server fails to start due to insufficient memory

**Solutions:**
```bash
# Check available memory
free -h

# Start with memory limits
OLLAMA_MAX_QUEUE=10 ollama serve

# Use smaller models
ollama pull qwen3-coder:7b  # Instead of 30b
```

### 3. Model Issues

#### Model Download Fails

**Problem:** Model pull fails or is interrupted

**Solutions:**
```bash
# Retry download
ollama pull qwen3-coder:30b

# Check disk space
df -h

# Clear partial downloads
rm -rf ~/.ollama/models/manifests/*
ollama pull qwen3-coder:30b
```

#### Model Not Found

**Problem:** Model not available after pull

**Solutions:**
```bash
# List available models
ollama list

# Check model registry
curl -s http://localhost:11434/api/tags

# Pull specific model version
ollama pull qwen3-coder:30b
```

### 4. Integration Issues

#### Flock Extension Connection

**Problem:** Flock cannot connect to Ollama

**Solutions:**
```bash
# Test basic connectivity
curl -s http://localhost:11434/api/version

# Check Flock configuration
duckdb -c "SELECT * FROM duckdb_secrets();"

# Recreate secret with correct URL
CREATE OR REPLACE SECRET ollama_secret (TYPE OLLAMA, API_URL 'http://localhost:11434');
```

#### Performance Issues

**Problem:** LLM operations are slow or timeout

**Solutions:**
```bash
# Check server load
htop | grep ollama

# Monitor memory usage
free -h

# Use smaller models for faster responses
CREATE MODEL('fast_coder', 'qwen3-coder:7b', 'ollama');

# Reduce concurrent operations
export OLLAMA_MAX_QUEUE=20
```

## Advanced Configuration

### 1. Custom Model Creation

**Create Custom Model:**
```bash
# Create custom model file
cat > CustomCoder << 'EOF'
FROM qwen3-coder:30b
PARAMETER temperature 0.8
PARAMETER top_p 0.95
PARAMETER num_ctx 8192
SYSTEM "You are a helpful coding assistant focused on Rust development."
EOF

# Create custom model
ollama create custom-rust-coder -f ./CustomCoder

# Use custom model in Flock
CREATE MODEL('rust_coder', 'custom-rust-coder', 'ollama');
```

### 2. Multi-Model Setup

**Setup Multiple Models:**
```sql
-- Multiple text generation models
CREATE MODEL('fast_coder', 'qwen3-coder:7b', 'ollama');
CREATE MODEL('quality_coder', 'qwen3-coder:30b', 'ollama');
CREATE MODEL('creative_coder', 'qwen3-coder:14b', 'ollama');

-- Multiple embedding models
CREATE MODEL('fast_embedder', 'qwen3-embedding:4b', 'ollama');
CREATE MODEL('quality_embedder', 'qwen3-embedding:8b', 'ollama');
```

**Model Selection Strategy:**
```sql
-- Choose model based on requirements
SELECT CASE
    WHEN operation = 'quick_response' THEN 'fast_coder'
    WHEN operation = 'high_quality' THEN 'quality_coder'
    WHEN operation = 'embedding' THEN 'quality_embedder'
    ELSE 'quality_coder'
END as selected_model;
```

### 3. Distributed Setup

**Remote Ollama Server:**
```bash
# On remote server
OLLAMA_HOST=0.0.0.0:11434 ollama serve

# On client machine
CREATE SECRET ollama_secret (TYPE OLLAMA, API_URL 'http://remote-server:11434');

# Test remote connection
frozen-duckdb complete --prompt "test" --model coder
```

**Load Balancing:**
```bash
# Multiple Ollama servers
CREATE SECRET ollama_primary (TYPE OLLAMA, API_URL 'http://server1:11434');
CREATE SECRET ollama_secondary (TYPE OLLAMA, API_URL 'http://server2:11434');

-- Use based on load or availability
-- Implementation would require custom logic
```

## Security Considerations

### 1. Network Security

**Local Only (Recommended):**
```bash
# Restrict to localhost only
OLLAMA_HOST=127.0.0.1:11434 ollama serve

# Firewall configuration (Linux)
sudo ufw allow from 127.0.0.1 to any port 11434
sudo ufw deny from 0.0.0.0/0 to any port 11434
```

**Remote Access (Advanced):**
```bash
# Enable remote access with authentication
OLLAMA_HOST=0.0.0.0:11434 ollama serve

# Use HTTPS for encrypted communication
# Configure reverse proxy with SSL
```

### 2. Data Privacy

**Complete Privacy:**
- All operations happen locally
- No data sent to external services
- Models run in isolated environment
- No telemetry or data collection

**Audit Trail:**
```bash
# Monitor access logs
tail -f ~/.ollama/logs/access.log

# Check for unauthorized access
grep -v "127.0.0.1" ~/.ollama/logs/access.log
```

## Performance Benchmarks

### 1. Model Performance

**Text Generation Performance:**
```bash
# Benchmark text generation
time curl -s http://localhost:11434/api/generate \
  -H "Content-Type: application/json" \
  -d '{"model": "qwen3-coder:30b", "prompt": "Write a hello world program in Rust"}' \
  | jq -r '.response' > /dev/null
```

**Embedding Performance:**
```bash
# Benchmark embedding generation
time curl -s http://localhost:11434/api/embeddings \
  -H "Content-Type: application/json" \
  -d '{"model": "qwen3-embedding:8b", "prompt": "machine learning"}' \
  | jq '.embedding | length' > /dev/null
```

### 2. System Performance

**Resource Usage Monitoring:**
```bash
#!/bin/bash
# monitor_ollama_performance.sh

echo "📊 Ollama Performance Monitor"
echo "=============================="

# CPU usage
CPU_USAGE=$(top -bn1 | grep ollama | awk '{print $9}')
echo "CPU Usage: $CPU_USAGE%"

# Memory usage
MEM_USAGE=$(ps aux | grep ollama | grep -v grep | awk '{print $4}')
echo "Memory Usage: $MEM_USAGE%"

# Disk usage
DISK_USAGE=$(du -sh ~/.ollama/ | cut -f1)
echo "Disk Usage: $DISK_USAGE"

# Network connections
CONNECTIONS=$(netstat -tlnp | grep 11434 | wc -l)
echo "Active Connections: $CONNECTIONS"

# Model status
echo "Loaded Models:"
ollama list
```

## Best Practices

### 1. Resource Management

- **Monitor memory usage** during intensive operations
- **Use appropriate model sizes** for your hardware
- **Implement connection pooling** for multiple operations
- **Clean up unused models** to free storage space

### 2. Performance Optimization

- **Use local Ollama** for best performance and privacy
- **Configure memory limits** based on available resources
- **Monitor operation times** and optimize bottlenecks
- **Use batch processing** for multiple operations

### 3. Security

- **Keep Ollama updated** for security patches
- **Restrict network access** to localhost when possible
- **Monitor access logs** for unauthorized usage
- **Use secure model files** with appropriate permissions

### 4. Maintenance

- **Regularly update models** for improved performance
- **Monitor system resources** during operations
- **Backup model configurations** for quick recovery
- **Document custom setups** for team collaboration

## Production Deployment

### 1. Docker Deployment

```dockerfile
# Dockerfile with Ollama
FROM ollama/ollama:latest

# Pull models during build
RUN ollama pull qwen3-coder:30b && \
    ollama pull qwen3-embedding:8b

# Expose port
EXPOSE 11434

# Start server
CMD ["serve"]
```

**Docker Compose:**
```yaml
# docker-compose.yml
version: '3.8'
services:
  ollama:
    image: ollama/ollama:latest
    ports:
      - "11434:11434"
    volumes:
      - ollama_data:/root/.ollama
    environment:
      - OLLAMA_HOST=0.0.0.0:11434

volumes:
  ollama_data:
```

### 2. Kubernetes Deployment

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
        volumeMounts:
        - name: ollama-storage
          mountPath: /root/.ollama
        resources:
          requests:
            memory: "8Gi"
            cpu: "2"
          limits:
            memory: "16Gi"
            cpu: "4"
      volumes:
      - name: ollama-storage
        persistentVolumeClaim:
          claimName: ollama-pvc
```

### 3. Systemd Service (Linux)

```bash
# Create systemd service file
sudo tee /etc/systemd/system/ollama.service > /dev/null << 'EOF'
[Unit]
Description=Ollama Server
After=network.target

[Service]
ExecStart=/usr/local/bin/ollama serve
Restart=always
RestartSec=10
Environment=OLLAMA_HOST=0.0.0.0:11434
Environment=OLLAMA_MAX_QUEUE=100

[Install]
WantedBy=multi-user.target
EOF

# Enable and start service
sudo systemctl enable ollama
sudo systemctl start ollama

# Check service status
sudo systemctl status ollama
```

## Summary

Ollama provides a **powerful, local LLM infrastructure** that enables **privacy-focused AI operations** with **excellent performance**. The setup process is **straightforward** and supports **multiple deployment scenarios** from **development** to **production**.

**Key Setup Steps:**
1. **Install Ollama** using the appropriate method for your platform
2. **Start the server** and verify it's running on port 11434
3. **Pull required models** (qwen3-coder:30b and qwen3-embedding:8b)
4. **Configure Flock extension** in DuckDB for seamless integration
5. **Test functionality** with sample operations

**Performance Characteristics:**
- **Model loading**: 30-60 seconds for initial load
- **Text generation**: 2-5 seconds for typical requests
- **Embedding generation**: 1-3 seconds for typical texts
- **Memory usage**: 8-16GB for full model operations
- **Network**: Local only for privacy and performance

**Deployment Options:**
- **Development**: Local installation with default settings
- **Production**: Docker containers or systemd services
- **Distributed**: Remote servers with load balancing
- **Kubernetes**: Scalable deployment with resource management

**Next Steps:**
1. Complete the [Flock Extension Overview](./flock-overview.md) for usage patterns
2. Explore the [Text Completion Guide](./text-completion.md) for detailed examples
3. Set up [Embedding Operations](./embeddings.md) for similarity search
4. Build [RAG Pipelines](./rag-pipelines.md) for advanced applications
