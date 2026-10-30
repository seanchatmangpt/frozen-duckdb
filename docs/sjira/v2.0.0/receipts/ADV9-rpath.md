# ADV9 — KD-15 rpath fix (install-ID stamp at the cache persist layer)

- **Lane**: ADV9, repo `/Users/sac/frozen-duckdb`, branch `master`
  (uncommitted working tree shared with concurrent lanes; no git commands run).
- **Owned surface**: `crates/frozen-duckdb-builder/src/lib.rs`,
  `/tmp/adv9/consumer` (out-of-tree probe), this receipt.
- **Standing**: KD-15 **ALIVE (fixed and witnessed end to end)**.

## Root cause (confirmed, not inferred)

`otool -D` on the cached engine dylib, before the fix:

```
$ otool -D ~/.frozen-duckdb/cache/v2.0.0-arm64/libduckdb_arm64.dylib
/Users/sac/.frozen-duckdb/cache/v2.0.0-arm64/libduckdb_arm64.dylib:
@rpath/libduckdb.dylib
```

The dylib's Mach-O install ID is baked to `@rpath/libduckdb.dylib` by its
producer. macOS linkers record the *found library's install ID* as the
consumer's `LC_LOAD_DYLIB`, so every consumer binary carries
`@rpath/libduckdb.dylib` no matter how the linker found the library. The sys
build script's `cargo:rustc-link-arg=-Wl,-rpath,...` never propagates from a
dependency build script to a consumer's final link — reproduced:

```
$ otool -L /tmp/adv9/consumer/target/debug/adv9-consumer | grep duckdb
	@rpath/libduckdb.dylib (compatibility version 0.0.0, current version 0.0.0)
$ otool -l <bin> | grep -A2 LC_RPATH   →  NO LC_RPATH
$ env -u DYLD_LIBRARY_PATH ./target/debug/adv9-consumer
dyld[44490]: Library not loaded: @rpath/libduckdb.dylib
  Reason: no LC_RPATH's found
exit=134
```

## Mechanism experiments (all commands real, in /tmp/adv9)

1. **Dead ends confirmed**: `cargo:metadata` is not delivered downstream;
   `rustc-link-arg` from a dependency build script does not reach the
   consumer's link.
2. **What propagates**: `cargo:rustc-link-search=native={dir}` (already
   emitted by `frozen-duckdb-sys/build.rs:45`). The linker then finds
   `libduckdb.dylib` in the cache dir and — decisive — records the library's
   **install ID** as the load command. So the fix belongs at the source of the
   ID, not the consumer.
3. **Proof**: on a scratch copy of the real dylib
   (`/tmp/adv9/stamped/libduckdb.dylib`):
   `install_name_tool -id /tmp/adv9/stamped/libduckdb.dylib <file>` +
   `codesign -f -s -`, relink the consumer against it →
   `LC_LOAD_DYLIB` becomes the absolute path and a bare run exits 0.

## Fix (in `crates/frozen-duckdb-builder/src/lib.rs`)

New `ensure_install_id(path)` (macOS; non-macOS is a no-op returning
`Ok(false)`):

- Mach-O magic guard (reuses `is_macho_magic`): non-Mach-O payloads (unit-test
  fixtures) are skipped, not errors.
- Idempotence: `otool -D` equals `fs::canonicalize(path)` → no-op.
- `install_name_tool -id <abs-cache-path>`; on failure (dylib linked without
  `-headerpad_max_install_names` has no room to grow its LC_ID_DYLIB):
  **loud `warn!` + `Ok(false)`** — serving an unstamped library only re-opens
  KD-15, it must not brick the build. (Observed exactly this error on a
  minimal cc fixture: "larger updated load commands do not fit".)
- `codesign --force --sign -` re-sign, hard-error on failure (never serve a
  mutated, unresigned image).

Wired at four sites:

1. `copy_prebuilt_to_cache` — stamp BEFORE `stage_library_size_pin` so the
   `.size` pin covers the stamped bytes (stamp+re-sign change the length;
   measured 65,900,736 → 65,535,744 bytes on the real engine).
2. `persist_verified_for` — same order (download path).
3. `compile_duckdb_locally` — same order.
4. `accept_cached_entry` — **self-heal for pre-existing caches**: an entry
   written before this fix is still accepted, then stamped in place and its
   size pin re-staged. Second build is a no-op (otool -D idempotence check).

Also updated in-file courts for the new contract:

- `real_dylib` test fixture now compiles with
  `-Wl,-headerpad_max_install_names` so courts exercise the stamp path.
- `tr7_mismatched_download_never_reaches_the_cache` no longer asserts byte
  equality with the delivered payload (stamping legitimately changes bytes);
  it now asserts idempotence + `otool -D == canonical cache path`.

## Evidence (real output)

Builder tests after the change:

```
$ cargo test -p frozen-duckdb-builder
test result: ok. 43 passed; 0 failed; 0 ignored; 0 measured
```

End-to-end on the LIVE shared cache (`v2.0.0-arm64`), through the
out-of-tree consumer probe `/tmp/adv9/consumer` (links `frozen-duckdb-sys`
only; `crates/frozen-duckdb/Cargo.toml` was mid-edit by another lane —
`feature serde_json includes dep:serde_json, but serde_json is not an optional
dependency` — so the high-level crate was not consumable at probe time; the
sys crate is where the linking happens anyway):

```
$ otool -D ~/.frozen-duckdb/cache/v2.0.0-arm64/libduckdb_arm64.dylib
/Users/sac/.frozen-duckdb/cache/v2.0.0-arm64/libduckdb_arm64.dylib:
/Users/sac/.frozen-duckdb/cache/v2.0.0-arm64/libduckdb_arm64.dylib     ← stamped absolute ID
$ otool -L /tmp/adv9/consumer/target/debug/adv9-consumer | grep duckdb
	/Users/sac/.frozen-duckdb/cache/v2.0.0-arm64/libduckdb_arm64.dylib (compatibility version 0.0.0, current version 0.0.0)
$ codesign -v <cache dylib> && echo SIGN_OK
SIGN_OK
$ stat -f%z dylib; cat dylib.size
65535744 / 65535744        ← pin re-staged over stamped bytes
$ env -u DYLD_LIBRARY_PATH ./target/debug/adv9-consumer
CONSUMER_BARE_RUN_OK
exit=0
```

Second build (idempotence): no re-stamp, same bare run `exit=0`.

`cargo test -p frozen-duckdb-sys`: `89 passed; 0 failed` (no regression in the
stub-mode / link courts).

## Dead ends documented

- `cargo:metadata=...` — only visible to nothing downstream; dead end.
- `cargo:rustc-link-arg=-Wl,-rpath,...` from the sys build script — does not
  propagate to a consumer's final link (reproduced above: NO LC_RPATH).
- Consumer-side `.cargo/config.toml` rustflags (PB4 workaround) — works but is
  per-project manual setup; superseded by the stamp.

## Standing summary

| Claim | Standing | Evidence |
|---|---|---|
| KD-15 closed for new persists (download/compile/prebuilt copy) | ALIVE | builder tests 43/43; stamp wired before size pins |
| KD-15 closed for pre-existing caches | ALIVE | self-heal in `accept_cached_entry`; live cache stamped in place, pin re-staged |
| Consumer bare run, no rpath, no env | ALIVE | `CONSUMER_BARE_RUN_OK`, exit 0, real otool output above |
| Linux | UNVERIFIED | ELF has no LC_ID_DYLIB; stamp is macOS-only no-op; KD-15 is a Mach-O/@rpath defect |
| Other arch caches (v1.4/v1.5.5 entries) | UNVERIFIED | will self-heal on next accept via this code |
| GitHub Release assets absent | pre-existing, unchanged | tag cut, no assets; local prebuilt path served the live cache |

## Probe artifacts

- Consumer scratch: `/tmp/adv9/consumer` (Cargo.toml + src/main.rs, opens and
  closes a real in-memory DuckDB handle via frozen-duckdb-sys).
- Scratch stamped copy: `/tmp/adv9/stamped/` (mechanism-proof artifact).
