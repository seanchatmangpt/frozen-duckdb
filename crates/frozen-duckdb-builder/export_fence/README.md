# export fence (KD-01)

Committed allowlist of the exact C-ABI surface a fenced DuckDB dylib may
export: 1,177 `_duckdb_*` symbols, zero C++-mangled exports.

## Provenance

Derived by lane X1/X1r from the built-and-fenced dylib
(`libduckdb_arm64_fenced4.dylib`, sha256 `aab11996…93d2a5`); this copy is
byte-identical to the original `/private/tmp/x1/exports.list`
(sha256 `b764b4849525…9674`). It is embedded into the builder via
`include_str!("../export_fence/exports.list")` and applied by
`fence_exports` at the end of the local-compile path, so a local compile can
no longer produce (or a poison-refusal restage silently revert to) the
27,682-mangled-export dylib that fired the KD-01 fence-restage hazard.

## Refresh instructions

The list is a projection of the upstream source at `SOURCE_PIN`
(`b6013f7aa2f36caabb9466ce1791667ea8a280bb`). Refresh ONLY when the source
pin moves AND the C-ABI surface changes:

1. Build or obtain a dylib from the new source (release download or local
   compile).
2. Extract the C-ABI surface:
   `nm -gU libduckdb*.dylib | awk '$2=="T" && $3 ~ /^_duckdb_/ {print $3}' | sort -u > exports.list`
3. Sanity-check: line count must be exactly the number of `_duckdb_*` `T`
   exports (was 1,177); run `scripts/doctor/checks/bin_audit.sh` after
   fencing a dylib with the new list — it must be green.
4. Update this README's provenance line (old sha, new sha) and land the
   change with a court receipt showing `fence_exports` produces a green
   census (0 mangled, N `_duckdb_*`) and a passing functional probe.

Never hand-edit entries; the list is regenerated, never curated.
