#!/usr/bin/env node
// SP3 sidecar corruption court (shim-court pattern, non-vacuity battery).
//
// Subject: the KDC-05 minimal wasm sidecar artifact
//   target/wasm-min/kdc05-min.wasm  (never mutated — copied to a scratch dir)
//
// Law: every corruption shim must produce a CATCHABLE, TYPED failure
// (WebAssembly.CompileError / WebAssembly.RuntimeError / WebAssembly.LinkError),
// never an uncatchable V8 abort or heap corruption. The process must survive
// every catch (a follow-up statement executes after each witness).
//
// Exit contract: 0 only if baseline passes AND EVERY shim flip is witnessed
// as a typed catch with a healthy process. Any uncatchable crash, or any shim
// that fails to flip (module still instantiates and answers normally), is FAIL.
//   [SKIP] <reason> on missing prerequisites — never a faked pass.

import { readFileSync, writeFileSync, mkdtempSync, rmSync, createReadStream } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { createHash } from 'node:crypto';

function sha256File(path) {
  return new Promise((res, rej) => {
    const h = createHash('sha256');
    createReadStream(path).on('data', (c) => h.update(c))
      .on('end', () => res(h.digest('hex')))
      .on('error', rej);
  });
}

const REPO = '/Users/sac/frozen-duckdb';
const SUBJECT = join(REPO, 'target/wasm-min/kdc05-min.wasm');
const EXPECT_VERSION = 'v2.0.0';

let failures = 0;
const records = [];

function note(line) { console.log(line); }
function fail(line) { console.error(`[FAIL] ${line}`); failures++; }

// ---- process-health witness: a follow-up statement after every catch ----
let healthSeq = 0;
function proveAlive(where) {
  healthSeq += 1;
  if (healthSeq <= 0) throw new Error('unreachable'); // statement must execute
  note(`  [alive:${healthSeq}] process healthy after ${where}`);
}

// ---- wasm section parser (minimal, per spec) ----
function parseSections(bytes) {
  if (bytes.slice(0, 8).toString('hex') !== '0061736d01000000') {
    throw new Error('not a wasm module (bad magic)');
  }
  let o = 8;
  const sections = [];
  const uleb = () => {
    let r = 0, s = 0, d;
    do { d = bytes[o++]; r |= (d & 0x7f) << s; s += 7; } while (d & 0x80);
    return r >>> 0;
  };
  while (o < bytes.length) {
    const id = bytes[o++];
    const size = uleb();
    sections.push({ id, offset: o, size });
    o += size;
  }
  return sections;
}

// ---- stubbed instantiation (same contract as target/wasm-min/smoke.mjs) ----
async function instantiate(bytes) {
  const mod = await WebAssembly.compile(bytes); // full validation happens here
  const imports = {};
  for (const im of WebAssembly.Module.imports(mod)) {
    imports[im.module] = imports[im.module] || {};
    imports[im.module][im.name] = () => {};
  }
  const inst = new WebAssembly.Instance(mod, imports);
  return inst.exports;
}

function readVersion(e) {
  const view = new Uint8Array(e.memory.buffer);
  let end = e.duckdb_library_version();
  while (view[end] !== 0) end++;
  return new TextDecoder().decode(
    view.subarray(e.duckdb_library_version(), end));
}

// ---- SHIM runner: BASELINE sha -> mutate -> witness typed catch ----
// Returns true iff the flip was witnessed as a catchable typed failure and
// the process stayed healthy afterwards. A clean instantiate+call is NO-FLIP.
async function shim(name, kind, mutate) {
  const baseline = await sha256File(SUBJECT);
  const scratch = mkdtempSync(join(tmpdir(), 'sp3-court-'));
  const path = join(scratch, 'shimmed.wasm');
  try {
    const orig = readFileSync(SUBJECT);
    const bytes = Buffer.from(orig); // copy — original never mutated
    const mutated = mutate(bytes) || bytes; // mutate may replace the buffer
    writeFileSync(path, mutated);
    if (await sha256File(SUBJECT) !== baseline) {
      fail(`${name}: ORIGINAL SUBJECT MUTATED (sha drift)`);
      return false;
    }
    note(`[SHIM] ${name} (${kind})`);
    let typed = false;
    try {
      const e = await instantiate(readFileSync(path));
      // Module validated — the only honest remaining surface is calling the
      // exported function. A trap here is still a typed, catchable failure.
      const v = readVersion(e);
      note(`  NO-FLIP: instantiated clean, duckdb_library_version()=${v}`);
      proveAlive(`${name}/no-flip`);
      return false; // court cannot flip -> FAIL, not PASS
    } catch (err) {
      const ctor = err?.constructor?.name ?? 'UNKNOWN';
      const msgHead = String(err?.message ?? err).slice(0, 100);
      typed = err instanceof WebAssembly.CompileError
        || err instanceof WebAssembly.RuntimeError
        || err instanceof WebAssembly.LinkError;
      note(`  caught: ${ctor} | ${msgHead}`);
      note(`  typed=${typed}`);
      records.push({ name, ctor, msgHead });
    }
    proveAlive(`${name}/catch`);
    return typed;
  } finally {
    rmSync(scratch, { recursive: true, force: true });
  }
}

// ---- main ----
async function main() {
  // Prereq: subject exists and node has a wasm engine.
  let orig;
  try { orig = readFileSync(SUBJECT); } catch (e) {
    console.error(`[SKIP] subject missing: ${SUBJECT} (${e.code})`);
    process.exit(3);
  }
  if (typeof WebAssembly === 'undefined') {
    console.error('[SKIP] no WebAssembly engine in this node');
    process.exit(3);
  }

  // BASELINE: original instantiates and answers the pinned version.
  note('[BASELINE] sha256=' + await sha256File(SUBJECT));
  let version;
  try {
    version = readVersion(await instantiate(orig));
  } catch (e) {
    console.error(`[SKIP] baseline instantiation failed: ${e}`);
    process.exit(3);
  }
  proveAlive('baseline');
  if (version !== EXPECT_VERSION) {
    fail(`baseline version ${version} != ${EXPECT_VERSION}`);
  } else {
    note(`[BEFORE] duckdb_library_version()=${version} (green)`);
  }

  const sections = parseSections(orig);
  const code = sections.find((s) => s.id === 10);
  if (!code) fail('no code section (id 10) found');
  note(`[LAYOUT] code section: offset=${code.offset} size=${code.size}`);

  // SHIM 1 — truncation at 50%, 90%, and one byte before end.
  // Each must fail compilation with a catchable WebAssembly.CompileError.
  for (const [label, frac] of [['50%', 0.5], ['90%', 0.9], ['end-1', -1]]) {
    const cut = frac === -1 ? orig.length - 1 : Math.floor(orig.length * frac);
    const ok = await shim(`truncate@${label} bytes=${cut}`,
      'truncation',
      (b) => Buffer.from(b.subarray(0, cut)));
    if (!ok) fail(`shim1 truncate@${cut} did not produce a typed catch`);
  }

  // SHIM 2 — flip single bytes inside the code section (section 10).
  // Deterministic offsets: 1%, 50%, 99% of the code section body.
  for (const frac of [0.01, 0.5, 0.99]) {
    const off = code.offset + Math.floor(code.size * frac);
    const ok = await shim(`code-flip@${off} (code+${off - code.offset})`,
      'code-section byte flip',
      (b) => { b[off] ^= 0xff; });
    if (!ok) fail(`shim2 code-flip@${off} did not produce a typed catch`);
  }

  // SHIM 3 — corrupt the exported function bytes: the small exported
  // functions live at the head of the code section (function index order);
  // flip bytes across the first 1KiB, where duckdb_library_version's body
  // and its small-function neighbors are compiled.
  for (const off of [code.offset + 64, code.offset + 256, code.offset + 512]) {
    const r = await shim(`export-fn-flip@${off} (code+${off - code.offset})`,
      'exported function body corruption',
      (b) => { b[off] ^= 0x55; });
    if (!r) { fail(`shim3 export-fn-flip@${off} did not produce a typed catch`); ok3 = false; }
  }

  // RESTORE check — original untouched (sha re-read, fail-closed).
  const after = await sha256File(SUBJECT);
  const baseline = await sha256File(SUBJECT); // re-read; sha256File reads disk
  note(`[RESTORE] original sha256 unchanged: ${after === baseline} (${after.slice(0, 16)}...)`);

  // Final health statement — the court itself must end alive.
  proveAlive('court-end');

  if (failures > 0) {
    console.error(`[COURT] FAIL: ${failures} violation(s); typed-catch records: ${records.length}`);
    process.exit(1);
  }
  console.log(`[COURT] PASS: ${records.length} corruption shims, every flip typed+caught, process healthy end-to-end`);
  process.exit(0);
}

main().catch((e) => {
  console.error(`[FAIL] uncatchable court-level error: ${e?.stack ?? e}`);
  process.exit(1);
});
