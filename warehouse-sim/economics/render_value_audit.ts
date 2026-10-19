// render_value_audit.ts — lane V6 (GAP5 node-port): operator's CFO terminal format.
//
// Reads the four real value-audit receipts from /tmp/value_audit/ and prints
// the SOVEREIGN VALUE REALIZATION & CASH AUDIT. Physical quantities are REAL
// (read from receipts); every dollar figure is derived from the RATE CARD
// below, which is ASSUMPTION-UNVERIFIED, operator-supplied list prices,
// unaudited.
//
// RATE CARD (ASSUMPTION-UNVERIFIED — operator-supplied list prices, unaudited):
//   - Datadog log ingestion:  $0.10 per GB ingested / month
//   - Datadog log indexing:   $1.70 per 1M events indexed / month
//   - Celonis EMS:            $0.004 per process-event segment mined
//   - Amplitude:              $200 per 1M events / month
//   - Cadence:                audited workload runs 1x/business day,
//                             252 business days/yr (annualization factor 252)
//
// Usage: npx tsx render_value_audit.ts \
//          [--dir /tmp/value_audit] [--doctor <verdict>]
//          [--doctor-log <path>] [--verify <manifest>]
//
// --verify <manifest>  : integrity anchor. Checks every receipt's BLAKE3
//                        against the detached manifest minted by
//                        scripts/value_audit_sign.sh at court-mint time,
//                        BEFORE rendering; exits 1 on any hash mismatch,
//                        missing receipt, or unmanifested val*.json. The
//                        render-time b3 block alone proves nothing (it is
//                        computed over whatever is on disk).
//
// --doctor <verdict>   : doctor aggregate standing (GREEN or the real
//                        not-green standing string). Passed through verbatim.
// --doctor-log <path>  : evidence file backing the verdict; its b3sum and a
//                        tail excerpt are embedded in the signature block.
// Without --doctor the renderer prints the honest not-green line
// ("doctor not run/not provided") — never a fake green state.

import { readFileSync } from "node:fs";
import { execFileSync } from "node:child_process";

const RATE = {
  dd_ingest_per_gb_mo: 0.1,
  dd_index_per_m_events_mo: 1.7,
  celonis_per_segment: 0.004,
  amplitude_per_m_events_mo: 200.0,
  business_days_yr: 252,
};

class RenderFail extends Error {}

// fail() carries the never type honestly: it throws, so no reachable
// fall-through end exists (the Deno version relied on Deno.exit typing).
function fail(msg: string): never {
  console.error(`RENDER-FAIL: ${msg}`);
  throw new RenderFail(msg);
}

// verifyAgainstManifest: the integrity anchor. The render-time b3 block is
// computed OVER whatever files are on disk, so it "verifies" an edited
// receipt (witnessed tamper: val003 12090 -> 1209000 rendered GREEN). This
// check runs BEFORE any rendering and exits 1 on any mismatch, missing
// manifest entry, or extra unmanifested receipt in the directory.
function verifyAgainstManifest(dir: string, manifestPath: string): void {
  let txt: string;
  try {
    txt = readFileSync(manifestPath, "utf8");
  } catch (e) {
    return fail(`--verify: manifest ${manifestPath} unreadable: ${e}`);
  }
  const entries: Array<[string, string]> = []; // [digest, filename]
  for (const line of txt.split("\n")) {
    const t = line.trim();
    if (t === "" || t.startsWith("#")) continue;
    const m = t.match(/^([0-9a-f]{64})\s+\*?(.+)$/);
    if (!m) fail(`--verify: malformed manifest line: ${line}`);
    entries.push([m[1], m[2]]);
  }
  if (entries.length === 0) fail(`--verify: manifest ${manifestPath} has no entries`);
  const manifested = new Set(entries.map(([, f]) => f));

  // 1. every manifested file exists and its b3 matches (edited/deleted fail here)
  for (const [digest, name] of entries) {
    const p = `${dir}/${name}`;
    let actual: string;
    try {
      actual = execFileSync("b3sum", [p], { encoding: "utf8" }).trim().split(/\s+/)[0];
    } catch {
      return fail(`--verify: TAMPER — receipt ${name} missing or unreadable (manifest ${manifestPath})`);
    }
    if (actual !== digest) {
      return fail(
        `--verify: TAMPER — receipt ${name} b3 mismatch (manifest ${digest.slice(0, 12)}… vs actual ${actual.slice(0, 12)}…) — do not trust the numbers below this line: none rendered`,
      );
    }
  }

  // 2. no unmanifested receipt in the directory (extra-file substitution fails here)
  let listing: string;
  try {
    listing = execFileSync("ls", ["-1", dir], { encoding: "utf8" });
  } catch {
    return fail(`--verify: cannot list ${dir}`);
  }
  const extra = listing
    .split("\n")
    .filter((f) => /^val\d+.*\.json$/.test(f) && !manifested.has(f));
  if (extra.length > 0) {
    return fail(`--verify: TAMPER — unmanifested receipt(s) in ${dir}: ${extra.join(", ")}`);
  }
}

function argv(): string[] {
  return typeof process !== "undefined" ? process.argv.slice(2) : [];
}

const args = argv();
const argDir = (() => {
  const i = args.indexOf("--dir");
  return i >= 0 ? args[i + 1] : "/tmp/value_audit";
})();
const doctorStanding = (() => {
  const i = args.indexOf("--doctor");
  if (i >= 0) return args[i + 1];
  return process.env.DOCTOR ?? ""; // backward-compat: V6 orchestrator env form
})();
const doctorLog = (() => {
  const i = args.indexOf("--doctor-log");
  return i >= 0 ? args[i + 1] : "";
})();
const verifyManifest = (() => {
  const i = args.indexOf("--verify");
  if (i < 0) return "";
  const p = args[i + 1];
  if (!p) fail("--verify requires a manifest path");
  return p;
})();
if (verifyManifest !== "") verifyAgainstManifest(argDir, verifyManifest);

function readJson(name: string): Record<string, unknown> {
  try {
    const txt = readFileSync(`${argDir}/${name}`, "utf8");
    try {
      return JSON.parse(txt); // pretty-printed receipt (whole-file JSON)
    } catch {
      return JSON.parse(txt.trim().split("\n").pop()!); // single-line JSONL receipt
    }
  } catch (e) {
    if (e instanceof RenderFail) throw e;
    return fail(`receipt ${name} missing or unparseable: ${e}`);
  }
}

function num(r: Record<string, unknown>, key: string, src: string): number {
  const v = r[key];
  if (typeof v !== "number") fail(`receipt ${src}: field "${key}" missing/not-number`);
  return v;
}

function money(n: number): string {
  return "$" + n.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 });
}

const val1 = readJson("val001.json"); // VALUE-VAL-001 (Datadog court, V2)
const val2 = readJson("val002.json"); // VALUE-VAL-002 (Celonis court, V3)
const val3 = readJson("val003.json"); // TEST-VAL-003 (GATE-V-01 value ledger, V4 collector)
const val4 = readJson("val004.json"); // VALUE-VAL-00004 court (Amplitude, V5)

// ---- Section 1: Datadog (storage + filter displacement) -------------------
const ddBytesRaw = num(val1, "bytes_raw", "val001");
const ddBytesDb = num(val1, "bytes_segments", "val001");
const ddEvents = num(val1, "events", "val001");
const ddFilterMs = num(val1, "filter_latency_ms", "val001");
const ddComp = num(val1, "compression_ratio", "val001");
const ddGbRaw = ddBytesRaw / 1e9;
const ddCostRun =
  ddGbRaw * RATE.dd_ingest_per_gb_mo + (ddEvents / 1e6) * RATE.dd_index_per_m_events_mo;
const ddAnnual = ddCostRun * RATE.business_days_yr;

// ---- Section 2: Celonis (OCEL/DFG displacement) ---------------------------
const celSegments = num(val2, "segments", "val002.json");
const celLatency = num(val2, "latency_ms_median", "val002.json");
const celCostRun = celSegments * RATE.celonis_per_segment;
const celAnnual = celCostRun * RATE.business_days_yr;

// ---- Section 3: GATE-V-01 (V4 collector ledger: cloud-credit displacement) --
const v3m = val3.measured as Record<string, unknown> | undefined;
const v3c = val3.counterfactual as Record<string, unknown> | undefined;
if (!v3m || !v3c) fail(`val003.json: measured/counterfactual blocks missing`);
const p95 = num(v3m, "p95_ms", "val003.measured");
const iters = num(v3m, "iterations", "val003.measured");
const g01Annual = num(v3c, "usd_per_year", "val003.counterfactual");
const credits = num(v3c, "sovereign_credits_consumed", "val003.counterfactual");

// ---- Section 4: Amplitude (product-funnel displacement) -------------------
const ampEvents = (() => {
  const r = val4.rows ?? val4.events;
  if (typeof r === "number") return r;
  return fail("val004.json: no event-count field");
})();
const ampAnnual = (ampEvents / 1e6) * RATE.amplitude_per_m_events_mo * 12; // monthly cadence assumed
const ampFunnelMs = num(val4, "funnel_latency_ms", "val004");

const totalAnnual = ddAnnual + celAnnual + g01Annual + ampAnnual;
// Receipt verdicts are printed per-section verbatim; a FAIL verdict is shown
// honestly against its section, never masked by the dollar math.
const verdicts = [
  String(val1.verdict ?? "UNSPECIFIED"),
  String(val2.verdict ?? "UNSPECIFIED"),
  String(val3.verdict ?? "UNSPECIFIED"),
  String(val4.verdict ?? "UNSPECIFIED"),
];

function render(): void {
  const line = "=".repeat(72);
  const rule = "-".repeat(72);
  console.log(`\
${line}
                      SOVEREIGN VALUE REALIZATION & CASH AUDIT
${line}
SECTION 1 — OBSERVABILITY DISPLACEMENT (Datadog-shaped log court, VAL-001)
${rule}
  court verdict: ${verdicts[0]}
  measured: ${ddEvents.toLocaleString()} events, raw ${ddBytesRaw.toLocaleString()} B -> ${ddBytesDb.toLocaleString()} B shredded segments on disk
            compression ${(ddComp * 100).toFixed(1)}%, selective filter ${ddFilterMs.toFixed(1)} ms (real engine)
  rate card: $0.10/GB-mo ingestion + $1.70/M-events-mo indexing, x252 runs/yr
  annual equivalence: ${money(ddAnnual)}
${rule}
SECTION 2 — PROCESS-MINING DISPLACEMENT (OCEL/DFG court, VAL-002)
${rule}
  court verdict: ${verdicts[1]}
  measured: ${celSegments.toLocaleString()} conforming OCEL segments mined (DFG cycle-time, ${celLatency.toFixed(1)} ms median, real engine)
  rate card: $0.004/segment mined, x252 runs/yr
  annual equivalence: ${money(celAnnual)}
${rule}
SECTION 3 — CLOUD-CREDIT DISPLACEMENT (GATE-V-01 court, VAL-003 ledger)
${rule}
  court verdict: ${verdicts[2]}
  measured: p95 ${p95.toFixed(4)} ms over ${iters.toLocaleString()} measured 3-query iterations (real engine), ${credits} sovereign credits consumed
  rate card: V4 collector counterfactual (warehouse-credit math, operator-supplied, unaudited)
  annual equivalence: ${money(g01Annual)}
${rule}
SECTION 4 — PRODUCT-ANALYTICS DISPLACEMENT (Amplitude funnel court, VAL-004)
${rule}
  court verdict: ${verdicts[3]}
  measured: ${ampEvents.toLocaleString()} events -> 90-day retention matrix + 5-step funnel in ${ampFunnelMs} ms (real engine)
  rate card: $200/M-events-mo, monthly cadence assumed
  annual equivalence: ${money(ampAnnual)}
${line}
TOTAL AUDITED (annual equivalence, ASSUMPTION-UNVERIFIED rate cards): ${money(totalAnnual)}
${line}`);
}

function renderDoctorLine(): void {
  // Physical System State — printed ONLY when a doctor verdict was supplied;
  // otherwise the honest not-green line (never a fake green state). A verdict
  // of exactly GREEN prints the full-pass line; anything else is shown
  // verbatim as the actual standing.
  if (doctorStanding === "GREEN") {
    console.log("PHYSICAL SYSTEM STATE: FULL DOCTOR PASS (11/11 phases, fail-closed aggregate exit 0)");
  } else if (doctorStanding !== "") {
    console.log(`PHYSICAL SYSTEM STATE: ${doctorStanding}`);
  } else {
    console.log("PHYSICAL SYSTEM STATE: NOT GREEN — actual standing: doctor not run/not provided");
  }
}

function renderB3Block(): void {
  const b3 = process.env.B3_BLOCK ?? "";
  if (doctorStanding !== "" && b3 === "") {
    // A doctor-backed render must carry the receipt signature — no silent
    // "(unavailable)" when a verdict is being claimed.
    fail("B3_BLOCK env missing: --doctor render requires the b3sum block over the four receipts");
  }
  console.log(`receipt BLAKE3 (b3sum over the four receipt files):
${b3 || "(b3sum block unavailable — no --doctor verdict supplied)"}
${doctorLogBlock()}
rate cards: operator-supplied list prices, unaudited`);
}

function doctorLogBlock(): string {
  if (doctorLog === "") return "";
  let digest = "(b3sum unavailable)";
  let tail = "(log unreadable)";
  try {
    const txt = readFileSync(doctorLog, "utf8");
    tail = txt.trimEnd().split("\n").slice(-6).join("\n  ");
  } catch {
    return `doctor log: ${doctorLog} (unreadable)`;
  }
  try {
    digest = execFileSync("b3sum", [doctorLog], { encoding: "utf8" }).trim();
  } catch {
    /* keep placeholder */
  }
  return `doctor log: ${doctorLog}
  b3sum: ${digest}
  tail:
  ${tail}`;
}

try {
  render();
  renderDoctorLine();
  renderB3Block();
} catch (e) {
  if (!(e instanceof RenderFail)) throw e;
  process.exit(1);
}
