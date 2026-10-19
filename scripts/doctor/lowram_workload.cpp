// T4-replica workload for the Class-B cgroup court:
// 1M-frame ingest + rolling-window anomaly detection under a 128MB container kernel limit.
// DuckDB itself is capped at 64MB via memory_limit so the kernel limit is the binding leg.
#include <cstdio>
#include <cstdlib>
#include "duckdb.hpp"

using namespace duckdb;

int main() {
    try {
        DuckDB db(":memory:");
        Connection con(db);
        con.Query("SET threads=1");
        con.Query("SET memory_limit='64MB'");

        // 1M-frame workload: same shape as T4 (id, sensor, value frame stream)
        auto r = con.Query(
            "CREATE TABLE frames AS "
            "SELECT i AS id, (i % 64) AS sensor, "
            "       50.0 + 10*sin(i/1000.0) + ((i*7919) % 97) / 97.0 AS value "
            "FROM range(1000000) t(i);");
        if (r->HasError()) { fprintf(stderr, "INGEST FAIL: %s\n", r->GetError().c_str()); return 2; }

        // Rolling anomaly detection: z-score over a 50k-frame rolling window
        r = con.Query(
            "CREATE TABLE anomalies AS "
            "SELECT id, value, "
            "       (value - avg(value) OVER w) / "
            "       NULLIF(stddev_samp(value) OVER w, 0) AS z "
            "FROM frames "
            "WINDOW w AS (PARTITION BY sensor ORDER BY id ROWS BETWEEN 50000 PRECEDING AND CURRENT ROW);");
        if (r->HasError()) { fprintf(stderr, "ROLLING FAIL: %s\n", r->GetError().c_str()); return 2; }

        r = con.Query("SELECT count(*) FROM anomalies WHERE abs(z) > 2.0;");
        if (r->HasError()) { fprintf(stderr, "CHECK FAIL: %s\n", r->GetError().c_str()); return 2; }
        printf("ANOMALIES_Z_GT_2=%s\n", r->GetValue(0, 0).ToString().c_str());
        printf("VERDICT=RSS_HELD\n");
        return 0;
    } catch (const std::exception& e) {
        fprintf(stderr, "EXCEPTION: %s\n", e.what());
        return 2;
    }
}
