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
## Continuation — state and next steps (2026-09-25)

Branch `perfect_kernel` (check `git branch --show-current` first). Delegate read-only work
to Opus 4.8 `root-causer`/`reviewer`/`log-scanner` agents (≤4 at once). Memory cap 8 GB
(`--max-memory=8192`, `ulimit -v 8912896`, per conversion only, never on the test runner).
Sweeps: `~/data/pk_agents/w68/main/sweep122_launch.sh` is the current recipe (vendor TL +
`LATEXML_DUMP_DIR` vendor dumps, JOBS=16, 180 s, cores 0-63; sweep → validate → post mono → HTML
recall in one chain). arXiv A/B: `tools/perfect_kernel/arxiv_ab.sh <run> <binA> <binB>` (3,003 papers; manual-corpus
A/B: `manual_ab.sh`; scoreboard: `scoreboard.py`). Never run an engine with its cwd in the vendor TL doc tree. `run_doc.sh` removes
dead runs' spill directories there.

**Working rules (user, 2026-09-23):** every fix is catalogued as a minimal `.tex` repro in
`tools/perfect_kernel/repros/<mechanism>/` AND a red/green guard (a generalizing refactor pass is
planned at the end); a schema win counts only if content is preserved — run
`tools/perfect_kernel/content_diff.py old.xml new.xml` on every witness and
`pdf_recall.py` (PDF from the INTENDED engine; `repros.sh <topic> --recall` per repro).

**State 2026-09-25:**
- Sweep #122 (`latexml_oxide.56im`): S0∧S1 1912 / 2365 (80.5 %), recall mean 94.73, median 98.7,
  schema-valid 2246, and 63 manuals up with 0 down vs s121.
- Class census: 492 / 501 usable classes clean. K6 PDF mode landed (56id, DIVERGENCES #285).
- Since s122:
  - 56in: siunitx v3 keys.
  - 56io: minted listing files, geometry length expressions, named frontmatter.
  - 56ip: listing files through kpathsea and the virtual store, shared listings escapes, K11 class
    defaults. Recall: timeop 56 → 100, commalists-tools 77 → 100, randintlist 68 → 99.5.
  - 56iq: the TooManyErrors salvage (27 manuals from an empty document to partial content) and 56gn.
  - Every batch's 3,003-paper arXiv A/B shows no content loss.
- Next: sweep #123 to measure 56in–56iq. The schema axis is at its ruled ceiling
  (`~/data/pk_agents/w59/main/jing113/`). The remaining invalid documents are no-XML runs that fail
  in their own intended engine, dangling IDREF (RULED KEEP), sectioning in an item (RULED LEAVE,
  #189) and SHARED singletons. The levers left are the content and semantic axes below.
- Settled, do not re-mine:
  - The empty margin notes: tufte's citations `\marginpar`, SHARED (LEDGER 56ip).
  - The silent-loss structural scan: empty `<p/>`, icon inline-blocks and struts.

## Roadmap — ranked streams, parallel lanes, acceptance gates (user-accepted 2026-09-25)

This is the single ranked order for the program. PLANS.md, KERNEL_CAPABILITIES.md and the open
leads below feed it; they do not rank on their own. Every stream is sized on the latest sweep and
names the scoreboard column it must move.

| # | Stream | Size at s122 | Moves | Lane |
|---|---|---|---|---|
| 0 | Sweep #123: baseline for 56in–56iq | — | all columns | compute |
| A | Recall tail | 404 of 1,890 scored manuals below 95 %, 26,310 missing words; the worst 50 hold 16,246 | recall mean, %≥95, missing | analysis → implement |
| B | Manuals that finish with errors | 388 docs at status 2 | clean, errors | analysis → implement |
| C | 180 s ceiling: PLANS 12 raw-interpretation speed | 8 timeouts + slow manuals; the same hot paths on arXiv | timeout, wall time | analysis → implement |
| D | Architectural generalizations: virtual file store, `\everyeof`, expl3 file boundaries, PLANS 13 | taken up when A–C hit them | neutral-or-better + less special-case code | implement |
| E | Guard strength (PLANS 5: B1 `assert_element`, B5, B2–B4) | 938 weak assertions | weak-assertion count | implement, 1 batch per sweep cycle |
| F | Rulings: K6 DVI cue (`dvips`/`dvipdfmx` class option) | 4 manuals, 2 arXiv papers | — | user |
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

**Gate ladder (every fix, every batch):**
1. L0 red: the repro in `tools/perfect_kernel/repros/<mechanism>/` shows the defect on today's binary.
2. L1 green: the guard (whole-element assertions, pinned diagnostics) passes; full nextest,
   clippy and rustdoc pass.
3. L2 manual A/B: the manuals that exercise the mechanism, by grep
   (`tools/perfect_kernel/manual_ab.sh` + `manual_ab_compare.py`). No manual may lose recall.
4. L3 arXiv A/B: `tools/perfect_kernel/arxiv_ab.sh`, 3,003 papers; results in `~/data/pk_agents/ab56il/results/`. No status or word loss that
   pdflatex does not explain.
5. L4 reviewer, then one commit per batch.
6. L5 full sweep every 2-3 batches, with a scoreboard row. No manual down by more than 0.5 recall
   and none newly invalid, unless classified faithful to pdflatex.
7. L6 periodically, the cortex reruns of sandboxes 2605/2606. L7 at the end, stream G.

**Scoreboard** (`tools/perfect_kernel/scoreboard.py`; clean = status 0-1):

| sweep | clean | fatal | timeout | errors | valid | recall mean | median | %≥95 | missing |
|---|---|---|---|---|---|---|---|---|---|
| 113 | 1881 | 94 | 6 | 11252 | 2219 | 93.72 | 98.6 | 75.8 | 45359 |
| 118 | 1881 | 94 | 5 | 11139 | 2226 | 93.93 | 98.6 | 76.7 | 44570 |
| 119 | 1902 | 67 | 7 | 10654 | 2249 | 94.20 | 98.6 | 77.2 | 46052 |
| 120 | 1895 | 69 | 6 | 10548 | 2250 | 94.27 | 98.6 | 77.4 | 45398 |
| 121 | 1901 | 67 | 8 | 11255 | 2249 | 94.47 | 98.7 | 78.0 | 43072 |
| 122 | 1912 | 66 | 8 | 10529 | 2246 | 94.73 | 98.7 | 78.6 | 35374 |

**Open leads (ranked; updated after sweep 118 and batches 56gw-56hd):**
1. **Hidden macro-delimiter misses — landed 56iq.** A call that misses its `\def`'s leading
   delimiter is reported and ignored (tex.web §397-398, DIVERGENCES #295), which surfaces the expl3
   expandable errors (`\???`) that the lax engine swallowed. It landed together with the
   TooManyErrors salvage, which keeps the pre-Fatal document as latexmlc does. The misses were fixed
   at their roots along the way (56gt, 56gv, 56gp, 56go, 56gy, 56hb, 56hl). The documents that now
   reach the 100-error limit (euclideangeometry-man, chinesechess, mercatormap) are Fatal in
   pdflatex and lualatex too (LEDGER 56iq). Notes: `~/data/pk_agents/w59/main/56gn_NOTES.md`.
1b. **Recall tail (s116) — landed 2026-09-23**: scrlfile-hook underflow warnings in 983 manuals
   (56gz); uantwerpendocs title-page flow content (56ha, 6 manuals); babel main language from
   class options (56hd, 36 French/German manuals, colortbl-DE recall 86 → 99) with microtype's
   French shorthand switch-off (56hf, cahierprof); uni-titlepage `\maketitle[opts]` (56he, all 13
   manuals at 100 % recall); `\verbatim@start` + verbbox (56hc); `\protected\relax\def` (56hb,
   catoptions 70 → 1); titling `\pretitle`/`\posttitle` material (56hg, lion-msc/minimal 24 → 70 %);
   glossary phrases keep their markup and acronym refs show their phrase (56hi). Open,
   root-caused: quotchap `savequote` (locked `\chapter`), nomencl `nomentbl` unit/note columns
   (the XSLT renders only a glossary entry's label and definition), the glossaries binding's
   `absorb_string` of each key value (math/`\emph` reach the XML as TeX source), g-brief letter
   fields (page furniture, SHARED, 2 docs), catoptions' residual option-stack-limit error.
   arXiv sandboxes 2605/2606 rerun on worker `cortex-worker:56he` (HEAD 4c8a916fdb), runs 311/312 vs
   309/310. 2605: 53 `%auto-ignore` placeholders newly Fatal and 20 PoS papers with a `\q_no_value`
   recursion, both fixed in 56hh, plus 34 siunitx `detect-all` warnings (key registered); the 68
   `\endminipage` warnings are SHARED. 2606 has the same two roots at a larger scale: 154
   `\q_no_value` papers (pos.sty, aaskaiid.sty) and 28 placeholders, all clean on 56hh. Triaged
   2605 singletons: four post-stage `post:parse`/XPath `NO_MEMORY` failures (2605.15130, .21663,
   .02664, .14423) are fleet memory pressure: byte-identical output and peak memory on 56ec and 56he,
   0 errors locally at the 8 GB cap. 2605.15407's rootless Fatal is an unclosed `\ifpdf`, which
   pdflatex runs as TRUE; ours is FALSE, like Perl's. That is the open K6 PDF-mode question.
   2605.19748 is XeLaTeX-only (an unclosed `\ifXeTeX`), SHARED. 2605.07451 (56go's `\everypar` vs
   syntax.sty grammars) is fixed in 56hk. 2605.31475's tikz `PushbackLimit` is SHARED: the lock on
   amsmath's `\tag` refuses pgffor's loop variable. Open: a group-local `\def` of a locked CS, and
   Rust's empty root where Perl keeps a partial document.
1d. **Raw class loading (user direction 2026-09-24) — the class census**
   (`~/data/pk_agents/w67/clscensus/census.tsv`; LEDGER 2026-09-24):
   - 219 of the 720 classes are exempt because no engine compiles them; the manual sweeps exercised only 253.
   - Each class runs under its intended engine's persona (`xetex`/`luatex` where the oracle is xelatex/lualatex).
   - **Census on 56ic (`~/data/pk_agents/w67/clscensus4/census4.tsv`): 492 of 501 usable classes clean (98.2 %; 85.4 % on 56ht, 92.8 % on 56hy, 97.6 % on 56ia), 8 with errors, 1 failed (tikzposter).** 462 convert cleanly through the raw `.cls` path.
   - Re-run the census after each class batch (`clscensus4/run.sh <class> <binary>`, `xargs -P 16`, cores 64-127); it is the measure for this axis and catches lateral regressions (it caught four in 56ia).
   - Landed (56hx-56ia): kernel lengths as allocated `\dimen`s, the `xetex` profile, binding internals that raw classes call, `\chardef`'s expanding `=`, env/document/begin, `\pagestyle` running `\ps@…`, raw option tokens, the kernel `\@ifpackagelater`, titlesec records, patchable `\tableofcontents`, lazy listings colours.
   - 56ib-56ic cleared dtk, univie-ling-poster and letgut and took novel from Fatal to content (12 errors: its images' `_` in text, SHARED; two layout `\the` errors; fontspec's main-font check). Residual: neoschool (the K6 PDF-mode question: `\ifpdf` is false here as in Perl, and a `\pdfoutput=1` persona needs a ruling); imsproc (NFSS `.fd` loading); isodoc/nddiss2e (DVI persona, SHARED); FUbeamer (dvips-only graphicx-psmin); tikzposter (parked); udesoftec (`\abstract` constructor); upmethodology-document (SHARED).
   - Sweep #120 (56ia): fully done 1838 (77.4 %), no recall change; the regressions it showed are fixed in 56ib/56ic, except urcls/URpagestyles-DEMO, which pdflatex fails the same way (the kernel's `\@removeelement` on a braced raw option; oracle-unclean).
1c. **arXiv 2605 Fatal roots (run 313: 144 Fatals, run 315: 136) — root-caused 2026-09-24**
   (root-causers `~/data/pk_agents/w65/`, `w66/`). Landed, each with a repro and guard:
   - 56hq: sn-jnl, mdpi, `\include{x.tex}`, newpxmath.
   - 56hr: XeLaTeX-only packages under the default persona (11 papers).
   - 56hs: `\let\@left\left`.
   - 56ht: the ByteDance Seed `\author` loop (the no-result cluster).
   - 56hu: algorithm2e procedure captions; jmlr `\addr`; ungrouped `\subimport`; listings `\lst@ifdisplaystyle`; the xcolor name sets and active separators; the autoload hoist.
   - 56hv: `scan_font_ident` expansion; `\text..` closes with `\egroup`; the full `\mathcode`; glossaries labels; ieeetj on IEEEtran.

   Most of the rest is SHARED with Perl: els-mrw bibitems, class-body macros under OmniBus, the enumitem `label=\theenumi` self-reference, `\theequation` in `\tag*`, the mode-frame family, and missing packages. Open and Rust-reachable:
   - polytable needs array.sty's `\@mkpream`/`\@classz` builder (2605.08990).
   - 621 raw-mhchem `\ce` exceed the 16M conditional cap (2605.27177, volume, not a loop).
   - three pgf/tikz loops (2605.00058, .04377, .12601).
   - the cycle-guard false positive on a large repetitive table (2605.11798).
   - the autoload hoist's redesign as a load at group level 0 (DIVERGENCES #282).
2. **KOMA-Script `\part` — landed 56gq** (Axis 2b's first finding): the kernel's `\@part`/
   `\@spart` are locked sectioning hooks, so KOMA's `\SecDef\@part\@spart` yields `ltx:part` (24 of
   31 part-short manuals). KOMA `\addpart`/`\addpart*` follow, since `\@addpart` calls `\part` (re-probed 56if).
   Residual: ctex's localized part label becomes "Part I".
3. **Axis 2b is now measured** — `tools/perfect_kernel/semantic_coverage.py <corpus.tsv>
   <sweep_dir>`: s114, 2,274 completed docs — sections/lists/floats/refs 93-98 %, equations 93 %,
   graphics 88 %, `\part` 59.5 %. Known false deficits are listed in its docstring. Next: the
   graphics family (36 docs short) and the multi-family deficit docs, most of which are also
   status-2 (error) documents.
3b. **Beamer lists — landed 56gu**: list environments open one tagged item per `\item` and
   consume overlay specs (61 manuals; cursolatex 0 → 286 items). Residual: overlays not acted
   on (Perl's `ltx_covered` wrapper, #270).
4. **Caption outside a float — landed 56gs**: a `\caption` with `\@captype` set in a non-float box
   is now its type's numbered float (15 manuals, 0 words lost). Residual: the box's other content
   stays beside the float rather than inside it.
5. Perf ceiling (> 180 s at 8 GB): pgf-interference is a flat expansion profile once
   `read_digits`' regex went (56gl, −3 %); lie-hasse and wheelchart exceed the budget in their own
   engines too (277 s, > 900 s). The Talbot manuals (datatool-user, glossaries-extra-manual,
   212-334 s) and tcolorbox are digest-bound: the lever is native l3 int/tl primitives (PLANS 12).
6. Rulings — all decided: dangling IDREF (KEEP, 24 docs), sectioning inside an item/figure
   (LEAVE, #189, 13 docs), the `quote` content model (LANDED 56gw, #271: webquiz, aguplus valid)
   and inline-leak (LANDED 56gx, #272: ribbonproofs, sidenotesplus, ryethesis valid; equation-level
   footnotes now render in HTML). Measured on sweep #118: 2226/2368 (s117: 2221).
   Open ruling (K6 refinement): a DVI cue from a `dvips`/`dvipdfmx` class option (4 manuals, 2 arXiv
   papers; pdflatex fails them alike). Also: the ctex `fandol` fontset under vendor-tree pdfTeX
   fails in pdflatex too.

**Method notes:** read a witness's `(Loading …)` lines before assuming the raw-class path; probes
MUST pin the vendor TL (an unpinned run reads the distro tree — qworld reproduced only pinned);
stage by line when batches share a file (`git hash-object` of a HEAD+hunk file into the index);
a `git stash --keep-index` pop can meet the committed half — resolve with the stash's copy.

**Healthy-subset projection (audited 2026-09-22; root-causer audit of the main
session's first cut, which was wrong on several counts — recomputed with `LC_ALL=C`
sorting; data `~/data/pk_agents/w58/healthy_audit/`).** "Healthy" = the oracle's own
clean definition (`oracle.sh`: intended engine exit 0 AND zero `!` errors); the oracle runs
pdflatex or lualatex only (no xelatex). Two readings of the set, both on s110:

| bar | pdflatex-clean (1,248) | any-engine-clean (1,548) |
|---|---|---|
| completes with 0 Error lines | 1,241 (99.4 %), 7 short | 1,534 (99.1 %), 14 short |
| ≥ 1 Warning line | 636 | 859 |
| schema-valid | 1,216 (97.4 %), 32 invalid | 1,499 (96.8 %), 49 invalid |
| content recall **of record** (s111, post-mono HTML, Unicode auditor) | mean 94.4, **median 98.9**, 242 < 95, 148 < 90, **88 < 80**, 38 < 60 | corpus median 98.6; 57 docs < 60 |

Corrections to the first cut: "1,251 joined" was a sort-locale artifact (1,248 join); the
loose `exit 0` filter admitted 4 docs whose pdflatex run logged `!` errors (typog-example
among them — so it is not healthy and drops out of the error list); "30 invalid" was 33 on
the loose set / 32 on the strict one (aomsample, turnstile ×2, tufte sample-book,
rusnat-doc-ru, l3news, ltnews were missed); eight docs filed as "unruled mechanical"
(skeldoc 15, jsonparse 7, philex 7, biblatex2bibitem 5, europecv, elsdoc, crossreftools,
mla-example) are the RULED dangling-IDREF-keep set; iodhbwm/eqnnumwarn/gaceta and the
turnstile/tufte/shipunov group are SHARED frontmatter-after-body (Perl fails identically);
the project's tracked recall figure is the MEDIAN (98.3 corpus / 98.6 healthy), and the
low tail is mostly REAL loss, not pdftotext artifacts (4 of 5 inspected sub-80 docs:
newpax external PDF-page inclusion, rusnat-ex1 bibliography empty, notebeamer demo body
absent, biblatex2bibitem empty citations; a0poster's umlaut splits are the artifact case).

Honest projection per bar (pdflatex-clean):
1. **Error-free: gated entirely on rulings, no open kernel item (triaged 2026-09-22).** All 7
   are SHARED with Perl and already ruled: four are the K6 PDF-mode persona (`\pdfoutput=0`,
   `\ifpdf` false — bxcoloremoji, gentombow, scanpages, upmethodology; a `\pdfoutput=1` persona
   is a corpus-wide surpass needing a fresh ruling and sweep), chessboard_and_beamer is the
   single-pass overlay model, elzcards is OD #99 (geometry body box), bibarts is the
   write/rescan residue. Two small RUST-ONLY geometry side defects (eager `landscape` swap,
   brace-comma option splitting) are open but do not move any document.
2. **Schema-valid: reachable only as "valid or invalid by ruling".** The 32 split: 8 ruled
   dangling IDREF, 7 SHARED frontmatter-after-body (surpass decision), 6 mechanical
   (zx-calculus 8 `paragraph` in a list, principia 6 `angle`/`innerdepth` attributes,
   pdfmarginpar, prerex, aguplus `sectional-block`, webquiz `logical-block`), 3 SHARED
   inline-leak (ribbonproofs, sidenotesplus, ryethesis), 2 Fatal-downstream (bibarts,
   chessboard: no root), 2 aomart M4/M5, 2 thuaslogos duplicate ids, 2 M4/M5-tension
   (ltnews, l3news — Perl places them cleanly). Work: 6 mechanical + 2 Fatals; decisions:
   dangling IDREF, frontmatter surpass, inline-leak surpass, thuaslogos, M4/M5 reopen.
3. **Content: measured on s111 with the Unicode-aware auditor over the post-mono HTML (the
   recall of record — a core-XML pass mis-files every bibliography as loss, since the
   bibliography is a post-stage artifact in both engines).** The remaining sub-80 tail on the
   healthy set (88 docs) is, by the two triages of 2026-09-22, SHARED or artifact: locked
   `\maketitle` title-page fields (14, Perl identical; a keyval `\maketitle` for uni-titlepage
   would need a new binding file → ruling), letterheads in page heads (3, no page-head model),
   beamer/grid/calendar furniture (17), code rendered to images (11), non-Latin script absent
   (3: arabi, shipunov/russ — the one genuine kernel lead), external-PDF inclusion (1),
   golden-PDF mismatches and pdftotext artifacts, and five Rust-WINS where Perl crashes. No
   clean Rust-only recall lever remains in the tail. Then the sub-80 tail (100 docs) is themed work —
   bibliography rendering (27), beamer/slide/grid/titlepage furniture (37), non-Latin
   scripts (3), external PDF inclusion — each a real content class, not a metric fix. The
   auditor still needs umlaut/ligature normalization for the 90–95 band to be readable.

**Ruled / documented, do not re-open:** engine-primitive ink (21 docs) out of scope +
harness retry only; section-in-item/figure (14) Perl parity; thuaslogos duplicate `pgfcp*`
ids (`\copy` of a digested SVG box, SHARED, HIGH-risk id rewrite); ndsu `text` in
`listing` (document bug); translation-biblatex-de `\begin[…]{description}` (document bug);
M4/M5 bespoke title-page layouts (ltnews, l3news, lua-tikz3dtools, nostarch, elteiktdk,
sduthesis, aomart); dangling IDREF (27, RULED-KEEP); ~92 no-XML fatals (mostly intended
engine ≠ pdflatex).

**Diagnostics rule in force (2026-09-20/22):** every Warning/Error/Fatal logged AND counted
once; a resource Fatal ends digestion; `\openin` and other probes never reach a diagnostic;
the only sanctioned silence is `IgnoreDiagnosticsScope`. Read logs by counters, match
`(Warning|Error|Fatal):[A-Za-z_]+:` anywhere in the line (WISDOM 85).
