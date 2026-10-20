#!/usr/bin/env bash
# build_extensions_local.sh (ADV4)
#
# Attempt local closure of the tpch/flock extension 404s (KD-02-adjacent) by
# building loadable extensions NATIVELY (osx_arm64) from the pinned dev engine
# commit, then LOAD-testing them against the cached v2.0.0-dev engine.
#
# Pin: duckdb b6013f7aa2f36caabb9466ce1791667ea8a280bb (v2.0.0-dev88595)
#
# Usage: scripts/build_extensions_local.sh [tpch|flock|all]
#
# Strategy per extension:
#   tpch  — in-tree at the pin (extension/tpch): shallow-clone the pin,
#           configure with -DBUILD_EXTENSIONS="tpch" -DBUILD_EXTENSIONS_ONLY=1,
#           build the loadable-extension target.
#   flock — community extension (dais-labo/flock). Its CMake FetchContent's a
#           duckdb release; compatibility with the v2.0.0-dev pin is NOT
#           assumed — the build is attempted and loud-failed if the tree
#           refuses.
#
# Artifacts: target/adv4/native-duckdb/build/extension/<name>/<name>.duckdb_extension
# Receipt:   docs/sjira/v2.0.0-dev/receipts/ADV4-extensions.md

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PIN="b6013f7aa2f36caabb9466ce1791667ea8a280bb"
SCRATCH="${REPO_ROOT}/target/adv4"
NATIVE_TREE="${SCRATCH}/native-duckdb"
CORES="$(sysctl -n hw.ncpu)"
ENGINE_DYLIB="${HOME}/.frozen-duckdb/cache/v2.0.0-dev-arm64/libduckdb_arm64.dylib"

log() { printf '[adv4] %s\n' "$*"; }
die() { printf '[adv4] FAIL: %s\n' "$*" >&2; exit 1; }

want="${1:-all}"
case "$want" in tpch|flock|all) ;; *) die "usage: $0 [tpch|flock|all]" ;; esac

mkdir -p "$SCRATCH"

# ---------------------------------------------------------------------------
# Pinned native source tree (shallow clone at the exact pin, reused if present)
# ---------------------------------------------------------------------------
if [ ! -d "${NATIVE_TREE}/.git" ]; then
  log "shallow-cloning duckdb at pin ${PIN}"
  git clone --filter=blob:none --no-checkout \
    https://github.com/duckdb/duckdb.git "$NATIVE_TREE"
  git -C "$NATIVE_TREE" fetch --depth 1 origin "$PIN"
  git -C "$NATIVE_TREE" checkout --detach FETCH_HEAD
fi
ACTUAL="$(git -C "$NATIVE_TREE" rev-parse HEAD)"
[ "$ACTUAL" = "$PIN" ] || die "cloned tree HEAD ${ACTUAL} != pin ${PIN}"
log "source tree at pin: ${ACTUAL}"

# ---------------------------------------------------------------------------
# tpch (in-tree extension) — loadable build
# ---------------------------------------------------------------------------
build_tpch() {
  log "configuring tpch loadable build (ninja, release, extensions-only)"
  cmake -S "$NATIVE_TREE" -B "${NATIVE_TREE}/build" -G Ninja \
    -DCMAKE_BUILD_TYPE=Release \
    -DBUILD_EXTENSIONS="tpch" \
    -DBUILD_UNITTESTS=OFF \
    -DENABLE_SANITIZER=OFF \
    -DCMAKE_OSX_ARCHITECTURES=arm64 \
    >"${SCRATCH}/tpch-configure.log" 2>&1 || {
      tail -20 "${SCRATCH}/tpch-configure.log" >&2
      die "cmake configure failed (see ${SCRATCH}/tpch-configure.log)"
    }
  log "building tpch loadable extension"
  cmake --build "${NATIVE_TREE}/build" --target duckdb_static tpch_loadable_extension -j"$CORES" \
    >"${SCRATCH}/tpch-build.log" 2>&1 || {
      tail -30 "${SCRATCH}/tpch-build.log" >&2
      die "tpch build failed (see ${SCRATCH}/tpch-build.log)"
    }
  local ART
  ART="$(find "${NATIVE_TREE}/build" -name 'tpch.duckdb_extension' -type f | head -1)"
  [ -n "$ART" ] || die "tpch.duckdb_extension not found after build"
  log "artifact: ${ART}"
  printf '%s\n' "$ART" >"${SCRATCH}/tpch-artifact-path.txt"
}

# ---------------------------------------------------------------------------
# flock (community extension) — best-effort attempt, loud-fail allowed
# ---------------------------------------------------------------------------
FLOCK_REPO="https://github.com/dais-labo/flock.git"
FLOCK_TREE="${SCRATCH}/flock"

build_flock() {
  if [ ! -d "${FLOCK_TREE}/.git" ] && ! git clone --depth 1 "$FLOCK_REPO" "$FLOCK_TREE" 2>"${SCRATCH}/flock-clone.err"; then
    die "cannot clone ${FLOCK_REPO}: $(cat "${SCRATCH}/flock-clone.err")"
  fi
  log "flock tree: $(git -C "$FLOCK_TREE" log -1 --format='%H %s' 2>/dev/null || echo 'n/a')"

  local flock_ext_dir
  flock_ext_dir="$(find "$FLOCK_TREE" -maxdepth 2 -type d -name flock | head -1)"
  [ -n "$flock_ext_dir" ] || die "no flock extension dir found in clone"

  # Community extensions build against duckdb via the extension template.
  # Point it at the pinned tree so the ABI matches the cached engine exactly.
  export DUCKDB_VERSION="${PIN}"
  export DUCKDB_REPOSITORY="https://github.com/duckdb/duckdb"
  export CustomExtension="flock"

  if ! make release BUILD_EXTENSIONS="flock" \
      -C "$flock_ext_dir" >"${SCRATCH}/flock-build.log" 2>&1; then
    tail -40 "${SCRATCH}/flock.compile.log" "${SCRATCH}/flock-build.log" >&2 || true
    die "flock build failed — tree/tooling refuses at the dev pin (honest loud-fail; KD-02 stays open)"
  fi
  local ART
  ART="$(find "$flock_ext_dir" "${NATIVE_TREE}/build" -name 'flock.duckdb_extension' -type f 2>/dev/null | head -1)"
  [ -n "$ART" ] || die "flock.duckdb_extension not found after build"
  log "artifact: ${ART}"
  printf '%s\n' "$ART" >"${SCRATCH}/flock-artifact-path.txt"
}

# ---------------------------------------------------------------------------
# LOAD test against the cached dev engine via its C API (python ctypes)
# ---------------------------------------------------------------------------
load_test() {
  local name="$1"
  local art
  art="$(cat "${SCRATCH}/${name}-artifact-path.txt")"
  DUCKDB_LIB="$ENGINE_DYLIB" ARTIFACT="$art" python3 - <<'PYEOF'
import ctypes, os
lib = ctypes.CDLL(os.environ["DUCKDB_LIB"])
lib.duckdb_open(None, ctypes.byref(ctypes.c_void_p()))
db = ctypes.c_void_p()
lib.duckdb_open(None, ctypes.byref(db))
con = ctypes.c_void_p()
lib.duckdb_connect(db, ctypes.byref(con))
sql = f"LOAD '{os.environ['ARTIFACT']}'"
lib.duckdb_query(con, sql.encode(), ctypes.byref(ctypes.c_void_p()))
# re-query properly to capture the result / error
out = ctypes.c_void_p()
rc = lib.duckdb_query(con, sql.encode(), ctypes.byref(out))
if rc != 0:
    msg = lib.duckdb_result_error(out)
    print(f"LOAD-FAIL rc={rc}: {ctypes.c_char_p(msg).value.decode() if msg else '?'}")
    raise SystemExit(1)
print(f"LOAD-OK {os.environ['ARTIFACT']}")
lib.duckdb_close(ctypes.byref(db))
PYEOF
}

case "$want" in
  tpch|all) build_tpch; load_test tpch ;;
esac
case "$want" in
  flock|all) build_flock; load_test flock ;;
esac

log "done"
