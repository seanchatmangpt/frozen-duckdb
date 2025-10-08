//! # DuckDB FFI Header for Go Smoke Test
//!
//! This header provides the necessary C API declarations for testing
//! the frozen-duckdb FFI functionality from Go. It includes:
//!
//! - Core DuckDB C API functions
//! - Type definitions and constants
//! - Flock extension function signatures
//!
//! This header is used by the Go smoke test to validate that all
//! FFI functions are properly exposed and functional.

#ifndef DUCKDB_FFI_H
#define DUCKDB_FFI_H

#include <stdbool.h>
#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C"
{
#endif

  //===--------------------------------------------------------------------===//
  // Core DuckDB Types and Constants
  //===--------------------------------------------------------------------===//

  typedef enum
  {
    DuckDBSuccess = 0,
    DuckDBError = 1
  } duckdb_state;

  typedef enum
  {
    DUCKDB_TYPE_INVALID = 0,
    DUCKDB_TYPE_BOOLEAN = 1,
    DUCKDB_TYPE_TINYINT = 2,
    DUCKDB_TYPE_SMALLINT = 3,
    DUCKDB_TYPE_INTEGER = 4,
    DUCKDB_TYPE_BIGINT = 5,
    DUCKDB_TYPE_FLOAT = 6,
    DUCKDB_TYPE_DOUBLE = 7,
    DUCKDB_TYPE_VARCHAR = 8
  } duckdb_type;

  typedef uint64_t idx_t;

  // Opaque pointer types
  typedef struct _duckdb_database *duckdb_database;
  typedef struct _duckdb_connection *duckdb_connection;
  typedef struct _duckdb_result *duckdb_result;

  //===--------------------------------------------------------------------===//
  // Core DuckDB C API Functions
  //===--------------------------------------------------------------------===//

  // Database lifecycle
  duckdb_state duckdb_open(const char *path, duckdb_database *out_database);
  void duckdb_close(duckdb_database *database);
  duckdb_state duckdb_connect(duckdb_database database, duckdb_connection *out_connection);
  void duckdb_disconnect(duckdb_connection *connection);

  // Query execution
  duckdb_state duckdb_query(duckdb_connection connection, const char *query, duckdb_result *out_result);
  void duckdb_destroy_result(duckdb_result *result);

  // Library information
  const char *duckdb_library_version();

  // Result inspection
  idx_t duckdb_column_count(duckdb_result *result);
  idx_t duckdb_row_count(duckdb_result *result);
  const char *duckdb_column_name(duckdb_result *result, idx_t col);
  duckdb_type duckdb_column_type(duckdb_result *result, idx_t col);
  void *duckdb_column_data(duckdb_result *result, idx_t col);
  bool *duckdb_nullmask_data(duckdb_result *result, idx_t col);

  // Error handling
  const char *duckdb_result_error(duckdb_result *result);

  // Value extraction
  bool duckdb_value_boolean(duckdb_result *result, idx_t col, idx_t row);
  int8_t duckdb_value_int8(duckdb_result *result, idx_t col, idx_t row);
  int16_t duckdb_value_int16(duckdb_result *result, idx_t col, idx_t row);
  int32_t duckdb_value_int32(duckdb_result *result, idx_t col, idx_t row);
  int64_t duckdb_value_int64(duckdb_result *result, idx_t col, idx_t row);
  float duckdb_value_float(duckdb_result *result, idx_t col, idx_t row);
  double duckdb_value_double(duckdb_result *result, idx_t col, idx_t row);
  const char *duckdb_value_varchar(duckdb_result *result, idx_t col, idx_t row);

#ifdef __cplusplus
}
#endif

#endif // DUCKDB_FFI_H
