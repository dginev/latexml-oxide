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

## Moved 2026-10-23: batch and integration narratives 63a–int73

Moved verbatim from `docs/SYNC_STATUS.md` (2026-10-23 compaction, user request): the "Run-329 and sandbox open
residuals" and "Run-336 open residuals" sections with every batch and integration paragraph appended to them since
2026-10-04. Their open residuals are lifted into `perfect_kernel/OPEN_ROWS.md`.

### Run-336 open residuals (63a, 2026-10-07)

Mined from run 336's first ~35k papers (post-fix reconversions on the fleet worker; the env-gap papers are re-queued at
completion, row G). **63a** (landed): the `Fatal:Stomach:Recursion` cluster's fixable roots — arxbj `{longlist}` as the
class's own `\list` (KPE #517), etex's register-block allocators (KPE #516), the autoref tag's `~` bound to the kernel
space where it is digested (1601.02132), and arxbj's IMS markup through a binding shared with arximspdf
(`ims_support`); guards `perfect_kernel_batch63::*`. Open:
- **65c** (landed; RUST-ONLY): the content of environments left open at `\end{document}` (and inside a runaway
  `\abstract{…` ended by a section) is kept: every body digesting inside an abandoned group ends as at end of input
  with Perl's dummy trailer (stomach.rs `abandon_stack_frame`; Stomach.pm:111,130), where the trailer split used to
  take each enclosing body's last real box (the inner environment, or the rest of the document). The `ltx:document`
  close is deferred to the end of the build so every still-open element's close hooks run (`close_abandoned_document`,
  OD #207); the warning prints Perl's detail lines (initiator + locator) and closes and reports open conditionals; the
  end-of-document internals revert to nothing in `tex=`; `cleanup_math`'s real-math test is a linear walk
  (`has_real_math`; 1907.09124 265 s → 5 s). Witnesses powerdotDE (recall 26.3 → 97.8%), csvsimple-legacy (88.9 →
  99.6%), 1907.09124, 1902.00853. Guards `perfect_kernel_batch65c::*`. The regained content of arXiv papers whose
  environment or math an upstream defect leaves open surfaced as `malformed:*` placement errors; since 65i the open
  construct gives way to the structure after it (OD #542: 1507.01637 27 → 25; hep-lat9811036 stays `TooManyErrors` on
  its own alignment errors; 2110.01778's cause is RED `boxes-groups/boldsymbol_text_dollars_inside_math`).
- **64zf** (landed; diagnostics honesty, stage 1 — user directive 2026-10-11): Error/Fatal emitted at Perl's
  severity where Perl reports, and where only TeX errs a Warning when the recovery yields good markup, else an Error.
  `.bib` undefined macros at the entry's line with the `.bst`'s `MACRO`s (OD #518), overflow clamps (OD #519),
  setspace switches as checked no-ops (OD #520), listings begin-line text (OD #521), an unknown colour model, the
  open conditional at `\end{document}` (65c's report, guarded here), Perl's `Info:misdefined:utf-8`, picture `\line`
  outside a picture, booktabs widths, `\IEEEQEDhereeqn` in a row, the CLI's unusable inputs as a Fatal with a log;
  addenda OD #207/#316/#317/#481/#489; KPE #600-#602. Guards `perfect_kernel_batch64zf::*`,
  `119_final_status_report::unusable_archive_is_a_fatal_conversion`. Open REDs: sectioning-frontmatter/
  section_title_errors_reported_twice (task: the toc digestion re-raises the heading's diagnostics, shared with
  Perl), loader/plain_input_defining_documentstyle_routes_plain (hep-lat9301012), babel-lang/
  shorthandoff_quote_in_english_is_no_shorthand, expansion-primitives/uppercase_brace_hunt_file_end,
  alignment/tabularcalc_pcell_ifnum_cr, and the setspace REDs boxes-groups/{lrbox_doublespace_group_end,
  abstract_begingroup_doublespace_group_end, plain_group_around_itemize_doublespace,
  doublespace_end_in_a_nested_plain_group}. Stage 2 (the remaining Error/Fatal classes) is open.
- **65d** (C5 step G2; OD #530, KPE #607): `\left`/`\middle`/`\right` are TeX's unexpandable `left_right` command —
  `\left` the constructor Perl has, `\middle` a primitive dispatching to `\lx@delim@middle` — with tex.web §1160
  `scan_delimiter` (`tex_math::scan_delimiter`: `get_x_token` past blanks and `\relax`, a delimiter by meaning — a
  `\delcode` character, `\delimiter`/`\Udelimiter` or a `\let` copy with its code scanned there, a math symbol — else
  the Warning "Missing delimiter (. inserted)"), the `\left` group's K9 code checked by `\middle`/`\right` ("Extra
  \right." Error at the formula's group with the delimiter kept, user ruling 2026-10-12; §1064 `off_save` Warning
  against a document `{`/`\begingroup`), group ends inside it reported as Warnings ("Extra }, or forgotten \right.";
  "Missing \right. inserted" at a formula's or an argument's end, which now closes the formula instead of running on to
  the next `$`), a delimiter outside the map made stretchy as Perl does; rounds 3-4 (reviews r1, r3): the closers
  `off_save` inserts are owed by the group it closed and the debt follows the stack at every pop, so the author's
  `}`/`\endgroup` is deleted (§1069) where it meets a group it cannot close instead of setting the rest of the paragraph
  as math; `\endgroup`, `$` and a DefEnvironment's own end inside an open `\left` get `\right.` first (§1065 at the
  mismatching primitive); a brace where a delimiter belongs takes TeX's path ("Missing delimiter", the brace read
  again); a parameterless command is a delimiter only when it typesets a symbol, its trial digestion silent. Rounds 5-6:
  argument groups carry a group code, and the `}` a delimiter scan puts back is deleted at the formula's frame (Error,
  as TeX and Perl; REDs stray_brace_at_the_formula_level_is_deleted and
  right_in_an_argument_brace_group_after_the_inserted_brace green); every other stray `}` keeps 65a's recovery until row
  MS (§1046-1047). Open: RED boxes-groups/right_inside_an_argument_drops_the_argument. Closes WISDOM #41's `\left` row
  without an `undigested` parameter. braket's `\Set` no longer splits at a `\left|` (KPE #607, 2107.02628). amsmath's
  `\env@cases` defined (amsmath.sty:1121-1126; zhlineskip's `{originalcases}`, whose unmatched `\right.` otherwise
  cascades under the group check). A/B vs 65a (`scratch_65d/ab*`): 109 run-336 witnesses (48 X1 IfLimit + 60
  `unexpected:\right` + 2605.21750): Fatals 48 → 0, errors 3541 → 826, papers with errors 106 → 21; 200 `\left`-rich
  papers and 150 random: XML identical (194 + 141 same, 10 equivalent or ids-only, 1 timing-only), diagnostics unchanged
  (one random paper −1 ambiguity warning). 1710.02165, whose unclosed `\left` swallowed the rest of the paper, gains 4
  sections, 9 figures and its bibliography. Witnesses parens.tex:43 and 1207.4709 byte-identical; golden
  `tests/math/sampler` `\left\relax(` now stretchy, as Perl. Guards `perfect_kernel_batch65d::*`, golden
  `tests/parse/left_right_delimiters`. Residuals (OD #530): the `\right` delimiter digests after the group's
  `\aftergroup` tokens (= Perl); a stray `}` in math inserts `$` instead of TeX's "Extra }, or forgotten $", and a `$`
  meeting a document `{` defers the math end (pre-existing).
- **65b** (landed; C5 step G4):
  - **Column ends by command code** (`is_column_end`; the `\halign` preamble incl. `\tabskip`; OD #522, KPE #603).
  - **Frozen `\span`** for `\multicolumn`/`\multispan`/`\sp@n` (user ruling 2026-10-11, one Warning per document;
    OD #523).
  - **Frozen `\cr`** at a preamble's file end (OD #524).
  - **Misplaced `\noalign`** reported per site, its body typeset as an ordinary group with rule markers inert (OD #525,
    KPE #191 extended).
  - **aa `{longtab}`** wrapper (KPE #604, 1611.08428) and **aastex `\tablewidth{0pt}`** expandable (KPE #605,
    1805.09353).

  W1 (179 `\span`-redefining papers, run 336): errors 1730 → 839, papers with errors 138 → 57. W2: 280 → 218. Manual
  net 797: 0 XML changes. Guards `perfect_kernel_batch65::*`.

  Open:
  - ~~RED `class-bindings/aalongtable_lt_array`~~: fixed in 65h (OD #541, KPE #612).
  - Math readings of ambiguous formulas can flip between builds with no input change (2205.06656, 2503.00718,
    1708.09355). Each is deterministic per binary; likely an interning-order tie-break in the math parser.
- **65e** (RUST-ONLY regressions of run 338 against run 336; triage 2026-10-12): l-aa's plain `\matrix{…\cr}` (64u:
  aa's amsmath gave the environment-only form; aa's `\@currenvir` dispatch, KPE #608); crckapb's `\keywords` ends at
  the abstract's `\end` (64zd: read to EOF with the class `\abstract` refused); a `\par`/`\vskip` in an item's inline
  sectional block closes no paragraph (64d; OD #532); collcell takes the template's `\ignorespaces` as its delimiter
  and drops the trim (63v); the OmniBus theorem set declares only free names, each stub firing once (64y-amplified;
  OD #533, KPE #609; siamltex's theorems declared as its class does); a refused setter's replay that errs is rolled
  back by an assignment journal (64k; OD #534); ptephy's `$^{n}$` affiliation list goes to the names its marks label (64qa:
  2203.01270 83 MB/6.9 GB/357 s → 1.8 MB/3 s; its legend lines after the names a document pubnote each — only when every line after the first `\\` is led
  by a footnote symbol, else the whole list stays names: `ptephy_name_lines_after_break_stay_names`,
  `ptephy_textsuperscript_legend_line`);
  hyperref loads letltxmacro (OD #535, ruling Q5). Shared (`show_unanswered_marks`, base_utilities/frontmatter/insert.rs): an author's
  superscript symbol mark nothing answers is shown, as the PDF prints it, a math `*` as `∗` and a text one
  (`\textsuperscript{1,*}`) as `*` (unless it placed one of that author's contacts that drops it; a mark the author's
  email shows as well stays on the name too, 1910.00401); residual: a `$^{*}$` the kernel shows itself (keepsup, an
  email line) keeps ASCII `*` where pdflatex prints `∗` (1910.00401 email, probe hb); four older guards flipped to the PDF's reading (batch61 `author_thanks_and_footnote_keep_their_math`,
  `marked_affiliation_wrappers_and_legends`; batch63 `author_mark_comma_in_math`; batch64
  `marked_block_email_line_owner`). Run-338 Fatal →
  witnesses: astro-ph9609122, astro-ph9809009, astro-ph9503122, astro-ph0002500, astro-ph9901229, astro-ph9610167,
  astro-ph0003044, astro-ph9702104, astro-ph0003143, astro-ph0003144, 1806.03273, 1912.00582, 2604.02352, 1308.6566,
  0811.3048, cs0511055, 2106.06828, 1911.11856 → 0 errors; 1805.06896 2, 2304.12197 3, 2307.13461 47, hep-ex0406049 30,
  0705.0894 2, 2511.04367 7, 1811.04820 1, 2607.24171 3 (all Error, below or at 64c/64h). Guards
  `perfect_kernel_batch65e::*`. Open: `ltx:appendix isn't allowed in ltx:inline-sectional-block` after an item's
  bibliography (5 of the C witnesses; `list-structure/bare_itemize_end_leaves_appendix_in_block`, witness
  1805.06896: a bare `\itemize`'s `\end{itemize}` leaves the outer list open — GREEN in 65i, the open list giving way
  to the section); the set's amsthm load replaces a
  document's own `\newenvironment{proof}` (shared with Perl); a clean replay of a document setter emptied the
  stores its body `\gdef`s ("Written by ." where the PDF has "Written by Alice.") — GREEN 77i
  (`sectioning-frontmatter/replayed_document_setter_keeps_its_store`: a body store holding the argument stays set and
  is the title page's record, #759).
  Not in scope (lost identically in 65a): a composite mark `$^{2*}$` or `$^{1\dagger}$` (2307.15337, 2604.24425) and
  cas `\fnmark[$\dagger$]` (2511.01302, 2607.29394) lose their symbol. Also composite: under OSA a `$^{2\#}$` mark shows only its
  symbol half on the name (c2h), and a `$^{2\%}$` address mark places its email on the block, not its author (both
  as in 65a).
- **65a** (landed; RUST-ONLY): sweep #149's regressions against #148. A `\title`/`\author` the 64d seal turns away
  runs the raw class's own definition in place when it provably keeps no value (fail-safe store classifier); an
  `\author` the class hands only to page furniture becomes the author of its sectional unit (ruling 2026-10-11); heading
  names (`\contentsname`, `\refname`, `\indexname`, `\acknowledgmentsname` …) are digested in their own group, so an
  unbraced font switch no longer leaks into the document; insert_block keeps Perl's container order, with a guarded
  closing-caption hoist out of a rotate/scale box; ASCE grades (`M.ASCE`) qualify the name. OD #526-#529. Witnesses
  recipebook/LongSample (recipe titles and attributions back), heria/heria-proposal (schema-valid), ascelike/ascexmpl,
  1810.08269. Guards `perfect_kernel_batch65a::*`. Residuals: a late class `\author` that both stores and prints — FIXED
  77i r2 (runs in place, its text kept, the frontmatter's stores put back: silent-loss #8, OD #526); the figure-panel wrap for refused caption hoists (task #193;
  `captions-floats/subcaption_labels_in_scaled_panels_stay_on_their_panels` GREEN 78o for its labels and numbers, OD
  #853; the panel figure inside the `\scalebox`'s `ltx:inline-block` stays schema-invalid, 2 jing lines).
- **65h** (RUST-ONLY; run 340's Error→Fatal regressions, 287 papers against run 338):
  - **`\LT@array`**: the longtable binding is split as longtable.sty splits it (`\longtable` = `\begingroup\LT@array`,
    `\endlongtable` closes the group), so aalongtable.sty's and anlongtable.sty's `\longtable` build their tables
    (81 papers; OD #541, KPE #612).
  - **pasj00/pasj01/pasj02 without the shipped class** keep the class's `{longtable}` through the longtable binding,
    with the class's `\noalign` `\caption`/`\label` that gobble a following `\\` (pasj00.cls:1245-1270), so the head
    row after `\caption{…}\label{…}` is kept (astro-ph0407419, astro-ph0701701; OD #541).
  - **longtable caption row** (OD #103): a caption's row with an `&` after the caption is kept, not dropped as the
    caption's alone (`\caption{…}\label{…}` with no `\\` before the head row lost it).
  - **Misplaced-alignment cascade**: the first misplaced `\noalign`/`\omit`/`\span` in a group is the Error, the rest
    of that group Warnings; at top level the group is the document body (OD #540). This is open for the main session:
    it supersedes 65b's per-site Errors.
  - The 107 longtable papers: 107 Fatal → 0 Fatal, 2147 errors (64ze) → 350, 73 papers at 0 errors, none worse than
    64ze. Of the other 121 non-infra papers, 56 are no longer Fatal. Guards `perfect_kernel_batch65h::*`.

  Open, each a RED repro and none fixed here:
  - **rccol + natbib** (64ze): `Fatal:Timeout:PushbackLimit` in all 18 PushbackLimit papers.
    - Witnesses 1101.1265, 0812.2309, 1104.5326, 1106.3121, 1202.3790, 1212.1181, 1304.2825, 1307.3777, 1309.0372,
      1404.0550, 1501.04632, 1509.07583, 1610.05509, 1610.09398, 1801.07259, 1810.01491, 2207.03232, 2609.37147.
    - 64x converts them.
    - RED `alignment-bindings/rccol_with_natbib_pushback_loop`.
  - **Environments open at `\end{document}`** (65c): their kept content holds the bibliography or sections, and every
    `malformed:*` placement is an Error. Of the 21 sampled TooManyErrors papers, 19 reached Fatal at 65c.
    - Witnesses 1611.05714 (64ze: no bibliography, 1 error), 1507.01637, 2412.20775, 1806.06901, 1101.1959,
      2007.11971, 1210.0368, 1003.1368, physics0505113.
    - Near-threshold papers (64ze 96-99 errors): 1211.3050, 2210.11107, 1305.0086, astro-ph0303604, math9805003.
    - cs0009011 ends in `Fatal:Document:Generic("add_child`.
    - RED `list-structure/open_description_holds_the_bibliography`.
  - **standalone sub-file preamble**: xcolor's tabular hooks are installed without their definitions, so every later
    tabular is no alignment.
    - Witness 2512.09430: 64ze 84 errors, 65b+ Fatal, one misplaced `\toprule` per table.
    - RED `loader/standalone_subfile_xcolor_tabular_hooks`.
- **65i** (RUST-ONLY + one recorded surpass; run-340 Error→Fatal regressions):
  - **Constructs the document ended inside of** (65c's cluster: an upstream defect leaves a list, formula, script,
    alignment or `\fbox` open, and since 65c everything after it arrives inside it): such a construct is
    `lx@unended` and its elements auto-close (Perl's per-node `_autoclose`), so the next section, theorem or
    bibliography closes it instead of being held as `malformed:*` (OD #542; Perl errs as 65c did). Run 340's 228
    error→fatal papers (65bundle → 65i): 41 Fatal → OK (all 18 PushbackLimit, cs0009011's add_child, 2512.09430, 21
    TooManyErrors), none worse; no text word outside math lost in the 42 changed papers whose base XML parses
    (2512.09430 loses only the names in its `<ERROR>`s). Cluster vs 64ze: 1611.05714 1 → 1,
    1210.0368 30 → 30, physics0505113 75 → 75, cs0009011 13 → 13, 1507.01637 25 → 25, 1806.06901 52 → 52; a few
    errors above 64ze where 64ze dropped the rest of the paper: 2007.11971 9 → 13, nucl-th0401019 89 → 91, 1204.0071
    88 → 90, math9805003 96 → 97, 1101.1959 55 → 56. **A formula open at the end is the `$` deferral, not the end**:
    tex_math.rs `end_math_or_defer` defers the math end of `$S^{x$` to the script group's end, where TeX inserts the
    `}` (RED paragraph-mode/dollar_closes_open_script_group, row MS); 65i only closes such a formula at the next
    structure, so the text after it is kept but run together (2007.11971: 2,453 script-font words, 7,157 run-together
    pseudo-words by the review's count; 1210.0368 908 runs of 20+ letters; 1907.09124 148; math9805003 25) — no word
    gain is claimed for these. Fidelity given up: an unended `center` or `minipage` no longer marks the section that
    closes it (`ltx_centering`/`ltx_minipage`, OD #542). 65e's open item, a bare `\itemize` leaving the
    outer list open (RED → GREEN `list-structure/bare_itemize_end_leaves_appendix_in_block`): 1805.06896 2 → 0,
    2304.12197 3 → 0, 2511.04367 7 → 0, 2307.13461 47 → 42, hep-ex0406049 30 → 29, 0705.0894 2 → 2. Still Fatal (64ze 99, 97 errors), each the honest
    errors of regained content, their upstream defects not reduced: 1211.3050 (`align`s left open,
    Configurations.tex:288, ObtainingConf.tex:1207), 2210.11107 (4 stray `&` of a tabular left open,
    main_arxiv.tex:2979, after a run of `\acro{\smaller …}` errors in math; `\smaller` is undefined in pdflatex too).
  - **rccol `R{}{}` with natbib** (`Fatal:Timeout:PushbackLimit` since 64ze, 18 papers): the template reader empties
    `\@temptokena` before a column-type rewrite as array.sty:361 `\NC@do` does (natbib's `\appdef` left it filled), and
    an rccol binding gives `R` the result of its `\edef` rewrite, the raw cell code formatting the number as pdflatex
    does (KPE #615; 1101.1265, 0812.2309, 2609.37147: 64x 1 error → 0). **Known gap** (the kernel guard's fixture,
    kernel-alignment/column_rewrite_reads_temptokena_empty, converts with 3 Warnings, pinned): the template reader
    expands a column type and does not run it, so a rewrite written for array.sty's search — assignments to
    `\@temptokena` ending in `\NC@find`, array.sty's own `*` shape (array.sty:372-379) — leaves
    `Unrecognized tabular template` for `\@temptokena`, `\expandafter` and `\NC@find`; no pdflatex-valid rewrite that
    reads `\@temptokena` avoids them (array.sty's `\@mkpream` rejects an expandable `\the` in a `\newcolumntype` body,
    "Illegal pream-token"). Closing it means running a rewrite's assignments in the reader.
  - **Standalone sub-file preamble**: `AddToMacro` in a package loaded inside the bracket is local, so xcolor's
    tabular hooks end with its definitions (OD #543, KPE #614; 2512.09430 84 → 5 errors, 6,635 → 14,455 words).
  - Guards `perfect_kernel_batch65i::*` (65e's `par_in_item_bibliography_keeps_the_block` now on a list that ends,
    repro list-structure/bib_blank_lines_in_closed_list); repros list-structure/open_description_holds_the_bibliography,
    list-structure/open_math_script_holds_the_section, boxes-groups/unended_fbox_holding_figures,
    kernel-alignment/column_rewrite_reads_temptokena_empty, alignment-bindings/rccol_with_natbib_pushback_loop,
    loader/standalone_subfile_xcolor_tabular_hooks (all GREEN); RED paragraph-mode/dollar_closes_open_script_group
    (row MS).
- **65m** (recorded surpass + Perl-origin fixes; run 340's first-error cluster `Script _ can only appear in math mode`,
  6,357 papers, root-cause table `~/data/pk_agents/main/run340/clusters/underscore_rootcauses.md`): text TeX only
  stores, writes to an auxiliary file, or gobbles is not digested as typeset content.
  - **Written copies** (A1+A2, ≈2,750 papers): natbib's `\bibitem` label (natbib.sty:822, .aux only) and a caption's
    or heading's `[short]` form (.lof/.toc) are digested with `tex_math::WRITTEN_COPY` set (`\lx@written@copy` in
    `\lx@toc@copy@neutralize`; natbib's `\NAT@@wrout`), where a text-mode `_`/`^` is its character without the Error
    (OD #550, KPE #620).
  - **Names and gobbled arguments** (A3-A6, B1-B3): imsart's `\bdoi` gobbles (imsart.sty:2583); feynmp's `{fmffile}`,
    cas's `\bio`, aastex's `{interactive}` read their file arguments Semiverbatim; achemso's `\email` is the author's
    `ltx:contact`; a raw class's rerouted `\doi` store is a detokenized string; `\the<ctr>@ID` splices the counter's
    name as characters (OD #551, KPE #621). **A7**: cleveref's `\label[type]{key}` after `\caption`, a procedure's caption (algorithm2e) or
    `\captionof` (OD #552).
  - Witnesses (base 138cf5b167 → 65m, Errors): 23 of the table's 26 → 0; 2102.09024 3 → 1 (`\captionof`),
    1511.07613 3 → 2 (imsart `\setattribute`, `\endbibitem`), 2303.02355 28 → 7. 25 more found by grep (natbib
    labels, `\caption[…_…]`, `\newcounter{…_…}`): 17 with errors, all → 0. 60 random natbib/`\caption[` papers:
    byte-identical XML. Visible changes: imsart entries lose their DOI (as the PDF), 2404.11739's floats get their
    labels (`labels="LABEL:["` before), 1803.03591's e-mails are the authors' contacts.
  - **Open**: aastex's `\tablenotetext{^a}` digests its mark as text (deluxetable_sty.rs:128; aastex631.cls:5382 sets
    it `$^{…$#1$}$`: 2303.02355's 6 left); a `.bbl` call of a raw class's `\doi` adds a second DOI pubnote
    (1710.01977); cleveref's `[type]` (dropped, `\cref` saying "table" where the PDF says "Appendix Table", 2404.11739)
    FIXED 77s (KPE #998). Guards `perfect_kernel_batch65m::*`; repros index-bib/natbib_bibitem_label_is_aux_only,
    index-bib/natbib_authoryear_label_is_aux_only, captions-floats/caption_short_form_is_lof_only,
    captions-floats/cleveref_label_type_after_caption, captions-floats/cleveref_label_type_after_procedure_caption,
    captions-floats/cleveref_label_type_after_captionof, macro-state/counter_name_with_underscore,
    class-bindings/imsart_bdoi_is_gobbled, class-bindings/cas_bio_photo_file_is_not_typeset,
    class-bindings/achemso_email_is_an_address, class-bindings/aastex_interactive_arguments_are_not_typeset,
    graphics-tikz/feynmp_fmffile_name_is_not_typeset, sectioning-frontmatter/raw_class_doi_store_is_a_string (all
    GREEN).
- **65f** (RUST-ONLY + one recorded surpass; run-338 Error→Fatal kernel/mode clusters):
  - **Plain TeX documents** (`state::plain_tex_document`): the on-undefined LaTeX autoload no longer fires there, so a
    LaTeX kernel command is undefined as in TeX and Perl (1504.02513, 1402.1807, 1310.8121, 2405.04530 `\verbatim`
    Fatal → 0 E; math9810192 101 E Fatal → 55 E, its `gtmacros.tex` is missing; 1901.09683 Fatal → 1 E). Perl's
    TeX.pool triggers are held undefined there, so generic probes see plain TeX, and a use still loads the format
    (OD #536, KPE #610). graphics' epstopdf hook is guarded as pdftex.def:681-700 (1710.04722, 1802.06281 Fatal → 0 E).
  - **PDF/DVI by included figures** (OD #537): 1012.0815 Fatal → 3 E (= 64g), 1002.1751 → 0 E. DVI when every
    included figure resolves PostScript-only, epstopdf conversions or load notwithstanding, or pstricks is loaded;
    oracle = the Producer of arXiv's PDF ("dvips + GPL Ghostscript"): 1806.00093, 1806.00188, 1806.00284, 1806.00370
    flip from PDF (65a) to DVI; the epstopdf-loading EPS-only 1804.02991, 2104.01314, 1209.6096, 2210.10273, 1701.08490
    stay DVI, as 65a had them.
  - **Cycle guard**: a Mouth token's fingerprint carries its source position (OD #538; 1009.1698 Fatal → 1 E, = 64u).
    A sixteenth open file level is pdftex's "TeX capacity exceeded, sorry [text input levels=15]" Fatal (a
    self-`\input` macro ran to the watchdog's `Fatal:timeout:wallclock` at `--timeout`, ~0.75 MB of log a second; now
    0.2 s). The gullet checks the cooperative deadline too (`Fatal:Timeout:Convert`, which mattered only where no
    watchdog is armed: the test harness, `latexml::api`); open: under a watchdog sharing the deadline the two race, and
    a run may end in the cooperative Fatal with its XML or in the watchdog's with none (OD #538; grace: task #205).
  - **`\endinput`** through a macro on a line ending in `%` no longer reads the next line (comments off; atbegshi in
    plain TeX).
  - A/B 65a-rel → 65f (681 papers: 150 plain, 125 mode-flip, 66 pstricks, 98 EPS, 99 PDF-figure, 143 random): 639
    identical; plain 2078 → 1829 errors. The 22 PostScript files submitted as `.tex` change both ways; among real TeX
    sources 4 better (hep-lat9207024 Fatal → 26 E, 8k → 42k words), 4 worse by 1-8 `Error:undefined` (plain papers
    whose missing macro files LaTeX used to stand in for: hep-ph9209216 mtexsis, hep-th9109002 jnl). Guards
    `perfect_kernel_batch65f::*`, `preclass_kernel_autoload::plain_document_leaves_kernel_cs_undefined`,
    `converter::tests::figure_output_cue_of_sources`.
  - Open: math0305287, math0111222 stay Fatal: TL 2025 pstricks under plain TeX loads expl3 (pst-code-pspicture.tex:76
    `\ExplSyntaxOn`) and LaTeX with it, plus Perl's braced-`\input` LaTeX load (KPE #611). J (1911.11856, 2106.06828)
    is TeX-faithful once `\LetLtxMacro` is defined: both 0 E with letltxmacro loaded (65e's hyperref change).
- **65l** (RUST-ONLY binding gap; OD #545-546): the MnSymbol binding declares every name of MnSymbol.sty — the 1,307
  `\Decl@Mn@Char`/`\Decl@Mn@Alias` names, the 55 `\Decl@Mn@Op`/`\Decl@Mn@Int` operators, the `symbols` font's
  `\t…`/`\d…` operators, delimiters (`\lsem`, `\langlebar`, … fence after `\left`), brace pieces, accents and the
  package's own declarators — with unicode-math's code point, else TeX4ht's for the font slot, roles as the sibling
  symbol bindings; a name already defined keeps its definition, a slot-and-class sibling of one is `\let` to it
  (`\nin` → `\notin`). The stub's `Warning:missing_file` is now an `Info` (13,470 run-340 papers carried it: a status
  confound, OD #546). Run 340's first errors `\nin` 175, `\udots` 173, `\sumint` 165, `\cupdot` 130, `\strokedint`
  113, `\lsem` 108. A/B 65bundle-rel → 65l on the 80 witnesses (`run340/stubs/MnSymbol_{err,ok}.txt`): err set 40 → 10
  papers with errors, 78 → 39 errors, the 10 left non-MnSymbol; ok set 0 → 0 errors, words and Math identical; XML
  changes are `<ERROR>` → symbol, `\checkmark` amssymb's again (the stub overrode it), `\lcurvearrowright` ↶ → ↷ (the
  stub had MnSymbol's `l` family reversed). Round 2 (review): TeX4ht's `&gt;` decoded (`\gtr`), the footlines and
  `\upmodels`/`\downmodels` (2604.25507), `\ocirc`, `\filledlargestar` as MnSymbol.pdf draws them, a name a document
  declares with `\Decl@Mn@Char`/`\Decl@Mn@Open` takes its slot's glyph from the tables (else an Info), `\slimits@`
  left to amsmath. Guards `perfect_kernel_batch65l::*`, `mnsymbol_sty::tests::*`; repros `class-bindings/mnsymbol_*`.
- **65j** (regression of the 65 bundle; OD #544, KPE #616): `\left`/`\middle`/`\right` in a `\bm` argument stand at
  `\bm`'s level, as bm.sty's token scan emits them (bm.sty:162-217), their delimiter bold (`\bm@delimiter`, :416).
  `\bm` is a protected macro: without a top-level delimiter command the argument is the bounded `\lx@bm@bold` (Perl's
  constructor, byte-identical: 30 of 30 bm-loading control papers); with one, the runs between the commands are bolded
  apart and `\lx@bm@delimiter` bolds the delimiter's element; a run's leading script stays on the fence (2107.08152,
  1703.10645). Witness 2204.13665 (`\bm{\left\langle} #1 \bm{\right\rangle}`): 65a 0 E/521 W, bundle 587 E, 65j 0 E/227 W,
  words 200,003 → 202,457. Exposure: `\bm|\boldsymbol|\hm{\left|\right|\middle` in 81 of 400k sampled sources (≈600
  corpus-wide), 38 with bm loaded (≈280). A/B of 30 of them (65a / bundle / 65j): bundle errors on 9 (1-587), 65j
  0 E everywhere (2312.16075's 1 E is 65a's), no paper worse than 65a in errors, warnings or words. Guards
  `perfect_kernel_batch65j::*`; repro `boxes-groups/bm_left_right_at_the_outer_level`. RED side finding
  `boxes-groups/formula_end_after_extra_brace_in_left_group` (`${\left( } x$` runs to `\end{document}` on 65a and
  the 65 bundle alike; row MS's `}`-meets-`\left`-group class).
  Round 2: the commands that box or group their argument — amsbsy's `\boldsymbol`/`\pmb`, the `\bm` of RevTeX
  (Perl's bold `\bm`; RevTeX itself asks for bm.sty), the World Scientific classes and interact.cls — share the
  mechanism (bm_sty.rs `bold_math`, `BoldArgument::Grouped`): paired `\left…\right` in one argument stays the
  bounded constructor (byte-identical), an unpaired one (pdflatex: "Missing \right. inserted", ⟩ lost) is set at the
  outer level with one Warning per command instead of the bundle's "Attempt to close a group that switched to mode
  math". Repros `boxes-groups/{bm_revtex,bm_ws_class,bm_interact,boldsymbol}_unpaired_left_right`.
  Round 3: a paired `\left…\right` in a bm.sty `\bm` argument stays the bounded `\lx@bm@bold` too, byte-identical
  to the bundle (round 2 split it: `\bm{\left(A+B\right)^\mathsf{T}}` lost its superscript; the 8 A/B papers whose
  `tex` had changed match the bundle again). On the unpaired path a script's argument is bolded whole — a command
  takes its parameters' arguments (`\right\rangle_\mathrm{L}`), as bm.sty expands it (`\bm@expand`); one that
  cannot be told keeps the rest of the run, with a Warning — and a prime is a bold `^{\prime}`. Guards
  `perfect_kernel_batch65j::{bm_paired_left_right_keeps_its_superscript, bm_script_command_argument_after_unpaired_right,
  bm_prime_after_unpaired_right_is_bold, bm_script_argument_that_cannot_be_told_warns}`. RED row **BD**
  (`\DeclareBoldMathCommand` delimiter after `\left`, 1205.4484).
- **65r** (bindings load what their TeX Live files load, round 2; OD #561-564, KPE #627-629): amsrefs loads url,
  rkeyval, textcmds and mathscinet (`\cprime`, `\germ`, `\scr`, `\url`, `\DefineSimpleKey`; `lite` skips the last two),
  `\bibselect` reads its `.ltb` database (cited entries printed, `\bibselect*` all; the author's `.bbl` when no
  database ships), `{biblist}[…]`/`{biblist*}`, `\resetbiblist`, `\BibSpecAlias`, `\bysame`, `\ocite`; a repeat
  `\usepackage[…]{hyperref}` after a class binding's own hyperref load takes its new options through `\hypersetup`
  (`<file>_keyval_setter`), so `colorlinks` loads color (revtex4, elsarticle, …); PoS loads color as PoS.cls does;
  thm-restate loads thmtools (the mutual-require cycle is an Info, as LaTeX's `\ver@` ends it); elsarticle `1p`/`3p`/`5p`
  load geometry (one-column layouts give it their text width); xy's frame feature requires curve (xypdf-fr.tex:60:
  `\usepackage{xypic}` curved arrows, `\crvi`); revtex defines `\sanitize@url`. Run-340 first-error estimates: amsrefs
  ~1,860, colorlinks ~860, thm-restate ~477, xy curve ~380, `\sanitize@url` ~375, elsarticle geometry ~287. Witness A/B
  65int2 → 65r (`~/data/pk_wt/scratch_65r/ab/`, ≥4 Error + 4 OK/Warning papers per item, 61): 32 of 34 Error papers
  lose errors (28 to 0), none gains; no OK/Warning paper changes beyond tmp-path noise, save 1907.13203's placeholder
  `pdftitle`/`pdfauthor` now RDFa. Next first errors: revtex `\@AF@join` (2006.04679, 1702.04290, 1510.07897,
  2508.01050, from JASA.cls/ujp.sty); thmtools `\declaretheorem{theorem}` prints no "Theorem" name (shared with 65int2).
  Guards `perfect_kernel_batch65r::*`, `06_cluster_bibliography::amsrefs_bibselect_{database,star_database}`; repros
  `stub-dependencies/*` (7), `index-bib/amsrefs_bibselect_*`.
- **65k** (stub bindings load what their packages load; OD #547-548, KPE #618-619): mdframed loads zref-abspage,
  needspace and color (mdframed.sty:72-77) and its `framemethod` package (tikz/pst-node), `\mdfsetup` reads the keys;
  backref loads kvoptions/kvsetkeys/rerunfilecheck and defines its language hooks, `\ifbackrefparscan`, `\backref`,
  `\backrefalt`; hyperref's `backref`/`pagebackref` options load backref; mathpartir loads keyval and defines its
  separators; epstopdf loads pdftexcmds (`\ifpdf`); mciteplus its switches, counters and setters; arydshln `{Array}`/`{Tabular}`/`{Longtable}`; nag its rule
  commands. Class bindings: pnas-new (`\dates`, `\dropcap`, the stored sections, `[framemethod=tikz]{mdframed}`,
  jabbrv), achemso (caption/graphicx/url/xkeyval, `\SectionsOn`/`\AbstractOn`, the TOC-graphic lengths). A/B
  65bundle → 65k over the run-340 witness samples (`~/data/pk_wt/scratch_65k/`, 40 Error + 40 OK/Warning papers per
  stub, 560): Error papers 266 → 226 (40 Error → OK/Warning: mciteplus 39 → 14, mdframed 39 → 29, arydshln 37 → 35,
  mathpartir 39 → 37, nag 37 → 36; tabu and backref unchanged); no OK/Warning
  paper has more errors or fewer words or bibitems (1804.04674's figures renumbered by caption's `\caption*`). Loaded
  through hyperref's option (or `\PassOptionsToPackage`), backref's `missing_file` message is an Info, a direct
  `\usepackage{backref}` keeps the Warning (main-session decision; Perl is silent, the lists are generated navigation;
  witness 2203.07162 back to 0 warnings). arydshln witnesses 2502.17812, 2306.04970; nag 1303.5654. Guards
  `perfect_kernel_batch65k::*`; repros `stub-dependencies/*`. Not stub gaps (remaining first errors): wiley-article's
  `\keywords` is defined inside the class's `wiley@abstract` body, which the locked `{abstract}` binding bypasses
  (2009.12523, 2007.02789; landed in 68n, OXIDIZED_DESIGN_DIVERGENCES #634); lmcs's `\titlecomment` under `\documentclass{LMCS}` (upper-case name, no binding: 1103.0510);
  ulem's `\ULforem`; tabu's `|[width]` rule spec (1907.07986).
- **65p** (RUST-ONLY, Perl 0 errors; OD #559, KPE #626): a `\multicolumn` over a paragraph column (`p{}`, `m{}`, `b{}`,
  tabularx `X`) lets `\\` to `\lx@newline` inside its cell box instead of rewriting the body's brace-depth-0 `\\`
  tokens, so a nested `\begin{tabular}…\end{tabular}` keeps its rows (the rewrite made them line breaks: "Extra
  alignment tab", "\noalign cannot be used here" at the next `\hline`, or, for a one-column table, rows lost with no
  diagnostic), and a math alignment in the cell (`pmatrix`, `cases`, `array`, `aligned`) keeps its rows; the `\let`
  precedes the column's `>{…}` tokens (`Cell::prefix_at`); `X`/`J` cells are spliced as `\vtop{` (no more "Missing
  number" Warning and `width="0.0pt"`). Witnesses
  2607.14806 4 → 0 errors, 2010.13561 29 → 0, 1808.05579 18 → 0; 1610.00974 and 1805.01525 byte-identical. Sample of 30
  seeded random papers with a `p`/`m`/`b`/`X` `\multicolumn` (`~/data/pk_wt/scratch_65p/samp.tsv`): 28 byte-identical, 2112.08542
  loses `width="0.0pt"` on its `X` cells, 2410.18046 a temporary path. Guards `perfect_kernel_batch65p::*`; repros
  `kernel-alignment/multicolumn_*`. Open: two RED repros, rows in "Perfect-kernel RED repros without a row elsewhere"
  (a plain `\\` in a multicolumn paragraph cell recovered with no Warning; a `>{…}` prefix's arguments).
- **65n** (task #204; OD #553): TeX's math-group recovery for an inline formula outside alignments — tex_math.rs
  `end_math_or_defer` inserts the closer of a `{`/`\begingroup` group opened in the formula at its `$` (Error "Missing }
  inserted", §1193 + §1064-1065) instead of deferring the math end to that group's end; stomach.rs `insert_dollar_sign`
  ends the formula at a `\par` or a DefEnvironment's end ("Missing $ inserted", §1046-1047); a stray `}` at the formula's
  math shift ends the formula when it closes a construct digested as a unit (a box body, an argument digested on its
  own: `stomach::formula_ends_with_its_unit`; the DECISION `\mbox{$x}` repro GREEN) and is deleted otherwise, one Error "Extra }, or forgotten $" (§1069). Witnesses 65int2 → 65n: 2007.11971 13 → 1 errors, 2,643 → 43 warnings, run-together runs 179 → 0, text words
  outside math 5,268 → 9,865; 1710.02165 29 → 21, its swapped paragraph set right; 2008.11762 14 → 15 (a pre-existing
  `$` desync now ends at its `\par`). Unchanged: 1210.0368, 1907.09124 (deferrals inside alignments, row MS (1)),
  math9805003, hep-lat9811036, nucl-th0401019. 40 random papers (seed 65204) byte-identical. Guards
  `perfect_kernel_batch65n::*`; 65c/65d/65i guards updated to TeX's counts, 56/64y to the "Extra }" message. Round 2:
  the 70-paper A/B (witnesses + random, seed 65204) byte-identical to round 1. REDs left: row MS (1)
  cell_end_in_inline_math_inserts_dollar, (4) brace_meeting_left_group_in_inline_math_is_deleted. Round 3 (main-session
  decision): round 2's "partner outside" rule reverted to the unit-construct rule (it lost a1/a2/a16 and gained nothing);
  new RED (3) forgotten_dollar_in_a_plain_group_keeps_the_text.
- **65o** (SHARED, PERL-ORIGIN; OD #556-557, KPE #624-625, completes KPE #547's `\lefteqn` residual): eqnarray's
  alignment as latex.ltx defines it. A first-column `\lefteqn` is one cell (`\rlap`, latex.ltx:15800) that overprints:
  at normalization it spans the structurally empty cells after it (`Cell::overprint`, `normalize_overprint_spans`; TeX
  does not span, tex.web §791), so `\lefteqn{lhs}&&\nonumber\\` keeps Perl's one cell over the row with 0 errors, a
  cell with content (or an `\index`/anchor) keeps its column, and the row is classed `ltx_eqn_lefteqn` in the XML
  (Perl Base_XMath:956, dead there). eqnarray has the fourth, number column (:15764) a `\cr`-ended row may fill
  (1804.05059), its trailing empty cells dropped (eqnarray only), and latex.ltx's "Too many columns in eqnarray
  environment" where a `\\`-ended row reaches it. Run 340's `&` cluster R1 (about 2,050 papers) + R4 (about 71): the
  32 witnesses (R1 list, R4, 2205.04522, 2411.04274) go from 178 errors on 31 papers to 0, no words lost, warnings
  equal or fewer (nucl-th9512020's `\hat`-in-text and math0407427's `\span` warnings gone); 40 seeded random eqnarray
  papers: 39 byte-identical, cond-mat0601437 2 → 0 errors (a `\lefteqn` row). Guards `perfect_kernel_batch65o::*`;
  repros `alignment/{lefteqn_*,eqnarray_number_column*,eqnarray_empty_cell_keeps_later_column}` GREEN. Open (RED):
  `alignment/eqnarray_cr_row_column_count` (a raw `\cr` skips `\@@eqncr`: TeX counts on into the next row's "Too many
  columns"; we, as Perl, count afresh; its sibling `eqnarray_cr_row_unnumbered`, the row's number, is GREEN since 78n,
  OD #847);
  `alignment/lefteqn_overprint_keeps_template_material` (the `\multicolumn{1}{l}` drops column 1's `@{…}` and `|`,
  which TeX's `\rlap` keeps); `alignment/eqnarray_side_content_only_cell` (KPE #625: a cell holding only an `\index`
  or anchor measures under 1pt and is never absorbed, and an eqnarray row join drops a floated indexmark; rule
  proposed: size never decides dropping content, only `Digested::is_skippable` does).
- **65s** (KPE #630, #631, #632): the AMS bindings provide what the real packages define and documents call:
  amsmath's internals (`\bBigg@` on the scaled-delimiter path, `\big@size`, `\Let@`/`\restore@math@cr`/`\default@tag`,
  `\maketag@@@`, `\@mathmeasure`, `\spread@equation`, `\start@align`, `\intertext@`, `\mdots@`, `\ltx@label`), amsfonts'
  `\newsymbol` (through `\DeclareMathSymbol`; the msbm table is now also the `U`/`msb` font map), and amscd's connector
  read as an undelimited argument (spaces after `@` skipped) plus `\atdef@` and `\CDat`. Run-340 estimates: amsmath
  internals ~1,140 papers, amscd ~838, `\newsymbol` ~300 (run340/clusters/undefined_rootcauses.md rows 3, 5, 7).
  Guards `perfect_kernel_batch65s::*`; repros `ams-internals/*`.
- **65t** (KPE #633): the AASTeX binding defines the commands of the AASTeX 6.x class files that papers shipping
  those classes call: `\reportnum` and `\journalinfo`/`\submitted` as pubnotes, the edit and layout switches
  (`\turnoffedit*`, `\listofchanges`, `\suppressAffiliations`, `\NewPageAfterKeywords`, `\setwatermarkfontsize`), the
  author-limit count, `\tablefontsize`, `\abovedeluxetableskip`, `\fullcollaborationName`, `\lambdabar`, and the journal
  names `\aas`, `\dps`, `\jaavso`, `\maps`. Run-340 estimate ~1,700 papers (undefined_rootcauses.md row 1, aastex
  sub-family). Guard `perfect_kernel_batch65t::aastex6x_class_commands_are_defined`; repro
  `class-bindings/aastex6x_class_commands`.
  The World Scientific bindings (ws_journal_cls.rs, ws_procs_cls.rs; no Perl binding) load the figure packages the ws
  classes input (epsfig, epsf; ws-ijmpa.cls:551-555, whose psfig.sty is not in TeX Live), and define `\pub{received}{revised}` as pubnotes,
  `\cdash`, `{alphlist}`/`{arabiclist}`/`{romanlist}`/`{itemlist}` and ws-rv9x6's `\if@addchapnum`. Run-340 estimate
  ~1,500 papers (`\psfig`/`\epsfig` ~650, `\pub` ~500, `\cdash` ~120, lists ~150, `\if@addchapnum` 113). Guards
  `perfect_kernel_batch65t::{ws_journal_class_commands_are_defined, ws_rv9x6_defines_addchapnum}`; repro
  `class-bindings/ws_journal_class_commands`.
- **65v**: the jfm and mdpi bindings (contrib; no Perl binding) define what the versions of those author-shipped classes
  define and papers call (union of 119 sampled copies each, the class-family rule), and follow the paper's own copy
  where the versions conflict (`shipped_class_text`, `class_text_loads` in latexml_package/src/shipped_class.rs, as
  mn2e's journal abbreviations). jfm: `\ifCUPmtlplainloaded` (the template preamble's font checks; 597 first errors),
  the class's `\checkfont` block (amsbsy, upmath when shipped, `\upi`), the `lineno` option, `\aunote`, `\corresau` (a
  `role=corresponding` note), `\oneappendix`, and the author conveniences (`\Rey`, `\Pran`, `\ls`, `\ns`, `\p`,
  `\bnabla`, `\mathsfbi`, …) as `DefinitionOrigin::Fallback`, so a paper whose copy lacks one keeps its own
  `\newcommand`; by the copy: caption (35 of 119), `\le`/`\ge` as ⩽ ⩾ (73 of 119, jfm.cls:1361), `\mathsfbi` bold-italic
  Helvetica (newer) or slanted cmss (older). mdpi: by the copy, the packages its versions load (caption, lastpage,
  float, enumitem, mathpazo, setspace, epstopdf, upgreek, subfig; a 2013 copy loads no enumitem, so the paper's
  `[shortlabels]` load is the first) and `\conflictofinterests` (the statement under "Conflicts of Interest:", or the
  2012-2015 title form); as fallback guesses, the title-page setters as frontmatter notes (`\doinum` once accepted,
  `\secondnote`/`\thirdnote`, `\conference`, `\featuredapplication`, `\simplesumm`), `\addhighlights` (the switch; the
  block is the paper's own redefinition), `\acknowledgements`, `\publishersnote`, the page and layout setters,
  `\highlighting`, `\cellWidtha`…`\cellWidthz`, `\PreserveBackslash`, `\isAPAandChicago`. Run-340 estimate ~1,800 jfm +
  ~1,400 mdpi papers' first error (undefined_rootcauses.md row 1). Residual: `\oneappendix` keeps the appendix letter.
  (The captions' "Figure 1." is done: mdpi's since 77w, the copy read after its binding, 2608.13303 and 1307.3961's
  2013 copy bold `close=". "`; jfm's in 78a, OXIDIZED_DESIGN_DIVERGENCES #784.) Guards
  `perfect_kernel_batch65v::*` (with `perfect_kernel_batch64qa::mdpi_history_is_frontmatter`: lastpage's `LastPage`
  label ids the document); repros `class-bindings/{jfm,mdpi}_class_commands`.
- **65w**: six more class bindings define what their versions define and papers call (the class-family rule, from
  run-340 first errors and a census of the shipped copies). lmcs (contrib): `\titlecomment` a `role=titlenote` note,
  `\ACMCCS`/`\amsclass` classifications named by `\ACMCCSname`/`\amsclassname`, `\rsuper`/`\lsuper` (the newer copies'
  braced `\MakeLowercase{#1}`), `\qEd`, `{iteMize}{label}`/`{desCription}` as `{list}` (the enumitem the binding loads
  would read the label as a key; ~900). lipics (contrib): `{claimproof}` ending ◁, at its own end or `\claimqedhere`
  (other proofs end ∎: the class's grey ◀ for every proof is not reproduced, our `\qed` ignoring `\qedsymbol` as Perl's does),
  `\claimqedhere`, `\lipicsEnd`,
  `\proofsubparagraph`, the `(i)`/`(a)`/`(1)` enumerates, `{exercise}`, `{note}` and the starred theorems, `\flag`'s
  image, and the 2016-2019 setters as `DefinitionOrigin::Fallback` (`\EventShortName`, `\serieslogo`, `\DOIPrefix`,
  `\DOI` whose DOI, when set, becomes a pubnote once at `\maketitle`, `\volumeinfo`'s event as a `role=event` note;
  ~700). atlasdoc (contrib): the document-data setters store as atlasdoc.cls does (`\journal` too), and at `\maketitle`
  the title header the class options pick gives the notes, each prefix the note's label: with `preprint`/`cernpreprint`
  the journal reference, or else "Submitted to: <journal>", DOI and preprint number; with `HEPDATA` the preprint number
  and arXiv id; with `BOOK` (unless `PAPER`/`CONF`/`PUB`) none; otherwise "<\subprepend> <journal>" ("Journal:");
  `\arXivId`/`\HepDataRecord` store; `\AtlasPaperWebpage`
  from `\AtlasRefCode`, `\AtlasDate` as the date, the cover-page commands no-ops (~700). spie (contrib, which the loader
  gives spieman): `\linkable` and v1.00's `\supscr`/`\supscrsm` as Fallback (~650). arximspdf: `\doiurl`/`\arxivurl`
  (moved from the arxstspdf-only loader; 92 of 100 arximspdf copies define them) and `\surl`, `\setattribute`/
  `\getattribute`, `\tsup`/`\tsub`, `\OrigBibText…\endOrigBibText` (~700). aa: the online floats are set at
  `\AtEndDocument` as the class sets them, in both arities, told apart by the first argument (a number is the v7 form's
  N: `\onltab{N}{table}` numbered N, the counter stepped to N; v8.x's `\onltab{table}` keeps the number it reserved where
  called; v9.x defines none), `\longtabL{N}{…}` in place numbered N, `\AANum`, `\psj` (Fallback), and hyperref at the
  document's start when the paper's copy loads it (`class_text_loads`; ~530). Guards `perfect_kernel_batch65w::*`; repros
  `class-bindings/{lmcs,lipics,atlasdoc,spieman,arximspdf}_class_commands`, `class-bindings/aa_online_floats{,_v8}`.
- **65x**: seven more class bindings define what their versions define and papers call (run-340 first errors; census of
  the shipped copies, `scratch_356a0c47/census_65x.txt`). PoS: `\PoS{<identifier>}` the contribution's identifier as a
  `role=pubid` pubnote (PoS.cls :1395, a second call undefined as in the class; ~360). dmtcs-episciences (contrib):
  `\publicationdetails{volume}{year} {issue}{number}{id}` the head's "DMTCS vol. 19:3, 2017, #12" and the license line
  as pubnotes, the copyright as its date, and the class's number sets with `\primes` (~320). svmult: the 2018+ additions
  as `DefinitionOrigin::Fallback` — `\orcidID` an ORCID contact (as llncs's), `\ethics`/`\ethicssubhead`, the titled
  boxes (`{important}`, `{warning}`, `{tips}`, … : the title bold over the text), `\biblstarthook` under the
  bibliography heading (through `\lx@bibliography@preamble`), `\subsubruninhead`, `{petit}`, `{refguide}`,
  `\leftcaption`/`\rightcaption`, `\leftfigure`/`\rightfigure`, `\samenumber` (~350). iopart: the 55 journal
  abbreviations of the 2011-2014 iopart.cls and `\MLST`, which Perl's list predates, as Fallback (~200); every journal
  abbreviation keeps the class's trailing space (KPE #634). acmart: the 2.x additions as Fallback —
  `\noindentparagraph`, `\anon`, `\AtBeginMaketitle` (run as `\maketitle` begins, through the kernel's new
  `\lx@maketitle@begin`), the `acmcp` setters (stored), the translated title/subtitle/keywords as notes and
  `{translatedabstract}`; `\acmBadgeR`/`\acmBadgeL` take their arguments (Perl's `\acmBadgeL` printed them). IEEEtran:
  `{LaTeXdescription}`, `\yesnumber`, `\IEEEstrut`, `\IEEEsettopmargin`, `\IEEEeqnarraystrutmode`. elsarticle:
  `\fnmark[…]`/`\cormark[…]` as `\fnref`/`\corref`, `\nonumnote`, `\newpageafter` (Fallback). Residual: svmult's
  `\subfigures` (3 of 120 sampled; its `merk` counter machinery). Guards `perfect_kernel_batch65x::*`; repros
  `class-bindings/{pos,dmtcs,svmult,acmart,ieeetran}_class_commands`,
  `class-bindings/{iopart_journals,elsarticle_marks}`.
- **65y**: symbol and unit packages define the names papers call (run-340 first errors still undefined at the int7
  head, per package, `scratch_356a0c47/r7/still.txt`).
  SIunits (440): its own `\addunit`, `\addprefix`, `\usk`/`\unitskip`, `\per`, `\power`, `\squared`…`\rpfourth`,
  `\SIminus`, `\SIunits[…]`, `\SI@fstyle`, `\degreecelsius` (as ℃) and `\upmu`, its 198 units and prefixes (a table of
  SIunits.sty:195-543; the `derived`/`derivedinbase` sets and begin-document `\micro`/`\square`/`\gray` by its options;
  `\meter`/`\deka` aliases; `\degree`/`\arcminute`/`\arcsecond` as glyphs), all by `\providecommand` over the siunitx
  shim's `\relax` placeholders, so a unit outside `\si` keeps its symbol (2103.01586's `$221\usk\meter$` lost it; OD #565). constants (253): ported whole, `\Cl` recording at once (no `.aux`
  round trip), `\null` as `\relax`. newtxmath (400): `\widebar`, `\Angstrom`, `\bigtimes`/`\varprod`, `\vv`,
  `\mapsfrom`, `\transp`, `\lBrack`/`\rBrack`, `\uppartial`, `\lambdabar`, `\smallsum`, `\bigcupdot`, `\sumint`, `\cdotB`,
  `\re@DeclareMathSymbol` (names defined first keep theirs, as `\DeclareMathSymbol` refuses them; `\vv*{x}{i}` the
  two-argument `\vvstar`). gensymb `\celsius` (214, KPE #635); eucal `\EuScript` + options (229); libertine `\useosf`
  and the type1 figure-style commands (233); fourier's math symbols (163; names amssymb defined first keep theirs);
  stmaryrd `\oast`/`\ocircle` (118); mathtools `\newcases`/`\renewcases` as `\lx@ams@cases` (93); amsbsy `\binrel@`,
  `\binrel@@` (its content keeps its own class: no box measures math spacing here), `\math@atom` (161). Residual: a `\Cr` before its
  `\Cl` prints LaTeX's first-run "C^st ??" with a warning (no `.aux` round trip; 1307.4858); the
  parser does not pair `⦀` fences (RED `math-parse/fourier_vert_fences`, math parked); `\newcases` variants revert as
  `\begin{cases}` (the shared kernel reversion, base_xmath.rs:1471, as Perl's for `dcases` too); acronym's first-use
  double space after `\AC@placelabel` is Perl's (acronym.sty.ltxml:85-86; structure/acro.xml), kept; not taken (below the bar): amsmath's
  `\@mathmeasure` without amsmath (34), mathtools' `\delimsize` outside a paired delimiter (11). Guards `perfect_kernel_batch65y::*`; repros
  `stub-dependencies/{siunits_unit_commands,gensymb_celsius,eucal_euscript,libertine_figure_styles,fourier_math_symbols,`
  `newtxmath_symbols,stmaryrd_oast,constants_labelled_constants}`, `ams-internals/{amsbsy_binrel_math_atom,mathtools_newcases}`.
  A/B (262 run-340 witnesses, `ab_65y`): errors 701→101, Error status 230→33; the 18 word-count drops are
  error-recovery tokens leaving the math (text outside math equal) and 1307.4858's forward `\Cr`.
- **65z**: package bindings define the commands and internals their real `.sty` defines that papers, or the raw code
  they load, call (run-340 first-error "undefined" cluster, row 7). caption: `\subcaptionsetup` (`\captionsetup[sub…]`),
  `\captionlistentry` (the caption's counter step for a `\label`, no caption), `\captiontext*` (the float's label and
  text, the panel "(a)" in a subcaption group; with text, "Figure 2: text", the label's separator as a caption's). bigstrut/multirow: `\bigstrutjot`. natbib: the `\hyper@nat…` hooks
  (kotex extends them). titlesec: `\newpagestyle`/`\renewpagestyle`/`\widenhead` load titleps; `\ttl@straightclass{X}`
  is the kernel heading at X's level (`\lx@startsection`, as `\titleclass`'s commands now: a paper's own `\@startsection`
  is not called). titling: `\thetitle`/`\theauthor`/`\thedate` set by `\protected@xdef` as `\title`/`\author`/`\date`
  run (titling.sty:245-268, chained onto new kernel slots `\lx@title@after`/`\lx@author@after`/`\lx@date@after`, which
  the revtex4 and authblk `\author` call too; a name whose slot never fired is set at `\begin{document}` and at
  `\maketitle` (for a field changed since) from `\@title`/`\@author`/`\@date` — amsart/mnras with the fields in the body, IEEEtran over its
  placeholder), not at load, nor re-`\let` at `\maketitle` (KPE #636: `\title{\thetitle}` looped). Residual (as the
  int7 head): `\theauthor` empty under llncs, acmart, elsarticle, svjour3, aastex, whose `\author` stores no `\@author`
  and calls no slot (RED `package-bindings/titling_setter_llncs`). array: `\col@sep`, `\@acol`, `\NC@list`, `\extratabsurround` (raw eqnarray.sty, ltxutil.sty,
  dgruyter). authblk: the state `\author`/`\affil` keep (`\AB@las`, `\AB@au@str`, `\AB@thenote`, …), which papers'
  copies of `\affil` read. IEEEtrantools: `\bstctlcite`, `\IEEEstrut`, `\yesnumber`, `\IEEEsettextheight`,
  `\IEEEsettopmargin` (IEEEtran: `\IEEEsettextheight`). accents: `\cc@accent` (a `\hat` written to the .bbl) and
  `\cc@style` (`\widebar`). acronym: `\@iaci` (the `\acrodefindefinite` article, which now also defines
  `\fn@<acr>@IS/IL`), `\@firstupper`, `\texorpdfstring` at begin-document (PERL-ORIGIN: acronym.sty.ltxml calls
  `\@iaci`/`\@firstupper` undefined). crop: `\CROP@size`, the stock size, the corner marks. enumitem: `\labelindent`,
  `\enit@format`, `\enit@type` set per list (0/1/2). ulem: `\ULforem`/`\normalem` (switching `\em`), `\ULset`,
  `\UL@setULdepth`. footmisc: `\DefineFNsymbolsTM`. listings: `\lst@CheckVersion` (shipped lstpatch.sty skips itself),
  `\lst@mode`/`\lst@Pmode`, `\lst@token`, `\lst@SaveOutputDef`. amsthm: `\thm@headsep`, `\dth@everypar`,
  `\displaymath@qed`. feynmf/feynmp: `\fmfkeep` in a graph keeps it for `\fmfreuse`; `{fmffile}` sets `\thefmffile`.
  Witnesses (2-3 per name) in the code comments and repro headers. Residuals: an `\affil` copy's extra text (an
  `\email` written to `\AB@affillist`) is not in the frontmatter; `\@itemlabel` is still the kernel no-op; footmisc's
  symbol sets are read, not used (`\fnsymbol` is native); `{fmffile}` opened in the preamble and closed in the body
  (hep-ph0206138) is an environment-nesting error; the raw alignment code that calls array's internals
  (eqnarray.sty, revtex's ltxutil.sty, dgruyter) next reaches the rest of array's/latex.ltx's preamble builder
  (`\@mkpream`: `\prepnext@tok`, `\@classx`, `\@classz`, `\d@llarbegin`, `\NC@do`), undefined; IEEEtrantools' non-eqnarray
  half (`{IEEEproof}`, `\IEEEQED…`, `{IEEEitemize}`, `\IEEEPARstart`), which the IEEEtran class binding has, is
  missing from the package binding (2211.06691, 2310.15756). Guards `perfect_kernel_batch65z::*`; repros
  `package-bindings/*`.
  Integration 8 (65y + 65z, four review rounds): bundle A/B 912 papers (`ab_65int8`; 65y/65z witnesses + int7 sample)
  errors 3056→2138, Error status 499→175, Fatal 1→1, bibliography/tables equal, the 18 word drops error tokens leaving
  math; manual net 797 manuals 0 diffs, 30 repros fixed, 0 newly failing.
- **66b** (run-340 Fatal origins still Fatal on the int8 head; sample of 273 over the classes in
  `~/data/pk_agents/main/run340/fatal/`: Stomach:Recursion 22/25, PushbackLimit 22/25, Mouth:EoF 24/24, Malformed 24/25,
  Timeout:Recursion 19/25, IfLimit 11/23 still Fatal). Three mechanisms, 14 of the 22 Recursion witnesses Fatal → 0
  errors (~215 papers by the sample rate): the kernel's caption constructor is `\lx@@caption` with `\@@caption` its
  alias, since the Cambridge UP classes keep `\caption` under `\@@caption` for `\contcaption` (fac/ijsipaper/mscs; OD
  #566, KPE #637); `\newcommand`/`\renewcommand`/`\providecommand` with an optional argument also define latex.ltx's
  internal `\\cs` that the amsart idiom `\edef\cs{\@dblarg\csname\string\cs\endcsname}` calls (bcp.cls `\abtitle`,
  cmfne.cls; OD #567, KPE #638; review r2: every `\@xargdef` declarator, `\newenvironment` and `\DeclareRobustCommand`
  too); a locked robust binding locks its inner `\cs␣` against the files, so a class's copy of cite.sty's
  `\@namedef{cite }` no longer takes over the kernel `\cite` (cmfne, asme2e; OD #568); image fragments gobble
  `\lx@@caption` as well as `\@@caption` (latex_images.rs, LaTeXImages.pm:411). Guards
  `perfect_kernel_batch66b::*`; repros `captions-floats/cup_contcaption_lets_caption`,
  `macro-state/{newcommand_optional_internal_cs,locked_robust_inner_cite}`. Residual: AMS-TeX `\documentstyle{X}`
  never raw-loads a paper-local `X.sty` (Perl the same, `notex => 1`), so amams.sty's `\title#1` meets amsppt's
  `\title … \endtitle` and the file ends in the argument (math0402285, math0101270, math9906120; ~33 papers) — **FIXED
  78i** (OD #825, KPE #1051; quality ledger #37): a style of its own is read raw, one that `\input`s another style
  (gr-qc9507042's fic.sty) keeps the amsppt stand-in; gtmacros papers ship no gtmacros.tex
  (math9804003, math9902122, math9905203: harness).
- **66c** (run-340 `\endgroup` cluster, the empty-trigger frame: 937 papers have an `\endgroup` error whose frame was opened
  by no token, half of a 40-paper sample mdpi papers closing `\end{paracol}`): the mdpi binding follows the copy the
  paper ships — a copy that loads paracol (mdpi.cls:55) opens `\begin{paracol}{2}\switchcolumn` (the first page's side
  column) in its `\AfterEndPreamble`, after its own `\maketitle` (2105.08579's :880-887; 1809.00601's :908-912 via
  `\maketitlen`), so the binding loads paracol and opens it there too, and the template's `\end{paracol}` closes it (raw
  paracol.sty converts cleanly: the columns flow); a copy without paracol (1307.3961's 2013 one) and a paper shipping
  none are unchanged. And (R3 of the study, OD #569) a lock against the files holds for their `\let` as for their
  `\def` (`state::let_i`: arabtex's `\let\endabstract\endquotation`, 2401.14559), except a `\let` that only clears the
  name (projlib's `\cs_undefine:N \endabstract`: the homework/jwjournal manuals), whose binding comes back when the
  file's new meaning is refused; mn2e_support's `{keywords}` locks both halves against the files (mn2e-breakabs.sty's
  `\endkeywords`, 1403.3615, 1607.03149; plus mn2e.cls:992 `\ifSFB@keywords`). Residual (pre-existing, 65int8 too):
  `\usepackage{arabtex}` sets the whole body italic (its `\all@wcmd` font wrappers; pdflatex cmr10) — RED follow-up. Witnesses 2105.08579, 2201.05363. Guards `perfect_kernel_batch66b::{mdpi_*, file_let_*,
  refused_*_of_a_cleared_name_restores_the_binding, mn2e_*}`. A/B over 74 mdpi papers (`ab_66c`: 45 of 65v's, 29 of the empty-trigger
  list) vs 66b1: errors 154 → 100, Error status 37 → 17, none Fatal; the only word changes are error text gone
  (`{paracol} 2 \switchcolumn` printed by the undefined environment). Residual: 1905.04581's own `\begin{paracol}`
  after pgfplots figures raises paracol's `\ifinner` "Not in outer par mode" (2 errors; a mode left inner upstream,
  not reproduced minimally). The rest of the `\endgroup` study
  (`run340/clusters/endgroup_rootcauses.md`): R1 an eqnarray cell `a}{b` (latex.ltx:15760-15765 `{##}` templates; the
  gullet's before-column marker already resets align_state as tex.web §324, but real braces would put `{…}` into every
  eqnarray `tex` attribute: needs a hidden template group, ~86 papers), R2 diagrams stub body read raw inside an
  alignment cell (~51; the stub is a binding since 67d), R4/R5 singletons; the parked mode-switch family (29/56) beyond mdpi.
- **66d** (run-340 Fatal origins round 2, `~/data/pk_agents/main/run340/fatal/fatal_rootcauses.md`: 110 papers still
  Fatal after 66b, 16 mechanisms). Seven fixed, red/green with repros: R1 the caption-label hoist closes its
  `\ifx.#5.` before the caption is digested (latex.ltx's `\@firstoftwo` idiom; a stray `\fi` or open `\if` in a caption
  took the wrapper's `\else`/`\fi`; OD #570, KPE #641); R3 AMS-TeX `\Bbb`/`\bold` are font switches, amsfonts `\bold` is
  `\mathbf` (KPE #639); R4 AMS-TeX's `\abstract` drops the LaTeX kernel lock and amsppt.sty's state for amsppt1.tex exists
  (OD #571, KPE #642); R5 a driver with `\begin{document}` vetoes the preamble it inputs (main_tex.rs and cortex_worker.rs;
  OD #573); R6 `{filecontents}` ends at its `\end{…}` anywhere in a line (OD #572); R10 the kernel `\shorttitle`/
  `\shortauthor` are stubs a `\newcommand` replaces (OD #574); R13 `\lbrack`/`\rbrack` are macros (KPE #640); and, from
  the witness reconversion, amsppt's `\topcaption`/`\botcaption#1#2\endcaption` (amsppt.sty:820-836; q-alg9505010,
  q-alg9507011) and amstex.tex's `\Err@`. Bundle witness reconversion (user 2026-10-13: every arXiv id in
  `35feb675fa..` and the comments of touched code, 88 papers through the CLI's main-file pick, `~/data/pk_agents/main/
  wit66`, 65int8f vs 66d3): 38 Fatal → converted (Fatal 50 → 7, words 0.81M → 1.76M), 6 fewer errors (mdpi, mn2e,
  arabtex), 39 identical (incl. 2406.14142, 1905.05350, 2307.13586, 2606.02077), cond-mat0001395 changed for the better
  (jpsj.sty's raw cite copy no longer takes `\cite`: 0 → 51 `bibref` citations, the "?" marks gone); manuals latex4wp,
  gaceta, homework, jwjournal byte-identical. Errors 766 → 1006 only because the R1 papers now convert and show their
  authors' stray conditionals (1904.11818 594, 2012.08626 112, 2210.17505 97, 1602.01884 20; pdflatex reports each).
  Still erroring: AMS-TeX `\tag 1.2$'$` (1012.3760 Fatal → TooManyErrors, 1108.6264 3, 1112.1423 4; RED
  `amstex/amstex_tag_with_math_text`, next — fixed in 66e); 1905.04581 (paracol `\ifinner`, known); unchanged pre-existing errors in
  2304.05575 (`\BAmulticolumn`), gr-qc9405068/gr-qc9406008 (the author's `\beginabstract` pair), gr-qc9507042
  (merged creators), math0012150 (active `@`), 2401.14559 (3), and the AMS-TeX amams/gtmacros Fatals of the 66b residual.
  Bundle A/B (1009 papers, `~/data/pk_agents/main/ab_66bundle`, 66cr2 vs 66d2): identical tallies (E 2241, F 1), 12
  papers' content changed. Six lose the running-head paragraph the kernel `\shorttitle`/`\shortauthor` had leaked into
  the body (numapde 1908.02022, 2011.03580, 2301.02516; jnsao 2011.04474, 2001.07609; jpp-AAS 2305.03275; titles and creators
  unchanged); five print amsfonts `\bold` bold (1210.0557, 1811.00558, 1903.00970, 2005.01953, 2303.01678); and
  2210.02246 had lost cmfne's end-of-document author lines — 66c r3 skipped the `\@xargdef` internal of a refused
  definition, which the replayed `\@dblarg` setter calls; restored (OD #567, guard
  `perfect_kernel_batch66b::refused_dblarg_setter_reaches_its_internal`). Manual net (65int8f vs 66d4, `mnet_66d4`): 0
  regressions, 13 repros fixed; 4 manuals changed, all from 66c r4 sending every locked binding down `\let`'s plain
  path, which leaves plain `:locked` to `\let` — bera.sty:12-23 `\let\LaTeX=\fveLaTeX` (robust) replaced the `\LaTeX`
  logo constructor with the package's typeset boxes (enumitem, colorspace, mathador-doc; tikzbrickfigurines-doc's node widths). The skip
  now covers a locked inner only; the 4 manuals are byte-identical to 65int8f again (OD #568, guard
  `perfect_kernel_batch66b::robust_let_over_a_locked_constructor_keeps_it`); no witness or A/B paper's logo count moved.
  Guards `perfect_kernel_batch66d::*`,
  `main_tex::tests::{driver_vetoes_the_preamble_it_inputs, wrapper_without_begin_document_keeps_its_main}`. Open:
  R2 aastex `D` vs preloaded dcolumn (~50), R7 xargs robustness (~21), R8 `\enddocument` tail (~21), R9 hyperref
  `\H@refstepcounter` (~12), R11 qbordermatrix IfLimit, R12 shipped core-file copies, R14-R16 singletons.
- **66e** (AMS-TeX `\tag`, the 66d "next"): `\tag` was `\let` to `\eqno` (Perl too, KPE #643), whose tag is math up to
  the first `$`, so `\tag 1.2$'$` closed the display ("Missing $" cascade). amstex.rs `amstex_tag` reads the tag text as
  amstex.tex:597-605 does — unexpanded and brace-balanced up to the display's `$$` or, in a row (:629-640, 718-739, 797;
  a `\multline` tag follows `\endmultline`, :903, a display tag), to `\\`/`\cr`/the environment's `\end…` — and sets it
  with amsmath's `\tag` markup (`\tag*` for a quoted `"…"`, `$…$` under `\TagsAsMath`, amstex.tex:590-593, whose amsppt
  no-op stubs are gone). With no equation row (a plain `$$` display, also `$$\align` = amsmath's `aligned`) the tags
  float to the display (amstex.rs `\lx@equation@settag@float`, in the AmSTeX pool so a plain `\input amstex` + `\input
  amsppt.sty` document without the LaTeX pool has it; `\documentstyle` makes the route LaTeX, whose amsfonts
  `\DeclareSymbolFont` loads LaTeX.pool); one built in an `aligned` row is tied to it, and `rearrange_lone_ams_aligned`
  gives it to that row's equation. A row's tag ends at every row ender (`\displaybreak`, `\allowdisplaybreak`,
  `\intertext`, `\vspace`, a token meaning `\cr`/`\crcr`/`\\` or a macro expanding to one), the row test needs an
  alignment opened in the display's own math, and a delimited `{…}` argument loses its braces (`\tag{"A"}$$` quoted). In
  the `$$` form a `\par` abandons the call with TeX's runaway error and a `}` closing an outer group with TeX's two
  (tex.web §395-396: "extra }", then the inserted `\par`'s runaway; the input goes on with `\par }`); a row's tag (a
  LaTeX equation row's too) takes no argument, so there a `\par` outside the tag's own groups, or such a `}`, ends the
  tag cell without error; a `\par` inside a group of the tag that closes stays in its text (`\tag{3\par}\\`), and a
  row's or environment's end read inside an open group of the tag (`\tag{5\\ c&=d\endalign$$`) stops the scan at once:
  TeX's "Missing } inserted" per open group (tex.web §1126-1127; the runaway error if a `\par` was read in a group
  still open), the tag keeps its text with its groups closed, and the rest is read again, so no row, enclosing group
  (`{\it …}`, `\vbox{…}`), later paragraph or `\enddocument` is lost (r3 read them into the tag; r4 still did when an
  enclosing group's `}` came first); the document's end (`\enddocument`, `\bye`) is never read into a tag (a row end;
  in the `$$` form TeX's "Forbidden control sequence", the call dropped), and a `$` closing inline math in a tag is never
  the start of `$$` (`\tag{$a$$b$}\\`); the shapes where TeX errs and ours recovers are OD #575. In a LaTeX equation row a braced
  `\tag{…}`/`\tag*{…}` is amsmath's (an `\input amstex` or `\NoBlackBoxes`-autoloaded LaTeX document read `a=b` into the
  tag before 66e r3). r2 A/B (66d6 → 66e r2, 122 AMS-TeX zips): E 721 → 692, F 5 → 5, no paper worse, the 17 r1
  regressions (math9412232, hep-th9308082, hep-th9206021, hep-th9111004, 1102.5649, …) back to 66d6's counts or better;
  200 LaTeX `\tag` zips unchanged. Witnesses (66d6 → 66e, `wit66`): 1112.1423 E 4 → 0, 1108.6264 E 3 → 1 (merged
  creators), 1012.3760 still E 101 + TooManyErrors, but its first error moved from line 134 to 904 (lines 1-902: E 7 →
  1, all 60 tags text, 57 on their row/equation vs 51): the next class is `\Sb … \\ … \endSb` after `\input pictex` (RED
  `amstex/amstex_pictex_sb_newline`). A/B over 30 AMS-TeX `\tag` papers: no paper worse; math0001138 and math-ph0001014
  E 2 → 0. Residual: `$$\gather … \tag 1\\ … \endgather$$` keeps both tags on the one display equation (amsmath's
  `gathered` is not split into rows); AMS-TeX tags sit on the left in the PDF (`\TagsOnLeft`, amstex.tex:590), ours too
  since 78c (KNOWN_PERL_ERRORS #1025); `\intertext` inside `$$\align` errs (RED `amstex/amstex_align_intertext`, before 66e too); `$$\gather` in a
  plain document erred "`\ams@return@opt@arg` undefined" (amsmath_sty.rs `\lx@ams@start@box` let it to `\@empty`, which
  only the LaTeX pool defines; math9412232 and 6 more of the 122, 36 errors, before 66e too; fixed in 68r, the kernel's
  `\lx@empty`: `amstex/amstex_plain_gather_return_opt_arg` GREEN); an unbraced inner alignment in a row tag (`\tag 1\matrix a\cr
  b\endmatrix\\`) stops the tag at the inner `\cr` and errs; an active character let to `\\` (`\let~\\`) does not end a
  row tag (`amstex_tag_row_end`'s `Catcode::CS` gate); a `\par` in `$$\align … \endalign$$` raises no error (TeX's
  `\align` reads its body as an argument: "Paragraph ended before \align was complete"; before 66e too); an amsppt
  `\footnote{` whose `}` never comes reads to the file's end, the later paragraphs inside the `<note>` (amsppt_sty.rs
  `\footnote[]{}` reads `\long`, amsppt.sty:804/:814 `\nextiv@##1` does not; FIXED 77v, KNOWN_PERL_ERRORS #1012,
  OXIDIZED_DESIGN_DIVERGENCES #782, `amstex/amsppt_footnote_argument_runs_past_par` GREEN); an alignment open at `\enddocument` drops its open row's last cells
  without an error (RED `amstex/amstex_align_unclosed_at_enddocument`, before 66e too); a display open at `\enddocument`
  takes `\enddocument` into its formula's `tex` (RED `amstex/amstex_display_unclosed_at_enddocument`, before 66e too). Guards
  `perfect_kernel_batch66e::*`.
- **66f** (sandbox regressions, cortex runs 341/342 on 7a93c36531 vs the 2026-10-04/05 statuses; reports
  `agent_reports/2026-10-13_sbx2605_regressions.md`, `_sbx2606_regressions.md`): of 412 (2605) + 444 (2606) OK/Warning →
  Error/Fatal, 375 + 416 are intended signal (BibTeX undefined macros and parse errors, OD #518; `merged_creators`, 63e),
  3 + 4 fleet-only OOM/xpath; the rest root-caused. Fixed: `\begin{document}` opens `\jobname.aux` as latex.ltx's
  `\document` (OD #576; jpsj3/bio/lamuphys/caosp `\enddocument` re-reads; 2605.24927, 2605.04825, 2605.28673,
  2605.20125, 2606.02098, 2606.24241, 2606.25242, cond-mat9607109, astro-ph9805185); `\mathversion` expands its name
  (KPE #646; XCharter/figureversions, 2605.11518, 2605.12374, 2605.22763); an unnumbered appendix unit in a float
  (OD #189; 2605.02378, 2605.25534); authblk `\affil` robust (KPE #647; 2605.03796, body 15 → 7,748 words); SNmult.cls →
  svmult binding (2605.13070); url `\path` only if undefined (KPE #644; 2605.13185); jmlr `\jmlrissue`/`\jmlrpages`
  (2605.20066); IEEEtran `\appendices` through a copy of the kernel `\appendix` (2605.23316, Fatal → 0); nicematrix
  `[opacity=…]` color keys (2605.25123); braket `\set` splits only at `|` (KPE #645; 2605.31379). Every witness 0 errors,
  text equal or recovered. From 2606: natbib `\citealp`/`\citealt` and the capitalised forms through private copies of
  `\citep`/`\citet` (natbib.sty:702-730; since D1 a document's `\renewcommand{\citep}{\oldcitep*}` owned them,
  2606.16139 E 11 → 0, 2606.30583 E 69 → 0); calc's `<optional equals>` (2606.05909); appendix.sty internals (KPE #648,
  2606.09094); amsart `\@classname` (read in amsart/amsproc/amsbook's own load: the loading class) and `\subjclassname@<year>`
  (KPE #649, 2606.20943); IEEEtran's author pre-scan keeps a
  run whose `\else`/`\fi` close an `\if` from a block argument whole (2606.17718; residual: an empty affiliation contact
  from the skipped branch); `\hypertarget`: hyperref.sty:4805-4811 (nesting off) puts `#2` after the anchor in every mode; our mode split keeps
  Perl's anchor-holds-its-text in text, and in math an empty anchor with `#2` placed after the `\ifmmode` test, so a
  brace-less `\frac` takes its own arguments (hyperref's `#2` precedes its `\fi`; a `\relax` first keeps a row-initial
  call in `align`/`aligned`/`array` on it; OD #578; 2606.15438);
  braket's split never inside `\left…\right`, the count never below zero (2606.06185, KPE #645). Each 2606 witness
  E → 0. Guards `perfect_kernel_batch66f::*`. RED from its review (pre-existing on 66e1):
  `package-bindings/hypertarget_math_anchor_keeps_its_id` (a math anchor's `xml:id` is generated, so `\hyperlink`
  dangles, and the anchor parses as an operand), `fonts-nfss/mathversion_normal_restores_the_normal_version`,
  `sectioning-frontmatter/unnumbered_unit_in_float_pins_the_float_id` (Ax1.F1 for A1.F1),
  `sectioning-frontmatter/chapter_star_in_float_is_an_inline_block` (`malformed:ltx:chapter` in report),
  `package-bindings/biblatex_natbib_capitalized_cite_commands` (`\Citet` undefined; blx-natbib.def:49). Open (2605 report): C3 nomencl glossary in a float, C6 standalone sub-preamble loads, C7
  `\mathcolor` group, C8 bigfoot listing in a footnote, C11 JASA revtex4-2 internals (`\@AF@join`, `\@email`), C12
  2.09-style `\ds@<opt>` handlers (KPE-class, Perl Package.pm:2470), babel's on-demand Lithuanian `\fnum@table` loop
  (2605.29738; our kernel lacks `\fnum@figure`/`\fnum@table`), revtex4-2 `\frontmatter@footnote` (2605.04155); from
  2606: N1 scrhack/floatbytocbasic patching our primitive `\newfloat` (2606.04906, 2606.11358), N6 the expl3 scope-exit
  heuristic firing on a package that sets `_` a letter itself (etextools, 2606.06323), N7 NFSS shape probes
  `\try@load@fontshape`/`\DeclareFontShape` no-ops (2606.07112, Perl the same), N10 a sectioning unit in `ltx:quote`
  (2606.19241) and an `\item[]` excluded from the OD #189 item bridge (2606.00118) — both await a ruling.
- **Runs 346/347** (E/F rerun on `3ad0bbe622`, 66f-66h; vs runs 344/345 end): E 105,835 → 101,776, F 4,625 → 4,478;
  E→OK 1,290, E→W 2,814, F→E 171, F→W 97, E→F 126 — none a regression (agent_reports/2026-10-14_ef347_triage.md: 113
  not reproducible locally, fleet contention at the 5632 MB child cap / 180 s on an E/F-only rerun of the heaviest papers;
  13 long-standing slow or memory-heavy). Open from it: libmarpa `marpa_g_new` aborts on allocation failure with no
  Fatal line (SIGABRT; fix in the marpa-asf fork: an overridable OOM hook → `Fatal:oom:alloc_failed`), XPath OOM floods as
  one `Fatal:oom`, a 2.7-3.1 GB peak RSS for 4-10 s papers. Runs 348/349 (E/F rerun on `a7f5e7308e`) started 13:15Z.
- **67d** (Paul Taylor's `diagrams`, not in TeX Live, in arXiv's tree; run 349: `\diagramstyle` 467, `\newarrow` 441,
  `\rTo` 362 papers undefined): a native binding (latexml_contrib/src/diagrams_tex.rs, after amscd's `{CD}`; surface
  from the package manual v3.97 and the 100 witnesses' use) where Perl's ar5iv stub put `<ERROR>{diagram}</ERROR>`:
  `{diagram}`/`\diagram` is a math array of objects and labelled arrows (display in text, inline in a `c` cell or LR
  box, `[LaTeXeqno]` numbered, `\\` = the package's `\cr`), `\newarrow` draws each direction with the Unicode arrow
  of its components, `\pile`, text arrows, plain TeX `\input diagrams` (OD #589). An arrow reads the source's
  arguments: inside a diagram `(x,y)`, `<`, `>` too, outside one not (`$X\rTo (0,1)$` keeps its pair); a `.` and a
  positional `{g}` after `^f` as the source reads them; arrows and `\diagram` are mode-tested when digested, so one
  at an alignment cell's start is in the cell's maths (`&\rTo` in `align*`, math0404431); a horizontal arrow in text
  is the package's Error ("Commutative Diagram: "$" inserted before horizontal map"), a vertical one outside a
  diagram, in maths or text, its other Error ("vertical map ignored outside diagram"), a diagonal one none, as
  pdflatex; each arrow is kept (in text set as a formula); the `amstex` option (package option, `\diagramstyle[…]`
  or `\diagramstyle{…}`) loads amscd's `{CD}` (math0104206, math0403203). A/B of 102 witnesses vs 66x1: Errors
  468 → 86, Fatals 0 → 0, words +53k, 1,089 `<ERROR>{diagram}` → 1,267 diagram arrays (the 99 harness-run papers:
  1,083 → 1,250); no paper with more Errors/Fatals; 13 with 2-71 fewer words, all the printed arguments of the
  undefined `\newarrow`/`\diagramstyle`/`{CD}` (`@>>>`) and `<ERROR>` macro names.
  Witnesses math0001049, math-ph0001006, math0003019, math-ph0007001, math0010298, math0012073, math0102115,
  math0104065, math0104175, math0104206, math0105166, math0109209, math0203179, math0306018, math0310311,
  math0403203, math0404431, cs0204036; repros `package-bindings/diagrams_*`; guards `perfect_kernel_batch67d`.
  Open: the pullback corners (`\SEpbk`, a visible ⌟; CSS-placed corner proposed for the end-of-goal CSS review) and
  `\HonV`/`\VonH` draw nothing; `eqno=`, `\diagraminline` and the dotted `.` are not implemented; a vertical
  arrow's left label leaves its cell unparsed (`ltx_math_unparsed`, as amscd's `@VfVV`).
- **Fleet TeX tree** (2026-10-14, host change, no code): the cortex fleet reads the distro TeX Live
  (`/usr/share/texlive`, worker.env sets no TEXMF*), which lacked whole collections the vendor TL 2025 of the gates and
  A/Bs has, so some clusters were fleet-only (`{fmffile}` 298, `\Tree` 302 papers in run 349). Installed at the same TL
  version (2025.20260124-1, 0 upgrades): texlive-metapost, -humanities, -music, -games, -formats-extra, -xetex, and
  Debian's separate `feynmf`. The live dumps are unaffected (their source stamps name latex.ltx/expl3, unchanged).
  The full apt list (texlive-full is not needed) and the host check: `performance/CORTEX_WORKER_HARNESS.md` "Host TeX
  Live for arXiv coverage". Before calling a cluster a code bug, compare the distro `kpsewhich` with the vendor one
  for its package. E/F rerun
  on 4f131f2b84 started 20:13Z (baseline 2026-10-14 19:25:48.785430).
- **67g** (run 349 Error→Fatal regression 2109.06028): pict2e keeps `\@tempdimd`, which it allocated before v0.4b and
  which TeX Live's own ellipse.sty/longfbox.sty still use (TL 2025 pdflatex fails on both), loads trig as pict2e.sty:123
  does, and binds the graph-code layer (`\pIIe@add@nums`/`\pIIe@add@CP`/`\pIIe@addtoGraph`) ellipse.sty drives
  (OD #590; `perfect_kernel_batch67g`). 2109.06028: 643 errors, then the same-error cap's Fatal once 66l-67e removed the
  alignment errors that had interleaved, → 0. Run 349: 23 papers with undefined `\CalculateSin`, 18 with undefined
  `\pIIe@addtoGraph`. With it, 66w's look-ahead gives up at a `\catcode` (the tokens after it are not the ones TeX
  reads; `par_before_a_catcode_change_ends_the_formula`) and steps over only a `_` or `^` after a backquote
  (`par_before_a_backquoted_{formula,ref}_ends_the_formula`).
- **67j** (frequency-first, run 349 "undefined"): native bindings get their class's own user macros. espcrc (2004
  v1.2): `\readRCS` reads the `$Id$` stamp (espcrc1.sty:27-31), `\runtitle`/`\runauthor` store the running heads
  (:156-159). svjour `epjCONF` (svjour.cls:78-81 reads svepjCONF.clo): `\session-title` stored (svepjCONF.clo:234).
  imsart: `\let\endbibitem\relax` (imsart.sty:2514), every structured entry drops its `[author]` key list (:2585-2769,
  printed before every entry until now), `{binproceedings}`/`{bmanual}`, `\setattribute`/`\getattribute` (:464-466),
  `\thanksmark` (mark dropped). siamltex: `\slugger` stored (siamltex1213.cls:184-204). amsppt `\tenpoint`/`\eightpoint`
  (amsppt.sty:153-214), AmSTeX `\goth` (amstex.tex:1509). tcilatex: the binding (the 2.5 copy, chosen over a shipped
  3.0-5.0 tcilatex.tex as bindings are) gains `\TCItag` = amsmath's `\tag` (5.0 :776-783) and 4.0's `\U`/`\UNICODE`
  placeholders. In math `\eightpoint`/`\tenpoint` change the size only (the math families are resized, letters stay
  math italic: `\textfont\@ne\eighti`); the roman is text-only (1306.2523: 718 upright letters in r0 → 0). A row whose
  only content is its tag (Scientific Word's `… \nonumber \\ && \TCItag{27}`, amsmath's `\notag \\ \tag{29}`) is
  kept with its tag, where Perl drops it (KNOWN_PERL_ERRORS #682, OXIDIZED_DESIGN_DIVERGENCES #591; nlin0106009's 11
  such tags, lost in r0, are back). imsart's `\thanksmark` is locked. KNOWN_PERL_ERRORS #681; repros
  class-bindings/{espcrc_*,svjour_epjconf_*,imsart_structured_*,siamltex_slugger_*},
  amstex/amsppt_point_sizes_and_goth, package-bindings/tcilatex_tcitag_*, alignment/ams_tag_only_row_keeps_number;
  guards `perfect_kernel_batch67j`. A/B (82 papers, the 22 witnesses + ≤10 per family from the run-349 log scan): 275 →
  155 Errors, none worse. Every `\TCItag` in the 13 tcilatex papers is emitted as a tag. The words lost are the ERROR
  text, the `$Id$` math, the running heads, slug and session lines, "[author]", the `\U{…}` argument text (now the
  PDF's square), and tag text that sat inside the formula as math tokens and is now one `<tag>`. RED repro
  alignment/multline_notag_then_tag: multline ignores its `\tag` and `\notag` (always (n); hep-th0210191; Perl alike;
  FIXED 77v, KNOWN_PERL_ERRORS #1010).
  Residual in those papers, other mechanisms: plain TeX's
  `\settabs\+` under the LaTeX route (alg-geom9410018, hep-th9306053), amsppt internals papers copy
  (`\textonlyfont@`, `\ifsyntax@`, `\eightrm`: funct-an9706006, math0002200), imsart `{supplement}` without
  imsart.sty (1602.02055).
- **67l** (frequency-first, run 349 "undefined": `\State` 560, `\For` 470, `{algorithm}` 544, `\newfloat` 365):
  which algorithm and float definitions a paper gets when a binding stands in for the file that loads them. A stand-in
  binding takes the algorithm packages from the paper's copy of the file it stands in for
  (`shipped_class::require_shipped_algorithm_packages`, the dependency scan kept to `ALGORITHM_PACKAGES`:
  `require_dependencies_only`, content.rs; OD #592): icml2016-2025 and versioned ICML names (papers edit `algorithmic`
  out of their copy for algpseudocode, which the distributed pair bailed: 1706.03643, 2002.06473, 2106.05074),
  WileyNJD-v2/v5 (`algorithm, algorithmicx, algpseudocode`, a copy algorithm2e: 1810.03033, 2106.06046, 2402.13183),
  bmvc2k (algorithmic or algpseudocode by year: 2210.06270, 2210.01640), pnas-new (2402.10236), edited neurips and
  siamltex copies (2402.18905, 2210.03478); without a copy, ICML keeps the distributed pair, the others none. REVTeX
  4.1/4.2 and AASTeX 6+ define the class's `\newfloat@ltx`/`\newfloat@float`, which preambles let `\newfloat` to
  before algorithm.sty (all 25 sampled `\newfloat` papers: 2210.07980, 2210.08566, 2002.07183; KNOWN_PERL_ERRORS #683).
  autart gets the class's own `{algorithm}` float (autart.cls:568-577; 2106.14831, 2402.17000). The scan also drops a
  comment ending a file without a newline. Repros package-bindings/icml_style_copy_without_algorithmic,
  class-bindings/{revtex_float_repair_before_document,wileynjd_class_copy_loads_algorithm_packages,
  bmvc2k_class_copy_loads_algorithmic,autart_algorithm_float}; guards `perfect_kernel_batch67l`. A/B vs 67k1 (144
  papers: the run-349 `\State`/`\For`/`{algorithm}` sample, 31 more ICML papers 2017-2026, 7 Wiley, 3 bmvc2k,
  8 neurips): 3,118 → 279 Errors, Fatal 1 → 0, 83 better, 2 worse, both a decoy main the harness picked (the ICML
  template's example file, which needs the algorithm packages the paper's copy no longer loads: pdflatex 16 and 15
  errors; the real mains 14 → 0 and 15 → 0); revtex/aastex (31): 73 → 2. Words lost: 2301.00557's float now comes from
  its own algorithm2e alone (its copy loads no algorithm package). Residual, the paper's own combination: algorithmic
  loaded before algpseudocode (1706.04918, 2506.05791, 2506.13672, 2506.01215) bails algorithmicx as in Perl, where
  pdflatex errs too ("Command \algorithmic already defined", then each `\State`). Not covered: IEEEtran (a Perl
  binding) ignores an edited copy's algorithm packages (2506.04683). Perl's versioned fallback answers a paper's
  icml20xx.sty with icml.sty.ltxml and forces the pair too (KNOWN_PERL_ERRORS #684): the ICML part is beyond Perl.
  Open from the review: a nested dependency scan reads as "no copy" (`require_dependencies_filtered` returns `false`
  under the re-entrancy guard, so ICML forces the pair over a shipped copy loaded from an unbound class's scan); the
  pnas-new, siamltex and icml2019 raw-read paths have no guard; autart's caption period sits outside the bold
  (autart.cls:568-569 `{\bfseries #1.}`).
- **67m** (fleet throughput, run 351): an allocation that fails when the address space is spent no longer wedges the
  worker. `custom_alloc_error_hook` (cortex_worker.rs, latexml_oxide.rs) formatted a backtrace under the fleet's
  `RUST_BACKTRACE=1`; symbolizing it allocated while holding the standard library's backtrace lock, the failure
  re-entered the hook, and the re-entry waited forever on that lock — with the stderr lock held, so the watchdog's
  `Fatal:timeout` line blocked too and its 180 s exit never came. Children sat parked (conversion and watchdog
  threads both in a futex wait, 3-5 GB resident) until the harness's 360 s no-CPU-progress SIGKILL, which re-leases
  the paper as a retryable death: ≥574 such kills in 4 h of run 349/351 (journald dropped ~1M lines per 30 s, so a
  lower bound), 17 of 53 children parked at one sample. A re-entry that did not deadlock recursed instead: one child
  printed 6,048 nested `Fatal:oom:alloc_failed … 1 bytes (align 1)` lines in a millisecond (reading `RUST_BACKTRACE`'s
  value). The hooks now allocate nothing and take no lock (`watchdog::write_stderr_unlocked`, a stack-formatted
  `write(2)`; no backtrace), a re-entry ends the process at once and another thread's failure waits for the first's
  exit (`watchdog::enter_alloc_failure_hook`), and they end with `watchdog::exit_now` (`_exit`). The watchdog writes
  its two Fatal lines unlocked too. The harness child also registers pericortex's per-task input directory
  (`cortex_task*`, a `TempDir` its loop drops) for the exit sweep: every timeout and OOM exit had left one behind —
  565,483 of them, 434 GB, in the fleet's `/opt/cortex-scratch` since 2026-07. Stuck-child witnesses (fleet 4f131f2b84,
  symbolized by rebuilding that commit with `STRIP=none` and the same embedded dumps): 1804.07640, and the tasks
  33865595 (math parse, `XM` clone under ASF traversal) and 32779664 (`SymHashMap::insert`); 1804.07640 converts in
  13 s standalone, so the state is a long-lived child's. Guards (need `--features cortex`, which CI and the default
  suite do not enable) `cortex_worker_exit_reports::{an_allocation_failing_inside_the_hook_still_ends_in_one_fatal_oom,
  a_watchdog_timeout_is_reported_as_fatal_timeout, an_allocation_past_rlimit_as_is_reported_as_fatal_oom}` (the
  `alloc_exhausted` test fault spends the address space through the raw allocator first). Residual: a hook whose sweep
  itself fails to allocate exits without finishing it.
- **Runs 366/367** (E/F rerun of run 365 on 5059484050: int10-int31; MARKED 2026-10-19 09:43:34Z, 81,903 tasks =
  E 77,932 + F 3,971 of 2,922,325): triggered once the combined projection reached 98.06 % (int29a, gate ≥ 98.05 %), the
  perf/RAM pass had landed (int12) and every numbered silent-loss row was closed (int31). Fleet: `cortex_worker`
  maxperf-cortex built at 5059484050, distro dumps of 2026-10-11 (sorted-set identical to the int31a vendor dumps).
  COMPLETE 2026-10-19 12:57:33Z: E 77,932 → 53,977, F 3,971 → 3,068 (success 97.20 % → 98.05 % of 2,922,325, the
  int29a projection said 98.06 %). Transitions vs run 365: E→OK 4,130, E→warning 19,986, E→F 69, F→OK 46,
  F→warning 696, F→E 230. E→F re-converted locally (67int9 = run 365's code vs int31a, cores 64-127): 49 are Fatal on
  int31a and not on 67int9 — real regressions introduced between 026cf18f87 and 5059484050 (TooManyErrors 31, most
  from a handful of errors to >100; Timeout:Convert 6; PushbackLimit 5; IfLimit 2; Recursion 3; Mouth:EoF 1;
  Document:Malformed 2207.09187; a crash 2504.05919); 17 are non-Fatal on both binaries (fleet artifacts), 1 Fatal on
  both. Bisected over the 39 integration binaries 67int9…int31a (all deterministic; scratch
  `~/.claude/jobs/356a0c47/tmp/rc_ef367/`, `bis/<id>/steps.tsv`, pdflatex verdicts `pdf.tsv`): 21 first fail at 70b A1
  (int20, tex.web §1127 `align_error`: faithful itself, it turns brace-ledger mistakes upstream into "Missing { / }
  inserted" cascades). Mechanisms, papers: **C1** old-form `\pmatrix`/`\matrix`/`\cases` under amsmath (11; ptptex read
  raw since 67v loads amsmath, ptptex.cls:362), **C2** `\binom` through the document's `\atop` (5), **C3** revtex
  binding preloads hyperref, phfnote's deferred `\hypersetup` loops (5, int29b), **C4** `setstack.sty` in neither TL nor
  the papers (3), **C5** proof.sty premises not proof's own `\halign` (2), **C6** emulateapj deluxetable nested in an open
  one + `\label`, IfLimit (2, int18), **C7** easybmat `\lastbox` (2; RED
  `boxes-groups/easybmat_lastbox_alignment_rows`, kernel), **C8** siunitx `\DeclareSIUnit\%{\char`\%}` stack overflow
  (2504.05919), **C9** wds10 abstract replay vs `\pickup` (0910.4848), **C10** crckapb bibitem label autoloads natbib in
  the tag (astro-ph0007279), **C11** 9-argument `\p@equation` tag forms recurse (1902.09345), **C12** T4 encoding without
  fontmap loops NFSS (2405.01972), **C13** a second `\documentstyle` reloads the class (astro-ph9812142), **C14** minted
  bodies through listings take the document's `mathescape` (1812.09490), **C15** VisionStyle speculative title-page run
  (astro-ph0203021; pdflatex hangs too — ruled: the runaway is discarded with the speculative run), **C16**
  `\renewcommand\thesection{\chapter…}` in article (2209.15365; pdflatex 244 errors; no change). Not regressions (11):
  2207.09187 empty before (282 errors, now honestly Malformed); pdflatex fails too — siunitx S column holding a nested
  tabular (2011.09784, 2103.15543, 2105.10358), TL 2025 now defining `\clap` (1011.2267, 1308.3100), `\Ref`
  (math0204358, 1603.09649), `\robustify` (2003.07558) that the document `\newcommand`s (ruled: LaTeX's
  `\@notdefinable` stays, only our Rust-only loops there are fixed), pst `\of` taking a `}` (1512.03873, 731 errors),
  `vanilla.sty` missing (math9209216). Fixes: C1, C2, C4, C5, C14 in batch 77q; C3, C6, C8-C13, C15 and the
  redefinition loops in 77r.
  Side finding (fleet harness, upstream): pericortex 0.2.8 `harness.rs:642-670`'s unresponsive-worker watchdog SIGKILLs
  IDLE workers — a 60 s idle poll uses under one clock tick of CPU, so the tick count never advances (seen
  2026-10-19 09:41Z) — respawn churn only while the fleet is idle; fix upstream: arm the watchdog only while a task is
  leased.
  - **77q** (run-367 E→F regressions, part 1: triage clusters C1, C2, C4, C5, C14 — 21 papers, all Fatal on int31a):
    every one converts with no Fatal (int32a → 77q). C1 amsmath's old-form `\pmatrix{…}`/`\matrix{…}`/`\cases{…}`
    (KNOWN_PERL_ERRORS #993): `\lx@ams@matrix@check` (amsmath_sty.rs, amsmath.sty:1077-1090) gives the environment or
    amsmath's "Old form" error once and Plain TeX's matrix with its rows — hep-ph0110005, hep-ph9509392, hep-ph9511429,
    hep-ph9702336, hep-th9908038, nucl-th0305048, 0805.4522, 1003.3524, hep-th9910005, nucl-th0206027, quant-ph0607049
    (101 + Fatal each), and 1906.07043 of run 363 (101 + Fatal → 1). The error is amsmath's, so only where the real
    amsmath.sty loads: the 7 ptptex papers (ptptex.cls:362) keep it, as pdflatex does (hep-ph9511429 4 = 4,
    nucl-th0305048 2 = 2); a binding that loads amsmath where its original does not — spie (1003.3524), the old
    ws-procs9x6/ws-ijmpb copies (nucl-th0206027, quant-ph0607049), Scientific Word 2.5's tcilatex (0805.4522), OmniBus's
    autoload, the old aa.cls, and the audited sn-jnl, imsart, svproc, jfm, autart, ecai, egpubl, agujournal2019,
    wileymsp-template, asme2ej, tac, uai2025, gretsi, fcs, dmtcs-episciences, achemso, combine (copies in a 1/20 arXiv
    sample load none, directly or through a package; the paper's copy decides; optica-article, bytedance_seed, fairmeta
    and selfevolagent load it through newtxmath, nicematrix or `tcolorbox[most]` and keep the plain load) — goes through
    `amsmath_sty::require_amsmath_beyond_original`: amsmath's definitions in force, no old-form error,
    `\@ifpackageloaded{amsmath}` false, until a real load (the document's, a raw file's, a copy's, mathtools') runs the
    binding again (aa.cls.ltxml:57's reload, generalised; those 4 papers → 0 old-form errors, pdflatex 0) — a file whose
    original loads it is that real load even after a beyond-original one (spie.cls, then a Scientific Word 5.x
    tcilatex.tex copy). A mid-document reload (an OmniBus autoload of ams_support for `\subjclass` under spie) turns the
    error on for later old forms where pdflatex has none (it fails on `\subjclass` there; no clean witness). The dispatch
    replaces aa_cls.rs's own copy, AMS-TeX keeps `\matrix…\endmatrix` (amstex.rs; 1411.5748), gauss's `gmatrix` and
    physics' matrices open their amsmath environment by `\begin` as the packages do. C2 `\binom` on the kernel's `\atop`
    body, not the document's `\atop` (#994): 1010.0429, 1209.4365, 1308.4325, 1806.09142, 1904.01263 → 0 Errors; an
    `\atop` redefined before amsmath too (the binomials intact, 0 errors; pdflatex adds amsgen's "Unable to properly
    define \@@atop"). Open: amsgen's `\@saveprimitive` check (amsgen.sty:27-38) stays Perl's `\let` — `\meaning` reads
    the kernel's `\over` family as primitives (#985) but not `\eqno`/`\leqno`, which 0706.4029's amsmath-iop.sty saves
    (2 false errors with the check); porting it needs those, and every name a raw file saves (breqn's mathstyle.sty
    `\displaystyle`, `\mathchoice`), to read as primitives first. Also amsgen.sty:63's `\@ifstar` (a `\new@ifnextchar`,
    no space skipping) is not ported: after amsmath loads, LaTeX's skipping `\@ifstar` stays (no witness). C4 a
    `setstack` stand-in (latexml_contrib; IOP 1998, not in TeX Live) with the amsmath bindings' own `\substack`,
    `\overset`, `\text`, `\boxed`, … (`amsmath_sty::define_ams_setstack_commands`, `define_ams_matrix_base`,
    `amstext_sty::define_amstext_text`): 1008.1543, 1306.6377, 2206.02651 → 0; papers that ship setstack.sty now get the
    bindings' markup (0803.1550, 0812.2552, 1303.5499, 1609.03822, 1702.05765, 2203.11741). C5 proof.sty's premises
    split at an active `&` and keep their `\\`/`\cr` rows in an `XMArray` (#995): 2011.13396, 2205.02142 → 0;
    1709.05725's `\\[1pt]` rows too; a `*`/`[…]` after a space is the premise's text (`s \\ [t] u` keeps `[t]`, amsgen's
    non-skipping `\@ifstar`). C14 minted bodies start from minted's own escape defaults, not the document's `\lstset`
    (minted_sty.rs `MINTED_LST_RESET`; listings' `texcl=false` now clears as Perl's does): 1812.09490 → 0. A/B vs
    int32a: the 21 + 1906.07043 above, 150 control papers using amsmath matrices/cases, `\binom`, proof, minted or
    setstack, and 16 code-comment witnesses — 0 worse. Review round (r2) A/B vs r1 over those, 1709.05725, 1411.5748,
    1008.1366, 2212.12000 and 126 papers of the changed classes (1/20 sample copies; every pop67int23 paper with an old
    form): 302 of 312 byte-identical; 0805.4522 22 → 0, 1003.3524 16 → 0, nucl-th0206027 1 → 0, quant-ph0607049 1 → 0,
    0705.3096 (ws-ijmpb) 11 → 0 errors (XML identical; pdflatex 0 old-form errors each); 2002.04115 takes
    imsart.cls:72-74's `\PassOptionsToPackage{leqno}{amsmath}` (`ltx_leqno`, as pdflatex's amsmath gets it); 1402.0250,
    1704.02959, 2605.27010 a `\pgfsys@hbox` register number one higher in a `tex=` (amsmath's binding run again,
    its `\newbox\strutbox@`); 0 worse. Guards `perfect_kernel_batch77q::*` (11). Residual: a premise row's `tex=`
    attribute reads `\\[t]u`, which a re-read takes as a row skip (the `\infer` formula's `tex=` was already poor:
    `C\lx@proof@logical@and a=bx`); after a `\\` outside amsmath displays (`{array}`, `{eqnarray}`, no amsmath) LaTeX's
    `\@xarraycr` skips spaces before `[` (latex.ltx:16585) where we keep ` [t]` as text (no content lost).
  - **Fixed in 77r** (run-367 regressions part 2, triage clusters C3, C6, C8-C13, C15; guards
    `perfect_kernel_batch77r::*`): **C3** the revtex support's hyperref preload is marked unloaded, so the document's
    own `\usepackage{hyperref}` runs the binding (phfnote `hyperrefdefs=defer` looped in the one-time begin-document
    hook: 2207.13707, 2403.04828, 2508.03993, 2508.03994, 2601.20155; KNOWN_PERL_ERRORS #997), the binding taking
    `\H@refstepcounter` from the kernel's own `\refstepcounter` as hyperref.sty:6651 does (a second save of the current
    one looped under cleveref) and accepting hyperref's `runcolor` key;
    **C6** a deluxetable begun inside an open emulateapj one raises `\@parmoderr` and keeps the outer's `\label`
    (IfLimit: 1704.06321, 1204.1969; the inner table kept, as Perl and 67int17b, where pdflatex drops it); **C8** a
    siunitx unit named by its own character code (`\%{\char`\%}`, atlaspackage.sty:225-227) is the character, not a
    recursion to a stack overflow (2504.05919, exit 134), a unit's own `\DeclareSIUnit[<options>]` apply where it is a
    command's whole unit argument (siunitx.sty:6684-6694, OD #774; Perl stores and ignores them: 2504.05919's "85%", 1501.03532's
    `number-unit-product = \ `), and a presentation keeps the space ending a control word (`\char`\a b` "ab", not "a";
    `\textmu m` not an undefined `\textmum`); **C9** a refused begin code calling a macro whose delimiter does not occur
    later in it is not replayed (wds10 `\pickup#1\end#2`, 0910.4848) — only the code's own calls are judged: outside
    braces, not a definition's operands (`\def`/`\let`/`\newcommand` targets, `\let`'s value — also at the code's
    tail, `tail_reads_ahead`), not one after `\expandafter` or before `\meaning`/`\the`/`\string` (repro
    `sectioning-frontmatter/refused_begin_code_definition_operands`); **C10** an autoload a `\bibitem` label calls loads its package before the
    entry (crckapb `\citeauthoryear` → natbib, astro-ph0007279; OD #750 addendum); **C12** `\errmessage` shows
    `\errhelp` unexpanded, tex.web §1283 (t4enc.def's `\m #1` help called `\m` for ever: 2405.01972);
    **C13** a second `\documentstyle` raises LaTeX's "Two \documentclass or \documentstyle commands" and loads no
    class, unless the first fell to OmniBus (no binding, no file: astro-ph9812142's `#\documentstyle[…]{/disk/…/aa}`
    line, then the real aa — 1 Error, 3,433 words as 67int19b), latex209.def's definitions applied once (OD #773); **C15** a runaway limit (pushback, recursion) inside a speculative title-page run
    is discarded with the run, its box lists and conditionals unwound (astro-ph0203021; user ruling 2026-10-19, OD #767
    addendum). (C11) FIXED 77u for the witness: a `\p@<counter>` form is applied to the expanded `\the<counter>` as
    latex.ltx's `\@currentlabel` is (`\protected@edef`; `\lx@@p@the@@`, OD #778, KPE #1001): 1902.09345 18 → 2
    Errors (the two are `ltx:chapter` in `ltx:text`, base too), its first SI equation back, refnum "S1"
    (`parameter-conditional/p_form_applies_to_the_expanded_number`). Residual RED: the form is still typeset at the
    step, where pdflatex typesets it only at a `\ref`, so a nine-argument `\p@equation` before a one-token number still
    reads past its tag (8 errors, `parameter-conditional/p_equation_taking_nine_arguments`): our argument reader takes a
    `}` as an undelimited argument (`\def\g#1#2#3{[#3]}{\g a}b` prints "[b]"; Perl Gullet.pm:732-744 the same) where
    tex.web §395 stops with "Argument of \g has an extra }" and §396 drops the call — a gullet change of its own. (C12)
    FIXED 77u: T4 has a font map (t4_fontmap.rs, OD #777), so t4enc.def's composites are declared and `\m{y}` prints ƴ;
    2405.01972 45 → 1 Error (the `substitutefont.sty` TL 2025 lacks), the Pular letters in place
    (`fonts-nfss/t4_modified_letter_help_is_not_expanded`, `fonts-nfss/t4_african_letters_print`).
    Not loops of ours: `\clap` (1011.2267, 1308.3100), `\Ref` (math0204358) and `\robustify` (2003.07558) — the TL 2025
    definition wins, as ruled — stop pdflatex too at its 100-error limit (no PDF; ours TooManyErrors or PushbackLimit,
    the same document breakage); 1603.09649 (`\input{\Ref{…}}`, pdflatex 121 errors and a PDF) still ends in Fatal
    Timeout:Recursion: after the l3 expandable error of `\Ref` in the file name (`\???` "doesn't match its
    definition"), the l3hook file-hook code re-enters it (`\__hook_prop_gput_labeled_cleanup`, csname errors) — our 101
    errors where pdflatex has 7 on `\documentclass{article}\newcommand{\Ref}[1]{#1}\input{\Ref{foo_def}}`; not brief,
    open. (C6, from 68y, int32a too) an emulateapj deluxetable begun inside an open one: begun inside the outer's
    `\startdata` (1704.06321's shape) the inner table's first `\colhead` is lost and the alignment cascades (10 errors
    on `\begin{deluxetable}{cc}\tablehead{\colhead{OA}&\colhead{OB}}\startdata oone&otwo\\ \begin{deluxetable}{cc}
    \tablehead{\colhead{IA}&\colhead{IB}}\startdata ione&itwo\\ \enddata\end{deluxetable}\enddata\end{deluxetable}`,
    67int17b 0 and everything kept, int32a 6 and the whole inner table lost); begun before it, the inner `\tablecaption` replaces the outer's stored caption
    (`class-bindings/emulateapj_deluxetable_in_deluxetable`, RED part).
- **Runs 364/365** (E/F rerun of run 363 on 026cf18f87: 68c Part B, 68a; COMPLETE 2026-10-16 12:20:31Z):
  E 79,627 → 77,932, F 3,971 → 3,971 (success 97.14 % → 97.20 % of 2,922,325). Transitions vs run 363: E→OK 362,
  E→warning 1,330, E→F 33, F→warning 3, F→E 30. E→F: `timeout` 22 + `Timeout:Convert` 2 and `oom` 4 (fleet contention,
  retried per the 2026-10-12 ruling), `TooManyErrors` 5 — all five row **XP** (98-501 detail-less `post:xpath` Errors each:
  1809.02905, 2007.00624, 2104.14033, 2602.10769, hep-ph9803283); no conversion regression. Cluster rescan of the run-353
  Error/Fatal set (latest zip per paper, `~/data/pk_agents/main/scratch_356a0c47/attr_now/`): `undefined` 53,902 → 43,535
  papers, the structural categories flat (`unexpected` 23,505 → 23,064, `malformed` 8,564 → 8,551), the greedy
  clean-paper ranking unchanged at its top (`_` 2,035, `ltx:bibitem` 1,109, `&` 1,096, `\errmessage` 1,081, `\noalign`
  1,005); 24 papers now stop at mathtools internals (`\MT_start_cases:nnnn`, `\MH_else:`) after 67u's
  `\MHInternalSyntaxOn` (all Error before).
- **Runs 362/363** (E/F rerun of run 361 on 58956d026a: 67w, 68f, 68b; COMPLETE 2026-10-16 08:34:36Z):
  E 81,301 → 79,627, F 3,994 → 3,971 (success 97.08 % → 97.14 % of 2,922,325). Transitions vs run 361: E→OK 293,
  E→warning 1,386, E→F 35, F→OK 1, F→warning 17, F→E 40. E→F: `timeout` 22 + `Timeout:Convert` 2, `oom` 2,
  `TooManyErrors` 9. Reconverted locally on 1ca233a402, 58956d026a and 026cf18f87 (`~/data/pk_agents/main/ab_66bundle/
  results_e363{a,b}.tsv`): the 2 `oom` and 8 of the `TooManyErrors` convert with 1-14 Errors and no Fatal, identical on
  all three. Those 8 are not conversion failures: on the fleet, post's XPath evaluation of `ltx:XMath` failed 87-501 times
  "(no libxml2 error detail)" (latexml_post/src/document.rs:959) until the error cap — a libxml2-internal failure (C
  allocation under a long-lived child's address space, which the Rust allocator hook does not see), not reproducible
  standalone; queued as a retryable resource failure (row **XP** below). One is a regression: 1906.07043 (iopart + breqn)
  6 → 101 Errors + TooManyErrors since 68b, because breqn now loads amsmath as breqn.sty:46 does, and amsmath's `\cases`
  (an "Old form" error, as pdflatex gives with the paper's own iopart.cls) leaves the cell's `$x>l_0$` inside
  `{equation}`'s alignment, where the `$` does not end the display as TeX's §1197 ("Display math should end with $$")
  does; pdflatex: 9 errors and a PDF. That is row **MS**'s open item (1), now with this witness.
- **Runs 360/361** (E/F rerun of run 359 on 1ca233a402: 67y, 68d, 67x, 67z; COMPLETE 2026-10-16 04:30:35Z):
  E 86,301 → 81,301, F 4,011 → 3,994 (success 96.91 % → 97.08 % of 2,922,325). Transitions vs run 359: E→OK 894,
  E→warning 4,100, E→F 36, F→OK 4, F→warning 19, F→E 30. E→F: `timeout` 30 (two writers and the gates shared the
  host), `TooManyErrors` 3, SIGABRT 1 (math0407433, again as in run 357), SIGSEGV 1 (2112.13654), `Timeout:Convert` 1;
  every reconvertible one converts locally with no Fatal and the same output on 595ce52d9e and 1ca233a402 (fleet
  contention, not a regression; `~/data/pk_agents/main/ab_66bundle/results_ref361.tsv`).
- **Runs 358/359** (E/F rerun of run 357 on 595ce52d9e: 67q, 67r, 67p, 67s, 67u, 67ps; COMPLETE 2026-10-16
  00:15:50Z): E 89,367 → 86,301, F 4,091 → 4,011 (success 96.80 % → 96.91 % of 2,922,325). Transitions vs run 357:
  E→OK 480, E→warning 2,602, E→F 26, F→OK 1, F→warning 63, F→E 42. E→F: `timeout` 14 + `Timeout:Convert` 1, `oom` 6,
  `TooManyErrors` 4, `PushbackLimit` 1. Reconverted locally (595ce52d9e and 4a7853352a agree on every one): 9 of the
  12 non-timeout ones convert with no Fatal (fleet contention), 3 are regressions of this bundle against 1a95291ef6:
  2104.09383 (373 Errors → PushbackLimit, 0 words), 2201.11056 (6 Errors, 22,367 words → `oom:alloc_failed` from
  gullet pushback growth, 0 words), 2410.09116 (74 Errors → TooManyErrors, 16,027 → 3,634 words) — root-causing
  (`~/data/pk_agents/main/ab_66bundle/results_ref359{,b}.tsv`).
- **68f** (the three run-359 regressions, root-caused in agent_reports/2026-10-16_rc359_regressions.md): (A)
  `sect08.rs::record_dropped_environment_stores` read the name a dropped frontmatter environment's `\gdef` defines as
  the first control sequence after it, so rmmcart.cls:418's
  `\expandafter\gdef\expandafter\abstracts@ve\expandafter{\@grabstract}` recorded `\let\expandafter\@empty`, which the
  `\maketitle` deposit (sect05.rs `\lx@deposit@maketitle`) ran and its broken groups leaked: every later `\csname`
  construction failed (67o1's 373 Errors) and 67s's amsthm proof lock looped a tikz-cd into PushbackLimit (RUST-ONLY,
  since 56gm). The target is now read as TeX reads it (`dropped_store_target`: the `\expandafter` chain skipped, no name
  for a `\csname`-built one) and only a store is nulled (`nullable_store`: undefined, `\relax`, or a token-bodied macro
  — never a primitive, register, Rust-bodied kernel command or locked definition). (B) The binding dependency scan
  (`content.rs::require_dependencies_filtered`) loaded sn-jnl.cls:311's program.sty although 2201.11056's
  sn-jnl-setup.sty sets `\ver@program.sty` in `\AddToHook{class/sn-jnl/before}` (latex.ltx:18750 `\@ifl@aded` skips
  it): program's active `|` broke pgf and the gullet ran to `oom`. The scan now skips a candidate whose
  `\ver@<name>.<ext>` is set while our records say it is not loaded, with an Info naming it
  (`marked_loaded_unrecorded`; scan-only by the 2026-10-16 ruling, DIVERGENCES #605, KNOWN_PERL_ERRORS #780 — Perl
  Package.pm:2328-2330 tests only its records). (C) siunitx_sty.rs declared `S`/`s` at load only (Perl
  siunitx.sty.ltxml:1379-1399), so cellspace.sty:174-177's later `\newcolumntype{S}` won and 2410.09116's `{llSS}`
  tables gave 74 Errors (101, TooManyErrors, once 67u moved program's load); siunitx.sty:9348-9373 (and
  siunitx-v2.sty:5624-5625) re-declare them at begin-document, which the binding now does as the loaded release
  does: v3 `S` only (`table-column-type`, :9101), with its warning `Tabular column type "S" overwritten with siunitx
  definition.` also for an `S` declared before siunitx, a document's own `s` left alone; the `[=v2]` rollback `s` and
  `S`, silently (KNOWN_PERL_ERRORS #781; v2's cellspace-to-`C` move not ported). 2410.09116 stays a target
  although TL 2025 pdflatex aborts it at 100 errors (_main.tex:194: cellspace 2022 replaces `\@startpbox`, array 2025
  calls `\@startpbox@action`, array.sty:196, 388; clean with cellspace removed). Witnesses (67int4 → 68f, HTML,
  `--preload=ar5iv.sty`): 2104.09383 269 Errors + Fatal → 0 (0 → 3,932 words, recall 0 → 87.4 %; 67s1 269 + Fatal),
  2201.11056 Fatal `oom` → 5 Errors (13,817 words; 67s1 6 Errors, the extra lost words its undefined
  `\figurecaptionfont`), 2410.09116 102 Errors + Fatal → 0 (3,674 → 14,176 words; the 39 lost are the citation keys of
  missing citations, now resolved); the 67u sn-jnl witnesses 2010.12415, 2102.03176, 2104.02781 and the store-nulling
  manuals wkmgr1, wkmgr2, mcmthesis-demo, rpsample (resphilosophica) byte-identical. Repros
  sectioning-frontmatter/dropped_abstract_store_expandafter_gdef, loader/scan_skips_document_marked_loaded_package,
  package-bindings/siunitx_column_after_cellspace RED (67int4) → GREEN, package-bindings/siunitx_v3_document_s_column
  and siunitx_v2_rollback_columns RED (round 1) → GREEN; guards `perfect_kernel_batch68f` (8). A/B vs
  67int4-rel over 306 papers (100 pop67o, 100 siunitx of which 39 with a later column package, 100 with a shipped class
  renewing `{abstract}` or a `\ver@` marker, the 6 witnesses): only the three witnesses differ, 303 byte-identical;
  the sn-jnl net (30) unchanged.
- **68c** (binding gaps G3 B, author-file and symbol package bindings; bindgaps/g23_macros.tsv rows jmlr2e, spconf,
  dgruyter, icml2025/2026, biblatex, article, mathpazo, mathastext, wasysym, amsmath). The loader rule of round 1 (a
  shipped `.sty` that is another package than its TeX Live namesake loads raw: physics, proof, `myproof.sty`) is held
  on pk/68c-loader for a user ruling. Bindings, macros at their real file:line: jmlr2e borrows amsthm (a paper's
  `\let\proof\relax` + `\usepackage{amsthm}` gets amsthm's proof; 40 of 43 `{proof}` papers) and gains
  `{conjecture}`/`{axiom}`, `{example}` on its own counter (jmlr2e.sty 2022-10-17:383-396; 0907.5309, 1302.6584);
  spconf `\secondlinename`/`\thirdlinename`/`\nameplus`/`\nameplusplus`/`\secondname` append authors
  (`\lx@add@authors@append`), `\threeauthors`/`\fourauthors` (the 2006-07 copies :172-196; 1701.07847, 0708.3022);
  supertabular `\ST@newpage` (supertabular.sty:62-63, :515-543) and breakurl's character lists under `\ifpdf` false,
  after its xkeyval and ifpdf (breakurl.sty:35-38, :71-104, :186-187), for dgruyter.sty:1704-1706/:2052-2058
  (1503.04650, 1711.03341); article `\ps@headings`, `\ps@myheadings`, `\@makecaption` (article.cls:131-168, :480-489;
  KPE #760; 0711.0180, 1010.5412); mathpazo `\mathbold`, `\PazoBB` (mathpazo.sty:72, :101-103; KPE #761; 0807.0088);
  mathastext's user commands as no-ops with their arguments and their `\Mathastext…` names (:837-858, :964-968),
  `\MTDeclareVersion`/`\Mathastext[<name>]` declaring a math version and `\MTversion` selecting it with
  `\mathversion` (:2411-2420, :2585-2588, :2673-2697), `\Mathsf`/`\Mathtt` (:701-702; 1712.02869); wasysym 2.4
  `\wasyparagraph`/`\Paragraph` (§), `\wasyeuro`/`\euro` (`noeuro`), `\wasycmd`/`\applecmd`, `\longs`, `\wasyfamily`,
  `\textwasy` (wasysym.sty:47-55, :90-91, :262-270; KPE #762; 1001.4255, 1103.0853); biblatex `\RN`/`\Rn`
  (biblatex.sty:3948-3960), `\DeclareNolabel`/`\DeclareNoinit` (:13988), `\DeclareRefcontext` (:10428),
  `\BiblatexSplitbibDefernumbersWarningOff` (:16652), `.bbl` `\set`/`\inset`/`\annotation` (:8360-8378, :7943; a set
  entry prints as one more `<bibentry type="set">`, its members beside it; 2011.12274, 1707.02797, 1110.6541,
  2001.10285); icml `\Notice@String`, `\ICML@appearing`, `\ICML@preprint` — the paper's copy's where it ships one
  (`shipped_class::shipped_copy_text`; workshop lines 2606.28747, 2604.14572, 2507.02987, 2506.09099), else the
  version's as ICML distributed it (2015-2019, 2021-2026), empty for a version whose copies disagree (icml2020: PMLR
  108, 119 or empty) or an unknown one, so `accepted` never leaves the review notice nor invents a proceedings line —
  `\ificmlshowauthors`, `\ificml@noticeprinted`, the copy's (else the version's) accepted/preprint options;
  eso-pic/url/caption as the copy loads them, forloop and the affiliation counters as the copy (else 2026, resp.
  2017+) has them (icml2026.sty:124-177, :434, :504-518; 2601.22575, 2601.23143, 2601.04575); amsmath
  `\ifinalign@`/`\ifingather@`/`\ifst@rred`, `\intkern@` (amsmath.sty:1280-1290, :653; 1606.07128). Repros
  package-bindings/{jmlr2e_relaxed_proof_then_amsthm,jmlr2e_conjecture_and_axiom,spconf_second_line_names,
  spconf_threeauthors,dgruyter_patches_supertabular_and_breakurl,mathpazo_mathbold,mathastext_user_commands,
  wasysym_new_characters,biblatex_roman_and_config_commands,icml2026_notice_from_own_printaffiliations},
  class-bindings/article_page_styles_and_makecaption, ams-internals/amsmath_switches_and_intkern RED (67int6) → GREEN;
  package-bindings/{icml2024_accepted_notice_is_proceedings,icml2020_accepted_notice_unknown_is_empty,
  icml2025_customised_copy_notice,mathastext_version_selects_math_version} RED (round 1) → GREEN; guards
  `perfect_kernel_batch68c` (17). A/B vs 67int6 over 472 papers (372 row witnesses with the 43 loader-census papers, 100 of pop67o): Errors 1,332 →
  1,080, papers with Errors 362 → 188, Fatals 4 → 4, none more; per row, Errors 67int6 → 68c: jmlr2e 29 → 21, spconf
  28 → 14, dgruyter 47 → 19, article 48 → 27, mathpazo 28 → 2, mathastext 29 → 2, wasysym 27 → 2, biblatex 44 → 5 and
  19 → 2, icml2026 50 → 11, icml2025 30 → 27 (2505.21317's copy loads forloop), amsmath 89 → 84; physics (211),
  proof (21) and the census papers at 67int6's, the held rule's. Every word lost is former error text, `ſ`/`§`/`€`
  in place of the command names, or jmlr2e's examples numbered on their own counter as the copies number them
  (2311.05501, 2412.10837, 2411.09644); the 100 pop67o identical. Binding nets vs 67int6 (jmlr2e, spconf, dgruyter,
  mathpazo, mathastext, wasysym, biblatex, amsmath, article, breakurl, icml2023-2025): no document worse. RED: macro-state/counter_alias_macro_to_register
  (`\newcommand*{\c@thmlemma}{\c@theorem}`, math0701082's own theorem.sty, which the paper reaches only under the
  held loader rule). Residuals: wasysym `\textwasy`/`\wasyfamily` text is the raw letters, not the wasy glyphs;
  jmlr2e's claimed `{proof}` is amsthm's italic "Proof." ∎ where jmlr2e prints a bold "Proof" ■ (before 68c too);
  a math version mathastext declares changes no font (sect13.rs `\mathversion`), and `\MTversion{normal}` keeps a
  bold (KPE #687). Not this payload's: the jmlr2e `{assumption}` (84; four numbering forms in the copies),
  `{thm}`/`{cor}`/…, spconf `\names`/`\nametwo`/`\myname` (several arities), aas_macros journal additions (`\cqg`,
  `\prx`, …), the icml2025 long tail (`\printAffiliationsAndNoticeArxiv`, …) — each a copy's own addition, the pending
  user decision on a shipped copy's extra definitions (the ICML notice strings read from a copy are definitions the
  binding makes anyway, the copy's text of them); amsmath's public commands the witnesses lack (`\eqref`, `\dfrac`,
  `{align*}`: amsmath is not loaded there — ncc's, breqn's or the class's own dependency, not the binding); stix and
  the amsmath `\collect@body`/`\make@display@tag` internals not started.
- **Run 357** (E/F rerun of run 353 on 1a95291ef6: 67n, 67o; COMPLETE 2026-10-15 18:15:14Z in 4 h 05 min under host
  load): E 89,338 → 89,367, F 4,145 → 4,091 (success 96.80 % → 96.80 %). Transitions vs run 353: E→OK 4, E→F 43, F→E 76,
  F→OK 21. E→F: `timeout` 33 (contention: two writers and their builds shared the host), `TooManyErrors` 6, `oom` 2,
  SIGABRT 1, Document 1; all 10 non-timeout ones convert locally on the same commit with no Fatal (Errors 0-90). The
  fleet logs of the TooManyErrors papers (hep-th9703095, 1607.00237) are 68 and 101 × `Error:post:xpath XPath
  evaluation failed (no libxml2 error detail)` plus EPS "every converter failed": a memory-pressure signature in a
  fleet child. Candidate (open): treat a detail-less libxml2 XPath failure like an allocation failure (retryable,
  67o).
- **Run 353** (E/F rerun of run 351 on ba8dff38f3: 67d, 67j, 67l, 67m; COMPLETE 2026-10-15 05:52:14Z in 3 h 27 min,
  against 5 h 27 min for run 351 with the 67m deadlock): E 93,718 → 89,338, F 4,171 → 4,145 (success 96.65 % → 96.80 %).
  Transitions vs run 351: E→OK 4,360, F→OK 46, F→E 71, E→F 91 (`oom:alloc_failed` 59, `timeout:wallclock` 11,
  `TooManyErrors` 14, `crash:SIGABRT` 3, `Timeout:PushbackLimit` 2, Recursion 1, Convert 1).
- **67o** (fleet, user ruling 2026-10-15 "both"): an allocation failure in a grown harness child is retried, and children
  recycle earlier. 53 of run 353's 59 E→F `oom:alloc_failed` papers convert cleanly in a fresh process under the same
  5632 MB `RLIMIT_AS` (hep-ph0006359, math0111204, math0305067: 3.1-3.5 GB peak RSS); the allocator's reservations had
  grown the long-lived child's address space, and the recycle fired only within 1.5 GiB of the cap. Before 67m these
  children deadlocked and the harness's kill re-leased the paper; 67m made the failure clean and final. Now
  `begin_task` writes the child's address space as the paper starts (`space <KiB>`), `exit_report::retryable` tags an
  `oom:alloc_failed` retryable when that start was past a fresh worker's `FRESH_ADDRESS_SPACE_KB` (2.7 GB; an unknown
  start stays final), and `recycle_after_task` also recycles past `RECYCLE_ADDRESS_SPACE_KB` (3 GiB). Raising the cap
  instead is ruled out (worker.env: 8192 once spiked a sweep to 207 GB). Guards
  `cortex_worker_exit_reports::{an_allocation_failing_in_a_grown_worker_is_retryable,
  an_allocation_past_rlimit_as_is_reported_as_fatal_oom}` (`CORTEX_WORKER_TEST_GROW_MB` reserves address space in a
  debug child), `exit_report` unit tests. Open: measure the respawn cost of the 3 GiB recycle on the next rerun (spawn
  count and wall time vs run 353).
- **Run 351** (E/F rerun of run 349 on 4f131f2b84 with the fleet's added TeX Live collections; COMPLETE 2026-10-15
  01:41:54Z): E 100,336 → 93,718, F 4,418 → 4,171 (success 96.42 % → 96.65 % of 2,922,325). Transitions vs run 349:
  E→OK 6,658, F→OK 207, F→E 153, E→F 113 (`oom:alloc_failed` 56, `timeout:wallclock` 15, `ParamSpec:Expected` 14,
  `TooManyErrors` 18, `Timeout:PushbackLimit` 6, `crash:SIGABRT` 3). The OOM/timeout share is the run's slow tail: the
  67m deadlock parked OOM children for the harness's 360 s kill and their papers were re-leased. The 14 `ParamSpec`
  are a host-TL exposure, not a code change: eplain.tex (texlive-formats-extra, installed 2026-10-14) borrows `\cite`
  as scratch — btxmac `\edef\cite{\the\catcode`@}` then `\let\@oldatcatcode=\cite` (eplain.tex:346-348) — which our
  locked `\cite` ignores ("Ignoring redefinition of \cite"), so `\catcode`@ = \@oldatcatcode` (:666) misses its
  number and `\def\@Fornoop#1\@@#2#3` (:721) is read as `\def\@` (math0308041, math0612670, 0705.2148, 0802.3843,
  1207.1752, 1401.4197; fixed by 67n below). Per-paper cost (perf-measure 2026-10-15, CPU seconds: `perf_event_paranoid=4`
  on this host blocks `instructions:u`): no regression since the 2026-10-07 pass — 39 random 2609 papers HEAD/BASE
  median 1.000, 20 heavy fleet Fatals 0.989; 8 of the 9 fleet OOM papers peak under 3.9 GB and 8 of the 9 fleet
  timeouts finish in 5-140 s standalone, so the tail deaths are fleet limits and contention. Open from it: 1902.02884
  times out since 65n (math-group recovery at `partie_geometrique.tex:6430` `\begin{remark}`: "Missing $ inserted",
  nine "Missing \endgroup inserted", then a stall; stomach.rs `insert_dollar_sign`/`off_save_closer`, tex_math.rs
  `end_math_or_defer`); math0701165 costs +30 % CPU and +1.13 GB (4.87 GB peak) since 67d, which now converts its
  diagrams (3,152 → 3,196 formulae). Ops: the fleet's journald drops ~1M lines per 30 s (every child streams its whole
  log), and `/opt/cortex-scratch` holds 565,483 leaked `cortex_task*` input trees (434 GB, since 2026-07) awaiting a
  user-approved cleanup.
- **67n** (run 351's 14 E→F `ParamSpec:Expected`; user ruling 2026-10-15 "extend D1 to robust"): a robust locked
  binding owns its calls by what it wraps. The kernel `\cite` (sect11.rs, `robust => true, locked => true`, OD #88)
  is the `\protect \cite␣` wrapper, whose own parameter list is empty, so `state::owns_calls` said no and eplain's
  btxmac scratch `\edef\cite{\the\catcode`@}` (eplain.tex:346) was refused ("Ignoring redefinition of \cite"): its
  `\let\@oldatcatcode=\cite` (:348) copied the binding, `\catcode`@ = \@oldatcatcode` (:666) read no number, `@`
  became catcode 0 and `\def\@Fornoop#1\@@#2#3{}` (:721) raised Fatal `ParamSpec`. All six witnesses take the LaTeX
  route before `\input eplain` (AMS-TeX's `\documentstyle{amsppt}`; eplain's own `\usepackage` for 1207.1752 and
  1401.4197), which loads the locked `\cite`; Perl's `\cite` is not locked (math0308041: Perl 2 Errors, 0 Fatal).
  `owns_calls` now judges a robust wrapper (`robust_wrapper_body`) by its inner `\cs␣`'s meaning and `call_openers`
  takes the `[`/`*` openers from it, so the scratch value is the meaning for `\let` and every use without an
  argument, while `\cite{x}`/`\cite[p.~3]{x}` after eplain's own `\def\cite{\@getoptionalarg\@cite}` (:445) stay the
  binding's citations (OD #500 amended, OD #88 note). Witnesses (67m1 → 67n, Errors/Fatals): math0308041 1/1 → 0/0,
  math0612670 1/1 → 0/0, 0802.3843 1/1 → 1/0 (`frontmatter:merged_creators`), 1207.1752 1/1 → 7/0, 1401.4197 1/1 → 4/0,
  0705.2148 1/1 → 0/1 (side finding (a)). Repros macro-state/{m5_robust_cite_edef_scratch_eplain,
  m5_robust_cite_calls_after_value}, m5_robust_cite_value_in_group RED → GREEN; guards `perfect_kernel_batch67n`.
  A/B vs 67m1 (the M5 census's 95 `\def`/`\renewcommand`/`\let\cite` papers + 30 random natbib-free papers citing
  with `\cite[…]{…}`): 125/125 byte-identical XML (606 Errors, 3 Fatal timeouts, 8,727 `<cite>` both sides); 4 take
  the new path, parameterless forwarders whose every use is a call (`\renewcommand\cite{\citep}` 2306.00600,
  `{\citeasnoun}` 2006.07780/2006.13934, a cite.sty-style `\def\cite{…\@ifnextchar[…}` 1806.11139).
  Side findings, not chased: (a) 0705.2148 now ends in a Rust-only `Fatal:Timeout:PushbackLimit` while reading
  amssym.tex, which needs eplain, amstex, amssym and `\documentstyle` together (Perl 1 Error at eplain.tex:50, pdftex
  clean): RED amstex/eplain_amstex_amssym_pushback_limit. (b) `has_latex_cue` (latexml_oxide/src/converter.rs:136-146)
  counts `\usepackage`/`\RequirePackage`/`\include` as LaTeX cues, while the test `state::set_plain_tex_document`
  documents (state.rs, arXiv AutoTeX's) names only `\documentclass`, `\documentstyle` and `\begin{document}`; eplain
  provides `\usepackage` in plain TeX, so the plain eplain papers 1207.1752 and 1401.4197 take the LaTeX route and keep
  their `\usepkg@options@…` Errors (routing unchanged here; needs a decision).
- **Runs 354/355** (full sandbox reruns of 2605/2606 on 1a95291ef6 = 67n + 67o, the baseline before the rawclasses
  lever). 2605 (run 354) vs its 2026-10-13 snapshot: E 1,443 → 1,332, F 32 → 29; 10 papers worse: 5 fleet-contention
  `timeout:wallclock` Fatals (2605.04030, 2605.15781, 2605.20426, 2605.28027, 2605.14200: 40-72 s locally against the
  fleet's 180 s, with local measurement runs sharing the unpinned host), and two code classes, fixed as 67q. 2606
  (run 355) vs 2026-10-13: E 1,402 → 1,298, F 42 → 27; 2 worse: 2606.12944 (`\path`, 67q) and 2606.01369 (a
  wall-clock timeout).
- **67q** (run 354 regressions): (1) the arXiv hyperref fallback (67a, OXIDIZED_DESIGN #588) supplied `\href` but not
  url.sty, which the injected hyperref loads; an elsarticle-num `.bbl` defines `\path` only when `\href` is undefined, so
  its `{\path{doi:…}}` entries were undefined (2605.07373, 2605.08166, 2605.12003). The fallback now also loads url.sty
  provide-only (`\path`, `\urlstyle`, `\urldef`, `\Url` where still undefined). Guard
  `perfect_kernel_batch67a::url_path_kept_for_a_bbl_after_supplied_href`. (2) mathpazo.sty:63/67 declares the `upright`
  symbol font, so `\symupright` exists; flexisym's mathpazo.sym:49 reads it under `\usepackage[mathpazo]{flexisym}`
  (breqn), read since 66i. Our binding, like Perl's, declared none (2605.15826, 2605.30950). Guard
  `perfect_kernel_batch67q::mathpazo_declares_the_upright_symbol_font`. Residual (not changed): our kernel's
  `\DeclareSymbolFont` makes `\sym<name>` a `\let` to `\z@` (sect08.rs), where latex.ltx's `\new@mathgroup` allocates
  a `\chardef` family number (`\the\symupright`: 0.0pt vs pdflatex's 4). (3) From the rawclasses regression study
  (agent_reports/2026-10-15_rc_rawcls_regressions.md, R3): hyperref.sty:7302 keeps the original `\maketitle` as
  `\HyOrg@maketitle`; pittetd.cls's `\AtBeginDocument{\let\maketitle\HyOrg@maketitle}` left `\maketitle` undefined
  (1509.06375: 101 errors then Fatal). Our binding (and Perl's) saved none; now `\let`. Guard
  `perfect_kernel_batch67q::hyperref_keeps_the_original_maketitle`.
- **67r** (rawclasses study R1): in maths an old font command switches the math alphabet to the end of the group, as
  latex.ltx's `\@fontswitch` (14312-14321) does by letting `\math@bgroup` to `\relax`: `{\rm LP-HF}` is all upright,
  `{\cal H}` a calligraphic H. Ours and Perl's (latex_constructs.pool.ltxml:5200, :5315; KNOWN_PERL_ERRORS #685) ran the
  alphabet with its own argument (`\mathrm{L}P-HF`, `\mathcal{}H`, "Missing sub/superscript argument", "Extra
  \right"). `\lx@math@alphabet@switch` merges the font the alphabet's binding carries (`Definition::declared_font`,
  kept on primitives and constructors); another `#2` is TeX's `#2\relax` (OXIDIZED_DESIGN_DIVERGENCES #593). Under the
  rawclasses lever the `tex=` with an empty alphabet argument of 10 ptptex/jpsj2/paper witnesses go 3,214 → 0 (gr-qc0605108
  919 → 0, cond-mat0510381 296 → 0), errors and words unchanged. A/B without the lever (67o1; 401 papers, 392 converted:
  sources calling `\DeclareOldFontCommand` or amsart/revtex/aastex with `{\rm …}` in maths, and 100 of pop67o): 0 gain
  Errors or Fatals, cond-mat0205657 loses its one Error, 33 change `tex=` (`\mathrm{o}ut` → `{\rm out}`), and the 24
  word drops are maths tokens merging (`g`·`a`·`s` → `gas`; every maths character and text word kept). Guards
  `perfect_kernel_batch67r::{old_font_command_switches_the_math_alphabet,
  old_font_command_switches_package_and_declared_alphabets}`. Round 2 (review): an alphabet reached through a
  parameterless macro or `\protect` is a switch too (`\newcommand\Bbbx{\mathbb}` as `#3`: round 1 gave
  `tex="{\mathbb{}RZ}"` with both letters italic; guard `old_font_command_alphabet_through_a_macro`); the `{\cal AB}`
  `tex=` round-trip is a recorded residual in OD #593; round-2 A/B on the same 401 papers (392 converted): every
  output byte-equal to round 1's, so vs 67o1 still 0 gain Errors or Fatals; the 10 rawclasses witnesses byte-equal
  too. Side finding, FIXED 78a: `\unboldmath` and
  `\mathversion{normal}` did not undo `\boldmath` (both engines; `Font::merge` of `forcebold: false` was a no-op while the
  bold math font forces its series), KNOWN_PERL_ERRORS #687, repro fonts-nfss/boldmath_undone_by_unboldmath_and_group.
- **67p** (raw TeX Live classes under the coming rawclasses lever, study 2026-10-15_rawclasses_lever.md §2): two
  binding gaps. (1) xkeyval: the `OptionalAngle` reader returned the same empty value for `<>` and for no `<…>`, so
  `\ProcessOptionsX[sap]<>` (sapthesis.cls) and `\DeclareOptionX[sap]<>` took the default family
  `\@currname.\@currext` and the class's `\DeclareOptionX*` rejected `english`/`oneside`/`binding=…`; an absent
  `<…>` is now `None` and `<>` the empty family (header `<prefix>@`, xkeyval.tex:83-87, as Perl's front end :330
  keeps it), and `\key@ifundefined` searches the empty family too (xkvutils.tex:110-126, :228-237). Perl's
  `KeyVals` drops the empty family (KPE #212, extended). (2) theorem.sty (theorem_sty.rs): `\theorem@bodyfont`,
  `\theorem@style`, `\theorem@headerfont` defined as theorem.sty:62-81 does; `\theoremstyle` (grouped),
  `\theorembodyfont`, `\theoremheaderfont` (once only) are theorem.sty's; the binding's `\newtheorem` hands
  `define_new_theorem` the theorem.sty sequence `\th@<style> \the\theorem@bodyfont \@thm{<counter>}{<name>}` as the
  theorem's body font (a binding style's font in place of its `\th@`, unless a class redefined it; the current style
  font before a class style that sets its font in `\@begintheorem`), and `\@thm` ends it. theorem.sty does not load
  amsthm, so the binding leaves the amsthm it borrows unloaded (unless the paper had loaded it) and a paper's later
  `\usepackage{amsthm}` takes `\newtheorem` (amsthm.sty:77; amsthm_sty.rs `\lx@amsthm@newtheorem`) and the styles
  back, as the ifacconf template needs (KPE #686). Round 2 (review): the no-op that ends the sequence is a
  `\let\@thm\@gobbletwo` local to the theorem's group (round 1's global no-op `\@thm` silently dropped the heads and
  counter steps of raw `\th@plain\@thm{claim}{Claim}` environments; guard `theorem_raw_thm_keeps_heads`); the borrow
  is the kernel's `require_package_borrowed` on `state::mark_unloaded` (factored out of `end_provide_only`), used by
  ntheorem's binding too (ntheorem.sty loads only ifthen); the definition-time style limits are in KPE #686. Round-2
  A/B vs 67o1-rel, the 200 above + 30 ntheorem users (from a 29.7k-zip scan; 8 also load amsthm or theorem.sty), 218
  converted: Errors 1,324 → 1,322, no paper worse, the same six changes, and byte-equal tallies with round 1 on all
  190 shared papers; the 30 ntheorem papers unchanged. Open (a ruling): a borrowed binding's definitions block the
  ones LaTeX would take, as the real package never defines them — a document's `proof` in the
  `\@ifpackageloaded{amsthm}` false branch, and under the rawclasses lever cc.cls's ccproof.sty:56
  `\newenvironment{proof}` ("Ignoring redefinition"), so 1507.02955's proofs have no □ (base the same; round 1 had
  them only because its global no-op `\@thm` let ccqed.sty:74's wrapper leak `\@qedtrue`). Perl golden tests/theorem/theorem.xml: the
  last theorem is italic, as pdflatex sets it (CMTI10). Witnesses, rawclasses lever (`--path=…/rawcls/rhai
  --preload=lxrawclasses.sty`), Errors 67o1 → 67p: 2503.03277 2 → 0, 2403.20315 1 → 0, 1507.02955 4 → 1 (the
  `\currentaddress` left; creators 3 both, theorems 38 now slanted under small-caps heads as thcc.sty sets them);
  default profile unchanged for the three. A/B vs 67o1-rel, 190 papers (150 pop67o xkeyval/theorem.sty users + 50
  grepped; 10 no main): Errors 1,316 → 1,314, no paper worse; 6 change, each checked against pdflatex: 2211.14033 and
  2512.06733 (ifacconf + amsthm: `\theoremstyle` undefined / `{proof}` undefined gone, amsthm heads), 1911.07501 and
  1911.10645 (ifacconf: italic heads, upright bodies, SFTI/SFRM in the PDF), 1911.07591 (fundam's `\th@plain` no
  longer sets the title and abstract italic), math0505657 (`\theorembodyfont{\slshape}` survives
  `\theoremstyle{plain}`, SFSL); 2603.11260/2603.11265 take amsthm's heads (with the period) and its ∎. Repros
  package-bindings/{xkeyval_empty_family_processoptionsx (+ xkvemptyfam.cls), theorem_bodyfont_internals,
  theorem_user_style_and_headerfont, theorem_then_amsthm_takes_over} RED → GREEN; guards `perfect_kernel_batch67p`.
  RED side finding, not fixed (kernel, Perl shares): a `\newtheorem`'s shared counter named by a macro
  (ccthm.sty `[\cc@thmcnter]`; latex.ltx `\@othm` expands it, `define_new_theorem` takes `ToString`): 1507.02955's
  38 theorems unnumbered under the lever, repro package-bindings/newtheorem_shared_counter_named_by_a_macro.
- **67s** (rawclasses regression study R2 and R5, agent_reports/2026-10-15_rc_rawcls_regressions.md): theorem and
  proof machinery under raw classes. (1) smfart.cls:1194, smfbook.cls:1281 and ijmart.cls:336 `\renewenvironment{proof}`
  over a `\trivlist` replaced amsthm's `<proof>` (86 papers under the lever: smfart 72, ijmart 11, smfbook 3); amsthm_sty.rs
  now locks `\proof`/`\endproof` against the class and package files, the document's own `{proof}` stays its own
  (OXIDIZED_DESIGN #596, KNOWN_PERL_ERRORS #690). (2) `\newtheorem{theo}[\smf@thm]{…}` (smfthm.sty:52-56, ccthm.sty
  `[\cc@thmcnter]`): `define_new_theorem` took the counter's macro name literally, so every theorem raised
  `\the\smf@thm` undefined; it now expands the name as latex.ltx's `\@othm` does through `\csname` (KNOWN_PERL_ERRORS
  #691; 21 papers in both arms). Witnesses 1711.11547, 1902.02049, 2108.12000 (smfart), 1507.02955 (cc). Repros
  package-bindings/amsthm_proof_kept_against_a_class_renewenvironment, newtheorem_shared_counter_named_by_a_macro (67p
  RED → GREEN); guards `perfect_kernel_batch67s`. apxproof, which wraps the binding's proof to move a `theoremrep`'s
  proof to the appendix, had its binding yield the lock while it loads (2502.18805; since 67ps the lock lets its
  wrappers through). Witnesses under the lever (67o1 →
  67s, Errors / `<proof>`): 1711.11547 1/0 → 0/14, 1902.02049 1/0 → 0/30, 2108.12000 0/0 → 0/24, 2412.15231 (smfbook)
  1/0 → 0/256, 2306.11126 (smfbook) 0/0 → 0/44, ijmart 1001.1175 0 → 19, 1107.1667 0 → 8, 2207.06393 0 → 9 proofs;
  1507.02955 (cc) keeps its 4 theorem.sty Errors (67p) and loses 76 counter warnings. Word losses are generated text
  only: the `\the\smf@thm` error strings, the trivlist items' "item" autoref tags, the class's " — " after the proof
  head. A/B without the lever vs 67o1 (400 papers: 300 amsthm `{proof}`/`\newtheorem[..]` users, many shipping a
  `.sty`/`.cls` that redefines `proof`, + 100 from pop67o; plus 17 apxproof users): 0 worse; 9 gain proofs (1202.3460,
  1210.2844, 1609.05867, 2309.14926, math0003189, math0007097, math0203159, math0302254, math0412216: a shipped class's
  `\renewenvironment{proof}`; 2503.05183, first counted here, keeps GT.cls's trivlist: its copy is defined in
  `\AtBeginDocument`, OXIDIZED_DESIGN #596), 2309.14926 (shipped smfart) 1 → 0 Errors; the apxproof users are
  byte-identical. Residual: the proof head's font and punctuation are the binding's, not the class's (isamms.cls's
  small caps, smfart's " — ").
- **67u** (binding gaps G1, run 353's undefined-only papers whose missing commands come from a package the class, or a
  package it loads, requires; bindgaps/g1_deps.tsv): bindings load what the real file loads, in its order and under its
  options. Class bindings (dependency, real file:line, witnesses): sn-jnl → program (sn-jnl.cls:311/621 of the older
  copies, through the class scan; 2010.12415, 2102.03176, 2104.02781; its `\(`…`\)` programbox kept the kernel's inline
  maths under the class, OXIDIZED_DESIGN #597, approved by the main session 2026-10-15; the proof mark is the class's
  begin-document `\opensquare` □, sn-jnl.cls:1571-1575 of 1907.11468's copy, over program's `\blacksquare` — round 2,
  1907.11468, 2210.16458; the same hook's `\let\proof\spiproof`, an amsthm copy over program.sty:928's `\def\proof`, puts
  the binding's `<ltx:proof>` back — integration with 67ps, whose lock lets program's own-head `\proof` through); bmvc2k → subfigure/subcaption/subfig/caption as the
  paper's copy names them (bmvc2k.cls:121; 1507.05726, 1605.07146; shipped_class.rs `SUBFIGURE_PACKAGES`,
  `require_shipped_packages`); ieeeaccess → color, spotcolor unless `jtehm` (ieeeaccess.cls:121-131; 1901.05553,
  1712.00216); agujournal2019 → xcolor, url, lineno, indentfirst, ragged2e (:91-112; 1803.00457, 1902.01137);
  WileyMSP-template → graphicx, gensymb (:5-10; 2003.01904); oup-authoring-template → multirow, url, mathrsfs
  (TL 2025's copy :2426-2430, 2102.02669's :2391-2395; 2102.02669); cas-sc/cas-dc → cas-common's `\MSC`/`\JEL`/`\PACS`/`\orcidauthor`/`\stmAuthorSetup`/
  `\FullWidth` (cas-common.sty:415-423, 774, 1599-1610, 2067; `\MSC` 1909.01564, 2407.03849; `\orcidauthor`
  2104.12510, 2304.13458, 2407.18715, each after its own `\author` — 2106.08963 swaps the two arguments, the author's
  error, so its link reads orcid.org/Andrew S. Nencka); ws-rv9x6 → makeidx, rotating, url
  from the copy (ws-rv9x6.cls:1524-1528; 0706.1148); revtex4-1/4-2 → lineno under the `linenumbers` option, at the
  document's start (revtex4-1.cls:6186-6193, revtex4-2.cls:6207-6214; 1209.2541, 2003.07846; KNOWN_PERL_ERRORS #697);
  quantumarticle → the `amsfonts`/`amssymb`/`amsmath` options' packages, geometry, caption (quantumarticle.cls:155-168,
  282-284, 1508-1512 — caption at the class's `\AtEndPreamble`, with the class's etoolbox (:27), round 2;
  1802.10344, 1612.06828, 1910.03969; #698); aa → answers (verbatim's `{comment}`) when the copy
  loads it (aa.cls:249 of the v8 copies; 1501.03426; #699); atlasdoc → its font option's packages, newtxtext+newtxmath by
  default (atlasdoc.cls:52-54, 516-546; 2207.12246); ceurart → colortbl, dcolumn, stfloats (:88; 2212.08130); ejpecp →
  graphicx, mathtools, microtype, latexsym, dsfont unless `NODS`, geometry (ejpecp.cls:81-106; 1607.07914, 1111.6458);
  WileyNJD-v2 `APA` → NJDapacite, which is apacite.sty, as the apacite binding with `natbibapa` (WileyNJD-v2.cls:63,
  2722-2723; 1811.00964); jmlr → graphicx, url, xcolor[x11names] (jmlr.cls:150-154; 1902.03035); sigma → latexsym, amscd,
  epsfig, graphics, ifthen, color (sigma.cls:20-28; 0902.2977). Package chains the grep attributed to the class:
  amsart/article/IEEEtran → graphics through lscape.sty:35 and psfrag.sty:35 (129 papers; 0710.2354, 0801.1195; #692),
  → color through showkeys `[color]` (showkeys.sty:51-56; 73; 1010.3777; #692), → thm-patch through thmtools
  (`\addtotheorem…hook`, run by `define_new_theorem` inside the theorem and, as thm-patch.sty:178-217 wraps amsthm's
  `proof` when it exists, the `[proof]` hooks by the binding's proof; thmtools marks `thm-patch.sty_loaded`, so a later
  `\usepackage{thm-patch}` reads nothing over it — round 2; 69; 1710.06939; #695), → `\varlim@`
  (amsopn.sty:92-97, mis-attributed to amstex; 23; 1608.08273; #696); article → mhsetup through mathtools
  (mathtools.sty:37, the binding's TODO; 33; 1202.0045; #693); acmart → centernot through newtxmath (newtxmath.sty:36;
  21; 2002.09451; #694, a centernot binding: `\not`). The xcolor exclusion comments in agujournal2019_cls.rs and
  jmlr_cls.rs were stale (a repeat `[table]` load applies its options since 2605.00310) and are replaced. With program
  loaded two interactions surfaced and are fixed: adjustbox's trimclip, finding no driver for our empty `\Gin@driver`,
  reads pgf at once (pdflatex: tc-pdftex.def, no pgf), and pgf broke on program's active `|` (2202.10115, 2507.09618 +6
  Errors) — adjustbox_sty.rs read its file with `|` other (removed in 68h, which names the driver); program.sty:1009's `\index` (`\@sanitize` with nothing
  to end the group here) turned the rest of 2306.09847 into text and lost its bibliography — program_sty.rs keeps the
  kernel's `\index`/`\makeindex`. `shipped_class_text` now also reads a `{filecontents}` copy (the VFS), as the
  dependency scan does. Skipped, author's missing `\usepackage` (pdflatex errs too): mn2e → natbib (mn2e.cls:1201-1203
  loads it only under `usenatbib`, which none of the 128 witnesses passes (many have it commented out); `\citeauthoryear`
  then errs at each `\cite` in pdflatex: 0706.0055), acmart → colortbl (no witness of 47 loads colortbl or passes
  `table`: 1807.07984), article → forloop/multirow/color with no chain, the 24 revtex papers calling lineno without the
  option. Discounted (TL-2025 version limit): mnras → atbegshi (63; 1605.02911, 1609.06406: the `\Hy@FixNotFirstPage`
  hack relies on pre-2020 hyperref loading atbegshi; TL 2025 hyperref does not, pdflatex with the paper's mnras.cls
  leaves `\AtBeginShipout` undefined). Repros class-bindings/{snjnl_class_copy_loads_program,
  bmvc2k_class_copy_loads_subfigure, ieeeaccess_loads_color_and_spotcolor, agujournal2019_loads_lineno_ragged2e_xcolor,
  wileymsp_loads_gensymb, oup_loads_multirow, cas_sc_classification_lines, ws_rv9x6_class_copy_loads_makeidx,
  revtex_linenumbers_option_loads_lineno, quantumarticle_options_load_ams_geometry_caption, aa_class_copy_loads_answers,
  atlasdoc_default_fonts_load_newtxmath, ceurart_loads_colortbl, ejpecp_loads_mathtools_dsfont,
  wileynjd_apa_option_loads_apacite, jmlr_loads_xcolor, sigma_loads_amscd} and stub-dependencies/{lscape_psfrag_load_graphics,
  showkeys_color_option_loads_color, mathtools_loads_mhsetup, newtxmath_loads_centernot, thmtools_theorem_hooks_run,
  amsopn_varlim_builder} RED → GREEN; guards `perfect_kernel_batch67u`. A/B: final candidate vs the parent (67s1-rel; 67o1-rel where the r1 candidate matched it), 606 papers (up to 12
  witnesses per row, all 138 sn-jnl → program, 100 from pop67o): Errors 1,278 → 432, Fatal 3 → 2, error-free 65 → 496;
  pop67o 343 → 242. Word losses, over the 539 papers both arms converted: 366 lose some words, 82 more than 15, 20 at
  least 1 % of their words; each is error-recovery text or text pdflatex hides: the unloaded commands' names and
  arguments (`\definecolor cvprbluergb…`, `\cellcolor color1!33`, `\scalebox .4 \includegraphics krebes.eps`,
  WileyNJD's raw `\APACyear`/`\BBA`, the APA title printed twice; bibitems unchanged, every reference text kept) and
  aa's `{comment}` bodies — three aa papers lose a `<tabular>` that sits inside `\begin{comment}` (1911.00393,
  1708.03873, 1612.05281) — plus the sn-jnl proof mark (∎ in 257 proofs of the 138 sn-jnl papers; □ since round 2,
  as pdflatex prints). The review diffed 13 more (WileyNJD 2009.09092, 2106.13685, 2101.02028, 2012.15339; aa
  1708.03873, 1704.08487, 1505.02291, 1509.09223, 1911.00393, 2605.31060; ieeeaccess 2304.03771, 2205.04113; amsart
  1406.4669) and scanned every paper losing more than 15: no author text lost. 1709.10492 timed out once under load (170 s) and converts in 112 s alone. Binding nets
  (binding_nets run_net.sh, label 67u) for the 23 bindings touched vs 67s1: Errors fall in 19 (ieeeaccess 40 → 0,
  mathtools 55 → 13, ejpecp 53 → 26, atlasdoc 19 → 3), none worse after the adjustbox/program fixes (the sn-jnl net's
  2010.12415 2 → 7 at the first candidate, 0 now). Round-2 repros class-bindings/{snjnl_class_open_square_qed,
  quantumarticle_caption_at_end_preamble}, stub-dependencies/thmtools_then_thm_patch_proof_hooks (RED on 67u r1).
- **Residual of 67u — our empty `\Gin@driver` sends trimclip to its PGF fall-back** (SHARED, KNOWN_PERL_ERRORS #797;
  FIXED in 68h: graphics_sty.rs gives the driver file graphics.cfg would, and the `|` workaround is gone): trimclip.sty:24 `\def\tc@driver{tc-\Gin@driver}` names no file here, so :252-258 input tc-pgf.def and pgf at
  adjustbox's load, where pdflatex reads tc-pdftex.def and no pgf. 67u's adjustbox_sty.rs reads the file with `|` other
  (program.sty's active `|` broke pgf: 2202.10115, 2507.09618), a workaround; the root is the driver name graphics
  leaves empty.
- **67ps** (the integrated 67p + 67s review, agent_reports/2026-10-15_67ps_integrated_review.md, NO-SHIP finding 1 and
  the borrowed-definitions rule): (1) the borrow (`require_package_borrowed`, theorem.sty's and ntheorem.sty's bindings)
  sets no proof lock — 67s had set it during the borrow, so a theorem.sty-based class's `\def\proof` was refused — and
  what it defines carries `DefinitionOrigin::Borrowed`, which yields to any later definition, the document's or any
  file's (`is_definable_latex`; `fallback_meanings` now records the origin), as in LaTeX, where amsthm is not loaded;
  the files it loaded (amsgen) are unloaded with it, `mark_unloaded` also clears `_raw_loaded` and `\opt@<file>`, and
  amsthm's real load after a borrow keeps a proof put in place meanwhile and only provides its old-version font commands
  (KNOWN_PERL_ERRORS #686). (2) Under a real amsthm the proof lock holds only against a copy of amsthm's interface (an
  optional argument with a non-empty default first, `state::files_lock_holds`/`takes_optional_first`), titles the kept
  proof from that default (`record_redefined_default`: thuthesis's `\thu@proof@name`), lets a redefinition with its own
  head through (head text and end mark kept), and lapses once the protected `\@proof` is no longer in force (amsart, the
  document's `\let\proof\undefined`, then a package's copy `[1][Nachweis]`); 0705.0545 (the same `\let`, then nccthm's
  `\newcommand*{\proof}[1][]`: 1 Error → 0) goes through on the copy rule, its default being empty; apxproof needs no
  lift of the lock any more (OXIDIZED_DESIGN #596). Round 4 (re-review agent_reports/2026-10-15_67ps_r3_review.md): a
  robust copy (`\DeclareRobustCommand\proof[1][\proofname]`) is judged by its inner `\proof␣`; a borrowing binding
  claims the borrowed names its real package defines too (`state::claim_borrowed_meanings`: ntheorem's `[amsthm]`
  proof, which a later package's `\newenvironment{proof}` had replaced; theorem.sty's skips; KNOWN_PERL_ERRORS #686);
  `install_definition` probes the copies lock only where a lock against the files is set; and the test harness fails
  a guard whose fixture a binding shadows through the versioned-name fallback without reading it
  (`util::test::assert_fixtures_not_shadowed`; `perfect_kernel_batch67s::package_redefined_proof_keeps_the_binding`'s
  `myproof.sty` had loaded the proof.sty binding, so it is `relaxproofpkg.sty` now). Repros package-bindings/{theorem_borrowed_proof_yields_to_a_class_def,
  theorem_borrowed_proof_yields_to_a_package_newenvironment, amsthm_proof_lock_lets_a_hardcoded_head_through,
  amsthm_proof_copy_titled_by_its_default, amsthm_proof_lock_lapses_after_the_document_undefines_it,
  amsthm_proof_lock_passes_an_empty_default, ntheorem_amsthm_option_keeps_its_proof,
  amsthm_proof_lock_holds_against_a_robust_copy, theorem_borrow_leaves_amsgen_unloaded,
  amsthm_reload_keeps_theorem_font_commands} RED → GREEN; guards `perfect_kernel_batch67ps` (16), with 67s/67p guards tightened. A/B vs 67o1 (613 papers: the 67s and 67p sets +
  0705.0545): Errors 1838 → 1813, Fatals 5 → 5, 0 papers gain an Error or Fatal; vs the parent a03442572a 12 differ: 9
  papers where a theorem.sty/ntheorem document or class defines its own proof now print it as pdflatex does, without
  `<proof>` (2005.04068 22 Errors → 0, 2201.05175, math0505657 "Preuve.", 1911.07591 fundam.cls:169, math-ph0505075,
  math0505275, 0807.2196, 1802.03066, 2505.07298), 2 take the document's own QED mark, 1 nondeterministic (2210.15126).
  Under the raw-class lever 1507.02955's 19 proofs are ccproof.sty's theorems with their □, headed "Proof 1" where
  pdflatex prints "Proof." (the class style's `\@begintheorem`, KNOWN_PERL_ERRORS #686); the smfart/ijmart/smfbook
  witnesses are unchanged. Binding nets amsthm, tabls, amsart, amsbook, llncs, svjour3, elsarticle, siamart: 0 worse
  than 67o1 or 67s1.
- **68d** (loader fixes from the 2605+2606 `missing_file` study, agent_reports/2026-10-15_missing_file_study.md): (1)
  a path-prefixed class whose basename names a binding (`\documentclass{lib-acm/acmart}`, shipped copy in lib-acm/) is
  that binding, loaded as the request's own file, as Perl's FindFile finds `<class>.cls.ltxml` in its installation
  directory without the request's directory (Package.pm:2125-2128, Util/Pathname.pm:340-341): no `missing_file`
  warning, no "missing file" in the verdict (cortex no longer files it as a warning), no dependency scan of the
  shipped copy, which had loaded babel (no language: “?” quote glyphs), amsart, xstring, totpages, environ, framed,
  zref and draftwatermark over the acmart binding (`content.rs::class_binding_for_path`; `fallback_request` keeps the
  path for the family bindings that read the copy, JINST); 43 tasks (2605.00109, 2605.25901, 2606.20651). (2)
  `\usepackage`, `\RequirePackage`, `\LoadClass(WithOptions)` and `\documentclass` strip a lone brace group from the
  list and from each name as latex.ltx's delimited arguments do (a package name up to four groups after its argument's
  braces: the list at `\@fileswith@pti@ns` latex.ltx:18714 and `\zap@space` :18695, each item at `\reserved@b`
  :18727-18734 and `\@onefilewithoptions` :18740; a class name two; tex.web §400), and the PI spells the argument as
  Perl's, without its braces and with its whitespace (2605.12897 `{makecell}`, 2606.09700 `{multirow}`). (3)
  `\IfFileExists`/`\InputIfFileExists` take a binding for its file except a stand-in for an author's file TeX Live
  never distributed (`latexml_core::binding::author_files::AUTHOR_FILE_STAND_INS`, 224 names):
  `\IfFileExists{arxiv.sty}` is false unless the paper ships it (2606.27281; Perl shares it for its own stand-ins,
  KNOWN_PERL_ERRORS #770). A file TeX Live once distributed answers as one (ruling 2026-10-15: `FORMER_TEX_LIVE`, 21
  names with the years of TL 2007-2024's file lists — html.sty, psfig, rsfs.sty, luximono.sty, scrpage/scrpage2.sty,
  slashbox.sty, undertilde.sty, cropmark.sty, aastex.cls/.sty and aastex62/63/631.cls, elsart*.cls, phyzzx), as does
  LaTeXML's own (`LATEXML_OWN`). Guards: every entry is a registered binding kpathsea does not find, on one list only;
  on a full TeX Live tree, every registered `.cls`/`.sty`/`.tex` binding kpathsea does not find is on a list (which
  added jmlr2e, aistats2026, colt2024-2026, uai2025, l4dc2026, neus2025, pasj00-02, asme2ej, bmvc2k, SCIS2024,
  agutexSI2019, agujournal2019, ws-p8-50x6-00 and colm2025_conference as stand-ins). (4) jfm's `lineno` option
  (lineno-FLM.sty at the document's start) is declared only when the paper's copy declares it (2605.03843's v1.25 has
  none). (5) breqn's stub notice says "only minimally stubbed". Recorded, not changed: `\RequirePackage{setup}` after
  `\usepackage{style/setup}` (KNOWN_PERL_ERRORS #771). Repros loader/{subdir_class_is_its_binding (+
  lib-acm/acmart.cls), braced_package_name, iffileexists_author_file_stand_in, iffileexists_former_tex_live,
  jfm_v125/jfm_old_copy_has_no_lineno_option} RED (67int1) → GREEN; guards `perfect_kernel_batch68d`, the batch 61
  alternate guard now expects no warning for `misc/ieeetran`. A/B vs 67int2-rel, 147 papers (the 43 subdirectory-class
  tasks, the 4 other witnesses, 100 from pop67o): Errors 506 → 506, Fatals 6 → 6, Warnings 1,509 → 1,460, no paper
  worse; XML changes (15): the acmart papers lose the scan's babel (proper quotes, `~` a no-break space again) and the
  PIs of the witnesses; 1710.04900 (harvard.sty's `\IfFileExists{html.sty}`, no html.sty in the source) is unchanged,
  keeping the html binding and its hyperref, as TeX of its era found html.sty.
- **67x** (binding gaps G2, the IEEEtran family; bindgaps/g23_macros.tsv rows IEEEtran local 354 / tl 138, IEEEtran4PSCC
  185, ieeeconf 67; KNOWN_PERL_ERRORS #710-#712, Perl's IEEEtran.cls.ltxml has the same gaps). IEEEtran binding
  (ieeetran_cls.rs), as IEEEtran.cls V1.8b (TL 2025) writes them: the compsoc executors `\@IEEEcompsoconly`,
  `\@IEEEnotcompsoconly`, `\@IEEEcompsocconfonly`, `\@IEEEcompsocnotconfonly` (:456-462), `\@IEEEnormalsizeunitybaselineskip`
  and `\CLASSINFOnormalsizeunitybaselineskip` per size option (:606-697, :976), `\sublargesize` (:639-708), `\@IEEEtunefonts`/
  `\@IEEEsetfontdimens` (:1074, interword tuning: empty), the IED parameters `\IEEEiednormlabelsep`, `\IEEEiedmathlabelsep`,
  `\IEEEiedtopsep`, `\IEEEiedlistdecl`, `\ifIEEEnocalcleftmargin`, `\ifIEEEnolabelindentfactor` (:2085-2153), `{LaTeXitemize}`/
  `{LaTeXenumerate}` (:2219-2222), `\thesectiondis` … `\theparagraphdis` (:2571-2587; `\appendices` sets `\thesectiondis` as
  :5765-5774 does; `\appendix` clears it and numbers the subsections `\Alph`, :5734-5736), `\@IEEEsaveoriginalsectiononce` run by `\appendix`/`\appendices` (the kernel's otherwise) with
  `\@IEEEprocessthesectionargument`/`\@IEEEdestroythesectionargument` (:5695-5713; a document's `\suppl` copy, 1907.08915), the page styles `\ps@headings`, `\ps@IEEEtitlepagestyle`, `\ps@IEEEpeerreviewcoverpagestyle` with
  `\@IEEEheaderstyle`/`\@IEEEfooterstyle`, `\@IEEEpubid`, `\if@IEEEusingpubid`, `\@IEEEpubidpullup` (:4228-4347, :6127-6150; page
  furniture, set nowhere), `\IEEEnoauxwrite` (:4448), `\IEEEbibitemsep` (:4484), the author-block styles and
  `\@IEEEauthorblockconfadjspace` (:4552-4561), `\IEEEtitletopspace(extra)` (:4748), `\@IEEENORMtitlevspace`/
  `\@IEEEMINtitlevspace` (:4969-4978), `\IEEEquantizevspace` (:4983-5203: arguments read as written, the object set once,
  the measuring and the space not modelled), `\IEEEquantizedisabletitlecmds` (:5208), `\@IEEEaftertitletext` and
  `\@IEEEspecialpapernotice` (:5219, :6101; empty — `\IEEEaftertitletext`/`\IEEEspecialpapernotice` stay frontmatter notes),
  `\@ifmtarg` (:5668). Older versions and variants, as fallbacks a document's definition replaces: V1.5-1.6c
  `\ifcenterfigcaptions` (V1.6b :1966-1972), `\ifnocalcleftmargin` (:1512), `\@specialpapernotice` (:4379),
  `\ifuseRomanappendices` (:4075, false: V1.8b's `\Alph` default; `\appendices` reads it beside `romanappendices`); V1.7a's
  begin-document `\proof` (:4690) for a document that left `{proof}` undefined or `\relax` (V1.8b's error case), let as
  a fallback by a primitive the hook calls, so a document's own begin-document `{proof}` replaces it; IEEEtran4PSCC.cls's `\thanksto` (:4793-4804)
  as the frontmatter's thanks note (`\lx@add@pubnote@thanks`); the SIU template's `{ozet}`/`{IEEEanahtar}` (2011/11/03 "V1.8
  based on V1.7a" :1659-1662, :3745-3790), the Turkish abstract and keywords set in the body under "Özetçe"/"Anahtar
  Kelimeler" (the English ones stay the frontmatter's), the abstract bold and the keywords bold italic as the template's
  two-column form sets them (:3746, :3765). IEEEtran/IEEEtrantools shared IEEEeqnarray (ieeetrantools_sty.rs):
  `\@IEEEeqnnumcols` set from the column specification (IEEEtrantools.sty:1352, :2498/:2535), `\IEEEeqnarrayseprow`,
  `\IEEEeqnarrayseprowcut`, `\IEEEeqnarrayrulerow`, `\IEEEeqnarraydblrulerow`, `\IEEEeqnarraydblrulerowcut` (:1566-1690;
  IEEEtran.cls:3247-3360) as spanning leader-rule rows in an `{IEEEeqnarraybox}` (open, KPE #710 residual: in an
  `{IEEEeqnarray}` a rule row keeps its number but not its rule and a separator row is removed with its number, as
  `{eqnarray}` rearranges rows; RED `alignment/ieeeeqnarray_rule_rows_keep_rules_and_numbers`; ruled 2026-10-15: such
  rows stay numbered rows with their rules, as pdflatex numbers them — kernel-wide for the eqnarray family, batch 68e
  with its own A/B, low paper impact: the 32 sampled papers with rule rows use an `{IEEEeqnarraybox}`; IEEEtrantools'
  `\PackageError` under IEEEtran stays silent). An IEEEtran heading's tag is its heading form, "I.", "A.", "1)" and an
  appendix subsection's "A." (IEEEtran.cls:2570-2587, :5397; **FIXED 78c**, KNOWN_PERL_ERRORS #1022; compsoc arabic "1.1",
  Perl's IEEEtran.cls.ltxml:160-174 conditional, since 78c r2). IEEEtrantools under any
  class but IEEEtran (ieeetrantools_sty.rs; under IEEEtran the package only raises its error, IEEEtrantools.sty:73-78, and
  the class binding's IEEEproof, lists and `\IEEEauthorrefmark` stay): the
  IEEEproof family with its QED switches (:2726-2747) and the IED lists with their parameters (:1000-1140), moved from the
  class binding into shared definers both call (`define_ieeeproof`, `define_ied_lists`), `[redeflists]` (:1122-1129),
  `\IEEEauthorrefmark{<n>}` as the package writes it (:2721-2723, the footnote symbols * † ‡ …), `\IEEEPARstart` (its
  text, as the class binding's); RED `block-model/ieeetrantools_ieeeproof_any_class` (64ra) GREEN. ieeeconf (ieeeconf_cls.rs): `\appendices` sets `\thesectiondis`
  (ieeeconf.cls :4103-4117) without the subsection numbering IEEEtran's `\appendix` sets, the RBO copy's `\ieeefootline` (:4602-4630) a `role=footline` publication note. Witnesses before → after
  (Errors; 68d2-rel vs candidate): `\ps@headings` 1805.00151 1 → 0, 1805.02797 1 → 0, 1811.10132 1 → 0, 1908.05569 1 → 0,
  1910.06140 1 → 0; compsoc executors 2202.10460 5 → 0, 2202.12377 5 → 0, 2203.08603 5 → 0, 2204.11491 5 → 0, 2401.11581 5 → 0;
  rule rows cs0608041 2 → 0, 0911.1713 2 → 0, 1004.2757 3 → 0, 1112.1125 3 → 0, 0908.3162 2 → 0, 1312.4895 3 → 0;
  `\@IEEEtunefonts` 0705.1682 1 → 0, 0705.1922 1 → 0, 0801.1002 1 → 0, 1912.02718 1 → 0; V1.6 names 0704.1833, cs0501064,
  cs0504030, 0805.0012, 2009.04715, cond-mat0207222 1 → 0, 0705.3677 2 → 0; `\thanksto` 1601.01675, 1603.05188, 1710.09738,
  1803.06921, 2304.11418 1 → 0; `{ozet}` 1504.02975, 1807.04577, 1910.03090 2 → 0; `\sublargesize` 1208.3665 1 → 0, 2103.13318
  3 → 0; `\IEEEbibitemsep` 2007.01482, 2106.14760 1 → 0; `\thesectiondis` 1605.01233 1 → 0, 1702.07429 2 → 1 (empheq's
  `\empheqlbrace`); `\@ifmtarg` 1510.02032 1 → 0; `\IEEEquantizevspace` 2601.10876 5 → 0; `\proof` 2106.12111, 2208.13860, 2310.09933, 2310.09935, 2503.20066, 2509.09277 1 → 0; `\@IEEEappendixsavesection`
  1907.08915 1 → 0;
  ieeeconf 1811.05306, 1811.01547, 1905.11167, 1909.12483 2 → 0, 1803.09650, 1807.07870, 1810.07122, 1610.00529, 1807.11264
  1 → 0. Word drops are the error text gone. Binding nets vs 67int2: IEEEtran 30 docs 0 worse (0710.3817, 0805.3118 1 → 0),
  IEEEtrantools 25 0 worse (12 errors in 8 docs → 0: 1304.3949 3, 1609.02977 3, 1407.6872,
  1710.09177, 1802.00899, 1905.00616, 2002.01226, 2003.00712 1 → 0), ieeeconf 26 0 worse (1603.06163, 1610.00529, 1807.11264 1 → 0, 1811.01547 2 →
  0). Not taken (authors' additions to their copies, not a version's interface): `\eg`/`\ie`/`\etal`/`\etc`/`\vs` (11),
  `{refsize}` (11, each copy its own size), `\teaser`/`\IEEEteaser` (8), `\name`/`\address`/`\orcid` (7), `\footernote` (4),
  `\IEEEabstract`/`\IEEEkeyword` (3); version conflicts: the SIU copy's Turkish `\refname`/`\figurename`/`\IEEEkeywordsname`,
  V1.6's Roman-appendix default; `\@makecaption` (10) is undefined under article too (a kernel gap, not IEEEtran's); `\I`
  (2) is not defined by IEEEtran.cls; hyperref internals (`\Hy@raisedlink`, `\hyper@anchorstart`/`end`, `\@IEEEtheHrefequation`,
  1 each). Repros class-bindings/{ieeetran_internals_documents_call, ieeetran_eqnarray_rule_rows, ieeetran_v16_names,
  ieeetran_pscc_thanksto, ieeetran_siu_ozet, ieeetran_v17_proof_at_begin_document,
  ieeeconf_v16_title_internals_and_footline}, block-model/ieeetrantools_lists_marks_any_class RED (68d) → GREEN;
  guards `perfect_kernel_batch67x` (15; review round: `ieeetran_v17_proof_yields_to_document_begin_document_proof`,
  `ieeetrantools_under_ieeetran_keeps_class_author_marks`, `ieeetran_appendix_clears_sectiondis`). Seen, not changed: the class binding's `\IEEEauthorrefmark{<n>}` prints the
  numeral `n` as a superscript (beyond-Perl, 2404.07847) where IEEEtran.cls:4539-4546 prints the footnote symbol
  (`{2}` → †) outside the `transmag` option (whose mark is the numeral), as the IEEEtrantools one now does.
- **Fleet packaging (68d study, not a loader bug)**: the fleet image's distro TeX Live (`/usr/share/texlive`) has no
  `asymptote.sty`, which TL 2025 has and the probe reads raw without a warning: 8 + 5 `missing_file:asymptote` tasks in
  2605/2606 (2605.00095, 2605.03389, 2606.02087); a packaging fix for the worker image.
- **67y** (binding gaps G2: class bindings lacking commands their class defines; bindgaps/g23_macros.tsv rows amsart,
  revtex4-2, revtex4-1, kluwer, elsarticle, elsart, egpubl). Per binding (commands, real file:line, witnesses):
  amsart (ams_support_sty.rs `amsart_class_internals`, all three AMS classes) — `\except`/`\for`/`\forany` and the
  `toc`/`lof`/`lot`/`runhead` switches (amsart.cls:63-96, :305, :1143-1145; `\newswitch`/`\setTrue`/`\setFalse` real,
  no longer no-ops; 1304.2934, 1601.05128, 1912.08145), `\abstractbox`/`\@setabstract`/`\@setabstracta` with
  `\@setabstract` run at `\maketitle` (:837, :856-864; the box it sets the frontmatter `<ltx:abstract>`, one set after
  the title where it ends: 1512.02361, 2602.22909; 2303.07306, 2201.00740), `\@keywords`/`\@subjclass` empty (:554/:564; 2510.22858), `\copyrightyear`/`\copyrightholder` (:687-689), `\@tocwrite`/`\@tocwriteb`/`\@tocline`/`\l@figure`/
  `\l@table` (:1189-1241; 2004.14791, 1805.00244, 2503.14638), `\@defaultbiblabelstyle`, `\bibliofont` (:1244, :1262;
  2105.14892); `{proof}` after `\let\proof\relax` + `\usepackage{amsthm}` by the loader below (1510.08833 and 5 more).
  revtex (revtex4_support_sty.rs; `define_revtex41_internals` for 4.1/4.2) — `\doi`/`\doibase` now for 4.2 too
  (revtex4-2.cls:1986-1988; 2010.08759, 2301.10789), `\@@startlink`/`\@@endlink` an `<ltx:ref>` pair (:1990-2009;
  1211.1037), `\onecolumngrid@push`/`@pop`, `\onecolumn@grid@setup`, `\set@vsize` (ltxgrid.sty, layout no-ops;
  2311.06373, 2609.36113), `\let@environment` (ltxutil.sty:97; 1101.2683), `\widetext@top`/`@bot` (:7459; 2101.04636),
  `\frontmatter@title@above/format/below`, `\frontmatter@finalspace` (aps4-2.rtx:212-214; 2006.03287), `\REV@text`
  (:7547; 2202.11665), `\altaddress` (:7520), `\clo@groupedaddress`… (:6283-6308; 2402.07499), `\prep@math@patch`,
  `\ltx@footnote@pop`, `\setup@hook`, `\setfloatlink`, `\theequation@prefix`. kluwer (kluwer_cls.rs) — named
  references through natbib, borrowed (kluwer.cls never loads it: `\@ifpackageloaded{natbib}` false; `\cite` =
  `\citep`, `\cite*` = `\citeyear` (:2125), `\inlinecite` = `\citet`, `\opencite` = `\citealp`, `\shortcite` =
  `\citeyearpar`; all `\cite` under `numreferences`; kluwer.cls:2007-2145; 1306.4881, 1408.6262, 0802.0687), endnotes
  (borrowed endnotes.sty; :1909-1975; 0910.2393), `\lsim`/`\gsim`/`\nequiv`/`\dC`…`\dQ` as ≲ ≳ ≢ ℂ ℙ ℕ ℝ ℤ ℚ (:162-176;
  astro-ph0309194, math0212055; OXIDIZED_DESIGN_DIVERGENCES #598), `\I`, `{subequation}`, `\qed`, `{kaprotate}`,
  `\nomenclature`/`\nomen` and `\abbreviations` notes with their bold headings (:950-960), `{motto}`
  (a quote), `\tabcapfont`/`\figcapfont`/`\figtabdot`, `\crline`, `\rcline`… (as `\cline`), `\filedate`.
  elsarticle (elsarticle_cls.rs) — TL v3.4c `\tnotemark`, `\myfooter`/`\myfooterfont` (`[#1]#2␣`, :1022),
  `\abstracttitle` (`\@elsarticleabstitle`, :672, which names the frontmatter abstract; `\abstractname` stays the
  document's), `\keywordtitle`, `\resetTitleCounters` + `tnote`/`fnote`/`cnote`, `\absbox`/`\keybox`,
  `\Columnwidth`, `\elsLabel`/`\Newlabel`/`\elsRef`, `\comma`, `\@eadauthor`, `\pprinttitle`, `\hashchar`,
  `\elsparagraph` (2203.02441, 2209.12222, 2204.07076, 2401.05905); v1.21's (100 of 225 shipped copies) `\email`,
  `\correspondingauthor`, `{nomenclature}`, `{deflist}`/`\defitem`/`\defterm`, `\normalMode` (ends the appendices) as
  `Fallback` guesses (1305.6441, 2103.08145, 1303.2653, 2404.00783); the AAS journal abbreviations (`\apj`, `\mnras`…)
  from aas_macros' binding, provide-only, where the paper's copy has them pasted in (`class_code_contains`; 0812.4232,
  1504.02326, 1807.02064, 2003.05953, 2401.03908); `\@journal` stored by `\journal`, empty until then (:1302;
  elsart_support_core_sty.rs; 1212.6083, 1007.3192, 0812.4577). elsart (elsart_support_sty.rs, elsart_cls.rs) — `{pf}`/`{pf*}`
  an `<ltx:proof>` pair (elsart.cls:477-483; 0708.4409), `\cty`/`\cny`/`\jr`, `\@blstr`, `\ext@algorithm`,
  `\ftype@figure`, `\output@glob@notes`, `\if@nameyear`/`\if@natbibloaded` (0907.1629 adnd.sty), the MACMILLAN elsart5
  copies' `\KEY` and `\nolinenumbers` (0801.1765, 0904.0552), `\@journal` "Elsevier Science" until `\journal`
  (v2.18:28); the differential `\d` (Perl elsart_support.sty.ltxml:48) only where the paper's copy redefines it
  (`\if@symbold\else\def\d{\,\mathrm{d}}\fi`, v2.18:1382, 98 of 150 copies) or ships none, else the kernel's accent
  (elsart5/elsart4 copies; cond-mat0610502, 0706.1711), and the `symbold` option that keeps the accent. egpubl (egpubl_cls.rs) — `\p@ConfYear` stored by
  `\ConfYear` (egpubl.cls:986-987 v4.0; 1409.5623), `\shortcite` (:2063), `\CCScat` (:816; 1102.4258),
  `\EuroVisTutorialEditors`/`\EuroVisPanelsTutorialsEditors`/`\EuroVisGDxDREditors` (2604.14791),
  `\copyrightTextTitPag` (the frontmatter copyright, its leading `\copyright` left to the frontmatter's ©; 2404.02634;
  OXIDIZED_DESIGN_DIVERGENCES #599)/`\copyrightTextRunPag`, `\p@journalSubmissionText`,
  `\pStartPage`, and the single copies' selectors (`\PaperL2P`, `\ArxivPaper`…, `Fallback`). Kernel: the loader
  reloads a file a class binding marked loaded (`<file>:class_marked`: amsart's amsthm, ams_support_sty.rs) whose
  `\ver@<file>` the document let to `\relax` or `\undefined`, and `\@ifl@aded` answers false
  (`state::unmarked_by_document`; KNOWN_PERL_ERRORS #720); no other file (user ruling 2026-10-14: no general loader
  `\ver@` rule); the ltcmd declarators replace a `Fallback` guess or a kernel `Stub` as `\newcommand` does
  (`\lx@if@fallbackdefined`, latex_constructs_rust_only.rs; 2509.10728); `\addcontentsline` adds a list the node is
  not in yet (amsart's `\@tocwriteb` gave `inlist="toc toc"`; KNOWN_PERL_ERRORS #725). KNOWN_PERL_ERRORS
  #720-#725, OXIDIZED_DESIGN_DIVERGENCES #598-#599. Not ported: REVTeX's title-block production internals that documents replay for a second title page
  (`\titleblock@produce`, `\@author@finish`, `\frontmatter@author@produce@*`, `\title@column`, `\print@toc`: need
  ltxfront's author storage, which the frontmatter API replaces; 2105.06662, 2503.21669, 2604.15540), `\@makecaption`
  (undefined for every class, article's too), amsart's `\@secnumpunct` (set only inside amsart's `\@sect`; 1611.09402) and `\sectionname`… (:320-322, a running
  head in 1504.08082: a defined empty `\sectionname` put a space before every section number),
  elsart's `\@thebibliography` (a journal style's own `{thebibliography}` over it; 0907.1629). Repros
  class-bindings/{amsart_class_internals, revtex4_2_class_internals, revtex4_1_class_internals, kluwer_class_commands,
  elsarticle_class_internals, elsarticle_v1_21_commands, elsart_class_commands, elsart5_macmillan_key,
  egpubl_class_commands, kluwer_cite_star_and_title_notes, elsarticle_class_details,
  elsarticle_copy_aas_journal_macros, elsart_d_follows_the_shipped_copy}, loader/{amsart_reloads_amsthm_after_unmarking_it,
  amsart_reloads_amsthm_after_undefining_its_version, ltcmd_declarators_replace_a_fallback_guess} RED → GREEN; guards
  `perfect_kernel_batch67y` (17).
- **Residual of 67y — a font-size change after a formula rewrites its `tex=`** (RUST-ONLY, pre-existing): `$a\quad b$`
  then a top-level `\footnotesize` gives `tex="a\hskip 10.00002ptb"` (the XMath is unchanged); amsart's `\bibliofont`
  before `\bibliography` now triggers it (2105.14892, 2307.14222, 2609.15658, 2312.03234: every `\quad`). RED repro
  math-parse/quad_tex_after_a_later_size_change.
- **Residual of 67y — a `\par` in a frontmatter note pulls the formula after it into the styled text before it**
  (RUST-ONLY, pre-existing): `\lx@add@frontmatter{ltx:note}{{\bfseries Head:}\par\noindent $x$ -- v}` gives
  `<text font="bold">Head:<Math…/></text> – v`; kluwer's "Nomenclature:" heading (kluwer.cls:958-960, on a line of its
  own in the PDF) is run in for it. RED repro sectioning-frontmatter/frontmatter_note_par_then_math.
- **67z** (binding gaps G2, journal classes A: bindgaps/g23_macros.tsv rows sn-jnl, sigma, arxbj, informs, autart,
  asme2ej, aipproc, sagej, oup-authoring-template, ejpecp; every class but oup/ejpecp is one papers ship). Per binding
  (commands, real file:line, witnesses): sn-jnl (sn_jnl_cls.rs; 1802.08831's copy) — `\nomail` (:550; 2111.06962), the
  title/table/figure fonts `\Titlefont`, `\tablebodyfont`, `\figurecaptionfont`… (:443-461, :1231-1235; 2210.01472,
  2307.08421, 2402.15353), `{tablenotes}`, `\tnote` (:1280-1292; 2102.13454), booktabs for the copies that define its
  rules themselves (2006.16202 :1292-1309), `\sidewaystablefn` a no-op (:1361; 1802.08831, 2012.15392 — the kernel's
  `{table}` keeps the footnote), `\artnote`/`\miscnote` (:631-638; 2203.15424, 2605.14802) and
  `\intern`/`\equalsup`/`\equaladv`/`\equalcontribution` (2603.28334 :575-600, 2307.08214, 2501.01157, 2206.10381)
  frontmatter notes (a copy's optional-argument form with the copy's default text: 2606.24824's `\intern`, 2504.21788's
  `\equalcont`, `class_code_argument`), `\ackno` frontmatter acknowledgements (2404.03266 :639-646), `\presentaddresstxt`, the `\if…`
  switches, `\Newlabel`/`\jmkLabel` (2307.08421 :491), `{unenumerate}` (2303.09648 :1084-1116), `{biography}` with its
  bold `\author` (:1529; 2310.14485), `\orcidlogo`, the copies' `\abs`/`\argmin` only where the copy has them
  (2206.00975 :317-318); the begin-document `\let\proof\spiproof` now with or without program.sty (a preamble
  `\let\proof\relax`: 2211.03599, 2501.00469). sigma (sigma_cls.rs; 0705.3628 v3.0, 1404.0720 v1.7, 2609.01661 v1.8) —
  `\EmailDD`/`\EmailMarked`/`\URLaddressDD`/`\URLaddressMarked` notes, empty ones dropped (:77-83; 0706.0314,
  0711.1671), `\mail` where the copy has it (v1.8 :80), the class's theorem set on their own counters with the starred
  twins and `{conjecture}` from v1.2 (:161-178; 1301.4541, 1404.0720; OmniBus's fallback had numbered them by section on
  one counter; the v1.0/1.1 copies' set without them, nlin0510055), `\tsep`/`\bsep` (:182-191;
  0705.3628). arxbj (arxbj_cls.rs; 1106.2467's copy) — the VTeX URL and attribute helpers moved from arximspdf_cls.rs
  into ims_support_sty.rs, which both load (`\arxivurl`, `\doiurl`, `\surl`, `\setattribute`, `\tsub`; 1206.6586),
  `\docsubty` (:2259), `{keyword}[class=…]`/`\kwd` (:2490-2541; elsart's lost every entry) in ims_support_sty.rs
  with arximspdf's, whose collecting form a later block replaced (1107.3284's and 1309.4864's AMS codes): each block
  kept, the AMS one the subject classification, joined by `; ` (arxbj) or `, ` (arximspdf), under the class's headings,
  `\eqntext` (:1273; 1002.0175), `\startlocaldefs`, `\setvaluelist`/`\getitemvalue` (:392-502; 0811.0643), `\legend`,
  `{supplement}` with `\sname`/`\stitle`/`\slink`/`\sdatatype`/`\sdescription` (:2543-2598; 1302.0163), vtexthm's
  starred twin of every `\newtheorem`/`\newproclaim` on borrowed amsthm (:1302-1345; 1205.2947, 1307.5990), in
  ims_support_sty.rs, so arximspdf has it too (its arximspdf.cls:1287-1337; `{hy*}` 1107.3284). informs
  (informs_cls.rs; 1501.01126's informs3, 2309.11120's informs4) — the 133 `\BF…` bold symbols as
  `\mathbf`/`\boldsymbol` (:98-247; 1501.01126, 2008.02154), `\fs`, `\HD`/`\up`/`\down`/`\updown`, `\halmos`, `\n`
  (:391-913; 1402.4593, 1011.5677), `\SUBJECTCLASS`/`\AREAOFREVIEW`/`\MSCCLASS`/`\ORMSCLASS` classifications
  (:1001-1020; 1009.2653), `{Trivlist}`, the class's `\proof{head}` (:1829-1830; amsthm's took the head as text, "Proof.
  Proof.", 229 of the 456 inventory papers: 1301.2609) with theorem.sty in amsthm's place (:6; 2008.02211's
  `\let\proof\relax` + `\usepackage{amsthm}`), `\NatBibNumeric` a no-op, `{descr}`, the title/theorem fonts and hooks,
  `\ECRepeatTheorems` (:2680-2696; 1704.09011), the spacing switches, `\ECDisclaimer` dropped. autart (autart_cls.rs;
  1112.0879's copy) — elsart's `{keyword}` with `\PACS`/`\MSC` (elsart_support_core_sty.rs `define_keyword_markup`,
  shared; :1248-1258; 1106.2518, 1409.4624), `\sep`, `\e`, `\half`/`\threehalf`/`\quart`, `\Rset`…`\Zset`, `\nuc`,
  `\pol`, `\cv` (:722, :1429-1456; 1207.4104, 1611.06692), the theorem set as the copy declares it (:545-580; `{case}`
  2001.10125, a copy's `{assumption}` 1912.06343, separate counters in 0807.2769's, `secthm` 1909.02848). asme2ej
  (asme2ej_cls.rs; 1212.4243's copy) — `{acknowledgment}` acknowledgements (:1218; 1011.2269), `{nomenclature}`/`\entry`
  (:1209-1216; 1403.0296), `\qed` (:1246; 1610.03468), the fixed fonts (:301-311). aipproc (aipproc_cls.rs) — aipxfm.sty
  recorded loaded as the class loads it (:405; a paper's `\usepackage{aipxfm}` read the shipped file raw: 1010.4521,
  0705.1061; KNOWN_PERL_ERRORS #730), the designer commands (:212-216, 336-386), a copy's journal abbreviations where it
  has them (0809.4021 :1042-1124). sagej (sagej_cls.rs; 2101.10072's copy) — `{sm}`, `{credit}`, `{conflict}`, `{das}`
  statements (:464; 2505.13309, 2406.06038, 2204.02460; 1412.4039), each with `{funding}`/`{dci}` named by the heading
  the class prints ("Supplemental material", 2503.14633's copy "Supplemental Material", "Author contributions",
  "Declaration of conflicting interests", "Data Accessibility Statement", "Funding"; the XHTML prints `@name` as the
  heading, which had shown "sm." and "dci."), `{biog}`/`{biogs}` (:480; 1801.00336), `\sagesf`,
  `{refsize}`, `\titlesize`, `\absbox`, `\journalclass`, `\Highlights`, `\twitter`, and the rest of the class's
  `\RequirePackage` line (ifthen, rotating, calc, textcase, color: 2204.02460). oup-authoring-template (TL 2025) —
  `\vol` (:1054; 2104.15079), `\boxedtext` a frontmatter note led by its heading, the copy's constant ("Key Messages",
  "Significance statement") or the newer copies' first argument (:1117-1119; 2604.25257 :1271-1272; 2306.17210),
  `\secsize`… (:1676; 2606.05448's :1660), the `\sffamilyfont…` switches (:143-150; 2505.18036), `\@numbibtrue`; a
  newer copy's `\otherabstract` (a further abstract) and `\figalttext` (the image's description, as acmart's
  `\Description`, float_description.rs; the class drops it: OXIDIZED_DESIGN_DIVERGENCES #604; 2608.22901, 2608.14294,
  2609.06413) where it has them (2602.07066 :1232, :1935). ejpecp (TL 2025) — the class's theorem set within
  sections with its styles (:400-458; it had numbered `{theorem}` through; 1405.0815, 1406.6650), `\ARXIVID`/`\HALID`
  notes (:150-151, :316-328; 1104.1060, 2103.00338), `\ACKNO` (:152; 1103.0392), `\ARXIVPASSWORD`. Repros
  class-bindings/{sigma,snjnl,informs,autart,asme2ej,arxbj,sagej, ejpecp,oup_authoring_template}_class_commands,
  aipproc_paper_loads_aipxfm RED (68d2) → GREEN; guards `perfect_kernel_batch67z` (10). A/B vs 68d2-rel over 400
  witnesses + 100 pop67o (488 converted): Errors 1,291 → 354, error-free 61 → 410, Fatals 8 → 8, no paper gains Errors
  or Fatals; the 92 papers with fewer words lose error-recovery text, numbers the class does not print (sigma's and
  autart's statements on the class's counters, OmniBus's had numbered them by section on one counter), informs's
  duplicated "Proof." and amsthm's ∎, and the then-dropped alt texts (2608.22901, 2608.14294, 2609.06413); the 9 aipproc
  papers whose main file the harness missed (aipcheck.tex) 21 → 0 Errors. Binding nets vs 67int2: 0 worse in all nine
  (arxbj has none); Errors 127 → 18. Side finding, RED: class-bindings/elsart_support_theorem_tag_zero (under
  elsart_support — arxbj, elsart — a `\newtheorem` statement is tagged "Remark 0", its refnum 1; FIXED 77v,
  KNOWN_PERL_ERRORS #1008). Not taken: the
  modified copies' one-paper commands (sn-jnl's astronomy block 2108.07431/2507.08592, informs `{observation}` in
  `\TheoremsNumberedThrough`, oup `{methods}`/`\absection`, sigmasthesis.cls 2510.12158 — another class). Round 2
  (review fixes): repros class-bindings/{arximspdf_keyword_blocks_and_starred_theorems, snjnl_author_notes,
  sagej_v2503/sagej_sm_heading_follows_the_copy, snjnl_v2606/snjnl_notes_take_the_copys_default,
  oup_v2602/oup_figalttext_is_the_images_description} RED (67z) → GREEN, guards `perfect_kernel_batch67z` (15); the
  `{biography}` `\author` a primitive's before-digest code. Residuals: informs's `\TheoremHeaderFont`/`\ExampleHeaderFont`
  do not reach the heads (informs3.cls:1810-1845 sets them per style, TH and EX, `\def\theorem@headerfont{\normalfont
  \TheoremHeaderFont}`; theorem_sty.rs keeps one global `\theorem@headerfont`, and informs4's two fonts differ, so a
  global `\gdef` would set examples sans: pdflatex's small caps against our bold, PDF fidelity only); a loader rule
  running a shipped copy's own simple one-line definitions for names its binding leaves undefined (the modified copies'
  long tail across all ten classes) is a pending user decision; class-bindings/elsart_support_theorem_tag_zero stays RED
  (GREEN in 77v);
  sn-jnl with no shipped copy loads none of the class's packages (rotating…); the same visible-`@name` flaw in bindings
  outside this batch: sn_jnl_cls.rs `{declarations}` (`name='declarations'`, an environment sn-jnl.cls does not define)
  and jmlr_cls.rs/jmlr2e_sty.rs `\acks` (`name='acknowledgments-disclosure-of-funding'`) — FIXED 78c: jmlr.cls's
  "Acknowledgments", a jmlr2e.sty copy's own heading ("Acknowledgments" without a copy, 78c r2); `{declarations}`
  DELETED in 78c r2 as a speculative environment, no witness (0 of 206 shipped sn-jnl.cls copies define it, 0 of
  ~23,000 papers use it; the template writes `\section*{Declarations}`). Ruled 2026-10-15: arximspdf's
  borrowed amsthm also gives it `{proof}`, which the class lacks (pdflatex errs); kept, as it keeps the author's proof.
  The arximspdf/arxstspdf row from a 24-paper sample — `\ccline` (7 arxstspdf papers), `\freefootnotetext`,
  `\write@toc@…contentsline`, `\maxcsec`, `\setvaluelist` (1203.0565) — landed in 68k; open: arximspdf's own
  `{aligned}`.
- **68k** (binding gaps G2 C, class bindings whose papers ship their own copy; bindgaps/g23_macros.tsv rows gretsi,
  PoS, JHEP3, ws-procs9x6/ws-procs961x669/ws-ijmpe, svmult, pnas-new, interact, mdpi + Definitions/mdpi, svjour3,
  arxstspdf/arximspdf, siamart/siamonline; each command checked against the witnesses' copies, `copies.py` tallies in
  the batch report). Per binding (commands, real file:line, witnesses): gretsi (gretsi_cls.rs) follows the paper's copy
  where the releases conflict — v1.2 (2008, 45 of 62 papers, 1001.1912's copy) `\titre`, `\auteur` = `\author` of
  `\coord{first}{last}{mark}` names (:154-157), `\adresse`/`\address` of `\affil{mark}{lines}` (:158-166),
  `\resumefrancais`/`\frenchabstract`, `\resumeanglais`/`\englishabstract` (:106-122, the English one in
  `{texteenanglais}`, English under babel), `\legende` (:22-58); v1.3 (2023, 2304.10248's; the default) `\auteurs` of
  `\auteur{first}{last}{email}{mark}` (:96-97, an author each with e-mail), `\affils`, `\resume`/`\abstract` (:78-87),
  its packages (babel `[french,english]` with the `english` option's language, algorithm2e, booktabs; :35-75); the
  stub's `\auteur{}{}{}{}` (name + address/email notes holding the e-mail and the mark) is gone; names' marks request
  the affiliations so labelled (1001.1912, 0706.1914, 1311.3768, 1512.06999, 2206.05928, 2304.10248, 2310.15635). PoS
  (pos_cls.rs) `\@firstaubox`, `\PoSspecialurl` (`\relax` from `\begin{document}`, PoS.cls :452, :1416-1428),
  `\PoSpaper@url` (v1.2.6 :656, :704-709; 0912.1441, 1606.09248, 1812.08139; KPE #811). JHEP3 (jhep3_cls.rs)
  `\arXivid` (v3.1.4 :1708; 0704.3719, 0709.3309) and `\@firstaubox` (:748; KPE #810). The AAS journal abbreviations
  a copy pastes in (`\apj`, `\mnras`, `\aap`…) for PoS (16 of 92 copies: 0810.0108, 1708.06808), svmult (7:
  1711.01112, 1801.09940's svmult_aa.cls), svjour3 (9: 0812.0839, 1111.3868) and mdpi (2212.11355, 2211.06541,
  2510.22959's mdpi_mod.cls) through one helper, `shipped_class::provide_shipped_aas_macros` (elsarticle's 67y code
  moved there). World Scientific: ws_journal_cls.rs (every WS class) `\titlefont`, `\authorfont`, `\tablefont`,
  `{tabnote}[heading]`, `{tabfootnote}`, `\tabmark`, `{otherinfo}`, `\citelow`, `\figurebox` (ws-ijmpe v1.0 :58-71,
  :334-340, :740-763, :951, :667; 0711.3504, 0712.0330, 0811.0496, 1302.0454); ws_procs_cls.rs `\eref`…`\Tref`
  (ws-procs9x6 :984-993; 0704.0369, 1605.04791), `\figurecaptionfont`, `\wstoc` (a contents line, kept out),
  `\mastertoc` (`\tableofcontents`), `{unnumlist}`, `{algorithm}`/`{assumption}` (:918-921), `\aindx` as the copy writes
  it (`\index{author}{…}`, ws-rv9x6 `\index[aindx]{…}`). svmult (svmult_cls.rs; v5.4 0912.1479's, KPE #812) `\ts`,
  `\if@runhead`, `\svparindent`, `\verbatimindent`, `\twocaptionwidth`, `\mtaddtocont`, `\hyperhrefextend`, `\subfigures`
  (the counter restored at the figure's end through `\AddToHook{env/figure/end}`), `{abbrsymblist}`, `\greeksym` (the
  Symbol font's Greek letter, OD #623), v1.1's `\micro` (Euler's upright Greek μ, U+03BC), v5.5's
  `\circledmark`/`\formtmp`, v5.15's `\Description` (the float's description, as acmart's; 2602.21993). pnas-new (pnas_new_cls.rs) `\bibsplit`, `\Firstpage`,
  `\Parasplit`/`\Endparasplit` (layout: nothing), `\dataavail` (a titled paragraph; v1.47 :468-492, :688; 2208.11618,
  2602.17158). interact (interact_cls.rs; v1.05 1711.05866's) `{abbreviations}`, `{jelcode}`/`{pacscode}` (and v4.0's
  `{classcode}`) as classifications, the upright Greek `\ualpha`…`\upartial` and bold `\ubalpha`… (:167-226), `{tabnote}`,
  `\jvol`/`\jnum`/`\jmonth`/`\jyear`/`\jname` stores, `\history`, `\figwidth`, `\x`, v4.0's `\colrule`/`\botrule`
  (1901.07015, 2306.12924, 1709.03559, 1709.02637). mdpi (mdpi_cls.rs) `\conferencetitle`, `\contributed`,
  `\fourthnote`/`\fifthnote`, `\emails`, `\orcidauthorONE`…/`\orcidONE`… (2017) and `\orcidA`…`\orcidZ` (since 2018,
  were no-ops for A-F) as the author's ORCID, `\conflictofinterest`, `\useofartificialintelligence`,
  `\linksupplementary` (`\doinum` now stores `\@doinum`; round 2: else the copy's default DOI), `\bio`, `\justifyorcenter`, `\highlight`, `\CorrStatement`,
  `\dist` (1709.06513, 1506.06852, 1707.09764, 2311.18672, 2208.12066). svjour3 (svjour3_cls.rs) `\lastand` (an `\and`,
  KPE #813; 1907.12906), the economics edition's `\JEL` (1402.2599 :314-317, 6 copies) and the 2023 edition's
  `{funding}`/`{dataavailability}`/`{conflict}` (2411.13175 :1133-1159). arxstspdf/arximspdf (arximspdf_cls.rs)
  `\ccline` (a `\cline` per range), `\bhline`/`\thline`/`\fhline`/`\lhline`, `\freefootnotetext`, `\relateddois` (a note,
  its `\relateddoi` DOI links), `\shref`, `\write@toc@…contentsline`, `\maxcsec`…, and `\setvaluelist`/`\getitemvalue` moved
  from arxbj_cls.rs into ims_support_sty.rs for both
  (1210.0385, 1011.1160, 1010.0304, 1203.0565, 1205.5627). siamart/siamonline (siamart_cls.rs) `\proofbox`, `\slugger`
  (page furniture), `\siamprelabel`, `{DOI}`, `{AM}`, `{MSCcode}`, `{ACM}`, `{romannum}`, `{remunerate}`,
  `\appendixnotitle`, siamonline's `{Code}` and `\dedication` (2006.12120's siamart220329 :491-930, :1426; 1805.01916,
  1901.02925, 2306.16314). Class-interface commands only; a copy without one leaves it to the paper
  (`DefinitionOrigin::Fallback`). Repros class-bindings/{gretsi_v12/,gretsi_v13_frontmatter,pos_aas/,jhep3_arxivid,
  ws_procs_class_commands,ws_ijmpe_table_notes,svmult_aas/,pnas_layout_and_dataavail,interact_class_commands,mdpi_aas/,
  svjour3_statements,svjour3_jel_classification,arxstspdf_class_commands,siamart_class_commands} (RED 67int10 →
  GREEN); guards `perfect_kernel_batch68k` (12). A/B vs 67int10 over 400 row witnesses + 100 pop67o papers: errors
  1,374 → 691, 342 papers fewer, none more; every word drop an undefined command's name or its arguments no longer
  typeset (`\wstoc`'s title and authors, `\apj` → "ApJ"). Round 2 (review 2026-10-16): mdpi `\linksupplementary`
  without the paper's `\doinum` links under the default the copy computes (2208.12066's :523-530
  `\doinum{10.3390/\@doiabbr\@pubvolume\twodigits\@issuenum\fivedigits\@articlenumber}`): the binding inputs the journal
  table the copy inputs (`Definitions/journalnames.tex`, its `\@doiabbr`), executes the copy's `\ExecuteOptions`, keeps
  `\@pubvolume`/`\@issuenum`/`\@articlenumber` as the class does and ports `\cutdigits`/`\twodigits`/`\fourdigits`/
  `\fivedigits`; without the table, a journal it names or numeric values, the file's name and a Warning (19 papers:
  2208.12066 → 10.3390/s23001234, 1901.06933, 2607.17752, `preprints` 2111.09774 → 10.3390/11020074); gretsi abstracts
  in the class's order, Résumé first, the last call of each kept, added when the title is made (`\lx@maketitle@begin`;
  2205.13867, 2206.05928, 2304.07234, 2411.07577); v1.3 `\auteurs` keeps the text between its `\auteur` calls with the
  name before it, past separators (2508.03934); svmult `\if@runhead` from the version's option (`runningheads` v1.x,
  `norunningheads` v5.x), `\subfigures` prefixed only outside `\if@numart` (:1918-1920; `\title*` 1a), `numart`/`book`
  options; interact `\ubvarrho`/`\ubvarsigma` (:223-224); sv_support: an `\institute` piece opened by `\\` after `\and`
  still leads with its mark (1810.10167: Yaohua Hu's institute 2 was shared). Repros class-bindings/{mdpi_doi/,
  gretsi_v12/gretsi_v12_abstract_order,gretsi_v13_abstract_order,svmult_v11/,svmult_aas/svmult_norunningheads_subfigures,
  svjour3_marked_pieces_after_a_break}; guards `perfect_kernel_batch68k` (18). Open: the fuzzy name label of a creator
  (`Tag ltx:creator`, base_utilities/defs/author_macros.rs; Perl Base_Utility.pool.ltxml:870-876 the same) keeps the commas of the marks
  its name shows (`Simon Lepage1,2` → `fuzzy:Simon_Lepage1` and a stray `2`), and the stray piece is an accidental mark
  request: it takes a shared mark's affiliation from the author showing it alone (2508.03934: David Picard loses LIGM to
  Simon Lepage) and supplies the requests marked labels lack elsewhere (jheppub `\affiliation[1]` with `$^{1,2}$`
  names, 1912.00557; svjour3 1810.10167); dropping the commas alone, sampled over 60 comma-mark papers, fixed 7, broke
  1912.00557 and moved 1507.03278 — needs one rule (shown marks as the requests for labelled affiliations), pending a
  ruling (RED repro class-bindings/gretsi_v13_shared_mark_affiliation, 77t; quality ledger rank 93); PoS/mdpi merged authors split at `\ ` separators (1808.07470, 1909.00451, 2502.15737) and svjour3's empty
  creator for an unmarked `\institute` with several authors (sv_support) — frontmatter rows of their own; pnas-new
  `\theabstract` (the class's stored abstract; 2602.21299 reprints it); arximspdf's own `{aligned}` (:1209);
  `\@makecaption` from a paper's hangcaption.sty
  (interact 2208.03568, svjour3 1702.06262: G1). Not this payload's (a copy's private one-line edits, pending user
  decision (a)): JHEP3 `\mathph` (0901.2133), `\jcap` (0901.3074), JHEP3MaxThesisMod's thesis commands (1210.6500); PoS
  `\ReportNo` (1310.3045, 1012.2709's PoSmod), `\Preprint`/`\PREPRINT` (1601.04101, 1406.7652, 1712.05633), PoS_mod's
  `\speakershort`; svmult `\emailbrack` (1707.04674), svmult12 `\Scriptsize`, svmultivs_gm's `\ContactAuthor…`/
  `\Institution…`/`\authoremails` (7 papers), svmult6 `\chapterauthors`/`\acknowledgments`/`\shortauthname`, v5.5
  `{articlededication}` (2 copies) and `{contribution}` (1); mdpi 2009 `\arttype`/`\journal`/`\keywords`/`\pagerange`/
  `\sysname`/`\issn`/`\logo`/`\logosize` (0912.5334), `\academiceditor`, mdpimod's `\lastrevision`/`\titlenote`,
  `{Axiom}`, `{Conjecture}`, mdpi_pre's `\nameofjournal`, my-mdpi's `\bio`; svjour3 `\preprintnumbers` (2501.18289),
  `\makesecondtitle`, `\authormark`/`\issue`, svjour3mod's `{code}`/`\arxivnumber`/`\journalref`, svjour3VISbi's
  `\doi`, `\citetop`/`\doinum`/`\papernum`, `\MCS`, `\journalinfo`, `\rmGamma`…, `{das}`, `{datavail}`; interact
  `{published}`, `\subjclassname`, `{github_repo}`, 2404.11552's `\Ac`…, `{graphicabstract}`; siamart `\R`/`\F`/`\N`/`\eps`
  (template shortcuts in 3-7 copies), `\fundingl` (2111.11645), siamonline `\disclaimer`, `{subjects}`,
  `\supplementary`, `\correspondings`, `{setup}`; ws-procs9x6adap's `\citelow` copy. Excluded by the brief: svmult/
  svjour3 `\spn@wtheorem` (68g's `\spnewtheorem`).
  Integrated with 68j as int11 (gates 5,593/5,593; bundle A/B vs 67int10 over 1,038 papers — the 68j and 68k witnesses +
  186 of pop67o: Errors 2,906 → 1,749, 562 papers fewer, none more, Fatals 8 → 8; every word drop, by bag-of-words diff,
  is error text, an error's invisible-times product in a formula, or `\wstoc`'s TOC-only arguments; nets 136/136 0 worse).
- **68n** (class-defined keywords reach the frontmatter API; the run-353 cluster of papers whose only errors were
  `undefined:\keywords`, 373 papers, + R9 of the rawclasses lever, absorbing 67t; OXIDIZED_DESIGN_DIVERGENCES #634,
  KNOWN_PERL_ERRORS #835). K11 (frontmatter_stores.rs) serves the table's keywords setters in two more shapes, each by a
  check on what the class defined: (1) a one-argument `\keywords` a raw class's refused abstract defines for its body —
  wiley-article.cls:446-489's `\NewEnviron{wiley@abstract}` (nbdt-article.cls:430-473, and its renamed copies
  wiley-networks, wiley-article_final, zadpreprints, 1-article, wiley), astlb.cls:199-205's `{pazhabstract}` reached
  from `\endabstract` → `\maketitle` → `\@maketitle` (pazhastl, pazhb, pazhb_eng, pazha2, pazh_mod, pazhastlm,
  astrobul), mtt.cls:176-181's own `\renewenvironment{abstract}` (mttr) — found by `abstract_setter_definitions` and
  defined for the abstract's body only by `\lx@abstract@class@setters` (the head of `\lx@begin@abstract`,
  base_utilities/defs/frontmatter_fields.rs): the frontmatter's keywords, or, in an abstract kept in place as a supplement's, a paragraph in
  place with it; a refused `\renewenvironment` now keeps its code as `\lx@dropped@<name>`/`\lx@dropped@end<name>`
  (sect08.rs); (2) a `{keywords}`/`{keyword}` whose begin code only typesets a label (`label_only_begin_code`:
  paper.cls:621-627, acmconf.cls:340, pasa, biometrika, irmaems, ieice) becomes `\lx@class@keywords`: the environment
  is the frontmatter's keywords named by the label, `\keywords{…}` the keywords of its argument, a bare `\keywords
  text` the class's own code. Witnesses before → after (base 67int11-rel): 1706.00599, 1801.08881, 1202.2889,
  astro-ph0306137, 1507.06302, 2103.05801 each 1 Error → 0 with `<keywords>` (pdflatex prints them in the abstract);
  rawclasses lever, paper.cls: 9 of 10 papers (0707.1307, 0712.4042, 0808.0240, 0809.0271, 0812.2713, 0901.1497,
  1004.5246, 1007.3042, 1008.0145) run-in `<paragraph>` → `<keywords name="Keywords:">`, 0711.1008 has its
  `{keywords}` commented out. A/B vs 67int11-rel over the 373 cluster papers + 100 of pop67o + 150 papers on shipped
  raw classes defining a `{keywords}`: cluster Errors 374 → 43 (330 papers to 0, 331 gain `<keywords>`, none worse),
  pop67o identical (1902.02884 times out in both), shipped 18 gain `<keywords>`, errors identical, the only word
  drops the label moved into the keywords' name. Repros sectioning-frontmatter/class_keywords/* RED (67int11) → GREEN
  (three CONTROL); guards `perfect_kernel_batch68n` (22 with round 3). Not served (listed): the "article" row (36 papers: 25 with the
  document's own `\renewenvironment{abstract}{…\providecommand\keywords{\par\medskip\noindent\textit{Keywords:}…}}`,
  a parameterless run-in keywords inside an abstract the kernel's lock refuses — 1205.4818, 1403.4689, 2405.19523 —
  as hsrbericht.cls:348 in TeX Live; 5 shipping an arXiv template named `natbib.sty`/`neurips_202x.sty` whose
  `\keywords` our bindings of those names shadow — 2308.08611, 2403.02576, 2410.06567, 2411.12876; 1 defining it
  nowhere, 2403.06358); cup-pan.cls:134-147 and its copy mytemplate (6 papers: 2008.06930, 2102.01732, 2004.14479)
  patch the locked `\abstract` with `\preto` then `\appto`, and the refused-definition record keeps only the last
  patch, which lacks the `\keywords`; pazh's `\englishkeywords`; 2305.04260's `\DeclareObjectType` error (another
  cluster). Label-only shapes, in the versions read (TeX Live 2025 where it ships the class, else the papers' copies):
  converted — paper.cls (TL), pasa, biometrika, irmaems, ieice, emsprocart's label-only copies (1202.4161, 1006.0632,
  1610.08545, 0803.2841, 1203.6509), lni's older copies `\fontsize{9}{10}\selectfont\noindent{\bfseries Keywords:}`
  (1604.03313, 1810.05361, 2108.00996, 2110.01641); not taken (each stays the class's): a flag (tADR2e/rQUF2e/gGAF2e/
  tCPH2e `\global\keywordstrue`; TL 2025's lni.cls:561-565 and its copies 2403.01613, 2402.18227, which also `\def\and`),
  a box (ar-1col, iau, ar2e, biom, IOS-Book-Article, iosart2x, ifacconf, autart `\keyword`), a list (fundam, basi,
  ragtime, cpc-hepnp, fac), a definition (cidarticle `\def\and`; elsarticle-like `\sep`), a conditional (emsprocart's
  `\if@twocolumn\section*{\keywordname}` copies 0803.2060, math0612455; icmart; mnras-like), an optional argument
  (vmsta), centering environments (sf2a-conf), jmlr.cls:235 `\bgroup\leftskip 20pt…{\bfseries Keywords:}` would match
  but jmlr has a binding. Round 2 (review findings): the label becomes the keywords' `name` as its text only
  (`LabelText`; round 1 put a group label's `\noindent`/`\vskip` into the name and its `\bfseries` onto the keywords
  text of 10 of 18 converted shipped papers — emsprocart 1202.4161/1006.0632/1610.08545, ieice 2002.00666/1604.03276/
  2509.21881, lni 1604.03313); Part A names the keywords by the class's label too (`label_before_argument`: astlb "Key
  words:", pazh "Keywords:"; wiley's `\allcaps` and mtt's `\item` keep the default); `\keywords{…}` is the keywords of
  its group only when `\par`, a blank line or `\end…` follows it, else the class's code runs (`\keywords {alpha}, beta`
  kept together); `\end<name>` outside the environment is the class's own end code; a supplement's in-place abstract
  typesets its keywords with the class's own definition when that only typesets label and argument (kwmtt.cls:
  pdflatex's "Keywords: english words"), else as a paragraph of the class's label where its code shows one and the
  keywords; a found setter that does not use its argument (beilstein.cls:854, fbithesis.cls:272) is the class's own
  definition, not the keywords. Guards `perfect_kernel_batch68n` 11 → 20. Round 3 (re-review): where the class's label
  cannot be read (mtt.cls:181's language-chosen `\mtt@keywords`) the in-place keywords have no label — round 2 printed
  the English default "Keywords:" (with a double gap) where pdflatex prints "Ключовi слова:" (1310.0585, 1310.0589,
  1212.3890); the main frontmatter `<keywords>` of such a class carries the default name (its language-selected label
  cannot be read without running class code), a known limitation. A class macro's skip (`\quad`) or a `\kern` between two
  label pieces is one space of the name (round 2: `\textbf{Key}\quad\textbf{words}:` "Keywords:", base "Key words:").
  Guards 20 → 22 (`group_label_pieces_parted_by_quad_and_kern`,
  `supplement_abstract_keywords_without_a_readable_label_have_none`).
  Integrated as int13 (gates 5,618/5,618; bundle A/B vs 67int12 over 672 papers — the 373 cluster papers, 150 shipped
  classes with `{keywords}`, 150 of pop67o: Errors 2,165 → 1,834, 331 papers fewer, none more, no paper losing over 1 %
  of its words; nets 136/136 0 worse).
- **68g** (llncs theorem numbering, the 68a residual KNOWN_PERL_ERRORS #743): llncs_cls.rs installs the class's
  theorem set with llncs.cls's own code (:1142-1174, TL 2025 v2.26; the same in every shipped copy v2.2-v2.26, checked
  over 18 versions from 1,250 arXiv llncs papers): `{theorem}` by section only under `envcountsect` (reset by section
  under `envcountreset`, by chapter otherwise), the rest through `\spn@wtheorem` over `\@spynthm`/`\@spxnthm`/`\@spothm`
  mapped onto `\spnewtheorem` (the theorem counter under `envcountsame`), `{definition}`'s body italic as the class
  sets it. Was "Theorem 0.1", "Fact 0.2" under every option (Perl the same); now pdflatex's numbers under default,
  envcountsame, envcountsect, both and envcountreset. Repros class-bindings/llncs_theorem_numbering_{default,envcountsame,
  envcountsect,envcountsame_envcountsect,envcountreset} (RED 026cf18f87 → GREEN); guards `perfect_kernel_batch68g` (5),
  `perfect_kernel_batch68a::llncs_class_internals` flipped ("Fact 2", "Assumption 3"). A/B vs 026cf18f87 over 480
  papers (400 llncs, 41 of them with a numbering option; 6 witnesses; pop67o): Errors 605 → 605, Fatals 4 → 4, words
  unchanged; 357 identical, 50 differ only in theorem tags/refnums/ids, 64 also in `{definition}`'s italic body (font
  attributes, italic box widths), 1 (1610.02500, class fac) a math-parse nondeterminism (a base rerun equals ours).
  Witnesses: 0906.4121 "Fact 3.1" kept, its lemmas/definitions now "Lemma 2.1", "Definition 3.1" as the PDF;
  1003.2851, 2303.08726, 2202.10090 (envcountsame+envcountsect "Definition 2.1", "Problem 2.2"), 1312.1526, 1104.3211,
  1212.0139, 1808.10364, 1412.6065, 2308.02825 checked against pdflatex. llncs net vs 67int8 and vs 026cf18f87: 0
  worse. Round 2 (the 68g review): `\spnewtheorem` is the class's dispatch (llncs.cls:1061, block :1060-1125) over the
  `\lx@spnewtheorem` primitive, whose branches refuse an env already defined (`\@ifdefinable`, an Info) and define
  `\<env>name`; `\@spothm` follows the paper's copy of llncs.cls (v2.26 without one; OXIDIZED_DESIGN_DIVERGENCES #614
  corrected): v2.16+ make the env an alias counter of the base (`register_counter_alias`, shared with aliascnt_sty.rs),
  v2.24+ reset it by the section ("Hypothesis 1, Conj 2" in each section, v2.20 "Hypothesis 3, Conj 4"), an undefined
  base is the class's "No theorem environment" Error. hyperref's `\autoref` names an alias by its own
  `\<type>autorefname`/`\<type>name` (none if neither, as pdflatex): "Lemma 2", "Definition 3" under envcountsame
  (round 1 "Theorem 2", "Theorem 3"), a paper's `\spnewtheorem{obs}[theorem]` "Observation 4", an aliascnt lemma
  without names "2" (was "Theorem 2"). A `\newcounter` of an alias (`\let\c@case\relax\newcounter{case}`, 2303.08726)
  gives it its own register again (round 1: every case stepped the theorem counter, "Lemma 25" for the PDF's "Lemma
  23"). Guards `perfect_kernel_batch68g::{llncs_autoref_envcountsame{,_v214_copy}, llncs_spothm_section_reset{,_v220_copy},
  llncs_spnewtheorem_errors, llncs_alias_newcounter, aliascnt_autoref_names_the_alias}`, refs asserted in the five
  numbering guards. A/B vs 68g r1 over 568 papers (the round-1 480 and 95 hyperref+`\autoref` or aliascnt papers of
  pop67o): Errors 1,169 → 1,169, Fatals 6 → 6, words unchanged; 542 identical, 17 differ only in `\autoref` names (all
  the env's, as pdflatex: 2202.10090, 1312.1526, 1504.06105, 2412.05162, 2601.04573 "Nonexample 9", 2102.01037's
  aliascnt `{thm}` " 3.4" with amsthm's `\thmname`), 2303.08726 in its case tags and the lemmas after them (now the
  PDF's). Ruled: a paper's own `\newtheorem{x}{X}[section]` keeps "X 1.1" (the PDF's "X 11", llncs.cls:1058
  `\edef\@thmcountersep{}`, OXIDIZED_DESIGN_DIVERGENCES #615). Open: latex.ltx `\@definecounter` keeps an existing
  `\the<ctr>` ("Command `\thecase' already defined -- not changed"); the kernel's `\newcounter` redefines it, as Perl's
  NewCounter (Package.pm), so 2303.08726's cases print "Case 1, Case 2" where the PDF prints the enclosing lemma's
  "Case 22, Case 22" (repro class-bindings/llncs_alias_newcounter; FIXED 77v: `\newcounter` keeps an existing
  `\the<ctr>` with LaTeX's warning, KNOWN_PERL_ERRORS #1009). Round 3 (the 68g re-review): the branches title the
  env through `\csname <env>name\endcsname` (llncs.cls:1073/1082/1096/1122; `\renewcommand\lemmaname{Lemme}` "Lemme 2"
  in title and `\autoref`, r2 title "Lemma 2"), and the alias defines aliascnt.sty :54-57's `\AC@cnt@<env>`/`\cl@<env>`
  (a paper's own `\aliascntresetthe{lemma}`: no Error); guards `perfect_kernel_batch68g::{llncs_renamed_theorem_title,
  llncs_aliascntresetthe}`.
- **68a** (binding gaps G2, journal classes B: bindgaps/g23_macros.tsv rows aa, acmart, aastex631/aastex63, mnras,
  llncs, jfm, tac, aamas; each macro checked against TeX Live's class or the witnesses' shipped copies, 1,035 witness
  papers unpacked: a name defined by at least a version of the class went in, a one-line definition only some authors
  added to their copy did not). Per binding (commands, real file:line, witnesses): aa (aa_cls.rs, aa_support_sty.rs) —
  `{theorem}` (2509.20613's v9.1 :1556; 1601.05309, 1805.03673; Perl lacks it too, KNOWN_PERL_ERRORS #740), `\arxiv`
  with `\arxivprefixe`/`\arxivprefixesep` (:2058-2062; 1604.01307, 1809.05581, 2103.09505; #740), v7's `\if@referee`/
  `\if@onecolumn` set by the `referee`/`onecolumn` options (1201.4282 :85-88; 1208.0304, 1501.05323, 1608.08833), the
  language editor's `\LEt` on the `lenote` counter (:165-173, provided at begin-document; 1607.03898), `\topics` as a
  frontmatter note (:837; 1810.01624), `\etalname`, `\VSpaceAfterTabCaption` (2504.01061), `\yearCop` (2609.03060), and
  the A&A language editor's v9.1 copy's five journals `\adndt`, `\jpb`, `\jpcrd`, `\raa`, `\zphys` (7 identical copies,
  2008.09582 :1902-1961), provide-only. acmart (acmart_cls.rs, TL 2025 v2.16) — environ loaded where styles are read raw
  (:288; 121 of 128 copies; #741), `\acmConference@shortname`/`@name`/`@date`/`@venue` kept by `\acmConference` with the
  class's placeholder conference (:1582-1600; 30 papers: 2604.15990, 2604.21043), the `\@received` history (:1859-1872;
  2210.01206), `\@subtitle` (:1611-1612; 2608.16886), `\ACM@format`/`\ACM@format@nr` and by them `\@titlefont`,
  `\@authorfont`, `\@affiliationfont`, `\@proofnamefont`, `\@proofindent` (:51-56, :2515-2539, :2659-2687, :3490-3508;
  2006.07812, 2201.00614, 1810.00952, 2608.06906), `\acksname` (:3570; 2208.13741), the title-block registers
  `\author@bx@sep`, `\author@bx@wd`, `\num@authorgroups`, `\@ACM@title@width`, `\mktitle@bx` (:1613-1614, :1767, :2225,
  :2758-2761; 2606.30549, 2607.17095, 2608.24300, 2608.20335). aastex63/aastex631 (aas_support_sty.rs, aastex_cls.rs;
  2107.07519's aastex631.cls, 1907.08642's aastex63.cls) — `\ifsuppressAffiliations` (:7324-7325; 39 papers, 2108.09109,
  2001.00952), the edit switches' flags `\ifturnoffedit`/`one`/`two`/`three` (:5790-5809; 2109.04933, 2609.30834),
  `\authorcomment` (:5843-5850, honouring them; 2004.09658, 2205.09859, 2303.08830: `\turnoffedit` drops the notes, as
  the PDF does), `\resetapptablenumbers` (:7210-7216; 2308.05494), `\currtabletypesize` kept by `\tabletypesize`
  (:1928-1932; 2202.09386), `\savetablenum`, `\tablereferences`, `\deluxetablecaption`, `\tabcaption` (:1926, :5391,
  :4780, :1609-1611; 1910.11401, 2403.12118, 2408.17407), `\onecolfootnote` (:7316), `\BeforeBegin` (:581; 2111.03992),
  `\mdash` (:7141), `\apjsecfont` (:219), `\ackbox` (:1205), `\ifnumlines`, `\ifpreprint`/ `\ifpreprinttwo` set by their
  options (:167-171, :253; 2301.00027, 2608.12156), `\aspconf` provide-only in the class binding (:6164; 2307.09521;
  aas_macros_sty.rs untouched). mnras (mnras_cls.rs, TL 2025 v3.2) — `\doi`, `\eprint` with `\@eprint@arXiv`/`@dblp`,
  `\mniiiauthor` (:1399-1479; 1609.03154, 2008.09832, 1709.09375, 1507.08002; #742), v3.0's `\apspr` provide-only
  (2404.04493). llncs (llncs_cls.rs, TL 2025 v2.26) — `\if@envcntreset`/`@envcntsame`/ `@envcntsect`/`\if@runhead` set
  by their options (:25-36; pscproc2.sty:19, 0905.3107, 2303.08726), `\@spothm` and `\spn@wtheorem` by the numbering
  option (:1086-1097, :1155-1167; 0906.4121, 1003.2851, 2202.10090; #743), the language switches (:79-197; 0908.0239,
  1812.03939; `\switcht@albion` restores the binding's own names, OXIDIZED_DESIGN_DIVERGENCES #613), `\@subtitle`,
  `\institutename` (empty: the institutes are the frontmatter's; 2112.05140), `\phantomsection` (:561; 1709.06172). jfm
  (jfm_cls.rs; 1702.03412's copy) — the caption-legend symbols `\fullcirc`…`\full` and `\rmGamma`…`\rmOmega`
  provide-only (:1380-1405, :1419-1429; 1806.08341, 2004.10578), `\thesubequation` and `\returnthesubequation`
  (:641-649; cond-mat0107563), `\useAMSsubequations`, `\norule`, `\followon`, `\fulloutfootnote`, `\indexsize`,
  `\boldit`, `\halflineskip` (2304.12657). tac (tac_cls.rs; 2401.05974's 2023 copy, 0708.1921's 2015 copy) —
  `\mathrmdef`, `\mathbfdef`, `\mathopdef`, `\mathsfdef`, `\mathcaldef`, … with `\xspace` where the copy loads xspace
  (:651-678; 45 papers: 0709.0837, 0711.0674), `\refs`/`{refs}`, `{references}`/`{references*}` as bibliographies
  (:545-578; 0812.1674, 0902.2554, 1202.6034), `\dedication` a frontmatter note (:311; 0812.1674), `\cauthor`,
  `\endproofbox`, `\proofof`, `\state`/`\staterm`/`\thirdleveltheorems` (:148-163; 1106.1983), `{remarks}`/`{examples}`
  at begin-document where the paper has none (:466-470; 0907.2460; a paper's own `\newtheoremrm{examples}…[section]`
  replaces them, as tac.cls's `\def`-based `\newtheorem` lets it: 2407.21129). aamas (aamas_cls.rs; 2602.10598's copy) —
  `\doi{SUFFIX}`: `\@doi` = 10.65109/SUFFIX, a DOI pubnote from the preamble (:1435; 65 papers: 2602.03799), defined
  only where the paper's copy defines it or none is shipped (r2: the 2021 v1.78 copies of 2510.21200, 2602.05888 and
  2608.28553's commented-out lines have none, and a reference list's own `\providecommand{\doi}` prints). Repros
  class-bindings/{aa_class_commands_v7_v91, acmart_conference_parts_and_title_fonts, aastex63x_table_and_edit_commands,
  mnras_reference_links, llncs_class_internals, jfm_legend_symbols_and_subequations, tac_class_commands, aamas_doi} RED
  (67int6) → GREEN; guards `perfect_kernel_batch68a` (8). A/B vs 67int6-rel over the 1,035 witnesses + 100 pop67o:
  witnesses' Errors 1,993 → 964, error-free 56 → 536, 504 papers with fewer Errors, none with more, Fatals 0 → 0; the
  310 with fewer words lose error-recovery text (`\Hom` → Hom, the `\mathrmdef` arguments), `\mniiiauthor`'s second
  branch (1507.08002, 1806.09815), the notes `\turnoffedit` drops (2205.09859, 2303.08830, as their PDFs), a DOI read as
  one word, appendix tables renumbered by `\resetapptablenumbers` (2601.08948 B5 → B1), 1608.08833's one-column equation
  forms (its `[onecolumn]`); pop67o 100: Errors 402 → 401, Fatals 5 → 5. Binding nets vs 67int6 (aa, aamas, aastex,
  aastex62, aastex63, aastex631, acmart, jfm, llncs, mnras, tac, emulateapj, aas_macros): 0 worse in all; Errors 177 →
  78. Not taken (pending the loader-rule decision: one-line definitions only some authors added to their copy): aa's
  journals `\jaavso` 14, `\rnaas` 8, `\maps` 7, `\lrsp` 6, `\pnas`, `\aspc` (108 names in 187 uses); aastex's
  `\acceptjournal` 30 (one author's copies, "Added by Bin Ren"), `\plotthree`, the heliophysics journals `\frass`,
  `\jgra`, `\angeo`…; mnras's `\psj` 13, `\mnrasl` 6, `\kms`; acmart's `{remark}` 11, a variant's `\acmshortauthors`/
  `\acmshorttitle` 6 (2603.20062, 2609.01519; a copy without environ), `\authorwithoutinstuition` 5; llncs's
  `\etal`/`\ie`/`\eg`; jfm's `\Nu`, `\Ma`, `\Ri`, `\Da`; fairmeta's whole row (per-variant `\code`/`\project` with
  conflicting labels, `\authoremails`, `\github`; its `\addtolist` internals). Not this payload's: `\@makecaption` (aa
  12, mnras 5 papers: xtab's `\topcaption`, papers' own caption code; no class defines it in LaTeXML — an article/
  kernel gap). Not ported (main-session ruling 2026-10-16): acmart's title-block procedures `\@mktitle`, `\@mkauthors`, `\@printendtopmatter`,
  `\@mkauthorsaddresses`, `\@@authornotemark` (2-4 papers each, supplementary title pages: 2609.39019, 2605.14703), the
  one-paper storage macros `\@acmDOI`, `\@acmVolume`…; the old aamas20xx/aa-family classes the binding rows also list
  (aamas2017.cls `\numberofauthors`, aadebug.cls `\corr`…). Residuals: AASTeX's `\edit<n>` prints its text whatever
  `\turnoffedit<n>` says (the class drops it; kept by ruling 2026-10-16); llncs's default theorems ignore
  `envcountsame`/`envcountsect` (numbered within the section on their own counters; KNOWN_PERL_ERRORS #743, both
  engines; ruled to its own payload 68g, which flips the llncs guard's "Fact 0.2"/"Assumption 0.3"); the llncs language
  switch's divergence OXIDIZED_DESIGN_DIVERGENCES #613 accepted (ruling 2026-10-16); tac's theorems stay theorem
  environments (the class's are `\subsection` headings). Side finding (pre-existing; RED row, ruling 2026-10-16): a paper's
  `\newtheorem{proposition}{Proposition}[section]` over a `\newtheorem{proposition}[theorem]…` (a binding's, or the
  paper's own after `\let\proposition\relax`) numbers from a counter that reads 0 (tac's 2407.21129: "Proposition
  2.0"; pdflatex "Proposition 1.1" on the article repro macro-state/newtheorem_over_shared_counter_theorem; FIXED 77v:
  the shared counter's `counter_for_type` mapping outlived the theorem's own counter, KNOWN_PERL_ERRORS #1008). Round 2 (review of 68a): aa's `\arxiv`, `\arxivprefixe`, `\arxivprefixesep` and `\etalname` (v8.2+
  only: 14 and 13 of 139 copies lack them, 1201.4282's v7.0) and aas_support's AASTeX 6.2+ commands (`\mdash`,
  `\tabcaption`, `\BeforeBegin`, `\authorcomment`, `\apjsecfont`, `\onecolfootnote`, `\resetapptablenumbers`,
  `\savetablenum`, `\tablereferences`, `\deluxetablecaption`; none in AASTeX 5.2's aastex.cls, 1001.0026) are
  fallback guesses (`DefinitionOrigin::Fallback`), so a paper's own `\newcommand` of them wins (it was ignored, an
  Info); `\authorcomment`'s label follows the paper's copy, else the class name ("(Author~note: …)" in aastex62/63,
  2205.09859 :5731; numbered in aastex631 and the aastex63 copies of 2312.04630, 2409.02158, 2504.08889, 2512.11572);
  tac's `\dedication` is a pubnote (`\lx@add@pubnote[role=dedication]`, as aa's and svjour's). Repros class-bindings/
  {aamas_doi_older_copy, aamas_doi_2026_copy, aa_class_newer_commands_yield, aastex5_newer_commands_yield,
  aastex63_authorcomment_label, aastex63_copy_numbered_authorcomment}; guards `perfect_kernel_batch68a` (13). Not
  guarded the same way: the registers aa v9.1 adds (`\VSpaceAfterTabCaption`, `\yearCop`) and AASTeX's `\ackbox`,
  `\ifnumlines` (an allocation does not yield to a paper's own; no witness).
- **67w** (binding gaps G2: the Wiley class family; bindgaps/g23_macros.tsv rows WileyNJD-v2, WileyMSP-template,
  WileyASNA-v1, WileyNJDv5, and WileyNJD-v1 through the same binding). One family binding
  (`wileynjd_cls::wiley_njd_family`, serving WileyNJD-v1/-v2, WileyNJDv5, wileyNJDv5 and, no longer through a
  `LoadClass{WileyNJD-v1}`, WileyASNA-v1), the copy the paper ships deciding where versions conflict (box
  environments, theorem set and styles, subfig, amsrefs, apacite), else the class name (`version_from_text`, unit
  test `wileynjd_cls::tests::wiley_version_from_text_batch67w`). Line numbers are those of 1712.08341's
  WileyNJD-v2.cls (312 shipped copies, same bodies), 2302.02310's WileyNJDv5.cls, 1804.11014's WileyASNA-v1.cls,
  1511.08547's WileyNJD-v1.cls. `\presentaddress` (v2:1781, v5:5066, ASNA:1195; a present-address note; 156+68+38 papers
  of run 353: 1701.02349, 1705.09846, 1712.08341); `\abstract[2][]` (v2:1857, every copy; `\abstract[Summary]{…}` names
  the abstract, the kernel's `\abstract` read "[Summary]" as text in 219 of 490 sampled papers; 1701.02349); the 47
  theorems in the class's three amsthm styles and its redefined `plain` (v2:2902-3030, v5, ASNA; v1:4481-4568 its 20,
  with `{example}`), the styles as the paper's copy defines them (`copy_theorem_styles`; without a copy the version's:
  v2 bodies upright, v5's `plain` and WBstyleone italic, ASNA's WBstyleone italic, v1's WB styles italic with
  bold heads and its `plain` upright with an italic head; 2302.02310's v5 theorems and lemmas italic, 1902.09230's own
  `{Algorithm}` upright, as pdflatex sets them), WBstylethree
  left current (v2:3005: a paper's own `\newtheorem{thm}` upright as pdflatex sets it, 1712.08341's copy; the binding
  had ended with an invented `\theoremstyle{plain}`):
  `{assumption}` 44+23 papers (1803.11411, 1808.10367), `{eexample}` (1806.09518), `{property}`, `{result}`, `{LWrule}`,
  each its own counter as in pdflatex (1807.03475's "Lemma 1, Theorem 1", formerly OmniBus's "Theorem 2.2");
  `{tablenotes}`/`\tnote` (v2:2816-2828; 1808.05540, 1808.09739, 1910.13464); `{biography}{photo}{text}` (v2:3445,
  IEEEtran's float shape; 1705.00674, 1812.10266); `{ack}[title]` (v2:3393, named acknowledgements; 2205.06045,
  2412.02681); `{boxtext}` (v2:466, ASNA:269; v5:592 `[1]`) and `{boxwithhead}` (v5:601) as framed.sty `{framed}` blocks
  (1811.09505, 2309.00837, 2604.17508); `\bmsection`/`\bmsubsection`/`\bmsubsubsection` (v5:5762-5764, `\section` as it
  is when called: star, optional title and appendix numbering; the former `\bmsection{}` read `*` as the title,
  2405.03603, 2511.08532); `\doiheadtext` (v2:1820, a DOI note; 1803.06134); the page-furniture stores
  `\historydates`, `\fundinginfohead(text)`, `\artid`, `\journalname`/`\journallogo`/`\jurl` (v1 of 2020), `\uns`
  (v2:1180, `\thinspace` out of math for the class's `$\,$`, which ends the paragraph's `--` ligatures in the
  kernel: 2208.08472's 27 en dashes), `\subtitlefont`; `\fundingAgency`/`\fundingNumber` ("Agency, Number." in `\fundingInfo`, v2:1877; ASNA:1300
  bold and italic in running text, 70 papers: 1806.06754, 1810.06326); `\openaccess` (v1 2010.09463, a licence
  pubnote); ASNA's journal abbreviations (ASNA:2722-2788, `\aaa`: 1909.01905); the class's loads of listings and
  alltt (all versions; `\lstset`: 1802.08557), `[labelformat=empty]{subfig}` (ASNA:250, v1:256; 1511.08547) and v1's
  amsrefs (1511.08547's reference styles, :33-43, only under APA/AMS/ACS/AMA/VANCOUVER; `\bib`/`{bibdiv}`: 9
  papers; their APA style is amsrefs', apacite only for v2 and later). WileyMSP-template (wileymsp_template_cls.rs):
  `\dedication` (:34, a dedication pubnote; 158 papers, nearly all the template's empty `\dedication{}`: 2003.09099),
  `\threesubsection` (:49, "Head: " run in italic; 87 papers: 2009.06328). OXIDIZED_DESIGN_DIVERGENCES #601 (table
  notes' leading item), #602 (`\doiheadtext`), #603 (a `\section` in `{boxtext}` is the box head). Not done: v1's `\everymath{\rm}\let\text\mathrm` in theorem bodies (ruled 2026-10-15:
  left out, `UPRIGHT_MATH` — ported, it reads `A_{ij}` as one `\rm` identifier with the subscript "ij"; math in a v1
  theorem is italic where the PDF sets it upright: 1511.08547, 1707.00398, 1709.02675), v5's
  `[apaciteclassic]{NJDapacite}` (the binding loads apacite with `natbibapa`; no witness harmed), `\xleftrightarrow` (v2/v5 via amsmath's
  `\ext@arrow`, which the binding renders as →; 2211.16474, 2309.00837), one-copy names (`{Corollary}`… 2002.01818,
  `\cd`/`\ci` 2408.07575, `\E` 2407.00139, `\maketitlesup` 2510.16163: not in the class as shipped elsewhere), and
  wiley-article.cls (#212: Overleaf's own Wiley template, raw, no binding; its `\keywords` is defined inside the
  class's abstract body, which the kernel's locked `{abstract}` bypasses: 2009.12523, 2007.02789; landed in 68n without
  a binding, the K11 rule serving every class of the shape). Repros
  class-bindings/{wileynjd_v2_class_commands, wileynjdv5_box_environments, wileyasna_funding_in_text,
  wileymsp_dedication_threesubsection, wileynjd_v1_loads_amsrefs} RED (67int3) → GREEN, {wileynjd_theorem_styles,
  wileynjdv5_theorem_styles, wileynjd_uns_keeps_en_dashes, wileynjd_v1_reference_styles, wileynjd_v1_apa_is_amsrefs}
  RED (67w) → GREEN (67w r2); guards `perfect_kernel_batch67w`.

- **68q** (`\unvbox` puts a vertical box's list back; study agent_reports/2026-10-16_perf_ram_study.md (b)): tex.web
  §1110 `unpackage` splices the box's list and drops the box, so a `\lastbox` (§1080) peel loop
  `\loop \setbox\pb=\vbox{\unvbox\pb \global\setbox\lb=\lastbox}\ifvoid\lb … \repeat` takes the lines one by one and
  ends. `\unvbox`/`\unvcopy` returned the `\vbox`/`\vtop` whatsit itself (Perl `Whatsit::unlist`,
  TeX_Box.pool.ltxml:725-742), `\lastbox` took it whole and left an empty `\vbox{}` that the next round took again, for
  ever: infernce.sty's `\@adjustPremises` under every semantic.sty `\inference` (31 of the 44-paper fleet
  `oom:alloc_failed` sample are this loop; ~315 corpus-wide is an extrapolation, 31/44 × 448; KNOWN_PERL_ERRORS #840).
  tex_box.rs `vlist_of` splices into a vertical box's own list (`VBoxContents`, marked `lx@vbox@list` in
  base_utilities/boxes_lists.rs `predigest_list_in_mode`; `\unvcopy` as copies) between a pair of boundary marks
  (`\lx@splice@begin`/`\lx@splice@end`, round 2, replacing round 1's line-only `\par` mark): the end mark closes what
  the list opened and left open (the paragraph the box's end finished, §1086; a minipage's or `\parbox`'s `ltx:p`; a
  `\noindent` `ltx:para`), the list-tail primitives see through both; on the main vertical list, in a float's, note's
  or minipage's body, inside a mode-setting environment in the vbox, in `\vadjust`/`\insert` material and in math the
  box goes back whole (OXIDIZED_DESIGN_DIVERGENCES #637, awaiting R6 and a ruling). OOM sample (44, `ulimit -v
  8912896`, default `--max-memory`) 67int10b-rel → 68q: 37 killed/Fatal → 6 (31 more complete, 38 of 44, at 0.2-1.2 GB
  in 1-19 s; 29 of the 31 new completions load semantic; 1209.2208 PushbackLimit in a `table*`/`{kaprotate}` and
  hep-th0209063 IfLimit unchanged; 1410.2418, 2311.11318, 2402.17444 still out of memory, no semantic.sty; math0301370
  OOM → Timeout:Convert). A/B over 200 pop67o papers: 198 byte-identical, 2006.07243 one "Missing number" Warning fewer
  (its local amsmath copy asmath.sty `\measure@` peels the alignment), 2210.15126 nondeterministic `CODE(0x…)` text in
  both. Repros boxes-groups/unvbox_lastbox_peel_loop (p1 RED R7: no interline glue, pinned at `[1:h,2:h,3]`),
  boxes-groups/unvbox_splice_boundary (round 2: minipage, `\parbox`, minipage then `\hbox`, `\noindent` + minipage,
  `\unvcopy` twice, a splice in a splice — each one `ltx:p` in round 1), package-bindings/semantic_inference_premises,
  box_primitives_lastbox cases 5-7 and box_primitives_unpack cases 4, 7 RED → GREEN; latexml_oxide/tests/digestion/box.xml
  re-blessed (four lines in one box, not two nested blocks), alignment/arraycr_halign.xml (`tex=` one `\vbox` fewer);
  guards `perfect_kernel_batch68q::{unvbox_lastbox_peel_loop, unvbox_splice_closes_what_its_list_opened,
  semantic_inference_premises}`, `box_primitives::{lastbox, unpack}`, `perfect_kernel_batch58::floatrow_*` (4
  re-pinned: the caption spliced into floatrow's `\vtop` is `vattach="top"`, and a `\floatfoot` after it is a child of
  the caption's `<figure>` panel — the panel opens when the `\vtop`'s captured block is placed, after the marks; text
  order the PDF's; accepted by the main session 2026-10-16). Open: raw Paul Taylor `diagrams` loops (math0301370,
  `\input{taylor_diagrams}`; probe scratch_68q/wm03/d2.tex `$$\begin{diagram} A & \rTo & B \end{diagram}$$`, in both
  binaries): our `end_graf` keeps a line where TeX's paragraph is null (§1096), so the package's `\reformatmatrix`
  `\lastbox` loop keeps finding an hbox — not 68q's. 68q review side finding 5 (floatrow's
  `\floatfoot` in a plain `figure` dropped; captions-floats/floatrow_floatfoot_in_plain_figure, probes fr2 "Plain foot.",
  fr3 "TwoFoot") — FIXED int27 (floatrow_sty.rs `\lx@flrow@plainfoot`). Open RED (int12
  nets, sigma 0907.2394 errors 0 → 8): easybmat's `\@easy@buildbox` peels the `\halign`'s rows with `\lastbox` after
  `\unvbox`; our alignment is one whatsit, not TeX's row hboxes (§799), so the whole alignment is taken as a row and
  unhboxed → 2 `malformed` per BMAT; the matrix itself was LOST before 68q (the box went back whole, the loop ended at
  once) and is kept now (boxes-groups/easybmat_lastbox_alignment_rows).
  Integrated as int12 with 68p's perf items 1-4 and 6 (gates 5,596/5,596; bundle A/B vs 67int11: 298 random pop67o
  papers tallies identical; 161 fleet `oom:alloc_failed` + OOM-sample papers Fatals 46 → 15, words 6.52 M → 9.00 M, the
  9 with more Errors all Fatal → completed; nets 135/136 0 worse, sigma 0907.2394 the RED above).

- **68r** (AMS-TeX completeness: amstex.tex + amsppt.sty; run-353 class-less Error papers, attr_now/papers_now.tsv).
  (1) amsppt's `\author … \endauthor` list (amsppt.sty:322-326) is one creator a name (`\lx@add@authors@append`, as
  amsart's), and the addresses after a block naming several (:330-370) pair with that block's names in order when
  there are as many, else are its shared contacts (counted per block, amsguide.tex:547; OXIDIZED_DESIGN #624,
  KNOWN_PERL_ERRORS #819; math0002054, math0002116, math0002213); the separators between the creators are synthesized
  (chao-dyn9304004's `\&` reads "and"), a byline's "by" is the first creator's `before` (alg-geom9305006). (2) An
  AMS-TeX document keeps plain's `\+` (plain.tex:608): the LaTeX kernel seam's retraction (latex.rs) does not apply
  once `\input amstex` or amsppt.sty (not loaded as a LaTeX package) has established the format, and is undone when
  that comes after it (amstex.rs `enter_amstex_format`; math0103164, math0105110); a LaTeX document's stray `\+`
  (cond-mat0001412, also with `\usepackage{amsppt}` or `\documentstyle[amsppt]{article}`) stays one undefined error.
  (3) `\subjclassyear` (amsppt.sty:391-401, verbatim; the edition is the classification's scheme and name, none after
  `\subjclass\nofrills`, whose prefix is the author's: 0804.1078, 1107.2266; KNOWN_PERL_ERRORS #814; cs0201007,
  math0205233, math0209036) and `\transl` = `\moreref` (:1088; #815; math-ph0003008, math0003031); a `\moreref` part
  prints its text first and then its fields by the reference's kind, carried from the part before (#817, math0111124),
  a `\ref`'s text before its first field after the fields with a comma (math0703285's `\ref{CH}` was glued to the
  authors); the fields' punctuation is `\ppunbox@`'s, the first field without, a `\nofrills` field without its frills
  and the next one's punctuation (#817; math0406143's 41 "English transl.,, Soviet Math. Dokl."); `\publaddr` is printed
  (#818), and the bib ties are active `~` (were "pp.˜").
  (4) `$$\gather` without LaTeX: amsmath's `\lx@ams@start@box` lets to the kernel's `\lx@empty`, not `\@empty`
  (math9412232, math0005042, math0006111, math0009215; repro amstex_plain_gather_return_opt_arg RED → GREEN).
  (5) AMS-TeX's `\document` reuses a document element the material before it opened (KNOWN_PERL_ERRORS #816): an
  epigraph or footnote after `\endtopmatter` had replaced the title, authors and abstract (68 of 500 A/B papers got
  their frontmatter back: math0002213, hep-th9210071, cs0201007). A/B vs 67int12 over 400 cluster witnesses + 100
  pop67o: Errors 1,740 → 1,005, 314 papers clean, 0 worse; pop100 511 → 510. Per row (≤ 30 witnesses, all 247 for
  row 1): merged_creators-only 247 → 13 papers, `\+` 30 → 0, `\transl` 30 → 0, `\subjclassyear` 30 → 0,
  `\ams@return@opt@arg` 30 → 0. amsmath net: 0 of 27 worse. Open: the kernel's cautious name split keeps a names
  line with `\footnotemark` (math0211283, math0301047, math0304096, math0405036), `,\ \&\ ` separators (math0408166,
  math0410384) and a `\footnotetext` in the list (0711.1928 + 6) as one creator; `undefined:@` (88 papers: AMS-TeX's
  active `@` arrows outside `\CD`, amstex.tex:97-107 `\at@` and :1660-1702 `\atdef@`: math0004017, math0005042,
  math0009033; KNOWN_PERL_ERRORS #1018, FIXED 78b with `\halign`'s `\everycr`, KPE #1019); discounted (not in TL 2025, not shipped): bull-ppt.sty (98 papers, math9201251), plain
  `\input mn` (63 papers, astro-ph0002467). Open, before 68r (67int12 the same): `\thanks…\endthanks` inside `\author`
  is garbled ("Ann Oneartially supported." and a note "P"; TeX "Ann One and Bob Two"); AMS-TeX's `\footnote""{…}` is a
  "”" note and a paragraph (math0207223); an `\hbox`/`\vtop` author layout gives `<p>`s inside the personname
  (cond-mat9911046); bib parts end with "." where TeX prints ";" (a `\moreref` part's), and "Sci.." doubles the period
  (68r review's comparison with TeX, not root-caused). Repros amstex/{amsppt_author_list_pairs_addresses,
  amsppt_author_list_extra_address_shared, amsppt_author_blocks_pair_addresses, amsppt_author_byline,
  amstex_plain_tabbing_rows, amstex_mode_keeps_plain_tabbing, amsppt_subjclassyear, amsppt_subjclass_nofrills,
  amsppt_transl_reference, amsppt_transl_nofrills, amstex_material_before_document,
  amstex_plain_gather_return_opt_arg}; guards `perfect_kernel_batch68r` (12).
  Integrated as int14 with one review fix (an editors-only book reference printed its editors twice and lost a comma:
  amsppt.sty:1006-1010 empties the editors' box once set as authors; amstex: "J. Editor, ed., Book Four, Pub Four,
  1999."). Gates 5,630/5,630; bundle A/B vs 67int13 over 618 papers (the 400 cluster witnesses, 100 class-less Error
  papers, 150 of pop67o): Errors 2,393 → 1,622, 385 papers fewer, none more, words +267 k; nets 136/136 0 worse.
- **68s** (the title-ink dedup deletes visible content; 68p review finding 8): `maybe_dedup_leading_title_ink`
  (base_utilities/frontmatter/titlepage_layout.rs, OXIDIZED_DESIGN_DIVERGENCES #146, arXiv/html_feedback#6924, witness 2608.10928) ran at every
  `\lx@frontmatter@fallback`, i.e. at every sectioning command and at `\end{document}`, and each run removed one more
  centered display-font copy of the unprinted `\title` from anywhere before the first `<ltx:section>`: a figure's
  line, a chapter's opening block, a second printed cover (1911.09648's `Capa1`/`Capa2`). It now looks once per
  document (`lx_title_ink_dedup_done`, at the first flush that sees the structured title), and only in the
  document-level title block — paragraphs reached through para, logical-block, block, titlepage and cover-minipage
  wrappers (`is_title_block_wrapper`) before the first sectional unit, a back-matter list passed over (2010.07703) —
  pruning a wrapper it empties. A/B over 66 papers with the dedup's Info line in the A/B logs + 200 random pop67o
  (67int12-rel → 68s): 263 of 264 byte-identical, 1911.09648 +6 words (its second cover's title line, printed by
  pdflatex), 2 NOMAIN; 2608.10928 and the titlepage/minipage covers (1304.0681, 1503.03277, 2606.13892, 2010.07703)
  unchanged. Repros sectioning-frontmatter/{title_ink_dedup_float_keeps_its_line, title_ink_dedup_once_per_document,
  title_ink_dedup_not_in_chapter} RED (67int12) → GREEN; guards `perfect_kernel_batch68s`,
  `06_cluster_regressions::cluster_frontmatter_title_ink_dedup_6924`. Round 2 (title-print study stage 1; review
  findings 1, 3, 4): never in a document with an `<ltx:titlepage>` — LaTeXML-structure-xhtml.xsl hides a document
  `<ltx:title>` with a titlepage sibling (Perl :358-359), so the cover's copy is the displayed title; deleting it left
  no visible title (1304.0681, 1503.03277: 0 `<h1>`, Perl once) — and `ltx:titlepage` is no title-block wrapper; a
  non-centered `ltx:p` is entered only when it holds nothing but boxes (`is_cover_box_paragraph`), never a `\parbox`
  in prose; a wrapper with `labels` or a frame is not pruned. The dedup is now the last resort of the title rules
  (OXIDIZED_DESIGN_DIVERGENCES #146: stages 2-4 planned). A/B 67int13-rel → 68s round 2 over round 1's 266 papers + the
  review's 80: 327 of 344 byte-identical (2 NOMAIN), 17 changed, all word gains, no Error/Fatal change; 15 titlepage
  covers keep their title (1304.0681, 1503.03277, 1601.07172, 2606.13892, 1909.04773, 2012.02929, 2110.11938,
  2201.05480, 2404.13450, 2407.17528, 2408.01596, 2409.11413, 2501.10784, 2512.19775, 2609.10069: HTML title shown
  0 → 1, or as often as pdflatex prints it — 1909.04773, 2012.02929 and 2512.19775 on two pages each), and 0812.2309
  and 1911.09648 are round 1's; 2608.10928 and 2010.07703 (a cover minipage, no titlepage) unchanged, one `<h1>`.
  Repros sectioning-frontmatter/{title_ink_dedup_in_titlepage, title_ink_dedup_beside_titlepage,
  title_ink_dedup_parbox_in_prose} RED (67int13) → GREEN, title_ink_dedup_framed_cover_keeps_its_frame RED (round 1)
  → GREEN.
  Integrated as int15 with the main session's framed-wrapper rule (a framed wrapper is never searched: no empty frame
  the PDF lacks; ff5ccc41f6). Gates 5,637/5,637, inits 0/0, dumps identical; bundle A/B vs 67int14 over 494 papers
  (68s witnesses + 150 pop67o): Errors 4,514 → 4,514, Fatals 24 → 24, 0 worse, 15 papers changed — the titlepage
  covers above, each gaining its title words only; nets 136/136 0 worse.

- **68t** (title study stage 2, rule R2; agent_reports/2026-10-16_title_print_study.md): a refused printing `\title`
  called inside the document's `{titlepage}` prints there. The `{titlepage}` the document lays out sets
  `lx_titlepage_cover`; the kernel `\title` (and the bindings' own `\title[]{}`: revtex4_support, ams_support,
  iopart_support, mn2e_support) then runs the class's/document's refused `\title` at the call, output kept
  (sect05.rs `titlepage_class_call`), and still keeps the structured title (metadata, hidden by
  LaTeXML-structure-xhtml.xsl:374-375 beside the titlepage). A run that prints nothing (a store-only setter) is undone
  (assignment journal); one that stores and prints prints (round 2, probe p2; cernphprep.cls:285, ALICE/COMPASS
  preprints 1205.3963, 1205.5121, 1503.00681, 2112.00610: the CERN header and title on the cover, 0 → 1 visible); the argument's `\thanks`, `\footnote`,
  `\footnotemark`, `\label`, `\index` are gobbled in the print (the kernel's copy owns them, probe p5). Not
  `\author`/`\date` (their frontmatter copy is displayed). The print lies between two empty `_Capture_` marker
  elements; `drop_cover_title_ink` (base_utilities/frontmatter/field_ink.rs) removes what lies between them, text included, wherever the cover
  stops standing in for the document title: `\unwind@titlepage` (probe p13), `demote_stranded_titlepage` (probe p9),
  and `settle_cover_title_ink` at `\end{document}` when no top-level `ltx:titlepage` hides the title (XSLT test). The paragraph trims
  reach past an empty `_Capture_` mark (document.rs `is_empty_capture_mark`). The title-ink dedup leaves a document
  with a titlepage alone (68s round 2's rule), so the cover print stays. Witnesses 0904.2708 (cernart.cls:217), gr-qc9404034 (document `\def\title`, `\documentstyle{revtex}`):
  0 → 1 visible title; 1304.0681, 1503.03277 (dedup) 0 → 1; 1708.09335, 2506.18627, astro-ph0208566, hep-th9701175
  byte-identical HTML. OXIDIZED_DESIGN_DIVERGENCES #629 (#146 note), KNOWN_PERL_ERRORS #828. Repro
  sectioning-frontmatter/titlepage_refused_title_prints_on_the_cover RED (67int13) → GREEN; round 2 repros
  sectioning-frontmatter/{titlepage_unwound_cover_drops_the_class_title_print,
  titlepage_demoted_cover_drops_the_class_title_text, titlepage_class_title_print_gobbles_label_and_thanks,
  titlepage_storing_class_title_prints_on_the_cover} RED (68t r1) → GREEN; guards `perfect_kernel_batch68t` (10, five
  with an HTML one-visible-title assertion). Round 2 A/B (105 review papers + 4 witnesses, 67int15 vs 68t r2): 51
  identical, 58 changed, all words gained, 0 rc/E/F/W deltas; visible titles 0 → 1 in 53 (hep-th0010274's print beside
  its emptied cover, hep-th0501109, 1802.04445 among them), none lost.

- **68v** (undefined-CS binding gaps: 12 single-error clusters, run 353-365, rc67v/clusters). (1) `\symAMSb` (197
  papers; nyjm.cls 2401.06615, cedram.cls 2105.03188, myart.cls 1506.02470, fcaa-var): amsart's amsfonts test
  (amsart.cls:377-391) copied into raw classes failed — `\try@load@fontshape` was empty, `\DeclareFontShape` a no-op and
  `\fontencoding{U}` fell back to OT1 — so amsfonts never loaded; now latex.ltx's (:10605-10620, :10279-10305), and
  `\f@encoding` names the encoding `\fontencoding` was given while an unmapped one still decodes through OT1 — `U` has
  no map of its own, so a U symbol font of another family stays undecoded as before (2306.04672 `\shuffle`, stmaryrd
  `\boxslash`) (OXIDIZED_DESIGN_DIVERGENCES #645, KNOWN_PERL_ERRORS #845; repros
  fonts-nfss/{amsfonts_try_load_fontshape, u_symbol_font_without_map}). (2) `\empheqlbrace` (229 papers; 1410.0933, 1906.02657, 2210.09120): empheq's
  `overload` and its `left`/`right` keys and delimiter commands (empheq.sty:785-1066; #646, KPE #846; repros
  package-bindings/{empheq_overload_left, empheq_lbrace_empty_fence}; residual, KNOWN GAP: the math parser does not
  parse a cell that opens with the empty fence `\left\{\right.` the brace commands give — 7 warnings, pinned by
  `batch68v_empheq_lbrace_empty_fence_known_gap`; math parsing is a separate stream). (3)
  Package internals the bindings lacked: lastpage's `\lastpage@putlabel` (`\relax`, lastpagemodern.sty:84-89; 136
  papers, cta-author/IET-Conf-Paper/ECOC-author: 2209.11927, 2609.02892; KPE #847), caption's `\caption@AtBeginDocument`
  (caption3.sty:178-191; 111 papers, oasics/lipics/tgdk: 2608.11802, 2403.18812; KPE #848), color's `\color@block`
  (color.sty:155-156, laid under a coloured textpos block; 77 papers: 2305.15117, 2407.11064; KPE #849); repros
  package-bindings/{lastpage_class_putlabel, caption_atbegindocument, textpos_block_colour}. (4) `\abstractbox` (106
  papers, memo-l.cls on amsbook: 2412.15928, 2511.16188, 1512.07128): the amsbook binding now has amsbook.cls's
  internals (`amsart_class_internals`, amsbook.cls:113-114, :616, :638-649, :1090-1121), with `\@dateposted`/`\@PII`
  empty as `\@keywords` is (KPE #850; repro class-bindings/memo_l/memo_l_amsbook_internals). (5) `\csentence` +
  `\parttitle` (178 papers, bmcart.cls:716, :1508: 2305.08284, 1611.01327, 2007.07886, 2106.01700): a command a class's
  own `\figure`/`\abstract` defines for its body is made where it is called, when the kernel's environment runs instead
  (`kernel_autoload::try_refused_environment_definition`, frontmatter_stores.rs `refused_environment_definition`) —
  only code that never ran (a refused `\lx@dropped@X`, an `\X` a `\begin{X}` constructor shadows) and no definition
  inside an `\if…\fi` (OXIDIZED_DESIGN_DIVERGENCES #647, KPE #851; repros
  class-bindings/bmcart/{bmcart_environment_body_commands, environment_code_ran_not_searched}). (6)
  `\authorname` (86 papers, MastersDoctoralThesis.cls:218: 2603.21941, 2312.06150, 2011.02318) and `\collaboration@sw`
  (79 papers, revtex4 copies qm2008_abs/snp/jasatex: 0805.2253, 0806.0481, 0805.4759): a refused class setter's replay
  keeps its local definitions at the call (`\lx@replay@capture`; OXIDIZED_DESIGN_DIVERGENCES #648, KPE #852; repros
  class-bindings/{mastersdoctoralthesis/mdt_authorname, revtex4_copy/rtcopy_collaboration_switch}; 0805.2253,
  0806.0481, 0805.4759 lose only the ERROR, their creators intact: "Takeshi Osada"/"Grzegorz Wilk", "Lusaka
  Bhattacharya"/"Pradip k Roy", "Korinna Zapp"/…). Answered (77w, user ruling 2026-10-20; OD #731 rule 12): the
  qm2008_abs/snp copies `\ProvidesClass{revtex4}` take the revtex4 binding, the copy read after it — 0805.2253 and
  0806.0481 lose their 3 malformed-caption Errors and gain each author's e-mail and affiliation contacts; ltxgrid's
  `\prepdef\@begindocumenthook` in the copy registers nothing (`state::copy_provides_defers_hook_write`). (7) `\fnm` (69 papers,
  vmsta.cls:278-283: 1508.02842, 1411.4499, 1601.01109): while a frontmatter field is digested the class's own setter of
  it is searched as well (#647; repro class-bindings/vmsta/vmsta_name_parts, its class renamed vtexnames.cls in 77t so
  no binding answers it; residual: the identity `\fnm`/`\snm` glue the name parts and the initials — 1508.02842's
  creator "Yu.YuliyaMishura" (67int15 "Yu.<ERROR>YuliyaMishura") for the title page's "Yuliya Mishura": FIXED 77t, a
  vmsta binding reading the class raw gives the author content its title page's name parts, OD #789, guard
  `perfect_kernel_batch77t::vmsta_title_page_name_parts`; 71 vmsta papers; re-verified on int45a: 1508.02842 "Yuliya
  Mishura" / "Ivan Voronov", 1411.4499 clean; its e-mails and addresses, lost or body text, FIXED 78e, OD #802). (8) `\ETC` (106 papers, cimento.cls:746, shipped, not in TL: astro-ph0502331,
  astro-ph0506233, 1804.04677): a cimento binding (latexml_contrib cimento_cls.rs) reads the shipped class (or the renamed copy the prefix fallback
  answers with it, cimento_modif.cls 2203.13736) raw (OmniBus
  when there is none) and provides `\ETC` as the title page's `\unskip` (the running head's "etc." cut has no place in
  the kernel's author list); repro class-bindings/cimento/cimento_etc. Residual: cimento's `\atque` ("and" before the last author) is
  not an author separator for the kernel (hep-th0703055 now reports `Error:frontmatter:merged_creators` where `\ETC`
  had errored first). Not done here, covered by 68u's rawclasses lever
  (TL classes read raw under ar5iv, checked with `--preload=[rawclasses]latexml.sty`: 0 errors): `\recdate` (127),
  `\notypesetlogo` (33), `\citen` (144) — jpsj2.cls (TL) and ptptex.cls (TL; also `\documentstyle{ptptex}`), whose
  overcite then loads the cite binding; left: `\citen` under nic-series.cls (5) and `\documentstyle[…jltp]{article}`
  (2). Guards `perfect_kernel_batch68v` (14). Round 2 (review 68v): the `U` encoding lost the OT1 map round 1 gave it
  (it decoded shuffle's `\shuffle` as Δ, 2306.04672 — now as 67int15); the refused-environment search reads only code
  that never ran, outside conditionals; the empheq fixture split into a 0/0 one and the known-gap fence one. A/B vs
  67int15 (217 papers: 60 pop67o + 40 ams/amssymb + 17 witnesses + 98 cluster papers + 2306.04672): Errors 2,074 →
  1,961, Fatals 5 → 5, none worse; 113 cluster papers and witnesses 1 → 0, every word lost an ERROR token or empheq's
  option text; outside the clusters only 1810.05130 changed (empheq `left=` material); the review's 42 font/probe
  documents byte-identical to 67int15.
  Integrated as **int16** with 68t (rounds 1-2), the rawclasses lever (67v) and 68u (rounds 1-3), plus 5d239c757d
  (cimento.cls on the author-file stand-in list; a public doc naming a private helper as code). Gates 5,683/5,683, inits
  0/0, dumps identical. Bundle A/B vs 67int15 over 5,075 papers (the lever's 3,362, the 68v clusters' 1,427, 68t's 106
  witnesses, 200 pop67o): Errors 12,673 → 3,578, Fatals 27 → 22 (6 better, 1 worse), 3,208 papers with fewer errors,
  words 85.29 M → 85.87 M; nets 136/136 0 worse. Of 160 papers worse or losing > 1 % of words (bag-of-words classified):
  22 base inflation (undefined names, OmniBus text, empheq option text), 119 generated tokens (numbers, bib tags,
  invisible times), 13 known rows (R7 captions; confproc; the mode-frame pair; aguplus agums.sty; ptptex), 6 author
  content — open rows: **aguplus `\authoraddr`** dropped, affiliations as body text (1107.5305, 1203.5042,
  physics9702009, and 1010.3998 with its 75 "?" citations); **apa6 `\authornote`** dropped in 2406.08022 (the one apa6
  paper using it); cjs-rcs `{englishabstract}` (2608.13762, the round-3 row); 1808.01507 (nature) reaches the
  100-error Fatal one error earlier. **ptptex under the lever**: TL's ptptex.cls loads amsmath, so papers written for
  the 1996 ptptex.sty with plain `\matrix`/`\pmatrix{…\cr}` get amsmath's old-form errors (pdflatex on TL 2025 the
  same — a harness limit); hep-th9606114 reaches the 100-error Fatal (words 12,180 → 7,242), hep-th9902200 1 → 18
  errors.

- **68w** (first-error clusters `malformed:ltx:bibitem` and `unexpected:_` of run 336; study
  agent_reports/2026-10-16_rc_underscore_bibitem.md). B1 — a heading inside the author's `{thebibliography}`
  (`\section*{References}` right after `\begin{thebibliography}`, mostly revtex4/4-1; topical `\section*`/`\subsection*`
  groups; a beamer frame): the heading's unit closed the bibliography while `\if@lx@inbibliography` stayed true, so every
  later `\bibitem` was an Error in the unit (Perl the same, KNOWN_PERL_ERRORS #855). sect11.rs `\lx@bibliography@sectioned`
  (on `\@startsection@hook`; beamer_cls.rs `{frame}` digests it too) ends the bibliography state; the next `\bibitem`
  reopens one in the unit (`\lx@mung@bibliography`, `continued`), which takes the heading, numbers on and is closed by
  the author's `\end{thebibliography}`; the empty first bibliography goes when its heading is the author's
  (OXIDIZED_DESIGN_DIVERGENCES #653); natbib's lonely check skips these entries. Study sample 27/27 papers → 0 errors
  (1702.06834 59 → 0, 2402.15720 115 → 0, 2008.08910 96 → 0, 2307.13534 73 → 0, hep-th0109182 72 → 0, 1301.0681 10 → 0),
  words −1 each (the duplicate heading). Repros index-bib/heading_inside_thebibliography{,_topical,_revtex,_frame};
  guards `perfect_kernel_batch68w::heading_inside_thebibliography_*`.
  A1 — lipics (latexml_contrib lipics_cls.rs): the five-part `\author{name}{affiliations}{email}{orcid}{funding}`
  (lipics-v2019.cls:660, lipics-v2021.cls:766-800, the 2018 copy :636; the 2016 `lipics.cls` keeps LaTeX's) through the
  frontmatter API — affiliations split on `\and`, `\and \url{…}` the homepage, email and ORCID Semiverbatim, funding a
  "name: funding" note, the `anonymous` option — and `\relatedversiondetails`/`\supplementdetails` (:910-935, :941-985)
  with their keys and their URL a link (KNOWN_PERL_ERRORS #856). 93 lipics papers of the cluster: 64 → 0 errors, none
  worse; words lost are the ORCID URLs and "and" separators now in links/creators (2202.05360, 2302.05810) and URLs
  shown as their `linktext` (2112.14457). The `_` cluster's lipics share is ~94 papers, not the study's ~300. Repros
  class-bindings/lipics_{author_five_parts,anonymous}; guards `perfect_kernel_batch68w::lipics_*`.
  A2 — thmtools' key=value theorem option (thm-kv.sty:434-437; `label` :485, `name` :488, `continues` :511-516):
  latex_constructs `thmtools_use_keys` in the native theorem's properties (thmtools_sty.rs sets
  `thmtools_thmuse_keys`, provides `\thmcontinues`): no "(label=…)" title, the label set after the head, a continued
  theorem numbered by `\ref` to the one it continues, its counter not stepped (KNOWN_PERL_ERRORS #857). 130 papers of
  the cluster writing `[label=`/`[name=`/`[continues=` (many are enumitem's `\begin{enumerate}[label=…]`, not thmtools):
  27 → 0 errors, none worse; words lost are the key text (2201.10483 −61) and the later numbers a continuation no
  longer steps (2103.08319). thm-restate's `restate=` key (2108.11527): FIXED 77v (KNOWN_PERL_ERRORS #1006). Repro
  package-bindings/thmtools_keyval_optional_argument; guard `perfect_kernel_batch68w::thmtools_keyval_*`.
  One-liners (one witness each, all → 0 errors): apacite's short family `\shortcite`/`\shortciteA`/`\shortciteNP`/
  `\shortciteauthor` take `\cite`'s `<pre-note>` (apacite.sty:277-401; 1805.05444 23 → 0); `\DeclareGraphicsRule` a
  macro typesetting nothing (graphics.sty:150; 1605.08743; KNOWN_PERL_ERRORS #858); dcolumn's `$` does nothing only at
  the `D` cell's level and is a math shift again in a box the cell holds, its saved `$` the real meaning
  (`\lx@DC@dollar`; 2306.05928 24 → 0; #859); arxbj loads elsart_support without elsart.cls's theorems
  (`lx_elsart_class_theorems_off`; arxbj.cls defines `{thm}` only; 1601.03906 97 → 0, words 1,828 → 22,549); quantumarticle's
  natbib `\doi` (quantumarticle.cls:1396-1400; 1902.03164; #860); sn-jnl without `pdflatex` loads breakurl and lets `\href`
  be `\burlalt` (sn-jnl.cls:1462-1467; 2401.05832), breakurl's `\burlalt` now hyperref's `\hyper@linkurl{\Hurl{#1}}{#2}`
  (breakurl.sty:50-51), not `\href`, which a class's `\let` made a loop. Repros
  package-bindings/{apacite_shortcite_prenote, declaregraphicsrule_typesets_nothing, dcolumn_box_in_cell_math_shift},
  class-bindings/{arxbj_author_command_over_elsart_theorem, quantumarticle_doi_detokenized, sn_jnl_href_is_burlalt};
  guards `perfect_kernel_batch68w::{apacite_*, declaregraphicsrule_*, dcolumn_*, arxbj_*, quantumarticle_*, sn_jnl_*}`.

- **68x** (raw-class frontmatter rows from the int16 A/B). (1) K11 for every class read raw: `input_definitions`
  (latexml_core/src/binding/content.rs) fires the title-page store reroutes at the end of a class load for the class a
  binding read raw (`UNCLAIMED_RAW_CLASS_READS`, claimed by the nearest enclosing class load), after the binding, whose
  own setters `store_setters::mark_bound` keeps; the explicit `\lx@class@loaded@raw` calls in apa6_cls.rs (apa, apa6),
  beilstein_cls.rs and lni_cls.rs are gone. Output identical to 67int16 on the four bindings' witnesses (2102.04789,
  1710.04173, 1811.01821, 2011.06172, 1401.2054, 1402.6720, 1605.09288, 1609.04156, 1203.0762, 2608.21389) and 20
  apa6/apa/lni/beilstein papers from rawcls3/list3362.tsv; not for every binding that reads its class raw (~25: memoir,
  KOMA, apa7, jinst, compositio, tlp, …): apa7 gains its `\shorttitle` as `<toctitle>` (2609.00670, 2609.02899,
  2609.12869, 2609.19448), and tlp (1209.2617) had its class placeholders made pubnotes and its date line cut, fixed in
  round 2 (OXIDIZED_DESIGN_DIVERGENCES #662: a class default is handed only when its title page or a page style prints it; a store the
  title page prints in one line with a value the frontmatter does not carry stays the line's; RULED 2026-10-22 and fixed in
  79a: only printed text is a value and `\par`/`\\` end the line, repros class-bindings/title_page_typesetting/* GREEN). Repros
  class-bindings/{lni_subtitle_through_k11, tlp/tlp_placeholders_and_date_line}; guards
  `perfect_kernel_batch68x::{a_class_a_binding_reads_raw_reroutes_its_stores,
  tlp_placeholders_stay_and_its_date_line_is_whole}`. (2) aguplus (OXIDIZED_DESIGN_DIVERGENCES #659): a
  binding over the raw class (latexml_contrib aguplus_cls.rs) — natbib's binding with `square` for the class's natbib
  7.0 copy (aguplus.cls:861-1173), the AGUTeX title page as the AAS binding's (`\affil`, `\altaffilmark`/`\altaffiltext`,
  dates, `\slugcomment`), `\authoraddr` (agupp.sty:154-158, printed by the class after the references) placed with the
  author it spells; the refused `\author` not replayed over the kernel's `\@title`. The 9 list3362 aguplus papers vs
  67int16: errors 11 → 5 (0811.3118's 5: its agums.sty is not shipped and not in TeX Live, the class's own
  `\ClassError`, pdflatex the same), warnings 201 → 44, words 79,932 → 81,614 (astro-ph0205369 −13: the duplicated title text the
  class's box left in the body); 1010.3998 the 75 "?" citations → natbib's, 104 → 1 warnings. Open: a mark after a
  comma and a space (`\author{S. G. Claudepierre, \altaffilmark{1,2} S. R. Elkington, …}`, 1010.3998) goes to the
  next name (the kernel's author split). Repros class-bindings/aguplus/*; guards
  `perfect_kernel_batch68x::{aguplus_authors_carry_affiliations_and_addresses,
  aguplus_altaffiliations_and_addresses_go_to_their_authors}`. (3) apa6/apa notes: `\note` (apa6.cls:776, apa.cls:459 →
  `\@note`, printed under the affiliations) and apa6's `\authornote` (:779 → `\@acks`, under "Author Note") are
  frontmatter `ltx:note`s (roles note/authornote) as apa7's binding makes them, not under `mask` (apa6_cls.rs
  `define_notes`); `\note` is no K11 table name and K11's `\authornote` ignored the mask. 2406.08022 (its author note is
  a `\note`) 0 → 1 note. Repro class-bindings/apa6_note_and_authornote; guard
  `perfect_kernel_batch68x::apa6_note_and_authornote_are_frontmatter_notes`. (4) cjs-rcs-article (OXIDIZED_DESIGN_DIVERGENCES #660): a binding over
  the raw class (cjs_rcs_article_cls.rs) — the environ-collected `{englishabstract}`/`{frenchabstract}`/`{keywords}`
  (cls:379-386) handed to the frontmatter: two abstracts with `xml:lang` and babel's heading ("Résumé"), the keywords'
  items joined by "; " and ended by "." (:387-393, `after={.}`), `{classification}` (:394-404) the frontmatter's
  classification named by `\CJS@classifname` ("MSC2020" by default; round 2); and babel_support's `\iflanguage` expandable for a known language (babel.sty:562-568;
  KNOWN_PERL_ERRORS #861: Perl's `\edef\@@@@lang` errs inside cls:368's `\edef`). 2608.13762: 4 → 3 errors (the 3 left
  are body math), abstracts 0 → 2, keywords 0 → 1. Repros class-bindings/cjs_rcs_article_abstracts_keywords,
  babel-lang/iflanguage_expands_in_edef; guards `perfect_kernel_batch68x::{cjs_rcs_abstracts_and_keywords_are_frontmatter,
  babel_iflanguage_expands_in_an_edef}`. (5) smfart and projlib-author (OXIDIZED_DESIGN_DIVERGENCES #661):
  bindings over the raw files — smfart.cls:568-583 `{altabstract}` the second abstract (other language, the class's
  caption, `\alttitle` leading its text; a same-name abstract in another `xml:lang` no longer replaces the first,
  `frontmatter_clear_same_name`; 2506.20120 0 → 1); projlib-author.sty:98-139's provided `\institute`/`\address`/
  `\curraddr`/`\email` the author's contacts (2609.02711: the Nantes address 0 → 1). Repros
  class-bindings/{smfart_altabstract, projlib_author_address}; guards
  `perfect_kernel_batch68x::{smfart_altabstract_is_a_second_abstract, projlib_author_lines_are_the_authors_contacts}`. (6) mathtools internals (the row "24 papers now
  stop at mathtools internals"; 43 run-365 papers in four shapes): the binding defines, as mathtools.sty does, the
  `show_only_refs`/`show_manual_tags` booleans and `\MT_showonlyrefs_true:`/`false:` (:252-291; witharrows.sty:1119-1124
  tests them — 13 papers: 2012.15617, 2206.15309, 2309.00504, 2507.07065; the switches set the boolean only, the binding
  keeps every tag), `\MT_delim_default_inner_wrappers:n` (:829-841; xfunctions.sty — 14: 1704.08847, 1609.02993,
  2310.09182), `\MT_start_cases:nnnn` (:995-1008, as `\newcases` opens `\lx@ams@cases` — 13: 1906.10268, 2011.05472,
  2310.19169; its end `\MH_end_cases:`, :1009-1013, since round 2, the `\right` after it the cases' own fence; since round 3 an
  environment ending `\MH_end_cases:\right<d>`, :1018, gives `<d>` as the cases' right fence — a copied `{rcases}`,
  bracket cases; repro package-bindings/mathtools_cases_right_fence, guard
  `perfect_kernel_batch68x::mathtools_start_cases_takes_the_right_fence_of_its_environment`),
  `\MT_matrix_begin:N`/`\MT_matrix_end:` (:1039-1043 — 5: 1905.08664, 2307.06921) and the short-intertext dimens
  (:1483-1484; 2109.05369). KNOWN_PERL_ERRORS #862. Repro package-bindings/mathtools_internals_after_internal_syntax;
  guard `perfect_kernel_batch68x::mathtools_internals_after_internal_syntax_are_defined`. (7) eulerpx `\mathscr` (OXIDIZED_DESIGN_DIVERGENCES
  #569 extended; KNOWN_PERL_ERRORS #863): a plain-locked binding a file's `\let` clears is kept aside and put back when
  the file's new meaning is refused (state.rs `let_i`); eulerpx.sty:130-132's cleared `\mathscr` is the kernel's again
  (2311.14793, 2501.10505, 2510.26876: 1 → 0 errors); since round 2 the document's own clearing `\let` too (the plain
  lock refuses its redefinition as well: `\let\mathcal\undefined \DeclareMathAlphabet{\mathcal}…` left `\mathcal`
  undefined, Perl the same, KNOWN_PERL_ERRORS #863). Repros macro-state/plain_locked_binding_cleared_by_{a_file,
  the_document}_comes_back; guards `perfect_kernel_batch68x::{a_plain_locked_binding_a_file_cleared_comes_back,
  a_plain_locked_binding_the_document_cleared_comes_back}`.

  class-bindings/{arxbj_author_command_over_elsart_theorem, quantumarticle_doi_detokenized};
  guards `perfect_kernel_batch68w::{apacite_*, declaregraphicsrule_*, dcolumn_*, arxbj_*, quantumarticle_*}`.
  Round 2 (review findings). sn-jnl keeps hyperref's `\href` (the class's `\let\href\burlalt` not made,
  OXIDIZED_DESIGN_DIVERGENCES #654; 2405.04967, 2207.12959, 2603.09685; 2401.05832's swapped order RED, row under
  "Perfect-kernel RED repros"); repro class-bindings/sn_jnl_href_keeps_hyperref_order, guard
  `perfect_kernel_batch68w::sn_jnl_href_keeps_hyperref_order_without_pdflatex`.
  B1 continuations: the entry redirection undone as `\restoring@bibitem` does (`\lx@mung@bibliography`; hep-th0109182
  [17] one entry), no `\refname` title or TOC entry for one no heading gives (`drop_own_bibliography_heading`), and the
  one bibliography's ids `bib`/`bib.bibN` (the continuation of an entry-less interrupted bibliography takes its number,
  `lx_bibliography_reuse_number`; the survivor takes the pruned one's freed id; OD #653); repros
  index-bib/heading_inside_thebibliography_{entry_paragraphs,paragraph_aside,then_second_bibliography}, guards
  `perfect_kernel_batch68w::heading_inside_thebibliography_*`.
  natbib: only an absent post-note swaps (natbib.sty.ltxml:113; `swap_pre_post`), so `\citep[e.g.][]{k}` is "(e.g.
  Rigden, 2017)"; `\cite` with any optional argument, even an empty one, is parenthetical in author-year mode
  (natbib.sty:694-701; Perl textual for `\cite[]{k}`, KNOWN_PERL_ERRORS #867). lipics' ORCID slot as
  lipics-v2021.cls:797 links it (an email there no orcid.org address, 2607.06446). Repros
  package-bindings/natbib_{empty_postnote_keeps_prenote,cite_optional_is_parenthetical},
  class-bindings/lipics_orcid_slot_as_written; guards `perfect_kernel_batch68w::{natbib_*, apacite_*, lipics_orcid_*}`.
  r3: citations delegated to natbib keep an absent note absent (`natbib_invocation`; apacite's `\cite<pre>[post]` family
  `apacite_delegate`, nmbib's `\citeall`, natbib's own `\citealt`/`\citealp`/`\NAT@citetp`, KNOWN_PERL_ERRORS #868):
  apacite's `\cite[p.~5]{a}` is "(Smith, 2001, p. 5)" (2002.00730, 2005.00039); apacite's `\cite` parenthetical by name
  (apacite.sty:259-267, OD #655). A bibliography interrupted before its first entry is read from what it has digested
  (`holds_nothing_yet`, through the enclosing box lists, core `with_box_list_levels`), not only `@bibitem`, which
  labelled and paragraph entries never step (OD #653). lipics' ORCID
  slot by its value: an email address is `\lx@add@email` (2607.06446). Repros
  package-bindings/{apacite_notes_as_read,nmbib_citeall_postnote,natbib_citealt_empty_postnote},
  index-bib/heading_inside_thebibliography_{labelled_entry,preamble_paragraph}; guards
  `perfect_kernel_batch68w::{apacite_notes_*, nmbib_citeall_*, natbib_citealt_*, heading_inside_thebibliography_after_*,
  lipics_orcid_*}`.

- **int17** (fix round on the integrated 68w r3 + 68x r3 stack). (1) apacite's citation modes (apacite.sty:71-110):
  `apaciteclassic` (default), `natbibemu`, `natbibapa`, `nocitation` declared; the classic `\cite` alone is parenthetical
  (`\if@APAC@classic@cite`, :258-267, OXIDIZED_DESIGN_DIVERGENCES #655), `natbibapa`/`nocitation` keep natbib's own
  (textual without notes), `natbibemu` sends `\cite` to `\citep`/`\citet` (:883-892); sn-jnl's `sn-apa` loads
  `[natbibapa]{apacite}` as the class does (L1683-1685). Witnesses 2307.11525, 2406.19732. Repros
  package-bindings/{apacite_citation_modes, apacite_natbibemu_cite}; guards
  `perfect_kernel_batch68w::{apacite_natbibapa_keeps_natbibs_cite, apacite_natbibemu_cite_is_citet_or_citep}`. (2) The 68x guard
  `aguplus_authors_carry_affiliations_and_addresses` pinned `\cite[][]{key1}` textual; the class's natbib 7.0 copy
  makes it parenthetical (aguplus.cls:979-984), as pdflatex prints it ("see [Able, 1990]"): expectation corrected.
  (3) uiucthesis (`perfect_kernel_batch61::uiucthesis_title_page_is_replayed`): 68x r2's line rule (#662) keeps
  `\department` in uiucthesis.cls:177 `for the degree of \@degree\ in \@dept\\`, the line whole as pdflatex prints it,
  no affiliation (68u r2's rule (a) had made it one); expectation updated (79a's refinement keeps it: `\@degree` prints
  "Doctor of Philosophy", and `\\` ends the line after `\@dept`). Pre-existing (67int16 the same): the
  replayed copyright page prints "© 1994 by ." — `\copyrightnotice` reads the captured `\@author`. (4) B1: a whatsit that makes no box (the kernel's
  `alignmentSkippable`: `\label`, a mark) before the heading is nothing (sect11.rs `carries_nothing`), so
  `\label{refs}\section*{References}` keeps `bib`/`bib.bibN` (OD #653). Repro
  index-bib/heading_inside_thebibliography_label_before; guard
  `perfect_kernel_batch68w::heading_inside_thebibliography_label_before_keeps_the_ids`.
  Integrated as **int17** = 68w (rounds 1-3) + 68x (rounds 1-3) + this fix round on 98dd7cacbf. Gates 5,724/5,724,
  cortex 10/10, inits 0/0, dumps identical. Bundle A/B vs 67int16 over 5,439 papers (the 3,356 raw-class papers K11
  now covers, the 1,283 papers the 68w/68x writers and reviewers touched, 250 fresh `unexpected:_` and 250
  `malformed:ltx:bibitem` first-error papers, 300 pop67o): Errors 40,079 → 22,436, Fatals 76 → 64 (12 better, 0
  worse), 506 papers with fewer errors and 0 with more, 0 losing > 1 % of words, words 85.96 M → 86.26 M; the
  `malformed:ltx:bibitem` sample 9,417 → 1,173 Errors. The one raw-class paper with more script errors, 2203.12624,
  reaches the 100-error Fatal on both sides — base on `\mathscr` (eulerpx, fixed by #569's lock restore), int17 on a
  later `_` (16,523 → 43,460 words). Nets 136/136 0 worse. Open rows from the bundle: sn-jnl `\href{text}{URL}` written
  for the class's DVI-mode swap (2401.05832, RED class-bindings/sn_jnl_href_swapped_order); thm-restate `restate=`
  (2108.11527, 2602.09462; FIXED 77v); topical bibliography groups over an empty "References" give the second group `bibaa`
  (hep-th0109182; decide id reuse at heading time by the prune's own criterion); an `\appendix` heading inside the
  bibliography is not in `BIBLIOGRAPHY_HEADED_UNITS` (1810.04031, 1304.6684); apacite `\citeA<e.g.>{a}` gives
  "Smith (e.g. 2001)" (pdflatex "e.g. Smith (2001)") and its pre-notes lack apacite's comma; classic apacite
  `\fullcite` undefined; `\citeA`/`\citeNP` stay defined under `natbibapa`; `\author{A, \altaffilmark{1,2} B}` gives
  the mark to the next name (1010.3998); a French smfart paper's main abstract is named "Abstract" (captions set at
  `\begin{document}`); K11's line rule (#662) is a shape test — RED class-bindings/title_page_typesetting/ (ka, kb),
  left for the frontmatter design study (agent_reports/2026-10-16_frontmatter_design_study.md, stage S3; RULED 2026-10-22,
  fixed in 79a: only printed text is a value, `\par` ends the line).

- **68y** (AAS/aa/emulateapj/iopart/mn2e binding shapes of the rc_struct2 structural first-error clusters `&`,
  `\noalign`, `^`; study agent_reports/2026-10-16_rc_struct2.md, shapes A4 A6 A7 B3 B5 B6 C1 C2 C8). Each binding now
  does what its class does (KNOWN_PERL_ERRORS #871-#878): aa `\tablefootmark`/`\tablefoottext` set the mark in text
  (aa.cls v9.1 :1672-1681; 2211.06931 2507.11600 2012.11559 2404.06316 2104.06278) and `\ion` is robust (:639/:1729;
  0801.2767); AASTeX's `\tablenotetext` mark is math (aastex701.cls :11967-11969; deluxetable_sty.rs
  `\lx@deluxe@tablenote@mark`, its attribute the mark's text so `a` still pairs with `\tablenotemark{a}`; 2503.18379,
  0803.1504); emulateapj's deluxetable is open from `\tablehead`/`\startdata` to its end, `\enddata` only ending a row
  (emulateapj.cls:1324-1345, 1430-1460, 1497; emulateapj_cls.rs; 1411.3719 1909.11243 1002.1848 1902.00588), its marks
  text, notes/comments/`\label` after the data kept for the end, the commands global (a document's own `\tablehead`
  stands, 1102.2440) and in a document's `{longtable}` its head rows and caption (1810.10130); `\decimals` is a `\noalign` in a `tabular`
  (aastex701.cls :12014, `\deluxedecimals` in a deluxetable; 1805.06850); a `D` column with arguments is dcolumn's
  (:12010, :6286-6291; 1607.06811); mn2e `usedcolumn`'s `d`/`.`/`,` (mn2e.cls :1210-1216, mn2e_cls.rs;
  astro-ph0701731, astro-ph0312263); iopart `\ms \bs \ns \es` are `\noalign` spacers (iopart.cls :935-938;
  1502.00309). 1607.06811, astro-ph0701731, astro-ph0312263 and 1502.00309 are TL-2025-discounted (class not in TeX
  Live / no `iopart12.clo`), fixed anyway. Repros class-bindings/{aa_tablefootmark_text_mark, aa_ion_robust_cell_head,
  aastex_tablenotetext_math_mark, emulateapj_deluxetable_open_from_head, aastex_decimals_noalign,
  aastex_dcolumn_D_with_arguments, mn2e_usedcolumn_column_types, iopart_row_spacers_noalign}; guards
  `perfect_kernel_batch68y` (8). A/B vs 67int17b over 267 papers (the 17 study witnesses, 60 aa, 60 AASTeX, 30
  emulateapj, 20 iopart, 20 mn2e by `\documentclass`, 60 pop67o): errors 1,733 → 1,262, Fatal 9 → 7, words +48,193;
  the 17 witnesses 290 → 0 errors, 12 emulateapj and one each of aa, AASTeX, mn2e papers → 0 (two Fatals gone:
  1108.2958, astro-ph0701730). Open: 1102.2440 copies emulateapj's longtable `\tablehead` (`\@table@not@headedfalse`,
  `\kill`, `\endfirsthead`) into its preamble, which a deluxetable here does not run (16 errors, as before).
  Round 2 (68y review): after `\tablehead`, `\startdata` ends the row in progress and makes the rows since the head
  heading rows, empty rows and rules between them dropped (emulateapj.cls:1324 `\kill…\endhead`; 1605.03270,
  2002.02307: a second head line without `\\` had run into the first data row), and a `\tablecaption` there is kept
  for the table's end; AASTeX's note mark is set in the class's `\hbox{$#1$}` (1807.05231 `\tablenotetext{$a$}`, the
  note body had been lost); a table-note mark is keyed by what it shows, one function on both sides
  (deluxetable_sty.rs `shown_mark`: `c, d`, `*`/`\ast`, `$\dagger$`/`\dagger` "†", `\rm{b}`/`b`; 1710.01715). Repros
  class-bindings/{emulateapj_startdata_ends_head, aastex_tablenote_mark_shown}; guards
  `perfect_kernel_batch68y::{emulateapj_startdata_ends_the_head, aastex_tablenote_marks_pair_by_what_they_show}`.
  Residual: an emulateapj deluxetable with neither `\tablehead` nor `\startdata` (KNOWN_PERL_ERRORS #874).
  A/B vs 67int17b over 446 papers (round 1's 267 and the review's 179: 100 aa, 100 AASTeX, 50 emulateapj,
  40 iopart, 40 mn2e, 99 pop67o, 17 study): errors 2,659 → 2,154, Fatal 13 → 10, words +68,683; emulateapj 240 → 157
  errors (1605.03270 1 → 0, 2002.02307 1 → 0), 1807.05231 back to its 3 (`\omit`) errors with the note kept; fewer
  words only in 2512.06138 (−39) and 2604.11988 (−712), both aa's text marks (`a, b`, `**`) one word where the math
  marks were several tokens (aa.cls:1672-1681); no paper gains errors.
  Round 3 (68y re-review): `shown_mark` reads only what a mark shows — a value, key-value or semiverbatim argument
  sets no character (`\hyperlink{tn5}{e}` "e", `\raisebox{1pt}{g}` "g"; were "tn5e", "1.0ptg") — and prints the
  mark's spaces collapsed (`c, d`), notes pairing ignoring spaces (latex_constructs `relocate_footnote`); with heading
  rows promoted, the head's rule is above the first data row (1605.03270 Table 1, 2002.02307, 1509.01261). OD #663
  (rows TeX prints only in the continuation head kept), #664 (a `\tablecaption` after `\tablehead` is the table's,
  after the tabular). Guards `perfect_kernel_batch68y::{emulateapj_startdata_ends_the_head,
  aastex_tablenote_marks_pair_by_what_they_show}` extended. A/B vs 68y2 over the re-review's 100 + 4 witnesses:
  tallies identical, byte diffs only the head rule in 1605.03270, 2002.02307, 1509.01261.

- **68z** (alignment and kernel shapes from the rc_struct2 study, agent_reports/2026-10-16_rc_struct2.md). (1) A1+B4
  (M5) a braced amsmath body: `\begin{align}{a&=b\\&=c}\end{align}` typesets its rows, as amsmath's
  `\collect@@body#1\end#2` strips the one enclosing group of a delimited argument (amsmath.sty:1424, tex.web §400);
  amsmath_sty.rs `\lx@ams@collect@body` after `\lx@begin@alignment` in align/gather/flalign/alignat/xalignat/
  xxalignat/multline/split (KNOWN_PERL_ERRORS #879). Witnesses 2011.06919 2211.06952 1909.04856 2101.06401. Repro
  kernel-alignment/braced_amsmath_body; guard `perfect_kernel_batch68z::braced_amsmath_body_loses_its_group`.
  (2) A5 (RUST-ONLY): a column end goes back after the v-part as the token read (gullet.rs `read_token` →
  `handle_template`, Perl Gullet.pm:310 `$token`), not its canonical `\cr`: with Cowan's tables.tex `\let\crnorm=\cr`
  and `\cr` redefined to `&\crnorm`, a v-part `\vrule`'s keyword scan read the macro's `&` ("Extra alignment tab").
  Witnesses hep-ph9612244, hep-ph9711334 (the latter pdflatex-discounted). Repro
  kernel-alignment/let_cr_ends_the_row_as_itself; guard `perfect_kernel_batch68z::a_let_cr_ends_the_row_as_itself`.
  (3) A7 kernel half (ruling: tex.web §785 `align_peek`): the cell-head look-ahead stops at a `\newcommand`
  optional-argument macro (gullet.rs `read_x_token_cell_head`, `Parameter::testopt`; its `\@protected@testopt`
  reaches `\kernel@ifnextchar`'s `\let`), so the arguments are read inside the cell (KNOWN_PERL_ERRORS #880);
  diagbox_sty.rs defines `\diagbox` by `\newcommand` as diagbox.sty:212 does; tabularray's table commands take their
  optional argument by `\@ifnextchar` so the row-start scan still runs them. The binding closures that peek by
  `\futurelet` (`\@ifnextchar`, booktabs' `\cmidrule`) still expand in the look-ahead. Witness 2406.11917. Repros
  kernel-alignment/cell_head_peek_stops_at_futurelet (with `\multicolumn`, `\noalign`, `\omit`, `\span` by
  expansion), alignment-bindings/diagbox_empty_before_tab; guard
  `perfect_kernel_batch68z::a_cell_head_newcommand_optional_macro_reads_its_arguments_in_the_cell`. aa's `\ion` is
  robust (`protected`) as aa.cls:639 `\DeclareRobustCommand` makes it (aa_support_sty.rs; Perl
  aa_support.sty.ltxml:247 plain): witness 0801.2767, repro class-bindings/aa_ion_before_tab, guard
  `perfect_kernel_batch68z::aa_ion_is_robust_at_a_cell_head`.
  (4) C3 (RUST-ONLY; Perl raw-loads tabularray): tabularray's `mode` (`math`/`imath`/`dmath`/`text`, `$`, `$$`) and
  `cmd` cell keys — from a colspec's `Q[…]`/`X[…]` options (a `\NewColumnType`'s too) and the `cells`/`rows`/
  `columns`/`row{…}`/`column{…}`/`cell{…}{…}` keys in order, with tabularray's child selectors (`n`, `a-b`, `U`-`Z`,
  `odd`/`even`, `-`) — set each cell's text as `$<style><cmd>{<text>}$` (tabularray.sty:2635-2639, :2874-2877,
  :3040-3043, :4593-4618): tabularray_sty.rs `tblr_cell_styles` and the cell hook `\lx@tblr@cell@open` (the kernel's
  `\lx@alignment@column@before` inside a styled table), which puts the mode after the column's own template tokens
  and its close before the after part (both then the cell's template, which the reversion trims), and gives a `cmd`
  the cell's text up to its end. The table's styles are taken once (globally) by its first cell, so an alignment inside
  a later cell keeps its own cells (2407.17127's paper.sty `<*` is an `\ialign`). A cell holding an environment keeps its
  text as written under a `cmd`. Witnesses 2603.14418 (`cell{3-Z}{2-7}={mode=math}`), 2407.17127 (sbnf
  `X[l,mode=math,cmd=\checkemptycase]`). Repro alignment-bindings/tabularray_cell_mode_and_cmd; guard
  `perfect_kernel_batch68z::tabularray_cells_take_their_mode_and_cmd`; `braced_quantity_tail::
  tabularray_new_column_type_expands` now bold under `cmd={\textbf}`, as pdflatex prints it.
  (5) C4: an `\omit` cell of an amsmath alignment is text, as `\multicolumn`'s since 66l (tex.web §789; the
  template's `$` is dropped): tex_tables.rs marks it (`Cell::omit_text_box`) and alignment.rs boxes its material as
  `\hbox{…}` (KNOWN_PERL_ERRORS #662). Witness 2306.14299 (1811.08044 pdflatex-discounted). Repro
  kernel-alignment/omit_cell_in_amsmath_alignment_is_text; guard
  `perfect_kernel_batch68z::an_omit_cell_of_an_amsmath_alignment_is_text`. Round 2: an `array`'s `\omit` cell too
  (latex.ltx `\@arrayclassz` `$\relax\@sharp$`; sect10.rs `math_array_bindings` `omit_is_text` →
  `math_cells_template`, tex_tables.rs `omit_cell_is_text`; IEEEeqnarraybox not, its rule rows keep their leader
  cells), `\multicolumn` keeping its re-inserted math; an `\omit` cell that ends at once (`\omit\span`, `\multispan`)
  is not boxed, so a spanned cell is one text box; the `\multicolumn` omit count (`MULTICOLUMN_OMITS`) is the
  alignment's, cleared at each cell end, so a misplaced mid-cell `\multicolumn` leaves later `\omit` cells text.
  Repros kernel-alignment/omit_cell_in_array_is_text, kernel-alignment/multicolumn_misplaced_then_omit; guards
  `perfect_kernel_batch68z::{an_omit_cell_of_an_array_is_text, a_misplaced_multicolumn_leaves_later_omit_cells_text}`.
  Round 3: the text box keeps the cell's `\hfil`/`\hfill` outside it (tex_tables.rs `hoist_omit_text_box_fills`), so
  the cell is flush right or centered as before the box (2306.14299's eight `\omit{\hfill $…$}` cells right again);
  repro kernel-alignment/omit_cell_text_box_keeps_the_cell_alignment, guard
  `perfect_kernel_batch68z::an_omit_cell_text_box_keeps_the_cell_alignment`; `pmatrix`/`cases`/IEEEeqnarraybox `\omit`
  cells still math (KNOWN_PERL_ERRORS #662 residual).
  (6) C5: a first-column `\lefteqn` followed by more of its eqnarray cell is latex.ltx:15800's `\rlap{$\displaystyle
  #1$}` inside the cell's math, still overprinting (sect07.rs `\lefteqn`; OXIDIZED_DESIGN_DIVERGENCES #556 extended):
  the 65o `\multicolumn{1}{l}` cell is text, so `\lefteqn{\widehat G}\hphantom{G}^{(r)}` erred "Script ^"; a
  `\lefteqn` that is the whole cell (or followed only by spacing, 2210.09695), and any in an `array`/amsmath, keep 65o's
  span (its goldens unchanged). Witness 2411.17699. Repro alignment/lefteqn_then_script; guard
  `perfect_kernel_batch68z::a_first_column_lefteqn_keeps_the_rest_of_its_cell_in_math`. Round 2 (the review's
  blocker): the next token is compared by meaning, through the macros it starts (`\def\nn{\nonumber}`, `\def\eeq
  {\end{eqnarray}}`), past spacing (`\lefteqn{X}\quad y^2` keeps the math): the rule in #556. Repro
  alignment/lefteqn_row_end_by_meaning; guard `perfect_kernel_batch68z::a_lefteqn_before_a_row_end_macro_is_the_whole_cell`.
  Round 3: the look-ahead (`LefteqnLookAhead`) tests the cell end before following a macro (`\def\eq{&=&}`), follows a
  robust command past its `\protect`, reads macro bodies and then the input after them (`\def\rnn{\relax
  \nonumber}`), and passes braces (`{\nonumber}`), `\relax`, `\strut`, `\mathstrut`, `\vphantom{…}` and `\hskip`/`\kern`
  with their glue (`\lefteqn{X}\hskip 1cm y^2` keeps the math); limits in #556. Witness hep-ph0607130. Repro
  alignment/lefteqn_cell_end_forms; guard `perfect_kernel_batch68z::every_lefteqn_cell_end_form_is_the_whole_cell`.
  (7) C10: `{tabu}` in math is an `array` (tabu.sty:616-618), tabu_sty.rs `\lx@tabu@start`: witness 2010.06598, repro
  alignment-bindings/tabu_in_math_is_an_array, guard `perfect_kernel_batch68z::tabu_in_math_is_an_array`. C11 (amsart
  caption `$$`, 0803.4182) and C6 (`\cite` note in display math, math0402383) are not local: RED rows in "Perfect-kernel
  RED repros" above.
  Integrated as **int18** = 68y (rounds 1-3) + 68z (rounds 1-3) on 871a6a20eb, with 0e8a0f5ec6 (one `\ion` comment
  after both batches made it robust) and 78bbc8f972 (`65_graphics::xytest` golden keeps the `\halign` row end as
  written, `\cr&\\&\crcr`, as Perl's own t/graphics/xytest.xml does). Gates 5,747/5,747, cortex 10/10, inits 0/0, dumps
  identical. Bundle A/B vs 67int17b over 3,417 papers (1,170 the 68y/68z rounds touched, 1,247 pop67o, 250 fresh
  first-error papers each of `unexpected:&`, `unexpected:\noalign`, `unexpected:^`, `malformed:ltx:section`): Errors
  26,546 → 22,014, Fatals 89 → 80 (9 better, 0 worse), 252 papers with fewer errors, 2 with more, words 62.71 M →
  62.88 M; `unexpected:^` 5,925 → 3,992, `unexpected:&` 2,435 → 1,205, `\noalign` 954 → 819, `ltx:section` unchanged.
  The 2 with more errors: 0908.4271 (12 → 13, a deluxetable inside `thebibliography`, the 68y review's existing
  bibitem-in-td class) and **2305.03559** (144 → 163, −0.8 % words): author optional-argument macros at an `align*`
  cell head (`\overrel[\leq]{\eqref{…}}{} &`, `\fillwidthof[c]{…}{…}`), where the §785 look-ahead now stops (KPE #880)
  — RED row, next alignment batch. Word losses > 1 %: 1904.00651, 2604.11988 (aa marks now text, the 68y review's
  re-tokenization). Nets 136/136 0 worse.

- **69a** (bibliography containers and small package shapes of the rc_struct2 study,
  agent_reports/2026-10-16_rc_struct2.md D1 D3 D5). (1) A bibliography begun in `{quote}`/`{quotation}`/`{verse}` (the
  AAAI template's `\begin{quote}\begin{small}\bibliography`): the quote is closed before it and dropped when empty, the
  entries keep the quote's `\small` (latex_constructs/mod.rs `close_list_containers_before_backmatter` → document.rs
  `close_quotes_blocking`, only when closing quotes alone makes the place reachable; ruling 2026-10-16, OD #677;
  2203.09193 1904.12066 1610.09996 2109.09861 2403.09260 1609.06434 astro-ph0410583 1504.06728); the environment's end
  acts on its own quote, found closed it ends only the tail's `ltx:p` (sect06.rs `quote_environment`). (2) A display
  sectioning unit (`\part`..`\subsubsection`) in a quote closes it the same way and is a real unit (main-session ruling
  2026-10-17, as the margin-list exemption; OD #678; 0712.1985); a run-in `\paragraph`/`\subparagraph` stays in the
  quote, in an `ltx:inline-sectional-block`, as pdflatex sets it in the margins (document.rs `AUTO_OPEN_BRIDGES`, ruling
  2026-10-17 round 3; 2607.27159 2505.18677 2305.11603 1809.03802 2208.14587), and a later display unit in the same
  quote still closes it (`holds_in_quote_block`; 2110.03376 1905.08876). An `\appendix` unit's heading is taken by a bibliography (sect11.rs
  `BIBLIOGRAPHY_HEADED_UNITS`; astro-ph0503283). (3) Kluwer `\normallatexbib` `{chapthebibliography}` (`\section*` +
  `\list` + `\item[]`, cargese.cls:1859-1906; kapproc, edbk): a `\bibitem` in the environment that opened the innermost
  `\list` closes that list first (sect06.rs `lx@list@currenvir`, sect11.rs `\lx@mung@bibliography` `list`), the
  bibliography taking the class's heading (KNOWN_PERL_ERRORS #887; hep-th0212109 hep-th0212193 cond-mat0112379
  cond-mat0110649 cond-mat0002157). Repros index-bib/{bibliography_in_quote, bibliography_in_verse_after_text,
  kluwer_chapthebibliography}, sectioning-frontmatter/section_in_quotation; guards `perfect_kernel_batch69a` (4). (4)
  Small package shapes (study A8 A9 A10 B7 B8 C7 C9 C12): multirow's text between struts (multirow.sty:174-177;
  2507.07731; KPE #888); colortbl's `\arrayrulecolor`/`\doublerulesepcolor` brace-delimited (colortbl.sty:156-164;
  1910.02206; KPE #889); cases.sty's options, `amsstyle` making the explanation column math (:228-233; 2508.08751; KPE
  #890); `\DeclareMathOperator{\N^1}` defines an already defined `\N` with the parameter text, an undefined one errs as
  pdflatex does (amsopn.sty:50-53, latex.ltx:1386-1389; 2301.04262, math0312019; KPE #891); mathpartir's stub splits
  premises only at brace depth 0 (mathpartir.sty:133-146; 2604.09165); `{NiceTabular}` ends in nicematrix's guard
  column, whose template errs (nicematrix.sty:2236-2243, :9660-9670; 2505.05139); a glossary entry's stored fields read
  `&` as the character (OD #679, KPE #892; 2602.09131). C7 (dcolumn `\text{$…$}`, 1302.5691) was already fixed by 68w
  and is guarded. Repros package-bindings/{multirow_text_one_argument_macro, colortbl_arrayrulecolor_unbraced,
  dcolumn_text_box_dollar, cases_amsstyle_explanation_math, mathpartir_premise_braced_rows, nicematrix_guard_column,
  glossaries_untypeset_ampersand, declaremathoperator_parameter_text, declaremathoperator_undefined_parameter_text},
  index-bib/{bibliography_in_quote_tail_paragraph, bibliography_in_quote_in_quotation,
  bibliography_in_quote_in_minipage, appendix_references_heading_quote},
  sectioning-frontmatter/{whole_paper_in_quotation, chapter_in_quotation, paragraph_in_quote,
  section_after_paragraph_in_quote, paragraph_in_quote_in_theorem, subsection_in_quote_in_proof (RED)}; guards
  `perfect_kernel_batch69a` (22).
  Residuals: `\hhline{~~~---}` rules every column (hhline binding, pre-existing; FIXED 78c, KPE #1024: per-column
  borders, re-verified on int45a); amsrefs `{bibdiv}` in a quote errs
  (its bibliography is not placed by `adjust_backmatter_element`); a quote holding a bibliography or a heading inside a
  list item, or outside the `minipage` it is in, errs as before; a display unit in a quote inside a theorem or proof
  closes the theorem/proof too (0 errors, but the heading, its text and the proof's tail leave it and the text after
  the environment joins the unit's last paragraph; 67int17b kept the structure with 1 error; OD #678; RED
  sectioning-frontmatter/subsection_in_quote_in_proof, review probes d_thm/d_proof3 with `\subsection`); `\multirow{3}{*}{\textsc{X}}` under a size change nests
  its fonts differently (2509.09853, rendering unchanged).

- **69b** (low-risk shapes from the rc_struct3 study, agent_reports/2026-10-17_rc_struct3.md). (1) B1: bm.sty's `\bm`
  leaves a script that ends its argument where it stands (bm.sty:162-217 `\bm@general`), so `\bm{G_}{(10)}`,
  `M\bm_{max}`, `\bm{(A^\top A)^{-1}A^}\top` take the script's argument after `\bm` (bm_sty.rs `bold_math`;
  KNOWN_PERL_ERRORS #897; est. 408 run-365 papers). Repro package-bindings/bm_trailing_script_takes_the_next_argument;
  guard `perfect_kernel_batch69b::bm_trailing_script_takes_the_next_argument`. (2) L1 (RUST-ONLY): the kernel's
  `\@trivlist` is locked as Perl's (latex_constructs.pool.ltxml:1673 `locked => 1`; sect06.rs), so the ACM 1.8
  family's spacing-only `\def\@trivlist` (vldb.cls:1118, sig-alternate*.cls, acm_proc_article-sp.cls, acmtrans2m.cls)
  no longer stops `\list` opening its list ("Attempt to end mode internal_vertical"; est. 252; 1212.6177, 1710.00867,
  whose vldb `\keywords` quotation residual is frontmatter). Repro
  list-structure/class_redefining_trivlist_keeps_the_list; guard
  `perfect_kernel_batch69b::class_redefining_trivlist_keeps_the_list`. (2b) With the lists open, those classes'
  `\keywords` (vldb.cls:778-782 `\if@twocolumn\section*{Keywords}\else\small\quotation\fi`) took the never-closed
  quotation branch, since our `\twocolumn` (acm_proc_article-sp.cls:1441) set no flag: `\twocolumn`/`\onecolumn` now
  set `\if@twocolumn` as latex.ltx:20430-20465 (sect05.rs; KNOWN_PERL_ERRORS #901; geometry left alone). Repro
  class-bindings/twocolumn_sets_the_twocolumn_flag; guard `perfect_kernel_batch69b::twocolumn_sets_the_twocolumn_flag`. (3) L2: `\endlist` with no list open — no
  `internal_vertical` mode frame on the stack — does nothing, as latex.ltx:15915-15930 `\endtrivlist` (sect06.rs
  `\endlx@list`; KNOWN_PERL_ERRORS #898; siuro210301.cls, lt6-proc.cls, 0710.0460 30 → 0 Errors; est. 94). Repro
  list-structure/endlist_with_no_list_open_does_nothing; guard
  `perfect_kernel_batch69b::endlist_with_no_list_open_does_nothing`. (4) E2: mdwtab's `array` enters math itself
  (mdwtab.sty:505-510 `\tab@left` = `\leavevmode\hbox\bgroup$…\vcenter\bgroup`), so mdwtab_sty.rs's
  `\lx@array@text@open` opens the inline formula without the kernel's `[c]` "Missing $ inserted"; Wooding's mathenv
  binding now loads mdwtab as mathenv.sty:53 does (est. 89; Perl, without an mdwtab binding, KNOWN_PERL_ERRORS #677).
  Repro alignment-bindings/mdwtab_array_in_text_is_a_formula; guard
  `perfect_kernel_batch69b::mdwtab_array_in_text_is_a_formula`. (5) E3: the `$` closing a display is read expanded,
  as tex.web §1197 `get_x_token` (tex_math.rs `\lx@dollar@default`; KNOWN_PERL_ERRORS #899; est. 30). Repro
  expansion-primitives/display_closing_dollar_from_a_macro; guard
  `perfect_kernel_batch69b::display_closing_dollar_from_a_macro`. (6) X1: enumitem's `\setlist[…]*{…}` appends, the
  star tested by `\@ifstar` after the names (enumitem.sty:1661-1667; enumitem_sty.rs `\setlist` → `\lx@enit@setlist`;
  KNOWN_PERL_ERRORS #900; est. 26). Repro list-structure/setlist_star_after_the_names; guard
  `perfect_kernel_batch69b::setlist_star_after_the_names`. Round 2 (review 2026-10-17_69b_review.md): (5) read the
  token after the closing `$` under the enclosing mode (tex.web §1194 pops the nest first) — reverted in int19 (the
  int19 bullet; guard now `perfect_kernel_batch69b::display_closing_dollar_enclosing_mode_residual`). (7) etoolbox's `\robustify{\cite}`
  (2210.15126, wiley-article.cls:774) set the text "CODE(0x…)" for each citation and lost the next token: the binding's
  native no-op now tests a LaTeX-robust command's inner command too (etoolbox_sty.rs; KNOWN_PERL_ERRORS #902); guard
  `perfect_kernel_batch69b::robustify_ltx_protected_native_cite`. Witness ids in comments: 2110.14271 (E3), 1807.04837
  (E2), 2303.05244 (X1), 2010.13108 and 1708.01493 (B1). Open rows: (a) L2 tests "an `internal_vertical` frame open",
  not "a list open": a stray `\endlist` in a minipage/figure still ends that frame (then `\end{minipage}` errs;
  pdflatex none; base the same) — a list frame needs a name that `{enumerate}`/`{itemize}` frames carry too (KPE
  #898). (b) "Missing $ closing display math." keeps Perl's recovery (the display stays open, the following text set
  in it) where tex.web §1197 finishes the display and reads the token again — a recovery ruling.

- **72a** (a stand-in binding's shipped copy adds its extras; OXIDIZED_DESIGN_DIVERGENCES #731; census
  agent_reports/2026-10-17_standin_extras_census.md; report agent_reports/2026-10-17_payload_72a.md, rounds 1-10; the
  int23 integration fix round agent_reports/2026-10-18_int23_fix.md). After
  a binding on `AUTHOR_FILE_STAND_INS` loads, the paper's copy is read raw (content.rs `read_stand_in_copy`) when it is
  the file the binding stands in for (`copy_is_the_stood_in_file`, r3: its `\Provides(Expl)…` names the file, the
  binding or a file the binding loads — 2308.11432's `\ProvidesExplClass{fcs}`, 2011.13564's IEEEtai.cls declaring
  IEEEtran —, r4: a declared macro read as its definition, cimart.cls:13-14, 2412.00032 —, else it defines a quarter
  of the binding's public command set, a stand-in its own code loads included, not one a file it loads requests: r4,
  OmniBus's aas_macros had swelled asme2ej's and nature-pre's sets, 2309.12017, 2102.01782, 2111.09589; a binding
  with no command set reads no undeclared copy); its definitions of names the binding owns hold only inside its own
  code (state.rs `note_copy_provides_assignment`; the binding's OmniBus base and its own `\let`s included, a `\let`
  alias of an overlaid name restored with it: astro-ph0011001's `\subjectheadings`), its extras stand, its
  `\AtEndOfClass`/`\AtEndOfPackage` code runs in its mode, its document-time hooks are not registered (decided on the
  r1 evidence), its nested loads are real loads, its own Errors stay Errors (user ruling 2026-10-17); its
  `\usepackage` lines take the dependency scan's `\ver@` test; a counter is the binding's only when the binding's own
  load numbered it (r3 main-session call: 2112.14648 numbers (2.1) as its PDF); a frontmatter setter it calls stores
  a class default that the document's own call replaces, whatever its value (r4: decided by the call — `\date{}`,
  `\author{}`, `\title{}`; the kernel fields' setters observed in `Expandable::invoke` once a copy stored a default),
  and that shows at `\maketitle` only where the copy's title page prints it (r3: atlasdoc.cls:189-190 gave two
  `<creator>`s and an unprinted `<date>` in 1805.01649, 1806.07355, 1807.02012, 1902.05892, 1904.03536; r4: the walk
  reads as K11's `ClassReaders` does, `\unless` and data conditionals handled, and a title page that runs the
  binding's `\@maketitle` prints the defaults). Binding fixes from the copies' Errors: WileyNJDv5 loads lettersp in the class's
  order (:377, before :6820's algorithms), pnas-new passes titlesec's `explicit` (:229), IEEEtran declares its
  paper-size flags (KNOWN_PERL_ERRORS #957), crckapb autoloads natbib's `\citeyear`/`\citeauthor`, a `\definecolor` of
  a driver's model (spotcolor.sty:71 `\color@spotcolor`) keeps the earlier colour, or, defined first, takes the
  model's alternate (r4: a spotcolor binding keeps `\AddSpotColor`'s CMYK; KNOWN_PERL_ERRORS #958; the six
  ieeeaccess papers), amsart's `\copyrightinfo{}{}` adds no copyright (amsart.cls:723-725; jgcc.cls:397;
  KNOWN_PERL_ERRORS #959), a package option list keeps every special whose catcode is LETTER (fcs.cls:150
  `\LoadClass[…\g__fcs_article_fontsize_tl]` under `\ExplSyntaxOn`), and (r4) the renamed natbib copies
  NJDnatbib.sty and bmvc2k_natbib.sty are stand-in bindings loading natbib's (contrib `njdnatbib_sty.rs`,
  `bmvc2k_natbib_sty.rs`): the 14 papers whose raw fork headed the references "Bibliography" with `bib.bibxN` ids
  (2412.14509, 2212.00057, 2210.06270, 1501.07873, 2311.17607, 2406.08486; 2101.06297, 2211.16474, 2411.11493,
  2206.08798, 2602.03829, 2408.07575, 2609.08425, 2506.14995) are "References" with `bib.bibN` and natbib's
  semantic cites (bmvc's numeric `[N]` as its PDF, NJD's superscripts). r5: NJDnatbib's heading by version (v5's
  "REFERENCES" under WileyNJDv5 or a copy whose `\bibsection` uses `\bmsection`; v2/MRM `\refname`) and its
  superscript postnote inside (`NATBIB_SUPER_POSTNOTE_INSIDE`); the MRM binding no longer loads natbib before
  MRM.cls:2441-2442's `[numbers,super]{NJDnatbib}` (2506.14995 superscripts). r6: a repeat `\usepackage[…]{natbib}`
  applies its options as natbib's options (`natbib.sty_keyval_setter`, Info) only after a compiled binding's own
  early load (`<file>:requested_by_binding`) or for a renamed copy's request (`natbib.sty:options_of_a_renamed_copy`):
  2112.14648, 2607.20603 (jheppub's binding now loads jheppub.sty:37's `[numbers,sort&compress]`), 2405.09762,
  1012.4730, 2604.20688, 2107.02450, 2409.07188, 2604.22515 numbered as their PDFs; after the document's world's load
  (the document, a raw class, jmlr's and acmart's class loads) it is LaTeX's "Option clash for package natbib" Error
  and the first options stay (the reviewer's nat_twice, jmlr_numbers); "chapterless" is decided by `\chapter` as the
  forks' `\@ifundefined{chapter}` (`state::class_defines_chapter`: a copy's own `\chapter`, a non-kernel one, or book/
  report — right now and after the `\chapter` batch 72b), and a copy's counter is declared for real unless the
  binding's load declared it (`\c@<ctr>`, or the `\the<ctr>` of an existing counter; OmniBus's `\thechapter` stub
  does not count): WileyNJDv5.cls:5728's `\newcounter{chapter}[part]` with no `\chapter` still heads "REFERENCES",
  2301.03442's two counter Warnings gone (ws-rv9x6.cls:399; it numbers chapter 1, sections 1 and 2 and subsection
  2.1, 1602.07982 chapter 1 and section 1.1, as their PDFs), and the kernel's `\appendix` numbers within chapters only
  where the class has `\chapter` (2211.16474
  "Appendix A"); a copy's `\newcommand` of a name the base class it
  `\LoadClass`es defines is LaTeX's "Command … already defined" Error (`state::copy_provides_text_default_clashes`;
  the reviewer's newcmd_base), a name it lacks the copy's (`\bibname` over article); ws-rv-har.sty bound as natbib (2301.03442: 28 "(?)" citations → natbib's, "References"); a copy's
  environment decided as a pair, OmniBus's theorem autoload stubs `Stub` (2412.00032 W 12 → 11, the reviewer's
  thm_copy W 1 → 0); spotcolor's `[pantone]`/`[hks]` preloads keep their CMYK; text defaults (main-session ruling):
  a parameterless text macro the binding's base class or the kernel assigned is the copy's to set (MRM.cls:3191
  `\refname`, ws-rv9x6.cls:91 `\newcommand\bibname`; natbib provides `\refname`/`\bibname`, KNOWN_PERL_ERRORS #960)
  — of the 415 A/B papers 89 changed vs r4, in float and abstract labels, headings and the fork papers' citations,
  each label kind matching its PDF on a 10-paper pdflatex sample ("Fig. 1", IEEE Access "FIGURE 1"/"INDEX TERMS",
  "TABLE 1", "Summary.", "Abstract.", "REFERENCES"); Errors and Fatals per paper as r4.
  Census A/B (the 400 `conv_sel.tsv` papers) vs 67int21, Errors kept (r4; r5 the same Errors per paper, words −7,847 (70) /
  +18,646 (62), the 8 more losses author-year labels become numbers as their PDFs): E 590 → 203, 61 papers E-better, 0
  E-worse, 0 newly Fatal; words −7,166 (64 papers) / +18,648 (64), the r4 change being the 10 census fork papers'
  natbib labels (raw `[An et al.(2021)An, Zhu, …]` tags become natbib's number/authors/year tags); instructions
  +3.4 %; the 91 census, review and r4 witnesses 0 worse. Costs (faithful loads): atlasdoc +26-33 % (its
  `\LoadClass{scrartcl}`, atlasdoc.cls:319), 2309.03819 +65 % (jgcc.cls:52's tikz), 2609.08425 +37 % (WileyNJDv5's
  unconditional loads; one Warning for its unclosed `\if@large@design`, :4748-4844, which xelatex reports too). Open: (a) aipcheck.tex:53-54 reads a pre-2020 `\fmtversion` (0806.2199; no longer read: its binding has
  no command set); (b) a shipped file of the stood-in name that is another file (hep-ph9609379's diagrams.sty,
  2411.02780's annotate-equations.sty as equations.sty) is not read at all, where TeX reads it raw; (c) extras defined
  inside an owned body stay unreachable (neurips `\TEST` 2406.07381, `\ADDRESS` 1909.08174; spconf's `\thetitle` from
  its `\title` wrapper, 2311.11306); (d) babel's `[english]` turns `~` into a plain space (base the same; babel.def's
  `~` shorthand, `\active@prefix~\active@char~`, reaches the kernel's `\lx@NBSP` as `\normal@char~` but sets a space:
  every non-breaking space of 2609.08425, whose WileyNJDv5 copy loads babel); (e) 2609.08425's WileyNJDv5.cls loads
  fontspec, xunicode, mathastext, microtype, geometry… unconditionally under its `LATO2COL` option (:670-680; a
  XeLaTeX paper: pdflatex stops at fontspec), +37 % instructions, faithful; (f) resolved in r5 (2609.08425 "REFERENCES");
  (g) resolved in r5 (ws-rv-har bound); (h) a second `\author` keeps both under
  the inst_support binding (accumulating, as for llncs-style `\author` per author; pdflatex atlasdoc keeps the last);
  (i) 2301.03442 prints "(Mayor et al., 2011)" where its PDF has "(Mayor et al. 2011)": its `\bibliographystyle{aa}`
  (brojasayala.tex:255) comes after the citations, and the natbib binding applies `\bibstyle@aa`'s punctuation only from
  there on (in TeX the style reaches the citations through the aux file's `\bibstyle`, at `\begin{document}` of the next
  run); (j) asme2ej (2309.12017, 2404.14053) heads its appendices "APPENDIX A" where the PDF prints no appendix word
  (asme2ej.cls:689's `\appendixname` is not used by its `\appendix`); (k) acmart's citations are author-year where
  acmart.cls's default `\setcitestyle` numbers them (the reviewer's acm_ay; base the same); (l) sv_support loads natbib
  for every svjour/svmult paper where svjour3.cls:67-68 loads it only under its `natbib` option (Perl the same), and
  sn-jnl's binding loads natbib optionless where sn-jnl.cls:1641-1720 loads it with the reference style's options (its
  copy's repeat load sets them); (m) the bindings revtex4_support (revtex4_support:9), achemso, apacite and mnras load
  natbib other than their real files do and do not mark the load as the file's own, so a document's repeat after them
  applies its options (the r6 reviewer's sizing over 2506: 38 subset repeats, which clash with no file; 2506.08397's
  `sort` the only non-subset) — per-binding calls, left open; (n) latex.ltx's two-`\LoadClass` Error
  (:18800, `\@twoloadclasserror` :18938-18942) is not emulated in general: a document's own second `\LoadClass` still
  loads the class (the r7 reviewer's probe scr2, `\documentclass{article}` + `\LoadClass{scrartcl}`, Fatal on 67int21
  too, typearea.sty:1400-1413 under article's `\@ptsize` 0); (o) a slot-filled copy's calls of its absent class's
  interface are undefined, Errors as the copy's own (r8, OXIDIZED_DESIGN_DIVERGENCES #736): ceurart.cls:63
  `\KOMAoptions{DIV=10}` 1 Error in each of 107 ceurart papers of 2506+2509 (0 on 67int21, which read no copy),
  atlasdoc.cls 11 (`\KOMAoptions`, `\KOMAClassName`, `\setkomafont`, `\setcapindent`, `\deffootnote`, …) in each of 14
  atlasdoc papers — resolved in r9 by option B (the binding loads the real base); open where the slot still fires:
  Perl's svmult binding loads book where svmult.cls/svmono.cls `\LoadClass{article}` (book covers article's interface;
  its numbering "0.1" against the copy's "1"), daj's binding emulates tocbase (not in TeX Live) over article; (p) a raw
  KOMA base lacked the article binding's semantic layer (r9): KOMA's `\appendix`, no id prefixes, the 2 sectioning
  Warnings — resolved in r10 for every KOMA document (OXIDIZED_DESIGN_DIVERGENCES #737); (q) ws-rv9x6 copies had
  counter-named ids from r6 on (2301.03442, 1602.07982: `chapter1`, sections `S1` not nested; 67int21 `id1.1`/`Ch0.S1`)
  — resolved with 72b r3's binding chapter counter, which the copy's `\newcounter{chapter}` keeps: `Ch1`, `Ch1.S1` (int23
  guard `perfect_kernel_batchint23::ws_rv9x6_copy_counters_keep_the_binding_ids`); (r) `atlasdoc_cls::main_class` keys
  on the copy's version, not on the class its `\LoadClass` names: a modified copy naming `article` or `book` gets its
  version's KOMA class, which the slot then stands for the copy's (72a review r7 NIT 8; open, no witness yet).
  r7: a raw file's own repeat of natbib after a binding's early load — a stand-in copy that loads natbib itself,
  neurips_2024.sty:71, icml2024.sty:95 — applies its options, joins them to the first load's and ends the early-load
  state, so the document's `\usepackage[numbers]{natbib}` after it is LaTeX's "Option clash" (the reviewer's
  neurips_rep, icml_rep; OXIDIZED_DESIGN_DIVERGENCES #733), while copies that comment the load out keep the document's
  options (2506.00573, 2506.01987, 2506.08967, 2506.17103, 2506.18071, 2506.22567 numbered); the clash test reads the
  passed options (`\opt@natbib.sty`: 2110.14346's `\PassOptionsToPackage{numbers, compress}{natbib}`, no clash);
  elsarticle (elsarticle.cls:1243), jheppub (jheppub.sty:37) and acmart under its `natbib` boolkey record their natbib
  load as the file's own. What a base class defines is told by its load frame, not its text (#732,
  `state::ClassLoadFrame`): article/report/book assign their classes.dtx names in their own load (the kernel keeps
  its predefinitions), a class loaded while a copy's `\LoadClass` is open is the copy's base class too (the reviewer's
  nc_chain 2 Errors, nc_namedef 1, nc_cond 0, as pdflatex), the r6 class-text scan and its per-thread cache are gone;
  a name made `\relax` stays definable for the copy's `\newcommand`. A text default still the macro a class load
  assigned is the class layer's, so a copy's redefinition stands: 1202.3442 "ABSTRACT" (emulateapj.cls:470),
  astro-ph0105230, astro-ph9912326, hep-ph0102191 "REFERENCES" (aipproc.sty:49), each as its PDF.
  r8: one class slot, the exception to "nested loads are real loads" (OXIDIZED_DESIGN_DIVERGENCES #736): a stand-in
  class's copy's own `\LoadClass{Y}` of a class not loaded and not shipped, after the binding loaded its base class,
  does not load Y — ceurart.cls:62's scrartcl had raw-read typearea over article (Fatal "Conditional limit of 48000000
  exceeded": 2509.11496, 2608.23214, 2310.02658, 2505.16276 now 0 Fatal, 1 Error each), atlasdoc.cls's
  `\LoadClass{\ADOC@mainclass}` (2509.01219: no KOMA Warnings; atlasdoc instructions +0.8-3.8 % vs 67int21, r3-r7
  +26-33 %); the binding's classes are the copy's base classes (newcmd_base, nc_chain 2 Errors); a shipped class is
  read. Class-layer identity holds the definition (`state::ClassLoadRecord`, `Rc::ptr_eq`); the files read raw and the
  provides-modes reset at every `Converter::prepare_session` (the reviewer's probe two, `latexml_oxide t.tex u.tex`,
  both copies read); natbib's repeat options in natbib's declaration order with `nobibstyle`/`bibstyle`
  (`[square,authoryear]` gobbles `\bibstyle`, natbib.sty:238-263; OXIDIZED_DESIGN_DIVERGENCES #733). Rows (n), (o)
  above.
  r9: a stand-in class binding loads the base class its real file loads (OXIDIZED_DESIGN_DIVERGENCES #736), so the
  copy's own `\LoadClass` is a repeat: ceurart → scrartcl, atlasdoc → its KOMA class by its options and version, lmcs,
  jgcc, czipreprint → amsart, pnas-new → extarticle, clear2025 → jmlr; OmniBus provided over a real base where it was the
  base before (`omnibus_cls::provide_over_base_class`: ceurart's `{keywords}`). The slot stays as the net (svmult, daj).
  2509.11496, 2608.23214, 2310.02658, 2505.16276 0 E; ceurart 38 → 21 E (vs 67int21) and atlasdoc 1 → 0 over 2506+2509,
  0 worse; rows (o), (p) above.
  r10: the KOMA classes get the standard classes' semantic layer (OXIDIZED_DESIGN_DIVERGENCES #737, koma_script.rs):
  the ids of article/report/book (`S1`, `S1.SS1`, `S1.E1`; `Ch1.S1` under scrreprt/scrbook; `define_counter_ids`),
  `\appendix` = `start_appendices` + KOMA's own (`<appendix>`, KOMA's lettering), and KOMA's copies of the kernel
  `\@startsection`/`\@sect` set to LaTeXML's before the raw load, so its check passes as under pdflatex (0 Warnings
  where every KOMA paper had 2); ceurart's `\title[mode=sub]` is the subtitle (19 of 112 lost their main title), and
  OmniBus declares `\if@mainmatter` with its `\mainmatter` (scrlayer's "\mainmatter defined without \if@mainmatter!",
  111 of the 112 ceurart papers under r9). Perf: the raw KOMA load is 2.2-2.4× pdflatex's instructions for the same
  class (scrartcl minimal 6.9 G vs article 0.94 G; pdflatex 4.09 G vs 1.35 G), no double load or loop: #731's costs.
  A/B (72a10 vs 72a9 / 67int21): r9's 602 papers 0 better, 0 worse, 0 new Fatal, Warnings 7342 → 6958 (the KOMA trio
  384 → 0), words +597 (41), papers with `<appendix>` 143 → 172; 100 `\documentclass{scr*}` papers (2506-2509) E 120
  = 120, 0 worse, KOMA Warnings 194 → 0, `<appendix>` in 3 → 32, counter-named ids in 97 → 0, instructions ±0.2 %.
  int23 (72a review r7): scrbook's `\frontmatter` unnumbers the chapters and `\mainmatter` numbers them
  (`koma_script::koma_book_matter`, `no_number_sections` as book_cls.rs, before KOMA's own `\if@mainmatter` code; #737
  item 4): 2506.17485 "Abstract", then "1 Introduction", as its PDF (was "Chapter 1 Abstract", every number +1);
  `\backmatter` left to KOMA, as book's was until 78a (the flag would unnumber KOMA's back-matter sections too:
  2508.01496's appendix sections, referenced by number; open: its appendix chapter "Appendix A", PDF "Appendix" — book's
  78a `no_number_chapters` would unnumber it but not reset the counters within it as KOMA ≥3.15 does, scrbook.cls
  :5590-5600, its 16 appendix figures then off by one); the
  class slot reads the classes the binding loaded from the copy's first class load, so a class a shipped intermediate
  class `\LoadClass`es (copy → mybase.cls → article under svmult's book) meets the slot too (#736,
  `state::open_class_load_classes`). int23 fix round 2 (agent_reports/2026-10-18_int23_fix2.md): a file a copy
  `\input`s (its `.clo`/`.cfg`) is part of the copy, only `\usepackage`/`\RequirePackage`/`\LoadClass` are real loads
  (#744; svjour affiliations back: 0901.1015, 1012.5412, 1111.3868, 1512.01957, 2411.18962, 2107.02119); a `.cls`/`.sty`
  copy's expl3 syntax ends with its load (fcs 2606.16349, 2308.11432); the declaration test reads the binding that
  answered (1705.03989, KNOWN_PERL_ERRORS #970); elife's `\figsupp` prints its supplements at the end (2311.00616);
  neurips copy setters `\projectpage`/`\emailaddress` are contacts (2605.09681; since 73a r4 read from the copy's
  store at the abstract's end, the binding defining neither, OD #741); mdpi loads cleveref only where its
  copy does (2111.02495, 2201.05363); WileyNJD marks NJDapacite loaded with its apacite (2606.10545, 2602.12288);
  typewriter whitespace stays out of a listing (#745; 2608.15322, 2602.03247, 2607.19926). Open: (s) siamart 1703.08345
  `hypdvips: Wrong hyperref driver` (siamart.cls:1269-1272 `\ifpdf` false under `\pdfoutput=0`, K6 ruling, while the
  hyperref driver is pdftex's); (t) elife 1907.05060 E 1 → 3 (`\format@title@figure` in a paragraph) — fixed in 73a r4 (caption's title
  takes its text `\long`); (u) aamas
  1702.08222 E 4 → 6 (`\alignauthor` tabular in `\author`); `\cal` from a copy (svjour3.cls:1145, aamas2017.cls:646)
  is `font="caligraphic"` L in the XML and ℒ in the post's MathML, as the binding's. Guards `perfect_kernel_batch72a::*`,
  `perfect_kernel_batchint23::*`; repros `stand-in-copies/`.

- **int20** (fix round for the int20 bundle A/B's 48 worse rows, 70a + 70b + latex209 on 9cc0920336;
  agent_reports/2026-10-17_payload_int20.md). (1) The plain-TeX `\halign` rows (math9611222, math9611223, math9412232,
  math0005042) were not 70b A1's template shape: diag.tex's `\newCD` calls `\Let@`, which the amsmath binding (65s)
  made `\let\\\math@cr`, so `\\` reached amsmath's `\@ifstar` (plain AMS-TeX has none) and the row's `\cr` came with
  its braces open — A1 then gave TeX's "Missing } inserted" cascade. amstex.rs defines amstex.tex:486's `\Let@`
  (`\let\\=\cr`; KNOWN_PERL_ERRORS #949): 67int19b → bundle → int20 errors 21 → 101+F → 7 (math9412232), 25 → 101+F → 9 (math9611223),
  54 → 84 → 28 (math0005042), 101+F → 40+2F → 75 (math9611222), every diagram kept (words at or above base). OD #707 stands (a bare `\halign{#…}` cell's stray `}`: pdflatex ends the alignment, §1132, 6 errors
  and the cells after it lost; 67int19b and 70b keep every cell with 0-2). (2) 70a's regressions: graphicx's
  `\lx@Gin@include` reads the name at the document's catcodes and expands it (graphicx.sty:198-203; KPE #950), so
  `\def\seq_name{…}` names match (1908.04391 101+F → 0, 2609.16620 8 → 0; 67int19b had 54 and 4); a sub-file gincltex
  boxes beside standalone has its preamble skipped (standalone_sty.rs wraps `\gincltex@boxfile`; OD #463 amended):
  2512.06733's sub-file `\usepackage{cleveref}` no longer runs in the body (8 → 0, its figure drawn: +50 words on 67int19b). (3) `\slashbox`/`\backslashbox` are
  `\def`s as diagbox.sty:216-221, so an alignment's cell-head peek stops there and their arguments are read in the cell
  (KPE #951; 1106.0505 10 → 1, its table cells kept; 67int19b 2, the one left its `\citeauthoryear`); their second optional argument is `trim=` (it was `font=`).
  Classified, no change: the 32 `clev` rows and 7 more (2210.03396 2307.04997 2401.06723 2501.07466 2508.02914
  1906.06075 2407.02300) lose only hyperref's hidden `<tag role="autoref">` words (70a B1: no hyperref loaded, as in
  pdflatex; visible bag-of-words identical, 1711.09373 −58 = its 253 autoref-tag words). Open rows: (a)
  cond-mat9806166 72 → 81: the kernel `{subequations}` with a plain `\pmatrix{d\cr s\cr}` in an `equation` breaks the
  display (67int19b 3 errors, A1 adds TeX's "Missing } inserted" at the `\cr`; trigger `\documentclass{article}`
  `\begin{subequations}\begin{equation}\psi=\pmatrix{d\cr s\cr}\end{equation}\end{subequations}`, and under
  jpsj.sty's own `\subequations` the same; not reproduced with that definition in an article preamble). (b)
  revtex4/differ.tex 8 → 9 (int20 net): docs.sty `\cmd\documentstyle` writes `\index{documentstyle=\string\cmd`
  + the `\string`ed name}; the `.idx` re-read (`reread_sanitized_entry`) turns those characters into the live
  `\documentstyle` before process_index_phrases expands `\string\cmd` (pdflatex: expand, write, and only `\printindex`
  re-reads `\cmd\documentstyle`, which `\cmd` strings), so the phrase runs `\documentstyle` in the body — the pre-existing
  Fatal "File not found: .cls"; the +1 is latex209.def's `\RequirePackage` trap on that same run (19f347283b). The fix
  is the index write/re-read order (expand before the re-read), not attempted here. (c) the amstex papers' remaining
  `\next@` "Missing argument Match:." errors (4-72 each) are base's. Repros amstex/amstex_let_row_end_is_cr,
  graphics-tikz/{graphic_name_macro_with_underscore, gincltex_subfile_preamble_skipped},
  alignment-bindings/backslashbox_reads_its_arguments_in_the_cell; guards `perfect_kernel_batchint20` (4). A/B vs
  67int20-rel over 131 papers (the 48 rows, 60 random "better" rows of results_r67int20, the 45 70a/70b witnesses): 123
  identical in errors and words, 8 changed — the 5 better above, and three that read worse against the bundle only:
  math9611222 (40 errors + 2 Fatal with 0 words → 75 with 12,366), math0005042 and 2512.06733 (words lost = the
  bundle's `<ERROR>` texts `\@eqpen`/`\@ifstar`, `\lx@cleverrefnum@@`); against 67int19b, past the 39 autoref-tag rows, none
  of the 48 is worse but cond-mat9806166 (open a). **Integration (39aec5312c, binary 67int20b):**
  - Gates: nextest 5816/5816 + cortex 10/10, clippy/rustdoc clean, inits plain=0 latex=0, dumps identical to 67int20.
  - Population A/B vs 67int19b over the 2,729 int20 papers (pop67int20.tsv, results_r67int20b): E 19,358 → 16,863, F 64 → 62, 0 newly Fatal, 203 papers E-better, 2 E-worse; words +28,024; `missing $` 2,251 → 2,152.
  - The 39 word-loss rows are the autoref-tag rows above.
  - The 2 E-worse papers are cond-mat9806166 (open a) and 2005.03021 (1 → 3). 2005.03021's base Error was the `l` argument of its own `\RenewDocumentCommand\order{ l m }{\fbraces#1{\lparen}{\rparen}…}`; physics now loads xparse, as physics.sty does, so the `l` type exists. That exposes (d): the physics binding lacks physics.sty's `\fbraces` (:158) and `\lparen`/`\rparen` (:31-32), so 2 Errors where pdflatex has 0.
  - Nets 135/136 no worse vs 67int19b; the one is revtex4/differ.tex (open b).
  - Open (d), the physics binding gap: provide `\lparen`/`\rparen` (`\ProvideDocumentCommand{}{(}`) and `\fbraces` (star, `\big`…`\Bigg` sizes, 4 arguments) as physics.sty defines them, witness 2005.03021.

- **70b** (kernel shapes: the 69c follow-up and the rc_struct4 study, agent_reports/2026-10-17_rc_struct4.md). (1) A1:
  tex.web §1127 `align_error` — a tab/`\cr`/`\span` reaching the stomach at alignment count ±1/±2 inserts the brace
  TeX supposes omitted and ends the cell after it (stomach.rs `align_error`; KNOWN_PERL_ERRORS #923), the cell frame
  the template's brace pair for every alignment (OD #707), the row-continuation macros keep their brace half
  (sect10.rs). The 69c round-2 table: `a} &=& b` 17 → 1 error, `a &=& b}` 13 → 1 with `b` kept, amsmath `\\ 6} c`
  9 → 1, tabular `{a & b}` / `a & b}` keep their first cell. Guards `perfect_kernel_batch70b::{eqnarray_stray_close_brace_inserts_a_left_brace,
  eqnarray_trailing_stray_brace_keeps_the_row, align_stray_close_brace_keeps_the_rows, tabular_stray_brace_keeps_its_cells}`.
  (2) Q1: `\quote@@name` delimited by `"` (latex.ltx:9665; sect11.rs; KNOWN_PERL_ERRORS #924) — VTeX arXiv configs'
  `\let\IfFileExists\@iffileonpath`; witnesses 1312.4083 2001.03064 2006.14814 1504.06523. Guard
  `perfect_kernel_batch70b::quote_name_is_delimited_by_the_quote`.
  (3) S1: `\documentstyle` applies latex209.def's size switch (:138-139), its `\@documentclasshook` and the 2.09
  registers/size commands/latexsym (tex_job.rs `\lx@latex@ix@definitions`; the parts not applied listed there;
  KNOWN_PERL_ERRORS #925); witnesses hep-ex0107077 3 → 0, 0802.1120 4 → 0, astro-ph0009372 2 → 0. Guard
  `perfect_kernel_batch70b::documentstyle_class_sets_normalsize_through_at_normalsize`. Open (ruling): latex209.def's
  "LaTeX2e command \usepackage in LaTeX 2.09 document" error (:79-123) is not raised.
  (4) V1: `\vsplit`'s `to` is a keyword (tex.web §1082/§407; tex_inserts.rs; KNOWN_PERL_ERRORS #926), "Missing `to'
  inserted" without it; witness 2505.21389 (cutwin). Guard `perfect_kernel_batch70b::vsplit_to_is_a_keyword`.
  (5) M2 held back: defining `\mv@<name>` (latex.ltx:13083-13090) lets chemist's `\mathversion{chem}` set mathcodes
  for its version's OT1 letters, which our decoding reads as OML (commas → `;`, parentheses → harpoons in 2404.06813,
  1707.08434, 1807.01228, 1704.03424); needs per-version symbol fonts first (KNOWN_PERL_ERRORS #927; RED
  `fonts-nfss/declared_math_version_defines_its_mv_macro`).
  (6) K1: `\opt@<file>` holds the option list without its spaces outside braces, as latex.ltx:18517/18757
  `\zap@space` stores it (content.rs `zap_space`; `\@raw@opt@` keeps them), so l3keys2e finds keys an expl3 package
  declared with spaces (`share counter` → `sharecounter`); witness 2607.15530 (pomegranate.sty); KNOWN_PERL_ERRORS #928
  (latent in Perl, which reads no raw package). Guard `perfect_kernel_batch70b::opt_macro_drops_spaces_as_zap_space`.
  (7) E4 (etna `\remark` ended by the class's `\@endtheorem`): not local, RED row in "Perfect-kernel RED repros"
  (`boxes-groups/etna_remark_ends_by_class_endtheorem`).
- **69c** (medium-risk shapes from the rc_struct3 study, agent_reports/2026-10-17_rc_struct3.md). (1) P1: a theorem,
  proof or TOC inside a container whose model holds no Para.class (`{quote}`, a `\footnote`, a `{wrapfigure}`, an
  `ltx:text`, a cell) opens an `ltx:inline-logical-block` there — the last auto-open bridge of document.rs
  `find_insertion_point_qsym`, after closing the auto-closable `ltx:p` around the point (OD #689, ruling P1; #271
  unchanged). Witnesses 2406.11811 2103.03402 0806.0214 2602.15784 2301.10544 1410.7242 1412.2039 math0603642
  2507.09788 2307.10959 2308.06338 2410.15097 0804.3724 math0612590 1310.0491. Repros
  block-model/para_level_in_{quote,note_wrapfigure}_opens_inline_logical_block; guards
  `perfect_kernel_batch69c::para_level_block_in_{quote,note_and_wrapfigure}_opens_an_inline_logical_block`. Not in
  a frontmatter container (`ltx:abstract`, `ltx:acknowledgements`, `ltx:keywords`, `ltx:classification`,
  `ltx:creator`: the "isn't allowed" Error stands until the frontmatter study is ruled on; control
  block-model/theorem_in_abstract_is_not_bridged, guard `perfect_kernel_batch69c::theorem_in_abstract_is_not_bridged`).
  (2) E1: a `$` inside a math TeX sets as `\hbox{$…$}` — amsmath's `{equation}`/`{equation*}`/`\[` under
  `\if@fleqn` (amsmath.sty:2794-2812), proof.sty's label/conclusion/premises (proof.sty:160-166), accents.sty's
  under-accent (accents.sty:206-213) — is text in that math: tex_math.rs `\lx@math@in@hbox` (`$` as in a math cell),
  run by the kernel displays' `\lx@mathdisplay@hook` (default in latex_constructs_rust_only.rs, run by sect07.rs's
  `\[` and display environments; amsmath_sty.rs under
  `\if@fleqn`), proof_sty.rs `in_hbox_math` and accents_sty.rs `\underaccent` through `\lx@math@in@hbox@group`
  (scoped to the part; control paragraph-mode/proof_infer_parts_keep_the_formulas_dollar) (KNOWN_PERL_ERRORS #907). Repros
  paragraph-mode/dollar_in_hbox_math_is_text_{fleqn_display,proof_accents} + control dollar_in_display_math_still_errs;
  guards `perfect_kernel_batch69c::{dollar_in_an_amsmath_fleqn_display_is_text,
  dollar_in_proof_and_accents_hbox_math_is_text, dollar_in_a_display_still_errs}`. fleqn.clo's `{equation}`
  without amsmath (also `\hbox{$…$}`): modelled in 73b (KNOWN_PERL_ERRORS #907).
  (3) U1: an eqnarray cell's `{`/`}` that balance only across the template's own `{##}` (latex.ltx:15760-15763) — mod.rs
  `eqnarray_columns` ends/starts its math cells with tex_box.rs `\lx@template@bgroup`/`\lx@template@egroup`, a mark on
  the cell's frame, not a group (stomach.rs `TEMPLATE_BRACE_OPEN`, `close_template_brace`): a well-formed cell is as
  before (KNOWN_PERL_ERRORS #908); an `\egroup` closes the mark as a `}` does (stomach.rs `closes_template_brace`;
  control kernel-alignment/egroup_closing_a_template_brace_errs_once, TeX's one error). Open: a cell with a net stray
  `}` (`a} &=& b`) leaves the alignment count at −1 and unravels (pdflatex 1, before 2, now 17; §1127 `align_error`
  not modelled). Witnesses 2207.06932 1912.10594 (12 → 0, 8 → 0 errors). Repro
  kernel-alignment/eqnarray_cell_braces_balance_across_the_template; guard
  `perfect_kernel_batch69c::eqnarray_cell_braces_balance_across_the_template`.
  Then amsmath's templates (amsmath.sty:1483/1576/2350/2424) in amsmath_sty.rs gather/align/aligned: only `{aligned}`
  shows the shape (`{align}`/`{gather}`/`{split}` collect their bodies first). Repros
  kernel-alignment/amsmath_cell_braces_balance_across_the_template, control
  dollar_in_a_braced_template_cell_ends_its_formula; guards
  `perfect_kernel_batch69c::{amsmath_aligned_cell_braces_balance_across_the_template,
  dollar_in_a_braced_template_cell_ends_its_formula}`.
  (4) 2305.03559 (int18's 144 → 163, `\overrel[\leq]{…}{} &` / `\fillwidthof[c]{…}{…}` at an `align*` cell head): not
  the §785 stop, which is right (TeX expands `\fillwidthof` inside the cell's math, `\ifmmode` true), but mathtools'
  `\clap` digesting its argument in math (Perl mathtools.sty.ltxml:98, KNOWN_PERL_ERRORS #909): mathtools_sty.rs
  `\clap` provide-only (mathtools.sty:1539 `\providecommand*`), so the kernel's box serves (latex.ltx:16409). 2305.03559 163 → 90 errors; `\overrel` alone is clean.
  Repro package-bindings/mathtools_clap_is_a_box_in_math; guard `perfect_kernel_batch69c::clap_is_a_box_in_math`.
  Open: its clevethm.sty `\setlist[enum#1, 1]{enumerate, label=…}` (90 "'…' is neither a list nor a level").

- **int19** (fix round for the int19 bundle A/B's 39 worse rows, agent_reports/2026-10-17_payload_int19.md). (1) 69b
  round 2's enclosing-mode read of the token after a display's closing `$` is reverted (tex_math.rs
  `\lx@dollar@default` reads it expanded in the display's own mode, as 69b round 1; `with_enclosing_mode` removed):
  under Perl's "Ignoring" recovery the display stays open, so a follower expanded as text was re-read in display math
  — 2207.12569 (`$${\boxed{$u$\mbox{-}$S$…}}$$`) 6 → 101 errors + Fatal, 11,975 → 1,333 words; hep-th9504043,
  0912.2815, math0309133, 1602.01956, 1408.3870, hep-th0002001 the same cascade (KNOWN_PERL_ERRORS #899). Repro
  expansion-primitives/display_closing_dollar_text_after_the_display_kept; guard
  `perfect_kernel_batchint19::display_closing_dollar_text_after_the_display_kept`. OPEN ROW (one coupled change, its
  own A/B): read the token under the enclosing mode AND recover as tex.web §1197 (finish the display, `back_error`)
  for every "Missing $ closing display math." — RED residual expansion-primitives/display_closing_dollar_read_in_the_enclosing_mode
  (`$$y=2$\ifmmode$\fi G`: pdflatex 1 error, here none, the content as pdflatex's), guard
  `perfect_kernel_batch69b::display_closing_dollar_enclosing_mode_residual`; the bundle's alg-geom9411003 (64 → 3
  errors) and 2304.12827 (977 → 202, 39,328 → 48,413 words) were that read's gains and return with the row. (2) amstext's `\text` digests its argument with the alignment
  count raised, as amstext.sty:28-39 sets it in `\hbox{{…}}` (tex.web §342): a `\let\\=\crcr` or `&` in it no
  longer ends the `{align}` cell around it (amstext_sty.rs; amstex.rs and revtex4_support_sty.rs `\text` the same;
  KNOWN_PERL_ERRORS #931). 1703.08664 (`\mWord`, an `\halign` matrix in `\text{\ $…$}`) 58 → 0 errors (69c 101 + Fatal).
  Repro kernel-alignment/text_argument_is_braced_in_an_alignment_cell; guard
  `perfect_kernel_batchint19::text_argument_is_braced_in_an_alignment_cell`. Open: a cell that does end inside a
  mode-switching constructor's argument (`\textbf{$\bgroup\let\\=\crcr\egroup$}` in an `{align}` cell: pdflatex 30
  errors) now leaves that constructor's frames open past the alignment, so the document after it errs (69c: the
  template's braced v-part changed what the cell-ending `\let` swallows; base kept the cascade inside the align).
  (3) amsmath's `\boxed` = `\fbox{\m@th$\displaystyle#1$}` (amsmath.sty:399): an author's `$` in its argument is text
  in that math, as 69c's E1 (amsmath_sty.rs `\boxed`, `\lx@math@in@hbox` in a `\begingroup` group so a script keeps
  its `scriptpos`; KNOWN_PERL_ERRORS #907). 2207.12569 6 → 0 errors. Repro paragraph-mode/dollar_in_boxed_is_text;
  guard `perfect_kernel_batchint19::dollar_in_boxed_is_text`.
  Integrated as **int19** = 69a (rounds 1-3) + 69b (rounds 1-2) + 69c (rounds 1-2) + this fix round on 349b6935a1
  (head a034d4e7f5). Gates 5,793/5,793, cortex 10/10, inits 0/0, dumps identical. Bundle A/B vs 67int18b over 4,031
  papers (1,617 the 69 rounds touched, 1,064 pop67o, 200 fresh first-error papers each of `malformed:ltx:TOC`,
  `unexpected:\endlx@list`, `expected:$`, `unexpected:}`, `expected:{`, `malformed:ltx:theorem`, 150 of
  `malformed:ltx:section`): Errors 35,282 → 28,957, Fatals 119 → 109 (10 better, 0 worse), 1 paper with more errors
  (2304.09123 +1, its invalid `\ensuremath{$\rew$}`, pdflatex errs too), words 82.60 M → 83.43 M; papers cleared to 0
  errors: theorem 69.5 %, `\endlx@list` 73.4 %, `{` 56.6 %, `}` 30.8 %, section 28.7 %, `$` 21.4 %, TOC 18.5 %.
  13 word losses all reach 0 errors (letters set as math in the base now text; base bibliography duplicated). Nets
  136/136 0 worse. Census of the fixed random sample (3,000 run-365 E/F papers) on this head: 18.9 % now clean,
  measured projection 97.66 % (E+F 68,463; 98 % needs ≤ 58,446). Rows: the coupled E3 change (follower read in the
  enclosing mode + tex.web §1197 recovery, together; alg-geom9411003, 2304.12827 come back with it); a cell ending
  inside a mode-switching argument leaves its frames open (invalid TeX); nomencl's `$…$` in `\nomenclature` inside an
  equation errs (pdflatex 0; 1205.1310, 1102.2460).

- **70a** (binding shapes from the rc_struct4 study, agent_reports/2026-10-17_rc_struct4.md; one commit per item).
  (B1) biblatex loads what biblatex.sty:38-44 loads — url, not hyperref (biblatex_sty.rs; KNOWN_PERL_ERRORS #915):
  a cleveref before biblatex passes its begin-document check (cleveref.sty:2370-2376); the binding's links are the
  kernel's (`ltx:cite`/`ltx:bibref`, the BibTeX pool's `ltx:bib-url`/`ltx:bib-identifier`), `\ifhyperref` false
  without an author hyperref (2108.12939 2403.13692 2501.07466 2401.13438 2211.04287 2307.04997 2210.03396
  2112.07048 2508.02914 2112.15046 2106.15390 1811.07125; guard `perfect_kernel_batch70a::biblatex_loads_url_not_hyperref`).
  (M1, X1) minted loads fvextra (minted.sty:32; 2512.12360; KPE #916), physics loads xparse (physics.sty:15;
  2603.20157; KPE #917). (N1, K2, A1) binding-private names: natbib's entry constructor `\lx@NAT@lbibitem` (citeref.sty:96
  defines `\@@lbibitem`; 2309.05436 58 → 0; KPE #918), amsthm's proof style `\lx@amsthm@proof@style` (keytheorems'
  `\newkeytheoremstyle{proof}`; 2512.04638; KPE #919), the proof title ending `\@addpunct{.}` as amsthm.sty:435-437
  (`\nopoint` gobbler; 2503.02416; KPE #920). (I1) a TeX source given as a graphic is read as TeX (OD #701, KPE #921):
  `\DeclareGraphicsRule` keeps `\Gin@rule@<ext>` (graphics.sty:150-154) and graphicx's `\includegraphics` follows a
  rule whose type a package includes itself (`\lx@Gin@include`/`\lx@Gin@setfile`, graphics.sty:254-301) — gincltex's
  `.tex` (gincltex.sty:43), loaded by standalone (standalone.sty:1012), the name expanded (tikzscale.sty:183); the
  epsfig binding no longer overrides `\DeclareGraphicsRule`; svg's `\includeinkscape{x.pdf_tex}`/`{x.pdf}` inputs the
  overlay (svg.sty:1018-1060, :1348-1394) (1604.05572 2511.17298 2305.04329 2408.06999 2305.16841 2402.04829
  2310.03515). (G1) pgfsys-latexml.def's entry in gnuplot-lua-tikz's driver table is `html`, pgfsys-tex4ht.def's
  (OD #702; 2501.11366). Repros package-bindings/{biblatex_cleveref_before_biblatex, natbib_citeref_bibitem,
  keytheorems_style_named_proof, amsthm_proof_title_gobbler}, stub-dependencies/{minted_loads_fvextra,
  physics_loads_xparse}, graphics-tikz/{gincltex_tex_graphic, standalone_tikzscale_tex_graphic,
  includeinkscape_overlay, gnuplot_lua_tikz_driver_mode}; guards `perfect_kernel_batch70a` (10). A/B vs 67int18b over
  196 papers (the 25 study witnesses, 40 biblatex+cleveref, 30 standalone/gincltex/svg `\includeinkscape`, 20 minted,
  20 physics, 20 citeref/amsthm/natbib, 39 pop67o): errors 454 → 221 (124 of them 2205.09799, R1, untouched), 0
  Fatal either side, papers with errors 71 → 28, none worse, no word lost (2309.05436's raw bibliography keys became
  its author-year citations; the graphics papers +2,800 words of figure text). Not changed: H1 — html.sty:31-47
  (LaTeX2HTML 2025) itself loads hyperref under PDF output, which this engine's `\pdfoutput=1` is, and harvard's
  `\IfFileExists{html.sty}` answering true is the 2026-10-15 former-TeX-Live ruling (author_files.rs
  `FORMER_TEX_LIVE`); its witness 1410.6878 is TL-2025-discounted (iopart12.clo). R1 (verify only): 69b's
  `\lx@robustify@ltxprotect` does not cover `\robustify{\ref}` — `\ref `, a constructor, has no `CODE(`/`Stored[`
  in its `\meaning`, so etoolbox's `\robustify` still runs (2205.09799 2 Errors on 69b2-rel and 67int19-rel).

- **71a** (binding shapes from the rc_struct5 study, agent_reports/2026-10-17_rc_struct5.md). (1) B1: amsrefs'
  `{biblist}` stands alone (amsrefs.sty:532-538): outside `{bibdiv}`/`{bibsection}`/`{bibchapter}`
  (`\if@lx@inbibliography`) amsrefs_sty.rs opens a bibliography where the backmatter goes, through the kernel's
  implicit-bibliography marker `\lx@mung@bibliography@pre{implicit}`, headed by the author's `\section*{References}`
  before it or its heading line, else untitled (KNOWN_PERL_ERRORS #939; the BIB_ABSENCE_AUDIT F12 dead end 2 — the
  wrapper opened inside the section — avoided by the backmatter route). Witnesses 1002.1686 1112.3058 2111.14165
  0808.1974 1411.6913 (14/14 sample papers 1 → 0 errors, words unchanged). Repro
  index-bib/amsrefs_biblist_without_bibdiv; guard `perfect_kernel_batch71a::amsrefs_biblist_without_bibdiv`.
  (2) T1: mn2e.cls (v2.2, TL's doc/latex/mnras/LEGACY/mn2e.cls, every shipped copy) defines no `\ion`, so the
  author's `\newcommand{\ion}` is the document's: mn2e_support_sty.rs drops it, mnras_cls.rs keeps the class's, as
  each version defines it — v3.2's (mnras.cls:488) `\text` form where `\text` is defined, v3.0's (1809.05472's copy,
  :477) `\textrm{#1}\,\textsc{…}` where it is not (KNOWN_PERL_ERRORS #940). Witnesses 0908.0351 1112.1796 astro-ph0501147 0909.3906 1206.2419 (11/11 sample mn2e
  papers 1 → 0, 1606.03734's author-modified mnras.cls unchanged; words −7 to −370 are the `\text` error tokens gone and "/ He" runs joined, bag-of-words checked).
  Worse: 1309.5950 4 → 5 (`\ion` undefined: it ships an aas_macros.sty of its own, :142 `\def\ion`, which the
  aas_macros binding shadows — the H1 class of the study, as its `\lya`/`\kms` errors already were). Not added to the
  aas_macros binding: of 301 arXiv copies (6 months, 0905-2405) the ADS releases (v4.0, 88/90 lines, 56 copies; v5.2,
  107 lines, 123) define no `\ion`, and only 2 copies do (1309.5950's 143-line copy, `\def\ion#1#2{#1~{\small\rm…}}`;
  1205.4724's, aastex's `\newcommand\ion[2]{#1$\;$…}`); AASTeX 4.0's aaspp4.sty:481 has it, for its own documents
  (astro-ph9906269's copy) — the author-copy stand-in payload's case. Repro class-bindings/mn2e_author_ion_wins; guards
  `perfect_kernel_batch71a::{mn2e_author_ion_wins, mnras_ion_by_version}`.
  (3) P1: appolb.cls (Acta Phys. Pol. B; not in TL, 15 of 17 papers do not ship it): latexml_contrib appolb_cls.rs on
  OmniBus with `\PACS` (appolb.cls:322) as the pacs classification, `\received`, `\eqsec`, `\lambdabar`
  (OXIDIZED_DESIGN_DIVERGENCES #719), `\headtitle`/`\headauthor` as running-head notes, `\ie`/`\eg`/`\cf`/`\etal`/`\ibid`
  and `\e`, and the rest of the class's public commands (2210.03396's copy): `\preprint`/`\ifPreprint`/`\prepNo`
  (:278-282, a preprint note made at `\maketitle`, which sets `\prepNo`, :399), the `\volume`/`\No` counts and `\pageno` (:283-285), `\ifRedakcja` (:83), `\WERS`
  (:311), `\anymode` (:1019), `\half`/`\Half` (:1053-1057), `\Tr` (:1061), `\Eq` (:1063),
  `\seteqprefix`/`\reseteqprefix` (:1065-1066); its `{eqletters}`/`{eqlettarray}` (:627-648, :1096-1112, lettered
  equations over a redefined `\equation`) are not bound (no witness). The 2 shipping it (0801.2427, cond-mat0205657; raw-loaded before, 0 errors) now
  get the address as a contact and the PACS as a classification, where the raw class put the address in the
  personname and the PACS in a paragraph. Witnesses 0712.3606 1007.2277 1101.3125 1108.2782 1605.06550 nucl-th0308018
  physics0501160 (13/13 1 → 0; hep-ph0403117 5 → 0 with the class's `\headtitle`/`\headauthor` running
  heads, its abbreviations `\ie` & co. and `\e`). Repro class-bindings/appolb_pacs; guards
  `perfect_kernel_batch71a::{appolb_pacs, appolb_running_heads_and_abbreviations, appolb_class_commands}`.
  (4) U1: tkz-euclide's pre-v3 `\usetkzobj`/`\usetkztool` (tkz-tools-eu-modules.tex:18-58): latexml_contrib
  tkz_euclide_sty.rs reads the real package and defines both, where undefined, as loaders with nothing left to load
  (v3+ loads every object; user ruling 2026-10-17, OXIDIZED_DESIGN_DIVERGENCES #720). Witnesses 1506.03598 1612.05374
  2005.14485 1906.04134 1608.04592 (16 sample papers: 12 → 0, 4 one error fewer). Repro
  package-bindings/tkz_euclide_old_usetkzobj; guard `perfect_kernel_batch71a::tkz_euclide_old_usetkzobj`.
  (5) N1: siunitx's bracketed uncertainty with a decimal marker (`4.02(0.52)` → "4.02(52)", siunitx.sty:700-848):
  siunitx_sty.rs `six_match_uncertainnumber` (KNOWN_PERL_ERRORS #941). Witnesses 2311.18489 2504.14399 2605.13387 (9 of
  the 11 sample papers to 0; open: 2607.19972's `:` uncertainty divide, siunitx.sty:780-800, and 2511.08682's 15
  "expected to find complex number").
  Repro package-bindings/siunitx_uncertainty_with_decimal_marker; guard
  `perfect_kernel_batch71a::siunitx_uncertainty_with_decimal_marker`.
  (6) L1: txfonts/pxfonts `\lambdabar`, `\lambdaslash` (txfonts.sty:727-728) as U+019B (KNOWN_PERL_ERRORS #942).
  Witnesses 0801.4523 1001.3510 1202.5232 2103.10486 2104.03192 (9/9 sample papers 1 → 0). Repro
  package-bindings/txfonts_lambdabar; guard
  `perfect_kernel_batch71a::txfonts_lambdabar`.
  (7) Small: the forest stub loads forest.sty:50-53's TikZ libraries and :129's `/forest` family (2012.09112 1910.06571
  2003.06880); todonotes loads positioning (todonotes.sty:32; 1807.00512 36 → 0, 2107.03711; KNOWN_PERL_ERRORS #943);
  pgfmath `mod` truncates and `Mod` adds the divisor to a negative remainder only (pgfmathfunctions.basic.code.tex:263-297;
  `Mod(-5,5)` = 0, 1811.01404; `\pgfmathmod@` was floored; KNOWN_PERL_ERRORS #944); amsmath's 11 accents and amsfonts'
  `\widehat`/`\widetilde` are macros of one argument over LaTeXML's own accents (amsmath.sty:754-797, amsfonts.sty:78-87),
  and breqn's `\sb`/`\sp` macros reading their script (mathstyle.sty:124-127), both through one helper,
  amsmath_sty.rs `\lx@undelimited@argument`: the argument braced, so `\tilde\muu_a` accents `\muu`'s whole expansion
  (0801.1527, 2609.01909; KNOWN_PERL_ERRORS #945), `\Delta_\tasknum` is a subscript τ (2509.22335, 1705.04506 26 → 0),
  `\hat\bbU` with `\newcommand\bbU{\mathbb{U}}` a hat over the U where it was a hat over nothing (1009.2900, 1111.3335);
  a single command is read as its definition says (user ruling 2026-10-17, OXIDIZED_DESIGN_DIVERGENCES #721): one with
  no mandatory argument braced, as TeX braces it (`\vec\nabla`, `x_\infty`; `\hat\relax x` the hat over an empty
  nucleus; `\hat\vec x` the hat over the vec over one, amsmath's nested accent, under amsmath's `\mathaccentV` only), one
  reading a mandatory argument (`\hat\mathbf{x}`, breqn's `x_\text{abc}`, a robust `\vv` through its `\protect \vv␣`,
  an amsmath accent under amsfonts' `\widehat` or breqn's `^`) read with its own arguments, the call the author wrote
  (a macro's by its parameters, braced: `\hat\pair{a}{b}` over `(a,b)`, `\hat\vq[a]{x}` for an `{om}` ltcmd command;
  braced, 2210.10207 lost 11,424 words), with one `Warning:malformed:<reader>` that TeX errs there per source occurrence
  (none where a source typeset in place is re-read — a toc copy without `[short]`, a tag's reference forms,
  `tex_math::REREAD_COPY` —; a `[short]` title or caption warns once), naming the reading made and the source line.
  `\Hat`…`\Vec` are bound after the macro accents (amsmath.sty:892 `\def\Hat{\hat}`): `\Hat\muu_a` as `\hat\muu_a`.
  The shape is rare (7 occurrences in 7,500 sampled 1505/1905/2305 papers, most in unused macros): 1905.01764 gets 7
  Warnings for its 7 `\bm{\bar\mathcal{C}}`, 1505.00442 1; r2 A/B (499 papers vs 67int19b): 109 better, 1 worse
  (1309.5950, above: `\ion` from its own aas_macros copy, restored by the 72a author-copy extras payload), errors 4350 →
  4065, no #721 Warning (no typeset occurrence); r3 (the reading by the call, the per-source-occurrence rule) the same 499
  identical in errors, Warnings, words and `tex=` to r2, 2210.10207 37,122 words and 0 Warnings, 1809.05472 1 → 0; r4: a
  call missing an undelimited argument where the next token ends or structures the list (a column end `&`/`\cr`/`\crcr`/
  `\span` or a catcode-6 token by meaning; `\par`, `\\`, `\end`, `\]`, `\)` by name; the end of the input) is left
  unbraced, its arguments read so far put back — r3's `\hat{\pair{a}{&}}` lost a `cases` cell and `\hat{\nv{\\}}`
  merged two `align` rows, now every cell kept, as 67int19b —, and a robust wrapper's parameterless `\protect <cs>`
  (`\ensuremath`) is looked through (`\hat\ensuremath{x}` warns, read as `\hat{\ensuremath{x}}`). Repros
  stub-dependencies/{forest_loads_its_tikz_libraries, todonotes_loads_positioning},
  package-bindings/{pgfmath_mod_of_a_multiple_is_zero, breqn_script_macro_argument},
  ams-internals/{amsmath_accent_macro_argument, amsfonts_widehat_macro_argument, accent_argument_reads_the_whole_call,
  accent_argument_call_stops_at_the_list_end};
  guards `perfect_kernel_batch71a::{forest_loads_its_tikz_libraries, todonotes_loads_positioning,
  pgfmath_mod_of_a_multiple_is_zero, amsmath_accent_macro_argument, amsmath_accent_ltcmd_argument,
  undelimited_argument_reads_the_whole_call, undelimited_argument_call_stops_at_the_list_end,
  undelimited_argument_through_a_robust_wrapper, amsfonts_widehat_macro_argument, breqn_script_macro_argument,
  undelimited_argument_warning_once_per_occurrence}`.
  **Integration (int21, cd833745d0, binary 67int21):** gates nextest 5836/5836 + cortex 10/10, clippy/rustdoc clean,
  inits plain=0 latex=0, dumps identical to 67int20b; population A/B vs 67int20b over 899 papers (pop67int21.tsv = 71a's
  499 + 400 random from pop67int20, results_r67int21): E 5,959 → 5,674, F 16 → 16, 0 newly Fatal, 109 papers E-better,
  1 E-worse — 1309.5950 (4 → 5, `\ion`, which its own aas_macros.sty:142 defines and mn2e.cls does not; the stand-in
  copy-extras payload 72a reads it); 4 word losses > 1 % (1905.00691, astro-ph0307049, 0707.4285, 2006.09152) are only
  the `undefined:\text` error tokens gone with mnras v3.0's `\ion` (1905.00691 gains its "He" ions); nets 136/136 no worse.

- **fms1** (frontmatter model M3 stage S1, field provenance only; user rulings 2026-10-17; main-session ruling
  2026-10-17: out-of-band provenance, not in-band marks; design study agent_reports/2026-10-16_frontmatter_design_study.md
  §6, the title-print study's stage 3 = rule R3). `Token` gains `prov: u16` in its padding (still 8 bytes,
  `token_size_invariant`; excluded from `PartialEq`, never written or dumped). The kernel `\title`/`\author`
  (sect05.rs) stamp their stored tokens once the frontmatter has its copy (`\lx@field@mark`, base_utilities/defs/frontmatter_fields.rs: field +
  position, and the field's character extent `lx_field_extent_<F>`); expansion inherits it (`inherit_field_prov`, gullet
  and stomach), case changes keep it (`\uppercase`, `\lx@latex@changecase`), the stomach copies it to what a stamped
  token digests to (`fieldProv`), and core `Document::absorb` brackets each run of a field's ink with paired processing
  instructions (`FieldRun`, gated by `note_field_prov_stamped`). `lift_marked_field_ink` — once, at
  `\lx@frontmatterhere` and the fallback's non-abstract branch — joins a print split by its cells/tables, and removes a
  run that is the whole field, in the title block, filling its lines (all fields together), when the XSLT displays the
  field (no document-level titlepage; a document-level title, resp. author creator); a lifted title leaves the
  text-matching dedup (OXIDIZED_DESIGN_DIVERGENCES #146) unrun. OXIDIZED_DESIGN_DIVERGENCES #715 (every whole print
  is lifted, user ruling 2026-10-17), KNOWN_PERL_ERRORS #935. No lock change, no observer,
  nothing deleted (S2/S3). Round 2 fixes the review's findings 1 (author beside a titlepage kept), 2 (no cross-print
  pairing: runs carry ids), 3 (every token test = pdflatex: the round-1 in-band marks are gone, with `is_field_mark` /
  `strip_field_marks` and their `\write`/`\typeout`/`writable_tokens` hooks), 5 (fields judged together) and 6
  (`remove_marked_ink` O(gone·depth)). A/B fms1-rel (round 1) → fms1b over the review's 207 and the round-1 writer's
  336 papers (542 converted; math0309133 has no main file): 542 byte-identical, 0 Error/Warning/Fatal change, no mark in any XML or log, the same 12 `field_ink` lifts (the review's 0706.0979, 1004.2125, 2205.02555 duplicate title/author lifts kept). Round-1 A/B 67int19b-rel → fms1 (336 papers): 332 byte-identical, 0
  Error/Warning change; 2402.14614, 1603.09631 the title lifted instead of text-matched; 1605.00182 and 1503.06457 lose
  the printed author block that repeated their creator (pdflatex prints it once). Repros sectioning-frontmatter/
  field_prov_{token_tests, author_beside_titlepage_kept, fragments_kept, title_author_one_line,
  author_tabular_lifted} RED (round 1) → GREEN; field_mark_{store_print_lifted, before_maketitle_lifted, uppercase,
  centerline_hbox_lifted} and the idiom and scope guards (mention_in_prose_kept, ifx_empty_title, edef_copy,
  write_and_typeout, pdfstring, in_titlepage_kept, beside_titlepage_kept, framed_kept, in_float_kept,
  after_section_kept) and title_study_tp_{dedup,setter} GREEN; guards `perfect_kernel_batchfms1` (20). Round 3 (the
  re-review's findings 1-7 and the profile): `Token` is `#[repr(C)]` `text`, `code`, `prov` with a two-byte `Catcode`,
  gap-free with `code` at byte 4 (`token_layout_code_at_byte_4`): `perf stat -e instructions:u` against base
  a034d4e7f5, empty article +1.29 % (round 2) → +0.65 %, 2311.18489 +3.27 % → +1.66 %. A whole print covers every
  position of the value — a macro's, a primitive's arguments (`gullet::field_cover_of`), a boxless font switch
  (`token::field_cover_set_pending`), the structure digested between a print's boxes (the field ledger,
  `token::field_ledger_cover`) — not its first and last character, and cells join only where the ledger covers the
  stretch between them (finding 2: `\StrLeft{\@title}{5}` of `About \nm` and two slices with a gap stay); only the title
  block is lifted, no running body text before the run (finding 3: a letter's closing `\noindent\@author` stays; a
  cover's preprint-number `\parbox` is no body text, 0706.0979, 1605.00182); positions per `\title` call (finding 7);
  intra-doc links to private items gone (finding 1). Repros sectioning-frontmatter/field_prov_{slice_before_macro_kept,
  two_slices_kept, signature_after_body_kept, earlier_value_copy_kept} RED → GREEN, field_prov_args_whole_lifted GREEN;
  guards `perfect_kernel_batchfms1` (25). A/B 67int19b-rel / fms1c / fms1b over the same 543 papers (542 converted):
  fms1c = fms1b byte-identical on all 542, 0 Error/Warning/Fatal change, 12 `field_ink` lifts each, no mark leaked; the
  review's probes q1-q16, p1-p7: q2, q3, q16 now as pdflatex, the others byte-identical to round 2. Round 4 (the
  round-3 review's findings 1-4, one class: a lift that removed or moved text pdflatex prints; findings 5-6 nits): a
  run's cover is what its own boxes consumed (`Document::absorb` keeps each run's ledger serials) plus what an alignment
  took between and right after them (`digest_alignment_body`'s row-end markers; round 5: taken only when the alignment is shown), never a
  ledger entry absorbed elsewhere or never absorbed (finding 1, probes r19, r20; siblings `\sbox`, `\savebox`,
  `{lrbox}`, a formula's `\mbox` of a box also shown in text, a box used twice, `\phantom`/`\vphantom`); covers are
  clipped to the current call's positions (`lx_field_range_<F>`) and only runs within them join (finding 2, probes r12,
  r13; siblings `\author` twice, `\title` after `\maketitle`, `\edef` copies in reverse order); body text is decided
  by content order, the text earlier in the run's own block too, only boxes exempt (finding 3, probes r9, r18 kept, r16
  = r17 kept, r11 kept; the header cases kept are listed in OXIDIZED_DESIGN_DIVERGENCES #715); a box whose cover ends
  at or before a start the run holds opens a new run, and a run holding one character of the value twice is no whole
  print (finding 4, probes r4, r5 kept, r3 lifted twice; siblings fragment/whole/fragment,
  `\StrLeft{\@title}{1}\@title`); the ledger records nothing after the ink is settled (finding 5); r7, r8, r14, r15
  listed as residual missed lifts (finding 6). Repros sectioning-frontmatter/field_prov_{unused_boxes_kept,
  math_mbox_kept, box_in_math_and_text, phantom_kept, title_calls_kept, edef_calls, author_calls_kept,
  letter_flushleft_kept, letter_quote_kept, header_makebox_kept (round 5: _lifted), reprint_order_kept,
  author_trailing_cr_lifted, whole_twice_lifted (fms1c lifted 3 of its 4 prints)} RED → GREEN,
  field_prov_{box_used_twice_lifted, title_after_maketitle_kept, header_rightline_kept (round 5: _lifted),
  header_text_kept, header_parbox_lifted} GREEN; guards `perfect_kernel_batchfms1` (43) and
  `token::tests::field_ledger_take_cover_and_settle`. A/B over the same 543 papers (542 converted): fms1d = fms1c
  byte-identical on all 542 (the round-3 lifts of 2205.02555, 0706.0979, 1004.2125, 1605.00182, 1503.06457 kept), 0
  Error/Warning/Fatal change, 12 `field_ink` lifts each, no mark leaked; of the review's probes r4, r5, r9, r17, r18,
  r19, r20 and q9 changed, each to pdflatex's content, the rest byte-identical to round 3. `perf stat -e
  instructions:u`, bench, 3 runs: empty article 943.53M → 942.20M (−0.14 %), 2311.18489 4815.06M → 4813.88M
  (−0.02 %), XML identical. Round 5 (the round-4 review's findings 1-4 and nits): the field provenance's thread state
  (the stamped gate, the field ledger and its settled flag, the pending cover) is reset at every conversion's start
  (`document::reset_field_provenance` in `Converter::prepare_session`, not only `reset_thread_engine`): a persistent
  `cortex_worker` thread, a streaming restart and the CLI's supplements lifted nothing after their first document
  (finding 1, probe two/: 2 + 0 → 2 + 2); an alignment's row-end markers are kept on its whatsit
  (`document::FIELD_TAKEN_PROPERTY`) and taken when `Document::absorb` absorbs it, so an alignment in an unused
  `\setbox`/`\sbox`/`{lrbox}` covers no print (finding 2, probes s25, s27, s28 kept as s26, s29); a box is decided by
  the TeX mechanism, not by its element (finding 3, main-session call): the material of the box readers
  (`HBoxContents`, `HBoxArgContents`, `VBoxContents`: `\hbox`, `\vbox`, `\vtop`, `\mbox`, `\makebox`, `\framebox`/`\fbox`,
  `\raisebox`, `\parbox`, a `p` column; so `\rightline`/`\leftline`/`\centerline`), the body of an environment whose begin
  code opens a box group (`TexGroup::Box`: a minipage), and an alignment marked the element they went in when it was
  still empty (`_fieldbox`; round 6: only the element the box's own constructor opened), and only that text is set
  apart — r16 (`\rightline`), r17 (`\makebox`) and an `\mbox`
  header now lifted as the `\parbox` header, `{flushright}`/`{center}`/`quote` headers kept; `ltx:indexmark` is set
  apart (finding 4, s31 lifted). Repros sectioning-frontmatter/field_prov_{second_conversion_lifted,
  unshown_alignment_kept, unshown_alignment_trailing_kept, header_mbox_lifted, indexmark_lifted} RED → GREEN,
  field_prov_header_{rightline,makebox}_kept → _lifted (by the call), field_prov_{header_flushright_kept,
  alignment_box_used_twice_lifted} GREEN; guards `perfect_kernel_batchfms1` (50). Of the review's probes r1-r20, q, p
  and s01-s31 only r16, r17, s31 (lifted) and s25, s27, s28 (kept) changed, each to pdflatex's content. A/B over the
  same 543 papers (542 converted): fms1e = fms1d byte-identical on all 542 (no paper's lift changed by the box call),
  0 Error/Warning/Fatal change against 67int19b and fms1d, 12 `field_ink` lifts, no mark or `_fieldbox` leaked; 20 of
  them through one CLI process (main + 19 supplements) lift the same 12 as one process each. `perf stat -e
  instructions:u`, bench, 3 runs: empty article 942.67M → 944.10M (+0.15 %; no new work on its path, every added
  step is gated on a stamped field), XML identical. (That A/B ran on a pre-commit candidate; the review reran 12 of its
  papers on the committed fms1e-rel, the 7 lift papers among them: byte-identical.) Round 6 (the round-5 review's
  finding 1 and nits, main-session calls): a box marks only the element its own constructor opened, at the point it
  opens it (option B; `Document::mark_field_box`) — round 5 marked the current element when empty, so a `\vbox` nested
  in a `\vbox`/`\vtop`, absorbed straight into an empty paragraph or quote, marked that paragraph and the body text
  after it stopped counting (probes b2, b10, b11: 2 lifts → 0, as fms1d and pdflatex). Marked: a constructor declared
  `mode => "restricted_horizontal"` outside math (`DeclaredMode::restricted_box`, the `fieldBox` whatsit property; the
  first element its absorption opens, `Document::open_element`): `\hbox` (so `\rightline`, `\colorbox`), `\mbox`,
  `\makebox`, `\raisebox`, `\fcolorbox`, `\scalebox`/`\resizebox`/`\rotatebox`/`\reflectbox`; `\framebox`/`\fbox` (its
  text branch); `insert_block`'s capture (`\vbox`, `\vtop`, `\parbox`, minipage, `p` column, framed boxes; not
  `aligning_environment`'s `{center}`/`{flushright}`), the mark riding its rename and an attribute-taking lone child,
  and the font collapse's merge-up (`auto_collapse_children`); a `TexGroup::Box` environment; an alignment's container.
  Not marked: `mode => "text"` (`\textbf`), `\textsuperscript` (its `\mbox` is in math), material a `\vbox` absorbs
  straight into its context. Of the 54 constructors so declared (audited one by one; the 4 environments declaring it are
  marked only by a box group), no TeX box in the original: `\@index`, `\lxDeclare`/`\@lxDefMathDeclare`, `\textbb`
  (these open no element or are set apart), `\lx@acm@addresspart@` (inside the contact) and aastex7's `\tablenotemark`
  (in a table) — listed in OXIDIZED_DESIGN_DIVERGENCES #715. h3 (`\fcolorbox`) and h4 (`\scalebox`) headers now lifted;
  r16, r17, s31, the `\mbox` header and h1/h2 (`\colorbox`) stay lifted. `dialect::def_environment`'s box-environment
  check reads the field-ink gate before the State lookup (nit 3), and `initialize_session` resets the field provenance
  too (nit 5). Repros sectioning-frontmatter/ field_prov_{nested_vbox_body_kept, nested_vtop_body_kept,
  nested_vbox_quote_kept, header_fcolorbox_lifted, header_scalebox_lifted} RED → GREEN,
  field_prov_header_colorbox_lifted GREEN; guards `perfect_kernel_batchfms1` (56). Of the review's probes (new/, sib/,
  r1-r20, q, p, s01-s31) only b2, b10, b11 (kept) and h3, h4 (lifted) changed, each to pdflatex's content. A/B over the
  same 543 papers (542 converted): fms1f byte-identical to round 5's outputs on all 542 (the 5 lift papers ≠ 67int19b as
  before), 0 Error/Warning/Fatal change, 12 `field_ink` lifts, no mark or `_fieldbox` leaked; 20 papers through one CLI
  process: byte-identical to round 5, the same 12 lifts. `perf stat -e instructions:u`, bench, 3 runs, fms1d → fms1e →
  fms1f: empty article 942.46M → 944.60M → 944.63M, 1708.01239 11,773.0M → 11,794.3M → 11,794.6M (round 6 +0.08 % by the
  r6 review, 943.1M → 943.8M; round 5's +0.2 % stands, on the empty article too, where no field is stamped), XML
  identical. int22 (main session, r6 review finding 1): digest-time box marks wait for the live ledger, not for the stamp,
  so a header box before a body-declared `\title` is a box (`field_prov_header_mbox_before_title_decl_lifted`). Open
  (pre-existing, Perl the same): `\hypersetup{pdftitle=\@title}` records the RDF `dcterms:title` as the literal
  `\@title`; our `\pdfstringdef` is a `\gdef`, so a copy made before `\maketitle` empties the store prints nothing.
  **Integration (int22, 9472af17e3, binary 67int22):** fms1 rounds 1-6 rebased onto 7a7b95864d (code delta identical
  to dd4e1592b3) + the main-session r6-review fix; gates nextest 5895/5895 + cortex 10/10, clippy/rustdoc clean, inits
  plain=0 latex=0, dumps identical to 67int21; population A/B vs 67int21 over 1,428 papers (pop67int22.tsv = pop67int21's
  899 + fms1's 543-paper frontmatter set, results_r67int22): E 6,130 → 6,130, F 17 → 17, 0 newly Fatal, 0 E-worse, 0
  word losses > 1 %; 8 changed: the 5 known lift papers (2205.02555, 0706.0979, 1004.2125, 1605.00182, 1503.06457), two
  new whole-print lifts of title + author (1112.0959 −9 words, 2409.06655 −18: the in-place duplicate of the title the
  frontmatter shows, e.g. 1112.0959's title 2 → 1 occurrence), and 2005.02474 (timeout on both sides, W 6 → 5); nets
  136/136 no worse. Reviews: agent_reports/2026-10-17_fms1_review_r{3,4,5,6}.md.
- **73a** (binding shapes from the rc_struct6 study, agent_reports/2026-10-17_rc_struct6.md, and the user rulings of
  2026-10-18). (1) aastex: `\dataset[text]{url}` links the text (aastex_cls.rs, per version from 6 on), `\tablecaption`
  is set where each class prints it (deluxetable_sty.rs `\lx@deluxe@tablecaption`, `\lx@deluxe@print@caption` at
  `\startdata`, emulateapj_cls.rs at its head): emulateapj and AASTeX 5 print their global, never-emptied store at
  every table's head, numbered, so a caption-less table repeats it (OD #740, faithful); AASTeX 6+ print the table's
  own and keep one given before the table (OD #739 surpass) unnumbered, a long table's (`\startlongtable`,
  `{longrotatetable}`) always numbered, empty or not; in a longtable it is the caption only while the longtable has
  none (1903.10014 keeps its printed `\caption`); an aastex `{longrotatetable}` around a plain `tabular` is its
  table float, the stored caption set at its `\begin{tabular}` (aas_support_sty.rs; 2204.03327, its labels the
  subsubsection's as pdflatex's aux); a deluxetable's `[h]` placement is read again (1511.09213), AASTeX 7's `\bibinfo` drops the
  title unless `longbib`, chosen at the class's load (KPE #964; 2502.04486 2608.18221 1601.05101 1010.4329 1903.10014 2204.03327
  2508.20177 2510.05388 → 0 Errors). (2) Provided later, not at load: subcaption `\subfloat`, REVTeX 4.0's `\bm` at
  `\begin{document}` (4.1/4.2 define it at load), elsart's `{thm}` not given to elsarticle (KPE #965; 2511.09162 117 → 0, nucl-th0307033, 2212.01438
  14 → 0). (3) pst-node `{psmatrix}` a group holding an alignment of boxed nodes (pst-node.tex:1163-1220; 0705.2689
  19 → 0, 1712.01407),
  `\cnodeput` reads `(x,y)` and boxes its stuff (cs0702114). (4) ieeeaccess `\Figure` a figure (ieeeaccess.cls:270-300;
  1803.04935 23 → 0, 1910.07402). (5) sigma passes `fleqn` to article and amsmath (sigma.cls:20-21; 2111.06645's `$`
  errors gone, its `\substack}` typo cascade remains). (6) `\DeclareGraphicsExtensions`, `\bibstyle` (KPE #966),
  elsart `{subbibitems}` key, svjour `\tableheadseprule` (KPE #967), mdframed options undigested (1810.02483 2412.10240
  0707.3045 astro-ph0609697 2408.09945 → 0); epsfig no longer replaces graphics.sty's `\DeclareGraphicsExtensions`.
  (7) User ruling 1: `\index`/`\nomenclature`/`\glossary` entries are kept raw and typeset only when the list is
  printed (latexml_core stomach.rs `defer_print`/`resolve_deferred_prints`, OD #738); a nomencl entry makeindex rejects
  (a `|`/`@` in its prefix, symbol, description, or table-mode unit/note) is left out with a Warning (1112.3972 20 → 0,
  2506.02168 1 → 0). Index guards whose fixtures had no print command now print their list; the streaming sweep pins
  its one documented exception, `cluster_regressions/verb_in_index.tex`, compared without its `<indexmark>`s
  (`STREAMING_DIVERGENT_FIXTURES`, asserted by
  `114_streaming_cluster_regressions::streaming_yield_typesets_pending_index_entries`). Accepted (main session): a
  document with an unprinted index has no HTML `<meta keywords>`. mdframed's `frametitle=` FIXED int27 (the options run as
  `\mdfsetup` in the frame's group; the title is the frame's first paragraph, mdframed_sty.rs). Open: cimento.cls `\PACSes`/`\instlist` for papers that do not ship the
  class (1012.2907 1208.1260 0907.0634: the binding falls to OmniBus; a stand-in port of cimento's frontmatter is
  needed). Round 4 (review 73a r3, rebased on int23 8fad3f852f): a `{longrotatetable}` caption no `\begin{tabular}`
  took (`tabular*`, `tabularx`, `longtable`, or after the tabular) is set at its end (`\lx@lrt@end@caption`); a kept
  unnumbered caption holding a `\label` takes an id step (`\lx@deluxe@id@step`, Perl `RefStepID`), so the label names
  the table — a `\ref` to it shows the enclosing section's number, as Perl's CrossRef walks up before its title
  fallback; an AASTeX 5 or 6+ longtable's lone `\tablecaption` is its numbered caption; the emulateapj/AASTeX 5 repeat of an
  in-data caption drops its `\label` (OD #739, #740, #664). caption's title takes the caption `\long`
  (caption_sty.rs `\lx@caption@settitle`, caption3.sty:1488; elife 1907.05060 E 3 → 0, row (t)). neurips (user
  2026-10-18, option C): the binding no longer defines `\projectpage`/`\emailaddress`; a copy's setters store and the
  abstract's end makes the stored values contacts (OD #741; 2605.09681). Repros under class-bindings/,
  captions-floats/, stand-in-copies/neurips_contact/, stand-in-copies/neurips_no_copy/, package-bindings/, graphics-tikz/, index/, index-bib/; guards
  `perfect_kernel_batch73a::*` (36).

- **72b** (`\chapter` exists only where the class has chapters; user rulings 2026-10-17: TeX's answer to every
  `\@ifundefined{chapter}` while the class and packages load, documents that call it still convert, the fallback is
  LaTeXML's ordinary numbered chapter; rounds 1-4, report agent_reports/2026-10-17_payload_72b.md).
  (1) OXIDIZED_DESIGN #179 rewritten: `\documentclass` clears the kernel's `\chapter` before the class loads
  (`retract_chapter_before_class_load`; the lock stays, the binding kept aside, so a raw class's own definition is
  refused and LaTeXML's chapter stands in its place: scrbook), and after each class load (`\documentclass`, raw
  `\LoadClass`, a binding's `LoadClass!`) `settle_chapter_after_class_load` makes the one "has chapters" test
  (`state::class_has_chapters_now`): (a) the class's own code assigned `\chapter` (a meaning not the kernel's, a refusal
  by the lock, a stand-in copy's overlay or declined definition: pasj00), (b) a compiled binding made `\c@chapter`, (c)
  OmniBus guesses an unknown class (`OmniBus:fallback`, set only when the class loader chose OmniBus as the fallback, and
  by subfiles; the OmniBus that 72a r9 provides over a named binding's real base — ceurart, atlasdoc — is no guess, so
  ceurart and atlasdoc's scrartcl answer "no chapters" as pdflatex). It stores the answer, which
  `state::class_defines_chapter` returns to natbib's forks, `\appendix`, appendix.sty and titlesec (with the counter).
  Bindings whose class has chapters define them: ws-rv9x6 (with its counter: 1807.01378), nic-series, jmlr (a call is
  LaTeXML's chapter with jmlr's ClassError text as the Warning, #735), kluwer `[chapter]`; amsart's `\specialsection`
  reads `\lx@kernel@chapter`. (2) OXIDIZED_DESIGN #734: at `\begin{document}`, after the begin-document hooks, an
  undefined `\chapter` gets a stand-in (`kernel_autoload::name_undefined_fallback`, read at the gullet's and stomach's
  undefined-CS sites): the name stays undefined, a call reads LaTeXML's numbered chapter with a counter of its own and
  one Warning (worded without a TeX claim when the class was answered by another binding); a preamble call errs as TeX
  and its error stub is cleared at `\begin{document}` (KNOWN_PERL_ERRORS #962: Perl errs `undefined:\thechapter`).
  (3) Use-time tests in bindings (tocbibind `\tocbibname`, fancyhdr `\f@nch@initialise`, biblatex's
  `\printbibliography` heading) unchanged. Witnesses 0907.3938 1403.2269 1511.03874 1606.03650 2504.00704 0909.2812
  (`undefined:\chapter` gone); 0802.3014 and 1507.01872 call it in preamble package code (pretex's `\UInewAction`), the
  hep-th phyzzx papers (hep-th9303147 hep-th9308050 hep-th9312154 hep-th9412062) in plain TeX. Round-1 A/B vs 67int21
  over 188 papers (the 12, 15 ruling witnesses, 100 on OmniBus-based bindings, 50 shipping a `\@ifundefined{chapter}`,
  17 ws-rv9x6/jmlr/kluwer/pasj00; overlapping): 6 better, 0 worse, 0 new Fatal, 182 byte-identical. Round 2 on top of
  72a r6: see the report. Repros sectioning-frontmatter/{chapter_undefined_in_a_chapterless_class,
  chapter_call_in_a_chapterless_class, chapter_undefined_under_an_omnibus_based_binding,
  chapter_in_the_preamble_errs_and_the_body_chapter_converts, appendix_after_a_chapter_in_a_chapterless_class}; guards
  `perfect_kernel_batch72b::*`. Round 3 (72b review r2): a `\documentstyle` document, which never settles, gets the
  test made when asked (`\documentstyle[appendix]{report}` "Appendix A"); the counter `kernel_chapter_with_warning`
  makes is marked the stand-in's, which `has_chapter_counter` ignores (jmlr's appendix "Appendix A", #735); only the
  first `\documentclass` retracts (memoir with a second one: 2 chapters, 0 Errors); kluwer `[chapter]` and ws-rv9x6
  declare the section within the chapter, ids `Ch1.S1`, and kluwer's `\appendix` letters chapters; every compiled
  binding's class load is `LoadClass!` (`withoptions => true` added); OmniBus's guess (c) puts back
  `\lx@kernel@chapter@counted` (1906.09200's two counter Warnings gone). Open: a standalone raw class's own `\chapter`
  body is LaTeXML's chapter (the lock, as Perl), where TeX typesets the class's; jmlrbook's false Warning and combine's
  ignored `book` option (Corpus Triage Quick Wins). Round 4 (72b review r3): a `\documentstyle` document stores its class's
  answer where the class load ends (tex_job.rs `\lx@latex@ix@store@chapters`, after `\@documentclasshook`, before
  `\compat@loadpackages`; `\chapter:stored`), so the lock's refusal of the document's own `\def\chapter` is no class's
  chapters (`\documentstyle{article}` appendix "A X", as pdflatex; repro
  sectioning-frontmatter/documentstyle_document_chapter_is_no_class_chapter); a runtime script binding's `LoadClass`
  settles too; rows for the body `\documentclass` diagnostic and jmlrbook's `\bookappendix` (Corpus Triage Quick Wins).
  **Integration (int23, c3a799ebad, binary 67int23b):** 72a r1-r10 + 72b r1-r4 cherry-picked onto int22 49f63f14d1
  (conflicts: KPE/OD entry-wise, guards module list, converter.rs reset blocks, state.rs `class_layer_definition` kept by
  72b's `class_defines_chapter` doc), then two integration fix rounds (agent_reports/2026-10-18_int23_fix.md: scrbook
  front matter unnumbered, the class slot through a shipped intermediate class, ws-rv9x6 ids guarded;
  2026-10-18_int23_fix2.md, from the classifier's report 2026-10-18_int23_classify.md: a copy's expl3 syntax ends with its
  load — fcs 2606.16349 recall 14.6 → 95.4; a copy's `\input` files (.clo/.cfg) follow its provides rule — svjour
  0901.1015, 1012.5412, 1111.3868, 1512.01957, 2411.18962, 2107.02119 keep their affiliations; identity by the answering
  binding; elife `\figsupp`, neurips contacts (fallback guess; user 2026-10-18 prefers reading the copy's stored values —
  int24), mdpi cleveref only where the copy loads it, NJDapacite marked loaded with apacite, listing whitespace). Gates
  nextest 5954/5954 + cortex 10/10, clippy/rustdoc clean, inits plain=0 latex=0, dumps identical to 67int22; population
  A/B vs 67int22 over 1,428 papers (pop67int23.tsv = pop67int22's set, results_rint23b): E 6,130 → 6,015, F 17 → 17, 0
  newly Fatal, 35 E-better, 2 E-worse both pdflatex-faithful (1711.03503 "No valid journal"; 2506.06265 natbib option
  clash, pdflatex log:323), 15 word losses > 1 % all classified faithful (mdpi numeric citations per mdpi.cls:265-271 and
  cleveref type-name tags gone; 2604.08415's pgf code dump replaced by tikz pictures, E 40 → 0; MRM/Wiley superscripts;
  "Fig." per the classes); the writers' combined A/B (871 papers: census 400, forks, natbib gains, ceurart 112, atlasdoc 14,
  lmcs/pnas-new/jgcc/czipreprint/clear2025 55, KOMA 100, 72b's 188): E 2,938 → 1,795, F 10 → 8, appendix elements 216 →
  245, counter-named ids 140 → 42. Nets 135/136: elife 1907.05060 E 1 → 3 (the copy's `\format@title@figure` misreads our
  caption; `{fullwidth}` now defined — row (t)), the fcs recall dips 0.6-0.8 faithful. Perf: KOMA load ≈ 2.2× pdflatex's
  (ceurart +162 %, atlasdoc +34 % instructions), recorded in OD #731.

- **73b** (kernel shapes of the rc_struct6 study, agent_reports/2026-10-17_rc_struct6.md rows 2, 4, 5, 6, 9 and the
  kernel one-offs of row 10; report agent_reports/2026-10-18_payload_73b.md, review 2026-10-18_73b_review.md). (1) Row 2:
  fleqn.clo's `{equation}` (fleqn.clo:92-103, `\hbox{$…$}`) is `{equation}`'s new hook `\lx@equation@hook` running
  `\lx@math@in@hbox` (fleqn_sty.rs `fleqn_clo`/`math_displays_in_hbox`; article/report/book/slides `fleqn`, fleqn.sty),
  as iopart's (iopart_support_sty.rs) and a shipped pasj00/pasj01's (pasj00_cls.rs, by the copy's `\def\equation{`);
  their `\[` keeps the kernel display, as it errs in pdflatex (control paragraph-mode/dollar_in_fleqn_clo_bracket_still_errs);
  amsmath sets the hook back. sigma's binding inputs fleqn.clo and loads amsmath `[fleqn]` (sigma.cls:20-21). Witnesses
  cond-mat0202518, hep-ex9806021, 1905.11073, 2311.01721 (2111.06645 non-sole: its 2 `$` errors gone, its `\substack`
  typo Fatal stays). (2) Row 9: amsbsy's `\boldsymbol` runs `\lx@math@in@hbox` in its bounded argument; bm's `\bm`
  sets bold runs between its top-level `$`s, each `$` at the outer level (bm.sty:281); an empty in-math `XMText` is
  dropped (2508.01295, 2506.20975; KNOWN_PERL_ERRORS #907 extended). (3) Row 4: `\setlength` on a non-register runs `#1
  #2\relax`, in the preamble too, where a paragraph it starts raises latex.ltx:9137's `\@nodocument` ("Missing
  \begin{document}"; OXIDIZED_DESIGN_DIVERGENCES #746, KNOWN_PERL_ERRORS #971; 2201.08831, 2401.09168; fancyheadings'
  `\headrulewidth` prints "0.4pt 0pt" with 1 Error, as pdflatex). (4) Row 6: `\captionof` outside a float (`lx@in@float`)
  sets `\@captype` after its caption (caption.sty:391; caption_sty.rs `\lx@caption@of@set@after`; 2206.08980,
  2508.21533; a second `\captionof` of another type then a bare `\caption`: Table 2, as pdflatex). (5) Row 5: a
  `\refstepcounter` in math whose label text cannot set is labelled as `\ensuremath{\the<ctr>}`
  (OXIDIZED_DESIGN_DIVERGENCES #748, KNOWN_PERL_ERRORS #973; 1609.03070 93 → 0, 1904.00290 57 → 0). (6) Row 10:
  `{center}`/`{flushleft}`/`{flushright}` and `\centering`/`\raggedright`/`\raggedleft` let `\\` be the kernel
  newline (latex.ltx:15402-15432; hep-ph9607435; also `{center}` in a `p{}` cell: physics0603169 0 → 6,445 words,
  1307.3556 101+F → 0); ragged2e's commands only where `\\` is still the one it saved (ragged2e.sty:189, 216;
  1606.01980). A heading hands its title font the title braced (sect04.rs; latex.ltx:17256 `#6{…}`; sectsty's
  trailing `\uppercase`, 1509.02957); a caption does not (`\@makecaption` has no font argument). A listings escape is
  read by a mouth of its own where the listing is digested (`\lx@lst@escape`; `\verb._.`, 1106.5908).
  `\DeclareMathAlphabet` makes a one-argument alphabet (OXIDIZED_DESIGN_DIVERGENCES #747, KNOWN_PERL_ERRORS #972;
  1810.09080, and 2211.02508's lost `^d`). (7) Side rows of the review: gensymb's commands are set again in
  `\AtBeginDocument` (gensymb.sty:112-140; 2206.05542's restored one-argument `\degree`); a `"` in a package name is no
  part of it (content.rs `require_package`, tex.ch `more_name`; 2407.00688's `\usepackage{"mydefs"}`). Guards
  `perfect_kernel_batch73b::*`, repros under paragraph-mode, parameter-conditional, captions-floats, macro-state,
  package-bindings, listings-algorithms, fonts-nfss. Round-1 A/B vs 67int23b over 229 papers: Errors 5,881 → 5,438,
  Fatals 13 → 11, error-free 84 → 106, 0 new Fatal; its 2 worse rows (2206.05542 56 → 60, 2407.00688 266 → 267) were
  spurious, not "errs in pdflatex too" (pdflatex: 1 preamble error, 0): the title bracing reached captions, removed in
  round 2. Open rows:
  - (a) `$$` in restricted horizontal mode (tex.web §1138, `\mbox{\[…}`, 1508.07482) dropped: MODE is
    restricted_horizontal in LaTeXML constructs where TeX is not (a beamer frame, a stale `\lx@tag` or in-math-text
    frame left by an earlier error: 1301.0681 0 → 4, 1111.0674 244 → 313, 2312.14898, 2406.05750), so `\[` cannot test
    it; needs a box-level test.
  - (b) A heading's `[short]` toc title (2304.03934, `\subsection[Example: $M(-2;1/2,1/2,3/4)]{…}`, no contents) still
    digests at the heading and raises its errors there (1 Error; pdflatex 0). The faithful design, not built: expand it
    at the heading as latex.ltx:17263 `\addcontentsline` → `\protected@write` does (under `\@unexpandable@protect`,
    raising expansion errors there: `\section[Short \foo]{…}` errs in pdflatex with no contents), then
    `defer_print("toc", …)` with an anchor in the toctitle, resolved by every printed contents (`\tableofcontents`, a
    class's or package's `\@starttoc{toc}`: minitoc, etoc, titletoc, raw classes); a heading after the contents digests
    at once (`deferred_list_is_printed`). Running heads (`\sectionmark{#7}`, which also typesets it) are page furniture,
    dropped by ruling. `\caption[short]`'s lof copy is the same shape.
  - (c) `>{\centering}p{…}` as a last column ending its rows with `\\` now errs as in pdflatex ("Extra alignment tab"),
    where the row end was read before.
  - (d) RED: a Fortran listing line that is a comment (`C(i) = …` with `language=Fortran`, 1106.5908's listing) ends
    with a stray "C" pdflatex does not print (base too): `\lstset{escapechar=\%,language=Fortran}` + `  C(i) = val(j) *
    %B(col\verb._.idx(j))%`.
  - (e) fleqn's siblings: extarticle, extreport, extbook, memoir, aastex701 and other classes `\input{fleqn.clo}` for
    `fleqn` but only article/report/book/slides/sigma/iopart/pasj are wired to `fleqn_clo`; check each (bound or raw:
    a raw class inputs fleqn.clo itself). pasj sets its hook after the whole raw class (a copy loading amsmath after its
    own `\def\equation` would differ; none found).
  - (f) bm: the split sees only top-level `$`; a `$` inside a braced subgroup of a `\bm` argument (bm.sty:252-256,
    `\mathchoice\hbox`) gets no in-hbox treatment. The `Grouped` rule also covers RevTeX 4.x's own unboxed `\bm` (TeX
    errs there anyway).
- **70c** (a `\bibitem` label is typeset only where TeX prints it; rc_struct4 row C1; user ruling 2026-10-18 on text
  written to an auxiliary file). `\lx@bibitem` reads its `[label]` undigested and splits the entry's tags
  (base_utilities/util_text.rs `tag_forms`): the role-less tag (`\fnum@@bibitem`, the class's `\@biblabel`) at the `\bibitem`, the
  reference forms (refnum, autoref, …; TeX's `\b@key`) under the deferred-print list `b@key` of 73a's `defer_print`,
  resolved when a `\@@bibref` names the key (sect11.rs `cite_prints_keys`; `\nocite` does not count; the streaming
  yield's fallback typesets them at the spill). mn2e/mnras bindings define the class's `\def\@biblabel#1{}`
  (mn2e.cls:1370, mnras.cls:1636). OXIDIZED_DESIGN_DIVERGENCES #750. An uncited `\citeauthoryear` label without natbib
  no longer errs (0812.1037 3 → 0, 1105.2318, 1306.6339, 1805.00040, 1712.05964 1 → 0); a cited one keeps pdflatex's
  Error. Guards `perfect_kernel_batch70c::*` (6), repros `index-bib/mnras_bibitem_label_*`,
  `index-bib/bibitem_numeric_label_cited_and_uncited`, `index-bib/mnras_usenatbib_bibitem_label_cited`;
  `perfect_kernel_batch64x::keyless_entries_with_content_are_kept` (a keyless entry has no refnum) and
  `114_streaming_structure::streaming_bibliography_takes_its_heading_unit` (the uncited entry's autoref typeset at the
  spill) updated. A/B vs 67int23b over 118 papers (19 projection witnesses, 19 rc_struct4 C1, 20 citing mn2e/mnras,
  20 natbib author-year, 40 random bibliography papers of pop67int23): Errors 1,181 → 1,118, error-free 41 → 85, 0
  worse, 0 new Fatal; 58 changed, every one only in bibitem `<tags>` (bibblocks, citations and the rest of the XML
  identical); all 3,593 cited keys keep an identical refnum. Open: (a) pasj01's own `\@lbibitem` (pasj01.cls:1600-1628,
  the author/year parse) is not emulated, so its display tag still typesets a `\citeauthoryear` label (1801.05986,
  1610.05396; Error kept); (b) old mn.sty prints `[label]` (mn.sty:1605), so 0903.4835 and astro-ph0010188 keep theirs,
  as pdflatex; (c) the bibliography XSLT shows a bibitem's refnum when it has no role-less tag, so under mn2e/mnras a
  cited entry shows its label in HTML and an uncited one none, where the PDF prints none for either (a 4-line patch
  keeping the empty role-less tag exists, `base_utilities/defs/tags_refnums.rs` `\lx@tag@intags` after_construct, but it diverges from
  Perl's `removeEmptyElement` and shifts `.ltx_bibitem .ltx_tag`'s -2em margin: kept as a row; `.bbl` lists hold only
  cited or `\nocite`d entries); (d) `\@lbibitem`'s `\write` expansion of the label (latex.ltx:17746-17749, `\let\protect
  \noexpand`) is not run, so an unprotected undefined command in an uncited label (`\bibitem[\foo{A} 2017]{a}`) loses
  the Error pdflatex raises (73b row (b)'s `\protected@write` stage); (e) acmlarge.cls:1054-1058 defines
  `\citeauthoryear` inside its own `\cite` (no-natbib branch of its `\AtBeginDocument`), which our locked `\cite` never
  runs: 1710.09074, 1508.03599 keep 1 Error each where the faithful result is 0 (pdflatex's errors there come from its
  broken TL 2025 preamble); (f) html.sty's `\htmlcite` (html_sty.rs:47) builds its `ltx:bibref` without
  `cite_prints_keys`. A cite resolves its key's list only while it has pending entries (stomach.rs
  `lx@deferred@print@pending@<list>`; 40k unprinted `\index` entries and 5k cites took 19.5 s before). Integration
  fixes from the int24 population A/B (its 2 worse rows; minimal repros checked against pdflatex and Perl): (i) under
  natbib, own or a renamed copy (`\NAT@ifcmd`), a `\bibitem[label]` is digested once at the `\bibitem` as before 70c
  (OD #750), and mn2e's binding allocates `\bibhang` (mn2e.cls:1372-1378) — 1003.3017 2 → 0 (repro
  `index-bib/mn2e_natbib_copy/natbib_copy_citeauthoryear_label_cited`); (ii) `\lx@alignment@multicolumn` saves `\\`
  after its `\omit`s and puts it back after the cell, the row end latex.ltx:16603-16607's cell group protects, so 73b's
  `\centering` `\\` (`\@centercr`) stays in the cell — 1511.05496 12 → 0 (repro
  `alignment/multicolumn_centering_then_row_end`; a whole `\begingroup` also ends the alignment state the cell reads at
  its close, setting a centered cell right). Open: (g) under a raw natbib copy a labelled `\bibitem` keeps no refnum, so
  its `\cite` shows no label text (pdflatex "Schneider et al. (2010)"; as 67int23b) — not a label route: the copy's own
  citation commands find no `\b@key` at all (silent-loss row "renamed natbib copies read raw", 77s).
  **Integration (int24, 71b2a9b485, binary 67int24b):** 73a r1-r4 (+ review fixes c64e948812) and 73b r1-r2
  (6253aecd5d) cherry-picked onto int23 8fad3f852f, then 70c (conflicts: SYNC/OD entry-wise, sigma_cls.rs keeps both
  fleqn hooks); a joint review of 73b r2 + 70c (agent_reports/2026-10-18_73b_r2_70c_review.md: SHIP both); main-session
  fixes d5711e807a (deferred-print pending counter: a `\cite` resolves its key's list only while it has entries, 40k
  `\index` + 5k cites 20.9 s → 0.99 s, output identical) and 71b2a9b485 (the A/B's two worse rows: natbib-parsed labels,
  mn2e `\bibhang`, `\multicolumn`'s `\\`; reviewed SHIP). Gates nextest 6012/6012 + cortex 10/10, clippy/rustdoc clean,
  inits plain=0 latex=0, dumps identical to 67int24; population A/B vs 67int23b over 1,428 papers (pop67int23.tsv,
  results_r67int24b): E 6,015 → 5,663, F 17 → 15, 0 worse, 0 newly Fatal, 91 better; of the 80 papers losing >1 % of
  their words, 78 lose them in the bibliography (70c: an uncited label is not typeset) or in index/nomenclature entries
  of a list never printed (73a; 1204.6199's `\makenomenclature` without `\printnomenclature`), 2106.12527 its garbled
  math (E 25 → 0). Binding nets 136/136, 0 documents worse than 67int23.

- **74a** (abstract content: the two shapes of ~40 of the ~61 projection-sample papers still needed for 98 %; report
  agent_reports/2026-10-18_payload_74a.md). (A) User ruling 2026-10-18 "Replay, lock kept": a raw class's refused
  `{abstract}` begin code runs at the kernel abstract's head for its effects (sect05.rs `replay_class_begin_code`, from
  frontmatter_stores.rs `\lx@abstract@class@setters`; OXIDIZED_DESIGN_DIVERGENCES #482 extended, KNOWN_PERL_ERRORS #979):
  what it binds in its plain groups, the class's own `\begingroup` included, holds in the abstract's group
  (state.rs `top_group_frames_meanings`); NFSS's font record and the #634 keywords setters are not kept; a begin code
  whose final command may read past it (environ's `\Collect@Body`) is not replayed; a refused `\newenvironment` keeps
  its code (sect08.rs); every replay restores the boxes its body `\setbox`es (memo-l 1409.3892). The Frontiers/bioinfo/
  4arXiv `\section{Motivation:}` labels are run-in bold paragraphs, as in the PDF (1508.06756, 1406.7114, 1404.0027 and
  14 more). (B) A `\tableofcontents`/`\listoffigures`/`\listoftables` inside a frontmatter container goes after it
  (`Document::float_out_of_frontmatter`, Perl's `floatToElement`; sect04.rs `insert_toc`), and the kernel's three begin
  `\par\@startsection@hook` as the `\section*` they are, ending an unclosed `\abstract` (OXIDIZED_DESIGN_DIVERGENCES #753,
  KNOWN_PERL_ERRORS #978; 1011.3976, 1602.08755, 2007.03709, hep-lat9910044, 2104.05770, sapthesis 1603.02077 whose
  sections are numbered again). Guards `perfect_kernel_batch74a::*`; repros `sectioning-frontmatter/class_abstract/*`,
  `sectioning-frontmatter/{toc_*abstract*, lists_of_figures_and_tables_inside_abstract}`. A/B vs 67int23b over 140
  papers (the 45 sample papers, 15 more Frontiers/bioinfo, 20 raw classes redefining `{abstract}`, 20 balanced
  amsart/revtex/article abstracts, 40 random of pop67int23): E 178 → 116, F 0 → 0, W 2,302 → 2,302, 54 better, 0
  worse, 0 new Fatal. Open rows:
  - (a) Not these shapes, left (sample papers): MRM.cls:1583-1591's `\abstract[2][]` stores `\let\section\@abssection` in
    the `\@abstract` its `\maketitle` prints (2202.01479, 2511.23135, 2409.04353: sections nested in the abstract);
    scrartcl 1507.08575 (`\section*{Abstract:}` twice inside `{abstract}`: the second nests in the first); iopart
    1111.0026 (the whole paper inside `{abstract}`); 1404.0027 and 2108.03515 keep a `caption`/`toccaption`/`para` in
    `ltx:block` error (a figure's).
  - (b) RULED + FIXED int26 (user 2026-10-18: "A \section inside an abstract should close/complete the abstract";
    OXIDIZED_DESIGN_DIVERGENCES #755): a section-level heading ends the abstract. Was: a `\section*` inside a balanced abstract is still kept in place silently by document.rs's sectioning leniency
    (`is_lenient_container`; Perl errs `<ltx:section> isn't allowed in <ltx:abstract>`), schema-invalid; a TOC there
    now goes after the abstract. Whether a heading inside an abstract should end it, go after it or stay is a ruling.
  - (c) Review fixes (int25; agent_reports/2026-10-18_74a_review.md NO-SHIP → fixed in the main session): the
    kernel's `\maketitle` is inert in every replay (cmmse2008.cls:146 called it from the begin code: 0806.4576 lost its
    title and 3 creators); a brace-less `\abstract` is not replayed (frontiersFPHY's `\let\section\absection` disabled
    the hook that ends it: an empty `<document/>`, 0 errors); a begin code's own-group `\def`s reach the body (not put
    back with the stores; its `\gdef`s still are, SolarPhysics.cls:909); `tail_reads_ahead` steps over a `\def`-family
    definer. OXIDIZED_DESIGN_DIVERGENCES #482; guards
    `perfect_kernel_batch74a::{class_abstract_begin_code_calling_maketitle_keeps_the_frontmatter,
    class_abstract_begin_code_local_definition_reaches_the_body, braceless_abstract_does_not_replay_the_class_begin_code}`.
  - (d) 2210.02246 (cmfne.cls:777-791, AMS-style): an empty `<para class="ltx_align_left"><p><text xoffset="36.0pt"/>`
    before the TOC since 74a: the class's `\@maketitle` finds `\abstractbox` non-void after the begin-code replay and
    prints `\moveright 3pc\box\abstractbox`, although `filled_boxes` restores the box; blanking cmfne.cls's
    `\global\setbox\abstractbox=\vtop\bgroup` line removes it, synthetic copies do not reproduce it. No content lost;
    root cause open (low priority, one paper).
  - (e) FIXED int26 (OXIDIZED_DESIGN_DIVERGENCES #754): an unclosed brace-less `\abstract` dropped the whole document
    silently — `\documentclass{article}\begin{document}\abstract Text.` + blank line + `More.\end{document}` gives
    `<document/>` with 0 errors and 2 warnings (`\end{document}` with open groups; `\lx@end@abstract` body unended), 74a
    review finding 2. Content loss must at least be an Error.

- **int25** (a3a3dc2712, from the 67int24b projection's one Error → Fatal, 0903.2259: 5 Errors → Fatal infinite
  expansion): deluxetable_sty.rs wraps `\caption` once per table nesting (`\lx@deluxe@caption@mark`; a deluxetable begun
  inside another's group wrapped the wrapper, 73a); `\aas@table@dollar` (a table's active `$`) only under AASTeX 6+
  (aastex701.cls:8608-8610, :8849; AASTeX 5.x and emulateapj have none, so an author's `$`-delimited macro in a table
  ran away, 62x). 0903.2259 0 Errors, 13 tables (was 3). Repros captions-floats/{deluxetable_inside_deluxetable,
  aastex5_table_dollar_delimited_macro} (pdflatex 0, Perl 0).
  **Integration (int25, a7092cd8e5, binary 67int26):** 74a (7d17c7f365, cherry-picked from pk/74a) + the AASTeX fixes
  (a3a3dc2712) + the 74a review fixes (a7092cd8e5; review agent_reports/2026-10-18_74a_review.md). Gates nextest
  6023/6023 + cortex 10/10, clippy/rustdoc clean, inits plain=0 latex=0. Population A/B over 1,428 papers
  (pop67int23.tsv): 67int25 vs the pushed 67int24b E 5,663 → 5,660, F 15 → 15, 12 changed, 0 worse, no paper losing >1 %
  of its words; the review fixes alone (67int26 vs 67int25) 1 changed (+30 words), 0 worse. Binding nets 136/136, 0
  documents worse than 67int23. Witnesses: 0806.4576 title + 3 creators back; 2210.02246, 1508.06756, 0901.4589,
  1805.07007, 1209.2208, 0908.1913, 1603.02077, 2007.03709, 1011.3976 byte-identical to 67int25.

- **int26** (silent-loss inventory agent_reports/2026-10-18_silent_loss_inventory.md items #1/#2; the `\thetitle`
  cluster, agent_reports/2026-10-18_thetitle_rootcause.md). (A) A frontmatter body left open at `\end{document}` — an
  unclosed `\begin{abstract}`, `\abstract{`, or brace-less `\abstract` — ends there and keeps its text (sect02.rs
  `\lx@end@open@frontmatter`; was an empty `<document/>` with 0 errors); the unclosed `\begin{abstract}` reports
  LaTeX's error as pdflatex and Perl do (OXIDIZED_DESIGN_DIVERGENCES #754, KNOWN_PERL_ERRORS #980). (B) The kernel's
  `\title` and the four support bindings' replay the refused `\title` as `\author` does (OXIDIZED_DESIGN #482): the
  cvpr-template `\thetitle` (16 of 17 witnesses error-free: 2308.03900, 2311.18531, 2407.18899, 2409.09893, 2410.13860,
  2411.17580, 2502.19922, 2503.15412, 2503.23679, 2503.24009, 2506.01799, 2510.03312, 2511.06823, 2511.16669, 2511.18452,
  2602.09494; 0907.1044 3 → 2, its `&` errors left; each gains the title text, `<title>` counts unchanged, its `<ERROR>`
  node gone). 2402.10194 (amsart `\markboth{…}{\thetitle}`, never defined; pdflatex errs too) is an honest author
  error. Guards `perfect_kernel_batch74b::*`. Review (agent_reports/2026-10-18_int26_review.md, SHIP) fixes folded in:
  keywords left open inside an open abstract end both (one body per pass; p8), a `{titlepage}` class `\title` runs
  once (p10), the error names the open environment when recorded; plus (C) user ruling 2026-10-18 — a section-level
  heading inside an abstract closes it (OXIDIZED_DESIGN_DIVERGENCES #755; 74a row (b)); (D) physics `[…]` arguments
  read align-safe (KNOWN_PERL_ERRORS #981, 2606.03447; R3d closed). Open rows from the review: a class `\gdef\Title{#1}`
  stored by a refused `\title` and printed by the document renders empty (probe p2; pre-existing silent loss — the
  replay now runs the setter, re-measure before filing); a heading below section level inside an abstract stays
  inside it (lower levels not covered by ruling 6). (E) A/B fix round: the first int26 A/B (67int27 vs 67int26) gave
  4 papers a spurious `\begin{keywords} ended by \end{document}` — command forms (mn2e `\keywords{…}` 1004.2791,
  mnras `\keywords: …` 2201.07008, spconf `\keywords …` 2309.07195, ieeeconf `\keywords` inside `{abstract}`
  1401.5657) whose body, already on the pushed binary, swallowed the whole document into `<ltx:keywords>` (every
  section an Error). A section-level heading now ends an open keywords body as it ends an abstract
  (`\lx@heading@ends@frontmatter`), `\lx@end@abstract` ends a keywords body opened inside the abstract, and the
  `\end{document}` Error is raised once, naming the innermost environment begun with `\begin` (`begun_environment`;
  the IEEEtran binding's `\begin{IEEEkeywords}` sets `\@currenvir`), never for a command form: the 4 papers 28 Errors
  → 0, words unchanged, sections un-nested (OXIDIZED_DESIGN_DIVERGENCES #754/#755). Pre-int26 projection samples
  held the keywords nesting in 4 + 3 error papers. Guards `perfect_kernel_batch74b::{keywords_command_form_ends_at_first_section,
  keywords_label_inside_abstract_ends_with_it}`; `keywords_unclosed_inside_unclosed_abstract_keep_their_text` now
  pins pdflatex's single error.
  **Integration (int26, 678a5d55d4, binary 67int28):** 0158522543 + 8e1cb136bb + f5874ceff9 + ad466e67b7 (the
  content rule: a frontmatter body that typesets nothing adds no entry, golden `babel/elsart_keyword_brace_form`) +
  the A/B fix round 678a5d55d4. Gates nextest 6035/6035 + cortex 10/10, clippy/rustdoc clean, inits plain=0 latex=0.
  Population A/B over 1,428 papers (pop67int23.tsv): first round 67int27 vs 67int26 5 changed, E 5,660 → 5,665 (the 4
  spurious keywords Errors above, hep-th9412183 −1 word), 0 newly Fatal; fix round 67int28 vs 67int27 4 changed, E
  5,665 → 5,632 (exactly the 4 papers, each → 0 Errors, words unchanged, keywords 94-118 characters, sections back at
  document level), 0 worse. Binding nets 136/136 on 67int27 and on 67int28, 0 documents worse than 67int23. Witnesses
  as listed under (B)-(E); 1507.08575 2 → 0 errors, no empty abstract (2,488 words); 1111.0026 5 → 0, sections
  un-nested.

- **76a** (silent-loss inventory agent_reports/2026-10-18_silent_loss_inventory.md item #19; the run-340
  `undefined:\@pc@lb` cluster, 27 papers). A document's `\halign` redefinition now runs where an alignment the binding
  builds reaches `\halign` in TeX (amsmath's environments, the `\ialign`-based bindings — a document's own `\ialign`, `\let` to `\halign`, does not; not `tabbing`, which latex.ltx
  builds from boxes — its alignment carries `ALIGNMENT_WITHOUT_HALIGN`): `\lx@begin@alignment`'s before-digest
  (tex_tables.rs `run_halign_redefinition_prelude`) digests a parameterless `\halign` macro whose expansion ends with a
  constructor declared `halign_primitive` (the primitive under any name, or its pgf SVG stand-in `\lxSVG@halign`) just
  before the alignment, the final primitive being the alignment already built; any other shape is not run. Typeset
  material precedes the alignment, except where the alignment is the display itself (`ALIGNMENT_IS_DISPLAY`: TeX's
  "Improper \halign inside $$'s"; not `numcases`, a `\vbox`; `multline` unmarked, material kept) — a Warning, dropped; a redefinition's global side effects run once per alignment
  (TeX's amsmath measured environments run them twice) (OXIDIZED_DESIGN_DIVERGENCES #760; Perl never loads cryptocode).
  cryptocode.sty:950-957 binds `\@pc@lb` and its line-numbering `\\` that way: every pseudocode line after the first
  had lost its number, silently, and `mode=text` erred. A/B vs 67int28-rel over the 27 papers: Errors 45 → 0 (each
  paper's `\@pc@lb` plus its cascades; 2107.05054 in TikZ nodes 11 → 0), Fatals 0 → 0, numbered pseudocode lines 145 →
  1,180 (1804.05141 8 → 217, 2410.11134 95 → 481, 2305.06833 9 → 151), each row its own equation, line numbers in order
  with `lnstart` honored, words outside math unchanged (the few deltas are rows' text moving into the numbered row's
  formula); warnings ±: the newly numbered rows add unparsed-math warnings on 2202.07253 (+2), 2305.06833 (+6),
  2311.03723 (+1), the `\@pc@lb` cascades' drop elsewhere. Six amsmath/tabular-heavy papers without cryptocode
  (1303.1693, 1408.3693, 1708.04180, 1809.10292, 2008.06429, 2401.14742) byte-identical. Residual: cryptocode's
  measuring pass sums amsmath's `\maxcolumn@widths`, which the binding leaves empty — one `Warning:expected:<number>`
  per pseudocode without `width=` (pre-existing). Guards `perfect_kernel_batch76a::{cryptocode_halign_redefinition_numbers_every_line,
  alignments_after_a_pseudocode_block_are_unaffected, halign_redefinition_runs_for_a_tabular_not_for_a_tabbing,
  halign_redefinition_not_ending_with_the_primitive_is_not_run,
  halign_redefinition_material_before_a_display_alignment_is_reported,
  halign_redefinition_material_before_a_tabular_is_kept, halign_redefinition_material_before_numcases_is_kept,
  cryptocode_pseudocode_in_a_tikz_node_numbers_every_line}`;
  repros kernel-alignment/cryptocode_halign_redefinition_numbers_lines RED → GREEN,
  kernel-alignment/halign_redefinition_skips_tabbing_and_keeps_material.

- **75a** (silent loss of authored title-page text: K11's off-table stores, agent_reports/2026-10-18_k11_offtable_reach.md
  — 41 + 22 of the 3,000 sample papers lose title-page text because the class's title-page code never typesets; report
  agent_reports/2026-10-18_payload_75a.md). (F1) The `\maketitle` replay gate reads a body as TeX runs it
  (sect05.rs `body_vocabulary_is_defined`, OXIDIZED_DESIGN_DIVERGENCES #756): a `\def`/`\newcommand`'s stored
  replacement text is checked where the body calls it, a called macro's parameter-text delimiters (`\@nil`, `|`) are
  matched, what a called macro defines is defined, `\futurelet` defines. 201 of the 204 sample papers logging "not
  replayed: it uses \@thefnmark" now replay (1505.04771 `\toappear`; ACM copyright blocks, NeurIPS venue lines, author
  notes: 0712.1016, 1609.02839, 1506.04304, 2407.00478). (F2) A class's `\appto`/`\g@addto@macro` to the `\maketitle`
  the lock refused extends the class's recorded definition (state.rs `extended_refused_definition`, #757; wiley-article
  1911.06213, vmsta 1506.02940/1801.00977 whose dates, affiliation and keywords are now kept). (W) K11 table setters
  `corraddress`, `corremail`, `fundinginfo` (notes), `papertype`, `paperfield` (pubnotes), `ACMCCS` (classification)
  (store_setters.rs, #758; 1808.01384, 1911.06213, 2012.13110, 2203.03068, CSML 1608.04592/1602.00476). (D) The class's
  part a refused `\author` replay added to a list (from the journal's value before: the tail it appended, the head it
  prepended, or the text it inserted before the list's end sentinel — round 3) is taken out of that list in the title-page deposit and replay (`note_replay_records`, `\lx@replay@records`, #759; vmsta printed
  its author twice once F2 let its `\maketitle` run), and what another class setter appended to the same list stays.
  (K) `\@topnewpage` takes latex.ltx's bracketed argument and sets the header as a paragraph of its own in a group
  (KNOWN_PERL_ERRORS #982; jpsj's replayed `\twocolumn[…]` printed "[" "]": cond-mat0011490). Round 2 (review
  agent_reports/2026-10-18_75a_review.md): the gate never reports — its own `gate_parameters` reads a parameter text,
  never the sticky-Fatal `parse_def_parameters`, and a `\csname`-made name's tokens run in place while an `\expandafter` chain expands only the token after the
  brace, the rest of either's replacement text stored (`expandafter_definee`; round 3); a called macro's definitions count only for an undefined control sequence
  (`callee_definees`); a body nested past 8 definitions is refused with an Info. Guards `perfect_kernel_batch75a::*`; repros
  `sectioning-frontmatter/class_title_page/*`. A/B vs 67int27 over 244 papers (the 204 + 31 two-refusal `\maketitle`
  papers + the wiley/nbdt/CSML papers of the sample): E 1,933 → 1,924, F 2 → 2, 84 changed, 0 with more E/F, 0 losing
  >1 % of words, 0 doubled creators. Round 3 (r2 review findings 1, 2, 4): a replay that inserts into a list before its
  end sentinel records only the inserted middle (`note_replay_records`, common head and tail; guard
  `maketitle_replay_keeps_other_setters_in_sentinel_list` — `\workplace` first lost "Widget University"); the gate's
  `\csname`/`\expandafter` definitions store their replacement text (guard
  `maketitle_replay_gate_stores_csname_and_expandafter_definitions`); `\lx@replay@records` skips a macro that has since
  taken parameters. Open rows:
  - (a) FOLLOW-UP (not done, needs its own batch): the report's generic rule — an off-table pure store the document set,
    whose only readers are title-page code that did not typeset (a refused body the gate rejects, a deposit or replay
    that errs), handed on as `\lx@add@frontmatter{ltx:note}[role=<setter>]` at `\lx@store@defaults`; ~15 sample papers
    (jfpc.cls:188-191 `\def\@author` built from its stores, 1503.05530; risethesis cover fields; `\fnm`), risk of
    duplicates if the "was it typeset" tracking misses a case.
  - (b) 1912.00312 (nowfnt `[plain]`): its `\author[1]{Deng,Eric}` is the class's bibliography/header data and
    `\maintitleauthorlist{…}` (nowfnt.cls:801, `\gdef\@author{#1}`) is the printed author block; the locked kernel
    `\author` makes creators "Deng", "Eric", … and the author block is lost. A ruling on a class setter that stores into
    the kernel's `\@author` is needed; 1 Error (`\nowfnt@font@articledata` undefined, nowfnt.cls:322, pre-existing).
  - (c) Not tabled: bullsrsl `\correspondance` → `\BSRSL@CorrEmail` (2410.19120; rule (a) needs an `\@` store); vmsta
    `\publishedonline`/`\received` (wrapped, not pure stores; they now print with its title page); pazhastl.cls:81
    `\submitted` (a pure store, printed "Received \@submitted" by its dropped `\@maketitle`: 1108.4222 "September 6,
    2010" lost) — tabled as `\lx@add@date[role=submitted]` (emulateapj's role) it gave 1108.4222 its date, but
    tlp.cls:627's line "submitted \@submitted; revised \@revised; accepted \@accepted" then has every value carried
    and the deposit keeps its labels alone, "submitted ; revised ; accepted" (1209.2617). Needs a rule for a title-page
    line whose values are all carried (drop the line), then the entry.
  - (d) The replay's output is body paragraphs after the title, not frontmatter elements (#265): sig-alternate's
    `\toappear` is a copyrightbox float; an unset `\toappear` leaves an empty copyrightbox float (1202.3179,
    1308.1440, 1402.4624; `typesets_content` counts the float's placement and the picture's coordinates). vmsta's
    replayed title page prints its journal header with the values the document did not set left empty: "Modern
    Stochastics: Theory and Applications () 51–65 / DOI:" and an empty `<ref href="http://dx.doi.org/">` (1506.02940;
    the shape the `\submitted` entry of row (c) was withheld for; page furniture by ruling; OD #757).
  - (e) NEEDS A RULING (the hole class of (D)/#759, second shape): vmsta 2018+ (vmsta.cls `\author@optarg`:
    `\csxdef{author\the\c@author}{…}`, `\xappto\author@list{,author<n>}`) records each author under a `\csname`-made name
    holding the processed name, and its list holds names, not the argument, so #759's "holds the argument" test misses
    it: once #757 lets its `\maketitle` replay, the title page prints the author line (bold "Vitalii Miroshnichenko,
    Rostyslav Maiboroda 0"), a duplicate of the creators, and the e-mail line twice — recovered content, kept nowhere
    else, built in the toks register `\email@author@toks` (vmsta.cls:1226/1248), not a macro — beside what it recovers (affiliation, the received/
    revised/accepted/published dates, keywords, MSC) — 1806.03850, 1810.09667, 2012.03249 (3 of the 3,000 sample;
    67int28 kept the authors once and lost the rest). Proposed single rule: the title-page deposit and replay undo
    every assignment a kept refused-setter replay made (from the journal's values before: an appended tail taken out,
    a newly defined macro emptied), keeping what other code added, replacing #759's argument test; risk: a title page
    that prints only from a setter's records (an affiliation a setter stores) loses it, and it would not touch the
    doubled e-mails (a toks register). Reviewer's safer variant (75a r2 review): take a recorded part out only when its
    expanded text is contained in the creator built from the same setter call, and undo text our own replay doubled from
    that replay's own toks/macro tail. Either needs its own A/B counting e-mail, affiliation and orcid occurrences.
    nowfnt (2108.12618, 2308.02698): with its `\author` replay running (#756) pdflatex's author block "Marina Haikin¹,
    Matan Gavish², …" is now typeset beside the creators split "Haikin", "Marina" (row (b)).
  - (f) RESIDUE (int27 A/B): eas.cls (1512.08933) appends each per-call `\author` name to its own author block and
    `\address` appends the affiliation mark after it; #759 takes the names out of the deposited block (duplicates of
    the five creators, 672 → 663 words), the marks stay as a line of bare superscripts "¹,², ³, ⁴, ¹ and ²".
  **Integration (int27, 2468d4e4b5, binary 67int29):** the silent-loss commits cb964cb607 … 17894c7dd9 + 908281ca98
  (inventory #10, #11 asmejour, #13, #15, #17, #18 honest Error, #20, #22, #24), 76a 884cfa1943 + review r2
  bffcf87498, 75a b17c218eed + r2 a59de2000b + r3 3e9c58b1d7, and 2468d4e4b5 (deepseek.cls listed as an author-file
  stand-in — the one gates failure, `perfect_kernel_batch68d::every_binding_outside_tex_live_is_listed`). Gates on
  3e9c58b1d7: nextest 6064/6065 (that guard) + cortex 10/10, clippy/rustdoc clean, inits plain=0 latex=0, dump diff
  empty; on 2468d4e4b5 the batch68d/74b guards 26/26. Population A/B over 1,428 papers (pop67int23.tsv), 67int29 vs
  67int28: 35 changed, E 5,632 → 5,632, F 15 → 15, 0 worse, words +912; one paper over 1 % word loss, 1512.08933 (row
  (f), explained). Binding nets 136/136, 0 documents worse than 67int28. Earlier int27 measures: the first 9 commits
  (i27a) E 5,632 → 5,632, 2 changed (+525 words); 76a 27 `\@pc@lb` papers 45 E → 0, numbered lines 145 → 1,180; 75a
  r2 over the 3,000 projection sample E 34,993 → 34,981, F 163 → 163, "not replayed" 329 → 52.

  **Integration (int28, a58d155573, binary 67int35):** 77c pasa.cls era `\@popfilename` (7c1db57606; OD #764), 77e
  `RawRead`/`read_raw` (75f505f9ad, c7feb06bd4; WISDOM #99), 77b an/apa/cclayout stores (037f043479; OD #763), 77f
  paragraph breaks in inline-only frontmatter items (a994bb245c, 046f346fc1, f9f254f890, 204de3b6c7; OD #766, KPE
  #984, WISDOM #100 — user ruling (b), decided at construction at the item's own mode frame), 77d author-name
  separators (07a13d93d4 with review-r2 doc/guard fixes; OD #765; user ruling: "and" after a control word splits), 77g
  `\meaning` of the kernel's primitive stand-ins (a81d5ad502, a58d155573; KPE #985). Gates 67int35: nextest
  6088/6088 + cortex 10/10, clippy/rustdoc clean, inits 0/0, dump diff empty. Population A/B (1,428, pop67int23) vs
  67int29: 8 changed, E 5,632 → 5,612, F 15 → 15, 0 newly Fatal, no word loss over 1 %; one paper more (1906.03608: the
  77d mark-as-comma check flags five names merged, XML unchanged). Binding nets (67int34) 136/136: 13 documents gain
  the same honest `merged_creators` flag, XML byte-identical (open row above). 77g witnesses (10 of the projection
  samples): 1005.4193/1803.03483 7 → 0, 1111.1504 8 → 1, 0705.0395 13 → 5, 0907.5495 9 → 1, 1804.04572 14 → 4,
  2004.09110 24 → 16, 2312.05929/1306.1270/cs0603063 −7 each; the only text changes are `\@@over` error tokens
  replaced by fractions. 77a (biblatex filters) is held for its round-3 review.
- **77j** (sole-cause `unexpected:_` shapes of the 67int29 projection samples, agent_reports/sole_underscore_2026-10-19
  and extracted_356a0c47/…a8e6b1083b3a27d34: of 91 sole-`_` papers 25 are TL-2025 author errors pdflatex also raises —
  14 `\DeclareMathOperator{\Mid_3}` names, kept by the TL-version ruling — ~13 unverifiable): names LaTeX only writes or
  uses as control-sequence names are read as names, not digested (KPE #986: `\addcontentsline` file and type,
  acronym keys, a theorem's `[within]`, nomencl's sort prefix); a raw expl3-syntax load (expl3, xparse, l3keys2e, mhsetup)
  gives the catcodes and `\endlinechar` it found back, as `\ExplSyntaxOff` restores what `\ExplSyntaxOn` saved
  (expl3_sty.rs `with_expl_syntax_restored`, expl3-code.tex:247-261; gb4e's active `_`, 2408.09634); underscore.sty's
  begin-document catcode goes through `\AtBeginDocument` in load order (mathstyle's later catcode 12 wins, 2507.09194).
  Witnesses 2003.05548 1 → 0, 2408.09634 1 → 0, 2507.09194 1 → 0, 2402.17181 1 → 0, 1307.7314 2 → 0, 1204.2781 1 → 0,
  2603.23723 2 → 0, 2105.05016 7 → 0, words kept. Guards `perfect_kernel_batch77j::*`; nextest 6095/6095. Open from the
  same study (all five done in 77k, below): `$$` in a caption (Perl `PBoxContents`, sect09.rs `\lx@@caption` text mode; 1005.1555, 1907.01498, 2310.11556,
  1404.2359), pstricks boxes in math (`\put@end`, psmathbox; 2603.19804 +3), blkarray block column specs (1706.05431,
  2207.06508, 2210.16910), aastex `C`/`h` columns (2411.04196, 1907.13126), revtex `\preprint` display toggle (5); rulings
  needed: bibliography/citation commands a paper redefines through the kernel hooks `\@citex`/`\@lbibitem` and the
  Rust-only `locked` `\cite` (6), OD #654's sn-jnl `\href` exception (2), front-matter fields the class never typesets
  (revtex `\thanks` in `\author`, aastex running heads).
  **Integration (int29, 41b82a6abd, binary int29c):** 77a biblatex `\printbibliography` filters, refsection lists and
  numbering (d6d52c14f9, reviews r1-r6 to SHIP; OD #762, KPE #983; `\bibliographystyle` ignored with biblatex's error as
  a Warning, 64zf), 77h an.cls deposit with an unset store (dce4398ef1, 3ff6aa59ab; its per-store `\relax`/`\@empty`
  choice is finished in 77i), 77j names LaTeX only writes, expl3-syntax catcode restore, underscore.sty and backref's
  `\Hy@backout` (e4f6da9779, 5ef0139a04; KPE #986; review SHIP + r2 12187e990e: the document hooks are LaTeX's
  one-time hooks, `\hook_use_once:n` as latex.ltx:9470/9512/9525/15257 — code a begin-document hook adds to itself runs,
  where it was dropped (nested `\AtBeginDocument`, underscore.sty loaded from the hook); r3 41b82a6abd: a standalone
  sub-file's preamble still runs no begin-document code, as before and in Perl — 2405.01743), 77l `\@@underline`
  without a dump (8db91138db; KPE #987). Gates int29c: nextest 6124/6124 + cortex 10/10, clippy/rustdoc clean, inits 0/0, dump diff vs int29b empty. Population A/B (1,428, pop67int23): int29a vs
  67int35 44 changed, E 5,612 → 5,609, F 15 → 15, 0 newly Fatal, 0 worse; 42 are 77a's biblatex data
  (`<bib-data role="labelnumber">` on `.bbl` entries, `inlist="refsection<n>"` on citations), 1812.11661 −9,730 words is
  its three keyword-filtered lists printing 13 + 34 + 48 entries instead of all 95 each (pdflatex prints each once; its
  `filter=`/`segment=` stay unmodelled, OD #762) with E 1 → 0 (`{refsegment}` defined); int29c vs int29a: 0 tally changes, one byte change (2407.09320, a nested formula parsed vs unparsed — the known math-parse nondeterminism); the r2 hook change alone (int29b) changed only 2405.01743 (E 6 → 12, the standalone sub-preamble case r3 fixes).
  Binding nets 136/136 on int29a and on int29c, 0 documents worse than 67int34. Projection int29a: 98.10 % / 98.04 % / combined
  98.06 % (E+F 56,654). Still before the E/F rerun: 77i (title-page sentinel run, silent-loss rows #5-#8 and 23b) in
  round 4, 77m (row #16, tcolorbox listing titles, + `\newtcbinputlisting` text/label and raw tcolorbox `label=`) in
  round 2.
  **Integration (int30, 819537839c, binary int30a):** 77m tcolorbox listing boxes keep their title, listings' `nolol`
  keeps tcolorbox's read-back out of the list of listings (silent-loss row #16; reviews r1 NO-SHIP → r2 SHIP, nits folded;
  OD #768, KPE #988). Gates int30a: nextest 6139/6139 + cortex 10/10, clippy/rustdoc clean, inits 0/0, dump diff vs int29c
  empty. Population A/B (1,428) vs int29c: 0 tally changes, one byte change (2407.09320, the math-parse nondeterminism
  seen in int29). Binding nets 136/136, 0 documents worse than int29c. Witnesses (int29c → int30a, E/F/W identical):
  titles gained in 2402.13846, 2405.01480, 2405.01576, 2405.02221, 2405.02462, 2405.03285, 2409.11413, 2605.21082,
  2606.00555; the spurious "<job>.listing" toc caption gone in 2512.18601, 2606.05464, 2606.27251, 2607.12625; 2507.00833,
  2508.16690 unchanged.
  **Integration (int31, 9482a8e7db, binary int31a):** 77i frontmatter stores — TeX decides what the title page prints
  (silent-loss rows #5 (a), #6a, #6b, #7, #8, 23b; reviews r1-r4 NO-SHIP, r4 narrowly with no real-paper regression;
  r5 closes its six findings, re-checked by the main session on the r4 probes; OD #767): a class's off-table
  store is handed to the frontmatter only when a speculative title-page run with sentinel values shows it printed,
  labels are read from what the run typesets right before the store, per-store `\relax`/`\@empty` for unset stores.
  Gates int31a: nextest 6168/6168 + cortex 10/10, clippy/rustdoc clean, inits 0/0, dump diff vs int30a empty.
  Population A/B (1,428) vs int30a: E 5,609 → 5,609, F 15 → 15, 0 newly Fatal, 0 worse, +42 words; 9 XML changes,
  all gains — smfart address blocks name their author (0801.2953, 1001.4269, 1011.3162, 1901.00144, 2102.05898,
  math0610405) and lose the stray `\bullet` leaders (also 0805.2826), 1703.10331's author block recovered, 1309.6905's
  keywords named by its printed "KEYWORDS:" label with the orphan label paragraph gone. Binding nets (int31a) 136/136,
  0 documents worse than int30a.
  Witnesses (21, int30a → int31a): E/F identical, words equal or up (1503.05530 +22, 1603.03011 +35, 1605.03227 +28,
  1608.02012 +20, 2108.12618 +34, 1303.1716 +36), 1309.6905 −1 (the label above), two papers' warnings 4 → 2 and 5 → 3
  (0801.3557, 2101.09366). With int31 every HIGH-priority silent-loss row is closed; the open inventory rows are
  duplication, mis-split, hover-only or reference-number defects (no lost text).
  **Integration (int32, 62d0b9231b + 4fc65b8e35, binary int32a):** 77k sole-cause shapes part 2 — captions are
  paragraphs of a vertical box (Perl `PBoxContents`; OD #769), pstricks boxes in math keep their math (KPE #989),
  blkarray block preambles format their rows (OD #771), aastex `C`/`h` columns (KPE #991), revtex `\preprint` printed
  per class family (REVTeX 3 `\ifpreprintsty`, 4.0 preamble calls, 4-1/4-2 options; unprinted numbers become
  `dcterms:identifier`; OD #770, KPE #990); reviews r1 NO-SHIP → r2 SHIP; follow-ups: RED
  `boxes-groups/extra_brace_meeting_a_macro_formula_ends_it` + SYNC row (synthetic, 0 of 115,456 zips). Gates int32a:
  nextest 6180/6180 + cortex 10/10, clippy/rustdoc clean, inits 0/0, dump diff vs int31a empty. Population A/B
  (1,428) vs int31a: E 5,609 → 5,577, F 16 → 16, 0 worse; 19 changed — gains (an algorithm's first line numbered
  "1" where base numbered from the second line: 1711.00181, 2005.14485, 2103.04487; blkarray block alignment
  1711.10309; Errors cleared in 1912.12399, 2510.18653, 2310.11556, 0706.4029, 2302.13128), neutral (algorithm2e
  `\KwIn` line nesting after a caption, 2606.11926; whitespace), and three timeout flips that are load noise
  (1805.03313, 2005.02474, 2410.03281 re-run alone: identical status; 2410.03281 times out at 200 s on both).
  Binding nets (int32a) 136/136, 0 documents worse than int31a.
  **Integration (int33, 62fa5906f1 + 52cbccb1db + fd41958ae4 + ca74d03ac4, binary int33a):** 77n merged_creators S4
  (names split inside layout wrappers) + S5 (around note commands), reviews r1-r6 NO-SHIP → r7 SHIP with main-session
  follow-ups (a glued symbol mark is asked for by its label too; a symbol legend's run extent recorded as a SYNC row; OD
  #765, KPE #992); 77q run-367 regressions part 1 (C1 old-form matrices, C2 `\binom`, C4 setstack, C5 proof premises, C14
  minted; KPE #993-995), r1 NO-SHIP → r2 + main-session follow-ups (a real amsmath load after a binding's beyond-original
  one; optica-article/bytedance_seed/fairmeta/selfevolagent load amsmath through a package). Gates int33a: nextest
  6265/6265 + cortex 10/10, clippy/rustdoc clean, inits 0/0, dump diff vs int32a empty; `tools/lint.sh` all checks
  passed. Witness A/B (130 arXiv ids the two batches name) vs int32a: E 3,373 → 933, F 25 → 0, 0 worse; 3 papers lose
  words that were sentences swallowed into math (0705.3096: text outside math 985 → 3,723 words; 1008.1366,
  hep-ph9509392). Population A/B (1,428) vs int32a: E 5,577 → 5,569, F 15 → 15, 8 XML changes — gains: affiliations by
  the authors' marks instead of every affiliation on every author (1809.02949), a junk creator gone (2404.16541), IEEE
  member notes kept (2204.00514, 2303.01876), setstack markup (1303.5499), proof premise rows (1709.05725), 1008.1366
  (C1); math9911253 17 → 22 Errors: AMS-TeX `\pmatrix … \endpmatrix` under LaTeX amsmath (gtart, not in TL; minimal
  repro: pdflatex 6 errors, int32a 0, int33a 3 — the honest old-form diagnostic). Binding nets (int33a) 136/136, 0
  documents worse than int32a but neurips_2024's 2406.09417 (0 → 1 `merged_creators`: four names int32a merged silently
  are now flagged — not spacing runs, as batch 77p found: 2406.09417 is one ` ~ ` inside a group, 2603.20775 a `\quad`
  inside `{\large \bf …}`, both the layout-group row "names parted by a separator inside a layout or declaring group").
  **Integration (int34, 3ee3125260 + 3cfb24d0d3 + a4ffc3fe0a, binary int34a):** 77o HF14 M1 (a box's position is no
  printed text: `OptionalPosition`/`Position` parameter types; a cross-reference is ink) + M3 (aistats family binding,
  OD #772; colm registration), reviews r1-r3 NO-SHIP → main-session follow-up (an address is a key to marked names only
  with two or more distinct marks; 1905.11190, 2405.18281); 77r run-367 regressions part 2 (C3 revtex hyperref reload KPE
  #997, C6, C8, C9, C10, C11 partial, C12, C13 OD #773, C15; `\DeclareSIUnit[<options>]` applied, OD #774), review SHIP
  → r2 should-fixes. Gates int34a: nextest 6305/6305 + cortex 10/10, clippy/rustdoc clean, inits 0/0, dump diff vs
  int33a empty; `tools/lint.sh` all checks passed. Witness A/B (98 ids, the ten 2610 html_feedback sources included) vs
  int33a: F 20 → 7, 0 Fatal worse; 9 papers from Fatal with no output to converted documents (2207.13707 126,755 words,
  2405.01972 122,266, 2508.03994 108,087, 2403.04828, 2601.20155, 1902.09345, 1704.06321, 1204.1969, astro-ph0203021) —
  their Errors count as they convert; 2202.05722 0 → 2 `merged_creators` (the SYNC row above); 2209.14855's word drop is
  an emptied strut table, no text lost. Population A/B (1,428) vs int33a: E 5,569 → 5,566, F 15 → 15, 0 worse. Binding
  nets (int34a) 136/136, 0 documents worse than int33a.
  **Integration (int35, 77s r1-r3 + two main-session follow-ups, binary int35b):** quality batch 1 (longtable
  `\caption*`, algpseudocodex labels, cleveref `\label[type]` at every caption site and in longtables with cleveref's
  name derivation, T2A/T2B/T2C і/ј, `{filecontents}` over a disk file, caption.sty's sub-type refusal with subfigure.sty
  panels hosted in caption's format, a `\captionof` taking the enclosing float's options, an untyped `\captionsetup`
  inside a float; KPE #998-#1000, OD #552/#776, the new "Rust Error Fixes" section); 77p (spacing-run author separators)
  stays in review (it lands after int36). Reviews: r1 SHIP-after-fixes, r2 SHIP-after-fixes, r3 SHIP after the
  `\captionsetup` follow-up. Gates int35a: nextest 6318/6318 + cortex 10/10, clippy/rustdoc clean, inits 0/0, dump diff
  vs int34a empty. Witness A/B (396 papers of the 77s rounds) vs int34a: E 3,381 → 732, F 13 → 8, 0 Fatal worse,
  +45,700 words; the 9 papers with more Errors are caption + subfig `\captionof{subfigure}` refusals pdflatex raises too
  (counts checked against pdflatex for 1506.06268, 1710.07132, 2002.04205, 1907.12365), each with more words.
  Population A/B (1,428) vs int34a: E 5,566 → 5,566, F 15 → 15, 0 worse; 35 papers change, net −954 words, all hidden
  cleveref name tags following cleveref's own derivation (cleveref.sty:1296-1325: a `\crefname{equation}{}{…}` keeps
  the other variant unchanged — 1609.00341; a name given as one macro is uppercased whole by `\MakeUppercase`, as
  pdflatex does — 1704.08542's `\crefname{proposition}{\nameProposition}…` gives "PROPOSITION").
  Binding nets (int35a) caught three regressions of 77s's cleveref name derivation, fixed in the main session
  (int35b): a name led by a command that takes its own argument (`\crefname{line}{\text{line}}…`, lipics-v2021
  2604.22361 0 → 572 Errors; `\Crefname{figure}{\text{Fig.}}…`, IEEEojcsys 2206.03535 0 → 63) was split from it by the
  grouped `\MakeUppercase` (latex.ltx:22385 `{{…}}` — pdflatex errs the same way, but only where that variant is
  typeset, and our tags typeset every variant), now changed whole, braced; a shipped cleveref.sty shim read in place
  of TeX Live's left `\if@cref@capitalise` undefined (siamonline 2605.00332 0 → 3), now read as false (cleveref.sty:3815).
  Guard `perfect_kernel_batch77s::cleveref_derived_name_led_by_a_command`. Open: a cleveref name wrapped in `\text{…}`
  gives no creftype tag at all (int34a the same for `\Crefname{figure}{\text{Fig.}}`; 2206.03535's `\cref{fig}` shows no
  name where pdflatex prints "Fig.").
  Gates int35b (with that follow-up): nextest 6319/6319 + cortex 10/10, clippy/rustdoc clean, inits 0/0, dump diff vs
  int34a empty; witness and population A/B on int35b identical to int35a's tallies above; binding nets (int35b) 136/136,
  0 documents worse than int34a.

- **77k** (sole-cause `unexpected:_` shapes, part 2, of the same study): a float caption is a paragraph of a vertical
  box, Perl's `\@@caption PBoxContents` (tex_box.rs `PBoxContents`, `\lx@enterhorizontal`; OD #769), so `$$…$$` in it is
  display math — read as its macro argument from a mouth of its own, in the inline vertical mode (no `\par`, no
  `\everypar`), a trailing `\label` in front of it as `\lx@caption@postlabel`; `\@@add@caption@counters` reads
  `\ext@<type>` by expansion (digested, its letters left every float body after its caption in horizontal mode);
  pstricks boxes called in math set their material in math under `\ifpsmathbox` (`\put@end`, `\psframebox` family,
  pst-code-box.tex:10-17; KPE #989); a blkarray block's preamble formats its rows (`>{}`/`<{}`, delimiters dropped; OD
  #771); an AASTeX 6+ deluxetable's `$` is active from its begin (aastex701.cls:11298-11299; a Rust-only lag: the
  binding activated it at `\startdata`, so a `$m_{G}$` in the `\tablehead` of a `C` column was text, 2411.04196) and
  `\eatone` is the class's math box (KPE #991); revtex `\preprint` follows the class family's rule (r2: REVTeX 4.1/4.2
  `\preprint@sw` and no preamble number, REVTeX 4.0 a preamble number and the `preprint` style forcing it, REVTeX 3
  `\ifpreprintsty` wherever called) — printed Perl's pubnote read as TeX, otherwise an `ltx:rdf` identifier read as
  written, split at top-level `\\` only (OD #770, KPE #990). Witnesses (int29c → 77k, Errors): 1005.1555 4 → 0,
  1907.01498 2 → 0, 2310.11556 3 → 0, 1404.2359 10 → 0, 2603.19804 2 → 0, 0707.0546 12 → 0, 1010.2311 9 → 0, 1106.3964 2
  → 0, 1706.05431 12 → 0, 2207.06508 4 → 0, 2210.16910 12 → 0, 2411.04196 7 → 0, 1907.13126 76 → 0, 1205.1463 2 → 0,
  1207.6705 1 → 0, 1912.04576 1 → 0, 2110.00437 2 → 0, 2408.13616 2 → 0; words kept or gained. Guards
  `perfect_kernel_batch77k::*`; repros `captions-floats/caption_display_math_is_display_1005.1555`,
  `boxes-groups/pstricks_box_in_math_is_math_2603.19804`,
  `alignment-bindings/blkarray_block_preamble_formats_its_rows_1706.05431` (+ control), `class-bindings/
  aastex_table_dollar_and_hidden_column_2411.04196`,
  `sectioning-frontmatter/revtex_preprint_number_is_an_identifier_2408.13616`. Size attributes of floats sized from a
  caption follow the paragraph (graphrot.xml, `perfect_kernel_batch65a`); open (OD #769 residual): a tabularray
  `talltblr` caption is set at the float's `\hsize`, so a talltblr beside another table is a row of its own
  (`perfect_kernel_batch57::tabularray_tall_table_in_a_*`); `\ifinner` in a display's array cell and blkarray's
  `\Left`/`\Right` side labels (rows in "Perfect-kernel RED repros without a row elsewhere"); a `\label` right after a
  `$$…$$` in a caption labels the display, not the float (Perl the same; OD #769). Cluster A/B (161 projection-sample
  papers: pstricks, blkarray, AASTeX 6+, revtex4, amsart and `$$`-in-caption, 30 each): E 3,350 → 3,248, F 4 → 4, 0
  worse; every byte change classified — display math and cell/box math gained, figure `\label`s recovered (2503.18937,
  2507.06693, 2606.21656: the label after a caption had landed in the leaked paragraph), leading/trailing listing-line
  spaces gone (algorithm2e/listings after a caption), blkarray block `|`/`r` formats applied (2404.19328, 2407.12215),
  rotated-float size attributes (2008.08518, 2403.11191), 2107.02579's side-by-side subfigures no longer broken by a row
  break; inits 0/0, dumps identical to int29c. r2 (review findings): the printed `\preprint` branch is chosen before the
  argument is read and per class family; pstricks boxes keep pst-code-box's group in the formula and revert to the
  author's argument; `Cell::mark_intercol_after` serves `tabular_bindings` and blkarray. r2 A/B on int31a (the 161 + 18
  witnesses + 30 of the 305 REVTeX 3 preprint-style papers of 9601-0001): E 3,514 → 3,249, F 4 → 4, 0 worse; vs r1 the
  REVTeX 3 papers are byte-identical to int31a again (pubnotes, hep-ph0001309 and hep-ph9801334 5 + 4 Errors of r1
  gone), and pstricks `tex` attributes show the source (0707.0546, 1010.2311, 1106.2128, 1106.3964, 2603.19804,
  math0602047, quant-ph0308170).
- **77s** (quality batch 1: brief silent content losses outside the frontmatter, quality inventory
  agent_reports/2026-10-19_quality_inventory.md rows LT-CAPTION-STAR, ALGPSEUDOCODEX, CLEVEREF-TYPE, NATBIB-COPY-LABEL,
  T2A-CYRILLIC, FILECONTENTS-OVERRIDE, CAPTIONOF-TIKZ; guards `perfect_kernel_batch77s::*`, 6). (1) longtable's
  `\caption*{text}` is the text alone (longtable.sty:471-474 `\LT@capti@n`), no "Table N: *" (KPE #363; 2401.16093,
  2401.10535, 2101.04168, 2401.16364 now keep their continued-table captions). (2) algpseudocodex adds its box check
  through the algorithmicx binding's line hook (`\lx@algorithmicx@item@hook`), so `\Require`/`\Ensure`/a document's
  `\item[\algorithmicinput]` keep their labels and `\Statex` steps no number (2408.09313, 2602.08027, 2108.09101,
  2202.00232's substates). (3) cleveref's `\label[type]{key}` gives the labelled object that type's cleveref tags, from
  any `\label` and from the three sites that move a trailing one (`typed_label`, a type token only when a type is
  given), and `\crefname`/`\Crefname` derive the other variant as cleveref.sty:1296-1325 does (KPE #998; 2404.11739
  "Appendix Table", 2605.13648; a derived empty name, `\crefname{equation}{}{}`, now shows no word as in the PDF:
  2409.02896, 2202.13659, 2508.14995). (4) T2A/T2B/T2C `\cyrii`/`\cyrje`/… are Cyrillic і/ј/… (OD #776; 1501.05933).
  (5) a `{filecontents}` keeps a file `\openin` finds unless `[force]`/`[overwrite]` (KPE #279; 2311.02007's shipped
  main_paper.tex/main_supp.tex now read, its Acknowledgement section back). (6) `\captionof` of a caption sub-type is
  refused as caption.sty:346-352 refuses it, then made by `\caption` (KPE #999; 2008.05535 197 → 10 Errors, +5,651
  words, the figure, its captions and the following text kept; 2002.04564 77 → 6, 1801.09806 42 → 8, each a
  pdflatex error per refused `\captionof`). (7) NATBIB-COPY-LABEL is not a label route: the silent-loss row "renamed
  natbib copies read raw" (ruling asked). A/B vs int34a over 349 papers (41 witnesses, 30 per item from a 10-month
  and a 14-month corpus scan, 99 pop67int23 controls): E 1,729 → 1,439, F 10 → 10, W 5,893 → 5,845, words +4,874; one
  paper with more errors, 2508.00020 52 → 54: two more `readBalanced` errors from the derived `\cref@equation@name`
  (`Eq.\!`, cleveref.sty:1283-1325 under `capitalise`; the derived name is right, and `\!` works in text,
  latex.ltx:15673) built into each label's tags after base's own error cascade — `\captionof{subfigure}` with
  subfigure.sty loaded, which `\begin{subfigure}` turned into a call of subfigure.sty's `\subfigure` command (group level
  20 at the end; KPE #999, fixed in r2 below); every other XML change is a
  kept caption/label, a cleveref tag following the given type or derivation, a Cyrillic letter, or the disk file of a
  `{filecontents}`. Residuals: a `\crefformat` for a type with no `\crefname` is not modelled, so `\label[prob]` with
  only `\crefformat{prob}{prob.~(#2#1#3)}` shows the bare number where pdflatex prints "prob. (P)" (2609.28438; it
  showed the equation's "eq." before); an undefined `\label[type]` shows the number without cleveref's "??"
  (2502.00149); the type is the object's, not the label's (two labels of one object with different types share the
  last); `\captionof{subfigure}` inside a `{subfigure}` reports caption.sty's subtype message where pdflatex's is
  "subfigure inside subfigure" (caption.sty:655-656, one error either way).
  **r2** (review follow-ups; guards `perfect_kernel_batch77s::*`, 10): (1) a `\label[type]{key}` inside a caption's text
  (sect09.rs `\@@@hack@caption@`) and in a longtable (`\lx@longtable@label`, the type applied after the table) reads the
  optional type as the other three sites do — read as the key, `\caption{Foo\label[appendixtable]{tin}}` labelled the
  float `[` and printed "Fooappendixtable]tin" (KPE #998). The other `\label` readers: algorithm2e's procedure caption,
  caption's `\@captionof@postlabel`, `\@caption@postlabel` and amsmath's `align` label (`\lx@eqnarray@label`) read the
  optional; lxRDFa's longtable wrapper passes it on; subeqn's `\@lab@subeqnarray` reads `[` as pdflatex does
  (subeqn.sty:57); ntheorem's `\label{key}[type]` is its own syntax; thmtools' `label=` and listings' `label=` build the
  `\label` themselves. (2) `\captionof`'s sub-type refusal tests both lists of caption.sty:348, caption's sub-types and
  subfig's `\sf@counterlist` (caption + subfig: "Figure 1: First" and the `\subfloat` "(a)", where `First` was panel (a)),
  and outside any float the fallback `\caption` is caption.sty's `\caption@OutsideFloat` (:200-207: a second error, the
  text dropped; "CCCLone" printed). (3) a `\captionof` type that is no environment is hosted by the environment its
  binding names (`captionof_environment@<type>`): subfigure.sty's `subfigure` is a command, and `\begin{subfigure}`
  called it with the caption's internals (caption + subfigure: 20 errors, "subfigureFirst", the figure's caption "(c)",
  the text after it drawn in; now 0 errors, "(a) First", "(b) SubA", "Figure 1: Whole" — pdflatex "(a): First", caption's
  format, r3 below; KPE #999), and subfig's
  `\sf@counterlist` names only the sub-floats subfig made (subfig.sty:298-299; after subfigure.sty none): 2508.00020 52
  Errors at int34a (54 r1) → 0, Figs. 4-9 as the PDF numbers them; 2605.30420 101 Errors + Fatal → 0, 1608.05355,
  1806.00503, 1905.01354, 2502.11752 Fatal → 0 errors.
  (4) cleveref's derived variant changes case for both names or neither, as the singular is empty or not
  (cleveref.sty:1309 `\ifx\@tempa\@empty`; it was tested per name); the derivation itself is a Rust error fix (section
  "Rust Error Fixes"). Residuals: a `\label[type]` in an amsmath `align`/`split` —
  pdflatex prints "??" (cleveref's `\label@in@display@optarg` hands `[type]{key}` to amsmath), we drop the type in
  `align` with an "Unexpected text after the value" Warning and give it to the display in `split`; `{filecontents}` named
  `\jobname`/`\jobname.tex` under `[overwrite]` (latex.ltx:19005-19021 "Trying to overwrite `\jobname.tex'", an Error)
  and the "Writing or overwriting file" Warning under `[force]`/`[overwrite]` are not modelled; X2 and OT2 Cyrillic text
  symbols (`\usepackage[X2]{fontenc}` + `\cyrje`, `\fontencoding{OT2}\selectfont\cyrje`) are dropped without a
  diagnostic — no x2/ot2 fontmap here or in Perl (OD #776; silent-loss row "X2/OT2 Cyrillic text symbols"). Not reading `[type]`: pasj00's PASJ longtable label
  (`\lx@pasj@lt@label`, pasj00_cls.rs) and the frontmatter's `\lx@set@frontmatter@label [prefix]` (a `\label[type]` in an
  author or affiliation would take the type as its prefix); neither seen with cleveref. A/B r2 (vs r1 over r1's 349
  papers + 30 with a `\label[type]` in or after a caption + 30 with `\captionof{subfigure}` under subfig/subfigure from a
  full-corpus scan): E 2,283 → 743, F 13 → 8, words +33,684, 363 of 409 byte-identical (7 more are PDF-only sources);
  vs int34a on the 60 new papers E 1,521 → 265, F 2 → 0, words +30,666. More Errors than r1 in 9 caption + subfig papers,
  every one a `\captionof{subfigure}` pdflatex refuses too (2002.04205 1 → 19 and the 1907.12365 thesis 3 → 55, pdflatex
  19 and 53 of the same error; 1506.06268, 1603.00573, 1904.11277, 1907.12250, 2002.02143, 1808.00739, 1710.07132), their
  other errors gone; 1801.09806's four refused `\captionof` outside floats drop "Bounds for y" as pdflatex does.
  **r3** (re-review follow-ups; guards `perfect_kernel_batch77s::*`, 12): (1) a hosted `\captionof` panel is caption's
  own `\caption` of its type (`\lx@caption@hosted`, `caption_format_tokens`): caption's `colon` separator, its
  `labelformat=` and its fonts, where `\subfigure`'s own panels keep subfigure.sty's — "(a): Time step = 1" as the PDFs
  of 1608.05355 and 1806.00503 (r2 "(a) Time step = 1" in footnotesize), 2605.30420's `labelformat=empty` panels with no
  label (r2 "(a)"-"(t)"), an empty-text panel "(a)" with no separator (2508.00020, 2508.03326). (2) a `\captionof`
  takes the options of the type in force where it stands, then its own (caption.sty:300-320; `lx@caption@outer`):
  2502.11752's `\captionof{subtable}` in tables under `\captionsetup[table]{labelfont=bf,labelsep=space}` is "(a) For
  participant 1", bold label, as the PDF. (3) `\bothIfFirst`/`\bothIfSecond` expand their test argument (caption3.sty:54-65):
  a label format's name argument for a type with no name gave `labelformat=simple` a leading no-break space (" (a)");
  Perl's `IsEmpty` test (caption.sty.ltxml:81-87) does not expand either. (4) `\captionof*{subfigure}` with
  subfigure.sty is hosted too (it opened the undefined `{subfigure*}`: an Error and a visible node; pdflatex "Star").
  Residual: pdflatex leaves `\@captype` set to the `\captionof`'s type in the float, so a `\caption` after
  `\captionof*{subfigure}{Star}` directly in a figure is "(a): Whole2" there; ours keeps the float's own type ("Figure 2:
  Whole2"), as Perl (its `\begin{#1}…\end{#1}` wrapper group restores `\@captype`, caption.sty.ltxml:124-125), kept by
  main-session decision 2026-10-20: setting the type for the float is what lost a figure its number and shifted every
  later float (57bu, 2605.19656, 2605.20199; guard `perfect_kernel_gemini::captionof_in_a_float_keeps_the_float_type`).
  r3 review follow-up (main session): an untyped `\captionsetup` inside a float is also that float type's setting
  there, locally, as TeX's order has it (the float's typed options apply when it begins, caption.sty:271-274, :315),
  so a `\captionof{table}` in it reads it before the figure's preamble options (rev77s3 probe p1: "Table 1 Tab in fig
  one", not "Table 1. "; guard `perfect_kernel_batch77s::untyped_captionsetup_in_a_float_overrides_its_type`). Open: a
  document that defines `\@captype` in its preamble makes every `\captionof` outside floats take that type's options
  (caption applies none) — rare, no corpus case. Older problems the r3 review found (int34a the same; open):
  subfigure.sty `\subfigure[]{CCC}` drops its "(a)" (a panel with no tag); `\caption*`/`\captionof*` never apply a typed
  `textfont=`; acmart main captions "Figure 1." where pdflatex prints "Figure 1:" and acmart.cls:958's caption fonts are
  not applied (2602.09445); with subcaption and subfigure.sty both loaded, the main caption and the `\subfigure` panels
  are at 90% where pdflatex has normal size and 80% (probes p7, p9, p3b in ~/data/pk_agents/main/scratch_356a0c47/rev77s3_probes/).
  Docs: OD #776 titled
  T2A/T2B/T2C, X2/OT2 an open silent-loss row; OD #552 lists the caption-text and longtable sites and the
  no-cleveref divergence; the cleveref derivation moved to "Rust Error Fixes". A/B r3 vs r2 over r2's 409 papers:
  E 743 → 743, F 8 → 8, words −20 (2605.30420's 20 labels the PDF does not print); 23 papers change, every one a
  hosted panel's separator, label format or font as caption sets it, or one space less in an algorithm2e label (item 3).
  A 30-paper `labelformat=` sample (12k-paper scan): 28 byte-identical, the other two the same algorithm2e label space
  (1809.08822, 1811.06803). Open, pre-existing (r2 and int34a the same): under lipics-v2021's `labelformat=boxed` an
  algorithm2e caption takes caption's format (a rule box, then "1 Title"), where pdflatex prints algorithm2e's own "Algorithm 1 Title"
  (its ruled caption does not go through caption's; 2002.11064, 1809.08822, 1811.06803) — the word "Algorithm" lost.
- **77u** (quality batch 2: T4 letters, `\p@<counter>` reference forms, the font at `\begin{document}`; guards
  `perfect_kernel_batch77u::*`, 13). (1) T4, X2 and OT2 have font maps (t4_fontmap.rs from fccoding.mf and fc.rme's
  Appendix D, x2_fontmap.rs from x2enc.def/.dfu over T2A's upper half, ot2_fontmap.rs from wncyr10.afm with the fonts'
  transliteration ligatures from wncyr10.tfm; OD #777): t4enc.def's composites are declared, so `\m{y}` is ƴ (2405.01972
  45 → 1 Error, the remaining one the `substitutefont.sty` TL 2025 lacks; its Pular letters ɓ ɗ ƴ in place); X2/OT2
  symbols print (the silent-loss row), OT2's `sh` is ш (1405.1988, 2510.25931), its `\U{i}` й (a `\DeclareTextAccent`
  for a command another encoding declared first now defines this encoding's); a combining-mark accent slot (T4 `\G`
  `\U` `\I`) and T2A/X2's `\C` compose; a text symbol of an encoding still without a map (tipa's T3) warns where it is
  dropped (`\lx@unmapped@text@symbol`). (2) The reference form `\p@<counter>\the<counter>` is applied to the expanded
  number as latex.ltx's `\refstepcounter` stores `\@currentlabel` (`\protected@edef`; `\lx@@p@the@@`, only for a
  `\p@` that does something; OD #778, KPE #1001): 1902.09345 18 → 2 Errors, its first SI equation back, refnum "S1";
  enumitem's short label is the list's `label=` (refnum "(a)", KPE #808) and its label/ref below the first level empties
  `\p@<counter>` ("(a).i", not "(a)(a).i"; 1404.5334 "2(a)" for `ref=\theenumi(\alph*)`, was "22(a)"). (3) The
  begin-document constructor runs latex.ltx's `\init@series@setup` (OD #315 addendum): a preamble or class-body font
  switch does not reach the body (2402.07363's preamble `\sffamily` no longer makes its date and abstract sans; the PDF's
  are CMR; 1506.02089's class-body `\fontfamily{aer}` no longer sets its math letters upright). A/B vs int35a (release,
  `--preload=ar5iv.sty`): 187 papers (31 T4/OT2/X2 from a 40k-paper scan + pop67int23, 33 with `\p@` definitions, 30
  with preamble font switches, 100 pop67int23 controls) E 426 → 366, F 2 → 2, W 4,643 → 4,643, 9 changed outputs; 27
  papers with a top-level preamble switch (a second scan) E 2 → 2, W 764 → 764, 2 changed; every change classified a
  gain (letters, refnums, fonts as the PDF); no paper worse (2405.01972's one more Warning is the T3 notice). Dumps
  identical to int35a, inits 0/0. Residuals: the C11 synthetic shape (a `\p@` reading past a one-token number; our
  reader takes `}` as an undelimited argument, tex.web §395), T3 (tipa) has no map (silent-loss row; closed by 77x, OD #781), T4's own quote/guillemet ligatures (fcr10.tfm:
  the double quotes and `,,` as T1's, `<<`/`>>` reversed) are not modelled.
  **r2** (user ruling 2026-10-20; OD #779): `\normalfont`/`\reset@font` are latex.ltx's (:14113-14122:
  `\encodingdefault`, `\familydefault`, `\seriesdefault`, `\shapedefault`, where Perl and r1 reset to `\rmdefault`), the
  neutral font of notes and tags takes the same defaults, a default the font table does not know resets to serif
  (`\lx@normalfont@base`; droidserif's `[default]`, 2603.23615), and the `.bib` session digests in the defaults where
  `\bibliography` stands, not the document's end (cmpj3's `\ukrainianpart`). A `\renewcommand\familydefault{\sfdefault}`
  document is sans throughout, as its PDF. Corpus: 1,646 of 2601-2609's papers redefine `\familydefault` outside the
  bound acmart (lrec2026.sty 351, imsart.sty's no-op `\rmdefault` 212, aamas 197, other shipped classes and styles
  382, the documents' own 504). A/B r1 vs r2
  (release, `--preload=ar5iv.sty`): the r1 set + 40 of those papers, 254 papers, E 385 → 385, F 2 → 2, W 6,122 →
  6,122, words identical but 0710.2028 (its `\cyr` = `\renewcommand\encodingdefault{OT2}\normalfont` now prints the OT2
  transliteration in Cyrillic, as the PDF); 21 outputs gain `font="sansserif"`, each sans in its PDF (pdffonts/mutool:
  CMSS, TeX Gyre Heros, Nimbus Sans, Source Sans, DroidSans; two papers TL 2025 pdflatex does not build, fontspec and Greek under lrec2026, sans by
  their style), none worse.
  Guards `perfect_kernel_batch77u::{family_default_sets_the_body, family_default_survives_a_preamble_switch,
  unknown_family_default_resets_to_serif}`, `06_cluster_bibliography::bib_entries_take_the_defaults_where_the_bibliography_stands`;
  golden `tikz/ac-drive-components.xml` re-blessed (sans node text, CMSS widths). Cost: controls +0.0-0.25 %
  instructions; a sans document +8.5 % (2603.06291: 6,196 more `<text font="sansserif">` runs). Residual: `\textnormal`
  in maths keeps its fixed serif font.
  **r3** (review fixes; OD #780, KPE #1002, #1003): a formula's token fonts are written relative to the text default,
  not the enclosing text element (document.rs `finalize_rec` resets family, series and shape at an `ltx:Math`), and
  MathML reads the context font inside the formula only (`math_context_font`): r2's sans `\normalfont` had made every
  italic or bold text of a sans document set its math upright (`\emph{$y$}` → `<mi mathvariant="normal">`, a theorem's
  letters, `\mathsf` in `\textit` lost; 2608.14133 1,890 letters, 2608.31026, 2605.03549, 2604.00146, 2607.19982,
  2606.05381, 2604.16675), and Perl's `\textsf{\emph{$y$}}` did so in a serif document. An `\item[…]`'s refnum is the
  enclosing numbered item's `\p@<ctr>\the<ctr>` (latex.ltx `\@item` keeps `\@currentlabel`; dialect.rs
  `ref_step_item_counter`): enumitem's emptied `\p@enumii` lost it (2603.02347, 7 items), covington's `\item[b.]` now
  references its example's number (2405.01972), and Perl's third-level form read "1(a)" where pdflatex has "1a". cmbright.sty:26-28 and acmart's sigchi-a
  (acmart.cls:238-240, :888-890) set the sans default in their bindings; the OD #315 addendum points to #779;
  `\lx@normalfont@base` falls back to the text default without a font. Goldens re-blessed (token fonts only):
  theorem/{amstheorem,latextheorem,ntheorem}, math/{niceunits,declaration_scope_labels}, alignment/listing,
  tokenize/alltt, structure/{algorithm2e_linenumbers,paralists}; guards updated in batch56/61/64/64p/65n/68c (64p's autobold formula is now `bold italic`, CMMIB10 in the PDF,
  where MathML had lost the bold). A/B r2 vs r3 (release, `--preload=ar5iv.sty`; the
  review's 132 + r2's 254 = 386 papers): E 393 → 393, F 2 → 2, W 8,967 → 8,967, 65 outputs changed. Single-letter
  upright `<mi>` 13,409 → 9,272 (plain italic +4,015, sans +101): the seven witnesses back to int35a's counts exactly,
  and serif documents' italic or bold-italic contexts (2102.09074, 1803.04622) gain their italic; 850 bold-upright letters
  are bold italic (`\bm`, `\boldsymbol`, `\boldmath` in italic text, CMMIB10 in the PDF: 1503.06160, 2008.06429,
  2208.10521); covington's `\item[b.]` references its example's number (2405.01972, 135 references); 2603.02347's 12
  nested `\item[…]` refnums are back ("1", "3"); empty-token names and reference tooltips are no longer italic letters.
  None worse: a sibling rule for `\item[…]` (the numbered sibling's label, as pdflatex without the authors' fix) printed
  "(P1)" and "(iv)" in 2502.11630 and 2409.02896 and was dropped (residual below). Inits 0/0, dumps identical to int35a.
  Residuals: Font-Selection Chain Residuals 5-7; the `\item[…]` residuals are restated in r4.
  **r4** (re-review fixes; KPE #1003 corrected, OD #780 worded): under enumitem an `\item[…]` after a numbered item
  of its own nested list had no refnum in r3 (`\setlist[enumerate,2]{label=(\alph*),ref=\alph*}`: `\ref` "(Z)",
  `\autoref` a dangling "item ", pdflatex "a"/"item a"; Perl and int35a "1", Perl's enumitem binding never empties
  `\p@`): where the list's `\p@<ctr>` prints nothing and an enclosing numbered item is in force, the refnum is the
  sibling's `\p@<ctr>\the<ctr>`, read before `\the<counter>` is emptied, LaTeX's `\@currentlabel` (dialect.rs
  `ref_step_item_counter`, `ITEM_LABEL_ENCLOSING`). An itemize's `\item[…]` after a plain `\item` of its itemize, inside an
  enumerate item, had none either: r3's `prints_something` read an empty macro as printing (an empty body is stored
  as no expansion, `Expandable::new`), so `\the@itemii` (sect06.rs) made the itemize item a numbered sibling; it now
  takes the enumerate item's "1", as pdflatex. 2502.11630 and 2409.02896 are byte-identical to r2/r3 (their
  `\item[…]`s are in top-level lists). Repros `list-structure/{enumitem_sibling_item_reference_forms,
  item_after_an_itemize_sibling_takes_the_enclosing_label}`; guards `perfect_kernel_batch77u::{
  enumitem_sibling_item_reference_forms, item_after_an_itemize_sibling_takes_the_enclosing_label}`,
  `enumitem_item_with_its_own_label_takes_the_enclosing_label`'s (Z) now "a". Residuals (KPE #1003): an `\item[…]`
  keeps Perl's `\p@<counter>` form after a numbered sibling whose `\p@` prints (the kernel's `\p@enumii` =
  `\theenumi`: "1", pdflatex "1a") and in a top-level list (no refnum, the reference reads the item's own label;
  pdflatex the sibling's label, or the last section's or equation's with no numbered item in force): authors who
  reference such an item set `\@currentlabel` themselves (`\def\@currentlabel{(P2')}\label{…}`, 2502.11630;
  2409.02896's `\phantomitem`), which `\label` does not read here or in Perl, and the sibling rule printed "(P1)" and
  "(iv)" for them; in a nested enumitem list the sibling rule does replace an author's own label: 2609.14520's
  `\item[ii$')$]\def\@currentlabel{ii$')$}\label{…}` after a numbered sibling prints the sibling's "ii)" (pdflatex
  "ii′)", int35a "(1.1)", rev77u3 finding 1). Reading an author-set `\@currentlabel` at `\label` would close all
  three — DEFERRED by user ruling 2026-10-20 (a residual, not a batch now). A
  plain itemize `\item` has no refnum (Perl's form; pdflatex the enclosing item's label). `\cref` to a description
  `\item[…]` in an enumerate prints no name (RED row above), and math a package makes sans stays serif
  (Font-Selection Chain Residuals 8).
  **Integration (int36, 77u r1-r5, binary int36a):** quality batch 2 lands alone (77p stays in review and lands after
  int37). Reviews: rev77u NO-SHIP (math fonts in sans-default documents, enumitem `\item[…]`) → r3; rev77u2 NO-SHIP
  (enumitem sibling refnum) → r4; rev77u3: the sibling rule kept as LaTeX's `\@currentlabel` (main-session ruling, its
  author-override witness 2609.14520 recorded under the deferred `\@currentlabel` residual) → r5 docs. Gates int36a:
  nextest 6342/6342 + cortex 10/10, clippy/rustdoc clean, inits 0/0, dump diff vs int35b empty. Population A/B (923:
  528 papers of the 77u rounds and reviews + 395 random of the proj98 sample) vs int35b: E 6,709 → 6,647, F 23 → 23, 0
  Error or Fatal worse; one word loss > 1 % (2609.14520, 50,807 → 50,123: its `\ref` texts "(1.1)(i))a" → "a" as its
  enumitem `ref=` sets them, no authored text lost). Binding nets (int36a) 136/136, 0 documents worse than int35b.
- **78a** (quality batch 5, quality-ledger rows #132 jfm, #142 part 1, #164, #146, #26, #133 starred, #149, book
  `\backmatter`, #11, llncs labels; guards `perfect_kernel_batch78a::*`, 12, each RED on int40c; OD #784-#788, KPE
  #1015-#1018, #687 fixed). jfm copies' captions (caption with the copy's options; a copy's own `\@makecaption` its
  small-caps "Figure 1." label format); `\multirow` above an empty p/m/b cell spans its rows (`is_empty_paragraph_cell`,
  only for row-span coverage: Perl's empty-p-cell `align` and kept empty rows unchanged, tests/alignment goldens);
  `\unboldmath`/`\mathversion{normal}` undo `\boldmath`; text libxml merges into a text node is current and ligated;
  biblatex `\citeauthor` prints the name in every style, numeric `\textcite` "Name [1]"; unnumbered headings take the
  title font (Rust-only loss) and titlesec's after-code (`\lx@format@title@after@@`), the title braced; nicematrix
  `\Block<…>`; book's back-matter chapters unnumbered (`no_number_chapters`); `\pdfxform`/`\pdfrefxform` keep and place
  the box (xsavebox); llncs's item labels ("–" bold, then "•"). Guards changed for the intended output: 63
  `biblatex_capital_citeauthor`, 64ze/67x starred titles in the class font, 67w WileyNJD's `\uns` (the class's own again,
  U+2006), 77w book derivative's back-matter chapter.
  Stopped: AMS-TeX's active `@` (KPE #1018; needs `\everycr`, row "`\halign` never inserts `\everycr`"); KOMA's
  `\backmatter` (an unnumbered KOMA chapter resets its counters; OD #737 note). Verified fixed on int40c and closed: #107
  (enumitem refs, 77u), #157 (elsarticle `\ifpreprint`, 62j), #132 mdpi half (77w), #79 "((a))", #133 sectsty numbered
  titles. A/B vs int40c (release, `--preload=ar5iv.sty`; 393 papers: 30 per item from a 30k-sample census — jfm,
  multirow-in-p, boldmath undo, space-formula ligature, starred IEEEtran/titlesec/sectsty, AMS-TeX, book backmatter,
  llncs — 22 biblatex numeric, 2 nicematrix `<>`, 2 xsavebox, 32 witnesses, 99 random controls): E 1143 → 1144, F 1 → 1,
  W 8538 → 8537, 128 outputs changed, each the item's change (no control but one llncs-class paper); the one E+1,
  2108.12618, is nowfnt.cls's `\nowfnt@font@paragraph`, undefined here before as its section fonts are (pre-existing),
  now also reached by its starred paragraph titles. Inits 0/0, dumps identical to int40c.
  Round 2 (rev78a; guards +6, 18 in all, each RED on 78a r1 but two controls, subfigure and floatrow — the floatrow
  repro is byte-identical on r1 and r2, the guard pins the r1 output r2 first lost in its own A/B): a jfm copy's own caption format
  is installed at `\begin{document}` only where caption is not loaded — caption, or subcaption/subfig, which load it,
  replace the class's `\@makecaption` ("Figure 1:"; 1711.01065, 2205.10181, 2507.07213, 1805.00129), the old subfigure
  package and floatrow (caption's kernel alone, `\lx@caption@kernel@only`; 2312.13002, 2111.12004) do not; svmult's
  `\backmatter` keeps its chapters numbered (svmult.cls:478-484; svmono numbers by `\if@chapnum`), book's `\appendix`
  restarts the sections (book.cls:520-524); a numeric `\textcite` of an entry with no name keeps its brackets, per entry
  (crossref.rs `nameless_numeric_show`: "[2]", natbib "Smith [2], [1]" for Perl's "2, 1"; guard
  `bibref_show::natbib_numbers_citet_merges_same_authors` changed; OD #786 residuals: labeltitle, nameless
  `\citeauthor`); KPE #1015's text describes the `paragraph_cell_body` mechanism; the empty `<text/>` in an autoref
  tag is hyperref's `\null` (every figure's, pre-existing). A/B r1 → r2 (release; 503 papers: r1's 393, the review's
  95, the 40 jfm copies with their own `\@makecaption` the census found — not all: rev78a2's 40k-zip sample found 23
  more, 1612.03374 and 2008.04616 among them —, 3 svmult/svmono papers with `\backmatter` from a 141k-zip
  scan): E 1153 → 1153, F 1 → 1, W 9623 → 9623, 10 outputs changed, each a jfm copy under caption/subcaption/subfig
  ("Figure 1:" as their PDFs, 2205.09346 and 1712.00836 checked); HTML A/B of the 28 biblatex papers and 100 controls:
  0 changed.
  Round 3 (rev78a2; guards +5, 23 in all, each RED on 78a r2 — the svmult guard's back-matter half a control): svmult's main-matter `\appendix` is `\par` (svmult.cls:713), so its sections count on — r2's book
  section reset renumbered 1808.08417 (sections 5, 6), 2108.07775 ("8 Appendix") and 2212.13109 ("9 Appendix:
  Octonions", "10") from 1; its `\backmatter` `\let\appendix=\bppendix` (:482), book's resetting appendix; svmono's
  `\backmatter` sets `\@mainmatterfalse` (:418; chapters stay numbered); book's section reset moved into `\appendix`, so
  a second one resets too (KPE #1016). `\usepackage[caption=false]{subfig}` loads `caption3` (subfig.sty:129-141), which
  keeps a jfm copy's own caption; subcaption and subfig's caption path clear `\lx@caption@kernel@only`, as they load
  caption after floatrow's `caption3` (OD #784, its residual narrowed to an explicit `\usepackage{caption}` after
  floatrow). OD #786 retitled for the natbib change; the subcaption 90 % caption size a row of its own ("Perfect-kernel
  RED repros without a row elsewhere"). A/B r2 → r3 (release, cores 0-31; 701 papers: r2's 503, rev78a2's 35 and 8,
  and all 168 svmult/svmono papers of rev78a2's 141,963-zip scan): E 1160 → 1160, F 1 → 1, W 12848 → 12848, 3 outputs
  changed — 1808.08417, 2108.07775, 2212.13109, each byte-identical (ids aside) to r1's and numbered as its PDF; HTML A/B
  (141): 0 outputs changed (2003.06847's +1 trim Warning is the shared-directory artifact of round 2). rev78a3 SHIP;
  main-session follow-ups (5269735e1d): subfig reads its last `caption=` option, as its `\@for` does; guards cite only
  witnesses of their own shape; OD #784's residual covers any `caption3` load before a document's `\usepackage{caption}`
  (RED class-bindings/jfm_caption/jfm_copy_caption_package_after_subfig_without_caption); a row for svmult's main-matter
  `\appendix`.
  **Integration (int42, 78a r1-r3 + follow-ups, binary int42a):** gates int42a: nextest 6569/6569 + cortex 10/10,
  rustdoc clean, inits 0/0, dump diff vs int41a empty; clippy's `double_ended_iterator_last` on the follow-up fixed
  (34b67b76a3, `next_back`). Population A/B (1,000: the 78a rounds' 701 + 299 random of the proj98 sample) vs int41a:
  E 4,718 → 4,719, F 13 → 13, 0 Fatal worse; 21 outputs changed; the one Error more is 2108.12618's nowfnt.cls
  `\nowfnt@font@paragraph`, of the undefined family its section fonts already raise (classified in r1); no word loss
  > 1 %. Binding nets (int42a) 136/136, 0 documents worse than int41a.
- **78b** (`\halign`'s `\everycr`, then AMS-TeX's active `@`, 78a item 8; guards `perfect_kernel_batch78b::*`, 7, six
  RED on 78a2; KPE #1019, #1018 fixed). tex_tables.rs `insert_everycr`: the `\halign` primitive inserts `\everycr`
  after its preamble and after every row a `\cr` or nonredundant `\crcr` ends (tex.web §774/§799; not after `\noalign`
  material, read from the slot `\lx@frozen@everycr`); binding-built alignments run none (their macros start with their
  own `\everycr`, `\ialign`'s `{}`). Then the saved 78a patch: the active `@` reads AMS-TeX's connectors (`\cd@`
  prefers a document's ` @<token>`), `\FN@` = `\futurelet\next`, the punctuation connectors. A/B vs 78a2 (release,
  `--preload=ar5iv.sty`; 757 papers: 4 witnesses, the 56 AMS-TeX papers of 78a, 200 with `\everycr` in a shipped file
  — nearly all `\everycr{}` in bound classes —, 200 setting a non-empty `\everycr` in their own files from a
  200k-paper scan, 100 amsmath `align`/`gather`, 100 longtable/colortbl/arydshln/hhline, 100 random controls): E 1305
  → 1182, F 12 → 12, W 11337 → 11331, 0 papers worse in E/F, 9 outputs changed, each the items' change: the 4
  witnesses (math0005042 28 → 0 errors, its lamstex `\newCD` diagrams built), math9611222 (lamstex `\newCD`, 75 → 1),
  1112.1924 and math9201256 (only the CD's `tex=` changes: AMS-TeX's `@.` reverts as `&&` where it was dropped),
  math0612243 (no false "`\newpage` not allowed" from amstex's `\vnonvmode@` lookahead; its PushbackLimit Fatal is
  pre-existing), 2001.00662 (asl.cls's raw amsmath copy: 17 "`\omit` cannot be used here" → 0, the equation tags in
  their column; +8 "Missing number (Dimension)" warnings — pdflatex has none: asl.cls's raw amsmath measuring pass,
  `\maxcol@width\column@` = `\ifcase\column@\maxcolumn@widths\fi` (gencalcs.tex:874, :1983), reached by more rows now
  that `\everycr` resets `\column@`; a measuring-pass gap, the `\@tempdimb` class). Review follow-ups (int43): a bound
  alignment clears `\everycr` in its group as `\ialign` does (an `\halign` nested in a tabular cell, pdflatex "rows:
  0"); plain_base.rs's `\ialign` is plain.tex's on the NODUMP branch (guards
  `perfect_kernel_batch78b::a_bound_alignment_clears_everycr_for_what_it_holds`,
  `dump_gate_init::nodump_plain_ialign_clears_everycr`). Open: AMS-TeX's quote connector `@"` (amstex.tex:177-190) is
  undefined under the active `@` (pre-existing, an undefined-`@` error before; FIXED 78c item 8, re-verified on int45a:
  "Hello" kept, 0 Errors). Every amsmath, table, control and
  `\displ@y`-copy paper (`\ifdtup` noaligns) is byte-identical. rev78b SHIP.
  **Integration (int43, 78b + follow-ups, binary int43a):** gates int43a: nextest 6578/6578 + cortex 10/10,
  clippy/rustdoc clean, inits 0/0, dump diff vs int42a empty. Population A/B (1,099: the 78b and rev78b sets + 300
  random of the proj98 sample) vs int42a: E 3,889 → 3,766, F 22 → 22, 0 Error or Fatal worse, 7 Error better; 9 outputs
  changed, 0 of the controls; two word losses > 1 % are the witnesses whose `\newCD` diagrams are now built,
  math9611222 (75 → 1 errors) and math0005042 (28 → 0) — no text word lost outside formulas, the counted words the
  error ink that went; the lams letters they gain are the font-map row above. Binding nets (int43a) 136/136, 0
  documents worse than int42a.
- **78c** (quality batch 6, the sel78c selection study's eight items; guards `perfect_kernel_batch78c::*`, 8, each RED on
  int43a; KPE #1021-#1026, #1018 residual). (1) subcaption sets only the sub-captions `\small` (subcaption.sty:218-222;
  Perl's subcaption.sty.ltxml:34-35 made every float caption 90 %, KPE #1021; ~378,000 papers): 30 caption guards and
  the subcaption, figure_mixed_content goldens re-pinned to the normal size, a caption's math, inline TikZ and
  `em`-sized graphics now measured at it. (2) IEEEtran heading tags are the `\the<level>dis` forms "I.", "A.", "1)"
  (IEEEtran.cls:2570-2587, :5397) through `\fnum@<level>`; the refnum stays "I-A", an `\appendices` section keeps
  "Appendix A", its subsections "A." (KPE #1022; ~178,000 papers). (3) acmart's `\citestyle{acmnumeric}` under natbib,
  `\bibstyle@acmauthoryear` for `\citestyle{acmauthoryear}` ("[Able 2001]"), natbib's `citesep` key (acmart.cls:379-393,
  natbib.sty:338; KPE #1023; witness 2112.15328 "[1, 2]"; natbib's `sort&compress`, the citation order and
  ranges, modelled since 78j, OD #844). (4) `\hhline` rules the columns it names, a `=` column twice, `*{n}{…}` repeats, colortbl's
  `>{…}` skipped, one `\cline{a-b}` per run (hhline.sty:61-96; KPE #1024): alignment/colortbls' rainbow rules are double
  now. (5) AMS-TeX's `\TagsOnLeft` default sets `ltx_leqno`, `\TagsOnRight` clears it (amstex.tex:587-590; KPE #1025).
  (6) jmlr's `\acks` is "Acknowledgments" (jmlr.cls:595), jmlr2e's the heading its shipped copy prints
  ("Acknowledgments" in 1512.04829's, "Acknowledgments and Disclosure of Funding" in 2511.10879's; without a copy
  "Acknowledgments" since r2) — not slugs. (7) svmult's `\maketitle` numbers a `\title` contribution as a
  chapter and resets its counters (svmult.cls:2141-2152: 1801.05601 "1.1", "Fig. 1.1", "(1.1)" as its PDF, int43a
  "0.1"), `\thesection` is `\@arabic\c@section` without the chapter's reset until a `\title` (78a's repros now read
  pdflatex's "3 Proofs" and "2 Notes"), the main-matter `\appendix` is `\par`, the back-matter `\bppendix` numbers in
  the appendix chapter (KPE #1026; open: svmult's figure/table/equation numbering under `\title*`, `\ds@numart`'s
  `\@takefromreset`, stays book's). (8) AMS-TeX's `@"` quote connector (amstex.tex:173-190; KPE #1018). A/B vs int43a
  (release, `--preload=ar5iv.sty`; 300 papers: 20 witnesses, 30 each of acmart, IEEEtran with `\subsection`, hhline,
  subcaption + `\caption`, 29 svmult/svmono, the 20 AMS-TeX `\tag` and 11 jmlr `\acks` papers of the 30k sample, 100
  random of the 1k sample): E 84 → 84, F 0 → 0, W 2,970 → 2,970, 0 papers worse; 172 outputs changed, every one by the
  items (caption sizes and the ids their wrappers shifted, heading tags and their new `toctitle`, citation brackets and
  `show`, cell borders and the header guess that follows them, `ltx_leqno`, acknowledgement names, svmult numbering and
  ids), no word of text outside numbering tags gained or lost in any of them. Ledger rows the study re-verified fixed on
  int43a, closed: #4 CLEVEREF-TYPE, #12 ALGPSEUDOCODEX, #14 BIB-PREAMBLE (natbib, KOMA; the apacite residual stays
  open), #16 FILECONTENTS-OVERRIDE, #18 LT-CAPTION-STAR, #19 NAMEREF-IN-TITLE, #20 NATBIB-COPY-LABEL, #21 T2A, #22
  AAS-KEYLESS-REF, #42 THMRESTATE, #48 CAPTIONOF-TIKZ, #57 SNJNL-BOOKTABS, #62 TODONOTES-STYLE, #78 THEOREM-COUNTERS, #90
  EMULATEAPJ deluxetable, #91 ENDNOTES-MARK, #95 MULTLINE-TAG, #104 AMSPPT-FOOTNOTE, #112 COMBINE-BOOK.
  **r2** (rev78c NO-SHIP, its two should-fix findings; guards `perfect_kernel_batch78c::{
  caption_options_after_subcaption_apply}` new, `…::{ieeetran_headings_print_their_heading_forms,
  jmlr_acknowledgements_carry_the_printed_heading}` extended, each new case RED on r1). (1) A `\usepackage[<options>]{caption}` after subcaption runs its options
  (caption3.sty:416-429; the subcaption binding's caption load is the first): caption_sty.rs names `\captionsetup` as
  caption's repeat-load setter (`caption.sty_keyval_setter`, hyperref's mechanism): witness 1602.00134 80 % with bold
  labels as its PDF; caption's v1 options there are first-load keys (pdflatex "undefined" Error), an unknown-key Warning
  here (KPE #1021; open: the sub-caption fonts, `passthrough`; after caption's kernel alone see the follow-up). (2) IEEEtran compsoc numbers arabic, Perl's
  IEEEtran.cls.ltxml:160-174 conditional ported (IEEEtran.cls:2545-2556): "1", "1.1", "1.1.1" and `\ref` "1.1" where r1
  read "I", "I.1", "I.1.1" (63 of 558 IEEEtran papers use compsoc; KPE #1022 corrected: a Rust-only gap; open: a compsoc
  conference appendix heading's "Appendix A." period, `\paragraph` unnumbered). Nits: jmlr2e's fallback heading is
  "Acknowledgments" (45 of 64 shipped copies, jmlr.cls:595); sn-jnl's speculative `{declarations}` deleted; recorded
  open: the hhline guard's `thead` is the header heuristic's, a `\hhline{|=|…}` after a header row loses its `<thead>`
  as `\hline\hline` does (1706.03044, 2207.13493), a `\hhline` longer than the row is silent where pdflatex errs (KPE
  #1024); svmult volumes of several contributions (KPE #1026, row above). A/B r1 vs r2 (release, `--preload=ar5iv.sty`;
  420 papers: r1's 299 + 3 by main file, rev78c's 60, 20 IEEEtran compsoc, 10 compsoc conference, 29 subcaption papers
  loading caption with options after it, witness 1602.00134): E 213 → 213, F 1 → 1, W 4,626 → 4,626, 0 papers worse; 54
  outputs changed — 29 compsoc papers (heading tags and refnums arabic), 25 caption-option papers with 1602.00134 (each
  its own options: sizes, bold or italic labels, `labelsep`, `textfont=it`) — no text word outside tags gained or lost.
  **Follow-up** (rev78c2 SHIP after its finding 1; guard `perfect_kernel_batch78c::caption_v1_options_after_caption3_apply`
  new, RED on r2): after caption's kernel alone (floatrow.sty:39 `caption3`, subfig's `caption=false`) a
  `\usepackage[<options>]{caption}` is caption's first load: caption_sty.rs's repeat-load setter is
  `\lx@caption@repeat@options`, which there runs each option through the first load's handler (the v1 options valid) and
  drops the kernel-only mark, else `\captionsetup` (2509.03211, 2409.17596, 2304.03621: `[bf,small]`, two unknown-key
  Warnings and the options dropped on r2; 90 % with bold labels as pdflatex now). Recorded open (KPE #1021, #1022): a
  repeated identical option after an intervening setup, an option-less caption load after caption's kernel alone (OD
  #784's residual), a `\footnote`'s size inside a sized caption (1907.07929), IEEEtran's `\appendix[<title>]` (heading
  and title lost, rev78c2 probe i2).
  **Integration (int45, 78c r1-r2 squashed 137b084d73, follow-up 6266374b9b; binary int45b):** gates int45b: nextest
  6614/6614 + cortex 10/10, clippy/rustdoc clean, inits 0/0, dump diff vs int45a empty. Population A/B (719: the 78c
  rounds' and reviews' 420 + 299 random of the proj98 sample) int45a vs int44b: E 3,708 → 3,708, F 12 → 12, 0 Error or
  Fatal worse; 164 outputs changed; 30 papers lose > 1 % of their words, every one acmart, the words its author-year
  citations printed ("et al", years) now `[N]` (acmnumeric, item 3; 2405.12538, 2404.06432). Follow-up A/B (73: the 3
  witnesses, 12 papers loading caption with options after floatrow or `[caption=false]{subfig}`, 58 subcaption controls)
  int45b vs int45a: E 139 → 139, F 1 → 1, 3 outputs changed — the witnesses, each one unknown-key Warning fewer and its
  option applied. Binding nets (int45a) 136/136, 0 documents worse than int44a.
- **78d** (quality batch 7, the sel78d selection study's items 1, 3, 6, 8, 9; guards `perfect_kernel_batch78d::*`, 5,
  each RED on int45b, and `token_kernel_gaps::kernel_def_hash_brace_delimiter`; OD #793, KPE #1027-#1029, #1022
  residual fixed). (1) REVTeX's section numbers (revtex4-2.cls:6528-6551 `\secnums@rtx`, revtex4.cls:5163-5172; KPE
  #1027; ~3,050 of the 30k census): "I", "A", "1", "a", `\ref` "I A 1" through `\p@<level>` with thin spaces,
  paragraphs numbered (`secnumdepth` 4), heading tags `\the<level>.` (aps4-2.rtx:288) via `\fnum@<level>`, `\theequation`
  with REVTeX's prefix, the class's `\appendix` (subsections `\arabic`, equations "(A1)", `\appendix*` "(A.1)");
  `rmp`, `aapm` (4.2), `secnumarabic`, `eqsecnum` as their `.rtx`/class code — revtex4_support_sty.rs
  `define_section_numbers`, called by the revtex4/revtex4-2 and revtex4-1 bindings; AASTeX (and emulateapj through it)
  keeps its own numbers (`lx@revtex@sections@by@class`), REVTeX 3 is unchanged. (2) IEEEtran's (and ieeeconf's)
  `\appendix[<title>]` is one appendix (IEEEtran.cls:5719-5745): refnum "A", tag "Appendix", the title, subsections
  from 1 (rev78c2 probe i2: a "[Proof]" paragraph and a "B." subsection before); the class's discarding of a following
  `\section`'s argument is not ported — an unstarred `\section` right after `\appendix` is the appendix, a later one
  another appendix (OD #793; KPE #1022 residual closed). (3) subcaption's sub-floats set `\hsize`, `\textwidth`,
  `\columnwidth`, `\linewidth` from their width as the minipage they are (subcaption.sty:70-92; one helper
  `set_minipage_width`, the minipage's too; `\subcaptionbox` without a width changes none, caption.sty:462-464; KPE
  #1028; witness 2509.19464 A1.F6 397.5pt → 111.3pt/139.1pt as pdflatex; subfig's `\subfloat` and subfigure.sty set
  an `\hbox` and change no width). (4) mdpi's caption settings (mdpi.cls:1362-1373, "Figure 1. " bold small) when no copy
  is shipped (quality ledger 132); a shipped copy, read after the binding, set them already — all 22 census papers ship
  one and printed "Figure 1. " on int45b; the study's witness 2302.03012 converts its arXiv main file, which is
  `article` ("Figure 1: " correct), so the item reaches no sampled paper. (5) A parameter text's closing `#{` extends
  the last delimiter (tex.web §473-476; KPE #1029): `Until:){`, the brace put back after the match, `read_until`
  matching a delimiter's `{` (gullet.rs), `\meaning` "(#1){->…{"; "Expected a { here." with one brace; witness
  2610.26441 (tkz-euclide `\def\tkz@DefPoint[#1](#2)#{`) 9 Errors → 0. Guards changed for the intended output:
  `50_structure` figure_grids golden (110 sub-figure image widths: `0.8\columnwidth` is the panel's), `perfect_kernel_batch57::
  collapsed_panel_keeps_the_float_geometry` (110.4pt, the subfigure's `\linewidth`), `perfect_kernel_batch67x::
  ieeetran_appendix_clears_sectiondis` (the "Appendix" heading, ids `A1.SS1`), `perfect_kernel_batch77r::
  revtex_hyperref_loaded_twice_keeps_cleveref` ("I."). A/B vs int45b (release, `--preload=ar5iv.sty`; 248 papers: 40
  revtex4* with `\subsection`, 17 IEEEtran `\appendix[`, 40 with a width in a sub-float, 20 mdpi, 20 tkz-euclide, 100
  random of the 30k census, 12 witnesses incl. 2610.26441): E 37 → 28, F 0 → 0, W 3,019 → 3,018, 0 papers worse; 118
  outputs changed — revtex in 37 + 22 random + 2610.08787 (heading tags, refnums, `toctitle`s, paragraph ids `Px` →
  `P`, appendix equations "(A1)" in 25 papers), IEEEtran/ieeeconf appendices in 13 + 2102.02632, 2002.08586, 1907.08915
  (its `\appendix \label` now on the appendix), sub-float widths in 32 + 9 random + 4 witnesses (graphics and TikZ;
  some papers in two groups), 2610.26441 (its pictures drawn); no text word
  lost outside numbering tags (the ±1 are the stray "*" of `\appendix*` gone and "[Proof]" brackets), formulas
  unchanged but their ids. Inits 0/0, dumps identical to int45b's. That "0 papers worse" was wrong (rev78d): the A/B
  sampled neither an IEEEtran `\appendix` followed by a blank line (16 of 1,923 IEEEtran papers; r1 made an empty
  appendix "A" and the titled one "B") nor a PRL appendix, and judged 1509.05768 (PRL, unnumbered in its PDF) correct
  with r1's numbers; PRL papers gained paragraph tags and "(A1)" appendix equations (1411.2258, 2606.23684,
  2404.18827, 1508.03506). Round 2 (rev78d findings 1-10): (1) IEEEtran `\appendix` without a title takes the first
  unstarred `\section` past blank lines, spaces, comment lines and `\label`s (which then label the appendix) as the
  appendix; anything else next (`\section*`, `\subsection`) follows the empty "Appendix" heading pdflatex prints;
  witnesses 1903.01287, 1103.2068, 2002.03484, 2410.15995, 2401.07123 one appendix "A" again. (2) REVTeX's PRL journal
  unnumbers the headings (`\c@secnumdepth=-\maxdimen`, aps4-2.rtx:587, aps4-1.rtx:564, revtex4 aps.rtx:517): the
  journal read from the class options as the class reads them (`revtex4_support_sty::selected_journal`: first given in
  4.1/4.2, last declared in 4.0); appendix equations count on ("(16)", 1508.03506's PDF). AIP's `cp`
  (aip4-2.rtx:1181 `secnumdepth` 0) is dead code in the class — its `\ifx\@journal\jnl@cp` is read before the journal
  is selected — so `cp` stays numbered (pdflatex; no `cp` paper in the census). No other society file sets
  `secnumdepth` or the numbers. (3) `\appendices` after `\appendix` restores the kernel's appendix tag and numbers from
  "A" (pdflatex "APPENDIX", "APPENDIX A", "APPENDIX B"). (4) A later `\section` after `\appendix` is tagged "Appendix
  B", "C" as its refnum (OD #793 records that pdflatex's refs to it read "A"). (5) `\subcaptionbox[<width>]` is a
  `\parbox`: `\hsize` and `\linewidth` only (`\lx@subcaption@parbox`). (6) `\meaning` of `\def\b#1#{…}` is
  "macro:#1{->…{". (8) gullet.rs tests the BEGIN catcode first. (9) the `token_kernel_gaps` and `batch67x` guards
  assert whole elements. Open (7): a `\ref` to an `\appendix*` appendix prints "Appendix" where pdflatex prints
  nothing — the scan reads the role-less heading tag as the refnum and the post-processor never renders a `\ref`
  empty (KPE #1027). Guards extended: `revtex_sections_number_as_the_class` (`aps,prl` in 4.2/4.1/4.0, `prb,prl` in
  4.2/4.0, `aip,cp`, PRL's appendix element), `ieeetran_appendix_with_a_title_is_one_appendix` (blank line, comment
  line, `\label`, `\section*`, a later `\section`, `\appendices`), `subfigure_sets_the_line_width` (the
  `\subcaptionbox`), `hash_brace_extends_the_last_delimiter` and `kernel_def_hash_brace_delimiter` (`\meaning` of
  `#1#{`) — each new case red on 78d r1. A/B r1 vs r2 (release, 332 papers: the 248 above, rev78d's 40, the 16
  blank-line IEEEtran, 30 PRL with `\appendix` or `\paragraph`, 10 AIP): E 195 → 195, F 0 → 0, W 3,683 → 3,683; 45
  outputs changed — 34 PRL papers lose their heading numbers and keep their equation numbers counting (2501.04107
  checked against its PDF), 10 IEEEtran appendices as above (2507.18220 gains the "Appendix" heading before its
  `\section*`), 1909.01124 ids only. int45b vs r2 over the same 332: E 204 → 195, F 0 → 0, W 3,684 → 3,683, 180
  changed. rev78d2 (SHIP after two fixes): that set sampled no IEEEtran `\appendix` followed by an empty group, a
  non-printing command or a float — 2002.07641 (`\appendix{}`), 2404.01875, 2007.07179 were worse than int45b (an empty
  appendix "A", the titled one "B"); the main-session follow-up skips empty groups (guard row "an empty group", red on
  r2), the rest is OD #793's recorded residual. PRL headings take the unnumbered ids `Sx1`/`Ax1` (KPE #1027).
  **Integration (int46, 78d r1-r2 + follow-ups squashed cd808dabf3; binary int46a):** gates int46a: nextest 6620/6620 +
  cortex 10/10, clippy/rustdoc clean, inits 0/0, dump diff vs int45b empty. Population A/B (494: the 78d rounds' 326,
  rev78d2's 22, 146 random of the 30k sample) int45b vs int46a: E 234 → 234, F 1 → 1, 0 Error or Fatal worse; 174
  outputs changed (22 of the controls); 7 papers lose > 1 % of their words, every one numbering — PRL heading tags,
  "§"/"Appendix" typed refs and the `toctitle` copies of unnumbered PRL headings (0901.1919, 1404.6478, 1609.03541,
  2405.02277, 2601.16391, quant-ph0009002, 2605.18605), no authored word lost. Binding nets (int46a) 136/136, 0
  documents worse than int45a.
- **78e** (quality batch 8, the sel78d selection study's items 2, 4, 5 and 7; guards `perfect_kernel_batch78e::*`, 5,
  each RED on 77e6fd7f85 (int45b), 10 after r2; OD #800-804). (1) webofc.cls (Web of Conferences, shipped; ~2,900 papers): a
  binding reading the class raw with inst_support's `\institute`/`\inst`, its `\author` split at `\and` and `\\`, the
  e-mail the author's contact, `\sidecaption` layout only (the html_feedback report on 2610.23498: its side captions are
  `<caption>`s, their numbers no longer stepped twice — 1305.2348's "Figure 3" is the PDF's "Figure 2"), the class's
  caption label "**Figure 1.**"; the kernel's frontmatter `\fnmsep` rule covers webofc's `\fnsep` (2301.07438 "Andrea
  Shindler,1" → "Andrea Shindler"; 1411.7792 each author its own institutes). (2) jinstpub.sty (JINST, shipped; 23 of
  the 30k sample): jheppub's binding with JINST's extras (`\note[]`, `\collaboration[t]`, section-numbered equations,
  the caption label; 2006.05543, 2303.10224: the affiliations and e-mail contacts, "Corresponding author." out of the
  personname — 7 papers lose a false creator); both bindings are author-file
  stand-ins (`AUTHOR_FILE_STAND_INS`), so jinstpub's copy is read after it (OD #731) and keeps its page geometry. `caption_sty::
  class_makecaption_label` is the class-`\@makecaption` label format both share. (3) vmsta.cls: the e-mails (lost) and
  addresses (title-page paragraphs) are the authors' contacts in the 2014-2018 copies (`\@email`, `\@@address`) and the
  2018+ ones (`\ead`, `\address@optarg`), labelled addresses by the authors' labels, the class's own `\author`
  replayed for its counters (1508.02842, 1801.00977, 1411.4499, 1805.02511, 2001.02487; 9 of 9 witnesses). (4)
  tcolorbox theorem labels (quality ledger rank 124): STOPPED, not brief — recorded at the tcolorbox `label=` row
  (A2 table). Ledger rows the study re-verified fixed on int45a, closed: #17 IEEEKEYWORDS-BLOCKS (77t), #59
  VMSTA-NAMEPARTS (77t), #34 JINST re-scoped to (2), #154 AMSTEX-TAGS-LEFT, #163 ACM-NUMERIC, #142's `\hhline` part,
  #162's jmlr part, #24/#35's `@"` part (78c). A/B vs int45b (release, `--preload=ar5iv.sty`; 193 papers: 5 witnesses,
  the 30 webofc, 23 jinstpub and 9 vmsta papers, 30 tcolorbox `label=`/`\ref` papers of the 30k sample, 100 random of
  it): E 48 → 48, F 0 → 0, W 1,699 → 1,687 (1305.2348's 11 webofc "No sufficient room for the legend" Warnings of the
  raw side caption, 2601.11288's open-group Warning), 0 papers worse; 63 outputs changed, all 62 target papers and 2610.23498, 0 of
  the tcolorbox and random controls: creators' affiliations and e-mails, side captions, caption labels — no affiliation
  or e-mail text lost, the removed words the superscripts and "e-mail:" labels, the JINST header line, the doubled vmsta
  e-mail line. Residuals: 2603.09495's TikZ figure is 0.5 % shorter than read raw (226.5 → 225.3pt; likely the float's
  size including its caption box, whose label format changed: 2103.15405's rotated figure `innerdepth` 26.0 → 23.8pt,
  width 306.4 → 304.2pt, the rev78e review's finding 11); the 2018+ vmsta `\thanksref{cor1}` key in the name (row 17 of
  the A2 table). **r2** (the rev78e review, NO-SHIP; its round-1 claims "the e-mails on the right creators" and "0
  worse" were wrong for jinstpub): (a) jinstpub's `\emailAdd` took jheppub's n-th-address-to-n-th-author rule, which
  put 16 of 23 papers' addresses on the wrong authors (2311.16063 bross@fnal.gov on Andrea Bersani; 2307.12991,
  1910.06438, 2311.00870, 2605.30709) where JINST's `\maketitle` prints one shared "E-mail:" line: an address is now
  placed with the author it spells, else the group's only author, else the author block (`\lx@add@contact@placed`),
  an `\emailAdd{a, b}` list an address each, the corresponding author's `\note{…}` no part of the name it spells (the
  kernel's `is_note_macro` takes a macro whose arguments all become frontmatter annotations as a name annotation:
  2406.08712, 2603.01867); jheppub keeps Perl's rule. (b) Ruling 2026-10-21: a frontmatter note all
  of whose content became contacts is not emitted (`\lx@annotate@frontmatter@now`, every class: webofc's
  `\fnsep\thanks{\email{…}}`, aa's `\fnmsep\thanks{\email{…}}`; 2409.01693's "†" opening an empty note; OD #803); an
  author's own empty `\thanks{}` stays. (c) A jinstpub copy's own header line (`\gdef\@fpheader{…}` other than the
  stock "Prepared for submission to JINST") is a preprint pubnote (2103.06391 FERMILAB-PUB-21-079-ND-SCD, 2205.10526,
  2311.06155, 2512.23612). (d) jinstpub_sty.rs and jcappub_sty.rs moved to latexml_contrib (Perl has neither).
  (e) The repro stand-ins: jinstpub.sty keeps SISSA Medialab's copyright and LPPL 1.3 notice and says it is a
  modified, shortened copy; webofc.cls (no licence stated) is a minimal stand-in written from scratch. (f) A split at
  `\\` drops the break's `*`/`[…]` from the next piece (`\lx@splitting`: webofc's `\author`, llncs's `\inst` list; OD
  #804). (g) webofc's caption label period is bold ("**Figure 1.**", webofc.cls:786-812). (h) jinstpub drops the
  table of contents as jheppub's binding does (main session ruling: one rule for the family; the user ruling on
  printing it is the silent-loss table's JHEP/JINST row). Guards +5 (`jinstpub_emails_go_to_the_authors_they_spell`,
  `jinstpub_edited_header_is_a_preprint_note`, `webofc_author_line_break_options`,
  `llncs_author_line_break_options_with_inst_marks`, `aa_thanks_email_leaves_no_empty_note`), the round-1 guards
  extended (whole-element equation and table caption, no table of contents, no empty note). A/B (release,
  `--preload=ar5iv.sty`) over 390 papers (the 193 above, the review's 78, the 91 papers of the 30k sample with
  `\thanks{…\email` — 27 webofc, 16 aa, 6 llncs, 8 article, 34 other —, and its 28 jheppub/jcappub papers with a
  `\note` in an `\author`): int45b → r2 E 58 → 58, F 0 → 0, W 3,242 → 3,217, 0 papers worse, 163 outputs changed
  (none of the 28 jheppub/jcappub controls); r1 → r2 E, F and W equal, 140 outputs changed (113 papers): 133 empty
  thanks notes dropped (each note's whole content a contact, one only a space: 2308.11493), 28 jinstpub tables of
  contents dropped, 20 jinstpub papers' e-mails re-owned — of the 44 addresses on the 32 jinstpub papers (the
  review's 2311.12551 is an iopart paper), 39 are on the author they spell, 1 on 2103.11983's sole creator (its
  collaboration) and 4 on the author block, as the PDF's shared line leaves them (2605.30709 microboone_info@,
  2409.18288 sungbino@ and 2211.07892 yoomin@ — S. Oh's and Y. M. Oh's, a two-letter surname the spelling does not
  score —, 2211.05026 yannick.mueller@ — Y. Müller's, "ue" for "ü" unread), none on a wrong author —, 13 webofc caption labels only (the bold period), 1 header
  pubnote (2512.23612's "Preprint"). Residuals recorded in the
  silent-loss table: a jinstpub/jheppub copy's own `\maketitle` lines (2103.11983's DOI line); jinstpub/jheppub
  `\collaboration`/`\proceeding` pubnotes hover-only in LaTeXML.css.
  **Integration (int47, 78e r1-r2 + follow-ups squashed cd7a8c6d31; binary int47a):** rev78e2 SHIP; its nits applied in
  the main session (the test harness's contrib dispatcher narrowed to the jcappub test, `convert_to_xml_contrib_clean`;
  OD #803 scoped to `ltx:note` with its separator-note residual; silent-loss rows for the jheppub e-mail rule, aa
  2401.16957's comma before `\inst`, `\thanks{\label…\email…}`, iopart `\ead`). Gates int47a: nextest 6630/6630 + cortex
  10/10, clippy/rustdoc clean, inits 0/0, dump diff vs int46a empty. Population A/B (541: 78e's 208, rev78e's 83,
  rev78e2's 100, 150 random of the 30k sample) int46a vs int47a: E 154 → 154, F 1 → 1, 0 Error or Fatal worse; 108
  outputs changed; 11 papers lose > 1 % of their words, every one webofc/jinstpub frontmatter — the institutes no longer
  repeated on every author and the "e-mail:" labels of addresses now contacts (no affiliation word lost; 1309.3933,
  1411.7792, 1412.3488, 1609.01040, 1609.05029, 1710.06250, 1711.00130, 1711.05413, 2009.04166, 2103.01871, 2105.10529).
  Binding nets (int47a) 136/136; mathastext 1 of 26 below int46a: 2201.00699 (jinstpub) recall 98.0 → 97.4 — the stock
  header line, the table of contents (OD #801, ruling pending) and the "E-mail:"/"Keywords" labels no longer printed; its
  arXiv number stays a `role=arxiv` pubnote; no authored word lost.
- **78m** (fidelity audit round 3, items FM1-FM4 and N9, agent_reports/2026-10-21_fidelity3_audit.md; guards
  `perfect_kernel_batch78m::*`, 8, each RED on int50a or on r1; OD #840-842, KPE #1067-1069; r1 + r2). (1) A shipped
  spie.cls that loads authblk (SPIE's 2015 v3.4, its :31) loads authblk in the spie binding too, so
  `\author[\udem]{…}`/`\affil[…]{…}` are authblk's, not the kernel's one-argument `\author` 77w installed (spie_cls.rs;
  one creator "[" and the names in the body: 2406.08304, now its 114 authors with their affiliations; 28 of the 40
  census spie papers changed, each from one creator "[" — or the kernel's last `\author` only, 2608.07857 — to its
  authors; 2210.16202, 77w's witness, unchanged). (2) A horizontal space right before a name's own mark or note is the
  name's, no separator (rows.rs `space_before_own_mark`: `Oleg L.\thanks{…}\hspace*{2mm}$^{\rm a,b}$, Sergei B.`,
  0905.3658, had parted ", Sergei Blinnikov" off as an affiliation; 1811.08180, 1906.01604, 2402.15911 likewise); a
  spacing-only math group (`$\qquad$`, 1602.04504, 1104.2983, 2305.13533, 1904.08825) and an `\enskip` right after a
  mark or note (1312.1414) part names, one `\enskip` elsewhere staying inside a name as #52(o)'s other spacing commands
  do (`J.\enskip Smith`; r2, r1 parted it); a tie or control space right after an entry's mark is the entry's
  (`$^1$~Univ A\quad$^2$~Univ B`, r2: r1 gave Ann "Univ A² Univ B"; 2606.01993, 2402.19080 byte-identical to int50a) (OD
  #840, KPE #1069); a mark's `\rm` glued to its label is stripped as Perl's `s/\\rm//g` does (queue_digest.rs; `$^{\rm
  a,b}$` requested "\rma"). (3) jpconf.cls (IOP's conference class) is read raw, then its
  `\address`/`\ead`/`\eads`/`\mailto` are iopart_support's (jpconf_cls.rs; `iopart_support_sty::iop_address_and_email`;
  OD #841, KPE #1068): 0705.0202, 1011.0523 and 55 of the 56 census jpconf papers gain their affiliations and e-mails,
  which were body lists. (4) bmcart.cls is read raw, then its `\title@fmt`, `\author@fmt` (its
  `addressref`/`presentaddressref`/`email`/`noteref` keys), `\address@fmt` (its `id`) and `\@bmcartnote` (`{artnotes}`'s
  `\note[id=…]{…}`, a `note:`-labelled note contact, r2) are the frontmatter and `{fmbox}` only holds them
  (bmcart_cls.rs; OD #842, KPE #1067; not the `frontmatter_stores.rs` reroute the brief named — none existed for bmcart,
  its printers are no pure stores): 1711.01177 0 → 10 creators, all 19 census bmcart papers gain their title, authors,
  e-mails and affiliations. (5) The pst-plot binding requires pst-xkey and multido as pst-plot.sty:29-31 does (0711.2399
  6 → 1 Errors, the one left the elsarticle v1 copy's `{comment}`, audit N10; 1406.1992 6 → 0). A/B (release,
  `--preload=ar5iv.sty`, int50a vs 78m; 918 papers: the 370 scan.py `spacemark` shapes, 223 of the `author_spacing_sep`
  feature, 40 spie, 56 jpconf, 19 bmcart, 101 pst-plot, 99 controls, the 10 witnesses): E 1,946 → 1,936, F 3 → 3, +6,468
  words, 121 changed (byte comparison; 1912.00500 a timeout on both sides), controls 0; r1 → r2 (922 papers, the 918 and
  2606.01993, 2310.12072, hep-th0310140): 4 outputs changed — the article notes of 1507.06515 ("Corresponding author"),
  2007.00437 ("Second corresponding author: …") and 2112.10068 ("Equal contributor", on both its authors), which r1 had
  dropped silently from its `{artnotes}` (the review's blocker), and 1712.06536's University of Bath now on Neill D. F.
  Campbell (its `$\quad$ \textsuperscript{$\dagger$}~University of Bath` entry, r1 a tail of the "*University of
  Bristol" line); 2310.12072's `\enskip\enskip\enskip` runs part its four names (int50a one creator, r1 alike); every
  changed output read against its source: authors, affiliations, e-mails, notes and titles gained; the scan's other
  shapes (a mark-led affiliation line, `$^1$Univ A\quad$^2$Univ B`) unchanged. Worse: 1312.1414 0 → 1 Error, the honest
  `merged_creators` flag on "Richard Cleve Robin Kothari Rolando D. Somma" (base: one creator "Dominic W. Berry Andrew
  M. Childs" with "Richard Cleve" as its affiliation and a nameless one, silent); 0711.2399's 42 fewer words are
  `\multido` error ink. Repro A/B (968 sectioning-frontmatter repros, class-bindings with its 53 copy directories,
  stub-dependencies, package-bindings): only this batch's 3 repros changed. Residuals: in a block marked with labels a
  name whose only mark is a footnote symbol continues the line before it (`Bob Baker$^{2}$\quad Cat Cole$^{\S}$`, the
  base too; 1312.1414) — FM S2b; a two-name block `Ann Able\thanks{…}\hspace*{1mm}$^{,}$\thanks{…} \enskip Bob
  Baker\thanks{…}` (a comma-only mark, 1312.1414's shape) is one creator, r1 alike (1312.1414 itself parts its names);
  iopart_support's routing, now jpconf's too: an `\address` of an unmarked block goes to the last author (1110.0674's
  three authors under one address), an e-mail no name spells to a nameless creator (1309.7930, 1510.00572, 1609.09832,
  2111.06834), a `\boldsymbol` in a mark (2010.05094) and a `\label` in an `\address` (2211.03667) leave the mark
  unanswered — the same under `\documentclass{iopart}`.
- **78p** (quality batch 13: the SYNC tabularray row and the fidelity3 audit's N12, N14, N16-N19,
  agent_reports/2026-10-21_fidelity3_audit.md; guards `perfect_kernel_batch78p::*`, 16, each item RED on int52a; OD
  #855-859, KPE #1091-1096). (1) tabularray's rule keys are drawn: `hlines`/`vlines`, `hline{…}`/`vline{…}` with their
  `{<index>}{<cross>}{<style>}` values and a rowspec's `|` become the cells' `border` letters as `\hline` and `|` give
  them, against the body's shape (after `evaluate=`/`expand=`), at the table's end (`\lx@tblr@rules`, expandable); an
  all-empty grid is kept; no hline through a `\SetCell[r=…]` span; `text=` and white rules draw none (OD #855; the
  tabularray row's rules item closed). (2) `\patchcmd` patches the texts etoolbox writes (`\meaning`/`\detokenize`, a
  control word always followed by a space): `\patchcmd{\x}{|}{\mid}` keeps `\mid` before a letter, a search `\foo@bar`
  no longer matches inside `\foo@barx` (etoolbox_sty.rs `etb_patch_text`; 2011.13897; KPE #1091). (3) A `\label` after
  a counter step a group has undone climbs from the current node, not from the element just closed: after an inner
  `equation` in a theorem it names the theorem (math0107195 "Theorem 5.3"; 0902.3453 `lem:bias` 3 → 6 as pdflatex), after
  a `subequations` block's inner `align` the block's number (2505.03288 `eq:GNG` → 13), after a closed table, figure or
  list the enclosing unit; every step records its serial in scope and as the latest (`label_stepped_counter`); caption's
  `\captionof`, whose step LaTeX makes in the current group, keeps its float (`\lx@refstep@outlives@group`; 2402.05535);
  where no ancestor takes the label Perl's last-child/root fallback stays (lastpage's `\AtEndDocument` label; the root
  taking it renumbered every generated id) (OD #856, KPE #1092). (4) delarray delimits a `tabular` as an `array`:
  `\begin{tabular}[t]\{{l}.` is the tabular in a formula between `\left\{` and `\right.` (delarray_sty.rs; 1304.5109 6
  Errors → 0, 1106.2670 4 → 0; OD #858, KPE #1096). (5) mathabx `\curvearrowbotleftright` is U+293B (OD #857, KPE
  #1095; 2308.12945). (6) An unused glossary entry's definition raises nothing: the definition digest's diagnostics
  are deferred (`defer_to_label`, `glossary:<key>`) and replayed only for an entry referenced in a document that prints
  a glossary (2104.09252, 2505.18389 18 Errors → 0; OD #859, KPE #1093). (7) `multicols` inside `algorithmic` makes no
  pagination marker the `ltx:listing` cannot hold (`construct_multicols`, `pagination_insertable`; the run-336
  `malformed:ltx:pagination` cluster, 38 papers, 94 Errors → 0, e.g. 1708.03274, 1405.5689: a `multicols` in
  `algorithmic`, `\columnbreak`/`\newpage` its break; KPE #1094). A/B (release 78p vs int52a, int52a dumps,
  `--preload=ar5iv.sty`, census populations `census/pops/pop78p_*.tsv`): 942 papers (mathabx 1, delarray 36, the run-336
  pagination cluster 38, `\columnbreak` 61, tblr 56, glossaries 200, theorem+`\label` 200, `\patchcmd` 250, 100 random
  controls) E 7,818 → 7,636, F 74 → 74, W 19,470 → 19,469, 0 papers worse; 133 outputs changed (129 papers): 32 tblr
  papers' borders (and the heights they add) only; labels moved in 52 papers — into theorems from their inner
  equations, onto `subequations` blocks, onto the enclosing section after a closed table/figure/list, `sec:`-named
  labels onto their sections — each checked against its source and, where referenced, pdflatex's `.aux` (0902.3453
  `lem:bias` 6, 2505.03288 `eq:GNG` 13, 2510.08379 `AgeSex` 2.1, 2106.05350 `tab:hyperparams_gan` 5, 2312.09748 `opt`
  II-B, 1809.06638 `fig:failure` the figure — all now as pdflatex; 2103.17127 below); the rest the Error removals
  above. Residuals: tabularray's body
  `\SetHline`/`\SetHlines`/`\SetVline`/`\SetVlines` stay no-ops, rule widths/dashes/colours unrendered; cleveref's
  `\lx@cleverref@labeltype` placed a `\label[type]`'s type by `float_to_label_for` (done in 78j r5: `float_to_label_scoped`); supertabular's `\tablecaption{…}\label{…}` before the table
  names the enclosing unit (pdflatex the table: the binding steps the counter at the table, not at `\tablecaption`,
  2103.17127; int52a named the previous table); `\left\{ … \right.` around a text tabular stays unparsed math with a
  Warning (math parked); an unused entry's unparsable formula still warns from the math parse (2104.09252).
  **Integration (int54, 78p squashed 31d4f6f612 onto int53; binary int54a; 78j moves to int55 after its r4):** review rev78p
  SHIP — the label-scoping rule never turned a `\ref` that matched pdflatex into one that does not (≈103 synthetic labels, 78
  moved labels from real papers checked against pdflatex `.aux`). Folded in by the main session: `etb_patch_text` writes a space
  token as one space whatever its source text (a line end kept `\n`, so a `\patchcmd` search across a source line break never
  matched; probe int53a FAIL → int54a OK, as pdflatex), and OD #856 no longer claims `\subfloat{…}\label` names the figure when the
  caption follows the subfloats (1504.02206). Open from the review: cleveref's `\label[type]` placed its type with
  `float_to_label_for` (switched to `float_to_label_scoped` in 78j r5); supertabular steps `table` at the table, not at
  `\tablecaption` (2103.17127, wrong before and after); a guessed header row now depends on the drawn tabularray borders (2206.04438
  7 → 3 tables). Gates int54a: nextest 6797/6797 + cortex 10/10, clippy/rustdoc clean, inits 0/0, dump diff vs int53a empty. A/B
  int53a vs int54a (1,244 of 1,246: the `pop78p_*` populations + 300 random census controls, seed 54): E 7,222 → 7,044, F 71 = 71,
  0 papers worse, 60 better, no word loss; 7 of 290 random controls changed, all label moves pdflatex makes the same way (1311.4795:
  lastpage's end-document label names the appendix, not the closed figure before it). Binding nets (int54a) 136/136, 0 worse than int53a.
- **78i** (quality batch 10, the sel78i selection study's five non-frontmatter items; guards `perfect_kernel_batch78i::*`,
  9; OD #823-825, KPE #1048-1052; quality ledger rows #155 BIBLATEX-ONLY-FIELDS and #37 AMSTEX-LOCALSTY closed). (1)
  biblatex's own `.bib` fields (`journaltitle`, `location`, `urldate`, `addendum`, `maintitle`, the event fields, …: 47
  handlers, bibtex.rs `define_biblatex_fields`) are read by the biblatex binding and, on the BibTeX route, only where the
  document's `.bst` declares them in its `ENTRY` list (`bst_declared_fields`; ACM-Reference-Format's `location`,
  sn-mathphys's and mdpi's `urldate`); elsewhere raw `bib-data` as in Perl: 2602.10008's 54 and 2606.11822's 10
  "Accessed:" lines and the duplicated event titles gone (OD #823). (2) thm-restate: `\<name>*` prints the stored
  statement again with the original's number, labels gobbled, the equation counter put back; `\<name>` a new numbered
  theorem; the `restate=` key stores the same way (Perl "See ⟨ref⟩", KPE #1048; 2108.11527, 2602.09462, 2402.14202;
  `perfect_kernel_batch77v::thmtools_restate_key` and `perfect_kernel_batch66h::lipics_thm_restate_option_loads_restatable`
  now pin the restated statement). (3)
  A minipage's `\footnote`s are lettered on `mpfootnote` (latex.ltx:16323, :17638, :17649), the page's notes go on
  unshifted; `\footnotetext` keeps to `footnote` so it pairs with its `\footnotemark` (2304.06412; KPE #1049; 0802.0707).
  (4) hyperref's information keys are expanded as `\pdfstringdef` expands them when `\hypersetup` sets them
  (`\lx@pdfstringdef`, `pdf_string`: notes gobbled, markup dropped, accents composed; OD #824, KPE #1050; 1210.7805).
  (5) AMS-TeX's `\documentstyle{X}` reads a paper's own X.sty raw unless it `\input`s another style (fic.sty,
  grubb-p.sty keep the amsppt stand-in); amstex.tex's `\Invalid@` (OD #825, KPE #1051; math0402285, math9806089,
  math0101270: 0 → 5,826 / 34,174 / 32,200 words, the Fatal gone). A/B (release, `--preload=ar5iv.sty`, int47a vs 78i;
  1,695 papers: the 476 `.bib`-without-`.bbl` papers and 200 of the 2,307 `.bib` papers, 200 biblatex, 250
  restatable/thm-restate, 52 + 150 minipage-`\footnote`, 64 `\hypersetup` pdfinfo-macro and 100 hyperref papers, 152
  AMS-TeX papers, 100 random controls, the 12 witnesses): E 2,946 → 3,044, F 24 → 21; 467 outputs changed, of the controls
  only minipage-footnote papers and one hyperref triple. Every changed output read against its PDF: 210 bibliographies
  lose fields their style does not print (the paper's `.bst` checked: none declares a dropped field), the declared ones
  kept; 437 restated theorems where "See ⟨ref⟩" stood; minipage notes lettered as pdflatex letters them (mn2e's
  `\thempfootnote` too); 151 raw-TeX `rdf` triples now text, 27 now empty (a class's own author storage our kernel's
  `\author` does not fill: LIPIcs `\@authorsforpdf`; a `\hypersetup{pdftitle=\@title}` before `\title`, empty in the PDF
  too). Error rises, each root-caused: the amams papers now read past their title, and what errs there is our
  emulation, not amams.sty (`tex` converts math9906120 clean; SYNC "AMS-TeX papers read through their own amams.sty"):
  math9906120 5 → 102 (Fatal both, 0 → 1,209 words; 43 "Use \Bbb only in math" and 43 "Script ^/_ can only appear in
  math mode" where amams's `\everydisplay{\lookforbreak}` (amams.sty:585-590) meets amstex's `\multline`,
  JohanssonArchive.tex:250), math9806089 8 → 13 (no longer Fatal; amams's `\split` over amstex internals the pool lacks,
  `\ifinany@`; boxedeps.tex, shipped by neither the paper nor TeX Live), math0101270 6 → 8 (no longer Fatal; our
  amsmath `\smallmatrix`'s `\scriptsize`, plain having none), math0402285 5 → 0 (the kernel's tests use `\lx@ifdefined`,
  etex.rs, which amams.sty:1099's three-argument `\ifdefined` cannot reach); 1711.05227 55 → 58 (the `\thmrelevance*`
  replay at proofs.tex:45 inside a math frame left open at proofs.tex:23 and an alignment open since magic.tex:150-155,
  its error locations moving to `\end{restatable}`). r2 (review round; guards
  `perfect_kernel_batch78i::{restatable_star_takes_the_later_number, kernel_ifdefined_survives_a_document_redefinition}`,
  11; KPE #1052): PDF strings define puenc.def's text glyphs as their characters and compose an accent on `\i`/`\j` as
  on i/j ("Martínez, García, Straße", as pdfinfo; r1 "Martınez, Garcıa, Strae"); `{restatable*}` prints its theorem
  unstepped, its labels gobbled, numbered by reference to the later `\<name>` (1702.04225's previews "Theorem 6.21",
  "Proposition 6.13", "Theorem 6.24", as the PDF; r1 "Theorem 1.1…1.8"; 1407.7827, 1910.14249, 1911.00646, 2004.07319,
  2402.00791, 2404.17677, 2508.18759); the kernel's own `\ifdefined` tests (`\lx@latin@name`, the frontmatter stores'
  nulls, `\makeindex`, `\makeglossary`) use `\lx@ifdefined` (`\loggingall`, `\tracingnone` keep the public
  `\ifdefined`, as latex.ltx:662/:679 and Perl do). A/B r2 (release
  15346dafb8; 1,727 papers: r1's 1,695, the 31 census papers whose hyperref values hold `\i`, `\ss`, `\o`, `\ae`, `\l`
  or an accent, the papers defining `\ifdefined` — none in the 30k census, hep-th9410188, q-alg9412016 and math9806089
  in the projection sample, math0402285 and math9906120 —, the 8 `{restatable*}` papers): r1 → r2 E 3,699 → 3,686, F 21 → 21, 0 worse, 3 better (math0402285, math9806089,
  math0101270), 11 outputs changed (2 `rdf` triples, 7 `{restatable*}` papers, 2 amams papers' note ids); base → r2 E
  3,601 → 3,686, F 24 → 21, 493 changed, 4 worse (math9906120, math9806089, math0101270, 1711.05227, above). Residuals:
  after a `{restatable}` a `\<name>*` before a later `\<name>`, which pdflatex numbers it by from the `.aux`, prints the
  original's number; a `{restatable*}`'s equations are numbered on the current counter; a `{restatable*}` no `\<name>`
  follows prints an empty missing-label reference with a Warning (pdflatex "??"). r3 (review follow-ups; guards
  `perfect_kernel_batch78i::{hyperref_metadata_text_symbols, restatable_star_takes_the_last_number}`, 13;
  `perfect_kernel_batch56::restatable_star_and_lineno_bframe` 0/0 with a later `\mainthm`, the whole elements): PDF
  strings also take puenc.def's text symbols (`\textendash`, `\S`, `\dag`, `\copyright`, `\pounds`, `\ldots`,
  `\guillemotleft`, `\textquoteright`, …) and hyperref's `---`/`--` dashes (hyperref.sty:620-621): "A–B, §2, †, ©",
  "Pre–post—end", as pdfinfo (r2 "AB, 2, , ,", "Pre--post---end"); every `\<name>` sets the label a `{restatable*}`'s
  number refers to (thm-restate.sty:158-160), so the preview is the last one's, as pdflatex; `\loggingall` and
  `\tracingnone` back to the public `\ifdefined` (latex.ltx:662/:679, Perl plain_base.pool.ltxml:40/:56).
  **Integration (int49, 78h r1-r3 squashed 5738b779d3 + 78i r1-r3 squashed c3b28b252d; binary int49a):** 78h rev78h2's
  only blocker (the copy-body check failing open) fixed in r3, guard red on r2; 78i rev78i2 SHIP, its follow-ups in r3. Gates
  int49a: nextest 6688/6688 + cortex 10/10, clippy/rustdoc clean, inits 0/0, dump diff vs int48a empty. Population A/B
  (2,331 of 2,340: 78h's 542, 78i's 1,727 + r3's 546) int48a vs int49a: E 3,760 → 3,848, F 26 → 23, +197k words, no paper
  loses > 1 % of its words; the 4 with more errors are 78i's root-caused ones (math9906120, math9806089, math0101270 read
  past the title through their own amams.sty, 0 → thousands of words; 1711.05227's restatement replays an erring
  alignment). 313 outputs changed: 293 classified by the writers, 15 = 78h's declared names, and 5 checked here — 2104.11163
  (en dash in `dcterms:subject`), 2407.02379 (metadata macros expanded), 2407.14159 (empty creator triple, as Perl), 1805.03240
  and 2202.10894 (footnotes in a minipage / tcolorbox lettered a, b, as pdflatex; no text lost). Binding nets (int49a)
  136/136, 0 worse than int48a.
- **78l** (quality batch 12, the sel78k selection study's items 4, 7, 8; r1-r2; guards `perfect_kernel_batch78l::*`, 9,
  each RED on int49a or on 78l r1; OD #834, KPE #1063, KPE #361 updated). (1) `\textcircled` circles the text its
  argument typesets when that is one run of characters (sect13.rs `typeset_text`: a box's string, a list's pieces, a
  plain formula's body, a whatsit's one shown box-valued argument by base_utilities.rs `shown_arguments`, nothing for
  an `isSpace` whatsit or a rule, the spacing around it trimmed): the `\textcircled{\raisebox{-0.9pt}{1}}` idiom is ①,
  `\textcircled{$\cap$}` ∩⃝, where int49a and Perl printed "\raisebox{-0.9pt}{1}⃝" (2104.12292, 2110.15350, 2207.02277's
  426 ⓚ, 1401.5657, 2501.11185's `\phantom{0}` padding, 2501.14912's `\hspace` spacing); anything else keeps its source
  as before (r2 fail-safe, after r1 circled `$\frac12$` as ② and `$q_0$` as "q0⃝", 2508.14614): a whatsit in a formula,
  two box-valued arguments (`\scalebox{.8}{3}`), an environment; the circled numbers reach 50 (U+3251-325F,
  U+32B1-32BF). Residuals: a `\ref` keeps its reversion, a structured formula or `\scalebox` its source.
  (2) tabularray's `\SetCell[c=<n>,r=<m>]` spans (tabularray_sty.rs `tblr_span_cells`, the body read whole as
  tabularray's `+b`), a `\SetCell` at a cell's start, after any row-level rule command (`\hline`, `\cline{…}`,
  `\SetRow{…}`, a row end's `[…]`/`*`; r2, 2308.16907's 11, 2507.01654, 2502.18449, 2404.12789, 2601.16161,
  2604.10925, 2606.09108): `r=` the cell's rowspan (as `\multirow`), `c=` the covered separators made the primitive
  `\span`, padded when the row ends first (2305.00635), so the columns' rules stay; `halign` and the other inner keys
  stay unrendered (a `\multicolumn` dropped rules: 2408.02133, 2403.06461); open: `cell{i}{j}={r=,c=}` spans,
  `\SetCells`, a row span over a non-empty cell below, a math table's `tex=` after a span. (3) `\caption*` (and
  longtable's) takes caption's `font=`/`textfont=`/`textformat=` through the kernel hook `\lx@format@scaption@@`
  (caption_sty.rs `caption_star_format_tokens`), no label (KPE #1063). r1 A/B (release 78l vs int49a, int49a dumps,
  1,110 papers: every census `\textcircled` paper 274, every tblr-family paper 152 incl. all 25 `\SetCell`/`\SetCells`
  callers, the 187 `\caption*` papers of the 30k + a seeded 400 caption papers, 100 random controls; counted per
  pool): E 3,467 → 3,467, F 8 → 8, W 18,022 → 18,022, 0 worse; 62 distinct papers changed, all classified — 30 circled
  texts, 10 tblr spans, 22 starred-caption fonts (incl. 2112.14209's `font=normalsize` `\caption*` in a `\scriptsize`
  table, now 100 % as pdflatex, and 1511.03137 whitespace only); random controls 0 changed; words −22 (2501.14912: its
  "\raisebox{-0.8pt}{{\hskip-3.1298pt 1}}⃝" source tokens become ①). r2 A/B (release r2 vs r1 and vs int49a, the same 1,110 papers, int49a dumps): E 3,467 → 3,467,
  F 8 → 8, W 18,022 → 18,022, 0 worse; r1 → r2 10 distinct papers changed, all classified — 8 tblr papers gain spans
  after a row rule (2305.00635, 2308.16907, 2311.09782, 2502.18449, 2507.01654, 2601.16161, 2604.10925, 2606.09108:
  53 stray empty cells fewer, words equal), 2 circles back to their source (2508.14614's `$q_{0}$⃝`, 2606.27167's
  `$\bm{e}$⃝`, where r1 printed "q0⃝" and a plain ⓔ); int49a → r2 66 distinct papers changed (r1's 62 less those 2, plus the 6
  new span papers), random controls 0.
- **78k** (quality batch 11, the sel78k selection study's items 1, 2, 3, 5, 6; guards `perfect_kernel_batch78k::*`, 11;
  OD #829-831, KPE #1056-1062; quality ledger #101 BIBENTRY closed, the study's new rows THMTOOLS-NAME, ALG2E-END,
  FOOTMISC-SYMBOL and NATBIB-CITETEXT closed). (1) thmtools: a `\declaretheorem` without `name=` is named
  `\thmt@modifycase{<env>}` (thm-kv.sty:359; `uppercase`/`lowercase`/`anycase`), "Defn 1.2" for "1.2" (RUST-ONLY, OD
  #829; 1505.04294: 52 unnamed heads → 0 of 90). (2) algorithm2e: `\algocf@@@block` takes its three arguments
  (algorithm2e.sty:1868), so a block's end is the bold `shortend` "end" (`longend` "end if", `noend` none) where Perl and
  int49a printed a plain "end if" (KPE #1056; 1810.03652 7 ends, 1911.01815 3; 1903.04631 unchanged). (3) bibentry: a
  binding (latexml_contrib bibentry_sty.rs) reads the `<jobname>.bbl` in store mode for `\bibentry`; `\nobibliography*`
  builds no list (2505.07606, 2507.01709, 2608.05624, 2012.02570, 1211.2618, 2010.07703: the duplicated or misplaced list
  gone), the starless form keeps the list its citations resolve to (OD #831; 2210.16098: 48 lines "." → the entries);
  tufte's margin citations print their entries (2309.03201); bibentry's end warning only when no `\bibliography` followed
  (the kernel's `lx_bibliography_calls`, sect11.rs). (4) footmisc `[symbol]`/`[symbol*]` mark footnotes with symbols
  (KPE #1057; 1907.00006). (5) natbib `\citetext` in the citation brackets (KPE #1058). A/B (release, `--preload=ar5iv.sty`,
  int49a vs 78k; 2,261 papers: 528 thmtools, 500 of 1,819 algorithm2e (seeded), 194 bibentry, 414 footmisc (400 seeded +
  the 26 symbol papers), 415 `\citetext` (400 seeded + 29), the 4 tufte papers, 245 papers with a numbered footnote beside
  a symbol or letter `\thefootnote`, 100 random controls, the 7 witnesses): E+F 13,202 → 13,201, F 24 → 24, 0 papers with
  more Errors or Fatals; 279 outputs changed (4 controls, each an algorithm2e end), every one read against its PDF: theorem
  heads named (and the `\cref`/`\autoref` texts that took the empty name: 2310.05384 "Theory 3.6", 2504.04267 "Problem
  1"), "end" lines, bibentry texts, footnote marks, cite brackets. One paper loses content: 0912.5438's bibliography
  (23 entries, 24 citations now missing) — int49a printed it at the `\nobibliography*` at the top; now it stands where
  the author put it, at the end, which our errors swallow with the conclusion (E 156 in both binaries, ours, not the
  paper's: its shipped 2004 mathtools.sty v1.01a reads amsmath's `\totwidth@`/`\alignsep@`, which our amsmath binding
  does not define, at its `{gather}`/`{align}` ends with `\cr` rows, FVT.tex:1257-2774, and the group-close errors after
  them leak past `\end{example}`; plus its shipped tikz.code.tex's arrow tips; TL 2025 pdflatex is no oracle here, it
  stops Fatal on that tikz.code.tex) — RED row "amsmath's alignment dimens" below; no bibentry heuristic. One pairing lost: 2507.05750's `\footnotemark` ("∗") and
  `\footnotetext[1]` ("1") — the RED row "A footnote's numeric `[<n>]` is not marked `\thefootnote`" (KPE #1060; formatting
  `[n]` alone was tried and unpaired 58 papers' title-block notes). Residuals: `\setfnsymbol`/`\DefineFNsymbols` stay
  no-ops (5 papers of the 30k); a `.bib`-only paper's `\bibentry` prints nothing and warns (2512.12049, tufte-book);
  footmisc `[symbol]` marks from 10 on follow our radix `\fnsymbol`, not footmisc's set, and `perpage` is not emulated
  (KPE #1057; 0807.1434). 1804.06840's `numbered=no` proofs were already unnumbered; its biblatex "Cited by"
  back-references from them now read "Proof" (the referrer's name, as for any unnumbered theorem; int49a " 3.5", the
  enclosing subsection's number, from the empty theorem name) where the PDF lists pages — no thmtools residual.
  **r2** (review rev78k; guards +4, 15): (6) algorithm2e: a paper's own algorithm2e.sty before 5.1 (5.00 2013/01/06,
  :1532-1534) has two-argument block macros; the arity of the loaded `\algocf@@@block` picks the form (the
  three-argument one read the next statement's first token as a side comment and scrambled the listing: 1402.6889,
  1406.0089, 1410.1699, 1410.6122, 1502.02887, 1507.00513, 1507.00980, 1511.03137, 1511.05741, 1805.07253, 2208.10273;
  KPE #1056 amended); under 5.1+ the line ends only after an end keyword (an empty one leaves the side comment on the
  line, :1868), and only a trailing `\relax` is stripped from a side comment. (7) thmtools: `unnumbered`/`starred`
  are `numbered=no` (thm-kv.sty:241-242, the last numbered key deciding: 2311.15711, 1706.08429 "Proof 161" → "Proof"),
  and `\declaretheorem{a, b}[opts]` declares each name with both option lists (thm-kv.sty:333-348; 2608.16854's
  lemmas, propositions, definitions, examples, remarks were undefined) (KPE #1061). New RED rows: algorithm2e
  `noline` indentation (KPE #1062), amsmath's alignment dimens (0912.5438).
  **Integration (int51, 78k r1-r2 squashed da215b9142 + 78l r1-r2 squashed 3e8c5345f4 + 78m r1-r2 squashed 5b7182df45;
  binary int51a):** 78l's `shown_arguments` change landed on its moved home `base_utilities/util_text.rs` (split114 had
  deleted the old file; `pub(crate)` + the `\textcircled` doc line), reached by sect13.rs through the `pub use util_text::*`
  re-export. Gates int51a: nextest 6720/6720 + cortex 10/10, clippy/rustdoc clean, inits 0/0, dump diff vs int50a empty.
  A/B int50a vs int51a over the three writers' reach populations (2,563 of 2,564: 78k 944 — `\citetext`, `\declaretheorem`
  + thmtools, algorithm2e, bibentry, footmisc (seed 51, cap 150 each) + the rev78k lists; 78l 1,041; 78m 579): E 13,373 →
  13,313, F 24 = 24, 0 papers worse in E or F, 19 better; 421 changed. Word loss > 1 % in 7, all classified: 2012.02570,
  1211.2618, 2505.07606, 2507.01709, 2608.05624 print a `\nobibliography*` + `\bibliography` list once instead of twice
  (bibentry.sty: the starred form reads the .bbl silently); 0711.2399 loses its leaked `\multido \Ik =0+12…` source text
  (pst-plot loads multido, E 6 → 1); 0912.5438 shows no bibliography where the base printed it at `\nobibliography*` —
  both sides stop at equation 4.21 (the `\totwidth@`/`\alignsep@` cascade, RED
  `package-bindings/amsmath_alignment_dimens_totwidth_alignsep_are_defined`, queued as 78o item 5), so its real
  `\bibliography` was never reached on either side. Random controls: 2 of 98 (78l) and 1 of 95 (78m) changed — the
  algorithm2e default block end "end" (2210.14751, 2608.25754; 78k) and 1706.03762's Aidan N. Gomez keeping one
  "University of Toronto" instead of two, Polosukhin's thanks before his e-mail as printed (78m). Binding nets (int51a) 136/136, 0 worse than int50a.
- **78o** (quality batch 12; guards `perfect_kernel_batch78o::*`, 14 in r1, 19 after r2 (one r1 guard replaced), 26 after r3, each
  RED on int51a; OD #851-854, KPE #1060 and #1062 amended, #1085-1087). (1) Footnotes: `[<n>]` is `\thefootnote` at
  `<n>` (latex.ltx:17652-17699; a mark that is no number stays as given); article/report/book/ext* and siamltex, when the document's class, mark their title
  block's notes in `\@fnsymbol` (none under a title page) and reset the counter after `\maketitle`
  (`declare_fnsymbol_title_block`); a mark and a text pair by printed mark, else by the counter value they stand for
  (`_fnvalue`). 2412.20504, 2006.14582, 0903.0014 print the PDF's ∗/†/‡ and stay paired; 2507.05750, 1405.7962,
  2606.03847 pair what int51a left apart; 2505.14146's title "†" pairs with its `\footnotetext[2]` "2" (pdflatex prints
  them apart). (2) algorithm2e `noline`/`\SetAlgoNoLine` blocks and groups give their indentation back (1410.4452 as its
  PDF). (3) A minipage holding one `\subcaption` in a float is the sub-float — id, tags "2a"/"2b", list entry, its
  labels; `ltx:table` in a table — the float counter stepped ahead (2011.13897; the RED repros
  subcaption_labels_in_scaled_panels_stay_on_their_panels and continuedfloat_minipage_subcaption_continues_the_letters
  green); a box with two sub-captions stays as before (2008.05644). (4) acmart's theorems are made `\AtEndPreamble`
  under `\@ifundefined` and `acmthm` (2306.01662 "Theorem 1", "Theorem 2"; 2510.04987's `\example`; authors'
  `\theoremstyle{definition}` theorems get their bold heads). (5) amsmath's alignment dimens and `\black@` (0912.5438
  156 → 124 Errors, the rest its shipped tikz.code.tex; 15,382 → 43,578 words, its tail and bibliography back). A/B
  (release, `--preload=ar5iv.sty`, int51a vs 78o, int51a dumps; 1,215 papers, census pops `pop78o_{wit,fn,footmisc,
  fnart,algnoline,subcap,acmart,amsmath,ctrl}.tsv`: `\footnotetext|\footnotemark` cap 300, footmisc cap 150, article +
  `\footnotemark` cap 150, all 82 algorithm2e `noline`/`\SetAlgoNoLine` papers, all 181 `\subcaption`+`{minipage}`, acmart
  + `\newtheorem` cap 200, amsmath cap 100, 100 random controls, 8 witnesses): E 6,040 → 6,007, F 9 = 9, W 58,970 →
  59,003 (0912.5438's recovered math), 0 papers worse in E or F, +32,370 words, 235 outputs changed, all classified
  (pools overlap): 101 papers' footnote marks (title marks "1" → "∗"/"†" as the PDFs print; pairing equal in 97, better
  in 3, and in 2609.31116 the title "∗" int51a had paired with a body `\footnotetext` is alone, as in the PDF — its 14
  duplicated words gone), 27
  algorithm2e listings (indentation as the PDF), 98 sub-figure panels, 16 acmart theorem numbers/heads, 0912.5438.
  Controls: 5 of 100 changed, all title footnote marks. Repro A/B (sectioning-frontmatter, captions-floats,
  listings-algorithms, class-bindings, package-bindings): only the two RED repros' error counts changed (6 → 0, 2 → 0).
  r2 (review rev78o; rebased onto int52 `0dfa195c22`; guards `perfect_kernel_batch78o::{footnotemark_value_footnote_is_the_counter,
  article_title_notes_step_the_footnote_counter, article_title_thanks_in_every_field_step_in_turn,
  footnotetext_by_number_pairs_only_across_the_title_block, unmarked_footnotemark_pairs_with_its_unmarked_text,
  declared_title_thefootnote_prints_the_marks_the_observer_labels}`, each RED on r1 or int52a): the title block's
  `\thanks` and `\footnote` step the footnote counter as the `\maketitle` digests them (`\lx@title@footnote@step`,
  `lx_title_counts_footnotes`), so `\footnotemark[\arabic{footnote}]`/`\samethanks` after one prints its symbol
  (2201.11183, 2111.13260 "∗" for r1's ""; 1001.4802, 1806.01827, 1904.04720, 2504.19677, 2405.18926 "†"/"∗", each as
  its PDF); one title `\thefootnote` hook, `\lx@title@thefootnote` (`latex_constructs::TITLE_THEFOOTNOTE`), which the
  frontmatter observer reads too (rehearsal.rs `TITLE_FOOTNOTE_HOOK`; S2a.1's `\lx@fm@title@thefootnote` gone; OD #837);
  the counter-value pairing only across the title block or between two unmarked notes (`title_block_partner`; the
  review's standalone `\footnotetext[1]` no longer taken by a table's mark; 2505.14146, 1406.0089, 2002.11721,
  2606.03847 still pair); `[<n>]` read undigested (`OptionalUndigested`), so `\footnotemark[\value{footnote}]` is the
  counter (1804.03292 "series5" as its PDF; 2509.17784's `\footnotetext[\value{footnote}]` prints its PDF's "1" and so
  pairs, by Perl's printed-mark rule, with the appendix's `\footnotemark[1]` that cites it). The r1 report's "every
  changed mark matches its PDF" did not hold for 0707.3986: its PDF's "0" is pdflatex's error recovery, ours follows
  RR.sty (row "A footnote's numeric `[<n>]`…" above). A/B (release, int52a vs r2, int52a dumps, the same 1,215 papers):
  E 6,040 → 6,007, F 9 = 9, W 58,970 → 58,990, +32,425 words, 0 papers worse in E or F, 240 outputs changed; r1 → r2: E,
  F equal, 9 papers changed (the 9 above), all checked against pdftotext. Residuals: `\fnsymbol` (Perl's table) prints
  "∥" where `\@fnsymbol` prints "‖" for 6; a minipage with two `\subcaption`s numbers no panel; siamltex1213 (a raw class)
  marks its title as the body; the field order and `[$\dagger$]` rows above.
  r3 (review rev78o2; guards `perfect_kernel_batch78o::{title_counter_mark_keeps_to_the_title_block,
  report_title_bare_footnotemark_keeps_to_the_title_block, footnotemark_number_error_reported_once,
  footnotetext_number_error_reported_once}` RED on r2, `{title_bare_footnotemark_pairs_with_a_numbered_body_text,
  title_mark_of_another_counter_pairs_with_its_body_text, section_title_footnotemark_pairs_with_its_text}` RED on the
  first cut): in a class that resets the counter after `\maketitle`, a note numbered from the counter (no `[<n>]`, or
  one that moves with it, `FootnoteNumber::from_counter`) carries `_fncounted`, and across the reset a mark and a text
  pair by value only when neither does and by printed mark unless both do (`same_count`): the review's article "C†"
  (`\footnotemark[\value{footnote}]`) no longer takes the body's `\footnotetext[2]`, report's title-page "B2" no longer
  copies the body's bare `\footnotetext` "two"; 2505.14146 (value), 2507.05750, 1405.7962, astro-ph9712060 (printed
  mark) keep their pairing — the brief's stricter rule (no cross-reset pairing for a counted title mark) split those
  three in the first A/B. The title block is the document's title, creators and date, not a section's title
  (1907.00457). `[<n>]` is read for a number once per note, its diagnostics held (`given_footnote_number`), so an error
  in it is reported once, by the mark's digestion (review valerr 2 → 1, val 3 → 1 Errors, as pdflatex). A/B r2 → r3
  (release, int52a dumps; census pops `pop78o_{fn,fnart,footmisc,wit}.tsv` and 1406.0089, 2606.03847, 1503.07490, 599
  papers): E 3,703 = 3,703, F 6 = 6, W 39,621 = 39,621, words equal, no footnote changed; the only differences are run
  to run on the r2 side (1410.4917 one formula unparsed, 1608.08920 one ambiguous_math Warning fewer than in its two
  other runs). The first cut's A/B changed 1405.7962, 2507.05750, astro-ph9712060 (pairing lost) and 1907.00457 (a
  section title's mark), each now guarded.
  **Integration (int53, 78n r1-r3 squashed ba83a4e50a + 78o r1-r3 squashed 6f959635dd onto int52; binary int53a; 78j moves to
  int54 after its r3):** reviews rev78n (NO-SHIP: an empty numbered eqnarray row deleted with its number), rev78n2 (SHIP; its doc
  fixes folded in: the empty-labelled-row comment points at the one rule, the rule doc lists all five guards, OD #847 and two SYNC
  rows state what our `\ref` prints for a label with no numbered row after it and the `\nonumber`-in-`{equation}` residual),
  rev78o (NO-SHIP: title `\thanks` stepped no `\c@footnote`), rev78o2 (SHIP once r3 fixed counter-derived title marks pairing
  across the reset and the triple-reported `[<n>]` errors; the refined cross-reset rule kept 2507.05750, 1405.7962,
  astro-ph9712060 paired as their PDFs print them). Gates int53a: nextest 6781/6781 + cortex 10/10, clippy/rustdoc clean, inits
  0/0, dump diff vs int52a empty. A/B int52a vs int53a over the two writers' saved populations (2,719 of 2,726: `pop78n_*`,
  `pop78o_*`): E 8,625 → 8,591, F 18 = 18, 0 papers worse in E or F; 1,022 outputs byte-changed. Word loss > 1 % in 2
  (1012.3225, 2111.02107: the "Equation" tags of rows `\@@eqncr` does not number). Random controls: 26 of 188 changed — 19 by the
  eqnarray rule (the common `… \nonumber \\ \end{eqnarray}`: the empty last row is the one `\@@eqncr` numbers, pdflatex prints its
  (1); 0905.4550 `S2.Ex3` → `S2.E1`), the rest by 78o's title marks (1812.11634 "2"/"3" → "†"…). Binding nets (int53a) 136/136, 0 worse than int52a.
- **78q** (quality batch 14, class numbering: the sel78q selection study's items 1, 2, 3, 7, 8, and r2 the review's
  findings 1-5; guards `perfect_kernel_batch78q::*`, 15, each RED on int53a or r1; OD #860, KPE #1097-#1101, #1111; SYNC
  jfm row closed). Each binding numbers as its class does, the class's source read first (charter §5: the binding
  defines what the class defines, a copy never overrides it): (1) equations within the sections — JHEP/JHEP3/PoS
  (jhep_cls.rs: `\ifnum\c@section=0\else\thesection.\fi`, `\@addtoreset{equation}{section}`), jheppub (`\numberwithin`),
  jfm (every version, the appendices "(A 1)", "(0.1)" before a section; the older copies' `\nosectioneqnreset`; r2:
  `\section` zeroes the counter, `\section*` too — not yet in the appendices, where the locked `\section` ignores the
  binding's re-wrap: "A 2" for pdflatex's "A 1", a residual (rev78q2) — except under an FLM copy, whose `\@addtoreset` resets at numbered
  sections only); (2) the classes' `\appendix` counters after the kernel's `\@appendix` — elsarticle "Appendix A"/"A.1"/
  floats "A.2" (heading tag the number alone), mnras/mn2e/mn (mn2e_support_sty.rs) and interact "A1", aa (`\appendix` =
  the `{appendix}` begin code) and elsart (`\appendix*` too) "A.1", emulateapj's `[appendixfloats]`, and r2 l-aa's own
  (l-aa.sty:208-214: equations "A1", figures and tables running; it had inherited aa's in r1: astro-ph9705112 "Fig. 12."
  and "(A1)" as pdflatex, r1 "Fig. A.1."); where versions conflict the paper's copy decides (elsarticle v1.0.1 has no
  `\appendix`; an emulateapj copy without the option numbers the floats as its appendix code does: "A1" always where it
  zeroes them, 11/26/04-08/13/06, three of four 6/22/04 copies, one 01/26/10, 17 of 98 census copies; "A2"/"B3", the
  letter before the running number, where the zeroing is commented out, 05/14/03 to 06/22/04, most 6/22/04 copies
  zeroing, 16 census copies: 0709.1471, 0803.1491, astro-ph0508218, astro-ph0501224); (3) llncs's appendices are article's, within the document
  ("A", ids `A1`; `begin_appendices_within`, latex_constructs/mod.rs); (4) svjour's `envcountsame`/`envcountsect`/
  `envcountreset`/`numbook` (svjour_cls.rs, before sv_support, whose `\@ifundefined` set stays svmult's; r2: an
  `envcountreset` counter is declared within the section, ids `S1.Thmtheorem1`, not the order-dependent `Thmtheorem1a`);
  (5) aipproc's default `unnumberedheadings`; r2 (6) llncs's and svjour's `\@thmcountersep` (empty unless envcountsect):
  a paper's own `\newtheorem{x}{X}[section]` prints "11", "A1" as pdflatex (KPE #1111; under OD #783 a document
  `\newtheorem{theorem}…[section]` that replaces llncs's now prints "Theorem 11", pdflatex "Theorem 1." with an error,
  guard `batch77v::llncs_document_newtheorem_replaces_the_class` updated). Repros
  cross-references/*_{equations_within_sections,appendix_counters}, …/{llncs_appendices_are_sections,
  svjour3*_theorem_counters, aipproc_headings_unnumbered, jfm_section_star_equations,
  {llncs,svjour3}_own_theorem_separator{,_envcountsect}}, cross-references/class_copies_78q/ (stubs of the copies'
  headers and appendix code), stand-in-copies/provides_route/copy_numbers_equations_by_section (GREEN now). Three older
  guards follow the class: `batch63::jfm_subeqnarray` and `batch68a::jfm_legend_symbols_and_subequations` "(0.1a)"
  (pdflatex with the jfm copy), `batch67z::aipproc_paper_loads_aipxfm` id `Sx1`. Reach-complete A/B int54a → 78q r2
  (census pops `pop78q_{1,2b,3,4,5,ctl,wit,rec,6}`: pop2b = pop2 + the 14 l-aa papers; rec = the 6 papers whose main
  file is `.ltx`/`.TEX`/in a subdirectory, which r1's A/B dropped: 1011.3048, 1901.06175, 2010.00953, 2111.11983,
  1208.6574, 2003.04542; pop6 = the llncs/svjour papers with a `\newtheorem`): E and F equal in every pop (pop1 382/2, pop2b 973/5, pop3 807/4, pop4 689/2, pop5 50/1, ctl
  1/0, wit 1/0, rec 1/0, pop6 431/2), 0 worse; changed 257/441, 596/1,371, 145/183, 48/315, 22/68, 3/100 controls (all
  three of an item-2 class), 2/7, 2/6, 34/110; every changed token a number (word loss > 1 % only aipproc's dropped
  section numbers, 3). r1 → r2 on the same pops: 4 papers changed, all as pdflatex (0709.1471 and astro-ph0501224, the
  letter-only emulateapj copies: labels 9 → 10 of 10, 36 → 39 of 39; astro-ph9705112 "Fig. 12.", "(A1)";
  astro-ph9607057 "(B1)…(C2)"). Label refnums vs pdflatex's `.aux` (20 changed papers per pop, int54a / r1 / r2): pop1
  191 / 669 / 669 of 672, pop2b 793 / 1,005 / 1,005 of 1,011, pop3 576 / 677 / 677 of 815, pop4 638 / 791 / 791 of 827,
  pop5 40 / 47 / 47 of 48, ctl 124 / 137 / 137 of 139, pop6 725 / 832 / 832 of 875, wit 104 / 191 / 191 of 220, rec 21 /
  23 / 23 of 23; no label matched before and not after. The class probes (the review's 41 + l-aa, two emulateapj
  copies, jfm FLM): 1,017 → 1,040 of 1,061 r1 → r2. Notes (no code): 2111.07466's
  elsarticle copy is elsart-shaped (`\def\appendix{\@ifstar…}`, no `\renewcommand\appendix`) and keeps the kernel's
  numbering; `\appendix` twice under mn2e/elsarticle restarts at "A" as pdflatex does (repeated ids uniquified, `A1a`,
  nothing logged). Residuals (pre-existing, each its own row): `\nonumber` in `{equation}` without amsmath (0708.0971),
  labels after no counter step (`\label{lastpage}`, labels in prose), aipproc's `{ltxtable}` label on the section
  (0810.5501), an unnumbered target's `\ref` text is its title where pdflatex prints nothing (LaTeXML's crossref form),
  old svjour "Appendix 1" (1109.2662), aa.cls v4.03's "A1" (not followed), jfm's `\oneappendix` title, report's and
  article's double-`\appendix` numbering.
  **Integration (int55, 78q r1-r2 squashed onto int54; binary int55b):** review rev78q SHIP after r2 (r1 NO-SHIP: l-aa
  inherited aa's `\appendix`), rev78q2 SHIP with doc follow-ups folded in by the main session: emulateapj's non-zeroing
  copies are 05/14/03 to 06/22/04 (16 census copies), 2505.15463's JFM-FLM_Au class is read raw (not a jfm-binding witness),
  and jfm's appendix `\section*` keeping the equation count ("A 2" for pdflatex's "A 1", jfm.cls:303) is a residual — the
  binding's re-wrap of `\section` inside `\appendix` is ignored ("Ignoring redefinition of \section": the locked `\section`
  refuses a document-time redefinition), so `perfect_kernel_batch78q::jfm_section_star_zeroes_equations` pins e6 at "A 2".
  Gates int55b: nextest 6812/6812 + cortex 10/10, clippy/rustdoc clean, inits 0/0, dump diff vs int54a empty. A/B int54a vs
  int55b (2,815: the `pop78q_*` populations + 300 random census controls, seed 55): E 2,839 = 2,839, F 13 = 13, 0 papers
  worse; 1,075 outputs changed, all in the 78q populations (pop1 257, pop2b 597, pop3 145, pop4 48, pop5 22, pop6 2) plus
  two aa/mn2e controls (1506.07705, 2404.01383) and two witnesses (2303.08726, 2507.06135); 0 of 286 random controls changed;
  word loss > 1 % only in aipproc papers losing their section numbers (0910.2753, 1512.08572, hep-ex0510038), as the class
  prints none. Binding nets: 136/136, 0 documents worse (int54a vs int55b).

- **78j** (quality batch 13, the sel78i selection study's items 3-5 and rider 9; r1-r5; guards `perfect_kernel_batch78j::*`, 37,
  each RED on int50a or on an earlier 78j build, and unit tests `crossref::tests::compressed_joints_join_runs_of_three`,
  `pdfpages_sty::tests::include_pdf_pages_reads_pdfpages_lists`; OD #844-846, KPE #1072-1075). (1) Numeric citations are sorted and
  compressed as their package prints them: natbib's `sort`/`compress`/`sort&compress` (natbib.sty:267-271), cite.sty
  (sorts and compresses unless `nosort`/`nocompress`), revtex4/4-1/4-2 (`[sort&compress]{natbib}`), achemso, acmart's
  `acmnumeric` (its own `\setcitestyle` reads the ordering words), jheppub and quantumarticle — "[1–3]" where int50a
  printed "[3, 1, 2]"; the choice is a `class` on the `ltx:bibref` (`ltx_cite_sort`, `ltx_cite_compress`), applied by
  latexml_post `make_bibcite` where the numbers are known (a run of two keeps its separator; natbib's textual `\citet`
  is sorted, not compressed; a superscript range stays in its superscript) (KPE #1072, OD #844). (2) cleveref prints a
  number in its type's label format, an equation's "(1)": `\cref` "eq. (1)", `\crefrange` "eqs. (1) to (3)",
  `\labelcref` "(1)", `\creflabelformat`/`\crefdefaultlabelformat` honoured; a `creflabel` tag only for a type whose
  format is not plain, its diagnostics held until a reference prints it (2506.11592), the label type's format under
  `\label[<type>]` (KPE #1073, OD #845). (3) A negative `\multirow{-n}` spans the rows above it when they are empty: its
  content moves to the top row with `rowspan=n`, each row keeping the `\rowcolor` its first cell carried
  (`alignmentNoalign`); over a non-empty cell it stays, with one Warning naming the cell (1509.00049: 42 Warnings → 0;
  KPE #1074). (4) pdfpages' `\includepdf` places the pages its `pages` list names as `\includegraphics[page=N]`, one
  paragraph a page, the list and the name expanded (a `\foreach` loop's `pages={\x}`), the page count read for an open
  range; a PDF not found keeps Perl's "See pages … of ⟨link⟩" (OD #846). A/B (release, `--preload=ar5iv.sty`, int50a
  vs 78j; 2,232 papers: the 576 natbib-sort papers, a seeded 600 of the cite/revtex4*/achemso papers, 100 acmart, a
  seeded 500 cleveref and the 216 `\creflabelformat`/`\crefformat` papers, the 151 negative-`\multirow` papers, the 112
  `\includepdf` papers, 100 random controls, 4 witnesses): E 944 → 944, F 4 → 4, W 30,028 → 29,109, no paper with more
  Errors, Fatals or Warnings; 1,745 outputs changed (34 of the 100 controls, all citations): 5,581 citations compressed
  to ranges in 995 papers and about 6,500 reordered; 6,398 references gained the equation parentheses in 258 cleveref
  papers; negative-span tables 2,813 → 3,666 `rowspan`s; `\includepdf` papers 1,344 → 2,935 graphics, no "See pages"
  left. Word counts fall only by the citation numbers a range drops (1,084 papers, -33,104 words, each checked by
  sample: "[2, 3, 4]" → "[2–4]") and pdfpages' "See pages - of" text. Other output changes: internal ids shifted where a
  `creflabel` tag holds math (`\tag{$\star$}`) and whitespace-only lines in footnotes. **r2** (the review rev78j; guards
  `perfect_kernel_batch78j::*` 8 more, each RED on r1 or an r2 build; KPE #1075): (5) the document's `\crefformat`/`\Crefformat`/
  `\crefmultiformat`/`\labelcrefformat` replace the whole reference (cleveref.sty:1851, :1886-1906): a tag form at the
  target for each format the document set (noted as it is set, cleveref's own builders of the defaults bracketed out,
  the derived variant and a sub-type's copy followed), which a reference prints when its target has one (crossref.rs
  `show_alternatives`, a show `<format>|<default>` only in a document that set a format; a `\cref` list's conjunction
  dropped before a position format that holds its own) — 2004.14422 `\crefformat{equation}{#2#1#3}` "1" (r1 "eq. (1)"),
  2312.00252 `#2Equation~#1#3` and 2406.15983 (`\Crefformat` in a `.sty`) "Equation 5", 1409.7964/1603.08012's multi
  formats "(1, 3)"; `\labelcref` the bare number for a type whose `\crefformat` the document set; a format that computes (2608.12136's
  xstring calls on the label being referenced, a Fatal PushbackLimit at the target; a `\cref` in a format) or typesets
  a numbered object (`\crefformat{footnote}{#2\footnotemark[#1]#3}`, a Fatal recursion: 1601.06262, 1711.01177,
  2005.13962, 2008.04624) gives no form, the reference keeping cleveref's default ("footnote 1"), the fail-safe of
  building at the target what cleveref typesets at the reference (OD #845, KPE #1073).
  (6) Under `\label[<type>]` the label type's forms (names, label format, the document's formats) with the object's own
  number, its refnum put in place of a mark: 2609.28438's `\label[prob]` under `\crefformat{prob}{prob.~(#2#1#3)}` printed
  "prob. ( P 1 )" in r2, the tag's math flattened with its whitespace (r3: "prob. (P1)", the tag's math a formula). (7) A label format's plain-number test reads the definition, so the format is expanded once
  (`\creflabelformat{enumi}{label*=\arabic*}`: 4 Warnings → 2 for one `\cref`; 2506.11592). (8) A page past the PDF's
  last is an Error placing nothing, in pdfpages' words ("Page 13 of `…' does not exist", pdfpages.sty:909-920; r1
  placed the last page again). (9) natbib's numbers mode prints a given non-author-year label as the number
  (`\bibitem[Knu84]{k}`: "[Knu84, Lam86, Mil90]", r1 "[4–6]"; natbib.sty:811-817, :1005-1012; KPE #1075), never
  compressed. (10) cite.sty prints the keys it cannot sort first (`ltx_cite_unsorted_first`, cite.sty:205-230): "[Knu84,
  Lam86, 1]" (OD #844, KPE #1072). A/B r2 (release, `--preload=ar5iv.sty`, int51a vs 78j r2; 2,621 papers, populations saved as census
  `pops/pop78j_{natsort,cite600,acmart100,cref500,creffmt,negmultirow,pdfpages,ctl100,wit}` (r1's) plus `pop78j_creffmt2`
  (all 320 census papers setting a `\crefformat`/`\Crefformat`/`\crefrangeformat`/`\crefmultiformat`/
  `\labelcrefformat`, shipped `.sty` included), `pop78j_natbib` (seeded 300) and `pop78j_wit2` (9 witnesses)): E 2,374 →
  2,374, F 9 → 9, W 39,643 → 38,713, no paper with more Errors, Fatals or Warnings; 1,963 outputs changed (r1's
  citations and label formats included). Every changed equation reference of the format papers checked against the
  paper's own format: 2,930 match, 66 multi/range, the rest explained (a format in an inactive `\iffalse` or unloaded
  file, a `\label[<type>]`'s own `\labelcrefformat`, a derived `\Cref` variant); words fall only by the names the
  formats drop (1910.07464 -400: "Eq. 1.3" → "(1.3)") and r1's citation ranges. r1 → r2 on 593 changed papers: 393
  identical, the rest reference texts by the document's formats (2602.03754 "fig. 1" → "1", 2207.10888 "section 5.1" →
  "§5.1", 2409.01424 "Section 2" → "Sec. 2"), 3 natbib/cite labels ("[21]" → "[FNW23]", 2608.05327) and int51's own
  changes (algorithm2e block ends); internal ids shift where a form holds math (`\ensuremath`, 1905.03839's equation
  math `m1` → `m8`) or a discarded note mark took an id (1601.06262 `footnotex1` → `footnotex2`). Residuals: `\crefrangeformat`/`\Crefrangeformat` (a range's
  format holds both numbers, `\crefstripprefix{#1}{#2}`, which a tag at one target cannot carry), `\labelcrefmultiformat`
  and cleveref's grouping and range compression of a `\cref` list (`\cref{e3,e1,e2}`: "eqs. (1) to (3)"); a format
  that computes or makes an object keeps cleveref's default where pdflatex prints the format (the fail-safe; a ruling
  question: typesetting the formats at the reference instead would need the post stage to run TeX); forms are copies
  at each target (an `\ensuremath` format's math in every equation's `ltx:tags`, its ids taken first); a section after
  `\appendix` is our "appendix" type where TL 2025's cleveref keeps "section" in `article` (its `\appendix` patch, cleveref.sty
  :174-287, no longer takes; older TL types it "appendix", as we do); a
  `\creflabelformat`, `\crefformat` or `\crefmultiformat` set after its target is not seen for it (the format in force at the
  target prints, pdflatex's at the reference; OD #845); a `\label[<type>]` inside an `{align}` row is read by
  `\lx@eqnarray@label` (one Warning, pre-existing); achemso's numeric/superscript style is not
  the binding's (it passes `sort&compress` alone); natbib's author-year modes keep Perl's split of a label's trailing
  digits as a year; a row whose first cell is empty or covered loses its `\rowcolor` on the `ltx:tr` (pre-existing,
  positive spans too); a negative `\multirow` inside a multi-column `\multicolumn` stays in its row with its Warning,
  whose cell(i,j) counts the `\hline` rows; pdfpages' layout keys (`nup`, `landscape`, `pagecommand`) are not modelled,
  and a PDF-only paper's pages make large outputs, the pages kept as content where Perl's link chose size (OD #846;
  1805.02202: 133 page images, 23 MB, 1.38 s cold). **r3** (the review rev78j2; guards `perfect_kernel_batch78j::*` 7
  more, each RED on r2, `crossref::tests::cite_sty_sort_keys_and_runs`; batch78n's merged-citation guard and
  doc_changes_index's two index refs re-pinned): (11) a document format's form whose typesetting errs or leaves its
  groups unbalanced gives none (frames unwound, `stomach::unwind_stack_frames_to`; the derived `\Cref` variant
  `\MakeUppercase#2\color{blue}…` of 2008.12750's/2604.24399's `\crefformat` had left a group to the tag's `\endgroup`:
  "Attempt to close non-boxing group", the figure unended, the reference "id1"); its diagnostics are the held tag's,
  raised where a reference prints the form (an empty held tag stays until its target ties them, `note_empty_held_tag`),
  a stray `&` dropped with its Error held ("F 1", 6 Errors → 1 as pdflatex). (12) A document format the target has is
  the whole reference: siamart's `\textup{#2(#1)#3}` with `\tag{$\star$}` "(⋆)", not "( ⋆ ) ‣ Theorem E" (1905.03839, 12
  refs; 1802.04511, 13). (13) Scan stores a tag with markup as its copy (Perl Scan.pm:269), so a reference prints the
  tag's math as a formula: "( 1.2 )" → "(1.2)", "( i )" → "(i)" (1905.03839: 232 refs; 2510.06152), 2609.28438's
  "Problems (P) and (P1)", 2308.16196 "(𝐇𝟏)"; a tag copy keeps its spaces (a multi format's " and (P1)"); a tag with
  font markup keeps it (doc.sty's index refs in typewriter). (14) A `\labelcref` list of a type whose `\crefmultiformat`
  the document set prints the default `\labelcref` format, "1 and 3" (cleveref.sty:8028-8030; role `labelcrefmulti`).
  (15) cite.sty sorts `[prefix]number[suffix]` labels by its sort numbers ("[1, 2a, 7]") and compresses suffix runs
  ("[4a–4c]"). (16) natbib numbers mode: a label given is not the full authors too (`\citet{k}` "Knu84 [Knu84]" →
  "Knu84"). (17) `settle_cref_conjunctions` unwraps through `PostDocument::unwrap_node`. A/B r3 (release,
  `--preload=ar5iv.sty`, int53a vs 78j r3, the 2,621 papers of the saved `pop78j_*` populations): E 2,374 → 2,374, F 9
  → 9, W 39,624 → 38,694, no paper with more Errors, Fatals or Warnings; 2,001 outputs changed (r1-r2's included). r2 →
  r3 net of int52-int53's own changes (r2's outputs on int51a, compared by reference position): 2,008 papers identical,
  30 with reference changes of r3's own: 709 references whose text differs only by the formula's whitespace and 340
  more, each checked (formulas in tags: 1601.01352 "(𝔹1)", 2409.09376, 2608.12136 "P(u)" ×84, "Second inequality in
  (52):" where r2 printed "()"; 2210.14919 "Eq. (15′)" where r2 printed "Eq. (′)"; the two "(⋆)" papers), and 7
  citations compressed where 78n's merged `*key`s now resolve (1204.4241 "[5–7]", revtex's `sort&compress`).
  **r4** (the review rev78j3; guards `perfect_kernel_batch78j::*` 5 more, each RED on r3): (18) a form is digested in a
  group of its own, `stomach::digest_in_own_group` (r3's `unwind_stack_frames_to` now private to it; every held tag,
  `\lx@tag@intags@held`, and every document format, `\lx@cref@docformat`), which unwinds the frames its text leaves open
  and puts back the alignment brace count: the derived `\Cref` form's open `{` had stayed counted, so a figure's tabular
  of two captioned minipages read its next `&` as text (2 "Missing } inserted" + "Extra alignment tab", "p" moved into
  the row above; rev78j3 k5) and an item's form in a cell swallowed the row's `\\` (k2). (19) Empty held tags no target
  closed over (captioned minipages folded into figure panels; a label type's tags placed after their target closed) are
  tied to their owner's labels (the element holding their `ltx:tags`; since r5) and removed once the document is built
  (`settle_empty_held_tags`, before the replay): k6's `\Cref{fa}` raises its form's Error (pdflatex 2 for one runaway argument; r3 0) and no `<tag/>` is left;
  the pending count is the job's, reset with its deferred diagnostics (a cortex_worker's next paper starts at none); a
  held tag whose typesetting raised only Debug messages holds nothing (a plain label format's empty `creflabel` carried
  `_deferred` for its Debug count). (20) `\egroup` in a format is text (a command let to `}`): `#2Fig.~#1#3\egroup`
  raises pdflatex's one error at the reference; the group Error names the format, its type and the target's place. Tag
  copies keep their spaces where Perl's `prepRefText` trims them (OD #845). A/B r4 (release, int53a / r3 / r4; the
  2,621 `pop78j_*` papers plus `pop78j_tabfmt`, the 1,159 census papers with `\crefformat`/`\Crefformat`/`\crefname` and a
  tabular or minipage; 3,317 comparable, 2509.13772 and 2609.30372 failing alike on all three): E 5,173 → 5,173 → 5,173,
  F 15 → 15 → 15, W 66,280 → 65,341 → 65,341, no paper worse; r3 → r4 changed 3 outputs (2112.14232, 2503.10630,
  2312.05616: an algorithm line's empty `creflabel` tag gone, one whitespace line less), no table row; the XML of the
  1,576 cleveref papers holds no empty held tag (the 57 empty `ltx:tag`s left are `\item[]`'s role-less labels and two
  bibitem `key`s).
  Residuals: natbib's "(author?)" for a textual citation of an entry without authors; a format's `#1` in `\normalfont`
  (OD #845); `\eqref` of a `\tag{*}` still walks to its parent for context ("* ‣ 1", Perl's `generateRef`); the review
  rev78j3's later items (row in "Perfect-kernel RED repros without a row elsewhere"); a format closing several groups it did
  not open raises one Error (pdflatex one per `}`), the document's groups kept.
  **r5** (the review rev78j4, rebased onto int54; guards `perfect_kernel_batch78j::*` 3 more, each RED on r4, and the two row
  guards now assert every row whole): (21) an empty held tag is tied to its owner only — r4 tied it to the nearest labelled
  ancestor, so an unlabelled panel's form error was replayed at a reference to the section or the outer figure (rev78j4 m1c,
  m1, m2: pdflatex 0, r4 2/2/1 → 0; m5 r4 3 → 1, pdflatex 2 for one runaway argument); an unlabelled owner drops the tag and
  its diagnostics, as pdflatex never typesets that format. (22) A held tag's text that errs on its own raises no second,
  summary Error (`\lx@tag@intags@held`, as `\lx@cref@docformat`): rev78j4 n9's `\autoref` name `\bgroup Fig` 2 → 1, as
  pdflatex. (23) cleveref's `\lx@cleverref@labeltype` places a `\label[type]`'s type with `float_to_label_scoped`, as the
  kernel's `\label` since 78p: after a closed table, an inner equation or a `\footnote` the type names the object in scope
  (pdflatex "appendix section 1, appendix theorem 1 and appendix section 2"; r4 the table's and the equation's types). A/B r5 (release, int54a vs r5, all 3,319
  `pop78j_*` papers incl. `pop78j_tabfmt`; 3,318 comparable, 2509.13772 panicking alike — "range end index 44 out of
  range for slice of length 43", int54a too): E 5,149 = 5,149, F 16 = 16, W 66,281 → 65,342, no paper worse; r4 → r5
  changed 93 outputs, each by int54's own label moves (81 the same line delta as int53a → int54a, 12 the same link and
  id delta), none by r5's three rules; the XML of the 1,576 cleveref papers holds no empty held tag (the 57 empty
  `ltx:tag`s are `\item[]`'s role-less labels and two bibitem `key`s).
  **Integration (int57, 78j r1-r5 squashed onto int56; binary int57a):** reviews rev78j/rev78j2/rev78j3/rev78j4 NO-SHIP
  (fix rounds r2-r5), rev78j5 SHIP with doc follow-ups folded in by the main session: OD #856 no longer says cleveref's
  label type is placed by `float_to_label_for`; witness 2205.00263 on `settle_empty_held_tags`; new open rows: a
  `\label[type]` after a `\caption` in a minipage panel keeps the figure's type, a `\begingroup` left open in held text
  raises an Error TeX does not, and the 2509.13772 slice panic (int54a the same). Main-session decision: one Error per
  runaway fault (rev78j4 m5, k6: 1 where pdflatex reports 2 from one runaway argument). Gates int57a: nextest 6861/6861 +
  cortex 10/10, clippy/rustdoc clean, inits 0/0, dump diff vs int56a empty. A/B int56a vs int57a (3,581 of 3,585: the
  `pop78j_*` populations + 300 random census controls, seed 57): E 5,146 = 5,146, F 18 = 18, 0 papers worse, words
  +181k; 3,119 outputs changed — the 78j populations and 116 of 266 random controls, classified: 103 numeric citations
  carrying `ltx_cite_sort ltx_cite_compress` (REVTeX's / natbib's `sort&compress`, as pdflatex prints them), `\bibitem[…]`
  labels printed as given (1808.01513 "[AMR88]", cond-mat9912070 "[*]", as the PDF), 10 cleveref/autoref tags; word loss
  > 1 % only in 46 pdfpages papers whose "See pages 1-15 of x.pdf" placeholder is now the included pages as graphics
  (1006.4647, 1805.10685). Binding nets: 136/136, 0 documents worse (int56a vs int57a).
- **78g** (kernel batch: tcolorbox box labels, TCB-LABEL, user ruling 2026-10-21 item 12; + the repro stand-in licence
  audit, item 11; guards `perfect_kernel_batch78g::*`, 4, each RED on 25730beb73; OD #813, KPE #1038). (1) A label after a
  raw `\refstepcounter` names the object the step numbers, which takes the step's number: the kernel's
  `\refstepcounter` records its tags (`note_raw_refstep`, local), a label while `\@currentlabel` is still that step's
  may stop at the box's `ltx:picture` (`Document::float_to_label_for`; schema: `ltx:picture` gets `Labelled.attributes`
  and a leading `ltx:tags`), the object takes the tags unless it has a number or is heading-numbered
  (`Document::take_raw_refstep_number`, the role-less display tag dropped), cleveref's `label type` adds the box type's
  names (and no longer overwrites the enclosing section's), the SVG post-processor keeps a labelled picture around its
  drawing. tcolorbox `\newtcbtheorem`, `\newtcolorbox[auto counter]`, `[blend into=figures]` boxes and the delegated
  `\newtcblisting`/`\newtcbinputlisting` boxes: `\ref` prints pdflatex's number and links to the box. Guards updated
  for the numbered listing lines (`perfect_kernel_batch59::tcb_listing_label_names_its_listing`,
  `perfect_kernel_batch77m::newtcbinputlisting_keeps_its_label`) and the procedure's cleveref names
  (`perfect_kernel_batch65m::cleveref_label_type_after_caption`, now "Procedure" as pdflatex's `\cref`). A/B vs
  int47a (release, 250: the sel78d census's 150 `tcb_label_ref` papers + 100 random): E/F identical in all 250; 8
  papers' references changed (2505.20088, 2311.09542, 2509.17784, 2602.03411, 2606.06976, 2402.06608, 2506.01072,
  2609.20563), all 44 labels now on boxes carrying their pdflatex `.aux` number; 2 papers' unreferenced labelled boxes
  gained numbers (2510.21712, 2601.22385: their inner ids shift by one, the tags first); 0 of the 100 random changed.
  Residual: a box whose listing numbers its lines keeps the label on the numbered first line (its line number
  printed). r2 (review rev78g; guards `perfect_kernel_batch78g::{listing_label_names_no_step_before_it,
  amsmath_row_label_names_the_display_step, unnumbered_display_label_takes_the_step_number,
  picture_second_step_label_climbs, amsmath_saved_kernel_label_names_the_cell_step}`, each RED on r1): a listing's start forgets the raw step (`forget_raw_refstep`;
  a tcolorbox listing box notes its step again, `\lx@lst@renote@raw@refstep`), so a label in an unnumbered listing no
  longer takes an earlier `\refstepcounter{problem}`'s 41; an amsmath display's own `\label` (`\lx@ams@display@label`, amsmath's
  `\label@in@display`) names the step current where it starts (`hold_raw_refstep_for_display`: `{gather*}` row label
  after a step in its cell the section's 7, as pdflatex — r1 44), a saved kernel `\label` (2310.19138's `\labelrel`)
  the cell's step ("a"); an object another raw step numbered is not the next step's (serial mark `_raw_refstep`: a second
  `\put` step's label climbs to the section, r1 printed the first step's number); `\eqno` joins the step's `ltx:tags`
  (`$$\refstepcounter{foo}\label{x2} c=d \eqno(B)$$` one `ltx:tags`, `\ref` 42 as pdflatex; r1 two, "(B)").
  r3 (review rev78g2; guard rows in `amsmath_row_label_names_the_display_step`, RED on r2 "Also 45, 46, 47, 47 and 47."):
  the align family (`{align}`, `{flalign}`, `{alignat}`) and `{multline}`, starred or not, hold no step at the display's
  start (`hold_raw_refstep_for_display(false)`: `\math@cr@@@align` makes the display tag only under `\iftag@`,
  amsmath.sty:1833-1843), so an untagged row's label in a `\refstepcounter` environment is the next tagged row's (1/2 as
  pdflatex and int47a; r2 41/42) and a starred one's the section's (pdflatex `??`); `{gather}` and an `{aligned}`/
  `{split}` in an `{equation*}` keep the held step (44, 45 as pdflatex); a `{minipage}`/`\parbox`/`\@arrayparboxrestore`
  in a display labels at once with its own step (`leave_display_for_parbox`, amsmath.sty:1237-1238; 42 as pdflatex, r2 7);
  the outermost display start clears a display label announced and never made.
  r4 (review rev78g3; guard `perfect_kernel_batch78g::math_box_label_climbs_to_the_display`, RED on r3 "See , , , ,
  and 46." with a Missing Target warning): a label after a raw step never stops inside an `ltx:Math` (`within_math` in
  `Document::float_to_label_for`) — a box in math (`\text{\begin{tcolorbox}\label{t1}…}`, `\mbox`) has an id the math's
  rendering drops, so r1-r3 left the `\ref` empty — it climbs to where int47a's lands: a `{gather*}` row or `\[…\]` with
  the step's number (41, 43 as pdflatex; int47a 2, 2), an inline formula's to the section (7; pdflatex 42, int47a the
  same), a numbered display keeps its own.
  Residuals (OD #813): unnumbered listing line label → the section's (pdflatex the line's 2); second raw step in one
  picture → the section's (pdflatex its own) — decided (main session 2026-10-21): the section fallback stays, no `ltx:g`
  schema change (`ltx:g` labels remain only this residual's future path); untagged `{align*}`/`{flalign*}`/`{multline*}`
  row labels → the row or the section as before (pdflatex `??`); a saved kernel `\label` in a later `{align*}` row than a
  cell's raw step names that step (45; pdflatex 7); a `$$` label after an `{align*}` whose cell stepped → the section's
  (pdflatex the earlier step's 42); a label after `\eqno` → the `\eqno` tag (pdflatex the step's 43); an object with its
  own number keeps it (`equation` with an inner step: pdflatex the step's); a raw step in running text → the section's;
  cleveref `\label[]` inside `equation` → "condition 1" (pdflatex `??`, pre-existing). The tlp stand-in has its `\journal`
  store again (default printed by the `prodtf` head only; `perfect_kernel_batch68x::tlp_placeholders_stay_and_its_date_line_is_whole`
  goes red — a "Journal" pubnote — with the 68x class-default gate reverted); the same-name LPPL copies (vmsta ×2,
  natbib forks ×3) say they are distributed under LPPL 1.3c. (2) Stand-in licence audit (ruling 2026-10-21 item 11; 187 `.cls`/`.sty` under the repros and
  `latexml_oxide/tests/`, their verbatim lines measured against TeX Live and the witnesses' shipped files): 22 copies
  of LPPL (or rename-if-modified) sources now keep their copyright and licence notice and say they are modified
  (revtex4, jmlr, sn-jnl ×2, vmsta ×4, OUP, ceurart, bmcart, memo-l, MastersDoctoralThesis, natbib ×3 forks, apacite,
  minimal.cls/theorem.sty/latex.ltx lines, the article.cls/book.cls copies; vw's natbib.sty ×2 are byte-identical
  redistributions with their notice; jinstpub.sty since 78e); 31 copies of unlicensed or restrictive publisher files are rewritten from
  scratch as minimal stand-ins citing the source's lines (compositio, hha, jcm, JINST, aero ×2, tlp, astlb, gnuplot's
  tikz files, ws-rv9x6, an ×3, wiley-article, jfm ×2, bioinfo, frontiers, deepseek, mdpi, CS18, CSML, pasa, sagej,
  ajour, jmlr2e ×2, cvpr, paper.cls (GPL), sig-alternate, elife), every affected guard green with its expected output
  unchanged; journal-name data, one-line idioms and the ~110 synthetic stand-ins need nothing. Open (minor): the
  sources of three reduced models were not located to compare (cimento's author list, aip-cp, neurips_2026).
- **78f** (quality batch 9, the 78e silent-loss rows; guards `perfect_kernel_batch78f::*`, 6, each RED on 25730beb73
  (int47a); OD #805-807, KPE #1030-1032). (1) jheppub's `\emailAdd` (jcappub and jinstpub ride the binding): measured
  first — by the spelling alone (jinstpub's 78e rule) 40 of the 150-paper jheppub sample's addresses that the list's
  order gives right (1303.3040's `zhl61@` for Zhen Liu) would go to the author block — so an address goes to the author
  it spells, else to the n-th author when the list is one place for each author in their order (by the address or by
  the call), else to the group's only author or the block (`\lx@add@contact@placed[owner=sequence]`,
  `\lx@sequence@call`, `sequenced_contact_labels` at the frontmatter flush, so a list before the `\author`s is read
  against them: 2108.07972), an `\emailAdd{a, b}` an address each (`\lx@split@addresses`, a shared domain written out).
  (2) iopart's `\ead` the same, a `\mailto` inside it the address font (0811.0981); 2311.12551's benedikt.maier@ now B.
  Maier's. (3) aa/llncs: an `\inst` after a comma (`,⏎\inst`, `,~\inst`) and a `\fnmsep` between marks stand on the
  name before the comma (`marks_before_glued_commas`, `leading_inst_marks_end`; llncs.cls:894-896 `\unskip`):
  2401.16957's Clara Dehman keeps her four institutes and note, Mario Centelles his 5-6. (4) `\thanks{\label…\email…}`
  STOPPED (pdflatex errs on the shape; 0 census papers) and (5) 2311.12551's `\AA rrestad` recorded (the paper's own
  raised-superscript `\AA` read as an unanswered affiliation mark; not brief): the silent-loss table's rows. A/B
  (int47a-rel → 78f release, `--preload=ar5iv.sty`, 544 papers: 14 witnesses, 150 jheppub + 83 jcappub `\emailAdd`
  papers and 162 iopart `\ead` papers of the sel78d 30k census (random, cap 150 per class plus the 12 pre-2007 iopart
  ones), its 23 jinstpub and 15 `,\inst` aa/llncs papers, 100 random controls; no "another TeX tree"): E 127 → 127, F 1 →
  1, W 8,329 → 8,329, 0 papers worse, 159 outputs changed, 0 of the controls and 0 jinstpub. By the reviewer's spelling
  test (`emailmatch.py`; owners checked against the addresses), addresses on an author another name spells: jheppub 7 →
  0, jcappub 15 → 0, iopart 83 (in 81 of the 162 papers) → 4 (two an `\ead` inside an `\address`, unchanged; two the
  test's false flags, each right); on the spelled author 360 → 386,
  174 → 196, 154 → 269; on the author block 0 → 2, 0 → 8, 3 → 14 (each spelling no single author — `\l` dropped from
  Nałęcz, 2212.09776; a list shorter than its authors with no spelling, 1802.07622, which the n-th rule guessed); 19
  iopart papers only reorder a creator's contacts. The 7 changed `,\inst` papers (1604.07826, 1811.07968, 2011.09991,
  2309.00573, 2404.01383, 2502.20626, 2503.07779) each give every author its own institutes, where all after the first
  comma had moved one on. Residuals: an `\ead` inside an `\address` takes the address's labels (1205.6797, 1607.05491);
  name keys drop `\l`/`ø` and read no `ue` for `ü` (2212.09776, 2205.06148); an address spelling two authors equally
  goes by the order or to the block (1710.06069's `sjkim0305@` beside two Kims, 2607.04241).
  r2 (review rev78f, user rulings 2026-10-21; guards +12, OD #808-809): step 2 kept, isolated (`author_order_step`; the
  consistent list ruled evidence); the one address left on a multi-author paper goes to the ONE author a note or a
  mark's legend names the corresponding author (`marked_corresponding_author`: 1901.08773, 2510.11030, 2211.07892;
  r2 also counted a `*` with no legend and a note's owner alone, so 1802.03677 and 2308.06532 moved too — narrowed in
  r3, below; 2111.00753's two stars → the block); an iopart `\ead` after an
  `\address` whose mark one author requests is that author's (iopart.cls:363 prints it in that block: 2407.14305,
  2311.07087) — only in a document that writes an `\ead` between its addresses, as IOP's guidelines put them "after the
  postal address(es)" (the mark named the wrong author in 3 of the 4 trailing-`\ead` papers: 0910.0753, 1111.3344,
  1911.06133) and only where no list in author order reads it otherwise (1902.07874, 1710.02251); each iopart `\author`
  a group; `\ead{a and b}` an address each (0710.2104, 1003.0073); a list in markup kept whole, a list of plain
  addresses (with `\mailto`, spacing, `\\`) written out (1105.4742, 0711.4000); an `\emailAdd` inside `\author` one
  contact; the `\inst` comma rule by `\inst`'s meaning (`inst_takes_back_space`; epl2's title `\inst` via
  `\lx@title@inst`, 1201.1900; not beamer's or a document's), not for names that open with their `\inst`; the outer
  flush's sequence labels restored after a nested one; shortened, notice-keeping jheppub.sty / iopart.cls copies beside
  the repros (charter §7). A/B 78f r1 → r2 and int47a → r2 (release, 1,140 papers: all 278 iopart `\ead` papers of the
  census, 300 `\emailAdd` and 300 `\inst` ones (seeded cap), its 15 `,\inst` papers, the review's 150, 28 witnesses,
  100 random controls; no "another TeX tree"): E 1,490 = 1,490 = 1,490, F 11, W 14,118, 0 papers worse; r1 → r2 14
  outputs changed, each an e-mail owner, each classified right against its source (1911.05451's unspelled `ccdxslj@` the
  block's), 0 controls; int47a → r2 214. Addresses on an author another name spells, int47a → r2: iopart 119 (113
  papers) → 5 (two `\ead` inside `\address`, two test false flags, one svjour3 line, the last three unchanged from
  int47a), `\emailAdd` 25 (17) → 1 (2504.02039, unchanged from int47a: an "ID" creator). Side findings (unchanged from
  int47a, not this batch): 2504.02039 a creator named "ID"; 1603.06724 (svjour3) `\email` in an `\institute` line on the
  last author, not the "[corresponding author]" it names. Residual: name keys read no `ue` for `ü` nor a split surname
  (0910.0753's `muender@`, 1911.06133's `ledou@` for Le Doussal stay unplaced).
  r3 (review rev78f2; guards +11, OD #810, KPE #1033): the corresponding-author step to the ruling — the order is
  spelling → the consistent list's n-th → the group's only author → for the one address left on the block, the ONE
  author marked by a note naming them so or a mark whose legend says so → the block. A `*` with no legend marks no
  one (save where the address itself says "(Corresponding author)" and one candidate carries such a `*`: 2308.06532);
  an author is marked by a note in their name or by carrying its printed mark — jheppub's `\note[*]`, or a
  jheppub-family `\note`'s footnote number n on `\author[…,n]` (1802.03677's "1 Corresponding author." on two
  authors → the block); note owners are a set; a legend or note saying something else unmarks its mark (1510.03434);
  the candidates are the address's author group and its address mark's authors (rev78f2 probes i1, i2); a note names
  correspondence by a phrase, not the word ("the AdS/CFT correspondence"; English only, as the census). jheppub's
  `\note[]{}` takes its optional mark (jheppub.sty:84-89; Perl's one-argument `\note` broke 2606.26596's, 1510.03434's
  and 2404.00867's names, KPE #1033; jinstpub takes jheppub's `\note`); a digested thin space parts labels (OD #810:
  2404.00867's `\author[*\,b,c]`). Stand-ins: the jheppub.sty/iopart.cls `\Provides…` lines say "(shortened copy,
  latexml-oxide)", the iopart12.clo stand-in written from scratch. A/B 78f r2 → r3 and int47a → r3 (release, 1,247
  papers: the r2 set, the review's 66 correspondence `\emailAdd` papers, 41 more naming correspondence or writing
  `\note[`): E 1,559, F 12, W 53,315 equal, 0 worse, 0 controls changed; r2 → r3 4 changed, each right (1802.03677's
  `gear8mike@` the block's; 2606.26596, 1510.03434, 2404.00867 names whole, Steinhardt's affiliation b back). Side
  findings (unchanged from int47a, not this batch): 2409.17846's names keep a trailing comma before `\orcidlink`
  ("Qin Tan,", "You Wu,"); an iopart `\author{A and B}\address{…}` gives the address to B only (rev78f2 probe i1).
  r4 (review rev78f3; guards +2, one extended): a `\note`'s counter number is its legend only where no affiliation is
  labelled with that number — there a number on a name is the name's affiliation request (charter §4; rev78f3 probe a6
  `\author[1]{Ann Able,\note{Corresponding author.}}\author[1]{Bob Baker}\affiliation[1]{Univ A}`: zq77@ Ann Able's
  again, as r2/int47a/Perl; the shape of 2005.05695, 2005.14143, 2405.09762, 1403.3797, 2101.00866); an explicit
  `\note[m]` mark is unchanged. `CORRESPONDENCE_PHRASE`: an opening "Correspondence" only before ":" or "to", "for
  correspondence" with no word between ("Correspondence between …", "a test for the correspondence" name no one).
  Recorded in OD #809's residuals, not implemented: notes apart from the names get no counter number (probes a2, a4b;
  0 realised cases in the 1,247), and a `\thanks`/`\footnote` inside jheppub names steps the counter uncounted. KPE
  #1033 witnesses +2212.09764, 2404.17559, 2101.00866, 2405.20427. A/B 78f r3 → r4: 1,251 papers (r3's 1,247 and 2005.05695, 2005.14143, 2405.09762, 2101.00866 — 1403.3797 among them —, release, `--preload=ar5iv.sty`): E 1,559, F 11, W 53,326 equal, 0 worse, 0 outputs changed (temp-dir paths masked; the five a6-shape papers place each address by its spelling, none left over; 1806.01746 times out on both).
  **Integration (int48, 78g r1-r4 squashed c38409d75d + 78f r1-r4 squashed 50e5ba3da2 + the charter rulings f5eeb2c219;
  binary int48a):** rev78g3 SHIP-if (its box-in-math finding fixed in r4, guard red on r3) and rev78f3's regression fixed in
  r4 (guard red on r3; r3 → r4 byte-identical on 1,251). Gates int48a: nextest 6671/6671 + cortex 10/10, clippy/rustdoc
  clean, inits 0/0, dump diff vs int47a empty. Population A/B (2,302 of 2,310: 78f's 1,251, 78g's 765, the tcolorbox census
  400) int47a vs int48a: E 2,222 → 2,222, F 12 → 11, 0 Error or Fatal worse, no paper loses > 1 % of its words; 245 outputs
  changed, 234 already classified by the writers, the other 11 checked: 5 tcolorbox census papers whose box labels now name
  the box (1904.12098, 2409.06317, 2509.11120, 2601.01576, 2609.27175), an empty `\label{}` moving with them (1906.11223),
  2304.11865's cleveref plural names, 2101.00866's `\note[*]` name repaired, 2405.09762's e-mails now on the authors they
  spell, 2109.03031's unspelled iopart address on the block, and 2406.15038 (IOS `aff.*` labels moved between hidden
  places; row in Corpus Triage Quick Wins). Binding nets (int48a) 136/136, no recall below int47a; 6 nets flagged 3
  documents (1704.06338, 2607.02089, 2608.05298) "valid 1 → 0" against the nets' schema copy of 2026-10-15, which lacked
  78g's `ltx:picture` labels and tags — all three validate (0 errors) against the repo schema, and the copy is refreshed.
- **78n** (fidelity audit round 3, numbering and cross-reference text: N1, N2, N3, N5, N7, N8; r2 the review rev78n's
  findings; r3 the main session's two rulings; guards `perfect_kernel_batch78n::*`, 14, each RED on int50a, r1 or r2 but
  `natbib_merge_options_raise_level`; OD #847-848, KPE #1077-1081). (1, rewritten r3 as one rule) eqnarray numbers its
  rows by latex.ltx's `\@@eqncr` state (OD #847; opt-in `eqncr=1` for `eqnarray`/`eqnarray*` only): a row is numbered
  only if it ends through `\@@eqncr` (`\\`, `\end{eqnarray}`) and no `\nonumber` came since the previous one (`\if@eqnsw`
  holds across `\cr` rows), and every `\label` names the number the next printing `\@@eqncr` prints (an unnumbered row's
  labels move to the next numbered row). So a `\cr` row is unnumbered (1011.0523 77 → 73 numbers, PDF 73;
  nucl-th9903062 12 → 6; 0806.3762), a `\nonumber` in a `\cr` row unnumbers the next `\\` row too (2112.13115), and a
  `\label` in a `\nonumber` or `\cr` row names the next number (1206.6051's NA3; 0708.0012); the number printed on an
  empty row of its own under an unnumbered one (`x &=& y \cr \end{eqnarray}`, `… \cr &&\\`, `x \nonumber\\
  \end{eqnarray}`) joins that equation, its tags, id and labels with it (r1 deleted it, `\ref` "III A" or "S0.Ex5":
  0708.0012); `perfect_kernel_batch65o::eqnarray_number_column` updated to pdflatex's (1)/(star)/(2). (2) REVTeX tables
  `\Roman` ("Table I", `\ref` "I"; revtex4-1.cls:6759; KPE #1077); caption names stay "Table"/"Figure" (user ruling
  2026-10-21, 18); AASTeX and emulateapj set them back to arabic (aastex_cls.rs). (3) iopart `\appendix`: "Appendix A"
  section numbers and `\ref`s, equations "A.1", figures/tables "A1"/"B1" (iopart.cls:489-510; KPE #1078; 0712.1190,
  1509.00381, 0910.0753 now matching its PDF). (4) IEEEtran theorems within a section print `\arabic` sections ("Example
  2.1"), "A"/`\Alph` in appendices (IEEEtran.cls:5607-5627; a kernel hook `set_theorem_within_display`, cleared by
  amsthm, OD #848; KPE #1079; 1911.01952). (5) elsarticle `\newproof`: a numbered theorem (`\@prf` steps
  its counter, `[y]` steps y's) whose head prints the name alone, small caps, with its period ("Proof."), body upright
  (elsarticle.cls:1200-1236; KPE #1080; 1304.5109's 12 proofs; r1 left it unnumbered, `\ref` "Proof"); r3: declared
  under the class's `\@ifdefinable`, so after amsthm `\newproof{proof}` is LaTeX's "Command \proof already defined" (an
  Info, as every such message) and amsthm's proof stays. (6) natbib merging (`mcite`/`merge`/`elide`, REVTeX
  4.1/4.2 unless `nomerge`): a citation key's `*` and `[pre][post]` notes are dropped, so `\cite{a,*b}` cites a and b
  (KPE #1081; 1808.00247, 1509.05768, 1206.6986, 1209.2422); `merge`/`elide` raise `\NAT@merge` only (natbib.sty:273-274).
  (7, r2) iopart `\numparts` numbers by the class's arabic form, `\endnumparts` sets it back (iopart.cls:815-825; KPE
  #1078): "(1.1a)" in Appendix A, and the command form's letters no longer pile up (0910.0753 "4.3dabga" → 4.3a; 9 papers,
  439 tags in the A/B). Residual: the merged entry is not folded into the one before (its own row in the
  Silent-loss inventory). A/B (release,
  `--preload=ar5iv.sty`, int50a vs 78n; 1,343 papers: 150 eqnarray-`\cr` + 100 random eqnarray, 298 revtex-with-tables,
  50 REVTeX 3, 50 AASTeX/emulateapj, 218 iopart-`\appendix`, 110 IEEEtran `[section]` theorems, 200 elsarticle
  `\newproof`, 71 revtex `\cite{…,*`, 96 random controls, the witnesses): E 2,423 → 2,423, F 7 → 7, W 14,401 → 14,353
  (math-ambiguity counts as eqnarray rows regroup); 0 papers worse in E or F. Changed outputs: the eqnarray papers' numbers
  (four lose >1 % words: "Equation" autoref and number tags of the unnumbered rows, the continuation rows' math kept in
  the merged equation), revtex table numbers (381 of 421 differ only by Roman numbers and cite keys), iopart appendices,
  IEEEtran theorem numbers, elsarticle proof heads; controls changed only where revtex. A second A/B after the cite-note
  change (516 revtex + control papers): E 1,214 → 1,214, F 1 → 1. r2 A/B (release, int51a / r1 / r2 on the same 1,346
  papers, populations saved as `census/pops/pop78n_<item>.tsv`): E 2,418 / 2,423 / 2,418, F 7 / 7 / 7, none worse in E or F;
  132 changed r1 → r2 (eqnarray groups, theorem heads and tags only, beside the rebase onto int51); referenced labels on
  an unnumbered equation 543 / 619 / 496, none rising in any paper. Equation `\ref` numbers against pdflatex's `.aux` on
  22 changed eqnarray papers: 374 / 608 / 655 of 741 (0708.0012 28/29, its miss a label defined twice). r3 A/B (release, int52a / r2 / r3,
  1,482 papers: the saved `pop78n_<item>.tsv` populations plus 150 seeded eqnarray papers with `\nonumber`/`\notag`,
  `pop78n_nonumber.tsv`, deduplicated): E 2,556 / 2,556 / 2,556, F 8 / 8 / 8, W r2 = r3, none worse in E or F; 147 changed
  r2 → r3, every difference inside eqnarray groups, their tags and ids (no paper changes elsewhere; each paper's label
  set unchanged, labels only move); referenced labels on an unnumbered equation 643 / 595 / 310, none rising in any
  paper. Equation `\ref` numbers against pdflatex's `.aux` on 35 changed papers: 805 / 933 / 1,066 of 1,151 (2112.13115
  30 / 63 / 112 of 112, 1206.6051 66 / 79 / 98 of 98; 0708.0012 28/29 and 0806.3762 15/16, each miss a label defined
  twice). The 113 elsarticle papers with amsthm and `\newproof` (`pop78n_elsamsthm.tsv`): none declares a defined name,
  E 190 = 190, the 3 changed are eqnarray labels. Residuals: a label in an unnumbered row with no numbered row after it
  in its group (an `eqnarray*`, a trailing `\nonumber` row) stays there, where LaTeX's `\ref` names the counter's next
  value (1303.3431's `appa3`); a `\nonumber` inside `{equation}` still unnumbers it (Perl's extension; LaTeX without
  amsmath prints the number: hep-ph9710521's `\nl` in a `\beq`, one behind from 3.22). (Both FIXED 78r.)
- **78r** (quality batch 15, the sel78q selection study's items 4, 5, 6, 10; r1 + r2 after review rev78r; guards
  `perfect_kernel_batch78r::*`, 9, each item RED on int54a or r1; OD #865 and the OD #847 amendment, KPE #1105-1107).
  (1) AASTeX 6+ set their own journal abbreviations (aastex6.cls:1303, aastex631.cls:1838-1900, aastex701.cls:8310-8373:
  `\apjl` "ApJL", `\prd` "PhRvD", 28 names differing from aas_macros' ADS forms; aastex_cls.rs `AASTEX6_JOURNALS` under
  `aastex6_plus`), and aas_macros then counts as not loaded, so a document's own `\usepackage{aas_macros}` restores the
  ADS names as in pdflatex (the 7 census papers that do; 1701.04641 "ApJL" 0 → 3; KPE #1105). (2) An eqnarray `\label`
  with no numbered row after it names the counter's next value: the row keeps that number's reference forms (no printed
  tag) and stands as its own equation (`eqnarray_next_number_tags`, digested only for a row left unnumbered,
  `NEXT_NUMBER_TAGS_PROPERTY`; OD #847 amended, KPE #1106; the SYNC row closed). (3) `\nonumber` in LaTeX's `{equation}`
  is a no-op unless amsmath is loaded (`\ver@amsmath.sty` or its loaded flag; latex.ltx:15737-15739; `LATEX_EQNNUM`; OD
  #865; the SYNC row closed), through one helper, `prepare_latex_equation_counter`, for LaTeX's `{equation}` and the
  mn2e/mnras/mn and aa bindings' own (r2, review finding 2). (4) algorithm2e's `\Indp`/`\Indm` are a signed offset in
  levels that a block saves and restores (its `\hbox{\vtop{…}}` group); a line ends with `blocks + offset` levels, none at
  the least (r2, review finding 1: r1 pushed and popped the blocks' own stack, which set the flush-left
  `\Indm\Indmm \KwIn … \Indp\Indpp` idiom's body a level in — 2102.06989, 1412.1462, 2106.03753, 2508.14614 — and an
  `\Indm` in a nested block popped the outer block's level; KPE #1107; the RED row closed). `.aux` equation `\ref`
  numbers (int54a → 78r): 1602.03553 68 → 89 of 89, 2301.00760 37 → 58 of 58, 1303.3431 64 → 65 of 65, 20 more item-2
  papers 676 → 702 of 704; hep-ph9710521 61 → 74 of 74, nucl-th0404042 15 → 17 of 17, 20 item-3 papers (13 build) 92 →
  273 of 274, 19 REVTeX 4.x papers 370 → 560 of 561. A/B (release 78r r2 vs int54a and vs r1, int54a dumps,
  `--preload=ar5iv.sty`, `census/pops/pop78r_{1,2,2ctl,3,3ctl,3rvx,3cls,4,aasm,alg2ectl,ctl}.tsv`, 1,781 papers converted
  by all three): E 5,039 = 5,039 = 5,039, F 12 = 12 = 12, W 31,912 → 31,938 → 31,938; no paper gains an Error or a Fatal
  anywhere. Item 1, 468 papers: 273 changed, bibliography journal names only (165 papers lose words, the short names).
  Item 2, 801: 569 changed, W 18,514 → 18,540 (math-parse Warnings, unparsed or ambiguous, of 122 continuation rows split
  in 84 papers; the TeX of every visible cell kept, rev78r). Item 3, 323 + 19 REVTeX 4.x + 18 mn2e/mnras/aa papers with a
  `\nonumber` in an `{equation}` (`pop78r_3cls`): 102 + 18 + 5 changed, numbers gained where no amsmath is loaded (of
  the 5, r1 → r2 the 3 mn2e papers without amsmath, 1210.2963, astro-ph0503435, astro-ph9901366, one number each; 0705.0429
  and 1511.06770 are item 2's eqnarray rows); the aa papers that ship today's aa.cls (it loads amsmath: 1712.01010,
  1812.04463, 2501.07980) unchanged. Item 4, 78 `\Indp`/`\Indm` papers: 21
  changed vs int54a (r1 26), 0 words lost, line levels only; r1 → r2 8 changed (the 4 Input-idiom papers and 1903.06418
  back to int54a's levels, which their PDFs print; 1905.03814's unbalanced `\Indp` in a block ends with the block;
  2410.03035, 2602.20199 the same levels with the block's rule before the offset). Their line levels against pdflatex's
  `-bbox` x positions: 2007.00596, 2010.00626, 2006.01085, 1908.06049, 1803.03622 (`\pushline` = `\Indp`), 1801.01253
  match the PDF where int54a set the lines at the block's level; 2102.06989, 1903.06418 match it as int54a did. 30
  algorithm2e papers without `\Indp`/`\Indm` (`pop78r_alg2ectl`): byte-identical; 100 + 150 + 200 random controls:
  5 + 17 + 15 changed by items 1-3 only (r1 → r2 byte-identical). Residuals: `\Indpp`/`\Indmm` (0.5em) and `\Indentp` are not read, and a negative offset at the
  algorithm's level is the margin (pdflatex sets the line left of it); the open rows above (eqnarray continuation-only
  rows under a numbered row; `\nonumber` in an `aligned` inside `{equation}`; IEEEeqnarray labels in an unnumbered row;
  REVTeX 3's "??" labels; `\RequirePackage{aas_macros}` before an AASTeX 6+ class). (r1's third residual, a single
  relation column whose rows join one numbered equation keeping such a label on that number, does not trigger: 11
  probed shapes, `= a \\ = b \label{k}\nonumber` among them, give pdflatex's next number.)
  **Integration (int56, 78r r1-r2 squashed onto int55; binary int56a):** review rev78r NO-SHIP (r1's `\Indm` popped the
  block stack: the flush-left Input idiom indented every later line, 2102.06989) → r2 → rev78r2 SHIP with doc follow-ups
  folded in by the main session: witnesses at the mn2e/aa `{equation}` call sites; the REVTeX 3 "??" and `aas_macros`
  rows restated as decided (main-session decisions 2026-10-22: keep the resolved number; no action, the oracle unclean);
  new open rows for an algorithm's last line ignoring the `\Indp` offset (2508.06088) and the global `\DontPrintSemicolon`
  (SHARED with Perl). Gates int56a: nextest 6821/6821 + cortex 10/10, clippy/rustdoc clean, inits 0/0, dump diff vs int55b
  empty. A/B int55b vs int56a (2,060 of 2,064: the `pop78r_*` populations + 300 random census controls, seed 56): E 5,044 =
  5,044, F 12 = 12, 0 papers worse, no word loss over 1 %; 952 outputs changed, all in the 78r populations except three
  controls, each a 78r item (1005.3618, hep-th0307191: the eqnarray label's refnum-only tag; astro-ph9906362: `\nonumber`
  in `{equation}` without amsmath now numbered, as pdflatex). Binding nets: 136/136, 0 documents worse (int55b vs int56a).
- **78s** (quality batch 16, fidelity audit round 4's binding Errors R2, R1, R13, R11, R5, R7, F5, F3; guards
  `perfect_kernel_batch78s::*`, 13, each RED on int57a; r2 20, of its 7 new 5 RED on r1, the caption one on the r2 draft, the TL-reset one pinning the global branch; KPE #1115-1124, no OD). (1) amsmath's `\genfrac` is robust
  (amsmath.sty:244), so xy's entry scanner no longer meets its bare `\if…\fi` (0704.1039 101 Errors + TooManyErrors →
  0, its 28 bibitems back; 2006.11499 the same; KPE #1115). (2) `\@ifdefinable` on a group of more than one token runs
  latex.ltx:1296-1305's `\string` test (1703.00683's 23 undefined `\olu`/`\calA`… defined; KPE #1116). (3) epsf's and
  epsfig's `\epsfsize` is `\epsfxsize` (epsf.sty:648, epsfig.sty:60), and epsf's `\epsfbox` resets the sizes after each
  graphic, in the scope of the epsf copy TeX would load (r2): TL's :505-506 `\global`ly, a 1991 copy's locally, so inside
  `\subfigure{\epsfbox{…}}` the size serves the next sub-figure too (nucl-th0009090's 15 "=0.99" texts gone;
  hep-ph9404329's 9 sub-figure widths as its latex log, Fig1b.ps 198.5pt; KPE #1117). (4) wrapfig's `{wrapfloat}{<type>}` (:40) and
  float.sty's `wrap<type>` after wrapfig (:265-301); minted's `listing` is a float type of its own, not a `{figure}`:
  with the `newfloat` option newfloat's, else float.sty's named `\listingscaption` (r2; minted.sty:284, :1931-1944), within
  the `section`/`chapter` option's counter (:157-159), and newfloat's caption tag closes with ": " as the kernel's
  (r2, KPE #1124); subfig's `\newsubfloat{<type>}` makes the type's sub-floats (it was a no-op, the Rust Error Fixes
  entry) — 2411.16612 4 Errors → 0, its listing references 2, 3, 4, 1a, 1b as the PDF (KPE #1118, #1123). (5)
  equations.sty keeps the kernel eqnarray with the file's `\if@defeqnsw` (eqnarray*'s rows unnumbered unless
  `\yesnumber`; `set_eqnarray_default_eqnsw`), no raw `$$\halign` begin (hep-ph9904268 8 Errors → 0, 30/30 equation
  labels as its `.aux`, 21 before; KPE #1119). (6) ntheorem's `\newtheorem*` steps its counter (ntheorem.sty:824-839,
  :870; `THEOREM_UNNUMBERED_STEPS`; 1606.01585 "Lemma 7, 8, Corollary 9" as the PDF; KPE #1120). (7) iopart's `\submitted`
  is the journal pubnote of `\journal` (iopart.cls:277-278; hep-th9910179's recursion Error gone; KPE #1121). (8) espcrc's
  `\addressmark[<key>]` reads its key (espcrc1.sty:159-162; hep-lat0409072's "[ETHZ]"; KPE #1122). A/B (release 78s vs
  int57a, int57a dumps, `--preload=ar5iv.sty`, `census/pops/pop78s_{witness,genfrac,ifdef,epsfsize,wrapfloat,equations,
  ntheorem,iopart,espcrc,mintedlisting,mintedctl,newsubfloat,subfigctl,wrapfigctl,eqnarrayctl,epsfctl,ifdefctl,ctl}.tsv`,
  1,478 papers): E 7,330 → 6,814, F 18 → 16, W 66,912 → 66,751, bibitems 57,958 → 58,014, 0 papers worse; 88 changed,
  every one an item's (genfrac 2006.11499; 3 macros defined and 2007.16006's leaked "rg@l@chapter…" text gone; 12
  `\epsfsize` papers lose "=x" text or gain the width; 6 equations.sty papers E 204 → 0; 10 iopart journal pubnotes; 9 espcrc names; 34 minted papers' listings, E 453 →
  423); 1111.1264, 1912.00500, 1806.01746 crash or time out in both binaries. Census: `\@ifdefinable` with a braced
  multi-token argument in 53 papers (scan over the 3,955 users); ntheorem papers other than the witness use no
  `\newtheorem*` (183 unchanged); `{wrapfloat}` in a body is 2411.16612's alone (the other census hits are comments in
  shipped wrapfig.sty copies). Residuals: a `\label` on a `\yesnumber[x]` row names "x" (LaTeX's names the next number);
  2411.16612's apxproof headings (R12); minted's `\SetupFloatingEnvironment{}{}` no-op (minted_sty.rs) also
  replaces newfloat's under `[newfloat]` (fixed 78v); a float.sty float's `autoref` tag under hyperref holds an empty `<text>`
  (base too); acmart's `labelsep=period` reaches figures and tables only (ams_support_sty.rs's
  `\format@title@figure`), so its float.sty and newfloat types close with ": " where the PDF prints "Listing 1."
  (1807.03757); under floatrow a float.sty listing's tag closes with a space (2206.08452, PDF "Listing 1:"); raw minitoc's
  end-document hints for a loaded float package leave an empty `ltx_noindent` paragraph (2512.16676). R2 (rev78s
  findings 1-4, 7; guards `perfect_kernel_batch78s::{epsfbox_local_reset_of_a_1991_copy, epsfbox_global_reset_of_a_tl_copy,
  minted_listing_is_float_sty_s_by_default, minted_newfloat_option_within_section, newfloat_caption_tag_closes_with_a_colon,
  newsubfloat_keys_and_an_existing_type, caption_label_format_names_a_float_sty_type}`, 20 in all): also caption's label
  format names a float.sty type by its `\floatname` (caption.sty:878-881 `\caption@float@setname`; caption_sty.rs
  `\lx@caption@fnum`), which minted's float.sty `listing` exposed ("[] 1" → "Listing 4" under LIPIcs's boxed format,
  2602.12973, 2402.14485) and which restores "Algorithm 1" in float.sty algorithm floats under a caption label format
  (base too: 1305.3417, 1804.10044, 1706.01865, 2206.07592, as pdflatex) — except a ruled/boxed float.sty float, whose style
  resets the label format (caption.sty:864-866 `\caption@setstyle*\float@style`): there pdflatex prints "Algorithm 1" where
  ours keeps the global format, "ALGORITHM 1" (2106.08363), "[Listing] 1" (review rev78s2; fixed 78v). A/B (release r2 vs int58a and vs r1, int58a
  dumps, the 18 r1 populations + `census/pops/pop78s_{minted_all,epsfship}.tsv`, 1,540 papers): E 7,586 / 7,049 → 7,047,
  F 19 / 17 → 17, W 67,704 / 67,509 → 67,509, 0 papers worse than int58a (1111.1264's crash varies run to run: E 20, 19,
  20), no word loss over 1 %; 51 outputs change from r1, each an item's: hep-ph9404329's 9 widths back (198.5pt as latex,
  byte-identical to int58a); 1305.3417 and 2211.11635 the caption name; 1111.1264, 1912.00500, 1806.01746 crash or time
  out in every binary; 45 minted papers take float.sty's `listing` (": " tags, `\listingscaption` renames "JSON n" 1902.08498 and "Code snippet
  n" 2609.21330, `[section]` "Listing 6.1" 2511.15479, ruled bold labels where algorithm.sty left `\floatstyle{ruled}`
  as TeX does, hyperref's `autoref` without a name as pdflatex's "1"; 2309.04695 E 3 → 0, its `\floatstyle` from minted's
  float.sty); `pop78s_capfloat.tsv` (80 caption-label-format papers with float/algorithm/minted): E 1,227 → 1,223, 21
  changed, all named float tags. Caption labels against pdflatex on 12 minted papers: 10 match, 1807.03757 and
  2206.08452 differ in the separator only (above).
  **Integration (int59, 78s r1-r2 squashed onto int58; binary int59a):** review rev78s NO-SHIP (r1: epsf's reset global for
  every copy, minted's `listing` always newfloat's) → r2 → rev78s2 SHIP after the main session's follow-ups: TL's epsf.sty
  is read as the copy that decides (`\def \epsfbox #1{` with its spaces, `epsf_resets_globally`; the TL stub repro rewritten
  in TL's spelling; new repro + guard `epsfbox_tl_sty_over_a_shipped_1991_tex`: a paper shipping only a 1991 epsf.tex gets
  TL's global reset, review probe e1), the reset choice stored globally, KPE #1117 says which copy decides, the caption-name
  fallback's ruled/boxed exception above. Gates int59a: nextest 6933/6933 + cortex 10/10, clippy/rustdoc clean, inits
  0/0, dump diff vs int58a empty. A/B int58a vs int59a (2,152 of 2,160: the `pop78s_*` populations, 300 algorithm/float/minted
  papers with caption label settings, 300 random census controls, seed 59): E 8,667 → 8,130, F 22 → 20, missing `$` 205 → 33,
  0 papers worse (1111.1264's crash varies run to run, E 20 → 22), no word loss over 1 %; 0 of 282 random controls changed; 2
  of 266 caption-format papers changed, both "Algorithm 1" restored (2409.05617, 2501.12231). Binding nets: 136/136, 0 documents worse (int58a vs int59a).

- **78u** (frontmatter content loss: the FM S2a.2 r3/r4 side findings and SYNC row 596's residual; guards
  `perfect_kernel_batch78u::*`, 6, each RED on int59a, and
  `author_split_tests::{notes_out_of_marks_takes_the_note_after_the_mark, classify_thanks_reads_funding_words_whole}`;
  OD #880-881, KPE #1135-1136). (1) A note written inside an author's mark is taken out after it (`notes_out_of_marks`,
  OD #881, KPE #1136): `Nataliia Babina\textsuperscript{\rm 1,\rm 2\thanks{Work done …}}` lost the mark `2` and the
  note, the label `2\thanks{…}` answering nothing (2512.05464, PDF-confirmed); `$^{1,\thanks{…}}$`,
  `^{\mathbf{1,\thanks{…}}}`, `$^{\thanks{…}}$`, `$^{1\footnotemark[1]}$`, and a macro of optional arguments that makes
  a note (AAAI's `\equalcontrib`, 2603.17529; a document's `\samethanks`, 2309.12862) the same; the mark readers keep a
  note left in an operand too. (2) `classify_thanks` reads its funding words as words (`funding_word`, OD #880): "erc"
  in "Perception" (2509.05368), "Recherche", "Merced", "nsf" in "transferred" (the IEEE copyright notice), "fund" in
  "Fundamental" no longer make a note or an institution line funding; SNSF, FNRS, NSERC, SERC kept as funders. (3) A
  footnote's `[\numexpr…]` is a number (KPE #1135; 2210.14698's text pairs with its mark "3"); the Warning for a
  non-number `[<n>]` decided for row 596 is NOT landed: the census A/B found 15 of its 19 papers PASJ papers whose class
  reads the mark as text, a rule is open (row 596, 78u report). A/B (release int59a vs 78u, int59a dumps,
  `--preload=ar5iv.sty`; `census/pops/pop78u_{1,2,3,rand}.tsv`, 32 + 1,441 + 364 + 100 papers, 5 without a main file): E
  3,585 = 3,585, F 10 = 10, W 21,155 → 21,154, 0 papers worse; XML changed in 28 of pop 1 (notes 76 → 110, words +332),
  55 of pop 2 (class only, funding → note), 2 of pop 3 (2210.14698's `\numexpr` mark paired, W 2 → 1; 2309.12862), 1 of
  the 100 random controls (2603.29506, the IEEE notice's class). FM shadow (the S2a.2 300 seeded census papers, 78u
  binary): XML identical with `LATEXML_FM_SHADOW` unset and set (modulo the zip temp-dir `graphicspath`), E/F equal. PDF
  checks: 16 item-1 papers (OD #881); item 2's change is a class only (60 notes read); item 3 the 10 PASJ/emnlp papers
  whose marks print as the class prints them. Residuals: the unanswered-label order after a note mark (OD #881); a
  non-number `[<n>]` Warning (row 596, rule open). Scratch `~/data/pk_wt/scratch_78u/` (ab/, shadow/, pdf/).
  **Integration (int61, 78u squashed onto int60; binary int61a):** review rev78u SHIP after the doc fold-ins (row 596's
  decision text + its open counter-definition rule, OD #851/#880, KPE #1135/#1136, the RED repro's status; two new rows: the
  `\numexpr…\relax`/leading-space marks, the `\protect\thanks` and printing-macro marks). Gates int61a: nextest 6959/6959 +
  cortex 10/10, clippy/rustdoc clean, inits 0/0, dump diff vs int60a empty. A/B int60a vs int61a (2,196 of 2,202: the
  `pop78u_*` populations + 300 random census controls, seed 61): E 3,651 = 3,651, F 9 = 9, 0 papers worse, words +332;
  XML changed in 84 — 28 of pop 1 (the note-in-mark fix), 55 of pop 2 (each only `role="thanks"` lines, the funding words read
  whole; 2603.29506 counted here), 1 of pop 3 (2210.14698's `\numexpr` mark); 0 of the random controls (280 seed-61 + 92 `pop78u_rand`). FM shadow on the
  r4 resolver (int61a unset vs `LATEXML_FM_SHADOW=1`, the 300 controls + pop 1, 332 papers): XML identical. Binding nets: 136/136, 0 documents worse (int60a vs int61a).
- **78t** (quality batch 17, fidelity audit 4's numbering classes R4, R3, R9, R8; guards `perfect_kernel_batch78t::*`,
  each RED before 78t; OD #875-#877 and the OD #847/#865 amendments, KPE #1125-#1131). (1) subfigure's `\subfigure` and
  `\subtable` number their panel whatever the arguments (subfigure.sty:217), an explicit `[]` prints the "(a)" caption,
  `[list][caption]` captions with the second (`subfloat_tokens`, the arguments as read; KPE #1125); subfigure.cfg is read
  (`TABTOPCAP`: a sub-table after its table's `\caption` is 1(a), not 2(a), its caption above it; KPE #1127). (2) The
  sub-captions of subfigure and subfig are the kernel's (`\lx@caption`), not the document's `\caption`
  (subfigure.sty:279-290, subfig.sty:384-411), so 1808.10020's `\def\caption#1{\splitter #1@@}` no longer reads them (48
  Errors → 0; 1806.11431 16 → 0; KPE #1126); subfig's `\subfloat[list][caption]` captions with the second (KPE #1130);
  subcaption's `\subcaption` keeps `\caption`, as subcaption.sty does. (3) A `\label` in an amsmath `{equation}` with
  `\nonumber` names the stepped number (the 78r reference forms, `AMS_EQUATION`, dropped when the equation has no label),
  and a `\nonumber` after `\tag` keeps the tag (OD #875, KPE #1128). (4) A `\label` after an algorithm2e line's number
  names the line — in it, after its `\;`, at a block's start: `\algocf@printnl@i` records the line as its `\nl` step's
  object (`Document::note_raw_refstep_object`), made global as algorithm2e's `\gdef\@currentlabel` (`\lx@refstep@global`)
  and forgotten at the next algorithm's start (`\@algocf@init`'s `\refstepcounter{algocfline}`); with hyperref
  `\algocf@nl@sethref` records it; the line takes the step's reference forms (`Document::take_raw_refstep_number`),
  renumbered with the printed number (OD #877, KPE #1129). `.aux` `\ref` numbers, int57a → 78t (release): witnesses
  1507.00722 7 → 8 of 9, 1808.10020 275 → 379 of 385, 1306.2116 75 → 77 of 77, 2406.05437 37 → 39 of 40, 1602.05866
  78 → 92 of 95; 20 changed papers per item (those with an `.aux`): item 1 490 → 555 of 709 (18), item 2 882 → 903 of 987
  (19), item 3 1,193 → 1,215 of 1,324 (19), item 4 904 → 1,039 of 1,168 (19), no paper fewer. A/B (release 78t vs
  int57a, int57a dumps, `--preload=ar5iv.sty`; `census/pops/pop78t_{1,2,3,4,ctl,rsc}.tsv`, 1,607 papers: item 1 665
  subfigure papers with `\subfigure[]`/`\subfigure{`/`\subtable`, item 2 189 subfigure/subfig/subcaption papers that
  redefine `\caption` or use `\subfloat[..][..]`, item 3 231 amsmath papers with a labelled `{equation}` holding
  `\nonumber`/`\notag`, item 4 414 algorithm2e papers with a `\label` in an algorithm not after its `\caption`, 100
  random, 75 `\refstepcounter`/tcolorbox-theorem papers): E 6,700 → 6,636, F 20 = 20, W 55,084 = 55,084, 0 papers worse,
  no word loss over 1 %; changed 606 / 68 / 213 / 171 / 3 / 6 (the controls' changes items 1-4 cases: 1509.06955,
  1709.02761, 2301.11459; 2205.01906, 2207.14807, 2412.08690, 2506.16787, 2606.23059, 2609.39351). Item 5 (R6, a `\label`
  after a raw `\refstepcounter` in a user environment, 1505.02540) NOT done: no element of the schema can carry the label
  and a number there (an `ltx:para` has neither `labels` nor `ltx:tags`); kept out by the main session pending the schema
  ruling (open row above). Residuals: `\nlset{text}` lines take no reference form; a labelled algorithm2e line's id is
  made at its close (out of its listing's order); `renumber_algo_lines` restarts each listing at 1 where a paper
  continues the count (1311.7635); a `\label` after a `\subfigure{…}` group (pdflatex the enclosing section's number)
  names the figure; `\enl` lines are no step's object (OD #877). (5) A sub-float panel's stated width is its body's,
  its captions, labels and glue left out (`note_sub_float_body_width`, `panel_body_width`; subfigure.sty:232-233): r1
  measured captioned panels with their caption at the column's width and set 28 papers' panels one per row (2006.08949,
  cond-mat9707225, astro-ph0205132; 1208.0688's `\subfigure[text]` pairs); a `\label` between spaces counts nothing
  (2511.21020 Figs 7-8 one row of four, 1607.02761, 2211.12798, 1601.02116); a body of no known width keeps its box's
  width (1610.06413, nlin0606018), and epsfig's `\psfig` is sized by its keys (Perl's `image_graphicx_sizer`;
  hep-ex0703011, cond-mat9707225). (6) The later `\label` of a key wins: a `\label` takes its key from the earlier
  holders that contain it or that it contains (`Document::release_earlier_label_holders`), as LaTeX's last written
  `\newlabel` wins, where the post scan let an inner, earlier holder win (2001.00288: "Algorithm 2" again; 1908.05655;
  holders apart keep it, so the streaming build agrees; OD #877, KPE #1131). (7) A `\newsubfloat` type's sub-floats take
  the kernel caption as the pre-built ones (`define_subfloat`, 78s; 2411.16612). Docs: subfigure.cfg's "0(a)" for
  sub-tables before a bottom caption (OD #876, KPE #1127), the cleveref `@cref` quirk of a `\nonumber` equation label
  (OD #875), `\enl` (OD #877); open rows for `\subref`'s "1(b)", the item 5 schema ruling, algorithm2e's caption-text
  labels (2511.10625), sub-figures in a `tabular`, `\\` in `{center}`, the two-column/REVTeX 3 geometry (awaiting a
  ruling), a deferred float's label, the EPS 72.27/100 size (FIXED 78z), and the panel grid rule and markup.
  **The split (r5, grid design study 2026-10-22's payload 1; FIDELITY_CHARTER §2 "Float panels", user 2026-10-22: the
  panels are a logical grid from what the source states, the LaTeX-side widths kept; do not re-run TeX's line breaking
  against one page geometry).** Landed: items 1-7, the rows Perl's greedy rule (`arrange_panels`, as int61a) fed by
  item 5's stated widths. Removed: r3's TeX line-breaking emulation — the `\sloppy` overfull rule
  (`justified_row_is_overfull`, `has_alignment`, `float_em_of`, `hmode_panel`, `arrange_panels`'s font argument) and the
  `\@bsphack…\@esphack` label-space rule (`is_positive_skip`): item 5 leaves all glue out instead. The grid rule (explicit
  breaks, stated widths and stated glue, `\par` breaks, the `\Gm@tw` line width; target ≥ 408/453) is the next payload
  (landed with 78x, OD #885: 429/453 rows, 422 shapes), the markup the one after (open rows above). Guards `perfect_kernel_batch78t::*` (11: the 6 numbering/label guards,
  `subfigure_panel_is_as_wide_as_its_body`, `a_label_between_spaces_counts_one`, `psfig_panels_are_as_wide_as_their_keys`,
  and `the_later_label_wins`; each RED on int57a-int60a); `a_justified_row_keeps_its_overfull_panel` is a RED repro of
  the grid rule's (Figures 2-4 differ here: 2 and 4 one row, 3 one panel per row, against the PDF's 2+1, 2+1 and one
  per row; review rev78t5 finding 2). At integration (int62) a body of no width without its glue is measured with it
  before the box fallback (review rev78t5 finding 1: 1408.2843 Fig 1 one row again, as int61a and the PDF; guard
  `subfigure_panels_of_no_known_width_share_a_row`). 77s's captionof guard,
  `structure/figures.xml` and `complex/acm_aria.xml` follow (uncaptioned panels numbered; a row as the PDF). Rows per
  float against the PDF's on the study's oracle set (453 multi-panel floats in 108 papers, pdflatex box dumps): int61a
  300, r3 344, r5 354 (exact shapes 282, 330, 336). A/B r5 release (int61a dumps; `census/pops/pop78t_{1,2,3,4,ctl,rsc}.tsv`,
  300 random controls `who_uses.py --sample 30k --cap 300 --seed 785` and 2001.00288: 1,888 papers, 2 without a main
  file): vs int61a E 6,839 → 6,775, F 22 = 22, W 58,713 = 58,713 (1603.04486's ambiguous-math warning, which varies between runs, equal this time), 0 papers worse, no word loss over 1 %; 1,032 outputs
  changed (path-normalized), in items 1-7's classes (panel tags, ids and captions, row breaks and the panel class,
  `\nonumber` and algorithm2e line labels, keys given to the later label, `\psfig` boxes, 1808.10020's recovered content);
  of the 282 controls new to this A/B 6 changed, each an item's (1105.4243, 2511.06063 `\nonumber` labels; 2407.14788 an
  algorithm2e line; 1604.04424, 2511.06063, nucl-th0408073 a subsection's key taken by the later `\label`;
  hep-ex0012018's `\psfig` boxes sized). vs r3 (its 1,608 papers, int60a dumps): 61 outputs changed — 53 in row breaks
  alone, 8 in int61's 78u author notes (one also in a row break) — and nothing else. Sub-float row breaks r3 3,068 → r5
  3,091: 46 floats in 26 papers gain a break (the overfull rule's rows: 1201.0008 against its PDF, 2301.02117; its
  geometry-blind rows, 1411.5428 Figs 6-7, 1508.01788 Figs 19-22, 2111.05508, now as their PDFs; 1408.2843, 1503.04804 a
  missing graphic), 30 in 26 lose one (the glue now left out of the bodies); int61a → r5 3,813 → 3,144 (471 floats in 192
  papers lose a break, 13 in 6 gain one, OD #876). `.aux` labels matching pdflatex's last `\newlabel` (144 papers):
  6,654 (int61a) → 7,167 (r5) = 7,167 (r3); below int61a only 2511.10625 (104 → 103, the caption-text label row).
  **Integration (int62, 78t r1-r5 squashed onto int61; binary int62a):** the user's direction 2026-10-22 (FIDELITY_CHARTER
  §2 "Float panels": the grid first, the LaTeX widths kept; gallery vs matrix) replaced r2-r4's TeX line-breaking emulation
  by the split (r5; grid design study `agent_reports/2026-10-22_grid_design.md`, its payloads 2-3 open rows above). Reviews
  rev78t (r1), rev78t2, rev78t3 NO-SHIP; rev78t5 SHIP with its finding 1 fixed at integration (a body of no width without
  its glue measured with it before the box fallback; guard `subfigure_panels_of_no_known_width_share_a_row`) and finding 2's
  doc nit. Gates int62a: nextest 6970/6970 + cortex 10/10, clippy/rustdoc clean, inits 0/0, dump diff vs int61a empty. A/B
  int61a vs int62a (the `pop78t_*` populations + 300 random census controls, seed 62, + 2001.00288; 1,888 papers): E 6,715
  → 6,651, F 20 = 20, 0 papers worse, no word loss over 1 %, words +44,986; XML changed in 1,032 (the 78t classes: panel
  tags and captions, rows, the `\nonumber`/algorithm2e labels, keys taken by a later `\label`, `\psfig` sizes); 6 of the 280
  converted seed-62 controls changed, each by a 78t class (0807.1408 panel, 1905.12895 algorithm2e label, 2005.04806 panel,
  2205.07594 `\nonumber` label, astro-ph0312238 and physics0011046 `\psfig` sizes). 1408.2843 Fig 1 one row (as int61a and
  its PDF). Binding nets: 136/136, 0 documents worse (int61a vs int62a).

- **78v** (open review rows: a panic, footnote and author marks, float captions, epsf's reset route, an `.eps` shipped
  as `.pdf`; guards `perfect_kernel_batch78v::*`, 11, each RED on int61a but the LIPIcs one (red on the round's first
  draft); OD #881 (extended), #882; KPE #1117/#1135/#1137 (updated), #1138). (1) 2509.13772's panic ("range end index 44
  out of range for slice of length 43"): a marked affiliation's trailing separator was trimmed when the next entry was
  pushed, before an address run under it was spliced at its old length; now each entry but the last is trimmed once its
  runs are in place (affiliations.rs). (2) `[\numexpr…\relax]` and `[ \numexpr…]` footnote marks are numbers
  (`footnote_number_digits`; KPE #1135). (3) A `\protect` before a note command in an author's mark goes with the note,
  and a macro is taken out of a mark only when its body is note commands alone; one that prints text beside a note is
  read with its body in place (OD #881). (4) caption formats a float.sty float in its float style: the type's style is
  `\fst@<type>` (float.sty:40-42, read as caption.sty:887-893 `\float@getstyle`), `\DeclareCaptionStyle` declares a
  caption style (a no-op before), caption.sty's `ruled` (`labelfont=bf,labelsep=space`) is declared unless its `ruled`
  option, and the style resets the untyped settings (caption3.sty:655-662) under the type's own; LIPIcs's `ruled@notop`
  algorithm style ported into lipics_cls.rs from the class's `\AtBeginDocument` (which a stand-in's copy does not
  register), so its settings stand (2008.11837). (5) `\captionsetup[<type>]{name=…}` of a float.sty type sets
  `\floatname` too (caption.sty:882-884). (6) newfloat's `\SetupFloatingEnvironment{type}{options}` (newfloat.sty:304;
  KPE #1138); minted's no-op only where newfloat is not loaded. (7) not done: no one rule covers a class's separator for
  new float types (row 214's finding: acmart's own `\captionsetup` calls first, IEEEtran's figure format). (8) epsf's
  reset scope reads the file the binding answers (`binding_request`, binding/content.rs), so `\input epsf.sty` reads the
  paper's epsf.sty, and psfrag's epsf.sty (`\RequirePackage{epsfig}`) resets locally (KPE #1117). (9) An `.eps` the
  source names but ships only as `.pdf` is found under its base name at digestion (`image_candidates`, OD #882; KPE
  #1137), so the box has its size. A/B (release int62a vs 78v on int62, int62a dumps, `--preload=ar5iv.sty`;
  `census/pops/pop78v_{witness,eps,epsf,setupfloat,classfloat,
  rulecap,numexpr,markmacro,protectthanks,ieeeaffil,marks,lipicsalg,rand}.tsv`, 1,364 rows, 1,361 compared, 3 without a
  main file papers deduplicated): E 3,812 = 3,812, F 4 → 3 (2509.13772's panic), W 14,694 = 14,694, bibitems +70 and
  words +23,324 (2509.13772's document), 0 papers worse; XML changed in 268, each an item's: 140 ruled float.sty
  captions under caption lose the double space after the tag (caption now formats them, pdflatex one space), 46 take the
  `ruled` style's reset, PDF-checked "Algorithm 1 BoN-SFT" (2412.15287), "Algorithm 1 Next-Embedding Prediction"
  (2512.16922), "Algorithm 1 Gauss-Newton Method" (2301.11228), "Algorithm 1 Dual-SVDAE." (2109.00138), "Algorithm
  Nonstaggered…" (2106.08363), AAAI's declared style "Algorithm 1:" (2312.05616); 77 graphics gain the shipped `.pdf` as
  candidate and their size (1904.03199's "top to bottom" panels now one per row as in the PDF; an ORCID icon sized,
  2212.04802); 3 newfloat names (2306.12224 "Source Code", 2507.09840, 2411.16612 "List."); 10 of the 289 random
  controls, all ruled captions or sized graphics. LIPIcs papers with algorithm.sty are byte-identical to the base
  (2008.11837, 1901.10453, 2503.16336, 2607.26704; `pop78v_lipicsalg`). Gates on the code-final commit: nextest
  6981/6981, lint (fmt, clippy, rustdoc, vendored audit, cargo-deny, machete, fontmap) ok, inits 0/0, dump diff vs
  int62a 0 + 0 lines. Residuals: our revtex4 `\textwidth` is 345pt where revtex4's preprint is 468pt (1408.2843,
  harmless there since 78t's panel widths); under `[ruled]{caption}` a ruled float's label adds caption's `labelfont` to
  float.sty's bold (base too); the 10 epsf witnesses' XML is unchanged (each `\epsfbox` there sets its own size).
  Scratch `~/data/pk_wt/scratch_78v/` (ab/, pdfchk/, acm/, ieee/).
  r2 (review rev78v): a macro in a mark is read with its body in place only when that body has no conditional and,
  its notes out, only label-safe tokens; any other is taken out whole as in int62a (`\fnD{\thanks{…}\unskip}`,
  `\fnE{\ifshow\footnotemark\fi}`, `\fnF[1][x]{6\footnotemark}` keep what int62a kept: the "3" and its note, the "4" and no mark, the "6" and its mark; OD #881); KPE
  #1139 records Perl's no-op `\DeclareCaptionStyle` (caption.sty.ltxml:71) and `\restylefloat` (float.sty.ltxml:27);
  the base-name retry's candidates come in a fixed type order, not `read_dir`'s (0907.3747; OD #882; unit test
  `candidate_order_tests`). A/B (release r2, the same 1,361 papers, int62a dumps, against r1's stored
  int62a and r1 outputs): r1 → r2 E/F/W, bibitems and words equal, XML changed in 2, both the candidate order only
  (0907.3747 `nuebar_limit.pdf,nuebar_limit.ps`, 1404.2880 six `.pdf,.png`), no size changed; int62a → r2 as r1's
  table (E 3,812 = 3,812, F 4 → 3, 0 worse, the same 268 changed).
  r3 (review rev78v2, a convergence round): a `\relax` and a `\protect` are no longer label-safe in a mark's macro body
  (a `\protect` right before a note command still goes with the note), and a macro after a `\protect` is not read in
  place, so `{\relax 5\footnotemark}`, `{\protect 5\footnotemark}`, a `\DeclareRobustCommand`'s body, `1\fnS` and
  `\protect\fnX` keep their digit as int63a did (in the name text) where r1-r2 lost it (OD #881); the base-name retry's
  hits are ordered by type, then search directory, then name within one directory, the uppercase `\Gin@extensions`
  after the lowercase (`\graphicspath{{new/}{figs/}}` takes `new/plot.pdf`, as pdflatex; r2 took `figs/` and sized
  from it; OD #882; guard `includegraphics_eps_retry_keeps_the_graphicspath_order`). Probe net (both reviews' probes,
  int63a / r2 / r3): r3 equals the better of int63a and r2 on every author and graphic. Two older losses RED-recorded
  (rows above: `\textsuperscript{\relax 3}`, a four-deep note-macro chain). A/B (release r3 vs int63a, int63a dumps,
  the same 1,361 papers): E 3,812 = 3,812, F 4 → 3, W 14,694 = 14,694, 0 worse, the same 268 changed; r2 → r3 XML
  identical in all 1,361 but for the work directory's depth in 7 texmf-relative candidates.
  **Integration (int64, 78v r1-r3 squashed onto int63; binary int64a):** review rev78v3 SHIP; its findings became rows (a `\protect` before a note macro inside a body, LaTeX's working-directory-first search, a mark label holding any control sequence — the `\relax` row widened) and the comment/guard cites fixed (`\@iffileonpath` latex.ltx:9698 for the directory order). Gates int64a: nextest 7000/7000 + cortex 10/10, fmt/clippy/rustdoc clean, inits 0/0, dump diff vs int63a empty. A/B int63a vs int64a (`census/pops/pop78v_*` + 300 seed-64 census controls, 1,648 converted): E 3,844 = 3,844, F 4 → 3 (2509.13772's panic), 0 worse, words +23,324; XML byte-changed in 280. The 1,361 papers shared with the writer's r3 A/B are identical to r3's output once the work-directory path is normalised, so its classification carries; the 13 changed seed-64 controls are all the ruled float.sty caption class (no space after the tag; `labelsep=space`; 1803.10168's shinyart `labelfont={color=structure,sf}` reset to bold — caption.sty:864-866 `\caption@setstyle*` runs `\caption@resetstyle`, `labelfont=`, then caption.sty:907 `ruled` = `labelfont=bf,labelsep=space`). Binding nets (int64a) 136/136, 0 worse than int63a.

- **78y** (class caption formats; review rev78s2 finding 5; user rulings 2026-10-22 "port both" and "names and separators
  only"; OD #884, KPE #1140; the row "A class's caption separator set outside caption's settings" above holds the rule and
  its residuals): r1 872088bed1 (review rev78y SHIP after findings 1-3), r2 5188ae7e96 — the class caption fonts behind
  caption_sty.rs `CLASS_CAPTION_FONTS` (off by default; acmart, IEEEtran, jfm, webofc, jinstpub), the AMS format from the AMS
  class bindings (article+amsaddr keeps ": "), a `\restylefloat`ed figure or table keeps float.sty's format under IEEEtran,
  ieeeaccess/ieeeoj/ieeetj names and arabic tables, algorithm2e's ": " (`\algocf@capseparator`), caption.sty's float.sty
  label font (caption.sty:860-861). Guards `perfect_kernel_batch78y::*` (37). Writer A/B (int65a → r2, release, 1,031
  papers: ieee 300, acm 300, amsaddr 110, caption/listings/algorithm2e/minted sets; 2011.02194 without a main file): E 4,679
  = 4,679, F 6 = 6, 0 worse; r1 → r2 classified by review rev78y2 (of the 91 papers changed beyond fonts, 71 fonts or
  algorithm2e's separator only; the rest webofc's ". " close, ieeeaccess/ieeeoj/ieeetj arabic tables — 2307.05128 "FIGURE 1."
  / "TABLE 1." as its PDF —, the algorithm2e leading-space fix 1812.03871, 2408.08621, acmart consequences of the fonts going).
  Scratch `~/data/pk_wt/scratch_78y/r2/`.
  **Integration (int67, 5188ae7e96 on 793c8c8865; binary int67a):** review rev78y2 SHIP after a docs fix — finding 1, the
  knob does not combine with ar5iv.sty (mechanism and route: the row "A preload's options do not reach a package another
  preload already loaded"; the route, ar5iv.sty passing `classcaptionfonts`, queued in 78z); findings 2-4 queued in 78z
  (`\restylefloat{figure}` and `\restylefloat*{table}` guards; the 78v ruled-label-font residual re-checked against
  pdflatex); KPE #1140's listings trigger added (`\documentclass{IEEEtran}` + a captioned `lstlisting`: pdflatex "Listing 1.
  C", ours `close=". "`). Gates int67a: nextest 7054/7054 + cortex 10/10, fmt/clippy/rustdoc clean, inits 0/0, dump content
  diff vs int66a empty. A/B int66a → int67a (`ab_66bundle/popint67a.tsv` = the writer's 1,032 + 80 rand61/rand65 controls;
  1,111 converted): E 4,692 = 4,692, F 6 = 6, 0 worse; XML byte-changed in 593, the writer's int65a → r2 outputs reproduced
  byte for byte on 1,030 of the 1,031 shared papers (1705.09704: a relative `candidates` path one level shallower, the
  workdir's depth) and the 80 controls byte-identical; words −12 net in 2 papers, both 78y classes (2301.00894 acmart
  journal `Fig<text>.</text> 1` → `Fig. 1` in 9 captions, a word-split change; 2307.12301 algorithm2e "Algorithm 1: ").
  Binding nets (int67a) 136/136, 0 worse than int66a.

- **78w** (determinism, task #354: the det354 audit's emit sites, agent_reports/2026-10-22_det354_audit.md; OD #883;
  guards `perfect_kernel_batch78w::{missing_targets_are_reported_in_order,
  includegraphics_extensionless_candidates_in_type_order}`, each RED on int64a, and unit tests
  `dump_writer::tests::a_shuffled_diff_writes_the_same_dump`, `document::pi_data_order`, `sect11::held_tag_role_order`,
  `fragment_index::tests::save_lists_each_kind_in_key_order`, each RED with its sort removed; r2: the conversion guard
  for 4b is red on int64a only on a tmpfs `/tmp`, so the order is guarded by
  `util::image::candidate_order_tests::extensionless_candidates_in_one_order_whatever_the_directory_lists` over
  `image_candidate_lists`, the per-directory collection split out, red without the sort). (B1) The format dump's
  records are in (table, key) order in each section (`write_dump`, Perl TeX_Job.pool.ltxml:150,153), where they followed
  the snapshot diff's hash order (one key set in 55 orders across 173 vendor dumps), so a dump, and the symbol ids of
  every conversion, depend on its content alone. (C1) CrossRef's unresolved-reference report in Perl's order (`error, warn,
  info`, types sorted, CrossRef.pm:64-76) with each type's ids sorted. (C3) A PI's keys after `class/package/options` sorted
  (Document.pm:707-708), and (r2) those three only when Perl reads the value as true (`$attrib{$_} ? ($_) : ()`), so no
  `options=""` (`document::pi_data_order::a_pi_drops_an_operator_key_perl_reads_as_false`). (B3) The held-tag roles
  sorted (`tie_held_tags`). (4b) An image name's candidates in TeX's order (`in_graphics_type_order`) whatever the
  directory lists, so `\includegraphics{fig}` beside `fig.eps` and `fig.pdf` is `fig.pdf,fig.eps` on every host (and, as
  TeX, by type before search directory, where Perl's Pathname.pm:375 puts the leading directory first;
  graphics.sty:206-224). (C4, C5) idcheck's cached-id list and the fragment index's side file sorted. Not changed: C2
  (custom frontmatter keys follow the source, main-session decision); B2 (`xmlns:*` order) a row above. Dump
  check: `--init=plain.tex`/`latex.ltx` 0 Error/Fatal (release 78w, `LATEXML_INIT_DEBUG=1`, from a scratch dir), sorted
  `comm -3` vs int64a's dumps 0 + 0 lines (1,007 and 24,253 lines, content identical, order only), and the 78w binary on
  int64a's (old-order) vs its own (sorted) dumps: 20 papers (10 `poprand61`, 10 `pop78u_1`) byte-identical XML, logs
  identical but for timings. A/B (release int64a, int64a dumps, vs 78w, its own dumps; `--preload=ar5iv.sty`;
  `census/pops/poprand61.tsv` + `pop78u_1.tsv` + `pop78v_eps.tsv`, 428 papers, 427 compared, cond-mat9605106 without a
  main file): E 138 = 138, F 1 = 1, W 5,251 = 5,251, bibitems and words equal, 0 papers worse; XML changed in 1, the candidate order only
  (2011.14197, four `fig<n>.eps,fig<n>.pdf` → `fig<n>.pdf,fig<n>.eps`, options-sized, no size changed); logs identical
  but for timings, the math parser's time-budgeted enumeration counts and one source location in a diagnostic (`document.rs:4295` → `:4281`).
  Residuals: the sizer boxes the first raster candidate and EPS bp as 100-DPI pixels (the 78t EPS-size row above,
  1106.2745, OD #876; repro `package-bindings/includegraphics_eps_box_is_its_bounding_box`; both FIXED 78z, OD #886); graphics-tikz/graphic_extension_order_pdf_first stays
  RED for the box (GREEN since 78z) and the rendered image (its candidate order now right). r2 A/B (release 78w r1 vs r2, the same 428
  papers, its own dumps, byte-identical to r1's): E/F/W, bibitems and words equal, 0 worse, logs identical; XML changed
  in 2, the PI only (`\usepackage{}` in 1106.3485: `<?latexml package=""?>` → `<?latexml ?>`; `\usepackage[utf8]{}` in
  2511.15479: `package="" options="utf8"` → `options="utf8"`, as Perl). Scratch `~/data/pk_wt/scratch_78w/` (ab/, ab2/,
  init/, init2/).
  **Integration (int65, 78w r1-r2 squashed onto 9e00bdd0f5; binary int65a):** review rev78w SHIP after r2 (its findings 1-3:
  the extensionless-order guard red only on a tmpfs → a unit guard over permuted listings; the EPS row merged into the 78t
  row; the A/B figures; nits 4-7 folded). The dump content checked by the main session (`LC_ALL=C` sorted `comm -3` vs
  int64a: 0 + 0 lines, 1,007 / 24,253). Gates int65a: nextest 7008/7008 + cortex 10/10, fmt/clippy/rustdoc clean, inits
  0/0, dump content diff vs int64a empty. A/B int64a vs int65a (`poprand61` + `pop78u_1` + `pop78v_eps` + 300 seed-65 census
  controls, 724 converted): E 169 = 169, F 1 = 1, words equal, 0 worse; XML byte-changed in 4, all 78w classes — the PI's
  empty `package` (1106.3485, 2511.15479) and an extensionless graphic's candidates in type order (2011.14197
  `fig3.pdf,fig3.eps`; 1812.05867 `images/EUflag.jpg,images/EUflag.eps`; sizes from `options=`, unchanged). Binding nets (int66a, covering int65 + int66) 136/136, 0 worse than int64a.

- **78z** (an EPS graphic's box in TeX's bp, user ruling 2026-10-22; side item B, review rev78y2's fold-ins; OD #886, OD
  #884's ar5iv route; guards `perfect_kernel_batch78z::*`, 5: the EPS box, the natural `\psfig` panel row and the ar5iv
  knob option RED on int67a, the knob's kept ar5iv options RED in the recipe it replaces, the `[ruled]{caption}` label
  font green on int67a and pinned). (1) `image_graphicx_sizer` reads the first candidate whose size it can read,
  whatever its type, and sizes a vector one (EPS/PS by its `%%BoundingBox`, PDF, SVG) in pt by the graphicx algebra, a
  raster by Perl's DPI rule (`read_image_size`, `ImageSize`; util/image.rs): 40bp = 40.15pt (latex; pdflatex 40.14989pt,
  int67a 28.908pt), `fig` beside `fig.pdf` and `fig.ps` the `.pdf`'s box; an EPS's natural `cssstyle` from the
  BoundingBox, not a stale HiRes box (hep-ex0703011). (2) `--preload=[classcaptionfonts]ar5iv.sty` sets the class caption
  fonts knob and keeps ar5iv's options (ar5iv_sty.rs passes it to latexml.sty); the knob's documented mechanism corrected
  (caption_sty.rs, latexml_sty/mod.rs, OD #884, the repeat-preload row: a preload's options reach `opt@<name>.<ext>`
  only). (3) `ieeetran_restylefloat_table_keeps_float_sty_format` also converts a `\restylefloat{figure}` figure ("Fig. 1:
  Fig one.") and a `\restylefloat*{table}` table ("TABLE I" over the text), pdflatex-checked; a `\restylefloat` in the
  document body, after `\begin{document}` installed the class format, is not seen (pdflatex "Fig. 2: …" and "TABLE II"
  over the text after mid-document restyles; ours the class's ". " and float.sty's ": "): the restyle row's residual. (4)
  The 78v `[ruled]{caption}` residual re-measured: resolved by 78y r2 (the caption row). A/B (release int67a vs 78z, both
  on int67a's dumps, 452 papers: 300 census EPS papers, 80 pre-2008 graphicx papers, 59 controls, 14 witnesses): E 540 =
  540, F 2 = 2, W 5,698 = 5,698, words equal, 0 worse; XML changed in 20 (controls 0): box sizes 12, the natural
  `cssstyle` 5, panel rows 3 (1012.3355 now as latex; 1106.2745 2+2, the two-column geometry row, one per row as latex since 78x; cond-mat0610021 the
  grid row's `\hbox` residual). Scratch `~/data/pk_wt/scratch_78z/`.
  **Integration (int68, 03d1df4718 fast-forwarded onto 8576e7a97d + fold-ins a01d8b410f; binary int68a):** review rev78z
  SHIP (its own A/B check: 452 compared, E/F/W equal, 0 worse, 20 changed; sizer probes — `(atend)`, negative and float
  boxes, a HiRes line first, a DOS-binary EPS, a pdf beside a png, a page-sized pdf beside an eps — each within 0.0001pt of
  pdflatex and latex+dvips; rasters byte-identical to base; no measurable cost, 39 EPS-heavy papers 34,112 → 33,625 ms);
  fold-ins: the sizer comment and OD #886 say what `natural_display_size_pt_of_candidates` really does (finding 1 → the
  open row "The natural `cssstyle` and the box read different candidates", queued in 79a), the EPS DPI round-trip
  wording (graphicx_sty.rs, OD #107), the sp deltas quoted, hep-ex0703011 named, KPE #1141 (Perl sizes a vector graphic
  as device pixels), the open row for graphicx `bb=`/`natwidth=`/`hiresbb` (finding 5). Gates int68a: nextest 7059/7059 +
  cortex 10/10, fmt/clippy/rustdoc clean, inits 0/0, dump content diff vs int67a empty. A/B int67a → int68a
  (`ab_66bundle/popint68a.tsv` = the writer's 453): E 540 = 540, F 2 = 2, words equal, 0 worse; XML byte-changed in the
  same 20, and both sides byte-identical to the writer's run on all 452 compared. Binding nets (int68a) 136/136, 0 worse
  than int67a.

- **79a** (K11's line rule refined, user ruling 2026-10-22 "only printed text is a value, decided by expansion; `\par`
  and `\\` end a line", OD #662; side item B, review rev78z finding 1, OD #886; review rev79a findings 1-7; guards
  `perfect_kernel_batch79a::*`, 16). (1) `ClassReaders::store_lines` (frontmatter_stores.rs) cuts the store's innermost
  group at the line breaks around it (`ends_a_line`: `\par`, `\endgraf`, `\\`, `\newline`, `\break`, `\linebreak`,
  `\cr`, `\vskip`) and records the other `\@` macros there; one is a value only when its expansion prints text
  (`prints_as_a_value`, `prints_text`: a font/size/shape switch — `Definition::declared_font` or LaTeX's declarations
  —, spacing, a rule, `\relax`, a line break and nothing print none; anything unknown prints): zzfma's
  `{\@addrfont\@address\par}` (`\itshape`), zzfmb's `\vbox{…\@jinfo\par…\@address\par}`, zzfmc's
  `{\@jinfo\\[2pt]\@addrfont\@address}` and zzfmf's `{\@jinfo\linebreak\@address\par}` give the address contact;
  zzfmd's `{\@jinfo, \@address\par}`, tlp.cls:627 (1209.2617) and uiucthesis.cls:177 keep their lines. r2: the values
  are judged when the class is loaded and again at `\maketitle` with the document's (`lx_store_line_values_<name>`,
  `harvest_stores` → `prints_in_a_line_with_a_value`; a store handed on as it is set waits for `\maketitle` when its
  line has such values, `lx_store_deferred_<name>`), so zzfmd's `{\itshape submitted \@submitted; e-mail \@email\par}`
  with the class's empty `\@submitted` and the document's `\submitted{1 May 2026}` keeps the e-mail in the line (r1
  "submitted 1 May 2026; e-mail" and an e-mail contact; review Q1); a store glued to the class's printed text — a
  letter or digit beside it, or a `.`, `/`, `-` inside a word; not a sentence's full stop, a skip's dimension
  (`in_a_dimension`), an `\if`/`\ifx` operand or `\href`'s first argument — keeps its line the class's
  (`glued_to_class_text`; piparticle-final.cls:272 `{DOI: 10.4279/PIP.\@doi}`, where r1's pubnote "020001" was a DOI
  that does not exist; review finding 1, Q2); a store a running head prints is in no title-page line
  (`running_head_arguments`: `\markboth`, fancyhdr's and scrlayer-scrpage's heads; preprint.cls
  `\ihead{\@shortauthor:~\@shorttitle}`, 2512.04983's `<toctitle>`). (2) The box and the natural `cssstyle` read one
  candidate, the first any reader sizes (`first_natural_size`, util/image.rs), a raster there no `em` size; the
  candidates follow `\Gin@extensions` (`declared_graphics_extensions`), which r2 seeds as pdftex.def:597-600 sets it
  (r1 started it `\@empty`, so grfext's `\AppendGraphicsExtensions{.eps}` made `.eps` the list alone and first; review
  finding 3), with TeX's edits: `\DeclareGraphicsExtensions`, grfext's Append/Prepend/Remove (1711.06128), pdftex.def:695's
  begin-document `,.eps`, and epstopdf-base's `\AppendGraphicsExtensions{.eps}` when epstopdf loads or at
  `\begin{document}` with grfext loaded (`\lx@graphics@eps@last`): 2107.02597's gnuplot figures are their `.eps` (box
  361.4pt x 252.9pt, the BoundingBox), as pdflatex reads them.
  r1 A/B (release a01d8b410f vs r1, int67a dumps, 892 papers): E 2,517 = 2,517, F 14 = 14, W 17,839 = 17,839, 11
  changed. r2 A/B (release int68a vs r2, with r1 alongside, int68a's dumps; 955 compared of 958, 3 without a main file:
  r1's 894 — raw-class census, shipped-class line shapes, raster+vector basenames, `\DeclareGraphicsExtensions` papers,
  controls, witnesses —, 43 papers of TL and shipped classes whose title page glues a table store to text (≤6 per TL
  class, ≤4 per shipped class), 21 users of grfext/`\Append…`/`\Prepend…`/`\RemoveGraphicsExtensions`): E 4,087 =
  4,087, F 19 = 19, W 20,118 = 20,118, contacts 5,102 = 5,102, words 14,022,039 → 14,022,020, 0 worse; XML changed in 15
  against int68a: r1's own (2310.04553 webofc subtitle; 2605.00042 WileyNJD journal pubnote, r2 places it after the
  volume, handed at `\maketitle`; 2107.02597; 1410.6039, acmsmall, its xfig `.pstex` EPS figures given their
  BoundingBox `cssstyle` in a `\scalebox{.32}`, as latex/dvips reads them; 1002.3829, 1012.4507, 1503.03445,
  1503.07722, 1510.06950 piparticle-final, the received/accepted dates and the editor the frontmatter's, which r2
  keeps — the DOI line is the class's again, as int68a and pdflatex print it) and r2's (0907.2032 rose_v01
  `ROSE.2008.\@doi`: the line "DOI 10.1515 / ROSE.2008.002" whole, int68a an invented pubnote "002"; 2005.14200,
  2602.20067 rsos/rspa `\@jname.royalsocietypublishing.org`: the header "rsos.royalsocietypublishing.org" and its
  crossmark link whole, int68a a pubnote "rsos" and ".royalsocietypublishing.org"; 2005.00555 lhep's
  `\hbox{Letters in High Energy Physics\hfill\@journal\enspace\@vol,…}` and 2401.01585 fundam's `\@volume(\@issue),
  20\@publyear` whole with the document's values, int68a pubnotes beside "Letters in High Energy Physics  xx, xxx, 2020"
  and "(), 2024"; 1608.05172 IEEEtran `\DeclareGraphicsExtensions{.pdf,.jpeg,.png}` beside `img/x.PNG` and `x.eps`: r2
  took the eps after the begin-document `,.eps`, but pdflatex's log reads `<./img/nsfnet.PNG>` — kpathsea finds the
  `.png` entry's file in another case; int68a's PNG was right, restored in r3). r1 → r2 changed in 15: those r2 changes, the
  five piparticle DOI lines, 2605.00042's order, and 1903.12215, 2004.12182, 2603.20393 (ar-1col `https://doi.org/\@doi`)
  back to int68a byte for byte. Witnesses byte-identical: 1209.2617, 1406.5776, 2105.09416, 2107.05207, 2304.14584,
  1503.05530, 2609.00670, 1711.06128, 2512.04983, and the jps-cp/jpsj-suppl `\hskip.5em\@kword` papers (1705.02293,
  1309.6905). Residual rows (ranked list): the page style the class never selects (`\ps@jhead`), the line rule's
  residuals after r2, the natural `cssstyle` against the file post renders. Scratch `~/data/pk_wt/scratch_79a/` (r2:
  `r2/`, r3: `r3/`).
  r3 (review rev79a2, NO-SHIP → findings 1-6): (1) a head or foot definition in the title page is page furniture as a
  running head is (`head_definition_body`: `\def`/`\gdef`/`\edef`/`\xdef`/`\renewcommand` of `\@oddhead`, `\@evenhead`,
  `\@oddfoot`, `\@evenfoot`; titleps's `\sethead`/`\setfoot` with three arguments), for the glued rule too: a class's
  `{\def\@oddhead{\@shortauthor: \@shorttitle\hfil}}` had kept `\shorttitle` the class's and lost "Short T" (r2
  regression; int68a and r1 the `<toctitle>`; reach ptptex.cls); (2) `declared_rank` matches an extension in any ASCII
  case, the entry's own case first, as kpathsea finds a file (`texmf_casefold_search=1`): 1608.05172 is its PNG again;
  (3) the `\Gin@extensions` seed's comment says it is pdftex.def's under any driver option (dvips.def:202's list under
  `[dvips]`; a ranked row); (4) epstopdf's `prepend` option and `\epstopdfsetup{prepend}`/`{append}` act as
  epstopdf-base.sty:498-515 (pdflatex the eps); (5) a store waits for `\maketitle` only for a placeholder value
  (`may_become_a_value`: undefined, `\relax`, a parameterless macro expanding to nothing), not for the line's code
  (`\@setfontsize`, USG's `\@journalfont`): 2605.00042's journal pubnote is back before the volume, in the document's
  order, as r1; (6) OD #662's guard list already named all seven r2 guards (checked), r3's added. Guards
  `perfect_kernel_batch79a::{a_store_in_a_head_definition_is_the_frontmatters, a_store_beside_a_font_macro_keeps_its_order,
  declared_graphics_extensions_fold_case, epstopdf_prepend_puts_eps_first}` (16 in all). A/B of record (int68a → r3, one
  binary): the batch report.
  **Integration (int69, bd4caf7453 fast-forwarded onto 2b4ec522c0 + fold-in 4fce85df74; binary int69b):** reviews rev79a
  (NO-SHIP: the piparticle DOI, Q1, `\AppendGraphicsExtensions` → r2), rev79a2 (NO-SHIP: head definitions, case folding → r3),
  rev79a3 (one SHOULD-FIX: a head command's optional arguments — titleps `\sethead[…][…][…]`, scrlayer `\ihead[…]`, fancyhdr's old
  `\lhead[…]` — were left out of the page-furniture span, so a short title there was lost as in r2; fixed in the main session,
  `running_head_arguments` starts the span before them, guard variant in `a_store_in_a_head_definition_is_the_frontmatters`;
  its nit, the case fold sorted ahead of the `\graphicspath` directory, an OD #886 residual). Main-session decisions: the
  title-page fields are judged by the document's value when the title page prints (Q1); a store glued to the class's printed
  text is the class's line (Q2); piparticle's received/accepted `<date>`s and editor creator kept; the five fragment pubnotes
  int68a split out of class text ("002", "rsos", "193", …) dropped with their values printed whole in the line. Gates int69b:
  nextest 7075/7075 + cortex 10/10, fmt/clippy/rustdoc clean, inits 0/0, dump content diff vs int68a empty. A/B int68a → int69b
  (`ab_66bundle/popint69b.tsv` = r2's 958; 955 converted): E 4,087 = 4,087, F 19 = 19, contacts and toctitles equal, words −19
  (the five fragment pubnotes), 0 worse; XML byte-changed in 14, both sides byte-identical to the writer's r3 run on 952 of 954
  (2605.17443, 2609.01974: a relative `candidates` path's depth, the workdir's). Binding nets (int69b) 136/136, 0 worse than int68a.

- **78x** (the panel grid rule, user rulings 2026-10-22: rows from what the source states, against the class's real line width; the
  row "**Panel grid rule**" above holds the rule, numbers and residuals; OD #885, #876, #886; KPE #1142; guards `perfect_kernel_batch78x::*`,
  33): r1 9e0fb5a83c (kernel shadow geometry — review rev78x NO-SHIP: the delta counted height keys, box bodies and package reads), r2
  e53cad5110 (no shadow hooks: a panel's stated width tokens recorded as read — `Parameter::stated`, `KeyVals` `stated_value`, gated by
  `KEEP_STATED_LENGTHS` — and evaluated against the real geometry; per-class rows pages, `declare_panel_page`: IEEEtran, REVTeX, AASTeX
  6.x, mnras, aa; rev78x2 NO-SHIP), r3 f57449342f (evaluation inside the float's group, scaled/rotated deltas, the `\hbox` row key, page
  values for compsoc conference / draft a4 / REVTeX preprint+twocolumn, document `\setlength` records, row-start glue; rev78x3 NO-SHIP),
  r4 fef469477b (an unevaluable or inconsistent width falls back to the base's line, geometry included; a centred `\\` starts a
  paragraph; rev78x4 NO-SHIP), r5 cc1f05a97b (`\newline` is `\@normalcr`; AASTeX/emulateapj floats not centred; subcaption `\subfloat`
  withheld lengths; rev78x5 SHIP), r6 861a819dd7 (the integration's binding-net catch, 1710.06503 0 → 8 Errors: a panel row
  separator's `ltx:break` was taken for a `{center}` capture's first content node by `insert_block_as`, so a leading `\hspace*`/`\hskip`
  hoisted the `\parbox` panels out as `logical-block`s and kept the caption in the row's `ltx:block`; separators are now marks, not
  content — `block_insertion.rs`, `is_panel_separator`; guard `an_hspace_star_opens_the_row_paragraph`). Oracle (453 floats against the arXiv PDFs' box dumps): rows 410 (r1) → 429, shapes 399 → 422. Writer
  A/B r5 (one binary, 1,907 papers): E 6,680 = 6,680, F 20 = 20, W 59,351 = 59,351; 750 floats in 313 papers change rows — 424 better, 19
  worse in 13 papers (each in a filed residual row: `\label` key-text widths, subfig `\subfloat` body glue, sub-figures in a tabular cell,
  undeclared class pages, rows TeX keeps tight or overfull, the scorer's image-only count), 25 both wrong, 66 same count, 216 unverifiable.
  **Integration (int70, cc1f05a97b fast-forwarded onto f24574e043 + fold-ins 9193b0a596 + r6 861a819dd7; binaries int70a, int70b):** fold-ins: KPE #1142, the
  adjustbox-around-`\subfloat` RED row (rev78x5 finding 1), the tight nested `\subfloat` and a raw revtex4-1 copy class's floats as
  residuals. Gates int70a: nextest 7108/7108; int70b: nextest 7109/7109 + cortex 10/10, fmt/clippy/rustdoc clean, inits 0/0, dump content diff vs
  int69b empty.
  A/B int69b → int70a (`ab_66bundle/popint70a.tsv` = the writer's 1,907): E 6,680 = 6,680, F 20 = 20, words equal, 0 worse; XML
  byte-changed in 328, byte-identical to the writer's r5 outputs on all 1,907; int70b byte-identical to int70a on all 1,907. Binding nets:
  int70a 135/136 (1710.06503, ws-procs961x669: 0 → 8 Errors, the r6 catch); int70b 136/136, 0 worse than int69b.

- **79b** (audit #363, `agent_reports/extracted_356a0c47/2026-10-23T011644_a95c8312ad38d03b1.md`: class begin-document code the
  bindings drop; the silent-loss row "Class begin-document code the bindings drop" above holds the fix, witnesses and residual rows;
  KPE #1143; guards `perfect_kernel_batch79b::*`, 7, and `shipped_class::tests::class_code_ends_a_line_at_a_carriage_return`): r1
  1e4a3cdd9d (sn-jnl's `{appendices}` as appendix.sty `[title]` re-dressed by the class's begin-document code, lettered floats and
  equations; `shipped_class::class_code` ends a line at CR, LF or CR LF; the llncs/svjour/svjour3/svmult/svproc bold `\vec` under each
  copy's own conditions, `sv_support_sty::define_bold_vectors`; rev79b SHIP after fold-ins), r2 d4fceeef29 (tests and docs only: every
  `\vec` guard a whole-element `assert_element`, the KPE number, the sn-jnl repro's "Figure A1", five residual rows — cleveref's
  `appendix` type for `{appendices}` units, adjacent `\vec`s merged into one identifier (math parked), capital Greek upright in the
  bold-italic forms, the internal name in llncs's text-mode `\vec` warning, svmono/SNmult copies not read). Main-session decisions:
  +7 `ambiguous_math` Warnings in 4 papers accepted (`\vec v(t)` parses as `\mathbf{v}(t)` does), 2608.10941's section-less
  `{appendices}` unwrapped as the kernel's `\appendix`, `\vec{kr}` one bold identifier.
  **Integration (int71, d4fceeef29 cherry-picked onto 79469473fb as bef26c401a; binary int71a):** gates int71a nextest 7117/7117 +
  cortex 10/10, fmt/clippy/rustdoc clean, inits 0/0, dump content diff vs int70b empty. A/B int70b → int71a (`ab_66bundle/popint71a.tsv`
  = the writer's 362: sn-jnl, llncs, svjour, svjour3, svmult, svproc and controls): E 1,178 → 1,177, F 4 = 4, 0 worse, words −3,454 in
  the 9 papers over 1 % (arrow accents → bold letters, invisible times, section numbers → appendix letters; no authored text lost, the
  review's sample against the PDFs), 125 changed; both sides byte-identical to the writer's r1 outputs on all 362. Binding nets int71a
  136/136, 0 worse than int70b.

- **split114** (task #114, user directive 2026-10-07; a pure move, before FM S2a): `latexml_engine/src/base_utilities.rs`
  (23,317 lines) is the module directory `base_utilities/` — `mod.rs` (the imports, `FRONTMATTER_ELEMENTS`,
  `REPLACEABLE_FRONTMATTER_TAGS`, and `LoadDefinitions!` calling `defs::{general, frontmatter_api, frontmatter_fields,
  author_macros, tags_refnums}::load()` in source order), `defs/` (the five parts of that body), `author_block/` (16
  files), `frontmatter/` (7), eight utility files and `author_split_tests.rs` (module path unchanged). Each file is under
  1,000 lines except `author_block/dispatch.rs`: `add_authors_calls_sectioned`, one 1,460-line function kept whole and
  lifted out of the `LoadDefinitions!` body. `pub use` re-exports keep every existing path. The only edits are
  `use`/`mod` lines, `//!` file heads, visibility (276 cross-file items, 14 fields and `NameKeys::of`; nothing new in the
  prelude) and three `super::tex_glue::` paths, now `crate::tex_glue::`. Check: 585 items and 218 statements are identical
  after normalising visibility and rustfmt. Inits 0/0; `comm -3` against the int48a dumps is 0 lines for latex, plain
  and texlive.version. nextest 6671/6671, the same count as int48a. The 40-paper `feature:author_marks` census sample
  (seed 114) gives byte-identical XML against int48a.
  **Integration (int50, split114 squashed 2dcd4588f1 onto int49; binary int50a):** int49's one changed line of the old file
  (78i's `\lx@ifdefined` in `\lx@latin@name`) carried to `base_utilities/defs/tags_refnums.rs:242`; the pure-move check re-run
  by the main session (only-old 0, only-new 0). Gates int50a: nextest 6688/6688 + cortex 10/10, clippy/rustdoc clean, inits
  0/0, dump diff vs int49a empty. A/B (297 of 298: 200 census `feature:author_*` + 97 mixed-class) int49a vs int50a: every XML
  byte-identical, E 40 = 40, F 0 = 0. Binding nets (int50a) 136/136, 0 worse than int49a.
- **FM S2a.1** (the frontmatter observer core, shadow mode only; KERNEL_CAPABILITIES K23 row, step 1; OD #837; design
  `agent_reports/2026-10-21_fms2_design_refresh.md` §2/§3, defaults 1, 2, 6, 9): `base_utilities/frontmatter/observer/`
  — a ledger of the `\author`-family calls (the kernel `\author` and the bindings' `\lx@add@authors*` calls, with their
  stamped or projected positions, options and queue places; the `\title` S1 stamps; the creator/contact/note API calls,
  inside a block's expansion or after it), a rehearsal of each block at the flush under Layer-B event bindings (the
  speculative-run engine of #767; field ledger put back), and the event lexer (Name, Mark with `symbol`/line-first,
  Sep and/comma/word/gap, Break, Contact, Note, Layout, ApiCall, with field spans). `LATEXML_FM_SHADOW=1` (runtime) makes
  the CLI write `<dest>.fmshadow.json`; without it nothing runs. New core API: `gullet::{InputMark, input_mark,
  input_unread_since}`, `Mouth::read_position`, `token::FieldLedgerScope`, `document::{field_prov_stamped,
  restore_field_prov_stamped}`, `DiagnosticsHold::warnings_raised`; `\lx@field@stamp` defined, no call site. Guards
  `perfect_kernel_batchfms2a1::*` (14); repros `sectioning-frontmatter/observer_*` (6: four RED for output, the S2b
  targets; `observer_rehearsal_aftergroup_stays_inside`, `observer_fill_gap_and_footnotemark_labels` GREEN).
  A/B int50a → fms2a1 on 300 seeded census papers: XML byte-identical 300/300 with the variable unset and 300/300 with it
  set (the zip's `/tmp/.tmp…` graphicspath normalised), E 27 = 27, F 0 = 0. Shadow on a seeded 1,000 of the 30k census:
  901 papers with blocks, 3,670 blocks, 3,662 lexed, 7 empty, 1 out of positions (2606.14105, a 49,591-character
  collaboration list; with hep-ex0311039 of the 300, 2 of 1,159 papers with blocks over both samples, 0.17 %: the 14-bit budget stays, default 2); 6
  rehearsals held diagnostics (3 sig-alternate blocks, 4 errors each: the class's `\alignauthor`, `\end{tabular}\begin{tabular}`
  at sig-alternate.cls:274, with no tabular open — 1102.4021, 1611.03298, 2208.01741; 1811.05708's `\note`, 1 error;
  2403.16604's two blocks, 6 warnings each); 43 papers have creators only from a binding's single-author calls (`\lx@add@author`/`\lx@add@creator`: llncs,
  authblk's unlabelled `\author`, 23 more bindings), recorded as API calls, no block. Cost (`perf stat -e
  instructions:u`, 9 stable documents of 10): off −0.007 %, on +0.008 % against int50a (the 10th, 2108.08785, varies
  ±1.5 % run to run on int50a itself). **r2** (review revfms2a1): the rehearsal's whole group reads from a mouth of its
  own under a current token of its own — r1 leaked an `\aftergroup` of the block into the document's input
  (`\author{Ann Able\aftergroup\leak \and Bob}`: `<p>LEAKEDBody after.</p>` with the switch on); `\hfil`/`\hfill` a
  fill gap (always a separator) and a spacing box's width from its `width` property in em of its font; a bare
  `\footnotemark` labelled with what it prints (`\thefootnote` under the kernel `\maketitle`'s `\@fnsymbol`, at the
  counter plus the block's earlier steps, `\c@footnote` never stepped); `Sep(and)` keeps its command (`\And`, `\AND`);
  ledger `epoch` (flushes before a call), `SkipReason`, a failed JSON write a Warning; the status line's undefined
  macros/missing files sorted (their hash order followed the interning history, which the rehearsal changes). The
  departures S2a.2 must know are listed in OD #837 (a Note carries no mark, a font Layout no span, `\corref`/`\fnref`/
  `\thanksref`/`\ead` not bound, projected spans keyed by (block, span), a block-final `]` uncovered, a mark printing
  only a `\ref` — A&A's `\inst{\ref{…}}`, 1809.08354, 94 of the 1,000's 3,662 blocks — no mark). A/B int50a → r2:
  300/300 XML byte-identical unset and set (E 27 = 27, F 0 = 0, W 2,854 = 2,854), logs off = on 299/300 (hep-ph9612328:
  a time-budgeted math enumeration count, which varies run to run on int50a too); the 970 sectioning-frontmatter repros
  970/970 identical to int50a unset and set, off = on 972/972 in XML and stderr. Shadow 1,000 unchanged (3,670 blocks,
  3,662 lexed, 7 empty, 1 out of positions, 6 held); 17 blocks' footnotemark labels now the printed symbols, 225 `\and`
  separators with their command (113 `\and`, 101 `\And`, 11 `\AND`). **r3** (review revfms2a1_2): a container's
  placement argument is no name (`\begin{tabular}[t]{c}` gave Name "t", `\parbox[t]{5cm}{…}` "tAnn AbleUniv A": the walk
  reads the arguments `shown_arguments` marks shown); the kernel's `\lx@newline` a `\centering`/`\raggedright` or a
  parbox/minipage rebinds `\\` to is a Break (once one Name); a stray `&` a `column` separator — the `&` constructor
  errs only when built into the document, so nothing is held (OD #837 corrected); an `\input` in the field no longer
  doubles the log's `(Processing content …` note with the switch on (core `error::SilenceProgressNotesGuard` in the
  rehearsal; its tokens carry no span); the title's footnote marks follow a binding-declared `\lx@title@thefootnote` (the kernel's hook since 78o r2)
  (hook, default `\@fnsymbol`; revtex's default is `\@fnsymbol` too, `\@alph` only under `altaffilletter`); a block
  deferred for want of a journal reports `journal unavailable`; a font switch inside a run stays a departure (splitting
  would split gretsi's `first \textsc{last}` names, 2508.03934). Guards +6 (20) and an engine unit test. A/B int51a →
  r3: XML byte-identical on the 300 seeded census papers, 300/300 with the variable unset and 300/300 set (E 27 = 27, F
  0 = 0, W 2,854 = 2,854), logs off = on 299/300 (hep-ph9612328, the time-budgeted math count as before); the 976 repros
  976/976 identical to int51a unset and set, off = on 976/976 in XML and stderr. Shadow 1,000 vs r2: 3,670 blocks, 3,662
  lexed, 7 empty, 1 out of positions, 6 held (as r2); Breaks 1,201 → 1,218 (+17 in 4 papers: minipage and sig-alternate
  `\\`s), Names 7,228 → 7,218 (−10 in 9: placement letters and `\cite` keys no longer names, rows split at their
  breaks), Layout 134 → 125 and font Layout 322 → 329 (a cell's run now starts at its own font, not a placement
  letter's), gaps and the and/comma/word separators unchanged, no `&` column in the sample; ApiCall +4 (1609.09832,
  2010.05094: jpconf's address and e-mail calls, int51's 78m, from the rebase).
  Next: the resolver and the comparator (S2a.2).
  **Integration (int52, FM S2a.1 r1-r3 squashed a2917112cf onto int51; binary int52a):** reviews revfms2a1 (NO-SHIP:
  `\aftergroup` leak), revfms2a1_2 (NO-SHIP: placement arguments read as names, breaks lost under a rebound `\\`, an `\input`
  in the field doubling its log note) and revfms2a1_3 (SHIP); main-session follow-up from revfms2a1_3: `note_progress` /
  `note_progress_detailed` honour the rehearsal silence too (a `{comment}` skipped inside a rehearsed `\thanks` logged its note
  twice), and the two guards that pin the token analyser's Error by count also pin it by name (`merged_creators`). Gates int52a:
  nextest 6741/6741 + cortex 10/10, clippy/rustdoc clean, inits 0/0, dump diff vs int51a empty. A/B int51a vs int52a (544: 300
  census `feature:author_*`, 147 across 12 classes, 97 rare author shapes `author_tabular`/`author_comma_inst`/
  `author_user_mark_macro`; seed 52), switch unset: every XML byte-identical, E 782 = 782, F 2 = 2. The status line's
  undefined-macro and missing-file lists are now sorted (`error.rs`; Perl's order is hash order), so log diffs against
  pre-int52 bases reorder those lists. Binding nets (int52a) 136/136, 0 worse than int51a.
- **FM S2a.2** (the frontmatter resolver, policy table and shadow comparator, shadow only; K23 row; OD #837 "S2a.2";
  design §2 points 1-9, §3 S2a, defaults 4, 5, 6, 8, 9; the main session's S2a.2 notes): `observer/resolver.rs` reads the
  ledger's blocks and API calls into `front.rs`'s tuples, each association with its rule (`resolved` in the dump;
  rulings 1-4, the owner chain, the API calls' `label=`/`labelseq=`/leading mark/`annotate=`), under `observer/policy.rs`
  (keyed on (the binding owning `\author`, the hook): revtex, authblk, llncs, aa, inst_support, svjour, jheppub, jcappub,
  jinstpub, iopart, jpconf, each citing its OD); the lexer adds `ref:<key>` marks (A&A `\inst{\ref{…}}`) and binding
  request marks (`\corref`, `\fnref`), a `\footnotemark[<n>]` read with the counter the block's notes stepped, the ledger the API calls' contents (rehearsed and lexed), attributes and parent,
  and the `\author` owner (`note_binding_load`, the latexml_package and latexml_contrib `dispatch`). `ab_audit fmshadow ROOT` compares with the
  XML per paper: kind, family, deciding rule, classified (an adopted ruling's change) or not; "no block" and "held"
  outside the gate's denominator. Guards `perfect_kernel_batchfms2a2::*` (15), engine `policy::tests` (2), ab_audit
  `fmshadow::tests` (4). **Measured (release `fms2a2-rel`, `--preload=ar5iv.sty`):** XML byte-identical to int52a on the 300 seeded
  census papers with the variable unset and set (E 27 = 27, F 0 = 0, W 2,854 = 2,854; logs off = on 298/300,
  hep-ph9612328's and 2111.06996's time-budgeted math counts) and on the 976 sectioning-frontmatter repros (off and on, XML and stderr);
  the final binary re-run gives the same, save one warning fewer once with the variable unset (2111.06996's
  time-budgeted ambiguous-math count; a rerun gives 35 = 35); cost `perf stat -e instructions:u` on 10 documents with blocks: off vs int52a +0.007 %, on vs off +0.071 %
  (at most +0.48 %, hep-ex0408042). Shadow census, the whole 30k (`sel78d/census_30k.tsv`, 26,322 converted; the
  first 12,031 with a binary lacking the `\ref` marks, the 37 of those whose blocks hold a `\ref` re-converted):
  23,359 papers with blocks, 76,503 blocks, 76,381 lexed, 112 empty, 10 out of positions, 204 blocks holding
  rehearsal diagnostics (111 papers, the "held" bucket); 3,000 papers "no block". `ab_audit fmshadow --re-resolve`,
  per family (papers; agree / disagree classified / disagree unclassified; gate = unclassified share of the counted):
  kernel 6,867 (4,618 / 355 / 1,837; 27.0 %), revtex4_support 5,639 (5,075 / 265 / 299; 5.3 %), ams_support 3,105
  (2,563 / 41 / 500; 16.1 %), IEEEtran 2,202 (1,467 / 116 / 618; 28.1 %), authblk 939 (23.9 %), elsart_support_core 818
  (7.0 %), inst_support 776 (35.7 %), aas_support 688 (39.8 %), mn2e_support 560 (20.4 %), "binding" 488 (38.7 %: a
  contrib binding's `\author` before its loader seam was noted — fixed, the census predates it), llncs 400 (40.5 %),
  iopart_support 264 (8.0 %), sv_support 262 (72.9 %), jheppub 196 (9.3 %), ijcai 59 (78.0 %), JHEP.cls 58 (82.8 %);
  total 23,207 counted, 17,409 agree, 938 classified, 4,860 unclassified (20.9 %). Largest unclassified buckets
  (papers): name-split by the names line 1,126, an XML-only affiliation no shadow rule explains 1,094, an XML-only
  e-mail 699, a note beside its name 496, an unmarked line's affiliation 433. PDF checks on a stratified 200 of the
  disagreeing 5,000-paper sample (pdflatex on copies; 17 do not compile): shadow right 36, XML right 34, both wrong 24,
  undecidable 89 (the automated name/label/spelling checks decide only part); of the unclassified 157 judged, 30 shadow
  wins — the gate with those subtracted ≈ 16.9 % overall (kernel 19.7 %, revtex 4.1 %, IEEEtran 23.4 %). The K23 class
  guard (under 0.5 % per family) is far: S2b waits on further resolver rounds per family (questions in the fms2a2
  report). Scratch `~/data/pk_wt/scratch_fms2a2/` (ab/rep/final.*, pdf/verdicts.tsv).
  **r2** (review revfms2a2: of 40 sampled unclassified disagreements 31 were the resolver's; OD #837 "S2a.2 r2"; rebased
  on int53 a2db6c5a86): the resolver by defect class — notes keep their whole text (an emptied one in `dropped`),
  legends by their leading Mark, whole `fuzzy:` labels, a creator's `author:N`, Perl's prefix fallback, names lines by
  evidence, marks mid-line and trailing, the API intake creator calls only, declared contact roles, authblk's
  `\AB@thenote` (ruling 1), today's joined unmarked lines, contact grid and replacing `\author`, pre-creator
  annotations; the policy key's class fallback (report/book title-page marks arabic; the inert revtex field dropped —
  revtex4-2.cls's default is `\@fnsymbol`, :5976-5978); the comparator compares the block, explains an XML item only by
  the shadow's pieces joined, reads ORCID links by identifier, counts elimination as today's (OD #52, R10), and
  classifies only by an adopted ruling's tag or a PDF-confirmed XML-wrong class listed by name (wide-space names merged,
  a name list in one personname, an ORCID logo read as "ID"). Guards `perfect_kernel_batchfms2a2::*` (27, 12 new, each
  red on r1), engine `resolver::tests` (4), ab_audit `fmshadow::tests` (6). **A/B** (release, int53a vs r2,
  `--preload=ar5iv.sty`, int53a dumps): 300 seeded census papers XML identical with the variable unset and set (save
  the per-run temp-dir `graphicspath` instruction), E 32 = 32 = 32, F 0, W 3,021 = 3,021 = 3,021 paper by paper; the 979
  sectioning-frontmatter repros XML identical (base, off, on) and stderr identical once paths, times and the binary's
  name are normalised. **Fresh census** (the same 26,322 papers re-converted with the r2 binary, not `--re-resolve`;
  the dump's own `resolved` equals re-resolution): counted 23,207 — agree 19,101, classified 1,312, unclassified 2,794
  (**12.0 %**, from 20.9 %); per family (gate): kernel 6,800 (19.9 %), revtex4_support 5,639 (**2.7 %**), ams_support
  3,105 (2.9 %), IEEEtran 2,202 (17.3 %), authblk 939 (14.3 %), elsart_support_core 818 (6.9 %), inst_support 786
  (26.0 %), aas_support 688 (7.1 %), mn2e_support 560 (13.0 %), llncs 400 (22.2 %), "binding" 317 (11.4 %),
  iopart_support 264 (8.0 %), sv_support 262 (17.9 %), jheppub 196 (7.7 %), jcappub 93 (3.9 %), ijcai 59 (74.6 %),
  JHEP.cls 58 (82.8 %); the class fallback now names report (23), book (20), jinstpub (24, all held), jpconf (14).
  Largest unclassified buckets (papers): an XML-only affiliation no shadow item explains 655, a names-line split 644,
  a name split no rule explains 208, an XML-only e-mail 201, a block e-mail line the XML gives an author 184. PDF
  precision samples (pdflatex, seeded; 10 of the first 75 per side do not compile, replaced): classified 60 — the
  shadow's change right 41, wrong 18, undecidable 1 (≈ 69 %; wrong: an API affiliation holding several marked entries
  not parted 6, a `fuzzy:` label that spells no name 4, a glued label `12`/`1⋆`/`∗∗` 3, a note's e-mail by adjacency
  not spelling 2, a name class misapplied 2, other 1); unclassified 60 — resolver wrong 25, XML wrong 19 (mostly names
  merged with marks or affiliations, classes not yet listed), both 5, normalisation 9, undecidable 2. The class guard
  is not met in any family; revtex is the closest. Scratch `~/data/pk_wt/scratch_fms2a2r2/` (rep/final.*,
  pdf/judged.tsv).
  **r3** (OD #837 "S2a.2 r3"; rebased on int54 b232b0eed4; backup `backup/fms2a2-r2` at eb506a0d28): the classified
  bucket's wrong classes and the pilot families' residuals — marked entries of one API call parted by their marks, glued
  labels parted by the marks shown (`ruling1:parted`; a number into digits only under single-digit entries), fuzzy
  labels by surname and with their macros written out (`ledger::label_text`), a note's e-mail by spelling, author-order
  address lists for every class (`owner:order`, charter §4), e-mail Contacts placed address by address, addresses in a
  names line as e-mails, a per-author class's single name (`Sanhueza, Patricio`), names not ending at a gap after a
  given name or an initial, collaborations as authors, revtex prefix marks on a sole line, ORCID links (`contact_parts`),
  `\cite` marks and environment arguments in the lexer, mark-free note texts, links kept in affiliation text, amsart's
  worded `\and` (policy `ams`), a sole author's orphans, title notes (`api:title-note`); the comparator reads marks typed
  as text or as floating-superscript formulas, ORCID ids out of notes, glued e-mail labels, label-prefixed joined items;
  change tags `ruling1:parted`, `role:url-as-email`, `owner:order`; XML-wrong classes added on PDF-confirmed witnesses
  (a later names row read as an affiliation, a name with its affiliation, a collaboration's whole list, names parted only
  by a mark or a note, a title note on the first author, names-line addresses, an ORCID id as text) and the r2 classes
  fenced (one-word pieces only in lists of five or more, no address a name). Guards `perfect_kernel_batchfms2a2::*` (36,
  9 new, each red on r2), `resolver::tests` (5), `fmshadow::tests` (8). **A/B** (release, int54a vs r3, int54a dumps): 300
  seeded census papers XML identical with the variable unset and set (one transient math-parse divergence in
  quant-ph0206123's "on" run; two reruns each way identical), E 32 = 32 = 32, F 0, W 3,021 = 3,021 = 3,020 (the
  transient); the 979 sectioning-frontmatter repros XML identical (base, off, on), E 23, W 39 each. **Fresh census** (the
  26,322 papers re-converted with the r3 binary, resolved again with the final resolver): counted 23,207 — agree 19,425,
  classified 1,688, unclassified 2,094 (**9.0 %**, from 12.0 %); per family (gate): kernel 6,800 (16.1 %),
  **revtex4_support 5,639 (0.9 %)**, **ams_support 3,105 (1.2 %)**, IEEEtran 2,202 (12.0 %), authblk 939 (10.0 %),
  elsart_support_core 818 (6.0 %), inst_support 786 (23.1 %), aas_support 688 (4.5 %), mn2e_support 560 (7.1 %), llncs
  400 (17.0 %), "binding" 317 (9.1 %), iopart_support 264 (6.1 %), sv_support 262 (12.6 %), jheppub 196 (6.0 %),
  jcappub 93 (3.9 %), ijcai 59 (72.9 %), JHEP.cls 58 (81.0 %). The five r2 buckets (10 papers each against the PDF):
  XML-only affiliation — resolver 1, XML 6, both 2, normalisation 1; names-line split — resolver 2, XML 6, both 1,
  undecidable 1; unexplained name split — resolver 5 (collaborations as authors: fixed), XML 3, normalisation 1,
  undecidable 1; XML-only e-mail — resolver 6, both 2, undecidable 2 (fixed: glued `E-mail:` labels, address-by-address
  Contacts); block e-mail line — resolver 10 (6 author-order lists: fixed; names lost or institutions read as names 4). PDF precision (pdflatex, seeded, drawn from the first
  9,286 fresh papers, a random third; no PDF or no author block on page 1-2 replaced): classified 62 — the shadow right
  57, wrong 3 (2407.20481 a lone unrequested label, 1501.05343 a parted `21` — fixed, 1606.05885 a name split at a line
  end), undecidable 2: **95.0 %** of the decidable; unclassified 60 — resolver 25, XML 19, both 10, normalisation 3,
  undecidable 3. The class guard (< 0.5 %) is not met; revtex (52) and ams (37) residuals are listed by class in the r3
  report. Side finding (**fixed 78u**, OD #880): `author_block/notes.rs` `classify_thanks` read "erc" inside a word
  as funding ("Perception", 2509.05368). Scratch `~/data/pk_wt/scratch_fms2a2r3/` (rep/final.*, pdf_verdicts.tsv).
  **Integration (int58, S2a.2 r1-r3 squashed onto int57; binary int58a; shadow only, the fleet never sets
  `LATEXML_FM_SHADOW`):** review revfms2a2_3 LAND — unset, nothing new runs on the conversion path (every entry checks the
  switch; 60 papers byte-identical XML + logs vs int54a; +0.034 % instructions on 10 papers × 2 runs), set, the XML is
  unchanged (40 papers; worst time 1.13×, memory 1.06×; 2103.04797's 974 creators 0.62 s both ways), shadow dumps
  deterministic. Folded in by the main session: OD #837 lists all six r3 XML-wrong classes with their witnesses, K23 says
  six, the note-e-mail fallback (the author carrying the note's mark) recorded as the main-session decision of 2026-10-22,
  and the r3 runs credited to their pre-commit build. Main-session decisions on the r3 questions: author-order address
  lists apply to every class (OD #805); `role:url-as-email` is adopted (r2 item 8); a title's note stays in the block (the
  first-author placement is an XML-wrong class); hand-written line labels (`1-`, `1.`) stay unread pending a user ruling
  (0903.2366, 2303.09899). Open for S2a.2 r4 (before the S2b pilot on revtex4_support): the revtex4_support residual to < 0.5
  %; split `observer/resolver.rs` (3,181 lines) by stage and `ab_audit/fmshadow.rs` (1,772) below 1,000 lines; the review's
  nits — `ledger.rs:237-255` identifies the `\author` definition by raw address (keep an `Rc` and `Rc::ptr_eq`),
  `ledger.rs:358-364` `lookup_register` warns (use `lookup_register_quiet`), `ledger.rs:520-531` writes `attrs` in hash
  order (`get_pairs()`), `events.rs:173` `ref_key` matches any `…ref` (`\href`, `\pageref`: an explicit list), `label_text`
  bounds output tokens, not visited ones. Gates int58a: nextest 6912/6912 + cortex 10/10, clippy/rustdoc clean, inits 0/0, dump diff vs int57a
  empty. A/B int57a vs int58a (597: 300 random census + 300 frontmatter-heavy revtex/amsart/IEEEtran/authblk, seed 58):
  byte-identical, 0 of 597 changed. Binding nets: 136/136, 0 documents worse.
  **r4** (OD #837 "S2a.2 r4"; branch pk/fms2a2r4 from int58 b1118a80f1, three commits: the module split d2b81cc429,
  the fixes fa2932c628, the docs): `observer/resolver/` split by stage (labels, lines, blocks, api, post) and
  `ab_audit/fmshadow/` (dump, compare, classes, driver, tests), every file below 1,000 lines — shadow dumps
  byte-identical on 200 census papers and the re-resolved r3 census tables identical against b1118a80f1; the
  revtex4_support residual by class — an empty entry's role and its place in today's contact pile, a
  `\lx@affiliation@withsup` line's first superscript its label (`_withsup=true`), today's contact grid before the labels
  are answered, annotations before any author as `relocate_annotations`' orphans, a prefix-marked name's notes, ORCID
  links printing a name, spacing before a mark; the lexer's `\thanks` inside a superscript, `\lx@orcidlink`,
  `\lx@frontmatter@mark`, numeric box arguments and `\scalebox`'s scales; the comparator's `unobserved` bucket (a block
  the rehearsal could not see, outside the denominator), notes in the personname, an affiliation's formula superscript,
  a label after a joined item's "and", the XML-wrong class "legend lines welded onto the affiliation above them" (7
  PDF-confirmed witnesses); the review's ledger nits (the `\author` owner by `Rc::ptr_eq`, `lookup_register_quiet`,
  attributes in source order, `events::REF_COMMANDS`, `label_text` bounded by tokens visited). Guards
  `perfect_kernel_batchfms2a2r4::*` (15, each red on r3 but the XML pin), `fmshadow::tests` (11, 3 new). Gates (the fixes commit fa2932c628; the docs commit after it changes no code): nextest
  6930/6930, clippy/rustdoc/fmt clean, inits 0/0 with the dumps' `comm -3` against int58a 0 lines each. **A/B**
  (release, int58a vs r4 stamped fa2932c628, int58a dumps): 300 seeded census papers (seed 4005, none of the r3 300) XML
  identical with the variable unset and set (the temp-dir `graphicspath` instruction aside), E 13 = 13 = 13, F 0, W
  3,279 = 3,279 = 3,279 paper by paper; the 979 sectioning-frontmatter repros XML identical (base, off, on), E 23, W 39
  each. **Fresh census** (the 26,322 papers converted with the r4 release binary, the dump's `resolved` equal to
  `--re-resolve`): counted 23,205 — agree 19,528, classified 1,732, unclassified 1,945 (**8.4 %**, from 9.0 %); 111
  held, 2 unobserved; per family (gate): kernel 6,800 (15.5 %), **revtex4_support 5,639 (0.4 %: 23 papers, under the
  class guard)**, ams_support 3,105 (0.7 %: 22), IEEEtran 2,202 (9.8 %), authblk 939 (9.7 %), elsart_support_core 818
  (5.9 %), inst_support 786 (23.1 %), aas_support 688 (4.2 %), mn2e_support 560 (6.6 %), llncs 400 (17.0 %),
  "binding" 317 (8.5 %), iopart_support 264 (6.1 %), sv_support 262 (12.6 %), jheppub 196 (6.0 %), jcappub 93
  (3.9 %), ijcai 59 (72.9 %), JHEP.cls 58 (81.0 %). Against the r3 dumps resolved by the r4 code, the new dumps (the
  lexer's changes) move 16 papers to unclassified (15 kernel: the `\thanks` inside a superscript the shadow now reads
  and today's XML loses, below) and 30 out. **PDF precision** (classified, seeded 404, drawn from the whole census, the
  first 60 of the draw with a PDF; pdflatex on copies or an earlier round's pages): the shadow right 52, wrong 6,
  undecidable 2 — **89.7 %** of the decidable (wrong: 1802.03043 a legend listing every author's affiliation given to
  its leading mark's authors, 2202.06129 a line of two marked entries given whole to the first mark's, 2404.09857 a
  glued mark parted wrongly, 2006.12418 a shared address pattern given to the corresponding author, and two XML-wrong
  classes misapplied: 2609.10596 an affiliation line read as names under "a later names row read as the affiliation",
  1706.00234 a Vietnamese name with spacing accents split under "names parted by a wide space"). The revtex4_support
  residual (23): collaboration lists read otherwise (0807.2167, gr-qc0410007, hep-ex0103033, hep-ex0406068,
  2412.14256), notes after a name's comma given to the next name in the XML with no PDF to confirm it (1705.11098,
  1711.04571, nucl-ex0002003), unmarked address lines (cond-mat0409389, hep-ph0005026, physics0209010), an affiliation
  line as a creator in the XML (2001.06842), and singletons (0907.1243, 1112.1723, 1203.3894, 1309.7542, 1607.07940,
  2008.06624, 2012.13946, 2106.12436, 2504.02039, hep-ex0607072, hep-ph9805510); ams_support (22): name lines
  (0808.2556, 1106.0130, 1703.08733, 2006.13694, 2106.11727, 2109.14517, 2210.11574, 2311.00652, 2411.16601,
  math0112263, math0610203), unmarked lines (1710.07902, 2506.07291, math0610612), and singletons (1005.4252,
  1909.09152, 2108.13723, 2208.03878, 2301.00097, 2402.04675, 2506.11626, 2507.07513). Side finding (output; FIXED by 78u at int61, OD #881): today's kernel `\author{Nataliia
  Babina\textsuperscript{1,2\thanks{Work done …}}}` loses the mark `2` and the note (2512.05464, PDF-confirmed: a
  silent content loss; 15 census papers: 1911.03118, 1912.00833, 2211.09622, 2305.16614, 2306.17472, 2309.12862,
  2402.18447, 2403.04164, 2410.12564, 2501.01423, 2502.15412, 2503.05905, 2512.05464, 2603.17529, 2605.17958). Scratch `~/data/pk_wt/scratch_fms2a2r4/` (rep/final.*, pdf_verdicts.tsv).
  **Integration (int60, S2a.2 r4 squashed onto int59; binary int60a; shadow only):** review revfms2a2_4 LAND — unset, nothing
  new runs on the conversion path (60 papers byte-identical XML + logs; −0.003 % instructions on 10 papers × 2 runs), set, the
  XML is unchanged (40 papers; 1302.1274's one difference is the load-dependent ASF second-chance deadline, byte-identical run
  alone), the split moves code only (shadow dumps byte-identical on 30 papers), the census comes from the committed code (40
  dumps byte-identical). Main-session decisions on the r4 questions: the `\textsuperscript{1,2\thanks{…}}` content loss is an
  output fix (batch 78u); the two misapplied r3 XML-wrong classes are tightened in r5; notes after a name's comma stay
  unclassified; S2b pilots on revtex4_support once classified precision is back ≥ 95 %. Open for r5 (the review's findings):
  `fmshadow/compare.rs:316-320` never flags an XML name note the shadow lacks — 63 "agree" papers hold one (kernel 55,
  `Name\footnotemark[1]` + a later `\footnotetext`, 1108.5155, 1404.2124), which S2b would drop: make it a disagreement and
  correct OD #837's "aastex `\altaffiltext`" sentence; the `unobserved` bucket drops a whole paper for one skipped block and
  three of its four reasons are rehearsal failures (count per reason, compare the observed blocks, S2b falls back to today's
  output for an unobserved block); `events.rs:161-165` comment vs `\pageref`/`\namecref`; `events.rs` 1,133 lines; a sole
  author's pre-author note lands in `contacts` (`resolver/post.rs` `held_before_creators`); `spacing_before_a_mark_parts_the_entries`
  pins a U+200B in the name; a vacuous `.all(…)` at `perfect_kernel_batchfms2a2r4.rs:152-164`; the welded-legend witnesses'
  source evidence into OD #837. Gates int60a: nextest 6951/6951 + cortex 10/10, clippy/rustdoc clean, inits 0/0, dump diff vs
  int59a empty. A/B int59a vs int60a (597: 300 random + 300 revtex/amsart census, seed 60): byte-identical, 0 of 597. Binding
  nets: 136/136, 0 documents worse (int59a vs int60a).
  **r5** (OD #837 "S2a.2 r5"; branch pk/fms2a2r5 from int60 3cba36f53d; review revfms2a2_4 findings 1-9): the r4 sample's
  six wrong papers fixed by class (a note of several legends no one label's, 1802.03043; a glued same-kind mark opening
  an API entry, 2202.06129; a `\footnotemark` inside an `\inst` a mark of its own, 2404.09857; an address pattern the
  block's, 2006.12418; an acronym row no names row, 2609.10596; a gap inside an accent macro, 1706.00234) and both
  misapplied r3 XML-wrong classes fenced; the comparator's name-note gap (finding 1) — the ledger records each
  `\footnotetext` (`\lx@notetext`'s after-digest hook, under the switch) and the resolver pairs it with the names'
  `\footnotemark` as today's `relocate_footnote` does (by the printed mark: 55 of the 63 "agree" papers read; the 8 paired
  by counter value alone, or a second text of a used mark, stay unclassified: 1404.2124, 0909.2782, 1111.4983, 1706.00602,
  1812.04149, 2210.05617, 2507.22611, 2608.04206); the `unobserved` bucket per reason, observed blocks compared (finding 2); ams_support's residual (group
  breaks, an institution's address, ORCIDs in scripts and e-mails, `\and` columns); the nits (events/ split, comments,
  `api:note`, zero-width ink, guard counts). Then the r5 classified sample's own wrong classes (a first draw: 12 of 59):
  unshown-kind labels at their `annotate=` run, no corresponding owner in a multi-address print, address lists over
  lines and before a braced group (26 kernel papers' lost addresses), surname-first names (XML-wrong class
  1503.08374, 1609.07297), API creators' marks, fuzzy labels with kernel accents and internal commands. Guards
  `perfect_kernel_batchfms2a2r5::*` (12, each red on r4 but the XML pin), `resolver::tests` (+1 and 3 assertions),
  `fmshadow::tests` (+3); the r2/r4 guards updated (`sep:column`, no U+200B, counts). **A/B** (release, int60a vs r5 —
  the 300 seeded census papers, seed 5005, none of r4's, and the 981 sectioning-frontmatter repros, the r5 binaries
  stamped f6fd12c4da and 99f35d6bce, int60a dumps): XML identical with the variable unset and set (the temp-dir
  `graphicspath` instruction aside), E 42 = 42 = 42, F 0, W 3,082 each, paper by paper; repros E 23, W 39 each. **Fresh
  census** (the 26,322 papers converted with the release binary stamped 99f35d6bce, the commit's code; the dumps'
  `resolved` equal to `--re-resolve`): counted 23,205 — agree 19,603, classified 1,693, unclassified 1,909 (**8.2 %**,
  from 8.4 %); 111 held, 10 unobserved (all "out of positions", 0 of them disagreeing unclassified in their observed
  blocks); per family (gate): kernel 6,800 (15.1 %), **revtex4_support 5,639 (0.4 %: 24)**, **ams_support 3,105 (0.4 %:
  13, from 22: under the class guard)**, IEEEtran 2,202 (9.8 %), authblk 939 (9.2 %), elsart_support_core 818 (6.0 %),
  inst_support 786 (23.2 %), aas_support 688 (4.4 %), mn2e_support 560 (7.3 %), llncs 400 (17.0 %), "binding" 317
  (8.2 %), iopart_support 264 (6.1 %), sv_support 262 (12.6 %), jheppub 196 (4.9 %), jcappub 93 (3.9 %), ijcai 59
  (72.9 %), JHEP.cls 58 (81.0 %). 170 papers changed status against r4. **PDF precision** (classified only, a seeded
  hash-ordered draw from the whole census — `sha256("505:"+key)`, stable under small changes of the classified set, the
  first 60 with page text; pdflatex on copies): right 46, wrong 10, undecidable 4 — **82.1 %** of the decidable
  (informational since decision 1 below: the gate is per family). The wrong ones: four apply ruling 4's "a note's inner contacts its owner's" to a note whose links
  are another author's or the paper's (2001.02600, 1803.04996, 2504.08644, 2602.13176: main-session decision 2 below), two
  give a mixed legend or a symbol entry to the wrong name (2306.14857, 2304.07838), and one each a names-run address to
  the corresponding author (2207.11290), a two-name correspondent to the first only (2410.19543), glued non-footnote
  symbols unparted (2510.17795), "for the Milagro Collaboration" as an affiliation (astro-ph0611457). The earlier draws of
  this round (each fixed by class before this one) found 12 of 59 and 4 of 57 wrong. All 11 revtex4_support papers of
  the sample are right (r4: 19 of 19 — one of them, 1412.5812, judged against a wrong r4 shadow and re-checked right
  under r5 by the landing review: r4's method missed 1 in 19; 30 of 30 holds for the r5 code). Residuals — revtex4_support (24): r4's 23 less nothing, plus hep-th0302032 (a
  footnote paired by counter value); ams_support (13): 0808.2556, 1005.4252, 1703.08733, 1909.09152, 2108.13723,
  2109.14517, 2208.03878, 2210.11574, 2301.00097, 2311.00652, 2507.07513, math0112263, math0610203. Side findings
  (output, not fixed — shadow only): today's XML loses an ORCID link set in a name's superscript
  (`Zhiyuan Yao$^{\orcidlink{…}}$`, 2208.11385; 10 census papers) and a `\thanks` inside an `\inst` (`M.~Briquet\inst{6,
  \thanks{…}}` prints "Belgium" in a `sup`, 1307.5791) — the 78u content-loss class. Scratch `~/data/pk_wt/scratch_fms2a2r5/`
  (rep/final.*, pdf_verdicts.tsv, verdict_db.json).
  **Integration (int63, S2a.2 r5 squashed onto int62; binary int63a; shadow only):** landing review revfms2a2_5 LAND —
  unset, nothing new runs on the conversion path (the only addition, `\lx@notetext`'s after-digest hook, one cached
  check); 40 papers outside the writer's 300 byte-identical across base, unset and set; the census and its per-family
  table reproduced byte for byte. Main-session decisions on the writer's four questions (2026-10-22): **(1)** S2b gates per
  family — a family migrates when its own seeded classified-precision sample (n ≥ 30) is ≥ 95 % right and its unclassified
  rate < 0.5 %, the cross-family bucket informational: revtex4_support qualifies (30/30, 0.4 %) and is the S2b pilot;
  ams_support meets the unclassified bar only (5 papers in r5's draw, 3 right, 2 undecidable — no sample of its own yet).
  **(2)** a note's inner contacts go to its owner only when they spell the owner or are its only link, the rest stay in the
  note text (user ruling 2026-10-20 #4, no guessing) — the code still gives every inner contact to the owner
  (`blocks.rs` `note()`, `ruling4:inner-contact`): r6 target, witnesses 2001.02600, 1803.04996, 2504.08644, 2602.13176.
  **(3)** a braced `{a,b,c}@x` group in an `\institute`/`\affil` entry is written out and each address goes to the author
  it spells, an unspelled one to the block's shared creator (user rulings 2026-10-20 #4/#8/#10; ~115 llncs/inst_support
  papers then move from agree to classified) — r6. **(4)** the corresponding-author fallback only for an address printed
  alone — consistent with user ruling 2026-10-21 #14, confirmed. Review findings for r6: a replaced `\author` call's block
  items are dropped (`resolver/mod.rs:396-407`, 2510.26354 — undocumented, unguarded) and `answer_note_texts`
  (`post.rs:463ff`) still pairs a footnote text with a replaced creator's mark; `new_creator` drops name pieces with no
  letter or digit (visible ink: a printed U+2010) while the join paths keep them — decide printed text vs normalised key;
  `NoteText.value` is dead data; an address-list continuation skips its lines' notes and contacts (`blocks.rs` `skip_to`).
  Side finding (both engines): revtex's `\altaffiliation` is `annotate=new` (Perl revtex4_support.sty.ltxml:61), so
  cond-mat0310516's "Present address" goes to all six authors before it where the PDF marks Mikami only — a SYNC row.
  Gates int63a: nextest 6986/6986 + cortex 10/10, clippy/rustdoc clean, inits 0/0, dump diff vs int62a empty. A/B int62a
  vs int63a (300 random census controls, seed 63, + `pop78u_1` + 200 of `pop78u_2`; 528 of 531 converted): XML byte-identical
  on all 531, E 149 = 149, F 1 = 1; FM shadow (int63a unset vs `LATEXML_FM_SHADOW=1`, the same 531): XML identical. Binding
  nets: 136/136, 0 documents worse (int62a vs int63a).
  **r6** (OD #837 "S2a.2 r6"; branch pk/fms2a2r6 from int63 410a2a85e8; review revfms2a2_5 findings 1, 2, 5, 6 and the
  main session's decisions 2-3): a replaced `\author` call keeps nothing — the post-passes read the live creators only
  (`Resolver::live`: the footnote-text pairing, a sole author's orphans, the address list's order, the upfront grid; the
  replaced call's block items go with it, 2510.26354); a resolved name is its printed text (`new_creator`'s piece filter
  removed); a note's inner contact is its owner's only as the note's only link with its mark shown on the owner, or by
  spelling the owner — the rest stay in the note's text (decision 2), and IEEEtran's title notes print no mark
  (IEEEtran.cls:4756 `\def\thefootnote{}`; policy `IEEEtran`, `unmarked_notes`), so there only spelling places a link
  (2001.02600, 1803.04996, 2504.08644, 2602.13176 now agree); an entry's `\email` printing several addresses (`\{a,b,c\}@x`)
  is written out, each to the author it spells among the document's, an unspelled one to the block (decision 3,
  `ruling3:written-out`, a comparator change tag; 1904.07206, 2302.05745); `NoteText.value` documented as diagnostic; an
  address list's lines place their notes, the block's (`note:address-line`). Guards `perfect_kernel_batchfms2a2r6::*` (7,
  each red on r5 but the XML pin), `resolver::tests` (the U+2010 kept), `fmshadow::tests` (+2). **A/B** (release, int63a
  vs r6 stamped a5bad80abe, int63a dumps): 300 seeded census papers (seed 6006, none of the earlier rounds' 300s nor the
  r5 review's 40) XML identical with the variable unset and set (the temp-dir `graphicspath` instruction aside), E 103 =
  103 = 103, F 0, W 4,284 each, paper by paper; the 985 sectioning-frontmatter repros XML identical (base, off, on), E 23,
  W 39 each. **Fresh census on int63** (the 26,322 papers converted with the release binary stamped a5bad80abe; the dumps'
  `resolved` equal to `--re-resolve`, tsv byte-identical): counted 23,205 — agree 19,505, classified 1,820, unclassified
  1,880 (**8.1 %**; the same dumps under the int63 resolver, `ab_audit` built from 410a2a85e8: 19,620 / 1,695 / 1,890 —
  int63's XML side, 78u, moved 20 papers against r5's int60a census, 14 kernel and 2 IEEEtran unclassified → agree); 111
  held, 10 unobserved; per family (gate): kernel 6,800
  (14.8 %), **revtex4_support 5,639 (0.4 %: 24, r5's list unchanged)**, **ams_support 3,105 (0.4 %: 13)**, IEEEtran 2,202
  (9.6 %), authblk 939 (9.2 %), elsart_support_core 818 (6.0 %), inst_support 786 (23.1 %), aas_support 688 (4.4 %),
  mn2e_support 560 (7.5 %), llncs 400 (15.0 %, from 17.0 %), "binding" 317 (8.2 %), iopart_support 264 (6.1 %),
  sv_support 262 (11.8 %), jheppub 196 (4.9 %), jcappub 93 (3.9 %), ijcai 59 (72.9 %), JHEP.cls 58 (81.0 %). Against the
  int63 resolver, 158 papers move: decision 3 moves 130 agree → classified (llncs 109, inst_support 16, sv_support 4,
  authblk 1) and 11 unclassified → classified (llncs 8, sv_support 2, inst_support 1); decision 2 moves 14 classified →
  agree (IEEEtran 12, kernel 1, authblk 1); finding 1 moves 1611.08655 unclassified → agree (a replacing call's sole
  author takes the block's address); 2 move classified → unclassified (2406.07529:
  the `\vphantom{\thanks{Equal contribution.}}` today's XML loses, now the block's note in the shadow; 2407.21626: the
  name ": Matilde Barberi-Squarotti", r5's piece filter having hidden the split's colon). Decision 3's PDF sample (10 of
  the 130, seeded, pdflatex on copies; 2109.08655's garbled page text replaced): right 9 — two of them leave a
  human-evident address to the block as the no-guessing rule requires (1307.2669 `vpest283@`, 2302.05745 `guyam@`
  shared by two Guys) —, wrong 1 (1911.11496: `duerrschnabel@` unspelled for Dürrschnabel, `ü` written `ue`).
  **revtex4_support precision** (decision 1's gate; a fresh seeded draw, `sha256("606:"+key)` over its 381 classified-only
  papers, the first 41, two without a PDF): right 38 of 39 — 2308.13439, 1710.04348, 2506.05954, 2101.11090, 2605.28357,
  2102.01842, cond-mat0207129, 0909.4814, 0812.0321, 1111.5058, 1603.06794, 1503.02813, 1905.08661, 2008.10319,
  1406.0038, 2605.26378, cond-mat0502298, cond-mat0404120, 0812.1531, 1607.08533, 0707.1549, 0906.2758, 1512.06435,
  1310.1663, 2406.18974, 2012.09251, 2501.04244, 2003.03434, 0804.4669, 2101.02613, 1901.05740, 2310.04763, 2209.05548,
  2602.20526, 2203.06767, 1812.04462, 2412.05291, 2210.15537 —, wrong 1 (1607.07280: a `\footnote[1]` mark on a name
  answering `\affiliation{\footnotemark[1]…}`, left to the block); no PDF: cond-mat0007316, hep-ph0002275; the rows of
  all 41 identical on the r5 dumps re-resolved. **revtex4_support passes the S2b gate on int63** (97.4 % ≥ 95 %, 0.4 % <
  0.5 %). Residuals: the two classes above (1607.07280; 1911.11496); a name's edge glyphs the split leaves, visible since
  the filter's removal (7 census names: 0806.2911 `∥`, 0907.1243 `⋄`, 2003.01448, 2311.00652 `|`, 2407.21626 `:`,
  2511.07187 `--`, 2511.18580 `⋅` — marks or separators the lexer does not read as such). Side finding (output, not fixed):
  today's XML loses a `\thanks` set inside `\vphantom` after an author block's last address (2406.07529). Scratch
  `~/data/pk_wt/scratch_fms2a2r6/` (rep/final.*, rep/int63_rr.*, rt60.txt, d3_10.txt).
  **Landing (int66, r6 squashed onto int65; binary int66a):** gates int66a: nextest 7017/7017 + cortex 10/10, fmt/clippy/rustdoc
  clean, inits 0/0, dump diff vs int65a empty; A/B int65a → int66a on int65's 724 papers byte-identical with `LATEXML_FM_SHADOW`
  unset AND set (E 169 = 169, F 1 = 1, words equal). Binding nets 136/136, 0 worse than int64a. Review revfms2a2_6 LAND — shadow-only proven again (no hook added; the
  policy table read only after `records_now()`; 40 papers, seed `rev6-77` + 10 witnesses, XML identical base/unset/set);
  census, revtex4_support (the same 24 unclassified as r5) and the 158 moves reproduced byte for byte; 5 of the 38 revtex
  "right" and 3 of decision 3's 10 confirmed against arXiv PDFs. Main-session decisions on the writer's questions: (1)
  an unmarked-notes class (`IEEEtran` policy) links a note's address to an author only by spelling — confirmed; (2) the
  7 edge glyphs are r7's (the lexer reads them as the marks/separators they are); (3) an address matches any author of
  the document (1904.07206) — confirmed; (4) `ü` written `ue` (1911.11496) is an output row (shared spelling helper).
  **r7 queue** (before the S2b pilot): `answer_sequence`'s `in_order` compares a live place with an all-creators index
  (`post.rs:100-103`; a replacing `\author` under a `SpelledElseOrder` class) — fix + guard; a url that spells a co-author
  goes to that author as an e-mail does (`blocks.rs:689-693` vs `post.rs:446ff`; 2001.02600 Peng Xu's homepage — main
  session: route urls through the spelled-author pass); `note()` decides an e-mail "taken" before the whole block is read
  (`blocks.rs:666-688`; `\author{Ann\thanks{ann@x.org, bob@x.org} \and Bob}` would print both twice under S2b) — defer to
  `answer_note_emails`; `label_owners`' `author:N` indexes all creators where `place.rs:94` numbers the live ones; the 7
  edge glyphs; 1607.07280 (`\footnote[1]` mark not read as `\affiliation{\footnotemark[1]…}`'s request); and a census on
  the int66 binary to re-confirm the revtex4_support gate (int64's 78v changed the XML side).
  **r7** (OD #837 "S2a.2 r7"; branch pk/fms2a2r7 from int69 f24574e043; the r7 queue above): `answer_sequence` compares
  the live creator at a place with its spelled owner, `label_owners`' `author:N` is the N-th live creator; a note's
  e-mails and urls are placed once every name is read (`answer_note_links`): a url spelling a co-author is that author's
  as an e-mail is (2001.02600 Peng Xu), an address-group url only its own name's (1407.7882), and a note its links may
  empty is kept or dropped there; the lexer reads `∥` (`$\|$`) as `\@fnsymbol`'s `‖`, the mark glyphs (`⋄ ♯ ⋆ ⊕ …`) as
  marks of no footnote-symbol kind (their legend lines legends), a bar, colon, math dot, bullet or dash standing alone as
  a word separator (6 of the 7 edge-glyph names; 2003.01448's `J .` is the source's print, a dash inside a word stays as
  typed — hep-ex9811040); a `\footnote[1]`'s mark answers `\affiliation{\footnotemark[1]…}` (`note_marks`,
  `mark:footnote`: 1607.07280, hep-th0302032, nucl-th9803030, 2608.27295 — their affiliations now right;
  hep-th0302032 keeps 2 unclassified rows, the e-mails its names hold as footnotes, and stays among revtex's 23) — its print read by
  `note_mark_quietly`, which unwinds a runaway (jpconf.cls:153's `\@fnsymbol`: 1105.2659 went Fatal and 0709.0673 lost
  its name on the first r7 build, found by this census) and holds no rehearsal past `\@fnsymbol`'s nine; the comparator
  explains a line of a continued entry (`part …`), a legend note against today's affiliation and back. Guards
  `perfect_kernel_batchfms2a2r7::*` (12), ab_audit `fmshadow::tests` (+4), the r1-r3 guard's `⋆` legend a note.
  **Shadow-only proof** (release 5895eedfeb, int69b vendor dumps): int65's 724 papers (725 zips) and the 989
  sectioning-frontmatter repros XML byte-identical int69b → r7 with `LATEXML_FM_SHADOW` unset AND set (E 179 = 179, F 1,
  W 8,425 each, words 9,802,513 each; repros E 23, W 39 each); the census's 26,323 papers int69b-set vs r7-set XML
  identical paper by paper but 2109.00764's printed clock time, E 5,759 / F 18 / W 333,911 both. **Census on int69b**
  (int69b binary, shadow set): byte-identical to r6's (0 status moves, the disagreement rows identical; 2509.13772 now
  converts) — 78v/78y/79a moved no paper. **r7 census** (26,323, the dumps' `resolved` = `--re-resolve`): agree 19,502,
  classified 1,888, unclassified 1,816 (**7.8 %**, r6 8.1 %); the comparator alone (r7's on int69b's dumps) moves 62
  unclassified → classified (157 rows `part`, 97 `moved` legend rows, 4 `overlaps`), the resolver and lexer 7 more
  (IEEEtran 4 agree → classified, the url rule; kernel 1402.2255 agree → classified, `♯` lines; ams_support
  2311.00652 unclassified → agree; mn2e 2407.21626 unclassified → classified) and the rows of 40 papers in all, none
  to unclassified; 111 held, 10 unobserved as before. Per family (gate): kernel 14.4 %, **revtex4_support 5,639
  (0.4 %: 23, r6's 24 less 0907.1243)**, ams_support 0.4 % (12), IEEEtran 8.4 %, authblk 8.9 %, mn2e 7.3 %, the rest
  unchanged. **revtex4_support gate**: the seeded `sha256("606:"+key)` draw over its 382 classified-only papers gives
  r6's 41 again; 40 have the rows they had, 1607.07280's changed and now matches the PDF — **39 of 39 right**
  (cond-mat0007316, hep-ph0002275 no PDF): passes (≥ 95 %, 0.4 % < 0.5 %). Residuals: a glued two-symbol run one line
  and one name show as written (`§†`, 0907.1243 — each line to every carrier of either; the one census paper; r8),
  repository urls under an author's account become that author's (2008.12970, 2511.08416 — r8's page rule),
  `M$^{\rm c}$Hardy` (2003.01448) parted.
  **Landing (int72, 458382f335 cherry-picked onto int71 d56a5027de as 83c5f9ee04; binary int72a):** review revfms2a2_7 LAND
  (shadow-only proven again on 40 papers, seed `rev7-` + 10 witnesses incl. 1105.2659 and 0709.0673; the main-session decisions
  on the url homepage rule, glued symbol runs, mark glyphs and ownerless notes went to r8 with its findings 1-4). Gates int72a:
  nextest 7133/7133 + cortex 10/10, fmt/clippy/rustdoc clean, inits 0/0, dump content diff vs int71a empty; A/B int71a → int72a
  on int65's 724 papers byte-identical with `LATEXML_FM_SHADOW` unset AND set (E 169 = 169, F 1 = 1, words equal, 0 changed).
  Binding nets 136/136, 0 worse than int71a.
  **r8** (OD #837 "S2a.2 r8"; branch pk/fms2a2r8 from r7 458382f335; review revfms2a2_7 findings 1-8 and the main
  session's decisions 1-4 on r7's questions): a creator's notes and contacts, and the block's own, in source order once
  `resolve` ends (`in_source_order` — a note kept once every name is read came after the later ones,
  `[Second note., ann@x.org, zed@y.org]`), and the comparator sees note order (`note-order` rows, unclassified unless a
  ruling placed the first note out of today's place); an ownerless note's link to its speller or left in its text, never
  the block's (`fallback` asks an owner); a url an author's only as the page it spells (`link_page`: the last path
  segment, an `index.*`/`*.html` page aside, else the host; read from an `\href`'s target) — repositories, projects and
  files under an account stay in the note (2008.12970, 2511.08416, 2103.05251, 2503.02954, 2509.01952), the 21 homepages
  of 12 census papers stay their authors' (1409.7028, 1603.02588, 1909.06689, 2001.02600, 2103.04211, 2210.15991,
  2301.03638, 2601.07017, 2604.11526, 2609.10517), and a page no name spells is no one's, the note's only link too
  (1401.0489's `~laci` among them); an address link is the address (`\href{mailto:…}`, an address-group `\url`);
  jpconf's note marks its arabic `\thefootnote` (`TitleMarks::Body`: 1105.2659 shows "3", its PDF's) and a bare
  `\footnotemark`'s print read quietly; a glued symbol run a line leads with and a name shows one mark
  (`glue_shown_runs`: 0907.1243's Ching has only the Missouri line, French only DuPont, as the PDF — still classified,
  its other rows today's every-line-to-everyone); a text mark a footnote symbol only of `\@fnsymbol`'s characters; an
  unlabelled entry no legend; an API entry's correspondence or equal-contribution legend a note (0812.1531, 2605.28357,
  2501.04244; not a funding word, 0710.0802's Capital Fund Management); the comparator's `part` line-exact (the dump's
  `continued`), an API call's affiliations-plus-legend one contact explained as their sequence, a collision of two
  items of one text keeping a ruling's rule (2509.23109). Guards `perfect_kernel_batchfms2a2r8::*` (10), five re-pinned
  (fms2a2 3, r6 1, r7 1), engine unit tests (2), ab_audit `fmshadow::tests` (+2). **Shadow-only proof** (the r8 release's
  code, int69b vendor dumps): int65's 725 papers and the 989 sectioning-frontmatter repros XML byte-identical
  int69b → r8 with `LATEXML_FM_SHADOW` unset AND set (E 179, F 1, W 8,425 each, words 9,802,513 each; repros E 23,
  W 39 each, one `\today` past midnight); the census's 26,323 papers r8-set and r8-unset each against int69b-set: set
  identical but 2109.00764 and 2208.11989 (a printed clock, `\today`), E 5,759 / F 18 / W 333,911 all three; unset the
  same plus 2109.00445 (a time-budgeted math parse; three reruns give the base) and three papers whose XML int69b and r7
  print differently with the switch unset and set — a pre-existing rehearsal leak (row below; r7's proof compared
  int69b-set with r7-set only). **r8 census** (26,323, `--re-resolve`; the same rows as r8's resolver on r7's dumps):
  agree 19,510, classified 1,882, unclassified 1,814 (**7.8 %**, r7 1,816): 30 papers moved, none to unclassified — 18
  classified → agree (the url rule: repositories, projects and fallback urls left in their notes), 10 agree →
  classified (API legends as notes 8, 2206.14396's the block's; a homepage behind `\href{…}{work}`, 2210.15991;
  2607.03557's note order), 2 unclassified → classified (1802.05992's block legend explained as today's affiliation,
  1910.01401's second ORCID print by its ruling-4 rule); one `note-order` row in the census, 2607.03557's (classified: today's relocates
  the legend last). Per family as r7 but kernel 14.3 % (967), elsart_support_core 6.0 %, binding 8.2 %. **revtex4_support
  gate**: 5,639 papers, **23 unclassified (0.4 %)**, the same 23 as r7 (hep-th0302032 among them); 383 classified-only
  (1412.0709 joins, an API legend); the seeded `sha256("606:"+key)` draw gives r7's 41: 38 with their r7 rows, 3 with
  roles changed only — 0812.1531, 2605.28357, 2501.04244's `†`/`∗` correspondence and equal-contribution entries now
  legend notes, as their PDFs print them — **39 of 39 right**. 0907.1243 classified, Ching and French now as the PDF.
  Residuals: jpconf's `\footnote[<n>]` sets the counter where the rehearsal's binding does not (a later bare
  `\footnotemark` shows the kernel's count); a homepage no name's spelling reaches (1401.0489's `~laci`) stays in its
  note.
  **Landing (int73, 412f9d4dd1 cherry-picked onto int72 dc5b58276a as 6604bc49cf; binary int73a):** review revfms2a2_8 LAND
  (shadow-only proven again on 40 papers, seed `rev8-` + 10 witnesses; the main-session decisions — contacts sorted with the
  notes, the strict url fallback, the rehearsal leak its own round — stand; S2b on revtex4_support GO once a link contact
  carries its target). Its findings go to r9: a link contact keeps the link's anchor text, not its target (2210.15991 `url
  work`, 2309.10007 `url tsamak`; a `mailto:` stays role `url`) — blocks S2b's first written XML; the API legend rule takes a
  whole affiliation entry for a legend (1903.06934); the comparator's note-order check reads only the first differing note and a
  same-text collision keeps one rule per key; nits (`continued` optional, `link_page` edges, span-less order, `is_symbol_char`).
  Gates int73a: nextest 7147/7147 + cortex 10/10, fmt/clippy/rustdoc clean, inits 0/0, dump content diff vs int72a empty; A/B
  int72a → int73a on int65's 724 papers byte-identical with `LATEXML_FM_SHADOW` unset AND set (E 169 = 169, F 1 = 1, words equal,
  0 changed). Binding nets 136/136, 0 worse than int72a.

- **77x** (T3 font map, user ruling 2026-10-20: phonetic symbols are author content; guards
  `perfect_kernel_batch77x::*`, 6; OD #781, KPE #1004). tipa's T3 has a map (t3_fontmap.rs: each slot the glyph tipa.mf's
  parts draw, as Unicode's IPA letter, cross-checked against tipauni.sty; the `""`/`||` ligatures ˌ ǁ), so its input
  letters (`\textipa{[D@"lIm@]}` [ðəˈlɪmə]), named symbols and `\;` `\:` `\!` `\*` composites print; t3enc_def.rs sets
  t3enc.def's `\upper@accent`/`\lower@accent` as combining marks (`\textsubring{n}` n̥, `\~*{a}` a̰, `\|[{t}` t̪, the
  kernel accents in T3 taking t3enc.def's commands), `\ipabar` and `\textsuperimposetilde` as overlays, the made letters
  Unicode has as its letters (ƀ đ ǥ ƻ ƾ ʣ ʥ ʨ); tone_sty.rs writes `\tone{51}` as ˥˩. `\DeclareTextAccentDefault` and
  `\UseTextAccent` are latex.ltx's (a T3 accent in T1 text keeps its mark and its T1 letter; in math the letters alone),
  a text symbol at an empty slot of a mapped encoding warns, and T3 has no dash ligatures. Census: 1,550 of 2601-2609's
  papers load tipa (306 with T3 content). A/B vs int36a (release, `--preload=ar5iv.sty`; 40 T3-content papers, 18
  tipa-only, 2405.01972, 1802.05444): E 93 → 93, F 1 → 1 (2605.10232's timeout, both), W 432 → 354 (the T3 notices), 32
  outputs changed, each the IPA as the PDF prints it (2405.01972 ɲ ɓ ɡɔ; 2606.11542's clicks ǀ ǁ ǂ, 2106.15390 247 ʃ,
  2604.00174 /rʃʧ/, 2603.05213 /ɪ̃/); no paper worse. Inits 0/0, dumps identical to int36a. Residuals: glyphs Unicode
  has no character for print nothing with a Warning (the hooktop, the Celtic palatalization mark, tone-letter parts
  reached outside tone.sty, the Viby and left-hooktop long letters, `\textoverw`'s mark); a box accent (`\sups`,
  `\Upperaccent`'s `b` form) keeps t3enc.def's stacked boxes.
  **r2** (rev77x NO-SHIP + user ruling 2026-10-20, T3 by the IPA meaning; 11 more guards, each RED on r1; OD #781,
  KPE #1004). r1 made tipa.sty:429-458's `\!` `\;` `\:` T3 text accents that read the next token (`&`, `$`, `}`) as
  their argument: 2603.03017 E 0 → 12, 2605.00529 E 0 → 66 (16,534 words lost), 2608.02769 E 0 → 14. amsmath_sty.rs
  now re-asserts the kernel's `\,` `\!` `\:` `\;` `\negmedspace` `\negthickspace` as amsmath.sty:155-175 does
  (`latex_constructs::define_amsmath_tmspace_commands`); with amsmath before tipa (pdflatex errs) the robust
  `\UseTextAccent` takes no `&`/`$`/`}`/`\end`/row end as its argument and sets the accent alone with a Warning
  (2609.13770's `\crefname{section}{\S\!}`; 2605.00132 and 2607.12821 exercise the robust `\UseTextAccent` and its math
  path, no refusal). t3enc.def/tone.sty read twice no longer hang. T3 slots follow tipauni.sty: `!` ǃ, `&` ɞ,
  `\textcorner` U+031A, `\textsubbar` U+0320 (`\textopencorner` stays ˹); `\textrthook`/`\textpalhook`/`\textcorner`
  attach to the letter before them. `\ipabar` overlays character runs only (`\textdoublebaresh` printed TeX source).
  `\accent` performs the assignments before its character (tex.web §1123/§1270), so KPE #1004's
  `\UseTextAccent{OT1}{\'}{e}` is é. OT4/LY1 `\j` ȷ, LAE shadda vowels mapped; `\tone{6}` warns. A/B vs r1:
  245 papers (r1's 60, rev77x's 42 tipa + 75 kernel + 58 cell/bang, the 61 `\!`/`\;` exposure papers, witnesses;
  release, `--preload=ar5iv.sty`): E 246 → 154, F 1 → 1 (2605.10232's timeout, both), W 3732 → 3734, 16 outputs
  changed — the three witnesses E 0 (2605.00529 +16,534 words, 2603.03017 +4,038), tipa-then-amsmath formulas get their
  `\;` `\!` spacing back (2601.02906, 2603.22709, 2604.04825, 2604.05410, 2607.19079, 2608.22273), amsmath-then-tipa
  cells read in math as pdflatex (2607.12821, 2607.16418), 2606.11542's clicks ǃ (52 words), the corner U+031A by the
  ruling (2604.00122's Quine corners ˹x̚, 2609.04232), `\S\!` a zero-width space as without tipa (2604.17105).
  That A/B's worse papers, 2609.36534 (91 numbers), 2608.02769 (`\mathbf 1\!`) and 2603.03017 (`(1 \! - \! f)`),
  lost a number before `\!` — a pre-existing Rust-only loss of every document (the silent-loss row above), which r1's
  and int36a's `\!` hid — now fixed in base_xmath.rs's number ligature (Perl's empty `\!` space char ends the scan);
  its A/B vs the build before (66 papers: 30 of a 500-paper 2607 sample's 46 with a digit before `\!`,
  the 24 such of the 245, the changed ones; 2605.10232 times out in both): E 7 → 7, W 1591 → 1575, 26 outputs changed,
  every one numbers back (2609.36534 +91, 2606.16857 +68 "1", 2607.00847 +54, 2607.11470 𝟙) or symmetric operator
  spacing, none lost; the Rust golden `tests/babel/numprints.xml` had the loss baked in (`\numprint[\!{}^{\prime
  \prime}]{6.00117}` without its 17) and now matches Perl's. 2609.13770 +1 Warning (`\crefname{section}{\S\!}`: the binding's group closes after `\!`,
  where pdflatex reads a `~`). Residuals (rev77x2, 0 census uses each): an accent proper in a formula loses its mark
  (`$\textsubring{n}$` n, pdflatex n̥); a T3 accent over a math group or `\\` prints TeX source ("$̥x$", as the kernel's
  `\'{$x$}` already does — apply_accent should mark only a character run); a combining-mark slot (36, 37, 60, 62, 94)
  with no letter before it in its text node attaches to the preceding space (`{\textcorner}x`, `\textipa{<s}`,
  2604.00122 after `</math>`); `{\accent19\selectfont\'e}` gives é́ (pdflatex "´é"; int36a's `\accent19\"o` ö́ the same
  class); `\textdoublebaresh` sets ʃ with its bars as rules beneath it, without a Warning.
  **Integration (int37, 77x r1-r3, binary int37a):** T3 lands alone (77p, review SHIP at r11, lands as int38). Reviews:
  rev77x NO-SHIP (tipa's `\!` `\;` `\:` swallowing a boundary token after amsmath — 2603.03017, 2605.00529, 2608.02769;
  double t3enc.def load; the user's IPA-meaning ruling) → r2; rev77x2 SHIP → r3 docs (KPE #1005, refusal witness,
  residuals, LAE guard whole). Gates int37a: nextest 6360/6360 + cortex 10/10, clippy/rustdoc clean, inits 0/0, dump
  diff vs int36a empty. Population A/B (653: 335 papers of the 77x rounds and reviews — tipa census, `\!`/`\;` exposure,
  digit-before-`\!`, Greek/mathdesign/Cyrillic kernel set — + 318 random of the proj98b sample) vs int36a: E 4,116 →
  4,116, F 22 → 22, 0 Error or Fatal worse, words +1,812, no word loss > 1 %. Binding nets (int37a) 136/136, 0 documents
  worse than int36a.
- **77p** (author blocks: spacing-run name separators, e-mails by spelling under the no-guessing rulings R10-R12, link
  lines as URL contacts and mailto lines as e-mail lines, `\@fnsymbol` symbol marks and symbol-labelled lines placed by
  their carriers per row, one non-printing-argument table for address text, aero.cls binding; OD #52(o), #775; rows above
  and the e-mail/symbol-mark rows). Eleven rounds; rev77p8 SHIP under the stopping rule (no new blocker), its remaining
  should-fixes, nits and the int34a-same shapes are batch 77y.
  **Integration (int38, 77p r11, binary int38a):** gates int38a: nextest 6469/6469 + cortex 10/10, clippy/rustdoc clean,
  inits 0/0, dump diff vs int37a empty. Population A/B (1,891: 1,596 papers of the 77p rounds and reviews + 291 random
  of the proj98 sample + 4 html_feedback) vs int37a: E 7,195 → 7,172, F 74 → 74, 0 Fatal worse, 19 Error better; 2
  Error worse are honest `merged_creators` diagnostics of merges int37a already made silently (2401.16553 three names in
  one creator, now with its affiliation and e-mails split off; 2602.09443 the legend line as a creator, now flagged);
  3 word losses > 1 % are affiliation lines no longer copied onto every name (2603.07345, 2607.20027) and one jammed
  affiliation string now 26 per-institution affiliations (2605.10858). Binding nets (int38a) 136/136, 0 documents worse
  than int37a.
- **77v** (quality batch 4: package bindings that lose text or mislabel it, quality inventory rows 42, 62, 14, 57, 78,
  95, 108, 104, 112, 91; guards `perfect_kernel_batch77v::*`, 13, each RED on 529dc1b543 but the sn-jnl one; OD #782,
  KPE #1006-#1013). thm-restate's `restate=[<keys>]<name>` key stores the theorem for `\<name>` as `{restatable}` does
  (`thmtools_use_keys`, `\lx@thm@restate@store`; 2108.11527, 2602.09462); todonotes' `\todo[<options>]` options are keys,
  not the mark; KOMA's `\setbibpreamble` is typeset after the bibliography heading (koma_script.rs); a `\newtheorem` with a
  counter of its own drops an earlier shared-counter mapping ("Proposition 1.0", "Lemma 0" → their numbers: 2407.21129,
  2602.12471, 2407.02950, 2407.10640's thmtools theorems over acmart's); `\newcounter` keeps an existing `\the<ctr>` with
  LaTeX's warning (llncs `\let\c@case\relax\newcounter{case}`, 2303.08726; not over a fallback's guess or under a
  stand-in's copy; pdflatex's warnings, now ours too: cryptocode.sty:636-644's four `\newcounter{H…}` meeting the
  kernel's `\theH<ctr>`, +4 W per cryptocode paper, and the tcolorbox template's `\newcounter{example}` after `use
  counter=example`, 1912.06298); `{multline}` numbers as `{equation}`, so `\tag`/`\notag`/`\nonumber` act (hep-th0210191); a text-mode
  `{IEEEeqnarrayboxm}` is inline math of its own; amsppt's `\footnote`/`\footnotetext` are non-`\long` (`short => true`,
  OD #782); combine loads the class its options name; endnotes' `\theenmark` is the note's mark. sn-jnl's table rules
  verified fixed since 67z (guard added). A/B vs int37a (release, `--preload=ar5iv.sty`; 410 papers: 18 thm-restate, 30
  `\todo[`, 3 `\setbibpreamble`/`\bibpostamble`, 30 sn-jnl, 18 `\let\c@…\relax`, 30 `[section]` theorems, 30
  arxbj/elsart/tac, 40 multline (30 with `\tag`/`\notag`), 12 IEEEeqnarraybox, 30 amsppt `\footnote`, 30 endnotes, 30
  KOMA, the 9 witnesses, 100 random of 2108/2310/2404/2407/2602 and five 1990s months): E 406 → 314, F 5 → 5, W 8329 →
  8330 (2303.08726's LaTeX warning), 53 outputs changed, all improvements — 18 thm-restate papers lose every undefined
  `\<name>` error (their E 112 → 21, the rest other causes) and print a reference to the stored theorem ("See
  ⟨ref⟩") where pdflatex restates it in full (residual row "A restated theorem prints a reference"), 29 papers' multlines number as
  pdflatex (`\nonumber`'d ones unnumbered, `\tag{24}` tagged, later numbers back in step), 4 papers' theorem numbers
  (2407.21129, 2602.12471, 2407.02950, 2407.10640), 2303.08726's cases; no word loss above 1 %. A further 30 papers with live `\todo[`: no output changed (none prints a note), 2602.24285 +1 Error
  (its `\if\todo[…]{…` comment hack around an unclosed `\todo` brace: `\if` now expands `\todo` with its arguments
  and the skip misses the `\fi` inside them; pdflatex's `\todo` peeks by `\futurelet`, its `\if` is true; XML
  byte-identical). Residuals: `\todo`'s `author=` name is not kept; apacite's first-citation author list and
  `\citeyear` parentheses (bibliography row above); pdflatex's `\@xnthm` "Command `\theX' already defined" warning for
  a `\newtheorem` over a shared counter is not emitted; a restated theorem prints a reference, not its statement (same
  row). r2 (review rev77v; guards `perfect_kernel_batch77v::*` +6, each RED on d486e9cbc0): `{multline*}` keeps a
  `\tag` (the `{equation*}` path: "(Z)"); a second `\newtheorem{X}` of a theorem the document's world declared is
  refused as latex.ltx `\@nthm`'s `\@ifdefinable` refuses it ("Command \X already defined", an Info as every "already
  defined. Or name" message), the first kept — a binding's declaration is still replaced, `\let\X\relax` still makes
  it definable (`\newtheorem{prop}[theorem]{Prop}` then `\newtheorem{prop}{Prop}`: pdflatex "Theorem 1, Prop 2,
  Theorem 3, Prop 4", 77v renumbered; ntheorem's `\renewtheorem` lets `\X`/`\X*` to `\relax` first, ntheorem.sty:808-814);
  amsppt's footnote in a formula reverts as `\footnote`/`\footnotetext` in `tex=`
  (`alias`); endnotes' `\endnote[<n>]` (`\endnotemark`, `\endnotetext` alike) marks the note `\theendnote` with the
  counter at <n> (endnotes.sty:278-283: "iv", not "4", under `\roman`); combine reads `colclass=` before
  `\ProcessOptions` (combine.cls:31-50: `[11pt,colclass=amsart]` passes 11pt to amsart, `[colclass=report,book]`
  loads book). `{multline*}`'s equation now has an xml:id as `{equation*}`'s (56_ams mathtools golden: ids only). A/B vs
  77v-rel (556 papers: r1's 410, rev77v's 120, 26 repeating a `\newtheorem` name): E/F/W 474/5/11114 unchanged, 37
  outputs changed — 36 xml:id-only (the multline* ids), 1604.03189's `\label` in a `{multline*}` on the equation, not
  the subsection (as `{equation*}`; pdflatex "??"). r3 (re-review rev77v2; guards `perfect_kernel_batch77v::*` +5, four RED
  on f30e7bb268, the llncs one pinning OD #783): a binding's theorem guess made while the document is read (OmniBus's
  set at the first undeclared `\begin{proof}`, neurips' and tac's `\AtBeginDocument` sets) declares through
  `\lx@guess@newtheorem` (sect08.rs), which runs the `\newtheorem` in force without claiming the name, so the author's
  own later `\newtheorem{note}{Notation}` replaces the guess instead of being refused ("Note 1.1", "Remark 1.2" under
  r2); tac's `\newtheorem`/`\newtheoremrm` free `\<name>` before declaring it, as tac.cls's `\def`s never refuse
  (0708.1921's tac.cls:409-418); ntheorem's `\renewtheorem` of an undefined name raises ntheorem.sty:808-811's "Theorem
  keyword … undefined"; KPE #1008 records the refusal's scope and amsthm's unemulated error-path text; OD #783 the
  llncs case (the document's `\newtheorem{theorem}` replaces the class's: "Theorem 1.1", pdflatex "Theorem 1." with an
  error). A/B vs 77v2-rel (747 papers: r2's 556, rev77v2's 144, 23 unbound-class and 13 neurips papers with a body
  `\newtheorem`, 22 tac and guess witnesses): E/F/W 524/5/15581 unchanged; 2406.11672 (neurips, body
  `\newtheorem{definition}`) "Definition 3.1" → "Definition 1" as pdflatex; 2602.02822 one formula unparsed, math-parse
  noise (identical on rerun). Residuals: under tac + amsthm the binding's amsthm keeps tac's non-refusing
  `\newtheorem` (amsthm_sty.rs:18 saves the one in force), where amsthm.sty:77 restores a refusing one; theorem.sty's
  binding `\newtheorem` refuses nothing (theorem.sty's `\@xnthm` does); tac numbers every theorem on the one
  `\subsection` sequence (tac.cls's `\@@newthm#1[#2]#3[#4]` drops both counter arguments: "0.1. Theorem", "0.2.
  Proposition"), the tac binding by each declaration's own counter (2407.21129 "Definition 1.1, Proposition 1.1,
  Corollary 1.1"; the titles right since r1); a tac theorem's `\autoref` tag has no type name (" 1"). rev77v3 (SHIP):
  the guess flag is restored on every path, the three switched sites are the only theorem declarations made while
  the document is read; follow-ups folded at integration — the OmniBus guard pins the refusal after a guess (a second
  `\newtheorem{note}{Again}`, pdflatex's one error, "Notation 2"), OD #783 and the sect08.rs comment say the document's
  declaration gives the title (not tac's number), neurips' witness 2406.11672.
  **Integration (int39, 77v r1-r3 + the rev77v3 follow-ups, binary int39a):** reviews rev77v SHIP (r1) → r2; rev77v2
  NO-SHIP (a binding's theorem guess made while the document is read claimed the name, so the author's `\newtheorem`
  was refused) → r3; rev77v3 SHIP. Gates int39a (on dcfec45915): nextest 6493/6493 + cortex 10/10, clippy/rustdoc
  clean, inits 0/0, dump diff vs int38b empty; the follow-up commit (fixture, guard, comments) re-ran `batch77v` 24/24,
  fmt, clippy. Population A/B (1,038: 747 papers of the 77v rounds and reviews — thm-restate, `\todo[`, KOMA, sn-jnl,
  `[section]` and repeated theorems, arxbj/elsart/tac, multline `\tag`, IEEEeqnarraybox, amsppt, endnotes, ntheorem,
  neurips/OmniBus guesses, the witnesses — + 291 random of the proj98 sample) vs int38b: E 6,025 → 5,934, F 17 → 17,
  0 Fatal worse, 17 Error better (15 thm-restate papers: E 112 → 21), 60 outputs changed (thm-restate, multline tags
  and numbers, theorem numbers, ids); 1 Error worse is load timing (2310.15806 times out on both sides at 180 s, the
  candidate 10 lines further on, the same `\omit` error class); no word loss > 1 %. Binding nets (int39a) 136/136, 0 documents worse than int38a.
- **77w** (a shipped file that `\Provides` a bound name loads that binding, then is read as its stand-in's copy; user
  ruling 2026-10-20; OD #731 rules 11-13, KPE #1014; rows above: aistats renamed copies, renamed natbib copies,
  standard-class derivatives, author-block gaps, the routed copy's text overrides, llncs list labels, pazh's title line).
  Three rounds (rev77w NO-SHIP: derivatives lost the binding's frontmatter → r2 raw read over the binding; rev77w2
  NO-SHIP: a derivative's `\newcommand` replaced `\appendix`/`\frontmatter` → r3 text names only; rev77w3 SHIP, its docs
  follow-ups at integration).
  **Integration (int40, 77w r1-r3 + follow-ups, binary int40c):** the first population A/B (int40a) found 2207.07492's
  self-named sn-jnl.cls copy losing its `\usetikzlibrary{snakes}` (0 → 6 Errors) — rule 2's saved-meaning decline caught
  pgf's scratch `\let\pgfutil@reserved@c\pgfutil@reserved@b` — and the nets 2101.08181's lmcs.cls copy spinning pgf's
  loop to the timeout (3 s → Fatal Timeout): a copy's `\let` is now declined only while its source still holds the
  binding's meaning and that meaning reaches the name (`state::meaning_reaches`, `CopyProvidesFrame::assigned`; guards
  `perfect_kernel_batch77w::{copy_loads_a_tikz_library, copy_runs_its_own_loop}`). A classification of the route's six
  outliers found two binding gaps the route exposed, both fixed: an acmproc-era aamas copy (`\alignauthor` columns,
  1401.1086's confSt.cls) is read raw, not as acmart (0 → 6 Errors → 0; guard `acmproc_era_aamas_reads_raw`), and
  spie.cls's author block is the kernel's (2210.16202: 9 creators, 81 contacts → the PDF's 5 authors, each with its
  marked lines; guard `spie_author_block_is_the_kernel_author`). Gates int40c (318722c41b): nextest 6513/6513 + cortex
  10/10, clippy/rustdoc clean, inits 0/0, dump diff vs int40b empty. Population A/B vs int39a (1,244: the 70 rule-13
  papers, 77w r2's 905 — routes, natbib copies, controls —, 291 random of the proj98 sample, the spie witnesses): E 5,598 →
  5,555, F 16 → 16, 0 Fatal worse, 23 Error better; 2 Error worse are the recorded residuals (2209.10785 cidr.cls's
  natbib default, 2605.07897 bytedance_seed's honest `merged_creators`); words +114,373; 6 word losses > 1 % classified vs
  their PDFs — 2210.16202 above, 1501.06758 and 2607.29279 (llncs routes: per-author affiliations, duplicate contacts
  gone), astro-ph0402416 (aa: per-author affiliations, an `<abstract>`), 1602.07024 (aamas2017.cls read raw: numeric
  citations as its PDF, acmart's boilerplate copyright box gone, E 1 → 0), 1402.7103 (aastex route: title, 10 creators,
  numbers gained; the copy's "Fig. 1.—" and "Astrophys. J." text overrides lost — ruling row above). Binding nets (int40c)
  136/136, 0 documents worse than int39a.
- **77y** (77p follow-up, the rev77p8 should-fixes, nits and int34a-same shapes; OD #52(o) 77y bullet; the e-mail,
  symbol-mark and glued-symbol rows above): names under a merged symbol block stay names; a symbol row's unmarked
  continuation lines stay welded to their marked affiliation where one stands between two labelled ones (2602.05059); a
  marked address no name spells goes to its mark's carriers (both branches); link texts holding link words are URL
  contacts, a bare domain's link naming its domain an affiliation, an icon a `\quad` parts from its link that link's
  (2601.17879); an exact address spelling outranks a truncated one, an unparted list spells no one (2605.27805);
  `;`-separated and multi-mailto e-mail lines (2604.10316, 2604.25444). Guards `perfect_kernel_batch77y` (14, red on
  int38), six 77p guards to whole elements. r2 (the 77y review's findings 1-9): a link-word surname keeps a linked name
  a name (`Lawrence E. Page`, `Hans-Peter Link`, `Lucy Page-Smith`, `Mary Ann Code` over its institution); a line under
  a merged symbol block is a names line only with a mark of its own or an institution under it, no note; a symbol row
  welds only institutions and places, nothing past the last labelled one in a row with an unmarked name (R12;
  2604.02904, 2602.03421 the block's again); a homepage's handle link stays a URL contact; a diverging local part scores
  its six letters (`zhangweiwei@` Wei Zhang's). Guards +7 (red on 77y r1), the icon guard whole; residual rows: "Source
  Code" (no corpus witness, probes a5, a6), comma-list symbol marks (q1-q3), a doubled mark in an address (f4), the
  numbered x12 weld of notes. r3 (the rev77y2 re-review's findings 1-8): an unmarked line under the last labelled
  institution of a row with an unmarked name is every name's of the row unless that institution ends in a comma (R12
  literally: c4's University B, w1's last line); a line a symbol mark leads is no names line (m2's legend on Ann Able,
  m4's `$^\S$Google DeepMind` the block's), nor one whose institution under it goes on with its words (k12, k17), and a
  line over an e-mail it spells is (k15); a link word that is also a surname (page, link, code) keeps a linked name a
  name over any line or none, earlier words read whole (l4-l7), and a column of links over an institution is the
  block's (l1-l3); a handle a name of the block spells stays a URL contact (d5, d9). Guards +6 and c4 re-pinned (red on
  77y r2); residual rows: a name over a place under symbol lines (k10), a bare place in a weld (w9), 2606.06924's
  swapped `yehj@`/`huhr@`. r4 (the rev77y3 re-review's findings 1-6, narrowing r3's rules): an institution under a
  line goes on with its words only as one generic institution noun or after a connective (k20, k26, k31, k40-k42 keep
  their authors); a handle is a homepage only when its letters are exactly a spelling of one name (h1-h9 affiliations
  again); the lines under a block's link column go by their kind (b9's address an e-mail); a plural link-word surname
  and a name suffix after it keep a linked name a name (s9, s8). Guards +3 and three names-split cases (red on 77y
  r3); the "Source Code" residual row extended to the three-word texts (s1, s3, s4, s7). rev77y4 SHIP (probe net 173:
  r4 no worse than the better of int38b and r2 on any probe outside the residual list); main-session follow-ups
  (cf9986651a): a generic institution noun is the whole word (`Cat Cole \\ Max-Planck-Institut` keeps its author, c2,
  c5), a labelled address under a link column is the block's e-mail (`Email: contact@x.org`, x2); residual rows for a
  name over a one-word generic noun or a connective-led institution (c1, c3, c4, c8, k27, k30), a per-author `\author`'s
  link column (x7, x8), an organization spelled exactly as an author's handle (h20, h21).
  **Integration (int41, 77y r1-r4 + follow-ups, binary int41a):** gates int41a: nextest 6546/6546 + cortex 10/10,
  clippy/rustdoc clean, inits 0/0, dump diff vs int40c empty. Population A/B (2,220: 1,921 papers of the 77y rounds and
  reviews + 299 random of the proj98 sample) vs int40c: E 6,805 → 6,805, F 74 → 74, 0 Error or Fatal worse; 30 outputs
  changed, all in the 77y set, 0 of the controls; one word loss > 1 %, 2604.02904 (3,116 → 3,036), its affiliation
  lines now on the names their marks pick instead of on all five (the witness). Binding nets (int41a) 136/136, 0
  documents worse than int40c.
- **77t** (quality batch 3: frontmatter content and marks, quality inventory ranks 7, 15, 17, 19, 59, 92, 93, 96, 98,
  110; guards `perfect_kernel_batch77t::*`, 21). (7) the thanks-abuse list's ordinary superscript ("10 m²") — GREEN on
  the base, guarded. (15) the `\dag` mark is shown where nothing answers it (base); what still broke: names marked only by
  text `\dag`/`\ddag` (affiliation marks, so a marked block) or by unanswered numbers over an unmarked institution line
  welded it into the last name — the line is each name's of the names row above it (R12; OD #52(p); a symbol-led line
  is no such line, 2604.02578). (17) several `{IEEEkeywords}` blocks are all kept, a supplement's still sealed in place (KPE
  #1020; 2604.23289). (19) a `\nameref` nested in a title is kept (63k; the ledger misread the row). (59) a vmsta binding
  gives the author content the title page's name parts (OD #789; 71 papers). (92) an entics/entcs binding reads the class
  with elsart's frontmatter interface (OD #790). (96) a parenthesized math lead labels its line (OD #52(p); 1011.0516).
  (98) a class-numbered `\institute` store goes by the authors' numbers (OD #791; 2405.16615, 2512.08621's `\inst`).
  (110) an llncs `\\` line reading as an affiliation is the names' since the last line under names (OD #792). Not done: (93) the fuzzy-name-label
  mark request needs the ruling the 68k row asks for (RED repro class-bindings/gretsi_v13_shared_mark_affiliation).
  A/B vs int41a over 293 papers (witnesses, 30 vmsta, 30 entics/entcs, 30 preprint.cls, 5 two-`{IEEEkeywords}`, 30
  `$^{\dag}$`-author, 30 llncs, 20 imsart, 100 random): E/F/W 99/0/4962 both, 66 changed — 25 vmsta names, 29
  entics/entcs authors (keys out of the names, addresses and notes on them), 11 preprint.cls institutes by number,
  1011.0516 — none worse by the PDF (2411.09950's swapped e-mails are its source's `\thanks` keys; 2204.12376's
  unrequested `\address[a]` is the shared creator's, was body text).
  **r2** (the rev77t review's NO-SHIP, findings 1-8: r1's rules narrowed; guards +10, each RED on r1 bf9f65aac9 but
  the re-pinned symbol-led one and `llncs_author_e_mail_line_ends_a_row`, which pins a regression of a mid-round r2
  version): an unmarked institution line is the names row's directly above it, not every name of its `\and` column
  made before it (r1, r2); a names row under such a line is more authors (r7: two were lost); the line reads as an
  institution by a word naming one — a role or contact word alone ("research", "professor", "collaboration", "member",
  "team", "email"), a corresponding-author note or a math lead keeps the base's weld (r4, r6, r8; so "Google Research"
  alone welds as on the base); a requested `$^{(a)}$` asks for the `$(a)$`-led line (A2); the llncs rule reads the
  whole `\author` across its inline `\and` (llncs.cls:1013-1018), a line going to the names since the last affiliation
  or e-mail line (L1, L2, L4; 2604.11038's three authors); vmsta prints a first name no `\snm` follows. Residual rows
  17-20 of the fidelity-audit table (newer vmsta copies' `\thanksref` keys, llncs `Stanford, CA 94305`, a numbered
  `\institute` before `\author`, an llncs link line read as a name). Doc numbers OD #789-#792, KPE #1020. rev77t2
  NO-SHIP (two r2 changes over-reached, synthetic probes, no corpus witness); main-session follow-ups (int44): a names
  row under a shared institution is more authors only over an institution line of its own (`Hong Kong SAR`, `Equal
  Contribution` stay the institution's next line: s1-s4); in llncs only a line after `\\` can be an affiliation, a
  piece after the inline `\and` is names (`\and the ATLAS Collaboration`: L6); an llncs list with `\inst` marks keeps
  the split (Inst A/B by the marks again; its `University of X` line a creator as on the base: L11); each mark of
  `$^{(a),(b)}$` stripped of its parentheses. Guards `perfect_kernel_batch77t::{
  place_or_note_line_under_a_shared_institution_is_no_names_row, llncs_author_and_piece_is_names,
  llncs_author_inst_marks_keep_the_split}` (red on r2 65fcae409f). Residuals: an llncs `\texttt{ann@x.org, bob@x.org}`
  line under the names makes two creators (L8, as before), an `\inst` list's line under the names a creator (L11, as
  before).
  **Integration (int44, 77t r1-r2 + follow-ups squashed f8e7ca3d6b, entics `\qed` ec39a99ec4; binary int44b):** gates
  int44b: nextest 6604/6604 + cortex 10/10, clippy/rustdoc clean, inits 0/0, dump diff vs int43a empty. Population A/B
  (782: the 77t rounds' and reviews' 486 + 296 random of the proj98 sample) vs int43a (int44a): E 3,109 → 3,109, F 7 →
  7, 0 Error or Fatal worse, no word loss > 1 %; 87 outputs changed (2 of the controls); 317 fewer `<Math>`, of which
  the entics/entcs proofs' `$\Box$` → "∎" (elsart's `\ltx@qed` loaded after the raw class; 8 papers, 2204.12376 31) —
  fixed (ec39a99ec4, the class's `\qed` kept; guard `perfect_kernel_batch77t::entics_qed_is_the_class_box`) and
  re-checked over those 42 papers (int44b: E 12 → 12, the one Math difference 1011.0516's five `$(a)$` marks now its
  affiliations' labels). Binding nets (int44a) 136/136, 0 documents worse than int43a.
- **68i** (binding gaps G3 C, TeX Live package bindings; bindgaps/g23_macros.tsv rows tocbibind, amsopn, datetime,
  rotating, paralist, todonotes, standalone, ccaption, enumitem; each macro checked against the TL 2025 file, or the
  witnesses' shipped classes for enumitem). tocbibind (tocbibind_sty.rs) — `\tocbibname`/`\settocbibname` (:115-132;
  46 papers: 0710.3210, 1106.3946, 1401.4404, 1412.4869, 1505.05782) heading the bibliography where tocbibind sets
  `\bibsection` (:154-198, written as `\<unit>*{\tocbibname}`; the class's name by its chapters, OXIDIZED_DESIGN_DIVERGENCES
  #620), `\settocname`/`\setlofname`/`\setlotname`/`\setindexname` (:124-131; 0710.3210, 2010.07703, 2012.02570),
  `\tocfile{<title>}{<ext>}` (:263-271, a TOC of `<ext>`; 1605.05950, 1712.01817, 2204.09786, 2408.05806, 2409.09967),
  `\tocsection`, `\toc@section`/`\toc@chapter`/`\toc@headstar`, `\toc@start`/`\toc@finish` (:107-114, :250-262,
  :314-316; etoc's tocbibind style, 2412.08175), `\tocotherhead` (:102, formerly empty); `\simplechapter`/`\tocchapter`
  not ported (no witness). amsopn (amsopn_sty.rs) — `\newmcodes@` (:28-42, the pdfTeX branch; 1511.04387, 1712.08220,
  1802.09883, 1809.00159, 1908.01362), `\@declmathop` (:51-53; 2101.09853), `\varliminf@`/`\varlimsup@` (:105-108;
  2005.13511); `\varlim@` was 67's (KNOWN_PERL_ERRORS #696). The row's `\DeclareMathOperator` (21) and `\operatorname`
  (3) are not amsopn's: 1502.03077 (`a msmath` option typo, pdflatex errs too), 2110.09723, 2201.10869, 2208.02596
  (libertinust1math.sty:75 loads amsmath, the libertinust1math binding does not: next payloads' row), 2405.12464 and
  2410.06995 (no amsmath at all, pdflatex errs too); the rest convert clean at 67int9. datetime (latexml_contrib
  datetime_sty.rs; English, monolingual) — the date counts and `\dateformat`/`\THEDAY`/`\THEMONTH`/`\THEYEAR`
  (:564-565, :175-180; 2608.06092), the format selectors verbatim with `\dateseparator`, `\twodigit` and the day of the
  week while `\ifshowdow` (`\showdowtrue`/`\showdowfalse`, `dayofweek`/`nodayofweek`; :75-148; 2209.06700),
  `\ordinaldate` (:64-74, fmtcount's English `\ordinalnum` with `\fmtord`, raised by default, `level`/`raise`;
  1601.00929, 1706.09444, 1804.06840), `\monthname`/`\shortmonthname` via datetime-defaults.sty's English names,
  `\testifleapyear`/`\computedayofyear`/`\computedayofweek`/`\dayofweekname`/`\shortdayofweekname`/
  `\thisdayofweekname` (:613-731; 26 papers: 1308.5288, 1409.5169, 1409.7716, 2112.11386, 2203.14410), `\today` in the
  current format (:733-736) unless ar5iv keeps it `\relax` for an undated source (ruling 64s), `\pdfdate` (:789-829; 17
  papers: 1005.0302, 1203.1927, 1209.4035, 1211.7272, 1212.4375), `iso`'s separators; `\textdate` keeps its
  approximation (fmtcount's spelled numbers), `oclock` and the language modules not ported. rotating (rotating_sty.rs) —
  `\if@rot@twoside`/`\rot@LR` with `figuresleft`/`figuresright` (:31-58; 2306.14600, 2403.02437), `\rotdriver` reading
  its driver and loading none (:64; 42 papers: 0704.0929, 0704.1238, 0704.2885, 0802.2495, 0908.2043;
  OXIDIZED_DESIGN_DIVERGENCES #619). paralist (paralist_sty.rs) — `{inparablank}`/`{asparablank}` under `defblank`
  (:380-399; 15 papers: 1205.4564, 1306.3906, 1801.06340, 1804.00914, 2005.05886; 1707.03997), `\if@empty`,
  `\setdefaultleftmargin` setting the indents, `\defaultleftmargin`, `\defaultitem`, `\defaultenum` (which calls
  `\setdefaultitem`, as the file does) with their deprecation warnings (:75-77, :400-448; 24 papers: 1209.4819,
  1301.6228, 1308.4189, 1403.1528, 1406.4053; 0907.0929, math0106001). todonotes (todonotes_sty.rs) — `\todostyle`
  (:581-583; 27 papers: 2407.14906, 2409.03694, 2410.02385, 2411.05495, 2501.02553), the begin-document `\phantomsection`
  when hyperref has none (:46-49; 12 papers: 1308.6029, 2103.09181, 2208.10521, 2305.03446, 2402.03164),
  `\if@todonotes@disabled` set by `disable` (:159-160; 2606.13443). standalone (standalone_sty.rs) — `\ifstandalone`,
  `\ifstandalonebeamer`, `\onlyifstandalone`, `\IfStandalone` (:26-39; 1310.0809, 2010.10238, 2011.05064, 2208.05046,
  2403.07821), `{standalone}` (:707-709; 2310.18279), `\standaloneconfig` setting `mode`/`subpreambles` (:296-304; 17
  papers: 1905.04614, 1910.10882, 2210.09924; its other keys unexpanded, so 1703.03769's `\buildjobname` is never run),
  and an `\input` sub-file's `\standaloneenv` marking nothing (:802; 2505.21582). ccaption (latexml_contrib
  ccaption_sty.rs) — the options, `\captionwidth`/`\changecaptionwidth`/`\normalcaptionwidth` (:76-82; 24 and 20
  papers: 0803.1137, 0806.4323, 0811.1937, 0905.4349, 1207.5269), `\centerlastline`/`\flushleftright` (:66-72;
  1706.01843), `\precaption`/`\postcaption`/`\midbicaption` (:91-96; 0711.3402, 1001.1088, 1408.3003),
  `\hangcaption`/`\indentcaption`/`\normalcaption` setting their switches, `\bitwonumcaption` (:204-217; 1302.5179),
  `\bionenumcaption`, `\bicaption` in the file's five-argument form (formerly four, keeping one caption) and
  `\bicontcaption` (:219-259), `\contcaption` under the continued float's number (:138-141, formerly the next number),
  and under `[subfigure]` `\subbottom`/`\subtop`, `\contsubbottom`/`\contsubtop`/`\contsubfigure`/`\contsubtable`
  numbering on from the previous float's subfloats and `\subconcluded` (:309-312, :382-430; 0706.2285, 0907.2099,
  0907.2405), the count kept after every subfloat through subfigure_sty.rs's new `\lx@subfloat@done`. enumitem — the
  `{enumerate*}` row (28 papers) is the class bindings loading enumitem without the `inline` their classes give it:
  lmcs (lmcs.cls:37-38; 1611.06929, 1711.09946, 1808.05051, 1904.11378, 2007.10842), Interspeech (Interspeech2024.cls:62,
  Interspeech.cls:69; 2406.16145, 2408.04363, 2505.19937, 2609.16274), INTERSPEECH2021.sty:54 (2104.05980), pnas-new
  (pnas-new.cls:433; 2508.21277) and wlpeerj (wlpeerj.cls:134, which the binding did not load; 2402.14602) now pass it;
  2408.11792's `[in line]` errs in pdflatex too; the row's `\e` (5) is not enumitem's (no definition in the file;
  2110.09723 is libertinust1math's `\DeclareMathOperator`). KNOWN_PERL_ERRORS #800-#807, OXIDIZED_DESIGN_DIVERGENCES
  #619-#620. Repros package-bindings/{tocbibind_names_and_tocfile, amsopn_internals, datetime_names_and_pdfdate,
  rotating_rotdriver, paralist_blank_lists, todonotes_todostyle, standalone_in_main_document,
  ccaption_continued_and_bilingual, enumitem_inline_under_lmcs, enumitem_shortlabels_under_lmcs} RED (67int9) → GREEN;
  guards `perfect_kernel_batch68i` (13). A/B vs 67int9-rel over 502 papers (the rows' 402 witnesses, 100 of pop67o): Errors 705 → 323, papers with
  Errors 380 → 56, Fatals 4 → 4, none gains an Error; per row, Errors: tocbibind 66 → 6, amsopn 46 → 26 (the
  libertinust1math and no-amsmath papers above), datetime 61 → 11 (papers' own: `\THEDAy`, unloaded `\newdateformat`),
  rotating 54 → 10 (unloaded `{sideways}`), paralist 44 → 1, todonotes 40 → 0, standalone 40 → 2, ccaption 57 → 0,
  enumitem 39 → 9; the 100 pop67o byte-identical. Words fall only where `<ERROR>` text left (8 `\newmcodes@` papers:
  1511.04387, 1712.08220, 1802.09883, 1908.01362, 2307.12224, 2309.05133, 2512.10779, 2609.04177) and in 2504.05198,
  whose sub-files' `\onlyifstandalone{\bibliography…}` calls are gobbled as in pdflatex: of the base's four
  bibliographies (152 items) the three sub-file copies of main.bbl go (3 × 38 = 114 duplicate items; the appendix's call
  rendered none), the main one (38 items, before the appendix as in the PDF) stays. Binding nets (tocbibind, amsopn,
  datetime, rotating, paralist, todonotes, standalone, ccaption, enumitem, pnas-new) vs 67int9: no document worse,
  Errors 524 → 471. Rulings (main session, 2026-10-16): `\rotdriver` loading no driver file (OXIDIZED_DESIGN_DIVERGENCES
  #619), tocbibind's name told by the class's chapters (#620), datetime's `\today` in its format with the source date,
  and the class bindings loading enumitem with their classes' `inline` accepted. Round 2: lmcs passes `shortlabels` too
  (the newer lmcs.cls:45-47 in 2003.01491, :51-56 in 2604.16477; the union of the versions), so `[(a)]` numbers "(a)",
  "(b)" (repro package-bindings/enumitem_shortlabels_under_lmcs, guard `enumitem_shortlabels_under_lmcs`); pnas-new's
  and wlpeerj's `inline` is the union with their newer copies (pnas-new v1.44/v1.45 and wlpeerj v1.2 load enumitem
  without it: 1812.09361, 2311.05803, 1710.01085); `\tocsection` numbers `\tocfile`'s heading in a class without
  chapters (pdflatex "1 List of Algorithms"; the bibliography's number is OXIDIZED_DESIGN_DIVERGENCES #620's residual);
  ccaption's load warning names only what stays approximated (`\bottomcaption`, `\legend`, the caption styling);
  subfigure's `\lx@subfloat@done` is empty without ccaption. Open: `\todo[<style>]` takes its style key as the note's
  mark (`mark="mine"`; 19 papers' `\todo[inline]` alike, before 68i too; todonotes_sty.rs:15 `\todo` is `\lx@note{todo}`,
  whose `[<mark>]` takes todonotes' option list; repro package-bindings/todonotes_style_key; FIXED 77v,
  KNOWN_PERL_ERRORS #1007; open: `author=`'s name, printed before the text, is not kept); enumitem's short
  label numbers a reference without the label's punctuation (`\ref` "a" for pdflatex "(a)"; Perl alike,
  KNOWN_PERL_ERRORS #808; repro package-bindings/enumitem_shortlabel_ref, GREEN since 77u, verified on int40c in 78a).
- **68j** (binding gaps G3 D, TeX Live package bindings, the tail; bindgaps/g23_macros.tsv rows newpxmath, listings,
  eulervm, nicematrix, nameref, mathalpha, sidecap, tabls, cryptocode, old-arrows, libertinust1math; each macro checked
  against the TL 2025 file). newpxmath (newpxmath_sty.rs) — newtxmath.sty's Palatino twin, so the binding carries the
  newtxmath binding's set (txfonts' symbols, `txfonts_sty::def_txfonts_symbols`, without loading txfonts — newpxmath.sty
  never does, so `\@ifpackageloaded{txfonts}` stays false and `\varv`/`\varw`/`\vary` undefined, as pdflatex — and
  newtxmath's own layer, now `newtxmath_sty::def_newtx_math_symbols`, which re-declares `\coloneq` ≔, `\eqcolon` ≕ and
  `\varg` an italic g unconditionally, as :1541-1542/:1858 and newtxmath.sty:1973-1974/:2238 do; the newtxmath binding
  stops loading txfonts too, keeping its names, KNOWN_PERL_ERRORS #825):
  the upright Greek `\Gammaup`… and `\upGamma`… (:1662-1743; 1709.08558, 1902.06558, 2001.06387, 2305.12589), `\nequiv`
  (:1374), `\fint` (:967), `\mapsfrom` (:1313), `\uppartial` (:1860), `\transp`, `\Angstrom`, `\circleddot`, `\cdotB`,
  `\vv`, and, new in the shared set (newtxmath.sty too), `\laplac` (:745 / newtxmath.sty:1166, U+29E0; 2603.09013) and
  the italic Greek `\deltait`… (:1817-1856 / :2262-2301; 2508.05134); `\widebar` stays newtxmath's only (newpxmath has
  none); amsmath, etoolbox, xkeyval, centernot loaded as newpxmath.sty:23-33 does. listings (listings_sty.rs) —
  `\lst@InterruptModes`/`\lst@ReenterModes`/`\lst@PrintToken`/`\lst@CalcLostSpaceAndOutput` (no mode stack, nothing
  pending), `\lst@whitespacetrue/false`, `\lst@boxtrue/false`, `\lst@label`, `\lst@aboveskip`, `\lst@frametextsep`,
  `\lst@width`, `\lst@FVConvert`, `\lst@Init`/`\lst@DeInit`, `\lst@AddToHook`/`…Exe`/`…AtTop` creating the hooks and
  listings.sty's own `\lsthk@<name>` hooks (mmacells.sty's inline cells, 9 errors each: 1705.02863, 1712.09863,
  1806.04503, 2012.00279, 2012.07851; 2012.14984, 1904.01009, 2210.07578, 2504.10323, 2505.04744, 2608.14263,
  1704.06338). eulervm (eulervm_sty.rs) — `\mathbold` (:58), a bounded bold upright alphabet (27 papers; 0710.3183,
  1111.7285): the kernel's `\DeclareMathAlphabet` makes a switch that runs past its argument (KNOWN_PERL_ERRORS #823,
  open; repro macro-state/declare_math_alphabet_scopes_its_argument RED). nicematrix (nicematrix_sty.rs) — `\iddots`
  when undefined (:146 `\ProvideDocumentCommand`, U+22F0; 26 papers, 2108.05847, 2210.04594), `\Iddots` now ⋰ (was
  `\ddots`), nicematrix's own (`\lx@nicematrix@Iddots`, :1632) whatever a paper made `\iddots`. nameref (nameref_sty.rs) — `\strip@period` (:70; 17 papers, 1310.1737, 1805.08920), `\label@hook` (:88;
  2511.17271), `\T@nameref`/`\@namerefstar` (:361-372; 2404.09236, 2503.18999, 2511.11546); the row's `\N`/`\Z`/`\C`
  are the papers' own. sidecap (sidecap_sty.rs) — `\sidecaptionsep` (:39-40; 1302.0875), its options (`origragged`,
  :118) and ragged2e at `\begin{document}` (:156-172; `\RaggedRight` 18 papers, 1112.5588, 1812.02282; 2406.14861 and
  2009.09454 never load sidecap; golden graphics/sidecap_vpos.xml gains the ragged2e processing instruction). tabls (tabls_sty.rs) — `\tablinesep`, `\arraylinesep`, `\extrarulesep` (:16-18; 22
  papers, 1011.3468, hep-th0309263). old-arrows (raw) — stmaryrd_sty.rs `\stmry@if` is stmaryrd.sty:40's
  (`\let#2=\@undefined\iftrue#1#2`): the stub gobbled `\def\varmapsfrom` so old-arrows.sty:489-492 ran
  `{\varleftarrow\mapsfromchar}` at its load (19 papers, 2006.11831, 2102.02198; 2406.00395, 2406.02375 unchanged).
  libertinust1math (libertinust1math_sty.rs) — loads xkeyval, etoolbox, amsmath (:7, :75; the 68i residual 2110.09723,
  2201.10869, 2208.02596) and defines its own `\mathbcal`, `\mathbfit`, `\widebar`, `\widecheck`, `\lesssim`,
  `\gtrsim`, `\blacksquare`, `\varkappa`, `\eqdef`, `\cupdot`, `\bigcupdot`, `\oiint` when undefined. Not done:
  cryptocode `\@pc@lb` (cryptocode.sty:950-957 redefines `\halign` to set it and its line-numbering `\\`, and the
  amsmath binding's alignments never reached `\halign` — also every pseudocode line after the first lost its number,
  silently: FIXED 76a, the kernel runs the redefinition where `\lx@begin@alignment` starts, OXIDIZED_DESIGN_DIVERGENCES
  #760; repro kernel-alignment/cryptocode_halign_redefinition_numbers_lines GREEN);
  mathalpha `\mathbcal`/`\mathbscr` (25 papers, discounted: the pre-1.14 mathalfa interface — TL 2025 mathalpha.sty:1107
  defines them only under `oldbold`, and TL 2025 pdflatex errs undefined on 1607.07843, 1612.01166, 1612.06674,
  1703.06879, 1709.03549). KNOWN_PERL_ERRORS #820-#825. Repros package-bindings/{newpxmath_symbol_set,
  listings_mode_and_hook_internals, eulervm_mathbold, nicematrix_iddots, nameref_internals, sidecap_ragged2e_and_sep,
  tabls_line_separations, stmaryrd_if_old_arrows, libertinust1math_amsmath_and_names, newtxmath_coloneq_over_mathtools}
  RED (67int10) → GREEN, {newpxmath_coloneq_over_mathtools, nicematrix_iddots_own_definition} RED (68j round 1) →
  GREEN; guards `perfect_kernel_batch68j` (12). A/B vs 67int10-rel over 359 papers (the rows' 265 witnesses, 100 of pop67o, 6 without a
  main file): Errors 1216 → 755, papers with Errors 291 → 88, Fatals 5 → 5, none gains an Error; per row, Errors:
  newpxmath 75 → 4 (not newpxmath's: `\oren`, `\DeclareQuotePunctuation`, `\lst@InstallKeywords`, `\textgreekfont`),
  listings 191 → 2, eulervm 27 → 0, nicematrix 26 → 0, nameref 110 → 74, sidecap 27 → 4, tabls 28 → 0, old-arrows 38 → 0,
  libertinust1math 23 → 0; of the pop67o papers 92 byte-identical, 2 (1803.02290, 1911.12144, sidecap papers) gaining only the
  `<?latexml package="ragged2e" options="NewParameters"?>` line. Words fall by `<ERROR>` text, leaked label keys
  and `=15pt` text, and merged bold letters (34 papers) — except that round 1's newpxmath binding turned `\coloneq` ≔ into
  ":-" (2103.00564 ×34, 2508.05134, 2603.09013, 2212.01087; the txfonts binding's value), fixed in round 2; warnings rise
  only as math parse warnings over formulas whose ERROR atoms are now symbols (2305.12589, 2507.05434, 2605.03875). The
  base binary crashed once (exit 139 in tikz.code.tex) on 2505.23260, clean on a rerun. Binding nets (newpxmath,
  newtxmath, listings, eulervm, nicematrix, nameref, sidecap, tabls) vs 67int10: no document worse. Round 2 re-A/B over
  60 papers (the newpxmath and nicematrix rows' witnesses, the `\coloneq` papers): Errors 67int10 106 → round 1 9 →
  round 2 9; the 6 papers that change from round 1 change only `\coloneq`/`\eqcolon` (":-" → ≔ in 2103.00564 ×34,
  2108.04582 ×33 — undefined under 67int10 —, 2508.05134, 2603.09013, 2212.01087; 2304.05713, newtxmath after
  mathtools, ×93 and ≕ ×2, pre-existing); the newtxmath papers 2303.02628, 2303.11093, 1812.07002 are byte-identical to
  67int10.
- **68b** (binding gaps G3, core package bindings: bindgaps/g23_macros.tsv rows hyperref, breqn, natbib, url,
  blkarray, pict2e, amsthm; each macro verified against the TL 2025 file). hyperref (hyperref_sty.rs) — `\Hy@Info`,
  `\Hy@Warning`, `\Hy@Error`, `\Hy@WarningNoLine`, `\Hy@InfoNoLine` (:130-135; 1310.6851, 1712.03838, 1901.10472),
  `\Hy@ReturnAfterFi`… (:357-361), `\HyperRaiseLinkHook`/`\HyperRaiseLinkDefault`, `\Hy@SaveSpaceFactor`/
  `\Hy@RestoreSpaceFactor` (:2078-2090; 2403.10429, 2501.18381, 2510.01943, 2602.21138), `\thispdfpagelabel`
  (:6312-6314; 1810.05756, 2002.05778), `\if@hyper@item`/`\if@skiphyperref` (:6655-6658; 1806.02596),
  `\hyper@refstepcounter` (:6727-6733 without `\HyCnt@ProvideTheHCounter`/`\Hy@ifnotmeasuring`, the anchor the
  kernel's; 2102.05485, 2303.09820, 2405.07080, 2505.07323), `\hyper@link` (hpdftex.def:396; 1403.6472, 1502.05411,
  1509.00838, 1612.08986, 1907.10062), `\@extra@b@citeb` (:7443; hep-ph0412159, hep-ph0703293, 2202.05188),
  `\hyperpage`/`\hyperindexformat`/`\hyperxindexformat`/ `\nohyperpage` (:7886-7901, :8019-8068, the page numbers as
  text, OXIDIZED_DESIGN_DIVERGENCES #608; 0712.1142, 0804.2154, 0907.3749, 1011.1558, 1310.3213, 1707.07140,
  2312.17745, quant-ph0409125), `\@refstar`/`\@pagerefstar`/ `\@namerefstar` (:8126-8134; 2204.02601),
  `\psdmapshortnames`/`\psdaliasnames` under `psdextra` (psdextra.def:75; 1801.02767, 1710.02246, 1809.07440,
  1908.10510, 1908.03268). natbib (natbib_sty.rs) — `\NAT@biblabel`, `\NAT@biblabelnum` (:622-623; 1409.8404,
  1508.04752, 1604.03470, 1605.02776, 1607.05855, 1610.04415), `\NAT@bibsetnum`/`\NAT@bibsetup` and `\@bibsetup` by
  the style (:627-645, :658-668; 1102.3365, 1103.0594, 1312.5975, 1804.04445, 1911.00031), `\NAT@force@numbers`
  (:974-981, natbib's error under author-year citations, as pdflatex; 2508.10404), `\@extra@b@citeb`/`\@extra@binfo`
  (:1104-1105), `\NAT@open`/`\NAT@close`/`\NAT@sep`/`\NAT@cmt`/`\NAT@aysep`/ `\NAT@yrsep` from the style's values
  (:239-300; 1906.11206), `\ifNAT@numbers`/`\ifNAT@super` set by the options and `\bibpunct` (:235-245, :320-324
  locally, `\bibpunct` :294-297 `\global`ly). breqn (latexml_contrib breqn_sty.rs; completed, not raw-loaded,
  OXIDIZED_DESIGN_DIVERGENCES #607: flexisym's mathcode macros would strip the formulas' meaning) — `{dgroup}` as a
  subequation group, `{dgroup*}` (:1771-1784; 0903.2087, 1008.1366, 1107.4667, 1111.2451, 1201.6197, 1302.6047,
  1303.0187, 1305.0701, 1306.0788), `{dsuspend}` (:2027-2041; 1606.07330), `{dseries}`/`{dseries*}` (:1724-1748;
  1201.6197, 1607.06055, 1706.02708, 1811.11912), the `breqn` keys with `label` (taking `labelprefix`)/`number` and
  `\breqnsetup` (:57, :383-436; 2008.13194), `{dgroup}`'s options (:1793, a `label` the group's; 2607.02409), the
  display parameters with breqn's types (:179-198, `\eqbreakdepth` a count, `\eqindent` the macro `C`,
  `\eqbinoffset`/`\eqdelimoffset`/`\eqstyle`; 2008.13194, 1805.03313), amsmath and graphicx loaded as breqn.sty:47-48
  does; `\Biggg` (1907.10288) is not breqn's with amsmath loaded (:1523-1528). url (url_sty.rs) — `\Url@FormatString`
  (:127-133, text in the url font, OXIDIZED_DESIGN_DIVERGENCES #608; 1902.08702, 1905.04294, 1905.04768, 1908.01711,
  1908.10377, 1910.08562), `\UrlTildeSpecial` (:63; 0907.0729). blkarray (blkarray_sty.rs) — `\BAmulticolumn`
  (:934-947; 1202.0500, 1204.2176, 1301.3189, 1311.0327, 1401.4921, 1407.4836), `\BAnewcolumntype` (:1677-1683, the
  template's `\NC@rewrite@<c>`; 1302.2340, 1402.5950, 1408.0807, 1702.06723, 2005.02853), `\BAextrarowheight` (:1063;
  1312.6738, 1503.04543, 1503.08375), `\BAnoalign` (:1005-1012; 1912.05110). pict2e (pict2e_sty.rs) — `\arc`/`\arc*`
  (:667-684; 1201.4907, 1503.03277, 1503.03676, 1507.03747, 1611.00631, 2001.00020, 0811.3832), and `\pIIe@arc`'s mode
  now pict2e's (0 a line to the arc's start, 1 a new subpath, :643-646; the filled sector starts at its centre and
  closes there once). amsthm (amsthm_sty.rs) — `\ifmeasuring@`, `\iffirstchoice@` (:294-295, where amsmath has not
  made them; setstack.sty's copy of `\text@`: 0803.1550, 0806.2059, 0812.2552, 1001.4940, 1105.1277).
  KNOWN_PERL_ERRORS #750-#753. Repros package-bindings/{hyperref_internals_called_by_papers,
  breqn_display_environments, breqn_group_label_and_parameters, natbib_label_and_list_internals,
  url_format_string_and_tilde, blkarray_public_commands, pict2e_arc, amsthm_firstchoice_conditional}; guards
  `perfect_kernel_batch68b` (8). Not this payload's (verified): hyperref row's `\chapter`, `\xspace`, `\textcolor`,
  `\ding`, `\citeauthoryear`, `\hypersetup`/`\texorpdfstring`/`\hyperlink`/`\nameref`/`\autoref` (called before or
  without `\usepackage{hyperref}`), `\x`/`\y`/`\z`/`\F`/`\For` (the papers' own), `\PrerenderUnicode`/`\unichar`
  (ucs.sty:591, `utf8x`; 50 papers, the inputenc/ucs binding's), `\CJKkern` (CJK), `\flqq`/`\frqq`/`\glqq`/`\grqq`/
  `\cyr` (hyperref sets them only in PDF strings), `\bbl@info` (babel), `\harvarditem` (harvard), `\tocsection`
  (amsart); natbib row's `\bibsep` (2003.02474, 2008.04185: cas-common.sty raw before natbib is loaded),
  `\citeauthoryear`/`\citep`/`\Citet` without natbib, `\N`/`\NA`/`\Doi`/`\bibAnnoteFile` (the papers' or their
  `.bst`'s). Residuals: `\hyper@link{cite}{cite.<key>}` links to hyperref's anchor name, which our bibliography items
  do not carry (`ltx:ref idref="cite.<key>"` dangles; kept, the dangling-links ruling); blkarray's own `>{…}`/`<{…}`
  preamble tokens are read only where array is loaded; amsthm's `\QED@stack`/`\math@qedhere` (a document's own
  `\qedhere` over amsthm's token stack, 1801.02767, 1710.02246, 1809.07440, 1908.10510, 1908.03268) need the QED stack
  as amsthm's macro, not this binding's native one (the proof lock's area, left); under breqn's amsmath an old-form
  `\cases{…\cr}` (1008.1366) is amsmath's "Old form" error in pdflatex too (amsmath.sty:1077-1117 `\matrix@check`,
  which gobbles `\env@cases`; pdflatex then errs on each `&`/`\cr`), and our `\cases`, as Perl's, then leaves its
  alignment open: 1 → 13 errors there, words 11,629 → 16,621 — faithful, no surpass (main-session ruling; since 77q
  amsmath's error is followed by Plain TeX's `\cases` with its rows, 2 errors, KNOWN_PERL_ERRORS #993);
  `{dgroup*}`'s options are read and dropped (a `label` there names the first equation in pdflatex, `\grp@setnumber`
  :1794 not run; no witness); our amsmath binding never defines `\iffirstchoice@` (amstext.sty:43; no witness since
  77q's setstack stand-in: 0803.1550's copy with amsmath and no amsthm converts with 0 errors, pdflatex 2 "already
  defined").
  Not started (brief rows after the top six):
  amsthm `{proof}`/`\QED@stack`, color/xcolor/graphicx (`\color@block`, `\color@box`, `\definecolor`, `\@testcolor`,
  `\Grot@box@std`, `\Gread@parse@vp`), caption `\DeclareCaptionSubType` and `\captionsetup` keys, the `\Gin@driver`
  root behind trimclip's PGF fallback, cleveref `\crefalias`, the ORCID-URL normalisation of `\lx@add@orcid`.
- **68h** (the 68b tail: caption keys, ORCID URLs, `\crefalias`, `\Gin@driver`). caption (caption_sty.rs) — the
  `\captionsetup` keys and the package options (caption.sty:171-172, the v1 options `bf`, `small`… :114-147) format
  the float captions as caption3.sty:1661-1715 does, through the kernel's new caption hook `\lx@format@caption@@`
  (sect09.rs, longtable_sty.rs): `labelsep` (:774), `labelformat` (:733), `name` (:1347-1350), `figurename`/`tablename`
  (:1336-1345), `textformat` (:791), `font`/`labelfont`/`textfont` and their `+` forms (:842-858), an empty text with
  no separator (:1680-1683); typed `\captionsetup[type]` settings stay the type's (they also set the general keys
  before); `\DeclareCaptionFont` (:830, the predefined fonts :860-950), `\DeclareCaptionTextFormat` (:787) and the
  `endash`/`newline`/`gobble` separators (:776-785) as caption3 defines them; `\DeclareCaptionSubType` (caption3.sty:
  1774-1806; run 336: 39 first errors; 1202.5818, 1604.05412, 2002.03129, 2106.01798, 1706.00437, 2405.16148, 1907.05468,
  2512.12259), moved from subcaption_sty.rs's no-op, its counter made with subcaption's sub-type ids (subcaption and
  subfig register theirs: `\caption@subtypelist`, subfig.sty:261-263 `\sf@counterlist`). OXIDIZED_DESIGN #617. Kernel
  (base_utilities/defs/frontmatter_fields.rs) — `\lx@add@orcid` takes the iD of an orcid.org address (2104.12510; OXIDIZED_DESIGN #616).
  cleveref (cleveref_sty.rs) — raw cleveref's `\crefalias` (cleveref.sty:1277-1278) replaces the no-op; `cref_type`
  reads a type's alias, else its counter's (1810.01223, 1706.03576, 2210.04778, 2506.14950). graphics (graphics_sty.rs)
  — `\Gin@driver` is the driver file the options or graphics.cfg choose (pdftex.def for PDF output): trimclip reads
  tc-pdftex.def and no pgf (2202.10115, 2306.09847, 2507.09618), and adjustbox_sty.rs's `|`-as-other workaround is
  removed. Side fixes the A/B found: bmvc2k_cls.rs defines the class's colours (bmvc2k.cls:268-278, :716;
  2210.10486's `labelfont={color=bmv@captioncolor}`, and its 13 `bmv@sectioncolor` errors), cas_dc_cls.rs loads
  setspace under `[review]` (cas-sc.cls/cas-dc.cls:49, :67; 2512.10464). KNOWN_PERL_ERRORS #795-#797. A/B vs
  67int8-rel over 304 papers (the 39 `\DeclareCaptionSubType` papers, 95 caption-key users, 26 `\crefalias` users, 43
  adjustbox users, 4 ORCID, 100 pop67o): E 2,963 → 2,931, F 5 → 5, error-free 129 → 149; the `\DeclareCaptionSubType`
  papers E 160 → 141, error-free 1 → 20. Binding nets (caption, graphics, graphicx, acmart, cas-sc, ceurart, svmult,
  revtex4/4-1/4-2, bmvc2k) vs 67int7: 0 worse. Repros
  captions-floats/{caption_keys_set_the_label_and_separator, declarecaptionsubtype_numbers_the_subtype},
  sectioning-frontmatter/orcid_given_as_url_is_the_id, package-bindings/{cleveref_crefalias_names_the_type,
  adjustbox_clipping_driver_is_pdftex}; guards `perfect_kernel_batch68h` (8). Faithful error counts that rise:
  `\DeclareCaptionSubType` after subfigure.sty is caption's "counter already defined" error, once per type as in
  pdflatex (1706.00437, 1907.05468: 2 errors, as pdflatex; 1705.07909, 1807.06322). Residuals: sub-captions keep
  their packages' formats (the general keys do not reach them); `\caption*` takes no font and no text format; a sub-type
  `\DeclareCaptionSubType` makes after subcaption has no `{sub<type>}` environment (subcaption.sty:109-111
  `\ForEachCaptionSubType`); `\captionof{subfigure}` in a minipage still reads the caption as the sub-figure's width
  (pre-existing, 2002.04564). Not started: amsthm `\QED@stack`/`\math@qedhere` (the binding's proof begin and end,
  amsthm_sty.rs:236, :297, use its native QED stack: the proof-lock area; 1801.02767, 1710.02246, 1809.07440,
  1908.10510, 1908.03268), color/xcolor/graphicx internals (`\color@block`, `\color@box`, `\@testcolor`,
  `\Grot@box@std`, `\Gread@parse@vp`). Round 2 (review agent_reports/2026-10-16_68h_review.md): the package options of
  caption, subcaption and subfig reach `\captionsetup` as written, and `\ProcessOptions` reads a package's option list
  as latex.ltx:18557's `\protected@edef\@curroptions` (content.rs; it fully expanded it, so `[figurename=\textsc{Fig}]
  {caption}` gave 2 undefined-command errors and a "\edefn sc\selectfontFig 1" label; pdflatex 0, "Fig 1");
  `figurename=`/`tablename=` are added to babel's `\captions<language>` as newfloat.sty:179-188 `\newfloat@setname`
  does (under `[english]{babel}` they were reset to "Figure"); the empty-caption test is caption3.sty:1713-1748
  `\caption@ifempty` (a `\label`, `\index`, `\glossary` or a leading `\ignorespaces` is no text: `\caption{}\label{f}`
  after the hoist, `\caption{\label{f}}`); caption v1's size options replace `font=` (caption.sty:124-129); mrm's
  `\orcid` and apa7's `\addORCIDlink` take an orcid.org address's iD (OXIDIZED_DESIGN #616). More residuals:
  `\captionsetup[longtable]` is not read (longtable's captions take the `table` settings); `\caption{{}}` counts as
  empty (the kernel's label hoist strips the group; pdflatex "Figure 1:"); subcaption's/subfig's separators
  (`labelsep=` under `[sub]`) do not reach the sub-captions. Repros captions-floats/{caption_option_values_unexpanded,
  caption_names_survive_babel, caption_empty_text_ignores_label}, sectioning-frontmatter/orcid_inline_link_takes_the_id;
  guards `perfect_kernel_batch68h` (+5). Round 3 (re-review agent_reports/2026-10-16_68h_r2_review.md): caption's
  v1-keyword match reads `\CurrentOption`'s body (caption_sty.rs `current_option_text`; caption3.sty:374-399 passes
  the list on by `\@expandtwoargs`, a `\protected@edef`), so `\DeclareRobustCommand\rb{\laterdefined}` +
  `[figurename=\rb]{caption}` no longer runs `\rb` at package load (1 Error → 0, pdflatex 0); subfig keeps its expanded
  key read, as subfig.sty:125 and :217 `\edef` the list (pdflatex errors there too). A `\<type>name` of `\protect` and
  a robust or xparse command is a name noun (base_utilities/misc.rs `is_name_noun` follows `\protect\<cs>` hops through
  distinct macros — a hop back to one is a loop, fuse 32 (int10, re-review finding 1: five hops, pdflatex "Ex 1"); it stopped
  at the first, so `\renewcommand\figurename{\protect\rb}` gave the tag "1", pdflatex "Fx 1"; KPE #194). Repros
  captions-floats/{caption_option_key_is_read_unexpanded, protected_robust_name_is_a_name_noun},
  loader/processoptions_reads_as_protected_edef (+ poq68h.sty; the direct `\ProcessOptions` guard: pdflatex's list
  except `[beta=\textbf {x}]` and `[o=\oa ]`, which pdflatex prints with a `\protect`, the kernel's `\textbf` and an
  optional-argument `\newcommand` not being `\protect`-wrapped macros here); guards `perfect_kernel_batch68h` (+3). Integration (int10 nets): with caption's keys in effect, papers' caption setups name their class's colours and fonts, which two class bindings lacked — ieeeaccess `accessblue` (ieeeaccess.cls:121-133, the PANTONE 3015 C spot colour's CMYK alternate 1 0.3 0 0.2, color having no `spotcolor` model) and `\figcapfont` & co. (:75-79; 2405.14505 0 → 9 Errors, 2205.15416 0 → 20), IEEEojcsys `ojcolor` (IEEEojcsys.cls:3346-3348, `\AtBeginDocument`, after loading color) and its caption fonts (:4669-4672; 2206.03535 7 → 14): now defined as the classes do; nets ieeeaccess 0/28 and IEEEojcsys 0/20 worse than 67int9; repros class-bindings/{ieeeaccess,ieeeojcsys}_caption_class_colour, guards `perfect_kernel_batch68h::{ieeeaccess,ieeeojcsys}_caption_class_colour`.
- **subfig passes a key's value unexpanded** (open, latent; 68h r3 re-review finding 4, probe
  scratch_356a0c47/rev68h3/p/s1.tex): subfig.sty:217 and :220 `\edef` the option list before its keys see it, so a
  value macro is expanded at load; since 68h r2 subfig_sty.rs hands the value on as written (OD #617). No visible
  effect today — `labelformat=\lf` with `\lf` redefined after loading prints "(a)" in both, as the sub-caption format
  is fixed — but it is an unruled divergence. Fix: expand the value as subfig's `\edef` does, keeping the key read.
- **A stored `\ProcessOptions` list drops `\protect` before a robust or optional-argument command** (open, low impact;
  68h r3 re-review finding 4): pdflatex's `\protected@edef` (latex.ltx:18557) writes `[beta=\protect \textbf {x}]` and
  `[o=\protect \oa ]`; ours writes `\textbf {x}` and `\oa` — the kernel's `\textbf` and a `\newcommand` with an optional
  argument are not `\protect`-wrapped macros here. Pinned as-is in
  `perfect_kernel_batch68h::processoptions_reads_as_protected_edef` (repro loader/processoptions_reads_as_protected_edef).
- **caption v1 keywords given by a protected or robust macro** (open, small; 68h r3 re-review finding 5, probes
  rev68h3/p/c5.tex, c8.tex): `\protected\def\kw{bf}` with `[\kw]{caption}` makes pdflatex's label bold (keyval's
  `\csname` expands a protected macro), ours stays plain; `\DeclareRobustCommand\kw{bf}` gives pdflatex 2 errors, ours 0.
- **`\ProcessKeyOptions` sees an expanded option list** (open, side finding of the 68h r2 re-review, probe
  scratch_356a0c47/rev68h2/probes/a2.tex): ltkeys' `\ProcessKeyOptions` reads `\@raw@opt@<file>` (latex.ltx:19398,
  :19457-19470), which `\@pass@ptions` stores as the argument tokens (:18521-18525), so pdflatex's
  `\usepackage[name=\textsc{Y}\nm]{myl3}` sets the key to `\textsc {Y}\nm`; ours to `\textsc {Y}Bob`: the loader's
  explicit `[…]` list reaches content.rs `pass_options` already expanded and its raw record is re-tokenized from that
  text (`option_argument_tokens`; `\PassOptionsToPackage` keeps its tokens through `pass_options_with_raw`). Fix: carry
  the `\usepackage`/`\RequirePackage` argument tokens to `pass_options_with_raw`.
- **A local `my<binding>.sty` loads the binding instead** (open, SHARED with Perl, side finding of the 68h r2
  re-review, probe a2.tex): without raw interpretation content.rs `find_file_fallback` (Perl Package.pm:2186-2221
  `FindFile_fallback`, the `rw|my|preprint` prefix) maps `\usepackage{mypgf}` to the pgf binding even though the
  document ships mypgf.sty, and logs "Interpreted as versioned package, falling back to pgf.sty" (content.rs:985-1000,
  the fallback step before the raw one), so the local file's `\showmypgf` is undefined (pdflatex loads mypgf.sty). A
  ruling is needed: keep Perl's binding-wins order (witnesses 1206.0536 mysvjour3 → svjour3, astro-ph0005021), or
  prefer the document's own file over a prefix-stripped fallback (the version-suffix case unchanged).
- **Residual of 68h — trimclip's PGF fall-back under `[dvipdfmx]`/`[dvipdf]`** (open; review
  agent_reports/2026-10-16_68h_review.md finding 8): adjustbox ships tc-pdftex.def, tc-dvips.def, tc-xetex.def and
  tc-luatex.def only, so with `\Gin@driver` = dvipdfmx.def/dvipdf.def trimclip.sty:252-258 inputs tc-pgf.def and pgf,
  and `\clipbox{1pt 2pt 3pt 4pt}{Clipped text}` becomes an `ltx:picture` (an svg:clipPath over the box, the text only
  in an svg:foreignObject) where pdftex/dvips give an `ltx:text`; pdflatex with `[dvipdfmx]` reads tc-pgf.def too
  (the content is the clip). Before 68h every driver mode took this path. Probe
  scratch_356a0c47/rev68h/p3/g_dvipdfmx.tex.
- **67v** (rawclasses lever, user directive 2026-10-15): `ar5iv.sty` passes `rawclasses`, not `localrawclasses`, so a
  class in TeX Live without a binding is interpreted raw instead of falling to OmniBus (OXIDIZED_DESIGN_DIVERGENCES
  #444, batch-67v paragraph); every binding route, exact or alternate, still comes first (`load_class`'s
  `raw_excluded` now holds whenever raw classes are on). Population: 3,362 run-353 papers on a TeX Live class with no
  binding (paper, ptptex, nature, jpsj2, apa6, jacow, smfart, artikel1, …) + 827 `\documentstyle{ptptex}`; lever
  off/on over 3,345 of them: error-free 1,485 → 3,079, errors 10,172 → 4,316, words +1.4%. Data:
  `~/data/pk_agents/main/rawcls3/`; regression study agent_reports/2026-10-15_rc_rawcls_regressions.md (R1 → 67r,
  R2/R5 → 67s, R3 → 67q, R9 → 67t, R4/R6/R8 → 68u; open: R7 caption machinery from class hooks, 7). Guard
  `perfect_kernel_batch67v::a_tex_live_class_without_a_binding_is_interpreted`. Validation plan: sandbox 2605/2606
  reruns vs runs 354/355, then the full arXiv rerun, prev-OK → fail attributed by lever off/on.
- **68u** (the rawclasses lever's last roots; study agent_reports/2026-10-15_rc_rawcls_regressions.md): (R4) the
  undefined-token error stub installs past a lock (state.rs `past_the_lock`, also the undefined-environment stub), so a
  locked binding a `\let` cleared errs once instead of looping to TooManyErrors (OXIDIZED_DESIGN_DIVERGENCES #641,
  KNOWN_PERL_ERRORS #841; 2006.07968 apa6 `\let\cite\parencite` 507+Fatal → 1 as pdflatex, 1509.06375 pittetd; what
  cleared `\mathscr` in 2311.14793, 2501.10505, 2510.26876 is eulerpx.sty:130-132 `\let\mathscr=\undefined` then a
  `\DeclareMathAlphabet\mathscr` the kernel lock refuses: 511+Fatal → 1, pdflatex 0 — residual below; 2203.12624 the
  same, then a separate `multlined`-in-`equation` root, 101 + Fatal, pdflatex clean). (R6) `\|` is a protected macro
  over the atom `\lx@math@Vert`, as fontmath.ltx:467 / latex.ltx:13641-13645 make it, so beilstein.cls:264's
  `\expandafter\DeclareRobustCommand\expandafter\|\expandafter{\|}` no longer recurses (#642, KPE #842; 2604.06890
  51 Errors under OmniBus, Fatal raw → 0). (R8) the kernel leaves `secnumdepth` 0 as latex.ltx:17332 and Perl
  latex_base.pool.ltxml:312 do (sect04.rs; its 3 was Rust-only), so letter.cls sections are unnumbered (the 24
  b_regressions letter papers 0 errors, 0907.4214; the 156 class bindings' numbering unchanged but slides, brief,
  g-brief, g-brief2, which define no `\section`). (R4 side) a class setter replay runs no frontmatter fallback
  (sect05.rs `replay_class_setter`): paper.cls:624's `\abstract`, replayed for jheppub's `\abstract{…}`, ran a
  `\subsubsection*` whose `\@startsection@hook` emitted the title and authors in the preamble (2012.12760,
  2207.06852). Guards `perfect_kernel_batch68u`; repros macro-state/locked_name_let_to_undefined_stub_loop,
  expansion-primitives/robust_double_bar_over_its_expansion_beilstein, sectioning-frontmatter/{letter_section_is_unnumbered,
  class_setter_replay_runs_no_frontmatter_fallback_jheppub}. A/B over all 3,349 list3362 papers (67int15 vs 68u
  without the replay fix): error-free 1,491 → 3,157, errors 9,146 → 1,282, Fatals 12 → 8, words +1.4 %, contacts
  2,420 → 7,973, keywords 425 → 677, classification 392 → 2 (into `<keywords>`), creators 12,394 → 11,052 (A
  over-split), proofs 7,838 → 7,701, captions 26,877 → 27,087. Round 2 (OXIDIZED_DESIGN_DIVERGENCES #643, #644):
  K11 setters by name — rule (a), a table setter's store of another name is the frontmatter's when the reader scan
  (`frontmatter_stores.rs` `ClassReaders`: every raw-file macro at the class's end; title-page code = the refused
  `\maketitle`, `\@maketitle` and what only they call) finds only the title page reading it (apa6.cls:760
  `\affiliation` → `\@affil`, :779 `\authornote` → `\@acks`, `authornote` added to `STORE_SETTERS`; uiucthesis.cls:97
  `\department`); a setter may make side definitions beside its store; a raw file's `\date` storing `\@date`
  (tufte-common.def:608-614) keeps its definitions and sets the date; a `{keywords}` collecting its body for the title
  page (cclayout.sty:71-73); beilstein.cls:586-737 and lni.cls:384-464 author blocks as bindings (latexml_contrib
  `beilstein_cls.rs`, `lni_cls.rs`; authblk's `define_author_and_affil`). Witnesses 1710.08171, 1705.02935,
  2003.11668, 2603.01440, 1905.07821, 2203.09951, 2604.06890, 2608.21389; the round-1 RED repro
  maketitle_replay_gate_ifundefined_branch_apa6 GREEN through rule (a) (the replay gate unchanged). Sample A/B (249
  papers: round 1's 161 + apa/apa6/beilstein/lni/tufte; 67int15 / r1 / r2): error-free 91 / 227 / 227, errors 2,702 /
  162 / 162, words 4,177,195 / 4,291,150 / 4,293,590, contacts 224 / 327 / 483, dates 36 / 28 / 45, keywords 15 / 43 /
  44; no paper worse than round 1. Round 3 (the review's should-fixes): apa6/apa numbered author and affiliation
  forms (`\twoauthors` … `\sixauthors`, `\twoaffiliations` … `\sixaffiliations`, apa6.cls:745-777, apa.cls:427-459:
  71 of the population's papers — 64 apa6, 7 apa — had no creator, as under OmniBus, which kept the names only as
  text beside an ERROR) are bindings over the raw class (`apa6_cls.rs`, `apa_cls.rs`; OD #644): at `\maketitle` slot n's
  names carry affiliation n, as every layout of the class's title page sets them (`\put@two@authaffil`, :1344-1365);
  each binding over a raw class (apa6, apa, beilstein, lni) fires K11 for it; beilstein.cls:627
  `\author[marks][e-mail]{name}`. A locked name's error stub is local (state.rs `stub_scope`, OD #641): a group's `\let`
  clearing a binding no longer outlives the group (`{\let\cite\undefined…}`, `\begingroup\let\section\nonexist…`).
  A class declaring another format (`\NeedsTeXFormat{pLaTeX2e}`: TeX Live's 19 pLaTeX/upLaTeX classes — jarticle,
  jbook, jreport, tarticle, tbook, treport, jltxdoc, plnews, jsarticle, jsbook, jsreport, jspf, kiyou, ujarticle,
  ujbook, ujreport, utarticle, utbook, utreport; ctexart declares LaTeX2e and stays raw) is not read raw (content.rs
  `declared_other_format`, OD #444; latex.ltx:18663-18674 reads the declaration and stops): the declaration is read
  (K12's pTeX profile for pLaTeX2e), the class falls to OmniBus — 1203.0762 5 E + Fatal → 0 E, as before the lever;
  a minimal jsarticle document 17 E → 0. `ClassReaders::survey` builds a caller index (no quadratic scan). A/B 266 papers (every apa/apa6
  paper of list3362, the 71 numbered-form users, 1203.0762, the review's 80; 67int15 / r2 / r3): error-free 94 / 234 /
  235, errors 1,142 / 498 / 493, Fatal papers 3 / 2 / 1, words 4,149,827 / 4,475,515 / 4,478,132, creators 524 / 504 /
  717, contacts 286 / 659 / 873; no paper worse than r2. Rows (round 3): cjs-rcs-article.cls:381-386
  `\NewEnviron{englishabstract}{\global\let\CJS@englishabstract\BODY}` (also `{frenchabstract}`, `{keywords}`;
  2608.13762: the abstracts and keywords reach no output; not the `\Collect@Body\X` shape
  `collects_body_for_the_title_page` reads, and not a table environment); smfart.cls:568-583 `{altabstract}` boxed for
  the title page (2506.20120); colorart's `\address` through projlib-author.sty:98-125, an expl3
  `cmd/@author/after` hook (2609.02711, the address lost). Still open: apa's
  `\acknowledgements`/`\note` (not table names); thcc.sty:81-91 `\th@ccproof` (theorem.sty's binding does not run
  a raw style's `\@begintheorem`: cc's proofs are numbered theorems "Proof 1", pdflatex "Proof.", 14 papers; repro
  class-bindings/cc_proof_style_unnumbered_thcc, RED); cclayout.sty:76-81 `{subject}` (the classification, lost; not a
  table name); smfart/nrc/letter/saj.sty addresses stay class-typeset body text (semantic only); a
  plain-locked name a file `\let` clears and then redefines (eulerpx.sty:130-132 `\mathscr`, 2311.14793, 2501.10505,
  2510.26876: 1 Error each where pdflatex has 0; restoring the binding when the file's redefinition is refused would
  extend OD #569 — later);
  ctexart (43 papers) words 592,004 → 592,543, error-free 24 → 13 (fandol, as pdflatex).
- **67b** (frequency-first): a kluwer.cls binding (`latexml_contrib/src/kluwer_cls.rs`; the class is in TeX Live, so
  these papers fell back to OmniBus): `{article}`, `{opening}` (its end makes the title, kluwer.cls:707-738), one
  author per `\author` with an `\institute` going to the authors since the previous one (:766-769), `\surname`,
  `\newdisplay`, `\newproof`/`{pf}`, the frontmatter setters. Run 349: `{article}` 384, `{opening}` 344, `\surname`
  335, `\newdisplay` 128 papers; 40-paper A/B E 169 → 58, error-free 0 → 24, none worse
  (`perfect_kernel_batch67b`). Open: `\author{A (1) and B (2)}` with parenthesised marks stays one creator (the
  merged_creators cluster).
- **67a** (frequency-first; ruled 2026-10-14 "provide-only fallback"): under the ar5iv preload a paper that uses
  `\url`/`\href` with neither url.sty nor hyperref loaded gets hyperref's definitions at `\begin{document}`, as arXiv's
  build (which loads hyperref into every paper) gives them — only the ones still undefined (OD #588;
  `perfect_kernel_batch67a`). Cluster: 812 `\url` + 473 `\href` papers live in run 349.
- **66z** (frequency-first, ruled 2026-10-14: the image-conversion Error cluster, ~1,240 papers live in run 349): a
  MetaPost figure (`prologues:=0`, fonts left to dvips in `%*Font:` comments) is converted from a copy holding the fonts
  dvips would download — Type1 programs via `pdftex.map` + kpathsea, TeX names bound, MetaPost's `fshow` (OD #587;
  `latexml_post/src/postscript_fonts.rs`). 40/40 sampled failing figures were MetaPost (≈2/3 of the cluster's papers);
  12-witness HTML A/B: 10 papers E → 0 (math0102124 38 figures, cs0005010 34), none worse. Rest of the cluster:
  dvips-prolog fragments (`startTexFig`, `@endspecial`, `xxxprolog.ps` blocked by `-dSAFER`), re-encoded fonts (`ptmr8r`),
  `.eps` files that are TeX sources.
- **66y** (K1 roots, ruled 2026-10-14 "park K1, fix the roots"): an `array` outside math is an inline formula of its
  own, its cells math as latex.ltx `\@arrayclassz` sets them (1611.02392 E 24 → 0; KPE #677), and `\raisebox` in a
  picture reads its box in text mode (0902.1321 E 101 + F → 0; KPE #678); `perfect_kernel_batch66y`. The other K1
  roots (`\cite*` under the locked `\cite`, qtree, gtart) are one-paper causes, parked under the frequency-first rule.
- **66x** (K9 study row L, ruled 2026-10-14 "fix the identified mode bugs, Perl wording"): the `\lx@list` frame of
  `\list`/`\@trivlist` is marked a bare environment's, so a `}`/`\endgroup` meeting it ends the list (an Info) and then
  its own group — iopconf `\references`, jT `resume`, INFORMS `\Trivlist` no longer swallow the rest of the document into
  the last item (0902.3464: 28 sections inside list items → 0, 1611.07992 9 → 0, 2106.10067 19 → 11; 5 witnesses E → 0; KPE
  #676, OD #586; `perfect_kernel_batch66x`). Row B (a preamble `\begingroup` around
  `\begin{document}`, ~38 papers, 1 Error, content kept) stays open: faithful unwinding pops the document frame and
  the author's group beneath it, then re-establishes the document frame (latex.ltx:9490 `\document`'s `\endgroup`).
- **66w** (K1 cluster of run 345, ruled 2026-10-14: narrow OD #553): a `\par` read in an inline formula looks ahead,
  unexpanded, to the next `$` (stomach.rs `formula_continues_past_par`, `gullet::peek_upcoming_tokens` over a scratch
  mouth): a math-only token before it (a script mark, a DefMath symbol, a requireMath definition, tex.web §1046's
  math-only primitives) keeps the formula open across the blank line, the `\par` reverting as nothing (Perl's content);
  otherwise §1047 ends it as 65n did. hep-th9402135 101 errors + Fatal TooManyErrors → 22 errors, 0 Fatal (eleven split
  formulas whole; left: `$\bull$` with a blank line in `\bull`'s body, as pdflatex, and `\References` non-long `\par`
  errors); hep-ph9809328, hep-ph9802283, hep-ph9706250, hep-ph9706246, 0903.4382 fewer errors or set-right formulas,
  still capped elsewhere; the 65n witnesses (2007.11971, 1710.02165, 2008.11762, 1210.0368, 1907.09124, math9805003,
  hep-lat9811036, nucl-th0401019) and the other K1 papers byte-identical (39-paper A/B vs 66t2). KPE #679; guards
  `perfect_kernel_batch66w::*`. Round 1 (review blocker): any doubt ends the formula — a command's `*`, `[…]` and
  `{…}` arguments are not looked into (`\cite[p.~2]{smith_2001}`, `\ref*{eq_1}`, `\citep[e.g.][]{a_b}`), a command
  it cannot follow (verbatim/semi-verbatim/delimited parameter, a code-bodied macro such as `\verb`, one reached
  through a macro's body such as `\lstinline`) or a stop by meaning (`\medskip`'s `\vskip`, `&`, `\cr`) gives up
  (`stomach.rs lookahead_command`); the look-ahead is lazy and bounded (2,000 tokens, 16,000 characters), and starts
  after the judged `\par` (an `\obeylines` line end continues as a blank line does: hep-ph9809328, hep-ph9706250).
  Not done:
  §1046's re-opening itself (66i draft, parked).
- **66v** (66m E1, ruled 2026-10-14): the floatrow binding records float.sty/rotfloat.sty loaded, as floatrow.sty:34-37
  marks them (`\ver@float.sty`), so a later `\usepackage{float}` no longer replaces floatrow's `\float@end` (2010.00977,
  2003.11778, 2011.10793, 2105.04869; KPE #668; `perfect_kernel_batch66v::floatrow_marks_float_loaded`). No general
  loader `\ver@` rule (it kept out memoir's and floatrow's semantic-layer bindings).
- **66t** (K9 study row H, 54 papers losing their whole body): a bare `\endX` ends the mode only when the top frame
  is an environment's (any: amsmath ends `\equation*` with `\endequation`); a box's frame gets one Warning and no frame work, as
  LaTeX's list-ending `\endquotation` does none — an IEEE Open Journal `\IEEEkeywords` `\vbox\bgroup` box ended by
  `\par\endquotation\egroup` no longer loses its frame and the body (2301.00752 words 0→16,663, 2602.19492 0→6,164;
  KPE #674, OD #584; `perfect_kernel_batch66t`). K9 Stage 1's general frame-ownership rule and Stage 2 stay open.
- **66s** (K9 study row G, 74 papers, all 2511+): a Semiverbatim identifier field's TeX escapes and grouping braces
  are not part of the identifier (bibtex.rs `identifier_data`): MathSciNet's DOI `…3\{_}10` is balanced for BibTeX
  (it counts the escaped brace) but the Semiverbatim reader took `\{` for a control symbol and met a stray `}`;
  `\{_}`/`{\_}`/`\_` now read `_`, the DOI link matches the entry's `url`; values holding a control word (`\href`)
  pass as before (2601.01900 E 2→0, 2511.01040 E 6→0, 2601.01772 E 2→0;
  `06_cluster_bibliography::bib_doi_with_escaped_brace`).
- **66r** (two quick wins from the K9 mode-switch study, `agent_reports/2026-10-14_k9_mode_switch_study.md`): the
  contrib emlines binding's `\emline` lost its two stray `}` (Rust-only typo; ~40 papers, 0911.1421 had 78 Errors), and
  `\twocolumn[…]` tests its header in Rust instead of inside an `\ifx`, so a header spanning the document's own
  conditional (achemso, price.sty; ~46 papers) keeps it balanced (KPE #672; `perfect_kernel_batch66r`). The study's
  K9 Stage 1 (H, 54 papers losing their whole body) / Stage 2 (L 126, B 38) design and six ruling questions are open.
- **66q** (K1 report row W2): natbib's internals `\NAT@citetp`/`\NAT@citexnum` defined over the binding's citation
  commands, selected by `\NAT@ctype`/`\ifNAT@swa`/`\ifNAT@par`/`\ifNAT@full` (natbib.sty:685-749), so a document's own
  citation commands built on them cite (2104.01159 E 35→0 with its 62 bibitems back, 2105.06268 E 10→0; KPE #671;
  `perfect_kernel_batch66q::natbib_internals_serve_user_cite_commands`).
- **66o** (`\endgroup` cluster E3): a natbib `\bibitem` label piece's math shifts are balanced within its brace
  groups (1208.0605's `{Mantz}, $\, {Ebeling}, \& {Rapetti}`; revtex's `\citenamefont {Sasaki$}`, 1205.3745,
  1508.02141) — natbib.sty:818-823 only writes the label to the .aux, but the binding typesets it as tags, and the open
  math ran past the `\bibitem` into the bibliography; 6/6 witnesses lose that error (KPE #670;
  `perfect_kernel_batch66o::natbib_unbalanced_dollar_in_label`).
- **66n** (root-causer on 2308.07459's raw ytableau): an `\hrule` is an alignment's border only between its rows;
  in an open cell it is the cell's (its box's) material — Perl adds a border row and `add_line` restarts the row's
  columns, so every ytableau row with more than two cells lost its middle cells, and after a `\none` cell the next cell
  left math (KPE #669; `perfect_kernel_batch66n::rule_in_a_cell_box_keeps_the_row`).
- **66m** (the low-risk mechanisms of the arXiv `\endgroup` cluster, report
  `agent_reports/2026-10-14_endgroup_cluster.md`): the algorithm2e binding no longer loads float, which algorithm2e.sty
  does not, so floatrow's `\float@end` stays (1811.04370, 2007.08784, 1609.05867, 1910.09976, 1807.05059; OD #580);
  `\lstinline` robust, so ifthen's `\equal` keeps it unexpanded (1406.5453, 1605.05104; OD #581); IEEEtran's
  convenience `\keywords` a `Fallback` the document's `\newcommand` replaces (2006.10005, 2008.05854, 2011.04315; OD
  #582, KPE #664); `{plate}` undefined for AASTeX 6+ and emulateapj, whose classes have none (1909.00812, 2006.16258,
  2305.06275; KPE #666); emulateapj's `\enddeluxetable` closes a still-open data (1506.07185, 1705.08932; KPE #665); a
  brace-less `\abstract`'s capture frame is a bare environment's, so an enclosing `\endgroup` ends it (2305.17823; KPE
  #667). Guards `perfect_kernel_batch66m`. The loader's `\ver@` half was ruled 2026-10-14 (floatrow's marks only, no
  general loader rule) and landed as 66v. Open from the same report: E2, E7, E10,
  E11 (E3: 66o).
- **66l** (the rest of K1's wrong-mode roots from the same report): array.sty `w`/`W` columns keep the alignment's
  mode, as array.sty:463-474 (2407.14297; KPE #660); `\lx@marginpar` digests in `inline_internal_vertical`, so a
  `$…$` note called from a formula is a formula (2002.06461; KPE #661); forest's `content format` (`plain content`,
  `math content`: forest.sty:3944-3946) is applied to a node's content (2305.14297); what OmniBus requires as a guess
  (inst_support, aas_macros) loads `Fallback`, so a document's `\newcommand{\inst}` wins (2603.21740, cas-sc; OD #408);
  an eqnarray.sty binding maps `equationarray[pos]{cols}` onto the eqnarray alignment over the specification's columns,
  cells `$\displaystyle…$` (1605.07858, 2507.02169); a `\multicolumn` cell of an amsmath math alignment (`aligned`,
  `gathered`, `split`, `multline`: `template_math`) is text, as `\omit` drops the template's `$` (1808.07192; KPE #662). Open, shared with Perl (KPE #663): a mid-paragraph `\footnote` or (since 66l) `\marginpar` in a measured
  `\vbox` counts as two lines — the parameter digester's mode repair repacks the line around the note
  (`notes_measure_as_their_mark` case 3 pinned at 18.83pt).
  Guards `perfect_kernel_batch66l::*`. Open: `\opt@` string re-tokenized (2407.02071), ytableau after `\omit`
  (2308.07459), two xy-pic gaps (1012.0388, 1103.1294); eqnarray.sty `\eqnnum`/`\yesnumber`, fleqn and `[pos]` not
  modelled.
- **66k** (K1's word-loss roots, root-causer report `agent_reports/2026-10-14_k1_wordloss_triggers.md`: in 19 of the
  20 papers our engine read the `_`/`^` in text where pdflatex reads it in math): `\cancel`/`\bcancel`/`\xcancel` are
  robust, as cancel.sty:70-78, so an aligned cell's start (§791) does not expand them before the template's `$`
  (1801.01765, 2404.04195, 2505.00286, 2505.10361, 2512.10194); pstricks keeps graphics' boxing `\scalebox` (2104.00361,
  2504.03865; KPE #658; 1208.6481 still 0 errors); graphicx's `alt`/`actualtext` are read undigested, as graphicx.sty
  never typesets them (2605.23217, KPE #659). Guards `perfect_kernel_batch66k::*`. The report's other roots: 66l.
- **66j** (2605/2606 sandbox regressions, root-causer reports `agent_reports/2026-10-13_sbx2605_regressions.md`,
  `…_sbx2606_regressions.md`): an option no `\DeclareOption` declared runs its `\ds@<option>` when defined, latex.ltx's
  `\@process@pti@ns` (C12; 2.09-style classes: gji.cls 2605.19951, biom.cls `usenatbib` 2606.12015; KPE #657); revtex's
  `\@AF@join` for a class's own `\@email` (C11, JASA.cls: 2605.18495, 2605.23416; KPE #655); `\mathcolor` adds no group
  and restores the color it found, as mathcolor.ltx:20-29, so a `\left` and `\right` in two calls pair (C7, 2605.11792);
  the loader's expl3 scope-exit cleanup fires on expl3's own `\l__kernel_expl_bool`, not on `_` being a letter, so a
  package that restores its own catcodes keeps them (N6, etextools: `/` stayed 8; 2606.06323). Review r1: the `\ds@`
  lookup is the kernel `\ProcessOptions`'s only (not the key=value processors'); option handlers a binding defines but
  never processes are retired as processed ones are — REVTeX 3's under a revtex4 class (`retire_option_handlers`; a later
  package's `[draft]` ran revtex's no-op), hyperref's and minted's key names — and preview/floatflt/floatfig process
  their options as their packages do. Guards `perfect_kernel_batch66j::*`. RED: `loader/etextools_translations_begin_document` (2606.06323's second error,
  etextools + translations at `\begin{document}`); C3 (nomencl in a float) still open.
- **66g** (payload, 2605/2606 cluster C5): an input's end closes only a `\left` group opened in that input — `\left`
  records the input it was read in (`gullet::reading_context`), so physics' `\left( \lx@xmarg{…}{#1} \right)` keeps its
  pair when `#1` (`\cosh(a\over 2)`, `\expval{a \over b}` in `aligned`) ends (OD #530; 2605.02768, 2606.01486 E 1 → 0;
  65d's witnesses unchanged). Guards `perfect_kernel_batch66g::*`.
- **66h** (run 345's Error → Fatal regressions, 118 confirmed against 64zf, root-causer report
  `agent_reports/2026-10-14_ef345_regressions.md`, per-paper `~/data/pk_agents/main/ef344/attribution.tsv`; and three
  2605/2606 sandbox regressions): the caption wrapper's branch is a control sequence, not a braced argument, so a
  one-token box argument's `}` no longer splits it (K3, 16 of 17: astro-ph0107461 and more back to their pre-66d Errors,
  OD #570); `\right.` is inserted once per `\left` group (OD #530) and svjour3's `\institute` drops its `\at` sentinel (K5a,
  9: 1404.4148, 1705.07573, 1911.12144, 1912.03170, 2107.00750, 2107.11151, 2205.01215, 2210.16010, 2305.18835; KPE
  #652); the AMS-TeX front matter is amsppt's (K2b, 7 AGT papers: math0103177; KPE #650); amstex.tex's `\RIfM@`/`\RIfMIfI@`
  (K2c, 6 of 7 Michor diag.tex papers: math9611223; KPE #651); revtex `\@email`/`\@homepage` (K4a, 7: 0710.1022,
  1201.4230; KPE #655); amsrefs `\strip@MRprefix`/`\MRhref`, `msc-links` loading hyperref and `{bibchapter}` (K4b/K4c, 6:
  0806.1269, 1612.01009, 1706.05540; KPE #656); lipics loads thm-restate only on its option, read from the class's own
  option list too (K4d, 2604.22493; 2211.04601 still restatable), and follows the class's `autoref` (aliascnt sibling
  counters unless `thm-restate`, :1119-1236) and `numberwithinsect` options (2604.22493's keytheorems
  `sibling=corollary`: 6 Errors → 0; over the 36 lipics papers in the A/B population only it changed counts, words
  identical; `\autoref` names the sibling, "Proposition 2" for "Theorem 2" in 2103.15659; tags follow
  `numberwithinsect`, "Problem 2.1" for "Problem 1" in 2406.18299). From the sandboxes: the article, report and book
  bindings define `\fnum@figure`/`\fnum@table` as the composed tag (2605.29738 Fatal → 0, OD #579, KPE #654); revtex
  `\frontmatter@footnote` (2605.04155, KPE #655); float.sty's `\newfloat` a macro scrhack can patch
  (2606.04906, 2606.11358; KPE #653). Guards `perfect_kernel_batch66h::*`; A/B over pop.tsv (1,002 papers)
  byte-identical in every fingerprint, and over the
  E → F papers with a main file (75 of 121): papers with a Fatal 72 → 34, words 660k → 1.12M. Open from the report: K6
  (2601.10843, 2602.04048): `\start@align` re-enters a re-dressed `{alignat}`; starting the binding's own alignment
  instead (66h r1) only turned the Fatal into an Error over the same truncated document (review 66h: an empty
  `<document/>` on the repro), so it was reverted — `\start@align` must open the alignment in a group whose `\egroup` ends
  it, with `\black@`/`\totwidth@` (amsmath.sty:1364/:1316) defined (RED `alignment/start_align_under_redressed_alignat`);
  K1 (31, 65n's
  §1047 `\par`-closes-math without §1046's math-token-reopens-math: `$` parity flips; repro
  `paragraph-mode/script_in_text_after_par_closed_formula_reenters_math` GREEN by 66w, OD #553 narrowed: the `\par`
  keeps the formula open when math follows it. Batch 66i, §1046 for sub/superscript marks,
  is parked: over 1,235 A/B papers errors 9,542 → 8,361 and Fatals 40 → 32, but 20 papers lost >1 % of their words
  where an earlier wrong-mode bug puts a script in text — 2104.00361 `\scalebox{1}{$b_1$}` in `\substack` (the inner `$`
  closes the outer math), several at binding-built strings — so those go first; root-causer report
  `agent_reports/2026-10-14_k1_wordloss_triggers.md`),
  K2a (19, missing macro files: shared), K2d (4, Paul Taylor's diagrams.tex not in TL, now converting 2k-11k words; a binding since 67d), K5b
  (2, L3 papers' unshipped physics.sty), 5 singletons; 1309.7562's `$$` meeting an open `{` (13 Errors on every binary,
  pdflatex 1); `\bibselect` does not read a `filecontents` `.ltb` (`Info:missing_file`).
- **63b** (landed): nameref's begin-document reset of `\ref`/`\pageref`/`\Ref` under its own hook label (KPE #519;
  2503.08060, 1908.01329, 1811.01873, which also needed nameref's `\NR@setref`), a nested alignment's rows run no
  enclosing row hook (KPE #518; arrays, `\halign`, TikZ matrices, tabbing), the 63a tag `~` binding narrowed to the
  tags a reference prints,
  and the stomach loop detector tells register values apart by amount (Rust-only: `digested.rs` `fingerprint_into`
  hashed a `RegisterValue` by kind, so PiCTeX's dots, each `\raise`d differently, read as a loop; 0801.0709). Guards
  `perfect_kernel_batch63::{preamble_ref_redefinition_reset_at_begin_document,
  begin_document_ref_redefinition_survives_nameref, nested_alignment_inherits_no_eqnarray_row_hook,
  pictex_finite_dots_are_not_a_loop}` and `digested::tests::cycle_fingerprint_distinguishes_register_amounts`;
  trade-off in OXIDIZED_DESIGN_DIVERGENCES #326.
- **63l**: author-line residuals of fidelity audit round 2, all Perl-origin: "~and" and `{\ and}` separate authors
  (KPE #532; nlin0101056, 1508.01140, both a merged_creators error before); `\fnmsep` leaves no comma in a name
  (KPE #533; astro-ph0001054, astro-ph0611016); `\IEEEmembership` is the author's membership contact (KPE #534;
  2306.15457, 2508.00603, 2408.00327; name lines only, not in conference mode unless `\IEEEoverridecommandlockouts`, a document's own definition kept). A/B
  540 papers vs 63k (80 `\IEEEmembership` papers of 2408 among them): 65 change, 0 more errors or warnings (57 gain
  grades only, none on another author). Open from the round: entics.cls `\thanksref{ALL}` prints its key (2212.08177, a raw-class
  two-pass mark), an.cls `\abstract{…}` a plain paragraph (astro-ph0611016), llncs `\author{A\\Univ\\\email{x}}`
  reads Univ as a person. Guards `perfect_kernel_batch63::{author_and_tied_or_grouped_splits,
  author_fnmsep_leaves_no_comma, ieee_membership_is_a_contact, ieee_membership_in_a_block_list,
  ieee_membership_outside_the_names_stays_dropped, ieee_membership_own_definition_kept,
  ieee_membership_conference_override}`.
- **63k**: titles' refs and JHEP-family authors (fidelity audit round 2):
  - RUST-ONLY: a `\ref`/`\eqref`/`\autoref`/`\cref`/`\nameref` in a section title was empty wherever the title is
    reused (TOC entries, `title=` tooltips): Perl's `CrossRef::fillInTitle` (CrossRef.pm:882-904) was never ported.
    `crossref.rs` now fills a copy of the stored title before each use (`entry_text_children`, `filled_value_text`),
    the ObjectDB staying read-only; a title whose refs lead back to itself is used as stored the second time (Perl
    recurses without end). 1011.3492, 1111.3672. Open: a `\cite` in a title reads empty in the tooltip ("cite []";
    the TOC is filled by `fill_in_bibrefs`).
  - RUST-ONLY: `\nameref` showed the target title's number ("2 Target"); Perl drops the title's tag
    (CrossRef.pm:774-777). A `\nameref` nested in a title drops the tag too (KPE #531, where Perl keeps the number): the
nested `\nameref` itself is kept, its title read "Target" in the TOC entry, a `\nameref`/`\ref` tooltip of the
enclosing section ("2 After Target") — verified 77t (quality ledger rank 19 had read this as the ref dropped).
  - RUST-ONLY: a display copy (TOC entry, ref text) named its `xml:` attributes bare: a `\foreignlanguage` title's
    copy was `<text lang="de">`, which the schema refuses (`PostDocument::clone_subtree`).
  - JHEP/JHEP3/PoS `\author` appends, one author per call (KPE #529; astro-ph0611258 keeps its three authors).
  - An author block's email line is the email, its label the contact's `name` (the class's own name when the author
    wrote none: amsart's "Email address: "); a group's shared line gives each
    address to the name it spells (KPE #530; 0911.0568, 0911.0082). A `$^a$` inside a `\footnote`/`\thanks` no
    longer turns an author block into a marked list (`has_author_marks`; 0911.0568's affiliation line read as a
    person). (A group's shared affiliation going to its last name only: DONE 63m.)

  A/B vs 63j: XML 454 papers, 51 change with 0 more errors or warnings (JHEP/PoS papers gain their authors:
  0711.2174 1 → 16, 0811.2355 1 → 15, 1211.0565 1 → 14); HTML 151 papers, 22 change, 0 worse (empty TOC refs 1011.3492
  4 → 0; tooltips fill their refs, "Proof of Theorem 3.2." in 0909.5610, and a title's `\\` reads as a space as in
  Perl). Guards `perfect_kernel_batch63::{ref_in_section_title_fills_toc_and_tooltip, nameref_drops_the_section_number,
  jhep_author_calls_accumulate, author_email_line_label_is_its_name, author_class_email_line_keeps_its_name,
  toc_copy_keeps_its_language}`.
- **63j**: three silent classes from fidelity audit round 2, all Perl-origin:
  - numbered eqnarray, align and gather rows holding only `\label` keep their number and label (KPE #526). Before,
    the row was dropped and the `\ref` printed "( )". hep-ph0208046: 12 missing labels → 0, equation numbers gap-free 1-59;
    hep-th9608109 7 → 0; 1011.4399, 1305.3072, hep-th9412215 all resolve.
  - algorithmic's `\REQUIRE`/`\ENSURE` show their labels (Require:/Input:, Ensure:) rather than "0:", and
    algpseudocode's `\Require`/`\Ensure` theirs rather than line numbers 1, 2; `\Statex`/`\item[]` lines go
    unnumbered as pdflatex prints them (KPE #527; 2201.01230, 1406.5162, 1811.08330, 2507.12875, 2602.10387,
    2309.03769, 1606.06256). algpseudocodex's contrib binding, which dropped the label, takes it since 77s (repro
    list-structure/algpseudocodex_require_shows_its_label GREEN).
  - " and~" separates authors (KPE #528; 2011.10474, 2408.09035, 1406.6147).

  A/B 388 papers vs 63i: 19 change, each one of these, with 0 more errors or warnings. Guards
  `perfect_kernel_batch63::{eqnarray_empty_labelled_row_keeps_number, align_empty_labelled_row_keeps_number,
  unnumbered_empty_labelled_row_stays_unresolved, algorithmic_labelled_item_shows_its_label,
  algpseudocode_require_shows_its_label, algpseudocode_statex_keeps_its_bracket_text,
  algorithmic_empty_and_bracketed_item_labels, author_and_tied_to_the_last_name_splits}`.
- **63i**: affiliations linked by the marks authors show (OD #159, KPE #525). An institute labelled by a leading
  letter or number mark goes to the authors whose names show it. revtex lists all the authors first, then the marked
  `\affiliation`s (2011.01984, 2301.08449, 1510.01235, 0808.2763, 1205.4587, 1509.05545, 2601.16864). svjour3 marked
  list pieces (1406.6147, 2405.03406, 1709.04623, 2003.05787) now link per author. Before, every group author had every
  affiliation, or all were shared. Interleaved revtex groups keep the group rule, and an aa numbered-lines
  `\institute` stays shared (astro-ph0305539). A/B 384 papers vs 63h: 11 change, all to the marks; 0 more errors or
  warnings. Guards `perfect_kernel_batch63::{revtex_marked_affiliations_link_by_mark,
  svjour3_marked_institutes_link_by_mark, institute_with_numbered_lines_is_no_marked_list,
  revtex_interleaved_mark_with_symbol_keeps_the_group_rule, revtex_interleaved_group_with_unanswered_mark,
  revtex_affiliation_with_inner_superscript_is_no_marked_line, revtex_marked_affiliations_with_symbol_marks_link_by_mark,
  svjour3_marked_institutes_with_thanksref_link_by_mark, svjour3_symbol_marked_institute_pairs_by_position,
  revtex_mathrm_mark_interleaved_group_keeps_the_group_rule, revtex_mathrm_marked_affiliations_link_by_mark,
  revtex_textsuperscript_interleaved_group_keeps_the_group_rule, revtex_interleaved_typo_mark_keeps_the_group_rule}`
  (marks read alike on both sides; a line's first mark labels it). Open RED: affiliation_line_inner_superscript_is_its_text.
  Fidelity audit round 2 (80 papers, 63h5): authors 65/69 right
  (round 1 52/73); its next classes (eqnarray label-only row, IEEE "and~", `\ref` in a TOC title, PoS
  `\author`, algorithmic labels) landed in 63j/63k/63l.
- **63h**: a labelled annotation relocated into a title (the AAS title footnote `\title{..\altaffilmark{1}}` +
  `\altaffiltext{1}`, 0704.0478, 1001.2402; elsarticle `\tnoteref`/`\tnotetext`) is the title's frontmatter note, not a
  schema-invalid `<contact>` (KPE #524, OD #461; both papers 1 → 0 jing errors). Orphaned frontmatter annotations go
  where the evidence names (OD #159):
  - svjour3 `\at` institutes go by surname to every author they name, in any order (a shared institute's e-mail
    stays shared);
  - as many numbered institutes as authors pair by position;
  - a sole author's orphans are that author's;
  - otherwise they are shared, and none is dropped;
  - the prefix-stripped label fallback serves creators only.

  svjour3's unlabelled institute pieces are numbered and its `\author` pieces split at " and " (OD #460).
  A/B vs 63g:
  - 64 svjour3 papers: 34 change and none is worse; a script checked each `\at` link against its source. Changes:
    - stranded institutes reach their authors (1205.2208, 1811.03214, 2105.03148, …) or become shared;
    - 3 merged creators split (1406.5162, 2105.12728, 1709.07063 errors 1 → 0).
  - 318 other papers: only the two title papers and kluwer astro-ph0302424 (its sole author keeps the institute)
    change. No paper has more errors or warnings.

  Guards `perfect_kernel_batch63::{aastex_title_altaffilmark_is_a_note, elsarticle_title_tnote_is_a_note,
  svjour3_epj_one_institute_shared, svjour3_sole_author_keeps_the_institute, llncs_sole_author_keeps_the_institute,
  svjour3_two_institutes_pair_by_position, svjour3_at_institutes_link_by_surname_in_any_order,
  svjour3_at_piece_naming_several_authors, svjour3_name_only_piece_keeps_the_institute_shared,
  elsarticle_lettered_addresses_no_one_cites_stay_shared, author_svjour3_and_piece_splits,
  elsarticle_orphan_fntext_stays_off_the_title}`. Open RED: svjour3_marked_institutes_link_by_mark,
  svjour3_name_only_piece_shares_the_next_institute, author_svjour3_mbox_list_splits,
  author_collaboration_name_with_and_stays_one.
- **63g**: svjour3's EPJ style (svepjc3.clo) links an author to its institutes and notes by key (OD #460) —
  `\thanksref{addr1,e1}` requests them, an institute's `\label` labels its affiliation, `\thankstext{e1}` is a labelled
  thanks note. Before, the keys printed as marks and nothing linked (2304.02920; also 2406.12029, 2406.12545, whose
  e-mail notes now reach their authors). A title's `\thanksref` note stays in the title. An institute's own
  `\thanksref` keeps its note, which goes to a trailing creator when no author cites it (OD #159). An amsrefs `biblist`
  prints all its entries, cited or not, each bibliography its own (OD #57; 1012.2719 29 → 34 references). Guards
  `perfect_kernel_batch63::svjour3_epj_*`, `06_cluster_bibliography::amsrefs_*`. The revtex shape of the same audit row
  (2011.01984, 2301.08449: all `\author`s, then `$^{a}$`-marked `\affiliation`s) is the marked-affiliation linking still
  open for per-author classes (OD #459). Repro index-bib/amsrefs_case_distinct_keys_both_kept: two `\bib` keys
  differing only in case merge into one entry (keys lowercased in MakeBibliography, as Perl's `lc` and bibtex) — since
  int27 an honest Error naming the replaced entry (make_bibliography.rs `scan_bibentries`, bibtex's "Repeated entry";
  guard `amsrefs_case_distinct_keys::case_distinct_amsrefs_keys_report_the_lost_entry`); keeping both (case-sensitive
  keys through citation and bibliography) stays RED.
- **63f**: frontmatter fidelity follow-ups (run-336 audit): acmart's `\received` keeps its whole history — received,
  revised (several), labelled — each its own date via the frontmatter key `accumulate` (KPE #522; 2307.05988); initials
  glued to a surname (`A.G.Bogdanchikov`, `Yu.M.Shatunov`; not `St.`/`Mt.`/`Ft.`) read as a name and a `\vspace`'s length
  is no name text (hep-ex0105093 5 → 38 creators; hep-ph9306209 recovers D.P. Roy and Sridhar K.); a name's closing
  initial (one capital) or suffix keeps its period (KPE #523). A/B 318 papers vs 63e: 3 papers change, each a recovery
  (hep-ex0105093, hep-ph9306209, 2307.05988). Next (63g): amsrefs inline biblist kept whole
  (1012.2719), svjour3 `\thanksref`.
- **63e** (landed): author lists (the run-336 fidelity audit's 21 of 73 papers with merged or lost co-authors, all
  silent; KPE #521, OD #459): per-author classes (aastex/emulateapj, amsart, revtex, llncs/aa, IEEE blocks, labelled
  authblk) parse their `\author` as an author block; lines without marks find later names ("and" lines, unfinished
  lists, names after styled affiliations); `\newline`, " \& ", `\altaffilmark` after the comma; names marked before
  them are found by layout and by the affiliation below that answers their mark (Perl drops them all); a per-author
  class's affiliation-line `\thanks`/`\altaffilmark` stays with its author; a merged author is
  `Error:frontmatter:merged_creators`. A/B on 318 papers (the 80-paper fidelity sample + 238 random): 56 papers'
  authors split as LaTeX typesets them, none with more errors, no merged author left. 67 guards
  `perfect_kernel_batch63::author_*`; open RED repros (sectioning-frontmatter/, listed in OD #459): shared-label names
  on lines of their own, unanswered labels, "and"-joined marked names, name-shaped affiliations at an `\and` group's
  head, stacked unmarked names in per-author blocks, amsart `\address` after a split `\author`, glued initials (63f).
- **63d** (landed): pinlabel figure labels (85 papers of run 336's first 265k had `\labellist`/`\pinlabel` undefined):
  pinlabel.sty read raw with its `\ps@begin` placing the picture through graphicx on both driver branches,
  `\pdfximage` keywords + `\pdfximagebbox` (OD #458), `\openin` closes the stream first (KPE #520), PDF object streams
  with `/Filter` first. A/B on the 100 papers with them undefined by then (63b → 63d): errors 843 → 5, no Fatal, none with more errors, 26 with
  more images, one with fewer (1310.1838's unshipped figure: pinlabel's own "not found" text, as pdflatex prints it); the reader's `/Filter`-first object streams reach every Illustrator PDF figure, A/B'd on these papers only.
- **Fleet address-space ceiling → `Fatal:TooManyErrors`** (63c, deployed 2026-10-07 16:35Z): 113 of run 336's first
  253 TooManyErrors papers carry only `Error:post:xpath` "XPath evaluation failed (no libxml2 error detail)" on
  `ltx:XMath`, 15 more were tipped over by it (math0005012, 1209.0448, 1310.4783). They convert with 0 errors in a fresh
  process under the same 5632 MiB `RLIMIT_AS`, on 62zt and 62zu alike (`VmPeak` 3.6 GB for 1106.3786). Live harness
  children sat at ~1 GB RSS but 4.8-5.6 GB of address space (mimalloc reservations), so the RSS recycle never fired
  and libxml2's `malloc` failed. 63c recycles a harness child when its `VmSize` nears the cap
  (`docs/performance/CORTEX_WORKER_HARNESS.md`); after the swap children held 3.1 GB of address space on average
  (4.2 GB max) where they had held 4.8 GB (5.6 GB max). Re-queue the affected tasks at completion (row G). Open: post-processing
  still reports such an OOM as ~100 `Error:post:xpath` → `Fatal:TooManyErrors`; on a detail-less XPath failure under
  memory pressure (`latexml_post/src/document.rs` ≈959/1031) emit one latched resource `Fatal` instead. Also open:
  `fatal/cortex never_completed_with_retries` (150 in the first 145k), likely the same ceiling killing workers.
- **Mouth EoF** (10, runaway delimiters, singletons): 3 × `\input amstex` (amsppt-as-plain, below); `\NOT` 2002.09653
  2212.04182; `\endcaption` 1210.4309 2407.08262; `\@checkend` 1404.3085; ` at ` 0811.2708; `\@nil` 1402.2014.
- **Document Malformed** (8): 0901.1457 ships the AIP proceedings kit and a template driver is taken as the main file.
- **never_completed** (53): fleet pressure; 3 of 4 convert standalone, 1007.0100 is a clean 180 s timeout.
- **IMS bindings**: imsart (contrib) keeps a partial copy of the `\b*` markup — fold it into `ims_support`; arximspdf's
  `{longlist}` is a structural environment where arxbj's is now the class's labelled `\list`.

### Run-329 and sandbox open residuals (61r-62f, 2026-10-04)

Open items from the cortex reruns of sandboxes 2605/2606 (61r) and from mining run 329 (62a-62f). The fixed narratives
(61r's validation, the stop of run 329 and the 62a-62f fixes with their numbers, witnesses and guards) are archived in
[`archive/SYNC_SESSIONS_2026-10.md`](archive/SYNC_SESSIONS_2026-10.md); first-error and Fatal cluster tables of run 329
in `~/data/pk_agents/main/scratch_g/run329/clusters62c/`.

From 61r (sandboxes 2605/2606): a tabular
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

62s: the 2609 templates' new author and title macros, from each template's source (2609 cluster study): neurips_2026
`\workshoptitle` + tracks (113 papers), acmart `\correspondingauthor` + `\if@ACM@anonymous` (76+8), aa `\corrauth` (58),
spconf `\sthanks` (45), revtex's `\move@AU`/`\move@AF`/`\@affiliation` for openjournal (33), wacv via the cvpr binding
with `\thetitle` the title's copy (21; wrapped once when cvpr and wacv both load, and for iccv.sty:477-479), natbib's
`\@ifxundefined`/`\NAT@sectionbib` for iau/JFM (24), acmart `\if@ACM@balance` (11), neurips' `preprint` switch and
tracks' `\@trackname` and `\@noticestring` with the year's own ordinal, location and track wording — also for a style
requested under a directory, whose fallback records the request (`content.rs`, Perl's `\@currname`); no venue note of
the binding's own, as the paper's style copy decides what its first page prints, so a camera-ready workshop paper's
workshop name is not output (DIVERGENCES #448; a workshop-only note awaits a ruling) — acmart's `balance`
options, aa `\aa@emailfont`, newtxmath's `\up<letter>` names (`\upmu`, 15). Guards
`perfect_kernel_batch61::templates_of_2609_keep_their_author_and_title_macros`,
`perfect_kernel_batch61::neurips_notice_names_the_year_of_its_style`. Known limit: a preamble `\newcommand`
of one of these names is ignored like any binding macro's (no 2609 bundle is hurt; older-template papers could be). The 2609 study's other findings:
the expl3 "Mismatched LaTeX support files" errors are the dump regression below (62u), not the harness; 945 EPS
failures: AppArmor `gs` denies `/opt/cortex-scratch` (open).

62t: REGRESSION since 62b fixed — a caption opening with a conditional (`\caption{\ifdefined\x …\fi …}`) ended in
"Extra \else" + Fatal EoF scanning for `\endcaption` (2609.27590, 2609.28438): `\@dblarg` always gives a `[short]` and
`\@caption@@@`'s `\ifx.#2.#3\else#2\fi` took the conditional as its second token; the list entry is now chosen by the
detokenized `[short]` without skipping either text, as are longtable's and listings' (KNOWN_PERL_ERRORS #468). The 2609
classes' commands: llncs `\doi` (a single `\url` link under hyperref; 2609.04690; #492), sagej `\affilnum` (2609.04585), the
appendices' `\Roman` numbering of ieeeconf (`\ifuseRomanappendices`, default on; 2609.16300) and IEEEtran
(`romanappendices`; KNOWN_PERL_ERRORS #491), subfigure's captions chosen the same way as the
kernel's (#468), acronym `\Acp`/`\Aclp`/`\Acfp`/`\Aclu`/`\Acfip` (let to the lower-case forms, the capital not
rendered) and `\acfip` (#490). Guard `perfect_kernel_batch61::captions_and_2609_class_commands_keep_their_text`.
OPEN (design, user 2026-10-05): a paper and its supplement are two top-level documents — revtex's supplement idiom
restarts the title block (`\frontmatter@init` + lets of `\title`/`\author`/`\maketitle` to `\frontmatter@…`,
revtex4-2.cls:2115-2140,3127-3140; 2609.07332, TooManyErrors Fatal) — and the frontmatter API has one title block, so
lets to the binding's `\title` replace the paper's title and append the supplement's authors (the draft is set aside
locally, recorded in the main handoff register); the frontmatter API needs extending for a second document.
62t (part 2): the 2609 classes load what their class files load — lipics-v2021 (array, subcaption, comment with the
`{CCSXML}` exclusion, multirow, tabularx, …, and at the document's start xcolor unless the paper has it plus the class's
named colours, soul's `\textsolittle`; 2609.13401 64 → 44 errors, 2609.10114, 2609.13485 13 → 0), fairmeta (2609.11172),
WileyNJDv5 (and the WileyNJD-v2/WileyASNA-v1 copies the binding serves, which load the same) with caption
(2609.06025 2 → 0), bmvc2k's xcolor and T1 with its `\addauthor` mail read verbatim and set in sans (2609.06007 4 → 0),
lmcs's theorem set with its `defC`/`thmC` styles, tikz, xparse, mathtools (2609.11893 10 → 0), informs3's
`\TheoremsNumbered*`/`\EquationsNumbered*`/`\ECSwitch` (2609.08001), MnSymbol `\llangle`/`\rrangle` (2609.07645);
`\tracingmacros`/`\tracingcommands` stored as numbers so `\the` reads them back (Perl TeX_Debugging.pool.ltxml:214-225;
was silently empty). Guard `perfect_kernel_batch61::the_2609_classes_load_their_packages_and_commands`. The bindings'
named witnesses, re-converted: 2502.11299 25 → 0, 2305.19985 14 → 2, 2305.14448 2 → 0, 2511.16624 1 → 0, the other fifteen
unchanged (lipics 2311.17226, 2211.04601, 2606.01187; Wiley 2203.16535, 2406.06228, 2407.00139, 2504.02281; fairmeta
2412.06264, 2508.07407, 2509.24704, 2605.29955; lmcs 1607.01886, 1607.04128, 1709.06170; bmvc2k's xcolor reload
2605.00310). `\ECSwitch` without a `\TheoremsNumbered*` is undefined, as under pdflatex (the
class defines no default). Residuals:
math-mode `^`/`_` in 2609.13401; a raw `_` in an email (`\lx@add@email`) prints OT1's dot accent (a catcode-12 `_` in
roman OT1, as pdflatex would — classes that set the address with url.sty, in typewriter or under T1 need the binding to
say so, as bmvc2k's now does); the amsthm binding ignores a `\newtheoremstyle` head spec (lmcs's `thmC` drops the
note's parentheses, ours keeps them) and informs3's `\mdseries\scshape` head font (ours bold, :1857-1859); thm-restate does not load thmtools' `\declaretheorem` (2606.01187); 2609.25833's
98 errors are arabtex (`\setcode`, `\RL`); amsart's `\@xsetfontsize` internals (2609.37833) landed in 62z. GENERALIZATION
(open): these bindings hand-copy their class's package list, which drifts by class version and drops options
(fairmeta's `[numbers,sort&compress]{natbib}`); Perl's own dependency scan of the shipped class
(`require_dependencies_except`, as sn_jnl_cls.rs uses it) with per-class exceptions is the general form — needs its own
A/B over each class's papers.

62zl: iopart's `\address` adds its affiliation on each call, as iopart.cls prints one block per call (:240-248),
through `\lx@add@affiliation@marked`; Perl's `\lx@add@affiliations` dequeued the ones before, keeping only the last
(KNOWN_PERL_ERRORS #511; 21 of the 41 2609 iopart papers call it more than once, 2609.01831 lost three of four). In a
marked affiliation list (`affiliation_calls`) an unmarked line continues the affiliation before it, as in
`\lx@add@authors` (each `\\` line had been its own affiliation, the unmarked ones placed by position; 2609.19448) —
not across `\and` (an entry boundary there too), nor a footnote-symbol legend (`* Equal contribution`, `\dag …`),
which stays apart (llncs `\institute`, 2609.06094); address lines are placed as runs once the list is
read (`place_address_runs`, explicit `author:N` labels): where the list puts addresses under its affiliations (one
affiliation, or a run above another that is no footnote-symbol legend; 2609.19448), one address under an affiliation one author requests is theirs and
any other run continues the affiliation; else the runs are the block's — by order when there is one address per
author, else on the shared creator below the authors (2609.06094); a one-address line is kept as written
(`\href{mailto:…}` stays a link), an `\author` call's tail is placed so only at `\maketitle` when that call made
every author, and a marked iopart `\address` before `\author` keeps its affiliations (`email_line_calls`, shared with
`\lx@add@authors`; an email command's wrapper is no longer nested in `\lx@add@email`; DIVERGENCES #52(j)); the mark splitter cuts only outside
groups, and a line opening with a font command or group that holds the marks (`\textit{$^1$A, $^2$B}`, `{\small …}`,
`\small{$^1$A, $^2$B}. Email: …`, `{\color{blue} $^1$A, …}`) is split inside it, the wrapper (a group with its opening
declarations and their arguments) repeated on each piece and what follows split at its own marks, while a
`\thanks{…}` holding the list is unwrapped (2609.00995, 24896: both had regressed under a depth-only rule,
caught by the A/B) (OXIDIZED_DESIGN_DIVERGENCES #52(f)). A new
apa7 binding (contrib) reads apa7.cls itself for its modes, packages and bibliography choice (none by default, :179-216
— the dependency scan had loaded both biblatex and apacite, 17 malformed bibitems in 2609.02899) and maps its author
block onto the frontmatter API: `\authorsnames[marks]{names}` with `\authorsaffiliations{…}` numbered in order, linked
through the kernel's author-line parser (affiliations given first are kept until the names, as apa7.cls builds the
title block from stored lists at `\maketitle`, :780-890); `\author` (the class's one-name `\authorsnames`), `\authornote`, `\note`, `\abstract{}`, `\keywords{}`
(`\lx@add@keywords`); the student page's course, instructor and due date and the journal masthead's journal, volume,
copyright and number, as the modes print them; `\addORCIDlink` as the kernel ORCID link; `\doi` a link to the record, where apacite's own sets url-styled text only (semantic markup before
PDF fidelity, user 2026-10-06); `mask` hides the author identity as the PDF does (:131; user 2026-10-06); the kernel's `\appendix`
stands over the class's (:1041-1075, whose `\section` redefinition the kernel refuses: "Appendix 9" for "Appendix A",
2609.00670). 4 papers, 32 errors -> 0 (2609.00670, 02899, 12869, 19448). RED `marked_affiliation_list_email_line_leaves_no_empty_contact` (in a
marked list an `\email{…}` line had left an empty affiliation contact; GREEN with the shared `email_line_calls`). Guards `perfect_kernel_batch61::{iopart_each_address_adds_an_affiliation,
marked_affiliations_inside_a_font_group_keep_the_font, marked_affiliation_continuation_lines_stay_with_it,
marked_affiliation_wrappers_and_legends, marked_affiliation_review_shapes, apa7_author_block_maps_onto_the_frontmatter}` (and `omnibus_doi_reads_as_a_url`
now on apa6, OmniBus's class, with apa7's linked DOI beside it).

62zm: a bibliography right after a unit that heads it takes its place (user 2026-10-06; sect11.rs
`absorb_bibliography_heading`, the bibliography's `after_close`): titled as it is, or an otherwise empty unit when the
bibliography has no title or the `\bibitem`s opened it in — Pandoc's `\section{References}` over its
`{CSLReferences}` list (2609.02899), `\section*{References}` before `{thebibliography}`; Perl keeps two headings, the
first over nothing (KNOWN_PERL_ERRORS #512, OXIDIZED_DESIGN_DIVERGENCES #456). 62zn: `\bibitem`s in
`{enumerate}`/`{itemize}`/`{description}` no longer err at the list's `\end` (`\endgroup`, shared with Perl,
KNOWN_PERL_ERRORS #513): the auto-open opens the group that `\end` closes. Open: what the list sets before its first
`\bibitem` is undone with the list's group (RED repro sectioning-frontmatter/bibitems_in_a_list_keep_the_lists_settings).

62zk: the informs binding (informs3 and its h/a/noheader copies, informs4 and its copies, informs5) gains the class
API its papers use — the journal switches and the review options (informs4.cls:9-54; informs3 declares twelve journals
and no anonymous-review options), the stored metadata and running heads and feet (:948-985, read back as
`\theRUNTITLE`, `\theJOURNAL`), `\argmax`/`\argmin` (:889-890), `\FUNDING` (:1023), `\FIGURE`/`\TABLE` with their notes
and a grouped body (:2229-2259, 2449-2470), the hanging lists `{henumerate}`/`{hitemize}` (:1702-1763) — and reads `\AUTHOR` and `\AFF` as the class does, one argument each, the
affiliation under its author (:1198-1206): `\AUTHOR{name}{aff}` had swallowed the next `\AFF`, leaving the affiliations
as loose notes (2609.37380). A marked author list links its marked `\AFF` lines by their marks through two kernel
macros, `\lx@add@authors@append` (the author-line parsing without `\author`'s replacing) and
`\lx@add@affiliation@marked` (2609.17368, 22690, 28084); `affiliation_calls` splits a line that starts with a mark
and names several marked institutions at its marks (`$^1$Univ A; $^2$Univ B`), as the author-block parser already
did, and drops a trailing `;` as it dropped a `,` (OXIDIZED_DESIGN_DIVERGENCES #52(f)): 7 of the A/B's 600
non-informs papers (picked for marker-led lines with several marks) changed, all for the better (2609.12549, 16472, 17549, 21045, 21169, 21347, 29760). Under
`blindrev`/`dblanonrev` the class's notice replaces the author block and the history, acknowledgment and author bio
print nothing and `\RUNAUTHOR` names no one, as in the PDF (:987-995, 1071-1076, 1225-1233, 1952; user 2026-10-06);
the binding reads which class was asked
for (`fallback_request`, now also recorded for a prefix fallback such as `informs3noheader`) for informs3's
"blinded" wording, informs3noheader's empty notice and informs3's MOOR acknowledgment. No 2609 paper sets the blind
options (2609.23739 has them commented out); older informs4 copies without `dblanonrev` word the notice as informs3
does, and print the MOOR acknowledgment under `blindrev` as informs3 does; they get informs4's current behavior
(2609.10587, 17368, 25924, 28947, 28263). The notice is a frontmatter note, shown only on hover, where the PDF prints
a bold line in the author slot. 8 papers' errors (2609.10587, 17368, 22690,
23739, 24605, 25924, 37380, 38842). Open: `\argmax` (`\mathop{\rm arg\,max}`, and `\operatorname*{arg\,max}` alike)
parses as the product "arg × max" (math stream); the residual informs diagnostics (2609.28451's own informs3 copy
comments out `\ECSwitch`'s `\ECHowTheorems` call, :2753, so its undefined `\ECHowTheorems` is ours alone; the
`THkey`/`EXkey` theorem styles, whose head is the optional argument alone, revert to plain with a warning, 21433,
28451); `\RUNTITLE`/`\RUNAUTHOR`/`\MANUSCRIPTNO` stay
notes though the class prints them only in the page heads (page furniture). RED
`sectioning-frontmatter/authblk_ieeemembership_after_comma_affiliation_twice` (IEEEtran + authblk: `\author{A. Author,
\IEEEmembership{Member, IEEE}}\affil{Univ X}` attaches the affiliation twice — Rust-only — and drops "Member, IEEE",
as Perl does); RED `affiliation_before_any_author_is_kept` (an `\AFF` before any `\AUTHOR` is lost without a warning;
Perl removes an unlabelled pending annotation silently too); `author_symbol_mark_links_its_affiliation` (fixed by 62zp); `iopart_each_address_adds_an_affiliation` (fixed by 62zl); `split_before_affiliation_marks`, now reached from
`affiliation_calls`, ignored brace depth (`affiliation_marks_inside_a_font_group_keep_the_font`, fixed by 62zl) and took an ordinal superscript for a mark
(`affiliation_line_superscript_ordinal_is_not_a_mark`, fixed by 62zp). Kernel author-mark linking
(`author_suffix_marks_after_commas_link_affiliations`, SHARED with Perl, 2609.25924; `author_prefix_marks_first_line_is_names`,
Perl drops the authors): fixed by 62zp.

62zt: the 2609 frontmatter digest cluster (`\lx@add@frontmatter@now`/`\lx@frontmatterhere`/`\lx@personname`, 9
papers; root-cause by agent, verified): the author splitter keeps an unbraced `\begin{name}...\end{name}` inside a piece
whole (a `{center}`/`{flushleft}`/`{flushright}` around the whole block or an `\and` group is unwrapped first) and
splits at a superscripted separator (SHARED with Perl; OXIDIZED_DESIGN_DIVERGENCES paren-split entry); a note's
`\\` is the plain one, as `\@footnotetext`'s `\@parboxrestore` gives it (`\thanks` in a `p{}` cell, 2609.32661);
IEEEtran's `\begin{@IEEEauthorhalign}`/`\end{@IEEEauthorhalign}` are `\relax` as in Perl (`\authorrowbreak`,
2609.02116); achemso's `{suppinfo}` is its "Supporting Information Available" section (2609.03387, 10829). Guard
`perfect_kernel_batch61::frontmatter_digest_cluster_2609`. Open: RED `ieeetran_authorrowbreak_leaves_no_empty_affiliation` (one
empty affiliation contact; Perl merges the two authors); a block-wide `{center}` followed by more tokens (`\begin{center}Ann
\and Bob\end{center}\thanks{...}`) is not unwrapped, its authors kept together; 2609.13401's frontmatter errors follow a
math-mode leak earlier
in sections/mdps.tex, not this cluster.

62zs: a sectioning unit in a float body opens an `ltx:inline-sectional-block` there — numbered, labelled and identified
in sequence as pdflatex numbers it, schema-valid — and the float's id is fixed before the unit steps its counters
(S1.F1, not S2.F1); user ruling 2026-10-06 after a demo (38 2609 papers, 117 errors; OXIDIZED_DESIGN_DIVERGENCES
#189). Guard `perfect_kernel_batch54::sections_inside_floats_keep_number_id_and_ref`.

62zr: MathSciNet's `\cprime`/`\Cprime`/`\cdprime`/`\Cdprime`/`\cftil` provided within `{bibtex@bibliography}` only,
where free, for `.bib` entries copied without the export's `@preamble` (15 2609 papers, 2609.00627, 2609.11266; user
ruling 2026-10-06; OXIDIZED_DESIGN_DIVERGENCES #78). Guard
`06_cluster_bibliography::bib_mathscinet_fields_without_preamble`. Rulings 2026-10-06 recorded: `\foreach` over a
locked variable keeps Perl's lock (already bounded, PERFECT_KERNEL "Engine"); an eqnarray cell's `+} {` relying on
the `{##}` template is an accepted residual; a `\section` in a float: ruled after the demo, landed in 62zs.

62zq: an author's `\thanks` and `\footnote` read an ordinary argument, as LaTeX's and a title's do — Perl's
Semiverbatim `\lx@add@thanks`/`\lx@add@note` printed their math as text (1,391 2609 papers have math in a `\thanks`;
KNOWN_PERL_ERRORS #515) — and a note's content in a marked author line keeps its superscripts as text
(`\lx@frontmatter@savesups`/`\lx@frontmatter@plainsups`), save a leading mark authors request on a note that is one
legend, which labels the note, the legend going to them (`note_body`; 2609.00885, 2609.19600); an author's requests are
its marks outside its notes (`author_mark_operands`). RED `thanks_with_several_legends_goes_to_each_legends_authors`
(2609.39576: several legends in one `\thanks` stay whole with its author); `\dag` is not among the footnote symbols
`rewrite_symbol_superscripts` shows (an author's `$^{\dag}$` is consumed where `$^{*}$` is shown — shown on the 77t base
when nothing answers it, `show_unanswered_marks`; a text `\dag`/`\ddag` stays an affiliation mark, `is_symbol_mark`,
so a block of `\dag` names over an unmarked institution line was a marked one and the line welded into the last name:
each name's of the names row above it since 77t, R12, repro `sectioning-frontmatter/author_dag_marked_names_over_a_shared_institution`,
guard `perfect_kernel_batch77t::dag_marked_names_over_a_shared_institution`). Guard
`perfect_kernel_batch61::author_thanks_and_footnote_keep_their_math`. Open from its review: RED
`sectioning-frontmatter/thanks_affiliation_list_keeps_superscripts_in_its_text` (the thanks-abuse idiom, 2606.00313:
every `$^{n}$` in the `\thanks` goes through `\lx@affiliation@withsup`, so `10 m$^{2}$ labs.` loses its "2"; the same
on 62zp) — GREEN on the 77t base (cf9986651a), guard `perfect_kernel_batch77t::thanks_affiliation_list_keeps_superscripts_in_its_text`.

62zp: marks glued to a comma in a line that does not open with a mark are the name's before it
(`marks_before_glued_commas`), a `\thanks` glued among them too (2609.37343); an author block's first line is its names even when the marks lead them, and so is a
later `\and` group's first line once the first line did, when no author before it requests its mark and no
affiliation has been read; a
footnote-symbol author mark, still shown, also requests the affiliation of its symbol (`\lx@frontmatter@symbolmark`,
creators only; `\inst{*}` too), so informs4's `\AFF{$^*$Corresponding author}` goes to its author (2609.38842); an
ordinal suffix superscript after a numeral (`5\textsuperscript{th}`, `21$^{st}$`) is shown, not taken for a mark
(`rewrite_ordinal_superscripts`; 2609.01831). OXIDIZED_DESIGN_DIVERGENCES #52(k). Guard
`perfect_kernel_batch61::author_marks_link_their_affiliations`. Guards
`perfect_kernel_batch61::{informs_class_api, informs_blind_review_hides_the_authors,
informs_author_marks_link_affiliations, informs_hanging_lists, marked_affiliation_lines_split_at_their_marks}`.
Open from the 62zp review: RED `sectioning-frontmatter/author_symbol_legend_line_links_its_starred_author` (Rust-only;
Perl links it): in an article `\author`, a `$^*$Corresponding author` line continues the affiliation before it
instead of going to the author whose mark carries `*` (`$^{1,*}$`), because `rewrite_symbol_superscripts` makes its
`$^*$` a visible superscript before the lines are split; `author_thanks_and_footnote_keep_their_math` (SHARED,
KNOWN_PERL_ERRORS #515; fixed by 62zq); RED `author_concatenated_digit_marks_link_each_affiliation`
(2609.37343: `$^{23}$` for affiliations 2 and 3 labels `affiliation:23`, so they sit on a nameless creator; Perl
gives the first and drops the second); `author_and_groups_sharing_a_prefix_mark_are_names` (`\and` groups whose
names open with the mark of an author before them; GREEN in 63e, guard `perfect_kernel_batch63::author_and_groups_sharing_a_prefix_mark_are_names`; Perl drops both authors).

62zj: the jmlr binding defines the ML4H copy of jmlr.cls's track machinery (`\mlhtrack`, the `\ifmlh…` switches; :83-142,
its "no track" `\AtBeginDocument` error left out; 8 papers); IEEEoj's front-matter setters (`\authornote`/`\corresp` as
notes; the dates and `\doiinfo` as dates and a DOI pubnote, as the first page prints them, :3457-3458 — IEEEtj's
too, which were no-ops; an empty `\doiinfo{}` prints none, :3458, 2609.27083; the published and current dates under the
class's names; IEEEoj.cls:3447-3451, 3713-3717; 3 papers); when the lock refused a raw class's `\author`
(`\author:redefined`), author content reads the author markup still undefined there — `\fnms`/`\snm`/`\inits`/
`\prefix`/`\particle`/`\suffix`/`\degs`/`\roles`/`\orcid`, which IOS-Book-Article.cls and econsocart.cls define
only inside their own `\author` — with its OmniBus meaning, undefined again after, a class's or the document's own
definition standing (base_utilities/defs/frontmatter_fields.rs `\lx@author@markup@begin`; 5 papers; Rust-only exposure through
`localrawclasses`, OXIDIZED_DESIGN_DIVERGENCES #444; `\editor`/`\translator` content does not take it yet); amsthm `\@upn` (KNOWN_PERL_ERRORS #509);
newtxmath's `\re@DeclareMath…` helpers (KNOWN_PERL_ERRORS #510). Open from the same root-cause report
(`~/data/pk_agents/main/agent_reports/2026-10-06_rc62zj_template_clusters.md`): the informs binding's class API (fixed by 62zk), an apa7 binding (`\authorsnames`/`\authorsaffiliations`/`\authornote`/`\addORCIDlink`; 4 papers), IEEEtran
`\@IEEEabskeysecsize` (6 papers, fixed by 62zo: the class internals a document copies, KNOWN_PERL_ERRORS #514; open RED repros from it:
list-structure/enumitem_declares_labelindent, sectioning-frontmatter/ieeekeywords_blocks_are_all_kept (GREEN 77t,
KNOWN_PERL_ERRORS #1020) and ieeekeywords_redefined_with_quotation_in_one_column), MathSciNet `MRREVIEWER` macros (`\cftil`, `\cprime`…; 6 papers,
NEEDS A RULING), vmsta2's local `\author` redefinitions under the lock (1 paper, deferred). The IEEEojcsys binding still maps `\authornote` to
`role=authornote` and `\receiveddate` to a note, and lacks the other setters (not yet checked against IEEEojcsys.cls). Guards
`perfect_kernel_batch61::{jmlr_reads_the_ml4h_track, ieeeoj_front_matter_setters,
raw_class_author_markup_reads_within_author_content, amsthm_upn_and_newtxmath_redeclare}`.

62zi: acmart reads `acmart-preload-hook.tex` before its packages, with the class's warning, and defines the ACM palette
(acmart.cls:44-47, 673-680; KNOWN_PERL_ERRORS #507; 2609.29962 298 errors → 0); the biblatex binding defines biblatex's
message commands `\blx@error`/`\blx@warning`/`\blx@info` and variants, which shipped style files call
(biblatex.sty:135-155, 1353-1370; KNOWN_PERL_ERRORS #508; 2609.32652, 58 papers in 2609) — which lets acmauthoryear.bbx
print a "bibmacro missing" warning pdflatex does not, the binding's `\ifbibmacroundef` always answering undefined (RED
`index-bib/ifbibmacroundef_knows_the_standard_bibmacros.tex`). Guards
`perfect_kernel_batch61::{acmart_reads_its_preload_hook_and_palette, biblatex_message_commands_are_defined}`.

62zh: the conversion binaries remove a run's temporary trees however they exit. `process::exit` skips destructors, so
the `latexml_oxide` CLI left each archive's unpacked sources (a `TempDir`) in the temp directory after every archive
conversion — clean, or stopped early by a Fatal — until a per-user `/tmp` quota filled during the 62zg A/B; its exits
after the unpacking now go through `exit_removing` (latexml_oxide.rs). The exits no caller sees — the watchdog's
timeout (124) and memory ceiling (137), the allocation-failure hooks — first sweep a registry
(`latexml_core::watchdog::remove_on_exit`/`remove_registered_paths`): the CLI's archive and archive-output staging
directory, cortex_worker's per-job sources, destination and output zip until the job hands it on, and post's page-spill
directory register there, so a timed-out paper no longer leaves its tree in a fleet worker's scratch directory for the
life of the container. The watchdog and the allocation-failure hooks claim the exit before they sweep: a thread
finishing meanwhile parks (`park_if_exiting`, bounded at a minute, then exits with the claimed code) instead of exiting
with its own code or handing on output built over the deleted trees. Residual: the page-render children
(`LATEXML_RENDER_JOBS` > 1, off by default) have no watchdog of their own and are not killed by the parent's watchdog
exit — orphans over a deleted spill, which can recreate a swept directory; what kills the process from outside still
leaves the trees — the harness's SIGKILL of an unresponsive worker (cortex_worker.rs), a cgroup or
systemd-oomd kill — and so does a panic in a `panic = "abort"` (maxperf) build; for the fleet, a per-slot `TMPDIR` the
harness wipes on respawn would cover these. The image-size unit tests' `lxsize-*` fixtures remove themselves (image.rs `Fixture`).
Residual: the graphics-cache unit tests leave one small `gcache-shared-<nanos>` directory per test process
(graphics_cache.rs, a process-lifetime `static`). Guards `cluster_cli::whatsinout::archive_sources_are_removed_at_exit`,
`watchdog::tests::registered_paths_are_removed_before_an_exit`.

62zg: the 2609 `\endgroup`-cluster triage (report `~/data/pk_agents/main/agent_reports/2026-10-06_endgroup_triage.md`,
groups A-K). Landed: (A) an author block in any alignment stays whole — the splitter's list gains `{tabular*}`,
`{tabularx}`, `{tabulary}`, `{longtable}`, `{array}` (KNOWN_PERL_ERRORS #502; 2609.34061, 34965, 39374, 39909);
(B, Rust-only) `\lstinline{…}` and minted's frozencache `\mintinline` retract their `{` delimiter from the alignment
ledger only when the gullet did not count the closing `}` (a pre-tokenized argument keeps its END catcode;
`settle_verbatim_brace_ledger`, listings_sty.rs; 2609.04372, 29962); (D) `\captionsetup{type=}` digests
`\caption@settype`, setting `\@captype` (KNOWN_PERL_ERRORS #504; 2609.06557, 32742), and a caption that becomes the float
of its box (OXIDIZED_DESIGN_DIVERGENCES #182) now carries its list entry — `\@@toccaption`, which runs before that float
exists, hands it on (`lx@unplaced@toccaption`; the review caught the list of tables repeating one entry and dropping the
next); a sub-float's caption (`\subcaption`, `\@captype` `sub<type>`) becomes its parent type's element, not
`ltx:float`; and `float_to_element`'s closing form (`^^`) takes only open nodes — a text node's previous sibling that
could hold the float (a minipage holding a sub-figure) was chosen and "closed", an "isn't open" error over an unchanged
tree (2609.15558, 33998; the old binary hit it with `\def\@captype{figure}` too), nor — once the walk skips them — end
it at a Math or rule between such minipages and enter the earlier one (OXIDIZED_DESIGN_DIVERGENCES #455). Residual:
sub-figures captioned before their figure's `\caption` take the figure counter as it then is — "0a", then the PREVIOUS
figure's number in each later group — where pdflatex, writing the label at shipout, prints "1a", "2a" (RED
`captions-floats/subcaption_before_caption_numbers_its_figure.tex`); (E) physics' operators read no
size (KNOWN_PERL_ERRORS #503; 2609.12812, 39468; the operatorP comment's witnesses 2003.02721, 2605.20398 re-converted);
(G) no global biblatex `\sort`, and `\DeclareSortingTemplate`/`\DeclareSortingScheme` read their `{<spec>}`
(biblatex.sty:14796), which that stub used to swallow (KNOWN_PERL_ERRORS #505; 2609.39511); (I) `{IEEEeqnarraybox*}` and the box's
`[<decl>][<pos>][<width>]` options, its column glue dropped from the expanded specification (:2280 `\edef`), shared by the IEEEtrantools and IEEEtran
bindings (`define_eqnarraybox`; KNOWN_PERL_ERRORS #506; 2609.32652). Guards `perfect_kernel_batch61::{
author_in_tabular_star_stays_whole, lstinline_in_text_in_align_cell, captionsetup_type_sets_the_caption_type,
captionsetup_type_caption_keeps_its_list_entry, subcaptions_in_minipages_then_the_caption,
subcaption_boxes_around_math_then_the_caption, caption_after_framed_text_keeps_the_reading_order,
physics_operator_leaves_a_sized_bracket,
physics_operator_bracket_across_a_row_break, biblatex_leaves_sort_to_the_document,
ieeeeqnarraybox_star_with_options_and_glue}`. RED from the review: `\text{\lstinline|w|}`'s tex= carries the listing's
internal setup macros (`tools/perfect_kernel/repros/singletons/lstinline_in_text_reverts_as_written.tex`); a text-mode
`{IEEEeqnarrayboxm}` opens no inline math, so its Math swallows the following text
(`alignment-bindings/ieeeeqnarrayboxm_in_text_keeps_its_math.tex`; FIXED 77v, KNOWN_PERL_ERRORS #1011). Open: (C) arabtex's asect.sty:297 `\let\endabstract\endquotation`
(reached through alatex.sty's 2.09 `\begin`) closes the frontmatter `\abstract` with a quotation's end (2609.06402,
10153) — locking `\endabstract` does not help, since `\let` assigns past a lock in both engines (State.pm:380-384
`assignMeaning`; state.rs checks locks only in `install_definition`); (F) `\foreach \x/\text` against a locked kernel
macro and (J) an eqnarray cell `+} {` relying on the `{##}` template need rulings; (H) `\underaccent{$\text{.}$}`
deferred. A `\captionsetup{type=figure}` minipage with `\subcaptionbox`es converts clean but places the sub-figures
beside the captioned figure, not inside it. The `latexml_oxide` CLI left each archive's unpacked sources in the temp
directory (`process::exit` skips the `TempDir`'s drop), which filled a per-user `/tmp` quota during this A/B — fixed in 62zh.

62zf: elsarticle declares the class's other conditionals (elsarticle.cls:40-45 `\iflongmktitle`, `\ifdoubleblind`,
`\ifnonatbib`, `\ifnopreprintline`, `\ifuseexplthreefunctions` — the last stays false, the class's expl3 helpers it
selects being unemulated) and their options: the Elsevier journal styles
(jasr.sty, cnf.sty, jcomp.sty) test them inside the `\if@twocolumn` branches of their `\maketitle`, and undefined they
unbalanced TeX's skip of the false branch, so the other branch's body ran at load (`\finalMaketitle` undefined, an
unbalanced `}`, stray `\else`/`\fi`). The class's `\emailauthor{<email>}{<name>}`/`\urlauthor` (:211-235) are a
frontmatter note "email (name)", as the first page prints them — `\ead`'s contact would land on whichever author
precedes the call; the styles' `\KWD` (defined in their own `\keyword`, which the binding's {keyword} bypasses) is a
no-op. 2609.23725, 23732, 27838, 39431, 09773, 31741, 39502: 4-6 errors → 0. `nonatbib` leaves natbib out, as the class
does (:1242; KNOWN_PERL_ERRORS #501; 2609.05849, 19225, 20719, 23461, 26003). Guards
`perfect_kernel_batch61::{elsarticle_journal_styles_load, elsarticle_nonatbib_leaves_natbib_out}`.

62ze: the mnras binding takes mnras.cls's own settings: its enumerate labels "(i)", "(a)" (:930-940), the bibliography
heading "REFERENCES" (:1312, 1339), with `usenatbib` the author-year punctuation "(Draine 2011)" (:1336 `\bibpunct`),
with `usedcolumn` the `d`/`.`/`,` columns on dcolumn (:1349-1354); the mn binding declared that option as `usedcolum`
(Perl's typo, KNOWN_PERL_ERRORS #500), so dcolumn never loaded. 216 mnras + 18 rasti papers in 2609. Guard
`perfect_kernel_batch61::mnras_lists_citations_and_columns`.

62zd: rasti.cls (RAS Techniques and Instruments, mnras.cls under another name: v3.0 defines the same commands) loads
as the mnras binding (rasti_cls.rs): all 18 rasti papers in 2609 at 0 errors (2609.08700, 09329, 27622, 29860, 35676
had 5-33). cas-common's `stm/mktitle` title-page keys are declared (2609.00281). The box capture's backmatter lift
stops at a float, as at an alignment (base_utilities/blocks.rs `insert_block`, OXIDIZED_DESIGN_DIVERGENCES #205): an appendix
section or a bibliography in a minipage in a table was lifted past it with no diagnostic, leaving the table empty and
its caption at the top level; it stays in the table and errors as Perl's does (A/B over 867 papers: 8 improved; 2609.05763,
a cas paper's abbreviation list in a framed table, now in its table with Perl's `ltx:glossary`-in-`ltx:block` error
instead of moved after an emptied table). OPEN (third stop on one climb —
consolidate): the climb's stops are per-tag (list, alignment, float), and a box in a `quote` still leaks — the lift
moves the insertion point past the open quote, so the text after the box escapes it (RED
repros/boxes-groups/bibliography_in_minipage_in_quote_keeps_the_quote.tex); Perl's `floatToElement` restores the
insertion point after floating past an open container, the likely single rule, but the xebaposter/juradiss hoists
(surpass) must keep working — needs its own A/B. The mnras-binding gaps rasti papers inherited landed in 62ze. RULED (applied 62zs/64d): a `\section` inside a float (38 papers in 2609, 117 errors; 29 have no
other error: 2609.03590, 00943, 05680) is valid TeX — `\@startsection` needs only vertical mode — and could open in an
`ltx:inline-sectional-block` in the float (schema-valid, the appendix as its `ltx:section` stand-in; prototyped, 0
errors, jing-clean) — applied in 62zs for floats (ruling 2026-10-06) and in 64d for list items (ruling 2026-10-09;
OXIDIZED_DESIGN_DIVERGENCES #189, `perfect_kernel_batch54::sectioning_unit_inside_item_or_figure_opens_a_block`). Guards `perfect_kernel_batch61::{rasti_loads_as_mnras, appendix_in_a_minipage_stays_in_its_float,
cas_common_helpers_can_be_renewed}`.

62zc: class bindings that skipped what the class loads or defines. optica-article's binding runs the class file's own
dependency scan (`require_dependencies_except`, as sn-jnl's, without soul — the `\else` arm of its `\ifpdf`): array,
tabularx, multirow, newtxmath were missing (2609.00899, 05706, 06191, 10235, 01145; all 24 optica papers in 2609 at 0
errors); its `fontenc[T1]` now prints `"`, `<`, `>` as pdflatex does (9 papers' text, e.g. 2609.12212's bibliography). cas-dc/cas-sc define cas-common's
name parsers, printers and page styles (no-ops; the frontmatter keeps e-mails and notes itself), so a paper's
`\RenewDocumentCommand\firstname` or `\ps@cas` no longer errors (2609.16168, 16199, 36345, 00634, 20010). The general
form — every class binding scanning its class file — stays open (GENERALIZATION above). Guards
`perfect_kernel_batch61::{optica_article_loads_its_class_packages, cas_common_helpers_can_be_renewed}`.

62zb: three kernel gaps from the 2609 recheck. `\input@path` searched for files kpathsea does not find, after the local
paths and kpathsea, a `/` added as l3file does (l3file's `\file_full_name:n`, expl3-code.tex:12585-12612; content.rs
`find_on_input_path`, OXIDIZED_DESIGN_DIVERGENCES #454):
71 papers in 2609 set it, 2609.02998's class style (and the natbib its `\citep`s needed) was missing. `\pdfstartlink`
reads its rule, attr and action specs (pdftex.rs `LinkSpecification`, KNOWN_PERL_ERRORS #499): the bare no-op typeset
`attr {…} goto name {…}`, 57 papers in 2609 use it (AAAI forbids hyperref; 2609.00161). aa's `\pmatrix`/`\cases` take
amsmath's environment form when `\@currenvir` names them, as `\matrix@check` does (KNOWN_PERL_ERRORS #498; 2609.02726,
2609.04318). Guards `perfect_kernel_batch61::{aa_cases_and_pmatrix_take_the_environment_form,
pdfstartlink_consumes_its_spec, input_path_finds_a_style_in_a_subdirectory}`.

62za: `\minipage`/`\endminipage` locked against class and package files (`:locked@files`, state.rs `is_name_locked`;
sect12.rs, OXIDIZED_DESIGN_DIVERGENCES #453), the document's own definitions free as in Perl: iucr.cls's `\endminipage`
(kernel internals, to drop the footnote rule) broke every minipage and, through tikz's `\pgfutil@endminipage`, every
`text width` node: 2609.07722 and 2609.02106, 265 errors each, Fatal → 0; memoir's own save-and-restore is retired. From
the 2609 tally (cortex bundles of worker a01c8944b4: 3,374 Error, 64 Fatal), class gaps the class files fill:
aaai2027.sty's `\corresponding` and aaai's `\equalcontrib`, defined only inside their `\@maketitle` with their footnotes,
now carry the note on each author (kernel stubs, `DefinitionOrigin::Stub`, which the document's or a raw file's own
`\newcommand` replaces, as copernicus.cls:1673's; sect05.rs; 223 AAAI-27 papers); egpubl (argument-less
`\PrintedOrElectronic`/`\BibtexOrBiblatex`/`\biberVersion`, whose argument swallowed the paper's `\ifpdf`; `\ConfName`; the
remaining editor setters kept as the others; the paper-type, page-number and licence selectors; `\excludecomment{CCSXML}`;
`\teaser` a figure at `\maketitle`, as `\@maketitle` sets it with `\def\@captype{figure}`, after the abstract since the
schema keeps top matter ahead of figures; `\EGyear` only stores the year `\ConfYear` keeps), bmvc2k (geometry,
`\BMVA@blfootnote`, `\bmvaEtAl`, two-argument `\runninghead`), sn-jnl `\unnumbered`/`\numbered`, acmart's section fonts
(`\@secfont`, patched by pvldb.sty): 2609.00420/00555 1 → 0, 2609.00994 23 → 0, 2609.00732 13 → 0, 2609.00981/08090 2 → 0,
2609.05015/05615 1 → 0, 2609.00548/02328 1 → 0. Open: acmart's teaser rewrite (acmart_cls.rs `\lx@relocate@teaser`)
never matches, since the abstract precedes the teaser in the built document, and moving the teaser ahead of the abstract
would break the schema (`ltx:document` takes its top matter first) — retire or rethink; raw-loaded copernicus.cls drops `\affil` and its equal-contribution note, both typeset only by its
`\maketitle` body, which is not replayed (`\citati@nbyarticlenumber`): 2609.05557, 07363, 08383, 12496, 13482, 17175,
21633, 36077; 931 papers' first error
`imageprocessing:failed_to_convert` (not examined); `\cprime` in MathSciNet `MRREVIEWER`/`FJOURNAL` fields of cited
`.bib` entries (40 papers; Perl's MathReview synthesis digests the reviewer too; the always-on stub was retracted by
maintainer decision 2026-07-27, latex_constructs_rust_only.rs — needs a ruling); the tally's `_` cluster (216 papers) is
mostly gone on HEAD (splncs04 `\doi{…_14}` in the `.bbl`, now clean); Perl-origin, pdflatex-clean: `\lefteqn{…} & &` in
eqnarray (Perl's `\lefteqn` spans 3 columns, so the tabs the idiom writes are extra, 2609.03020: fixed in 65o, 10 → 0
errors, OD #556) and an etoolbox
`\AtBeginEnvironment{table}{\begingroup…}` + `\AfterEndEnvironment{table}{\endgroup}` pair (Package.pm:1918-1925 binds the
environment's mode into the hook's group, 2609.01057). Guards `perfect_kernel_batch61::{class_endminipage_does_not_replace_the_environment,
document_endminipage_definition_is_kept, class_bindings_2609_clusters}`.

62z: amsart's size machinery in ams_support (amsart.cls:169-219, 258-296): `\@xsetfontsize`, `\@currsizeindex`,
`\@adjustvertspacing`, `\@mainsize`/`\@ptsize`, and each size option's `\@typesizes` (10pt the default), which a class
built on amsart redefines its size commands with (m2an.cls:241): 2609.37833 undefined `\@xsetfontsize` then Fatal
PushbackLimit → 0 errors; the class bindings declare the size options as the classes do (ams_support keeps them no-ops,
as Perl, so a package does not run the document's size again), so `\LoadClass[12pt]{amsart}` sets the 12pt sizes. `\label` no longer digests `\@currentlabel` into a `LABEL@` value nothing reads (sect11.rs):
read lazily after a list whose `label=` redefined `\theenumi`, it reached the document's self-referential `\theenumi`
(KNOWN_PERL_ERRORS #496, OXIDIZED_DESIGN_DIVERGENCES #452; Perl hangs): 2609.13327 Fatal Recursion → 0. mathtools' paired delimiters follow the package
(`\delimsize` in a group, `\<size>l`/`\<size>r`; KNOWN_PERL_ERRORS #497, OXIDIZED_DESIGN_DIVERGENCES #451, ams
`mathtools` golden: three `tex` attributes): 2609.17447 861 errors, Fatal → 0. A/B over 551 papers (62y5 → 62z10, before the `\@ptsize`
restore): no output changed. Open (RED `tools/perfect_kernel/repros/math-parse/sized_bars_do_not_merge_into_a_norm.tex`, math
parked): two adjacent sized bars of different sizes merge into one `‖` (`\Bigl\lvert\bigl\lvert y…`), which
`\abs[\bigg]{\abs[\big]{y}}` now reaches; `\reDeclarePairedDelimiterInnerWrapper` still writes wrappers the X forms do
not read. Guards
`perfect_kernel_batch61::{amsart_xsetfontsize_sizes, label_after_a_relabeled_list_does_not_loop,
paired_delimiter_size_and_delimsize}`.

62y: four 2609 Fatal witnesses. (1) aastex7/aastex701 load rotating before their own `\rotate` (aastex701.cls:11416,
12196; aastex631 and earlier do not); the binding now does too (aastex_cls.rs, by the requested name), so a
deluxetable's `\rotate` stays its no-op instead of rotating's `{rotate}` environment, which the versioned fallback's
dependency scan of the class file loaded after the binding (Perl's versioned fallback scans nothing, Package.pm:2226):
2609.09266 Fatal → 2 errors (its .bib's ADS `\lt`, `\CID`). `rotatetable(*)` (:12212) set as its body: 2609.01052 → 0.
(2) `\left`/`\right`/`\middle` read a delimiter character by its `\delcode` (tex.web §1160): `DelimiterToken` +
`stomach::digest_as_delimiter`, where a math-active `(` (`\mathcode`(="8000`, active meaning `\left(`) is the
parenthesis (KNOWN_PERL_ERRORS #494), for `\bigl(` too (`TeXDelimiter`, one token); a math-active character whose active
meaning is undefined self-inserts as the plain character, where it was dropped (Perl: Γ, KNOWN_PERL_ERRORS #495):
2609.40266 Fatal Recursion → 0. (3) autobreak bound (latexml_contrib): lines as the package reads them (a line end inside
braces stays in its line, a leading `,`/`.`/`;`/`:` joins the line before), the first non-empty one the left-hand
side, the rest after the tab, the width-driven breaks left out; the raw package needed
amsmath's `\start@align`/`\collect@body`, which the native `align` never runs: 2609.08470 Fatal → 0 (and 2609.04368,
which loads autobreak unused, loses csquotes' failed patch of the raw package's `\collect@body`, 1 → 0, as pdflatex).
A/B over 551 papers (the 157 aastex + every 20th of the first 8000 2609 ids, 62x8 → 62y2): 2 improved, 0 regressed,
1 output changed (2609.01052). Open:
2609.13328's `\def\year{\pdfprimitive\year}` (an ieeeaccess `\def\year#1` workaround) loops, as `\pdfprimitive` is `#1`
(pdftex.rs:611, Perl pdfTeX.pool.ltxml:208 likewise); a faithful one needs the engine's primitive meanings kept in
State past the TeX pools (before plain_bootstrap) — one paper, deferred. Remaining 2609 Fatal witnesses (root causes in "Open rows from the main-stream handoff register"): 2609.07018 (nested
tikzpicture under `\widetilde`), 2609.14043 (tikz-timing). Guards
`perfect_kernel_batch61::{aastex7_rotate_and_rotatetable, math_active_delimiter_is_its_character,
math_active_character_without_meaning_is_itself, autobreak_lines_after_the_left_side}`.

62x: AASTeX's table columns (aas_support_sty.rs), ported from aastex701.cls (TL 2025 copy). `D` decimal columns
(:12010): after `\decimals` a cell's first word splits at its first `.` (`\lookfordecimal`, :11980), both parts in math
(a sign is a minus), the point only before a fraction, the rest of the cell kept; without `\decimals` the cell stays whole
(each deluxetable resets `\decimals`, :11330; Perl's split uses an undefined `\lx@alignment@align`, KNOWN_PERL_ERRORS
#493). `C`/`L`/`R` are the class's math cells
(:8857-8859), with `$` active in every table (:8849, tabular's and deluxetable's bindings) and doing nothing there, as in
the class's `\nodata` (:8268-8269); `\centerwidetable` (:13497) and the `\movetabledown`/`\movetableright` registers
(:11756, 13690) defined. OXIDIZED_DESIGN_DIVERGENCES #450. Kernel: a `$$` display opens and closes on a math shift by
meaning (tex.web §1138, §1197; tex_math.rs `next_is_math_shift`), so the active `$` pairs in a `p{}` cell; a
`split_tokens_delimited` math span closes as it opens, by meaning.
2609.05675 102 → 0 errors (was Fatal), 2609.00308 102 → 0 (was Fatal), 2609.06985 63 → 0, 2609.04324 12 → 0,
2609.06112 2 → 0; A/B over the 157 aastex papers in the first 8000 2609 ids (62w3 → 62x2): 5 improved, 0 regressed,
10 outputs changed — the 5, 3 more whose `C`/`L` cells are now one math expression (2609.00140, 2609.00283,
2609.07836: `-16.44$\pm$0.07` → `−16.44±0.07`), and 2 whose `\nodata` keeps the class's surrounding spaces. Open: a
`\colhead` over a `D` column covers its integer half only (the class spans both, :12027-12028 `\CheckNumberAndSwitch`;
no witness, `\twocolhead` is the documented form), and the class's `d` (:12011) is the hidden pair, which this binding,
like Perl's, prints as `D`. Guards `perfect_kernel_batch61::aastex_decimal_and_math_columns`,
`perfect_kernel_batch61::active_math_shift_pairs_for_display_math`.

62w: the ar5iv profile's `iflimit` 16M → 48M (ar5iv_sty.rs). Five 2609 pgfplots/tikz papers ended `Fatal:Timeout:IfLimit`
with no error before it; measured without the limit (a release probe build counting conditionals), three are finite and
complete: 2609.10563 (19M, 48 s), 2609.16075 (21M, 45 s), 2609.07725 (39M, 78 s); so do 2605.27177 (raw mhchem, 56 s)
and 2605.04377 (98 s). Any runaway still stops at the worker's 180 s per-document timeout (the dispatcher lease is
240 s); the limit only ends a faster one sooner with its label, and one between ≈89k and ≈267k cond/s now ends at
the timeout. Outcomes that change label: 2609.30783 (lirseg, a memory runaway) now ends at the box-list memory budget
(`Fatal:Stomach:MemoryBudget`, which rides the memory cap: 8 GB probe, 5.25 GiB in the cortex fleet), and 2609.08966 (pgfplots
`shader=interp`, slow) and 2605.12601 at the 180 s timeout — performance items (open). Guard
`perfect_kernel_batch61::ar5iv_profile_sets_its_runaway_limits`.

62u (REGRESSION 2026-10-05, critical): 12,144 of 38,624 2609 papers ended as Error with expl3's "Mismatched LaTeX
support files" + "Cannot run piped system commands" (expl3.sty:64-78). Cause: the worker's `LATEXML_DUMP_DIR`
(`resources/dumps`) held dumps built from `/usr/local/texlive/2025` (L3 2025-11-06), copied there from a gates dump
dir on 2026-10-04 21:28, while the worker reads packages from `/usr/share/texlive` (L3 2026-01-19); every engine
revision fails the same way with that dump (61m17..62s11), all pass with a dump built from the runtime tree. Fix: the
dumps regenerated by `tools/make_formats.sh` in the worker's environment; a dump now records the format files it was
built from (`# source` header: `latex.ltx` + `expl3-code.tex` / `plain.tex`, size + CRC-32) and every loader passes
over one this TeX tree does not have (`dump_paths::dump_matches_tree`: `resolve_versioned_in_dir`, the embedded
dumps, `LATEXML_DUMP_PATH`), the files looked up by tree subpath so a document's own `expl3-code.tex` never
counts; when none matches, `latexml::format_dumps` builds the format in process, once per tree and build, in a
directory with no TeX files, under a lock, into a per-user cache (`~/.cache/latexml-oxide/formats/`,
`$LATEXML_FORMAT_CACHE`), keeps it only if the build logged no error, and uses it, else the base branch (DUMP_DESIGN
"Tree match", DIVERGENCES #449). Dev dumps from before the stamps are passed over: regenerate them with
`tools/make_formats.sh`. Guard `dump_gate_init::a_dump_built_from_another_tree_is_not_used` (an older tree's forged
dumps — bogus stamps, L3 date 2000-01-01 — as the only dumps allowed: expl3's mismatch on the old engine; now both
formats built into the cache with this tree's stamps and loaded from there, on any TeX Live; the document's own
`expl3-code.tex` never read; matching dumps load and build nothing). Perf audit: the stamps cost a CLI process +0.6M
instructions (+0.09%, 78.2 → 78.9 ms on a one-line article, within noise) and `cortex_worker` once per child
(DUMP_DESIGN "Tree match"). 62v: per format, builds that have not succeeded are counted
(`.attempts.<kind>`, so a build dying with its process counts), two in a row mark it `.failed.<kind>` for a day, and a
process that waited while a build failed does not retry it at once; a dump on disk wins over the cache's, the embedded
one comes after it; a failed build is a counted `Warning:dump:build_failed` in each conversion's log; the embedded dumps'
`$TMPDIR` cache is per user and owner-checked (`latexml_core::util::private_files`, unit-tested).
Rerun: run 332 (error/errmessage), 1,532 of whose papers converted during a dump-less minute of a second regeneration
and need regenerating again (dumps must be swapped atomically under a running fleet).

62r: textcomp's symbols are TS1 text symbols that dispatch on the encoding (KPE #489): Greek `\textmu` and babel
greek's `\figurename` print μ, not µ; a document's per-encoding `\DeclareTextCommand` takes effect.

62q: a captioned minipage or parbox in a float is the float its caption numbers (DIVERGENCES #447, KPE #488): its
panel takes the caption's id, tags, list and `\label`s, a table's are tables — side-by-side captioned minipages read
"See 1 and 2", not "2 and 2" (shared with Perl; the common idiom, and 1601.03744's tabular cells).

62p: graphicx's pdftex driver chain at `\begin{document}` (pdftex.def → epstopdf-base → pdftexcmds → iftex, when
`\@curroptions` is not empty) defines `\ifpdf` afresh (KPE #487): a paper's `\let\ifpdf\relax` no longer leaves
JINST's `\label` a stray `\fi` (1310.6454, a Fatal). `\ProcessOptions` and the package loader keep `\@curroptions`
as latex.ltx does, key=value processors leave it (40 bindings say so: `ProcessKeyOptions!`/`key_options_processed`);
amsmath passes `namelimits` to amsopn. 31 synthetic probes agree with pdflatex on iftex/pdftexcmds loading.

62o: a captioned minipage in a tabular cell is a figure panel in the cell (KPE #486): the box placement no longer climbs
out of an alignment (it left `<td>` open in the figure, 5 malformed errors, 1601.03744), and caption material left in a
box in running text becomes a figure panel in an inline logical block (labels: 62q).

62n: tocloft keeps the kernel's lists (DIVERGENCES #446): interpreted raw for its `\cft…` parameters, its
`\tableofcontents`/`\listoffigures`/`\listoftables` replaced by the kernel's under its own condition and times —
they ran `\@starttoc` on a `.toc` LaTeXML never writes, losing the `<TOC>` (SciPost.cls; shared with Perl).

62m: the ACM SIG classes (sig-alternate, sigchi; shipped, run raw since 62k) keep their authors: `\alignauthor` is of the
`\and` family, its brace group and `\affaddr` read as their content (DIVERGENCES #445; open: 1609.00045, 1608.06253), and a missing `figure`/`table` counter is made
once the class has loaded (KPE #485, sig-alternate.cls:699 `\@ifundefined{figure}`). 1605.02827 18 → 1 errors,
2003.09061 24 → 0 (4 creators with affiliations, OmniBus merged them), 1607.07514 22 → 0, 1906.01122 17 → 0,
1707.05754 30 → 0.

62l: a PASJ binding (`latexml_contrib` `pasj00_cls.rs`, also pasj01/pasj02): the shipped class is interpreted raw and
the kernel `\caption` (`\lx@caption`) put back over PASJ's own, which bypassed `\@caption` (captions without number,
dangling labels since 62k); `\@maketitle` emptied; `\KeyWords`, `\affil`, `\altaffiltext`, `\email`, `\orcid`,
`\Received`/`\Accepted` go to the frontmatter; the binding splits the one `\author` itself (commas, `\&`, `\\`; a
name's marks follow its comma). 11 PASJ papers: 0 errors
but 1601.03744 (3, a shared kernel defect, RED), captions back (≥ 62j4), keywords and affiliations gained. RED:
`\caption[<number>]`, captions in minipages in a tabular.

62k: the arXiv profile interprets a shipped class without a binding raw (`localrawclasses` in ar5iv.sty; DIVERGENCES
#444; a binding route, Perl's prefix alternate, Rust's case step or a path-prefixed class's basename binding, still
wins): OmniBus-class papers 1,151 → 220 errors and 2 → 89 error-free (127), random run-329 error papers 4,645 → 2,714
and 45 → 155 error-free (383), general sample 752 → 691 errors (873), no body text lost. With it: extsizes bindings
(`\@ptsize` in points), `\documentstyle` binding-first probes and a raw `.cls` probe, tocbibind conditionals (KPE
#482), ragged2e `\LaTeX*` saves under `newcommands` (KPE #481), IEEEtran `\ifCLASSINFOpdf` from `\pdfoutput` (KPE
#480, 22 papers), and four kernel paths raw classes reach: an empty `\@startsection` type (KPE #483), a re-let
`\@startsection` recursing through `\@sect` (`\lx@startsection`), the `\@maketitle` deposit's diagnostics hold, the
locked `\NAT@wrout` (KPE #484). Open, worse than OmniBus (DIVERGENCES #444, RED repros): a paper's `\let\ifpdf\relax`
that graphicx's driver undoes in TeX (JINST, Fatal), JINST's locked-`\author` flag, raa crossed groups,
`\alignauthor`/`\newauthor` vs `\lx@personname`, `\@ifundefined{figure}` seeing the kernel's environment
(sig-alternate).

62j: elsarticle `\jtype` (KPE #478) and txfonts `\varv`/`\varw`/`\vary` (KPE #479) fixed; the run's one panic,
1111.1991 (Rust-only: an auto-opened `ltx:picture` sized the `{picture}` whatsit being absorbed and stored the size
memo on it — "RefCell already borrowed"), fixed by `set_memo_property` (a busy box keeps no memo; any other re-entrant
property write is an Error, not a panic). Residuals: `Digested::compute_size` answers 0 for any box it cannot borrow
mutably — one being absorbed too, unless all three of its sizes are requested (1111.1991 passes because the
`{picture}` carries them), not only the astro-ph0310145 self-reference; a `\parbox` after a text `\put` sizes the
auto-opened picture 204.16×22.56 where Perl gives 7.3×5.96 (pre-62j); 0709.1145's `\LT@array`.

From the 62i review: amsmath `multline` drops a `\tag` and numbers the equation (pdflatex "(Q)"; the counter then
runs one ahead) — `repros/alignment-bindings/multline_tag_replaces_the_number.tex`, FIXED 77v (KNOWN_PERL_ERRORS #1010).

From the 62h in-scope manual pass (each pdflatex-clean, so Rust-only; both were Fatals on 62g1 and stay failures):
- typog-example: Rust's `\fontname\font` gives the fallback `cmss9` for an NFSS family (`Inter-LF`, T1Inter-LF.fd)
  where pdflatex gives `Inter-Regular-lf-t1 at 9.0pt`, so the document's `\projectoutfontname#1-#2-#3\relax`
  (typog-example.tex:263-264, inside an `\edef`) finds no `-`. 62g1: the argument ran to the end of the file (Fatal);
  62h: "Paragraph ended before \projectoutfontname" ×11 (tex.web §395's "extra }" at the `\edef`'s close is not
  implemented), then the run times out near line 1813.
- chessboard_and_beamer: skak's `\mainline` under beamer `\only<n>` reports "black, not white, to move", then
  `\EatNumberA(#1.#2)` (skak.sty:1319) finds no `.` — 62g1 Fatal at the end of the file, 62h 4 errors.

From run 329 (62a-62c):
- `\jobname.aux` re-read: FIXED 66f — `\begin{document}` opens the `.aux` as latex.ltx's `\document` does (OD #576);
  lamuphys.sty:1240-1247, caosp.sty:916, jpsj3.cls:90-96 and bio.cls `\input \jobname.aux` at `\enddocument`
  (cond-mat9607109, astro-ph9805185, 2605.24927, 2605.04825, 2605.28673, 2605.20125: 1 error → 0).
- Still Fatal after 62b, each erring under pdflatex too: 1409.3401 (pdflatex emergency stop), 2105.00771 (101 = 101), cs0702042 (citesort.sty missing from TL; Rust
  then 101 math-alignment errors, pdflatex 3).
- Rust-only, still open: CJK GB `\@inpenc@undefined` (1007.1512); singles 1205.5844 (`\@journal`), 1711.06710
  (`\@rticle@options`), 1811.00686, 1907.03566, 2004.12109 (memoir font command), 2105.02164, 2203.13766, 2207.02360,
  2306.10394, 2409.00304.
- AMSTeX `\documentstyle{amsppt}` documents still get the LaTeX format (Perl reads them plain). Kept plain they measured
  worse: 64 sampled 1996-99 amsppt papers, 192 → 319 errors, 1 → 2 Fatal (math9806005 0 → `TooManyErrors`; plain
  AMSTeX lacks `\ams@return@opt@arg`, `\rightpoint`; math9603201, math9709201, math9801043, math9803037). Fix the
  plain-AMSTeX gaps first, then classify them plain (`converter.rs` `compile_route`); sample and A/B in
  `~/data/pk_agents/main/scratch_g/run329/amsppt/`.
- `\input eplain` (pre-existing, tex clean): `Error:undefined:\auxfile`, then `Fatal:ParamSpec` "Parameters for `\@`
  not in order" (Perl: the one error).

From 62d/62e: Open, pre-existing (review probes `scratch_g/review62e/r3/r4.tex`, `r5.tex`): with
`linesnumbered` the first statement right after `\caption` takes no number; `{quote}`/`{center}`/`{flushleft}` end no
algorithm line, so the text after them stays on their line; a statement inside `\href` breaks inside the link
(`ltx:ref` does not auto-close). Not fixed: missing journal classes (raa `\pagerange`/`\volnopage`, ws-* `\bodymatter`,
iopart `\ioptwocol`, old A&A `\thesaurus`), text-mode `_`/`&`/`^`, and the Fatals `TooManyErrors` (49),
`never_completed` (24), `PushbackLimit` (21 then, 86 by 126k papers; fixed for harvmac by 62f below).

From 62f: Side risk, open: any genuine `\@ifnextchar` autoload in a plain
  document still replaces the document's macros through the dump apply.
- 62i fixed six of the 17 PushbackLimit Fatals left after 62f: 2108.13640 (siunitx `S` cell read past csvsimple's
  `\relax`, KPE #474), 2407.10582 + 2505.05474 (Rust-only: a document's own `\emails`/`\affiliations` taken for an
  IJCAI author-block marker, `has_ijcai_section_marker`), 2309.08676 + 2502.21053 (`\DeclareEmphSequence`:
  `\selectfont` defined no font identifier, KPE #477), 2408.12869 (`\tag` text mentioning `\theequation`, KPE #475),
  2305.06365 (`\let\@@cite\cite`, KPE #476 / DIVERGENCES #443; its revtex `\@AF@join`/`\twocolumn@sw` stay
  undefined). 0902.2281 and 1007.3028 are AMSTeX documents (`\input amstex` + amsppt: label.def needs amstex.tex's
  `\W@`, `\eat@`), the amsppt-as-plain residual. 2408.12869 then shows 3 `<ltx:XMArray>` in `<ltx:figure>` errors.
- Open, still Fatal: 2302.07191, 2501.17908,
  2509.10120 (error before run 329, Fatal now); unclassified: 1807.10890, 1807.08405, 2107.07104, dg-ga9410001,
  2406.19307. A `\noexpand`-ed undefined control sequence raises "undefined" in csvsimple's
  space trim where TeX treats it as `\relax` (`g62f/c_m.tex`, a CSV cell `\foo` with `\foo` undefined).



## Rows removed as landed (2026-10-23)

From SYNC_STATUS "Silent-loss inventory" (FIXED / CLOSED / DONE rows whose status names nothing open).

| # | problem | status |
|---|---|---|
| 1, 2 | open frontmatter body at `\end{document}` → empty `<document/>` | FIXED int26 (OD #754) |
| — | command-form `\keywords` swallowing the document into `<ltx:keywords>` (found by the int26 A/B) | FIXED int26 (OD #755) |
| 3 | real `\cite`s, empty bibliography (6 papers) | CLOSED stale 2026-10-18 (R3b) |
| 4 | `&` in a fenced macro argument truncating the document | CLOSED int26 (R3d, KPE #981) |
| 19 | cryptocode: pseudocode lines after the first lose their numbers; text mode `\@pc@lb` (20 papers) | FIXED int27 (76a: `\halign` redefinition run where binding alignments reach `\halign`; 27 papers 45 E → 0, numbered lines 145 → 1,180) |
| 13 | floatrow `\floatfoot` in a plain float / a `\DeclareNewFloatType` float lost | FIXED int27 (floatrow_sty.rs `\lx@flrow@plainfoot`); the float itself is kept |
| 17 | longtable caption row: the cell before `x & \caption` lost | FIXED int27 (kept, LaTeX's "Misplaced \noalign"); other shapes lose nothing |
| 9 | revtex supplement title replaces the paper's | CLOSED on head 2026-10-18 (probe 67int35): only the set-aside binding aliases replaced the title; today revtex's `\frontmatter@init` is undefined (an honest Error, 2609.07332) and a second `\title`/`\author` after `\maketitle` prints in place, the paper's title kept. Residual (synthetic, no reach): in a paper with no heading and no bibliography before its supplement, the seal (`seal_digested_frontmatter`, base_utilities/frontmatter/queue_digest.rs) never runs, so a second `{abstract}` replaces the first (0 errors; pdflatex prints both) — of the 6,000 projection-sample papers 74 have two abstracts, 56 with no heading or bibliography between, and none loses its first abstract's text to a later one (the two flagged by the word probe are Error/Fatal papers: 2202.10792, 2007.05437) |
| 10 | deepseek.cls `\correspondingauthor` note, printed only in the title page's `firststyle` footer | FIXED int27 (deepseek_cls.rs: the shipped class loads raw; the note is added where its `\maketitle` selects the footer; 2401.14196) |
| 24 | aastex `\figcaption` outside a figure: the text a `<p>`, no caption, no number | FIXED int27 (aas_support_sty.rs `\@figcaption` = a figure with `\caption`, aastex701.cls:8111-8114; astro-ph9806172 0 → 3 captions) |
| 23 | an.cls `\abstract{}` (`\AtEndOfClass{\let\abstract\aaabstract}` store, printed by the class's `\maketitle` as a plain paragraph; astro-ph0611016) | FIXED 77b (an_cls.rs stand-in: the shipped class read raw, an.cls:414 `\aaabstract` = `\lx@add@abstract` before the :98 hook copies it; OD #763). Reach ~26 of 72,694 sampled papers (≈ 0.036 %); prefix fallback side effect: andp2012.cls (1504.05091, 1810.06303) gains a missing-binding Warning, XML unchanged; v1.03 (astro-ph0306340) prints `\abstractname` before the store — the label is the abstract's name, the deposit takes the line out of `\@maketitle` (sect05.rs `note_title_page_part`), no orphan bold "Abstract"; `{abstract}`-environment papers keep the title block's keywords/© lines; without a shipped an.cls the loader's missing-binding Warning is raised again (`omnibus_without_copy`, shared by the cimento/compositio/jinst/apa-family stand-ins) |
| 23b | an.cls title block (keywords line, © line) dropped when the paper sets no `\institute`: the class's refused `\maketitle` gives `\@institute` its default (`\ch@ckobl{institute}{Address(es) of author(s) should be given}`, v1.09a :532), so the deposit's `\institutename` (:703, reads `\@institute` :427/:440) raises 1 error and the whole deposit is dropped — the `\keywords` text is lost with 0 errors (base too; reviewer probe an_noabs, which sets no `\abstract` either — a paper with `\abstract` but no `\institute` loses it the same way, one with `\institute` keeps it) | FIXED 77h, finished 77i (frontmatter_stores.rs `\lx@captured@stores`, `decide_unset_nulls`: in the title-page deposit and replay a rerouted store the paper never set, still undefined, is defined — the undefined `\@institute` no longer drops the deposit, the "Key words" line is kept — as `\relax` or `\@empty`, whichever the title page prints less under, decided by speculative runs of the deposit and replay (OD #767): `\relax` where ptptex.cls:88-104 tests `\@ifundefined{@pubinfo}`/`{@recdate}` (77h r1's `\@empty` printed its journal line and "(Received )": hep-th9803240, 1309.6905, 0911.1235, 0708.3877, 0801.2466, 1009.4028, 1007.1507, cond-mat0411234; jpsj 0903.1491, 0912.1097, 1101.2977, 1203.5392), `\@empty` where a class tests `\ifx\@institute\@empty` (77h r2's `\relax` printed an orphan "Institute:"); guards `perfect_kernel_batch77h::an_without_institute_keeps_the_title_block`, `perfect_kernel_batch77i::unset_store_reads_as_the_title_page_tests_it`; repros `sectioning-frontmatter/class_notes/an_noinst/`, `unset_store_tested_for_{emptiness,undefinedness}_*`); residual (base too): a store that IS handed on reads as empty in the deposit, so ptptex/jpsj2's `\@ifundefined{@recdate}{}{(Received\space\@recdate)}` (jpsj2.cls:740-741) leaves an orphan "(Received )" paragraph beside the `<date role="received">` (0708.3877, 1203.5392) — the label is not taken out with the store as #6a's labels are |
| 15 | mdframed `frametitle=` (options, `\newmdenv` presets, `\mdfsetup`) never typeset | FIXED int27 (mdframed_sty.rs: options run as `\mdfsetup` in the frame's group; the title is the frame's first paragraph in `\mdf@frametitlefont`; ~0.07 % of 2401 papers use it; 2401.03158) |
| 16 | tcolorbox listing title | FIXED 77m: (a) the box options are set as the box sets them and the resolved `title` becomes the listing's title — an unnumbered `ltx:float` with the title as `ltx:caption`, always above the code (the box's title bar ignores listings' `captionpos`: 2409.11413, 2402.13846 set `captionpos=b`) — for `\newtcblisting`, the `\NewTCBListing` family and `\newtcbinputlisting`, whose `[init]` now runs tcolorbox's init processor (`use counter=` → `\thetcbcounter`; no counter → an empty one, 2605.21082), the box's `listing remove caption` (default) drops listings' own caption/title (2409.11413's `\lstset{title=\lstname}`), an absent `O{}` argument is its empty default to the options (neoschool: 40 spurious titles, 141 → 97 errors), and a leading `O{default}`/`o` its default / xparse's no-value marker (tcolorbox_sty.rs `tcb_store_pending_title`, `tcb_xparse_listing`, listings_sty.rs `lst_process_display_with`; OD #768); `\newtcbinputlisting` also keeps a per-use `label=`/`hypertarget=` and follows the box's mode (`listing and text` typesets the file again, `text and listing`, `text only`; the `\examplefile` template: 2405.01480, 2405.02221, 2405.02462, 2405.03285, incgraph-doc gain; 1912.06298, 2105.06630, 2301.07825, 2302.14514, 2410.04867 ship it unused); (b) listings' `nolol` is honoured, so tcolorbox's read-back `\lstinputlisting[nolol]{\jobname.listing}` is no `lol` float captioned "<job>.listing" (KPE #988); also a plain `\lstnewenvironment` after a tcolorbox box (diverted to a file or not) no longer runs its body (the box's `LISTINGS_EXECUTE_BODY` is taken by its own environment). A listings `title=` now sits where `captionpos` puts it (top by default), as Perl places it (alignment/listing.xml: its three titled/named floats re-blessed to Perl's order; the file still differs from Perl's golden elsewhere). Switching to tcolorbox's own definitions was measured and not taken (OD #768: a box `label` → the nearest labelled ancestor, 3 xparse begin-line guards). Residuals (OD #768): `blend into=listings`, `before title=`, `attach title to upper` not captured; `adjusted title`'s `\vphantom` is a hidden span with width; a `text only` listing environment still shows its code; the comment modes' comment not typeset (tcolorbox_sty.rs ~:103-123; sample reach 0); a `text only` `\newtcbinputlisting` drops its `label=` (no element of the output may carry it; reach 0); `listing remove caption=false` with a listings caption loses the box title (listings_sty.rs ~:2519-2526); a leading optional followed by `s`/`t` (`{ O{} s m }` with `*`) reads the star as the mandatory argument (tcolorbox_sty.rs ~:562-564, older; atableau.tex:655/:691 one use, sample reach 0); a listing environment's executed text is not closed off as a paragraph and runs into the next one (older; `\lx@lstenv@body`, review probe rev77m2/y). Witnesses 1912.06298, 2105.06630, 2301.07825, 2302.14514, 2410.04867, 2405.01480, 2405.01576, 2409.11413, 2508.16690 (titles), 2512.18601, 2606.05464, 2606.27251, 2607.12625 (read-back); guards `perfect_kernel_batch77m::*`. A/B (77j3 → 77m, 33 papers + 36 manuals): E/F/W identical except neoschool (141 → 97 E); every word change classified a title gain or a dropped spurious toc caption; r2 (77m → 77m2): E/F/W identical, box titles above the code in 2409.11413 and 2402.13846, `\examplefile` labels kept in the four 2405 papers and incgraph-doc, randintlist-doc random numbers only. Reach: 15 of the 6,000 projection papers and 14 of 8,000 2405 papers use a tcolorbox listing |
| — | tcolorbox box `label=`: the `\ref` names the wrong element or number (0 errors) | **FIXED 78g** (OD #813, KPE #1038; user ruling 2026-10-21 item 12). A label after a raw `\refstepcounter` (one no constructor takes: tcolorbox's in a numbered box's phantom code, tcolorbox.sty:2311, :998, :1154-1161, placed in the frame picture at :1925; a `\put`'s) names the object the step numbers: the kernel records the step's tags (`note_raw_refstep`), the label may stop at the box's `ltx:picture` (`Document::float_to_label_for`; the schema gives `ltx:picture` `labels` and `ltx:tags`) or lands on a listing box's first line, and that object takes the step's number when it has none and is not heading-numbered (`Document::take_raw_refstep_number`); cleveref's `label type` gives the box its type's names; the SVG post-processor keeps a labelled picture. `\newtcbtheorem[number within=section]` `\ref` "1.1" as pdflatex (was the section's "1"), links to the box. A/B 250 (the sel78d census's 150 `tcb_label_ref` papers + 100 random): 8 papers' references changed, all 44 boxed labels with pdflatex's `.aux` number; 0 Error/Fatal changes. Residual: a box whose listing numbers its lines keeps the label on the numbered first line (its line number printed). r2: a listing's start, an amsmath display's rows, a second step in one picture and `\eqno` follow `\@currentlabel` (residuals in OD #813). r4: a box in math labels its display, never an element inside `ltx:Math` (whose id the HTML drops). Witnesses 2505.20088, 2311.09542, 2509.17784, 2602.03411, 2606.06976, 2402.06608, 2506.01072, 2609.20563; guards `perfect_kernel_batch78g::*` |
| 14 | `\refstepcounter` rebinding text | CLOSED 2026-10-18 (measured, no reach): a document's `\refstepcounter` redefinition that typesets text loses that text at sections, equations and captions (they step the counter natively, as Perl's `RefStepCounter`; probe: `[RSC-section]` printed by pdflatex, dropped by both engines); 85 of the 6,000 projection-sample papers redefine `\refstepcounter` and every form is label/anchor machinery that typesets nothing (latex.ltx's body in shipped hyperref copies 47, `\H@refstepcounter`/`\Hy@saved@refstepcounter` 22, a `[label]` optional-argument form 10, `\refsteponlycounter` 12, fancyref's `\FR@loc@refcnt` 2) |
| 22 | an empty `{titlepage}` hiding the document title in HTML (XSLT) | FIXED int27 (OD #761: only a titlepage with ink hides the title) |
| 20 | `\gls` only in display math: entry left out of the glossary | FIXED int27 (glossaries_sty.rs: the location-only reference goes in an `ltx:text` of the `ltx:equation`, after the formula; OD #320 updated) |
| — | pasa.cls (shipped) redefines `\@pushfilename` with the pre-2020 kernel body: "Extra \PopDefaultHookLabel" per package load, every Error of 13 projection-sample papers (~215 corpus) | FIXED 77c (pasa_cls.rs reads the class with the pop of its era, OD #764; 13/13 witnesses → 0 Errors, words unchanged; TL 2025 pdflatex stops on the same) |
| — | shipped classes carrying amsgen's `\@saveprimitive` (asl.cls:327, :717-738; 1005.4193 and 9 more in the 67int29 projection samples): "Unable to properly define \@@overline; primitive \overline no longer primitive", 7 Errors per paper (`\over`, `\atop`, `\above`, the `withdelims` forms, `\overline`) — our `\over`/`\overline` are kernel macros (tex_math.rs:1500, :1764; Perl TeX_Math.pool the same), so `\meaning` is not the primitive, and the ar5iv preload's amsmath binding saves `\@@over`… but not `\@@overline` (amsmath.sty:397) | FIXED 77g (tex_debugging.rs: `\meaning` of the kernel's `\over` family, `\overline`, `\underline` is the primitive's name while LaTeXML owns the definition — Perl's TeX_Debugging.pool exception list extended; KPE #985; removes 7 spurious Errors per paper — 1005.4193, 1803.03483 7 → 0, 1111.1504 8 → 1 — the XML byte-identical on the witnesses (only a document that builds on the unsaved `\@@over` lost content, the minimal trigger); amsmath's binding saves `\@@overline` too, amsmath.sty:397) |
| — | names parted by spacing alone in an ACL `\author` (`Shanshan Wang$^1$~~~~ Derek F. Wong$^1$~~~~ …`, 2603.20851; `~~`, 2605.30274) are one creator, 0 Errors | FIXED 77p (user ruling 2026-10-19, OD #52(o)): a top-level run of two or more spacing commands (`\ `, `\:`, `\>`, `\;`, `~`, `\enspace`, a line-ending `\ `'s control return) between two names is `\quad` — not beside "and", not before a mark or note, not after a line-leading mark; on a mark-led line it parts the entries (2310.15160); a `\let` alias of `\textsuperscript` is the mark (2209.14855); spacing in a mark parts its labels (`\up{~~1}`, `$^{1,2\,*}$`, 2403.08629). Witnesses 2610.20409, 2310.10873, 1702.05821, 1508.00093, 2009.12421, 2603.20851, 2605.30274, 2209.14855 split. The r2 review (226 papers of 2603-2605 with runs in `\author`) found r2 split three people at a run after an initial (2603.17646, 2604.25437, 2603.20914), made creators of address, URL and e-mail lines and of a byline's "By" (2604.10005, 2605.28131, 2603.25922, 2604.12864, 2603.27258, 2604.07186: Donald Robertson and Florian K. Richter lost as authors) and empty ones before icons (2605.12398, 2605.06250); r3 keeps the rule off initials, "By", pieces without a name letter, and lines after the names line unless both sides read as names (marked entries and e-mail rows part by their own readings). The r3 re-review (150 fresh papers of 2606-2608) found "AND"/"And" read as names and a wrapped "and" as an affiliation (2606.27261 `\;\;\textnormal{AND}\;\;`, 2607.03677 `~~ {and}~~`, 2608.24427 `\quad \textnormal{and} \quad`) and institution rows under names made creators (synthetic: `Ann Able~~~~Bob Baker \\ Carnegie Mellon~~~~Johns Hopkins`); r4 reads the word "and" in any case and one wrapper as the separator (`and_word_end`, `is_and_piece`) and joins a line after the names line as a row of names only when it lists another number of names, or a line that is no row of as many entries follows it (`row_of_names`). Guards `perfect_kernel_batch77p::*` |
| — | names parted by `\quad`, `\hspace`/`\hfill` (OD #52(i)) or a spacing run (77p) on one names line: the address lines under it go to the line's last name only (`Yuxi Fu ~~~ Han Zhu \\ BASICS, …`: Han Zhu alone, 1508.00093; 1009.3124, 1202.3179, 1503.08895), where a comma-parted line gives them to every name and pdflatex prints them under the whole row | FIXED 77p r2 (user ruling 2026-10-19, OD #52(o)): the `\quad`-parted blocks join into one group when only horizontal space parts them and the earlier names have no lines of their own, a `\quad` before the line break going on with the next row (2507.06670); guards `perfect_kernel_batch77p::{space_parted_names_share_the_address, space_parted_names_negatives_pin_residuals}` |
| — | preprint.cls numbered institutes: `\institute{Imperial College London, … \and University of Edinburgh, …}` numbered 1, 2 by the class (preprint.cls:235-257, llncs's `\institutename`) and the authors' `$^1$`/`$^2$` picking one each — 2405.16615 gives each author both institutes (the stored `\@institute` annotates every author, `\lx@add@store@affiliations` `annotate=all`; the entries carry no marks to label them) | FIXED 77t (OD #791): a class carrying llncs's `\institutename` (its `\and` stepping a counter) has the `\and` entries of its `\institute` store labelled 1, 2, … as its title page numbers them, when the store has no marks and the queued authors' marks are all numbers (`class_numbered_institutes`); 2405.16615 Imperial College on Ajay Chandra, Edinburgh on Harprit Singh; guard `perfect_kernel_batch77t::class_numbered_institutes_go_by_the_authors_marks` |
| — | aistats renamed copies: ~100 papers ship a renamed aistats style the versioned fallback and the registrations do not reach (`aistats2022_arxiv.sty` 9, `aistats_arxiv.sty` 7, `*_preprint`, `*_mod`, `*_old`, …): no title, no creators | FIXED 77w (OD #731 rule 12, the declaration route): a copy's `\ProvidesPackage{aistats20NN}` routes it to the aistats binding, the copy read after it — 2302.00003 (twocolumn2023.sty), 2402.04821 (aistats2024_cr.sty), 1512.07422 (myStyle.sty) 0 → 6/6/3 creators and a title, 2410.15721 (aistats2025halarxiv.sty) 0 → 1; 1810.02453, 2110.12087 unchanged (already bound); guard `perfect_kernel_batch77w::aistats_renamed_copy_is_aistats` |
| — | renamed natbib copies read raw: a shipped natbib copy under a name the versioned fallback does not reduce to `natbib` (`natbib_vw.sty` 16 papers, `modnatbib` 21, `hbnatbib` 16, `ctextemp_natbib` 13, `natbib_jrm` 12, `NAR-natbib` 9, `natbib_padeu` 8, `rnatbib` 7, `wsnatbib`/`natbibsty`/`igsnatbib` 6 each, … — census: 1,509 corpus papers ship a `*natbib*.sty` (1,677 paper/file rows), `~/data/pk_agents/main/scratch_356a0c47/natbib_copies/natbib_copies_all.tsv`; `natbib2`/`natbib209`/`natbib1`/`natbib_mod` already fall back to the binding, "Interpreted as versioned package"; NJDnatbib/bmvc2k_natbib/ws-rv-har are 72a stand-ins) is read raw: its `\citet`/`\citep` read `\b@<key>`, which no `\bibitem` of ours writes (no `\bibcite`), so every citation prints "?" with a "Citation undefined" Warning, and a labelled `\bibitem` keeps no refnum, so the kernel `\cite` shows no label (1003.3017: 62 Warnings, 0802.4100: 142, astro-ph0512042; pdflatex "Schneider et al. (2010), Smith (2001), (Smith, 2001)."; RED `index-bib/mn2e_natbib_copy/natbib_renamed_copy_citations_resolve`). Not the brief label route of 70c (g): the copy's whole citation layer is missing | FIXED 77w (OD #731 rule 12; user ruling 2026-10-20): a shipped file whose first `\Provides…` names a bound file of its kind loads that binding with the request's options, then is read as its stand-in's copy. Census: 389 of the 1,677 copies declare `natbib` under another name (90 new routes — modnatbib 21, natbib_vw 16, natbib_jrm 11, my_natbib 7, natbibsty 6, … —, 299 the versioned fallback reached before: natbib209 137, natbib2 134, now read as copies); A/B (161 census papers that request their copy, base 529dc1b543): "?" in the XML 2,289 → 16, "Citation undefined" Warnings 3,317 → 0, Errors 59 → 52, Warnings 5,037 → 1,684, words +0.9 %, none worse; 0802.4100 and 1003.3017 resolve every citation. Not routed (they declare their own names, so the rule's exclusion): hbnatbib 16, ctextemp_natbib 13, NAR-natbib 9, natbib_padeu 8, rnatbib 7, wsnatbib/igsnatbib/dmnatbib 6 each, … and the 399 undeclared copies — still read raw (residual). Guards `perfect_kernel_batch77w::{natbib_copy_is_natbib, natbib_renamed_copy_citations_resolve}` |
| — | standard-class derivatives under the declaration route (77w r1 review finding 1, blocker): a shipped class declaring a LaTeX class (aip-cp.cls `\ProvidesClass{article}`, AIP Conference Proceedings, ~540 corpus papers; mf.cls, antonio.cls, HERON.cls/mbook.cls `book`, avh.cls `report`; 17 of the reviewer's 87,937 sampled papers) was read as the article binding's stand-in copy: aip-cp's accumulating `\author[aff1]{…}` kept the last author with no diagnostic (1506.04050 5 → 1, 1604.08112 2 → 1), its `\affil` list was lost, 0709.4467 lost its 10.95pt and 1808.10740 its 12pt nominal size, 1903.06504/nucl-th0703058 a contact | FIXED 77w r2-r3 (user rulings 2026-10-20, OD #731 rule 13): such a copy is an override of the kernel's class, read after the binding as a raw class — its frontmatter setters through the raw-class records (#253 appending `\author`, #482 replay, #265 deposit, K11 stores), its `\newcommand`s of the binding's TEXT names defining them (`\refname`, `\figurename`, `\@ptsize`: a binding macro whose expansion has no control sequence; `state::standard_class_binding_name`, scoped to the standard binding's own load), never its structural commands (`\appendix`, book's `\frontmatter`/`\mainmatter`/`\backmatter`: r2 lost the appendices of 0705.2210, hep-ex0405054, hep-ph0008011, cond-mat0402505, quant-ph0409078) nor its counters' ids (`S1.F1`), everything else as TeX makes it; the 70 rule-13 papers of the r2 re-review's census are each equal to int37a or better (appendix, chapter, creator, contact, abstract, bibitem, figure, equation counts; Errors 38 → 26, Warnings 754 → 700; 5 and 2 creators, "REFERENCES"/"FIGURE" as aip-cp prints them, 11pt/12pt nominal sizes int37a lacked, math0301023's front chapters unnumbered as its PDF); guards `perfect_kernel_batch77w::{standard_class_derivative_keeps_every_author, standard_class_derivative_patches_stand, standard_class_derivative_keeps_book_structure, standard_class_derivative_defines_only_its_class_names, standard_class_derivative_beside_a_self_named_article}`; residuals: a chapter after book's `\backmatter` stays numbered (the book binding's `\backmatter` is Perl's no-op, book.cls.ltxml:121, as under `\documentclass{book}`; pdflatex prints "Afterword" unnumbered; follow-up batch: book.cls's `\@mainmatterfalse`, its own A/B); the size commands do not yield (binding primitives): a copy with its own `\small`… beside a 12pt `\normalsize` keeps the binding's relative sizes (`\small` 75 %, pdflatex 91.7 %; synthetic, rev77w3 probe5; no corpus paper); a copy's `\newcounter{figure}[subsection]` finer than the binding's id scope gives suffixed ids (`S1.F1a`; rev77w3 probe6; no corpus paper) |
| — | a routed copy's text overrides of names a binding it requires owns are dropped: 1402.7103's proto.cls (→ aastex): its `\newcommand\apj{\ref@jnl{\it Astrophys.\ J.}}` (:1731) kept the aas_macros binding's "ApJ" | **DONE 78h** (ruled 2026-10-21, refined the same day: the binding sets the mechanism, with a reasonable default): a copy routed to a binding supplies only the text names its binding declares — the kernel's `state::COPY_TEXT_NAMES` (`\abstractname`, `\appendixname`, `\bibname`, `\chaptername`, `\contentsname`, `\figurename`, `\indexname`, `\listfigurename`, `\listtablename`, `\partname`, `\proofname`, `\refname`, `\tablename`) and the names a binding declares (`state::declare_copy_text_names`: aas_macros' journal abbreviations, IEEEtran's `\IEEEkeywordsname`/`\IEEEproofname`; global, not tied to the copy's binding) —, when its definition reads as text (fails closed, r3: every macro and robust `\protect\X` followed; only characters, conditionals, printing and font primitives, the space primitives and LaTeX's font selection commands read as text; a constructor, `\par`, `\csname`, a macro whose expansion is code, any other primitive refused, Info naming the token; guard `routed_copy_text_name_check_fails_closed`); rule 13's derivatives `\newcommand` the same list; everything else (caption formats and separator, `\@thmcountersep`, list labels, counters, structure) stays the binding's (OD #731 rules 13-14; guards `perfect_kernel_batch78h::{routed_copy_supplies_only_declared_text_names, self_named_copy_supplies_no_text_names, standard_class_derivative_newcommands_only_declared_text_names}`). A/B 542 papers (every routed copy incl. the shapes r1's scan missed, 69 derivatives, 34 witnesses, 100 controls): E/W/F 300/5,124/0 on int47a, r1 and r2; 15 outputs changed vs int47a, each a declared name as the copy writes it; of r1's 88 changes, 73 back to int47a. 1402.7103's "Fig. 1.—" is its `\fnum@figure`, a caption format: the binding's "Figure 1: " (int47a's; r1's gain withdrawn) |
| — | llncs `\labelitemi`/`\labelitemii`: llncs.cls:521-522 sets a bold "–" at the first level and "•" at the second; the llncs binding (and Perl llncs.cls.ltxml) kept article's "•" then "–" — every llncs paper (2607.29279's waica.cls copy repeats the definition) | **DONE 78a** (KNOWN_PERL_ERRORS #1017; guard `perfect_kernel_batch78a::llncs_item_labels_are_the_class_s`) |
| — | jheppub's `\emailAdd` gives the n-th address to the n-th author (Perl jheppub.sty.ltxml), while jheppub.sty's `\maketitle` prints one shared "E-mail:" line, as jinstpub's — whose binding now places each address on the author it spells (`\lx@add@contact@placed`, 78e r2; 16 of 23 jinstpub papers had a wrong owner under the n-th rule, e.g. 2311.16063's bross@fnal.gov on Bersani); jheppub very likely misattributes the same way (~986 papers in 8 months) | **FIXED 78f** (OXIDIZED_DESIGN_DIVERGENCES #805, KNOWN_PERL_ERRORS #1030; measured first: placing by the spelling alone would have put 40 of the 150-paper jheppub sample's addresses that the list's order gives right — `zhl61@` for Zhen Liu, 1303.3040 — on the author block): an address goes to the author it spells, else to the n-th author when the list is one place for each author in their order (the consistent list is evidence: user ruling 2026-10-21), else to the group's only author, else — the one address left on the author block of a multi-author paper — to the ONE author marked as the corresponding author (a "Corresponding author" note, or a mark whose legend says so; user ruling 2026-10-21, OD #809: 1901.08773, 2510.11030, 2211.07892; 1802.03677's note number two authors carry → the block; 2308.06532 by its address's "(Corresponding author)" and its one legend-less `*`), else the author block (`\lx@add@contact@placed[owner=sequence]`, `sequenced_contact_labels`); jcappub and jinstpub ride the binding; `\emailAdd{a, b}` an address each. A/B: jheppub addresses on another spelled author 7 → 0 (150 papers), jcappub 15 → 0 (83), jinstpub unchanged (23); guards `perfect_kernel_batch78f::{jheppub_emailadd_in_author_order, jheppub_emailadd_one_call_lists_the_authors, jheppub_emailadd_out_of_order_by_spelling}`. Residual: a name letter the keys drop (`\l`: 2212.09776's inalecz@) or a spelling they do not read leaves an address on the block |
| — | aa 2401.16957: a comma before the first `\inst` (`\author{Clara Dehman,\n\inst{1}\fnmsep…\inst{4}\thanks{…} Mario Centelles,\inst{5}…}`) makes the splitter give Clara's four affiliations and her e-mail note to Mario Centelles and lose his institutes 5-6 — wrong with 0 errors (as int45b; rev78e2 side finding) | **FIXED 78f** (OXIDIZED_DESIGN_DIVERGENCES #807, KNOWN_PERL_ERRORS #1032): an `\inst` after the comma (with or without the space; llncs.cls:894 / aa.cls:759 `\unskip`) and a `\fnmsep` between marks are the name's before the comma (`marks_before_glued_commas`, `leading_inst_marks_end`): 2401.16957's Clara Dehman keeps her four institutes and note, Mario Centelles his 5-6; 15 census papers write a comma before `\inst`, 7 changed, each as its PDF; guard `perfect_kernel_batch78f::llncs_comma_before_inst_is_the_name_before` |
| — | jfm copies' equation numbers: a jfm.cls copy's `\theequation{\thesection.\@arabic\c@equation}` and `\@addtoreset{equation}{section}` (JFM-FLM_Au.cls:1317-1318) keep the OmniBus base's "(1)" — "Counter equation keeps LaTeXML's numbering" (2507.06135, PDF (2.1), (2.14a), (A 1); a `\documentclass{jfm}` without a copy the same; review finding 3; RED `stand-in-copies/provides_route/copy_numbers_equations_by_section`) | **FIXED 78q** by the binding, charter §5 (no copy override): jfm_cls.rs numbers the equations as every jfm.cls version does, "(2.1)", the appendices "(A 1)", "(0.1)" before the first section, with or without a copy (2608.04345: 116 labels, now as its PDF; KNOWN_PERL_ERRORS #1097; guard `perfect_kernel_batch78q::jfm_equations_within_sections`). History: needs-a-ruling (77w r2): letting a counter the binding's base class (OmniBus, its article) numbered yield to the copy, as rule 2 lets the pool's article defaults yield to a package's copy (`stand_in_base_defaults` in `copy_provides_keeps_counter`), fixes 2507.06135 but changed 258 of 905 A/B papers: 198 ids only (`S3.F1` → `F1`: the copy's `\newcounter{figure}` declared for real), 54 tags — the copy's own forms, mostly as its PDF (aastex701 copies "TABLE I", jheppub-like (1.1)), some not ("(i)." with the binding's `\labelenumi` period, 2206.08798's "1 " with a trailing space) — and 31 other; reverted, no jfm-only patch |
| — | X2/OT2 Cyrillic text symbols dropped with no diagnostic: `\usepackage[X2,OT2,T1]{fontenc}` + `A {\fontencoding{X2}\selectfont \cyrje\cyrii\CYRA} B {\fontencoding{OT2}\selectfont \cyrje\CYRJE} C.` — pdflatex "A јіА B jJ C." (OT2 sets Ј ј in the J j slots, ot2enc.def:96,124), we and Perl "A А B  C." with 0 Errors/Warnings: there is no x2/ot2 fontmap, so the slots decode to nothing and `\DeclareTextSymbol` gives no glyph (T2A/T2B/T2C print since 77s, OD #776). `fonts-nfss/x2_ot2_cyrillic_symbols_print` (77s r2 re-review probe x2) | FIXED 77u (x2_fontmap.rs, ot2_fontmap.rs, OD #777: "A јіА B јЈ C."; a `\DeclareTextAccent` of a command another encoding declared first defines this encoding's too, so OT2's `\U{i}` is й). A text symbol of an encoding that still has no map now warns where it is dropped, naming the encoding (tipa's T3: residual row below) |
| — | tipa's T3 has no font map: its named symbols (`\textschwa`, `\textltailn` …, t3enc.def) print nothing — since 77u with one Warning per symbol ("Font encoding T3 has no font map …"), 2405.01972's `\textipa{\textltailn}`; the ASCII IPA shorthand inside `\textipa{…}` decodes as OT1 letters (pdflatex ə for `@`). A T3 map (tipa's fonts: tipa.sty, t3enc.def slots) closes both | FIXED 77x (t3_fontmap.rs, t3enc_def.rs, tone_sty.rs; OD #781): the input letters and symbols are the tipa fonts' IPA letters (`\textipa{[D@"lIm@]}` [ðəˈlɪmə], 2405.01972's ɲ ɓ ɡɔ), the diacritics combining marks (`\textsubring{n}` n̥, `\~*{a}` a̰), `\tone{51}` ˥˩; the glyphs Unicode lacks (hooktop, tone-letter parts, Viby letters, `\textoverw`) warn |
| — | math: a NUMBER before a negative thin space is dropped from the parsed formula — `$2\!x$` text "x", `$2\!\cdot 3$` "absent cdot 3", `$+5.9\!\cdot\!10^{-7}$` without the 5.9, `\mathbf 1\!\left(` without the 1 (Perl "2 * x", "2 cdot 3"): base_xmath.rs's number ligature took the `\!` hint as a number separator (its `NAMED_SPACE_CHARS` entry is "", which Perl's `($s = $space_chars{$s})`, Base_XMath.pool.ltxml:513-516, reads as false and stops), making a two-node "number" of the NUMBER and the hint, which the parser dropped with the hint. Every document with `\!` after a number (int36a too); surfaced by 77x r2's A/B, where tipa then amsmath makes `\!` the spacing command again (int36a had it `\relax`): 2609.36534 91 numbers, 2608.02769, 2603.03017 `(1 \! - \! f)` | FIXED 77x r2 (the empty space char ends the scan; `math-parse/number_before_negative_thin_space_kept`, guard `perfect_kernel_batch77x::number_before_negative_thin_space_kept`) |
| — | eqnarray label with no numbered row after it (pre-existing; review rev78n2): a `\label` in a trailing `\nonumber` row or in an `eqnarray*` stays on its unnumbered row, and its `\ref` prints a wrong number with no diagnostic — the group's (5 for pdflatex's 6), a section's (iopart: "1" for 2), or a raw id ("S0.Ex5" for 7); LaTeX's `\ref` names the counter's next value. Fix: give such a label a refnum-only tag holding the counter's next value (OD #847). Witness 1303.3431 `appa3`; probes `/home/deyan/.claude/jobs/356a0c47/tmp/rev78n2/p/q1.tex`, `iop/q8.tex` (copy into a repro). | **FIXED 78r** (OD #847, KPE #1106): the label's row keeps the next number's reference forms (refnum, typerefnum …, no printed tag) and stands as its own equation; guard `perfect_kernel_batch78r::eqnarray_label_with_no_numbered_row_after_it_names_the_next_number` |
| — | `\nonumber` inside `{equation}` removes the number (Perl's extension; LaTeX without amsmath still prints it): hep-ph9710521 (`\nl` inside `\beq`) has every number from (3.22) on one behind its PDF, 45/56 matching (78n r3 report). | **FIXED 78r** (OD #865): a no-op in LaTeX's `{equation}` unless amsmath is loaded; guards `perfect_kernel_batch78r::{equation_nonumber_without_amsmath_keeps_the_number, equation_nonumber_with_amsmath_removes_the_number}` |
| — | An `.eps` the source names but ships only as `.pdf` (the author ran epstopdf) has no size at digestion: `image_candidates` (`util/image.rs`, Perl Image.pm:43) searches `x.eps.*`/`x.eps` only, so the graphic gets no candidates, no `imagewidth`/`imageheight`, and a sub-figure panel measures 0 wide (it then takes a row of its own: 1408.2843 Fig 1, 1503.04804 Fig 3). The HTML is right — post drops the extension and finds `x.pdf` (Perl's post does not, KPE #1137). Fix: when a path with a known graphics extension finds nothing, retry its base name with any graphics type (post's rule), so the sizer reads the real file. Queued in 78v (user 2026-10-22) | FIXED 78v (`image_candidates` retries the base name under any graphics type, OD #882; 97 census papers; guard `perfect_kernel_batch78v::includegraphics_eps_shipped_only_as_pdf_is_sized`; 1408.2843 Fig 1's two panels sized and in one row as in the PDF) |
| — | **Panel grid rule** (grid design study 2026-10-22's payload 2, `agent_reports/2026-10-22_grid_design.md` Q3; FIDELITY_CHARTER §2 "Float panels"; user rulings 2026-10-22). `arrange_panels` (latex_constructs/mod.rs) keeps Perl's greedy rule (latex_constructs.pool.ltxml:3229-3349) fed by what the source states: an explicit break always breaks — the source's `\par` (not the one leaving horizontal mode inserts, 2605.00347), a `{center}`-like boundary, `\\` in its capture, `\vskip` (not `\vspace`/`\medskip`), a forced `\linebreak`, recorded where constructed (`note_panel_separator`); the glue stated between panels (`\hskip`, `\quad`, `\hspace` as `\lx@panel@hskip`, absorbed as the kernel's box) counts, plain spaces, labels and `\hfill` 0, an uncaptioned sub-figure its body's material, a row TeX sets in one `\hbox` one row (`note_panel_hbox`); the line is the class's real one for rows only (`install_panel_line`/`panel_page`: a class binding's declared page, `declare_panel_page` — IEEEtran, REVTeX, AASTeX 6.x, mnras, aa, their one-column options and regions — else `\textwidth`, the geometry's `\Gm@tw` while `\textwidth` is `\Gm@doctw`, `\if@twocolumn`'s column), each panel's stated width (`declare_panel_width`: kept as tokens in the float, `KEEP_STATED_LENGTHS`) evaluated again at that geometry at the float's end inside its group (`after_float` → `note_stated_widths`, the body the box list being built: a width macro the float defines or a later `\renewcommand` changes is read as it stands there; review rev78x2 finding 1), in the ratio form (the width the construct gave its box times `real/ours − 1`; a `\scalebox`/`\resizebox` of no width of its own scales its content's, a rotation turns it: finding 2), a length register read through what its `\setlength` stated (`note_setlength`); a width the float cannot evaluate — a register the document assigned otherwise, a macro of a group that has ended or shadowed, a stated length that does not give its box's value again (within 4sp), a box's length whose tokens were not kept (not one a binding gave and withheld, subcaption's `\subfloat`: `STATED_LENGTHS_WITHHELD`, review rev78x4 finding 3) — leaving the float arranged exactly as before the rule, at its `\hsize` or the geometry's `\Gm@tw` (OD #99; review rev78x3 findings 1-2: never worse than int68a), a parbox's content evaluated against its width and an `\hbox`'s against the enclosing line (`declare_panel_parbox_width`); a row that begins a paragraph keeps the glue stated before its first panel (after a justified `\\` only an `\hspace*`'s: 2212.03198; a centred `\\` — `\centering`, `\raggedright`, `{center}`, REVTeX's floats, `declare_panel_floats_centred` — is `\@centercr` and begins one: finding 3; AASTeX's and emulateapj's floats are not centred, `withdraw_panel_floats_centred`, and a `\newline` is `\@normalcr` everywhere, `PANEL_NORMALCR`: review rev78x4 findings 1-2), a sub-float body's stated glue counts towards its width (subfigure.sty; 1307.3388 Fig 3), `\hspace` counts in a `{center}` as `\hskip` does (finding 6), the `\hbox` row keyed on the group that set the box's mode (finding 3), a wrapfigure's `\hsize` its own line (2609.34643), and the declared pages corrected (IEEEtran compsoc `conference`, draft options on A4/cspaper, REVTeX `[preprint,twocolumn]` 229/468pt: finding 4); the registers and image sizes unchanged. Oracle set (453 floats, rows as the PDF): int61a 300, 78t r3 344, int65a 354, 78x r1 410, r2 428, 78x r3 429, 78x r4 429, **78x r5 429** (shapes 336 → 399 → 420 → 422 → 422 → 422; r4's rows identical to r3's on all 453, r5's to r4's). A/B int69b → 78x r5, one binary (`latexml_oxide.78x-r5-rel`), the same 1,907 papers: E/F/W equal, every fingerprint identical, the same 750 changed floats in 313 papers and the same 19 worse in 13 (r3's list); the float-free papers identical; r4 → r5 one paper byte-changed, no top-level float's rows: 2504.20901 A1.F8's minipage of two `\subfloat`s separated by `\\` is two rows again (r3's and the PDF's; r4 one, the float unevaluable: review rev78x4 finding 3). A/B int69b → 78x r4 (rebased on int69), one binary, 1,907 papers: E 6,680 = 6,680, F 20 = 20, W 59,351 = 59,351, every fingerprint identical; 328 papers byte-changed; 750 floats in 313 papers change rows, against the PDF 424 better, 19 worse in 13 papers (r3's list, all in r2's residual classes, none new), 25 both wrong, 66 the same count, 216 unverifiable; the 162 float-free papers identical; r3 → r4 no float's rows changed in the population (the r4 shapes are the review's probes'; an interim r4 split 1307.3388 Fig 3 2+1+1 once a centred `\\` kept its row's leading glue, until a sub-float body's stated glue counted: 2+2 as the PDF). A/B int68a → 78x r3, one binary (1,907 papers, census pop78t + 300 seeded controls): E 6,680 = 6,680, F 20 = 20, W 59,351 = 59,351, words, bibitems, tabulars and tex/font fingerprints identical; 329 papers byte-changed: 750 floats in 313 papers change rows, against the PDF (pdflatex box dumps) 424 better, 19 worse in 13 papers (all the residual rows below: 0809.3275 ×4, 2208.08007 ×2, 2008.06630 ×2, 1911.11519, 2502.05562, 2109.08525, 2310.04470, 1108.5543, 1712.01536, 2009.04593, 2406.05437; and the scorer's 2310.05307 ×2, 2609.14985, the PDF row counts image panels only), 25 both wrong, 66 the same count, 216 unverifiable (173 no PDF, 28 PDF without image panels, 15 unaligned); the 16 others change only nested sub-float rows, a break before a caption (1109.3733 as the PDF; 1906.08196, a pdflscape page) or a zero-width row start (1705.04732, the picture alone in its `{center}`'s row; 2012.03147, an `ERROR` after a stated break no panel, Perl's rule latex_constructs.pool.ltxml:3331-3333); 1010.0060 Fig 5 has the PDF's two rows (4+1 for 3+2, a sub-figure body's spaces), 2202.03884's settled row S4.SS1.fig1 and 2403.19113 as int68a; the 162 papers without a float identical. Cost: instructions:u +0.07..+0.14 % on two papers (noise 0.07 %). Witnesses 0706.4410 Figs 3, 5, 1110.0084 Figs 1, 27, 29, 1107.3402 Fig 5, 1206.5015 Fig 3, 1007.2355 Figs 1, 3, 1705.10798, 2601.01508, 0912.4528 Fig 4, 2605.03152 S3.F2, 2212.03198, 2609.34643, cond-mat0610021 Fig 11 (one overfull row per sub-figure as the PDF since the 78z EPS sizes, `note_panel_hbox`); r2's in OD #885. Residuals: the rows below, 2301.02117's negative-glue overfill (accepted, the charter), TeX's line breaking (RED `a_justified_row_keeps_its_overfull_panel`, by the charter), a plain `\hskip` (at the registers' geometry), and the evaluation's limits (a width macro defined in a group inside the float arranged as before the rule, not as the PDF; a register set in a region of another column setting, `keepaspectratio` with both keys, `\resizebox{!}{h}`, a pdflscape page's line; panels TeX cannot break between set in one overfull line where the rule breaks, `p7.tex` F2; OD #885) | FIXED (78x r1-r6, OD #885; guards `perfect_kernel_batch78x::*`, 34; repro `par_breaks_a_panel_row` GREEN; r6: a row separator is a mark, not a `{center}` capture's content — `insert_block_as` chooses the container without it, 1710.06503 Fig 1 0 Errors again, the ws-procs961x669 net's int70a regression; repro `captions-floats/an_hspace_star_opens_the_row_paragraph` GREEN) |
| — | An EPS graphic's natural or `scale=` size comes out at 72.27/100 of TeX's: `image_graphicx_sizer` (`latexml_core/src/util/image.rs:799`; Perl Util/Image.pm:271, KNOWN_PERL_ERRORS #1141 `$w * 72.27 / $dpi`) reads the BoundingBox size as pixels at 100 DPI where TeX reads bp (`\includegraphics[scale=0.565]{x.eps}`: latex 260.88pt, ours 187.90pt); since 78t sizes `\psfig` by that sizer it reaches the panel rows (1106.2745 Figs 1-2, 7, 8, 10; OD #876). The digestion-time box shows it too: `\sbox0{\includegraphics{e.eps}}` (40bp) is 28.908pt where pdflatex gives 40.14989pt (Perl 0pt). And among a name's candidates the sizer reads the first raster-readable one, so beside `fig.pdf` a `fig.ps` sizes the box of an extensionless `\includegraphics{fig}` where pdflatex reads the `.pdf` (78w side finding). The natural `cssstyle` is right (read as bp). Direction: the bp → pt reading the PDF branch uses, and the first readable candidate whatever its type, as `natural_display_size_pt_of_candidates` does. RED repro `package-bindings/includegraphics_eps_box_is_its_bounding_box` | FIXED 78z (RULED 2026-10-22: fix to TeX's bp). `image_graphicx_sizer` reads the first candidate whose size it can read, whatever its type (`read_image_size`: `ImageSize::Pixels` / `ImageSize::PostScriptBp`, then `natural_size_pt`), and sizes a vector one (EPS/PS, PDF, SVG) in pt by the graphicx algebra (`graphicx_box_pt`); a raster keeps Perl's DPI rule (OD #337). An EPS's size is its `%%BoundingBox` (graphics.sty:52 `\Gread@BBox`, epstopdf), the `%%HiResBoundingBox` only without one (it came first, rounded: 199bp for a 200bp box; post reads it so already), for the natural `cssstyle` too. 40bp = 40.15pt as latex (pdflatex 40.14989pt); 1106.2745's `\psfig` panels at their BoundingBox. OD #886. Guards `perfect_kernel_batch78z::{includegraphics_eps_box_is_its_bounding_box, psfig_natural_panels_are_as_wide_as_their_bounding_box}` (repros package-bindings/includegraphics_eps_box_is_its_bounding_box, captions-floats/psfig_natural_panels_are_as_wide_as_their_bounding_box), `util::image` unit matrix. Residuals: graphicx's `bb=`/`bbllx=`… and `natwidth=` keys and the `hiresbb` key/option are not read by the sizer (Perl alike); a raster without a stated resolution is 100 DPI where pdfTeX takes 72 (Perl's rule, OD #337); the box is TeX's bp to the sp, where pdfTeX's image arithmetic lands a few sp under it (40.14989pt) and graphics.sty's `\Gscale@div` off an explicit size (`width=100pt`: latex 50.00186pt, ours 50.0pt, ~120sp; `height=1in`: latex 144.5733pt, ours 144.53998pt, ~0.033pt ≈ 2,200sp) |
| — | The natural `cssstyle` and the box read different candidates when a raster comes before a vector: `image_graphicx_sizer` stops at the first candidate it can read whatever its type (78z, OD #886, Perl's rule), but `natural_display_size_pt_of_candidates` (`latexml_core/src/util/image.rs:1293-1302`) skips a raster to the first vector. Extensionless `\includegraphics{q}` with `q.png` (200 px) and `q.eps` (40 bp): candidates `q.png,q.eps`, box 144.54pt (the png at 100 DPI), `cssstyle` `width:4.015em` (the eps's 40.15pt); post renders `q.png`, so the browser shows it ~3.6× smaller than pdflatex (200.75pt). int67a the same (pre-existing). Fix: stop at the first candidate any reader sizes; a raster there → `None` (no em sizing). Needs its own A/B (papers shipping a raster and a vector of one name). Review rev78z finding 1, probe `rev78z/probe2/` | FIXED 79a: `first_natural_size` (util/image.rs) is the one candidate both read, a raster there no `em` size; the candidates also follow `\Gin@extensions` (2107.02597), seeded as pdftex.def sets it and edited as in TeX — `\DeclareGraphicsExtensions`, grfext, epstopdf's `.eps` (r2, review rev79a finding 3); OD #886; guards `perfect_kernel_batch79a::{natural_cssstyle_reads_the_candidate_the_box_is_sized_from, declared_graphics_extensions_order_the_candidates, append_graphics_extensions_keeps_the_driver_order, prepend_graphics_extensions_puts_the_type_first, epstopdf_appends_eps_last, declared_graphics_extensions_fold_case, epstopdf_prepend_puts_eps_first}` (r3: extensions matched in any case, as kpathsea; epstopdf's `prepend`) |
| — | A `\\` between sub-figures inside `{center}` is lost (`aligning_environment` → `insert_block_as`: the floats leave the paragraph that held the `ltx:break`), so the PDF's rows are not kept: 0912.4528 Fig 4 (`\subfigure…\subfigure} \\ \subfigure…`: pdflatex 2×2, ours 3+1, int60a one row), 1206.5015 Fig 3 (6/6/5 in the PDF, ours 8/8/1); `\centering` keeps it (`\begin{figure}\centering A B \\ C D`: ours 2×2) | FIXED (78x, OD #885: a `\\` in a `{center}`'s capture directly in a float is a row break; 0912.4528 Fig 4 2+2, 1206.5015 Fig 3 6/6/5; guard `perfect_kernel_batch78x::a_center_and_its_breaks_end_a_panel_row`) |
| — | Page geometry under the panel rows: a two-column document's `figure*` holds panels sized by `\columnwidth`, which is our `\textwidth` (latex_constructs.pool.ltxml:321 `AssignRegister('\columnwidth' => $textwidth)` at `\begin{document}`, Perl the same; and our two-column `\textwidth` is the one-column 345pt), so `\subfigure{…width=0.74\columnwidth…}` pairs in a `figure*` break where the PDF sets one row (1707.00190 Fig 2, nucl-ex0511052 Fig 13, 2502.05562 Fig 6, astro-ph0205132 Figs 2, 4, 5); REVTeX 3 (`\documentstyle{revtex}`) keeps 345pt where its preprint text is 6.5in (cond-mat9707225 Figs 1, 4, 5, 8, 9: `\epsfig` pairs of `0.46\textwidth` under a mid-document `\setlength{\textwidth}{17cm}`, one per row, the PDF 3×2; exposed by 78t r3 sizing `\psfig`). Needs a ruling on the web page's geometry (a starred float's row width in a two-column document) | FIXED (78x, OD #885; user ruling 2026-10-22: rows against the class's real line, the `\columnwidth` register Perl's; a two-column `figure*`'s `\columnwidth` panels share a row; guard `perfect_kernel_batch78x::two_column_floats_set_rows_against_the_column`). Since 78x r2 a class binding declares its real page (`declare_panel_page`: IEEEtran, REVTeX, AASTeX 6.x, mnras, aa); the others' residual is the "Class pages not declared" row |
| — | Panic: 2509.13772 "range end index 44 out of range for slice of length 43" (int54a and 78j r5 the same; outside 78j's paths; found by the 78j r5 A/B) — root-cause and fix (panic → Error with the reason) | FIXED 78v (an entry's trailing separator is trimmed once its address runs are placed, affiliations.rs; guard `perfect_kernel_batch78v::affiliation_address_run_after_a_trailing_comma`) |
| — | A footnote mark `[\numexpr\value{footnote}-1\relax]` (the `\relax` form, the common one) or `[ \numexpr…]` (a leading space) is not read as a number: 2 "Missing number" Warnings, the mark lost, the text unpaired, where pdflatex gives 0 errors and "A1 B21 C." (`latex_constructs/mod.rs:698-710` evaluates `\the\numexpr…\relax\relax` → "1\relax"; the space stops the arm). Fix: skip leading spaces, drop trailing unexpandable `\relax` before the digit test. Review rev78u finding 1 | FIXED 78v (`footnote_number_digits`; guard `perfect_kernel_batch78v::footnote_mark_numexpr_relax_and_space_are_numbers`) |
| — | Author marks: `\textsuperscript{1\protect\thanks{…}}` keeps the note but drops the label (the left-behind label is `1\protect`; pdflatex "Ann Alpha1∗"), and a macro whose body prints text plus a note (`\newcommand\fnA{1\footnotemark}`) is moved out whole, its "1" landing in the name (`author_block/preprocess.rs:84, :140`; synthetic, no census paper). Fix: carry a `\protect` before a note command out with it; a macro counts as a note only when its body is note commands alone. Review rev78u findings 3-4 | FIXED 78v (`split_mark_notes`, `makes_a_note`, `prints_beside_a_note`; guard `perfect_kernel_batch78v::author_protect_thanks_and_note_macro_inside_a_mark`; r2: a macro is read in place only when its body is a label beside notes, any other taken out whole as before, review rev78v finding 1; r3: a `\relax` or a `\protect` beside the label, other than a `\protect` right before a note command, and a macro after a `\protect`, take the macro out whole as int63a, review rev78v2 finding 1) |
| — | caption: a ruled/boxed float.sty float resets the label format and separator to its style's (caption.sty:864-866 `\caption@setstyle*\float@style`), so pdflatex prints "Algorithm 1 Two." under a global `labelformat=allcaps` where ours prints "ALGORITHM 1" (2106.08363 wiley-article; boxed format "[Listing] 1", 2109.00138); untyped settings should give way to the style's, typed `\captionsetup[algorithm]` still apply (LIPIcs). Review rev78s2 finding 2 | FIXED 78v (a float.sty type's style recorded, `\DeclareCaptionStyle` a caption style, caption.sty's `ruled` declared unless its `ruled` option; `caption_setting`, `float_style_setting`; guards `perfect_kernel_batch78v::{caption_ruled_float_sty_float_takes_its_style, caption_declared_ruled_style_formats_a_ruled_float}`; residual resolved by 78y r2's `float_sty_caption` label-font reset, caption.sty:860-861: under `[ruled,labelfont=it]{caption}` a ruled and a boxed float's labels are italic only, pdflatex's CMTI10 alone, re-measured on int67a 78z, review rev78y2 finding 4; guard `perfect_kernel_batch78z::caption_ruled_option_ruled_float_takes_caption_label_font`) |
| — | caption: `\captionsetup[<type>]{name=…}` of a float.sty type does not reach `\fname@<type>` (caption.sty:882-884 calls `\floatname`; `caption_sty.rs:241` `\lx@caption@setname` sets only `\<type>name`): pdflatex "Snip 1:", ours "Listing 1" (int58a "Snip 1" for the old figure-typed listing; algorithm "Proc 1" vs "Algorithm 1" in base too). No census paper sets a non-default name. Fix: `\def\lx@caption@setname#1#2{\@ifundefined{fname@#1}{}{\@namedef{fname@#1}{#2}}\@namedef{#1name}{#2}}`; the fallback's `\ifcsname #2name` should test `\ifx…\relax` as caption does. Review rev78s2 finding 3 | FIXED 78v (`\lx@caption@setname` calls `\floatname`, `\lx@caption@fnum` tests `\ifx…\relax`; guard `perfect_kernel_batch78v::caption_typed_name_of_a_float_sty_type`) |
| — | newfloat `\SetupFloatingEnvironment` has Perl's `OptionalKeyVals {}` signature (`newfloat_sty.rs:5`), not newfloat.sty:304's `{type}{options}`, so `\SetupFloatingEnvironment{listing}{name=Prog}` prints "name=Prog" and the name is unchanged; minted's no-op copy (`minted_sty.rs:197`) protects minted papers today — do not just drop it. 6 census papers (2306.12224, 2405.15425, 2411.16612 "List.", 2507.09840, 2510.09447 "Listagem"). Review rev78s2 finding 4 | FIXED 78v (newfloat.sty:304's signature and keys, KPE #1138; minted's no-op only where newfloat is not loaded; guard `perfect_kernel_batch78v::newfloat_setup_floating_environment_sets_its_options`) |
| — | A class's caption separator set outside caption's settings (acmart's `labelsep=period` via ams_support's `\format@title@figure`; IEEEtran's "Fig. 1.") does not reach newly declared float types: listings close with ": " where the PDF prints "Listing 1." (1807.03757; IEEEtran probe, figures wrong in base too); 23 of 125 minted papers are acmart. A rule, not per class: a new float type takes the class's figure title format. Review rev78s2 finding 5 | FIXED 78y (RULED 2026-10-22: port both; r2 RULED: names and separators only, the class caption fonts behind a knob, off by default). acmart_cls.rs issues acmart.cls:952-984's `\captionsetup` by format as names and separators (the journal flags of :214-254 ported with it): journals "Fig. 1. …", sigconf/acmengage/sigchi/siggraph "Figure 1: …", sigplan "Figure 1. …" — figure, table, newfloat and float.sty types alike (1807.03757's minted `listing` "Listing 1."). The rule: a class's `\@makecaption` is `\lx@makecaption` (latex_constructs_rust_only.rs), which the figure's and table's formats, newfloat's types and listings' captions take (the AMS classes' ". ", from their class bindings, not from ams_support: article+amsaddr keeps "Figure 1: "; IEEEtran's "Fig. 1." and "TABLE I" over its text by mode; ieeeaccess/ieeeoj/ieeetj "FIGURE 1." / "TABLE 1.", arabic tables, ieeeaccess Roman under `jtehm`); a `\restylefloat`ed figure or table keeps float.sty's (2006.08364); caption replaces it with its own (`\lx@caption@makecaption`, its default separator per class: an AMS class's ". ", caption-ams-smf.sto) and formats float.sty types in it, in caption's fonts; float.sty's own styles and longtable's `\LT@makecaption` keep their ": "; algorithm2e makes its own, "Algorithm 1: " (`\algocf@capseparator`), unless float.sty made the type (algorithm.sty beside `[algo2e]`: 2204.02306, 1812.03871, 2408.08621). The class caption fonts (acmart, IEEEtran, jfm, webofc, jinstpub) under caption_sty.rs `CLASS_CAPTION_FONTS` alone: latexml.sty's `classcaptionfonts` option, under ar5iv `--preload=[classcaptionfonts]ar5iv.sty` (78z: ar5iv.sty passes it on with its own options; a second latexml.sty preload beside ar5iv.sty's sets no knob or loses ar5iv's options, the repeat-load row below). OD #884, KPE #1140. Guards `perfect_kernel_batch78y::*` (37; the knob on in `*_with_class_caption_fonts`; repros class-bindings/{acmart,ieeetran}_*_caption_format, amsart_*, amsaddr_*, ieee{access,oj,tj}_caption_names, ieeetran_restylefloat_table_keeps_float_sty_format, algorithm2e_caption_closes_with_its_separator, algorithm_sty_beside_algo2e_caption_is_caption_s, ieeetran_caption_yields_to_caption, ieeetran_listings_and_longtable_captions*, acmart_algorithm2e_caption_is_not_caption_s). Residuals: IEEEtran's `captionsoff` (captions kept) and caption sizes; ieeetaes/ieeeaerospace/ieeeojcsys/smc_ieeeconf take IEEEtran's format unverified (no copy in the census); acmcp untested (pdflatex needs its logo); older acmart copies (v1.55-era) set sigchi's caption fonts differently (fonts only: behind the knob, not ported per version); a `\restylefloat`ed figure in a ruled style keeps the kernel's ": " where float.sty prints "Fig. 1 text" (float_sty.rs records the style only; pre-existing); float.sty's plain-style text after a stray space without caption (Perl float.sty.ltxml:52; pre-existing) |
| — | epsf via `\input epsf.sty` reads only epsf.tex for the reset scope (`\@currname` unset): 10 papers, 5 shipping a local 1991 copy (0906.1361, astro-ph0006390, astro-ph0109126, astro-ph0410051, astro-ph9701125: latex keeps the width, ours natural size); psfrag's epsf.sty (`\RequirePackage{epsfig}`, local reset; astro-ph0012205, astro-ph9611186, astro-ph9911225) is treated as global. Fix: pass the requested extension from `input_definitions`. Review rev78s2 finding 1 | FIXED 78v (`binding_request`: the file the binding answers, content.rs `_load_binding`; a wrapper loading epsfig resets locally; guards `perfect_kernel_batch78v::{epsfbox_input_sty_reads_the_shipped_copy, epsfbox_psfrag_wrapper_resets_locally}`; the 10 witnesses' XML unchanged: each `\epsfbox` there sets its own size) |
| — | Class begin-document code the bindings drop (audit #363, `agent_reports/extracted_356a0c47/2026-10-23T011644_a95c8312ad38d03b1.md`): sn-jnl's `{appendices}` was a bare `<ltx:appendix>` wrapper, so its sections were "§2", its floats and equations "Fig. 2", "(2)" where the PDF prints "Appendix A", "Fig. A1", "(A1)" (46-54 of the census's 164 sn-jnl papers); the Springer classes' `\vec` was the kernel's arrow where llncs, svjour, svjour3, svmult and svproc print a bold vector (113 census papers, KNOWN_PERL_ERRORS #1143); llncs/sv* babel `\extras…` switches never wired | FIXED 79b: sn_jnl_cls.rs loads appendix.sty `[title]` as sn-jnl.cls:1365 and runs the class's begin-document `{appendices}` (:1409-1426; its `\end{oldappendices}` a no-op, so the appendix binding's end restores the section form and the counter goes on, "3 After"), witnesses 2609.24634, 2609.11123, 2609.04047 as their PDFs; `\vec` by the class (`sv_support_sty::define_bold_vectors`, under `orivec`, `\if@mathematic` and the journal file an svjour option inputs — the paper's `sv<option>.clo`, else the census's copies, `JOURNAL_VECTORS` —, `vecphys`/`vecarrow` as each svmult copy tests them); guards `perfect_kernel_batch79b::*` (7). Not done: the babel switches (no witness's output changes: each keeps English as its main language, where `\switcht@albion` restores the binding's own names, OD #613; a document switching to German mid-text would get babel's "Abbildung" where pdflatex prints llncs's "Abb."); svproc's `vecphys`/`vecarrow` (0 census papers); a section after sn-jnl's `{appendices}` keeps the appendix figure numbering ("Fig. C2", as pdflatex) |

Bullets removed from `perfect_kernel/OPEN_ROWS.md` as FIXED (2026-10-23, digest ambiguity 2):

- **subcaption sets the main caption at 90 %** (PERL-ORIGIN; found by rev78a2; **FIXED 78c**, KNOWN_PERL_ERRORS #1021:
  the sub-captions alone are `\small`, the guards re-pinned to the normal size): subcaption_sty.rs's
  `\fnum@font@float`/`\format@title@font@float` are `\small` (Perl subcaption.sty.ltxml:34-35), so with subcaption loaded
  every figure's and table's label and caption text read `fontsize="90%"`; pdflatex sets them in the text font (CMR10,
  checked in jfm and in plain article + subcaption). Pinned as it is in
  `perfect_kernel_batch78a::{jfm_copy_caption_yields_to_caption_loaders,
  jfm_copy_caption_yields_to_caption_loaders_after_floatrow}` (repro
  `class-bindings/jfm_caption/jfm_copy_caption_yields_to_subcaption`); open, small: drop the two `\small`s, then
  re-pin the guards and the captions-floats goldens. 78c r2: a `\usepackage[<options>]{caption}` after subcaption (the
  binding's caption load is the first) runs its options as caption3.sty:416-429 does (`caption.sty_keyval_setter`;
  witness 1602.00134). Open (KPE #1021): sub-captions stay `\small` whatever `\captionsetup[sub]{font=…}`,
  subcaption's `font=` or caption's global `font=small` (→ `smaller`, 8 pt) say (caption_sty.rs `passthrough`).
- **svmult's main-matter `\appendix` starts the appendices** (PERL-ORIGIN, svmult_cls.rs:56 `\@appendix` + `\gdef\@chapapp
  {\appendixname}`; found by rev78a3; **FIXED 78c**, KNOWN_PERL_ERRORS #1026: the main-matter `\appendix` is `\par`, the
  sections are `\@arabic\c@section` without the chapter's reset until a `\title`, svm_app now "Chapter 2 After", "3 N"):
  svmult.cls:713's `\appendix` is only `\par` — the main matter's sections keep
  counting (78a r3 keeps that) and its chapters stay "Chapter N"; here a later `\backmatter\chapter{After}` prints
  "Appendix A", "A.1" where pdflatex prints "Chapter 2 After", "3 N" (probe `svm_app`; int40c, 78a r1-r3 the same; none
  of the 8 corpus svm papers with `\appendix` changes). Open, small: svmult's `\appendix` a `\par` outside the back
  matter, its back-matter `\bppendix` as 78a r3 ports it.
- **biblatex-only `.bib` fields are digested under natbib/BibTeX** (GENUINE-RUST-ONLY, predates 77a; 77a r5 review
  finding 3): 47 field handlers of latexml_engine/src/bibtex.rs, none in Perl's BibTeX.pool — the block at :1962-2172
  (19 defaults: `journaltitle`, `location`, `urldate`, `titleaddon`, `addendum`, `maintitle`, `mainsubtitle`,
  `maintitleaddon`, `booksubtitle`, `booktitleaddon`, `issuetitle`, `issuesubtitle`, `eventtitle`, `eventtitleaddon`,
  `venue`, `eventdate`, `editora`, `editorb`, `editorc`; 27 per-type variants: `inbook|incollection@location`,
  `inbook|incollection|inproceedings@maintitle|mainsubtitle|maintitleaddon`, the book/proceedings host variants) and
  `annotation` (:2337) — exist for the biblatex `.bib` route but are defined on every route, so natbib + plainnat
  prints content the PDF does not have: pdflatex "Ann Able. Paper. J Real, 3:1–2, 2001." 0 errors; we print
  "Main Work Title . Paper . Addon Words . J Real, \undefinedmacro Jt 3, pp. 1–2. Note: An addendum text Accessed:
  2020-01-01" and `Error:undefined:\undefinedmacro` (Perl keeps the fields raw, `\bib@field@default@default`).
  **FIXED 78i** (OD #823; quality ledger #155): the handlers moved next to the `shorthand`/`labelalpha` handlers the
  biblatex binding defines after its `LoadPool!("BibTeX")` (biblatex_sty.rs); witnesses 2602.10008, 2606.11822; GREEN
  `index-bib/natbib_biblatex_only_fields_are_raw_data` (77a r5 review probe n7) and its biblatex twin.
- **A restated theorem prints a reference, not its statement** (SHARED, Perl the same for `{restatable}`:
  thm-restate.sty.ltxml:28 `\lx@thm@restate{}` = "See \ref{restatable:#1}"; 77v review finding 1): thm-restate's
  `\<name>`/`\<name>*` (thm-restate.sty:103-184 `thmt@restatable`: `\thmt@stored@<name>` re-typesets the collected
  body) restate the theorem in full, its head and statement again, numbered as the original; ours print "See ⟨ref⟩" —
  the restated statement text is lost against the PDF wherever a paper restates (`{restatable}` and the `restate=` key;
  2108.11527, 2602.09462, 2402.14202). **FIXED 78i** (KNOWN_PERL_ERRORS #1048): the store collects the body, `\<name>*`
  replays it numbered as the original, `\<name>` as a new theorem, and `{restatable*}` (r2; 1702.04225) prints its
  theorem with the later `\<name>`'s number by reference; guards `perfect_kernel_batch78i::{restated_theorem_prints_its_statement,
  restatable_star_takes_the_later_number}`, `perfect_kernel_batch77v::thmtools_restate_key` (the restated statement).

## Integrations from 2026-10-23 on (narratives here; SYNC_STATUS keeps rows only)

**int74** (2026-10-23; 79c r1-r2 → 61039b657e, docs 140d474145): panel grid markup — `panelrows`/`panelcols`, `ltx_panels_gallery`/`ltx_panels_matrix`, `panelwidth`/`panelspan`, the float's `panelline`/`panelfill` (OD #887; sizing ruling 2026-10-23) — and a `\label` makes no box (KPE #1144). Reviews rev79c (NO-SHIP: data-table matrices, caption rule, collapse strip, schema types) → r2 → rev79c2 SHIP with doc corrections (the minipage-held float's `panelline`, RED `captions-floats/panel_line_of_a_float_in_a_minipage`; two-column classes without a declared page; both rows). Gates int74a: 7163/7163, fmt/clippy/rustdoc clean, inits 0/0, dumps unchanged vs int73a. A/B int73a → int74a over popint70a (1,907): E 6,680 = 6,680, F 20 = 20, words 33,954,819 equal, 0 worse; with the grid markup stripped 1,855 byte-identical, the 52 others the label item's papers rev79c classified (text identical; rows toward the PDF, e.g. 0809.3275 Figs 2-6). Nets 136/136, 0 worse — after refreshing the nets' schema copy for the new attributes (the first run read 1,456 `valid 1->0` against the stale copy).

Row removed as landed (int74):

| — | A `\label` in a box is sized by its key text (`\lx@label`, Perl Whatsit.pm:253-265; TeX's `\label` is a `\write`, no width), so a box holding one is wider than TeX's: four `\subfigure[]{\fbox{\label{fig:110raw-base}\includegraphics[width=0.23\textwidth]{…}}}` in a REVTeX column, 2+2 in the PDF, one per row here (0809.3275 Figs 2-6; the panel rule's real 246pt column made it visible; probe review rev78x `p1/o1.tex`); sub-floats holding only a `\label` (2208.08007's `\newcommand{\labelphantom}[1]{\subfloat{\label{#1}}}`, five or six before the one graphic) are panels of the label's width, a row of their own. Fix: a label makes no box (0 wide; an empty sub-float leaves the panel list) — every box measure holding a label changes, its own A/B | FIXED (79c, KNOWN_PERL_ERRORS #1144): `\lx@label` and cleveref's `\lx@cleverref@labeltype` are `sizer => 0`; a sub-float holding only its number is no panel (`holds_only_its_number`); an alignment cell holding a label at any depth is not empty (`holds_preserved_box`: amsmath's `{gather}` row `c \\ \label{ga}` was kept by the key's width). A/B int71a → 79c (1,907 papers): 52 papers change with the grid markup stripped, every one's text identical; 23 floats in 9 papers change rows, all to the PDF's (0809.3275 Figs 2-6 2+2/3, 2208.08007 Figs 1-4, 6, 8, 2406.04934 Figs 3, 6, A7, A8, A10, 1306.2459 Fig 2, 1404.0431 Fig 1, 1501.06448 Fig 19, 2106.04060 Figs 1, 3, cond-mat0310505 Fig 13 one row of two; 2410.08976 Fig 4 no PDF float, its two graphics one row), the only element change a dropped `ltx:break`; 43 change box sizes only (rotated tables, `\resizebox` scales, TikZ/foreignObject boxes holding a label; 2012.11799's pgfpicture `tex=` sizes). Guards `perfect_kernel_batch79c::{a_label_makes_no_box, a_label_only_sub_float_is_no_panel}` |

**int75** (2026-10-23; 79d r1-r3 → c9b8c1bda8, main-session follow-up 15afe92799): equation numbers and reference text — a numbered eqnarray continuation row keeps its number (OD #888; 1310.2253 `eqn:zg` 12 → 11 as the PDF); IEEEeqnarray labels in unnumbered rows name the next number (`labelnext=1`, OD #847's rule), sub-equations back out a pre-advance and a `\nonumber` after `\IEEEyesnumber`/`\IEEEyessubnumber`/`\IEEEnosubnumber` holds the counters (KPE #1146); cleveref's `\appendix` alias is global and never cleared, every sectioning unit after it an appendix type, `{appendices}` alone keeps `section` (KPE #1145; first-aid-off pdflatex the oracle); the Springer `\vec` residuals of 79b (OD #889). Reviews rev79d (NO-SHIP: the alias) → r2 → rev79d2 (NO-SHIP: sub-number `\nonumber`, starred rollback, A/B population) → r3 → rev79d3 (NO-SHIP: `\IEEEnosubnumber\nonumber`, one line) → main-session fix with guard `ieeeeqnarray_nonumber_after_nosubnumber_holds` (probe matches pdflatex, as the review's SHIP condition). Gates int75a 7176/7176, inits 0/0, dumps unchanged. A/B int74a → int75a over popint75a (496: the three rounds' populations): E 1,440 = 1,440, F 7 = 7, words +124, 0 worse; 85 byte-changed — 55 cleveref types only (subsections after `\appendix` → appendix, `\appendix` around `{appendices}`; controls 2412.06505, 2512.18601, 2605.17958 and the sv paper 2609.03341 are this class), 10 equation numbers only, 20 numbers with math moving between split/joined rows. Nets 136/136, 0 worse. Capital Greek (user ruling 2026-10-23, Option A) → 79e.

Rows removed as landed (int75):

| — | eqnarray: a continuation-only row (`&&+ 2,`, nothing before its relation column) under a numbered row joins that equation though both rows are numbered, and the join keeps the last number: the first row's number is lost and a `\label` on it names the second's (1310.2253 `eqn:zg`, PDF 11, ours 12; Perl `rearrangeEqnarray` the same; found by 78r's `.aux` check) | FIXED 79d: a numbered third-column-only row under a numbered equation is its own equation (OD #888); guard `perfect_kernel_batch79d::eqnarray_numbered_continuation_row_keeps_its_number` |
| — | IEEEeqnarray: a `\label` in an unnumbered row does not name the counter's next value as IEEEtrantools' does (`a &=& b \\ c &=& d \label{ie1}\nonumber`: ours "1", pdflatex "2"; `\label{ie2}` in an `IEEEeqnarray*` before the next equation: ours the raw id "S0.Ex2", pdflatex "2"; int54a and 78r alike; review rev78r finding 6) | FIXED 79d: IEEEeqnarray opts into #847's rule with the numbering key `labelnext=1` (IEEEtrantools.sty:1986-1991, :2072-2117 pre-advance the counter and `\@currentlabel`); guard `perfect_kernel_batch79d::ieeeeqnarray_unnumbered_row_label_names_next_number` |
| — | IEEEeqnarray sub-equations: `\IEEEyessubnumber` on a pre-advanced row starts a new equation's sub-numbers ("1, 2a, 3a" where pdflatex prints "1, 1a, 1b"; int72a, 79d r1; Perl IEEEtran.cls.ltxml:269-278 the same, KPE #1146); review rev79d finding 6 | FIXED 79d r2: `\IEEEyessubnumber` backs out a pre-advanced number as IEEEtrantools.sty:1408-1424 does ("1, 1a, 1b", was "1, 2a, 3a"; 2012.02697 `eq:XHp`/`eq:Q2` 11a/11b as the PDF), `\IEEEnosubnumber`/`\IEEEyesnumber` as :1426-1437/:1393-1403, and a label in an `\IEEEnonumber` row before such a row keeps the number it named (probe `rev79d/eq/p6`: "2, 1a"); guard `perfect_kernel_batch79d::ieeeeqnarray_sub_numbers_follow_the_last_number`. r3 (review rev79d2 findings 1-2): a `\nonumber`/`\IEEEnonumber` after `\IEEEyesnumber`/`\IEEEyessubnumber` on a row wins (`\global\@eqnswfalse`), the row's counters staying for the next row to print (`EQUATIONROW_HOLDS`/`EQUATIONROW_CARRY`, latex_constructs/mod.rs; probe `rev79d2/eq/n.tex` 10 printed numbers → pdflatex's 7, `h.tex` block k "(4) (4)" → "(4) – (4a)"; 2012.02697 `eq:fpar`/`eq:par` 14a/14b as the `.aux`), and every retraction gives back the sub-equation number its pre-advance reset (an `IEEEeqnarray*` `\nonumber` row before an `\IEEEyessubnumber` row: `s.tex` "0d, 0e, 0a" → "0d, 0e, 0f"; 1610.06458 `eq:inq3` 36c); guards `perfect_kernel_batch79d::{ieeeeqnarray_sub_numbers_follow_the_last_number, ieeeeqnarray_nonumber_after_yessubnumber_wins, ieeeeqnarray_starred_nonumber_gives_back_the_sub_number}`; int75 follow-up: `\IEEEnosubnumber` then `\nonumber` holds the counters too (rev79d3 finding 1, guard `ieeeeqnarray_nonumber_after_nosubnumber_holds`) |
| — | cleveref types a unit of appendix.sty's `{appendices}` `appendix` ("appendix A", `<tag role="creftype">appendix</tag>`) where pdflatex keeps `section` ("Sec. A"): cleveref.sty:175-176 patches only `\appendix`, and the environment never calls it, so a `\cref` to a section inside it keeps the section name. Reached through appendix_sty.rs `\lx@pp@appendix@begin` → `begin_appendices` (latex_constructs/mod.rs:306), so every appendix.sty + cleveref paper; base printed "Sec. 5" (the right name, the section number where the PDF has a letter). Witnesses 2507.16108 (`\crefname{section}{Sec.}`, ~8 "Sec. A…" in the PDF), 2511.19260 ("Section A" twice); the type changed in 2203.14452, 2502.21249, 2508.04603, 2608.04513 too (review rev79b (`agent_reports/extracted_356a0c47/2026-10-23T024112_a8b3383dff3c35201.md`, probes `~/.claude/jobs/356a0c47/tmp/rev79b/`), finding 1). Fix direction: units opened by the environment, not `\appendix`, keep the cleveref type `section` | FIXED 79d r2 (KPE #1145): cleveref_sty.rs wraps the `\appendix` cleveref patched, a global flag never cleared (cleveref.sty:174-287 `\gdef`s the aliasing by counter name); `cref_type` names a unit by its counter until it is set, and every sectioning unit by its appendix type after (`appendix`, `subappendix`, … ; with chapters one level down). 79d r1 flagged the environment's units instead, so `\appendix` before or inside `{appendices}` gave "section" (review rev79d finding 1: 1810.06583, 1904.07153, 2208.03406, 2112.09636 — they refer by `\ref`, so their PDFs never print the type; the oracle is the first-aid-off pdflatex probes). Guards `perfect_kernel_batch79d::{cleveref_appendices_environment_keeps_section_type, cleveref_appendix_before_appendices_names_appendix, cleveref_appendix_inside_appendices_names_appendix, cleveref_appendix_in_a_book_aliases_its_sections}`. Main-session decision 2 (79d): after `\appendix` "appendix", as first-aid-off pdflatex prints (TL 2025's first aid "section"; the arXiv-era kernel is pre-first-aid); so `\appendix\crefalias{section}{appsec}` prints "appendix A" too (cleveref.sty:183-193 reads only `\cref@appendix@alias`), re-pinned `perfect_kernel_batch68h::cleveref_crefalias_names_the_type` (its `{appendices}` form "supplement A" as before). |
| — | A text-mode `\vec{w}` under llncs warns "`\lx@sv@vec@boldmath` should only appear in math mode" (sv_support_sty.rs `\lx@sv@vec@boldmath`, `require_math`), naming the internal macro instead of `\vec`; pdflatex errs "Missing $ inserted", so a diagnostic is right, only the name is off (probe `rev79b/ltext/`; review rev79b (`agent_reports/extracted_356a0c47/2026-10-23T024112_a8b3383dff3c35201.md`, probes `~/.claude/jobs/356a0c47/tmp/rev79b/`), finding 7) | FIXED 79d: a constructor's `require_math` warning names its `alias`, the command it stands in for (dialect.rs `def_constructor`); guard `perfect_kernel_batch79d::llncs_text_mode_vec_warns_as_vec` |
| — | svmono and SNmult papers, served by the svmult binding, never have their own class copy read for `\vec`: svmult_cls.rs asks `shipped_class_text("svmult")` (:80; the same pattern as `asks_for_runheads`, :372), which looks for `svmult.cls`, so they get the class-as-distributed (begin-document) form; it differs from the class only under `vecarrow` with an older copy whose test runs in the class body (review rev79b (`agent_reports/extracted_356a0c47/2026-10-23T024112_a8b3383dff3c35201.md`, probes `~/.claude/jobs/356a0c47/tmp/rev79b/`), finding 8) | FIXED 79d: the binding reads the copy of the class it serves (`svmono.cls`, `SNmult.cls`), for `\vec` and the running heads; witnesses 0910.4696, 1001.2906, 1408.4098, 2407.02289, 2602.15816, 2609.20684; guard `perfect_kernel_batch79d::svmono_vec_reads_its_own_copy` |

**int76** (2026-10-23; 79e r1-r2 → 42278a7938): capital Greek follows the class (user ruling 2026-10-23, Option A; FIDELITY_CHARTER §5; KPE #1147) — `Font::specialize_letters`: a capital Greek `\DeclareMathSymbol`'d `\mathalpha` from an OML italic `letters` font is italic like a Latin letter (a math alphabet overrides), `\mathord` from `letters` always italic (amsmath's `\var…`, now inside `\@ifundefined` as amsmath.sty:384-395); `declare_capital_greek` where each original declares (llncs, svmult/svmono/SNmult, svjour/2/3, svproc; mathptmx/mathpple/ccfonts/cmbright/newtxmath/newpxmath/mathpazo under their options; upright re-declarers newtx/newpx/mathpazo default, eulervm, euler, fourier, stix, stix2, libertinust1math); mathptmx/mathpazo/eulervm `\up…` keep their names (`tex=` as written); newtxmath/newpxmath option order as `\ProcessOptions`/`\ProcessOptions*`. Review rev79e SHIP with r2 recommended (the upright re-declarers, mathptmx `\up…`) → r2 → main-session probe check (llncs+stix upright; llncs+mathptmx `\up…` upright, `\Omega` italic). Gates int76a 7185/7185, inits 0/0, dumps unchanged. A/B int75a → int76a over popint76a (829: both rounds' populations): E 3,969 = 3,969, F 8 = 8, W and words equal, 0 worse; 223 byte-changed — 215 capital Greek font attributes or box sizes only, 5 widths/positions following the italic glyph (1702.08154, 2102.03510, 2106.13878, 2210.17139, 2508.05134), 3 roles by load order (1401.7612 imamat's own `\varOmega` kept by amsmath's `\@ifundefined`; 2305.05769 newpxmath re-declaring isomath's `\Delta`; 2310.12571 IEEE-Con-Sys-mag's raw `\Phi` now Φ, was the glyph "8"). Nets 136/136, 0 worse.

Row removed as landed (int76):

| — | Capital Greek in the bold-italic `\vec` forms is upright bold: llncs, svmult under `vecphys`/`vecarrow`, svjour without `\if@mathematic` declare capital Greek `\mathalpha` letters (svjour3.cls:157, svmult.cls:981, llncs.cls:380-389), so pdflatex prints `\vec{\Omega}` in CMMIB (probes `rev79b/v_l`, `rev79b/v_m_hook_arrow`: no CMBX) — the bindings keep capital Greek upright. Separately, svmult_cls.rs's `\varOmega` DefMath is italic where svmult.cls:993 declares it from `operators` (upright) (review rev79b (`agent_reports/extracted_356a0c47/2026-10-23T024112_a8b3383dff3c35201.md`, probes `~/.claude/jobs/356a0c47/tmp/rev79b/`), finding 4) | `\varOmega` & co. FIXED 79d (declared upright as svmult.cls:982-993). Capital Greek FIXED 79e (user ruling 2026-10-23, Option A "follow the class"; KNOWN_PERL_ERRORS #1147): `Font::specialize_letters` — a capital Greek glyph `\DeclareMathSymbol`'d from an OML italic symbol font (`letters`) takes the rule of its fontmath.ltx class, `\mathalpha` the Latin letters' (italic unless a math alphabet is in force), `\mathord` the lowercase Greek letters' (italic always); set by the kernel's `\DeclareMathSymbol` (raw copies, fixmath) and by `declare_capital_greek` in llncs, svmult/svmono/SNmult, svjour/svjour2/svjour3, svproc, mathptmx/mathpple/ccfonts/cmbright (`slantedGreek`), newtxmath/newpxmath/mathpazo (`slantedGreek`/`uprightGreek`/`frenchmath`, default upright over a class's italic), eulervm/euler/fourier and (79e r2, review rev79e finding 1) stix/stix2 (not under `nomath`) and libertinust1math (`slantedGreek`/`uprightGreek` in the order given, `frenchmath`, `ISO`) (upright over a class's italic), amsmath's `\var…` (`\@ifundefined`) and apjfonts'; mathptmx's, mathpazo's and eulervm's `\up…` letters are glyphs of their own (`declare_capital_greek_named`: upright under any class, `tex="\upGamma"` as written; review rev79e finding 2, main-session decision 1); newtxmath's options in declaration order (`slantedGreek` wins), newpxmath's in the order given (`\ProcessOptions*`). `$\Omega$` `<mi>Ω</mi>`, `\boldsymbol`/`\bm`/the bold-italic `\vec` `<mi>𝜴</mi>`, `\mathrm` `normal`, `\mathbf` 𝛀, as pdflatex (probes `scratch_79e/g`). Guards `perfect_kernel_batch79e` |

**int77** (2026-10-23; FM S2a.2 r9 → d53fe993d3, main-session nits 34d6d13fc4; shadow only): a link contact carries its target (`mailto:` → an e-mail matched by address; 2210.15991, 2309.10007), the API legend rule reads its pieces (1903.06934's affiliations back; tag `api:legend`), an honest note order and per-copy rules in the comparator, nits; the rehearsal leaves the conversion as it found it — its scope removes the field-provenance stamps it added to `\usebox`ed boxes, and the assignment journal restores a name's fallback-guess standing (siamart's `\AND`) — so the XML is byte-identical with `LATEXML_FM_SHADOW` set as well as unset (2405.14627, 2603.26792, 2409.10200 fixed). Review revfms2a2_9 LAND (positions 1-4 confirmed; S2b NO-GO on the plan as written → restated in the S2b r1 brief). Gates int77a 7193/7193, inits 0/0, dumps unchanged. A/B int76a → int77a over int65's 725: unset 0 byte-changed, set 0 byte-changed, int77a set vs unset 724/724 identical (workdir path normalised); E 169 = 169, F 1 = 1. Census (writer, 26,323): 19,510 agree / 1,880 classified / 1,816 unclassified; revtex4_support 23/5,639 unclassified. Nets 136/136, 0 worse.

Row removed as landed (int77):

| — | **FM shadow rehearsal leaks into the conversion** (FM S2a.2 r8's census, unset vs set; pre-existing — int69b and r7 alike): a box register an author block prints with `\usebox` came out wrapped in an `ltx:text` with `LATEXML_FM_SHADOW` set (2405.14627's `\usebox{\ORCIDicon}`, 2603.26792's one box behind four authors' `\href`s); under siamart220329 algorithmic's `\AND` printed a plain "and" (2409.10200). Causes: the rehearsal's print stamped a register's shared box with a field provenance its `FieldLedgerScope` dropped (the conversion's print then did not stamp it again), and its group-local `\let\AND` cleared OmniBus's fallback standing (`fallback_meanings`), which no group end restores, so `\newcommand{\AND}` was refused | **FIXED fms2a2r9** (OD #837 "S2a.2 r9": the scope unstamps the boxes it stamped, `token::field_stamp_noted`; an assignment journal saves and restores a meaning's fallback standing, `AssignmentJournal::fallback`). Census 26,323 set vs unset: 0 differences but 2109.00764's clock and 1001.0023's time-budgeted math parse (4 reruns identical in both modes); int65's 725 and the 992 sectioning-frontmatter repros identical. Guards `perfect_kernel_batchfms2a2r9::the_rehearsal_leaves_the_conversion_unchanged`, `state::assignment_journal_tests::rollback_restores_a_fallback_standing`; repros `observer_rehearsal_leaves_a_used_box_unchanged` (GREEN), `…_shared_box_unchanged`, `observer_rehearsal_keeps_a_fallback_meaning` |

**Stopping point** (2026-10-23, quota): main at int77 (`a77feedf2b`, pushed). Two payloads were interrupted mid-round, their state saved as
unreviewed, ungated WIP commits on local branches (not pushed): `pk/79f` `3e02cbffe8` (worktree `~/data/pk_wt/79f`; the panel page model —
declared pages for two-column classes, a minipage-held float's `panelline`, a sub-float's stated glue; brief
`~/data/pk_agents/main/briefs/79f.md`) and `pk/fms2b` `e1fff9a9f5` (worktree `~/data/pk_wt/fms2b`; FM S2b pilot on revtex4_support, the
r9 review's restated plan; brief `briefs/fms2b_r1.md`; r9's census tooling kept in `~/data/pk_wt/scratch_fms2a2r9/`). Next: finish each
from its WIP (or restart from its brief), review, integrate as int78.
