#!/usr/bin/env bash
# extension_watch.sh — KD-02 closure watcher (ADV12).
#
# One-shot poll of the DuckDB extension CDN at OUR dev pin (d54cbbe3d).
# Cron-able: no daemon, no state, one invocation = one probe.
#
# 200 on HEAD -> the extension now ships for the dev pin -> print
# "AVAILABLE" + exact un-ignore commands. Else exit 1 typed "still absent".
#
# Canonical divergence: KNOWN_DIVERGENCES.md row KD-02.

set -u

PIN="d54cbbe3d"
PLATFORM="osx_arm64"
BASE="http://extensions.duckdb.org/${PIN}/${PLATFORM}"

# extensions probed (KD-02 closure set)
EXTENSIONS=(flock tpch)

TEST_FILE="crates/frozen-duckdb/tests/flock_tests.rs"

rc_all=0
for ext in "${EXTENSIONS[@]}"; do
  url="${BASE}/${ext}.duckdb_extension.gz"
  code=$(curl -s -o /dev/null -w '%{http_code}' -I --max-time 15 "$url")
  ts=$(date -u +%Y-%m-%dT%H:%M:%SZ)
  if [ "$code" = "200" ]; then
    echo "[$ts] $ext ${url} -> 200 AVAILABLE"
    echo "  ACTION: un-ignore the gated tests and rerun:"
    echo "    sed -i '' 's/^#\[ignore = \"${ext} community extension has no build/#\[ignore:x = \"${ext} community extension has no build/' ${TEST_FILE}"
    echo "    cargo test -p frozen-duckdb --test flock_tests"
    rc_all=1   # AVAILABLE is an actionable signal, surface it loudly
  else
    echo "[$ts] $ext ${url} -> HTTP ${code} still absent"
    rc_all=1
  fi
done

if [ "$rc_all" -ne 0 ]; then
  echo "STILL_ABSENT: no dev-pin build on the extension CDN for: ${EXTENSIONS[*]} (pin ${PIN}, ${PLATFORM})"
fi
exit "$rc_all"
