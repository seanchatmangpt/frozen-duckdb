# sovereign-lifecycle-contracts

Status: governing spec for the four-phase fail-closed lifecycle
(Bootstrapping / First Mile / Lifecycle / Last Mile), transcribed from the
operator's 2026-10-19 topology. Every phase maps to landed courts; net-new
contracts are labeled.

## Phase map (spec → landed courts)

| Phase | Boundary | Landed witness | Net-new contract |
|---|---|---|---|
| Bootstrapping | Cold cache → verified runtime | ENGINE_VERSION marker + size pin + Mach-O magic + 10MiB floor + BuildLock (siege-proven, 46/46) | Negative-mutation shims: POISON_STATE_DETECTED typed exit (AD1/AD6 courts cover; formalize the exit code) |
| First Mile | Event log → trace lattice | tpch/ocel ingress gated (monotonic schema checks; tpch harness timer bug fixed) | TRACE_SCHEMA_BREACH typed rejection + −5ms causality shim (spec'd, not implemented) |
| Lifecycle | Ingested trace → conformance state | OOM typed aborts (X9), 128MB cgroup kernel enforcement (DOCKER1/KDC-09), leak deceleration law (D6), state conservation (X4/ADV14) | Lease-renewal failure → deterministic termination (spec'd; cgroup witness covers memory; lease renewal itself is the gap) |
| Last Mile | Analytic output → enterprise action | value-audit mint→verify integrity chain (AD4); TAG-REHEARSAL precondition checks | STALE_PRECONDITION_REFUSED two-phase commit (spec'd for actuation; the render --verify manifest is the analog) |

## Fail-closed exit contract

Exit `0` clean / `1` invariant breach / `2` environment precondition /
`3` architectural divergence (KD-registered) / `124` bounded-watchdog
ceiling. Every phase script in `scripts/doctor/` carries this contract.

## Negative-mutation shims (landed)

- Bootstrapping: truncated dylib, foreign marker, symlinked marker, fat
  binary, missing pin (AD1/AD6 siege matrix)
- First Mile: malformed JSON 400s, CRLF injection, __proto__ keys (AD2)
- Lifecycle: kill -9 mid-txn + no-op-kill flip (Z2/X4), Page.crash (BP2 spec)
- Last Mile: doctored sovereign receipt refused exit 3 (G5/AD4 selftest)

## Net-new implementation queue (spec'd, unblocked)

1. TRACE_SCHEMA_BREACH typed rejection in the ocel ingress path
2. −5ms causality-dependence shim (quarantine, not drop)
3. Lease-renewal failure → deterministic typed termination
4. STALE_PRECONDITION_REFUSED two-phase actuation guard

## See Also

doctor-verification-blueprint.md - gate-v01..05 specs - KD-CLOSURE-PLAN.md
