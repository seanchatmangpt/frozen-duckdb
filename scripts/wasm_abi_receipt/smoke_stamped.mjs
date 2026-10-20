// SP2 — smoke: the STAMPED artifact must still be a valid, instantiable wasm
// module whose real version function returns the pinned engine version.
// Real collaborator (node WebAssembly), assert on final state — no mocks.
import { readFileSync } from 'node:fs';
const WASM = process.argv[2];
const EXPECT = process.argv[3] || 'v2.0.0-dev88595';
if (!WASM) { console.error('[smoke] usage: smoke_stamped.mjs <wasm> [expected-version]'); process.exit(2); }

const bytes = readFileSync(WASM);
const mod = new WebAssembly.Module(bytes); // throws on invalid module
const names = WebAssembly.Module.exports(mod).map((e) => e.name);
if (!names.includes('duckdb_library_version')) {
  console.error('[smoke] FAIL: duckdb_library_version missing from exports');
  process.exit(1);
}
// Stub every import (WASI surface): undefined coerces to 0 for i32 results.
const imports = {};
for (const im of WebAssembly.Module.imports(mod)) {
  (imports[im.module] ??= {})[im.name] = () => {};
}
const e = new WebAssembly.Instance(mod, imports).exports;
const verPtr = e.duckdb_library_version();
const view = new Uint8Array(e.memory.buffer);
let end = verPtr; while (view[end] !== 0) end++;
const version = new TextDecoder().decode(view.subarray(verPtr, end));
console.log(`[smoke] ${WASM} duckdb_library_version() = ${version}`);
if (version !== EXPECT) {
  console.error(`[smoke] FAIL: version ${version} != expected ${EXPECT}`);
  process.exit(1);
}
console.log('[smoke] PASS');
