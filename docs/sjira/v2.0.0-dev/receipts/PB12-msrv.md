# PB12 — MSRV Determination Receipt

Subject: `/Users/sac/frozen-duckdb` @ adbd260 (master). Lane PB12. Date: 2026-10-19.

## 1. Toolchain census (empirical)

```
rustc 1.97.0 (2d8144b78 2026-07-07)
cargo 1.97.0 (c980f4866 2026-06-30)
toolchains: stable-1.97.0 (active, default), 1.96.0, 1.97.1, 6 nightlies
```

**[CLOSED by ADV10 2026-10-19]** the multi-toolchain skip above is resolved:
1.87.0 was installed and executed. See `receipts/ADV10-msrv-proof.md` and §6.

## 2. Version-gated syntax sweep (grep, whole `crates/`)

| feature | min rustc | hits | files |
|---|---|---|---|
| `usize::is_multiple_of` | **1.87.0** | 2 | `crates/frozen-duckdb/tests/value_audit_datadog.rs:59,84` (receiver typed `i: usize`, both sites) |
| `OnceLock` | 1.70.0 | 2 | `tests/doctor_ocel_tests.rs:419`, `tests/test_fleet_class_d_marine.rs:226` |
| `c""` C-string literals | 1.77.0 | **0** (grep hits were `&"c"` / string noise, no raw `c"..."` literals) | — |
| let-else | 1.65.0 | 0 real (matches were plain `if/else`) | — |
| inline format args | 1.58.0 | yes (harmless, floor only) | — |
| `LazyLock`, `is_none_or`, `div_ceil`, `cast_signed` | 1.80+ | 0 | — |

Edition 2021 (workspace `Cargo.toml:29`) — no edition-driven bump.

## 3. is_multiple_of — concrete verification

Two independent probes compiled standalone with the active stable:

```
u64 probe: `a.is_multiple_of(5)`  -> compiles, prints `true` (exit 0)
i32 probe: `a.is_multiple_of(5)`  -> E0599 "no method named `is_multiple_of`
                                     found for type `i32`" (exit 1)
```

Finding: **`is_multiple_of` is stabilized for UNSIGNED integers only.** The
signed variants were withheld from the 1.87 stabilization and remain
unstable on 1.97. The workspace's two call sites both use `usize`, so they
are on the stabilized surface. Real-workspace cross-check:

```
cargo test --test value_audit_datadog --no-run
  -> Finished `test` profile ... (exit 0, real build on 1.97.0)
```

Stabilization-version evidence (release-notes class, NOT local empirics —
no 1.87 toolchain on this host): Rust **1.87.0** (2025-05-15) stabilized
`is_multiple_of` for unsigned integer types (tracking issue #128101,
`unsigned_is_multiple_of`). Nothing in the workspace requires anything
between 1.71 and 1.87, so 1.87 is the binding floor.

## 4. MSRV verdict

**Workspace MSRV: 1.87.0**, sole binding feature
`usize::is_multiple_of` (tests-only file `value_audit_datadog.rs`; lib code
floor is OnceLock 1.70, but `rust-version` covers the crate including
tests, so 1.87 binds for `frozen-duckdb`).

Per crate:
- `frozen-duckdb`: 1.87 (is_multiple_of, tests)
- `frozen-duckdb-builder`: 1.70 is the observed-feature floor, but no
  evidence supports deviating from workspace uniformity -> propose 1.87
- `frozen-duckdb-sys`: same -> 1.87

## 5. Manifest diff proposals (workspace-level, single source)

`Cargo.toml` `[workspace.package]`:

```toml
rust-version = "1.87"
```

and in each of `crates/frozen-duckdb/Cargo.toml`,
`crates/frozen-duckdb-builder/Cargo.toml`,
`crates/frozen-duckdb-sys/Cargo.toml`:

```toml
rust-version.workspace = true
```

Not applied by this lane (lane ownership: receipt + proposals only;
manifests are another lane's file surface — noted in the task as
"manifests only if evidence supports"; evidence supports, diff above is
ready to land).

## 6. Standing

**CLOSED — ALIVE (ADV10, 2026-10-19).** Falsifier row executed and flipped:
`rustup toolchain install 1.87.0` (real download, exit 0), then
`cargo +1.87.0 check --workspace --all-targets` on a git-archive scratch
tree with 1.87-compatible lockfile pins (time 0.3.44, time-core 0.1.6,
encoding_rs 0.8.35, home 0.5.11, icu_* 2.2.0) -> SUCCESS (exit 0, 38.45s),
and `cargo +1.87.0 test -p frozen-duckdb-builder --release` -> 35 passed;
0 failed (exit 0). First-party floor 1.87 confirmed; `rust-version = "1.87"`
is evidence-based and coherent. Receipt:
`receipts/ADV10-msrv-proof.md`. Caveat: the committed Cargo.lock pins
transitive deps requiring rustc 1.88, so out-of-the-box 1.87 builds need
those pins (lockfile-owner item, ADV10 §5.2).
