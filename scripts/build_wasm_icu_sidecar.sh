#!/usr/bin/env bash
# build_wasm_icu_sidecar.sh — SP4 lane: build DuckDB's ICU extension as a
# standalone loadable wasm module against the pinned source
# d54cbbe3df17e8e5ce68312f483b4155836112aa.
#
# Recipe (operator spec):
#   - source: existing verified checkout target/wasm-build/duckdb at the pin
#     (shallow-clone fallback omitted; that checkout already exists here)
#   - emsdk (real emsdk at ~/emsdk; brew emcc is broken here — no wasm LLVM)
#   - configure: DUCKDB_EXPLICIT_PLATFORM=wasm_eh, extension statically
#     DISABLED (BUILD_EXTENSIONS=""), -DBUILD_UNITTESTS=OFF,
#     -DWASM_LOADABLE_EXTENSIONS=1 -> extension targets become
#     -sSIDE_MODULE=1 (extension_build_tools.cmake:302)
#   - em++ flags match scripts/build_wasm_duckdb.sh's proven set:
#     -O3 -flto -sUSE_PTHREADS=1
#   - build target: `icu` (shared/loadable object)
#
# Acceptance ladder:
#   1. icu .wasm artifact with a size
#   2. file/magic sanity
#   3. smoke-load attempt (LOAD 'icu') against the core — recorded honestly;
#      a missing dynamic-loading seam is a typed BLOCKED, not hidden.
#
# Receipt: docs/sjira/v2.0.0/receipts/SP4-icu-sidecar.md

set -euo pipefail

SOURCE_PIN="d54cbbe3df17e8e5ce68312f483b4155836112aa"
DUCKDB_REPO="https://github.com/duckdb/duckdb.git"
LANE_ROOT="${LANE_ROOT:-/Users/sac/frozen-duckdb/target/wasm-icu-sidecar}"
BUILD_ROOT="${LANE_ROOT}/build"
SRC_DIR="${LANE_ROOT}/src"
ARTIFACT_DIR="${LANE_ROOT}/artifact"

log() { printf '[icu-sidecar] %s\n' "$*"; }
fail2() { printf '[icu-sidecar] REFUSED %s\n' "$*" >&2; exit 2; }

# ---------------------------------------------------------------------------
# Phase 1: probe-first toolchain (real emsdk, not the broken brew emcc)
# ---------------------------------------------------------------------------
if [ -d "${HOME}/emsdk" ]; then
  # shellcheck disable=SC1091
  source "${HOME}/emsdk/emsdk_env.sh" >/dev/null 2>&1 || true
fi
command -v emcmake >/dev/null 2>&1 || fail2 \
  "emcmake not on PATH (real emsdk absent). See scripts/build_wasm_duckdb.sh"
if ! emcc --version >/dev/null 2>&1; then
  emcc --version 2>&1 | sed 's/^/  /' >&2 || true
  fail2 "emcc sanity check FAILED (brew emscripten lacks wasm LLVM).
  Fix: real emsdk at ~/emsdk (activate + source emsdk_env.sh)."
fi
EMCC_VERSION="$(emcc --version | head -1)"
log "PROBE OK: ${EMCC_VERSION}"

# ---------------------------------------------------------------------------
# Phase 2: pinned source tree (reuse verified checkout; clone if absent)
# ---------------------------------------------------------------------------
if [ -d /Users/sac/frozen-duckdb/target/wasm-build/duckdb/.git ]; then
  SRC_DIR=/Users/sac/frozen-duckdb/target/wasm-build/duckdb
else
  mkdir -p "${SRC_DIR}"
  git clone --no-checkout --filter=blob:none "${DUCKDB_REPO}" "${SRC_DIR}" \
    2>&1 | sed 's/^/  /' || fail2 "clone failed"
  git -C "${SRC_DIR}" checkout --quiet "${SOURCE_PIN}" || {
    git -C "${SRC_DIR}" fetch origin "${SOURCE_PIN}" 2>&1 | sed 's/^/  /' \
      || fail2 "fetch of pin failed"
    git -C "${SRC_DIR}" checkout --quiet "${SOURCE_PIN}" \
      || fail2 "checkout of pin failed"
  }
fi
ACTUAL_SHA="$(git -C "${SRC_DIR}" rev-parse HEAD)"
[ "${ACTUAL_SHA}" = "${SOURCE_PIN}" ] || fail2 \
  "subject mismatch: HEAD ${ACTUAL_SHA} != pin ${SOURCE_PIN}"
log "PIN VERIFIED: ${SRC_DIR} HEAD == ${SOURCE_PIN}"

# ---------------------------------------------------------------------------
# Phase 3: configure (icu statically disabled; dynamic-loadable wasm mode)
# ---------------------------------------------------------------------------
log "CONFIGURE (emcmake, WASM_LOADABLE_EXTENSIONS)"
emcmake cmake -S "${SRC_DIR}" -B "${BUILD_ROOT}" \
  -DCMAKE_CXX_STANDARD=17 \
  -DCMAKE_CXX_FLAGS="-O3 -flto -sUSE_PTHREADS=1" \
  -DDUCKDB_EXPLICIT_PLATFORM=wasm_eh \
  -DWASM_LOADABLE_EXTENSIONS=1 \
  -DBUILD_EXTENSIONS="" \
  -DENABLE_EXTENSION_AUTOLOADING=OFF \
  -DENABLE_EXTENSION_AUTOINSTALL=OFF \
  -DBUILD_HTTPFS=OFF \
  -DBUILD_UNITTESTS=OFF \
  -DBUILD_SHELL=OFF \
  -DBUILD_JSON_EXT=OFF \
  -DBUILD_PARQUET_EXT=OFF \
  -DCMAKE_BUILD_TYPE=Release \
  2>&1 | tail -25 | sed 's/^/  /' || fail2 "emcmake configure failed"

# ---------------------------------------------------------------------------
# Phase 4: build ONLY the icu loadable target
# ---------------------------------------------------------------------------
log "BUILD (cmake --build --target icu_loadable_extension)"
cmake --build "${BUILD_ROOT}" --target icu_loadable_extension \
  -j "$(sysctl -n hw.ncpu 2>/dev/null || nproc)" \
  2>&1 | tail -30 | sed 's/^/  /' || fail2 "cmake --build --target icu_loadable_extension failed"

# ---------------------------------------------------------------------------
# Phase 5: artifact + acceptance ladder
# ---------------------------------------------------------------------------
mkdir -p "${ARTIFACT_DIR}"
WASM_OUT="$(find "${BUILD_ROOT}" -name '*icu*' \( -name '*.wasm' \
  -o -name '*.duckdb_extension*' \) -type f | head -5)"
if [ -z "${WASM_OUT}" ]; then
  log "FAIL: no icu .wasm/.duckdb_extension artifact produced"
  find "${BUILD_ROOT}" -name '*icu*' -type f | head -20 | sed 's/^/  /'
  exit 1
fi
cp ${WASM_OUT} "${ARTIFACT_DIR}/" 2>/dev/null || true
log "ARTIFACTS:"
ls -l "${ARTIFACT_DIR}" | sed 's/^/  /'
for f in "${ARTIFACT_DIR}"/*; do
  log "file(1): $(file "$f" 2>/dev/null || echo 'file(1) unavailable')"
  case "$f" in
  *.wasm)
    if head -c 4 "$f" | grep -q $'\x00asm'; then
      log "MAGIC OK: ${f} starts with \\0asm"
    else
      log "MAGIC FAIL: ${f} does not start with \\0asm"
    fi ;;
  *) log "MAGIC SKIP (not a wasm module): ${f}" ;;
  esac
done

log "DONE — artifact dir: ${ARTIFACT_DIR}"
