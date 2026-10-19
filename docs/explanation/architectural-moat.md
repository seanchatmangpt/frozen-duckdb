# Architectural Moat

Why the builder-freeze + local-first architecture is defensible,
read through five strategy lenses. This is strategic analysis, not
measurement: strategy claims below are labeled **ARGUMENT**. Every
* factual claim about the repo (numbers, file paths, court results) is
receipt-tied. The mechanism-level detail of the freeze lives in
`docs/design/freeze-moat.md`; this doc is the why-layer above it.

## The position, in one paragraph

frozen-duckdb removes the C++ compiler from every consumer's critical
path: the expensive manufacturing step (the 30-90 minute DuckDB
compile) runs once on the publisher side, and consumers link a frozen
mega-library from a content-addressed cache — cache hit, then release
download, local compile as last-resort fallback
(`crates/frozen-duckdb-builder/src/lib.rs`, `ensure_binary`). Paired
with the sovereign/local-first surface (engine ships to the customer's
silicon; data lands once), the result is an analytics engine whose
cost curve is decoupled from per-query metering. **ARGUMENT**: that
pairing is a moat, not a feature list. The lenses below say why.

## Lens 1 — The Innovator's Dilemma

Christensen's pattern: incumbents rationally ignore disruptive
offerings because the offering is *worse* on the incumbent's metric
and better on one customers don't yet pay for. **ARGUMENT**: this
position is disruptive in exactly that shape. On the incumbent's
metric — petabyte joins, concurrency, RBAC/lineage/SOC2 — the
sovereign engine is deliberately worse (single-writer engine, OPFS
quota caps, 256MB-class ceilings, no warehouse ecosystem). On the
underserved metric — $0 marginal cost per exploratory query, offline
execution, data never leaving the device — it is structurally better,
because the incumbent's revenue model *is* the meter. **ARGUMENT**: a
warehouse vendor is argued to be unable to follow without pricing its
core product at zero, and a warehouse is argued to be unable to ship
inside an application binary. The economic
model doc draws the same line ("not a Snowflake killer — a
workload-class extractor").

## Lens 2 — Blue Ocean

**ARGUMENT**: the uncontested space is the exploratory analytics loop
on already-owned silicon: cohort retention, funnels, process mining,
local ML at OPFS-fitting data sizes. The measured, receipted facts
underneath the strategy: p95 10.5088 ms over 1,000 three-query
iterations (real engine), 12.16M rows/sec release throughput,
0 bytes egress / $0.00 invoice across five workload classes (scorecard
5x CLEAR, cold-dominated-sim caveat per
`sovereign-economics.md`). The dollar *value* of that space is
ASSUMPTION-UNVERIFIED pending the two-arm harness — the ocean is
argued, not yet audited.

## Lens 3 — Conway's Law

Conway: system structure mirrors the communicating organization's
structure. **ARGUMENT**: the freeze inverts the embedded-DuckDB
communication pattern. The old shape forces every consumer
organization to maintain C++17/cmake competence — toolchain structure
replicated N times downstream. The freeze concentrates the compile in
one publisher workflow (`.github/workflows/build-binaries.yml`, the
only surface needing cmake) so downstream organizations communicate
with the artifact, not the toolchain. **ARGUMENT**: a moat that
requires no downstream reorganization to adopt is argued to be one
incumbents cannot price against: the consumer's switching cost is a
one-line dependency change. The wasm tier pushes this further — collapsing the
(version, arch) matrix to (version), one artifact across macOS arm64,
x86_64, and Linux.

## Lens 4 — Little's Law

Little's Law: L = lambda x W; throughput, concurrency, and latency are
one constraint, not three. **ARGUMENT**: the sovereign position trades
concurrency (a warehouse's strength: thousands of parallel tenants) for
latency and per-user throughput (p95 ~10.5ms with zero queueing — the
"queue" is one local process on an idle device). Because end-user
silicon sits idle most of the day, the fleet's aggregate lambda scales
with users, not with a shared compute pool — capacity arrives
pre-purchased with each device. This is the queuing-theory substrate
under the subsidized-silicon economics: the vendor sells bytes; the
queue lives on the customer's clock.

## Lens 5 — Gall + Chesterton

Gall: a complex system that works is found to have evolved from a
simple system that worked. Chesterton: do not remove a fence until you
know why it was put up. The campaign honored both, and the honoring is
itself moat evidence:

- The freeze evolved from a working simple core (builder sub-crate,
  cache-first resolution) rather than being redesigned in; the stale-
  cache guard (`lib.rs` ~line 101) fences the one failure mode of the
  cache hit — a wrong binary passing as instant — and this window's
  cache-poison incident (v1.5.5 bytes staged into the v2.0.0-dev slot,
  quarantined, rebuilt from pin `b6013f7`, identity re-witnessed via
  strings probe `v2.0.0-dev88595`) proved the fence at production
  scale. The fence was not removed because the poison was annoying; it
  was hardened.
- Fail-closed gates were kept even when they blocked progress: doctor
  stages exit 3 loudly rather than passing economics legs whose court
  runner is absent; compile aborts are never read as failure counts;
  the X4 "engine defect" claim was withdrawn to the fixture layer
  after re-derivation rather than papered over. **ARGUMENT**: an
  architecture whose every capability claim carries a fail-closed
  court is harder to copy than one whose claims are marketing,
  because the court battery is accumulated organizational capital
  (Gall's evolved system), not a spec someone can re-implement in a
  sprint.

## What would falsify the moat

Consistent with repo law, the strategy carries falsifiers:

- A warehouse or browser vendor shipping per-query-zero exploratory
  analytics inside app binaries at OPFS-fitting sizes (Lens 1/2
  refuted for that envelope).
- A consumer cold-start that spawns a compiler (the cache-path court
  in `freeze-moat.md` — the zero-compilation claim regresses).
- The two-arm harness refuting edge-loop usability or OPFS quota
  sufficiency at the reference dataset (the subsidized-silicon
  economics, per `economic-model.md`).

## See Also

`docs/design/freeze-moat.md` (mechanism-level moat doc) -
`docs/design/economic-model.md` (thesis + falsifier) -
`docs/explanation/sovereign-economics.md` (the economics) -
`docs/sjira/v2.0.0-dev/CAMPAIGN-RECEIPT.md` (receipts cited)
