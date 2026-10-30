# KDC-01 — KD-01 upstream issue filed

Lane KDC-01, 2026-10-19/20. Repo: /Users/sac/frozen-duckdb (no git actions).
Closes the 80/20 path for KD-01 (native dylib symbol table).

## Filed

**UPSTREAM ISSUE FILED: https://github.com/duckdb/duckdb/issues/27511**

Verified via `gh issue view 27511 -R duckdb/duckdb`:
author `seanchatmangpt`, createdAt 2026-10-20T01:19:33Z, title
"Main library SHARED target exports entire C++ core (27,682 mangled symbols)
despite extension targets using visibility-hidden + exported-symbol
whitelists". `gh issue create` exit 0.

## Issue content (as filed)

- Precedent: upstream's own `extension/extension_build_tools.cmake:311-328`
  (APPLE `CXX_VISIBILITY_PRESET hidden` + `-Wl,-exported_symbol` whitelist;
  GNU `hidden` + `--exclude-libs,ALL` / version script) and
  `tools/cpp/CMakeLists.txt:9-10, 29-30` harden extension/archive targets,
  but `src/CMakeLists.txt:96-111` `add_library(duckdb SHARED)` gets none,
  and `winapi.hpp:22-24` leaves `DUCKDB_API` empty on non-Windows.
- Our demangled census (ADV3 receipt, re-run this lane):
  28,863 T exports / 27,682 C++-mangled (95.9%) / 26,656 `duckdb::` core /
  930 C-API-adjacent mangled helpers (889 unique demangled) / 1,177 true
  `_duckdb_*` C-ABI exports.
- Fix proposals: (1) mirror the extension fence onto the SHARED target
  (visibility hidden + exported_symbols_list / version script);
  (2) `DUCKDB_HIDE_SYMBOLS` CMake option; both require the C API marked
  `visibility("default")` in `duckdb.h` since `DUCKDB_API` expands empty
  on non-Windows.

## LOCAL fallback triage (step 3)

Question: how many of the 930 mangled `duckdb_*` helpers are referenced by
any extension binary we ship — scoping the exported_symbols_list whitelist.

Method: extracted all 27,682 mangled T exports from
`~/.frozen-duckdb/cache/v2.0.0-arm64/libduckdb_arm64.dylib` via
`nm -gU`, demangled (`c++filt`), filtered to names beginning `duckdb_` →
889 unique demangled helpers (≈ the 930 bucket; remainder is the
adbc_helpers bucket split); 750 unique `::`-suffix name components.
Searched every shipped extension binary (5 total: 4 wasm
`target/wasm-build/build/repository/d54cbbe3d/wasm_eh/{json,parquet,core_functions,icu}.duckdb_extension.wasm`
+ native `target/adv4/native-duckdb/build/extension/tpch/tpch.duckdb_extension`)
for each suffix as a literal byte string.

**Result: 0 hits across all 5 extensions.** The only `duckdb_*` strings in
the extensions are 1–10 plain C-ABI names per binary (e.g.
`duckdb_open/create/query/...`), all within the 1,177 `_duckdb_*` C-ABI
whitelist.

Scope verdict: the 930 mangled helpers are NOT load-bearing for any
extension we ship. A C-ABI-only `exported_symbols_list` fence (1,177
`_duckdb_*` entries) is safe for the extension surface; the LOCAL relink
fallback from `libduckdb_static.a` needs no extension-ABI carve-outs.

## Fix-class update

KD-01 fix-class: **UPSTREAM-issue-filed** (gh#27511, verified 2026-10-20)
+ **LOCAL-relink-triaged** (fence scope = 1,177 `_duckdb_*` C-ABI only;
zero mangled-helper references in shipped extensions).
Demangled count correction (27,682, not 21) landed with ADV3 and is cited
in the filed issue.

Commands replay:

```bash
gh issue create -R duckdb/duckdb --title "Main library SHARED target exports entire C++ core (27,682 mangled symbols) despite extension targets using visibility-hidden + exported-symbol whitelists" --body-file /tmp/kdc01_issue.md   # exit 0 -> #27511
gh issue view 27511 -R duckdb/duckdb --json number,title,author,createdAt
nm -gU ~/.frozen-duckdb/cache/v2.0.0-arm64/libduckdb_arm64.dylib | grep -E 'T[[:space:]]*__Z' | awk '{print $3}' > /tmp/kdc01_mangled.txt   # 27682
# grep each of 750 helper suffixes against the 5 shipped extension binaries -> 0 hits
```
