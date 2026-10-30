#!/usr/bin/env bash
# quickstart_demo.sh — one-shot onboarding probe for frozen-duckdb (LSS10).
#
# Scaffolds a scratch cargo project in /tmp, wires frozen-duckdb as a PATH
# dependency, pre-seeds the builder cache, runs a real query, and asserts on
# real output. Chicago discipline: real cargo project, real subprocess, real
# assertion — no mocks.
#
# Measures and reports:
#   phase 1 wall = dep-tree compile + cache acquisition (the known drag;
#                  LSS6 lever: precompiled-deps / artifact caching)
#   phase 2 wall = demo compile + run (warm)
#   rpath probe  = run the built binary with DYLD_LIBRARY_PATH explicitly
#                  unset, to empirically check the A2-flagged rpath gap.
#
# Receipt lines are prefixed "QUICKSTART-RECEIPT:" for machine parsing.
set -euo pipefail

REPO="${FROZEN_DUCKDB_REPO:-/Users/sac/frozen-duckdb}"
CRATE="$REPO/crates/frozen-duckdb"
SCRATCH_ROOT="${QUICKSTART_SCRATCH_ROOT:-/tmp}"
KEEP="${QUICKSTART_KEEP:-0}"

log()  { printf '%s\n' "$*"; }
die()  { printf 'FAIL: %s\n' "$*" >&2; exit 1; }
receipt() { printf 'QUICKSTART-RECEIPT: %s\n' "$*"; }

command -v cargo >/dev/null || die "[SKIP] cargo not on PATH — cannot run"
[ -f "$CRATE/Cargo.toml" ] || die "[SKIP] crate not found at $CRATE"

STAMP="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
HEAD="$(git -C "$REPO" rev-parse HEAD 2>/dev/null || echo unknown)"
ARCH="$(uname -m)"
CACHE_ENTRY="$HOME/.frozen-duckdb/cache/v2.0.0-${ARCH}"

log "== frozen-duckdb one-shot onboarding probe =="
receipt "repo=$REPO"
receipt "subject_head=$HEAD"
receipt "stamp=$STAMP"
receipt "arch=$ARCH"

# ---- Phase 0: cache state BEFORE -------------------------------------------
if [ -f "$CACHE_ENTRY/libduckdb.dylib" ] || [ -f "$CACHE_ENTRY/libduckdb.so" ]; then
    receipt "cache_before=HIT $CACHE_ENTRY"
    CACHE_BEFORE=HIT
else
    receipt "cache_before=MISS (build will acquire it)"
    CACHE_BEFORE=MISS
fi

# ---- Scaffold ---------------------------------------------------------------
DEMO_DIR="$(mktemp -d "$SCRATCH_ROOT/fd-quickstart.XXXXXX")"
trap '[ "$KEEP" = 1 ] || rm -rf "$DEMO_DIR"' EXIT
log "-- scaffolding $DEMO_DIR"

cargo new --bin --quiet "$DEMO_DIR/quickstart-demo"

cat > "$DEMO_DIR/quickstart-demo/Cargo.toml" <<EOF
[package]
name = "quickstart-demo"
version = "0.1.0"
edition = "2021"

[dependencies]
frozen-duckdb = { path = "$CRATE" }
EOF

cat > "$DEMO_DIR/quickstart-demo/src/main.rs" <<'EOF'
use frozen_duckdb::duckdb;

fn main() {
    let conn = duckdb::Connection::open_in_memory().expect("connect");
    let v: i64 = conn
        .query_row("SELECT 42", [], |r| r.get(0))
        .expect("query_row");
    assert_eq!(v, 42, "DuckDB returned wrong value");
    println!("QUICKSTART-OK: SELECT 42 -> {}", v);
}
EOF

# ---- Phase 1: cold-ish build (dep tree + cache acquisition) -----------------
log "-- phase 1: cargo build (dep tree + cache acquisition)"
# Re-running the probe on the same machine re-pays the dep-tree compile unless
# a target dir is reused; QUICKSTART_TARGET_DIR opts into a warm target.
T0=$(date +%s.%N)
if [ -n "${QUICKSTART_TARGET_DIR:-}" ]; then
    cargo build --manifest-path "$DEMO_DIR/quickstart-demo/Cargo.toml" \
        --target-dir "$QUICKSTART_TARGET_DIR" >"$DEMO_DIR/phase1.log" 2>&1 \
        || { tail -40 "$DEMO_DIR/phase1.log" >&2; die "phase 1 build failed"; }
    BIN="$QUICKSTART_TARGET_DIR/debug/quickstart-demo"
else
    cargo build --manifest-path "$DEMO_DIR/quickstart-demo/Cargo.toml" >"$DEMO_DIR/phase1.log" 2>&1 \
        || { tail -40 "$DEMO_DIR/phase1.log" >&2; die "phase 1 build failed"; }
fi
T1=$(date +%s.%N)
P1=$(echo "$T1 - $T0" | bc)
receipt "phase1_dep_tree_compile_wall_s=$P1"

# ---- Phase 2: warm run -------------------------------------------------------
log "-- phase 2: warm cargo run + assert"
BIN="${BIN:-$DEMO_DIR/quickstart-demo/target/debug/quickstart-demo}"

# Leg A (non-vacuity): with DYLD_LIBRARY_PATH unset, the load MUST fail —
# this is the A2-flagged rpath gap, reproduced here as a real observation.
T2=$(date +%s.%N)
if OUT_A="$(env -u DYLD_LIBRARY_PATH "$BIN" 2>"$DEMO_DIR/legA.err")"; then
    receipt "rpath_gap_probe=UNEXPECTED-PASS (binary loaded without rpath; gap appears closed)"
else
    grep -q "Library not loaded: @rpath/libduckdb.dylib" "$DEMO_DIR/legA.err" \
        || { cat "$DEMO_DIR/legA.err" >&2; die "leg A failed for an unmodeled reason"; }
    receipt "rpath_gap_probe=CONFIRMED (dyld: Library not loaded @rpath/libduckdb.dylib, no LC_RPATH)"
fi
T3=$(date +%s.%N)
receipt "phase2_legA_bare_run_wall_s=$(echo "$T3 - $T2" | bc)"

# Leg B (workaround + assert): the current one-shot reality requires pointing
# dyld at the cache. Receipted as a divergence, fixed by the rpath proposal.
LIB_DIR="$HOME/.frozen-duckdb/cache/v2.0.0-${ARCH}"
T4=$(date +%s.%N)
OUT="$(DYLD_LIBRARY_PATH="$LIB_DIR" "$BIN")" || die "demo binary failed even with DYLD_LIBRARY_PATH"
T5=$(date +%s.%N)
receipt "phase2_legB_workaround_run_wall_s=$(echo "$T5 - $T4" | bc)"

case "$OUT" in
    "QUICKSTART-OK: SELECT 42 -> 42") log "$OUT" ;;
    *) die "unexpected demo output: $OUT" ;;
esac
receipt "assert=PASS ($OUT)"
receipt "dyld_workaround=DYLD_LIBRARY_PATH=$LIB_DIR"

# ---- Cache state AFTER -------------------------------------------------------
if [ -f "$CACHE_ENTRY/libduckdb.dylib" ] || [ -f "$CACHE_ENTRY/libduckdb.so" ]; then
    receipt "cache_after=HIT $CACHE_ENTRY"
else
    receipt "cache_after=MISS"
fi

if [ "$KEEP" = 1 ]; then
    receipt "demo_dir=$DEMO_DIR (kept)"
else
    receipt "demo_dir=$DEMO_DIR (removed at exit)"
fi

log ""
log "Known drag: phase 1 (dep-tree compile) dominates first-run wall time."
log "Lever: LSS6 precompiled-deps / artifact caching (see precompile_deps.sh)."
log "PASS: one-shot onboarding probe complete."
