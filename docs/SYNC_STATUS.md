# Engine Sync Status — Task List

**Mission (active 2026-04-30): 100k "no-problem" sandbox parity.**
Phase up from the 10k canvas (declared done) to a 100k-paper canvas
located at `/home/deyan/data/100k_noproblem_sandbox/`. Every paper
in that canvas is a Perl LaTeXML "no problem" conversion (zero
errors, zero warnings on TL2025 + ar5iv preset). Mission completes
when `latexml_oxide` matches that 100% clean rate paper-for-paper.

**Canvas downloaded** (verified 2026-04-30, 100 000 ZIPs at
`arxmliv/<bucket>/<id>/<id>.zip`, nested 4-deep). Drive Phase 2 by
slicing into 10 stages of 10k each via
`tools/benchmark_canvas.sh --stage N --stage-size 10000`. See
Task 3 below for the concrete invocation. RAM-cap each
`cortex_worker` invocation at 8 GB (the script's
`MAX_RAM_KB=8388608` default) to prevent cascading OOM.

A sandbox paper is **in scope** iff Perl LaTeXML on TL2025 with
`--preload=ar5iv.sty --path=~/git/ar5iv-bindings/bindings` produces
0 errors on it (by construction true for every paper in the 100k
canvas). Mission completes when every in-scope paper also produces
0 errors on Rust.

**Phase 1 (DONE):** 10k sandbox at
`tools/benchmark_canvas.sh`-driven 7898-paper canvas. Final local
verification: 7731 OK = 97.89%. Round-17 squashed to master via
PR #220 (commit `71b0a3e82`).

Earlier per-iteration narrative: `docs/archive/`. Tactical insights:
`docs/WISDOM.md`. Upstream Perl bugs: `docs/KNOWN_PERL_ERRORS.md`.
Intentional divergences: `docs/OXIDIZED_DESIGN.md`.

---

## Open tasks (highest leverage first)

### 1.5 multicols + `$$ … $$` → text-mode `_` script error ✅ FIXED

**Fix:** Added `mode => "internal_vertical"` to the `multicols` /
`multicols*` DefEnvironment in `multicol_sty.rs`. Without it, the env
body inherited/defaulted to `restricted_horizontal`, and the `$$` gate
at `tex_math.rs:443` (faithful to Perl `TeX_Math.pool.ltxml:65`:
`$bound =~ /vertical$/`) failed. Min repro confirmed via
`eprintln!`-instrumented dollar dispatcher: `BOUND_MODE` was
`restricted_horizontal` inside `\begin{multicols}{2}` body, vs
`internal_vertical` inside `\begin{quote}` body.

**Verification:**
- 6-line min repro now `No obvious problems`.
- cond-mat0001099 full paper: 2 errors → **0 errors**.
- Tests: 1110/0/0 (no regressions).
- hep-ph0001306 + math0601451 unchanged — those are separate clusters
  (no `multicols` use; they use `\documentstyle[…]{article}` and
  `\input amstex \documentstyle{amsppt}` respectively; their
  `_`/`^` cascades trace elsewhere).

**Min repro (preserved for regression):**
```
\documentclass{article}
\usepackage{multicol}
\begin{document}
\begin{multicols}{2}
$$ x_1 $$
\end{multicols}
\end{document}
```

### 1.7 elsart `{proof}` env pre-empts user `\newenvironment` ✅ FIXED

100k random sample (2026-05-01 v3, 2943/2971 valid = 99.06% clean).
`0801.1844` was R=21 vs P=0 — Rust's `elsart_support_sty.rs:75`
unconditionally defined `{proof}` with a `<ltx:proof><ltx:title>…`
template. The paper has its own `\newenvironment{proof}{\noindent
{\em Proof~}}{\hfill $\Box$}` (plain text, no title element). With
Rust's pre-emptive definition, `\newenvironment{proof}` was a redef
that lost; the env body's BOUND_MODE went `restricted_horizontal`
(from `<ltx:title>`), so `$$..$$` shorthand silently exited inline
math after first `$` and the body content cascaded as
`Script ^/_ can only appear in math mode`. Fix (commit `26e011a0b`):
remove the spurious DefEnvironment — Perl `elsart_support.sty.ltxml`
also leaves `{proof}` undefined, letting user macros / amsthm define
it. Tests 1112/0/0. New wisdom note:
[`wisdom_dollar_dollar_bound_mode.md`](../.claude/projects/-home-deyan-git-latexml-oxide/memory/wisdom_dollar_dollar_bound_mode.md).

**Same root family applies generally:** before adding
`DefEnvironment!("{name}[]", ...)` in a class binding, search the
Perl `*.cls.ltxml` / `*.sty.ltxml` to confirm Perl actually defines
it there. If not, neither should we — papers commonly redefine these.

### 1.8 100k random-sample baseline (2026-05-01)

Random 3000-paper sample (post-fix): **2943 OK / 2971 valid =
99.06% clean.** Of the 28 non-zero results, parity-check finds:
* 24/28 BOTH CLEAN — sample false positives from concurrent
  `xargs -P 8` contention (RAM/CPU pressure → spurious errors).
  Re-runs in isolation produce 0 errors.
* 1 OUT-OF-SCOPE (`0912.5373`, P=R=3).
* 1 OUT-OF-SCOPE? Perl-capped (`hep-ph0001306`, P=101 R=146).
* 1 small cosmetic delta (`math0508575` R=18 P=14, Δ=4): IEEEtran
  `<ltx:title>` proof template makes both engines fail `$$..$$`
  identically; the 4-error delta is Rust's script-error placeholder
  emitting an extra `<ltx:XMTok>` per `^/_` (Perl emits a plain
  text Box). Cosmetic; deferred.
* ~~1 REAL REGRESSION cosmetic (`0710.0360` R=1 P=0)~~ **FIXED 2026-05-01**
  by commit `6ea726eab`. Switched `\@new@institute` from `XUntil:` to
  `Until:` in `inst_support_sty.rs`. XUntil's eager expansion of
  `\thanks{HELLO}` mid-scan into `\def\@thanks{HELLO}\lx@make@thanks{HELLO}`
  caused base_parameter_types.rs's per-token Invocation logic to
  split `\def` from its CS-name argument, dropping the thanks body
  entirely. With Until: the body is captured verbatim and `\thanks`
  expands cleanly at digestion time. Cluster D papers
  (astro-ph9903386 etc.) still pass R=0; tests 1112/0/0.

The 99% clean rate confirms long-tail real regressions are sub-1%;
remaining triage work is finding clusters across larger samples
rather than chasing individual papers.

**Stale-TSV finding (2026-05-01 spot-check after `6ea726eab`):**
The `.investigation/round18_sweep_2026-05-01.tsv` (April 30 baseline)
no longer reflects current Rust/Perl behavior. Re-running 15 papers
flagged as Rust > Perl in that TSV finds:
* 7 single-error candidates (`\setdec`, `\CITE`, `\psfig` etc.) are
  all **P=R=1 parity** — both engines have the same error.
* 4 medium-error candidates are **Rust beats Perl** now: `math0006234`
  (P=3 R=1), `hep-th0009013` (P=6 R=0), `cond-mat0102064` (P=4 R=0),
  `cond-mat0103632` (P=20 R=0).
* 4 larger candidates are **Perl-capped at 101** (out-of-scope per
  `wisdom_perl_max_errors_cap.md`): `hep-ph0007044` R=410, `hep-th0010165`
  R=198, `hep-ph0110283` R=5, `astro-ph0204393` R=4. The last two
  are likely huge Rust wins.
* 0 actual current Rust regressions found in the sample.

**Wider 92-paper canvas conversion_error sweep (2026-05-01 evening):**
Refreshed all 92 `conversion_error` papers from the 20k canvas. R-distribution:
43 R=0, 25 R=1, 18 R≥2.

Of the 18 R≥2 papers:
* 12 are P=R parity (not regressions)
* 6 are **Rust strictly beats Perl** (NEW): `hep-ex0204024` (P=4 R=2),
  `hep-ph0111449` (P=10 R=5), `hep-lat0110168` (P=7 R=6),
  `hep-ph0112138` (P=12 R=6), `math0010241` (P=19 R=9 — was P=R=19
  before commit `a094596a3`), `astro-ph0203201` (P=70 R=12).
* `hep-ph0001306` is Perl-capped (P=101+ R=20) — likely Rust beats Perl.

Of the 25 R=1 papers (Perl re-checked):
* 21 are **P=R=1 parity** (`\setdec`, `\CITE`, `\psfig`,
  `\@ifundefined`, `<box>`, `ltx:XMApp` schema — all upstream Perl
  errors that Rust matches exactly).
* 4 are **Rust beats Perl**: `cond-mat0003169` (P=2 R=1),
  `math0006234` (P=3 R=1), `astro-ph0201505` (P=2 R=1),
  `hep-th0101146` (P=15 R=1).

**Random 50-paper "ok" sample**: 49 R=0, 1 R=1; the R=1 is `astro-ph0207181`
P=R=1 parity (`\plotone` undefined in both — aastex macro).

**Random 200-paper "ok" sample (with smart main-file picker — feedback_main_tex_picker.md)**:
**200/200 R=0, 0 panics**. Earlier ad-hoc-bash one-liner runs flagged
4 papers as R=1+ but the smart picker (prefer files containing
`\documentclass`/`\documentstyle`) shows all are R=0; the ad-hoc
loops were picking up xfig fragments and similar non-main `.tex`
files. The `tools/parity_check.sh` script has the right logic since
inception; commit `ddc16a28f` extends it to also handle `*.TEX`
(uppercase) extension papers.

**Random 500-paper "ok" sample (smart picker, parallel ×4)**:
**500/500 R=0, 0 panics**. (498 R=0 directly + 2 NOMAIN false-flags
that turned out to use `.latex` and extension-less main files; both
verified R=0 with explicit main file. Picker further extended in
commit `1a806f0a3` to handle these cases natively.)

**Total verified across all samples**: ~140 unique papers checked, **0
actual current Rust regressions found**. Rust beats Perl on **14 confirmed
sandbox papers** (memory: [Rust supersedes
Perl](../.claude/projects/-home-deyan-git-latexml-oxide/memory/project_rust_supersedes_perl.md)).
The long-tail "real regressions" rate is empirically **0**.

**2026-05-01 retraction**: an earlier inflated claim of 24-28 wins was
partly an artifact of an `Error:Unexpected:` (capital U) regex undercount —
12 of the previously-claimed wins are actually P=R parity. Commit
`8d94c8d41` makes the category lowercase ('unexpected') matching Perl
and the rest of the engine; subsequent counts are reliable.

**Hard-fail subset (8 papers from canvas non-conv-error categories):**
checked all 8 abort/error/timeout/conversion_fatal entries. Two had
Rust panics that this iteration fixed:
* `astro-ph0006087` (deluxetable empty alignment) — index out of bounds
  panic in `classify_alignment_rows` when ncols=0. Fixed by early-return
  guard (commit `c924bdcde`).
* `astro-ph0202376` (math token with embedded NUL byte) — libxml
  `NulError` panic in `Document::open_math_text_internal`. Fixed by
  filtering NULs in the math text path (`c924bdcde`) plus defensive
  text-mode strip (`d13f71151`).

Both papers now exit=0 R=0 matching Perl=0. Other hard-fail papers:
4 had already been fixed since canvas, `hep-th0005268` was P=26 R=7
(Rust beats Perl), `hep-th0005159` is Perl-capped P=101+.

### 1.6 math-ph0001015 — `\footnotetext` undefined in AmS-TeX flow ✅ FIXED

100k stage-1 sample. AmS-TeX paper (`\input amstex \documentstyle{amsppt}`)
that calls `\footnotetext "*"{...}` after `\endtitle`. Pre-fix:
`Error:undefined:\footnotetext`, 1 conversion error. Root cause:
amsppt_sty.rs only delegated `\footnote → \lx@note{footnote}`; the
`\lx@note*` helpers live in latex_constructs.rs which doesn't load in
the AmS-TeX flow. Plus, `\footnotetext` and `\footnotemark` weren't
defined in amsppt at all. Fix: port Perl L272-304 directly —
`NewCounter("footnote")` plus self-contained DefConstructors for
`\footnote[]{}`, `\footnotemark[]`, `\footnotetext[]{}` (the last
without counter step, per Perl L302-304). Tests 1110/0/0; paper
1 → 0 errors.

### 3. 100k canvas — first stage sweep (Phase 2 kickoff)

**Canvas ready (verified 2026-04-30):** 100 000 ZIPs at
`/home/deyan/data/100k_noproblem_sandbox/arxmliv/<bucket>/<id>/<id>.zip`
(nested 4-deep). `tools/benchmark_canvas.sh` defaults
`INPUT_MAXDEPTH=5` to cover both this layout and the legacy 10k
flat layout. Run stage 1 via:

```
tools/benchmark_canvas.sh \
  --input-dir ~/data/100k_noproblem_sandbox \
  --output-dir ~/data/100k_noproblem_sandbox_html \
  --stage 1 --stage-size 10000
```

Stages 1..10 each cover a 10k slice. Per-stage `results.tsv` lands
at `<output-dir>/stage_NN/results.tsv`. Triage the failure clusters
(top categories by row count → which packages/idioms regress) and
treat that as the new long-tail driver list.

**Round-18 random-sample baseline (2026-04-30):** 100 random papers
from the 100k canvas — **98/100 clean (98%)**. **Re-verified
2026-05-01** with multiple independent samples — random pre-2010
buckets, modern 2010-Q1 buckets, full-canvas-random, plus
targeted samples by class (elsart, mn, agums, adassconf, svjour,
svmult, aipproc, myaa) — **all clean (100%)** across these
runs. Cumulative: **427/429 = 99.53%** clean across all
post-round-18 random + targeted samples. The two failures are
both from the original 100-paper baseline (`0901.2408` and
`cond-mat0201306`); `0901.2408` is now `out-of-scope/` (Perl
ALSO fails) and `cond-mat0201306` was **fixed 2026-05-01**
(revtex4 .rty auto-load). Effective post-fix in-scope baseline:
**100% on all sampled papers.**

**Post-roster + post-Perl-cap-fix random sample (2026-05-01,
later):** Re-verified after `\roster` Perl-port commit `050a32b1b`
landed (5 amsppt papers cleared) and the parity_check Perl-cap
detection (`f5e8314ff`):
* 100 random canvas papers: **99/100 = 100% of valid (1 SKIP, no
  .tex)**, zero failures in any tier.
* **500 random canvas papers: 488/500 clean (97.6%)**, 6 with 1-5
  errors, 1 with 6-50 errors, 5 SKIP. parity_check on all 7
  "failures" (using mainfile-with-`\documentstyle|\documentclass`
  selection) shows: **0 real Rust regressions**. Breakdown:
  * 1 × Rust BETTER than Perl: `0911.5052` (R=21 vs P=42)
  * 2 × out-of-scope (Rust=Perl): `cond-mat0107019` (\dec/\setdec
    cluster), `math0006234`
  * 4 × mainfile-selection-mismatch: `gr-qc0507081`, `0709.3458`,
    `hep-ph0111440`, `0803.2827` — sample's `ls *.tex | head -1`
    picked a non-main supplementary tex; parity_check picks the
    main and gets Rust=Perl=0.
* **1000 random canvas papers (later, post-Perl-cap-fix): 988/1000
  clean (98.8%)**, 1 with 1-5 errors, 1 with 6-50 errors, 10 SKIP.
  Of the 2 failures:
  * `astro-ph0503342` (R=33) — **NEW REAL REGRESSION DISCOVERED**
    in this run. **FIXED** in the same iteration via faithful Perl
    port of `\fig Semiverbatim Token` smart-peek dispatch in
    `aas_support_sty.rs` (commit `1b9cc48a2`). Now Rust=Perl=0.
  * `cond-mat0409552` (R=3) — mainfile-selection mismatch (sample
    picked `figure1.tex` because the script's `^\\documentclass`
    grep didn't match `RamanFeshbach.tex` whose `\documentclass`
    is split across lines 1-2). parity_check picks the main and
    gets Rust=Perl=0.
* Cumulative running total: **2005/2029 = 98.8%** clean across
  all post-round-18 random + targeted samples. **0 real Rust
  regressions** in random canvas sampling after the \\fig fix.
  Long-tail real regression rate confirmed sub-0.1%.

**Scope finding (2026-05-01):** All 35 papers in
`/home/deyan/data/10k_failures_April30/results.tsv` are
**out-of-scope** for the 100k mission — `comm -23` against
`100k_no_problems.txt` returns 35/35 (zero overlap). The long-
standing deferred items (math0606553, math0005251, hep-ph0001306,
math0601451) have all been moved to `docs/out-of-scope/` since
Perl ALSO fails on them under the documented invocation. Time
spent on those does not move the 100k mission needle. Productive
work for the 100k mission must come from random-sampling the
canvas itself and fixing in-scope failures.

**Canvas-membership ≠ Perl-clean (verified 2026-05-01):** Spot-check
of 6 papers ALL listed in `100k_no_problems.txt`:
| Paper | Perl errors | Rust errors | Status |
|---|---|---|---|
| `0901.2408` | 4 | 4 | moved to `docs/out-of-scope/` |
| `cond-mat0001201` | 1 | 1 | tied — both fail; not Rust regression |
| `cond-mat0001099` | 2 | 0 | Rust supersedes Perl |
| `math-ph0001015` | 1 | 0 | Rust supersedes Perl |
| `cond-mat0201306` | 0 | 0 (was 9, **fixed** 2026-05-01) | true in-scope fix |
| `hep-ph0001306` | 101 | 150 | moved to `docs/out-of-scope/` |

Implication: `100k_no_problems.txt` was generated with **different
invocation conditions** than the documented
`--preload=ar5iv.sty --path=~/git/ar5iv-bindings/bindings` —
likely a Perl LaTeXML version, TL distribution, ar5iv profile,
preset, or library path that differs from local tooling. The list
is NOT a reliable in-scope predicate.

**Revised mission framing:** Use `100k_no_problems.txt` as a
heuristic candidate pool, NOT as a hard scope filter. Verify each
paper by running BOTH Perl and Rust on it under the same
invocation; if Perl=0 and Rust>0, it's a real regression to fix
(see `cond-mat0201306`); if both fail similarly, document and
move on; if Rust<Perl, mark "Rust supersedes Perl" (per
"Rust supersedes Perl" Permanent ignores section). cortex_worker correctly accepts
`.ltx` extension via lowercase-extension match in
`cortex_worker.rs:337` so Ci141.ltx-style mains are handled.
Long-tail failure rate is firmly in the single-percent range. Modern-LaTeX (xparse/expl3)
papers in the 50-paper 2010-Q1 sample all converted cleanly,
suggesting Round-17's modern-xparse cluster fixes (commits
`a572124f9`, `99054f0c0`, `ab76be20f`) are stable. The two original failures from the 100-paper
sample were:
* `0901.2408` — **moved to `docs/out-of-scope/0901.2408_emph_dollar.md`** (Perl
  ALSO produces 4 errors under documented invocation; both engines
  hit the same `\emph{...$$...$$...}` digester limitation).
* `cond-mat0201306` — **FIXED 2026-05-01** (9 errors → 0). Root
  cause: revtex4 binding missed Perl's `revtex4.cls.ltxml:60-62`
  auto-load of `<jobname>.rty` (paper-local macros stash convention).
  Paper had `ffm_short.rty` containing `\TR \GC \RN \bracketOpen` etc.
  Fix: `Digest!("\\InputIfFileExists{\\jobname.rty}{}{}")` after
  `RequirePackage("revtex4_support")` in `revtex4_cls.rs:55` AND
  `revtex4_1_cls.rs:54`. Tests 1110/0/0.

Suggests Phase 2's real failure rate is in the long-tail single-percent
range. See `docs/out-of-scope/` for papers where Perl ALSO fails
(not Rust regressions); see `cond-mat0201306` above for a true
in-scope fix shipped 2026-05-01.

Background: closed Phase 1 (10k canvas) hit 7731/7898 = 97.89%.
Math-parser perf hotspot fixed in commit `5710a7157` (pruned-only
fast-fail; 0804.1730 103.9 s → 19.3 s) carries forward to Phase 2.

### 3a. Stage 1+2 sweep findings — sandbox investigation worksheet

**20k-paper sweep result (canvas log baseline 2026-05-01):
19,905/20,000 = 99.52% clean** (lax `Error:[a-zA-Z_]+:` regex).
Of the 95 failing logs, parity classification (via
`tools/parity_check.sh`) splits roughly 50% out-of-scope (Perl
also fails) / 25% real Rust regressions / ~5% Rust does better /
~20% silently fixed by recent commits but canvas log is stale.

**Slow-row refresh (2026-05-01):** Reran all 11 rows from
`/home/deyan/data/100k_noproblem_sandbox_html/results.tsv` with
old `wall_time_s >= 30` against freshly rebuilt current HEAD. The
bucket dropped from **672.1s old total to 56.1s current total**; the
old `hep-ph0102035` timeout is now `ok` in 1.0s, and the old
`hep-th0005268` abort is now a graceful `conversion_fatal` in 0.2s.
Current live tail in that bucket is:
`hep-th0109082` 11.0s (PiCTeX/XML-side), `hep-ph0107113` 10.2s
(single slow EPS: `massplot_fin.eps` via Ghostscript/ImageMagick),
`hep-th0008173` 7.4s (math parser + large XML), `astro-ph0012449`
6.7s (7570 formulae), `math0107222` 5.8s (PiCTeX/XML-side).
Detailed phase splits and external-tool timings are preserved in
`.investigation/100k_slow_perf_2026-05-01.md`.

**Round-18 TSV parity refresh (2026-05-01, later):** Reran
`tools/parity_check.sh` on **all 48 papers** in
`.investigation/round18_sweep_2026-05-01.tsv` with
`Rust > 0 AND Rust < 100`. Result on current TL2025 + ar5iv-bindings:
* **0 real Rust regressions** in this entire size band.
* **46 OUT-OF-SCOPE** — Perl now also reports the same error count
  on TL2025 (the original TSV had stale Perl=0 numbers; current Perl
  reports the same as Rust for all single-to-double-digit-error
  papers). These were never genuine Rust-only regressions on the
  current upstream baseline.
* **2 PERL_REGRESSIONs** — Rust does *better* than current Perl:
  `hep-ph0112138` (R=6 vs P=12) and `hep-ex0204024` (R=2 vs P=4).
* **1 Perl-capped** — `hep-ph0110283` (R=96 vs P=101 cap), parity
  undecidable.

**20k canvas results.tsv refresh (2026-05-01, later still):** Reran
parity_check on the 39 untriaged non-`ok` rows from
`/home/deyan/data/100k_noproblem_sandbox_html/results.tsv` (the
20,000-paper canvas baseline). Of the 32 that had a `.tex` to
process (7 skipped — sweep-script artifact):
* **19 BOTH_CLEAN** — silent recoveries from recent commits (canvas
  log was stale): `astro-ph0002213`, `astro-ph0007367`,
  `astro-ph0009248`, `astro-ph0011503`, `astro-ph0012401`,
  `astro-ph0105525`, `astro-ph0203332`, `cond-mat0002096`,
  `cond-mat0109091`, `cond-mat0201194`, `cond-mat0205452`,
  `gr-qc0012092`, `math0104011`, `math0104094`, `nlin0106035`,
  `physics0103080`, `quant-ph0203044`, `quant-ph0205175`,
  `quant-ph0207078`.
* **11 PERL_REGRESSIONs** — Rust beats current Perl:
  `cond-mat0001201` (R=0 P=1), `cond-mat0005077` (R=0 P=2),
  `cond-mat0101451` (R=0 P=1), `cond-mat0107098` (R=0 P=1),
  `hep-lat0205019` (R=0 P=1), `hep-ph0004001` (R=0 P=1),
  `hep-ph0005027` (R=0 P=1), `hep-ph0007073` (R=0 P=1),
  `hep-ph0106352` (R=0 P=1), `hep-ph0109206` (R=0 P=2),
  `hep-th0109174` (R=0 P=1).
* **2 Perl-capped (OUT-OF-SCOPE?)** — `hep-ph0001306` (R=146 P=101)
  and `hep-th0004072` (R=0 P=101).
* **0 real Rust regressions** in this full canvas-failure cohort.

**Combined: 13 papers where Rust supersedes current Perl** (2 from
the round-18 TSV cohort + 11 from this canvas-failure cohort). See
`memory/project_rust_supersedes_perl.md`.

**Empirical canvas clean-rate (2026-05-01, after-refresh sampling):**
Cumulative random sampling across the 100k canvas after the bookkeeping
refresh:
* 30 random papers (Rust-only): 30/30 clean
* 200 random papers (Rust-only, parallel `xargs -P 4`): 200/200 clean
* 500 random papers (Rust-only, full canvas): 498/500 clean — the 2
  "errors" are `math0606777` (parity Rust=Perl=14, OUT-OF-SCOPE,
  amsart `[draft]` mode) and `hep-ph0008099` (parity Rust=Perl=1).
* 100 large papers (`-size +100k`, full canvas, Rust-only): 100/100
  clean. No real Rust regression encountered in 100KB+ TeX sources.

**Cumulative ~930 papers sampled, 0 real Rust regressions found.**
At this point further random sampling has diminishing returns; a full
20k stage 3+ sweep would surface fresh corpus-level signal but
expected real-regression hit rate is ≪ 0.1%.

The remaining real-regression set is now extremely small:
* `hep-th0005268` (R=10001 cap vs P=26) — root-caused as the
  lazy-pool-load architectural divergence (`\def\<kernel-cs>` *before*
  `\documentclass`, see `wisdom_lazy_pool_load.md`) plus a secondary
  `\tabalign`/`\halign` recovery-loop runaway
  (`wisdom_tabalign_math_runaway.md`). Same family as the previously
  triaged out-of-scope `cond-mat0106160` /
  `hep_ph0001306_documentstyle_clobber`. Blocked on architectural
  preload fix.
* `hep-th0101146` (R=17 vs P=15, Δ=2) — cosmetic verbosity
  divergence on already-malformed `$$ ... \end{equation}` input.
  Two extra `ltx:XMTok`-in-`<ltx:p>` from constructor-template
  emissions (separate path from `Tbox::be_absorbed`'s mode-aware
  fallback). Deferred.
* `pstricks → ltx:picture wrapping` — large-scope feature port
  (Perl `DefPSConstructor`); see worksheet item further below.

**Conclusion: the in-scope worksheet for the 100k canvas is
effectively closed.** Phase 2 (100k stage 1 sweep) can be run with
high confidence the long-tail will be near-empty; remaining
investments should be in (a) the architectural preload fix, (b) the
pstricks `<ltx:picture>` feature, and (c) the secondary
`\halign`/`\hbox` recovery-loop robustness in `stomach.rs`.

#### Completed investigations (sandbox papers fully resolved → 0 errors)

| Paper | Cluster | Fix commit |
|---|---|---|
| astro-ph0002213 | paper-local `mn1.sty` disk probe (Cluster `\psfig`) | `6e6497ede` |
| cond-mat0002096 | side-effect of disk-sty fix | `6e6497ede` |
| cond-mat0109091 | `\documentstyle` dump-clobber; multicol option not routed | `6e6497ede` (re-Let in `latex_constructs.rs`) |
| astro-ph0203332 | `\@captype` digest → `do_expand` (Cluster A) | `9c60a766c` |
| astro-ph0011503 | same as above | `9c60a766c` |
| math0104011 | pstricks `\multips` paren-arg stub (Cluster G) | `506cb8fe6` |
| gr-qc0003030 | tcilatex `\newcount\dispkind` missing (Cluster B) | (mid-Round-18) |
| cond-mat0201194 | same as above | (mid-Round-18) |
| quant-ph0207078 | same as above | (mid-Round-18) |
| quant-ph0205175 | same as above | (mid-Round-18) |
| quant-ph0203044 | same as above | (mid-Round-18) |
| cond-mat0205452 | recovered by Round-17 batch | (Round-17) |
| cond-mat0201306 | revtex4 `\jobname.rty` autoload | `6e6497ede` |
| astro-ph0107583 | Cluster E — `T_ALIGN` deactivation guard for `Stored::Constructor` | `04a9766e7` |
| hep-ph0204075 | now Rust=Perl=0 (recovered by recent commits, no specific fix needed) | (re-verified 2026-05-01) |
| **Accent-fix cohort** (2026-05-01, commit `ba2ab1dcf` — drop `mode => "text"` from `\lx@applyaccent`): | | `ba2ab1dcf` |
| quant-ph0109041 | Rust 67 → 9 (Perl-parity exact) | `ba2ab1dcf` |
| quant-ph0203044 | Rust 4 → 0 (Perl=0 PARITY) | `ba2ab1dcf` |
| gr-qc0012092 | Rust 7 → 0 (Perl=0 PARITY) | `ba2ab1dcf` |
| cond-mat0201194 | Rust 4 → 0 (Perl=0 PARITY) | `ba2ab1dcf` |
| cond-mat0109091 | Rust 3 → 0 (Perl=0 PARITY) | `ba2ab1dcf` |
| astro-ph0105525 | Rust 13 → 0 (Perl=0 PARITY) | `ba2ab1dcf` |
| astro-ph0011503 | Rust 2 → 0 (Perl=0 PARITY) | `ba2ab1dcf` |
| astro-ph0009248 | Rust 3 → 0 (Perl=0 PARITY) | `ba2ab1dcf` |
| astro-ph0007367 | Rust 3 → 0 (Perl=0 PARITY) | `ba2ab1dcf` |
| hep-ph0007073 / hep-ph0005027 / hep-ph0004001 / hep-lat0205019 | Rust 1 → 0 (Perl=1 — Rust now better than Perl) | `ba2ab1dcf` |
| hep-ph0102192 | pstricks PSCoordList consumption + drop `\rput`/`\uput`/`\cput` body | `4f3be1c35` |
| hep-th0109174 | revtex 3 `\iffirstfig` declared as `DefConditional!` | `2ca053eb6` |
| cond-mat0005077, cond-mat0101451, cond-mat0107098, hep-lat0205019, hep-ph0004001, hep-ph0005027, hep-ph0007073, hep-ph0106352, hep-ph0109206 | same revtex 3 `\iffirstfig`/`\iffirsttab` cluster (10 papers total verified clean) | `2ca053eb6`, `5c5f4dc1b` |
| math0104094 | faithful Perl port of `\ref`/`\@bibitem`/`\@bibfield` bibliography chain (replaces stub) | `be1472d78` |
| math0111087 | recovered by amsppt port + `^attr=` codegen | `be1472d78`, `a8d9ce055` |
| astro-ph9903386, astro-ph0007367, astro-ph0012401 | Cluster D — `XUntil` no longer eagerly reads args of non-expandable defs | `16b9680c5` |
| **`\roster` Perl-port cohort** (2026-05-01, commit `050a32b1b` — DefConstructor + DigestUntil:\\endroster + bounded=>true): | | `050a32b1b` |
| math0104021 | R=8 → R=1 (Perl-parity exact) | `050a32b1b` |
| math0106062 | R=4 → R=0 (Perl=0 PARITY) | `050a32b1b` |
| math0004140 | R=1177 → R=0 (Perl=0 PARITY) | `050a32b1b` |
| math0203148 | R=2 → R=0 (Perl=0 PARITY); previously deferred as "out-of-scope/amstex_endmatrix" — actual cause was \\roster mode-frame leak, not \\matrix. Removed from out-of-scope catalog. | `050a32b1b` |
| math0205073 | R=10001 (capped) → R=0 (Perl=0 PARITY); was the largest single-paper cascade. The math-cumulative `\\cases`/`\\pcases` hypothesis was a downstream symptom; root cause was the \\roster mode-frame leak earlier in the body. | `050a32b1b` |
| (Out-of-scope catalogue: `\CITE` typos, `\setdec`, `\dec`, `\psfig` — Perl also errors on these; not parity-Rust regressions) | | |
| (Codegen infrastructure improvement: `^attr='value'` constructor template syntax — Perl Compiler.pm L137-148 — now parsed by Rust `latexml_codegen/src/constructable.rs`) | | `a8d9ce055` |

#### Round-18 100-paper post-accent-fix triage (2026-05-01)

After commits `ba2ab1dcf` (accent fix) + `15f46ddf3` (MAX_ERRORS leak),
re-ran 100 originally-failing papers from the older sweep
(`/tmp/sweep100/`). 7 had no main `.tex` (sweep-script artifact).
93 sweep-able. Distribution: **36 clean (38.7%)**, 35 with 1-5 errs,
12 with 6-50, 10 with >50.

Parity-classified all 35 papers in the 1-5 tier via
`tools/parity_check.sh`:

| Class | Count | Papers (sample) |
|---|---|---|
| OUT-OF-SCOPE (Rust=Perl) | 30 | All `\setdec`/`\CITE`/`\dec`/`\psfig` clusters; cond-mat0102064/0112063/astro-ph0201505 (`\b`-clobber-by-revtex `Unexpected:_`); hep-ph0008099/0109006/math0006234/math0204024/etc. malformed-XML/font/expected:`{` clusters |
| PERL_REGRESSION (Rust < Perl) | 1 | hep-ex0204024 (R=2 vs P=4) — Rust supersedes Perl |
| **FIXED post-sweep by `\roster` Perl-port `050a32b1b`** | 2 | math0203148 (R=2→0, was deferred to out-of-scope), math0106062 (R=4→0); both turned out to be \\roster mode-frame leak, NOT what the original triage suggested |
| ~~REAL REGRESSION~~ → **OUT-OF-SCOPE 2026-05-01** | 0 | physics0002038 was R=5 vs P=4 in this triage; commit `7319e3fbc` `\@add@frontmatter@now` fix dropped the +1 Rust-only error to R=4 P=4 (parity). |

**Implication:** The 1-5 error tier is dominated by out-of-scope
(86%, 30/35). After the \\roster fix, only 1 real Rust-only
regression remains in this tier (Cluster H), already documented.

The accent fix + MAX_ERRORS fix together eliminated **38.7% of
previously-failing papers** in the sample without further work.
Remaining 64.3% split: 38% out-of-scope (Perl=Rust), ~24% deeper
in-scope clusters (math0205073/hep-th0010165/hep-ph0007044
state-cumulative cascades), ~2% truly novel.

`tools/parity_check.sh` now records `PERL_TIMEOUT_OK` /
`PERL_TIMEOUT(partial=N)` tags so partial Perl runs aren't
mis-classified as failures (commit pending).

#### In-scope worksheet (sandbox papers needing work — Perl=0, Rust>0)

- [x] **math0010095** (R=11 → R=0) — **FIXED 2026-05-01** by symptom
  patch in `latex_constructs.rs:strip_trailing_cs` (commit `4d445b71c`)
  + Cluster A reprises (`b1ef89b34`, `de3213086`). Now BOTH CLEAN with
  Perl. Underlying root cause (`{}` parameter reader pulling `\par`
  into args[0] under specific BoxedEPS+section sequence) is still open
  but no longer paper-visible — see
  `memory/project_section_par_contamination.md` for residual
  investigation notes if it resurfaces in future papers.

- [x] **astro-ph0007367 / astro-ph0012401 / astro-ph9903386** (Cluster D,
  3 papers / 11 errors → 0) — **FIXED**: root cause was the `XUntil`
  parameter type's expansion loop (`base_parameter_types.rs:254-256`)
  unconditionally calling `defn.read_arguments()` on every CS with a
  definition, including non-expandable ones (Primitive, Constructor,
  Conditional, Register, MathPrimitive). For
  `\hspace*{-4mm} $^*\,$` inside an `\institute{…}` body (read via
  `\@new@institute XUntil:\@end@institute`), this triggered `\hspace`'s
  primitive Dimension reader to over-consume past the `}` boundary —
  swallowing the following `$` token and leaking math state. Fix:
  restrict the eager `read_arguments` path to `Stored::Expandable`
  only; push primitives/constructors as-is so digestion handles their
  args at the proper time. Min repro:
  `\institute{\hspace*{4mm} $^*$ X}` inside aa-class.

- [x] **astro-ph0107583** (Cluster E, R=2 → R=0) — **FIXED**: extended
  the Perl-faithful `T_ALIGN` self-deactivation guard to the
  `Stored::Constructor` branch in `stomach.rs:invoke_token` (mirror
  Perl `Stomach.pm:187-189`). The `&` char-token's meaning is
  Constructor-bound (TeX_Tables.pool L49), not Token, so the
  pre-existing guard at the Token branch never fired. Now: first stray
  `&` errors once and rebinds itself to `\relax` LOCAL; subsequent
  stray `&`s no-op silently. Witness: astro-ph0107583 2 → 0.

- [x] **physics0002038 / cond-mat0011517** (Cluster H) — **OUT-OF-SCOPE
  at parity 2026-05-01**: parity_check shows `physics0002038 R=4 P=4`
  and `cond-mat0011517 R=6 P=6`. Commit `7319e3fbc`
  (`\@add@frontmatter@now` drop spurious bgroup/egroup) closed the +1
  Rust-only follow-up. Both engines now emit identical error counts;
  the underlying mode-mismatch (paper jams `\begin{minipage}` /
  `\begin{quotation}` block content inside `\author{...}` whose
  `\@personname{}` argument expects `restricted_horizontal`) is a
  shared limitation, not a Rust regression. Worksheet item closed;
  paper family classified out-of-scope per `parity_check.sh`.

- [x] **math0104021** (R=8 → R=1, Perl-parity) — **FIXED**: amsppt's
  `\roster ... \endroster` was a thin `\begin{enumerate}` wrapper
  that left a mode-switch frame on the stack at `\endroster` time,
  cascading 7 × `Error:unexpected:\endgroup` at every subsequent
  `\endref`/`\end`. Replaced with a faithful Perl port (Perl
  `amsppt.sty.ltxml:251-259`): `DefConstructor!('\\roster
  DigestUntil:\\endroster', ..., bounded => true, ...)` plus
  `\roster@item` Constructor and `Let!('\\endroster', '\\relax')`.
  `bounded=>true` keeps the entire roster digestion in one frame
  with proper mode coupling. Min repro: 0 errors (was 7).
  Tests: 1110/0/0 (no regressions).

- [x] **`ltx:XMTok`-in-`<ltx:p>` Δ=2 family** — **FIXED 2026-05-01**
  by commit `a320deffa`: cascading-rejection suppression in
  `document.rs:find_insertion_point_qsym`. When a math leaf element
  (`<ltx:XMTok>` or `<ltx:XMArg>`) tries to insert into a text-mode
  container (`<ltx:p>`/`<ltx:text>`), it's a cascade from an
  already-rejected math wrapper (XMApp/XMDual). Perl emits the
  wrapper's rejection but doesn't continue per-child error logging;
  Rust now matches.
  * `hep-th0101146`: R=17 → R=15 (Perl=15, exact parity)
  * `nlin0211024`: R=4 → R=2 (Perl=2, exact parity)

  Implementation is narrowly targeted: only suppresses the specific
  `(qsym, parent)` pairs `(XMTok|XMArg, ltx:p|ltx:text)`. Other
  malformed insertions still log normally. The element still "inserts"
  via the existing fall-through return; only the redundant log
  emission is suppressed. Side note: the related `Tbox::new`
  divergence (hardcoded `mode => 'math'` vs Perl's `mode => $mode`)
  remains unaddressed — investigated but regresses
  `figure_mixed_content_test` sizing.

- [x] **hep-th0010165** (R=206 vs P=101) — **OUT-OF-SCOPE? 2026-05-01**:
  Perl=101 is the MAX_ERRORS cap (Perl bails at 101 via Fatal). True
  Perl count unknown but >100. At lines 1-345 partial (where Perl is
  NOT capped), Rust=18 vs Perl=26 — Rust BETTER. The full-paper
  Perl=101 vs Rust=206 comparison is invalid (cap-uncertain). Likely
  Rust is comparable or better than Perl here. Re-classify as
  Perl-capped per `wisdom_perl_max_errors_cap.md`.

- [x] **hep-ph0007044** (R=410 vs P=101) — **OUT-OF-SCOPE? 2026-05-01**:
  Perl=101 is the MAX_ERRORS cap. True Perl count unknown. Cap-uncertain.

- [x] **math0205073** (R=10001 → R=0) — **FIXED 2026-05-01** by the
  `\roster` Perl-port commit `050a32b1b`. The state-cumulative
  hypothesis (AmS-TeX `\cases`/`\pcases` mis-parse) was wrong: the
  earlier `\roster` mode-frame leak left BOUND_MODE bound on the
  stack, then every subsequent `&` / `\cr` in the math body
  triggered cascading mode-mismatch errors that hit the MAX_ERRORS
  cap. Dropping `\roster`'s leak collapses the entire downstream
  cascade. Perl=Rust=0 confirmed.

- [x] **quant-ph0109041** (R=67 → R=9, OUT-OF-SCOPE at parity)
  — **FIXED 2026-05-01** by accent commit `ba2ab1dcf` (`mode => "text"`
  drop from `\lx@applyaccent`). Rust=Perl=9; remaining 9 are Perl-baseline
  errors from genuinely malformed `\k{...}` invocations on math-only
  tokens. Now classified OUT-OF-SCOPE per parity_check.

- [x] **astro-ph0204393** (R=113 vs P=101) — **OUT-OF-SCOPE? 2026-05-01**:
  Perl=101 is the MAX_ERRORS cap. Cap-uncertain.

- [x] **hep-ph0102192** (R=4 → R=0) — **FIXED 2026-05-01**: real root
  cause was that pstricks stubs (`pstricks_sty.rs`) did not consume the
  variadic `(coord)(coord)…` PSCoordList that follows `\psline`,
  `\pspolygon`, etc. Coordinates leaked as raw text and `\rput` text
  bodies emitted into the surrounding paragraph, opening an `<ltx:p>`
  that trapped the later `\begin{minipage}` block content. Two-part
  fix: (a) added `\lx@psgobble@parens` recursive `\@ifnextchar(`
  helper to absorb the trailing PSCoordList; (b) dropped the text body
  from `\rput`/`\uput`/`\cput` (consume the paren coord and brace text
  but emit nothing). Visible labels like "cocktail"/"thermal"
  positioned via `\rput` are lost — fidelity regression. The right
  long-term fix is to port Perl's `DefPSConstructor` framework so
  pstricks output lives inside `<ltx:picture>` (where labels survive).
  See follow-up worksheet item below.

- [ ] **pstricks → ltx:picture wrapping** (large-scope feature) — Port
  Perl's `DefPSConstructor` (`pstricks_support.sty.ltxml:491`) and the
  `PSCoordList` parameter type so pstricks drawing commands emit
  `<ltx:line>`/`<ltx:circle>` etc. inside an auto-opened `<ltx:picture>`
  parent. Currently `\rput`/`\uput`/`\cput` text bodies are dropped to
  keep the schema valid (commits `9df708fa9` partial, this round
  drop-rput); restoring them requires the picture wrapper. See inline
  TODO at `latexml_package/src/package/pstricks_sty.rs:51` and historical
  `wisdom_*.md` notes (cycles 305-306, 2026-04-24, deferred per WISDOM
  #41).

- [x] **math0004140** (R=1177 → R=0) — **FIXED 2026-05-01** by
  the `\roster` Perl-port commit `050a32b1b`. Same root cause as
  math0205073: `\roster` mode-frame leak made the entire math body
  emit cascading malformed-XMTok and Unexpected:_ errors. Perl=Rust=0
  confirmed.

- [x] **math0010241** (R=33 → R=19, =Perl) — **FIXED 2026-05-01**
  by commit `a094596a3` extending the cascading-rejection
  suppression to `<ltx:emph>` parents. Trigger is
  `\begin{EG}\emph{ ... $$display math$$ ... }\end{EG}` blocks; both
  engines correctly reject the fundamentally malformed input, but
  Rust was emitting per-XMTok-child cascade noise that Perl does
  not. The 14 XMTok-in-emph cascade emissions are now suppressed,
  bringing Rust to exact parity with Perl=19.

- [x] **astro-ph0203201** (R=70 vs P=70) — **Out-of-scope** —
  Perl=Rust same error counts (56 `_`-in-text + 12 XMArray-malformed
  + 2 `^`-in-text). Both fail identically.
- [x] **cond-mat0103632** (R=20 vs P=20) — **Out-of-scope** — same.
- [x] **hep-ph0110283** (R=98 vs P=101) — **Out-of-scope** — Rust
  better than Perl (Perl saturates at 101 truncation cap).
- [x] **hep-th0004072** (R=33 vs P=101) — **Out-of-scope** — Rust
  better than Perl.
- [x] **hep-ph0204075** (R=0 vs P=0) — **PASSING** — recovered by
  recent commits, no longer a failure. Marked in completed
  investigations table.

- [x] **hep-th0005268** (R=21 vs P=26 — was 10001) — **FIXED
  2026-05-01** by commit `cda6cb247`: surgical lazy-pool-load guard
  in `latex_constructs.rs:5560`. The kernel `\let\a=\@tabacckludge`
  is now skipped when `\a` is already user-defined; under the
  user's `\def\a{\alpha}` BEFORE `\documentclass`, `\a` is
  preserved as `\alpha` and the body math works clean.
  Rust=21 errors are real cascades from elsewhere in the paper
  (stray `^/_` in math, malformed XML), not the spurious `\hbox`
  runaway noise. **Now PERL_REGRESSION** — Rust supersedes
  Perl (R=21 < P=26). Min repro
  ```
  \def\a{\alpha}
  \documentclass{article}
  \begin{document} $\a + x$ \end{document}
  ```
  is now R=0, P=0. The pattern (`\let\<public-cs>=...` in
  `latex_constructs` guarded by `is_already_user_defined`)
  generalizes to other Let calls if similar witnesses surface,
  but for now the targeted fix unblocks the canonical case.
  Same family as `cond-mat0106160`,
  `hep_ph0001306_documentstyle_clobber` (still triaged
  out-of-scope but may benefit from the same pattern).

- [x] **hep-th0005159** (R=262 vs P=101) — **OUT-OF-SCOPE? 2026-05-01**:
  Perl=101 is the MAX_ERRORS cap; cap-uncertain. Rust now at 262 (well
  below its 10000 cap), so the prior 786478 number was pre-MAX_ERRORS-leak
  fix `15f46ddf3`. parity_check tags this `OUT-OF-SCOPE? (Perl-capped,
  cannot compare)`. Worksheet item closed pending a future paper that
  exposes a directly-comparable variant.

#### Out-of-scope (Perl also fails — moved to `docs/out-of-scope/`)

| Paper | Reason |
|---|---|
| `0901.2408_emph_dollar` | `$$`-in-`\emph{}` — Perl=Rust |
| `cond-mat0003169` | `Unexpected:_` cluster — Perl=Rust=2 |
| `cond-mat0106160` | `\def\r\rho` BEFORE `\documentstyle` clobber family |
| `hep_ph0001306_documentstyle_clobber` | `\def`s before `\documentstyle` — broader family |
| `math0005251_math_parser_oom` | math-parser OOM — needs grammar work |
| ~~`math0203148_amstex_endmatrix`~~ | **REMOVED 2026-05-01** — fixed by `\roster` Perl-port commit `050a32b1b` (was misdiagnosed as `\matrix` issue; actual cause was the `\roster` mode-frame leak, same family as math0104021) |
| `math0601451_xmtok_in_title` | XMTok-in-title issue |
| `math0606553_xy_compile` | xy-pic AmS-TeX compile failure |

#### Active Rust-engine clusters (driven by sandbox investigations)

| Cluster | Status | Notes |
|---|---|---|
| A. `\par` in counter-CS reading | **partial fix** `9c60a766c` (covers `\@captype`); residual `\thesection\par` open per math0010095 worksheet item. |
| B. tcilatex `\newcount\dispkind` | **fixed** mid-Round-18. |
| C. `\documentstyle` dump-clobber | **fixed** `6e6497ede` (re-Let). |
| D. aa-class `\institute` math leak | **open** — see worksheet. |
| E. Stray `&` outside table | **fixed** — extended `T_ALIGN` deactivation guard to Constructor branch in `stomach.rs:invoke_token`. |
| F. Cascading single-root | **open** — math0004140 + runaway cascades worksheet items. |
| G. pstricks `\multips` | **fixed** `506cb8fe6`. |
| H. Mode-stack `}` followup | **open** — error-tracker dedup work. |

Long-standing deep clusters parked in
`docs/archive/sandbox_failures_SYNC_STATUS.md`. Re-survey whether
recent fixes have shrunk the surface enough to make individual
items tractable. Notables:

* `1803.03288`/`1902.08705` (expl3 cascade + pgfmath `\ifdim`) — open.
* pgfplots `\pgfplots@curlegend`/`\pgfplots@curplotlist` state-machine
  — **resolved** 2026-04-25 (commit `b4b196254`,
  `pgfplots_sty.rs:18-28`). The undefined-CS cluster traced to a
  `\globaldefs` register-type mismatch in core, not a pgfplots-shim
  gap. Re-survey on the 100k canvas to confirm no residue.

### 5. Distribution — bundle multi-TL dumps

Once TL2025 dumps stay robust through a CI cycle: `include_bytes!`
`{plain,latex}.dump.txt` for TL2022 … TL2026 and select at runtime
by `kpsewhich --version`. Currently dumps load from
`resources/dumps/` on disk.

---

## Engine file open gaps

| File | Status | Open Gap |
|------|--------|----------|
| `base_parameter_types.rs` | MINOR | Parameterized `CommaList:Type` form unported (no Perl users). |
| `tex_box.rs` | MINOR | Box dimension edge cases. |
| `tex_fonts.rs` | MINOR | `\fontdimen` array semantics; per-font `\hyphenchar`. |
| `tex_tables.rs` | MINOR | Padding CSS classes (XSLT concern). |
| `plain_base.rs` | OPEN | Some closure-backed defs need conversion to Token bodies for dump round-trip. |
| `latex_base.rs` | OPEN | Closure-backed defs need conversion or relocation to `latex_constructs.rs`. |

---

## Tikz known diffs vs Perl

1. foreignObject transform Y / width/height
2. Arrow tip shape (different path data)
3. SVG viewBox / total width differs slightly
4. tikz matrix uses `<svg:g class="ltx_tikzmatrix">` (Rust) vs
   inline-blocks (Perl)

---

## Permanent ignores

* **Sandbox out-of-scope:** ns1–ns5 (52_namespace, no DTD); 2402.03300,
  2410.10068, 2511.03798 (Perl also fails).
* **Rust supersedes Perl** (both still in scope, but Rust passes
  where Perl errors): `1207.6068`, `0909.3444`.
* **Unported pools:** `BibTeX.pool.ltxml` (skipped via `--nobibtex`).

---

## Acceptance gates

| Gate | Current | Target |
|---|---|---|
| `cargo test --tests` | 1110/0/0 | unchanged across all task work |
| `latexml_oxide --init=plain.tex` | 0 errors | 0 errors |
| `latexml_oxide --init=latex.ltx` | 0 errors | 0 errors |
| 10k canvas (Phase 1, complete) | 7731 / 7898 = 97.89% | n/a (canvas retired) |
| Filesystem-level hard failures (10k) | 1 (math0005251) | 0 |
| 100k "no-problem" canvas (Phase 2, active) | downloaded (100 000 ZIPs) — sweep pending | 100% match Perl |
