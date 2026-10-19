# HARNESS-UPGRADE-WO — frozen-duckdb — v26.10.8

| field | value |
|---|---|
| repo | `/Users/sac/frozen-duckdb` (canonical checkout) |
| work order | Upgrade repo harness config to smalt patterns |
| pattern source | `/Users/sac/smalt` equivalents |
| lane | W10 (writer), per `_LANES.md` (same dir, 20 lanes: W1–W10 writers, V1–V6 verifiers, A1–A4 auditors) |
| standing | **IN_PROGRESS** |
| date opened | 2026-10-19 |

## Scope

One canonical checkout, no worktrees. Lane partition and git ownership per
`docs/sjira/v26.10.8/_LANES.md`: lanes never run git state-changing commands; the
coordinator owns all transitions and commits. This document is lane W10's sole write
surface; no other file is modified by this lane.

## Deliverables (file ownership per _LANES.md)

| deliverable | owner lane | pattern source (smalt) |
|---|---|---|
| `CLAUDE.md` | W9 | smalt CLAUDE.md |
| `.claude/settings.json` | W1 | smalt `.claude/settings.json` |
| `.claude/hooks/ban_generic_agents.py` | W1 | smalt `.claude/hooks/ban_generic_agents.py` |
| `.claude/agents/bench-runner.md` | W2 | smalt `.claude/agents/bench-runner.md` |
| `.claude/agents/gate-author.md` | W3 | smalt `.claude/agents/gate-author.md` |
| `.claude/agents/milestone-doc.md` | W4 | smalt `.claude/agents/milestone-doc.md` |
| `.claude/agents/release-op.md` | W5 | smalt `.claude/agents/release-op.md` |
| `.claude/skills/fd-lane/SKILL.md` | W6 | smalt lane-dispatch skill |
| `.claude/skills/fd-court/SKILL.md` | W7 | smalt court skill |
| `.claude/skills/fd-anneal/SKILL.md` | W8 | smalt annealing skill |
| this document | W10 | — |

Shared seams: none this wave (all files new, `CLAUDE.md` owned solely by W9).

## Falsifier

The work order fails if either:

1. Any path or command cited in the new harness config does not exist on disk at
   the canonical checkout (or in the cited smalt pattern source).
2. Any verifier receipt (V1–V6) is missing real command output — a receipt that
   asserts a result without the command and its actual exit/output is not a receipt.

A verifier receipt with fabricated or missing output is itself a falsifier trigger,
per the no-sleep / Chicago-school discipline: observe, do not assert.

## Standing vocabulary

- **IN_PROGRESS** (current): deliverables not yet all landed and verifier-receipted.
- ALIVE requires: every deliverable on disk, every V-lane receipt carrying real
  command + exit + output, and zero falsifier triggers across A1–A4 findings.

## Execution boundary

Nothing in this document asserts execution. Commands, exits, and digests belong in
verifier receipts (V1–V6) and auditor findings (A1–A4), not here. This work order
records scope, ownership, and the falsifier only.
