# Issue #1 reply — post as repo owner after v1.5.5 publish

> Reply target: @dfeyer, opened 2026-03-23 —
> "Thanks a lot for your package... Curious is you plan a DuckDB 1.5 support ?
> Does this need a lot of effort ?"

> **FENCE (dry-run-publish-pack overclaim law — binding):** post ONLY after
> PUBLISH_RUNBOOK steps 4–8 are complete (all three crates live on crates.io;
> step 3 has already gated the release assets before that). The present-tense
> claims below — "it shipped", "v1.5.5 is out" — are true **only at post
> time**; posting earlier is exactly the release overclaim the fence forbids
> (gate-green ≠ release shipped). `#{PR_NUMBER}` substitutes to **#3**
> (merged, a3a69e4) per runbook step 10. This fence and the meta header above
> are repo-side law — step 10 posts only the body below the `---` separator.

---

Thanks for the kind words, @dfeyer — and sorry for the wait! Good news: it
shipped. **frozen-duckdb v1.5.5 is out, and the crate version now tracks the
upstream DuckDB release it bundles** — so `frozen-duckdb = "1.5.5"` gives you
DuckDB 1.5.5.

It's a drop-in bump: same API surface as before, just pinned to the newer
engine. On macOS, a prebuilt universal dylib (arm64 + x86_64) downloads
automatically on first build and is cached under `~/.frozen-duckdb/`, so there
is nothing to compile; the DuckDB headers are vendored into the crate, so it
even works offline. On platforms without a prebuilt asset (Windows/Linux for
now — those are on the roadmap), the build falls back to compiling DuckDB
v1.5.5 from source automatically.

Details in #{PR_NUMBER}. Let us know if you hit anything on the upgrade!
