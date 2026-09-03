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
| 5 | No coherent engine persona (Unicode mouth, pdfTeX primitives, `\pdfoutput=0`) | P16-vii/xiii, neoschool-fr, l2tabu, every `\ifnum\pdfoutput` doc | **needs user approval** |
| 6 | File loading bypasses `\@onefilewithoptions`; file I/O not a VFS | P19, P16-xii, expl3 file-boundary state; VFS queue #1 | half-landed (b42/b47/b50) |
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
(PLANS "Approach revision", 25 raw-blocking stubs). Risk per seam LOW–MED;
each seam needs its arXiv counter-witness re-converted (P38: 0802.2207).

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
`\escapechar`-aware); audit `to_string()`→`Tokenize!` pairs.

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

## Not architectural (recorded so it is not re-litigated)

- **Throughput.** P59 (tikzlings 444 M tokens, no loop), tikzpingus,
  glossaries-user's ~706 tcolorbox frames at ≈0.45 M tokens each,
  schulmathematik's timeout: pgf draws frames by macro expansion and we
  run ~10–100× slower than TeX on that. The lever is a native pgfsys SVG
  scope protocol (`pgfsys_latexml_def.rs`) and gullet throughput, not the
  token model; settled perf dead-ends in the memory index apply.
- **Binding arity / `\newif` / `\let` gaps** (P20–P22, P24, P33, P43,
  wave-4 cahierprof/glossariesbegin): long tail, fixed as found.
- **Math parse shape**: Marpa vs Parse::RecDescent, by design
  (OXIDIZED_DESIGN).

## Ordering recommendation

1. Theme 5 (persona) — smallest code, corpus-wide, needs approval.
2. Theme 2 + 4a as standing policy on every batch (already in force from
   batch 54: seam over surface, Tokens over strings).
3. Theme 1 (R9) — the decision brief is this section 1; nothing else
   unlocks the exemplar or the mdframed/tcolorbox mass.
4. Theme 3 (`\halign`) — after 1; retires most table bindings.
5. Theme 6 (loader at `\@@input`, VFS completion) — independent, MED.
