# MAX7-converge: binaryen --converge on KDC-05 minimal wasm

Date: 2026-10-19. Lane: MAX7. Subject: `/Users/sac/frozen-duckdb/target/wasm-min/kdc05-min.wasm` (KDC-05 passing artifact, version-gated).

## Command

```
~/emsdk/upstream/bin/wasm-opt -O3 --strip-debug --converge \
  --enable-bulk-memory --enable-nontrapping-float-to-int \
  --enable-sign-ext --enable-mutable-globals \
  /tmp/max7/pre.wasm -o /tmp/max7/post.wasm
```

Exit 0. Runtime: 509.03s user / 2:47 wall (308% cpu), darwin, node v26.10.0.

First invocation without feature flags FAILED (exit 1, wasm-validator):
`memory.fill operations require bulk memory [--enable-bulk-memory-opt]` and
`all used features should be allowed` on `i32.trunc_sat_f64_*` — the emcc
output (emcc 6.0.13) targets bulk-memory + nontrapping-float-to-int; the flags
above are required for any binaryen pass over this artifact.

## Size / loadability matrix

| artifact | bytes | MiB | node V8 load | version | open/close | heap before/after | smoke |
|---|---|---|---|---|---|---|---|
| pre (`kdc05-min.wasm` copy, sha256 `be3bad5b…713085`) | 25,888,446 | 24.69 | OK | v2.0.0 | rc=0, handle 1893192 | 67,108,864 / 67,174,400 | PASS |
| post (converged, sha256 `1fdeda0d…4bc1c7`) | 25,731,939 | 24.54 | OK | v2.0.0 | rc=0, handle 1893192 | 67,108,864 / 67,174,400 | PASS |

Delta: -154,507 bytes (-0.60%).

Post-converge real smoke output:

```
[max7] artifact=/tmp/max7/post.wasm bytes=25731939
[max7] duckdb_library_version() = v2.0.0
[max7] duckdb_open rc= 0 db handle= 1893192n
[max7] heap before=67108864 after=67174400
[max7] PASS
EXIT=0
```

## Standing

**ALIVE** — CL4's caveat does NOT reproduce on the minimal build: converged
binaryen output loads and instantiates in node V8 (v26.10.0) with correct
version string and a passing open/close + heap probe. Post-converge is smaller
and fully passing, so `/tmp/max7/post.wasm` is the distribution candidate
(recommended install path: replace/promote alongside `target/wasm-min/kdc05-min.wasm`;
lane MAX7 does not own `target/`, so no promotion was performed).

## Notes

- Heap probe identical pre/post (64 MiB initial, +64 KiB after open/close cycle) — well under the 128 MiB ceiling.
- Gain is modest (0.6%); --converge fixed point was reached, further -Oz-style shrinking is a separate lane.
- Artifacts kept for replay: `/tmp/max7/pre.wasm`, `/tmp/max7/post.wasm`, runner `/tmp/max7/smoke_max7.mjs`.

## Coordinator promotion (2026-10-19)

`/tmp/max7/post.wasm` copied to `target/wasm-min/kdc05-min.wasm`.
sha256 post-copy: `1fdeda0dd6fe8a104b9f94eed48cd90bd20e6fdb6cc60697a1d1f490fd4bc1c7` — byte-identical to the converged artifact. Distribution candidate is now the converged module.
