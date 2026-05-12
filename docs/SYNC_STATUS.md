# Engine Sync Status — Active Worklist

**Mission (closed 2026-05-12)**: 100k "no-problem" sandbox parity on
the 426,555-paper arxmliv corpus. A paper was in scope iff Perl
LaTeXML on TL2025 with `--preload=ar5iv.sty
--path=~/git/ar5iv-bindings/bindings` produces 0 errors. Round-25
stages 1-43 closed at ~99.85% aggregate OK, stage 41 = 100.00%, 30
RUST-REGRESSIONs fixed (~15 cascade-amp/single-paper deferred). See
"Round-25 canvas stages 1-43" below for the archived per-stage record.

**Active focus**: retire hand-stub bindings via raw-load (mhchem
77-error expl3 csname-protocol gap remains — see "Planned" below).

### Round-25 active worklist

`cargo test --tests` = **1185/0/0** (post-rebase onto master commit
`bffd1be471`, +schema-docs + split post-processor).

**Round-25 landings** (compressed):

| Commit | Driver | What |
|---|---|---|
| `488ed74c41` | 2001.07651, 1807.04759 | mn2e_support `\ion` Perl-parity `\text{...}` wrap |
| `7edfb8eeb1` | 7-paper hang | `latexml_contrib::scicite_sty` (Science journal cite stub) |
| `588ad90263`+`1d21ee0d29` | 60-paper expl3 cluster | `input_definitions` `@currname` leak fix |
| `488ed74c41`–`be45566b7e` | session 4 | math-CS protected flags + cleveref×hyperref dispatch + recursion guard + `\genfrac` raw readArg |
| `662571777f`+`92c1a40850`+`6c9ad70d38` | glossaries chain | mfirstuc + datatool-base + chemgreek + substr raw-load shims |
| `81ec5536d9` | 2210.13325 | vtop × gls × p RESOLVED (silent side-effect of glossaries rewrite) |
| `f8e20b648e` | mhchem 92→77 | gullet csname `\let`-to-char substitution (cited tex.web + texbook) |
| `6a7d8fee7d` | `\let\amp=&` halign | tex_tables: implicit-`&`/`\cr` in `\halign` preamble (Knuth-faithful) |
| `43e75591dd`+`c6067ca6f5`+`22bf0619cf` | perf | arena + meaning + char-keyed HashMaps pre-allocated to skip startup growth |
| `228471f5e1` | perf | dump_reader: drop per-line Vec alloc (~800 ms debug / ~30 ms release per conversion) |
| `48fd96ac75` | 2402.01336 (LHCb 1067-author) | graphics: content-hash dedup (Plan::Copy + Plan::Convert key on SipHash(content) instead of source path; 1083-node paper → 17 output files) + direct gs EPS path (1.7× faster than IM convert, matches IM antialiasing flags) |
| `71b3b1d390` | xslt perf | skip XSLT serialize/reparse roundtrip (`Document::dup()`) + cache LazyLock regexes in `extract_svg_fragments` + `finalize_html5` |
| `10e1117709` | kpathsea + PDF box | background-thread kpathsea prewarm (saves ~38 ms wall on 1910.01256 by overlapping with bootstrap); `read_pdf_page_box` byte-level fast-fail (skips Utf8Chunks on matplotlib PDFs); `find_graphics_paths` regex cache |
| `e8b49a3233` | mouth perf | `decode_bytes` fast path: `str::from_utf8` (SIMD-validated) instead of `from_utf8_lossy`; skip `\u{FFFD}` replace when no FFFD present |
| `d8ee016f8e` | graphics + LHCb | worker_cap 8 → 32 (helps tail papers like 17-PDF LHCb dedup); `extract_svg_fragments` picture-fast-fail (skip backtracking regex when no `<picture>` substring) |
| `eb916edff5` | astro-ph0005397 + astro-ph0103041 | `apply_graphicx_transforms` width-only / height-only path: auto-scale unset dimension per graphicx semantics (Fig 11 sfh_burst 230×958 → 230×230); `postscript_is_landscape()` + ps2pdf route for `%%Orientation: Landscape` PS (PGPLOT, NickMorgan.fig2 992×1403 portrait-upside-down → 1404×992 landscape correct); cortex_worker kpathsea prewarm |

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
- **Measured gap (initial 2026-05-12)**: raw-load probe (mhchem stub
  temporarily replaced with `InputDefinitions("mhchem", noltxml=>1)`)
  on a `\ce{H2O}` paper produced **92 errors**.
- **Reduced to 77 errors (2026-05-12, commit `f8e20b648e`)**:
  generalised the gullet csname-reader to substitute any
  `\let`-to-char CS (Stored::Token whose target is LETTER/OTHER/
  SPACE) with its character — was hardcoded for `\lx@NBSP` only.
  Killed the `\exp_stop_f:`-undefined cluster (~15 errors).
- Remaining 77-error residue:
    * `\exp_args:Nc` between `\csname`/`\endcsname` — partial-cs
      accumulation; root cause not yet isolated
    * `\scan_stop:`, `\s__tl`, `\tex_skip:D` between csname/endcsname
      (these are PA-aliased to `\relax`; real TeX errors on these
      inside csname, may be SHARED-FAILURE)
    * `\fi:` appearing outside conditional
    * `<relationaltoken>` expected (numeric comparison gaps)
  Probe restored; the contrib stub remains the load-bearing path.
  Chemgreek shim added so direct `\usepackage{chemgreek}` raw-loads.
- Engine work to retire stub: isolate `\exp_args:Nc` partial-cs
  issue (the partial-cs message shows `\exp_args:Nc` text appended
  literally, hinting at a non-expansion path); fix relational-token
  numeric scanner; verify `\fi:` PA-aliasing is honoured by the
  conditional tracker. Tracked in Round-26 candidates.

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

- **Paul Taylor `diagrams.tex` time-bomb** — papers using
  `\usepackage{diagrams}` with the TL `diagrams.tex` v3.96 ship a
  `\count@=\year\multiply\count@12 \advance\count@\month
  \ifnum\count@>24307 \message{because this one expired in July
  2025!}\expandafter\endinput\fi` time-bomb at L2630-2631 of the
  raw file. As of 2026-05 (`\year*12+\month = 24317 > 24307`) the
  file aborts via `\endinput` before defining `\diagram`/`\rTo`/
  `\dTo`/etc., even when `--path=$HOME/git/ar5iv-bindings/originals`
  exposes the raw file. Perl handles this by shipping a stub that
  comments out `InputDefinitions('diagrams', noltxml=>1)` — the
  raw file would abort anyway. Rust mirrors that stub
  (`latexml_contrib/src/diagrams_tex.rs`): emit a single
  `Error:undefined:{diagram}` per kind, discard the body. Witness:
  1701.07720. SHARED-FAILURE. Re-evaluate when Paul Taylor ships
  v3.97 with a later expiry.

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

## Implicit-character semantics (2026-05-12)

Knuth TeX's "implicit characters" (texbook p.~277) are CSes that
were `\let`-equivalenced to a character token. The implicit form
dispatches by the underlying char's command in most contexts but
not all. Current Rust-port status:

| Primitive | Implicit-character handling | Status |
|---|---|---|
| `\ifcat\X A` (X let to letter) | matches both letters | ✓ working |
| `\if\X X` (X let to char X) | same char comparison | ✓ working |
| `\ifx\X\Y` (both let to same char) | recognises equivalence | ✓ working |
| Math `$\X b$` (X let to `+`) | renders as math operator | ✓ working |
| `\halign` preamble `\amp` (let to `&`) | column separator | ✓ commit `6a7d8fee7d` |
| `\halign` preamble `\rowEnd` (let to `\cr`) | row separator | ✓ commit `6a7d8fee7d` |
| `\halign` body `\rowEnd` (let to `\cr`) | row separator at digest time | ✗ niche gap |
| `\csname` consumption | Knuth: error; we: soft-substitute | divergence (commit `f8e20b648e`, citations in `latexml_core/src/gullet.rs`) |

The `\halign` body-side implicit-`\cr` gap is a low-impact niche
(`\let\rowEnd=\cr` is rare in real papers). Open if witnesses emerge.

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

## Round-25 canvas stages 1-43 (2026-05-10 → 2026-05-12, archived)

Mission closed. Mini-sandbox triage walked the entire 426,555-paper
arxmliv corpus in 44 staged 10k slices. Per-stage OK% range:
**99.56% – 100.00%**, aggregate ~99.85%, stage 41 = **100.00%**.
Cumulative RUST-REGRESSIONs fixed across stages 1-43: **30**;
deferred: **~15** (all single-paper niche or cascade-amplification).
Residue at close: ~110 SHARED-FAILURE (Perl identical:
auto-ignore/`%PDF`/plain-TeX), ~28 RUST-NONDETERMINISTIC transient
OOMs under 16-worker concurrency (converge cleanly standalone).

Notable fix commits landed during stages 1-43 (compressed): `9673bf8b98`
reverted by `ac0965abfd` then `31154d0760` (pt-family); `a85b50ce2b`
(siunitx v1 area/vol); `4c98699468` (`color[usenames]`); `cb84b8781f`
(pstricks `\scalebox`, `\nccircle`); `2bd41220b4` (mhchem auto-dep);
`1a74fc8eb1` (mn2e `{proof}`); `8609c8e793` (`\DeclareSIUnit`
SkipSpaces); `e9b7673bab` (siunitx hep block); `ab043cc826` /
`3883d4d14d` (glossaries 1140→129 lines raw-load); `85cf242dba`
(pstricks raw-load); `087dc31aaf` (`\definecolorseries`); `e22ab01185`
(`\glsdisp`); `7edfb8eeb1` (scicite stub); `588ad90263`+`1d21ee0d29`
(@currname leak); `488ed74c41` (mn2e `\ion`); `a965623dcd` (math-CS
protected flags); `6bb95be594` (cleveref×hyperref); `be45566b7e`
(`\genfrac` raw readArg).

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

## Round-17 → 23 (archived)

All pre-Round-25 sprint narratives live in
`docs/archive/round19_iteration_log.md`. Headline numbers:

- **Round-20** (closed 2026-05-03): 100k canvas at **99.83% raw OK**,
  Phase A Gate 0 cleared. 0 NEW non-OK, 56 recovered.
- **Round-22** (closed 2026-05-07): 335-paper baseline-failure
  sprint, **295 / 329 = 89.7%** unique OK at v22 wrap.
- **Round-23** (closed 2026-05-08): **300 / 328 = 91.5%** unique OK;
  natbib `\NAT@@wrout` (3198b744ab), siunitx `\DeclareSIUnit`,
  pdftocairo png/svg fast paths, listings `\lstinline` verbatim,
  `\MakeUppercase` UTF@N@octets pre-stub, `lx_read_and_change_case`
  `\dont_expand` insertion. 0 REAL_REGRESSIONs at end-state.
- **Round-17/18/19**: see archive. Major commits: `d44f1cb38`
  (`\relax` sentinel on EOF), `817d91624` (XUntil re-Invoke),
  `6ac613b48` (xy.sty preloads amstext), `a6b4cb5161` (psfig
  cluster), `342b237199` (ntheorem [standard]).

## Phase B clusters — residual SHARED-FAILUREs (archived 2026-05-03)

Post-Phase-A-Gate-0 sampling found that every remaining cluster
papers is SHARED-FAILURE with Perl, not a Rust-only regression:

| Cluster | Papers | Verdict |
|---|---:|---|
| `_/^` Sub-A: `$$math$$` in horizontal mode | 78 | SHARED-FAILURE; surpass-Perl candidate (would need `OXIDIZED_DESIGN` entry to fall back to `$..$`) |
| `_/^` Sub-B: `_/^` in `\cite`/`\bibitem` key | ~5-10 | SHARED-FAILURE; surpass-Perl candidate (would switch arg catcodes) |
| `\endproof` outside amsthm | 15 | SHARED-FAILURE |
| `\@` (at_letter scope on `\input`) | 4 | SHARED-FAILURE |
| `\psfig` via `\input psfig.sty` | 6 | SHARED-FAILURE (different from `\documentstyle[psfig]`) |
| `Error:expected:<box>` cascade | 26 | mostly cascade noise from earlier errors |
| `Error:expected:{` brace mismatch | 18 | user-malformed TeX |

**Already-recovered clusters** are pinned as fixtures in
`tests/06_cluster_regressions.rs`: NBSP-in-csname (18 papers),
`\@ifundefined` (33), `\setdec`/`\dec` (12), `\CITE` (11), psfig
via `\documentstyle[epsfig]` (12, `a6b4cb5161`).

The two surpass-Perl candidates above remain open. The CLAUDE.md
guard rules them out of automatic loop work without an explicit
upstream-PR design entry.

---

## PERFORMANCE.md follow-ups (separate track)

PERFORMANCE.md sets the policy for performance work. Active items
ordered by impact:

- **P0 done** — phase-attributed telemetry, telemetry.jsonl.gz, perf_phase_summary.py, perf_compare.py.
- **P1 graphics & output-heavy jobs** — primary rasterizer
  optimization DONE 2026-05-12 (`5244a5a4e2` → `feaf8bcd16`):
  subprocess `mutool draw` is now the first PDF→PNG/SVG attempt,
  ~1.7× faster than pdftocairo on the canvas slow-tail (matplotlib /
  pgfplots scatter PDFs). Graphics phase on 1910.01256 dropped from
  1031 ms to ~480 ms. Still-open: content-identity conversion
  cache + duplicate coalescing across documents. Sentinels:
  0809.3849, 0908.3201, 1003.0368, 0803.4343, 0907.4282.
- **P1 math/large-document jobs** — `LATEXML_PARSE_AUDIT=1` on
  astro-ph0204009, 0911.0884, astro-ph0401354, 0809.5174,
  astro-ph0507615; rank by total parse time + repeated token sequences.
- **P1 failure/control-flow outliers** — re-run 5 timeouts with phase
  telemetry; `0903.3465` is an Xy-pic/token-limit recovery bug.
- **P2 allocation/startup cleanup** — partial landings 2026-05-12:
  arena pre-alloc 32K → 131K (`43e75591dd`), `State::meaning` pre-alloc
  131K (`c6067ca6f5`), char-keyed `catcode`/`mathcode`/etc 512
  (`22bf0619cf`), `dump_reader::parse_and_load` Vec elimination
  (`228471f5e1` — ~800 ms debug / ~20-30 ms release per conversion).
  Remaining open: `*_sym` accessors, `Tokens` conversions, `Stored`
  deep copies, package lookup caching.

### Mini-benchmark: beat 2× pdflatex on 1910.01256 (badge of honor)

**Tied as of 2026-05-12 re-measurement.** On `1910.01256` (CVPR-style
~6-page article, `\usepackage[acronym]{glossaries}`, 110 math formulae,
.bbl):

| Pipeline                            | Real time | User    | RSS    |
|-------------------------------------|-----------|---------|--------|
| `latexml_oxide → HTML` (2026-05-11) | 3.13 s    | 3.88 s  | 225 MB |
| `latexml_oxide` (post 2026-05-12 perf pass + content-hash dedup + gs-direct EPS) | **1.18 s** (median of 10 idle) | 1.25 s | 242 MB |
| `pdflatex × 2` (idle CPU, 2026-05-12) | **1.11 s** (median of 10) | – | 64 MB |
| `pdflatex × 2` (under stress, 28-CPU yes-load, 2026-05-12) | ~2.05 s | – | 64 MB |
| `latexml_oxide` (under same stress, 2026-05-12) | ~2.70 s | – | 242 MB |

Under idle single-process conditions, pdflatex×2 is now ~6% faster.
Under canvas-grade CPU saturation, pdflatex×2 stays ahead (~20% on
this paper). The earlier "MET 1.19s vs 1.21s" baseline was within
measurement noise of pdflatex×2 — the gap that read as a marginal
win has narrowed below the noise floor on the current measurement
machine. The improvements since then (graphics rotation correctness,
content-hash dedup, AA-correct gs-direct EPS) are quality wins, not
perf wins — they don't move 1910.01256, which has 5 PDFs and zero
duplicates.

Reclaiming the lead is open. Candidates (ordered by likely impact):
- in-document graphics scheduling latency: 5 mutool spawns serialize
  inside thread pool — measure overlap vs sequential
- digest+gullet allocator pressure: still ~0.2 s on small papers
- math-parse pruning aggressiveness: 110 formulae, ~16-17 % of wall

How we got there (chronological):
1. `43e75591dd` — arena pre-allocated to 131K (latex.dump capacity).
2. `c6067ca6f5` — `State::meaning` HashMap pre-allocated to 131K.
3. `228471f5e1` — `dump_reader::parse_and_load` Vec elimination.
4. `4a1fabea3e` — `load_value`+`load_meaning` Vec elimination.
5. `fe41a54ce0` — E/R-branch field-split Vec elimination (~80k
   allocations).
6. `feaf8bcd16` — mutool subprocess as first PDF rasterizer
   (graphics phase 1031 ms → ~480 ms).

The RSS gap (242 MB vs 64 MB) is structural — Marpa math grammar
tables + interned states — and out of scope.

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
| `latexml_oxide --init=plain.tex` | 0 errors (dump and `LATEXML_NODUMP=1` paths) | 0 errors |
| `latexml_oxide --init=latex.ltx` | 0 errors (dump and `LATEXML_NODUMP=1` paths) | 0 errors |
| Round-25 cumulative regressions | **31 fixed, ~14 deferred** (most are single-paper niche or cascade-amplification); 420k canvas mission closed 2026-05-12 | drive deferred set to zero |
| Per-conversion wall time (debug build, glossaries+math fixture) | ~0.21 s (was ~1.31 s pre-2026-05-12 perf pass — **6× speedup**) | mini-benchmark target 2.88 s release on 1910.01256 |
| Per-conversion wall time (release build, same fixture) | ~0.17-0.20 s | met |
| 1910.01256 mini-benchmark vs pdflatex×2 | **1.18 s** median (latexml_oxide) vs **1.11 s** (pdflatex×2), idle CPU — tied within measurement noise on canvas, pdflatex marginally ahead on idle | mini-benchmark target 2.88 s — still met (we are at 0.4× pdflatex×2's stretch goal) |

---

## Distribution follow-up

Once TL2025 dumps stay robust through a CI cycle: `include_bytes!`
`{plain,latex}.dump.txt` for TL2022…TL2026 and select at runtime by
`kpsewhich --version`. Currently dumps load from `resources/dumps/`.

---

## Post-processing graphics renderer chain (decided 2026-05-12)

After a full evaluation including in-process Rust crates and CLI
benchmarks (see `latexml_post/src/graphics.rs` comments + commit
history `5244a5a4e2` → `feaf8bcd16`), the rasterizer chain is now
**subprocess-only** with measured-speed ordering:

  PDF → PNG:
    1. `mutool draw`         — MuPDF CLI, ~1.7× faster than pdftocairo
                                on matplotlib/pgfplots scatter PDFs
    2. `pdftocairo --png`    — poppler fallback, 25× faster than gs
    3. `convert` + `gs`      — last-resort, hard-timeout 60 s

  PDF → SVG:
    1. `mutool convert -F svg` — MuPDF CLI, ~4× more gzip-compressible
                                  output than pdftocairo
    2. `pdftocairo --svg`      — poppler fallback
    3. `inkscape`              — last-resort vector, hard-timeout 15 s

**Subprocess only — no library linking.** AGPL/GPL on the underlying
C libraries (MuPDF, poppler) does NOT propagate to latexml_oxide
because we invoke standalone binaries via `exec`, not link the
libraries. Same legal pattern as a non-GPL tool invoking `git`.

**Rust-crate alternatives rejected (2026-05-12)**:
  - `mupdf-rs`     — AGPL-3.0, incompatible with project CC0 license
  - `poppler-rs`   — GPL, same problem
  - `pdfium-render` — Apache-2.0/BSD-3 (license-clean) BUT PDFium isn't
                      thread-safe; serialising the 5-worker graphics
                      phase through a Mutex wipes out the fork-free
                      benefit (measured: 1.33 s vs 1.21 s subprocess
                      pdftocairo on 1910.01256).

Required apt packages: `poppler-utils` (mandatory), `mupdf-tools`
(recommended optional, ~1.7× faster), `imagemagick + ghostscript`
(last-resort), `inkscape` (SVG last-resort).

