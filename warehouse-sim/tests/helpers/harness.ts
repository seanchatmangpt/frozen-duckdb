// Real-server harness for warehouse-sim contract tests (Chicago: real HTTP).
// Boots `next dev` in a child process, deadline-polls for readiness, and kills
// the whole process group on teardown. No fixed sleeps: readiness and all
// waits are bounded polls on real observables.

import { spawn, type ChildProcess } from "node:child_process";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

export const WAREHOUSE_SIM_ROOT = join(import.meta.dirname, "..", "..");

export interface Server {
  baseUrl: string;
  ready(): Promise<void>;
  close(): Promise<void>;
}

export function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

const READINESS_DEADLINE_MS = 90_000;

export async function startServer(opts?: { port?: number }): Promise<Server> {
  const port = opts?.port ?? 3100 + Math.floor(Math.random() * 2000);
  const tmp = mkdtempSync(join(tmpdir(), "whsim-harness-"));
  const child: ChildProcess = spawn("npm", ["run", "dev", "--", "--port", String(port), "--hostname", "127.0.0.1"], {
    cwd: WAREHOUSE_SIM_ROOT,
    stdio: ["ignore", "pipe", "pipe"],
    detached: true, // own process group so teardown kills next-server children too
    env: { ...process.env, NEXT_TELEMETRY_DISABLED: "1" },
  });
  let log = "";
  // stdio is ["ignore","pipe","pipe"], so stdout/stderr are non-null here;
  // the non-null assertion encodes that spawn-contract invariant.
  child.stdout!.on("data", (d: Buffer) => { log += d.toString(); });
  child.stderr!.on("data", (d: Buffer) => { log += d.toString(); });

  let closed = false;
  async function close(): Promise<void> {
    if (closed) return;
    closed = true;
    try { process.kill(-child.pid!, "SIGTERM"); } catch { /* already gone */ }
    const deadline = Date.now() + 5_000;
    while (Date.now() < deadline && (await isAlive(child.pid!))) await sleep(100);
    // Unconditional SIGKILL escalation on the whole process group — next-server
    // workers survive the npm SIGTERM when only the npm pid is checked.
    try { process.kill(-child.pid!, "SIGKILL"); } catch { /* gone */ }
    for (let i = 0; i < 10; i++) {
      if (!(await isAlive(child.pid!))) break;
      await sleep(100);
    }
    rmSync(tmp, { recursive: true, force: true });
  }

  async function ready(): Promise<void> {
    const deadline = Date.now() + READINESS_DEADLINE_MS;
    let lastErr = "";
    while (Date.now() < deadline) {
      if (child.exitCode !== null || child.signalCode !== null) {
        throw new Error(`server exited early (code=${child.exitCode} signal=${child.signalCode})\n--- log ---\n${log}`);
      }
      try {
        // Ready = the API route actually answers with API behavior (a 405
        // Method Not Allowed from the rendered route, JSON body), never a
        // 404 HTML page.
        const res = await fetch(`http://127.0.0.1:${port}/api/v2/statements`, { method: "GET", signal: AbortSignal.timeout(2_000) });
        const text = await res.text();
        if (res.status === 405 && !text.includes("<!DOCTYPE")) return;
        lastErr = `status=${res.status} body=${text.slice(0, 80)}`;
        await sleep(250);
      } catch (err) {
        lastErr = err instanceof Error ? err.message : String(err);
        await sleep(250);
      }
    }
    await close();
    throw new Error(`server not ready within ${READINESS_DEADLINE_MS}ms; last error: ${lastErr}\n--- log ---\n${log}`);
  }

  return { baseUrl: `http://127.0.0.1:${port}`, ready, close };
}

async function isAlive(pid: number): Promise<boolean> {
  try { process.kill(pid, 0); return true; } catch { return false; }
}

// Deadline-bounded poll of a GET endpoint until a predicate holds or the
// budget expires. Returns every observed payload + the final one.
export interface PollTrace<T> {
  observations: { atMs: number; status: number; body: T }[];
  final: { atMs: number; status: number; body: T };
  settled: boolean; // predicate met within budget
  elapsedMs: number;
}

export async function pollUntil<T>(
  url: string,
  predicate: (body: T) => boolean,
  opts: { budgetMs: number; intervalMs?: number },
): Promise<PollTrace<T>> {
  const started = Date.now();
  const observations: PollTrace<T>["observations"] = [];
  let last: PollTrace<T>["final"];
  while (Date.now() - started < opts.budgetMs) {
    const res = await fetch(url, { signal: AbortSignal.timeout(5_000) });
    const text = await res.text();
    let body: T;
    try {
      body = JSON.parse(text) as T;
    } catch {
      // REAL dev-mode behavior: during on-demand compilation Next can serve
      // its HTML error page from the dynamic route. Record it, retry.
      body = { _nonJson: text.slice(0, 120) } as unknown as T;
    }
    const atMs = Date.now() - started;
    const obs = { atMs, status: res.status, body };
    observations.push(obs);
    last = obs;
    if (!(body as any)?._nonJson && predicate(body)) {
      return { observations, final: obs, settled: true, elapsedMs: Date.now() - started };
    }
    await sleep(opts.intervalMs ?? 500);
  }
  return { observations, final: last!, settled: false, elapsedMs: Date.now() - started };
}

export function statesOf(trace: PollTrace<{ state?: string; status?: { state?: string } }>): string[] {
  return trace.observations.map((o) =>
    (o.body as any)?._nonJson ? "_nonJson" : o.body.state ?? o.body.status?.state ?? "?",
  );
}
