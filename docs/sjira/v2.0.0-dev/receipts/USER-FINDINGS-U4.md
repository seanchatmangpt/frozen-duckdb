# USER-FINDINGS-U4 — Linux/Windows user matrix at 2.0.0-dev (lane U4)

Status: PARTIAL_ALIVE (static analysis + HTTP probes + builder unit tests; no Linux/Windows build executed in this lane — docker legs are REL4/REL6's)
Date: 2026-10-19
Lane: U4 (non-macOS user findings; NO git commands run)

## Real commands run (receipt basis)

```
$ curl -s -o /dev/null -w "%{http_code}" .../releases/download/v2.0.0-dev/libduckdb_x86_64.so
404
$ curl -s -o /dev/null -w "%{http_code}" .../releases/download/v2.0.0-dev/libduckdb_x86_64.dll
404
$ curl -s https://api.github.com/repos/seanchatmangpt/frozen-duckdb/releases
[]
```

```
$ cd crates/frozen-duckdb-builder && CARGO_TARGET_DIR=target-laneU4 cargo test --lib
test tests::tr7_bench_regression_bounds ... ok
test result: ok. 46 passed; 0 failed; 0 ignored; ... finished in 2.54s
```

## Compile-freeze disclosure (SLA addition 3, disclosed in this lane's receipt only)

Every cargo command in the tree failed with
`failed to read /Users/sac/frozen-duckdb/u2-probe/Cargo.toml` (u2-probe is a
root-workspace member with no manifest — U-lane in-flight dir). Minimal unblock
applied per SLA: stub `u2-probe/Cargo.toml` + `u2-probe/lib.rs` created,
disclosed here. Coordinator: drop the stubs at integration if U2's real landing
supersedes them, and delete `crates/frozen-duckdb-builder/target-laneU4`
(`rm -rf` was permission-denied in this lane).

## Answers to the six questions

1. **Filename ensure_binary seeks**: `library_extension_for` (lib.rs:50-57) →
   `libduckdb_{arch}.so` (linux), `libduckdb_{arch}.dll` (windows);
   `get_binary_path` (lib.rs:1142) caches `libduckdb_{arch}.{so|dll}` under
   `~/.frozen-duckdb/cache/v2.0.0-dev-{arch}/`.
2. **Download URL v2.0.0-dev**: `https://github.com/seanchatmangpt/frozen-duckdb/releases/download/v2.0.0-dev/libduckdb_{arch}.{so|dll}`
   (lib.rs:1154-1158). WITNESSED above: both return HTTP 404; the releases API
   returns `[]` — no release exists. Fallback compile runs on every platform.
3. **Fallback compile on Linux**: yes, handled — cmake configure
   (lib.rs:1264-1291) then `make -j4` (lib.rs:1294-1305). cmake's default
   generator on Linux is Unix Makefiles, so `make -j4` matches. **BUT
   OPENSSL_ROOT_DIR is NOT propagated** (no occurrence in
   `compile_duckdb_locally`, lib.rs:1197-1339) — httpfs is in the extension
   list (lib.rs:1269), and httpfs needs OpenSSL dev headers; a user with
   nonstandard OpenSSL hits a loud cmake configure bail (lib.rs:1285-1291).
   Windows: `make -j4` will not exist under an MSVC/VS generator → loud
   "Failed to build DuckDB" bail. Windows is documented fallback-only
   (lib.rs:44-49).
4. **verify_library_slice on Linux (F3 ELF check)**: CONFIRMED landed —
   `verify_library_slice_for` (lib.rs:731+) demands ELF magic `7f 45 4c 46`
   for linux (lib.rs:746) and `MZ` for PE/windows (lib.rs:777), with the
   10MiB floor (lib.rs:713, 756-763). Wrong-platform slices hard-refuse.
5. **ENGINE_VERSION / size-pin markers**: OS-AGNOSTIC — marker payload is
   `v{VERSION}` bytes (lib.rs:1033-1035), path is
   `<cache>/ENGINE_VERSION` (lib.rs:1039-1041), size pin is a plain byte
   count; neither touches OS-specific metadata.
6. **rpath story on Linux**: `cargo:rustc-link-arg=-Wl,-rpath,{lib_dir}`
   (frozen-duckdb-sys/build.rs:54) is target-unscoped, so on Linux it records
   an absolute `DT_RUNPATH` to the cache dir in every consumer binary — same
   mechanism as macOS. Un-tested in this lane (docker leg is REL4's); the
   cheap local corroboration: the unscoped form is documented to apply to all
   downstream rustc invocations. Caveat: the absolute path embeds the
   *build-time* `$HOME`; a different runtime user/host breaks load silently
   (loader ignores a nonexistent RUNPATH entry → "libduckdb.so: cannot open
   shared object file" — loud at exec, silent about the cause).

## Platform matrix (linux × windows, per phase)

Cells are static-analysis verdicts at 2.0.0-dev; nothing here was executed on
a Linux/Windows host in this lane.

| Phase | Linux x86_64/arm64 | Windows x86_64/arm64 |
|---|---|---|
| Filename sought | `libduckdb_{arch}.so` — correct | `libduckdb_{arch}.dll` — correct |
| Download | LOUD-FAIL → fallback (404 witnessed, lib.rs:71-78 comment matches) | LOUD-FAIL → fallback (404 witnessed) |
| Fallback compile | WORKS with caveats: git+cmake+make required; OpenSSL dev headers required by httpfs (NOT propagated as OPENSSL_ROOT_DIR — same gap as GH2 macOS); cross-arch refused loudly (lib.rs:673-684) | LOUD-FAIL: `make` absent under MSVC generator; check_local_compile_target permits it (same-arch), then cmake configure/make bails loudly |
| verify (slice) | WORKS — ELF magic + 10MiB floor hard-refuse (F3 landed) | PARTIAL — MZ check landed in code (lib.rs:777); REL5 receipt notes windows host leg untested; Unverifiable-degradation risk on hosts without a PE checker |
| ENGINE_VERSION/size pin | WORKS (OS-agnostic) | WORKS (OS-agnostic) |
| Link | WORKS expected — unscoped `-Wl,-rpath` yields DT_RUNPATH (static verdict; REL4 to docker-test) | UNKNOWN — `-Wl,-rpath` is invalid for MSVC (lld-link); risk of loud link failure or ignored arg; REL6's leg |
| Run | PARTIAL — DT_RUNPATH embeds build-time $HOME; works same-user, fails on other users/hosts (loader error at exec, cause unexplained) | UNKNOWN — DLL search: no PATH entry, no delay-load; exe next to DLL or PATH needed; rustc-link-arg story unresolved |
| Docs build (DOCS_RS=1) | WORKS by design — vendored headers, zero linking (build.rs:30-33, 95+) | same |

## Blocking/overclaim notes

- Nothing in this receipt was executed on Linux or Windows; all "WORKS" cells
  above are static verdicts with file:line citations, marked as such.
- `rm -rf` of the lane target dir was permission-denied; coordinator deletes
  `crates/frozen-duckdb-builder/target-laneU4` at integration.
- The u2-probe stub unblock is disclosed above; treat U2's landing as the
  authority for that directory.
