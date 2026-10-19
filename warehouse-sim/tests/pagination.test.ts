// Pagination non-vacuity test (gap S3/J3) — lane GAP2.
// Chicago discipline: real HTTP against a real `next dev` server. Pages a
// large SUCCEEDED result to exhaustion over the wire and asserts token
// termination, slice integrity (no duplicate / no missing rows), and
// deterministic row content (same page re-fetched is byte-identical).

import assert from "node:assert/strict";
import { after, before, describe, it } from "node:test";
import { startServer, pollUntil, type Server } from "./helpers/harness.ts";

const SNOWFLAKE_SUBMIT = "/api/v2/statements";
const DATABRICKS_SUBMIT = "/api/2.0/sql/statements";
const POLL_BUDGET_MS = 15_000; // covers cold start (4500/3800ms) + jitter
const EXPECTED_PAGE_SIZE = 10000;
const EXPECTED_ROW_COUNT = 25000;

interface ResultSet {
  rowCount: number;
  pageSize: number;
  page: number;
  data: string[][];
  nextPageToken?: string;
}

interface SnowflakeSucceeded {
  handle: string;
  state: string;
  resultSet?: ResultSet;
}

interface DatabricksSucceeded {
  statement_id: string;
  status: { state: string; response_code?: number };
  result?: ResultSet;
}

let server: Server;

before(async () => {
  server = await startServer();
  await server.ready();
});

after(async () => {
  if (server) await server.close();
});

async function submitSnowflake(): Promise<string> {
  const res = await fetch(`${server.baseUrl}${SNOWFLAKE_SUBMIT}`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ sqlText: "SELECT 1", warehouse: "WH", database: "DB", schema: "S", asyncMode: true }),
  });
  assert.equal(res.status, 202, `submit expected 202, got ${res.status}`);
  const { statementHandle } = (await res.json()) as { statementHandle: string };
  return statementHandle;
}

async function submitDatabricks(): Promise<string> {
  const res = await fetch(`${server.baseUrl}${DATABRICKS_SUBMIT}`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ statement: "SELECT 1", warehouseId: "abc123", catalog: "main", schema: "default", waitTimeout: "0s" }),
  });
  assert.equal(res.status, 200, `submit expected 200, got ${res.status}`);
  const { statement_id } = (await res.json()) as { statement_id: string };
  return statement_id;
}

/** Page a paginated result to exhaustion via the opaque nextPageToken chain.
 * Returns every row in page order plus the per-page slice metadata. */
async function pageToExhaustion(
  pollUrl: string,
  firstPage: ResultSet,
): Promise<{ pages: ResultSet[]; rows: string[][] }> {
  const pages: ResultSet[] = [firstPage];
  const rows: string[][] = [...firstPage.data];
  let guard = 0;
  while (pages[pages.length - 1].nextPageToken !== undefined) {
    if (++guard > 50) throw new Error("nextPageToken chain did not terminate within 50 pages");
    const token = encodeURIComponent(pages[pages.length - 1].nextPageToken!);
    const res = await fetch(`${pollUrl}?pageToken=${token}`, { signal: AbortSignal.timeout(10_000) });
    assert.equal(res.status, 200, `page fetch expected 200, got ${res.status}`);
    const body = (await res.json()) as { resultSet?: ResultSet; result?: ResultSet };
    const rs = body.resultSet ?? body.result;
    assert.ok(rs, `page fetch returned no result envelope: ${JSON.stringify(body).slice(0, 120)}`);
    pages.push(rs!);
    rows.push(...rs!.data);
  }
  return { pages, rows };
}

function assertSlices(pages: ResultSet[], rows: string[][]): void {
  // Token termination: exactly one tokenless page, and it is last.
  const tokenless = pages.filter((p) => p.nextPageToken === undefined);
  assert.equal(tokenless.length, 1, `expected exactly one tokenless (final) page, got ${tokenless.length}`);
  assert.equal(pages.indexOf(tokenless[0]), pages.length - 1, "tokenless page must be the last page");

  // Slice integrity: page size honored, full coverage, no duplicates.
  assert.equal(rows.length, EXPECTED_ROW_COUNT, `expected ${EXPECTED_ROW_COUNT} rows total, got ${rows.length}`);
  for (const p of pages.slice(0, -1)) {
    assert.equal(p.data.length, EXPECTED_PAGE_SIZE, `non-final page must be full (${EXPECTED_PAGE_SIZE} rows)`);
  }
  assert.equal(pages[pages.length - 1].data.length, EXPECTED_ROW_COUNT % EXPECTED_PAGE_SIZE, "final page is the remainder slice");
  const seen = new Set(rows.map((r) => JSON.stringify(r)));
  assert.equal(seen.size, rows.length, "duplicate rows across pages");

  // Envelope sanity: page indices monotone from 1; rowCount/pageSize echo.
  pages.forEach((p, i) => assert.equal(p.page, i + 1, `page index out of order at ${i}`));
  for (const p of pages) {
    assert.equal(p.rowCount, EXPECTED_ROW_COUNT);
    assert.equal(p.pageSize, EXPECTED_PAGE_SIZE);
  }
}

async function assertDeterministic(pollUrl: string, page2: ResultSet): Promise<void> {
  // Determinism = identical envelope bytes on repeat reads of the same page.
  const a = await fetch(`${pollUrl}?page=2`, { signal: AbortSignal.timeout(10_000) });
  const b = await fetch(`${pollUrl}?page=2`, { signal: AbortSignal.timeout(10_000) });
  const ra = (await a.json()) as { resultSet?: ResultSet; result?: ResultSet };
  const rb = (await b.json()) as { resultSet?: ResultSet; result?: ResultSet };
  const sa = ra.resultSet ?? ra.result;
  const sb = rb.resultSet ?? rb.result;
  assert.deepEqual(sa, sb, "repeated fetch of the same page must be byte-identical");
  assert.deepEqual(sa, page2, "page 2 re-read must match the exhaustion-loop page");
}

describe("Snowflake result pagination (S3/J3)", () => {
  it("inlines the first page on the terminal poll and pages to exhaustion", async () => {
    const handle = await submitSnowflake();
    const pollUrl = `${server.baseUrl}${SNOWFLAKE_SUBMIT}/${handle}`;
    const trace = await pollUntil<SnowflakeSucceeded>(pollUrl, (b) => b.state === "SUCCEEDED" || b.state === "FAILED", {
      budgetMs: POLL_BUDGET_MS,
      intervalMs: 500,
    });
    assert.equal(trace.settled, true, "poll did not reach SUCCEEDED within budget");
    const firstPage = trace.final.body.resultSet;
    assert.ok(firstPage, "terminal SUCCEEDED poll must carry the inline first page");
    assert.equal(firstPage!.page, 1, "inline page must be page 1");
    assert.equal(firstPage!.data.length, EXPECTED_PAGE_SIZE, "inline first page must be full");

    const { pages, rows } = await pageToExhaustion(pollUrl, firstPage!);
    assertSlices(pages, rows);
    await assertDeterministic(pollUrl, pages[1]);
  });
});

describe("Databricks result pagination (S3/J3)", () => {
  it("inlines the first page on the terminal poll and pages to exhaustion", async () => {
    const statementId = await submitDatabricks();
    const pollUrl = `${server.baseUrl}${DATABRICKS_SUBMIT}/${statementId}`;
    const trace = await pollUntil<DatabricksSucceeded>(pollUrl, (b) => b.status.state === "SUCCEEDED" || b.status.state === "FAILED", {
      budgetMs: POLL_BUDGET_MS,
      intervalMs: 500,
    });
    assert.equal(trace.settled, true, "poll did not reach SUCCEEDED within budget");
    const firstPage = trace.final.body.result;
    assert.ok(firstPage, "terminal SUCCEEDED poll must carry the inline first page");
    assert.equal(firstPage!.page, 1, "inline page must be page 1");
    assert.equal(firstPage!.data.length, EXPECTED_PAGE_SIZE, "inline first page must be full");

    const { pages, rows } = await pageToExhaustion(pollUrl, firstPage!);
    assertSlices(pages, rows);
    await assertDeterministic(pollUrl, pages[1]);
  });
});
