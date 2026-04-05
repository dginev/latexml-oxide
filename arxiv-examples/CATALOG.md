# arxiv-examples Conversion Catalog

Generated 2026-04-04. 47 papers tested with latexml-oxide (Rust) and latexmlc (Perl).
Both use `--format=html5 --nodefaultresources --preload=ar5iv.sty` + ar5iv CSS CDN links.

## Summary

- **37/47 OK** (79%) -- produce meaningful Rust HTML5 output
- **8 EMPTY** -- produce minimal 39B output (cascading errors or TooManyErrors)
- **2 FAIL** -- timeout (no output)
- **22/37 >=90% size parity** with Perl (59%)

## Results

| Paper ID | Status | Rust | Perl | Ratio | Main File | Notes |
|----------|--------|------|------|-------|-----------|-------|
| 0710.2281 | OK | 757KB | 748KB | 101% | paper.tex | Fixed via `\@setref` stub |
| 1312.5845 | OK | 36KB | 38KB | 95% | iclr_tshino_2014_v5.tex | Zero errors |
| 1502.04955 | OK | 481KB | 511KB | 94% | paper.tex | |
| 1706.03762 | OK | 129KB | 141KB | 91% | ms.tex | Zero errors, Attention paper |
| 1907.08050 | OK | 1220KB | 1269KB | 96% | paper.tex | Was timeout, now works |
| 1910.06709 | OK | 61KB | 62KB | 98% | paper.tex | Zero errors |
| 2005.13625 | EMPTY | 0KB | 990KB | 0% | main.tex | pgf boxing group mismatch |
| 2008.08932 | OK | 18KB | 19KB | 95% | main.tex | Zero errors |
| 2101.00726 | OK | 625KB | 650KB | 96% | wasserstein_arXiv_v2.tex | |
| 2103.01205 | FAIL | 0KB | 497KB | 0% | main.tex | Timeout (pgf/tikz) |
| 2209.14198 | EMPTY | 0KB | 720KB | 0% | gucycles.tex | pgf arrow 'Stealth' |
| 2306.00809 | OK | 142KB | 141KB | 101% | backup.tex | |
| 2306.06628 | OK | 201KB | 252KB | 80% | Contraction20.tex | Zero errors |
| 2308.06254 | OK | 3KB | 281KB | 1% | main.tex | Missing: enumitem, complexity |
| 2310.18318 | OK | 334KB | 371KB | 90% | Hyperon-Sep-2023.tex | |
| 2401.08110 | OK | 1172KB | 1205KB | 97% | errorsInHybridQST_arXiv2.tex | Was timeout |
| 2401.18036 | OK | 167KB | 229KB | 73% | manuscript.tex | |
| 2401.18052 | OK | 155KB | 165KB | 94% | ms_feII.tex | |
| 2402.03300 | EMPTY | 0KB | 392KB | 0% | main.tex | pgfkeys recursion |
| 2402.10301 | FAIL | 0KB | 1420KB | 0% | paper.tex | Timeout (pgf arrows) |
| 2403.07652 | OK | 104KB | 111KB | 94% | acl_latex.tex | |
| 2403.15796 | OK | 131KB | 4KB | - | 0_main.tex | Perl has wrong main file |
| 2405.17032 | OK | 302KB | 793KB | 38% | ms.tex | tikz figures missing |
| 2405.19425 | OK | 242KB | 479KB | 51% | main.tex | Was timeout |
| 2406.06608 | OK | 731KB | 739KB | 99% | main.tex | Improved by `\@setref` |
| 2408.11158 | OK | 65KB | 69KB | 94% | aipsamp.tex | |
| 2408.13687 | OK | 106KB | 109KB | 97% | main.tex | |
| 2410.10068 | EMPTY | 0KB | 974KB | 0% | main.tex | tikz-cd + pgf arrows |
| 2410.12896 | OK | 453KB | 763KB | 59% | sample-manuscript.tex | |
| 2502.04134 | OK | 112KB | 125KB | 90% | iclr2025_conference.tex | |
| 2503.08256 | OK | 167KB | 190KB | 88% | main.tex | |
| 2506.03074 | OK | 1234KB | 1261KB | 98% | _main.tex | 19 .tex files |
| 2507.23241 | EMPTY | 0KB | 944KB | 0% | main.tex | expl3 loading timing |
| 2508.15260 | EMPTY | 0KB | 2KB | 0% | main.tex | tcolorbox; Perl also fails |
| 2508.18544 | OK | 374KB | 856KB | 44% | Main_Communi_submit.tex | |
| 2509.18103 | OK | 197KB | 194KB | 102% | main.tex | |
| 2511.03798 | EMPTY | 0KB | 68KB | 0% | deSitter_resurgence_I.tex | eqnarray recursion; Perl also fails |
| 2511.11713 | OK | 94KB | 197KB | 48% | IEEE-conference-template.tex | Zero errors, size gap |
| 2511.14458 | OK | 193KB | 286KB | 67% | main_mattille.tex | |
| 2511.15304 | OK | 114KB | 141KB | 81% | main.tex | |
| 2512.09456 | OK | 98KB | 98KB | 100% | Main.tex | Exact match |
| 2512.16911 | OK | 579KB | 703KB | 82% | main.tex | |
| 2602.18719 | OK | 35KB | 527KB | 7% | CDKU.tex | tikz-cd errors (182) |
| 2602.23324 | OK | 626KB | 682KB | 92% | main.tex | Was timeout |
| 2603.14602 | EMPTY | 0KB | 297KB | 0% | main.tex | listing parameter errors |
| 2603.15617 | OK | 34KB | 1190KB | 3% | paper.tex | tikzpicture mode corruption |
| 2603.19312 | OK | 275KB | 328KB | 84% | main.tex | Was timeout |

## Failure Analysis

### FAIL (2 papers)
- **2103.01205**: pgf/tikz timeout (>120s)
- **2402.10301**: pgf 'Computer Modern Rightarrow' arrow + cascading, timeout

### EMPTY (8 papers)
- **2005.13625**: pgf `\pgf@x` number parsing, boxing group mismatch
- **2209.14198**: pgf arrow 'Stealth' undefined, cascading to token limit
- **2402.03300**: `\pgfkeys@mainstop` recursive self-expansion loop
- **2410.10068**: tikz-cd matrix processing, pgf arrows
- **2507.23241**: `\ExplSyntaxOn` undefined during preamble (expl3 timing)
- **2508.15260**: tcolorbox cascading errors (Perl also only 2KB)
- **2511.03798**: `\@@eqnarray` recursion in jheppub (Perl also fails)
- **2603.14602**: listing parameter parsing, TooManyErrors

### Size parity analysis (37 OK papers)
- **>=90% parity**: 22 papers (59%)
- **70-89%**: 6 papers (16%)
- **50-69%**: 3 papers (8%)
- **<50%**: 6 papers (16%) -- need investigation

### Papers needing investigation (Rust <<< Perl)
1. **2308.06254** (1%): 3KB vs 281KB -- Missing: enumitem `\newlist`/`\setlist`, complexity.sty, biblatex
2. **2603.15617** (3%): 34KB vs 1190KB -- tikzpicture mode corruption, only frontmatter renders
3. **2602.18719** (7%): 35KB vs 527KB -- tikz-cd `\tikzcdmatrixname` (98 errors) + pgf GenericError (59)
4. **2405.17032** (38%): 302KB vs 793KB -- pgf 'Circle' arrow tips (7 errors), tikz figures
5. **2508.18544** (44%): 374KB vs 856KB -- elsarticle; XUntil param type needed for elsart_support_core
6. **2511.11713** (48%): 94KB vs 197KB -- NOT a bug: gap from `--nobibtex` (bibliography not rendered)
7. **2405.19425** (51%): 242KB vs 479KB -- NOT a bug: gap from CSS class differences (listing tokens)

### Visual comparison results (2026-04-04)
Screenshotted all 38 papers with both Rust+Perl output.

**Pixel-perfect parity (frontmatter):**
- 1312.5845, 1706.03762, 1910.06709, 2008.08932, 2502.04134, 2512.09456

**Excellent visual match:**
- 0710.2281, 1502.04955, 1907.08050, 2101.00726, 2306.00809, 2310.18318,
  2401.08110, 2401.18052, 2403.07652, 2406.06608, 2408.11158, 2408.13687,
  2506.03074, 2509.18103, 2602.23324, 2603.19312

**Size gap but frontmatter matches:**
- 2405.17032, 2405.19425, 2511.14458 -- tikz/figure content missing in body
- 2410.12896, 2511.11713, 2508.18544 -- later sections have less content

**Significant content gaps:**
- 2602.18719 (7%): tikz-cd errors consume most content
- 2603.15617 (3%): `\halign`/tikz errors, only frontmatter renders
- 2308.06254 (1%): cascading package errors (enumitem/complexity/biblatex)

### Session fixes (2026-04-04)
- **`\@setref` stub**: Recovered 0710.2281 from 39% to 101%, 2406.06608 from 82% to 99%
- **XSLT path resolution**: All papers now output proper HTML5 (was raw XML)
- **iopart_support.sty**: `\buildrel` parameter fix + 30 missing definitions
- **`\include`/`\includeonly`**: Ported from Perl
- **generate_all.sh**: `_main.tex` candidate + `\documentclass` detection (fixed 5 papers)
