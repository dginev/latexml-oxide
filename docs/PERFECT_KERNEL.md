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
## Continuation — state and next steps (2026-09-20)

Branch `perfect_kernel`, HEAD `915ac60f79` (pushed). Suite 3007/3007, clippy/rustdoc/fmt clean.
Per-batch detail: [`perfect_kernel/LEDGER.md`](perfect_kernel/LEDGER.md) rows 56es…56fd;
mechanism entries [`parity/OXIDIZED_DESIGN_DIVERGENCES.md`](parity/OXIDIZED_DESIGN_DIVERGENCES.md)
#245–#251; method notes [`parity/WISDOM.md`](parity/WISDOM.md) 85.

**Measured (axis 2, schema validity):** s99 416 → s106 303 → **s107 270 invalid** (of 2,371;
`~/data/perfect_kernel_s107/validate_verdicts.tsv`). Landed since s107 without a sweep: 56et.1
(8 marker-leak docs), 56ez (9 singletons), 56fb (8 section-in-box docs), 56fd (4 jlreq docs no
longer Fatal), 56fc (a resource Fatal now ends digestion — Perl `hardYankProcessing` — so
Fatal documents get SHORTER, by design). Projection ≈247 invalid; **unmeasured until s108**.

**Load-bearing next steps, in order:**

1. **s108 sweep** (measurement; ~1 h at JOBS=8, alone). `cargo build --release` into
   `~/data/pk_target`; `tools/perfect_kernel/run_doc.sh` over `~/data/perfect_kernel/corpus.tsv`
   with `LATEXML_DUMP_DIR=~/data/pk_agents/vendor_dumps/resources/dumps`,
   `TL_ROOT=/usr/local/texlive/2025`, output `~/data/perfect_kernel_s108/`; validate with jing
   (`~/data/pk_agents/rngdir_56eo/LaTeXML.rng`) and run the counters-vs-lines cross-check
   (`~/data/pk_agents/w56/s107_diag_audit/{audit,agg}.py`) as the regression oracle. Judge
   56fc by S3 recall on the Fatal documents (they must end at the Fatal, not lose pre-Fatal
   content elsewhere) and by `silent_undefined = 0` everywhere.
2. **Memory lever for the pgf family** (pgf-spectra LSE, pgf-PeriodicTable, tcolorbox,
   wheelchart; task #23). Root cause measured (dhat, `~/data/pk_agents/w56/dhat_lse/`): LSE
   native release = 107 s, 6.63 GB, 0 errors — a memory-fuse casualty; 93 % of the peak is
   the eager whole-document box tree (`Rc<DigestedData>` + per-box `SymHashMap` property maps +
   `Rc<Font>`) pinned by `Document.node_boxes` until serialization; streaming never engages
   (source-byte projection misses `.sty`-internal `\input`, restart watermark `None` at
   `--max-memory=0`). SHARED shape with Perl (`@LaTeXML::LIST` + `node_boxes`; Perl has no
   streaming). Steps: (a) `isEmpty` as a `Tbox` bool field instead of a one-entry map
   (`tex_box.rs:378/445`; −16 % peak, zero semantics); (b) arm the RAM/8 watermark at
   `--max-memory=0` and make the eager→streaming transition an in-place continuation at the
   existing `\pgfsys@endpicture` yield seam (`core_interface.rs` eager loop → streaming loop),
   no from-scratch restart. Falsification: LSE peak 6.63 GB → < 4.5 GB with wall < 180 s.
   Dead ends (measured): 56ex node_boxes svg skip, 56fa pgfsys marker flip, the `manypath3k`
   synthetic (one picture; the cost is ~194 pictures outliving their close).
3. **The frontmatter-order class (77 docs, largest axis-2 residual).** Table:
   `~/data/pk_agents/w56/frontmatter_ink/leading_ink_s107.tsv`. ~55 docs = preamble ink from
   undefined pTeX/XeTeX/LuaTeX primitives in manuals whose intended engine is not pdflatex
   (`\kanjiskip`, `\xspcode`, `\mubytein`, `\setmonofont`, `\fontid`, …) — now honestly
   `Error:undefined` + `<ltx:ERROR class="undefined">`; **needs a ruling**: out of scope by the
   intended-engine gate, or define those registers in the K12/luatex profiles. ~20 docs = raw
   classes whose `\author[…]`/`\title` shapes leak option text (aomart, els-cas, oup, cnbwp,
   hebdomon, elteiktdk, sduthesis) — kernel-binding work within the no-new-`*_cls.rs` rule.

**Smaller residual tiers (each ≤ 14 docs):** sectioning unit escaping an `item`/`figure`
(14 docs; memoir/memman 10, keytheorems 5 — a `\section` inside `\item` is Perl's shape too,
ruling pending: "sectioning A", recommend leave); `item` not allowed (5: biblatex-ext,
fancybox-doc, plarray, robust-externalize, biblatex-de); `text` in `listing` (1);
`XMTok` under `p` (2: egpeirce, hitszbeamer); `para` in `figure` (2: chemobabel); duplicate
`pgfcp*` clip ids (2: thuaslogos); dangling IDREF (27, RULED-KEEP); ~94 no-XML fatals
(mostly intended-engine ≠ pdflatex, e.g. jlreq/luatexja → `[luatex]` retry).

**Rulings pending (user):** the 77-doc class above; sectioning A (12 docs, recommend leave);
C subparagraph asymmetry (2 docs); a native xstring binding for lie-hasse/circularglyphs
despite the no-new-`*_sty.rs` rule (lie-hasse is O(L²) xstring, not a bug).

**Diagnostics rule now in force (user directive 2026-09-20):** every Warning/Error/Fatal is
logged AND counted once; a Fatal ends digestion; the only sanctioned silence is
`IgnoreDiagnosticsScope` (Perl `IGNORE_ERRORS`). Read logs by counters and match
`(Warning|Error|Fatal):[A-Za-z_]+:` anywhere in the line (WISDOM 85).
