# TeX Live manuals: regressions to recover before the merge (living worklist)

**Why.** The user, 2026-11-06: "all regressions on the perfect_kernel texlive manuals need to be addressed before merge to main. This remains the primary goal of this branch." The bar is in [`../PERFECT_KERNEL.md`](../PERFECT_KERNEL.md) "Goal bar"; it was met at sweeps #146-148 and nearly at #149, and no sweep ran for 25 days and many bundles after that.

**How to run it.** `~/data/pk_agents/main/chain/manual_sweep.sh <N> <label> [cores] [jobs]`, then `tools/perfect_kernel/scoreboard.py <prev> <N>` and `--open <N>`. It now joins every bundle's checks, because the arXiv A/B and the binding nets do not see these regressions.

## The measure

Sweep #150 on `d6e175bacb` (binary int140a), 180 s, 8 GB, its timeouts re-run on quiet cores (the worklist below keeps its first-pass statuses). Sweep 902 on M2 (`f145413b64`), the same protocol, before the review's fix rounds.

| Measure | #149 (2026-10-11, `e57be39640`) | #150 (2026-11-06) | 902, M2 (2026-11-06) |
|---|---:|---:|---:|
| Goal set 1,552, passing G1-G4 | 1,549 (99.81 %) | 1,531 (98.65 %) | 1,550 (99.87 %) |
| G1 unclean / G2 invalid / G3 below 95 % / G4 over 180 s | 0 / 1 / 0 / 2 | 10 / 6 / 5 / 0 | 0 / 0 / 0 / 2 |
| In scope 1,602, clean | 1,590 (99.25 %) | 1,582 (98.75 %) | 1,590 (99.25 %) |
| Whole corpus 2,374: clean; no Fatal or timeout | | 1,945 (81.93 %); 2,311 (97.35 %) | |
| In scope, clean + valid + raw recall ≥ 95 %, no classification (first pass) | 1,363 (85.08 %) | 1,350 (84.27 %) | |

Vs #149: status worse on 53 manuals, better on 220; errors 11,246 → 11,278, up on 44, down on 85. **None comes from Bundle int140a:** 21 of them run on int139c give the same status and error count.

## The clusters

- **A, a binding's name prefix captures a different class. FIXED** (`content.rs` `is_unrelated_distribution_class`, OD #444; guards `perfect_kernel_batchm1`): a prefix marks an author's copy beside the document or a sibling shipped beside the binding's class (`jmlrbook`, `acmart-tagged`); any other class of the distribution is read raw. Measured: the 7 manuals back to #149's status at 0 errors. Was: `content.rs` `raw_excluded`: under raw classes, a class with a prefix-matching binding is not read raw (OD #444, for an author's renamed copy: `IEEEtranTCOM.cls`, `IEEEtran4PSCC.cls`). It also fires on a published class of its own: `bookest` → `book`, `bookcover` → `book`, `beamerswitch` → `beamer`, `bookshelf` → `book`; so their commands are undefined. Sweep #149's log read them raw. TeX Live classes the prefix catches (16): `aalok`, `aastex701`, `acmart-tagged`, `anima`, `apa6e`, `articleingud`, `beamer-rl`, `beamerswitch`, `bookcover`, `bookest`, `bookshelf`, `jmlrbook`, `postcards`, `reporting`, `revtex4-1`, `revtex4-2`.
- **C1, a package's own `\multicolumn` refused. FIXED** (`state.rs` `another_interface`, `\multicolumn:locked@arity`, OD #209; guard `perfect_kernel_batchm1::a_package_multicolumn_of_another_arity_is_its_own`): sltables.sty:193's one-argument command stands; a three-argument copy and one with no parameters stay refused (a guard each; the lock looks at a definition's parameters, and a `\let` was never refused by plain `:locked`: `let_i`, as Perl's `Let`). sltables-doc-en 11 errors → 0. A no-parameter redefinition of another interface is refused too, an existing limit (RED `multicolumn_of_no_declared_parameters_is_refused_red`).
- **C2, the tblr read-ahead takes an inner `&` as the outer cell's end. FIXED** (`tabularray_sty.rs`, `SuppressedTabMarks` across `tblr_scan_body`; `::a_tblr_in_a_p_cell_reads_its_tab_marks_safely`). ProfSio-doc-fr 10 → 0, tikz3d-fr-doc 20 → 0.
- **C3, the latex-lab block binding omits instances. FIXED** (`latex_lab_testphase_block_sty.rs`, the `block` instances of the file's :1362-1411; `::the_block_instances_of_the_file_are_declared`). keytheorems-doc 4 → 0.
- **C4, a model raw code defines reported unknown. FIXED** (`xcolor_sty.rs`, the test of `\XC@mod@<model>`, KPE #958; `::a_color_model_is_known_by_its_xcolor_definition`). colorspace 8 errors → 0; the colour stays the recovery black, said by one counted `Warning:color_model:<model>` per model, so the manual is status 1 (#149: 0 errors, status 0, silently black).
- **C5, ijsra's `\def\abstract{text}` refused. FIXED by a class binding, the kernel's lock unchanged** (`ijsra_cls.rs`; `::the_ijsra_abstract_is_semantic`): this class frees `\abstract` (as amsppt's binding does), and `{IJSRAabstract}` is the kernel's `{abstract}` with an `ltx:keywords`, so the opening's and the manual's French abstract are both `<abstract>`. Under any other class the value stays refused, 1 Error, the text lost (repro `macro-state/abstract_value_under_another_class_is_refused`; this manual alone in the corpus writes such a value, and none of the census's 35,942 papers uses this class). The opening's author stays a `{center}` line, no `<creator>`: making one means re-implementing `\IJSRAopening`'s block of up to five authors, affiliations and notes. A document with no `\abstract` was clean before the binding too (`\undef` is a `\let`).
- **C6, a cover line between two late abstracts stays a body `<para>`. FIXED** (`cover_order.rs` `place_later_abstract`, OD #903; `::a_cover_line_between_two_abstracts_is_a_titlepage`). The 6 `-eng`/`-fra` manuals 2 jing lines → 0.
- **C7, `\selectfont` of an unmapped family stays in `\nullfont`. FIXED** (`sect13.rs` `nfss_selected_font`, OD #779; `::selectfont_of_an_unmapped_family_leaves_nullfont`). prerex/chart 51.0 % → 100 %.
- **C8, a block whose only text is `\LaTeX` counts as empty. FIXED** (`util_text.rs` `template_prints_text`, KPE #428; `::a_template_of_its_own_text_is_content`). niepraschk-eso-pic 66.7 % → 100 %.
- **C9, multibib's style never reaches `BIBSTYLE`. FIXED** (`multibib_sty.rs`, OD #823; `::a_multibib_style_reaches_the_bst_fields`). quantum-bibliographystyle-demo 93.2 % → 97.3 %. Each list reads its own style by its suffix (`::each_multibib_list_reads_its_own_style`). Limit: the `.bst` fields are read once per document, the main list's style, with none the last declared.
- **The re-review's REDs, all synthetic and no arXiv reach found** (`tools/perfect_kernel/repros/package-bindings/`): `color_model_of_a_driver_over_an_earlier_color_red` (a driver's model keeps the earlier colour silently, pre-existing), `xcolor_redefined_hsb_ranges_red` (`\rangeHsb`, `\rangetHsb` ignored; `Hsb` pre-existing, `tHsb` new with its conversion), `colorbox_of_an_unconverted_model_black_on_black_red` (the recovery black behind black words, pre-existing; no background instead needs a ruling).
- **Ruled out of the worklist:** serbian-def-cyr/proba `recall 19` and latex-refsheet/LaTeX_RefSheet `recall 75` in `accepted_residuals.tsv`, with their reasons (`j` in T2A; a word inside an included PDF).
- **F1-F10, one mechanism each. FIXED** (guards `perfect_kernel_batchm3`, each repro red first against pdflatex; statuses in the table's `Done`): F1 `\@ensuremath` of no parameter (KPE #1212); F2 comment.sty's internals in its binding (KPE #1213); F3 a `\protected` `\begin`, `\protect\end ` in an expansion-only regime (KPE #1211), blkarray's `\BA@begin`, and a pending `\afterassignment` held through an `\edef`, which had left `\protect` stuck (KPE #1215); F4 `\everypar{}` in `\parbox`, minipage, `p` cells, floats (KPE #1214); F5 `\lx@pdfstringdef` under a latin `inputenc`, the digit commands (OD #974); F6 `\lx@repeat@xcolor@<opt>` (OD #43); F7 pgfmath's arrays (OD #973); F8 keytheorems' `\__keythms_thm_tcbtemphead:` (OD #975); F9 a `\NewColumnType` body's commands as letters-only stand-ins, a `cmd`'s cell text trimmed (`column_type_body_text`; repro alignment-bindings/tblr_column_type_body_keeps_its_commands); F10 autonum.sty:20-22 ported, etoolbox's `\forlistloop` back after etextools, the load kept, not dropped. **Residuals:** non-decimal-units E14: `\g__tblr_table_count_int` is in tabularray-2021.sty, not in TL 2025's tabularray.sty, so pdflatex errs too (16 `!` on a three-row table), our table now its content; the 4 `Match:?` were `HEADER ` not matching `HEADER`, gone. tikz-trackschematic-snippets E147: `./snippets/*.tikz` and `./examples` are absent as the manual names them (the bundle ships `tikz-trackschematic-snippets/`), each an honest Error, no Fatal. tikzpingus-doc: E0, a timing question, 180 s. RED left: `package-bindings/etextools_forlistloop_under_translations_hook_red` (an explicit siunitx + etextools).
- **floatrow-rus, root-caused** (SYNC row 0a): no defect of the deadline; external (picins.sty absent, `\Describe@Macro` gone from doc.sty v3).
- **B, `\pdftex_if_engine:TF` undefined** (mdframed, 4 manuals, E about 290 each): not yet root-caused.
- **T, timeouts:** re-run on quiet cores first; several may be load. `cpu_h` rose 0.92 → 1.20 in scope, partly load. **G4, pgf-interference-en/-de, is real, not load:** on M2's binary both time out at 180 s with their SMT siblings idle (mean sibling load 6 %), where they took 160 s on idle siblings on 2026-10-11 and 156.6 s at batch 59q. **A kernel drift, not this manual:** `kit/perf_bisect.sh`, `instructions:u` on the TikZ audit's benchmarks, 59q `1be2a69de7` → `d6e175bacb`: fp1000 83.77 → 88.94 G (+6.2 %), fill1000 13.26 → 14.85 G (+12 %), addplot500 28.22 → 31.82 G (+12.8 %); 59q's own 83.77 G reproduces exactly. Every pgf-heavy document pays it. Steps refined by `kit/perf_bisect.py`; the fix batch is next. One pattern is 143 G, 9.3 s (lualatex 9.1 s, pdflatex 3.0 s).
- **V, newly schema-invalid** (annee-scolaire, paresse, simples-matrices): C6 above.
- **R, recall** (LaTeX_RefSheet, prerex/chart, quantum-bibliographystyle-demo, serbian-def-cyr/proba, niepraschk-eso-pic): C7, C8, C9 and the two residuals above. tabularray and tkz-grapheur were their HTML pass, back at M2. Above the bar, no loss, closed: datetime2-english-sample (100 → 96.6 %) and SubSupScripts (99.3 → 98.5 %) miss `october` because `SOURCE_DATE_EPOCH` now prints 1 January 2026 and the goldens carry their own year (2019, 2009); #149's HTML matched only by printing "October 11, 2026"; more than 10 other manuals share it. SubSupScripts' `expression` was missed at #149 too, 19 in the XML. cmpj/template (98.9 → 96.0 %) misses Cyrillic words, i → і (0xB3 → U+0456), the mechanism of serbian's j (0xBC → U+0458).
- **?, the rest:** each row's first error below is the lead.

**Also watch:** 25 manuals went status 0 → 1, new Warnings only, listed at the end; read them after the Errors, since 108b's and 108a's counted Warnings are intended. Japanese `jsarticle` manuals moved to OmniBus by `declared_other_format`, intended and out of scope.

## The worklist (non-warning rows, 66)

Status / errors at #149 → #150, in scope or canary. `in` manuals count against the bar; canaries count for a new Fatal or timeout.

| Manual | Scope | #149 | #150 | What regressed | Cluster | First error at #150 | Done |
|---|---|---|---|---|---|---|---|
| annee-scolaire/annee-scolaire-eng | in | 1/E0 | 1/E0 | invalid | V |  | fixed, M2 C6 |
| annee-scolaire/annee-scolaire-fra | in | 1/E0 | 1/E0 | invalid | V |  | fixed, M2 C6 |
| beamerswitch/beamerswitch-example | in | 1/E0 | 2/E2 | status,errors,recall-lost | A | Error:undefined:\handoutlayout The token T_CS[\handoutlayout] is not defined. | fixed, M1 |
| biblatex-arthistory-bonn/arthistory-bonn | canary | 3/E7 | 124/E6 | status | T | Error:missing_file:arthistory-bonn-preamble.tex Can't find TeX file arthistory-bonn-preamble.tex | |
| biblatex-ms/biblatex-ms | canary | 2/E2 | 2/E3 | errors | ? | Error:undefined:{release} The environment {release} is not defined. | |
| bibleref-german/de-bibleref-german | canary | 2/E1 | 3/E0 | status,invalid | ? | Fatal:Timeout:Recursion  Infinite expansion loop: a window of 2 token(s) repeated 100+ times | fixed, M3 F5, 2/E1 |
| bibleref-german/en-bibleref-german | canary | 2/E1 | 3/E0 | status,invalid | ? | Fatal:Timeout:Recursion  Infinite expansion loop: a window of 2 token(s) repeated 100+ times | fixed, M3 F5, 2/E1 |
| bookcover/bookcover-example1 | canary | 0/E0 | 2/E5 | status,errors,recall-lost | A | Error:undefined:\newbookcovercomponenttype The token T_CS[\newbookcovercomponenttype] is not defined | fixed, M1 |
| bookcover/bookcover-example2 | canary | 0/E0 | 2/E2 | status,errors,recall-lost | A | Error:undefined:{bookcover} The environment {bookcover} is not defined. | fixed, M1 |
| bookest/bookestdoc-en | in | 0/E0 | 2/E5 | status,errors,recall-lost | A | Error:undefined:\setoddheadtext The token T_CS[\setoddheadtext] is not defined. | fixed, M1 |
| bookest/bookestdoc-it | in | 1/E0 | 2/E4 | status,errors,recall-lost | A | Error:undefined:\setoddheadtext The token T_CS[\setoddheadtext] is not defined. | fixed, M1 |
| bracketkey/Malva | canary | 0/E0 | 2/E61 | status,errors,recall-lost | ? | Error:unexpected:T_CS[\theindex] \theindex' invalid at this point | |
| cmpj/template | in | 1/E0 | 1/E0 | recall-drop 98.9→96.0 % | R |  | closed, i→і as serbian's j, no loss |
| colorspace/colorspace | in | 0/E0 | 2/E8 | status,errors,recall-lost | ? | Error:unexpected:fooshaded Unrecognized color model 'fooshaded' | fixed, M2 C4 |
| currency/currency_doc | canary | 2/E39 | 2/E41 | errors | ? | Error:undefined:\href The token T_CS[\href] is not defined. | |
| datetime2-english/datetime2-english-sample | in | 0/E0 | 0/E0 | recall-drop 100.0→96.6 % | R |  | closed, the sweep's fixed date, no loss |
| draftfigure/draftfigure | canary | 2/E24 | 2/E25 | errors | ? | Error:missing_file:draftfigure-preamble.tex Can't find TeX file draftfigure-preamble.tex | |
| epslatex-fr/fepslatex | canary | 2/E10 | 2/E11 | errors | ? | Error:expected:{ Missing { inserted | |
| floatrow/floatrow-rus | canary | 3/E101 | 124/E52 | status | T | Error:latex:(caption) Package caption Error: The counter subtable' was already defined by | root-caused, M3: external, no deadline defect; 124/E52 |
| gtrcrd/gtrcrd-doc | in | 0/E0 | 2/E6 | status,errors,recall-lost | A | Error:undefined:\setleftmark The token T_CS[\setleftmark] is not defined. | fixed, M1 |
| ijsra/ijsra | in | 1/E0 | 2/E1 | status,errors,recall-lost | ? | Error:unexpected:\endlx@list Attempt to end mode internal_vertical | fixed, M2 C5 |
| keytheorems/keytheorems-doc | in | 1/E0 | 2/E4 | status,errors,recall-lost | ? | Error:errmessage:\errmessage LaTeX template Error: The instance 'theoremblock-0' of type 'block' is | fixed, M2 C3 |
| kotex-oblivoir/ob-mathleading-doc | canary | 2/E4 | 2/E5 | errors | ? | Error:undefined:\luatexkomainhangulfamily The token T_CS[\luatexkomainhangulfamily] is not defined. | |
| lineno/fnlineno | canary | 0/E0 | 2/E1 | status,errors,recall-lost | ? | Error:undefined:\3 The token T_CS[\3] is not defined. | fixed, M3 F5 |
| ltxkeys/ltxkeys-guide | canary | 3/E27 | 3/E31 | errors | ? | Error:undefined:\ltscomment The token T_CS[\ltscomment] is not defined. | |
| mathfont/mathfont-example-typey | canary | 2/E9 | 2/E10 | errors | ? | Error:latex:(mathfont) Package mathfont Error: | |
| mdframed/mdframed-example-default | canary | 2/E44 | 2/E295 | errors | B | Error:undefined:\pdftex_if_engine:TF The token T_CS[\pdftex_if_engine:TF] is not defined. | |
| mdframed/mdframed-example-pstricks | canary | 2/E44 | 2/E49 | errors | B | Error:undefined:\pdftex_if_engine:TF The token T_CS[\pdftex_if_engine:TF] is not defined. | |
| mdframed/mdframed-example-texsx | canary | 2/E44 | 2/E292 | errors | B | Error:undefined:\pdftex_if_engine:TF The token T_CS[\pdftex_if_engine:TF] is not defined. | |
| mdframed/mdframed-example-tikz | canary | 2/E44 | 2/E290 | errors | B | Error:undefined:\pdftex_if_engine:TF The token T_CS[\pdftex_if_engine:TF] is not defined. | |
| membranecomputing/membranecomputing | canary | 1/E0 | 2/E8 | status,errors,recall-lost | ? | Error:expected:{ Missing { inserted | fixed, M3 F1 |
| memorygraphs/memorygraphs | canary | 2/E4 | 2/E5 | errors | ? | Error:latex:(latexgit) Package latexgit Error: invoke LaTeX with the -shell-escape flag         See  | |
| ndsu-thesis-2022/NDSU-Thesis-Extended | canary | 2/E1 | 2/E2 | errors | ? | Error:undefined:\iftotaltables The token T_CS[\iftotaltables] is not defined. | fixed, M3 F6, 2/E1 |
| ndsu-thesis-2022/ndsu-example | canary | 1/E0 | 2/E1 | status,errors,recall-lost | ? | Error:undefined:\iftotaltables The token T_CS[\iftotaltables] is not defined. | fixed, M3 F6 |
| non-decimal-units/non-decimal-units | canary | 2/E1 | 2/E22 | errors | ? | Error:latex:(csvsimple) Package csvsimple Error: File 'unicode-alphabets.MUFI    .csv' not existent, | M3 F9: content back, E14 external (tabularray-2021's int, pdflatex errs too) |
| paresse/paresse-eng | in | 1/E0 | 1/E0 | invalid | V |  | fixed, M2 C6 |
| paresse/paresse-fra | in | 1/E0 | 1/E0 | invalid | V |  | fixed, M2 C6 |
| plweb/pl | canary | 2/E12 | 3/E0 | status | ? | Fatal:Stomach:Recursion  Excessive recursion(?): Tokens on stack: ["{", "\\lx@hidden@bgroup", "\\lx@ | fixed, M3 F4, 1/E0 (better than #149) |
| pmhanguljamo/pmhanguljamo-kdoc | canary | 2/E5 | 2/E6 | errors | ? | Error:undefined:\luatexkomainhangulfamily The token T_CS[\luatexkomainhangulfamily] is not defined. | |
| prerex/chart | in | 0/E0 | 0/E0 | recall 100.0→51.0 % | R |  | fixed, M2 C7 |
| profsio/ProfSio-doc-fr | in | 1/E0 | 2/E10 | status,errors,recall-lost | ? | Error:expected:{ Missing { inserted | fixed, M2 C2 |
| pynotebook/pynotebook-examples-pitonpyluatex | canary | 2/E9 | 2/E10 | errors | ? | Error:latex:(PyLuaTeX) Package PyLuaTeX Error: Shell escape is required (add --shell-escape option)  | |
| serbian-def-cyr/proba | in | 0/E0 | 0/E0 | recall 100.0→90.3 % | R |  | residual, recall 19 |
| shipunov/sltables-doc-en | in | 1/E0 | 2/E11 | status,errors,recall-lost | ? | Error:unexpected:\omit \omit cannot be used here | fixed, M2 C1 |
| simpler-wick/simpler-wick | canary | 1/E0 | 2/E1 | status,errors,recall-lost | ? | Error:unexpected:fi Didn't expect a "T_CS[\\fi]" since we seem not to be in a conditional | fixed, M3 F10 |
| simples-matrices/simples-matrices-eng | in | 1/E0 | 1/E0 | invalid | V |  | fixed, M2 C6 |
| simples-matrices/simples-matrices-fra | in | 1/E0 | 1/E0 | invalid | V |  | fixed, M2 C6 |
| subsupscripts/SubSupScripts | in | 0/E0 | 2/E6 | status,errors,recall-lost | A | Error:undefined:\setbibname The token T_CS[\setbibname] is not defined. | fixed, M1 |
| tabularray/tabularray | in | 1/E0 | 1/E0 | recall 100.0→0.0 % | R |  | HTML pass, 100 % at M2 |
| textpos/niepraschk-eso-pic | in | 0/E0 | 0/E0 | recall 100.0→66.7 % | R |  | fixed, M2 C8 |
| tikz-feynman/tikz-feynman | canary | 2/E4 | 2/E5 | errors | ? | Error:unexpected:fi Didn't expect a "T_CS[\\fi]" since we seem not to be in a conditional | fixed, M3 F10, 2/E4 |
| tikz-trackschematic/tikz-trackschematic | canary | 3/E3 | 3/E101 | errors | ? | Error:undefined:\CommentEndDef The token T_CS[\CommentEndDef] is not defined. | fixed, M3 F2, 2/E1 |
| tikz-trackschematic/tikz-trackschematic-snippets | canary | 3/E3 | 3/E101 | errors | ? | Error:undefined:\CommentEndDef The token T_CS[\CommentEndDef] is not defined. | fixed, M3 F2, 2/E147: the manual's absent files |
| tikz3d-fr/tikz3d-fr-doc | canary | 1/E0 | 2/E20 | status,errors,recall-lost | ? | Error:expected:{ Missing { inserted | fixed, M2 C2 |
| tikzpingus/tikzpingus-doc | canary | 3/E17 | 124/E48 | status,errors | T | Error:unexpected:\@pingu@wingdown@0.0@ The control sequence "\\@pingu@wingdown@0.0@" should not appe | M3 F7: E0, 124 by time |
| tkz-grapheur/tkz-grapheur-doc-en | canary | 1/E0 | 1/E0 | recall 99.8→0.0 % | R |  | HTML pass, 99.8 % at M2 |
| tkz-grapheur/tkz-grapheur-doc-fr | canary | 1/E0 | 1/E0 | recall 99.8→0.0 % | R |  | HTML pass, 99.8 % at M2 |
| tutodoc/tutodoc-en | canary | 2/E2 | 2/E3 | errors | ? | Error:unexpected:} Attempt to close a group that switched to mode horizontal | fixed, M3 F8, 2/E2 |
| tutodoc/tutodoc-fr | canary | 2/E2 | 2/E3 | errors | ? | Error:unexpected:} Attempt to close a group that switched to mode horizontal | fixed, M3 F8, 2/E2 |
| tuzuk/oyd_tuzuk_2023 | canary | 0/E0 | 2/E2 | status,errors,recall-lost | ? | Error:unexpected:& Extra alignment tab '&' | |
| typog/typog-example | canary | 124/E45 | 124/E46 | errors | T | Error:latex:(typog) Package typog Error: Patching plain LaTeX macro \itemize failed         See the  | |
| tzplot/tzplot-doc | canary | 1/E0 | 124/E0 | status,recall-lost | T |  | |
| urcls/URpagestyles-DEMO | canary | 3/E4 | 2/E25 | errors | ? | Error:misdefined:# The token "#" (catcode PARAM) should never reach Stomach! | |
| wheelchart/wheelchart | in | 1/E0 | 124/E0 | status,recall-lost | T |  | |
| worldflags/worldflags | canary | 1/E0 | 124/E0 | status,recall-lost | T |  | |
| yquant/yquant-doc | canary | 2/E203 | 2/E206 | errors | ? | Error:latex:(etoolbox) Package etoolbox Error: \bmod not a macro         See the etoolbox package do | fixed, M3 F3, 2/E202 |

## Status 0 → 1, new Warnings only (25)

aiaa/template_basic, beamertheme-mirage/mirage-poster-en, biblatex-cse/biblatex-cse-doc, biblatex-trad/biblatex-trad, boldtensors/boldtensors, ctib/ctib4tex, ddphonism/ddphonism, frege/frege, grid/grid, intexgral/intexgral-en, intexgral/intexgral-fr, jourcl/jourcl, kdgdocs/kdgcoursetext-example, kdgdocs/kdgmasterthesis-example, montex/mlsquick, montex/montex, pecha/example, przechlewski-book/wkmgr1, przechlewski-book/wkmgr2, resphilosophica/rpsample, statex/statex-example, statex2/statex2-example, tabvar/demo, thaienum/thaienum, xskak/skak-comment-test.
