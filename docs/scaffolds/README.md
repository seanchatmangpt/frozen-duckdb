# doc-hdit scaffolds — top safe-wrapper gap modules

<!-- GENERATED-BANNER: reference.md / how_to.md / explanation.md are rendered -->
<!-- by doc-hdit scaffold from gen_doc_surface.py extraction of THIS repo at   -->
<!-- d45739d+. Regenerate:                                                     -->
<!--                                                                           -->
<!--   python3 /Users/sac/ggen-marketplace/scripts/gen_doc_surface.py code /Users/sac/frozen-duckdb \
> /tmp/hdit/f38.code.json                                                       -->
<!--   python3 - <<'EOF'  # deterministic filter to the top-gap modules        -->
<!--   import json; d=json.load(open('/tmp/hdit/f38.code.json'))               -->
<!--   want=('duckdb/statement.rs','duckdb/core/vector.rs','duckdb/row.rs')    -->
<!--   d['modules']=[m for m in d['modules'] if any(w in m['file'] for w in want)] -->
<!--   json.dump(d, open('/tmp/hdit/f38.scaffold.json','w'))                   -->
<!--   EOF                                                                     -->
<!--   /Users/sac/ggen-marketplace/packs/rust-doc-hdit-pack/target/release/doc-hdit scaffold \
--code /tmp/hdit/f38.scaffold.json                                             -->
<!--       --templates /Users/sac/ggen-marketplace/packs/rust-doc-hdit-pack/templates -->
<!--       --out docs/scaffolds                                                -->

Scaffolds for the three largest remaining safe-wrapper coverage gaps
(audit @d45739d: S_coverage 0.8789, 179-item gap, 131 of which are the
bindgen sys pair; the residual 48 items concentrate here):

| module | public items |
|---|---|
| `crates/frozen-duckdb/src/duckdb/statement.rs` | 21 |
| `crates/frozen-duckdb/src/duckdb/core/vector.rs` | 31 |
| `crates/frozen-duckdb/src/duckdb/row.rs` | 16 |

Only the bounded slots between AGENT markers are hand-writable; the
reference tables are AGENT-FORBIDDEN and rendered from the real code
surface (rows match `crates/` source; no synthetic signatures).
