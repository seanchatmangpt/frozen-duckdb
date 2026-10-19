// T4-replica workload for the Class-B cgroup court:
// 1M-frame ingest + rolling-window anomaly detection under a 128MB container kernel limit.
// Uses the DuckDB C API (stable across versions); DuckDB capped at 64MB via memory_limit
// so the container kernel limit (128MB) is the binding leg.
#include <cstdio>
#include <cstdlib>
#include "duckdb.h"

int main() {
    duckdb_database db;
    duckdb_connection con;
    if (duckdb_open_ext(":memory:", &db, nullptr, nullptr) == DuckDBError) { fprintf(stderr, "OPEN FAIL\n"); return 2; }
    duckdb_connect(db, &con);

    auto q = [&](const char* sql) -> duckdb_result {
        duckdb_result res;
        duckdb_state st = duckdb_query(con, sql, &res);
        if (st == DuckDBError) {
            fprintf(stderr, "SQL FAIL: %s :: %s\n", sql, duckdb_result_error(&res));
            exit(2);
        }
        return res;
    };

    q("SET threads=1; SET memory_limit='64MB';");

    // 1M-frame ingest (id, sensor, value), same shape as T4
    q("CREATE TABLE frames AS "
      "SELECT i AS id, (i % 64) AS sensor, "
      "       50.0 + 10*sin(i/1000.0) + ((i*7919) % 97) / 97.0 AS value "
      "FROM range(1000000) t(i);");

    // Rolling anomaly detection: z-score over 50k-frame rolling window per sensor
    q("CREATE TABLE anomalies AS "
      "SELECT id, value, "
      "       (value - avg(value) OVER w) / "
      "       NULLIF(stddev_samp(value) OVER w, 0) AS z "
      "FROM frames "
      "WINDOW w AS (PARTITION BY sensor ROWS BETWEEN 50000 PRECEDING AND CURRENT ROW);");

    duckdb_result res = q("SELECT count(*) FROM anomalies WHERE abs(z) > 2.0;");
    printf("ANOMALIES_Z_GT_2=%lld\n", (long long)duckdb_value_int64(&res, 0, 0));
    printf("VERDICT=RSS_HELD\n");
    duckdb_destroy_result(&res);
    duckdb_disconnect(&con);
    db ? duckdb_close(&db) : (void)0;
    return 0;
}
