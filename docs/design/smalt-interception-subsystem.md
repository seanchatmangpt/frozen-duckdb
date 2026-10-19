# Smalt Interception Subsystem: Cross-Repo Design Capture

**Status**: CROSS-REPO CAPTURE — smalt lives in a SEPARATE repo (canonical
checkout elsewhere). Nothing in this document is implemented in
frozen-duckdb. All latency/perf numbers carry the source spec's claims as
UNVERIFIED-here; no gate, bench, or court in this repo substantiates them.

## Purpose

Capture Subsystem C (smalt) of the sovereign spec as a durable design
surface: wire-interception of Azure/boto3 SDK protocols on BEAM, the
admission equation, the in-process state fabric, and PQC custody — plus the
one place smalt touches this repo: frozen-duckdb embeds as the
relational/document engine inside smalt's state fabric.

## 1. Wire-interception of SDK protocols on BEAM

smalt intercepts cloud SDK wire protocols at the protocol layer rather than
patching SDK client libraries, so one interceptor serves any language whose
SDK speaks the wire protocol.

| surface | port | mechanism |
|---|---|---|
| Azure blob/table SDKs (TLS twin termination) | 9444 | TLS twin |
| AMQP 1.0 (Service Bus / Event Hubs) | 5672 | protocol interceptor |
| REST / ARM plane | 9443 | REST interceptor |
| OAuth2 client-credentials + RS256/JWKS STS | n/a | token issuer |

- **TLS twin termination (:9444)** — terminates the SDK's TLS at a local
  twin endpoint so payload bytes are inspectable in clear inside the BEAM
  node; the twin re-originates to the real endpoint per policy
  (pass-through, mutate, refuse).
- **AMQP 1.0 (:5672)** — terminates AMQP 1.0 frames (Service Bus / Event
  Hubs clients) natively on BEAM, enabling broker-like semantics
  (peek/lock, settle) without the real broker.
- **REST (:9443)** — terminates plain TLS REST/ARM calls for surfaces the
  twin/AMQP listeners do not cover.
- **OAuth2 STS** — issues tokens signed RS256 with a locally-served JWKS,
  so SDKs complete real client-credentials flows against the local STS.
  STS signing keys come from the same PQC custody boundary (§4).

Design law: the interceptor never patches client libraries — fidelity
comes from speaking the wire protocol itself; one interceptor per protocol,
not per SDK.

## 2. Admission equation

The governance kernel of smalt is the equation:

```
ΔS_admitted + Φ_lease ≡ R_receipt (mod Σ_PQC)
```

- `ΔS_admitted` — admitted state delta (the change actually landed).
- `Φ_lease` — outstanding leases / authority grants (in-flight authority).
- `R_receipt` — the receipt ledger; claims settle against receipts.
- `mod Σ_PQC` — all terms under post-quantum custody: deltas and receipts
  are custody-sealed, leases are custody-sealed, ledger integrity is
  PQC-signed (FIPS 203 ML-KEM / FIPS 204 ML-DSA — see §4).

Reading: a state change is admitted only when the admitted delta plus
outstanding leases reconcile exactly against the receipt ledger, all under
PQC custody. Unreceipted actuation is a reconciliation failure, not a
policy opinion.

## 3. In-process state fabric table

smalt replaces per-service cloud dependencies with in-process BEAM
primitives:

| cloud service | in-process twin | dimension | claim (UNVERIFIED-here) |
|---|---|---|---|
| Redis | ETS | read latency | < 800 ns |
| Cosmos DB | Mnesia / DETS | durability | DETS-backed persistence |
| Azure Service Bus | pg + gen_statem | delivery | at-least-once via pg dispatch + state machines |
| Azure API Management | Ranch / Cowboy | actuation | in-process HTTP + TLS termination |
| DuckDB SQL + docs | frozen-duckdb (this repo) | query/rel + document engine | ALIVE here, see §5 |

## 4. PQC custody

All custody sealing uses FIPS 203 (ML-KEM) and FIPS 204 (ML-DSA):

- ML-KEM: key encapsulation of custody keys at rest.
- ML-DSA: signatures over every custody seal (deltas, leases, receipts).
- STS signing keys (§1) are served via JWKS from the same custody
  boundary, so the token plane and the custody plane share one key root.

## 5. Relation to frozen-duckdb

frozen-duckdb embeds as the relational/document engine inside the state
fabric (§3, engine row). The division of standing:

- smalt's fabric-table claims (ETS latency, AMQP/REST fidelity, PQC
  custody) are UNVERIFIED-here — they belong to the smalt repo's courts.
- frozen-duckdb's own standing is receipt-gated in THIS repo:
  `make verify-gates` (5 native gates + ggen receipt verify) is the
  evidence for the engine slot; it runs on this checkout, not smalt's.

Any future integration order (smalt consuming frozen-duckdb) must cite a
receipt from both repos; neither side's standing transfers by prose.
