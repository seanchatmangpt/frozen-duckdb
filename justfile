# frozen-duckdb task runner

# Run the full 10-phase doctor verification battery
doctor:
    bash scripts/doctor.sh

# Doctor preflight only (phase 00; fast prerequisite check)
doctor-quick:
    bash scripts/doctor/00_preflight.sh

# Run the doctor dual-pass idempotence court (TEST-025):
# two back-to-back doctor runs at one HEAD + assertions A1-A5
court-x2:
    bash scripts/doctor/dual_pass.sh

# Run all anti-vacuity shim courts (BASELINE/BEFORE/SHIM/AFTER/RESTORE)
courts:
    bash scripts/courts/run_all.sh

# Run the four value-audit courts + CFO cash-audit render (lane V6).
# Every dollar figure in the output derives from the ASSUMPTION-UNVERIFIED
# rate card (operator-supplied list prices, unaudited) in
# scripts/value_audit.sh / warehouse-sim/economics/render_value_audit.ts.
value-audit:
    bash scripts/value_audit.sh

# Re-render the generated surface (Makefile, scripts/verify-gates.sh) from ggen.toml
sync:
    make sync

# Run the rendered gate battery only (scripts/verify-gates.sh)
verify-g:
    make verify-gates
