# Doctor Dual-Pass Court (TEST-025)

Drift court for the aggregate doctor boundary: `bash scripts/doctor.sh`
must be **idempotent at identical HEAD** — two back-to-back runs at the
same commit must produce the same verdict, and neither run may leak
state into the next.

- Lane: D11 (write) — see `docs/sjira/v26.10.8/_LANES.md`
- Parent spec: `docs/design/doctor-verification-blueprint.md` (TEST-025)
- Status: **PROPOSED** — until two real passes have run at one HEAD
  (post-rebuild) with all assertions green and a receipt embedded below.

## Mechanism

```bash
HEAD=$(git rev-parse HEAD)
bash scripts/doctor.sh > /tmp/doctor-pass1.log 2>&1; rc1=$?
bash scripts/doctor.sh > /tmp/doctor-pass2.log 2>&1; rc2=$?
```

Both passes run back-to-back, no intervening edits, no `make sync`,
no rebuild between them. If `rc1 != rc2`, the court FAILs before any
further assertion.

## Assertions

Each assertion names a real observable; a skip is not a pass.

### A1 — identical exits

`rc1 == rc2`. Doctor is fail-closed (`any_fail` collapse), so a
non-reproducible verdict means hidden order- or state-dependence.

### A2 — zero leaked fixture processes after pass 2

After pass 2 completes, no doctor-owned fixture processes remain:

```bash
pgrep -fl x4writer
pgrep -fl gate_v0
pgrep -fl doctor.*fixture
pgrep -fl flock_holder
```

All four must return no matches (exit 1 from `pgrep` = clean).
Any hit = leaked fixture = FAIL with the `pgrep` output embedded
in the receipt.

### A3 — zero undeclared residue

After pass 2:

```bash
git status --porcelain
ls -d target-lane*/ 2>/dev/null
ls /tmp | grep -E 'doctor|frozen-duckdb|x4writer'
```

- `git status --porcelain` must list nothing beyond the lanes'
  declared in-flight files (coordinator-declared set; any file
  outside that set is residue).
- No `target-lane*/` dirs may be created or left by either pass.
- No new `/tmp/doctor*` / `/tmp/frozen-duckdb*` / `/tmp/x4writer*`
  artifacts beyond the two declared pass logs
  (`/tmp/doctor-pass1.log`, `/tmp/doctor-pass2.log`).

### A4 — verify-evidence.ttl bound to same HEAD + graph hash both passes

Pass 1 and pass 2 must each regenerate
`gates/verify-evidence.ttl` bound to the identical commit and graph
hash. Concretely:

- sha256 of `gates/verify-evidence.ttl` after pass 1 == after pass 2.
- The `HEAD` and graph-hash fields inside the file match
  `git rev-parse HEAD` in both passes.

A differing digest means the doctor writes wall-clock- or
environment-dependent content — the file is not a pure projection,
which breaches the generated-surface law.

### A5 — verify_publish_receipt.py green

```bash
python3 scripts/verify_publish_receipt.py
```

exits 0 after pass 2 (and after pass 1; pass 2 is the court's
binding observation).

## Receipt

To be embedded here after the post-rebuild dual run. Format:
both exit codes, A2 `pgrep` output, A3 `porcelain` + `ls` output,
A4 sha256 pair, A5 exit code. A receipt without pasted output is
SUSPECT.

## Residual checklist

- [ ] Rebuild landed; `scripts/doctor.sh` green single-pass.
- [ ] Pass 1 + pass 2 executed back-to-back at one HEAD.
- [ ] A1..A5 all observed green with output captured.
- [ ] Falsifier executed (below) and court REFUSED the planted leak.
- [ ] Receipt embedded; status flipped PROPOSED -> ALIVE in this
      file AND in the ticket's final History row (they must agree).

## Falsifier (anti-vacuity)

A court that cannot fail is vacuous. Before pass 2, plant a leak:

```bash
sleep 300 & echo $! > /tmp/planted-leak.pid
```

then run the court with the planter active and a `gate_v0`-shaped
process name visible:

```bash
bash -c 'exec -a gate_v01_fixture sleep 300' &
bash scripts/doctor.sh; rc2=$?   # planted before this pass
pkill -f gate_v01_fixture; kill "$(cat /tmp/planted-leak.pid)"
```

The court must FAIL: A2's `pgrep -fl gate_v0` must report the planted
process and the verdict must be FAIL, not PASS. If the court passes
with the leak planted, the court is VACUOUS and its ALIVE standing is
refused.

## Status

PROPOSED. Flip to ALIVE only via the residual checklist above, with
the receipt-gated standing-flip discipline
(`scripts/check_sjira_standing.sh` clean, frontmatter and History row
agreeing).

## Pass-2 invariants (operator, lane H3)

The Pass-2 invariants are: (1) **zero resource leaks** — no doctor-owned
fixture processes (`x4writer`, `e2writer`, `gate_v0*`, `flock_holder`)
survive pass 2, no `target-lane*/` dirs, no undeclared `/tmp` artifacts;
(2) **zero in-memory drift** — pass 2's verdict is reproducible from
pass 1's state at identical HEAD (A1 identical exits, fail-closed);
(3) **idempotent evidence generation** — `scripts/verify-evidence.ttl`
is regenerated as a pure projection each pass (A4 digest stability +
HEAD binding, A5 receipt check green). Implemented: A1-A5 in
`scripts/doctor/dual_pass.sh` cover all three as observed state at one
HEAD. PROPOSED (recorded honestly, assertion not weakened): strict
**bit-for-bit TTL identity across passes** is UNSUPPORTED-BY-CURRENT-
SURFACE — the evidence TTL records wall-clock `recordedAt` timestamps,
so two passes cannot in general produce identical bytes until the
runner normalizes timestamps (or emits them as derived-from-content
values). A4 therefore asserts digest stability only when the doctor
regenerates the file with unchanged content; once the runner
normalizes timestamps, this invariant upgrades to byte-exact equality
with no change to the assertion's meaning.
