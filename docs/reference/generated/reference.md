# frozen-duckdb reference

<!-- ============================================================= -->
<!-- AGENT-FORBIDDEN-BEGIN: reference body is RIGID                -->
<!-- Every row below is rendered from queries/ast_extract.rq.      -->
<!-- Agents MUST NOT add, edit, reorder, or remove any row or      -->
<!-- table cell. Prose outside the fenced slot below is refused    -->
<!-- by the doc_quality court.                                     -->
<!-- ============================================================= -->

## Modules


### crates/frozen-duckdb-builder/src/lib.rs

| `ensure_binary` | function | ensure_binary() |  |  |  |  |


### crates/frozen-duckdb-sys/build.rs

| `main` | function | main(_out_dir: &str, out_path: &Path, lib_dir: &Path) |  |  |  |  |

| `write_to_out_dir` | function | write_to_out_dir(header: HeaderLocation, out_path: &Path) |  |  |  |  |


### crates/frozen-duckdb-sys/src/bindgen_bundled_version.rs

| `duckdb_add_aggregate_function_to_set` | function | duckdb_add_aggregate_function_to_set(
        set: duckdb_aggregate_function_set,
        function: duckdb_aggregate_function,
    ) |  |  |  |  |

| `duckdb_add_replacement_scan` | function | duckdb_add_replacement_scan(
        db: duckdb_database,
        replacement: duckdb_replacement_callback_t,
        extra_data: *mut ::std::os::raw::c_void,
        delete_callback: duckdb_delete_callback_t,
    ) |  |  |  |  |

| `duckdb_add_scalar_function_to_set` | function | duckdb_add_scalar_function_to_set(
        set: duckdb_scalar_function_set,
        function: duckdb_scalar_function,
    ) |  |  |  |  |

| `duckdb_aggregate_function_add_parameter` | function | duckdb_aggregate_function_add_parameter(
        aggregate_function: duckdb_aggregate_function,
        type_: duckdb_logical_type,
    ) |  |  |  |  |

| `duckdb_aggregate_function_get_extra_info` | function | duckdb_aggregate_function_get_extra_info(info: duckdb_function_info) |  |  |  |  |

| `duckdb_aggregate_function_set_destructor` | function | duckdb_aggregate_function_set_destructor(
        aggregate_function: duckdb_aggregate_function,
        destroy: duckdb_aggregate_destroy_t,
    ) |  |  |  |  |

| `duckdb_aggregate_function_set_error` | function | duckdb_aggregate_function_set_error(info: duckdb_function_info, error: *const ::std::os::raw::c_char) |  |  |  |  |

| `duckdb_aggregate_function_set_extra_info` | function | duckdb_aggregate_function_set_extra_info(
        aggregate_function: duckdb_aggregate_function,
        extra_info: *mut ::std::os::raw::c_void,
        destroy: duckdb_delete_callback_t,
    ) |  |  |  |  |

| `duckdb_aggregate_function_set_functions` | function | duckdb_aggregate_function_set_functions(
        aggregate_function: duckdb_aggregate_function,
        state_size: duckdb_aggregate_state_size,
        state_init: duckdb_aggregate_init_t,
        update: duckdb_aggregate_update_t,
        combine: duckdb_aggregate_combine_t,
        finalize: duckdb_aggregate_finalize_t,
    ) |  |  |  |  |

| `duckdb_aggregate_function_set_name` | function | duckdb_aggregate_function_set_name(
        aggregate_function: duckdb_aggregate_function,
        name: *const ::std::os::raw::c_char,
    ) |  |  |  |  |

| `duckdb_aggregate_function_set_return_type` | function | duckdb_aggregate_function_set_return_type(
        aggregate_function: duckdb_aggregate_function,
        type_: duckdb_logical_type,
    ) |  |  |  |  |

| `duckdb_aggregate_function_set_special_handling` | function | duckdb_aggregate_function_set_special_handling(aggregate_function: duckdb_aggregate_function) |  |  |  |  |

| `duckdb_append_blob` | function | duckdb_append_blob(
        appender: duckdb_appender,
        data: *const ::std::os::raw::c_void,
        length: idx_t,
    ) |  |  |  |  |

| `duckdb_append_bool` | function | duckdb_append_bool(appender: duckdb_appender, value: bool) |  |  |  |  |

| `duckdb_append_data_chunk` | function | duckdb_append_data_chunk(appender: duckdb_appender, chunk: duckdb_data_chunk) |  |  |  |  |

| `duckdb_append_date` | function | duckdb_append_date(appender: duckdb_appender, value: duckdb_date) |  |  |  |  |

| `duckdb_append_default` | function | duckdb_append_default(appender: duckdb_appender) |  |  |  |  |

| `duckdb_append_default_to_chunk` | function | duckdb_append_default_to_chunk(
        appender: duckdb_appender,
        chunk: duckdb_data_chunk,
        col: idx_t,
        row: idx_t,
    ) |  |  |  |  |

| `duckdb_append_double` | function | duckdb_append_double(appender: duckdb_appender, value: f64) |  |  |  |  |

| `duckdb_append_float` | function | duckdb_append_float(appender: duckdb_appender, value: f32) |  |  |  |  |

| `duckdb_append_hugeint` | function | duckdb_append_hugeint(appender: duckdb_appender, value: duckdb_hugeint) |  |  |  |  |

| `duckdb_append_int16` | function | duckdb_append_int16(appender: duckdb_appender, value: i16) |  |  |  |  |

| `duckdb_append_int32` | function | duckdb_append_int32(appender: duckdb_appender, value: i32) |  |  |  |  |

| `duckdb_append_int64` | function | duckdb_append_int64(appender: duckdb_appender, value: i64) |  |  |  |  |

| `duckdb_append_int8` | function | duckdb_append_int8(appender: duckdb_appender, value: i8) |  |  |  |  |

| `duckdb_append_interval` | function | duckdb_append_interval(appender: duckdb_appender, value: duckdb_interval) |  |  |  |  |

| `duckdb_append_null` | function | duckdb_append_null(appender: duckdb_appender) |  |  |  |  |

| `duckdb_append_time` | function | duckdb_append_time(appender: duckdb_appender, value: duckdb_time) |  |  |  |  |

| `duckdb_append_timestamp` | function | duckdb_append_timestamp(appender: duckdb_appender, value: duckdb_timestamp) |  |  |  |  |

| `duckdb_append_uhugeint` | function | duckdb_append_uhugeint(appender: duckdb_appender, value: duckdb_uhugeint) |  |  |  |  |

| `duckdb_append_uint16` | function | duckdb_append_uint16(appender: duckdb_appender, value: u16) |  |  |  |  |

| `duckdb_append_uint32` | function | duckdb_append_uint32(appender: duckdb_appender, value: u32) |  |  |  |  |

| `duckdb_append_uint64` | function | duckdb_append_uint64(appender: duckdb_appender, value: u64) |  |  |  |  |

| `duckdb_append_uint8` | function | duckdb_append_uint8(appender: duckdb_appender, value: u8) |  |  |  |  |

| `duckdb_append_value` | function | duckdb_append_value(appender: duckdb_appender, value: duckdb_value) |  |  |  |  |

| `duckdb_append_varchar` | function | duckdb_append_varchar(appender: duckdb_appender, val: *const ::std::os::raw::c_char) |  |  |  |  |

| `duckdb_append_varchar_length` | function | duckdb_append_varchar_length(
        appender: duckdb_appender,
        val: *const ::std::os::raw::c_char,
        length: idx_t,
    ) |  |  |  |  |

| `duckdb_appender_add_column` | function | duckdb_appender_add_column(appender: duckdb_appender, name: *const ::std::os::raw::c_char) |  |  |  |  |

| `duckdb_appender_begin_row` | function | duckdb_appender_begin_row(appender: duckdb_appender) |  |  |  |  |

| `duckdb_appender_clear_columns` | function | duckdb_appender_clear_columns(appender: duckdb_appender) |  |  |  |  |

| `duckdb_appender_close` | function | duckdb_appender_close(appender: duckdb_appender) |  |  |  |  |

| `duckdb_appender_column_count` | function | duckdb_appender_column_count(appender: duckdb_appender) |  |  |  |  |

| `duckdb_appender_column_type` | function | duckdb_appender_column_type(appender: duckdb_appender, col_idx: idx_t) |  |  |  |  |

| `duckdb_appender_create` | function | duckdb_appender_create(
        connection: duckdb_connection,
        schema: *const ::std::os::raw::c_char,
        table: *const ::std::os::raw::c_char,
        out_appender: *mut duckdb_appender,
    ) |  |  |  |  |

| `duckdb_appender_create_ext` | function | duckdb_appender_create_ext(
        connection: duckdb_connection,
        catalog: *const ::std::os::raw::c_char,
        schema: *const ::std::os::raw::c_char,
        table: *const ::std::os::raw::c_char,
        out_appender: *mut duckdb_appender,
    ) |  |  |  |  |

| `duckdb_appender_create_query` | function | duckdb_appender_create_query(
        connection: duckdb_connection,
        query: *const ::std::os::raw::c_char,
        column_count: idx_t,
        types: *mut duckdb_logical_type,
        table_name: *const ::std::os::raw::c_char,
        column_names: *mut *const ::std::os::raw::c_char,
        out_appender: *mut duckdb_appender,
    ) |  |  |  |  |

| `duckdb_appender_destroy` | function | duckdb_appender_destroy(appender: *mut duckdb_appender) |  |  |  |  |

| `duckdb_appender_end_row` | function | duckdb_appender_end_row(appender: duckdb_appender) |  |  |  |  |

| `duckdb_appender_error` | function | duckdb_appender_error(appender: duckdb_appender) |  |  |  |  |

| `duckdb_appender_error_data` | function | duckdb_appender_error_data(appender: duckdb_appender) |  |  |  |  |

| `duckdb_appender_flush` | function | duckdb_appender_flush(appender: duckdb_appender) |  |  |  |  |

| `duckdb_array_type_array_size` | function | duckdb_array_type_array_size(type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_array_type_child_type` | function | duckdb_array_type_child_type(type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_array_vector_get_child` | function | duckdb_array_vector_get_child(vector: duckdb_vector) |  |  |  |  |

| `duckdb_arrow_array_scan` | function | duckdb_arrow_array_scan(
        connection: duckdb_connection,
        table_name: *const ::std::os::raw::c_char,
        arrow_schema: duckdb_arrow_schema,
        arrow_array: duckdb_arrow_array,
        out_stream: *mut duckdb_arrow_stream,
    ) |  |  |  |  |

| `duckdb_arrow_column_count` | function | duckdb_arrow_column_count(result: duckdb_arrow) |  |  |  |  |

| `duckdb_arrow_row_count` | function | duckdb_arrow_row_count(result: duckdb_arrow) |  |  |  |  |

| `duckdb_arrow_rows_changed` | function | duckdb_arrow_rows_changed(result: duckdb_arrow) |  |  |  |  |

| `duckdb_arrow_scan` | function | duckdb_arrow_scan(
        connection: duckdb_connection,
        table_name: *const ::std::os::raw::c_char,
        arrow: duckdb_arrow_stream,
    ) |  |  |  |  |

| `duckdb_bind_add_result_column` | function | duckdb_bind_add_result_column(
        info: duckdb_bind_info,
        name: *const ::std::os::raw::c_char,
        type_: duckdb_logical_type,
    ) |  |  |  |  |

| `duckdb_bind_blob` | function | duckdb_bind_blob(
        prepared_statement: duckdb_prepared_statement,
        param_idx: idx_t,
        data: *const ::std::os::raw::c_void,
        length: idx_t,
    ) |  |  |  |  |

| `duckdb_bind_boolean` | function | duckdb_bind_boolean(
        prepared_statement: duckdb_prepared_statement,
        param_idx: idx_t,
        val: bool,
    ) |  |  |  |  |

| `duckdb_bind_date` | function | duckdb_bind_date(
        prepared_statement: duckdb_prepared_statement,
        param_idx: idx_t,
        val: duckdb_date,
    ) |  |  |  |  |

| `duckdb_bind_decimal` | function | duckdb_bind_decimal(
        prepared_statement: duckdb_prepared_statement,
        param_idx: idx_t,
        val: duckdb_decimal,
    ) |  |  |  |  |

| `duckdb_bind_double` | function | duckdb_bind_double(
        prepared_statement: duckdb_prepared_statement,
        param_idx: idx_t,
        val: f64,
    ) |  |  |  |  |

| `duckdb_bind_float` | function | duckdb_bind_float(prepared_statement: duckdb_prepared_statement, param_idx: idx_t, val: f32) |  |  |  |  |

| `duckdb_bind_get_extra_info` | function | duckdb_bind_get_extra_info(info: duckdb_bind_info) |  |  |  |  |

| `duckdb_bind_get_named_parameter` | function | duckdb_bind_get_named_parameter(info: duckdb_bind_info, name: *const ::std::os::raw::c_char) |  |  |  |  |

| `duckdb_bind_get_parameter` | function | duckdb_bind_get_parameter(info: duckdb_bind_info, index: idx_t) |  |  |  |  |

| `duckdb_bind_get_parameter_count` | function | duckdb_bind_get_parameter_count(info: duckdb_bind_info) |  |  |  |  |

| `duckdb_bind_hugeint` | function | duckdb_bind_hugeint(
        prepared_statement: duckdb_prepared_statement,
        param_idx: idx_t,
        val: duckdb_hugeint,
    ) |  |  |  |  |

| `duckdb_bind_int16` | function | duckdb_bind_int16(prepared_statement: duckdb_prepared_statement, param_idx: idx_t, val: i16) |  |  |  |  |

| `duckdb_bind_int32` | function | duckdb_bind_int32(prepared_statement: duckdb_prepared_statement, param_idx: idx_t, val: i32) |  |  |  |  |

| `duckdb_bind_int64` | function | duckdb_bind_int64(prepared_statement: duckdb_prepared_statement, param_idx: idx_t, val: i64) |  |  |  |  |

| `duckdb_bind_int8` | function | duckdb_bind_int8(prepared_statement: duckdb_prepared_statement, param_idx: idx_t, val: i8) |  |  |  |  |

| `duckdb_bind_interval` | function | duckdb_bind_interval(
        prepared_statement: duckdb_prepared_statement,
        param_idx: idx_t,
        val: duckdb_interval,
    ) |  |  |  |  |

| `duckdb_bind_null` | function | duckdb_bind_null(prepared_statement: duckdb_prepared_statement, param_idx: idx_t) |  |  |  |  |

| `duckdb_bind_parameter_index` | function | duckdb_bind_parameter_index(
        prepared_statement: duckdb_prepared_statement,
        param_idx_out: *mut idx_t,
        name: *const ::std::os::raw::c_char,
    ) |  |  |  |  |

| `duckdb_bind_set_bind_data` | function | duckdb_bind_set_bind_data(
        info: duckdb_bind_info,
        bind_data: *mut ::std::os::raw::c_void,
        destroy: duckdb_delete_callback_t,
    ) |  |  |  |  |

| `duckdb_bind_set_cardinality` | function | duckdb_bind_set_cardinality(info: duckdb_bind_info, cardinality: idx_t, is_exact: bool) |  |  |  |  |

| `duckdb_bind_set_error` | function | duckdb_bind_set_error(info: duckdb_bind_info, error: *const ::std::os::raw::c_char) |  |  |  |  |

| `duckdb_bind_time` | function | duckdb_bind_time(
        prepared_statement: duckdb_prepared_statement,
        param_idx: idx_t,
        val: duckdb_time,
    ) |  |  |  |  |

| `duckdb_bind_timestamp` | function | duckdb_bind_timestamp(
        prepared_statement: duckdb_prepared_statement,
        param_idx: idx_t,
        val: duckdb_timestamp,
    ) |  |  |  |  |

| `duckdb_bind_timestamp_tz` | function | duckdb_bind_timestamp_tz(
        prepared_statement: duckdb_prepared_statement,
        param_idx: idx_t,
        val: duckdb_timestamp,
    ) |  |  |  |  |

| `duckdb_bind_uhugeint` | function | duckdb_bind_uhugeint(
        prepared_statement: duckdb_prepared_statement,
        param_idx: idx_t,
        val: duckdb_uhugeint,
    ) |  |  |  |  |

| `duckdb_bind_uint16` | function | duckdb_bind_uint16(
        prepared_statement: duckdb_prepared_statement,
        param_idx: idx_t,
        val: u16,
    ) |  |  |  |  |

| `duckdb_bind_uint32` | function | duckdb_bind_uint32(
        prepared_statement: duckdb_prepared_statement,
        param_idx: idx_t,
        val: u32,
    ) |  |  |  |  |

| `duckdb_bind_uint64` | function | duckdb_bind_uint64(
        prepared_statement: duckdb_prepared_statement,
        param_idx: idx_t,
        val: u64,
    ) |  |  |  |  |

| `duckdb_bind_uint8` | function | duckdb_bind_uint8(prepared_statement: duckdb_prepared_statement, param_idx: idx_t, val: u8) |  |  |  |  |

| `duckdb_bind_value` | function | duckdb_bind_value(
        prepared_statement: duckdb_prepared_statement,
        param_idx: idx_t,
        val: duckdb_value,
    ) |  |  |  |  |

| `duckdb_bind_varchar` | function | duckdb_bind_varchar(
        prepared_statement: duckdb_prepared_statement,
        param_idx: idx_t,
        val: *const ::std::os::raw::c_char,
    ) |  |  |  |  |

| `duckdb_bind_varchar_length` | function | duckdb_bind_varchar_length(
        prepared_statement: duckdb_prepared_statement,
        param_idx: idx_t,
        val: *const ::std::os::raw::c_char,
        length: idx_t,
    ) |  |  |  |  |

| `duckdb_cast_function_get_cast_mode` | function | duckdb_cast_function_get_cast_mode(info: duckdb_function_info) |  |  |  |  |

| `duckdb_cast_function_get_extra_info` | function | duckdb_cast_function_get_extra_info(info: duckdb_function_info) |  |  |  |  |

| `duckdb_cast_function_set_error` | function | duckdb_cast_function_set_error(info: duckdb_function_info, error: *const ::std::os::raw::c_char) |  |  |  |  |

| `duckdb_cast_function_set_extra_info` | function | duckdb_cast_function_set_extra_info(
        cast_function: duckdb_cast_function,
        extra_info: *mut ::std::os::raw::c_void,
        destroy: duckdb_delete_callback_t,
    ) |  |  |  |  |

| `duckdb_cast_function_set_function` | function | duckdb_cast_function_set_function(cast_function: duckdb_cast_function, function: duckdb_cast_function_t) |  |  |  |  |

| `duckdb_cast_function_set_implicit_cast_cost` | function | duckdb_cast_function_set_implicit_cast_cost(cast_function: duckdb_cast_function, cost: i64) |  |  |  |  |

| `duckdb_cast_function_set_row_error` | function | duckdb_cast_function_set_row_error(
        info: duckdb_function_info,
        error: *const ::std::os::raw::c_char,
        row: idx_t,
        output: duckdb_vector,
    ) |  |  |  |  |

| `duckdb_cast_function_set_source_type` | function | duckdb_cast_function_set_source_type(cast_function: duckdb_cast_function, source_type: duckdb_logical_type) |  |  |  |  |

| `duckdb_cast_function_set_target_type` | function | duckdb_cast_function_set_target_type(cast_function: duckdb_cast_function, target_type: duckdb_logical_type) |  |  |  |  |

| `duckdb_clear_bindings` | function | duckdb_clear_bindings(prepared_statement: duckdb_prepared_statement) |  |  |  |  |

| `duckdb_client_context_get_connection_id` | function | duckdb_client_context_get_connection_id(context: duckdb_client_context) |  |  |  |  |

| `duckdb_close` | function | duckdb_close(database: *mut duckdb_database) |  |  |  |  |

| `duckdb_column_count` | function | duckdb_column_count(result: *mut duckdb_result) |  |  |  |  |

| `duckdb_column_data` | function | duckdb_column_data(result: *mut duckdb_result, col: idx_t) |  |  |  |  |

| `duckdb_column_has_default` | function | duckdb_column_has_default(
        table_description: duckdb_table_description,
        index: idx_t,
        out: *mut bool,
    ) |  |  |  |  |

| `duckdb_column_logical_type` | function | duckdb_column_logical_type(result: *mut duckdb_result, col: idx_t) |  |  |  |  |

| `duckdb_column_name` | function | duckdb_column_name(result: *mut duckdb_result, col: idx_t) |  |  |  |  |

| `duckdb_column_type` | function | duckdb_column_type(result: *mut duckdb_result, col: idx_t) |  |  |  |  |

| `duckdb_config_count` | function | duckdb_config_count() |  |  |  |  |

| `duckdb_connect` | function | duckdb_connect(database: duckdb_database, out_connection: *mut duckdb_connection) |  |  |  |  |

| `duckdb_connection_get_arrow_options` | function | duckdb_connection_get_arrow_options(
        connection: duckdb_connection,
        out_arrow_options: *mut duckdb_arrow_options,
    ) |  |  |  |  |

| `duckdb_connection_get_client_context` | function | duckdb_connection_get_client_context(connection: duckdb_connection, out_context: *mut duckdb_client_context) |  |  |  |  |

| `duckdb_create_aggregate_function` | function | duckdb_create_aggregate_function() |  |  |  |  |

| `duckdb_create_aggregate_function_set` | function | duckdb_create_aggregate_function_set(name: *const ::std::os::raw::c_char) |  |  |  |  |

| `duckdb_create_array_type` | function | duckdb_create_array_type(type_: duckdb_logical_type, array_size: idx_t) |  |  |  |  |

| `duckdb_create_array_value` | function | duckdb_create_array_value(
        type_: duckdb_logical_type,
        values: *mut duckdb_value,
        value_count: idx_t,
    ) |  |  |  |  |

| `duckdb_create_bignum` | function | duckdb_create_bignum(input: duckdb_bignum) |  |  |  |  |

| `duckdb_create_bit` | function | duckdb_create_bit(input: duckdb_bit) |  |  |  |  |

| `duckdb_create_blob` | function | duckdb_create_blob(data: *const u8, length: idx_t) |  |  |  |  |

| `duckdb_create_bool` | function | duckdb_create_bool(input: bool) |  |  |  |  |

| `duckdb_create_cast_function` | function | duckdb_create_cast_function() |  |  |  |  |

| `duckdb_create_config` | function | duckdb_create_config(out_config: *mut duckdb_config) |  |  |  |  |

| `duckdb_create_data_chunk` | function | duckdb_create_data_chunk(types: *mut duckdb_logical_type, column_count: idx_t) |  |  |  |  |

| `duckdb_create_date` | function | duckdb_create_date(input: duckdb_date) |  |  |  |  |

| `duckdb_create_decimal` | function | duckdb_create_decimal(input: duckdb_decimal) |  |  |  |  |

| `duckdb_create_decimal_type` | function | duckdb_create_decimal_type(width: u8, scale: u8) |  |  |  |  |

| `duckdb_create_double` | function | duckdb_create_double(input: f64) |  |  |  |  |

| `duckdb_create_enum_type` | function | duckdb_create_enum_type(
        member_names: *mut *const ::std::os::raw::c_char,
        member_count: idx_t,
    ) |  |  |  |  |

| `duckdb_create_enum_value` | function | duckdb_create_enum_value(type_: duckdb_logical_type, value: u64) |  |  |  |  |

| `duckdb_create_error_data` | function | duckdb_create_error_data(
        type_: duckdb_error_type,
        message: *const ::std::os::raw::c_char,
    ) |  |  |  |  |

| `duckdb_create_float` | function | duckdb_create_float(input: f32) |  |  |  |  |

| `duckdb_create_hugeint` | function | duckdb_create_hugeint(input: duckdb_hugeint) |  |  |  |  |

| `duckdb_create_instance_cache` | function | duckdb_create_instance_cache() |  |  |  |  |

| `duckdb_create_int16` | function | duckdb_create_int16(input: i16) |  |  |  |  |

| `duckdb_create_int32` | function | duckdb_create_int32(input: i32) |  |  |  |  |

| `duckdb_create_int64` | function | duckdb_create_int64(val: i64) |  |  |  |  |

| `duckdb_create_int8` | function | duckdb_create_int8(input: i8) |  |  |  |  |

| `duckdb_create_interval` | function | duckdb_create_interval(input: duckdb_interval) |  |  |  |  |

| `duckdb_create_list_type` | function | duckdb_create_list_type(type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_create_list_value` | function | duckdb_create_list_value(
        type_: duckdb_logical_type,
        values: *mut duckdb_value,
        value_count: idx_t,
    ) |  |  |  |  |

| `duckdb_create_logical_type` | function | duckdb_create_logical_type(type_: duckdb_type) |  |  |  |  |

| `duckdb_create_map_type` | function | duckdb_create_map_type(
        key_type: duckdb_logical_type,
        value_type: duckdb_logical_type,
    ) |  |  |  |  |

| `duckdb_create_map_value` | function | duckdb_create_map_value(
        map_type: duckdb_logical_type,
        keys: *mut duckdb_value,
        values: *mut duckdb_value,
        entry_count: idx_t,
    ) |  |  |  |  |

| `duckdb_create_null_value` | function | duckdb_create_null_value() |  |  |  |  |

| `duckdb_create_scalar_function` | function | duckdb_create_scalar_function() |  |  |  |  |

| `duckdb_create_scalar_function_set` | function | duckdb_create_scalar_function_set(name: *const ::std::os::raw::c_char) |  |  |  |  |

| `duckdb_create_selection_vector` | function | duckdb_create_selection_vector(size: idx_t) |  |  |  |  |

| `duckdb_create_struct_type` | function | duckdb_create_struct_type(
        member_types: *mut duckdb_logical_type,
        member_names: *mut *const ::std::os::raw::c_char,
        member_count: idx_t,
    ) |  |  |  |  |

| `duckdb_create_struct_value` | function | duckdb_create_struct_value(type_: duckdb_logical_type, values: *mut duckdb_value) |  |  |  |  |

| `duckdb_create_table_function` | function | duckdb_create_table_function() |  |  |  |  |

| `duckdb_create_task_state` | function | duckdb_create_task_state(database: duckdb_database) |  |  |  |  |

| `duckdb_create_time` | function | duckdb_create_time(input: duckdb_time) |  |  |  |  |

| `duckdb_create_time_ns` | function | duckdb_create_time_ns(input: duckdb_time_ns) |  |  |  |  |

| `duckdb_create_time_tz` | function | duckdb_create_time_tz(micros: i64, offset: i32) |  |  |  |  |

| `duckdb_create_time_tz_value` | function | duckdb_create_time_tz_value(value: duckdb_time_tz) |  |  |  |  |

| `duckdb_create_timestamp` | function | duckdb_create_timestamp(input: duckdb_timestamp) |  |  |  |  |

| `duckdb_create_timestamp_ms` | function | duckdb_create_timestamp_ms(input: duckdb_timestamp_ms) |  |  |  |  |

| `duckdb_create_timestamp_ns` | function | duckdb_create_timestamp_ns(input: duckdb_timestamp_ns) |  |  |  |  |

| `duckdb_create_timestamp_s` | function | duckdb_create_timestamp_s(input: duckdb_timestamp_s) |  |  |  |  |

| `duckdb_create_timestamp_tz` | function | duckdb_create_timestamp_tz(input: duckdb_timestamp) |  |  |  |  |

| `duckdb_create_uhugeint` | function | duckdb_create_uhugeint(input: duckdb_uhugeint) |  |  |  |  |

| `duckdb_create_uint16` | function | duckdb_create_uint16(input: u16) |  |  |  |  |

| `duckdb_create_uint32` | function | duckdb_create_uint32(input: u32) |  |  |  |  |

| `duckdb_create_uint64` | function | duckdb_create_uint64(input: u64) |  |  |  |  |

| `duckdb_create_uint8` | function | duckdb_create_uint8(input: u8) |  |  |  |  |

| `duckdb_create_union_type` | function | duckdb_create_union_type(
        member_types: *mut duckdb_logical_type,
        member_names: *mut *const ::std::os::raw::c_char,
        member_count: idx_t,
    ) |  |  |  |  |

| `duckdb_create_union_value` | function | duckdb_create_union_value(
        union_type: duckdb_logical_type,
        tag_index: idx_t,
        value: duckdb_value,
    ) |  |  |  |  |

| `duckdb_create_uuid` | function | duckdb_create_uuid(input: duckdb_uhugeint) |  |  |  |  |

| `duckdb_create_varchar` | function | duckdb_create_varchar(text: *const ::std::os::raw::c_char) |  |  |  |  |

| `duckdb_create_varchar_length` | function | duckdb_create_varchar_length(text: *const ::std::os::raw::c_char, length: idx_t) |  |  |  |  |

| `duckdb_create_vector` | function | duckdb_create_vector(type_: duckdb_logical_type, capacity: idx_t) |  |  |  |  |

| `duckdb_data_chunk_from_arrow` | function | duckdb_data_chunk_from_arrow(
        connection: duckdb_connection,
        arrow_array: *mut ArrowArray,
        converted_schema: duckdb_arrow_converted_schema,
        out_chunk: *mut duckdb_data_chunk,
    ) |  |  |  |  |

| `duckdb_data_chunk_get_column_count` | function | duckdb_data_chunk_get_column_count(chunk: duckdb_data_chunk) |  |  |  |  |

| `duckdb_data_chunk_get_size` | function | duckdb_data_chunk_get_size(chunk: duckdb_data_chunk) |  |  |  |  |

| `duckdb_data_chunk_get_vector` | function | duckdb_data_chunk_get_vector(chunk: duckdb_data_chunk, col_idx: idx_t) |  |  |  |  |

| `duckdb_data_chunk_reset` | function | duckdb_data_chunk_reset(chunk: duckdb_data_chunk) |  |  |  |  |

| `duckdb_data_chunk_set_size` | function | duckdb_data_chunk_set_size(chunk: duckdb_data_chunk, size: idx_t) |  |  |  |  |

| `duckdb_data_chunk_to_arrow` | function | duckdb_data_chunk_to_arrow(
        arrow_options: duckdb_arrow_options,
        chunk: duckdb_data_chunk,
        out_arrow_array: *mut ArrowArray,
    ) |  |  |  |  |

| `duckdb_decimal_internal_type` | function | duckdb_decimal_internal_type(type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_decimal_scale` | function | duckdb_decimal_scale(type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_decimal_to_double` | function | duckdb_decimal_to_double(val: duckdb_decimal) |  |  |  |  |

| `duckdb_decimal_width` | function | duckdb_decimal_width(type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_destroy_aggregate_function` | function | duckdb_destroy_aggregate_function(aggregate_function: *mut duckdb_aggregate_function) |  |  |  |  |

| `duckdb_destroy_aggregate_function_set` | function | duckdb_destroy_aggregate_function_set(aggregate_function_set: *mut duckdb_aggregate_function_set) |  |  |  |  |

| `duckdb_destroy_arrow` | function | duckdb_destroy_arrow(result: *mut duckdb_arrow) |  |  |  |  |

| `duckdb_destroy_arrow_converted_schema` | function | duckdb_destroy_arrow_converted_schema(arrow_converted_schema: *mut duckdb_arrow_converted_schema) |  |  |  |  |

| `duckdb_destroy_arrow_options` | function | duckdb_destroy_arrow_options(arrow_options: *mut duckdb_arrow_options) |  |  |  |  |

| `duckdb_destroy_arrow_stream` | function | duckdb_destroy_arrow_stream(stream_p: *mut duckdb_arrow_stream) |  |  |  |  |

| `duckdb_destroy_cast_function` | function | duckdb_destroy_cast_function(cast_function: *mut duckdb_cast_function) |  |  |  |  |

| `duckdb_destroy_client_context` | function | duckdb_destroy_client_context(context: *mut duckdb_client_context) |  |  |  |  |

| `duckdb_destroy_config` | function | duckdb_destroy_config(config: *mut duckdb_config) |  |  |  |  |

| `duckdb_destroy_data_chunk` | function | duckdb_destroy_data_chunk(chunk: *mut duckdb_data_chunk) |  |  |  |  |

| `duckdb_destroy_error_data` | function | duckdb_destroy_error_data(error_data: *mut duckdb_error_data) |  |  |  |  |

| `duckdb_destroy_expression` | function | duckdb_destroy_expression(expr: *mut duckdb_expression) |  |  |  |  |

| `duckdb_destroy_extracted` | function | duckdb_destroy_extracted(extracted_statements: *mut duckdb_extracted_statements) |  |  |  |  |

| `duckdb_destroy_instance_cache` | function | duckdb_destroy_instance_cache(instance_cache: *mut duckdb_instance_cache) |  |  |  |  |

| `duckdb_destroy_logical_type` | function | duckdb_destroy_logical_type(type_: *mut duckdb_logical_type) |  |  |  |  |

| `duckdb_destroy_pending` | function | duckdb_destroy_pending(pending_result: *mut duckdb_pending_result) |  |  |  |  |

| `duckdb_destroy_prepare` | function | duckdb_destroy_prepare(prepared_statement: *mut duckdb_prepared_statement) |  |  |  |  |

| `duckdb_destroy_result` | function | duckdb_destroy_result(result: *mut duckdb_result) |  |  |  |  |

| `duckdb_destroy_scalar_function` | function | duckdb_destroy_scalar_function(scalar_function: *mut duckdb_scalar_function) |  |  |  |  |

| `duckdb_destroy_scalar_function_set` | function | duckdb_destroy_scalar_function_set(scalar_function_set: *mut duckdb_scalar_function_set) |  |  |  |  |

| `duckdb_destroy_selection_vector` | function | duckdb_destroy_selection_vector(sel: duckdb_selection_vector) |  |  |  |  |

| `duckdb_destroy_table_function` | function | duckdb_destroy_table_function(table_function: *mut duckdb_table_function) |  |  |  |  |

| `duckdb_destroy_task_state` | function | duckdb_destroy_task_state(state: duckdb_task_state) |  |  |  |  |

| `duckdb_destroy_value` | function | duckdb_destroy_value(value: *mut duckdb_value) |  |  |  |  |

| `duckdb_destroy_vector` | function | duckdb_destroy_vector(vector: *mut duckdb_vector) |  |  |  |  |

| `duckdb_disconnect` | function | duckdb_disconnect(connection: *mut duckdb_connection) |  |  |  |  |

| `duckdb_double_to_decimal` | function | duckdb_double_to_decimal(val: f64, width: u8, scale: u8) |  |  |  |  |

| `duckdb_double_to_hugeint` | function | duckdb_double_to_hugeint(val: f64) |  |  |  |  |

| `duckdb_double_to_uhugeint` | function | duckdb_double_to_uhugeint(val: f64) |  |  |  |  |

| `duckdb_enum_dictionary_size` | function | duckdb_enum_dictionary_size(type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_enum_dictionary_value` | function | duckdb_enum_dictionary_value(type_: duckdb_logical_type, index: idx_t) |  |  |  |  |

| `duckdb_enum_internal_type` | function | duckdb_enum_internal_type(type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_error_data_error_type` | function | duckdb_error_data_error_type(error_data: duckdb_error_data) |  |  |  |  |

| `duckdb_error_data_has_error` | function | duckdb_error_data_has_error(error_data: duckdb_error_data) |  |  |  |  |

| `duckdb_error_data_message` | function | duckdb_error_data_message(error_data: duckdb_error_data) |  |  |  |  |

| `duckdb_execute_n_tasks_state` | function | duckdb_execute_n_tasks_state(state: duckdb_task_state, max_tasks: idx_t) |  |  |  |  |

| `duckdb_execute_pending` | function | duckdb_execute_pending(
        pending_result: duckdb_pending_result,
        out_result: *mut duckdb_result,
    ) |  |  |  |  |

| `duckdb_execute_prepared` | function | duckdb_execute_prepared(
        prepared_statement: duckdb_prepared_statement,
        out_result: *mut duckdb_result,
    ) |  |  |  |  |

| `duckdb_execute_prepared_arrow` | function | duckdb_execute_prepared_arrow(
        prepared_statement: duckdb_prepared_statement,
        out_result: *mut duckdb_arrow,
    ) |  |  |  |  |

| `duckdb_execute_prepared_streaming` | function | duckdb_execute_prepared_streaming(
        prepared_statement: duckdb_prepared_statement,
        out_result: *mut duckdb_result,
    ) |  |  |  |  |

| `duckdb_execute_tasks` | function | duckdb_execute_tasks(database: duckdb_database, max_tasks: idx_t) |  |  |  |  |

| `duckdb_execute_tasks_state` | function | duckdb_execute_tasks_state(state: duckdb_task_state) |  |  |  |  |

| `duckdb_execution_is_finished` | function | duckdb_execution_is_finished(con: duckdb_connection) |  |  |  |  |

| `duckdb_expression_fold` | function | duckdb_expression_fold(
        context: duckdb_client_context,
        expr: duckdb_expression,
        out_value: *mut duckdb_value,
    ) |  |  |  |  |

| `duckdb_expression_is_foldable` | function | duckdb_expression_is_foldable(expr: duckdb_expression) |  |  |  |  |

| `duckdb_expression_return_type` | function | duckdb_expression_return_type(expr: duckdb_expression) |  |  |  |  |

| `duckdb_extract_statements` | function | duckdb_extract_statements(
        connection: duckdb_connection,
        query: *const ::std::os::raw::c_char,
        out_extracted_statements: *mut duckdb_extracted_statements,
    ) |  |  |  |  |

| `duckdb_extract_statements_error` | function | duckdb_extract_statements_error(
        extracted_statements: duckdb_extracted_statements,
    ) |  |  |  |  |

| `duckdb_fetch_chunk` | function | duckdb_fetch_chunk(result: duckdb_result) |  |  |  |  |

| `duckdb_finish_execution` | function | duckdb_finish_execution(state: duckdb_task_state) |  |  |  |  |

| `duckdb_free` | function | duckdb_free(ptr: *mut ::std::os::raw::c_void) |  |  |  |  |

| `duckdb_from_date` | function | duckdb_from_date(date: duckdb_date) |  |  |  |  |

| `duckdb_from_time` | function | duckdb_from_time(time: duckdb_time) |  |  |  |  |

| `duckdb_from_time_tz` | function | duckdb_from_time_tz(micros: duckdb_time_tz) |  |  |  |  |

| `duckdb_from_timestamp` | function | duckdb_from_timestamp(ts: duckdb_timestamp) |  |  |  |  |

| `duckdb_function_get_bind_data` | function | duckdb_function_get_bind_data(info: duckdb_function_info) |  |  |  |  |

| `duckdb_function_get_extra_info` | function | duckdb_function_get_extra_info(info: duckdb_function_info) |  |  |  |  |

| `duckdb_function_get_init_data` | function | duckdb_function_get_init_data(info: duckdb_function_info) |  |  |  |  |

| `duckdb_function_get_local_init_data` | function | duckdb_function_get_local_init_data(info: duckdb_function_info) |  |  |  |  |

| `duckdb_function_set_error` | function | duckdb_function_set_error(info: duckdb_function_info, error: *const ::std::os::raw::c_char) |  |  |  |  |

| `duckdb_get_bignum` | function | duckdb_get_bignum(val: duckdb_value) |  |  |  |  |

| `duckdb_get_bit` | function | duckdb_get_bit(val: duckdb_value) |  |  |  |  |

| `duckdb_get_blob` | function | duckdb_get_blob(val: duckdb_value) |  |  |  |  |

| `duckdb_get_bool` | function | duckdb_get_bool(val: duckdb_value) |  |  |  |  |

| `duckdb_get_config_flag` | function | duckdb_get_config_flag(
        index: usize,
        out_name: *mut *const ::std::os::raw::c_char,
        out_description: *mut *const ::std::os::raw::c_char,
    ) |  |  |  |  |

| `duckdb_get_date` | function | duckdb_get_date(val: duckdb_value) |  |  |  |  |

| `duckdb_get_decimal` | function | duckdb_get_decimal(val: duckdb_value) |  |  |  |  |

| `duckdb_get_double` | function | duckdb_get_double(val: duckdb_value) |  |  |  |  |

| `duckdb_get_enum_value` | function | duckdb_get_enum_value(value: duckdb_value) |  |  |  |  |

| `duckdb_get_float` | function | duckdb_get_float(val: duckdb_value) |  |  |  |  |

| `duckdb_get_hugeint` | function | duckdb_get_hugeint(val: duckdb_value) |  |  |  |  |

| `duckdb_get_int16` | function | duckdb_get_int16(val: duckdb_value) |  |  |  |  |

| `duckdb_get_int32` | function | duckdb_get_int32(val: duckdb_value) |  |  |  |  |

| `duckdb_get_int64` | function | duckdb_get_int64(val: duckdb_value) |  |  |  |  |

| `duckdb_get_int8` | function | duckdb_get_int8(val: duckdb_value) |  |  |  |  |

| `duckdb_get_interval` | function | duckdb_get_interval(val: duckdb_value) |  |  |  |  |

| `duckdb_get_list_child` | function | duckdb_get_list_child(value: duckdb_value, index: idx_t) |  |  |  |  |

| `duckdb_get_list_size` | function | duckdb_get_list_size(value: duckdb_value) |  |  |  |  |

| `duckdb_get_map_key` | function | duckdb_get_map_key(value: duckdb_value, index: idx_t) |  |  |  |  |

| `duckdb_get_map_size` | function | duckdb_get_map_size(value: duckdb_value) |  |  |  |  |

| `duckdb_get_map_value` | function | duckdb_get_map_value(value: duckdb_value, index: idx_t) |  |  |  |  |

| `duckdb_get_or_create_from_cache` | function | duckdb_get_or_create_from_cache(
        instance_cache: duckdb_instance_cache,
        path: *const ::std::os::raw::c_char,
        out_database: *mut duckdb_database,
        config: duckdb_config,
        out_error: *mut *mut ::std::os::raw::c_char,
    ) |  |  |  |  |

| `duckdb_get_profiling_info` | function | duckdb_get_profiling_info(connection: duckdb_connection) |  |  |  |  |

| `duckdb_get_struct_child` | function | duckdb_get_struct_child(value: duckdb_value, index: idx_t) |  |  |  |  |

| `duckdb_get_table_names` | function | duckdb_get_table_names(
        connection: duckdb_connection,
        query: *const ::std::os::raw::c_char,
        qualified: bool,
    ) |  |  |  |  |

| `duckdb_get_time` | function | duckdb_get_time(val: duckdb_value) |  |  |  |  |

| `duckdb_get_time_ns` | function | duckdb_get_time_ns(val: duckdb_value) |  |  |  |  |

| `duckdb_get_time_tz` | function | duckdb_get_time_tz(val: duckdb_value) |  |  |  |  |

| `duckdb_get_timestamp` | function | duckdb_get_timestamp(val: duckdb_value) |  |  |  |  |

| `duckdb_get_timestamp_ms` | function | duckdb_get_timestamp_ms(val: duckdb_value) |  |  |  |  |

| `duckdb_get_timestamp_ns` | function | duckdb_get_timestamp_ns(val: duckdb_value) |  |  |  |  |

| `duckdb_get_timestamp_s` | function | duckdb_get_timestamp_s(val: duckdb_value) |  |  |  |  |

| `duckdb_get_timestamp_tz` | function | duckdb_get_timestamp_tz(val: duckdb_value) |  |  |  |  |

| `duckdb_get_type_id` | function | duckdb_get_type_id(type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_get_uhugeint` | function | duckdb_get_uhugeint(val: duckdb_value) |  |  |  |  |

| `duckdb_get_uint16` | function | duckdb_get_uint16(val: duckdb_value) |  |  |  |  |

| `duckdb_get_uint32` | function | duckdb_get_uint32(val: duckdb_value) |  |  |  |  |

| `duckdb_get_uint64` | function | duckdb_get_uint64(val: duckdb_value) |  |  |  |  |

| `duckdb_get_uint8` | function | duckdb_get_uint8(val: duckdb_value) |  |  |  |  |

| `duckdb_get_uuid` | function | duckdb_get_uuid(val: duckdb_value) |  |  |  |  |

| `duckdb_get_value_type` | function | duckdb_get_value_type(val: duckdb_value) |  |  |  |  |

| `duckdb_get_varchar` | function | duckdb_get_varchar(value: duckdb_value) |  |  |  |  |

| `duckdb_hugeint_to_double` | function | duckdb_hugeint_to_double(val: duckdb_hugeint) |  |  |  |  |

| `duckdb_init_get_bind_data` | function | duckdb_init_get_bind_data(info: duckdb_init_info) |  |  |  |  |

| `duckdb_init_get_column_count` | function | duckdb_init_get_column_count(info: duckdb_init_info) |  |  |  |  |

| `duckdb_init_get_column_index` | function | duckdb_init_get_column_index(info: duckdb_init_info, column_index: idx_t) |  |  |  |  |

| `duckdb_init_get_extra_info` | function | duckdb_init_get_extra_info(info: duckdb_init_info) |  |  |  |  |

| `duckdb_init_set_error` | function | duckdb_init_set_error(info: duckdb_init_info, error: *const ::std::os::raw::c_char) |  |  |  |  |

| `duckdb_init_set_init_data` | function | duckdb_init_set_init_data(
        info: duckdb_init_info,
        init_data: *mut ::std::os::raw::c_void,
        destroy: duckdb_delete_callback_t,
    ) |  |  |  |  |

| `duckdb_init_set_max_threads` | function | duckdb_init_set_max_threads(info: duckdb_init_info, max_threads: idx_t) |  |  |  |  |

| `duckdb_interrupt` | function | duckdb_interrupt(connection: duckdb_connection) |  |  |  |  |

| `duckdb_is_finite_date` | function | duckdb_is_finite_date(date: duckdb_date) |  |  |  |  |

| `duckdb_is_finite_timestamp` | function | duckdb_is_finite_timestamp(ts: duckdb_timestamp) |  |  |  |  |

| `duckdb_is_finite_timestamp_ms` | function | duckdb_is_finite_timestamp_ms(ts: duckdb_timestamp_ms) |  |  |  |  |

| `duckdb_is_finite_timestamp_ns` | function | duckdb_is_finite_timestamp_ns(ts: duckdb_timestamp_ns) |  |  |  |  |

| `duckdb_is_finite_timestamp_s` | function | duckdb_is_finite_timestamp_s(ts: duckdb_timestamp_s) |  |  |  |  |

| `duckdb_is_null_value` | function | duckdb_is_null_value(value: duckdb_value) |  |  |  |  |

| `duckdb_library_version` | function | duckdb_library_version() |  |  |  |  |

| `duckdb_list_type_child_type` | function | duckdb_list_type_child_type(type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_list_vector_get_child` | function | duckdb_list_vector_get_child(vector: duckdb_vector) |  |  |  |  |

| `duckdb_list_vector_get_size` | function | duckdb_list_vector_get_size(vector: duckdb_vector) |  |  |  |  |

| `duckdb_list_vector_reserve` | function | duckdb_list_vector_reserve(vector: duckdb_vector, required_capacity: idx_t) |  |  |  |  |

| `duckdb_list_vector_set_size` | function | duckdb_list_vector_set_size(vector: duckdb_vector, size: idx_t) |  |  |  |  |

| `duckdb_logical_type_get_alias` | function | duckdb_logical_type_get_alias(type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_logical_type_set_alias` | function | duckdb_logical_type_set_alias(type_: duckdb_logical_type, alias: *const ::std::os::raw::c_char) |  |  |  |  |

| `duckdb_malloc` | function | duckdb_malloc(size: usize) |  |  |  |  |

| `duckdb_map_type_key_type` | function | duckdb_map_type_key_type(type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_map_type_value_type` | function | duckdb_map_type_value_type(type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_nparams` | function | duckdb_nparams(prepared_statement: duckdb_prepared_statement) |  |  |  |  |

| `duckdb_nullmask_data` | function | duckdb_nullmask_data(result: *mut duckdb_result, col: idx_t) |  |  |  |  |

| `duckdb_open` | function | duckdb_open(path: *const ::std::os::raw::c_char, out_database: *mut duckdb_database) |  |  |  |  |

| `duckdb_open_ext` | function | duckdb_open_ext(
        path: *const ::std::os::raw::c_char,
        out_database: *mut duckdb_database,
        config: duckdb_config,
        out_error: *mut *mut ::std::os::raw::c_char,
    ) |  |  |  |  |

| `duckdb_param_logical_type` | function | duckdb_param_logical_type(
        prepared_statement: duckdb_prepared_statement,
        param_idx: idx_t,
    ) |  |  |  |  |

| `duckdb_param_type` | function | duckdb_param_type(prepared_statement: duckdb_prepared_statement, param_idx: idx_t) |  |  |  |  |

| `duckdb_parameter_name` | function | duckdb_parameter_name(
        prepared_statement: duckdb_prepared_statement,
        index: idx_t,
    ) |  |  |  |  |

| `duckdb_pending_error` | function | duckdb_pending_error(pending_result: duckdb_pending_result) |  |  |  |  |

| `duckdb_pending_execute_check_state` | function | duckdb_pending_execute_check_state(pending_result: duckdb_pending_result) |  |  |  |  |

| `duckdb_pending_execute_task` | function | duckdb_pending_execute_task(pending_result: duckdb_pending_result) |  |  |  |  |

| `duckdb_pending_execution_is_finished` | function | duckdb_pending_execution_is_finished(pending_state: duckdb_pending_state) |  |  |  |  |

| `duckdb_pending_prepared` | function | duckdb_pending_prepared(
        prepared_statement: duckdb_prepared_statement,
        out_result: *mut duckdb_pending_result,
    ) |  |  |  |  |

| `duckdb_pending_prepared_streaming` | function | duckdb_pending_prepared_streaming(
        prepared_statement: duckdb_prepared_statement,
        out_result: *mut duckdb_pending_result,
    ) |  |  |  |  |

| `duckdb_prepare` | function | duckdb_prepare(
        connection: duckdb_connection,
        query: *const ::std::os::raw::c_char,
        out_prepared_statement: *mut duckdb_prepared_statement,
    ) |  |  |  |  |

| `duckdb_prepare_error` | function | duckdb_prepare_error(prepared_statement: duckdb_prepared_statement) |  |  |  |  |

| `duckdb_prepare_extracted_statement` | function | duckdb_prepare_extracted_statement(
        connection: duckdb_connection,
        extracted_statements: duckdb_extracted_statements,
        index: idx_t,
        out_prepared_statement: *mut duckdb_prepared_statement,
    ) |  |  |  |  |

| `duckdb_prepared_arrow_schema` | function | duckdb_prepared_arrow_schema(
        prepared: duckdb_prepared_statement,
        out_schema: *mut duckdb_arrow_schema,
    ) |  |  |  |  |

| `duckdb_prepared_statement_column_count` | function | duckdb_prepared_statement_column_count(prepared_statement: duckdb_prepared_statement) |  |  |  |  |

| `duckdb_prepared_statement_column_logical_type` | function | duckdb_prepared_statement_column_logical_type(
        prepared_statement: duckdb_prepared_statement,
        col_idx: idx_t,
    ) |  |  |  |  |

| `duckdb_prepared_statement_column_name` | function | duckdb_prepared_statement_column_name(
        prepared_statement: duckdb_prepared_statement,
        col_idx: idx_t,
    ) |  |  |  |  |

| `duckdb_prepared_statement_column_type` | function | duckdb_prepared_statement_column_type(
        prepared_statement: duckdb_prepared_statement,
        col_idx: idx_t,
    ) |  |  |  |  |

| `duckdb_prepared_statement_type` | function | duckdb_prepared_statement_type(statement: duckdb_prepared_statement) |  |  |  |  |

| `duckdb_profiling_info_get_child` | function | duckdb_profiling_info_get_child(info: duckdb_profiling_info, index: idx_t) |  |  |  |  |

| `duckdb_profiling_info_get_child_count` | function | duckdb_profiling_info_get_child_count(info: duckdb_profiling_info) |  |  |  |  |

| `duckdb_profiling_info_get_metrics` | function | duckdb_profiling_info_get_metrics(info: duckdb_profiling_info) |  |  |  |  |

| `duckdb_profiling_info_get_value` | function | duckdb_profiling_info_get_value(
        info: duckdb_profiling_info,
        key: *const ::std::os::raw::c_char,
    ) |  |  |  |  |

| `duckdb_query` | function | duckdb_query(
        connection: duckdb_connection,
        query: *const ::std::os::raw::c_char,
        out_result: *mut duckdb_result,
    ) |  |  |  |  |

| `duckdb_query_arrow` | function | duckdb_query_arrow(
        connection: duckdb_connection,
        query: *const ::std::os::raw::c_char,
        out_result: *mut duckdb_arrow,
    ) |  |  |  |  |

| `duckdb_query_arrow_array` | function | duckdb_query_arrow_array(result: duckdb_arrow, out_array: *mut duckdb_arrow_array) |  |  |  |  |

| `duckdb_query_arrow_error` | function | duckdb_query_arrow_error(result: duckdb_arrow) |  |  |  |  |

| `duckdb_query_arrow_schema` | function | duckdb_query_arrow_schema(result: duckdb_arrow, out_schema: *mut duckdb_arrow_schema) |  |  |  |  |

| `duckdb_query_progress` | function | duckdb_query_progress(connection: duckdb_connection) |  |  |  |  |

| `duckdb_register_aggregate_function` | function | duckdb_register_aggregate_function(
        con: duckdb_connection,
        aggregate_function: duckdb_aggregate_function,
    ) |  |  |  |  |

| `duckdb_register_aggregate_function_set` | function | duckdb_register_aggregate_function_set(
        con: duckdb_connection,
        set: duckdb_aggregate_function_set,
    ) |  |  |  |  |

| `duckdb_register_cast_function` | function | duckdb_register_cast_function(con: duckdb_connection, cast_function: duckdb_cast_function) |  |  |  |  |

| `duckdb_register_logical_type` | function | duckdb_register_logical_type(
        con: duckdb_connection,
        type_: duckdb_logical_type,
        info: duckdb_create_type_info,
    ) |  |  |  |  |

| `duckdb_register_scalar_function` | function | duckdb_register_scalar_function(
        con: duckdb_connection,
        scalar_function: duckdb_scalar_function,
    ) |  |  |  |  |

| `duckdb_register_scalar_function_set` | function | duckdb_register_scalar_function_set(con: duckdb_connection, set: duckdb_scalar_function_set) |  |  |  |  |

| `duckdb_register_table_function` | function | duckdb_register_table_function(con: duckdb_connection, function: duckdb_table_function) |  |  |  |  |

| `duckdb_replacement_scan_add_parameter` | function | duckdb_replacement_scan_add_parameter(info: duckdb_replacement_scan_info, parameter: duckdb_value) |  |  |  |  |

| `duckdb_replacement_scan_set_error` | function | duckdb_replacement_scan_set_error(info: duckdb_replacement_scan_info, error: *const ::std::os::raw::c_char) |  |  |  |  |

| `duckdb_replacement_scan_set_function_name` | function | duckdb_replacement_scan_set_function_name(
        info: duckdb_replacement_scan_info,
        function_name: *const ::std::os::raw::c_char,
    ) |  |  |  |  |

| `duckdb_result_arrow_array` | function | duckdb_result_arrow_array(
        result: duckdb_result,
        chunk: duckdb_data_chunk,
        out_array: *mut duckdb_arrow_array,
    ) |  |  |  |  |

| `duckdb_result_chunk_count` | function | duckdb_result_chunk_count(result: duckdb_result) |  |  |  |  |

| `duckdb_result_error` | function | duckdb_result_error(result: *mut duckdb_result) |  |  |  |  |

| `duckdb_result_error_type` | function | duckdb_result_error_type(result: *mut duckdb_result) |  |  |  |  |

| `duckdb_result_get_arrow_options` | function | duckdb_result_get_arrow_options(result: *mut duckdb_result) |  |  |  |  |

| `duckdb_result_get_chunk` | function | duckdb_result_get_chunk(result: duckdb_result, chunk_index: idx_t) |  |  |  |  |

| `duckdb_result_is_streaming` | function | duckdb_result_is_streaming(result: duckdb_result) |  |  |  |  |

| `duckdb_result_return_type` | function | duckdb_result_return_type(result: duckdb_result) |  |  |  |  |

| `duckdb_result_statement_type` | function | duckdb_result_statement_type(result: duckdb_result) |  |  |  |  |

| `duckdb_row_count` | function | duckdb_row_count(result: *mut duckdb_result) |  |  |  |  |

| `duckdb_rows_changed` | function | duckdb_rows_changed(result: *mut duckdb_result) |  |  |  |  |

| `duckdb_scalar_function_add_parameter` | function | duckdb_scalar_function_add_parameter(scalar_function: duckdb_scalar_function, type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_scalar_function_bind_get_argument` | function | duckdb_scalar_function_bind_get_argument(info: duckdb_bind_info, index: idx_t) |  |  |  |  |

| `duckdb_scalar_function_bind_get_argument_count` | function | duckdb_scalar_function_bind_get_argument_count(info: duckdb_bind_info) |  |  |  |  |

| `duckdb_scalar_function_bind_get_extra_info` | function | duckdb_scalar_function_bind_get_extra_info(info: duckdb_bind_info) |  |  |  |  |

| `duckdb_scalar_function_bind_set_error` | function | duckdb_scalar_function_bind_set_error(info: duckdb_bind_info, error: *const ::std::os::raw::c_char) |  |  |  |  |

| `duckdb_scalar_function_get_bind_data` | function | duckdb_scalar_function_get_bind_data(info: duckdb_function_info) |  |  |  |  |

| `duckdb_scalar_function_get_client_context` | function | duckdb_scalar_function_get_client_context(info: duckdb_bind_info, out_context: *mut duckdb_client_context) |  |  |  |  |

| `duckdb_scalar_function_get_extra_info` | function | duckdb_scalar_function_get_extra_info(info: duckdb_function_info) |  |  |  |  |

| `duckdb_scalar_function_set_bind` | function | duckdb_scalar_function_set_bind(
        scalar_function: duckdb_scalar_function,
        bind: duckdb_scalar_function_bind_t,
    ) |  |  |  |  |

| `duckdb_scalar_function_set_bind_data` | function | duckdb_scalar_function_set_bind_data(
        info: duckdb_bind_info,
        bind_data: *mut ::std::os::raw::c_void,
        destroy: duckdb_delete_callback_t,
    ) |  |  |  |  |

| `duckdb_scalar_function_set_bind_data_copy` | function | duckdb_scalar_function_set_bind_data_copy(info: duckdb_bind_info, copy: duckdb_copy_callback_t) |  |  |  |  |

| `duckdb_scalar_function_set_error` | function | duckdb_scalar_function_set_error(info: duckdb_function_info, error: *const ::std::os::raw::c_char) |  |  |  |  |

| `duckdb_scalar_function_set_extra_info` | function | duckdb_scalar_function_set_extra_info(
        scalar_function: duckdb_scalar_function,
        extra_info: *mut ::std::os::raw::c_void,
        destroy: duckdb_delete_callback_t,
    ) |  |  |  |  |

| `duckdb_scalar_function_set_function` | function | duckdb_scalar_function_set_function(
        scalar_function: duckdb_scalar_function,
        function: duckdb_scalar_function_t,
    ) |  |  |  |  |

| `duckdb_scalar_function_set_name` | function | duckdb_scalar_function_set_name(
        scalar_function: duckdb_scalar_function,
        name: *const ::std::os::raw::c_char,
    ) |  |  |  |  |

| `duckdb_scalar_function_set_return_type` | function | duckdb_scalar_function_set_return_type(scalar_function: duckdb_scalar_function, type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_scalar_function_set_special_handling` | function | duckdb_scalar_function_set_special_handling(scalar_function: duckdb_scalar_function) |  |  |  |  |

| `duckdb_scalar_function_set_varargs` | function | duckdb_scalar_function_set_varargs(scalar_function: duckdb_scalar_function, type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_scalar_function_set_volatile` | function | duckdb_scalar_function_set_volatile(scalar_function: duckdb_scalar_function) |  |  |  |  |

| `duckdb_schema_from_arrow` | function | duckdb_schema_from_arrow(
        connection: duckdb_connection,
        schema: *mut ArrowSchema,
        out_types: *mut duckdb_arrow_converted_schema,
    ) |  |  |  |  |

| `duckdb_selection_vector_get_data_ptr` | function | duckdb_selection_vector_get_data_ptr(sel: duckdb_selection_vector) |  |  |  |  |

| `duckdb_set_config` | function | duckdb_set_config(
        config: duckdb_config,
        name: *const ::std::os::raw::c_char,
        option: *const ::std::os::raw::c_char,
    ) |  |  |  |  |

| `duckdb_slice_vector` | function | duckdb_slice_vector(vector: duckdb_vector, sel: duckdb_selection_vector, len: idx_t) |  |  |  |  |

| `duckdb_stream_fetch_chunk` | function | duckdb_stream_fetch_chunk(result: duckdb_result) |  |  |  |  |

| `duckdb_string_is_inlined` | function | duckdb_string_is_inlined(string: duckdb_string_t) |  |  |  |  |

| `duckdb_string_t_data` | function | duckdb_string_t_data(string: *mut duckdb_string_t) |  |  |  |  |

| `duckdb_string_t_length` | function | duckdb_string_t_length(string: duckdb_string_t) |  |  |  |  |

| `duckdb_struct_type_child_count` | function | duckdb_struct_type_child_count(type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_struct_type_child_name` | function | duckdb_struct_type_child_name(type_: duckdb_logical_type, index: idx_t) |  |  |  |  |

| `duckdb_struct_type_child_type` | function | duckdb_struct_type_child_type(type_: duckdb_logical_type, index: idx_t) |  |  |  |  |

| `duckdb_struct_vector_get_child` | function | duckdb_struct_vector_get_child(vector: duckdb_vector, index: idx_t) |  |  |  |  |

| `duckdb_table_description_create` | function | duckdb_table_description_create(
        connection: duckdb_connection,
        schema: *const ::std::os::raw::c_char,
        table: *const ::std::os::raw::c_char,
        out: *mut duckdb_table_description,
    ) |  |  |  |  |

| `duckdb_table_description_create_ext` | function | duckdb_table_description_create_ext(
        connection: duckdb_connection,
        catalog: *const ::std::os::raw::c_char,
        schema: *const ::std::os::raw::c_char,
        table: *const ::std::os::raw::c_char,
        out: *mut duckdb_table_description,
    ) |  |  |  |  |

| `duckdb_table_description_destroy` | function | duckdb_table_description_destroy(table_description: *mut duckdb_table_description) |  |  |  |  |

| `duckdb_table_description_error` | function | duckdb_table_description_error(table_description: duckdb_table_description) |  |  |  |  |

| `duckdb_table_description_get_column_name` | function | duckdb_table_description_get_column_name(
        table_description: duckdb_table_description,
        index: idx_t,
    ) |  |  |  |  |

| `duckdb_table_function_add_named_parameter` | function | duckdb_table_function_add_named_parameter(
        table_function: duckdb_table_function,
        name: *const ::std::os::raw::c_char,
        type_: duckdb_logical_type,
    ) |  |  |  |  |

| `duckdb_table_function_add_parameter` | function | duckdb_table_function_add_parameter(table_function: duckdb_table_function, type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_table_function_get_client_context` | function | duckdb_table_function_get_client_context(info: duckdb_bind_info, out_context: *mut duckdb_client_context) |  |  |  |  |

| `duckdb_table_function_set_bind` | function | duckdb_table_function_set_bind(table_function: duckdb_table_function, bind: duckdb_table_function_bind_t) |  |  |  |  |

| `duckdb_table_function_set_extra_info` | function | duckdb_table_function_set_extra_info(
        table_function: duckdb_table_function,
        extra_info: *mut ::std::os::raw::c_void,
        destroy: duckdb_delete_callback_t,
    ) |  |  |  |  |

| `duckdb_table_function_set_function` | function | duckdb_table_function_set_function(table_function: duckdb_table_function, function: duckdb_table_function_t) |  |  |  |  |

| `duckdb_table_function_set_init` | function | duckdb_table_function_set_init(table_function: duckdb_table_function, init: duckdb_table_function_init_t) |  |  |  |  |

| `duckdb_table_function_set_local_init` | function | duckdb_table_function_set_local_init(
        table_function: duckdb_table_function,
        init: duckdb_table_function_init_t,
    ) |  |  |  |  |

| `duckdb_table_function_set_name` | function | duckdb_table_function_set_name(table_function: duckdb_table_function, name: *const ::std::os::raw::c_char) |  |  |  |  |

| `duckdb_table_function_supports_projection_pushdown` | function | duckdb_table_function_supports_projection_pushdown(table_function: duckdb_table_function, pushdown: bool) |  |  |  |  |

| `duckdb_task_state_is_finished` | function | duckdb_task_state_is_finished(state: duckdb_task_state) |  |  |  |  |

| `duckdb_to_arrow_schema` | function | duckdb_to_arrow_schema(
        arrow_options: duckdb_arrow_options,
        types: *mut duckdb_logical_type,
        names: *mut *const ::std::os::raw::c_char,
        column_count: idx_t,
        out_schema: *mut ArrowSchema,
    ) |  |  |  |  |

| `duckdb_to_date` | function | duckdb_to_date(date: duckdb_date_struct) |  |  |  |  |

| `duckdb_to_time` | function | duckdb_to_time(time: duckdb_time_struct) |  |  |  |  |

| `duckdb_to_timestamp` | function | duckdb_to_timestamp(ts: duckdb_timestamp_struct) |  |  |  |  |

| `duckdb_uhugeint_to_double` | function | duckdb_uhugeint_to_double(val: duckdb_uhugeint) |  |  |  |  |

| `duckdb_union_type_member_count` | function | duckdb_union_type_member_count(type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_union_type_member_name` | function | duckdb_union_type_member_name(type_: duckdb_logical_type, index: idx_t) |  |  |  |  |

| `duckdb_union_type_member_type` | function | duckdb_union_type_member_type(type_: duckdb_logical_type, index: idx_t) |  |  |  |  |

| `duckdb_validity_row_is_valid` | function | duckdb_validity_row_is_valid(validity: *mut u64, row: idx_t) |  |  |  |  |

| `duckdb_validity_set_row_invalid` | function | duckdb_validity_set_row_invalid(validity: *mut u64, row: idx_t) |  |  |  |  |

| `duckdb_validity_set_row_valid` | function | duckdb_validity_set_row_valid(validity: *mut u64, row: idx_t) |  |  |  |  |

| `duckdb_validity_set_row_validity` | function | duckdb_validity_set_row_validity(validity: *mut u64, row: idx_t, valid: bool) |  |  |  |  |

| `duckdb_value_blob` | function | duckdb_value_blob(result: *mut duckdb_result, col: idx_t, row: idx_t) |  |  |  |  |

| `duckdb_value_boolean` | function | duckdb_value_boolean(result: *mut duckdb_result, col: idx_t, row: idx_t) |  |  |  |  |

| `duckdb_value_date` | function | duckdb_value_date(result: *mut duckdb_result, col: idx_t, row: idx_t) |  |  |  |  |

| `duckdb_value_decimal` | function | duckdb_value_decimal(result: *mut duckdb_result, col: idx_t, row: idx_t) |  |  |  |  |

| `duckdb_value_double` | function | duckdb_value_double(result: *mut duckdb_result, col: idx_t, row: idx_t) |  |  |  |  |

| `duckdb_value_float` | function | duckdb_value_float(result: *mut duckdb_result, col: idx_t, row: idx_t) |  |  |  |  |

| `duckdb_value_hugeint` | function | duckdb_value_hugeint(result: *mut duckdb_result, col: idx_t, row: idx_t) |  |  |  |  |

| `duckdb_value_int16` | function | duckdb_value_int16(result: *mut duckdb_result, col: idx_t, row: idx_t) |  |  |  |  |

| `duckdb_value_int32` | function | duckdb_value_int32(result: *mut duckdb_result, col: idx_t, row: idx_t) |  |  |  |  |

| `duckdb_value_int64` | function | duckdb_value_int64(result: *mut duckdb_result, col: idx_t, row: idx_t) |  |  |  |  |

| `duckdb_value_int8` | function | duckdb_value_int8(result: *mut duckdb_result, col: idx_t, row: idx_t) |  |  |  |  |

| `duckdb_value_interval` | function | duckdb_value_interval(result: *mut duckdb_result, col: idx_t, row: idx_t) |  |  |  |  |

| `duckdb_value_is_null` | function | duckdb_value_is_null(result: *mut duckdb_result, col: idx_t, row: idx_t) |  |  |  |  |

| `duckdb_value_string` | function | duckdb_value_string(result: *mut duckdb_result, col: idx_t, row: idx_t) |  |  |  |  |

| `duckdb_value_string_internal` | function | duckdb_value_string_internal(result: *mut duckdb_result, col: idx_t, row: idx_t) |  |  |  |  |

| `duckdb_value_time` | function | duckdb_value_time(result: *mut duckdb_result, col: idx_t, row: idx_t) |  |  |  |  |

| `duckdb_value_timestamp` | function | duckdb_value_timestamp(result: *mut duckdb_result, col: idx_t, row: idx_t) |  |  |  |  |

| `duckdb_value_to_string` | function | duckdb_value_to_string(value: duckdb_value) |  |  |  |  |

| `duckdb_value_uhugeint` | function | duckdb_value_uhugeint(result: *mut duckdb_result, col: idx_t, row: idx_t) |  |  |  |  |

| `duckdb_value_uint16` | function | duckdb_value_uint16(result: *mut duckdb_result, col: idx_t, row: idx_t) |  |  |  |  |

| `duckdb_value_uint32` | function | duckdb_value_uint32(result: *mut duckdb_result, col: idx_t, row: idx_t) |  |  |  |  |

| `duckdb_value_uint64` | function | duckdb_value_uint64(result: *mut duckdb_result, col: idx_t, row: idx_t) |  |  |  |  |

| `duckdb_value_uint8` | function | duckdb_value_uint8(result: *mut duckdb_result, col: idx_t, row: idx_t) |  |  |  |  |

| `duckdb_value_varchar` | function | duckdb_value_varchar(result: *mut duckdb_result, col: idx_t, row: idx_t) |  |  |  |  |

| `duckdb_value_varchar_internal` | function | duckdb_value_varchar_internal(
        result: *mut duckdb_result,
        col: idx_t,
        row: idx_t,
    ) |  |  |  |  |

| `duckdb_vector_assign_string_element` | function | duckdb_vector_assign_string_element(
        vector: duckdb_vector,
        index: idx_t,
        str_: *const ::std::os::raw::c_char,
    ) |  |  |  |  |

| `duckdb_vector_assign_string_element_len` | function | duckdb_vector_assign_string_element_len(
        vector: duckdb_vector,
        index: idx_t,
        str_: *const ::std::os::raw::c_char,
        str_len: idx_t,
    ) |  |  |  |  |

| `duckdb_vector_copy_sel` | function | duckdb_vector_copy_sel(
        src: duckdb_vector,
        dst: duckdb_vector,
        sel: duckdb_selection_vector,
        src_count: idx_t,
        src_offset: idx_t,
        dst_offset: idx_t,
    ) |  |  |  |  |

| `duckdb_vector_ensure_validity_writable` | function | duckdb_vector_ensure_validity_writable(vector: duckdb_vector) |  |  |  |  |

| `duckdb_vector_get_column_type` | function | duckdb_vector_get_column_type(vector: duckdb_vector) |  |  |  |  |

| `duckdb_vector_get_data` | function | duckdb_vector_get_data(vector: duckdb_vector) |  |  |  |  |

| `duckdb_vector_get_validity` | function | duckdb_vector_get_validity(vector: duckdb_vector) |  |  |  |  |

| `duckdb_vector_reference_value` | function | duckdb_vector_reference_value(vector: duckdb_vector, value: duckdb_value) |  |  |  |  |

| `duckdb_vector_reference_vector` | function | duckdb_vector_reference_vector(to_vector: duckdb_vector, from_vector: duckdb_vector) |  |  |  |  |

| `duckdb_vector_size` | function | duckdb_vector_size() |  |  |  |  |

| `ArrowArray` | struct |  |  |  |  |  |

| `ArrowSchema` | struct |  |  |  |  |  |


### crates/frozen-duckdb-sys/src/bindgen_bundled_version_loadable.rs

| `duckdb_add_aggregate_function_to_set` | function | duckdb_add_aggregate_function_to_set(
    set: duckdb_aggregate_function_set,
    function: duckdb_aggregate_function,
) |  |  |  |  |

| `duckdb_add_replacement_scan` | function | duckdb_add_replacement_scan(
    db: duckdb_database,
    replacement: duckdb_replacement_callback_t,
    extra_data: *mut ::std::os::raw::c_void,
    delete_callback: duckdb_delete_callback_t,
) |  |  |  |  |

| `duckdb_add_scalar_function_to_set` | function | duckdb_add_scalar_function_to_set(
    set: duckdb_scalar_function_set,
    function: duckdb_scalar_function,
) |  |  |  |  |

| `duckdb_aggregate_function_add_parameter` | function | duckdb_aggregate_function_add_parameter(
    aggregate_function: duckdb_aggregate_function,
    type_: duckdb_logical_type,
) |  |  |  |  |

| `duckdb_aggregate_function_get_extra_info` | function | duckdb_aggregate_function_get_extra_info(
    info: duckdb_function_info,
) |  |  |  |  |

| `duckdb_aggregate_function_set_destructor` | function | duckdb_aggregate_function_set_destructor(
    aggregate_function: duckdb_aggregate_function,
    destroy: duckdb_aggregate_destroy_t,
) |  |  |  |  |

| `duckdb_aggregate_function_set_error` | function | duckdb_aggregate_function_set_error(
    info: duckdb_function_info,
    error: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_aggregate_function_set_extra_info` | function | duckdb_aggregate_function_set_extra_info(
    aggregate_function: duckdb_aggregate_function,
    extra_info: *mut ::std::os::raw::c_void,
    destroy: duckdb_delete_callback_t,
) |  |  |  |  |

| `duckdb_aggregate_function_set_functions` | function | duckdb_aggregate_function_set_functions(
    aggregate_function: duckdb_aggregate_function,
    state_size: duckdb_aggregate_state_size,
    state_init: duckdb_aggregate_init_t,
    update: duckdb_aggregate_update_t,
    combine: duckdb_aggregate_combine_t,
    finalize: duckdb_aggregate_finalize_t,
) |  |  |  |  |

| `duckdb_aggregate_function_set_name` | function | duckdb_aggregate_function_set_name(
    aggregate_function: duckdb_aggregate_function,
    name: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_aggregate_function_set_return_type` | function | duckdb_aggregate_function_set_return_type(
    aggregate_function: duckdb_aggregate_function,
    type_: duckdb_logical_type,
) |  |  |  |  |

| `duckdb_aggregate_function_set_special_handling` | function | duckdb_aggregate_function_set_special_handling(
    aggregate_function: duckdb_aggregate_function,
) |  |  |  |  |

| `duckdb_append_blob` | function | duckdb_append_blob(
    appender: duckdb_appender,
    data: *const ::std::os::raw::c_void,
    length: idx_t,
) |  |  |  |  |

| `duckdb_append_bool` | function | duckdb_append_bool(
    appender: duckdb_appender,
    value: bool,
) |  |  |  |  |

| `duckdb_append_data_chunk` | function | duckdb_append_data_chunk(
    appender: duckdb_appender,
    chunk: duckdb_data_chunk,
) |  |  |  |  |

| `duckdb_append_date` | function | duckdb_append_date(
    appender: duckdb_appender,
    value: duckdb_date,
) |  |  |  |  |

| `duckdb_append_default` | function | duckdb_append_default(appender: duckdb_appender) |  |  |  |  |

| `duckdb_append_default_to_chunk` | function | duckdb_append_default_to_chunk(
    appender: duckdb_appender,
    chunk: duckdb_data_chunk,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_append_double` | function | duckdb_append_double(
    appender: duckdb_appender,
    value: f64,
) |  |  |  |  |

| `duckdb_append_float` | function | duckdb_append_float(
    appender: duckdb_appender,
    value: f32,
) |  |  |  |  |

| `duckdb_append_hugeint` | function | duckdb_append_hugeint(
    appender: duckdb_appender,
    value: duckdb_hugeint,
) |  |  |  |  |

| `duckdb_append_int16` | function | duckdb_append_int16(
    appender: duckdb_appender,
    value: i16,
) |  |  |  |  |

| `duckdb_append_int32` | function | duckdb_append_int32(
    appender: duckdb_appender,
    value: i32,
) |  |  |  |  |

| `duckdb_append_int64` | function | duckdb_append_int64(
    appender: duckdb_appender,
    value: i64,
) |  |  |  |  |

| `duckdb_append_int8` | function | duckdb_append_int8(appender: duckdb_appender, value: i8) |  |  |  |  |

| `duckdb_append_interval` | function | duckdb_append_interval(
    appender: duckdb_appender,
    value: duckdb_interval,
) |  |  |  |  |

| `duckdb_append_null` | function | duckdb_append_null(appender: duckdb_appender) |  |  |  |  |

| `duckdb_append_time` | function | duckdb_append_time(
    appender: duckdb_appender,
    value: duckdb_time,
) |  |  |  |  |

| `duckdb_append_timestamp` | function | duckdb_append_timestamp(
    appender: duckdb_appender,
    value: duckdb_timestamp,
) |  |  |  |  |

| `duckdb_append_uhugeint` | function | duckdb_append_uhugeint(
    appender: duckdb_appender,
    value: duckdb_uhugeint,
) |  |  |  |  |

| `duckdb_append_uint16` | function | duckdb_append_uint16(
    appender: duckdb_appender,
    value: u16,
) |  |  |  |  |

| `duckdb_append_uint32` | function | duckdb_append_uint32(
    appender: duckdb_appender,
    value: u32,
) |  |  |  |  |

| `duckdb_append_uint64` | function | duckdb_append_uint64(
    appender: duckdb_appender,
    value: u64,
) |  |  |  |  |

| `duckdb_append_uint8` | function | duckdb_append_uint8(appender: duckdb_appender, value: u8) |  |  |  |  |

| `duckdb_append_value` | function | duckdb_append_value(
    appender: duckdb_appender,
    value: duckdb_value,
) |  |  |  |  |

| `duckdb_append_varchar` | function | duckdb_append_varchar(
    appender: duckdb_appender,
    val: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_append_varchar_length` | function | duckdb_append_varchar_length(
    appender: duckdb_appender,
    val: *const ::std::os::raw::c_char,
    length: idx_t,
) |  |  |  |  |

| `duckdb_appender_add_column` | function | duckdb_appender_add_column(
    appender: duckdb_appender,
    name: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_appender_begin_row` | function | duckdb_appender_begin_row(appender: duckdb_appender) |  |  |  |  |

| `duckdb_appender_clear_columns` | function | duckdb_appender_clear_columns(appender: duckdb_appender) |  |  |  |  |

| `duckdb_appender_close` | function | duckdb_appender_close(appender: duckdb_appender) |  |  |  |  |

| `duckdb_appender_column_count` | function | duckdb_appender_column_count(appender: duckdb_appender) |  |  |  |  |

| `duckdb_appender_column_type` | function | duckdb_appender_column_type(
    appender: duckdb_appender,
    col_idx: idx_t,
) |  |  |  |  |

| `duckdb_appender_create` | function | duckdb_appender_create(
    connection: duckdb_connection,
    schema: *const ::std::os::raw::c_char,
    table: *const ::std::os::raw::c_char,
    out_appender: *mut duckdb_appender,
) |  |  |  |  |

| `duckdb_appender_create_ext` | function | duckdb_appender_create_ext(
    connection: duckdb_connection,
    catalog: *const ::std::os::raw::c_char,
    schema: *const ::std::os::raw::c_char,
    table: *const ::std::os::raw::c_char,
    out_appender: *mut duckdb_appender,
) |  |  |  |  |

| `duckdb_appender_create_query` | function | duckdb_appender_create_query(
    connection: duckdb_connection,
    query: *const ::std::os::raw::c_char,
    column_count: idx_t,
    types: *mut duckdb_logical_type,
    table_name: *const ::std::os::raw::c_char,
    column_names: *mut *const ::std::os::raw::c_char,
    out_appender: *mut duckdb_appender,
) |  |  |  |  |

| `duckdb_appender_destroy` | function | duckdb_appender_destroy(appender: *mut duckdb_appender) |  |  |  |  |

| `duckdb_appender_end_row` | function | duckdb_appender_end_row(appender: duckdb_appender) |  |  |  |  |

| `duckdb_appender_error` | function | duckdb_appender_error(
    appender: duckdb_appender,
) |  |  |  |  |

| `duckdb_appender_error_data` | function | duckdb_appender_error_data(
    appender: duckdb_appender,
) |  |  |  |  |

| `duckdb_appender_flush` | function | duckdb_appender_flush(appender: duckdb_appender) |  |  |  |  |

| `duckdb_array_type_array_size` | function | duckdb_array_type_array_size(type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_array_type_child_type` | function | duckdb_array_type_child_type(
    type_: duckdb_logical_type,
) |  |  |  |  |

| `duckdb_array_vector_get_child` | function | duckdb_array_vector_get_child(vector: duckdb_vector) |  |  |  |  |

| `duckdb_arrow_array_scan` | function | duckdb_arrow_array_scan(
    connection: duckdb_connection,
    table_name: *const ::std::os::raw::c_char,
    arrow_schema: duckdb_arrow_schema,
    arrow_array: duckdb_arrow_array,
    out_stream: *mut duckdb_arrow_stream,
) |  |  |  |  |

| `duckdb_arrow_column_count` | function | duckdb_arrow_column_count(result: duckdb_arrow) |  |  |  |  |

| `duckdb_arrow_row_count` | function | duckdb_arrow_row_count(result: duckdb_arrow) |  |  |  |  |

| `duckdb_arrow_rows_changed` | function | duckdb_arrow_rows_changed(result: duckdb_arrow) |  |  |  |  |

| `duckdb_arrow_scan` | function | duckdb_arrow_scan(
    connection: duckdb_connection,
    table_name: *const ::std::os::raw::c_char,
    arrow: duckdb_arrow_stream,
) |  |  |  |  |

| `duckdb_bind_add_result_column` | function | duckdb_bind_add_result_column(
    info: duckdb_bind_info,
    name: *const ::std::os::raw::c_char,
    type_: duckdb_logical_type,
) |  |  |  |  |

| `duckdb_bind_blob` | function | duckdb_bind_blob(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
    data: *const ::std::os::raw::c_void,
    length: idx_t,
) |  |  |  |  |

| `duckdb_bind_boolean` | function | duckdb_bind_boolean(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
    val: bool,
) |  |  |  |  |

| `duckdb_bind_date` | function | duckdb_bind_date(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
    val: duckdb_date,
) |  |  |  |  |

| `duckdb_bind_decimal` | function | duckdb_bind_decimal(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
    val: duckdb_decimal,
) |  |  |  |  |

| `duckdb_bind_double` | function | duckdb_bind_double(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
    val: f64,
) |  |  |  |  |

| `duckdb_bind_float` | function | duckdb_bind_float(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
    val: f32,
) |  |  |  |  |

| `duckdb_bind_get_extra_info` | function | duckdb_bind_get_extra_info(
    info: duckdb_bind_info,
) |  |  |  |  |

| `duckdb_bind_get_named_parameter` | function | duckdb_bind_get_named_parameter(
    info: duckdb_bind_info,
    name: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_bind_get_parameter` | function | duckdb_bind_get_parameter(
    info: duckdb_bind_info,
    index: idx_t,
) |  |  |  |  |

| `duckdb_bind_get_parameter_count` | function | duckdb_bind_get_parameter_count(info: duckdb_bind_info) |  |  |  |  |

| `duckdb_bind_hugeint` | function | duckdb_bind_hugeint(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
    val: duckdb_hugeint,
) |  |  |  |  |

| `duckdb_bind_int16` | function | duckdb_bind_int16(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
    val: i16,
) |  |  |  |  |

| `duckdb_bind_int32` | function | duckdb_bind_int32(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
    val: i32,
) |  |  |  |  |

| `duckdb_bind_int64` | function | duckdb_bind_int64(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
    val: i64,
) |  |  |  |  |

| `duckdb_bind_int8` | function | duckdb_bind_int8(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
    val: i8,
) |  |  |  |  |

| `duckdb_bind_interval` | function | duckdb_bind_interval(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
    val: duckdb_interval,
) |  |  |  |  |

| `duckdb_bind_null` | function | duckdb_bind_null(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
) |  |  |  |  |

| `duckdb_bind_parameter_index` | function | duckdb_bind_parameter_index(
    prepared_statement: duckdb_prepared_statement,
    param_idx_out: *mut idx_t,
    name: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_bind_set_bind_data` | function | duckdb_bind_set_bind_data(
    info: duckdb_bind_info,
    bind_data: *mut ::std::os::raw::c_void,
    destroy: duckdb_delete_callback_t,
) |  |  |  |  |

| `duckdb_bind_set_cardinality` | function | duckdb_bind_set_cardinality(
    info: duckdb_bind_info,
    cardinality: idx_t,
    is_exact: bool,
) |  |  |  |  |

| `duckdb_bind_set_error` | function | duckdb_bind_set_error(
    info: duckdb_bind_info,
    error: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_bind_time` | function | duckdb_bind_time(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
    val: duckdb_time,
) |  |  |  |  |

| `duckdb_bind_timestamp` | function | duckdb_bind_timestamp(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
    val: duckdb_timestamp,
) |  |  |  |  |

| `duckdb_bind_timestamp_tz` | function | duckdb_bind_timestamp_tz(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
    val: duckdb_timestamp,
) |  |  |  |  |

| `duckdb_bind_uhugeint` | function | duckdb_bind_uhugeint(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
    val: duckdb_uhugeint,
) |  |  |  |  |

| `duckdb_bind_uint16` | function | duckdb_bind_uint16(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
    val: u16,
) |  |  |  |  |

| `duckdb_bind_uint32` | function | duckdb_bind_uint32(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
    val: u32,
) |  |  |  |  |

| `duckdb_bind_uint64` | function | duckdb_bind_uint64(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
    val: u64,
) |  |  |  |  |

| `duckdb_bind_uint8` | function | duckdb_bind_uint8(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
    val: u8,
) |  |  |  |  |

| `duckdb_bind_value` | function | duckdb_bind_value(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
    val: duckdb_value,
) |  |  |  |  |

| `duckdb_bind_varchar` | function | duckdb_bind_varchar(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
    val: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_bind_varchar_length` | function | duckdb_bind_varchar_length(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
    val: *const ::std::os::raw::c_char,
    length: idx_t,
) |  |  |  |  |

| `duckdb_cast_function_get_cast_mode` | function | duckdb_cast_function_get_cast_mode(
    info: duckdb_function_info,
) |  |  |  |  |

| `duckdb_cast_function_get_extra_info` | function | duckdb_cast_function_get_extra_info(
    info: duckdb_function_info,
) |  |  |  |  |

| `duckdb_cast_function_set_error` | function | duckdb_cast_function_set_error(
    info: duckdb_function_info,
    error: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_cast_function_set_extra_info` | function | duckdb_cast_function_set_extra_info(
    cast_function: duckdb_cast_function,
    extra_info: *mut ::std::os::raw::c_void,
    destroy: duckdb_delete_callback_t,
) |  |  |  |  |

| `duckdb_cast_function_set_function` | function | duckdb_cast_function_set_function(
    cast_function: duckdb_cast_function,
    function: duckdb_cast_function_t,
) |  |  |  |  |

| `duckdb_cast_function_set_implicit_cast_cost` | function | duckdb_cast_function_set_implicit_cast_cost(
    cast_function: duckdb_cast_function,
    cost: i64,
) |  |  |  |  |

| `duckdb_cast_function_set_row_error` | function | duckdb_cast_function_set_row_error(
    info: duckdb_function_info,
    error: *const ::std::os::raw::c_char,
    row: idx_t,
    output: duckdb_vector,
) |  |  |  |  |

| `duckdb_cast_function_set_source_type` | function | duckdb_cast_function_set_source_type(
    cast_function: duckdb_cast_function,
    source_type: duckdb_logical_type,
) |  |  |  |  |

| `duckdb_cast_function_set_target_type` | function | duckdb_cast_function_set_target_type(
    cast_function: duckdb_cast_function,
    target_type: duckdb_logical_type,
) |  |  |  |  |

| `duckdb_clear_bindings` | function | duckdb_clear_bindings(
    prepared_statement: duckdb_prepared_statement,
) |  |  |  |  |

| `duckdb_client_context_get_connection_id` | function | duckdb_client_context_get_connection_id(
    context: duckdb_client_context,
) |  |  |  |  |

| `duckdb_close` | function | duckdb_close(database: *mut duckdb_database) |  |  |  |  |

| `duckdb_column_count` | function | duckdb_column_count(result: *mut duckdb_result) |  |  |  |  |

| `duckdb_column_data` | function | duckdb_column_data(
    result: *mut duckdb_result,
    col: idx_t,
) |  |  |  |  |

| `duckdb_column_has_default` | function | duckdb_column_has_default(
    table_description: duckdb_table_description,
    index: idx_t,
    out: *mut bool,
) |  |  |  |  |

| `duckdb_column_logical_type` | function | duckdb_column_logical_type(
    result: *mut duckdb_result,
    col: idx_t,
) |  |  |  |  |

| `duckdb_column_name` | function | duckdb_column_name(
    result: *mut duckdb_result,
    col: idx_t,
) |  |  |  |  |

| `duckdb_column_type` | function | duckdb_column_type(result: *mut duckdb_result, col: idx_t) |  |  |  |  |

| `duckdb_config_count` | function | duckdb_config_count() |  |  |  |  |

| `duckdb_connect` | function | duckdb_connect(
    database: duckdb_database,
    out_connection: *mut duckdb_connection,
) |  |  |  |  |

| `duckdb_connection_get_arrow_options` | function | duckdb_connection_get_arrow_options(
    connection: duckdb_connection,
    out_arrow_options: *mut duckdb_arrow_options,
) |  |  |  |  |

| `duckdb_connection_get_client_context` | function | duckdb_connection_get_client_context(
    connection: duckdb_connection,
    out_context: *mut duckdb_client_context,
) |  |  |  |  |

| `duckdb_create_aggregate_function` | function | duckdb_create_aggregate_function() |  |  |  |  |

| `duckdb_create_aggregate_function_set` | function | duckdb_create_aggregate_function_set(
    name: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_create_array_type` | function | duckdb_create_array_type(
    type_: duckdb_logical_type,
    array_size: idx_t,
) |  |  |  |  |

| `duckdb_create_array_value` | function | duckdb_create_array_value(
    type_: duckdb_logical_type,
    values: *mut duckdb_value,
    value_count: idx_t,
) |  |  |  |  |

| `duckdb_create_bignum` | function | duckdb_create_bignum(input: duckdb_bignum) |  |  |  |  |

| `duckdb_create_bit` | function | duckdb_create_bit(input: duckdb_bit) |  |  |  |  |

| `duckdb_create_blob` | function | duckdb_create_blob(data: *const u8, length: idx_t) |  |  |  |  |

| `duckdb_create_bool` | function | duckdb_create_bool(input: bool) |  |  |  |  |

| `duckdb_create_cast_function` | function | duckdb_create_cast_function() |  |  |  |  |

| `duckdb_create_config` | function | duckdb_create_config(out_config: *mut duckdb_config) |  |  |  |  |

| `duckdb_create_data_chunk` | function | duckdb_create_data_chunk(
    types: *mut duckdb_logical_type,
    column_count: idx_t,
) |  |  |  |  |

| `duckdb_create_date` | function | duckdb_create_date(input: duckdb_date) |  |  |  |  |

| `duckdb_create_decimal` | function | duckdb_create_decimal(input: duckdb_decimal) |  |  |  |  |

| `duckdb_create_decimal_type` | function | duckdb_create_decimal_type(width: u8, scale: u8) |  |  |  |  |

| `duckdb_create_double` | function | duckdb_create_double(input: f64) |  |  |  |  |

| `duckdb_create_enum_type` | function | duckdb_create_enum_type(
    member_names: *mut *const ::std::os::raw::c_char,
    member_count: idx_t,
) |  |  |  |  |

| `duckdb_create_enum_value` | function | duckdb_create_enum_value(
    type_: duckdb_logical_type,
    value: u64,
) |  |  |  |  |

| `duckdb_create_error_data` | function | duckdb_create_error_data(
    type_: duckdb_error_type,
    message: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_create_float` | function | duckdb_create_float(input: f32) |  |  |  |  |

| `duckdb_create_hugeint` | function | duckdb_create_hugeint(input: duckdb_hugeint) |  |  |  |  |

| `duckdb_create_instance_cache` | function | duckdb_create_instance_cache() |  |  |  |  |

| `duckdb_create_int16` | function | duckdb_create_int16(input: i16) |  |  |  |  |

| `duckdb_create_int32` | function | duckdb_create_int32(input: i32) |  |  |  |  |

| `duckdb_create_int64` | function | duckdb_create_int64(val: i64) |  |  |  |  |

| `duckdb_create_int8` | function | duckdb_create_int8(input: i8) |  |  |  |  |

| `duckdb_create_interval` | function | duckdb_create_interval(input: duckdb_interval) |  |  |  |  |

| `duckdb_create_list_type` | function | duckdb_create_list_type(
    type_: duckdb_logical_type,
) |  |  |  |  |

| `duckdb_create_list_value` | function | duckdb_create_list_value(
    type_: duckdb_logical_type,
    values: *mut duckdb_value,
    value_count: idx_t,
) |  |  |  |  |

| `duckdb_create_logical_type` | function | duckdb_create_logical_type(type_: duckdb_type) |  |  |  |  |

| `duckdb_create_map_type` | function | duckdb_create_map_type(
    key_type: duckdb_logical_type,
    value_type: duckdb_logical_type,
) |  |  |  |  |

| `duckdb_create_map_value` | function | duckdb_create_map_value(
    map_type: duckdb_logical_type,
    keys: *mut duckdb_value,
    values: *mut duckdb_value,
    entry_count: idx_t,
) |  |  |  |  |

| `duckdb_create_null_value` | function | duckdb_create_null_value() |  |  |  |  |

| `duckdb_create_scalar_function` | function | duckdb_create_scalar_function() |  |  |  |  |

| `duckdb_create_scalar_function_set` | function | duckdb_create_scalar_function_set(
    name: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_create_selection_vector` | function | duckdb_create_selection_vector(size: idx_t) |  |  |  |  |

| `duckdb_create_struct_type` | function | duckdb_create_struct_type(
    member_types: *mut duckdb_logical_type,
    member_names: *mut *const ::std::os::raw::c_char,
    member_count: idx_t,
) |  |  |  |  |

| `duckdb_create_struct_value` | function | duckdb_create_struct_value(
    type_: duckdb_logical_type,
    values: *mut duckdb_value,
) |  |  |  |  |

| `duckdb_create_table_function` | function | duckdb_create_table_function() |  |  |  |  |

| `duckdb_create_task_state` | function | duckdb_create_task_state(database: duckdb_database) |  |  |  |  |

| `duckdb_create_time` | function | duckdb_create_time(input: duckdb_time) |  |  |  |  |

| `duckdb_create_time_ns` | function | duckdb_create_time_ns(input: duckdb_time_ns) |  |  |  |  |

| `duckdb_create_time_tz` | function | duckdb_create_time_tz(micros: i64, offset: i32) |  |  |  |  |

| `duckdb_create_time_tz_value` | function | duckdb_create_time_tz_value(value: duckdb_time_tz) |  |  |  |  |

| `duckdb_create_timestamp` | function | duckdb_create_timestamp(input: duckdb_timestamp) |  |  |  |  |

| `duckdb_create_timestamp_ms` | function | duckdb_create_timestamp_ms(input: duckdb_timestamp_ms) |  |  |  |  |

| `duckdb_create_timestamp_ns` | function | duckdb_create_timestamp_ns(input: duckdb_timestamp_ns) |  |  |  |  |

| `duckdb_create_timestamp_s` | function | duckdb_create_timestamp_s(input: duckdb_timestamp_s) |  |  |  |  |

| `duckdb_create_timestamp_tz` | function | duckdb_create_timestamp_tz(input: duckdb_timestamp) |  |  |  |  |

| `duckdb_create_uhugeint` | function | duckdb_create_uhugeint(input: duckdb_uhugeint) |  |  |  |  |

| `duckdb_create_uint16` | function | duckdb_create_uint16(input: u16) |  |  |  |  |

| `duckdb_create_uint32` | function | duckdb_create_uint32(input: u32) |  |  |  |  |

| `duckdb_create_uint64` | function | duckdb_create_uint64(input: u64) |  |  |  |  |

| `duckdb_create_uint8` | function | duckdb_create_uint8(input: u8) |  |  |  |  |

| `duckdb_create_union_type` | function | duckdb_create_union_type(
    member_types: *mut duckdb_logical_type,
    member_names: *mut *const ::std::os::raw::c_char,
    member_count: idx_t,
) |  |  |  |  |

| `duckdb_create_union_value` | function | duckdb_create_union_value(
    union_type: duckdb_logical_type,
    tag_index: idx_t,
    value: duckdb_value,
) |  |  |  |  |

| `duckdb_create_uuid` | function | duckdb_create_uuid(input: duckdb_uhugeint) |  |  |  |  |

| `duckdb_create_varchar` | function | duckdb_create_varchar(
    text: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_create_varchar_length` | function | duckdb_create_varchar_length(
    text: *const ::std::os::raw::c_char,
    length: idx_t,
) |  |  |  |  |

| `duckdb_create_vector` | function | duckdb_create_vector(
    type_: duckdb_logical_type,
    capacity: idx_t,
) |  |  |  |  |

| `duckdb_data_chunk_from_arrow` | function | duckdb_data_chunk_from_arrow(
    connection: duckdb_connection,
    arrow_array: *mut ArrowArray,
    converted_schema: duckdb_arrow_converted_schema,
    out_chunk: *mut duckdb_data_chunk,
) |  |  |  |  |

| `duckdb_data_chunk_get_column_count` | function | duckdb_data_chunk_get_column_count(chunk: duckdb_data_chunk) |  |  |  |  |

| `duckdb_data_chunk_get_size` | function | duckdb_data_chunk_get_size(chunk: duckdb_data_chunk) |  |  |  |  |

| `duckdb_data_chunk_get_vector` | function | duckdb_data_chunk_get_vector(
    chunk: duckdb_data_chunk,
    col_idx: idx_t,
) |  |  |  |  |

| `duckdb_data_chunk_reset` | function | duckdb_data_chunk_reset(chunk: duckdb_data_chunk) |  |  |  |  |

| `duckdb_data_chunk_set_size` | function | duckdb_data_chunk_set_size(chunk: duckdb_data_chunk, size: idx_t) |  |  |  |  |

| `duckdb_data_chunk_to_arrow` | function | duckdb_data_chunk_to_arrow(
    arrow_options: duckdb_arrow_options,
    chunk: duckdb_data_chunk,
    out_arrow_array: *mut ArrowArray,
) |  |  |  |  |

| `duckdb_decimal_internal_type` | function | duckdb_decimal_internal_type(type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_decimal_scale` | function | duckdb_decimal_scale(type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_decimal_to_double` | function | duckdb_decimal_to_double(val: duckdb_decimal) |  |  |  |  |

| `duckdb_decimal_width` | function | duckdb_decimal_width(type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_destroy_aggregate_function` | function | duckdb_destroy_aggregate_function(
    aggregate_function: *mut duckdb_aggregate_function,
) |  |  |  |  |

| `duckdb_destroy_aggregate_function_set` | function | duckdb_destroy_aggregate_function_set(
    aggregate_function_set: *mut duckdb_aggregate_function_set,
) |  |  |  |  |

| `duckdb_destroy_arrow` | function | duckdb_destroy_arrow(result: *mut duckdb_arrow) |  |  |  |  |

| `duckdb_destroy_arrow_converted_schema` | function | duckdb_destroy_arrow_converted_schema(
    arrow_converted_schema: *mut duckdb_arrow_converted_schema,
) |  |  |  |  |

| `duckdb_destroy_arrow_options` | function | duckdb_destroy_arrow_options(arrow_options: *mut duckdb_arrow_options) |  |  |  |  |

| `duckdb_destroy_arrow_stream` | function | duckdb_destroy_arrow_stream(stream_p: *mut duckdb_arrow_stream) |  |  |  |  |

| `duckdb_destroy_cast_function` | function | duckdb_destroy_cast_function(cast_function: *mut duckdb_cast_function) |  |  |  |  |

| `duckdb_destroy_client_context` | function | duckdb_destroy_client_context(context: *mut duckdb_client_context) |  |  |  |  |

| `duckdb_destroy_config` | function | duckdb_destroy_config(config: *mut duckdb_config) |  |  |  |  |

| `duckdb_destroy_data_chunk` | function | duckdb_destroy_data_chunk(chunk: *mut duckdb_data_chunk) |  |  |  |  |

| `duckdb_destroy_error_data` | function | duckdb_destroy_error_data(error_data: *mut duckdb_error_data) |  |  |  |  |

| `duckdb_destroy_expression` | function | duckdb_destroy_expression(expr: *mut duckdb_expression) |  |  |  |  |

| `duckdb_destroy_extracted` | function | duckdb_destroy_extracted(
    extracted_statements: *mut duckdb_extracted_statements,
) |  |  |  |  |

| `duckdb_destroy_instance_cache` | function | duckdb_destroy_instance_cache(instance_cache: *mut duckdb_instance_cache) |  |  |  |  |

| `duckdb_destroy_logical_type` | function | duckdb_destroy_logical_type(type_: *mut duckdb_logical_type) |  |  |  |  |

| `duckdb_destroy_pending` | function | duckdb_destroy_pending(pending_result: *mut duckdb_pending_result) |  |  |  |  |

| `duckdb_destroy_prepare` | function | duckdb_destroy_prepare(
    prepared_statement: *mut duckdb_prepared_statement,
) |  |  |  |  |

| `duckdb_destroy_result` | function | duckdb_destroy_result(result: *mut duckdb_result) |  |  |  |  |

| `duckdb_destroy_scalar_function` | function | duckdb_destroy_scalar_function(
    scalar_function: *mut duckdb_scalar_function,
) |  |  |  |  |

| `duckdb_destroy_scalar_function_set` | function | duckdb_destroy_scalar_function_set(
    scalar_function_set: *mut duckdb_scalar_function_set,
) |  |  |  |  |

| `duckdb_destroy_selection_vector` | function | duckdb_destroy_selection_vector(sel: duckdb_selection_vector) |  |  |  |  |

| `duckdb_destroy_table_function` | function | duckdb_destroy_table_function(table_function: *mut duckdb_table_function) |  |  |  |  |

| `duckdb_destroy_task_state` | function | duckdb_destroy_task_state(state: duckdb_task_state) |  |  |  |  |

| `duckdb_destroy_value` | function | duckdb_destroy_value(value: *mut duckdb_value) |  |  |  |  |

| `duckdb_destroy_vector` | function | duckdb_destroy_vector(vector: *mut duckdb_vector) |  |  |  |  |

| `duckdb_disconnect` | function | duckdb_disconnect(connection: *mut duckdb_connection) |  |  |  |  |

| `duckdb_double_to_decimal` | function | duckdb_double_to_decimal(
    val: f64,
    width: u8,
    scale: u8,
) |  |  |  |  |

| `duckdb_double_to_hugeint` | function | duckdb_double_to_hugeint(val: f64) |  |  |  |  |

| `duckdb_double_to_uhugeint` | function | duckdb_double_to_uhugeint(val: f64) |  |  |  |  |

| `duckdb_enum_dictionary_size` | function | duckdb_enum_dictionary_size(type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_enum_dictionary_value` | function | duckdb_enum_dictionary_value(
    type_: duckdb_logical_type,
    index: idx_t,
) |  |  |  |  |

| `duckdb_enum_internal_type` | function | duckdb_enum_internal_type(type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_error_data_error_type` | function | duckdb_error_data_error_type(
    error_data: duckdb_error_data,
) |  |  |  |  |

| `duckdb_error_data_has_error` | function | duckdb_error_data_has_error(error_data: duckdb_error_data) |  |  |  |  |

| `duckdb_error_data_message` | function | duckdb_error_data_message(
    error_data: duckdb_error_data,
) |  |  |  |  |

| `duckdb_execute_n_tasks_state` | function | duckdb_execute_n_tasks_state(
    state: duckdb_task_state,
    max_tasks: idx_t,
) |  |  |  |  |

| `duckdb_execute_pending` | function | duckdb_execute_pending(
    pending_result: duckdb_pending_result,
    out_result: *mut duckdb_result,
) |  |  |  |  |

| `duckdb_execute_prepared` | function | duckdb_execute_prepared(
    prepared_statement: duckdb_prepared_statement,
    out_result: *mut duckdb_result,
) |  |  |  |  |

| `duckdb_execute_prepared_arrow` | function | duckdb_execute_prepared_arrow(
    prepared_statement: duckdb_prepared_statement,
    out_result: *mut duckdb_arrow,
) |  |  |  |  |

| `duckdb_execute_prepared_streaming` | function | duckdb_execute_prepared_streaming(
    prepared_statement: duckdb_prepared_statement,
    out_result: *mut duckdb_result,
) |  |  |  |  |

| `duckdb_execute_tasks` | function | duckdb_execute_tasks(database: duckdb_database, max_tasks: idx_t) |  |  |  |  |

| `duckdb_execute_tasks_state` | function | duckdb_execute_tasks_state(state: duckdb_task_state) |  |  |  |  |

| `duckdb_execution_is_finished` | function | duckdb_execution_is_finished(con: duckdb_connection) |  |  |  |  |

| `duckdb_expression_fold` | function | duckdb_expression_fold(
    context: duckdb_client_context,
    expr: duckdb_expression,
    out_value: *mut duckdb_value,
) |  |  |  |  |

| `duckdb_expression_is_foldable` | function | duckdb_expression_is_foldable(expr: duckdb_expression) |  |  |  |  |

| `duckdb_expression_return_type` | function | duckdb_expression_return_type(
    expr: duckdb_expression,
) |  |  |  |  |

| `duckdb_extract_statements` | function | duckdb_extract_statements(
    connection: duckdb_connection,
    query: *const ::std::os::raw::c_char,
    out_extracted_statements: *mut duckdb_extracted_statements,
) |  |  |  |  |

| `duckdb_extract_statements_error` | function | duckdb_extract_statements_error(
    extracted_statements: duckdb_extracted_statements,
) |  |  |  |  |

| `duckdb_fetch_chunk` | function | duckdb_fetch_chunk(result: duckdb_result) |  |  |  |  |

| `duckdb_finish_execution` | function | duckdb_finish_execution(state: duckdb_task_state) |  |  |  |  |

| `duckdb_free` | function | duckdb_free(ptr: *mut ::std::os::raw::c_void) |  |  |  |  |

| `duckdb_from_date` | function | duckdb_from_date(date: duckdb_date) |  |  |  |  |

| `duckdb_from_time` | function | duckdb_from_time(time: duckdb_time) |  |  |  |  |

| `duckdb_from_time_tz` | function | duckdb_from_time_tz(micros: duckdb_time_tz) |  |  |  |  |

| `duckdb_from_timestamp` | function | duckdb_from_timestamp(ts: duckdb_timestamp) |  |  |  |  |

| `duckdb_function_get_bind_data` | function | duckdb_function_get_bind_data(
    info: duckdb_function_info,
) |  |  |  |  |

| `duckdb_function_get_extra_info` | function | duckdb_function_get_extra_info(
    info: duckdb_function_info,
) |  |  |  |  |

| `duckdb_function_get_init_data` | function | duckdb_function_get_init_data(
    info: duckdb_function_info,
) |  |  |  |  |

| `duckdb_function_get_local_init_data` | function | duckdb_function_get_local_init_data(
    info: duckdb_function_info,
) |  |  |  |  |

| `duckdb_function_set_error` | function | duckdb_function_set_error(
    info: duckdb_function_info,
    error: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_get_bignum` | function | duckdb_get_bignum(val: duckdb_value) |  |  |  |  |

| `duckdb_get_bit` | function | duckdb_get_bit(val: duckdb_value) |  |  |  |  |

| `duckdb_get_blob` | function | duckdb_get_blob(val: duckdb_value) |  |  |  |  |

| `duckdb_get_bool` | function | duckdb_get_bool(val: duckdb_value) |  |  |  |  |

| `duckdb_get_config_flag` | function | duckdb_get_config_flag(
    index: usize,
    out_name: *mut *const ::std::os::raw::c_char,
    out_description: *mut *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_get_date` | function | duckdb_get_date(val: duckdb_value) |  |  |  |  |

| `duckdb_get_decimal` | function | duckdb_get_decimal(val: duckdb_value) |  |  |  |  |

| `duckdb_get_double` | function | duckdb_get_double(val: duckdb_value) |  |  |  |  |

| `duckdb_get_enum_value` | function | duckdb_get_enum_value(value: duckdb_value) |  |  |  |  |

| `duckdb_get_float` | function | duckdb_get_float(val: duckdb_value) |  |  |  |  |

| `duckdb_get_hugeint` | function | duckdb_get_hugeint(val: duckdb_value) |  |  |  |  |

| `duckdb_get_int16` | function | duckdb_get_int16(val: duckdb_value) |  |  |  |  |

| `duckdb_get_int32` | function | duckdb_get_int32(val: duckdb_value) |  |  |  |  |

| `duckdb_get_int64` | function | duckdb_get_int64(val: duckdb_value) |  |  |  |  |

| `duckdb_get_int8` | function | duckdb_get_int8(val: duckdb_value) |  |  |  |  |

| `duckdb_get_interval` | function | duckdb_get_interval(val: duckdb_value) |  |  |  |  |

| `duckdb_get_list_child` | function | duckdb_get_list_child(value: duckdb_value, index: idx_t) |  |  |  |  |

| `duckdb_get_list_size` | function | duckdb_get_list_size(value: duckdb_value) |  |  |  |  |

| `duckdb_get_map_key` | function | duckdb_get_map_key(value: duckdb_value, index: idx_t) |  |  |  |  |

| `duckdb_get_map_size` | function | duckdb_get_map_size(value: duckdb_value) |  |  |  |  |

| `duckdb_get_map_value` | function | duckdb_get_map_value(value: duckdb_value, index: idx_t) |  |  |  |  |

| `duckdb_get_or_create_from_cache` | function | duckdb_get_or_create_from_cache(
    instance_cache: duckdb_instance_cache,
    path: *const ::std::os::raw::c_char,
    out_database: *mut duckdb_database,
    config: duckdb_config,
    out_error: *mut *mut ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_get_profiling_info` | function | duckdb_get_profiling_info(
    connection: duckdb_connection,
) |  |  |  |  |

| `duckdb_get_struct_child` | function | duckdb_get_struct_child(
    value: duckdb_value,
    index: idx_t,
) |  |  |  |  |

| `duckdb_get_table_names` | function | duckdb_get_table_names(
    connection: duckdb_connection,
    query: *const ::std::os::raw::c_char,
    qualified: bool,
) |  |  |  |  |

| `duckdb_get_time` | function | duckdb_get_time(val: duckdb_value) |  |  |  |  |

| `duckdb_get_time_ns` | function | duckdb_get_time_ns(val: duckdb_value) |  |  |  |  |

| `duckdb_get_time_tz` | function | duckdb_get_time_tz(val: duckdb_value) |  |  |  |  |

| `duckdb_get_timestamp` | function | duckdb_get_timestamp(val: duckdb_value) |  |  |  |  |

| `duckdb_get_timestamp_ms` | function | duckdb_get_timestamp_ms(val: duckdb_value) |  |  |  |  |

| `duckdb_get_timestamp_ns` | function | duckdb_get_timestamp_ns(val: duckdb_value) |  |  |  |  |

| `duckdb_get_timestamp_s` | function | duckdb_get_timestamp_s(val: duckdb_value) |  |  |  |  |

| `duckdb_get_timestamp_tz` | function | duckdb_get_timestamp_tz(val: duckdb_value) |  |  |  |  |

| `duckdb_get_type_id` | function | duckdb_get_type_id(type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_get_uhugeint` | function | duckdb_get_uhugeint(val: duckdb_value) |  |  |  |  |

| `duckdb_get_uint16` | function | duckdb_get_uint16(val: duckdb_value) |  |  |  |  |

| `duckdb_get_uint32` | function | duckdb_get_uint32(val: duckdb_value) |  |  |  |  |

| `duckdb_get_uint64` | function | duckdb_get_uint64(val: duckdb_value) |  |  |  |  |

| `duckdb_get_uint8` | function | duckdb_get_uint8(val: duckdb_value) |  |  |  |  |

| `duckdb_get_uuid` | function | duckdb_get_uuid(val: duckdb_value) |  |  |  |  |

| `duckdb_get_value_type` | function | duckdb_get_value_type(val: duckdb_value) |  |  |  |  |

| `duckdb_get_varchar` | function | duckdb_get_varchar(value: duckdb_value) |  |  |  |  |

| `duckdb_hugeint_to_double` | function | duckdb_hugeint_to_double(val: duckdb_hugeint) |  |  |  |  |

| `duckdb_init_get_bind_data` | function | duckdb_init_get_bind_data(
    info: duckdb_init_info,
) |  |  |  |  |

| `duckdb_init_get_column_count` | function | duckdb_init_get_column_count(info: duckdb_init_info) |  |  |  |  |

| `duckdb_init_get_column_index` | function | duckdb_init_get_column_index(
    info: duckdb_init_info,
    column_index: idx_t,
) |  |  |  |  |

| `duckdb_init_get_extra_info` | function | duckdb_init_get_extra_info(
    info: duckdb_init_info,
) |  |  |  |  |

| `duckdb_init_set_error` | function | duckdb_init_set_error(
    info: duckdb_init_info,
    error: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_init_set_init_data` | function | duckdb_init_set_init_data(
    info: duckdb_init_info,
    init_data: *mut ::std::os::raw::c_void,
    destroy: duckdb_delete_callback_t,
) |  |  |  |  |

| `duckdb_init_set_max_threads` | function | duckdb_init_set_max_threads(info: duckdb_init_info, max_threads: idx_t) |  |  |  |  |

| `duckdb_interrupt` | function | duckdb_interrupt(connection: duckdb_connection) |  |  |  |  |

| `duckdb_is_finite_date` | function | duckdb_is_finite_date(date: duckdb_date) |  |  |  |  |

| `duckdb_is_finite_timestamp` | function | duckdb_is_finite_timestamp(ts: duckdb_timestamp) |  |  |  |  |

| `duckdb_is_finite_timestamp_ms` | function | duckdb_is_finite_timestamp_ms(ts: duckdb_timestamp_ms) |  |  |  |  |

| `duckdb_is_finite_timestamp_ns` | function | duckdb_is_finite_timestamp_ns(ts: duckdb_timestamp_ns) |  |  |  |  |

| `duckdb_is_finite_timestamp_s` | function | duckdb_is_finite_timestamp_s(ts: duckdb_timestamp_s) |  |  |  |  |

| `duckdb_is_null_value` | function | duckdb_is_null_value(value: duckdb_value) |  |  |  |  |

| `duckdb_library_version` | function | duckdb_library_version() |  |  |  |  |

| `duckdb_list_type_child_type` | function | duckdb_list_type_child_type(
    type_: duckdb_logical_type,
) |  |  |  |  |

| `duckdb_list_vector_get_child` | function | duckdb_list_vector_get_child(vector: duckdb_vector) |  |  |  |  |

| `duckdb_list_vector_get_size` | function | duckdb_list_vector_get_size(vector: duckdb_vector) |  |  |  |  |

| `duckdb_list_vector_reserve` | function | duckdb_list_vector_reserve(
    vector: duckdb_vector,
    required_capacity: idx_t,
) |  |  |  |  |

| `duckdb_list_vector_set_size` | function | duckdb_list_vector_set_size(
    vector: duckdb_vector,
    size: idx_t,
) |  |  |  |  |

| `duckdb_logical_type_get_alias` | function | duckdb_logical_type_get_alias(
    type_: duckdb_logical_type,
) |  |  |  |  |

| `duckdb_logical_type_set_alias` | function | duckdb_logical_type_set_alias(
    type_: duckdb_logical_type,
    alias: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_malloc` | function | duckdb_malloc(size: usize) |  |  |  |  |

| `duckdb_map_type_key_type` | function | duckdb_map_type_key_type(
    type_: duckdb_logical_type,
) |  |  |  |  |

| `duckdb_map_type_value_type` | function | duckdb_map_type_value_type(
    type_: duckdb_logical_type,
) |  |  |  |  |

| `duckdb_nparams` | function | duckdb_nparams(prepared_statement: duckdb_prepared_statement) |  |  |  |  |

| `duckdb_nullmask_data` | function | duckdb_nullmask_data(result: *mut duckdb_result, col: idx_t) |  |  |  |  |

| `duckdb_open` | function | duckdb_open(
    path: *const ::std::os::raw::c_char,
    out_database: *mut duckdb_database,
) |  |  |  |  |

| `duckdb_open_ext` | function | duckdb_open_ext(
    path: *const ::std::os::raw::c_char,
    out_database: *mut duckdb_database,
    config: duckdb_config,
    out_error: *mut *mut ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_param_logical_type` | function | duckdb_param_logical_type(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
) |  |  |  |  |

| `duckdb_param_type` | function | duckdb_param_type(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
) |  |  |  |  |

| `duckdb_parameter_name` | function | duckdb_parameter_name(
    prepared_statement: duckdb_prepared_statement,
    index: idx_t,
) |  |  |  |  |

| `duckdb_pending_error` | function | duckdb_pending_error(
    pending_result: duckdb_pending_result,
) |  |  |  |  |

| `duckdb_pending_execute_check_state` | function | duckdb_pending_execute_check_state(
    pending_result: duckdb_pending_result,
) |  |  |  |  |

| `duckdb_pending_execute_task` | function | duckdb_pending_execute_task(
    pending_result: duckdb_pending_result,
) |  |  |  |  |

| `duckdb_pending_execution_is_finished` | function | duckdb_pending_execution_is_finished(
    pending_state: duckdb_pending_state,
) |  |  |  |  |

| `duckdb_pending_prepared` | function | duckdb_pending_prepared(
    prepared_statement: duckdb_prepared_statement,
    out_result: *mut duckdb_pending_result,
) |  |  |  |  |

| `duckdb_pending_prepared_streaming` | function | duckdb_pending_prepared_streaming(
    prepared_statement: duckdb_prepared_statement,
    out_result: *mut duckdb_pending_result,
) |  |  |  |  |

| `duckdb_prepare` | function | duckdb_prepare(
    connection: duckdb_connection,
    query: *const ::std::os::raw::c_char,
    out_prepared_statement: *mut duckdb_prepared_statement,
) |  |  |  |  |

| `duckdb_prepare_error` | function | duckdb_prepare_error(
    prepared_statement: duckdb_prepared_statement,
) |  |  |  |  |

| `duckdb_prepare_extracted_statement` | function | duckdb_prepare_extracted_statement(
    connection: duckdb_connection,
    extracted_statements: duckdb_extracted_statements,
    index: idx_t,
    out_prepared_statement: *mut duckdb_prepared_statement,
) |  |  |  |  |

| `duckdb_prepared_arrow_schema` | function | duckdb_prepared_arrow_schema(
    prepared: duckdb_prepared_statement,
    out_schema: *mut duckdb_arrow_schema,
) |  |  |  |  |

| `duckdb_prepared_statement_column_count` | function | duckdb_prepared_statement_column_count(
    prepared_statement: duckdb_prepared_statement,
) |  |  |  |  |

| `duckdb_prepared_statement_column_logical_type` | function | duckdb_prepared_statement_column_logical_type(
    prepared_statement: duckdb_prepared_statement,
    col_idx: idx_t,
) |  |  |  |  |

| `duckdb_prepared_statement_column_name` | function | duckdb_prepared_statement_column_name(
    prepared_statement: duckdb_prepared_statement,
    col_idx: idx_t,
) |  |  |  |  |

| `duckdb_prepared_statement_column_type` | function | duckdb_prepared_statement_column_type(
    prepared_statement: duckdb_prepared_statement,
    col_idx: idx_t,
) |  |  |  |  |

| `duckdb_prepared_statement_type` | function | duckdb_prepared_statement_type(
    statement: duckdb_prepared_statement,
) |  |  |  |  |

| `duckdb_profiling_info_get_child` | function | duckdb_profiling_info_get_child(
    info: duckdb_profiling_info,
    index: idx_t,
) |  |  |  |  |

| `duckdb_profiling_info_get_child_count` | function | duckdb_profiling_info_get_child_count(
    info: duckdb_profiling_info,
) |  |  |  |  |

| `duckdb_profiling_info_get_metrics` | function | duckdb_profiling_info_get_metrics(
    info: duckdb_profiling_info,
) |  |  |  |  |

| `duckdb_profiling_info_get_value` | function | duckdb_profiling_info_get_value(
    info: duckdb_profiling_info,
    key: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_query` | function | duckdb_query(
    connection: duckdb_connection,
    query: *const ::std::os::raw::c_char,
    out_result: *mut duckdb_result,
) |  |  |  |  |

| `duckdb_query_arrow` | function | duckdb_query_arrow(
    connection: duckdb_connection,
    query: *const ::std::os::raw::c_char,
    out_result: *mut duckdb_arrow,
) |  |  |  |  |

| `duckdb_query_arrow_array` | function | duckdb_query_arrow_array(
    result: duckdb_arrow,
    out_array: *mut duckdb_arrow_array,
) |  |  |  |  |

| `duckdb_query_arrow_error` | function | duckdb_query_arrow_error(
    result: duckdb_arrow,
) |  |  |  |  |

| `duckdb_query_arrow_schema` | function | duckdb_query_arrow_schema(
    result: duckdb_arrow,
    out_schema: *mut duckdb_arrow_schema,
) |  |  |  |  |

| `duckdb_query_progress` | function | duckdb_query_progress(
    connection: duckdb_connection,
) |  |  |  |  |

| `duckdb_register_aggregate_function` | function | duckdb_register_aggregate_function(
    con: duckdb_connection,
    aggregate_function: duckdb_aggregate_function,
) |  |  |  |  |

| `duckdb_register_aggregate_function_set` | function | duckdb_register_aggregate_function_set(
    con: duckdb_connection,
    set: duckdb_aggregate_function_set,
) |  |  |  |  |

| `duckdb_register_cast_function` | function | duckdb_register_cast_function(
    con: duckdb_connection,
    cast_function: duckdb_cast_function,
) |  |  |  |  |

| `duckdb_register_logical_type` | function | duckdb_register_logical_type(
    con: duckdb_connection,
    type_: duckdb_logical_type,
    info: duckdb_create_type_info,
) |  |  |  |  |

| `duckdb_register_scalar_function` | function | duckdb_register_scalar_function(
    con: duckdb_connection,
    scalar_function: duckdb_scalar_function,
) |  |  |  |  |

| `duckdb_register_scalar_function_set` | function | duckdb_register_scalar_function_set(
    con: duckdb_connection,
    set: duckdb_scalar_function_set,
) |  |  |  |  |

| `duckdb_register_table_function` | function | duckdb_register_table_function(
    con: duckdb_connection,
    function: duckdb_table_function,
) |  |  |  |  |

| `duckdb_replacement_scan_add_parameter` | function | duckdb_replacement_scan_add_parameter(
    info: duckdb_replacement_scan_info,
    parameter: duckdb_value,
) |  |  |  |  |

| `duckdb_replacement_scan_set_error` | function | duckdb_replacement_scan_set_error(
    info: duckdb_replacement_scan_info,
    error: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_replacement_scan_set_function_name` | function | duckdb_replacement_scan_set_function_name(
    info: duckdb_replacement_scan_info,
    function_name: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_result_arrow_array` | function | duckdb_result_arrow_array(
    result: duckdb_result,
    chunk: duckdb_data_chunk,
    out_array: *mut duckdb_arrow_array,
) |  |  |  |  |

| `duckdb_result_chunk_count` | function | duckdb_result_chunk_count(result: duckdb_result) |  |  |  |  |

| `duckdb_result_error` | function | duckdb_result_error(
    result: *mut duckdb_result,
) |  |  |  |  |

| `duckdb_result_error_type` | function | duckdb_result_error_type(result: *mut duckdb_result) |  |  |  |  |

| `duckdb_result_get_arrow_options` | function | duckdb_result_get_arrow_options(
    result: *mut duckdb_result,
) |  |  |  |  |

| `duckdb_result_get_chunk` | function | duckdb_result_get_chunk(
    result: duckdb_result,
    chunk_index: idx_t,
) |  |  |  |  |

| `duckdb_result_is_streaming` | function | duckdb_result_is_streaming(result: duckdb_result) |  |  |  |  |

| `duckdb_result_return_type` | function | duckdb_result_return_type(result: duckdb_result) |  |  |  |  |

| `duckdb_result_statement_type` | function | duckdb_result_statement_type(
    result: duckdb_result,
) |  |  |  |  |

| `duckdb_row_count` | function | duckdb_row_count(result: *mut duckdb_result) |  |  |  |  |

| `duckdb_rows_changed` | function | duckdb_rows_changed(result: *mut duckdb_result) |  |  |  |  |

| `duckdb_rs_extension_api_init` | function | duckdb_rs_extension_api_init(
    info: duckdb_extension_info,
    access: *const duckdb_extension_access,
    version: &str,
) |  |  |  |  |

| `duckdb_scalar_function_add_parameter` | function | duckdb_scalar_function_add_parameter(
    scalar_function: duckdb_scalar_function,
    type_: duckdb_logical_type,
) |  |  |  |  |

| `duckdb_scalar_function_bind_get_argument` | function | duckdb_scalar_function_bind_get_argument(
    info: duckdb_bind_info,
    index: idx_t,
) |  |  |  |  |

| `duckdb_scalar_function_bind_get_argument_count` | function | duckdb_scalar_function_bind_get_argument_count(
    info: duckdb_bind_info,
) |  |  |  |  |

| `duckdb_scalar_function_bind_get_extra_info` | function | duckdb_scalar_function_bind_get_extra_info(
    info: duckdb_bind_info,
) |  |  |  |  |

| `duckdb_scalar_function_bind_set_error` | function | duckdb_scalar_function_bind_set_error(
    info: duckdb_bind_info,
    error: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_scalar_function_get_bind_data` | function | duckdb_scalar_function_get_bind_data(
    info: duckdb_function_info,
) |  |  |  |  |

| `duckdb_scalar_function_get_client_context` | function | duckdb_scalar_function_get_client_context(
    info: duckdb_bind_info,
    out_context: *mut duckdb_client_context,
) |  |  |  |  |

| `duckdb_scalar_function_get_extra_info` | function | duckdb_scalar_function_get_extra_info(
    info: duckdb_function_info,
) |  |  |  |  |

| `duckdb_scalar_function_set_bind` | function | duckdb_scalar_function_set_bind(
    scalar_function: duckdb_scalar_function,
    bind: duckdb_scalar_function_bind_t,
) |  |  |  |  |

| `duckdb_scalar_function_set_bind_data` | function | duckdb_scalar_function_set_bind_data(
    info: duckdb_bind_info,
    bind_data: *mut ::std::os::raw::c_void,
    destroy: duckdb_delete_callback_t,
) |  |  |  |  |

| `duckdb_scalar_function_set_bind_data_copy` | function | duckdb_scalar_function_set_bind_data_copy(
    info: duckdb_bind_info,
    copy: duckdb_copy_callback_t,
) |  |  |  |  |

| `duckdb_scalar_function_set_error` | function | duckdb_scalar_function_set_error(
    info: duckdb_function_info,
    error: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_scalar_function_set_extra_info` | function | duckdb_scalar_function_set_extra_info(
    scalar_function: duckdb_scalar_function,
    extra_info: *mut ::std::os::raw::c_void,
    destroy: duckdb_delete_callback_t,
) |  |  |  |  |

| `duckdb_scalar_function_set_function` | function | duckdb_scalar_function_set_function(
    scalar_function: duckdb_scalar_function,
    function: duckdb_scalar_function_t,
) |  |  |  |  |

| `duckdb_scalar_function_set_name` | function | duckdb_scalar_function_set_name(
    scalar_function: duckdb_scalar_function,
    name: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_scalar_function_set_return_type` | function | duckdb_scalar_function_set_return_type(
    scalar_function: duckdb_scalar_function,
    type_: duckdb_logical_type,
) |  |  |  |  |

| `duckdb_scalar_function_set_special_handling` | function | duckdb_scalar_function_set_special_handling(
    scalar_function: duckdb_scalar_function,
) |  |  |  |  |

| `duckdb_scalar_function_set_varargs` | function | duckdb_scalar_function_set_varargs(
    scalar_function: duckdb_scalar_function,
    type_: duckdb_logical_type,
) |  |  |  |  |

| `duckdb_scalar_function_set_volatile` | function | duckdb_scalar_function_set_volatile(
    scalar_function: duckdb_scalar_function,
) |  |  |  |  |

| `duckdb_schema_from_arrow` | function | duckdb_schema_from_arrow(
    connection: duckdb_connection,
    schema: *mut ArrowSchema,
    out_types: *mut duckdb_arrow_converted_schema,
) |  |  |  |  |

| `duckdb_selection_vector_get_data_ptr` | function | duckdb_selection_vector_get_data_ptr(
    sel: duckdb_selection_vector,
) |  |  |  |  |

| `duckdb_set_config` | function | duckdb_set_config(
    config: duckdb_config,
    name: *const ::std::os::raw::c_char,
    option: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_slice_vector` | function | duckdb_slice_vector(
    vector: duckdb_vector,
    sel: duckdb_selection_vector,
    len: idx_t,
) |  |  |  |  |

| `duckdb_stream_fetch_chunk` | function | duckdb_stream_fetch_chunk(result: duckdb_result) |  |  |  |  |

| `duckdb_string_is_inlined` | function | duckdb_string_is_inlined(string: duckdb_string_t) |  |  |  |  |

| `duckdb_string_t_data` | function | duckdb_string_t_data(
    string: *mut duckdb_string_t,
) |  |  |  |  |

| `duckdb_string_t_length` | function | duckdb_string_t_length(string: duckdb_string_t) |  |  |  |  |

| `duckdb_struct_type_child_count` | function | duckdb_struct_type_child_count(type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_struct_type_child_name` | function | duckdb_struct_type_child_name(
    type_: duckdb_logical_type,
    index: idx_t,
) |  |  |  |  |

| `duckdb_struct_type_child_type` | function | duckdb_struct_type_child_type(
    type_: duckdb_logical_type,
    index: idx_t,
) |  |  |  |  |

| `duckdb_struct_vector_get_child` | function | duckdb_struct_vector_get_child(
    vector: duckdb_vector,
    index: idx_t,
) |  |  |  |  |

| `duckdb_table_description_create` | function | duckdb_table_description_create(
    connection: duckdb_connection,
    schema: *const ::std::os::raw::c_char,
    table: *const ::std::os::raw::c_char,
    out: *mut duckdb_table_description,
) |  |  |  |  |

| `duckdb_table_description_create_ext` | function | duckdb_table_description_create_ext(
    connection: duckdb_connection,
    catalog: *const ::std::os::raw::c_char,
    schema: *const ::std::os::raw::c_char,
    table: *const ::std::os::raw::c_char,
    out: *mut duckdb_table_description,
) |  |  |  |  |

| `duckdb_table_description_destroy` | function | duckdb_table_description_destroy(
    table_description: *mut duckdb_table_description,
) |  |  |  |  |

| `duckdb_table_description_error` | function | duckdb_table_description_error(
    table_description: duckdb_table_description,
) |  |  |  |  |

| `duckdb_table_description_get_column_name` | function | duckdb_table_description_get_column_name(
    table_description: duckdb_table_description,
    index: idx_t,
) |  |  |  |  |

| `duckdb_table_function_add_named_parameter` | function | duckdb_table_function_add_named_parameter(
    table_function: duckdb_table_function,
    name: *const ::std::os::raw::c_char,
    type_: duckdb_logical_type,
) |  |  |  |  |

| `duckdb_table_function_add_parameter` | function | duckdb_table_function_add_parameter(
    table_function: duckdb_table_function,
    type_: duckdb_logical_type,
) |  |  |  |  |

| `duckdb_table_function_get_client_context` | function | duckdb_table_function_get_client_context(
    info: duckdb_bind_info,
    out_context: *mut duckdb_client_context,
) |  |  |  |  |

| `duckdb_table_function_set_bind` | function | duckdb_table_function_set_bind(
    table_function: duckdb_table_function,
    bind: duckdb_table_function_bind_t,
) |  |  |  |  |

| `duckdb_table_function_set_extra_info` | function | duckdb_table_function_set_extra_info(
    table_function: duckdb_table_function,
    extra_info: *mut ::std::os::raw::c_void,
    destroy: duckdb_delete_callback_t,
) |  |  |  |  |

| `duckdb_table_function_set_function` | function | duckdb_table_function_set_function(
    table_function: duckdb_table_function,
    function: duckdb_table_function_t,
) |  |  |  |  |

| `duckdb_table_function_set_init` | function | duckdb_table_function_set_init(
    table_function: duckdb_table_function,
    init: duckdb_table_function_init_t,
) |  |  |  |  |

| `duckdb_table_function_set_local_init` | function | duckdb_table_function_set_local_init(
    table_function: duckdb_table_function,
    init: duckdb_table_function_init_t,
) |  |  |  |  |

| `duckdb_table_function_set_name` | function | duckdb_table_function_set_name(
    table_function: duckdb_table_function,
    name: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_table_function_supports_projection_pushdown` | function | duckdb_table_function_supports_projection_pushdown(
    table_function: duckdb_table_function,
    pushdown: bool,
) |  |  |  |  |

| `duckdb_task_state_is_finished` | function | duckdb_task_state_is_finished(state: duckdb_task_state) |  |  |  |  |

| `duckdb_to_arrow_schema` | function | duckdb_to_arrow_schema(
    arrow_options: duckdb_arrow_options,
    types: *mut duckdb_logical_type,
    names: *mut *const ::std::os::raw::c_char,
    column_count: idx_t,
    out_schema: *mut ArrowSchema,
) |  |  |  |  |

| `duckdb_to_date` | function | duckdb_to_date(date: duckdb_date_struct) |  |  |  |  |

| `duckdb_to_time` | function | duckdb_to_time(time: duckdb_time_struct) |  |  |  |  |

| `duckdb_to_timestamp` | function | duckdb_to_timestamp(ts: duckdb_timestamp_struct) |  |  |  |  |

| `duckdb_uhugeint_to_double` | function | duckdb_uhugeint_to_double(val: duckdb_uhugeint) |  |  |  |  |

| `duckdb_union_type_member_count` | function | duckdb_union_type_member_count(type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_union_type_member_name` | function | duckdb_union_type_member_name(
    type_: duckdb_logical_type,
    index: idx_t,
) |  |  |  |  |

| `duckdb_union_type_member_type` | function | duckdb_union_type_member_type(
    type_: duckdb_logical_type,
    index: idx_t,
) |  |  |  |  |

| `duckdb_validity_row_is_valid` | function | duckdb_validity_row_is_valid(validity: *mut u64, row: idx_t) |  |  |  |  |

| `duckdb_validity_set_row_invalid` | function | duckdb_validity_set_row_invalid(validity: *mut u64, row: idx_t) |  |  |  |  |

| `duckdb_validity_set_row_valid` | function | duckdb_validity_set_row_valid(validity: *mut u64, row: idx_t) |  |  |  |  |

| `duckdb_validity_set_row_validity` | function | duckdb_validity_set_row_validity(
    validity: *mut u64,
    row: idx_t,
    valid: bool,
) |  |  |  |  |

| `duckdb_value_blob` | function | duckdb_value_blob(
    result: *mut duckdb_result,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_value_boolean` | function | duckdb_value_boolean(
    result: *mut duckdb_result,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_value_date` | function | duckdb_value_date(
    result: *mut duckdb_result,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_value_decimal` | function | duckdb_value_decimal(
    result: *mut duckdb_result,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_value_double` | function | duckdb_value_double(
    result: *mut duckdb_result,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_value_float` | function | duckdb_value_float(
    result: *mut duckdb_result,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_value_hugeint` | function | duckdb_value_hugeint(
    result: *mut duckdb_result,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_value_int16` | function | duckdb_value_int16(
    result: *mut duckdb_result,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_value_int32` | function | duckdb_value_int32(
    result: *mut duckdb_result,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_value_int64` | function | duckdb_value_int64(
    result: *mut duckdb_result,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_value_int8` | function | duckdb_value_int8(
    result: *mut duckdb_result,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_value_interval` | function | duckdb_value_interval(
    result: *mut duckdb_result,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_value_is_null` | function | duckdb_value_is_null(
    result: *mut duckdb_result,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_value_string` | function | duckdb_value_string(
    result: *mut duckdb_result,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_value_string_internal` | function | duckdb_value_string_internal(
    result: *mut duckdb_result,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_value_time` | function | duckdb_value_time(
    result: *mut duckdb_result,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_value_timestamp` | function | duckdb_value_timestamp(
    result: *mut duckdb_result,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_value_to_string` | function | duckdb_value_to_string(
    value: duckdb_value,
) |  |  |  |  |

| `duckdb_value_uhugeint` | function | duckdb_value_uhugeint(
    result: *mut duckdb_result,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_value_uint16` | function | duckdb_value_uint16(
    result: *mut duckdb_result,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_value_uint32` | function | duckdb_value_uint32(
    result: *mut duckdb_result,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_value_uint64` | function | duckdb_value_uint64(
    result: *mut duckdb_result,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_value_uint8` | function | duckdb_value_uint8(
    result: *mut duckdb_result,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_value_varchar` | function | duckdb_value_varchar(
    result: *mut duckdb_result,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_value_varchar_internal` | function | duckdb_value_varchar_internal(
    result: *mut duckdb_result,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_vector_assign_string_element` | function | duckdb_vector_assign_string_element(
    vector: duckdb_vector,
    index: idx_t,
    str_: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_vector_assign_string_element_len` | function | duckdb_vector_assign_string_element_len(
    vector: duckdb_vector,
    index: idx_t,
    str_: *const ::std::os::raw::c_char,
    str_len: idx_t,
) |  |  |  |  |

| `duckdb_vector_copy_sel` | function | duckdb_vector_copy_sel(
    src: duckdb_vector,
    dst: duckdb_vector,
    sel: duckdb_selection_vector,
    src_count: idx_t,
    src_offset: idx_t,
    dst_offset: idx_t,
) |  |  |  |  |

| `duckdb_vector_ensure_validity_writable` | function | duckdb_vector_ensure_validity_writable(vector: duckdb_vector) |  |  |  |  |

| `duckdb_vector_get_column_type` | function | duckdb_vector_get_column_type(
    vector: duckdb_vector,
) |  |  |  |  |

| `duckdb_vector_get_data` | function | duckdb_vector_get_data(
    vector: duckdb_vector,
) |  |  |  |  |

| `duckdb_vector_get_validity` | function | duckdb_vector_get_validity(vector: duckdb_vector) |  |  |  |  |

| `duckdb_vector_reference_value` | function | duckdb_vector_reference_value(vector: duckdb_vector, value: duckdb_value) |  |  |  |  |

| `duckdb_vector_reference_vector` | function | duckdb_vector_reference_vector(
    to_vector: duckdb_vector,
    from_vector: duckdb_vector,
) |  |  |  |  |

| `duckdb_vector_size` | function | duckdb_vector_size() |  |  |  |  |

| `ArrowArray` | struct |  |  |  |  |  |

| `ArrowSchema` | struct |  |  |  |  |  |


### crates/frozen-duckdb-sys/src/error.rs

| `ErrorCode` | enum |  |  |  |  |  |

| `code_to_str` | function | code_to_str(_: duckdb_state) |  |  |  |  |

| `new` | function | new(result_code: duckdb_state) |  |  |  |  |

| `Error` | struct |  |  |  |  |  |


### crates/frozen-duckdb-sys/src/string.rs

| `from_ptr` | function | from_ptr(ptr: *const c_char) |  |  |  |  |

| `from_raw_parts` | function | from_raw_parts(ptr: *const c_char, len: usize) |  |  |  |  |

| `DuckDbString` | struct |  |  |  |  |  |


### crates/frozen-duckdb/src/architecture.rs

| `detect` | function | detect() |  |  |  |  |

| `get_binary_name` | function | get_binary_name() |  |  |  |  |

| `is_supported` | function | is_supported(arch: &str) |  |  |  |  |


### crates/frozen-duckdb/src/cli/commands.rs

| `Commands` | enum |  |  |  |  |  |

| `Cli` | struct |  |  |  |  |  |


### crates/frozen-duckdb/src/cli/dataset_manager.rs

| `convert_dataset` | function | convert_dataset(
        &self,
        input: &str,
        output: &str,
        input_format: &str,
        output_format: &str,
    ) |  |  |  |  |

| `download_chinook` | function | download_chinook(&self, output_dir: &str, format: &str) |  |  |  |  |

| `download_tpch` | function | download_tpch(&self, output_dir: &str, format: &str) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `show_info` | function | show_info(&self) |  |  |  |  |

| `DatasetManager` | struct |  |  |  |  |  |


### crates/frozen-duckdb/src/cli/flock_manager.rs

| `all_passed` | function | all_passed(&self) |  |  |  |  |

| `complete_text` | function | complete_text(&self, prompt: &str, model: &str) |  |  |  |  |

| `format_results` | function | format_results(&self) |  |  |  |  |

| `generate_embeddings` | function | generate_embeddings(
        &self,
        texts: Vec<String>,
        model: &str,
        normalize: bool,
    ) |  |  |  |  |

| `is_flock_ready` | function | is_flock_ready(&self) |  |  |  |  |

| `llm_filter` | function | llm_filter(
        &self,
        criteria: &str,
        input_file: &str,
        model: &str,
        positive_only: bool,
    ) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `semantic_search` | function | semantic_search(
        &self,
        query: &str,
        _corpus: &str,
        _threshold: f32,
        _limit: usize,
    ) |  |  |  |  |

| `setup_ollama` | function | setup_ollama(
        &self,
        ollama_url: &str,
        text_model: &str,
        embedding_model: &str,
        skip_verification: bool,
    ) |  |  |  |  |

| `success_rate` | function | success_rate(&self) |  |  |  |  |

| `summarize_texts` | function | summarize_texts(
        &self,
        texts: Vec<String>,
        strategy: &str,
        max_length: usize,
        model: &str,
    ) |  |  |  |  |

| `validate_ffi` | function | validate_ffi(&self) |  |  |  |  |

| `FFIValidationResult` | struct |  |  |  |  |  |

| `FlockManager` | struct |  |  |  |  |  |

| `ValidationLayerResult` | struct |  |  |  |  |  |


### crates/frozen-duckdb/src/duckdb/appender/arrow.rs

| `append_record_batch` | function | append_record_batch(&mut self, record_batch: RecordBatch) |  |  |  |  |


### crates/frozen-duckdb/src/duckdb/appender/mod.rs

| `flush` | function | flush(&mut self) |  |  |  |  |

| `Appender` | struct |  |  |  |  |  |


### crates/frozen-duckdb/src/duckdb/appender_params.rs

| `AppenderParamsFromIter` | struct |  |  |  |  |  |

| `AppenderParams` | trait |  |  |  |  |  |

| `Sealed` | trait |  |  |  |  |  |


### crates/frozen-duckdb/src/duckdb/arrow_batch.rs

| `get_schema` | function | get_schema(&self) |  |  |  |  |

| `Arrow` | struct |  |  |  |  |  |

| `ArrowStream` | struct |  |  |  |  |  |


### crates/frozen-duckdb/src/duckdb/cache.rs

| `discard` | function | discard(mut self) |  |  |  |  |

| `flush_prepared_statement_cache` | function | flush_prepared_statement_cache(&self) |  |  |  |  |

| `prepare_cached` | function | prepare_cached(&self, sql: &str) |  |  |  |  |

| `set_prepared_statement_cache_capacity` | function | set_prepared_statement_cache_capacity(&self, capacity: usize) |  |  |  |  |

| `with_capacity` | function | with_capacity(capacity: usize) |  |  |  |  |

| `CachedStatement` | struct |  |  |  |  |  |

| `StatementCache` | struct |  |  |  |  |  |


### crates/frozen-duckdb/src/duckdb/column.rs

| `column_count` | function | column_count(&self) |  |  |  |  |

| `column_index` | function | column_index(&self, name: &str) |  |  |  |  |

| `column_name` | function | column_name(&self, col: usize) |  |  |  |  |

| `column_names` | function | column_names(&self) |  |  |  |  |

| `column_type` | function | column_type(&self, idx: usize) |  |  |  |  |

| `decl_type` | function | decl_type(&self) |  |  |  |  |

| `name` | function | name(&self) |  |  |  |  |

| `Column` | struct |  |  |  |  |  |


### crates/frozen-duckdb/src/duckdb/config.rs

| `AccessMode` | enum |  |  |  |  |  |

| `DefaultNullOrder` | enum |  |  |  |  |  |

| `DefaultOrder` | enum |  |  |  |  |  |

| `access_mode` | function | access_mode(mut self, mode: AccessMode) |  |  |  |  |

| `allow_unsigned_extensions` | function | allow_unsigned_extensions(mut self) |  |  |  |  |

| `custom_user_agent` | function | custom_user_agent(mut self, custom_user_agent: &str) |  |  |  |  |

| `default_null_order` | function | default_null_order(mut self, null_order: DefaultNullOrder) |  |  |  |  |

| `default_order` | function | default_order(mut self, order: DefaultOrder) |  |  |  |  |

| `enable_autoload_extension` | function | enable_autoload_extension(mut self, enabled: bool) |  |  |  |  |

| `enable_external_access` | function | enable_external_access(mut self, enabled: bool) |  |  |  |  |

| `enable_object_cache` | function | enable_object_cache(mut self, enabled: bool) |  |  |  |  |

| `max_memory` | function | max_memory(mut self, memory: &str) |  |  |  |  |

| `threads` | function | threads(mut self, thread_num: i64) |  |  |  |  |

| `with` | function | with(mut self, key: impl AsRef<str>, value: impl AsRef<str>) |  |  |  |  |

| `Config` | struct |  |  |  |  |  |


### crates/frozen-duckdb/src/duckdb/core/data_chunk.rs

| `array_vector` | function | array_vector(&self, idx: usize) |  |  |  |  |

| `flat_vector` | function | flat_vector(&self, idx: usize) |  |  |  |  |

| `get_ptr` | function | get_ptr(&self) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `list_vector` | function | list_vector(&self, idx: usize) |  |  |  |  |

| `new` | function | new(logical_types: &[LogicalTypeHandle]) |  |  |  |  |

| `num_columns` | function | num_columns(&self) |  |  |  |  |

| `set_len` | function | set_len(&self, new_len: usize) |  |  |  |  |

| `struct_vector` | function | struct_vector(&self, idx: usize) |  |  |  |  |

| `DataChunkHandle` | struct |  |  |  |  |  |


### crates/frozen-duckdb/src/duckdb/core/logical_type.rs

| `LogicalTypeId` | enum |  |  |  |  |  |

| `array` | function | array(child_type: &Self, array_size: u64) |  |  |  |  |

| `child` | function | child(&self, idx: usize) |  |  |  |  |

| `child_name` | function | child_name(&self, idx: usize) |  |  |  |  |

| `decimal` | function | decimal(width: u8, scale: u8) |  |  |  |  |

| `decimal_scale` | function | decimal_scale(&self) |  |  |  |  |

| `decimal_width` | function | decimal_width(&self) |  |  |  |  |

| `id` | function | id(&self) |  |  |  |  |

| `list` | function | list(child_type: &Self) |  |  |  |  |

| `map` | function | map(key: &Self, value: &Self) |  |  |  |  |

| `num_children` | function | num_children(&self) |  |  |  |  |

| `struct_type` | function | struct_type(fields: &[(&str, Self) |  |  |  |  |

| `union_type` | function | union_type(fields: &[(&str, Self) |  |  |  |  |

| `LogicalTypeHandle` | struct |  |  |  |  |  |


### crates/frozen-duckdb/src/duckdb/core/vector.rs

| `array_child` | function | array_child(&self) |  |  |  |  |

| `array_vector_child` | function | array_vector_child(&self, idx: usize) |  |  |  |  |

| `capacity` | function | capacity(&self) |  |  |  |  |

| `child` | function | child(&self, capacity: usize) |  |  |  |  |

| `child_name` | function | child_name(&self, idx: usize) |  |  |  |  |

| `get_array_size` | function | get_array_size(&self) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `list_child` | function | list_child(&self) |  |  |  |  |

| `list_vector_child` | function | list_vector_child(&self, idx: usize) |  |  |  |  |

| `logical_type` | function | logical_type(&self) |  |  |  |  |

| `num_children` | function | num_children(&self) |  |  |  |  |

| `row_is_null` | function | row_is_null(&self, row: u64) |  |  |  |  |

| `set_entry` | function | set_entry(&mut self, idx: usize, offset: usize, length: usize) |  |  |  |  |

| `set_len` | function | set_len(&self, new_len: usize) |  |  |  |  |

| `set_null` | function | set_null(&mut self, row: usize) |  |  |  |  |

| `struct_child` | function | struct_child(&self, capacity: usize) |  |  |  |  |

| `struct_vector_child` | function | struct_vector_child(&self, idx: usize) |  |  |  |  |

| `ArrayVector` | struct |  |  |  |  |  |

| `FlatVector` | struct |  |  |  |  |  |

| `ListVector` | struct |  |  |  |  |  |

| `StructVector` | struct |  |  |  |  |  |

| `Inserter` | trait |  |  |  |  |  |

| `Vector` | trait |  |  |  |  |  |


### crates/frozen-duckdb/src/duckdb/error.rs

| `Error` | enum |  |  |  |  |  |

| `result_from_duckdb_appender` | function | result_from_duckdb_appender(
    code: ffi::duckdb_state,
    appender: *mut ffi::duckdb_appender,
) |  |  |  |  |

| `result_from_duckdb_arrow` | function | result_from_duckdb_arrow(code: ffi::duckdb_state, mut out: ffi::duckdb_arrow) |  |  |  |  |

| `result_from_duckdb_extract` | function | result_from_duckdb_extract(
    num_statements: ffi::idx_t,
    mut extracted: ffi::duckdb_extracted_statements,
) |  |  |  |  |

| `result_from_duckdb_prepare` | function | result_from_duckdb_prepare(
    code: ffi::duckdb_state,
    mut prepare: ffi::duckdb_prepared_statement,
) |  |  |  |  |


### crates/frozen-duckdb/src/duckdb/inner_connection.rs

| `close` | function | close(&mut self) |  |  |  |  |

| `execute` | function | execute(&mut self, sql: &str) |  |  |  |  |

| `get_interrupt_handle` | function | get_interrupt_handle(&self) |  |  |  |  |

| `interrupt` | function | interrupt(&self) |  |  |  |  |

| `is_autocommit` | function | is_autocommit(&self) |  |  |  |  |

| `new` | function | new(db: ffi::duckdb_database, owned: bool) |  |  |  |  |

| `open_with_flags` | function | open_with_flags(c_path: &CStr, config: Config) |  |  |  |  |

| `try_clone` | function | try_clone(&self) |  |  |  |  |

| `InnerConnection` | struct |  |  |  |  |  |

| `InterruptHandle` | struct |  |  |  |  |  |


### crates/frozen-duckdb/src/duckdb/mod.rs

| `DatabaseName` | enum |  |  |  |  |  |

| `appender` | function | appender(&self, table: &str) |  |  |  |  |

| `appender_to_db` | function | appender_to_db(&self, table: &str, schema: &str) |  |  |  |  |

| `checked_memory_handle` | function | checked_memory_handle() |  |  |  |  |

| `close` | function | close(self) |  |  |  |  |

| `execute_batch` | function | execute_batch(&self, sql: &str) |  |  |  |  |

| `interrupt_handle` | function | interrupt_handle(&self) |  |  |  |  |

| `is_autocommit` | function | is_autocommit(&self) |  |  |  |  |

| `open_from_raw` | function | open_from_raw(raw: ffi::duckdb_database) |  |  |  |  |

| `open_in_memory` | function | open_in_memory() |  |  |  |  |

| `open_in_memory_with_flags` | function | open_in_memory_with_flags(config: Config) |  |  |  |  |

| `path` | function | path(&self) |  |  |  |  |

| `prepare` | function | prepare(&self, sql: &str) |  |  |  |  |

| `try_clone` | function | try_clone(&self) |  |  |  |  |

| `version` | function | version(&self) |  |  |  |  |

| `Connection` | struct |  |  |  |  |  |

| `OptionalExt` | trait |  |  |  |  |  |


### crates/frozen-duckdb/src/duckdb/params.rs

| `ParamsFromIter` | struct |  |  |  |  |  |

| `Params` | trait |  |  |  |  |  |

| `Sealed` | trait |  |  |  |  |  |


### crates/frozen-duckdb/src/duckdb/polars_dataframe.rs

| `get_schema` | function | get_schema(&self) |  |  |  |  |

| `Polars` | struct |  |  |  |  |  |


### crates/frozen-duckdb/src/duckdb/pragma.rs

| `as_str` | function | as_str(&self) |  |  |  |  |

| `close_brace` | function | close_brace(&mut self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `open_brace` | function | open_brace(&mut self) |  |  |  |  |

| `pragma_update` | function | pragma_update(
        &self,
        schema_name: Option<DatabaseName<'_>>,
        pragma_name: &str,
        pragma_value: &dyn ToSql,
    ) |  |  |  |  |

| `push_dot` | function | push_dot(&mut self) |  |  |  |  |

| `push_equal_sign` | function | push_equal_sign(&mut self) |  |  |  |  |

| `push_identifier` | function | push_identifier(&mut self, s: &str) |  |  |  |  |

| `push_int` | function | push_int(&mut self, i: i64) |  |  |  |  |

| `push_keyword` | function | push_keyword(&mut self, keyword: &str) |  |  |  |  |

| `push_pragma` | function | push_pragma(
        &mut self,
        schema_name: Option<DatabaseName<'_>>,
        pragma_name: &str,
    ) |  |  |  |  |

| `push_real` | function | push_real(&mut self, f: f64) |  |  |  |  |

| `push_schema_name` | function | push_schema_name(&mut self, schema_name: DatabaseName<'_>) |  |  |  |  |

| `push_space` | function | push_space(&mut self) |  |  |  |  |

| `push_string_literal` | function | push_string_literal(&mut self, s: &str) |  |  |  |  |

| `push_value` | function | push_value(&mut self, value: &dyn ToSql) |  |  |  |  |

| `Sql` | struct |  |  |  |  |  |


### crates/frozen-duckdb/src/duckdb/r2d2.rs

| `memory` | function | memory() |  |  |  |  |

| `memory_with_flags` | function | memory_with_flags(config: Config) |  |  |  |  |

| `DuckdbConnectionManager` | struct |  |  |  |  |  |


### crates/frozen-duckdb/src/duckdb/raw_statement.rs

| `bind_parameter_count` | function | bind_parameter_count(&self) |  |  |  |  |

| `clear_bindings` | function | clear_bindings(&self) |  |  |  |  |

| `column_count` | function | column_count(&self) |  |  |  |  |

| `column_name` | function | column_name(&self, idx: usize) |  |  |  |  |

| `column_type` | function | column_type(&self, idx: usize) |  |  |  |  |

| `execute` | function | execute(&mut self) |  |  |  |  |

| `execute_streaming` | function | execute_streaming(&mut self) |  |  |  |  |

| `is_null` | function | is_null(&self) |  |  |  |  |

| `new` | function | new(stmt: ffi::duckdb_prepared_statement) |  |  |  |  |

| `ptr` | function | ptr(&self) |  |  |  |  |

| `reset_result` | function | reset_result(&mut self) |  |  |  |  |

| `result_unwrap` | function | result_unwrap(&self) |  |  |  |  |

| `row_count` | function | row_count(&self) |  |  |  |  |

| `schema` | function | schema(&self) |  |  |  |  |

| `sql` | function | sql(&self) |  |  |  |  |

| `step` | function | step(&self) |  |  |  |  |

| `step2` | function | step2(&self) |  |  |  |  |

| `streaming_step` | function | streaming_step(&self, schema: SchemaRef) |  |  |  |  |

| `RawStatement` | struct |  |  |  |  |  |


### crates/frozen-duckdb/src/duckdb/row.rs

| `as_ref` | function | as_ref(&self) |  |  |  |  |

| `next` | function | next(&mut self) |  |  |  |  |

| `AndThenRows` | struct |  |  |  |  |  |

| `Map` | struct |  |  |  |  |  |

| `MappedRows` | struct |  |  |  |  |  |

| `Row` | struct |  |  |  |  |  |

| `Rows` | struct |  |  |  |  |  |

| `RowIndex` | trait |  |  |  |  |  |

| `Sealed` | trait |  |  |  |  |  |


### crates/frozen-duckdb/src/duckdb/statement.rs

| `parameter_count` | function | parameter_count(&self) |  |  |  |  |

| `raw_execute` | function | raw_execute(&mut self) |  |  |  |  |

| `raw_query` | function | raw_query(&self) |  |  |  |  |

| `row_count` | function | row_count(&self) |  |  |  |  |

| `schema` | function | schema(&self) |  |  |  |  |

| `step` | function | step(&self) |  |  |  |  |

| `step2` | function | step2(&self) |  |  |  |  |

| `stream_step` | function | stream_step(&self, schema: SchemaRef) |  |  |  |  |

| `Statement` | struct |  |  |  |  |  |


### crates/frozen-duckdb/src/duckdb/transaction.rs

| `DropBehavior` | enum |  |  |  |  |  |

| `commit` | function | commit(mut self) |  |  |  |  |

| `drop_behavior` | function | drop_behavior(&self) |  |  |  |  |

| `finish` | function | finish(mut self) |  |  |  |  |

| `new` | function | new(conn: &mut Connection) |  |  |  |  |

| `new_unchecked` | function | new_unchecked(conn: &Connection) |  |  |  |  |

| `rollback` | function | rollback(mut self) |  |  |  |  |

| `set_drop_behavior` | function | set_drop_behavior(&mut self, drop_behavior: DropBehavior) |  |  |  |  |

| `transaction` | function | transaction(&mut self) |  |  |  |  |

| `unchecked_transaction` | function | unchecked_transaction(&self) |  |  |  |  |

| `Transaction` | struct |  |  |  |  |  |


### crates/frozen-duckdb/src/duckdb/types/from_sql.rs

| `FromSqlError` | enum |  |  |  |  |  |

| `FromSql` | trait |  |  |  |  |  |


### crates/frozen-duckdb/src/duckdb/types/mod.rs

| `Type` | enum |  |  |  |  |  |

| `Null` | struct |  |  |  |  |  |


### crates/frozen-duckdb/src/duckdb/types/ordered_map.rs

| `get` | function | get(&self, key: &K) |  |  |  |  |

| `iter` | function | iter(&self) |  |  |  |  |

| `keys` | function | keys(&self) |  |  |  |  |

| `values` | function | values(&self) |  |  |  |  |

| `OrderedMap` | struct |  |  |  |  |  |


### crates/frozen-duckdb/src/duckdb/types/string.rs

| `as_bytes` | function | as_bytes(&mut self) |  |  |  |  |

| `as_str` | function | as_str(&mut self) |  |  |  |  |

| `new` | function | new(ptr: &'a mut duckdb_string_t) |  |  |  |  |

| `DuckString` | struct |  |  |  |  |  |


### crates/frozen-duckdb/src/duckdb/types/to_sql.rs

| `ToSqlOutput` | enum |  |  |  |  |  |

| `ToSql` | trait |  |  |  |  |  |


### crates/frozen-duckdb/src/duckdb/types/value.rs

| `Value` | enum |  |  |  |  |  |

| `data_type` | function | data_type(&self) |  |  |  |  |


### crates/frozen-duckdb/src/duckdb/types/value_ref.rs

| `EnumType` | enum |  |  |  |  |  |

| `ListType` | enum |  |  |  |  |  |

| `TimeUnit` | enum |  |  |  |  |  |

| `ValueRef` | enum |  |  |  |  |  |

| `as_blob` | function | as_blob(&self) |  |  |  |  |

| `as_str` | function | as_str(&self) |  |  |  |  |

| `data_type` | function | data_type(&self) |  |  |  |  |

| `to_micros` | function | to_micros(&self, value: i64) |  |  |  |  |

| `to_owned` | function | to_owned(&self) |  |  |  |  |


### crates/frozen-duckdb/src/duckdb/vscalar/arrow.rs

| `ArrowScalarParams` | enum |  |  |  |  |  |

| `exact` | function | exact(params: Vec<DataType>, return_type: DataType) |  |  |  |  |

| `variadic` | function | variadic(param: DataType, return_type: DataType) |  |  |  |  |

| `ArrowFunctionSignature` | struct |  |  |  |  |  |

| `VArrowScalar` | trait |  |  |  |  |  |


### crates/frozen-duckdb/src/duckdb/vscalar/function.rs

| `add_function` | function | add_function(&self, func: ScalarFunction) |  |  |  |  |

| `add_parameter` | function | add_parameter(&self, logical_type: &LogicalTypeHandle) |  |  |  |  |

| `add_variadic_parameter` | function | add_variadic_parameter(&self, logical_type: &LogicalTypeHandle) |  |  |  |  |

| `new` | function | new(name: &str) |  |  |  |  |

| `set_function` | function | set_function(
        &self,
        func: Option<
            unsafe extern "C" fn(
                info: duckdb_function_info,
                input: duckdb_data_chunk,
                output: duckdb_vector,
            ) |  |  |  |  |

| `set_return_type` | function | set_return_type(&self, logical_type: &LogicalTypeHandle) |  |  |  |  |

| `ScalarFunction` | struct |  |  |  |  |  |

| `ScalarFunctionSet` | struct |  |  |  |  |  |


### crates/frozen-duckdb/src/duckdb/vscalar/mod.rs

| `ScalarParams` | enum |  |  |  |  |  |

| `exact` | function | exact(params: Vec<LogicalTypeHandle>, return_type: LogicalTypeHandle) |  |  |  |  |

| `register_scalar_function_set` | function | register_scalar_function_set(
        &mut self,
        f: ScalarFunctionSet,
    ) |  |  |  |  |

| `set_error` | function | set_error(&self, error: &str) |  |  |  |  |

| `variadic` | function | variadic(param: LogicalTypeHandle, return_type: LogicalTypeHandle) |  |  |  |  |

| `ScalarFunctionSignature` | struct |  |  |  |  |  |

| `VScalar` | trait |  |  |  |  |  |


### crates/frozen-duckdb/src/duckdb/vtab/arrow.rs

| `arrow_arraydata_to_query_params` | function | arrow_arraydata_to_query_params(data: ArrayData) |  |  |  |  |

| `arrow_ffi_to_query_params` | function | arrow_ffi_to_query_params(array: FFI_ArrowArray, schema: FFI_ArrowSchema) |  |  |  |  |

| `arrow_recordbatch_to_query_params` | function | arrow_recordbatch_to_query_params(rb: RecordBatch) |  |  |  |  |

| `data_chunk_to_arrow` | function | data_chunk_to_arrow(
    chunk: &DataChunkHandle,
) |  |  |  |  |

| `flat_vector_to_arrow_array` | function | flat_vector_to_arrow_array(
    vector: &mut FlatVector,
    len: usize,
) |  |  |  |  |

| `record_batch_to_duckdb_data_chunk` | function | record_batch_to_duckdb_data_chunk(
    batch: &RecordBatch,
    chunk: &mut DataChunkHandle,
) |  |  |  |  |

| `to_duckdb_logical_type` | function | to_duckdb_logical_type(
    data_type: &DataType,
) |  |  |  |  |

| `to_duckdb_type_id` | function | to_duckdb_type_id(
    data_type: &DataType,
) |  |  |  |  |

| `write_arrow_array_to_vector` | function | write_arrow_array_to_vector(
    col: &Arc<dyn Array>,
    chunk: &mut dyn WritableVector,
) |  |  |  |  |

| `ArrowBindData` | struct |  |  |  |  |  |

| `ArrowInitData` | struct |  |  |  |  |  |

| `ArrowVTab` | struct |  |  |  |  |  |

| `WritableVector` | trait |  |  |  |  |  |


### crates/frozen-duckdb/src/duckdb/vtab/function.rs

| `add_named_parameter` | function | add_named_parameter(&self, name: &str, logical_type: &LogicalTypeHandle) |  |  |  |  |

| `add_parameter` | function | add_parameter(&self, logical_type: &LogicalTypeHandle) |  |  |  |  |

| `add_result_column` | function | add_result_column(&self, column_name: &str, column_type: LogicalTypeHandle) |  |  |  |  |

| `get_bind_data` | function | get_bind_data(&self) |  |  |  |  |

| `get_column_indices` | function | get_column_indices(&self) |  |  |  |  |

| `get_init_data` | function | get_init_data(&self) |  |  |  |  |

| `get_named_parameter` | function | get_named_parameter(&self, name: &str) |  |  |  |  |

| `get_parameter` | function | get_parameter(&self, param_index: u64) |  |  |  |  |

| `get_parameter_count` | function | get_parameter_count(&self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `set_bind` | function | set_bind(&self, bind_func: Option<unsafe extern "C" fn(duckdb_bind_info) |  |  |  |  |

| `set_bind_data` | function | set_bind_data(
        &self,
        data: *mut c_void,
        free_function: Option<unsafe extern "C" fn(*mut c_void) |  |  |  |  |

| `set_cardinality` | function | set_cardinality(&self, cardinality: idx_t, is_exact: bool) |  |  |  |  |

| `set_error` | function | set_error(&self, error: &str) |  |  |  |  |

| `set_extra_info` | function | set_extra_info(
        &self,
        extra_info: *mut c_void,
        destroy: duckdb_delete_callback_t,
    ) |  |  |  |  |

| `set_function` | function | set_function(
        &self,
        func: Option<unsafe extern "C" fn(info: duckdb_function_info, output: duckdb_data_chunk) |  |  |  |  |

| `set_init` | function | set_init(&self, init_func: Option<unsafe extern "C" fn(duckdb_init_info) |  |  |  |  |

| `set_init_data` | function | set_init_data(
        &self,
        data: *mut c_void,
        freeer: Option<unsafe extern "C" fn(*mut c_void) |  |  |  |  |

| `set_local_init` | function | set_local_init(&self, init: duckdb_table_function_init_t) |  |  |  |  |

| `set_max_threads` | function | set_max_threads(&self, max_threads: idx_t) |  |  |  |  |

| `set_name` | function | set_name(&self, name: &str) |  |  |  |  |

| `supports_pushdown` | function | supports_pushdown(&self, supports: bool) |  |  |  |  |

| `BindInfo` | struct |  |  |  |  |  |

| `InitInfo` | struct |  |  |  |  |  |

| `TableFunction` | struct |  |  |  |  |  |

| `TableFunctionInfo` | struct |  |  |  |  |  |


### crates/frozen-duckdb/src/duckdb/vtab/mod.rs

| `register_table_function` | function | register_table_function(&mut self, table_function: TableFunction) |  |  |  |  |

| `VTab` | trait |  |  |  |  |  |


### crates/frozen-duckdb/src/duckdb/vtab/value.rs

| `to_int64` | function | to_int64(&self) |  |  |  |  |

| `Value` | struct |  |  |  |  |  |


### crates/frozen-duckdb/src/env_setup.rs

| `get_include_dir` | function | get_include_dir() |  |  |  |  |

| `get_lib_dir` | function | get_lib_dir() |  |  |  |  |

| `is_configured` | function | is_configured() |  |  |  |  |

| `validate_binary` | function | validate_binary() |  |  |  |  |



<!-- AGENT-FORBIDDEN-END -->

## Signature/type/default/errors table

<!-- RIGID table: header order is fixed; rows come only from the query. -->

| Item | Type | Signature | Params | Defaults | Errors | Invariants |
|------|------|-----------|--------|----------|--------|------------|

| `ensure_binary` | function | ensure_binary() |  |  |  |  |

| `main` | function | main(_out_dir: &str, out_path: &Path, lib_dir: &Path) |  |  |  |  |

| `write_to_out_dir` | function | write_to_out_dir(header: HeaderLocation, out_path: &Path) |  |  |  |  |

| `duckdb_add_aggregate_function_to_set` | function | duckdb_add_aggregate_function_to_set(
        set: duckdb_aggregate_function_set,
        function: duckdb_aggregate_function,
    ) |  |  |  |  |

| `duckdb_add_replacement_scan` | function | duckdb_add_replacement_scan(
        db: duckdb_database,
        replacement: duckdb_replacement_callback_t,
        extra_data: *mut ::std::os::raw::c_void,
        delete_callback: duckdb_delete_callback_t,
    ) |  |  |  |  |

| `duckdb_add_scalar_function_to_set` | function | duckdb_add_scalar_function_to_set(
        set: duckdb_scalar_function_set,
        function: duckdb_scalar_function,
    ) |  |  |  |  |

| `duckdb_aggregate_function_add_parameter` | function | duckdb_aggregate_function_add_parameter(
        aggregate_function: duckdb_aggregate_function,
        type_: duckdb_logical_type,
    ) |  |  |  |  |

| `duckdb_aggregate_function_get_extra_info` | function | duckdb_aggregate_function_get_extra_info(info: duckdb_function_info) |  |  |  |  |

| `duckdb_aggregate_function_set_destructor` | function | duckdb_aggregate_function_set_destructor(
        aggregate_function: duckdb_aggregate_function,
        destroy: duckdb_aggregate_destroy_t,
    ) |  |  |  |  |

| `duckdb_aggregate_function_set_error` | function | duckdb_aggregate_function_set_error(info: duckdb_function_info, error: *const ::std::os::raw::c_char) |  |  |  |  |

| `duckdb_aggregate_function_set_extra_info` | function | duckdb_aggregate_function_set_extra_info(
        aggregate_function: duckdb_aggregate_function,
        extra_info: *mut ::std::os::raw::c_void,
        destroy: duckdb_delete_callback_t,
    ) |  |  |  |  |

| `duckdb_aggregate_function_set_functions` | function | duckdb_aggregate_function_set_functions(
        aggregate_function: duckdb_aggregate_function,
        state_size: duckdb_aggregate_state_size,
        state_init: duckdb_aggregate_init_t,
        update: duckdb_aggregate_update_t,
        combine: duckdb_aggregate_combine_t,
        finalize: duckdb_aggregate_finalize_t,
    ) |  |  |  |  |

| `duckdb_aggregate_function_set_name` | function | duckdb_aggregate_function_set_name(
        aggregate_function: duckdb_aggregate_function,
        name: *const ::std::os::raw::c_char,
    ) |  |  |  |  |

| `duckdb_aggregate_function_set_return_type` | function | duckdb_aggregate_function_set_return_type(
        aggregate_function: duckdb_aggregate_function,
        type_: duckdb_logical_type,
    ) |  |  |  |  |

| `duckdb_aggregate_function_set_special_handling` | function | duckdb_aggregate_function_set_special_handling(aggregate_function: duckdb_aggregate_function) |  |  |  |  |

| `duckdb_append_blob` | function | duckdb_append_blob(
        appender: duckdb_appender,
        data: *const ::std::os::raw::c_void,
        length: idx_t,
    ) |  |  |  |  |

| `duckdb_append_bool` | function | duckdb_append_bool(appender: duckdb_appender, value: bool) |  |  |  |  |

| `duckdb_append_data_chunk` | function | duckdb_append_data_chunk(appender: duckdb_appender, chunk: duckdb_data_chunk) |  |  |  |  |

| `duckdb_append_date` | function | duckdb_append_date(appender: duckdb_appender, value: duckdb_date) |  |  |  |  |

| `duckdb_append_default` | function | duckdb_append_default(appender: duckdb_appender) |  |  |  |  |

| `duckdb_append_default_to_chunk` | function | duckdb_append_default_to_chunk(
        appender: duckdb_appender,
        chunk: duckdb_data_chunk,
        col: idx_t,
        row: idx_t,
    ) |  |  |  |  |

| `duckdb_append_double` | function | duckdb_append_double(appender: duckdb_appender, value: f64) |  |  |  |  |

| `duckdb_append_float` | function | duckdb_append_float(appender: duckdb_appender, value: f32) |  |  |  |  |

| `duckdb_append_hugeint` | function | duckdb_append_hugeint(appender: duckdb_appender, value: duckdb_hugeint) |  |  |  |  |

| `duckdb_append_int16` | function | duckdb_append_int16(appender: duckdb_appender, value: i16) |  |  |  |  |

| `duckdb_append_int32` | function | duckdb_append_int32(appender: duckdb_appender, value: i32) |  |  |  |  |

| `duckdb_append_int64` | function | duckdb_append_int64(appender: duckdb_appender, value: i64) |  |  |  |  |

| `duckdb_append_int8` | function | duckdb_append_int8(appender: duckdb_appender, value: i8) |  |  |  |  |

| `duckdb_append_interval` | function | duckdb_append_interval(appender: duckdb_appender, value: duckdb_interval) |  |  |  |  |

| `duckdb_append_null` | function | duckdb_append_null(appender: duckdb_appender) |  |  |  |  |

| `duckdb_append_time` | function | duckdb_append_time(appender: duckdb_appender, value: duckdb_time) |  |  |  |  |

| `duckdb_append_timestamp` | function | duckdb_append_timestamp(appender: duckdb_appender, value: duckdb_timestamp) |  |  |  |  |

| `duckdb_append_uhugeint` | function | duckdb_append_uhugeint(appender: duckdb_appender, value: duckdb_uhugeint) |  |  |  |  |

| `duckdb_append_uint16` | function | duckdb_append_uint16(appender: duckdb_appender, value: u16) |  |  |  |  |

| `duckdb_append_uint32` | function | duckdb_append_uint32(appender: duckdb_appender, value: u32) |  |  |  |  |

| `duckdb_append_uint64` | function | duckdb_append_uint64(appender: duckdb_appender, value: u64) |  |  |  |  |

| `duckdb_append_uint8` | function | duckdb_append_uint8(appender: duckdb_appender, value: u8) |  |  |  |  |

| `duckdb_append_value` | function | duckdb_append_value(appender: duckdb_appender, value: duckdb_value) |  |  |  |  |

| `duckdb_append_varchar` | function | duckdb_append_varchar(appender: duckdb_appender, val: *const ::std::os::raw::c_char) |  |  |  |  |

| `duckdb_append_varchar_length` | function | duckdb_append_varchar_length(
        appender: duckdb_appender,
        val: *const ::std::os::raw::c_char,
        length: idx_t,
    ) |  |  |  |  |

| `duckdb_appender_add_column` | function | duckdb_appender_add_column(appender: duckdb_appender, name: *const ::std::os::raw::c_char) |  |  |  |  |

| `duckdb_appender_begin_row` | function | duckdb_appender_begin_row(appender: duckdb_appender) |  |  |  |  |

| `duckdb_appender_clear_columns` | function | duckdb_appender_clear_columns(appender: duckdb_appender) |  |  |  |  |

| `duckdb_appender_close` | function | duckdb_appender_close(appender: duckdb_appender) |  |  |  |  |

| `duckdb_appender_column_count` | function | duckdb_appender_column_count(appender: duckdb_appender) |  |  |  |  |

| `duckdb_appender_column_type` | function | duckdb_appender_column_type(appender: duckdb_appender, col_idx: idx_t) |  |  |  |  |

| `duckdb_appender_create` | function | duckdb_appender_create(
        connection: duckdb_connection,
        schema: *const ::std::os::raw::c_char,
        table: *const ::std::os::raw::c_char,
        out_appender: *mut duckdb_appender,
    ) |  |  |  |  |

| `duckdb_appender_create_ext` | function | duckdb_appender_create_ext(
        connection: duckdb_connection,
        catalog: *const ::std::os::raw::c_char,
        schema: *const ::std::os::raw::c_char,
        table: *const ::std::os::raw::c_char,
        out_appender: *mut duckdb_appender,
    ) |  |  |  |  |

| `duckdb_appender_create_query` | function | duckdb_appender_create_query(
        connection: duckdb_connection,
        query: *const ::std::os::raw::c_char,
        column_count: idx_t,
        types: *mut duckdb_logical_type,
        table_name: *const ::std::os::raw::c_char,
        column_names: *mut *const ::std::os::raw::c_char,
        out_appender: *mut duckdb_appender,
    ) |  |  |  |  |

| `duckdb_appender_destroy` | function | duckdb_appender_destroy(appender: *mut duckdb_appender) |  |  |  |  |

| `duckdb_appender_end_row` | function | duckdb_appender_end_row(appender: duckdb_appender) |  |  |  |  |

| `duckdb_appender_error` | function | duckdb_appender_error(appender: duckdb_appender) |  |  |  |  |

| `duckdb_appender_error_data` | function | duckdb_appender_error_data(appender: duckdb_appender) |  |  |  |  |

| `duckdb_appender_flush` | function | duckdb_appender_flush(appender: duckdb_appender) |  |  |  |  |

| `duckdb_array_type_array_size` | function | duckdb_array_type_array_size(type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_array_type_child_type` | function | duckdb_array_type_child_type(type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_array_vector_get_child` | function | duckdb_array_vector_get_child(vector: duckdb_vector) |  |  |  |  |

| `duckdb_arrow_array_scan` | function | duckdb_arrow_array_scan(
        connection: duckdb_connection,
        table_name: *const ::std::os::raw::c_char,
        arrow_schema: duckdb_arrow_schema,
        arrow_array: duckdb_arrow_array,
        out_stream: *mut duckdb_arrow_stream,
    ) |  |  |  |  |

| `duckdb_arrow_column_count` | function | duckdb_arrow_column_count(result: duckdb_arrow) |  |  |  |  |

| `duckdb_arrow_row_count` | function | duckdb_arrow_row_count(result: duckdb_arrow) |  |  |  |  |

| `duckdb_arrow_rows_changed` | function | duckdb_arrow_rows_changed(result: duckdb_arrow) |  |  |  |  |

| `duckdb_arrow_scan` | function | duckdb_arrow_scan(
        connection: duckdb_connection,
        table_name: *const ::std::os::raw::c_char,
        arrow: duckdb_arrow_stream,
    ) |  |  |  |  |

| `duckdb_bind_add_result_column` | function | duckdb_bind_add_result_column(
        info: duckdb_bind_info,
        name: *const ::std::os::raw::c_char,
        type_: duckdb_logical_type,
    ) |  |  |  |  |

| `duckdb_bind_blob` | function | duckdb_bind_blob(
        prepared_statement: duckdb_prepared_statement,
        param_idx: idx_t,
        data: *const ::std::os::raw::c_void,
        length: idx_t,
    ) |  |  |  |  |

| `duckdb_bind_boolean` | function | duckdb_bind_boolean(
        prepared_statement: duckdb_prepared_statement,
        param_idx: idx_t,
        val: bool,
    ) |  |  |  |  |

| `duckdb_bind_date` | function | duckdb_bind_date(
        prepared_statement: duckdb_prepared_statement,
        param_idx: idx_t,
        val: duckdb_date,
    ) |  |  |  |  |

| `duckdb_bind_decimal` | function | duckdb_bind_decimal(
        prepared_statement: duckdb_prepared_statement,
        param_idx: idx_t,
        val: duckdb_decimal,
    ) |  |  |  |  |

| `duckdb_bind_double` | function | duckdb_bind_double(
        prepared_statement: duckdb_prepared_statement,
        param_idx: idx_t,
        val: f64,
    ) |  |  |  |  |

| `duckdb_bind_float` | function | duckdb_bind_float(prepared_statement: duckdb_prepared_statement, param_idx: idx_t, val: f32) |  |  |  |  |

| `duckdb_bind_get_extra_info` | function | duckdb_bind_get_extra_info(info: duckdb_bind_info) |  |  |  |  |

| `duckdb_bind_get_named_parameter` | function | duckdb_bind_get_named_parameter(info: duckdb_bind_info, name: *const ::std::os::raw::c_char) |  |  |  |  |

| `duckdb_bind_get_parameter` | function | duckdb_bind_get_parameter(info: duckdb_bind_info, index: idx_t) |  |  |  |  |

| `duckdb_bind_get_parameter_count` | function | duckdb_bind_get_parameter_count(info: duckdb_bind_info) |  |  |  |  |

| `duckdb_bind_hugeint` | function | duckdb_bind_hugeint(
        prepared_statement: duckdb_prepared_statement,
        param_idx: idx_t,
        val: duckdb_hugeint,
    ) |  |  |  |  |

| `duckdb_bind_int16` | function | duckdb_bind_int16(prepared_statement: duckdb_prepared_statement, param_idx: idx_t, val: i16) |  |  |  |  |

| `duckdb_bind_int32` | function | duckdb_bind_int32(prepared_statement: duckdb_prepared_statement, param_idx: idx_t, val: i32) |  |  |  |  |

| `duckdb_bind_int64` | function | duckdb_bind_int64(prepared_statement: duckdb_prepared_statement, param_idx: idx_t, val: i64) |  |  |  |  |

| `duckdb_bind_int8` | function | duckdb_bind_int8(prepared_statement: duckdb_prepared_statement, param_idx: idx_t, val: i8) |  |  |  |  |

| `duckdb_bind_interval` | function | duckdb_bind_interval(
        prepared_statement: duckdb_prepared_statement,
        param_idx: idx_t,
        val: duckdb_interval,
    ) |  |  |  |  |

| `duckdb_bind_null` | function | duckdb_bind_null(prepared_statement: duckdb_prepared_statement, param_idx: idx_t) |  |  |  |  |

| `duckdb_bind_parameter_index` | function | duckdb_bind_parameter_index(
        prepared_statement: duckdb_prepared_statement,
        param_idx_out: *mut idx_t,
        name: *const ::std::os::raw::c_char,
    ) |  |  |  |  |

| `duckdb_bind_set_bind_data` | function | duckdb_bind_set_bind_data(
        info: duckdb_bind_info,
        bind_data: *mut ::std::os::raw::c_void,
        destroy: duckdb_delete_callback_t,
    ) |  |  |  |  |

| `duckdb_bind_set_cardinality` | function | duckdb_bind_set_cardinality(info: duckdb_bind_info, cardinality: idx_t, is_exact: bool) |  |  |  |  |

| `duckdb_bind_set_error` | function | duckdb_bind_set_error(info: duckdb_bind_info, error: *const ::std::os::raw::c_char) |  |  |  |  |

| `duckdb_bind_time` | function | duckdb_bind_time(
        prepared_statement: duckdb_prepared_statement,
        param_idx: idx_t,
        val: duckdb_time,
    ) |  |  |  |  |

| `duckdb_bind_timestamp` | function | duckdb_bind_timestamp(
        prepared_statement: duckdb_prepared_statement,
        param_idx: idx_t,
        val: duckdb_timestamp,
    ) |  |  |  |  |

| `duckdb_bind_timestamp_tz` | function | duckdb_bind_timestamp_tz(
        prepared_statement: duckdb_prepared_statement,
        param_idx: idx_t,
        val: duckdb_timestamp,
    ) |  |  |  |  |

| `duckdb_bind_uhugeint` | function | duckdb_bind_uhugeint(
        prepared_statement: duckdb_prepared_statement,
        param_idx: idx_t,
        val: duckdb_uhugeint,
    ) |  |  |  |  |

| `duckdb_bind_uint16` | function | duckdb_bind_uint16(
        prepared_statement: duckdb_prepared_statement,
        param_idx: idx_t,
        val: u16,
    ) |  |  |  |  |

| `duckdb_bind_uint32` | function | duckdb_bind_uint32(
        prepared_statement: duckdb_prepared_statement,
        param_idx: idx_t,
        val: u32,
    ) |  |  |  |  |

| `duckdb_bind_uint64` | function | duckdb_bind_uint64(
        prepared_statement: duckdb_prepared_statement,
        param_idx: idx_t,
        val: u64,
    ) |  |  |  |  |

| `duckdb_bind_uint8` | function | duckdb_bind_uint8(prepared_statement: duckdb_prepared_statement, param_idx: idx_t, val: u8) |  |  |  |  |

| `duckdb_bind_value` | function | duckdb_bind_value(
        prepared_statement: duckdb_prepared_statement,
        param_idx: idx_t,
        val: duckdb_value,
    ) |  |  |  |  |

| `duckdb_bind_varchar` | function | duckdb_bind_varchar(
        prepared_statement: duckdb_prepared_statement,
        param_idx: idx_t,
        val: *const ::std::os::raw::c_char,
    ) |  |  |  |  |

| `duckdb_bind_varchar_length` | function | duckdb_bind_varchar_length(
        prepared_statement: duckdb_prepared_statement,
        param_idx: idx_t,
        val: *const ::std::os::raw::c_char,
        length: idx_t,
    ) |  |  |  |  |

| `duckdb_cast_function_get_cast_mode` | function | duckdb_cast_function_get_cast_mode(info: duckdb_function_info) |  |  |  |  |

| `duckdb_cast_function_get_extra_info` | function | duckdb_cast_function_get_extra_info(info: duckdb_function_info) |  |  |  |  |

| `duckdb_cast_function_set_error` | function | duckdb_cast_function_set_error(info: duckdb_function_info, error: *const ::std::os::raw::c_char) |  |  |  |  |

| `duckdb_cast_function_set_extra_info` | function | duckdb_cast_function_set_extra_info(
        cast_function: duckdb_cast_function,
        extra_info: *mut ::std::os::raw::c_void,
        destroy: duckdb_delete_callback_t,
    ) |  |  |  |  |

| `duckdb_cast_function_set_function` | function | duckdb_cast_function_set_function(cast_function: duckdb_cast_function, function: duckdb_cast_function_t) |  |  |  |  |

| `duckdb_cast_function_set_implicit_cast_cost` | function | duckdb_cast_function_set_implicit_cast_cost(cast_function: duckdb_cast_function, cost: i64) |  |  |  |  |

| `duckdb_cast_function_set_row_error` | function | duckdb_cast_function_set_row_error(
        info: duckdb_function_info,
        error: *const ::std::os::raw::c_char,
        row: idx_t,
        output: duckdb_vector,
    ) |  |  |  |  |

| `duckdb_cast_function_set_source_type` | function | duckdb_cast_function_set_source_type(cast_function: duckdb_cast_function, source_type: duckdb_logical_type) |  |  |  |  |

| `duckdb_cast_function_set_target_type` | function | duckdb_cast_function_set_target_type(cast_function: duckdb_cast_function, target_type: duckdb_logical_type) |  |  |  |  |

| `duckdb_clear_bindings` | function | duckdb_clear_bindings(prepared_statement: duckdb_prepared_statement) |  |  |  |  |

| `duckdb_client_context_get_connection_id` | function | duckdb_client_context_get_connection_id(context: duckdb_client_context) |  |  |  |  |

| `duckdb_close` | function | duckdb_close(database: *mut duckdb_database) |  |  |  |  |

| `duckdb_column_count` | function | duckdb_column_count(result: *mut duckdb_result) |  |  |  |  |

| `duckdb_column_data` | function | duckdb_column_data(result: *mut duckdb_result, col: idx_t) |  |  |  |  |

| `duckdb_column_has_default` | function | duckdb_column_has_default(
        table_description: duckdb_table_description,
        index: idx_t,
        out: *mut bool,
    ) |  |  |  |  |

| `duckdb_column_logical_type` | function | duckdb_column_logical_type(result: *mut duckdb_result, col: idx_t) |  |  |  |  |

| `duckdb_column_name` | function | duckdb_column_name(result: *mut duckdb_result, col: idx_t) |  |  |  |  |

| `duckdb_column_type` | function | duckdb_column_type(result: *mut duckdb_result, col: idx_t) |  |  |  |  |

| `duckdb_config_count` | function | duckdb_config_count() |  |  |  |  |

| `duckdb_connect` | function | duckdb_connect(database: duckdb_database, out_connection: *mut duckdb_connection) |  |  |  |  |

| `duckdb_connection_get_arrow_options` | function | duckdb_connection_get_arrow_options(
        connection: duckdb_connection,
        out_arrow_options: *mut duckdb_arrow_options,
    ) |  |  |  |  |

| `duckdb_connection_get_client_context` | function | duckdb_connection_get_client_context(connection: duckdb_connection, out_context: *mut duckdb_client_context) |  |  |  |  |

| `duckdb_create_aggregate_function` | function | duckdb_create_aggregate_function() |  |  |  |  |

| `duckdb_create_aggregate_function_set` | function | duckdb_create_aggregate_function_set(name: *const ::std::os::raw::c_char) |  |  |  |  |

| `duckdb_create_array_type` | function | duckdb_create_array_type(type_: duckdb_logical_type, array_size: idx_t) |  |  |  |  |

| `duckdb_create_array_value` | function | duckdb_create_array_value(
        type_: duckdb_logical_type,
        values: *mut duckdb_value,
        value_count: idx_t,
    ) |  |  |  |  |

| `duckdb_create_bignum` | function | duckdb_create_bignum(input: duckdb_bignum) |  |  |  |  |

| `duckdb_create_bit` | function | duckdb_create_bit(input: duckdb_bit) |  |  |  |  |

| `duckdb_create_blob` | function | duckdb_create_blob(data: *const u8, length: idx_t) |  |  |  |  |

| `duckdb_create_bool` | function | duckdb_create_bool(input: bool) |  |  |  |  |

| `duckdb_create_cast_function` | function | duckdb_create_cast_function() |  |  |  |  |

| `duckdb_create_config` | function | duckdb_create_config(out_config: *mut duckdb_config) |  |  |  |  |

| `duckdb_create_data_chunk` | function | duckdb_create_data_chunk(types: *mut duckdb_logical_type, column_count: idx_t) |  |  |  |  |

| `duckdb_create_date` | function | duckdb_create_date(input: duckdb_date) |  |  |  |  |

| `duckdb_create_decimal` | function | duckdb_create_decimal(input: duckdb_decimal) |  |  |  |  |

| `duckdb_create_decimal_type` | function | duckdb_create_decimal_type(width: u8, scale: u8) |  |  |  |  |

| `duckdb_create_double` | function | duckdb_create_double(input: f64) |  |  |  |  |

| `duckdb_create_enum_type` | function | duckdb_create_enum_type(
        member_names: *mut *const ::std::os::raw::c_char,
        member_count: idx_t,
    ) |  |  |  |  |

| `duckdb_create_enum_value` | function | duckdb_create_enum_value(type_: duckdb_logical_type, value: u64) |  |  |  |  |

| `duckdb_create_error_data` | function | duckdb_create_error_data(
        type_: duckdb_error_type,
        message: *const ::std::os::raw::c_char,
    ) |  |  |  |  |

| `duckdb_create_float` | function | duckdb_create_float(input: f32) |  |  |  |  |

| `duckdb_create_hugeint` | function | duckdb_create_hugeint(input: duckdb_hugeint) |  |  |  |  |

| `duckdb_create_instance_cache` | function | duckdb_create_instance_cache() |  |  |  |  |

| `duckdb_create_int16` | function | duckdb_create_int16(input: i16) |  |  |  |  |

| `duckdb_create_int32` | function | duckdb_create_int32(input: i32) |  |  |  |  |

| `duckdb_create_int64` | function | duckdb_create_int64(val: i64) |  |  |  |  |

| `duckdb_create_int8` | function | duckdb_create_int8(input: i8) |  |  |  |  |

| `duckdb_create_interval` | function | duckdb_create_interval(input: duckdb_interval) |  |  |  |  |

| `duckdb_create_list_type` | function | duckdb_create_list_type(type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_create_list_value` | function | duckdb_create_list_value(
        type_: duckdb_logical_type,
        values: *mut duckdb_value,
        value_count: idx_t,
    ) |  |  |  |  |

| `duckdb_create_logical_type` | function | duckdb_create_logical_type(type_: duckdb_type) |  |  |  |  |

| `duckdb_create_map_type` | function | duckdb_create_map_type(
        key_type: duckdb_logical_type,
        value_type: duckdb_logical_type,
    ) |  |  |  |  |

| `duckdb_create_map_value` | function | duckdb_create_map_value(
        map_type: duckdb_logical_type,
        keys: *mut duckdb_value,
        values: *mut duckdb_value,
        entry_count: idx_t,
    ) |  |  |  |  |

| `duckdb_create_null_value` | function | duckdb_create_null_value() |  |  |  |  |

| `duckdb_create_scalar_function` | function | duckdb_create_scalar_function() |  |  |  |  |

| `duckdb_create_scalar_function_set` | function | duckdb_create_scalar_function_set(name: *const ::std::os::raw::c_char) |  |  |  |  |

| `duckdb_create_selection_vector` | function | duckdb_create_selection_vector(size: idx_t) |  |  |  |  |

| `duckdb_create_struct_type` | function | duckdb_create_struct_type(
        member_types: *mut duckdb_logical_type,
        member_names: *mut *const ::std::os::raw::c_char,
        member_count: idx_t,
    ) |  |  |  |  |

| `duckdb_create_struct_value` | function | duckdb_create_struct_value(type_: duckdb_logical_type, values: *mut duckdb_value) |  |  |  |  |

| `duckdb_create_table_function` | function | duckdb_create_table_function() |  |  |  |  |

| `duckdb_create_task_state` | function | duckdb_create_task_state(database: duckdb_database) |  |  |  |  |

| `duckdb_create_time` | function | duckdb_create_time(input: duckdb_time) |  |  |  |  |

| `duckdb_create_time_ns` | function | duckdb_create_time_ns(input: duckdb_time_ns) |  |  |  |  |

| `duckdb_create_time_tz` | function | duckdb_create_time_tz(micros: i64, offset: i32) |  |  |  |  |

| `duckdb_create_time_tz_value` | function | duckdb_create_time_tz_value(value: duckdb_time_tz) |  |  |  |  |

| `duckdb_create_timestamp` | function | duckdb_create_timestamp(input: duckdb_timestamp) |  |  |  |  |

| `duckdb_create_timestamp_ms` | function | duckdb_create_timestamp_ms(input: duckdb_timestamp_ms) |  |  |  |  |

| `duckdb_create_timestamp_ns` | function | duckdb_create_timestamp_ns(input: duckdb_timestamp_ns) |  |  |  |  |

| `duckdb_create_timestamp_s` | function | duckdb_create_timestamp_s(input: duckdb_timestamp_s) |  |  |  |  |

| `duckdb_create_timestamp_tz` | function | duckdb_create_timestamp_tz(input: duckdb_timestamp) |  |  |  |  |

| `duckdb_create_uhugeint` | function | duckdb_create_uhugeint(input: duckdb_uhugeint) |  |  |  |  |

| `duckdb_create_uint16` | function | duckdb_create_uint16(input: u16) |  |  |  |  |

| `duckdb_create_uint32` | function | duckdb_create_uint32(input: u32) |  |  |  |  |

| `duckdb_create_uint64` | function | duckdb_create_uint64(input: u64) |  |  |  |  |

| `duckdb_create_uint8` | function | duckdb_create_uint8(input: u8) |  |  |  |  |

| `duckdb_create_union_type` | function | duckdb_create_union_type(
        member_types: *mut duckdb_logical_type,
        member_names: *mut *const ::std::os::raw::c_char,
        member_count: idx_t,
    ) |  |  |  |  |

| `duckdb_create_union_value` | function | duckdb_create_union_value(
        union_type: duckdb_logical_type,
        tag_index: idx_t,
        value: duckdb_value,
    ) |  |  |  |  |

| `duckdb_create_uuid` | function | duckdb_create_uuid(input: duckdb_uhugeint) |  |  |  |  |

| `duckdb_create_varchar` | function | duckdb_create_varchar(text: *const ::std::os::raw::c_char) |  |  |  |  |

| `duckdb_create_varchar_length` | function | duckdb_create_varchar_length(text: *const ::std::os::raw::c_char, length: idx_t) |  |  |  |  |

| `duckdb_create_vector` | function | duckdb_create_vector(type_: duckdb_logical_type, capacity: idx_t) |  |  |  |  |

| `duckdb_data_chunk_from_arrow` | function | duckdb_data_chunk_from_arrow(
        connection: duckdb_connection,
        arrow_array: *mut ArrowArray,
        converted_schema: duckdb_arrow_converted_schema,
        out_chunk: *mut duckdb_data_chunk,
    ) |  |  |  |  |

| `duckdb_data_chunk_get_column_count` | function | duckdb_data_chunk_get_column_count(chunk: duckdb_data_chunk) |  |  |  |  |

| `duckdb_data_chunk_get_size` | function | duckdb_data_chunk_get_size(chunk: duckdb_data_chunk) |  |  |  |  |

| `duckdb_data_chunk_get_vector` | function | duckdb_data_chunk_get_vector(chunk: duckdb_data_chunk, col_idx: idx_t) |  |  |  |  |

| `duckdb_data_chunk_reset` | function | duckdb_data_chunk_reset(chunk: duckdb_data_chunk) |  |  |  |  |

| `duckdb_data_chunk_set_size` | function | duckdb_data_chunk_set_size(chunk: duckdb_data_chunk, size: idx_t) |  |  |  |  |

| `duckdb_data_chunk_to_arrow` | function | duckdb_data_chunk_to_arrow(
        arrow_options: duckdb_arrow_options,
        chunk: duckdb_data_chunk,
        out_arrow_array: *mut ArrowArray,
    ) |  |  |  |  |

| `duckdb_decimal_internal_type` | function | duckdb_decimal_internal_type(type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_decimal_scale` | function | duckdb_decimal_scale(type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_decimal_to_double` | function | duckdb_decimal_to_double(val: duckdb_decimal) |  |  |  |  |

| `duckdb_decimal_width` | function | duckdb_decimal_width(type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_destroy_aggregate_function` | function | duckdb_destroy_aggregate_function(aggregate_function: *mut duckdb_aggregate_function) |  |  |  |  |

| `duckdb_destroy_aggregate_function_set` | function | duckdb_destroy_aggregate_function_set(aggregate_function_set: *mut duckdb_aggregate_function_set) |  |  |  |  |

| `duckdb_destroy_arrow` | function | duckdb_destroy_arrow(result: *mut duckdb_arrow) |  |  |  |  |

| `duckdb_destroy_arrow_converted_schema` | function | duckdb_destroy_arrow_converted_schema(arrow_converted_schema: *mut duckdb_arrow_converted_schema) |  |  |  |  |

| `duckdb_destroy_arrow_options` | function | duckdb_destroy_arrow_options(arrow_options: *mut duckdb_arrow_options) |  |  |  |  |

| `duckdb_destroy_arrow_stream` | function | duckdb_destroy_arrow_stream(stream_p: *mut duckdb_arrow_stream) |  |  |  |  |

| `duckdb_destroy_cast_function` | function | duckdb_destroy_cast_function(cast_function: *mut duckdb_cast_function) |  |  |  |  |

| `duckdb_destroy_client_context` | function | duckdb_destroy_client_context(context: *mut duckdb_client_context) |  |  |  |  |

| `duckdb_destroy_config` | function | duckdb_destroy_config(config: *mut duckdb_config) |  |  |  |  |

| `duckdb_destroy_data_chunk` | function | duckdb_destroy_data_chunk(chunk: *mut duckdb_data_chunk) |  |  |  |  |

| `duckdb_destroy_error_data` | function | duckdb_destroy_error_data(error_data: *mut duckdb_error_data) |  |  |  |  |

| `duckdb_destroy_expression` | function | duckdb_destroy_expression(expr: *mut duckdb_expression) |  |  |  |  |

| `duckdb_destroy_extracted` | function | duckdb_destroy_extracted(extracted_statements: *mut duckdb_extracted_statements) |  |  |  |  |

| `duckdb_destroy_instance_cache` | function | duckdb_destroy_instance_cache(instance_cache: *mut duckdb_instance_cache) |  |  |  |  |

| `duckdb_destroy_logical_type` | function | duckdb_destroy_logical_type(type_: *mut duckdb_logical_type) |  |  |  |  |

| `duckdb_destroy_pending` | function | duckdb_destroy_pending(pending_result: *mut duckdb_pending_result) |  |  |  |  |

| `duckdb_destroy_prepare` | function | duckdb_destroy_prepare(prepared_statement: *mut duckdb_prepared_statement) |  |  |  |  |

| `duckdb_destroy_result` | function | duckdb_destroy_result(result: *mut duckdb_result) |  |  |  |  |

| `duckdb_destroy_scalar_function` | function | duckdb_destroy_scalar_function(scalar_function: *mut duckdb_scalar_function) |  |  |  |  |

| `duckdb_destroy_scalar_function_set` | function | duckdb_destroy_scalar_function_set(scalar_function_set: *mut duckdb_scalar_function_set) |  |  |  |  |

| `duckdb_destroy_selection_vector` | function | duckdb_destroy_selection_vector(sel: duckdb_selection_vector) |  |  |  |  |

| `duckdb_destroy_table_function` | function | duckdb_destroy_table_function(table_function: *mut duckdb_table_function) |  |  |  |  |

| `duckdb_destroy_task_state` | function | duckdb_destroy_task_state(state: duckdb_task_state) |  |  |  |  |

| `duckdb_destroy_value` | function | duckdb_destroy_value(value: *mut duckdb_value) |  |  |  |  |

| `duckdb_destroy_vector` | function | duckdb_destroy_vector(vector: *mut duckdb_vector) |  |  |  |  |

| `duckdb_disconnect` | function | duckdb_disconnect(connection: *mut duckdb_connection) |  |  |  |  |

| `duckdb_double_to_decimal` | function | duckdb_double_to_decimal(val: f64, width: u8, scale: u8) |  |  |  |  |

| `duckdb_double_to_hugeint` | function | duckdb_double_to_hugeint(val: f64) |  |  |  |  |

| `duckdb_double_to_uhugeint` | function | duckdb_double_to_uhugeint(val: f64) |  |  |  |  |

| `duckdb_enum_dictionary_size` | function | duckdb_enum_dictionary_size(type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_enum_dictionary_value` | function | duckdb_enum_dictionary_value(type_: duckdb_logical_type, index: idx_t) |  |  |  |  |

| `duckdb_enum_internal_type` | function | duckdb_enum_internal_type(type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_error_data_error_type` | function | duckdb_error_data_error_type(error_data: duckdb_error_data) |  |  |  |  |

| `duckdb_error_data_has_error` | function | duckdb_error_data_has_error(error_data: duckdb_error_data) |  |  |  |  |

| `duckdb_error_data_message` | function | duckdb_error_data_message(error_data: duckdb_error_data) |  |  |  |  |

| `duckdb_execute_n_tasks_state` | function | duckdb_execute_n_tasks_state(state: duckdb_task_state, max_tasks: idx_t) |  |  |  |  |

| `duckdb_execute_pending` | function | duckdb_execute_pending(
        pending_result: duckdb_pending_result,
        out_result: *mut duckdb_result,
    ) |  |  |  |  |

| `duckdb_execute_prepared` | function | duckdb_execute_prepared(
        prepared_statement: duckdb_prepared_statement,
        out_result: *mut duckdb_result,
    ) |  |  |  |  |

| `duckdb_execute_prepared_arrow` | function | duckdb_execute_prepared_arrow(
        prepared_statement: duckdb_prepared_statement,
        out_result: *mut duckdb_arrow,
    ) |  |  |  |  |

| `duckdb_execute_prepared_streaming` | function | duckdb_execute_prepared_streaming(
        prepared_statement: duckdb_prepared_statement,
        out_result: *mut duckdb_result,
    ) |  |  |  |  |

| `duckdb_execute_tasks` | function | duckdb_execute_tasks(database: duckdb_database, max_tasks: idx_t) |  |  |  |  |

| `duckdb_execute_tasks_state` | function | duckdb_execute_tasks_state(state: duckdb_task_state) |  |  |  |  |

| `duckdb_execution_is_finished` | function | duckdb_execution_is_finished(con: duckdb_connection) |  |  |  |  |

| `duckdb_expression_fold` | function | duckdb_expression_fold(
        context: duckdb_client_context,
        expr: duckdb_expression,
        out_value: *mut duckdb_value,
    ) |  |  |  |  |

| `duckdb_expression_is_foldable` | function | duckdb_expression_is_foldable(expr: duckdb_expression) |  |  |  |  |

| `duckdb_expression_return_type` | function | duckdb_expression_return_type(expr: duckdb_expression) |  |  |  |  |

| `duckdb_extract_statements` | function | duckdb_extract_statements(
        connection: duckdb_connection,
        query: *const ::std::os::raw::c_char,
        out_extracted_statements: *mut duckdb_extracted_statements,
    ) |  |  |  |  |

| `duckdb_extract_statements_error` | function | duckdb_extract_statements_error(
        extracted_statements: duckdb_extracted_statements,
    ) |  |  |  |  |

| `duckdb_fetch_chunk` | function | duckdb_fetch_chunk(result: duckdb_result) |  |  |  |  |

| `duckdb_finish_execution` | function | duckdb_finish_execution(state: duckdb_task_state) |  |  |  |  |

| `duckdb_free` | function | duckdb_free(ptr: *mut ::std::os::raw::c_void) |  |  |  |  |

| `duckdb_from_date` | function | duckdb_from_date(date: duckdb_date) |  |  |  |  |

| `duckdb_from_time` | function | duckdb_from_time(time: duckdb_time) |  |  |  |  |

| `duckdb_from_time_tz` | function | duckdb_from_time_tz(micros: duckdb_time_tz) |  |  |  |  |

| `duckdb_from_timestamp` | function | duckdb_from_timestamp(ts: duckdb_timestamp) |  |  |  |  |

| `duckdb_function_get_bind_data` | function | duckdb_function_get_bind_data(info: duckdb_function_info) |  |  |  |  |

| `duckdb_function_get_extra_info` | function | duckdb_function_get_extra_info(info: duckdb_function_info) |  |  |  |  |

| `duckdb_function_get_init_data` | function | duckdb_function_get_init_data(info: duckdb_function_info) |  |  |  |  |

| `duckdb_function_get_local_init_data` | function | duckdb_function_get_local_init_data(info: duckdb_function_info) |  |  |  |  |

| `duckdb_function_set_error` | function | duckdb_function_set_error(info: duckdb_function_info, error: *const ::std::os::raw::c_char) |  |  |  |  |

| `duckdb_get_bignum` | function | duckdb_get_bignum(val: duckdb_value) |  |  |  |  |

| `duckdb_get_bit` | function | duckdb_get_bit(val: duckdb_value) |  |  |  |  |

| `duckdb_get_blob` | function | duckdb_get_blob(val: duckdb_value) |  |  |  |  |

| `duckdb_get_bool` | function | duckdb_get_bool(val: duckdb_value) |  |  |  |  |

| `duckdb_get_config_flag` | function | duckdb_get_config_flag(
        index: usize,
        out_name: *mut *const ::std::os::raw::c_char,
        out_description: *mut *const ::std::os::raw::c_char,
    ) |  |  |  |  |

| `duckdb_get_date` | function | duckdb_get_date(val: duckdb_value) |  |  |  |  |

| `duckdb_get_decimal` | function | duckdb_get_decimal(val: duckdb_value) |  |  |  |  |

| `duckdb_get_double` | function | duckdb_get_double(val: duckdb_value) |  |  |  |  |

| `duckdb_get_enum_value` | function | duckdb_get_enum_value(value: duckdb_value) |  |  |  |  |

| `duckdb_get_float` | function | duckdb_get_float(val: duckdb_value) |  |  |  |  |

| `duckdb_get_hugeint` | function | duckdb_get_hugeint(val: duckdb_value) |  |  |  |  |

| `duckdb_get_int16` | function | duckdb_get_int16(val: duckdb_value) |  |  |  |  |

| `duckdb_get_int32` | function | duckdb_get_int32(val: duckdb_value) |  |  |  |  |

| `duckdb_get_int64` | function | duckdb_get_int64(val: duckdb_value) |  |  |  |  |

| `duckdb_get_int8` | function | duckdb_get_int8(val: duckdb_value) |  |  |  |  |

| `duckdb_get_interval` | function | duckdb_get_interval(val: duckdb_value) |  |  |  |  |

| `duckdb_get_list_child` | function | duckdb_get_list_child(value: duckdb_value, index: idx_t) |  |  |  |  |

| `duckdb_get_list_size` | function | duckdb_get_list_size(value: duckdb_value) |  |  |  |  |

| `duckdb_get_map_key` | function | duckdb_get_map_key(value: duckdb_value, index: idx_t) |  |  |  |  |

| `duckdb_get_map_size` | function | duckdb_get_map_size(value: duckdb_value) |  |  |  |  |

| `duckdb_get_map_value` | function | duckdb_get_map_value(value: duckdb_value, index: idx_t) |  |  |  |  |

| `duckdb_get_or_create_from_cache` | function | duckdb_get_or_create_from_cache(
        instance_cache: duckdb_instance_cache,
        path: *const ::std::os::raw::c_char,
        out_database: *mut duckdb_database,
        config: duckdb_config,
        out_error: *mut *mut ::std::os::raw::c_char,
    ) |  |  |  |  |

| `duckdb_get_profiling_info` | function | duckdb_get_profiling_info(connection: duckdb_connection) |  |  |  |  |

| `duckdb_get_struct_child` | function | duckdb_get_struct_child(value: duckdb_value, index: idx_t) |  |  |  |  |

| `duckdb_get_table_names` | function | duckdb_get_table_names(
        connection: duckdb_connection,
        query: *const ::std::os::raw::c_char,
        qualified: bool,
    ) |  |  |  |  |

| `duckdb_get_time` | function | duckdb_get_time(val: duckdb_value) |  |  |  |  |

| `duckdb_get_time_ns` | function | duckdb_get_time_ns(val: duckdb_value) |  |  |  |  |

| `duckdb_get_time_tz` | function | duckdb_get_time_tz(val: duckdb_value) |  |  |  |  |

| `duckdb_get_timestamp` | function | duckdb_get_timestamp(val: duckdb_value) |  |  |  |  |

| `duckdb_get_timestamp_ms` | function | duckdb_get_timestamp_ms(val: duckdb_value) |  |  |  |  |

| `duckdb_get_timestamp_ns` | function | duckdb_get_timestamp_ns(val: duckdb_value) |  |  |  |  |

| `duckdb_get_timestamp_s` | function | duckdb_get_timestamp_s(val: duckdb_value) |  |  |  |  |

| `duckdb_get_timestamp_tz` | function | duckdb_get_timestamp_tz(val: duckdb_value) |  |  |  |  |

| `duckdb_get_type_id` | function | duckdb_get_type_id(type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_get_uhugeint` | function | duckdb_get_uhugeint(val: duckdb_value) |  |  |  |  |

| `duckdb_get_uint16` | function | duckdb_get_uint16(val: duckdb_value) |  |  |  |  |

| `duckdb_get_uint32` | function | duckdb_get_uint32(val: duckdb_value) |  |  |  |  |

| `duckdb_get_uint64` | function | duckdb_get_uint64(val: duckdb_value) |  |  |  |  |

| `duckdb_get_uint8` | function | duckdb_get_uint8(val: duckdb_value) |  |  |  |  |

| `duckdb_get_uuid` | function | duckdb_get_uuid(val: duckdb_value) |  |  |  |  |

| `duckdb_get_value_type` | function | duckdb_get_value_type(val: duckdb_value) |  |  |  |  |

| `duckdb_get_varchar` | function | duckdb_get_varchar(value: duckdb_value) |  |  |  |  |

| `duckdb_hugeint_to_double` | function | duckdb_hugeint_to_double(val: duckdb_hugeint) |  |  |  |  |

| `duckdb_init_get_bind_data` | function | duckdb_init_get_bind_data(info: duckdb_init_info) |  |  |  |  |

| `duckdb_init_get_column_count` | function | duckdb_init_get_column_count(info: duckdb_init_info) |  |  |  |  |

| `duckdb_init_get_column_index` | function | duckdb_init_get_column_index(info: duckdb_init_info, column_index: idx_t) |  |  |  |  |

| `duckdb_init_get_extra_info` | function | duckdb_init_get_extra_info(info: duckdb_init_info) |  |  |  |  |

| `duckdb_init_set_error` | function | duckdb_init_set_error(info: duckdb_init_info, error: *const ::std::os::raw::c_char) |  |  |  |  |

| `duckdb_init_set_init_data` | function | duckdb_init_set_init_data(
        info: duckdb_init_info,
        init_data: *mut ::std::os::raw::c_void,
        destroy: duckdb_delete_callback_t,
    ) |  |  |  |  |

| `duckdb_init_set_max_threads` | function | duckdb_init_set_max_threads(info: duckdb_init_info, max_threads: idx_t) |  |  |  |  |

| `duckdb_interrupt` | function | duckdb_interrupt(connection: duckdb_connection) |  |  |  |  |

| `duckdb_is_finite_date` | function | duckdb_is_finite_date(date: duckdb_date) |  |  |  |  |

| `duckdb_is_finite_timestamp` | function | duckdb_is_finite_timestamp(ts: duckdb_timestamp) |  |  |  |  |

| `duckdb_is_finite_timestamp_ms` | function | duckdb_is_finite_timestamp_ms(ts: duckdb_timestamp_ms) |  |  |  |  |

| `duckdb_is_finite_timestamp_ns` | function | duckdb_is_finite_timestamp_ns(ts: duckdb_timestamp_ns) |  |  |  |  |

| `duckdb_is_finite_timestamp_s` | function | duckdb_is_finite_timestamp_s(ts: duckdb_timestamp_s) |  |  |  |  |

| `duckdb_is_null_value` | function | duckdb_is_null_value(value: duckdb_value) |  |  |  |  |

| `duckdb_library_version` | function | duckdb_library_version() |  |  |  |  |

| `duckdb_list_type_child_type` | function | duckdb_list_type_child_type(type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_list_vector_get_child` | function | duckdb_list_vector_get_child(vector: duckdb_vector) |  |  |  |  |

| `duckdb_list_vector_get_size` | function | duckdb_list_vector_get_size(vector: duckdb_vector) |  |  |  |  |

| `duckdb_list_vector_reserve` | function | duckdb_list_vector_reserve(vector: duckdb_vector, required_capacity: idx_t) |  |  |  |  |

| `duckdb_list_vector_set_size` | function | duckdb_list_vector_set_size(vector: duckdb_vector, size: idx_t) |  |  |  |  |

| `duckdb_logical_type_get_alias` | function | duckdb_logical_type_get_alias(type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_logical_type_set_alias` | function | duckdb_logical_type_set_alias(type_: duckdb_logical_type, alias: *const ::std::os::raw::c_char) |  |  |  |  |

| `duckdb_malloc` | function | duckdb_malloc(size: usize) |  |  |  |  |

| `duckdb_map_type_key_type` | function | duckdb_map_type_key_type(type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_map_type_value_type` | function | duckdb_map_type_value_type(type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_nparams` | function | duckdb_nparams(prepared_statement: duckdb_prepared_statement) |  |  |  |  |

| `duckdb_nullmask_data` | function | duckdb_nullmask_data(result: *mut duckdb_result, col: idx_t) |  |  |  |  |

| `duckdb_open` | function | duckdb_open(path: *const ::std::os::raw::c_char, out_database: *mut duckdb_database) |  |  |  |  |

| `duckdb_open_ext` | function | duckdb_open_ext(
        path: *const ::std::os::raw::c_char,
        out_database: *mut duckdb_database,
        config: duckdb_config,
        out_error: *mut *mut ::std::os::raw::c_char,
    ) |  |  |  |  |

| `duckdb_param_logical_type` | function | duckdb_param_logical_type(
        prepared_statement: duckdb_prepared_statement,
        param_idx: idx_t,
    ) |  |  |  |  |

| `duckdb_param_type` | function | duckdb_param_type(prepared_statement: duckdb_prepared_statement, param_idx: idx_t) |  |  |  |  |

| `duckdb_parameter_name` | function | duckdb_parameter_name(
        prepared_statement: duckdb_prepared_statement,
        index: idx_t,
    ) |  |  |  |  |

| `duckdb_pending_error` | function | duckdb_pending_error(pending_result: duckdb_pending_result) |  |  |  |  |

| `duckdb_pending_execute_check_state` | function | duckdb_pending_execute_check_state(pending_result: duckdb_pending_result) |  |  |  |  |

| `duckdb_pending_execute_task` | function | duckdb_pending_execute_task(pending_result: duckdb_pending_result) |  |  |  |  |

| `duckdb_pending_execution_is_finished` | function | duckdb_pending_execution_is_finished(pending_state: duckdb_pending_state) |  |  |  |  |

| `duckdb_pending_prepared` | function | duckdb_pending_prepared(
        prepared_statement: duckdb_prepared_statement,
        out_result: *mut duckdb_pending_result,
    ) |  |  |  |  |

| `duckdb_pending_prepared_streaming` | function | duckdb_pending_prepared_streaming(
        prepared_statement: duckdb_prepared_statement,
        out_result: *mut duckdb_pending_result,
    ) |  |  |  |  |

| `duckdb_prepare` | function | duckdb_prepare(
        connection: duckdb_connection,
        query: *const ::std::os::raw::c_char,
        out_prepared_statement: *mut duckdb_prepared_statement,
    ) |  |  |  |  |

| `duckdb_prepare_error` | function | duckdb_prepare_error(prepared_statement: duckdb_prepared_statement) |  |  |  |  |

| `duckdb_prepare_extracted_statement` | function | duckdb_prepare_extracted_statement(
        connection: duckdb_connection,
        extracted_statements: duckdb_extracted_statements,
        index: idx_t,
        out_prepared_statement: *mut duckdb_prepared_statement,
    ) |  |  |  |  |

| `duckdb_prepared_arrow_schema` | function | duckdb_prepared_arrow_schema(
        prepared: duckdb_prepared_statement,
        out_schema: *mut duckdb_arrow_schema,
    ) |  |  |  |  |

| `duckdb_prepared_statement_column_count` | function | duckdb_prepared_statement_column_count(prepared_statement: duckdb_prepared_statement) |  |  |  |  |

| `duckdb_prepared_statement_column_logical_type` | function | duckdb_prepared_statement_column_logical_type(
        prepared_statement: duckdb_prepared_statement,
        col_idx: idx_t,
    ) |  |  |  |  |

| `duckdb_prepared_statement_column_name` | function | duckdb_prepared_statement_column_name(
        prepared_statement: duckdb_prepared_statement,
        col_idx: idx_t,
    ) |  |  |  |  |

| `duckdb_prepared_statement_column_type` | function | duckdb_prepared_statement_column_type(
        prepared_statement: duckdb_prepared_statement,
        col_idx: idx_t,
    ) |  |  |  |  |

| `duckdb_prepared_statement_type` | function | duckdb_prepared_statement_type(statement: duckdb_prepared_statement) |  |  |  |  |

| `duckdb_profiling_info_get_child` | function | duckdb_profiling_info_get_child(info: duckdb_profiling_info, index: idx_t) |  |  |  |  |

| `duckdb_profiling_info_get_child_count` | function | duckdb_profiling_info_get_child_count(info: duckdb_profiling_info) |  |  |  |  |

| `duckdb_profiling_info_get_metrics` | function | duckdb_profiling_info_get_metrics(info: duckdb_profiling_info) |  |  |  |  |

| `duckdb_profiling_info_get_value` | function | duckdb_profiling_info_get_value(
        info: duckdb_profiling_info,
        key: *const ::std::os::raw::c_char,
    ) |  |  |  |  |

| `duckdb_query` | function | duckdb_query(
        connection: duckdb_connection,
        query: *const ::std::os::raw::c_char,
        out_result: *mut duckdb_result,
    ) |  |  |  |  |

| `duckdb_query_arrow` | function | duckdb_query_arrow(
        connection: duckdb_connection,
        query: *const ::std::os::raw::c_char,
        out_result: *mut duckdb_arrow,
    ) |  |  |  |  |

| `duckdb_query_arrow_array` | function | duckdb_query_arrow_array(result: duckdb_arrow, out_array: *mut duckdb_arrow_array) |  |  |  |  |

| `duckdb_query_arrow_error` | function | duckdb_query_arrow_error(result: duckdb_arrow) |  |  |  |  |

| `duckdb_query_arrow_schema` | function | duckdb_query_arrow_schema(result: duckdb_arrow, out_schema: *mut duckdb_arrow_schema) |  |  |  |  |

| `duckdb_query_progress` | function | duckdb_query_progress(connection: duckdb_connection) |  |  |  |  |

| `duckdb_register_aggregate_function` | function | duckdb_register_aggregate_function(
        con: duckdb_connection,
        aggregate_function: duckdb_aggregate_function,
    ) |  |  |  |  |

| `duckdb_register_aggregate_function_set` | function | duckdb_register_aggregate_function_set(
        con: duckdb_connection,
        set: duckdb_aggregate_function_set,
    ) |  |  |  |  |

| `duckdb_register_cast_function` | function | duckdb_register_cast_function(con: duckdb_connection, cast_function: duckdb_cast_function) |  |  |  |  |

| `duckdb_register_logical_type` | function | duckdb_register_logical_type(
        con: duckdb_connection,
        type_: duckdb_logical_type,
        info: duckdb_create_type_info,
    ) |  |  |  |  |

| `duckdb_register_scalar_function` | function | duckdb_register_scalar_function(
        con: duckdb_connection,
        scalar_function: duckdb_scalar_function,
    ) |  |  |  |  |

| `duckdb_register_scalar_function_set` | function | duckdb_register_scalar_function_set(con: duckdb_connection, set: duckdb_scalar_function_set) |  |  |  |  |

| `duckdb_register_table_function` | function | duckdb_register_table_function(con: duckdb_connection, function: duckdb_table_function) |  |  |  |  |

| `duckdb_replacement_scan_add_parameter` | function | duckdb_replacement_scan_add_parameter(info: duckdb_replacement_scan_info, parameter: duckdb_value) |  |  |  |  |

| `duckdb_replacement_scan_set_error` | function | duckdb_replacement_scan_set_error(info: duckdb_replacement_scan_info, error: *const ::std::os::raw::c_char) |  |  |  |  |

| `duckdb_replacement_scan_set_function_name` | function | duckdb_replacement_scan_set_function_name(
        info: duckdb_replacement_scan_info,
        function_name: *const ::std::os::raw::c_char,
    ) |  |  |  |  |

| `duckdb_result_arrow_array` | function | duckdb_result_arrow_array(
        result: duckdb_result,
        chunk: duckdb_data_chunk,
        out_array: *mut duckdb_arrow_array,
    ) |  |  |  |  |

| `duckdb_result_chunk_count` | function | duckdb_result_chunk_count(result: duckdb_result) |  |  |  |  |

| `duckdb_result_error` | function | duckdb_result_error(result: *mut duckdb_result) |  |  |  |  |

| `duckdb_result_error_type` | function | duckdb_result_error_type(result: *mut duckdb_result) |  |  |  |  |

| `duckdb_result_get_arrow_options` | function | duckdb_result_get_arrow_options(result: *mut duckdb_result) |  |  |  |  |

| `duckdb_result_get_chunk` | function | duckdb_result_get_chunk(result: duckdb_result, chunk_index: idx_t) |  |  |  |  |

| `duckdb_result_is_streaming` | function | duckdb_result_is_streaming(result: duckdb_result) |  |  |  |  |

| `duckdb_result_return_type` | function | duckdb_result_return_type(result: duckdb_result) |  |  |  |  |

| `duckdb_result_statement_type` | function | duckdb_result_statement_type(result: duckdb_result) |  |  |  |  |

| `duckdb_row_count` | function | duckdb_row_count(result: *mut duckdb_result) |  |  |  |  |

| `duckdb_rows_changed` | function | duckdb_rows_changed(result: *mut duckdb_result) |  |  |  |  |

| `duckdb_scalar_function_add_parameter` | function | duckdb_scalar_function_add_parameter(scalar_function: duckdb_scalar_function, type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_scalar_function_bind_get_argument` | function | duckdb_scalar_function_bind_get_argument(info: duckdb_bind_info, index: idx_t) |  |  |  |  |

| `duckdb_scalar_function_bind_get_argument_count` | function | duckdb_scalar_function_bind_get_argument_count(info: duckdb_bind_info) |  |  |  |  |

| `duckdb_scalar_function_bind_get_extra_info` | function | duckdb_scalar_function_bind_get_extra_info(info: duckdb_bind_info) |  |  |  |  |

| `duckdb_scalar_function_bind_set_error` | function | duckdb_scalar_function_bind_set_error(info: duckdb_bind_info, error: *const ::std::os::raw::c_char) |  |  |  |  |

| `duckdb_scalar_function_get_bind_data` | function | duckdb_scalar_function_get_bind_data(info: duckdb_function_info) |  |  |  |  |

| `duckdb_scalar_function_get_client_context` | function | duckdb_scalar_function_get_client_context(info: duckdb_bind_info, out_context: *mut duckdb_client_context) |  |  |  |  |

| `duckdb_scalar_function_get_extra_info` | function | duckdb_scalar_function_get_extra_info(info: duckdb_function_info) |  |  |  |  |

| `duckdb_scalar_function_set_bind` | function | duckdb_scalar_function_set_bind(
        scalar_function: duckdb_scalar_function,
        bind: duckdb_scalar_function_bind_t,
    ) |  |  |  |  |

| `duckdb_scalar_function_set_bind_data` | function | duckdb_scalar_function_set_bind_data(
        info: duckdb_bind_info,
        bind_data: *mut ::std::os::raw::c_void,
        destroy: duckdb_delete_callback_t,
    ) |  |  |  |  |

| `duckdb_scalar_function_set_bind_data_copy` | function | duckdb_scalar_function_set_bind_data_copy(info: duckdb_bind_info, copy: duckdb_copy_callback_t) |  |  |  |  |

| `duckdb_scalar_function_set_error` | function | duckdb_scalar_function_set_error(info: duckdb_function_info, error: *const ::std::os::raw::c_char) |  |  |  |  |

| `duckdb_scalar_function_set_extra_info` | function | duckdb_scalar_function_set_extra_info(
        scalar_function: duckdb_scalar_function,
        extra_info: *mut ::std::os::raw::c_void,
        destroy: duckdb_delete_callback_t,
    ) |  |  |  |  |

| `duckdb_scalar_function_set_function` | function | duckdb_scalar_function_set_function(
        scalar_function: duckdb_scalar_function,
        function: duckdb_scalar_function_t,
    ) |  |  |  |  |

| `duckdb_scalar_function_set_name` | function | duckdb_scalar_function_set_name(
        scalar_function: duckdb_scalar_function,
        name: *const ::std::os::raw::c_char,
    ) |  |  |  |  |

| `duckdb_scalar_function_set_return_type` | function | duckdb_scalar_function_set_return_type(scalar_function: duckdb_scalar_function, type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_scalar_function_set_special_handling` | function | duckdb_scalar_function_set_special_handling(scalar_function: duckdb_scalar_function) |  |  |  |  |

| `duckdb_scalar_function_set_varargs` | function | duckdb_scalar_function_set_varargs(scalar_function: duckdb_scalar_function, type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_scalar_function_set_volatile` | function | duckdb_scalar_function_set_volatile(scalar_function: duckdb_scalar_function) |  |  |  |  |

| `duckdb_schema_from_arrow` | function | duckdb_schema_from_arrow(
        connection: duckdb_connection,
        schema: *mut ArrowSchema,
        out_types: *mut duckdb_arrow_converted_schema,
    ) |  |  |  |  |

| `duckdb_selection_vector_get_data_ptr` | function | duckdb_selection_vector_get_data_ptr(sel: duckdb_selection_vector) |  |  |  |  |

| `duckdb_set_config` | function | duckdb_set_config(
        config: duckdb_config,
        name: *const ::std::os::raw::c_char,
        option: *const ::std::os::raw::c_char,
    ) |  |  |  |  |

| `duckdb_slice_vector` | function | duckdb_slice_vector(vector: duckdb_vector, sel: duckdb_selection_vector, len: idx_t) |  |  |  |  |

| `duckdb_stream_fetch_chunk` | function | duckdb_stream_fetch_chunk(result: duckdb_result) |  |  |  |  |

| `duckdb_string_is_inlined` | function | duckdb_string_is_inlined(string: duckdb_string_t) |  |  |  |  |

| `duckdb_string_t_data` | function | duckdb_string_t_data(string: *mut duckdb_string_t) |  |  |  |  |

| `duckdb_string_t_length` | function | duckdb_string_t_length(string: duckdb_string_t) |  |  |  |  |

| `duckdb_struct_type_child_count` | function | duckdb_struct_type_child_count(type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_struct_type_child_name` | function | duckdb_struct_type_child_name(type_: duckdb_logical_type, index: idx_t) |  |  |  |  |

| `duckdb_struct_type_child_type` | function | duckdb_struct_type_child_type(type_: duckdb_logical_type, index: idx_t) |  |  |  |  |

| `duckdb_struct_vector_get_child` | function | duckdb_struct_vector_get_child(vector: duckdb_vector, index: idx_t) |  |  |  |  |

| `duckdb_table_description_create` | function | duckdb_table_description_create(
        connection: duckdb_connection,
        schema: *const ::std::os::raw::c_char,
        table: *const ::std::os::raw::c_char,
        out: *mut duckdb_table_description,
    ) |  |  |  |  |

| `duckdb_table_description_create_ext` | function | duckdb_table_description_create_ext(
        connection: duckdb_connection,
        catalog: *const ::std::os::raw::c_char,
        schema: *const ::std::os::raw::c_char,
        table: *const ::std::os::raw::c_char,
        out: *mut duckdb_table_description,
    ) |  |  |  |  |

| `duckdb_table_description_destroy` | function | duckdb_table_description_destroy(table_description: *mut duckdb_table_description) |  |  |  |  |

| `duckdb_table_description_error` | function | duckdb_table_description_error(table_description: duckdb_table_description) |  |  |  |  |

| `duckdb_table_description_get_column_name` | function | duckdb_table_description_get_column_name(
        table_description: duckdb_table_description,
        index: idx_t,
    ) |  |  |  |  |

| `duckdb_table_function_add_named_parameter` | function | duckdb_table_function_add_named_parameter(
        table_function: duckdb_table_function,
        name: *const ::std::os::raw::c_char,
        type_: duckdb_logical_type,
    ) |  |  |  |  |

| `duckdb_table_function_add_parameter` | function | duckdb_table_function_add_parameter(table_function: duckdb_table_function, type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_table_function_get_client_context` | function | duckdb_table_function_get_client_context(info: duckdb_bind_info, out_context: *mut duckdb_client_context) |  |  |  |  |

| `duckdb_table_function_set_bind` | function | duckdb_table_function_set_bind(table_function: duckdb_table_function, bind: duckdb_table_function_bind_t) |  |  |  |  |

| `duckdb_table_function_set_extra_info` | function | duckdb_table_function_set_extra_info(
        table_function: duckdb_table_function,
        extra_info: *mut ::std::os::raw::c_void,
        destroy: duckdb_delete_callback_t,
    ) |  |  |  |  |

| `duckdb_table_function_set_function` | function | duckdb_table_function_set_function(table_function: duckdb_table_function, function: duckdb_table_function_t) |  |  |  |  |

| `duckdb_table_function_set_init` | function | duckdb_table_function_set_init(table_function: duckdb_table_function, init: duckdb_table_function_init_t) |  |  |  |  |

| `duckdb_table_function_set_local_init` | function | duckdb_table_function_set_local_init(
        table_function: duckdb_table_function,
        init: duckdb_table_function_init_t,
    ) |  |  |  |  |

| `duckdb_table_function_set_name` | function | duckdb_table_function_set_name(table_function: duckdb_table_function, name: *const ::std::os::raw::c_char) |  |  |  |  |

| `duckdb_table_function_supports_projection_pushdown` | function | duckdb_table_function_supports_projection_pushdown(table_function: duckdb_table_function, pushdown: bool) |  |  |  |  |

| `duckdb_task_state_is_finished` | function | duckdb_task_state_is_finished(state: duckdb_task_state) |  |  |  |  |

| `duckdb_to_arrow_schema` | function | duckdb_to_arrow_schema(
        arrow_options: duckdb_arrow_options,
        types: *mut duckdb_logical_type,
        names: *mut *const ::std::os::raw::c_char,
        column_count: idx_t,
        out_schema: *mut ArrowSchema,
    ) |  |  |  |  |

| `duckdb_to_date` | function | duckdb_to_date(date: duckdb_date_struct) |  |  |  |  |

| `duckdb_to_time` | function | duckdb_to_time(time: duckdb_time_struct) |  |  |  |  |

| `duckdb_to_timestamp` | function | duckdb_to_timestamp(ts: duckdb_timestamp_struct) |  |  |  |  |

| `duckdb_uhugeint_to_double` | function | duckdb_uhugeint_to_double(val: duckdb_uhugeint) |  |  |  |  |

| `duckdb_union_type_member_count` | function | duckdb_union_type_member_count(type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_union_type_member_name` | function | duckdb_union_type_member_name(type_: duckdb_logical_type, index: idx_t) |  |  |  |  |

| `duckdb_union_type_member_type` | function | duckdb_union_type_member_type(type_: duckdb_logical_type, index: idx_t) |  |  |  |  |

| `duckdb_validity_row_is_valid` | function | duckdb_validity_row_is_valid(validity: *mut u64, row: idx_t) |  |  |  |  |

| `duckdb_validity_set_row_invalid` | function | duckdb_validity_set_row_invalid(validity: *mut u64, row: idx_t) |  |  |  |  |

| `duckdb_validity_set_row_valid` | function | duckdb_validity_set_row_valid(validity: *mut u64, row: idx_t) |  |  |  |  |

| `duckdb_validity_set_row_validity` | function | duckdb_validity_set_row_validity(validity: *mut u64, row: idx_t, valid: bool) |  |  |  |  |

| `duckdb_value_blob` | function | duckdb_value_blob(result: *mut duckdb_result, col: idx_t, row: idx_t) |  |  |  |  |

| `duckdb_value_boolean` | function | duckdb_value_boolean(result: *mut duckdb_result, col: idx_t, row: idx_t) |  |  |  |  |

| `duckdb_value_date` | function | duckdb_value_date(result: *mut duckdb_result, col: idx_t, row: idx_t) |  |  |  |  |

| `duckdb_value_decimal` | function | duckdb_value_decimal(result: *mut duckdb_result, col: idx_t, row: idx_t) |  |  |  |  |

| `duckdb_value_double` | function | duckdb_value_double(result: *mut duckdb_result, col: idx_t, row: idx_t) |  |  |  |  |

| `duckdb_value_float` | function | duckdb_value_float(result: *mut duckdb_result, col: idx_t, row: idx_t) |  |  |  |  |

| `duckdb_value_hugeint` | function | duckdb_value_hugeint(result: *mut duckdb_result, col: idx_t, row: idx_t) |  |  |  |  |

| `duckdb_value_int16` | function | duckdb_value_int16(result: *mut duckdb_result, col: idx_t, row: idx_t) |  |  |  |  |

| `duckdb_value_int32` | function | duckdb_value_int32(result: *mut duckdb_result, col: idx_t, row: idx_t) |  |  |  |  |

| `duckdb_value_int64` | function | duckdb_value_int64(result: *mut duckdb_result, col: idx_t, row: idx_t) |  |  |  |  |

| `duckdb_value_int8` | function | duckdb_value_int8(result: *mut duckdb_result, col: idx_t, row: idx_t) |  |  |  |  |

| `duckdb_value_interval` | function | duckdb_value_interval(result: *mut duckdb_result, col: idx_t, row: idx_t) |  |  |  |  |

| `duckdb_value_is_null` | function | duckdb_value_is_null(result: *mut duckdb_result, col: idx_t, row: idx_t) |  |  |  |  |

| `duckdb_value_string` | function | duckdb_value_string(result: *mut duckdb_result, col: idx_t, row: idx_t) |  |  |  |  |

| `duckdb_value_string_internal` | function | duckdb_value_string_internal(result: *mut duckdb_result, col: idx_t, row: idx_t) |  |  |  |  |

| `duckdb_value_time` | function | duckdb_value_time(result: *mut duckdb_result, col: idx_t, row: idx_t) |  |  |  |  |

| `duckdb_value_timestamp` | function | duckdb_value_timestamp(result: *mut duckdb_result, col: idx_t, row: idx_t) |  |  |  |  |

| `duckdb_value_to_string` | function | duckdb_value_to_string(value: duckdb_value) |  |  |  |  |

| `duckdb_value_uhugeint` | function | duckdb_value_uhugeint(result: *mut duckdb_result, col: idx_t, row: idx_t) |  |  |  |  |

| `duckdb_value_uint16` | function | duckdb_value_uint16(result: *mut duckdb_result, col: idx_t, row: idx_t) |  |  |  |  |

| `duckdb_value_uint32` | function | duckdb_value_uint32(result: *mut duckdb_result, col: idx_t, row: idx_t) |  |  |  |  |

| `duckdb_value_uint64` | function | duckdb_value_uint64(result: *mut duckdb_result, col: idx_t, row: idx_t) |  |  |  |  |

| `duckdb_value_uint8` | function | duckdb_value_uint8(result: *mut duckdb_result, col: idx_t, row: idx_t) |  |  |  |  |

| `duckdb_value_varchar` | function | duckdb_value_varchar(result: *mut duckdb_result, col: idx_t, row: idx_t) |  |  |  |  |

| `duckdb_value_varchar_internal` | function | duckdb_value_varchar_internal(
        result: *mut duckdb_result,
        col: idx_t,
        row: idx_t,
    ) |  |  |  |  |

| `duckdb_vector_assign_string_element` | function | duckdb_vector_assign_string_element(
        vector: duckdb_vector,
        index: idx_t,
        str_: *const ::std::os::raw::c_char,
    ) |  |  |  |  |

| `duckdb_vector_assign_string_element_len` | function | duckdb_vector_assign_string_element_len(
        vector: duckdb_vector,
        index: idx_t,
        str_: *const ::std::os::raw::c_char,
        str_len: idx_t,
    ) |  |  |  |  |

| `duckdb_vector_copy_sel` | function | duckdb_vector_copy_sel(
        src: duckdb_vector,
        dst: duckdb_vector,
        sel: duckdb_selection_vector,
        src_count: idx_t,
        src_offset: idx_t,
        dst_offset: idx_t,
    ) |  |  |  |  |

| `duckdb_vector_ensure_validity_writable` | function | duckdb_vector_ensure_validity_writable(vector: duckdb_vector) |  |  |  |  |

| `duckdb_vector_get_column_type` | function | duckdb_vector_get_column_type(vector: duckdb_vector) |  |  |  |  |

| `duckdb_vector_get_data` | function | duckdb_vector_get_data(vector: duckdb_vector) |  |  |  |  |

| `duckdb_vector_get_validity` | function | duckdb_vector_get_validity(vector: duckdb_vector) |  |  |  |  |

| `duckdb_vector_reference_value` | function | duckdb_vector_reference_value(vector: duckdb_vector, value: duckdb_value) |  |  |  |  |

| `duckdb_vector_reference_vector` | function | duckdb_vector_reference_vector(to_vector: duckdb_vector, from_vector: duckdb_vector) |  |  |  |  |

| `duckdb_vector_size` | function | duckdb_vector_size() |  |  |  |  |

| `ArrowArray` | struct |  |  |  |  |  |

| `ArrowSchema` | struct |  |  |  |  |  |

| `duckdb_add_aggregate_function_to_set` | function | duckdb_add_aggregate_function_to_set(
    set: duckdb_aggregate_function_set,
    function: duckdb_aggregate_function,
) |  |  |  |  |

| `duckdb_add_replacement_scan` | function | duckdb_add_replacement_scan(
    db: duckdb_database,
    replacement: duckdb_replacement_callback_t,
    extra_data: *mut ::std::os::raw::c_void,
    delete_callback: duckdb_delete_callback_t,
) |  |  |  |  |

| `duckdb_add_scalar_function_to_set` | function | duckdb_add_scalar_function_to_set(
    set: duckdb_scalar_function_set,
    function: duckdb_scalar_function,
) |  |  |  |  |

| `duckdb_aggregate_function_add_parameter` | function | duckdb_aggregate_function_add_parameter(
    aggregate_function: duckdb_aggregate_function,
    type_: duckdb_logical_type,
) |  |  |  |  |

| `duckdb_aggregate_function_get_extra_info` | function | duckdb_aggregate_function_get_extra_info(
    info: duckdb_function_info,
) |  |  |  |  |

| `duckdb_aggregate_function_set_destructor` | function | duckdb_aggregate_function_set_destructor(
    aggregate_function: duckdb_aggregate_function,
    destroy: duckdb_aggregate_destroy_t,
) |  |  |  |  |

| `duckdb_aggregate_function_set_error` | function | duckdb_aggregate_function_set_error(
    info: duckdb_function_info,
    error: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_aggregate_function_set_extra_info` | function | duckdb_aggregate_function_set_extra_info(
    aggregate_function: duckdb_aggregate_function,
    extra_info: *mut ::std::os::raw::c_void,
    destroy: duckdb_delete_callback_t,
) |  |  |  |  |

| `duckdb_aggregate_function_set_functions` | function | duckdb_aggregate_function_set_functions(
    aggregate_function: duckdb_aggregate_function,
    state_size: duckdb_aggregate_state_size,
    state_init: duckdb_aggregate_init_t,
    update: duckdb_aggregate_update_t,
    combine: duckdb_aggregate_combine_t,
    finalize: duckdb_aggregate_finalize_t,
) |  |  |  |  |

| `duckdb_aggregate_function_set_name` | function | duckdb_aggregate_function_set_name(
    aggregate_function: duckdb_aggregate_function,
    name: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_aggregate_function_set_return_type` | function | duckdb_aggregate_function_set_return_type(
    aggregate_function: duckdb_aggregate_function,
    type_: duckdb_logical_type,
) |  |  |  |  |

| `duckdb_aggregate_function_set_special_handling` | function | duckdb_aggregate_function_set_special_handling(
    aggregate_function: duckdb_aggregate_function,
) |  |  |  |  |

| `duckdb_append_blob` | function | duckdb_append_blob(
    appender: duckdb_appender,
    data: *const ::std::os::raw::c_void,
    length: idx_t,
) |  |  |  |  |

| `duckdb_append_bool` | function | duckdb_append_bool(
    appender: duckdb_appender,
    value: bool,
) |  |  |  |  |

| `duckdb_append_data_chunk` | function | duckdb_append_data_chunk(
    appender: duckdb_appender,
    chunk: duckdb_data_chunk,
) |  |  |  |  |

| `duckdb_append_date` | function | duckdb_append_date(
    appender: duckdb_appender,
    value: duckdb_date,
) |  |  |  |  |

| `duckdb_append_default` | function | duckdb_append_default(appender: duckdb_appender) |  |  |  |  |

| `duckdb_append_default_to_chunk` | function | duckdb_append_default_to_chunk(
    appender: duckdb_appender,
    chunk: duckdb_data_chunk,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_append_double` | function | duckdb_append_double(
    appender: duckdb_appender,
    value: f64,
) |  |  |  |  |

| `duckdb_append_float` | function | duckdb_append_float(
    appender: duckdb_appender,
    value: f32,
) |  |  |  |  |

| `duckdb_append_hugeint` | function | duckdb_append_hugeint(
    appender: duckdb_appender,
    value: duckdb_hugeint,
) |  |  |  |  |

| `duckdb_append_int16` | function | duckdb_append_int16(
    appender: duckdb_appender,
    value: i16,
) |  |  |  |  |

| `duckdb_append_int32` | function | duckdb_append_int32(
    appender: duckdb_appender,
    value: i32,
) |  |  |  |  |

| `duckdb_append_int64` | function | duckdb_append_int64(
    appender: duckdb_appender,
    value: i64,
) |  |  |  |  |

| `duckdb_append_int8` | function | duckdb_append_int8(appender: duckdb_appender, value: i8) |  |  |  |  |

| `duckdb_append_interval` | function | duckdb_append_interval(
    appender: duckdb_appender,
    value: duckdb_interval,
) |  |  |  |  |

| `duckdb_append_null` | function | duckdb_append_null(appender: duckdb_appender) |  |  |  |  |

| `duckdb_append_time` | function | duckdb_append_time(
    appender: duckdb_appender,
    value: duckdb_time,
) |  |  |  |  |

| `duckdb_append_timestamp` | function | duckdb_append_timestamp(
    appender: duckdb_appender,
    value: duckdb_timestamp,
) |  |  |  |  |

| `duckdb_append_uhugeint` | function | duckdb_append_uhugeint(
    appender: duckdb_appender,
    value: duckdb_uhugeint,
) |  |  |  |  |

| `duckdb_append_uint16` | function | duckdb_append_uint16(
    appender: duckdb_appender,
    value: u16,
) |  |  |  |  |

| `duckdb_append_uint32` | function | duckdb_append_uint32(
    appender: duckdb_appender,
    value: u32,
) |  |  |  |  |

| `duckdb_append_uint64` | function | duckdb_append_uint64(
    appender: duckdb_appender,
    value: u64,
) |  |  |  |  |

| `duckdb_append_uint8` | function | duckdb_append_uint8(appender: duckdb_appender, value: u8) |  |  |  |  |

| `duckdb_append_value` | function | duckdb_append_value(
    appender: duckdb_appender,
    value: duckdb_value,
) |  |  |  |  |

| `duckdb_append_varchar` | function | duckdb_append_varchar(
    appender: duckdb_appender,
    val: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_append_varchar_length` | function | duckdb_append_varchar_length(
    appender: duckdb_appender,
    val: *const ::std::os::raw::c_char,
    length: idx_t,
) |  |  |  |  |

| `duckdb_appender_add_column` | function | duckdb_appender_add_column(
    appender: duckdb_appender,
    name: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_appender_begin_row` | function | duckdb_appender_begin_row(appender: duckdb_appender) |  |  |  |  |

| `duckdb_appender_clear_columns` | function | duckdb_appender_clear_columns(appender: duckdb_appender) |  |  |  |  |

| `duckdb_appender_close` | function | duckdb_appender_close(appender: duckdb_appender) |  |  |  |  |

| `duckdb_appender_column_count` | function | duckdb_appender_column_count(appender: duckdb_appender) |  |  |  |  |

| `duckdb_appender_column_type` | function | duckdb_appender_column_type(
    appender: duckdb_appender,
    col_idx: idx_t,
) |  |  |  |  |

| `duckdb_appender_create` | function | duckdb_appender_create(
    connection: duckdb_connection,
    schema: *const ::std::os::raw::c_char,
    table: *const ::std::os::raw::c_char,
    out_appender: *mut duckdb_appender,
) |  |  |  |  |

| `duckdb_appender_create_ext` | function | duckdb_appender_create_ext(
    connection: duckdb_connection,
    catalog: *const ::std::os::raw::c_char,
    schema: *const ::std::os::raw::c_char,
    table: *const ::std::os::raw::c_char,
    out_appender: *mut duckdb_appender,
) |  |  |  |  |

| `duckdb_appender_create_query` | function | duckdb_appender_create_query(
    connection: duckdb_connection,
    query: *const ::std::os::raw::c_char,
    column_count: idx_t,
    types: *mut duckdb_logical_type,
    table_name: *const ::std::os::raw::c_char,
    column_names: *mut *const ::std::os::raw::c_char,
    out_appender: *mut duckdb_appender,
) |  |  |  |  |

| `duckdb_appender_destroy` | function | duckdb_appender_destroy(appender: *mut duckdb_appender) |  |  |  |  |

| `duckdb_appender_end_row` | function | duckdb_appender_end_row(appender: duckdb_appender) |  |  |  |  |

| `duckdb_appender_error` | function | duckdb_appender_error(
    appender: duckdb_appender,
) |  |  |  |  |

| `duckdb_appender_error_data` | function | duckdb_appender_error_data(
    appender: duckdb_appender,
) |  |  |  |  |

| `duckdb_appender_flush` | function | duckdb_appender_flush(appender: duckdb_appender) |  |  |  |  |

| `duckdb_array_type_array_size` | function | duckdb_array_type_array_size(type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_array_type_child_type` | function | duckdb_array_type_child_type(
    type_: duckdb_logical_type,
) |  |  |  |  |

| `duckdb_array_vector_get_child` | function | duckdb_array_vector_get_child(vector: duckdb_vector) |  |  |  |  |

| `duckdb_arrow_array_scan` | function | duckdb_arrow_array_scan(
    connection: duckdb_connection,
    table_name: *const ::std::os::raw::c_char,
    arrow_schema: duckdb_arrow_schema,
    arrow_array: duckdb_arrow_array,
    out_stream: *mut duckdb_arrow_stream,
) |  |  |  |  |

| `duckdb_arrow_column_count` | function | duckdb_arrow_column_count(result: duckdb_arrow) |  |  |  |  |

| `duckdb_arrow_row_count` | function | duckdb_arrow_row_count(result: duckdb_arrow) |  |  |  |  |

| `duckdb_arrow_rows_changed` | function | duckdb_arrow_rows_changed(result: duckdb_arrow) |  |  |  |  |

| `duckdb_arrow_scan` | function | duckdb_arrow_scan(
    connection: duckdb_connection,
    table_name: *const ::std::os::raw::c_char,
    arrow: duckdb_arrow_stream,
) |  |  |  |  |

| `duckdb_bind_add_result_column` | function | duckdb_bind_add_result_column(
    info: duckdb_bind_info,
    name: *const ::std::os::raw::c_char,
    type_: duckdb_logical_type,
) |  |  |  |  |

| `duckdb_bind_blob` | function | duckdb_bind_blob(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
    data: *const ::std::os::raw::c_void,
    length: idx_t,
) |  |  |  |  |

| `duckdb_bind_boolean` | function | duckdb_bind_boolean(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
    val: bool,
) |  |  |  |  |

| `duckdb_bind_date` | function | duckdb_bind_date(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
    val: duckdb_date,
) |  |  |  |  |

| `duckdb_bind_decimal` | function | duckdb_bind_decimal(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
    val: duckdb_decimal,
) |  |  |  |  |

| `duckdb_bind_double` | function | duckdb_bind_double(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
    val: f64,
) |  |  |  |  |

| `duckdb_bind_float` | function | duckdb_bind_float(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
    val: f32,
) |  |  |  |  |

| `duckdb_bind_get_extra_info` | function | duckdb_bind_get_extra_info(
    info: duckdb_bind_info,
) |  |  |  |  |

| `duckdb_bind_get_named_parameter` | function | duckdb_bind_get_named_parameter(
    info: duckdb_bind_info,
    name: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_bind_get_parameter` | function | duckdb_bind_get_parameter(
    info: duckdb_bind_info,
    index: idx_t,
) |  |  |  |  |

| `duckdb_bind_get_parameter_count` | function | duckdb_bind_get_parameter_count(info: duckdb_bind_info) |  |  |  |  |

| `duckdb_bind_hugeint` | function | duckdb_bind_hugeint(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
    val: duckdb_hugeint,
) |  |  |  |  |

| `duckdb_bind_int16` | function | duckdb_bind_int16(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
    val: i16,
) |  |  |  |  |

| `duckdb_bind_int32` | function | duckdb_bind_int32(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
    val: i32,
) |  |  |  |  |

| `duckdb_bind_int64` | function | duckdb_bind_int64(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
    val: i64,
) |  |  |  |  |

| `duckdb_bind_int8` | function | duckdb_bind_int8(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
    val: i8,
) |  |  |  |  |

| `duckdb_bind_interval` | function | duckdb_bind_interval(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
    val: duckdb_interval,
) |  |  |  |  |

| `duckdb_bind_null` | function | duckdb_bind_null(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
) |  |  |  |  |

| `duckdb_bind_parameter_index` | function | duckdb_bind_parameter_index(
    prepared_statement: duckdb_prepared_statement,
    param_idx_out: *mut idx_t,
    name: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_bind_set_bind_data` | function | duckdb_bind_set_bind_data(
    info: duckdb_bind_info,
    bind_data: *mut ::std::os::raw::c_void,
    destroy: duckdb_delete_callback_t,
) |  |  |  |  |

| `duckdb_bind_set_cardinality` | function | duckdb_bind_set_cardinality(
    info: duckdb_bind_info,
    cardinality: idx_t,
    is_exact: bool,
) |  |  |  |  |

| `duckdb_bind_set_error` | function | duckdb_bind_set_error(
    info: duckdb_bind_info,
    error: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_bind_time` | function | duckdb_bind_time(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
    val: duckdb_time,
) |  |  |  |  |

| `duckdb_bind_timestamp` | function | duckdb_bind_timestamp(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
    val: duckdb_timestamp,
) |  |  |  |  |

| `duckdb_bind_timestamp_tz` | function | duckdb_bind_timestamp_tz(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
    val: duckdb_timestamp,
) |  |  |  |  |

| `duckdb_bind_uhugeint` | function | duckdb_bind_uhugeint(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
    val: duckdb_uhugeint,
) |  |  |  |  |

| `duckdb_bind_uint16` | function | duckdb_bind_uint16(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
    val: u16,
) |  |  |  |  |

| `duckdb_bind_uint32` | function | duckdb_bind_uint32(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
    val: u32,
) |  |  |  |  |

| `duckdb_bind_uint64` | function | duckdb_bind_uint64(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
    val: u64,
) |  |  |  |  |

| `duckdb_bind_uint8` | function | duckdb_bind_uint8(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
    val: u8,
) |  |  |  |  |

| `duckdb_bind_value` | function | duckdb_bind_value(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
    val: duckdb_value,
) |  |  |  |  |

| `duckdb_bind_varchar` | function | duckdb_bind_varchar(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
    val: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_bind_varchar_length` | function | duckdb_bind_varchar_length(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
    val: *const ::std::os::raw::c_char,
    length: idx_t,
) |  |  |  |  |

| `duckdb_cast_function_get_cast_mode` | function | duckdb_cast_function_get_cast_mode(
    info: duckdb_function_info,
) |  |  |  |  |

| `duckdb_cast_function_get_extra_info` | function | duckdb_cast_function_get_extra_info(
    info: duckdb_function_info,
) |  |  |  |  |

| `duckdb_cast_function_set_error` | function | duckdb_cast_function_set_error(
    info: duckdb_function_info,
    error: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_cast_function_set_extra_info` | function | duckdb_cast_function_set_extra_info(
    cast_function: duckdb_cast_function,
    extra_info: *mut ::std::os::raw::c_void,
    destroy: duckdb_delete_callback_t,
) |  |  |  |  |

| `duckdb_cast_function_set_function` | function | duckdb_cast_function_set_function(
    cast_function: duckdb_cast_function,
    function: duckdb_cast_function_t,
) |  |  |  |  |

| `duckdb_cast_function_set_implicit_cast_cost` | function | duckdb_cast_function_set_implicit_cast_cost(
    cast_function: duckdb_cast_function,
    cost: i64,
) |  |  |  |  |

| `duckdb_cast_function_set_row_error` | function | duckdb_cast_function_set_row_error(
    info: duckdb_function_info,
    error: *const ::std::os::raw::c_char,
    row: idx_t,
    output: duckdb_vector,
) |  |  |  |  |

| `duckdb_cast_function_set_source_type` | function | duckdb_cast_function_set_source_type(
    cast_function: duckdb_cast_function,
    source_type: duckdb_logical_type,
) |  |  |  |  |

| `duckdb_cast_function_set_target_type` | function | duckdb_cast_function_set_target_type(
    cast_function: duckdb_cast_function,
    target_type: duckdb_logical_type,
) |  |  |  |  |

| `duckdb_clear_bindings` | function | duckdb_clear_bindings(
    prepared_statement: duckdb_prepared_statement,
) |  |  |  |  |

| `duckdb_client_context_get_connection_id` | function | duckdb_client_context_get_connection_id(
    context: duckdb_client_context,
) |  |  |  |  |

| `duckdb_close` | function | duckdb_close(database: *mut duckdb_database) |  |  |  |  |

| `duckdb_column_count` | function | duckdb_column_count(result: *mut duckdb_result) |  |  |  |  |

| `duckdb_column_data` | function | duckdb_column_data(
    result: *mut duckdb_result,
    col: idx_t,
) |  |  |  |  |

| `duckdb_column_has_default` | function | duckdb_column_has_default(
    table_description: duckdb_table_description,
    index: idx_t,
    out: *mut bool,
) |  |  |  |  |

| `duckdb_column_logical_type` | function | duckdb_column_logical_type(
    result: *mut duckdb_result,
    col: idx_t,
) |  |  |  |  |

| `duckdb_column_name` | function | duckdb_column_name(
    result: *mut duckdb_result,
    col: idx_t,
) |  |  |  |  |

| `duckdb_column_type` | function | duckdb_column_type(result: *mut duckdb_result, col: idx_t) |  |  |  |  |

| `duckdb_config_count` | function | duckdb_config_count() |  |  |  |  |

| `duckdb_connect` | function | duckdb_connect(
    database: duckdb_database,
    out_connection: *mut duckdb_connection,
) |  |  |  |  |

| `duckdb_connection_get_arrow_options` | function | duckdb_connection_get_arrow_options(
    connection: duckdb_connection,
    out_arrow_options: *mut duckdb_arrow_options,
) |  |  |  |  |

| `duckdb_connection_get_client_context` | function | duckdb_connection_get_client_context(
    connection: duckdb_connection,
    out_context: *mut duckdb_client_context,
) |  |  |  |  |

| `duckdb_create_aggregate_function` | function | duckdb_create_aggregate_function() |  |  |  |  |

| `duckdb_create_aggregate_function_set` | function | duckdb_create_aggregate_function_set(
    name: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_create_array_type` | function | duckdb_create_array_type(
    type_: duckdb_logical_type,
    array_size: idx_t,
) |  |  |  |  |

| `duckdb_create_array_value` | function | duckdb_create_array_value(
    type_: duckdb_logical_type,
    values: *mut duckdb_value,
    value_count: idx_t,
) |  |  |  |  |

| `duckdb_create_bignum` | function | duckdb_create_bignum(input: duckdb_bignum) |  |  |  |  |

| `duckdb_create_bit` | function | duckdb_create_bit(input: duckdb_bit) |  |  |  |  |

| `duckdb_create_blob` | function | duckdb_create_blob(data: *const u8, length: idx_t) |  |  |  |  |

| `duckdb_create_bool` | function | duckdb_create_bool(input: bool) |  |  |  |  |

| `duckdb_create_cast_function` | function | duckdb_create_cast_function() |  |  |  |  |

| `duckdb_create_config` | function | duckdb_create_config(out_config: *mut duckdb_config) |  |  |  |  |

| `duckdb_create_data_chunk` | function | duckdb_create_data_chunk(
    types: *mut duckdb_logical_type,
    column_count: idx_t,
) |  |  |  |  |

| `duckdb_create_date` | function | duckdb_create_date(input: duckdb_date) |  |  |  |  |

| `duckdb_create_decimal` | function | duckdb_create_decimal(input: duckdb_decimal) |  |  |  |  |

| `duckdb_create_decimal_type` | function | duckdb_create_decimal_type(width: u8, scale: u8) |  |  |  |  |

| `duckdb_create_double` | function | duckdb_create_double(input: f64) |  |  |  |  |

| `duckdb_create_enum_type` | function | duckdb_create_enum_type(
    member_names: *mut *const ::std::os::raw::c_char,
    member_count: idx_t,
) |  |  |  |  |

| `duckdb_create_enum_value` | function | duckdb_create_enum_value(
    type_: duckdb_logical_type,
    value: u64,
) |  |  |  |  |

| `duckdb_create_error_data` | function | duckdb_create_error_data(
    type_: duckdb_error_type,
    message: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_create_float` | function | duckdb_create_float(input: f32) |  |  |  |  |

| `duckdb_create_hugeint` | function | duckdb_create_hugeint(input: duckdb_hugeint) |  |  |  |  |

| `duckdb_create_instance_cache` | function | duckdb_create_instance_cache() |  |  |  |  |

| `duckdb_create_int16` | function | duckdb_create_int16(input: i16) |  |  |  |  |

| `duckdb_create_int32` | function | duckdb_create_int32(input: i32) |  |  |  |  |

| `duckdb_create_int64` | function | duckdb_create_int64(val: i64) |  |  |  |  |

| `duckdb_create_int8` | function | duckdb_create_int8(input: i8) |  |  |  |  |

| `duckdb_create_interval` | function | duckdb_create_interval(input: duckdb_interval) |  |  |  |  |

| `duckdb_create_list_type` | function | duckdb_create_list_type(
    type_: duckdb_logical_type,
) |  |  |  |  |

| `duckdb_create_list_value` | function | duckdb_create_list_value(
    type_: duckdb_logical_type,
    values: *mut duckdb_value,
    value_count: idx_t,
) |  |  |  |  |

| `duckdb_create_logical_type` | function | duckdb_create_logical_type(type_: duckdb_type) |  |  |  |  |

| `duckdb_create_map_type` | function | duckdb_create_map_type(
    key_type: duckdb_logical_type,
    value_type: duckdb_logical_type,
) |  |  |  |  |

| `duckdb_create_map_value` | function | duckdb_create_map_value(
    map_type: duckdb_logical_type,
    keys: *mut duckdb_value,
    values: *mut duckdb_value,
    entry_count: idx_t,
) |  |  |  |  |

| `duckdb_create_null_value` | function | duckdb_create_null_value() |  |  |  |  |

| `duckdb_create_scalar_function` | function | duckdb_create_scalar_function() |  |  |  |  |

| `duckdb_create_scalar_function_set` | function | duckdb_create_scalar_function_set(
    name: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_create_selection_vector` | function | duckdb_create_selection_vector(size: idx_t) |  |  |  |  |

| `duckdb_create_struct_type` | function | duckdb_create_struct_type(
    member_types: *mut duckdb_logical_type,
    member_names: *mut *const ::std::os::raw::c_char,
    member_count: idx_t,
) |  |  |  |  |

| `duckdb_create_struct_value` | function | duckdb_create_struct_value(
    type_: duckdb_logical_type,
    values: *mut duckdb_value,
) |  |  |  |  |

| `duckdb_create_table_function` | function | duckdb_create_table_function() |  |  |  |  |

| `duckdb_create_task_state` | function | duckdb_create_task_state(database: duckdb_database) |  |  |  |  |

| `duckdb_create_time` | function | duckdb_create_time(input: duckdb_time) |  |  |  |  |

| `duckdb_create_time_ns` | function | duckdb_create_time_ns(input: duckdb_time_ns) |  |  |  |  |

| `duckdb_create_time_tz` | function | duckdb_create_time_tz(micros: i64, offset: i32) |  |  |  |  |

| `duckdb_create_time_tz_value` | function | duckdb_create_time_tz_value(value: duckdb_time_tz) |  |  |  |  |

| `duckdb_create_timestamp` | function | duckdb_create_timestamp(input: duckdb_timestamp) |  |  |  |  |

| `duckdb_create_timestamp_ms` | function | duckdb_create_timestamp_ms(input: duckdb_timestamp_ms) |  |  |  |  |

| `duckdb_create_timestamp_ns` | function | duckdb_create_timestamp_ns(input: duckdb_timestamp_ns) |  |  |  |  |

| `duckdb_create_timestamp_s` | function | duckdb_create_timestamp_s(input: duckdb_timestamp_s) |  |  |  |  |

| `duckdb_create_timestamp_tz` | function | duckdb_create_timestamp_tz(input: duckdb_timestamp) |  |  |  |  |

| `duckdb_create_uhugeint` | function | duckdb_create_uhugeint(input: duckdb_uhugeint) |  |  |  |  |

| `duckdb_create_uint16` | function | duckdb_create_uint16(input: u16) |  |  |  |  |

| `duckdb_create_uint32` | function | duckdb_create_uint32(input: u32) |  |  |  |  |

| `duckdb_create_uint64` | function | duckdb_create_uint64(input: u64) |  |  |  |  |

| `duckdb_create_uint8` | function | duckdb_create_uint8(input: u8) |  |  |  |  |

| `duckdb_create_union_type` | function | duckdb_create_union_type(
    member_types: *mut duckdb_logical_type,
    member_names: *mut *const ::std::os::raw::c_char,
    member_count: idx_t,
) |  |  |  |  |

| `duckdb_create_union_value` | function | duckdb_create_union_value(
    union_type: duckdb_logical_type,
    tag_index: idx_t,
    value: duckdb_value,
) |  |  |  |  |

| `duckdb_create_uuid` | function | duckdb_create_uuid(input: duckdb_uhugeint) |  |  |  |  |

| `duckdb_create_varchar` | function | duckdb_create_varchar(
    text: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_create_varchar_length` | function | duckdb_create_varchar_length(
    text: *const ::std::os::raw::c_char,
    length: idx_t,
) |  |  |  |  |

| `duckdb_create_vector` | function | duckdb_create_vector(
    type_: duckdb_logical_type,
    capacity: idx_t,
) |  |  |  |  |

| `duckdb_data_chunk_from_arrow` | function | duckdb_data_chunk_from_arrow(
    connection: duckdb_connection,
    arrow_array: *mut ArrowArray,
    converted_schema: duckdb_arrow_converted_schema,
    out_chunk: *mut duckdb_data_chunk,
) |  |  |  |  |

| `duckdb_data_chunk_get_column_count` | function | duckdb_data_chunk_get_column_count(chunk: duckdb_data_chunk) |  |  |  |  |

| `duckdb_data_chunk_get_size` | function | duckdb_data_chunk_get_size(chunk: duckdb_data_chunk) |  |  |  |  |

| `duckdb_data_chunk_get_vector` | function | duckdb_data_chunk_get_vector(
    chunk: duckdb_data_chunk,
    col_idx: idx_t,
) |  |  |  |  |

| `duckdb_data_chunk_reset` | function | duckdb_data_chunk_reset(chunk: duckdb_data_chunk) |  |  |  |  |

| `duckdb_data_chunk_set_size` | function | duckdb_data_chunk_set_size(chunk: duckdb_data_chunk, size: idx_t) |  |  |  |  |

| `duckdb_data_chunk_to_arrow` | function | duckdb_data_chunk_to_arrow(
    arrow_options: duckdb_arrow_options,
    chunk: duckdb_data_chunk,
    out_arrow_array: *mut ArrowArray,
) |  |  |  |  |

| `duckdb_decimal_internal_type` | function | duckdb_decimal_internal_type(type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_decimal_scale` | function | duckdb_decimal_scale(type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_decimal_to_double` | function | duckdb_decimal_to_double(val: duckdb_decimal) |  |  |  |  |

| `duckdb_decimal_width` | function | duckdb_decimal_width(type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_destroy_aggregate_function` | function | duckdb_destroy_aggregate_function(
    aggregate_function: *mut duckdb_aggregate_function,
) |  |  |  |  |

| `duckdb_destroy_aggregate_function_set` | function | duckdb_destroy_aggregate_function_set(
    aggregate_function_set: *mut duckdb_aggregate_function_set,
) |  |  |  |  |

| `duckdb_destroy_arrow` | function | duckdb_destroy_arrow(result: *mut duckdb_arrow) |  |  |  |  |

| `duckdb_destroy_arrow_converted_schema` | function | duckdb_destroy_arrow_converted_schema(
    arrow_converted_schema: *mut duckdb_arrow_converted_schema,
) |  |  |  |  |

| `duckdb_destroy_arrow_options` | function | duckdb_destroy_arrow_options(arrow_options: *mut duckdb_arrow_options) |  |  |  |  |

| `duckdb_destroy_arrow_stream` | function | duckdb_destroy_arrow_stream(stream_p: *mut duckdb_arrow_stream) |  |  |  |  |

| `duckdb_destroy_cast_function` | function | duckdb_destroy_cast_function(cast_function: *mut duckdb_cast_function) |  |  |  |  |

| `duckdb_destroy_client_context` | function | duckdb_destroy_client_context(context: *mut duckdb_client_context) |  |  |  |  |

| `duckdb_destroy_config` | function | duckdb_destroy_config(config: *mut duckdb_config) |  |  |  |  |

| `duckdb_destroy_data_chunk` | function | duckdb_destroy_data_chunk(chunk: *mut duckdb_data_chunk) |  |  |  |  |

| `duckdb_destroy_error_data` | function | duckdb_destroy_error_data(error_data: *mut duckdb_error_data) |  |  |  |  |

| `duckdb_destroy_expression` | function | duckdb_destroy_expression(expr: *mut duckdb_expression) |  |  |  |  |

| `duckdb_destroy_extracted` | function | duckdb_destroy_extracted(
    extracted_statements: *mut duckdb_extracted_statements,
) |  |  |  |  |

| `duckdb_destroy_instance_cache` | function | duckdb_destroy_instance_cache(instance_cache: *mut duckdb_instance_cache) |  |  |  |  |

| `duckdb_destroy_logical_type` | function | duckdb_destroy_logical_type(type_: *mut duckdb_logical_type) |  |  |  |  |

| `duckdb_destroy_pending` | function | duckdb_destroy_pending(pending_result: *mut duckdb_pending_result) |  |  |  |  |

| `duckdb_destroy_prepare` | function | duckdb_destroy_prepare(
    prepared_statement: *mut duckdb_prepared_statement,
) |  |  |  |  |

| `duckdb_destroy_result` | function | duckdb_destroy_result(result: *mut duckdb_result) |  |  |  |  |

| `duckdb_destroy_scalar_function` | function | duckdb_destroy_scalar_function(
    scalar_function: *mut duckdb_scalar_function,
) |  |  |  |  |

| `duckdb_destroy_scalar_function_set` | function | duckdb_destroy_scalar_function_set(
    scalar_function_set: *mut duckdb_scalar_function_set,
) |  |  |  |  |

| `duckdb_destroy_selection_vector` | function | duckdb_destroy_selection_vector(sel: duckdb_selection_vector) |  |  |  |  |

| `duckdb_destroy_table_function` | function | duckdb_destroy_table_function(table_function: *mut duckdb_table_function) |  |  |  |  |

| `duckdb_destroy_task_state` | function | duckdb_destroy_task_state(state: duckdb_task_state) |  |  |  |  |

| `duckdb_destroy_value` | function | duckdb_destroy_value(value: *mut duckdb_value) |  |  |  |  |

| `duckdb_destroy_vector` | function | duckdb_destroy_vector(vector: *mut duckdb_vector) |  |  |  |  |

| `duckdb_disconnect` | function | duckdb_disconnect(connection: *mut duckdb_connection) |  |  |  |  |

| `duckdb_double_to_decimal` | function | duckdb_double_to_decimal(
    val: f64,
    width: u8,
    scale: u8,
) |  |  |  |  |

| `duckdb_double_to_hugeint` | function | duckdb_double_to_hugeint(val: f64) |  |  |  |  |

| `duckdb_double_to_uhugeint` | function | duckdb_double_to_uhugeint(val: f64) |  |  |  |  |

| `duckdb_enum_dictionary_size` | function | duckdb_enum_dictionary_size(type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_enum_dictionary_value` | function | duckdb_enum_dictionary_value(
    type_: duckdb_logical_type,
    index: idx_t,
) |  |  |  |  |

| `duckdb_enum_internal_type` | function | duckdb_enum_internal_type(type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_error_data_error_type` | function | duckdb_error_data_error_type(
    error_data: duckdb_error_data,
) |  |  |  |  |

| `duckdb_error_data_has_error` | function | duckdb_error_data_has_error(error_data: duckdb_error_data) |  |  |  |  |

| `duckdb_error_data_message` | function | duckdb_error_data_message(
    error_data: duckdb_error_data,
) |  |  |  |  |

| `duckdb_execute_n_tasks_state` | function | duckdb_execute_n_tasks_state(
    state: duckdb_task_state,
    max_tasks: idx_t,
) |  |  |  |  |

| `duckdb_execute_pending` | function | duckdb_execute_pending(
    pending_result: duckdb_pending_result,
    out_result: *mut duckdb_result,
) |  |  |  |  |

| `duckdb_execute_prepared` | function | duckdb_execute_prepared(
    prepared_statement: duckdb_prepared_statement,
    out_result: *mut duckdb_result,
) |  |  |  |  |

| `duckdb_execute_prepared_arrow` | function | duckdb_execute_prepared_arrow(
    prepared_statement: duckdb_prepared_statement,
    out_result: *mut duckdb_arrow,
) |  |  |  |  |

| `duckdb_execute_prepared_streaming` | function | duckdb_execute_prepared_streaming(
    prepared_statement: duckdb_prepared_statement,
    out_result: *mut duckdb_result,
) |  |  |  |  |

| `duckdb_execute_tasks` | function | duckdb_execute_tasks(database: duckdb_database, max_tasks: idx_t) |  |  |  |  |

| `duckdb_execute_tasks_state` | function | duckdb_execute_tasks_state(state: duckdb_task_state) |  |  |  |  |

| `duckdb_execution_is_finished` | function | duckdb_execution_is_finished(con: duckdb_connection) |  |  |  |  |

| `duckdb_expression_fold` | function | duckdb_expression_fold(
    context: duckdb_client_context,
    expr: duckdb_expression,
    out_value: *mut duckdb_value,
) |  |  |  |  |

| `duckdb_expression_is_foldable` | function | duckdb_expression_is_foldable(expr: duckdb_expression) |  |  |  |  |

| `duckdb_expression_return_type` | function | duckdb_expression_return_type(
    expr: duckdb_expression,
) |  |  |  |  |

| `duckdb_extract_statements` | function | duckdb_extract_statements(
    connection: duckdb_connection,
    query: *const ::std::os::raw::c_char,
    out_extracted_statements: *mut duckdb_extracted_statements,
) |  |  |  |  |

| `duckdb_extract_statements_error` | function | duckdb_extract_statements_error(
    extracted_statements: duckdb_extracted_statements,
) |  |  |  |  |

| `duckdb_fetch_chunk` | function | duckdb_fetch_chunk(result: duckdb_result) |  |  |  |  |

| `duckdb_finish_execution` | function | duckdb_finish_execution(state: duckdb_task_state) |  |  |  |  |

| `duckdb_free` | function | duckdb_free(ptr: *mut ::std::os::raw::c_void) |  |  |  |  |

| `duckdb_from_date` | function | duckdb_from_date(date: duckdb_date) |  |  |  |  |

| `duckdb_from_time` | function | duckdb_from_time(time: duckdb_time) |  |  |  |  |

| `duckdb_from_time_tz` | function | duckdb_from_time_tz(micros: duckdb_time_tz) |  |  |  |  |

| `duckdb_from_timestamp` | function | duckdb_from_timestamp(ts: duckdb_timestamp) |  |  |  |  |

| `duckdb_function_get_bind_data` | function | duckdb_function_get_bind_data(
    info: duckdb_function_info,
) |  |  |  |  |

| `duckdb_function_get_extra_info` | function | duckdb_function_get_extra_info(
    info: duckdb_function_info,
) |  |  |  |  |

| `duckdb_function_get_init_data` | function | duckdb_function_get_init_data(
    info: duckdb_function_info,
) |  |  |  |  |

| `duckdb_function_get_local_init_data` | function | duckdb_function_get_local_init_data(
    info: duckdb_function_info,
) |  |  |  |  |

| `duckdb_function_set_error` | function | duckdb_function_set_error(
    info: duckdb_function_info,
    error: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_get_bignum` | function | duckdb_get_bignum(val: duckdb_value) |  |  |  |  |

| `duckdb_get_bit` | function | duckdb_get_bit(val: duckdb_value) |  |  |  |  |

| `duckdb_get_blob` | function | duckdb_get_blob(val: duckdb_value) |  |  |  |  |

| `duckdb_get_bool` | function | duckdb_get_bool(val: duckdb_value) |  |  |  |  |

| `duckdb_get_config_flag` | function | duckdb_get_config_flag(
    index: usize,
    out_name: *mut *const ::std::os::raw::c_char,
    out_description: *mut *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_get_date` | function | duckdb_get_date(val: duckdb_value) |  |  |  |  |

| `duckdb_get_decimal` | function | duckdb_get_decimal(val: duckdb_value) |  |  |  |  |

| `duckdb_get_double` | function | duckdb_get_double(val: duckdb_value) |  |  |  |  |

| `duckdb_get_enum_value` | function | duckdb_get_enum_value(value: duckdb_value) |  |  |  |  |

| `duckdb_get_float` | function | duckdb_get_float(val: duckdb_value) |  |  |  |  |

| `duckdb_get_hugeint` | function | duckdb_get_hugeint(val: duckdb_value) |  |  |  |  |

| `duckdb_get_int16` | function | duckdb_get_int16(val: duckdb_value) |  |  |  |  |

| `duckdb_get_int32` | function | duckdb_get_int32(val: duckdb_value) |  |  |  |  |

| `duckdb_get_int64` | function | duckdb_get_int64(val: duckdb_value) |  |  |  |  |

| `duckdb_get_int8` | function | duckdb_get_int8(val: duckdb_value) |  |  |  |  |

| `duckdb_get_interval` | function | duckdb_get_interval(val: duckdb_value) |  |  |  |  |

| `duckdb_get_list_child` | function | duckdb_get_list_child(value: duckdb_value, index: idx_t) |  |  |  |  |

| `duckdb_get_list_size` | function | duckdb_get_list_size(value: duckdb_value) |  |  |  |  |

| `duckdb_get_map_key` | function | duckdb_get_map_key(value: duckdb_value, index: idx_t) |  |  |  |  |

| `duckdb_get_map_size` | function | duckdb_get_map_size(value: duckdb_value) |  |  |  |  |

| `duckdb_get_map_value` | function | duckdb_get_map_value(value: duckdb_value, index: idx_t) |  |  |  |  |

| `duckdb_get_or_create_from_cache` | function | duckdb_get_or_create_from_cache(
    instance_cache: duckdb_instance_cache,
    path: *const ::std::os::raw::c_char,
    out_database: *mut duckdb_database,
    config: duckdb_config,
    out_error: *mut *mut ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_get_profiling_info` | function | duckdb_get_profiling_info(
    connection: duckdb_connection,
) |  |  |  |  |

| `duckdb_get_struct_child` | function | duckdb_get_struct_child(
    value: duckdb_value,
    index: idx_t,
) |  |  |  |  |

| `duckdb_get_table_names` | function | duckdb_get_table_names(
    connection: duckdb_connection,
    query: *const ::std::os::raw::c_char,
    qualified: bool,
) |  |  |  |  |

| `duckdb_get_time` | function | duckdb_get_time(val: duckdb_value) |  |  |  |  |

| `duckdb_get_time_ns` | function | duckdb_get_time_ns(val: duckdb_value) |  |  |  |  |

| `duckdb_get_time_tz` | function | duckdb_get_time_tz(val: duckdb_value) |  |  |  |  |

| `duckdb_get_timestamp` | function | duckdb_get_timestamp(val: duckdb_value) |  |  |  |  |

| `duckdb_get_timestamp_ms` | function | duckdb_get_timestamp_ms(val: duckdb_value) |  |  |  |  |

| `duckdb_get_timestamp_ns` | function | duckdb_get_timestamp_ns(val: duckdb_value) |  |  |  |  |

| `duckdb_get_timestamp_s` | function | duckdb_get_timestamp_s(val: duckdb_value) |  |  |  |  |

| `duckdb_get_timestamp_tz` | function | duckdb_get_timestamp_tz(val: duckdb_value) |  |  |  |  |

| `duckdb_get_type_id` | function | duckdb_get_type_id(type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_get_uhugeint` | function | duckdb_get_uhugeint(val: duckdb_value) |  |  |  |  |

| `duckdb_get_uint16` | function | duckdb_get_uint16(val: duckdb_value) |  |  |  |  |

| `duckdb_get_uint32` | function | duckdb_get_uint32(val: duckdb_value) |  |  |  |  |

| `duckdb_get_uint64` | function | duckdb_get_uint64(val: duckdb_value) |  |  |  |  |

| `duckdb_get_uint8` | function | duckdb_get_uint8(val: duckdb_value) |  |  |  |  |

| `duckdb_get_uuid` | function | duckdb_get_uuid(val: duckdb_value) |  |  |  |  |

| `duckdb_get_value_type` | function | duckdb_get_value_type(val: duckdb_value) |  |  |  |  |

| `duckdb_get_varchar` | function | duckdb_get_varchar(value: duckdb_value) |  |  |  |  |

| `duckdb_hugeint_to_double` | function | duckdb_hugeint_to_double(val: duckdb_hugeint) |  |  |  |  |

| `duckdb_init_get_bind_data` | function | duckdb_init_get_bind_data(
    info: duckdb_init_info,
) |  |  |  |  |

| `duckdb_init_get_column_count` | function | duckdb_init_get_column_count(info: duckdb_init_info) |  |  |  |  |

| `duckdb_init_get_column_index` | function | duckdb_init_get_column_index(
    info: duckdb_init_info,
    column_index: idx_t,
) |  |  |  |  |

| `duckdb_init_get_extra_info` | function | duckdb_init_get_extra_info(
    info: duckdb_init_info,
) |  |  |  |  |

| `duckdb_init_set_error` | function | duckdb_init_set_error(
    info: duckdb_init_info,
    error: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_init_set_init_data` | function | duckdb_init_set_init_data(
    info: duckdb_init_info,
    init_data: *mut ::std::os::raw::c_void,
    destroy: duckdb_delete_callback_t,
) |  |  |  |  |

| `duckdb_init_set_max_threads` | function | duckdb_init_set_max_threads(info: duckdb_init_info, max_threads: idx_t) |  |  |  |  |

| `duckdb_interrupt` | function | duckdb_interrupt(connection: duckdb_connection) |  |  |  |  |

| `duckdb_is_finite_date` | function | duckdb_is_finite_date(date: duckdb_date) |  |  |  |  |

| `duckdb_is_finite_timestamp` | function | duckdb_is_finite_timestamp(ts: duckdb_timestamp) |  |  |  |  |

| `duckdb_is_finite_timestamp_ms` | function | duckdb_is_finite_timestamp_ms(ts: duckdb_timestamp_ms) |  |  |  |  |

| `duckdb_is_finite_timestamp_ns` | function | duckdb_is_finite_timestamp_ns(ts: duckdb_timestamp_ns) |  |  |  |  |

| `duckdb_is_finite_timestamp_s` | function | duckdb_is_finite_timestamp_s(ts: duckdb_timestamp_s) |  |  |  |  |

| `duckdb_is_null_value` | function | duckdb_is_null_value(value: duckdb_value) |  |  |  |  |

| `duckdb_library_version` | function | duckdb_library_version() |  |  |  |  |

| `duckdb_list_type_child_type` | function | duckdb_list_type_child_type(
    type_: duckdb_logical_type,
) |  |  |  |  |

| `duckdb_list_vector_get_child` | function | duckdb_list_vector_get_child(vector: duckdb_vector) |  |  |  |  |

| `duckdb_list_vector_get_size` | function | duckdb_list_vector_get_size(vector: duckdb_vector) |  |  |  |  |

| `duckdb_list_vector_reserve` | function | duckdb_list_vector_reserve(
    vector: duckdb_vector,
    required_capacity: idx_t,
) |  |  |  |  |

| `duckdb_list_vector_set_size` | function | duckdb_list_vector_set_size(
    vector: duckdb_vector,
    size: idx_t,
) |  |  |  |  |

| `duckdb_logical_type_get_alias` | function | duckdb_logical_type_get_alias(
    type_: duckdb_logical_type,
) |  |  |  |  |

| `duckdb_logical_type_set_alias` | function | duckdb_logical_type_set_alias(
    type_: duckdb_logical_type,
    alias: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_malloc` | function | duckdb_malloc(size: usize) |  |  |  |  |

| `duckdb_map_type_key_type` | function | duckdb_map_type_key_type(
    type_: duckdb_logical_type,
) |  |  |  |  |

| `duckdb_map_type_value_type` | function | duckdb_map_type_value_type(
    type_: duckdb_logical_type,
) |  |  |  |  |

| `duckdb_nparams` | function | duckdb_nparams(prepared_statement: duckdb_prepared_statement) |  |  |  |  |

| `duckdb_nullmask_data` | function | duckdb_nullmask_data(result: *mut duckdb_result, col: idx_t) |  |  |  |  |

| `duckdb_open` | function | duckdb_open(
    path: *const ::std::os::raw::c_char,
    out_database: *mut duckdb_database,
) |  |  |  |  |

| `duckdb_open_ext` | function | duckdb_open_ext(
    path: *const ::std::os::raw::c_char,
    out_database: *mut duckdb_database,
    config: duckdb_config,
    out_error: *mut *mut ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_param_logical_type` | function | duckdb_param_logical_type(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
) |  |  |  |  |

| `duckdb_param_type` | function | duckdb_param_type(
    prepared_statement: duckdb_prepared_statement,
    param_idx: idx_t,
) |  |  |  |  |

| `duckdb_parameter_name` | function | duckdb_parameter_name(
    prepared_statement: duckdb_prepared_statement,
    index: idx_t,
) |  |  |  |  |

| `duckdb_pending_error` | function | duckdb_pending_error(
    pending_result: duckdb_pending_result,
) |  |  |  |  |

| `duckdb_pending_execute_check_state` | function | duckdb_pending_execute_check_state(
    pending_result: duckdb_pending_result,
) |  |  |  |  |

| `duckdb_pending_execute_task` | function | duckdb_pending_execute_task(
    pending_result: duckdb_pending_result,
) |  |  |  |  |

| `duckdb_pending_execution_is_finished` | function | duckdb_pending_execution_is_finished(
    pending_state: duckdb_pending_state,
) |  |  |  |  |

| `duckdb_pending_prepared` | function | duckdb_pending_prepared(
    prepared_statement: duckdb_prepared_statement,
    out_result: *mut duckdb_pending_result,
) |  |  |  |  |

| `duckdb_pending_prepared_streaming` | function | duckdb_pending_prepared_streaming(
    prepared_statement: duckdb_prepared_statement,
    out_result: *mut duckdb_pending_result,
) |  |  |  |  |

| `duckdb_prepare` | function | duckdb_prepare(
    connection: duckdb_connection,
    query: *const ::std::os::raw::c_char,
    out_prepared_statement: *mut duckdb_prepared_statement,
) |  |  |  |  |

| `duckdb_prepare_error` | function | duckdb_prepare_error(
    prepared_statement: duckdb_prepared_statement,
) |  |  |  |  |

| `duckdb_prepare_extracted_statement` | function | duckdb_prepare_extracted_statement(
    connection: duckdb_connection,
    extracted_statements: duckdb_extracted_statements,
    index: idx_t,
    out_prepared_statement: *mut duckdb_prepared_statement,
) |  |  |  |  |

| `duckdb_prepared_arrow_schema` | function | duckdb_prepared_arrow_schema(
    prepared: duckdb_prepared_statement,
    out_schema: *mut duckdb_arrow_schema,
) |  |  |  |  |

| `duckdb_prepared_statement_column_count` | function | duckdb_prepared_statement_column_count(
    prepared_statement: duckdb_prepared_statement,
) |  |  |  |  |

| `duckdb_prepared_statement_column_logical_type` | function | duckdb_prepared_statement_column_logical_type(
    prepared_statement: duckdb_prepared_statement,
    col_idx: idx_t,
) |  |  |  |  |

| `duckdb_prepared_statement_column_name` | function | duckdb_prepared_statement_column_name(
    prepared_statement: duckdb_prepared_statement,
    col_idx: idx_t,
) |  |  |  |  |

| `duckdb_prepared_statement_column_type` | function | duckdb_prepared_statement_column_type(
    prepared_statement: duckdb_prepared_statement,
    col_idx: idx_t,
) |  |  |  |  |

| `duckdb_prepared_statement_type` | function | duckdb_prepared_statement_type(
    statement: duckdb_prepared_statement,
) |  |  |  |  |

| `duckdb_profiling_info_get_child` | function | duckdb_profiling_info_get_child(
    info: duckdb_profiling_info,
    index: idx_t,
) |  |  |  |  |

| `duckdb_profiling_info_get_child_count` | function | duckdb_profiling_info_get_child_count(
    info: duckdb_profiling_info,
) |  |  |  |  |

| `duckdb_profiling_info_get_metrics` | function | duckdb_profiling_info_get_metrics(
    info: duckdb_profiling_info,
) |  |  |  |  |

| `duckdb_profiling_info_get_value` | function | duckdb_profiling_info_get_value(
    info: duckdb_profiling_info,
    key: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_query` | function | duckdb_query(
    connection: duckdb_connection,
    query: *const ::std::os::raw::c_char,
    out_result: *mut duckdb_result,
) |  |  |  |  |

| `duckdb_query_arrow` | function | duckdb_query_arrow(
    connection: duckdb_connection,
    query: *const ::std::os::raw::c_char,
    out_result: *mut duckdb_arrow,
) |  |  |  |  |

| `duckdb_query_arrow_array` | function | duckdb_query_arrow_array(
    result: duckdb_arrow,
    out_array: *mut duckdb_arrow_array,
) |  |  |  |  |

| `duckdb_query_arrow_error` | function | duckdb_query_arrow_error(
    result: duckdb_arrow,
) |  |  |  |  |

| `duckdb_query_arrow_schema` | function | duckdb_query_arrow_schema(
    result: duckdb_arrow,
    out_schema: *mut duckdb_arrow_schema,
) |  |  |  |  |

| `duckdb_query_progress` | function | duckdb_query_progress(
    connection: duckdb_connection,
) |  |  |  |  |

| `duckdb_register_aggregate_function` | function | duckdb_register_aggregate_function(
    con: duckdb_connection,
    aggregate_function: duckdb_aggregate_function,
) |  |  |  |  |

| `duckdb_register_aggregate_function_set` | function | duckdb_register_aggregate_function_set(
    con: duckdb_connection,
    set: duckdb_aggregate_function_set,
) |  |  |  |  |

| `duckdb_register_cast_function` | function | duckdb_register_cast_function(
    con: duckdb_connection,
    cast_function: duckdb_cast_function,
) |  |  |  |  |

| `duckdb_register_logical_type` | function | duckdb_register_logical_type(
    con: duckdb_connection,
    type_: duckdb_logical_type,
    info: duckdb_create_type_info,
) |  |  |  |  |

| `duckdb_register_scalar_function` | function | duckdb_register_scalar_function(
    con: duckdb_connection,
    scalar_function: duckdb_scalar_function,
) |  |  |  |  |

| `duckdb_register_scalar_function_set` | function | duckdb_register_scalar_function_set(
    con: duckdb_connection,
    set: duckdb_scalar_function_set,
) |  |  |  |  |

| `duckdb_register_table_function` | function | duckdb_register_table_function(
    con: duckdb_connection,
    function: duckdb_table_function,
) |  |  |  |  |

| `duckdb_replacement_scan_add_parameter` | function | duckdb_replacement_scan_add_parameter(
    info: duckdb_replacement_scan_info,
    parameter: duckdb_value,
) |  |  |  |  |

| `duckdb_replacement_scan_set_error` | function | duckdb_replacement_scan_set_error(
    info: duckdb_replacement_scan_info,
    error: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_replacement_scan_set_function_name` | function | duckdb_replacement_scan_set_function_name(
    info: duckdb_replacement_scan_info,
    function_name: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_result_arrow_array` | function | duckdb_result_arrow_array(
    result: duckdb_result,
    chunk: duckdb_data_chunk,
    out_array: *mut duckdb_arrow_array,
) |  |  |  |  |

| `duckdb_result_chunk_count` | function | duckdb_result_chunk_count(result: duckdb_result) |  |  |  |  |

| `duckdb_result_error` | function | duckdb_result_error(
    result: *mut duckdb_result,
) |  |  |  |  |

| `duckdb_result_error_type` | function | duckdb_result_error_type(result: *mut duckdb_result) |  |  |  |  |

| `duckdb_result_get_arrow_options` | function | duckdb_result_get_arrow_options(
    result: *mut duckdb_result,
) |  |  |  |  |

| `duckdb_result_get_chunk` | function | duckdb_result_get_chunk(
    result: duckdb_result,
    chunk_index: idx_t,
) |  |  |  |  |

| `duckdb_result_is_streaming` | function | duckdb_result_is_streaming(result: duckdb_result) |  |  |  |  |

| `duckdb_result_return_type` | function | duckdb_result_return_type(result: duckdb_result) |  |  |  |  |

| `duckdb_result_statement_type` | function | duckdb_result_statement_type(
    result: duckdb_result,
) |  |  |  |  |

| `duckdb_row_count` | function | duckdb_row_count(result: *mut duckdb_result) |  |  |  |  |

| `duckdb_rows_changed` | function | duckdb_rows_changed(result: *mut duckdb_result) |  |  |  |  |

| `duckdb_rs_extension_api_init` | function | duckdb_rs_extension_api_init(
    info: duckdb_extension_info,
    access: *const duckdb_extension_access,
    version: &str,
) |  |  |  |  |

| `duckdb_scalar_function_add_parameter` | function | duckdb_scalar_function_add_parameter(
    scalar_function: duckdb_scalar_function,
    type_: duckdb_logical_type,
) |  |  |  |  |

| `duckdb_scalar_function_bind_get_argument` | function | duckdb_scalar_function_bind_get_argument(
    info: duckdb_bind_info,
    index: idx_t,
) |  |  |  |  |

| `duckdb_scalar_function_bind_get_argument_count` | function | duckdb_scalar_function_bind_get_argument_count(
    info: duckdb_bind_info,
) |  |  |  |  |

| `duckdb_scalar_function_bind_get_extra_info` | function | duckdb_scalar_function_bind_get_extra_info(
    info: duckdb_bind_info,
) |  |  |  |  |

| `duckdb_scalar_function_bind_set_error` | function | duckdb_scalar_function_bind_set_error(
    info: duckdb_bind_info,
    error: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_scalar_function_get_bind_data` | function | duckdb_scalar_function_get_bind_data(
    info: duckdb_function_info,
) |  |  |  |  |

| `duckdb_scalar_function_get_client_context` | function | duckdb_scalar_function_get_client_context(
    info: duckdb_bind_info,
    out_context: *mut duckdb_client_context,
) |  |  |  |  |

| `duckdb_scalar_function_get_extra_info` | function | duckdb_scalar_function_get_extra_info(
    info: duckdb_function_info,
) |  |  |  |  |

| `duckdb_scalar_function_set_bind` | function | duckdb_scalar_function_set_bind(
    scalar_function: duckdb_scalar_function,
    bind: duckdb_scalar_function_bind_t,
) |  |  |  |  |

| `duckdb_scalar_function_set_bind_data` | function | duckdb_scalar_function_set_bind_data(
    info: duckdb_bind_info,
    bind_data: *mut ::std::os::raw::c_void,
    destroy: duckdb_delete_callback_t,
) |  |  |  |  |

| `duckdb_scalar_function_set_bind_data_copy` | function | duckdb_scalar_function_set_bind_data_copy(
    info: duckdb_bind_info,
    copy: duckdb_copy_callback_t,
) |  |  |  |  |

| `duckdb_scalar_function_set_error` | function | duckdb_scalar_function_set_error(
    info: duckdb_function_info,
    error: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_scalar_function_set_extra_info` | function | duckdb_scalar_function_set_extra_info(
    scalar_function: duckdb_scalar_function,
    extra_info: *mut ::std::os::raw::c_void,
    destroy: duckdb_delete_callback_t,
) |  |  |  |  |

| `duckdb_scalar_function_set_function` | function | duckdb_scalar_function_set_function(
    scalar_function: duckdb_scalar_function,
    function: duckdb_scalar_function_t,
) |  |  |  |  |

| `duckdb_scalar_function_set_name` | function | duckdb_scalar_function_set_name(
    scalar_function: duckdb_scalar_function,
    name: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_scalar_function_set_return_type` | function | duckdb_scalar_function_set_return_type(
    scalar_function: duckdb_scalar_function,
    type_: duckdb_logical_type,
) |  |  |  |  |

| `duckdb_scalar_function_set_special_handling` | function | duckdb_scalar_function_set_special_handling(
    scalar_function: duckdb_scalar_function,
) |  |  |  |  |

| `duckdb_scalar_function_set_varargs` | function | duckdb_scalar_function_set_varargs(
    scalar_function: duckdb_scalar_function,
    type_: duckdb_logical_type,
) |  |  |  |  |

| `duckdb_scalar_function_set_volatile` | function | duckdb_scalar_function_set_volatile(
    scalar_function: duckdb_scalar_function,
) |  |  |  |  |

| `duckdb_schema_from_arrow` | function | duckdb_schema_from_arrow(
    connection: duckdb_connection,
    schema: *mut ArrowSchema,
    out_types: *mut duckdb_arrow_converted_schema,
) |  |  |  |  |

| `duckdb_selection_vector_get_data_ptr` | function | duckdb_selection_vector_get_data_ptr(
    sel: duckdb_selection_vector,
) |  |  |  |  |

| `duckdb_set_config` | function | duckdb_set_config(
    config: duckdb_config,
    name: *const ::std::os::raw::c_char,
    option: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_slice_vector` | function | duckdb_slice_vector(
    vector: duckdb_vector,
    sel: duckdb_selection_vector,
    len: idx_t,
) |  |  |  |  |

| `duckdb_stream_fetch_chunk` | function | duckdb_stream_fetch_chunk(result: duckdb_result) |  |  |  |  |

| `duckdb_string_is_inlined` | function | duckdb_string_is_inlined(string: duckdb_string_t) |  |  |  |  |

| `duckdb_string_t_data` | function | duckdb_string_t_data(
    string: *mut duckdb_string_t,
) |  |  |  |  |

| `duckdb_string_t_length` | function | duckdb_string_t_length(string: duckdb_string_t) |  |  |  |  |

| `duckdb_struct_type_child_count` | function | duckdb_struct_type_child_count(type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_struct_type_child_name` | function | duckdb_struct_type_child_name(
    type_: duckdb_logical_type,
    index: idx_t,
) |  |  |  |  |

| `duckdb_struct_type_child_type` | function | duckdb_struct_type_child_type(
    type_: duckdb_logical_type,
    index: idx_t,
) |  |  |  |  |

| `duckdb_struct_vector_get_child` | function | duckdb_struct_vector_get_child(
    vector: duckdb_vector,
    index: idx_t,
) |  |  |  |  |

| `duckdb_table_description_create` | function | duckdb_table_description_create(
    connection: duckdb_connection,
    schema: *const ::std::os::raw::c_char,
    table: *const ::std::os::raw::c_char,
    out: *mut duckdb_table_description,
) |  |  |  |  |

| `duckdb_table_description_create_ext` | function | duckdb_table_description_create_ext(
    connection: duckdb_connection,
    catalog: *const ::std::os::raw::c_char,
    schema: *const ::std::os::raw::c_char,
    table: *const ::std::os::raw::c_char,
    out: *mut duckdb_table_description,
) |  |  |  |  |

| `duckdb_table_description_destroy` | function | duckdb_table_description_destroy(
    table_description: *mut duckdb_table_description,
) |  |  |  |  |

| `duckdb_table_description_error` | function | duckdb_table_description_error(
    table_description: duckdb_table_description,
) |  |  |  |  |

| `duckdb_table_description_get_column_name` | function | duckdb_table_description_get_column_name(
    table_description: duckdb_table_description,
    index: idx_t,
) |  |  |  |  |

| `duckdb_table_function_add_named_parameter` | function | duckdb_table_function_add_named_parameter(
    table_function: duckdb_table_function,
    name: *const ::std::os::raw::c_char,
    type_: duckdb_logical_type,
) |  |  |  |  |

| `duckdb_table_function_add_parameter` | function | duckdb_table_function_add_parameter(
    table_function: duckdb_table_function,
    type_: duckdb_logical_type,
) |  |  |  |  |

| `duckdb_table_function_get_client_context` | function | duckdb_table_function_get_client_context(
    info: duckdb_bind_info,
    out_context: *mut duckdb_client_context,
) |  |  |  |  |

| `duckdb_table_function_set_bind` | function | duckdb_table_function_set_bind(
    table_function: duckdb_table_function,
    bind: duckdb_table_function_bind_t,
) |  |  |  |  |

| `duckdb_table_function_set_extra_info` | function | duckdb_table_function_set_extra_info(
    table_function: duckdb_table_function,
    extra_info: *mut ::std::os::raw::c_void,
    destroy: duckdb_delete_callback_t,
) |  |  |  |  |

| `duckdb_table_function_set_function` | function | duckdb_table_function_set_function(
    table_function: duckdb_table_function,
    function: duckdb_table_function_t,
) |  |  |  |  |

| `duckdb_table_function_set_init` | function | duckdb_table_function_set_init(
    table_function: duckdb_table_function,
    init: duckdb_table_function_init_t,
) |  |  |  |  |

| `duckdb_table_function_set_local_init` | function | duckdb_table_function_set_local_init(
    table_function: duckdb_table_function,
    init: duckdb_table_function_init_t,
) |  |  |  |  |

| `duckdb_table_function_set_name` | function | duckdb_table_function_set_name(
    table_function: duckdb_table_function,
    name: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_table_function_supports_projection_pushdown` | function | duckdb_table_function_supports_projection_pushdown(
    table_function: duckdb_table_function,
    pushdown: bool,
) |  |  |  |  |

| `duckdb_task_state_is_finished` | function | duckdb_task_state_is_finished(state: duckdb_task_state) |  |  |  |  |

| `duckdb_to_arrow_schema` | function | duckdb_to_arrow_schema(
    arrow_options: duckdb_arrow_options,
    types: *mut duckdb_logical_type,
    names: *mut *const ::std::os::raw::c_char,
    column_count: idx_t,
    out_schema: *mut ArrowSchema,
) |  |  |  |  |

| `duckdb_to_date` | function | duckdb_to_date(date: duckdb_date_struct) |  |  |  |  |

| `duckdb_to_time` | function | duckdb_to_time(time: duckdb_time_struct) |  |  |  |  |

| `duckdb_to_timestamp` | function | duckdb_to_timestamp(ts: duckdb_timestamp_struct) |  |  |  |  |

| `duckdb_uhugeint_to_double` | function | duckdb_uhugeint_to_double(val: duckdb_uhugeint) |  |  |  |  |

| `duckdb_union_type_member_count` | function | duckdb_union_type_member_count(type_: duckdb_logical_type) |  |  |  |  |

| `duckdb_union_type_member_name` | function | duckdb_union_type_member_name(
    type_: duckdb_logical_type,
    index: idx_t,
) |  |  |  |  |

| `duckdb_union_type_member_type` | function | duckdb_union_type_member_type(
    type_: duckdb_logical_type,
    index: idx_t,
) |  |  |  |  |

| `duckdb_validity_row_is_valid` | function | duckdb_validity_row_is_valid(validity: *mut u64, row: idx_t) |  |  |  |  |

| `duckdb_validity_set_row_invalid` | function | duckdb_validity_set_row_invalid(validity: *mut u64, row: idx_t) |  |  |  |  |

| `duckdb_validity_set_row_valid` | function | duckdb_validity_set_row_valid(validity: *mut u64, row: idx_t) |  |  |  |  |

| `duckdb_validity_set_row_validity` | function | duckdb_validity_set_row_validity(
    validity: *mut u64,
    row: idx_t,
    valid: bool,
) |  |  |  |  |

| `duckdb_value_blob` | function | duckdb_value_blob(
    result: *mut duckdb_result,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_value_boolean` | function | duckdb_value_boolean(
    result: *mut duckdb_result,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_value_date` | function | duckdb_value_date(
    result: *mut duckdb_result,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_value_decimal` | function | duckdb_value_decimal(
    result: *mut duckdb_result,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_value_double` | function | duckdb_value_double(
    result: *mut duckdb_result,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_value_float` | function | duckdb_value_float(
    result: *mut duckdb_result,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_value_hugeint` | function | duckdb_value_hugeint(
    result: *mut duckdb_result,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_value_int16` | function | duckdb_value_int16(
    result: *mut duckdb_result,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_value_int32` | function | duckdb_value_int32(
    result: *mut duckdb_result,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_value_int64` | function | duckdb_value_int64(
    result: *mut duckdb_result,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_value_int8` | function | duckdb_value_int8(
    result: *mut duckdb_result,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_value_interval` | function | duckdb_value_interval(
    result: *mut duckdb_result,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_value_is_null` | function | duckdb_value_is_null(
    result: *mut duckdb_result,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_value_string` | function | duckdb_value_string(
    result: *mut duckdb_result,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_value_string_internal` | function | duckdb_value_string_internal(
    result: *mut duckdb_result,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_value_time` | function | duckdb_value_time(
    result: *mut duckdb_result,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_value_timestamp` | function | duckdb_value_timestamp(
    result: *mut duckdb_result,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_value_to_string` | function | duckdb_value_to_string(
    value: duckdb_value,
) |  |  |  |  |

| `duckdb_value_uhugeint` | function | duckdb_value_uhugeint(
    result: *mut duckdb_result,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_value_uint16` | function | duckdb_value_uint16(
    result: *mut duckdb_result,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_value_uint32` | function | duckdb_value_uint32(
    result: *mut duckdb_result,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_value_uint64` | function | duckdb_value_uint64(
    result: *mut duckdb_result,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_value_uint8` | function | duckdb_value_uint8(
    result: *mut duckdb_result,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_value_varchar` | function | duckdb_value_varchar(
    result: *mut duckdb_result,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_value_varchar_internal` | function | duckdb_value_varchar_internal(
    result: *mut duckdb_result,
    col: idx_t,
    row: idx_t,
) |  |  |  |  |

| `duckdb_vector_assign_string_element` | function | duckdb_vector_assign_string_element(
    vector: duckdb_vector,
    index: idx_t,
    str_: *const ::std::os::raw::c_char,
) |  |  |  |  |

| `duckdb_vector_assign_string_element_len` | function | duckdb_vector_assign_string_element_len(
    vector: duckdb_vector,
    index: idx_t,
    str_: *const ::std::os::raw::c_char,
    str_len: idx_t,
) |  |  |  |  |

| `duckdb_vector_copy_sel` | function | duckdb_vector_copy_sel(
    src: duckdb_vector,
    dst: duckdb_vector,
    sel: duckdb_selection_vector,
    src_count: idx_t,
    src_offset: idx_t,
    dst_offset: idx_t,
) |  |  |  |  |

| `duckdb_vector_ensure_validity_writable` | function | duckdb_vector_ensure_validity_writable(vector: duckdb_vector) |  |  |  |  |

| `duckdb_vector_get_column_type` | function | duckdb_vector_get_column_type(
    vector: duckdb_vector,
) |  |  |  |  |

| `duckdb_vector_get_data` | function | duckdb_vector_get_data(
    vector: duckdb_vector,
) |  |  |  |  |

| `duckdb_vector_get_validity` | function | duckdb_vector_get_validity(vector: duckdb_vector) |  |  |  |  |

| `duckdb_vector_reference_value` | function | duckdb_vector_reference_value(vector: duckdb_vector, value: duckdb_value) |  |  |  |  |

| `duckdb_vector_reference_vector` | function | duckdb_vector_reference_vector(
    to_vector: duckdb_vector,
    from_vector: duckdb_vector,
) |  |  |  |  |

| `duckdb_vector_size` | function | duckdb_vector_size() |  |  |  |  |

| `ArrowArray` | struct |  |  |  |  |  |

| `ArrowSchema` | struct |  |  |  |  |  |

| `ErrorCode` | enum |  |  |  |  |  |

| `code_to_str` | function | code_to_str(_: duckdb_state) |  |  |  |  |

| `new` | function | new(result_code: duckdb_state) |  |  |  |  |

| `Error` | struct |  |  |  |  |  |

| `from_ptr` | function | from_ptr(ptr: *const c_char) |  |  |  |  |

| `from_raw_parts` | function | from_raw_parts(ptr: *const c_char, len: usize) |  |  |  |  |

| `DuckDbString` | struct |  |  |  |  |  |

| `detect` | function | detect() |  |  |  |  |

| `get_binary_name` | function | get_binary_name() |  |  |  |  |

| `is_supported` | function | is_supported(arch: &str) |  |  |  |  |

| `Commands` | enum |  |  |  |  |  |

| `Cli` | struct |  |  |  |  |  |

| `convert_dataset` | function | convert_dataset(
        &self,
        input: &str,
        output: &str,
        input_format: &str,
        output_format: &str,
    ) |  |  |  |  |

| `download_chinook` | function | download_chinook(&self, output_dir: &str, format: &str) |  |  |  |  |

| `download_tpch` | function | download_tpch(&self, output_dir: &str, format: &str) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `show_info` | function | show_info(&self) |  |  |  |  |

| `DatasetManager` | struct |  |  |  |  |  |

| `all_passed` | function | all_passed(&self) |  |  |  |  |

| `complete_text` | function | complete_text(&self, prompt: &str, model: &str) |  |  |  |  |

| `format_results` | function | format_results(&self) |  |  |  |  |

| `generate_embeddings` | function | generate_embeddings(
        &self,
        texts: Vec<String>,
        model: &str,
        normalize: bool,
    ) |  |  |  |  |

| `is_flock_ready` | function | is_flock_ready(&self) |  |  |  |  |

| `llm_filter` | function | llm_filter(
        &self,
        criteria: &str,
        input_file: &str,
        model: &str,
        positive_only: bool,
    ) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `semantic_search` | function | semantic_search(
        &self,
        query: &str,
        _corpus: &str,
        _threshold: f32,
        _limit: usize,
    ) |  |  |  |  |

| `setup_ollama` | function | setup_ollama(
        &self,
        ollama_url: &str,
        text_model: &str,
        embedding_model: &str,
        skip_verification: bool,
    ) |  |  |  |  |

| `success_rate` | function | success_rate(&self) |  |  |  |  |

| `summarize_texts` | function | summarize_texts(
        &self,
        texts: Vec<String>,
        strategy: &str,
        max_length: usize,
        model: &str,
    ) |  |  |  |  |

| `validate_ffi` | function | validate_ffi(&self) |  |  |  |  |

| `FFIValidationResult` | struct |  |  |  |  |  |

| `FlockManager` | struct |  |  |  |  |  |

| `ValidationLayerResult` | struct |  |  |  |  |  |

| `append_record_batch` | function | append_record_batch(&mut self, record_batch: RecordBatch) |  |  |  |  |

| `flush` | function | flush(&mut self) |  |  |  |  |

| `Appender` | struct |  |  |  |  |  |

| `AppenderParamsFromIter` | struct |  |  |  |  |  |

| `AppenderParams` | trait |  |  |  |  |  |

| `Sealed` | trait |  |  |  |  |  |

| `get_schema` | function | get_schema(&self) |  |  |  |  |

| `Arrow` | struct |  |  |  |  |  |

| `ArrowStream` | struct |  |  |  |  |  |

| `discard` | function | discard(mut self) |  |  |  |  |

| `flush_prepared_statement_cache` | function | flush_prepared_statement_cache(&self) |  |  |  |  |

| `prepare_cached` | function | prepare_cached(&self, sql: &str) |  |  |  |  |

| `set_prepared_statement_cache_capacity` | function | set_prepared_statement_cache_capacity(&self, capacity: usize) |  |  |  |  |

| `with_capacity` | function | with_capacity(capacity: usize) |  |  |  |  |

| `CachedStatement` | struct |  |  |  |  |  |

| `StatementCache` | struct |  |  |  |  |  |

| `column_count` | function | column_count(&self) |  |  |  |  |

| `column_index` | function | column_index(&self, name: &str) |  |  |  |  |

| `column_name` | function | column_name(&self, col: usize) |  |  |  |  |

| `column_names` | function | column_names(&self) |  |  |  |  |

| `column_type` | function | column_type(&self, idx: usize) |  |  |  |  |

| `decl_type` | function | decl_type(&self) |  |  |  |  |

| `name` | function | name(&self) |  |  |  |  |

| `Column` | struct |  |  |  |  |  |

| `AccessMode` | enum |  |  |  |  |  |

| `DefaultNullOrder` | enum |  |  |  |  |  |

| `DefaultOrder` | enum |  |  |  |  |  |

| `access_mode` | function | access_mode(mut self, mode: AccessMode) |  |  |  |  |

| `allow_unsigned_extensions` | function | allow_unsigned_extensions(mut self) |  |  |  |  |

| `custom_user_agent` | function | custom_user_agent(mut self, custom_user_agent: &str) |  |  |  |  |

| `default_null_order` | function | default_null_order(mut self, null_order: DefaultNullOrder) |  |  |  |  |

| `default_order` | function | default_order(mut self, order: DefaultOrder) |  |  |  |  |

| `enable_autoload_extension` | function | enable_autoload_extension(mut self, enabled: bool) |  |  |  |  |

| `enable_external_access` | function | enable_external_access(mut self, enabled: bool) |  |  |  |  |

| `enable_object_cache` | function | enable_object_cache(mut self, enabled: bool) |  |  |  |  |

| `max_memory` | function | max_memory(mut self, memory: &str) |  |  |  |  |

| `threads` | function | threads(mut self, thread_num: i64) |  |  |  |  |

| `with` | function | with(mut self, key: impl AsRef<str>, value: impl AsRef<str>) |  |  |  |  |

| `Config` | struct |  |  |  |  |  |

| `array_vector` | function | array_vector(&self, idx: usize) |  |  |  |  |

| `flat_vector` | function | flat_vector(&self, idx: usize) |  |  |  |  |

| `get_ptr` | function | get_ptr(&self) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `list_vector` | function | list_vector(&self, idx: usize) |  |  |  |  |

| `new` | function | new(logical_types: &[LogicalTypeHandle]) |  |  |  |  |

| `num_columns` | function | num_columns(&self) |  |  |  |  |

| `set_len` | function | set_len(&self, new_len: usize) |  |  |  |  |

| `struct_vector` | function | struct_vector(&self, idx: usize) |  |  |  |  |

| `DataChunkHandle` | struct |  |  |  |  |  |

| `LogicalTypeId` | enum |  |  |  |  |  |

| `array` | function | array(child_type: &Self, array_size: u64) |  |  |  |  |

| `child` | function | child(&self, idx: usize) |  |  |  |  |

| `child_name` | function | child_name(&self, idx: usize) |  |  |  |  |

| `decimal` | function | decimal(width: u8, scale: u8) |  |  |  |  |

| `decimal_scale` | function | decimal_scale(&self) |  |  |  |  |

| `decimal_width` | function | decimal_width(&self) |  |  |  |  |

| `id` | function | id(&self) |  |  |  |  |

| `list` | function | list(child_type: &Self) |  |  |  |  |

| `map` | function | map(key: &Self, value: &Self) |  |  |  |  |

| `num_children` | function | num_children(&self) |  |  |  |  |

| `struct_type` | function | struct_type(fields: &[(&str, Self) |  |  |  |  |

| `union_type` | function | union_type(fields: &[(&str, Self) |  |  |  |  |

| `LogicalTypeHandle` | struct |  |  |  |  |  |

| `array_child` | function | array_child(&self) |  |  |  |  |

| `array_vector_child` | function | array_vector_child(&self, idx: usize) |  |  |  |  |

| `capacity` | function | capacity(&self) |  |  |  |  |

| `child` | function | child(&self, capacity: usize) |  |  |  |  |

| `child_name` | function | child_name(&self, idx: usize) |  |  |  |  |

| `get_array_size` | function | get_array_size(&self) |  |  |  |  |

| `is_empty` | function | is_empty(&self) |  |  |  |  |

| `len` | function | len(&self) |  |  |  |  |

| `list_child` | function | list_child(&self) |  |  |  |  |

| `list_vector_child` | function | list_vector_child(&self, idx: usize) |  |  |  |  |

| `logical_type` | function | logical_type(&self) |  |  |  |  |

| `num_children` | function | num_children(&self) |  |  |  |  |

| `row_is_null` | function | row_is_null(&self, row: u64) |  |  |  |  |

| `set_entry` | function | set_entry(&mut self, idx: usize, offset: usize, length: usize) |  |  |  |  |

| `set_len` | function | set_len(&self, new_len: usize) |  |  |  |  |

| `set_null` | function | set_null(&mut self, row: usize) |  |  |  |  |

| `struct_child` | function | struct_child(&self, capacity: usize) |  |  |  |  |

| `struct_vector_child` | function | struct_vector_child(&self, idx: usize) |  |  |  |  |

| `ArrayVector` | struct |  |  |  |  |  |

| `FlatVector` | struct |  |  |  |  |  |

| `ListVector` | struct |  |  |  |  |  |

| `StructVector` | struct |  |  |  |  |  |

| `Inserter` | trait |  |  |  |  |  |

| `Vector` | trait |  |  |  |  |  |

| `Error` | enum |  |  |  |  |  |

| `result_from_duckdb_appender` | function | result_from_duckdb_appender(
    code: ffi::duckdb_state,
    appender: *mut ffi::duckdb_appender,
) |  |  |  |  |

| `result_from_duckdb_arrow` | function | result_from_duckdb_arrow(code: ffi::duckdb_state, mut out: ffi::duckdb_arrow) |  |  |  |  |

| `result_from_duckdb_extract` | function | result_from_duckdb_extract(
    num_statements: ffi::idx_t,
    mut extracted: ffi::duckdb_extracted_statements,
) |  |  |  |  |

| `result_from_duckdb_prepare` | function | result_from_duckdb_prepare(
    code: ffi::duckdb_state,
    mut prepare: ffi::duckdb_prepared_statement,
) |  |  |  |  |

| `close` | function | close(&mut self) |  |  |  |  |

| `execute` | function | execute(&mut self, sql: &str) |  |  |  |  |

| `get_interrupt_handle` | function | get_interrupt_handle(&self) |  |  |  |  |

| `interrupt` | function | interrupt(&self) |  |  |  |  |

| `is_autocommit` | function | is_autocommit(&self) |  |  |  |  |

| `new` | function | new(db: ffi::duckdb_database, owned: bool) |  |  |  |  |

| `open_with_flags` | function | open_with_flags(c_path: &CStr, config: Config) |  |  |  |  |

| `try_clone` | function | try_clone(&self) |  |  |  |  |

| `InnerConnection` | struct |  |  |  |  |  |

| `InterruptHandle` | struct |  |  |  |  |  |

| `DatabaseName` | enum |  |  |  |  |  |

| `appender` | function | appender(&self, table: &str) |  |  |  |  |

| `appender_to_db` | function | appender_to_db(&self, table: &str, schema: &str) |  |  |  |  |

| `checked_memory_handle` | function | checked_memory_handle() |  |  |  |  |

| `close` | function | close(self) |  |  |  |  |

| `execute_batch` | function | execute_batch(&self, sql: &str) |  |  |  |  |

| `interrupt_handle` | function | interrupt_handle(&self) |  |  |  |  |

| `is_autocommit` | function | is_autocommit(&self) |  |  |  |  |

| `open_from_raw` | function | open_from_raw(raw: ffi::duckdb_database) |  |  |  |  |

| `open_in_memory` | function | open_in_memory() |  |  |  |  |

| `open_in_memory_with_flags` | function | open_in_memory_with_flags(config: Config) |  |  |  |  |

| `path` | function | path(&self) |  |  |  |  |

| `prepare` | function | prepare(&self, sql: &str) |  |  |  |  |

| `try_clone` | function | try_clone(&self) |  |  |  |  |

| `version` | function | version(&self) |  |  |  |  |

| `Connection` | struct |  |  |  |  |  |

| `OptionalExt` | trait |  |  |  |  |  |

| `ParamsFromIter` | struct |  |  |  |  |  |

| `Params` | trait |  |  |  |  |  |

| `Sealed` | trait |  |  |  |  |  |

| `get_schema` | function | get_schema(&self) |  |  |  |  |

| `Polars` | struct |  |  |  |  |  |

| `as_str` | function | as_str(&self) |  |  |  |  |

| `close_brace` | function | close_brace(&mut self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `open_brace` | function | open_brace(&mut self) |  |  |  |  |

| `pragma_update` | function | pragma_update(
        &self,
        schema_name: Option<DatabaseName<'_>>,
        pragma_name: &str,
        pragma_value: &dyn ToSql,
    ) |  |  |  |  |

| `push_dot` | function | push_dot(&mut self) |  |  |  |  |

| `push_equal_sign` | function | push_equal_sign(&mut self) |  |  |  |  |

| `push_identifier` | function | push_identifier(&mut self, s: &str) |  |  |  |  |

| `push_int` | function | push_int(&mut self, i: i64) |  |  |  |  |

| `push_keyword` | function | push_keyword(&mut self, keyword: &str) |  |  |  |  |

| `push_pragma` | function | push_pragma(
        &mut self,
        schema_name: Option<DatabaseName<'_>>,
        pragma_name: &str,
    ) |  |  |  |  |

| `push_real` | function | push_real(&mut self, f: f64) |  |  |  |  |

| `push_schema_name` | function | push_schema_name(&mut self, schema_name: DatabaseName<'_>) |  |  |  |  |

| `push_space` | function | push_space(&mut self) |  |  |  |  |

| `push_string_literal` | function | push_string_literal(&mut self, s: &str) |  |  |  |  |

| `push_value` | function | push_value(&mut self, value: &dyn ToSql) |  |  |  |  |

| `Sql` | struct |  |  |  |  |  |

| `memory` | function | memory() |  |  |  |  |

| `memory_with_flags` | function | memory_with_flags(config: Config) |  |  |  |  |

| `DuckdbConnectionManager` | struct |  |  |  |  |  |

| `bind_parameter_count` | function | bind_parameter_count(&self) |  |  |  |  |

| `clear_bindings` | function | clear_bindings(&self) |  |  |  |  |

| `column_count` | function | column_count(&self) |  |  |  |  |

| `column_name` | function | column_name(&self, idx: usize) |  |  |  |  |

| `column_type` | function | column_type(&self, idx: usize) |  |  |  |  |

| `execute` | function | execute(&mut self) |  |  |  |  |

| `execute_streaming` | function | execute_streaming(&mut self) |  |  |  |  |

| `is_null` | function | is_null(&self) |  |  |  |  |

| `new` | function | new(stmt: ffi::duckdb_prepared_statement) |  |  |  |  |

| `ptr` | function | ptr(&self) |  |  |  |  |

| `reset_result` | function | reset_result(&mut self) |  |  |  |  |

| `result_unwrap` | function | result_unwrap(&self) |  |  |  |  |

| `row_count` | function | row_count(&self) |  |  |  |  |

| `schema` | function | schema(&self) |  |  |  |  |

| `sql` | function | sql(&self) |  |  |  |  |

| `step` | function | step(&self) |  |  |  |  |

| `step2` | function | step2(&self) |  |  |  |  |

| `streaming_step` | function | streaming_step(&self, schema: SchemaRef) |  |  |  |  |

| `RawStatement` | struct |  |  |  |  |  |

| `as_ref` | function | as_ref(&self) |  |  |  |  |

| `next` | function | next(&mut self) |  |  |  |  |

| `AndThenRows` | struct |  |  |  |  |  |

| `Map` | struct |  |  |  |  |  |

| `MappedRows` | struct |  |  |  |  |  |

| `Row` | struct |  |  |  |  |  |

| `Rows` | struct |  |  |  |  |  |

| `RowIndex` | trait |  |  |  |  |  |

| `Sealed` | trait |  |  |  |  |  |

| `parameter_count` | function | parameter_count(&self) |  |  |  |  |

| `raw_execute` | function | raw_execute(&mut self) |  |  |  |  |

| `raw_query` | function | raw_query(&self) |  |  |  |  |

| `row_count` | function | row_count(&self) |  |  |  |  |

| `schema` | function | schema(&self) |  |  |  |  |

| `step` | function | step(&self) |  |  |  |  |

| `step2` | function | step2(&self) |  |  |  |  |

| `stream_step` | function | stream_step(&self, schema: SchemaRef) |  |  |  |  |

| `Statement` | struct |  |  |  |  |  |

| `DropBehavior` | enum |  |  |  |  |  |

| `commit` | function | commit(mut self) |  |  |  |  |

| `drop_behavior` | function | drop_behavior(&self) |  |  |  |  |

| `finish` | function | finish(mut self) |  |  |  |  |

| `new` | function | new(conn: &mut Connection) |  |  |  |  |

| `new_unchecked` | function | new_unchecked(conn: &Connection) |  |  |  |  |

| `rollback` | function | rollback(mut self) |  |  |  |  |

| `set_drop_behavior` | function | set_drop_behavior(&mut self, drop_behavior: DropBehavior) |  |  |  |  |

| `transaction` | function | transaction(&mut self) |  |  |  |  |

| `unchecked_transaction` | function | unchecked_transaction(&self) |  |  |  |  |

| `Transaction` | struct |  |  |  |  |  |

| `FromSqlError` | enum |  |  |  |  |  |

| `FromSql` | trait |  |  |  |  |  |

| `Type` | enum |  |  |  |  |  |

| `Null` | struct |  |  |  |  |  |

| `get` | function | get(&self, key: &K) |  |  |  |  |

| `iter` | function | iter(&self) |  |  |  |  |

| `keys` | function | keys(&self) |  |  |  |  |

| `values` | function | values(&self) |  |  |  |  |

| `OrderedMap` | struct |  |  |  |  |  |

| `as_bytes` | function | as_bytes(&mut self) |  |  |  |  |

| `as_str` | function | as_str(&mut self) |  |  |  |  |

| `new` | function | new(ptr: &'a mut duckdb_string_t) |  |  |  |  |

| `DuckString` | struct |  |  |  |  |  |

| `ToSqlOutput` | enum |  |  |  |  |  |

| `ToSql` | trait |  |  |  |  |  |

| `Value` | enum |  |  |  |  |  |

| `data_type` | function | data_type(&self) |  |  |  |  |

| `EnumType` | enum |  |  |  |  |  |

| `ListType` | enum |  |  |  |  |  |

| `TimeUnit` | enum |  |  |  |  |  |

| `ValueRef` | enum |  |  |  |  |  |

| `as_blob` | function | as_blob(&self) |  |  |  |  |

| `as_str` | function | as_str(&self) |  |  |  |  |

| `data_type` | function | data_type(&self) |  |  |  |  |

| `to_micros` | function | to_micros(&self, value: i64) |  |  |  |  |

| `to_owned` | function | to_owned(&self) |  |  |  |  |

| `ArrowScalarParams` | enum |  |  |  |  |  |

| `exact` | function | exact(params: Vec<DataType>, return_type: DataType) |  |  |  |  |

| `variadic` | function | variadic(param: DataType, return_type: DataType) |  |  |  |  |

| `ArrowFunctionSignature` | struct |  |  |  |  |  |

| `VArrowScalar` | trait |  |  |  |  |  |

| `add_function` | function | add_function(&self, func: ScalarFunction) |  |  |  |  |

| `add_parameter` | function | add_parameter(&self, logical_type: &LogicalTypeHandle) |  |  |  |  |

| `add_variadic_parameter` | function | add_variadic_parameter(&self, logical_type: &LogicalTypeHandle) |  |  |  |  |

| `new` | function | new(name: &str) |  |  |  |  |

| `set_function` | function | set_function(
        &self,
        func: Option<
            unsafe extern "C" fn(
                info: duckdb_function_info,
                input: duckdb_data_chunk,
                output: duckdb_vector,
            ) |  |  |  |  |

| `set_return_type` | function | set_return_type(&self, logical_type: &LogicalTypeHandle) |  |  |  |  |

| `ScalarFunction` | struct |  |  |  |  |  |

| `ScalarFunctionSet` | struct |  |  |  |  |  |

| `ScalarParams` | enum |  |  |  |  |  |

| `exact` | function | exact(params: Vec<LogicalTypeHandle>, return_type: LogicalTypeHandle) |  |  |  |  |

| `register_scalar_function_set` | function | register_scalar_function_set(
        &mut self,
        f: ScalarFunctionSet,
    ) |  |  |  |  |

| `set_error` | function | set_error(&self, error: &str) |  |  |  |  |

| `variadic` | function | variadic(param: LogicalTypeHandle, return_type: LogicalTypeHandle) |  |  |  |  |

| `ScalarFunctionSignature` | struct |  |  |  |  |  |

| `VScalar` | trait |  |  |  |  |  |

| `arrow_arraydata_to_query_params` | function | arrow_arraydata_to_query_params(data: ArrayData) |  |  |  |  |

| `arrow_ffi_to_query_params` | function | arrow_ffi_to_query_params(array: FFI_ArrowArray, schema: FFI_ArrowSchema) |  |  |  |  |

| `arrow_recordbatch_to_query_params` | function | arrow_recordbatch_to_query_params(rb: RecordBatch) |  |  |  |  |

| `data_chunk_to_arrow` | function | data_chunk_to_arrow(
    chunk: &DataChunkHandle,
) |  |  |  |  |

| `flat_vector_to_arrow_array` | function | flat_vector_to_arrow_array(
    vector: &mut FlatVector,
    len: usize,
) |  |  |  |  |

| `record_batch_to_duckdb_data_chunk` | function | record_batch_to_duckdb_data_chunk(
    batch: &RecordBatch,
    chunk: &mut DataChunkHandle,
) |  |  |  |  |

| `to_duckdb_logical_type` | function | to_duckdb_logical_type(
    data_type: &DataType,
) |  |  |  |  |

| `to_duckdb_type_id` | function | to_duckdb_type_id(
    data_type: &DataType,
) |  |  |  |  |

| `write_arrow_array_to_vector` | function | write_arrow_array_to_vector(
    col: &Arc<dyn Array>,
    chunk: &mut dyn WritableVector,
) |  |  |  |  |

| `ArrowBindData` | struct |  |  |  |  |  |

| `ArrowInitData` | struct |  |  |  |  |  |

| `ArrowVTab` | struct |  |  |  |  |  |

| `WritableVector` | trait |  |  |  |  |  |

| `add_named_parameter` | function | add_named_parameter(&self, name: &str, logical_type: &LogicalTypeHandle) |  |  |  |  |

| `add_parameter` | function | add_parameter(&self, logical_type: &LogicalTypeHandle) |  |  |  |  |

| `add_result_column` | function | add_result_column(&self, column_name: &str, column_type: LogicalTypeHandle) |  |  |  |  |

| `get_bind_data` | function | get_bind_data(&self) |  |  |  |  |

| `get_column_indices` | function | get_column_indices(&self) |  |  |  |  |

| `get_init_data` | function | get_init_data(&self) |  |  |  |  |

| `get_named_parameter` | function | get_named_parameter(&self, name: &str) |  |  |  |  |

| `get_parameter` | function | get_parameter(&self, param_index: u64) |  |  |  |  |

| `get_parameter_count` | function | get_parameter_count(&self) |  |  |  |  |

| `new` | function | new() |  |  |  |  |

| `set_bind` | function | set_bind(&self, bind_func: Option<unsafe extern "C" fn(duckdb_bind_info) |  |  |  |  |

| `set_bind_data` | function | set_bind_data(
        &self,
        data: *mut c_void,
        free_function: Option<unsafe extern "C" fn(*mut c_void) |  |  |  |  |

| `set_cardinality` | function | set_cardinality(&self, cardinality: idx_t, is_exact: bool) |  |  |  |  |

| `set_error` | function | set_error(&self, error: &str) |  |  |  |  |

| `set_extra_info` | function | set_extra_info(
        &self,
        extra_info: *mut c_void,
        destroy: duckdb_delete_callback_t,
    ) |  |  |  |  |

| `set_function` | function | set_function(
        &self,
        func: Option<unsafe extern "C" fn(info: duckdb_function_info, output: duckdb_data_chunk) |  |  |  |  |

| `set_init` | function | set_init(&self, init_func: Option<unsafe extern "C" fn(duckdb_init_info) |  |  |  |  |

| `set_init_data` | function | set_init_data(
        &self,
        data: *mut c_void,
        freeer: Option<unsafe extern "C" fn(*mut c_void) |  |  |  |  |

| `set_local_init` | function | set_local_init(&self, init: duckdb_table_function_init_t) |  |  |  |  |

| `set_max_threads` | function | set_max_threads(&self, max_threads: idx_t) |  |  |  |  |

| `set_name` | function | set_name(&self, name: &str) |  |  |  |  |

| `supports_pushdown` | function | supports_pushdown(&self, supports: bool) |  |  |  |  |

| `BindInfo` | struct |  |  |  |  |  |

| `InitInfo` | struct |  |  |  |  |  |

| `TableFunction` | struct |  |  |  |  |  |

| `TableFunctionInfo` | struct |  |  |  |  |  |

| `register_table_function` | function | register_table_function(&mut self, table_function: TableFunction) |  |  |  |  |

| `VTab` | trait |  |  |  |  |  |

| `to_int64` | function | to_int64(&self) |  |  |  |  |

| `Value` | struct |  |  |  |  |  |

| `get_include_dir` | function | get_include_dir() |  |  |  |  |

| `get_lib_dir` | function | get_lib_dir() |  |  |  |  |

| `is_configured` | function | is_configured() |  |  |  |  |

| `validate_binary` | function | validate_binary() |  |  |  |  |


<!-- ============================================================= -->
<!-- AGENT-FORBIDDEN-END: nothing below this line may describe     -->
<!-- code behavior.                                                -->
<!-- ============================================================= -->
