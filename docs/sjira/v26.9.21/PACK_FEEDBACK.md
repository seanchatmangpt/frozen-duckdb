# PACK_FEEDBACK — v26.9.21 wave 5 (gap/staleness crawl → pack feedback)

Findings from the wave-5 crawl tickets (G1..G10, manufactured at 63070af).
One section per ticket; pack-relevant findings only — things whose lawful home
is a pack fact, template, or gate, not repo-side prose debt. Appended by each
G-ticket in its own worktree; coordinator merges.

## G8 — release docs coherence crawl (slice: PUBLISH_RUNBOOK, MILESTONE, PR_BODY, ISSUE_1_COMMENT, TPUB)

Fixed in-repo this wave at the prose layer (lawful per G8 scope: "doc text in
place for prose"); the durable fixes belong upstream in the named packs.

1. **dry-run-publish-pack: phase-1 evidence command hardcodes PR state.** The
   rendered `gates/dry-run-publish-gates.md` phase-1 row carries
   `gh pr view 3 … (OPEN)` — stale the moment the PR merges (now MERGED at
   a3a69e4). The render cannot be hand-edited (器: EDIT THE SOURCES), so the
   stale annotation persists until a source edit + re-render. Pack
   improvement: express the expected PR state as a lifecycle (OPEN→MERGED) or
   bind it live, so phase-1 evidence survives the merge event. Recorded here,
   NOT fixed (owned by the pack/schema sources).
2. **PUBLISH_RUNBOOK is manual composition of three sources** (TPUB base
   render + C4 modeled-gate section + C5 provenance gate), coherent only after
   a wave-5 crawl stitched them: "above/below" wording drift, the provenance
   gate absent from the linear cut sequence and from phase rows 2/6 until
   harmonized. Pack improvement: a runbook-assembly template that renders the
   operator procedure as ONE document — base steps + phase mapping +
   provenance gate composed at render time, not crawl time.
3. **The overclaim fence does not propagate to comms artifacts.**
   DRY-RUN-OVERCLAIM binds the modeled domain and gate language, but
   ISSUE_1_COMMENT.md carried present-tense publication claims ("it shipped",
   "v1.5.5 is out") conditioned on post timing only by a prose subtitle — and
   the runbook's original step-10 command would have posted the meta header
   verbatim. Pack improvement: comms templates render with a structural
   precondition fence header plus a machine-checkable body-extraction rule
   (post-condition: publish steps complete; post the body only).
4. **Merge-event staleness is structural.** Runbook step 1 ("Merge PR #3",
   `# OPEN`) and the `<merge-sha>` tag placeholder went stale at the merge;
   issue #1 flipped to CLOSED by the same merge ("closes #1" auto-closure) —
   three facts in two files needed a crawl to reconcile. Pack improvement:
   parameterize the runbook render on release-state facts (merge SHA, tag
   existence, issue/PR state) or declare the periodic crawl the lawful
   reconciler in the pack itself.
5. **C5 gate placement.** The provenance gate's "run before every cut and at
   close-out" lived outside the linear cut sequence (only the close-out step
   mentioned it). Pack improvement: phase mappings should natively include the
   provenance-gate rows (phase 2 pre-cut law; phase 6 close-out re-run with
   `chain_hash`), as now harmonized in the runbook.
