# A2A no-op decision

frozen-duckdb is a **documented no-op** for the agent-card plane: no servable
agent surface exists or is planned.

- The repo's surfaces are a Rust/C library (Cargo workspace under `crates/`)
  and a CLI — nothing in the repo serves HTTP or exposes an agent card.
- No `.well-known/agent-card.json` (or `agent.json`) exists.
- No A2A/sjira card workgraph order is open against this repo.

Classification: `documented-no-op` (neither card-served nor workgraph-only).
The fleet map (`ggen-marketplace/docs/reference/FLEET-SEMANTIC-MAP.md`)
carries this decision.
