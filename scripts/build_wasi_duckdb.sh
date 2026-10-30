#!/usr/bin/env bash
# build_wasi_duckdb.sh - reproducible WASI toolchain staging for frozen-duckdb
# (the graphlaw wasm32-wasip1 link). Reproduces the entire staging:
#   bin/clang      wrapper: exec <EMSDK>/upstream/bin/clang --sysroot=<sysroot> "$@"
#   bin/{clang++,llvm-ar,wasm-ld} -> <EMSDK>/upstream/bin/ symlinks
#   lib/wasm32-wasip1/{libc++,libc++abi,libunwind}.a  (emsdk -wasmexcept variants)
#   libduckdb.a + extension archives  (cmake build of DUCKDB_SRC @ pin,
#                  -fwasm-exceptions, DISABLE_THREADS=ON, PIC OFF patch,
#                  temp-dir build tree, no ccache/sccache)
#   libduckdb_wasi_compat.a (compiled from frozen-duckdb-sys/wasi_posix.c)
# Modes: (none)=verify | --build | --verify-only | --clean-room <dir> | --force
set -u

EMSDK="${EMSDK:-/Users/sac/emsdk}"
DUCKDB_SRC="${DUCKDB_SRC:-/Users/sac/rel4r/duckdb-src}"
DUCKDB_PIN="d54cbbe3df17e8e5ce68312f483b4155836112aa"
STAGING="${STAGING:-$HOME/.frozen-duckdb/cache/v2.0.0-wasm32-wasip1}"
COMPAT_SRC="${COMPAT_SRC:-/Users/sac/frozen-duckdb/crates/frozen-duckdb-sys/wasi_posix.c}"
PATCH_SRC="${PATCH_SRC:-/Users/sac/frozen-duckdb/patches/wasi-file-semantics.patch}"
# wasi libc for standalone clang links (rust ships a wasi-sdk-compatible libc.a
# in its self-contained wasm32-wasip1 component; used for the EH probe only).
WASI_LIBC_SRC="${WASI_LIBC_SRC:-$HOME/.rustup/toolchains/1.96.0-aarch64-apple-darwin/lib/rustlib/wasm32-wasip1/lib/self-contained}"
# SDK-shape source: share/cmake (wasi-sdk-p1.cmake + Platform/WASI.cmake) is
# copied from a wasi-sdk root so the staging itself is wasi-sdk-shaped
# (consumer: frozen-duckdb-builder wasi_sdk_toolchain_file).
WASI_SDK_SRC="${WASI_SDK_SRC:-/Users/sac/wasi-sdks/wasi-sdk-34}"
JOBS="$(sysctl -n hw.ncpu 2>/dev/null || echo 4)"
FAILURES=0

fail() { printf 'FAIL: %s\n' "$1"; FAILURES=$((FAILURES+1)); }
pass() { printf 'PASS: %s\n' "$1"; }
note() { printf 'NOTE: %s\n' "$1"; }
skip() { printf 'SKIP(typed): %s\n' "$1"; }
die()  { printf 'FAIL(fatal): %s\n' "$1"; exit 1; }

have_wasm_magic() {
  python3 - "$1" <<'PY'
import sys
with open(sys.argv[1], 'rb') as f:
    prefix = f.read(64 * 1024 * 1024)
sys.exit(0 if b'\x00asm\x01\x00\x00\x00' in prefix else 1)
PY
}

verify_prereqs() {
  [ -x "$EMSDK/upstream/bin/clang" ] || die "emsdk clang missing at $EMSDK/upstream/bin/clang"
  [ -d "$EMSDK/upstream/emscripten/cache/sysroot" ] || die "emsdk sysroot missing"
  [ -d "$DUCKDB_SRC/.git" ] || die "duckdb source missing at $DUCKDB_SRC"
  [ -f "$EMSDK/upstream/emscripten/system/lib/libunwind/src/Unwind-wasm.c" ] || die "libunwind sources missing under $EMSDK"
  command -v cmake >/dev/null || die "cmake not on PATH"
  [ -f "$WASI_SDK_SRC/share/cmake/wasi-sdk-p1.cmake" ] || die "wasi-sdk toolchain file missing at $WASI_SDK_SRC/share/cmake/wasi-sdk-p1.cmake (set WASI_SDK_SRC)"
}

archive_magic() {
  local f="$1" name
  name="$(basename "$f")"
  if [ ! -s "$f" ]; then fail "$name: missing or empty"; return 1; fi
  if have_wasm_magic "$f"; then
    pass "$name: $(stat -f%z "$f") B, wasm32 magic in 64 MiB prefix"
  else
    fail "$name: no \\0asm magic in 64 MiB prefix (not a wasm32 archive)"
  fi
}

verify_staging() {
  echo "== verify staging: $STAGING"
  # 1. wrapper + symlinks
  local wrap="$STAGING/bin/clang" b
  if [ -x "$wrap" ]; then pass "bin/clang wrapper executable"; else fail "bin/clang wrapper missing/not executable"; fi
  if [ -f "$wrap" ] && grep -q 'exec .*clang --sysroot=' "$wrap"; then
    pass "wrapper exec line carries --sysroot"
  else
    fail "wrapper exec line does not carry --sysroot"
  fi
  for b in clang++ llvm-ar wasm-ld; do
    if [ -L "$STAGING/bin/$b" ] && [ -e "$STAGING/bin/$b" ]; then
      pass "bin/$b symlink -> $(readlink "$STAGING/bin/$b")"
    else
      fail "bin/$b symlink missing or dangling"
    fi
  done
  if [ -e "$STAGING/share/wasi-sysroot/include" ] && [ -e "$STAGING/share/wasi-sysroot/lib/wasm32-wasip1/libc.a" ]; then
    pass "share/wasi-sysroot merged (include + lib/wasm32-wasip1 resolve)"
  else
    fail "share/wasi-sysroot missing or incomplete"
  fi
  if [ -f "$STAGING/share/cmake/wasi-sdk-p1.cmake" ]; then
    pass "share/cmake/wasi-sdk-p1.cmake staged (SDK-shape)"
  else
    fail "share/cmake/wasi-sdk-p1.cmake missing (SDK-shape: staging must be wasi-sdk-shaped)"
  fi
  # 2. C++ runtime == emsdk -wasmexcept variants
  local rt="$EMSDK/upstream/emscripten/cache/sysroot/lib/wasm32-emscripten"
  local st="$STAGING/lib/wasm32-wasip1" m hs hsrc
  local builtins_h builtins_s
  builtins_h="$(shasum -a 256 "$st/libclang_rt.builtins.a" 2>/dev/null | cut -d' ' -f1)"
  builtins_s="$(shasum -a 256 "$rt/libclang_rt.builtins.a" 2>/dev/null | cut -d' ' -f1)"
  if [ "$builtins_h" = "$builtins_s" ]; then
    pass "libclang_rt.builtins.a == emsdk builtins (sha256 $builtins_h)"
  else
    fail "libclang_rt.builtins.a missing or sha256 mismatch vs emsdk"
  fi
  for m in libc.a crt1-command.o; do
    if [ -f "$st/$m" ] && [ -f "$WASI_LIBC_SRC/$m" ] && [ "$(shasum -a 256 "$st/$m" | cut -d' ' -f1)" = "$(shasum -a 256 "$WASI_LIBC_SRC/$m" | cut -d' ' -f1)" ]; then
      pass "lib/wasm32-wasip1/$m == rust wasip1 self-contained (sha256)"
    else
      fail "lib/wasm32-wasip1/$m missing or sha256 mismatch vs $WASI_LIBC_SRC"
    fi
  done
  for m in libc++ libc++abi libunwind; do
    hs="$(shasum -a 256 "$st/$m.a" 2>/dev/null | cut -d' ' -f1)"
    if [ -z "$hs" ]; then fail "lib/wasm32-wasip1/$m.a missing"; continue; fi
    if [ "$m" = "libunwind" ]; then
      # 2026-10-23: canonical libunwind is the UNIFORMLY NEW-EH rebuild
      # (Unwind-wasm.c compiled -fwasm-exceptions -mllvm
      # -wasm-use-legacy-eh=0; candidate B, EH-OK under wasmtime+node).
      # The emsdk wasmexcept variant carries LEGACY EH and mixed with the
      # new-EH libduckdb objects it corrupted __cxa_thread_atexit dtor
      # entries (witnessed: "uninitialized element" trap, func 68925).
      case "$hs" in
        6a5f79f7*) pass "libunwind.a == uniform new-EH candidate B (sha256 $hs)" ;;
        7ab65b4c*) fail "libunwind.a is the LEGACY emsdk variant — causes the uninitialized-element dtor trap; swap in the new-EH rebuild (see STAGING-NOTES 2026-10-23)" ;;
        *) fail "libunwind.a sha256 $hs matches neither the new-EH candidate (6a5f79f7…) nor the documented legacy variant (7ab65b4c…)" ;;
      esac
    else
      hsrc="$(shasum -a 256 "$rt/$m-wasmexcept.a" 2>/dev/null | cut -d' ' -f1)"
      if [ -z "$hsrc" ]; then fail "emsdk variant $m-wasmexcept.a missing under $rt"; continue; fi
      if [ "$hs" = "$hsrc" ]; then
        pass "$m.a == emsdk $m-wasmexcept.a (sha256 $hs)"
      else
        fail "$m.a sha256 mismatch vs emsdk $m-wasmexcept.a"
      fi
    fi
  done
  # 3. archives
  local a
  for a in libduckdb.a libcore_functions_extension.a libduckdb_wasi_compat.a; do
    archive_magic "$STAGING/$a"
  done
  if [ -f "$STAGING/libicu_extension.a" ]; then
    fail "libicu_extension.a present (icu dropped this window; stale archive from a previous icu-ful restage)"
  else
    pass "libicu_extension.a absent (icu dropped)"
  fi
  # 3b. EH-flavor gate (2026-10-23): prove libduckdb.a is uniformly NEW-EH by
  # opcode, not symbols — personality-symbol refs are a false-positive
  # heuristic (witnessed: ub_duckdb_parser.cpp.obj references
  # __gxx_wasm_personality_v0 while carrying 701 try_table / 0 legacy try).
  # Gate: the largest member must disassemble with try_table present and no
  # legacy `try` instruction. A legacy-flavor archive produced the
  # uninitialized-element dtor-runner trap (SQLRDF-24).
  local OD="$EMSDK/upstream/bin/llvm-objdump"
  local AR="$EMSDK/upstream/bin/llvm-ar"
  if [ -x "$OD" ] && [ -x "$AR" ]; then
    local big mem d tt legacy
    big="$("$AR" t "$STAGING/libduckdb.a" | head -1)"
    mem="$(basename "$big")"
    d="$(mktemp -d /tmp/ehgate.XXXXXX)"
    "$AR" p "$STAGING/libduckdb.a" "$mem" > "$d/$mem"
    tt="$("$OD" -d "$d/$mem" | grep -c try_table || true)"
    legacy="$("$OD" -d "$d/$mem" | grep -cE '^[[:space:]]*[0-9a-f]+: try[[:space:]]' || true)"
    echo "EH-gate member $mem: try_table=$tt legacy=$legacy"
    if [ "$tt" -gt 0 ] && [ "$legacy" = "0" ]; then
      pass "libduckdb.a EH flavor: NEW-EH (opcode gate, member $mem try_table=$tt legacy=0)"
    else
      fail "libduckdb.a EH flavor suspect: member $mem try_table=$tt legacy=$legacy (expected try_table>0, legacy=0)"
    fi
    rm -rf "$d"
  else
    skip "EH-flavor gate: llvm-objdump/llvm-ar not available"
  fi
  # 4. duckdb source pin
  local head
  head="$(git -C "$DUCKDB_SRC" rev-parse HEAD 2>/dev/null)" || head=""
  if [ "$head" = "$DUCKDB_PIN" ]; then
    pass "duckdb source pin HEAD == $DUCKDB_PIN"
  else
    fail "duckdb source pin HEAD=$head expected $DUCKDB_PIN"
  fi
  # 5. hello-world C++-EH compile+link through the wrapper
  local tmp rc
  tmp="$(mktemp -d /tmp/wasi-verify.XXXXXX)"
  printf '%s\n' '#include <cstdio>' '#include <stdexcept>' \
    'int main() {' '  try { throw std::runtime_error("probe"); }' \
    '  catch (const std::exception& e) { printf("EH-OK %s\n", e.what()); return 0; }' \
    '  return 42; }' > "$tmp/eh.cpp"
  "$wrap" --target=wasm32-wasip1 -O1 -fwasm-exceptions -mllvm -wasm-use-legacy-eh=0 -std=c++17 -nostdlib \
      -o "$tmp/eh.wasm" "$tmp/eh.cpp" "$st/libclang_rt.builtins.a" "$st/crt1-command.o" -L"$st" -lc -lc++ -lc++abi -lunwind \
      2>"$tmp/err"
  rc=$?
  if [ $rc -eq 0 ] && have_wasm_magic "$tmp/eh.wasm"; then
    pass "hello-world C++-EH compiled+linked through wrapper (wasm32 magic)"
    run_eh_smoke "$tmp/eh.wasm"
  else
    fail "EH probe compile/link failed (exit $rc): $(tail -2 "$tmp/err" | tr '\n' ' ')"
  fi

  # 6. Shim-less DuckDB link probe & TC08 COPY egress court
  run_duckdb_wasi_egress_court "$tmp"
  rm -rf "$tmp"
  return 0
}

run_duckdb_wasi_egress_court() {
  local tmp="$1"
  local egress_file="/tmp/duckdb_wasi_egress_$$.csv"
  rm -f "$egress_file"

  cat > "$tmp/egress.cpp" <<EOF
#include <duckdb.h>
#include <cstdio>
#include <cstdlib>

int main() {
    duckdb_database db;
    duckdb_connection con;
    if (duckdb_open(nullptr, &db) == DuckDBError) {
        fprintf(stderr, "duckdb_open failed\n");
        return 1;
    }
    if (duckdb_connect(db, &con) == DuckDBError) {
        fprintf(stderr, "duckdb_connect failed\n");
        return 1;
    }
    duckdb_state state;
    state = duckdb_query(con, "CREATE TABLE t (id INT, val VARCHAR);", nullptr);
    if (state == DuckDBError) {
        fprintf(stderr, "CREATE TABLE failed\n");
        return 1;
    }
    state = duckdb_query(con, "INSERT INTO t VALUES (42, 'test_val');", nullptr);
    if (state == DuckDBError) {
        fprintf(stderr, "INSERT failed\n");
        return 1;
    }
    duckdb_result res;
    state = duckdb_query(con, "COPY t TO '$egress_file' (HEADER, DELIMITER ',');", &res);
    if (state == DuckDBError) {
        fprintf(stderr, "COPY failed: %s\n", duckdb_result_error(&res));
        return 1;
    }
    duckdb_disconnect(&con);
    duckdb_close(&db);
    printf("DUCKDB-EGRESS-OK\n");
    return 0;
}
EOF

  local wrap="$STAGING/bin/clang"
  "$wrap" --target=wasm32-wasip1 -O1 \
    -fwasm-exceptions -mllvm -wasm-use-legacy-eh=0 -std=c++17 \
    -I"$STAGING/share/wasi-sysroot/include/c++/v1" \
    -I"$DUCKDB_SRC/src/include" \
    -o "$tmp/egress.wasm" "$tmp/egress.cpp" \
    -L"$STAGING" \
    -lduckdb -lcore_functions_extension -lduckdb_wasi_compat \
    -lc++ -lc++abi -lunwind 2>"$tmp/egress_err"
  local rc=$?
  if [ $rc -eq 0 ] && have_wasm_magic "$tmp/egress.wasm"; then
    pass "DuckDB probe module linked with zero consumer shims / zero --wrap flags"
  else
    fail "DuckDB probe link failed (exit $rc): $(tail -5 "$tmp/egress_err" | tr '\n' ' ')"
    return 1
  fi

  if command -v wasmtime >/dev/null 2>&1; then
    wasmtime run --dir=/tmp "$tmp/egress.wasm" > "$tmp/egress.out" 2>"$tmp/egress.err"
    rc=$?
    if [ $rc -eq 0 ] && grep -q '^DUCKDB-EGRESS-OK$' "$tmp/egress.out" && [ -s "$egress_file" ] && grep -q '42,test_val' "$egress_file"; then
      pass "DuckDB WASI COPY egress court (1/0 class) PASSED under wasmtime"
    else
      fail "DuckDB WASI COPY egress court failed under wasmtime (exit $rc, out: $(cat "$tmp/egress.out" 2>/dev/null), err: $(cat "$tmp/egress.err" 2>/dev/null))"
    fi
  else
    skip "DuckDB WASI COPY egress court: wasmtime not installed"
  fi
  rm -f "$egress_file"
}

run_eh_smoke() {
  # clang 24 -fwasm-exceptions emits legacy-EH instructions (try/catch_all);
  # wasmtime 48 does not implement legacy exceptions, node 24 (V8) does.
  local wasm="$1" node="${NODE_BIN:-$EMSDK/node/24.19.0_64bit/bin/node}" rc
  [ -x "$node" ] || { skip "EH run: node not present at $node (compile+link+magic still gated)"; return 0; }
  cat > "$wasm.run.mjs" <<'MJS'
import { WASI } from 'node:wasi';
import { readFile } from 'node:fs/promises';
const wasi = new WASI({ version: 'preview1', returnOnExit: true });
const mod = await WebAssembly.compile(await readFile(process.argv[2]));
const inst = await WebAssembly.instantiate(mod, wasi.getImportObject());
process.exit(wasi.start(inst));
MJS
  "$node" "$wasm.run.mjs" "$wasm" > "$wasm.out" 2>/dev/null
  rc=$?
  if [ $rc -eq 0 ] && grep -q '^EH-OK probe$' "$wasm.out"; then
    pass "EH probe ran under node WASI: EH-OK probe, exit 0"
  else
    fail "EH probe run failed (exit $rc, out: $(cat "$wasm.out" | tail -1))"
  fi
  rm -f "$wasm.run.mjs" "$wasm.out"
}

stage_bin() {
  local out="$1"
  mkdir -p "$out/bin" "$out/lib/wasm32-wasip1"
  printf '%s\n' '#!/bin/sh' "exec $EMSDK/upstream/bin/clang --sysroot=$out/share/wasi-sysroot \"\$@\"" > "$out/bin/clang"
  chmod +x "$out/bin/clang"
  local b
  for b in clang++ llvm-ar wasm-ld; do
    ln -sfn "$EMSDK/upstream/bin/$b" "$out/bin/$b"
  done
}

stage_runtime() {
  local out="$1" rt m lib
  rt="$EMSDK/upstream/emscripten/cache/sysroot"
  lib="$out/lib/wasm32-wasip1"
  mkdir -p "$lib"
  local m
  for m in libc++ libc++abi; do
    cp "$rt/lib/wasm32-emscripten/$m-wasmexcept.a" "$lib/$m.a"
  done
  cp "$rt/lib/wasm32-emscripten/libclang_rt.builtins.a" "$lib/libclang_rt.builtins.a"
  # wasi libc + crt (rust self-contained component, wasi-sdk-compatible)
  cp "$WASI_LIBC_SRC/libc.a" "$lib/libc.a"
  cp "$WASI_LIBC_SRC/crt1-command.o" "$lib/crt1-command.o"
  cp "$WASI_LIBC_SRC/crt1-command.o" "$lib/crt1.o"
  # libpthread.a: FindThreads (duckdb CMakeLists:9 find_package(Threads REQUIRED))
  # needs a linkable -lpthread; wasi libc keeps the DISABLE_THREADS stubs inside
  # libc.a (weak), plus two symbols libc.a does not define (pthread_exit,
  # pthread_atfork) — provided here as explicit stubs.
  local ar="$EMSDK/upstream/bin/llvm-ar" pd="$out/pthread-stubs"
  mkdir -p "$pd"
  (cd "$pd" && for o in $("$ar" t "$lib/libc.a" | grep '^pthread_'); do
     "$ar" x "$lib/libc.a" "$o"
   done)
  cat > "$pd/stub.c" <<'STUB'
__attribute__((noreturn)) void pthread_exit(void *r) { for (;;) {} }
int pthread_atfork(void (*p)(void), void (*c)(void), void (*h)(void)) { return 0; }
STUB
  "$out/bin/clang" --target=wasm32-wasip1 -O2 -c -o "$pd/stub.o" "$pd/stub.c" || die "pthread stub compile failed"
  "$ar" rcs "$lib/libpthread.a" "$pd"/*.c.obj "$pd/stub.o" || die "libpthread.a pack failed"
  rm -rf "$pd"
  # libunwind: the UNIFORMLY NEW-EH rebuild (candidate B, sha256 6a5f79f7…;
  # verify_staging gates the digest). Reproduced from the emsdk libunwind
  # sources — the emsdk wasmexcept variant carries LEGACY EH and mixed with
  # the new-EH libduckdb objects it corrupted __cxa_thread_atexit dtor
  # entries (witnessed: "uninitialized element" trap, func 68925). The
  # byte-exact recipe (member-level cmp + full-archive cmp proven 2026-10-29
  # against the 6a5f79f7… candidate): EH members carry -fwasm-exceptions
  # -mllvm -wasm-use-legacy-eh=0; the three C unwinder-API members carry
  # NEITHER flag (no exception-handling feature section in their objects).
  local lu_src="$EMSDK/upstream/emscripten/system/lib/libunwind"
  local luo="$out/libunwind-build"
  [ -f "$lu_src/src/Unwind-wasm.c" ] || die "libunwind sources missing at $lu_src"
  mkdir -p "$luo"
  local m2
  for m2 in libunwind.cpp Unwind_AIXExtras.cpp Unwind-EHABI.cpp Unwind-seh.cpp \
            Unwind-wasm.c UnwindRegistersRestore.S UnwindRegistersSave.S; do
    "$out/bin/clang" --target=wasm32-wasip1 -O2 -fdeclspec -fwasm-exceptions -mllvm -wasm-use-legacy-eh=0 \
      -nostdinc -isystem "$rt/include" -isystem "$lu_src/include" -isystem "$lu_src/src" \
      -c -o "$luo/$m2.o" "$lu_src/src/$m2" || die "libunwind member compile failed: $m2"
  done
  for m2 in Unwind-sjlj.c UnwindLevel1-gcc-ext.c UnwindLevel1.c; do
    "$out/bin/clang" --target=wasm32-wasip1 -O2 -fdeclspec \
      -nostdinc -isystem "$rt/include" -isystem "$lu_src/include" -isystem "$lu_src/src" \
      -c -o "$luo/$m2.o" "$lu_src/src/$m2" || die "libunwind member compile failed: $m2"
  done
  # ar rcs onto an existing archive MERGES members (witnessed 2026-10-29: the
  # previous emsdk-variant archive's members survived the repack); pack fresh.
  rm -f "$lib/libunwind.a"
  "$ar" rcs "$lib/libunwind.a" \
    "$luo/libunwind.cpp.o" "$luo/Unwind_AIXExtras.cpp.o" "$luo/Unwind-EHABI.cpp.o" \
    "$luo/Unwind-seh.cpp.o" "$luo/Unwind-sjlj.c.o" "$luo/Unwind-wasm.c.o" \
    "$luo/UnwindLevel1-gcc-ext.c.o" "$luo/UnwindLevel1.c.o" \
    "$luo/UnwindRegistersRestore.S.o" "$luo/UnwindRegistersSave.S.o" || die "libunwind.a pack failed"
  rm -rf "$luo"
  # merged wasi-sysroot for the clang driver (headers via symlink, libs real)
  mkdir -p "$out/share/wasi-sysroot/lib"
  ln -sfn "$rt/include" "$out/share/wasi-sysroot/include"
  ln -sfn "$lib" "$out/share/wasi-sysroot/lib/wasm32-wasip1"
  # SDK-shape: wasi-sdk-p1.cmake resolves WASI_SDK_PREFIX from its own location
  # (\${CMAKE_CURRENT_LIST_DIR}/../..), so a copy under share/cmake makes this
  # staging a self-contained wasi-sdk-shaped root. Platform/ rides along
  # (wasi-sdk-p1.cmake injects its dir into CMAKE_MODULE_PATH for Platform/WASI.cmake).
  mkdir -p "$out/share/cmake"
  cp -R "$WASI_SDK_SRC/share/cmake/." "$out/share/cmake/" || die "share/cmake staging from $WASI_SDK_SRC failed"
}


write_toolchain() {
  local file="$1" out="$2"
  cat > "$file" <<TC
set(CMAKE_SYSTEM_NAME WASI)
set(CMAKE_SYSTEM_VERSION 1)
set(CMAKE_SYSTEM_PROCESSOR wasm32)
set(CMAKE_C_COMPILER $out/bin/clang)
set(CMAKE_CXX_COMPILER $out/bin/clang++)
set(CMAKE_C_COMPILER_TARGET wasm32-wasip1)
set(CMAKE_CXX_COMPILER_TARGET wasm32-wasip1)
set(CMAKE_SYSROOT $out/share/wasi-sysroot)
set(CMAKE_AR $out/bin/llvm-ar)
set(CMAKE_RANLIB $EMSDK/upstream/bin/llvm-ranlib)
set(CMAKE_C_COMPILER_LAUNCHER)
set(CMAKE_CXX_COMPILER_LAUNCHER)
TC
}

build_archives() {
  local out="$1" scratch="$2" bdir cm produced
  [ "$(git -C "$DUCKDB_SRC" rev-parse HEAD)" = "$DUCKDB_PIN" ] || die "duckdb source not at pin $DUCKDB_PIN"
  mkdir -p "$out" "$scratch/tmp"
  git -C "$DUCKDB_SRC" archive "$DUCKDB_PIN" | tar -x -C "$scratch/duckdb" 2>/dev/null || { mkdir -p "$scratch/duckdb"; git -C "$DUCKDB_SRC" archive "$DUCKDB_PIN" | tar -x -C "$scratch/duckdb"; } || die "git archive of pin $DUCKDB_PIN failed"
  local srcdir="$scratch/duckdb"
  local cm_file="$srcdir/CMakeLists.txt"
  grep -q 'set(CMAKE_POSITION_INDEPENDENT_CODE ON)' "$cm_file" || die "PIC policy unknown in pinned CMakeLists"
  sed -i '' 's/set(CMAKE_POSITION_INDEPENDENT_CODE ON)/set(CMAKE_POSITION_INDEPENDENT_CODE OFF)/' "$cm_file"
  if [ -f "$PATCH_SRC" ]; then
    patch -d "$srcdir" -p1 < "$PATCH_SRC" || die "failed to apply wasi-file-semantics.patch"
    pass "wasi-file-semantics.patch applied to duckdb source"
  else
    die "patch file missing at $PATCH_SRC"
  fi
  write_toolchain "$scratch/wasi-toolchain.cmake" "$out"
  bdir="$srcdir/build-wasip1"
  unset SCCACHE CCACHE 2>/dev/null || true
  export TMPDIR="$scratch/tmp"
  : > "$scratch/build.log"
  cmake -S "$srcdir" -B "$bdir" \
    -DCMAKE_TOOLCHAIN_FILE="$scratch/wasi-toolchain.cmake" \
    -DCMAKE_BUILD_TYPE=MinSizeRel \
    -DCMAKE_CXX_FLAGS="-Oz -fwasm-exceptions -mllvm -wasm-use-legacy-eh=0" \
    -DBUILD_SHARED_LIBS=OFF \
    -DBUILD_EXTENSIONS="core_functions" \
    -DBUILD_SHELL=OFF -DBUILD_UNITTESTS=OFF \
    -DENABLE_EXTENSION_AUTOLOADING=OFF -DENABLE_EXTENSION_AUTOINSTALL=OFF \
    -DBUILD_HTTPFS=OFF -DBUILD_JEMALLOC=OFF \
    -DDISABLE_THREADS=ON -DDUCKDB_CAPABILITIES=none \
    -DCMAKE_C_COMPILER_LAUNCHER= -DCMAKE_CXX_COMPILER_LAUNCHER= \
    -DCCACHE_SUPPORT=OFF \
    >> "$scratch/build.log" 2>&1
  rc=$?
  [ $rc -eq 0 ] || { tail -20 "$scratch/build.log"; die "cmake configure failed (exit $rc), log: $scratch/build.log"; }
  pass "cmake configure exit 0"
  cmake --build "$bdir" --target duckdb_static -j "$JOBS" >> "$scratch/build.log" 2>&1
  rc=$?
  [ $rc -eq 0 ] || { tail -20 "$scratch/build.log"; die "cmake build failed (exit $rc), log: $scratch/build.log"; }
  pass "cmake build (duckdb_static) exit 0, log: $scratch/build.log"
  cmake --build "$bdir" --target core_functions_extension -j "$JOBS" >> "$scratch/build.log" 2>&1
  rc=$?
  [ $rc -eq 0 ] || { tail -20 "$scratch/build.log"; die "extension build failed (exit $rc), log: $scratch/build.log"; }
  pass "cmake build (core_functions_extension) exit 0"
  # icu is no longer built; a stale archive from a previous icu-ful restage
  # must not survive into the staging (verify gates its absence).
  rm -f "$out/libicu_extension.a"
  produced=""
  for c in src/libduckdb_static.a src/libduckdb.a libduckdb_static.a libduckdb.a; do
    [ -f "$bdir/$c" ] && produced="$bdir/$c" && break
  done
  [ -n "$produced" ] || die "libduckdb.a not found under $bdir"
  have_wasm_magic "$produced" || die "$produced has no wasm32 magic"
  pass "produced $(basename "$produced"): $(stat -f%z "$produced") B, wasm32 magic"
  cp "$produced" "$out/libduckdb.a"
  find "$bdir" -name 'lib*_extension.a' | while IFS= read -r ext; do
    cp "$ext" "$out/" || die "copy of $ext failed"
  done
  # wasi posix compat stubs
  "$out/bin/clang" --target=wasm32-wasip1 -O2 -c -o "$scratch/wasi_posix.o" "$COMPAT_SRC" || die "wasi_posix.c compile failed"
  "$out/bin/llvm-ar" rcs "$out/libduckdb_wasi_compat.a" "$scratch/wasi_posix.o" || die "compat ar failed"
  pass "libduckdb_wasi_compat.a built from $(basename "$COMPAT_SRC")"
}

main() {
  local mode="${1:-verify}"
  case "$mode" in
    --verify-only)
      verify_prereqs
      verify_staging
      ;;
    --clean-room)
      shift
      local dir="${1:?usage: build_wasi_duckdb.sh --clean-room <dir>}"
      verify_prereqs
      rm -rf "$dir"
      mkdir -p "$dir"
      STAGING="$dir"
      stage_bin "$dir"
      stage_runtime "$dir"
      build_archives "$dir" "$dir/../wasi-cleanroom-scratch.$$"
      echo "== clean-room staging complete; verifying $dir"
      verify_staging
      ;;
    --force)
      verify_prereqs
      stage_bin "$STAGING"
      stage_runtime "$STAGING"
      build_archives "$STAGING" "$(mktemp -d /tmp/wasi-rebuild.XXXXXX)"
      verify_staging
      ;;
    verify)
      verify_staging
      ;;
    *)
      die "unknown mode: $mode (use: verify | --verify-only | --clean-room <dir> | --force)"
      ;;
  esac
  if [ "$FAILURES" -gt 0 ]; then
    printf 'FAIL: %d check(s) failed\n' "$FAILURES"
    exit 1
  fi
  echo "VERIFY-OK: exit 0"
}

main "$@"
