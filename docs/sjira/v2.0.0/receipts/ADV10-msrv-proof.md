# ADV10 — MSRV Proof Lane (closes PB12 falsifier)

Subject: `/Users/sac/frozen-duckdb` @ adbd260 (master, plus in-flight lane edits on
the same tree). Lane ADV10. Date: 2026-10-19. Scratch: `/tmp/adv10/`.

## 1. Toolchain install (real download)

```
$ rustup toolchain install 1.87.0
info: downloading 6 components
  1.87.0-aarch64-apple-darwin installed - rustc 1.87.0 (17067e9ac 2025-05-09)
EXIT=0
```

## 2. Canonical-tree check on 1.87.0 — FAILS on lockfile, not on code

Exact cargo output (session transcript, real exit 101):

```
$ CARGO_TARGET_DIR=/tmp/adv10 cargo +1.87.0 check --workspace --all-targets
error: rustc 1.87.0 is not supported by the following packages:
  encoding_rs@0.8.41 requires rustc 1.88   (x4 lines)
  home@0.5.12 requires rustc 1.88
  icu_collections@2.3.0 requires rustc 1.88
  icu_locale_core@2.3.0 requires rustc 1.88
  icu_normalizer@2.3.0 requires rustc 1.88
  icu_normalizer_data@2.3.0 requires rustc 1.88
  icu_properties@2.3.0 requires rustc 1.88
  icu_properties_data@2.3.0 requires rustc 1.88
  icu_provider@2.3.1 requires rustc 1.88
  time@0.3.55 requires rustc 1.88.0        (x2 lines)
  time-core@0.1.9 requires rustc 1.88.0    (x2 lines)
Either upgrade rustc or select compatible dependency versions with
`cargo update <name>@<current-ver> --precise <compatible-ver>`
```

The canonical lockfile pins rustc-1.88-requiring transitive versions. This
is a dependency-resolution finding, not a first-party MSRV failure.

## 3. Scratch re-resolution probe (git archive, sanctioned path)

Topology guard refused `rsync` of the repo; used `git archive HEAD` into
`/tmp/adv10/tree/`. Manifest note: the probe tree needed
`serde_json = { workspace = true, optional = true }` in
`crates/frozen-duckdb/Cargo.toml` (the one-line fix another lane landed on
the live tree mid-run; applied inside scratch only, disclosed here).

MSRV-aware `cargo +1.87.0 update` did NOT clear the blockers (kept
time 0.3.55 etc.), so explicit pins were applied in scratch:

```
time@0.3.55 -> 0.3.44          time-core@0.1.9 -> 0.1.6
encoding_rs@0.8.41 -> 0.8.35   home@0.5.12 -> 0.5.11
icu_collections/icu_locale_core/icu_normalizer/icu_normalizer_data/
icu_properties/icu_properties_data/icu_provider 2.3.x -> 2.2.0
(icu 2.0.0 refused: idna_adapter v1.2.2 requires icu_normalizer ^2.2)
```

Then:

```
$ cd /tmp/adv10/tree && CARGO_TARGET_DIR=/tmp/adv10/tgt \
  cargo +1.87.0 check --workspace --all-targets
    Checking frozen-duckdb-builder v2.0.0
    Checking test-validation v0.1.0
   Compiling frozen-duckdb-sys v2.0.0 (... Using prebuilt DuckDB binary:
      /Users/sac/.frozen-duckdb/cache/v2.0.0-arm64/libduckdb_arm64.dylib)
    Finished `dev` profile ... in 38.45s
CHECK_EXIT=0
```

Full `--all-targets` workspace (bins, tests, examples, lib) compiles clean
on 1.87.0. No first-party code requires anything above 1.87.

## 4. Test battery on 1.87.0

```
$ cd /tmp/adv10/tree && CARGO_TARGET_DIR=/tmp/adv10/tgt \
  cargo +1.87.0 test -p frozen-duckdb-builder --release
test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
   finished in 3.66s
test result: ok. 0 passed; 0 failed; ... (doc-tests)
TEST_EXIT=0
```

## 5. Findings

1. **First-party MSRV floor 1.87 CONFIRMED empirically** — workspace
   `--all-targets` check + builder release battery pass on rustc
   1.87.0 (17067e9ac 2025-05-09). `rust-version = "1.87"` is coherent.
2. **Lockfile defect (pre-existing, lockfile-owner surface)**: the canonical
   `Cargo.lock` pins encoding_rs 0.8.41, home 0.5.12, icu_* 2.3.x,
   time 0.3.55, time-core 0.1.9 — all requiring rustc 1.88. A 1.87 build
   from the committed lockfile fails at resolution. Remedy belongs to the
   lockfile owner: pin time 0.3.44 / time-core 0.1.6 / encoding_rs 0.8.35 /
   home 0.5.11 / icu_* 2.2.0 (icu 2.0.0 is not reachable; idna_adapter
   1.2.2 requires icu_normalizer ^2.2). Not applied to the canonical tree.
3. Manifest observation (disclosure, no edit by ADV10): early in the run
   `crates/frozen-duckdb/Cargo.toml` briefly presented a broken
   `serde_json` optional-dep form (`dep:` feature against a non-optional
   dep, resolution exit 101 on every toolchain); repaired on disk by
   another lane before ADV10 edited anything.

## 6. Standing

PB12 falsifier CLOSED. MSRV 1.87 claim: ALIVE for first-party code
(compile + test receipts above, real 1.87.0 toolchain). Caveat stands:
out-of-the-box 1.87 builds need the lockfile pins of §5.2; without them the
committed lockfile enforces rustc >= 1.88 regardless of `rust-version`.
