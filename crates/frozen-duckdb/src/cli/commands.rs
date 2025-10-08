//! # CLI Commands for Frozen DuckDB
//!
//! This module defines the command-line interface commands and their
//! argument structures using clap for argument parsing.

use clap::{Parser, Subcommand};

/// Main CLI application structure for frozen DuckDB operations.
///
/// This struct defines the command-line interface with support for multiple
/// subcommands and verbosity control. It uses clap for argument parsing
/// and provides a clean, intuitive interface for all frozen DuckDB operations.
///
/// # Examples
///
/// ```bash
/// # Basic usage with default verbosity
/// frozen-duckdb info
///
/// # Increased verbosity for debugging
/// frozen-duckdb -v download --dataset chinook
/// frozen-duckdb -vv convert --input data.csv --output data.parquet
/// frozen-duckdb -vvv benchmark --operation query
/// ```
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Increase verbosity level (can be used multiple times: -v, -vv, -vvv)
    ///
    /// Controls the level of detail in output messages:
    /// - No flag: WARN level and above
    /// - `-v`: INFO level and above
    /// - `-vv`: DEBUG level and above
    /// - `-vvv`: TRACE level and above (most verbose)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// The command to execute
    #[command(subcommand)]
    pub command: Commands,
}

/// Available CLI commands for frozen DuckDB operations.
///
/// This enum defines all the subcommands available in the CLI tool,
/// each with their specific arguments and options. Commands are designed
/// to be intuitive and follow common CLI patterns.
#[derive(Subcommand)]
pub enum Commands {
    /// Download and generate sample datasets for testing and development.
    ///
    /// This command provides access to popular datasets like Chinook (music database)
    /// and TPC-H (decision support benchmark). Datasets can be generated in
    /// multiple formats for different use cases.
    ///
    /// # Examples
    ///
    /// ```bash
    /// # Download Chinook dataset in CSV format
    /// frozen-duckdb download --dataset chinook --format csv
    ///
    /// # Generate TPC-H dataset in Parquet format
    /// frozen-duckdb download --dataset tpch --format parquet --output-dir ./data
    /// ```
    Download {
        /// Dataset name to download or generate
        ///
        /// Available datasets:
        /// - `chinook`: Music database with artists, albums, tracks, and sales data
        /// - `tpch`: TPC-H decision support benchmark with 8 tables
        #[arg(short, long)]
        dataset: String,

        /// Output directory for the dataset files
        ///
        /// Defaults to "datasets" in the current directory.
        /// The directory will be created if it doesn't exist.
        #[arg(short, long, default_value = "datasets")]
        output_dir: String,

        /// Output format for the dataset
        ///
        /// Supported formats:
        /// - `csv`: Comma-separated values (human-readable, larger files)
        /// - `parquet`: Columnar format (compressed, faster queries)
        /// - `duckdb`: Native DuckDB database format (fastest for DuckDB)
        #[arg(short, long, default_value = "csv")]
        format: String,
    },

    /// Convert datasets between different file formats.
    ///
    /// This command provides format conversion capabilities for data files,
    /// allowing you to convert between CSV, Parquet, JSON, and other formats.
    /// Conversion is optimized for performance and maintains data integrity.
    ///
    /// # Examples
    ///
    /// ```bash
    /// # Convert CSV to Parquet
    /// frozen-duckdb convert --input data.csv --output data.parquet
    ///
    /// # Convert Parquet to CSV with explicit formats
    /// frozen-duckdb convert --input data.parquet --output data.csv --input-format parquet --output-format csv
    /// ```
    Convert {
        /// Input file path to convert from
        #[arg(short, long)]
        input: String,

        /// Output file path to convert to
        #[arg(short, long)]
        output: String,

        /// Input file format
        ///
        /// Supported input formats: csv, parquet, json
        #[arg(short, long, default_value = "csv")]
        input_format: String,

        /// Output file format
        ///
        /// Supported output formats: csv, parquet, json, arrow
        #[arg(short, long, default_value = "parquet")]
        output_format: String,
    },

    /// Display information about running tests.
    ///
    /// This command provides guidance on running the comprehensive test suite.
    /// It doesn't actually run tests (use `cargo test` for that) but shows
    /// the available test commands and options.
    Test,

    /// Benchmark operations to measure performance characteristics.
    ///
    /// This command runs performance benchmarks on various DuckDB operations
    /// to help understand performance characteristics and validate that
    /// performance targets are being met.
    ///
    /// # Examples
    ///
    /// ```bash
    /// # Benchmark query operations
    /// frozen-duckdb benchmark --operation query --iterations 1000
    ///
    /// # Benchmark with different dataset sizes
    /// frozen-duckdb benchmark --operation insert --size large --iterations 100
    /// ```
    Benchmark {
        /// Operation type to benchmark
        ///
        /// Available operations:
        /// - `query`: SQL query execution performance
        /// - `insert`: Data insertion performance
        /// - `export`: Data export performance
        #[arg(short, long, default_value = "query")]
        operation: String,

        /// Number of iterations to run for statistical accuracy
        ///
        /// Higher values provide more accurate results but take longer.
        /// Recommended: 1000 for quick tests, 10000 for detailed analysis.
        #[arg(short, long, default_value = "1000")]
        iterations: usize,

        /// Dataset size for benchmarking
        ///
        /// Available sizes:
        /// - `small`: ~1K rows (fast, good for development)
        /// - `medium`: ~10K rows (balanced, good for most testing)
        /// - `large`: ~100K rows (slow, good for performance validation)
        #[arg(short, long, default_value = "medium")]
        size: String,
    },

    /// Show comprehensive information about frozen DuckDB configuration.
    ///
    /// This command displays system information, available extensions,
    /// architecture details, and configuration status. Useful for
    /// troubleshooting and verifying that the environment is properly set up.
    ///
    /// # Examples
    ///
    /// ```bash
    /// # Show basic information
    /// frozen-duckdb info
    ///
    /// # Show detailed information with verbose output
    /// frozen-duckdb -v info
    /// ```
    Info,

    // === FLOCK/LLM COMMANDS ===

    /// Setup Ollama models and secrets for Flock LLM operations.
    ///
    /// This command configures the necessary models and secrets for using
    /// Flock's LLM capabilities with Ollama. It supports both text generation
    /// (qwen3-coder:30b) and embedding generation (qwen3-embedding:8b) models.
    ///
    /// # Examples
    ///
    /// ```bash
    /// # Setup default Ollama models
    /// frozen-duckdb flock-setup
    ///
    /// # Setup with custom Ollama URL
    /// frozen-duckdb flock-setup --ollama-url http://localhost:11434
    /// ```
    FlockSetup {
        /// Ollama server URL
        ///
        /// The URL where Ollama is running. Defaults to localhost:11434.
        #[arg(long, default_value = "http://localhost:11434")]
        ollama_url: String,

        /// Text generation model
        ///
        /// The Ollama model to use for text completion and generation.
        /// Examples: llama3.1:8b, qwen2.5:14b, codellama:13b
        #[arg(long, default_value = "qwen3-coder:30b")]
        text_model: String,

        /// Embedding model
        ///
        /// The Ollama model to use for generating embeddings.
        /// Examples: mxbai-embed-large, all-minilm:latest, nomic-embed-text
        #[arg(long, default_value = "qwen3-embedding:8b")]
        embedding_model: String,

        /// Skip model verification
        ///
        /// If set, skips checking if the required models are available.
        /// Useful for offline setup or when models will be pulled later.
        #[arg(long)]
        skip_verification: bool,
    },

    /// Generate text completions using LLM models via Flock.
    ///
    /// This command uses the configured LLM models to generate text completions
    /// for the provided prompts. Supports both single prompts and batch processing.
    ///
    /// # Examples
    ///
    /// ```bash
    /// # Complete a single prompt
    /// frozen-duckdb complete --prompt "Write a hello world function in Python"
    ///
    /// # Complete with specific model
    /// frozen-duckdb complete --prompt "Explain recursion" --model coder
    ///
    /// # Batch completion from file
    /// frozen-duckdb complete --input prompts.txt --output responses.txt
    /// ```
    Complete {
        /// Text prompt for completion
        ///
        /// The text to complete using the LLM model.
        #[arg(short, long)]
        prompt: Option<String>,

        /// Input file containing prompts (one per line)
        ///
        /// If provided, each line will be treated as a separate prompt.
        /// Cannot be used with --prompt.
        #[arg(short, long, conflicts_with = "prompt")]
        input: Option<String>,

        /// Output file for responses
        ///
        /// If provided, responses will be written to this file.
        /// If not provided, responses will be printed to stdout.
        #[arg(short, long)]
        output: Option<String>,

        /// Model to use for completion
        ///
        /// Use the model alias configured during setup (default: "text_generator")
        /// This corresponds to the text generation model set in flock-setup.
        #[arg(short, long, default_value = "text_generator")]
        model: String,

        /// Maximum tokens to generate
        ///
        /// Controls the length of the generated response.
        /// Higher values allow for longer responses.
        #[arg(long, default_value = "512")]
        max_tokens: usize,

        /// Temperature for text generation
        ///
        /// Controls randomness in generation:
        /// - 0.0: Deterministic (same input always gives same output)
        /// - 1.0: Balanced creativity and coherence
        /// - Higher: More creative but potentially less coherent
        #[arg(short, long, default_value = "0.7")]
        temperature: f32,
    },

    /// Generate embeddings for text using LLM models via Flock.
    ///
    /// This command generates vector embeddings for the provided text,
    /// which can be used for semantic search, similarity comparison,
    /// and other vector-based operations.
    ///
    /// # Examples
    ///
    /// ```bash
    /// # Generate embedding for single text
    /// frozen-duckdb embed --text "Python is a programming language"
    ///
    /// # Generate embeddings for multiple texts from file
    /// frozen-duckdb embed --input texts.txt --output embeddings.json
    /// ```
    Embed {
        /// Text to generate embeddings for
        ///
        /// The text content to convert into vector embeddings.
        #[arg(short, long)]
        text: Option<String>,

        /// Input file containing texts (one per line)
        ///
        /// If provided, each line will be treated as separate text for embedding.
        /// Cannot be used with --text.
        #[arg(short, long, conflicts_with = "text")]
        input: Option<String>,

        /// Output file for embeddings
        ///
        /// If provided, embeddings will be written to this file in JSON format.
        /// If not provided, embeddings will be printed to stdout.
        #[arg(short, long)]
        output: Option<String>,

        /// Model to use for embedding generation
        ///
        /// Use the model alias configured during setup (default: "embedder")
        /// This corresponds to the embedding model set in flock-setup.
        #[arg(short, long, default_value = "embedder")]
        model: String,

        /// Normalize embeddings
        ///
        /// If set, embeddings will be normalized to unit length.
        /// Useful for cosine similarity calculations.
        #[arg(long)]
        normalize: bool,
    },

    /// Perform semantic search using embeddings and Flock.
    ///
    /// This command performs semantic similarity search by comparing
    /// query embeddings against a corpus of documents. Results are
    /// ranked by semantic similarity rather than just keyword matching.
    ///
    /// # Examples
    ///
    /// ```bash
    /// # Search in a document corpus
    /// frozen-duckdb search --query "machine learning algorithms" --corpus documents.txt
    ///
    /// # Search with specific similarity threshold
    /// frozen-duckdb search --query "data science" --corpus docs/ --threshold 0.8
    /// ```
    Search {
        /// Search query text
        ///
        /// The text query to search for semantically similar content.
        #[arg(short, long)]
        query: String,

        /// Corpus file or directory
        ///
        /// File containing documents (one per line) or directory containing text files.
        #[arg(short, long)]
        corpus: String,

        /// Similarity threshold
        ///
        /// Minimum similarity score (0.0 to 1.0) for results to be included.
        /// Higher values return more similar (but fewer) results.
        #[arg(short, long, default_value = "0.7")]
        threshold: f32,

        /// Maximum number of results
        ///
        /// Maximum number of similar documents to return.
        #[arg(short, long, default_value = "10")]
        limit: usize,

        /// Output format for results
        ///
        /// Available formats:
        /// - `text`: Human-readable text format
        /// - `json`: JSON format for programmatic processing
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Filter data using LLM-based classification via Flock.
    ///
    /// This command uses LLM models to classify and filter data based
    /// on natural language criteria. Useful for content moderation,
    /// categorization, and intelligent data filtering.
    ///
    /// # Examples
    ///
    /// ```bash
    /// # Filter code samples for validity
    /// frozen-duckdb filter --criteria "Is this valid Python code?" --input code_samples.csv --output valid_code.csv
    ///
    /// # Filter with custom prompt
    /// frozen-duckdb filter --prompt "Does this text contain positive sentiment?" --input reviews.txt
    /// ```
    Filter {
        /// Filtering criteria or prompt
        ///
        /// Natural language description of what to filter for,
        /// or a custom prompt for classification.
        #[arg(short, long)]
        criteria: Option<String>,

        /// Custom prompt for filtering
        ///
        /// A custom prompt template for the LLM to use when classifying content.
        /// Use {{text}} as placeholder for the content being filtered.
        #[arg(short, long, conflicts_with = "criteria")]
        prompt: Option<String>,

        /// Input file containing data to filter
        ///
        /// File containing the data to be filtered (CSV, JSON, or text format).
        #[arg(short, long)]
        input: String,

        /// Output file for filtered results
        ///
        /// File where matching results will be written.
        #[arg(short, long)]
        output: Option<String>,

        /// Model to use for filtering
        ///
        /// Use the model alias configured during setup (default: "text_generator")
        /// This corresponds to the text generation model set in flock-setup.
        #[arg(short, long, default_value = "text_generator")]
        model: String,

        /// Filter for positive matches only
        ///
        /// If set, only items that match the criteria will be included.
        /// If not set, all items will be included with match scores.
        #[arg(long)]
        positive_only: bool,
    },

    /// Generate summaries using LLM aggregation via Flock.
    ///
    /// This command uses LLM models to generate summaries and insights
    /// from collections of text data. Supports various aggregation
    /// strategies for different types of content analysis.
    ///
    /// # Examples
    ///
    /// ```bash
    /// # Summarize multiple text files
    /// frozen-duckdb summarize --input documents/ --output summary.txt
    ///
    /// # Summarize with custom aggregation strategy
    /// frozen-duckdb summarize --input articles.txt --strategy reduce --max-length 200
    /// ```
    Summarize {
        /// Input file or directory containing text to summarize
        ///
        /// File containing texts (one per line) or directory containing text files.
        #[arg(short, long)]
        input: String,

        /// Output file for summary
        ///
        /// File where the generated summary will be written.
        #[arg(short, long)]
        output: Option<String>,

        /// Summarization strategy
        ///
        /// Available strategies:
        /// - `reduce`: Use LLM reduce function for hierarchical summarization
        /// - `map`: Generate individual summaries then combine
        /// - `extractive`: Extract key sentences without generation
        #[arg(short, long, default_value = "reduce")]
        strategy: String,

        /// Maximum summary length in words
        ///
        /// Controls the length of the generated summary.
        #[arg(short, long, default_value = "150")]
        max_length: usize,

        /// Model to use for summarization
        ///
        /// Use the model alias configured during setup (default: "text_generator")
        /// This corresponds to the text generation model set in flock-setup.
        #[arg(short, long, default_value = "text_generator")]
        model: String,
    },

    /// Validate FFI functionality including core DuckDB + Flock LLM extensions.
    ///
    /// This command runs comprehensive FFI validation to ensure that
    /// the frozen-duckdb library properly exposes all required functionality.
    /// Tests binary loading, core functions, extensions, and LLM integration.
    ///
    /// # Examples
    ///
    /// ```bash
    /// # Run all FFI validation tests
    /// frozen-duckdb validate-ffi
    ///
    /// # Run with specific architecture
    /// ARCH=x86_64 frozen-duckdb validate-ffi
    /// ARCH=arm64 frozen-duckdb validate-ffi
    ///
    /// # Skip LLM validation (faster, no Ollama required)
    /// frozen-duckdb validate-ffi --skip-llm
    ///
    /// # Output results in JSON format
    /// frozen-duckdb validate-ffi --format json
    /// ```
    ///
    /// # Validation Layers
    ///
    /// - **Binary Validation**: Check library files and headers
    /// - **FFI Function Validation**: Verify C API functions are available
    /// - **Core Functionality**: Test basic DuckDB operations
    /// - **Extension Validation**: Test Flock LLM functions
    /// - **Integration Validation**: Test end-to-end LLM workflows
    ValidateFfi {
        /// Skip LLM validation (faster, no Ollama required)
        ///
        /// If set, skips the integration validation layer that requires
        /// Ollama to be running and models to be available. Useful for
        /// quick validation of core functionality.
        #[arg(long)]
        skip_llm: bool,

        /// Output format for results
        ///
        /// Choose the format for displaying validation results.
        /// Human-readable format is default, JSON is useful for automation.
        #[arg(long, default_value = "human")]
        format: String,

        /// Verbose output
        ///
        /// If set, shows detailed information about each validation step
        /// including timing and error details.
        #[arg(long)]
        verbose: bool,
    },
}
