# boxes-groups — Checkpoint 1 candidate list + repros

Topic: a box/group opened in one macro and closed in another; mode-frame errors
(`Attempt to close a group that switched to mode …`, `Attempt to end mode …`,
`Attempt to close boxing/non-boxing group`). Theme 1 (ARCHITECTURE_THEMES.md:28).

## Ranked candidates (docs × capped error-lines, FIRST-error box/group root)

TIER 1 — clear restricted_horizontal box root, FIRST error, high impact:
- examdesign examplea/b/c — 3 docs × 101 (capped) = 303 lines. pdflatex-clean.
  SHARED (Perl fails identically). ulem `\uline` (cls:141-200) `\hbox\bgroup…\egroup\egroup`
  word boxes interleaving with `\@makebox` mode frame in the truefalse/MC answer KEY
  (`\item[\uline{\makebox[.5in][c]{…}}]`, cls:1208). repro: restricted_hbox_uline_examdesign.tex
- newcommand — 101 (capped) lines. pdflatex-clean. SHARED. syntax.sty `\mbox\bgroup … \egroup`
  (syntax.sty:158 `\syn@assist`, `\[[ … \]]` in `\begin{grammar}`). repros:
  restricted_hbox_mbox_bgroup.tex (minimal kernel) + restricted_hbox_grammar_newcommand.tex

TIER 2 — box/group straddle, FIRST error, smaller (Checkpoint-N candidates):
- functional (20, lualatex) — `\endminipage` internal_vertical, boxing group `\lx@hidden@bgroup`.
- modernposter/demo (5) — `\hbox` restricted_horizontal, non-boxing group `\begingroup` (pgf node box).
- nih/example-biosketch (3) — `\endlx@list` internal_vertical, mode-switch `\Onumerate` (list clone; P58).
- pst-exa (3, lualatex) — `}` vertical.
- hyperbar/example (1) — `\end{Form}` restricted_horizontal, non-boxing group `\begingroup` (hyperref Form).
- uwthesis (1) — `}` internal_vertical, mode-switch `\titlepage`.

TIER 3 — box/group error is NOT the first error (downstream of another root; lower priority):
- lshort-german/l2kurz (44) — first error "Can't close environment verse" (verse/lstlisting box).
- msc (34) — first error missing-file (\verbatiminput).
- tex-font-errors-cheatsheet (24) — first error `\noalign` (ALIGNMENT topic).
- adjmulticol (4) — first error undefined `\mult@@cols`.

EXCLUDED — belong to other topics (frame owned by alignment/math, or parked):
- alignment: derivative, pfdicons, shipunov, uantwerpendocs (`\lx@begin@alignment` / `\@@tabular`
  / `\lx@tabular@p@`), numerica, tablists, mhchem, polynom, t-angles.
- math: kblocks, titlecaps (`\lx@end@inline@math`).
- PARKED (Japanese pTeX): gckanbun, kksymbols (`\epTeXinputencoding`).

## Two candidate roots (to be root-caused one at a time at Checkpoint N)

Root A — box CONSTRUCTOR `{}` arg reader ignores implicit begin-group `\bgroup`.
  Witness: restricted_hbox_mbox_bgroup.tex (`\mbox\bgroup A … B\egroup`), grammar/newcommand.
  `\mbox`/`\@makebox` are DefConstructor "{}" (mode=>text, bounded) at
  latex_constructs.rs:10442/10461. When the `{}` arg is opened by an implicit begin-group
  token (`\let\bgroup={`) rather than a literal `{`, the arg reader does not scan the
  balanced `\bgroup..\egroup` group; `\bgroup` then runs inside the box as its own boxing
  frame and `\mbox`'s bounded end_mode meets that frame → `\mbox Attempt to end mode
  restricted_horizontal, current frame is boxing group due to \bgroup`.
  Perl faithful: readArg/readBalanced must treat `defined_as(T_BEGIN)` as the opener
  (TeXbook ch.24). Likely fix in the arg-reader (latexml_core gullet/mouth), NOT the box.
  Classification: SHARED (Perl 1 err on the minimal, pdflatex 0). Fix direction = brief (c):
  argument scan recognises implicit begin-group — CONFIRM at Checkpoint N.

Root B — restricted_horizontal hbox reader uses TWO frames where Perl uses ONE.
  Witness: restricted_hbox_uline_examdesign.tex. base_utilities.rs:3519
  predigest_box_contents_in_mode uses the faithful one-frame loop (3542-3574) ONLY for
  vertical modes; for restricted_horizontal it does begin_mode(mode) THEN
  invoke_token(T_BEGIN) (3576-3579) = a SECOND frame. Perl readBoxContents
  (TeX_Box.pool.ltxml:164-185) uses ONE frame (the mode frame IS the group) and a loop
  that stops at the matching T_END at the same frame depth for BOTH modes. With ulem's
  open-here/close-there `\hbox\bgroup…\egroup\egroup` boxes plus a `\@makebox` mode frame,
  the extra frame desyncs and every stray `\egroup`/`\@makebox` lands on the wrong frame.
  Fix direction = brief (a): extend the one-frame loop (3542) to restricted_horizontal
  (hbox/math don't paragraph-wrap → no repack-before-pop ordering needed).
  Classification: SHARED (Perl 7 err, pdflatex 0).

Whether A and B are one fix or two: CONFIRM at Checkpoint N (A may be a prerequisite of
the two-frame removal, or independent arg-reader work).

## Dead ends (Checkpoint 1)
- Isolated `\uline{\makebox…}`, `\mbox{\uline…}`, `\hbox{\uline…}`, `\item[\uline{\makebox…}]`
  in a plain list, malformed `\begin{list}%{label}{…}`: all CLEAN. examdesign needs the
  truefalse/MC KEY (answer pass) + `\NumberOfVersions`.
- `\setbox\x\hbox\bgroup\bgroup A … \egroup\egroup` split across macros: CLEAN (the raw
  `\hbox` two-frame path tolerates balanced `\bgroup/\egroup`); only the constructor arg
  (`\mbox\bgroup`) and the ulem+`\@makebox` interleave fail.
- tikz `\node{\parbox…}`, graphicx `\scalebox{\parbox…}`: CLEAN — modernposter needs the
  specific pgf node text-box path; minimization deferred.

## Checkpoint N #2 — Root B RESOLVED (root-caused; fix = option (a))

Repro: restricted_hbox_uline_examdesign.tex — RED with b54m (10 errors; first line
`Error:unexpected:\egroup Attempt to close a group that switched to mode restricted_horizontal`).
Root A's fix (implicit-`\bgroup` argument) did NOT touch this; it is a distinct root.

Mechanism (traced with LXML_TRACE_BOUND_MODE + the egroup-ERROR backtrace):
  The restricted_horizontal branch of predigest_box_contents_in_mode
  (base_utilities.rs:3576-3614) does `begin_mode("restricted_horizontal")` (mode frame MK)
  THEN `invoke_token(&T_BEGIN!())` (base_utilities.rs:3579) — the `{` primitive
  (tex_box.rs:361) which `bgroup()`s a SYNTHETIC group frame G1 and `digest_next_body`s
  the box body. So one `\hbox`/`\uline`-word box costs TWO frames (MK + G1) where Perl
  readBoxContents (TeX_Box.pool.ltxml:164-185) uses ONE — the mode frame IS the group,
  and the matching T_END ends the loop WITHOUT being invoked (`last if defined_as(T_END)
  && level >= getFrameDepth`, :182). tex.web §1083 `begin_box` pushes nest AND save level
  TOGETHER for `\hbox\bgroup` (§1100 `package` pops both) — Perl's one frame mirrors that
  single combined push; Rust's two frames split it.
  The egroup-ERROR backtrace at the failure: HBoxContents (tex_box.rs:629) →
  predigest_box_contents_in_mode:3579 (invoke_token T_BEGIN) → `{` digest_next_body ×2 →
  `\@makebox` Constructor (constructor.rs:299/322) → read_arguments_and_digest →
  ArgWrap::be_digested → the ISOLATED `digest(tokens)` (stomach.rs:1612) of `\@makebox`'s
  `{}` argument → a `}`/`\egroup` (T_END, tex_box.rs:439) → egroup() → ERROR, because the
  argument's isolated digest starts at the outer boxing depth and a stray `\egroup` from
  ulem's open-here/close-there `\hbox\bgroup…\bgroup` … `\egroup\egroup`
  (examdesign.cls:186-200 `\UL@start`/`\UL@stop`) reaches it and meets the still-open
  restricted_horizontal mode frame.
  Invariant broken: "the box's mode frame is closed by the box READER (loop termination on
  its own T_END), not by an `\egroup`." The extra synthetic G1 makes the reader depend on
  an `\egroup` to close the box; when a bounded constructor (`\@makebox`) inside the box
  digests its argument in isolation, the desynced `\egroup` from the open-here/close-there
  ulem box lands in that isolation and closes the wrong frame. (Simple well-nested hbox is
  fine — r3/p1/p2/p3 clean — because the 2 `\egroup`s pop G2+G1 and end_mode pops MK; the
  break only surfaces when the closing `\egroup` count is split across macros AND a bounded
  constructor's isolated arg-digest sits between the frames.)

Decision — fix is (a), NOT (b):
  (a) Make predigest_box_contents_in_mode use Perl's one-frame loop for restricted_horizontal
      (and math) exactly as it already does for vertical (base_utilities.rs:3542-3574):
      begin_mode(mode); level=get_frame_depth(); loop{ break on defined_as(T_END) &&
      level>=get_frame_depth(); else extend_box_list(invoke_token(t)) }; end_mode(mode).
      For horizontal DO NOT call simplify_vertical_list (a vertical List.pm simplification);
      set mode_tex = Math if IN_MATH else Text; set the "mode" property to the mode string.
      No repack: hbox/math don't paragraph-wrap (the vertical branch's leave_horizontal
      repack-before-pop ordering is irrelevant here), which is exactly why the two-frame
      shortcut was originally taken — but it is what desyncs.
      THEN add the `reversion` closure to HBoxContents (tex_box.rs:629) mirroring the one
      VBoxContents already has (tex_box.rs:640-645: `{` + arg + `}`), because the loop path
      returns a bare `List::new(boxes)` whose revert() drops the delimiting braces, so
      `\hbox{a}` must be re-wrapped to revert as `\hbox{a}`.
  (b) REJECTED. `\@makebox`'s isolated `digest(tokens)` of its `{}` arg (stomach.rs:1612)
      is the faithful DefConstructor behavior (Perl digests constructor args as bodies) and
      is correct for every well-nested case; the stray `\egroup` reaching it is the SYMPTOM
      of the hbox reader's extra frame, not a makebox bug. Changing makebox would mask, not fix.

Classification: SHARED. pdflatex 0 errors (examplea.pdf ships clean); Perl 0.8.x = 7 errors
  on the minimal (`\lx@tag Attempt to end mode restricted_horizontal`, same family); Rust = 10.
  In scope (surpass-Perl approved; pdflatex is the clean oracle).

Fix site: latexml_engine/src/base_utilities.rs `predigest_box_contents_in_mode` (~:3542/:3576)
  + latexml_engine/src/tex_box.rs HBoxContents `DefParameterType!` (:626-629, add `reversion`).
Guard (guard test on restricted_hbox_uline_examdesign.tex): 0 Error/Fatal lines AND the
  truefalse answer-key list renders — assert ≥1 `<ltx:item>` containing an `<ltx:text>`
  (the underlined boxed answer). Re-verify the pure-kernel Root-A guards
  (restricted_hbox_mbox_bgroup, grammar_newcommand) stay GREEN.
Risk: MED — the one-frame switch touches EVERY `\hbox`/`\mbox`/`\makebox`/`\framebox`/`\fbox`
  and math sub-box digestion. Re-convert the two-frame model's arXiv witnesses before landing:
  the p{}-cell/`\vtop` sizing witness 2210.13325 (vertical branch, should be unaffected) and
  any hbox-sizing goldens; watch `\hbox{a}` reversion (the new reversion closure).
Expected corpus gain: examdesign examplea/b/c → 0 (3 docs, ~303 capped error-lines). Other
  restricted_horizontal box-in-macro docs share this reader; modernposter/hyperbar are a
  DIFFERENT (`\begingroup`-straddle) root, not fixed by (a).

Dead ends:
- `\hbox\bgroup\bgroup … \egroup\egroup` split across macros, with/without `\makebox`
  (p1/p2/p3), and `\uline{\makebox…}`, `\item[\uline{\makebox}]`, `\sbox{\uline{\makebox}}`
  (r4/r10/r14/r15): all CLEAN — the two-frame path only desyncs when the closing `\egroup`s
  are split across macros AND a bounded constructor's isolated arg-digest sits between the
  frames (the full examdesign truefalse-KEY `\item` label path); could not shrink below the
  class doc.
