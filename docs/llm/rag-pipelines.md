# RAG Pipelines Guide

## Overview

**Retrieval-Augmented Generation (RAG)** combines **information retrieval** with **text generation** to create **context-aware AI responses**. Frozen DuckDB enables **complete RAG pipelines** directly within database workflows using **embeddings**, **semantic search**, and **LLM completion**.

## Basic RAG Implementation

### Simple RAG Pipeline

**Complete RAG workflow:**
```bash
#!/bin/bash
# basic_rag_pipeline.sh

# 1. Generate embeddings for knowledge base
echo "🧠 Generating embeddings for knowledge base..."
frozen-duckdb embed --input knowledge_base.txt --output knowledge_embeddings.json

# 2. Search for relevant context
if [[ -n "$1" ]]; then
    QUERY="$1"
    echo "🔍 Searching for: $QUERY"

    # Find most relevant documents
    CONTEXT=$(frozen-duckdb search --query "$QUERY" --corpus knowledge_base.txt --threshold 0.8 --limit 3 --format text | head -10)

    # 3. Generate answer with context
    echo "📝 Generating answer with context..."
    echo "Context: $CONTEXT

Question: $QUERY
Please provide a comprehensive answer based on the context above." | frozen-duckdb complete
else
    echo "Usage: $0 <question>"
    echo "Example: $0 'How does machine learning work?'"
fi
```

**SQL-based RAG:**
```sql
-- Complete RAG pipeline in SQL
WITH query_embedding AS (
    SELECT llm_embedding({'model_name': 'embedder'}, [{'data': 'machine learning applications'}]) as embedding
),
relevant_docs AS (
    SELECT
        content,
        1 - array_distance(embedding, (SELECT embedding FROM query_embedding)) as similarity
    FROM document_embeddings
    WHERE 1 - array_distance(embedding, (SELECT embedding FROM query_embedding)) > 0.7
    ORDER BY similarity DESC
    LIMIT 5
),
context AS (
    SELECT string_agg(content, ' ') as combined_context
    FROM relevant_docs
)
SELECT llm_complete(
    {'model_name': 'coder'},
    {
        'prompt_name': 'rag_answer',
        'context_columns': [{
            'data': 'Based on this context: ' || (SELECT combined_context FROM context) || ' Answer this question: machine learning applications'
        }]
    }
) as answer;
```

## Advanced RAG Patterns

### Multi-Step RAG

**Iterative refinement:**
```sql
-- 1. Initial broad search
CREATE TEMP TABLE initial_results AS
SELECT
    content,
    1 - array_distance(embedding, query_embedding) as similarity
FROM document_embeddings, (
    SELECT llm_embedding({'model_name': 'embedder'}, [{'data': 'machine learning'}]) as embedding
) qe
WHERE similarity > 0.5
ORDER BY similarity DESC
LIMIT 10;

-- 2. Filter for relevance using LLM
CREATE TEMP TABLE filtered_results AS
SELECT
    content,
    llm_filter(
        {'model_name': 'coder'},
        {'prompt_name': 'relevance_filter'},
        [{'data': 'Is this relevant to machine learning applications?'}]
    ) as is_relevant
FROM initial_results;

-- 3. Generate answer with filtered context
SELECT llm_complete(
    {'model_name': 'coder'},
    {
        'prompt_name': 'comprehensive_answer',
        'context_columns': [{
            'data': 'Answer based on these relevant documents: ' || (
                SELECT string_agg(content, ' ') FROM filtered_results WHERE is_relevant = true
            ) || ' Question: What are practical applications of machine learning?'
        }]
    }
) as final_answer;
```

### Hierarchical RAG

**Multi-level retrieval:**
```sql
-- 1. Coarse-grained search across categories
CREATE TEMP TABLE category_matches AS
SELECT
    category,
    AVG(similarity) as avg_similarity
FROM (
    SELECT
        dm.category,
        1 - array_distance(de.embedding, qe.embedding) as similarity
    FROM document_embeddings de
    JOIN document_metadata dm ON de.document_id = dm.document_id,
    (SELECT llm_embedding({'model_name': 'embedder'}, [{'data': 'artificial intelligence'}]) as embedding) qe
) category_results
GROUP BY category
HAVING AVG(similarity) > 0.6
ORDER BY avg_similarity DESC
LIMIT 3;

-- 2. Fine-grained search within top categories
CREATE TEMP TABLE detailed_results AS
SELECT
    de.content,
    dm.category,
    1 - array_distance(de.embedding, qe.embedding) as similarity
FROM document_embeddings de
JOIN document_metadata dm ON de.document_id = dm.document_id
JOIN category_matches cm ON dm.category = cm.category,
(SELECT llm_embedding({'model_name': 'embedder'}, [{'data': 'artificial intelligence'}]) as embedding) qe
WHERE similarity > 0.7
ORDER BY similarity DESC
LIMIT 15;

-- 3. Generate answer with categorized context
SELECT llm_complete(
    {'model_name': 'coder'},
    {
        'prompt_name': 'categorized_answer',
        'context_columns': [{
            'data': 'Provide a comprehensive answer organized by categories. Categories: ' ||
                   (SELECT string_agg(category, ', ') FROM category_matches) ||
                   ' Context: ' || (SELECT string_agg(content, ' ') FROM detailed_results) ||
                   ' Question: What are the main applications of artificial intelligence?'
        }]
    }
) as organized_answer;
```

## RAG Pipeline Components

### 1. Document Ingestion

**Automated document processing:**
```sql
-- Ingest documents with metadata extraction
CREATE TABLE document_collection AS
SELECT
    id,
    content,
    -- Extract metadata using LLM
    llm_complete(
        {'model_name': 'coder'},
        {
            'prompt_name': 'metadata_extractor',
            'context_columns': [{'data': 'Extract title, category, and keywords from: ' || content}]
        }
    ) as metadata,
    -- Generate embeddings
    llm_embedding({'model_name': 'embedder'}, [{'data': content}]) as embedding,
    CURRENT_TIMESTAMP as ingested_at
FROM raw_documents;
```

**Metadata extraction:**
```sql
-- Extract structured metadata
CREATE PROMPT('metadata_extractor', 'Extract JSON metadata from this text. Include: title, category, keywords, summary. Text: {{text}}');

-- Use extracted metadata for better search
SELECT
    content,
    metadata->'category' as category,
    metadata->'keywords' as keywords,
    similarity
FROM (
    SELECT
        content,
        metadata,
        1 - array_distance(embedding, query_embedding) as similarity
    FROM document_collection, query_embedding
) results
WHERE similarity > 0.6
    AND metadata->>'category' = 'technical'
ORDER BY similarity DESC;
```

### 2. Context Assembly

**Intelligent context selection:**
```sql
-- Select most relevant context based on multiple factors
CREATE TEMP TABLE best_context AS
SELECT
    content,
    similarity,
    metadata->'recency_score' as recency_score,
    metadata->'authority_score' as authority_score,
    -- Combined relevance score
    similarity * 0.5 + recency_score * 0.3 + authority_score * 0.2 as relevance_score
FROM (
    SELECT
        content,
        metadata,
        1 - array_distance(embedding, query_embedding) as similarity
    FROM document_collection, query_embedding
) scored_results
WHERE similarity > 0.6
ORDER BY relevance_score DESC
LIMIT 8;
```

**Context optimization:**
```sql
-- Optimize context length and quality
SELECT
    -- Truncate very long documents
    CASE
        WHEN LENGTH(content) > 2000 THEN LEFT(content, 2000) || '...'
        ELSE content
    END as optimized_content,
    relevance_score
FROM best_context
ORDER BY relevance_score DESC;
```

### 3. Answer Generation

**Context-aware generation:**
```sql
-- Generate answer with full context
SELECT llm_complete(
    {'model_name': 'coder'},
    {
        'prompt_name': 'contextual_answer',
        'context_columns': [{
            'data': 'You are an expert assistant. Use ONLY the information provided in the context to answer the question. If the context doesn''t contain enough information, say so clearly.

Context: ' || (SELECT string_agg(optimized_content, ' ') FROM best_context) || '

Question: ' || 'What are the main applications of machine learning in healthcare?'

Provide a comprehensive, accurate answer based solely on the context above.'
        }]
    }
) as final_answer;
```

## Specialized RAG Applications

### Code Assistant RAG

**Programming help with code context:**
```sql
-- Search code examples for specific patterns
CREATE TEMP TABLE relevant_code AS
SELECT
    code_snippet,
    function_name,
    1 - array_distance(embedding, query_embedding) as similarity
FROM code_embeddings, (
    SELECT llm_embedding({'model_name': 'embedder'}, [{'data': 'error handling in async Rust'}]) as embedding
) qe
WHERE similarity > 0.7
ORDER BY similarity DESC
LIMIT 5;

-- Generate code with examples
SELECT llm_complete(
    {'model_name': 'coder'},
    {
        'prompt_name': 'code_with_examples',
        'context_columns': [{
            'data': 'Generate Rust code for async error handling. Use these examples as reference: ' ||
                   (SELECT string_agg(code_snippet, ' ') FROM relevant_code) ||
                   ' Create a robust async function that handles network errors properly.'
        }]
    }
) as generated_code;
```

### Research Assistant RAG

**Academic research assistance:**
```sql
-- Find relevant research papers
CREATE TEMP TABLE relevant_papers AS
SELECT
    title,
    abstract,
    authors,
    publication_year,
    1 - array_distance(embedding, query_embedding) as similarity
FROM research_papers, (
    SELECT llm_embedding({'model_name': 'embedder'}, [{'data': 'quantum computing algorithms'}]) as embedding
) qe
WHERE similarity > 0.75
ORDER BY similarity DESC, publication_year DESC
LIMIT 10;

-- Generate research summary
SELECT llm_complete(
    {'model_name': 'coder'},
    {
        'prompt_name': 'research_summary',
        'context_columns': [{
            'data': 'Summarize the key findings from these research papers on quantum computing algorithms. Organize by year and main contribution:

Papers: ' || (
                SELECT string_agg(
                    title || ' (' || publication_year || ') - ' || abstract,
                    ' | '
                ) FROM relevant_papers
            ) || '

Provide a comprehensive overview of recent developments in quantum computing algorithms.'
        }]
    }
) as research_summary;
```

### Technical Support RAG

**Troubleshooting assistance:**
```sql
-- Search troubleshooting guides
CREATE TEMP TABLE troubleshooting_context AS
SELECT
    problem_description,
    solution,
    category,
    1 - array_distance(embedding, query_embedding) as similarity
FROM troubleshooting_docs, (
    SELECT llm_embedding({'model_name': 'embedder'}, [{'data': 'database connection timeout'}]) as embedding
) qe
WHERE similarity > 0.7
ORDER BY similarity DESC
LIMIT 5;

-- Generate troubleshooting guide
SELECT llm_complete(
    {'model_name': 'coder'},
    {
        'prompt_name': 'troubleshooting_guide',
        'context_columns': [{
            'data': 'Create a step-by-step troubleshooting guide for database connection timeout issues. Use this information:

Problems and Solutions: ' || (
                SELECT string_agg(
                    'Problem: ' || problem_description || ' Solution: ' || solution,
                    ' '
                ) FROM troubleshooting_context
            ) || '

Structure the guide with clear steps and verification methods.'
        }]
    }
) as troubleshooting_guide;
```

## Performance Optimization

### 1. Indexing for RAG

**Efficient retrieval:**
```sql
-- Create RAG-optimized index
CREATE INDEX embedding_similarity_idx ON document_embeddings USING ivfflat (embedding vector_cosine_ops);

-- Fast similarity search
SELECT content
FROM document_embeddings
WHERE embedding <-> (SELECT embedding FROM query_embedding) < 0.3
ORDER BY embedding <-> (SELECT embedding FROM query_embedding)
LIMIT 10;
```

**Metadata indexing:**
```sql
-- Index by category for faster filtering
CREATE INDEX category_idx ON document_metadata (category);
CREATE INDEX tags_idx ON document_metadata USING gin (tags);

-- Combined search with metadata filtering
SELECT de.content
FROM document_embeddings de
JOIN document_metadata dm ON de.document_id = dm.document_id
WHERE dm.category = 'technical'
    AND de.embedding <-> query_embedding < 0.25
ORDER BY de.embedding <-> query_embedding
LIMIT 15;
```

### 2. Caching Strategy

**Multi-level caching:**
```sql
-- Cache embeddings
CREATE TABLE embedding_cache (
    content_hash VARCHAR PRIMARY KEY,
    embedding FLOAT[1024],
    cached_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Cache search results
CREATE TABLE search_result_cache (
    query_hash VARCHAR PRIMARY KEY,
    query_text TEXT,
    results JSON,
    cached_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Cache generated answers
CREATE TABLE answer_cache (
    query_hash VARCHAR PRIMARY KEY,
    context_hash VARCHAR,
    answer TEXT,
    cached_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

**Cache-aware RAG:**
```sql
-- Use caching to avoid redundant operations
CREATE OR REPLACE FUNCTION cached_rag_answer(question TEXT, context_docs TEXT)
RETURNS TEXT AS $$
DECLARE
    query_hash VARCHAR;
    context_hash VARCHAR;
    cached_answer TEXT;
BEGIN
    query_hash := hash(question);
    context_hash := hash(context_docs);

    -- Check if answer already cached
    SELECT answer INTO cached_answer
    FROM answer_cache
    WHERE query_hash = cached_answer.query_hash
        AND context_hash = cached_answer.context_hash
        AND cached_at > CURRENT_TIMESTAMP - INTERVAL '24 hours'
    LIMIT 1;

    IF cached_answer IS NOT NULL THEN
        RETURN cached_answer;
    END IF;

    -- Generate new answer and cache it
    cached_answer := llm_complete(
        {'model_name': 'coder'},
        {
            'prompt_name': 'contextual_answer',
            'context_columns': [{'data': 'Answer: ' || question || ' Context: ' || context_docs}]
        }
    );

    INSERT INTO answer_cache (query_hash, context_hash, answer)
    VALUES (query_hash, context_hash, cached_answer);

    RETURN cached_answer;
END;
$$ LANGUAGE plpgsql;
```

### 3. Batch Processing

**Efficient batch RAG:**
```python
import subprocess
import json

def batch_rag_processing(questions, knowledge_base):
    """Process multiple questions efficiently"""
    results = []

    # Process in batches of 10
    for i in range(0, len(questions), 10):
        batch = questions[i:i+10]

        # Generate embeddings for batch
        embeddings = []
        for question in batch:
            embedding_result = subprocess.run([
                "frozen-duckdb", "embed",
                "--text", question,
                "--model", "embedder"
            ], capture_output=True, text=True)

            if embedding_result.returncode == 0:
                embedding_data = json.loads(embedding_result.stdout)
                embeddings.append(embedding_data[0]["embedding"])

        # Search for each question in batch
        for j, question in enumerate(batch):
            search_result = subprocess.run([
                "frozen-duckdb", "search",
                "--query", question,
                "--corpus", knowledge_base,
                "--threshold", "0.7",
                "--limit", "5",
                "--format", "json"
            ], capture_output=True, text=True)

            if search_result.returncode == 0:
                context_docs = json.loads(search_result.stdout)
                context = " ".join([doc["document"] for doc in context_docs])

                # Generate answer
                answer_result = subprocess.run([
                    "frozen-duckdb", "complete",
                    "--prompt", f"Based on context: {context}\n\nAnswer: {question}"
                ], capture_output=True, text=True)

                if answer_result.returncode == 0:
                    results.append({
                        "question": question,
                        "answer": answer_result.stdout.strip(),
                        "context_docs": len(context_docs)
                    })

    return results

# Example usage
questions = [
    "How does machine learning work?",
    "What are neural networks?",
    "How is AI used in healthcare?"
]

results = batch_rag_processing(questions, "knowledge_base.txt")
for result in results:
    print(f"Q: {result['question']}")
    print(f"A: {result['answer'][:100]}...")
    print(f"Context docs: {result['context_docs']}\n")
```

## Integration Examples

### Chatbot RAG System

**Intelligent chatbot:**
```python
import subprocess
import json

class RAGChatbot:
    def __init__(self, knowledge_base):
        self.knowledge_base = knowledge_base
        self.conversation_history = []

    def ask(self, question):
        # Add to conversation history
        self.conversation_history.append({"role": "user", "content": question})

        # Search for relevant context
        search_result = subprocess.run([
            "frozen-duckdb", "search",
            "--query", question,
            "--corpus", self.knowledge_base,
            "--threshold", "0.7",
            "--limit", "5",
            "--format", "json"
        ], capture_output=True, text=True)

        if search_result.returncode == 0:
            context_docs = json.loads(search_result.stdout)

            if context_docs:
                context = " ".join([doc["document"] for doc in context_docs])

                # Generate answer with context and history
                full_prompt = f"""
Previous conversation:
{" ".join([msg["content"] for msg in self.conversation_history[-3:]])}

Relevant context:
{context}

Current question: {question}

Please provide a helpful, accurate response based on the context and conversation history.
"""

                answer_result = subprocess.run([
                    "frozen-duckdb", "complete",
                    "--prompt", full_prompt
                ], capture_output=True, text=True)

                if answer_result.returncode == 0:
                    answer = answer_result.stdout.strip()
                    self.conversation_history.append({"role": "assistant", "content": answer})
                    return answer

        return "I don't have enough information to answer that question accurately."

# Example usage
bot = RAGChatbot("company_knowledge.txt")
print(bot.ask("How do I reset my password?"))
print(bot.ask("What about two-factor authentication?"))
```

### Document Q&A System

**Intelligent document analysis:**
```rust
use duckdb::Connection;
use serde_json::{json, Value};

struct DocumentQASystem {
    conn: Connection,
}

impl DocumentQASystem {
    fn answer_document_question(&self, question: &str, document_path: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Extract text from document
        let document_text: String = self.conn.query_row(
            "SELECT content FROM documents WHERE file_path = ?",
            [document_path],
            |row| row.get(0),
        )?;

        // Generate embeddings for question and document
        let question_embedding: Vec<f32> = self.conn.query_row(
            "SELECT llm_embedding(?, ?)",
            [
                &json!({"model_name": "embedder"}).to_string(),
                &json!([{"data": question}]).to_string()
            ],
            |row| {
                let embedding_str: String = row.get(0)?;
                serde_json::from_str(&embedding_str).unwrap_or_default()
            },
        )?;

        // Split document into chunks for better context
        let chunks: Vec<String> = self.split_document_into_chunks(&document_text, 1000);

        // Find most relevant chunks
        let mut relevant_chunks = Vec::new();
        for chunk in chunks {
            let chunk_embedding: Vec<f32> = self.conn.query_row(
                "SELECT llm_embedding(?, ?)",
                [
                    &json!({"model_name": "embedder"}).to_string(),
                    &json!([{"data": &chunk}]).to_string()
                ],
                |row| {
                    let embedding_str: String = row.get(0)?;
                    serde_json::from_str(&embedding_str).unwrap_or_default()
                },
            )?;

            let similarity = self.cosine_similarity(&question_embedding, &chunk_embedding);
            if similarity > 0.7 {
                relevant_chunks.push(chunk);
            }
        }

        // Generate answer using relevant chunks
        let context = relevant_chunks.join(" ");
        let answer: String = self.conn.query_row(
            "SELECT llm_complete(?, ?)",
            [
                &json!({"model_name": "coder"}).to_string(),
                &json!([{
                    "data": format!("Based on this document context: {} Answer this question: {}", context, question)
                }]).to_string()
            ],
            |row| row.get(0),
        )?;

        Ok(answer)
    }

    fn split_document_into_chunks(&self, text: &str, chunk_size: usize) -> Vec<String> {
        text.chars()
            .collect::<Vec<_>>()
            .chunks(chunk_size)
            .map(|chunk| chunk.iter().collect())
            .collect()
    }

    fn cosine_similarity(&self, a: &[f32], b: &[f32]) -> f32 {
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if magnitude_a == 0.0 || magnitude_b == 0.0 {
            0.0
        } else {
            dot_product / (magnitude_a * magnitude_b)
        }
    }
}
```

### API Integration

**REST API for RAG:**
```python
from flask import Flask, request, jsonify
import subprocess
import json

app = Flask(__name__)

@app.route('/rag/answer', methods=['POST'])
def rag_answer():
    data = request.get_json()
    question = data.get('question', '')
    knowledge_base = data.get('knowledge_base', 'default.txt')
    threshold = data.get('threshold', 0.7)

    try:
        # Search for relevant context
        search_result = subprocess.run([
            "frozen-duckdb", "search",
            "--query", question,
            "--corpus", knowledge_base,
            "--threshold", str(threshold),
            "--limit", "5",
            "--format", "json"
        ], capture_output=True, text=True)

        if search_result.returncode == 0:
            context_docs = json.loads(search_result.stdout)

            if context_docs:
                context = " ".join([doc["document"] for doc in context_docs])

                # Generate answer
                answer_result = subprocess.run([
                    "frozen-duckdb", "complete",
                    "--prompt", f"Based on this context: {context}\n\nAnswer: {question}"
                ], capture_output=True, text=True)

                if answer_result.returncode == 0:
                    return jsonify({
                        "answer": answer_result.stdout.strip(),
                        "context_docs": len(context_docs),
                        "confidence": min(1.0, len(context_docs) * 0.2)
                    })

        return jsonify({"answer": "I don't have enough information to answer that question.", "context_docs": 0, "confidence": 0.0})

    except Exception as e:
        return jsonify({"error": str(e)}), 500

@app.route('/rag/search', methods=['POST'])
def rag_search():
    data = request.get_json()
    query = data.get('query', '')
    threshold = data.get('threshold', 0.7)

    try:
        result = subprocess.run([
            "frozen-duckdb", "search",
            "--query", query,
            "--corpus", "knowledge_base.txt",
            "--threshold", str(threshold),
            "--limit", "10",
            "--format", "json"
        ], capture_output=True, text=True)

        if result.returncode == 0:
            results = json.loads(result.stdout)
            return jsonify({"results": results})

        return jsonify({"results": [], "error": "Search failed"})

    except Exception as e:
        return jsonify({"error": str(e)}), 500

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)
```

## Performance Monitoring

### RAG Pipeline Metrics

**Performance tracking:**
```sql
-- Track RAG pipeline performance
CREATE TABLE rag_metrics (
    question TEXT,
    context_docs INTEGER,
    answer_length INTEGER,
    processing_time_ms INTEGER,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Log pipeline metrics
INSERT INTO rag_metrics (question, context_docs, answer_length, processing_time_ms)
SELECT
    'How does machine learning work?' as question,
    5 as context_docs,
    LENGTH(answer) as answer_length,
    2500 as processing_time_ms
FROM rag_answer;
```

**Quality assessment:**
```sql
-- Assess answer quality
SELECT
    question,
    answer_length,
    context_docs,
    -- Simple quality indicators
    CASE
        WHEN answer_length > 100 THEN 'detailed'
        WHEN answer_length > 50 THEN 'adequate'
        ELSE 'brief'
    END as quality_category,
    timestamp
FROM rag_metrics
ORDER BY timestamp DESC
LIMIT 20;
```

## Best Practices

### 1. Context Quality

**Ensure high-quality context:**
```sql
-- Filter for document quality
SELECT content
FROM document_embeddings de
JOIN document_metadata dm ON de.document_id = dm.document_id
WHERE dm.quality_score > 0.8
    AND LENGTH(content) > 100
    AND similarity > 0.7
ORDER BY similarity DESC
LIMIT 10;
```

**Context length optimization:**
```python
def optimize_context_length(context_docs, max_length=2000):
    """Optimize context length for LLM processing"""
    total_length = sum(len(doc["document"]) for doc in context_docs)

    if total_length <= max_length:
        return context_docs

    # Truncate documents proportionally
    optimized_docs = []
    remaining_length = max_length

    for doc in context_docs:
        doc_length = len(doc["document"])
        if doc_length <= remaining_length:
            optimized_docs.append(doc)
            remaining_length -= doc_length
        else:
            # Truncate this document
            truncated_content = doc["document"][:remaining_length] + "..."
            optimized_docs.append({
                "document": truncated_content,
                "similarity_score": doc["similarity_score"]
            })
            break

    return optimized_docs
```

### 2. Error Handling

**Robust RAG pipeline:**
```python
def robust_rag_answer(question, knowledge_base, max_retries=3):
    """Generate RAG answer with retry logic"""
    for attempt in range(max_retries):
        try:
            # Search for context
            context = search_relevant_documents(question, knowledge_base)

            if not context:
                return "I don't have enough information to answer that question."

            # Generate answer
            answer = generate_answer_with_context(question, context)

            # Validate answer quality
            if len(answer.strip()) < 50:
                raise ValueError("Answer too short")

            return answer

        except Exception as e:
            if attempt == max_retries - 1:
                return f"I encountered an error while processing your question: {str(e)}"
            print(f"RAG attempt {attempt + 1} failed: {e}")
            time.sleep(2 ** attempt)  # Exponential backoff

    return "Unable to generate answer after multiple attempts."
```

### 3. Performance Optimization

**Efficient context selection:**
```sql
-- Select optimal context size
WITH context_candidates AS (
    SELECT
        content,
        similarity,
        ROW_NUMBER() OVER (ORDER BY similarity DESC) as rank
    FROM (
        SELECT
            content,
            1 - array_distance(embedding, query_embedding) as similarity
        FROM document_embeddings, query_embedding
    ) scored
    WHERE similarity > 0.6
)
SELECT
    content,
    similarity,
    -- Cumulative context length
    SUM(LENGTH(content)) OVER (ORDER BY rank) as cumulative_length
FROM context_candidates
WHERE SUM(LENGTH(content)) OVER (ORDER BY rank) <= 3000  -- Max 3000 chars
ORDER BY similarity DESC;
```

## Summary

RAG pipelines with Frozen DuckDB provide **intelligent question answering** by combining **information retrieval** with **context-aware text generation**. The system supports **multiple RAG patterns**, **performance optimization**, and **integration with various applications**.

**Key RAG Capabilities:**
- **Context retrieval**: Find relevant information using semantic search
- **Answer generation**: Create comprehensive answers based on retrieved context
- **Multi-step processing**: Iterative refinement and filtering
- **Performance optimization**: Caching, indexing, and batch processing

**RAG Patterns:**
- **Basic RAG**: Simple retrieval and generation
- **Advanced RAG**: Multi-step processing with filtering and ranking
- **Hierarchical RAG**: Category-based context selection
- **Specialized RAG**: Domain-specific applications (code, research, support)

**Performance Characteristics:**
- **End-to-end time**: 2-8 seconds for typical questions
- **Context quality**: High relevance with proper similarity thresholds
- **Scalability**: Handles large knowledge bases efficiently
- **Reliability**: Robust error handling and fallback strategies

**Use Cases:**
- **Chatbots and assistants**: Intelligent conversational interfaces
- **Document Q&A**: Answer questions about large document collections
- **Code assistance**: Programming help with code context
- **Research tools**: Academic and technical research assistance
- **Support systems**: Intelligent troubleshooting and guidance
