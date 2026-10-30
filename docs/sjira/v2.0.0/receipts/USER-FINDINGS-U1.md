# USER-FINDINGS-U1 — cold-start newbie lane (U1)

Subject: frozen-duckdb cold-start experience, fresh HOME (`/tmp/u1/home`, no
`~/.frozen-duckdb` cache pre-seed), toolchain env preserved
(`RUSTUP_HOME`/`CARGO_HOME` at the host's — a synthetic-HOME artifact, not a
product finding). Repo HEAD at time of run: `d73d017cb59018cd59db7917604da4f5ac18b2eb`.
Date: 2026-10-19. Lane ownership: `/tmp/u1` scratch + README/QUICKSTART fixes + this receipt.

## Findings

### F1 — BLOCKER: crates.io has only 0.1.0; `cargo add` silently installs the wrong crate

Real output (`cargo new demo && cargo add frozen-duckdb`, fresh HOME):

```
    Updating crates.io index
      Adding frozen-duckdb v0.1.0 to dependencies
     Locking 282 packages to latest Rust 1.97.0 resulting compatible versions
```

Exit 0. No warning. Cargo.toml got `frozen-duckdb = "0.1.0"`. A newbie follows
the old README (`frozen-duckdb = "2.0.0"`) — a version string that cannot
resolve on crates.io — and the tool's "successful" 0.1.0 add actively launders
the failure. Both README and QUICKSTART previously told the user to write
`frozen-duckdb = "2.0.0"` — unresolvable. Fixed in both (see F5/F6).

### F2 — BLOCKER: git dependency is broken — `vendors/duckdb-rs` gitlink with no `.gitmodules`

```
$ cargo add frozen-duckdb --git https://github.com/seanchatmangpt/frozen-duckdb
    Updating git repository `https://github.com/seanchatmangpt/frozen-duckdb`
error: failed to load source for dependency `frozen-duckdb
```

Exact error (with `--rev 0a205b7` too): `no URL configured for submodule
'vendors/duckdb-rs'; class=Submodule (17)`. Confirmed root cause on the tree:

```
$ git ls-tree HEAD vendors/
160000 commit f4fe688a8718dd1645f41f4eef336d5c8dd0738f	vendors/duckdb-rs
$ git config -f .gitmodules --list
fatal: unable to read config file '.gitmodules': No such file or directory
```

This kills the documented git-dep alternative AND makes cargo fail where plain
`git clone` only warns. Fix is coordinator-owned: add `.gitmodules` with a URL
for `vendors/co_rust` (or drop the gitlink for a subtree). Not done in this
lane (outside file ownership; shared infra).

### F3 — MINOR: fresh clone leaves `vendors/duckdb-rs` empty but it is harmless

Plain `git clone --depth 1` succeeds (3211 files). `vendors/duckdb-rs/` is
empty in the clone; the crate still builds — `frozen-duckdb-builder` vendors
its own headers (`crates/frozen-duckdb-builder/vendored-headers/`), the
gitlink directory is not on the build path. Verified: path dep from the fresh
clone resolves and compiles the three crates (EXIT=0 through `Compiling
frozen-duckdb-sys`). Should still be cleaned up (F2) but is not a wall today.

### F4 — MAJOR: first build on empty cache takes the 404 → local-compile fallback silently

Real output, first build, fresh HOME (empty `~/.frozen-duckdb`):

- GitHub release asset probe: `curl -sI .../releases/download/v2.0.0/libduckdb_arm64.dylib -L` → **HTTP 404** (real, this session).
- The build entered the fallback: `~/.frozen-duckdb/cache/v2.0.0-arm64/`
  contained only `.build-lock` (dir created + lock minted, no dylib after a
  3-minute bounded build; bounded per lane instructions, fallback NOT awaited).
- At default verbosity the newbie sees only ordinary `Compiling …` lines; the
  builder's `warn!`/`info!` lines do not print without a logger init or `-vv`;
  the 30-90 min fallback starts with no announcement, no progress, no ETA.
  (Receipt fix landed in QUICKSTART: the first-build warning.)

Also observed (environment, not product): a concurrent lane's `doctor` battery
was running against `/Users/sac/.frozen-duckdb` during my bounded builds — did
not touch my lane's isolated HOME cache.

## Fixes landed (this lane)

- `README.md` Quick Start now leads with the prerelease warning and the working
  install path (clone + `cargo add --path`, pinned SHA `d73d017…`), documents the
  `--git` failure mode explicitly, and says the crates.io `2.0.0` form
  returns only after publish.
- `QUICKSTART.md` section 3 carries the same prerelease warning; first-build
  paragraph now states the 404 → local-compile fallback, its duration, and its
  silence at default verbosity.

## Open items for the coordinator

1. **F2 fix**: add `.gitmodules` (url for `vendors/duckdb-rs`, branch main) or
   convert the gitlink to a subtree — unblocks `--git`/`--rev` deps and clean
   submodule init.
2. **F4 fix**: init a logger in `frozen-duckdb-sys` build.rs (or print to stderr
   unconditionally) so the fallback announces itself; consider printing the
   remaining-time expectation ("local compile, ~30-90 min").
3. **Publish** `2.0.0` to crates.io or yank/annotate 0.1.0 so `cargo add`
   stops silently installing the wrong generation.
