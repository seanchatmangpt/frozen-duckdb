---
name: fd-anneal
description: Run continuous annealing waves over the frozen-duckdb checkout — concurrent Audit/Mutate/Repair/Verify lanes, wave after wave, until a full wave returns zero defects (saturation). Use for 8am-class deadlines or "keep annealing", "run waves", "anneal until saturated".
---

# frozen-duckdb anneal — wave loop until saturation

Standing vibration loop per entropic-annealing doctrine. Keep ~20 lanes
running continuously; the ONLY stop conditions are (a) a full wave returns
zero defects (saturation), (b) the usage-limit / ENV warning pauses the
run, or (c) the operator stops it. Never stop early to "summarize".

## Wave shape (repeat)

1. **Dispatch EXACTLY 20 lanes in ONE message** over the same canonical
   checkout (/Users/sac/frozen-duckdb, one checkout, never worktrees or
   copies), disjoint file ownership per docs/sjira/v26.10.8/_LANES.md.
   COUNT the Agent tool calls in the message before sending: fewer than
   20 is a dispatch defect — fix it in the same turn. Lanes are background
   Agents; use Explore for read-only audit and the project writer types
   (bench-runner, gate-author, milestone-doc, release-op) for
   mutate/repair — `general-purpose` and `claude` are banned by the
   PreToolUse hook; if no project agent covers a write surface, that is a
   defect against .claude/agents/: author the type, then dispatch. Mix per
   wave: 8 Audit / 4 Mutate / 4 Repair / 4 Verify. ERRC order when
   capacity frees: kill/prune/tighten first, repair audited defects
   second, generation last.
2. **Lane contracts** — every prompt carries: absolute repo path
   /Users/sac/frozen-duckdb, lane file ownership from _LANES.md,
   `CARGO_TARGET_DIR=target-lane<N>` (per-lane build isolation; the
   coordinator deletes every `target-lane<N>` at integration — lane
   build roots are leases, not assets), and "report defects with
   file:line + real command output (exit code, counts). No output = no
   finding." Auditors never build what they audit. Lanes never run git
   state-changing commands — the coordinator owns all transitions.
3. **Integrate when notifications drain** — classify defects, dispatch
   repair lanes only on verified defects (one hypothesis each), then
   verify with the cheapest rung that covers the blast area
   (`cargo test` → `cargo bench` + SLO gate → full receipt). Re-run the
   failing boundary first, then expand. No stash baselining — snapshot a
   file via `git show HEAD:<path> > /tmp/...` instead; a lane edit that
   breaks the shared compile is fixed or reverted by its owner within
   ~10 minutes (compile-freeze SLA; minimal unblock fixes allowed with
   disclosure in BOTH lanes' receipts).
4. **Record the wave** — append one jsonl row per lane to
   docs/sjira/v<version>/wave-telemetry.jsonl; keep a running defect
   count for the wave. A test that mints a unique-per-run build root
   must `on_exit`-delete it in the same file.

## Saturation check

A wave is CLEAN when every lane returns zero verified defects (a PASS with
a real-command receipt is the minimum unit). Saturation = one full clean
wave. Until then, immediately dispatch the next wave — do not pause, ask,
or narrate between waves.

## Stop conditions (hard)

- Full wave, zero verified defects → report saturation + wave count, stop.
- ENV/usage-limit warning arrives → park cleanly (finish in-flight lanes,
  note wave count + open defects), resume on the next prompt with
  "continue annealing".
- Operator stop → same park protocol.

## Drift guards

- Auditors verify by execution, never by reading another lane's claim:
  N lanes agreeing is not N observations.
- Any defect found by coordinator pre-thinking instead of a lane is a
  wave-decoration failure — log it, don't count it.
- 3+ consecutive waves still returning defects means the audit lanes are
  tautological or repairs are regressing — audit the receipts before
  dispatching wave N+1.
- Integration: lane-clean before the coordinator commits, `git commit
  -F <file>` (never inline `-m`), delete all `target-lane<N>` build
  roots, then run the doc/verify gates.
