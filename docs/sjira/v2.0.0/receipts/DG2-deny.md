# DG2 — cargo-deny advisories receipt (P0 zip validation)

Date: 2026-10-19. Subject: `/Users/sac/frozen-duckdb` @ master (cbeb249),
working tree with in-flight lane edits. NO git commands run.

## Result: PASS (exit 0) with documented accepts

## 1. cargo-deny availability

`cargo-deny` preinstalled: `/Users/sac/.cargo/bin/cargo-deny`. No install
needed; no loud-skip path taken.

## 2. zip migration verified (Cargo.lock)

```
name = "zip"
version = "9.0.5"
```

## 3. Pre-config check (no deny.toml, default config)

`cargo deny check advisories 2>&1` -> exit 1.

- RUSTSEC-2026-0367 / 2026-0368 / 2026-0369 (zip 0.x): **GONE** — none
  reported post zip-9-migration.
- RUSTSEC-2026-0258 (h2 0.3.27, unbounded empty DATA frames, Low; patched
  in h2 0.4.16): **REMAINING** — via hyper 0.14.32 / reqwest 0.11.27 ->
  frozen-duckdb, frozen-duckdb-builder.
- RUSTSEC-2025-0134 (rustls-pemfile 1.0.4 unmaintained): **REMAINING** —
  via reqwest 0.11.27.

Exactly the expected non-blocking set; no new advisories.

## 4. Config

Authored `/Users/sac/frozen-duckdb/deny.toml` with `[advisories]` ignore =
[RUSTSEC-2026-0258, RUSTSEC-2025-0134], each with a comment citing PB11's
classification and the close path (reqwest 0.12 migration wave, separately
owned lane). Nothing else suppressed.

## 5. Post-config check

```
$ cargo deny check advisories 2>&1 | tail -5
advisories ok
EXIT=0
```

## Standing

- deny.toml advisories gate: ALIVE (exit 0, two documented accepts, both
  gated on the reqwest 0.12 wave).
- New findings: NONE.
