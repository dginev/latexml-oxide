# Perfect Kernel — architectural themes behind the root causes

Design brief distilled from the ~60 root causes in [`PLANS.md`](PLANS.md)
(P1–P59, batches 33–53) and the wave-3/4 root-causer reports
(2026-09-01/02). Written on the user's request (2026-09-02) as the
standing input to the R9 mode-frame decision and to any re-architecture
of the kernel. The long tail of one-line arity gaps, missing `\newif`s
and `\let`s is *not* here — that is ordinary binding hygiene. What is
here is the set of mechanisms that recur **regardless of package**, each
with its tex.web / latex.ltx model, the Rust sites, the witnesses, and a
fix shape.

**Standing practice (user, 2026-09-26):** record an architectural insight here as soon as it is met; once a
large goal completes, revisit the open themes and implement them in dedicated sessions. Themes 7–10 and 2b
were recorded from batches 56jm–56kc and sweep #126.

**2026-09-05:** the user approved a generalized kernel-capability program built on these themes — landing plans, abstractions and order live in [`KERNEL_CAPABILITIES.md`](KERNEL_CAPABILITIES.md) (K2 = theme 1, K3/K7 = theme 6, K5 = theme 4, K6 = theme 5).

Ranking is by corpus mass capped, not by ease. Themes 1 and 3 are the two
that move the curve by tens of points; 1 is a precondition for much of
3's benefit; 2 and 4 are pursued as *policy* on every fix; 5 is one
approval away.

| # | Theme | Mass (witness docs) | Status |
|---|---|---|---|
| 1 | Grouping and mode are one stack in the Stomach; TeX has two | ≈50+ docs / ~1050 lines (mode-frame study), plus every list/box clone | **USER-PARKED (R9)** — decision brief |
| 2 | Constructors bind at the user macro, not at the latex.ltx seam | P22, P27, P30, P38, P48, P52, P58, P16-vi/xii | policy + queue |
| 3 | No `\halign`; alignment intercepts `&`/`\\` at constructor level | nicematrix ×8 plans, tabularray, tabu, longtable/xltabular, aguplus, bibleref-parse, memman | queue (largest unparked lever) |
| 4 | Token stream ≠ TeX's: string round-trips lose catcodes; isolated mouths invent EOFs | P3, P8, P15, P18, P29, P50, P53; tagpdf, hobby, swfigure, stex-doc | policy + queue #4 |
| 5 | No coherent engine persona (Unicode mouth, pdfTeX primitives, `\pdfoutput=0`) | P16-vii/xiii, neoschool-fr, l2tabu, every `\ifnum\pdfoutput` doc | PDF-mode persona ruled 2026-09-24 and LANDED (56id, K6); the Unicode-engine font model open |
| 6 | File loading bypasses `\@onefilewithoptions`; file I/O not a VFS | P19, P16-xii, expl3 file-boundary state; VFS queue #1 | half-landed (b42/b47/b50) |
| 7 | Typed parameters are claims about how TeX reads; each binding can disagree with the real macro | 56jm, 56jp, 56jr, 56ju, 56jw (4 roots), 56jx, 56jz; sweep #126: tkz-grapheur, bxcalc, PixelArtTikz | in progress: K13 stage 0 landed (57b) |
| 8 | The horizontal list is not represented: glue becomes text, so `\unskip`/`\lastskip`/trims guess | 56jy, babel-french `;`, the paragraph text-node split | **open** (2026-09-26) |
| 9 | Bibliography formatting is tables, not the style's programs | 56ii, 56jt, 56kc; abntex2cite; biblatex-chicago/apa samples | **open** (2026-09-26) |
| 10 | Process: the regression net sees arXiv, not the manuals | 56jr, 56js regressions found five batches late; 56jo `tex=` loss | LANDED (57a, K17: `manual_net.sh`) |
| 11 | A box's size is its rendered attribute: typed sizes are stored as strings and ignored | 56kf side finding (bxcalc); `box_dimensions_measured.tex` | step 1 landed (56kj, K18); `\height` binding 56kl; rest of step 2 open |
| — | Throughput on macro-generated volume (pgf drawing) | P59, tikzpingus, glossaries-user, schulmathematik | perf lane, not structure |

## 1. Grouping and mode are one stack; TeX keeps two

**TeX model.** tex.web keeps the *save stack* (§268–284, `\begingroup`,
`{`, `\bgroup`) and the *semantic nest* (§211–219, `mode`, `push_nest`/
`pop_nest`) as independent stacks. `\hbox\bgroup…\egroup` pushes both
(§1083 `begin_box` → `push_nest` + `new_save_level`, §1086);
`\begingroup…\endgroup` and `{…}` push only the save stack; a mode change
never fails because of where a group boundary falls. Real packages rely
on this constantly: mdframed/tcolorbox open the box in one macro and
close it in another, `\list` clones open a `\trivlist` group and close it
via `\endtrivlist` (latex.ltx:15871/15912), fancyvrb's `\VerbatimFootnotes`
closes the footnote box with `\aftergroup` (fancyvrb.sty:33–58).

**LaTeXML model (inherited from Perl).** `begin_mode` pushes a stack frame
*and* binds `BOUND_MODE` in it (`latexml_core/src/stomach.rs:951–1001`,
Perl Stomach.pm:474–517); `end_mode` errors "Attempt to end mode …" unless
the *top* frame is the one that bound the mode (`stomach.rs:1008–1060`,
Stomach.pm:522–541). Every `\egroup`/`}` that lands on a mode frame, and
every `end_mode` that lands on a plain group, is an error — that is the
R9 family.

**Evidence.** The mode-frame study (LEDGER #18, PLANS P42): 38 oracle-clean
docs, ~1050 lines, 100 % SHARED with Perl. Wave 4 (2026-09-02): cnltx_en,
chemformula-manual and endiagram_en each cap at 1001 errors of which
~900 are `\endmdframed` "Attempt to end mode internal_vertical";
schulmathematik 96×. Same mechanism under other names: P36 (inline verb in
`\footnote`), P38 (`\@trivlist` neutered to `\relax` because a shared
opener would need a shared closer), P48 (`\@tabarray` bare), P52
(`\VerbatimFootnotes` cannot swap the closer), P56a (`\widthof` box
closing the outer `$`), mhchem `\ce` in `align*`, P58 (`\endlx@list`
boxing group).

**Fix shape (design, not landed).** Separate the two stacks: mode entry
pushes a *nest* record (mode, element-open depth, font) and does not
require a frame; `end_mode` pops the nest record whose mode matches,
independently of frame depth; groups keep the save stack only. Element
pairing (the XML side) rides on the nest record, which is what
`maybeCloseElement` already tolerates. Risk MED-HIGH: touches every
`begin_mode` site; the arXiv canvas witnesses for the current model
(1112.6246 halign frame balance, 0802.2207 `mathtrivlist`) must be
re-converted. **Entry needs the user's go** (directive 2026-07/08, R9);
granted 2026-09-02 ("all queued surpass shapes + R9 approved").

**Correction (2026-09-03) — what the two-stack model must NOT do.** A
wave-12 design pass proposed making `$` close inline math "only when the
math frame is the current group", on the claim that `X ${$b$}$ Y` and
`{$b$}` inside an `align*` cell are RUST-ONLY failures. Both claims were
wrong: the agent's Perl runs never executed (empty stderr files read as
"0 errors"), and same-host Perl 0.8.8 emits the same two
`Attempt to end mode math` errors. tex.web agrees: §1065 (`mmode +
math_shift: if cur_group = math_shift_group then after_math else
off_save`) makes a `$` under a simple/semi-simple group an error
("Missing } inserted", §1064 recovery inserts the closer). So tex.web
keeps two stacks *and* still rejects a mode close across a group
boundary — the nest is separate from the save stack, but `after_math`
is gated on `cur_group`. The two-stack design therefore only changes the
cases where TeX itself pushes both stacks together — `\hbox\bgroup` /
`\vbox\bgroup` / `$` (§1083 `begin_box` → `push_nest` + `new_save_level`;
§1139 `init_math`) — and where LaTeXML today opens the box in one macro
and closes it in another; a `$` or `\egroup` meeting the wrong group
stays an error in both models, with §1064's insert-the-closer recovery
as the surpass-grade improvement over Perl's "don't pop". Any witness
proposed for this theme must be re-verified against Perl **with the
same preload** and against pdflatex's log, and its stderr must contain
`Conversion complete:` to count as a run.

## 2. Constructors bind at the user-level macro, not at the latex.ltx seam

**LaTeX model.** Classes and packages redefine the *user* macros by
wrapping the *internal* hook points: `\list` → `\@trivlist`
(latex.ltx:15848/15871), `\footnote` → `\@footnotetext` (:17658),
`\section` → `\@sect` (:17247), `\caption` → `\@makecaption`,
`tabular` → `\@array` (:16564). memoir.cls:4580 `\renewcommand*{\list}`,
fancyvrb `\let\@footnotetext\V@footnotetext`, tudapub.cls's
`\AddToHook{class/scrbook/after}` are all wrapping, not replacing.

**LaTeXML model.** Perl never loaded latex.ltx, so it re-implemented the
*user* macros as constructors and left the internals unbound or
neutered. Our dump *does* load latex.ltx, but the constructors still
attach at the user level: `\list` opens `\lx@list`'s bgroup and only
`\endlist`=`\endlx@list` can close it (`latex_constructs.rs:5918–5924`,
P58); `\footnote` is locked (P52); OmniBus pre-binds `\begin{example}`
ahead of a document `\newenvironment` (P30, `omnibus_cls.rs:538–557`);
stubs hide whole raw classes (P27 memoir, P13 curve2e, P9 atableau);
`\@enumctr` was never set because `beginItemize` set only `\@listctr`
(P22).

**Fix shape (policy, incremental).** Attach the XML construction to the
internal seams with the existing `\lx@*` idiom and let the real latex.ltx
user macros run above them: `\@trivlist` = the shared opener,
`\endtrivlist` = the shared closer (P38); `\@footnotetext` = the note
constructor with `\footnote` as the real `\@footnotemark`/`\@footnotetext`
macro (P52); `\@array` = the alignment opener (P48, theme 3);
`\@makecaption` = the caption constructor. Every new fix chooses the seam
over the surface; every stub gets the delete-if-raw-loads-clean audit
(PLANS "Approach revision", now in `archive/PLANS_DONE_PHASE56_2026-09-27.md`; 25 raw-blocking stubs). Risk per seam LOW–MED;
each seam needs its arXiv counter-witness re-converted (P38: 0802.2207).

### 2b. Replacing a macro drops its mode transitions

**LaTeX model.** A user macro's first tokens often change mode: `\leavevmode` in `\mbox`
(latex.ltx:16082), `\makebox` (16077-16078), `\fbox` (16182-16183), `\raisebox` (16373-16374),
`\rule` (16361), `\parbox` (16250), `minipage` (16306), `\@array` for `tabular` (16560), color.sty's
`\textcolor` (104) and `\color@b@x` (163-164); `\par`, `\@bsphack` elsewhere. `\everypar` fires at
that point, before the box.

**LaTeXML model.** A constructor that replaces the macro restates the transition in its options
(`mode =>`, `enter_horizontal`, `leave_horizontal`), per binding, and the restatement drifts: batch 54n's
change from `mode => "text"` to `"restricted_horizontal"` silently dropped `\makebox`/`\raisebox`'s
enterHorizontal, and Perl never had it on `\mbox`/`\@framebox`/`\colorbox` (space after a box at a
paragraph start lost; fixed 56kb, DIVERGENCES #323). Still open: `\fcolorbox`'s `internal_vertical` body
ends the running paragraph (SHARED; repro `boxes-groups/fcolorbox_splits_paragraph.tex`); a block box
(`\rule`, `tabular`) at a paragraph start splits the paragraph (SHARED; `\parbox` fixed 57d, `minipage`
57f, DIVERGENCES #338);
`\trivlist`'s `\item` rebinding dies with its own mode block (PERL-ORIGIN; the fix restructures 101
sweep docs, needs a ruling; repro in `~/data/pk_agents/w70/scratch-streamA126/structural/repros/`).
The same drift happens inside the port, between a definition form and the Perl semantics it
compiles: `DefPrimitive!` with a literal body compiled to a closure returning the box, so
Primitive.pm:73's `enterHorizontal` for a string body never ran for ~600 glyph primitives (`\dag`,
`\copyright`, `\cent`, wasysym; a space after `\dag{}` lost; fixed 57k, the literal is now the string
body `invoke_primitive` already handles). A definition form's compiled shape must keep its Perl
`invoke` semantics — a closure standing for a string, token or undef body loses them.

**Fix shape.** Theme 7's conformance detector also records each replaced macro's prologue
(`\leavevmode`/`\par`/`\@bsphack`) and checks the constructor's options against it; longer term the
transition comes from the macro at the seam (policy above), not from a restated option.

## 3. Alignment IS a faithful `\halign`; the gaps are the width pass, theme 1, and package internals

*Rewritten 2026-09-03 after a design investigation (the earlier text claimed
"no `\halign`, no template insertion, no `\@sharp` semantics" — wrong).*

**TeX model.** `\halign` (tex.web §768–812: `init_align` 15327 pushes an
align_group on the save stack AND a nest level; `init_row`/`init_span`/
`init_col` insert the u-part; the alignment tab and `\cr` are recognised
purely by `align_state` and fire `fin_col` (v-part via `\endtemplate`) and
`fin_row`; `fin_align` 15743 runs the two-pass column-width computation over
unset nodes).

**LaTeXML model (Perl = Rust, line-faithful port).** `TeX_Tables.pool.ltxml:164`
/ `tex_tables.rs:278` implement `\halign` with the REAL `#` preamble
(`parseHAlignTemplate` → `parse_halign_template`, `tex_tables.rs:1498`: u/v
split at `CC_PARAM`, `\tabskip`, `\span`, repeated columns on a leading `&`),
and the §309 alignment-tab-as-scanner-event: `Gullet::readToken` classifies
`&`/`\cr`/`\crcr`/`\span` when `ALIGN_STATE == 0` (`Gullet.pm:266-278` →
`gullet.rs:3341`), `handleTemplate` inserts the v-part and a `before-column`
marker resets the state (`gullet.rs:3366`). Raw `\halign{#\hfil&\hfil#\cr…}`
and a tikz `matrix of nodes` convert with 0 errors in both engines; `\valign`
already beats Perl. The `Alignment` object (`alignment.rs:108`) with its
`Template`/`Cell` (u/v token parts, align, tabskip, colspan) is what the
LaTeX `tabular`/`array` bindings sit on (`DefColumnType` → `\NC@rewrite@<c>`,
`read_alignment_template` `alignment.rs:951`).

**The real mismatches.** (1) **No `fin_align` width pass** — cells digest
straight to boxes; `normalizeAlignment` guesses widths, so width-driven
columns (tabularx `X`, longtable auto-measure) cannot come from the kernel
and live in per-package bindings (tabu, tabularx, longtable, xltabular,
tabularray, supertabular, tabulary all exist and pass). (2) **One stack, not
two (theme 1)** — the alignment runs eagerly under a single `bgroup` +
`begin_mode`, so pgf's `\hbox\bgroup\vbox\bgroup\halign\bgroup…\egroup
\egroup\egroup` (pgf matrix: tex-font-cheatsheet) and tabular-in-box collide
with the frame model. (3) **Eager body read** — a row-boundary command in an
unexpected position (`\hline\newpage\hline`, harmony) desyncs the leading-row
scan (`tex_tables.rs:958-1002`) instead of being absorbed by the main loop
(SHARED: Perl 3 errors, Rust 1). (4) `\span`/`omit_template` splicing is
simplified (`\lx@alignment@multicolumn`, `tex_tables.rs:586`).

**Fix shape.** Not a second `\halign` engine (it would duplicate working
code and touch none of the roots). In order: (i) binding hygiene — the only
real hole is `ltxtable` (`\LTXtable{width}{file}`: no binding in Perl or Rust;
raw ltxtable reaches `\TX@col@width`/`\TX@target`/`\LT@echunk`/`\LT@get@widths`,
none of which exist — tikzcodeblocks, vhistory ~30-error cascade); (ii) theme 1
for the box-nested `\halign` family; (iii) optionally skip `\newpage`-family
marks in the leading-row scan (SHARED, low payoff); (iv) DEFER the width pass —
only the width-driven columns need it and their bindings already provide it.
Running array/tabularx/longtable RAW is not worth it: `\@array`/`\@mkpream`/
`\@classz` are reimplemented via `DefColumnType`, and the raw path would need
the whole `\TX@*`/`\LT@*` surface plus the width pass. Guard corpus:
`latexml_oxide/tests/alignment/` (32 pairs, swept by `53_alignment.rs` and
`114_streaming_alignment.rs`), `87_trip::halign_body_implicit_cr`, the
`cluster_package_guards` tabular tests. Repros:
`~/data/pk_agents/w12/halign/`.

## 4. The token stream is not TeX's

**(a) String round-trips lose catcodes.** tex.web has no string→token
boundary inside the engine; `\detokenize`/`\meaning`/`\string` produce
OTHER (+ SPACE) tokens with `\escapechar` (§1594 print_esc). We
stringify and re-tokenize at many sites: `\scantokens`/`writable_tokens`
hard-coded `\` (P3), `\DeclareMathOperator` re-tokenized under sty
catcodes (P18, dialect.rs:478), `\index` never sanitized (P29),
`\filename@parse` re-lettered its argument (P50), `\@currenvir` one
multi-char token (P53, dialect.rs:1193), the `#`-PARAM storms
(cnltx/endiagram/memman `\@sharp`). Policy: carry `Tokens` through; where
a string is unavoidable, re-enter with `\detokenize` semantics (all OTHER,
`\escapechar`-aware); audit `to_string()`→`Tokenize!` pairs. Evidence 2026-09-26: an `\index`/`\glossary` entry's display is re-tokenized
with the internal catcodes and digested *at the mark* (`SanitizedVerbatim`, mod.rs `process_index_phrases`),
where TeX only writes it to the `.idx`/`.glo` and re-reads it at `\printindex`; so robustglossary's
`formula&explanation` raises "Stray alignment" at the mark (robustsample, sweep #126), and a fix that
inspects the phrase after `do_expand_partially` finds `\begin`/`\(` already expanded away (56kd review).

**(b) Isolated mouths invent EOFs.** In tex.web only a *file* end is an
EOF (§362); token lists and backed-up levels are transparent, so a
delimited argument (`\def\foo#1\relax`) or an `\if…\fi` can start in a
macro body and finish in the file. Our per-argument and per-file mouths
end early: P8 (post-undefined `Until:` loops), P15 (`\everyeof` wiring),
tagpdf `\prg_break_point:Nn`, hobby `Until:\relax`, swfigure `Until:@`,
stex-doc's 508 misses (wave 3/4). The architectural queue item #4 already
names the model: fewer isolated mouths, delimited scans that cross
token-list mouths and stop only at file mouths, `\everyeof` inserted once
per *file* (b51 landed the file side). Risk MED-HIGH (P15's spath3/
litetable/zref 5-token loops were the crossing-order bug; bounded crossing
fixed it).

## 5. No coherent engine persona

The mouth yields one token per Unicode codepoint (XeTeX/LuaTeX-like) but
the primitive surface is pdfTeX's: no `\Umathcode` family, so
`\sys_if_engine_opentype:TF` is false (expl3-code.tex:7864–7865 tests
`\tex_Umathcode:D`, :1121) and l3text's `\__text_codepoint_process:nN`
reads `é` as a UTF-8 lead byte and dies at `\q__text_recursion_stop`
(neoschool-fr, P16-xiii; SHARED, Perl 101 errors); `\pdfoutput=0` sends
every `\ifnum\pdfoutput=…` doc down the DVI branch while the pdflatex
oracle takes PDF (l2tabu, P16-vii); LuaTeX probes (`\directlua`,
`\luatexversion`, `\csstring`) are forbidden by directive
(LUA_REBINDING.md) because defining them makes packages take the Lua
path. Each symptom is currently filed as a separate "expl3 bug".

**Decision needed.** Assert one persona *before* expl3 loads in the dump
build (latex.rs INI_MODE ~L84–125): the Unicode-engine character surface
(`\Umathcode`/`\Umathchardef`/`\Uchar` family, `latex_constructs_rust_only.rs:117–124`
moved earlier), `\pdfoutput=1`, and the pdfTeX primitive set otherwise —
XeTeX-like tokenization without XeTeX's font loading and without
`\XeTeXversion` (fontspec must still see no OpenType engine). Perl's
persona differs, so this is a P16 approval item; the arXiv risk is the
`\ifnum\pdfoutput` graphics-extension branches and the encoding probes at
latex.ltx:9437/14453/14662/15463 (greek_test LGR guard).

## 6. File loading and file I/O bypass the kernel

`\usepackage`/`\documentclass`/`\RequirePackage` run a Rust-side path
that bypasses `\@onefilewithoptions` (latex.ltx:18740) and
`\@fileswith@ptions` (:18709): the `package/<name>/after` and
`file/<name>/after` hooks never fire (P16-xii, DEMO-TUDaPhD `\@addchap`),
`\@pushfilename` (:18363)'s expl3 boundary state is emulated by flags
(architectural queue #5), option lists were pushed as one nested string
(P19). Write-out/read-back is four ad-hoc capture scanners over a
de-facto VFS (queue #1; b42 tee, b47 empty-file existence, b50 `\relax`
first line landed). Same principle as theme 2: let latex.ltx's loader run
and intercept only at `\@@input` (binding lookup: substitute a Rust/ltxml
binding for the file when one exists, else raw-input), and make every
`\openout`/`\write`/`\closeout` land in one virtual store that every
`\input`/`\openin`/`\IfFileExists` consults first.

Instance landed (56kv, DIVERGENCES #334): the Rust loader pushed every class load's options onto the
global options, where `\@fileswith@pti@ns` sets `\@classoptionslist` only at the first class load
(latex.ltx:18716-18718). A class-load depth now separates the document class from the classes it loads. The
same loader still overwrites `document_class_filename` on nested loads (SYNC_STATUS side findings).
Letting latex.ltx's own loader run would make both rules hold by construction.

## 7. Typed parameters are claims about how TeX reads

**TeX model.** A LaTeX command is a macro: its arguments are token lists read by its `\def` parameter
text (an undelimited `#1` is one token or a balanced group; a delimited one runs to its delimiter) or by
the kernel's peeks (`\@ifnextchar`/`\@testopt`, a `\futurelet` with no expansion). Only primitives scan
with expansion (`\hskip`, a `\dimen` assignment), from the live stream, after the macro has run.

**LaTeXML model.** A binding declares typed parameters (`Dimension`, `Number`, `Semiverbatim`, `[]`,
`Undigested`, `HBoxContents`, `AlignmentTemplate`), each a reader that scans the live gullet itself, and
each a claim that can disagree with the real macro:
- reads too much: a column type's argument read past the template's `}` into the document (56jz,
  KPE #292); a number scan ran through `\@ifnextchar` (56jm); binding closures that define ran inside a
  look-ahead (56jw D, KPE #286);
- reads too little: a braced argument's rest was dropped (56jr/56ju, DIVERGENCES #317: multido,
  nicematrix `X[..]`, tabularray, `\resizebox` lengths);
- expands at the wrong moment: `\hyperref@@iv`'s fourth argument expanded `\textbf` into
  `\edef\f@series` (56jw A); the case changer (56jw C);
- has the wrong shape: arydshln's `\hdashline[..]` and `;{..}` (56jx, KPE #291); listings' Invocation
  reverted against cleveref's `\label[]` (56jw B).
The rest policy is itself per reader: 56jr's fix produced three sweep-#126 regressions (tkz-grapheur
Fatal "infinite digestion loop", bxcalc 13 calc errors, PixelArtTikz 2). Root-caused: bxcalc's is this
theme exactly — 56jr's braced-length evaluator calls calc's expression reader directly, where latex.ltx
passes the length through whatever `\setlength` currently means (`\@hspace` 9425, `\@vspace@calcify` 9254;
bxcalcux.sty:290-293 and pgf's `\pgf@setlength` redefine it), so `\hspace{6tm}` lost the custom unit;
PixelArtTikz's is a stub reader that takes `*{\count}` only as a literal integer (tabularray evaluates an
integer expression, tabularray.sty:3361). Both landed at their one seam, not by the fix shape below:
56ke evaluates the count as `\numexpr`; 56kf digests `\setlength<scratch>{<argument>}` when `\setlength`
is a macro, the typed reader stepping aside for the real one (DIVERGENCES #325). 56kf's first cut is
this theme's cost in one line: it read every braced `{Dimension}` as a `\setlength` operand, a claim true
of latex.ltx's commands and false of `\pgfsetlinewidth` (pgfmath), and the arXiv A/B caught two papers
recursing to the pushback limit in a nested `\pgfpicture`; the operand is now a declared type
(`SetlengthDimension`), the claim made explicit per command. The same side finding
shows the claim reaching past reading: a box's typed width is right in its XML attribute but not in its
measured `\wd`/`\ht` (`\framebox[w]`, `\raisebox`, `\resizebox`; SYNC_STATUS).

**Fix shape.** (a) A binding-conformance detector: load each package raw in a scratch state, record every
public macro's real signature (parameter text and delimiters, `\newcommand` arity and optional default,
xparse spec) and body prologue (theme 2b), diff against the binding's declared parameters and options,
and rank the mismatches by corpus usage. Static, no conversions; turns the "arity long tail" below into
one worklist. (b) Two-phase typed parameters for macro bindings: read the argument token list exactly as
the real macro does, then parse the typed value from it in an isolated mouth under one central rest
policy (56jr's ArgumentTails, generalized); only primitive bindings scan live. Risk MED–HIGH for (b);
start where (a) finds the most mass.

## 8. The horizontal list is not represented

**TeX model.** Horizontal mode builds a list of typed items — characters, glue (inter-word space
included), kerns, penalties, boxes. `\unskip`/`\unkern`/`\unpenalty` remove the last item of their kind
(tex.web §1105), `\lastskip`/`\lastkern`/`\lastpenalty` read it (§424). LaTeX's spacing idioms depend on
it: `\@bsphack`/`\@esphack`, `\removelastskip`, `\@finalstrut`'s `\unskip`, babel-french's high
punctuation (`\unskip` before the thin space), `\xspace`.

**LaTeXML model.** A space becomes a character in a DOM text node at digestion; afterwards glue has no
type, and every tail operation guesses from text and from how the DOM happened to split it. The
paragraph/cell trim (56jy) is Perl's `s/\s+$//` on the last text node, and Perl's appendChild/
appendTextNode split text differently from libxml2's merging add_child (`<p>U+2006</p>` vs `<p/>`; RED
repro `block-model/paragraph_trim_text_node_split.tex`); babel-french `;` keeps the space before it
(RED repro `babel-lang/french_highpunct_unskips_space.tex`).

The same gap misleads the stomach's loop guard (`cycle_guard_record`, stomach.rs): content-free items —
the stray space of pgflibraryplothandlers.code.tex:59, empty brace-group lists from `\pgf@process` — are
boxes like any other, so a finite `plot[smooth]` of 11,000 points reads as a 5-box cycle and is a Fatal
(tkz-grapheur-doc, sweep #126; TeX accumulates them as glue under `\nullfont`, pgfcorescopes.code.tex:243).
Typed items would let the guard fingerprint content only.

**Fix shape.** Keep a typed tail of the current horizontal list in the stomach (the last item's kind and
amount: glue, kern, penalty, char, box) so `\unskip`, `\lastskip`, `\ifdim\lastskip`,
`\removelastskip` act on it; turn glue into spaces only at the XML boundary, under one trim rule that
replaces `trim_node_right_whitespace` and the text-node dependence. Risk MED; whitespace golden churn.

## 9. Bibliography formatting is tables, not the style's programs

**Model.** BibTeX runs the `.bst` stack program to write the `.bbl` pdflatex typesets; biblatex's
`.bbx`/`.cbx` drivers typeset biber's `.bbl` data, with type aliases, bibstrings (`.lbx`), date
formats, `related` entries and per-style fields.

**LaTeXML model.** MakeBibliography's FMT_SPEC tables (Perl) approximate all of it, so every style
feature is a formatter change with golden churn: URLs, translators, annotations (56ii); field separators
(56jt); type aliases (56kc); and, still open, editor roles, `related` entries, long dates, style-specific
types, notes-style footcites (root-caused 2026-09-26, `~/data/pk_agents/w70/scratch-streamA126/
biblatexstyles/`). A `.bib` with a `.bst` and no `.bbl` is a deferred family (DEFERRED_FAMILIES):
abntex2cite reads 80.5 %, 99.4 % with the `.bbl` bibtex writes.

**Fix shape.** (a) BibTeX half: a native `.bst` interpreter (the stack VM with its ~40 built-ins;
btxdoc/btxhak), writing the `.bbl` our reader already digests — pdflatex's exact reference text for any
`.bst`. (b) biblatex half: drive the formatter from the style's own declarations
(`\DeclareBibliographyAlias` — 56kc's hook is the pattern — `.lbx` bibstrings, `\DeclareFieldFormat`)
rather than hard-coded rows. Running biblatex's drivers raw over the `.bbl` would be exact but loses the
per-field markup: a ruling.

## 10. Process: the regression net sees arXiv, not the manuals

**Evidence.** 56jr passed the suite and the 3,003-paper arXiv A/B of its train; its manual-only
regressions (tkz-grapheur Fatal, bxcalc +13, PixelArtTikz +2) and 56js's (robustsample, ryesample)
surfaced five batches later, in sweep #126. Identical A/B tallies also hid 56jo's `tex=` loss in 153
papers (now fingerprinted).

**Fix shape.** Per batch, beside the arXiv A/B: a fixed, stratified manual subset (~200 manuals chosen
for package diversity; ~15 min on 64 cores) and the whole repro catalog (`repros.sh`), each against the
previous binary with byte-diff classification. The gate ladder's L2 exists but is grep-selected per fix;
a fixed stratified set catches lateral regressions at their batch.

## 11. A box's size is its rendered attribute

**TeX model.** A box's width, height and depth are integer fields (tex.web §135). `\wd`/`\ht`/`\dp` read and assign them (§420, §1247), and latex.ltx builds each box to its size: `\@iframebox` puts the frame inside `\hb@xt@#1` (latex.ltx:16196-16229), and `\@irsbox` raises and then sets `\ht`/`\dp` (16378-16393).

**LaTeXML model.** A whatsit carries typed requested sizes and a sizer (Box.pm:260-299). The size code (`get_size`, latexml_core/src/lib.rs:443-517) accepts only a typed `Dimension`/`Glue`. The Rust bindings break this in four ways; the root-causer's report (2026-09-26, `~/.claude/jobs/4a65d6f9/tmp/rc_boxdim/`) covers each:
- **(a) Size rendered to an attribute string.** Properties are stored as attribute *strings* at digestion, and the size code silently ignores them. `\framebox[1em]` measures 12.08pt, where pdflatex gives 10.0pt. `\resizebox`, `\scalebox`, `\rotatebox` and `\reflectbox` fall to the default sizer, which sums every argument as text: `\resizebox{1em}` measures 49.7pt. Also rotate, makecell and diagbox.
- **(b) Lossy round trip.** The 0.1pt string is parsed back: `\parbox{5em}` measures 50.0pt, not 50.00008pt. The `panel_width` fallback works around the same loss.
- **(c) Missing geometry in the sizer.** The box's own geometry is absent from its sizer: `\raisebox`'s raise is not counted, and Perl's `raisedSizer` is left commented out. The audit to run: 51 `sizer =>` sites against 170 constructors.
- **(d) Requested width leaks into line breaking.** The requested width leaks into the contents' line breaking: `\makebox[1em]{aaa bbb ccc ddd}` measures 4.3pt high and 36pt deep, where TeX gives 6.9pt and 0pt (`font.rs:1603-1611`, `list.rs:132-136`).

Only `\framebox[w]` is shared with Perl, and Perl prints an object address as the width there.

**Fix shape.** Store typed Dimensions only, and render them with `Stored::to_attribute` when the XML is written; the attributes stay byte-identical. Give each box constructor a sizer with its own geometry. The cheap detector: log from `compute_size_and_cache` (lib.rs:528) whenever a size key holds a `Stored::String`. This is a sibling of theme 8, which likewise turns glue into text at digestion.

**Evidence.** The side finding of 56kf, RED repro `boxes-groups/box_dimensions_measured.tex` (SYNC_STATUS). The effect shows as geometry fidelity: `\settowidth`, SVG and picture sizes, scaled boxes inside boxes. It rarely shows as `Error:` lines, except where code divides by a measured size (the bfhsciposter `\rule` precedent).

**Status.** (a)-(d) landed for `\framebox[w]`, `\parbox`, `\raisebox`, the graphics boxes and the width leak in 56kj (K18 step 1; rotating.sty shares `rotated_properties`). `\height` etc. bound to the box while the size arguments are read landed in 56kl (`TempboxaDimension`, `within_tempboxa`). Open: `\raisebox`'s raise, held back until `yoffset` renders with reserved space (a true lowered icon otherwise overflows into the next row: the XML being right is not enough when the renderer draws it without the room TeX gives it); `\resizebox{\width}`/`\resizebox*`; makecell and diagbox; `\Gscale@div`'s arithmetic; the detector and the sizer audit (K18 step 2).

## Not architectural (recorded so it is not re-litigated)

- **Throughput.** P59 (tikzlings 444 M tokens, no loop), tikzpingus,
  glossaries-user's ~706 tcolorbox frames at ≈0.45 M tokens each,
  schulmathematik's timeout: pgf draws frames by macro expansion and we
  run ~10–100× slower than TeX on that. The lever is a native pgfsys SVG
  scope protocol (`pgfsys_latexml_def.rs`) and gullet throughput, not the
  token model; settled perf dead-ends in the memory index apply.
- **Binding arity / `\newif` / `\let` gaps** (P20–P22, P24, P33, P43,
  wave-4 cahierprof/glossariesbegin): long tail, fixed as found — until theme 7's conformance
  detector turns it into one ranked worklist.
- **Math parse shape**: Marpa vs Parse::RecDescent, by design
  (OXIDIZED_DESIGN).

## Ordering recommendation

Themes 7–10 and 2b (recorded 2026-09-26) have landing plans as KERNEL_CAPABILITIES K13–K17 and are
implemented in dedicated sessions after the current large goal (standing practice above): K17 (theme 10)
is the cheapest; K13 (themes 7 and 2b) comes first among the kernel ones and sizes K14.

1. Theme 5 (persona) — smallest code, corpus-wide, needs approval.
2. Theme 2 + 4a as standing policy on every batch (already in force from
   batch 54: seam over surface, Tokens over strings).
3. Theme 1 (R9) — the decision brief is this section 1; nothing else
   unlocks the exemplar or the mdframed/tcolorbox mass.
4. Theme 3 (`\halign`) — after 1; retires most table bindings.
5. Theme 6 (loader at `\@@input`, VFS completion) — independent, MED.
