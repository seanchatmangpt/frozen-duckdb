# Benchmarking Utilities

The `benchmark` module measures and compares wall-clock durations. Real
timing is `std::time::Instant`-based; nothing here is mockable or sampled.

## Measuring

- `measure_build_time` — run a closure and return its elapsed `Duration`.
  The closure runs exactly once; timing survives an `Err` return (the error
  is returned with the measured duration).
- `compare_build_times` — run two closures and return both durations for
  A/B comparison of build or load strategies.

## Example

```rust,no_run
use frozen_duckdb::benchmark;
let (a, b) = benchmark::compare_build_times(
    || cold_start_strategy(),
    || warm_cache_strategy(),
);
println!("cold={a:?} warm={b:?}");
```
