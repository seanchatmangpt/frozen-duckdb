#!/usr/bin/env bash
# Class-B cgroup leg (Docker): run the T4-replica 1M-frame + rolling-anomaly
# workload under a HARD KERNEL MEMORY LIMIT (128MB, swap disabled).
# Exit 137 (SIGKILL) => OOM-kill => ceiling breach, reported honestly.
# Exit 0 => RSS held under the kernel-enforced cgroup ceiling.
set -u
IMAGE="kcura-lowram:latest"
MEM_MB="${MEM_MB:-128}"
OUT="${1:-/tmp/value_audit/fleet002_cgroup_docker.json}"

mkdir -p "$(dirname "$OUT")"

ENGINE=$(docker version --format '{{.Server.Os}}/{{.Server.Arch}} engine {{.Server.Version}}')
echo "[lowram] engine: $ENGINE"
echo "[lowram] running ${IMAGE} with --memory ${MEM_MB}m --memory-swap ${MEM_MB}m --cpus 2"

START=$(date +%s)
docker run --rm --memory "${MEM_MB}m" --memory-swap "${MEM_MB}m" \
  --cpus 2 --name kcura-lowram-court "$IMAGE" > /tmp/lowram_run_stdout.txt 2> /tmp/lowram_run_stderr.txt
EXIT=$?
END=$(date +%s)
DUR=$((END - START))

echo "[lowram] container exit code: $EXIT (${DUR}s)"
echo "--- stdout ---"; cat /tmp/lowram_run_stdout.txt
echo "--- stderr ---"; cat /tmp/lowram_run_stderr.txt

if [ "$EXIT" -eq 0 ]; then VERDICT="RSS_HELD (kernel-enforced ${MEM_MB}MB cgroup limit held through 1M-frame + rolling-anomaly workload)"
elif [ "$EXIT" -eq 137 ]; then VERDICT="OOM_KILLED (ceiling breach at kernel-enforced ${MEM_MB}MB; exit 137 SIGKILL)"
else VERDICT="RUN_FAILED (exit ${EXIT}, not an OOM verdict)"; fi

cat > "$OUT" <<EOF
{
  "court": "fleet002_class_b_cgroup_docker",
  "engine": "${ENGINE}",
  "platform": "linux/arm64 container (colima virtualized ARM on darwin/arm64 host)",
  "caveat": "kernel-enforced cgroup limit is real, but the kernel itself is virtualized (colima VM), not bare metal",
  "pinned_duckdb_sha": "b6013f7aa2f36caabb9466ce1791667ea8a280bb",
  "mem_limit_mb": ${MEM_MB},
  "mem_swap_mb": ${MEM_MB},
  "container_exit": ${EXIT},
  "duration_s": ${DUR},
  "stdout": $(python3 -c 'import json,sys; print(json.dumps(open(sys.argv[1]).read()))' /tmp/lowram_run_stdout.txt),
  "stderr": $(python3 -c 'import json,sys; print(json.dumps(open(sys.argv[1]).read()))' /tmp/lowram_run_stderr.txt),
  "verdict": "${VERDICT}"
}
EOF
echo "[lowram] receipt written: $OUT"
exit $EXIT
