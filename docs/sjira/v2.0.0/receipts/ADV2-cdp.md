# ADV2 — CDP smoke court receipt (KD-11 harness skeleton)

Date: 2026-10-19 · Repo: /Users/sac/frozen-duckdb · Lane: ADV2 · Standing: PASS

## Subject

- `scripts/doctor/cdp_smoke.mjs` (new, court-owned) — self-contained CDP control-loop
  smoke: launch → connect → navigate → evaluate → teardown. No puppeteer dep; raw
  `WebSocket` global (Node v26.10.0).

## Browser acquisition

`chrome-headless-shell` not on PATH; installed to /tmp via:

```
npx -y @puppeteer/browsers install chrome-headless-shell@stable --path /tmp/chs-court
# chrome-headless-shell@155.0.8059.84, exit 0
```

Binary:
`/tmp/chs-court/chrome-headless-shell/mac_arm-155.0.8059.84/chrome-headless-shell-mac-arm64/chrome-headless-shell`

## Real run (exit 0)

```json
{
  "court": "cdp_smoke",
  "standing": "PASS",
  "launch":  { "port": 57652, "pid": 15742, "devtoolsActivePort": true },
  "version": { "Browser": "HeadlessChrome/155.0.8059.84", "Protocol": "1.3" },
  "connect": { "browserWS": "ws://127.0.0.1:57652/devtools/browser/0bbee2ba-...", "open": true },
  "navigate": { "url": "data:text/html,...SHIM-COURT cdp smoke...", "loadEventFired": true },
  "evaluate": { "typeofWebAssembly": "object", "moduleExportsType": "function",
                "title": "cdp-smoke", "marker": "SHIM-COURT cdp smoke" },
  "teardown": { "exit": { "code": 0, "sig": null }, "tmpdirRemoved": true }
}
```

WASM probes: `typeof WebAssembly === "object"`,
`typeof WebAssembly.Module.exports === "function"` — engine present, no wasm payload
required yet (this is the mount point for the OPFS courts).

## Loud-skip path (proven, not asserted)

```
$ CDP_SMOKE_BIN=/nonexistent/chrome node scripts/doctor/cdp_smoke.mjs
[SKIP] chrome-headless-shell unavailable — set CDP_SMOKE_BIN or run: npx -y @puppeteer/browsers install chrome-headless-shell@stable --path /tmp/chs-court
EXIT=2
```

Exit codes: 0 PASS · 2 SKIP-LOUD · 1 FAIL. Discovery is directory-typed
(`.metadata` file in the cache root is skipped, ENOTDIR-proofed after first run).

## Teardown proof

`Browser.close` → child process exit `{code: 0}` observed via bounded poll (no
fixed sleeps); temp user-data-dir removed and verified absent.

## Notes

- Launch uses `--remote-debugging-port=0` (OS-assigned), port read from the
  browser-written `DevToolsActivePort` file — no port guessing.
- All waits are deadline-bounded polls on real observables (no-sleep doctrine).
- No git operations performed (lane law). Court owns only the two assigned files.
