# REL5 — Windows Prebuilt Feasibility Audit (technical feasibility only)

Lane: REL5 · Window: v2.0.0 · Date: 2026-10-19
Scope: technical feasibility of the "Windows falls back to source compile" claim.
No legal analysis. No git commands run.

## Method

Read the builder source and CI directly, and probed upstream DuckDB's v1.5.5
release assets live (this is the one upstream ground-truth check; both probes
ran from this session):

```
$ curl -s https://api.github.com/repos/duckdb/duckdb/releases/tags/v1.5.5 | grep '"name"'
  ...
  "name": "libduckdb-linux-amd64-musl.zip"
  "name": "libduckdb-linux-amd64.zip"
  "name": "libduckdb-linux-arm64-musl.zip"
  "name": "libduckdb-linux-arm64.zip"
  "name": "libduckdb-osx-universal.zip"
  "name": "libduckdb-src.zip"
  "name": "libduckdb-windows-amd64.zip"
  "name": "libduckdb-windows-arm64.zip"
```

## (1) Upstream DuckDB v1.5.5 release assets

**YES — upstream ships Windows prebuilts.** Real observed asset names from the
v1.5.5 tag (unauthenticated GitHub API, exit 0):

- `libduckdb-windows-amd64.zip` and `libduckdb-windows-arm64.zip`
- also `libduckdb-linux-amd64.zip` / `-arm64.zip` (+ musl variants)

So the pattern's "Windows falls back to source compile" limitation is a
frozen-duckdb limitation, not an upstream constraint. Upstream naming is
hyphen-separated (`libduckdb-windows-amd64.zip`); ours is
underscore-separated (`libduckdb_{arch}.dylib`), so a "download upstream
zip" shortcut is a re-shape, not a rename.

## (2) Builder Windows path (`crates/frozen-duckdb-builder/src/lib.rs`)

`library_extension_for` (lib.rs:50-58):

```rust
"macos" => "dylib",
"linux" => "so",
"windows" => "dll",
```

Windows IS supported in the naming law — `.dll` is produced by
`release_asset_name_for("windows", ...)` and unit-tested
(`test_library_extension_for_platforms`, lib.rs:1396-1402, asserts
`library_extension_for("windows") == "dll"`).

Honest audit of what is macOS-shaped beyond naming:

- **`verify_library_slice_for` is macOS-only** (lib.rs:726-735):
  `if os != "macos" { return Ok(SliceCheck::Unverifiable("slice check is
  macOS-only (os=…)")) }`. It uses `lipo -archs` + Mach-O magic checks. On
  Linux/Windows the slice check silently degrades to Unverifiable — the
  download path would serve an unverified payload on those platforms.
- **`ensure_install_id` is `#[cfg(target_os = "macos")]`** (lib.rs:808);
  non-macOS is a no-op stub (lib.rs:885-889). Correct for ELF/PE (no rpath
  mechanism), so not a gap — but it means the KD-15 fix has zero Windows
  coverage.
- **Local-compile candidate list includes `.dll`** (lib.rs:1286-1303:
  `libduckdb.dll` in 6 candidate locations incl. `src/Release/`), so the
  MSVC local-compile output name is anticipated, though never exercised
  (devs are on macOS; no Windows host has run this path).
- **Download would 404 today on every platform**: no release is published
  (documented in the doc-comment at lib.rs:66-78), so `ensure_binary` falls
  back to local compile everywhere; Windows is simply the only platform
  where that fallback is slow/fragile (full MSVC DuckDB build).

## (3) Linux `.so` naming

**Supported in code.** `library_extension_for("linux") == "so"` (lib.rs:53),
unit-test asserts `release_asset_name_for("linux", "arm64") ==
"libduckdb_arm64.so"` and `("linux", "x86_64") == "libduckdb_x86_64.so"`
(lib.rs:1416-1421). Same macOS-only slice-check gap as Windows. Doc-comment
at lib.rs:71-78 records that no `.so` asset exists yet.

## (4) Conclusion table

| platform | builder-code-ready? | CI leg exists? | asset produced? | gap-to-close |
|---|---|---|---|---|
| macOS arm64/x86_64 | ALIVE (naming + Mach-O verify + KD-15 stamp) | yes — `build-binaries.yml:16` `runs-on: macos-latest`, arch matrix, stages `libduckdb_{arch}.dylib` | yes (v2.0.0 pipeline) | publish the release (assets stage, none published) |
| Linux .so | naming yes (`libduckdb_{arch}.so`); verify degrades to Unverifiable; ELF check absent | no — single `runs-on: macos-latest`, no OS matrix | no | ubuntu runner leg staging `.so` (local compile already emits `libduckdb.so` candidates); ELF/magic slice check to replace macOS-only `lipo` path |
| Windows .dll | naming yes (`libduckdb_{arch}.dll`); verify degrades to Unverifiable; no PE check | no — no windows runner | no | windows runner leg (local compile is MSVC-shaped; `.dll` candidates exist); PE/`dumpbin`-class slice check; cross-check upstream `libduckdb-windows-amd64.zip` as alternative source |
| (reference) upstream duckdb v1.5.5 | — | — | windows-amd64 + windows-arm64 + linux amd64/arm64 zips SHIPPED | — |

## Verdict

- Upstream: Windows prebuilts exist (v1.5.5 ships `libduckdb-windows-amd64.zip`).
- Our builder: Windows `.dll` naming is code-ready and unit-tested; the
  verification layer (slice check) is macOS-only and degrades silently, and
  no CI leg produces any non-macOS asset. The README claim
  ("Linux / Windows: pinned local-compile fallback", README.md:161;
  "Prebuilt Linux `.so` / Windows assets — planned, not built",
  README.md:193) is **accurate as of this audit** — PARTIAL_ALIVE for
  builder-code readiness, no prebuilt Windows or Linux asset produced.

Probe evidence: GitHub API v1.5.5 asset-name listing (exit 0, names quoted
above); extensions CDN `windows_amd64` wasm probe returned 404 (expected —
extension CDN layout differs; the release-API listing is the load-bearing
ground truth for libduckdb assets).
