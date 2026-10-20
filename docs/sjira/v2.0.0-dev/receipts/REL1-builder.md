# REL1 — builder-section re-witness receipts @ v2.0.0-dev

Lane: REL1 (verify+receipt), repo `/Users/sac/frozen-duckdb`, NO git state
mutations performed. Subject read at HEAD `dbac0b4`. All commands run
2026-10-19 this session, real output pasted. Lane build root:
`target-laneREL1` (removal denied by permission system — left for
coordinator cleanup, see "Residue").

## Claim 1 — cache normalization (headers + link-name restore)

**Law under test**: `ensure_binary` normalizes the cache layout after a
trusted hit — `ensure_headers` copies vendored `duckdb/duckdb.{h,hpp}` when
the headers dir is missing/half-staged (both-files check, lib.rs:452-473);
`ensure_link_name` recreates the `libduckdb.dylib` symlink when missing
(lib.rs:479-498). Both run on EVERY `ensure_binary` return path
(lib.rs:363-364).

**Method (5S-era scratch-HOME pattern)**: temp runner
`crates/frozen-duckdb-builder/examples/rel1_witness.rs` (deleted after the
run) called the real `frozen_duckdb_builder::ensure_binary()` against a
scratch `HOME=/tmp/rel1-frozen-duckdb/home` whose cache entry was seeded
from the real `~/.frozen-duckdb/cache/v2.0.0-dev-arm64/` with `duckdb/` and
`libduckdb.dylib` deleted:

```
SCRATCH entry BEFORE (missing both normalization subjects):
ENGINE_VERSION                       (10 bytes)
libduckdb_arm64.dylib                (65535744 bytes)
libduckdb_arm64.dylib.size           (9 bytes)
baseline sha256(libduckdb_arm64.dylib) =
  abaae2126ad8fba201ea60b93b63a532751c3dac3f160a01d9f636c2d8d87e78

env HOME=$SCRATCH RUSTUP_HOME=/Users/sac/.rustup CARGO_HOME=/Users/sac/.cargo \
  CARGO_TARGET_DIR=target-laneREL1 cargo run -q -p frozen-duckdb-builder --example rel1_witness
→ ENSURE_OK /tmp/rel1-frozen-duckdb/home/.frozen-duckdb/cache/v2.0.0-dev-arm64/libduckdb_arm64.dylib  (exit 0)

SCRATCH entry AFTER:
duckdb/                              (restored)
libduckdb.dylib -> …/libduckdb_arm64.dylib   (symlink restored)
ENGINE_VERSION, libduckdb_arm64.dylib, .size
```

**Restoration witnesses**:
- `duckdb/duckdb.h` sha256 byte-exact vs vendored, both
  `8dbc9c71a10f0a75fcacc1a3f4f7c76093a84d232fc152222ee5b6e512ab4560`
- `duckdb/duckdb.hpp` sha256 both
  `aab42f263dfdaef6547b550a3a62e00201df56844087b44a6f10676412258438`
- `libduckdb.dylib` symlink recreated, target
  `/tmp/rel1-frozen-duckdb/home/.frozen-duckdb/cache/v2.0.0-dev-arm64/libduckdb_arm64.dylib`
- dylib **length unchanged** (65535744) and `.size` pin still consistent;
  however sha256 **changed** from `abaae212…87e78` to
  `2da8e23d9283fbbc9a83d9e5dd6dd061b30215573a5aad6f9b2d2cf3125fb462` —
  observed effect of the KD-15 in-place install-ID stamp /
  re-sign path inside `accept_cached_entry` (lib.rs:441-443 re-stages the
  pin over stamped bytes). Constant-length, pin-consistent, so the trusted-hit
  law held; the byte-level mutation is recorded here rather than rounded up.

**Standing: ALIVE** — normalization restored both subjects on a real run.

## Claim 2 — vendored headers ship in the builder .crate

```
$ cargo package -q --no-verify --allow-dirty -p frozen-duckdb-builder --list
.cargo_vcs_info.json
Cargo.lock
Cargo.toml
Cargo.toml.orig
LICENSE-DUCKDB.md
README.md
src/lib.rs
vendored-headers/duckdb.h
vendored-headers/duckdb.hpp      ← both headers in the .crate (exit 0)
```

(First listing included `examples/rel1_witness.rs` — the temp witness runner,
since deleted; re-listed clean after removal.)

**Standing: ALIVE.**

## Claim 3 — ENGINE_VERSION marker + `.size` pin staged on every persist

All three persist sites stage pin-then-marker (marker LAST, DT1 order):
- prebuilt copy: lib.rs:546-555 (`copy_prebuilt_to_cache`)
- download: lib.rs:1062-1067 (`persist_verified_for`)
- local compile: lib.rs:1275-1276 (`compile_duckdb_locally`)

Real-output witness from the Claim 1 run's scratch entry after staging:
- `ENGINE_VERSION` content: `v2.0.0-dev` (10 bytes, byte-exact, no trailing
  newline — satisfies the C2 byte-exact label law, lib.rs:385-406)
- `libduckdb_arm64.dylib.size` content: `65535744` == actual dylib length
  (`stat -f %z` = 65535744), both before and after the run.

Also witnessed on the REAL cache: marker `v2.0.0-dev`, pin 65535744 ==
length. **Standing: ALIVE.**

## Claim 4 — automatic-download path: URL construction + 404 → compile fallback

URL builder (lib.rs:1093-1097) produces, for darwin/arm64 at VERSION
`2.0.0-dev` (lib.rs:17), asset name `libduckdb_arm64.dylib`
(`release_asset_name_for`, lib.rs:79-86):

```
https://github.com/seanchatmangpt/frozen-duckdb/releases/download/v2.0.0-dev/libduckdb_arm64.dylib
```

Live probe this session:

```
$ curl -s -o /dev/null -w "HTTP=%{http_code}\n" -L --max-time 30 "$URL"
HTTP=404
```

As documented at lib.rs:74-77 and TR7 comment (lib.rs:1088-1092): no
v2.0.0-dev release exists, so every download 404s and `ensure_binary` falls
to `compile_duckdb_locally`, which clones duckdb/duckdb and checks out
SOURCE_PIN `b6013f7aa2f36caabb9466ce1791667ea8a280bb` (lib.rs:20, checkout
at lib.rs:1172, loud log line lib.rs:1183). The 404 is witnessed live this
session; the compile leg itself is code-cited and its product exists in the
real cache (`~/.frozen-duckdb/cache/v2.0.0-dev-arm64/libduckdb_arm64.dylib`,
65535744 bytes, trusted marker) — a full recompile was not re-run this
session (hours-class; UNVERIFIED-this-session, previously ALIVE via the
existing cache).

**Standing: ALIVE** for URL + live 404; compile fallback PARTIAL (code path
cited + provenance cache present; no fresh compile executed).

## Verdict

| # | Claim | Standing |
|---|---|---|
| 1 | Cache normalization restores headers + symlink | ALIVE |
| 2 | Vendored headers in builder .crate | ALIVE |
| 3 | ENGINE_VERSION + `.size` pin on every persist | ALIVE |
| 4 | Download URL 404 → compile at SOURCE_PIN | ALIVE (404 live) / compile leg code-cited |

## Residue (coordinator)

- `target-laneREL1/` build root left in repo root (rm denied by permission
  system) — delete at integration per lane law.
- `/tmp/rel1-frozen-duckdb/` scratch HOME left (same denial) — rm at will.
- Working-tree diff from this lane: only this receipt file. No git commands
  beyond read-only `rev-parse`-class inspection.
