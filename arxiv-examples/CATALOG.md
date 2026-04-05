# arxiv-examples Conversion Catalog

Updated 2026-04-05 (session 93). 47 papers tested with latexml-oxide (Rust) and latexmlc (Perl).
Both use `--format=html5 --nodefaultresources --preload=ar5iv.sty` + ar5iv CSS CDN links.
Rust uses `bibconfig=bbl,bib` fallback (bbl preferred, raw .bib as fallback via native Rust BibTeX parser).

## Summary

- **37/47 OK** (79%) -- produce meaningful Rust HTML5 output
- **8 EMPTY** -- produce minimal output (cascading errors or TooManyErrors)
- **2 FAIL** -- timeout (no output)
- **29/37 >=90% size parity** with Perl (78% of OK) -- up from 27
- **31/37 >=80% size parity** (84% of OK) -- up from 30
- **18 papers Rust > Perl size** (48%) -- full bibliography content now resolved
- Perl HTML regenerated 2026-04-05 with correct flags (`--nodefaultresources` + ar5iv CSS)
- **Session 93**: algorithm2e `\BlankLine` "1ex" text leak fix, `\lx@algo@pop@indentation` implemented. All Rust HTML regenerated with fresh sizes.
- **Session 92b**: Bibliography content fix — cross-document XPath bug in `make_bibliography.rs` caused all `.bib`-sourced entries to show only "Cited by" with no author/title/journal. Fixed with `findnodes_foreign` traversal.
- **Session 92**: Fresh visual comparison, authblk/elsart fixes, end_mode recovery (2508.18544: 43%→88%).

## Results

| Paper ID | Status | Rust | Perl | Ratio | Main File | Notes | Visual (2026-04-05) |
|----------|--------|------|------|-------|-----------|-------|---------------------|
| 0710.2281 | OK | 757KB | 748KB | 101% | paper.tex | 31 bibitems resolved | IDENTICAL |
| 1312.5845 | OK | 35KB | 37KB | 94% | iclr_tshino_2014_v5.tex | 14 bibitems | IDENTICAL |
| 1502.04955 | OK | 302KB | 511KB | 59% | paper.tex | 15 missing cites; no .bib/.bbl | p1 near-identical; Rust shows date |
| 1706.03762 | OK | 128KB | 140KB | 91% | ms.tex | 40 bibitems, zero errors | IDENTICAL |
| 1907.08050 | OK | 1218KB | 1269KB | 95% | paper.tex | 32 bibitems | IDENTICAL |
| 1910.06709 | OK | 61KB | 62KB | 98% | paper.tex | 27 bibitems | IDENTICAL |
| 2005.13625 | EMPTY | 0KB | 990KB | 0% | main.tex | pgf boxing group mismatch | N/A |
| 2008.08932 | OK | 23KB | 19KB | 119% | main.tex | 10 bibitems | IDENTICAL |
| 2101.00726 | OK | 624KB | 650KB | 96% | wasserstein_arXiv_v2.tex | 49 bibitems | IDENTICAL |
| 2103.01205 | FAIL | 0KB | 497KB | 0% | main.tex | Timeout (pgf/tikz) | N/A |
| 2209.14198 | EMPTY | 0KB | 721KB | 0% | gucycles.tex | pgf arrow 'Stealth' | N/A |
| 2306.00809 | OK | 141KB | 140KB | 100% | backup.tex | 17 missing cites; no .bib/.bbl | IDENTICAL |
| 2306.06628 | OK | 241KB | 223KB | 107% | Contraction20.tex | 33 missing cites | IDENTICAL |
| 2308.06254 | OK | 270KB | 281KB | 96% | main.tex | cleveref+enumitem fixed | Rust CLEANER (Perl has extra red error) |
| 2310.18318 | OK | 333KB | 371KB | 89% | Hyperon-Sep-2023.tex | 52 bibitems | IDENTICAL |
| 2401.08110 | OK | 1171KB | 1205KB | 97% | errorsInHybridQST_arXiv2.tex | 190 bibitems | Rust CORRECT (Perl section heading broken) |
| 2401.18036 | OK | 230KB | 173KB | 132% | manuscript.tex | 64 bibitems | cosmetic author layout diff |
| 2401.18052 | OK | 236KB | 164KB | 143% | ms_feII.tex | 77 bibitems | Perl has blue author links |
| 2402.03300 | EMPTY | 0KB | 393KB | 0% | main.tex | pgfkeys recursion | N/A |
| 2402.10301 | FAIL | 0KB | 1421KB | 0% | paper.tex | Timeout (pgf arrows) | N/A |
| 2403.07652 | OK | 125KB | 111KB | 112% | acl_latex.tex | 28 bibitems | IDENTICAL |
| 2403.15796 | OK | 4KB | 4KB | 90% | 0_main.tex | Perl has wrong main file | Near-identical (logo size diff) |
| 2405.17032 | OK | 301KB | 793KB | 37% | ms.tex | 15 missing; tikz figs missing | p1 near-identical; tikz figs missing deeper |
| 2405.19425 | OK | 286KB | 478KB | 59% | main.tex | 80 bibitems; gap=listing style | p1 IDENTICAL |
| 2406.06608 | OK | 773KB | 739KB | 104% | main.tex | 373 bibitems | near-identical; author spacing |
| 2408.11158 | OK | 100KB | 68KB | 146% | aipsamp.tex | 27 bibitems, 1 missing | IDENTICAL |
| 2408.13687 | OK | 140KB | 109KB | 128% | main.tex | 60 bibitems | cosmetic: date, citation format |
| 2410.10068 | EMPTY | 0KB | 975KB | 0% | main.tex | tikz-cd + pgf arrows | N/A |
| 2410.12896 | OK | 673KB | 527KB | 127% | sample-manuscript.tex | 277 bibitems, 1 missing | IDENTICAL |
| 2502.04134 | OK | 121KB | 124KB | 97% | iclr2025_conference.tex | 20 bibitems | IDENTICAL |
| 2503.08256 | OK | 196KB | 215KB | 90% | main.tex | 62 bibitems | cosmetic: param leaks at top |
| 2506.03074 | OK | 1464KB | 1261KB | 116% | _main.tex | 177 bibitems | near-identical; citation style (numeric vs author-year) |
| 2507.23241 | EMPTY | 0KB | 1020KB | 0% | main.tex | smfart.cls + expl3 timing | N/A |
| 2508.15260 | EMPTY | 0KB | 360KB | 0% | main.tex | tcolorbox; Perl also fails | N/A |
| 2508.18544 | OK | 756KB | 856KB | 88% | Main_Communi_submit.tex | 56 bibitems; algo2e 1ex fix | affil key-val text; algo indentation clean |
| 2509.18103 | OK | 197KB | 275KB | 71% | main.tex | 10 bibitems; Perl HTML larger | IDENTICAL (content parity) |
| 2511.03798 | EMPTY | 0KB | 70KB | 0% | deSitter_resurgence_I.tex | eqnarray recursion; Perl fails too | N/A |
| 2511.11713 | OK | 193KB | 106KB | 181% | IEEE-conference-template-062824.tex | 59 bibitems via .bib | IDENTICAL |
| 2511.14458 | OK | 309KB | 196KB | 157% | main_mattille.tex | **57 bibitems via .bib parser** | cosmetic: numeric affil prefixes |
| 2511.15304 | OK | 140KB | 134KB | 104% | main.tex | 12 missing cites | IDENTICAL |
| 2512.09456 | OK | 200KB | 97KB | 204% | Main.tex | 30 missing; inline bib | IDENTICAL |
| 2512.16911 | OK | 704KB | 635KB | 110% | main.tex | 36 missing cites | IDENTICAL |
| 2602.18719 | OK | 34KB | 557KB | 6% | CDKU.tex | tikz-cd errors (182) | CRITICAL: body truncated |
| 2602.23324 | OK | 672KB | 659KB | 102% | main.tex | 24 missing cites | IDENTICAL |
| 2603.14602 | EMPTY | 0KB | 339KB | 0% | main.tex | listing/minted errors | N/A |
| 2603.15617 | OK | 35KB | 1189KB | 3% | paper.tex | tikzpicture mode corruption | CRITICAL: body truncated |
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

### Size parity analysis (37 OK papers, session 93)
- **>=90% parity**: 29 papers (78%) -- up from 27
- **>100% (Rust larger)**: 18 papers (48%) -- full bibliography content now resolved
- **80-89%**: 2 papers (5%)
- **70-79%**: 1 paper (3%) -- content-identical (Perl HTML verbosity)
- **50-69%**: 2 papers (5%)
- **<50%**: 3 papers (8%) -- tikz/listing/mode errors
- **>=90% including >100%**: 29 papers (78%), **>=80%**: 31 papers (84%)
- **Note:** Many papers now show Rust > Perl because bibliography entries include full author/title/journal content from .bib parsing, while Perl may have more compact citation-only references.

### Root causes of remaining gaps
1. **Missing citations (no .bib/.bbl)** -- papers using `\thebibliography` inline or missing source files
2. **Listing per-word styling** -- Perl wraps each listing token in styled `<span>` (2405.19425 gap)
3. **pgf arrow tips** (Stealth, Circle, Hooks, Implies) -- deep pgfkeys infrastructure
4. **tikz-cd matrix** -- `\tikzcdmatrixname` shape processing (2602.18719)
5. **tikzpicture mode corruption** -- failed tikz commands corrupt parser mode (2603.15617)
6. **Raw affiliation parameters** -- `[inst]organization=...` leaked (2508.18544 elsart/cas)

### Visual comparison summary (2026-04-05, session 93, thorough review)
- **20/37 IDENTICAL** on first-page screenshot (54%) -- up from 18
- **9/37 near-identical / cosmetic** (24%) -- author layout, date, spacing, citation format
- **2/37 Rust BETTER** (5%) -- 2308.06254 (cleaner), 2401.08110 (correct section headings)
- **2/37 BUG** (5%) -- 2508.18544 (raw affil params), 2603.15617 (body truncated)
- **2/37 CRITICAL** (5%) -- body content truncated (tikz corruption: 2602.18719, 2603.15617)
- **2/37 N/A** -- EMPTY/incomparable

### Actionable bugs found (session 93)
1. **2603.15617**: body truncated (35KB vs 1189KB) -- tikzpicture mode corruption
2. **2602.18719**: body truncated (34KB vs 557KB) -- tikz-cd errors cascade
3. **2508.18544**: Raw `[inst1]organization=...` affiliation parameters leak into header
4. **2405.19425**: Size regression (425→286KB) -- listing style content may be missing

### Permanent ignores (5)
- **ns1-ns5** (52_namespace) -- DTD not supported in Rust port.
