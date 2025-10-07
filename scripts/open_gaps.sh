#!/usr/bin/env bash
set -euo pipefail

# Create GitHub issues for each v1 gap with explicit DoD/Gates
# Usage: ./scripts/open_gaps.sh [repo_path]

repo="${1:-.}"

# Define gaps with priority levels
mapfile -t gaps <<'EOF'
BLOCKER:core_parsers Implement real OWL/SHACL parsing (no fallbacks)
BLOCKER:owl_rl Implement OWL RL (full + incremental) and wire to engine
BLOCKER:cli_bindings CLI uses KCura instance methods, not static calls
BLOCKER:ffi_timer FFI timer hook execution + handle casting fixed
BLOCKER:timer_runtime Timers: cron parsing + real execution + receipts
BLOCKER:missing_funcs Implement materialize_full/incremental, receipt verify
HIGH:errors Align error codes with docs (S-, C-, O- families)
HIGH:opt Enable CBO: join order, filter pushdown, projection prune
HIGH:tests Fix golden tests to real KCuraEngine/KCura methods
MEDIUM:benches Implement SLO benches + CI regression gate
MEDIUM:kernels Cost-based router + dtype coverage
MEDIUM:serde Serde derives + FFI round-trip tests
MEDIUM:receipts Real cryptographic verify (Merkle+sig)
LOW:todos Remove all TODO/XXX in library code
EOF

echo "Creating GitHub issues for KCura v1 gaps..."

for g in "${gaps[@]}"; do
  level="${g%%:*}"
  slug="${g#*:}"
  title="${g#*:}"
  
  echo "Creating issue: [${level}] ${title}"
  
  gh issue create \
    --title "[${level}] ${title}" \
    --label "${level},v1,launch-readiness" \
    --body "$(cat <<MD
**Acceptance (DoD):**
- Code implemented (no stubs/unimplemented!) ✅
- Unit + property tests ✅
- Integration/E2E passing ✅
- Docs updated (docs/specs + USER_GUIDE) ✅
- Added to CI gates where applicable ✅

**Linked tests to update:** \`crates/kcura-tests/*\`, benches, FFI E2E
**Owner:** @pod-<choose>

**Implementation Notes:**
- Must pass fake-guard script (no unimplemented!/todo!/panic! stubs)
- Must have real functionality, not hardcoded responses
- Must include proper error handling with documented error codes
- Must have corresponding FFI implementations if applicable
MD
)"
done

echo "✅ All gap issues created successfully"
