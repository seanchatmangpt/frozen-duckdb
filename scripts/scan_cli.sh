#!/usr/bin/env bash
set -euo pipefail

# KCura CLI surface scan: dumps help for all commands/subcommands to docs/CLI_HELP.md
# Fails if any help path is broken or missing

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")"/.. && pwd)"
OUT_FILE="$ROOT_DIR/docs/CLI_HELP.md"
PKG="kcura-cli"

echo "Scanning KCura CLI surface…"

_run_help() {
  # Runs cargo help for given args; prints to stdout
  cargo run -q --package "$PKG" -- "$@" --help
}

_list_subs() {
  # Lists subcommands for given args; prints one per line
  # Handles clap help sections labeled either SUBCOMMANDS or Commands:
  _run_help "$@" 2>/dev/null | awk '
    BEGIN{flag=0}
    /^SUBCOMMANDS/ {flag=1; next}
    /^Commands:/ {flag=1; next}
    flag==1 {
      if ($0 ~ /^Options:|^OPTIONS|^ARGS|^Arguments:|^USAGE|^Usage:/) {flag=0; next}
      if ($0 ~ /^[[:space:]]*$/) next
      if (match($0, /^[[:space:]]*([[:alnum:]_\-:]+)\b/, m)) {
        cmd=m[1]; if (cmd!="help") print cmd;
      }
    }
  ' | sort -u
}

_append_section() {
  local header="$1"; shift
  echo "\n### $header" >> "$OUT_FILE"
  echo "\n```text" >> "$OUT_FILE"
  _run_help "$@" >> "$OUT_FILE"
  echo "```" >> "$OUT_FILE"
}

mkdir -p "$ROOT_DIR/docs"
echo "# KCura CLI Surface (auto-generated)\n" > "$OUT_FILE"
echo "Generated: $(date -u +%Y-%m-%dT%H:%M:%SZ)\nCommit: $(git -C "$ROOT_DIR" rev-parse --short HEAD 2>/dev/null || echo unknown)\n" >> "$OUT_FILE"

# Top-level help
if ! _run_help >/dev/null; then
  echo "❌ failed to render top-level help" >&2
  exit 1
fi
_append_section "kcura --help"

# First-level subcommands
mapfile -t SUBS < <(_list_subs)
if [[ ${#SUBS[@]} -eq 0 ]]; then
  echo "⚠️  no subcommands detected" >&2
fi

declare -i FAIL=0
for cmd in "${SUBS[@]}"; do
  if ! _run_help "$cmd" >/dev/null; then
    echo "❌ help failed for: $cmd" >&2; FAIL=1; continue
  fi
  _append_section "kcura $cmd --help" "$cmd"

  # nested subcommands (second level)
  mapfile -t NESTED < <(_list_subs "$cmd")
  for sub in "${NESTED[@]}"; do
    if ! _run_help "$cmd" "$sub" >/dev/null; then
      echo "❌ help failed for: $cmd $sub" >&2; FAIL=1; continue
    fi
    _append_section "kcura $cmd $sub --help" "$cmd" "$sub"
  done
done

if [[ $FAIL -ne 0 ]]; then
  echo "\n=== ❌ CLI scan found broken help paths ===" >&2
  exit 1
fi

echo "\n=== ✅ CLI scan complete → $OUT_FILE"

