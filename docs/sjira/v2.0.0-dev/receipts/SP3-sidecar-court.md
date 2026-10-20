# SP3 Receipt: Sidecar Corruption Court (non-vacuity, negative mutation)

- Date: 2026-10-19
- Subject: `target/wasm-min/kdc05-min.wasm` sha256 `1fdeda0dd6fe8a104b9f94eed48cd90bd20e6fdb6cc60697a1d1f490fd4bc1c7` (25,731,939 bytes; original never mutated — shims run on copies in per-shim temp dirs, deleted after)
- Court: `scripts/doctor/sidecar_corruption_court.mjs` (self-contained, node >= 18)
- Exit: **0** (PASS) — see raw output below.

## Claim under test

Corrupting the wasm sidecar (exported function table / code section / truncation)
produces a **catchable, typed** failure — `WebAssembly.CompileError` / `RuntimeError` /
`LinkError` — never an uncatchable V8 abort or heap corruption.

## Result matrix (9 shims, real output)

| shim | mutation | error constructor | message head |
|---|---|---|---|
| 1 | truncate @50% (12,865,969 B) | CompileError | section (code 10, "Code") extends past end of the module |
| 2 | truncate @90% (23,158,745 B) | CompileError | section (code 10, "Code") extends past end of the module |
| 3 | truncate @end-1 | CompileError | section (code 11, "Data") extends past end of the module |
| 4 | code-flip @316725 | CompileError | Compiling function #276 failed: Invalid opcode: 0xfa42 |
| 5 | code-flip @12096766 | CompileError | Compiling function #12356 failed: undeclared reference to function #0 |
| 6 | code-flip @23876807 | CompileError | function index #2074777 is out of bounds |
| 7 | export-fn-flip @76380 (code+64) | CompileError | Compiling function #22 failed: invalid alignment |
| 8 | export-fn-flip @76572 (code+256) | CompileError | Compiling function #22 failed: invalid local index: 84 |
| 9 | export-fn-flip @76828 (code+512) | CompileError | Compiling function #22 failed: invalid signature index: 1807008 |

Every shim: typed=true, catch witnessed, `[alive:N]` follow-up statement executed
after each catch (process healthy, no heap corruption). Baseline before all shims:
`duckdb_library_version() = v2.0.0-dev88595` (matches
`target/wasm-min/receipt.txt`). Restore: original sha256 unchanged.

## Non-vacuity note (session finding)

First run of shims 1-3 reported NO-FLIP: the truncation shim returned a fresh
`Buffer.from(b.subarray(0,cut))` that was discarded, so the unmutated module was
written. The court correctly FAILED (a court that cannot flip is FAIL, not PASS).
Isolated check confirmed V8 does reject truncated modules
(`WebAssembly.compile(): section (code 10, "Code") extends past end of the module`),
so this was a court bug, fixed by letting `mutate` replace the buffer
(`const mutated = mutate(bytes) || bytes`). Second run: all 9 flips witnessed.

## Exit contract

0 only if baseline green AND every shim yields a typed catch with a healthy
process; any uncatchable crash or no-flip shim exits 1; missing subject/engine
exits 3 with `[SKIP] <reason>`.

## Replay

```bash
node /Users/sac/frozen-duckdb/scripts/doctor/sidecar_corruption_court.mjs
```
