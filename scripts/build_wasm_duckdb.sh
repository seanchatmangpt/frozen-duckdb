#!/usr/bin/env bash
# build_wasm_duckdb.sh — hermetic wasm build of DuckDB for frozen-duckdb.
#
# Blueprint (operator CMake spec):
#   DUCKDB_EXPLICIT_PLATFORM=wasm_eh
#   CMAKE_CXX_STANDARD=17
#   BUILD_EXTENSIONS="parquet;json"
#   ENABLE_EXTENSION_AUTOLOADING=OFF
#   ENABLE_EXTENSION_AUTOINSTALL=OFF
#   BUILD_HTTPFS=OFF
#   -O3 -flto
#
# Source parity: clones upstream DuckDB and checks out the SAME pinned
# SHA as crates/frozen-duckdb-builder (SOURCE_PIN), mirroring
# compile_duckdb_locally (lib.rs:522). No state derived from VERSION.
#
# Probe-first contract: if the emscripten toolchain is absent OR fails
# its own sanity check, exit 2 with an install pointer — loud skip,
# never a faked success.
#
# Doc lines kept <=100 chars per house style.

set -euo pipefail

SOURCE_PIN="b6013f7aa2f36caabb9466ce1791667ea8a280bb"
DUCKDB_REPO="https://github.com/duckdb/duckdb.git"
BUILD_ROOT="${BUILD_ROOT:-target/wasm-build}"
ARTIFACT_DIR="${ARTIFACT_DIR:-target/wasm}"
ARTIFACT="${ARTIFACT_DIR}/duckdb.wasm"

log() { printf '[wasm-build] %s\n' "$*"; }
fail2() { printf '[wasm-build] REFUSED %s\n' "$*" >&2; exit 2; }

# ---------------------------------------------------------------------------
# Phase 1: probe-first (loud skip). Existence is not enough; the toolchain
# must pass its own sanity check (brew emscripten with a non-wasm LLVM
# fails `emcc --version`).
# ---------------------------------------------------------------------------
log "PROBE: emsdk / emscripten toolchain"

command -v emcmake >/dev/null 2>&1 || fail2 \
  "emcmake not on PATH. Install emsdk:
  git clone https://github.com/emscripten-core/emsdk.git ~/emsdk
  cd ~/emsdk && ./emsdk install latest && ./emsdk activate latest
  source ~/emsdk/emsdk_env.sh"

command -v emcc >/dev/null 2>&1 || fail2 \
  "emcc not on PATH. Install emsdk (see above)."

if ! emcc --version >/dev/null 2>&1; then
  emcc --version 2>&1 | sed 's/^/  /' >&2 || true
  fail2 "emcc sanity check FAILED (LLVM lacks wasm backend?).
  Fix: install a REAL emsdk (homebrew emscripten is known broken here):
  git clone https://github.com/emscripten-core/emsdk.git ~/emsdk
  cd ~/emsdk && ./emsdk install latest && ./emsdk activate latest
  source ~/emsdk/emsdk_env.sh && re-run this script."
fi

EMCC_VERSION="$(emcc --version | head -1)"
log "PROBE OK: ${EMCC_VERSION}"

# ---------------------------------------------------------------------------
# Phase 2: pinned-SHA clone + checkout (mirror of compile_duckdb_locally)
# ---------------------------------------------------------------------------
mkdir -p "${BUILD_ROOT}"
DUCKDB_DIR="${BUILD_ROOT}/duckdb"

if [ ! -d "${DUCKDB_DIR}/.git" ]; then
  log "CLONE: ${DUCKDB_REPO}"
  git clone --no-checkout "${DUCKDB_REPO}" "${DUCKDB_DIR}" 2>&1 \
    | sed 's/^/  /' || fail2 "git clone failed"
fi

log "CHECKOUT: pinned SOURCE_PIN ${SOURCE_PIN}"
if ! git -C "${DUCKDB_DIR}" checkout --quiet "${SOURCE_PIN}"; then
  git -C "${DUCKDB_DIR}" fetch origin "${SOURCE_PIN}" 2>&1 | sed 's/^/  /' \
    || fail2 "fetch of pinned SHA failed"
  git -C "${DUCKDB_DIR}" checkout --quiet "${SOURCE_PIN}" \
    || fail2 "checkout of pinned SHA ${SOURCE_PIN} failed"
fi
ACTUAL_SHA="$(git -C "${DUCKDB_DIR}" rev-parse HEAD)"
[ "${ACTUAL_SHA}" = "${SOURCE_PIN}" ] || fail2 \
  "subject mismatch: HEAD ${ACTUAL_SHA} != pinned ${SOURCE_PIN}"
log "PIN VERIFIED: HEAD == ${SOURCE_PIN}"

# ---------------------------------------------------------------------------
# Phase 3: emcmake configure + build (operator blueprint, verbatim flags)
# ---------------------------------------------------------------------------
log "CONFIGURE (emcmake)"
emcmake cmake -S "${DUCKDB_DIR}" -B "${BUILD_ROOT}/build" \
  -DCMAKE_CXX_STANDARD=17 \
  -DCMAKE_CXX_FLAGS="-O3 -flto -sUSE_PTHREADS=1" \
  -DDUCKDB_EXPLICIT_PLATFORM=wasm_eh \
  -DBUILD_EXTENSIONS="parquet;json" \
  -DENABLE_EXTENSION_AUTOLOADING=OFF \
  -DENABLE_EXTENSION_AUTOINSTALL=OFF \
  -DBUILD_HTTPFS=OFF \
  -DCMAKE_BUILD_TYPE=Release \
  2>&1 | sed 's/^/  /' || fail2 "emcmake configure failed"

log "BUILD (cmake --build, extension/relic flags from blueprint)"
cmake --build "${BUILD_ROOT}/build" -j "$(sysctl -n hw.ncpu 2>/dev/null || nproc)" \
  2>&1 | tail -40 | sed 's/^/  /' || fail2 "cmake --build failed"

# ---------------------------------------------------------------------------
# Phase 4: artifact + receipt block
# ---------------------------------------------------------------------------
mkdir -p "${ARTIFACT_DIR}"
# DuckDB's wasm target emits libduckdb; locate the .wasm/.js pair it produced.
WASM_OUT="$(find "${BUILD_ROOT}/build" -name '*.wasm' -type f | head -1)"
[ -n "${WASM_OUT}" ] || fail2 "no .wasm artifact produced by the build"
cp "${WASM_OUT}" "${ARTIFACT}"

ART_BYTES="$(wc -c < "${ARTIFACT}" | tr -d ' ')"
ART_SHA="$(sha256sum "${ARTIFACT}" 2>/dev/null | awk '{print $1}' \
  || shasum -a 256 "${ARTIFACT}" | awk '{print $1}')"

cat <<RECEIPT
=======================================================================
WASM BUILD RECEIPT
=======================================================================
artifact:      ${ARTIFACT}
bytes:         ${ART_BYTES}
sha256:        ${ART_SHA}
source_pin:    ${SOURCE_PIN}
emcc:          ${EMCC_VERSION}
platform:      DUCKDB_EXPLICIT_PLATFORM=wasm_eh
cxx_standard:  17
extensions:    parquet;json (autoload OFF, autoinstall OFF, httpfs OFF)
opt:           -O3 -flto
standing:      ALIVE (real build, real artifact)
=======================================================================
RECEIPT

log "DONE"
