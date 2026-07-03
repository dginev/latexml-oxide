# arXiv/LaTeXML fork audit — features of value for latexml-oxide (2026-07-03)

> Due-diligence survey of https://github.com/arXiv/LaTeXML (the "velocity
> fork") against upstream `brucemiller/LaTeXML` and against latexml-oxide.
> Point-in-time snapshot; no implementation performed. Fork state at audit:
> 57 commits ahead / 15 behind upstream (merge-base `51fea96a`, ~#2806),
> last pushed 2026-06-02. Authors of the delta: Deyan Ginev (30),
> Vincenzo Mantova (20), Bruce Miller (7).

## Verdict summary

| Cluster | Fork content | Rust status | Action |
|---|---|---|---|
| A. pgfmath parser (~20 commits) | native `\pgfmathparse` grammar: ternary/formula entry, radix literals, factorial/`r` postfix, gcd/bin/frac/int, string results, retokenized `\pgfmathresult`, comparison→integer | **MOSTLY PORTED** — `pgfmath_code_tex.rs` (1774 lines) mirrors the fork's grammar shape; probes pass for ternary, radix (`0xA+0b10`=12), `4!`=24, `frac`, `gcd`, `ifthenelse(1,"yes","no")`="yes" | Port the two probed residuals: top-level string results (`\pgfmathparse{"hello"}` → Rust `0.0`, PGF/fork `hello`) and integer-formatted comparisons (`2==2` → Rust `1.0`, fork `1`); also review the untested refinements (retokenized results w/ macros, `ExpandedPartially` arg, `UnTeX` no-linebreak read, latex3 `_` CS names in expressions, sqrt Error-not-Fatal guard) |
| B. abstract/acknowledgements in TOC (`inlist="toc"` + xml:id, `\tableofcontents` exempt) | 4 commits | **NOT PORTED** | Small, valuable for ar5iv nav-TOC (ar5iv-css has a `navtoc-abstract-and-references` branch expecting this shape). Good candidate. |
| C. `usebbl` / bbl-first bibliography | 1 commit + tests | **DONE** — Rust `BIB_CONFIG` bbl/bib precedence, 5-case `cluster_bbl_bib_precedence` fixture; ar5iv sets `bibconfig={bbl,bib}` programmatically | none |
| D. calc-aware parameter types (`{Number}`/`{Dimension}`/`[Dimension]` accept calc syntax; calc in GraphixDimensions) | 6 commits (Bruce) | **DONE** (behaviorally) — probes pass: `\setlength{\parindent}{2pt+3pt}`→5pt, `\setcounter{page}{2*3}`→6, `\resizebox{1cm+2pt}` clean, 0 errors | none (spot-check `\hspace{...}`-class sites if a corpus signal appears) |
| E. hyperref/hyperxmp metadata (all `\hypersetup` options, `pdfmetalang` RDF, undigested args) | 5 commits | **LARGELY PORTED** — `hyperxmp_sty.rs` exists, `pdfmetalang` handled in `hyperref_sty.rs` | Minor residual: `\XMPLangAlt`/`\xmpquote`/`\xmpcomma` stubs not found in Rust hyperxmp — trivial adds if papers hit them |
| F. accessibility: acmart ARIA `\Description`; graphicx `actualtext`/`artifact` keys | 2 commits | Description **PORTED** (`acmart_cls.rs` + aria namespace); graphicx `actualtext`/`artifact` **NOT PORTED** (27-line Perl delta) | Small candidate; aligns with the accessibility story |
| G. core: `readBalanced` drops comment tokens | 1 commit (`4e1578d1`) | **NOT PORTED** — Rust `read_balanced` still flushes `pending_comments` into the captured tokens (gullet.rs ~L1170), the exact code the fork removes | Low urgency: Rust defaults `INCLUDE_COMMENTS=false`, so pending comments are rare outside `--comments` runs. Port when touching that seam. |
| — | list envs `internal_vertical` (3d56fb14) | perltidy noise on inspection; the mode was already there | none |
| — | register-assignment tracing QoL, ARXIV_CHANGELOG, MANIFEST, worker/preview ops, test regens | arXiv-ops / repo upkeep | none |

## Notes

* The big raw-diff entries (Font.pm ±303, latex_constructs ±230, Box.pm ±113)
  are dominated by the fork being 15 commits BEHIND upstream plus perltidy —
  they are not fork features.
* Bruce-authored fork commits (calc param types) are natural upstreaming
  candidates; if they land in `brucemiller/LaTeXML` they'll enter our normal
  upstream-sync pipeline anyway (and Rust already behaves correctly).
* The fork's `t/pgfmathparse/pgfmathparse.{tex,xml}` (195-line test) is a
  richer harness than our independent `tests/pgf/stress_pgfmath.tex` (135
  lines; PR #203): it wraps every case in
  `ifthenelse(1,#2,"never")` to co-test string passthrough. Worth adopting
  as a fixture when porting the cluster-A residuals.

## Ranked recommendation (if/when implementation is approved)

1. **B — abstract/acknowledgements `inlist=toc`** (small, user-visible,
   ar5iv-css counterpart already exists).
2. **A residuals — pgfmath string results + integer comparisons** (+ adopt
   the fork's test file); bounded, corpus-relevant (tikz labels use string
   results).
3. **F residual — graphicx `actualtext`/`artifact`** (27 Perl lines,
   accessibility value).
4. **E residual — hyperxmp `\xmpcomma`/`\XMPLangAlt` stubs** (trivial).
5. **G — readBalanced comment-token drop** (defer to the next gullet-seam
   session; low exposure under default `INCLUDE_COMMENTS=false`).
