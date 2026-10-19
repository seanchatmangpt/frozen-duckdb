#!/usr/bin/env bash
# G6 courts — warehouse-sim render side. Self-contained.
#   Part A: determinism court  — two `ggen sync run` passes; every rendered
#           file must be sha256-identical across passes.
#   Part B: schema-drift court — a SHIM-COURT-marked copy of the ttl in /tmp
#           (creditCostPerHour mutated) must change the rendered ledger
#           constant; proves the graph, not the templates, is the source.
# No git state mutations; render targets are lane-local; the drift court
# touches only a /tmp copy. Fail-closed: a court that cannot flip is FAIL.
set -u
COURT_DIR="$(cd "$(dirname "$0")" && pwd)"
WS_DIR="$(dirname "$COURT_DIR")"
MANIFEST="$WS_DIR/ggen.toml"
TTL="$WS_DIR/schema/warehouse_sim.ttl"
EV_DIR="$COURT_DIR/.evidence"
HASH="sha256sum"; command -v sha256sum >/dev/null 2>&1 || HASH="shasum -a 256"
h() { $HASH "$1" | awk '{print $1}'; }
log() { printf '%s\n' "$*"; }

log "== warehouse-sim render courts =="
log "date (UTC): $(date -u +%Y-%m-%dT%H:%M:%SZ)"
log "ggen: $(command -v ggen && ggen --version 2>&1 | head -1)"
OVERALL=0

# ---- prerequisites ----------------------------------------------------------
if ! command -v ggen >/dev/null 2>&1; then log "[SKIP] ggen not on PATH"; exit 2; fi
if [ ! -f "$MANIFEST" ]; then log "[SKIP] $MANIFEST absent"; exit 2; fi
if [ ! -f "$TTL" ]; then log "[SKIP] $TTL absent"; exit 2; fi

# Rendered-file inventory: read output_file entries straight from the manifest
# (mechanical, not hand-listed).
mapfile -t RENDERED < <(grep -o 'output_file = "[^"]*"' "$MANIFEST" | sed 's/output_file = "//; s/"//')
if [ "${#RENDERED[@]}" -eq 0 ]; then log "[SKIP] no output_file rules in manifest"; exit 2; fi
log "rendered inventory (${#RENDERED[@]}): ${RENDERED[*]}"

do_render() { # $1 = log file ; echoes rc
  ( cd "$WS_DIR" && ggen sync run ) > "$1" 2>&1
}

# ==== PART A: determinism court ==============================================
log ""
log "--- Part A: determinism court (BASELINE -> BEFORE -> SHIM=second render -> AFTER) ---"
mkdir -p "$EV_DIR"
rm -f "$EV_DIR"/* 2>/dev/null   # stale evidence from prior runs is not evidence

# BASELINE: hash of the source graph (the court subject).
log "BASELINE schema/warehouse_sim.ttl sha256 = $(h "$TTL")"

if do_render "$EV_DIR/render-pass1.log"; then
  : > "$EV_DIR/pass1.sha"
  for f in "${RENDERED[@]}"; do
    [ -f "$WS_DIR/$f" ] && printf '%s  %s\n' "$(h "$WS_DIR/$f")" "$f" >> "$EV_DIR/pass1.sha"
  done
  log "BEFORE (pass 1): rc=0, $(wc -l < "$EV_DIR/pass1.sha" | tr -d ' ') files hashed"
else
  log "FAIL (Part A): ggen sync run pass 1 exited nonzero — see $EV_DIR/render-pass1.log"
  sed -n '/ERROR/p' "$EV_DIR/render-pass1.log" | head -3
  echo "determinism: FAIL (render rc!=0; template defect upstream of G6)" > "$EV_DIR/determinism.verdict"
  OVERALL=1
fi

if [ "$OVERALL" -eq 0 ]; then
  # SHIM for determinism is the second identical render; AFTER must match.
  if do_render "$EV_DIR/render-pass2.log"; then
    : > "$EV_DIR/pass2.sha"
    for f in "${RENDERED[@]}"; do
      [ -f "$WS_DIR/$f" ] && printf '%s  %s\n' "$(h "$WS_DIR/$f")" "$f" >> "$EV_DIR/pass2.sha"
    done
    log "AFTER  (pass 2): rc=0, $(wc -l < "$EV_DIR/pass2.sha" | tr -d ' ') files hashed"
    if diff -u "$EV_DIR/pass1.sha" "$EV_DIR/pass2.sha" > "$EV_DIR/determinism.diff" 2>&1; then
      log "PASS (Part A): all rendered files byte-identical across two renders"
      echo "determinism: PASS ($(wc -l < "$EV_DIR/pass1.sha" | tr -d ' ') files identical)" > "$EV_DIR/determinism.verdict"
    else
      log "FAIL (Part A): render drift across passes"
      cat "$EV_DIR/determinism.diff"
      echo "determinism: FAIL (drift)" > "$EV_DIR/determinism.verdict"
      OVERALL=1
    fi
  else
    log "FAIL (Part A): pass 2 render exited nonzero"
    echo "determinism: FAIL (pass2 rc!=0)" > "$EV_DIR/determinism.verdict"
    OVERALL=1
  fi
fi

# ==== PART B: schema-drift court =============================================
log ""
log "--- Part B: schema-drift court (copy-only; no repo restore needed) ---"
DRIFT_ROOT="$(mktemp -d /tmp/ws-drift-court.XXXXXX)"
trap 'rm -rf "$DRIFT_ROOT"' EXIT
mkdir -p "$DRIFT_ROOT/ws"
# copy the whole manifest surface (schema, templates, manifest) into scratch
cp -R "$WS_DIR/schema" "$WS_DIR/templates" "$MANIFEST" "$DRIFT_ROOT/ws/" 2>/dev/null

DRIFT_TTL="$DRIFT_ROOT/ws/schema/warehouse_sim.ttl"
if [ ! -f "$DRIFT_TTL" ]; then
  log "[SKIP] copy failed"
  echo "schema-drift: SKIP (copy failed)" > "$EV_DIR/schema-drift.verdict"
  exit 1
fi

# SHIM: mutate creditCostPerHour in the COPY, marked SHIM-COURT.
python3 - "$DRIFT_TTL" <<'PY'
import re, sys
p = sys.argv[1]
s = open(p).read()
# value-format-agnostic: replaces ONLY the numeric literal inside the quotes
# of the creditCostPerHour object ("4.0"^^xsd:decimal -> "5.5"^^xsd:decimal),
# preserving the existing datatype annotation untouched.
pat = re.compile(r'((?:ws|sim):creditCostPerHour\s+")(\d+(?:\.\d+)?)(")')
m = pat.search(s)
assert m, "creditCostPerHour literal not found in ttl copy"
new = pat.sub(lambda x: x.group(1) + str(float(x.group(2)) + 1.5) + x.group(3), s, count=1)
marker = "# SHIM-COURT: creditCostPerHour mutated for schema-drift court (copy only)\n"
open(p, "w").write(marker + new)
print("shim ok")
PY
SHIM_RC=$?
if [ $SHIM_RC -ne 0 ]; then
  log "[SKIP] shim mutation failed on copy"
  echo "schema-drift: SKIP (shim failed)" > "$EV_DIR/schema-drift.verdict"
  exit 1
fi
log "SHIM applied in $DRIFT_TTL (creditCostPerHour +1.5, marked SHIM-COURT)"
log "SHIM-COURT baseline copy hash: $(h "$TTL") -> mutated copy $(h "$DRIFT_TTL")"

# Render the mutated copy to a scratch out-dir (the copy IS the out-dir).
if ( cd "$DRIFT_ROOT/ws" && ggen sync run ) > "$EV_DIR/drift-render.log" 2>&1; then
  LEDGER="$DRIFT_ROOT/ws/lib/ledger.ts"
  if [ ! -f "$LEDGER" ]; then
    log "FAIL (Part B): mutated render produced no lib/ledger.ts"
    echo "schema-drift: FAIL (no ledger rendered)" > "$EV_DIR/schema-drift.verdict"
    OVERALL=1
  else
    ORIG_CONST="$(grep -E '^export const creditCostPerHour' "$WS_DIR/lib/ledger.ts" 2>/dev/null | awk '{print $NF}' | tr -d ';')"
    DRIFT_CONST="$(grep -E '^export const creditCostPerHour' "$LEDGER" | awk '{print $NF}' | tr -d ';')"
    log "ledger constant repo=$ORIG_CONST drift-copy=$DRIFT_CONST"
    if [ -n "$DRIFT_CONST" ] && [ "$DRIFT_CONST" != "$ORIG_CONST" ] && [ "$DRIFT_CONST" = "5.5" ]; then
      log "PASS (Part B): graph mutation changed the rendered constant (graph is the source of truth)"
      echo "schema-drift: PASS (4.0 -> $DRIFT_CONST)" > "$EV_DIR/schema-drift.verdict"
    else
      log "FAIL (Part B): rendered ledger constant did not flip as expected"
      echo "schema-drift: FAIL (expected 5.5, got '$DRIFT_CONST' vs orig '$ORIG_CONST')" > "$EV_DIR/schema-drift.verdict"
      OVERALL=1
    fi
  fi
else
  log "FAIL (Part B): mutated-copy render exited nonzero — graph-driven flip unprovable while templates are broken"
  sed -n '/ERROR/p' "$EV_DIR/drift-render.log" | head -3
  echo "schema-drift: FAIL (mutated render rc!=0; template defect upstream of G6)" > "$EV_DIR/schema-drift.verdict"
  OVERALL=1
fi

log ""
log "verdicts:"
cat "$EV_DIR"/*.verdict 2>/dev/null
exit $OVERALL
