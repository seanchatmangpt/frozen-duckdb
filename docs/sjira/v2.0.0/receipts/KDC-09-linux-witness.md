# KDC-09 — KD-09 Linux-kernel witness receipt

Date: 2026-10-19. Lane KDC-09. Repo: /Users/sac/frozen-duckdb (master,
032f94a at lane start; no git actions taken). Surface: Class B hardware
court, cgroup leg — `scripts/doctor/docker_low_ram_runner.sh`.

## What ran

The `kcura-lowram` image had been pruned from the colima VM (previous
DOCKER1 receipts were on engine 29.5.2; at lane start the server reported
28.0.4/29.5.2 discrepancy — the image was absent, `docker images` had no
kcura-lowram). Rebuilt from source, fresh:

- `docker build -f scripts/doctor/Dockerfile.lowram -t kcura-lowram:latest scripts/doctor/`
  — exit 0. Clones duckdb at pinned SHA `d54cbbe3df17e8e5ce68312f483b4155836112aa`,
  Release build (no unittests/shell/ICU), `-j2`, then compiles
  `scripts/doctor/lowram_workload.cpp` against it. One earlier attempt was
  canceled at my own 600 s background cap, restarted at 7200 s — no
  Dockerfile/infra edits, `scripts/doctor/` untouched.
- Court, 128 MB leg: `scripts/doctor/docker_low_ram_runner.sh
  /tmp/value_audit/fleet002_cgroup_docker.json` — exit 0.
- Court, 32 MB negative control: `MEM_MB=32 scripts/doctor/docker_low_ram_runner.sh
  /tmp/value_audit/fleet002_cgroup_docker_negctl.json` — exit 137.
- Evidence run (same image, same limits, 32 MB, wrapper entrypoint —
  not part of the court script): reads kernel-side cgroup state from
  inside the container before/after the workload.

## Court output (verbatim)

128 MB leg (exit 0):

```
[lowram] engine: linux/arm64 engine 29.5.2
[lowram] running kcura-lowram:latest with --memory 128m --memory-swap 128m --cpus 2
[lowram] container exit code: 0 (7s)
--- stdout ---
ANOMALIES_Z_GT_2=261
VERDICT=RSS_HELD
```

32 MB negative control (exit 137):

```
[lowram] engine: linux/arm64 engine 29.5.2
[lowram] running kcura-lowram:latest with --memory 32m --memory-swap 32m --cpus 2
[lowram] container exit code: 137 (0s)
```

## Kernel-side evidence (from INSIDE the container)

32 MB evidence run, verbatim stdout:

```
=== kernel identity ===
Linux 1667ecbe571b 6.8.0-117-generic #117-Ubuntu SMP PREEMPT_DYNAMIC Thu May  7 17:26:37 UTC 2026 aarch64 GNU/Linux
PRETTY_NAME="Debian GNU/Linux 12 (bookworm)"
=== cgroup ===
cgroup2fs
33554432
=== memory.events BEFORE ===
low 0
high 0
max 0
oom 0
oom_kill 0
oom_group_kill 0
Killed
=== workload exit: 137 ===
=== memory.events AFTER ===
low 0
high 0
max 24
oom 1
oom_kill 1
oom_group_kill 0
=== memory.peak ===
33554432
=== vmstat oom fields ===
oom_kill 1
```

(The live copy to the virtiofs bind mount `/tmp/oom_evidence/` did not
persist files after container teardown; the transcript above is the
retained evidence.)

128 MB held run, kernel-side (same image, same limits, exit 0):

```
ANOMALIES_Z_GT_2=261
VERDICT=RSS_HELD
workload exit: 0
=== memory.events (128MB run) ===
low 0
high 0
max 0
oom 0
oom_kill 0
oom_group_kill 0
oom_kill 2
```

(The final `oom_kill 2` line is /proc/vmstat — the VM-global counter,
which counts the two earlier 32 MB kills; the container's own
memory.events for the held run shows oom_kill 0. Coherent.)

## Interpretation

- Real Linux kernel 6.8.0-117-generic (Ubuntu, aarch64), cgroup v2.
- The kernel set memory.max = 33554432 exactly as requested and
  memory.peak reached exactly 33554432 — the ceiling was reached, not
  approximated.
- memory.events oom_kill went 0 → 1 and workload exit was 137 (SIGKILL
  from the kernel's OOM killer, not a userspace abort).
- The 128 MB run completed the 1M-frame + rolling-z-score workload with
  container oom_kill 0.

The kernel limit is genuinely kernel-enforced. The caveat stands and is
recorded: the kernel runs in a colima VM on macOS Virtualization.Framework
(aarch64), so the hardware is virtualized. What this witnesses is the
cgroup enforcement semantics of a real Linux kernel, not bare-metal
memory bandwidth/latency.

## Verdict: CLOSED

KD-09's registered close condition reads: "Linux+root host runs the real
cgroup court; or DOCKER1's container leg witnesses the kernel limit."
The second disjunct is now witnessed — not merely by an exit code, but
by kernel-side counters read from inside the container's own cgroup
(memory.events oom_kill 0→1, memory.peak == memory.max) plus /proc/vmstat.
The virtualization caveat is noted; it changes the hardware class of the
witness, not the fact of kernel enforcement, which is what KD-09's
divergence ("cgroup leg requires Linux + root; darwin proxy is PARTIAL
ceiling") was about. Prior DOCKER1 receipts (/tmp/value_audit/
fleet002_cgroup_docker{,_negctl}.json from engine 29.5.2, same verdicts)
are corroborated by a fully fresh rebuild-and-run.

## Artifacts

- /tmp/value_audit/fleet002_cgroup_docker.json (fresh, exit 0, RSS_HELD,
  pinned sha d54cbbe3df17e8e5ce68312f483b4155836112aa, engine 29.5.2)
- /tmp/value_audit/fleet002_cgroup_docker_negctl.json (fresh, exit 137,
  OOM_KILLED, 32 MB)
- kcura-lowram:latest image rebuilt this session (Dockerfile.lowram,
  exit 0)
