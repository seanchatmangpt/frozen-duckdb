#!/usr/bin/env bash
# fastfollow_flip.sh — mechanical version flip for the frozen-duckdb fast-follow.
#
# Usage: fastfollow_flip.sh OLD_VER NEW_VER NEW_SHA [TARGET_DIR]
#
# Performs the declarative flip edit set proven in the 2026-10-21 rehearsal
# (/tmp/ff-rehearsal/RECEIPT.md):
#   1. workspace + per-crate version pins (Cargo.toml, dep-pins, Cargo.lock)
#   2. builder const VERSION + SOURCE_PIN -> NEW_SHA placeholder
#   3. GitHub workflows (BUILDER_VERSION + DUCKDB_VERSION x5)
#   4. ggen.toml [project].version
#   5. schema/domain.ttl (dcterms:version, fd:duckdbVersionLaw notation, RC
#      individual fd:release-v<NEW>) and schema/changelog.ttl Release block
#   6. scripts/* DUCKDB_VERSION pins + doctor attest_pin.sh GENESIS pin
#   7. docs/GENESIS.md header rows (@v<OLD> upstream refs)
#   8. KD register note (KNOWN_DIVERGENCES.md) annotating the flip
#   9. windows-none sweep (no-op today — no windows-none pins exist; kept so
#      the flip set stays complete if a Windows target pin lands later)
#
# Generated projections (Makefile, scripts/verify-gates.sh,
# scripts/verify-evidence.ttl, gates/dry-run-publish*) are NOT touched:
# they re-render via `ggen sync run` (edit sources, never projections).
# Documentation prose (README/CHANGELOG.md/docs/**, docs/sjira/v<OLD>/) is
# NOT touched: version mentions there are historical records, classified in
# the residue table of the rehearsal receipt.
#
# Fail-closed: exits nonzero if the target dir is missing, a curated file is
# absent, or any sed fails. Prints one FLIP line per applied edit.

set -euo pipefail

OLD_VER="${1:?usage: fastfollow_flip.sh OLD_VER NEW_VER NEW_SHA [TARGET_DIR]}"
NEW_VER="${2:?usage: fastfollow_flip.sh OLD_VER NEW_VER NEW_SHA [TARGET_DIR]}"
NEW_SHA="${3:?usage: fastfollow_flip.sh OLD_VER NEW_VER NEW_SHA [TARGET_DIR]}"
ROOT="${4:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"

[ -d "$ROOT" ] || { echo "FATAL: target dir $ROOT not found" >&2; exit 2; }
[ "$OLD_VER" != "$NEW_VER" ] || { echo "FATAL: OLD_VER == NEW_VER" >&2; exit 2; }
[[ "$NEW_SHA" =~ ^[0-9a-f]{40}$ ]] || {
  echo "FATAL: NEW_SHA must be a 40-hex upstream commit (got: $NEW_SHA)" >&2; exit 2; }

OLD_PIN="d54cbbe3df17e8e5ce68312f483b4155836112aa"   # duckdb v2.0.0 LTS tag d54cbbe
old_if() { [ -f "$ROOT/$1" ] || { echo "FATAL: missing $ROOT/$1" >&2; exit 2; }; }

# ---------------------------------------------------------------- edit set ---
# 1. Workspace + per-crate version pins + locks
for f in Cargo.toml \
         crates/frozen-duckdb/Cargo.toml \
         crates/frozen-duckdb-sys/Cargo.toml \
         crates/frozen-duckdb-builder/Cargo.toml \
         Cargo.lock test-dependency/Cargo.lock; do
  old_if "$f"
  if grep -qF "$OLD_VER" "$ROOT/$f"; then
    sed -i '' "s/$OLD_VER/$NEW_VER/g" "$ROOT/$f"
    echo "FLIP workspace-pin  $f  $OLD_VER -> $NEW_VER"
  fi
done

# 2. Builder: const VERSION + SOURCE_PIN, main-crate doc install example
BUILDER_LIB="crates/frozen-duckdb-builder/src/lib.rs"
old_if "$BUILDER_LIB"
sed -i '' "s/$OLD_VER/$NEW_VER/g" "$ROOT/$BUILDER_LIB"
sed -i '' "s/$OLD_PIN/$NEW_SHA/g" "$ROOT/$BUILDER_LIB"
echo "FLIP builder-lib    $BUILDER_LIB  VERSION + SOURCE_PIN -> $NEW_SHA (placeholder: replace with the real v$NEW_VER tag SHA when upstream tags)"

MAIN_LIB="crates/frozen-duckdb/src/lib.rs"
old_if "$MAIN_LIB"
sed -i '' "s/$OLD_VER/$NEW_VER/g" "$ROOT/$MAIN_LIB"
echo "FLIP main-crate-doc $MAIN_LIB  install example -> $NEW_VER"

# 3. Workflows: BUILDER_VERSION + DUCKDB_VERSION (ci.yml x3, ci-simple x1,
#    test-minimal x1, build-binaries default/description)
for f in .github/workflows/ci.yml \
         .github/workflows/ci-simple.yml \
         .github/workflows/test-minimal.yml \
         .github/workflows/build-binaries.yml; do
  old_if "$f"
  n=$(grep -cF "$OLD_VER" "$ROOT/$f" || true)
  if [ "$n" -gt 0 ]; then
    sed -i '' "s/$OLD_VER/$NEW_VER/g" "$ROOT/$f"
    echo "FLIP workflow       $f  ($n occurrence(s))"
  fi
done

# 4. ggen.toml [project].version
old_if ggen.toml
sed -i '' "s/$OLD_VER/$NEW_VER/g" "$ROOT/ggen.toml"
echo "FLIP ggen.toml      [project].version -> $NEW_VER"

# 5. Schema sources: domain.ttl (dcterms:version, duckdbVersionLaw notation,
#    RC individual fd:release-v<OLD> -> fd:release-v<NEW>) + changelog.ttl
for f in schema/domain.ttl schema/changelog.ttl; do
  old_if "$f"
  n=$(grep -cF "$OLD_VER" "$ROOT/$f" || true)
  sed -i '' "s/$OLD_VER/$NEW_VER/g" "$ROOT/$f"
  echo "FLIP schema         $f  ($n occurrence(s), includes RC individual + Release block)"
done

# 6. Scripts DUCKDB_VERSION + doctor attest_pin GENESIS pin
for f in scripts/build_duckdb_optimized.sh \
         scripts/build_full_featured_duckdb.sh \
         scripts/create_frozen_setup.sh \
         scripts/create_precompiled_duckdb.sh \
         scripts/build_frozen_duckdb.sh \
         scripts/download_duckdb_binaries.sh \
         scripts/setup_optimized_build.sh \
         warehouse-sim/economics/run_scorecard.sh \
         warehouse-sim/economics/run_court.sh \
         warehouse-sim/economics/sovereign_probe/Cargo.lock \
         prebuilt/setup_env.sh \
         scripts/doctor/checks/attest_pin.sh; do
  [ -f "$ROOT/$f" ] || continue   # scripts are best-effort: some are legacy
  n=$(grep -cF "$OLD_VER" "$ROOT/$f" || true)
  if [ "$n" -gt 0 ]; then
    sed -i '' "s/$OLD_VER/$NEW_VER/g" "$ROOT/$f"
    echo "FLIP script         $f  ($n occurrence(s))"
  fi
done

# 7. GENESIS header rows (@v<OLD> upstream refs + pinned SHA in prose)
if [ -f "$ROOT/docs/GENESIS.md" ]; then
  n=$(grep -cF "$OLD_VER" "$ROOT/docs/GENESIS.md" || true)
  if [ "$n" -gt 0 ]; then
    sed -i '' -e "s/$OLD_VER/$NEW_VER/g" -e "s/$OLD_PIN/$NEW_SHA/g" "$ROOT/docs/GENESIS.md"
    echo "FLIP genesis-rows   docs/GENESIS.md  ($n occurrence(s))"
  fi
fi

# 8. KD register note: append a flip annotation (idempotent per version pair)
KD="$ROOT/KNOWN_DIVERGENCES.md"
if [ -f "$KD" ]; then
  sed -i '' "s/$OLD_VER/$NEW_VER/g" "$KD"
  if ! grep -qF "fastfollow-flip $OLD_VER->$NEW_VER" "$KD"; then
    printf '\n<!-- fastfollow-flip %s->%s: declarative flip applied by scripts/fastfollow_flip.sh; SOURCE_PIN is a PLACEHOLDER (%s) until upstream tags v%s. -->\n' \
      "$OLD_VER" "$NEW_VER" "$NEW_SHA" "$NEW_VER" >> "$KD"
    echo "FLIP kd-register    KNOWN_DIVERGENCES.md  flip note appended"
  fi
fi

# 9. windows-none sweep (no-op today; the pin does not exist in this tree)
if grep -rIl --exclude-dir='target*' --exclude-dir=.git --exclude=fastfollow_flip.sh 'windows-none' "$ROOT" 2>/dev/null | head -1 | grep -q .; then
  echo "FLIP windows-none   pins found — update manually (not scripted)"
else
  echo "FLIP windows-none   no windows-none pins in tree (no-op, as rehearsed)"
fi

echo "FLIP done  $OLD_VER -> $NEW_VER (SOURCE_PIN placeholder $NEW_SHA in $ROOT)"
