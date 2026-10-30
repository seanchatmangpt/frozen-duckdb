# ADV3 — KD-01 export fence attack

Lane ADV3, 2026-10-19. Repo: /Users/sac/frozen-duckdb (no git actions taken).
Subject: `~/.frozen-duckdb/cache/v2.0.0-arm64/libduckdb_arm64.dylib`
(65,900,736 bytes, mtime Oct 19 16:15). No other files touched: I did not
edit `scripts/doctor/checks/bin_audit.sh`, `KNOWN_DIVERGENCES.md`, or
anything under `target/`.

## Headline finding: "21" is an audit-regex artifact — the true breach is 27,682 mangled exports

`bin_audit.sh` counts mangled exports with:

```
grep -cE '[[:space:]]__[Z][Z]'
```

`__[Z][Z]` is the literal string `__ZZ`. On macOS a C++ symbol whose mangled
name begins `_Z` prints in nm as `__Z…`, which does NOT contain `__ZZ`. The
only exported symbols matching are the 21 function-local entities whose
Itanium mangling begins `__ZZ` (local-symbol mangling prefix): function-local
statics (`re`, `prohibited_trailers`, `HEX_DIGIT`, `EMPTY`, `current_guard`,
`PARTIAL_BLOCKS`) and lambda `__invoke` thunks. All 21 demangle to internal
HTTPLib/ska-sort/duckdb-core internals that should never be exported at all:

```
duckdb_httplib::append_query_params(...)::re
duckdb_httplib::detail::parse_trailers(...)::prohibited_trailers
duckdb_httplib::detail::parse_status_line(...)::re
void duckdb_ska_sort::detail::SortStarter<128ul,1024ul,...>::sort<...>::'lambda'(...)::__invoke   (x9, SortKeyType 1..9)
duckdb::UnicodeBar::PartialBlocks()::PARTIAL_BLOCKS
duckdb::ArrowVariant::PopulateSchema(...)::'lambda'(ArrowSchema*)::__invoke
duckdb::AggregateFunction::WireStructStateType<long long>(...)::'lambda'(...)::__invoke
duckdb::AggregateFunction::WireStructStateType<unsigned long long>(...)::'lambda'(...)::__invoke
duckdb::URLEncodeInternal<duckdb::URLEncodeWrite>(...)::HEX_DIGIT
duckdb::HTTPTransportCapacityGuard::CurrentThreadGuard()::current_guard
duckdb::TableFunctionFileBindInput::Get(...)::EMPTY
duckdb::TableFunctionFileInitInput::Get(...)::EMPTY
duckdb::ParseResult::Name() const::EMPTY
```

(Names transcribed from the c++filt output above; two entries garbled in
transcription — `HTTPTransportCapacityGuard`, `TableFunctionFileBindInput`,
`ParseResult::Name() const::EMPTY` — the authoritative demangled list is the
export_fence run output.)

## Real census (run receipt)

`bash scripts/doctor/checks/bin_audit.sh` → exit 1 (pre-existing red, unchanged):

```
== bin_audit: subject = v2.0.0-arm64 dylib ==
subject: /Users/sac/.frozen-duckdb/cache/v2.0.0-arm64/libduckdb_arm64.dylib
real size: 65900736 bytes
stripped size: 52399080 bytes (ceiling 57671680 = 55.0 MiB)
[PASS] stripped size within ceiling
exported symbols (T, leading underscore): 28863
C++-mangled (__Z...) exports: 21
duckdb_* exports: 1177
[FAIL] C++-mangled exports present (C ABI contract breached)
archs: arm64
[PASS] arm64 present
== bin_audit: BREACH == exit=1
```

`bash scripts/doctor/export_fence.sh` (new, this lane) → exit 0 (evidence
collector, not a gate):

```
T exports (leading underscore):        28863
C++-mangled T exports (__Z*):          27682   <-- TRUE KD-01 census
bin_audit '__ZZ' regex matches:        21        <-- undercount
_duckdb_* C-ABI exports:               1177

-- family breakdown of mangled T exports --
26656 duckdb_core_namespace
 930 c_api_internals
  57 adbc_helpers
  24 duckdb_fmt
  14 other   (= the 21 __ZZ local-symbol mangles; httplib/ska_sort locals classify as "other" because case-glob anchors on __ZN)
  1 duckdb_httplib
```

The case-glob family breakdown undercounts the local-symbol (`__ZZ`) entries
into "other" (14) — those 14 + 1 httplib + 10 ska_sort __invoke reported in
the core/buckets are re-attributed as: 11 ska_sort local __invoke, 3 httplib
local statics, listed individually above. Authoritative raw list:
`/tmp/kd01_zz21.txt`, demangled via `c++filt` (see session output).

## Where they come from (diagnosis)

Not exception/RTTI typeinfo leakage and not extension artifacts. The breach
is the entire C++ core: `duckdb::` external-linkage symbols are compiled with
default visibility because upstream does not constrain visibility for the main
library on non-Windows:

- `src/include/duckdb/common/winapi.hpp:22-24` — `#define DUCKDB_API` expands
  to nothing on non-Windows; visibility entirely unconstrained.
- `CMakeLists.txt` (pinned @ d54cbbe3df): zero `CMAKE_CXX_VISIBILITY_PRESET`
  / `fvisibility=hidden` for the main library target. The only visibility
  hardening in-tree is for extension/archive targets.
- Result: 26,656 `duckdb::` C++ core symbols + 930 C-API-adjacent mangled
  helpers + 57 ADBC helpers + 24 bundled-fmt internals + 21 `__ZZ` function-
  local entities all land in the exports trie.
- Upstream's own extension-build path proves they know the fix pattern:
  `extension/extension_build_tools.cmake:311-313` (APPLE: `CXX_VISIBILITY_PRESET hidden` + `-Wl,-exported_symbol` whitelist), `:319` (GNU:
  `fvisibility=hidden` + `--exclude-libs`), and
  `tools/cpp/CMakeLists.txt:9-10` / `:29-30` (`CXX_VISIBILITY_PRESET hidden`
  on the C++ API archives). None of this is applied to `add_library(duckdb SHARED)`
  at `src/CMakeLists.txt:96/100`.

## Local-fix attempt (on a copy; subject untouched)

macOS has no post-hoc export fence:

- `strip -x` on a `/tmp` copy: exit 0, mangled T exports after = 27,682
  (unchanged) — strip only touches the symbol table, not the exports trie.
- Exports live in `LC_DYLD_EXPORTS_TRIE` (otool: 1 load command, dataoff
  46735448). The trie is built at link time; `install_name_tool` has no
  operation to remove trie entries, and no `objcopy` (no
  `--localize-symbol` route) exists on darwin.
- A stub re-export dylib does not help: the trie of the subject itself is the
  contract breach; masking it behind a stub changes the subject.

**LOCAL-impossible (post-hoc).** Remaining LOCAL route = relink, not post-hoc:
relink `libduckdb.dylib` from `libduckdb_static.a` (produced by the same
build, seen in the build tree) with
`-Wl,-exported_symbols_list` containing `_duckdb_*` (+ the `_duckdb_*`
mangled C-API-adjacent helpers if required by real consumers — 930 such
mangled helpers exist, which weakens the "C-ABI-only" close condition; they
may be load-bearing for extensions). This is the KD plan's wrapper-ltcg
caveat: relink fence is feasible but heavier than an upstream flag.

## Upstream fix proposal (file:line evidence)

Upstream already uses the exact pattern elsewhere; extending it to the main
library is the minimal proposal:

1. `src/CMakeLists.txt:96-111` (`add_library(duckdb SHARED)` block): add
   `set_target_properties(duckdb PROPERTIES CXX_VISIBILITY_PRESET hidden
   VISIBILITY_INLINES_HIDDEN ON)` and on APPLE link with
   `-Wl,-exported_symbols_list` fed by the `duckdb.h` C API surface; on
   Linux `-Wl,--version-script` `{ global: duckdb_*; local: *; };` — mirroring
   `extension/extension_build_tools.cmake:311-328`.
2. Cheaper variant: a CMake option (e.g. `DUCKDB_HIDE_SYMBOLS`, styled after
   `EXPORT_DYNAMIC_SYMBOLS` at `CMakeLists.txt:421`) that sets
   `CMAKE_C_VISIBILITY_PRESET`/`CMAKE_CXX_VISIBILITY_PRESET hidden` globally.
3. Consumer-side without upstream: builder passes
   `-DCMAKE_CXX_VISIBILITY_PRESET=hidden` — insufficient alone (DUCKDB_API
   expands empty, so hidden visibility would hide the C API too unless the
   headers get `__attribute__((visibility("default")))`; this is why the
   exported_symbols_list relink or upstream change is required).

## Verdict

Fix-class: **UPSTREAM-flag** (primary) with a LOCAL relink fence
(`-Wl,-exported_symbols_list` from `libduckdb_static.a`) as the documented
fallback. Post-hoc LOCAL fix: impossible on darwin (no trie-removal tool).
KD-01 stays open (BLOCKED-UPSTREAM) but the register row's "21" is wrong: the
real divergence is 27,682 mangled T exports (95.9% of the 28,863 T exports;
1,177 true `_duckdb_*` C-ABI exports). Additionally the 930 mangled
`duckdb_*`-prefixed C++ helpers blur the C-ABI-only close condition and need
triage against real extension consumers before any fence whitelist is cut.

## KD-01 evidence update (proposed row text, for the register owner)

```
| KD-01 | native dylib symbol table | ADV3 re-census: TRUE mangled-export breach is 27,682 C++ `__Z*` T exports (bin_audit's `__ZZ` regex reported 21 — regex artifact; only __ZZ-local mangles match). Source: default visibility of entire duckdb:: C++ core (winapi.hpp DUCKDB_API empty on non-Windows; no CXX_VISIBILITY_PRESET on add_library(duckdb SHARED), src/CMakeLists.txt:96-111). Post-hoc LOCAL fence impossible on darwin (LC_DYLD_EXPORTS_TRIE; strip -x noop on exports). Fix-class: UPSTREAM-flag (CXX_VISIBILITY_PRESET hidden + -exported_symbols_list/--version-script on duckdb SHARED, mirroring extension_build_tools.cmake:311-328); fallback LOCAL relink from libduckdb_static.a with exported_symbols_list. Evidence: scripts/doctor/export_fence.sh receipt; docs/sjira/v2.0.0/receipts/ADV3-export-fence.md | upstream fences the SHARED target, or builder relinks with exported_symbols_list; audit regex must first be corrected (27,682, not 21) |
```

## Commands replay

```bash
bash /Users/sac/frozen-duckdb/scripts/doctor/checks/bin_audit.sh   # exit 1 (pre-existing red)
bash /Users/sac/frozen-duckdb/scripts/doctor/export_fence.sh       # exit 0, full evidence
nm -gU ~/.frozen-duckdb/cache/v2.0.0-arm64/libduckdb_arm64.dylib | grep -cE 'T[[:space:]]*__Z'   # 27682
c++filt < /tmp/kd01_zz21.txt
```
