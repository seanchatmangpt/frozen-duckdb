# LLM Operations Guide

## Overview

LLM operations in Frozen DuckDB provide **advanced AI capabilities** directly within database queries using the **Flock extension** and **Ollama**. These operations enable **text completion**, **embedding generation**, **semantic search**, **intelligent filtering**, and **text summarization** with **local inference** for **privacy and performance**.

## Prerequisites

### 1. Ollama Setup

**Install Ollama:**
```bash
# macOS/Linux
curl -fsSL https://ollama.ai/install.sh | sh

# Verify installation
ollama --version
```

**Start Ollama Server:**
```bash
# Start server
ollama serve

# Verify server running
curl -s http://localhost:11434/api/version
# Expected: {"version":"0.12.3"}
```

**Pull Required Models:**
```bash
# Text generation model (30B parameters)
ollama pull qwen3-coder:30b

# Embedding model (8B parameters)
ollama pull qwen3-embedding:8b

# Verify models loaded
ollama list
```

### 2. Flock Extension Setup

```bash
# Setup Ollama integration
frozen-duckdb flock-setup

# Or setup with custom URL
frozen-duckdb flock-setup --ollama-url http://192.168.1.100:11434

# Verify setup
frozen-duckdb info
# Should show Flock extension available
```

## Text Completion Operations

### Basic Text Completion

**Simple completion:**
```bash
# Complete text directly
frozen-duckdb complete --prompt "Explain recursion in programming"

# Expected output:
# Recursion in programming is a technique where a function calls itself to solve a problem...
```

**File-based completion:**
```bash
# Read prompt from file
echo "Write a function to calculate fibonacci numbers" > prompt.txt
frozen-duckdb complete --input prompt.txt --output response.txt

# Interactive mode
echo "Explain quantum computing" | frozen-duckdb complete
```

**Advanced completion:**
```bash
# Use specific model
frozen-duckdb complete --prompt "Debug this Rust code" --model coder

# Multiple completions
for prompt in prompt1.txt prompt2.txt prompt3.txt; do
    frozen-duckdb complete --input "$prompt" --output "response_${prompt%.txt}.txt"
done
```

### Code Generation

**Function generation:**
```bash
frozen-duckdb complete --prompt "Write a Rust function to calculate factorial"

# Output:
# fn factorial(n: u64) -> u64 {
#     match n {
#         0 | 1 => 1,
#         _ => n * factorial(n - 1),
#     }
# }
```

**Code explanation:**
```bash
frozen-duckdb complete --prompt "Explain this code: fn main() { println!(\"Hello\"); }"

# Output:
# This is a simple Rust program that prints "Hello" to the console...
```

**Code debugging:**
```bash
cat > buggy_code.txt << 'EOF'
fn main() {
    let x: i32 = "hello";  // This will cause an error
    println!("{}", x);
}
EOF

frozen-duckdb complete --input buggy_code.txt --prompt "Fix the compilation error in this code"
```

## Embedding Operations

### Single Text Embedding

**Basic embedding:**
```bash
# Generate embedding for single text
frozen-duckdb embed --text "machine learning"

# Output (JSON):
# [
#   {
#     "text": "machine learning",
#     "embedding": [0.123, 0.456, ...],
#     "dimensions": 1024
#   }
# ]
```

**Normalized embeddings:**
```bash
# Generate normalized embeddings (better for similarity)
frozen-duckdb embed --text "artificial intelligence" --normalize

# Output includes normalized vectors for cosine similarity
```

**File-based embeddings:**
```bash
# Process multiple texts from file
cat > documents.txt << 'EOF'
Machine learning is a subset of AI
Deep learning uses neural networks
Natural language processing analyzes text
EOF

frozen-duckdb embed --input documents.txt --output embeddings.json
```

### Batch Embedding Processing

**Large document processing:**
```bash
#!/bin/bash
# process_large_documents.sh

# Split large document into chunks
split -l 100 large_document.txt chunk_

# Process each chunk
for chunk in chunk_*; do
    chunk_output="${chunk}.json"
    frozen-duckdb embed --input "$chunk" --output "$chunk_output"
    echo "Processed $chunk → $chunk_output"
done

# Combine results
cat chunk_*.json | jq -s 'add' > combined_embeddings.json
```

**Directory processing:**
```bash
# Process all text files in directory
frozen-duckdb embed --input ./documents/ --output all_embeddings.json
```

## Semantic Search Operations

### Basic Semantic Search

**Simple search:**
```bash
# Search in documents
frozen-duckdb search --query "machine learning" --corpus documents.txt

# Output:
# 🔍 Found 3 similar documents:
#   1. "Machine learning algorithms use neural networks" (similarity: 0.892)
#   2. "Deep learning is a subset of machine learning" (similarity: 0.856)
#   3. "AI systems learn from data patterns" (similarity: 0.743)
```

**Custom thresholds:**
```bash
# Search with higher similarity threshold
frozen-duckdb search --query "database optimization" --corpus papers.txt --threshold 0.8

# Limit number of results
frozen-duckdb search --query "rust programming" --corpus code.txt --limit 5
```

**JSON output for processing:**
```bash
# Get structured results
frozen-duckdb search --query "neural networks" --corpus research.txt --format json

# Output:
# [
#   {
#     "document": "Deep learning uses neural networks for pattern recognition",
#     "similarity_score": 0.892
#   }
# ]
```

### Advanced Search Patterns

**Multi-query search:**
```bash
#!/bin/bash
# search_multiple_queries.sh

QUERIES=("machine learning" "artificial intelligence" "neural networks")
CORPUS="documents.txt"

for query in "${QUERIES[@]}"; do
    echo "🔍 Searching for: $query"
    frozen-duckdb search --query "$query" --corpus "$CORPUS" --format json
done
```

**Search result processing:**
```python
import json
import subprocess

def search_and_process(query, corpus):
    # Run semantic search
    result = subprocess.run([
        "frozen-duckdb", "search",
        "--query", query,
        "--corpus", corpus,
        "--format", "json",
        "--threshold", "0.7"
    ], capture_output=True, text=True)

    # Parse results
    results = json.loads(result.stdout)

    # Process results
    for item in results:
        print(f"Found: {item['document']} (score: {item['similarity_score']:.3f".3f"
    return results

# Example usage
results = search_and_process("quantum computing", "research_papers.txt")
```

## Intelligent Filtering Operations

### Criteria-Based Filtering

**Simple filtering:**
```bash
# Filter items matching criteria
frozen-duckdb filter --criteria "Is this about technology?" --input items.txt

# Output:
# 📊 Filter results:
# ✅ MATCH: "Machine learning is transforming healthcare"
# ❌ NO MATCH: "The weather is nice today"
# ✅ MATCH: "AI algorithms improve efficiency"
```

**Custom evaluation prompts:**
```bash
# Custom filter logic
frozen-duckdb filter --prompt "Is this a programming language? Answer yes or no: {{text}}" --input languages.txt

# Complex criteria
frozen-duckdb filter --criteria "Contains 'machine learning' AND mentions 'AI'?" --input articles.txt
```

**Positive-only results:**
```bash
# Show only matching items
frozen-duckdb filter --criteria "Is this positive?" --input reviews.txt --output positive_reviews.txt
```

### Advanced Filtering

**Batch filtering:**
```bash
#!/bin/bash
# filter_by_category.sh

CATEGORIES=("technology" "science" "business" "entertainment")

for category in "${CATEGORIES[@]}"; do
    output_file="${category}_items.txt"
    frozen-duckdb filter --criteria "Is this about $category?" --input all_items.txt --output "$output_file"
    echo "Filtered $category items → $output_file"
done
```

**Filter result analysis:**
```python
import subprocess
import json

def analyze_filter_results(criteria, input_file):
    # Run filter
    result = subprocess.run([
        "frozen-duckdb", "filter",
        "--criteria", criteria,
        "--input", input_file,
        "--output", "filter_results.txt"
    ], capture_output=True, text=True)

    # Read and analyze results
    with open("filter_results.txt", "r") as f:
        lines = f.readlines()

    matches = sum(1 for line in lines if "✅ MATCH" in line)
    total = len(lines)

    print(f"Filter '{criteria}': {matches}/{total} matches ({matches/total*100".1f"}%)")

    return matches, total

# Example usage
matches, total = analyze_filter_results("Is this about technology?", "documents.txt")
```

## Text Summarization Operations

### Single Document Summarization

**Basic summarization:**
```bash
# Summarize single document
frozen-duckdb summarize --input article.txt --strategy concise

# Output:
# The research examines machine learning applications in healthcare, focusing on diagnostic accuracy improvements through neural networks. Key findings show 95% accuracy in medical image analysis, with recommendations for clinical implementation including data privacy considerations and model validation protocols.
```

**Different strategies:**
```bash
# Concise summary (default)
frozen-duckdb summarize --input research.txt --strategy concise

# Detailed summary
frozen-duckdb summarize --input paper.txt --strategy detailed --max-length 300

# Bullet-point format
frozen-duckdb summarize --input meeting_notes.txt --strategy bullet --max-length 150
```

### Multiple Document Summarization

**Directory processing:**
```bash
# Summarize all documents in directory
frozen-duckdb summarize --input ./research_papers/ --output combined_summary.txt --strategy detailed

# Process specific file types
find ./documents/ -name "*.txt" -exec frozen-duckdb summarize --input {} --output summaries/{}.summary.txt \;
```

**Batch summarization:**
```bash
#!/bin/bash
# summarize_collection.sh

DOCUMENTS_DIR="./documents"
SUMMARIES_DIR="./summaries"

# Create summaries directory
mkdir -p "$SUMMARIES_DIR"

# Summarize each document
for file in "$DOCUMENTS_DIR"/*.txt; do
    filename=$(basename "$file" .txt)
    summary_file="$SUMMARIES_DIR/${filename}_summary.txt"

    echo "Summarizing $file → $summary_file"
    frozen-duckdb summarize --input "$file" --output "$summary_file" --strategy concise
done

# Create combined summary
frozen-duckdb summarize --input "$SUMMARIES_DIR"/*_summary.txt --output "overall_summary.txt" --strategy bullet
```

## Advanced LLM Patterns

### RAG (Retrieval-Augmented Generation)

**Complete RAG pipeline:**
```bash
#!/bin/bash
# rag_pipeline.sh

# 1. Generate embeddings for knowledge base
frozen-duckdb embed --input knowledge_base.txt --output embeddings.json

# 2. Search for relevant context
CONTEXT=$(frozen-duckdb search --query "$1" --corpus knowledge_base.txt --format json | jq -r '.[0].document')

# 3. Generate answer with context
echo "Context: $CONTEXT
Question: $1
Please provide a comprehensive answer based on the context." | frozen-duckdb complete
```

**RAG with filtering:**
```python
import subprocess
import json

def rag_with_filtering(query, corpus, filter_criteria):
    # Generate embeddings for corpus
    subprocess.run([
        "frozen-duckdb", "embed",
        "--input", corpus,
        "--output", "corpus_embeddings.json"
    ])

    # Filter corpus for relevant documents
    subprocess.run([
        "frozen-duckdb", "filter",
        "--criteria", filter_criteria,
        "--input", corpus,
        "--output", "relevant_docs.txt"
    ])

    # Search in filtered documents
    result = subprocess.run([
        "frozen-duckdb", "search",
        "--query", query,
        "--corpus", "relevant_docs.txt",
        "--format", "json"
    ], capture_output=True, text=True)

    return json.loads(result.stdout)

# Example usage
results = rag_with_filtering(
    "How does machine learning work?",
    "documents.txt",
    "Is this about machine learning or AI?"
)
```

### Document Classification

**Multi-label classification:**
```bash
# Classify documents by topic
frozen-duckdb filter --criteria "Is this about machine learning?" --input documents.txt --output ml_docs.txt
frozen-duckdb filter --criteria "Is this about databases?" --input documents.txt --output db_docs.txt
frozen-duckdb filter --criteria "Is this about programming?" --input documents.txt --output prog_docs.txt
```

**Automated tagging:**
```python
import subprocess

def auto_tag_documents(input_file):
    # Define categories
    categories = {
        "machine_learning": "Is this about machine learning or AI?",
        "databases": "Is this about databases or data storage?",
        "programming": "Is this about programming or software development?",
        "science": "Is this about scientific research or methodology?"
    }

    for category, criteria in categories.items():
        output_file = f"{category}_documents.txt"
        subprocess.run([
            "frozen-duckdb", "filter",
            "--criteria", criteria,
            "--input", input_file,
            "--output", output_file
        ])
        print(f"Tagged {category} documents → {output_file}")

# Example usage
auto_tag_documents("research_papers.txt")
```

## Performance Optimization

### Memory Management

**Large document handling:**
```bash
# Process large files in chunks
split -l 1000 large_document.txt chunk_

# Process each chunk separately
for chunk in chunk_*; do
    frozen-duckdb embed --input "$chunk" --output "${chunk}_embeddings.json"
done

# Combine results
cat chunk_*_embeddings.json | jq -s 'add' > combined_embeddings.json
```

**Memory monitoring:**
```bash
# Monitor memory usage during operations
htop

# Check DuckDB memory usage
duckdb -c "SELECT * FROM pragma_memory_usage();"

# Optimize for available memory
# Use smaller models if memory constrained
CREATE MODEL('fast_coder', 'qwen3-coder:7b', 'ollama');
```

### Batch Size Optimization

**Optimal batch sizes:**
```python
# Configuration for different operations
BATCH_CONFIG = {
    "embedding": 100,      # Process 100 texts at once
    "completion": 10,      # 10 completions at once
    "search": 50,         # Search in 50 documents at once
    "filter": 200,        # Filter 200 items at once
    "summarize": 20       # Summarize 20 documents at once
}

def process_in_batches(operation, items, batch_size):
    results = []
    for i in range(0, len(items), batch_size):
        batch = items[i:i + batch_size]
        batch_results = process_batch(operation, batch)
        results.extend(batch_results)
    return results
```

### Network Optimization

**Local Ollama (recommended):**
```bash
# Benefits: No network latency, complete privacy, reliable
ollama serve  # Local server

# Configure for local
CREATE SECRET ollama_secret (TYPE OLLAMA, API_URL 'http://localhost:11434');
```

**Remote Ollama (advanced):**
```bash
# For distributed setups
CREATE SECRET ollama_secret (TYPE OLLAMA, API_URL 'http://ollama-server:11434');

# With authentication
CREATE SECRET ollama_secret (
    TYPE OLLAMA,
    API_URL 'https://your-server:11434',
    API_KEY 'your-api-key'
);
```

## Error Handling and Troubleshooting

### Common LLM Issues

#### 1. Model Not Found

**Error:** `Model not found`

**Solutions:**
```bash
# Check available models
ollama list

# Pull missing models
ollama pull qwen3-coder:30b
ollama pull qwen3-embedding:8b

# Verify models are loaded
curl -s http://localhost:11434/api/tags
```

#### 2. Ollama Server Issues

**Error:** `Failed to connect to Ollama`

**Solutions:**
```bash
# Check server status
curl -s http://localhost:11434/api/version

# Restart server
ollama stop
ollama serve

# Check server logs
tail -f ~/.ollama/logs/server.log
```

#### 3. Flock Extension Issues

**Error:** `Flock extension not available`

**Solutions:**
```bash
# Re-setup Ollama integration
frozen-duckdb flock-setup

# Check extension status
duckdb -c "SELECT extension_name FROM duckdb_extensions() WHERE extension_name = 'flock';"

# Try alternative installation
INSTALL flock FROM community;
LOAD flock;
```

### Performance Issues

#### Slow Operations

**Diagnosis:**
```bash
# Monitor operation timing
time frozen-duckdb complete --prompt "test"

# Check system resources
top -p $(pgrep ollama) -p $(pgrep frozen-duckdb)

# Monitor memory usage
free -h
```

**Optimizations:**
```bash
# Use smaller models for faster responses
CREATE MODEL('fast_coder', 'qwen3-coder:7b', 'ollama');

# Process in smaller batches
# Use local Ollama only
# Monitor and limit concurrent operations
```

## Integration Examples

### Rust Integration

```rust
use std::process::Command;

fn generate_text_completion(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("frozen-duckdb")
        .args(&["complete", "--prompt", prompt])
        .output()?;

    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?)
    } else {
        Err(format!("LLM operation failed: {}", String::from_utf8(output.stderr)?).into())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate text completion
    let response = generate_text_completion("Explain quantum computing in simple terms")?;
    println!("LLM Response: {}", response);

    // Generate embeddings
    let output = Command::new("frozen-duckdb")
        .args(&["embed", "--text", "machine learning", "--output", "embeddings.json"])
        .output()?;

    println!("Embeddings generated successfully");
    Ok(())
}
```

### Python Integration

```python
import subprocess
import json

def llm_complete(prompt, model="coder"):
    """Generate text completion using frozen-duckdb"""
    result = subprocess.run([
        "frozen-duckdb", "complete",
        "--prompt", prompt,
        "--model", model
    ], capture_output=True, text=True)

    if result.returncode == 0:
        return result.stdout.strip()
    else:
        raise Exception(f"LLM completion failed: {result.stderr}")

def generate_embeddings(texts, output_file="embeddings.json"):
    """Generate embeddings for multiple texts"""
    # Write texts to temporary file
    with open("temp_texts.txt", "w") as f:
        for text in texts:
            f.write(text + "\n")

    # Generate embeddings
    result = subprocess.run([
        "frozen-duckdb", "embed",
        "--input", "temp_texts.txt",
        "--output", output_file
    ], capture_output=True, text=True)

    if result.returncode == 0:
        print(f"Embeddings saved to {output_file}")
        return output_file
    else:
        raise Exception(f"Embedding generation failed: {result.stderr}")

# Example usage
response = llm_complete("Write a Python function to calculate fibonacci numbers")
print("Generated code:", response)

embeddings = generate_embeddings([
    "machine learning",
    "artificial intelligence",
    "neural networks"
])
```

### Shell Script Integration

```bash
#!/bin/bash
# llm_workflow.sh

# Setup environment
source ../frozen-duckdb/prebuilt/setup_env.sh

# Generate embeddings for knowledge base
echo "🧠 Generating embeddings for knowledge base..."
frozen-duckdb embed --input knowledge_base.txt --output knowledge_embeddings.json

# Process user query
if [[ -n "$1" ]]; then
    QUERY="$1"
    echo "🔍 Searching for: $QUERY"

    # Search for relevant information
    frozen-duckdb search --query "$QUERY" --corpus knowledge_base.txt --format json

    # Generate comprehensive answer
    echo "Context from knowledge base:
$(frozen-duckdb search --query "$QUERY" --corpus knowledge_base.txt --format text | head -5)

Question: $QUERY" | frozen-duckdb complete
else
    echo "Usage: $0 <query>"
    echo "Example: $0 'How does machine learning work?'"
fi
```

## Best Practices

### 1. Performance Optimization

- **Use appropriate model sizes** for your hardware
- **Batch operations** for better throughput
- **Monitor resource usage** during intensive operations
- **Cache frequently used embeddings** to avoid recomputation

### 2. Quality Assurance

- **Validate LLM outputs** for accuracy and relevance
- **Test with known inputs** before production use
- **Monitor for hallucinations** and incorrect responses
- **Implement fallback strategies** for when LLM operations fail

### 3. Privacy and Security

- **Use local Ollama** for complete data privacy
- **Avoid sending sensitive data** to external services
- **Monitor access logs** for unauthorized usage
- **Implement rate limiting** for production deployments

### 4. Error Handling

- **Implement retry logic** for transient failures
- **Provide meaningful error messages** for debugging
- **Log operations** for monitoring and troubleshooting
- **Graceful degradation** when LLM services are unavailable

## Summary

LLM operations with Frozen DuckDB provide **powerful AI capabilities** with **local inference** for **privacy, performance, and reliability**. The system supports **multiple AI tasks** through a **consistent interface** with **comprehensive error handling** and **performance optimization**.

**Key Capabilities:**
- **Text completion**: Code generation, explanations, writing assistance
- **Embedding generation**: Vector representations for similarity search
- **Semantic search**: Find relevant content using meaning, not keywords
- **Intelligent filtering**: LLM-powered content classification
- **Text summarization**: Generate concise summaries of large documents

**Performance Characteristics:**
- **Local inference**: No network latency or external dependencies
- **Batch processing**: Efficient handling of multiple operations
- **Memory efficient**: Optimized for typical hardware configurations
- **Scalable architecture**: Handles varying workloads and data sizes

**Integration Options:**
- **Command-line interface**: Direct CLI operations for scripting
- **Rust integration**: Native Rust API for seamless integration
- **Python/Node.js support**: Works with popular scripting languages
- **SQL integration**: Direct LLM operations within DuckDB queries

**Use Cases:**
- **Code assistance**: Generate, explain, and debug code
- **Document processing**: Summarize, classify, and search large document collections
- **Content analysis**: Filter and categorize text content intelligently
- **RAG systems**: Build retrieval-augmented generation applications
- **Educational tools**: Generate explanations and learning materials
