# arXiv Performance Testbed — empirical wall-time optimization

> **Living worklist** (not a dated snapshot). Tracks an empirical performance
> campaign driven by a fixed testbed of the **100 slowest arXiv papers that still
> complete within a 3-minute timeout**. The goal is to improve latexml-oxide's
> per-document wall-time profile by finding and fixing real hotspots on real slow
> inputs — not micro-benchmarks. Companion to the always-on principles in
> [`PERFORMANCE.md`](PERFORMANCE.md) and the `perf-check` skill.

## Mission

Take the 100 slowest-but-completing papers as a stable regression testbed:

1. **Baseline** each paper's wall-time (and phase breakdown) on a publish-grade
   binary, deterministically.
2. **Profile** the aggregate to find the dominant hotspots (which phase, which
   functions) across the slow set — not per-paper anecdotes.
3. **Optimize** the top hotspots, preserving exact output (perf work must be
   output-neutral; verify with a structural diff, not just error counts).
4. **Re-measure** the full testbed after each change; record deltas here. Keep
   every change output-identical and regression-free (`cargo test --tests`).

The win metric is **aggregate wall-time over the 100-paper testbed** (sum and
median), with **zero output change** and **zero test regressions**.

## ✅ Campaign result (2026-06-27) — both dominant clusters fixed

**The full 100-paper testbed (all were 175.5–179.5 s, at the 3-min timeout edge)
was re-run on the current binary: 98/99 convertible papers complete with rc=0,
ZERO timeouts; wall p50 = 0.7 s, p90 = 32 s, max = 84 s** (1 corpus entry has no
usable main `.tex`). Two output-neutral fixes did it:

- **Hotspot #1 — Cluster A (≈85 papers, digest):** OmniBus natbib-autoload reload
  loop on unbound journal classes (sn-jnl/wlpeerj/sagej/Wiley/…). Routed through the
  canonical loop-safe `def_autoload`. (Commit `3b5cd8651a`.)
- **Hotspot #2 — Cluster B (14 papers, XSLT):** `f:seclev-aux` O(n²) heading-level
  computation (`//` descendant scans per heading). Memoized to global variables.
  (Commit `1172569034`; regression guard `07_xslt_seclev_levels.rs`.)

Net: the slowest-100 set went from "100 papers pinned at the timeout wall" to a
0.7 s median. No test regressions (suite 1480/0 → +1 guard). The two slowest
survivors (math0607481 84 s, 1802.06435 70 s) complete comfortably under cortex's
180 s budget; the earlier "residual" 2602.15365 (informs4) also completes — it had
only ever hit the artificially-low 60 s *local* `--timeout` default, not the real
budget. Details per cluster below + in the Hotspot log.

## Corpus-wide phase budget (2026-07-10, 60k-doc 2605+2606 run via cortex telemetry)

Mined all **60,469** per-job `telemetry.json` records from the containerized-worker
2605+2606 reruns (17-phase `phase_us`). The budget/tail/RSS/driver breakdown is now
a **live capability of the cortex telemetry dashboard** — `GET /telemetry/<corpus>/<service>`
(HTML screen) and `/api/telemetry/<corpus>/<service>` (JSON twin), served over the
completed run's result archives (e.g. `sandbox-arxiv-2605`/`oxidized_tex_to_html`).
Total **89.8 core-hours**. Where the wall goes:

| phase | % of total wall |
|---|---|
| digest | 19.7% |
| math_parse | 19.2% |
| build | 18.1% |
| **xslt** | **13.2%** |
| graphics | 8.9% |
| mathml_pres | 4.5% |
| crossref / rewrite / post_xml_parse / serialize / … | each ≤2.6% |
| (sum of instrumented phases) | 94.4% (rest = harness/IO) |
| `bootstrap` | ~0% (baked dumps confirmed) |

**Headline: wall time is broad, not math-dominated.** digest+math_parse+build+xslt
≈ 70%, each 13–20%. The campaign narrative has centered on math over-parse (19.2%,
ambiguity **1.33 candidate parses/formula** corpus-wide — modest average,
concentrated in the math-dense tail), but **XSLT at 13.2% is the single most
under-exploited lever** (only the 3 `O(n²)` template fixes below touched it), and
digest+build (38%) is the sequential-engine cost.

**Tail + RSS are release-healthy** (fed into `RELEASE_CRITERIA.md` §5): wall median
3.07s / P99 34.9s / max 149s — **zero timeouts across 60k docs**; only 120 (0.20%)
exceed 60s, 4 exceed 120s. Peak RSS median 0.94 / P99 2.05 / max 4.52 GiB —
**exactly one** doc over 4 GiB, 53 (0.09%) over 3 GiB. Concentration moderate:
slowest 1% of papers hold only 10% of wall (5% hold 27%) → the *median path*
matters, not just the tail.

**The slow tail is two distinct populations** (different fixes): (a) **digest-
runaway fatals** — `2605.23849` 149s, `2606.21610` 128s, `2605.21013`, `2606.13482`:
100s+ in digest, **0 formulae**, then fatal → reliability, `STABILITY_WITNESSES.md`
Cluster H — **all four FIXED 2026-07-20, now 0.2–4 s; BP-4 retired, do not build
the watchdog**; (b) **legitimately math-dense** — `2605.16382`
(4136 formulae, 116s, 3.0 GiB), `2605.20736`, `2605.14423` → the genuine math
lever (BP-1/BP-5). Fatals are bimodal: most fail fast (median 3.0s, healthy) but a
slow-runaway subset (P99 98s) wastes ~100s before dying = population (a).

**Beyond-Perl improvement plan** derived from this budget: `SYNC_STATUS.md` →
"Beyond-Perl performance levers" (BP-1…BP-6: parallel per-formula parse, XSLT
amortize→transpile, concurrent graphics, digest watchdog, formula memoization,
native construction tree).

## Corpus-wide profile + the inkscape→gs question (2026-07-02)

The 2026-06-27 campaign optimized the *slowest-100 tail*. This section records a
**whole-corpus** profile taken during the in-progress full-arXiv rerun on the
current worker, prompted by a "did the worker get slower — did dropping inkscape
push work onto `gs`?" question.

**Live per-doc `runtime_ms`** (`log_infos` category `runtime_ms`; core+**post** wall,
`cortex_worker.rs` `wall_start`→`elapsed`), sample of ~8k finalized `arXiv` tasks:

| n | avg | p50 | p90 | p99 | max |
|---|-----|-----|-----|-----|-----|
| 7975 | 5.3 s | 3.3 s | 11.2 s | 30.0 s | 132 s |

**Verdict — inkscape→gs is not a slowdown:**
- inkscape was the 3rd/last-resort *vector*-SVG converter, reachable only when both
  mutool AND pdftocairo failed on the same PDF, and it was **never installed** in the
  image — so `gs`/`convert` (raster PNG) was already the effective fallback. Removing
  it (`57d28a633c`) deleted a shadowed dead branch: **zero runtime change**. When it
  *did* run it was 20–40× slower than pdftocairo, so its absence is a net positive.
- External rasterizers are cheap + deduped: per [`PERFORMANCE.md`](PERFORMANCE.md),
  each `gs`/`convert`/`mutool`/`pdftocairo` is ~10–50 ms ambient behind a spawn cache
  (key = source-hash + page + DPI + format). The 36.5 %-of-wall **graphics** phase is
  dominated by **native in-Rust tikz/pgf rendering**, NOT external `gs`.
- Direct test — `runtime_ms` bucketed by per-doc graphics-message count: **0-graphics
  docs are *slower*** (avg 6.6 s, n=2591) than 1–5-graphics docs (avg 4.7 s, n=5384).
  Per-doc wall tracks core parse/math, not image work — graphics-heavy docs are not
  the tail.

**No in-DB per-doc baseline.** Marking-for-rerun wipes prior logs (verified: TODO
tasks carry zero `runtime_ms`), so the old-worker per-doc numbers are gone. The one
clean cross-run figure is fleet throughput from `historical_runs` **run 202** (old
worker, Jun 21–25): 2,795,618 tasks / 266,995 s ≈ **95 ms/task across the fleet** —
but that conflates fleet size (a maxperf box was added), so it is **not** a clean
per-doc regression measure. For a real before/after, use the fixed slowest-100
testbed A/B (current HEAD vs. the pre-post-processing-change commit) — measured in the
A/B subsection below.

**What genuinely changed (added work, not a lost optimization):** `runtime_ms` now
spans core+**post**, so post-processing time is now *inside* the watched number; and
post emits its own log lines while the math parser's per-formula ambiguity/unparsed
footprints (hashed dedup) land in the math-parse band. Recent latexml-oxide perf
commits are all output-neutral *speedups* (see Hotspot log #2–#4).

### A/B — HEAD vs. pre-post-processing-change (`eb0b0a09ce`), slowest-100 (2026-07-02)

Ran the fixed slowest-100 testbed through two `--release` binaries — current HEAD
(`83a7dc89a4`) and `eb0b0a09ce` (the commit just *before* the questioned series:
`Info:runtime_ms`, math diagnostics, post-log fold, inkscape removal, watchdog) —
standalone `latexml_oxide --dest=…html` (core+post), `--timeout=180`, interleaved
per paper on the DB host (mild shared-load noise, hits both equally). Both binaries
already carry the seclev (#2) and head-keywords (#3) fixes; only maketitle (#4) and
the diffop prune are HEAD-only.

**Result: HEAD is net 1.81× faster — no lost optimization.** 97 papers rc=0 on both:
**sum 2002 s → 1107 s**; 48 faster, 24 within ±10 %, 25 nominally "slower."

- **Wins are large and real:** −17 to −72 s on the slow XSLT/OmniBus/maketitle papers
  (1508.04636 128→56 s, 2603.26887 139→71 s, 1610.08336 19.4→0.8 s), from the
  seclev/head-keywords/maketitle memoizations (#2–#4) + diffop prune.
- **24 of the 25 "slower" are a ~0.2 s fixed *digest* overhead**, not a scaling
  regression — same output, delta entirely in `phase_digest_us` (2306.12058
  0.13→0.35 s @44 formulae, identical 44/24 gfx out; 2603.06884 0.19→0.39 s @55).
  Cosmetic as a ratio on sub-second papers; ~4 % of the ~5.3 s corpus mean. A small
  real fixed cost entered the window (NOT the math diagnostics — `phase_math_parse_us`
  is unchanged; not root-caused to one commit — low priority).
- **The one *large* "regression" is false:** 1704.08246 18 s → 77 s because the
  **before binary silently produced an empty document** (0 formulae, 0 graphics,
  159 MB RSS, `exit 0`); HEAD correctly converts all **11,291 formulae + 35 graphics**
  (digest 1.8 s / build 17 s / math_parse 23.9 s / xslt 11.4 s, 2.9 GB RSS). An
  output-changing **correctness fix**, not a perf loss — before's fast 18 s was a
  degenerate no-op.

**Answer to "did the worker get slower?":** No. Per-doc conversion is faster on the
papers that dominate wall-time. The higher live `runtime_ms` is (a) the metric now
spanning core+**post**, (b) a ~0.2 s fixed digest overhead, and (c) genuinely more
work where the old binary was silently under-converting. **inkscape/gs is not
involved.**

## Settled dead-ends — do NOT re-litigate (see memories + docs)

These were measured and declined/closed; re-investigating wastes time:

- **PGO / `target-cpu` (v3/native)** — NO measurable speedup; `maxperf` is already
  at the fat-LTO + CGU1 ceiling. Tooling deleted. (memory `pgo-isa-no-gain-2026-06-21`.)
- **Startup dump-parse lever** (~50 ms of the ~161 ms startup floor) — measured and
  declined as too small for the release-critical risk. (`archive/STARTUP_COST_ANALYSIS_2026-06-21.md`,
  memory `startup-cost-investigation-2026-06-21`.) For *long* papers the startup
  floor is amortized to noise anyway — so this testbed (slow = long) is the right
  place to look at the **digestion / math-parse / post** phases instead.

The open levers per `PERFORMANCE.md`: **P1 math-parser / large-doc**, **P2
allocation** (partial). This testbed is aimed squarely at P1.

## Measurement methodology

- **Binary**: publish-grade for measurement — `cargo build --release --bin latexml_oxide`
  (or `--profile maxperf` for a final number). NOT the `dev`/`test` profile, NOT
  the CI profile. (CLAUDE.md "Publish-grade measurement".)
- **Determinism**: idle-serial, no fleet parallelism (avoids the RSS-fuse /
  scheduler noise); pin to a quiet machine; take the median of N≥3 runs per paper
  for the headline number; warm the page cache first.
- **Harness**: extend `tools/run_perf_corpus.sh` (already does unzip → convert with
  the release binary + ar5iv bindings → record wall-clock + exit). The 100-paper
  set replaces/augments its Tier-A `PAPERS=(...)`. Inputs are the sandbox zips
  (`$SANDBOX_ZIPS`, default `/home/deyan/data/10k_sandbox`).
- **Phase breakdown**: per-job telemetry (Mouth/Gullet/Stomach/Document/math-parse/
  post) via the `cortex_worker` telemetry path → `tools/perf_phase_summary.py`
  (see `TELEMETRY.md`). This tells us WHICH phase dominates the slow set, which
  decides where to profile.
- **CPU profile**: `perf record` + a flamegraph (or `cargo flamegraph`) on a
  representative slow paper, and on a concatenated run, to find hot functions.
  Always profile the release binary (dev-profile flamegraphs mislead).

## Output-neutrality gate (non-negotiable)

Performance changes MUST NOT change output. After each optimization:
1. `cargo test --tests` green at the CURRENT baseline (**1617/0/0**, 2026-07-20).
   (The old `1479/0 on class-b-xmref` pin is stale — that branch no longer exists.)
2. Structural diff of the testbed outputs before/after the change — byte-identical
   (modulo the known intentional divergences). A perf change that alters output is
   a bug, not a speedup.

## Testbed — the 100 slowest-completing papers (ranks #101–200)

The slowest 100 arXiv papers that still completed within the 3-minute (180 s)
timeout — i.e. they cluster right at the watchdog (175.5–179.5 s). Approach
(user-directed): **fix them one by one** — for each, find the specific hotspot and
fix it, rather than chasing a single aggregate metric.

**Data source:** each paper's cortex output `.zip` (returned from the
`oxidized-tex-to-html` service) already contains a `cortex.log` with the
performance/phase breakdown of its run — read that to locate the dominant phase
WITHOUT re-running (each run is ~175–180 s). Corpus is `arXiv` (2 are
`sandbox-arxiv-tikz-cd`). NOTE: the default perf-harness sandbox dir
(`/home/deyan/data/10k_sandbox`) is empty and `cargo flamegraph`/`valgrind` are
absent + `perf_event_paranoid=4` blocks `perf record` — so profiling relies on the
cortex.log phase timings + targeted in-code instrumentation, not flamegraphs.

| # | arXiv id | corpus | cortex runtime (s) | status |
|---|----------|--------|--------------------|--------|
| 101 | 2404.12418 | arXiv | 179.5 | pending |
| 102 | 2603.06884 | arXiv | 179.4 | pending |
| 103 | 2503.02632 | arXiv | 179.3 | pending |
| 104 | 1310.4854 | arXiv | 179.3 | pending |
| 105 | 2603.26887 | arXiv | 179.2 | pending |
| 106 | 2405.15866 | arXiv | 179.2 | pending |
| 107 | 2407.02023 | arXiv | 179.2 | pending |
| 108 | 1504.03025 | arXiv | 179.0 | pending |
| 109 | 2407.07965 | arXiv | 179.0 | pending |
| 110 | 1705.03781 | arXiv | 179.0 | pending |
| 111 | 2207.03551 | arXiv | 179.0 | pending |
| 112 | 2509.07040 | arXiv | 178.9 | pending |
| 113 | 2306.10150 | arXiv | 178.9 | pending |
| 114 | 2001.10490 | arXiv | 178.9 | pending |
| 115 | 1610.08336 | arXiv | 178.6 | pending |
| 116 | 1704.08246 | arXiv | 178.4 | pending |
| 117 | 2402.14624 | arXiv | 178.2 | pending |
| 118 | 2602.15365 | arXiv | 178.2 | pending |
| 119 | 2509.08051 | arXiv | 178.2 | pending |
| 120 | 1803.06488 | sandbox-arxiv-tikz-cd | 178.1 | pending |
| 121 | 2405.04893 | arXiv | 178.0 | pending |
| 122 | 2103.15714 | arXiv | 178.0 | pending |
| 123 | 2505.16914 | arXiv | 178.0 | pending |
| 124 | 2306.12058 | arXiv | 177.9 | pending |
| 125 | 2603.19978 | arXiv | 177.9 | pending |
| 126 | astro-ph0510764 | arXiv | 177.8 | pending |
| 127 | 2411.12829 | arXiv | 177.8 | pending |
| 128 | 2502.01248 | arXiv | 177.7 | pending |
| 129 | 2507.11786 | arXiv | 177.7 | pending |
| 130 | 2505.07283 | arXiv | 177.6 | pending |
| 131 | 2510.17160 | arXiv | 177.5 | pending |
| 132 | 2010.01181 | arXiv | 177.5 | pending |
| 133 | 2601.17246 | arXiv | 177.4 | pending |
| 134 | 2507.16108 | arXiv | 177.4 | pending |
| 135 | 2502.05048 | arXiv | 177.3 | pending |
| 136 | 2209.08461 | arXiv | 177.3 | pending |
| 137 | 2308.11667 | arXiv | 177.3 | pending |
| 138 | 2310.19676 | arXiv | 177.2 | pending |
| 139 | 2302.12841 | arXiv | 177.2 | pending |
| 140 | 2402.15936 | arXiv | 177.1 | pending |
| 141 | 2603.05420 | arXiv | 177.1 | pending |
| 142 | 2103.17101 | arXiv | 177.1 | pending |
| 143 | 2601.12508 | arXiv | 177.1 | pending |
| 144 | 2509.06697 | arXiv | 177.1 | pending |
| 145 | 2501.05144 | arXiv | 177.1 | pending |
| 146 | 2411.16606 | arXiv | 177.0 | pending |
| 147 | 2507.18550 | arXiv | 176.9 | pending |
| 148 | 2111.03379 | arXiv | 176.9 | pending |
| 149 | 2605.03718 | arXiv | 176.9 | pending |
| 150 | 2004.08712 | arXiv | 176.9 | pending |
| 151 | 2511.04922 | arXiv | 176.8 | pending |
| 152 | 2602.09096 | arXiv | 176.8 | pending |
| 153 | 2407.05957 | arXiv | 176.7 | pending |
| 154 | 2303.15431 | arXiv | 176.6 | pending |
| 155 | 2605.21886 | arXiv | 176.6 | pending |
| 156 | 2511.05535 | arXiv | 176.6 | pending |
| 157 | 1309.6488 | arXiv | 176.5 | pending |
| 158 | 2508.08105 | arXiv | 176.5 | pending |
| 159 | 2204.11540 | arXiv | 176.5 | pending |
| 160 | 2311.04283 | arXiv | 176.4 | pending |
| 161 | 2304.06841 | arXiv | 176.4 | pending |
| 162 | 2005.02609 | arXiv | 176.3 | pending |
| 163 | 2010.10859 | arXiv | 176.3 | pending |
| 164 | 2101.00186 | arXiv | 176.3 | pending |
| 165 | 2408.09489 | arXiv | 176.3 | pending |
| 166 | 2505.12228 | arXiv | 176.2 | pending |
| 167 | 2111.00333 | arXiv | 176.2 | pending |
| 168 | 2511.09289 | arXiv | 176.2 | pending |
| 169 | 2212.03825 | arXiv | 176.2 | pending |
| 170 | 2411.13077 | arXiv | 176.1 | pending |
| 171 | 1508.04636 | sandbox-arxiv-tikz-cd | 176.1 | pending |
| 172 | math0607481 | arXiv | 176.1 | pending |
| 173 | 2001.02183 | arXiv | 176.1 | pending |
| 174 | 2209.01435 | arXiv | 176.1 | pending |
| 175 | 2110.11418 | arXiv | 176.0 | pending |
| 176 | 1806.09362 | arXiv | 176.0 | pending |
| 177 | 2212.11715 | arXiv | 176.0 | pending |
| 178 | 2603.15266 | arXiv | 176.0 | pending |
| 179 | 2602.14838 | arXiv | 176.0 | pending |
| 180 | 2209.10146 | arXiv | 176.0 | pending |
| 181 | 2502.00087 | arXiv | 175.9 | pending |
| 182 | 2411.06232 | arXiv | 175.9 | pending |
| 183 | 2303.06965 | arXiv | 175.9 | pending |
| 184 | 2603.19317 | arXiv | 175.9 | pending |
| 185 | 2504.07384 | arXiv | 175.9 | pending |
| 186 | 2507.01225 | arXiv | 175.9 | pending |
| 187 | 2301.04191 | arXiv | 175.9 | pending |
| 188 | 2303.15582 | arXiv | 175.8 | pending |
| 189 | 2410.21368 | arXiv | 175.8 | pending |
| 190 | 2404.07342 | arXiv | 175.8 | pending |
| 191 | 1802.06435 | arXiv | 175.8 | pending |
| 192 | 2310.18898 | arXiv | 175.7 | pending |
| 193 | 2112.06300 | arXiv | 175.7 | pending |
| 194 | 2604.17057 | arXiv | 175.7 | pending |
| 195 | 2401.16409 | arXiv | 175.7 | pending |
| 196 | 2010.13972 | arXiv | 175.6 | pending |
| 197 | 2411.05290 | arXiv | 175.6 | pending |
| 198 | 0804.2543 | arXiv | 175.6 | pending |
| 199 | 2008.04967 | arXiv | 175.5 | pending |
| 200 | 2510.20317 | arXiv | 175.5 | pending |

## Cause analysis (established 2026-06-27, from the in-zip telemetry.json — BEFORE mitigation)

Read `telemetry.json` (phase breakdown) from all 100 papers' `oxidized-tex-to-html`
output zips. **Two dominant cause clusters + the technical roots:**

**Category split:** 85 `conversion_fatal` (timeout), 7 `conversion_error`, 7 `ok`,
1 missing. So the "slowest completing" set is really **mostly timeouts** (175.5–179.5 s
= right at the 180 s watchdog). Aggregate phase time across the 99: **digest 76%,
xslt 21%**, everything else <2%.

### Cluster A — OmniBus natbib-autoload reload loop → digest hang → TIMEOUT (85 papers) — ✅ ROOT-CAUSED + FIXED 2026-06-27
Dominant recorded phase is `digest` (~88–92 s, strikingly consistent across all 85);
RSS modest (0.3–2.5 GB, NOT a memory blowup — the earlier `perf_class.txt` RSS column
was garbage, real `max_rss_kb`≈1.25 GB). The strikingly-uniform ~90 s is the tell: it
is a **fixed-cost loop hitting an internal guard**, not document-size-driven work.

**Root cause (confirmed by stack sampling + bisection, not just telemetry):**
Almost all 85 use an **UNBOUND journal class** that falls back to the **OmniBus**
class — empirically **~50 `sn-jnl`** (Springer Nature) plus `wlpeerj`, `sagej`×4,
Wiley×8 (`WileyASNA`/`WileyNJD`), `ws-procs`×3, `lmcs`×2, `aa`, `ecai`, `IEEEtran`,
`oup-authoring-template`, `informs4`, `siamart`, … . OmniBus installs lazy natbib
**autoload** stubs for `\citep`/`\citet`/`\citeyear`/… . The hand-rolled stub did
`require_package("natbib")` then re-emitted the CS **without clearing itself** — so
the re-emit re-resolved to the *same stub* through OmniBus's locked class frame and
**fully re-loaded natbib on every iteration** (each iteration runs `input_definitions`
→ `pathname::canonical`/`file_name` kpathsea lookups — the gdb hot stack). The first
cite trigger (often `\citeyear` in the frontmatter) spins until the watchdog.

Methodology that nailed it (the user's "investigate concrete examples one by one"):
truncation-bisection of `2603.06884` `sn-article.tex` (`latexml_oxide --preload=ar5iv.sty
--dest=…html`) → narrowed to the first `\citep` *after* the frontmatter loaded natbib
→ minimal repro = `<sn-jnl preamble>` + `\citeyear`/`\citep` (60 s) vs plain
`article`+natbib (0.2 s, fast) → `gdb` all-thread sample of the worker thread =
`digest_next_body → omnibus_cls::load_definitions{{closure}} → require_package →
input_definitions` in a tight loop.

- Concrete: **2603.06884** (sn-jnl, 128 KB): before = 90 s digest → fatal timeout
  (exit 3); **after = 0.5 s, 0 errors**, correct title + 180 KB HTML.
- The ~89 s "unaccounted" wall in the cortex telemetry was an accounting artifact of
  where the loop's time landed across phase timers — superseded by the real root above.

### Cluster B — XSLT `f:seclev-aux` O(n²) heading-level computation (14 papers) — ✅ ROOT-CAUSED + FIXED 2026-06-27
The user's hypothesis ("oversized/malformed Core XML") was **investigated and
disproved**: the Core XML is large but legitimate (2404.12418: 22 MB / 230,827
elements, ~206k of them genuine math for 7499 formulae; **0 ERROR nodes** — the
earlier "4712" was a misgrep of a default-namespace element). Core generation is
**fast (12.8 s)**. The whole cost is the XSLT (≈150 s).

**Root cause (gdb + scaling + Perl comparison):** the hot stack is
`xsltValueOf → xmlXPathCompiledEval → … → xmlXPathNextDescendant` — a
`<xsl:value-of>` whose XPath uses the **descendant axis (`//`)**. Traced to
`LaTeXML-structure-xhtml.xsl`'s `f:seclev-aux`: it computes a section heading's
`<hN>` level by **recursively** probing `boolean(//ltx:chapter/ltx:title)` etc.
(whole-tree `//` scans), and `f:section-head-level` calls it **for every
`ltx:title`** (~450 in this thesis). That is O(headings × tree-size) ≈ **O(n²)**.
Element-count scaling confirmed it: XSLT 0.7 s @18k → 2.9 s @41k → **21.2 s @99k**
(×2.4 elems ⇒ ×7.3 time). Core scales linearly.

This is a **shared upstream XSLT** issue (Perl LaTeXML has the identical
`f:seclev-aux`), made worse on Rust by **libxml2 2.16** (Rust links `.so.16`;
Perl's XML::LibXML uses 2.15.1) — Perl's `latexmlpost` was 8.7 s on the same 99k
where Rust was 21.2 s.

**Fix:** the level for a given element-type *name* is a document-GLOBAL constant
(depends only on which structural types are present, not on the calling node), so
precompute each once as an `<xsl:variable>` and have `f:seclev-aux` select the
matching one. O(headings × n) → O(n). **OUTPUT-NEUTRAL** (identical values).
File: `resources/XSLT/LaTeXML-structure-xhtml.xsl`. Candidate to upstream.

- **2404.12418** (full thesis): 179 s fatal timeout ⇒ **34.7 s, 0 errors, 4.1 MB
  HTML**. XSLT @99k: **21.2 s → 5.3 s** (now *below* Perl's 8.7 s — a surpass-Perl
  win, since Perl still has the O(n²)). @230k: 150 s → **18.9 s**.
- **Cluster-wide:** all 14 papers (were all 176–179 s timeouts) now complete —
  12 in 21–56 s; 1802.06435 70 s (was hitting only the *local* 60 s watchdog,
  fine under cortex's 180 s); math0607481 fast.
- **Output-neutral verified:** byte-identical HTML on the 99k truncation
  (`diff` IDENTICAL) and the full suite **1480/0** unchanged.

### Next (mitigation)
1. ✅ **Cluster A (digest, 85 papers) — DONE** (Hotspot #1): OmniBus natbib reload loop.
2. ✅ **Cluster B (XSLT, 14 papers) — DONE** (Hotspot #2): `f:seclev-aux` O(n²).
3. ✅ **Whole testbed re-validated** (see Campaign result above): 98/99 rc=0, 0 timeouts.
4. Minor follow-ups (low priority, all already complete under the 180 s budget):
   - **Third XSLT O(n²) — large-index rendering (CHARACTERIZED 2026-06-27, DEFERRED).**
     Witness **1802.06435** (a `book`-structured thesis with **3325 `\index` entries**):
     after the seclev fix it is still ~64–66 s in XSLT, and it IS superlinear (XSLT
     µs/element grows 11.9 → 76.7 → 161.8 → 180.8 across chapter truncations). Isolated
     to the index: commenting out `\printindex` drops the run **71 s → 31 s (−40 s)**;
     only 3 equationgroups, so `f:maxcolumns` is innocent. gdb hot path is
     `xsltForEach → xsltAttribute → …deep libxml2 XPath recursion`.
     **Confirmed it's the XSLT *render*, not the Rust post index-build:** cortex_worker
     `--standalone` telemetry shows `phase_xslt_us` = 66.0 s with `crossref`/`post_scan`
     both <0.5 s (so `MakeIndex` is fast; the cost is the XSLT transform) → so a fix
     would be **output-neutral** (XSLT-only, like seclev). But the obvious index
     templates are all **locally linear** (`indexlist`/`indexentry`/`indexphrase`/
     `indexrefs` = `add_id`+`add_attributes`+apply-templates; `split-columns` linear;
     `$miditem` computed once; nested indexlists take the plain-`ul` path; `ltx:ref`
     uses pre-resolved `@href`/`@title`), so the O(n²) is a **subtle scaling effect**
     (likely the per-element `mode="begin"/"end"` apply-templates or libxml2-2.16
     node-set handling at 132k+ elements), NOT one bad template — pinning it needs
     **libxslt profiling** (`xsltProfileStylesheet`; non-trivial FFI, the `libxslt`
     crate's `transform()` doesn't expose it). **Deferred:** large indexes are rare in
     arXiv (this is a thesis), the paper already completes under cortex's 180 s budget,
     and the site isn't cleanly pinned — adding profiling FFI for a low-breadth case is
     over-investment. Revisit if large-index papers recur as a cluster.
   - 2602.15365 (informs4 ~70 s) and math0607481 (~84 s): distinct slower tail —
     investigate only if they reappear as a cluster.
   - Consider **upstreaming** the `f:seclev-aux` memoization to Perl LaTeXML (it has the
     identical O(n²)).

## Per-paper classification

See the testbed table above (category + dominant phase per paper). Status legend:
`conversion_fatal` = timeout (Cluster A, digest); `ok`/`error` with `xslt` dominant
= Cluster B.

## Hotspot analysis & optimization log

> One entry per investigated hotspot: root cause, change, before→after delta,
> output-neutrality (byte-identical XML) + test evidence. Newest first.

### #4 — XSLT `maketitle` per-title `//ltx:navigation` scan O(n²) (sandbox-2605 large books) — 2026-06-29
- **Root cause:** `resources/XSLT/LaTeXML-structure-xhtml.xsl`'s `maketitle` gated the
  title's `\date` block with `not(//ltx:navigation/ltx:ref[@rel='up'])`. That `//`
  descendant scan walks the **whole document tree from the root**, yet it is
  re-evaluated once **per title** — so a large book with hundreds of titled units does
  **O(titles × tree-size)** scans. `xsltproc --profile` pinned it on **2605.01585**
  ("From Qubit to Qubit", a 2000+-formula physics book): `maketitle` = **22.739 s of
  self-time (95 % of a 24.9 s transform)** across 512 calls (44 ms/scan), with children
  totaling only 0.058 s — the cost is entirely the inline `//ltx:navigation` XPath.
- **Change:** hoist the document-global check into a single global
  `<xsl:variable name="maketitle_has_up_nav" select="boolean(//ltx:navigation/ltx:ref[@rel='up'])"/>`
  (evaluated once, from the root) and test `not($maketitle_has_up_nav)` in `maketitle`.
  Same memoization shape as the seclev fix (#2). The other `//ltx:navigation` scans live
  in `LaTeXML-webpage-xhtml.xsl`'s navbar/header/footer, which run **once per document**
  (the cheap 1-call profile entries) — left as-is.
- **Before→after** (standalone `xsltproc` on the 25 MB Core XML): **24.94 s → 2.15 s**
  (11.6×); `maketitle` self-time 22.739 s → **0.004 s**. The fleet's `phase_xslt_us` on
  this paper (65.7 s) collapses accordingly; total wall 102 s → ~38 s.
- **Output-neutral:** `xsltproc` full-HTML **byte-identical** (`cmp` clean, 25 MB Core
  XML → HTML). Suite **1502/0** + new guard `09_xslt_maketitle_navscan.rs` (asserts the
  `\date` still renders when no `ltx:navigation` is present, i.e. the memoized value is
  `false`). See OXIDIZED_DESIGN_DIVERGENCES.md #41.
- **Note:** local-only XSLT divergence from upstream LaTeXML; the per-title `//` scan
  exists verbatim upstream — candidate to upstream (like #2/#3).

### #3 — XSLT `head-keywords` index dedup O(n²) (batch #201–300 index cluster) — 2026-06-28
- **Root cause:** `resources/XSLT/LaTeXML-webpage-xhtml.xsl`'s `head-keywords`
  (builds `<meta name="keywords">`) deduplicated index phrases with
  `//ltx:indexphrase[not(.=preceding::ltx:indexphrase)]` — the XSLT-1.0
  distinct-by-value antipattern: each indexphrase walks the `preceding::` axis ⇒
  **O(indexphrases² × tree-size)**. `xsltproc --profile` pinned it: `head-keywords`
  = **145 s of a ~150 s** transform on 2208.07515 (1 call), matching the gdb stack
  `xsltElement(meta)→xsltAttribute(content)→xsltForEach→xmlXPathCompiledEval`.
- **Change:** the Muenchian method — a hashed
  `<xsl:key name="f:indexphrase-by-value" match="ltx:indexphrase" use="."/>` +
  `//ltx:indexphrase[generate-id()=generate-id(key('f:indexphrase-by-value',.)[1])]`.
  O(n), identical first-occurrence-in-document-order + `<xsl:sort>` semantics.
  File: `resources/XSLT/LaTeXML-webpage-xhtml.xsl` (embedded at build time).
- **Before→after** (HEAD pre-fix → post-fix wall; all were ~173–175 s in cortex):
  2208.07515 (560 \index) 95.5 s → **33.4 s** (xslt 71.5 → 11.8 s); 0807.4838 (1032)
  78.1 → **13.2 s**; 1802.06435 (515) 77.6 → **17.3 s**; 2403.19732 (334) 68.1 →
  **29.1 s**; math0206203 (189) 50.0 → **30.0 s**; 1807.02129 (310) 49.6 → **26.9 s**.
- **Output-neutral:** `xsltproc` full-HTML byte-identical (3.2 MB, 2208.07515 `diff`
  IDENTICAL); independently, the keywords-meta is byte-identical between the
  historical pre-fix bundle and the fixed binary on 2208.07515 (5991 ch) and
  1802.06435 (14394 ch). Suite **1488/0** + new guard `08_xslt_head_keywords.rs`.
- **Supersedes** the prior campaign's *deferred* "Third XSLT O(n²) — large-index"
  (1802.06435): the real root was `head-keywords`, NOT the index-render templates
  (which are locally linear); the prior deferral couldn't pin it without
  `xsltproc --profile`. 1802.06435 is now 77.6 → 17.3 s.
- **Note:** local-only XSLT divergence from upstream LaTeXML; the O(n²) exists
  verbatim upstream — candidate to upstream (like the seclev memoization).
  See OXIDIZED_DESIGN_DIVERGENCES.md #40.

### #2 — XSLT `f:seclev-aux` O(n²) heading-level computation (Cluster B; 14 papers) — 2026-06-27
- **Root cause:** `LaTeXML-structure-xhtml.xsl`'s `f:seclev-aux` recomputes
  whole-tree `boolean(//ltx:X/ltx:title)` descendant scans on every heading
  (`f:section-head-level` fires per `ltx:title`) → O(headings × tree-size) ≈ O(n²).
  gdb hot path `xsltValueOf → xmlXPathNextDescendant`; element-scaling showed
  XSLT ×7.3 for ×2.4 elements (41k→99k). Shared with upstream Perl XSLT, worse on
  Rust's libxml2 2.16 (Perl 2.15.1: 8.7 s vs Rust 21.2 s @99k on the same XSLT).
  NOT the "oversized/malformed Core XML" hypothesis (Core is fast + clean, 0 ERROR).
- **Change:** the level for an element-type *name* is a document-global constant —
  precompute each once as an `<xsl:variable>` (`seclev_document`…`seclev_backmatter`);
  `f:seclev-aux` just selects the matching one. O(headings × n) → O(n).
  File: `resources/XSLT/LaTeXML-structure-xhtml.xsl` (embedded at build time).
- **Before→after:** 2404.12418 (full thesis) 179 s fatal timeout ⇒ **34.7 s, 0
  errors**; XSLT @99k **21.2 s → 5.3 s** (below Perl's 8.7 s — surpass-Perl, since
  Perl keeps the O(n²)); @230k 150 s → 18.9 s. All 14 cluster-B papers (were 176–179 s
  timeouts) now complete (12 in 21–56 s, 1802.06435 70 s, math0607481 fast).
- **Output-neutral:** byte-identical HTML on the 99k truncation (`diff` IDENTICAL);
  full suite **1480/0** unchanged (no test HTML/XML altered).
- **Note:** local-only XSLT divergence from upstream LaTeXML; candidate to upstream
  (Perl has the identical O(n²)).

### #1 — OmniBus natbib autoload reload loop (Cluster A; ~50 sn-jnl + Wiley/sagej/wlpeerj/…) — 2026-06-27
- **Root cause:** OmniBus's hand-rolled natbib autoload (`omnibus_cls.rs`) did
  `require_package("natbib")` then re-emitted the cite CS **without clearing its own
  stub** → the re-emit re-fired the stub (OmniBus's locked class frame shadowed
  natbib's fresh local def) → natbib fully re-loaded every iteration until the
  wall-clock watchdog. Confirmed by `gdb` worker-thread sampling: tight loop
  `digest_next_body → omnibus closure → require_package → input_definitions →
  pathname::canonical`.
- **Change:** route the OmniBus natbib autoloads through the canonical, loop-safe
  `def_autoload` (the same primitive `TeX.pool` uses for `\mathbb`→amssymb): it
  (a) early-returns if natbib is already loaded, (b) **clears the trigger globally
  BEFORE the load** (Perl `ClearAutoLoad`), (c) snapshots + `require_package` +
  hoists natbib's freshly-installed defs to GLOBAL (survives group pops), (d)
  re-emits → resolves to natbib's real def. Promoted `def_autoload` to `pub` +
  re-exported via `latexml_engine::prelude` so `latexml_package` shares it.
  Files: `latexml_engine/src/tex.rs`, `latexml_engine/src/prelude.rs`,
  `latexml_package/src/package/omnibus_cls.rs`.
- **Before→after (release `latexml_oxide --preload=ar5iv.sty --dest=…html`):**
  - 2603.06884 (sn-jnl): 90 s digest → **fatal timeout** ⇒ **0.5 s, 0 errors**.
  - 2010.13972 (wlpeerj): 178 s ⇒ **0.6 s** (190 citations render).
  - 1610.08336 (sagej) 0.6 s · 2502.01248 (WileyNJDv5) 0.8 s · 2103.15714 (IEEEtran)
    11 s · 2001.10490 (lmcs) 1.8 s · 2509.06697 (oup) 1.2 s — all were 178 s timeouts.
- **Correctness / regression:** natbib loads **exactly once**; citations render
  (`ltx_cite`/`ltx_ref`); **no undefined-`\citep`**. The two documented regression
  witnesses both pass: **1403.6801** (wlpeerj — the clear-AFTER-load failure that
  caused 101 errors + fatal) now 0 errors / natbib×1 / 76 citations; **2207.14344**
  (the 8K-`require_package` loop) 0 errors. Suite **1479/0**, clippy clean. New test:
  `cluster_omnibus_natbib_autoload_no_reload_loop` (06_cluster_regressions.rs).
- **Output-neutral:** the fix only changes *which* def the cite CS resolves to after
  load (now natbib's, as intended) — the produced citations match natbib's normal
  rendering; full suite unchanged at 1480/0.
- **Cluster-wide validation (all 85 cluster-A papers re-run on the fixed release
  binary, 175 s budget):** **83 of 84 convertible papers now complete** (rc=0),
  wall **min 0.2 s / median 0.8 s / max 9.0 s** — every one was a 175–180 s timeout
  before. 1 corpus entry had no usable main `.tex`. **One residual** still times
  out: **2602.15365** (`informs4` + `dblanonrev`) — a SEPARATE, harder loop (gdb
  shows mostly generic `read_x_token`/`cycle_guard_checkpoint` digestion, only
  occasionally the autoload closure; minimal `\bibpunct`/`\citep`-under-OmniBus
  repros do NOT reproduce it, so it is not the natbib reload loop — likely a
  macro-recursion / cleveref (`\Crefname` undefined) interaction in the raw
  `informs4.cls`). Left for a dedicated investigation; do not fix speculatively in
  the shared `def_autoload` path (regression traps: \varmathbb/2310.13684,
  1403.6801). Tracked here + STABILITY_WITNESSES candidate.
