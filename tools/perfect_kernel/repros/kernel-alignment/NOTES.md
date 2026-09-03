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
