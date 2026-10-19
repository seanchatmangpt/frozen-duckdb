# Lane partition — harness upgrade to smalt patterns (2026-10-19)

Canonical checkout: /Users/sac/frozen-duckdb. One wave, 20 lanes, disjoint file ownership.
Coordinator owns all git transitions and commits. Lanes never run git state-changing commands.

| lane | type | owns (write) |
|---|---|---|
| W1 | writer | .claude/settings.json, .claude/hooks/ban_generic_agents.py |
| W2 | writer | .claude/agents/bench-runner.md |
| W3 | writer | .claude/agents/gate-author.md |
| W4 | writer | .claude/agents/milestone-doc.md |
| W5 | writer | .claude/agents/release-op.md |
| W6 | writer | .claude/skills/fd-lane/SKILL.md |
| W7 | writer | .claude/skills/fd-court/SKILL.md |
| W8 | writer | .claude/skills/fd-anneal/SKILL.md |
| W9 | writer | CLAUDE.md |
| W10 | writer | docs/sjira/v26.10.8/HARNESS-UPGRADE-WO.md |
| V1–V6 | verify | read-only; run real commands, report receipts |
| A1–A4 | audit | read-only; findings with file:line + real output |

Shared seams: none this wave (all files new or CLAUDE.md owned solely by W9).
