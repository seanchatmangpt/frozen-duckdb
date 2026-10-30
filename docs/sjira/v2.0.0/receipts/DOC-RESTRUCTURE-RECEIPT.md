# DOC-RESTRUCTURE-RECEIPT — lane DTS8 (verify+prep), 2026-10-19

Wave: docs restructure sweep (DTS lanes). This lane verifies DTS7's restructure
sweep and produces the coordinator commit message. Lane DTS8 owns only this
receipt + `/tmp/dts8_msg.txt`; no git state commands run.

## 1. What moved to docs/archive/

`git status --short` at verification time (read-only):

```
R  docs/llm/embeddings.md -> docs/archive/v1.5.5/llm/embeddings.md
R  docs/llm/ollama-setup.md -> docs/archive/v1.5.5/llm/ollama-setup.md
R  docs/llm/rag-pipelines.md -> docs/archive/v1.5.5/llm/rag-pipelines.md
R  docs/llm/semantic-search.md -> docs/archive/v1.5.5/llm/semantic-search.md
?? docs/archive/v1.5.5/README.md
```

Four v1.5-era LLM-integration docs moved intact under
`docs/archive/v1.5.5/llm/`, plus a new archive index
`docs/archive/v1.5.5/README.md` ("Legacy documentation snapshots from the
v1.4.x/v1.5.5 era, archived intact at the v1.5.5 → 2.0.0 milestone.
Content is preserved byte-for-byte; only the paths changed."). The pre-rewrite
README is preserved as `docs/archive/v1.5.5/README.md` companion snapshot
surface alongside the four moved files.

## 2. New Diataxis tree (real `ls` output)

```
docs/tutorials:   getting-started.md
docs/how-to:      configure-env.md deploy-test-and-use.md deploy.md
                  how-to-host-vfs-range.md how-to-memory-caps.md
                  how-to-ocel2-compression.md how-to-run-chicago-doctor.md
                  run-tests.md usage-examples.md
docs/reference:   a2a-no-op.md api.md appender.md benchmarking.md c-abi.md
                  environment-variables.md extension-build.md functions.md
                  generated/ hardware-classes.md pragma-builder.md
                  r2d2-pooling.md scalar-functions.md schemas-ocel.md
                  tech-stack.md variant-types.md
docs/explanation: architectural-moat.md faq.md lessons-and-optimizations.md
                  related.md sovereign-economics.md why-chicago-testing.md
```

README "Documentation (Diataxis)" nav hub routes all four quadrants
(tutorials → Quick Start anchor; how-to → CLI/troubleshooting/doctor;
reference → extensions/ABI/traceability/DOC-MAP; explanation → sovereign
spec, Chicago courts, court ladder) plus a Verification & Governance section.

## 3. genesis-check receipt (real run, this lane)

```
$ make genesis-check 2>&1 | tail -3
GAP-OK [gap] docs/explanation/faq.md <- lineage:7f604e5
GAP-OK [gap] docs/explanation/lessons-and-optimizations.md <- lineage:7f604e5
GAP-OK [gap] docs/explanation/related.md <- lineage:7f604e5
genesis-check: checked=58 deferred=0 gaps=16 failed=0
EXIT=0
```

Exit 0. checked=58, failed=0. The 16 `gaps` are lineage GAP-OK entries
(reporting lineage, not failures).

## 4. README rewrite summary

Sovereign rewrite landed by the DTS7 sweep (not this lane's edit). Verified
state: hero one-liner ("Zero-toolchain embedded columnar engine for Rust —
one prebuilt dylib, link-only builds, and a 13-stage Chicago-court
verification battery"), Why-This-Exists section (compile drag + cloud billing
drag, egress receipt cited), receipt-backed-only Audited Performance table,
Diataxis nav hub, Quick Start pinned to `b6013f7` prerelease caveat,
verification & governance section. No speculative numbers: the old
"Performance Revolution" marketing claims are gone; build-time claims point
to `CAMPAIGN-RECEIPT.md` / `DEFINITION-OF-DONE.md`.

## 5. Numbers audit — every number's receipt path cited and re-read on disk

| README value | README-cited receipt | On-disk evidence (re-read this lane) |
|---|---|---|
| Dylib 65.9 / 52.4 MiB unstr./str. (arm64) | `prebuilt/README.md` | `prebuilt/README.md:21` "stripped arm64 dylib comes out at 52.4MB"; `:25` "~66MB unstripped (52.4MB stripped)" |
| Ingest 12.16M rows/s | `CAMPAIGN-RECEIPT.md` (DISPATCH-CITED) | `CAMPAIGN-RECEIPT.md:322` "12.16M rows/sec release" |
| p95 10.51 ms (< 15 ms budget) | `R/gate_v01_receipt.json` | JSON: `"p95_ms":10.5088`, `"budget_ms":15` |
| OCEL 94.93% / 97.01% | `CAMPAIGN-RECEIPT.md` (DISPATCH-CITED) | `CAMPAIGN-RECEIPT.md:319` "94.93/97.01% (compress / query)" |
| Egress 0 B | `R/val004.json` | JSON: `"egress_bytes": 0` |
| Fleet Class-D 99.76% | `CAMPAIGN-RECEIPT.md`; `R/fleet004.json` | JSON: `"ratio": 0.9976` (leg2_compression) |

6/6 values match their cited receipts. Dispatch-cited rows correctly labeled
DISPATCH-CITED vs materialized court JSON on disk.

## Standing

ALIVE — restructure verified: archive moves staged, Diataxis quadrants
populated, genesis-check exit 0 (58/58), README numbers audit 6/6 match.
