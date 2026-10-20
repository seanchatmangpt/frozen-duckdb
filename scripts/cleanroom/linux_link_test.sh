#!/usr/bin/env bash
# REL4 lane: Linux ELF linking story for the frozen-duckdb consumer.
# Runs INSIDE the linux/arm64 container (rust:1-bookworm / duckdb build image).
#
# Phases:
#   P1  compile libduckdb.so at pin b6013f7 (low-RAM flags)
#   P2  build a C consumer binary two ways:
#         A) bare link (no rpath)  -> expect load failure in fresh container
#         B) -Wl,-rpath,/usr/local/lib at link time -> expect clean load, no env
#   P3  soname audit (readelf -d): SONAME presence, RUNPATH recording
#   P4  fresh-container replay (host side, see rel4_host.sh) — this script
#       emits /out/rel4_artifacts.tgz for that step.
#
# Output: /out/ (mounted host dir). Exit 0 only if all phases pass.
set -euo pipefail
PIN="b6013f7aa2f36caabb9466ce1791667ea8a280bb"
OUT=/out
mkdir -p "$OUT"

echo "=== REL4 P0: environment ==="
uname -a
nproc; free -m | head -2
gcc --version | head -1; ld --version | head -1
command -v patchelf || echo "patchelf: not installed (lane does not require it)"

echo "=== REL4 P1: fetch + compile libduckdb.so at pin b6013f7 ==="
if [ ! -f "$OUT/libduckdb.so" ]; then
  mkdir -p /build && cd /build
  if [ ! -d duckdb/.git ]; then
    git init -q duckdb && cd duckdb
    git remote add origin https://github.com/duckdb/duckdb.git
    git fetch -q --depth 1 origin "$PIN"
    git checkout -q FETCH_HEAD
    git rev-parse HEAD
    cd /build/duckdb
  fi
  cd /build/duckdb
  # low-RAM cmake build, shared lib only
  apt-get update -qq && apt-get install -y -qq cmake >/dev/null
  cmake -S . -B build \
    -DCMAKE_BUILD_TYPE=Release -DBUILD_SHELL=0 -DBUILD_UNITTESTS=0 \
    -DBUILD_PARQUET_EXTENSION=0 -DBUILD_TPCH_EXTENSION=0 \
    -DBUILD_TPCDS_EXTENSION=0 -DBUILD_JSON_EXTENSION=0 \
    -DENABLE_SANITIZER=0 -DENABLE_UBSAN=0 \
    -DCMAKE_EXE_LINKER_FLAGS="-latomic" \
    -DCMAKE_SHARED_LINKER_FLAGS="-latomic"
  cmake --build build --target duckdb -j "$(nproc)"
  find build -name 'libduckdb.so' -exec cp {} "$OUT/" \;
fi
ls -l "$OUT/libduckdb.so"
echo "=== REL4 P1: soname audit of libduckdb.so ==="
readelf -d "$OUT/libduckdb.so" | grep -E 'SONAME|NEEDED' | head

echo "=== REL4 P2: consumer binary ==="
mkdir -p /consumer && cd /consumer
cat > consumer.c <<'EOF'
#include <stdio.h>
#include <duckdb.h>
int main(void) {
  duckdb_database db; duckdb_connection con; duckdb_result res;
  if (duckdb_open(NULL, &db)) { puts("FAIL open"); return 1; }
  if (duckdb_connect(db, &con)) { puts("FAIL connect"); return 1; }
  if (duckdb_query(con, "SELECT 40+2", &res)) { puts("FAIL query"); return 1; }
  printf("RESULT=%lld\n", (long long)duckdb_value_int64(&res, 0, 0));
  duckdb_destroy_result(&res); duckdb_disconnect(&con); duckdb_close(&db);
  return 0;
}
EOF
gcc -O2 -I/build/duckdb/src/include -I/build/duckdb/third_party/libpg_query/include \
  consumer.c -o consumer_bare -L"$OUT" -lduckdb
gcc -O2 -I/build/duckdb/src/include -I/build/duckdb/third_party/libpg_query/include \
  consumer.c -o consumer_rpath -L"$OUT" -lduckdb -Wl,-rpath,/usr/local/lib
echo "--- readelf -d consumer_bare ---"
readelf -d consumer_bare | grep -E 'NEEDED|RUNPATH|RPATH'
echo "--- readelf -d consumer_rpath ---"
readelf -d consumer_rpath | grep -E 'NEEDED|RUNPATH|RPATH'
cp consumer_bare consumer_rpath "$OUT/"

echo "=== REL4 P2-A: bare consumer, no rpath, no env (expect FAIL) ==="
set +e
( cd / && env -i /consumer/consumer_bare; echo "exit=$?" ) 2>&1 | tail -3
echo "=== REL4 P2-B: rpath consumer, no env (expect RESULT=42) ==="
cp "$OUT/libduckdb.so" /usr/local/lib/ && ldconfig
( cd / && env -i /consumer/consumer_rpath; echo "exit=$?" )
set -e
echo "=== REL4 PASS ==="
