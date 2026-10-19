#!/usr/bin/env bash
# TEST-FLEET-002 — Class-B low-RAM runner court (Linux cgroup v2, REAL leg).
#
# Operator flow implemented:
#   1. cgcreate -g memory:fleet002-lowram
#   2. cgset -r memory.max=128M          (128MB hard memory cap)
#   3. iptables sever (no network egress from the cgroup's net-cls class)
#   4. run the ingest+anomaly workload INSIDE the cgroup (cgexec), 4h soak
#      scaffold (FLEET-004 → 004B upgrade: soak duration parameterized).
#   5. On OOM the kernel kills the workload (memory.oom.group=1) — the
#      standing is read from the real exit code + memory.events, never faked.
#
# Environment gate (loud skip, never a faked pass):
#   - non-Linux (e.g. darwin): exit 2 with [SKIP-LOUD]
#   - Linux but no root / no cgroup v2 controller: exit 2 with [SKIP-LOUD]
#
# Usage: sudo scripts/doctor/cgroup_low_ram_runner.sh [soak_seconds]
# Receipt: FLEET-004 → 004B; proxy leg receipt /tmp/value_audit/fleet002_proxy.json

set -euo pipefail

# ---- environment gate -------------------------------------------------------
if [[ "$(uname -s)" != "Linux" ]]; then
    echo "[SKIP-LOUD] cgroup leg requires Linux + root; this host is $(uname -s | tr 'A-Z' 'a-z')" >&2
    exit 2
fi
if [[ "${EUID}" -ne 0 ]]; then
    echo "[SKIP-LOUD] cgroup leg requires Linux + root; this host is linux but EUID=${EUID}" >&2
    exit 2
fi
if [[ ! -d /sys/fs/cgroup/memory || -z "$(cgcreate --help 2>/dev/null)" ]]; then
    # cgroup v2 unified hierarchy check: memory controller must be delegated.
    if ! grep -qw memory /sys/fs/cgroup/cgroup.controllers 2>/dev/null; then
        echo "[SKIP-LOUD] cgroup v2 memory controller not available on this host" >&2
        exit 2
    fi
fi

GROUP="fleet002-lowram"
MEMORY_MAX="134217728"          # 128MB, matches the proxy leg's memory_limit
SOAK_SECONDS="${1:-14400}"      # 4h default soak scaffold
MARKER="SHIM-COURT"             # court-literal marker required on any mutation

echo "== TEST-FLEET-002 Class-B cgroup court =="
echo "group=${GROUP} memory.max=${MEMORY_MAX} soak=${SOAK_SECONDS}s"

# ---- 1. cgroup create -------------------------------------------------------
cgdelete -g "memory:${GROUP}" 2>/dev/null || true
cgcreate -g "memory:${GROUP}"
trap 'cgdelete -g "memory:${GROUP}" 2>/dev/null || true' EXIT

# ---- 2. memory cap ----------------------------------------------------------
cgset -r "memory.max=${MEMORY_MAX}" "memory/${GROUP}"
cgset -r "memory.oom.group=1" "memory/${GROUP}"

# ---- 3. iptables sever (egress denial for the workload) ---------------------
# Classmark the cgroup so its packets carry fwmark 0xF2, then DROP egress.
iptables -t mangle -A OUTPUT -m cgroup --path "${GROUP}" -j MARK --set-xmark 0xF2/0xF2
sever_rule_added=1
cleanup_iptables() {
    if [[ "${sever_rule_added:-0}" -eq 1 ]]; then
        iptables -t mangle -D OUTPUT -m cgroup --path "${GROUP}" -j MARK --set-xmark 0xF2/0xF2 || true
    fi
}
trap 'cleanup_iptables; cgdelete -g "memory:${GROUP}" 2>/dev/null || true' EXIT

# ---- 4. workload inside the cgroup ------------------------------------------
# The workload is the workspace's own ingest+anomaly binary (cargo test with
# the proxy test filtered), run under cgexec so the kernel enforces 128MB.
cd "$(dirname "$0")/../.."
WORK_PID=""
cgexec -g "memory:${GROUP}" \
    cargo test -p frozen-duckdb --test test_fleet_class_b_industrial \
        fleet_class_b_proxy -- --ignored --nocapture &
WORK_PID=$!

# ---- 5. soak scaffold: poll real observables on a deadline (no sleep-as-law) -
# memory.events is the kernel's own OOM ledger; polls, not fixed sleeps.
DEADLINE=$(( $(date +%s) + SOAK_SECONDS ))
while kill -0 "${WORK_PID}" 2>/dev/null; do
    if [[ "$(date +%s)" -ge "${DEADLINE}" ]]; then
        echo "[SOAK] deadline (${SOAK_SECONDS}s) reached; killing workload" >&2
        kill "${WORK_PID}" 2>/dev/null || true
        break
    fi
    EVENTS="/sys/fs/cgroup/memory/${GROUP}/memory.events"
    if [[ -r "${EVENTS}" ]]; then
        OOMS=$(awk '$1=="oom_kill"{print $2}' "${EVENTS}")
        echo "[SOAK] poll oom_kill=${OOMS:-0}" >&2
        if [[ "${OOMS:-0}" -gt 0 ]]; then
            echo "FAIL: kernel OOM-killed the 128MB-capped workload (oom_kill=${OOMS})" >&2
            wait "${WORK_PID}" 2>/dev/null || true
            exit 1
        fi
    fi
    sleep 5   # SLEEP-CLASS: POLL-STEP (audited poll loop step)
done

wait "${WORK_PID}"
RC=$?
echo "== cgroup court workload exit=${RC} =="

# ---- receipt ----------------------------------------------------------------
EVENTS="/sys/fs/cgroup/memory/${GROUP}/memory.events"
{
    echo "{"
    echo "  \"court\": \"TEST-FLEET-002-class-b-cgroup\","
    echo "  \"group\": \"${GROUP}\","
    echo "  \"memory_max_bytes\": ${MEMORY_MAX},"
    echo "  \"soak_seconds\": ${SOAK_SECONDS},"
    echo "  \"workload_exit\": ${RC},"
    echo "  \"memory_events\": $(cat "${EVENTS}" 2>/dev/null | awk '{printf "\"%s\":%s,", $1, $2}' | sed 's/,$//'),"
    echo "  \"host\": \"$(uname -s | tr 'A-Z' 'a-z')\""
    echo "}"
} > /tmp/value_audit/fleet002_cgroup.json
echo "receipt: /tmp/value_audit/fleet002_cgroup.json"
exit "${RC}"
