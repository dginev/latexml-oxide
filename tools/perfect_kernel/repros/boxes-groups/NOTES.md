# boxes-groups — Wave 15 Checkpoint 1 (candidate list + top-3 repros)

Binary: /home/deyan/data/pk_bin/latexml_oxide.b54r (batch 54n). Prior round's 3 repros
(restricted_hbox_uline_examdesign, restricted_hbox_mbox_bgroup, restricted_hbox_grammar_newcommand)
re-verified GREEN (0 errors, conv complete) under b54r — OD #188 held.

## Ranked candidates (frame-error docs from w14/first_errors.tsv, by docs x capped err-lines)

ROOT 1 — group-restore / \aftergroup straddling an ALIGNMENT CELL boundary.
  Symptom: `Error:unexpected:\endgroup Attempt to close non-boxing group`,
  `current frame is boxing group due to T_CS[\lx@begin@alignment]` (stomach.rs:839).
  - uantwerpendocs/uantwerpenexam-example2 (41, pdflatex) — babel \selectlanguage in
    \begin{tabular}{ccc} cells (\engdut, uantwerpenexam.cls:426). ROOT-CAUSED, see below.
    (Other uantwerpendocs *-example use \engdut/\engdutmc too -> multi-doc.)
  - derivative/derivative (101, LUALATEX) — expl3 \seq_map_inline emitting &/\\ + $...$
    into \center\tabular (derivative.tex:852-866). SAME symptom/frame, DISTINCT trigger
    (no babel). My pdflatex-preload minimal replica (deriv_seqmap) was CLEAN, so the
    real trigger needs the lualatex+unicode-math path; needs `[luatex,...]` preload +
    minimization at Checkpoint N. Treat as a sibling of ROOT 1, confirm at N whether the
    same fix covers it.
  Combined first-error mass: ~142 capped lines (largest in topic).

ROOT 2 — \begingroup opened inside a BOX, straddling box end.
  Symptom: `\hbox Attempt to end mode restricted_horizontal`,
  `current frame is non-boxing group due to T_CS[\begingroup]` (stomach.rs:1049).
  - modernposter/demo (5, pdflatex) — pgf \node[text width=..] in \posterbox
    (modernposter.cls:194-215). RED-today confirmed (16 err). Minimal deferred (pgf-node
    /pgfscope internals; plain tikz node, text-width node, node+itemize, node+\\ all CLEAN
    — same wall the previous round hit). Checkpoint-N minimization target.
  - hyperbar/example (1, pdflatex) — hyperref Form field \begingroup straddle. RED (1 err),
    but Perl is WORSE (Form env + \TextField undefined in Perl) -> narrow hyperref/hyperbar
    Form binding gap, NOT a kernel win. Lowest priority.

ROOT 3 — box / \end{env} boxing-group straddle, internal_vertical.
  Symptom: `\endminipage`/`\endlx@list`/`}` `Attempt to end mode internal_vertical`.
  - functional/functional (20, LUALATEX) — math delimiter (\lx@hidden@bgroup, tex_box.rs:451)
    from \Arg/\meta inside \begin{syntax}/\begin{minipage} straddling \end{minipage}
    (functional.tex:62-67). Frame owner \lx@hidden@bgroup. Math-delim-in-minipage; deeper.
  - nih/example-biosketch (3, pdflatex) — \endlx@list, list clone (P58).
  - uwthesis (1) — `}` internal_vertical, \titlepage. pst-exa (3, lualatex) — `}` vertical.

NOT boxes-groups (recommend reassign):
  - lshort-german/l2kurz (44) — `\end{verse} Can't close`. Cause: `beispiel`=verse wrapper
    (l2kurz.tex:225) used INSIDE `LTXexample`, an `\lstnewenvironment` (listings, l2kurz.tex:77).
    Error stack "Current are: lstlisting, verse, beispiel, LTXexample x N" is a listings /
    verbatim example-env capture cascade -> STRING-MOUTH topic, not a group/box frame bug.
  - (alignment-owned, already excluded: t-angles, tablists, polynom, numerica, mhchem,
    pfdicons, shipunov; math: kblocks, titlecaps; parked pTeX: gckanbun, kksymbols.)

## ROOT 1 — root-caused (flagship)

Repros (both RED under b54r, first error `\endgroup Attempt to close non-boxing group`):
  - selectlang_in_cell_uantwerpen.tex  (13 lines; faithful to \engdut, cls:426; rust=5)
  - selectlang_twocell_min.tex          (9 lines; shrink — minipage NOT needed)

Mechanism (first principles):
  babel.def:738-742  `\selectlanguage `#1 = \bbl@push@language \aftergroup\bbl@pop@language
  \bbl@set@language{#1}. Placed as the first token of a tabular cell, the
  `\aftergroup\bbl@pop@language` is scheduled to run when the CELL group closes.
  babel.def:716-720  \bbl@pop@language pops the stack and re-runs \bbl@set@language ->
  \select@language (which, via \bbl@switch redefined by babel_support to run the LaTeXML
  primitive \ltx@bbl@select@language, babel_support.sty.ltxml:144-149) executes a full
  language-select and its \endgroup.
  tex.web S1063 handle_right_brace / S1131-S1134 (alignment `\endtemplate`/`\cr`): in real
  TeX each \halign cell is a genuine save-stack group opened by the template's `{` and
  closed by the `&`/`\cr` `\endtemplate` `}`; `\aftergroup` there attaches to THAT cell
  group. LaTeXML's alignment cell model (latexml_core/src/alignment.rs start_column:461 +
  gullet.rs handle_template:659 / handle_marker:3370) does NOT push a matching non-boxing
  group frame per cell that `\aftergroup` and `\currentgrouplevel` (babel.def:702-709) can
  bind to. So `\bbl@pop@language` / its `\endgroup` fire while the top frame is the
  \lx@begin@alignment BOXING group -> `\endgroup Attempt to close non-boxing group`, then the
  alignment teardown meets a group that switched to internal_vertical / a stray `&`.
  Isolated controls CLEAN: `\aftergroup\relax` in a cell (ag_cell), `\begingroup..\endgroup`
  in a cell (bg_cell), and single \selectlanguage in the FIRST cell (sl_min) — the break
  needs \selectlanguage in a non-first cell so the \aftergroup fires across the cell boundary.

Classification: SHARED. Perl (same-host, same preload) fails identically on the shrink,
  3 errors: `\@@tabular Attempt to end mode restricted_horizontal`,
  `\endgroup Attempt to close a group that switched to mode restricted_horizontal`,
  `& Stray alignment "&"`. pdflatex 0 errors (the two-column exam layout ships). In scope
  (surpass-Perl approved; pdflatex is the clean oracle). Rust = 5 errors.

Fix direction (to confirm at Checkpoint N): give each alignment cell a real non-boxing
  group frame in the cell-open/cell-close path (alignment.rs start_column / the &,\cr
  handlers in gullet.rs handle_template:659) so that (a) `\aftergroup` tokens scheduled
  inside a cell fire at cell close bound to the cell frame, not the enclosing \lx@begin@
  alignment boxing frame, and (b) `\currentgrouplevel` inside a cell is non-zero
  (babel.def:705 resets \bbl@language@stack when it reads level 0). Faithful to tex.web
  S1063: the cell IS a group. Must NOT regress well-nested cells (b_mp, b_sl1, deriv_seqmap
  all CLEAN today).
Guard (on selectlang_twocell_min.tex + selectlang_in_cell_uantwerpen.tex): 0 Error/Fatal
  AND the tabular renders — assert >=1 <ltx:tabular> containing >=2 <ltx:td>. NB: TODAY
  both Rust AND Perl DROP the tabular entirely (0 <ltx:tabular> in either output) because
  they error out; the structural assertion only passes post-fix.
Risk: MED-HIGH — touches every alignment cell (all \halign/tabular/array/matrix). Re-run the
  alignment topic's goldens and the p{}-cell witnesses before landing; coordinate with the
  alignment topic agent (this root sits on the group/alignment seam).
Expected corpus gain: uantwerpendocs example2 (41) + likely sibling *-example docs that use
  \engdut; derivative (101) IF the same cell-group fix covers the expl3 trigger (confirm at N).

## Dead ends (Checkpoint 1)
- `\aftergroup\relax`, `\begingroup x\endgroup`, single \selectlanguage-in-first-cell, minipage-in-cell,
  \selectlanguage+one minipage: all CLEAN. Break needs \selectlanguage in a NON-first cell.
- deriv_seqmap (expl3 \seq_map emitting &/\\+$..$ into \center\tabular) CLEAN under pdflatex
  preload — derivative's real trigger is the lualatex/unicode-math path; re-minimize under [luatex].
- tikz node (plain), node[text width], node+itemize, node+\\ : all CLEAN — modernposter's
  \begingroup straddle is in the pgf node/pgfscope shipout path; not reachable by a bare node.
- align_lowlevel (\center\tabular..\endtabular\endcenter) and align_xparse_body (xparse b-body
  wrapping a \tabular): CLEAN — low-level tabular + body-arg are fine; babel \aftergroup is the trigger.

## Checkpoint N — ROOT 1 fix design: "each alignment cell is a real (align_group) group"

Binary b54t. Repros still RED (selectlang_twocell_min = 5 err; selectlang_in_cell = 5;
derivative under [rawstyles,rawclasses,luatex] = 102 err, first `\endgroup Attempt to close
non-boxing group` at \end{example} line 1454, frame \lx@begin@alignment — SAME root confirmed).

### (1) tex.web mechanism — the per-cell save level and what \aftergroup binds to
- init_align §774 (tex.web:15332-15338): `push_alignment; push_nest;`
  `scan_spec(align_group,false)` opens the WHOLE-alignment `align_group` save level (the
  `\halign{` brace); then a SECOND `new_save_level(align_group)` (:15338) opens the FIRST
  cell's save level.
- init_row §786 (:15533) `push_nest` (row list level); init_span §787 (:15546) `push_nest`
  (column list level). NEITHER calls new_save_level — the row/column are NEST (list/mode)
  levels, SEPARATE from the align_group SAVE level.
- init_col §788 (:15560) starts the u_j template (align_state:=1000000).
- fin_col §791 (:15614-15625) at `endv` (end of v_j template): `if extra_info<>span_code
  then begin unsave; new_save_level(align_group);` — CLOSE the cell's align_group save level
  (this fires the cell's \aftergroup), OPEN the next cell's; THEN `Package an unset box`
  which does `pop_nest` (:15680) = close the column LIST level and package the cell box.
  KEY ORDER: `unsave` (save level, fires \aftergroup) precedes `pop_nest` (list packaging).
- fin_align §800 (:15750-15754): `unsave` (last cell's align_group) then `unsave`
  (whole-alignment align_group) — exactly TWO align_group levels remain.
So per cell body there is EXACTLY ONE `align_group` save level; `\aftergroup` binds to it and
fires at fin_col's `unsave`, which is BEFORE the cell's list is packaged (nest still open),
and `\currentgrouplevel` counts it (>=1 inside a cell, more with enclosing groups).

### (2) Map onto latexml_core (current state)
- Cell open: alignment.rs `start_column` (:448) calls `bgroup()` (:454) =
  stomach.rs `push_stack_frame(false)` — a BOXING group: it pushes a binding frame AND
  pushes the token onto the `boxing` list-vec (stomach.rs:396,566-569). So Rust COUPLES
  tex.web's separate nest-level and align_group-save-level into ONE boxing frame.
- Cell close: `end_column` (:466) calls `egroup()` (:468) = `pop_stack_frame(false)`:
  removes `afterGroup` (:645), runs beforeAfterGroup, `pop_frame()` (:647), `boxing.pop()`
  (:650), THEN unreads the `\aftergroup` tokens (:653-661). So \aftergroup fires AFTER the
  binding frame AND the boxing/list level are popped = one boxing level too shallow vs tex.web.
- `\aftergroup Token` = `push_value("afterGroup", t)` (tex_macro.rs:45) — appends to the
  CURRENT frame's afterGroup VecDeque (bound per-frame at push_stack_frame stomach.rs:550).
- `\currentgrouplevel` = a CONSTANT-ZERO readonly register (etex.rs:191), NOT wired to
  get_frame_depth() (state.rs:2752). So it always reads 0.
- `&`/`\cr` path: gullet.rs `handle_template` (:650-687) inserts the v-template `post` +
  `\lx@alignment@row@after`; the cell bgroup/egroup are driven by `\lx@begin@alignment`
  (start_column, via tex_tables.rs digest_alignment_column:1020) and end_column.
- Group-kind checks: `egroup()` errors if top is non-boxing (:742 "close boxing group");
  `endgroup()` errors if top is boxing (:837-846 "Attempt to close non-boxing group") — the
  babel/derivative error lands here because the top is the cell's boxing bgroup frame.

### (2b) Why it fails (forensics, LXML_TRACE_BOUND_MODE on selectlang_twocell_min)
babel `\selectlanguage `#1 (babel.def:738-742) = `\bbl@push@language \aftergroup
\bbl@pop@language \bbl@set@language{#1}`. The `\aftergroup\bbl@pop@language` binds to the
cell frame. In cell 2 the trace shows many BALANCED `\begingroup/\endgroup` pairs at frame
depth 6<->7 (babel's encoding/hook groups), then ONE extra `\endgroup` fires at pre-depth=5
and lands on the cell's boxing `\lx@begin@alignment` frame -> "close non-boxing group". The
extra `\endgroup` is unbalanced by exactly one BOXING level because `\bbl@pop@language`
(-> \bbl@set@language -> \select@language, its \begingroup..\endgroup encoding switch) runs
AFTER egroup already popped the cell's boxing frame (Rust) instead of while it is still open
(tex.web fin_col unsave-before-pop_nest). Contrast: `\selectlanguage` inside a plain
`\begingroup..\endgroup` (sl_begingroup/sl_nested) is CLEAN — there `\aftergroup` fires at
the matching `\endgroup` with nothing packaged in between, so the off-by-one never appears.

### (3) Minimal change — add a non-boxing "align-cell" frame INSIDE the boxing cell frame
This is tex.web-faithful: the align_group SAVE level (C) is separate from and nested inside
the column LIST/nest level (the boxing frame B), and closes BEFORE the list is packaged.
- alignment.rs `start_column`: after `bgroup()` (B, :454) and `next_column()` (:460),
  push a NON-BOXING frame C: `push_stack_frame(true)` (stomach.rs:541), with
  groupInitiator = an align-cell marker token (e.g. `\lx@begin@alignment` or a dedicated
  `\lx@align@cell`) so diagnostics read sensibly. C carries the cell's afterGroup + local
  bindings; it does NOT touch the `boxing` vec (nobox=true).
- alignment.rs `end_column`: BEFORE `egroup()` (:468), pop C: `pop_stack_frame(true)` —
  this flushes C's `\aftergroup` tokens (stomach.rs:653) while B's boxing/list level is
  STILL open, matching fin_col `unsave` before `pop_nest`. Then `egroup()` pops B and
  packages the cell box.
- ORDER (innermost last): boxing/list frame B (bgroup, holds the cell's box + mode) ->
  non-boxing align-cell frame C (top, holds \aftergroup + cell-local defs). Pop C first
  (fires \aftergroup, B still open), then B (egroup, package). Bracket the WHOLE cell body
  incl. the `>{}` u-template / `<{}` v-template between C-push and C-pop (tex.web align_group
  brackets u_j..v_j).
- Well-nested `\begingroup..\endgroup` inside a cell push/pop frames ABOVE C -> balanced,
  unaffected. A stray `\endgroup` (babel) now pops C cleanly (non-boxing) at the correct
  depth; with the timing fixed babel's own groups balance and no stray `\endgroup` reaches B.
- Do NOT globally reorder pop_stack_frame(false) (flush afterGroup before boxing.pop): that
  would wrongly change \aftergroup timing for EVERY \hbox/\vbox/`{...}` (for a box, tex.web
  fires \aftergroup AFTER packaging; only align_group fires before pop_nest). The cell is the
  special case, so the fix must be cell-local (C), not a global reorder.
- COMPLEMENT (separate, defensive): wire `\currentgrouplevel` (etex.rs:191) to
  get_frame_depth() so babel's `\bbl@push@language` (babel.def:705) PUSHES (not resets) the
  language stack when nested. NOT required for the repros (the C-timing fix makes each
  \bbl@pop@language balanced regardless of the reset), and it is a BROAD change (many
  packages test \ifnum\currentgrouplevel>0), so gate/validate it independently.

### (4) Regression surface (each: does C change behavior?)
- \multicolumn / \omit: still one start_column/end_column per resulting cell; their internal
  grouping is balanced inside C. NO change (C pops with empty afterGroup).
- colortbl \columncolor, array `>{decl}`/`<{decl}` (incl. `>{$}c<{$}` math-column): u/v
  templates run between C-push and C-pop; their `$..$`/decls are balanced within the cell and
  are already DIGESTED into B's list before C pops. NO change to output; VALIDATE that
  cell-local font/color decls (`>{\bfseries}`) still reach the digested content (they apply
  during body digestion, before popC). Guard candidate.
- \noalign, \hline/\cline: run BETWEEN rows/cells, outside any cell — C is per-cell, untouched.
- nested tabular: inner cells get their own C; frames nest cleanly. NO change.
- math alignments align/gather/matrix (amsmath \halign — §785 applies to EVERY \halign):
  math cells also get C. \aftergroup in math cells is rare; the per-cell hidden `$` pairing
  (\lx@dollar@in@mathmode, owned by the ALIGNMENT topic agent) is balanced within the cell,
  so C brackets it transparently. COORDINATE with the alignment agent; re-run math-align goldens.
- Row group (end_row:442 egroup): analogous but out of scope here (\aftergroup at row scope is
  rare); leave unless a row-scope witness appears.

### (5) derivative confirmation
Under `[rawstyles,rawclasses,luatex]latexml.sty`, b54t: 102 errors, first identical
(`\endgroup Attempt to close non-boxing group`, frame \lx@begin@alignment, at \end{example}).
SAME root. NOTE: derivative's trigger is expl3 `\seq_map_inline` emitting `&`/`\\`/`$..$`
(derivative.tex:852-866) with `\endtabular\endcenter` in \__mydoc_example_end: — the `&`/`\\`
come from inside expl3's map, so its \group_end:/align_state interaction leans on the
align_state pushback desync (gullet.rs:451-526, OTHER agent). Expect the C fix to be NECESSARY
but possibly not SUFFICIENT for derivative; re-verify derivative after BOTH the C fix and the
align_state fix land.

### Classification / guard / risk
SHARED (Perl 3 err on the shrink; pdflatex 0). Fix sites: latexml_core/src/alignment.rs
start_column (~:454, add push_stack_frame(true) after bgroup) + end_column (~:468, add
pop_stack_frame(true) before egroup). Guard (cluster_package_guards or a perfect_kernel batch
on selectlang_twocell_min.tex + selectlang_in_cell_uantwerpen.tex): 0 Error/Fatal AND the
tabular renders (>=1 <ltx:tabular> with >=2 <ltx:td>) — today BOTH Rust and Perl drop it.
Risk: MED-HIGH — touches every \halign/tabular/array/matrix cell; re-run alignment goldens,
p{}-cell/\vtop sizing witnesses, and math-align goldens; coordinate with the alignment agent
(cell-$ pairing) and do NOT edit gullet.rs:451-526 (align_state, other agent).

## Checkpoint N — ROOT 3 root-caused: physics2 `\delopen`/`\delclose` deferred `\aftergroup\egroup`

Binary b54w (ROOT 1 landed; both selectlang repros re-verified GREEN, 0 err).
Repros (RED b54w, lualatex-clean oracle):
  - egroup_braket_physics2.tex  (`\braket< a | b >`, faithful witness; 1 err)
  - egroup_delopen_activepipe_reduced.tex  (reduced, no braket/`\middle`; 102 err cascade)
First error both: `Error:unexpected:\egroup Attempt to close boxing group`,
`current frame is non-boxing group due to T_CS[\begingroup]` (stomach.rs:744).

### Mechanism (first principles)
physics2.sty:83-84:
  `\DeclareRobustCommand\delopen{\mathopen{}\mathclose\bgroup\left}`
  `\DeclareRobustCommand\delclose{\aftergroup\egroup\right}`
`\delopen(` opens a `\bgroup` (boxing) then `\left(`; `\delclose)` DEFERS an `\egroup` via
`\aftergroup` to fire exactly when `\right` closes the `\left` group, so the whole
`\delopen(...\delclose)` acts as one `\mathclose`-boxed atom. In LaTeXML `\left(` =
`\@left ( \lx@hidden@bgroup` (tex_math.rs:961) = `bgroup()` (tex_box.rs:451); `\right)` =
`\lx@hidden@egroup@right` (etex.rs:965) = `egroup()` (tex_box.rs:460), which flushes the
frame's `\aftergroup` list (stomach.rs:653). The `ab.braket` module (phy-ab.braket.sty:55-58)
makes `|` math-active (`\mathcode="8000"`, `\def|{\egroup\phy@abb@bkv\bgroup}`), so the
delimiter body `\bgroup a|b\egroup` becomes two boxes `\bgroup a\egroup ... \bgroup b\egroup`.
With physics2 + unicode-math loaded (there IS a `unicode_math_sty.rs` binding), LaTeXML's
frame stack desyncs: the deferred `\egroup` (from `\delclose`'s `\aftergroup`) fires while the
module's outer `\begingroup` (phy-ab.braket.sty:55) is the top frame instead of the `\bgroup`
from `\delopen` => `\egroup Attempt to close boxing group`.

tex.web §1063-1065 (`handle_right_brace`/`off_save`): in real TeX `\aftergroup` saves a token
on the save stack of the CURRENT group and re-inserts it at that group's `unsave`. The math
`\left..\right` subformula is `math_left_group`/`math_shift` (§1191, §1194) nesting cleanly
inside the `\bgroup..\egroup` `simple_group`; `\aftergroup\egroup` fires at the `\right`
`unsave` of the math group, closing the `\bgroup` — balanced regardless of the active-`|`
box-split. LaTeXML's divergence: `\aftergroup` binds to `stomach.rs` per-frame `afterGroup`
and fires at `pop_stack_frame` (egroup), but the frame that `\aftergroup\egroup` targets — and
the order in which the active-`|`'s `\egroup`/`\bgroup`, the `\left`/`\lx@hidden@bgroup`, and
`\mathclose\bgroup` push/pop frames — does not match tex.web's save-stack nesting once
physics2/unicode-math's math-delimiter machinery is in play. Bare-kernel replicas of the same
macros (kern.tex, mc8000.tex, both_unimath.tex) are CLEAN; only with physics2+unicode-math
loaded does it desync — so the fault is in the LaTeXML math-delimiter/`\aftergroup` frame
interaction as exercised by that stack, not the physics2 bodies.

### Rust frame site + fix DIRECTION (needs main-session bisect + build)
Sites: `\lx@hidden@bgroup`/`\lx@hidden@egroup` = bgroup()/egroup() (tex_box.rs:451,460);
`\@left`..`\lx@hidden@bgroup` (tex_math.rs:961); `\lx@hidden@egroup@right` (etex.rs:965);
`\aftergroup` = push_value("afterGroup") (tex_macro.rs:45), flushed at pop_stack_frame
(stomach.rs:653); `\middle` (etex.rs:493, emit-only — NOT required for the bug). Direction:
add bgroup/egroup frame tracing (LXML_TRACE_BOUND_MODE only traces begingroup/endgroup/egroup-
ERROR) to egroup_delopen_activepipe_reduced.tex and identify which frame the deferred
`\egroup` targets vs which `\bgroup` (from `\delopen`'s `\mathclose\bgroup`) it should. Prime
suspect: `\mathclose\bgroup` — whether LaTeXML's `\mathclose` opens a boxing frame for the
implicit `\bgroup` at the SAME nesting real TeX does, vs absorbing it as an atom argument,
under unicode-math's redefined `\mathclose`/`\left`. Do NOT touch the alignment cell path.

### Classification / risk / gain
RUST-ONLY: same-host Perl never defines `\usephysicsmodule` (cannot raw-load physics2) — Perl
errors earlier and never reaches the braket; lualatex (oracle) is clean. In scope.
Guard: egroup_braket_physics2.tex — 0 Error/Fatal AND the fenced `<a|b>` renders (>=1
`<ltx:XMApp>`/`<ltx:XMTok>` in the display). Risk of the eventual fix: MED (touches
`\aftergroup`/math-delimiter frame ordering — re-run math-delimiter + `\left\middle\right`
goldens, braket.sty tests). Gain: physics2/physics2 + physics2-legacy = ~202 capped err-lines.

### hep-math / functional — SEPARATE roots (do not fold into physics2)
- hep-math (43, pdflatex): first error `\tempa` undefined; the 36 frame errors are
  `} Attempt to close a group that switched to mode restricted_horizontal`, frame owner
  `\eqnarray@row@after@` / `\lx@tag@intags` (s35 log) — an EQNARRAY-row / math-tag MODE-frame
  family, NOT the `\bgroup`/`\aftergroup` delimiter mechanism. Distinct root (alignment/math seam).
- functional (20, lualatex): `\endminipage Attempt to end mode internal_vertical`, frame
  `\lx@hidden@bgroup` (a `\left`-opened math-delimiter group straddling `\end{minipage}` in
  the codehigh/demohigh rendering). SAME frame-owner FAMILY as physics2 (unbalanced
  `\lx@hidden@bgroup`), plausibly the same underlying `\left`/`\aftergroup` frame bug, but the
  minimal trigger (bare `\meta`/`\Arg` in a minipage) is CLEAN — needs the codehigh machinery;
  re-attempt after the physics2 fix and check if it falls out.

### Dead ends (ROOT 3)
- kern.tex (explicit `\bgroup\left\langle\bgroup a\egroup\middle\vert\bgroup b\egroup
  \aftergroup\egroup\right\rangle`), mc8000.tex / actpipe.tex (arg-captured active-`|`),
  both_unimath.tex (robust `\ropen`/`\rclose` copies + unicode-math, NO physics2): all CLEAN.
  Only physics2 (or physics2+unicode-math) LOADED reproduces — rd_mycopy.tex (my robust copies
  INSIDE a physics2 doc) is RED, so it is the loaded math setup, not the macro bodies.
- `\middle` is NOT required (rd_nomid drops it, still RED). `\braket<\phi>` (no `|`) is CLEAN.
- `\delopen(\frac12\delclose)`, `\ab(\frac12)`, `\delopen(a\middle\vert b\delclose)`: CLEAN
  (single delimiter body, no active-`|` box-split).

## Checkpoint N — modernposter/demo REGRESSION (5 -> 16), NOT the cell-\aftergroup fix

Binaries: b54l (pre-regression, 5 err) vs b54y (HEAD, 16 err). Repro:
regression_modernposter_svgscope.tex (modernposter.cls, ~10 lines) — RED b54y, b54l has
only the 5 pre-existing (4x `\hbox Attempt to end mode restricted_horizontal` + 1 pgf `sep`).

### The 11 NEW errors (the regression)
2x `\egroup` + 7x `\endgroup` + 1x `\lxSVG@endscope` "Attempt to close a group that switched
to mode restricted_horizontal" (frame owner `\hbox`, stomach.rs:733/826) + 1x
`malformed:svg:g Attempt to close </svg:g>`. `\lxSVG@endscope` = `endgroup()`
(pgfsys_latexml_def.rs:1452) closing a pgf SVG scope (`\pgfsys@endscope`); pgf schedules scope
teardown via `\aftergroup` (pgfcorescopes.code.tex:165 `\aftergroup\pgf@collectresetcolor`).

### DECISIVELY ruled out
- **cell-`\aftergroup` digestion (alignment.rs end_column, my ROOT-1 fix)** — the coordinator's
  top hypothesis, but WRONG here: `grep -c 'lx@begin@alignment|start_column|handle_template|
  alignCellGroup|@@tabular|halign'` on the full LXML_TRACE_BOUND_MODE trace of the repro = **0**.
  modernposter's pgf nodes create NO alignment cell, so the `alignCellGroup` aftergroup path is
  never entered. Bare `\node[align=center]{a\\b}`, tikz-in-tabular, nested tikzpictures with
  hbox/scope all CLEAN under both b54l and b54y.
- **`\g@addto@macro` as latex.ltx's macro (batch 54q, KPE #170)** — direct test
  (`\g@addto@macro` in a tikz node + `\draw`) CLEAN under both; not present in the trace near
  the errors.
- **`\expandafter` brace-retraction reorder (54o/54p, KPE #169)** — only shifts the align-state
  ledger (brace count for `&`); no alignment here, so inert.

### Narrowed root (needs an intermediate-binary bisect — main session, requires a build)
The error frame is `mode-switch to restricted_horizontal due to \hbox`, and the closers are pgf
SVG-scope grouping. Prime suspect: the **`\hbox` restricted_horizontal body reader change** —
batch 54m "one-frame hbox reader" (base_utilities.rs `predigest_box_contents_in_mode`) or 54n
"HBoxContents box bodies (OD #188)" (tex_box.rs HBoxContents) — leaving the `\hbox` mode-frame
open across pgf's SVG scope `\begingroup..\endgroup`, so `\pgfsys@endscope`/`\lxSVG@endscope`
(`endgroup()`) and the surrounding `\endgroup`/`\egroup` hit the still-open `\hbox` frame. Both
`\maketitle` (full-page overlay node) and `\posterbox` (nested tikzpicture nodes) trigger it
independently — both are pgf nodes with SVG scope + `\hbox` text box. First-error backtrace:
`Constructor::execute_before_digest` -> `digest::<Tokens>` -> `invoke_token` -> `egroup`
(constructor.rs:299/312) — a box Constructor's before_digest digesting a body that contains a
stray scope-closer. NB: if b54l already includes 54m/54n (i.e. b54l is the batch-54n binary,
not 54l), re-scope to the 54o-54r changes that touch boxing (`titlepage` un-locked, 54r
locked-setter internals) — check b54l's embedded revision first (`--VERSION` is unsupported;
read the binary's build banner).

### Fix DIRECTION
Bisect 54m vs 54n by building the intermediate binaries and re-running
regression_modernposter_svgscope.tex; the restricted_horizontal `\hbox` reader must open/close
its mode-frame symmetrically with an enclosing pgf SVG scope's `\begingroup..\endgroup` so a
scope `\endgroup`/`\lxSVG@endscope` never lands on the `\hbox` mode-frame. Guard: 0 lines of
"close a group that switched to mode restricted_horizontal" on the repro (the pre-existing 4x
`\hbox Attempt to end mode` is a SEPARATE, older ROOT-2 bug — do not conflate).

### Dead ends
- Self-contained (non-cls) reduction: `\node[align=center]{a\\b}`, `\node[draw,fill,align=
  center,text width]{A\\B}`, tikzpicture-in-tabular, `\begin{scope}`/`\fill` in align node,
  nested tikzpicture with `\hbox` node — all CLEAN under BOTH binaries. modernposter's specific
  nested-pgf node theme (metropolis colors + `remember picture,overlay` title + `shapes.misc`
  rounded boxes) is required; could not shrink below the class.
- Hand-built `\aftergroup`+scope-closer-in-cell kernel cases error under BOTH (malformed), not
  a clean regression isolation.

## Checkpoint 1 (wave-15 re-run) — modernposter 5→16 BISECT RESULT

Bisect on modernposter/demo.tex (pdflatex oracle, preload [rawstyles,rawclasses]latexml.sty),
error counts are REAL (minus the trailing read-only-dir "Permission denied" log-write = +1):
  b54l=5 b54m=5 b54n=5 | b54o=16 | b54p=4(BROKEN) b54q..b54y=16
REGRESSION INTRODUCED AT **b54o** (commit cfa13124b8 "batches 54o/54p"). b54n is the last
clean baseline (5 = 4 pre-existing `\hbox Attempt to end mode restricted_horizontal` ROOT-2 +
1 pgf "No shape named sep"). b54o adds the 11 NEW: 7×`\endgroup` + 2×`\egroup` +
1×`\lxSVG@endscope` "close a group that switched to mode restricted_horizontal" +
1×`malformed:svg:g`. b54p is a RED HERRING (modernposter.cls failed to raw-load → undefined
\posterbox/\postercolumn/\highlight/\doubleposterbox, masking the regression); b54q restores
the class. Regression is COMMITTED and persists to HEAD (b54t=16, b54y=16).

### Frame mechanism (site pinned; source hunk NOT pinnable read-only)
During pgf SVG shipout of `\maketitle`/`\posterbox` (pgfsys SVG node-text: two nested `\hbox`
restricted_horizontal mode frames at `\lxSVG@beginscope`), **b54o pushes ONE FEWER stomach
frame than b54n**. First divergence at begingroup/endgroup op-index 734: b54n `\begingroup`
pre-depth=7, b54o pre-depth=6 (widens to 2 by op 746). The op-TYPE sequence
(begingroup/endgroup/bgroup/egroup/begin_mode_opt/set_mode) is IDENTICAL until the error tail
(op 4532+); the deficit is an UNTRACED `push_stack_frame` (NOT begingroup/bgroup/begin_mode).
With 1–2 fewer frames, pgf's SVG-scope teardown (`\lxSVG@endscope`=endgroup, pgfsys_latexml_def
.rs:1452; the scope's own `\begingroup..\endgroup`; the `\aftergroup`-scheduled
`\pgf@collectresetcolor`, pgfcorescopes.code.tex:165) UNDERFLOWS into the still-open `\hbox`
mode-switch frame → the 10 close/end-group errors (stomach.rs egroup:733, endgroup:826), and
the desync emits an orphan `<svg:svg><svg:g/></svg:svg>` then `malformed:svg:g`. First XML
divergence = that orphan svg inserted after `<p><text width="0.0pt"/>`; SVG numerics identical
up to it. Frame owner = `\hbox` (mode-switch restricted_horizontal); closer = pgf SVG scope
`\endgroup`/`\lxSVG@endscope`.

### RULED OUT (empirical) — saves the fix session from re-checking
- alignment.rs cell-`\aftergroup` (alignCellGroup): 0 alignment activity in full trace
  (LXML_TRACE_ALIGN_STATE=0 lines, no start_column/end_column/afterGroup). Coordinator's top
  hypothesis is WRONG here (prior round agreed).
- tex_macro.rs `\expandafter` retract reorder: retract_scanned_braces (gullet.rs:491-499) ONLY
  touches the align brace ledger; inert without an alignment (align_state stays ~1000000).
- document.rs math-in-ref inline-Math auto-open: 0 ltx:Math in either XML.
- document.rs sectioning-in-item leniency: 0 sectioning-tag diff between XMLs.
- pgfmath integer literals: SVG coordinate/size numerics identical (no numeric divergence).
- pdftex \pdfdest/\pdfoutline: not invoked (0). color_sty xcolor-storage: color-NAME resolution,
  no group push/pop. hyperref \autopageref / verbatim / utf8 / \matheqdirmode / beamer /
  marginnote / soul / ltablex / schooldocs: not exercised by the demo.
- The +2 begin_mode_opt / +2 egroup / +2 svg:g are CASCADE symptoms (op 4532+), not the root.

### Next step (main session, requires build): SOURCE-bisect cfa13124b8
Read-only cannot pin the hunk (op-types identical; deficit is an untraced push_stack_frame).
git-revert cfa13124b8's hunks against HEAD one at a time + rebuild, test
regression_modernposter_svgscope.tex (RED=16, target: 0 lines of "close a group that switched
to mode restricted_horizontal"; the pre-existing 4×`\hbox Attempt to end mode` is SEPARATE
older ROOT-2). All the obvious group-primitive/document changes are eliminated above, so the
culprit is a frame-count change in a code path not visible as a begingroup/bgroup/begin_mode
trace op — inspect stomach.rs push_stack_frame call sites and any Constructor before_digest /
box-reader path touched between the b54n (04:56) and b54o (06:12) dev states.

Repros verified RED (b54t): regression_modernposter_svgscope.tex=16;
egroup_braket_physics2.tex=1; egroup_delopen_activepipe_reduced.tex=102 (Fatal TooMany).

### Root-3 candidate docs (s36 residue, for later checkpoints)
- `\lx@begin@alignment Attempt to close boxing group`: t-angles/t-manual (101, alignment-seam);
  shipunov/boldline-ex-en (4, restricted_horizontal). hyperbar/example (1, \end{Form}, narrow).
- `\lx@add@frontmatter@until … internal_vertical`: screenplay-pkg/screenplay-pkg (6) — exact
  pattern. (functional 20 = root-2 sibling; nih 3 = list-clone P58; pfdicons/tikzcodeblocks =
  \@end@tabular, alignment-owned.)

## Checkpoint N (wave-15) — ROOT 2 physics2 \delopen/\delclose deferred \egroup — ROOT-CAUSED

Repros RED b54t: egroup_braket_physics2.tex=1; egroup_delopen_activepipe_reduced.tex=102(Fatal).
First error both: `\egroup Attempt to close boxing group, current frame is non-boxing group due
to T_CS[\begingroup]` (stomach.rs:744). The `\begingroup` = phy-ab.braket.sty:55 `\phy@@ab@bk`.

### Mechanism (file:line)
physics2.sty:83-84: `\delopen=\mathopen{}\mathclose\bgroup\left`, `\delclose=\aftergroup\egroup
\right`. `\delopen(` opens a `\bgroup` (the `\mathclose` nucleus group) then `\left`; `\delclose)`
DEFERS an `\egroup` via `\aftergroup` to fire when `\right` closes the `\left` subformula, so the
whole `\delopen…\delclose` is one auto-sized `\mathclose` atom. phy-ab.braket.sty:55-58
`\phy@@ab@bk#1` = `\begingroup \mathcode`|="8000 \def|{\egroup\phy@abb@bkv\bgroup}(=\egroup
\middle\vert\bgroup) … \delopen\langle\bgroup#1\egroup\delclose\rangle\endgroup`. The active `|`
SPLITS the delimiter body into multiple `\bgroup…\egroup` boxes.

tex.web §1063-1065 (handle_right_brace) / §1184-1194 (fin_mlist/`\right` unsave): real TeX pairs
`\bgroup`↔`\egroup` at EXECUTION time. `\aftergroup\egroup` saves `\egroup` on the math_left_group
(the `\left` group) save level; `\right` does the group's `unsave` and back_inputs `\egroup`,
which then closes `\delopen`'s `\bgroup` (a simple_group). Balanced regardless of the active-`|`
split, because group extents are decided when `}`/`\egroup` are EXECUTED.

Rust divergence: `\left` = `\@left <delim> \lx@hidden@bgroup` (tex_math.rs:940/985), and
`\lx@hidden@bgroup` is a **capture_body DefConstructor** (tex_box.rs:451) — it reads the subformula
body at TOKEN level up to `\lx@hidden@egroup@right` (`\right`, etex.rs:965); `\mathclose` =
`DefConstructor "\mathclose Digested" bounded` (tex_math.rs:887). Token-level body capture does NOT
execute the active `|` (mathcode 8000 only fires in live math digestion) nor the `\aftergroup`
deferral, so the split `\egroup`/`\bgroup` pairs plus `\delclose`'s deferred `\egroup` DESYNC the
`\bgroup`↔`\egroup` pairing; the deferred `\egroup` never meets `\delopen`'s `\bgroup` (it was
consumed/closed during the token capture) and fires against the OUTER `\begingroup` frame G1
(phy-ab.braket.sty:55) → `\egroup Attempt to close boxing group` (stomach.rs:744).

### Frame the \egroup meets
Non-boxing `\begingroup` (semi_simple) frame G1 from phy-ab.braket.sty:55 — NOT a `\hbox`
mode-switch, NOT a math frame. It should meet `\delopen`'s `\bgroup` (boxing/simple_group).

### Controls (all CLEAN b54t → isolate the trigger = active-| box-split)
- CONTROL_left_right_plain.tex `\left\langle a\middle| b\right\rangle` — native fences clean.
- CONTROL_bgroup_egroup_plain.tex `{\bgroup x\egroup}` + `\mathclose\bgroup a\egroup` — clean.
- CONTROL_delopen_single_physics2.tex `\delopen\langle a\delclose\rangle` (NO active |) — CLEAN:
  `\delopen`/`\delclose` + deferred `\egroup` work with a SINGLE delimiter body; the active-`|`
  `\egroup…\bgroup` split is the sole trigger.

### Classification: RUST-ONLY
Perl (same host, same preload) CANNOT raw-load physics2: falls back to generic physics.sty
(reads "physics2"→versioned "physics"), then `\usephysicsmodule` undefined (1 error) — never
digests the braket. lualatex oracle clean. Rust reaches the root only because it raw-loads
physics2. So the braket root is RUST-ONLY (Perl's 1 error is a different, earlier fallback).

### Fix — NOT kernel egroup group-kind; = physics2 contrib BINDING
- egroup's group-kind matching (stomach.rs:733/744) is ALREADY tex.web-faithful: `\egroup`
  closes a simple_group (boxing) and MUST error on a semi_simple_group (`\begingroup`, §1069
  "Missing \endgroup inserted"). Making it leniently close the `\begingroup` frame would be
  UNFAITHFUL (real TeX errors there too) and a stopgap. DO NOT change egroup.
- Root is upstream: LaTeXML's capture_body math-delimiter reader (`\lx@hidden@bgroup` +
  `\mathclose Digested`) can't pair a `\bgroup` whose closing `\egroup` is `\aftergroup`-deferred
  and whose body is split by an active `|`. BINDINGS OUTRANK RAW and physics2 is a contributed
  (non-upstream) package with NO existing binding → add `latexml_contrib/src/physics2_sty.rs`
  binding the auto-brace delimiter families (`\phy@@ab@bk`/`\phy@@mb@bk` and the ab/ab.braket/
  ab.legacy generators, ultimately `\phy@abopen`/`\phy@abclose`=`\delopen`/`\delclose`) to native
  `\left…\middle…\right` fences (CONTROL_left_right_plain proves that path is clean) instead of the
  `\aftergroup\egroup\bgroup` deferral. Faithful to physics2's rendered output (auto-sized fences
  with a middle bar). Guard: egroup_braket_physics2.tex — 0 Error/Fatal AND the fenced ⟨a|b⟩
  renders (≥1 `<ltx:XMApp>` with an OPEN/CLOSE/VERTBAR fence + the two operands).
- Kernel alternative (REJECTED for one raw-load-only pkg, MED-HIGH risk): make `\left` keep a LIVE
  frame (not capture_body) so `\aftergroup` binds to it and the deferred `\egroup` fires at
  `\right`'s pop (tex.web §1184-1194) — touches EVERY `\left`/`\right`/math-delimiter + the math
  parser's subformula capture.
Risk (binding): MED — cover ab/ab.braket/ab.legacy faithfully; re-run physics2/physics2 +
physics2-legacy (~202 capped err-lines). Expected gain: physics2 docs → 0.

### Dead ends
- `\delopen(single-body\delclose)`, `\left…\middle…\right`, `{\bgroup..\egroup}`,
  `\mathclose\bgroup a\egroup`: all CLEAN. Trigger requires the active-`|` `\egroup…\bgroup` split
  INSIDE the `\delopen…\delclose` deferred-`\egroup` span (prior round: `\middle` not required,
  `\braket<\phi>` no-`|` clean, bare-kernel robust copies without physics2 clean).
