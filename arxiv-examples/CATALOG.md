# arxiv-examples Conversion Catalog

Updated 2026-04-05. 47 papers tested with latexml-oxide (Rust) and latexmlc (Perl).
Both use `--format=html5 --nodefaultresources --preload=ar5iv.sty` + ar5iv CSS CDN links.
Rust uses `bibconfig=bbl,bib` fallback (bbl preferred, raw .bib as fallback via native Rust BibTeX parser).

## Summary

- **37/47 OK** (79%) -- produce meaningful Rust HTML5 output
- **8 EMPTY** -- produce minimal output (cascading errors or TooManyErrors)
- **2 FAIL** -- timeout (no output)
- **26/37 >=90% size parity** with Perl (70% of OK)
- **29/37 >=80% size parity** (78% of OK)
- Perl HTML regenerated 2026-04-05 with correct flags (`--nodefaultresources` + ar5iv CSS)
- **Session 91**: Bibliography revolution — pure Rust BibTeX parser, `\lx@ifusebbl` fallback chain, MakeBibliography ObjectDB registration. 20+ papers gain resolved bibliographies.

## Results

| Paper ID | Status | Rust | Perl | Ratio | Main File | Notes | Visual (2026-04-05) |
|----------|--------|------|------|-------|-----------|-------|---------------------|
| 0710.2281 | OK | 757KB | 748KB | 101% | paper.tex | 31 bibitems resolved | IDENTICAL |
| 1312.5845 | OK | 36KB | 38KB | 95% | iclr_tshino_2014_v5.tex | 14 bibitems | IDENTICAL |
| 1502.04955 | OK | 302KB | 511KB | 59% | paper.tex | 15 missing cites; no .bib/.bbl | p1 OK; bib gap |
| 1706.03762 | OK | 129KB | 141KB | 91% | ms.tex | 40 bibitems, zero errors | IDENTICAL |
| 1907.08050 | OK | 1218KB | 1270KB | 96% | paper.tex | 32 bibitems | IDENTICAL |
| 1910.06709 | OK | 61KB | 62KB | 98% | paper.tex | 27 bibitems | IDENTICAL |
| 2005.13625 | EMPTY | 0KB | 990KB | 0% | main.tex | pgf boxing group mismatch | N/A |
| 2008.08932 | OK | 18KB | 19KB | 94% | main.tex | 10 bibitems | IDENTICAL |
| 2101.00726 | OK | 625KB | 650KB | 96% | wasserstein_arXiv_v2.tex | 49 bibitems | IDENTICAL |
| 2103.01205 | FAIL | 0KB | 497KB | 0% | main.tex | Timeout (pgf/tikz) | N/A |
| 2209.14198 | EMPTY | 0KB | 721KB | 0% | gucycles.tex | pgf arrow 'Stealth' | N/A |
| 2306.00809 | OK | 142KB | 141KB | 100% | backup.tex | 17 missing cites; no .bib/.bbl | IDENTICAL (s90 fix) |
| 2306.06628 | OK | 201KB | 224KB | 90% | Contraction20.tex | 33 missing cites | p1 OK; bib gap |
| 2308.06254 | OK | 271KB | 281KB | 96% | main.tex | cleveref+enumitem fixed | Rust CLEANER (Perl macro leak) |
| 2310.18318 | OK | 334KB | 372KB | 90% | Hyperon-Sep-2023.tex | 52 bibitems | cosmetic title/author diffs |
| 2401.08110 | OK | 1172KB | 1205KB | 97% | errorsInHybridQST_arXiv2.tex | 190 bibitems | IDENTICAL |
| 2401.18036 | OK | 167KB | 174KB | 96% | manuscript.tex | 64 bibitems | p1 OK |
| 2401.18052 | OK | 155KB | 165KB | 94% | ms_feII.tex | 77 bibitems | author affiliation layout |
| 2402.03300 | EMPTY | 0KB | 393KB | 0% | main.tex | pgfkeys recursion | N/A |
| 2402.10301 | FAIL | 0KB | 1421KB | 0% | paper.tex | Timeout (pgf arrows) | N/A |
| 2403.07652 | OK | 104KB | 111KB | 94% | acl_latex.tex | 28 bibitems | author metadata layout |
| 2403.15796 | OK | 4KB | 4KB | 95% | 0_main.tex | Perl has wrong main file | N/A (incomparable) |
| 2405.17032 | OK | 302KB | 794KB | 38% | ms.tex | 15 missing; tikz figs missing | p1 OK; tikz figs missing deeper |
| 2405.19425 | OK | 242KB | 479KB | 50% | main.tex | 80 bibitems; gap=listing style | p1 near-identical |
| 2406.06608 | OK | 732KB | 740KB | 99% | main.tex | 373 bibitems | IDENTICAL |
| 2408.11158 | OK | 65KB | 69KB | 94% | aipsamp.tex | 27 bibitems, 1 missing | IDENTICAL |
| 2408.13687 | OK | 106KB | 109KB | 97% | main.tex | 60 bibitems | minor viewport diff |
| 2410.10068 | EMPTY | 0KB | 975KB | 0% | main.tex | tikz-cd + pgf arrows | N/A |
| 2410.12896 | OK | 453KB | 527KB | 86% | sample-manuscript.tex | 277 bibitems, 1 missing | p1 near-identical |
| 2502.04134 | OK | 112KB | 125KB | 90% | iclr2025_conference.tex | 20 bibitems | IDENTICAL |
| 2503.08256 | OK | 166KB | 216KB | 77% | main.tex | 62 bibitems; Perl HTML larger | cosmetic |
| 2506.03074 | OK | 1242KB | 1262KB | 98% | _main.tex | 177 bibitems | IDENTICAL (s90 fix) |
| 2507.23241 | EMPTY | 0KB | 1020KB | 0% | main.tex | smfart.cls + expl3 timing | N/A |
| 2508.15260 | EMPTY | 0KB | 360KB | 0% | main.tex | tcolorbox; Perl also fails | N/A |
| 2508.18544 | OK | 373KB | 856KB | 44% | Main_Communi_submit.tex | 25 missing; shortstack errors | p1 OK; missing keywords/PACS |
| 2509.18103 | OK | 197KB | 276KB | 71% | main.tex | 10 bibitems; Perl HTML larger | IDENTICAL (content parity) |
| 2511.03798 | EMPTY | 0KB | 70KB | 0% | deSitter_resurgence_I.tex | eqnarray recursion; Perl fails too | N/A |
| 2511.11713 | OK | 94KB | 107KB | 88% | IEEE-conference-template.tex | 76 missing; no .bib/.bbl | p1 near-identical |
| 2511.14458 | OK | 221KB | 196KB | 113% | main_mattille.tex | **57 bibitems via .bib parser** | Rust AHEAD (Perl has no bib) |
| 2511.15304 | OK | 113KB | 134KB | 84% | main.tex | 12 missing cites | cosmetic author diffs |
| 2512.09456 | OK | 98KB | 98KB | 100% | Main.tex | 30 missing; inline bib | minor email formatting |
| 2512.16911 | OK | 584KB | 635KB | 92% | main.tex | 36 missing cites | upstream Perl \and bug |
| 2602.18719 | OK | 35KB | 557KB | 6% | CDKU.tex | tikz-cd errors (182) | CRITICAL: body truncated |
| 2602.23324 | OK | 626KB | 659KB | 95% | main.tex | 24 missing cites | cosmetic |
| 2603.14602 | EMPTY | 0KB | 339KB | 0% | main.tex | listing/minted errors | N/A |
| 2603.15617 | OK | 35KB | 1190KB | 3% | paper.tex | tikzpicture mode corruption | CRITICAL: body truncated |
| 2603.19312 | OK | 278KB | 294KB | 94% | main.tex | 31 missing cites | p1 OK |

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

### Size parity analysis (37 OK papers, session 91)
- **>=90% parity**: 23 papers (62%)
- **>100% (Rust larger)**: 3 papers (8%) — Rust has more resolved content (bibliography)
- **80-89%**: 3 papers (8%)
- **70-79%**: 2 papers (5%) — content-identical (Perl HTML verbosity)
- **50-69%**: 2 papers (5%)
- **<50%**: 4 papers (11%) -- tikz/listing/mode errors
- **>=90% including >100%**: 26 papers (70%), **>=80%**: 29 papers (78%)
- **Note:** size ratio can be misleading. Papers with identical paragraph/section/math counts may show <80% ratio due to Perl HTML attribute verbosity.

### Root causes of remaining gaps
1. **Missing citations (no .bib/.bbl)** -- papers using `\thebibliography` inline or missing source files (13 papers with missing_citation)
2. **Listing per-word styling** -- Perl wraps each listing token in styled `<span>`
3. **shortstack/vtop mode cascade** -- DefConstructor bounded+mode interaction
4. **pgf arrow tips** (Stealth, Circle, Hooks, Implies) -- deep pgfkeys infrastructure
5. **tikz-cd matrix** -- `\tikzcdmatrixname` shape processing
6. **tikzpicture mode corruption** -- failed tikz commands corrupt parser mode

### Visual comparison issues (identified 2026-04-05, corrected after investigation)
- ~~Dark theme~~ FALSE POSITIVE: stale Perl HTML used different CSS (LaTeXML.css dark mode vs ar5iv.css)
- ~~Warning banners~~ **FIXED** session 90: `input_definitions()` search order corrected
- ~~\MakeUppercase~~ FALSE POSITIVE: CSS difference (`ltx-amsart.css` `text-transform:uppercase`)
- ~~Author \and~~ UPSTREAM PERL BUG: amsmath DefMath overrides neurips `\And` (both match)
- ~~Figure float~~ FALSE POSITIVE: figures identical; visual diff was bibliography gap
- **Missing affiliation text** -- 2511.14458: `authblkRelocateAffil` DOM surgery not ported (REAL BUG)

### Visual parity summary (2026-04-05, session 91)
- **11/37 IDENTICAL** on first-page screenshot (30%)
- **8/37 p1 OK / near-identical** (22%) -- differences only in deeper content (bib, tikz, listings)
- **9/37 cosmetic/minor diffs** (24%) -- author layout, spacing, email formatting
- **1/37 Rust AHEAD** (3%) -- 2511.14458: Rust has resolved bibliography, Perl doesn't
- **3/37 CRITICAL** (8%) -- body content truncated (tikz corruption, tikz-cd)
- **2/37 N/A** (incomparable: Perl wrong main file, or Rust cleaner)
- **3 false positives resolved** (session 90): dark theme + \MakeUppercase + figure float = stale Perl CSS
- **2 fixes applied** (sessions 90-91): warning banners, bibliography resolution

### Permanent ignores (5)
- **ns1-ns5** (52_namespace) -- DTD not supported in Rust port.
