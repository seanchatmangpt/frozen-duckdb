# X8r — Wave Consolidation Receipt

Lane X8r, repo /Users/sac/frozen-duckdb, 2026-10-19. No git operations.
Consolidates the wave from receipt files **verified present on disk** under
`docs/sjira/v2.0.0-dev/receipts/` at write time. Lanes without a receipt file
are listed IN-FLIGHT, not cited.

## Landed (receipt file verified on disk)

| lane | receipt file | standing | key evidence (from the receipt) |
|---|---|---|---|
| MAX6 | `MAX6-wasm-spec-flags.md` | BLOCKED (size SLA FAIL, honest) | operator exact flag set, emsdk 6.0.13, upstream `b6013f7aa2`; raw 29,242,868 B = **27.89 MiB** vs 25 MiB gate: FAIL (+7.7%); build/link/smoke PASS |
| MAX7 | `MAX7-converge.md` | ALIVE (converged, promoted) | `wasm-opt -O3 --converge` exit 0, 509s; 25,731,939 B = **24.54 MiB** (-0.60%), node load PASS; coordinator promotion: `/tmp/max7/post.wasm` → `target/wasm-min/kdc05-min.wasm`, sha256 `1fdeda0d…4bc1c7` post-copy byte-identical |
| SP2 | `SP2-abi-receipt.md` | ALIVE (observed execution) | ABI receipt over `target/wasm-min/kdc05-min.stamped.wasm`; subject sha256 identical to BASELINE; promoted artifact not overwritten |
| SP3 | `SP3-sidecar-court.md` | PASS (exit 0) | sidecar corruption court, 9 shims, real output matrix; corrupt wasm yields typed `CompileError`/`RuntimeError`/`LinkError`, never V8 abort — non-vacuity negative mutation |
| SP4 | `SP4-icu-sidecar.md` | PARTIAL_ALIVE / BLOCKED at rung 3 | 1.28 MiB icu side module built (exit 0); smoke exit 2 typed BLOCKED — dynamic-load seam needs `-sMAIN_MODULE=2` core (separate lane) |
| CL4b | `CL4b-icu-free.md` | BUILD_ALIVE / SMOKE_ALIVE / SIZE_FLOOR_UNCHANGED | icu lever is dead: icu-free build does not drop below the size floor (27.61 MiB class); closes the CL4-identified lever |
| KDC11r | `KDC-11-opfs.md` | ALIVE (L1/L3/L4) on `target/wasm/duckdb.wasm` 41,099,303 B | first browser court via CDP chrome-headless-shell, run17 full PASS exit 0; L2 programmatic surface MISSING (typed) |

Note: MAX6 BLOCKED + CL4b floor-unchanged jointly close the size question at
this pin: no flag/extension lever reaches ≤25 MiB; MAX7's converged 24.54 MiB
module is the distribution candidate.

## IN-FLIGHT (no receipt file on disk at consolidation time)

MAX1 (poison typed exit, 46+1+12 builder tests), MAX2, MAX3 (shredded VARIANT +
json_extract UDF), MAX4, MAX5, MAX8 (monotonic gate), MAX9 (DFG 4.968ms),
MAX10, SP1 (profile matrix, wired `pub mod`), SP5, AD3r, CF1 (rev002 fix),
CI-PS (pre-seed best-effort), REL4r, REL6r, X1r.

None of the above are cited as landed; their claims live only in dispatch text
until a receipt file lands under `docs/sjira/v2.0.0-dev/receipts/`.

## Pending lint nits (2)

- `scripts/wasm_abi_receipt/abi_receipt.mjs:21` — `statSync` imported, unused.
- `scripts/doctor/sidecar_corruption_court.mjs` — `ok3` assigned, read only in
  its own branch (unused-as-flag nit).

## Falsifier

Any row above whose receipt file is absent at replay time invalidates the
consolidation; IN-FLIGHT lanes flip to landed only by writing a receipt file
with real embedded command output.
