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
[`MATHML_POST_LINE_AUDIT_2026-07-05.md`](MATHML_POST_LINE_AUDIT_2026-07-05.md) (verdicts for all
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


---

## Lifted from `SYNC_STATUS.md` on 2026-07-25

Completed / historical sections, moved here verbatim so the live worklist stays
actionable (CLAUDE.md: "Completed session logs are lifted to
`docs/archive/SYNC_SESSIONS_*.md`"). Every item below was verified landed before
the move — by its named guard test present in the tree, not by its own label.
Nothing here is open work.

### apxproof bibliography + option-value catcode (LANDED 2026-07-10)
Rust Error Fix. `gdsm.tex` (biblatex + `\usepackage[bibliography=common]{apxproof}`)
now converts error-free in every config (bare / `--includestyles` / ar5iv): 24
linked bibitems, 6 `ltx_proof` (amsthm markup, correctly inline — apxproof defers
only its own `apxproof`/`proofatend` envs). Two parts:
1. **`latexml_contrib/src/apxproof_sty.rs`** — force-raw-loads `apxproof.sty` in
   all configs (no Perl binding exists; Perl aborts the bib on kvoptions
   `\ProcessLocalKeyvalOptions*`). Surpass-Perl; see KNOWN_PERL_ERRORS #44.
2. **Core catcode fix** (`binding/content.rs`): `\opt@<name>.<ext>` now built with
   `ExplodeText!` (LETTER catcode) not `Explode!` (OTHER), so kvoptions/keyval
   `\setkeys` values pass catcode-sensitive `\equal`/`\ifx` validation. Broad
   reach (every `\DeclareStringOption` validator). See WISDOM #61; regression
   fixture `tests/keyval_options/optcatcode*`. Full suite 1538/0.

### Figure panels of unmeasurable images wrapped by filename length (LANDED 2026-07-10)
Rust Error Fix (fidelity). A float of bare `\includegraphics` with no explicit
`\\` is partitioned into rows by `arrange_panels_and_breaks`, using each panel's
MEASURED box width. `read_image_dimensions` reads PNG/JPEG/EPS only (like Perl's
`imgsize`), so for **PDF/SVG** it early-returned with no `cached_width`;
`compute_size` then summed the whatsit's argument boxes — including the
Semiverbatim **path string** — so panels wrapped by *filename length*.
arXiv:2409.16471 fig 2 (12 uniform `0.245\textwidth` PDF panels) split 3/3/2/3/1
instead of 3 rows of 4. **Fix** (`latexml_core/src/util/image.rs`) emulates
pdfTeX, not Perl: on a raster-reader miss, read the natural size from the file
itself — a PDF's CropBox→MediaBox (pdfTeX's default, shared with
`LaTeXML::Post::Graphics::read_pdf_page_box`) or an SVG's viewBox — and apply the
graphicx transform in points. Only when the page box is hidden in a compressed
object stream do we fall back to the requested `width=` (else 0); `cached_width`
is always set so the filename is never summed. No ImageMagick dep (that is a
Perl-only workaround for Image::Size's lack of PDF support; even it forces
`use-cropbox` to match pdfTeX). Verified against `\the\wd` under pdflatex:
`width=` → the request outright; bare/`scale=`/`height=` → the natural box.
Corpus-wide reach but NARROW — `width=` figures get an identical box width either
way, so only no-explicit-width PDF/SVG figures change; a 260-paper before/after
sample (142 with PDF figures) showed 0 error/fatal/exit-code regressions, and the
2 layout changes were previously-merged multi-panel figures now wrapping into
rows (e.g. 8 panels → 2 rows of 4). Golden suite untouched (all-PNG/JPEG).
Regression tests `figure_panel_native` + `figure_panel_unmeasured`. See WISDOM
#62. Fig 2 → uniform 84.52pt → 3 rows of 4, 0 errors.

### `\halign`-in-math runaway (Cluster H #2 / kbordermatrix) — ✅ LANDED 2026-07-20

Rust Error Fix, **surpass-Perl**. The long-standing "HIGH difficulty, post-release"
`\lx@begin@alignment`/`\halign`-in-math crash turned out to be a one-line
**inherited-kernel-macro leak**, not deep frame surgery.

Rust raw-loads `latex.ltx` into the kernel dump, so it has the real
`\@arraycr`/`\@xarraycr` (L16583-16585); **Perl LaTeXML has neither**. That body
balances TeX's `align_state` with ``${\ifnum0=`}\fi … \ifnum0=`{\fi}${}\cr``,
valid only under a real `\halign` — digested by LaTeXML it re-opens an inline-math
frame the alignment's column-after template cannot balance. Any macro using the
documented `\bordermatrix` idiom `\let\\\@arraycr` inside its own `\ialign`
therefore leaked → `Attempt to close a group that switched to mode math` → runaway.
Fix: `Let!("\\@arraycr", "\\lx@alignment@newline")` in `latex_constructs.rs`,
beside the `\@tabularcr` retraction Perl already performs
(`latex_constructs.pool.ltxml:3612`).

- **arXiv:2605.23849** (the Cluster H witness): ~149 s runaway → token-limit Fatal,
  **0 formulae** ⇒ **1.9 s, 0 errors, 985 formulae / 8 XMArray, 1.34 MB**. Same-host
  Perl: 52.7 s, 3 errors, identical 985/8 counts — so Rust is now faster AND
  error-free at equal structure.
- **arXiv:2605.05194** (found by corpus scan): 125 errors + `Fatal:TooManyErrors`
  and a **39-byte** (empty) document ⇒ **0 errors, 422 KB**.
- Breadth **6 / 6,000** 2605 papers (0.1%); the other hits are byte-unchanged.
  Neutral by construction — no Rust binding and no `.ltxml` names `\@arraycr`.
- Suite **1614/0**, clippy clean. Guard `tests/alignment/arraycr_halign`.

**Two prior hypotheses were wrong; do not retry them.** (a) "Make `egroup`'s
mode-switch recovery degrade like Perl" — Perl was *skipping* the matrix (its `\\`
was undefined), not recovering, so matching its error count would have meant
matching a content loss. (b) The `\lastbox`/`\unhbox` box-peel repro is a
different, SHARED loop. See WISDOM #64 for the reusable bisection method
(hand-expand the suspect macro) and `docs/known_crashes/kbordermatrix_halign_math/`.

### Stale-autoload-trigger runaway (Cluster H #1 + #3) — ✅ LANDED 2026-07-20

Rust Error Fix. The remaining two Cluster H runaways — long framed as "Rust
error-recovery *loops* where Perl keeps *advancing*" and expected to need
separate per-mechanism gullet surgery — were **one bug**, in `def_autoload`
(`latexml_engine/src/tex.rs`).

The autoload closure's "package already loaded → just re-emit the trigger CS"
branch is correct only when a **different** CS was `\let` to the trigger (the
`\varmathbb` case it was written for, arXiv:2310.13684). But `<pkg>.sty_loaded`
is assigned **globally** while the package's macros install at the current
frame, so loading a package or class **inside a group** pops the macros and
keeps the flag — leaving the globally installed trigger as the CS's only
definition. It then re-emits *itself* forever, and because it emits **no
`Error:`**, the `too_many_errors` cap is never reached; the run grinds ~42 s to
the token limit and writes a 39-byte document. Fix: when the CS that fired the
closure IS the trigger itself, clear the stale trigger globally so the CS takes
the ordinary bounded undefined path.

- **2606.21610** (Overleaf/Springer `\IfFileExists{sn-jnl.cls}{\documentclass…}`
  template): 42.9 s `Fatal:Timeout:TokenLimit`, empty output ⇒ **0.203 s**,
  bounded `Fatal:TooManyErrors:MaxLimit(100)`. Perl: 1.1 s / 102 errors /
  `too_many_errors:100` — same verdict, **5× faster**.
- **2605.21013** (undefined-macro cascade, was `Fatal:Timeout:IfLimit` at 107 s):
  43.1 s ⇒ **0.203 s**, same bounded verdict. Perl 1.9 s — **~10× faster**.
- Both papers are genuinely broken LaTeX (pdflatex fatals too), so the win is
  *failing like Perl instead of grinding*, not converting them.
- Known `def_autoload` regression traps re-verified clean: 2310.13684 (0 err),
  1403.6801 (0 err), 1711.11576 (1 err, 3.5 MB).
- Suite **1615/0**, clippy clean. Guard `tests/100_stale_autoload_no_runaway.rs`
  (6-line self-contained repro; verified red at 54.7 s without the fix).

**Ground truth, recorded but deliberately NOT ported:** real LaTeX rejects the
premise outright — `\@fileswithoptions` (latex.ltx L18700) errors *"Loading a
class or package in a group"* when `\currentgrouplevel > 0`. Porting that guard
would give a better message. **Updated 2026-07-23 (#311):** we now reproduce that
invariant rather than its enforcement — `require_package` hoists a load's
definitions past LaTeXML's own brackets, marked `subfile:<depth>`
(OXIDIZED_DESIGN #65, KNOWN_PERL_ERRORS #55). The guard would still need an
exemption for those brackets and would add only the message, so it stays
unported. This section's witness `2606.21610` is a *different* shape: its braces
are `\IfFileExists`'s branch arguments, executing as a group only because
`\IfFileExists` is itself undefined that early — it too keeps losing its class.

**Diagnostic gap closed alongside:** the `TokenLimit` fatal previously printed
only "infinite loop?" with no window — the cycle guard dumps its repeating
tokens, but a run that reaches the *token limit* is by definition one the cycle
guard did not recognise, i.e. exactly the case with no other clue. It now dumps
the same recent-token ring under `LATEXML_DEBUG_FATAL` (and the ring fills
before the guard activates, so a lowered `LATEXML_TOKEN_LIMIT` still captures
it). That dump is what identified this bug in one run.

### Reproducer re-verification + 400-paper output-neutrality sweep (2026-07-20)

Validation pass for the two fixes above, which also **re-dated every committed
reproducer**. Both halves changed the worklist more than the fixes did.

**A. 400-paper corpus sweep, baseline (`381efaf81b`) vs fixed, same sample.**
`0` error-count changes, `0` fatal-class changes, total wall 575.9 s → 578.9 s
(+0.5%, noise). 26 papers differed by 1–21 bytes — **re-running those solo gave
byte-identical output from BOTH binaries**, so that is run-to-run
nondeterminism under parallel load, not a behaviour change. Neutrality of the
`\@arraycr` retraction is anyway structural: nothing else in the tree names it.

*Sweep-harness caveat worth reusing:* a naive `grep -rl '\begin{document}'`
main-file pick manufactured 2 of the 4 apparent "fatals" (it chose
`figures-pgf/tinylora_preamble.tex` and a fragment instead of the real main) —
the trap `SYNC_STATUS` already records for the bibliography sweeps. With the
right main, **all four are fine**: `2605.30585` Rust 0.2 s/102 err vs Perl
2.0 s/102 err (exact parity, 10× faster); `2605.12207` Rust 0.3 s/39 err vs
Perl **3 m 57 s**/47 err; `2605.14493` and `2605.25400` fail in BOTH engines,
Rust in 15 s / 8.5 s vs Perl timing out at 200 s. **Zero Rust-only regressions
in the sample.**

**B. Every committed reproducer re-run against same-host Perl.** Several
long-standing "OPEN, GENUINE-RUST-ONLY" entries are **already fixed** — they
were stale, and left in place they mis-rank the whole worklist:

| reproducer | recorded | measured 2026-07-20 |
|---|---|---|
| `1610.00974_multicolumn_pcell_newline` | OPEN, Rust-only, 502 err + Fatal | **0 err**, and the full paper `Nikbakht.tex` **0 err** |
| `array_pcolumn/B_prefix_alignment_td_align` | OPEN (`align="justify"` vs Perl `left`) | **byte-identical to Perl** |
| `array_pcolumn/C_m_column_vbox_rendering` | OPEN, deferred (2 structural diffs) | **byte-identical to Perl** |
| `pcolumn_block_content_in_p` | OPEN, **BLOCKED** on the `\hsize`-invariant box model | **byte-identical to Perl** — that blocker no longer gates it |
| `ieeeeqnarray_leading_empty_cell` | SHARED (both engines fail) | Rust **0** / Perl **5** — the surpass-Perl half is done |
| `tabbing_math_code_env_2311.06609` (ar5iv #472) | Rust-worse | **11 = 11**, parity on the repro |

`1610.00974` keeps one structural difference from Perl, and **pdflatex says Perl
is the wrong one**: for `\multicolumn{2}{|p{1cm}|}{\centering A\\ B}` Rust makes
`B` a line break *inside* the merged cell while Perl opens a new `<tr>`;
`pdftotext -layout` stacks A/B in the single merged cell with `y z` as the next
row. Do NOT "fix" Rust toward Perl there.

The only reproducer still genuinely Rust-worse is `glossaryref_math_xmtok`
(Rust 12 / Perl 1) — and that Perl `1` is a **timeout kill**, not a clean run
(`rc=124`), confirming the recorded "blocked on an unrunnable Perl reference"
verdict is still current. **Method note:** the first pass of this table was
wrong for exactly that reason — always capture the exit code, or a
timeout-killed Perl reads as a 1-error success and flips the verdict.

### A recoverable Fatal no longer throws the whole document away — LANDED 2026-07-20

Rust reliability fix (**beyond-Perl**). `digest_internal` is written to keep
partial output after a recoverable Fatal ("Perl `finishDigestion` L219-220: loop
consuming input even after errors"), but the intent only worked when the failure
landed in a **later** body: `digest_next_body` accumulates into the stomach's
`box_list` and hands it back only on the success path, so a Fatal inside the
FIRST body left the caller's `boxes` empty and the run wrote a **39-byte empty
document**. One pathological `\tikz` picture cost an entire paper.

New `stomach::salvage_pending_box_lists` unwinds the stranded levels in document
order. Results on ar5iv user-report papers, all previously **0 bytes**:

| paper | issue | now |
|---|---|---|
| `2405.19920` | #522 | **1.82 MB** — 6 sections + **80 bibitems**, ~the complete paper. Same-host Perl: **5 min, 0 bytes**. |
| `2508.07407` | #556 | **31 KB** — title/authors/abstract recovered |

**Scope was narrowed by measurement, twice — both narrowings matter:**
1. For the stomach box-cycle guard the innermost level IS the pathology (a
   repeating window past 50k boxes), so it is dropped and only the suspended
   outer levels kept — "drop the offending construct, keep the document".
   Grafting the window in would produce a vast garbage document.
2. **Salvage fires ONLY for `ErrorTarget::Stomach`.** Extending it to the
   gullet's `Timeout:Recursion` looked reasonable (the token stream vs the box
   list) and was actively harmful: on `2605.25400` it revived a poisoned state
   that re-entered the same loop during build, turning an 8.7 s fatal into a
   **2 m 12 s wall-clock timeout writing a ZERO-byte file** — strictly worse
   than the 39-byte stub it replaced, for a 1.7 KB gain on the single paper it
   helped. The same reasoning bars `TooManyErrors`. Widening to either needs its
   own measurement; do not assume more salvage is better.

Validation: suite **1617/0**, clippy clean, and the 400-paper sweep vs
`381efaf81b` shows **0 error-count and 0 fatal-class changes**, wall 575.9 s →
579.5 s. Guard `tests/101_fatal_salvages_partial_document.rs` (verified red —
39 bytes, prose gone — without the fix). It asserts the Fatal is still reported:
salvaging partial output is not a licence to downgrade the diagnostic.

**Corrects a stale claim:** `docs/reproducers/tikz_calc_node_recursion_2508.07407.tex`
and the AR5IV notes said this fatal was "caught gracefully — conversion
COMPLETES, only the one tikz table is dropped". Re-measured, the full paper
produced a **0-byte** file. It is graceful *now*.

### Landed (each with a red/green guard, full suite + clippy green)
1. **`\@arraycr` retraction** — ended the `\halign`-in-math runaway. 2605.23849
   ~149 s→Fatal ⇒ **1.9 s / 0 errors / 985 formulae**; 2605.05194 ⇒ 0 errors /
   422 KB. Now surpass-Perl.
2. **Stale-`def_autoload` guard** — Cluster H #1 and #3 were ONE bug.
   2606.21610 42.9 s ⇒ 0.203 s; 2605.21013 43.1 s ⇒ 0.203 s, both landing on
   Perl's own verdict 5–10× faster.
3. **`salvage_pending_box_lists`** — a Stomach Fatal no longer discards the
   document. 2405.19920 (ar5iv #522) 0 bytes ⇒ **1.82 MB**; 2508.07407 (#556)
   ⇒ 31 KB.
4. **Issue #312 operand slot** — see the caveat below.
5. **Docs vetting** — three commits; see "what changed" in git log.

### Landed this session (real features, verified + committed)
- `--timestamp=STR` (`--timestamp=0` omits) → XSLT `TIMESTAMP` footer param;
  deterministic no-timestamp default (divergence from Perl's localtime).
- `--icon=FILE` → XSLT `ICON` param + favicon resource copy.
- `--nographicimages` / `--graphicimages` → gate the Graphics post-phase.
- `--numbersections`, `--mathparse`, `--invisibletimes`, `--defaultresources`
  → positive complements of existing negative-only flags (verbatim Perl-CLI
  parity; the negative wins if both are given).

### A package loaded in a LaTeXML subfile bracket lost its definitions (#311) — ✅ LANDED 2026-07-23 (branch `fix-311-standalone-newif-group`)

Rust Error Fix; shared upstream defect fixed ahead of Perl (KNOWN_PERL_ERRORS
#55/#56, RELEASE_CRITERIA §8). `content.rs::require_package` hoists the load's
**Meaning** delta past brackets LaTeXML itself opened, named `subfile:<depth>`
(activated by `standalone_sty.rs` after its `bgroup()` and by `import_sty.rs`'s
`\lx@save@paths`). The depth is load-bearing: `StashActive` is `Scope::Local` at
the bracket's frame, so a bare activity test is also true at deeper frames and an
author's `{\usepackage{…}}` *inside* a subfile preamble got hoisted too — Rust 0
errors where Perl reports 1. Reproducing needs a **raw** `.sty` under
`--includestyles`; a bound package installs globally already. Also fixed:
`\includefrom`/`\subincludefrom` declared one argument but used `#3`, dropping
the file in silence; and three `state.rs` scope bugs (activity is the FRONT
`stash_active` value, not key presence — so a deactivated scope read active
forever, could never be re-activated, and a second `deactivate_scope` re-popped).
Mechanism, refuted alternatives, boundary and the Meaning-only partial:
OXIDIZED_DESIGN #65 + WISDOM #66. Guards: `06_cluster_regressions` (5 new) +
`state::reentrancy_tests` (3 new).

### Rhai `LookupDefinition(cs).push*` hook-splice re-installs at same-level, not global — ✅ LANDED 2026-07-21

Follow-up to the BookML/@xworld21 cluster: PR #333 review comment r3623947537 flagged that the
`LookupDefinition(cs).push*/unshift*` hook-splice (#321) re-installed the patched def at
`Scope::Global`, which **promotes a locally-bound def to global** and makes the patch survive
group exit — a divergence from Perl, which mutates the shared def-hash *in place* (never touching
the save stack). Harmless in practice (BookML only patches already-global `\hrule`/`\vrule`/`\rule`),
but a real gap. Fix: ported Perl `State.pm:175`'s fourth scope, `'inplace'` ("Special case for
`\box` & friends"), as first-class **`Scope::InPlace`** (`latexml_core/src/state.rs` enum +
`assign_internal` arm; `\globaldefs` deliberately does NOT re-scope it, matching Perl's
`$scope ne 'global' && ne 'local'` guard). The 9 `install_definition(d, Some(Scope::Global))` sites
in `script_bindings/wire.rs::push_definition_hook` now pass `Scope::InPlace`. The Value-table
`assign_value_inplace` fast path (WISDOM #19) was the pre-existing witness that this scope existed;
this lifts it to the Meaning table too. Guards: `state::reentrancy_tests::inplace_scope_keeps_the_bindings_level`
(proves neither-Global-nor-Local across a `push_frame`/`pop_frame` boundary) + the existing
`script_bindings::tests::lookup_definition_*` (unchanged — top-level pushes, where in-place ≡ global).
See WISDOM #48.

### XSLT `LATEXML_VERSION` param — generator-stamp parity gap + BookML `utils.xsl` — ✅ LANDED 2026-07-21 (branch `xslt-latexml-version-param`)

Completes the BookML/@xworld21 cluster follow-up. `latexml_oxide/src/post.rs` now injects
`LATEXML_VERSION` (= OUR Cargo `X.Y.Z`, `core_interface::LATEXML_VERSION`, #320) into the XSLT
params — mirrors Perl `LaTeXML.pm:562`, restores `LaTeXML-common.xsl`'s `LaTeXML_identifier`
generator stamp (`<!--Generated by LaTeXML (version X)…-->`) that oxide had been silently
omitting (empty param → `<xsl:if>` false), and gives BookML's `utils.xsl`
`b:version-leq($LATEXML_VERSION,…)` a non-empty value. Inserted before the user-override loop,
so `--xsltparameter LATEXML_VERSION=…` still wins (Perl's `LATEXML_VERSION:TEST` idiom).
Verified empirically: oxide's serialization keeps XSLT-emitted comments; no active test
full-compares an HTML golden, so **no re-bless was needed** (the `07/08/09_xslt_*`, `001`/`002`
tests are structural/`.contains`; `hello_new.html` + `daemon/formats/*.xml` are orphaned
Perl-copied artifacts). Guard: `tests/10_xslt_generator_version.rs` (default stamp carries the
const version, read dynamically → no version-bump churn; + explicit-param override wins).

### TL2026 `latex.ltx` dump init is NOT release-gate-clean — expl3 catcode gap (2026-07-12) — ✅ CLOSED 2026-07-23

> **✅ RE-MEASURED 2026-07-23 — BOTH TL2026 blockers are clear; 2026 is IN the
> release dump window.** Measured the way the release actually runs it, rather
> than on a local install: a kpathsea-UNLINKED dumper built exactly as
> `release-dumps.yml`'s `build-dumper` job does (`KPATHSEA_NO_LINK=1
> KPATHSEA_SKIP_TOOLCHAIN_CHECK=1 cargo build --release`, `ldd` asserted
> kpathsea-free), run inside the real `ghcr.io/tkw1536/texlive-docker:2026`
> under the verbatim gate (`LATEXML_INIT_DEBUG=1`, ANSI-strip, `grep -acE
> '^(Error|Fatal):'`):
>
> | init | 2026-07-12 | 2026-07-23 | 2026-07-25 |
> |---|---|---|---|
> | `--init=plain.tex` | exit 0, 0 errors | exit 0, **0 errors** | exit 0, **0 errors** |
> | `--init=latex.ltx` | exit 0, **137 errors** | exit 0, **0 errors** | exit 0, **0 errors** |
>
> The 07-25 column re-confirms the gate on `main` @ `b36c6cd21c` — 22 commits
> later — by the same method: 934 plain entries, **24,199** latex entries
> (24,221 lines), i.e. the 07-23 dump unchanged. The host's own TL2025 tree
> gates 0/0 on the same binary. **The stale "re-run the init gate before
> trusting the recorded blocker" residual in `SYNC_STATUS.md` is retired by
> this.**
>
> **Reproducing one year locally costs two false starts** — both surface only at
> `docker run`, and the first reads as a *clean* gate. Build the dumper in a
> container OLDER than the TL image, never on the dev host: a 2026-07-25 host
> (glibc 2.43, `libxml2.so.16`) yields a binary `:2026` (glibc 2.41,
> `libxml2.so.2`) cannot load, which the gate reports as `exit=127 errors=0` —
> **zero errors, because nothing ran.** `rustlang/rust:nightly` (bookworm, glibc
> 2.36) works, plus `clang libclang-dev`, which GitHub's runner preinstalls but
> a bare image does not (bindgen/`libmarpa-asf-sys`). `KPATHSEA_NO_LINK=1` is
> what keeps the binary kpathsea-free on a host that HAS libkpathsea-dev.
>
> The **likely** closers (not bisected — the 0/0 result is measured, the
> attribution is inferred) are the two expl3 fixes that landed **2026-07-20**,
> after the measurement below: force `\ExplSyntaxOff` when `_` is still LETTER
> (`latex_constructs.rs`) and the global `:`/`_`/`~` restore (`expl3_sty.rs`).
> They are the same pair credited with closing
> [`EXPL3_CATCODE_GAP_2026-06-08.md`](../parity/diagnostics/EXPL3_CATCODE_GAP_2026-06-08.md),
> and all three error families listed below are now gone (the 90 ×
> `unexpected:_` was the bulk). Dump is sound, not degenerate: **24,221** latex entries vs
> 2025's 21,997 — the delta IS TL2026's expanded l3 `text-case` module — and
> the plain dump is byte-identical to 2025's.
>
> The **container** blocker (TODO(#217), independent) cleared too:
> tkw1536/historic-texlive-docker#1 merged 2026-06-08 and `:2026` publishes on
> the SAME `none-5.42.0`/debian:trixie base as `:2025` — libxml2 apt candidate
> 2.9.14 → `libxml2.so.2`, glibc 2.41 ≥ the ubuntu-22.04 build host's 2.35 — so
> the one-binary-serves-all-containers design holds unchanged.
>
> Landed: `release-dumps.yml` matrix + **all five** of `release.yml`'s
> `verify dump window completeness` gates (they are duplicated per build leg —
> `build-macos`, `build-macos-intel`, `build-linux-arm64`, `build-windows`,
> `release`; update them together or the legs disagree about the window) now
> span **2022–2026**. Validated end-to-end, not just at the gate: a binary embedding
> only TL2025 warns `latex_dump:mismatch loaded the TL2025 kernel dump, but the
> ambient TeX Live is 2026` on a TL2026 host; rebuilt with the 2026 dump the
> warning is gone and the conversion is clean. Cost: 48.1 → 49.0 MB (`release`
> profile), i.e. ~876 KB gzipped, against the 64 MB RELEASE_CRITERIA §2 cap.
> `release-dumps.yml` was also dispatched for real (run 30014067643): all five
> legs green.
>
> **Runner-disk flake on the 2026 leg — `generate` no longer uses a job-level
> `container:`.** The first CI run went 5/5 green; an identical re-run failed
> *only* the 2026 leg with `failed to register layer: … no space left on
> device` (3 pull retries, all out of space). 2026 is the fattest image —
> compressed 2022=4.6, 2023=5.0, 2024=5.4, 2025=5.7, **2026=6.2 GB** (9.8 GB
> extracted), monotonically worse each year — so it is the first to fall over.
>
> **Careful with the diagnosis** (an earlier draft of this note got it wrong):
> this is *not* a steady-state capacity ceiling. Instrumenting the job shows
> `/` at **145 GB total, 88 GB free before any cleanup** — roughly 5× the ~16 GB
> the pull peaks at. The pass and the fail landed on *different runner image
> versions* (`ubuntu-24.04` `20260714.240.1` passed, `20260720.247.2` failed),
> so the fleet is heterogeneous and some images come up with far less headroom.
> There is no way to pin a runner image *version* on GitHub-hosted runners, so
> the fix has to be tolerance, not capacity.
>
> Fix, in two layers of *tolerance* (not capacity): (1) drop `container:` (it
> pulls during "Initialize containers", *before* any step can run, so nothing
> can free space first) and `docker run` the image explicitly after a
> `free runner disk space` step that drops the preinstalled Android SDK / ghc /
> dotnet / CodeQL trees — measured 58 GB used → 30 GB, ~28 GB of margin ahead of
> the pull; and (2) a `pull TL image (retry-tolerant)` step that does an
> explicit `docker pull` with reclaim + linear backoff (10/20/30/40 s) across 4
> attempts, so a transient network / marginal-disk first failure clears on
> retry (`docker run`'s implicit pull is a single, non-retried attempt).
> Verified by extracting each step's script straight out of the YAML and running
> it verbatim — the gate against the real `:2026` image (byte-identical
> artifacts), the retry loop's control flow via a failing-`docker` stub (4 tries
> then exit 1) — plus a 5/5 green CI run. **Do not "simplify" back to
> `container:`.** This matters more than it used to, because every `release.yml`
> build leg now hard-requires the full window, so one flaky leg blocks a whole
> release.
>
> **Observed in passing — dumps are not bit-reproducible (pre-existing, ALL
> years, not introduced here).** The CI-produced `latex.2026.dump.txt` and the
> local one are identical in size and entry count but differ in exactly ONE
> byte-range: `V\ttexsys.aux_contents` embeds a wall-clock stamp
> (`2026/07/23:13:54` local vs `:14:13` in CI). Harmless today (cosmetic
> `texsys.cfg` capture), but it means two dumps of the same TL year + same
> revision never compare equal, so any future "did the dump change?" check must
> normalize that field rather than `cmp`.
>
> History below kept for the root-cause trail.

Blocked adding **2026** to the release dump window (`release-dumps.yml`,
then 2022–2025; see also the container blocker TODO(#217) — the two are
independent). Measured on a full local TL2026 install (`x86_64-pc-windows-msvc`,
but Linux-equivalent — this is the raw-load path, not a platform issue), using
the exact release gate (`LATEXML_INIT_DEBUG=1 ./latexml_oxide --init=<init>`,
ANSI-strip, `grep -acE '^(Error|Fatal):'`):

- `--init=plain.tex` → exit 0, **0 errors** (release-clean). ✓
- `--init=latex.ltx` → exit 0, **137 errors** → would FAIL the gate:
  - 90 × `Error:unexpected:_ Script _ can only appear in math mode`
  - 29 × `Error:misdefined:# … catcode PARAM … should never reach Stomach`
  - ~18 × undefined l3 **case-change** internals (`\DeclareUppercaseExclusions`,
    `\DeclareCaseChangeEquivalent`, `\CaseSwitch`, `\@@text@case@aux`,
    `\NoCaseChange`, `\AddToNoCaseChangeList`, `\keys`/`\tl`/`\str`/`\clist`…).

Root cause: the **known deep raw-load expl3 catcode gap**
(`EXPL3_CATCODE_GAP_2026-06-08.md`; "four attempted fixes all regressed and
were reverted"), newly triggered by TL2026's expanded l3 `text-case` module,
which older TL (2022–2025, all gate-clean) did not exercise during init.
Distinct from the `\Declare*caseMapping` no-ops landed above (those are
different macros; that fix does not touch this). **NOT introduced by the
Windows branch** — pre-existing, surfaced only because that branch is the
first to run a full TL2026.

Note the two-bars distinction that hid this: `tools/make_formats*` write the
dump *despite* init errors (24,333 valid latex entries still land), and the
test fixtures don't exercise the affected macros — so `cargo test` is 1531/0
and everyday TL2026 conversion works, while the strict release gate would
still reject a TL2026 latex dump. "Usable dump" ≠ "gate-clean dump."

### TL2026 ambient-drift fixes (2026-07-12, windows-compatibility branch) — LANDED

Two suite failures surfaced by running against TL 2026 (bleeding-edge; CI's
ubuntu TL is older) — both root-caused and fixed TL-independently:

1. **`\Declare{Upper,Lower,Title}caseMapping` native no-op handlers**
   (`latex_constructs.rs`, next to the `DeclareText*` family). The TL2023+
   kernel case-mapping declarations ARE captured in the latex dump, so
   `\ifdefined` guards (greek-fontenc `lgrenc.def`) passed and the dumped
   expl3 kernel bodies executed — hitting the raw-load expl3 catcode gap
   (`EXPL3_CATCODE_GAP_2026-06-08.md`) and spraying `Script _` + undefined
   `\acc*` errors (81_babel `greek_test`: 87 errors → 0; real pdflatex is
   error-clean on the same fixture). LaTeXML cases via Unicode internally,
   so ignoring these matches the `ignoredDefinition` policy. **Perl has no
   handler either — same cascade expected there on TL2023+; candidate
   upstream.**
2. **tikz `ac_drive_components`: SKIPPED ON WINDOWS ONLY, kept live on
   Linux/macOS.** circuitikz 1.8.0 rewrote its path logic, lengthening drawn
   capacitor plates (12.4 → 12.68 in our SVG space). This coordinate tracks
   the exact circuitikz version, which is NOT pinnable (both Perl
   `FindFile_fallback` `[vV]?[-_.\d]+` and Rust version-strip a
   `circuitikz-X.Y.Z` request to the current binding) and **differs by the
   platform's TeX distribution**: Linux (apt texlive) and macOS (Homebrew
   texlive) ship an OLDER circuitikz → 12.4 = the committed golden; the
   Windows CI's `setup-texlive` net-install and any fresh `install-tl` get
   the NEWEST → 12.68. So the fixture is **compared on Linux/macOS (where the
   golden is deterministic) and skipped on Windows** via a `#[cfg(windows)]`
   `WINDOWS_GOLDEN_SKIP` guard in `latexml_test_single` — a Linux↔Windows
   portability difference, NOT a code divergence (the engine faithfully
   renders whatever circuitikz emits — it was testing circuitikz's version,
   not our code). Not TL-year-keyable (macOS and Windows are both TL2026 with
   different circuitikz — a `tl2026.xml` variant attempt regressed macOS CI
   and was reverted). `INTENTIONALLY_FAILING`/`ERROR_DEBT` don't apply (they
   gate error *counts*, not golden *diffs*). Discovered via Windows CI run
   `29219528633` (which fail-fast-stopped at 86_tikz; the complete tail came
   from a local `--no-fail-fast` run on a newest-circuitikz box — a faithful
   Windows-CI proxy — confirming 1530/0 with this one fixture out).

### Release-week stabilization (2026-07-10, user-directed) — THE LENS FOR THIS WEEK

**Public release is ~1 week out (branch `public-release-prep-week`). The bias is
STABILIZE, not add capability.** A regression introduced in release week is far
costlier than a feature deferred. So the actionable list below is re-ordered by
*risk*, not by *ambition*: the safe, landed-or-verification work leads; every
hot-path / broad-diff / deep-engine item is explicitly demoted to POST-RELEASE.

**SAFE — do in release week (low risk, high stabilization value):**

1. **Verify the already-landed >500 MB `index.xml` path on the release binary**
   (see the investigation note directly below). The foundation is **already in the
   release** — PR **#274** (`b0cc70f319`, squash-merged 2026-07-07): limit-safe
   DOM-walk queries so split fires + loud XPath errors, stream-the-file/skip engine
   init, CrossRef O(n²)→O(n) (42m50s→2m18s). So there is **nothing to land** (an
   earlier "not in the release branch" read was an ancestry-check error — #274 was
   squash-merged, so the branch SHAs aren't ancestors even though the content is).
   It fixes a **silent-failure class** (any doc large enough to cross libxml2's 10M-
   nodeset ceiling → NULL nodeset → swallowed → `[not split]`) and converts a
   document **Perl LaTeXML cannot** (Perl `latexmlpost` fatals at the nodeset
   ceiling in 8.67s). The release-week action is a **confidence check**: run the
   614 MB witness on the `maxperf`/release binary and confirm `Split into 40201
   pages` + byte-identical HTML (design-doc baseline 2m18s, ~21.6 GB peak; a 32 GB
   box handles it — watch RAM contention). *Excludes* the deferred two-pass
   streaming split (task #44 / `STREAMING_POST_DESIGN`) — that risky memory-only
   half is NOT needed for release.
2. **Full regression + smoke gate on the release binary** — the release
   discipline, pure risk reduction. `cargo test --tests --no-fail-fast` (expect
   ~1534/0), `cargo clippy --workspace --all-targets -- -D warnings`, then a
   `tools/benchmark_canvas.sh` smoke of a few hundred mixed papers on the
   `maxperf` binary, checking fatal classes against the known list + spot-checking
   HTML with the shipped CSS. (Mirrors the July-5 prep item 6.)
3. **Confirm the graceful-abort safety floors still fire** — these, NOT the deep
   loop fixes, are the release's real stability guarantee: the 4500 MB RSS fuse
   (Cluster A/D/E), IfLimit 16M / TokenLimit 1B (Cluster H), the 12k expand-depth
   guard + stack guard (Cluster F). All landed; this is verification only (a
   pathological paper must Fatal cleanly, never hang/segfault/OOM the process).

**DEFER to POST-RELEASE — do NOT start in release week (risk > reward now):**

- **All BP-1…BP-6 beyond-Perl perf levers** (below) — hot-path, output-neutrality
  gated, ambitious (rayon math parse, XSLT transpile, document-builder rewrite).
  A regression here is a release-killer; the 60k telemetry that motivates them
  keeps. **First post-release work, not release-week work.**
- **Cluster H deep runaway-loop fixes** (`STABILITY_WITNESSES.md`: `\kbordermatrix`
  box-peel, `\IfFileExists`-before-`\documentclass` readBalanced-past-EOF,
  undefined-cascade IfLimit). Genuine Rust bugs, but the fixes are deep
  gullet/box-register surgery with broad blast radius — AND current behavior is
  already SAFE (graceful Fatal via an existing limit ~100s in, bounded, no
  crash/corruption), so they are fidelity/perf gaps, NOT release-blocking
  stability risks. ~~The one clean regression (`2605.23849`, Perl completes) is a
  real fidelity loss whose fix is still deep.~~ **FIXED 2026-07-20** — and the
  premise was doubly wrong: Perl does not "complete" it (it skips the matrix),
  and the fix was one `Let!` retracting the inherited kernel `\@arraycr`, not deep
  surgery. All of Cluster H is now resolved.
- **`ltx_env_<name>` class enhancement** (below) — churns nearly every golden
  XML; running it in release week would swamp the regression baseline and mask
  real regressions. Isolated branch, post-release (as already noted).
- **MakeBibliography full re-port** (below) — already marked post-release.
- **`validate()` / `--validate`** (above) — already postponed to the next release
  (gated on the `rust-libxml` RelaxNG publish).
- **Verbatim-in-box items 4–6, biblatex `.bbl` `2605.17646`** (below) — low-value
  fidelity / graceful-fatal; not blockers.

*(Deliberately conservative: no contained "quick-win" bug fix in the current list
clears the risk/reward bar for release week — the parity long-tail is graceful
already. If a NEW same-host-confirmed GENUINE-RUST-ONLY regression surfaces from
the smoke sweep, that jumps the queue; nothing currently open does.)*

### Frontmatter-fidelity pass over the arXiv `html_feedback` reports — LANDED 2026-07-12

Drove the ~280 arXiv "front matter" `html_feedback` reports to clean, structured
frontmatter (branch `public-release-prep-week`). Method: convert each reported
paper to standalone HTML on the ar5iv config, then **Playwright red/green** DOM
checks (`.ltx_personname`/`.ltx_authors`/`.ltx_bibitem` counts + raw-macro-leak
regex). Two commits landed the class bindings: `12ccebefc1`/`537aac9e50` (20
classes), `3bc8a3342d` (JMLR structured author blocks + Wiley `MRM.cls`). See
[[frontmatter-class-bindings-2026-07-12]] memory for the binding patterns.

- **JMLR** (`jmlr_cls.rs`): `\Name`/`\Email`/`\addr` now digest **directly** into
  structured creators (name → personname, email/affiliation → contacts) instead
  of the generic `\and`/comma splitter, which crammed every author into one
  `<personname>` and split the affiliation's commas into phantom authors; `\nametag`
  no longer leaks. (Answers a user question on maximizing structured markup —
  beyond-Perl, Perl ships no jmlr binding.)
- **MRM.cls** (Wiley "Magnetic Resonance in Medicine", new `mrm_cls.rs`):
  `\author[idx]{name}{orcid}`, `\address`, `\corres`, `\finfo`, `\authormark`,
  `\state` (deliberately absent from OmniBus), plus own dep loads for ORCID/math/cites.

**Harness note (signal integrity):** the arXiv-source main-`.tex` detector must
skip `*-backup.tex` / `template/*` / `Rebuttal.tex` / `*_preprint.tex` /
versioned-subdir mains and *bonus* the file that carries the bibliography — an
early detector picked wrong mains and produced ~5 false "no authors / no
bibliography" reds (e.g. `2511.04594` = `Rebuttal.tex`). Corrected detector +
re-convert cleared them.

**Residual reds — all PARITY or already-beyond-Perl (NOT release-blocking):**
`2402.09505` (aa `\href`-in-name, parity/cosmetic), `2601.05137` (author `\def\name`
in a redefined `\@maketitle`, KPE #47 parity), `2403.07832` (minor `\footnotesize`
in a `\thanks`, no minimal repro); `2306.06628`/`2512.16391`/`2605.23904` (no
`\author` in source); `2508.20929` (atlasdoc author list `\input` in the body);
`2405.13705` (iidtp `\makeiidtp` `titlepage` suppresses the document title block —
**shared Perl XSLT rule**, and Perl *times out* entirely — authors show via the
titlepage ORCID links); `2505.13921` (neurips: Perl *times out*; Rust produces the
full doc with authors preserved in `<ltx:creator>` metadata, but the visible title
block doesn't render — a `\maketitle`-expandability interaction). The last two are
**beyond-Perl already** (Perl produces nothing).

### >500 MB `index.xml` (Nasser) — INVESTIGATED 2026-07-10

Witness `~/scratch/nasser/index.xml`: 614 MB, ~7M nodes, **40 000 one-equation
sections** (`solving_ODE` auto-generated notes), `--splitat=section`. Findings:

- **Perl LaTeXML cannot convert it.** The reporter's own `index.latexmlpost.log`:
  `latexmlpost` (0.8.8) dies `Fatal:perl:die … growing nodeset hit limit`
  (`XPath.pm:36`) in **8.67s** — libxml2's `XPATH_MAX_NODESET_LENGTH`. Perl's
  *core* also took **52m 7s** just to emit the XML (40000 formulae / 1577s math).
- **latexml-oxide CAN, and the fix is ALREADY in the release** (PR **#274**,
  `b0cc70f319`, squash-merged 2026-07-07 → ancestor of `public-release-prep-week`).
  With the foundation it converts fully: `Split into 40201 pages`, ~2m18s, peak
  ~21.6 GB, byte-identical across all pages (measured;
  `STREAMING_POST_DESIGN_2026-07-06`). A genuine **beyond-Perl** win (Perl outright
  fatals). Without the fix, `//*[@xml:id]` would overflow the 10M-nodeset ceiling →
  NULL → swallowed → `[not split]`, silently reproducing Perl's failure class — but
  that landed in #274, so the release-week action is only the confidence check in
  SAFE step #1, not a merge.
- **The lean-RSS half stays deferred (task #44).** Two-pass streaming split
  (21.6 GB → <1 GB) is unneeded for release (reporter has >64 GB RAM; eager path
  is correct + fast). Revisit only if a <64 GB target appears. Design preserved in
  `STREAMING_POST_DESIGN_2026-07-06.md`.

### Verbatim-in-box completeness (2026-07-04; breaklines LANDED same day)

Engine gaps behind the last ~1% of the 2605.00468 tcolorbox fidelity
arc (the class fixes — prevdepth glue transparency OXIDIZED #44, NFSS
family vocabulary #45, and the glowup verbatim contract — are landed):

1. ✅ **fvextra `breaklines` — DONE 2026-07-04**: the blanket
   `@Break→@NoBreak` line-processor neutralization in `fvextra_sty.rs`
   was an over-reach; only the `\FV@Break` char-scanner (the
   PushbackLimit/TokenLimit fatal source) needs relaxing. With the real
   `\FV@ListProcessLine@Break` running, every line is re-typeset as
   fvextra's `\parbox` (BOTH branches parbox — the over-wide one wraps),
   so the height budget counts the same wrapped lines pdflatex produces.
   Witness 2605.01024 (breaklines+breakanywhere fatal cluster):
   unchanged 4 errors, 0 fatals.
2. ✅ **Whitespace-river / 2× height budget — DONE 2026-07-04**: the
   `\lx@parbox` sizer was a pre-#2798 hand-rolled estimate
   (unwrapped-width/width, ceil, × baselineskip) that measured a
   one-line parbox at 2 baselineskips, inflating every breaklines
   prompt-box budget ~2×. Replaced with the faithful Perl delegation
   (sizer '#5' + Box::computeSizeStore: body through computeBoxesSize
   with the whatsit's width/vattach/totalheight; requested width wins).
   Also ported Perl's `\parindent\z@\parskip\z@skip` into the `\parbox`
   macro and the dropped `totalheight` property. 2605.00468 prompt-box
   fill 55–81% → **86% avg** (budget now line-exact on repro matrix).
3. ✅ **Leading spaces of verbatim lines — DONE 2026-07-04**: verbatim
   spaces are `\FV@Space` → `\FV@SpaceCatTen` (a braced ordinary space),
   eaten by TWO whitespace gates in the document builder (`open_text`'s
   initial-whitespace guard + `open_text_internal`'s Perl-L1146 gate)
   when the line's paragraph isn't open yet, plus the `ltx:p` afterClose
   trim. Fix: typewriter-font whitespace is never ignorable (guard
   bypass + `verbatim_space_pending` handoff + typewriter skip in
   `trim_node_whitespace`). JSON-schema indentation now preserved as
   REAL spaces (copy-paste-safe). Perl parity note: same-host Perl
   cannot convert these files at all (raw fvextra+breaklines exceeded
   7 min on a 6-line repro) — surpass-Perl scope.
4. **Prompt 1/6 budget undercounts wraps — paper-preamble-specific**
   (the remaining 2 spills on 2605.00468, 15/33px on 2/24 boxes,
   user-flagged 2026-07-05). CORRECTED diagnosis after bisection: NOT a
   `\small` attribution gap — in the paper the declared font at the fo
   AND its content block is serif-10 (traced), no size deltas exist to
   lose, and the budget counts NO wrapped lines for these boxes
   (~15 blocks × 12pt) while the browser wraps 6 borderline lines
   (383pt natural vs 345pt parbox width) → 19 rendered lines. The
   isolated repro chain does NOT reproduce (plain / breakable /
   breakable+title+colors all budget wraps correctly and emit `\small`
   deltas) — the trigger needs the paper's fuller preamble, prime
   suspect the colm class's inconsolata (`\ttdefault`=zi4) metrics vs
   cmtt in the line-width estimate (zi4 advance ≠ 0.525em → sub-list
   width/measure disagreement). Needs a preamble-bisection session with
   `LXML_SIZE_TRACE`; the speculative "anchor = declared fo font"
   change was built, traced, and REVERTED (no measurable effect — fo
   declared font equals the whatsit font in every observed case).
5. **Space-only verbatim lines still prune to empty** (blank-gap
   fidelity vs the PDF; render 0px + budget 0 = consistent, no
   overflow). Their spaces don't reach absorb (unlike line-leading
   ones); low priority.
6. **Non-verbatim `\ttfamily` lines in measured boxes don't wrap**
   (witness 2605.02240 `innercode`: `fontupper=\ttfamily\small` prose
   with `\\` breaks; pdflatex wraps each segment at the inner box
   width, our estimator emits one line-box per `\\` segment → 9–31px
   right pokes, ~2.7%). Same class as breaklines but general: paragraph
   wrap measurement inside measured boxes. Pre-existing (run-232-era
   binaries identical); not a July-5 blocker.

CSS side note: verbatim mono capacity is now token-derived
(`--code-font-advance` beside `--code-font-family`, `--tex-tt-advance`
constant) with `font-size-adjust: ch-width` upgrade where supported —
the browser font stays user-configurable; the conversion emits only TeX
facts (budgets + font-size anchor + abstract family). The breaklines
parbox shape has dedicated glowup rules (leaf-only `pre`/`pre-wrap`,
flex hbox rows, nested-picture fill-width exclusion).

### July-5 arXiv run — prep checklist (drafted 2026-07-02, user-approved sequence)

**Status 2026-07-05:** items 1, 3, 3b, 5 ✅ DONE — ar5iv-css **v0.9.0** released (on
jsDelivr); PR #273 merged → tag **`0.7.2`** "First public use of latexml-oxide in
ar5iv 2606" published (6 assets); `cortex_worker` rebuilt from tagged `main` +
fleet restarted; **ar5iv-editor redeployed to `latexml.rs`** (image
`20260705-9aafba841f`, public `/api/version` = `9aafba841f`, all services
healthy). Cross-repo required set is COMPLETE; items 6–8 are the run itself
(item 2 cortex/ar5iv CSS re-vendor: confirm).

Ordered; items 1–3 are cross-repo and REQUIRED (user, 2026-07-02):

1. **ar5iv-css `glowup`** — ✅ DONE 2026-07-05 (**v0.9.0** released, on jsDelivr):
   merged the `glowup` branch and **released a new ar5iv-css version**.
2. **Propagate ar5iv-css** to **ar5iv** (`~/git/ar5iv`) and **cortex**
   (`~/git/cortex`) — bump/vendor the released CSS in both (user, 2026-07-04:
   both should track the latest ar5iv-css whenever a release is available;
   cortex currently serves the glowup RC from `public/css/` — after the
   release, refresh those files from the released build, or point the
   preview template back at the released CDN tag).
3. **PR `ar5iv-2606-prep` → `main`** — ✅ DONE 2026-07-05: merged as **#273**
   (`8d9189f7e4`, squash) — parity fixes, perf audit + pin! sweep, fatal-mining
   fixes, docs consolidation. **Tagged + released** as `0.7.2` (`bdda7d4a33`),
   and **cortex** now runs a `cortex_worker` rebuilt from the tagged `main`
   (fleet restarted).
3b. **ar5iv-editor redeploy** — ✅ DONE 2026-07-05: rebuilt against
   latexml-oxide `main` @`9aafba841f` + ar5iv-css v0.9.0, pushed
   `ghcr.io/dginev/ar5iv-editor/{ar5iv-editor,ar5iv-validator}:20260705-9aafba841f`,
   cut over on `latexml.rs` (`/opt/ar5iv-editor/deploy`, `.env` repin + compose
   pull/up); public `https://latexml.rs/api/version` reports `9aafba841f`, all
   services healthy. Procedure + the `JAVA_HOME`=Java-21 vnu.jar gotcha captured
   in memory `ar5iv-editor-deploy-latexml-rs`.
   Mechanics (retained for reference): the editor path-deps on the sibling checkout and
   `deploy/Dockerfile` COPYs `~/git/latexml-oxide` into the build context —
   put the checkout on the tagged main, run `deploy/build-and-push.sh` +
   `deploy/release.sh`, and verify `/api/version` reports the tagged sha
   ("powered by latexml-oxide @<sha>").
   **CSS vendoring gotcha:** the editor EMBEDS ar5iv-css
   (`include_bytes!` of `frontend/public/css/ar5iv{,-fonts}.css`, plus the
   VS Code extension's `build:assets` copies the same files) and currently
   holds a PRE-glowup single-file copy. Glowup's `css/ar5iv.css` is modular
   (`@import "./ar5iv/*.css"`), so a raw copy silently drops the imports —
   re-vendor from the BUNDLED release build (`dist/ar5iv.min.css` /
   `dist/ar5iv-fonts.min.css`, lightningcss inlines the imports) and rebuild
   both the server crate and the extension.
4. ~~`f(x)` apply-vs-multiply dedicated session~~ — **CANCELLED 2026-07-02**:
   built, verified vs Perl, then reverted on user review; divergence #18
   (f(x) → function application) re-affirmed and stands. No math-output
   change ships in the July-5 binary from this item.
5. **After the current full-arXiv run finishes (~2026-07-04)**: rebuild
   `target/maxperf-cortex/cortex_worker` from merged `main` (fleet binary was
   deliberately NOT swapped mid-run). — ✅ DONE (folded into item 3's fleet
   rebuild from tagged `main`).
6. **Smoke canvas** on the new binary (a few hundred mixed papers via
   `tools/benchmark_canvas.sh`; verify fatal classes vs the known list, spot
   HTML with the new CSS).
7. **Corpus/service setup** for the July-5 (2606) run; verify the harness
   watchdog + memory-governor settings match `CORTEX_WORKER_HARNESS.md`.
8. Post-run: idle standing-corpus perf re-baseline (PERFORMANCE.md audit-log
   follow-up) — still OPEN — then ~~tag 0.7.0~~ **✅ tagged `0.7.2`** 2026-07-05
   (the release was cut now for the ar5iv 2606 first-public-use run rather than
   post-run; `0.7.0` rolled forward into `0.7.2`).

### Large arXiv corpus troubleshooting (2026-06-30, user-requested) — IN PROGRESS
**User directive 2026-06-30:** after the 2605 (10k/sandbox) troubleshooting, also troubleshoot
the **full arXiv corpus** at
<https://corpora.latexml.rs/corpus/arXiv/oxidized_tex_to_html>. **First pass done 2026-07-02**
(see the session entry above): live-run fatal mining at ~32% corpus produced 4 landed fixes
(2 panic sites, `\dabar@`, plain-`\+`) + PARITY verdicts for `\tikzcdmatrixname`/tikz-cd.
**Remaining threads for the next pass** (fresh fatals accrue as the run completes, ~2026-07-04;
fleet binary intentionally NOT swapped mid-run — rebuild only for the July-5 run):
- the residual `\lx@begin@alignment`/group-leak TooManyErrors family (516 papers; `\+` covered
  one driver, scalebox `\Gscale@@box` (~129, 2605 numbers) still open, others unidentified);
- the generic `_`/`^` math-mode cascade families (1.7k/1.4k papers — need sub-clustering by
  first-error);
- `never_completed_with_retries` (1,069) — sample for OOM/hang/crash witnesses
  (STABILITY_WITNESSES overlap);
- plain-layer leakage decision (55-name audit in the 2026-07-02 session entry): retract
  remaining tabbing entry points vs keep (user call pending).
Method: DB signature-clustering + `cortex_worker --standalone` (exact fleet binary) +
same-host Perl verbose; the canvas-triage skill encodes the rules.

### TokenLimit `tblr` colspec binding — ✅ DONE 2026-06-30 (`226d3bfa51`)
The cleanest fixable thread from the TokenLimit root-cause: `\tblr` now parses its inner spec,
extracts `colspec`, and translates the column mini-language to a classic `\tabular` template
(see the 2026-06-30 "Landed this session" TokenLimit note). **Remaining tabularray follow-ups
(not done):** the `colspec` translation drops X-column stretch (maps `X→l`) and ignores the
non-`colspec` keys (cell/row coloring, spans via `\SetCell`, `hlines`/`vlines` are no-ops) —
those are fidelity polish, not the alignment-leak/runaway bug (which is fixed). The babel-`.ini`
and expl3 TokenLimit hot loops (witnesses 2605.29738 / 2605.05840) remain deep open efforts.

### mhchem-manual fidelity mission (2026-06-27, on `followups-2026-06-27`) — LANDED
Driven by a manual review of `~/Downloads/mhchem.tex` (the mhchem package manual)
rendered with `--preload=ar5iv.sty --css=ar5iv.css --nodefaultresources
--path=~/git/ar5iv-css/css` (glowup branch), examined via playwright + Chrome.

1. **7 new `latexml_contrib` package bindings** for the manual's missing packages
   (errors 10→0): `fancyvrb-ex`, `rsphrase`, `hpstatement`, `tgpagella`,
   `sourcecodepro`, `AlegreyaSans` (raw-load real `.sty` where installed, per the
   user directive that raw-loading `.sty` is encouraged; fonts no-op where absent),
   and `scrreprt` (OmniBus `.cls` stub like `scrbook_cls`, + `\minisec`/`addmargin`/
   `\addtokomafont`). Perl ships no binding for any of these, so they are surpass-Perl
   contrib additions. `pstricks` already bound (its warning is a transitive
   fancyvrb-ex dep-scan artifact when the raw `pstricks.sty` is absent — benign).
2. **`\marginpar` font-leak fix** (`latex_constructs.rs`, `bounded => true`) — the
   manual's `\marginpar{\Large !}` leaked `\Large` document-wide (1388 `144%` nodes →
   4). PARITY bug (Perl 0.8.8 leaks identically); fixed surpass-Perl. OXIDIZED_DESIGN
   #39, KNOWN_PERL_ERRORS #38. Output-neutral (suite 1487/0).
3. **mhchem stub RETIRED → raw-load real `mhchem.sty`.** The engine's expl3/xparse/
   chemgreek support is now mature enough that `\usepackage{mhchem}` raw-loads the
   genuine package: chemistry renders with proper digit subscripts (`\ce{H2O}`→H₂O),
   charge superscripts, reaction arrows (`->`/`<=>`/`->[..]`), bonds, states,
   `\cesplit`. Simple `\ce` is 0 errors + correctly formatted (the old stub rendered
   formulae FLAT). chemformula stub updated to require mhchem with `version=4` (the
   real package warns without it; the old stub was silent). **Residual = SHARED Perl
   limitation, NOT a Rust gap (re-classified 2026-06-27):** the full manual still
   emits ~69 edge-case errors under raw-load (`\ce` inside `align*` →
   `\lx@begin@alignment`/`\end@amsalign`; ~56 `\lx@end@inline@math`). The minimal
   reduction `\begingroup$a$\endgroup` inside `align*` errors **IDENTICALLY in Rust
   AND same-host Perl** — deferred-alignment can't clean the cell `$`-frame across an
   intervening `\begingroup`. Nothing to fix for parity; a fix would be a deliberate
   deep surpass-Perl core divergence (not autonomous work). Basic
   `SideBySideExample`+`\ce` is clean. See memory `mhchem-ce-amsmath-alignment-2026-06-27`.

### 2. Release — ✅ `0.7.2` RELEASED 2026-07-05 (superseded the planned `0.7.0`)
Version bumped, `runtime-bindings` in the artifact, `.deb` deps, CHANGELOG/README
done. **Shipped:** tag **`0.7.2`** on `main` (`bdda7d4a33`, "First public use of
latexml-oxide in ar5iv 2606") → `release.yml` ran the TL-window `dumps` + macOS
arm64 leg + publish (each first-exercised on that tag); **6 assets live** —
Linux + macOS-arm64 tarballs and the `.deb`, each with a `.sha256`. The planned
`0.7.0` was rolled forward into `0.7.2` to fold the July-1–5 parity/perf/stability
fixes.

### 3. Speed: residual XSLT cost on large math books — ✅ FIXED 2026-06-29 (3rd O(n²) found)
After the seclev (`1172569034`) and head-keywords (`da74f6ecfe`) O(n²) XSLT fixes, the
slowest 2605 papers were multi-chapter math books where XSLT still dominated. Profiled
witness **2605.01585** ("From Qubit to Qubit", 2000+ formulae, 512 titles): `xsltproc
--profile` pinned **`maketitle` at 22.7 s of 24.9 s self-time (95 %)** — the inline
`not(//ltx:navigation/ltx:ref[@rel='up'])` full-tree scan, re-run **per title** =
O(titles × tree). Fixed by memoizing the document-global check into the global
`$maketitle_has_up_nav` (`LaTeXML-structure-xhtml.xsl`), same shape as the seclev fix.
**XSLT 24.94 s → 2.15 s (11.6×); maketitle self 22.7 s → 0.004 s; output byte-identical**
(`cmp` clean, 25 MB Core XML). Suite **1502/0** + guard `09_xslt_maketitle_navscan.rs`.
OXIDIZED_DESIGN #41, ARXIV_PERFORMANCE Hotspot #4. The three XSLT O(n²) templates on
large arXiv docs (seclev / head-keywords / maketitle) are now all O(n).

---


### amsrefs `\bib` field values reached the XML as dead text (arXiv/html_feedback#6776) — ✅ LANDED 2026-07-25 (branch `fix-6776-2508.17585-refs`)

Rust Error Fix. The **reported** symptom (2508.17585: "references are not
loading", empty `<ul class="ltx_biblist">`) is the *Perl* defect already fixed
here — KNOWN_PERL_ERRORS #49 / OXIDIZED_DESIGN #57, guarded by
`amsrefs_inline_bibliography_is_not_dropped`; same-host Perl still emits
`Warning:expected:bibkeys` + 0 bibitems where Rust emits 34 (pdflatex: 34).
Verifying that surfaced four **genuine Rust-only** divergences *inside* the
entries, all now matching Perl's core XML on all 34:

* **Field values were not live TeX.** Perl `BibTeX.pool.ltxml:134-166`
  (`\bibentry@create`) assembles the entry as TeX *source* and hands it to a
  fresh `Mouth`; `bibtex.rs` built a pre-tokenized stream with `Explode!`
  (catcode-12 OTHER throughout), so `\MR{849427}` reached the XML as literal
  characters (`Review “MR–849427˝` — the OT1 rendering). Now ports the Mouth,
  with `\csname …\endcsname` for the `@`-bearing handler names as Perl does.
  Lazy tokenization is also what lets the handlers' `Verbatim`/`Semiverbatim`
  params set catcodes first, so `%`/`#`/`~` in a `url` value stay intact — a
  plain `Tokenize!` swap would have regressed that.
* **`pages` rendered empty.** `\bib@@pages` stored `Stored::Tokens`, but
  `prop_digested!` renders only `Digested`/`VecDigested` and fell through its
  catch-all to nothing. Perl L674 is `Digest(Tokenize($pages))` — now digested.
* **`id` was aliased onto `xml:id`.** `LaTeXML-bib.rnc:335,353` declare a real
  `attribute id` on `ltx:bib-identifier`/`ltx:bib-review`; the blanket alias
  turned the ISSN into an invalid-NCName `xml:id="0010-3640,1097-0312"`.
  `document.rs::set_attribute` now normalises the key once (the port spells
  `xml:id` as bare `id` in ~15 internal call sites — math parser, `base_xmath`,
  alignment rows — so the alias stays, but yields where the model declares a
  real `id`), then follows Perl `Core/Document.pm:1370-1386` verbatim. The
  serializer's companion name-keyed fixup is gone: it emits the attribute's
  resolved qname, ordering unchanged (local name, `xml:id` last).
* **`<bib-review>` children were flattened.** Perl `MakeBibliography.pm:655-667`
  `do_links` clones `$node->childNodes` in every branch; Rust used
  `get_content()`, dropping the nested `<ltx:ref>` — the MathSciNet link
  vanished. Now clones (27 live links on the witness).

Also `amsrefs_sty.rs` uses `untex()` (Perl `UnTeX($v,1)`) for the keyval slot:
the tokenizer eats a control word's terminating space, so `to_string()` wrote
`661\ndash693` into the `<bib-data>` BibTeX dump. Guards: `amsrefs_basic`
(structure pair, extended with the pages/ISSN/url/review band) +
`amsrefs_inline_bibliography_is_not_dropped` (post layer). Two adjacent gaps this
surfaced were closed in the follow-up below.

### `BibEntry::fields` is dead, so MR/Zbl synthesis never fired — ✅ LANDED 2026-07-25 (branch `fix-bibtex-mrnumber-synthesis`)

Rust Error Fix; the closeout of the entry above. `current_entry_field`
(Perl `currentBibEntryField`) read `BibEntry::fields`, but `add_field` is called
**only** by `copy_crossref_fields` — outside a `crossref` that store is EMPTY, so
every lookup returned `None` for fields that plainly exist. Two observable
symptoms, one root:

* `\bib@synthesize@mr` / `@zbl` (Perl L803-845) could never fire — a `mrnumber`
  produced no `<ltx:bib-identifier scheme="mr">`, `mrreviewer` no
  `<ltx:bib-review>`, `zblno` no ZentralBlatt link. Perl emits all three.
* `\bib@field@default@year`'s "is `date` already set?" guard never fired, so an
  entry carrying **both** emitted a duplicate `<ltx:bib-date>` (Perl emits one).

Fix: `current_entry_field` falls back to the raw field (Perl's `getField`
returns the field's STRING, and `Pre::BibTeX::Entry` is built with both lists
populated — for amsrefs, `new(…, [@fields], [@fields])` passes the same list
twice). `\bib@field@unknownasdata` now reads the raw field FIRST, since it
reproduces source text verbatim and a Tokens round-trip eats a control word's
terminating space. Also: `pretty_print` appends the trailing newline Perl's
`Pre/BibTeX/Entry.pm:64` writes (`. "}\n"`) — the `<ltx:bib-data>` dump is now
byte-identical to Perl, and the unwired `tests/daemon/formats/makebib.xml`
reference fixture (copied from Perl) already had that shape. Guard: `55_bibtex.rs`
`bibtex_mode_emits_bibentries`, extended with all four synthesis shapes
(bare `mrnumber`; `mrnumber`+`mrreviewer`; `MR1380882 (96e:83024)` → id stripped,
review implied; `zblno`) plus the one-`bib-date` assertion, all verified against
same-host Perl.


### R3 — `latexmlmath_oxide` emptied a single-structure formula — ✅ LANDED 2026-07-25

`into_xmath` returns the parsed tree without attaching it; the main parser path pairs it
with `append_tree`, `bin/latexmlmath_oxide.rs` did not, so a body consisting of ONE
top-level structure (`\frac{1}{2}`, `\sqrt{2}`) serialized as an empty `<mrow/>`.
Multi-part bodies survived only because their parts are pre-existing lexeme nodes still
in the tree — which is why an UNCONDITIONAL append is wrong (it renders `\frac{a}{b}+c`
twice); the append is gated on the tree being detached. Both witnesses now byte-identical
to Perl `latexmlmath --pmml`. Guard `005_latexmlmath_single_structure`, verified red.
