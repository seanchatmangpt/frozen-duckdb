#!/usr/bin/env node
// SP2 — ABI co-pinning: stamp + verify a custom WASM section named `abi_receipt`.
//
// Section payload (UTF-8 JSON, deterministic — no timestamps):
//   { "abi_receipt_v": 1, "digest_alg": "sha256", "digest": "<hex>",
//     "pin": "<upstream sha>", "engine_version": "<describe>" }
//
// Canonical form: the module with ALL `abi_receipt` custom sections removed.
// The recorded digest is sha256 of that canonical form, so the pin attests the
// exact module code: any code mutation OR receipt tampering flips the verdict.
//
// Verifier fails CLOSED, typed:
//   exit 1, stderr `ABI_MISMATCH: <reason>` on section absent / duplicate,
//   malformed payload, pin differs, engine_version differs (when checked),
//   or module digest differs from the recorded digest.
//   exit 0 only on a full pass. exit 2 = usage/IO (not a verdict).
//
// SHIM-COURT: negative-mutation court lives in ./abi_court.mjs — every fail
// path must be proven flippable; a gate that cannot refuse is vacuous.
import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync } from 'node:fs';

const WASM_HEADER = Buffer.from([0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]);
const SECTION_NAME = 'abi_receipt';
const RECEIPT_VERSION = 1;

const sha256hex = (buf) => createHash('sha256').update(buf).digest('hex');

// ---- LEB128 / section framing ----
function readU32Leb(buf, pos) {
  let result = 0, shift = 0, byte;
  for (let i = 0; i < 5; i++) {
    if (pos >= buf.length) throw Framing('truncated LEB128');
    byte = buf[pos++];
    result |= (byte & 0x7f) << shift; // sizes < 2^32; >>> not needed for our magnitudes
    if ((byte & 0x80) === 0) return [result >>> 0, pos];
    shift += 7;
  }
  throw new FramingError('LEB128 u32 exceeds 5 bytes');
}
class FramingError extends Error {}
const Framing = (m) => new FramingError(m);

function u32Leb(value) {
  const out = [];
  do { let b = value & 0x7f; value >>>= 7; if (value !== 0) b |= 0x80; out.push(b); } while (value !== 0);
  return Buffer.from(out);
}

// Parse top-level sections. Custom sections carry their decoded name.
function parseSections(bytes) {
  if (bytes.length < 8 || !bytes.subarray(0, 8).equals(WASM_HEADER)) {
    throw Framing('not a wasm module (bad magic/version)');
  }
  const sections = [];
  let pos = 8;
  while (pos < bytes.length) {
    const idPos = pos;
    const id = bytes[pos++];
    const [size, afterSize] = readU32Leb(bytes, pos);
    const bodyStart = afterSize;
    const bodyEnd = bodyStart + size;
    if (bodyEnd > bytes.length) throw Framing(`section id=${id} at ${idPos} truncated (${size} > ${bytes.length - bodyStart})`);
    let name = null;
    if (id === 0) {
      const [nameLen, nameStart] = readU32Leb(bytes, bodyStart);
      if (nameStart + nameLen > bodyEnd) throw Framing('custom section name truncated');
      name = bytes.subarray(nameStart, nameStart + nameLen).toString('utf8');
    }
    sections.push({ id, name, idPos, bodyStart, bodyEnd });
    pos = bodyEnd;
  }
  return sections;
}

function customPayload(bytes, sec) {
  const [nameLen, nameStart] = readU32Leb(bytes, sec.bodyStart);
  return bytes.subarray(nameStart + nameLen, sec.bodyEnd);
}

function encodeCustomSection(name, payload) {
  const nameBytes = Buffer.from(name, 'utf8');
  const body = Buffer.concat([u32Leb(nameBytes.length), nameBytes, payload]);
  return Buffer.concat([Buffer.from([0x00]), u32Leb(body.length), body]);
}

// Canonical form: drop every `abi_receipt` custom section, splice bytes directly.
function canonicalForm(bytes) {
  const sections = parseSections(bytes);
  const parts = [bytes.subarray(0, 8)];
  for (const s of sections) {
    if (s.id === 0 && s.name === SECTION_NAME) continue;
    parts.push(bytes.subarray(s.idPos, s.bodyEnd));
  }
  return Buffer.concat(parts);
}

function findReceiptSections(bytes) {
  return parseSections(bytes).filter((s) => s.id === 0 && s.name === SECTION_NAME);
}

// ---- CLI ----
function parseArgs(argv) {
  const args = {};
  for (let i = 0; i < argv.length; i += 2) {
    const key = argv[i];
    if (!key.startsWith('--')) usage();
    args[key.slice(2)] = argv[i + 1];
  }
  return args;
}
function usage() {
  console.error(`usage:
  abi_receipt.mjs stamp  --in <wasm> --out <stamped> --pin <sha> --engine-version <v>
  abi_receipt.mjs verify --in <wasm> [--pin <sha>] [--engine-version <v>]
  abi_receipt.mjs strip  --in <wasm> --out <out>
  abi_receipt.mjs dump   --in <wasm>`);
  process.exit(2);
}

function main() {
  const [cmd] = process.argv.slice(2);
  const args = parseArgs(process.argv.slice(3));
  const readIn = () => { if (!args.in) usage(); return readFileSync(args.in); };

  if (cmd === 'stamp') {
    const { out, pin, 'engine-version': engineVersion } = args;
    if (!out || !pin || !engineVersion) usage();
    const input = readIn();
    const canonical = canonicalForm(input);
    const payload = Buffer.from(JSON.stringify({
      abi_receipt_v: RECEIPT_VERSION,
      digest_alg: 'sha256',
      digest: sha256hex(canonical),
      pin,
      engine_version: engineVersion,
    }), 'utf8');
    const stamped = Buffer.concat([
      input.subarray(0, 8),          // header
      encodeCustomSection(SECTION_NAME, payload), // receipt first — custom sections are legal anywhere
      input.subarray(8),
    ]);
    parseSections(stamped); // re-validate framing before writing (fail-closed stamper)
    writeFileSync(out, stamped);
    console.log(`[abi-receipt] stamped ${out} bytes=${stamped.length} pin=${pin} engine=${engineVersion} digest=${sha256hex(canonical)}`);
    return;
  }

  if (cmd === 'verify') {
    const input = readIn();
    const secs = findReceiptSections(input);
    if (secs.length === 0) failMismatch('section absent');
    if (secs.length > 1) failMismatch(`duplicate ${SECTION_NAME} sections (${secs.length})`);
    let receipt;
    try { receipt = JSON.parse(customPayload(input, secs[0]).toString('utf8')); }
    catch (e) { failMismatch(`malformed payload: ${e.message}`); }
    if (receipt.abi_receipt_v !== RECEIPT_VERSION) failMismatch(`unsupported abi_receipt_v ${receipt.abi_receipt_v}`);
    if (receipt.digest_alg !== 'sha256') failMismatch(`unsupported digest_alg ${receipt.digest_alg}`);
    if (args.pin !== undefined && receipt.pin !== args.pin) {
      failMismatch(`pin differs (expected ${args.pin}, recorded ${receipt.pin})`);
    }
    if (args['engine-version'] !== undefined && receipt.engine_version !== args['engine-version']) {
      failMismatch(`engine_version differs (expected ${args['engine-version']}, recorded ${receipt.engine_version})`);
    }
    const recomputed = sha256hex(canonicalForm(input));
    if (recomputed !== receipt.digest) {
      failMismatch(`module digest differs (recomputed ${recomputed} != recorded ${receipt.digest})`);
    }
    console.log(`[abi-receipt] PASS in=${args.in} pin=${receipt.pin} engine=${receipt.engine_version} digest=${receipt.digest} bytes=${input.length}`);
    return;
  }

  if (cmd === 'strip') {
    const { out } = args;
    if (!out) usage();
    const stripped = canonicalForm(readIn());
    writeFileSync(out, stripped);
    console.log(`[abi-receipt] stripped ${out} bytes=${stripped.length}`);
    return;
  }

  if (cmd === 'dump') {
    const input = readIn();
    const secs = findReceiptSections(input);
    if (secs.length === 0) failMismatch('section absent');
    console.log(customPayload(input, secs[0]).toString('utf8'));
    return;
  }

  usage();
}

function failMismatch(reason) {
  console.error(`ABI_MISMATCH: ${reason}`);
  process.exit(1);
}

main();
