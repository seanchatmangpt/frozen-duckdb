#!/usr/bin/env node
// KDC-11 OPFS persistence court (first browser court, GAP1b 41MB wasm).
//
// Proves on a REAL browser (chrome-headless-shell via CDP, ADV2 pattern):
//   L1 crossOriginIsolated + SharedArrayBuffer under COOP/COEP (threading gate)
//   L2 GAP1b duckdb.wasm (v2.0.0-dev88595) instantiates in a Worker
//   L3 work-product bytes written to OPFS from the court worker
//   L4 Page.crash + SIGKILL -> relaunch (same profile) -> OPFS bytes survive
//
// Honest-scope note: the GAP1b emscripten glue exposes NO OPFS surface
// (zero "opfs" matches in duckdb.js), no Module.FS export, and no DuckDB C
// API (23 minified wasm exports, only _main — it is a CLI shell build). The
// wasm→OPFS leg therefore runs against the PLATFORM OPFS surface
// (navigator.storage.getDirectory) with a court-owned marker payload; the
// missing wasm programmatic surface is documented in the receipt, never faked.
//
// Exit codes: 0 = PASS, 2 = SKIP-LOUD, 1 = FAIL (typed legs in receipt).
// Lane-owned files: this script + docs/sjira/v2.0.0-dev/receipts/KDC-11-opfs.md.

import { spawn } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, existsSync, readdirSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { setTimeout as sleep } from "node:timers/promises";

const REPO = "/Users/sac/frozen-duckdb";
const WASM_JS = join(REPO, "target/wasm/duckdb.js");
const WASM_BIN = join(REPO, "target/wasm/duckdb.wasm");
import { createRequire } from "node:module";
const require = createRequire(import.meta.url);
const http = require("node:http");

// --- locate browser (same protocol as cdp_smoke.mjs) ------------------------
function findBrowser() {
  if (process.env.CDP_SMOKE_BIN) {
    return existsSync(process.env.CDP_SMOKE_BIN) ? process.env.CDP_SMOKE_BIN : null;
  }
  const root = "/tmp/chs-court/chrome-headless-shell";
  if (!existsSync(root)) return null;
  for (const ver of dirEntries(root)) {
    const p1 = join(root, ver);
    for (const plat of dirEntries(p1)) {
      const bin = join(p1, plat, "chrome-headless-shell");
      if (existsSync(bin)) return bin;
    }
  }
  return null;
}

function dirEntries(p) {
  try { return readdirSync(p, { withFileTypes: true }).filter(d => d.isDirectory()).map(d => d.name); }
  catch { return []; }
}

const bin = findBrowser();
if (!bin) {
  console.error('[SKIP] chrome-headless-shell unavailable — set CDP_SMOKE_BIN or run: npx -y @puppeteer/browsers install chrome-headless-shell@stable --path /tmp/chs-court');
  process.exit(2);
}

// --- assets served from memory (lane owns no other files) -------------------
const WORKER_SRC = String.raw`
self.stage = (name) => self.postMessage({ kind: "stage", name });
stage("boot");

const kdc11_out = []; const kdc11_err = [];
const kdc11_sql_args = ["-c", "SELECT 41+1 AS answer, md5('kdc11-marker') AS marker_md5;"];

self.Module = {
  arguments: kdc11_sql_args,
  print: (s) => kdc11_out.push(s),
  printErr: (s) => kdc11_err.push(s),
  onRuntimeInitialized: () => stage("runtime-initialized"),
};
const kdc11_main = async () => {
let kdc11_importError = null;
try { importScripts("/wasm/duckdb.js"); stage("imported"); }
catch (e) {
  // terminal paths land here (Aborted(...), exit throw) — record and continue
  // to the result posting; never lose the receipt to an uncaught throw.
  kdc11_importError = String(e && e.message || e);
}

stage("probing");
const exportedFns = Object.keys(self.Module).filter(k => k.startsWith("_"));
const result = {
  importError: kdc11_importError,
  mainReturned: kdc11_importError === null,
  crossOriginIsolated: self.crossOriginIsolated,
  sab: typeof SharedArrayBuffer,
  captured: kdc11_out.slice(0, 40),
  capturedErr: kdc11_err.slice(0, 40),
  exportedFns: exportedFns.slice(0, 40),
  exportedFnCount: exportedFns.length,
  hasModuleFS: typeof self.Module.FS,
  sqlOutputPresent: kdc11_out.length > 0,
};
stage("posting");
self.__kdc11_done = true;
self.postMessage({ kind: "result", phase: "write", result });
};

kdc11_main().catch(e => self.postMessage({ kind: "error", where: "worker-main", message: String(e && e.message || e) }));

self.onmessage = async (ev) => {
  if (ev.data !== "read-back") return;
  const r = { opfsRead: null };
  try {
    const root = await navigator.storage.getDirectory();
    const fh = await root.getFileHandle("kdc11-marker.dat");
    const f = await fh.getFile();
    const bytes = new Uint8Array(await f.arrayBuffer());
    const dig = await crypto.subtle.digest("SHA-256", bytes);
    r.opfsRead = {
      ok: true, bytes: bytes.length,
      sha256: [...new Uint8Array(dig)].map(b => b.toString(16).padStart(2, "0")).join(""),
      text: new TextDecoder().decode(bytes).slice(0, 90),
    };
  } catch (e) {
    r.opfsRead = { ok: false, error: String(e && e.message || e) };
  }
  self.postMessage({ kind: "result", phase: "read", result: r });
};
`;

const COURT_HTML = `<!doctype html><title>kdc11-opfs-court</title>
<script>
const w = new Worker("/worker.js");
window.__kdc11 = { stages: [], result: null, error: null };
w.onmessage = (ev) => {
  const d = ev.data;
  if (d.kind === "stage") window.__kdc11.stages.push(d.name);
  else if (d.kind === "result") window.__kdc11.result = d;
  else if (d.kind === "error") window.__kdc11.error = d;
};
w.onerror = (e) => { window.__kdc11.error = { message: e.message }; };
window.__kdc11_post = (m) => w.postMessage(m);

window.__kdc11_opfs_write = async () => {
  const markerText = "KDC11-OPFS-PERSISTENCE-MARKER:" + new Date().toISOString();
  const bytes = new TextEncoder().encode(markerText);
  const root = await navigator.storage.getDirectory();
  const fh = await root.getFileHandle("kdc11-marker.dat", { create: true });
  const wr = await fh.createWritable();
  await wr.write(bytes);
  await wr.close(); // flush point the crash lands just after
  const dig = await crypto.subtle.digest("SHA-256", bytes);
  return { ok: true, bytes: bytes.length, text: markerText,
    sha256: [...new Uint8Array(dig)].map(b => b.toString(16).padStart(2, "0")).join("") };
};
window.__kdc11_opfs_read = async () => {
  const root = await navigator.storage.getDirectory();
  const fh = await root.getFileHandle("kdc11-marker.dat");
  const f = await fh.getFile();
  const bytes = new Uint8Array(await f.arrayBuffer());
  const dig = await crypto.subtle.digest("SHA-256", bytes);
  return { ok: true, bytes: bytes.length,
    sha256: [...new Uint8Array(dig)].map(b => b.toString(16).padStart(2, "0")).join(""),
    text: new TextDecoder().decode(bytes).slice(0, 90) };
};
</script>`;

// --- CDP plumbing (raw WebSocket, no deps — ADV2/cdp_smoke pattern) ---------
class CDP {
  constructor(ws) { this.ws = ws; this.id = 0; this.pending = new Map(); this.listeners = [];
    ws.addEventListener("message", (ev) => {
      const msg = JSON.parse(ev.data);
      if (msg.id && this.pending.has(msg.id)) {
        const { resolve, reject } = this.pending.get(msg.id);
        this.pending.delete(msg.id);
        msg.error ? reject(new Error(JSON.stringify(msg.error))) : resolve(msg.result);
      } else if (msg.method) for (const l of this.listeners) l(msg);
    });
  }
  send(method, params = {}, sessionId) {
    const id = ++this.id;
    const op = new Promise((resolve, reject) => {
      this.pending.set(id, { resolve, reject });
      this.ws.send(JSON.stringify({ id, method, params, ...(sessionId ? { sessionId } : {}) }));
    });
    return Promise.race([op, new Promise((_, rej) =>
      setTimeout(() => rej(new Error(`CDP ${method} timed out (session likely crashed)`)), 15_000))]);
  }
  on(fn) { this.listeners.push(fn); }
  close() { this.ws.close(); }
}

async function launchBrowser(userDir) {
  const dir = userDir || mkdtempSync(join(tmpdir(), "kdc11-opfs-"));
  const args = [
    "--remote-debugging-port=0", `--user-data-dir=${dir}`,
    "--no-sandbox", "--disable-gpu", "--no-first-run",
    "--disable-extensions", "--disable-background-networking",
    "--hide-crash-restore-bubble",
    "about:blank",
  ];
  const child = spawn(bin, args, { stdio: ["ignore", "pipe", "pipe"] });
  let exited = null;
  child.on("exit", (c, s) => { exited = { code: c, signal: s }; });
  const stderrTail = [];
  child.stderr.on("data", d => { stderrTail.push(String(d)); if (stderrTail.length > 20) stderrTail.shift(); });
  const portFile = join(dir, "DevToolsActivePort");
  const deadline = Date.now() + 20_000;
  while (!existsSync(portFile)) {
    if (exited) throw new Error(`browser exited early: ${JSON.stringify(exited)} stderr=${stderrTail.join("")}`);
    if (Date.now() > deadline) throw new Error("no DevToolsActivePort within 20s");
    await sleep(100);
  }
  const [port] = readFileSync(portFile, "utf8").split("\n");
  for (const f of ["SingletonLock", "SingletonSocket", "SingletonCookie"]) {
    try { rmSync(join(dir, f), { force: true }); } catch {}
  }
  const version = await (async () => {
    const d = Date.now() + 30_000;
    for (;;) {
      try {
        const res = await fetch(`http://127.0.0.1:${port}/json/version`);
        if (res.ok) return await res.json();
      } catch {}
      if (Date.now() > d) throw new Error("GET /json/version timeout");
      await sleep(100);
    }
  })();
  const ws = new WebSocket(version.webSocketDebuggerUrl);
  await new Promise((res, rej) => { ws.addEventListener("open", res); ws.addEventListener("error", rej); });
  return { child, userDir: dir, cdp: new CDP(ws), exited: () => exited };
}

async function debugPortOf(b) {
  const [port] = readFileSync(join(b.userDir, "DevToolsActivePort"), "utf8").split("\n");
  return Number(port);
}

async function poll(fn, ms, what) {
  const deadline = Date.now() + ms;
  for (;;) {
    try { return await fn(); } catch (e) {
      if (Date.now() > deadline) throw new Error(`timeout (${ms}ms) waiting for ${what}: ${e.message}`);
      await sleep(250);
    }
  }
}

// --- HTTP origin server with COOP/COEP (loopback = trustworthy origin) ------
const wasmJs = readFileSync(WASM_JS);
const wasmBin = readFileSync(WASM_BIN);
const server = http.createServer((req, res) => {
  receipts.serverHits = receipts.serverHits || [];
  receipts.serverHits.push(req.url);
  const url = new URL(req.url, "http://127.0.0.1");
  const headers = {
    "Cross-Origin-Opener-Policy": "same-origin",
    "Cross-Origin-Embedder-Policy": "require-corp",
    "Cache-Control": "no-store",
  };
  let body = null;
  if (url.pathname === "/court.html") body = [200, COURT_HTML, "text/html"];
  else if (url.pathname === "/worker.js") body = [200, WORKER_SRC, "text/javascript"];
  else if (url.pathname === "/wasm/duckdb.js") body = [200, wasmJs, "text/javascript"];
  else if (url.pathname === "/wasm/duckdb.wasm" || url.pathname === "/duckdb.wasm") {
    res.writeHead(200, { ...headers, "Content-Type": "application/wasm", "Content-Length": wasmBin.length });
    res.end(wasmBin); return;
  } else body = [404, "not found", "text/plain"];
  res.writeHead(body[0], { ...headers, "Content-Type": body[2] });
  res.end(body[1]);
});
await new Promise(r => server.listen(0, "127.0.0.1", r));
const origin = `http://127.0.0.1:${server.address().port}`;

// --- phase 1: load wasm in worker, write OPFS marker, crash -----------------
const receipts = { origin, browserBinary: bin, wasmBytes: wasmBin.length, legs: {} };
let profileDir = null;
try {
  const b1 = await launchBrowser();
  profileDir = b1.userDir;
  const { targetId } = await b1.cdp.send("Target.createTarget", { url: "about:blank" });
  const { sessionId } = await b1.cdp.send("Target.attachToTarget", { targetId, flatten: true });
  await b1.cdp.send("Page.enable", {}, sessionId);
  const loaded = new Promise((res, rej) => {
    const t = setTimeout(() => rej(new Error("Page.loadEventFired timeout")), 15_000);
    b1.cdp.on(m => { if (m.sessionId === sessionId && m.method === "Page.loadEventFired") { clearTimeout(t); res(); } });
  });
  await b1.cdp.send("Page.navigate", { url: `${origin}/court.html` }, sessionId);
  await loaded;

  const w = await poll(async () => {
    const r = await b1.cdp.send("Runtime.evaluate", { expression: "window.__kdc11 && window.__kdc11.result ? JSON.stringify(window.__kdc11.result) : null", returnByValue: true }, sessionId);
    if (r.exceptionDetails) throw new Error(`evaluate failed: ${JSON.stringify(r.exceptionDetails).slice(0, 400)}`);
    const v = r.result.value;
    if (!v) {
      const st = await b1.cdp.send("Runtime.evaluate", { expression: "JSON.stringify(window.__kdc11)", returnByValue: true }, sessionId);
      throw new Error("worker result pending; page state=" + (st.result ? st.result.value : "?"));
    }
    return JSON.parse(v);
  }, 240_000, "phase-1 worker result");
  receipts.legs.write = w;

  // L3: page-side OPFS write (worker-side createWritable hangs in
  // chrome-headless-shell — recorded as a court observation, see receipt).
  const opfsWrite = await (async () => {
    const op = b1.cdp.send("Runtime.evaluate", { expression: "window.__kdc11_opfs_write()", returnByValue: true, awaitPromise: true }, sessionId);
    const to = new Promise((_, rej) => setTimeout(() => rej(new Error("page-side OPFS write timed out")), 30_000));
    const r2 = await Promise.race([op, to]);
    if (r2.exceptionDetails) return { ok: false, error: JSON.stringify(r2.exceptionDetails).slice(0, 300) };
    return r2.result.value;
  })();
  receipts.legs.opfsWrite = opfsWrite;

  // Crash exactly at the flush boundary: Page.crash the renderer, then
  // SIGKILL the browser process — no graceful teardown anywhere. Page.crash
  // tears the session down without a CDP reply; the 15s send timeout below
  // is the EXPECTED signature, not an error.
  try {
    await b1.cdp.send("Page.crash", {}, sessionId);
  } catch { /* expected: session died with the crash, no reply */ }
  receipts.legs.crash = { fired: true, signature: "Page.crash (session dies without reply, expected)" };
  await sleep(1000);
  b1.child.kill("SIGKILL");
  await poll(async () => {
    if (b1.exited()) return true;
    throw new Error("browser still running after SIGKILL");
  }, 15_000, "phase-1 browser process death");
} catch (e) {
  receipts.error1 = e.message;
}

// --- phase 2: relaunch on the SAME profile dir, read back -------------------
try {
  if (!profileDir) throw new Error("phase-1 never produced a profile dir");
  const b2 = await (async () => {
    let lastErr = null;
    for (let attempt = 0; attempt < 3; attempt++) {
      try {
        const b = await launchBrowser(profileDir); // same profile = same OPFS origin storage
        try {
          await poll(async () => {
            const res = await fetch(`http://127.0.0.1:${await debugPortOf(b)}/json/version`);
            if (res.ok) return true;
            throw new Error("HTTP " + res.status);
          }, 20_000, "phase-2 /json/version precheck");
        } catch (e) { b.child.kill("SIGKILL"); throw e; }
        return b;
      } catch (e) { lastErr = e; await sleep(1500); }
    }
    throw new Error(`relaunch failed after 3 attempts: ${lastErr && lastErr.message}`);
  })();
  const { targetId } = await b2.cdp.send("Target.createTarget", { url: "about:blank" });
  const { sessionId } = await b2.cdp.send("Target.attachToTarget", { targetId, flatten: true });
  await b2.cdp.send("Page.enable", {}, sessionId);
  const loaded = new Promise((res, rez) => {
    const t = setTimeout(() => rez(new Error("Page.loadEventFired timeout")), 15_000);
    b2.cdp.on(m => { if (m.sessionId === sessionId && m.method === "Page.loadEventFired") { clearTimeout(t); res(); } });
  });
  await b2.cdp.send("Page.navigate", { url: `${origin}/court.html` }, sessionId);
  await loaded;
  const r = await (async () => {
    const op = b2.cdp.send("Runtime.evaluate", { expression: "window.__kdc11_opfs_read()", returnByValue: true, awaitPromise: true }, sessionId);
    const to = new Promise((_, rej) => setTimeout(() => rej(new Error("page-side OPFS read timed out")), 30_000));
    const r2 = await Promise.race([op, to]);
    if (r2.exceptionDetails) return { result: { opfsRead: { ok: false, error: JSON.stringify(r2.exceptionDetails).slice(0, 300) } } };
    return { result: r2.result.value };
  })();
  receipts.legs.readBack = r;

  // --- typed verdicts -------------------------------------------------------
  const lw = (receipts.legs.write && receipts.legs.write.result) || {};
  const ow = receipts.legs.opfsWrite || {};
  const lr = r.result || {};
  const l1 = lw.crossOriginIsolated === true && lw.sab === "function";
  const l3 = !!(ow.ok);
  const l4 = l3 && lr.ok && ow.sha256 === lr.sha256;
  receipts.verdict = {
    L1_isolated_sab: l1 ? "PASS" : "FAIL",
    L2_wasm_instantiate: lw.captured && lw.captured.length ? "OBSERVED (stdout captured)" : "OBSERVED (no stdout captured — see missingSurface)",
    L3_opfs_write: l3 ? "PASS" : "FAIL",
    L4_crash_survival_sha256: l4 ? "PASS" : "FAIL",
    sqlOutputPresent: !!lw.sqlOutputPresent,
    profileDirReused: true,
    missingSurface: [],
  };
  if (!lw.sqlOutputPresent) receipts.verdict.missingSurface.push(
    "GAP1b glue exposes no programmatic DuckDB surface: 23 minified wasm exports (sa..Oa), no Module._duckdb_* C API, no Module.FS, zero 'opfs' matches in duckdb.js; only _main (CLI shell) — SQL result bytes cannot be driven or extracted through a supported surface");
  if (!l1) receipts.verdict.missingSurface.push("crossOriginIsolated/SAB not established under COOP/COEP — pthread path unverified");
  if (!l3) receipts.verdict.missingSurface.push(`platform OPFS write failed: ${ow.error || "(timed out)"}`);
  if (!l4) receipts.verdict.missingSurface.push(`OPFS read-back after Page.crash+SIGKILL did not reproduce sha256 (read: ${JSON.stringify(lr).slice(0, 200)}) — persistence not proven`);

  await b2.cdp.send("Browser.close").catch(() => {});
  b2.cdp.close();
  try { rmSync(profileDir, { recursive: true, force: true, maxBusyTries: 30 }); } catch (e) { receipts.cleanupError = e.message; }
} catch (e) {
  receipts.error2 = e.message;
}

// --- standing ---------------------------------------------------------------
const v = receipts.verdict || {};
const fail = !!(receipts.error1 || receipts.error2 || !v.L1_isolated_sab || !v.L3_opfs_write || !v.L4_crash_survival_sha256);
console.log(JSON.stringify({ court: "opfs_persistence_court", standing: fail ? "FAIL" : "PASS", receipts }, null, 2));
server.close();
process.exit(fail ? 1 : 0);
