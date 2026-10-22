#  reference

<!-- ============================================================= -->
<!-- AGENT-FORBIDDEN-BEGIN: reference body is RIGID                -->
<!-- Every row below is rendered from queries/ast_extract.rq.      -->
<!-- Agents MUST NOT add, edit, reorder, or remove any row or      -->
<!-- table cell. Prose outside the fenced slot below is refused    -->
<!-- by the doc_quality court.                                     -->
<!-- ============================================================= -->

## Modules


### crates/frozen-duckdb/src/duckdb/core/vector.rs

| `array_child` | function | array_child(&self) -> ArrayVector |  |  |  |  |

| `array_vector_child` | function | array_vector_child(&self, idx: usize) -> ArrayVector |  |  |  |  |

| `as_mut_ptr` | function | as_mut_ptr(&self) -> *mut T |  |  |  |  |

| `as_mut_slice` | function | as_mut_slice(&mut self) -> &mut [T] |  |  |  |  |

| `as_mut_slice_with_len` | function | as_mut_slice_with_len(&mut self, len: usize) -> &mut [T] |  |  |  |  |

| `as_slice` | function | as_slice(&self) -> &[T] |  |  |  |  |

| `as_slice_with_len` | function | as_slice_with_len(&self, len: usize) -> &[T] |  |  |  |  |

| `capacity` | function | capacity(&self) -> usize |  |  |  |  |

| `child` | function | child(&self, capacity: usize) -> FlatVector |  |  |  |  |

| `child_name` | function | child_name(&self, idx: usize) -> DuckDbString |  |  |  |  |

| `copy` | function | copy(&mut self, data: &[T]) |  |  |  |  |

| `get_array_size` | function | get_array_size(&self) -> u64 |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `len` | function | len(&self) -> usize |  |  |  |  |

| `list_child` | function | list_child(&self) -> Self |  |  |  |  |

| `list_vector_child` | function | list_vector_child(&self, idx: usize) -> ListVector |  |  |  |  |

| `logical_type` | function | logical_type(&self) -> LogicalTypeHandle |  |  |  |  |

| `num_children` | function | num_children(&self) -> usize |  |  |  |  |

| `row_is_null` | function | row_is_null(&self, row: u64) -> bool |  |  |  |  |

| `set_child` | function | set_child(&self, data: &[T]) |  |  |  |  |

| `set_entry` | function | set_entry(&mut self, idx: usize, offset: usize, length: usize) |  |  |  |  |

| `set_len` | function | set_len(&self, new_len: usize) |  |  |  |  |

| `set_null` | function | set_null(&mut self, row: usize) |  |  |  |  |

| `struct_child` | function | struct_child(&self, capacity: usize) -> StructVector |  |  |  |  |

| `struct_vector_child` | function | struct_vector_child(&self, idx: usize) -> Self |  |  |  |  |

| `ArrayVector` | struct | ArrayVector { ptr: duckdb_vector } |  |  |  |  |

| `FlatVector` | struct | FlatVector { ptr: duckdb_vector, capacity: usize } |  |  |  |  |

| `ListVector` | struct | ListVector { entries: FlatVector } |  |  |  |  |

| `StructVector` | struct | StructVector { ptr: duckdb_vector } |  |  |  |  |

| `Inserter` | trait |  |  |  |  |  |

| `Vector` | trait |  |  |  |  |  |


### crates/frozen-duckdb/src/duckdb/row.rs

| `and_then` | function | and_then(self, f: F) -> AndThenRows<'stmt, F> |  |  |  |  |

| `as_ref` | function | as_ref(&self) -> Option<&Statement<'stmt>> |  |  |  |  |

| `get` | function | get(&self, idx: I) -> Result<T> |  |  |  |  |

| `get_ref` | function | get_ref(&self, idx: I) -> Result<ValueRef<'_>> |  |  |  |  |

| `get_ref_unwrap` | function | get_ref_unwrap(&self, idx: I) -> ValueRef<'_> |  |  |  |  |

| `get_unwrap` | function | get_unwrap(&self, idx: I) -> T |  |  |  |  |

| `map` | function | map(self, f: F) -> Map<'stmt, F> |  |  |  |  |

| `mapped` | function | mapped(self, f: F) -> MappedRows<'stmt, F> |  |  |  |  |

| `next` | function | next(&mut self) -> Result<Option<&Row<'stmt>>> |  |  |  |  |

| `AndThenRows` | struct | AndThenRows { rows: Rows<'stmt>, map: F } |  |  |  |  |

| `Map` | struct | Map { rows: Rows<'stmt>, f: F } |  |  |  |  |

| `MappedRows` | struct | MappedRows { rows: Rows<'stmt>, map: F } |  |  |  |  |

| `Row` | struct | Row { pub(crate) stmt: &'stmt Statement<'stmt>, arr: Arc<Option<StructArray>>, current_row: usize } |  |  |  |  |

| `Rows` | struct | Rows { pub(crate) stmt: Option<&'stmt Statement<'stmt>>, arr: Arc<Option<StructArray>>, row: Option<Row<'stmt>>, current_row: usize, current_batch_row: usize } |  |  |  |  |

| `RowIndex` | trait |  |  |  |  |  |

| `Sealed` | trait |  |  |  |  |  |


### crates/frozen-duckdb/src/duckdb/statement.rs

| `execute` | function | execute(&mut self, params: P) -> Result<usize> |  |  |  |  |

| `exists` | function | exists(&mut self, params: P) -> Result<bool> |  |  |  |  |

| `insert` | function | insert(&mut self, params: P) -> Result<()> |  |  |  |  |

| `parameter_count` | function | parameter_count(&self) -> usize |  |  |  |  |

| `query` | function | query(&mut self, params: P) -> Result<Rows<'_>> |  |  |  |  |

| `query_and_then` | function | query_and_then(&mut self, params: P, f: F) -> Result<AndThenRows<'_, F>> |  |  |  |  |

| `query_arrow` | function | query_arrow(&mut self, params: P) -> Result<Arrow<'_>> |  |  |  |  |

| `query_map` | function | query_map(&mut self, params: P, f: F) -> Result<MappedRows<'_, F>> |  |  |  |  |

| `query_one` | function | query_one(&mut self, params: P, f: F) -> Result<T> |  |  |  |  |

| `query_polars` | function | query_polars(&mut self, params: P) -> Result<Polars<'_>> |  |  |  |  |

| `query_row` | function | query_row(&mut self, params: P, f: F) -> Result<T> |  |  |  |  |

| `raw_bind_parameter` | function | raw_bind_parameter( &mut self, one_based_col_index: usize, param: T, ) -> Result<()> |  |  |  |  |

| `raw_execute` | function | raw_execute(&mut self) -> Result<usize> |  |  |  |  |

| `raw_query` | function | raw_query(&self) -> Rows<'_> |  |  |  |  |

| `row_count` | function | row_count(&self) -> usize |  |  |  |  |

| `schema` | function | schema(&self) -> SchemaRef |  |  |  |  |

| `step` | function | step(&self) -> Option<StructArray> |  |  |  |  |

| `step2` | function | step2(&self) -> Option<arrow2::array::StructArray> |  |  |  |  |

| `stream_arrow` | function | stream_arrow( &mut self, params: P, schema: SchemaRef, ) -> Result<ArrowStream<'_>> |  |  |  |  |

| `stream_step` | function | stream_step(&self, schema: SchemaRef) -> Option<StructArray> |  |  |  |  |

| `Statement` | struct | Statement { conn: &'conn Connection, pub(crate) stmt: RawStatement } |  |  |  |  |



<!-- AGENT-FORBIDDEN-END -->

## Signature/type/default/errors table

<!-- RIGID table: header order is fixed; rows come only from the query. -->

| Item | Type | Signature | Params | Defaults | Errors | Invariants |
|------|------|-----------|--------|----------|--------|------------|

| `array_child` | function | array_child(&self) -> ArrayVector |  |  |  |  |

| `array_vector_child` | function | array_vector_child(&self, idx: usize) -> ArrayVector |  |  |  |  |

| `as_mut_ptr` | function | as_mut_ptr(&self) -> *mut T |  |  |  |  |

| `as_mut_slice` | function | as_mut_slice(&mut self) -> &mut [T] |  |  |  |  |

| `as_mut_slice_with_len` | function | as_mut_slice_with_len(&mut self, len: usize) -> &mut [T] |  |  |  |  |

| `as_slice` | function | as_slice(&self) -> &[T] |  |  |  |  |

| `as_slice_with_len` | function | as_slice_with_len(&self, len: usize) -> &[T] |  |  |  |  |

| `capacity` | function | capacity(&self) -> usize |  |  |  |  |

| `child` | function | child(&self, capacity: usize) -> FlatVector |  |  |  |  |

| `child_name` | function | child_name(&self, idx: usize) -> DuckDbString |  |  |  |  |

| `copy` | function | copy(&mut self, data: &[T]) |  |  |  |  |

| `get_array_size` | function | get_array_size(&self) -> u64 |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `len` | function | len(&self) -> usize |  |  |  |  |

| `list_child` | function | list_child(&self) -> Self |  |  |  |  |

| `list_vector_child` | function | list_vector_child(&self, idx: usize) -> ListVector |  |  |  |  |

| `logical_type` | function | logical_type(&self) -> LogicalTypeHandle |  |  |  |  |

| `num_children` | function | num_children(&self) -> usize |  |  |  |  |

| `row_is_null` | function | row_is_null(&self, row: u64) -> bool |  |  |  |  |

| `set_child` | function | set_child(&self, data: &[T]) |  |  |  |  |

| `set_entry` | function | set_entry(&mut self, idx: usize, offset: usize, length: usize) |  |  |  |  |

| `set_len` | function | set_len(&self, new_len: usize) |  |  |  |  |

| `set_null` | function | set_null(&mut self, row: usize) |  |  |  |  |

| `struct_child` | function | struct_child(&self, capacity: usize) -> StructVector |  |  |  |  |

| `struct_vector_child` | function | struct_vector_child(&self, idx: usize) -> Self |  |  |  |  |

| `ArrayVector` | struct | ArrayVector { ptr: duckdb_vector } |  |  |  |  |

| `FlatVector` | struct | FlatVector { ptr: duckdb_vector, capacity: usize } |  |  |  |  |

| `ListVector` | struct | ListVector { entries: FlatVector } |  |  |  |  |

| `StructVector` | struct | StructVector { ptr: duckdb_vector } |  |  |  |  |

| `Inserter` | trait |  |  |  |  |  |

| `Vector` | trait |  |  |  |  |  |

| `and_then` | function | and_then(self, f: F) -> AndThenRows<'stmt, F> |  |  |  |  |

| `as_ref` | function | as_ref(&self) -> Option<&Statement<'stmt>> |  |  |  |  |

| `get` | function | get(&self, idx: I) -> Result<T> |  |  |  |  |

| `get_ref` | function | get_ref(&self, idx: I) -> Result<ValueRef<'_>> |  |  |  |  |

| `get_ref_unwrap` | function | get_ref_unwrap(&self, idx: I) -> ValueRef<'_> |  |  |  |  |

| `get_unwrap` | function | get_unwrap(&self, idx: I) -> T |  |  |  |  |

| `map` | function | map(self, f: F) -> Map<'stmt, F> |  |  |  |  |

| `mapped` | function | mapped(self, f: F) -> MappedRows<'stmt, F> |  |  |  |  |

| `next` | function | next(&mut self) -> Result<Option<&Row<'stmt>>> |  |  |  |  |

| `AndThenRows` | struct | AndThenRows { rows: Rows<'stmt>, map: F } |  |  |  |  |

| `Map` | struct | Map { rows: Rows<'stmt>, f: F } |  |  |  |  |

| `MappedRows` | struct | MappedRows { rows: Rows<'stmt>, map: F } |  |  |  |  |

| `Row` | struct | Row { pub(crate) stmt: &'stmt Statement<'stmt>, arr: Arc<Option<StructArray>>, current_row: usize } |  |  |  |  |

| `Rows` | struct | Rows { pub(crate) stmt: Option<&'stmt Statement<'stmt>>, arr: Arc<Option<StructArray>>, row: Option<Row<'stmt>>, current_row: usize, current_batch_row: usize } |  |  |  |  |

| `RowIndex` | trait |  |  |  |  |  |

| `Sealed` | trait |  |  |  |  |  |

| `execute` | function | execute(&mut self, params: P) -> Result<usize> |  |  |  |  |

| `exists` | function | exists(&mut self, params: P) -> Result<bool> |  |  |  |  |

| `insert` | function | insert(&mut self, params: P) -> Result<()> |  |  |  |  |

| `parameter_count` | function | parameter_count(&self) -> usize |  |  |  |  |

| `query` | function | query(&mut self, params: P) -> Result<Rows<'_>> |  |  |  |  |

| `query_and_then` | function | query_and_then(&mut self, params: P, f: F) -> Result<AndThenRows<'_, F>> |  |  |  |  |

| `query_arrow` | function | query_arrow(&mut self, params: P) -> Result<Arrow<'_>> |  |  |  |  |

| `query_map` | function | query_map(&mut self, params: P, f: F) -> Result<MappedRows<'_, F>> |  |  |  |  |

| `query_one` | function | query_one(&mut self, params: P, f: F) -> Result<T> |  |  |  |  |

| `query_polars` | function | query_polars(&mut self, params: P) -> Result<Polars<'_>> |  |  |  |  |

| `query_row` | function | query_row(&mut self, params: P, f: F) -> Result<T> |  |  |  |  |

| `raw_bind_parameter` | function | raw_bind_parameter( &mut self, one_based_col_index: usize, param: T, ) -> Result<()> |  |  |  |  |

| `raw_execute` | function | raw_execute(&mut self) -> Result<usize> |  |  |  |  |

| `raw_query` | function | raw_query(&self) -> Rows<'_> |  |  |  |  |

| `row_count` | function | row_count(&self) -> usize |  |  |  |  |

| `schema` | function | schema(&self) -> SchemaRef |  |  |  |  |

| `step` | function | step(&self) -> Option<StructArray> |  |  |  |  |

| `step2` | function | step2(&self) -> Option<arrow2::array::StructArray> |  |  |  |  |

| `stream_arrow` | function | stream_arrow( &mut self, params: P, schema: SchemaRef, ) -> Result<ArrowStream<'_>> |  |  |  |  |

| `stream_step` | function | stream_step(&self, schema: SchemaRef) -> Option<StructArray> |  |  |  |  |

| `Statement` | struct | Statement { conn: &'conn Connection, pub(crate) stmt: RawStatement } |  |  |  |  |


<!-- ============================================================= -->
<!-- AGENT-FORBIDDEN-END: nothing below this line may describe     -->
<!-- code behavior.                                                -->
<!-- ============================================================= -->
