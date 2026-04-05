# arxiv-examples Conversion Catalog

Updated 2026-04-05 (session 92). 47 papers tested with latexml-oxide (Rust) and latexmlc (Perl).
Both use `--format=html5 --nodefaultresources --preload=ar5iv.sty` + ar5iv CSS CDN links.
Rust uses `bibconfig=bbl,bib` fallback (bbl preferred, raw .bib as fallback via native Rust BibTeX parser).

## Summary

- **37/47 OK** (79%) -- produce meaningful Rust HTML5 output
- **8 EMPTY** -- produce minimal output (cascading errors or TooManyErrors)
- **2 FAIL** -- timeout (no output)
- **27/37 >=90% size parity** with Perl (73% of OK) -- up from 26
- **30/37 >=80% size parity** (81% of OK) -- up from 29
- **9 papers Rust > Perl size** (more resolved content)
- Perl HTML regenerated 2026-04-05 with correct flags (`--nodefaultresources` + ar5iv CSS)
- **Session 92**: Fresh visual comparison with regenerated Rust HTML. Significant size improvements in 2405.19425 (+38pp), 2410.12896 (+16pp), 2511.11713 (+50pp), 2512.09456 (+40pp).

## Results

| Paper ID | Status | Rust | Perl | Ratio | Main File | Notes | Visual (2026-04-05) |
|----------|--------|------|------|-------|-----------|-------|---------------------|
| 0710.2281 | OK | 757KB | 748KB | 101% | paper.tex | 31 bibitems resolved | IDENTICAL |
| 1312.5845 | OK | 36KB | 38KB | 94% | iclr_tshino_2014_v5.tex | 14 bibitems | IDENTICAL |
| 1502.04955 | OK | 302KB | 511KB | 59% | paper.tex | 15 missing cites; no .bib/.bbl | p1 near-identical; Rust shows date |
| 1706.03762 | OK | 129KB | 141KB | 91% | ms.tex | 40 bibitems, zero errors | IDENTICAL |
| 1907.08050 | OK | 1218KB | 1270KB | 95% | paper.tex | 32 bibitems | IDENTICAL |
| 1910.06709 | OK | 61KB | 62KB | 98% | paper.tex | 27 bibitems | IDENTICAL |
| 2005.13625 | EMPTY | 0KB | 990KB | 0% | main.tex | pgf boxing group mismatch | N/A |
| 2008.08932 | OK | 18KB | 19KB | 94% | main.tex | 10 bibitems | IDENTICAL |
| 2101.00726 | OK | 625KB | 650KB | 96% | wasserstein_arXiv_v2.tex | 49 bibitems | IDENTICAL |
| 2103.01205 | FAIL | 0KB | 497KB | 0% | main.tex | Timeout (pgf/tikz) | N/A |
| 2209.14198 | EMPTY | 0KB | 721KB | 0% | gucycles.tex | pgf arrow 'Stealth' | N/A |
| 2306.00809 | OK | 142KB | 141KB | 100% | backup.tex | 17 missing cites; no .bib/.bbl | IDENTICAL |
| 2306.06628 | OK | 215KB | 224KB | 96% | Contraction20.tex | 33 missing cites | IDENTICAL |
| 2308.06254 | OK | 271KB | 281KB | 96% | main.tex | cleveref+enumitem fixed | Rust CLEANER (Perl has extra red error) |
| 2310.18318 | OK | 334KB | 372KB | 89% | Hyperon-Sep-2023.tex | 52 bibitems | IDENTICAL |
| 2401.08110 | OK | 1172KB | 1205KB | 97% | errorsInHybridQST_arXiv2.tex | 190 bibitems | Rust CORRECT (Perl section heading broken) |
| 2401.18036 | OK | 170KB | 174KB | 97% | manuscript.tex | 64 bibitems | cosmetic author layout diff |
| 2401.18052 | OK | 168KB | 165KB | 102% | ms_feII.tex | 77 bibitems | Perl has blue author links |
| 2402.03300 | EMPTY | 0KB | 393KB | 0% | main.tex | pgfkeys recursion | N/A |
| 2402.10301 | FAIL | 0KB | 1421KB | 0% | paper.tex | Timeout (pgf arrows) | N/A |
| 2403.07652 | OK | 101KB | 111KB | 90% | acl_latex.tex | 28 bibitems | cosmetic: Gamma vs envelope symbol |
| 2403.15796 | OK | 4KB | 4KB | 95% | 0_main.tex | Perl has wrong main file | Near-identical (logo size diff) |
| 2405.17032 | OK | 302KB | 794KB | 38% | ms.tex | 15 missing; tikz figs missing | p1 near-identical; tikz figs missing deeper |
| 2405.19425 | OK | 425KB | 479KB | 88% | main.tex | 80 bibitems; gap=listing style | p1 IDENTICAL |
| 2406.06608 | OK | 732KB | 740KB | 98% | main.tex | 373 bibitems | near-identical; author spacing |
| 2408.11158 | OK | 70KB | 69KB | 101% | aipsamp.tex | 27 bibitems, 1 missing | IDENTICAL |
| 2408.13687 | OK | 109KB | 109KB | 99% | main.tex | 60 bibitems | cosmetic: date, citation format |
| 2410.10068 | EMPTY | 0KB | 975KB | 0% | main.tex | tikz-cd + pgf arrows | N/A |
| 2410.12896 | OK | 539KB | 527KB | 102% | sample-manuscript.tex | 277 bibitems, 1 missing | IDENTICAL |
| 2502.04134 | OK | 110KB | 125KB | 88% | iclr2025_conference.tex | 20 bibitems | IDENTICAL |
| 2503.08256 | OK | 153KB | 216KB | 70% | main.tex | 62 bibitems; Perl HTML larger | cosmetic: different param leaks at top |
| 2506.03074 | OK | 1299KB | 1262KB | 102% | _main.tex | 177 bibitems | near-identical; citation style (numeric vs author-year) |
| 2507.23241 | EMPTY | 0KB | 1020KB | 0% | main.tex | smfart.cls + expl3 timing | N/A |
| 2508.15260 | EMPTY | 0KB | 360KB | 0% | main.tex | tcolorbox; Perl also fails | N/A |
| 2508.18544 | OK | 729KB | 856KB | 85% | Main_Communi_submit.tex | 56 bibitems; end_mode recovery | affil key-val text; Conclusion+Appendix+Bib now present |
| 2509.18103 | OK | 197KB | 276KB | 71% | main.tex | 10 bibitems; Perl HTML larger | IDENTICAL (content parity) |
| 2511.03798 | EMPTY | 0KB | 70KB | 0% | deSitter_resurgence_I.tex | eqnarray recursion; Perl fails too | N/A |
| 2511.11713 | OK | 148KB | 107KB | 138% | IEEE-conference-template.tex | 76 missing; no .bib/.bbl | IDENTICAL |
| 2511.14458 | OK | 221KB | 196KB | 112% | main_mattille.tex | **57 bibitems via .bib parser** | cosmetic: numeric affil prefixes |
| 2511.15304 | OK | 122KB | 134KB | 91% | main.tex | 12 missing cites | IDENTICAL |
| 2512.09456 | OK | 137KB | 98KB | 140% | Main.tex | 30 missing; inline bib | IDENTICAL |
| 2512.16911 | OK | 640KB | 635KB | 100% | main.tex | 36 missing cites | IDENTICAL |
| 2602.18719 | OK | 35KB | 557KB | 6% | CDKU.tex | tikz-cd errors (182) | CRITICAL: body truncated; citation format diff |
| 2602.23324 | OK | 641KB | 659KB | 97% | main.tex | 24 missing cites | IDENTICAL |
| 2603.14602 | EMPTY | 0KB | 339KB | 0% | main.tex | listing/minted errors | N/A |
| 2603.15617 | OK | 35KB | 1190KB | 3% | paper.tex | tikzpicture mode corruption | CRITICAL: body truncated; affil0 placeholder bug |
| 2603.19312 | OK | 278KB | 294KB | 102% | main.tex | 31 missing cites | IDENTICAL |

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

### Size parity analysis (37 OK papers, session 92)
- **>=90% parity**: 27 papers (73%) -- up from 26
- **>100% (Rust larger)**: 9 papers (24%) -- Rust has more resolved content
- **80-89%**: 3 papers (8%)
- **70-79%**: 2 papers (5%) -- content-identical (Perl HTML verbosity)
- **50-69%**: 1 paper (3%)
- **<50%**: 4 papers (11%) -- tikz/listing/mode errors
- **>=90% including >100%**: 27 papers (73%), **>=80%**: 30 papers (81%)
- **Note:** size ratio can be misleading. Papers with identical paragraph/section/math counts may show <80% ratio due to Perl HTML attribute verbosity.

### Root causes of remaining gaps
1. **Missing citations (no .bib/.bbl)** -- papers using `\thebibliography` inline or missing source files (13 papers with missing_citation)
2. **Listing per-word styling** -- Perl wraps each listing token in styled `<span>`
3. **shortstack/vtop mode cascade** -- DefConstructor bounded+mode interaction
4. **pgf arrow tips** (Stealth, Circle, Hooks, Implies) -- deep pgfkeys infrastructure
5. **tikz-cd matrix** -- `\tikzcdmatrixname` shape processing
6. **tikzpicture mode corruption** -- failed tikz commands corrupt parser mode
7. **affil0 placeholder** -- `\affil` parameter not resolved (2603.15617)
8. **Raw affiliation parameters** -- `[inst]organization=...` leaked (2508.18544)

### Visual comparison summary (2026-04-05, session 92, fresh screenshots)
- **18/37 IDENTICAL** on first-page screenshot (49%) -- up from 11
- **11/37 near-identical / cosmetic** (30%) -- author layout, date, spacing, citation format
- **2/37 Rust BETTER** (5%) -- 2308.06254 (cleaner), 2401.08110 (correct section headings)
- **2/37 BUG** (5%) -- 2508.18544 (raw affil params), 2603.15617 (affil0 placeholder)
- **2/37 CRITICAL** (5%) -- body content truncated (tikz corruption, tikz-cd)
- **2/37 N/A** -- EMPTY/incomparable

### Actionable bugs found (session 92)
1. **2603.15617**: `\affil` shows "affil0" placeholder instead of resolved text
2. **2508.18544**: Raw `[inst1]organization=...` affiliation parameters leak into body; missing keywords/PACS metadata

### Permanent ignores (5)
- **ns1-ns5** (52_namespace) -- DTD not supported in Rust port.
