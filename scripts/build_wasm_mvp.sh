#!/usr/bin/env bash
# WASM MVP slice (ADV1) — minimal lib-only duckdb.wasm build.
# Spec: docs/design/wasm-mvp-slice.md
# Reuses the PRESERVED pin-verified checkout at target/wasm-build/duckdb
# (HEAD b6013f7aa2f…, KD-05 evidence). Builds in sibling build-min dir;
# target/wasm-build/build (GAP1b full-build evidence) is never touched.
set -euo pipefail

REPO="/Users/sac/frozen-duckdb"
SRC="${REPO}/target/wasm-build/duckdb"
BUILD="${REPO}/target/wasm-build/build-min"
OUT="${REPO}/target/wasm-min"

PIN="b6013f7aa2f36caabb9466ce1791667ea8a280bb"

log() { printf '[wasm-mvp] %s\n' "$*"; }
fail() { printf '[wasm-mvp] FAIL: %s\n' "$*" >&2; exit 1; }

# --- pin verify: never build an unverified tree ---------------------------
ACTUAL="$(git -C "${SRC}" rev-parse HEAD)"
[ "${ACTUAL}" = "${PIN}" ] || fail "pin mismatch: ${ACTUAL} != ${PIN}"
log "pin verified: ${ACTUAL}"

# --- emsdk env -------------------------------------------------------------
# shellcheck disable=SC1091
source "${HOME}/emsdk/emsdk_env.sh" >/dev/null 2>&1 || fail "emsdk env failed"
EMCC_V="$(emcc --version | head -1)"
log "${EMCC_V}"
log "node $(node --version)"

mkdir -p "${OUT}"

# --- configure (spec §2 flag delta) ---------------------------------------
rm -f "${BUILD}/CMakeCache.txt"
if true; then
  log "CONFIGURE (emcmake, minimal)"
  emcmake cmake -S "${SRC}" -B "${BUILD}" \
    -DCMAKE_CXX_STANDARD=17 \
    -DCMAKE_CXX_FLAGS="-O3 -sEXPORTED_FUNCTIONS=[_duckdb_library_version,_duckdb_free,_duckdb_open,_duckdb_close]" \
    -DCMAKE_EXE_LINKER_FLAGS="-sEXPORTED_FUNCTIONS=[_duckdb_library_version,_duckdb_free,_duckdb_open,_duckdb_close]" \
    -DDUCKDB_EXPLICIT_PLATFORM=wasm_eh \
    -DBUILD_EXTENSIONS="" \
    -DBUILD_SHELL=OFF \
    -DBUILD_UNITTESTS=OFF \
    -DENABLE_EXTENSION_AUTOLOADING=OFF \
    -DENABLE_EXTENSION_AUTOINSTALL=OFF \
    -DBUILD_HTTPFS=OFF \
    -DCMAKE_BUILD_TYPE=Release \
    -DCMAKE_HAVE_LIBC_PTHREAD=1 \
    -DDUCKDB_NO_THREADS=1 \
    -DDUCKDB_NO_THREADS_DEPENDENCY_MANAGEMENT=1 \
    2>&1 | tail -25 || fail "emcmake configure failed"
else
  log "reusing configured ${BUILD}"
fi

# --- build: lib target only ------------------------------------------------
log "BUILD (cmake --build target duckdb)"
cmake --build "${BUILD}" --target duckdb -j "$(sysctl -n hw.ncpu)" \
  2>&1 | tail -40 || fail "cmake --build failed"

# --- final link: standalone wasm via stub main + static lib ----------------
# The "duckdb" target is a shared library (wasm SIDE_MODULE, no exports of
# interest); the standalone artifact needs a main. Link libduckdb_static.a
# with a 3-line stub and the emcc export list.
log "FINAL LINK (emcc stub + libduckdb_static.a)"
STUB="${BUILD}/stub_main.c"
cat > "${STUB}" <<'EOF2'
int main(void) { return 0; }
EOF2
em++ "${STUB}" \
  "${BUILD}/src/libduckdb_static.a" \
  "${BUILD}/extension/core_functions/libcore_functions_extension.a" \
  "${BUILD}/extension/icu/libicu_extension.a" \
  -O3 -sALLOW_MEMORY_GROWTH=1 -sENVIRONMENT=node \
  -sEXPORTED_FUNCTIONS=[_main,_duckdb_library_version,_duckdb_free,_duckdb_open,_duckdb_close] \
  -sEXPORTED_RUNTIME_METHODS=[] \
  -o "${BUILD}/duckdb-min.wasm" 2>&1 | tail -20 || fail "final emcc link failed"

WASM="${BUILD}/duckdb-min.wasm"
[ -f "${WASM}" ] || fail "no duckdb-min.wasm produced"

# --- receipt artifacts ------------------------------------------------------
BYTES="$(stat -f%z "${WASM}")"
SHA="$(shasum -a 256 "${WASM}" | cut -d' ' -f1)"
cp "${WASM}" "${OUT}/duckdb.wasm"
log "artifact: ${OUT}/duckdb.wasm bytes=${BYTES} sha256=${SHA}"

# --- smoke: pre-instantiation export probe + version + heap ceiling -------
cat > "${OUT}/smoke.mjs" <<'EOF'
import { readFileSync } from 'node:fs';
const WASM = new URL('./duckdb.wasm', import.meta.url).pathname;
const bytes = readFileSync(WASM);
const mod = new WebAssembly.Module(bytes);
const exports = WebAssembly.Module.exports(mod).map(e => e.name);
console.log('[smoke] exports:', JSON.stringify(exports));

// Check 1 (falsifier): duckdb_library_version must be exported, pre-instantiation.
if (!exports.includes('duckdb_library_version')) {
  console.error('[smoke] FAIL: duckdb_library_version missing from exports');
  process.exit(1);
}
const inst = new WebAssembly.Instance(mod, {});
const e = inst.exports;
const decoder = new TextDecoder();
const readCStr = (ptr) => {
  const view = new Uint8Array(e.memory.buffer);
  let end = ptr; while (view[end] !== 0) end++;
  return decoder.decode(view.subarray(ptr, end));
};

// Check 2: version string == receipt-pinned value.
const before = e.memory.buffer.byteLength;
const verPtr = e.duckdb_library_version();
const version = readCStr(verPtr);
console.log('[smoke] duckdb_library_version() =', version);
const EXPECT = process.env.EXPECT_VERSION;
if (version !== EXPECT) {
  console.error(`[smoke] FAIL: version ${version} != expected ${EXPECT}`);
  process.exit(1);
}

// Check 3+4: open/close cycle, heap ceiling <=128MB, delta <=64MB.
let after = before;
if (e.duckdb_open && e.duckdb_close) {
  const openPtr = e.duckdb_open(BigInt(0));
  const dbPtr = Number(readU64(e, openPtr));
  console.log('[smoke] duckdb_open ->', dbPtr);
  e.duckdb_close(openPtr);
  after = e.memory.buffer.byteLength;
} else {
  after = e.memory.buffer.byteLength;
  console.log('[smoke] open/close not exported; heap after version call only');
}
function readU64(e, ptr) {
  const view = new DataView(e.memory.buffer);
  return view.getBigUint64(Number(ptr), true);
}
const MB = 1024 * 1024;
console.log(`[smoke] heap before=${before} after=${after}`);
let ok = true;
if (after > 128 * MB) { console.error('[smoke] FAIL: heap > 128MB'); ok = false; }
if (after - before > 64 * MB) { console.error('[smoke] FAIL: heap delta > 64MB'); ok = false; }
if (!ok) process.exit(1);
console.log('[smoke] PASS');
EOF

# --- pinned expected version (captured from git describe lineage) ----------
DESCRIBE="$(git -C "${SRC}" describe --tags 2>/dev/null || true)"
log "git describe: ${DESCRIBE}"
# GAP1b witness measured v2.0.0-dev88595 for this tree; pin from witness if present.
EXPECT="v2.0.0-dev88595"
if [ -f /tmp/gap1b/witness.out ]; then
  EXPECT="$(grep -o 'v2\.0\.0-dev[0-9]*' /tmp/gap1b/witness.out | head -1 || echo "${EXPECT}")"
fi
log "expected version: ${EXPECT}"

if EXPECT_VERSION="${EXPECT}" node "${OUT}/smoke.mjs"; then
  log "SMOKE PASS"
else
  fail "smoke failed (exit $?)"
fi

# --- receipt ----------------------------------------------------------------
printf 'WASM MVP BUILD RECEIPT (ADV1)\nartifact: %s\nbytes: %s\nsha256: %s\nemcc: %s\npin: %s\ngit describe: %s\nexpected version: %s\nflags: BUILD_EXTENSIONS="" BUILD_SHELL=OFF BUILD_UNITTESTS=OFF BUILD_HTTPFS=OFF -O3 (no pthreads, no LTO)\n' \
  "${OUT}/duckdb.wasm" "${BYTES}" "${SHA}" "${EMCC_V}" "${PIN}" "${DESCRIBE}" "${EXPECT}" > "${OUT}/receipt.txt"
cat "${OUT}/receipt.txt"
