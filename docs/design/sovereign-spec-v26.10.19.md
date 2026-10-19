# sovereign-spec-v26.10.19.md

Status: source-of-truth capture of the operator's "Sovereign Systems
Architecture & Operational Specification v26.10.19" (unified dual-target
platform). Subsystems ex4pm and smalt live in OTHER repos — this repo
(frozen-duckdb) implements Subsystem A (the dual-target kernel) plus the
GATE-V courts. Per repo law, everything not executed in-repo is
PROPOSED/UNVERIFIED; operator-specified claims about foreign subsystems
are recorded here as specification, never as standing.

## 1. Executive doctrine

One specification, one court ladder, two targets: the same DuckDB engine
runs natively on end-user silicon and as a WASM module inside the browser
sandbox. The engine is a tenant, not a peer — the host owns the network
stack, the filesystem, credentials, and time. The engine never performs
I/O it cannot have bounded and receipted.

Doctrine points:

- Sovereignty is displacement: work executed on silicon the customer
  already owns is work that never incurred a cloud scan fee (see
  `soc-valuation-model.md`).
- Every capability claim is a court receipt, not a paragraph. A gate
  that cannot exit 1 is decoration.
- Negative-marginal-cost axiom (Section 2): the platform's marginal
  cost per additional query trends to zero because compute is
  displaced to client hardware and the vendor's residual cost is
  bytes (bundle + data sync), per the BP4 margin mechanics.
- Standing vocabulary is uniform across all four subsystems:
  PROPOSED / PARTIAL_ALIVE / ALIVE / BLOCKED / UNSUPPORTED, always
  receipt-gated.

### 1.1 Topology

```
+--------------------------------------------------------------+
|                 SOVEREIGN PLATFORM v26.10.19                 |
|                                                              |
|  [A] frozen-duckdb            [B] ex4pm                      |
|      dual-target kernel          real-time conformance       |
|      (this repo)                 (other repo)                |
|         |    ^                       |                       |
|         |    | OCEL event log        | conformance verdicts  |
|         v    |                       v                       |
|  [C] smalt                     [D] Chicago verification      |
|      wire-interception             substrate (courts,        |
|      (other repo)                  shim-courts, receipts)    |
|         |                            ^                       |
|         +------- intercepted --------+                       |
|                  traffic twins                               |
+--------------------------------------------------------------+
```

Data flow: [A] emits OCEL usage events (each query execution IS an
object-centric event by construction); [C] intercepts wire traffic and
terminates it into twins; [B] evaluates conformance against those twins
in real time; [D] is the verification substrate every other subsystem
consumes — courts, shim-court non-vacuity protocol, and the receipt
schema. This repo owns [A] and the GATE-V courts of [D] that bind to it.

## 2. Subsystem A — frozen-duckdb dual-target kernel

### 2.1 Host-VFS range delegation

The embedded engine sees a local file surface; the host resolves remote
fetches. TLS/HTTP termination lives in the host, never in the
mega-library (architecture: `host-call-vfs.md`; contract:
`host-range-vfs-contract.md`).

Registered VFS prefix: `host-range://`, internal — user SQL types
`read_parquet('<url>')` and the shim rewrites the scheme.

Formal signature (from the contract, quoted):

```
host_read_range(URL: string,
                Offset: i64,          # byte offset, 0-based, from BOF
                Length: u64,          # requested byte count
                BufferPtr: mut [u8; Length]) -> Status
```

Wire form (HTTP Range header issued by the host for one call):

```
Range: bytes=<Offset>-<Offset + Length - 1>
```

One call = one contiguous range; no scatter/gather in v1. On `Success`
the callee has written exactly `Length` bytes; negative offsets are
never guest-visible.

### 2.2 80/20 static extension table

Four extensions carry ~80% of real analytical workloads
(`extension-matrix-8020.md`):

| Extension | Role | Pattern |
|---|---|---|
| `parquet` | columnar interchange; format of record | static bake |
| `json` | semi-structured ingest; variant-adjacent | static bake |
| `icu` | timezone + collation | static bake — drops out of the core
set only once DuckDB 2.0 native timezone support is verified;
currently UNVERIFIED-UPSTREAM-CLAIM, so `icu` stays baked |
| `httpfs` | remote object-store reads | EXCLUDED from the static
bake — replaced by the host-call VFS wrap (Section 2.1) |

Exclusion law: `httpfs` is the load-bearing trim. TLS stacks, curl,
and cert-store coupling stay out of the mega-library; the host already
owns a network stack. Current NATIVE builder diverges (nine-extension
pin in `crates/frozen-duckdb-builder/src/lib.rs`); trimming is tracked
in `extension-matrix-8020.md`.

### 2.3 Unboxed C ABI surface

Boundary law (`boundary-abi-contract.md`): primitives, pointers, and
lengths only. No `Result`/`Option`/`Vec`/`String`/fat slices across the
line; no ownership transfer; no panics across the boundary; every
acquire has exactly one destroy.

Sovereign export surface (operator-specified names; WASM instantiation
PROPOSED/UNVERIFIED, native instantiation maps onto the existing
`duckdb_*` surface):

| Symbol | Role |
|---|---|
| `fdb_init_engine` | acquire engine instance (config in, handle out) |
| `fdb_close_engine` | paired destroy; no implicit lifetimes |
| `fdb_ingest_conforming_batch` | zero-copy Arrow batch ingest (host
allocates, engine fills in place) |
| `fdb_execute_query` | vectorized query execution; Arrow C structures
out, release via the paired destroy symbol |
| `fdb_register_host_vfs` | register the host range-read callback
(Section 2.1) under the internal prefix |

### 2.4 VARIANT pipeline

Grounded state (`variant-shredding-design.md`): `DUCKDB_TYPE_VARIANT=41`
integrated end to end — `LogicalTypeId::Variant`, bind arm, serde_json
`FromSql`; DDL and `'x'::VARIANT` casts accepted; varchar binds
implicitly cast; Arrow projection surfaces `Struct(metadata Binary,
value Binary)`. Pipeline phases:

1. Phase 1 (next native step): construct `ValueRef::Variant` from live
   columns by decoding the Arrow Struct — close the Utf8-passthrough
   read gap.
2. Phase 2: true shredded columnar reads — project `metadata`/`value`
   as separate streams; pruning and partial projection on shredded
   paths. UNVERIFIED until a receipt lands.
3. WASM side inherits native behavior by construction.

### 2.5 Linear memory map

Operator-specified layout for the bounded in-browser engine (wasm32
linear memory; 256 MB tab-level budget per `memory-bounds.md`):

| Region | Base | Role |
|---|---|---|
| Query pool | `0x0` | engine working memory: buffer manager, sorts,
hash aggregates (bounded by `memory_limit`) |
| Arrow region | `0x08000000` | zero-copy Arrow C structures exchanged
across the ABI surface (Section 2.3) |
| Scratch pages | `0x0F000000` | host-VFS range buffers and transient
shim state; every byte copied here is counted by the GATE-V-05
transfer counter |

Standing: layout is PROPOSED/UNVERIFIED — no wasm build exists to
place these regions; the addresses are operator constants until a
linked artifact proves them.

## 3. Subsystem B — ex4pm real-time conformance (OTHER repo)

Specification only; standing in this repo is that of a foreign
subsystem. All mechanisms below are PROPOSED/UNVERIFIED here.

### 3.1 Abstraction function alpha

The conformance engine evaluates an abstraction function `alpha` that
maps a concrete execution trace (OCEL events emitted by Subsystem A)
onto the abstract state machine of the governing specification:

```
alpha : Trace -> AbstractState
```

A trace conforms iff `alpha(trace)` is reachable in the abstract model
from the initial state via permitted transitions. The formal
reachability check is: for each observed event `e` with
`alpha`-projected transition `t = step(alpha(prefix), e)`, conformance
holds iff `t` is an edge of the abstract transition relation; any
non-edge is a typed violation, emitted in real time, not batched.

### 3.2 Ledgers

| Ledger | Contents | Law |
|---|---|---|
| `:atomics` ledger | observed atomic events: completed query
executions, commits, crashes recovered | append-only; an event enters
only on engine completion, never on dispatch (OOMed dispatches do not
count) |
| escape ledger | events that crossed a boundary the abstract model
forbids or could not observe | every `:atomics` entry must be
reconciled; an atomics entry with no model edge lands in the escape
ledger, and escape entries are loud, never dropped |

Conservation: `|atomics| + |escape| = |observed events|`. A gap in that
identity is a conformance-engine defect, not a modeling choice.

## 4. Subsystem C — smalt wire-interception (OTHER repo)

Specification only; foreign subsystem; PROPOSED/UNVERIFIED here.

### 4.1 TLS twin termination

Intercepted TLS sessions terminate into a *twin*: the interceptor holds
the session keys for the mirrored stream only, re-emitting a faithful
plaintext twin of the traffic for [B]'s conformance evaluation. The
twin is read-only with respect to the live session; the interceptor
never injects into the terminated stream. Credential isolation: the
host/interceptor owns all credentials; the analyzed twin carries
redacted secrets.

### 4.2 AMQP link engine

Message-queue surfaces are intercepted at the AMQP link level: per-link
state (attach, transfer flow, disposition) is captured as first-class
events and fed to the same abstraction function as the TLS twins, so
broker traffic conforms under the identical check — no second
conformance calculus.

### 4.3 State fabric

| Fabric role | Mechanism |
|---|---|
| twin registry | session-keyed index of live twins (origin, Alpine
of handshake, expiry) |
| event bus | twins and link events publish onto the fabric; [B]
subscribes; ordering per-source is total |
| replay store | captured twins persisted for offline re-conformance;
replay is byte-exact or the entry is marked corrupt |

### 4.4 PQC conservation equation

Post-quantum accounting law (operator-specified):

```
Delta_S + Phi == R  (mod Sigma)
```

where `Delta_S` is the secret entropy consumed by key establishment in
the observed window, `Phi` is the entropy forwarded into twins (after
redaction), `R` is the residual entropy at rest in the replay store,
and `Sigma` is the modulus of the entropy counter fabric. The identity
is checked per window; a mismatch means entropy was consumed without
being either twinned or stored — an unaccounted channel, and the window
fails conformance.

## 5. Subsystem D — verification substrate: the 11-stage court ladder

Repo-local application: `verification-plan.md` + the GATE-V courts.
Pattern: shim-court (BASELINE sha256 -> BEFORE green -> SHIM marked
`SHIM-COURT` -> AFTER nonzero -> RESTORE byte-exact, fail-closed). A
court that cannot flip is FAIL; a skip is loud, never a faked pass.

Ladder stages (each stage's court must flip before the next claims
standing):

| # | Stage | Court |
|---|---|---|
| 1 | source identity | exact-SHA subject pin; archive/replay |
| 2 | build | compile of the exact subject |
| 3 | unit | real-collaborator tests (Chicago; mocks banned by default) |
| 4 | integration | cross-module real collaborators |
| 5 | boundary ABI | fault each export; panic containment |
| 6 | GATE-V-01 | latency: aggregation p95 < 15 ms over 100k in-memory
event rows (Revision-2 3-query workload; deterministic seed in every
receipt) |
| 7 | GATE-V-02 | crash-chaos: kill/restart under load; recovery
receipt or typed loss |
| 8 | GATE-V-03 | memory ceiling: peak RSS/heap <= 256 MB via CDP poll
(one excursion fails; unset-`memory_limit` control run must FAIL the
court or the court is VACUOUS) |
| 9 | GATE-V-04 | OPFS concurrency: lock races, quota, concurrent page
faults |
| 10 | GATE-V-05 | host-VFS transfer budget: 500 MB remote Parquet,
filtered, transfers <= 5 MB across the boundary; host-side exact
counter, not an estimate |
| 11 | receipt + standing flip | receipt validated against schema;
standing flips only with the receipt |

GATE-V-01..05 spec docs: `gate-v01-latency.md` .. `gate-v05-hostvfs-transfer.md`.
Current standing of every court: UNVERIFIED/PROPOSED until a real
receipt lands in this repo; the gate battery that is green today is
`make verify` (gates + ggen receipt verify), a separate, existing
surface.

## 6. Subsystem D — legal ratchet (operator-specified)

Status: operator doctrine capture. Not measured, not priced, not
drafted into any license in this repo. Per `soc-valuation-model.md`
policy, no dollar figure in this repo is measured; the worked example
below is the operator's specification arithmetic, ASSUMPTION-UNVERIFIED
until the BP4 harness produces a receipt.

### 6.1 Revenue condition precedent

License tiers key on gross-revenue condition precedent. The operator's
verbatim license text is the authority for the condition precedent
clause; the exact verbatim string is NOT present in material available
to this lane — the transcription point is reserved and the clause is
PROPOSED until the operator text lands on disk. This doc will not
fabricate the quote.

### 6.2 SOC damages formula + $5M worked example

Operator-specified formula (coefficients ASSUMPTION-UNVERIFIED):

```
damages = rate_per_SOC_unit * unlicensed_SOC_count
          * aggravation_multiplier
```

Worked example (operator arithmetic, not a measurement): an enterprise
tier found operating above its revenue condition precedent with
`unlicensed_SOC_count = 1,000,000` displaced client-side queries,
`rate_per_SOC_unit = $4.00`, `aggravation_multiplier = 1.25`:

```
damages = 4.00 * 1,000,000 * 1.25 = $5,000,000
```

### 6.3 ABA matrix

Operator-specified adherence matrix mapping the ABA model clauses to
the platform surfaces they govern:

| ABA clause area | Governed surface | Enforcing artifact |
|---|---|---|
| license scope / field of use | revenue tier vs deployed tier |
revenue condition precedent (6.1) |
| usage metering | SOC counters (client-observed query executions +
scans, OCEL-native) | `soc-valuation-model.md` measurement basis |
| audit rights | twin/replay store + OCEL logs | GATE-V courts,
receipt schema |
| damages / remedies | SOC damages formula (6.2) | sealed arbitration
award |

### 6.4 Sealed JAMS + residuals doctrine

Disputes resolve under sealed JAMS arbitration: no public filings, the
award and the underlying SOC ledgers are sealed, and the existence of
the dispute is itself confidential. Residuals doctrine: knowledge
residing in the unaided memory of personnel is not a licensed
artifact; only the metered artifacts (SOC stream, twins, replay store)
are subject to the license and the damages formula. Unmetered residuals
are outside the ratchet by design.

## 7. Master traceability matrix

| Spec element | Section | Repo artifact | Court | Standing in this repo |
|---|---|---|---|---|
| dual-target kernel [A] | 2 | `crates/frozen-duckdb*`,
`wasm-opfs-architecture.md` | V-W1 | PARTIAL_ALIVE native / PROPOSED wasm |
| host-VFS range delegation | 2.1 | `host-range-vfs-contract.md`,
`host-call-vfs.md` | GATE-V-05 | PROPOSED (contract only) |
| 80/20 extension table | 2.2 | `extension-matrix-8020.md` | build
gate | PARTIAL_ALIVE (httpfs trim open; NATIVE pins 9 extensions) |
| unboxed C ABI | 2.3 | `boundary-abi-contract.md` | boundary court |
PARTIAL_ALIVE native (`duckdb_*`) / PROPOSED `fdb_*` |
| VARIANT pipeline | 2.4 | `variant-shredding-design.md` | `cargo test
-p frozen-duckdb --test variant_tests` | PARTIAL_ALIVE (native landed;
shredded reads UNVERIFIED) |
| linear memory map | 2.5 | `memory-bounds.md`,
`wasm-opfs-architecture.md` | GATE-V-03 | PROPOSED |
| ex4pm alpha + ledgers | 3 | foreign repo (ex4pm) | [B] conformance
engine | UNSUPPORTED here (foreign subsystem, spec capture only) |
| smalt TLS twins / AMQP / PQC | 4 | foreign repo (smalt) | [C]
interceptor courts | UNSUPPORTED here (foreign subsystem) |
| 11-stage court ladder | 5 | `verification-plan.md`, `gate-v0*.md` |
shim-court protocol | PROPOSED (V-W*/V-S* inventory; none flipped) |
| GATE-V-01 | 5 | `gate-v01-latency.md` | latency court | PROPOSED |
| GATE-V-02 | 5 | `gate-v02-crash-chaos.md` | crash-chaos court | PROPOSED |
| GATE-V-03 | 5 | `gate-v03-memory-ceiling.md`, `memory-bounds.md` |
CDP RSS court | PROPOSED |
| GATE-V-04 | 5 | `gate-v04-opfs-concurrency.md` | OPFS court | PROPOSED |
| GATE-V-05 | 5 | `gate-v05-hostvfs-transfer.md` | transfer-budget
court | PROPOSED |
| revenue condition precedent | 6.1 | (reserved — operator verbatim
text pending) | none | PROPOSED |
| SOC damages formula | 6.2 | `soc-valuation-model.md` | BP4 harness |
ASSUMPTION-UNVERIFIED |
| ABA matrix | 6.3 | this doc + `soc-valuation-model.md` | BP5 license
draft (not on disk) | PROPOSED |
| sealed JAMS + residuals | 6.4 | this doc | none | PROPOSED |

Falsifier for this document as a whole: any row above claiming ALIVE
without a court receipt on the exact subject in this repo. Standing
flips happen only through `scripts/check_sjira_standing.sh`-gated
receipts, never through edits to this spec.

## See Also

`prd-ard-v2.md` · `requirements-traceability.md` · `verification-plan.md`
· `host-range-vfs-contract.md` · `boundary-abi-contract.md` ·
`extension-matrix-8020.md` · `soc-valuation-model.md` ·
`docs/sjira/v26.10.8/_LANES.md`
