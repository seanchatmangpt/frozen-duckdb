#!/usr/bin/env python3
"""PreToolUse guard: refuse the built-in generic agents (general-purpose, claude).

frozen-duckdb CLAUDE.md: only agents defined in .claude/agents/ plus read-only Explore.
A generic-agent dispatch exits 2 (block) with a typed refusal naming the
allowed agent types.
"""
import json
import sys

BANNED = {"general-purpose", "claude"}
ALLOWED = [
    "Explore", "Plan", "bench-runner", "gate-author", "milestone-doc",
    "release-op",
]

def main() -> None:
    try:
        payload = json.load(sys.stdin)
    except Exception:
        sys.exit(0)  # unreadable input: fail open, other guards own that case

    tool = payload.get("tool_name", "")
    if tool != "Agent":
        sys.exit(0)

    subagent = payload.get("tool_input", {}).get("subagent_type", "")
    if subagent in BANNED:
        reason = (
            f"GENERIC_AGENT_REFUSED: subagent_type '{subagent}' is banned in frozen-duckdb. "
            f"Allowed: {', '.join(ALLOWED)} (see .claude/agents/ + Explore). "
            "Re-dispatch with a project agent type."
        )
        print(json.dumps({"decision": "block", "reason": reason}))
        sys.exit(2)

    sys.exit(0)

if __name__ == "__main__":
    main()
