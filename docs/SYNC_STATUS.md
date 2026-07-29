# Engine Sync Status — Active Worklist

> **DO NOT downgrade Errors to cheat the task.** If Perl LaTeXML converts a paper
> without a downgrade, the Rust translation must match by improving the core
> engine — never by silencing diagnostics. New downgrades require explicit proof
> Perl emits the same severity on the SAME paper, else they hide a real gap.
> (User directive 2026-05-15.) Always classify with `latexml --verbose`, never
> `--quiet` (which hides Perl's `Error:` lines); cross-check pathological inputs
> with `pdflatex`.

## How to read this file

**Start at "Ranked worklist" below and take the top unblocked row.** That is the
whole intent of this file; everything after it is supporting detail.

| section | what it is | when you read it |
|---|---|---|
| **Ranked worklist** | every open item, ordered, with size + where the detail lives | **first, always** |
| Current status | suite count, the last session, release state | to orient |
| Open items | the detail behind the ranked rows | when you pick that row |
| Standing policies | rules that constrain *how* you fix things | before adding a CLI flag, a stub, or a divergence |
| Parked families | pointers to four extracted docs | only when starting that family |
| Reference | stable facts, not work | when something surprises you |

Three rules that keep this file honest:

1. **Verify a status label before acting on it — and before deleting it.** Four
   entries here have pointed at work that did not exist: a `13 commits NOT
   PUSHED` banner (merged as PR #323), a "#312 → render under MathJax 4" step
   (issue closed; `ISSUE_AUDIT.md` calls that screenshot out of scope), a
   `CLI options (#191) — ACTIVE` heading (issue closed), and a "PR #310 … ready
   to merge" line (already merged). Check the **named guard test** in the tree,
   or `gh issue view <N>` / `gh pr view <N>`. **SHA-ancestry does not work** as a
   check — the repo squash-merges, so a branch SHA quoted here is never an
   ancestor of `main`.
2. **This is the BRIEF ACTIONABLE LIST.** Day-by-day logs live in `git log` and
   `docs/archive/`. When you close an item, delete it here and lift anything
   worth re-reading into `docs/archive/SYNC_SESSIONS_YYYY-MM.md`.
3. **Keep it under ~500 lines.** When a section outgrows ~100 lines it has become
   its own subject — give it a doc under `docs/` and leave a one-line pointer.

*Last compaction: 2026-07-25 — 1979 → ~500 lines. 23 completed sections lifted to
`SYNC_SESSIONS_2026-07.md`; four standing families extracted (see Parked
families).*

## Ranked worklist — start here

Ordered by: **does it reproduce today** → **is a real user affected** → **is it
unblocked** → **effort**. Rows R1–R2 are small and self-contained; R4+ need a
session of their own. Re-verify a row before planning on it (rule 1).

| # | item | state | size | detail |
|---|---|---|---|---|
| **R1** | Upstream `brucemiller/LaTeXML#2852` — subfile `\documentclass` options | **OPEN upstream**, ours merged as #310 | minutes — chase review, no code | Open items |
| **R2** | `--preload=<cls>` trips the LaTeX hook stack (`Extra \PopDefaultHookLabel`) | **OPEN**, re-verified 2026-07-25 (1 error with `--preload=article.cls`, 0 without) | small–medium, self-contained | Open items |
| **R3** | **Bibliography-absence mini-sprints S1–S10** — audited 2026-07-29 over 2605/2606 sandboxes + full `/data/arxiv`: **255+278 sandbox / 52 299 corpus docs wrongly lack every `ltx_bibitem`**, 12 583 corpus cases fully SILENT. Cause families F1–F12 clustered with witnesses + min-repros; complete case lists committed. Take sprints in order: S1 un-silence (2 parity Error raises, small) → S2 `$$`-in-bib containment (~69+68 fatals/pair) → S3 `.bbl` fidelity (jobname fallback, REVTeX `auto@bib`, bibunits) → S4 biblatex arg bugs → S5 achemso `\iffalse` (1 700 corpus) → S6 no-discard-built-doc → S7 silent group-leak swallow → S8 post-stage OOM → S9 legacy conventions (16 464 corpus) → S10 surpass rulings (user) | **NEW 2026-07-29** — S1 unblocked, start here | per-sprint: small→large | [`BIB_ABSENCE_AUDIT_2026-07-29.md`](parity/BIB_ABSENCE_AUDIT_2026-07-29.md) |
| **R4** | biblatex `.bbl` `TokenLimit` loop (2605.17646) | ✅ **FIXED 2026-07-25** — self-referential `\let` on `setupPseudoBibitem` re-arm; shared with Perl | — | Open items |
| **R5** | Bibliography targets + MakeBibliography re-port | **re-port items 1 and 3 LANDED**: a raw `.bib` is converted by the engine (recursive BibTeX session on the LIVE core state), the 727-line string route is deleted, the eager-tokenization gap that cost 151 papers is closed at the root, and the 13-field digest whitelist is gone — the `\bib@field@default@*` name sets now match Perl exactly (45 each). The 2026-07-27 follow-ups closed the `.bib`-as-DATA family (divergences #74/#78/#79/**#80**). Remaining: item 2 (unisort, citestyle `AY`, `Formatter::Year` suffix, doc-global NUMBER) and the missing-references target list | **item 2 + targets** — the re-port itself is done | [`BIBLIOGRAPHY_WORKLIST.md`](parity/BIBLIOGRAPHY_WORKLIST.md) |
| **R6** | `ltx_env_<name>` env-markup class | user-requested, PLANNED | medium code, **large golden churn** → own branch | Open items |
| **R7** | Beyond-Perl performance levers BP-1…BP-6 | POST-RELEASE; internal order BP-2 → BP-3 → BP-1 | **family** | [`BEYOND_PERL_LEVERS.md`](performance/BEYOND_PERL_LEVERS.md) |
| **R8** | Content-MathML / math-parser gaps | **deferred by user directive 2026-06-20** | **family** — do not pick off in isolation | [`CONTENT_MATHML_GAPS.md`](math/CONTENT_MATHML_GAPS.md) |
| **R9** | Deep deferred families (`.bst`, xy-pic, mode-frame, …) | parked; several carry explicit "do NOT start". The `.bst` row's "`.bst` files *vendor macro definitions*" premise was **RETRACTED 2026-07-27** (`alpha.bst` has zero `Dbar`; the macro is `mathscinet.sty`'s) — it survives on label style / sort order / **field selection**, and the prerequisite is a corpus measurement of the `.bib`+`.bst`-with-no-`.bbl` population | **family** | [`DEFERRED_FAMILIES.md`](parity/DEFERRED_FAMILIES.md), and R9-BST below |
| — | `\gls`/`\acrshort` in math mode (1705.10306) | **PARITY, blocked** on unrunnable Perl | — | do not chase; Open items |
| — | Two-pass streaming split | **deferred by user decision 2026-07-06**; trigger = a <64 GB target appears | — | [`STREAMING_POST_DESIGN_2026-07-06.md`](performance/STREAMING_POST_DESIGN_2026-07-06.md) |

## Current status

- **2026-07-27 (latest) — the `unexpected:fi` fatal cluster: `\meaning` of a
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
  audit: [`EXPL3_CATCODE_GAP_2026-06-08.md`](parity/diagnostics/EXPL3_CATCODE_GAP_2026-06-08.md)
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
  [`BIBLIOGRAPHY_WORKLIST.md`](parity/BIBLIOGRAPHY_WORKLIST.md).

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

- `cargo test --tests`: **1760 passing / 105 targets, 0 failed, 0 ignored**
  (2026-07-29, on `main` @ `d5684f0bcf`, dev box with ImageMagick + ghostscript +
  poppler **and `mutool`** installed, so the vector-SVG branch really ran — both
  `test_vector_svg_*` report ok, not skipped). Re-run before quoting: the count
  moves with every PR that adds a guard. It rose from the long-quoted 1696 / 94
  targets (2026-07-26 @ `e07548e6b3`) as #403…#419 and then #430/#432/#434/#435
  landed — the last four adding `110_acmart_description_aria` and
  `111_build_memory_guard`. Two claims carried here for weeks
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
  * **#435 raises the default ceiling from a fixed 6144 MiB to 90% of machine
    RAM** (`watchdog::default_ceiling_mib`, capped at 64 GiB). Fewer documents
    reach any ceiling on a large box, pushing the rate DOWN — and the number is
    now **host-dependent**, so two runs on different hardware are not comparable
    unless `--max-memory` is pinned. Pin it when producing a baseline.

- **2026-07-17 — crates.io: all code blockers cleared; tagged `0.7.4-rc4`.**
  `#[derive(LoadModel)]` reads `latexml_core`'s **embedded** RelaxNG table instead of
  resolving `LaTeXML.model` cwd-relative, so `resources/RelaxNG` could move into
  `latexml_core/` (108 files) where `cargo package` sees it. Also B6 (`readme`
  outside the crate dir → symlink) and the dead `script-bindings` alias, dropped
  pre-publish. Detail: [`release/CRATES_IO_PUBLISH.md`](release/CRATES_IO_PUBLISH.md)
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

### Session logs (2026-06-22 … 2026-07-08) — ARCHIVED

Completed "Landed this session" entries, the slowest-100 batch triage, the
finished upstream-sync U1–U11 mission log, and the mined-out methodology
history now live in the dated session archives:

- [`archive/SYNC_SESSIONS_2026-07.md`](archive/SYNC_SESSIONS_2026-07.md) —
  the 2026-07-02 … 07-08 window: upstream PR #2829 "Framing", the MathML-post
  exhaustive line audit (waves 1+2), live-run fatal/error mining rounds,
  author/affiliation frontmatter split, width-based figure-panel arrangement,
  and the `\AtBeginDocument`/`\RequirePackage` #2846-port regression fix.
- [`archive/SYNC_SESSIONS_2026-06.md`](archive/SYNC_SESSIONS_2026-06.md) —
  the 2026-06-22 … 07-01 window plus the slowest-100 batch triage and the
  2026-06 cortex-cross-join methodology history.

(Upstream-sync catalog also at
[`archive/UPSTREAM_SYNC_2767_to_2833_2026-06-26.md`](archive/UPSTREAM_SYNC_2767_to_2833_2026-06-26.md).)

## Standing policies & method — read before changing behaviour

### Methodology & the cortex cross-join

Working method (2026-06): **re-triage LARGE-error papers** (the single-error tail
is exhausted) → bisect the doc to the trigger line → verify Perl with `--verbose`
→ fix the divergence. Random sweeps are low-yield.

**Clustering trap — a bibliography sub-conversion's fatal has no fatal message.**
It is deliberately downgraded to a trailing `Error:bibliography:convert` on the
parent (`bib_session.rs` uses the post-phase reporters; Perl's
`convertBibliography` returns empty-handed the same way,
`MakeBibliography.pm` L240-242). So a sweep that clusters on **fatal messages**
files those documents as "no fatal recorded" — measured **~80** in one run.
Cluster on `Status:conversion:3` or the last `Error:` line. Mechanism and what
DOES cross the boundary (`MergeStatus`): [`WISDOM.md`](parity/WISDOM.md) **#72**.

**Cortex agentic API (reads open, no token):** `http://127.0.0.1:8000/api`.
Recipe: `GET /api/reports/<corpus>/oxidized-tex-to-html/<severity>` → categories;
`…/<severity>/<category>` → per-`what`; `…/<category>/<what>` → paper list. Then
`GET /api/corpus/<corpus>/tex_to_html/document/<id>` for Perl status — a Rust-only
win is **Perl=no_problem/warning but Rust=error/fatal**. Corpus
`sandbox-arxiv-10k-shuffle`. URL-encode `\`→`%5C`, `^`→`%5E`.

### CLI options — the option-C policy (issue #191 CLOSED 2026-07-09) + `validate()`

Issue #191 "support the original latexmlc/latexmlpost options" is **closed**;
what survives here is the standing **option-C policy** it established, plus the
one feature deliberately left undone (`validate()`, below). The policy: wire only
options whose engine feature genuinely works end-to-end; keep the clap parser
**strict** (no accept-and-warn stubs); deferred/missing features stay hard parse
errors. Consult it before adding any CLI flag.

#### Deferred — feature genuinely NOT supported (do NOT stub)
- `--parse=STRATEGY` — grammar selection unsupported (one Marpa grammar);
  `--nomathparse` / `--mathparse` is the real interface. (Attempted + removed.)
- `--svg` / `--nosvg` — **deferred (verified 2026-07-09):** the HTML5 XSLT
  already renders `<ltx:picture>` as inline `<svg>` by default, so the standalone
  `svg.rs` post-processor (`impl Processor for SVG`, unwired) is redundant and
  produces divergent, unverified output (25 vs 27 `<svg>` on `tests/graphics/
  picture.tex`). Wiring it was built + reverted.
- `--pictureimages` / `--nopictureimages` — `picture_images.rs` delegates to the
  **unwired LaTeXImages latex+dvipng pipeline** (`latex_images.rs`); same
  category/effort as `--mathimages`.
- `--openmath|om` — no OpenMath serializer. (User: defer.)
- daemon net (`--port` / `--address` / `--expire` / `--autoflush` / `--cache_key`)
  — socket-daemon model; we ship `--server` (stdio LSP). (User: defer.)
- `--mode` (= alias for `--profile`); `--profile=NAME` — needs a preset registry.
- `--mathimages` / `--mathsvg` / `--mathimagemagnification` — needs a
  latex+dvipng math-render pipeline.
- `--unicodemath` / `--plane1` / `--hackplane1` / `--linelength` — plain/unicode
  math output modes.
- crossref cluster (`--crossref` / `--scan` / `--noscan` / `--urlstyle` /
  `--prescan` / `--dbfile` / `--bibliography` / `--splitbibliography`) + index
  cluster (`--index` / `--permutedindex` / `--splitindex`) — multi-doc site-DB
  features. (Scan IS wired as post Phase 2, so `--noscan` is a real-but-risky
  off-switch; parked with the cluster.)
- `--tex` / `--box` — intermediate box/tex serializers absent.
- `--omitdoctype` — DTD-only in Perl; Rust has no DTD (moot).

#### `validate()` / `--validate` — POSTPONED to the NEXT release (decided 2026-07-09)
Today `Post::Document::validate()` (`latexml_post/src/document.rs:1717`) is a
STUB: it logs "Would validate against RelaxNG schema" and returns `Ok(())`.
Real RelaxNG validation is wanted, but is **deferred to the next release** because
it is gated on a `rust-libxml` crates.io publish (see below). Reference: Perl
`LaTeXML/lib/LaTeXML/Common/XML/RelaxNG.pm` + `LaTeXML/lib/LaTeXML/Post.pm`.

**Architecture decision (owner, 2026-07-09): `rust-libxml` provides the public,
safe Rust RelaxNG interface; `latexml-oxide` is a pure consumer.** All libxml2
`unsafe`/FFI stays in the fork — the alternative (raw `xmlRelaxNG*` FFI inline in
`latexml_post`, which would compile against the shipped crates.io `libxml 0.3.15`
with no publish) was **rejected**. So this feature cannot fully land until the
fork's RelaxNG module is published as `libxml 0.3.16`.

Constraint: the schema is **modular** (`LaTeXML.rng` `<include>`s
`LaTeXML-common.rng`, `-structure`, `-math`, …) and the binary is
**self-contained** — no on-disk schema. Includes MUST resolve through the
embedded table (`latexml_core::common::relaxng::embedded::lookup`), served via
the fork's existing `libxml::io::register_input_callback` (built for exactly this
— "bundles RNG schemas via include_bytes! … RelaxNG `<include>` via
`xmlRelaxNGParse`"), NOT disk.

Steps (next-release session):
1. **rust-libxml fork — add a safe `relaxng` module.** The fork's `schemas`
   module is **XSD-only** (`xmlSchema*`). Mirror it: `relaxng/{parser,schema,
   validation}.rs` wrapping `xmlRelaxNGNewParserCtxt`(URL — so relative includes
   resolve through the callback) / `xmlRelaxNGNewMemParserCtxt` + `xmlRelaxNGParse`
   (→ `RelaxNGSchema`) and `xmlRelaxNGNewValidCtxt` + `xmlRelaxNGValidateDoc`
   (→ `RelaxNGValidationContext`), with `xmlRelaxNGSetValidStructuredErrors`
   capture. Fork unit test (valid + invalid doc). **Publish `libxml 0.3.16`.**
2. **Embedded-include resolution** via `libxml::io::register_input_callback`
   (`embed:///RelaxNG/LaTeXML-*.rng` → `embedded::lookup`); verify with the
   renamed-`resources/` smoke that no schema is read from disk.
3. **Consume in workspace** — bump the `libxml` dep `0.3.15` → `0.3.16`; `cargo test`.
4. **Flesh out `validate()`** — parse+cache the schema once; run `validate_doc`;
   map each captured `StructuredError` to a `Warn!` / `post_error` in the project
   logging convention (Perl reports schema violations).
5. **Wire `--validate` / `--novalidate`** — CLI flags + `PostOptions.validate`;
   call `validate()` in `run_post_processing_impl` when enabled. DEFAULT
   decision: Perl defaults ON; propose **opt-in** in Rust (validation cost +
   corpus warning noise) as a documented divergence — confirm with owner before
   flipping the default on.
6. **Tests** — a valid fixture validates clean; an intentionally schema-invalid
   doc reports the expected violation; `--novalidate` skips.

### Archived-audit residuals (2026-07-09 docs compaction) — still-open leftovers

Two completed diagnostic snapshots were dated + archived; their still-open
residuals stay here so the live worklist keeps them visible:

- **MathML-post line audit** (sweep complete; →
  `archive/MATHML_POST_LINE_AUDIT_2026-07-05.md`). Open feature-gaps: **F5**
  Linebreaker (full feature gap — the sketch used the wrong strategy), **F11**
  Hint width normalization, **F14** multirelation + lt-or-approx cMML, **F15**
  continued-fraction, **F16** OperatorDictionary Cat A/B data holes + U+2A50
  misclassification + fence U+0331, **F17** formulae pMML arm, plus PARTIAL
  inherited-context bindings on `pmml_top`/`pmml_parenthesize`/`stylizeContent`.
  (Content-MathML items obey the defer-to-a-dedicated-session directive above.)
- **arXiv velocity-fork audit** (items 1–4 landed 2026-07-03; →
  `archive/ARXIV_FORK_AUDIT_2026-07-03.md`). Sole residual: **item G** —
  `readBalanced` drops comment tokens (fork `4e1578d1`); Rust `read_balanced`
  still flushes `pending_comments` (gullet.rs ~L1170). Low urgency
  (`INCLUDE_COMMENTS=false` default); port at the next gullet-seam session.

## Open items — detail for the ranked rows

### R1 — upstream `brucemiller/LaTeXML#2852`: a subfile's `\documentclass` options are not packages

**OPEN upstream** (state checked 2026-07-25); **our half is already merged as PR
#310**, so nothing is pending here in this repo. The allowlist was hand-split on
`,` and missed every valued form (`[varwidth=5cm]` → `Error:undefined:{varwidth}`,
pdflatex clean); it now reads `OptionalKeyVals` and matches on the key. The same
fix is ported to Perl (`OptionalKeyVals` + `getPairs`) with a `t/structure` case
that actually guards it, pushed to `dginev/LaTeXML`. **Action: check its CI, then
ask for review** — no code work expected. *(This entry read "PR #310 … Ready to
merge" until 2026-07-25, long after it merged.)*


### R2 — `--preload=<cls>` alone trips the hook stack; class-name divergence — OPEN (re-verified 2026-07-25)

**Symptom.** `--preload=<any>.cls` prints `LaTeX hooks Error: Extra \PopDefaultHookLabel`
(article/book/report; `.sty` clean; `\documentclass` clean; `LATEXML_NODUMP=1` clean).
Perl is silent for the same preload.

**Mechanism (traced, 2026-07-17).** Push/pop are perfectly balanced and nested — the trace
is `PUSH article → (LaTeX.pool loads) → PUSH textcomp → POP textcomp → POP article → error`.
The bug is that **`\@pushfilename` changes MEANING mid-load**: `article` is pushed *before*
`LaTeX.pool` (and the kernel dump behind it) loads, so it uses a pre-pool `\@pushfilename`
that never touches `\g__hook_name_stack_seq`; the pool then installs the real expl3
`\@popfilename`, so `article`'s pop hits a seq holding only the *inner* packages' pushes,
finds it empty, and errors. `\documentclass` escapes because the pool is already loaded, so
both sites use the same meaning.

**A definedness check cannot see this** — the CS is defined at both sites; only its meaning
changes. Perl's `$pushpop` (Package.pm L2595, computed once and reused at L2637) is a
definedness check too, so Perl has the same hole; it is silent only because its dump omits
`\g__hook_name_stack_seq`, and `\seq_gpop:NNTF` on an *undefined* seq does not complain.
Ours dumps it as `\c_empty_seq`, so the real expl3 code correctly notices.

**Mitigated, not fixed (2026-07-17):** `util::preset::new_test_engine` now preloads
`LaTeX.pool` first (the order ar5iv's list already used), so `latexmlmath_oxide` stops
provoking it. `--preload=article.cls` on its own STILL errors.

**Dead ends — measured, do not retry:**
* Filtering the L3-hook stubs + filename stack from the dump (write+read): symptom gone,
  preloads clean — but `cluster_mhchem_cf_author_macro` 0 → **1003 errors** (suite 1581/0 →
  1572/9). The dump REPLACES base (DUMP_DESIGN rule 1), so filtering leaves a HOLE, not a
  fallback to `latex_base.rs`.
* Filtering ONLY `\g__hook_name_stack_seq` to match Perl's dump exactly: symptom gone,
  mhchem still fails — that record is load-bearing for our expl3 emulation.
* Threading Perl's `$pushpop` from push to pop instead of re-deciding (more Perl-faithful,
  worth doing anyway): does NOT fix it — the flag is `true` at both sites; the *meaning*
  moved underneath.
* Filtering `\PopDefaultHookLabel` alone: inert. The erroring caller is the internal
  `\__hook_curr_name_pop:`.

**Where the failing pair is NOT (2026-07-25, measured — corrects an earlier note
in this entry).** The Rust binding has exactly two push/pop sites:
`binding/content.rs:1000-1015` (push) and `:826-831` (pop). Probing both on the
failing `--preload=article.cls` run shows **only `textcomp`'s push reaches them —
`article`'s push and BOTH pops never do.** So the erroring pair is not a Rust-side
`digest`; it runs inside TeX (expl3's `\__hook_curr_name_pop:`, as noted above).

**Consequence: a Rust-side "thread the push's answer to the pop" fix cannot work**
— there is no Rust-side pop for the failing frame to pair with. An earlier version
of this entry proposed exactly that (inspect `\@pushfilename`'s body for
`\@expl@push@filename@aux@@` at push, carry it to the pop); it is a **fifth dead
end**, disproved before implementation. Any real fix has to act on the TeX side —
which points back at (c), ordering the pool load before the class's own
`\@onefilewithoptions` push, rather than at the Rust seams.

**Candidate fixes.** (a) Ensure a class/package preload cannot be the thing that drags in
the pool — auto-prepend `LaTeX.pool` when any `.sty`/`.cls` is preloaded. Rejected for the
release: Perl prepends only `TeX.pool` (LaTeXML.pm L710) and never auto-loads `LaTeX.pool`,
so this is a Rust-only divergence, and it would drag the LaTeX kernel into a `.sty` preload
on a plain-TeX document (the LaTeX-2.09 class `graphicx_sty.rs` already guards against). If
adopted, make it conditional on the pool being unloaded and LOG it. (b) Pair the pop to the
push's actual *meaning* rather than to definedness. (c) Make the pool load before any
handleoptions push. (b)/(c) address the cause.

**Second divergence, same area.** `\documentclass{article}` → `<?latexml class="article"?>`
but `--preload=article.cls` → `<?latexml class="article.cls"?>`; **Perl emits
`class="article"` for both.** Otherwise the two paths' output is byte-identical, so the
preload does load `article_cls.rs` correctly; `parse_preload_spec` splits correctly to
`("article","cls")`, so the extension is re-attached further in.

### R4 — biblatex `.bbl` TokenLimit loop, 2605.17646 — ✅ FIXED 2026-07-25

Root cause was **not** `\missing{Cowen2021}` (the `.bbl`'s last line, and the
entry's standing suspicion): deleting it leaves the Fatal untouched. It is a
self-referential `\let` in the engine's pseudo-bibitem machinery —
`setupPseudoBibitem` re-arming captures `\save@bibitem` ← `\restoring@bibitem`,
whose body ends in `\bibitem`, so it expands forever. The re-arm happens because
biblatex's apa style asks biber for **two sorting schemes**, so the `.bbl`
carries two `\datalist` blocks (2 × 29 entries here) and each `\enddatalist`
expands to a whole *bare-CS* `\thebibliography…\endthebibliography` — no group,
so the first arming was still live when the second opened.

Fixed in two symmetric halves: `setup_pseudo_bibitem` captures the originals
once per arming (`\ifx\bibitem\restoring@bibitem` guard), and
`\endthebibliography` now disarms — upstream has no teardown, relying on
`\begin`/`\end` popping the group, which the bare-CS pair never opens. The
missing teardown was separately costing a stray empty bibitem outside the
biblist (`Error:malformed:ltx:bibitem`) from the blank line after
`\printbibliography`.

Witness now converts in ~1 s with **1 error** (`\missing`, undefined in both
engines) and 58 bibitems / 2 bibliographies / 2 biblists — byte-for-byte the
structure same-host Perl produces, which takes 33.7 s and reports **59 errors**.
**The defect is shared with Perl** (`\thebibliography \endthebibliography
\thebibliography \bibitem{b}` hangs Perl 0.8.8 >400 s); it stays latent upstream
only because Perl's biblatex binding never defines `\printbibliography`, so Perl
never reads a real `.bbl` this way. Mechanism, minimal trigger and the
upstream-candidate note: `KNOWN_PERL_ERRORS.md` #57. Guard
`06_cluster_regressions::cluster_biblatex_two_datalists`.

**Follow-up the same day — the witness is now error-free.** Two more gaps it
surfaced, both landed:
* **`\missing{key}` was undefined** (`Error:undefined:\missing`). It is biber's
  marker for a cite-key absent from every `.bib` (TL `biblatex.sty` L8503
  `\blx@bbl@missing`): upstream records the key and emits a **warning**,
  typesetting nothing. Ported faithfully to `biblatex_sty.rs` — a no-op that
  names the key (`Warning:missing_entry:biblatex`), which is the author's bug,
  not ours (issue #92). Perl's binding leaves it commented out
  (ar5iv-bindings L613), so every biber `.bbl` carrying one errors there.
* **A leading relop + comma had NO parse.** `list_apply`'s fragment guard
  rejected any item with an `absent` relop operand while
  `formula relop formula_list` is deliberately gone (`KNOWN_PERL_ERRORS` #37),
  so `$>50,000$` was `ltx_math_unparsed` though `$>x$`, `$a,b$` and
  `$a>50,000$` all parsed. The guard now rejects a **comma** pair only when
  BOTH items are fragments (mirroring the relaxation `formulae_apply` already
  carried) and stays strict for `\quad`, where a fragment run is one broken-up
  equation — the `\quad` half is load-bearing: relaxing it too made
  `tests/math/sampler`'s `\displaystyle=f(x)+\phantom{g(x)}+h(x)` parse
  *wrongly* rather than not at all. Guard
  `06_cluster_regressions::cluster_leading_relop_comma_list`.

**Then the residual math gaps too (user-directed, same day) — the witness is now
0 errors AND 0 unparsed formulas.** Two grammar additions, both measured against
same-host Perl:
* **A bare operator used as an OPERAND** — `f(\cdot)`, `\langle\cdot,\cdot\rangle`,
  and operators NAMED rather than applied (`(+)`, `(=)`, `(\times)`). The grammar
  admitted fenced singleton bigops/OPERATORs but not the ADDOP/MULOP/BINOP/RELOP
  roles, so **Perl parsed 7 of 8 such shapes and we parsed 0**. New
  `placeholder` / `placeholder_list` (`grammar/builder.rs`) admit them only where
  FENCED — the same containment the bigop lines use — so a stray `a + \times b`
  still fails. `$\|\cdot\|$` stays unparsed as parity (Perl fails it too).
* **A comma list mixing ONE relation with a plain term.** `formula_list` carried
  only the all-`modified_term` variants, the mixed ones deferred *"until a
  witness shows them needed"* — so `f(a\geq 0, b\leq 1)` parsed while
  `f(a\geq 0, b)` did not. arXiv 2605.17646's
  `m_S(t \mid T_i \geq t_{\text{crit}}, \mathbf{Z})` is that witness; Phase 2
  adds both orders.

Not just this paper: on the #37 ambiguity stress witness **1510.03361** the two
additions took `ltx_math_unparsed` **170 → 136** *and* wall time **16.8 s →
11.7 s** — formulas that used to exhaust the parser now succeed early. The
`parse_tree_count_limits` canary stays green.

Witness end state: **0 errors, 1 warning** (the actionable missing-entry), 1.0 s,
**312 formulas / 0 unparsed** (Perl: 0 unparsed, 59 errors, 33.7 s), and
structurally identical to Perl — same counts for all 25 element classes sampled
(312 `Math`, 58 `bibitem`, 336 `td`, 78 `ref`, …; sole delta 87 vs 88 `para`).
Guards `cluster_fenced_bare_operator`, `cluster_leading_relop_comma_list`.

**Thousands separator — ✅ FIXED 2026-07-25 (US default; EU already worked).**
`50,000` is ONE number; both engines read the comma as a list separator. Owner
policy: **default US, EU a supported secondary.** The `en` half was the broken
one — Perl's thousands arm demands `$r ne 'PUNCT'` and a math comma is always
PUNCT, so it is dead code for English, while the EU decimal comma already works
through the language maps. **The ligature is the wrong seam and that is a
measured dead end** (built, reverted): ligatures run per-token during building,
so there is NO right context, and a merge-at-three-digits rule corrupts plausible
pairs — `$(1, 2024)$` → `12024`. Landed instead as a `DefRewrite` in the
post-build `Rewriting` phase, where the ligature has already collapsed each digit
run into one token, so the group length is testable with its right context and
those cases are safe by construction. Guards
`cluster_thousands_separator_us_default` / `_eu`; mechanism, the two
implementation traps and the full result table in
[`CONTENT_MATHML_GAPS.md`](math/CONTENT_MATHML_GAPS.md).


### R9-BST — `.bst` support: raw interpretation vs `_bst.rs` bindings — FUTURE, not started

**How this row came to be, and a correction (2026-07-27).** It was opened on the
claim that `\Dbar` in witness **2605.11579** proved `.bst` files *vendor macro
definitions*. **That claim was wrong and is retracted.** Checked afterwards:

* the witness uses `\bibliographystyle{alpha}`, and `alpha.bst` contains **zero**
  occurrences of `Dbar`;
* `\Dbar` is defined by **`mathscinet.sty` v1.05** (ships with **amsrefs**;
  `\ProvideTextCommand{\Dbar}{T1}{\DJ}`), which also defines `\cprime`,
  `\cdprime` and `\bud`;
* **the witness does not load `mathscinet.sty`**, so real pdflatex on that source
  raises the same undefined control sequence. `undefined:\Dbar` there is
  **PARITY**, not a defect — see R9-MSC below, which is now DONE. (The witness
  stopped *emitting* that error on 2026-07-27 for an unrelated reason: divergence
  **#80** digests only cited entries, and its `\Dbar` entry is uncited. The
  parity reasoning is unchanged; the witness is just no longer a demonstration
  of it.)

**What survives the correction.** The row still stands, on its original and
independently-true footing: we read `.bib` directly, never execute the `.bst`,
and therefore cannot reproduce the label style, sort order or **field selection**
a document's own `.bst` would produce. Field selection is the part currently
approximated by hand — OXIDIZED_DESIGN **#73** neutralizes `abstract`/`keywords`/
`contents` precisely because no standard `.bst` declares them in its `ENTRY`
list, which is a `.bst` fact we hard-coded rather than computed. A `.bst` we can
execute is what would replace that guess with an answer.

**One half of that shrank 2026-07-27.** *Entry* selection no longer needs a
`.bst` at all: divergence **#80** takes the cited set from the document's own
`BIBLABEL` records — the same information `bibtex(1)` reads out of the `.aux` —
so which entries reach the `.bbl` is now computed, not guessed. What remains
`.bst`-shaped is *field* selection (#73's hard-coded three), label style and sort
order.

**The lesson worth keeping** is about method, not `.bst`: the vendoring claim was
recalled from memory and written into a merged doc without checking the witness's
own `alpha.bst`. Two minutes of `grep` refuted it. Verify the file, not the
recollection.

**Two candidate resolutions.** They are not exclusive; (b) is a strict subset of
what (a) delivers.

**(a) Port upstream `brucemiller/LaTeXML#1955`, "[WiP] BibTeX emulation".** A
real BibTeX virtual machine: `LaTeXML/BibTeX/BibStyle/` parses the `.bst`
(`StyCommand`, `StyString`, `Precompiled`), `LaTeXML/BibTeX/Runtime/` executes it
(`Builtins`, `Names`, `Strings`, `Buffer`, `Entry`). **34 files, +4224/-801**,
open since **2022-10-05**, itself a refactor of tkw1536's closed #1231. Buys the
whole thing at once — correct labels, correct sort order, correct field
selection (which is what OXIDIZED_DESIGN #73/#74 currently approximate by hand),
*and* the vendored preamble. Costs: it is upstream **WiP and unmerged**, so
porting it means tracking a moving, unreviewed target; its author's own summary
says "I've undoubtedly broken a few features that had been working".

**(b) Per-style `_bst.rs` bindings**, mirroring the `_cls.rs` / `_sty.rs`
convention already in the tree. A binding per common style (`plain`, `unsrt`,
`alpha`, `abbrv`, `amsplain`, `amsalpha`, `splncs04`, `IEEEtran`) supplying its
preamble definitions and its label/sort rules. Cheaper per increment, matches how
the rest of the port already handles vendor files, and can land style-by-style
against measured witnesses. Costs: the long tail of `.bst` files is unbounded,
and hand-written rules will drift from what the real `.bst` computes.

**Prerequisite either way — the `.bbl` question.** arXiv's convention means most
papers ship a `.bbl`, which we already prefer; `.bst` support only matters for
sources with `.bib` + `.bst` and **no** `.bbl`. Before committing to (a) or (b),
**measure how large that population actually is** in a full-corpus run. The
existing deferral names witness **2605.16562** (LNCS, `splncs04.bst`); one
witness is not a business case. That measurement is cheap and is the honest first
step.

**Do NOT start** either resolution without that measurement and an explicit
decision — this row exists to stop the next `\Dbar` from being patched in
isolation without the context above.

### R9-MSC — `mathscinet.sty` binding — ✅ DONE 2026-07-27 (PRs #415 + #419)

Closed. `latexml_package/src/package/mathscinet_sty.rs` binds the AMS v1.05
package (amsrefs bundle), and the `\cprime`/`\Cprime`/`\cdprime`/`\Cdprime`
family that used to sit always-on in `latex_constructs.rs` — the LaTeX **kernel**
file, which must track `latex_constructs.pool.ltxml` — was moved out and then
**deleted**: it is package vocabulary, a paper gets it by loading the package (or
`amsrefs`, `amsrefs.sty` L217) or via its own `.bib` `@preamble`. The three
witnesses that opened this row all load the package by name (2508.13753 L7,
2508.20226 L3, 2509.07628 L13), refuting the `cyracc.def` justification the old
comment carried. Full record: divergence **#78**; the corpus tables and the
retracted "the stub stays" verdict are in
[`BIBLIOGRAPHY_WORKLIST.md`](parity/BIBLIOGRAPHY_WORKLIST.md).

**The one thing to carry forward:** this row said "witness 2605.11579's
`undefined:\Dbar` is correct and stays — do not use that witness to measure this
work." The *reasoning* is still right (that paper loads no package and
`alpha.bst` has zero `Dbar`, so pdflatex fails too) but the witness no longer
**shows** the error: since divergence **#80** only cited entries are digested,
and its `KacNilpotentorbits` entry (`biblo.bib` L2059) is uncited. Still do not
measure with it — now because it is silent, not because it is parity.

### R6 — `ltx_env_<name>` env-markup class — PLANNED, needs its own branch (churns every test XML)
**User-requested generic enhancement** (2026-06-27): tag environment wrapper markup
with `class="ltx_env_<name>"` so custom/minipage-like envs (e.g. `SideBySideExample`)
become responsively styleable in CSS instead of fixed-width minipages. **MUST be on a
dedicated branch** — it changes nearly every test XML (additive class on every env
element), so the golden-suite update is large and must be done in isolation.
Two implementations, same markup outcome:
- **Binding side (`DefEnvironment!`):** the constructor guarantees exactly one element,
  so unconditionally add `ltx_env_<name>` (via an `@ADDCLASS`/`add_class` after the
  begin constructor opens). Applies to ALL DefEnvironments (`figure`, `table`,
  `theorem`, `minipage`, …) — user chose full scope.
- **Raw side (`\newenvironment`/`\renewenvironment`):** arm at env start; at `\begin`
  construction record `{name, anchor = globally-unique gid of current node, mark}`; at
  `\end` afterConstruct, if EXACTLY ONE element was deposited under the anchor since
  the mark → tag it; zero (font/text-only) or >1 (siblings, e.g. SideBySideExample's
  parboxes) → nothing. **Needs a globally-unique monotonic node gid** (verify/ add;
  `record_node_ids` exists but is xml:id-oriented).
- **SideBySideExample:** keep the working `fancyvrb-ex` raw-load (correct source+result)
  + drive responsive layout from the resulting `ltx_minipage`/`ltx_env_*` hooks in
  `ar5iv.css`; do NOT re-implement the verbatim+render dual capture.

### (not ranked) `\gls`/`\acrshort` in MATH mode, 1705.10306 — PARITY, blocked on unrunnable Perl — do not chase
293 errors `ltx:XMTok isn't allowed in <ltx:glossaryref>`: a glossary command in
math mode digests the link display text (#3, the literal acronym term) as math →
bare per-letter `<XMTok>`, which the `glossaryref` content model rejects.
**Source-confirmed 2026-06-27 that this is most likely PARITY (NOT a Rust-only
gap — the cortex "Perl 1" is stale/unreliable, per `use-cortex-for-parity-work`):**
- Perl `Stomach.pm::enterHorizontal` (L422-434) is a **no-op in math** (`$mode
  =~ /math$/ => {}`) — Rust's `enter_horizontal` matches faithfully. So the
  `enterHorizontal => 1` on the shared `\lx@glossaries@gls@link` constructor does
  NOT switch #3 to text in math in EITHER engine.
- BOTH engines raw-load the SAME `glossaries.sty` (`InputDefinitions(noltxml=>1)`)
  with the SAME override constructor → both digest #3 in the ambient math mode →
  both produce `glossaryref > XMTok` → both hit the same schema rejection.
- `\ref`/`\cite` in math do NOT error (verified) — their content is STRUCTURED
  (bibref / ref-number), not a literal term; only `\gls`/`\acrshort` emit raw
  letter-XMToks. So glossaryref is specific, but the mechanism is shared with Perl.
- **The earlier "Perl raw-loads glossaries.sty and typesets as TEXT" hypothesis is
  weakened:** Rust raw-loads the identical `.sty`, so if it typeset the term as
  text, Rust would too. It doesn't (output: italic letter-XMToks) → so the `.sty`
  display chain does NOT force text in math.
**Perl confirmed UNRUNNABLE here (2026-06-27):** `latexml glx.tex` → `Fatal:terminate`
in `expl3-code.tex` (l3kernel) at 150 s — glossaries pulls in expl3 which is
pathologically slow in Perl 0.8.8 on this host; cannot capture ground truth.
**Fixing is therefore deferred as a likely non-bug.** If pursued, it parallels the
figure_mixed_content surpass-Perl pattern (a monotonic schema expansion to accept
the math content the builder already produces) — BUT the correct structure is
genuinely uncertain without Perl (XMTok directly? XMText-wrapped? operator-token
for the `\DeclareMathOperator` case? text PCDATA?), and there is **no precedent**
for `XMTok` in any inline element's model, so a speculative change risks an
unfaithful divergence. Repro + full notes:
`docs/reproducers/glossaryref_math_xmtok.tex`.

## Parked families — pointers, not content

Each outgrew this file and now lives on its own. Read the doc before starting;
several carry explicit "do NOT start" directives.

| family | rank | doc |
|---|---|---|
| Bibliography targets + MakeBibliography re-port | R5 | [`parity/BIBLIOGRAPHY_WORKLIST.md`](parity/BIBLIOGRAPHY_WORKLIST.md) |
| Beyond-Perl performance levers BP-1…BP-6 | R7 | [`performance/BEYOND_PERL_LEVERS.md`](performance/BEYOND_PERL_LEVERS.md) |
| Content-MathML / math-parser gaps | R8 | [`math/CONTENT_MATHML_GAPS.md`](math/CONTENT_MATHML_GAPS.md) |
| Deep deferred families (`.bst`, xy-pic, mode-frame, …) | R9 | [`parity/DEFERRED_FAMILIES.md`](parity/DEFERRED_FAMILIES.md) |
| Two-pass streaming split | deferred | [`performance/STREAMING_POST_DESIGN_2026-07-06.md`](performance/STREAMING_POST_DESIGN_2026-07-06.md) |

## Reference (stable — not active work)

### Engine file open gaps (MINOR, demand-driven)
- `tex_box.rs` box-dimension edges; `tex_fonts.rs` `\fontdimen` array + per-font
  `\hyphenchar`; `tex_tables.rs` padding CSS (XSLT concern).
- **Document-builder block/paragraph auto-wrap of inline content** (core,
  broad/risky family — two witnesses):
  - **`\fcolorbox` inline paragraph-grouping**: an inline `\fcolorbox`
    mid-paragraph — Perl breaks the `<p>` (its `internal_vertical` block ends
    it), Rust keeps it inline. SAME flags on both; Rust's inline reading
    arguably matches real LaTeX's `\mbox`-based `\fcolorbox`. (`\colorbox`
    matches.)
  - **bare `\includegraphics` run in a figure** (witness 1108.0198, found
    2026-06-21 via skeleton diff — a clean, error-free reproducer): a
    `\begin{figure*}` with several consecutive `\includegraphics` (no blank
    line) — Perl wraps the inline run in a `<ltx:block>` (`figure > tags >
    block > graphics×N`), Rust emits the graphics bare (`figure > graphics×N`).
    Rust is error-clean and schema-valid. **Re-witnessed + root-confirmed
    2026-06-27** (0704.0001, 0704.0017 via the corrected structural diff): NOT
    merely cosmetic — the panel `<graphics>` WIDTHS also diverge (Rust 303.5pt vs
    Perl 241.5pt, ~1.257×), so figure sizing is visibly affected. Root: Perl's
    `arrange_panels_and_breaks` (`latex_constructs.pool.ltxml:3229-3295`) does a
    full box-metric panel layout — it inserts `<break class="ltx_break">` and wraps
    panels using `getNodeBox($child)->getWidth` vs `float_width`; Rust's
    counterpart (`latex_constructs.rs:1784-1869`) is explicitly **"Simplified: mark
    panel children with the class"** and skips the break/block arrangement. A
    faithful port DEPENDS on matching box widths → the deep box session (sibling of
    the `\resizebox` panel-width item below), not a loop-tick fix.
- **`\resizebox` panel scale-VALUE divergence**: in `complex/figure_mixed_content`
  two panels get a different computed natural width (xscale 1.13 vs 0.88). The
  construct in ISOLATION matches exactly (both xscale=1.9685); the divergence
  only appears inside the paper's `\footnotesize` + `table*` + `\subfloat` panel
  context → a font-size/box-context interaction. Scale *formatting* (%.15g) is
  already Perl-faithful (`551c5286ba`); missing-image candidates too
  (`64dd30b284`). Deep box-metric; for the focused box session.
- **~72-CS Perl-only long tail** (from the archived LoadFormat audit): misc
  atomics (`\@charlb`, point-size CSes, `\batchmode`, …) Perl defines, Rust does
  not. Investigate a CS only when a real paper witnesses it; refresh the CS-name
  diff before quoting counts (predates the BibTeX port).

### Primitive layer — AUDITED FAITHFUL (2026-06-20)
Probe-based Rust-vs-Perl audit found the core primitive layer byte-identical
(arithmetic, dimensions, glue, conditionals, string/token, case tables). Don't
re-audit without a witnessing paper. Shared-with-Perl quirks (NOT Rust bugs):
`\numexpr` divideround round-half-toward-+∞ (KNOWN_PERL_ERRORS #33); `\the\skip`
drops stretch/shrink to bare pt.

### Permanent ignores
- **Out-of-scope**: ns1–ns5 (`52_namespace`, no DTD support); 2402.03300,
  2410.10068, 2511.03798 (Perl also fails).
- **Rust supersedes Perl**: `1207.6068`, `0909.3444`, + 40 more in
  `memory/project_rust_supersedes_perl.md`.
- **2026-07-20 ar5iv sprint (PR #323) residuals — do not re-mine.** Its three
  ar5iv leftovers all resolve parity-or-Rust-better, none Rust-only
  (`AR5IV_DIAGNOSTICS.md`); its TL2026 dump-gate scrap closed 2026-07-23 and
  was re-confirmed on `main` 2026-07-25 (0 errors on both inits inside
  `ghcr.io/tkw1536/texlive-docker:2026`; 2026 is in the release window). Both
  in `archive/SYNC_SESSIONS_2026-07.md`.
- **BibTeX**: `BibTeX.pool.ltxml` ported (Phases 1–8; remaining B1–B6 polish in
  `BIBTEX_PORT_PLAN.md`). `--nobibtex` is opt-out, not default.

### Tikz known diffs vs Perl
`foreignObject` transform; arrow-tip path data; SVG viewBox/width; matrix
`<svg:g class="ltx_tikzmatrix">` vs inline-blocks; **bare `svg:g` in `<ltx:block>`**
(tikz-cd) trips a core-XML validity error but post-processing recovers (witness
2006.12702) — Rust-only, low priority (output recovered).

### Graphics renderer chain (subprocess-only; LANDED)
PDF→PNG `mutool draw`→`pdftocairo`→`convert+gs`; PDF→SVG `mutool convert`→
`pdftocairo`→(raster PNG fallback). EPS/PS→`gs` direct→`convert+gs`. Subprocess
`exec` (no GPL linking). Apt: `poppler-utils` (req), `mupdf-tools` (rec),
`imagemagick+ghostscript`. A heavyweight inkscape third resort for PDF→SVG was
removed 2026-06-29 (GTK stack, 20–40× slower, timeout-prone, no coverage over the
raster fallback).

### Other tracks (separate docs)
- Performance: `PERFORMANCE.md` (P1 math/large-doc open; P2 allocation partial).
- Release gates: `RELEASE_CRITERIA.md`. Releasing: `RELEASING.md`.
- **BibTeX (plan archived 2026-07-02 →
  [`archive/BIBTEX_PORT_PLAN_2026-06-20.md`](archive/BIBTEX_PORT_PLAN_2026-06-20.md)):**
  Phases 1–8 shipped; live residuals = the Phase 4–5 field-handler/MR-Zbl
  long tail, divergences B1–B6 noted in `bibtex.rs`, and the deferred
  **native `.bst` interpretation** (witness 2605.16562, `f65cf7d6dc`) —
  demand-driven, pick up on corpus evidence.
- Completed missions (archived): strict-LoadFormat dump parity, Marpa ASF
  migration, distribution-readiness, the 500K/1M warning-corpus mission, the
  diagnostic-message faithfulness pass (2026-06-20), and the upstream-sync
  PR translation U1–U11 (2026-06-26) — see `docs/archive/` and `git log`.
