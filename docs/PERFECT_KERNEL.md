Task: iteratively test, audit and develop the latexml-oxide kernel support until we reach perfect conversion over all documentation bundles available in texlive.

As an example, let us take nicematrix.sty . Its documentation lives at /usr/local/texlive/2025/texmf-dist/doc/latex/nicematrix/ . It contains
 - the package manual, which will act as a test source file: nicematrix.tex
 - golden result reference:  nicematrix.pdf
 

With `kpsewhich nicematrix.sty` we see that the package is installed via texlive and will be discoverable via latexml-oxide . 

What we want to do is iteratively test and develop a conversion to core XML of the package manual . We want to ensure a high quality conversion that has healthy markup compliant with the LaTeXML schema, as well as preserving all content (which is testable against the PDF golden rendering). Some of these tests and choices are easy, such as plain text and math syntax. Some are difficult or open-ended, such as unsupported graphics packages, special placements, or side-notes.

We will catalog and plan the difficult cases, and we will attempt to support as many of the available .sty and .cls files as possible, using *raw interpretation* and not new binding files (no new `*_sty.rs` or `*_cls.rs` files). Let us gradually build a test suite, and leverage the `--preload=[rawstyles,rawclasses]latexml.sty` technique to ensure we load raw both style files (.sty) and class files (.cls). Ensure that we are not using OmniBus, as our main goal is raw interpretation using the TeX engine (we may also need to improve our method or coverage of generating pre-compiled kernel dumps).

Develop a systematic and details documentation system that tracks progress and plans the work, under `docs/perfect_kernel/`. Work on a new `perfect_kernel` branch and do not push to github until the entire work is complete.

## What "perfect conversion" means — the quality axes (user, 2026-09-19)

The target is **XML quality**, not a zero-error count. Error/Fatal counts are a
diagnostic aid for finding problems, never the goal: a conversion can be 0-error
yet drop content or emit presentational markup, and it can carry spurious
diagnostics over byte-perfect output (e.g. batch 56ee's `xpath` recovery changed
no XML). Quality has three axes, and they depend on different oracles:

1. **Content preservation** — every word/structure of the source survives into
   the XML. Oracle = the source (and the golden PDF where one exists). Measured
   by S3 word-recall (`tools/perfect_kernel/s3_sweep.sh` → `s3_verdicts.tsv`,
   recall vs the golden PDF's distinct words). This is the primary axis.
2. **Semantic markup** — every LaTeX macro/primitive construct that HAS a
   LaTeXML-schema element is emitted as that element, not a generic/presentational
   fallback (theorem-like envs, floats, cross-refs, citations, list/table
   structure, math sub-structures, MathML Core). Oracle = the schema + the
   construct's meaning; no PDF needed. This axis is under-instrumented — a
   coverage measure is worth building.
3. **Fidelity to the PDF** — visual/structural match to the rendered golden.
   Oracle = a rendered PDF from the source's **intended** engine.

**Scope / oracle-gate.** A source is judged on axes 1 and 2 whenever it is
parseable LaTeX, and on axis 3 only where a golden render exists. A source whose
**intended** engine (pdflatex, xelatex, or lualatex — detected from the class /
engine-specific packages / a `%!TEX` magic comment, not assumed pdflatex) produces
no usable/complete PDF is **exempt from the fidelity axis** — there is no reference
to be "as good as" — but still scored on content and semantics. "No usable PDF"
means no complete output, NOT merely that the engine printed errors (TeX is
error-tolerant and usually emits a PDF anyway). S3 already implements this gate:
docs with an absent/empty golden are recorded recall `-` and excluded from
content-loss, and `tools/perfect_kernel/oracle.sh` compiles each doc with its
engine and counts `!` lines (0 = clean oracle). Already out of scope: shell-escape
tool manuals (`tools/perfect_kernel/shell_escape_excluded.tsv`).

**Working priority.** Drive from the S3-recall tail and from schema validity, NOT from error-cluster mining — the
arXiv error histograms proved a weak, partly-stale proxy (batches 56ed/56ee genuine fixes were spurious-diagnostic
suppressions; `\NewTaggingSocket` etc. were stale-log false-positives that reproduce 0 errors on the current binary).
Since the user's re-steer (2026-10-01) batches come from the sweep scoreboard: S3 missing words, schema-invalid
in-scope manuals, timeouts (the ranked path below).

## Scope — the manuals some engine compiles cleanly (user, 2026-09-29)

The program works on the **1,602 of 2,374** manuals that compile cleanly with at least one of pdflatex,
lualatex or xelatex: `tools/perfect_kernel/oracle.sh` (the golden's producer engine first, then the others)
exits 0 with no `!` line (`~/data/perfect_kernel/oracle_verdicts.tsv`; 1,292 pdflatex, 286 lualatex, 24
xelatex; rerun it whenever TeX Live changes). The other **772** (769 that no engine completes, 3 that compile
with errors) are out of scope for quality work: they stay in every sweep as **crash canaries** only — a new
Fatal, timeout or abnormal exit there is a regression to fix; their errors, recall and schema are not worked.
The arXiv A/B and the cortex runs remain the real-world signal for the packages only they load.
**Japanese, Chinese and Korean manuals are out of scope (user, 2026-10-01)** — recorded for a future project,
not worked for this goal: bxcjkjatype-ja, bxcjkvert-ja, kanbun-example, bxcoloremoji-ja, bxjscls-manual, qworld,
zhlineskip and zxjafont (sweep #132's CJK cluster, mostly LuaTeX-ja/XeTeX/xeCJK machinery through the texlua
bridge); the Korean kotex manuals (kotex-doc, kotex-utf-doc, oblivoir-simpledoc, obchapterstyles-doc,
obsideparas); and any other in-scope manual whose content is Japanese, Chinese or Korean typesetting. They stay crash
canaries like the 772. A defect they expose in shared machinery stays in scope as an ordinary bug (the geometry
binding's missing `\ifGm@showframe`/`\Gm@initall`, RED `s41undef_ifGm_showframe`).
**Also out of scope (user, 2026-10-02)**, recorded for long-term later work: **milsymb** — its tables are
PythonTeX output (`\begin{pycode}` importing the manual's scripts) that needs an external pythontex run, the
shell-escape class (listed in `shell_escape_excluded.tsv`; 388 S3 missing words); **elzcards-examples** — our
geometry binding keeps `\textwidth` at the class default (OXIDIZED_DESIGN_DIVERGENCES #99), elzcards sizes its card
grid from it, raises "No space to print at least one card" and `\stop`s, losing the manual's tail (SHARED with
Perl; later: a page-dimension model that does not widen the flow, and a faithful `\stop` — Perl `closeMouth(1)`,
latex.ltx `\@@end`); and **a bibliography inside a tcolorbox** — `<ltx:bibliography>` in the `ltx:block` of an
`svg:foreignObject` is schema-invalid (biblatex-ext's last error; SHARED; later: a schema widening or demotion
rule). Like the CJK manuals they stay crash canaries; a shared-machinery defect they expose stays in scope.
`tools/perfect_kernel/scoreboard.py` reports the in-scope quality table first and the canary table after it
(`--scope all` for the whole corpus). The manual regression net (K17) is re-selected from the in-scope set.

## Goal bar and ranked path (2026-10-02)

This is the single ranked order to the goal. The handoff register (`~/data/pk_agents/main/HANDOFF.md`, evidence and
designs on file), PLANS.md and KERNEL_CAPABILITIES.md feed it and do not rank on their own. The superseded phase-57/58
plan, the 2026-09-25 stream table and the corpus-wide scoreboard history are in
[`perfect_kernel/archive/PERFECT_KERNEL_PHASE57_58_PLAN_2026-10-02.md`](perfect_kernel/archive/PERFECT_KERNEL_PHASE57_58_PLAN_2026-10-02.md).

**Goal bar** (proposed 2026-10-02 from the quality axes, the scope and the rulings; to be confirmed by the user). One
sweep on the closing head shows, on the in-scope set (the 1,602 oracle-clean manuals less the ruled-out ones above):
- **G1 clean:** every manual at status 0-1; a remaining error only of a ruled-out kind (biblatex-ext's bibliography
  inside a tcolorbox).
- **G2 valid:** every manual jing-valid, or each jing line in a ruled class — a dangling `\hyperlink` target (D15), a
  sectioning unit in a list item, a bilingual manual's cross-language link.
- **G3 recall:** every manual at ≥ 95 % S3 recall, or its gap classified as no conversion loss: reference-side (text
  inside included PDFs or images, PDF-extraction garble, CJK segmentation, a golden typeset from another source —
  `tools/perfect_kernel/golden_reference.tsv`), ruled page furniture, date words, external-tool output.
- **G4 time:** no manual over 180 s at 8 GB in the sweep.
- **G5 canaries:** no new Fatal or timeout among every other manual — the 772 no engine compiles plus the ruled-out
  ones (68 at s134: 64 Fatal and 3 timeouts of the 772, zxjafont's Fatal).
- **G6 no regression:** every batch's arXiv A/B neutral or explained; semantic coverage (`semantic_coverage.py`) not
  lower; at the close, the cortex reruns of sandboxes 2605/2606 (L6) show no new Fatal cluster.

**State against the bar** — sweep #135 (59l2, 2026-10-02); `scoreboard.py` prints the open counts per sweep and
`scoreboard.py --open N` the manuals (goal set: 1,558 = 1,602 in scope less the 44 in `out_of_scope.tsv` and
`shell_escape_excluded.tsv`; ruled counts in `accepted_residuals.tsv`):

| sweep | G1 unclean | G2 invalid | G3 below 95 % | G3 missing words | G4 over 180 s | G5 canary Fatal/timeout |
|---|---|---|---|---|---|---|
| 133 (58q5) | 4 | 3 | 143 | 4,020 | 3 | 68 |
| 134 (59c1) | 5 | 3 | 143 | 4,020 | 1 | 68 |
| 135 (59l2) | 1 | 2 | 142 | 3,992 | 1 | 68 |
| 136 (60b4) | 4 | 1 | 128 | 4,285 | 0 | 68 |
| 137 (60f1) | 1 | 1 | 125 | 3,629 | 2 | 68 |
| 138 (60j4) | 1 | 1 | 121 | 3,553 | 0 | 68 |

Open at s135: **G1** tabularray (1 error; fixed by 59n, rank 3 — G1 is clear); ruled: chessboard_and_beamer's Fatal (D14, 2026-09-10: keep Perl's
single-pass overlays) and biblatex-ext's last error. **G2** philexmanual (fixed by 59m: 7 → 6, the 6 dangling D15) and
biblatex2bibitem-hyperref-result (5 `page.1` links, rank 7c); ruled: 3 biblatex cite/backref manuals (854 lines, D15),
7 section-in-item/figure (prerex's `\paragraph` inside its figure, OXIDIZED_DESIGN_DIVERGENCES #189), 5 singleton
dangling links (elsdoc, europecv, crossreftools, iodhbwm, jsonparse), biblatex-ext, chessboard_and_beamer. **G3** 142
manuals, 3,992 words, by sweep #132's recall-tail classes: bibliography rendering in post 37 manuals / 1,762 words
(ruling 7b), the residual "source words dropped" 72 / 1,377 (uantwerpendocs ×5 and g-brief ×2 among them, rank 5),
generated text 31 / 589 (unverified), arabi/samplebook 196 (rank 6), wheelchart 68 (newly scored). Accepted as no
conversion loss (`accepted_residuals.tsv` kind `recall`): montex, mlsquick, zanabazr (816 words — the golden's Type 3
bitmap fonts have no Unicode text layer; root-caused 2026-10-02) and 61 manuals' reference-side, date, slide-furniture
or external-tool gaps (2,396 words, from the s132 classification — agent-made, not yet verified per manual). **G4** pgf-interference-en 180.1 s (166 s
alone; rank 4). s134 → s135 had no regression: every recall mover improved (cmpj, serbian-def-cyr/proba, hindawi) but
datetime2-en-fulltext (one date word, the new `SOURCE_DATE_EPOCH` pin), no manual became invalid, every status change
was an improvement; the in-scope missing-word total rose 14,437 → 14,667 only because four manuals are newly scored
(kaytannollista-latexia, qworld, robustsample, wheelchart: +299 words).

**Sweep #136 (60b4, 2026-10-03)** — G4 met (pgf-interference-en 99 s, no timeout); G3 below 95 % 142 → 128 (the
uantwerpen, g-brief, showexpl, pgf-spectra, pdfcomment and arabi batches). Its G1/G2/recall regressions were bisected
over the batch binaries and fixed in 60d (TikZ nested pictures over-closing `svg:g`, KPE #440; a class `\maketitle`
table dropped by 59p's content test; an autoref name that is a drawing command) or recorded: withargs' `\fileversion`
is undefined as in pdflatex; xskak's and biblatex-gost's errors are RED repros (the latter needs a ruling, item 7e);
montex's recall drop is its golden's Latin text layer against 60b's Cyrillic.

**Sweep #137 (60f1, 2026-10-03)** — G1 4 → 1 (60d's fixes: codeanatomy.usage 32 → 0 errors, causets_example2 6 → 0,
argumentation-doc 1 → 0; mercatormap valid); G3 below 95 % 126 → 125 (testcv 88.6 → 100); G4 0 → 2: pgf-interference-de
and -en at the 180 s edge again (170-180 s on every sweep, -en timed out at s135; load-sensitive, the TikZ performance
item). One regression: etextools-examples 4 → 6 errors, from 60e2's per-call `\vadjust` group around a pre-tokenized short
verbatim (RED boxes-groups/vadjust_material_is_read_live; 60g reads the material live). Counts from today's lists (the
136 row was computed before montex/mlsquick were accepted: now 126 / 3,637).

**Sweep #138 (60j4, 2026-10-03)** — G3 below 95 % 125 → 121, 3,629 → 3,553 missing words (glosmathtools en/fr
89.0/90.9 → 100, figbib_sample 54.2 → 95.8, clefval example-utf8 78.8 → 100: 60h/60i); G4 2 → 0 (pgf-interference-de
and -en back under 180 s); etextools-examples 6 → 1 error (60g reads `\vadjust` material live). No regression: every
status, validity and recall change was an improvement.

**Ranked path.** Batches come from the scoreboard (user re-steer 2026-10-01): S3 missing words, schema-invalid
in-scope manuals, timeouts. Each batch gets one reviewer round and one fix round; a synthetic finding becomes a RED
repro; one arXiv A/B at the ship candidate. Before taking a document, check it against every out-of-scope list (user
2026-10-02: out-of-scope marking exists to reach the goal sooner).
1. **Measure the bar** — DONE (f3c15b7daa, sweep #135). The lists are data beside the scoreboard: `tools/perfect_kernel/out_of_scope.tsv` (each
   ruled-out manual with its ruling and basis; the CJK clause applied to sources with ≥ 200 CJK characters or a
   zh/cn/jp/ja/tc name), `accepted_residuals.tsv` (ruled error and jing counts), and `run_doc.sh` pins
   `SOURCE_DATE_EPOCH` as `manual_net.sh:30` does (s132's recall drift was `\today`'s month). Next, **sweep #135** on
   the 59l head — the first sweep read against the bar — and re-cluster the goal set's recall tail
   (`sweep132_analysis/recall_tail_clusters.tsv`'s classes; a classified non-loss gets a list of its own).
2. **G2: philexmanual's anchor** — DONE (59m, fbca9a19da; KPE #424). `\hypertarget{id}{<display>}` puts the anchor before the display (hyperref's
   nesting-false order, hyperref.sty:4805-4810) and `localized_anchor` wraps only where the parent can hold an anchor
   (SHARED; root cause on file; LOW-MED; the gemini guard at `perfect_kernel_gemini.rs:2204` pins today's invalid
   shape).
3. **G1: tabularray's outer keys** — DONE (59n, ef92b4214a). `evaluate=`/`expand=` and `\SetTblrOuter` are ignored: collect the body and run
   tabularray's own preprocessing when they are present (RUST-ONLY, MED; root cause on file).
4. **G4: TikZ speed.** The audit's levers L1 (`macro_call` pstack path, −12..17 %) and L6 (`\expandafter`), one per
   measurement, until pgf-interference-en has margin under 180 s inside the sweep. 59q: L1's substitution half
   and the depth guard's cold path, fp1000 −7.8 %, en 164.5 → 156.6 s alone; open: L1's argument buffer, L6.
5. **G3: title-page and letter content** — DONE (59o g-brief, 59p uantwerpen). uantwerpendocs ×5 (59p) — eso-pic's `\AddToShipoutPicture*` title-page overlay
   (PERL-ORIGIN) and the frontmatter vocabulary gate's false negatives (RUST-ONLY); recall 83 → 96, 92 → 100,
   86 → 98.6; ruled 2026-10-02: the one-shot title-page overlay is kept whole, logo and form boxes included. g-brief ×2 — DONE (59o): the letter's sender (user 2026-10-01: kept) and addressee
   as frontmatter, with XSLT for the roles; beispiel2 60.7 → 97.6. Both designs on file.
6. **G3: losses not yet root-caused.** arabi/samplebook — DONE 59r (LAE/LFE fontmaps; the remaining S3 gap is the
   golden's text layer (presentation forms and slot codes, visual order), to be recorded in `accepted_residuals.tsv` at the next sweep; hvarabic is fontspec,
   not this); the residual class "source words dropped" (72 manuals, 1,377 words at
   s135), from the top — 59s took its kernel-level rows (braced `\openin` names, `\fileversion`, an unknown font
   family under pgf's `\nullfont`, textpos absolute blocks; pgf-pie, sepfootnotes, pdfcomment ×3, stubs, eso-pic);
   59t took nomentbl's 5-argument entries, glossaries `title=`, `\autoref` names (and two arXiv listings issues), 59u
   the showexpl preset (showexpl-test 50 → 95 %), 59v `\vsplit` forced breaks, 60e `\vadjust` material as its own
   paragraphs (KPE #441; open: material built where it is read and kept with its line, RED
   boxes-groups/vadjust_material_is_built_where_it_is_read, vadjust_material_stays_with_its_line); next (slice 3,
   agent_reports 2026-10-03): figbib `@fig` fields, clefval `.aux` values, glossary user keys and parent headings;
   nomencl group headings; the generated-text class (31 manuals, 589 words) to verify. Fidelity beside recall: the montex
   manuals' Cyrillic passages — DONE 60b (an LMC fontmap from the kmr fonts' encoding and ligature program, KPE #438);
   their Mongolian-script passages (LMS/LMO/LMU/LMA, no map) still read as transliteration (RED
   `fonts-nfss/lms_encoding_bicig`).
7. **Rulings needed** (each opens or closes a block of the bar):
   a. the bar itself;
   b. bibliography rendering in post — 40 manuals, 2,202 missing words at s132, the largest real-content class: the
      post-stage formatter does not print a biblatex/bibtex style's own words (K16, bibliographies from the style's
      programs; the harness has no biber `.bbl`) — in the goal, or a later project like math;
   c. biblatex2bibitem's `page.1` links — hyperref's page anchors, which the HTML has no pages for: D15 or not;
   d. token lists by reference (the structural TikZ lever) only if L1/L6 fall short. (The uantwerpen logo: ruled
      2026-10-02, the one-shot title-page overlay is kept whole.)
   e. an autoref name's diagnostics: 59t binds the name at the target (ruling 2026-10-02), so a name that cannot be
      typeset raises its errors at every target where TeX raises them only at an `\autoref` (biblatex-gost: hyperref's
      Russian `\cyr…` name under TU, 4 errors without any `\autoref`; RED singletons/autoref_name_evaluated_at_every_target).
      The same root sets a name that takes a required argument as the `~` alone (60d, DIVERGENCES #418): pdflatex
      prints a text-bearing one's text at an `\autoref` ("Hh1"), we " 1".
8. **Close.** The sweep that meets G1-G5; a review of every CSS change the goal made to `LaTeXML.css` (quality, and a
   port of what ar5iv needs to its standalone stylesheet, `~/git/ar5iv-css/css/` — user 2026-10-02; e.g. 59o's letter
   roles); the cortex reruns of 2605/2606 (L6); then stream G: the full arXiv rerun on the fleet.

**Not on the path** (recorded; taken up when a scoreboard manual needs one, or after the goal): math-parse fidelity
(PARKED 2026-10-02; the 57cj.22-23.8 train landed on perfect_kernel, merge 5d7a7d91f2); the out-of-scope manuals; the generalization trains (K13 stage 3, K14,
K15) and the whole-branch special-case audit; the RED repro backlog (`SYNC_STATUS.md`,
`~/data/pk_agents/main/red_inventory/`); semantic-coverage gaps; the class census; the arXiv-only residuals below;
SYNC rows R3d, R4, R6, R7, R8b.

## Method

Branch `perfect_kernel`, worktree `~/git/latexml-oxide-pk`. Subagents: narrow read-only types (`root-causer`,
`reviewer`, `log-scanner`, `perf-measure`, `math-diagnose`) on Opus 5.5 high, at most 2 at once; the main session is
the single writer. Memory cap 8 GB (`--max-memory=8192`, `ulimit -v 8912896`, per conversion only, never on the test
runner). Sweeps: `~/data/pk_agents/w70/sweep134_launch.sh <label>` is the current recipe (vendor TL +
`LATEXML_DUMP_DIR` vendor dumps, JOBS=16, 180 s; sweep → validate → post mono → HTML recall in one chain); no nextest
beside a sweep or an A/B. arXiv A/B: `tools/perfect_kernel/arxiv_ab.sh <run> <binA> <binB>` (3,003 papers), byte-diff
the outputs and read the `tex= changed` line. Never run an engine with its cwd in the vendor TL doc tree. Every fix is
a minimal `.tex` repro in `tools/perfect_kernel/repros/<mechanism>/` and a red/green guard with whole-element
assertions (user, 2026-09-23); a schema win counts only if content is preserved (`content_diff.py` on every witness,
`pdf_recall.py` with the intended engine's PDF).

**Gate ladder (every batch):**
1. L0 red: the repro shows the defect on today's binary.
2. L1 green: the guard passes; full nextest, clippy and rustdoc on the exact committed tree
   (`~/data/pk_agents/main/gates_pk.sh`).
3. L2 manuals: the mechanism's witness manuals before/after (`content_diff.py`, recall); the manual net
   (`manual_net.sh` + `manual_net_compare.py --recall`, K17) for cross-cutting changes. No manual loses recall
   unexplained.
4. L3 arXiv A/B once, at the first ship candidate: no status or word loss that pdflatex does not explain.
5. L4 one reviewer round and one fix round, then one commit per batch; push at checkpoints.
6. L5 a sweep every few batches, with a scoreboard row: no manual down by more than 0.5 recall or newly invalid,
   unless classified faithful to pdflatex.
7. L6 at the close, the cortex reruns of 2605/2606; L7 stream G.

**Performance rules** (`docs/performance/PERFORMANCE.md`):
- Levers are algorithmic or strategic only, with no caches or carried state in gullet/stomach/mouth/document.
- Stay behind the Perl-shaped interfaces.
- Profile first, with `--profile bench` symbols.
- One lever per measurement, with error bars; instructions are steadier than wall time under load.
- pdflatex's time on the same source is the throughput oracle.
- A document over 60 s needs a stated reason (`slow_calls.sh`).

**Speed gate:** every batch's L3 arXiv A/B prints `secs A→B` and the papers slower by more than 50 % and 5 s (`arxiv_ab_compare.py`). Each slow paper needs a reason. L5 must not raise cpu_h by more than 3 %, or the >60 s / >120 s counts, without an attributed cause. Manual-subset timings run beside another A/B are not comparable: re-time a flagged document alone.

## Scoreboard

In scope (`tools/perfect_kernel/scoreboard.py 130`; clean = status 0-1; cpu_h = the sum of
per-document conversion seconds; recall from the inline-glue audit walker since s125; the corpus-wide rows s113-s130
are in the archive):

| sweep | head | docs | clean | fatal | timeout | errors | valid | scored | recall mean | median | %≥95 | missing | cpu_h | p90 s | p99 s | >60 s | >120 s |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 130 | 57ak5 | 1602 | 1585 | 2 | 3 | 388 | 1569 | 1564 | 96.18 | 99.4 | 84.8 | 14929 | 0.92 | 2.5 | 26.7 | 11 | 4 |
| 131 | 57ck3 | 1602 | 1585 | 2 | 3 | 388 | 1569 | 1564 | 96.18 | 99.4 | 84.8 | 14928 | 0.99 | 2.8 | 27.1 | 11 | 4 |
| 132 | m58i | 1602 | 1585 | 2 | 3 | 350 | 1574 | 1564 | 96.16 | 99.4 | 84.8 | 14714 | 0.93 | 2.7 | 26.5 | 11 | 4 |
| 133 | 58q5 | 1602 | 1585 | 2 | 3 | 350 | 1576 | 1564 | 96.23 | 99.4 | 85.0 | 14435 | 0.94 | 2.7 | 26.9 | 11 | 4 |
| 134 | 59c1 | 1602 | 1586 | 2 | 1 | 351 | 1578 | 1565 | 96.23 | 99.4 | 85.0 | 14437 | 0.87 | 2.4 | 24.1 | 10 | 2 |
| 135 | 59l2 | 1602 | 1590 | 2 | 1 | 160 | 1578 | 1569 | 96.24 | 99.4 | 85.0 | 14667 | 0.86 | 2.5 | 24.1 | 10 | 2 |
| 136 | 60b4 | 1602 | 1588 | 2 | 0 | 202 | 1579 | 1567 | 96.58 | 99.4 | 86.0 | 14332 | 0.85 | 2.4 | 23.4 | 10 | 2 |
| 137 | 60f1 | 1602 | 1589 | 2 | 2 | 163 | 1577 | 1568 | 96.59 | 99.4 | 86.1 | 14320 | 0.92 | 2.7 | 24.7 | 10 | 2 |
| 138 | 60j4 | 1602 | 1591 | 2 | 0 | 163 | 1579 | 1570 | 96.64 | 99.4 | 86.4 | 14249 | 0.85 | 2.5 | 23.5 | 10 | 2 |

Crash canaries (772): 64 Fatal and 3 timeouts on every sweep since s130; cpu_h 1.18-1.32.

## Open residuals and rulings

**Off the path** (the phase-56 leads and the healthy-subset projection are in
[`perfect_kernel/archive/PERFECT_KERNEL_PHASE56_NOTES_2026-09-27.md`](perfect_kernel/archive/PERFECT_KERNEL_PHASE56_NOTES_2026-09-27.md)):
- **Kernel, arXiv 2605:** polytable needs array.sty's `\@mkpream`/`\@classz` builder (2605.08990); 621 raw-mhchem
  `\ce` exceed the 16M conditional cap (2605.27177, volume, not a loop); three pgf/tikz loops (2605.00058, .04377,
  .12601); the cycle guard's false positive on a large repetitive table (2605.11798); the autoload hoist as a load at
  group level 0 (DIVERGENCES #282).
- **Engine:** a group-local `\def` of a locked control sequence, and Rust's empty root where Perl keeps a partial
  document (both 2605.31475, LEDGER 56hk: pgffor's `\foreach \x/\tag` refused by the lock on amsmath's `\tag`; Perl
  ends in a bounded `Fatal:misdefined`, Rust in an unbounded pushback); catoptions' residual option-stack-limit error.
- **Semantic coverage (axis 2b, `semantic_coverage.py`; s114: sections/lists/floats/refs 93-98 %, equations 93 %,
  graphics 88 %, `\part` 59.5 %):** the graphics family (36 documents short); ctex's localized part label reads
  "Part I"; beamer overlays are not acted on (Perl's `ltx_covered` wrapper, #270), and frames are not Perl's
  `ltx:slide` in `ltx:slidesequence` model; a `\caption` in a non-float box becomes its float, but the box's other
  content stays beside it.
- **Class census** (`~/data/pk_agents/w67/clscensus7/census7.tsv` on `vendor_dumps56id`: 496 / 500 usable classes
  clean): novel (12 errors: its images' `_` in text, SHARED; two layout `\the` errors; fontspec's main-font check),
  imsproc (1, NFSS `.fd` loading), udesoftec (1, `\abstract` constructor), upmethodology-document (2, SHARED); not
  usable: tikzposter (parked). Re-run with `clscensus7/run.sh <class> <binary>`, whose dump directory must match.
- **Performance beyond the ceiling:** lie-hasse and wheelchart exceed the budget in their own engines too (277 s,
  > 900 s); the TikZ manuals run at 2.5-3× lualatex (pgf-interference 166 s vs 68 s); the remaining lever class is
  native l3 int/tl primitives (PLANS 12) and the token-list representation.
- **SHARED with Perl, not pursued:** the arXiv 2605 Fatal remainder (els-mrw bibitems, class-body macros under
  OmniBus, enumitem `label=\theenumi` self-reference, `\theequation` in `\tag*`, the mode-frame family, missing
  packages); tufte's citation `\marginpar`s left empty (LEDGER 56ip); the silent-loss structural scan (empty `<p/>`,
  icon inline-blocks, struts) is settled.

**Method notes:** read a witness's `(Loading …)` lines before assuming the raw-class path; probes MUST pin the vendor
TL (an unpinned run reads the distro tree — qworld reproduced only pinned); stage by line when batches share a file
(`git hash-object` of a HEAD+hunk file into the index).

**Ruled / documented, do not re-open:** engine-primitive ink (21 docs) out of scope + harness retry only;
section-in-item/figure Perl parity; thuaslogos duplicate `pgfcp*` ids (`\copy` of a digested SVG box, SHARED,
HIGH-risk id rewrite); ndsu `text` in `listing` (document bug); translation-biblatex-de `\begin[…]{description}`
(document bug); M4/M5 bespoke title-page layouts (ltnews, l3news, lua-tikz3dtools, nostarch, elteiktdk, sduthesis,
aomart); dangling `\hyperlink` idrefs stay (D15, 2026-09-18) — among them biblatex-gost-examples' 770 `back:<key>`
and biblatex-chicago's 84 `cite.0@<key>` links, whose targets biblatex makes only from a biber `.bbl`
(`biblatex.sty:10239-10245`; the harness runs no biber), philexmanual's 6 `\lbz`/`\lba` targets philex never emits
(pdfTeX warns too), elsdoc's targets inside `{comment}`, europecv's `\hyperlink` to `\label` names; ~92 no-XML fatals
(mostly intended engine ≠ pdflatex); page furniture dropped, semantic notes kept (2026-10-01); a second `\maketitle`
keeps the latest title (2026-10-01); bilingual documents (one manual per language, each half with its own
`\maketitle` and a cross-language `\hypertarget`/`\hyperlink` pair: circledtext, joinbox, pascaltriangle, suanpan-l3)
out of scope for now, potentially interesting later — the document keeps the last title, and the first half's anchor
link dangles (2026-10-01).

**Diagnostics rule in force (2026-09-20/22):** every Warning/Error/Fatal logged AND counted once; a resource Fatal
ends digestion; `\openin` and other probes never reach a diagnostic; the only sanctioned silence is
`IgnoreDiagnosticsScope`. Read logs by counters, match `(Warning|Error|Fatal):[A-Za-z_]+:` anywhere in the line
(WISDOM 85).
