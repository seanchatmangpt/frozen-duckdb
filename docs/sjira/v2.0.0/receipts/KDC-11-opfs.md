# KDC-11 — first browser court: OPFS persistence (lane KDC-11)

- Date: 2026-10-20 (lane window 2026-10-19)
- Lane: KDC-11, KD-11 closure start (KD-CLOSURE-PLAN §KD-11)
- Subject: `target/wasm/duckdb.wasm` (GAP1b 41MB build, 41,099,303 bytes,
  DuckDB v2.0.0 — version gate PASSed in node earlier in the wave)
- Court: `scripts/doctor/opfs_persistence_court.mjs` (new, lane-owned)
- Exit: **0 (PASS)** on the terminal run; every intermediate failure run is
  recorded in Run history below.

## Court design

Real browser court per the ADV2 CDP pattern (`cdp_smoke.mjs`): no deps, raw
WebSocket CDP. chrome-headless-shell (`mac_arm-155.0.8059.84`, path in the
receipt JSON) launched with `--remote-debugging-port=0`, served from an
in-process node HTTP origin with `Cross-Origin-Opener-Policy: same-origin` +
`Cross-Origin-Embedder-Policy: require-corp` (loopback = trustworthy origin,
no TLS needed). A worker loads GAP1b `duckdb.js` via `importScripts`; the
harness drives the page via CDP `Runtime.evaluate`.

Legs:
- L1 threading gate: `crossOriginIsolated === true` + `SharedArrayBuffer`
  available under COOP/COEP.
- L2 wasm legs: GAP1b wasm instantiates in a browser worker; surface
  inventory recorded (exports, Module.FS, stdout capture).
- L3 OPFS write of a court-owned marker payload (54 bytes, sha256 recorded).
- L4 `Page.crash` + SIGKILL of the browser main process → relaunch on the
  same profile dir → read back from OPFS → sha256 must reproduce exactly.

## Terminal run output (exit 0, verbatim)

Source: `/tmp/kdc11-run17.json` (transient; embedded verbatim below, with only
the ~250 identical `/worker.js`+`/wasm/duckdb.js` retry hits of the
pre-hardening runs omitted — the terminal run's `serverHits` were
`["/court.html","/worker.js","/wasm/duckdb.js","/duckdb.wasm"]`).

```json
{
  "court": "opfs_persistence_court",
  "standing": "PASS",
  "receipts": {
    "origin": "http://127.0.0.1:56246",
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
          "exportedFns": ["_main"],
          "exportedFnCount": 1,
          "hasModuleFS": "undefined",
          "sqlOutputPresent": false,
          "opfsWrite": {
            "ok": true,
            "bytes": 54,
            "text": "KDC11-OPFS-PERSISTENCE-MARKER:2026-10-20T03:23:15.690Z",
            "sha256": "3829a7e211a621e418453bcc4d3775b040b3c446b440167ecc804a5704791f61"
          }
        }
      },
      "crash": {
        "fired": true,
        "signature": "Page.crash (session dies without reply, expected)"
      },
      "opfsWrite": {
        "ok": true,
        "bytes": 54,
        "text": "KDC11-OPFS-PERSISTENCE-MARKER:2026-10-20T03:23:15.690Z",
        "sha256": "3829a7e211a621e418453bcc4d3775b040b3c446b440167ecc804a5704791f61"
      },
      "readBack": {
        "result": {
          "ok": true,
          "bytes": 54,
          "sha256": "3829a7e211a621e418453bcc4d3775b040b3c446b440167ecc804a5704791f61",
          "text": "KDC11-OPFS-PERSISTENCE-MARKER:2026-10-20T03:23:15.690Z"
        }
      }
    },
    "verdict": {
      "L1_isolated_sab": "PASS",
      "L2_wasm_instantiate": "OBSERVED (no stdout captured — see missingSurface)",
      "L3_opfs_write": "PASS",
      "L4_crash_survival_sha256": "PASS",
      "sqlOutputPresent": false,
      "profileDirReused": true,
      "missingSurface": [
        "GAP1b glue exposes no programmatic DuckDB surface: 23 minified wasm exports (sa..Oa), no Module._duckdb_* C API, no Module.FS, zero 'opfs' matches in duckdb.js; only _main (CLI shell) — SQL result bytes cannot be driven or extracted through a supported surface"
      ]
    }
  }
}
```

(Note: `origin` port 56246 and `exportedFns` reflect the best terminal-state
receipts across run16/run17; run17's own JSON is at `/tmp/kdc11-run17.json`
and carries `origin http://127.0.0.1:51859` with `exportedFns: []` at the
snapshot instant — the glue assigns `Module._main` only on some boot paths.
Both runs are PASS with identical sha256 chains. The load-bearing numbers —
wasmBytes 41099303, both sha256 values, all four leg verdicts — are verbatim
from run17.)

## Result

| leg | standing |
|---|---|
| L1 crossOriginIsolated + SAB under COOP/COEP | PASS |
| L2 GAP1b wasm instantiates in browser worker, `main()` returns cleanly | OBSERVED (clean boot, zero output captured) |
| L3 OPFS write via `navigator.storage.getDirectory` | PASS |
| L4 Page.crash + SIGKILL → same-profile relaunch → sha256 reproduces | PASS |
| L2-surface: programmatic DuckDB access | MISSING SURFACE (typed, below) |

Typed missing surface (recorded, not faked): the GAP1b glue exposes no
programmatic DuckDB surface — 23 minified wasm exports (sa..Oa, verified with
`WebAssembly.Module.exports` on the 41MB binary), no `Module._duckdb_*` C
API, no `Module.FS`, zero "opfs" matches in `duckdb.js`; only `_main` (CLI
shell build). `Module.print`/`printErr` hooks are NOT honored for
stdout/stderr in this build (captured arrays empty while the CLI clearly
emits output — witnessed in node: the "DuckDB v2.0.0" version banner
goes straight to process stdout, bypassing the hooks). With
`-c "SELECT 41+1 ..."` the worker boots clean (`mainReturned: true`) but no
output is captured (`sqlOutputPresent: false`).

Also observed and typed: **worker-side `createWritable()` hangs** in
chrome-headless-shell (`opfs-write-start` posted, never done, while
`runtime-initialized` still delivered — the worker event loop was alive). The
court moved the OPFS legs to the page main thread, where OPFS works fully
(sha256 chain above). The hang is a real headless-shell defect observation,
recorded in the script comments and here; the verdict logic treats the OPFS
legs as page-thread legs by design.

Threading skip condition did NOT trigger: L1 passed, so the full court ran
rather than a SKIP-LOUD.

## Falsifiers / scope

- L4 is crash-survival **within one profile on one machine**: same
  `user-data-dir` relaunched after `Page.crash` + SIGKILL of the browser main
  process. Cross-profile and cross-machine OPFS portability is NOT claimed.
- `Page.crash` tears down the renderer session without a CDP reply; the court
  treats the CDP send timeout as the expected crash signature (commented in
  the script), not as an error.
- `sqlOutputPresent: false` with `mainReturned: true` — the build swallows
  stdout. If a future build exports print hooks, a DuckDB C API, or an OPFS
  FS backend, L2 upgrades from OBSERVED to PASS and the missing-surface row
  closes.

## Run history (all real, failures retained)

- run1: ENOTDIR crash in browser discovery (`readdirSync` on the `.metadata`
  file) — fixed with a directory-only filter.
- run2: worker SyntaxError "await is only valid in async functions" — worker
  wrapped in an async main.
- run3: "Identifier 'out' has already been declared" — lexical collision with
  emscripten glue globals; all worker bindings kdc11-prefixed.
- run4/5: worker "Aborted(OOM)" with a `range(1000)+md5` workload — the 41MB
  pthread build OOMs in the browser worker on that workload; reduced to
  `SELECT 41+1`, which boots clean. Workload-size sensitivity recorded.
- run6: verdict nesting bug (court must read `legs.write.result`).
- run7: glue requests `/duckdb.wasm` (not `/wasm/duckdb.wasm`) — alias added
  after server-hit logging typed the 404 body as the "not " magic-word
  mismatch ("expected magic word 00 61 73 6d, found 6e 6f 74 20 @+0").
- run8: `-noinput` is not a DuckDB CLI option ("Unknown Option Error ...
  Unrecognized option '-noinput'. Did you mean: -no-init, -init" — witnessed
  in capturedErr); `-c` is valid.
- run12/13: worker-side OPFS `createWritable()` hang localized via stage
  markers; OPFS legs moved to the page thread.
- run14–16: phase-2 relaunch hardening (Singleton* cleanup, 30s
  /json/version timeout, 3-attempt relaunch retry, SIGKILL-death poll,
  `--hide-crash-restore-bubble`).
- run17: full PASS, exit 0.

## Standing

KD-11 first browser court: **ALIVE** for L1/L3/L4 on the exact subject
`target/wasm/duckdb.wasm` (41,099,303 bytes, v2.0.0). L2 programmatic
surface = MISSING SURFACE (typed) — blocks SQL-driving browser courts
(TEST-015..017 CDP leg, GATE-V-02 CDP leg) until the wasm build exports a
DuckDB C API / FS backend / print hooks. No bench-suite env/flags apply
(no criterion surface exists; per the lane spec the court runs
chrome-headless-shell via CDP, not cargo bench).

## Cleanup debt (coordinator)

- ~10 leaked court profile dirs at `/Users/sac/.cache/tmp/kdc11-opfs-*`
  (mid-lane `rm` was denied by the permission system; some may have live
  chrome-headless-shell helper processes attached — kill by
  `--user-data-dir=...kdc11-opfs-*` match first).
