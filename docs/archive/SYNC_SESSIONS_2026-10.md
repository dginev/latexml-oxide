# Sync sessions — 2026-10

Narratives moved verbatim from `docs/SYNC_STATUS.md` (2026-10-04 compaction): the cortex sandbox validation (61r), the
stop and resumption of the full arXiv run 329 with the 62a-62c fixes, and the run-329 cluster batches 62d-62f. Their
open residuals stay in SYNC_STATUS "Run-329 and sandbox open residuals". Also the rows removed as landed.

### Cortex sandbox 2605/2606 validation (61r, 2026-10-04)

Reruns on worker 61q (runs 325/326) against the last complete runs (324/322): 2605 errors 2,865 → 2,730, fatal 77 → 73;
2606 errors 2,927 → 2,751, fatal 100 → 95; 441 papers better, 119 worse by status. A paired container A/B
(`latexml-oxide/cortex-worker:56il` vs `:61q`, same box, `~/data/pk_agents/main/pair_ab_one.sh`) left 48 truly worse
(70 were fleet noise: `never_completed_with_retries`, retries). Engine check (`tex_errors.sh`, the paper's own
compiler): 3 faithful (pdflatex errs too: 2605.24084's `\dot@spacing` outside `\makeatletter` redefines `\dot`;
2605.05677, 2606.15990). Fixed by 61r: the box/alignment closer on any catcode-2 character (2606.11726: 251 → 115,678
words converted), `\setlength` read as one stream (6 papers), versioned-package fallback run once (2606.14467), pgf `@`
arithmetic on the sp grid (2606.26406, 2508.07407; KPE #461), bindings that scanned what their package never scans
(subcaption's `{subfigure}` signature, soul's `\setul`, amsmath's `\\[…]` a plain glue scan). Fixed by 61t: `\pgfmath@smuggleone` smuggles its whole argument (KPE #464;
2606.15113). Fixed by 61u: a `\NeedsTeXFormat{pLaTeX2e}` document gets plcore.ltx's registers
(`\Cht`…`\cHT`; sweep #148's 30 pLaTeX canaries, +1 error each since 61r's `\setlength`). Fixed by 61v (user ruling 2026-10-04):
`\raisebox` reads its raise with the box set and `yoffset` renders as `\raise` (KPE #466; 2606.06643 ×54, 2606.05044 ×10,
2605.28956, 2606.06288 → 0). Cortex reruns on worker 61u (runs 327/328 vs 325/326): 2605 fatal 73 → 68, errors 2,730 → 2,711;
2606 fatal 95 → 83, errors 2,751 = 2,751; no new Fatal cluster. Of the newly fatal, 10 were fleet
`never_completed_with_retries` and 10 `TooManyErrors` that convert identically (and nearly clean) in both images
standalone (run-environment noise); one real panic, 2605.18869 (61s's tabularx `X` met by an `\edef` outside a preamble,
`with_current_build_template` unwrap), fixed in 61v (`with_building_template`). Fixed by 61s: a bare `\array…\endarray`
gives `$` back (KPE #462; makecell in a math cell; 2606.05500, 30 errors and a Fatal → 0); a paragraph column's width
is read in its cells (OD #437; 2606.15832's `p|` column no row reaches); inside tabularx `X` is tabularx's own column
(2606.05563). Also open: a tabular
`\\[<len>]` with a non-positive length is `\setlength`-calcified in latex.ltx (:16586-16602, `\@vspace@calcify`) but read
by TeX's scan here; latex.ltx 2025's `\vspace` is calcified too (:9254/9362) while ours is `\vskip#2\relax`. Open, each pdflatex-clean, ids in
`~/data/pk_agents/main/hostb_61r4.tsv`:
  - calc's `!` protocol (`\calc@next`, calc.sty:52/147/153): linegoal's `\LNGL@set!` runs inside the expression
    (2605.05695).
  - Pre-broken papers now Fatal: 2605.05881 (`\pgfutil@emu@@unpack` Mouth EoF; 83 errors on 56il), 2605.10232 (tikz-cd
    cells holding `tblr` tables with `\&`; PushbackLimit since 57cp; 23 errors on 56il).
  - Latent panic: `gullet.rs` `read_token`'s `handle_template(data.borrow_mut(), …)` (RefCell already borrowed), Fatal
    on 2606.05500 under both images until 61s (KPE #462) removed the makecell cascade that reached it; the full paper
    on `latexml_oxide.61r6-rel` still reproduces it, no truncation of it does.
  - `runtime-bindings` builds only: a versioned name whose stripped name has only a runtime `.rhai` binding is found by
    loading it (`find_file_fallback`), without the document's options; the caller's load with them is then skipped.
    Fix: ask the rhai tier whether `<name>.rhai` exists instead of loading it.
  - Singletons (1-8 errors each, 56il-57ck window unless noted): Missing control sequence inserted (2605.26277,
    2606.00866, 2606.20186); listings input files not found (2605.15569, 2605.12863); delimited user macros
    (`\edef\__figfile` read through a neutralized Semiverbatim, KPE #465, 2606.13798; `\def\\ell` reaching
    authblk's `\affil` text, which TeX typesets where `\\` is local — authblk.sty:156, `\@maketitle`'s tabular —
    2605.28517; `\\` Match:X in neurips' author tabular, minimal shape clean, 2605.12000); 2605.04969 (sectioning in a box, 59j); 2605.07684 (`\ContinuedFloat` after a table); 2605.10068 (`#`
    reaches the stomach, 61m); 2605.11285, 2605.31585 (58h counter steps: `\the\cmdKV@…`, `\iffirstchoice@`);
    2606.15122 (utf8 keyboard character, 58q); 2605.19122 (`_Capture_` close); 2605.29722 (`_` outside math);
    2606.30845 (`\capitalizethefirst`); 2606.11726 (2 undefined counters in a plain-TeX paper now converted whole).

### Stream G stopped: run-329 regressions (62a, 2026-10-04)

The full arXiv rerun (cortex run 329, worker 61v) was stopped by the user after 16,333 papers. Against run 306's
statuses, 134 went worse into error or fatal (1,481 better; 1,802 clean → warning; the 200 invalid unchanged). The
remaining tasks stay paused until a fixed binary passes a new validation. Ids and per-binary counts:
`~/data/pk_agents/main/scratch_g/run329/` (`reg134.tsv`, host runs `reg134_<binary>.tsv`, image bisect
`res57_bisect.tsv`, engine oracle `res73_tex.tsv`). Fixed by 62a (on the host, 46 of the 134 → 0 errors; 22 more convert
clean on the host under every binary, a difference between the fleet's distro TeX Live and the host's 2025 tree).
Validation (`val/`): 400 random run-329 papers, 61v4 → 62a17, no status worse, 3 better, errors 170 → 156; arXiv A/B
61v4 → 62a14 (3,003 2605 papers): no status worse, 10 papers with fewer errors, fatals, words, bibliographies and
tables identical, −0.8 % time:
- A plain TeX document stays plain. latexml.sty's `\AddToHook{file/l3backend-dvips.def/after}` autoloaded the LaTeX
  format for every document, so a plain one got LaTeX's `\end`, and its closing `\end` raised "`\endgroup` Attempt to
  close a group that switched to mode vertical" (27 papers: 0911.4241, 1001.3079, 1312.4456, 1407.6634, hep-th9310069,
  chao-dyn9706006). `converter.rs` `compile_route` reads how arXiv's AutoTeX compiles the main file: LaTeX iff
  `\documentclass`/`\documentstyle`/`\begin{document}` or a command plain TeX cannot run (`\usepackage`,
  `\RequirePackage`, `\include`) appears outside comments (CR/LF/CRLF lines, `filecontents` bodies skipped, up to
  `\endinput`), in the main file or a file a class-less main file inputs (an input name only TeX resolves counts); `state::plain_tex_document`
  carries it and the hook is installed for LaTeX documents only. Every converter is told its source before its session
  (`Converter::note_main_source`; streaming restarts and supplements included) and one never told is LaTeX, PDF. Plain
  `\eject`'s page count advances plain's `\pageno`, and `\c@page` only once LaTeX is loaded (both `\count0` under the
  dumps; registers of their own on the NODUMP branch; guard
  `perfect_kernel_batch61::clearpage_advances_the_register_c_at_page_names`). A plain paper's ids are now LaTeXML's
  plain ones (`id1`…, not `p1`…), so its HTML anchors change.
  Bindings a plain document loads no longer call LaTeX-only commands that autoload the format mid-load, after which the
  latex dump replaces what the earlier packages defined: amsfonts' `\DeclareSymbolFont` (LaTeX only) and graphics'
  `\providecommand\Ginput@path` (graphics.sty:157's `\ifx` test): AMSTeX's amsmath `\cases … \endcases` survives
  (1409.5819, 89 errors on 62a5 → 0; Perl 1). Guards `perfect_kernel_batch61::plain_document_stays_plain`,
  `perfect_kernel_batch61::amstex_cases_stay_amsmath`. A scan of every binding `\input` from a plain document
  (`~/data/pk_agents/main/scratch_g/bscan/scan.tsv`) finds 56 more that autoload, all LaTeX-only packages (hyperref,
  natbib, babel, aastex…, which plain TeX cannot load either); the generic idioms (`\input tikz`, `xy`, `epsf`,
  `pstricks`, `miniltx` + graphicx, harvmac, phyzzx, amssym) do not.
- A Semiverbatim argument's pre-expansion reads definition names unexpanded (tex.web §1215; `parameter.rs`
  `take_definition_names`): a font switch's `\edef\f@series{…}` had its name expanded to the letter `m`, and a `\def~`
  lost its active `~` to neutralizing ("Missing control sequence inserted"; 1212.6174, 1303.4395, 1711.09355,
  1011.4121); a `\def`'s parameter text and body are kept too (§473), as are LaTeX's `\newcommand` family's
  arguments (`\renewcommand\textit` in a JHEP3 `\href` text went Fatal on 61v4). Perl expands and neutralizes them too
  (Parameter.pm:89/124-133; its amsart `\urladdr` goes to a macro's Semiverbatim parameter, which Perl never
  pre-expands, so that case is clean there): OXIDIZED_DESIGN_DIVERGENCES #439. Guard
  `perfect_kernel_batch61::semiverbatim_definitions_keep_their_names`.
- A class shipped with the source and run as OmniBus has its `\LoadClass`/`\RequirePackage` lines scanned (Perl
  `maybeRequireDependencies`, Package.pm); an option naming one of the class's own macros stays inert, as in Perl
  (`content.rs` `scanned_options`): easychair.cls's `\LoadClass[\@PaperFormat,…]{report}` raised "undefined" for each
  (2011.11995, 2211.09353, 2607.12736). Guard `perfect_kernel_batch61::scanned_class_options_naming_macros_stay_inert`.
- `\textcircled`'s argument is typeset as text in math too (omsenc.def:62-64's `\ooalign` box; `sect13.rs`), so
  `$\textcircled{$C$}_1$`'s inner `$` opens a formula (1009.5713, 53 errors → 0). Rust-only since its argument is
  digested (KPE #361). Guard `perfect_kernel_batch61::textcircled_argument_is_text_in_math`.
- A `\documentclass` option naming a DVI driver selects DVI output (OXIDIZED_DESIGN_DIVERGENCES #285; 0908.4150's
  `[12pt,dvips]`, l3backend's "Backend request inconsistent with engine"). Guard
  `perfect_kernel_batch61::dvips_class_option_is_dvi`.

Open (55 of the 134 still err on the host with 62b, 66 with 62a; image bisect over 0.7.6 → 56ea → 56il → 61q → 61v, the fleet's
environment; most regressed between 0.7.6 (2026-08-23) and 56ea). By the intended engine (TL 2025):
- Fixed by 62c (70cf545b8a; user ruling 2026-10-04, OXIDIZED_DESIGN_DIVERGENCES #441): the 16 ACL/EMNLP papers on
  acl.sty's "patch failed" (acl2019.sty:455 `\patchcmd\@combinedblfloats…`, an output-routine internal TL 2025's
  latex.ltx no longer matches) convert clean: a `\patchcmd` miss on the output routine succeeds without patching.
  Fleet-environment validation of `cortex-worker:62c` on the 527 (`run329/docker_val_62c.tsv`): errors 590 → 573,
  Fatals 5 = 5, the only changes 17 ACL-era papers 1 → 0; none worse than 61v or 62b. **Run 329 resumed** on it
  (container `cortex-worker-62c`, 2026-10-04 17:46Z, 2,891,776 tasks back to TODO); 61v's 1,780 Error and Fatal
  results were set back to TODO at 17:54Z for 62c to reconvert (`run329/rerun_ef/rerun_ids.txt`). Of the first 1,708
  reconverted, 1,591 already failed before run 329 (1,574 still do: long-standing, not regressions); of the 117 that
  were clean or warning then, 68 are back, and the 42 still in error or fatal are all among the 134 (the open list
  below; `rerun_ef/still_worse.tsv`, `crosstab_0917.tsv`).
- Faithful, pdflatex/latex errs too: 2105.00771
  (its own `\bbl@set@language` patch, 101 errors in both), 2203.12702 (acro property, 26 = 26), 1907.05651, 2011.07134
  (ctex fontset), 2105.03193, 2203.12692 (`\ContinuedFloat`), invalid UTF-8 (1309.3357, 1409.4967), classes or
  styles missing from TL (elsart, aipproc, psfig, citesort, tcilatex).
- Fixed by 62b (each with its `perfect_kernel_batch61` guard): `\emph` in math typesets its argument as text
  (OXIDIZED_DESIGN_DIVERGENCES #440; 2502.18190); verbatim.sty's `{comment}` after comment.sty wins, so an indented
  `\end{comment}` ends it (2607.07115, 2607.23269); the longtable binding turns acro's `patch/longtable` off (2310.14606,
  2606.11983); the native `\sf@subfloat` survives caption3's subfig patch (2003.01262); the in-cell `\\` is the
  kernel's `\lx@newline`, not a column-redefined `\newline` (1901.05279); the math parser's eqnarray column pair is
  clamped (2211.01040 panic); and four Perl-origin defects, KNOWN_PERL_ERRORS #467 `\cline` (1902.04834), #468
  `\caption`'s `[short]` (cond-mat0307356), #469 `oneside`/`twoside` (hep-ph0207204), #470 local shorthands
  (2003.08372). Residuals there: hep-ph0207204's raw `\thebibliography` (70 `bibitem isn't allowed`), 2003.08372's
  `_` in a `.bbl` URL (2). Validation: 400 random run-329 papers, no status worse than 61v4; arXiv A/B 61v4 → 62b3
  (3,003 2605 papers): 0 worse, 2 better, 12 with fewer errors, fatals 3 = 3, 2605.08004 5k → 48k words (its comment
  body), the 9 `tex=` changes intended (`\emph` in math as text, a `\\` reversion, emph text keeping its spaces).
- Fleet-environment validation of 62b (docker images `cortex-worker:61v` vs `:62b`, 3380ec483c, on the 134 plus the
  400 random run-329 papers, `run329/docker_val_62b.tsv`): errors 2,170 → 590, Fatals 15 → 5, 59 statuses better, none
  worse. Run 329 stayed paused for the ACL ruling (62c, above).
- `\jobname.aux` re-read: lamuphys.sty:1240-1247 and caosp.sty:916 redefine `\enddocument` to `\input \jobname.aux`
  under `\if@filesw` (true in Perl and Rust), a file pdflatex has written by then and LaTeXML never writes:
  `Error:missing_file` (cond-mat9607109, astro-ph9805185; 0.7.6 clean).
- Still Fatal after 62b, each erring under pdflatex too: 1001.1670 (pdflatex "Paragraph ended before …", 7 errors;
  Rust reads the delimited argument to the end of the file: no tex.web §392/§396 check, so a non-`\long` macro's
  argument crosses `\par` silently — `\def\x#1/{[#1]}` then `\x a⏎⏎b/` gives "[a", a paragraph, "b]" with no error;
  Perl alike; implementing it adds pdflatex's errors to papers that convert silently today, so it waits for a
  ruling), 1409.3401 (pdflatex emergency stop), 2105.00771 (101 = 101), cs0702042 (citesort.sty missing from TL; Rust
  then 101 math-alignment errors, pdflatex 3).
- Rust-only, still open: CJK GB `\@inpenc@undefined` (1007.1512); singles 1205.5844 (`\@journal`), 1711.06710
  (`\@rticle@options`), 1811.00686, 1907.03566, 2004.12109 (memoir font command), 2105.02164, 2203.13766, 2207.02360,
  2306.10394, 2409.00304, astro-ph9805185 and cond-mat9607109 (`.aux` input).
- AMSTeX `\documentstyle{amsppt}` documents still get the LaTeX format (Perl reads them plain). Kept plain they measured
  worse: 64 sampled 1996-99 amsppt papers, 192 → 319 errors, 1 → 2 Fatal (math9806005 0 → `TooManyErrors`; plain
  AMSTeX lacks `\ams@return@opt@arg`, `\rightpoint`; math9603201, math9709201, math9801043, math9803037). Fix the
  plain-AMSTeX gaps first, then classify them plain (`converter.rs` `compile_route`); sample and A/B in
  `~/data/pk_agents/main/scratch_g/run329/amsppt/`.
- `\input eplain` (pre-existing, tex clean): `Error:undefined:\auxfile`, then `Fatal:ParamSpec` "Parameters for `\@`
  not in order" (Perl: the one error).

### Run-329 long-standing error clusters (62d, 2026-10-04)

First-error clusters of run 329's 62c results (`~/data/pk_agents/main/scratch_g/run329/clusters62c/`), each already in
error before run 329. Fixed by 62d (guards in `perfect_kernel_batch61`, 62d block); witness probe 62c1 → 62d5, none
worse:
- arximspdf/arxstspdf (arXiv's IMS classes, Rust-only binding): the no-op list's `"printead*"` was the prototype
  `\printead` + a literal `*` and replaced the plain one ("Missing argument Match"); `{pf}`/`{pf*}` mapped to an amsthm
  `{proof}` the class never loads (now a run-in "Proof." ending in the class's automatic □, `\noqed` honoured);
  `\upqed`, `\tablewidth`, `\tabnotetext`/`\tabnoteref` (explicit marks kept), `{sidewaystable}` (the `rotating`
  option) and arxstspdf's `\doiurl`/`\arxivurl` were undefined. 31 papers: 148 → 26 errors, 25 better (1107.4843,
  1205.6055, 1409.7256; 0903.0664 5 → 0, 1104.3398 11 → 1).
- amsmath's `\mathaccentV{hat}05E{D}` (amsmath.sty:754-831), the written-out `\hat` in revtex bibnote `.bbl` files,
  reads as the named accent: 16 papers, all clean (2008.11212, 1309.7027; undefined in Perl too).
- ragged2e's `\justify`/`\endjustify` commands defined, and the `justify` environment is a grouped paragraph of its own
  (it ran into the text before and leaked its fonts): 14 of 18 better, 24 → 7 errors (2204.13885, 1903.04078;
  2406.15288 unchanged, 39,974 words).
- INTERSPEECH2021/22/23.sty load their packages (graphicx, amsmath, bm, booktabs, caption …) and bold `\vec`/`\mat`
  (`interspeech_sty.rs`) instead of binding as bare spconf: `\includegraphics` defined, 12 of 18 better, 236 → 83
  errors (2103.14512, 2211.09381).

Fixed by 62e (KNOWN_PERL_ERRORS #471, shared with Perl; guards `perfect_kernel_batch61::algorithm2e_*`):
`malformed:ltx:listingline` was algorithm2e's line machinery splitting lines it cannot reach. `algorithm2e_sty.rs`
`line_reach`: a line is reachable only through inline wrappers and the wrappers the document opened directly in it;
behind anything else (an item or list, an equation, a minipage or `\vbox`, a table cell, a footnote) a split is a
`<ltx:break/>` in that box and takes no line number, and a display list's end ends the line (`\lx@algo@listend` on
`env/<list>/after`); statements in a plain float (`[algo2e]` + algorithm.sty's `\newfloat{algorithm}`) open an
auto-closing listing; `\par` in restricted horizontal or math mode ends no line, `\\` there is the kernel's break (a
caption's `first\\second`, 1412.0600), and a `\parbox`'s `\\` breaks its own text. 63 cluster papers + the 19 code-comment witnesses (62d5 → 62e6): 789 → 127
errors, 57 better, none worse, no word lost, no internal macro in a `tex=`; lists stay whole where 62d5 threw their items into listinglines (2405.11213, 1410.4772), and the lines after a list
stay lines (2203.03384, 1807.02449); 2406.10356 50 → 0, 2010.03983 1 → 0. Root cause: `~/data/pk_agents/main/agent_reports/
2026-10-04_listingline_algorithm2e.md`. Open, pre-existing (review probes `scratch_g/review62e/r3/r4.tex`, `r5.tex`): with
`linesnumbered` the first statement right after `\caption` takes no number; `{quote}`/`{center}`/`{flushleft}` end no
algorithm line, so the text after them stays on their line; a statement inside `\href` breaks inside the link
(`ltx:ref` does not auto-close). Not fixed: missing journal classes (raa `\pagerange`/`\volnopage`, ws-* `\bodymatter`,
iopart `\ioptwocol`, old A&A `\thesaurus`), text-mode `_`/`&`/`^`, and the Fatals `TooManyErrors` (49),
`never_completed` (24), `PushbackLimit` (21 then, 86 by 126k papers; fixed for harvmac by 62f below).

### Run-329 Fatal clusters (62f, 2026-10-04)

`Fatal:Timeout:PushbackLimit` (86 papers of run 329's first 126k; 82 Fatal before run 329 too):
- Fixed by 62f (RUST-ONLY): 64 are plain TeX papers on harvmac (with tables.tex). `\hphantom`'s brace peek
  (`math_common.rs`, the Rust-only guard for 2004.10048/2508.13557) was LaTeX's `\@ifnextchar`, so in a plain document
  it autoloaded the LaTeX format and its dump replaced the document's own macros: harvmac's `\ref`, after which
  `\refs` (harvmac.tex:186-191: `\hphantom` then an `\edef` of the label `\lref` defined as `\ref\X`) re-expanded the
  label forever. `\lx@hphantom@peek` peeks natively (as Perl, whose `\hphantom` has no peek, converts them clean).
  The 86 + 3 `\hphantom` witnesses (62e10 → 62f1): Fatal 81 → 17, 64 better (57 of them 0 errors), none worse;
  2004.10048, 2508.13557, 2605.21158 unchanged.
  Guard `perfect_kernel_batch61::hphantom_in_plain_tex_loads_no_latex`; root cause and repros
  `~/data/pk_agents/main/scratch_g/rc_harvmac/`. Side risk, open: any genuine `\@ifnextchar` autoload in a plain
  document still replaces the document's macros through the dump apply.
- Open, 17 still Fatal after 62f: csvsimple's `\csvloop` with a siunitx `S` column loops (2108.13640; repro
  `scratch_g/g62f/b2.tex`: the witness CSV, `tabular={lS}`, `command=\texmodel & \mae`); 2302.07191, 2501.17908,
  2509.10120 (error before run 329, Fatal now); unclassified: 1007.3028 and 0902.2281 (`\W@` undefined first),
  1807.10890, 2305.06365, 1807.08405, 2407.10582, 2107.07104, dg-ga9410001, 2505.05474, 2408.12869, 2309.08676,
  2502.21053, 2406.19307. A `\noexpand`-ed undefined control sequence raises "undefined" in csvsimple's
  space trim where TeX treats it as `\relax` (`g62f/c_m.tex`, a CSV cell `\foo` with `\foo` undefined).


## Rows removed as landed (2026-10-04)

- **`newunicodechar` four-hex `^^^^` caret support** (~119 docs, e.g. `2606.00241`): Adding 4-hex/6-hex caret parsing to `mouth.rs:get_next_char` allows `newunicodechar` to take its Unicode branch cleanly.
- **A latexml.sty preload loads the LaTeX format before the document** (56jl triage; RED repro `tools/perfect_kernel/repros/loader/latexml_preload_keeps_plain.tex`): its body's `\AddToHook` (the dvips pagecount hook, `latexml_sty/mod.rs`) autoloads LaTeX.pool (`latex_kernel.rs`), as do `\ExplSyntaxOn`/`\NewDocumentCommand`/`\lua_*`/`install_unicode_format_encoding` under the luatex and xetex profiles. Right for a LaTeX document (pdflatex's model: `\newcount\foo \foo=7 \documentclass…` keeps 7, ar5iv's locked `\today` survives), wrong for a plain TeX one (`\@latexerr` defined: pstricks.tex:39-41 skips pstricks-tex.def, 3 errors in `graphics-tikz/pstricks_input_plain` under the raw preload). Deferring the hook to the "format loaded" seam alone moved every LaTeX document's format load to `\documentclass` (as Perl): ar5iv's `\today` printed the conversion date and pre-`\documentclass` assignments were lost to the dump replay — reverted. A faithful fix needs both: the lazy load must not clobber assignments made before it (a general capability; Perl and no-preload Rust lose them too), and ar5iv's `\today` at begin-document (Perl ar5iv.sty.ltxml). Side finding: the luatex/xetex profiles pick `l3backend-dvips.def`, because the profile's `\ExplSyntaxOn` raw-loads expl3.sty, whose `\sys_load_backend:n{}` (expl3.sty:153-154) runs before the engine identity is set (lualatex: l3backend-luatex, xelatex: l3backend-xetex) — needs a RED repro.
- **DefMath in text mode (57b review): LANDED 57n** — every DefMath constructor warns `unexpected:<cs>` outside math (Package.pm:1706), the text-safe symbols being boxes (`has_complex_option`, Package.pm:1603-1607). Guards `perfect_kernel_batch56::text_mode_defmath_constructor_is_reported`, `06_cluster_regressions::cluster_defmath_textmode_no_mode_warning`.
