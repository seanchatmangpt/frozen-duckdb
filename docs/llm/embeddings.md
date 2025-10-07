# Embedding Generation Guide

## Overview

**Embedding generation** creates **vector representations** of text that capture semantic meaning, enabling **similarity search**, **clustering**, and **semantic analysis** directly within DuckDB using the **Flock extension** and **Ollama models**.

## Basic Embedding Operations

### Single Text Embedding

**Generate embedding for text:**
```bash
# Generate embedding for single text
frozen-duckdb embed --text "machine learning"

# Output (JSON):
# [
#   {
#     "text": "machine learning",
#     "embedding": [0.123456, 0.789012, ...],
#     "dimensions": 1024
#   }
# ]
```

**Using SQL:**
```sql
-- Generate embedding for single text
SELECT llm_embedding(
    {'model_name': 'embedder'},
    [{'data': 'artificial intelligence'}]
) as embedding;
```

### Multiple Text Embeddings

**Batch processing:**
```bash
# Process multiple texts from file
cat > texts.txt << 'EOF'
machine learning
artificial intelligence
neural networks
deep learning
natural language processing
EOF

frozen-duckdb embed --input texts.txt --output embeddings.json
```

**Directory processing:**
```bash
# Process all text files in directory
frozen-duckdb embed --input ./documents/ --output all_embeddings.json
```

## Embedding Storage and Management

### Database Storage

**Create embeddings table:**
```sql
-- Create table for storing embeddings
CREATE TABLE document_embeddings (
    id INTEGER PRIMARY KEY,
    content TEXT,
    embedding FLOAT[1024],
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Insert embeddings
INSERT INTO document_embeddings (id, content, embedding)
SELECT
    row_number() OVER () as id,
    content,
    llm_embedding({'model_name': 'embedder'}, [{'data': content}]) as embedding
FROM documents;
```

**Optimized storage:**
```sql
-- Create table with metadata
CREATE TABLE embeddings (
    id INTEGER PRIMARY KEY,
    content_hash VARCHAR UNIQUE,
    content TEXT,
    embedding FLOAT[1024],
    model_used VARCHAR,
    dimensions INTEGER,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Insert with metadata
INSERT INTO embeddings (content_hash, content, embedding, model_used, dimensions)
SELECT
    hash(content) as content_hash,
    content,
    llm_embedding({'model_name': 'embedder'}, [{'data': content}]) as embedding,
    'qwen3-embedding:8b' as model_used,
    1024 as dimensions
FROM new_documents
ON CONFLICT (content_hash) DO NOTHING;
```

### File-Based Storage

**JSON format:**
```json
[
  {
    "text": "machine learning",
    "embedding": [0.123, 0.456, 0.789, ...],
    "dimensions": 1024,
    "normalized": false,
    "model": "qwen3-embedding:8b"
  }
]
```

**CSV format for analysis:**
```bash
# Export embeddings as CSV for external analysis
duckdb -c "
COPY (
    SELECT
        content,
        embedding[1] as dim_1,
        embedding[2] as dim_2,
        embedding[3] as dim_3
    FROM document_embeddings
) TO 'embeddings.csv' (FORMAT CSV, HEADER);
"
```

## Similarity Search

### Basic Similarity Search

**Cosine similarity calculation:**
```sql
-- Calculate similarity between two embeddings
SELECT array_distance(
    (SELECT embedding FROM document_embeddings WHERE content = 'machine learning'),
    (SELECT embedding FROM document_embeddings WHERE content = 'artificial intelligence')
) as similarity;

-- Find similar documents
SELECT
    content,
    array_distance(
        embedding,
        (SELECT embedding FROM document_embeddings WHERE content = 'machine learning')
    ) as similarity
FROM document_embeddings
WHERE similarity < 0.3
ORDER BY similarity
LIMIT 10;
```

**Advanced similarity search:**
```sql
-- Multi-vector similarity search
SELECT
    content,
    similarity_score,
    rank
FROM (
    SELECT
        content,
        array_distance(
            embedding,
            llm_embedding({'model_name': 'embedder'}, [{'data': 'neural networks'}])
        ) as similarity_score,
        ROW_NUMBER() OVER (ORDER BY array_distance(
            embedding,
            llm_embedding({'model_name': 'embedder'}, [{'data': 'neural networks'}])
        )) as rank
    FROM document_embeddings
) results
WHERE similarity_score < 0.25
ORDER BY similarity_score
LIMIT 20;
```

### Semantic Search Implementation

**Complete search pipeline:**
```sql
-- 1. Generate embeddings for search query
CREATE TEMP TABLE query_embedding AS
SELECT llm_embedding({'model_name': 'embedder'}, [{'data': 'machine learning applications'}]) as embedding;

-- 2. Find similar documents
CREATE TEMP TABLE similar_docs AS
SELECT
    content,
    array_distance(
        embedding,
        (SELECT embedding FROM query_embedding)
    ) as similarity
FROM document_embeddings
WHERE array_distance(
    embedding,
    (SELECT embedding FROM query_embedding)
) < 0.3
ORDER BY similarity
LIMIT 10;

-- 3. Return results
SELECT * FROM similar_docs;
```

## Embedding Normalization

### Why Normalize?

**Cosine similarity** works best with **normalized vectors** (unit length):
- **Before normalization**: Vectors have different magnitudes
- **After normalization**: All vectors have length 1
- **Similarity calculation**: More accurate semantic similarity

### Normalization Process

**Automatic normalization:**
```bash
# Generate normalized embeddings
frozen-duckdb embed --text "machine learning" --normalize

# Output includes normalized flag
# "normalized": true
```

**Manual normalization:**
```sql
-- Normalize embeddings using L2 normalization
SELECT
    embedding / SQRT(SUM(embedding * embedding) OVER ()) as normalized_embedding
FROM document_embeddings;
```

**Normalized similarity search:**
```sql
-- Search with normalized embeddings
SELECT
    content,
    1 - array_distance(
        normalized_embedding,
        (SELECT normalized_embedding FROM query_embedding)
    ) as cosine_similarity
FROM (
    SELECT
        content,
        embedding / SQRT(SUM(embedding * embedding) OVER ()) as normalized_embedding
    FROM document_embeddings
) normalized_docs
ORDER BY cosine_similarity DESC
LIMIT 10;
```

## Clustering and Analysis

### K-Means Clustering

**Simple clustering example:**
```sql
-- Group similar documents using embeddings
SELECT
    content,
    -- Simple clustering based on embedding patterns
    CASE
        WHEN embedding[1] > 0.5 THEN 'cluster_1'
        WHEN embedding[2] > 0.3 THEN 'cluster_2'
        ELSE 'cluster_3'
    END as cluster
FROM document_embeddings
ORDER BY cluster, content;
```

### Topic Modeling

**Extract topics from embeddings:**
```sql
-- Analyze embedding patterns to identify topics
CREATE TABLE topic_analysis AS
SELECT
    cluster,
    COUNT(*) as document_count,
    AVG(embedding[1]) as avg_dim_1,
    AVG(embedding[2]) as avg_dim_2,
    -- Sample documents from each cluster
    string_agg(content, ' | ') as sample_content
FROM (
    SELECT
        content,
        embedding,
        NTILE(5) OVER (ORDER BY embedding[1]) as cluster
    FROM document_embeddings
) clustered
GROUP BY cluster
ORDER BY cluster;
```

## Performance Optimization

### Batch Processing

**Efficient batch embedding:**
```bash
# Process large number of texts efficiently
split -l 1000 large_document.txt chunk_

# Process each chunk
for chunk in chunk_*; do
    frozen-duckdb embed --input "$chunk" --output "${chunk}_embeddings.json"
done

# Combine results
cat chunk_*_embeddings.json | jq -s 'add' > combined_embeddings.json
```

**Parallel processing:**
```python
import multiprocessing as mp
from functools import partial

def process_text_batch(texts, batch_id):
    """Process a batch of texts"""
    # Implementation would use frozen-duckdb CLI
    # or direct Ollama API calls
    pass

def parallel_embedding_generation(texts, num_processes=4):
    """Generate embeddings in parallel"""
    batch_size = len(texts) // num_processes
    batches = [texts[i:i+batch_size] for i in range(0, len(texts), batch_size)]

    with mp.Pool(num_processes) as pool:
        results = pool.map(partial(process_text_batch), enumerate(batches))

    return results

# Example usage
texts = ["text1", "text2", "text3", ...]  # Large list of texts
embeddings = parallel_embedding_generation(texts)
```

### Caching Strategy

**Embedding cache implementation:**
```sql
-- Create cache table
CREATE TABLE embedding_cache (
    content_hash VARCHAR PRIMARY KEY,
    content TEXT,
    embedding FLOAT[1024],
    model VARCHAR,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Check cache before computing
INSERT INTO embedding_cache (content_hash, content, embedding, model)
SELECT
    hash(content) as content_hash,
    content,
    llm_embedding({'model_name': 'embedder'}, [{'data': content}]) as embedding,
    'qwen3-embedding:8b' as model
FROM new_content
WHERE hash(content) NOT IN (SELECT content_hash FROM embedding_cache);
```

**Cache hit optimization:**
```sql
-- Get embedding with caching
CREATE OR REPLACE FUNCTION get_embedding_cached(text_content TEXT)
RETURNS FLOAT[1024] AS $$
DECLARE
    cached_embedding FLOAT[1024];
BEGIN
    -- Try to get from cache first
    SELECT embedding INTO cached_embedding
    FROM embedding_cache
    WHERE content_hash = hash(text_content)
    LIMIT 1;

    -- If not in cache, compute and store
    IF cached_embedding IS NULL THEN
        cached_embedding := llm_embedding(
            {'model_name': 'embedder'},
            [{'data': text_content}]
        );

        INSERT INTO embedding_cache (content_hash, content, embedding, model)
        VALUES (hash(text_content), text_content, cached_embedding, 'qwen3-embedding:8b');
    END IF;

    RETURN cached_embedding;
END;
$$ LANGUAGE plpgsql;
```

## Advanced Embedding Techniques

### Multi-Modal Embeddings

**Text + metadata embeddings:**
```sql
-- Combine text with metadata for richer embeddings
CREATE TABLE enriched_embeddings AS
SELECT
    content,
    metadata,
    -- Combine text and metadata for embedding
    llm_embedding(
        {'model_name': 'embedder'},
        [{'data': content || ' ' || metadata}]
    ) as combined_embedding,
    -- Separate embeddings for comparison
    llm_embedding({'model_name': 'embedder'}, [{'data': content}]) as text_embedding,
    llm_embedding({'model_name': 'embedder'}, [{'data': metadata}]) as metadata_embedding
FROM documents_with_metadata;
```

### Dimensionality Reduction

**PCA for visualization:**
```sql
-- Reduce 1024 dimensions to 2 for visualization
CREATE TABLE embedding_2d AS
SELECT
    content,
    -- Simple PCA approximation (first 2 components)
    embedding[1] as pc1,
    embedding[2] as pc2,
    -- Calculate magnitude for filtering
    SQRT(SUM(embedding * embedding)) as magnitude
FROM document_embeddings
WHERE SQRT(SUM(embedding * embedding)) > 0.1;  -- Filter weak embeddings
```

### Embedding Quality Assessment

**Embedding quality metrics:**
```sql
-- Calculate embedding statistics
SELECT
    COUNT(*) as total_embeddings,
    AVG(SQRT(SUM(embedding * embedding))) as avg_magnitude,
    MIN(SQRT(SUM(embedding * embedding))) as min_magnitude,
    MAX(SQRT(SUM(embedding * embedding))) as max_magnitude,
    -- Check for zero vectors
    COUNT(*) FILTER (WHERE SQRT(SUM(embedding * embedding)) < 0.01) as zero_vectors
FROM document_embeddings;
```

## Integration Examples

### Rust Integration

```rust
use duckdb::Connection;
use serde_json::{json, Value};

fn generate_embeddings(conn: &Connection, texts: Vec<&str>) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error>> {
    let mut embeddings = Vec::new();

    for text in texts {
        let embedding: Vec<f32> = conn.query_row(
            "SELECT llm_embedding(?, ?)",
            [
                &json!({"model_name": "embedder"}).to_string(),
                &json!([{"data": text}]).to_string()
            ],
            |row| {
                let embedding_str: String = row.get(0)?;
                serde_json::from_str(&embedding_str).unwrap_or_default()
            },
        )?;

        embeddings.push(embedding);
    }

    Ok(embeddings)
}

fn find_similar_documents(conn: &Connection, query: &str, threshold: f32) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let query_embedding: Vec<f32> = conn.query_row(
        "SELECT llm_embedding(?, ?)",
        [
            &json!({"model_name": "embedder"}).to_string(),
            &json!([{"data": query}]).to_string()
        ],
        |row| {
            let embedding_str: String = row.get(0)?;
            serde_json::from_str(&embedding_str).unwrap_or_default()
        },
    )?;

    let similar_docs: Vec<String> = conn.prepare(
        "SELECT content FROM document_embeddings WHERE array_distance(embedding, ?) < ? ORDER BY array_distance(embedding, ?) LIMIT 10"
    )?
    .query_map([&query_embedding, &threshold, &query_embedding], |row| row.get(0))?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(similar_docs)
}
```

### Python Integration

```python
import subprocess
import json
import numpy as np

class EmbeddingClient:
    def __init__(self):
        self.model = "embedder"

    def generate_embedding(self, text):
        """Generate embedding for single text"""
        result = subprocess.run([
            "frozen-duckdb", "embed",
            "--text", text,
            "--model", self.model,
            "--format", "json"
        ], capture_output=True, text=True)

        if result.returncode == 0:
            data = json.loads(result.stdout)
            return np.array(data[0]["embedding"])
        else:
            raise Exception(f"Embedding generation failed: {result.stderr}")

    def batch_embeddings(self, texts):
        """Generate embeddings for multiple texts"""
        # Write texts to temporary file
        with open("temp_texts.txt", "w") as f:
            for text in texts:
                f.write(text + "\n")

        # Generate embeddings
        result = subprocess.run([
            "frozen-duckdb", "embed",
            "--input", "temp_texts.txt",
            "--output", "temp_embeddings.json",
            "--model", self.model
        ], capture_output=True, text=True)

        if result.returncode == 0:
            with open("temp_embeddings.json", "r") as f:
                data = json.load(f)
                return [np.array(item["embedding"]) for item in data]
        else:
            raise Exception(f"Batch embedding failed: {result.stderr}")

    def semantic_search(self, query, corpus_embeddings, threshold=0.7):
        """Find similar texts using embeddings"""
        query_embedding = self.generate_embedding(query)

        similarities = []
        for i, emb in enumerate(corpus_embeddings):
            # Calculate cosine similarity
            similarity = np.dot(query_embedding, emb) / (np.linalg.norm(query_embedding) * np.linalg.norm(emb))
            if similarity >= threshold:
                similarities.append((i, similarity))

        return sorted(similarities, key=lambda x: x[1], reverse=True)

# Example usage
client = EmbeddingClient()

# Generate embeddings for documents
documents = ["machine learning", "artificial intelligence", "neural networks"]
embeddings = client.batch_embeddings(documents)

# Search for similar content
results = client.semantic_search("AI systems", embeddings)
print(f"Found {len(results)} similar documents")
```

### Similarity Search Pipeline

```bash
#!/bin/bash
# similarity_search_pipeline.sh

# 1. Generate embeddings for knowledge base
echo "🧠 Generating embeddings for knowledge base..."
frozen-duckdb embed --input knowledge_base.txt --output knowledge_embeddings.json

# 2. Create searchable database
duckdb knowledge.duckdb -c "
CREATE TABLE knowledge_base AS
SELECT
    content,
    embedding
FROM read_json('knowledge_embeddings.json');
"

# 3. Search interface
if [[ -n "$1" ]]; then
    QUERY="$1"
    echo "🔍 Searching for: $QUERY"

    # Find similar content
    duckdb knowledge.duckdb -c "
    SELECT content, similarity
    FROM (
        SELECT
            content,
            1 - array_distance(
                embedding,
                (SELECT embedding FROM knowledge_base WHERE content = '$QUERY' LIMIT 1)
            ) as similarity
        FROM knowledge_base
        WHERE similarity > 0.7
    ) results
    ORDER BY similarity DESC
    LIMIT 5;
    "
else
    echo "Usage: $0 <search_query>"
    echo "Example: $0 'machine learning applications'"
fi
```

## Performance Benchmarks

### Embedding Performance

| Model | Text Length | Generation Time | Memory Usage |
|-------|-------------|----------------|--------------|
| **qwen3-embedding:8b** | Short (~50 chars) | 0.5-1s | 150MB |
| **qwen3-embedding:8b** | Medium (~200 chars) | 1-2s | 200MB |
| **qwen3-embedding:8b** | Long (~1000 chars) | 2-3s | 300MB |

### Search Performance

| Corpus Size | Query Time | Memory Usage | Accuracy |
|-------------|------------|--------------|----------|
| **100 documents** | 0.2-0.5s | 50MB | High |
| **1,000 documents** | 0.5-1s | 100MB | High |
| **10,000 documents** | 2-5s | 500MB | Good |

### Batch Processing Performance

| Batch Size | Processing Time | Memory Usage | Efficiency |
|------------|----------------|--------------|------------|
| **10 texts** | 5-10s | 200MB | Good |
| **100 texts** | 30-60s | 500MB | Better |
| **1,000 texts** | 5-10 minutes | 2GB | Best |

## Troubleshooting

### Common Issues

#### 1. Poor Embedding Quality

**Problem:** Embeddings don't capture semantic meaning well

**Solutions:**
```bash
# Use higher quality model
frozen-duckdb embed --text "complex topic" --model quality_embedder

# Check text preprocessing
# Remove stop words, normalize text
frozen-duckdb embed --text "cleaned and normalized text"
```

#### 2. Memory Issues

**Problem:** Out of memory during embedding generation

**Solutions:**
```bash
# Process in smaller batches
split -l 100 large_file.txt chunk_
for chunk in chunk_*; do
    frozen-duckdb embed --input "$chunk" --output "${chunk}_embeddings.json"
done

# Use smaller model
CREATE MODEL('fast_embedder', 'qwen3-embedding:4b', 'ollama');
```

#### 3. Similarity Issues

**Problem:** Similar texts have low similarity scores

**Solutions:**
```sql
-- Check embedding normalization
SELECT
    content,
    SQRT(SUM(embedding * embedding)) as magnitude
FROM document_embeddings
ORDER BY magnitude;

-- Normalize embeddings
UPDATE document_embeddings
SET embedding = embedding / SQRT(SUM(embedding * embedding));
```

## Best Practices

### 1. Text Preprocessing

**Clean text before embedding:**
```python
import re

def preprocess_text(text):
    """Clean text for better embeddings"""
    # Remove extra whitespace
    text = re.sub(r'\s+', ' ', text)
    # Remove special characters (optional)
    text = re.sub(r'[^\w\s]', '', text)
    # Convert to lowercase (optional)
    text = text.lower()
    return text.strip()

# Example usage
cleaned_text = preprocess_text("Machine Learning & AI!!!")
embedding = generate_embedding(cleaned_text)
```

### 2. Embedding Storage

**Efficient storage strategies:**
```sql
-- Store with compression for large datasets
CREATE TABLE compressed_embeddings AS
SELECT
    id,
    content,
    -- Compress embedding data
    compress(embedding::BLOB) as compressed_embedding
FROM document_embeddings;

-- Query with decompression
SELECT
    content,
    decompress(compressed_embedding)::FLOAT[] as embedding
FROM compressed_embeddings;
```

### 3. Performance Optimization

**Batch processing optimization:**
```python
def optimal_batch_size(texts, target_time=30):
    """Find optimal batch size for target processing time"""
    # Start with small batch
    batch_size = 10
    while batch_size < len(texts):
        start_time = time.time()
        # Test batch processing time
        test_batch = texts[:batch_size]
        # ... process test batch ...

        elapsed = time.time() - start_time
        if elapsed > target_time:
            return max(1, batch_size // 2)
        batch_size *= 2

    return batch_size

# Example usage
texts = load_large_document()
optimal_size = optimal_batch_size(texts)
batches = [texts[i:i+optimal_size] for i in range(0, len(texts), optimal_size)]
```

## Summary

Embedding generation with Frozen DuckDB provides **powerful semantic analysis** capabilities with **local processing** for **privacy and performance**. The system supports **single and batch embedding generation**, **similarity search**, **clustering**, and **advanced text analysis** workflows.

**Key Capabilities:**
- **Vector embeddings**: 1024-dimensional semantic representations
- **Similarity search**: Find semantically similar content
- **Batch processing**: Efficient handling of large text collections
- **Caching strategies**: Avoid recomputation of existing embeddings

**Performance Characteristics:**
- **Generation speed**: 1-3 seconds per text for typical content
- **Search speed**: 0.5-2 seconds for corpus searches
- **Memory efficiency**: Optimized for typical hardware configurations
- **Scalability**: Handles large document collections efficiently

**Use Cases:**
- **Document similarity**: Find related documents and content
- **Content clustering**: Group similar texts automatically
- **Search enhancement**: Improve search with semantic understanding
- **Recommendation systems**: Suggest related content based on embeddings
- **Text analysis**: Understand content relationships and patterns
