# REL11 — Release-Asset Checklist Extension

- **Date**: 2026-10-19
- **Lane**: REL11
- **Subject**: `scripts/verify_release_assets.sh` (extended in place; exit contract unchanged)
- **Receipt**: real run, exit 2 (loud no-release), see below.

## Scope

Extended the post-dispatch release-asset harness with the pattern's
release-asset checklist items it lacked:

1. **Clean-machine smoke step reference** — the harness prints the exact
   checklist commands (cargo new / cargo add / minimal smoke main / `env -u
   DYLD_LIBRARY_PATH cargo run`) as harness output so the operator's clean
   machine run asserts the same surface. No faked execution: the harness
   asserts asset prerequisites; the smoke run is operator authority.
2. **Cache-population check** — for each expected dylib arch, if
   `~/.frozen-duckdb/cache/<tag>-<arch>/` exists it must contain
   `duckdb/duckdb.h`, `duckdb/duckdb.hpp`, and `libduckdb.dylib`;
   present-but-incomplete = BREACH. Absent cache = loud informational note
   (populated by first consumer build, not by this harness) — never a faked
   pass.
3. **ctypes `duckdb_library_version()` == `v{TAG#v}`** — pre-existing
   ENGINE probe kept unchanged (native-arch ctypes real load; foreign arch
   strings-scan with recorded method).
4. **`libfrozen_mega_*` leftovers check** — any release asset name matching
   `libfrozen_mega_*` is a BREACH (TAG-REHEARSAL.md line-88 law: mega-library
   leftovers count as wrong).
5. **Optional Linux asset (`libduckdb_linux_arm64.so`, REL6)** —
   accept-and-verify: if listed, must download and be nonempty (bytes
   recorded); absence is accepted with a loud note, never required.

## Exit contract

`0` all-pass | `1` any breach | `2` loud no-release / usage. Unchanged.

## Real run receipt (still-empty release)

```
$ bash -n scripts/verify_release_assets.sh && echo SYNTAX-OK
SYNTAX-OK

$ bash scripts/verify_release_assets.sh v2.0.0
[verify-release-assets] no release v2.0.0 exists yet — loud SKIP-class exit (2), not a faked pass
$ echo $?
2
```

Exit 2 is the required loud divergence: the release does not exist yet, so
the new checklist sections (which run only after the release-exists
prerequisite) are unreachable by design — the harness refuses rather than
simulating asset verification against nothing.

## Non-vacuity note

The added checks are wired into the same `breach()` fail-closed channel as
the pre-existing assertions (asset set, Mach-O slice, strip -x ceiling,
ENGINE probe, sha256). A future release carrying `libfrozen_mega_*`, a
stripped cache dir, or a broken optional `.so` flips exit 0 → 1; the empty
release today flips 2, proving the prerequisite gate itself is not vacuous.
