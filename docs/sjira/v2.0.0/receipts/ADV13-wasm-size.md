# ADV13 — WASM Size Court Receipt (GAP1b / SLA-2)

- Subject: `/Users/sac/frozen-duckdb/target/wasm/duckdb.wasm` — 41,099,303 bytes (39.20MiB), GAP1b artifact
- Court: `/Users/sac/frozen-duckdb/scripts/wasm_size_audit.sh` (reusable, self-contained; arg = artifact path, default `target/wasm/duckdb.wasm`)
- Run: 2026-10-20T00:39:26Z (plus 2026-10-20T00:36:23Z confirming script exit code 1 directly)
- Verdict: **FAIL** (script exit 1) — rung 1 and rung 2 over the raw SLA; this FAIL is the non-vacuous receipt

## Ladder results (real output)

| rung | tool (identity) | size | SLA | verdict |
|---|---|---|---|---|
| 1. strip | `llvm-strip` (emsdk `~/emsdk/upstream/bin/llvm-strip`) | 41,099,303 B (39.20MiB) | <=25MiB | **FAIL** |
| 2. wasm-opt | `wasm-opt version 112 (version_112)` — emsdk `upstream/bin/wasm-opt`, `--all-features -Os` | 41,017,453 B (39.12MiB) | <=25MiB | **FAIL** |
| 3. gzip -9 wire | `gzip -9` on wasm-opt output | 9,921,705 B (9.46MiB) | <=10MiB | **PASS** |
| 4. census | `wasm-objdump -h` (wabt 1.0.42) | see below | — | recorded |

No `[SKIP]` fired: binaryen (wasm-opt) IS present via emsdk, wabt present via Homebrew, llvm-strip present via emsdk. All four rungs executed for real (wall ~31s).

## Byte concentration (wasm-objdump -h, optimized artifact)

```
Code: 37,953,920 bytes (36.20MiB)   — 92.4% of the binary
Data:  2,954,646 bytes (2.82MiB)
Elem:     71,727 bytes (0.07MiB)
Function: 32,948 bytes (0.03MiB)
Type:      3,631 bytes (0.00MiB)
```

## Findings for KD-05 size close condition

1. Strip is a no-op here: 41,099,303 → 41,099,303 bytes. The artifact carries no
   strippable metadata; the size is payload, not debug info.
2. wasm-opt `--all-features -Os` recovers only 81,850 bytes (0.20%). The
   14.2MiB gap to the 25MiB raw SLA **cannot be closed by post-build binaryen
   passes**; the reduction must come upstream of codegen (feature/link flags,
   which DuckDB extensions get compiled in, `-Os`/`-Oz` at the C++ compile,
   LTO, or splitting the build).
3. Code section = 92.4% of bytes (36.2MiB). Data is only 2.82MiB — this is a
   code-volume problem (compiled-in DuckDB feature surface), not an embedded-
   asset problem.
4. Wire (gzip -9) is already within SLA at 9.46MiB < 10MiB — but with only
   0.54MiB margin, and it passes only because gzip absorbs the raw overflow;
   the raw rung remains the binding constraint.
5. All tools present (loud-skip paths exercised none). Toolchain: emsdk
   llvm-strip + wasm-opt 112; Homebrew wasm-strip 1.0.42 / wasm-objdump.

## Non-vacuity note

The court is fail-closed by construction: missing tool → `[SKIP]` and PASS=0
(nonzero exit); any rung over SLA → FAIL exit 1. The real run flipped FAIL at
rungs 1 and 2, proving the gate refuses rather than rubber-stamps.

## See Also

`docs/sjira/v2.0.0/PUBLISH-DECISION.md` · `scripts/build_wasm_mvp.sh` · KD-05
