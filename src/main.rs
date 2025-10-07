//! # Frozen DuckDB CLI Tool
//!
//! A comprehensive command-line interface for managing datasets and frozen DuckDB operations.
//! This tool provides utilities for dataset management, format conversion, performance
//! benchmarking, system information display, and LLM operations via Flock extension.
//!
//! ## Features
//!
//! - **📥 Dataset Management**: Download and generate sample datasets (Chinook, TPC-H)
//! - **🔄 Format Conversion**: Convert between CSV, Parquet, Arrow, and DuckDB formats
//! - **⚡ Performance Benchmarking**: Measure and compare operation performance
//! - **ℹ️ System Information**: Display frozen DuckDB configuration and capabilities
//! - **🧪 Testing Support**: Run comprehensive test suites
//! - **🤖 LLM Operations**: Text completion, embeddings, semantic search via Flock
//!
//! ## Usage
//!
//! ```bash
//! # Show help
//! frozen-duckdb --help
//!
//! # Download Chinook dataset in CSV format
//! frozen-duckdb download --dataset chinook --format csv
//!
//! # Generate TPC-H dataset in Parquet format
//! frozen-duckdb download --dataset tpch --format parquet
//!
//! # Convert CSV to Parquet
//! frozen-duckdb convert --input data.csv --output data.parquet --input-format csv --output-format parquet
//!
//! # Show system information
//! frozen-duckdb info
//!
//! # Setup Ollama for LLM operations
//! frozen-duckdb flock-setup
//!
//! # Generate text completion
//! frozen-duckdb complete --prompt "Explain recursion in programming"
//!
//! # Generate embeddings for semantic search
//! frozen-duckdb embed --text "Python programming language"
//!
//! # Perform semantic search
//! frozen-duckdb search --query "machine learning" --corpus documents.txt
//! ```
//!
//! ## Environment Setup
//!
//! Before using the CLI, ensure the frozen DuckDB environment is configured:
//!
//! ```bash
//! # Set up environment (required)
//! source prebuilt/setup_env.sh
//!
//! # Verify configuration
//! frozen-duckdb info
//! ```
//!
//! ## Performance Targets
//!
//! The CLI is designed to meet strict performance requirements:
//!
//! - **Startup time**: <100ms
//! - **Dataset generation**: <10s for small datasets
//! - **Format conversion**: <1s for typical files
//! - **Memory usage**: <100MB for most operations
//! - **LLM operations**: <5s for typical requests
//!
//! ## Error Handling
//!
//! The CLI provides clear error messages and exit codes:
//!
//! - **Exit code 0**: Success
//! - **Exit code 1**: General error (invalid arguments, file not found)
//! - **Exit code 2**: Environment not configured
//! - **Exit code 3**: Binary validation failed
//! - **Exit code 4**: Flock extension not available

use anyhow::{Context, Result};
use clap::Parser;
use frozen_duckdb::cli::commands::{Cli, Commands};
use frozen_duckdb::cli::dataset_manager::DatasetManager;
use frozen_duckdb::cli::flock_manager::FlockManager;
use serde_json::{self, Value};
use std::io;
use std::path::Path;
use tracing::{error, info};


fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing based on verbosity
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(match cli.verbose {
            0 => tracing::Level::WARN,
            1 => tracing::Level::INFO,
            2 => tracing::Level::DEBUG,
            _ => tracing::Level::TRACE,
        })
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");

    match cli.command {
        // === DATASET MANAGEMENT COMMANDS ===
        Commands::Download {
            dataset,
            output_dir,
            format,
        } => {
            let dataset_manager = DatasetManager::new()?;
            match dataset.as_str() {
                "chinook" => {
                    dataset_manager.download_chinook(&output_dir, &format)?;
                }
                "tpch" => {
                    dataset_manager.download_tpch(&output_dir, &format)?;
                }
                _ => {
                    error!("❌ Unknown dataset: {}", dataset);
                    error!("   Available datasets: chinook, tpch");
                    std::process::exit(1);
                }
            }
        }

        Commands::Convert {
            input,
            output,
            input_format,
            output_format,
        } => {
            let dataset_manager = DatasetManager::new()?;
            dataset_manager.convert_dataset(&input, &output, &input_format, &output_format)?;
        }

        Commands::Info => {
            let dataset_manager = DatasetManager::new()?;
            dataset_manager.show_info()?;
        }

        // === FLOCK/LLM COMMANDS ===
        Commands::FlockSetup {
            ollama_url,
            text_model,
            embedding_model,
            skip_verification,
        } => {
            let flock_manager = FlockManager::new()?;

            // Check if Flock is ready before proceeding
            if !flock_manager.is_flock_ready()? {
                error!("❌ Flock extension not available");
                error!("   Make sure DuckDB with Flock extension is properly installed");
                std::process::exit(4);
            }

            flock_manager.setup_ollama(&ollama_url, &text_model, &embedding_model, skip_verification)?;
        }

        Commands::Complete {
            prompt,
            input,
            output,
            model,
            max_tokens: _,
            temperature: _,
        } => {
            let flock_manager = FlockManager::new()?;

            // Check if Flock is ready
            if !flock_manager.is_flock_ready()? {
                error!("❌ Flock extension not available");
                error!("   Run 'frozen-duckdb flock-setup' first");
                std::process::exit(4);
            }

            let text_to_complete = if let Some(prompt_text) = prompt {
                prompt_text
            } else if let Some(input_file) = input {
                // Read from input file
                match std::fs::read_to_string(&input_file) {
                    Ok(content) => content.trim().to_string(),
                    Err(e) => {
                        error!("❌ Failed to read input file '{}': {}", input_file, e);
                        std::process::exit(1);
                    }
                }
            } else {
                // Read from stdin
                info!("📝 Enter text to complete (Ctrl+D to finish):");
                let mut buffer = String::new();
                io::stdin().read_line(&mut buffer)?;
                buffer.trim().to_string()
            };

            let response = flock_manager.complete_text(&text_to_complete, model.as_str())
                .unwrap_or_else(|_| {
                    error!("❌ Text completion failed - check if Ollama is running");
                    std::process::exit(1);
                });

            if let Some(output_file) = output {
                match std::fs::write(&output_file, &response) {
                    Ok(_) => info!("✅ Response written to: {}", output_file),
                    Err(e) => {
                        error!("❌ Failed to write to output file '{}': {}", output_file, e);
                        std::process::exit(1);
                    }
                }
            } else {
                println!("{}", response);
            }
        }

        Commands::Embed {
            text,
            input,
            output,
            model,
            normalize,
        } => {
            let flock_manager = FlockManager::new()?;

            // Check if Flock is ready
            if !flock_manager.is_flock_ready()? {
                error!("❌ Flock extension not available");
                error!("   Run 'frozen-duckdb flock-setup' first");
                std::process::exit(4);
            }

            let texts_to_embed = if let Some(text_content) = text {
                vec![text_content.clone()]
            } else if let Some(input_file) = input {
                // Read texts from input file (one per line)
                match std::fs::read_to_string(&input_file) {
                    Ok(content) => content.lines().map(|s| s.to_string()).collect(),
                    Err(e) => {
                        error!("❌ Failed to read input file '{}': {}", input_file, e);
                        std::process::exit(1);
                    }
                }
            } else {
                error!("❌ Must provide either --text or --input");
                std::process::exit(1);
            };

            let embeddings = flock_manager.generate_embeddings(texts_to_embed, &model, normalize)
                .expect("Embedding generation not implemented yet");

            if let Some(output_file) = output {
                // Write embeddings as JSON
                let json_data = serde_json::to_string_pretty(&embeddings)
                    .context("Failed to serialize embeddings to JSON")?;
                match std::fs::write(&output_file, json_data) {
                    Ok(_) => info!("✅ Embeddings written to: {}", output_file),
                    Err(e) => {
                        error!("❌ Failed to write to output file '{}': {}", output_file, e);
                        std::process::exit(1);
                    }
                }
            } else {
                // Print embeddings to stdout
                println!("{}", serde_json::to_string_pretty(&embeddings)?);
            }
        }

        Commands::Search {
            query,
            corpus,
            threshold,
            limit,
            format,
        } => {
            let flock_manager = FlockManager::new()?;

            // Check if Flock is ready
            if !flock_manager.is_flock_ready()? {
                error!("❌ Flock extension not available");
                error!("   Run 'frozen-duckdb flock-setup' first");
                std::process::exit(4);
            }

            let results = flock_manager.semantic_search(&query, &corpus, threshold, limit)
                .expect("Semantic search not implemented yet");

            match format.as_str() {
                "json" => {
                    let json_results: Vec<Value> = results
                        .into_iter()
                        .map(|(doc, score)| {
                            serde_json::json!({
                                "document": doc,
                                "similarity_score": score
                            })
                        })
                        .collect();
                    println!("{}", serde_json::to_string_pretty(&json_results)?);
                }
                _ => {
                    // Text format
                    if results.is_empty() {
                        info!("🔍 No similar documents found above threshold {:.3}", threshold);
                    } else {
                        info!("🔍 Found {} similar documents:", results.len());
                        for (i, (doc, score)) in results.iter().enumerate() {
                            println!("  {}. \"{}\" (similarity: {:.3})", i + 1, doc, score);
                        }
                    }
                }
            }
        }

        Commands::Filter {
            criteria,
            prompt,
            input,
            output,
            model,
            positive_only,
        } => {
            let flock_manager = FlockManager::new()?;

            // Check if Flock is ready
            if !flock_manager.is_flock_ready()? {
                error!("❌ Flock extension not available");
                error!("   Run 'frozen-duckdb flock-setup' first");
                std::process::exit(4);
            }

            let filter_criteria = if let Some(custom_prompt) = prompt {
                custom_prompt.clone()
            } else if let Some(criteria_text) = criteria {
                format!("{} Answer yes or no: {{text}}", criteria_text)
            } else {
                error!("❌ Must provide either --criteria or --prompt");
                std::process::exit(1);
            };

            let results = flock_manager.llm_filter(&filter_criteria, &input, &model, true)
                .expect("LLM filtering not implemented yet");

            if let Some(output_file) = output {
                let json_data = serde_json::to_string_pretty(&results)
                    .context("Failed to serialize filter results to JSON")?;
                match std::fs::write(&output_file, json_data) {
                    Ok(_) => info!("✅ Filter results written to: {}", output_file),
                    Err(e) => {
                        error!("❌ Failed to write to output file '{}': {}", output_file, e);
                        std::process::exit(1);
                    }
                }
            } else {
                // Print results to stdout
                if positive_only {
                    info!("✅ Items that match criteria:");
                    for (item, matches) in results {
                        if matches {
                            println!("✅ {}", item);
                        }
                    }
                } else {
                    info!("📊 Filter results:");
                    for (item, matches) in results {
                        let status = if matches { "✅ MATCH" } else { "❌ NO MATCH" };
                        println!("{}: {}", status, item);
                    }
                }
            }
        }

        Commands::Summarize {
            input,
            output,
            strategy,
            max_length,
            model,
        } => {
            let flock_manager = FlockManager::new()?;

            // Check if Flock is ready
            if !flock_manager.is_flock_ready()? {
                error!("❌ Flock extension not available");
                error!("   Run 'frozen-duckdb flock-setup' first");
                std::process::exit(4);
            }

            // Read input texts
            let texts = if Path::new(&input).is_dir() {
                // Read all text files in directory
                let mut all_texts = Vec::new();
                for entry in std::fs::read_dir(&input)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("txt") {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            all_texts.push(content.trim().to_string());
                        }
                    }
                }
                all_texts
            } else {
                // Read from single file (one text per line)
                match std::fs::read_to_string(&input) {
                    Ok(content) => content.lines().map(|s| s.to_string()).collect(),
                    Err(e) => {
                        error!("❌ Failed to read input file '{}': {}", input, e);
                        std::process::exit(1);
                    }
                }
            };

            let summary = flock_manager.summarize_texts(texts, &strategy, max_length, &model)
                .expect("Text summarization not implemented yet");

            if let Some(output_file) = output {
                match std::fs::write(&output_file, &summary) {
                    Ok(_) => info!("✅ Summary written to: {}", output_file),
                    Err(e) => {
                        error!("❌ Failed to write to output file '{}': {}", output_file, e);
                        std::process::exit(1);
                    }
                }
            } else {
                println!("{}", summary);
            }
        }

        // === UTILITY COMMANDS ===
        Commands::Test => {
            info!("🧪 Tests have been moved to the test suite");
            info!("   Run tests with: cargo test");
            info!("   Run specific tests with: cargo test <test_name>");
            info!("   Run all tests with: cargo test --all");
        }

        Commands::Benchmark {
            operation,
            iterations,
            size,
        } => {
            info!(
                "Benchmarking {} operation with {} iterations (size: {})",
                operation, iterations, size
            );
            info!("📊 Performance benchmarking feature coming soon!");
        }
    }

    Ok(())
}
