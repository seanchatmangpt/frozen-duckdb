| workload class | wh cold p50 (ms) | wh amortized p95 (ms) | sovereign p95 (ms) | lat ratio | F1 ≤0.05× | wh egress (B) | sov egress (B) | egress ratio | F2 ≤0.01× | counterfactual $ | invoice $ | F3 ≡$0.00 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| point filter | 5301 | 5301 | 1.483 | 2.80e-4 | CLEAR | 7389 | 0 | 0 | CLEAR | $0.022788 | $0.00 | CLEAR |
| grouped aggregation | 5065 | 5065 | 3.312 | 6.54e-4 | CLEAR | 8052 | 0 | 0 | CLEAR | $0.022468 | $0.00 | CLEAR |
| windowed rolling avg | 5054 | 5054 | 16.818 | 3.33e-3 | CLEAR | 7736 | 0 | 0 | CLEAR | $0.022447 | $0.00 | CLEAR |
| cohort retention journey | 5050 | 5054 | 8.464 | 1.67e-3 | CLEAR | 8076 | 0 | 0 | CLEAR | $0.022444 | $0.00 | CLEAR |
| OCEL rehydration | 5046 | 5051 | 9.261 | 1.83e-3 | CLEAR | 7452 | 0 | 0 | CLEAR | $0.022430 | $0.00 | CLEAR |
