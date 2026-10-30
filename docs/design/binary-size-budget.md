# Binary Size Budget (NFR-02 Court Design)

Status: DESIGN — every budget cell is BUDGET-ASSUMPTION until a real wasm
build produces measured numbers. This doc defines the court; it does not
claim the budget is met.

## Goal

The stripped engine artifact (wasm target) must be <= 25 MiB (26,214,400
bytes). The court FAILS on any artifact over that bound.

## Native reference point (measured, not the wasm target)

Measured on this machine, 2026-10-19, cached native arm64 dylib
(`~/.frozen-duckdb/cache/v2.0.0-arm64/libduckdb_arm64.dylib`):

| measurement            | command                             | result        |
|------------------------|-------------------------------------|---------------|
| unstripped native dylib| `ls -l` on cached file              | 66,935,424 B  |
| `strip -x` on a copy   | `strip -x /tmp/ld2.dylib`; `ls -l`  | 53,170,120 B  |
| full `strip` on a copy | `strip /tmp/ld.dylib` | FAILS nonzero: indirect syms unstrippable |

Interpretation: the ~67 MB unstripped native arm64 dylib is the native
comparison point only. Wasm strip semantics differ (wasm `strip` /
`wasm-opt -Oz` operate on custom sections and name section); the wasm
measurement command below is authoritative for NFR-02, not the native
numbers. Do not project native-to-wasm size ratios.

## Budget breakdown (all cells BUDGET-ASSUMPTION)

| component        | budget (MiB) | status            | measured (wasm) |
|------------------|--------------|-------------------|-----------------|
| core engine      | 14.0         | BUDGET-ASSUMPTION | none            |
| extensions       | 5.0 (total)  | BUDGET-ASSUMPTION | none            |
| VFS adapter      | 2.0          | BUDGET-ASSUMPTION | none            |
| boundary glue    | 2.0          | BUDGET-ASSUMPTION | none            |
| headroom         | 2.0          | BUDGET-ASSUMPTION | none            |
| total            | 25.0         | BUDGET-ASSUMPTION | none            |

The split is a starting hypothesis to force per-component attribution; it
is not a claim. First real wasm build replaces each cell with a measured
number + measurement command output in the receipt.

## Measurement command contract

Recorded verbatim per CI receipt, on the final stripped artifact:

```sh
STRIPPED=<artifact.wasm>
strip "$STRIPPED"        # or wasm toolchain strip; must exit 0
test "$(stat -f%z "$STRIPPED")" -le 26214400
echo "size_bytes=$(stat -f%z "$STRIPPED")"
```

Rules:

- Exit code of both commands is receipt content. `strip` nonzero or the
  bound test nonzero => court FAIL.
- A court that cannot measure (missing artifact, strip failure, stat
  failure) FAILS — it never SKIPs. A skip is loud and typed as FAIL.
- The byte number, artifact SHA-256, and DuckDB version go in the CI
  receipt alongside the command output.

## Falsifier

1. An artifact over 26,214,400 stripped bytes must FAIL the court.
2. A court that cannot produce a measurement must FAIL (exit nonzero,
   typed reason), not skip.
3. Anti-vacuity: injecting a size gate that always passes (e.g. bound
   of 2 GiB) must be detectable; the receipt must carry the literal byte
   count so a reviewer can recompute the comparison.

## Extension policy

- The extension set is an explicit build flag: `BUILD_EXTENSIONS` (e.g.
  `BUILD_EXTENSIONS="parquet,json,icu"`). No implicit default set.
- Each enabled extension's byte cost is measured: build once with the
  extension, once without, delta in bytes, recorded in the receipt.
- Per-extension budgets live under the 5.0 MiB extensions row once
  measured; until then each extension cost is BUDGET-ASSUMPTION.
- An extension whose measured delta exceeds its allocated slice fails
  the component court, independent of the total.

## See Also

- `docs/design/wasm-opfs-architecture.md`
- `docs/design/verification-plan.md`
