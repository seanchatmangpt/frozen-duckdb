# Freeze as Structural Moat

Revision-2 baseline fact 4: eliminating the multi-minute C++17 compile
transforms embedded deployment of DuckDB from an operational bottleneck into
instant, zero-dependency integration. This is not a convenience feature — it
is the structural moat of the `frozen-duckdb` design: the expensive
manufacturing step (native compilation) is performed once by the publisher,
and every consumer gets the frozen result in milliseconds.

## 1. The moat, stated precisely

A consumer integrating embedded DuckDB today faces two options:

- **Vendor the source, compile at build time.** Every downstream CI run pays
  a multi-minute (in practice 30-90 min) C++17 build. Toolchain drift,
  cmake/make failures, and arch-specific breakage are inherited by every
  consumer forever.
- **Ship the source as a build dependency.** Same cost, plus the consumer
  owns toolchain pinning.

`frozen-duckdb` removes the C++ compiler from the consumer's critical path
entirely. The C++17 build runs once, on the publisher's side, in a controlled
matrix. Consumers link a frozen mega-library out of a content-addressed local
cache. The README claim is the contract: "Zero-compilation DuckDB with
Builder Sub-Crate Pattern - Fast builds forever!" (README.md line 5).

## 2. Measured repo reality: the builder sub-crate

The moat is already implemented for consumers, in
`crates/frozen-duckdb-builder/src/lib.rs`.

### 2.1 `ensure_binary` resolution order

`ensure_binary` (lib.rs, documented at the `pub fn ensure_binary` entry,
~line 93) resolves the binary in this order:

1. **Cache hit** — `~/.frozen-duckdb/cache/v{VERSION}-{arch}/` is checked
   first. On hit: `info!("Using cached DuckDB binary: ...")`, and the builder
   returns the path. This path performs zero compilation and zero network
   I/O. This is the common case after the first acquisition, and it is
   effectively instant relative to any C++ build.
2. **Release download** — on cache miss, `download_from_github_release`
   fetches the prebuilt library into the cache (log: "Found prebuilt binary,
   copying to cache"), then normalizes the cache with `ensure_headers`
   (vendored `duckdb.h`/`duckdb.hpp` so bindgen works offline) and
   `ensure_link_name`.
3. **Local compile — fallback only** — `compile_duckdb_locally` is reached
   only when both cache and download fail. The doc comment at lib.rs line 73
   states it plainly: "`ensure_binary` falls back to `compile_duckdb_locally`,
   which is the" fallback. This is the multi-minute tier: measured local
   compiles of DuckDB from pinned `v1.5.5` source run 30-90 minutes and
   require a C++17 toolchain + cmake.

The order is load-bearing: compilation is the *last* resort, not the default.
That ordering IS the moat.

The ordering is verified by the gate surface, not just prose: `make
verify-gates` (bash `scripts/verify-gates.sh`) runs the full gate set, and
the repo's receipt discipline (gates/dry-run-publish-gates.md, 5/5 green per
commit e4cfbd3) requires the cache-hit path to be exercised, not narrated.

### 2.2 Why the cache hit is "instant"

On a warm cache, `ensure_binary` does: `get_cache_dir` → join a versioned
directory → stat a file → return. No process spawn. No cmake. No cargo-cc
invocation. The only work is filesystem metadata, which is why the consumer
build pays DuckDB's cost exactly once per machine — and with the release
download tier, exactly once per (version, arch) per machine ever.

The stale-cache guard (lib.rs ~line 101: "stale cached binary does not match
its label; remove ... and rebuild") closes the "instant" claim's failure
mode: a cache hit that would resolve the wrong slice fails loudly instead of
silently passing a wrong binary as instant.

## 3. The next freeze tier: the wasm artifact

Post-FR-01, the freeze surface extends to a wasm artifact: ≤25 MB stripped
on disk, <10 MB on the wire. This is the same moat pushed one tier further:

- **Today**: the frozen unit is a native mega-library per (version, arch),
  acquired via cache/download; compile is fallback.
- **Tomorrow**: the frozen unit is a single portable wasm module — no arch
  dimension at all. One artifact freezes across macOS arm64/x86_64 and Linux,
  collapsing the (version, arch) matrix to (version).

That collapse is the structural win: the publisher's compile matrix
(build-binaries.yml runs one local-compile leg per matrix arch, with
`CMAKE_OSX_ARCHITECTURES` selected per leg at lines 50-57) exists precisely
because native artifacts freeze per-arch. A wasm freeze deletes the arch
axis from the publisher side, so the 30-90 min compile cost is paid once per
version, not once per (version, arch).

## 4. The CI angle: only the publisher compiles

CI asymmetry is the operational face of the moat. The publisher workflow
`.github/workflows/build-binaries.yml` is the only surface that needs a C++
toolchain: it installs cmake (`brew install cmake`, line 37) and drives the
local-compile fallback via `cargo run --release` (line 113) per matrix arch.

Every consumer CI run — the whole downstream fleet — needs no cmake, no
C++17 compiler, no brew toolchain step. The consumer CI surface is plain
`cargo build`/`cargo test` against the frozen library. That asymmetry is the
mechanism by which "fast builds forever" is enforced: the compile cost is
confined to a single workflow in a single repo, and never propagates.

## 5. Falsifier: the cache-path court

The moat claim is falsifiable. A consumer cold-start must not invoke a
compiler.

**Court**: on a cold cache, run the consumer acquisition path and assert no
compiler spawn. Concretely: clear or point `~/.frozen-duckdb/cache` at a
temp dir, run `ensure_binary` (via the builder's public surface), and assert
that no `cmake`, `make`, or cargo-cc-style C++ build processes are spawned
during acquisition — the download tier must satisfy the consumer, and
`compile_duckdb_locally` must not run. If a compiler is ever spawned on the
consumer cold-start path, the moat is refuted: "zero-compilation" has
regressed into a hidden 30-90 minute tax.

The same court inverts into a non-vacuity check: if the court cannot flip on
a subject where cache and download are both severed (forcing the fallback),
the court is vacuous — it is not actually observing process spawns.

See Also: README.md line 5 · crates/frozen-duckdb-builder/src/lib.rs
(`ensure_binary`, `compile_duckdb_locally`) ·
.github/workflows/build-binaries.yml · gates/dry-run-publish-gates.md
