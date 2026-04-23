# 10k sandbox triage — high-priority parity worklist (2026-04-23)

Per user directive: **all remaining 10k-sandbox failures are high-priority
parity work**. The goal is: every paper in `~/data/10k_sandbox/` converts
error-free with good performance. This document groups the 618 failures
from the `22adfc355`-binary full sweep (`-j 16 --timeout 60`) into
actionable classes, each with a root cause, an owner file, and a target
Rust-port fix.

Data source: `/tmp/sandbox_full.log` + per-paper `~/data/10k_sandbox_html_full/$p.log`.
Triage helpers: `/tmp/sandbox_triage/{conv_err_by_category.tsv,
undef_only_papers.txt, undef_macros_all.txt, undef_macro_histogram.txt,
affil_papers.txt}`.

## Top-level distribution (566 conversion_error + 47 timeout + 3 abort + 2 fatal = 618)

### conversion_error by primary Error: category (566 papers)

| # papers | Error categories | Class |
|---|---|---|
| 309 | `Error:undefined:` only | **Missing package bindings** |
|  67 | `misdefined + undefined + unexpected` cascade | Cascade from miss |
|  45 | `Error:malformed:ltx:` | **Real Rust emitter bugs** |
|  38 | (empty — Warn-only) | Warn-cascade |
|  35 | `undefined + unexpected` | Cascade from miss |
|  17 | `unexpected:` only | Parse-state surprise |
|   6 | `missing_file:psfig + undefined` | psfig raw `.ps` include |
|   6 | `expected: + unexpected:` | Parameter-consumption bug |
|   5 | `malformed:ltx: + undefined` | Cascade into emitter |
|   5 | `expected: + undefined:` | Cascade |
|   4 | `unexpected:double` | Double-token register read |
|   4 | `undefined + unexpected:fi` | Conditional cascade |
|   3 | `malformed:ltx:bibitem + undefined` | bibitem structure bug |
|   3 | `latex: + undefined` | LaTeX diagnostic |
|   3 | `expected:` only | Parameter expected |
|   2 | `latex:` only | LaTeX diagnostic |
|   2 | `expected + misdefined + undefined + unexpected` | Full cascade |
|   1 | `malformed:ltx:section + unexpected` | section emitter bug |
|   1 | `undefined + unexpected + unexpected:fi` | Cascade |
| 38 | no ERROR markers | (may be spurious classification — verify) |

### Hard failures (5 papers)

| Paper | Category | Root cause | Class |
|---|---|---|---|
| 1112.6246 | abort (OOM) | unknown — error-cascade OOM | **Needs investigation** |
| 1710.03688 | abort (OOM) | babel-french `\bbl@exp@aux` infinite loop | **Class A: babel-french** |
| 1902.08705 | abort (OOM) | pgfmath `\ifdim` empty-token infinite loop | **Class B: pgfmath empty-arg** |
| 1803.03288 | fatal (10001-err) | missing xparse 2018+/pgfplots/cleveref | **Class C: modern-package cascade** |
| hep-ph0702114 | fatal (10001-err) | babel-french `\bbl@exp@aux` (same as 1710.03688) | **Class A: babel-french** |

## Undefined-macro histogram (top entries across 309 papers, 494 total incidents)

```
29 \keywords                  14 \lesssim          5 \righthead
22 \affil                     13 \psfig            5 \opening
19 \address                   10 \plotone          5 \lefthead
17 \references                10 \gtrsim           5 \figcaption
16 \altaffiltext              10 \apj              4 \sun
16 \altaffilmark               9 \acknowledgements 4 \slugcomment
16 \acknowledgments            8 \mnras            4 \plotfiddle
16 \abstracts                  7 \apjs             4 \ga
15 \reference                  6 \aj               3 \topmatter
                               6 \aap              3 \Cal
                               5 \tableline        3 \la
                               5 \epsscale         3 \endtopmatter
```

**Pattern observations:**

1. **49 papers** have at least one astronomy-journal macro undefined
   (`\affil`, `\altaffilmark`, `\altaffiltext`, `\acknowledgments`,
   `\apj`, `\mnras`, `\apjs`, `\aj`, `\aap`, `\apjl`, `\sun`,
   `\plotone`, `\plotfiddle`, `\slugcomment`, `\arcsec`, `\arcdeg`,
   `\figcaption`, `\lefthead`, `\righthead`). All identified cases
   route through **`\documentstyle[…,aaspp4]{article}`** (LaTeX 2.09),
   but `aaspp_sty.rs` doesn't define these — aastex macros live in
   `aas_support_sty.rs`. The aaspp4 option should pull aas_support
   automatically.
2. **8 papers** need AMS Plain-TeX header macros (`\topmatter`, `\Cal`,
   `\Refs`, etc.) — suggests `amsppt.sty` or `amstex.tex` path gaps.
3. **\psfig** (13) needs a PostScript-include fallback that emits a
   `<ltx:graphics>` pointing at the `.ps` file (psfig is pre-epsf era).
4. **`\lesssim`, `\gtrsim`, `\la`, `\ga`, `\sun`, `\arcsec`, `\arcdeg`**
   are math symbols from AASTeX — defining as `\lesssim=\mathrel{\sim<}`
   or Unicode equivalents in `aas_support_sty.rs` would close them.

## Attack order (by estimated paper-count payoff)

### Class A: **aaspp4.sty routing** (~49 papers)

`\documentstyle[aaspp4]{article}` ≡ `\documentclass[aaspp4]{article}`
loaded via LaTeX 2.09 compat. The Rust binding for `aaspp4.sty` should
load `aas_support_sty.rs` bindings (which already define `\affil`,
`\altaffilmark`, `\altaffiltext`, etc.) as its first act.

**Location:** `latexml_package/src/package/aaspp_sty.rs`
**Fix:** `RequirePackage!("aas_support")` at the top of the binding, or
inline the necessary `\affil`/`\altaffil*` definitions directly.

### Class B: **pgfmath empty-token `\ifdim` infinite loop** (~2 papers)

`\ifdim #1pt<3pt` with `#1` expanding to empty → `\ifdim pt<3pt` →
garbage → "expected: <number>" → treated as zero → same loop → OOM.

**Location:** `latexml_package/src/package/pgfmath_code_tex.rs`
**Fix:** guard the `\ifdim` expansion to short-circuit when the
operand is empty, matching Perl upstream commit `dfeeb1b8` (already
synced but may not cover the empty-token edge case).

### Class C: **babel-french `\bbl@exp@aux`** (~2 papers)

Undefined macro from the expl3-style babel-french hook system.

**Location:** `latexml_package/src/package/babel_support_sty.rs` (or
a new `french_ldf.rs`).
**Fix:** stub `\bbl@exp@aux` to a no-op or proper expansion.

### Class D: **45 `malformed:ltx:` Rust emitter bugs**

These are **actual Rust bugs** producing invalid document structure.
Per-paper triage needed — most frequent malformations are:
- `malformed:ltx:bibitem` (3+) — bibitem children with wrong tag
- `malformed:ltx:section` (1) — section structure break

**Approach:** pick one at a time, minimize reproducer, fix the emitter.

### Class E: **Math-parser ambiguity timeouts** (~3 papers)

1407.5769, 1308.5727 and similar papers time out inside the Marpa math
parser. Tracked under grammar-cycles / ambiguity-pruning work.

### Class F: **CPU-contention timeouts at `-j 16`** (~38 papers)

Fixable by dropping to `-j 8` for the benchmark (baseline config) OR by
making the core conversion faster so 60 s suffices under `-j 16`. The
user asked for **good performance** — per-paper wall time reduction is
a parallel workstream. Candidates:
- arena churn audit (SYNC_STATUS D4)
- SymHashMap migration (SYNC_STATUS D4)
- Scan/CrossRef post-processing wins (cycles 239/241) — already landed

### Class G: **xparse 2018+/pgfplots/cleveref (1 paper currently, many
conversion_errors secretly)**

1803.03288 explicitly names them. Large-scope; needs dedicated session
for each package.

## Environment hygiene

The local dev environment drifts from CI's apt-installed TeXLive 2023.
`tools/test_with_tl2023.sh` runs the Rust tests under `~/data/texlive2023/`
to reproduce CI behavior (documented in that file's commit).

## Open tasks — checklist

- [ ] Class A: aaspp4.sty → aas_support auto-load (~49 papers)
- [ ] Class B: pgfmath empty-arg `\ifdim` guard (~2 papers)
- [ ] Class C: babel-french `\bbl@exp@aux` stub (~2 papers)
- [ ] Class D: 45 `malformed:ltx:` triage, one-by-one
- [ ] Class E: math-parser ambiguity timeouts (grammar refactor)
- [ ] Class F: `-j 16` CPU-contention ~38 papers — perf audit
- [ ] Class G: xparse-2018+, pgfplots, cleveref bindings (1+ papers)
- [ ] 1112.6246 OOM root cause
- [ ] `\psfig`, `\plotone`, `\plotfiddle`, `\tableline`, `\opening`,
      `\figcaption`, `\lefthead`, `\righthead` — AASTeX-specific stubs
- [ ] AMS Plain-TeX headers: `\topmatter`/`\Cal`/`\Refs` (~8 papers)
- [ ] `\keywords`, `\address`, `\references` — high-count generic stubs
      (29 + 19 + 17 = 65 incidents; need class-aware binding)
- [ ] Math symbols `\lesssim`/`\gtrsim`/`\la`/`\ga`/`\sun`/`\arcsec`/`\arcdeg`
      — add to `aas_support_sty.rs` or `aastex_sty.rs`
