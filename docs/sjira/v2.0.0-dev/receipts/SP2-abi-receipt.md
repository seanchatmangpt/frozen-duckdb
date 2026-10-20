# SP2 — ABI Co-Pinning Receipt

**Standing: ALIVE (observed execution, exact commands below)**

Date: 2026-10-19. Lane SP2. Subject:
`/Users/sac/frozen-duckdb/target/wasm-min/kdc05-min.wasm`
(sha256 `1fdeda0dd6fe8a104b9f94eed48cd90bd20e6fdb6cc60697a1d1f490fd4bc1c7`,
25,731,939 bytes). Pin/engine bound to the ADV1 build receipt
(`target/wasm-min/receipt.txt`): pin `b6013f7aa2f36caabb9466ce1791667ea8a280bb`,
engine `v2.0.0-dev88595`.

## Surfaces landed

- `scripts/wasm_abi_receipt/abi_receipt.mjs` — stamper + fail-closed verifier
  (standalone node, zero deps; custom WASM section `abi_receipt`, payload JSON
  `{abi_receipt_v, digest_alg, digest, pin, engine_version}`; deterministic —
  no timestamps; digest = sha256 of the canonical form = module with all
  `abi_receipt` sections stripped, so the pin attests the exact module code).
- `scripts/wasm_abi_receipt/smoke_stamped.mjs` — instantiates the stamped
  artifact and asserts `duckdb_library_version()` == pinned version.
- `scripts/wasm_abi_receipt/abi_court.mjs` — negative-mutation court
  (BASELINE → BEFORE → SHIM → AFTER → RESTORE, fail-closed restore).
- `target/wasm-min/kdc05-min.stamped.wasm` — stamped artifact (promoted
  `kdc05-min.wasm` NOT overwritten; sha256 unchanged below).

No workspace deps added; no git commands run.

## 1. Stamp + verify (real output)

```
$ node scripts/wasm_abi_receipt/abi_receipt.mjs stamp --in kdc05-min.wasm \
    --out kdc05-min.stamped.wasm --pin b6013f7aa2f36caabb9466ce1791667ea8a280bb \
    --engine-version v2.0.0-dev88595
[abi-receipt] stamped kdc05-min.stamped.wasm bytes=25732155 pin=b6013f7aa2f36caabb9466ce1791667ea8a280bb engine=v2.0.0-dev88595 digest=1fdeda0dd6fe8a104b9f94eed48cd90bd20e6fdb6cc60697a1d1f490fd4bc1c7

$ node scripts/wasm_abi_receipt/abi_receipt.mjs verify --in kdc05-min.stamped.wasm \
    --pin b6013f7aa2f36caabb9466ce1791667ea8a280bb --engine-version v2.0.0-dev88595
[abi-receipt] PASS in=kdc05-min.stamped.wasm pin=b6013f7aa2f36caabb9466ce1791667ea8a280bb engine=v2.0.0-dev88595 digest=1fdeda0dd6fe8a104b9f94eed48cd90bd20e6fdb6cc60697a1d1f490fd4bc1c7 bytes=25732155
exit=0
```

The recorded digest equals the pre-stamp artifact sha256 — strip/re-stamp is
byte-idempotent. Section payload (`abi_receipt.mjs dump`):

```json
{"abi_receipt_v":1,"digest_alg":"sha256","digest":"1fdeda0dd6fe8a104b9f94eed48cd90bd20e6fdb6cc60697a1d1f490fd4bc1c7","pin":"b6013f7aa2f36caabb9466ce1791667ea8a280bb","engine_version":"v2.0.0-dev88595"}
```

## 2. Load + version() smoke on the stamped artifact (real output)

```
$ node scripts/wasm_abi_receipt/smoke_stamped.mjs target/wasm-min/kdc05-min.stamped.wasm v2.0.0-dev88595
[smoke] target/wasm-min/kdc05-min.stamped.wasm duckdb_library_version() = v2.0.0-dev88595
[smoke] PASS
exit=0
```

Stamps +216 bytes (custom section); module instantiates under node
v26.10.0 `WebAssembly` and the real C version function returns the pinned
engine version.

## 3. Non-vacuity court — all three shims flip (real output)

```
$ node scripts/wasm_abi_receipt/abi_court.mjs
[court] BASELINE subject=1fdeda0dd6fe8a104b9f94eed48cd90bd20e6fdb6cc60697a1d1f490fd4bc1c7
[court] BASELINE stamped=810519855f189545afacacfc122c638f6bf73b9eb08411f504a273b782150b4d
[court] BEFORE ok: [abi-receipt] PASS in=.../kdc05-min.stamped.wasm pin=b6013f7aa2f36caabb9466ce1791667ea8a280bb ... bytes=25732155
[court] BEFORE ok (self-stamp): [abi-receipt] PASS ... digest=1fdeda0d... bytes=25732155
[court] SHIM-COURT A applied: digest[0] 1->0 in .../digest-corrupt.wasm
[court] SHIM-COURT A FLIPPED: exit=1 ABI_MISMATCH: module digest differs (recomputed 1fdeda0d... != recorded 0fdeda0d...)
[court] SHIM-COURT B applied: abi_receipt section stripped -> .../stripped.wasm
[court] SHIM-COURT B FLIPPED: exit=1 ABI_MISMATCH: section absent
[court] SHIM-COURT C applied: re-stamped with pin=0000000000000000000000000000000000000000 -> .../wrong-pin.wasm
[court] SHIM-COURT C FLIPPED: exit=1 ABI_MISMATCH: pin differs (expected b6013f7aa2f36caabb9466ce1791667ea8a280bb, recorded 0000000000000000000000000000000000000000)
[court] RESTORE verified: subject+stamped match BASELINE byte-exact
[court] PASS: all shims flipped, restore byte-exact
court exit=0
```

A shim that fails to flip reports `[VACUOUS GATE]` and the court exits 1
(fail-closed: an unflippable gate is FAIL, not PASS).

## 4. Promoted artifact untouched (real output)

```
$ shasum -a 256 kdc05-min.wasm kdc05-min.stamped.wasm
1fdeda0dd6fe8a104b9f94eed48cd90bd20e6fdb6cc60697a1d1f490fd4bc1c7  kdc05-min.wasm
810519855f189545afacacfc122c638f6bf73b9eb08411f504a273b782150b4d  kdc05-min.stamped.wasm
```

Subject sha256 identical to BASELINE; promoted artifact not overwritten.

## Falsifiers

- A code mutation to the wasm module (any section) changes the canonical digest
  → `ABI_MISMATCH: module digest differs` (flip mechanism proven by SHIM A).
- Removing the receipt section → `ABI_MISMATCH: section absent` (SHIM B).
- A module stamped by a different authority/pin → `ABI_MISMATCH: pin differs`
  (SHIM C).
- Exit contract: 0 = full pass only; 1 = typed `ABI_MISMATCH` on stderr;
  2 = usage/IO. No silent pass path.
