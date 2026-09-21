# Makefile — frozen-duckdb reconciliation targets (ticket C10, wave 4).
#
# 器 law: generated/consequence files are edited at their owning source
# (docs/GENESIS.md maps every consequence to its source), never as projections.
#
# Targets (non-colliding by design):
#   sync           — ggen sync run (must exit 0 after any config/pack change)
#   genesis-check  — verify every docs/GENESIS.md manifest row: each source
#                    still exists, each consequence is present, upstream pins
#                    still match, pending rows have not silently landed
#   gates          — chain: sync + genesis-check
#
# NOTE: `verify` is deliberately NOT declared here — ticket C8 (verify-gate
# chain) owns Makefile verify. If both branches land, the later merge wins and
# reconciles; until then `gates` is this ticket's chain target.

SHELL := /bin/bash
GENESIS := docs/GENESIS.md

.PHONY: sync genesis-check gates

sync:
	ggen sync run

# Proposed CI check-mode step (documented in GENESIS.md, NOT enabled):
#   ggen sync run --dry-run --format quiet && make genesis-check
genesis-check:
	@set -euo pipefail; \
	fail=0; checked=0; deferred=0; gaps=0; \
	while IFS=$$'\t' read -r cons src status; do \
	  [ -n "$$cons" ] || continue; \
	  label="[$$status] $$cons <- $$src"; \
	  case "$$status" in \
	    pending) \
	      if [ -e "$$src" ]; then \
	        echo "FAIL $$label (source landed — flip row to active in $(GENESIS))"; fail=$$((fail+1)); \
	      else \
	        echo "DEFERRED $$label (source not in this tree yet)"; deferred=$$((deferred+1)); \
	      fi ;; \
	    gap) \
	      if [ -e "$$cons" ]; then \
	        echo "GAP-OK $$label"; gaps=$$((gaps+1)); \
	      else \
	        echo "FAIL $$label (gap file missing)"; fail=$$((fail+1)); \
	      fi ;; \
	    active) \
	      ok=1; \
	      case "$$src" in \
	        upstream:*) \
	          ref="$${src##*@}"; \
	          if [[ "$$ref" =~ ^[0-9a-f]{40}$$ ]]; then \
	            git ls-files -s vendors/duckdb-rs | grep -q "$$ref" || { echo "FAIL $$label (vendors/duckdb-rs gitlink != $$ref)"; ok=0; }; \
	          else \
	            grep -q "$${ref#v}" schema/domain.ttl || { echo "FAIL $$label (pin $$ref absent from schema/domain.ttl)"; ok=0; }; \
	          fi ;; \
	        HANDWRITTEN.md) \
	          grep -qF "$$cons" HANDWRITTEN.md || { echo "FAIL $$label (no ledger row names this path)"; ok=0; }; \
	          [ -e "$$cons" ] || { echo "FAIL $$label (file missing)"; ok=0; } ;; \
	        hand:*|unknown:*) : ;; \
	        *) \
	          if [[ "$$src" == *[\[\*\?]* ]]; then \
	            compgen -G "$$src" >/dev/null || { echo "FAIL $$label (source glob unmatched)"; ok=0; }; \
	          else \
	            [ -e "$$src" ] || { echo "FAIL $$label (source missing)"; ok=0; }; \
	          fi ;; \
	      esac; \
	      case "$$cons" in \
	        ext:*) : ;; \
	        *[\[\*\?]*) \
	          compgen -G "$$cons" >/dev/null || { echo "FAIL $$label (consequence glob unmatched)"; ok=0; } ;; \
	        *) \
	          [ -e "$$cons" ] || { echo "FAIL $$label (consequence missing)"; ok=0; } ;; \
	      esac; \
	      if [ $$ok -eq 1 ]; then echo "OK $$label"; checked=$$((checked+1)); else fail=$$((fail+1)); fi ;; \
	    *) \
	      echo "FAIL $$label (unknown status '$$status')"; fail=$$((fail+1)) ;; \
	  esac; \
	done < <(awk -F'|' '/^\| `/ { c=$$2; s=$$3; t=$$4; gsub(/[` ]/,"",c); gsub(/[` ]/,"",s); gsub(/[` ]/,"",t); print c "\t" s "\t" t }' $(GENESIS)); \
	echo "genesis-check: checked=$$checked deferred=$$deferred gaps=$$gaps failed=$$fail"; \
	[ $$fail -eq 0 ]

gates: sync genesis-check
	@echo "GATES GREEN: sync exit 0 + genesis-check exit 0 (verify remains reserved for ticket C8)"
