# attic/

Historical generators, retained for window provenance. Byte-exact as moved
(sha256 recorded at move time, ERRC-E1 Eliminate lane, 2026-10-19):

| file | provenance |
|---|---|
| `gen_sjira_tickets.sh` | wave-2 ticket renderer; SPECS table rendered `docs/sjira/v26.9.21/T[0-9]*.md` (closed window) |
| `gen_sjira_tickets_wave3.sh` | wave-3 hardening renderer; rendered `docs/sjira/v26.9.21/TR*.md` (closed window) |
| `gen_sjira_tickets_wave4.sh` | wave-4 marketplace-capability renderer; rendered `docs/sjira/v26.9.21/C[0-9]*.md` (closed window) |
| `gen_sjira_tickets_wave5.sh` | wave-5 crawl-wave renderer; rendered `docs/sjira/v26.9.21/G[0-9]*.md` (closed window) |

These are not run by any gate, Makefile target, or `ggen sync run` rule.
Do not re-activate without re-opening the corresponding `docs/sjira/v26.9.21/`
window. The `docs/GENESIS.md` rows for these paths are stale-by-design until
that reconciliation manifest is next edited.
