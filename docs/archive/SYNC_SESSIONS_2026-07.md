# SYNC_STATUS archive — completed 2026-07 session logs (`fidelity-improvements-072026` window)

> Moved out of [`../SYNC_STATUS.md`](../SYNC_STATUS.md) in the 2026-07-08 docs
> consolidation (release 0.7.3 prep). Everything here is COMPLETED work; live
> residuals were lifted into SYNC_STATUS before archiving. Covers the
> `ar5iv-2606-prep` tail (2026-07-02 … 07-05) and the
> `fidelity-improvements-072026` PDF-fidelity + upstream-sync work (2026-07-08).
> The 2026-06-22 … 2026-07-01 logs are in
> [`SYNC_SESSIONS_2026-06.md`](SYNC_SESSIONS_2026-06.md).

---

### Landed this session (2026-07-08, on `fidelity-improvements-072026`) — `\RequirePackage` in `\AtBeginDocument` (self-inflicted #2846-port regression)

**Symptom.** `error/unexpected/\RequirePackage` — "The current command
'\RequirePackage' can only appear in the preamble" — fired at begin-document.
Corpus witnesses: **arXiv:2605.00022, arXiv:2605.00119** (both
`\usepackage{inconsolata}`, whose `inconsolata.sty` does
`\AtBeginDocument{...\usepackage{upquote}}` → upquote.sty's top-level
`\RequirePackage{textcomp}` under ar5iv INCLUDE_STYLES raw-load).

**Minimal reproducer** (`docs/reproducers/atbegindocument_requirepackage.tex`):
```tex
\documentclass{article}
\AtBeginDocument{\RequirePackage{xcolor}}
\begin{document}
Hello
\end{document}
```
Ground truth, **same host**: `pdflatex` → exit 0; Perl `latexml` → exit 0
("No obvious problems"); Rust (pre-fix) → `Error: '\RequirePackage' can only
appear in the preamble`. A body-level (not hook) `\RequirePackage` **must still
error** — all three engines do (that IS `\@onlypreamble`; kept as parity).

**Root cause = upstream PR #2846 regression, faithfully inherited by our port
(RESOLVED — no scoping subtlety).** Real `latex.ltx` `\document` fires the
begindocument hook (L44) and only THEN runs `\@preamblecmds` to disable the
`\@onlypreamble` commands (L54) — so `\RequirePackage`/`\usepackage` inside
`\AtBeginDocument` is legal. **PR #2846 moved `AssignValue(inPreamble => 0)`
from AFTER `@at@begin@document` (pre-#2846: `# atbegin is still (sorta)
preamble`) to just BEFORE it (`# ...leaving the preamble (!?)`).** That is a
regression *in Perl itself* — **verified**: the vendored post-#2846 `latexml`
(rev 51fea96a) errors on the reproducer, while installed pre-#2846 Perl 0.8.8
does not. Our #2846 port (`3ebf6e1a3d`) copied the post-#2846 placement and
inherited the same error.

> **Correction to an earlier wrong theory in this note:** there is **no
> scoping / frame-topology subtlety**, and `assign_value` is NOT broken — it
> faithfully mirrors Perl `assignValue` (both default `local`, both revert on
> frame-pop; verified `state.rs:801-808` ≡ `State.pm:152`). The apparent
> paradox ("source sets 0 before the hook, yet the hook probe reads 1") was a
> **version mismatch**: the probe ran the *installed pre-#2846* binary (0 set
> after the hook) while the source read was the *vendored post-#2846* copy (0
> set before). Recorded as an upstream bug in `KNOWN_PERL_ERRORS.md` #43.

**Fix (`2fe9fd76fa` + doc/comment correction).** Restore the pre-#2846 point:
keep `inPreamble=1` across `@at@begin@document` + the begindocument L3 hook, and
clear it immediately afterward (`latex_constructs.rs`, `\begin{document}`
constructor). This matches latex.ltx + pdflatex + pre-#2846 Perl 0.8.8, and
*surpasses* the current buggy upstream (the #2754 `\AtEndPreamble` goal is still
met — `@document@preamble@atend` runs before the clear). Supersedes the narrower
mathtools `\lx@mathtools@require@graphicx` workaround (now redundant, harmless).
2605.00022 → 0 errors; 2605.00119 → only an unrelated babel/fontspec
`bidi=default` LuaLaTeX/XeLaTeX error remains.

### Landed this session (2026-07-05, on `ar5iv-2606-prep`) — faithful width-based figure-panel arrangement (2605.00347)

Same witness (2605.00347), Appendix F "maria" subfigure grids. User report:
Rust broke the subfigures 1-per-row where ar5iv shows 4-per-row in 2 rows.
Rewrote the simplified Rust-only `arrange_panels` into a faithful port of Perl
`arrange_panels_and_breaks` (`latex_constructs.pool.ltxml` L3229-3349) —
computing per-row breaks from actual panel box WIDTHS. A first-principles review
vs the Perl source surfaced three corrections (commit `8482891f55`):
* **floatwidth source.** `after_float` was missing Perl L3389's
  `$whatsit->setProperty(floatwidth => LookupRegister('\hsize'))`; arrange then
  fell back to the ambient `\hsize` at construction time (wrong for figure*,
  nested subfigures). Now captured on the whatsit and read back via
  `float_width_of` from the box the afterClose hook receives (Perl L3231).
* **standalone trailer break** (Perl L3334-3342) ported — a standalone panel as
  sole row content forces a break before the next sibling.
* `@all_contents` is **dead in Perl** (BuildPanelsAndID never uses the return) —
  correctly omitted.
Plus: `subcaption_width_props` records the `{Dimension}` arg as a `width`
property on ALL sub-float envs (Perl subcaption.sty L66/76/86/96; Rust-only
`subcaptionblock` aliases inherit it); `panel_width` falls back to the emitted
`width` attribute when a node has no tracked box width (minipage/parbox).
**Validated: output now matches the live Perl ar5iv EXACTLY** — 41
`ltx_figure_panel` / 41 `ltx_flex_cell` / 7 `ltx_flex_figure`, flex-size
12 break / 5 size_1 / 20 size_2 / 16 size_4 (pre-fix binary was 35/33). Goldens
re-blessed to Perl-matching output (figures/figure_mixed_content/tikz_figure).
Suite 1527/0.

**Latent divergence noted (NOT fixed — no current incorrectness):** `\framebox`,
`\parbox`, `\rule` store their `width` PROPERTY as a `Stored::String`
(`.to_attribute()`) rather than a `Stored::Dimension` as Perl does, so
`getNodeBox->getWidth` reads `None` for them; `panel_width`'s `width`-attribute
fallback covers the panel-arrangement path. `minipage`/`makebox` are faithful
(Dimension); `includegraphics` is parity (no width property, size via the
`image_graphicx_sizer` `cached_width`). Fix if a future path reads box width
without the attribute fallback.

### Landed this session (2026-07-05, on `ar5iv-2606-prep`) — author/affiliation frontmatter split (beyond-Perl)

Witness arXiv 2605.00347 (colm2026 class, 13 authors on three `\textbf{…}`
lines with `$^{1,2,3,*}$` markers). User report: "multiple frontmatter
duplicate notes"; ground truth = the PDF's author↔affiliation assignment.

Root cause in `\lx@add@authors` (`base_utilities.rs`): the two bold author
lines are each a whole-line `\textbf{A$^1$, B$^1$, …}` wrapper, so the
separating commas are brace-hidden. `split_tokens` skips delimiters inside
`{…}`, collapsing each bold line into ONE creator that then collected every
`$^1$` marker → 3–5 duplicated "Princeton…" affiliations, only 7 creators
instead of 13. Perl is broken identically (same-host confirmed) → surpass-Perl,
user-directed. Two fixes, both in the author arm `split_author_line`:
* unwrap a whole-line font wrapper (`whole_line_cs_wrapper`), split the inner
  name list, re-apply the wrapper per author → 13 individual creators, one
  affiliation each;
* literal " and " removed from line-level `author_affil_splits` (was shredding
  "Princeton Language **and** Intelligence"), applied only in the author arm so
  "Alice and Bob" still splits.
Result matches the PDF exactly (¹→11, ²→Lu, ³→Yang, \*→3 equal-contributors).
6 new `author_split_tests` unit tests; suite 1521→1527. Divergence #48 in
OXIDIZED_DESIGN. NOTE (separate, pre-existing, NOT fixed here): minimal
2–3 author blocks orphan their annotations and drop the creators in BOTH
engines (`label=affiliation:N`/`LABEL:N` warnings) — a frontmatter-resolution
timing quirk unrelated to the split; the real paper resolves fine.

### Landed this session (2026-07-03, on `ar5iv-2606-prep`) — live-run fatal/error mining round 2 + upstream sync to #2837

Mining the in-flight full-arXiv run (15,858 fatal tasks at ~half-complete;
canvas-triage rules, same-host Perl):

* **Panics (50 papers, 4 sites): ALL RESOLVED.** 49/50 were already fixed at
  HEAD (graphics `join().unwrap()` 43×, parser `Node::new` 5+1× — landed
  2026-07-02; the fleet binary predates them). The 50th — `\hbox`
  HBoxContents predigest `None` unwrap (`tex_box.rs`), witness
  math-ph/0405041 — fixed graceful (`62ecfdbb5e`), minimal LamsTeX
  reproducer in `docs/reproducers/hbox_none_contents_lamstex.tex`
  (all four components load-bearing: amstexl + lamstex + `\list\item` +
  `$$x \tag\label{F}$$`). Same-host Perl completes it with 15 errors.
* **`undefined` top-whats classified:**
  - `\Checkmark` 3018 / `\XSolidBrush` 2841 (bbding), `\Letter` 2985
    (ifsym `[misc]`): **HOST-PACKAGE GAP, not code** — this fleet host lacks
    `texlive-fonts-extra` (`kpsewhich bbding.sty/ifsym.sty/fourier.sty` all
    empty); both engines' bindings raw-load these + fontmaps (Rust has
    `ding/ifsym/ifblk/...` fontmaps ported). **OPS ACTION for July-5: install
    `texlive-fonts-extra`** (~9k+ projected tasks). The old Perl run's host
    had them (its counts are below the top-100 cutoff).
  - `\KeyWords`/`\REVIEW`/`\Year`/`\pagerange`/`\ack`/`\Name`/`\fnsep`
    (journal classes: pasj00, ptptex, CUP, EPJ-woc): **PARITY BY DESIGN** —
    both engines OmniBus unknown classes (class raw-load intentionally
    disabled in Rust AND Perl, user-confirmed 2026-07-03); same-host witness
    astro-ph0104039: Rust 13 errors vs Perl 29, `\KeyWords` undefined in
    both. Do-not-chase (surpass option: OmniBus-level frontmatter stubs —
    a user decision, not autonomous work).
  - `{diagram}` 2565, `\url` 1926: Perl counts comparable (3089/4394) —
    PARITY, skip.
* **`unexpected` reconfirms** `\lx@begin@alignment` 8193 tasks (tabularray
  `tblr` binding leak) as the largest single GENUINE code target — still the
  known deep deferred item (sandbox-2605 verdict stands).
* **Upstream sync now complete through #2837**: `\hdotsfor[]{N}` column-span
  ported (`43c8eae310`, cluster fixture 9+4 cells); #2832 N/A-verified,
  #2835/#2841/#2829 previously ported, #2842 already correct
  (`\plparsep`). Reference tree pulled to `9f3fa9fc`.

### Landed this session (2026-07-02, on `ar5iv-2606-prep`) — upstream PR #2829 "Framing" ported in full

Faithful translation of brucemiller/LaTeXML#2829 (merged upstream 2026-07-02,
ref `d666adf8` — post-dates the archived U1-U11 sync window):

- `LookupDimension` coercion widening (`state.rs::lookup_dimension_cs` →
  `Option<Dimension>`): obvious-dimension strings parse directly, tokenized
  specs resolve registers or read multi-token dimensions. **KNOWN_PERL_ERRORS
  #41**: the upstream rewrite unintentionally LOST the macro-body-read path
  for `\def`-ized lengths (warns + 0 now) — Rust keeps it (deliberate
  divergence, covered by the arraycolsep cluster regressions).
- `framedProperties` helper (`tex_box.rs`, pub via prelude): consistent
  framed/framecolor/cssstyle attributes + pad* Dimension properties;
  `\lx@framed` now takes `OptionalKeyVals:framed` (margin/rule).
- `insertBlock` filters properties to the ltx:figure attribute set.
- `\makebox`: alignment map gains `c`→center, `s`→stretched (was
  "justified"); width without explicit alignment defaults to center.
  `\@framebox` properties now via framedProperties (this RESOLVES the
  KNOWN_PERL_ERRORS #35 `$sep ne '3.0pt'` bug upstream — entry updated), and
  the single-child unwrap is skipped when an explicit width was given.
- framed.sty ({framed}/{oframed}/{shaded}/{shaded*}/{snugshade} via
  framedProperties — shaded family now carries framed=rectangle+framecolor;
  {leftbar} direct properties, its `color` filtered out by insertBlock;
  {titled-frame} margin 0pt/rule 2pt), ntheorem `\lx@addframing` (copies
  framed/framecolor/cssstyle, cssstyle SET not merged), soul `\textul`
  (framedProperties with font-color fallback → framecolor always present).
- CSS: `.ltx_framed { padding:3pt; }` default removed (padding now supplied
  per-construct via framedProperties).
- All 6 upstream fixture updates mirrored: our re-blessed
  tabbing/mathtools/marvosym/soul/framed/ntheorem golden diffs are
  **byte-identical to the upstream fixture diffs**. Suite 1506/0.

### Landed this session (2026-07-02, on `ar5iv-2606-prep`) — MathML-post exhaustive line audit, wave 1+2

User-commissioned ("the Rust translation wants to be an exhaustive port") audit
of the MathML post-processors, opened as the living worklist
[`docs/MATHML_POST_LINE_AUDIT.md`](MATHML_POST_LINE_AUDIT.md) (verdicts for all
60 MathML.pm subs + 197 DefMathML registrations + sibling files; F-numbered
findings). Ten findings LANDED same-day, each witness-verified byte-identical
against same-host Perl (commits `3ab9ce3cb3`…`e577613fb1` + cfrac):

- **F1** author-spacing (lpadding/rpadding) carry into the spacewalk
  (astro-ph0001001 witness — the user-reported lost-spaces bug).
- **F2** dead duplicate spacewalk deleted from `mathml/mod.rs` (three-way
  table verification first).
- **F10/F12/F13** pmml-wrapper parity (menclose/class/_role), Apply:ENCLOSE →
  `m:menclose` (`\cancel`), FRACOP verbatim linethickness/mathcolor/bevelled.
- **F18** `nth-root` arg order: all THREE consumers had (degree, radicand)
  swapped — `<mroot>` was spec-backwards, cmml `<degree>` wrapped the
  radicand, unicode_math used the radicand as index.
- **F7** mathstyle→`m:mstyle` propagation (stylemap tables, needsMathstyle,
  XMApp/XMArray/bigop/script wraps, mode-sensitive entry baseline
  display↔text) — `\tfrac`/`\dfrac`/`\displaystyle` sizing.
- **F8** faithful mo styler (opdict xor-emission, largeop/movablelimits/
  symmetric, minsize/maxsize stretchyhack, %→em size resolution, mathsize on
  all token types). **F9** `pmml_maybe_resize` wired at all five call sites.
  **F4** `fmt_em` byte-parity (`%.3f`, trailing zeros kept).
- **F3/F6/F11** spacewalk rewritten as Perl's stream algorithm (mrow/script
  unwinding into the pair stream, negative-target mpadded via string metrics,
  mspace merge, both-mo target/2 split; Hint widths normalized to em,
  `_ignorable` + filter_row).
- **F14** content-MathML: multirelation → `m:and`-chained pairwise applies
  with `m:share` (generateNodeID port), or-composition, and STYLIZED ci
  content (plane1: `<ci>𝑥</ci>` — formerly raw ASCII on every cmml
  identifier), decorated symbols, Perl-regex integer test.
- **F16** OperatorDictionary Content_form/fence tables REGENERATED verbatim
  from Perl's range strings (machine-parsed, non-overlap asserted) — closes
  the arrow/negation codepoint holes and the U+2A50 misclassification.
- **F15** `do_cfrac` unrolling behind the `cfrac-inline` gate + the amsmath
  `\cfrac` binding rewritten to Perl's trampoline (capture surrounding
  mathstyle once, nested reuse, no size compounding).

- **F8b** inherited-context bindings (same day): `{\color{red}$a+b$}` math
  now colors its tokens (was black — visible arXiv bug class); + a latent
  rust-libxml misaligned-ns-read crash fixed in `find_inherited_attribute`.

- **lxDeclare dead-predicate class — ✅ COMPLETE 2026-07-03, exact Perl
  parity 84/84** (PR_READINESS cluster C). Stage 1 (core, `e11ee74f8e`):
  dead `@font`/`@meaning` XPath predicates replaced by Rust-side font-CLASS
  filtering (`_declare_font` → declare_node_matches) and `(@meaning|@name)`
  predicates; replace-rules carry the same declare-side filter (new
  `declare_filter` rewrite option); untagged `scope=section` gates the fast
  path via an explicit scope_prefix field (51 → 67 decl_id, strictly
  additive). Stage 2 (residual families, 67 → **84 = Perl, zero
  per-declaration diffs**): (1) function-application patterns
  `f\WildCard[(\WildCard)]` / `(\WildCard,\WildCard)` — new "funcapply"
  compile arm + exact-adjacency filter, `_nowrap` now threaded from the
  keyval (was parsed but never read); (2) the wrap path's XMDual rebuilt to
  Perl's exact `XMDual[XMApp(op,refs), XMWrap(span)]` shape — the old
  "flat" R11 variant (presentation tokens as direct dual children) was
  DESTROYED downstream, silently dropping the matched span (`g(a)` → bare
  `)`); dead restructure_scripts_in_dual deleted; (3) multi-wildcard
  subscripts `q_{\WildCard,\WildCard}` now require the literal comma-list
  (child 2i-1 wildcard paths) while 1-ary `q_{\WildCard}` keeps Perl's
  whole-argument "accidental" match; (4) leading-wildcard `\WildCard[a]b`
  ("leadwild" arm) + the rewrite-creation gate now accepts decl_id-only
  (tag-only) declarations like Perl; (5) `\lxDefMath` tag/description →
  next_declaration_id + `decl_id` through DefMathI (use-site stamping) +
  the `\@lxDefMathDeclare` constructor (declare element, digested
  description); (6) `\weird{\WildCard}{\WildCard}` ("cmddual" arm — marks
  the use-site XMDual, Perl's single-XMDual branch); (7) declare elements
  now carry Perl's `<tags><tag role="term">` (digested math, itself
  rewrite-marked) + `role="short"` + `<text>` description via
  normalizeDeclareKeys/splitDeclareTag. The unrecognized-pattern Warn now
  prints the offending pattern. Suite 1517/0; declare.xml re-blessed.

Open queue lives in the audit doc: F17 misc, F14 share-suffix wiring,
**F5** linebreaker: DECIDED 2026-07-03 — no linebreaking work on the
`ar5iv-2606-prep` branch (user directive); remains a feature gap (Perl
gates on `--linelength`, default OFF → not a production divergence). Method traps recorded in the doc
(installed Perl 0.8.8 lags the reference tree; trace producer-vs-consumer
before patching post).

- **F19 FIXED 2026-07-03** — role-carrying XMWraps were unparseable by
  construction: `parse_children` sub-parsed them with the role in place, and
  `node_to_grammar_lexemes` emitted `start_ROLE…end_ROLE` wrapper tokens no
  grammar rule consumes (the grammar only knows the script roles), so
  `\mathrel{\mathop{=}\limits^{def}}`, `\mathop` nesting, extensible-arrow
  labels, and siunitx unit wraps ALL fell to the kludge. Perl never lexes the
  wrap's own role: it parses the children, then copies the wrap's attributes
  (role included) onto the replacement. Ported exactly: strip non-script
  roles pre-parse, re-apply to the result, and mark it `_rewrite` so the
  lexer treats the pre-parsed replacement as ONE atomic terminal (the
  `_rewrite` lexer arm now also updates bigop context so a following script
  lexes BIGOPSUB/BIGOPSUP). Four goldens re-blessed, each verified formula-
  by-formula against `LaTeXML/t` reference goldens: mathtools extensible
  arrows + testscripts nested-`\mathop` + si unit-wraps now byte/shape-match
  Perl; physics S1.Ex7 (`\overrightarrow{\mathbf a}` etc.) recovered from
  whole-formula unparsed to Perl-identical shapes. KNOWN micro-residual:
  `physics` `\PV`'s `P.V.` wrap — Rust's generous grammar parses the
  punctuated content Perl rejects, so the presentation gains a nested
  role-less mrow-equivalent (semantic string unchanged, `fragments@`/`list@`
  head divergence pre-existing).

### Landed this session (2026-07-02, on `ar5iv-2606-prep`) — live-run fatal mining: 2 panic sites, `\dabar@`, plain-`\+` retraction

Mining the in-flight full-arXiv rerun's fresh fatals (6.9k at ~32% corpus) produced
four fixes, each witness-verified against same-host Perl:

- **Graphics worker-join panic (15 papers)** — `graphics.rs` `join().unwrap()`
  escalated a pressure-induced worker-thread panic into a whole-conversion
  `Fatal:panic`. Now degrades per the function's own design: payload surfaced as
  `Error:imageprocessing:worker_panicked`, survivors' outcomes kept. Witness
  1811.01777 converts clean standalone (pressure-dependent, not paper-dependent).
- **`parser.rs` `Node::new().unwrap()` panic (2 papers)** — allocation failure in
  kludge-script restructuring now records `Error:misc:allocation` and returns the
  base un-scripted (`Result` threaded `new_script_node` → `kludge_scripts_rec` →
  `parse_kludge`); a genuine OOM then dies via the designed RSS watchdog.
- **`\dabar@` runaway (31 papers)** — KNOWN_PERL_ERRORS #40: real `amsfonts.sty`
  defines the dash glyph; both bindings omitted it, and author copies of the
  `\xdashrightarrow` snippet `\@whiledim`-loop on a 0-width `\sbox` of it forever
  (Rust's real label widths → `Fatal:Timeout:TokenLimit`; Perl escapes only via
  all-zero box widths). Binding now defines it (`╌`); witness 1705.09248
  180s-Fatal → completes with 1 error (same class as Perl's 2); pdflatex ground
  truth compiles. Reproducer `docs/reproducers/xdasharrow_dabar_whiledim_loop.tex`.
- **plain-`\+` retraction (Rust-only fix; part of the 516-paper
  `\lx@begin@alignment` TooManyErrors family)** — real LaTeX (INITEX-based) never
  defines plain.tex's `\+` (= `\tabalign`), but Rust's latex layer inherited it
  from the plain dump, so an author typo `\+` in math expanded into `\halign` and
  detonated a 102-error mode cascade (witness cond-mat0001412; Perl: 1 undefined
  error). The latex format loader (`latex.rs`, at the "kernel layer complete"
  seam after the dump/base branch) now retracts the inherited definition
  (guarded on the body still being plain's bare `\tabalign`; new
  `state::remove_meaning_global`). Witness now: exactly 1 error `undefined \+`
  — byte-parity with Perl. Watch the same class for other plain-only macros
  (`\tabalign` invoked directly, etc.) if cascade signatures persist in the
  next run.

Triage byproducts: `\tikzcdmatrixname` PushbackLimit cluster (345 papers) verified
**PARITY** (witness 1304.2913: Perl `Fatal:too_many_errors` in pgfmath, 44 s) —
known tikz-cd deep-divergence territory, not chaseable; `never_completed` (1,069)
spread evenly across months (governor sheds/hangs, overlaps STABILITY_WITNESSES).

**Plain-layer leakage audit (same day, follow-up to the `\+` fix).** The
layering is Perl-identical (Perl's `TeX.pool.ltxml:23` also runs
`LoadFormat('plain')` under LaTeX.pool); the divergence is content — Perl's
plain layer is the hand-curated `plain_base.pool.ltxml`, Rust's is the dump of
REAL `plain.tex`. Name-diff (plain-dump CSes − CSes mentioned anywhere in
Perl's engine pools − latex-dump-redefined CSes, `LC_ALL=C`): **55 survivors**,
coherently two subsystems plus stragglers: (1) the plain **tabbing machinery**
(`\tabalign`, `\settabs`/`\sett@b`/`\s@tt@b`/`\s@tcols`, `\cleartabs`, `\tabs`,
`\tabsdone`, `\tabsyet`, `\t@bbox`/`\t@bb@x`, `\m@ketabbox`, `\us@*`, `\if@cr`+
friends) — the `\+` family; (2) the plain **output routine** (`\plainoutput`,
`\pagebody`, `\pagecontents`, `\makeheadline`, `\makefootline`,
`\dosupereject`, `\@ins`, `\if@mid`/`\ifp@ge`/`\ifr@ggedbottom` + setters);
(3) inert stragglers (`\Orb`, `\oldstyle`, `\preloaded`, `\getf@ctor`, `\m@g`,
`\p@renwd`, `\if@`, `\@nother`) and record-format artifacts (`%NN`,
`\skewchar\<font>`, `count/dimen/skip254`). Live-run evidence: zero errors key
on any of these names (only `\+` was a typo-magnet; the rest execute only when
intentionally invoked and are silent if they work). **DECIDED 2026-07-02
(user): keep and watch** — the remaining tabbing entry points stay defined as
beneficial plain coverage; revisit only if next-run cascade signatures
implicate them. Regenerate the list with the three-set diff above.

**Resolved en passant (2026-07-01 lxDeclare session):** the long-standing
"DefMathRewrite `\WildCard` subscript bug" (wildcard-subscript rewrite not
firing; `math/simplemath` fixture encoded the buggy `Unknown@()` output) was
fixed by the `\lxDeclare` B+C parity work (`dd226d1973`, `d74529d9eb`,
`786d9ed89d`) — simplemath is now byte-identical to same-host Perl and the
golden was re-blessed.
