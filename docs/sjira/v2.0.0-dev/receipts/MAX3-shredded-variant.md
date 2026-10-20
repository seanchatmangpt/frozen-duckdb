# MAX3-shredded-variant: shredded variant bind via variant_json_udf

Date: 2026-10-19. Lane: MAX3.
Subject files:
- `crates/frozen-duckdb/tests/variant_shredded_bind.rs` (new)
- `crates/frozen-duckdb/src/duckdb/variant_json_udf.rs` (new)
- cfg gates declared for `vscalar` / `vtab-arrow` features in the UDF module

## Scope

Proves shredded variant binding end to end: JSON is ingested through the
variant JSON UDF (`variant_json_udf.rs`) and read back through shredded
variant binds, rather than round-tripping as opaque strings.

## Results (lane's final report, session 2026-10-19)

```
cargo test -p frozen-duckdb --test variant_shredded_bind
```

- 4 passed
- throughput: 100,000 rows at 80,244 rows/s

Cfg gates: the `vscalar` and `vtab-arrow` code paths in
`variant_json_udf.rs` are feature-gated (`#[cfg(feature = ...)]`), declared
up front so the crate compiles clean without the extension features enabled.

**Correction from fresh re-run (2026-10-19, this transcription lane)**: the
plain `cargo test -p frozen-duckdb --test variant_shredded_bind` command
does NOT compile — the test imports `variant_json_udf::register_json_extract`,
gated `#[cfg(all(feature = "serde_json", feature = "vscalar"))]` in
`src/duckdb/mod.rs:159` (E0432 when features are absent). The features flag
is REQUIRED; command corrected below.

Fresh re-run output (transcription lane, features enabled):

```
$ cargo test -p frozen-duckdb --test variant_shredded_bind --features serde_json,vscalar
running 4 tests
test null_payload_round_trips_as_none ... ok
test nested_path_reads_via_varchar_and_json_extract ... ok
test sampled_byte_fidelity ... ok
test appender_100k_heterogeneous ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.78s
```

## Standing

**PASS** — 4/4 green, throughput measured on real 100k-row ingress.

## Provenance

This receipt was transcribed post-run from the lane's final report (not
written by the lane itself). Verification command re-runnable verbatim:

```
cargo test -p frozen-duckdb --test variant_shredded_bind --features serde_json,vscalar
```

Re-run to verify (fresh output pasted above; exit 0).
