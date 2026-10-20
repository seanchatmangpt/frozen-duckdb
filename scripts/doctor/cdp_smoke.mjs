#!/usr/bin/env node
// CDP smoke court (ADV2 / KD-11 harness skeleton).
// Proves the CDP control loop end-to-end on a REAL browser:
//   launch (remote-debugging-port) -> connect (raw WebSocket, no deps)
//   -> navigate (data: URL) -> Runtime.evaluate (WebAssembly probes)
//   -> teardown (Browser.close + child exit + tmpdir cleanup).
// No wasm payload is required yet — this is the mount point for the OPFS courts.
//
// Exit codes: 0 = PASS, 2 = SKIP-LOUD (browser unavailable), 1 = FAIL.
// Env: CDP_SMOKE_BIN  — explicit chrome-headless-shell binary path.
//      CDP_SMOKE_PORT — fixed debug port (default: OS-assigned via port 0).

import { spawn, execFile } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, existsSync, readdirSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, dirname } from "node:path";
import { setTimeout as sleep } from "node:timers/promises";

const REPO = "/Users/sac/frozen-duckdb";
const MARKER = "SHIM-COURT"; // court-owned artifact namespace

// --- locate browser --------------------------------------------------------
function findBrowser() {
  if (process.env.CDP_SMOKE_BIN) {
    // explicitly pinned and missing = loud skip, never a guess
    return existsSync(process.env.CDP_SMOKE_BIN) ? process.env.CDP_SMOKE_BIN : null;
  }
  const root = "/tmp/chs-court/chrome-headless-shell";
  if (!existsSync(root)) return null;
  for (const ver of dirEntries(root)) {
    for (const plat of dirEntries(join(root, ver))) {
      const bin = join(root, ver, plat, "chrome-headless-shell");
      if (existsSync(bin)) return bin;
    }
  }
  return null;
}

function dirEntries(p) {
  try { return readdirSync(p, { withFileTypes: true }).filter((d) => d.isDirectory()).map((d) => d.name); }
  catch { return []; }
}

const bin = findBrowser();
if (!bin) {
  console.error("[SKIP] chrome-headless-shell unavailable — set CDP_SMOKE_BIN or run: npx -y @puppeteer/browsers install chrome-headless-shell@stable --path /tmp/chs-court");
  process.exit(2);
}

// --- launch ----------------------------------------------------------------
const userDir = mkdtempSync(join(tmpdir(), "cdp-smoke-"));
const port = Number(process.env.CDP_SMOKE_PORT || 0); // 0 = OS-assigned
const args = [
  `--remote-debugging-port=${port}`,
  `--user-data-dir=${userDir}`,
  "--no-sandbox", "--disable-gpu", "--no-first-run",
  "--disable-extensions", "--disable-background-networking",
  "about:blank",
];
const child = spawn(bin, args, { stdio: ["ignore", "pipe", "pipe"] });
let exited = null;
child.on("exit", (code, sig) => { exited = { code, sig }; });
const stderrTail = [];
child.stderr.on("data", (d) => { stderrTail.push(String(d)); if (stderrTail.length > 20) stderrTail.shift(); });

async function devtoolsEndpoint() {
  const portFile = join(userDir, "DevToolsActivePort");
  const deadline = Date.now() + 20_000;
  while (Date.now() < deadline) {
    if (exited) throw new Error(`browser exited early: ${JSON.stringify(exited)} stderr=${stderrTail.join("")}`);
    if (existsSync(portFile)) {
      const [actualPort] = readFileSync(portFile, "utf8").split("\n");
      return { port: Number(actualPort), file: portFile };
    }
    await sleep(100);
  }
  throw new Error(`no DevToolsActivePort within 20s (stderr: ${stderrTail.join("")})`);
}

// --- bounded poll helper (no fixed sleeps as coordination) -----------------
async function poll(fn, ms, what) {
  const deadline = Date.now() + ms;
  for (;;) {
    try { return await fn(); } catch (e) {
      if (Date.now() > deadline) throw new Error(`timeout (${ms}ms) waiting for ${what}: ${e.message}`);
      await sleep(100);
    }
  }
}

// --- CDP over raw WebSocket ------------------------------------------------
class CDP {
  constructor(ws) { this.ws = ws; this.id = 0; this.pending = new Map(); this.listeners = [];
    ws.addEventListener("message", (ev) => {
      const msg = JSON.parse(ev.data);
      if (msg.id && this.pending.has(msg.id)) {
        const { resolve, reject } = this.pending.get(msg.id);
        this.pending.delete(msg.id);
        msg.error ? reject(new Error(JSON.stringify(msg.error))) : resolve(msg.result);
      } else if (msg.method) {
        for (const l of this.listeners) l(msg);
      }
    });
  }
  send(method, params = {}, sessionId) {
    const id = ++this.id;
    const payload = { id, method, params };
    if (sessionId) payload.sessionId = sessionId;
    return new Promise((resolve, reject) => {
      this.pending.set(id, { resolve, reject });
      this.ws.send(JSON.stringify(payload));
    });
  }
  on(fn) { this.listeners.push(fn); }
  close() { this.ws.close(); }
}

const receipts = {};
try {
  const { port: actualPort, file: portFile } = await devtoolsEndpoint();
  receipts.launch = { binary: bin, port: actualPort, devtoolsActivePort: portFile, pid: child.pid };

  const version = await poll(async () => {
    const res = await fetch(`http://127.0.0.1:${actualPort}/json/version`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }, 10_000, "GET /json/version");
  receipts.version = { Browser: version.Browser, Protocol: version["Protocol-Version"] };

  const ws = new WebSocket(version.webSocketDebuggerUrl);
  await new Promise((res, rej) => { ws.addEventListener("open", res); ws.addEventListener("error", rej); });
  const cdp = new CDP(ws);
  receipts.connect = { browserWS: version.webSocketDebuggerUrl, open: true };

  const { targetId } = await cdp.send("Target.createTarget", { url: "about:blank" });
  const { sessionId } = await cdp.send("Target.attachToTarget", { targetId, flatten: true });

  await cdp.send("Page.enable", {}, sessionId);
  const loaded = new Promise((res, rej) => {
    const t = setTimeout(() => rej(new Error("Page.loadEventFired timeout")), 15_000);
    cdp.on((m) => { if (m.sessionId === sessionId && m.method === "Page.loadEventFired") { clearTimeout(t); res(); } });
  });
  const DATA_URL = `data:text/html,${encodeURIComponent("<title>cdp-smoke</title><h1 id=h>SHIM-COURT cdp smoke</h1>")}`;
  await cdp.send("Page.navigate", { url: DATA_URL }, sessionId);
  await loaded;
  receipts.navigate = { url: DATA_URL, loadEventFired: true };

  const evalExpr = async (expression) => {
    const r = await cdp.send("Runtime.evaluate", { expression, returnByValue: true }, sessionId);
    if (r.exceptionDetails) throw new Error(`evaluate failed: ${JSON.stringify(r.exceptionDetails)}`);
    return r.result.value;
  };

  const typeofWA = await evalExpr("typeof WebAssembly");
  const exportsFn = await evalExpr("typeof WebAssembly.Module.exports");
  const title = await evalExpr("document.title");
  const marker = await evalExpr("document.getElementById('h').textContent");

  receipts.evaluate = { typeofWebAssembly: typeofWA, moduleExportsType: exportsFn, title, marker };
  const pass =
    typeofWA === "object" &&
    exportsFn === "function" &&
    title === "cdp-smoke" &&
    marker.includes("SHIM-COURT");

  // --- teardown ------------------------------------------------------------
  await cdp.send("Browser.close").catch(() => {});
  cdp.close();
  const exit = await poll(async () => {
    if (exited) return exited;
    throw new Error("still running");
  }, 10_000, "browser process exit after Browser.close");
  rmSync(userDir, { recursive: true, force: true });

  receipts.teardown = { exit, tmpdirRemoved: !existsSync(userDir) };
  console.log(JSON.stringify({ court: "cdp_smoke", standing: pass ? "PASS" : "FAIL", receipts }, null, 2));
  process.exit(pass ? 0 : 1);
} catch (e) {
  receipts.error = e.message;
  child.kill("SIGKILL");
  rmSync(userDir, { recursive: true, force: true });
  console.error(JSON.stringify({ court: "cdp_smoke", standing: "FAIL", receipts }, null, 2));
  process.exit(1);
}
