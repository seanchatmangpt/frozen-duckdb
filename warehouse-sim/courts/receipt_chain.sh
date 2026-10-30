#!/usr/bin/env bash
# G6 receipt-chain court — bind the real inputs and court outputs into one
# receipt with the operator pin + date, and validate the parseable surfaces:
#   - rdflib parse of schema/warehouse_sim.ttl
#   - tsc --noEmit over warehouse-sim
# Fail-closed; loud skips; no git state mutations; blake3 (b3sum) preferred,
# falls back to sha256 and states which in the receipt.
set -u
COURT_DIR="$(cd "$(dirname "$0")" && pwd)"
WS_DIR="$(dirname "$COURT_DIR")"
TTL="$WS_DIR/schema/warehouse_sim.ttl"
EV_DIR="$COURT_DIR/.evidence"
RECEIPT="$COURT_DIR/receipt.json"
PIN="d54cbbe3df17e8e5ce68312f483b4155836112aa"
DATE_UTC="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
mkdir -p "$EV_DIR"
log() { printf '%s\n' "$*"; }

log "== receipt-chain court =="
log "date (UTC): $DATE_UTC"

# ---- hash algorithm: blake3 preferred ---------------------------------------
ALGO=blake3
if command -v b3sum >/dev/null 2>&1; then
  H() { b3sum "$1" | awk '{print $1}'; }
else
  ALGO=sha256
  H() { shasum -a 256 "$1" | awk '{print $1}'; }
fi
log "hash algo: $ALGO ($(command -v b3sum >/dev/null 2>&1 && echo b3sum || echo 'shasum fallback'))"

# ---- validation 1: rdflib parse of the ttl ----------------------------------
python3 - "$TTL" > "$EV_DIR/rdflib.out" 2>&1 <<'PY'
import sys
try:
    from rdflib import Graph
except ImportError:
    print("rdflib-missing"); sys.exit(3)
g = Graph(); g.parse(sys.argv[1], format="turtle")
print(f"rdflib OK: {len(g)} triples from {sys.argv[1]}")
PY
RDFLIB_RC=$?
case $RDFLIB_RC in
  0) log "rdflib: $(cat "$EV_DIR/rdflib.out")" ;;
  3) log "[SKIP] rdflib not installed — ttl parse not verified this run" ;;
  *) log "FAIL: rdflib parse of ttl failed"; sed -n '1,5p' "$EV_DIR/rdflib.out" ;;
esac

# ---- validation 2: tsc --noEmit ---------------------------------------------
TSC_RC=0
if command -v tsc >/dev/null 2>&1 || [ -x "$WS_DIR/node_modules/.bin/tsc" ]; then
  TSC_BIN="$(command -v tsc || echo "$WS_DIR/node_modules/.bin/tsc")"
  ( cd "$WS_DIR" && "$TSC_BIN" --noEmit ) > "$EV_DIR/tsc.out" 2>&1
  TSC_RC=$?
  if [ $TSC_RC -eq 0 ]; then
    log "tsc --noEmit: green (rc=0)"
  else
    log "FAIL: tsc --noEmit rc=$TSC_RC"; sed -n '1,10p' "$EV_DIR/tsc.out"
  fi
else
  TSC_RC=2
  log "[SKIP] tsc not available"
fi

# ---- hash inventory ----------------------------------------------------------
if [ ! -f "$TTL" ]; then log "[SKIP] ttl absent"; exit 2; fi
python3 - "$WS_DIR" "$EV_DIR/hashes.json" "$ALGO" <<'PY'
import json, hashlib, subprocess, sys, os
ws, out, algo = sys.argv[1], sys.argv[2], sys.argv[3]
def h(p):
    if algo == "blake3":
        r = subprocess.run(["b3sum", p], capture_output=True, text=True)
        if r.returncode == 0: return r.stdout.split()[0]
    return hashlib.sha256(open(p, "rb").read()).hexdigest()
manifest = os.path.join(ws, "ggen.toml")
routes = {}
for line in open(manifest):
    if "output_file" in line:
        rel = line.split('"')[1]
        fp = os.path.join(ws, rel)
        routes[rel] = h(fp) if os.path.isfile(fp) else None
json.dump({
    "schema_ttl": h(os.path.join(ws, "schema/warehouse_sim.ttl")),
    "ggen_manifest": h(manifest),
    "rendered": routes,
    "lib_sim_engine_ts": h(os.path.join(ws, "lib/sim_engine.ts")) if os.path.isfile(os.path.join(ws, "lib/sim_engine.ts")) else None,
    "lib_ledger_ts": h(os.path.join(ws, "lib/ledger.ts")) if os.path.isfile(os.path.join(ws, "lib/ledger.ts")) else None,
}, open(out, "w"), indent=2, sort_keys=True)
PY
HASH_RC=$?
[ $HASH_RC -ne 0 ] && log "FAIL: hash inventory failed"

# ---- assemble receipt ---------------------------------------------------------
export RDFLIB_RC TSC_RC HASH_RC DATE_UTC PIN ALGO
python3 - "$EV_DIR" "$RECEIPT" "$EV_DIR/hashes.json" <<'PY'
import json, os, subprocess, sys
ev, receipt, hashes = sys.argv[1], sys.argv[2], sys.argv[3]
algo = os.environ["ALGO"]
def h(p):
    if algo == "blake3":
        r = subprocess.run(["b3sum", p], capture_output=True, text=True)
        if r.returncode == 0: return r.stdout.split()[0]
    import hashlib
    return hashlib.sha256(open(p, "rb").read()).hexdigest()
court_outputs = {}
for f in sorted(os.listdir(ev)):
    p = os.path.join(ev, f)
    if os.path.isfile(p):
        court_outputs[f] = {"hash": h(p), "bytes": os.path.getsize(p)}
verdicts = {}
for f in ("determinism.verdict", "schema-drift.verdict"):
    fp = os.path.join(ev, f)
    verdicts[f.replace(".verdict", "")] = open(fp).read().strip() if os.path.isfile(fp) else "ABSENT (run determinism.sh first)"
rdflib_out = open(os.path.join(ev, "rdflib.out")).read().strip()
out = {
    "receipt": "warehouse-sim/g6-courts/v1",
    "date_utc": os.environ["DATE_UTC"],
    "operator_pin": os.environ["PIN"],
    "hash_algo": algo,
    "inputs": json.load(open(hashes)),
    "court_outputs": court_outputs,
    "verdicts": verdicts,
    "validations": {
        "rdflib_ttl_parse": rdflib_out[:200],
        "rdflib_rc": 0 if rdflib_out.startswith("rdflib OK") else int(os.environ["RDFLIB_RC"]),
        "tsc_noEmit_rc": int(os.environ["TSC_RC"]),
    },
}
json.dump(out, open(receipt, "w"), indent=2, sort_keys=True)
print("receipt written:", receipt)
PY
FINAL_RC=$?
log ""
cat "$RECEIPT"

# ---- fail-closed verdict -------------------------------------------------------
if [ $RDFLIB_RC -ne 0 ] || [ $TSC_RC -ne 0 ] || [ $FINAL_RC -ne 0 ]; then
  log "receipt-chain court: FAIL (validation or assembly failure)"
  exit 1
fi
if grep -q "FAIL" "$EV_DIR/determinism.verdict" 2>/dev/null || \
   grep -q "FAIL" "$EV_DIR/schema-drift.verdict" 2>/dev/null; then
  log "receipt-chain court: FAIL (a render court is red)"
  exit 1
fi
log "receipt-chain court: green"
exit 0
