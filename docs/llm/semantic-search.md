# Semantic Search Guide

## Overview

**Semantic search** finds content based on **meaning and context** rather than exact keyword matches, using **embeddings** and **similarity calculations** to identify relevant documents. Frozen DuckDB enables **powerful semantic search** directly within database queries using the **Flock extension**.

## Basic Semantic Search

### Simple Search Implementation

**Search for similar content:**
```bash
# Basic semantic search
frozen-duckdb search --query "machine learning" --corpus documents.txt

# Output:
# 🔍 Found 3 similar documents:
#   1. "Machine learning algorithms use neural networks" (similarity: 0.892)
#   2. "Deep learning is a subset of machine learning" (similarity: 0.856)
#   3. "AI systems learn from data patterns" (similarity: 0.743)
```

**Using SQL:**
```sql
-- Search for similar documents
SELECT
    content,
    similarity_score
FROM (
    SELECT
        content,
        1 - array_distance(
            embedding,
            llm_embedding({'model_name': 'embedder'}, [{'data': 'machine learning'}])
        ) as similarity_score
    FROM document_embeddings
) results
WHERE similarity_score > 0.7
ORDER BY similarity_score DESC
LIMIT 10;
```

### Search Configuration

**Custom thresholds and limits:**
```bash
# High precision search (higher threshold)
frozen-duckdb search --query "database optimization" --corpus papers.txt --threshold 0.8

# Broad search (lower threshold)
frozen-duckdb search --query "programming" --corpus code.txt --threshold 0.5

# Limited results
frozen-duckdb search --query "artificial intelligence" --corpus research.txt --limit 5
```

**JSON output for processing:**
```bash
# Get structured results for programmatic use
frozen-duckdb search --query "neural networks" --corpus documents.txt --format json

# Output:
# [
#   {
#     "document": "Deep learning uses neural networks for pattern recognition",
#     "similarity_score": 0.892
#   }
# ]
```

## Advanced Search Techniques

### Multi-Query Search

**Search with multiple related queries:**
```sql
-- Search using multiple related terms
CREATE TEMP TABLE search_queries AS
SELECT 'machine learning' as query
UNION ALL
SELECT 'artificial intelligence' as query
UNION ALL
SELECT 'neural networks' as query;

-- Combine results from multiple queries
SELECT
    content,
    AVG(similarity_score) as avg_similarity,
    MAX(similarity_score) as max_similarity
FROM (
    SELECT
        de.content,
        1 - array_distance(
            de.embedding,
            llm_embedding({'model_name': 'embedder'}, [{'data': sq.query}])
        ) as similarity_score
    FROM document_embeddings de
    CROSS JOIN search_queries sq
) combined_results
GROUP BY content
HAVING AVG(similarity_score) > 0.6
ORDER BY avg_similarity DESC
LIMIT 15;
```

### Weighted Search

**Search with query term weighting:**
```sql
-- Give different weights to search terms
SELECT
    content,
    (
        0.5 * (1 - array_distance(embedding, (SELECT embedding FROM query_embeddings WHERE term = 'machine learning'))) +
        0.3 * (1 - array_distance(embedding, (SELECT embedding FROM query_embeddings WHERE term = 'algorithms'))) +
        0.2 * (1 - array_distance(embedding, (SELECT embedding FROM query_embeddings WHERE term = 'optimization')))
    ) as weighted_similarity
FROM document_embeddings
ORDER BY weighted_similarity DESC
LIMIT 10;
```

### Contextual Search

**Search within specific contexts:**
```sql
-- Search within technical documents only
SELECT content, similarity
FROM (
    SELECT
        de.content,
        1 - array_distance(
            de.embedding,
            llm_embedding({'model_name': 'embedder'}, [{'data': 'advanced algorithms'}])
        ) as similarity
    FROM document_embeddings de
    JOIN document_metadata dm ON de.document_id = dm.document_id
    WHERE dm.category = 'technical'
        AND dm.difficulty_level = 'advanced'
) technical_results
WHERE similarity > 0.7
ORDER BY similarity DESC;
```

## Search Index Management

### Creating Search Indexes

**Efficient search with pre-computed embeddings:**
```sql
-- Create search-optimized table
CREATE TABLE search_index (
    document_id INTEGER PRIMARY KEY,
    content TEXT,
    embedding FLOAT[1024],
    content_hash VARCHAR UNIQUE,
    metadata JSON,
    indexed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Populate with embeddings
INSERT INTO search_index (document_id, content, embedding, content_hash, metadata)
SELECT
    id,
    content,
    llm_embedding({'model_name': 'embedder'}, [{'data': content}]) as embedding,
    hash(content) as content_hash,
    json_object('category', category, 'tags', tags) as metadata
FROM documents;
```

**Incremental index updates:**
```sql
-- Add new documents to index
INSERT INTO search_index (document_id, content, embedding, content_hash)
SELECT
    id,
    content,
    llm_embedding({'model_name': 'embedder'}, [{'data': content}]) as embedding,
    hash(content) as content_hash
FROM new_documents
WHERE hash(content) NOT IN (SELECT content_hash FROM search_index);

-- Update existing documents
UPDATE search_index
SET
    content = new_content,
    embedding = llm_embedding({'model_name': 'embedder'}, [{'data': new_content}]),
    content_hash = hash(new_content),
    indexed_at = CURRENT_TIMESTAMP
FROM updated_documents
WHERE search_index.document_id = updated_documents.id;
```

### Search Performance Optimization

**Indexed search:**
```sql
-- Fast search using pre-computed embeddings
SELECT
    content,
    1 - array_distance(
        embedding,
        llm_embedding({'model_name': 'embedder'}, [{'data': 'information retrieval'}])
    ) as similarity
FROM search_index
WHERE 1 - array_distance(
    embedding,
    llm_embedding({'model_name': 'embedder'}, [{'data': 'information retrieval'}])
) > 0.7
ORDER BY similarity DESC
LIMIT 20;
```

**Filtered search:**
```sql
-- Search within specific categories
SELECT
    content,
    similarity,
    metadata->'category' as category
FROM (
    SELECT
        content,
        metadata,
        1 - array_distance(
            embedding,
            (SELECT embedding FROM query_embedding)
        ) as similarity
    FROM search_index
    WHERE metadata->>'category' = 'research'
) category_results
WHERE similarity > 0.6
ORDER BY similarity DESC;
```

## Similarity Metrics

### Cosine Similarity

**Standard similarity metric:**
```sql
-- Calculate cosine similarity
SELECT
    content,
    -- Cosine similarity = 1 - cosine_distance
    1 - array_distance(embedding, query_embedding) as cosine_similarity
FROM document_embeddings, query_embedding
ORDER BY cosine_similarity DESC;
```

**Normalized similarity:**
```sql
-- Similarity with normalized embeddings
SELECT
    content,
    -- Dot product of normalized vectors
    SUM(normalized_embedding * query_normalized_embedding) as cosine_similarity
FROM (
    SELECT
        content,
        embedding / SQRT(SUM(embedding * embedding) OVER ()) as normalized_embedding
    FROM document_embeddings
) normalized_docs, query_normalized_embedding
GROUP BY content, normalized_embedding
ORDER BY cosine_similarity DESC;
```

### Alternative Similarity Metrics

**Euclidean distance:**
```sql
-- Euclidean distance (lower is more similar)
SELECT
    content,
    SQRT(SUM((embedding - query_embedding) * (embedding - query_embedding))) as euclidean_distance
FROM document_embeddings, query_embedding
ORDER BY euclidean_distance
LIMIT 10;
```

**Manhattan distance:**
```sql
-- Manhattan distance for high-dimensional data
SELECT
    content,
    SUM(ABS(embedding - query_embedding)) as manhattan_distance
FROM document_embeddings, query_embedding
ORDER BY manhattan_distance
LIMIT 10;
```

## Search Result Processing

### Result Ranking

**Multi-factor ranking:**
```sql
-- Rank by similarity and recency
SELECT
    content,
    similarity_score,
    metadata->'created_date' as created_date,
    -- Combined score (similarity + recency bonus)
    similarity_score + (julianday('now') - julianday(metadata->'created_date')) * 0.01 as combined_score
FROM (
    SELECT
        content,
        metadata,
        1 - array_distance(embedding, query_embedding) as similarity_score
    FROM search_index, query_embedding
) results
WHERE similarity_score > 0.6
ORDER BY combined_score DESC
LIMIT 15;
```

**Diversity ranking:**
```sql
-- Ensure diverse results across categories
SELECT
    content,
    similarity_score,
    metadata->'category' as category,
    ROW_NUMBER() OVER (PARTITION BY metadata->'category' ORDER BY similarity_score DESC) as category_rank
FROM search_results
WHERE similarity_score > 0.7
    AND ROW_NUMBER() OVER (PARTITION BY metadata->'category' ORDER BY similarity_score DESC) <= 3
ORDER BY similarity_score DESC;
```

### Result Filtering

**Content-based filtering:**
```sql
-- Filter results by content characteristics
SELECT content, similarity_score
FROM search_results
WHERE similarity_score > 0.7
    -- Filter out very short results
    AND LENGTH(content) > 50
    -- Filter out results with too many special characters
    AND LENGTH(REGEXP_REPLACE(content, '[^a-zA-Z0-9\s]', '')) > LENGTH(content) * 0.8
ORDER BY similarity_score DESC;
```

**Metadata-based filtering:**
```sql
-- Filter by document metadata
SELECT
    content,
    similarity_score,
    metadata->'author' as author,
    metadata->'publication_date' as pub_date
FROM search_results
WHERE similarity_score > 0.6
    AND metadata->>'author' IS NOT NULL
    AND metadata->>'publication_date' > '2020-01-01'
ORDER BY similarity_score DESC, pub_date DESC;
```

## Real-World Search Applications

### Document Search Engine

**Complete search implementation:**
```sql
-- Create search function
CREATE OR REPLACE FUNCTION semantic_search(search_query TEXT, min_similarity FLOAT DEFAULT 0.7, max_results INTEGER DEFAULT 10)
RETURNS TABLE(content TEXT, similarity FLOAT) AS $$
BEGIN
    RETURN QUERY
    SELECT
        si.content,
        1 - array_distance(si.embedding, qe.embedding) as similarity
    FROM search_index si, (
        SELECT llm_embedding({'model_name': 'embedder'}, [{'data': search_query}]) as embedding
    ) qe
    WHERE 1 - array_distance(si.embedding, qe.embedding) >= min_similarity
    ORDER BY similarity DESC
    LIMIT max_results;
END;
$$ LANGUAGE plpgsql;

-- Use search function
SELECT * FROM semantic_search('machine learning applications', 0.7, 15);
```

### Code Search

**Search code repositories:**
```sql
-- Search for code patterns
SELECT
    file_path,
    function_name,
    code_snippet,
    similarity
FROM code_embeddings
WHERE similarity > 0.75
ORDER BY similarity DESC
LIMIT 20;
```

**Function search:**
```sql
-- Find functions with similar purposes
SELECT
    function_name,
    file_path,
    similarity
FROM (
    SELECT
        function_name,
        file_path,
        1 - array_distance(
            embedding,
            llm_embedding({'model_name': 'embedder'}, [{'data': 'calculate fibonacci numbers'}])
        ) as similarity
    FROM code_function_embeddings
) function_results
WHERE similarity > 0.7
ORDER BY similarity DESC;
```

### Research Paper Search

**Academic paper search:**
```sql
-- Search research papers by topic
SELECT
    title,
    abstract,
    authors,
    publication_year,
    similarity
FROM research_papers
WHERE similarity > 0.8
ORDER BY similarity DESC, publication_year DESC
LIMIT 25;
```

**Citation-based enhancement:**
```sql
-- Search with citation context
SELECT
    paper_title,
    cited_by_count,
    -- Boost score for highly cited papers
    similarity * (1 + LOG(10 + cited_by_count) * 0.1) as enhanced_similarity
FROM (
    SELECT
        paper_title,
        cited_by_count,
        1 - array_distance(
            embedding,
            llm_embedding({'model_name': 'embedder'}, [{'data': 'quantum computing research'}])
        ) as similarity
    FROM research_paper_embeddings rpe
    JOIN citation_data cd ON rpe.paper_id = cd.paper_id
) enhanced_results
WHERE similarity > 0.6
ORDER BY enhanced_similarity DESC;
```

## Performance Optimization

### Indexing Strategy

**Approximate nearest neighbor search:**
```sql
-- For large datasets, use approximate methods
CREATE INDEX embedding_idx ON document_embeddings USING ivfflat (embedding vector_cosine_ops)
WITH (lists = 100);

-- Faster approximate search
SELECT content
FROM document_embeddings
WHERE embedding <-> (SELECT embedding FROM query_embedding) < 0.3
ORDER BY embedding <-> (SELECT embedding FROM query_embedding)
LIMIT 20;
```

**Hierarchical clustering:**
```sql
-- Create hierarchy for faster search
CREATE TABLE embedding_clusters AS
SELECT
    cluster_id,
    AVG(embedding) as cluster_center,
    COUNT(*) as cluster_size
FROM (
    SELECT
        content,
        embedding,
        -- Simple clustering based on first dimension
        NTILE(100) OVER (ORDER BY embedding[1]) as cluster_id
    FROM document_embeddings
) clustered
GROUP BY cluster_id;
```

### Caching Strategy

**Query result caching:**
```sql
-- Cache frequent search results
CREATE TABLE search_cache (
    query_hash VARCHAR PRIMARY KEY,
    search_query TEXT,
    results JSON,
    cached_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    hit_count INTEGER DEFAULT 1
);

-- Check cache before expensive search
CREATE OR REPLACE FUNCTION cached_semantic_search(search_query TEXT, min_similarity FLOAT DEFAULT 0.7)
RETURNS JSON AS $$
DECLARE
    query_hash VARCHAR;
    cached_results JSON;
BEGIN
    query_hash := hash(search_query);

    -- Try to get from cache first
    SELECT results INTO cached_results
    FROM search_cache
    WHERE query_hash = cached_results.query_hash
        AND cached_at > CURRENT_TIMESTAMP - INTERVAL '1 hour'
    LIMIT 1;

    IF cached_results IS NOT NULL THEN
        -- Update hit count
        UPDATE search_cache SET hit_count = hit_count + 1 WHERE query_hash = cached_results.query_hash;
        RETURN cached_results;
    END IF;

    -- Perform search and cache results
    -- ... perform actual search ...
    cached_results := json_array(results);

    INSERT INTO search_cache (query_hash, search_query, results)
    VALUES (query_hash, search_query, cached_results)
    ON CONFLICT (query_hash) DO UPDATE SET
        results = cached_results,
        cached_at = CURRENT_TIMESTAMP,
        hit_count = search_cache.hit_count + 1;

    RETURN cached_results;
END;
$$ LANGUAGE plpgsql;
```

## Integration Examples

### Web Search Interface

**Simple web search API:**
```python
from flask import Flask, request, jsonify
import subprocess
import json

app = Flask(__name__)

@app.route('/search', methods=['POST'])
def semantic_search():
    data = request.get_json()
    query = data.get('query', '')
    threshold = data.get('threshold', 0.7)
    limit = data.get('limit', 10)

    # Run semantic search
    result = subprocess.run([
        "frozen-duckdb", "search",
        "--query", query,
        "--corpus", "knowledge_base.txt",
        "--threshold", str(threshold),
        "--limit", str(limit),
        "--format", "json"
    ], capture_output=True, text=True)

    if result.returncode == 0:
        results = json.loads(result.stdout)
        return jsonify({"results": results})
    else:
        return jsonify({"error": "Search failed"}), 500

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)
```

### Chatbot Integration

**RAG-powered chatbot:**
```python
import subprocess
import json

class SemanticChatbot:
    def __init__(self, knowledge_base):
        self.knowledge_base = knowledge_base

    def answer_question(self, question):
        # Search for relevant context
        search_result = subprocess.run([
            "frozen-duckdb", "search",
            "--query", question,
            "--corpus", self.knowledge_base,
            "--threshold", "0.7",
            "--limit", "3",
            "--format", "json"
        ], capture_output=True, text=True)

        if search_result.returncode == 0:
            context_docs = json.loads(search_result.stdout)
            context = " ".join([doc["document"] for doc in context_docs])

            # Generate answer with context
            answer_result = subprocess.run([
                "frozen-duckdb", "complete",
                "--prompt", f"Based on this context: {context}\n\nAnswer this question: {question}"
            ], capture_output=True, text=True)

            if answer_result.returncode == 0:
                return answer_result.stdout.strip()

        return "I don't have enough information to answer that question."

# Example usage
bot = SemanticChatbot("company_knowledge.txt")
answer = bot.answer_question("How do I reset my password?")
print(answer)
```

### Document Management System

**Intelligent document search:**
```rust
use duckdb::Connection;
use serde_json::{json, Value};

struct DocumentSearch {
    conn: Connection,
}

impl DocumentSearch {
    fn search_documents(&self, query: &str, threshold: f32) -> Result<Vec<(String, f32)>, Box<dyn std::error::Error>> {
        let query_embedding: Vec<f32> = self.conn.query_row(
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

        let results: Vec<(String, f32)> = self.conn.prepare(
            "SELECT content, 1 - array_distance(embedding, ?) FROM document_embeddings WHERE 1 - array_distance(embedding, ?) > ? ORDER BY 1 - array_distance(embedding, ?) DESC LIMIT 10"
        )?
        .query_map([&query_embedding, &query_embedding, &threshold], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, f32>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(results)
    }
}
```

## Best Practices

### 1. Query Optimization

**Clear, specific queries:**
```bash
# Good query
frozen-duckdb search --query "machine learning algorithms for image classification"

# Less specific query
frozen-duckdb search --query "machine learning"
```

**Query expansion:**
```sql
-- Expand query with related terms
CREATE TEMP TABLE expanded_query AS
SELECT 'machine learning' as term
UNION ALL
SELECT 'neural networks' as term
UNION ALL
SELECT 'computer vision' as term
UNION ALL
SELECT 'deep learning' as term;

-- Search with multiple terms
SELECT
    content,
    AVG(similarity) as avg_similarity
FROM (
    SELECT
        de.content,
        1 - array_distance(de.embedding, qe.embedding) as similarity
    FROM document_embeddings de
    CROSS JOIN (
        SELECT llm_embedding({'model_name': 'embedder'}, [{'data': term}]) as embedding
        FROM expanded_query
    ) qe
) expanded_results
GROUP BY content
HAVING AVG(similarity) > 0.6
ORDER BY avg_similarity DESC;
```

### 2. Threshold Selection

**Threshold guidelines:**
- **High precision (0.8+)**: Only very similar content
- **Balanced (0.6-0.8)**: Good mix of precision and recall
- **Broad search (0.4-0.6)**: Include somewhat related content
- **Very broad (<0.4)**: Include loosely related content

### 3. Performance Monitoring

**Search performance tracking:**
```sql
-- Track search performance
CREATE TABLE search_metrics (
    search_query TEXT,
    results_found INTEGER,
    avg_similarity FLOAT,
    search_time_ms INTEGER,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Log search metrics
INSERT INTO search_metrics (search_query, results_found, avg_similarity, search_time_ms)
SELECT
    'machine learning' as search_query,
    COUNT(*) as results_found,
    AVG(similarity_score) as avg_similarity,
    150 as search_time_ms
FROM search_results;
```

## Troubleshooting

### Common Issues

#### 1. No Results Found

**Problem:** Search returns no results

**Solutions:**
```bash
# Lower similarity threshold
frozen-duckdb search --query "topic" --threshold 0.5

# Check if corpus has content
wc -l documents.txt

# Verify embeddings exist
duckdb -c "SELECT COUNT(*) FROM document_embeddings;"
```

#### 2. Poor Result Quality

**Problem:** Results not relevant to query

**Solutions:**
```sql
-- Check embedding quality
SELECT
    content,
    SQRT(SUM(embedding * embedding)) as magnitude
FROM document_embeddings
ORDER BY magnitude
LIMIT 10;

-- Try different query terms
frozen-duckdb search --query "artificial intelligence" --corpus documents.txt

-- Check for query-document mismatch
frozen-duckdb search --query "machine learning" --corpus documents.txt --format json
```

#### 3. Performance Issues

**Problem:** Search operations are slow

**Solutions:**
```bash
# Use smaller corpus for testing
head -1000 documents.txt > small_corpus.txt
frozen-duckdb search --query "test" --corpus small_corpus.txt

# Check system resources
htop | grep frozen-duckdb

# Optimize batch sizes
# Process embeddings in smaller batches
```

## Summary

Semantic search with Frozen DuckDB provides **powerful content discovery** capabilities using **embeddings and similarity calculations**. The system supports **single and multi-query search**, **result ranking and filtering**, **performance optimization**, and **integration with various applications**.

**Key Features:**
- **Semantic understanding**: Find content based on meaning, not keywords
- **Flexible similarity metrics**: Cosine similarity, Euclidean distance, Manhattan distance
- **Advanced filtering**: Content-based and metadata-based result filtering
- **Performance optimization**: Indexing, caching, and batch processing

**Performance Characteristics:**
- **Search speed**: 0.5-2 seconds for typical corpus searches
- **Result quality**: High relevance with appropriate thresholds
- **Scalability**: Handles large document collections efficiently
- **Integration**: Works with web APIs, chatbots, and document systems

**Use Cases:**
- **Document search**: Find relevant documents in large collections
- **Code search**: Discover similar code patterns and functions
- **Research assistance**: Find related academic papers and articles
- **Content recommendation**: Suggest relevant content to users
- **Knowledge management**: Build intelligent knowledge bases
