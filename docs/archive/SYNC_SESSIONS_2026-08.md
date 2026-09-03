# SYNC_STATUS session logs — lifted 2026-08-14

Completed "Landed this session" entries lifted out of the live
`../SYNC_STATUS.md` "Current status" changelog (rule 2: day-by-day logs
live in git and here). Conclusions only — defect / cause / fix / guard.
Covers the **2026-07-09 … 2026-07-27** window. The 2026-07-29/07-30 and
2026-08-02 entries stayed in the live worklist as the freshest status.

---

- **2026-07-27 — the `unexpected:fi` fatal cluster: `\meaning` of a
  `\chardef` token returned the internal class name.** GENUINE-RUST-ONLY,
  **18 papers, one cause.** Largest unclassified first-error cluster in the 186
  `Fatal:TooManyErrors` papers of sandbox-arxiv-2605+2606:
  `2605.{03971,04451,09005,15128,16720,29156,29341}`
  `2606.{06712,07410,11290,11722,13769,14502,15753,18180,24256,26947}` — all 17
  ship `bxcoloremoji.sty`. Rust `\meaning` had **no chardef arm**: CharDef and
  plain Register are both `Stored::Register` (discriminated by `register_type`),
  so a chardef fell through a catch-all and rendered as the literal string
  `Register`. The dropped `"` is load-bearing — `bxcoloremoji.sty` L1373 recovers
  the value with the delimited `\def\bxce@do#1"#2\relax`, so with no `"` the
  argument scan runs away and swallows the `\fi\fi` of the enclosing
  `\AtEndOfPackage{…\@whilenum…}` loop (L1366-1386). Those `\fi`s then executed
  against an empty if-stack **from a macro body — hence the `at Anonymous String`
  locator**, which is the tell that separates this from a source-level `\fi`.
  Fixed faithfully per `TeX_Debugging.pool.ltxml` L166-168 (`\char` + `"` +
  decimal). Measured, release + dumps, `--preload=ar5iv.sty`: **1002 -> 1-10
  errors on all 17, zero `fi`, no fatal**; same-host Perl was 1-102 with no `fi`,
  so post-fix Rust is at or below Perl on every witness. (Both caps must be named
  or the deltas mislead: our 1002 is the tikz-raised 1000 cap, and Perl's lone
  102 — 2606.11290 — is Perl's own `MAX_ERRORS`=100, so that one Perl total is
  >=100 and unknown.) PR #426 fixed none of them (all 17 still reproduced at
  `fc56b4d081`, which *is* #426). Guard `meaning_chardef`
  (`latexml_oxide/tests/expansion/`). Perl's own two deviations from `tex.web`
  L22897-22899 here (decimal not hex; `\char` for `\mathchardef`) are
  deliberately inherited — recorded as `KNOWN_PERL_ERRORS` #65, which also warns
  why Rust's populated `mathglyph` must NOT be used to revive the `\mathchar`
  arm.
  **Method note — first-error bucketing UNDER-counted this cluster.** Exactly 18
  papers in 2605+2606 (of 60,513) ship `bxcoloremoji.sty`; 17 surfaced as
  `unexpected:fi`, and the 18th, **2605.14271**, bucketed elsewhere because its
  *first* error was `undefined:\SetTitleBoxVerticalShift` — yet it carried the
  same two `fi` errors, was a real `Status:conversion:3`, and went **1002 -> 12**
  (Perl 42) on the same fix. So a first-error histogram is a lower bound on a
  cause's reach, not a census: confirm membership by the mechanism (here, "ships
  the package"), then re-measure.
  Second method note, the counterpart trap: a *heuristic* main-file pick
  (largest `.tex` containing `\documentclass`) chose `macro.tex` for that paper
  and manufactured a plausible-but-wrong 103 -> 1. Always take the main file
  from cortex's own `Processing content` line.
  Third: this worktree had **no dumps**, so the first sweeps ran in DEGRADED
  raw-load mode. The tell was an identical error count (1003) across 17
  *different* papers — a same-number-everywhere result is an environment
  artifact until proven otherwise. `tools/make_formats.sh`, then re-measure.

- **2026-07-27 (later still) — spconf.sty's `keywords` and `\twoauthors` were
  unbound.** `Error:undefined:{keywords}` was the **single largest `undefined`
  what** in the sandbox corpora — **94 tasks in sandbox-arxiv-2605, 49 in
  sandbox-arxiv-2606**; 142 of those 143 papers ship a byte-identical
  `spconf.sty`. The block is a bare `\def\keywords`/`\def\endkeywords` pair
  (L211-214), not a `\newenvironment`, and `latexml_contrib/src/spconf_sty.rs`
  covered neither. Bound as `\lx@begin@keywords[name={…:~}]` / `\lx@end@keywords`
  — verbatim what Perl does for the same markup in `IEEEtran.cls.ltxml` L147-148
  (spconf says the section was "adapted from IEEEtrans"; IEEEtran.cls L5286-5288
  typesets it identically). Raw-loaded spconf gives Perl inline bold body text
  and **zero creators in either configuration** (`\maketitle` is locked, so
  spconf's own one never emits `\@name`) — **divergence #82**. Sibling gap
  `\twoauthors` (3 papers) routed to the same author machinery; braced
  `\keywords{a,b}` guarded with Perl's `\keywords@onearg` brace-peek (without it
  the until-scan runs to EOF and swallows the body). Witnesses, bare and
  `--preload=ar5iv.sty` alike: 2605.00480 1→0, 2605.00698 1→0, 2605.00721 1→0,
  2605.01187 2→1 (residual `undefined:\bstctlcite`), 2605.05692 2→0, 2605.18923
  1→0, 2605.26747 2→0. Guards
  `06_cluster_frontmatter::{frontmatter_spconf_keywords,
  frontmatter_spconf_keywords_braced, frontmatter_spconf_twoauthors}`.
- **2026-07-27 — `\usepackage{xparse}` silently destroyed the `\c` cedilla
  accent (issue 421).** GENUINE-RUST-ONLY, **0 errors** both before and after —
  a wrong glyph, not a diagnostic, on any document loading `xparse`/`expl3`.
  `expl3_sty.rs` emitted the `\c_sys_*` constants through `raw_tex`, which
  tokenizes with the AMBIENT catcodes; under the document regime (`_` = SUB)
  `\edef\c_sys_shell_escape_int{0}` parsed as `\edef\c` + parameter text
  `_sys_shell_escape_int`, so `\meaning\c` became
  `macro:_sys_shell_escape_int->0` and `Fran\c cois` rendered **"Fran0cois"**
  (Perl 0.8.8, same host: "François"). **The block was DELETED, not
  re-tokenized** — measurement killed both of its premises: the constants are
  already defined at package-load time with live values, and the block had never
  run, so repairing its tokenization would have overwritten those with frozen
  dummies + a hardcoded year. Perl's `expl3.sty.ltxml` has no such block. The
  surviving raw expl3 chunk now goes through `with_expl_catcodes`
  (save/restore, error path included). Witness 2605.11579: `Fran0cois` →
  `François`, 0 errors, 36 bibitems, unchanged otherwise; named witnesses
  2406.14142 / 2002.07146 byte-identical. Guard
  `expl3_load_does_not_clobber_cedilla_accent`. Detail + the workspace-wide
  audit: [`EXPL3_CATCODE_GAP_2026-06-08.md`](../parity/diagnostics/EXPL3_CATCODE_GAP_2026-06-08.md)
  (third member of that family), method in **WISDOM #73**.

- **2026-07-27 — the LaTeX kernel autoloads on ANY undefined kernel
  CS, not just a curated trigger list.** A document may legitimately use a
  kernel command before `\documentclass` (real LaTeX has no "before the
  kernel"), but LaTeXML only loads `LaTeX.pool` on a *trigger* CS — Perl
  `TeX.pool.ltxml` L33-56. Anything off that list was `<ltx:ERROR/>`, and for
  the standard "use this class if installed" idiom
  `\IfFileExists{X.cls}{\documentclass{X}}{\documentclass{Y}}` the collapsed
  conditional selects the **rejected** branch's class: witnesses 2605.25877 and
  2606.06905 went **101 errors + `Fatal:TooManyErrors`, no class → 0 errors**.
  Shared with Perl (**KNOWN_PERL_ERRORS #64**) — "at parity" is not "not a bug".
  Fixed generally, not by growing the list: `latexml_engine/src/latex_kernel.rs`
  registers a hook (`latexml_core::binding::kernel_autoload`) consulted at the
  two undefined-CS paths *before* the error, which loads the format and retries
  when the **ambient kernel dump** names the CS. The dump is the oracle, not
  `latex.ltx`, because it answers "what will be defined AFTER the pool loads" —
  it is generated by our current code inside a pinned TL-year container.
  Backward-compatible only, by ruling: a CS newer than the ambient dump year is
  out of scope, so no forward-compat seam exists. Fires at most once per
  session, never under `LATEXML_INI_MODE` (dump-build) or
  `SUPPRESS_UNDEFINED_ERRORS`, and **not at all on the degraded no-dump branch**
  of `LoadFormat('latex')` (no key set → inert, behaviour bit-identical to
  before). Retired the two Rust-only trigger accretions `\UseRawInputEncoding`
  (2403.19280) and `\DocumentMetadata` (2305.08034), both re-verified unchanged;
  the faithful Perl L33-56 port stays. Dump-neutral: regenerated `plain`/`latex`
  2025 dumps are content-identical (record counts and section partition equal,
  sorted-set diff = only the `texsys.aux_contents` timestamp, which also differs
  between two runs of the same binary), and conversions against the pre- and
  post-change dumps are byte-identical. No lateral drift: 60 plain-TeX/AmSTeX-era
  papers and 160 arXiv-2605 papers all unchanged. Guards
  `preclass_iffileexists_test`, `preclass_kernel_cs_test`,
  `nodump_leaves_pre_documentclass_kernel_cs_undefined`.

- **2026-07-27 — a `.bib` field's `^` is data too, and `mathscinet.sty` gets a
  binding.** Two changes, one PR. (a) `^` joins `_` in treatment 2 of
  **OXIDIZED_DESIGN #74** — verified symmetric with `_` rather than assumed
  (both are TeX scripting characters, both inert to `bibtex(1)`, both raise
  "Script … can only appear in math mode" outside math; `note = {q _ r ^ s}`
  now renders literally, zero errors). It needs its OWN escaper arm, because
  `\^` is the circumflex **accent** — the generic `\` + character arm would
  render `^o` as "ô", a wrong glyph rather than a diagnostic — so
  `BIB_DATA_CARET` emits `\textasciicircum{}`. Knock-on:
  `105_bib_field_digest_once` lost its last non-self-healing probe and moved to
  `\hline` (→ `\noalign`, a context error).
  (b) **`mathscinet.sty`** (AMS, v1.05, in the amsrefs bundle) is now bound at
  `latexml_package/src/package/mathscinet_sty.rs` — Perl has `amsrefs.sty.ltxml`
  but no `mathscinet.sty.ltxml`, so Rust-only, though ported from the real
  `.sty` (mappings from its T1 branches: `\Dbar`→`\DJ`, `\dbar`→`\dj`,
  `\cprime`→`\tprime`, `\polhk`→`\k`). **Nothing auto-loads it**, and that is
  the decision: witness 2605.11579 never loads the package and uses
  `\bibliographystyle{alpha}`, whose `.bst` has zero `Dbar`, so its
  `undefined:\Dbar` is **PARITY** with the author's own pdflatex build. `\Dbar`
  is package-only for a second measured reason: 4 of 4,000 arXiv-2605 papers
  define it with `\newcommand`, which an always-on definition silently shadows
  (LaTeXML keeps the OLD meaning, no diagnostic). The `\cprime` family moved out
  of `latex_constructs.rs` (Perl-parity file) into
  `latex_constructs_rust_only.rs` §5 with its witnesses 2508.13753 / 2508.20226
  / 2509.07628, all three of which load the package by name, refuting the old
  `cyracc.def` justification. Divergence **#78**. Guards
  `bib_mathscinet_package_supplies_its_transliteration_glyphs`,
  `bib_mathscinet_macro_yields_to_the_authors_own_definition`,
  `escape_specials_caret_is_textasciicircum_not_an_accent`.
  *Two claims in this bullet were overturned within the day — see the next one.
  The `\cprime` stub is **deleted**, not merely relocated; and 2605.11579 no
  longer emits `undefined:\Dbar` (the reasoning stands, the witness went silent
  because its `\Dbar` entry is uncited).*

- **2026-07-27 (later) — the `.bib`-as-DATA family closed, and a `.bib` library
  is filtered to its CITED entries.** Five PRs, in dependency order.
  **#413** — `TeXString`, so a flattened `Tokens` cannot reach the tokenizer
  (543→536 `to_string()` sites, 6 weld-risk families → 0; WISDOM **#71**).
  **#416 — divergence #80, the big one:** Perl `Pre/BibTeX.pm::toTeX` L110-122
  emits `\ProcessBibTeXEntry` for *every* entry, which was free under the old
  string parser and is a full expand/digest/construct cycle since #396.
  `anthology.bib` = 80,576 ACL entries for 9 cited; witness **2605.07796** went
  112 s / 4.8 GB / memory-budget-tripped / **0 bibentries** / fleet-killed →
  **10 s / 9 bibentries / 0 errors**. Same shape in **59 of the 69** 2605/2606
  `never_completed_with_retries` papers. Filtering is *more* faithful —
  `bibtex(1)` has always read the `.aux`'s `\citation` records — and is closed
  over `crossref` and inner `\cite`; every entry stays registered; `None` (=
  digest all) covers `\nocite{*}` and a missing `BIBLABEL` record.
  **#417** — the `.bib` `@preamble` already executes (Perl `toTeX` L118-122 →
  `pre_bibtex::to_tex`); guard + docs only, no behaviour change.
  **#418 — divergence #79:** an unmatched `$` in a field is currency, not a math
  shift; 2605.00166 went 103 errors + Fatal → 0, and same-host Perl cascades
  identically.
  **#419** — the always-on `\cprime` stub is **deleted**; the family is
  `mathscinet.sty` vocabulary and lives only in the binding. Its justification
  (four papers regaining `undefined:\cprime`) collapsed because #416 removed the
  trigger: three of the four only regressed on **uncited** entries. Current main,
  `--includestyles`: 2605.00173/.00186/.00190 **0 errors**, 2605.11579 **0**
  (its own `@preamble` covers 17 uses), 2605.00305 **1** — the only cost, and
  PARITY (it cites `MR710121`, loads no `mathscinet`/`amsrefs`, and `plain.bst`
  has zero `cprime`, so pdflatex fails too).
  **Standing consequence — re-measure any bibliography error count recorded
  before 2026-07-27.** An error raised only by an uncited entry now disappears
  without the macro becoming available; that is also what removed 2605.11579's
  `undefined:\Dbar` (its `KacNilpotentorbits` entry, `biblo.bib` L2059, is
  uncited). Guards: `filter_digests_only_the_cited_entries` + 7 siblings
  (`pre_bibtex.rs`), `bib_preamble_defines_macros_for_the_whole_bibliography`,
  `bib_unmatched_dollar_does_not_leak_math` + 5
  `escape_specials_*` unit tests.

- **2026-07-26 — undefined CSes from packages with no binding: `silence.sty`,
  bundled `arxiv.sty`/`PRIMEarxiv.sty`.** Long-standing gaps, **not** a
  regression: Perl 0.8.8 has no binding for either and reproduces the identical
  `undefined:\WarningFilter` / `undefined:\keywords` on the same witnesses
  today. They surface only where the raw `.sty` is not read (bare mode; or a
  bundled class whose `\RequirePackage{silence}` never reaches a raw load —
  2504.08779). The four witnesses' current `no_problem → error` flip in
  sandbox-arxiv-2605 is a *different* cause (`unexpected:&`, `undefined:\sqrtn`,
  bibliography `malformed:ltx:bibitem`/`ltx:bibentry`). New contrib bindings,
  two deliberately different shapes — silence unconditional (the raw file's
  `\ErrorsOff` rebinding of `\PackageError`/`\GenericError` *suppresses* real
  LaTeXML diagnostics: measured Perl 0 vs Rust 1 on a probe), the two bundled
  arxiv styles gated on `INCLUDE_STYLES` so the paper's own file still wins in
  ar5iv mode (all four witnesses byte-identical there, before vs after). Bare:
  1→0, 4→1, 1→0, 1→0. Divergence #77. Guards `00_contrib::{silence_filters,
  arxiv_keywords, primearxiv_keywords}_test`, `106_arxiv_sty_defers_to_bundled`,
  `107_silence_keeps_diagnostics`.
- **2026-07-26 (later still) — a bare `&` in a `.bib` field is data (OXIDIZED_DESIGN #74).**
  Seven 2605 witnesses carried `Error:unexpected:&` from `publisher` / `journal`
  / `booktitle` / `author` / `copyright` ("Taylor & Francis"). Not a Rust-only
  defect: same-host `latexmlc` raised the identical per-`&` count on all six
  re-measured witnesses, and bibtex 0.99d + pdflatex agree (the `&` reaches the
  `.bbl` under `plain` and `abbrvnat`; pdflatex stops with "Misplaced alignment
  tab character &" and prints "Taylor Francis"). **A `.bib` field's content is
  DATA** — authorized surpass-Perl and surpass-pdflatex, since LaTeXML reads
  `.bib` directly and decides what reaches the tokenizer.
  Landed inside the consolidated **OXIDIZED_DESIGN #74**, which covers `%`, `&`,
  `#` and `_` under one two-treatment design — be `bibtex` (the per-entry Mouth
  and `mouth::tokenize_bib_literal`, via `Mouth::with_bib_data_literals`), then
  be `pdflatex` on the `.bbl` you just synthesized
  (`bibtex.rs::escape_bib_data_specials`, at three seams: the entry line,
  `\bib@@title` and `\bib@@pages`). `_` is in the escaper ONLY: a catcode is
  fixed at tokenization and cannot tell whether it is inside `$…$`, and a
  subscript in a title's math is legitimate TeX — putting `_` in the Mouth set
  flattened every one of them. Measured across all sixteen witnesses of the
  three clusters (`_`, `%`, `&`): **193 -> 0**.
  Also fixed, a different bug the neutralization does *not* reach: the doubly
  escaped `\&amp;` / `{\&}amp;` / `&amp;`, an HTML entity that survived into
  the `.bib` and printed as "&amp;" in Perl and pdflatex alike
  (`undouble_escaped_ampersand`). Guards
  `bib_bare_ampersand_is_literal_data`, `bib_bare_ampersand_leaves_live_markup_alone`
  (the `\emph` / inline-math / space-form-accent boundary) and
  `bib_escaped_amp_entity_decodes_to_one_ampersand`. Detail in
  [`BIBLIOGRAPHY_WORKLIST.md`](../parity/BIBLIOGRAPHY_WORKLIST.md).

- **2026-07-26 (later) — session: resilience mining, and a regression the sweep caught.**
  Mined the 2605+2606 fatals: `Timeout:PushbackLimit` (25), `TooManyErrors`
  (`MaxLimit(100)` is Perl's own default — parity; `MaxLimit(500)` is our
  consecutive-error bail). Fixed the Semiverbatim text-symbol loop at its shape
  (`d28cd6427d`, PR #390): `\UseTextSymbol` now resolves to the direct glyph, as
  Perl's own `\DeclareTextSymbolDefault` does. Witness 2606.11784
  (`[OT1]{fontenc}` + a literal `í` in a `\cite` key) went `Fatal:Timeout` → 0
  errors / 519 KB; it also clears the SHARED hang 2004.08143.
  **The Perl oracle was rebuilt WITH DUMPS** (`cd LaTeXML && sudo cpanm
  --build-arg formats .`, rev `1eed356a` → `0d02309d`) — which *disproved* the
  first explanation: Perl's dump carries the same `\UseTextSymbol`-shaped
  `\?\i` (72 records), so the dump is not the differentiator. Both verdicts
  survived the apples-to-apples re-test.
  **A full 2605 rerun then caught a regression in #383's own field digest**:
  90 papers `no_problem → error`, 61 `warning → error`, 87 % of them raised
  `at Anonymous String`. Two thirds fixed in PR #391 (the `.bbl`
  `\providecommand` block; a `%` catcode phase for percent-encoded URLs — `%`
  must be corrected BEFORE tokenizing, since the comment has already eaten the
  line). The residual 61 (`_ & ^ #`) is bounded by the eager-tokenization gap —
  see R5. Guards: `textsymbol_semiverbatim`,
  `bib_field_bbl_fallbacks_render_without_a_url_package`.

- **2026-07-26 — session: the email-reported "missing References" clusters, and rc3 prep.**
  Five clusters over 11 witness papers arrived by email; oxide was already clean on
  four of them (all ~1 s, 0 errors, 1:1 cited↔rendered). The fifth was real, and
  landed as `8a964d484b` (PR #383): `.bib` field values were digested and then
  **stringified** — a Whatsit stringifies to its TeX *reversion*, so
  `note={\url{…}}` rendered as the dead literal `\urlhttps://…` — with a second,
  independent flatten in `apply_formatter`; and eleven field kinds
  (`howpublished`, `institution`, `organization`, `school`, `address`, `edition`,
  `series`, `part`, `type`, `status`, `language`) reached **no emit branch at
  all**. Guards `bib_field_markup_survives_into_the_bibliography` +
  `105_bib_field_digest_once`. Also merged `071e1541ff` (PR #384, thousands
  separator, divergence #70) and `e07548e6b3` (PR #385, short author-year label,
  divergence #71 / KNOWN_PERL_ERRORS #61). `type`-appended-to-entry-label recorded
  as **KNOWN_PERL_ERRORS #60** (PARITY, byte-identical in Perl).

- **2026-07-25 — session: siunitx v3 + split-fence math, and a worklist freshening.**
  Merged `0f7711c0b5` (PR #372) — faithful `six_format_complexnumber`
  (imaginary-unit semantics, `complex-root-position`, mantissa brackets; 0→17 of
  Perl's 20 golden signatures), the five undefined siunitx v3 commands,
  `\qtyproduct` off `\SIlist`, and Perl's `\sisetup` defaults mirrored 57→107 keys.
  Merged `0dda6ca833` (PR #373) — fences split by TeX's null delimiter now parse
  (divergence #67); measured over 24 arXiv 2606 papers carrying the pattern,
  `unparsed_math` **177 → 102**, 19 improved, 0 regressed, 0 new errors. Witness
  2606.13010 (arXiv/html_feedback#6624) now converts at 0 errors / 0 warnings /
  0 unparsed math. This file was compacted the same day — see the header.

- `cargo test --tests`: **1763 passing / 106 targets, 0 failed, 0 ignored**
  (2026-07-29, `main` @ `48de8eaa5f` plus the R5-item-2 guard, dev box with
  ImageMagick + ghostscript + poppler **and `mutool`** installed, so the
  vector-SVG branch really ran — both `test_vector_svg_*` report ok, not
  skipped). Re-run before quoting: the count moves with every PR that adds a
  guard. It rose from the long-quoted 1696 / 94 targets (2026-07-26 @
  `e07548e6b3`) as #403…#419, then #430/#432/#434/#435 (adding
  `110_acmart_description_aria` and `111_build_memory_guard`), then #442's
  `109_preload_pi_attributes` — which is the 106th target, so a
  "105 targets" quote predates it. Two claims carried here for weeks
  did **not** reproduce and have been dropped:
  `latexml_post::graphics::process_coalesces_only_matching_conversion_options`,
  long labelled "the one red, known local-only artifact", passes; and `mutool` is
  no longer absent. Re-measure before quoting either.
  **Caveat that keeps mattering:** the two vector-SVG tests
  (`test_vector_svg_graphics_path`, `test_vector_svg_pathological_convert_case`)
  do NOT go red on a bare host — `svg_converter_available()`
  (`tests/integration.rs`) returns early and reports **ok** when neither `mutool`
  nor `pdftocairo` is on PATH, and the skip `eprintln!` is swallowed without
  `--nocapture`. So a green local run does not prove that branch ran; CI installs
  poppler/mupdf. (An earlier "one `latexml_post` graphics failure needs a host
  image tool" caveat was carried forward for weeks before being shown not to
  reproduce — no `latexml_post` test can produce it as written.)

- **The next fleet rerun's fatal rate is NOT comparable to the 0.78% baseline**
  (CLAUDE.md "Active priorities"). Two 2026-07-29 changes move it in opposite
  directions, so read a delta as a measurement change first, a regression second:
  * **#434 converts silent kills into counted fatals.** Build had no cooperative
    `check_timeout()` — only digestion did — so an over-budget document was
    SIGKILLed by the hard watchdog: exit 137, no `Fatal:` line, no summary, a
    0-byte output. Those papers were never counted as fatals by a log-parsing
    tally. They now end with `Fatal:Timeout:MemoryBudget`, a partial document,
    and `Status:conversion:3`. **Fatal count goes UP with no behavior getting
    worse** — in fact strictly better, since the partial output now survives.
  * **#435 raises the default ceiling from a fixed 6144 MiB to a fraction of
    machine RAM** (`watchdog::default_ceiling_mib`, capped at 64 GiB; the
    fraction was 90% until 2026-07-30, now HALF — see the streaming-core
    design doc for why 90% was laptop-hostile). Fewer documents
    reach any ceiling on a large box, pushing the rate DOWN — and the number is
    now **host-dependent**, so two runs on different hardware are not comparable
    unless `--max-memory` is pinned. Pin it when producing a baseline.

- **2026-07-17 — crates.io: all code blockers cleared; tagged `0.7.4-rc4`.**
  `#[derive(LoadModel)]` reads `latexml_core`'s **embedded** RelaxNG table instead of
  resolving `LaTeXML.model` cwd-relative, so `resources/RelaxNG` could move into
  `latexml_core/` (108 files) where `cargo package` sees it. Also B6 (`readme`
  outside the crate dir → symlink) and the dead `script-bindings` alias, dropped
  pre-publish. Detail: [`release/CRATES_IO_PUBLISH.md`](../release/CRATES_IO_PUBLISH.md)
  B3b/B6.
  **The class worth remembering: a resource move silently breaks path-referencing
  gates.** `audit_vendored_natives.py` scanned only the root `resources/`, so B3a had
  *already* dropped XSLT/CSS/js out of the license audit unnoticed, and B3b would have
  printed `ok resources/RelaxNG/svg/ (0 file(s))` and exited 0 — its own header's
  scenario. Fixed, plus a prefix-must-exist guard (verified to fire). Same for
  `THIRD-PARTY-NOTICES` §2.2/§2.3, `LICENSE_INVENTORY` §B, `compileschema.sh`, the XML
  catalog, and ar5iv-editor's deploy script.
  **Tags are bare-numeric, no `v`** (`release.yml` matches `[0-9]+.[0-9]+.[0-9]+-*`;
  `v0.7.4-rc4` runs nothing), and `make_release.sh` refuses a tag ≠ Cargo.toml version.

- **2026-07-09 — `\AtBeginDocument` #2754/#2846 re-done via context-aware `\par`
  (Direction B retired; ported to Perl too).** The earlier `inBeginDocumentHook`
  guard-decouple is reverted: `\begin{document}` restores the pre-#2846
  `inPreamble=0`-after-hooks placement and `only_preamble` is a plain `inPreamble`
  check again (no second flag). `\lx@normal@par` is a no-op **only in the raw
  preamble** — `inPreamble` set AND `document` NOT on the `current_environment`
  stack; everywhere else it closes the paragraph being built. So a blank line in
  `\AtBeginDocument` (runs in the document env) splits paragraphs (#2754) while a
  deferred `\RequirePackage`/`\usepackage` stays legal (inPreamble still 1). NOT the
  note's literal "no-op in vertical mode": LaTeXML's mode tracking isn't faithful
  (stays vertical after display math — would also mis-merge `\AtBeginDocument{\[x\]…}`;
  raw-preamble text is horizontal yet must merge — expl3 case fixtures), so CONTEXT
  (are we in the document env) is the stable signal; the env-**stack** check also
  handles nested envs inside hooks. Applied identically in Perl
  (`LaTeXML/lib/.../latex_constructs.pool.ltxml` + `TeX_Paragraph.pool.ltxml`,
  `lookupStackedValues`). New regression tests: `tests/structure/atbegindocument_*`.
  See `KNOWN_PERL_ERRORS.md` #43. Candidate to upstream as the #2846 follow-up.

---

## Completed fixes lifted from the live worklist (cleanup 2026-08-18) — window 2026-07-29 … 2026-08-04

Lifted out of `../SYNC_STATUS.md` when the 2026-08-18 cleanup dated them done
(rule 2). Conclusions only — the durable facts live in the named guards and the
cited `OXIDIZED_DESIGN.md` / `KNOWN_PERL_ERRORS.md` entries.

- **Streaming (CORE + POST) shipped in 0.7.5.** Bounded-memory fragmented core
  conversion (`--streaming`, auto-when-doomed), page-major POST rendering, and
  two-pass streaming split — the 131 MB Nasser witness converts end-to-end in one
  call on a 31 GB laptop. PRs **#448** (CORE), **#451** (page-major POST),
  **#477/#478** (streaming split); the `{nowrap}` residual closed as issue **#297**.
  Design docs kept: `performance/STREAMING_CORE_DESIGN_2026-07-29.md`,
  `performance/STREAMING_POST_DESIGN_2026-07-06.md`. Guards `113_streaming_core.rs`,
  `118_streaming_split_parity.rs`, `114_streaming_*`.

- **2026-07-30 — a font selected by FAMILY decoded through OT1** (PR #450).
  `\selectfont`'s missing middle branch (`LoadFontMap($family)` + `MergeFont`) made
  `ding_fontmap` dead code, so bbding's `\XSolidBrush`/`\Checkmark` rendered as
  literal `%`/`!` at **zero errors** (witness 2503.04421, 28 result-table cells
  inverted). Fixed + Perl's report-once guards. Guards
  `tests/fonts/ding_family_fontmap.tex`,
  `cluster_fontmap.rs::ding_family_glyphs_decode_through_the_family_fontmap`; method
  in `WISDOM.md` §80. Residual (carried to the font-selection audit): `DeclareFontMap`'s
  `(uppercase|lowercase|digit)_mathstyle` options unported.

- **2026-07-29 — an arg-taking `\fnum@<type>` absorbed the document into an
  unclosed `<figure>`** (surpass; OXIDIZED_DESIGN #85 / KNOWN_PERL_ERRORS #68). The
  `Fig. 1:`→`Fig. 1.` hook ate the caption's closing brace because LaTeXML's
  separator is a tag *attribute*, not a token; the figure never closed and swallowed
  the bibliography. Fixed by expanding `\csname fnum@#1\endcsname{}` at all three
  `fnum@` sites. 106/106 targets, zero goldens re-blessed. Guard
  `06_cluster_regressions::cluster_fnum_arg_hook`.

- **2026-07-29 — `\bibliographystyle{alpha}` wrong label shape + no author-year
  disambiguation** (R5 item 2, `make_bibliography.rs`; KNOWN_PERL_ERRORS #67,
  divergence #84). `AY` is the abbreviated `[AS64]` label (Rust had it swapped with
  `alpha`); disambiguation must key off the SHORT name; `unisort` UCA collation;
  `NUMBER` assigned in format order. Also fixed a `fragid`-less `<ltx:biblist>`
  (MakeBibliography now registers the list). Guards
  `06_cluster_bibliography::{cluster_bib_alpha_style_labels,bib_entry_ids_are_bib_rooted_like_perl}`.

- **2026-08-04 — streaming diverged on a fancyvrb `fontsize=`+`numbers=` Verbatim**
  (PR #504; OXIDIZED_DESIGN #96 / KNOWN_PERL_ERRORS #74). `tex_glue::dimension_to_spaces`
  read the live font at CONSTRUCTION time; streaming builds mid-document, so it chose
  a different space glyph than eager. Fixed by passing the whatsit's digest-time font.
  Guards `06_cluster_regressions::faked_space_is_sized_by_the_font_it_was_digested_in`,
  `114_streaming_cluster_regressions`. Repro note: the CLI can't reach the sweep's
  3-box budget (the RSS fuse trips first); drive `streaming_sweep::convert_with(src, Some(3), …)`.

- **2026-07-29 — a `robust` DefConstructor reverted under its munged cs** (`\ref`).
  `robust=>true` installs under `\ref␣` (trailing space); Perl sets the pre-munge cs
  as `alias` and reverts to it (`DefConstructorI` L1480-1481). `dialect.rs::def_constructor`
  now sets the alias when `options.alias.is_none()`. `get_cs_or_alias()` is the clean
  accessor; code identifying a whatsit by cs must still accept both `\ref` and `\ref␣`.
  Guard `06_cluster_regressions::cluster_robust_cs_reverts_unmunged`.

- **2026-08-03 — pooled-worker libxml panic on a dead docref** (PR #491; rc4 fatal
  cluster `panic:caught`, 3/60k). The math parser's `PENDING_DISCARDS` was drained only
  after the formula loop; the resource-fatal abort path returned early, so the next
  paper on that pooled thread walked handles into the freed document. Fixed by draining
  on the abort path + a wrapper-only stale sweep (`sweep_stale_math_state`). Guard
  `latexml_math_parser/src/data.rs::stale_handles_from_a_dead_document_are_swept_without_panic`.
  Same-class residuals recorded in git (ALIGNING_NODE, `Stored::Alignment` cells,
  `STAGED_SNAPSHOTS`).

- **2026-07-25 — biblatex `.bbl` `TokenLimit` loop** (R4; witness 2605.17646).
  Self-referential `\let` on `setupPseudoBibitem` re-arm; shared with Perl. Guard
  `06_cluster_bibliography.rs::cluster_biblatex_two_datalists`. Sibling numeric-format
  fixes: guards `cluster_thousands_separator_us_default`/`_eu`, `cluster_fenced_bare_operator`,
  `cluster_leading_relop_comma_list`.

- **2026-07-27 — R9 `mathscinet.sty`** (PRs #415 + #419): `\Dbar` etc. are mathscinet's
  macros, not any `.bst`'s. Binding `latexml_package/src/package/mathscinet_sty.rs`;
  guards `bib_mathscinet_{package,author_macro}`.

- **`--format=xml` emits no `ltx:bibitem` for a BibTeX source — NOT A BUG** (triaged
  2026-08-18). `<ltx:bibitem>` from `\bibliography{}` is a MakeBibliography (*post*)
  product; `--format=xml` runs no post (`do_post=false`, latexml_oxide.rs:1336-1345),
  matching Perl `latexmlc --format=xml` byte-for-byte (0 bibitem, placeholder only;
  `--format=html5` expands it). Explicit `\begin{thebibliography}` emits `<bibitem>` at
  CORE in both engines (green goldens `tests/structure/{natbib,crazybib}.xml`). Live-doc
  takeaway retained: an xml-format dump is core-only.

- **2026-07-29/30 — Presentation-MathML F17 CLOSED** (from the archived MathML-post
  line audit). F17 was a *list*, not a family — 9 items each individually scoped: **4
  fixed, 3 do-not-port/N-A, 1 blocked, 1 unreachable.** Durable lesson: **run both
  engines on an item before porting** — reading the audit alone would have added a
  divergence or dead code.
  - **FIXED:** `pmml_text_aux` `%attr` threading onto `m:mtext` (+ leading-ws→NBSP,
    cross-pass `m:math` reuse by namespace URI; deleted the dead second
    `stylize_content`) — guard `90_latexmlpost::mtextstyle_post_test`; `outerWrapper`
    altimg + RDFa families, which also needed the missing `CrossRef::fill_in_RDFa_refs`
    port (resolves `aboutidref`/`labelref`) — guards `mathouter_post_test`,
    `06_cluster_regressions::cluster_rdfa_math_subject`; `pmml_scriptsize_padded`
    embellishment padding for primed sums (`\mathop{X'}\limits`) — guard
    `mathprimed_post_test`; `preprocess` plane1 config + new
    `--plane1`/`--noplane1`/`--hackplane1` — guard `plane1_modes_match_perl`. Also
    2026-07-30: braced script-chain fold made unbounded left-recursion (`{x^a}^b` etc.)
    — guard `06_cluster_math::cluster_script_chain_depth`; an absent `mmultiscripts`
    slot now emits empty `m:mrow` not `m:none` (Core removed `<none>`; OXIDIZED_DESIGN
    #86 retracted) — guard `scriptlevels_post_test`.
  - **DO-NOT-PORT (would create a divergence or dead code; do not re-open without a
    witness):** `pmml_infix` ADDOP flatten via `pmml_unrow` is dead in Perl too
    (`associateNode` stamps `_sourced`, so the empty-attr guard never passes — Perl
    emits the same 5 nested `m:mrow`s); `Apply:?:formulae` phantom op never reaches
    pMML (XMDual/XMWrap on both sides); `pmml_parenthesize`'s `usemfenced` is never set
    anywhere in Perl and `m:mfenced` is gone from Core; `nestmath` has no CLI in either
    engine; Perl's `$emb_left` is dead code.
  - **BLOCKED:** `combineParallel`'s non-MathML-secondary branches need
    `--openmath`/`--mathimages`/`--mathsvg` (absent in Rust) — untestable dead code
    until that larger math-format feature lands.

- **2026-08-05 — Font-selection chain audit fixes (PRs #450, #452, #453, #454, #455, #456):**
  Six font audit findings merged:
  - `\char`/`\symbol` yielding empty string in math mode fixed (guard: `117_char_font_decode::char_decodes_through_ot1_in_math_and_does_not_wrap_out_of_range`).
  - `\DeclareSymbolFont` encoding argument expanded before storage (guard: `117_char_font_decode::symbol_font_encoding_argument_is_expanded_before_storage`).
  - `\DeclareMathAlphabet` calls `lookup_tex_font` instead of raw NFSS codes (`latex_constructs.rs`).
  - `\mathversion{bold}` merges `mathfont` instead of text font (guard: `06_cluster_regressions::mathversion_switches_the_mathfont_like_boldmath`).
  - Dead font helpers (`ding_fontmap.rs`, `font::decode_str` FontDecode variant, `font::lookup_tex_font`, `font::rationalize_font_size`) connected or cleaned up.
  - `\textit@math` shape switch assigns `it` not `i`.

- **2026-08-21 … 2026-08-23 — Sandbox cortex triage (2605/2606) & Omnibus safe slice:**
  - **PR #720 landed:** `scalerel` neutralization (preserves object, CSS-sized to text height), `neurips` `\if@anonymous`, `NiceTabular` reduction, `expl3` `#630` token scanning, `biblatex` loop guard, `cleanup_scripts` optimization $O(M \times N) \to O(N+M)$.
  - **Stacked PR landed:** cleveref class stubs, AASTeX, and subdir/`.sty` binding shadow fix (dropped directory stripping at both package dispatcher and `find_file_fallback` so paper-local `subdir/<name>` raw-loads under `localrawstyles`; guard: `cluster_package_guards.rs::subdir_dispatch_no_strip`).
  - **OmniBus frontmatter safe slice landed (0.7.6, OXIDIZED_DESIGN #160):** `\orcid[]{}` captures ID as `<contact role="orcid">` with orcid.org link; `\lefttitle`/`\righttitle` no-op'd as presentational running heads (guard: `omnibus_captures_orcid_and_drops_running_heads`).


