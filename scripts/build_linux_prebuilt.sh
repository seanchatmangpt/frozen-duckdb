#!/usr/bin/env bash
# build_linux_prebuilt.sh — REL6: first Linux (linux/arm64) libduckdb.so prebuilt.
#
# Mirrors crates/frozen-duckdb-builder/src/lib.rs (SOURCE_PIN b6013f7...,
# BUILD_EXTENSIONS list at line 1208), minus httpfs unless OpenSSL is present.
# Build + verify happen in linux/arm64 containers (colima/docker).
#
# Usage: scripts/build_linux_prebuilt.sh [output-dir]   (default /tmp/rel6)
set -euo pipefail

PIN="b6013f7aa2f36caabb9466ce1791667ea8a280bb"
EXTS="parquet;json;icu;tpch;tpcds;fts;inet;sqlsmith"   # full set minus httpfs
OUT_DIR="${1:-/tmp/rel6}"
IMG="rust:1-bookworm"
CTR="rel6-duckdb-build"

mkdir -p "$OUT_DIR"

echo "== [1/4] Build container: clone pin + cmake Release =="
docker rm -f "$CTR" >/dev/null 2>&1 || true
docker run --name "$CTR" --platform linux/arm64 \
  -v "$OUT_DIR":/out \
  "$IMG" bash -c '
    set -euo pipefail
    export DEBIAN_FRONTEND=noninteractive
    apt-get update -qq
    apt-get install -y -qq git cmake make g++ libssl-dev >/dev/null
    git clone https://github.com/duckdb/duckdb.git /src
    cd /src && git checkout '"$PIN"'
    mkdir -p build && cd build
    cmake .. -DCMAKE_BUILD_TYPE=Release \
             -DBUILD_EXTENSIONS="'"$EXTS"'" \
             -DBUILD_JEMALLOC=ON \
             -DBUILD_AUTOLOAD=ON
    make -j"$(nproc)" duckdb
    find /src/build -name "libduckdb*.so" -print
    cp -v "$(find /src/build -name "libduckdb*.so" | head -1)" /out/libduckdb.so
    mkdir -p /out/duckdb
    cp -rv /src/src/include/duckdb.h /src/src/include/duckdb /out/duckdb/ 2>/dev/null || true
    cp -rv /src/tools/odbc/include/* /out/duckdb/ 2>/dev/null || true
  '
echo "== [2/4] Package: strip + tar.gz =="
docker run --rm --platform linux/arm64 -v "$OUT_DIR":/out "$IMG" bash -c '
    set -e
    strip /out/libduckdb.so
    cd /out
    tar -czf libduckdb_linux_arm64.tar.gz libduckdb.so duckdb/
'
ls -l "$OUT_DIR"

echo "== [3/4] Fresh-container consumer verification =="
docker run --rm --platform linux/arm64 -v "$OUT_DIR":/out:ro "$IMG" bash -c '
    set -e
    apt-get update -qq && apt-get install -y -qq g++ >/dev/null
    cd /tmp && cp -r /out/duckdb .
    cat > consumer.cpp <<EOF
#include "duckdb.h"
#include <cstdio>
int main() {
    duckdb_database db; duckdb_connection con; duckdb_result res;
    if (duckdb_open(NULL, &db)) return 2;
    if (duckdb_connect(db, &con)) return 3;
    if (duckdb_query(con, "SELECT version()", &res)) return 4;
    printf("version()=%s\n", duckdb_result_get_string(res, 0));
    duckdb_destroy_result(&res); duckdb_disconnect(&con); duckdb_close(&db);
    return 0;
}
EOF
    g++ -std=c++11 consumer.cpp -L/out -lduckdb -o consumer
    LD_LIBRARY_PATH=/out ./consumer
'
echo "== [4/4] Receipt hashes =="
shasum -a 256 "$OUT_DIR/libduckdb_linux_arm64.tar.gz"
