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
## Continuation — state and next steps (2026-09-22, end of session)

Branch `perfect_kernel` (check `git branch --show-current` first). Gemini is out of quota
for a week — delegate to Opus 4.8 `root-causer`/`reviewer` agents, ≤4 at once. Memory cap
is **8 GB** from s110 on (`--max-memory=8192`, `ulimit -v 8912896`, user ruling).

**Measured (axis 2):** s107 270 → s108 248 → s109 241 → s110 229 → **s111 222 invalid (2145/2367,
90.6 %)**; binary 3e039d304c, 0 newly invalid. Landed after s111: 56fx (ERROR-marker hoist, 8 → 0),
56fy (XMText attribute gate, principia 6 → 0), 56ga (bib-resource scanner skips `\jobname`-style
names; gitlog/incgraph post errors → 0), the Unicode-aware recall auditor, the 8 GB cap. Withdrawn:
56fz (Perl's `FontDecodeString` map_max port: faithful, fixes russ_doc's Cyrillic, but Rust's accent
composite emits precomposed char TOKENS that the digester cannot tell from raw bytes — 11 encoding
fixtures red; needs the composite-token marker or an inputenc-table mouth decode, plus a utf8 default
for `INPUT_ENCODING` — LEDGER row). Also landed: 56gb (pgfmath `divide(x, 0)` = x per
tex.web §107/§1240; carbohydrates_en Fatal → valid), 56gc (a space token's character code is 32,
tex.web §289; tikzviolinplots' `Extra \else` gone, Fatal → completes). Open kernel leads from the
loop triage: frankenstein/titles — an engine `\aftergroup` × url-`\Url` push-back × active `^^M`
re-digestion loop (17-line repro, needs a stepping session); stringstrings under raw interpretation —
`\whereisword` returns 0 and `\noblanks[e]` swallows the body (root-causer running). s112 sweep
(with post pass + HTML recall) running; projection ≈ 210 invalid. Landed AFTER s110, not yet swept: 56fq (harness keep-better:
pgfornament-han), 56fr (`xmlns` for spilled-only prefixes: 9 docs), 56fs (PushbackLimit
trio), 56ft (conditional skip stops at the input boundary), 56fu (rootless document = Fatal),
56fw (stranded `{titlepage}` → layout paragraph: chemexec ×2, l2picfaq, ClassicThesis).
Projection for s111 ≈ 218 invalid. Axes 1/3 not re-measured this session (recall 98.3 at
the last reading, `s3_sweep.sh`).

**Where the 229 stand** (clustering `~/data/pk_agents/w58/s109_residual/summary.md`,
triage `~/data/pk_agents/w58/frontmatter69/triage.tsv`, LEDGER triage rows 2026-09-22):
- Ruled: truncations of Fatal/timeout runs 84, dangling IDREF 25, engine-primitive ink 14,
  section-in-item 11.
- SHARED with Perl (a fix is a surpass, each a judgment call): 55 of the 69
  frontmatter-after-body docs (a leaked undefined preamble command's argument opens the body
  before `\maketitle`; Perl fails harder or strands identically), every list-structure and
  math-leak case, 5 of the 7 remaining loop Fatals.
- **Open, mechanically fixable (~15 docs), ranked:**
  1. ERROR-marker hoist (8 docs, surpass, 7 beyond Perl): a leading para whose only
     elements are `<ltx:ERROR>` markers with whitespace-only text outside them is
     content-free for `place_frontmatter` (`node_is_content_free` /
     `subtree_elements_all_invisible`, base_utilities.rs); the marker stays, relocated below
     the frontmatter. Repro `~/data/pk_agents/w58/frontmatter69/repro_erronly_hoist.tex`.
     Needs a DIVERGENCES entry.
  2. Block-class child of a list-only container → auto-opened `item` (qworld 24 errors,
     colorframed, tableaux; surpass): an `AUTO_OPEN_BRIDGES` row `itemize|enumerate|
     description → item → para` for Block-class non-item children, and `insert_block`
     routing its final placement through `find_insertion_point` when the context is a list.
     Repros `~/data/pk_agents/w58/listmath/{m_ii,repro_blockinlist}.tex`.
  3. uspatent/PatentApplicationGuide: `latexml_contrib/src/refstyle_sty.rs` stub shadows
     the on-disk refstyle.sty without `\RS@ifundefined` (kernel gap: `\@ifundefined` vs
     `\let\x\@undefined`). macros2e: `\MakeSpecialShortVerb` undefined.
  4. Rust-only loops: carbohydrates_en (chemfig's stale `\CF@atom@sep` — an undefined CS
     recovered as a macro in a dimen context must yield a once-only "not a register" and
     ADVANCE); the store-verbatim-then-re-input family (frankenstein/titles is the clean
     driver, Perl clean; tikzfxgraph, xsim). Repros `~/data/pk_agents/w58/pushback/`.
  5. tikzviolinplots `Error:unexpected:\else Extra \else already saw \else` (a Rust
     conditional bug, line 182); xytree-doc-en recursion through our `\lx@xy@svg` binding.
- Perf ceiling (>180 s at 8 GB): lie-hasse (E8 loop), pgf-interference-de/-en (360 s =
  luatex retry ×2), pgf-PeriodicTableManual, tcolorbox, wheelchart (now runs to the wall
  instead of the memory fuse). Open follow-up: pass 2 clones `node_fonts` per segment.
- Ruling tension to surface: l3news/ltnews/lua-tikz3dtools sit on the M4/M5 do-not-reopen
  list, but Perl places their frontmatter cleanly — mechanically fixable, not fidelity-bound.

**Method notes that saved time today** (memory `wisdom_perfect_kernel_batch56fl_fp_recipes`):
read a witness's `(Loading …)` lines before assuming the raw-class path; count per-yield
(`node_boxes`, C-live) on the release binary instead of dhat; never lower the cap to make a
doc fit; every scan-until-delimiter loop needs Perl's run-out branch; stage by line
(`scratchpad/stage_lines.py`) when batches share a file; a rewrite of pushed backup commits
is a lease push after a backup ref.

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
