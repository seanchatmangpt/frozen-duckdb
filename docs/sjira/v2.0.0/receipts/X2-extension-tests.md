# X2 — extension_tests + flock ignore-message spec alignment (KD-02 closure)

Lane X2, 2026-10-19. KD-02 criterion: "run `cargo test --test extension_tests`
and assert 0 unhandled failures."

## Files owned / changed

- `crates/frozen-duckdb/tests/extension_tests.rs` — NEW. One opt-in
  (`#[ignore]`-gated, tpch_integration_test.rs pattern) test
  `test_tpch_load_and_dbgen_smoke`: opens with
  `Config::default().allow_unsigned_extensions()`, `LOAD tpch;` from ADV4's
  locally-built artifact at `~/.duckdb/extensions/d54cbbe3d/osx_arm64/`,
  dbgen smoke at `sf = 0.01` under a dbgen serialization lock, asserts
  `nation` populated and a TPC-H query over it returns.
- `crates/frozen-duckdb/tests/flock_tests.rs` — both `#[ignore]` messages
  (lines 27, 533) replaced with the exact spec language:
  `flock blocked on upstream 2.0 AST changes at pin d54cbbe3d — parse_function_t API migration`
- `docs/sjira/v2.0.0/KNOWN-LIMITATIONS-2.0.0.md` — new section
  3 "flock `INSTALL` fails at the dev pin" with the exact disclaimer
  ("INSTALL flock fails at this pin — unsupported until upstream publishes
  a 2.0-compatible build"); later sections renumbered 4-8.

## Real runs (this session)

1. `cargo test -p frozen-duckdb --test extension_tests -- --ignored`
   - First run: FAILED on `nationkey` (wrong column name; TPC-H schema is
     `n_`-prefixed). Fixed to `n_nationkey`.
   - Final: `test test_tpch_load_and_dbgen_smoke ... ok` —
     `1 passed; 0 failed; ... finished in 0.95s`. Exit 0. 0 unhandled
     failures — KD-02 criterion met.
2. `cargo test -p frozen-duckdb --test flock_tests`
   - `9 passed; 0 failed; 2 ignored` — flock stays ignored (both sites),
     compiles clean with the new ignore text.

## Standing

- tpch extension smoke: ALIVE (ran + passed this session, exact commands above).
- flock: UNSUPPORTED at pin `d54cbbe3d` by design, ignore-gated, message
  now matches spec.
