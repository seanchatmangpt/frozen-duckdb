# Text Completion Guide

## Overview

**Text completion** is the core LLM capability that enables **code generation**, **text writing**, **question answering**, and **content creation** directly within DuckDB using the **Flock extension** and **Ollama models**.

## Basic Text Completion

### Simple Completion

**Using CLI:**
```bash
# Generate text completion
frozen-duckdb complete --prompt "Explain recursion in programming"

# Expected output:
# Recursion in programming is a technique where a function calls itself to solve a problem...
```

**Using SQL:**
```sql
-- Basic text completion
SELECT llm_complete(
    {'model_name': 'coder'},
    {'prompt_name': 'complete', 'context_columns': [{'data': 'Explain recursion in programming'}]}
) as explanation;
```

### File-Based Completion

**Input from file:**
```bash
# Read prompt from file
echo "Write a function to calculate fibonacci numbers in Rust" > prompt.txt
frozen-duckdb complete --input prompt.txt --output response.txt

# Interactive mode
echo "Explain quantum computing" | frozen-duckdb complete
```

**Output to file:**
```bash
# Save response to file
frozen-duckdb complete --prompt "Write a Python web server" --output server_code.py

# Append to existing file
frozen-duckdb complete --prompt "Add error handling" --output server_code.py
```

## Code Generation

### Function Generation

**Basic function:**
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

**Advanced function:**
```bash
frozen-duckdb complete --prompt "Write a Python function to merge two sorted lists"

# Output:
# def merge_sorted_lists(list1, list2):
#     result = []
#     i = j = 0
#     while i < len(list1) and j < len(list2):
#         if list1[i] < list2[j]:
#             result.append(list1[i])
#             i += 1
#         else:
#             result.append(list2[j])
#             j += 1
#     result.extend(list1[i:])
#     result.extend(list2[j:])
#     return result
```

### Class and Module Generation

**Class generation:**
```bash
frozen-duckdb complete --prompt "Write a Python class for a bank account"

# Output:
# class BankAccount:
#     def __init__(self, account_number, balance=0):
#         self.account_number = account_number
#         self.balance = balance
#
#     def deposit(self, amount):
#         if amount > 0:
#             self.balance += amount
#             return True
#         return False
#
#     def withdraw(self, amount):
#         if 0 < amount <= self.balance:
#             self.balance -= amount
#             return True
#         return False
#
#     def get_balance(self):
#         return self.balance
```

**Module generation:**
```bash
frozen-duckdb complete --prompt "Write a Python module for data validation"

# Output:
# """
# Data validation utilities
# """
# import re
# from typing import List, Optional
#
# def validate_email(email: str) -> bool:
#     pattern = r'^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$'
#     return re.match(pattern, email) is not None
#
# def validate_phone(phone: str) -> bool:
#     pattern = r'^\+?1?\d{9,15}$'
#     return re.match(pattern, phone) is not None
#
# def validate_credit_card(card_number: str) -> bool:
#     # Remove spaces and dashes
#     card = re.sub(r'[\s-]', '', card_number)
#     if not card.isdigit() or len(card) < 13 or len(card) > 19:
#         return False
#     # Luhn algorithm
#     digits = [int(d) for d in reversed(card)]
#     for i in range(1, len(digits), 2):
#         digits[i] *= 2
#         if digits[i] > 9:
#             digits[i] -= 9
#     return sum(digits) % 10 == 0
```

## Content Creation

### Article Writing

**Blog post generation:**
```bash
frozen-duckdb complete --prompt "Write a blog post about machine learning in healthcare"

# Output:
# # Machine Learning in Healthcare: Transforming Patient Care
#
# ## Introduction
# Machine learning is revolutionizing healthcare by enabling more accurate diagnoses,
# personalized treatments, and improved patient outcomes...
#
# ## Diagnostic Applications
# ### Medical Imaging
# Machine learning algorithms can analyze medical images with remarkable accuracy...
#
# ## Treatment Personalization
# ### Predictive Analytics
# By analyzing patient data, ML models can predict treatment responses...
#
# ## Challenges and Considerations
# ### Data Privacy
# Healthcare data is highly sensitive and must be protected...
#
# ## Conclusion
# Machine learning holds immense potential for improving healthcare delivery...
```

**Technical documentation:**
```bash
frozen-duckdb complete --prompt "Write documentation for a REST API endpoint"

# Output:
# ## User Profile API
#
# ### GET /api/users/{id}
# Retrieves user profile information.
#
# **Parameters:**
# - `id` (path): User ID (integer)
#
# **Response:**
# ```json
# {
#   "id": 123,
#   "username": "john_doe",
#   "email": "john@example.com",
#   "profile": {
#     "first_name": "John",
#     "last_name": "Doe",
#     "bio": "Software developer",
#     "avatar_url": "https://example.com/avatar.jpg"
#   },
#   "created_at": "2024-01-15T10:30:00Z"
# }
# ```
#
# **Error Responses:**
# - `404`: User not found
# - `401`: Unauthorized access
```

### Creative Writing

**Story generation:**
```bash
frozen-duckdb complete --prompt "Write a short story about a robot learning to paint"

# Output:
# In the quiet corners of the artist's studio, where canvases whispered secrets of color and form,
# there lived a robot named Palette. Unlike his mechanical brethren who calculated trajectories and
# optimized algorithms, Palette dreamed in hues and shades...
```

**Poetry generation:**
```bash
frozen-duckdb complete --prompt "Write a haiku about autumn"

# Output:
# Golden leaves descend
# Whispers of wind through branches
# Autumn's quiet song
```

## Question Answering

### Technical Q&A

**Programming questions:**
```bash
frozen-duckdb complete --prompt "How do I handle errors in async Rust code?"

# Output:
# In async Rust, error handling follows similar patterns to synchronous code but with some
# important differences. Here are the key approaches:
#
# ## Using Result<T, E>
# ```rust
# async fn fetch_data() -> Result<String, Box<dyn std::error::Error>> {
#     // ... async operation that might fail
# }
# ```
#
# ## Error Propagation
# ```rust
# async fn process_data() -> Result<(), Box<dyn std::error::Error>> {
#     let data = fetch_data().await?;
#     // ... process data
#     Ok(())
# }
# ```
#
# ## Custom Error Types
# ```rust
# #[derive(Debug)]
# enum ApiError {
#     Network(reqwest::Error),
#     Parse(serde_json::Error),
#     Validation(String),
# }
# ```
#
# ## Best Practices
# - Use `?` operator for error propagation
# - Implement `From` traits for easy conversion
# - Consider using `anyhow` or `thiserror` for flexible error handling
# - Log errors appropriately for debugging
```

**Database questions:**
```bash
frozen-duckdb complete --prompt "How do I optimize a slow SQL query?"

# Output:
# Optimizing slow SQL queries involves several strategies:
#
# ## 1. Analyze Query Execution Plan
# ```sql
# EXPLAIN ANALYZE SELECT * FROM large_table WHERE condition;
# ```
#
# ## 2. Create Appropriate Indexes
# ```sql
# CREATE INDEX idx_condition ON large_table(condition_column);
# CREATE INDEX idx_composite ON large_table(col1, col2);
# ```
#
# ## 3. Query Optimization Techniques
# - Use SELECT only needed columns instead of SELECT *
# - Avoid functions in WHERE clauses when possible
# - Use JOIN instead of subqueries when appropriate
# - Consider partitioning for very large tables
#
# ## 4. Database Configuration
# - Adjust memory settings: `SET work_mem = '256MB'`
# - Enable query result caching when appropriate
# - Monitor slow query logs for patterns
#
# ## 5. Application-Level Optimization
# - Implement query result caching
# - Use prepared statements for repeated queries
# - Batch multiple simple queries when possible
```

### Educational Content

**Concept explanation:**
```bash
frozen-duckdb complete --prompt "Explain quantum entanglement for beginners"

# Output:
# Imagine two coins that are quantum-entangled. When you flip one coin and get heads,
# the other coin instantly shows tails, no matter how far apart they are. This isn't
# just a fast communication - it's as if the coins are connected in a way that
# transcends space and time...
```

**Tutorial creation:**
```bash
frozen-duckdb complete --prompt "Create a beginner's tutorial for Docker"

# Output:
# # Docker Tutorial for Beginners
#
# ## What is Docker?
# Docker is a platform that allows you to package applications and their dependencies
# into lightweight, portable containers...
#
# ## Installation
# ### Linux
# ```bash
# curl -fsSL https://get.docker.com -o get-docker.sh
# sudo sh get-docker.sh
# ```
#
# ### macOS
# ```bash
# brew install --cask docker
# ```
#
# ## Your First Container
# ```bash
# docker run hello-world
# ```
#
# ## Building Custom Images
# ```dockerfile
# FROM node:14
# WORKDIR /app
# COPY package*.json ./
# RUN npm install
# COPY . .
# EXPOSE 3000
# CMD ["npm", "start"]
# ```
#
# ## Common Commands
# - `docker build -t my-app .` - Build an image
# - `docker run -p 3000:3000 my-app` - Run a container
# - `docker ps` - List running containers
# - `docker images` - List available images
#
# ## Best Practices
# - Use multi-stage builds to reduce image size
# - Pin specific versions in your Dockerfile
# - Use .dockerignore to exclude unnecessary files
# - Scan images for vulnerabilities regularly
```

## Advanced Completion Patterns

### Multi-step Generation

**Iterative improvement:**
```bash
# Generate initial code
frozen-duckdb complete --prompt "Write a basic calculator in Python" --output calculator_v1.py

# Improve with error handling
frozen-duckdb complete --prompt "Add error handling to this calculator code" --input calculator_v1.py --output calculator_v2.py

# Add features
frozen-duckdb complete --prompt "Add scientific calculator functions" --input calculator_v2.py --output calculator_v3.py
```

### Context-Aware Generation

**Using conversation history:**
```bash
# Build context through multiple completions
echo "I'm learning Python and want to understand classes" > context.txt

frozen-duckdb complete --input context.txt --prompt "Explain Python classes with examples" --output explanation.txt

frozen-duckdb complete --input explanation.txt --prompt "Show me how to create a class for a bank account" --output bank_account.py
```

### Template-Based Generation

**Custom prompts for specific tasks:**
```sql
-- Create specialized prompts
CREATE PROMPT('api_docs', 'Generate API documentation for this endpoint: {{endpoint}}');

CREATE PROMPT('test_cases', 'Generate test cases for this function: {{function}}');

CREATE PROMPT('code_review', 'Review this code for bugs and improvements: {{code}}');

-- Use with parameters
SELECT llm_complete(
    {'model_name': 'coder'},
    {
        'prompt_name': 'api_docs',
        'context_columns': [{'endpoint': 'GET /api/users/{id}'}]
    }
);
```

## Performance Optimization

### Model Selection

**Quality vs Speed Trade-off:**
```bash
# Fast responses (7B model)
frozen-duckdb complete --prompt "Quick explanation" --model fast_coder

# High quality (30B model)
frozen-duckdb complete --prompt "Detailed technical explanation" --model quality_coder
```

**Model configuration:**
```sql
-- Create models for different use cases
CREATE MODEL('fast_coder', 'qwen3-coder:7b', 'ollama');
CREATE MODEL('quality_coder', 'qwen3-coder:30b', 'ollama');

-- Use based on requirements
SELECT CASE
    WHEN prompt_length < 100 THEN 'fast_coder'
    ELSE 'quality_coder'
END as selected_model;
```

### Batch Processing

**Multiple completions:**
```bash
#!/bin/bash
# batch_completions.sh

PROMPTS=("Explain variables" "Explain functions" "Explain classes")
OUTPUT_DIR="python_tutorials"

for i in "${!PROMPTS[@]}"; do
    prompt="${PROMPTS[$i]}"
    output_file="$OUTPUT_DIR/tutorial_$((i+1)).txt"

    frozen-duckdb complete --prompt "$prompt in Python" --output "$output_file"
    echo "Generated: $output_file"
done
```

**Large document processing:**
```python
import subprocess

def process_document_chunks(file_path, chunk_size=1000):
    """Process large document in chunks"""
    with open(file_path, 'r') as f:
        content = f.read()

    # Split into chunks
    chunks = [content[i:i+chunk_size] for i in range(0, len(content), chunk_size)]

    results = []
    for i, chunk in enumerate(chunks):
        # Generate completion for each chunk
        result = subprocess.run([
            "frozen-duckdb", "complete",
            "--prompt", f"Summarize this text chunk: {chunk[:200]}..."
        ], capture_output=True, text=True)

        if result.returncode == 0:
            results.append(result.stdout.strip())

    return results

# Example usage
summaries = process_document_chunks("large_document.txt")
```

## Error Handling and Quality Assurance

### Input Validation

**Check prompt quality:**
```python
def validate_prompt(prompt):
    """Validate prompt before sending to LLM"""
    if not prompt or len(prompt.strip()) < 10:
        raise ValueError("Prompt too short or empty")

    if len(prompt) > 10000:
        raise ValueError("Prompt too long")

    return prompt.strip()

# Example usage
try:
    validated_prompt = validate_prompt(user_input)
    response = llm_complete(validated_prompt)
except ValueError as e:
    print(f"Invalid prompt: {e}")
```

### Output Validation

**Check response quality:**
```python
def validate_response(response, expected_format=None):
    """Validate LLM response quality"""
    if not response or len(response.strip()) < 20:
        raise ValueError("Response too short")

    if expected_format == "code" and not any(keyword in response.lower() for keyword in ["def", "function", "class"]):
        raise ValueError("Response doesn't appear to contain code")

    return response

# Example usage
response = llm_complete("Write a Python function")
validated_response = validate_response(response, expected_format="code")
```

### Retry Logic

**Handle transient failures:**
```python
import time

def robust_completion(prompt, max_retries=3):
    """Generate completion with retry logic"""
    for attempt in range(max_retries):
        try:
            response = llm_complete(prompt)
            return validate_response(response)
        except Exception as e:
            if attempt == max_retries - 1:
                raise e
            print(f"Attempt {attempt + 1} failed: {e}")
            time.sleep(2 ** attempt)  # Exponential backoff

    raise Exception("All retry attempts failed")

# Example usage
response = robust_completion("Write a complex algorithm")
```

## Integration Examples

### Rust Integration

```rust
use std::process::Command;

fn generate_documentation(function_name: &str) -> Result<String, Box<dyn std::error::Error>> {
    let prompt = format!("Generate comprehensive documentation for the {} function", function_name);

    let output = Command::new("frozen-duckdb")
        .args(&["complete", "--prompt", &prompt])
        .output()?;

    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?)
    } else {
        Err(format!("Documentation generation failed: {}",
                   String::from_utf8(output.stderr)?).into())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let docs = generate_documentation("process_data")?;
    println!("Generated documentation:\n{}", docs);
    Ok(())
}
```

### Python Integration

```python
import subprocess
import json

class LLMClient:
    def __init__(self):
        self.model = "coder"

    def complete(self, prompt, output_file=None):
        """Generate text completion"""
        cmd = ["frozen-duckdb", "complete", "--prompt", prompt, "--model", self.model]

        if output_file:
            cmd.extend(["--output", output_file])

        result = subprocess.run(cmd, capture_output=True, text=True)

        if result.returncode == 0:
            return result.stdout.strip()
        else:
            raise Exception(f"LLM completion failed: {result.stderr}")

    def generate_code(self, description, language="python"):
        """Generate code for given description"""
        prompt = f"Write {language} code for: {description}"
        return self.complete(prompt)

    def explain_code(self, code):
        """Generate explanation for code"""
        prompt = f"Explain this code in detail: {code}"
        return self.complete(prompt)

# Example usage
client = LLMClient()
code = client.generate_code("a function to calculate fibonacci numbers")
explanation = client.explain_code(code)

print("Generated code:", code)
print("Explanation:", explanation)
```

### Shell Script Integration

```bash
#!/bin/bash
# code_generator.sh

if [[ -z "$1" ]]; then
    echo "Usage: $0 <description>"
    echo "Example: $0 'a web server in Python'"
    exit 1
fi

DESCRIPTION="$1"
OUTPUT_DIR="generated_code"

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Generate code
echo "🤖 Generating code for: $DESCRIPTION"
frozen-duckdb complete --prompt "Write complete, runnable code for: $DESCRIPTION" --output "$OUTPUT_DIR/$(echo "$DESCRIPTION" | tr ' ' '_').py"

# Generate tests
echo "🧪 Generating tests..."
frozen-duckdb complete --prompt "Write comprehensive tests for this code" --input "$OUTPUT_DIR/$(echo "$DESCRIPTION" | tr ' ' '_').py" --output "$OUTPUT_DIR/test_$(echo "$DESCRIPTION" | tr ' ' '_').py"

# Generate documentation
echo "📚 Generating documentation..."
frozen-duckdb complete --prompt "Write README documentation for this code" --input "$OUTPUT_DIR/$(echo "$DESCRIPTION" | tr ' ' '_').py" --output "$OUTPUT_DIR/README.md"

echo "✅ Code generation complete!"
echo "Generated files in: $OUTPUT_DIR/"
ls -la "$OUTPUT_DIR/"
```

## Best Practices

### 1. Prompt Engineering

**Clear and specific:**
```bash
# Good prompt
frozen-duckdb complete --prompt "Write a Python function that takes a list of numbers and returns the sum of all even numbers"

# Bad prompt
frozen-duckdb complete --prompt "Write code"
```

**Include context:**
```bash
# Good prompt with context
frozen-duckdb complete --prompt "Write a Rust function for a web server that handles GET requests. Include error handling and JSON responses."

# Bad prompt without context
frozen-duckdb complete --prompt "Write a function"
```

### 2. Quality Assurance

**Validate outputs:**
```python
def validate_code_response(response, language="python"):
    """Validate that response contains valid code"""
    if language == "python":
        # Check for Python syntax indicators
        return any(indicator in response for indicator in ["def ", "class ", "import ", "from "])
    elif language == "rust":
        return any(indicator in response for indicator in ["fn ", "struct ", "impl ", "use "])
    return True

# Example usage
response = llm_complete("Write Python code")
if validate_code_response(response, "python"):
    print("✅ Response appears to contain valid code")
else:
    print("❌ Response may not contain expected code")
```

### 3. Performance Optimization

**Batch processing:**
```bash
# Process multiple prompts efficiently
cat > prompts.txt << 'EOF'
Explain variables in Python
Explain functions in Python
Explain classes in Python
EOF

# Process all at once (more efficient)
frozen-duckdb complete --input prompts.txt --output responses.txt
```

**Caching results:**
```python
import hashlib
import json

def get_cached_completion(prompt):
    """Get completion with caching"""
    prompt_hash = hashlib.md5(prompt.encode()).hexdigest()
    cache_file = f"cache/{prompt_hash}.json"

    # Check cache first
    try:
        with open(cache_file, 'r') as f:
            cached = json.load(f)
            return cached['response']
    except FileNotFoundError:
        pass

    # Generate new completion
    response = llm_complete(prompt)

    # Cache result
    os.makedirs("cache", exist_ok=True)
    with open(cache_file, 'w') as f:
        json.dump({'prompt': prompt, 'response': response}, f)

    return response
```

## Troubleshooting

### Common Issues

#### 1. Poor Quality Responses

**Problem:** Generated text is irrelevant or low quality

**Solutions:**
```bash
# Improve prompt clarity
frozen-duckdb complete --prompt "Write a detailed explanation of recursion in programming with examples"

# Use better model
frozen-duckdb complete --prompt "Explain recursion" --model quality_coder

# Add more context
frozen-duckdb complete --prompt "As a senior developer, explain recursion to a junior developer with concrete examples"
```

#### 2. Incomplete Responses

**Problem:** Responses cut off or incomplete

**Solutions:**
```bash
# Check for length limits
frozen-duckdb complete --prompt "Write a comprehensive guide" --model quality_coder

# Use streaming if available (future feature)
# frozen-duckdb complete --prompt "Long explanation" --stream
```

#### 3. Inconsistent Formatting

**Problem:** Code formatting issues

**Solutions:**
```bash
# Specify formatting requirements
frozen-duckdb complete --prompt "Write Python code with proper indentation and comments"

# Post-process formatting
frozen-duckdb complete --prompt "Format this code properly" --input unformatted_code.txt
```

## Summary

Text completion with Frozen DuckDB provides **powerful code generation**, **content creation**, and **question answering** capabilities with **local inference** for **privacy and performance**. The system supports **multiple programming languages**, **various content types**, and **advanced prompting techniques**.

**Key Capabilities:**
- **Code generation**: Functions, classes, modules in multiple languages
- **Content creation**: Articles, documentation, tutorials, creative writing
- **Question answering**: Technical explanations, concept clarification
- **Educational content**: Tutorials, explanations, learning materials

**Performance Characteristics:**
- **Response time**: 2-5 seconds for typical requests
- **Quality**: Excellent with qwen3-coder:30b model
- **Privacy**: Complete local processing
- **Reliability**: Consistent results with proper prompting

**Integration Options:**
- **CLI interface**: Direct command-line operations
- **Rust/Python/Node.js**: Native integration with popular languages
- **SQL queries**: Direct LLM operations within DuckDB
- **Batch processing**: Efficient handling of multiple requests

**Best Practices:**
- **Clear prompting**: Specific, contextual prompts for better results
- **Quality validation**: Check outputs for relevance and accuracy
- **Performance optimization**: Use appropriate models and batch processing
- **Error handling**: Implement retry logic and fallback strategies
