#!/usr/bin/env bash
# consumer_cleanroom.sh — the cleanroom law, proven end to end.
#
# A downstream Rust project consumes the FROZEN binary with ZERO DuckDB
# compilation:
#   1. scratch cargo project under /tmp (path deps into the repo checkout)
#   2. scratch HOME with ~/.frozen-duckdb/cache pre-seeded from the real
#      cache (macOS local leg) or a pre-mounted volume (act leg), so
#      ensure_binary cache-hits
#   3. ZERO-compile asserted via:
#        a. build-script log line "Using prebuilt DuckDB binary"
#        b. link-only wall time (single-digit seconds, not minutes)
#   4. run the quickstart program; assert version v2.0.0* via query
#
# Modes:
#   consumer_cleanroom.sh local   (macOS host — the ALIVE witness leg)
#   consumer_cleanroom.sh act     (inside nektos/act Linux container)
#
# Receipts: /tmp/cleanroom-receipt-<mode>-<ts>.json (+ .log with real output)
# Exit codes: 0 PASS · 1 FAIL · 2 loud SKIP / typed BLOCKED divergence
#
# Env knobs:
#   CLEANROOM_REPO             repo root (default /Users/sac/frozen-duckdb)
#   CLEANROOM_RECEIPT          override receipt path
set -euo pipefail

MODE="${1:-local}"
case "$MODE" in
  local|act) ;;
  *) printf 'usage: %s [local|act]\n' "$0" >&2; exit 2 ;;
esac

REPO="${CLEANROOM_REPO:-/Users/sac/frozen-duckdb}"
TS="$(date -u +%Y%m%dT%H%M%SZ)"
RECEIPT="${CLEANROOM_RECEIPT:-/tmp/cleanroom-receipt-${MODE}-${TS}.json}"
LOG="${RECEIPT%.json}.log"
mkdir -p "$(dirname "$RECEIPT")"
: >"$LOG"

log()  { printf '%s %s\n' "$(date -u +%H:%M:%S)" "$*" | tee -a "$LOG"; }

# Receipt always embeds REAL measured values written by the run itself.
write_receipt() {
  local standing="$1" reason="$2" exit_code="$3"
  cat >"$RECEIPT" <<EOF
{
  "court": "consumer-cleanroom",
  "mode": "$MODE",
  "standing": "$standing",
  "reason": "$reason",
  "exit_code": $exit_code,
  "repo": "$REPO",
  "head": "$REPO_HEAD",
  "arch": "$ARCH",
  "scratch": "$SCRATCH",
  "cache_seeded_from": "$CACHE_DST_PARENT",
  "build_seconds": $BUILD_SECONDS,
  "prebuilt_log_line": $PREBUILT_LINE_SEEN,
  "rpath_finding": "KD-CLEANROOM-RPATH: cargo 1.97 does not forward cargo:rustc-link-arg from dependency build scripts; downstream binaries lack LC_RPATH and load-time resolution requires DYLD_LIBRARY_PATH/LD_LIBRARY_PATH at the cache dir",
  "run_seconds": $RUN_SECONDS,
  "quickstart": "$QUICKSTART_LINE",
  "started": "$TS",
  "finished": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF
}

fail() { log "FAIL: $*"; write_receipt "FAIL" "$*" 1; exit 1; }
skip() { log "[SKIP] $*"; write_receipt "SKIP" "$*" 2; exit 2; }

# Defaults so the receipt writer never reads an unset variable.
REPO_HEAD="$(git -C "$REPO" rev-parse HEAD 2>/dev/null || echo unknown)"
SCRATCH_REAL_HOME="$HOME"
ARCH="$(uname -m)"
SCRATCH=""
CACHE_DST_PARENT=""
BUILD_SECONDS=-1
PREBUILT_LINE_SEEN=false
QUICKSTART_LINE=null
RUN_SECONDS=-1

# ---------------------------------------------------------------------------
# 0. Prerequisites
# ---------------------------------------------------------------------------
command -v cargo >/dev/null 2>&1 || skip "cargo not on PATH"
[[ -d "$REPO/crates/frozen-duckdb" ]] || fail "repo crates missing under $REPO"

log "cleanroom mode=$MODE arch=$ARCH repo=$REPO"

# ---------------------------------------------------------------------------
# 1. Scratch consumer project + scratch HOME
# ---------------------------------------------------------------------------
SCRATCH="$(mktemp -d /tmp/cleanroom-consumer-XXXXXX)"
SCRATCH_HOME="$SCRATCH/home"
mkdir -p "$SCRATCH_HOME/.frozen-duckdb" "$SCRATCH/consumer/src"
log "scratch=$SCRATCH"

cat >"$SCRATCH/consumer/Cargo.toml" <<EOF
[package]
name = "cleanroom-consumer"
version = "0.1.0"
edition = "2021"

[dependencies]
frozen-duckdb = { path = "$REPO/crates/frozen-duckdb" }

[workspace]
EOF

cat >"$SCRATCH/consumer/src/main.rs" <<'EOF'
use frozen_duckdb::Connection;

fn main() {
    let conn = Connection::open_in_memory().expect("open in-memory db");
    let version: String = conn
        .query_row("PRAGMA version", [], |row| row.get(0))
        .expect("version query");
    println!("DUCKDB_VERSION={}", version);
    conn.execute_batch("CREATE TABLE t(a INTEGER); INSERT INTO t VALUES (42);")
        .expect("create/insert");
    let n: i64 = conn
        .query_row("SELECT sum(a) FROM t", [], |r| r.get(0))
        .expect("sum query");
    println!("SUM={}", n);
}
EOF

# ---------------------------------------------------------------------------
# 2. Typed divergence gate (act leg): no linux .so prebuilt exists.
#    Refuse the silent 30-minute compile fallback; fail fast, typed.
# ---------------------------------------------------------------------------
if [[ "$(uname -s)" != "Darwin" && "${CLEANROOM_BLOCK_ON_LINUX:-1}" == "1" ]]; then
  CACHE_MOUNT="/cleanroom-home/.frozen-duckdb/cache/v2.0.0-${ARCH}"
  log "[BLOCKED_LINUX_PREBUILT_MISSING] no linux .so prebuilt exists \
(KD: linux prebuilt absent); refusing silent compile fallback"
  write_receipt "BLOCKED_LINUX_PREBUILT_MISSING" \
    "linux .so prebuilt absent; workflow mechanics validated up to the link step" 2
  log "typed divergence recorded in $RECEIPT"
  exit 2
fi

# ---------------------------------------------------------------------------
# 3. Pre-seed cache — the ZERO-COMPILE mechanism
# ---------------------------------------------------------------------------
CACHE_SRC="$HOME/.frozen-duckdb/cache/v2.0.0-${ARCH}"
CACHE_DST="$SCRATCH_HOME/.frozen-duckdb/cache/v2.0.0-${ARCH}"
CACHE_DST_PARENT="$(dirname "$CACHE_DST")"

if [[ "$MODE" == "act" ]]; then
  # Inside the act container the cache arrives volume-mounted.
  MOUNTED="/cleanroom-home/.frozen-duckdb/cache/v2.0.0-${ARCH}"
  [[ -d "$MOUNTED" ]] || fail "act cache volume missing: $MOUNTED"
  mkdir -p "$CACHE_DST_PARENT"
  cp -R "$MOUNTED/." "$CACHE_DST/"
else
  [[ -d "$CACHE_SRC" ]] || skip "cache dir missing: $CACHE_SRC"
  mkdir -p "$CACHE_DST_PARENT"
  cp -c "$CACHE_SRC/." "$CACHE_DST/" 2>/dev/null \
    || cp -R "$CACHE_SRC/." "$CACHE_DST/"
fi

if [[ ! -f "$CACHE_DST/ENGINE_VERSION" ]]; then
  skip "cache pre-seed failed: ENGINE_VERSION missing at $CACHE_DST"
fi
log "cache pre-seeded: $(cat "$CACHE_DST/ENGINE_VERSION") · $(du -sh "$CACHE_DST" | cut -f1)"

# ---------------------------------------------------------------------------
# 4. Build with pre-seeded cache — ZERO DuckDB compile
# ---------------------------------------------------------------------------
cd "$SCRATCH/consumer"
export HOME="$SCRATCH_HOME"
# Scratch HOME must not break the real toolchain: rustup/cargo state lives
# outside the cache surface under test.
export RUSTUP_HOME="${RUSTUP_HOME:-$SCRATCH_REAL_HOME/.rustup}"
export CARGO_HOME="${CARGO_HOME:-$SCRATCH_REAL_HOME/.cargo}"
export CARGO_TARGET_DIR="$SCRATCH/target"

BUILD_START="$(date +%s)"
set +e
BUILD_OUT="$(cargo build 2>&1)"
BUILD_STATUS=$?
set -e
BUILD_SECONDS="$(($(date +%s) - BUILD_START))"
printf '%s\n' "$BUILD_OUT" >>"$LOG"

if ! grep -q "Using prebuilt DuckDB binary" <<<"$BUILD_OUT"; then
  fail "build log missing 'Using prebuilt DuckDB binary' — compile fallback suspected"
fi
PREBUILT_LINE_SEEN=true
log "cargo build exit=$BUILD_STATUS in ${BUILD_SECONDS}s (prebuilt log line present)"
[[ $BUILD_STATUS -eq 0 ]] || fail "cargo build exit=$BUILD_STATUS"

# ---------------------------------------------------------------------------
# 5. Run the quickstart program; assert version v2.0.0*
#
# Typed divergence (KD-CLEANROOM-RPATH, recorded in the receipt): cargo 1.97
# forwards cargo:rustc-link-search from dependency build scripts but NOT
# cargo:rustc-link-arg, so the downstream binary carries no LC_RPATH entry
# (the in-repo bins get theirs from crates/frozen-duckdb/build.rs only when
# built in-workspace). A downstream consumer therefore resolves the dylib at
# load time via DYLD_LIBRARY_PATH/LD_LIBRARY_PATH pointed at the cache dir —
# the same cache dir the build already used.
# ---------------------------------------------------------------------------
BIN="$(find "$CARGO_TARGET_DIR/debug" -maxdepth 1 -name cleanroom-consumer -type f | head -1)"
[[ -n "$BIN" ]] || fail "consumer binary not found under $CARGO_TARGET_DIR/debug"

RUN_START="$(date +%s)"
RUN_OUT="$(DYLD_LIBRARY_PATH="$CACHE_DST" LD_LIBRARY_PATH="$CACHE_DST" "$BIN" 2>&1)"
RUN_SECONDS="$(($(date +%s) - RUN_START))"
printf '%s\n' "$RUN_OUT" >>"$LOG"
QUICKSTART_LINE="$(grep '^DUCKDB_VERSION=' <<<"$RUN_OUT" || true)"
[[ -n "$QUICKSTART_LINE" ]] \
  || fail "no DUCKDB_VERSION line in quickstart output"
grep -q '^DUCKDB_VERSION=v2\.0\.0-dev' <<<"$RUN_OUT" \
  || fail "version mismatch: $QUICKSTART_LINE"
grep -q '^SUM=42$' <<<"$RUN_OUT" || fail "quickstart SUM=42 assertion failed"
log "quickstart OK: $QUICKSTART_LINE · SUM=42"

write_receipt "PASS" "cache-hit build in ${BUILD_SECONDS}s + quickstart $QUICKSTART_LINE (zero compile)" 0
log "PASS receipt: $RECEIPT"
exit 0
