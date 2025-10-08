//! Example demonstrating Flock integration with Ollama using custom models
//!
//! This example shows how to use frozen-duckdb with Flock extension to:
//! 1. Setup Ollama with custom models
//! 2. Generate text completions using specific models
//! 3. Generate embeddings using custom embedding models
//! 4. Perform semantic search and filtering
//! 5. Generate summaries using different strategies

use frozen_duckdb::Connection;
use frozen_duckdb::cli::flock_manager::FlockManager;
use anyhow::Result;

fn main() -> Result<()> {
    println!("🦆 Frozen DuckDB - Flock Ollama Integration Example");
    println!("==================================================");
    
    // Create Flock manager
    let flock_manager = FlockManager::new()?;
    
    // Check if Flock extension is available
    if !flock_manager.is_flock_ready()? {
        println!("❌ Flock extension not available");
        println!("   Please install the Flock extension first");
        println!("   Run: frozen-duckdb flock-setup");
        return Ok(());
    }
    
    println!("✅ Flock extension is ready!");
    
    // Setup Ollama with custom models
    println!("\n🔧 Setting up Ollama integration...");
    let ollama_url = "http://localhost:11434";
    let text_model = "llama3.2";  // Custom text generation model
    let embedding_model = "mxbai-embed-large";  // Custom embedding model
    
    match flock_manager.setup_ollama(ollama_url, text_model, embedding_model, true) {
        Ok(_) => println!("✅ Ollama setup completed"),
        Err(e) => {
            println!("⚠️  Ollama setup failed: {}", e);
            println!("   Make sure Ollama is running on {}", ollama_url);
            println!("   And that models '{}' and '{}' are available", text_model, embedding_model);
            return Ok(());
        }
    }
    
    // Test 1: Text Completion with Custom Model
    println!("\n🤖 Test 1: Text Completion with Custom Model");
    println!("Model: {}", text_model);
    
    let prompt = "Write a short poem about databases";
    match flock_manager.complete_text(prompt, "text_generator") {
        Ok(response) => {
            println!("✅ Completion generated:");
            println!("   Prompt: {}", prompt);
            println!("   Response: {}", response);
        },
        Err(e) => {
            println!("❌ Text completion failed: {}", e);
            println!("   Make sure Ollama is running and model '{}' is available", text_model);
        }
    }
    
    // Test 2: Embedding Generation with Custom Model
    println!("\n🧠 Test 2: Embedding Generation with Custom Model");
    println!("Model: {}", embedding_model);
    
    let texts = vec![
        "Rust is a systems programming language".to_string(),
        "Python is great for data science".to_string(),
        "Databases store and retrieve data efficiently".to_string(),
    ];
    
    match flock_manager.generate_embeddings(texts.clone(), "embedder", true) {
        Ok(embeddings) => {
            println!("✅ Generated {} embeddings", embeddings.len());
            for (i, embedding) in embeddings.iter().enumerate() {
                println!("   Text {}: {} dimensions", i + 1, embedding.len());
            }
        },
        Err(e) => {
            println!("❌ Embedding generation failed: {}", e);
            println!("   Make sure model '{}' is available in Ollama", embedding_model);
        }
    }
    
    // Test 3: Text Filtering with Custom Model
    println!("\n🎯 Test 3: Text Filtering with Custom Model");
    
    // Create a temporary file with test data
    let test_file = "temp_filter_test.txt";
    std::fs::write(test_file, "Rust is a programming language\nPython is easy to learn\nDatabases are important\nCooking is fun\nMachine learning is powerful")?;
    
    let criteria = "Is this text about programming or technology?";
    match flock_manager.llm_filter(criteria, test_file, "text_generator", true) {
        Ok(results) => {
            println!("✅ Filtered {} items with criteria: {}", results.len(), criteria);
            for (text, matches) in results {
                println!("   {}: {}", if matches { "✅" } else { "❌" }, text);
            }
        },
        Err(e) => {
            println!("❌ Text filtering failed: {}", e);
        }
    }
    
    // Clean up test file
    let _ = std::fs::remove_file(test_file);
    
    // Test 4: Text Summarization with Custom Model
    println!("\n📝 Test 4: Text Summarization with Custom Model");
    
    let texts_to_summarize = vec![
        "Rust is a systems programming language that runs blazingly fast, prevents segfaults, and guarantees thread safety. It has zero-cost abstractions and memory safety without garbage collection.".to_string(),
        "Python is a high-level programming language known for its simplicity and readability. It's widely used in data science, web development, and automation.".to_string(),
        "Databases are essential for storing and retrieving data efficiently. They provide ACID properties and support complex queries for data analysis.".to_string(),
    ];
    
    match flock_manager.summarize_texts(texts_to_summarize, "reduce", 50, "text_generator") {
        Ok(summary) => {
            println!("✅ Generated summary ({} chars):", summary.len());
            println!("   {}", summary);
        },
        Err(e) => {
            println!("❌ Text summarization failed: {}", e);
        }
    }
    
    // Test 5: Direct DuckDB Flock Usage
    println!("\n🦆 Test 5: Direct DuckDB Flock Usage");
    
    let conn = Connection::open_in_memory()?;
    
    // Test if we can use Flock functions directly
    match conn.query_row(
        "SELECT llm_complete({'model_name': 'text_generator'}, {'prompt': 'What is Rust?'})",
        [],
        |row| row.get::<_, String>(0)
    ) {
        Ok(response) => {
            println!("✅ Direct Flock usage successful:");
            println!("   Response: {}", response);
        },
        Err(e) => {
            println!("❌ Direct Flock usage failed: {}", e);
        }
    }
    
    println!("\n🎉 Flock Ollama Integration Example Completed!");
    println!("💡 This demonstrates how frozen-duckdb enables:");
    println!("   - Custom model configuration for Ollama");
    println!("   - Text completion with specific models");
    println!("   - Embedding generation with custom models");
    println!("   - Intelligent text filtering and classification");
    println!("   - Text summarization with different strategies");
    println!("   - Direct Flock function usage in SQL");
    
    Ok(())
}
