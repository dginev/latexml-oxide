# arxiv-examples Conversion Catalog

Updated 2026-04-05. 47 papers tested with latexml-oxide (Rust) and latexmlc (Perl).
Both use `--format=html5 --nodefaultresources --preload=ar5iv.sty` + ar5iv CSS CDN links.
Rust uses `--nobibtex`. Perl uses MakeBibliography for .bib processing.

## Summary

- **37/47 OK** (79%) -- produce meaningful Rust HTML5 output
- **8 EMPTY** -- produce minimal output (cascading errors or TooManyErrors)
- **2 FAIL** -- timeout (no output)
- **24/37 >=90% size parity** with Perl (65% of OK)
- **29/37 >=80% size parity** (78% of OK)
- Perl HTML regenerated 2026-04-05 with correct flags (`--nodefaultresources` + ar5iv CSS)

## Results

| Paper ID | Status | Rust | Perl | Ratio | Main File | Notes | Visual (2026-04-05) |
|----------|--------|------|------|-------|-----------|-------|---------------------|
| 0710.2281 | OK | 757KB | 748KB | 101% | paper.tex | `\@setref` stub | IDENTICAL |
| 1312.5845 | OK | 35KB | 37KB | 94% | iclr_tshino_2014_v5.tex | Zero errors | IDENTICAL |
| 1502.04955 | OK | 302KB | 511KB | 59% | paper.tex | elsarticle; Perl MakeBibliography | p1 OK; bib gap |
| 1706.03762 | OK | 128KB | 140KB | 91% | ms.tex | Zero errors, Attention paper | IDENTICAL |
| 1907.08050 | OK | 1218KB | 1269KB | 95% | paper.tex | Was timeout, now works | IDENTICAL |
| 1910.06709 | OK | 61KB | 62KB | 98% | paper.tex | Zero errors | IDENTICAL |
| 2005.13625 | EMPTY | 0KB | 990KB | 0% | main.tex | pgf boxing group mismatch | N/A |
| 2008.08932 | OK | 18KB | 19KB | 94% | main.tex | Zero errors | IDENTICAL |
| 2101.00726 | OK | 624KB | 650KB | 96% | wasserstein_arXiv_v2.tex | | IDENTICAL |
| 2103.01205 | FAIL | 0KB | 497KB | 0% | main.tex | Timeout (pgf/tikz) | N/A |
| 2209.14198 | EMPTY | 0KB | 720KB | 0% | gucycles.tex | pgf arrow 'Stealth' | N/A |
| 2306.00809 | OK | 141KB | 140KB | 100% | backup.tex | | IDENTICAL (s90 fix) |
| 2306.06628 | OK | 200KB | 224KB | 89% | Contraction20.tex | gap=bib | p1 OK; bib gap |
| 2308.06254 | OK | 270KB | 280KB | 96% | main.tex | **Fixed** cleveref+enumitem | Rust CLEANER (Perl macro leak) |
| 2310.18318 | OK | 334KB | 371KB | 89% | Hyperon-Sep-2023.tex | | cosmetic title/author diffs |
| 2401.08110 | OK | 1172KB | 1205KB | 97% | errorsInHybridQST_arXiv2.tex | Was timeout | IDENTICAL |
| 2401.18036 | OK | 167KB | 174KB | 96% | manuscript.tex | | p1 OK |
| 2401.18052 | OK | 155KB | 164KB | 94% | ms_feII.tex | | author affiliation layout |
| 2402.03300 | EMPTY | 0KB | 392KB | 0% | main.tex | pgfkeys recursion | N/A |
| 2402.10301 | FAIL | 0KB | 1420KB | 0% | paper.tex | Timeout (pgf arrows) | N/A |
| 2403.07652 | OK | 104KB | 111KB | 93% | acl_latex.tex | | author metadata layout |
| 2403.15796 | OK | 131KB | 4KB | - | 0_main.tex | Perl has wrong main file | N/A (incomparable) |
| 2405.17032 | OK | 301KB | 793KB | 38% | ms.tex | tikz figures missing | p1 OK; tikz figs missing deeper |
| 2405.19425 | OK | 241KB | 478KB | 50% | main.tex | gap=listing per-word styling | p1 near-identical |
| 2406.06608 | OK | 731KB | 739KB | 98% | main.tex | `\@setref` fix | IDENTICAL |
| 2408.11158 | OK | 64KB | 68KB | 93% | aipsamp.tex | | IDENTICAL |
| 2408.13687 | OK | 105KB | 109KB | 97% | main.tex | | minor viewport diff |
| 2410.10068 | EMPTY | 0KB | 974KB | 0% | main.tex | tikz-cd + pgf arrows | N/A |
| 2410.12896 | OK | 453KB | 527KB | 85% | sample-manuscript.tex | gap=bib richness, cross-refs | p1 near-identical |
| 2502.04134 | OK | 112KB | 124KB | 90% | iclr2025_conference.tex | | IDENTICAL |
| 2503.08256 | OK | 166KB | 215KB | 77% | main.tex | same content; Perl HTML larger | cosmetic |
| 2506.03074 | OK | 1242KB | 1261KB | 98% | _main.tex | 19 .tex files | IDENTICAL (s90 fix) |
| 2507.23241 | EMPTY | 0KB | 944KB | 0% | main.tex | smfart.cls + expl3 timing | N/A |
| 2508.15260 | EMPTY | 0KB | 1KB | 0% | main.tex | tcolorbox; Perl also fails | N/A |
| 2508.18544 | OK | 373KB | 856KB | 43% | Main_Communi_submit.tex | shortstack mode errors | p1 OK; missing keywords/PACS |
| 2509.18103 | OK | 197KB | 276KB | 71% | main.tex | same content; Perl HTML larger | IDENTICAL (content parity) |
| 2511.03798 | EMPTY | 0KB | 67KB | 0% | deSitter_resurgence_I.tex | eqnarray recursion; Perl fails too | N/A |
| 2511.11713 | OK | 93KB | 106KB | 87% | IEEE-conference-template.tex | gap=Perl MakeBibliography | p1 near-identical |
| 2511.14458 | OK | 192KB | 196KB | 98% | main_mattille.tex | Near parity | IDENTICAL (s90 authblk fix) |
| 2511.15304 | OK | 114KB | 134KB | 84% | main.tex | gap=bib | cosmetic author diffs |
| 2512.09456 | OK | 97KB | 97KB | 99% | Main.tex | Exact match | minor email formatting |
| 2512.16911 | OK | 578KB | 635KB | 91% | main.tex | gap=bib | upstream Perl \and bug |
| 2602.18719 | OK | 34KB | 527KB | 6% | CDKU.tex | tikz-cd errors (182) | CRITICAL: body truncated |
| 2602.23324 | OK | 626KB | 659KB | 94% | main.tex | Was timeout | cosmetic |
| 2603.14602 | EMPTY | 0KB | 296KB | 0% | main.tex | listing/minted errors | N/A |
| 2603.15617 | OK | 34KB | 1189KB | 3% | paper.tex | tikzpicture mode corruption | CRITICAL: body truncated |
| 2603.19312 | OK | 275KB | 294KB | 93% | main.tex | gap=bib | p1 OK |

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

### Size parity analysis (37 OK papers, fresh Perl 2026-04-05)
- **>=90% parity**: 24 papers (65%)
- **80-89%**: 5 papers (14%)
- **70-79%**: 3 papers (8%) — 2 are content-identical (Perl HTML verbosity)
- **50-69%**: 2 papers (5%)
- **<50%**: 3 papers (8%) -- tikz/listing/mode errors
- **Note:** size ratio can be misleading. Papers with identical paragraph/section/math counts may show <80% ratio due to Perl HTML attribute verbosity.

### Root causes of remaining gaps
1. **Perl MakeBibliography** -- .bib-only papers processed by Perl's convertBibliography (7 papers)
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

### Visual parity summary (2026-04-05, corrected)
- **11/37 IDENTICAL** on first-page screenshot (30%)
- **8/37 p1 OK / near-identical** (22%) -- differences only in deeper content (bib, tikz, listings)
- **9/37 cosmetic/minor diffs** (24%) -- author layout, spacing, email formatting (3 former "functional" were CSS false positives)
- **1/37 functional diff** (3%) -- 2511.14458 missing affiliations (authblk bug)
- **3/37 CRITICAL** (8%) -- body content truncated (tikz corruption, tikz-cd)
- **2/37 N/A** (incomparable: Perl wrong main file, or Rust cleaner)
- **3 false positives resolved**: dark theme + \MakeUppercase + figure float = stale Perl CSS
- **1 fix applied**: warning banners (input_definitions search order)

### Permanent ignores (5)
- **ns1-ns5** (52_namespace) -- DTD not supported in Rust port.
