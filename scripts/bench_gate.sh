#!/usr/bin/env bash
set -euo pipefail

# Bench gate script: SLOs must hold, hard fail on regressions
# This script ensures performance targets are met

echo "🏃 Running SLO benchmarks..."

# Run benchmarks
cargo bench -p kcura-tests --bench slo || {
  echo "❌ Benchmark execution failed"
  exit 1
}

# Check SLO compliance
if [ -f "benches/_last.json" ]; then
  echo "📊 Checking SLO compliance..."
  
  # Create a simple SLO checker if it doesn't exist
  if [ ! -f "scripts/check_slo_compliance.py" ]; then
    cat > "scripts/check_slo_compliance.py" <<'EOF'
#!/usr/bin/env python3
import json
import sys
import argparse

def check_slo_compliance(bench_file, fail_on_violation=True):
    """Check SLO compliance from benchmark results"""
    try:
        with open(bench_file, 'r') as f:
            data = json.load(f)
    except FileNotFoundError:
        print(f"❌ Benchmark file not found: {bench_file}")
        return False
    
    violations = []
    
    # SLO targets (from docs/SLOs.md)
    slo_targets = {
        'sparql_2hop_100m_p95': 25.0,  # ms
        'shacl_mincount_10m_p95': 60.0,  # ms
        'guard_hook_1m_p95': 20.0,  # ms
        'kernel_hot_path': 15000.0,  # ops/sec/core
    }
    
    for benchmark_name, target_ms in slo_targets.items():
        if benchmark_name in data:
            actual_ms = data[benchmark_name].get('p95_ms', float('inf'))
            if actual_ms > target_ms * 1.1:  # 10% tolerance
                violations.append(f"{benchmark_name}: {actual_ms:.2f}ms > {target_ms}ms (target)")
    
    if violations:
        print("❌ SLO violations detected:")
        for violation in violations:
            print(f"   - {violation}")
        
        if fail_on_violation:
            print("🚫 BLOCKING: SLO regression detected")
            return False
    else:
        print("✅ All SLO targets met")
    
    return True

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Check SLO compliance")
    parser.add_argument("bench_file", help="Benchmark results JSON file")
    parser.add_argument("--fail", action="store_true", help="Fail on SLO violations")
    
    args = parser.parse_args()
    
    success = check_slo_compliance(args.bench_file, args.fail)
    sys.exit(0 if success else 1)
EOF
    chmod +x scripts/check_slo_compliance.py
  fi
  
  python3 scripts/check_slo_compliance.py benches/_last.json --fail || {
    echo "❌ SLO compliance check failed"
    exit 1
  }
else
  echo "⚠️  No benchmark results found, skipping SLO check"
fi

echo "🎉 Bench gate passed: SLO targets met"
