#!/bin/bash
# Create Pre-compiled DuckDB Binary - Never Compile Again!
# This creates a static DuckDB binary that can be used without any compilation

set -e

echo "🦆 Creating Pre-compiled DuckDB Binary (Never Compile Again!)"
echo "============================================================"

# Configuration
DUCKDB_VERSION="1.4.0"
BUILD_DIR="target/duckdb-precompiled"
BINARY_NAME="libduckdb_precompiled.a"

# Create build directory
mkdir -p "$BUILD_DIR"
cd "$BUILD_DIR"

echo "📦 Step 1: Downloading DuckDB source..."
# Download DuckDB source
DUCKDB_URL="https://github.com/duckdb/duckdb/archive/refs/tags/v${DUCKDB_VERSION}.tar.gz"
curl -L "$DUCKDB_URL" -o "duckdb-${DUCKDB_VERSION}.tar.gz"
tar -xzf "duckdb-${DUCKDB_VERSION}.tar.gz"
mv "duckdb-${DUCKDB_VERSION}" duckdb

echo "🔧 Step 2: Configuring DuckDB build..."
cd duckdb

# Create a minimal build configuration
cat > CMakeLists.txt << 'EOF'
cmake_minimum_required(VERSION 3.20)
project(duckdb_precompiled)

# Set C++ standard
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Build options
option(BUILD_EXTENSIONS "Build extensions" OFF)
option(BUILD_PARQUET_EXTENSION "Build parquet extension" OFF)
option(BUILD_JSON_EXTENSION "Build json extension" OFF)
option(BUILD_ICU_EXTENSION "Build icu extension" OFF)
option(BUILD_TPCH_EXTENSION "Build tpch extension" OFF)
option(BUILD_TPCDS_EXTENSION "Build tpcds extension" OFF)
option(BUILD_FTS_EXTENSION "Build fts extension" OFF)
option(BUILD_HTTPFS_EXTENSION "Build httpfs extension" OFF)
option(BUILD_VISUALIZER_EXTENSION "Build visualizer extension" OFF)
option(BUILD_AUTOLOAD_EXTENSIONS "Build autoload extensions" OFF)

# Disable all extensions for minimal build
set(BUILD_EXTENSIONS OFF)
set(BUILD_PARQUET_EXTENSION OFF)
set(BUILD_JSON_EXTENSION OFF)
set(BUILD_ICU_EXTENSION OFF)
set(BUILD_TPCH_EXTENSION OFF)
set(BUILD_TPCDS_EXTENSION OFF)
set(BUILD_FTS_EXTENSION OFF)
set(BUILD_HTTPFS_EXTENSION OFF)
set(BUILD_VISUALIZER_EXTENSION OFF)
set(BUILD_AUTOLOAD_EXTENSIONS OFF)

# Add DuckDB source
add_subdirectory(src)

# Create static library
add_library(duckdb_precompiled STATIC
    src/main/duckdb.cpp
    src/main/relation/read_csv_relation.cpp
    src/main/relation/read_parquet_relation.cpp
    src/main/relation/query_relation.cpp
    src/main/relation/table_function_relation.cpp
    src/main/relation/value_relation.cpp
    src/main/relation/aggregate_relation.cpp
    src/main/relation/filter_relation.cpp
    src/main/relation/order_relation.cpp
    src/main/relation/limit_relation.cpp
    src/main/relation/top_n_relation.cpp
    src/main/relation/join_relation.cpp
    src/main/relation/setop_relation.cpp
    src/main/relation/insert_relation.cpp
    src/main/relation/create_relation.cpp
    src/main/relation/create_table_relation.cpp
    src/main/relation/create_view_relation.cpp
    src/main/relation/explain_relation.cpp
    src/main/relation/export_csv_relation.cpp
    src/main/relation/export_parquet_relation.cpp
    src/main/relation/update_relation.cpp
    src/main/relation/delete_relation.cpp
    src/main/relation/drop_relation.cpp
    src/main/relation/alter_relation.cpp
    src/main/relation/pragma_relation.cpp
    src/main/relation/transaction_relation.cpp
    src/main/relation/version_relation.cpp
    src/main/relation/copy_relation.cpp
    src/main/relation/copy_database_relation.cpp
    src/main/relation/attach_relation.cpp
    src/main/relation/detach_relation.cpp
    src/main/relation/use_relation.cpp
    src/main/relation/show_relation.cpp
    src/main/relation/describe_relation.cpp
    src/main/relation/summarize_relation.cpp
    src/main/relation/pivot_relation.cpp
    src/main/relation/unpivot_relation.cpp
    src/main/relation/sample_relation.cpp
    src/main/relation/distinct_relation.cpp
    src/main/relation/union_relation.cpp
    src/main/relation/except_relation.cpp
    src/main/relation/intersect_relation.cpp
    src/main/relation/projection_relation.cpp
    src/main/relation/cross_product_relation.cpp
    src/main/relation/subquery_relation.cpp
    src/main/relation/table_relation.cpp
    src/main/relation/empty_relation.cpp
    src/main/relation/expression_relation.cpp
    src/main/relation/cte_relation.cpp
    src/main/relation/recursive_cte_relation.cpp
    src/main/relation/materialized_relation.cpp
    src/main/relation/column_data_relation.cpp
    src/main/relation/column_data_collection_relation.cpp
    src/main/relation/column_data_scan_relation.cpp
    src/main/relation/column_data_aggregate_relation.cpp
    src/main/relation/column_data_join_relation.cpp
    src/main/relation/column_data_setop_relation.cpp
    src/main/relation/column_data_distinct_relation.cpp
    src/main/relation/column_data_order_relation.cpp
    src/main/relation/column_data_limit_relation.cpp
    src/main/relation/column_data_top_n_relation.cpp
    src/main/relation/column_data_filter_relation.cpp
    src/main/relation/column_data_projection_relation.cpp
    src/main/relation/column_data_cross_product_relation.cpp
    src/main/relation/column_data_subquery_relation.cpp
    src/main/relation/column_data_table_relation.cpp
    src/main/relation/column_data_empty_relation.cpp
    src/main/relation/column_data_expression_relation.cpp
    src/main/relation/column_data_cte_relation.cpp
    src/main/relation/column_data_recursive_cte_relation.cpp
    src/main/relation/column_data_materialized_relation.cpp
    src/main/relation/column_data_column_data_relation.cpp
    src/main/relation/column_data_column_data_collection_relation.cpp
    src/main/relation/column_data_column_data_scan_relation.cpp
    src/main/relation/column_data_column_data_aggregate_relation.cpp
    src/main/relation/column_data_column_data_join_relation.cpp
    src/main/relation/column_data_column_data_setop_relation.cpp
    src/main/relation/column_data_column_data_distinct_relation.cpp
    src/main/relation/column_data_column_data_order_relation.cpp
    src/main/relation/column_data_column_data_limit_relation.cpp
    src/main/relation/column_data_column_data_top_n_relation.cpp
    src/main/relation/column_data_column_data_filter_relation.cpp
    src/main/relation/column_data_column_data_projection_relation.cpp
    src/main/relation/column_data_column_data_cross_product_relation.cpp
    src/main/relation/column_data_column_data_subquery_relation.cpp
    src/main/relation/column_data_column_data_table_relation.cpp
    src/main/relation/column_data_column_data_empty_relation.cpp
    src/main/relation/column_data_column_data_expression_relation.cpp
    src/main/relation/column_data_column_data_cte_relation.cpp
    src/main/relation/column_data_column_data_recursive_cte_relation.cpp
    src/main/relation/column_data_column_data_materialized_relation.cpp
)

# Link against DuckDB core
target_link_libraries(duckdb_precompiled duckdb)
EOF

echo "⚡ Step 3: Building DuckDB static library..."
# Build with minimal configuration
mkdir -p build
cd build

# Configure with minimal options
cmake .. \
    -DCMAKE_BUILD_TYPE=Release \
    -DCMAKE_CXX_FLAGS="-O3 -DNDEBUG" \
    -DBUILD_EXTENSIONS=OFF \
    -DBUILD_PARQUET_EXTENSION=OFF \
    -DBUILD_JSON_EXTENSION=OFF \
    -DBUILD_ICU_EXTENSION=OFF \
    -DBUILD_TPCH_EXTENSION=OFF \
    -DBUILD_TPCDS_EXTENSION=OFF \
    -DBUILD_FTS_EXTENSION=OFF \
    -DBUILD_HTTPFS_EXTENSION=OFF \
    -DBUILD_VISUALIZER_EXTENSION=OFF \
    -DBUILD_AUTOLOAD_EXTENSIONS=OFF

# Build only the core library
make -j$(nproc) duckdb

echo "📦 Step 4: Creating pre-compiled binary package..."
# Copy the built library
cp src/libduckdb.a "../../${BINARY_NAME}"

# Create header file
cp ../src/include/duckdb.h ../../duckdb_precompiled.h

cd ../..

echo "✅ Pre-compiled DuckDB binary created!"
echo "📁 Binary: $BUILD_DIR/$BINARY_NAME"
echo "📁 Header: $BUILD_DIR/duckdb_precompiled.h"
echo ""
echo "🎯 This binary will NEVER need to be compiled again!"
echo "   - Static library: $BINARY_NAME"
echo "   - Header file: duckdb_precompiled.h"
echo "   - Size: $(du -h $BINARY_NAME | cut -f1)"
echo ""
echo "🚀 Next: Update kcura-duck to use this pre-compiled binary"
