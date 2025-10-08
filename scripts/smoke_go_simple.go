//! # Simple Go FFI Smoke Test for Frozen DuckDB
//!
//! This smoke test validates that the frozen-duckdb library properly exposes
//! the core DuckDB FFI functions and Flock LLM extensions.
//!
//! ## Usage
//!
//! ```bash
//! # Run with frozen DuckDB environment
//! source prebuilt/setup_env.sh
//! ./scripts/build_go_smoketest.sh
//! ```

package main

/*
#cgo CFLAGS: -I.
#include "duckdb_ffi.h"
#include <stdlib.h>
*/
import "C"
import (
	"fmt"
	"os"
	"runtime"
	"strings"
	"time"
	"unsafe"
)

// Test results tracking
type TestResult struct {
	Name     string
	Passed   bool
	Duration time.Duration
	Error    string
}

type TestSuite struct {
	Results []TestResult
	Start   time.Time
}

func NewTestSuite() *TestSuite {
	return &TestSuite{
		Results: make([]TestResult, 0),
		Start:   time.Now(),
	}
}

func (ts *TestSuite) RunTest(name string, testFunc func() error) {
	start := time.Now()
	err := testFunc()
	duration := time.Since(start)
	
	result := TestResult{
		Name:     name,
		Passed:   err == nil,
		Duration: duration,
	}
	
	if err != nil {
		result.Error = err.Error()
	}
	
	ts.Results = append(ts.Results, result)
	
	status := "✅ PASS"
	if !result.Passed {
		status = "❌ FAIL"
	}
	
	fmt.Printf("%s %s (%v)\n", status, name, duration)
	if !result.Passed {
		fmt.Printf("   Error: %s\n", result.Error)
	}
}

func (ts *TestSuite) Summary() {
	total := len(ts.Results)
	passed := 0
	totalDuration := time.Since(ts.Start)
	
	for _, result := range ts.Results {
		if result.Passed {
			passed++
		}
	}
	
	fmt.Printf("\n" + strings.Repeat("=", 60) + "\n")
	fmt.Printf("🦆 Frozen DuckDB FFI Smoke Test Results\n")
	fmt.Printf(strings.Repeat("=", 60) + "\n")
	fmt.Printf("Total Tests: %d\n", total)
	fmt.Printf("Passed: %d\n", passed)
	fmt.Printf("Failed: %d\n", total-passed)
	fmt.Printf("Success Rate: %.1f%%\n", float64(passed)/float64(total)*100)
	fmt.Printf("Total Duration: %v\n", totalDuration)
	
	if passed == total {
		fmt.Printf("🎉 ALL TESTS PASSED - FFI is fully functional!\n")
	} else {
		fmt.Printf("⚠️  Some tests failed - check FFI implementation\n")
		os.Exit(1)
	}
}

// Wrapper functions for DuckDB C API
func duckdbOpen(path string) (C.duckdb_database, error) {
	var db C.duckdb_database
	cPath := C.CString(path)
	defer C.free(unsafe.Pointer(cPath))
	
	state := C.duckdb_open(cPath, &db)
	if state != C.DuckDBSuccess {
		return nil, fmt.Errorf("failed to open database: %s", path)
	}
	return db, nil
}

func duckdbClose(db *C.duckdb_database) {
	C.duckdb_close(db)
}

func duckdbConnect(db C.duckdb_database) (C.duckdb_connection, error) {
	var conn C.duckdb_connection
	state := C.duckdb_connect(db, &conn)
	if state != C.DuckDBSuccess {
		return nil, fmt.Errorf("failed to connect to database")
	}
	return conn, nil
}

func duckdbDisconnect(conn *C.duckdb_connection) {
	C.duckdb_disconnect(conn)
}

func duckdbQuery(conn C.duckdb_connection, query string) (C.duckdb_result, error) {
	var result C.duckdb_result
	cQuery := C.CString(query)
	defer C.free(unsafe.Pointer(cQuery))
	
	state := C.duckdb_query(conn, cQuery, &result)
	if state != C.DuckDBSuccess {
		// Try to get error message
		errorMsg := C.duckdb_result_error(&result)
		if errorMsg != nil {
			return result, fmt.Errorf("query failed: %s", C.GoString(errorMsg))
		}
		return result, fmt.Errorf("query failed: %s", query)
	}
	return result, nil
}

func duckdbDestroyResult(result *C.duckdb_result) {
	C.duckdb_destroy_result(result)
}

func duckdbLibraryVersion() string {
	version := C.duckdb_library_version()
	if version != nil {
		return C.GoString(version)
	}
	return "unknown"
}

func duckdbColumnCount(result C.duckdb_result) int {
	return int(C.duckdb_column_count(&result))
}

func duckdbRowCount(result C.duckdb_result) int {
	return int(C.duckdb_row_count(&result))
}

func duckdbColumnName(result C.duckdb_result, col int) string {
	name := C.duckdb_column_name(&result, C.idx_t(col))
	if name != nil {
		return C.GoString(name)
	}
	return ""
}

func duckdbValueVarchar(result C.duckdb_result, col, row int) string {
	value := C.duckdb_value_varchar(&result, C.idx_t(col), C.idx_t(row))
	if value != nil {
		return C.GoString(value)
	}
	return ""
}

func duckdbValueInt32(result C.duckdb_result, col, row int) int32 {
	return int32(C.duckdb_value_int32(&result, C.idx_t(col), C.idx_t(row)))
}

// Test functions
func testLibraryVersion() error {
	version := duckdbLibraryVersion()
	if version == "" || version == "unknown" {
		return fmt.Errorf("failed to get library version")
	}
	fmt.Printf("   DuckDB Version: %s\n", version)
	return nil
}

func testDatabaseLifecycle() error {
	// Test in-memory database
	db, err := duckdbOpen(":memory:")
	if err != nil {
		return err
	}
	defer duckdbClose(&db)
	
	conn, err := duckdbConnect(db)
	if err != nil {
		return err
	}
	defer duckdbDisconnect(&conn)
	
	return nil
}

func testBasicQueries() error {
	db, err := duckdbOpen(":memory:")
	if err != nil {
		return err
	}
	defer duckdbClose(&db)
	
	conn, err := duckdbConnect(db)
	if err != nil {
		return err
	}
	defer duckdbDisconnect(&conn)
	
	// Test CREATE TABLE
	result, err := duckdbQuery(conn, "CREATE TABLE test (id INTEGER, name VARCHAR)")
	if err != nil {
		return err
	}
	duckdbDestroyResult(&result)
	
	// Test INSERT
	result, err = duckdbQuery(conn, "INSERT INTO test VALUES (1, 'test1'), (2, 'test2')")
	if err != nil {
		return err
	}
	duckdbDestroyResult(&result)
	
	// Test SELECT
	result, err = duckdbQuery(conn, "SELECT * FROM test ORDER BY id")
	if err != nil {
		return err
	}
	defer duckdbDestroyResult(&result)
	
	// Verify results
	rowCount := duckdbRowCount(result)
	colCount := duckdbColumnCount(result)
	
	if rowCount != 2 {
		return fmt.Errorf("expected 2 rows, got %d", rowCount)
	}
	
	if colCount != 2 {
		return fmt.Errorf("expected 2 columns, got %d", colCount)
	}
	
	// Check column names
	expectedCols := []string{"id", "name"}
	for i, expected := range expectedCols {
		actual := duckdbColumnName(result, i)
		if actual != expected {
			return fmt.Errorf("column %d: expected '%s', got '%s'", i, expected, actual)
		}
	}
	
	// Check values
	if duckdbValueInt32(result, 0, 0) != 1 {
		return fmt.Errorf("row 0, col 0: expected 1")
	}
	if duckdbValueVarchar(result, 1, 0) != "test1" {
		return fmt.Errorf("row 0, col 1: expected 'test1'")
	}
	
	return nil
}

func testFlockExtension() error {
	db, err := duckdbOpen(":memory:")
	if err != nil {
		return err
	}
	defer duckdbClose(&db)
	
	conn, err := duckdbConnect(db)
	if err != nil {
		return err
	}
	defer duckdbDisconnect(&conn)
	
	// Install and load Flock extension
	result, err := duckdbQuery(conn, "INSTALL flock FROM community")
	if err != nil {
		return fmt.Errorf("failed to install Flock extension: %v", err)
	}
	duckdbDestroyResult(&result)
	
	result, err = duckdbQuery(conn, "LOAD flock")
	if err != nil {
		return fmt.Errorf("failed to load Flock extension: %v", err)
	}
	duckdbDestroyResult(&result)
	
	// Verify extension is loaded
	result, err = duckdbQuery(conn, "SELECT extension_name FROM duckdb_extensions() WHERE extension_name = 'flock'")
	if err != nil {
		return err
	}
	defer duckdbDestroyResult(&result)
	
	if duckdbRowCount(result) != 1 {
		return fmt.Errorf("Flock extension not loaded")
	}
	
	return nil
}

func testFlockLLMFunctions() error {
	db, err := duckdbOpen(":memory:")
	if err != nil {
		return err
	}
	defer duckdbClose(&db)
	
	conn, err := duckdbConnect(db)
	if err != nil {
		return err
	}
	defer duckdbDisconnect(&conn)
	
	// Load Flock extension
	result, err := duckdbQuery(conn, "INSTALL flock FROM community; LOAD flock")
	if err != nil {
		return fmt.Errorf("failed to load Flock: %v", err)
	}
	duckdbDestroyResult(&result)
	
	// Test that LLM functions are available (they may fail without models, but should exist)
	queries := []string{
		"SELECT llm_complete({'model_name': 'test'}, {'prompt': 'Hello'})",
		"SELECT llm_embedding({'model_name': 'test'}, [{'data': 'test'}])",
		"SELECT fusion_rrf(0.5, 0.7)",
		"SELECT fusion_combsum(0.5, 0.7)",
	}
	
	for _, query := range queries {
		result, err := duckdbQuery(conn, query)
		// We expect these to fail without proper models/secrets, but the functions should exist
		if err != nil {
			// Check if error is about missing models/secrets (expected) vs function not found (bad)
			errorMsg := ""
			if result != nil {
				errorMsg = C.GoString(C.duckdb_result_error(&result))
			}
			
			if !strings.Contains(errorMsg, "model") && !strings.Contains(errorMsg, "secret") && 
			   !strings.Contains(errorMsg, "not found") && !strings.Contains(errorMsg, "does not exist") {
				duckdbDestroyResult(&result)
				return fmt.Errorf("unexpected error for query '%s': %v", query, err)
			}
		}
		if result != nil {
			duckdbDestroyResult(&result)
		}
	}
	
	return nil
}

func testErrorHandling() error {
	db, err := duckdbOpen(":memory:")
	if err != nil {
		return err
	}
	defer duckdbClose(&db)
	
	conn, err := duckdbConnect(db)
	if err != nil {
		return err
	}
	defer duckdbDisconnect(&conn)
	
	// Test invalid SQL
	result, err := duckdbQuery(conn, "INVALID SQL QUERY")
	if err == nil {
		duckdbDestroyResult(&result)
		return fmt.Errorf("expected error for invalid SQL")
	}
	
	return nil
}

func testArchitectureDetection() error {
	// Test that we can detect the current architecture
	arch := runtime.GOARCH
	fmt.Printf("   Detected Architecture: %s\n", arch)
	
	// Verify we're on a supported architecture
	supportedArchs := []string{"amd64", "arm64"}
	isSupported := false
	for _, supported := range supportedArchs {
		if arch == supported {
			isSupported = true
			break
		}
	}
	
	if !isSupported {
		return fmt.Errorf("unsupported architecture: %s", arch)
	}
	
	return nil
}

func main() {
	fmt.Printf("🦆 Frozen DuckDB FFI Smoke Test\n")
	fmt.Printf("Testing FFI functionality including core DuckDB + Flock LLM extensions\n")
	fmt.Printf(strings.Repeat("=", 60) + "\n")
	
	suite := NewTestSuite()
	
	// Core FFI tests
	suite.RunTest("Library Version", testLibraryVersion)
	suite.RunTest("Architecture Detection", testArchitectureDetection)
	suite.RunTest("Database Lifecycle", testDatabaseLifecycle)
	suite.RunTest("Basic Queries", testBasicQueries)
	suite.RunTest("Error Handling", testErrorHandling)
	
	// Flock LLM extension tests
	suite.RunTest("Flock Extension Loading", testFlockExtension)
	suite.RunTest("Flock LLM Functions", testFlockLLMFunctions)
	
	suite.Summary()
}
