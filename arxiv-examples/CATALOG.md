# arxiv-examples Conversion Catalog

Updated 2026-04-05 (session 93). 47 papers tested with latexml-oxide (Rust) and latexmlc (Perl).
Both use `--format=html5 --nodefaultresources --preload=ar5iv.sty` + ar5iv CSS CDN links.
Rust uses `bibconfig=bbl,bib` fallback (bbl preferred, raw .bib as fallback via native Rust BibTeX parser).

## Summary

- **37/47 OK** (79%) -- produce meaningful Rust HTML5 output
- **8 EMPTY** -- produce minimal output (cascading errors or TooManyErrors)
- **2 FAIL** -- timeout (no output)
- **27/37 >=90% size parity** with Perl (73% of OK)
- **30/37 >=80% size parity** (81% of OK)
- **10 papers Rust > Perl size** (27%) -- more resolved bibliography content
- Perl HTML regenerated 2026-04-05 with correct flags (`--nodefaultresources` + ar5iv CSS)
- Rust HTML regenerated 2026-04-05 with `--css` ar5iv CDN links + `bibconfig=bbl,bib` fix
- **Session 93**: algorithm2e fixes (BlankLine 1ex, pop@indentation, vertical bars), bibconfig=bbl,bib parity, CSS injection. All Rust HTML regenerated.
- **Session 92b**: Bibliography content fix — cross-document XPath bug in `make_bibliography.rs` caused all `.bib`-sourced entries to show only "Cited by" with no author/title/journal. Fixed with `findnodes_foreign` traversal.
- **Session 92**: Fresh visual comparison, authblk/elsart fixes, end_mode recovery (2508.18544: 43%→88%).

## Results

| Paper ID | Status | Rust | Perl | Ratio | Main File | Notes | Visual (2026-04-05) |
|----------|--------|------|------|-------|-----------|-------|---------------------|
| 0710.2281 | OK | 757KB | 748KB | 101% | paper.tex | 31 bibitems (bbl) | IDENTICAL |
| 1312.5845 | OK | 35KB | 37KB | 94% | iclr_tshino_2014_v5.tex | 14 bibitems | IDENTICAL |
| 1502.04955 | OK | 302KB | 511KB | 59% | paper.tex | 15 missing cites; no .bib/.bbl | near-identical; Rust shows date |
| 1706.03762 | OK | 128KB | 140KB | 91% | ms.tex | 40 bibitems, zero errors | IDENTICAL |
| 1907.08050 | OK | 1218KB | 1269KB | 95% | paper.tex | 32 bibitems | BUG: empty abstract |
| 1910.06709 | OK | 61KB | 62KB | 98% | paper.tex | 27 bibitems | IDENTICAL |
| 2005.13625 | EMPTY | 0KB | 990KB | 0% | main.tex | pgf boxing group mismatch | N/A |
| 2008.08932 | OK | 18KB | 19KB | 94% | main.tex | 10 bibitems | BUG: empty abstract, missing authors |
| 2101.00726 | OK | 630KB | 650KB | 96% | wasserstein_arXiv_v2.tex | 49 bibitems | IDENTICAL |
| 2103.01205 | FAIL | 0KB | 497KB | 0% | main.tex | Timeout (pgf/tikz) | N/A |
| 2209.14198 | EMPTY | 0KB | 721KB | 0% | gucycles.tex | pgf arrow 'Stealth' | N/A |
| 2306.00809 | OK | 141KB | 140KB | 100% | backup.tex | 17 missing cites; no .bib/.bbl | IDENTICAL |
| 2306.06628 | OK | 241KB | 223KB | 107% | Contraction20.tex | 33 missing cites | IDENTICAL |
| 2308.06254 | OK | 270KB | 281KB | 96% | main.tex | cleveref+enumitem; biblatex | Rust CLEANER (Perl has red error) |
| 2310.18318 | OK | 334KB | 371KB | 89% | Hyperon-Sep-2023.tex | 52 bibitems | IDENTICAL |
| 2401.08110 | OK | 1172KB | 1205KB | 97% | errorsInHybridQST_arXiv2.tex | 190 bibitems | Rust CORRECT (Perl heading broken) |
| 2401.18036 | OK | 167KB | 173KB | 96% | manuscript.tex | 64 bibitems (bbl) | cosmetic: author layout diff |
| 2401.18052 | OK | 155KB | 164KB | 94% | ms_feII.tex | 77 bibitems (bbl) | Perl has blue author links |
| 2402.03300 | EMPTY | 0KB | 393KB | 0% | main.tex | pgfkeys recursion | N/A |
| 2402.10301 | OK | 1800KB | 1454KB | 124% | paper.tex | tikz-cd FIXED (S94); 0 errors | Rust >Perl; full content |
| 2403.07652 | OK | 104KB | 111KB | 93% | acl_latex.tex | 28 bibitems (bbl) | IDENTICAL |
| 2403.15796 | OK | 4KB | 4KB | 95% | 0_main.tex | Perl has wrong main file | near-identical (logo size diff) |
| 2405.17032 | OK | 301KB | 793KB | 38% | ms.tex | 15 missing; tikz figs missing | p1 near-identical; tikz missing deeper |
| 2405.19425 | OK | 434KB | 490KB | 88% | main.tex | 80 bibitems; page=N PDF fix (S94) | IDENTICAL; all 10 images render |
| 2406.06608 | OK | 773KB | 739KB | 104% | main.tex | 373 bibitems | near-identical; author spacing |
| 2408.11158 | OK | 64KB | 68KB | 93% | aipsamp.tex | 27 bibitems (bbl) | IDENTICAL |
| 2408.13687 | OK | 105KB | 109KB | 97% | main.tex | 60 bibitems (bbl) | cosmetic: date, citation format |
| 2410.10068 | EMPTY | 0KB | 975KB | 0% | main.tex | tikz-cd + pgf arrows | N/A |
| 2410.12896 | OK | 453KB | 527KB | 85% | sample-manuscript.tex | 277 bibitems (bbl) | IDENTICAL |
| 2502.04134 | OK | 112KB | 124KB | 90% | iclr2025_conference.tex | 20 bibitems (bbl) | IDENTICAL |
| 2503.08256 | OK | 166KB | 215KB | 77% | main.tex | 62 bibitems | cosmetic: param leaks at top |
| 2506.03074 | OK | 1242KB | 1261KB | 98% | _main.tex | 177 bibitems (bbl) | near-identical; citation style |
| 2507.23241 | EMPTY | 0KB | 1020KB | 0% | main.tex | smfart.cls + expl3 timing | N/A |
| 2508.15260 | OK | 376KB | 369KB | 102% | main.tex | tcolorbox FIXED (S94 quark fixups) | NEW: was EMPTY |
| 2508.18544 | OK | 714KB | 856KB | 83% | Main_Communi_submit.tex | 56 bibitems (bbl); algo2e+affil fixed | IDENTICAL |
| 2509.18103 | OK | 197KB | 275KB | 71% | main.tex | 10 bibitems; Perl HTML larger | IDENTICAL (content parity) |
| 2511.03798 | EMPTY | 0KB | 70KB | 0% | deSitter_resurgence_I.tex | eqnarray recursion; Perl fails too | N/A |
| 2511.11713 | OK | 193KB | 106KB | 181% | IEEE-conference-template-062824.tex | 59 bibitems via .bib | IDENTICAL |
| 2511.14458 | OK | 309KB | 196KB | 157% | main_mattille.tex | 57 bibitems via .bib | cosmetic: numeric affil prefixes |
| 2511.15304 | OK | 141KB | 134KB | 105% | main.tex | 12 missing cites | IDENTICAL |
| 2512.09456 | OK | 200KB | 97KB | 205% | Main.tex | 30 missing; inline bib | IDENTICAL |
| 2512.16911 | OK | 704KB | 635KB | 110% | main.tex | 36 missing cites | IDENTICAL |
| 2602.18719 | OK | 677KB | 557KB | 119% | CDKU.tex | tikz-cd FIXED (S94); 17 xref errs | Rust >Perl size; full content |
| 2602.23324 | OK | 672KB | 659KB | 102% | main.tex | 24 missing cites | IDENTICAL |
| 2603.14602 | OK | 337KB | 347KB | 97% | main.tex | tcolorbox breakable FIXED (S94) | NEW: was EMPTY |
| 2603.15617 | OK | 1163KB | 1218KB | 95% | paper.tex | tcolorbox breakable FIXED (S94) | NEW: was 3% truncated |
| 2603.19312 | OK | 337KB | 294KB | 114% | main.tex | 31 missing cites | IDENTICAL |

## Failure Analysis

### FAIL (2 papers)
- **2103.01205**: pgf/tikz timeout (>120s)
- **2402.10301**: pgf 'Computer Modern Rightarrow' arrow + cascading, timeout

### EMPTY (8 papers)
- **2005.13625**: pgf `\pgf@x` number parsing, boxing group mismatch
- **2209.14198**: pgf arrow 'Stealth' undefined, cascading to token limit
- **2402.03300**: `\pgfkeys@mainstop` recursive self-expansion loop
- **2410.10068**: tikz-cd matrix processing, pgf arrows
- **2507.23241**: smfart.cls raw TeX + expl3 timing
- **2508.15260**: tcolorbox cascading errors (Perl also only 1KB)
- **2511.03798**: `\@@eqnarray` recursion in jheppub (Perl also fails)
- **2603.14602**: minted/listing parameter errors, TooManyErrors

### Size parity analysis (37 OK papers, session 93 final)
- **>=90% parity**: 27 papers (73%)
- **>100% (Rust larger)**: 10 papers (27%)
- **80-89%**: 3 papers (8%)
- **70-79%**: 2 papers (5%) -- content-identical (Perl HTML verbosity)
- **50-69%**: 2 papers (5%) -- listing style gap
- **<50%**: 3 papers (8%) -- tikz/listing/mode errors
- **>=80% parity**: 30 papers (81%)
- **Note:** With bibconfig=bbl,bib fix, papers with .bbl files now use BBL ordering matching Perl. Papers without .bbl fall back to .bib parser.

### Root causes of remaining gaps
1. **Missing citations (no .bib/.bbl)** -- papers using `\thebibliography` inline or missing source files
2. **Listing per-word styling** -- Perl wraps each listing token in styled `<span>` (2405.19425 gap)
3. **pgf arrow tips** (Stealth, Circle, Hooks, Implies) -- deep pgfkeys infrastructure
4. **tikz-cd matrix** -- `\tikzcdmatrixname` shape processing (2602.18719)
5. **tikzpicture mode corruption** -- failed tikz commands corrupt parser mode (2603.15617)
6. **Raw affiliation parameters** -- `[inst]organization=...` leaked (2508.18544 elsart/cas)

### Visual comparison summary (2026-04-05, session 93 final, CSS + BBL + affil fix)
- **21/37 IDENTICAL** on first-page screenshot (57%) -- 2405.19425 listing fix, 2508.18544 affil fix
- **9/37 near-identical / cosmetic** (24%) -- author layout, date, spacing, citation format
- **2/37 Rust BETTER** (5%) -- 2308.06254 (cleaner), 2401.08110 (correct section headings)
- **2/37 CRITICAL** (5%) -- body content truncated (tikz corruption: 2602.18719, 2603.15617)

### Actionable bugs (session 93)
1. **2602.18719**: Body truncated (34KB vs 557KB) -- `\lxSVG@halign` unimplemented, tikz-cd matrix
2. **2603.15617**: Body truncated (35KB vs 1189KB) -- tikzpicture `\node`/`\draw` undefined
3. **pgf arrow tips**: 'Computer Modern Rightarrow', 'Hooks', 'Implies', 'Circle' undefined

### Permanent ignores (5)
- **ns1-ns5** (52_namespace) -- DTD not supported in Rust port.
