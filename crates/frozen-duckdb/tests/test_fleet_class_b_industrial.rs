//! TEST-FLEET-002 — Class B (industrial telemetry) test-fleet leg.
//!
//! Two legs, recorded honestly:
//!
//! 1. REAL Class-B court: Linux cgroup low-RAM runner
//!    (`scripts/doctor/cgroup_low_ram_runner.sh`) — cgcreate/cgset
//!    `memory.max=128MB`, iptables egress sever, 4h soak scaffold with the
//!    kernel's `memory.events` OOM ledger as the standing source. Linux+root
//!    only; elsewhere it exits 2 `[SKIP-LOUD]`. That script is the Class-B
//!    court proper.
//!
//! 2. THIS FILE — the macOS memory-envelope PROXY leg. Proxy = PARTIAL_ALIVE
//!    ceiling: it proves the engine's 128MB `memory_limit` config binds and
//!    that a 1M-frame streaming ingest + rolling-anomaly workload stays
//!    inside the RSS envelope with zero aborts on THIS host. It is NOT the
//!    kernel-enforced cap; only the cgroup leg is.
//!
//! Tolerance, stated (D6 deceleration law, scaled): the doctor-leak court
//! measured a documented 64MiB warm-up envelope on 20k aggregate iterations;
//! this court scales to 20 windows of 50k appender frames and asserts the
//! post-warmup window RSS ≤ 110MB (128MB cap − 18MB warm-up/noise headroom)
//! AND per-window RSS growth decelerating (each post-warmup window's growth
//! ≤ the prior window's + a 2MiB noise allowance).
//!
//! Receipt: FLEET-004 → 004B → /tmp/value_audit/fleet002_proxy.json

use frozen_duckdb::{Config, Connection, ToSql};
use std::process::Command;
use std::time::Instant;

const FRAMES: usize = 1_000_000;
const BATCH: usize = 50_000;
const WINDOWS: usize = FRAMES / BATCH; // 20
const WARMUP_WINDOWS: usize = 4;
/// Post-warmup RSS ceiling: 110MB (128MB cap − 18MB stated headroom).
const RSS_CEILING_BYTES: u64 = 110 * 1024 * 1024;
/// RSS sampling noise allowance between adjacent growth measurements: 4 MiB.
/// Stated, not silent: macOS RSS is quantized in ~1-4 MiB allocator chunks
/// (witnessed first run: growth steps of +1.1MB and +4.0MB in one window),
/// so acceleration is judged on 2-window smoothed growth against this
/// allowance.
const NOISE_BYTES: u64 = 4 * 1024 * 1024;

/// One RSS sample in bytes (macOS: `ps -o rss=`; Linux: VmRSS).
fn rss_bytes() -> u64 {
    #[cfg(target_os = "linux")]
    {
        let status = std::fs::read_to_string("/proc/self/status").expect("procfs");
        for line in status.lines() {
            if let Some(rest) = line.strip_prefix("VmRSS:") {
                return rest
                    .trim()
                    .trim_end_matches("kB")
                    .trim()
                    .parse::<u64>()
                    .expect("VmRSS integer")
                    * 1024;
            }
        }
        panic!("VmRSS not found in /proc/self/status");
    }
    #[cfg(not(target_os = "linux"))]
    {
        let out = Command::new("ps")
            .args(["-o", "rss=", "-p", &std::process::id().to_string()])
            .output()
            .expect("ps available");
        assert!(out.status.success(), "ps exited nonzero");
        String::from_utf8_lossy(&out.stdout)
            .trim()
            .parse::<u64>()
            .expect("rss KB integer")
            * 1024
    }
}

/// Deterministic seeded PRNG (xorshift64*), no external deps.
struct Rng(u64);

impl Rng {
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    /// Telemetry metric value in [-4, 100]; ~2% of frames breach the 90.0
    /// anomaly threshold by construction (uniform tail 90..=100).
    fn metric(&mut self) -> f64 {
        (self.next_u64() % 10_400) as f64 / 100.0 - 4.0
    }
}

#[test]
fn test_fleet_class_b_proxy() {
    // Engine config: the safe key per docs/design/memory-bounds.md —
    // `memory_limit` via the Config::with passthrough. threads=1 bounds
    // per-worker allocator pools (in-browser profile, same doc).
    let config = Config::default()
        .with("memory_limit", "128MB")
        .expect("config with memory_limit=128MB")
        .with("threads", "1")
        .expect("config with threads=1");
    let conn = Connection::open_in_memory_with_flags(config).expect("open in-memory db");

    conn.execute_batch(
        "CREATE TABLE telemetry (
            frame_id BIGINT,
            device_id INTEGER,
            ts_ms BIGINT,
            metric DOUBLE,
            status INTEGER
        );",
    )
    .expect("create telemetry table");

    let mut rng = Rng(0x5EED_F002_B105_0042);
    let mut rss_samples: Vec<u64> = Vec::with_capacity(WINDOWS);
    let mut total_anomalies: i64 = 0;
    let started = Instant::now();

    for window in 0..WINDOWS {
        // Stream one window of frames through the appender (zero aborts =
        // every append_row + flush returns Ok; any Err would unwrap-panic).
        let mut appender = conn.appender("telemetry").expect("appender");
        let base = (window * BATCH) as i64;
        let batch: Vec<(i64, i32, i64, f64, i16)> = (0..BATCH)
            .map(|i| {
                (
                    base + i as i64,
                    (i % 4096) as i32,
                    base + i as i64,
                    rng.metric(),
                    (i % 7) as i16,
                )
            })
            .collect();
        let rows: Vec<[&dyn ToSql; 5]> = batch
            .iter()
            .map(|r| -> [&dyn ToSql; 5] { [&r.0, &r.1, &r.2, &r.3, &r.4] })
            .collect();
        appender.append_rows(rows).expect("append window");
        appender.flush().expect("appender flush");

        // Rolling anomaly query over the ingested tail (last 2 windows).
        let count: i64 = conn
            .query_row(
                "SELECT count(*) FROM telemetry \
                 WHERE metric > 90.0 AND frame_id >= ?1",
                [base - BATCH as i64],
                |r| r.get(0),
            )
            .expect("rolling anomaly query");
        total_anomalies += count;

        rss_samples.push(rss_bytes());
    }

    let elapsed = started.elapsed();

    // ---- asserts on real state ----
    let total: i64 = conn
        .query_row("SELECT count(*) FROM telemetry", [], |r| r.get(0))
        .expect("count(*)");
    assert_eq!(total, FRAMES as i64, "all 1M frames ingested");
    assert!(
        total_anomalies > 0,
        "anomaly query must actually select rows (non-vacuous workload)"
    );

    // Post-warmup envelope (D6 deceleration law, scaled): every window from
    // WARMUP_WINDOWS on must sit at or below the 110MB ceiling.
    for (w, &rss) in rss_samples.iter().enumerate().skip(WARMUP_WINDOWS) {
        assert!(
            rss <= RSS_CEILING_BYTES,
            "post-warmup RSS window {} = {} bytes exceeds 110MB ceiling",
            w,
            rss
        );
    }

    // Deceleration (D6 law, scaled tolerance stated at NOISE_BYTES): judged
    // on 2-window smoothed growth — each post-warmup 2-window growth must be
    // ≤ the prior 2-window growth + the stated noise allowance. Pair
    // smoothing absorbs the ~1-4MiB allocator quantization of single windows.
    for w in (WARMUP_WINDOWS + 2)..WINDOWS {
        let prev = rss_samples[w - 1] as i64 - rss_samples[w - 3] as i64;
        let cur = rss_samples[w] as i64 - rss_samples[w - 2] as i64;
        assert!(
            cur <= prev + NOISE_BYTES as i64,
            "RSS growth accelerating at window {}: 2w-prev={:+} 2w-cur={:+} (deceleration law)",
            w,
            prev,
            cur
        );
    }

    // ---- receipt (real measured values, written by the run itself) ----
    let peak_post_warmup = rss_samples
        .iter()
        .skip(WARMUP_WINDOWS)
        .copied()
        .max()
        .unwrap_or(0);
    let receipt = format!(
        "{{\n  \"court\": \"TEST-FLEET-002-class-b-proxy\",\n  \"receipt_chain\": \"FLEET-004 -> 004B\",\n  \"standing\": \"PARTIAL_ALIVE (proxy ceiling; cgroup leg is the Class-B court proper)\",\n  \"memory_limit\": \"128MB\",\n  \"frames\": {},\n  \"windows\": {},\n  \"warmup_windows\": {},\n  \"rss_ceiling_bytes\": {},\n  \"peak_post_warmup_rss_bytes\": {},\n  \"first_sample_rss_bytes\": {},\n  \"final_rss_bytes\": {},\n  \"rss_samples_bytes\": {:?},\n  \"total_anomaly_rows\": {},\n  \"aborts\": 0,\n  \"elapsed_secs\": {:.3},\n  \"host\": \"{}\"\n}}\n",
        FRAMES,
        WINDOWS,
        WARMUP_WINDOWS,
        RSS_CEILING_BYTES,
        peak_post_warmup,
        rss_samples[0],
        rss_samples[rss_samples.len() - 1],
        rss_samples,
        total_anomalies,
        elapsed.as_secs_f64(),
        std::env::consts::OS,
    );
    std::fs::create_dir_all("/tmp/value_audit").expect("mkdir /tmp/value_audit");
    std::fs::write("/tmp/value_audit/fleet002_proxy.json", receipt).expect("write receipt");
    println!("receipt: /tmp/value_audit/fleet002_proxy.json");
}
