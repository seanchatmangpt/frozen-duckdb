# LICENSE-ENTERPRISE-DRAFT

## DESIGN DRAFT — NOT LEGAL ADVICE; requires review by qualified counsel before any adoption

Status: DESIGN DRAFT (lane BP5). Every legal-effect claim in this document is
UNKNOWN-PENDING-COUNSEL. Nothing here is an offer, a license, or a binding term.
Compliance-design references are to the ABA Model Rules of Professional Conduct
(cited by rule number only, no verbatim reproduction).

## 1. Purpose and Scope

Design outline for a future Enterprise Commercial Agreement (ECA) tier of the
frozen-duckdb project. Goals:

- Keep the permissive/open tier viable for individuals, research, and
  organizations below the revenue threshold.
- Introduce a revenue-gated Condition Precedent so large commercial users
  fund continued development.
- Define a non-traditional valuation concept (SOC) as a damages framework,
  flagged as untested.
- Specify dispute resolution via sealed JAMS single-arbitrator arbitration
  with a bilateral NDA.

## 2. Definitions (design terms, not operative language)

- "Organization": any corporation, LLC, partnership, government body,
  nonprofit, or other legal entity deploying the Software.
- "Gross Revenue": worldwide consolidated gross receipts of an Organization
  and its Affiliates for the most recent completed fiscal year, measured
  under U.S. GAAP or IFRS as consistently applied. UNKNOWN-PENDING-COUNSEL:
  consolidated-vs-standalone measurement, currency conversion date, and
  treatment of pending fiscal years all need counsel.
- "Threshold": USD $50,000,000 in Gross Revenue.
- "Software": the frozen-duckdb crates, their vendored headers, and
  associated build scripts, excluding DuckDB core itself (separately
  licensed upstream).
- "SOC" (Synthetic Operational Capacity): a proposed valuation metric for
  compute-time-not-spent, defined in Section 5.

## 3. License Grant Scope (design)

- Open tier: perpetual, worldwide, royalty-free license for use,
  modification, and redistribution under the existing open terms, available
  to any user whose Organization Gross Revenue is at or below the Threshold.
- Enterprise tier: same scope plus indemnity, support, and warranty terms,
  available only under an executed ECA.
- UNKNOWN-PENDING-COUNSEL: whether a revenue-gated grant conditioned on
  self-reported revenue is enforceable against an Organization that
  crosses the Threshold mid-term, and what cure/grandfathering applies.

## 4. Condition Precedent: Revenue Gate

Mechanics (design intent):

1. At first download or first production deployment by an Organization,
   the Organization represents its Gross Revenue relative to the Threshold.
2. If Gross Revenue exceeds the Threshold, formation of an ECA is a
   Condition Precedent to continued use beyond a 30-day evaluation period.
3. Self-reporting is honor-based at gate time; the ECA contemplates audit
   rights (records inspection on reasonable notice) as the verification
   mechanism.
4. Crossing the Threshold mid-year triggers a prospective obligation, not
   a retroactive fee, for the balance of the then-current term.

Enforceability of a self-reported Condition Precedent is
UNKNOWN-PENDING-COUNSEL, as is the characterization of use past the gate
(breach vs. unlicensed use vs. infringement).

## 5. Synthetic Operational Capacity (SOC) Valuation

Design concept: where the Software saves compute cost, damages or fees may
be valued by capacity avoided rather than by license-fee equivalence.

- SOC(unit) = (baseline wall-clock compute cost of an equivalent
  non-frozen toolchain) minus (actual compute cost with the Software),
  priced at prevailing cloud on-demand rates for equivalent instance
  hours.
- Aggregation: SOC accrues per deployment and is summed over a measurement
  period agreed in the ECA.
- Use cases (design only): (a) Enterprise fee floor justification;
  (b) liquidated-damages baseline for unlicensed use; (c) cost-sharing in
  multi-party support agreements.

UNKNOWN-PENDING-COUNSEL: whether SOC is a valid measure of damages, whether
it can operate as a liquidated-damages provision (reasonableness-at-drafting
and difficulty-of-estimation requirements), and whether it survives as a
penalty challenge. No jurisdictional precedent has been validated.

## 6. Dispute Resolution

Design intent:

- All disputes under the ECA go to a single arbitrator administered by JAMS
  under its Comprehensive Arbitration Rules and Procedures.
- The arbitrator is a single neutral jointly appointed by the parties from
  JAMS' panel; failing agreement, appointed per JAMS default strike-and-
  rank procedure.
- The arbitration is confidential and sealed: filings, evidence, transcripts,
  and the award are non-public, subject to the arbitrator's power to disclose
  as required by law.
- A bilateral NDA (mutual confidentiality agreement) is executed at ECA
  signing and survives termination; it covers the existence and content of
  the arbitration, SOC calculations, and any exchanged business terms.
- Seat/venue: to be fixed by counsel; the drafting assumption is a single
  agreed seat to avoid intra-U.S. vs. international regime splits.
- UNKNOWN-PENDING-COUNSEL: enforceability of secrecy of the award itself,
  carve-outs needed for enforcement proceedings, and required disclosures
  (e.g., to auditors or regulators).

## 7. Compliance-Design Considerations (ABA Model Rules, cited by number)

This draft is written by a non-lawyer as software design. The following
Model Rules are design constraints on the drafting process, not recitals:

- Rule 1.2(d): the document must not be drafted or deployed so as to
  further conduct the drafting party knows is criminal or fraudulent —
  e.g., no audit mechanism designed to entrap below-Threshold users.
- Rule 3.1: claims advanced under this framework (including any SOC-based
  demand) must have a nonfrivolous basis; a SOC theory with no supporting
  authority should not be pleaded.
- Rule 3.3: no false statements of fact to a tribunal; SOC methodology must
  be disclosed honestly, including its novel/untested nature.
- Rule 4.1: no false statements of material fact to third persons in
  negotiation — revenue representations and SOC figures must not be
  misrepresented.
- Rule 8.4: misconduct constraints apply to the drafting and enforcement
  posture generally, including any use of the revenue gate as a harassment
  lever against competitors.

## 8. Self-Executing Ethical Tripwires — Design Aspiration Only

The blueprint contemplates "self-executing ethical tripwires": license terms
or technical gates that automatically restrict use in ways that keep the
drafting party and its counsel inside the bounds above without case-by-case
judgment. This document treats that as a DESIGN ASPIRATION ONLY. Whether any
such tripwire is enforceable as contract, whether an automated gate's
decisions can bind a court or arbitrator, and whether automation satisfies
or undermines the compliance duties in Section 7 are all LEGAL UNKNOWN. No
statement in this document asserts enforceability of any tripwire.

## 9. Open Items for Counsel

1. Choice of governing law and whether the revenue Condition Precedent is
   enforceable in that law.
2. Whether SOC is viable as a damages theory or only as pricing logic.
3. Audit-right scope consistent with Rule 1.2(d) constraints.
4. Arbitration seat, confidentiality carve-outs, and award-secrecy limits.
5. Whether the open-tier/enterprise-tier split interacts with upstream
   DuckDB licensing obligations.
6. Grandfathering and cure mechanics for mid-year Threshold crossings.

Each open item resolves a current UNKNOWN-PENDING-COUNSEL flag; none is
settled by this draft.

## Revision 2 — Adversarial Ratchet (v26.10.19 spec)

Appended by lane X12 from the operator's Section 6 specification. Every
legal-effect claim below remains governed by the header of this document:
DESIGN DRAFT, NOT LEGAL ADVICE, UNKNOWN-PENDING-COUNSEL. Citation of a
statute or rule is a design reference, not a claim that the cited authority
supports the mechanism.

### R2.1 Revenue Condition Precedent — 'Covered Entity' clause

Operator's clause, transcribed as quoted design language (operative effect
UNKNOWN-PENDING-COUNSEL):

- Covered Entity: any Organization whose Gross Revenue exceeds the
  Threshold, together with each Affiliate that deploys the Software. For
  such an entity, execution of an ECA is a Condition Precedent to any use
  beyond the evaluation window. Use of the Software by a Covered Entity
  without an executed ECA is not licensed use; the drafting design treats
  such use as unauthorized and reserves all remedies available under
  applicable law, including (by reference, not by assertion of effect)
  exclusive rights of the copyright holder under 17 U.S.C. § 106 and
  protections against misappropriation of trade secrets under
  18 U.S.C. § 1836, in each case only to the extent the underlying subject
  matter actually qualifies for those protections — a characterization
  that is UNKNOWN-PENDING-COUNSEL (breach vs. unlicensed use vs.
  infringement of an open-licensed work whose condition failed).

### R2.2 SOC damages valuation

Valuation framing (design, untested):

- Statutory reference points, cited as design anchors only:
  35 U.S.C. § 284 (adequate-damages framing for patent; applicability to
  this subject matter is UNKNOWN-PENDING-COUNSEL) and 17 U.S.C. § 504(b)
  (actual damages and profits of the infringer; same caveat).
- Method: damages are framed per the Georgia-Pacific factors as a
  reasonable-royalty analogue, with SOC substituted for the royalty base.
- SOC = Σ_k Executions_k × BaselineCloudUnitCost_k: per execution class k,
  the number of executions in the measurement period times the baseline
  cloud unit cost of the equivalent non-frozen execution, summed over all
  classes.
- Worked example (operator's illustration, marked
  ILLUSTRATIVE-ASSUMPTION): 50,000,000 executions at a $0.10 baseline
  cloud unit cost yields SOC = $5,000,000. The $0.10 unit cost and the
  execution count are both assumptions of the illustration; no measured
  figure from this repository supports them.

UNKNOWN-PENDING-COUNSEL: whether SOC as so defined satisfies
reasonableness-at-drafting and difficulty-of-estimation for a liquidated-
damages clause, or survives a penalty challenge; no jurisdictional
precedent validated for this formula.

### R2.3 ABA enforcement matrix (operator's design)

The operator specifies that enforcement conduct under the framework be
gated by an explicit matrix mapping enforcement actions to the Model Rules
they must not violate. Transcribed as design (rule numbers only, no
verbatim rule text):

| Enforcement action | Governing rules (design constraint) | Class label |
|---|---|---|
| Deploying the revenue gate or audit mechanism to entrap or | 1.2(d), 8.4(c) | rogue-dev |
|   harass below-Threshold users | | |
| Concealing revenue representations or audit findings during | 4.1; ASC 450 | audit-concealment |
|   negotiation or in records | | |
| Advancing SOC-based claims or suits without nonfrivolous basis, | 3.1, 4.4(a); | predatory-suits |
|   or as cost-harassment (28 U.S.C. § 1927 exposure noted as a | 28 U.S.C. § 1927 | |
|   design consequence, not asserted law) | | |

Rule 4.4(a) is cited as a constraint on using methods of obtaining
evidence that violate the legal rights of a person; ASC 450 is cited as
the accounting-control frame for revenue-recording honesty. Both are
design anchors; effect is UNKNOWN-PENDING-COUNSEL.

### R2.4 Dispute resolution (sealed)

- All disputes under the ECA go to a single arbitrator administered by
  JAMS under its Comprehensive Arbitration Rules and Procedures.
- The proceedings, filings, evidence, transcripts, and award are sealed
  and confidential; an evidentiary gate governs what may enter the record
  (design: SOC methodology disclosures must be complete and honest per
  Rule 3.3; no sealed record may be used to conceal material facts under
  Rule 4.1).
- Fee-shifting: the prevailing party recovers 100% of reasonable fees and
  costs from the non-prevailing party, per the operator's specification.
  Enforceability of full fee-shifting in arbitration is
  UNKNOWN-PENDING-COUNSEL.
- Doctrine of Residuals: rights, capabilities, and know-how not expressly
  transferred are reserved to each party, and the framework is designed
  to preserve each party's freedom to compete using residual capabilities.
  Characterization of this reservation against trade-secret law
  (18 U.S.C. § 1836 reference carried from R2.1) is
  UNKNOWN-PENDING-COUNSEL.

### R2.5 Revision-2 open items

1. Whether the 'Covered Entity' clause's remedy reservation survives the
   open tier's license grant (R2.1 UNKNOWN).
2. Whether the SOC formula is a damages measure, a fee floor, or neither
   (R2.2 UNKNOWN).
3. Whether the enforcement matrix binds enforcement counsel or is
   aspirational only (R2.3 UNKNOWN; cf. Section 8's tripwire caveat).
4. Whether sealed arbitration plus full fee-shifting chills nonfrivolous
   claims or unenforceably restricts access to remedy (R2.4 UNKNOWN).

No item above is settled by this revision; each remains governed by the
document header.
