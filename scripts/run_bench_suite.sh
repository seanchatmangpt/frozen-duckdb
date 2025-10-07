#!/usr/bin/env bash
set -euo pipefail
export RUSTFLAGS="-C target-cpu=native"
cargo bench --bench sparql_exec_bench --bench kernels_bench --bench hooks_bench --bench convert_validate_bench --bench receipts_ffi_bench
KC_ROWS=${KC_ROWS:-200000} KC_ITERS=${KC_ITERS:-300} cargo test -p kcura-tests whitepaper_stress -- --nocapture
echo "Artifacts:"
ls -l target/criterion
ls -l target/whitepaper
