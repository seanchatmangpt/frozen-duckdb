# AD3r Break Attempt — KD-15 / KD-09 / KD-14

Date: 2026-10-19. Lane AD3r (replaces stalled AD3). Adversarial break attempt against 3 CLOSED KD rows. Every claim below cites a real command and exit code from this session; artifacts in `/tmp/ad3r/`.

## Verdict summary

| Row | Break attempt | Verdict | Row status |
|---|---|---|---|
| KD-15 | copy-stamp survival + scrubbed-id self-heal | **HOLDS** (self-heal verified twice, byte-exact restore) | stays CLOSED |
| KD-09 | 128MB cgroup re-witness | **HOLDS** (exit 0, RSS_HELD) | stays CLOSED |
| KD-14 | 1000-deep nesting + 10MB + malformed JSON | **HOLDS** (typed behavior, zero panics) | stays CLOSED |

**No row reopens.** One operational hazard and one pre-existing compile break were found (see Findings).

## KD-15 — install-name stamping and self-heal

**Leg 1: stamp survives a copy to a fresh path.**
`cp ~/.frozen-duckdb/cache/v2.0.0-dev-arm64/libduckdb.dylib /tmp/ad3r/fresh_copy.dylib`, then `otool -D`:
id `…/v2.0.0-dev-arm64/libduckdb_arm64.dylib` is embedded and survives the copy (absolute-path stamp, not symlink-relative). Nuance: the copied dylib still carries the OLD path as its id — a copy to a new location is not re-stamped by anything except the builder's persist path (`ensure_install_id(dest)`), so ad hoc copies do not get a new-path id. That is by design (id = source-of-truth cache path); consumers of manual copies need `install_name_tool` or DYLD_* themselves.

**Leg 2: self-heal repairs a scrubbed id.** Full shim court:

- BASELINE: `sha256 = 670acb8a…afe2498`, `otool -D` → own canonical cache path.
- SHIM: `install_name_tool -id "SCRUBBED-SHIM-COURT" <cache>/libduckdb_arm64.dylib` → `otool -D` witnesses `SCRUBBED-SHIM-COURT`.
- AFTER: `cargo build -p frozen-duckdb-sys` exit 0, "Using prebuilt DuckDB binary", `otool -D` → canonical cache path restored, ad-hoc codesign re-applied (`codesign -dv` Identifier present), `sha256 = 670acb8a…afe2498` — **byte-exact to baseline**. Self-heal is idempotent and length-stable.
- Repeated on a freshly restaged entry (see below): scrub → build (2.80s, id restored, pin consistent) → second build (1.91s, accept). **3 consecutive accepts post-self-heal.**

**Hazard found (not a reopen): fail-closed deletion is expensive.** During the first court pass, the cache entry was deleted by the builder's fail-closed path (`POISON_STATE_REFUSED … (truncated-dylib)`) and the subsequent build performed a **full local DuckDB compile: 19m27s** (no release tag exists for 2.0.0-dev, so restage = local compile, per CLAUDE.md known-open-item). The deletion-on-pin-mismatch policy is correct security-wise but converts a one-byte sidecar inconsistency into a ~20-minute rebuild. Candidate improvement (not landed): attempt re-staging from `/tmp/ad3r/pristine_arch.dylib`-class local evidence before deletion, or a restage cache of last-known-good bytes.

Also recorded honestly: the first `truncated-dylib` refusal that followed an otherwise-successful self-heal on the OLD cache entry (staged 13:40) did **not** reproduce in 3 consecutive runs on the fresh entry — UNRESOLVED single observation; the old entry was staged by an earlier builder build and is now replaced by the restage.

## KD-09 — cgroup low-ram witness (docker up, ran for real)

`MEM_MB=128 bash scripts/doctor/docker_low_ram_runner.sh /tmp/ad3r/kd09_128mb.json`, engine `linux/arm64 engine 29.5.2` (colima), image `kcura-lowram:latest` (pinned DuckDB b6013f7aa2):

- container exit code **0** (1s), stdout `ANOMALIES_Z_GT_2=261`, `VERDICT=RSS_HELD` — kernel-enforced 128MB cgroup ceiling held through the 1M-frame + rolling-anomaly workload.
- Receipt: `/tmp/ad3r/kd09_128mb.json` (same caveat as KDC-09: kernel virtualized, enforcement genuine).

Verdict: break FAILED, KD-09 stays closed under its existing virtualization caveat.

## KD-14 — variant adversarial payloads

Existing suites: `cargo test -p frozen-duckdb --test variant_tests` → **5 passed 0 failed**; `--test doctor_types_tests` → **3 passed 0 failed**.

Scratch court `/tmp/ad3r/kd14court` (path-dep on the repo crate, `cargo run --offline` exit 0):

- baseline round-trip `{"a":1,"b":[1,2,{"c":"x"}]}` → readback identical.
- **1000-level deep nesting** (`{"k":`×1000 … `}`×1000): insert Ok, readback returns the full nesting, no panic, no truncation error.
- **10MB payload** (`"blob"` of 10×2^20 chars, bind len 10,485,771): insert Ok in 12ms; readback `len=10485771 intact=true` — byte-exact round-trip.
- **10MB under 1000-deep nesting**: insert Ok.
- **Malformed JSON** `"{not json"`: insert Ok — and readback returns it verbatim, `payload::VARCHAR` = `{not json`. Semantic note: **VARIANT bind does not validate JSON at insert time** (store-as-is; validation is deferred to shredding). Typed, non-panicking, matches DuckDB VARIANT semantics; flagged so no downstream court assumes insert-time JSON validation.

Verdict: break FAILED, KD-14 stays closed.

## Pre-existing break (session-introduced? No — pre-existing)

`cargo test -p frozen-duckdb --test variant_shredded_bind` does not compile:

```
error[E0432]: unresolved import `frozen_duckdb::duckdb::variant_json_udf`
```

`crates/frozen-duckdb/src/duckdb/variant_json_udf.rs` is an untracked new file not (correctly) exposed through the `duckdb` module surface. Owned by whoever landed KDC-11-adjacent work; this lane did not touch it.

## Cache integrity (RESTORE receipt)

The court mutated the shared cache; final state is verified healthy:

- Entry was fully restaged by the builder itself (19m27s local compile): `libduckdb_arm64.dylib` 65,535,744 bytes, `.size` pin consistent (65535744), `otool -D` → own canonical path, ad-hoc signed, `cargo build -p frozen-duckdb-sys` exit 0 ("Using prebuilt", Finished 1.91s). New dylib sha `609c1111…632f1d` (restage rebuilt from upstream source; byte identity across local compiles is not a cache invariant).
- `/tmp/ad3r/pristine_arch.dylib` retains the pre-court bytes (sha 670acb8a…) for audit.
