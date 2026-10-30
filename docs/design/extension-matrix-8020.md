# Extension Matrix 80/20

Operator architecture for which DuckDB extensions frozen-duckdb ships, which it
excludes, and why. Grounded in the NATIVE builder as it exists on disk.

## The 80/20 Claim

Four extensions carry ~80% of real analytical workloads. Everything else in
upstream's extension catalog is binary bloat with no sovereign-runtime leverage:
the features cannot be exercised through the host-call VFS wrap or the thin
drop-in API surface frozen-duckdb exposes.

## Core Four

| Extension | Operational role | Implementation pattern |
|---|---|---|
| `parquet` | Columnar interchange; read/write format of record | Static bake (`BUILD_EXTENSIONS`) |
| `json` | Semi-structured ingest; variant-adjacent access | Static bake (`BUILD_EXTENSIONS`) |
| `icu` / native time | Timezone + collation (see note below) | Static bake (`BUILD_EXTENSIONS`) |
| `httpfs` (virtual) | Remote object-store reads | NOT compiled; host-call VFS wrap |

### httpfs-virtual: delegation, not compilation

`httpfs` is excluded from the static bake. Remote-read capability is provided
by the host-call VFS wrap: the embedded engine sees a local file surface, the
host resolves remote fetches. Pattern precedent:
`docs/design/vfs-journal-design.md` and the wasm/OPFS wrap in
`docs/design/wasm-opfs-architecture.md`. This keeps TLS stacks, curl, and
cert-store coupling out of the mega-library binary; the host already owns a
network stack.

### icu: UNVERIFIED-UPSTREAM-CLAIM

Operator claim: DuckDB 2.0 has native non-ICU timezone support, which would let
`icu` drop out of the core set. Status: **UNVERIFIED-UPSTREAM-CLAIM**. The
vendored headers are pinned at `b6013f7` reporting `2.0.0`
(`crates/frozen-duckdb-builder/vendored-headers/`), but no receipt in this
repo demonstrates native timezone coverage equivalent to `icu` (DST rules,
named-zone lookup) without the extension. Until a falsifier runs — a build
without `icu` that passes a named-timezone + DST court — `icu` stays in the
static bake and the claim stays flagged.

## Current NATIVE Build Reality (divergence)

The NATIVE builder hard-pins a NINE-extension set, not the four above —
`crates/frozen-duckdb-builder/src/lib.rs`:

```bash
-DBUILD_EXTENSIONS=parquet;json;icu;httpfs;tpch;tpcds;fts;inet;sqlsmith
```

(line ~594, inside the cmake configure `Command`; TR2 rehearsal established
that this flag is an extension-NAME list, not a boolean).

Divergences from the operator architecture:

1. `httpfs` IS currently compiled into the NATIVE mega-library, contradicting
   the httpfs-virtual delegation pattern. Trimming it is the load-bearing
   change of this matrix.
2. `tpch`, `tpcds`, `fts`, `inet`, `sqlsmith` are baked despite being on the
   exclusion list below.

## Wasm Build Matrix (different set)

The wasm path (`docs/design/wasm-opfs-architecture.md`) is a DIFFERENT, smaller
surface. Its candidate `emcmake` flag set sets no `BUILD_EXTENSIONS` at all —
statically baked extensions default to upstream's minimal set (json/parquet
class). The wasm matrix and the NATIVE matrix are independently pinned and
currently disagree; any convergence must be an explicit flag change in both
places, never a silent assumption that one implies the other.

## Explicit Exclusion List

| Extension | Rationale |
|---|---|
| `tpch` | Benchmark-only; zero end-user workload coverage; binary bloat |
| `tpcds` | Benchmark-only; same class as `tpch` |
| `fts` | Full-text search is a host/index-layer concern, not engine surface |
| `inet` | IP typed literals; narrow; no sovereign-runtime leverage |
| `sqlsmith` | Fuzzer; test-harness tool, never shipped in a runtime binary |
| `sqlite_scanner` | Foreign-DB attach; niche; pulls in sqlite shim surface |
| `postgres_scanner` | Foreign-DB attach; host-delegation territory (httpfs pattern) |
| `excel` | Ingest format; host-side conversion precedes load |
| `autocomplete` | Shell/REPL UX; meaningless in an embedded library |
| `iceberg`, `delta` | Catalog-coupled; re-evaluate only with a real workload demand |

General rationale: every excluded extension grows the mega-library binary
(budget: `docs/design/binary-size-budget.md`) while adding capability reachable
only through SQL paths frozen-duckdb does not adjudicate, or capability the
host substrate already provides better (network, search indexing, format
conversion).

## Falsifier

A shipped mega-library whose `BUILD_EXTENSIONS` set disagrees with this
document (re-read the cmake args in `crates/frozen-duckdb-builder/src/lib.rs`
at use time) is FAIL. The `icu` claim flips from UNVERIFIED-UPSTREAM-CLAIM to
VERIFIED only via a build-without-icu court passing named-timezone + DST
behavior.

## See Also

- `docs/design/binary-size-budget.md`
- `docs/design/wasm-opfs-architecture.md`
- `docs/design/vfs-journal-design.md`
- `docs/reference/extension-build.md`
- `crates/frozen-duckdb-builder/src/lib.rs` (cmake configure, line ~594)
