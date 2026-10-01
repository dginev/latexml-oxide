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

**Working priority (post-56ee).** Drive from the S3-recall tail (docs below ~90%
recall = real content loss) and from semantic-markup gaps, NOT from error-cluster
mining — the arXiv error histograms proved a weak, partly-stale proxy (batches
56ed/56ee genuine fixes were spurious-diagnostic suppressions; `\NewTaggingSocket`
etc. were stale-log false-positives that reproduce 0 errors on the current binary).
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
`tools/perfect_kernel/scoreboard.py` reports the in-scope quality table first and the canary table after it
(`--scope all` for the whole corpus). The manual regression net (K17) is re-selected from the in-scope set.

On sweep #130 the in-scope set is at clean **1,585 / 1,602 (98.9 %)**, errors 388 (of 11,141 corpus-wide),
schema-valid 1,569, recall mean 95.93 (median 99.2, 84.3 % at ≥ 95 %), missing words 19,468; the canaries
hold 64 Fatal and 3 timeouts. In-scope worklists (sweep #130):
- **Not clean (17):** timeouts wheelchart, pgf-interference-de/-en; Fatal zxjafont, chessboard_and_beamer;
  errors kaytannollista-latexia (178), tabularray (26), bxjscls-manual (15), bxcjkjatype-ja (12),
  kanbun-example (11), biblatex-ext (10), bxcjkvert-ja (9), bxcoloremoji-ja (7), qworld (7), zhlineskip (6),
  robustsample (2), elzcards-examples (1).
- **Schema-invalid (30, + 3 without XML):** suanpan-l3 (9,481 jing lines), biblatex-gost-examples (770),
  cms-dates-intro (80), glossariesbegin (44), mfirstuc-manual (32), cms-noteref-demo (18), skeldoc (15),
  zx-calculus (8), jsonparse-doc and philexmanual (7), and 20 with 1-5.
- **Recall below 95 % (245 manuals, 13,810 missing words; 133 below 90 %, 63 below 80 %):** the most missing
  words in dinbrief (1,932), geradwp (777), abntex2cite (618), JACoW A4/Letter (483 each), milsymb (389),
  montex (374), elsdoc (327), kotex-doc (278), sduthesis-demo (270).

## Continuation — state and next steps (2026-09-28)

Branch `perfect_kernel` (check `git branch --show-current` first). Delegate read-only work
to Opus 5.5 (xhigh) `root-causer`/`reviewer`/`log-scanner` agents (≤4 at once). Memory cap 8 GB
(`--max-memory=8192`, `ulimit -v 8912896`, per conversion only, never on the test runner).
Sweeps: `~/data/pk_agents/w70/sweep130_launch.sh <label>` is the current recipe (vendor TL +
`LATEXML_DUMP_DIR` vendor dumps, JOBS=16, 180 s, cores 0-63; sweep → validate → post mono → HTML
recall in one chain). arXiv A/B: `tools/perfect_kernel/arxiv_ab.sh <run> <binA> <binB>` (3,003 papers; manual-corpus
A/B: `manual_ab.sh`; scoreboard: `scoreboard.py`). Never run an engine with its cwd in the vendor TL doc tree. `run_doc.sh` removes
dead runs' spill directories there.

**Working rules (user, 2026-09-23):** every fix is catalogued as a minimal `.tex` repro in
`tools/perfect_kernel/repros/<mechanism>/` AND a red/green guard (a generalizing refactor pass is
planned at the end); a schema win counts only if content is preserved — run
`tools/perfect_kernel/content_diff.py old.xml new.xml` on every witness and
`pdf_recall.py` (PDF from the INTENDED engine; `repros.sh <topic> --recall` per repro).

**Phase 57 (2026-09-27, the generalization pass):** sweep #129 (`latexml_oxide.56kx5-rel`, batches 56kt–56kx) equals
#128 in every quality column (clean 1,933 / 2,374, errors 11,149, schema-valid 2,272, recall mean 95.76), so phase 56 closed
clean. Phase 57 works through the ARCHITECTURE_THEMES ordering: K17 (the per-batch manual regression net, `manual_net.sh`)
landed in 57a; K13 (binding-conformance detector, designed in KERNEL_CAPABILITIES) has stage 0 (`DeclaredMode`) landed
in 57b and stage 1 (the chain walker and comparator, `latexml::conformance`) in 57c; its first finding (`\parbox`
paragraph start, KPE #309) landed in 57d, the `isVAttached` child-count port its review found in 57e, the
`{minipage}` counterpart in 57f, and in 57g (user-ruled) a box that is clearly a figure becomes a `<figure>` holding
its image and figure-boxes on one TeX line form an uncaptioned outer `<figure>` of panels; stage 2 (the corpus audit,
`binding_conformance.sh`, all 612 package bindings) landed in 57h, its 60 complete HIGH findings (clean sessions, less three walker limits) are the 57i worklist
(SYNC_STATUS), of which 57i landed the first five (orcidlink, xr, physics `\xmatrix*`, fontawesome, arydshln) and 57j the second round (threeparttable, caption/subfig, titlesec, revsymb); then its side findings (literal primitives entering horizontal mode, changepage, apacite), the rest, stage 3, K14, K15. Batches keep the gate ladder, with L2
now the net.

**Plan to phase 58 (2026-09-28).** Phase 57 closes, and 58 opens, when: (1) a sweep on the phase-57
head holds #130 in every in-scope quality column (clean 1,585 / 1,602, errors ≤ 388, schema-valid 1,569,
recall mean 95.93; cpu_h within +3 %) with every mover classified, and the crash canaries gain no Fatal or
timeout (scope: "Scope" above; the whole-corpus bar was clean 1,933, errors ≤ 11,149 at #129); (2) K13 has stage 3 (environments' `\endX`, classes,
the allowlist) and its HIGH findings are landed or verified no-change; (3) K14 and K15 have landed their
first stage with their class guards green; (4) every RED repro (≈100 in 24 topics at 57aa) has a SYNC row —
fix, PERL-ORIGIN kept, or ruling needed; (5) L6: the cortex reruns of 2605/2606 on the closing binary show no
new Fatal cluster. Work runs in trains of 3-4 batches (gates + reviewer per batch, arXiv A/B + manual net
per train), with up to four read-only Opus 5.5 agents in the analysis lane and the main session as the
single writer.

| Train | Batches (main session) | Analysis lane (≤4 read-only agents) | Compute lane |
|---|---|---|---|
| T0 (done) | 57aa: `\let` copies stay `\let`s in the dump; copies a preload takes follow the job's `\jobname` | — | A/B 57z1→57aa, push; then **sweep #130** (none since 57a, 27 batches) |
| T1 (57ab-57ad landed: DefMath groups + font at digestion, `mathvariant="normal"`, ASCII ligature; global reset lists; `\ContinuedFloat`; 57ae-57ag math; 57ah K13 residue batch 1: amsaddr, varioref, attachfile, svn-multi, savetrees, placeins, case changers, todonotes, supertabular; 57ai-57am math: operator application, OPFUNCTION arguments, bigop operands, bar pairs as deep as Perl's `MAX_ABS_DEPTH`; 57an: the green `math-parse/` repros are grouped golden pairs under `latexml_oxide/tests/parse/`) | #135 `mathvariant` per MathML Core; the K13 stage-2 residue batch 2 (relsize, caption `\DeclareCaptionOption`, siunitx pre-unit and qualifier; MnSymbol/fdsymbol `\not` harmless); 57ag follow-ups (sequent lists, an operator's bare argument); captions-floats REDs (`\ContinuedFloat`, `\subref` letter, panel-row `\par`, sub-label formats); test hygiene (self-contained byte-mouth repro; env-mutating tests → thread-local overrides) | A: K13 residue verdicts against the real `.sty`; B: captions-floats REDs; C: math REDs (#117: VERTBAR, `\left` delimiters, DefMath font, `\DeclareMathOperator`); D: sweep #130 movers (log-scanner → root-causer) | A/B + net for T1 |
| T2 | K13 stage 3: the walker/comparator over environments (`\endX`), classes, the allowlist; audit run; top HIGH findings in two batches | A, B: rank and verify stage-3 findings per package family; C: boxes-groups REDs (16); D: kernel-alignment REDs (8) | audit run; A/B + net |
| T3 | K14 stage 1 (two-phase `Dimension`; class guards 56jz, 56jr, tkz-grapheur, bxcalc, PixelArtTikz) + K18 step 2 residue (`\resizebox`, `\Gscale@div`, makecell, diagbox) | A: K14 per-type order from K13's report; B: K18 sizer audit; C, D: root causes for T2's findings | A/B + net; **sweep #131** |
| T4 | K15 stages 1-2 (hlist tail record, `\unskip`/`\lastskip`, one boundary trim; class guards french highpunct, paragraph text-node split, `\@bsphack`) — golden churn, its own train; then retire the per-site whitespace patches | A: catalog every whitespace patch K15 replaces; B: golden-diff classifier; C, D: the remaining RED topics | A/B + net; **sweep #132** + cortex 2605/2606 → phase 58 |

Rulings pending (not scheduled until ruled): #73 `\trivlist` item binding; non-`normal` `mathvariant` on
`<mn>`/`<mo>` (MathML Core keeps only `normal`, on `<mi>`); `\meaning` of expandable primitives
(`CODE(0x…)` vs TeX's name); where a formula's trailing punctuation goes (math root-cause batch E, ≈80 arXiv papers). Phase 58 then takes the endgame order per ARCHITECTURE_THEMES: stream G's
arXiv rerun preparation and K16 (bibliographies from the style's programs).

## Roadmap — ranked streams, parallel lanes, acceptance gates (user-accepted 2026-09-25)

This is the single ranked order for the program. PLANS.md, KERNEL_CAPABILITIES.md and the open
residuals below feed it; they do not rank on their own. Every stream is sized on the latest sweep and
names the scoreboard column it must move.

| # | Stream | Size at s122 | Moves | Lane |
|---|---|---|---|---|
| 0 | Sweep #123: baseline for 56in–56iq | — | all columns | compute |
| A | Recall tail | 404 of 1,890 scored manuals below 95 %, 26,310 missing words; the worst 50 hold 16,246 | recall mean, %≥95, missing | analysis → implement |
| B | Manuals that finish with errors | 388 docs at status 2 | clean, errors | analysis → implement |
| C | **Performance** (user, 2026-09-25: part of goal completion). C1, the 180 s ceiling: PLANS 12 raw-interpretation speed. C2, throughput of the typical document, which the 2.8M-paper arXiv rerun pays. C3, peak memory at the 8 GB cap | C1: 7 timeouts, 29 docs > 60 s. C2: corpus 2.42 h, arXiv sample 1.60 s/paper; cpu_h drifted 2.10 → 2.42 h over s118-s123 (+15 %, part of it more docs completing, not yet attributed). C3: not yet recorded per doc | timeout, >60s, >120s, cpu_h, p90/p99; arXiv secs | analysis → implement |
| D | Architectural generalizations: virtual file store, `\everyeof`, expl3 file boundaries, PLANS 13 | taken up when A–C hit them | neutral-or-better + less special-case code | implement |
| E | Guard strength (PLANS 5: B1 `assert_element`, B5, B2–B4) | 938 weak assertions | weak-assertion count | implement, 1 batch per sweep cycle |
| F | K6 DVI cue from a `dvips`/`dvipdfmx` class option: RULED 2026-09-25, correct in principle but low priority (we always build XML); only when a witness shows a content or diagnostic difference | 4 manuals, 2 arXiv papers | — | implement on evidence |
| G | Endgame: the full arXiv corpus rerun on the fleet | ~2.8M papers | fleet status distribution | compute |

**Parallel lanes.**
- **Compute:** one sweep or A/B at a time on cores 0-63 (`systemd-run`, JOBS=16).
- **Analysis:** up to 4 read-only Opus 5.5 (xhigh) drivers, one per stream, probing on cores
  64-127. Each returns execution-ready plans: witnesses, a red repro, file:line root cause, a fix
  shape, a guard design, the expected metric delta and a risk.
- **Implementation:** the main session is the single writer. It takes the drivers' plans in
  value order and batches 3-5 fixes.
- A, B and C analyse in parallel. D is triggered by their findings. E runs while a sweep occupies
  the compute lane.
- Performance has two drivers: C1 on the outliers, and C2 on a representative arXiv sample (median
  papers, not outliers). C2 attributes the s118→s123 cpu_h drift per batch and finds the hot paths
  that every paper pays: tokenizer, expansion, fonts, DOM, math parse, post.

**Gate ladder (every fix, every batch):**
1. L0 red: the repro in `tools/perfect_kernel/repros/<mechanism>/` shows the defect on today's binary.
2. L1 green: the guard (whole-element assertions, pinned diagnostics) passes; full nextest,
   clippy and rustdoc pass.
3. L2 manual A/B: the fixed manual regression net and the repro catalog, every batch
   (`tools/perfect_kernel/manual_net.sh` + `manual_net_compare.py --recall`, K17), plus the manuals
   that exercise the mechanism, by grep (`manual_ab.sh` + `manual_ab_compare.py`). No manual may lose
   recall, and the REGRESSIONS block is empty or every row has a reason.
4. L3 arXiv A/B: `tools/perfect_kernel/arxiv_ab.sh`, 3,003 papers; results in `~/data/pk_agents/ab56il/results/`. No status or word loss that
   pdflatex does not explain.
5. L4 reviewer, then one commit per batch.
6. L5 full sweep every 2-3 batches, with a scoreboard row. No manual down by more than 0.5 recall
   and none newly invalid, unless classified faithful to pdflatex.
7. L6 periodically, the cortex reruns of sandboxes 2605/2606. L7 at the end, stream G.

**Performance rules** (stream C; `docs/performance/PERFORMANCE.md`, memory perf notes):
- Levers are algorithmic or strategic only, with no caches or carried state in gullet/stomach/mouth/document.
- Stay behind the Perl-shaped interfaces.
- Profile first, with `--profile bench` symbols.
- One lever per measurement, with error bars; instructions are steadier than wall time under load.
- pdflatex's time on the same source is the throughput oracle.
- A document over 60 s needs a stated reason (`slow_calls.sh`).

**Speed gate:** every batch's L3 arXiv A/B prints `secs A→B` and the papers slower by more than 50 % and 5 s (`arxiv_ab_compare.py`). Each slow paper needs a reason. L5 must not raise cpu_h by more than 3 %, or the >60 s / >120 s counts, without an attributed cause. Manual-subset timings run beside another A/B are not comparable: re-time a flagged document alone.

**Scoreboard** (`tools/perfect_kernel/scoreboard.py`; clean = status 0-1; cpu_h = the sum of per-document conversion seconds; from s125 recall uses the inline-glue audit walker, 0d10215663 — s124 re-scored with it reads mean 95.60, missing 30,222):

| sweep | clean | fatal | timeout | errors | valid | recall mean | median | %≥95 | missing | cpu_h | p90 s | p99 s | >60 s | >120 s |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 113 | 1881 | 94 | 6 | 11252 | 2219 | 93.72 | 98.6 | 75.8 | 45359 | 2.31 | 4.0 | 62.2 | 27 | 15 |
| 118 | 1881 | 94 | 5 | 11139 | 2226 | 93.93 | 98.6 | 76.7 | 44570 | 2.10 | 3.4 | 61.2 | 26 | 15 |
| 119 | 1902 | 67 | 7 | 10654 | 2249 | 94.20 | 98.6 | 77.2 | 46052 | 2.20 | 3.5 | 62.7 | 27 | 16 |
| 120 | 1895 | 69 | 6 | 10548 | 2250 | 94.27 | 98.6 | 77.4 | 45398 | 2.25 | 3.5 | 66.7 | 28 | 18 |
| 121 | 1901 | 67 | 8 | 11255 | 2249 | 94.47 | 98.7 | 78.0 | 43072 | 2.31 | 3.6 | 71.1 | 30 | 19 |
| 122 | 1912 | 66 | 8 | 10529 | 2246 | 94.73 | 98.7 | 78.6 | 35374 | 2.41 | 3.6 | 67.0 | 28 | 17 |
| 123 | 1908 | 70 | 7 | 11853 | 2269 | 95.05 | 98.8 | 79.8 | 34203 | 2.42 | 3.5 | 69.6 | 29 | 18 |
| 124 | 1923 | 67 | 8 | 12045 | 2269 | 95.27 | 98.8 | 80.4 | 32133 | 2.45 | 4.1 | 74.7 | 31 | 17 |
| 125 | 1920 | 67 | 6 | 11222 | 2271 | 95.64 | 99.1 | 82.0 | 29519 | 2.23 | 3.9 | 64.7 | 29 | 16 |
| 126 | 1927 | 69 | 6 | 11165 | 2271 | 95.75 | 99.1 | 82.9 | 29473 | 2.13 | 3.8 | 60.5 | 26 | 14 |
| 127 | 1932 | 66 | 7 | 11142 | 2271 | 95.80 | 99.1 | 82.9 | 29427 | 2.25 | 3.8 | 64.9 | 29 | 15 |
| 128 | 1933 | 66 | 6 | 11149 | 2272 | 95.76 | 99.1 | 82.9 | 29434 | 2.18 | 3.7 | 66.1 | 28 | 15 |
| 129 | 1933 | 66 | 6 | 11149 | 2272 | 95.76 | 99.1 | 82.9 | 29434 | 2.17 | 3.7 | 64.3 | 29 | 15 |
| 130 | 1934 | 66 | 6 | 11141 | 2273 | 95.76 | 99.1 | 82.9 | 29437 | 2.17 | 3.7 | 64.6 | 27 | 15 |

**Open residuals** (carried from phase 56; the ranked leads, the 2026-09-25 state and the healthy-subset
projection are in [`perfect_kernel/archive/PERFECT_KERNEL_PHASE56_NOTES_2026-09-27.md`](perfect_kernel/archive/PERFECT_KERNEL_PHASE56_NOTES_2026-09-27.md)):
- **Kernel, Rust-reachable (arXiv 2605):** polytable needs array.sty's `\@mkpream`/`\@classz` builder
  (2605.08990); 621 raw-mhchem `\ce` exceed the 16M conditional cap (2605.27177, volume, not a loop);
  three pgf/tikz loops (2605.00058, .04377, .12601); the cycle guard's false positive on a large
  repetitive table (2605.11798); the autoload hoist as a load at group level 0 (DIVERGENCES #282).
- **Engine:** a group-local `\def` of a locked control sequence, and Rust's empty root where Perl keeps a
  partial document (both 2605.31475, LEDGER 56hk: pgffor's `\foreach \x/\tag` refused by the lock on amsmath's
  `\tag`; Perl ends in a bounded `Fatal:misdefined`, Rust in an unbounded pushback); catoptions' residual
  option-stack-limit error.
- **Content (axis 1), genuine losses on s129:** arabi/samplebook — the non-Latin body text is absent from the XML
  (cp1256 inputenc + LAE fontenc; 0 Arabic code points, recall 0/196, 0 diagnostics; the s111 triage's one
  genuine kernel lead); notebeamer/notebeamer-demo body absent (recall 1.4 %, RUST-ONLY);
  uantwerpenexam-example1/2 (80.3 / 83.1 %).
- **Semantic coverage (axis 2b, `semantic_coverage.py`; s114: sections/lists/floats/refs 93-98 %, equations
  93 %, graphics 88 %, `\part` 59.5 %):** the graphics family (36 documents short) and
  the multi-family deficit documents, most of them also status 2; ctex's localized part label reads
  "Part I"; beamer overlays are not acted on (Perl's `ltx_covered` wrapper, #270), and frames are not Perl's
  `ltx:slide` in `ltx:slidesequence` model (the follow-up of PLANS P12, archived); a `\caption` in a
  non-float box becomes its float, but the box's other content stays beside it.
- **Class census** (`~/data/pk_agents/w67/clscensus7/census7.tsv` on `vendor_dumps56id`, letgut re-run since: 496 / 500
  usable classes clean): novel (12 errors: its images' `_` in text, SHARED; two layout `\the` errors; fontspec's
  main-font check), imsproc (1, NFSS `.fd` loading), udesoftec (1, `\abstract` constructor),
  upmethodology-document (2, SHARED); not usable: tikzposter (parked). Re-run after each class batch with
  `clscensus7/run.sh <class> <binary>`, whose dump directory must match the binary.
- **Performance ceiling (> 180 s at 8 GB):** pgf-interference (a flat expansion profile); lie-hasse and
  wheelchart exceed the budget in their own engines too (277 s, > 900 s); the Talbot manuals (datatool-user,
  glossaries-extra-manual, 212-334 s) and tcolorbox are digest-bound, the lever being native l3 int/tl primitives
  (PLANS 12).
- **SHARED with Perl, not pursued:** the arXiv 2605 Fatal remainder (els-mrw bibitems, class-body macros under
  OmniBus, enumitem `label=\theenumi` self-reference, `\theequation` in `\tag*`, the mode-frame family, missing
  packages); g-brief letter fields (page furniture, 2 documents); tufte's
  citation `\marginpar`s left empty (LEDGER 56ip); the silent-loss structural scan (empty `<p/>`, icon
  inline-blocks, struts) is settled.

**Method notes:** read a witness's `(Loading …)` lines before assuming the raw-class path; probes
MUST pin the vendor TL (an unpinned run reads the distro tree — qworld reproduced only pinned);
stage by line when batches share a file (`git hash-object` of a HEAD+hunk file into the index);
a `git stash --keep-index` pop can meet the committed half — resolve with the stash's copy.

**Ruled / documented, do not re-open:** engine-primitive ink (21 docs) out of scope +
harness retry only; section-in-item/figure (14) Perl parity; thuaslogos duplicate `pgfcp*`
ids (`\copy` of a digested SVG box, SHARED, HIGH-risk id rewrite); ndsu `text` in
`listing` (document bug); translation-biblatex-de `\begin[…]{description}` (document bug);
M4/M5 bespoke title-page layouts (ltnews, l3news, lua-tikz3dtools, nostarch, elteiktdk,
sduthesis, aomart); dangling IDREF (27, RULED-KEEP); ~92 no-XML fatals (mostly intended
engine ≠ pdflatex); bilingual documents (one manual per language, each half with its own
`\maketitle` and a cross-language `\hypertarget`/`\hyperlink` pair: circledtext, joinbox,
pascaltriangle, suanpan-l3) out of scope for now, potentially interesting later — the
document keeps the last title, and the first half's anchor link dangles (2026-10-01).

**Diagnostics rule in force (2026-09-20/22):** every Warning/Error/Fatal logged AND counted
once; a resource Fatal ends digestion; `\openin` and other probes never reach a diagnostic;
the only sanctioned silence is `IgnoreDiagnosticsScope`. Read logs by counters, match
`(Warning|Error|Fatal):[A-Za-z_]+:` anywhere in the line (WISDOM 85).
