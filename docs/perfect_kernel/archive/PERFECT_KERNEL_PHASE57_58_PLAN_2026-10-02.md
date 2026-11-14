# Perfect kernel — phase 57/58 plan and corpus-wide scoreboard history (archived 2026-10-02)

Frozen copy of the sections of `docs/PERFECT_KERNEL.md` superseded on 2026-10-02 by its "Goal bar and ranked path":
the sweep-#130 in-scope worklists, the phase-57 continuation and the phase-58 train plan (T0-T4; the generalization
trains T2-T4 were not run — the user re-steered to scoreboard-driven batches on 2026-10-01), the 2026-09-25 stream
table with its parallel lanes, and the whole-corpus scoreboard rows s113-s130. Text verbatim.

## Sweep #130 in-scope worklists


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


## Whole-corpus scoreboard, s113-s130

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
