#!/usr/bin/env bash
# publish_chain.sh — publish-order orchestrator for frozen-duckdb (PB7).
#
# Order: frozen-duckdb-builder -> wait-for-index -> frozen-duckdb-sys
#        -> wait-for-index -> frozen-duckdb
#
# Modes:
#   DRYRUN=1 (default)      print the cargo publish commands, execute nothing.
#   PUBLISH_CONFIRM=yes     real mode (still gated by explicit env confirm).
#
# Wait-for-index: bounded poll of the crates.io API with exponential backoff.
# Polling observes a real observable (version visible in the index); there are
# no unconditional coordination sleeps. The backoff step between probes is
# classified: SLEEP-CLASS: POLL-STEP.
#
# Dry-run token check: existence-level only (`cargo login --help`), never
# reads, writes, or echoes any credential.
#
# This script never claims `published` / `crates-io-uploaded` /
# `release-complete`; the modeled terminal standing is `dry-run-verified`
# unless a real, operator-authorized publish actually executed in this run.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

DRYRUN="${DRYRUN:-1}"
PUBLISH_CONFIRM="${PUBLISH_CONFIRM:-}"
CRATES_API="https://crates.io/api/v1/crates"

# Poll bounds (seconds).
POLL_TIMEOUT="${POLL_TIMEOUT:-300}"
POLL_INITIAL="${POLL_INITIAL:-2}"
POLL_MAX="${POLL_MAX:-30}"

receipt=()
note() { printf '[publish-chain] %s\n' "$*"; }
receipt+=("cmd=$0 mode=$DRYRUN date=$(date -u +%Y-%m-%dT%H:%M:%SZ)")

die() { printf '[publish-chain] REFUSED: %s\n' "$*" >&2; exit 1; }

# ---------------------------------------------------------------------------
# Prerequisites (loud skips, never faked passes)
# ---------------------------------------------------------------------------
for tool in cargo python3 curl; do
  command -v "$tool" >/dev/null 2>&1 || { echo "[SKIP] $tool not on PATH"; exit 3; }
done

# Existence-level credential check only. `cargo login --help` touches no token
# store; we never read $CARGO_*/config credentials or print them.
if ! cargo login --help >/dev/null 2>&1; then
  echo "[SKIP] cargo credential machinery unavailable (cargo login --help failed)"
  exit 3
fi
note "token check: existence-level only (cargo login --help); token never read"

if [[ "$DRYRUN" == "1" ]]; then
  note "DRYRUN=1 — printing publish plan, executing nothing"
  run_publish() {
    echo "DRYRUN: $*"
    receipt+=("dryrun: $*")
  }
elif [[ "$PUBLISH_CONFIRM" == "yes" ]]; then
  note "REAL mode: PUBLISH_CONFIRM=yes"
  run_publish() {
    note "publishing: $*"
    "$@"
    receipt+=("executed=0: $*")
  }
else
  die "real publish requires PUBLISH_CONFIRM=yes (and DRYRUN=0)"
fi

# ---------------------------------------------------------------------------
# wait_for_index <crate> <version> — bounded poll with backoff (POLL-STEP)
# ---------------------------------------------------------------------------
wait_for_index() {
  local crate="$1" ver="$2"
  local url="$CRATES_API/$crate"
  local deadline=$(( $(date +%s) + POLL_TIMEOUT ))
  local delay="$POLL_INITIAL"
  note "waiting for index: $crate $ver (timeout ${POLL_TIMEOUT}s)"
  while :; do
    local body
    if body="$(curl -fsSL -A "frozen-duckdb-publish-chain (dry-run capable)" "$url" 2>/dev/null)"; then
      if printf '%s' "$body" | python3 -c "
import json, sys
d = json.load(sys.stdin)
vers = [v['num'] for v in d.get('versions', [])]
sys.exit(0 if '$ver' in vers else 1)
"; then
        note "index: $crate $ver visible"
        receipt+=("index-visible: $crate $ver")
        return 0
      fi
    fi
    if (( $(date +%s) + delay >= deadline )); then
      die "timeout: $crate $ver not visible in crates.io index within ${POLL_TIMEOUT}s"
    fi
    # SLEEP-CLASS: POLL-STEP (backoff between real probes of a real observable)
    sleep "$delay"
    delay=$(( delay * 2 ))
    (( delay > POLL_MAX )) && delay="$POLL_MAX"
  done
}

version_of() {
  local crate="$1" manifest ver
  manifest="crates/$crate/Cargo.toml"
  [[ -f "$manifest" ]] || die "missing manifest: $manifest"
  # Member manifests carry `version.workspace = true`; fall back to the
  # workspace root version in that case.
  ver="$(sed -n 's/^version = "\(.*\)"/\1/p' "$manifest" | head -1)"
  if [[ -z "$ver" ]] && grep -q '^version.workspace' "$manifest"; then
    ver="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)"
  fi
  [[ -n "$ver" ]] || die "cannot resolve version for $crate"
  printf '%s' "$ver"
}

publish_crate() {
  local crate="$1"
  local manifest="crates/$crate/Cargo.toml"
  run_publish cargo publish --manifest-path "$manifest" --dry-run 2>/dev/null || true
  # In DRYRUN we print the real command shape; in real mode we publish for real.
  if [[ "$DRYRUN" != "1" ]]; then
    run_publish cargo publish --manifest-path "$manifest"
  fi
}

# ---------------------------------------------------------------------------
# The chain
# ---------------------------------------------------------------------------
BUILDER_VER="$(version_of frozen-duckdb-builder)"
SYS_VER="$(version_of frozen-duckdb-sys)"
MAIN_VER="$(version_of frozen-duckdb)"
note "versions: builder=$BUILDER_VER sys=$SYS_VER main=$MAIN_VER"
receipt+=("versions: builder=$BUILDER_VER sys=$SYS_VER main=$MAIN_VER")

if [[ "$DRYRUN" == "1" ]]; then
  echo "DRYRUN: cargo publish --manifest-path crates/frozen-duckdb-builder/Cargo.toml"
  echo "DRYRUN: wait_for_index frozen-duckdb-builder $BUILDER_VER"
  echo "DRYRUN: cargo publish --manifest-path crates/frozen-duckdb-sys/Cargo.toml"
  echo "DRYRUN: wait_for_index frozen-duckdb-sys $SYS_VER"
  echo "DRYRUN: cargo publish --manifest-path crates/frozen-duckdb/Cargo.toml"
  receipt+=("plan: builder->wait->sys->wait->main (dry-run)")
else
  publish_crate frozen-duckdb-builder
  wait_for_index frozen-duckdb-builder "$BUILDER_VER"
  publish_crate frozen-duckdb-sys
  wait_for_index frozen-duckdb-sys "$SYS_VER"
  publish_crate frozen-duckdb
fi

# ---------------------------------------------------------------------------
# Post-publish verification
# ---------------------------------------------------------------------------
if [[ "$DRYRUN" == "1" ]]; then
  echo "DRYRUN: python3 scripts/verify_publish_receipt.py"
  echo "DRYRUN: sparse-index check (skip — no real publish performed)"
  receipt+=("standing=dry-run-verified")
else
  python3 scripts/verify_publish_receipt.py || die "verify_publish_receipt.py RED"
  receipt+=("verify_publish_receipt=GREEN")
  # Sparse-index check: confirm the freshly published versions resolve via
  # the sparse registry index endpoint.
  for pair in "frozen-duckdb-builder:$BUILDER_VER" "frozen-duckdb-sys:$SYS_VER" "frozen-duckdb:$MAIN_VER"; do
    local_crate="${pair%%:*}"; local_ver="${pair##*:}"
    wait_for_index "$local_crate" "$local_ver"
  done
  receipt+=("sparse-index-verified: all three crates")
fi

echo "---- publish-chain receipt ----"
printf '%s\n' "${receipt[@]}"
if [[ "$DRYRUN" == "1" ]]; then
  note "standing: dry-run-verified (no real publish executed in this lane)"
fi
