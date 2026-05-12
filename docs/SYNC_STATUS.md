# Engine Sync Status — Active Worklist

**Mission**: 100k "no-problem" sandbox parity. A paper is in scope iff
Perl LaTeXML on TL2025 with `--preload=ar5iv.sty
--path=~/git/ar5iv-bindings/bindings` produces 0 errors. Mission completes
when every in-scope paper produces 0 errors on Rust too.

**Status**: Round-25 stages 1-43 (426,555-paper arxmliv corpus)
**closed 2026-05-12**. 30 RUST-REGRESSIONs fixed; ~15 deferred.
Stage 41 hit **100.00% OK**. Aggregate ~99.85%. Next focus: retire
hand-stub bindings via raw-load (xfor → mfirstuc → datatool-base →
glossaries — see "Planned" below).

### Round-25 active worklist

`cargo test --tests` = **1185/0/0** (post-rebase onto master commit
`bffd1be471`, +schema-docs + split post-processor).

**Round-25 landings** (compressed):

| Commit | Driver | What |
|---|---|---|
| `488ed74c41` | 2001.07651, 1807.04759 | mn2e_support `\ion` Perl-parity `\text{...}` wrap (`\$\beta\$` math toggles safe) |
| `7edfb8eeb1` | 7-paper hang | `latexml_contrib::scicite_sty` (Science journal cite stub) |
| `588ad90263`+`1d21ee0d29` | 60-paper expl3 cluster | `input_definitions` `@currname` leak on handleoptions=false; Perl `Package.pm:2580-2611` parity fix |
| `8ac3eae2c4` | post-cleanup | removed redundant `tex_file_io`/`xy_sty` `@currname` wrappers (no-ops after the fix above) |
| Master rebase 2026-05-10 | — | branch onto `bffd1be471` (schema-docs + split post-processor); 12 commits replayed, 1185/0/0 |
| `488ed74c41`–`be45566b7e` | session 4 | `\ion` + math-CS protected flags + cleveref×hyperref dispatch + recursion guard + `\genfrac` raw readArg |
| `662571777f`+`92c1a40850`+`6c9ad70d38` | glossaries chain | mfirstuc + datatool-base + chemgreek + substr raw-load shims |

**Format dump enabled 2026-05-08** (`resources/dumps/latex.dump.txt`,
25,439 entries, 3.9 MB, 389 expl3 markers). Dump path 5 in
`latex.rs::latex_dump_available`. With dump present, expl3.sty's
`\ifx\csname tex_let:D\endcsname\relax` short-circuits the raw
`\input expl3-code.tex`. Perf: 30-cluster avg ~10× faster, peak 46×
(`\usetikzlibrary{calligraphy}` 28.1s → 0.6s). Tests stable; dump
file is gitignored.

**"Core dump" investigations closed**: 1607.04981 (LyX/babel/hyperref,
~90s) and 1506.04659 (harvmac/epsf) are NOT panics — internal 60s
watchdog SIGABRT mislabelled by `timeout` as "dumped core". Slow,
not crashing.

## Planned: replace hand-stub bindings with raw-load (2026-05-11)

**Strategic direction** (user feedback 2026-05-11): when Rust ships a
`*_sty.rs` hand-stub that intercepts the actual TexLive `*.sty`
file because raw-load fails on expl3/xparse/l3* emulation gaps, the
**proper** fix is to **improve the Rust engine until raw-load works**,
not to extend the stub indefinitely. Perl LaTeXML raw-loads these
packages successfully via its mature expl3 emulation; every stub we
ship is a divergence from Perl and an accumulator of incomplete
coverage. See [`memory/feedback_prefer_raw_load.md`].

**Affected stubs and gap measurements:**

### `latexml_contrib/src/mhchem_sty.rs`
- Intercepts: TL `mhchem.sty` (~640 lines).
- Real chain: `chemgreek` → `xparse` → expl3 (`\group_begin:`,
  `\__file_tmp:w`, l3regex, l3tl-analysis).
- Status: existing TODO at file head says "DELETE this binding
  once engine can faithfully handle the expl3/xparse/chemgreek
  raw-load chain". Driver: arXiv:1806.06448.
- **Measured gap (2026-05-12)**: raw-load probe (mhchem stub
  temporarily replaced with `InputDefinitions("mhchem", noltxml=>1)`)
  on a `\ce{H2O}` paper produces **92 errors**, all in the
  expl3 emulation layer:
    * `\exp_args:Nc` between `\csname`/`\endcsname` (Error:unexpected)
    * `\scan_stop:`, `\s__tl`, `\tex_skip:D` between csname/endcsname
    * `\exp_stop_f:` undefined
    * `\fi:` appearing outside conditional (Error:unexpected:fi)
    * `<relationaltoken>` expected (numeric comparison gaps)
  Probe restored; the contrib stub remains the load-bearing path.
  Chemgreek shim added (`chemgreek_sty.rs`) so direct
  `\usepackage{chemgreek}` does raw-load — but mhchem itself
  remains stubbed.
- Engine work needed: faithful `\exp_*` / `\__tl_*` / `\__file_*` /
  `\scan_stop:` / `\group_begin:` etc. — i.e. proper csname-time
  exp-and-mark protocol from expl3. Tracked in Round-26 candidates.

### `latexml_package/src/package/glossaries_sty.rs` — **DONE 2026-05-12**
- Intercepts: TL `glossaries.sty` (7714 lines as of TL2025).
- **Status: production raw-load**. Commit `3883d4d14d` swapped from
  1140-line hand-stub to 129-line strict translation of Perl
  `glossaries.sty.ltxml` (which raw-loads TL glossaries.sty via
  `InputDefinitions(noltxml=>1)`). This session's add-ons closed
  the remaining dependency gaps:
  - `662571777f` — mfirstuc + datatool-base raw-load shims
  - `92c1a40850` — chemgreek raw-load shim
  - `6c9ad70d38` — substr raw-load shim
  Surgical overrides remain in `glossaries_sty.rs` for `\@gls@link`
  → `<ltx:glossaryref>`, `\@newglossaryentryposthook` →
  `<ltx:glossarydefinition>`, `\printglossary` → `<ltx:glossary>`.
- **End-to-end verification (2026-05-12)**: elsarticle + glossaries +
  3-column tabular with `\gls`/`\acrshort` → **0 errors**. Three
  styles (long/list/tree) all clean. Witness 1910.01256 Chrome
  preview byte-for-byte matches Perl latexmlc `--format=html`.

### Plan of attack

1. ~~**Foundation pass — expl3 / l3* emulation for glossaries
   chain**~~ **DONE 2026-05-12** (xfor + mfirstuc + datatool-base
   + chemgreek + substr + tracklang shims; transitive substr /
   datatool-fp / fp-* / glossary-long / glossary-super /
   glossary-list / glossary-tree / glossary-hypernav all load
   0-error).
2. **mhchem retirement**: blocked by **92-error expl3
   csname-protocol gap** (measured 2026-05-12). Specific
   primitives: `\exp_args:Nc`, `\scan_stop:`, `\s__tl`,
   `\tex_skip:D`, `\exp_stop_f:`, csname-time `\fi:`, relational
   gaps. Engine work tracked as Round-26 candidate.
3. **Regression guard**. When a new `\<missing-cs>` error
   surfaces in a paper that loads a currently-stubbed package,
   document the gap rather than land a no-op stub. Witnesses
   that BLOCK stage advance may still get a stub as an interim
   fix; commit body should note "interim".

## SHARED-FAILURE log (Perl + Rust both fail identically)

- **`\def\<one-letter-CS>` before `\documentclass`** — user code like
  `\def \d {\delta}`, `\def \th {\theta}`, `\def \b {\beta}` placed
  before `\documentclass{<class>}` is silently overwritten when the
  LaTeX kernel loads (e.g. `\d` becomes `\d{...}` text-accent;
  `\th` becomes thorn). Inside subsequent `$\d_x$` math, the
  unintended kernel definition trips text-mode underscore.
  Witnesses (stage 1 verify, mini-canvas):
    * hep-th0005159 — Rust 99 / Perl 101 errors + 1 fatal
    * hep-th0010165 — Rust 92 / Perl 101 errors + 1 fatal
    * hep-ph0001306 — Rust 75 / Perl 101 errors + 1 fatal
    * cond-mat0102064 — Rust 4 / Perl 4 errors
    * cond-mat0103632 — Rust 20 / Perl 20 errors
    * hep-th0005268 — Rust 11 / Perl 26 errors
  Together: the entire residual `expected:$` (191) + the bulk of
  residual `_/^` clusters on stage 1. Both engines fail identically
  on the fatal-cascade boundary. SHARED-FAILURE; out of scope.

- **pstricks `\ifpst@useCalc` / `\ifpst@psfonts` undefined** — when a
  paper `\input`s `pstricks-dots.tex` (or other pstricks subfiles)
  before `pstricks-tex.def` has run, the `\newif` conditionals
  defined in pstricks-tex.def are missing. Both Perl and Rust emit
  the identical pair of `Error:undefined:\ifpst@*` events. Witnesses:
  astro-ph0002346, astro-ph0002348. SHARED-FAILURE.

- ~~**AmSTeX `\@` undefined (`\input amstex` + `\documentstyle{amsppt}`)**~~
  **RESOLVED 2026-05-11** (commit `1cb3c81a6d`): both ports were
  shared-failing because amstex.tex L165 (`\edef\@{\string @}`) was
  unmirrored in our AmSTeX pool. Adding `DefMacro!("\\@", "@")` in
  `amstex.rs` fixes 36 papers across the canvas (math-ph0001012/15,
  math0209244, math0311498, …, 2012.06011, 1809.08150). SURPASS-PERL,
  but a faithful translation of the canonical AmSTeX `.tex` file.
  See `docs/KNOWN_PERL_ERRORS.md` §21 for the Perl-upstream gap.

- **amsart `_/^` cascade after `\maketitle` / `\numberwithin{equation}{section}`**
  — math0010241 (`amsart` with `\numberwithin{equation}{section}`)
  emits 8 `Error:malformed:ltx:XMArray` + 19-ish `_/^` cascade. Perl
  emits 19 errors + 22 warnings on same paper. SHARED-FAILURE.

- **plain-TeX `\input psfig.sty` reload mid-document** — papers using
  plain TeX (no `\documentclass`) with multiple `\input psfig.sty`
  invocations scattered through the body. The first `\input` loads
  the binding (RequirePackage epsfig → defines `\psfig`); subsequent
  `\input`s hit a reload path that unconditionally re-routes through
  the raw `psfig.sty` on disk, where mid-file plain-TeX constructs
  expect a `\hbox`/`\vbox` build context that LaTeXML cannot provide.
  Perl LaTeXML hits the identical `Error:undefined:\psfig` at the
  exact same source line (255 col 1). Witnesses: cond-mat0010356,
  cond-mat0101405. SHARED-FAILURE.

## ~~Known engine gap: cleveref × algorithmicx × hyperref infinite-loop~~

**RESOLVED 2026-05-11** (two-part Perl-parity fix). Witness 2403.15855
(Springer Nature `sn-jnl`) now converts cleanly; 8-line minimal repro
(algpseudocode + hyperref + cleveref + `\begin{algorithmic}\item a`)
finishes in <1 s with no errors. `cargo test --workspace --tests
--no-fail-fast` = 1185/0/0.

**Two layers were needed** — neither alone is sufficient, both reflect
faithful Perl behaviour:

1. **`\refstepcounter → \H@refstepcounter` dispatch** in
   `latexml_package::hyperref_sty`. Real `hyperref.sty:6631+6638-6657`
   does:
   ```tex
   \let\H@refstepcounter\refstepcounter
   \def\refstepcounter#1{ \H@refstepcounter{#1} … }
   ```
   Perl `hyperref.sty.ltxml:383` skips the `\def` — it relies on a
   Perl-side recursion guard (next bullet) to keep cleveref happy.
   We instead mirror real hyperref: `Let` + `DefMacro!("\\refstepcounter
   {}", "\\H@refstepcounter{#1}")`. This is principled because
   downstream packages (notably cleveref `cleveref.sty:2045-2053`)
   patch `\H@refstepcounter` to set `\cref@currentlabel`, and that
   patch only fires if `\refstepcounter` actually dispatches through
   `\H@refstepcounter`. Without the dispatch, `\cref@currentlabel`
   retained its `\ALG@beginalgorithmic` placeholder
   `[line][\arabic{ALG@line}][\cref@currentprefix]\theALG@line`, the
   `\@@cref@getprefix` body did `\def\cref@currentprefix
   {\cref@currentprefix}` (self-ref), and `\xdef
   \cref@currentprefix{\cref@currentprefix}` looped.

2. **Self-recursion guard fixed in `latexml_core::definition::expandable`**.
   Perl `Expandable.pm:81-89` errors with
   `Token X expands into itself!` and substitutes empty tokens for the
   invocation. The Rust port already detected the recursion but tried
   to "fix" it by `assign_meaning(self.cs, Stored::Token(self.cs))` —
   a NO-OP because `assign_meaning` short-circuits on `token == mt`
   (state.rs:1918-1922). The Expandable definition stayed in place
   and the guard re-fired forever. Replaced with the Perl strategy:
   `Error!("recursion", cs, "Token X expands into itself!"); Tokens!()`.
   Identity for expl3 quarks (`\q_no_value`, …) is preserved because
   quarks are `\cs_new_protected:Npn` — protected expandables aren't
   expanded under the partial-expansion path, so the guard never
   fires; `\ifx`-by-meaning stays distinct.

The two layers are complementary: (1) fixes the *cause* of the
runaway expansion (cleveref's patch now fires properly); (2) is the
*safety net* — any other downstream package that hits a similar
`\def\foo{\foo}` situation gets a visible error instead of a hang.

Driver: 2403.15855 (Springer Nature `sn-jnl` class).
Files: `latexml_package/src/package/hyperref_sty.rs`,
`latexml_core/src/definition/expandable.rs`.

## ~~Known engine gap: `\vtop` × `\gls{...}` × `p{}` tabular column~~

**RESOLVED 2026-05-12** (silent side-effect of glossaries raw-load
chain rewrite). Driver 2210.13325 and minimal repro:

```tex
\documentclass{article}
\usepackage[acronym]{glossaries}
\newacronym{ddos}{DDoS}{Distributed Denial of Service}
\begin{document}
\begin{tabular}{|p{5cm}|} \hline
\gls{ddos} \\ \hline
\end{tabular}
\end{document}
```

now converts at **0 errors** (was 7 errors with cascade starting
`Error:unexpected:\vtop Attempt to end mode internal_vertical`).
Stress test with elsarticle + 3-column tabular + 3 acronyms via
`\gls`/`\acrshort` likewise 0 errors. Output is properly nested
`<tabular><tbody><tr><td><inline-block><p><glossaryref>...`.

Likely fixed-by-chain: the etoolbox `&`-catcode-3 trick that gated
the original cascade was only relevant because the prior hand-stub
glossaries binding routed through `\ifdefempty`/`\ifdefparam`. The
commit-`3883d4d14d` rewrite swapped to raw-load TL glossaries.sty
via `InputDefinitions(noltxml=>1)`, taking a completely different
`\gls`-implementation path that doesn't trip the `\edef`-frozen-
catcode quirk. The etoolbox `&`-catcode hypothesis stands as the
likely-root if it ever resurfaces from another driver.

## Round-25 canvas stages 1-43 (2026-05-10 → 2026-05-12)

Mini-sandbox triage walked the entire 426,555-paper arxmliv corpus in
44 staged 10k slices (stage 43 closes 6,555 papers). Per-stage OK%
range: **99.56% – 100.00%**. Cumulative RUST-REGRESSIONs fixed across
stages 1-43: **30**; deferred: **~15**. Stage 41 hit **100.00% OK**
(10,000/10,000). Tail-session stages 34-43 totals: 96,555 processed
→ 96,413 clean (99.85%). Per-stage detail below; verbose narratives
elided.

| Stage | OK%       | RUST-REGRESSIONs (fix SHA) | Notable RUST-CLEANER |
|-------|-----------|----------------------------|----------------------|
| 1     | sandbox triage | 0 (`52ca5d6299` binding-fallback policy) | hep-th0005268 (-5) |
| 2     | 99.91%   | 1 `\xpt`/`\ixpt` (`9673bf8b98` reverted by `ac0965abfd`; safe pt-family in `31154d0760`) | – |
| 3     | 99.84%   | 0                          | hep-th0308103 (-64) |
| 4     | 99.74%   | 0                          | – |
| 5     | 99.81%   | 0                          | astro-ph0506245 (-72) |
| 6     | 99.82%   | 0                          | gr-qc0601055 (-31) |
| 7     | 99.77%   | 0                          | 0706.2862 (-43) |
| 8     | 99.78%   | 0                          | – |
| 9     | 99.77%   | 0 (0901.0054 cascade-amp deferred) | 0809.4243 (-23) |
| 10    | 99.71%   | 0                          | 0909.3255 (-8) |
| 11    | 99.80%   | 1 siunitx v1 area/vol aliases (`a85b50ce2b`) | 1009.1106 (-7) |
| 12    | 99.61%   | 0                          | 1107.5988 (-15) |
| 13    | 99.58%   | 1 `color[usenames]` (`4c98699468`) | 1203.0262 (-17) |
| 14    | 99.63%   | 2 pstricks `\scalebox`, `\nccircle`/etc. (`cb84b8781f`) | – |
| 15    | 99.60%   | 1 mhchem→amsmath+graphicx auto-dep (`2bd41220b4`) | 1312.3586 (-9) |
| 16    | 99.59%   | 1 mn2e `{proof}` env (`1a74fc8eb1`) | 1403.6207 (-4) |
| 17    | 99.72%   | 1 `\DeclareSIUnit` SkipSpaces (`8609c8e793`) | 1501.03446 (-5) |
| 18    | 99.66%   | 0                          | 1509.05326 (-4) |
| 19    | 99.64%   | 2 siunitx hep block (`e9b7673bab`) | 1606.03888 (-15) |
| 20    | 99.70%   | 0 (5 deferred — `\colorbox`/diagrams.sty/`\color[`/`\hbox`/`\GenericError`-amp) | 1612.07821 (-3) |
| 21    | 99.63%   | 0 (4 deferred — `\GenericError` cascade-amp, listingline) | 1711.00728 (-103); 1712.01695 (-100) |
| 22    | 99.71%   | 0 (4 deferred — babel newline option, mode cascades) | 1805.03020 (-9) |
| 23    | 99.59%   | 1 glossaries stubs (`ab043cc826`; subsumed by `3883d4d14d`) | 1810.13097 (-28) |
| 24    | 99.67%   | 1 pstricks raw-load `InputDefinitions(noltxml)` (`85cf242dba`) | 1907.05384 (-101); 1907.07910 (-58) |
| 25    | 99.63%   | 3 — `\definecolorseries` signature (`087dc31aaf`); `\glsdisp` (`e22ab01185`); **glossaries rewrite raw-load** (`3883d4d14d`, 1140→129 lines) | – |
| 26    | ≥99.6%   | scicite stub (`7edfb8eeb1`); expl3 file-machinery cluster — input_definitions @currname leak (`588ad90263`+`1d21ee0d29`) | 60-paper expl3 cluster cleared |
| 27-33 | ≥99.6%   | mn2e_support `\ion` (`488ed74c41`); math-CS protected flags (`a965623dcd`); cleveref×hyperref dispatch + recursion guard (`6bb95be594`); `\genfrac` raw readArg (`be45566b7e`) | – |
| 34-43 | 99.56-**100.00%** | 4 RUST-REGRESSIONs (above) all landed; 1 deferred (wicsbook nested-trivlist, single-paper niche) | stage 41 = **100.00% OK** |

Residue at stage-43 close: ~110 SHARED-FAILURE (Perl identical:
auto-ignore/`%PDF`/plain-TeX), ~28 RUST-NONDETERMINISTIC transient
OOMs under 16-worker concurrency (converge cleanly standalone). All
30 fixed regressions match Perl semantics; see Phase B clusters
below for the residual sub-cause taxonomy.

**Post-rebase landings 2026-05-10** (compressed): 12 commits
landing the keyval cluster + siunitx CS-tokenize fix. Highlights:

- `21e730e71e` — silent-content-loss promotion: `Duplicated attribute
  xml:id` Info→Error; `Encountered unknown KeyVals key` Info→Warn.
  Witness 1410.8171: previously `[ok]` despite empty S3+; now reports
  legitimate `Status:conversion:2`. Intentional Perl divergences.
- `fc2aae7266` — siunitx `six_format_1unit`: replace `ExplodeText!`
  with `mouth::tokenize` so `\SIUnitSymbolMicro` becomes a single CS
  token, not 17 OTHER tokens. Test fixture `tests/complex/si.xml`
  regenerated 169→77 lines. Witness 1410.8171: `\SI{0,1}{\micro\kelvin}`
  now renders correctly as µK.
- **Keyval registration cluster** (8 commits, `75bab231a5` /
  `4255f5a7cd` / `254b4f54c9` / `ece08d7ea5` / `be595f4084` /
  `d27be28dc0` / `3a65bf6a88` / `571fa4ed87`): siunitx 32 bool + 45
  non-bool + 26 rounding/table SIX; hyperref ~80 Hyp + 8 graphics;
  hyperxmp 5; mathtools 16 mt; xargs 10; tabular vattach + listings;
  graphicx 20 Gin; epsfig 18 epsGin + caption 22. Net effect: warn
  reserved for genuine binding gaps; sandbox 160+ papers found
  0 residual gaps.
- **1410.8171 outcome (2026-05-10)**: `SarkanyPRArevision.tex` now
  reports `No obvious problems` (was 54 warnings + 3 errors).

## Round-20/22/23 (archived 2026-05-03 → 2026-05-08)

Three sprints closed before the Round-25 stage walk. Full narratives
moved to `docs/archive/round19_iteration_log.md`. One-paragraph each:

- **Round-20 (closed 2026-05-03)**: 100k canvas Phase A Gate 0
  closure at **99,829 / 100,003 = 99.83%** raw OK. 56 papers recovered,
  0 NEW non-OK. Key fixes: `parity_check.sh` PERL_TIMEOUT
  reclassification, `find_main_tex` `\r`-aware comment-stripper
  (cond-mat0002096, 0708.2784), `alignment.rs:add_line` autoviv on
  `\hline`/`\cline` past column count, `tests/06_cluster_regressions.rs`
  greps `Error:<class>:`. Phase D first landing `48f0c1ce8a`:
  `%auto-ignore` archives emit `Fatal:invalid:auto-ignore` (legitimate
  skip, not error).

- **Round-22 (active 2026-05-07)**: 335-paper baseline-failure
  sprint. v22 closing: **295 / 329 = 89.7%** unique OK. 24 commits
  including `9fe3e77c92` (Document::open_text `<ltx:text>` walk
  stop), `fc2ff67389` (aa_support drop spurious `\isotope`),
  `70a8f2280f` (etoolbox `DeclareListParser` `TeX!`→`RawTeX!`),
  `f53ab3ecda` (`\DeclareFontEncoding` defines `<encoding>-cmd` —
  recovers 13 T1-cmd-loop papers), defensive xml::findnodes/Node::new
  guards. Residual: math-parser stack-overflows (1904.02716/1904.10251),
  expl3 group_begin (2406.14142 — later fixed by `e436a9cda7` +
  `fedc89cabd` regex stubs), schema-strictness divergences (2211.01875,
  2301.10618, 2302.11635).

- **Round-23 (active 2026-05-07/08)**: continuation. Final v27:
  **300 / 328 = 91.5%** with `3198b744ab` natbib `\NAT@@wrout`
  bgroup/Expand-on-labels fix (2404.06289: 19→0). Other landings:
  `ad77a29f47` siunitx `\DeclareSIUnit` presentation; `fd8bb072a7`
  siunitx `\mathrm` wrap + pdftocairo png/svg fast paths;
  `5b2e38590c` `ltx:block` auto-close (2302.11635); `ba56a30a33`
  listings `\lstinline` body under verbatim catcodes (2301.10618);
  `d42de4439e` bin: `Fatal:invalid:not_tex_source` on `%PDF` magic
  (2301.04210); `1790c32b1b` `\MakeUppercase` pre-stub UTF@N@octets;
  `31b6cc1e00` lx_read_and_change_case `\dont_expand` insertion
  (2009.10018: 16→0); `e436a9cda7`+`fedc89cabd` regex stubs
  (2406.14142: 21→0 — last historical REAL_REGRESSION).

  **End-state**: 0 REAL_REGRESSIONs remain; remaining cortex-failure
  rows are RAM/post-processing capped (1904.02716, 2007.13470,
  2011.14413, 2105.04174, 2203.01231, 2310.15090). True Rust regression
  count: 0 for ported error conditions. Error/Fatal coverage audit
  in `ERROR_PARITY_AUDIT.md` notes ~43% of Perl callsites absent in
  Rust (concentrated in `latexml_post` and siunitx/pgfmath/xcolor/calc).

## Phase B clusters (the work pool)

**Re-classification after Phase A Gate 0 (2026-05-03):** every paper
in the post-fix 170-paper residual that I sampled is SHARED-FAILURE
(Rust = Perl), not a Rust-only regression. The "easy Phase B cluster
wins" the prior plan envisioned have all been harvested by round-19
or earlier. Remaining work is Phase C "surpass Perl" territory.

Sampled verdicts of remaining clusters:

| Cluster | Papers | Sample verdict | Classification |
|---|---:|---|---|
| `_/^` (Sub-cause A: `$$math$$` in horizontal mode) | 78 | Rust=Perl on all witnesses | SHARED-FAILURE / Phase C surpass-Perl |
| `\endproof` outside amsthm | 15 | All 9 originally sampled Rust=Perl | SHARED-FAILURE / Phase C |
| `\@` (at_letter scope on `\input`) | 4 | 0708.2570/0801.0329/0808.1829/0901.0353 all Rust=Perl=1 | SHARED-FAILURE / Phase C |
| `\psfig` via `\input psfig.sty` | 6 | cond-mat0010356 etc. Rust=Perl=1 | SHARED-FAILURE / Phase C (different from `\documentstyle[psfig]` already fixed) |
| `Error:expected:<box>` cascade | 26 | Mostly cascade noise from earlier errors | Phase C 1-2/day |
| `Error:expected:{` brace mismatch | 18 | User-malformed TeX | Phase C |

**Already-recovered clusters (committed)**: NBSP-in-csname (18),
`\@ifundefined` (33 — LaTeX-only), `\setdec`/`\dec` (12), `\CITE` (11),
psfig via `\documentstyle[epsfig]` (12 papers, `a6b4cb5161`). Pinned
as fixtures in `tests/06_cluster_regressions.rs`.

**`_/^` cluster sub-causes** (≈78-paper bucket — measured 2026-05-03):

Distribution from a 5-witness bisection (3 from `^,_` bucket, 2 from
bare `_`):

| # | Paper | Bucket | Source pattern | Sub-cause |
|---|---|---|---|---|
| 1 | `hep-th0009013` | `^,_` | `\begin{abstract}…$$math$$…\end{abstract}` | **A** |
| 2 | `math0010241` | `^,_` | amsart with `$$math$$` and macro-expanded math (Anonymous String) | **A** (likely; macro-expansion variant) |
| 3 | `astro-ph0203201` | `_` | `\begin{center}…$$math$$…\end{center}` | **A** |
| 4 | `cond-mat0003169` | `_` | `\CITE{IsobeUeda_deficit}` after undefined `\CITE` auto-defined as zero-arg constructor → arg digested as text group | **B** (variant) |
| 5 | `hep-lat0110168` | `_` | `\begin{center}{\small …$$math$$…}\end{center}` | **A** |

**Measured ratio: 4/5 Sub-A, 1/5 Sub-B, 0/5 Sub-C.** Consistent with
the bucket size ratio (41 `^,_` + 21 `_` + 5 `^` = 67 bare-token
papers; 13 with extra-token combinations; total ≈80, matching the 78
SYNC_STATUS estimate).

- **Sub-cause A** — `$$math$$` in non-vertical-mode (horizontal /
  restricted_horizontal). Dominant pattern (≈80% of cluster). The
  enclosing context is typically `\begin{abstract}`, `\begin{center}`,
  or `\begin{center}{\small …}`. Per `wisdom_dollar_dollar_bound_mode`,
  Rust's `\lx@dollar@default` only treats `$$` as display-math start
  when `BOUND_MODE` ends with `vertical`; in any horizontal context
  the `$$` is silently treated as text and `_/^` errors cascade.
  **Both engines fail identically** — Perl-faithful behaviour matches
  plain TeX. Surpass-Perl candidate: fall back to inline-math (`$..$`)
  when `$$` lands in horizontal mode. Requires `OXIDIZED_DESIGN`
  divergence entry.

- **Sub-cause B** — text-mode `_/^` reaching a digester arg whose
  catcodes weren't overridden. Witnesses:
  - `cond-mat0112063` — `\cite{Raimondi_etal}`, `\bibitem{us_fermionsII}`.
  - `cond-mat0003169` — `\CITE{IsobeUeda_deficit}` where `\CITE` is
    undefined and auto-defined as zero-arg constructor, so the
    `{IsobeUeda_deficit}` group is digested as text.
  Both engines fail identically. Surpass-Perl plan: switch `_/^`
  catcodes inside the key-bearing arg of `\cite`/`\bibitem` (and any
  CS that treats its arg as a key). For the auto-defined-undefined-CS
  variant, the better fix is to *consume + drop* one mandatory arg in
  the auto-defined error constructor (matches user expectation when
  the typo had a `{key}` form).

- **Sub-cause C** (revert-token serializer leak / user-class macro
  shadow) — **REMOVED 2026-05-03**: hypothetical, no witness in this
  bisection or in any prior triage. Drop from active tracking unless
  a witness emerges.

---

## PERFORMANCE.md follow-ups (separate track)

PERFORMANCE.md sets the policy for performance work. Active items
ordered by impact:

- **P0 done** — phase-attributed telemetry, telemetry.jsonl.gz, perf_phase_summary.py, perf_compare.py.
- **P1 graphics & output-heavy jobs** — biggest identifiable slow tail.
  Per-asset graphics telemetry + content-identity conversion cache +
  duplicate coalescing. Sentinels: 0809.3849, 0908.3201, 1003.0368,
  0803.4343, 0907.4282.
- **P1 math/large-document jobs** — `LATEXML_PARSE_AUDIT=1` on
  astro-ph0204009, 0911.0884, astro-ph0401354, 0809.5174,
  astro-ph0507615; rank by total parse time + repeated token sequences.
- **P1 failure/control-flow outliers** — re-run 5 timeouts with phase
  telemetry; `0903.3465` is an Xy-pic/token-limit recovery bug.
- **P2 allocation/startup cleanup** — only after profile shows hot
  path; `*_sym` accessors, `Tokens` conversions, `Stored` deep copies,
  package lookup caching, dump loading.

### Mini-benchmark: beat 2× pdflatex on 1910.01256 (badge of honor)

Goal: on the arxmliv glossaries-witness paper `1910.01256` (CVPR-style
~6-page article with `\usepackage[acronym]{glossaries}`, 110 math
formulae, .bbl included), `latexml_oxide --format html` must complete
strictly faster than two `pdflatex` passes on the same source. Beating
pdflatex on its own playing field is the badge of honor we want.

Baseline measured 2026-05-11 (release binary, fresh source dir per run):

| Pipeline                            | Real time | User    | RSS    |
|-------------------------------------|-----------|---------|--------|
| `latexml_oxide → HTML` (1 cmd)      | 3.13 s    | 3.88 s  | 225 MB |
| `pdflatex × 2`                      | 2.89 s    | 2.57 s  |  64 MB |

Headroom needed: ~0.25 s real, ~1.3 s user. RSS gap is structural
(Marpa math grammar tables + interned states) and out of scope here.

Acceptance: `latexml_oxide --format html /tmp/glo_oxide/root.tex` ≤
2.88 s real time on this machine, measured the same way (fresh dir,
release build), without regressing the `cargo test` workspace count
or any other PERFORMANCE.md baseline doc.

Likely candidates to profile first (informed by phase telemetry):
post-processing XSLT pass, math-parser amortized cost on 110
formulae, dump-replay overhead, package autoload chain for glossaries.
Use `LATEXML_PHASE_AUDIT=1` and `perf_phase_summary.py` to attribute
time before touching code.

Optimization Acceptance Checklist (PERFORMANCE.md §Optimization
Acceptance Checklist) governs every perf change.

---

## Engine file open gaps

| File | Status | Open Gap |
|------|--------|----------|
| `base_parameter_types.rs` | MINOR | `CommaList:Type` parameterized form unported (no Perl users). |
| `tex_box.rs` | MINOR | Box dimension edge cases. |
| `tex_fonts.rs` | MINOR | `\fontdimen` array semantics; per-font `\hyphenchar`. |
| `tex_tables.rs` | MINOR | Padding CSS classes (XSLT concern). |
| `plain_base.rs` | NON-BLOCKING | Closures kept in memory (always loaded before dump); dump add-only policy skips same-named entries. PA aliases capture `\let` round-trips. Architecturally documented in `latex_core/src/state.rs::is_serializable`. |
| `latex_base.rs` | NON-BLOCKING | Same architecture. Re-classified from OPEN — runtime is correct, no measured regression. |

---

## Tikz known diffs vs Perl

1. foreignObject transform Y / width/height
2. Arrow tip shape (different path data)
3. SVG viewBox / total width differs slightly
4. tikz matrix uses `<svg:g class="ltx_tikzmatrix">` (Rust) vs inline-blocks (Perl)

---

## Permanent ignores

* **Sandbox out-of-scope:** ns1–ns5 (52_namespace, no DTD); 2402.03300, 2410.10068, 2511.03798 (Perl also fails).
* **Rust supersedes Perl** (both still in scope, but Rust passes where Perl errors): `1207.6068`, `0909.3444`, plus 40+ papers identified in round-19 sweep (memory: `project_rust_supersedes_perl.md`).
* **Unported pools:** `BibTeX.pool.ltxml` (skipped via `--nobibtex`).

---

## Acceptance gates

| Gate | Current | Target |
|---|---|---|
| `cargo test --tests` | **1185/0/0** | unchanged across all task work |
| `latexml_oxide --init=plain.tex` | 0 errors | 0 errors |
| `latexml_oxide --init=latex.ltx` | 0 errors | 0 errors |
| 420k arxmliv canvas (stages 1-43) | **99.56-100.00% per stage**, stage 41 = **100.00%**, ~99.85% aggregate | 100% match Perl |
| Round-25 cumulative regressions | **30 fixed, ~15 deferred** (most are single-paper niche or cascade-amplification) | drive deferred set to zero |

---

## Distribution follow-up

Once TL2025 dumps stay robust through a CI cycle: `include_bytes!`
`{plain,latex}.dump.txt` for TL2022…TL2026 and select at runtime by
`kpsewhich --version`. Currently dumps load from `resources/dumps/`.

---

## Long-term: consolidate post-processing graphics renderer

Currently the post-processing graphics pipeline shells out to **four**
external tools depending on source format and target asset:
`convert` / `gs` (ImageMagick → Ghostscript) for PDF→PNG fallback,
`inkscape` for PDF→SVG fallback, `pdftocairo` for the fast PDF→PNG
and PDF→SVG paths (added 2026-05-07), and `ps2pdf` + `pdftocairo` for
EPS→PNG. Each adds a runtime dependency, fork cost, and timeout
plumbing; the convert/gs path in particular is 25-50× slower than
`pdftocairo` on vector-heavy PDFs and produces no better output.

Goal: **converge on a single primary renderer**, with a clearly-scoped
fallback (or none). Two candidates worth evaluating:

1. **`pdftocairo` (poppler)** as the sole subprocess renderer.
   Empirically the fastest, available wherever TeXLive is, produces
   clean PNG output and acceptable SVG for all benign PDFs we've
   measured. SVG output explodes on R-Graphics-class PDFs, but the
   8 MB size guard + PNG fallback already handles that case. Pros:
   no new build dependency; binary already on the path.
   Cons: still a subprocess, not a Rust crate.

2. **`pdfium-render`** (Rust crate wrapping Google's PDFium). Pure
   in-process rendering; same engine that powers Chrome's PDF view.
   Mature for raster output; SVG export is more limited. Pros:
   no subprocess, no fork cost. Cons: requires linking the PDFium
   dynamic library at runtime — same external-dependency footprint
   as poppler, but newer/less ubiquitous than poppler-utils.

Tasks (in order):

1. Benchmark `pdftocairo` vs `pdfium-render` on a representative
   sample (W.pdf-class R-Graphics, matplotlib/pgfplots vector,
   raster-embedded PDF, multi-page PDF with `--pdf-page`). Record
   wall-clock + output-size + faithfulness vs Perl.
2. Decide: single `pdftocairo` path, single `pdfium-render` path,
   or `pdfium-render` primary with `pdftocairo` fallback. Prefer
   the single-tool option if quality matches.
3. Strip the unused fallbacks from `latexml_post/src/graphics.rs`
   — `convert`/`gs`, `inkscape`, and the `ps2pdf` + `pdftocairo`
   double-shell for EPS — once the primary renderer covers EPS via
   poppler's `pdftops`/`pdftocairo` (or pdfium equivalent).

Driver: 2303.02756 W.pdf (R-Graphics) ran `gs` at 110 s in v22 before
my fix; the same paper now uses `pdftocairo --png` at 1.8 s. The
fast path is already in; the long-term goal is to stop maintaining
the slow paths.

---

## Earlier work (archived)

Round-17 / 18 / 19 narrative + REG-1, REG-2, REG-3, CLUSTER-NBSP
detail moved to `docs/archive/round19_iteration_log.md`. Commit log:
`git log --oneline master..claude-round-19`. Major commits include
`d44f1cb38` (`\relax` sentinel on EOF), `817d91624` (XUntil
`\def`-family re-Invoke), `6ac613b48` (xy.sty preloads amstext),
`a6b4cb5161` (psfig cluster), `342b237199` (ntheorem [standard]),
plus 25+ others.
