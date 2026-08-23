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
   (issue closed; that screenshot was out of scope), a
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

*Last compaction: 2026-08-18 — 1462 → ~890 lines. Completed/solved sections
(streaming CORE+POST, the font-FAMILY/`\fnum@`/`alpha` current-status fixes, the
`--format=xml` non-bug, fancyvrb/robust, R4, R9-MSC, pMML F17) removed or lifted to
`SYNC_SESSIONS_2026-08.md`. Prior: 2026-07-25 — 1979 → ~500 lines, 23 sections to
`SYNC_SESSIONS_2026-07.md`; four standing families extracted (see Parked families).*

## Ranked worklist — start here

Ordered by: **does it reproduce today** → **is a real user affected** → **is it
unblocked** → **effort**. R1 is a review nudge (no code); R2's cheap half landed
2026-07-29 and what remains of it, like the R5+ rows, needs a session of its own.
Re-verify a row before planning on it (rule 1).

| # | item | state | size | detail |
|---|---|---|---|---|
| **R1** | Upstream `brucemiller/LaTeXML#2852` — subfile `\documentclass` options | **OPEN upstream**, ours merged as #310; **CI all-green + mergeable, re-verified 2026-07-29** | nothing left but a review nudge — no code, no automatable step | Open items |
| **R2** | `--preload=<cls>` trips the LaTeX hook stack (`Extra \PopDefaultHookLabel`) | **OPEN**, re-verified 2026-07-29 (1 error with `--preload=article.cls`, 0 without) | hook half is **not** small: five measured dead ends, `(c)` now collapsed into the rejected `(a)`, and any real fix is TeX-side | Open items |
| **R3** | **Bibliography-absence campaign** (PR #444) — **16 fixes landed**, **291 of the 533** known articles recovered / 20 338 entries, re-verified by reconversion. **242 still empty, all characterized** — plan R3a-R3g below. Corpus scope 50 777 | **R3a next** | per-item | [`BIB_ABSENCE_AUDIT_2026-07-29.md`](parity/BIB_ABSENCE_AUDIT_2026-07-29.md), [`RESIDUAL.md`](parity/bib_absence_2026-07-29/RESIDUAL.md) |
| **R5** | Bibliography targets + MakeBibliography re-port | **the re-port is DONE** — items 1 and 3 landed 2026-07-26/27 (recursive BibTeX session on the LIVE core state, the 727-line string route deleted, the 13-field digest whitelist gone: the `\bib@field@default@*` name sets match Perl exactly, 45 each; `.bib`-as-DATA closed as divergences #74/#78/#79/**#80**), and **item 2 landed 2026-07-29** (citestyle `AY`, short-name `{ay}`, collating `unisort`, format-order NUMBER). Remaining: the missing-references target list | **targets only** | [`BIBLIOGRAPHY_WORKLIST.md`](parity/BIBLIOGRAPHY_WORKLIST.md) |
| **R3** | Presentation-MathML **F5** Linebreaker (F17 closed — see archive) | **F5 alone remains** — a full line-breaker feature gap needing a port-or-drop scope decision. A math-parser `scriptpos` bug and a FUNCTION-APPLICATION over-insertion witness found en route are **other rows** | **family** — scope decision | Open items |
| **R6** | `ltx_env_<name>` env-markup class | user-requested, **PHASE 2 — do NOT start yet** (user directive 2026-07-29) | medium code, **large golden churn** → own branch | Open items |
| **R7** | Beyond-Perl performance levers BP-1…BP-6 | POST-RELEASE; internal order BP-2 → BP-3 → BP-1 | **family** | [`BEYOND_PERL_LEVERS.md`](performance/BEYOND_PERL_LEVERS.md) |
| **R8** | Content-MathML / math-parser gaps | **deferred by user directive 2026-06-20** | **family** — do not pick off in isolation | [`CONTENT_MATHML_GAPS.md`](math/CONTENT_MATHML_GAPS.md) |
| **R9** | Deep deferred families (`.bst`, xy-pic, mode-frame, …) | parked; several carry explicit "do NOT start". The `.bst` row's "`.bst` files *vendor macro definitions*" premise was **RETRACTED 2026-07-27** (`alpha.bst` has zero `Dbar`; the macro is `mathscinet.sty`'s) — it survives on label style / sort order / **field selection**, and the prerequisite is a corpus measurement of the `.bib`+`.bst`-with-no-`.bbl` population | **family** | [`DEFERRED_FAMILIES.md`](parity/DEFERRED_FAMILIES.md), and R9-BST below |
| — | `\gls`/`\acrshort` in math mode (1705.10306) | **PARITY, blocked** on unrunnable Perl | — | do not chase; Open items |

### R3 mini plan — the remaining bibliography failures

Detail and per-paper rows: [`RESIDUAL.md`](parity/bib_absence_2026-07-29/RESIDUAL.md).
Red/green for every item is the bibliography length (0 = red); reconvert with
`tools/bib_recheck.sh` and never trust a first-error label alone — 96 of the 123
classes are singletons and the first error is often incidental.

| # | item | papers | state |
|---|---|---|---|
| ~~R3a~~ **19 of 29 LANDED** (`\DeclareCiteCommand` defines its command; `\addbibresource` harvested from a shipped class; **OmniBus now autoloads biblatex from `\addbibresource`/`\printbibliography`** — 7 papers, 548 entries, for classes that load biblatex themselves but have no binding, e.g. now-journal.cls building the package name from macros). Four of those 7 had an unrelated first error, so first-error clustering had scattered one cause across four buckets | **biblatex, document-level** | 10 left |
| **R3b** | **No diagnostic at all** — silent loss, complete document. **13 of 31 LANDED**: `\nocite{*}` now includes the whole library as bibtex does (7), a native `xpatch.sty` binding stopped an expl3 sentinel-delimited scan from eating the document to EOF (5 — audit F12; the 10 xpatch papers' other 5 causes are each unrelated and listed there), and **2606.01320's `\pretocmd{\cite}{\stepcounter{cite}}` now assigns through the core `\cite` CS-lock** (2026-08-05; etoolbox hooks are non-destructive, so they open a scoped unlock window — divergence #88; guard `06_cluster_bibliography::etoolbox_pretocmd_assigns_through_cite_lock`). Of the rest, **14 have real `\cite` calls** (chase these) and **4 cite nothing at all** (0 entries is correct — exclude). One is characterized-but-unfixed with the remedy written down in `RESIDUAL.md`: 2605.08378 (submission ships no `PurdueThesis.cls`). **Batch re-triaged 2026-08-17** (27 no-diagnostic candidates, fresh worker): **4 already fixed** (2606.23302, 2606.30032, 2605.28547, 2605.25157), **2 orphan-`.bbl`** (commented/unreferenced `\bibliography` → exclude: 2606.09394, 2605.03978, 2606.31667), and **one genuine Rust-only fix landed** — **2606.11493 (0→31):** a `comment.sty` `\begin{comment}…text.\end{comment}` with the end MID-LINE overran to EOF and swallowed the inline `thebibliography`; fixed by detecting `\end{comment}` mid-line (surpass-Perl #133, guard `comment_midline_end_keeps_bibliography`). **Second Rust-only fix (2026-08-18): 2605.19817 (0→39)** — a physics.sty `\qty(\frac12 D_{(\alpha}…)` with a `(` inside a `{…}` subscript group ran the delimiter reader away to EOF, swallowing the `\section` + bibliography into the math (58 `malformed:ltx`); `phys_read_arg` now matches `(`/`)` only at brace-level 0 (guard `physics_qty_braced_paren_does_not_run_away`). **2606.01320 CONFIRMED SHARED** (natbib `\cite` not `\pretocmd`-patchable in either engine — its `\ifnum\value{cite}>0` gate stays closed). Remaining chase candidates: 2605.14990, 2606.05629 (a math-in-body silent digestion drop, not yet root-caused), 2606.10056, 2606.17491, 2606.00231, 2605.29754 | **~6 left** | 2 fixes landed |
| **R3c** | `\ce`/`\ch` inside a `p{}` column leaks a mode → `\@end@tabular` cannot close. 7-line repro in [`repros/f8_ce_in_p_column/`](parity/bib_absence_2026-07-29/repros/f8_ce_in_p_column/). **CONFIRMED SHARED (2026-08-04, re-confirmed 2026-08-17):** `latexml --includestyles` raw-loads mhchem and gives the byte-identical 7-error cascade (bare Perl was a false-clean — it can't find raw mhchem without `--includestyles`, so `\ce` was inert; a subagent re-hit this trap and mis-called it Rust-only). Root cause isolated to expl3 **`\regex_replace_all`** in a paragraph column — `regex_replace_in_p_column.tex` reproduces it with NO chem package on plain binary AND bare Perl. Generalizes past mhchem: **chemformula `\ch` triggers the identical cascade** (witness 2606.04125). **Surpass-only** — mode-robust `\ce` or an mhchem binding, needs the 3 qualifying tests + user escalation; deprioritized | **7** | ✅ triaged → surpass-only |
| **R3d** | **PARTLY FIXED — 14 of 28, 961 entries.** A `&` inside a **delimiter-fenced** macro argument split the alignment row and truncated the document. `tex.web` §394 `macro_call` disables tab marks while scanning parameters; we — and Perl, which raises the identical error — did not. `SuppressedTabMarks` armed inside alignments on physics.sty's `phys_read_arg` fixes the 16 `\mqty` users. The general `Parameters::read_arguments` site is TeX-correct but regresses 5 tests (`cells_test` 17 errors, `numprints_test`, `xytest_test`, `consort_flowchart_test`, `unit_tests_by_silviu_test`) because that path also reads alignment cell content — needs a parameter-scan-vs-cell-read distinction. Divergence #90 | **12 left** | needs the distinction |
| **R3e** | `N bibentries, 0 cited` — citation records never attach, so an empty References heading renders. The raw-`\cite`-clobber half landed as #88. **Batch-triaged (2026-08-17):** of 13 `0cited/noentries` residuals, most are SHARED (missing vendor `.cls`/`.sty` — WSC `\WSCpagesetup`, `\ccode`, `\OneAndAHalfSpacedXI`; leading-space `\input`; `underscore.sty [strings]`; all fail identically in same-host Perl) or already fixed (2605.15421 by #641, re-verified 0→101). **One genuine Rust-only fix landed: 2606.03480 (0→29).** Its active-`@` shorthand leaked catcode into the reused `.bib` post-session and corrupted the `\begin{bibtex@bibliography}` wrapper → `readBalanced ran out of input`; the fix resets `@` to its standard catcode per `.bib` (matching Perl's fresh `catcodes=>'standard'` state). Guard `active_at_catcode_does_not_corrupt_bibtex_wrapper` | **1 fixed, rest shared** | genuine-Rust-only fixed |
| **R3f** | Ships **no** bibliography source (`.bbl`/`.bib`/`thebibliography`/biblatex). Verify each against its own PDF, then exclude — the PDF says what the *toolchain* produced, not what we may emit ([`README_residual_triage.md`](parity/bib_absence_2026-07-29/repros/README_residual_triage.md)) | **17** | triage to close |
| **R3g** | amsrefs bare `\begin{biblist}` with no `{bibdiv}` wrapper → `malformed:ltx:biblist`. Two dead ends recorded (naive `auto_open`, and a conditional wrapper) — needs the `BACKMATTER_ELEMENT` route | **4** | dead ends recorded |
| — | No output at all (`Stomach:Recursion`, `Timeout`, `MemoryBudget`) → the general **fatal-mining** mission, not bibliography work | 9 | routed away |
| — | Truncation residue behind R3d, plus a ~96-strong singleton tail of per-template defects | rest | work after R3a-R3g |


## Current status

- **2026-08-02 (latest) — rc4-recut full rerun of sandbox-arxiv-2605+2606 (60,505
  docs): success at parity, every recorded fatal-witness set improved, one new
  REAL cluster (a fleet-only libxml panic ×3) and one REAL seed (Perl-0-vs-
  Rust-101 error floods).** Fleet: fresh `maxperf-cortex` worker on main
  `69ec59620f`, 72 workers, both corpora drained in ~1 h. Headline (2605 /
  2606): no_problem 6,078/6,359 · warning 19,744/19,724 · error 3,991/4,102 ·
  fatal 266/241. The huge `no_problem→warning` shift (2605: 16,791→6,078 np,
  9,043→19,744 warn vs the prior run) is **#484's lossless diagnostic tally
  recording mostly math warnings that were previously lost — visibility, not
  regression** (owner-confirmed; do not chase).
  **Witness sets** (per-id via `api/corpus/…/document/`): the 17
  `Fatal:TooManyErrors` papers are now ALL plain `error` at 1–10 errors; the
  bxcoloremoji 18/18 non-fatal (2605.14271 at its recorded post-fix 12); the
  `\lx@tag@intags` pair resolved (2605.12842 warning; 2605.01731 one unrelated
  `undefined:\nequiv`); the flip-4 all `warning`/0-error.
  **Fatal clusters** (sampled 8 each, `parity_check.sh` 180 s, protocol
  cluster-classify):
  | cluster | size | verdict |
  |---|---|---|
  | `panic:caught` | 3 | **FIXED 2026-08-03 (PR #491)** — pooled-worker use-after-free: the math parser's `PENDING_DISCARDS` was not drained on the resource-fatal abort path, so the next paper on that thread walked handles into the predecessor's freed document (innocent papers; poison is the predecessor's). Drain-on-abort + wrapper-only stale sweep (`sweep_stale_math_state`). Guard `latexml_math_parser/src/data.rs::stale_handles_from_a_dead_document_are_swept_without_panic`; full conclusion in `archive/SYNC_SESSIONS_2026-08.md`. Same-class residuals (ALIGNING_NODE, `Stored::Alignment` cells, `STAGED_SNAPSHOTS`) recorded in git. |
  | `TooManyErrors:MaxLimit(100)` | 117 | **MIXED, REAL seed** — 4/8 REAL incl. **2605.22927 and 2606.11121: Perl 0 errors vs Rust 101-capped flood** (also 2606.01136 P63/R101, 2605.10685 P7/R101); 3/8 both-capped (uncomparable), 1 Rust-win. Root-cause the two P=0 witnesses first; cap-bucket ⇒ sub-group by first-error class before scoping. |
  | `Stomach:Recursion` | 55 | **MIXED** — 3/8 REAL-by-count (2605.17696 R144/P56, 2606.05321 R35/P15, 2606.08524 R94/P50), 2 Rust-wins, 1 Perl-capped, 1 shared-timeout, 1 clean. NOTE: none hit the recursion *fatal* locally — the fleet's memory-governed guard levels are part of the trigger; sub-group by next-error class. |
  | `Timeout:PushbackLimit` | 120 | **Environmental/policy** — 3/8 both-clean locally (same binary!), 3 Rust-wins/shared, 1 trivial R2-vs-P1. Fleet budget caps, not conversion bugs. |
  | `Timeout:TokenLimit` | 88 | **Not a bug cluster** — 7/8 Rust-wins (P 3–101+ vs R 0–2) but 6/8 with local `RUST_TIMEOUT` partials: legitimately heavy papers, Rust cleaner-but-slow; perf domain. |
  | `cortex:never_completed` | 26 | **Fleet-environmental** — 0/8 REAL, 5/8 Rust-wins, 2 both-clean; the known pattern. |
  Rust-wins recorded this round (do NOT "fix"): 2606.31920, 2605.27614,
  2606.28450, 2605.16450, 2605.03102, 2606.28253, 2605.16752, 2606.19764,
  2605.22335, 2606.26525, 2606.15193, 2606.25331, 2606.31502, 2606.28434,
  2605.26647, 2605.26895, 2605.15001, 2606.18685.
  **Actionable queue from this round**: (1) the libxml panic teardown (above);
  (2) `2605.22927`/`2606.11121` Perl-0-vs-Rust-101 floods; (3) the
  Stomach:Recursion REAL trio. Everything else is policy/perf/upstream.

### Session logs (2026-06-22 … 2026-07-27) — ARCHIVED

Completed "Landed this session" entries, the slowest-100 batch triage, the
finished upstream-sync U1–U11 mission log, and the mined-out methodology
history now live in the dated session archives:

- [`archive/SYNC_SESSIONS_2026-08.md`](archive/SYNC_SESSIONS_2026-08.md) —
  the 2026-07-09 … 07-27 window (lifted 2026-08-14): the `unexpected:fi`/chardef
  `\meaning` fix, spconf keywords, the xparse `\c`-cedilla clobber, kernel
  autoload-on-undefined, the `.bib`-as-DATA family close, silence.sty/bare-`&`
  bib fields, the 07-26 resilience-mining + email-reported missing-References
  clusters, siunitx v3, the 0.7.4-rc4 crates.io tag, and the `\AtBeginDocument`
  #2846 re-port.
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

### CSS themes — `ar5iv.css` is the active surface; base `LaTeXML.css` is upstream

**Policy.** In latexml-oxide we actively develop **`ar5iv.css`** only (repo
`~/git/ar5iv-css`, mirror workflow: off `main`, rebuild `dist/`, CHANGELOG). The
base **`LaTeXML.css`** (`latexml_post/resources/CSS/LaTeXML.css`) is a faithful
copy of Perl LaTeXML's default theme — its rendering behaviour and bugs route
**upstream to `brucemiller/LaTeXML`**, not here. When a user reports a
rendering/CSS complaint, first establish which theme; base-CSS issues → upstream.

**Plan (not yet scheduled).** Make `ar5iv.css` the **default** theme (currently the
base `LaTeXML.css` is default). Tracks the reality that the base theme is upstream
and unmaintained here.

**Absolute-vs-responsive image sizing — data-model gap (witness #721, ar5iv#83).**
`\includegraphics[width=7in]` inside a `minipage{.5\textwidth}` renders small, not
7in. The engine is faithful: it emits the absolute width as the `<img>` attribute
(`width="698"` ≈ 7in), **byte-identical to Perl 0.8.8** — no engine clamp. The
clamp is purely CSS and differs by theme: base `LaTeXML.css:639`
`.ltx_minipage > .ltx_graphics { max-width:100% }` caps to the container (PARITY —
same rule at Perl `LaTeXML.css:573`); `ar5iv.css` goes further and **fluidizes**
layout-nested images (`ar5iv.css:1681-1701` orientation rules set `width:auto` +
max in `--main-width:52rem`; `2291` re-declares the minipage cap), *discarding* the
absolute width **by design** (responsive column-fill). So neither engine, nor the
default nor the ar5iv theme, honours the 7in for a nested image. The durable fix is
**NOT CSS tuning** — it is preserving the sizing **intent** the pipeline currently
flattens (`to_bp`, `latexml_core/src/util/image.rs:186`, collapses absolute `7in`
and relative `\textwidth` to the same `pt`). Mark absolute-authored vs
relative/natural widths (and, longer-term, panel vs sized-box minipages) so the
theme can *deterministically* honour absolute intents (7in fits `--main-width`)
instead of guessing by nesting depth — the fragile negation-selector heuristic the
ar5iv authors flag at `ar5iv.css:1678-1679`. Trackers: upstream data-model discussion
`brucemiller/LaTeXML#1797` (model + styling for figures in minipages); theme-side
`ar5iv#83` and `dginev/ar5iv-css#38` (side-by-side minipages).

### Algorithm markup + CSS unification — SCHEDULED 0.7.7 (deferred, user-directed 2026-08-22)

**Goal.** Unify the markup emitted for **all** algorithm kinds (algorithmic,
algorithmicx/algpseudocode + the language variants, algorithm2e) onto one shared
vocabulary and marker class, and derive **generic CSS that works in BOTH
`LaTeXML.css` and `ar5iv.css`** — one algorithm-layout rule set, not per-theme
per-package selectors. Design plan: [`parity/ALGORITHM_RENDERING.md`](parity/ALGORITHM_RENDERING.md)
§"Markup unification".

**Part 2 LANDED (2026-08-22), independent of the unification goal** — do not re-open these
as part of it (details + guards in ALGORITHM_RENDERING.md "Landed in Part 2"): uniform
`\NlSty`-bold line numbers, ruled-family caption-at-top (#153), `\hbox to \hsize` leader
separators → `width:100%` (#152, witness 1510.02728), `\Comment*[r]` inline side-comment,
frontmatter dedup (#154), and the CSS batch (algorithm phantom vertical scrollbar
`overflow-y:hidden` witness 2002.09766, wrapfig overlap 2605.03143, side-by-side minipage
width-strip 2402.19043, framed-lstlisting page-scroll 2512.24601 — ar5iv.css + embedded
`LaTeXML.css` mirror).

**Why it is the right fix (not a per-theme CSS patch).** The algorithm-layout rule
(`white-space:nowrap`, so the pretty-printer's newlines between a line's number tag
and its statement do not render as breaks) is currently keyed on the **wrapper
classes** `.ltx_float_algorithm` / `.ltx_algorithm`. An algorithm authored **outside
an `algorithm` float** — e.g. the popular `breakablealgorithm` recipe, which wraps
`\begin{algorithmic}` in a bare `center` — emits a bare `.ltx_listing` with **neither
wrapper class**, so it falls through to code's `white-space:pre` and renders broken
(numbers stacked above wildly-spaced content). This is the commonly-reported
"algorithm displayed wrongly" class: html_feedback #6080 (2602.20153), #6236
(2512.24601), #5492 (2511.21969), #3450 (2406.08374); **witness this review: arXiv
2408.07803** (html_feedback #1998), whose caption now compiles (via the `\fname@`
fix, OXIDIZED #150) exposing the body breakage.

**Why it can't be a one-line CSS discriminator.** A numbered **code** `lstlisting`
uses the *same* `.ltx_tag_listingline` / `.ltx_lst_numbers_left` as an algorithm, so
`:has(.ltx_tag_listingline)` would wrongly `nowrap` code and destroy its indentation
(#6632). `minted` DOES carry `.ltx_lstlisting` (verified), so `.ltx_listing:not(.ltx_lstlisting)`
is *nearly* safe but still fragile against future bare-`.ltx_listing` producers. The
robust fix is the **shared markup class**: give every algorithm listing a positive
marker (regardless of surrounding env), then ONE generic rule targets it in both
stylesheets. That is why markup unification and generic CSS are the SAME work item.

**Scope note.** Faithful-translation caveat: any new marker class must be justified
against Perl (Perl's algorithmic listing carries no such class today) — see the
`surpass-perl` skill protocol. Defer until the 0.7.6 release lands. Related open theme
item above: side-by-side minipages (`dginev/ar5iv-css#38`, witness 2402.19043).

### tcolorbox / framed listings render poorly (widths + font size) — PARTIALLY ADDRESSED (user-flagged 2026-08-22)

- **Framed lstlisting page-scroll — FIXED (Part 2, CSS).** The reported 2512.24601 defect
  was NOT a tcolorbox: a plain framed `lstlisting` overflowed and scrolled the WHOLE page.
  `.ltx_lstlisting { display:block; max-width:100%; overflow-x:auto; box-sizing:border-box }`
  (ar5iv.css + embedded `LaTeXML.css` mirror) confines the scroll to the box.
- **Generic tcolorbox width/oversized-font — DEFERRED, no current witness.** A
  `\tcblisting`/`\newtcblisting` code box may still size/scale poorly; the listings dialect
  is out of scope to change (see the unification note above), so this wants a dedicated pass
  over the `tcolorbox` box model + the ar5iv `.ltx_lstlisting` width/font rules. Not started.

### Frontmatter + footnote rendering residuals (user-flagged 2026-08-22)

From the manual review, witness arXiv 2511.21969:
- **Duplicated abstract heading — FIXED (Part 2, OXIDIZED #154).** The nested `{abstract}`
  env pushed a second `ltx:abstract`; replaceable-frontmatter dedup
  (`base_utilities.rs` `REPLACEABLE_FRONTMATTER_TAGS`) now keeps one. Guard
  `cluster_frontmatter_replaceable_dedup`. The same fix resolves 2002.09766's duplicated
  `<title>`/author block (appendix `\icmltitle`).
- **"Authors missing" — NOT a bug.** The preview was built from the wrong source file:
  `main-ieee.tex` has its authors commented out; the toplevel is `main-white-paper.tex`.
  No code change.
- **Footnote side-margin overlap on wide displays (same witness) — DEFERRED:** footnotes 3
  and 4 overlap in the ar5iv side-margin rendering at wide viewports — a CSS margin-note
  layout concern (`ar5iv-css`), not core XML.

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
  `archive/MATHML_POST_LINE_AUDIT_2026-07-05.md`). **This list was stale until
  2026-07-29** — it named F11/F14/F15/F16 as open when the archive marks all four
  ✅ and the code confirms it (`filter_row` in `mathml/mod.rs`, `do_cfrac` in
  `presentation.rs`, the `0x2A50`→Cat C / `0x27A1` / `0x0331` rows plus their
  guard in `operator_dictionary.rs`). **F17 is now also CLOSED (2026-07-29)** — see
  R3 for the per-item disposition. What is genuinely open is **F5** Linebreaker
  (full feature gap — the sketch used the wrong strategy; needs a port-or-drop
  scope decision), the **F14 residual** (`m:share` hrefs use the primary ID
  suffix; the `MATHPROCESSOR->IDSuffix` secondary-suffix wiring is unconnected),
  and PARTIAL inherited-context bindings on `pmml_top`/`pmml_parenthesize`.
  (Content-MathML items obey the defer-to-a-dedicated-session directive above.)
- **arXiv velocity-fork audit** (items 1–4 landed 2026-07-03; →
  `archive/ARXIV_FORK_AUDIT_2026-07-03.md`). Sole residual: **item G** —
  `readBalanced` drops comment tokens (fork `4e1578d1`); Rust `read_balanced`
  still keeps comments in its result via `CommentSink::Into` (gullet.rs ~L1363,
  `CommentSink` at ~L565). Low urgency (`INCLUDE_COMMENTS=false` default); port
  at the next gullet-seam session.

### `fragid` parity audit — id preservation (2026-08-04)

Full Perl↔Rust comparison of the `fragid` mechanism, since `fragid` — not
`xml:id` — is what the HTML5 XSLT's `add_id` emits the HTML `id` from, so a
node with an `xml:id` but no `fragid` reaches HTML with **no id at all**.
`CrossRef::fill_in_frags` (CrossRef.pm L312-324) is the assigner and is
faithfully ported; it stamps only nodes carrying an ObjectDB `ID:` entry.

**Measured end state** (same-host Perl 0.8.8, id-rich document with sections,
labels, footnotes, equation/align, table, figure, refs): HTML id sets are
`24 rust / 23 perl`, **zero ids present in Perl and missing here**, zero
dangling in-page anchors in either engine; the one extra is our `abstract1`
(we make the abstract linkable, Perl does not). Under `--splitat=section` the
per-page sets match exactly (`S1.html` 22/22) with no broken cross-page
anchors. Witness dirs under the session scratchpad.

**Two audit claims did NOT survive checking** — do not re-chase:
- *"Scan skips XM* descendants ⇒ hrefless `<m:share/>` in all Content MathML"*:
  **refuted for documents.** A `$a<b<c$` document emits
  `<share href="#p1.m1.sh1">` in BOTH engines, byte-identical. The hrefless
  `<share/>` appears only in the standalone `latexmlmath_oxide --cmml` path —
  where Perl's `latexmlmath` emits `href="#Ex1.m1.sh1"` pointing at **nothing**
  (its own output has zero ids). Shared limitation, Perl's arguably worse (a
  dangling href vs a missing one). Not a Rust-only gap.
- *"`generate_node_id`'s fragid half is missing"*: **present and faithful**
  (document.rs:1361-1366 vs Post.pm L1490-1492). It was merely never reached,
  because the parent walk above it used a bare `xml:id` read that always
  missed; the accessor sweep activated it.

**Fixed here:** id-bearing markup CLONED into a generated bibliography (an
`ltx:Math` in a `.bib` title) reached HTML with no id, because Perl's
`Collector::rescan` (Collector.pm L97, called from MakeBibliography.pm L71/78)
re-runs the whole Scan over the generated subtree and we have no rescan.
MakeBibliography now registers every id-bearing node of that subtree
(id/fragid half only, never overwriting an existing entry). Witness went
`4 rust / 5 perl` ids → `5 / 5`. Guards:
`06_cluster_bibliography::{bib_entry_ids_are_bib_rooted_like_perl,
bib_entry_cloned_markup_keeps_its_id}` (both red-checked).

**Still open, ranked** (each verified as real code, blast radius NOT
re-measured except where noted):
1. **No `Collector::rescan`.** The fix above is a bibliography-local stand-in.
   The general wiring — re-run `Scan` from `MakeIndex`/`MakeBibliography` —
   also restores `labels`/relations/per-type values a full Scan derives, and
   would let the three ad-hoc registrations be deleted. Blocked on `Scan`
   owning its `ObjectDB` by value (`scan.rs:49`), so it needs a borrow-based
   or take/restore refactor.
2. **`associateNode` (Post.pm L508-585) unported** — generated MathML/OpenMath
   nodes carry no `xml:id`, so `convertedIDs` and pmml↔cmml parallel
   cross-linking do not exist. This is also the only real caller of
   `generate_node_id`, which currently has **zero callers**.
3. **`in_page_id` lacks the `labelids` branch** (Scan.pm L176-184) — affects
   `--splitnaming=label*` only; `Scan::new` takes no options to carry it.
4. **`in_page_id` lacks the `split_from_id` fallback** (Scan.pm L191-192);
   `PostDocument::split_from_id` is read by nobody (the `in_page_id` fallback is
   missing). It IS set on eager split subdocs (`document.rs:709`) but never on
   streaming pages. Anchor-naming only — links stay self-consistent.
5. **`strip_ref_display_fragids`** (crossref.rs:131) is Rust-only and matches
   `//ltx:ref//*[@fragid]` wholesale, so genuine id'd content inside an
   `ltx:ref` loses its fragid too. Narrow it to ids absent from the ObjectDB.
6. **`make_sub_collection_documents` returns `vec![]`** (collector.rs:141) —
   `--splitindex`/`--splitbibliography` drop every entry past the first
   initial. Note its only callers are its own unit tests, one of which is
   named `…_currently_returns_empty`, so the accessor fix landed there is
   inert until this is implemented.
7. **Glossary term/description flattened to text** (make_index.rs:553-596)
   where Perl deep-clones the phrase markup with suffix `glo`, keeping ids.

## Open items — detail for the ranked rows

### R1 — upstream `brucemiller/LaTeXML#2852`: a subfile's `\documentclass` options are not packages

**OPEN upstream**; **our half is already merged as PR #310**, so nothing is
pending here in this repo. **CI re-verified 2026-07-29: all 15 checks SUCCESS**
(TeX Live none/2021/2022/2023/2024/2025 × Perl 5.34-5.42, Linux + Windows),
`mergeable`, not a draft, **zero reviews**, untouched since 2026-07-20. So the
code side of this row is done and green; what is left is a review nudge to the
maintainer, which is a human-voice action, not a task to automate. The
allowlist was hand-split on
`,` and missed every valued form (`[varwidth=5cm]` → `Error:undefined:{varwidth}`,
pdflatex clean); it now reads `OptionalKeyVals` and matches on the key. The same
fix is ported to Perl (`OptionalKeyVals` + `getPairs`) with a `t/structure` case
that actually guards it, pushed to `dginev/LaTeXML`. **Action: check its CI, then
ask for review** — no code work expected. *(This entry read "PR #310 … Ready to
merge" until 2026-07-25, long after it merged.)*


### R2 — `--preload=<cls>` alone trips the hook stack — OPEN (re-verified 2026-07-29)

*(The "class-name divergence" this heading used to also name was the second
divergence below; it is fixed. Only the hook stack is still open.)*

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

**(c) collapses into (a) — checked 2026-07-29.** The pool load is not *ours* to reorder:
`LoadPool('LaTeX')` is the **class binding's own first statement**, in Perl
(`article.cls.ltxml` L5) exactly as in Rust (`article_cls.rs:4`), and it runs *inside*
`InputDefinitions`, i.e. after the `\@pushfilename`. Perl's ordering is therefore
identical to ours — Perl is silent only for the dump reason above. So "load the pool
earlier" means hoisting it out of the binding, which is the same Rust-only divergence
that got (a) rejected. That leaves (b), or a TeX-side repair of the stack at the moment
`LoadPool` swaps `\@pushfilename`'s meaning underneath an already-open frame.

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
  **PARITY**, not a defect (the `mathscinet.sty` binding landed 2026-07-27, PRs
  #415/#419 — see `archive/SYNC_SESSIONS_2026-08.md`). (The witness
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

### R3b — `m:menclose` is not in MathML Core — OPEN, deferred by user 2026-07-30

We emit `m:menclose` for `\cancel` / `\boxed`
(`latexml_post/src/mathml/presentation.rs` ~L520 and ~L918; Perl `MathML.pm`
L339-341 and L1507-1513 do the same). **MathML Core removed the element**, and
CLAUDE.md's standing rule is that our output targets Core. So this is a genuine
open item, not a divergence to document away.

It is deferred because unlike `<none/>` → empty `<mrow/>` there is **no
mechanical replacement**, and the two notations we emit need different answers:
- `notation="box"` (`\boxed`) → an `m:mrow` carrying a CSS border, which means
  the border has to survive the XSLT and the stylesheet, not just the MathML;
- `notation="updiagonalstrike"` (`\cancel`) → **no Core equivalent at all**.
  Options are a drawn overlay or accepting a visual regression; neither is a
  rename.

So this is a rendering change, a golden change (`tests/post/mathgolden-post.xml`
pins both today), and a deliberate divergence from Perl — it needs its own branch
and its own decision on the strike case. **Do not fix it incidentally** while
touching neighbouring pMML code.

### R3 — Presentation-MathML F5 Linebreaker (open); two math-parser witnesses found en route

**F17 is CLOSED** (2026-07-29/30). The full per-item disposition — 4 fixed, 3
do-not-port, 1 blocked, 1 unreachable, with every guard and the do-not-port
negatives — is in [`archive/SYNC_SESSIONS_2026-08.md`](archive/SYNC_SESSIONS_2026-08.md).
**F5 Linebreaker is the only pMML row left** — a full line-breaker feature gap; the
audit's sketch used the wrong strategy, so it needs a port-or-drop scope decision
before any code (`archive/MATHML_POST_LINE_AUDIT_2026-07-05.md` has the per-item
Perl line references).

Two bugs found while closing F17, both **math-parser family (R8), deferred by user
directive 2026-06-20 — do not fix in isolation**, recorded so the repros are not lost:

- **FUNCTION-APPLICATION over-insertion.** `\[ \mathop{X'}\limits_{p}^{q} c \]`:
  Rust inserts `<m:mo>⁡</m:mo>` before the trailing factor where Perl juxtaposes.
  Same family as `opdecoration_post_test`'s `op_base_is_mo`, but the base's
  presentation is a `munderover`, not a bare `mo`.
- **Script-position mislabel.** `\( {}^{n}a_{i} \)`: Rust classifies the trailing
  `_{i}` as a **prescript** (both scripts `pre`), so `a_i` renders as `{}_i^n a` — a
  relocated subscript, not a padding difference (Perl: `n` pre-sup, `i` post-sub). The
  post stage is faithful — fed Perl's core XML it agrees; the *parser* mislabels
  `scriptpos`. Reachable from `\sum'_{i=1}^{n} a_i`. A fully-populated tensor
  (`{}^{1}_{2}X^{3}_{4}`) is byte-identical, so it is specific to a partial pre/post mix.

### R6 — `ltx_env_<name>` env-markup class — PHASE 2, do NOT start yet
**Deferred by user directive 2026-07-29: this waits until the parity and
first-arXiv-release milestones are done.** It is a beyond-Perl styling
enhancement, not a parity gap, and its golden churn (an additive class on every
env element in nearly every test XML) would sit on top of release-critical work.
Pick it up only after those milestones land — then on its own branch.

Design notes, kept so the analysis is not re-derived:
- `Document::open_element` (`latexml_core/src/document.rs`) is the single funnel
  for element creation, so one armed-in-`before_construct` /
  consumed-by-first-`open_element` slot on `Document` tags exactly the env's
  wrapper element. That survives the schema's auto-open/auto-close (which defeats
  a parent-anchor + child-count mark for `figure`/`table`) and needs **no** node
  gid — so the "needs a globally-unique monotonic node gid" prerequisite below
  applies only to the raw `\newenvironment` half.
- Name sanitizing is already available: `clean_class_name` (Perl
  `Package.pm:527 CleanClassName`), giving `figure*` → `ltx_env_figure`.
- 302 of 305 `DefEnvironment!` sites are template-based and 3 are closures; both
  paths funnel through `open_element`, so one hook covers all of them.
- `add_class` merges with a template's existing `class` and is schema-filtered,
  so `minipage` would become `class="ltx_env_minipage ltx_minipage"`.
- Golden churn is mechanical via `tools/maketests.sh` (`LATEXML_BLESS=1`), with a
  filtered diff to prove only `class=` changed.

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

### (not ranked) Font-selection chain audit, 2026-07-30 — residuals

**STATUS 2026-07-30, end of day: 6 of the audit's findings are FIXED and merged
(PRs #450, #452, #453, #454, #455, #456). What is left is recorded below.**

Every merged fix was re-verified against same-host Perl 0.8.8 on the fully merged
binary: 8 of 10 scenarios byte-identical, the two exceptions being the two gaps
listed under "Still open" — i.e. nothing diverges that was not already known.
Witness 2503.04421 sits at 99.922% token similarity with dingbats matching exactly
(15 ✗ / 13 ✓ both sides).

**The recurring shape, and the reason this section exists.** Four of the six fixes
were the SAME defect class: a faithfully translated helper with **zero callers** —
`ding_fontmap.rs`, `font::decode_str`'s FontDecode variant, `font::lookup_tex_font`,
`font::rationalize_font_size`. The tables and functions were correct; nothing
invoked them, and the fallback output stayed plausible, so no test could see it.
**The compiler already catches this — if visibility is honest.** A `pub fn` in a
library crate is public API, so `dead_code` stays silent no matter how unreachable
it is. Mark an internal helper `pub(crate)` and rustc says
`function \`x\` is never used` — which CI's `cargo clippy --workspace
--all-targets -- -D warnings` turns into a hard failure. Verified on
`font_decode_string`. So the rule is not "run an audit", it is **do not write
`pub` on a helper that is not cross-crate API**; the four dead helpers were all
over-exposed, which is exactly what silenced the lint.

Two caveats, because a clean lint is not proof of wiring:
- It cannot see a dead *binding module*. `ding_fontmap.rs` was reachable only if
  something assigned `encoding = "ding"`, which `\selectfont` derives dynamically
  from the family. For declared tables ask instead: *what assigns the key this
  table is filed under?*
- It cannot see a call graph wired to the WRONG live node. `\char` called
  `decode_str` when it needed the `FontDecode` sibling — the two differ by a
  single `|| 'OT1'`. Nothing is uncalled; only reading both Perl definitions
  catches it.

#### Two known-bad values PINNED by tests — do not "fix" by loosening the assert

These assert today's WRONG output on purpose, so that repairing the underlying bug
turns the test red and forces an update. Each says so in its own doc comment.

- `120_multichar_slot_paths.rs::declare_math_symbol_still_drops_the_overlay_known_gap`
  — `\DeclareMathSymbol` routes through `mathchar.rs`, whose `props.glyph` is
  `Option<char>`, so a two-character slot loses its combining mark: T2B 128 gives
  `Ӷ` where Perl gives `Ӷ̶` (U+04F6 U+0336). Fixing it means making that field a
  string, a real type change through the math-char pipeline.
- `118_delimiter_size_nominal_font.rs` pins **4 `Error:expected:<variable>`** from
  `a0poster` — Rust-only, Perl 0.8.8 emits **zero** on the same input. Unrelated to
  delimiter sizing; nobody has diagnosed it.

#### Still open, with verification status

- **`\cal ABC` collapses to one `<mi>`** and drops `class="ltx_font_mathcaligraphic"`
  — VERIFIED by me. Perl emits three `<mi>` elements, Rust one containing `𝒜ℬ𝒞`.
  This is the `<enc>_<category>_mathstyle` gap: Perl's `FontDecode`
  (`Package.pm` L2884-2889) keeps the alphanumeric as ASCII and records the semantic
  font; Rust decodes to the styled codepoint. NOT a searchability issue — both
  engines emit styled Unicode in the final HTML, which is the MathML-Core-correct
  answer; the defect is the token grouping and the missing class.
- **No base font map for encoding `U`** — VERIFIED by me, and **SHARED with Perl**,
  not a port gap. `U` is the catch-all for every symbol font (`manfnt`,
  `latexsym`/`lasy`, `mathdesign`, XY-pic, `mathx`, `MnSymbol`), so they decode
  through the OT1 fallback and print that slot's TEXT character. Reproduce with
  `--includestyles` (the ar5iv profile) — it does NOT appear in the default profile:
  `\usepackage{manfnt}` + `\dbend` gives `¨` (U+00A8) in **both** engines.
  Structural: because the family lookup is nested inside
  `if let Some(map) = load_font_map(encoding)` — byte-faithful to Perl — a
  `U_lasy_fontmap` can never be consulted until `U` itself has a base map. Corpus
  base rate: 2.0% of 547 sampled papers emit any font diagnostic at all, and the
  log census is a LOWER bound because a family-map miss on a mapped encoding is
  completely silent.
- **`\mathversion{bold}` merges the text font instead of `mathfont`** — ✅ **FIXED
  2026-08-05** (see sweep #5 below).
- **`\DeclareTextCommand`/`\ProvideTextCommand` don't install the encoding-dispatch
  chain** — VERIFIED still-broken 2026-08-05 (`latex_constructs.rs:6525`/`6544`).
  Kernel accents are masked by the dump, so only package-declared text commands
  (tipa T3, T2A extras, TS1 additions) would show it. Rust's `\DeclareTextSymbol`
  *does* install the chain. (Same item as sweep #6 below.)
- **Slot-by-slot table parity**: 4048 slots compared across all 24 maps, reported as
  0 wrong characters / 0 off-by-N / 0 extra. I verified the AMSb subset myself; the
  remaining totals are a single-agent claim (its Perl parser was cross-validated by
  running real `perl`, and `ding[0x21]`=U+2713 was hand-checked).
- One `fontsize="40%"` element differs in count under `a0poster` (Perl 4, Rust 3),
  before and after the delimiter fix. Unexplained.

#### The original sweep (kept for its file:line pointers)


Sweep of every `selectfont` occurrence in `LaTeXML/lib/LaTeXML/` (22 hits) plus the
sibling primitives, done while fixing the family-as-encoding gap above. **The
`\selectfont` question itself is CLOSED**: Perl has exactly one
`DefPrimitive('\selectfont')` (`latex_constructs.pool.ltxml:5202`) and **no package
or class anywhere redefines or `Let`s it**, so there is no second definition site to
port. The 9 family/series/shape switches, `\normalfont`, `\verbatim@font`,
`\usefont`, `subfigure`'s 6 font hooks, `\fontencoding`/`\f@encoding`,
`\DeclareFontShape`/`Family`/`FixedFont`/`Encoding`/`Subset`, `\symbol`,
`\fontsize`, `\Declare{Text,Old}FontCommand`, `\DeclareSymbolFontAlphabet` and
`\try@load@fontshape` all verified faithful.

What the sweep turned up is in *sibling* primitives — same subsystem, same class of
symptom (a font attribute or glyph silently wrong, no diagnostic). Ranked, with
verification status stated explicitly; **the unverified rows are single-agent claims
with file:line, not established facts — re-verify before acting**:

1. **`\char`/`\symbol` yield the EMPTY STRING in math mode** — ✅ **FIXED**
   (re-verified 2026-08-05; guard
   `117_char_font_decode::char_decodes_through_ot1_in_math_and_does_not_wrap_out_of_range`,
   ground-truthed vs Perl 0.8.8). `font::decode` defaults the encoding to `OT1`
   when the current font carries none — matching Perl `FontDecode`
   `$font->getEncoding || 'OT1'` (`common/font.rs:2103-2126`, `Package.pm:2877`);
   `decode_str`/`FontDecodeString` keep the deliberate empty fallback, so the
   asymmetry is intended — do NOT "align" them. `\char` uses `u8::try_from`, not
   `as u8`, so out-of-range `\char300` yields nothing like Perl instead of `,`
   (`tex_character.rs:61-84`).
2. **`\DeclareSymbolFont`'s encoding arg is `ExpandedPartially`** — ✅ **FIXED**
   (guard `117_char_font_decode::symbol_font_encoding_argument_is_expanded_before_storage`).
   `latex_constructs.rs:6907` declares `\DeclareSymbolFont{} ExpandedPartially
   {}{}{}` (Perl `latex_constructs.pool.ltxml:2664`), so the `\encodingdefault`
   that `fontmath.ltx` writes expands before storage and dependent
   `\DeclareMathSymbol`/`\DeclareMathAccent` lookups hit the right fontmap.
3. **`DeclareFontMap`'s `(uppercase|lowercase|digit)_mathstyle` options are
   unported** — VERIFIED write-only by me: `tex_fonts.rs` writes
   `OMS_uppercase_mathstyle`, `amsb_fontmap.rs:2` records a dropped blackboard
   `uppercase_mathstyle` in a comment, and nothing reads either key. Perl's
   `FontDecode` (`Package.pm:2884-2889`) uses them to keep an alphanumeric as ASCII
   while recording the semantic font change. Claimed-but-unmeasured consequence:
   `$\cal A$` double-styles (U+1D49C *and* `font=caligraphic`) where Perl gives `A`
   + caligraphic, and hands a non-ASCII letter to the grammar.
4. **`\DeclareMathAlphabet` skips `lookupTeXFont`** — ✅ **FIXED** (re-verified
   2026-08-05): `latex_constructs.rs:7006` calls
   `font::lookup_tex_font(&family, &series, &shape)` (the abstract
   `sansserif`/`medium`/`upright` mapping, not raw NFSS codes) and `:7004` emits
   `Info!("ignore", …)` on the already-defined branch — matching Perl
   `latex_constructs.pool.ltxml:2677`. The def is at `latex_constructs.rs:6998`
   (`:6957` is now `\new@internalmathalphabet`).
5. **`\mathversion{bold}` merges the text font, not `mathfont`** — ✅ **FIXED
   2026-08-05** (`latex_constructs.rs:10653`). It was `MergeFont!(forcebold =>
   true)` (merges the current TEXT `font`), so math never went bold; now it does
   `AssignValue(mathfont => LookupValue('mathfont')->merge(forcebold => N),
   'local')` exactly like `\boldmath`/`\unboldmath` (Perl
   `latex_constructs.pool.ltxml` L5290-5297), and an unknown version raises
   `Error('unexpected', …)` instead of the silent `_ => {}`. Guard
   `06_cluster_regressions::mathversion_switches_the_mathfont_like_boldmath`.
6. **`\DeclareTextCommand`/`\ProvideTextCommand` don't install the encoding-dispatch
   chain** — VERIFIED still-broken 2026-08-05 (`:2584`/`:2598` vs
   `latex_constructs.rs:6525`/`6544`, which bind the bare `\cs` to the raw
   first-encoding expansion): the first encoding to declare a CS wins permanently.
   Kernel accents are masked by the dump, so only *package*-declared text commands
   (tipa T3, T2A extras, TS1 additions) show it. Rust's `\DeclareTextSymbol`
   (`:6624`) *does* install the chain, so the two are inconsistent.
7. Lower: `\DeclareTextSymbol` decodes eagerly at declaration instead of installing a
   deferred `CharDef` (loses the glyph permanently if the fontmap is not yet
   loaded); `LoadFontMap` never emits Perl's `Info('fontmap', …)`;
   `\DeclareErrorFont` is a bare no-op where Perl defines its arg as `\relax`.
   (The `\textit@math` `\f@shape` `i`-vs-`it` sub-item is ✅ FIXED —
   `latex_constructs.rs:10568` now assigns `it`.)

`docs/parity/OXIDIZED_DESIGN.md` has no font section, so none of these is a
documented divergence. Method and the two detection traps: [`WISDOM.md`](parity/WISDOM.md) §80.

### (not ranked) scalerel family — NEUTRALIZED, full scaling deferred (2026-08-21)

`\scalerel` / `\scaleto` / `\stretchto` / `\scaleobj` / `\hstretch` / `\vstretch` /
`\scaleleftright` / `\stretchleftright` are bound
(`latexml_package/src/package/scalerel_sty.rs`, arXiv/html_feedback#6895) but **neutralized,
not fully supported**: the object is *preserved* (wrapped in `.ltx_scalerel`, CSS-sized to
text height) and the requested scale — the target height (`\scaleto`/`\stretchto`/`\scalerel`)
or the numeric factor (`\scaleobj`/`\hstretch`/`\vstretch`) — is **dropped**, so every scaled
object renders at ~1em regardless of the requested size. This is **step 1** (preserve content,
stop `Error:undefined` + broken layout). **Full support** = honour the real scale: measure the
object box and the reference box (or read the factor), compute the ratio, and emit CSS
`transform: scale()` / an SVG viewBox — which needs box measurement the engine does not yet
expose. Witnesses 2605.02053 / 2605.03024 / 2605.03521 (now convert clean, content preserved).
Do not mistake the text-height approximation for correct sizing.
### (not ranked) sandbox-arxiv-2605/2606 cortex corpus triage — deferred items (2026-08-21)

Two waves of subagent triage over the 2605/2606 `oxidized_tex_to_html` error+fatal
clusters. Landed: PR #720 (scalerel, neurips `\if@anonymous`, NiceTabular, expl3
`#630`, biblatex loop, `cleanup_scripts` O(M×N)→O(N+M)) + stacked PR (cleveref class
stubs, AASTeX, subdir/`.sty` binding shadow — no directory stripping in dispatch or
`find_file_fallback`). Method for every row below: reproduce the witness with
`--preload=ar5iv.sty --path ar5iv-bindings/originals` (raw-loads bundled **styles**,
NOT bundled **classes**), then run the same-host Perl `latexml` oracle to classify
Rust-only vs shared. The dominant finding — the 2605 "43 new fatals" were ~90% fleet
**memory pressure**, not code — is in [[reference_cortex_fleet_memory_pressure_hardening]]
/ CorTeX#423; the error population is overwhelmingly faithful-Perl-parity.

**A. Genuine Rust-only — worth fixing, needs deeper work:**

- **sn-jnl.cls raw-load drops booktabs + appendix** → `\toprule`/`\midrule`/`\bottomrule`
  undefined → table breaks → `malformed:ltx:{section,subsection,appendix}` cascade.
  Witness **2606.00121** (`\documentclass[sn-mathphys]{sn-jnl}`). Method: Perl dep-scans
  sn-jnl.cls (booktabs loads → 11 clean errors, no cascade); Rust raw-loads the full
  1765-line sn-jnl.cls but `\usepackage{booktabs}` (sn-jnl.cls:307) + `\usepackage[title]{appendix}`
  (:303) fail to take effect while flat-block neighbours (multirow:298, rotating:302,
  xcolor:304, algorithm:308) load fine. Isolated `\usepackage{booktabs}` loads correctly
  — only the full raw-cls-load drops it. Needs instrumented tracing of raw-load dependency
  handling (`content.rs:1993` `maybe_require_dependencies`, ~10-witness guard); a blind
  patch is reckless. Min-repro: `\documentclass[sn-mathphys]{sn-jnl}` (real .cls present)
  + a `\toprule`/`\bottomrule` tabular → Rust undefined; generic class + same
  `\usepackage{booktabs}` → 0 errors.
- **subdir/`.sty` binding shadow — LANDED (no directory stripping anywhere).** A paper-local
  `\usepackage{subdir/<name>}` whose basename collided with a bound CTAN package (e.g.
  `utils/mathenv` → the `mathenv` binding, `latexml_package/src/package/mathenv_sty.rs`, a
  no-op) had its directory stripped at TWO sites — the package dispatcher (`lib.rs`) and
  `find_file_fallback`/`_exists` (`content.rs`, the `BasenameOnly` fallback) — so the binding
  shadowed the local file and its cleveref/theorem defs never loaded. Perl never strips a
  directory (`Package.pm:2191` FindFile_fallback strips VERSION suffixes only). Fix: dropped
  the strip at BOTH sites — `subdir/<name>` is a PATH, so the local file raw-loads under
  `localrawstyles`. The retired `find_file_fallback` `BasenameOnly` convenience (subdir copies
  of KNOWN packages — 2105.02087 `misc/ieeetran`, 2405.18387 `assets/equations`) now falls to
  OmniBus/raw-load like Perl (no test guarded them; full-suite blast radius nil). Witness
  **2606.02073** (its own `\cref` is also defined via the icml binding, so the corpus error
  there was already masked — the shadow is proven by the synthetic guards). Guards:
  `cluster_package_guards.rs::subdir_dispatch_no_strip` (`.sty` raw-loads, `.cls` stays OmniBus
  under classes-off), both driven through `convert_to_xml_ar5iv` (the real fleet config).

**B. Beyond-Perl levers — policy call (need an OXIDIZED_DESIGN entry + Perl upstream):**

- **OmniBus frontmatter vocabulary extension** (~150 docs, the biggest cluster). Bundled
  journal `.cls` (INCLUDE_CLASSES defaults false → OmniBus fallback, `content.rs:2457-2622`,
  faithful port of Perl `LoadClass`) leaves `\orcid \contribution {contribution} \ack
  \correspondence \aff \lefttitle \righttitle \reportnumber \data \checkdata
  \restartappendixnumbering` undefined. **Perl fails identically** (Rust slightly ahead:
  3 vs 5 errors on the witness). ~15+ distinct bundled classes. Witnesses: 2606.01241
  (xiaomiev: contribution/correspondence/checkdata), 2606.00645 (jfm: aff/lefttitle/righttitle),
  2606.04098 (iopjournal/youtu: data), 2606.00213 (pasj02: orcid). Single edit to `omnibus_cls.rs`,
  NOT 15 class bindings; do NOT raw-load the 2000-line classes (the cascade OmniBus exists to
  prevent). Must upstream the same to Perl's `OmniBus.cls.ltxml` to stay parity.
  - **SAFE SLICE LANDED (0.7.6, OXIDIZED_DESIGN #160):** `\orcid[]{}`→`\lx@add@orcid{#2}` (captures
    the id as `<contact role=orcid>` with an orcid.org link) + `\lefttitle`/`\righttitle` no-op'd
    (presentational running heads). Guard `omnibus_captures_orcid_and_drops_running_heads`.
  - **CORRECTION to the original plan (verified against the real classes, 2026-08-23):** the plan's
    `\aff`→`\lx@add@affiliation` mapping is **WRONG** — jfm's `\def\aff#1{\ignorespaces\textsuperscript{#1}}`
    is a **superscript reference marker**, NOT affiliation text; routing it to affiliation would inject
    "1"/"2" as affiliations and corrupt every jfm byline. Likewise the semantics of `\correspondence`
    (xiaomiev→checkdata, youtu→email-metadata, others→store), `\data` (youtu HuggingFace URL), and
    `\contribution`/`\checkdata` (xiaomiev author-contribution lists) **vary by class**, so a single
    generic mapping risks corrupt output — these need per-class output review before capture. `\ack`
    is `\begin{acknowledgments}#1\end{...}` (1 arg, needs the acks env wired); `\reportnumber` has NO
    texmf definition (arity unknown). REMAINING DEFERRED: `\aff \contribution \correspondence \data
    \ack \reportnumber \checkdata \restartappendixnumbering`. Note `\contribution`/`\correspondence`
    are already captured for classes with dedicated bindings (fairmeta/selfevolagent) — only the
    generic OmniBus path is open. CUP family (jfm/iau/pas) subsumed here.
- **native newunicodechar binding** (~50 docs, ~100 occurrences). `\newunicodechar{<non-ASCII>}{repl}`
  → both engines take the 8-bit path (UTF-8 char = one Unicode token → length 1 →
  `\nuc@onebyteerr` → `Error:latex:(newunicodechar) ASCII character requested`). **Perl
  byte-identical**, non-fatal (char passes through; the mapping is DROPPED by both).
  Witnesses: 2606.00241 (icml2026.sty), 2606.00683 (colm2024), 2606.00739 (acl.sty).
  Min-repro: `\usepackage{newunicodechar}\newunicodechar{，}{,}` (， = U+FF0C). Lever
  (separate branch, upstream to Perl): a native binding that registers the
  Unicode-char→replacement mapping and suppresses the spurious error.

**C. Pure Perl-parity — record only (Perl-origin; belongs in KNOWN_PERL_ERRORS too):**

- **forest** `Error:undefined:{forest}` — line-for-line port of ar5iv `forest.sty.ltxml`,
  non-fatal (body swallowed via `discard_env`, valid output). Witnesses 2605.07358/12090/12792.
  Shared helper `discard_env.rs` also backs nicematrix (math family)/diagrams/pb-diagram.
  Whole-family lever (policy): `Error!`→`Warn!` at `discard_env.rs:55`.
- **malformed:ltx:para** — `\begin{center}` inside `\begin{titlepage}` with an abstract +
  multi-paragraph flow → `insertBlock` fallback forces `ltx:block` (`TeX_Box.pool.ltxml:516`)
  which can't hold `ltx:para`/`ltx:abstract`. Perl identical (oracle-verified). Witnesses
  2605.00729/00750/12448.
- **malformed:ltx:section/subsection (parity subset — the majority)** — broken source
  (unclosed lists/boxes, broken alignments) traps sectioning inside an open container; a
  CASCADE, not a sectioning bug. Perl identical. Witnesses 2606.00338 (unclosed `\squishlist`
  `\begin{list}`), 2606.00679 (`\ytableaushort` inside an `eqnarray*` alignment).
- **graphicx-in-neurips** — author-customized `neurips_2026.sty` (does `\usepackage{graphicx}`)
  is discarded by the neurips-binding interception (#690); `\includegraphics`/`\rotatebox`
  undefined + a downstream `_` cascade. Perl fails identically (22 errors; version-suffix
  fallback → `neurips.sty.ltxml`, which also lacks graphicx). Witness 2605.21325. #690 brought
  Rust *to* Perl parity (it was accidentally better before).

### (not ranked) sandbox-arxiv-2606 study — 2026-08-23 (methodology-corrected)

Third-wave triage of the 2606 `oxidized_tex_to_html` clusters (6 clusters, parallel
subagents), under the CORRECT same-host method: BOTH engines with `--preload=ar5iv.sty
--path=ar5iv-bindings/bindings`, NO `--quiet` (it hides Perl's `Error:` lines), ANSI-strip
Perl stderr (`sed 's/\x1b\[[0-9;]*m//g'` — else `^Error:` reads 0 on Perl), real `--dest`.
A first pass WITHOUT these produced FALSE "Rust-only" verdicts (comparing oxidized-with-the-
paper's-`macros.sty` against Perl-without-it): every top cluster re-classified to parity once
corrected. See [[feedback_sandbox_preload_ar5iv]] + [[feedback_strict_vs_lax_error_grep]].

**Net: the 2606 clusters are overwhelmingly PARITY + error amplification, not regressions.**
2605↔2606 cluster doc-counts are near-identical (stable long-tail, not new breakage). Under
the correct method Perl and oxidized fail on the SAME constructs; oxidized is often marginally
BETTER (clean DOM vs Perl's `<ltx:ERROR/>` nodes; runaway backstop bails as fast or faster).

**LANDED (PR fix/newtcblisting-leading-optarg):** `\newtcblisting` binding stub
`tcolorbox_sty.rs:41` lacked the LEADING `[init-options]` optional (real sig `{ m +O{} m o +o +m }`),
so `\newtcblisting[auto counter,...]{name}[2][]{...}` read `[...]` as the mandatory name → env
undefined → verbatim body tokenized as LaTeX → every `_`/`^` → "Script can only appear in math
mode". The ONLY genuine Rust-only witness across the `unexpected/_` cluster. Witness **2606.00555**
455→0 errors; priors 2507.00833/2402.13846 preserved; guard `cluster_newtcblisting_leading_optarg`.

**Cross-cluster policy item — ERROR AMPLIFICATION (biggest measurement distortion):** oxidized's
diagnostic error cap (500/1000) vs Perl's MAX_ERRORS=100 → oxidized emits 1.4–30× more error
records on the SAME shared failure (2606.14954 `_`: 141 vs Perl 100; alignment 213 vs 17; achemso
"501 hits" = the 500-cap on ONE doc = 5× Perl's cap-100). Inflates cluster hit-counts AND the
`TooManyErrors(500/1000)` fatal population (~96 docs) vs Perl's cap-100 fatals; does NOT reflect
worse conversion. Policy call (needs an OXIDIZED_DESIGN entry): align the DIAGNOSTIC cap toward
Perl's 100 — this is distinct from the runaway/expansion backstops, which
[[feedback_runaway_limit_raise_not_reduce]] governs (raise, never lower) and this does not touch.
Per-doc DOMINANT-cluster counts (robust to amplification): `unexpected/_` 360, newunicodechar 119,
`\GenericError` 96, `&` 93, `malformed:ltx:caption` 88.

**Deferred plan, priority order (cross-ref the 2026-08-21 section above; do NOT duplicate):**
1. **newunicodechar (~119 docs) — ROOT CAUSE found** (deepens §B "native newunicodechar binding"):
   newunicodechar.sty's engine-probe uses four-hex `^^^^HHHH` caret (XeTeX/LuaTeX only); neither
   engine supports it (`mouth.rs:715 get_next_char` ≈ Perl `Mouth.pm:156`, both only two-hex `^^HH`
   + single `^^X`) → both take the utf8-byte-count path → both error. Better fix than a per-package
   binding: add four-hex/six-hex caret in `get_next_char` so newunicodechar takes its Unicode branch
   (oxidized's Unicode branch already maps α→945 correctly). Diverges from Perl (oxidized clean / Perl
   still errors) — upstream the same to `Mouth.pm`. Min-repro
   `\usepackage{newunicodechar}\newunicodechar{α}{\ensuremath{\alpha}}` → OX/Perl 1 err, pdflatex 0.
2. **OmniBus frontmatter vocabulary (~150 docs)** — §B above (confirmed parity, unchanged).
3. **Missing package/class bindings (beyond-Perl conversion wins)** — `tabu`/`tabularray` (alignment
   cluster; both engines unbound; oxidized amplifies 2–12× and misparses `\begin{tabu} to <dim>{preamble}`
   as a column template `t`, `alignment.rs:997`), qcircuit/semantex/nicematrix (cascade fatals),
   achemso/revtex4-2 (frontmatter classes unbound in both). quantikz already landed on HEAD.
4. **`_` math-mode-loss residual (~360 dominant docs, minus 00555) — PARITY:** both engines lose math
   mode on shared constructs (theorem `\label{..._..}` + crossreftools active; undefined metadata envs
   like `{CCSXML}` leaking verbatim bodies; math-`array` `\tabularnewline`+`\hline`). No single Rust fix;
   the amplification item is the only oxidized-side lever, plus per-binding cascade prevention.
5. **floatrow raw-load (2606.10047)** — narrow genuine residue (OX 18 `malformed` vs Perl 0); floatrow
   reroutes subcaption placement, oxidized's raw interp still malforms. Own ticket.

**Confirmed PARITY — do NOT re-triage or "fix" (both engines identical / oxidized cleaner):**
- `latex/\GenericError` "Not in outer par mode" (88) + "strip used only in twocolumn mode!" (56) —
  `cuted.sty` `\begin{strip}` misused (one-column, or after `\maketitle` in non-outer-par). Both
  engines emit the same message at the SAME source line/col; pdflatex would too. Witnesses 2606.27050
  (ECOC template `\begin{strip}` after `\maketitle`), 2606.00001 (`jaist` one-column + cuted). Do NOT
  touch par-mode / twocolumn tracking — it is correct.
- `malformed/ltx:listing` (52) — always a CASCADE, never root: listings style-hooks with mode-switching
  bodies (`keywordstyle=\..\bfseries{#1}`, `stringstyle=\text{#1}`) raise "close a group that switched
  to mode horizontal" in BOTH engines (`stomach.rs:733`); the listing malforms only surface past the
  100-cap (amplification). Witnesses 2606.00625 (lstlisting in adjustbox), 2606.30854 (macros.tex `\code`
  = `\lstinline` with style hooks), 2606.29025 (algorithm2e+algorithmic). 2606.17850 is mis-clustered
  (actually an `ltx:td`/`ltx:tr` table cluster). The only real (shared, deep, not-small) defect is the
  mode-switch-in-listing-style-hook — a future parity-improvement, not a regression.
- `misdefined/#` "token # (catcode PARAM) should never reach Stomach" (~32) — algpseudocode `#` leak;
  both engines emit it ~equally (2606.03769 OX 98 / Perl 96, both capped); oxidized clean where Perl
  errors on 2606.05922. PARITY.

Runaway/fatal re-verification (corrects any "oxidized hard-loops worse than Perl" impression): on
2606.13219 (pgfplots) oxidized bails `Fatal:Timeout:PushbackLimit` <90s while Perl HANGS (timeout);
on 2606.30928 (tikz-cd/quiver) oxidized bails in 3s vs Perl 23s. Runaway-fatals = PARITY; the 650k
PushbackLimit backstop is a strength, not a regression.

### (not ranked) fairmeta.cls family trailing items (2026-08-21)

The fairmeta author↔institution binding fix (arXiv/html_feedback#1396 + the
family #662/#3512/#4707/#4971/#5035/#5466; PR #748, shared `meta_class.rs` routes
`\author`/`\affiliation`/`\contribution` marks through the annotation/label plan)
left two witness-side issues OUT OF SCOPE, to pursue later. All eight in-scope
witnesses convert exit 0, front matter error-clean; these are the only residuals:

- **`\pie` undefined (pgf-pie) — missing-package binding.** Non-fatal body error
  (conversion still exit 0), unrelated to the front matter. Witness
  **2408.00714v2** (#5144, the SAM 2.1 paper) — a `\pie{…}` pie-chart macro in the
  document body; the `\documentclass{fairmeta}` front matter converts clean (18/18
  authors linked). Needs a pgf-pie binding — **confirm same-host Perl parity first**
  (pgf-pie is niche and likely unbound in both engines → shared missing-package).
  NOT what #5144 reported (that was the author block, now fixed).
- **Wrong main-file selection — NOT a binding bug.** Witness **2602.06855v1**
  (#5967, "2026 template … rendered template instead of content"): the source
  ships BOTH `og_template.tex` (the FAIR template filler — `\lipsum`,
  `\rectanglecolor`, `subfigure`; 4 errors) and the real `paper.tex`; the pipeline
  built the template. A cortex/main-file-selection concern, not the fairmeta
  binding — the real `paper.tex` converts fine.

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
| Multi-document streaming post-join (main+supplements) | queued | [`performance/MULTIDOC_JOIN.md`](performance/MULTIDOC_JOIN.md) |

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
    Perl 241.5pt, ~1.257×), so figure sizing is visibly affected.
    **The break/block-arrangement half LANDED (PR #273):** `arrange_panels`
    (`latex_constructs.rs`, faithful port of `arrange_panels_and_breaks`) now does
    the break-insert + block/merge arrangement at three float call sites. **What
    remains OPEN is only the box-WIDTH divergence** (tied to the deep box session /
    the `\resizebox` panel-width item below); re-witness 1108.0198 / 0704.0001 /
    0704.0017 to confirm whether widths still diverge now that the arrangement is
    ported.
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

### (not ranked) SVG picture path: regex/string-splice → DOM, to match Perl
The post SVG handling for `<ltx:picture>` in `latexml_oxide/src/post.rs`
(`extract_svg_fragments` → `convert_picture_children_to_svg` → the
`finalize_html5` splice) is **regex/string-based**: it serializes each picture in
pass A, converts an *enumerated* child set (`g`/`line`/`circle`/`ellipse`/`rect`/
`polygon`/`path`/`bezier`/`text`/`Math`/`graphics`) to inline SVG by regex, and
splices the string into the empty `ltx_picture` placeholder **after XSLT**. It
exists only to dodge a rust-libxml `PostDocument`-cleanup use-after-free (see the
`finalize_html5` splice note); the canonical DOM port `latexml_post::svg::SVG`
(faithful `SVG.pm::convertNode`, `LaTeXML/lib/LaTeXML/Post/SVG.pm` L148-182) is
already written but unused. Because our content never re-enters the XSLT, it
diverges from Perl on foreignObject content: Perl wraps it in
`<span class="ltx_foreignobject_container"><span class="ltx_foreignobject_content">`
(we omit both), and picture geometry is pt→px-rounded differently (138 vs 132.44
on the `none.png` repro). The **task**: once rust-libxml's `PostDocument` cleanup
no longer UAFs on an inserted subtree (adjacent UAF/NULL-deref bugs fixed there in
2026), retire the string path for the `svg::SVG` `Processor` — inject the SVG as
child nodes pre-serialization, so all picture content (and any element type, not
just the enumerated set) goes through the real XSLT and matches Perl byte-for-byte.
Witnesses: arXiv:2311.14363v2 (#1291 — picture-nested `\includegraphics`, image
resolution now restored at splice by re-injecting the Graphics-phase result; the
`<img>`/`<object>` itself is byte-identical to Perl, only the wrappers/geometry
differ), html_feedback#74 (xfig Math labels). Ordering note: our pass-A extraction
snapshots the picture BEFORE the pass-B Graphics phase, inverting Perl's
Graphics-before-SVG order (`LaTeXML.pm` L493 before L502) — hence the splice-time
re-resolution `#1291` added.

**Picture-nested-graphic rendering — status & follow-ups** (found while landing
overpic #677 + #675; giant/double LANDED, two roots OPEN):
- **FIXED (`post.rs` `render_resolved_graphic` / `convert_picture_children_to_svg`):**
  the resolved `<img>`/`<object>` was double-emitted (once inside the makebox
  `<text>`, once as the `<foreignObject>`) and the foreignObject copy drew at the
  raster's *natural* pixel size (giant, spilling the figure+caption). Now the
  `<graphics>`/`<Math>` is stripped from the `<text>` copy and the foreignObject
  image is constrained to its box (`object-fit:contain`). Guards
  `graphic_inside_a_makebox_text_is_not_double_emitted`,
  `degenerate_picture_svg_leaves_image_unconstrained`. Witness overpic 2510.17772
  Fig 7 (A/B/C now clean).
- **OPEN — picture-SVG `\unitlength` sizing (core, high blast radius).** An
  Inkscape `.pdf_tex` picture (`\setlength\unitlength{458pt}\begin{picture}(1,0.7)…`)
  gets a DEGENERATE outer `<svg width="1.33" height="0.95">` — the picture's
  outer dimension is computed at ~1pt/unit, IGNORING `\unitlength` (post.rs:1809
  reads the `<ltx:picture>` `width`/`height` attrs verbatim; the tiny value comes
  from the core `{picture}` sizing, not post). Every foreignObject then collapses
  in the browser, so the picture-nested image can only *overflow-leak* at natural
  size (giant) — a `100%`/object-fit constraint would zero it, so the giant-fix is
  **gated** to non-degenerate SVGs (`DEGENERATE_SVG_PX = 4.0`) and this path is
  left at its prior giant-but-visible behavior. Real fix: apply `\unitlength` to
  the picture's outer dimensions AND its child coordinates (positions are laid out
  at the wrong scale too — resizing the SVG alone mispositions). Witness
  arXiv:2311.14363v2 (18 degenerate picture SVGs). Blast radius: all `{picture}`
  users (tikz/pgf/pict2e), so validate broadly.
- **OPEN — graphicx `trim`/`clip` not physically cropped.** Rust recomputes only
  the width/height for `trim`/`clip` (`latexml_core/src/util/image.rs:433-442`,
  `apply_graphicx_ops`) and references the ORIGINAL uncropped raster; the browser
  then scales the whole image into the cropped box (visual mismatch — a colorbar
  slice renders as a full heatmap). Perl physically crops via
  `image_internalop($image,'Crop',…)` (`LaTeXML/lib/LaTeXML/Util/Image.pm:414`,
  `image_graphicx_complex`). Localized fix: a `crop_image_inplace` sibling to
  `rotate_image_inplace` (`latexml_post/src/graphics.rs:703`, shells out to
  `convert -crop WxH+X+Y`), 2 call sites (graphics.rs:2464/2501, raster-gated),
  porting the crop geometry (Image.pm:404-418, incl. the lower-left→upper-left flip
  + source DPI). No new crate (already shells to ImageMagick `convert`). Witness
  arXiv:2510.17772 Fig 7 (the `bottleneck_heatmap` matrix/colorbar `trim` split).

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
