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

**Measured (axis 2):** s107 270 → s108 248 → **s109 241 invalid** (2127/2368 valid, binary fb9db2b0dc = 56fl–56fp; 16 newly valid incl. pgf-spectra LSE, 8 newly invalid: six TikZ manuals with an unbound `xlink` prefix after a streaming spill — batch 56fr — and two PushbackLimit Fatals with memman, prime suspect 56fo; 56fq harness fix landed after the sweep). Earlier: s107 270 → s108 248 invalid (of 2,371; release at `b4968da63f`,
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

1. **Streaming-pass creep — LANDED as 56fm** (LEDGER row): the driver was `node_boxes`, not
   `node_fonts` — inline centered pictures close under `ltx:para > ltx:p > ltx:text >
   ltx:picture`, and `spill_prose_free_children`'s prose test matched the `ltx:p` itself, so
   those paragraphs were kept whole and their digested boxes pinned (163,636 entries, ~4.4 GB
   on LSE). A text-less `ltx:p` now spills whole. LSE at the sweep cap: `--streaming` 107 s /
   0.40 GB / all 112,295 paths (s108: Fatal at 165 s); default adaptive 106 s / 1.73 GB.
   Open follow-up: pass 2 clones `node_fonts` per segment (`core_interface.rs`), a real but
   pass-2-only quadratic. Never measure at `--max-memory=0` on the 246 GB host (RAM/8 watermark).
   **Memory cap = 8 GB from s110 on** (`--max-memory=8192`, `ulimit -v 8912896`; user ruling
   2026-09-22: test against production-grade hardware, 8 GB is allowable). s108/s109 ran at 6144.
2. **Raw `\author` surpass — LANDED as 56fl** (853f7b2434, DIVERGENCES #253): the locked
   `\author` absorbs a class's trailing `[keyval]`/`{affiliation}` and appends creators only
   when `\author:redefined`; the els-cas witnesses turned out to load the contrib binding
   `cas_dc_cls.rs`, which now carries cas-common's `\author[marks]{name}[keyvals]`.
   cnbwp 3 → 0, cas-sc/cas-dc 8 → 0 schema errors. Check a witness's `(Loading …)` log lines
   before assuming the raw-class path.
3. **Open RUST-ONLY singletons — three of four LANDED** (LEDGER rows): plarray `\verb` in
   `\footnote` → 56fn (Perl `readUntil` run-out unreads; schema 1 → 0); fancybox `\Benumerate`
   → 56fo (`\usecounter` counter-only per latex.ltx:16048, list starts at `\@trivlist`;
   DIVERGENCES #254; 3 → 0); biblatex-ext list float-out → 56fp (the `insert_block` hoist
   never leaves a list; DIVERGENCES #255; 4 → 1 SHARED). robust-externalize is shell-escape
   (`shell_escape_excluded.tsv:24`), OUT. Remaining: pgfornament-han `\setsansfont` leak (no
   lualatex oracle → consider the font-command signal without the oracle gate). Then **s109**
   (release binary built at HEAD in `~/data/pk_target_s109/release/`; rebuild after these land).

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
