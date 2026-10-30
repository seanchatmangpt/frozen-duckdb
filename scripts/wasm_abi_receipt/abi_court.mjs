#!/usr/bin/env node
// SP2 — negative-mutation court for the abi_receipt verifier (SHIM-COURT).
// Shape: BASELINE -> BEFORE -> SHIM -> AFTER -> RESTORE, fail-closed.
// A shim that does NOT flip the verdict is VACUOUS GATE (court FAIL, not PASS).
// Shims operate on temp copies only; originals are re-hashed at RESTORE and
// must match BASELINE byte-exact or the court reports CORRUPTED.
//
// Shims (all marked SHIM-COURT):
//   A: corrupt the recorded digest inside the section  -> digest differs
//   B: strip the abi_receipt section                   -> section absent
//   C: re-stamp with a wrong pin                       -> pin differs
import { spawnSync } from 'node:child_process';
import { mkdtempSync, rmSync, readFileSync, writeFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { tmpdir } from 'node:os';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = dirname(fileURLToPath(import.meta.url));
const TOOL = join(HERE, 'abi_receipt.mjs');
const ROOT = '/Users/sac/frozen-duckdb';
const SUBJECT = process.argv[2] ?? join(ROOT, 'target/wasm-min/kdc05-min.wasm');
const STAMPED = process.argv[3] ?? join(ROOT, 'target/wasm-min/kdc05-min.stamped.wasm');
const PIN = 'd54cbbe3df17e8e5ce68312f483b4155836112aa';
const WRONG_PIN = '0'.repeat(40);
const ENGINE = 'v2.0.0';
const sha256hex = (p) => createHash('sha256').update(readFileSync(p)).digest('hex');

const tmp = mkdtempSync(join(tmpdir(), 'abi-court-'));
let failures = 0;
const run = (args) => spawnSync(process.execPath, [TOOL, ...args], { encoding: 'utf8' });

function after(shim, argv, expectSubstring) {
  const r = run(argv);
  const stderr = (r.stderr || '').trim();
  if (r.status === 0) {
    console.error(`[VACUOUS GATE] SHIM-COURT ${shim}: verify exited 0 — gate cannot refuse`);
    failures++;
    return;
  }
  if (r.status !== 1 || !stderr.startsWith('ABI_MISMATCH') || !stderr.includes(expectSubstring)) {
    console.error(`[court] SHIM-COURT ${shim}: wrong refusal (exit=${r.status}, stderr="${stderr}")`);
    failures++;
    return;
  }
  console.log(`[court] SHIM-COURT ${shim} FLIPPED: exit=${r.status} ${stderr}`);
}

try {
  // BASELINE
  const baseSubject = sha256hex(SUBJECT);
  const baseStamped = sha256hex(STAMPED);
  console.log(`[court] BASELINE subject=${baseSubject}`);
  console.log(`[court] BASELINE stamped=${baseStamped}`);

  // BEFORE: green on the promoted stamped artifact AND on a fresh self-stamp.
  let r = run(['verify', '--in', STAMPED, '--pin', PIN, '--engine-version', ENGINE]);
  if (r.status !== 0) { console.error(`[court] BEFORE failed on stamped artifact: exit=${r.status} ${r.stderr}`); failures++; }
  else console.log(`[court] BEFORE ok: ${r.stdout.trim()}`);
  const selfStamp = join(tmp, 'self.wasm');
  r = run(['stamp', '--in', SUBJECT, '--out', selfStamp, '--pin', PIN, '--engine-version', ENGINE]);
  if (r.status !== 0) { console.error(`[court] BEFORE self-stamp failed: ${r.stderr}`); failures++; }
  r = run(['verify', '--in', selfStamp, '--pin', PIN, '--engine-version', ENGINE]);
  if (r.status !== 0) { console.error(`[court] BEFORE verify of self-stamp failed: exit=${r.status} ${r.stderr}`); failures++; }
  else console.log(`[court] BEFORE ok (self-stamp): ${r.stdout.trim()}`);

  // SHIM A — corrupt one hex char of the recorded digest (same length, framing intact).
  const A = join(tmp, 'digest-corrupt.wasm');
  run(['stamp', '--in', SUBJECT, '--out', A, '--pin', PIN, '--engine-version', ENGINE]);
  let bytes = readFileSync(A);
  const marker = Buffer.from('"digest":"');
  const at = bytes.indexOf(marker);
  if (at < 0) { console.error('[court] SHIM-A setup failed: digest field not found'); failures++; }
  else {
    const d = at + marker.length;
    const orig = String.fromCharCode(bytes[d]);
    const flipped = orig === '0' ? '1' : '0';
    bytes[d] = flipped.charCodeAt(0);
    writeFileSync(A, bytes);
    console.log(`[court] SHIM-COURT A applied: digest[0] ${orig}->${flipped} in ${A}`);
    after('A', ['verify', '--in', A, '--pin', PIN], 'digest differs');
  }

  // SHIM B — strip the section entirely.
  const B = join(tmp, 'stripped.wasm');
  run(['strip', '--in', STAMPED, '--out', B]);
  console.log(`[court] SHIM-COURT B applied: abi_receipt section stripped -> ${B}`);
  after('B', ['verify', '--in', B, '--pin', PIN], 'section absent');

  // SHIM C — re-stamp the same module with a wrong pin.
  const C = join(tmp, 'wrong-pin.wasm');
  run(['stamp', '--in', SUBJECT, '--out', C, '--pin', WRONG_PIN, '--engine-version', ENGINE]);
  console.log(`[court] SHIM-COURT C applied: re-stamped with pin=${WRONG_PIN} -> ${C}`);
  after('C', ['verify', '--in', C, '--pin', PIN], 'pin differs');

  // RESTORE — fail-closed: originals must be untouched.
  if (sha256hex(SUBJECT) !== baseSubject) { console.error('[court] RESTORE: subject CORRUPTED'); failures++; }
  if (sha256hex(STAMPED) !== baseStamped) { console.error('[court] RESTORE: stamped artifact CORRUPTED'); failures++; }
  console.log(`[court] RESTORE verified: subject+stamped match BASELINE byte-exact`);
} finally {
  rmSync(tmp, { recursive: true, force: true });
}
if (failures > 0) { console.error(`[court] FAIL: ${failures} court failure(s)`); process.exit(1); }
console.log('[court] PASS: all shims flipped, restore byte-exact');
