# MAX10 — Maximization-Spec Closure Summary

Date: 2026-10-19. Lane MAX10. Full receipt:
`docs/design/maximization-receipt.md` (all claims cited there).

**Four boundaries.** Zero-Poison: MAX1 poison-exit court PASS (46+4+12
builder tests, `MAX1-poison-exit.md`) + MAX5 STALE_PRECONDITION_REFUSED guard
(5/5 this lane, receipt in-flight). Zero-Alloc: memory-ceiling family
PARTIAL_ALIVE on native legs (`COURT-INDEX.md`: rev004 OBSERVED 256MB,
fleet002 RSS WITHIN-BUDGET); wasm leg open. Zero-Leak: `PB6-leakscan.md`
clean on all 3 crates. Zero-Drift: `make gates` run by this lane —
`genesis-check: checked=58 … failed=0`, `GATES GREEN` exit 0; SP2 ABI
baseline sha identity.

**Net-new, this lane's re-run (one command, 14 passed / 0 failed,
`--features serde_json,vscalar,vtab`):** pot_ingestion 1/1 (MAX2, receipt
in-flight; X1r differential witness shows its throughput floor is
pre-existing, not fence-caused), monotonic_ingress 2/2 (MAX8, non-vacuous by
shim flip), variant_shredded_bind 4/4 (MAX3, 100k rows @ 80,244 rows/s),
query_receipt 1/1 (MAX4, BLAKE3 tamper-red court), precondition_guard 5/5
(MAX5), dfg_cut_detection 1/1 (MAX9, oracle-exact on 1M edges, 4.968 ms).

**KD movement.** KD-05: minimal wasm 24.69 MiB PASS (`KDC-05`, `X3`), full-spec
build honestly FAIL at 27.89 MiB (`MAX6`) with the icu lever dead (`CL4b`),
converged module 24.54 MiB promoted to `target/wasm-min/kdc05-min.wasm`
(`MAX7`, sha `1fdeda0d…4bc1c7`). KD-06: local `read_local_range` slice ALIVE,
6/6 tests (`CL13`); icu sidecar BLOCKED at rung 3 (`SP4`). KD-01
CLOSED-LOCAL: fence 1,177 C-ABI / 0 mangled, gate proven able to fail
(`X1r`).

**Open.** Network host-callback delegation (GATE-V-05 unexercised); browser
OPFS L2 typed MISSING (`KDC-11`); 25 MiB SLA unreachable for full-spec build
at this pin; fence-restage hazard — a poison-refusal cache restage silently
reverts the KD-01 fence (builder fallback lacks `exports.list`) — open work
order. In-flight, not cited: MAX2/MAX5 receipts, SP5, REL4r, REL6r.
