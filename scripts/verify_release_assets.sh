#!/usr/bin/env bash
# verify_release_assets.sh — post-dispatch release-asset verification harness (lane GH5).
#
# Chicago-school: real downloads, real Mach-O inspection, real ENGINE probe.
# SHIM-COURT compatible: any assertion breach exits nonzero; empty release
# state exits 2 loudly ("no release <tag> exists yet") — never a faked pass.
#
# Usage: verify_release_assets.sh <TAG>
# Exit: 0 all-pass | 1 any breach | 2 loud no-release / usage
set -uo pipefail

TAG="${1:-}"
if [[ -z "$TAG" ]]; then
  echo "usage: $0 <TAG>   (e.g. v2.0.0)" >&2
  exit 2
fi

EXPECTED_ASSETS=(libduckdb_arm64.dylib libduckdb_x86_64.dylib)
OPTIONAL_ASSETS=(libduckdb_linux_arm64.so)   # REL6 Linux arm64 — accept-and-verify, never required
BANNED_ATOM='libfrozen_mega_'                # mega-library leftovers never ship as release assets
MAX_BYTES=$((55 * 1024 * 1024))   # strip -x size ceiling: 55 MiB
WORK="$(mktemp -d "${TMPDIR:-/tmp}/verify_release_assets.XXXXXX")"
trap 'rm -rf "$WORK"' EXIT

fail=0
breaches=()
note() { printf '[verify-release-assets] %s\n' "$*"; }
breach() { fail=1; breaches+=("$1"); printf '[verify-release-assets] BREACH: %s\n' "$1"; }

# ---------------------------------------------------------------------------
# 0. Loud prerequisite: the release must exist. Missing release = exit 2.
# ---------------------------------------------------------------------------
if ! gh release view "$TAG" --json tagName >/dev/null 2>&1; then
  note "no release $TAG exists yet — loud SKIP-class exit (2), not a faked pass"
  exit 2
fi

note "release $TAG found; asserting exact asset set"

# ---------------------------------------------------------------------------
# 1. Exactly the two expected assets, exact names, no extras.
# ---------------------------------------------------------------------------
mapfile -t actual < <(gh release view "$TAG" --json assets -q '.assets[].name' | sort)
expected_sorted=($(printf '%s\n' "${EXPECTED_ASSETS[@]}" | sort))

if [[ ${#actual[@]} -ne 2 ]]; then
  breach "asset count ${#actual[@]} != 2 (found: ${actual[*]:-none})"
fi
for name in "${expected_sorted[@]}"; do
  if [[ ! " ${actual[*]:-} " == *" $name "* ]]; then
    breach "missing expected asset: $name"
  fi
done
for name in "${actual[@]:-}"; do
  [[ -z "$name" ]] && continue
  if [[ ! " ${EXPECTED_ASSETS[*]} " == *" $name "* ]]; then
    breach "unexpected extra asset: $name"
  fi
done

# ---------------------------------------------------------------------------
# 1b. No libfrozen_mega_* leftovers on the release asset list (REHEARSAL law).
# ---------------------------------------------------------------------------
for name in "${actual[@]:-}"; do
  [[ -z "$name" ]] && continue
  case "$name" in
    libfrozen_mega_*)
      breach "asset '$name' is a $BANNED_ATOM* mega-library leftover — must never ship"
      ;;
  esac
done
note "no ${BANNED_ATOM}* leftovers on asset list (checked ${#actual[@]} names)"

# ---------------------------------------------------------------------------
# 1c. Optional Linux arm64 asset (REL6): accept-and-verify if present.
#     Its presence is NOT required; if shipped it must download and be nonempty.
# ---------------------------------------------------------------------------
for name in "${OPTIONAL_ASSETS[@]}"; do
  if [[ " ${actual[*]:-} " == *" $name "* ]]; then
    dl="$WORK/$name"
    if gh release download "$TAG" --pattern "$name" --dir "$WORK" --clobber >/dev/null 2>&1 \
       && [[ -s "$dl" ]]; then
      bytes="$(stat -f%z "$dl")"
      note "optional $name present: verified download, $bytes bytes"
      printf 'optional  %s  bytes=%s\n' "$name" "$bytes" >>"$WORK/optional-assets.txt" || true
    else
      breach "optional asset '$name' listed but download failed or empty file"
    fi
  else
    note "optional $name absent from release (REL6) — accepted, not required"
  fi
done

# ---------------------------------------------------------------------------
# 1d. Cache-population check: post-download cache for this tag must carry the
#     duckdb/ headers + libduckdb.dylib. Absent cache = loud informational note
#     (populated on this machine by first consumer build, not by this harness);
#     present-but-incomplete cache = breach.
# ---------------------------------------------------------------------------
cache_root="${HOME}/.frozen-duckdb/cache"
cache_hits=0
for name in "${EXPECTED_ASSETS[@]}"; do
  arch="${name#libduckdb_}"; arch="${arch%.dylib}"
  cdir="$cache_root/${TAG#v}-${arch}"
  if [[ -d "$cdir" ]]; then
    cache_hits=$((cache_hits + 1))
    for req in duckdb/duckdb.h duckdb/duckdb.hpp libduckdb.dylib; do
      [[ -e "$cdir/$req" ]] || breach "cache $cdir missing required artifact: $req"
    done
    note "cache $cdir populated (duckdb/ headers + libduckdb.dylib verified)"
  fi
done
if (( cache_hits == 0 )); then
  note "cache $cache_root/${TAG#v}-<arch> not populated on this machine — informational (first consumer build populates it)"
fi

# ---------------------------------------------------------------------------
# 1e. Clean-machine smoke step reference: print the exact checklist commands a
#     clean machine must run (no DYLD_LIBRARY_PATH — the dylib embeds its rpath).
# ---------------------------------------------------------------------------
note "clean-machine smoke checklist (REL11):"
cat <<'SMOKE'
  cd "$(mktemp -d)" && cargo new smoke && cd smoke
  cargo add frozen-duckdb --path <repo>/crates/frozen-duckdb   # or crates.io pin post-publish
  cat > src/main.rs <<'EOF'
fn main() { let db = frozen_duckdb::Connection::open_in_memory().unwrap();
            println!("smoke: {}", frozen_duckdb::version()); }
EOF
  env -u DYLD_LIBRARY_PATH cargo run   # must link + run with NO DYLD_LIBRARY_PATH
SMOKE
note "smoke checklist printed — operator runs it on a clean machine; harness asserts the asset prerequisites"

# ---------------------------------------------------------------------------
# 2. Per-asset: real download -> Mach-O arch -> strip -x size -> ENGINE -> sha256
# ---------------------------------------------------------------------------
sha_report="$WORK/sha256.txt"
: >"$sha_report"

probe_engine() {
  # $1 = dylib path, $2 = expected version (e.g. 2.0.0), $3 = arch
  local lib="$1" want="$2" arch="$3" got=""
  if [[ "$arch" == "$(uname -m)" ]]; then
    # Native arch: real load via python ctypes duckdb_library_version.
    got="$(python3 - "$lib" "$want" <<'PY'
import ctypes, sys
lib = ctypes.CDLL(sys.argv[1])
lib.duckdb_library_version.restype = ctypes.c_char_p
sys.stdout.write(lib.duckdb_library_version().decode())
PY
)"
    echo "ctypes-duckdb_library_version:${got}"
  else
    # Foreign arch on this host: strings-only inspection (recorded method).
    got="$(strings "$lib" | grep -m1 -oE 'v[0-9]+[0-9A-Za-z.+_-]*dev[0-9A-Za-z.+_-]*' || true)"
    echo "strings-scan:${got:-<no-dev-version-string-found>}"
  fi
}

for name in "${EXPECTED_ASSETS[@]}"; do
  dl="$WORK/$name"
  if ! gh release download "$TAG" --pattern "$name" --dir "$WORK" --clobber >/dev/null 2>&1; then
    breach "$name: download failed"
    continue
  fi
  [[ -f "$dl" ]] || { breach "$name: file absent after download"; continue; }

  want_arch="${name#libduckdb_}"; want_arch="${want_arch%.dylib}"
  got_archs="$(lipo -archs "$dl" 2>&1)" || { breach "$name: lipo failed ($got_archs)"; continue; }
  case " $got_archs " in
    *" $want_arch "*) : ;;
    *) breach "$name: lipo -archs='$got_archs' lacks required slice '$want_arch'" ;;
  esac

  stripped="$WORK/stripped-$name"
  cp "$dl" "$stripped"
  if ! strip -x "$stripped" 2>/dev/null; then
    breach "$name: strip -x failed"
    continue
  fi
  size="$(stat -f%z "$stripped")"
  if (( size > MAX_BYTES )); then
    breach "$name: stripped size $size bytes exceeds ${MAX_BYTES}-byte ceiling"
  fi

  expected_ver="${TAG#v}"
  probe_out="$(probe_engine "$dl" "$expected_ver" "$want_arch")"
  method="${probe_out%%:*}"
  got_ver="${probe_out#*:}"
  if [[ "$got_ver" != *"$expected_ver"* ]]; then
    breach "$name: ENGINE probe ($method) got '$got_ver', expected contains '$expected_ver'"
  fi

  sha="$(/usr/bin/shasum -a 256 "$dl" | awk '{print $1}')"
  printf '%s  %s  arch=%s method=%s version=%s stripped_bytes=%s\n' \
    "$sha" "$name" "$want_arch" "$method" "$got_ver" "$size" | tee -a "$sha_report"
done

if (( fail )); then
  note "FAIL — ${#breaches[@]} breach(es):"
  for b in "${breaches[@]}"; do note "  - $b"; done
  exit 1
fi

note "PASS — 2/2 assets verified (Mach-O slice, strip -x <=55MiB, ENGINE probe, sha256)"
cat "$sha_report"
exit 0
