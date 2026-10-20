# ADV12 receipt: extension watcher (KD-02 closure watcher)

Date: 2026-10-20T00:35Z. Lane ADV12. Subject: /Users/sac/frozen-duckdb
(master, HEAD adbd260).

## Artifact

`scripts/extension_watch.sh` — one-shot, cron-able probe of the DuckDB
extension CDN at OUR dev pin `b6013f7aa2` (osx_arm64), for the KD-02
closure set `flock` + `tpch`:

- `http://extensions.duckdb.org/b6013f7aa2/osx_arm64/flock.duckdb_extension.gz`
- `http://extensions.duckdb.org/b6013f7aa2/osx_arm64/tpch.duckdb_extension.gz`

HTTP 200 on HEAD -> prints `AVAILABLE` + the exact un-ignore sed line and
`cargo test -p frozen-duckdb --test flock_tests`. Any other code ->
typed `STILL_ABSENT` line, exit 1. No daemon, no state, one invocation =
one probe.

## Real run (2026-10-20T00:35:04Z)

```
[2026-10-20T00:35:04Z] flock http://extensions.duckdb.org/b6013f7aa2/osx_arm64/flock.duckdb_extension.gz -> HTTP 404 still absent
[2026-10-20T00:35:05Z] tpch http://extensions.duckdb.org/b6013f7aa2/osx_arm64/tpch.duckdb_extension.gz -> HTTP 404 still absent
STILL_ABSENT: no dev-pin build on the extension CDN for: flock tpch (pin b6013f7aa2, osx_arm64)
EXIT=1
```

Standing: STILL_ABSENT (exit 1) — the honest outcome; matches the
`flock_tests.rs` `#[ignore]` evidence verbatim. Nothing faked, no
AVAILABLE claim.

## Wiring

`KNOWN_DIVERGENCES.md` KD-02 close-condition cell now names the watcher:
`bash scripts/extension_watch.sh` (exit 0+AVAILABLE = close now, exit 1 =
still absent).

## Upstream v2.0.0 final question

Per task scope: not probed — direct CDN probes at our pin only. The v2.0.0
final tag would change the pin itself; re-run the watcher with the new pin
when that lands (the `PIN`/`BASE` variables are the only edit).

## Falsifier

When upstream publishes a dev-pin flock build, the watcher must flip to
AVAILABLE; a watcher that reports STILL_ABSENT against a curl-verified 200
is FAIL.
