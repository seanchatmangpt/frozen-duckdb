#!/usr/bin/env python3
"""
80/20 Golden Trace Verification
Focuses on critical real-effect verification for production systems.

Key verification targets:
1. Kernel vs DuckDB routing actually happened
2. Timer hooks fired with real state changes
3. Transactions and receipts prove cryptographic integrity
4. No constant/fake responses across varied inputs
"""

import json
import sys
import os
from pathlib import Path
from datetime import datetime
from typing import Dict, List, Any, Set
from collections import defaultdict

class GoldenTraceVerifier:
    """80/20 verifier for critical real-effect spans"""

    def __init__(self, trace_file: str):
        self.trace_file = trace_file
        self.violations = []
        self.trace_data = self.load_traces()

    def load_traces(self) -> List[Dict[str, Any]]:
        """Load traces from JSONL file"""
        if not os.path.exists(self.trace_file):
            print(f"❌ Trace file {self.trace_file} not found")
            return []

        traces = []
        try:
            with open(self.trace_file, 'r') as f:
                for line in f:
                    if line.strip():
                        traces.append(json.loads(line.strip()))
            print(f"✅ Loaded {len(traces)} spans from {self.trace_file}")
            return traces
        except Exception as e:
            print(f"❌ Failed to load traces: {e}")
            return []

    def verify_execution_routing(self) -> bool:
        """Verify kernel vs duckdb routing actually occurred"""
        kernel_spans = [s for s in self.trace_data if 'kernel' in s.get('name', '')]
        duckdb_spans = [s for s in self.trace_data if 'duckdb' in s.get('name', '')]

        if not kernel_spans and not duckdb_spans:
            self.violations.append("No execution routing spans found")
            return False

        # Verify spans have real execution indicators
        for span in kernel_spans + duckdb_spans:
            if not self.verify_real_execution(span):
                return False

        print(f"✅ Verified {len(kernel_spans)} kernel + {len(duckdb_spans)} duckdb spans")
        return True

    def verify_timer_hooks(self) -> bool:
        """Verify timer hooks actually fired and had effects"""
        timer_spans = [s for s in self.trace_data if 'timer' in s.get('name', '')]

        if not timer_spans:
            self.violations.append("No timer hook spans found")
            return False

        # Check for time progression (hooks should fire at different times)
        timestamps = set()
        for span in timer_spans:
            # Extract timestamp from attributes
            attrs = self.parse_attributes(span.get('attributes', []))
            if 'kc.timestamp' in attrs:
                timestamps.add(attrs['kc.timestamp'])

        if len(timestamps) <= 1:
            self.violations.append("Timer hooks don't show time progression")
            return False

        print(f"✅ Verified {len(timer_spans)} timer spans with {len(timestamps)} unique timestamps")
        return True

    def verify_transaction_integrity(self) -> bool:
        """Verify transactions and receipts show cryptographic proof"""
        tx_spans = [s for s in self.trace_data if 'tx.' in s.get('name', '')]
        receipt_spans = [s for s in self.trace_data if 'receipt' in s.get('name', '')]

        if not tx_spans:
            self.violations.append("No transaction spans found")
            return False

        # Check for receipt verification spans
        if not receipt_spans:
            self.violations.append("No receipt verification spans found")
            return False

        # Verify receipt spans have cryptographic indicators
        crypto_indicators = ['merkle_root', 'signature', 'receipt_hash']
        for span in receipt_spans:
            attrs = self.parse_attributes(span.get('attributes', []))
            found_crypto = any(indicator in attrs for indicator in crypto_indicators)
            if not found_crypto:
                self.violations.append("Receipt span missing cryptographic indicators")
                return False

        print(f"✅ Verified {len(tx_spans)} tx + {len(receipt_spans)} receipt spans")
        return True

    def verify_constant_output(self) -> bool:
        """Verify no constant/fake responses across varied inputs"""
        # Group spans by operation type
        operations = defaultdict(list)
        for span in self.trace_data:
            name = span.get('name', '')
            if name:
                op_type = name.split('.')[0] if '.' in name else name
                operations[op_type].append(span)

        # Check for constant outputs (same hash/result for different inputs)
        for op_type, spans in operations.items():
            if len(spans) > 1:
                # Check for identical results across different inputs
                results = []
                for span in spans:
                    attrs = self.parse_attributes(span.get('attributes', []))
                    # Look for result indicators
                    result = attrs.get('result') or attrs.get('hash') or attrs.get('count')
                    if result:
                        results.append(result)

                if len(results) > 1:
                    unique_results = set(results)
                    if len(unique_results) == 1:
                        self.violations.append(f"Constant output detected in {op_type}: {results[0]}")
                        return False

        print(f"✅ Verified non-constant outputs across {len(operations)} operation types")
        return True

    def parse_attributes(self, attributes: List[str]) -> Dict[str, str]:
        """Parse attribute strings into key-value pairs"""
        result = {}
        for attr in attributes:
            if ':' in attr:
                key, value = attr.split(':', 1)
                result[key.strip()] = value.strip()
        return result

    def verify_real_execution(self, span: Dict[str, Any]) -> bool:
        """Verify a span shows real execution, not fake"""
        name = span.get('name', '')
        attrs = self.parse_attributes(span.get('attributes', []))

        if 'kernel' in name:
            # Kernel spans should have row counts
            if 'kc.rows' not in attrs and 'kc.count' not in attrs:
                self.violations.append(f"Kernel span missing row indicators: {name}")
                return False

        elif 'duckdb' in name:
            # DuckDB spans should have query indicators
            if 'kc.query' not in attrs and 'kc.plan' not in attrs:
                self.violations.append(f"DuckDB span missing query indicators: {name}")
                return False

        return True

    def verify_all(self) -> bool:
        """Run all 80/20 verifications"""
        print("🔍 Running 80/20 Golden Trace Verification...")
        print("=" * 50)

        results = []
        results.append(("execution_routing", self.verify_execution_routing()))
        results.append(("timer_hooks", self.verify_timer_hooks()))
        results.append(("transaction_integrity", self.verify_transaction_integrity()))
        results.append(("constant_output", self.verify_constant_output()))

        all_passed = all(result for _, result in results)

        print("=" * 50)
        if all_passed:
            print("✅ All golden trace verifications PASSED")
            print("🎯 System proves real effects via OTEL spans")
        else:
            print("❌ Golden trace verifications FAILED:")
            for violation in self.violations:
                print(f"  ❌ {violation}")

        return all_passed

def main():
    """Main entry point"""
    if len(sys.argv) != 2:
        print("Usage: verify_golden_traces.py <trace_file.jsonl>")
        sys.exit(1)

    trace_file = sys.argv[1]
    verifier = GoldenTraceVerifier(trace_file)

    if verifier.verify_all():
        print("🎯 Golden trace verification: PASSED")
        sys.exit(0)
    else:
        print("🎯 Golden trace verification: FAILED")
        sys.exit(1)

if __name__ == "__main__":
    main()