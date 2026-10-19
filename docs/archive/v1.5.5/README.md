# docs/archive/v1.5.5

Legacy documentation snapshots from the v1.4.x/v1.5.5 era, archived intact at
the v1.5.5 → 2.0.0-dev milestone. Content is preserved byte-for-byte; only the
paths changed.

## Provenance

- **Origin**: authored for the v1.4.0 release (commit `b41574b`) as part of the
  pre-wave hand-authored docs tree (`docs/{api,architecture,cli,guides,llm,...}`).
- **Why archived**: each file carries its own audited STATUS marker stating it
  was "authored for the v1.4.0 release ... and was not part of the v1.5.5 doc
  reconciliation" and documents commands (`embed`, `search`, parts of RAG
  pipelines) that are **not implemented** (exit 101). They are wholly
  historical, documenting intended surface rather than verified behavior.
- **Selection criterion**: wholly historical only. Live docs that merely cite
  v1.4.x facts in historical sections (e.g.
  `docs/contributing/architecture-decisions.md` wave-2 SUPERSEDED block, the
  design corpus's VERIFIED-SYNTAX-on-v1.4.1-CLI markers) stayed in place.
- **Excluded from archiving (GENESIS-tracked)**: all C2 Diataxis files
  (`docs/index.md`, `docs/meta.md`, `docs/tutorials/`, `docs/how-to/`,
  `docs/reference/`, `docs/explanation/`) carry `lineage:7f604e5` `gap` rows in
  `docs/GENESIS.md` and are existence-checked; they were never candidates.

## Contents

| archived file | original path | marker |
|---|---|---|
| `llm/embeddings.md` | `docs/llm/embeddings.md` | STATUS audited 2026-09-22, v1.4.0-authored |
| `llm/ollama-setup.md` | `docs/llm/ollama-setup.md` | STATUS audited 2026-09-22, v1.4.0-authored |
| `llm/semantic-search.md` | `docs/llm/semantic-search.md` | STATUS STATUS audited 2026-09-22 header, v1.4.0-authored |
| `llm/rag-pipelines.md` | `docs/llm/rag-pipelines.md` | STATUS audited 2026-09-22, v1.4.0-authored |

## GENESIS note

None of these paths are GENESIS-tracked (no rows in `docs/GENESIS.md`), so no
manifest row updates were required. `make genesis-check` was re-run after the
moves and stayed exit 0 (58/58 checks + gaps, see window receipts).
