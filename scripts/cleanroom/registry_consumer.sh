#!/usr/bin/env bash
# registry_consumer.sh — REGISTRY-shape cleanroom (lane PB17), extends
# consumer_cleanroom.sh (ACT1, path-dep shape) to registry-shaped consumption.
#
# A downstream consumer whose Cargo.toml uses a registry req
# (frozen-duckdb = "=2.0.0-dev") resolved from packaged .crate artifacts:
#   1. one cargo vendor pass over the family's FULL external dep surface
#      (staging consumer -> all three checkout crates), producing the externals
#   2. `cargo package` the three crates in dependency order, with crates.io
#      source-replaced by a STITCHED directory registry:
#      cargo-vendor externals ∪ extracted .crate dirs, each stitched dir given
#      a generated .cargo-checksum.json (the registry checksum format). This is
#      required because cargo package demands registry-only resolution of path
#      deps and cargo vendor excludes path deps — neither alone suffices.
#   3. the consumer then resolves frozen-duckdb = "=2.0.0-dev" ONLY through
#      that same stitched directory registry (source replacement), builds
#      against a pre-seeded cache with ZERO DuckDB compile (ACT1's proof:
#      "Using prebuilt DuckDB binary" log line + link-time bound), and runs
#      the quickstart asserting DUCKDB_VERSION=v2.0.0-dev* and SUM=42.
#      cargo tree must show v2.0.0-dev with NO path into the repo checkout.
#
# Version fence (proven live, not assumed): the packaged version is 2.0.0-dev
# (workspace version, prerelease). Cargo's semver rule: req "2.0.0" does NOT
# match 2.0.0-dev. The court runs a negative-control resolution with req
# "2.0.0" and REQUIRES failure — a court that resolves "2.0.0" against a -dev
# package is vacuous. The literal task req "2.0.0" is unreachable until a real
# 2.0.0 release cuts non-dev versions; the fence is asserted instead of faking.
#
# Receipts: /tmp/pb17/registry-consumer-receipt-<ts>.json (+ .log, real output)
# Exit codes: 0 PASS · 1 FAIL · 2 loud SKIP / typed divergence
#
# Env knobs:
#   REGISTRY_CLEANROOM_REPO    repo root (default /Users/sac/frozen-duckdb)
#   REGISTRY_CLEANROOM_RECEIPT override receipt path
set -euo pipefail

REPO="${REGISTRY_CLEANROOM_REPO:-/Users/sac/frozen-duckdb}"
TS="$(date -u +%Y%m%dT%H%M%SZ)"
RECEIPT="${REGISTRY_CLEANROOM_RECEIPT:-/tmp/pb17/registry-consumer-receipt-${TS}.json}"
LOG="${RECEIPT%.json}.log"
mkdir -p "$(dirname "$RECEIPT")"
: >"$LOG"

log() { printf '%s %s\n' "$(date -u +%H:%M:%S)" "$*" | tee -a "$LOG"; }

SCRATCH_REAL_HOME="$HOME"
REPO_HEAD="$(git -C "$REPO" rev-parse HEAD 2>/dev/null || echo unknown)"
ARCH="$(uname -m)"
SCRATCH=""
REGISTRY_MECHANISM="stitched-directory-source: cargo-vendor externals + packaged .crate extracts with generated .cargo-checksum.json (registry checksum format); crates.io source-replaced"
CRATE_VERSION="2.0.0-dev"
CARGO_PKG_EXIT=-1
CARGO_VENDOR_EXIT=-1
NEGCTL_RESOLVED=false
CRATE_SHA256=""
BUILD_SECONDS=-1
PREBUILT_LINE_SEEN=false
CARGO_TREE_LINE=null
CARGO_TREE_EXIT=-1
QUICKSTART_LINE=null
RUN_SECONDS=-1
VENDOR_MODE=offline

write_receipt() {
  local standing="$1" reason="$2" exit_code="$3"
  cat >"$RECEIPT" <<EOF
{
  "court": "registry-consumer-cleanroom",
  "lane": "PB17",
  "standing": "$standing",
  "reason": "$reason",
  "exit_code": $exit_code,
  "repo": "$REPO",
  "head": "$REPO_HEAD",
  "arch": "$ARCH",
  "scratch": "$SCRATCH",
  "version_req_consumer": "=2.0.0-dev",
  "registry_mechanism": "$REGISTRY_MECHANISM",
  "vendor_mode": "$VENDOR_MODE",
  "cargo_vendor_exit": $CARGO_VENDOR_EXIT,
  "cargo_package_exit": $CARGO_PKG_EXIT,
  "crate_sha256": {$CRATE_SHA256
  },
  "negative_control_req_2_0_0_resolved": $NEGCTL_RESOLVED,
  "negative_control_verdict": "req 2.0.0 MUST NOT resolve against 2.0.0-dev packages (cargo semver prerelease rule); if it ever resolves, this court is vacuous",
  "build_seconds": $BUILD_SECONDS,
  "prebuilt_log_line": $PREBUILT_LINE_SEEN,
  "cargo_tree": $CARGO_TREE_LINE,
  "cargo_tree_exit": $CARGO_TREE_EXIT,
  "run_seconds": $RUN_SECONDS,
  "quickstart": "$QUICKSTART_LINE",
  "rpath_finding": "carried from ACT1 KD-CLEANROOM-RPATH: cargo forwards cargo:rustc-link-search but not cargo:rustc-link-arg from dep build scripts; DYLD_LIBRARY_PATH/LD_LIBRARY_PATH at the cache dir required at load time",
  "true_registry_delta": [
    "index/service: a real registry serves the sparse HTTP index + download endpoint + config.json; here resolution comes from a directory scan, so the index protocol is unproven",
    "checksum admission: cargo verifies downloaded .crate bytes against the index checksum; here checksums are generated locally from the same bytes being admitted — self-referential, so a corrupted .crate would NOT be caught",
    "publish-side laws: cargo publish ownership/duplicate-name/semver gates and crates.io limits never execute",
    "version selection: exactly one version per crate in the directory, so no multi-version resolution or YANK pressure is exercised",
    "the literal req \"2.0.0\": proven unresolvable against 2.0.0-dev by the negative control; only a real 2.0.0 non-dev release can admit it",
    "lazy download: vendor materializes ahead of time; registry cache layout (~/.cargo/registry) and on-demand fetch are unproven"
  ],
  "started": "$TS",
  "finished": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF
}

fail() { log "FAIL: $*"; write_receipt "FAIL" "$*" 1; exit 1; }
skip() { log "[SKIP] $*"; write_receipt "SKIP" "$*" 2; exit 2; }

# ---------------------------------------------------------------------------
# 0. Prerequisites
# ---------------------------------------------------------------------------
command -v cargo >/dev/null 2>&1 || skip "cargo not on PATH"
[[ -d "$REPO/crates/frozen-duckdb" ]] || fail "repo crates missing under $REPO"
[[ -d "$SCRATCH_REAL_HOME/.frozen-duckdb/cache/v2.0.0-dev-${ARCH}" ]] \
  || skip "prebuilt cache missing: $SCRATCH_REAL_HOME/.frozen-duckdb/cache/v2.0.0-dev-${ARCH}"

log "registry cleanroom arch=$ARCH repo=$REPO"

SCRATCH="$(mktemp -d /tmp/registry-consumer-XXXXXX)"
SCRATCH_HOME="$SCRATCH/home"
REGDIR="$SCRATCH/registry"
PKG_TARGET="$SCRATCH/pkg-target"
STAGING="$SCRATCH/vendor-staging"
VDIR="$SCRATCH/vendor-registry"
CONSUMER="$SCRATCH/consumer"
mkdir -p "$SCRATCH_HOME/.frozen-duckdb" "$REGDIR" "$STAGING/consumer/src" \
  "$VDIR" "$CONSUMER/src" "$SCRATCH/negctl/src"
# ---------------------------------------------------------------------------
# 0b. Snapshot the crate family into scratch — lane-safe on the shared
#     checkout (concurrent lanes may hold manifests mid-edit; the court is
#     immune) and zero-mutation of the repo tree.
# ---------------------------------------------------------------------------
FAMILY="$SCRATCH/family"
mkdir -p "$FAMILY/crates"
cp -R "$REPO/crates/frozen-duckdb" "$REPO/crates/frozen-duckdb-sys" \
  "$REPO/crates/frozen-duckdb-builder" "$FAMILY/crates/"
rm -rf "$FAMILY"/crates/*/target
cp "$REPO/Cargo.toml" "$FAMILY/Cargo.toml"
cp "$REPO/README.md" "$FAMILY/README.md" 2>/dev/null || true
# shrink the snapshot workspace to the three crates (root lists
# test-validation too, which the snapshot does not carry)
python3 - "$FAMILY/Cargo.toml" <<'PY'
import sys
p = sys.argv[1]
lines = open(p).read().split("\n")
out, in_members, members = [], False, []
for ln in lines:
    s = ln.strip()
    if s.startswith("members"):
        in_members, members = True, []
        out.append("members = [")
        continue
    if in_members:
        if s.startswith("]"):
            out.append('    "crates/frozen-duckdb",')
            out.append('    "crates/frozen-duckdb-sys",')
            out.append('    "crates/frozen-duckdb-builder",')
            out.append("]")
            in_members = False
        continue
    out.append(ln)
open(p, "w").write("\n".join(out))
PY
log "family snapshot: $FAMILY"

export HOME="$SCRATCH_HOME"
export RUSTUP_HOME="${RUSTUP_HOME:-$SCRATCH_REAL_HOME/.rustup}"
export CARGO_HOME="${CARGO_HOME:-$SCRATCH_REAL_HOME/.cargo}"
export CARGO_TARGET_DIR="$SCRATCH/target"

# ---------------------------------------------------------------------------
# 1. Vendor the family's FULL external dep surface (offline-first)
# ---------------------------------------------------------------------------
echo 'pub fn _placeholder() {}' >"$STAGING/consumer/src/lib.rs"

# Vendor from the SNAPSHOT WORKSPACE itself (members = the three crates), so
# members' dev-dependencies (pretty_assertions, ...) are included — a staging
# consumer would treat the family as plain path deps and drop their dev-deps,
# which cargo package then cannot resolve.
set +e
VENDOR_OUT="$(cd "$FAMILY" && cargo vendor --offline "$VDIR" 2>&1)"
CARGO_VENDOR_EXIT=$?
set -e
if [[ $CARGO_VENDOR_EXIT -ne 0 ]]; then
  log "offline vendor failed (exit=$CARGO_VENDOR_EXIT); retrying ONLINE — divergence recorded"
  VENDOR_MODE=online-fallback
  set +e
  VENDOR_OUT="$(cd "$FAMILY" && cargo vendor "$VDIR" 2>&1)"
  CARGO_VENDOR_EXIT=$?
  set -e
fi
printf '%s\n' "$VENDOR_OUT" >>"$LOG"
[[ $CARGO_VENDOR_EXIT -eq 0 ]] || fail "cargo vendor exit=$CARGO_VENDOR_EXIT (both offline and online)"
log "externals vendored ($VENDOR_MODE): $(ls "$VDIR" | wc -l | tr -d ' ') crates"

# ---------------------------------------------------------------------------
# 2. Package the three crates (dependency order) against the stitched
#    directory registry
# ---------------------------------------------------------------------------
mk_checksum_file() {
  python3 - "$1" <<'PY'
import hashlib, json, os, sys
d = sys.argv[1]
files = {}
for root, _, names in os.walk(d):
    for n in names:
        p = os.path.join(root, n)
        files[os.path.relpath(p, d)] = hashlib.sha256(open(p, "rb").read()).hexdigest()
json.dump({"files": files}, open(os.path.join(d, ".cargo-checksum.json"), "w"))
PY
}

package_crate() {
  local crate="$1"
  local crate_dir="$FAMILY/crates/$crate"
  mkdir -p "$crate_dir/.cargo"
  cat >"$crate_dir/.cargo/config.toml" <<EOF
[source.crates-io]
replace-with = "pb17-stitched-registry"

[source.pb17-stitched-registry]
directory = "$VDIR"
EOF
  set +e
  PKG_OUT="$(cd "$crate_dir" && cargo package --offline --no-verify --allow-dirty 2>&1)"
  CARGO_PKG_EXIT=$?
  set -e
  printf '%s\n' "$PKG_OUT" >>"$LOG"
  [[ $CARGO_PKG_EXIT -eq 0 ]] || fail "cargo package $crate exit=$CARGO_PKG_EXIT"

  local crate_path
  crate_path="$(ls -1 "$SCRATCH/target/package/${crate}-${CRATE_VERSION}.crate" 2>/dev/null | head -1 || true)"
  [[ -n "$crate_path" ]] || fail "no .crate produced for $crate (looked in $SCRATCH/target/package)"

  cp "$crate_path" "$REGDIR/"
  local sha
  sha="$(shasum -a 256 "$crate_path" | cut -d' ' -f1)"
  CRATE_SHA256="${CRATE_SHA256}${CRATE_SHA256:+,
    }\"$(basename "$crate_path")\": \"$sha\""
  log "packaged $(basename "$crate_path") sha256=$sha"

  # stitch the packaged artifact back into the directory registry (with the
  # checksum admission file) so the next crate's package run resolves it
  tar -xzf "$crate_path" -C "$VDIR"
  mk_checksum_file "$VDIR/${crate}-${CRATE_VERSION}"
}

package_crate frozen-duckdb-builder
package_crate frozen-duckdb-sys
package_crate frozen-duckdb

OVERLAY_CLEANED=false
cleanup_overlay() {
  $OVERLAY_CLEANED && return 0
  OVERLAY_CLEANED=true
  for crate in frozen-duckdb-builder frozen-duckdb-sys frozen-duckdb; do
    rm -f "$FAMILY/crates/$crate/.cargo/config.toml"
    rmdir "$FAMILY/crates/$crate/.cargo" 2>/dev/null || true
  done
}
trap cleanup_overlay EXIT
cleanup_overlay
log "packaging overlay configs removed from snapshot crates"

# ---------------------------------------------------------------------------
# 3. Consumer project: registry req + source replacement to the stitched
#    directory registry — resolution identical to the packaging leg
# ---------------------------------------------------------------------------
cat >"$CONSUMER/Cargo.toml" <<EOF
[package]
name = "registry-consumer"
version = "0.1.0"
edition = "2021"

[dependencies]
frozen-duckdb = "=2.0.0-dev"

[workspace]
EOF

mkdir -p "$CONSUMER/.cargo"
cat >"$CONSUMER/.cargo/config.toml" <<EOF
[source.crates-io]
replace-with = "pb17-stitched-registry"

[source.pb17-stitched-registry]
directory = "$VDIR"
EOF

cat >"$CONSUMER/src/main.rs" <<'EOF'
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
# 4. Negative control: req "2.0.0" MUST NOT resolve against 2.0.0-dev
# ---------------------------------------------------------------------------
cat >"$SCRATCH/negctl/Cargo.toml" <<EOF
[package]
name = "registry-negctl"
version = "0.1.0"
edition = "2021"

[dependencies]
frozen-duckdb = "2.0.0"

[workspace]
EOF
echo 'fn main() {}' >"$SCRATCH/negctl/src/main.rs"
mkdir -p "$SCRATCH/negctl/.cargo"
cp "$CONSUMER/.cargo/config.toml" "$SCRATCH/negctl/.cargo/config.toml"

set +e
NEGCTL_OUT="$(cd "$SCRATCH/negctl" && cargo tree 2>&1)"
NEGCTL_STATUS=$?
set -e
printf '%s\n' "$NEGCTL_OUT" >>"$LOG"
if [[ $NEGCTL_STATUS -eq 0 ]]; then
  NEGCTL_RESOLVED=true
  fail "negative control RESOLVED frozen-duckdb \"2.0.0\" against prerelease packages — resolution fence broken, court vacuous"
fi
log "negative control OK: req \"2.0.0\" refused (exit=$NEGCTL_STATUS) — prerelease fence live"

# ---------------------------------------------------------------------------
# 5. Build the consumer — zero DuckDB compile (cache pre-seeded)
# ---------------------------------------------------------------------------
CACHE_SRC="$SCRATCH_REAL_HOME/.frozen-duckdb/cache/v2.0.0-dev-${ARCH}"
CACHE_DST="$SCRATCH_HOME/.frozen-duckdb/cache/v2.0.0-dev-${ARCH}"
mkdir -p "$(dirname "$CACHE_DST")"
cp -c "$CACHE_SRC/." "$CACHE_DST/" 2>/dev/null || cp -R "$CACHE_SRC/." "$CACHE_DST/"
[[ -f "$CACHE_DST/ENGINE_VERSION" ]] || fail "cache pre-seed failed: ENGINE_VERSION missing at $CACHE_DST"
log "cache pre-seeded: $(cat "$CACHE_DST/ENGINE_VERSION")"

BUILD_START="$(date +%s)"
set +e
BUILD_OUT="$(cd "$CONSUMER" && cargo build 2>&1)"
BUILD_STATUS=$?
set -e
BUILD_SECONDS="$(($(date +%s) - BUILD_START))"
printf '%s\n' "$BUILD_OUT" >>"$LOG"

grep -q "Using prebuilt DuckDB binary" <<<"$BUILD_OUT" \
  || fail "build log missing 'Using prebuilt DuckDB binary' — compile fallback suspected"
PREBUILT_LINE_SEEN=true
[[ $BUILD_STATUS -eq 0 ]] || fail "cargo build exit=$BUILD_STATUS in ${BUILD_SECONDS}s"
log "cargo build exit=0 in ${BUILD_SECONDS}s (prebuilt log line present, zero compile)"

# ---------------------------------------------------------------------------
# 6. Prove the dep edge is registry-shaped
# ---------------------------------------------------------------------------
set +e
TREE_OUT="$(cd "$CONSUMER" && cargo tree 2>&1)"
CARGO_TREE_EXIT=$?
set -e
printf '%s\n' "$TREE_OUT" >>"$LOG"
[[ $CARGO_TREE_EXIT -eq 0 ]] || fail "cargo tree exit=$CARGO_TREE_EXIT"
CARGO_TREE_LINE="$(printf '%s' "$TREE_OUT" | head -1 | python3 -c 'import sys,json; print(json.dumps(sys.stdin.readline().rstrip("\n")))')"
grep -q 'frozen-duckdb v2.0.0-dev' <<<"$TREE_OUT" || fail "cargo tree lacks frozen-duckdb v2.0.0-dev"
grep -q 'frozen-duckdb-sys v2.0.0-dev' <<<"$TREE_OUT" || fail "cargo tree lacks frozen-duckdb-sys v2.0.0-dev"
if grep -q "$REPO/crates" <<<"$TREE_OUT"; then
  fail "cargo tree shows a path into the repo checkout — registry shape violated"
fi
log "cargo tree: registry-shaped edge confirmed (no repo path, v2.0.0-dev from stitched registry)"

# ---------------------------------------------------------------------------
# 7. Run the quickstart; assert engine version
# ---------------------------------------------------------------------------
BIN="$(find "$CARGO_TARGET_DIR/debug" -maxdepth 1 -name registry-consumer -type f | head -1)"
[[ -n "$BIN" ]] || fail "consumer binary not found under $CARGO_TARGET_DIR/debug"

RUN_START="$(date +%s)"
RUN_OUT="$(DYLD_LIBRARY_PATH="$CACHE_DST" LD_LIBRARY_PATH="$CACHE_DST" "$BIN" 2>&1)"
RUN_SECONDS="$(($(date +%s) - RUN_START))"
printf '%s\n' "$RUN_OUT" >>"$LOG"
QUICKSTART_LINE="$(grep '^DUCKDB_VERSION=' <<<"$RUN_OUT" || true)"
[[ -n "$QUICKSTART_LINE" ]] || fail "no DUCKDB_VERSION line in quickstart output"
grep -q '^DUCKDB_VERSION=v2\.0\.0-dev' <<<"$RUN_OUT" || fail "version mismatch: $QUICKSTART_LINE"
grep -q '^SUM=42$' <<<"$RUN_OUT" || fail "quickstart SUM=42 assertion failed"
log "quickstart OK: $QUICKSTART_LINE · SUM=42"

write_receipt "PASS" \
  "3 crates packaged against stitched directory registry -> consumer resolved =2.0.0-dev registry-shaped (no repo path in cargo tree), cache-hit build ${BUILD_SECONDS}s, quickstart $QUICKSTART_LINE (zero compile); negctl req \"2.0.0\" refused as required" 0
log "PASS receipt: $RECEIPT"
exit 0
