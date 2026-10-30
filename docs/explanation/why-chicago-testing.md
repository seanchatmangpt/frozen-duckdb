# Why Chicago Testing

Why this repo bans mocks by default and tests against real
collaborators. This is an understanding-oriented doc: it explains the
discipline using this campaign's own evidence, including the mistakes
the discipline caught. The rule itself lives in the operator's testing
doctrine ("Chicago not London — real collaborators, mocking banned by
default"); the repo's court/gate surface enforces it mechanically.

## The divide

**London (mockist)**: replace every collaborator with a test double,
assert on interactions. The test verifies its own model of the system —
it can pass while the integration is broken, precisely because every
double was built to agree.

**Chicago (classicist)**: use real collaborators wherever practical;
assert on final state. A hand-written real implementation of an
interface is not a mock. The one permitted exception — a genuinely
infeasible collaborator — requires a stated reason, a real degraded
alternative first, and a simple fake, never a mock. Skips are loud
(`[SKIP] <reason>`), never faked passes.

The failure mode the Chicago discipline targets is not aesthetic. A
mock converts "the test ran" into "the system works" — that conversion
is exactly what this campaign kept catching.

## This campaign's evidence

### The cache-poison catch

Mid-window, v1.5.5 library bytes were staged into the v2.0.0 cache
slot by an unidentified lane build path (~13:22). Everything that had
"passed" against that slot was void for engine identity — a test suite
run against the wrong binary is a mock-adjacent error: the collaborator
under test was not the collaborator claimed. The campaign's response
was physical, not procedural: quarantine, rebuild from the pinned
`v2.0-cyanoptera b6013f7` source, and a strings-level identity probe
(`v2.0.0` self-report witnessed; stale `v2.1.0-dev89402`
self-report absent). Post-13:22 receipts were re-witnessed or remain
marked as such — never silently upgraded. (Record:
`docs/sjira/v2.0.0/CAMPAIGN-RECEIPT.md`, incident record.)

### The vacuity flips

A gate that cannot fail is vacuous — it certifies nothing. The campaign
runs anti-vacuity mutations to prove courts refuse when they should:

- The disruption scorecard's non-vacuity selftest
  (`scorecard.ts --selftest`, exit 3) doctored a sovereign leg and
  proved the assertion machinery refuses it.
- The crash-chaos court's no-op-kill flip was DETECTED — the court
  distinguishes a real `kill -9` leg (2 legs green with 200,000 and
  260,000 events in-flight) from a no-op that would pass trivially.
- The concurrency court (2/2 green) recorded a vacuity catch of its
  own; the OCEL stage carries a SHIM flip alongside its 94.93/97.01%
  compression/query ratios.
- The G6 receipt-chain court at
  `warehouse-sim/economics/court.ts:117` fails closed on the
  missing-link path — and the authorized fix was constrained to keep
  it fail-closed: "a repair that converts a genuine chain gap into a
  pass is a vacuous-gate defect, not a fix."
- Compile aborts are never read as failure counts: fleet stage 12's
  census-vs-compile-abort rule reported cargo rc=101 legs as compile
  aborts with the owning file, not as "0 tests, 0 failures."

### The X4 fixture false-positive and its withdrawal

Lane X4 reported a "WAL-replay atomicity defect" in the engine. It was
an artifact of E2's broken fixture script — the fixture, not the
engine. The correction record withdraws the engine-defect claim to the
fixture layer, and re-facts were witnessed only after: Z2's court green
on 3 legs, GATE-V-02 native leg PARTIAL_ALIVE green. This is the
discipline working in both directions: a false *accusation* against a
real collaborator is caught the same way a false *exoneration* by a
mock would be — by re-deriving against physical state, not by
re-asserting.

### Why this matters economically

Every one of these incidents is a case where a cheaper testing posture
would have produced a durable falsehood: a test suite green against
the wrong binary, a gate that always passes, a defect claim resting on
a broken fixture. The campaign's standing vocabulary (ALIVE /
PARTIAL_ALIVE / PASS / SUSPECT / BLOCKED, plus ASSUMPTION-UNVERIFIED
for dollar figures) exists so that a claim's evidentiary register is
carried with the claim — DISPATCH-CITED is never silently promoted to
WITNESSED.

## See Also

`~/.claude/rules/testing-chicago-style.md` (the rule) -
`docs/design/verification-plan.md` (court battery) -
`docs/design/economic-model.md` (fail-closed economics, same
discipline applied to dollars) -
`docs/sjira/v2.0.0/CAMPAIGN-RECEIPT.md` (the incidents cited)
