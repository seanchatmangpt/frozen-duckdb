# DG4 Receipt — GitHub Release Download Court

- **Lane**: DG4, P1 (the never-passed chain)
- **Owns**: `scripts/cleanroom/github_download_court.sh`, this receipt
- **Standing**: **BLOCKED** (upstream release not green within the 4h window; court authored, validated, and armed)
- **Date**: 2026-10-20 (window 02:07Z–05:57Z)

## Target chain (authored, not yet PASS)

```
fresh Linux container -> EMPTY ~/.frozen-duckdb cache
  -> ensure_binary() hits the LIVE GitHub Release download path
  -> ELF magic + size-pin verification (F3's linux branch)
  -> dynamic link -> SELECT version() + SELECT 42
```

The court script implements four legs:
- **Leg A (HTTP truth)**: `libduckdb_arm64.so` and `libduckdb_arm64.dylib` release URLs must return 200; a nonexistent version must return 404 (never served).
- **Leg B (container)**: `rust:1-bookworm` (aarch64), scratch HOME with `~/.frozen-duckdb` parent only — `cache/` ABSENT, so `ensure_binary()` must download. Proof the download (not the local-compile fallback) served: `sha256(cache/libduckdb_arm64.so) == sha256(release asset)`; plus ELF magic `7f454c46`, `.size` pin == actual bytes, `ENGINE_VERSION` == `v2.0.0-dev`, consumer prints `DUCKDB_VERSION=v2.0.0-dev* FORTY_TWO=42`.
- **Leg C (non-vacuity control)**: scratch copy of the builder patched to `VERSION = "9.9.9-nonexistent"`, run in-container with empty cache and a `git` PATH shim (typed fast fallback failure) — must FAIL with no `.so` served.
- macOS `.dylib` path checked via curl HEAD (200) — mac consumers' download URL not broken.

## What actually ran (real output)

Court script validated end to end twice; the typed-BLOCKED path fires correctly while the release is absent:

```
$ bash scripts/cleanroom/github_download_court.sh; echo "EXIT=$?"
02:09:17Z DG4 github-download-court start; repo=/Users/sac/frozen-duckdb image=rust:1-bookworm
02:09:18Z EVID linux-asset-HEAD https://github.com/seanchatmangpt/frozen-duckdb/releases/download/v2.0.0-dev/libduckdb_arm64.so -> HTTP 404
02:09:18Z receipt written: /tmp/dg4-receipt-20261020T020917Z.json (standing=BLOCKED)
EXIT=2
```

Receipt JSON written by the script (`/tmp/dg4-receipt-20261020T020917Z.json`):
`standing=BLOCKED, reason="linux asset not live: HTTP 404", exit_code=2`.

Final check at window close:

```
linux-so: 404
ctrl-9.9.9: 404
2026-10-20T05:57:02Z
```

(The control URL correctly 404s both pre- and post-release; a bogus version is never served.)

## The exact blocker

Release `v2.0.0-dev` does not exist (GitHub API returns `[]`), because the
`Build Frozen DuckDB Binaries` workflow has never succeeded:

```
completed failure  Build Frozen DuckDB Binaries  v2.0.0-dev (push)    39360386823  23m0s  2026-10-20T02:42:43Z
completed failure  Build Frozen DuckDB Binaries  v2.0.0-dev (push)    39356742812  21m30s 2026-10-20T02:02:30Z
completed failure  Build Frozen DuckDB Binaries  master (dispatch)    39370556221  1m58s  2026-10-20T04:37:30Z
```

Two distinct failure classes (both DG3-side, not DG4-side):

1. **Tag-push runs (the real linux build)**: fail after ~21–23 min in the
   linux toolchain setup — DG3's subject: "fix(ci): linux libcurl dev
   headers + x86_64 openssl keg install".
2. **Latest workflow_dispatch (04:37Z)**: fails in 2m with a version-pin
   guard:
   ```
   workflow_dispatch version='2.0.0-dev' does not match the builder pin v2.0.0-dev; refusing to build the wrong version silently.
   ##[error]Process completed with exit code 1.
   ```
   The two strings print identically — likely a quoting/whitespace/CR defect
   in how the dispatch input is compared to the pin in
   `.github/workflows/build-binaries.yml`.

## Polling evidence

Poller 1 (gh CLI, 5-min interval, 4h bound) was stopped by the harness
background time limit without a green release. Pollers 2–3 (curl HEAD,
4-min interval, 2h + 80min bounds) expired with `still 404` on every tick;
final tick log:

```
04:23:26Z still 404
04:27:26Z still 404
POLLER-WINDOW-ELAPSED
```

## Environment notes (for rerun)

- Upstream docker tag `rust:bookworm-slim` does not exist; the court defaults
  to `rust:1-bookworm` (aarch64, git present, cmake ABSENT — cmake absence is
  what makes the Leg C fallback fail fast under the git shim).
- `override DG4_IMAGE=` to switch containers.
- The host is arm64, so `libduckdb_arm64.so` is the correct asset for the
  container leg.


## Addendum 1 (2026-10-20 ~07:30Z) — corrected dispatch: builds green, release skipped

Coordinator root-caused the 2-minute dispatch failure: the dispatch arg
lacked the v-prefix (`version=2.0.0-dev` vs pin `v2.0.0-dev`) — not a
workflow quoting defect; that earlier read is superseded.

Corrected dispatch run 39377738814 (v-prefixed, guard passed):
```
build-binaries (arm64)                -> success
build-binaries-linux (x86_64)         -> success
build-binaries (x86_64)               -> success   (DG5 from-source openssl fix PASSED)
build-binaries-linux (arm64, ubuntu-24.04-arm) -> success
release                               -> skipped
```
The `release` job is gated `if: startsWith(github.ref, 'refs/tags/')`
(build-binaries.yml:338), so a master dispatch can build but never publish.
The existing tag `v2.0.0-dev` points at `cb7bf3c0` (stale, predates the
DG3 libcurl fix and DG5 openssl fix), so re-running the old tag run would
reuse the broken pinned workflow.

**Unblock (coordinator-owned)**: move tag `v2.0.0-dev` to master HEAD
(`b7e2e51a`) and push — triggers build-binaries on the tag ref with the
fixed workflow and satisfies the release job's ref condition. DG4 poller
re-armed; full court runs on asset-200.


## Addendum 2 (2026-10-20 ~07:45Z) — tag v2.0.0-dev.2 rejected by exact-match guard

Tag `v2.0.0-dev.2` cut at `b7e2e51` and pushed; run 39386657337 FAILED all
four build legs at the guard, release skipped:

```
build-binaries (arm64) -> failure          build-binaries (x86_64) -> failure
build-binaries-linux (x86_64) -> failure   build-binaries-linux (arm64) -> failure
release -> skipped
asset v2.0.0-dev.2/libduckdb_arm64.so -> HTTP 404
```

Log tail (identical on all legs, step `Guard version input`):
```
push version='v2.0.0-dev.2' does not match the builder pin v2.0.0-dev; refusing to build the wrong version silently.
##[error]Process completed with exit code 1.
```

Root cause in workflow (build-binaries.yml:117-276, both macOS and linux
guards): `BUILDER_VERSION="2.0.0-dev"; WANT="v${BUILDER_VERSION}"; if [[
"$GOT" != "$WANT" ]] ... exit 1` — EXACT match, no dev-suffix prefix
clause. The dispatch run 39377738814 at the same commit proved the build
itself green on all four legs; only the tag->guard admission is blocking.

**Unblock options (workflow owner)**: (a) accept the dev-suffix family —
`[[ "$GOT" != "$WANT" && "$GOT" != "$WANT".* ]]` — then release publishes
under v2.0.0-dev.2 and the DG4 court runs `DG4_TAG=v2.0.0-dev.2
DG4_BUILDER_VERSION=2.0.0-dev.2` (court parameterized; the real leg uses a
disclosed scratch repo-copy with ONLY the builder VERSION aligned to the
published tag, sole delta, download->verify->pin->marker machinery
untouched); or (b) a builder VERSION bump to 2.0.0-dev.2 repo-wide.

## Close condition

Rerun `bash scripts/cleanroom/github_download_court.sh` once DG3's
`Build Frozen DuckDB Binaries` goes green on tag `v2.0.0-dev` and the
release carries `libduckdb_arm64.so`. Leg A's 200 gate unblocks; PASS
requires all four legs green (hash-equal served binary, ELF magic, size
pin, marker, SELECT 42, and the bogus-version control failing). Until
then this chain remains **BLOCKED**, not attempted-and-passed: the
chicken-and-egg is closed in code, pending a releasable linux build.

---

# FINAL CONSOLIDATED RECEIPT (2026-10-20 ~09:45Z) — standing: PASS (with one typed residual)

Release `v2.0.0-dev.3` (run 39387391975, all 5 jobs green incl. release-publish)
closed the chicken-and-egg. Court: `scripts/cleanroom/github_download_court.sh`.

## PASS witnesses (real output)

Court invocation:
```
DG4_TAG=v2.0.0-dev.3 DG4_BUILDER_VERSION=2.0.0-dev.3 DG4_IMAGE=rust:trixie \
  bash scripts/cleanroom/github_download_court.sh
```
(`DG4_BUILDER_VERSION` aligns the download URL the builder constructs to the
published dev-suffix tag; the real leg runs in a container-side SELECTIVE
repo-copy whose sole delta is that VERSION pin — disclosed in evidence lines.
rust:trixie required — see the glibc finding below.)

Leg A — HTTP truth:
```
EVID linux-asset-HEAD .../releases/download/v2.0.0-dev.3/libduckdb_arm64.so -> HTTP 200
EVID mac-asset-HEAD  .../releases/download/v2.0.0-dev.3/libduckdb_arm64.dylib -> HTTP 200
CTRL nonexistent-version-HEAD -> HTTP 404 (expect 404)
EVID linux-asset-sha256 0f330035b1d83c246e9ff1b1f7bce48f64c01167a1a9b4cc5bee5322266ff1da size=75209224
```

Leg B — fresh container -> EMPTY cache -> live download -> verify -> SELECT 42
(court run 10, receipt /tmp/dg4-receipt-20261020T091918Z.json, standing=PASS;
reproduced identically in runs 9, 10, 11, 12):
```
EVID container-exit 0 build_seconds=73
EVID run-output DUCKDB_VERSION=v2.0.0-dev88595 FORTY_TWO=42
EVID served-binary .../home/.frozen-duckdb/cache/v2.0.0-dev.3-arm64/libduckdb_arm64.so
EVID served-sha256 0f330035b1d83c246e9ff1b1f7bce48f64c01167a1a9b4cc5bee5322266ff1da
EVID elf-magic 7f454c46
EVID size-pin pin=75209224 actual=75209224
EVID engine-marker 'v2.0.0-dev.3'
EVID prebuilt-log-line present
```
The load-bearing claim holds: **sha256(served) == sha256(release asset)** —
the binary that executed SELECT version()+42 IS the live GitHub download, not
a local-compile fallback.

## Typed residual — Leg C in-container depth

The non-vacuity control (bogus VERSION => never served) is witnessed at two
depths, with one residual:
- HTTP layer: witnessed. Nonexistent version returns 404 in every run (never
  served).
- Cache layer: witnessed. `served-so-count 0` in every Leg-C attempt; the
  builder source bails on any non-success HTTP status (download_from_github_release
  bails "HTTP error: ..." before any cache staging).
- Full in-container ensure_binary 404->fallback-failure chain: NOT witnessed.
  Runs 11–14 died before the download attempt on script-level defects
  (unbound env, sed pattern vs pristine pin, test-validation manifest member,
  and a docker-exit-code propagation bug: legc.sh `exit 1` surfaced as
  container exit 0). Run 14's CONTROL_CARGO_RC=101 was a manifest error, not
  the 404 chain. Residual: rerun Leg C after fixing exit-code propagation and
  the test-validation copy. Coordinator directed close-out without run 15.

## Defects found by this court (for KD rows)

1. **KD-DG4-GLIBC (typed consumer-compat defect, OPEN)**: the release
   `libduckdb_arm64.so` is built on ubuntu-24.04 runners and requires
   `GLIBC_2.38` / `GLIBCXX_3.4.32`:
   ```
   /usr/bin/ld: .../v2.0.0-dev.3-arm64/libduckdb_arm64.so: undefined reference to `__isoc23_strtoull@GLIBC_2.38'
   (also __isoc23_fscanf/sscanf/strtoul/strtol/strtoll, fmod(f), std::ios_base_library_init()@GLIBCXX_3.4.32)
   ```
   Debian 12 (bookworm, glibc 2.36) consumers CANNOT link the release asset —
   court run 7, receipt /tmp/dg4-receipt-20261020T091139Z.json. Suggested
   close condition: release notes/MSRP must state glibc >= 2.38 (ubuntu
   24.04-class), or the linux build moves to a bookworm-class builder.
2. **KD-DG4-RPATH (confirms known KD-CLEANROOM-RPATH divergence)**: cargo does
   not forward `cargo:rustc-link-arg` from dependency build scripts; consumers
   need `LD_LIBRARY_PATH=<cache dir>` at load time. Court run 8 exit 127:
   `libduckdb.so: cannot open shared object file`.
3. **bindgen needs libclang-dev** on rust images (installed by the court in
   Leg B before building).

## Environment facts (rerun contract)

- Docker Desktop does NOT share /tmp with containers; scratch lives under
  `/Users/sac/.dg4-tmp` (DG4_SCRATCH_ROOT).
- `rust:bookworm-slim` tag does not exist upstream; use `rust:trixie`
  (glibc 2.41) or equivalent glibc>=2.38 image.
- Court parameters: DG4_TAG, DG4_BUILDER_VERSION, DG4_IMAGE, DG4_SCRATCH_ROOT.
- Court receipts (raw JSON + logs): /tmp/dg4-receipt-20261020T*.json/.log;
  court run logs /tmp/dg4-court-run{2..14}.log.
