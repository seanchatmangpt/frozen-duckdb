# Sync Digest Design (Zero-Cloud Analytics Sync)

**Status: DESIGN / UNVERIFIED — no implementation exists.**
Everything below is a specification only; no code, crate, or court has been
built. Every claim is a design candidate until a receipt-gated court runs
against a real implementation.

Source: PRD §2.3, journey 2 — clients execute analytics fully locally; only
cryptographically signed digests of local state sync upstream.

## 1. Model

- Each client device owns its local DuckDB database (frozen-duckdb embedded).
- Upstream (the sync service) never receives data — only signed digests.
- Digests are the *sole* sync payload. If a payload is not a digest, the
  sync transport refuses it.
- The digest is a Merkle-style fold: table digests fold from segment digests,
  the state digest folds from table digests.

## 2. Digest content

- Hash function: **BLAKE3** keyed mode over a canonical byte serialization.
- Canonical serialization: length-prefixed UTF-8 of each element, elements
  sorted lexicographically before hashing (order-independent fold, so a
  re-ordered local layout produces the identical digest).
- Leaf level: per **segment** — BLAKE3 over (segment id, row count,
  per-column min/max digest).
- Mid level: per **table** — BLAKE3 over the sorted segment digests.
- Root: **state digest** = BLAKE3 over the sorted table digests plus a
  schema-version tag (e.g. `"fzddb-sync-digest-v1"`).
- Repo precedent: the ggen receipt chain already uses BLAKE3; this reuses
  the same primitive rather than introducing a second hash family.
- Digest size: 32-byte root; payload carries root + per-table digests so
  upstream can diff per table without ever seeing rows.

## 3. Signing scheme

- Each client generates an **Ed25519 keypair on-device**.
- Payload = `{state_digest, table_digests, schema_version, ed25519_sig}`.
- Upstream **verifies only**. It stores the digest chain; it never signs.
- Key generation, backup, and rotation are **UNVERIFIED-DESIGN**: no
  decision yet on escrow, social recovery, multi-device derivation, or
  rotation cadence. Marked open; any of these choices changes the threat
  model and must not be assumed.
- Upstream maintains a per-device key registry (device id -> public key).
  First-seen key registration is UNVERIFIED-DESIGN (TOFU vs. attested
  enrollment is undecided).

## 4. What never leaves the device

- Raw events / rows (event-level data of any kind).
- Query text (user queries are local-only; digests carry no SQL).
- Column values, min/max bounds in cleartext (the per-column bounds fold
  into the segment digest — only the digest crosses the wire, not the
  bounds themselves).
- Column names? **UNVERIFIED-DESIGN**: schema-shape leakage via table
  digests is acknowledged as an open privacy question; candidate mitigations
  (salted schema hashing, per-table salt) are listed, not decided.

## 5. Privacy falsifier: the sync-payload court

A court (automated gate, must be able to exit 1) asserting the uploaded
blob contains no event-level data. Requirements:

- **Real parse, not a stub**: the court deserializes the exact payload
  bytes the transport would upload — parse the envelope, walk every field,
  and assert each field is one of: 32-byte digest, signature, integer,
  version tag, or device id. Any string field longer than a version tag
  fails the court.
- **Non-vacuity**: the court must demonstrably refuse on a mutated payload
  (e.g. inject a raw event row into the payload; the court MUST exit 1).
  A court that passes on the mutation is a vacuous gate — the release is
  blocked until the court detects it.
- **Byte-identity**: the court parses the same bytes the transport sends;
  no "court fixture" substitutes its own fixture bytes. Transport and court
  share one payload-serialization function; the court calls it.
- Standing vocabulary: implementation PASS = court green on the real
  transport payload + mutation refused; SUSPECT = any stub parse or fixture
  substitution; BLOCKED = no court exists.

## 6. Open items (all UNVERIFIED-DESIGN)

- Key rotation, escrow, multi-device key derivation (§3).
- First-key enrollment (TOFU vs. attested) (§3).
- Table-name/schema leakage mitigation (§4).
- Payload envelope format (CBOR vs. protobuf) — UNVERIFIED-DESIGN.
- Digest frequency / batching cadence — UNVERIFIED-DESIGN.
- Replay protection (nonce/timestamp window) — UNVERIFIED Ed25519 sigs
  are deterministic; replay of a signed digest is possible without a
  freshness field. Freshness is not yet specified.

## 7. Falsifier for the whole design

The design is refuted if either court cannot be built as specified:

1. The sync-payload court cannot parse-and-assert on real transport bytes.
2. A payload containing raw events passes the court (vacuous gate).

Until an implementation runs both, the entire document is UNVERIFIED.
