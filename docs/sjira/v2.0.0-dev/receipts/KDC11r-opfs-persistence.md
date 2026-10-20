# KDC11r — OPFS Persistence Court Receipt (lane KDC11r, replaces stalled KDC-11)

- **Date**: 2026-10-19 (local run 2026-10-20T03:2x UTC stamp inside marker)
- **Subject**: /Users/sac/frozen-duckdb @ `7aab55bef00917f3df89caf2cbb6deb964739627` (branch master)
- **Court script**: `scripts/doctor/opfs_persistence_court.mjs` (sha256 `9dbf59037a0714826d36e2820a84ecd76906c3a8117782bf32e5832d282d30f5`)
- **Raw output**: `/tmp/kdc11r-fresh.json` (sha256 `d51955c913ca2dc8d94cecd7ceb28771446d8ec758ca11d13af47610cc9ec6d4`), stderr `/tmp/kdc11r-fresh.err` (empty)
- **Exit code**: `0` → standing **PASS** (exit contract: 0=PASS, 2=SKIP-LOUD, 1=FAIL)
- **Browser**: chrome-headless-shell 155.0.8059.84 (mac_arm) via `/tmp/chs-court`, raw CDP WebSocket, no deps
- **Wasm subject**: `target/wasm/duckdb.wasm` GAP1b build, **41,099,303 bytes** (v2.0.0-dev88595)

## Legs

| Leg | Verdict | Evidence |
|---|---|---|
| L1 crossOriginIsolated + SAB under COOP/COEP | **PASS** | `crossOriginIsolated: true`, `sab: "function"` |
| L2 GAP1b wasm instantiates in a Worker | **OBSERVED (no stdout captured)** | `importError: null`, `mainReturned: true`; see missingSurface below |
| L3 OPFS marker write (platform surface) | **PASS** | 54 bytes, marker text + sha256 below |
| L4 Page.crash + SIGKILL → relaunch same profile → read-back | **PASS** | sha256 identical pre/post crash |

Marker: `KDC11-OPFS-PERSISTENCE-MARKER:2026-10-20T03:27:44.471Z` (54 bytes)
sha256 (L3 write == L4 read-back, byte-identical): `12badefa81d562d6ae2f2b70daf17ea92dd59c0b14ab0ff6b01dc4e70729ab9f`

## Honest scope

`sqlOutputPresent: false`. The GAP1b emscripten glue exposes no programmatic DuckDB surface
(23 minified wasm exports, no `Module._duckdb_*` C API, no `Module.FS`, zero `opfs` matches in
duckdb.js; only `_main` CLI shell). The wasm→OPFS leg therefore exercises the **platform OPFS
surface** (`navigator.storage.getDirectory`) with a court-owned marker payload; the wasm
programmatic surface gap is registered, not faked.

Also observed: worker-side `createWritable` hangs in chrome-headless-shell; the write leg runs
page-side (documented in-script as a court observation).

## Verbatim court JSON output (exit 0)

One intentional collapse: `receipts.serverHits` (86 identical `/worker.js`/asset hits) is
replaced by a count annotation to keep this file readable; the byte-exact raw JSON including
the full array lives at `/tmp/kdc11r-fresh.json` (sha256 above).

```json
{
  "court": "opfs_persistence_court",
  "standing": "PASS",
  "receipts": {
    "origin": "http://127.0.0.1:61993",
    "browserBinary": "/tmp/chs-court/chrome-headless-shell/mac_arm-155.0.8059.84/chrome-headless-shell-mac-arm64/chrome-headless-shell",
    "wasmBytes": 41099303,
    "legs": {
      "write": {
        "kind": "result",
        "phase": "write",
        "result": {
          "importError": null,
          "mainReturned": true,
          "crossOriginIsolated": true,
          "sab": "function",
          "captured": [],
          "capturedErr": [],
          "exportedFns": [],
          "exportedFnCount": 0,
          "hasModuleFS": "undefined",
          "sqlOutputPresent": false
        }
      },
      "opfsWrite": {
        "ok": true,
        "bytes": 54,
        "text": "KDC11-OPFS-PERSISTENCE-MARKER:2026-10-20T03:27:44.471Z",
        "sha256": "12badefa81d562d6ae2f2b70daf17ea92dd59c0b14ab0ff6b01dc4e70729ab9f"
      },
      "crash": {
        "fired": true,
        "signature": "Page.crash (session dies without reply, expected)"
      },
      "readBack": {
        "result": {
          "ok": true,
          "bytes": 54,
          "sha256": "12badefa81d562d6ae2f2b70daf17ea92dd59c0b14ab0ff6b01dc4e70729ab9f",
          "text": "KDC11-OPFS-PERSISTENCE-MARKER:2026-10-20T03:27:44.471Z"
        }
      }
    },
    "serverHits": [
      "<86 hits> first 4: ['/court.html', '/worker.js', '/wasm/duckdb.js', '/duckdb.wasm']"
    ],
    "verdict": {
      "L1_isolated_sab": "PASS",
      "L2_wasm_instantiate": "OBSERVED (no stdout captured \u2014 see missingSurface)",
      "L3_opfs_write": "PASS",
      "L4_crash_survival_sha256": "PASS",
      "sqlOutputPresent": false,
      "profileDirReused": true,
      "missingSurface": [
        "GAP1b glue exposes no programmatic DuckDB surface: 23 minified wasm exports (sa..Oa), no Module._duckdb_* C API, no Module.FS, zero 'opfs' matches in duckdb.js; only _main (CLI shell) \u2014 SQL result bytes cannot be driven or extracted through a supported surface"
      ]
    },
    "cleanupError": "ENOTEMPTY, Directory not empty: '/Users/sac/.cache/tmp/kdc11-opfs-wmPGje'"
  }
}```

## Prior-run corroboration

`/tmp/kdc11-run17.json` (stalled KDC-11's run): same PASS shape, its own marker
`...03:23:15.690Z`, its own sha256 `3829a7e211a621e418453bcc4d3775b040b3c446b440167ecc804a5704791f61`
with internal write==read match. Cross-run sha256 equality is NOT expected: each run writes a
fresh timestamped marker, crashes, and reads back its own bytes — the corroborating fact is
each run's internal write/read sha256 identity, which both runs show.

## Standing

**PASS** — exit 0, all hard legs (L1, L3, L4) PASS; L2 OBSERVED with the missing programmatic
wasm surface registered in `missingSurface`. No SKIP condition fired (browser + wasm assets
present).
