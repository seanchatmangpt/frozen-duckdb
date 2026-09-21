# RUNBOOK — v26.9.21 dispatch contract (_RUNBOOK.md = canonical dispatch prompt form)

## Dispatch form (per agent)

1. Ticket path (this directory, e.g. docs/sjira/v26.9.21/T2A.md)
2. Worktree path (inside the ticket; worktree pre-created by coordinator on the ticket's branch)
3. Nothing else. Session state lives in the ticket's History table, never in conversation.

## Laws in force

- 流 loop: parse → orient (this runbook + ticket) → 階 (reuse/compose/extend before inventing) → edit → verify → 証 → stop.
- 帳: any hand-written line outside a generator/rendered path on 産面 needs an UNSUPPORTED row in the ticket History.
- 器: generated/consequence files are edited at their source, never as projections.
- 並: one agent per worktree; shared mutable trees are serialized by the coordinator, never by agents.
- 証: ALIVE requires observed execution this session against the exact subject. Preserve command + exit. No acceptance mocks.
- 偽: attempt your own falsifiers before claiming DoD; a gate you skipped is UNKNOWN, not green.
- 権: no push, no publish, no tag, no release, no issue/PR mutation. Coordinator merges; operator cuts remain in MILESTONE.md.
- 延: any operating knowledge an agent uses must land in the repo (ticket, comment, or gate), not stay in the session.

## Coordinator merge order (serialized, --no-ff, mix of gates before each)

T5 (ci-parity) → T6 (scripts-fix) → T4 (docs-final) → T2A → T2B → T3 (examples) → T7 (verify) → T8 (release) — order may be re-sequenced by conflict evidence; conflicts resolve toward the branch whose scope owns the file (scope table in each ticket).
