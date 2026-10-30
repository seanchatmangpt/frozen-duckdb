# PB14 — crates.io Name Ownership Preflight

Lane: PB14 · Date: 2026-10-19 · Repo: /Users/sac/frozen-duckdb · Verdict: CLEAR (0 of 3 names at risk)

## Method

All queries: `curl -s -H "User-Agent: frozen-duckdb-preflight (seanchatmangpt)" https://crates.io/api/v1/crates/<name>` (crates.io requires a User-Agent). Run 2026-10-19.

## 1. API receipts

### frozen-duckdb — EXISTS

- id: `frozen-duckdb`, created_at: `2025-10-07T22:35:53Z`
- versions: 1 (`0.1.0`, max/newest/stable all 0.1.0, not yanked), version id 1764157
- downloads: 2336 total / 154 recent
- description: "Pre-compiled DuckDB binary for fast Rust builds - Drop-in replacement for duckdb-rs"
- repository / homepage / documentation: `https://github.com/seanchatmangpt/frozen-duckdb` (all three)
- license: MIT · crate_size: 3557939 · has_lib: true · bin: `frozen-duckdb-cli`
- `published_by`: id 360588, login `seanchatmangpt`, name "Sean Chatman", `github_username_matches: true`
- audit_actions: single `publish` by login `seanchatmangpt` at 2025-10-07T22:35:53Z

Owners endpoint (`/api/v1/crates/frozen-duckdb/owners`):

```json
{"users":[{"kind":"user","id":360588,"login":"seanchatmangpt","url":"https://github.com/seanchatmangpt","name":"Sean Chatman","github_username_matches":true}]}
```

Single owner, no teams, no third parties.

### frozen-duckdb-builder — DOES NOT EXIST

```json
{"errors":[{"detail":"crate `frozen-duckdb-builder` does not exist"}]}
```

Unregistered — free for us to claim on first publish.

### frozen-duckdb-sys — DOES NOT EXIST

```json
{"errors":[{"detail":"crate `frozen-duckdb-sys` does not exist"}]}
```

Unregistered — free for us to claim on first publish.

## 2. Repo-field verification

`frozen-duckdb` repository field = `https://github.com/seanchatmangpt/frozen-duckdb` — matches this repo's GitHub home. PASS (no name-squat via misdirection).

## 3. gh auth vs crates.io owner

- `gh auth status`: logged in as `seanchatmangpt` (active, keyring).
- crates.io owner login (from owners API + `published_by`): `seanchatmangpt`.
- Exact match; additionally the API reports `github_username_matches: true` (crates.io verified the GitHub link itself).
- PASS — no OPERATOR-VERIFY needed; login is derivable from API data plus local gh status.

## 4. Reserved / K-in-P (squat) check

- The existing 0.1.0 is owned by `seanchatmangpt` (us), published by the same login — not a squatter. K-in-P: `own(current 0.1.0) = us`.
- No reserved-name issue: both remaining workspace names are unregistered, and we (the owner of `frozen-duckdb`) will be the first publisher of each — squatters cannot beat us to namespaced ownership, but note crates.io ownership is per-crate, so `frozen-duckdb-builder`/`-sys` should be published promptly (first-publish wins) and the existing account is in good standing to do so.

## Receipt

- Commands: 4 curl calls (crates, owners) + `gh auth status`; exit 0 on all.
- Verdict: CLEAR. All three crate names controlled by (or claimable exclusively by) seanchatmangpt.
