#!/usr/bin/env bash
# build_extensions_local.sh (ADV4)
#
# Local closure attempt for the tpch/flock extension 404s (KD-02-adjacent):
# builds loadable extensions NATIVELY (osx_arm64, release) from the pinned dev
# engine commit and LOAD-tests them against the cached v2.0.0-dev engine.
#
# Pin: duckdb b6013f7aa2f36caabb9466ce1791667ea8a280bb (engine reports
#      v2.0.0-dev88595)
#
# Usage:
#   scripts/build_extensions_local.sh            # tpch + flock attempt
#   scripts/build_extensions_local.sh tpch       # tpch ignored-tests un-blocker
#   scripts/build_extensions_local.sh flock      # flock only (expected loud-fail)
#
# Strategy:
#   tpch  — in-tree at the pin (extension/tpch). BUILD_EXTENSIONS_ONLY must be
#           explicitly 0: the loadable statically links duckdb_static, so the
#           core library has to be in the build graph.
#   flock — community extension (dais-labo/flock, HEAD dev 2026-09-30). Its
#           duckdb submodule is re-pointed at the engine pin before building.
#           Known-expected loud-fail: flock's parser-extension code targets the
#           old `const std::string &` parse_function_t API; the pin's API is
#           `vector<SimpleToken>` — see ADV4 receipt.
#
# Artifacts:
#   target/adv4/native-duckdb/build/extension/tpch/tpch.duckdb_extension
#   Install dir so `INSTALL tpch` resolves without the network:
#   ~/.duckdb/extensions/b6013f7aa2/osx_arm64/tpch.duckdb_extension
#   (version dir = the SHORT SHA from the registry URL, not v2.0.0-dev88595)
#
# Receipt: docs/sjira/v2.0.0-dev/receipts/ADV4-extensions.md

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PIN="b6013f7aa2f36caabb9466ce1791667ea8a280bb"
PIN_SHORT="b6013f7aa2"
SCRATCH="${REPO_ROOT}/target/adv4"
NATIVE_TREE="${SCRATCH}/native-duckdb"
FLOCK_TREE="${SCRATCH}/flock"
CORES="$(sysctl -n hw.ncpu)"

log() { printf '[adv4] %s\n' "$*"; }
die() { printf '[adv4] FAIL: %s\n' "$*" >&2; exit 1; }

want="${1:-all}"
case "$want" in tpch|flock|all) ;; *) die "usage: $0 [tpch|flock|all]" ;; esac

mkdir -p "$SCRATCH"

# ---------------------------------------------------------------------------
# tpch — build loadable extension at the pin
# ---------------------------------------------------------------------------
build_tpch() {
  if [ ! -d "${NATIVE_TREE}/.git" ]; then
    log "shallow-cloning duckdb at pin ${PIN}"
    git clone --filter=blob:none --no-checkout \
      https://github.com/duckdb/duckdb.git "$NATIVE_TREE"
    git -C "$NATIVE_TREE" fetch --depth 1 origin "$PIN"
    git -C "$NATIVE_TREE" checkout --detach FETCH_HEAD
  fi
  local actual
  actual="$(git -C "$NATIVE_TREE" rev-parse HEAD)"
  [ "$actual" = "$PIN" ] || die "cloned tree HEAD ${actual} != pin ${PIN}"
  log "source tree at pin: ${actual}"

  log "configuring (ninja, release, tpch as unlinked loadable)"
  cmake -S "$NATIVE_TREE" -B "${NATIVE_TREE}/build" -G Ninja \
    -DCMAKE_BUILD_TYPE=Release \
    -DBUILD_EXTENSIONS="tpch" \
    -DBUILD_EXTENSIONS_ONLY=0 \
    -DBUILD_UNITTESTS=OFF \
    -DENABLE_SANITIZER=OFF \
    -DCMAKE_OSX_ARCHITECTURES=arm64 \
    >"${SCRATCH}/tpch-configure.log" 2>&1 || {
      tail -20 "${SCRATCH}/tpch-configure.log" >&2
      die "cmake configure failed (see ${SCRATCH}/tpch-configure.log)"
    }

  log "building tpch_loadable_extension (pulls the core static lib)"
  cmake --build "${NATIVE_TREE}/build" --target tpch_loadable_extension \
    -j"$CORES" >"${SCRATCH}/tpch-build.log" 2>&1 || {
      tail -30 "${SCRATCH}/tpch-build.log" >&2
      die "tpch build failed (see ${SCRATCH}/tpch-build.log)"
    }

  local art
  art="$(find "${NATIVE_TREE}/build" -name 'tpch.duckdb_extension' -type f | head -1)"
  [ -n "$art" ] || die "tpch.duckdb_extension not found after build"
  log "artifact: ${art} ($(du -h "$art" | cut -f1))"

  log "installing into ~/.duckdb/extensions/${PIN_SHORT}/osx_arm64/ (INSTALL without network)"
  mkdir -p "${HOME}/.duckdb/extensions/${PIN_SHORT}/osx_arm64"
  cp "$art" "${HOME}/.duckdb/extensions/${PIN_SHORT}/osx_arm64/"
}

# ---------------------------------------------------------------------------
# flock — best-effort, loud-fail expected (API incompatibility at the pin)
# ---------------------------------------------------------------------------
FLOCK_REPO="https://github.com/dais-labo/flock.git"

build_flock() {
  if [ ! -d "${FLOCK_TREE}/.git" ]; then
    git clone --depth 1 "$FLOCK_REPO" "$FLOCK_TREE" || die "cannot clone ${FLOCK_REPO}"
  fi
  git -C "$FLOCK_TREE" submodule update --init --depth 1
  # Re-point flock's duckdb submodule at OUR engine pin so any produced
  # loadable would carry the b6013f7aa2 version stamp.
  git -C "${FLOCK_TREE}/duckdb" fetch --depth 1 origin "$PIN" || true
  git -C "${FLOCK_TREE}/duckdb" checkout --detach FETCH_HEAD || true
  log "flock duckdb submodule at: $(git -C "${FLOCK_TREE}/duckdb" rev-parse HEAD 2>/dev/null || echo n/a)"

  if ! make -C "$FLOCK_TREE" release -j"$CORES" \
      >"${SCRATCH}/flock-build.log" 2>&1; then
    echo "---- flock build errors ----" >&2
    grep -B2 "Error [0-9]" "${SCRATCH}/flock-build.log" | head -40 >&2 || true
    die "flock build failed at the dev pin — flock's parser-extension code still targets the pre-v2.0 parse_function_t API (const std::string&); the pin uses vector<SimpleToken>. KD-02 (flock half) stays open."
  fi
  local art
  art="$(find "${FLOCK_TREE}/build" -name 'flock.duckdb_extension' -type f 2>/dev/null | head -1)"
  [ -n "$art" ] || die "flock.duckdb_extension not found after build"
  log "artifact: ${art}"
}

case "$want" in
  tpch|all) build_tpch ;;
esac
case "$want" in
  flock|all) build_flock ;;
esac

log "done"
