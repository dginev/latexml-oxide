# Design review — Suggestion 1 (HANDOFF_2026-09-03 §2): ragged-row tolerance
# in core alignment vs per-package padding.  VERDICT: REJECT the core move.

## The faithful rule (tex.web §792-800 / latex.ltx / Perl parity)
- tex.web §792 `fin_col`: when the preamble is exhausted and the row continues,
  TeX re-instantiates `cur_loop` — the PERIODIC part after `&&` in the \halign
  preamble — IF one exists; else it raises
  "! Extra alignment tab has been changed to \cr". So TeX's ONLY "repeat/absorb"
  behaviour is the `&&` periodic part. A non-periodic preamble ALWAYS errors.
- LaTeX `array`/`tabular` (\@mkpream) never builds a periodic part → a plain
  {ccc} over-long row is the SAME genuine error. pdflatex prints it.
- Perl LaTeXML is byte-faithful: Alignment.pm:136-144 `nextColumn` calls
  Template.pm:142-151 `column`, which auto-extends the periodic (`repeated`)
  part when `$$self{repeating}`; only when NO column exists does it Error
  'Extra alignment tab' and add a fallback center column.

## The kernel ALREADY implements tex.web §792 periodic repetition — nothing to add
- `latexml_engine/src/tex_tables.rs:1544-1554` parse_halign_template: a leading
  `&` (i.e. `&&`) sets `repeated=true` and splits nonreps/repeated; :1603-1616
  builds `Template{columns:nonreps, repeated:cols}`.
- `latexml_core/src/alignment/template.rs:393-410` Template::get_column_mut:
  when `n > columns.len() && repeating`, clones `repeated[(i-non_repeating)%m]`
  — the faithful `cur_loop` re-instantiation.
- `latexml_core/src/alignment.rs:266-295` Alignment::next_column: errors
  "Extra alignment tab '&'" (:287) ONLY when the column is absent (no periodic
  part) — exact mirror of Perl nextColumn.
- `Template::set_repeating` (template.rs:267) is DEAD (unused in Rust AND Perl):
  the periodic flag is set at construction via `repeated:` (matrix_template
  alignment.rs:1085, base_xmath.rs:1020-1027, tex_tables.rs:1606). Not a gap.

Empirical proof (repros here, b54y = HEAD e8e2ce315d):
- halign_periodic.tex  (`\halign{#\tabskip1pt&&#\cr a&b&c&d\cr}`): pdflatex 0,
  b54y 0.  Legal TeX; kernel repeats the periodic part. GREEN.
- halign_nonperiodic_control.tex (`\halign{#&#\cr a&b&c\cr}`): pdflatex 1
  "Extra alignment tab has been changed to \cr", b54y 1 "Extra alignment tab
  '&'".  FAITHFUL. This error MUST stay — moving tolerance to core deletes it.

## Why Suggestion 1 (absorb-extra-& in latexml_core::stomach) is WRONG
1. It would silence the genuine §792 error for raw \halign / plain tabular /
   tabularx / arydshln — ALL verified to error in pdflatex (see below) — i.e.
   surpass in the wrong direction, and violate fail-toward-flagging.
2. The corpus "Extra alignment tab" symptom has THREE heterogeneous roots, only
   one of which is genuine raggedness. A blanket core absorb masks the other two
   (column-undercount binding bugs; label-column miscounts) as silently-narrow
   tables instead of loud errors. Classification below.

## Corpus classification — 13 docs / 302 "Extra alignment tab" lines
(grep /home/deyan/data/perfect_kernel_s36/*/*/*.log; pdflatex = oracle)

A. SELF-PARSING PACKAGES THAT GENUINELY TOLERATE (pdflatex CLEAN) — RUST-ONLY,
   fix in the BINDING, never core:
   - tabularray tblr (already handled by tabularray_sty.rs:404 `*{16}{c}` pad;
     tblr_ragged.tex: pdflatex 0, b54y 0). colspec undercount also possible.
   - tabvar/demo (80): raw-loaded; `\begin{array}{|C|CCCCCCCCC|}` (10 cols, rows
     FIT — pdflatex clean). b54y over-errors from a COLUMN-DESYNC in the raw
     variation macros (`\niveau{a}{b}`, `\dbarre`, `\decroit`), NOT raggedness.
     Distinct root, own investigation. (A single-row cut errors differently —
     `\noalign cannot be used here` — confirming it is not a ragged-tab issue.)
   - numerica/numerica (56) + numerica-tables (18): raw `\eval[...]{a & b & c}`
     self-parses a `&`-list; numerica_eval.tex pdflatex 0. Rust `\eval` path
     under-provisions the template. Distinct root (raw-exec / binding).

B. GENUINE §792 ERROR — pdflatex ALSO reports; b54y is FAITHFUL, leave as-is:
   - metre/demo (11): raw `\halign` preamble (metre.sty MetricalScheme, 3 cols,
     NO `&&`). Over-long row = real error.
   - arydshln-man (35): arydshln_ragged.tex pdflatex 1 "Extra alignment tab"
     (`:` is a dashed BORDER, not a column; standard array counting). If the
     manual's examples are genuinely over-long these are faithful; if b54y errors
     where pdflatex is clean it is a `:`/`;`-border parsing undercount (verify the
     whole-doc pdflatex before treating as RUST-ONLY).
   - floatrow-rus (21): tabularx; tabularx_ragged.tex pdflatex 2 "Extra alignment
     tab". tabularx does NOT tolerate. Same verify-the-doc caveat.

C. NOT RAGGED TOLERANCE — pdflatex Fatal on true raggedness → separate bug:
   - nicematrix (25) + nicematrix-french (9): nicematrix_wide.tex pdflatex FATAL
     "Too much columns. In the row 2, you try to use more columns". nicematrix
     does NOT tolerate ragged rows; the corpus errors are a label-column
     (first-col/last-col / \CodeBefore) MISCOUNT in the nicematrix binding.
   - harmony (6), fancybox-doc (5), LaTeX_RefSheet (1): unclassified; small.

D. PARKED (pTeX/platex family — do not touch):
   - plextdelarray (31), plextcolortbl (4): platex plext primitives.

## Recommendation on the tabularray `*{16}{c}` padding
KEEP the tolerance in the binding (tabularray_sty.rs:404) — core is the wrong
home (see above). Two faithful refinements, in priority order:
  (1) LOW risk, do now: pad with the LAST colspec column type, not a hardcoded
      `c` (tabularray continues the last column's alignment). One-line change to
      the format string in `\lx@tblr@env`.
  (2) MED risk, defer unless a witness needs >colspec+16 cells: replace the
      finite `*{16}{...}` with a genuinely PERIODIC last column, reusing the
      kernel's existing `repeated` machinery (template.rs:393-410) — but the
      LaTeX-preamble reader `read_alignment_template` (alignment.rs:990-1078) has
      no `&&` syntax and never sets `repeating`, so this needs a small kernel
      plumb: a preamble sentinel that marks "the rest repeats". Only then is the
      pad unbounded like tex.web `&&`. Not worth it for current corpus (no witness
      exceeds 16 extra cells).
Do NOT lower/remove the pad and do NOT move it to core.

## Fix sites (reference)
- Kernel periodic mechanism (CORRECT, cite as evidence): tex_tables.rs:1544-1554
  & 1603-1616; template.rs:393-410; alignment.rs:266-295. Perl: Alignment.pm:
  136-144, Template.pm:142-151.
- tabularray pad: latexml_contrib/src/tabularray_sty.rs:404.
- Suggestion 1's proposed core site (REJECT): latexml_core alignment next_column
  / stomach_alignment.

## Dead ends
- Minimal tabvar_var.tex / numerica_eval.tex do NOT reproduce (pdflatex 0, b54y
  0): the 80/56 corpus errors need the full multi-row / option-laden structure,
  not the bare env. Their roots are column-desync, not ragged tolerance.
- set_repeating()/setRepeating are dead in both Rust and Perl — not the missing
  link; periodic is set via `repeated:` at construction.

================================================================================
# ROUND 2 (b54t, 2026-09-03) — CHECKPOINT 1: kernel-alignment residue candidates
================================================================================
Roots assigned from first_errors36.tsv (s36=b54q). Verified against b54t binary.
All three top-3 repros are SHARED (Perl fails identically) AND pdflatex-clean →
surpass-Perl in scope.

## Candidate list (ranked by LIVE docs × error lines; * = top-3 repro)
1. `\the0 You can't use 0 after \the` — physics2, physics2-legacy (202) —
   OUT-OF-TOPIC. lualatex-oracle; error "at Anonymous String" inside unicode-math
   `\if_bool:N \g__um_main_font_defined_bool`. `\if_bool:N`=`\tex_ifodd:D`; the
   bool is not a live \chardef in Rust → "Missing number→0" → `\the0`. expl3 /
   unicode-math bootstrap, NOT alignment. Hand to expl3 / luatex-profile.
2. `\TikzEveryCell` — nicematrix-french (400) — ALREADY FIXED in b54t. Minimal AND
   doc-exact (`[corners]`+`\Block`+`\CodeAfter \TikzEveryCell{offset=1pt,draw}`)
   both convert CLEAN with a real <ltx:tabular>. Post-54q nicematrix `\CodeAfter`
   batch resolved it. (Re-verify the full 8342-line doc before closing.)
3.*`\xmathstrut` — numerica (57) — BINDING GAP (mathtools). RED, SHARED (Perl also
   `\xmathstrut undefined`; LaTeXML mathtools.sty.ltxml omits it too), pdflatex 0.
4.*`\@end@tabular / \@@tabular … internal_vertical` — pfdicons-doc (8),
   tikzcodeblocks-documentation (14), +shipunov boldline variant (4) — KERNEL
   mode-frame. RED, SHARED (Perl 2 identical), pdflatex 0.
5.*`\noalign cannot be used here` — harmony (9) — KERNEL longtable. RED, SHARED
   (Perl 5+ identical), pdflatex 0. (tex-font-cheatsheet 24 = related Verbatim-in-
   cell variant, not yet reduced; objectz/polynom/aguplus have OTHER first errors.)
6. `\lx@end@inline@math` — kblocks-doc (97): `\bm{}` (bm.sty) leaves a boxing group
   open inside `$…$`; boxes-groups, not alignment. titlecaps (15): `$\SaveHardspace`
   is an \ifx-only delimiter sentinel (titlecaps.sty:424,446) that must never be
   typeset; string-mouth. Minimal `\titlecap{a b c}` does NOT repro → needs a title
   with the active `~`/special path. Neither is kernel-alignment.
7. `\CT@everycr` — srdp-mathematik (10): tabu raw-load needs colortbl internals
   (`\CT@everycr`=`\let`→`\everycr`, colortbl.sty:116; also `\NC@list`,`\NC@do`,
   `\tabu@rewritefirst`). colortbl_sty.rs binding shadows colortbl.sty so these raw
   internals are undefined. tabu-specific, complex; defer.
8. `\org@halign … mode math` — tablists-rus (101): OWNED by alignment-ledger topic
   (tablists_arraycr_*). Not re-worked here.

## TOP-3 REPROS (all RED in b54t, pdflatex-clean, SHARED)

### #1  pcol_lstlisting_endtabular.tex  (KERNEL mode-frame; pfdicons+tikzcodeblocks, ~22, 2 docs)
- Root: a `p{}` paragraph column opens a mode-switch-to-internal_vertical frame
  (`\lx@tabular@p@`); a listings env/command (`lstlisting` / `\lstinputlisting`),
  which does `\begingroup` + catcode change + its own mode, inside that cell pops
  the wrong frame — leaves the p-column's internal_vertical frame open. At row/table
  end `\endgroup`→"close non-boxing group", then `\@@tabular`/`\@@longtable`→
  "Attempt to end mode restricted_horizontal". Same family: shipunov boldline
  `\@@tabular … restricted_horizontal`.
- Classification: SHARED (Perl 2 identical errors), pdflatex 0. Surpass in scope.
- Diverging Rust site: latexml_core/src/stomach.rs mode-close (733/826/1049); the
  listings env's group/mode open+close interleaving with the p-column paragraph
  frame. Fix is kernel: the p-column paragraph frame must be closed at the cell
  boundary (`&`/`\\`, LaTeX array.sty `\@endpbox`/`\@finalstrut`) BEFORE the row
  ender, and a verbatim env's `\endgroup` must not target it.

### #2  xmathstrut_mathtools.tex  (BINDING gap; numerica, 57, 1 doc)
- Root: mathtools.sty:1897 `\def\xmathstrut{\@dblarg\xmathstrut@}`;
  `\xmathstrut@[#1]#2 = \vphantom{\mathpalette\xmathstrut@do{#2}}` (mathtools.sty:
  1898-1907). Neither the Rust mathtools_sty.rs binding nor LaTeXML's
  mathtools.sty.ltxml defines it → undefined; in numerica the frac/`\eval` array
  then miscounts → downstream "Extra alignment tab".
- Classification: SHARED (Perl also `\xmathstrut undefined`), pdflatex 0.
- Fix site: latexml_package/src/package/mathtools_sty.rs — add `\xmathstrut`
  faithfully (optional depth arg via `\@dblarg`; a `\vphantom` of the braced body).
  Simplest faithful: DefMacro `\xmathstrut` → `\mathstrut`-style vphantom of `#2`
  scaled; but minimal = grab `[#1]{#2}` and emit `\vphantom{#2}` (structure-neutral).

### #3  noalign_longtable_newpage.tex  (+ noalign_tabular_newpage_CONTROL.tex)
- Root: a longtable is NOT a box — its rows sit in outer vertical mode, so
  `\newpage` (vertical material) between rows is legal and the FOLLOWING `\hline`'s
  `\noalign` is still at a valid alignment boundary (tex.web §768-812; longtable's
  page-break `\cr` splicing). Rust treats longtable like a boxed `\halign`; after
  `\newpage` it loses the "at row boundary / noalign-legal" state → `\noalign
  cannot be used here` → row desync → `Extra alignment tab` cascade.
- CONTROL: plain `tabular` + `\newpage` between rows = pdflatex 9 errors (tabular is
  a box). The longtable fix must NOT be a blanket "tolerate \noalign everywhere".
- Classification: SHARED (Perl 5+ identical), pdflatex 0 (longtable) / 9 (tabular).
- Fix site: kernel longtable/alignment row-boundary state (latexml_engine/src/
  tex_tables.rs:224 \noalign guard + longtable.sty page-break handling): after a
  between-row vertical command in a longtable, keep the alignment at noalign-legal
  boundary state.

## Dead ends (round 2)
- `\TikzEveryCell` no longer reproduces (fixed post-54q) — do not re-open.
- Minimal `\titlecap{a b c}` clean — titlecaps needs the active-`~`/special path.
- physics2 `\the0` is unicode-math/expl3 under lualatex, not a table root.

================================================================================
# ROUND 2 — CHECKPOINT N: ROOT #1  p{}-column + block-listings frame collision
================================================================================
Repros: pcol_lstlisting_endtabular.tex (RED, rust=2), pcol_lstlisting_minipage.tex
(RED, rust=7), + CONTROLs pcol_lstlisting_parbox_CONTROL (pdflatex ALSO 7 err),
pcol_lstinline_clean_CONTROL (0/0), pcol_minipage_clean_CONTROL (0/0).
Witnesses: pfdicons/pfdicons-doc:996, tikzcodeblocks-documentation:679/735.

## CLASSIFICATION: SHARED (Perl fails identically: 2 errors, same
   `\endgroup close non-boxing group` + `\@@tabular end mode restricted_horizontal`
   signature). pdflatex 0 → surpass-Perl in scope.

## MECHANISM (proven by tracer LXML_TRACE_BOUND_MODE + LXML_TRACE_ALIGN_STATE)
- The tex.web align_state ledger is BALANCED end-to-end (ends at 0). This is NOT a
  ledger-count drift; it is a mode/group FRAME-stack imbalance.
- p{} column: DefColumnType "p" (tex_tables.rs:123) → `\lx@tabular@p@ VBoxContents`
  (tex_tables.rs:154) → the cell body is digested by the ONE-FRAME box loop
  `predigest_box_contents_in_mode` (base_utilities.rs:3575): `begin_mode(internal_vertical)`
  pushes frame P, records `level = get_frame_depth()`, then reads+invokes tokens until a
  `T_END` at `level >= get_frame_depth()` (the box's own `}`), then `end_mode` closes P.
- block `\begin{lstlisting}` (listings_sty.rs:2287): at EXPANSION time (inside the box
  loop's read_x_token) the closure calls `bgroup()` = push_stack_frame(false) [G1 — Perl
  listings.sty.ltxml:117 does the same `$stomach->bgroup`], reads the raw lines (correctly
  consuming `\end{lstlisting}`; `\end` never re-runs — verified via LXML_TRACE_ARGS='\end'
  = only reads {tabular},{document}), and emits `{ \@@listings@block{c}{body}{name} } trailer`.
  `\@@listings@block` is a DefConstructor with `mode => internal_vertical` (listings_sty.rs:2509):
  its before_digest bgroup+begin_mode pushes a SECOND internal_vertical frame M on top of the
  p-column's P; its body digests `\@listingGroup` (a further text frame).
- Trace shows M (internal_vertical) and the text frame STILL OPEN when the row/table ender
  arrives: the block's nested frames leave get_frame_depth() elevated above `level`, so the
  cell-closing `}` (inserted by the alignment at `&`/`\\`) is mis-consumed as a group-close
  instead of breaking the box loop; the p-column frame P is never closed. `\endgroup` then
  finds the `\@@tabular` boxing group (→ "close non-boxing group") and `\@@tabular`'s
  end-mode meets the leftover internal_vertical P (→ "end mode restricted_horizontal").
- Confirmed by discriminators: block lstlisting in TEXT (0/0), in an l-column (0/0), a
  nested minipage alone in a p-cell (0/0), and `\lstinline` inline in a p-cell (0/0) are ALL
  clean. The break needs BOTH (a) an enclosing internal_vertical paragraph box read by the
  one-frame loop (p-cell, or minipage-in-a-table) AND (b) the block-listings group+mode.

## WHAT REAL LaTeX DOES (single save-level each, LIFO)
- array.sty p-column = `\vtop \@startpbox{wd} … \@endpbox`; `\@startpbox` (array.sty:189)
  = `\bgroup …`, `\@endpbox` (array.sty:197) = `… \par … \egroup\hfil`. ONE box group.
- Real listings block = the LaTeX env group only: `\begin{lstlisting}`→`\begingroup`
  (kernel `\begin`) →`\lst@Init`; `\end{lstlisting}`→`\lst@DeInit`→`\endgroup`. `\lst@Init`/
  `\lst@DeInit` (listings.sty) typeset the lines IN the enclosing box's own vertical list —
  they do NOT open a second internal-vertical box. ONE group, LIFO inside the p-box
  (tex.web §1085 unset_box / §1100 handle_right_brace pop exactly the levels opened within).

## FIX (binding — BINDINGS OUTRANK RAW; trailer position is a DEAD END, see below)
File: latexml_package/src/package/listings_sty.rs.
The defect is the block's REDUNDANT nested internal_vertical frame (`\@@listings@block`
mode-switch M) plus the expansion-time `bgroup()` G1, which don't LIFO-close inside an
enclosing internal_vertical box read by the one-frame loop. Faithful to real listings
(single group; lines typeset in the enclosing box's vertical list), make the block NOT
open a second internal_vertical mode-switch when the current bound mode is already
`internal_vertical` (join the surrounding vertical list). Candidate implementations:
  (1) `\@@listings@block`: gate the `mode => internal_vertical` so it switches only when the
      bound mode is not already internal_vertical (a conditional-mode constructor), or
  (2) emit the block-group open as an in-stream `\bgroup` TOKEN (processed in-order by the
      box loop at digestion time) instead of the mid-expansion state `bgroup()` push, so G1
      nests with the box's `level` accounting.
Guard (perfect_kernel batch / cluster_package_guards):
  pcol_lstlisting_endtabular.tex → 0 errors AND
  `//ltx:tabular//ltx:td//ltx:listing[contains(@data,'')]` present (listing inside the cell);
  KEEP pcol_lstlisting_parbox_CONTROL erroring (pdflatex also errors) and
  pcol_lstinline_clean_CONTROL / pcol_minipage_clean_CONTROL at 0 errors.
Risk: MED (mode-frame family, theme 1). Expected gain: pfdicons-doc (8) +
tikzcodeblocks-documentation (14) + shipunov boldline variant (4) ≈ 26 err / 3 docs, plus
any doc with a listing in a longtable/array p-cell.

## DEAD END (confirmed by coordinator + by structure)
- Moving the trailer INSIDE the `{…}` wrapper (to Perl's listings.sty.ltxml:205 position)
  does NOT fix this (repro still 7 err) AND regresses cnltx_en to ~1002 err (mdframed/
  minipage mode-switch inside a listing must meet its OWN frame, listings_sty.rs:1968-1975).
  The p-column bug is orthogonal to trailer position: it is the `\@@listings@block` nested
  mode-switch, not where the balancing T_END sits.
- Tolerating a leftover internal_vertical frame in the p-column loop (base_utilities.rs:3575)
  would be a symptom patch on the tex.web-faithful one-frame loop — reject; fix the binding.

================================================================================
# ROUND 2 — CHECKPOINT N: ROOT #3  longtable \newpage/\nopagebreak between rows
================================================================================
Repros: noalign_longtable_newpage.tex (RED, rust=3), noalign_longtable_nopagebreak.tex
(RED, rust=3); CONTROLs noalign_longtable_clearpage_CONTROL.tex (pdflatex ALSO 2 err),
noalign_tabular_newpage_CONTROL.tex (plain tabular, pdflatex 9 err). Witness harmony/harmony.

## CLASSIFICATION: SHARED (Perl fails identically, 5+ errors: same "\noalign cannot be
   used here" + "Extra alignment tab" + "\@@longtable end mode restricted_horizontal").
   pdflatex 0 (longtable) -> surpass-Perl in scope.

## MECHANISM (tex.web S785 + longtable.sty:135)
- tex.web S785 `align_peek` (background/tex.web:15509-15522): after `\cr`, set align_state
  :=1000000 and peek the next non-blank token. If cur_cmd=no_align -> scan the `\noalign`
  group; if right_brace -> `fin_align`; if `\crcr` -> restart; ELSE `init_row`+`init_col`
  (S786) which `back_input`s the peeked token as the FIRST CELL of a NEW ROW. So a bare
  vertical command (`\newpage`) between rows starts a new row; the FOLLOWING `\hline`'s
  `\noalign` is then misplaced (mid-row) -> a genuine error even in real TeX.
- longtable avoids this by REDEFINING, inside the table body (longtable.sty):
    :135 \def\newpage{\noalign{\break}}
    :136 \def\pagebreak{\noalign{\ifnum`}=0\fi\@testopt{\LT@no@pgbk-}4}}
    :137 \def\nopagebreak{\noalign{\ifnum`}=0\fi\@testopt\LT@no@pgbk4}}
  so each becomes `\noalign{…}` and is consumed via S785's no_align branch, never starting a
  new row. `\clearpage` is NOT redefined -> genuinely errors in a longtable body (CONTROL).
- Rust binding gap: longtable_sty.rs:27-28 (`\@@longtable` before_digest; Perl
  longtable.sty.ltxml:45 `Let('\pagebreak','\@gobble@optional')`) handles ONLY `\pagebreak`.
  `\newpage` and `\nopagebreak` stay the global vertical commands, reach the kernel's
  align_peek path as non-`\noalign` tokens, start a row, and the next `\hline`'s `\noalign`
  is rejected at tex_tables.rs:224 -> row desync -> Extra alignment tab. (`\pagebreak`
  rust=0 already; `\clearpage` rust=3/pdflatex=2 CONTROL.)
- The kernel S785 behaviour is NOT the bug: substituting an explicit `\noalign{\break}` for
  `\newpage` in the repro is CLEAN (0 errors, valid <ltx:table>/<ltx:tabular> a,b,c,d). The
  fix is the longtable binding installing longtable.sty:135/137 — not a kernel align change.

## FIX (binding, faithful to longtable.sty:135-137)
File: latexml_package/src/package/longtable_sty.rs — `\@@longtable` before_digest (line 27,
alongside the existing `\pagebreak` Let), which runs after `bgroup()` so the redefs are
group-local (undone at longtable end), mirroring longtable.sty's group-local `\def`s and
Perl's scoped `Let`. Add:
  - `\newpage` -> `\noalign{\break}`  (longtable.sty:135; VERIFIED clean + valid table)
  - `\nopagebreak` -> gobble its optional and vanish, e.g. Let to `\@gobble@optional`
    (same device the clean `\pagebreak` uses) OR `\noalign{}`  (longtable.sty:137)
  Keep `\pagebreak` -> `\@gobble@optional` (already clean). Do NOT touch `\clearpage`.
Guard (perfect_kernel batch / cluster_package_guards):
  noalign_longtable_newpage.tex -> 0 errors AND `//ltx:table//ltx:tabular//ltx:td` count = 4
  with cells a,b,c,d (two body rows survive). Add noalign_longtable_nopagebreak.tex likewise.
  KEEP noalign_longtable_clearpage_CONTROL and noalign_tabular_newpage_CONTROL erroring
  (pdflatex errors on both).
Risk: LOW (two scoped Lets in the longtable env; no kernel change). Expected gain: harmony (9)
  + any longtable doc using \newpage/\nopagebreak between rows.

## DEAD END
- Tolerating `\noalign` after any vertical command in the kernel (alignment.rs / tex_tables.rs
  :224) would silence the genuine plain-tabular `\newpage` error (noalign_tabular_newpage_CONTROL:
  pdflatex 9) and the longtable `\clearpage` error (pdflatex 2). The tolerance belongs to
  longtable's `\def\newpage{\noalign…}`, not the kernel.

================================================================================
# ROUND 2 — CHECKPOINT N: ROOT (a)  colortbl \CT@* internal surface (\CT@everycr)
================================================================================
Repro: colortbl_ct_everycr.tex (RED, rust=2, perl=2, pdflatex=0). Witness srdp-mathematik:638.

## CLASSIFICATION: SHARED — Perl colortbl.sty.ltxml defines NO \CT@* internals either
   (grep: none), so Perl fails identically (2 errors). pdflatex 0 -> surpass in scope.

## MECHANISM
colortbl_sty.rs stands in for colortbl.sty but defines only \arrayrulecolor (:197) and
\doublerulesepcolor (:199), both no-ops. The colortbl \CT@* INTERNAL surface is absent, so
raw colortbl-derivative code that reaches those internals fails undefined. \everycr is a
Tokens register in the kernel (tex_tables.rs:202 DefRegister). colortbl.sty:116
`\let\CT@everycr\everycr` makes \CT@everycr an alias of that toks register; tabu.sty:720
`\iftabu@colortbl\CT@everycr\expandafter{\expandafter\iftabu@everyrow \the\CT@everycr \fi}\fi`
assigns to it and \the-s it -> needs \CT@everycr to BE a toks register.

## \CT@* SURFACE REACHED BY RAW DERIVATIVES (grep \CT@ under texmf-dist; raw = tabu STUB
## tabu_sty.rs:7, tabulary, tabularht, keyvaltable, ctable)
  tabu.sty:      \CT@arc@ \CT@do@color \CT@drsc@ \CT@end \CT@everycr \CT@LT@sep
  tabulary.sty:  \CT@arc@ \CT@cell@color \CT@color \CT@column@color \CT@do@color \CT@drsc@
                 \CT@extract \CT@row@color \CT@setup \CT@start
  tabularht.sty / keyvaltable.sty: \CT@arc@
(tcolorbox, nicematrix, xcolor, revtex4-1, aastex are BOUND -> not raw reaches.)

## FAITHFUL BINDING DEFS (colortbl.sty file:line -> colortbl_sty.rs) — "internal surface"
## shape like the biblatex fix (define the internals the raw code names, mirroring the .sty)
  \CT@everycr        colortbl.sty:116  \let\CT@everycr\everycr   -> Let!("\\CT@everycr","\\everycr")  [toks register]
  \CT@arc@           :165  \let\CT@arc@\relax                    -> Let \relax   (arrayrule color; unrendered)
  \CT@drsc@          :160  \let\CT@drsc@\relax                   -> Let \relax   (doublerulesep color)
  \CT@do@color       :166  \let\CT@do@color\relax                -> Let \relax
  \CT@@do@color      :78   \def\CT@@do@color{<leaders vrule>}    -> \relax (visual only)
  \CT@column@color   :91   \let\CT@column@color\@empty           -> Let \@empty
  \CT@row@color      :204  \let\CT@row@color\relax               -> Let \relax
  \CT@cell@color     :139  \let\CT@cell@color\relax              -> Let \relax
  \CT@color          :75   \def\CT@color{...\color}              -> \relax (color on rule = no-op in binding)
  \CT@setup          :72   \def\CT@setup{...}                    -> \relax
  \CT@start/\CT@end  :119/:125  \def (save/restore color state)  -> faithful \def OR \relax (state unused when colors no-op)
  \CT@extract{b,d,e,f} :89-112  preamble \columncolor parser     -> faithful \def chain (only if the binding runs colortbl's \@classz; else \relax)
  \CT@LT@sep         (longtable sep)                             -> \relax
Minimum to clear the witness surface: the six tabu reaches + \CT@color/\CT@setup/\CT@start/
\CT@extract/\CT@cell@color/\CT@column@color/\CT@row@color (tabulary). \CT@everycr MUST be a
toks register (Let to \everycr); the rest are safe \relax/\@empty no-ops (colortbl paints
colors on rules/cells, which the binding does not render — it already no-ops the public
\arrayrulecolor/\doublerulesepcolor).

## FIX
File: latexml_package/src/package/colortbl_sty.rs (load_definitions). Add the \CT@* surface
above. Prefer a RawTeX block copying colortbl.sty:72-166's \let/\def bodies verbatim where
they matter, plus `Let!("\\CT@everycr","\\everycr")` for the toks register; keep the color
emitters (\CT@color/\CT@do@color/\CT@arc@/\CT@drsc@) as \relax since rule/cell color is not
rendered. Mirror into Perl's colortbl.sty.ltxml if upstreaming (both lack it = PERL-shared).
Guard (cluster_package_guards / perfect_kernel batch):
  colortbl_ct_everycr.tex -> 0 errors AND the document element is present (//ltx:document with
  text "x"); a tabulary raw table (\CT@color/\CT@setup reach) -> 0 undefined \CT@*.
Risk: LOW (adds only internal \CT@* definitions; no public-command or rendering change).

## SCOPE NOTE (honest gain)
srdp-mathematik is a MULTI-root doc: after \CT@* it still needs the array surface (\NC@list,
\NC@do, \col@sep, \extratabsurround), tabu's \tabu@rewritefirst, and longtable's \LT@bchunk
(its Fatal). So \CT@* alone reduces but does NOT clean srdp. Class-level value is the raw
colortbl-derivative family (tabulary/tabularht/keyvaltable/tabu docs) reaching \CT@*.

## DEAD END
- Defining \CT@everycr as an ordinary macro (\def\CT@everycr{}) not a toks register: tabu:720
  `\the\CT@everycr` then errors "You can't use \CT@everycr after \the" — it MUST be a register.

================================================================================
# ROUND 2 — CHECKPOINT N: ROOT (b)  tablists-rus \org@halign … mode math -> RESOLVED
================================================================================
Re-ran /usr/local/texlive/2025/texmf-dist/doc/latex/tablists/tablists-rus.tex from a
writable cwd with b54t (--preload=[rawstyles,rawclasses]latexml.sty).

RESULT: CLEAN. 0 Error/Fatal (the only stderr `Error:` is the read-only-dir log-write
"Permission denied" artifact, ignored per PREAMBLE), `Conversion complete: 1 warning`
(hypdoc/hyperref hyperindex — cosmetic), 153 KB / 2760-line XML, no <ERROR>/undefined
markers. Structure: 161 <td>, 29 <tr> (tables), 78 <Math>, 75 <verbatim>, 52 <para>.

The s36 first error `\org@halign Attempt to close a group that switched to mode math`
(101 errors) is GONE: the alignment-ledger `tablists_arraycr_backslash`/`_math` fixes
(array-style `\TeXr@arraycr` \\ in a raw \halign leaving an open inline-math frame) plus
batches 54w/54x cleared the whole cascade. No further action; do not re-open.

Guard suggestion: add tablists-rus (or the reduced tablists_arraycr_* repros already in
alignment-ledger, now GREEN) to the doc-level regression set to lock the 101->0 win.

================================================================================
# ROUND 2 — CHECKPOINT N: tabu strategy — RECOMMEND (a) FAITHFUL BINDING, already ~90% done
================================================================================
Binary b54x. Repros: tabu_to_X.tex, longtabu_X.tex (GREEN guards); tabu_everyrow_gap,
tabu_rowfont_gap, tabu_extrarowsep_gap (RED, the only user-surface gaps).

## RECOMMENDATION: option (a) — extend the existing tabu binding (tabu_sty.rs) on
## tabularx/longtable. Option (b) (surface the array/longtable internals) is a DEAD HOLE.

## (a) is ALREADY IMPLEMENTED (b54x tabu_sty.rs, 99 lines) — NOT the old stub:
  \tabu -> \lx@tabu@start reads to/spread + \tabularx{<dim|\linewidth>}     [GREEN]
  \longtabu -> \longtable (to/spread dim read & dropped)                    [GREEN]
  X[coef,align,$,p|m|b] DefColumnType: align l/c/r/j, $ = inline-math cell, p/m/b vattach [GREEN]
  \tabucline[]{} -> \hline ; |[rulespec]| rule (spec ignored, presentational) [GREEN]
  \taburulecolor/\taburowcolors/\tabulinestyle/\newtabulinestyle/\tabuphantomline/
    \savetabu/\usetabu/\preamble -> no-ops ; \tracingtabu/\tabulinesep/\above*/\below* -> registers [GREEN]
  Guards prove it: tabu_to_X -> <ltx:tabular> 4 <ltx:td> (2col x 2row, td1 align=left td2 align=right);
  longtabu_X -> <ltx:table>/<ltx:tabular> 4 td. Real witnesses that \usepackage{tabu}: amnestyreport,
  coloring, europasscv, exam-randomizechoices, ftc-notebook, kotex-oblivoir, sduthesis (7+ docs).
  ONLY 3 user-surface GAPS remain (all documented tabu user cmds; RED, pdflatex 0):
    \everyrow{...}   (tabu header L20) — inter-row rule tokens; presentational -> gobble arg
                     (def_macro_noop("\\everyrow{}")); matches existing \taburulecolor no-op policy.
    \rowfont[pos]{f} (tabu user cmd)   — per-row font; presentational -> \rowfont[]{} gobble.
    \extrarowsep     (tabu.sty:232 \newcommand*\extrarowsep, opt +/_ then =<dimen>) — row spacing;
                     -> a primitive swallowing the (+/_)?=<dimen> assignment (no-op; \extrarowheight
                     already handled by array). Fix all three in tabu_sty.rs.
  Guard: the 5 repros above -> tabu_to_X/longtabu_X 0 errors + 4 <ltx:td>; the 3 gap repros -> 0 errors.
  Risk: LOW (3 presentational no-op/swallow adds). Gain: the ~7 \usepackage{tabu} witnesses.

## (b) is a DEAD HOLE — do NOT surface the internals:
  Raw tabu reaches ~26 array/longtable internals (grep tabu.sty): \@arstrutbox(27) \NC@list(21)
  \NC@do(15) \NC@find(14) \@preamble(10) \extratabsurround(8) \prepnext@tok(6) \d@llarend(5)
  \NC@rewrite@(4) \LT@next(4) \LT@bchunk(4) \d@llarbegin(4) \@classz(4) \@addtopreamble(4)
  \LT@cols(3) \@lastchclass(3) \col@sep(3) \@mkpream \@chnum \LT@startpbox \LT@echunk \save@decl …
  DEFINING the names is NOT enough: tabu DRIVES array.sty's preamble state machine at runtime —
  \tabu@setup (tabu.sty:696) does `\NC@list{\NC@do \tabu@rewritefirst}`, and \tabu@tabu@ (:667)
  appends `\tabu@rewritefirst` which calls \NC@rewrite@ to REWRITE the preamble through
  \@mkpream/\@classz/\@chclass/\@chnum/\@lastchclass/\prepnext@tok. latexml-oxide REPLACES that
  machinery wholesale with a native template reader (read_alignment_template / DefColumnType), so
  raw tabu cannot run to completion. Proof: srdp Fatal `Missing argument Until:\LT@bchunk … File
  ended` — tabu's `\tabu@longpream #1\LT@bchunk #2\LT@bchunk` (tabu.sty:1207) delimited-scans for
  \LT@bchunk which the longtable binding never emits. Perl LaTeXML "raw-loads tabu.sty and dies"
  too (tabu_sty.rs comment). So (b) needs array.sty's \@mkpream/\@classz scanner in the kernel — a
  large, separate effort; PARK it.

## srdp-mathematik WITNESS REFRAMED — NOT a tabu-binding doc:
  srdp-mathematik.sty -> srdp-tables.sty, which is a VENDORED INLINE COPY of tabu.sty's source
  (1630 \tabu@ defs, \ProvidesPackage{srdp-tables}[2021/11/09]; it does NOT \usepackage{tabu}).
  So the tabu binding cannot help srdp; srdp raw-runs tabu code and hits the (b) dead hole
  (\NC@list, \tabu@rewritefirst, \col@sep, \extratabsurround, \LT@bchunk Fatal). srdp is only
  winnable by implementing array.sty's preamble scanner (parked). Do NOT chase srdp under tabu;
  re-file it as "raw vendored-tabu copy (srdp-tables.sty) / array \@mkpream scanner (parked)".

================================================================================
# ROUND 2 — CHECKPOINT N: ROOT (a2)  tabbing \a<accent> -> \@tabbing@<char> undefined
================================================================================
Repros: tabbing_acc_macron.tex (\a=, encguide), tabbing_acc_dieresis.tex (\a", greek). Both
RED (rust=1, pdflatex 0). Witnesses base/encguide (\@tabbing@=), greek-fontenc/test-lgrenc +
textalpha-doc (\@tabbing@" + \@tabbing@<, lualatex).

## CLASSIFICATION: PERL-ORIGIN. Perl latex_constructs.pool.ltxml:3547 `\@tabbing@accent{}` ->
   `\@tabbing@<char>`, and :3572-3573 pre-saves ONLY `\@tabbing@'` and `\@tabbing@\`` with no
   fallback. Perl fails identically (\@tabbing@= undefined, 1 error). pdflatex 0 -> surpass.

## MECHANISM
Inside tabbing, tabbing_bindings() (latex_constructs/mod.rs:2391) rebinds \= \< \> \' \` to
tabbing ops (tabset/untab/nexttab/flushright/hfil). To still get accents, LaTeX uses \a<accent>
where \a = \@tabacckludge (latex.ltx:10007). The Rust/Perl binding models \a as
\@tabbing@accent{x} -> \@tabbing@<x> (sect10.rs:132 / pool:3547), and pre-saves \@tabbing@'
<- \' and \@tabbing@` <- \` (mod.rs:2445-2446) — but NOT =, <, >. So:
  \a=  -> \@tabbing@=  undefined   (encguide macron)
  \a"  -> \@tabbing@"  undefined   (" is not rebound; needs a fallback to \")
  \a<  -> \@tabbing@<  undefined   (greek breathing; < IS rebound -> needs pre-save)
Real \@tabacckludge#1 = \@changed@cmd\csname\string#1\endcsname\relax (latex.ltx:10005):
recovers the ENCODING-level accent \<char> by name, bypassing tabbing's rebinding — which is
why \a= works in real LaTeX even though \= is the tab-set.

## FIX (binding, faithful to \@tabacckludge)
File: latexml_engine/src/latex_constructs/mod.rs `tabbing_bindings()` + sect10.rs `\@tabbing@accent`.
1. In tabbing_bindings(), BEFORE rebinding, pre-save the rebound accent chars:
   let_i(\@tabbing@=, \=), let_i(\@tabbing@<, \<), let_i(\@tabbing@>, \>)  (plus existing ',`).
   (Order: the pre-save let_i must precede the corresponding rebind let_i.)
2. Change \@tabbing@accent{x} (sect10.rs:132) to fall back when \@tabbing@<x> is undefined
   (is_defined_token, dialect.rs:78): emit \<x> (T_CS "\\"+x) = the real accent (\", \., \^, \~,
   \u, \v, \H, \c, \d, \b, \r, \t … — none of which tabbing rebinds). This mirrors
   \@tabacckludge recovering \csname\string#1\endcsname.
   Undefined accents still error (fallback \<x> undefined) — matches pdflatex (boundary).
Mirror into Perl latex_constructs.pool.ltxml:3547/3572 for parity (both share the bug).
Guard (perfect_kernel batch): tabbing_acc_macron.tex + tabbing_acc_dieresis.tex -> 0 errors AND
//ltx:tabular[@class="ltx_tabbing"] present with the accented glyph. Risk: LOW (3 pre-saves +
a defined-check fallback). Gain: encguide (1), test-lgrenc (3), textalpha-doc (3) = 3 docs.

## SIDE NOTE (not this root)
Bare \< in a non-greek tabbing: rust=0 (no-ops \@tabbing@untab) but pdflatex=1 — a separate
OVER-tolerance (\@tabbing@untab/flushright/hfil/pushtabs/poptabs are all no-op stubs, sect10.rs
:125-129; Perl pool:3583-3590 same "NOT handled"). Distinct from the accent root; leave.

================================================================================
# ROUND 2 — CHECKPOINT N: ROOT (b)  Error:expected:Until at true EOF (DIAGNOSIS)
================================================================================
Docs: l3kernel/l3prefixes (Until:,), chessboard/chessboard_and_beamer (Until:.). (colorspace's
first error is a colorspace latex "Unknown spot color"; Until:\@@ is secondary — not this root.)
Repros: l3prefixes.tex + l3prefixes.csv (real, RED rust=2 b54x); until_eof_control.tex (CONTROL).

## THE Until: READER ITSELF IS CORRECT — not the bug.
base_parameter_types.rs:181 `Until` reader: a ran-out scan at TRUE end-of-all-input
(gullet::at_end_of_all_input) reports "Missing argument Until:X at end of input" + a Fatal
(tex.web S338 "File ended while scanning use of"). CONTROL until_eof_control.tex proves parity:
`\def\grabdot#1.{}\grabdot no dot` -> pdflatex 3 errors ("File ended while scanning use of
\grabdot"), rust 2. So the reader faithfully mirrors the runaway; do NOT soften it. The bug in
each doc is UPSTREAM (the scan should never reach true EOF).

## l3prefixes (Until:,) — SHARED (Perl fails too: 5 errors "Until:\" Missing argument").
Caller: l3prefixes.tex:41 `\__prefix_readii:w #1 , #2 , #3 , #4 \q_stop` (and :35
`\__prefix_readi:w #1 " #2 " #3 \q_stop`), a self-recursive CSV parser driven by
`\ior_map_inline:Nn \l_tmpa_ior { \__prefix_readi:w ##1 " \q_nil " \q_stop }` (:66) over
l3prefixes.csv. NOT a raw-load-path delimiter (`,` is a real char in the file) and NOT the
`\ior` empty-line (a trailing empty line with a `,,,,` sentinel is safe; verified). It is a
CUMULATIVE mouth/file-boundary interaction: a minimal readi/readii over the FULL 342-line csv
reproduces `Until:,` at EOF, but NEITHER half (1-341, 51-342, 1-200, the 8 quote-lines alone)
does — so it needs the full-file `\ior_map_inline` run. The quote-lines (l3prefixes.csv:40,58,
151,155,239,277,296,327 = `"embedded,comma"` fields) take the readi RECURSION branch
`\__prefix_readi:w #1 {#2} #3 \q_stop` (quark_if_nil FALSE); the leading hypothesis is a
per-recursion `{#2}` group / mouth-state drift that only crosses a boundary after the full run,
whereupon the last iteration's `,`-scan runs into true EOF. Root lives in the expl3 layer
(`\ior_map_inline` line-mouth handling and/or the recursive quark-delimited scan), NOT kernel
alignment or the Until reader. -> EXPL3 TOPIC. `\quark_if_nil:nTF {\q_nil}` alone is correct
(ISNIL) — the drift is stateful/cumulative, needs expl3-internal tracing (LXML_TRACE_FRAMES).

## chessboard (Until:.) — xskak RAW-LOAD, needs beamer-overlay context.
xskak/skak/chessboard are NOT bound (raw-load). Caller: xskak move parser (\mainline ->
\ExecuteMoves -> \xskak@do@parsemainline(#1 #2), xskak.sty:1055+), a `.`-delimited chess-move
scan, triggered by `\mainline{2... Nc6}` (chessboard_and_beamer.tex:29) inside beamer
`\only<>` overlays. Minimal `\newchessgame\hidemoves{...}\mainline{2... Nc6}` is CLEAN on b54x
(rust=0), so the runaway needs the fuller beamer-overlay + `\chessboard` context (not reducible
cheaply). -> xskak/beamer interaction, not kernel-alignment; separate root.

## CONTROL
until_eof_control.tex: delimited macro missing its delimiter at true EOF -> pdflatex 3 /
rust 2. Must STAY erroring (it is tex.web S338). Bounds the Until-reader EOF policy.

================================================================================
# ROUND 2 — CHECKPOINT N: ROOT (1)  \noalign+\hrule -> border (batch-54y aftermath)
================================================================================
Repros: noalign_hrule_height_border.tex (Root A, RED-dropped), noalign_bracehack_boldline.tex
(Root B, RED err=1), bracehack_plaingroup_CONTROL.tex (CONTROL, 0/0). Witness shipunov/
boldline-ex-en:17-21 (tabular \hlineB{2.7}). Two DISTINCT roots.

## ROOT A — the rule->border mapping SPEC (a real, small gap; coordinator's ask)
Most \noalign+\hrule ALREADY map to a border: `\hrule` after_digest (tex_box.rs:1346) inside an
alignment sets isHorizontalRule + `add_line("t")` when `dominated_by_width`; the kernel's own
`\hline` = `\noalign{\@@alignment@hline}` (tex_tables.rs:522) does the same via add_line("t")
(:523-531). GAP: `dominated_by_width` (tex_box.rs:1387-1392) =
  (None,None)=>true, (None,Some w)=>w>20, (Some h,Some w)=>w>3h, _=>false
so `(Some h, None)` — explicit HEIGHT + DEFAULT (full) width — falls to `_=>false`: the rule is
NEITHER rendered NOR a border (silently dropped; noalign_hrule_height_border.tex: 0 errors, 0
border, 0 <ltx:rule>). A full-width \hrule with explicit height IS a horizontal rule.
  FAITHFUL RULE (per coordinator): a \noalign whose digested body is only rule/kern/skip boxes
  -> adjacent row's border; thickness -> border class or ignored; else keep the row. Concretely:
  FIX tex_box.rs:1389 add arm `(Some(_h), None) => true` (explicit-height, full/default width =
  horizontal rule). Perl TeX_Box.pool.ltxml:851 sets isHorizontalRule "if dimensions suggest a
  real rule". The row-level border transfer already exists (TeX_Tables.pool.ltxml:501,520
  isHorizontalRule -> saveleft/saveright, stripped from cell "meat"; Rust normalize.rs:104-114
  `isrule` -> cell.empty/skippable). Risk LOW.
  Guard: noalign_hrule_height_border.tex -> 0 errors AND first row border="t", td count = data
  cells only (no rule <td>).

## ROOT B — the boldline \hlineB WITNESS failure (a section-1206 digest_next_body bug)
The latex.ltx brace hack `\noalign{\ifnum0=`}\fi ... \ifnum0=`{\fi}}` (boldline.sty:13-17
\hlineB/\@xhlineB, and any raw rule macro on the hack) DESYNCS the boxing level in the
section-1206 \noalign branch's digest_next_body path (tex_tables.rs:998-1017): the SAME hack is
CLEAN in a plain group AND in \hbox (bracehack_plaingroup_CONTROL: rust 0, pdflatex 0; balanced
frames) but yields `\@end@tabular Attempt to close boxing group` inside \noalign. LXML_TRACE_FRAMES
shows `\@@tabular` pushed twice / popped once, so \@end@tabular meets the env `\begingroup`
(nobox=true) on top. Isolation: `bracehack-only` (no \futurelet, no arg) already fails -> it is
the brace hack, not the trailer. batch 54y fixed the OPENING `\ifnum0=`}` (execute the body per
section-1206 vs the old read_arg pre-scan, guard noalign_body_is_executed_to_its_group_end); the
CLOSING `\ifnum0=`{\fi}` is the residual: read_non_space consumes `\noalign{`'s `{` then
bgroup() pushes the frame (tex_tables.rs:1011), but the char-code `{`/`}` of the closing hack are
not accounted the way the NORMAL group-digest loop does, so digest_next_body's
`init_depth > boxing.len()` termination (stomach.rs:1737) fires off-by-one.
  Classification: SHARED (Perl noalign branch TeX_Tables.pool.ltxml:391 uses `digest(readArg)`,
  whose token-level balanced read the `\ifnum0=`}` `}` also mis-closes; Perl identical pre-54y).
  pdflatex 0 -> surpass. FIX site: the section-1206 \noalign branch group accounting
  (tex_tables.rs:998-1017) — make bgroup()/digest_next_body/egroup mirror the plain-group
  digest so the `\ifnum`-backtick char-code `{`/`}` balance identically in \noalign as in `{...}`.
  Risk MED (mode/box-frame family). Guard: noalign_bracehack_boldline.tex -> 0 errors AND
  <ltx:tabular> with 2 data rows; KEEP bracehack_plaingroup_CONTROL at 0 errors.

## Expected gain: shipunov/boldline-ex-en (4 err) + any doc with a raw rule macro on the hline
brace hack (A+B together). A alone also un-drops explicit-height rules cluster-wide.

================================================================================
# ROUND 2 — CHECKPOINT N: ROOT (2)  nicematrix Extra-alignment-tab = NESTED-BRACKET OPTION
================================================================================
Repro: nm_nested_bracket_option.tex (RED rust=1, pdflatex 0). CONTROL: nicematrix_wide.tex
(genuine over-count, pdflatex 3 "Too much columns"). Witness nicematrix/nicematrix:1364.

## NOT the coordinator's "label-column miscount". first-col/last-col/first-row/last-row and the
## rule-specs |[..]| / || in a colspec are ALL CLEAN in isolation (tested). The doc's 16
## "Extra alignment tab" are a CASCADE from ONE corrupted env.

## MECHANISM (isolated by bisecting nicematrix.tex)
The 16 Extra-tabs (nicematrix.tex 1438/1440, 1502-1505, 2055-2058, 3054) do NOT reproduce with
the doc preamble + the isolated env; they cascade from the env at 1363-1365:
  \begin{NiceTabular}{|ccc|}[rules/color=[gray]{0.9},rules/width=1pt,no-cell-nodes] \hline ...
whose FIRST error is `\noalign cannot be used here` (line 1365), followed by open-itemize
mode-frame errors (1402/1406) that corrupt the document, cascading Extra-tabs into every later
table. Root: nicematrix reads env options with xparse `O{}` (\NewDocumentCommand{\NiceTabular}
{ O{} m !O{} }, nicematrix.sty:3806-3841), which BALANCES nested `[..]`. The binding
\NiceTabular[]{}[] (nicematrix_sty.rs:267) uses the `[]`/Optional param -> read_optional ->
read_optional_delimited (gullet.rs:2419) = read_until the FIRST `]` (NON-nesting; correct for
plain-LaTeX \@ifnextchar[). So `[rules/color=[gray]…]` is cut at the inner `]` after `[gray`;
`{0.9},rules/width=1pt,no-cell-nodes]` spills into the alignment body, and the following \hline
(= \noalign) lands mid-row. Confirmed minimal: `[rules/color=[gray]{0.9},…]`+\hline = rust 1 /
pdflatex 0; `[rules/color=red]` (non-nested) and the nested option WITHOUT a rule are clean.

## CLASSIFICATION: RUST-ONLY. Perl cannot build nicematrix here (raw-loads nicematrix -> pgf
   `pgfsys-` driver not found, 47 errors, DIFFERENT failure). pdflatex clean -> surpass.

## FIX (binding): read \NiceTabular/\NiceTabular*/\NiceTabularX/\NiceArray/<x>NiceMatrix
   `[options]` with a NESTING-AWARE bracket reader (xparse `O{}` semantics: balance `[`/`]`),
   not the non-nesting `[]`/Optional. Site: nicematrix_sty.rs (the option slots at :267/279/290
   and the matrix `[opts]` at :322) — either a new `OptionalNested` param type (balances brackets,
   matching xparse O{}) reused there, or a bracket-balanced option grab in \lx@nice@setopts@wrap
   (:176-186). Keep read_optional non-nesting (it is correct for plain LaTeX). CONTROL
   nicematrix_wide.tex must STAY erroring.
   Guard: nm_nested_bracket_option.tex -> 0 errors AND <ltx:tabular> with a 3-cell row;
   nicematrix_wide.tex -> keeps >=1 error.
Risk: LOW-MED (a nesting-aware optional read on the nicematrix env options only). Expected gain:
   nicematrix (16 Extra-tab + itemize cascade) + nicematrix-french (shares the doc) — most of the
   nicematrix Extra-tab cluster from the prior study, since they cascade from this root.

## Dead ends
- first-col/last-col/first-row/last-row extra label columns: CLEAN in isolation — not the root.
- Rule-spec colspec |[color=..]| / c||ccccc: CLEAN in isolation — the doc failures are cascades.

================================================================================
# ROUND 2 — CHECKPOINT N: class-level  xparse O{}/o/d[] bracket nesting audit
================================================================================
Repros: xparse_nested_O.tex (xparse O{} NESTS — rust 0), newcommand_nonnest_CONTROL.tex
(plain \newcommand stays non-nesting — rust 0). Binary b54x.

## Q: does the Rust xparse layer read O{}/o/d[] with bracket nesting?  A: YES — no layer fix.
\NewDocumentCommand RAW-LOADS xparse.sty (xparse_sty.rs:14 input_definitions("xparse")), so
O{}/o run the REAL l3 `\__cmd_grab_optional` grabber, which balances [ ]. Verified:
`\NewDocumentCommand\foo{O{}m}` + `\foo[a[b]c]{z}` -> #1 = "a[b]c" (rust 0, pdflatex 0);
`\foo[a=[x]{y},b]{z}` -> #1 = "a=[x]{y},b". So the xparse LAYER is correct; nothing to fix there.

## The Rust-native mirror for HAND-ROLLED bindings: OptionalBalanced (base_parameter_types.rs:466)
A declared, nesting-aware `[...]` param (balances [ ] and { }); its doc comment already cites
nicematrix.tex:1364; guard perfect_kernel_batch54::optional_balanced_nests_brackets. It is the
vehicle for bindings that build option macros with DefMacro!/DefPrimitive! `[]` (the non-nesting
`Optional` = read_optional, gullet.rs:2419) rather than \NewDocumentCommand, when the REAL
package reads that option via xparse O{}.

## nicematrix: ALREADY FIXED IN THE TREE (b54x predates it). nicematrix_sty.rs now spells the
env/matrix option slots `OptionalBalanced`: \NiceTabular (:267), \NiceTabular* (:279),
\NiceTabularX (:290), and \{,p,b,B,v,V}NiceMatrix (:583-604). So nm_nested_bracket_option.tex
(RED on b54x) will go green on b54z. No further nicematrix action.

## CONTROL: plain \newcommand\bar[1][] MUST stay non-nesting (read_optional -> first ]),
matching pdflatex's \@ifnextchar[. newcommand_nonnest_CONTROL.tex: `\bar[a]b` -> #1="a" (rust 0).
Do NOT make read_optional nesting-aware — only xparse O{} and OptionalBalanced nest.

## AUDIT — other bindings that HAND-ROLL `[]` for options a real (xparse) package reads via O{}:
  - tabularray_sty.rs: `\lx@tblr@env{} []{}` (:380), `\SetTblrInner []{}` (:418), `\SetCell[]{}`
    (:435). tabularray is \NewDocumentCommand-based; tblr keys can carry braced sub-key-lists.
  - tcolorbox_sty.rs: `\newtcblisting[]{}[][]{}` (:53), `\NewTCBListing[]{}{}{}` (:151),
    `\DeclareTCBListing`/`\RenewTCBListing` (:154/157). tcolorbox/pgfkeys options.
  - cleveref_sty.rs: `\lx@cleverref@label[]` (:9). `[type]` is a bare word — non-nesting is FINE.
  - siunitx_sty.rs: `\num`/`\qty` option lists — key=value, values usually BRACED.
  RISK: the bug only bites when an option value carries an UNBRACED nested `[...]` (xcolor model
  `[gray]{0.9}` / `[HTML]{..}` or a `key=[..]` unbraced) AND the slot uses `[]` not O{}. nicematrix
  is the confirmed case (its docs write `rules/color=[gray]{0.9}` unbraced). tcolorbox/tabularray/
  siunitx usually BRACE such values (`{[HTML]{..}}`), so the outer `[..]` reader sees a balanced
  `{..}` — my bare-nested probes there errored in pdflatex too (invalid syntax), so no live witness
  found. RECOMMENDATION: switch every option slot that MIRRORS an xparse O{} to OptionalBalanced
  (uniform + faithful); priority only where a package ships an unbraced nested `[model]{value}` in
  its docs. cleveref `[type]` and braced-value key-lists can stay `[]` safely.

================================================================================
# ROUND 2 — CHECKPOINT N: alignment-class LOG TALLY (all logs) + top-2 uncovered roots
================================================================================
Tally over /home/deyan/data/perfect_kernel_s36/*/*/*.log (ANSI-stripped, Error/Fatal, unique
per doc), alignment-class messages, by distinct DOC COUNT:
  18  \noalign cannot be used here
  14  \endgroup Attempt to close ... internal_vertical      (54x p-cell listings family; partial)
  14  } Attempt to close ... vertical                        (UNCOVERED — root #2 below)
  13  & Extra alignment tab                                  (nicematrix nested-opt 54z; tabu parked; numerica)
   9  \endgroup ... math      6 \lx@begin@alignment ... internal_vertical
   5  \@@tabular ... restricted_horizontal   5 \@end@tabular close boxing   5 \@end@tabular ... internal_vertical
   5  \@end@tabular ... horizontal   3 \halign ... restricted_horizontal   (+ tikz@pin/@label = NOT alignment)

## TOP UNCOVERED ROOT #1: \omit/\noalign "cannot be used here" via array.sty \@mkpream/\ialign
Docs whose FIRST (root) error is \omit/\noalign cannot: sgame (\omit), tabularcalc_doc_en/fr/vn
(\noalign), tabvar/demo (\noalign), epslatex-fr/fepslatex (\noalign) — ~6 docs. (The other 12
\noalign-anywhere docs are CASCADES from a different first error: csvsimple \csvline undefined,
objectz oz math-version, polynom display_math egroup, topiclongtable \theLT@tables undefined,
storecmd \caption-in-tabularx+colortbl, nicematrix nested-opt = 54z, harmony longtable = 54w,
boldline = 54y — all already covered or non-alignment.)
Mechanism: these packages BUILD their alignment via array.sty's char-class preamble scanner, not
a plain \halign. sgame redefines \@array (sgame.sty:51-89): \@mkpream{#2} then
`\edef\@preamble{\ialign\noexpand\@halignto\bgroup\@arstrut\@preamble\tabskip\z@skip\cr}` then
executes \@preamble (:79-89). LaTeXML does NOT run array.sty \@mkpream/\@classz (it has a native
DefColumnType/read_alignment_template reader), so the \ialign the package assembles is not a
recognized LaTeXML alignment; \omit/\noalign/\cline inside (from \hline, \multicolumn, thick-rule
tricks: sgame.sty:265/322 `\cr\noalign{\vskip-\arrayrulewidth}\cline{...}`) hit the "cannot be
used here" guards (tex_tables.rs:224 noalign, :240 omit). SAME array-preamble-machinery gap that
parked tabu/srdp (Round-2 tabu report). Fix = implement array.sty's \@mkpream char-class scanner
+ \ialign recognition in the kernel (large; PARKED effort), OR bind \@array/\@mkpream to LaTeXML's
native alignment so a package's `\edef\@preamble{\ialign...}` funnels through \lx@begin@alignment.
Repro (RED-ish; needs the package's exact args): \begin{game} matrix; a bare `\ialign{\@mkpream…}`
is the kernel-level shape. Classification: needs verification vs Perl (Perl also lacks \@mkpream
→ likely SHARED). Gain ~6 docs (sgame + tabularcalc×3 + tabvar + epslatex-fr).

## TOP UNCOVERED ROOT #2: `}` Attempt to close a group that switched to mode vertical (14 docs)
Docs: circledtext, codebox, joinbox, pascaltriangle, suanpan-l3 (all \ProvidesExplPackage /
l3draw / l3coffins box builders), biblatex-caspervector×2, pst-exa (pspicture-in-tcolorbox),
sduthesis, shtthesis, thesis-gwu, + CJK bxcjkjatype×2 / kanbun (may be parked). Signature is
uniform: `current frame is mode-switch to vertical due to ` (EMPTY opener) at "Anonymous String"
— a group inside a deep expl3/l3draw box construction (\vbox_set:Nn / \hcoffin_set:Nn / l3draw
\draw path) switches to vertical mode and a `}` meets it. Mode-frame family (theme 1). Minimal
\circledtext{A} / \joinbox... do NOT reproduce (need the l3draw/coffin path + real content), so
not cheaply reducible; needs LXML_TRACE_FRAMES on a full witness (circledtext.tex with the CJK
chars, or pascaltriangle) to pin which expl3 box primitive opens the unbalanced vertical frame.
Classification: RUST-ONLY suspected (expl3 box-primitive mode handling). Distinct from 54x's
internal_vertical p-cell family (that is a p-column box; this is a plain `vertical` mode-switch).

## Dead ends
- csvsimple \noalign is a cascade from \csvline undefined (a register-not-a-macro root, expl3 IO)
  — not an alignment root.
- Minimal \circledtext{A}/\csvautotabular{f.csv} are CLEAN — the failures need the full l3draw/
  filter-option context.

================================================================================
# ROUND 2 — CHECKPOINT N: ROOT #1 root-caused — \@sharp cell placeholder in a raw \ialign
================================================================================
Repros: ialign_sharp_placeholder.tex (RED rust=5, pdflatex 0), noalign_outside_CONTROL.tex
(CONTROL). Witnesses sgame, tabularcalc×3, tabvar/demo, epslatex-fr/fepslatex.

## THE PRINCIPLED ROUTE (not "implement \@mkpream in Rust"): array.sty's \@mkpream ALREADY runs
## raw and, with the \let\@classz\@tabclassz that \array/\@tabular do, builds a REAL template.
## The only gap is the kernel's raw \halign reader not recognizing the \@sharp cell placeholder.

## (1) Does the raw \@mkpream/\@classz chain run, or is it shadowed? — IT RUNS.
sgame.sty:58 `\def\@array[#1]#2{…\@mkpream{#2}\edef\@preamble{\ialign…\@preamble…\cr}…}`
OVERRODE the kernel's bound \@array (a plain DefMacro `\@array@bindings…\lx@begin@alignment`;
`\def` replaces it — bindings did NOT outrank raw here, but that is fine). The game env → sgame
`gtabular`/\@gtabular does `\let\@classz\@tabclassz` (sgame.sty:226) then `\@tabarray`→\@array→
\@mkpream. Probe: \@mkpream/\@testpach/\@classi/\@addtopreamble are the real latex.ltx bodies;
\@classz/\@classiv/\@acol/\insert@column/\@sharp/\prepnext@tok/\d@llarbegin/\d@llarend default to
\relax at top level but are \let to the FUNCTIONAL \@tabclassz/\@arrayclassz/\@tabacol/… by
\array/\@tabular (dump latex.2025:8278/15564).

## (2) What \@preamble does \@mkpream build, and where does the reader stop?
WITH `\let\@classz\@arrayclassz`, `\@mkpream{cc}` builds a REAL preamble (probe):
  `\hskip\arraycolsep\hfil$\relax\@sharp$\hfil\hskip\arraycolsep & …`
i.e. array.sty templates the cell as `\d@llarbegin \@sharp \d@llarend` where \@sharp is the `#`
placeholder (a cs \let to #, array.sty:97/230; \d@llarbegin/end = $/$ or \begingroup/\endgroup).
The kernel's raw \halign parser parse_halign_template (latexml_engine/src/tex_tables.rs:1548,
slot check :1590 `else if cc == Catcode::PARAM`) tests the TOKEN'S OWN catcode. A literal `#`
(catcode PARAM) passes; the cs \@sharp is catcode CS (meaning = #) so it FAILS the test, the slot
is never marked, and the # meaning leaks to the stomach: "# (catcode PARAM) should never reach
Stomach!". PROVEN minimal: `\let\@sharp=# ; \ialign{\hfil\@sharp\hfil&&\hfil\@sharp\hfil\cr a&b\cr}`
-> rust 5 errors, td=4 (structure built, # leaks 5×); the LITERAL-# twin `\ialign{\hfil#\hfil…}`
-> 0 errors.

## (3) Do \omit/\noalign/\cline behave? — YES once the alignment is real.
`\ialign{\hfil#\hfil&&\hfil#\hfil\cr a&b\cr \noalign{\hrule} \omit X&Y\cr}` -> 0 errors, 4 <td>.
CONTROL noalign_outside_CONTROL.tex: `\noalign{\hrule}` OUTSIDE any alignment -> rust 1 / pdflatex
1 (must STAY an error — tex_tables.rs:224 guard).

## FIX (smallest kernel gap)
File: latexml_engine/src/tex_tables.rs, parse_halign_template :1590. Broaden the slot test:
  `else if cc == Catcode::PARAM || meaning_is_param(&t)`
where meaning_is_param resolves the token's \let-meaning and returns true iff it is a catcode-
PARAM `#` (array.sty's \@sharp). (Mirror in read_alignment_template alignment.rs:990 if a package
routes a #-template through the LaTeX-preamble path.) This lets a package-assembled \ialign
preamble (\@sharp cell slot) parse as a proper template -> \lx@begin@alignment, and \omit/\noalign
/\cline/& then work. NOT "implement \@mkpream in Rust" — \@mkpream already runs and builds the
template; only the placeholder recognition is missing.
Guard: ialign_sharp_placeholder.tex -> 0 errors + <ltx:tabular> with cells a,b,c,d;
noalign_outside_CONTROL.tex -> keeps >=1 error; the literal-# \ialign stays working.
Risk: LOW-MED (broadens PARAM recognition to \let-to-# cs; literal # still matches; \@sharp is
specifically the array.sty placeholder). Gain: sgame + tabularcalc×3 + tabvar + epslatex-fr (~6
docs) AND advances the parked tabu/srdp effort (their \@mkpream-built \@sharp templates become
readable — the "principled route" the tabu report deferred).

================================================================================
# ROUND 2 — CHECKPOINT N: `}` "switched to mode vertical" family — DRAINED on b54x
================================================================================
The 14-doc `} Attempt to close a group that switched to mode vertical` family from the s36
(b54q) residue is RESOLVED / cascade-only on b54x — the landed 54x-54z mode-frame batches
drained the expl3-box root. Per-doc re-verification (b54x):
  CLEAN (0 errors):        pascaltriangle, circledtext, joinbox, biblatex-caspervector
  `}`-vertical GONE, shifted to unrelated undefined-macro/encoding errors:
     codebox-doc-en (now \pkg/\url undefined), sduthesis-demo (now inputencoding utf8)
  CASCADE, not a genuine root:
     pst-exa-doc: the `}`-vertical is at \begin{pspicture} (line 116) but is a CASCADE from a
       MISSING INCLUDE `Can't find TeX file pst-exa-doc.inc` (first error) that corrupts state;
       a minimal `\begin{pspicture}[showgrid](4,4)…\end{pspicture}` is CLEAN (0 errors). Not an
       expl3-box / mode-frame root — it is a missing-file cascade (pstricks graphics lane).
  PARKED / other-root first errors (the `}`-vertical is a downstream cascade):
     shtthesis-user-guide, suanpan-l3: FIRST error `\luatexattributedef`/`\ltj@@attr@zero` =
       luatexja (CJK) PARKED family. thesis-gwu/thesis-sample: FIRST error `\fancyhf` undefined
       (fancyhdr, an undefined-macro root, not alignment).
Grep of every b54x run stderr: ONLY pst-exa-doc still emits `switched to mode vertical` (1×, the
cascade above). NO live expl3-box `vbox_set:Nn`/`hcoffin_set:Nn`/l3draw `\draw_begin:` vertical-
mode-frame root remains — do NOT re-open this family; its residue was b54q-stale.
Recommendation: retire this family from the alignment lane. The two remaining tails are
non-alignment and already lane-owned elsewhere: pst-exa missing-`.inc` (IO/missing-file),
shtthesis/suanpan-l3 luatexja (parked CJK), thesis-gwu `\fancyhf` (undefined-macro).
