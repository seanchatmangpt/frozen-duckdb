#!/usr/bin/env bash
# github_download_court.sh — DG4: the never-passed chain, closed end to end.
#
#   fresh Linux container -> EMPTY ~/.frozen-duckdb cache -> ensure_binary()
#   hits the LIVE GitHub Release download path -> ELF magic + size-pin
#   verification (F3's linux branch) -> dynamic link -> SELECT version() +
#   SELECT 42.
#
# Legs:
#   A. HTTP truth: release asset URLs (linux .so, macOS .dylib) return 200;
#      a nonexistent version returns 404 (never served).
#   B. Container leg (rust:bookworm-slim, arm64): consumer crate with path
#      deps into the repo; scratch HOME whose ~/.frozen-duckdb is EMPTY so
#      ensure_binary MUST take the download path; assert on-disk cache
#      artifacts (ELF magic, .size pin, ENGINE_VERSION marker) and
#      sha256(served .so) == sha256(release asset) — a local-compile
#      fallback cannot be byte-identical to the release build, so hash
#      equality is the proof the download path (not the fallback) served.
#   C. Non-vacuity control (in-container): a scratch copy of the builder
#      with VERSION patched to a nonexistent version must FAIL to serve any
#      binary (404 -> fallback failure -> build error, no .so in cache).
#
# Receipt: docs/sjira/v2.0.0/receipts/DG4-download-court.md is written by
# the lane from this script's real output; this script also writes
# /tmp/dg4-receipt-<ts>.json + .log.
#
# Exit codes: 0 PASS · 1 FAIL · 2 typed BLOCKED/SKIP
set -euo pipefail

REPO="${CLEANROOM_REPO:-/Users/sac/frozen-duckdb}"
OWNER_REPO="seanchatmangpt/frozen-duckdb"
# DG4_TAG: release tag under which assets are published (asset URLs).
# DG4_BUILDER_VERSION: the builder's VERSION pin the download URL is keyed
# from. The repo's builder pins "2.0.0"; when the live release tag is a
# dev-suffix (e.g. v2.0.0), the real leg runs against a SCRATCH COPY of
# the repo whose builder VERSION is sed-aligned to DG4_BUILDER_VERSION —
# the only delta; the download->verify->pin->marker machinery is untouched.
# Disclosed in the receipt whenever != the repo pin.
TAG="${DG4_TAG:-v2.0.0}"
BUILDER_VERSION="${DG4_BUILDER_VERSION:-2.0.0}"
ASSET_LINUX="libduckdb_arm64.so"
ASSET_MAC="libduckdb_arm64.dylib"
BASE_URL="https://github.com/${OWNER_REPO}/releases/download/${TAG}"
TS="$(date -u +%Y%m%dT%H%M%SZ)"
RECEIPT="${DG4_RECEIPT:-/tmp/dg4-receipt-${TS}.json}"
LOG="${RECEIPT%.json}.log"
# note: the upstream tag rust:bookworm-slim does not exist; rust:1-bookworm
# is the fresh-container image (aarch64, git present, cmake ABSENT)
IMAGE="${DG4_IMAGE:-rust:1-bookworm}"
: >"$LOG"
log() { printf '%s %s\n' "$(date -u +%H:%M:%SZ)" "$*" | tee -a "$LOG"; }

STATUS="FAIL"; REASON="incomplete"
declare -a EVIDENCE=()
note() { EVIDENCE+=("$1"); log "$1"; }
finish() { # standing reason exit_code
  STATUS="$1"; REASON="$2"; local exit_code="$3"
  python3 - "$RECEIPT" "$STATUS" "$REASON" "$exit_code" "$LOG" <<'PY'
import json, sys, datetime
receipt, standing, reason, exit_code, log_path = sys.argv[1:6]
evidence = [l.rstrip("\n") for l in open(log_path)
            if l.startswith(("PASS ", "EVID ", "CTRL ", "SKIP ", "FAIL "))]
json.dump({
    "court": "dg4-github-download-court",
    "standing": standing,
    "reason": reason,
    "exit_code": int(exit_code),
    "repo": "/Users/sac/frozen-duckdb",
    "tag": __import__("os").environ.get("DG4_TAG", "v2.0.0"),
    "builder_version": __import__("os").environ.get("DG4_BUILDER_VERSION", "2.0.0"),
    "asset_linux": "libduckdb_arm64.so",
    "asset_mac": "libduckdb_arm64.dylib",
    "evidence": evidence,
    "log": log_path,
    "finished": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
}, open(receipt, "w"), indent=2)
PY
  log "receipt written: $RECEIPT (standing=$STATUS)"
  exit "$exit_code"
}

command -v docker >/dev/null || finish "SKIP" "docker not on PATH" 2
command -v curl >/dev/null || finish "SKIP" "curl not on PATH" 2
command -v shasum >/dev/null || finish "SKIP" "shasum not on PATH" 2

log "DG4 github-download-court start; repo=$REPO image=$IMAGE"

# ---------------------------------------------------------------------------
# Leg A — HTTP truth
# ---------------------------------------------------------------------------
LINUX_URL="${BASE_URL}/${ASSET_LINUX}"
MAC_URL="${BASE_URL}/${ASSET_MAC}"

LINUX_CODE="$(curl -sIL -o /dev/null -w '%{http_code}' "$LINUX_URL")"
note "EVID linux-asset-HEAD ${LINUX_URL} -> HTTP ${LINUX_CODE}"
[ "$LINUX_CODE" = "200" ] || finish "BLOCKED" "linux asset not live: HTTP $LINUX_CODE" 2

MAC_CODE="$(curl -sIL -o /dev/null -w '%{http_code}' "$MAC_URL")"
note "EVID mac-asset-HEAD ${MAC_URL} -> HTTP ${MAC_CODE}"
[ "$MAC_CODE" = "200" ] || { note "FAIL mac dylib HEAD $MAC_CODE"; finish "FAIL" "mac dylib asset HEAD = $MAC_CODE" 1; }

CTRL_CODE="$(curl -sIL -o /dev/null -w '%{http_code}' \
  "https://github.com/${OWNER_REPO}/releases/download/v9.9.9-nonexistent/${ASSET_LINUX}")"
note "CTRL nonexistent-version-HEAD -> HTTP ${CTRL_CODE} (expect 404)"
[ "$CTRL_CODE" = "404" ] || finish "FAIL" "nonexistent version returned HTTP $CTRL_CODE, expected 404" 1

ASSET_SHA="$(curl -sL "$LINUX_URL" | shasum -a 256 | awk '{print $1}')"
ASSET_SIZE="$(curl -sIL "$LINUX_URL" | awk 'tolower($1)=="content-length:"{gsub("\r","");print $2}' | tail -1)"
note "EVID linux-asset-sha256 ${ASSET_SHA} size=${ASSET_SIZE:-unknown}"
[ -n "$ASSET_SHA" ] || finish "FAIL" "could not hash release asset" 1

# ---------------------------------------------------------------------------
# Leg B — fresh container, empty cache, real download path
# ---------------------------------------------------------------------------
# Docker Desktop does NOT share /tmp with containers (bind mounts of /tmp
# dirs come up EMPTY inside); /Users IS shared. Scratch must live under /Users.
SCRATCH_ROOT="${DG4_SCRATCH_ROOT:-/Users/sac/.dg4-tmp}"
mkdir -p "$SCRATCH_ROOT"
SCRATCH="$(mktemp -d "$SCRATCH_ROOT/dg4-court-XXXXXX")"
note "EVID scratch ${SCRATCH}"
W=/work
mkdir -p "$SCRATCH/home/.frozen-duckdb"   # parent only; cache/ left ABSENT
# Dev-suffix release: the repo's builder pins "2.0.0", so the download
# URL it constructs must be aligned to the published tag. Done CONTAINER-SIDE
# as a SELECTIVE copy (root manifests + crates/ + test-validation/Cargo.toml
# — the full path-dep closure, NOT target/ etc.) with ONLY the builder
# VERSION sed-aligned; sole delta, disclosed in evidence. REPO_COPY=1 is
# passed into the container; consumer Cargo.toml is repointed there.
REPO_COPY=0
[ "$BUILDER_VERSION" != "2.0.0" ] && REPO_COPY=1
note "EVID builder-version ${BUILDER_VERSION} repo_copy=${REPO_COPY}"

cat >"$SCRATCH/consumer-Cargo.toml" <<EOF
[package]
name = "dg4-consumer"
version = "0.1.0"
edition = 2021

[dependencies]
frozen-duckdb = { path = "/repo/crates/f-duckdb" }

[workspace]
EOF
# fix the two heredoc-unsafe strings via sed (path + edition quoting)
sed -i '' -e 's#/repo/crates/f-duckdb#/repo/crates/frozen-duckdb#' \
          -e 's/^edition = 2021$/edition = "2021"/' "$SCRATCH/consumer-Cargo.toml"

cat >"$SCRATCH/consumer-main.rs" <<'EOF'
use frozen_duckdb::Connection;

fn main() {
    let conn = Connection::open_in_memory().expect("open in-memory db");
    let version: String = conn
        .query_row("SELECT version()", [], |row| row.get(0))
        .expect("SELECT version()");
    let forty_two: i32 = conn
        .query_row("SELECT 42", [], |row| row.get(0))
        .expect("SELECT 42");
    println!("DUCKDB_VERSION={} FORTY_TWO={}", version, forty_two);
    assert!(version.starts_with("v2.0.0"), "unexpected version: {}", version);
    assert_eq!(forty_two, 42);
}
EOF

BUILD_SECONDS=-1
log "launching container ($IMAGE): consumer build + run"
START="$(date +%s)"
set +e
docker run --rm \
  -v "$REPO":/repo:ro \
  -v "$SCRATCH":$W \
  -e HOME=$W/home \
  -e REPO_COPY=${REPO_COPY} \
  -e BUILDER_VERSION=${BUILDER_VERSION} \
  -e CARGO_TARGET_DIR=$W/target \
  -e CARGO_HOME=$W/cargo-home \
  -w $W \
  "$IMAGE" \
  bash -euo pipefail -c '
    set -x
    # bindgen needs libclang; rust:1-bookworm does not ship it
    apt-get update -qq && apt-get install -y -qq libclang-dev >/dev/null
    echo "--- EMPTY cache proof ---"
    ls -la /work/home/.frozen-duckdb; ls /work/home/.frozen-duckdb/cache 2>&1 || true
    CONSUMER_PATH="/repo/crates/frozen-duckdb"
    if [ "${REPO_COPY}" = "1" ]; then
      mkdir -p /work/repo-copy/test-validation
      cp /repo/Cargo.toml /repo/Cargo.lock /work/repo-copy/
      cp -r /repo/crates /work/repo-copy/crates
      cp -r /repo/test-validation /work/repo-copy/test-validation
      sed -i "s/const VERSION: \&str = \"2.0.0\";/const VERSION: \&str = \"${BUILDER_VERSION}\";/" \
        /work/repo-copy/crates/frozen-duckdb-builder/src/lib.rs
      grep -n "const VERSION" /work/repo-copy/crates/frozen-duckdb-builder/src/lib.rs
      CONSUMER_PATH="/work/repo-copy/crates/frozen-duckdb"
    fi
    mkdir -p /work/consumer/src
    cp /work/consumer-Cargo.toml /work/consumer/Cargo.toml
    sed -i "s#/repo/crates/frozen-duckdb#${CONSUMER_PATH}#" /work/consumer/Cargo.toml
    grep -n "path =" /work/consumer/Cargo.toml
    cp /work/consumer-main.rs  /work/consumer/src/main.rs
    echo "--- cargo build (ensure_binary runs here) ---"
    time cargo build --manifest-path /work/consumer/Cargo.toml 2>&1 | tee /work/build.log
    echo "--- cache artifacts after ensure_binary ---"
    find /work/home/.frozen-duckdb -type f -printf "%s %p\n" | sort -k2
    echo "--- dynamic link + SELECT version() + SELECT 42 ---"
    # KD-CLEANROOM-RPATH: cargo does not forward cargo:rustc-link-arg from
    # dependency build scripts, so the consumer has no baked rpath; load-time
    # resolution uses LD_LIBRARY_PATH at the cache dir (documented contract).
    CACHE_DIR=$(dirname "$(find /work/home/.frozen-duckdb -name 'libduckdb_*.so' | head -1)")
    LD_LIBRARY_PATH="$CACHE_DIR" /work/target/debug/dg4-consumer | tee /work/run.log
    echo "--- served binary forensics ---"
    SO=$(find /work/home/.frozen-duckdb -name "libduckdb_*.so" | head -1)
    echo "served: $SO"
    sha256sum "$SO"
    echo "ELF-magic: $(head -c4 "$SO" | od -An -tx1)"
    echo "size-pin:  $(cat "$SO.size" 2>/dev/null || echo MISSING)"
    echo "marker:    $(cat "$(dirname "$SO")/ENGINE_VERSION" 2>/dev/null || echo MISSING)"
  '
DOCKER_EXIT=$?
set -e
BUILD_SECONDS=$(( $(date +%s) - START ))
note "EVID container-exit ${DOCKER_EXIT} build_seconds=${BUILD_SECONDS}"
if [ "$DOCKER_EXIT" != "0" ]; then
  tail -40 "$SCRATCH/build.log" 2>/dev/null | while IFS= read -r l; do note "FAIL build-log: $l"; done
  finish "FAIL" "container leg exited $DOCKER_EXIT" 1
fi

RUN_LINE="$(grep '^DUCKDB_VERSION=' "$SCRATCH/run.log" || true)"
note "EVID run-output ${RUN_LINE:-<missing>}"
echo "$RUN_LINE" | grep -q "FORTY_TWO=42" || finish "FAIL" "SELECT 42 wrong: $RUN_LINE" 1
echo "$RUN_LINE" | grep -q "DUCKDB_VERSION=v2.0.0" || finish "FAIL" "version mismatch: $RUN_LINE" 1

SO="$(find "$SCRATCH/home/.frozen-duckdb" -name 'libduckdb_*.so' -type f | head -1)"
[ -n "$SO" ] || finish "FAIL" "no libduckdb_*.so staged in scratch cache" 1
note "EVID served-binary ${SO}"

SERVED_SHA="$(shasum -a 256 "$SO" | awk '{print $1}')"
note "EVID served-sha256 ${SERVED_SHA}"
[ "$SERVED_SHA" = "$ASSET_SHA" ] || finish "FAIL" "served sha256 $SERVED_SHA != release asset sha256 $ASSET_SHA" 1

ELF_MAGIC="$(head -c4 "$SO" | od -An -tx1 | tr -d ' \n')"
note "EVID elf-magic ${ELF_MAGIC}"
[ "$ELF_MAGIC" = "7f454c46" ] || finish "FAIL" "ELF magic absent: $ELF_MAGIC" 1

PIN="$(cat "$SO.size" 2>/dev/null || true)"
ACTUAL="$(stat -f%z "$SO")"
note "EVID size-pin pin=${PIN:-MISSING} actual=${ACTUAL}"
[ -n "$PIN" ] && [ "$PIN" = "$ACTUAL" ] || finish "FAIL" "size pin ${PIN:-MISSING} != actual $ACTUAL" 1

MARKER="$(cat "$(dirname "$SO")/ENGINE_VERSION" 2>/dev/null | tr -d '\n' || true)"
note "EVID engine-marker '${MARKER}'"
[ "$MARKER" = "v${BUILDER_VERSION}" ] || finish "FAIL" "ENGINE_VERSION marker '$MARKER' != v${BUILDER_VERSION}" 1

grep -q "Using prebuilt DuckDB binary" "$SCRATCH/build.log" || \
  finish "FAIL" "build log lacks 'Using prebuilt DuckDB binary' warning line" 1
note "EVID prebuilt-log-line present"

# ---------------------------------------------------------------------------
# Leg C — non-vacuity control: bogus VERSION must NOT be served
# ---------------------------------------------------------------------------
log "leg C: bogus-VERSION control (patched builder copy, in-container)"
mkdir -p "$SCRATCH/ctrl-home/.frozen-duckdb"   # parent only; cache ABSENT
# Payload as a standalone script file: avoids nested-quote fragility.
cat >"$SCRATCH/legc.sh" <<'LEGC'
#!/usr/bin/env bash
set -euxo pipefail
mkdir -p /work/shim
# git shim: the fallback compile must never rescue a 404 in this court; the
# shim makes clone_duckdb_locally fail FAST with a typed exit.
printf '#!/bin/sh\necho "dg4-control: git blocked" >&2\nexit 1\n' > /work/shim/git
chmod +x /work/shim/git
# The control runs INSIDE the repo-copy workspace (staged by Leg B) so the
# builder crate's workspace inheritance resolves: add two members — a pristine
# builder copy patched to a nonexistent VERSION, and a consumer calling
# ensure_binary directly.
test -d /work/repo-copy/crates
cp -r /repo/crates/frozen-duckdb-builder /work/repo-copy/ctrl-builder
sed -i 's/const VERSION: &str = "[^"]*";/const VERSION: \&str = "9.9.9-nonexistent";/' \
  /work/repo-copy/ctrl-builder/src/lib.rs
grep -n "9.9.9-nonexistent" /work/repo-copy/ctrl-builder/src/lib.rs
mkdir -p /work/repo-copy/ctrl-consumer/src
cat > /work/repo-copy/ctrl-consumer/Cargo.toml <<'TOML'
[package]
name = "dg4-ctrl-consumer"
version = "0.1.0"
edition = "2021"

[dependencies]
frozen-duckdb-builder = { path = "../ctrl-builder" }
TOML
# ensure_binary fails at RUNTIME, so the control must RUN, not just build.
cat > /work/repo-copy/ctrl-consumer/src/main.rs <<'EOF'
fn main() {
    let p = frozen_duckdb_builder::ensure_binary()
        .expect("DG4 control: ensure_binary must FAIL for nonexistent version");
    println!("UNEXPECTED_SERVED={}", p.display());
}
EOF
sed -i 's#"crates/frozen-duckdb",#"crates/frozen-duckdb", "ctrl-builder", "ctrl-consumer",#' \
  /work/repo-copy/Cargo.toml
grep -n "ctrl-builder" /work/repo-copy/Cargo.toml
set +e
PATH=/work/shim:$PATH cargo run --manifest-path /work/repo-copy/ctrl-consumer/Cargo.toml 2>&1 | tee /work/ctrl.log
CARGO_RC=${PIPESTATUS[0]}
set -e
echo "CONTROL_CARGO_RC=${CARGO_RC}"
if [ "$CARGO_RC" = "0" ]; then
  echo "CONTROL_EXIT=0 (unexpected — control must fail)"
fi
exit 1  # control MUST exit nonzero (expected failure leg)
LEGC
set +e
docker run --rm \
  -v "$REPO":/repo:ro \
  -v "$SCRATCH":$W \
  -e HOME=$W/ctrl-home \
  -e CARGO_TARGET_DIR=$W/ctrl-target \
  -e CARGO_HOME=$W/cargo-home \
  -w $W \
  "$IMAGE" \
  bash /work/legc.sh
CTRL_EXIT=$?
set -e
CTRL_EXIT=$?
set -e
note "CTRL container-exit ${CTRL_EXIT} (nonzero expected)"
if [ "$CTRL_EXIT" = "0" ]; then
  finish "FAIL" "non-vacuity control PASSED unexpectedly — bogus VERSION was served" 1
fi
if grep -q "UNEXPECTED_SERVED" "$SCRATCH/ctrl.log" 2>/dev/null; then
  finish "FAIL" "control served a binary for a nonexistent version" 1
fi
CTRL_SO_COUNT="$(find "$SCRATCH/ctrl-home" -name 'libduckdb_*.so' -type f | wc -l | tr -d ' ')"
note "CTRL served-so-count ${CTRL_SO_COUNT} (0 expected)"
[ "$CTRL_SO_COUNT" = "0" ] || finish "FAIL" "control cached a .so for bogus version" 1
if [ -f "$SCRATCH/ctrl.log" ]; then
  tail -25 "$SCRATCH/ctrl.log" | while IFS= read -r l; do note "CTRL log: $l"; done
fi
note "PASS control failed as required; no binary served for v9.9.9-nonexistent"

finish "PASS" "download court: fresh container -> empty cache -> live release download -> ELF+size-pin verify -> dynamic link -> SELECT version()+42" 0
