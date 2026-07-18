# ar5iv issue mini-sprint — diagnostic sweep (2026-07-18)

**Purpose.** The [`dginev/ar5iv`](https://github.com/dginev/ar5iv/issues) tracker
holds 100 open "Improve article X" reports. Each was filed against the **Perl**
LaTeXML HTML that ar5iv served at the time. We now serve **latexml-oxide** (Rust),
which may already have fixed some. This doc screens every open-issue paper against
the *current* Rust binary, classifies each vs same-host Perl, and produces the
ranked worklist for the follow-on implementation sprint.

> Diagnostic snapshot — dated per the CLAUDE.md naming rule. Regenerate before
> re-planning; do not treat as a live worklist once acted on. Branch
> `ar5iv-minisprint`.

## Method

- Source: `/data/arxiv/<YYMM>/<id>/<id>.zip`, copied to a scratch dir and
  converted from within it (so local `.cls`/`.sty` resolve).
- Rust command (the ar5iv site configuration):
  `latexml_oxide --preload=ar5iv.sty --nodefaultresources --path=~/git/ar5iv-css/css --css=ar5iv.css --dest=out.html <main>`
  under the debug build, 150 s timeout.
- Perl baseline (classification): `latexml --path=~/git/ar5iv-bindings/bindings
  --preload=ar5iv.sty --dest=perl.xml <main>`, v0.8.8, 200 s timeout.
- Signals: ANSI-stripped `^Error:` / `^Fatal:` counts, exit code (124 = timeout),
  HTML size, top error categories. Ground truth spot-checked with `pdflatex`
  (`.log` + PDF).

**Caveat on Perl timeouts.** The Perl baseline ran under 10× parallelism, so many
`rc=124` (timeout) rows are contention artifacts, not true 1× timeouts. The
**reliable** signal is the *error-count* comparison where both engines complete;
"Perl times out" rows should be re-checked at 1× before claiming a win. The Perl
`101e/1f` rows are its `too_many_errors` fatal (>100 errors) — genuine Perl
failure.

## Verdict tally (by distinct paper; some papers span multiple issues)

| class | papers | meaning |
|---|---|---|
| **CLEAN** | 46 | Rust converts 0-error — issue effectively already resolved |
| **RUST-BETTER** | 25 | Rust completes where Perl fatals/times out (several still have reducible Rust errors) |
| **PARITY** | 8 | Rust ≈ Perl error count — shared missing macro/package |
| **PARITY-TIMEOUT** | 9 | both time out — shared, hard |
| **RUST-WORSE** | 4 | genuine Rust regression — top priority |
| **??** | 2 | #518 feature (dark mode, ar5iv frontend); #537 no main file detected |

Of 100 issues: **~48 issues already convert clean** in latexml-oxide (46 papers +
duplicate-issue coverage). The actionable engine work is the RUST-WORSE set plus
the high-error RUST-BETTER papers (surpass-Perl, real-LaTeX-correct).

## Root-cause clusters (drives the worklist)

1. **`dvipsnames` global class option** — `2405.04517` (**912** errors,
   #503/#495/#474). `\documentclass[dvipsnames]{article}` + `\usepackage{xcolor}`.
   `pdflatex` loads `dvipsnam.def` once (via xcolor) and `OliveGreen` works; both
   LaTeXML engines fail. **Root-caused** (Rust): the binding's `xcolor →
   RequirePackage(color)` divergence (real xcolor is standalone) lets `color`
   process the *global* `dvipsnames` first — it loads `dvipsnam.def` **without
   `usenames`**, and the `input_definitions` load-dedup then makes xcolor's
   proper (usenames-active) load a no-op → the 68 dvips colors never register in
   xcolor's DB. The *direct* form `\usepackage[dvipsnames]{xcolor}` works (color
   never sees the option). **surpass-Perl, self-contained → Tier 1.**
2. **`fairmeta.cls` / ML meta-class cluster** — `2412.06264` (**504** `\or`),
   `2509.24704` / `2511.16624` (`\metadata`, `\contribution`, `\beginappendix`
   undefined), `2508.07407` (`Stomach:Recursion` + `\metadata`). These classes
   (fairmeta, selfevolagent) are expl3/`nicematrix`/`luabridge`-heavy; a failure
   partway through the class body leaves later `\newcommand`s undefined, and the
   `\or` flood traces to conditional/`\ifcase` parsing under the class's expl3
   code. Cross-cutting → Tier 3.
3. **`\lx@begin@alignment` / `\lx@end@inline@math` grouping** — `2311.06609`
   (**82**, RUST-WORSE, siamart), `2405.21060` (26), `2310.07298` (24),
   `2309.16609` (31), `1811.10792` (RUST-WORSE timeout). Inline-math / amsmath
   alignment group balance. Contains the clearest RUST-WORSE regression.
4. **`_` / `^` "script can only appear in math mode" cascade** — `2604.16007`
   (60), `2305.05665` (33, `axessibility`), `1404.3143` (ytableau, **parity** —
   Perl fatals worse), `2312.11805`, `2408.15403`. Mixed roots; `axessibility`
   (redefines `_`/`^`) is a recurring suspect.
5. **`\else`/`\fi` flood (minted)** — `2602.15902` (**783**, RUST-WORSE).
   `minted`/`fvextra` conditional machinery.
6. **`malformed:ltx:subsection/section` nesting** — `2402.13846` (16),
   `2507.00833` (8). Sectioning depth / float placement.
7. **`{forest}` undefined** — `2107.13586`, `2511.18538`, `2605.12090`,
   `2505.01658`. The `forest` tree-drawing package (unbound; often also a
   timeout). Largely parity.
8. **tabular grouping** — `2301.12995` (16 `\@end@tabular`).
9. **Timeouts (11)** — mostly **parity** (both engines time out: forest, tikz,
   tcolorbox, pgf). `1811.10792` and `2310.17416` are **Rust-only** timeouts.
10. **Single missing macro (parity)** — `\BibSpecAlias`, `\titlehead`,
    `{refsegment}`/biblatex, `\bfR`, etc. Both engines fail identically.

## Ranked worklist for the implementation sprint

- **Tier 1 — land now (confirmed, high-value, self-contained):**
  - [x] **LANDED** `dvipsnames` global class option → xcolor (2405.04517, **912
    → 0** err, #503/#495/#474). Fix: `xcolor` flags `xcolor_driving` around its
    `RequirePackage(color)` so `color` defers its eager `dvipsnam.def` load to
    xcolor's authoritative (usenames-active) load — matching pdflatex's one-load
    outcome. Guard: `tests/graphics/xcolor_global_dvipsnames`.
- **Tier 2 — genuine RUST-WORSE regressions:**
  - [ ] `2602.15902` minted `\else/\fi` flood (783).
  - [ ] `2311.06609` inline-math/alignment grouping (82, siamart).
  - [ ] `1811.10792`, `2310.17416` Rust-only timeouts (min-repro the hot loop).
- **Tier 3 — recurring clusters (surpass-Perl):**
  - [ ] fairmeta meta-class family (`\or`, `\metadata`) — 2412.06264 + 3 more.
  - [ ] `_`/`^` cascade — axessibility path (2305.05665) + 2604.16007.
  - [ ] malformed sectioning (2402.13846, 2507.00833).
- **Tier 4 — verify + comment/close the 48 already-clean issues** (batch).
- **Tier 5 — document as parity / shared-Perl** (PARITY + PARITY-TIMEOUT +
  single-missing-macro): note on the issue, no engine change unless a cheap
  shared fix (record in `KNOWN_PERL_ERRORS.md`).

## Full results

<!-- BEGIN GENERATED TABLE -->
### RUST-WORSE (4 papers)

| issue(s) | arxiv | rust | perl | top rust errors |
|---|---|---|---|---|
| #591 | 2602.15902 | ERRORS 783 | 101e/1f | 184×Error:unexpected:\else; 184×Error:unexpected:fi; 174×Error:unexpec |
| #472 | 2311.06609 | ERRORS 82 | 17e/1f | 36×Error:unexpected:\lx@end@inline@math; 12×Error:malformed:ltx:XMTok; |
| #594 | 1811.10792 | TIMEOUT 20 | 101e/1f | 14×Error:unexpected:\lx@end@inline@math; 4×Error:unexpected:\halign; 2 |
| #473 | 2310.17416 | TIMEOUT 5 | 101e/1f | 3×Error:unexpected:_; 2×Error:latex:\GenericError; 1×Fatal:Timeout:Con |

### RUST-BETTER (25 papers)

| issue(s) | arxiv | rust | perl | top rust errors |
|---|---|---|---|---|
| #474, #495, #503 | 2405.04517 | ERRORS 912 | 0e/1f/TO | 912×Error:unexpected:OliveGreen |
| #520 | 2412.06264 | ERRORS 504 | 2e/1f/TO | 353×Error:unexpected:\or; 3×Error:unexpected:\lx@end@display@math; 1×E |
| #597 | 1404.3143 | ERRORS 69 | 101e/1f | 32×Error:unexpected:_; 16×Error:unexpected:^; 9×Error:unexpected:\lx@b |
| #557 | 2305.05665 | ERRORS 33 | 0e/1f/TO | 33×Error:unexpected:_ |
| #568 | 2309.16609 | ERRORS 31 | 0e/1f/TO | 18×Error:unexpected:\lx@begin@alignment; 3×Error:unexpected:\endgroup; |
| #497, #516 | 2405.21060 | ERRORS 26 | 101e/1f | 8×Error:unexpected:\lx@end@inline@math; 6×Error:unexpected:\lx@begin@a |
| #477 | 2310.07298 | ERRORS 24 | 0e/1f/TO | 11×Error:unexpected:\lx@end@inline@math; 3×Error:unexpected:\lx@begin@ |
| #504 | 2402.13846 | ERRORS 16 | 2e/1f/TO | 9×Error:malformed:ltx:subsection; 3×Error:malformed:ltx:section; 2×Err |
| #558 | 2301.12995 | ERRORS 16 | 0e/1f/TO | 16×Error:unexpected:\@end@tabular |
| #605 | 2605.12090 | ERRORS 14 | 2e/1f/TO | 12×Error:unexpected:openmossblue; 1×Error:undefined:\checkdata; 1×Erro |
| #569, #570 | 2507.00833 | ERRORS 8 | 0e/1f/TO | 5×Error:malformed:ltx:subsection; 1×Error:malformed:ltx:section; 1×Err |
| #538 | 2003.03231 | ERRORS 7 | 101e/1f | 3×Error:unexpected:\caption; 1×Error:undefined:{sidewaystable}; 1×Erro |
| #483 | 2312.11805 | ERRORS 6 | 0e/1f/TO | 4×Error:unexpected:_; 1×Error:undefined:\reportnumber; 1×Error:malform |
| #567 | 2509.24704 | ERRORS 5 | 2e/1f/TO | 1×Error:undefined:\g_luabridge_method_int; 1×Error:expected:<relationa |
| #576 | 2511.16624 | ERRORS 4 | 2e/1f/TO | 1×Error:undefined:\contribution; 1×Error:undefined:\metadata; 1×Error: |
| #573 | 2511.18538 | ERRORS 3 | 0e/1f/TO | 2×Error:unexpected:_; 1×Error:undefined:{forest} |
| #556 | 2508.07407 | FATAL 2 | 2e/1f/TO | 1×Error:undefined:\contribution; 1×Error:undefined:\metadata; 1×Fatal: |
| #476 | 2107.13586 | ERRORS 1 | 0e/1f/TO | 1×Error:undefined:{forest} |
| #482 | 2310.06461 | ERRORS 1 | 0e/1f/TO | 1×Error:malformed:ltx:bibitem |
| #498 | 2305.01582 | ERRORS 1 | 0e/1f/TO | 1×Error:undefined:\titlehead |
| #499 | 2405.08669 | ERRORS 1 | 0e/1f/TO | 1×Error:undefined:{pNiceMatrix} |
| #523 | 2408.15403 | ERRORS 1 | 1e/1f/TO | 1×Error:unexpected:_ |
| #527 | 2410.19788 | ERRORS 1 | 0e/1f/TO | 1×Error:undefined:\fail |
| #555 | 2408.08435 | ERRORS 1 | 0e/1f/TO | 1×Error:undefined:\sidecaptionvpos |
| #566 | 2407.16741 | ERRORS 1 | 63e/0f | 1×Error:malformed:ltx:itemize |

### PARITY (8 papers)

| issue(s) | arxiv | rust | perl | top rust errors |
|---|---|---|---|---|
| #601 | 2604.16007 | ERRORS 63 | 89e/0f | 60×Error:unexpected:_; 1×Error:undefined:\@ACM@balancefalse; 1×Error:u |
| #493 | 1705.07115 | ERRORS 8 | 8e/0f | 4×Error:unexpected:_; 2×Error:malformed:ltx:section; 2×Error:malformed |
| #584 | 2511.20436 | ERRORS 7 | 7e/0f | 1×Error:undefined:\onehalfspacing; 1×Error:undefined:\prof; 1×Error:un |
| #484 | 2311.14451 | ERRORS 3 | 3e/0f | 2×Error:unexpected:\noalign; 1×Error:undefined:\bfR |
| #554 | 2406.02507 | ERRORS 2 | 2e/0f | 2×Error:malformed:ltx:listing |
| #580 | 2112.06778 | ERRORS 2 | 2e/0f | 1×Error:undefined:{refsegment}; 1×Error:undefined:\defbibfilter |
| #485 | 2302.08557 | ERRORS 1 | 1e/0f | 1×Error:undefined:\BibSpecAlias |
| #585 | 1802.09089 | ERRORS 1 | 1e/0f | 1×Error:unexpected:_ |

### PARITY-TIMEOUT (9 papers)

| issue(s) | arxiv | rust | perl | top rust errors |
|---|---|---|---|---|
| #471 | 2308.04512 | TIMEOUT 7 | 0e/1f/TO | 3×Error:expected:<variable>; 1×Error:undefined:\tablinesep; 1×Error:un |
| #596 | 2505.01658 | TIMEOUT 1 | 0e/1f/TO | 1×Error:undefined:{forest}; 1×Fatal:Timeout:Convert |
| #522 | 2405.19920 | TIMEOUT 0 | 0e/1f/TO | 1×Fatal:Timeout:Convert |
| #533 | 2406.15882 | TIMEOUT 0 | 0e/1f/TO | 1×Fatal:Timeout:Convert |
| #546 | 2504.07033 | TIMEOUT 0 | 0e/1f/TO | 1×Fatal:Timeout:Convert |
| #550 | 2501.09223 | TIMEOUT 0 | 0e/1f/TO | 1×Fatal:Timeout:Convert |
| #551 | 2501.10235 | TIMEOUT 0 | 4e/1f/TO | 1×Fatal:Timeout:Convert |
| #598 | 1611.02087 | TIMEOUT 0 | 0e/1f/TO | 1×Fatal:Timeout:Convert |
| #599 | 1802.01134 | TIMEOUT 0 | 0e/1f/TO | 1×Fatal:Timeout:Convert |

### CLEAN (46 papers)

| issue(s) | arxiv | rust | perl | top rust errors |
|---|---|---|---|---|
| #478 | 2402.16499 | OK 0 | -e/-f |  |
| #480 | 2403.13327 | OK 0 | -e/-f |  |
| #486 | 2311.11571 | OK 0 | -e/-f |  |
| #487 | 2306.11825 | OK 0 | -e/-f |  |
| #490 | 2310.00718 | OK 0 | -e/-f |  |
| #491 | 2305.15121 | OK 0 | -e/-f |  |
| #494 | 2305.15852 | OK 0 | -e/-f |  |
| #496 | 2210.04610 | OK 0 | -e/-f |  |
| #500 | 2409.10821 | OK 0 | -e/-f |  |
| #501 | 2305.20086 | OK 0 | -e/-f |  |
| #502 | 1612.01474 | OK 0 | -e/-f |  |
| #505 | 2405.14205 | OK 0 | -e/-f |  |
| #508 | 2401.11605 | OK 0 | -e/-f |  |
| #511 | 1806.03335 | OK 0 | -e/-f |  |
| #512 | 2401.00905 | OK 0 | -e/-f |  |
| #513 | 2405.03553 | OK 0 | -e/-f |  |
| #515 | 2409.01392 | OK 0 | -e/-f |  |
| #517 | 1701.02434 | OK 0 | -e/-f |  |
| #519 | 2406.16976 | OK 0 | -e/-f |  |
| #521 | 1610.06545 | OK 0 | -e/-f |  |
| #525 | 2412.13420 | OK 0 | -e/-f |  |
| #526 | 2502.13923 | OK 0 | -e/-f |  |
| #528 | 2110.02178 | OK 0 | -e/-f |  |
| #529 | 2502.16671 | OK 0 | -e/-f |  |
| #530 | 2410.07745 | OK 0 | -e/-f |  |
| #531 | 2405.16376 | OK 0 | -e/-f |  |
| #532, #563 | 2503.20215 | OK 0 | -e/-f |  |
| #534, #590 | 2412.15115 | OK 0 | -e/-f |  |
| #535 | 2103.14899 | OK 0 | -e/-f |  |
| #536 | 2407.09394 | OK 0 | -e/-f |  |
| #539 | 2304.14646 | OK 0 | -e/-f |  |
| #540 | 2502.08235 | OK 0 | -e/-f |  |
| #548 | 2311.03307 | OK 0 | -e/-f |  |
| #552 | 2507.19457 | OK 0 | -e/-f |  |
| #560 | 2406.16860 | OK 0 | -e/-f |  |
| #562 | 2406.12045 | OK 0 | -e/-f |  |
| #565 | 2505.02881 | OK 0 | -e/-f |  |
| #571 | 1406.4858 | OK 0 | -e/-f |  |
| #572 | 1307.6856 | OK 0 | -e/-f |  |
| #578 | 1509.03700 | OK 0 | -e/-f |  |
| #581 | 1604.00449 | OK 0 | -e/-f |  |
| #582 | math/9405204 | OK 0 | -e/-f |  |
| #587 | 2505.22648 | OK 0 | -e/-f |  |
| #592 | 2602.20089 | OK 0 | -e/-f |  |
| #595 | 2505.11584 | OK 0 | -e/-f |  |
| #600 | 2507.00769 | OK 0 | -e/-f |  |

### ?? (2 papers)

| issue(s) | arxiv | rust | perl | top rust errors |
|---|---|---|---|---|
| #518 | NONE | NOLOG 0 | - |  |
| #537 | 1606.05250 | NOLOG -1 | -e/-f |  |
<!-- END GENERATED TABLE -->
