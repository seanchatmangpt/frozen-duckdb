# DOC-HDIT-CERTIFY-RECEIPT — frozen-duckdb — v26.10.8

| field | value |
|---|---|
| repo | `/Users/sac/frozen-duckdb` (canonical checkout) |
| branch | `docs/doc-hdit-scaffold` |
| parent HEAD | `c7e53c3528eafe77a2546ce74327d1eddd80f981` |
| verdict | **ACCEPTED** (`doc-hdit certify`, exit 0) |
| standing | ALIVE for the certified doc surface — **subject to the Andon-Yellow R4 ceiling: `promotionAllowed=false`** (carried from the recertification receipt; this certification does NOT authorize promotion of the frozen-duckdb surface or any dependent; it certifies gates only) |
| extractor pin | sha256 `4c862576ab63595f9cd0417b35341af3ec1001f49450e79bf2e4c291a4a4246f` (`/Users/sac/ggen-marketplace/scripts/gen_doc_surface.py`); recorded natively in the chain receipt as BLAKE3 extractor identity `a579e2109941e1f27f2faf0403d6c91e3eebb7f234dcc191a814573001309616` |
| court | `ggen-marketplace/packs/rust-doc-hdit-pack/courts/doc_quality.court` (S ≥ 0.90, Φ ≤ 0.001, Q ≥ 0.65 — thresholds unmodified) |

> extractor pinned in receipt is historical; current fleet pin see ggen-marketplace docs/sjira/v26.10.8/PIN-ROTATION-LEDGER.md; pin retained as historical subject identity for this ACCEPTED certify at `c7e53c352`.

## Pipeline (real runs, commands + exits)

Extract (repo-level `.doc-surface.toml` `[[generated]]` bindgen sys-pair config honored by the
pinned extractor — the two `bindgen_bundled_version*.rs` files are excluded from the doc-surface
denominator):

```sh
python3 scripts/gen_doc_surface.py code /Users/sac/frozen-duckdb   # exit 0
python3 scripts/gen_doc_surface.py doc /Users/sac/frozen-duckdb --code-json fd.code.json  # exit 0
```

Merge to inputs schema (id + string coercion), then:

```sh
doc-hdit vectorize /tmp/hdit-fd/fd.inputs.json            # exit 0
doc-hdit audit     /tmp/hdit-fd/fd.inputs.json <court>    # exit 1 (first run) -> 0 (after grounding fix)
doc-hdit certify   /tmp/hdit-fd/fd.inputs.json <court> \
  --docs /Users/sac/frozen-duckdb/docs \
  --chain /Users/sac/frozen-duckdb/docs/sjira/v26.10.8/doc-hdit.receipts.jsonl \
  --extractor <pinned gen_doc_surface.py>                 # exit 0, verdict ACCEPTED
```

## Metrics (final certified run)

| gate | value | threshold | verdict |
|---|---|---|---|
| S_coverage (public set) | **0.9082** | ≥ 0.90 | PASS |
| Phi_halluc | **0.0000** | ≤ 0.001 | PASS |
| Q_density | **1.0000** | ≥ 0.65 | PASS |

Report-only: `S_coverage_raw` 0.0942, `coverage_denominators` set=425 raw=427
collapsed_delta=2, `prose_artifacts` 151, `external_documented` 3718.

## Grounding fix (first audit failed coverage)

First audit: S_coverage 0.8753 (FAIL), 53 public items uncovered. Fixed by writing genuine
reference documentation (no threshold relaxation):

- `docs/reference/r2d2-pooling.md` — `DuckdbConnectionManager` (`file`,
  `file_with_flags`, `memory`, `memory_with_flags`, `register_table_function`,
  `register_scalar_function`)
- `docs/reference/appender.md` — `Appender` (`append_row`, `append_rows`, `flush`)
- `docs/reference/scalar-functions.md` — `VScalar` / `ScalarFunctionSignature`
  (`exact`, `variadic`) / `ScalarParams` / registration functions
- `docs/reference/pragma-builder.md` — pragma `Sql` builder + `pragma*` Connection methods
- `docs/reference/benchmarking.md` — `measure_build_time`, `compare_build_times`

Coverage rose 0.8753 → 0.9082 (386/425 public items). Phi and Q unchanged (Φ was already 0).

## Pathspec note

This commit's pathspec is `docs/sjira/v26.10.8/*`, `docs/doc-hdit*`, `docs/.doc-surface.toml`
plus the five `docs/reference/*.md` grounding-fix files above — widened by necessity: the
certification gates are only reproducible at the tagged HEAD with the docs that ground the
covered symbols present in the tree. Omitting them would mint a tag whose subject does not
replay its own gates.

## Replay

```sh
shasum -a 256 /Users/sac/ggen-marketplace/scripts/gen_doc_surface.py   # 4c862576...
python3 /Users/sac/ggen-marketplace/scripts/gen_doc_surface.py code  /Users/sac/frozen-duckdb
python3 /Users/sac/ggen-marketplace/scripts/gen_doc_surface.py doc   /Users/sac/frozen-duckdb --code-json <code.json>
# merge: id + stringified fields per Claim schema, then:
/Users/sac/ggen-marketplace/packs/rust-doc-hdit-pack/target/release/doc-hdit certify <inputs.json> \
  /Users/sac/ggen-marketplace/packs/rust-doc-hdit-pack/courts/doc_quality.court \
  --docs docs --chain docs/sjira/v26.10.8/doc-hdit.receipts.jsonl --extractor <extractor>
```

Chain subject digest: `6693b85f292aae5c060038a6f42af2618c4bfa3c03d26119b1d726937554e395`;
receipt hash `b19213a2e05b6e427a041c78b5068bddead9dee1efb1db7a47aeb2d394f55c20`; parent none
(first receipt in chain).
