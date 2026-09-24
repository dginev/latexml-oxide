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
## Continuation — state and next steps (2026-09-23)

Branch `perfect_kernel` (check `git branch --show-current` first). Delegate read-only work
to Opus 4.8 `root-causer`/`reviewer`/`log-scanner` agents (≤4 at once). Memory cap 8 GB
(`--max-memory=8192`, `ulimit -v 8912896`). Sweeps: `~/data/pk_agents/w59/main/sweep113_launch.sh`
is the current recipe (vendor TL + `LATEXML_DUMP_DIR` vendor dumps, JOBS=16, 180 s, cores 0-63;
validate → post mono → HTML recall in one chain).

**Working rules added 2026-09-23 (user):** every fix is catalogued as a minimal `.tex` repro in
`tools/perfect_kernel/repros/<mechanism>/` AND a red/green guard (a generalizing refactor pass is
planned at the end); a schema win counts only if content is preserved — run
`tools/perfect_kernel/content_diff.py old.xml new.xml` on every witness and
`pdf_recall.py` (PDF from the INTENDED engine; `repros.sh <topic> --recall` per repro).

**Measured:** s110 229 → s111 222 → s112 211 → **s113 148 invalid (2219/2367, 93.7 %)**, 63 newly
valid, 0 newly invalid, 0 tally regressions, recall of record unchanged (median 98.6, mean 93.72,
no doc down > 0.5). Landed this session: 56gd (auto-opened `item` for a block / same-kind list in
a list, #261; refstyle loads raw; newverbs `\MakeSpecialShortVerb`), 56ge (`\global\read`
bookkeeping stays local — csvsimple-legacy's ~1,600 `\par`-corrupted attributes), **56gf (the
frontmatter always opens the document — preamble residue marked at `\begin{document}`, visible
pre-flush content follows the head-placed frontmatter; user-approved surpass #262; 55 of the 59
title-after-body manuals valid)**, 56gg (beamer in-frame `\frame` = kernel box frame; titlepage
unwind re-homes children #263; heading page breaks precede the section #264), stricter diagnostic
counting in guards/tools, and the content tooling.

**Where the 148 stand** (`~/data/pk_agents/w59/main/jing113/`):
- 91 no-XML Fatal/timeout runs — 89 fail in their own intended engine (oracle not clean), the
  other 2 (bibarts, chessboard_and_beamer) are ruled; `~/data/pk_agents/w59/noxml/`.
- 24 dangling IDREF (RULED KEEP); 2 duplicate ids (thuaslogos, ruled).
- 13 sectioning inside an item/figure (`paragraph`/`subsubsection`/`section`; RULED LEAVE, #189 —
  note #189 also inserts WITHOUT a diagnostic, so these docs are silently invalid).
- 2 block inside `quote` (aguplus `sectional-block`, webquiz `logical-block`) — LANDED 56gw
  (user ruling: `quote_model` widened, #271).
- SHARED / document bugs, one each: item-in-math ×2, bibliography-in-box ×2 (biblatex-ext,
  biblatex-cv), note/indexmark/glossarydefinition in XMText (inline-leak ×3 — LANDED 56gx, user
  ruling: `Meta.class` floats out of math text, #272), swfigure, msgguide
  TOC-in-date, pgf-spectra late abstract, thalie/ndsu text-in-listing, brochure block-in-text,
  stanli figure-in-titlepage, philex anchor.
The schema axis is at its ruled ceiling; the four rulings (section-in-item LEAVE, dangling IDREF
KEEP, quote model and inline-leak LANDED 56gw/56gx) are all decided, so the remaining levers are the
mechanical singletons and the content/semantic axes below.

**Open leads (ranked; updated after sweep 118 and batches 56gw-56hd):**
1. **Hidden macro-delimiter misses (56gn, held back).** A `\def` parameter text's leading
   delimiter that is missing at a call is silent here and the macro expands anyway (Perl reports
   it; TeX reports and IGNORES the call, tex.web §397-398) — the frankenstein/titles loop. The
   strict check (`~/data/pk_agents/w59/main/56gn_macro_delimiter.patch` + `56gn_guard.rs` +
   `56gn_NOTES.md`) surfaced ~30 latent clusters in 29 manuals; `\??? Match:?` is l3msg's
   expandable-error sentinel — each a REAL expl3 error the engine swallowed. Fixed so far: 56gt
   (etoolbox `\patchcmd` `##` → pgfornament-han 501 → 0, biblatex-gost 88 → 0), 56gv (`\typeout`
   partial), 56gp (notebeamer), 56go (`\everypar`), 56gy (one-space delimiter: the whole xint
   `\XINT_zapsp_b` cluster, 9 manuals), 56hb (`\protected\relax\def`: catoptions 70 → 1,
   keyval2e-examples 101 → 2). Strict re-measure after 56gy (`~/data/pk_agents/w59/main/strict3/`,
   binary `latexml_oxide.strict3`): euclideangeometry 101, leporello 74 + jsonparse 60 (native
   pgfkeys `Expand!`), titles 101 (with 56gn the loop becomes 101 `\aftergroup` Match errors;
   pdflatex stops at 9, so a re-trigger remains), bfh-ci 11, guitar 10; mercatormap shell-escape
   excluded; greektonoi/chinesechess/bxjaholiday SHARED. catoptions residual: 1 option-stack-limit
   error per load. Root-caused 2026-09-23 (`~/data/pk_agents/w63/`): leporello 74 + jsonparse 60 are
   NOT pgfkeys. `\NewTCBListing` substituted an absorbed `s` argument as empty, so `\IfBooleanT{}` hit
   l3's `if-boolean` error (fixed in 56hl). titles is gone with 56hj. euclideangeometry is curve2e's own
   `\MV@c` error, which pdflatex reports 34 times (SHARED). guitar is SHARED with Perl. bfh-ci SciPoster
   is SHARED at the origin, but we keep less of the poster than Perl (l3 `\dim` registers in
   pgfmath; an open content target). **Landing hazard:** with 56gn, euclideangeometry's
   `\???` misses reach the 100-error cap and the Fatal empties a complete 1.4 MB document. Before
   landing, keep the l3 sentinel's SHARED misses from turning good output into a Fatal.
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
   31 part-short manuals). Residuals: KOMA `\addpart` (`\@addpart`) is still a paragraph; ctex's
   localized part label becomes "Part I".
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
   engines too (277 s, > 900 s).
6. Rulings — all decided: dangling IDREF (KEEP, 24 docs), sectioning inside an item/figure
   (LEAVE, #189, 13 docs), the `quote` content model (LANDED 56gw, #271: webquiz, aguplus valid)
   and inline-leak (LANDED 56gx, #272: ribbonproofs, sidenotesplus, ryethesis valid; equation-level
   footnotes now render in HTML). Measured on sweep #118: 2226/2368 (s117: 2221).

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
