# TeX Live manuals: regressions to recover before the merge (living worklist)

**Why.** The user, 2026-11-06: "all regressions on the perfect_kernel texlive manuals need to be addressed before merge to main. This remains the primary goal of this branch." The bar is in [`../PERFECT_KERNEL.md`](../PERFECT_KERNEL.md) "Goal bar"; it was met at sweeps #146-148 and nearly at #149, and no sweep ran for 25 days and many bundles after that.

**How to run it.** `~/data/pk_agents/main/chain/manual_sweep.sh <N> <label> [cores] [jobs]`, then `tools/perfect_kernel/scoreboard.py <prev> <N>` and `--open <N>`. It now joins every bundle's checks, because the arXiv A/B and the binding nets do not see these regressions.

## The measure

Sweep #150 on `d6e175bacb` (binary int140a), 180 s, 8 GB, cores shared with two writers. 13 timeouts are provisional until re-run on quiet cores.

| Measure | #149 (2026-10-11, `e57be39640`) | #150 (2026-11-06) |
|---|---:|---:|
| Goal set 1,552, passing G1-G4 | 1,549 (99.81 %) | 1,527 (98.39 %) |
| G1 unclean / G2 invalid / G3 below 95 % / G4 over 180 s | 0 / 1 / 0 / 2 | 10 / 6 / 6 / 3 |
| In scope 1,602, clean | 1,590 (99.25 %) | 1,579 (98.56 %) |
| In scope, clean + schema-valid | 1,576 | 1,560 (97.38 %) |
| In scope, clean + valid + raw recall ≥ 95 %, no classification | 1,363 (85.08 %) | 1,350 (84.27 %) |
| Whole corpus 2,374: clean; no Fatal or timeout | | 1,940 (81.72 %); 2,306 (97.14 %) |

Vs #149: status worse on 53 manuals, better on 220; errors 11,246 → 11,278, up on 44, down on 85. **None comes from Bundle int140a:** 21 of them run on int139c give the same status and error count.

## The clusters

- **A, a binding's name prefix captures a different class. Root-caused.** `content.rs` `raw_excluded`: under raw classes, a class with a prefix-matching binding is not read raw (OD #444, for an author's renamed copy: `IEEEtranTCOM.cls`, `IEEEtran4PSCC.cls`). It also fires on a published class of its own: `bookest` → `book`, `bookcover` → `book`, `beamerswitch` → `beamer`, `bookshelf` → `book`; so their commands are undefined. Sweep #149's log read them raw. TeX Live classes the prefix catches (16): `aalok`, `aastex701`, `acmart-tagged`, `anima`, `apa6e`, `articleingud`, `beamer-rl`, `beamerswitch`, `bookcover`, `bookest`, `bookshelf`, `jmlrbook`, `postcards`, `reporting`, `revtex4-1`, `revtex4-2`.
- **B, `\pdftex_if_engine:TF` undefined** (mdframed, 4 manuals, E about 290 each): not yet root-caused.
- **T, timeouts:** re-run on quiet cores first; several may be load. `cpu_h` rose 0.92 → 1.20 in scope, partly load.
- **V, newly schema-invalid** (annee-scolaire, paresse, simples-matrices, the same author's family, 2 jing lines each): not yet root-caused.
- **R, recall** (LaTeX_RefSheet, prerex/chart, quantum-bibliographystyle-demo, serbian-def-cyr/proba, niepraschk-eso-pic; tabularray is its HTML pass, to re-check): not yet root-caused.
- **?, the rest:** each row's first error below is the lead.

**Also watch:** 25 manuals went status 0 → 1, new Warnings only, listed at the end; read them after the Errors, since 108b's and 108a's counted Warnings are intended. Japanese `jsarticle` manuals moved to OmniBus by `declared_other_format`, intended and out of scope.

## The worklist (non-warning rows, 66)

Status / errors at #149 → #150, in scope or canary. `in` manuals count against the bar; canaries count for a new Fatal or timeout.

| Manual | Scope | #149 | #150 | What regressed | Cluster | First error at #150 | Done |
|---|---|---|---|---|---|---|---|
| annee-scolaire/annee-scolaire-eng | in | 1/E0 | 1/E0 | invalid | V |  | |
| annee-scolaire/annee-scolaire-fra | in | 1/E0 | 1/E0 | invalid | V |  | |
| beamerswitch/beamerswitch-example | in | 1/E0 | 2/E2 | status,errors,recall-lost | A | Error:undefined:\handoutlayout The token T_CS[\handoutlayout] is not defined. | |
| biblatex-arthistory-bonn/arthistory-bonn | canary | 3/E7 | 124/E6 | status | T | Error:missing_file:arthistory-bonn-preamble.tex Can't find TeX file arthistory-bonn-preamble.tex | |
| biblatex-ms/biblatex-ms | canary | 2/E2 | 2/E3 | errors | ? | Error:undefined:{release} The environment {release} is not defined. | |
| bibleref-german/de-bibleref-german | canary | 2/E1 | 3/E0 | status,invalid | ? | Fatal:Timeout:Recursion  Infinite expansion loop: a window of 2 token(s) repeated 100+ times | |
| bibleref-german/en-bibleref-german | canary | 2/E1 | 3/E0 | status,invalid | ? | Fatal:Timeout:Recursion  Infinite expansion loop: a window of 2 token(s) repeated 100+ times | |
| bookcover/bookcover-example1 | canary | 0/E0 | 2/E5 | status,errors,recall-lost | A | Error:undefined:\newbookcovercomponenttype The token T_CS[\newbookcovercomponenttype] is not defined | |
| bookcover/bookcover-example2 | canary | 0/E0 | 2/E2 | status,errors,recall-lost | A | Error:undefined:{bookcover} The environment {bookcover} is not defined. | |
| bookest/bookestdoc-en | in | 0/E0 | 2/E5 | status,errors,recall-lost | A | Error:undefined:\setoddheadtext The token T_CS[\setoddheadtext] is not defined. | |
| bookest/bookestdoc-it | in | 1/E0 | 2/E4 | status,errors,recall-lost | A | Error:undefined:\setoddheadtext The token T_CS[\setoddheadtext] is not defined. | |
| bracketkey/Malva | canary | 0/E0 | 2/E61 | status,errors,recall-lost | ? | Error:unexpected:T_CS[\theindex] \theindex' invalid at this point | |
| cmpj/template | in | 1/E0 | 1/E0 | recall-drop 98.9→96.0 % | R |  | |
| colorspace/colorspace | in | 0/E0 | 2/E8 | status,errors,recall-lost | ? | Error:unexpected:fooshaded Unrecognized color model 'fooshaded' | |
| currency/currency_doc | canary | 2/E39 | 2/E41 | errors | ? | Error:undefined:\href The token T_CS[\href] is not defined. | |
| datetime2-english/datetime2-english-sample | in | 0/E0 | 0/E0 | recall-drop 100.0→96.6 % | R |  | |
| draftfigure/draftfigure | canary | 2/E24 | 2/E25 | errors | ? | Error:missing_file:draftfigure-preamble.tex Can't find TeX file draftfigure-preamble.tex | |
| epslatex-fr/fepslatex | canary | 2/E10 | 2/E11 | errors | ? | Error:expected:{ Missing { inserted | |
| floatrow/floatrow-rus | canary | 3/E101 | 124/E52 | status | T | Error:latex:(caption) Package caption Error: The counter subtable' was already defined by | |
| gtrcrd/gtrcrd-doc | in | 0/E0 | 2/E6 | status,errors,recall-lost | A | Error:undefined:\setleftmark The token T_CS[\setleftmark] is not defined. | |
| ijsra/ijsra | in | 1/E0 | 2/E1 | status,errors,recall-lost | ? | Error:unexpected:\endlx@list Attempt to end mode internal_vertical | |
| keytheorems/keytheorems-doc | in | 1/E0 | 2/E4 | status,errors,recall-lost | ? | Error:errmessage:\errmessage LaTeX template Error: The instance 'theoremblock-0' of type 'block' is | |
| kotex-oblivoir/ob-mathleading-doc | canary | 2/E4 | 2/E5 | errors | ? | Error:undefined:\luatexkomainhangulfamily The token T_CS[\luatexkomainhangulfamily] is not defined. | |
| lineno/fnlineno | canary | 0/E0 | 2/E1 | status,errors,recall-lost | ? | Error:undefined:\3 The token T_CS[\3] is not defined. | |
| ltxkeys/ltxkeys-guide | canary | 3/E27 | 3/E31 | errors | ? | Error:undefined:\ltscomment The token T_CS[\ltscomment] is not defined. | |
| mathfont/mathfont-example-typey | canary | 2/E9 | 2/E10 | errors | ? | Error:latex:(mathfont) Package mathfont Error: | |
| mdframed/mdframed-example-default | canary | 2/E44 | 2/E295 | errors | B | Error:undefined:\pdftex_if_engine:TF The token T_CS[\pdftex_if_engine:TF] is not defined. | |
| mdframed/mdframed-example-pstricks | canary | 2/E44 | 2/E49 | errors | B | Error:undefined:\pdftex_if_engine:TF The token T_CS[\pdftex_if_engine:TF] is not defined. | |
| mdframed/mdframed-example-texsx | canary | 2/E44 | 2/E292 | errors | B | Error:undefined:\pdftex_if_engine:TF The token T_CS[\pdftex_if_engine:TF] is not defined. | |
| mdframed/mdframed-example-tikz | canary | 2/E44 | 2/E290 | errors | B | Error:undefined:\pdftex_if_engine:TF The token T_CS[\pdftex_if_engine:TF] is not defined. | |
| membranecomputing/membranecomputing | canary | 1/E0 | 2/E8 | status,errors,recall-lost | ? | Error:expected:{ Missing { inserted | |
| memorygraphs/memorygraphs | canary | 2/E4 | 2/E5 | errors | ? | Error:latex:(latexgit) Package latexgit Error: invoke LaTeX with the -shell-escape flag         See  | |
| ndsu-thesis-2022/NDSU-Thesis-Extended | canary | 2/E1 | 2/E2 | errors | ? | Error:undefined:\iftotaltables The token T_CS[\iftotaltables] is not defined. | |
| ndsu-thesis-2022/ndsu-example | canary | 1/E0 | 2/E1 | status,errors,recall-lost | ? | Error:undefined:\iftotaltables The token T_CS[\iftotaltables] is not defined. | |
| non-decimal-units/non-decimal-units | canary | 2/E1 | 2/E22 | errors | ? | Error:latex:(csvsimple) Package csvsimple Error: File 'unicode-alphabets.MUFI    .csv' not existent, | |
| paresse/paresse-eng | in | 1/E0 | 1/E0 | invalid | V |  | |
| paresse/paresse-fra | in | 1/E0 | 1/E0 | invalid | V |  | |
| plweb/pl | canary | 2/E12 | 3/E0 | status | ? | Fatal:Stomach:Recursion  Excessive recursion(?): Tokens on stack: ["{", "\\lx@hidden@bgroup", "\\lx@ | |
| pmhanguljamo/pmhanguljamo-kdoc | canary | 2/E5 | 2/E6 | errors | ? | Error:undefined:\luatexkomainhangulfamily The token T_CS[\luatexkomainhangulfamily] is not defined. | |
| prerex/chart | in | 0/E0 | 0/E0 | recall 100.0→51.0 % | R |  | |
| profsio/ProfSio-doc-fr | in | 1/E0 | 2/E10 | status,errors,recall-lost | ? | Error:expected:{ Missing { inserted | |
| pynotebook/pynotebook-examples-pitonpyluatex | canary | 2/E9 | 2/E10 | errors | ? | Error:latex:(PyLuaTeX) Package PyLuaTeX Error: Shell escape is required (add --shell-escape option)  | |
| serbian-def-cyr/proba | in | 0/E0 | 0/E0 | recall 100.0→90.3 % | R |  | |
| shipunov/sltables-doc-en | in | 1/E0 | 2/E11 | status,errors,recall-lost | ? | Error:unexpected:\omit \omit cannot be used here | |
| simpler-wick/simpler-wick | canary | 1/E0 | 2/E1 | status,errors,recall-lost | ? | Error:unexpected:fi Didn't expect a "T_CS[\\fi]" since we seem not to be in a conditional | |
| simples-matrices/simples-matrices-eng | in | 1/E0 | 1/E0 | invalid | V |  | |
| simples-matrices/simples-matrices-fra | in | 1/E0 | 1/E0 | invalid | V |  | |
| subsupscripts/SubSupScripts | in | 0/E0 | 2/E6 | status,errors,recall-lost | A | Error:undefined:\setbibname The token T_CS[\setbibname] is not defined. | |
| tabularray/tabularray | in | 1/E0 | 1/E0 | recall 100.0→0.0 % | R |  | |
| textpos/niepraschk-eso-pic | in | 0/E0 | 0/E0 | recall 100.0→66.7 % | R |  | |
| tikz-feynman/tikz-feynman | canary | 2/E4 | 2/E5 | errors | ? | Error:unexpected:fi Didn't expect a "T_CS[\\fi]" since we seem not to be in a conditional | |
| tikz-trackschematic/tikz-trackschematic | canary | 3/E3 | 3/E101 | errors | ? | Error:undefined:\CommentEndDef The token T_CS[\CommentEndDef] is not defined. | |
| tikz-trackschematic/tikz-trackschematic-snippets | canary | 3/E3 | 3/E101 | errors | ? | Error:undefined:\CommentEndDef The token T_CS[\CommentEndDef] is not defined. | |
| tikz3d-fr/tikz3d-fr-doc | canary | 1/E0 | 2/E20 | status,errors,recall-lost | ? | Error:expected:{ Missing { inserted | |
| tikzpingus/tikzpingus-doc | canary | 3/E17 | 124/E48 | status,errors | T | Error:unexpected:\@pingu@wingdown@0.0@ The control sequence "\\@pingu@wingdown@0.0@" should not appe | |
| tkz-grapheur/tkz-grapheur-doc-en | canary | 1/E0 | 1/E0 | recall 99.8→0.0 % | R |  | |
| tkz-grapheur/tkz-grapheur-doc-fr | canary | 1/E0 | 1/E0 | recall 99.8→0.0 % | R |  | |
| tutodoc/tutodoc-en | canary | 2/E2 | 2/E3 | errors | ? | Error:unexpected:} Attempt to close a group that switched to mode horizontal | |
| tutodoc/tutodoc-fr | canary | 2/E2 | 2/E3 | errors | ? | Error:unexpected:} Attempt to close a group that switched to mode horizontal | |
| tuzuk/oyd_tuzuk_2023 | canary | 0/E0 | 2/E2 | status,errors,recall-lost | ? | Error:unexpected:& Extra alignment tab '&' | |
| typog/typog-example | canary | 124/E45 | 124/E46 | errors | T | Error:latex:(typog) Package typog Error: Patching plain LaTeX macro \itemize failed         See the  | |
| tzplot/tzplot-doc | canary | 1/E0 | 124/E0 | status,recall-lost | T |  | |
| urcls/URpagestyles-DEMO | canary | 3/E4 | 2/E25 | errors | ? | Error:misdefined:# The token "#" (catcode PARAM) should never reach Stomach! | |
| wheelchart/wheelchart | in | 1/E0 | 124/E0 | status,recall-lost | T |  | |
| worldflags/worldflags | canary | 1/E0 | 124/E0 | status,recall-lost | T |  | |
| yquant/yquant-doc | canary | 2/E203 | 2/E206 | errors | ? | Error:latex:(etoolbox) Package etoolbox Error: \bmod not a macro         See the etoolbox package do | |

## Status 0 → 1, new Warnings only (25)

aiaa/template_basic, beamertheme-mirage/mirage-poster-en, biblatex-cse/biblatex-cse-doc, biblatex-trad/biblatex-trad, boldtensors/boldtensors, ctib/ctib4tex, ddphonism/ddphonism, frege/frege, grid/grid, intexgral/intexgral-en, intexgral/intexgral-fr, jourcl/jourcl, kdgdocs/kdgcoursetext-example, kdgdocs/kdgmasterthesis-example, montex/mlsquick, montex/montex, pecha/example, przechlewski-book/wkmgr1, przechlewski-book/wkmgr2, resphilosophica/rpsample, statex/statex-example, statex2/statex2-example, tabvar/demo, thaienum/thaienum, xskak/skak-comment-test.
