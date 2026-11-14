# Perfect Kernel — PLANS rows and items closed 2026-10-02

Frozen 2026-10-02 from `PLANS.md` (living): the table rows whose plans landed or whose witnesses are clean or out of
scope at sweep #134, and the queue items replaced by one-paragraph summaries there. P-numbers and item numbers are
cited from code (`PLANS P16 ii` sect09.rs, `P16 xii` content.rs, `P65` codehigh_sty.rs, `PLANS 11`
112_fragment_yield.rs, `13(f)` colt2024_cls.rs) — keep them. Phase-56 closures:
[`PLANS_DONE_PHASE56_2026-09-27.md`](PLANS_DONE_PHASE56_2026-09-27.md).

## Verdicts

| P | verdict (2026-10-02, against sweep #134 and the tree at 59l) |
|---|---|
| P14 | witnesses clean at s134 (incgraph, colorblind_doc, couleurs-fr-doc, abntexto, askmaps); pgfmath `array` returns its operand's form (59b); albi not in the corpus |
| P25 | LANDED b54, guard `perfect_kernel_batch54::xkeyval_usevalue_is_replaced_eagerly_at_setkeys` |
| P26 | `\meaning` of a MathPrimitive LANDED (tex_debugging.rs:198-203, witness sunpath); witnesses 0 errors at s134; the row's "biblatex-gost out of scope" is wrong — biblatex-gost-examples is in scope (its 770 jing lines are ruled D15 dangling links) |
| P30 | witness derivative 0 errors; 59g made OmniBus definitions yield (DIVERGENCES #408); whether a document `\newenvironment{example}` beats the combined stub (omnibus_cls.rs:554-573) is unverified — live verification line kept |
| P31 | LANDED 729e01e307 (nicematrix_sty.rs:820-826) + balanced `O{}` 55b |
| P34 | LANDED b54, guard `perfect_kernel_batch54::nicematrix_block_ampersand_body_is_a_subgrid` |
| P35 | open (live row kept) |
| P36 | fancybox-doc out of scope; VerbatimFootnotes landed 55t (P52) |
| P39 | isorot binding 54w; witnesses 0 errors at s134; kotex out of scope |
| P42 | LANDED: `\lx@list`/`\endlx@list` mode frames (latex_constructs/sect06.rs:525-549), guards `perfect_kernel_batch51::endlist_without_lx_list_frame`, `perfect_kernel_batch56::endlist_closes_an_enumerate_opened_by_its_begin_macro` |
| P48 | LANDED 54x (`\@tabarray` via an `\@array` internal) |
| P54b | LANDED 54q (l3draw surface); chinesechess an out-of-scope canary; circledtext, suanpan-l3 out of scope |
| P56 | fancybox-doc out of scope |
| P57 | LANDED 56e-56f (curve2e port, 117b82baeb; pict2e +56kn) |
| P58 | memoir runaway LANDED b52 (KPE #115); remaining witnesses out of scope or clean |
| P60 | quiver-doc 0 errors; tikz `MAX_ERRORS` recorded at KNOWN_PERL_ERRORS (2647) |
| P61 | superseded by 59a/59c levers and 59b (wheelchart 210 s Fatal → 118 s; pgf-interference 220 → 166 s) |
| P62 | qworld out of scope (CJK) |
| P63, P64 | LANDED 55a (gauss, shapepar bindings) |
| P65 | LANDED 54j/54k (codehigh binding) |
| P67 | derivative 0 errors |
| P68 | LANDED 54l (examdesign binding) |
| P69, P70 | LANDED 54m |
| P72 | open (live row kept) |
| P76 | LANDED 55c (theme options through the real `\ProcessOptionsBeamer`) |
| P77 | argumentation-doc 0 errors |
| P49 | nicematrix 0 errors since 2026-09-04 |
| P46 | neoschool, tikz-network out of scope |
| P47 | witnesses status ≤ 1, 0 errors |
| P16 | ii 54e, iii 56gf, iv 56jt, vi 54m, vii 56id, viii listings colour tokens, xii 54c, xiii 56if (l3text code-point layer, KPE #243), xv 54c landed; residue (i), (ix), (x), (xi) kept live |

## The table as it stood (verbatim)

| # | Target (mass) | Status | Plan summary |
|---|---|---|---|
| P14 | grab-bag singles (agF) | PARTIAL b40 (assoccnt deps, biblatex datamodel) | Remaining FINAL: incgraph \tcbusetemp/\dispListing temp write + `listing file=` via the VFS (13); colorblind pgfmath `array(list,i)` real implementation (17; Perl silently no-ops); couleurs-fr color-name key symmetric normalization (\lx@applyaccent leak); albi svg CaptureBlock→svg wrap (10). Recorded: abntexto \csstring = Lua-absorb tier (probe caveat); askmaps/iwonamath \mathversion = PERL-PARITY, surpass needs approval. |
| P25 | w3G-1 xkeyval `\usevalue` eager pointer replacement (1000/1 + any pointer user) | **FINAL → b46** | SHARED (Perl: unsupported error per use). xkeyval.tex:529/560-583 replaces `\usevalue{key}` EAGERLY inside `\setkeys` against the family being processed; our `\usevalue` is a lazy DefMacro (xkeyval_sty.rs:1165-1184) reading the GLOBAL `XKV@ptr@keysets` latch (keyvals.rs:936-943) at expansion time, after later `\setkeys` calls overwrote it. Fix: in `read_from` (~keyvals.rs:1013) splice `replace_usevalue_pointers` over the value tokens using the LOCAL `self.keysets`; keep the lazy closure as fallback. Repro: two families, `\stored` must be `42`. LOW-MED (hot path, gated on `\usevalue` presence). |
| P26 | w3G-3/4/5 + w3H residuals (recorded, not scheduled) | PARKED | sunpath: tikzmath variable dispatch needs `\meaning\angle` = macro; both engines model `\angle` as a math primitive (SHARED); sub-defect: `\meaning` of `Stored::MathPrimitive` prints `Stored[??]` (tex_debugging.rs:270 / store.rs:235) — fix cheaply when touching `\meaning`. hobby: expl3 fp/prop/pml3array (stub stays; Perl chokes raw). nicematrix calligraphic-decoration csname runaway → lane w3I. windycity biblatex declaration stubs (MED-HIGH, long tail); biblatex-gost/-sbl + fspsample = LuaTeX/fontspec docs, out of scope. newcommand/polydemo/t-manual/autolist = mode-frame family (USER-PARKED). calctab `\or` vs stale `\ifmmode` frame (SHARED, complex). synthslant engine-branch (SHARED). |
| P30 | w3C-2 OmniBus lazy theorem stubs hijack document-defined envs (derivative 61 lines; scrartcl → OmniBus) | FINAL → b47 | RUST-ONLY. `omnibus_cls.rs:538-557` binds the combined `\begin{example}` CS; `\begin` dispatch (latex_constructs.rs:3357-3394) prefers it over a later `\newenvironment{example}` (`\example`/`\endexample`). Fix: `\newenvironment`/`\renewenvironment`/xparse env definers clear `\begin{<name>}`/`\end{<name>}` combined CSes (latex.ltx `\@newenv` → `\<name>` only; the combined CS is our optimisation). Guard: `\newenvironment{example}{A}{B}` under an OmniBus class → body wrapped, no theorem. MED (touches env-definition hot path). |
| P31 | w3J-1/3 `\NiceArray` family leading `[pos/keys]` optional + balanced `[…]` scanning (≈170 lines / 6 docs: nicematrix 71, nicematrix-french 53, simples-matrices ×2, ProfSio ×2) | **FINAL → b47** | RUST-ONLY (Perl has no nicematrix binding: raw expl3 collapses, 89 errors on the repro). `\NiceArrayWithDelims` real sig `m m O{} m !O{} t\CodeBefore` (nicematrix.sty:1953, keys merged :2007); binding `nicematrix_sty.rs:488/495/527-536` lacks the LEADING `O{}` → `[t]{lcccccc}` binds colspec=`[`, rows spill → `&` storm + `\noalign`. Fix: `\NiceArrayWithDelims{}{}[]{}[]` and `\<x>NiceArray[]{}[]`, merge both optionals for `\lx@nice@setopts` + first/last-col growth (:514-520), drop `t/b/c`. Plus nested `[rules/color=[gray]{0.9}]` (nicematrix.tex:1364): `DefMacro!` `[]` = `read_until("]")` (gullet.rs:2325) non-balanced, while our xparse `o/O` already balances → route nicematrix env/`\Block`/`\Hline` signatures through balanced scanning (add a balanced-optional reader or define via `\NewDocumentEnvironment`). Repros `$SC/w3J/r1.tex` (9 `&` + 1 `\noalign`), `nested_bracket_opt.tex`. MED. |
| P34 | w3J-2 `\Block{…&…}` under `ampersand-in-blocks` leaks tabs (nicematrix.tex:1152) | FINAL → b48 | RUST-ONLY. Real `\Block` (:7257, `m m D<>{} +m`) boxes its content; binding emits `#4` inline (:556). Fix: when `#4` has a top-level `&`, wrap as nested array in the cell. Repro `$SC/w3J/block_amp.tex` (2 `&`). MED, niche. |
| P35 | w3J-4 `\Block{i-j}<fmt>{content}` angle optional (nicematrix.tex:555, 821; 0 errors, garbled content) | FINAL → b48 | RUST-ONLY. Add `D<>{}` slot via `gullet.rs:2298 read_optional_angled`, discard (formatting). LOW, content-only gain. |
| P36 | w3J out-of-scope witnesses: inline verb inside `\footnote` (`\MakeShortVerb{\|}` + `\footnote{…\|x\|…}` → `\lx@note` "Attempt to end mode internal_vertical", malformed itemize/td, stray `\omit`/`\endgroup`; SHARED engine bug, repro `$SC/w3J/shortvrb_fn.tex`); `\rowcolors[gray]{2}{0.8}{}` ignores leading `[model]` (xcolor/colortbl binding, 1 line) | DRAFT | Inline-verb-in-footnote needs its own root-cause lane (mode handling in `\lx@note` + `\lx@hidden@bgroup` box from latex_constructs.rs:9969). `\rowcolors` model optional: LOW once looked at (xcolor.sty `\rowcolors[⟨commands⟩]{⟨row⟩}{⟨odd⟩}{⟨even⟩}` — note the real optional is COMMANDS not a model; the doc's `[gray]` is a model passed where xcolor expects commands, check what real xcolor does before fixing). |
| P39 | w3K residual malformed roots (recorded) | PARKED | D: beamer sectioning `section→p/subsection` (~14 theme docs, 1 line each) — tail of beamer-navigation breakage (`\headcommand`, `\theframenumber` undefined; beamerbasesection raw), SHARED/entangled → needs a beamer-navigation lane. E: caption→block is a symptom (isorot `\Grot@setangle/@x/@y/@box` undefined = real sub-root; heria `\IfEq`/`\seformat` undefined; rubik `\ShowCube` shell-escape). F: `XMTok/XMApp→ref` cascades from `\node` outside picture (argumentation-doc) and 15× `expected:<variable>` (rec-thy) — chase the upstream errors. Mode-frame-adjacent docs (curve2e-manual, lineno, nih, objectz, screenplay-pkg, shtthesis, titlecaps, homework ×7) USER-PARKED; kotex-doc CJK-parked. |
| P42 | w3O-B1 `\lx@list`/`\endlx@list` mode framing | **FINAL → b47** | SHARED-adjacent but structural: latex_constructs.rs:5918-5924 should call `begin_mode`/`end_mode("internal_vertical")` exactly as Perl pool:1647-1653 does (only actionable non-parked item of the mode-frame study: 38 oracle-clean docs/~1050 lines are 100% SHARED and stay USER-PARKED pending Option A go). LOW-MED; re-convert 0802.2207. |
| P48 | aguplus `planotable` residual (24 errors after P41): `\pt@tabular` = `\hbox\bgroup$…\let\\\@tabularcr\@tabarray` (aguplus.cls:305-307) runs the bare `\@@array[c]` constructor without `\@array@bindings`/`\lx@begin@alignment`, so `&`/`\\`/`\noalign` fire outside an Alignment and `\lx@end@inline@math` mode errors follow; raw `\@classz`/`\@acol` undefined | DRAFT | SHARED (same-host Perl on the t2 shape: 2 `Stray alignment "&"`). Fix shape: give `\@tabarray` the same bindings `\array`/`tabular` get (`\@array@bindings[c]{#1}…\lx@begin@alignment`) — Perl pool:3765 has the same bare macro, so this is a surpass candidate (P16-x) unless it can be framed as kernel-quality. Repro `$SC/b47/t2.tex`. LOW-MED. |
| P54b | l3draw `\draw_*` undefined (chinesechess → TokenLimit Fatal, circledtext, suanpan-l3) | DRAFT (agent in flight) | Unmasked by b48. MED. |
| P56 | fancybox-doc residuals (46 + PushbackLimit Fatal): fatal = short-verb `\"\begin\"` inside `\footnote` (fancybox-doc.tex:310) and ~40 = `\SaveVerb`/`\vitem`/`{LVerbatim}` delimited-`\Verb` scans whose catcode-12 `\"` delimiter never matches in a frozen argument, leaking `\Verbatim@Space` into the verbtable (`&`/`\vcenter` cascades) — all the P52 family; 4 = `\caption`/`\toccaption` inside `\fbox{\begin{minipage}…}` (fancybox-doc.tex:359-373, SHARED Perl 2: float-only element in `<ltx:block>`, real LaTeX typesets it inline via `\@makecaption`) | DRAFT | Boxes/`{Sbox}`/B-environments/verbtable are clean in isolation (`$SC/b48-fancybox/hZ0_vt.tex`, `b_beqnarray`). Caption-in-box inline form LANDED (OXIDIZED_DESIGN #182). Rest waits on P52. |
| P57 | curve2e/pict2e raw support: `latexml_contrib/src/curve2e_sty.rs` is a version-only stub and `pict2e_sty.rs` is empty, so `\moveto`/`\lineto`/`\circlearc`/`\Ang`/`\Vect*`… are undefined; curve2e-manual 41 errors after b50, 6 corpus docs load curve2e | DRAFT | Picture family: pict2e.sty (:400-900) draws via `\pIIe@*` driver primitives (`\pIIe@moveto` etc., pict2e-pdftex.def) that need an SVG-path backend like the `picture` binding's `\lx@picture@*` layer; curve2e.sty (v2.x) is pure-macro over pict2e + xfp, so once pict2e's ~12 driver primitives map to SVG it should raw-load. MED. Witnesses: curve2e-manual, curve2e-v161-man, euclideangeometry-man, pict2e-doc, xfp-adjacent docs (`grep -l curve2e` over the corpus). |
| P58 | `\endlx@list Attempt to close boxing group` cluster (52 docs: incgraph 106, biblatex-gb7714-2015 54, cnltx_en 42, tcolorbox 26, ox*-doc memoir 16-17, digiconfigs 5 …): engine `\list` (latex_constructs.rs:5981) opens a `\lx@list` bgroup that `\endlist`=`\endlx@list` closes; any class that `\renewcommand*{\list}` (memoir.cls:4580 confirmed for digiconfigs/ox*-doc, repro `$SC/b50/dc7.tex` 1 error) replaces the OPEN side only, so the close finds no boxing group — but most of the 52 are NOT memoir, so a second opener-replacing path exists | **PARTIAL b51** — tolerant closer landed; memoir csname runaway root-causing in flight | Fix shape depends on the agent's mechanism for the non-memoir majority (candidates: `\@trivlist` raw redefinitions, tcolorbox/enumitem `\list` wrappers, `\@doendpe`). Pairing rule: whichever macro OPENS the bgroup must be the one `\endlist` mirrors — likely make `\endlx@list` tolerant when `\list` was rebound, or bind `\list` so `\renewcommand{\list}` layers over `\lx@list` (latex.ltx:5049 `\list` is a macro, so a class MAY replace it). MED-HIGH (touches every list). **b51:** `\endlx@list` pops only a `\lx@list`-initiated frame (groupInitiator) and otherwise emits Perl's `endMode` error without popping (Stomach.pm:524-531) — the cascade source (popping the enclosing `{` frame) is gone, but the memoir witnesses still hit an unbounded EXPANDING `\csname` scan right after (memman 1001: partial `\LP:\endcsname{L}}…`, oxalph-doc 1001: `\bm@bicolor ,colframe = hacked…`; first site memman.tex:5682 `\end{egresult}` after `typeseteg`=`\list…\endlist`, memsty.sty:775-782). Non-memoir majority (incgraph, biblatex-gb7714, cnltx, tcolorbox) still unassigned. Guard `perfect_kernel_batch51::endlist_without_lx_list_frame` (asserts Perl's one-error shape).  **b52:** the memoir `\csname` runaway was `\@currbox` — Perl's EMPTY macro (latex.ltx:12099 `\newbox\@currbox`) so dpfloat/memoir's `\csname …\@currbox\endcsname` store scanned to EOF (KPE #115, guard `perfect_kernel_batch52::currbox_is_a_box_register`); memman 1001→478 (`&` 130, `\endlx@list` 94 = P38, `\GenericError` 88, caption/toccaption malformed 46, `\@sharp` 28); oxalph-doc's is a DISTINCT tcolorbox runaway (still open). Non-memoir majority still unassigned. |
| P60 | pgf matrix (`tikzcd`) inside a tabular cell: `\hbox\bgroup\vbox\bgroup\halign\bgroup` (pgfmodulematrix.code.tex:192-193) collides with the outer alignment — first error `\lx@begin@alignment Attempt to close boxing group`, then `\tikzcdmatrixname` csname cascade, and with ≥2 such cells the column-end template regeneration in `gullet::handle_template` never consumes cell content: a ~2.4 GB single Vec → exit 137 under the RAM guard (quiver-doc; repro `~/data/pk_agents/w4/quiver/min8.tex`). SHARED (Perl loops at 4.8 GB to timeout). | DRAFT | Levers (agent-ranked): (1) zero-progress fuse on template re-insertion → recoverable Error + `end_row` (beyond-Perl, MED); (2) extend the cooperative `--max-memory` fuse to the gullet pushback Vec (LOW-MED; turns 137 into a Fatal with partial output); (3) real nested-alignment boxing (HIGH; the true root, R9-adjacent). Also RUST-ONLY note: tikz_sty.rs:15 raises `MAX_ERRORS` to 1000 (Perl keeps 100) — not the crash cause. |
| P61 | tikz/pgf + expl3 VOLUME in the debug harness: tikz-network manual (42 `\Vertices`/29 `\Edges` over datatool CSVs — `\__datatool_parse_datum:n` ≈0.31 s/call, ~10 s per CSV row, LINEAR, Rust 4× faster than Perl), liftarm (≈225 animation frames × ~2.7 s of raw l3fp Newton-Raphson + pgf), wheelchart (93 charts × 4.6 s), twoxtwogame (836 pictures, 492 s to complete; pdflatex 18.6 s) — all correct output, no loop; each ~0.5-2.7 s per tikzpicture in the DEBUG build. | PARKED (perf) | No faithful code fix; corpus levers = release-profile sweeps / higher budget for these 4, or a native l3fp fast-path (HIGH). Do NOT revert `divideround` (it is the cure: the storms it removed were what made these docs "finish" early). |
| P62 | ltjsarticle (luatexja) leaves `\currentgrouplevel=18` + a leaked restricted_horizontal frame after the class load (its `\ExplSyntaxOn`+undefined `\patchcmd` block, ltjsarticle.cls:771-791), and IN THAT STATE our `\if`/`\ifcat` char comparison returns EQ for `\if AB` (both operands read as charcode 256; `\ifx`/`\ifnum` fine) — pgfkeys' `\pgfkeys@gobbletoslash` (`\if\relax…`) then keeps trailing slashes and every `.code`/`.store in` handler no-ops (qworld: 1001 pgf errors; Perl 49). Repros `~/data/pk_agents/w4/qworld/{k1,cap}.tex`. | PARKED (LuaTeX-Japanese) | The `\if` operand-read divergence (gullet.rs `read_x_token` → `ExpandedIfToken`, tex_logic.rs:30-43) is RUST-ONLY but no non-luatexja trigger was found; revisit if a pdfTeX witness shows `Stored`-typed operands reaching `\if`. |
| P63 | gauss.sty `gmatrix` measures its `\ialign` result with `\lastbox` recursions (`\g@measureRows`/`\g@measureCols`, gauss.sty:966-1047) that need one non-void box per matrix row; LaTeXML's box model yields one box then void → `\g@maxrow` never reaches <0 → PushbackLimit (gauss-ex, gauss-doc; Perl hangs identically). Repro `~/data/pk_agents/w4/pkgloader_gauss/g3.tex`. | DRAFT | Only real fix = a gauss binding (`gmatrix` → XMArray + `\rowops`/`\add`/`\mult`/`\swap` annotations), HIGH effort; a stub that neutralises the measure loops would drop the row-operation arrows (out-of-scope emulation, not parity). |
| P64 | shapepar `\shapepar`/`\heartpar` fixed-point loop `\loop\if\AbsVal\@tempdima<\p@ \multiply\@tempdima\@cclvi…\repeat` (shapepar.sty:217-219) never converges because the box measurements stay 0pt → MemoryBudget Fatal (ArsClassica + ~7 docs; Perl hangs to timeout). Repro `~/data/pk_agents/w4/jnu_b2t/repro_shapepar.tex`. | DRAFT | Beyond-Perl option: a shapepar binding typesetting the argument as an ordinary paragraph (MED, presentational package). |
| P65 | codehigh's highlighter is O(n²) l3regex calls (`\__cdhh_parse_code_once:nN`, codehigh.sty:539-549: 8 rules × `\regex_extract_once`+`\regex_split` over the whole remaining list per emitted token) run through the interpreted l3regex VM — a 928-line `\dochighinput` (pegmatch manual) is ~10G tokens; Perl times out too, Rust is 24× faster per line, pdflatex is C-fast. Repro `~/data/pk_agents/w4/pegmatch/mini.tex` at `LATEXML_TOKEN_LIMIT=20000000`. | PARKED (perf) | Only lever = a native l3regex fast-path (HIGH; the 2026-06-20 regex-crate shim was removed for semantics). Not a regression: sweep 28 stopped at the then-undefined `spectblr`. |
| P67 | derivative manual: a `\cs_new_protected` env-end macro carrying `\endtabular` (derivative.tex:824-828) is NOT expanded by the alignment-body scan (`read_x_token(fully_expand=false)` returns a protected macro unexpanded — Gullet.pm readXToken / gullet.rs:1286), so `\endtabular` runs mid-column and the `\lx@begin@alignment` frame outlives the env (~100 errors, Fatal). SHARED (Perl 101+Fatal). Fix shape: in `digest_alignment_column` (tex_tables.rs:953/1025) expand a protected MACRO whose body does not start with `&`/`\cr` instead of invoking it (keeps the protected-`&`-stays-content witness at gullet.rs:467). Repro `~/data/pk_agents/w6/derivative/R1.tex`. MED, 1 doc. | DRAFT | agent w6 |
| P68 | examdesign (examplea/b/c, ~60 lines, pdflatex-clean): examdesign.cls:323 `\def\section{\stepcounter{section}\setcounter{question}{1}}` + `:344 \def\endsection` and every exam env opens with `\begin{section}` — our `\section` is LOCKED (Perl pool:559 too), so the kernel `\section` runs and hunts a `{title}` (`expected:{` + `\lx@tag` mode cascade). SHARED (Perl 67 errors). Fix = an examdesign class binding: scoped unlock of `\section`/`\endsection` around the raw class load (MED, the exam-collection machinery unverified) or native exam-list environments (HIGH). 3 docs only → DEFERRED. Repros `~/data/pk_agents/w7/examdesign/{r3,kmin}.tex`. | DEFERRED |  |
| P69 | screenplay-pkg.tex:67 `\abstract{\noindent\begin{quote}…` never closes its brace — pdflatex tolerates it (article's `\abstract` is an environment-begin taking no argument; the `{` is a plain group closed at `\end{document}`) while our `\abstract`+`{` reads `\lx@add@abstract{}` to EOF (14 errors; Perl: same readBalanced then `Fatal:deep_recursion`). Fix shape: when the braced read is unbalanced, fall back to the environment-open. SHARED, 1 doc. | DRAFT | agent w8 |
| P70 | `\cl@<ctr>` is a State VALUE, not a macro (`new_counter`, `latexml_core/src/binding/counter/dialect.rs:106`; Perl `Package.pm:674` identical): contract.sty:336 `\edef\cl@Clause{\cl@Clause\cl@contractClause}` → `undefined:\cl@contractClause` ×60 per doc (contract en/de, pdflatex-clean). | DRAFT — SHARED surpass | Fix shape: also define `\cl@<ctr>` as an expandable macro (`\@empty`) and keep the reset list in sync — `\@addtoreset` and the reset machinery read the VALUE, so both must update together (MED risk). Repro in the wave-9 grabbag report (`~/data/pk_agents/w9/grabbag/repro_cl.tex`). |
| P72 | skak `\mainline`/xskak `\xskakloop` move parser under beamer `\only<…>` (chessboard_and_beamer, xskak_and_beamer: `expected:Until:` + `Fatal:Mouth:EoF` after the `\board` fix) | DRAFT | Separate root from the `\setrmkeys` one-step fix (batch 54h); bottoms out in `base_parameter_types.rs` `Until` scan of `1.` move numbers. Repros `~/data/pk_agents/w9/chess/*_and_beamer.tex`. |
| P76 | beamer `\usetheme[sidebar]{Verona}`: theme OPTIONS are dropped (`beamer_cls.rs` `\use*theme` discard `[opts]`, `\ProcessOptionsBeamer` no-op) so `\ifbeamer@sidebar` never turns true and `\sidegraphics` is the theme's error stub (verona-sidebar ×4). | DRAFT | Forward `[opts]` to `require_package` under the theme's `\@currname` and give `\ProcessOptionsBeamer` beamerbaseoptions.sty:15-32's body; a probe replacing only the latter did not bind the keyset. MED-HIGH (theme-loader plumbing, content.rs:2165). |
| P77 | argumentation.sty `af` (xparse wrapper of its own `af*` = `\tikzpicture[standard,af,#1]…\endtikzpicture`): `\node` undefined only through the original `af*` entered from `af` (hand-rolled replicas do not reproduce). SHARED (Perl identical). | DRAFT | Repro `~/data/pk_agents/w11/beamer/argmin.tex`; also 8 SHARED `XMApp in ltx:ref` from `\afref` math inside a node. |
| P49 | nicematrix exemplar TokenLimit Fatal at EOF (b47: 160 errors + Fatal; `readBalanced ran out of input` at line 8334 → `\endgroup` storm + glued CS names `\smallskipWhenthe` = re-tokenized body with spaces ignored) | DRAFT | RUST-ONLY (oracle lualatex 0/0). Over-read starts after line 3061 (`cut3061.tex` → 88 errors, no Fatal; `$SC/b47/nmb/cut{4000..8000}.tex` bisect in flight). Pre-b33 residual 58× `&` at lines 409-411 is separate. MED. |
| P46 | w3P ROOT2 neoschool `every box on layer 0` (tcolorbox.sty:1409-1422, `\c@tcblayer` stays 0) + ROOT4 tikz-network `\cmdNW@vertex@fontcolor` in `\csname\color@…` (213×) | DRAFT | Both likely RUST-ONLY; ROOT2 needs a batch 42→44 bisect, ROOT4 check the batch-43 color-key rework. MED. ROOT1 cnltx `example` (`\lstnewenvironment` opening `\mdframed`) is SHARED/mode-frame-parked — the batch-42 VFS tee is correct, do not revert. |
| P47 | w3E loop Fatals (RUST-ONLY set) | DRAFT | panda `\clist_item`+`\int_step` f-expansion loop (etex.rs:50-105 numexpr vs `\exp:w`, repro `$SC/w3E/k_twochar_nodigit.tex`) HIGH; zugferd `\fi:\relax` deferral loop (conditional.rs:182-235 `parsing` flag) MED-HIGH; europecv longtable header re-tokenization (`ecv1.tex`) MED-HIGH; istgame `\printindex` loop MED; tex-font-cheatsheet fancyvrb-in-tikz HIGH. knowledge `\filluptopage` LANDED (OXIDIZED_DESIGN #178: `\lx@newpage` steps `\c@page`); europecv LANDED (KPE #132 hook catcode); msc HIGH. biblatex-apa-test no longer fatals. |
| P16 | SHARED-failure surpass candidates (agB/agF/agH/agI) — NEEDS USER APPROVAL | DRAFT | (i) svg-verb (68/2): \verb's before_construct force-opens ltx:p inside svg:g where foreignObject would be legal — skip the p-open when it cannot be contained in an SVG context (Perl identical, latex_constructs.pool.ltxml:1844). (ii) LANDED batch 54e (OXIDIZED_DESIGN #182; guard `caption_without_a_float_ancestor_degrades_to_text` (renamed `caption_outside_a_float_becomes_its_float` in 56gs): `\@@caption` degrades to the inline `ltx_caption` text when no float ancestor exists, box captures end the walk). Was: margin-caption (148/7): tufte marginfigure caption in ltx:text → insert_block ltx:block fallback rejects caption; wrap as figure instead (Perl identical). (iii) frontmatter fallback placement for `\maketitle` inside box captures (KPE #86, 16/4). (iv) glossaries math-aware `\lx@glossaries@gls@link` (KPE #84, 106/2 — drop the glossaryref wrapper in math mode). (v) albi svg CaptureBlock→svg wrap (10/1). All pdflatex-clean. w3D re-confirmed (i) (fix site latex_constructs.rs:6170-6174: gate the p-open on `is_openable("ltx:p")`, Perl Document.pm:833; makeshape 22) and (ii) (tufte-common.def:1110-1133, `^^`-float caption with no float ancestor → degrade to `\@@generic@caption` inline form; tikzrput 23; batch 53 adds xltabular-doc: raw tocbasic `\captionaboveof{table}` at top level sets `\@captype` and runs our `\caption` with no float ancestor — 2 of its 3 residual errors). NEW (vi) w3F-3: `\underline`/`\overline` are non-robust `\protect\ifmmode…\fi` macros in BOTH engines (tex_math.rs:1283 locked, Perl TeX_Math.pool:991) vs latex.ltx:16369 `\DeclareRobustCommand` — bibarts' write/rescan path orphans the `\fi` (14 lines/2 docs); MED-HIGH, wants a deliberate robust-command pass over the `\protect\ifmmode` family. NEW batch 53: (vii) l2tabu (and any `\ifnum\pdfoutput=…` doc) takes the DVI branch in BOTH engines (`\pdfoutput`=0) while the pdflatex oracle takes PDF — engine-level (`\pdfoutput`=1 like Perl's `--pdf`-less default? decide against the arXiv corpus: many papers key graphics extensions on it); (viii) listings `backgroundcolor=\color{…}`/`rulecolor` are digested EAGERLY at `\lstset` time (Perl `lstExtractColor`, listings.sty.ltxml:945-953; real lstmisc.sty stores `\lst@bkgcolor` unexpanded) — `\lstset` before `\usepackage{xcolor}` errors `undefined:\color` in both (hvfloat.tex:29/45); fix shape: store the tokens, extract the color in `\lst@@@set@background` at listing time (1 witness so far — wait for a cluster); (ix) `\fontsize`/`\@setfontsize` font model for raw classes (raw KOMA `scrsize11pt.clo` sets sizes our font model ignores; typearea 'Bad type area settings!' warnings are the visible symptom); (x) #175's `malformed` warning path — flag which unknown section types still warn after the level mapping; (xi) `\markright`/`\markboth`/`\pagenumbering` primitive→macro (same KPE #121 shape) if a raw-class witness appears; (xii) LANDED batch 54c (`use_load_hooks` in content.rs — package/class/file before+after hooks around every binding or raw load; guard `package_after_hook_fires_for_a_binding_load`). Was: **package/file load hooks** for raw classes: `\AddToHook{package/scrbook/after}` / `class/…/after` / `file/…/after` (ltcmdhooks/lthooks, latex.ltx `\@@_hook_file_…`) never fire because our `\usepackage`/`\documentclass` paths bypass `\@onefilewithoptions`' hook sites — DEMO-TUDaPhD's `\@addchap` (tudapub.cls hooks scrbook's `\addchap` after load) stays undefined; SHARED (Perl identical); probe `$SC/b53/ps/hook/h.tex`; fix shape = run `\UseHook{package/<name>/after}`+`file/<name>.sty/after` at the end of every raw-file load (package.rs `input_definitions`), MED. (xiii) **engine self-description vs per-codepoint tokenization** (neoschool-fr, SHARED — Perl 101 errors): `\sys_if_engine_opentype:TF` is FALSE (no `\tex_Umathcode:D` when expl3 loads during the dump build) while the mouth yields one token per Unicode codepoint, so l3text's `\__text_codepoint_process:nN` (expl3-code.tex:35892-35928) reads `é` as a UTF-8 LEAD byte, eats `\q__text_recursion_tail`/`\q__text_recursion_stop` and dies `Fatal:Mouth:EoF Until:\q__text_recursion_stop` (repro `$SC/agents/neoschool/perltest.tex`: `\text_titlecase:e {Corrigé}`). Fix shape: define the `\Umathcode` family (latex_constructs_rust_only.rs:117-124) BEFORE expl3.ltx in the dump build (latex.rs INI_MODE ~L84-125), `\let\Umathcode\@undefined` right after, keep the post-dump definition; MED-HIGH risk (greek_test LGR, encoding probes latex.ltx:9437/14453/14662/15463); do NOT override `\__text_codepoint_process:nN` alone. (xiv) pgf/tcolorbox `enhanced`+`attach boxed title` frame drawing costs ≈0.45M tokens per box (glossaries-user: ~706 boxes → 1G TokenLimit; `LATEXML_TOKEN_LIMIT=0` completes; Perl ≈2.6 s/box would time out) — lever is a native pgfsys SVG scope protocol (`pgfsys_latexml_def.rs` `\lxSVG@begin/endscope`, `\pgfsys@invoke`), perf not parity; repro `$SC/agents/glossaries2/RUNAWAY.tex`. upstream filings for #175 / KPE #118-#121 NOT yet done. (xiv) **accent+base composition under forced expansion** (bibleref-parse.tex, silent 300 s hang → `Fatal:Timeout:Convert` only via the internal deadline): `\brp@@expandcs` (bibleref-parse.sty:491-508) expands one token at a time until a CHARACTER; under T1 real LaTeX collapses `\"o` to one char via `\DeclareTextComposite` (fontenc; under OT1 pdflatex hangs identically), which BOTH engines ignore (Perl latex_base.pool:364-367, Rust latex_base.rs:426) — `\expandafter` (tex_macro.rs:205) pushes the non-expandable `\lx@applyaccent` back unchanged → flat tail loop no cycle guard catches. SHARED (masked in Perl only by its KPE #123 backquote bug). Fix shape: in `\expandafter`'s unexpandable branch (and later `\edef`-class full expansion), reduce `\lx@applyaccent … {letter}` to the NFC-composed char (reuse `tex_character.rs::apply_accent` composition) — the `\csname` loop already special-cases it (gullet.rs:2118). Digestion untouched. Repro `~/data/pk_agents/brp2/REPROS/r_min.tex`; oracle pdflatex `IK`+0xF6+`nige`. LOW-MED. (xv) LANDED batch 54c (OXIDIZED_DESIGN #177). Was: **source-tree relative package paths** (tikzpingus-doc.tex:15-16 `\def\input@path{{../tex/}}` + `\usepackage[glows]{../tex/tikzpingus}`): the manual compiles only from the CTAN source layout where `doc/` and `tex/` are siblings; in the installed TL tree `../tex/` does not exist, kpathsea fails, pdflatex AND Perl fail identically (SHARED, build-environment) and the whole 354-error shape+layer mass follows from the package never loading (agent-verified: the real basename loads clean, 0 errors, correct SVG). Beyond-Perl lever: when a `.sty`/`.cls` request contains a directory separator, the literal path misses, and the BASENAME is kpathsea-resolvable, retry with the basename (content.rs miss-handler :948-957, before `missing_file`) — **61 doc/latex manuals** use `\usepackage{../…}` (classicthesis, pgf-go, cora-macs …). Risk MED-HIGH: can load the wrong (system) version where a doc means its local copy (memory 0906.3507); gate to package loads, OXIDIZED_DESIGN entry. Repro `~/data/pk_agents/w4/tikzpingus/repro_root.tex`. |

## Architectural queue (verbatim)

## Architectural queue — principled abstractions over per-package scanners

User directive (2026-09-01): no stopgap guards / one-off defensive logic;
model the kernel's underlying mechanics generally. Assessment of the
session's landed shapes against that bar, and the generalizations owed:

1. **TeX file I/O as a virtual file store (HIGH).** The `{name}_contents`
   cache is already a de-facto VFS, but it has FOUR ad-hoc writers
   (filecontents env, fancyvrb VerbatimOut, fancybox VerbatimOut, memoir
   writeverbatim line-capture) and ad-hoc readers (verbatiminput cache
   check, find_file slurp path). The kernel mechanics being modeled are
   exactly `\openout`/`\write`/`\closeout` → `\input`/`\openin`/
   `\IfFileExists` round-trips. The general abstraction: one virtual
   file-store module (latexml_core) that ALL \write-to-stream output lands
   in and ALL file reads consult first; verbatim WRITING environments
   become one shared "raw-line capture until end-marker (with
   `\VerbatimEnvironment`-style env redirect)" facility parameterized by
   terminator + sink. Retires the three duplicated scanners and makes
   every future write-out/read-back package (dry.sty, answers.sty,
   exercisebank, tutodoc "examples") work without per-package code.
2. **Beamer template/option execution (MEDIUM).** The color model now
   mirrors beamerbasecolor's mechanics; templates and `\DeclareOptionBeamer`
   remain absorbing no-ops. General model = actually storing template
   bodies + executing the beamer option processor against declared keys.
3. **`\iffontchar` truth (LOW).** Currently always-true (args faithfully
   consumed). General model needs per-font glyph coverage (TFM bc/ec +
   existence) in the metrics layer.
4. **`\everyeof` + artificial mouth ends (MEDIUM-HIGH).** The eTeX
   mechanism is implemented (MouthRuntime::insert_everyeof, close-time
   token-identity insertion, read_token transparency for marked mouths)
   but UNWIRED: enabling it makes `\tl_set_rescan` exact (stop quarks
   delivered) yet loops spath3/litetable/zref in 5-token expansion cycles
   — an unresolved interaction to root-cause (suspect: repeated insertion
   or crossing order vs l3tl's grouped everyeof discipline). Related
   insight: our isolated argument mouths create ARTIFICIAL EOFs that real
   TeX doesn't have — the deep model is fewer isolated mouths, not more
   EOF patches.
5. **expl3 file-boundary state (MEDIUM-HIGH).** batch 31 narrowed the
   ad-hoc exit-Off to kernel-managed frames; the principled model is
   routing ALL load boundaries through the dump's real `\@pushfilename`/
   `\@popfilename` (their expl3 hooks) and deleting the flag machinery
   (CLUSTERS ctex part-2 row).

## Standing execution queue (verbatim, items 1-13)

## Standing execution queue (main session)

1. Execute FINAL plans in ascending risk order; batch 3-5 per suite run
   (feedback_batch_fixes_parallel_rootcause).
2. Every executed plan: red/green guard naming the witness + separate
   vetting of the witness's full XML + LEDGER batch row.
3. Sweep after each 2-3 batches on a quiet machine, release binary,
   tallied with `tally.sh` against the previous sweep; re-run timeouts solo
   before recording; exemplar row every sweep.
4. Every 2-3 sweeps: S2 validation + S3 recall over the S0∧S1 slice.
5. **Guard-strength side-goal (user 2026-09-17):** assertions must capture the
   whole markup context of the guarded feature — the complete element with its
   attributes and children, or a line-by-line golden (the original LaTeXML
   `t/` style) — never a short substring that unrelated markup can satisfy.
   Every new guard follows this. The FULL AUDIT of the existing suite is done
   (read-only classifier, 2026-09-17, `~/data/pk_agents/w23/test_audit/`
   REPORT.md + inventory.tsv): of 2,867 tests, A (golden / whole-document /
   multi-element) 421 = 367 `tex_tests!` pairs + 54 handwritten; B (one full
   element, count or absence) 1,508; **C (short substring, attribute-only,
   bare tag, error-count-only) 938 = 520 on XML output + 418 typed-value unit
   predicates**. 371 of the 520 live in the `cluster_package_guards` binary. The three
   golden harnesses differ in strictness: `tex_tests!` exact line-by-line
   (`util/test.rs:371-405`), `post_test` normalized LCS diff
   (`90_latexmlpost.rs:47-120`), `streaming_sweep` byte-equal eager-vs-streamed.
   Upgrade batches, in order: **B1** an `assert_element(xml, tag, attrs, inner)`
   helper lifting the bare-tag / attribute-fragment / split-element families
   (~120 mechanically); **B5** one structural assertion for the 40
   error-count-only guards; **B2** computed-value marker probes → golden
   `.tex`/`.xml` pairs; **B3** extract-and-compare text; **B4** structural
   absence checks; the 418 unit predicates last. No test is deleted.
7. **pgf/TikZ throughput to pdflatex speed (user directive 2026-09-18).** Constant-factor
   levers landed (56da/56db/56de/56df/56dg: picC 559.5 G → 459.6 G instructions, −17.9 %;
   corpus wall 9,531 → 8,729 s); levers B and G reverted as measured losses — the user's
   rule since: algorithmic and strategic gains only, no caches or carried state in
   gullet/stomach/mouth/document. The corpus token budget
   (`~/data/pk_agents/w23/perf_pgf/token_budget2/`) puts 69 % of the wall on manuals
   loading pgfkeys and 40-55 % of a style-heavy conversion's instructions in raw
   pgfkeys dispatch. The native pgfkeys dispatch, hybrid on the raw `\pgfk@` storage
   with the raw handlers kept, has landed slice 0 (accessors, 56dk) and slice 1 (the
   `\pgfkeys{}` loop as a stream continuation, 56dl): zx_full 25.51 → 10.62 G (−58 %),
   tcb_full 18.31 → 13.25 G (−28 %), keys_heavy 15.23 → 11.90 G (−22 %), the zx-calculus
   manual 150 → 72 s; ON/OFF byte-identity harness of ten fixtures
   (`docs/performance/PERFORMANCE.md`, `~/data/pk_agents/w23/perf_pgf/pgfkeys_native/`).
   Slice 2 (56dp) landed the definition handlers: zx_full 25.51 → 9.98 G (−61 % over
   three slices), tcb_full −27 %, keys_heavy −21 %; slice 1 was corrected to the raw
   stream token for token (56dn/56dr — the four sweep-95 regressions were all shapes a
   handler could observe: an absent handler, a forward scan, a `}` as list, leading
   spaces, the splitter's surplus). The non-pgfkeys floor is profiled (`~/data/pk_agents/w23/perf_pgf/tikzcore/`,
   PERFORMANCE.md): pure TikZ is ~1.1× pdflatex (pgfmath and pgfsys are native);
   tikz-network's 5.5× is datatool v3's l3regex CSV parse plus `\DTLforeach`. The
   native CSV load landed as 56dt (registers byte-identical to pdflatex; picC −7.6 %,
   the manual −8.3 %), then native `\DTLifeq`/`\DTLifstringeq` as 56du — 95 % of a
   `\Vertices` call — picC 458 → 16 G, the tikz-network manual 190.7 → 25.0 s against
   pdflatex's 34.5 s on the same host: **the directive's headline document is past
   pdflatex speed.**
   The remaining slow calls are profiled (PERFORMANCE.md "The remaining slow calls"):
   pgf-spectra and tabularray are FASTER than pdflatex, circularglyphs 1.4×, tilings
   3.4× on the raw pgf-core drawing pipeline (no native leaf left; `.try` is under
   0.5 % of wall — declined). Open: the pgf-periodictable (25 GB retained DOM,
   282k `svg:g`) blowup as a stability lead (11; the picture-end seam landed as 56dv),
   the tcolorbox RSS lead (10), and the generic token/allocator lever (P5) as the
   only remaining throughput lever with corpus reach. Wheelchart (420 s timeout,
   10 GB) is SETTLED 2026-09-19 as pathological for every engine — multiline
   `arc data` under `arc around text` spins pdflatex too (no PDF in 120 s on a
   10-line repro) and Perl aborts on l3fp errors before reaching it; registered in
   DIFFICULT_CASES §D11, no lever.

8. **K12 — pTeX kanji control-word names.** LANDED as batch 56ds
   (`KERNEL_CAPABILITIES.md` K12): `PTEX_PROFILE` from `\NeedsTeXFormat{pLaTeX2e}`,
   the kcatcode block set lettered in `state::is_ptex_kanji_letter`. The residual on
   jsarticle documents is the PARKED pTeX engine-primitive set (`\kanjiskip`…),
   shared with Perl.

9. **`\\` under a corrupted alignment template — the align-state guard (lead,
   root-caused 2026-09-18).** plextdelarray: Rust 27 `Extra alignment tab` vs Perl 10
   (the +17 cross the shared `MAX_ERRORS` 100 into a Fatal). The classifier
   (`gullet.rs:3454-3477` ≡ Gullet.pm:266-277) and the overflow recovery
   (`alignment.rs:265-294` ≡ Alignment.pm:136-144) are identical; the divergence is
   `\\`: Perl runs raw latex.ltx `\@arraycr` with its `{\ifnum0=`}\fi` align-state
   guard (latex.ltx:16583-16594) and reaches a real `\cr` per row, Rust `Let!`s
   `\@arraycr`/`\@tabularcr` to the native `\lx@alignment@newline`
   (sect10.rs:175/194, tex_tables.rs:359-375) whose `inside_cell_group()` heuristic
   (tex_tables.rs:1561-1564, self-documented "not there (yet)") misfires on the
   broken preamble, so a visual row overflows three times. tex.web §792 (change the
   extra `&` to `\cr`) is implemented by neither engine. Fix = model the real
   align-state guard for `\\` — HIGH risk (1610.00974, tabularray/ProfSio in-cell
   `\\`, kbordermatrix guards), LOW gain (one pLaTeX doc): fold into an align-state
   modeling program with those witnesses, never a report cap. Side find: plain
   `delarray` without Japanese gives Rust 0 / Perl 3 errors (`aligntab/da.tex`) —
   verify against pdflatex and guard. Scratch `~/data/pk_agents/w23/regr96/aligntab/`.

10. **tcolorbox manual RSS — VERDICT (2026-09-19, `~/data/pk_agents/w23/perf_pgf/tcb_rss/`):
    a known-heavy outlier, not a leak.** Uncapped it completes in 349 s at a peak of
    **37.8 GB**, 78 MB of XML (198,801 `svg:g`, 177,582 `svg:path`, 62,227 `text`,
    4,604 pictures, 5,477 `listingline`); RSS climbs ~100 MB/s monotonically across
    all 28 chapters and never drops — the whole document's digested box forest,
    pinned per element by `node_boxes` (`document.rs:91`, `set_node_box` at
    :2193/:5244/:5589/:5646/:5797), swept only on the streaming spill path
    (`core_interface.rs:1296`). Perl keeps the same per-node box (`Document.pm:1654-1671`,
    refcounted, heavier) — SHARED; the `MemoryBudget` fuse is Rust-only tooling doing
    its job. Not an accumulator elsewhere: the listings VFS holds 582 temp files under
    1 MB, pgfkeys/arena/idstore stay small. Forced streaming at 16 GB holds ~4 GB for 17
    chapters, then creeps 4 → 12 GB through theorems/breakable/magazine/poster and dies
    at poster: those chapters are un-yielding digestion units (multi-page `breakable`
    boxes, `tcbposter` layouts) with no seam — the creep already noted at
    `latexml_oxide.rs:620-623`. Levers: (1) sweep policy — convert this manual on a
    dedicated high-`--max-memory` pass or exclude it from the 6 GB parity target
    (LOW); (2) yield seams inside `breakable`/poster digestion and a mid-large-box spill
    (MED-HIGH, the real prize, a separate initiative with PLANS 11); (3) eager-path
    release of `node_boxes` for closed subtrees (HIGH: every `get_node_box` consumer —
    `document.rs:1192-1204` math relocation, :4089 margin repositioning, :4462 root
    hooks — must be proven not to read a closed node's box; diverges from Perl's
    refcount-keep). Raising the streaming projection constant would not help (streaming
    dies too) and would over-stream healthy documents.

11. **Memory: the whole-document box tree is the peak — streaming needs a picture-end
    seam (FINAL, 2026-09-19; `~/data/pk_agents/w23/perf_pgf/periodictable/`).**
    pgf-PeriodicTableManual: 24.8 GB RSS, and it is not the DOM (2.8 MB XML for 8 tables ≈
    30 MB) nor a drawing excess (141,157 `svg:g` against pdflatex's 419,452 graphics
    scopes — we already elide same-colour path wrappers as Perl does,
    `pgfsys_latexml_def.rs:753` ≡ pgfsys-latexml.def.ltxml L397). RSS is linear in
    tables (216 MB + ~106 MB per table, ~47,000 boxes per table of which ~92 % are
    non-element pgf path/coordinate/scope whatsits) and peaks at the END of digestion,
    before Build: `digest_internal` (`core_interface.rs:1541-1549`) materializes the
    whole document as one `List` before `convert_document` absorbs it (:1521) — the
    same shape as Perl `Core.pm:214-235` → `Document.pm:564-604` (SHARED), with a
    per-box footprint ~2.4 KB (`Rc<DigestedData>`, `stomach.rs:1592` calibration) against
    Perl's ~0.7 KB, so Rust peaks where Perl merely crawls. The streaming path is the
    only mechanism that can bound it, and today it FAILS on this document: with
    `--max-memory=6144` the fuse trips at 4.8 GB and writes a 39-byte empty document,
    because `digest_next_body` yields only at a legal seam on the current level
    (`stomach.rs:438-446`) and a tikzpicture's ~46k boxes sit at a deep group level with
    no seam until the picture closes. Fix, part one LANDED as 56dv: `\pgfsys@endpicture`
    requests a yield (`stomach::request_fragment_yield`, honored by the yield predicate
    at the next legal seam whatever the box count; inert under eager digestion), so
    `n8.tex` (eight `\pgfPT[show title=false,show legend=false]\newpage`) streams at a
    2 GB cap with one yield per table: 0 Error/Fatal, exactly 6,264 `svg:g`, byte-identical
    XML, 22 s against the eager path's 166 s, peak 1,088 MB against eager 1,120 MB on the
    debug binary (whose per-table boxes are larger than release's; the 1 GB cap still
    trips inside one table there). Guard `112_fragment_yield::a_picture_end_is_a_seam_request`.
    Part two, LANDED in the same batch: the resident-wrapper spill and the wrapper's box
    release (LEDGER 56dv) — without them the seam freed nothing (RSS monotonic 205 → 1,056 MB,
    the reviewer's catch). Part three LANDED as 56dx: the ~6 MB per picture was
    `remove_node` unlinking without freeing (the libxml fork frees only doc-less
    orphans) — pgfsys's transient `svg:g` groups, eager and streaming alike; n8's C-live
    is flat at 11 MB now (was 11 → 47). The manual itself now streams to completion at
    the 6 GB ceiling (0 fatals, 3,326 pictures, peak 4.78 GB, 379 s). Sweep #103
    (56dy): six former fuse fatals complete through the 56dw restart, the manual at
    418 s — 2 s inside the cap. 56dz added the pre-fuse watermark (stop at 2/3 of the
    fuse, no Fatal, rerun with the remaining allowance): n8 peak 784 → 522 MB, wall
    33 → 28 s; but the manual's streaming rerun ALONE is ~380 s, so after a ~40 s eager
    start it still overruns 420 s. Remaining levers for that witness: streaming speed
    (PLANS 12) or a sweep-cap policy for restarted documents; sweep #104 measures the
    watermark corpus-wide. Part four LANDED as
    56dw: the CLI restarts a fused eager conversion under `--streaming` (the fuse itself
    is the signal — no new threshold, documents that finish eager are untouched); the
    sweep converts through the CLI, so a streaming win now reaches the verdict for any
    document whose streaming peak fits the 6 GB ceiling (the pgf manual does not yet:
    part three). The `cortex_worker` restart LANDED as 56ea (`streaming_restart.rs`
    shared by both binaries; thread-local stop signal; watermark at nine tenths of
    the fuse after sweep #104 showed two thirds cost 330 s for no verdict); the
    sandbox reruns (11a) validate it against the fleet's ceiling. Part five SETTLED
    2026-09-19 (agent, heaptrack): the constant ~37 MB per in-process conversion is
    the fresh test thread's `#[thread_local]` engine roots (`MODEL`, `GULLET`,
    `STOMACH`, token constants, the reset interner) — Rust memory that the test binary
    counts on glibc only because it links no mimalloc; the C heap's only residue is
    kpathsea's 8.6 MB ls-R/cnf tables, once per process; the libxml document tree is
    fully freed. `cortex_worker` reuses its thread and pays neither per paper. The real
    fleet lever is different and OPEN (MED-HIGH): per-paper State additions
    accumulate because the daemon-frame port (`state.rs` `push_daemon_frame`, Perl
    State.pm:607-627) is unused — wrap each paper's convert in push/pop. Secondary (LOW, output-neutral): Perl-parity refcount pruning of
    `node_boxes` in the eager path (`Document.pm:1667-1669`; Rust's
    `document.rs::sweep_stale_node_boxes` is streaming-gated) — ~340 MB on the manual.
    Dead ends: fewer `svg:g` (already 0.34×), draining during Build (too late),
    a leaner `Whatsit` (already niche-optimized, issue #361 M4).

11a. **Sandbox reruns as the regression oracle (user 2026-09-19).** With the memory
    batches landed, rerun `sandbox-arxiv-2605` and `sandbox-arxiv-2606` on cortex with a
    `cortex_worker` built from this branch (the containerized recipe: `docker build
    --target worker` from this repo's `Dockerfile`, `docker run --network host` with
    `WORKERS=72 PROFILE=ar5iv`, then `POST /api/reports/<corpus>/oxidized_tex_to_html/rerun`
    with the claude-session token, 2605 then 2606; judge only `todo == 0 && queued == 0`
    runs against the frozen baselines, by category shape and `/api/runs/<c>/<s>/diff`).
    Baselines at the time of the directive (both complete): 2605 = 30,079 docs,
    no_problem 6,155 / warning 19,772 / error 3,894 / fatal 258; 2606 = 30,426 docs,
    no_problem 6,463 / warning 19,742 / error 3,984 / fatal 237. The reruns tell whether
    the goal is reached without regressing other documents; sequenced right after 56ea
    (the worker's own restart), before 12.

12. **High-performance raw interpretation (user directive 2026-09-19: "let us do it
    soon, but do not change next steps, add after").** Queued AFTER 11's open parts
    (the pre-fuse restart watermark / cap for restarted documents, the `cortex_worker`
    restart). Why: raw interpretation is not uniformly slow — pure TikZ manuals run at
    ~1.1× pdflatex (pgfmath/pgfsys native leaves) — but token-churning macro code runs
    5-6× slower (tikz-network raw 190.7 s vs pdflatex 34.5 s before the datatool
    natives), and the last slow-call profiles put the remaining time in the gullet's
    generic reading (`read_keyword`/`read_x_token`/`read_dimension`) with no native leaf
    left. pdfTeX's advantage is tex.web's data model, not cleverness: a token is one
    packed integer (an `eqtb` index for a control sequence, hashed once at
    tokenization), macro expansion pushes an input level INTO the existing token list
    (no copy), `\edef`/`\csname` build cells from a free list, registers and
    conditionals are `eqtb` stores replayed by the save stack, and nodes are freed as
    pages ship. Our likely costs, by inspection (NOT yet measured): interned-symbol
    tokens with lookups on every `\csname`/meaning read, expansion that clones token
    vectors instead of walking in place, ~2.4 KB per `Rc<DigestedData>` box (Perl
    0.7 KB), and a whole-document box tree until the build. Method (per
    [[feedback_perf_algorithmic_not_memory_tangles]]): a symbolized `--profile bench`
    profile on a token-churning witness (datatool-user or glossaries-user with
    `LATEXML_DATATOOL_NATIVE=0`, plus a plain-macro benchmark) that ranks the three
    costs — lookup, expansion copying, box allocation — then ONE tex.web-shaped lever
    per run with pre-registered bars (packed tokens / in-place expansion /
    `eqtb`-style hot state), readability of gullet-stomach-mouth as a review criterion.
    **Interface constraint (user 2026-09-19):** this codebase will be maintained by
    LaTeXML developers who know the Perl Gullet/Mouth/Stomach/State abstractions and
    ergonomics — any representation change (packed tokens, in-place expansion, hot
    state) must stay behind the Perl-shaped interfaces (`readToken`/`readXToken`,
    `unread`, `Tokens`, `lookupMeaning`/`assignValue`, mouth push/pop) so the code still
    reads like the Perl to them; a faster core is not allowed to turn the gullet into
    something a Perl LaTeXML developer would not recognize. This is the only remaining
    throughput lever with corpus reach (P5's successor).

13. **Generalize the sandbox-regression fixes into kernel capabilities (2026-09-19, from the
    2605/2606 rerun; user: "take detailed notes, add to our PLAN, proceed to improve").** Batch
    56eb fixed five regression clusters concretely; each is an instance of a general kernel
    weakness that should become a capability/guard so the class cannot recur (notes:
    `~/data/pk_agents/w23/regress_2605/{CLUSTERS,KERNEL_LESSONS}.md`,
    [[wisdom_sandbox_regression_kernel_lessons]]): (a) **driver-surface completeness** — a probe
    sentinel defined for `\@ifundefined` (hyperref `\hyper@makecurrent`, 55b) must define its
    whole companion family or none; audit every driver-probe `def_macro_noop`; (b) **defining
    primitives must define, never noop** — `\newbibmacro`/`\newcommand`-like/`\NewTaggingSocket`
    must create a stub target so downstream `\patchcmd`/introspection succeeds (biblatex-ieee, 53
    papers); (c) **beyond-Perl `Until:` captures need a guaranteed sentinel or a structural bound**
    (never consume `\end{document}`) — DEFERRED: the jmlr `\addr` global-no-op + `\let`-scoped-capturer
    attempt collapsed multi-author affiliation capture (colt 2→1, midl 7→1) under `\let`; the 56eb
    per-class wrappers cover the current corpus, so the durable bound needs a `Until:`-reader
    EOF-recovery design instead of a `\let` dance; (d) **OmniBus recovers derived-class author wrappers**
    (`\Xauthor`→`\author` for the whole jmlr/pmlr family) instead of a binding per class — the
    56ed concrete additions `l4dc2026` (2605.22207) and `neus2025` (2605.05795), both identical in the load-shape to colt202x (`\LoadClass[pmlr]{jmlr}` + `\newcommand{\coltauthor}`), are two
    more registry aliases the general recovery would subsume; (e)
    **`try_spawn_or_degrade`** at every thread spawn (graphics fixed; OPEN: `latexml_oxide.rs:757`,
    `cortex_worker.rs:1729` prewarm, `render_workers`, `api`) — EAGAIN must degrade, never panic;
    (f) **raw-loader bindings carry an idempotent `\providecommand` native fallback** for their
    public macros (tipa; trimmed-host + portability). PROCESS: run the sandbox reruns as a
    RECURRING regression oracle — these sat undetected for weeks (batches 55b/7/51-52).
    (g) **a raw-TeX binding must not silently drop a Perl semantic guard** — batch 54 replaced
    Perl's native `\g@addto@macro` (a `DefMacro`+sub carrying `AddToMacro`'s expandability check,
    Package.pm:2534) with a raw `\long\def` to dodge the numspell `\ifnum` look-ahead, but the raw
    `\def` dropped the guard, so appending to the `\normalsize` font-switch *primitive* built a
    self-reference (`recursion:\normalsize`, ~6 papers). Fixed 56ed by rebinding as a non-expandable
    `DefPrimitive`+`AddToMacro!`: it halts the look-ahead like `\begingroup` did AND restores the
    guard. Lesson: a State side effect belongs at stomach level (`DefPrimitive`, per the `\newif`
    precedent); when a raw-TeX shortcut is chosen for one property, re-audit which Perl guard the
    native binding carried that the shortcut loses.
    LANDED: (e) `try_spawn_or_degrade` (56ec, `latexml_core::util::thread`, both kpathsea-prewarm
    sites; the graphics drain in 56eb); (g) `\g@addto@macro` primitive+guard (56ed). NEXT: (b)
    defining-primitives, (a)/(f) audits, (d) OmniBus author-wrapper; revisit (c) as a `Until:`-reader
    EOF-recovery.
