#!/usr/bin/env python3
"""Provenance gate for the frozen-duckdb release path (v26.9.21 C5).

Wires `ggen receipt verify` (BLAKE3 chain recompute over .ggen-v2/receipt-log.jsonl)
into the PUBLISH_RUNBOOK gate sequence, and validates the verdict document against
the RENDERED receipt-contract census (generated/receipt_contract_matrix.json,
contract key `ggen_sync_receipt`). The field contract lives in the ontology
(schema/domain.ttl, rp: vocabulary from receipt-provenance-unification-pack) and
its render; this script is only the executor. Do not hand-edit the matrix.

Verdict semantics: gate is GREEN only when
  1. `ggen receipt verify`   exits 0 with valid=true and signature_valid=true
  2. `ggen receipt history`  exits 0 (full chain replays)
  3. every mandatory field of the ggen_sync_receipt contract is present in the
     verdict, matches its declared JSON type, and fullmatches its declared
     pattern (when a pattern is declared)

Exit codes: 0 = GREEN, 1 = RED (gate failure), 2 = unusable (missing render /
ggen missing / parse error).

Usage: python3 scripts/verify_publish_receipt.py   (run from repo root)
"""

from __future__ import annotations

import json
import re
import shutil
import subprocess
import sys
from pathlib import Path

CONTRACT = "ggen_sync_receipt"
MATRIX = Path("generated/receipt_contract_matrix.json")


def fail(code: int, msg: str) -> int:
    print(f"PROVENANCE-GATE RED: {msg}", file=sys.stderr)
    return code


def run_ggen(args: list[str]) -> tuple[int, str]:
    proc = subprocess.run(
        ["ggen", *args, "--format", "json"],
        capture_output=True, text=True,
    )
    # ggen logs tracing lines to stdout; the verdict is the last JSON object.
    line = ""
    for raw in proc.stdout.splitlines():
        if raw.lstrip().startswith("{"):
            line = raw
    return proc.returncode, line


def main() -> int:
    if shutil.which("ggen") is None:
        return fail(2, "ggen CLI not found on PATH")
    if not MATRIX.is_file():
        return fail(2, f"{MATRIX} missing — run `ggen sync run` first (rendered consequence)")

    try:
        matrix = json.loads(MATRIX.read_text())
    except json.JSONDecodeError as exc:
        return fail(2, f"{MATRIX} is not valid JSON: {exc}")

    rows = [f for f in matrix.get("fields", []) if f.get("contract") == CONTRACT]
    if not rows:
        return fail(2, f"no {CONTRACT} rows in {MATRIX} — ontology facts missing from render")

    # Gate 1: chain recompute + signature over the current sync receipt.
    code, line = run_ggen(["receipt", "verify"])
    if code != 0 or not line:
        return fail(1, f"`ggen receipt verify` exit {code}: no verdict parsed")
    try:
        verdict = json.loads(line)
    except json.JSONDecodeError as exc:
        return fail(1, f"`ggen receipt verify` output not JSON: {exc}")

    problems: list[str] = []
    if verdict.get("valid") is not True:
        problems.append(f"valid={verdict.get('valid')!r} (chain recompute failed)")
    if verdict.get("signature_valid") is not True:
        problems.append(f"signature_valid={verdict.get('signature_valid')!r}")

    # Gate 2: field shapes per the RENDERED contract census.
    for row in rows:
        path, name = row["path"], row["path"]
        if path not in verdict:
            if row.get("mandatory"):
                problems.append(f"missing mandatory field `{name}`")
            continue
        value = verdict[path]
        jtype = row.get("json_type")
        if jtype == "integer" and not isinstance(value, int):
            problems.append(f"`{name}` expected integer, got {type(value).__name__}")
        elif jtype == "boolean" and not isinstance(value, bool):
            problems.append(f"`{name}` expected boolean, got {type(value).__name__}")
        elif jtype == "string" and not isinstance(value, str):
            problems.append(f"`{name}` expected string, got {type(value).__name__}")
        pattern = row.get("pattern", "")
        if pattern and isinstance(value, str) and re.fullmatch(pattern, value) is None:
            problems.append(f"`{name}`={value!r} fails value form {row.get('value_form')!r} ({pattern})")

    # Gate 3: the whole receipt log replays.
    hcode, hline = run_ggen(["receipt", "history"])
    history: dict = {}
    if hcode == 0 and hline:
        try:
            history = json.loads(hline)
        except json.JSONDecodeError:
            pass
    if hcode != 0 or history.get("valid") is not True:
        problems.append(f"`ggen receipt history` exit {hcode}, valid={history.get('valid')!r} (chain replay failed)")

    print("ggen receipt verify verdict:")
    print(json.dumps(verdict, indent=2))
    print(f"ggen receipt history: {history.get('records', '?')} records, valid={history.get('valid')}, "
          f"head_chain_hash={history.get('head_chain_hash', '?')}")
    print(f"contract {CONTRACT}: {len(rows)} fields checked against {MATRIX}")

    if problems:
        for p in problems:
            print(f"PROVENANCE-GATE RED: {p}", file=sys.stderr)
        return 1

    print("PROVENANCE-GATE GREEN: receipt chain verified, signature valid, "
          f"{CONTRACT} shapes conform to the rendered census")
    return 0


if __name__ == "__main__":
    sys.exit(main())
