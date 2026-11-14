> **COMPLETE — the goal was met on 2026-11-13 and merged into main as `6df4632fb1`.** Archived verbatim from
> `docs/PERFECT_KERNEL.md`; the program's rows continue in [`docs/perfect_kernel/`](../perfect_kernel/README.md).
> The live residual (the deferred list and the after-merge queue) is [`SYNC_STATUS.md`](../SYNC_STATUS.md) row R.

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

This is the single ranked order to the goal. The agent reports (`~/data/pk_agents/main/agent_reports/`, evidence and designs on file), the open rows of
`SYNC_STATUS.md`, PLANS.md and KERNEL_CAPABILITIES.md feed it and do not rank on their own. The superseded phase-57/58
plan, the 2026-09-25 stream table and the corpus-wide scoreboard history are in
[`perfect_kernel/archive/PERFECT_KERNEL_PHASE57_58_PLAN_2026-10-02.md`](../perfect_kernel/archive/PERFECT_KERNEL_PHASE57_58_PLAN_2026-10-02.md).

**Goal bar** (proposed 2026-10-02 from the quality axes, the scope and the rulings; met at sweeps #146-148,
confirmed by the user 2026-10-04). One
sweep on the closing head shows, on the in-scope set (the 1,602 oracle-clean manuals less the ruled-out ones above):
- **G1 clean:** every manual at status 0-1; a remaining error only of a ruled-out kind (biblatex-ext's bibliography
  inside a tcolorbox; a sectioning unit inside a list item or figure, Perl's error restored by user ruling 2026-10-04,
  OXIDIZED_DESIGN_DIVERGENCES #189).
- **G2 valid:** every manual jing-valid, or each jing line in a ruled class — a dangling `\hyperlink` target (D15), a
  sectioning unit in a list item, a bilingual manual's cross-language link.
- **G3 recall:** every manual at ≥ 95 % S3 recall, or its gap classified as no conversion loss: reference-side (text
  inside included PDFs or images, PDF-extraction garble, CJK segmentation, a golden typeset from another source —
  `tools/perfect_kernel/golden_reference.tsv`), ruled page furniture (kept as metadata once the 2026-11-11 element lands: QA22, after the merge),
  date words, external-tool output.
- **G4 time:** no manual over 180 s at 8 GB in the sweep.
- **G5 canaries:** no new Fatal or timeout among every other manual — the 772 no engine compiles plus the ruled-out
  ones (68 at s134: 64 Fatal and 3 timeouts of the 772, zxjafont's Fatal; 57-58 since #183, 58 at the closing sweep #186).
- **G6 no regression:** every batch's arXiv A/B neutral or explained; semantic coverage (`semantic_coverage.py`) not
  lower; at the close, the cortex reruns of sandboxes 2605/2606 (L6) show no new Fatal cluster.

**State against the bar** — sweeps #133-149 and the bundle sweeps to the close; `scoreboard.py` prints the open counts
per sweep and `scoreboard.py --open N` the manuals (goal set: the 1,602 in scope less those in `out_of_scope.tsv` and
`shell_escape_excluded.tsv` — 1,558 at #133-148, 1,552 since #149; ruled counts in `accepted_residuals.tsv`):

| sweep | G1 unclean | G2 invalid | G3 below 95 % | G3 missing words | G4 over 180 s | G5 canary Fatal/timeout |
|---|---|---|---|---|---|---|
| 133 (58q5) | 4 | 3 | 143 | 4,020 | 3 | 68 |
| 134 (59c1) | 5 | 3 | 143 | 4,020 | 1 | 68 |
| 135 (59l2) | 1 | 2 | 142 | 3,992 | 1 | 68 |
| 136 (60b4) | 4 | 1 | 128 | 4,285 | 0 | 68 |
| 137 (60f1) | 1 | 1 | 125 | 3,629 | 2 | 68 |
| 138 (60j4) | 1 | 1 | 121 | 3,553 | 0 | 68 |
| 139 (60l5) | 1 | 0 | 118 | 3,456 | 2 | 68 |
| 140 (60t3) | 0 | 0 | 99 | 2,776 | 0 | 68 |
| 143 (61k5) | 0 | 0 | 20 | 753 | 0 | 68 |
| 145 (61m17) | 0 | 0 | 7 | 451 | 2 | 68 |
| 146 (61p6) | 0 | 0 | 0 | 0 | 0 | 68 |
| 147 (61q2) | 0 | 0 | 0 | 0 | 0 | 68 |
| 148 (61t2) | 0 | 0 | 0 | 0 | 0 | 68 |
| 149 (64x, e57be39640) | 1 | 1 | 0 | 0 | 2 | 66 |
| 168-185 (int154a-int170a) | 0* | 0 | 0 | 0 | 0 | 57-58 |
| 186 (int171a, bfda096e68) | 0 | 0 | 0 | 0 | 0 | 58 |

\* #178-#180: G1 2 / 3 / 1 (pagelayout's two manuals → PGR1b; ddphonism, tikzbrickfigurines-doc → DTL1h; algxpar-doc →
VARW), each clean on the next bundle.

The 2026-10-04 close status and its "Still to close" list (all superseded since), the in-scope scoreboard s130-s138
and the off-the-path leads of 2026-09-27 are in
[`perfect_kernel/archive/PERFECT_KERNEL_CLOSE_2026-10-04.md`](../perfect_kernel/archive/PERFECT_KERNEL_CLOSE_2026-10-04.md).

**Close status (2026-11).** The goal bar of 2026-10-04 held through the frontmatter single-engine work (Option C,
bundles int154a-int171a) and is re-validated on the final head (row R's GOAL), up to and except the rows DG moved after
the merge (below). Basis per bar, latest first:
- **G1-G4 (manual sweeps, goal set 1,552):** every bundle sweep #168-#186 G1-G4 0 except #178 G1 2 (pagelayout's two
  manuals, PGR1 reading a `\dimexpr` operand inside pgfmath → PGR1b), #179 G1 3 (DTL1d's promotion of pre-existing
  Rust-only overflows: ddphonism, tikzbrickfigurines-doc → DTL1h; algxpar-doc → VARW) and #180 G1 1 (algxpar-doc →
  VARW), each clean on the next bundle. Row 0 (every manual regression since #149 addressed) met at #152 (`pk/int142a`)
  and held since; its last three deferred items are post-merge (FNTN, GINS, TUZK). Closing sweep: #186 on int171a
  (`bfda096e68`; the final tree `ad465256df` adds INT1, byte-identical, and FIX171: an old tcolorbox, Greek list
  names), G1 / G2 / G3 / G4 = 0 / 0 / 0 / 0 on the 1,552; 1,602 docs, 1,592 clean; recall mean 96.89, median 99.5,
  ≥ 95 % 87.7 %; missing words 13,616 (#185 13,619); cpu_h 0.74.
- **G5 (crash canaries):** #185 57 vs #183's 58 (lie-hasse at the 180 s line, jitter), the same set as #172; 68 at the
  2026-10 close (s134), 10 gone at #152. Closing sweep #186: G5 58 = #183's 58, no new Fatal or timeout (canaries fatal
  50, timeout 7): tikzpingus-doc straddles the 180 s line (#158-#163, #171, #176; #183 timeout, #184 176.9 s, #185
  172.8 s, #186 180.1 s; `instructions:u` int170a 2,532.25 G vs int171a 2,527.99 G, −0.17 %).
- **G6 — every bundle's arXiv A/B** (28,354 papers since int154a, every changed paper classified; E/F = Error/Fatal
  results): int154a 909 changed, E 45,800 → 45,827, F 172; int155a E → 45,818; int156a 23 changed; int159a 21 changed,
  0 worse; int160a 269 (1 worse → RB1c); int161a 86, 0 worse, E → 45,788; int162a 830 (`tex=` only, RV1), E → 45,762,
  2 worse → RG1, FMK1; int163a 1,295 (marks, P4c), E → 45,784, 6 worse → P4cR, CAP1; int164a 901, E → 46,288 (+504:
  DTL1d's promoted overflow Warnings, XML byte-identical), 2 regressions → PMK1, DTL1h; int165a 362, E → 46,269, 1
  worse-minor (0705.2788 → TAILN, counted); int166a 94, E → 45,901, F 172 → 170, 6 worse → P4b4f, GA2c; int167a 907
  (GA3's `sort=`), 0 worse; int168a 96, 0 worse; int169a 30, 9 worse → P6e2, P28m3 (re-checked on int169b); int170a 18,
  E 45,696 → 45,702, F 170, 0 worse; int171a 81 (28 frontmatter, 20 XML-only, 33 counts/logs only), E 45,692 →
  45,686, F 171 = 171 — CL171a 23 better / 4 lateral / 1 worse (2502.05252, the legend-line shape DG ruled post-merge
  by reach: LEGAB), CL171b 26 better / 26 lateral (25 math-parse wall-clock flips under load, phantom on an idle box) /
  1 worse (1606.02242, an old tcolorbox) and a Greek TOC `lists` regression (0712.0401), both fixed by FIX171; net
  words −41 (+81 frontmatter, −8 MEM1's phantom chapter-number boxes, −114 math flips). Each bundle's worse papers were
  fixed before its push or ruled post-merge by reach (DG's pragmatic bar, 2026-11-12). The nets: 136, worse 0 at every
  bundle since int155a except int164a's 1 (2109.14286 → DTL1h). Final head (`ad465256df`, the lean recheck int171b):
  the 81 changed papers 59 byte-identical, 4 graphicspath-only, 17 math-only, 0712.0401 fixed, 1606.02242 0 E; stage2
  1,434, 0 creator changes; nets 136 worse 0.
- **G6 — semantic coverage** (`tools/perfect_kernel/semantic_coverage.py`): the closing sweep #186 equals #174 per
  family — part 83.3 %, chapter 87.8 %, section 97.3 %, subsection 98.9 %, equation 96.2 %, graphics 92.2 %, cite
  96.0 %, ref 97.5 % — except tabular 98.8 → 98.7 %: five memoir manuals (biblatex-oxref's 4 docs, MemoirChapStyles)
  lose the chapter-style heading tabular that held a phantom number and a rule, replaced by MEM1's `<TOC>` (better).
  All above #114's recorded 59.5 % (part), 93-98 % (sections/lists/floats/refs), 93 % (equations), 88 % (graphics);
  #146-148's outputs are gone.
- **G6 — cortex reruns (L6):** the fleet rebuilt at `862de44e59` (dumps regenerated on the distro TL); runs 373 (2605) /
  374 (2606) against 368/369: 2605 error 923 → 893, fatal 16 → 8; 2606 error 897 → 881, fatal 22 → 19; 57 papers newly in
  error, 3 new fatals. One new Fatal cluster — `\c@author`, biblatex's name-list counters (2606.16762, 2606.26281) —
  fixed by CAU1 (`4af92450cd`); the error regressions (DG 2026-11-13 "Fix all 12 now") by SBX1, SBX2, GRP1 in int171a,
  every witness 0 E. The reruns on the pushed final head (CORTEX2, DG 2026-11-13: "ideally they do not have obvious
  degradation by the time we merge to main"): runs 375 (2605) / 376 (2606) on `2a1fa68d95`
  (2026-11-13, the fleet rebuilt, dumps from the distro TL): 2605 error 893 → 875, fatal 8 → 9; 2606 error 881 → 863,
  fatal 19 → 15 (against October's 368/369: 923 → 875, 16 → 9; 897 → 863, 22 → 15). Against 373/374, 42 papers better
  and two worse, both noise: 2605.17422 Fatal:oom at the worker's address-space limit (4.04 GB RSS on int170a and the
  final tree alike), 2606.10946 Fatal:timeout under fleet load (48 s alone, `instructions:u` equal). Against October,
  24 papers newly in Error (CX171): 20 the widened author check on unchanged or better output (18 acmart one-block
  author lists, citations now acmart's numeric `[n]`), 2 TeX-correct (2605.10816, 2606.20173), 2 DTLX false overflow
  Errors (2606.14412, 2605.31088; post-merge). No new Fatal cluster; no obvious degradation.
- **Frontmatter bars (per bundle, DG 2026-11-12):** every printed word kept or counted by a Warning, no paper worse on
  any A/B, stage2's creator changes classified at every bundle (int163a 115, 5 keep-better; since int166a 0-2, each
  keep-better or better; int171a 0). The 98/98 claim (truth set, holdout v4) is post-merge: TRUTH, HOLD4.
- **Cost bar ≤ +0.5 % per document (user 2026-11-08):** int170a vs int169b 1710.09320 −5.35 %, 1901.01540 −5.05 %,
  2103.04797 −6.62 %, 1507.02068 +0.28 % (MLN1 +0.13 %, accepted); the >1,000-author AASTeX papers +7.3 / +8.3 % vs the
  2026-10-31 audit after AAS1 (were +13.3 / +14.4 %) — the rest is FMPL, post-merge. The final tree vs int170a: within
  ±0.33 % (1507.02068 −0.164, 1710.09320 +0.058, 1901.01540 −0.000, 2005.08745 −0.106, 2103.04797 −0.003, b5_fill1000
  −0.327, hello +0.002 %).
- **L1 on the final tree:** `gates_pk.sh` int171b on `ad465256df`: nextest 8103/8103 (4 skipped, the `#[ignore]` RED
  rows), cortex 10/10, clippy 0, rustdoc 0, dump inits plain 0 / latex 0, dump diff vs int170a 0; CI on the push to main
  validates CIPERF (8 Linux shards, macOS) with a Windows CI dispatch (its guards optional to pass): at MPR. After
  CORTEX2, CORE0's cached frame-trace flag and the ieeeaccess test's TeX-tree path: gates on `803ce550a6` 8103/8103 again.
Verdict: G1-G5, G6's A/B, coverage and cost bars and L1 met on the final tree; the cortex reruns show no new
Fatal cluster and no obvious degradation — goal met on the final tree (`803ce550a6` and this fold), 2026-11-13, up to
and except the deferred rows below; DG's confirmation before the merge (MPR: a regular command-line merge into main,
DG 2026-11-13).

**Deferred to after the merge, with the ruling.** Moved from before the merge by DG (2026-11-12, "having deferred as
much work as we have already, what else can be deferred", unless dated otherwise):
- **TRUTH** — the bars' truth set (every paper the frozen scorer flags + ~500 it passes, two independent reads, disagreements
  to DG) — after the merge; the per-bundle bars gate the merge.
- **HOLD4** — the fresh holdout v4, read once, 98/98 per creator and per paper — after the merge, after TRUTH; sealed until then.
- **PERF** — the perf + RAM pass's remaining levers (#346) — after the merge; each bundle's cost bar blocks slowdowns.
- **FMPL** — the frontmatter perf levers (FMPERF 2026-11-13: rehearsals ≈0.33 G, `emit_commands` ≈0.53 G, entry
  digestion +0.3 G on the >1,000-author AASTeX papers) — DG: after the merge; FMPERF cut to one measurement, AAS1 the one round.
- **T1** — the repro suite move (1,469 files → `latexml_oxide/tests/repros`) — after the merge.
- **P6R** — P6's rest (`cut_list`, `may_open_with_mark` out of the moved directories; P6S compaction; LST0's plan
  P6R1-P6R4, 2026-11-13) — P6 closes at P6f.
- **QA80** — 8 upstream LaTeXML patches prepared, summary e-mailed to BM — submitted at BM's go and DG's OK; no merge gate.
- **QA92** — the reply owed to BM (Q3.9, Q4.6, Q3.13) — any time; no merge gate.
- **CSS** — the sizing CSS (QA27, BM's em-of-parent lengths; RBX0's stock-CSS transform overlap, RBXB) — after the merge.
- **QA84** — comment compaction across the crates — after the merge, on main.
- **QA75** — the full eyeball review of rendered HTML — after the merge; a short look (QA75S, 2026-11-13) folded into
  SWEEP: LOGO1 and MEM1 fixed before the merge, its minor patterns QA75M after it.
- **CLEAN** — workspace cleanup, keep-list to DG — after the merge.
- **ARXIV (L7)** — the full arXiv rerun (~2.96M papers, ~3 days; the last complete one run 336, 2026-10-10) — after the
  merge, from main; its new Error/Fatal clusters classified into row R.
- **CI** — CIPERF's speedups (TC0's test-cost levers, CI under 30 minutes on main) — after the merge; before it only CI
  fitting its timeouts (DG 2026-11-12; the perfect_kernel guards an optional Windows job, DG 2026-11-13).
- **Earlier rulings kept post-merge:** P2-8b's block-call fact (P28B, DG 2026-11-12); ROW0A's three items (FNTN, GINS,
  TUZK, DG 2026-11-12); SR4/SR5 (QA60 site 8, after DG's "author's group" ruling); IEGNM (ruled post-merge at int167a);
  QA37 side-by-side figures (DG: "requires care"); MENC, `<m:menclose>` (DG 2026-07-30); math (PARKED 2026-10-02);
  the author-block legend lines and nameless creators (LEGAB, DG 2026-11-13: fix before the merge only if many papers
  show them; LEG0 measured 1 of 35,942).

**The after-merge queue** — 229 lines, weight 473, one line each in `SYNC_STATUS.md` row R ("After the merge"), the
moved SYNC rows' detail in `perfect_kernel/OPEN_ROWS.md` "Post-merge rows moved from SYNC_STATUS (SYNC3, 2026-11-12)".
DG 2026-11-13: after the merge, first a medium-sized area from latexml-oxide's open PRs and issues, worked as PRs
against latexml-oxide, before this list. By kind (grouping by reading each line):
- frontmatter residuals and catalogued REDs (Option C's tail, marks, cuts, e-mails, separators SMB1-SMB3, TP2, int171a's
  LEGAB, WRAPN, GRPRD, APCOL), each counted by a Warning where it loses a word: ≈98;
- engine, markup and bibliography REDs and residuals (pgfmath, calc, box sizes, alignments/tabu, graphics/psfrag,
  bibliography ordering, the error boundary, MENC, int171a's RBXA, STARTTOC, MEMW, BRKC) and the A/B harness (MATHAB): ≈52;
- BM Q&A rows (POST / LATER / horizon, QA05-QA91): 39;
- SYNC rows moved by SYNC3 (G, G1, KC, MS, A2-A4, AT, TLY, FONT, FRAG, R6, R7, R8b, CI, REL, CSSPORT, …): 27;
- the pre-merge rows DG moved (above): 13.

**Not on the path:** math-parse fidelity (PARKED 2026-10-02); the out-of-scope manuals; everything in row R's
after-merge list.

## Method

Branch `perfect_kernel` in the main checkout (`~/git/latexml-oxide`), where the main session integrates each bundle,
runs its checks and pushes. Payload writers work one batch each on a `pk/<batch>` branch in a `~/data/pk_wt/<batch>`
worktree and commit there; reviewers and the narrow read-only types (`root-causer`, `reviewer`, `log-scanner`,
`perf-measure`, `math-diagnose`) are read-only; all on Opus 5.5 high, at most 4 at once (user 2026-11-11; 2 before, 1
until 2026-10-07). Memory cap 8 GB (`--max-memory=8192`, `ulimit -v 8912896`, per conversion only, never on the test
runner). Scripts: the shared kit (`~/data/pk_agents/main/kit/README.md`); a bundle's checks are
`chain/bundle_chain.sh` (gates, the one large arXiv A/B, the nets, the manual sweep via `chain/manual_sweep.sh`:
vendor TL + `LATEXML_DUMP_DIR` vendor dumps, 180 s); no nextest beside a sweep or an A/B. arXiv A/B:
`ab_66bundle/scripts/run_ab2.sh` over 28,354 papers once per bundle (a writer's targeted A/B on its own population),
every changed paper classified. Never run an engine with its cwd in the vendor TL doc tree. Every fix is
a minimal `.tex` repro in `tools/perfect_kernel/repros/<mechanism>/` and a red/green guard with whole-element
assertions (user, 2026-09-23); a schema win counts only if content is preserved (`content_diff.py` on every witness,
`pdf_recall.py` with the intended engine's PDF).

**Gate ladder (every batch; L3-L5 per bundle):**
1. L0 red: the repro shows the defect on today's binary.
2. L1 green: the guard passes; full nextest, clippy and rustdoc on the exact committed tree
   (`~/data/pk_agents/main/gates_pk.sh`).
3. L2 manuals: the mechanism's witness manuals before/after (`content_diff.py`, recall); the manual net
   (`manual_net.sh` + `manual_net_compare.py --recall`, K17) for cross-cutting changes. No manual loses recall
   unexplained.
4. L3 arXiv A/B: a small targeted A/B per writer, the large one (28,354 papers) once per bundle; every changed paper
   classified, no status or word loss that pdflatex does not explain.
5. L4 a reviewer before every integration, at most 2 fix rounds after the first review (user 2026-11-11), each round a
   new commit on the batch's branch; the main session merges the bundle and pushes after its checks.
6. L5 a sweep every bundle, with a scoreboard row: no manual down by more than 0.5 recall or newly invalid, unless
   classified faithful to pdflatex; the cost bar (≤ +0.5 % per document, user 2026-11-08) and the frontmatter bars
   (every printed word kept or counted by a Warning, no paper worse on any A/B).
7. L6 at the close, the cortex reruns of 2605/2606; L7 stream G (the full arXiv rerun, after the merge: ARXIV).

**Performance rules** (`docs/performance/PERFORMANCE.md`):
- Levers are algorithmic or strategic only, with no caches or carried state in gullet/stomach/mouth/document.
- Stay behind the Perl-shaped interfaces.
- Profile first, with `--profile bench` symbols.
- One lever per measurement, with error bars; instructions are steadier than wall time under load.
- pdflatex's time on the same source is the throughput oracle.
- A document over 60 s needs a stated reason (`slow_calls.sh`).

**Speed gate:** every bundle's arXiv A/B prints `secs A→B` and the papers slower by more than 50 % and 5 s; each slow paper needs a reason. Every bundle's cost bar: ≤ +0.5 % `instructions:u` per document (user 2026-11-08) on 2103.04797, 1901.01540, 1507.02068, 1710.09320 (and 2005.08745, b5_fill1000, hello; `chain/cost.sh`) against the bundle base. L5 must not raise cpu_h by more than 3 %, or the >60 s / >120 s counts, without an attributed cause. Manual-subset timings run beside another A/B are not comparable: re-time a flagged document alone.

## Scoreboard

`tools/perfect_kernel/scoreboard.py <prev> <N>` prints a sweep's in-scope table (clean = status 0-1; cpu_h = the sum of
per-document conversion seconds; recall from the inline-glue audit walker since s125) and the crash-canary table after
it. The bundle sweeps' figures are in each bundle entry of `archive/SYNC_SESSIONS_2026-11.md`, the closing sweep #186
in the close status above; the s130-s138 table is in
[`perfect_kernel/archive/PERFECT_KERNEL_CLOSE_2026-10-04.md`](../perfect_kernel/archive/PERFECT_KERNEL_CLOSE_2026-10-04.md),
the corpus-wide rows s113-s130 in the phase-57/58 archive.

## Open residuals and rulings

**Off the path:** the leads of 2026-09-27 (kernel arXiv-2605 items, engine, class census, performance beyond the
ceiling, SHARED items), unverified since, are archived in
[`perfect_kernel/archive/PERFECT_KERNEL_CLOSE_2026-10-04.md`](../perfect_kernel/archive/PERFECT_KERNEL_CLOSE_2026-10-04.md)
"Off the path" — probe each before taking it; the semantic-coverage figures are the close status's (#186).

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
(mostly intended engine ≠ pdflatex); page furniture kept as metadata (2026-11-11, FIDELITY_CHARTER §2; its element is QA22, after the merge), semantic notes kept (2026-10-01); a second `\maketitle`
keeps the latest title (2026-10-01); bilingual documents (one manual per language, each half with its own
`\maketitle` and a cross-language `\hypertarget`/`\hyperlink` pair: circledtext, joinbox, pascaltriangle, suanpan-l3)
out of scope for now, potentially interesting later — the document keeps the last title, and the first half's anchor
link dangles (2026-10-01).

**Diagnostics rule in force (2026-09-20/22):** every Warning/Error/Fatal logged AND counted once; a resource Fatal
ends digestion; `\openin` and other probes never reach a diagnostic; the only sanctioned silence is
`IgnoreDiagnosticsScope`. Read logs by counters, match `(Warning|Error|Fatal):[A-Za-z_]+:` anywhere in the line
(WISDOM 85).
