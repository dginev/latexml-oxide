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
## Continuation — state and next steps (2026-09-22)

Branch `perfect_kernel` (check `git branch --show-current` first — the tree was found on
`gemini/pk-helpers-11` once). Batches 56fe–56fj landed 2026-09-22 (LEDGER rows); Gemini is
out of quota for a week — delegate to Opus 4.8 `root-causer`/`reviewer` agents, ≤4 at once.

**Measured (axis 2):** s107 270 → **s108 248 invalid** (of 2,371; release at `b4968da63f`,
JOBS=4, 180 s timeout — s107 used 300 s, so 3 slow docs now read as timeouts): 21 newly
valid, 1 newly invalid (pgfornament-han-doc, `\setsansfont` font leak, no lualatex oracle
so no retry), 0 real code regressions after 56fi (the `Fatal:Mouth:MissingFile` cluster was
56fc surfacing a caught `\openin` probe — fixed). Diagnostics integrity: 0 silent
Errors/Warnings, 0 fatal counters without a line, 1 caught panic (listings gobble, fixed
56fj). Since s108, verified per witness with the RESOLVED jing schema (`~/data/pk_agents/
rngdir_56fg`, built like `validate.sh` — a raw copy of the `.rng` files makes jing throw
and `grep -c error:` read 0): gaceta 2→0, oup 9→0, emoji-doc 4→0, asternote 3→0,
egpeirce 1→0, hitszbeamer 4→0, chemobabel-en 3→0, smf-edoc/fdoc →0. Projection for s109
≈ 238 invalid.

**Load-bearing next steps, in order:**

1. **Memory lever B** (pgf-spectra LSE 6.36 GB / 108 s, pgf-PeriodicTable, tcolorbox,
   wheelchart): single-pass eager→streaming continuation — design in the 2026-09-22
   root-causer report (LEDGER 56fe row): arm `spill_watermark_bytes` (RAM/8) at
   `--max-memory=0`, accumulate in the eager loop, on a `request_fragment_yield` seam with
   RSS over the watermark hand the boxes to the streaming pass-1 body (`stream_setup`/
   `stream_pass1`/`stream_finish` factored out of `convert_streaming`) and continue from the
   same gullet position; keep `StreamingRestart` as the no-seam fallback. Guards: ~40 tiny
   `\pgfpicture`s under a low `LATEXML_RSS_CAP_BYTES` → all `svg:path` present,
   `fragment_yield_count()>0`, a new `digest_setup_count()==1`; control with a huge cap →
   byte-identical eager output. Falsification: LSE peak < 4.5 GB, wall < 180 s. Lever A
   (isEmpty field) measured −0.27 GB only — the DOM, not the Rust heap, dominates.
   Secondary target after B: the streaming pass creep 271 → 2361 MB (suspect
   `Document.node_fonts: HashMap<u64, Font>` by-value, never swept) — dhat the `--streaming`
   run.
2. **Raw `\author` surpass (56fk)** — ruling "surpass now", mechanism corrected by the
   2026-09-22 root-causer: the class body must NOT run (private accumulators, locked
   `\maketitle` → authors lost); instead the locked kernel/binding `\author` ABSORBS a
   trailing `[…]`/`{…}` (`\lx@author@trailing`: `orcid=` → `ltx:contact role=orcid`, a
   trailing group → `role=affiliation`) and, when `\author:redefined`, appends via
   `\lx@splitting{\lx@add@author}` instead of dequeue-replace. Patch the four twins together
   (`sect05.rs`, `inst_support_sty.rs:33`, `sv_support_sty.rs:25`, `llncs_cls.rs:48`).
   Witnesses els-cas cas-sc/cas-dc (+3 creators with orcid), cnbwp (1 → 3 creators).
   Repros `~/data/pk_agents/w57/author_redef/`.
3. **Open RUST-ONLY singletons** (one batch each): fancybox `Bitemize` in the doc's
   re-`\input` example env (list binding in `fancybox_sty.rs`); plarray `\verb` inside
   `\footnote` running past its delimiter; biblatex-ext / robust-externalize residual list
   float-out (unreduced — `LXML_TRACE_INSERT_BLOCK=1` on the real docs); pgfornament-han
   `\setsansfont` leak (no oracle → consider the font-command signal without the oracle
   gate). Then s109.

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
