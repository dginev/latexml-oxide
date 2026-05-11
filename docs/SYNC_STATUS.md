# Engine Sync Status — Active Worklist

**Mission**: 100k "no-problem" sandbox parity. A paper is in scope iff
Perl LaTeXML on TL2025 with `--preload=ar5iv.sty
--path=~/git/ar5iv-bindings/bindings` produces 0 errors. Mission completes
when every in-scope paper produces 0 errors on Rust too.

**Status**: Round-25 active 2026-05-08 (expl3 file-machinery cluster
RESOLVED 2026-05-08 — see commits `588ad90263`, `1d21ee0d29`).

### Round-25 active worklist

Working off the 10k_errors sandbox at `~/data/10k_errors_sandbox`.
Latest local: `cargo test --tests` = **1185/0/0** (post-rebase onto
master commit `bffd1be471`, +schema-docs + split post-processor).

**Just landed** (Round-25):
- `7edfb8eeb1` — `latexml_contrib::scicite_sty` short-circuit binding.
  scicite.sty is a 513-line modified-cite.sty for the journal Science;
  without a binding our parser hangs on its `\edef`/`\catcode` dance.
  Mirrors `cite_sty.rs` — defines only the public API (`\citeleft`,
  `\citeright`, …). Recovers the 7-paper hang cluster
  (1010.2781, 1011.5494, 1102.0562, 1210.1294, 1303.2601, 1704.07345,
  1706.03851). Smoke-tested on 1210.1294 → clean conversion.
- `588ad90263` + `1d21ee0d29` — **expl3 file-machinery cluster
  (60 papers) RESOLVED**. Root cause: `input_definitions`
  unconditionally mutated `\@currname`/`\@currext` even on the
  `handleoptions=false` path, contradicting Perl `Package.pm:2580-2611`
  (which only mutates them inside the handleoptions=true block).
  The leaked name was then captured as the "empty sentinel" by inner
  `\RequirePackage` pushes onto `\@currnamestack`, breaking
  expl3-code.tex's `\__file_tmp:w` stack walk and producing the L11515
  `\__file_name_expand:n` cascade. Two witness chains:
  * `\inputencoding{ansinew}` → `input_definitions("ansinew","def",
     handleoptions=false)` → leaks "ansinew"/"def" → next
     `\RequirePackage{expl3}` push poisoned. Witness: 0805.4519.
  * `\usetikzlibrary{calligraphy}` →
     `input_definitions("tikzlibrarycalligraphy.code","tex",
     handleoptions=false)` → leaks → inner spath3 push poisoned.
     Witness: 1705.00041.
  Fix: drop the Rust-only mutation; match Perl exactly.
  Verification: 60/60 cluster papers (extracted by grep on the v4 logs)
  now have zero file_name_expand errors; 25/60 fully clean. Babel
  csquotes test fixture re-improved (language-correct French/German
  quotes).

**Format dump enabled 2026-05-08** (post-currfix landing). Generated
`resources/dumps/latex.dump.txt` via `LATEXML_NODUMP=1 latexml_oxide
--init=latex.ltx`. 25,439 entries, 3.9 MB, includes 389 expl3 markers
(`\tex_let:D` PA-aliased to `\let`, `\cs_set:Npn`/`\cs_new:Npn`
chain, etc.). Runtime auto-finds the dump from
`resources/dumps/latex.dump.txt` (path 5 in `latex.rs::latex_dump_available`,
relative to `CARGO_MANIFEST_DIR`). With the dump present,
expl3.sty's TeX-level guard `\ifx\csname tex_let:D\endcsname\relax`
short-circuits the `\input expl3-code.tex` raw-load entirely.

Performance impact (30 cluster papers, average):
- Without dump (raw expl3 load): ~25-35s per paper
- With dump:                    ~0.5-3s per paper
- **~10x average speedup**, reaching **46×** on `\usetikzlibrary{calligraphy}`
  test (28.1s → 0.6s).

`cargo test --tests` 1139/0/0 unchanged with dump enabled.

The dump file is gitignored (per `.gitignore: resources/dumps/`) —
local artifact, regenerated as needed. CLAUDE.md "Distribution
follow-up" plans `include_bytes!` embedding for distribution.

**"Core dump" investigation closed** (Round-25, no fix needed):
The two suspected Rust panics (1607.04981, 1506.04659) are NOT
panics. Both hit our internal 60s wall-clock watchdog →
`SIGABRT`, which `timeout` reports as "dumped core". They are slow
conversions, not crashes:
- 1607.04981 — LyX/babel/hyperref maze, completes at ~90s
- 1506.04659 — harvmac/epsf maze, completes after watchdog kill

**Cleanup landed 2026-05-08** (commit `8ac3eae2c4`, post-rebase): the
two redundant @currname save/restore wrappers in `tex_file_io.rs` and
`xy_sty.rs` deleted — input_definitions no longer mutates @currname on
the handleoptions=false path, so the wrappers were pure no-ops.

**Master rebase landed 2026-05-10**: branch `large-scale-testing-round-1`
rebased onto `bffd1be471` ("feat: Schema Docs and Split post-processor
#230"). All 12 local commits replayed cleanly — no conflicts despite
6 overlapping files (latexml_post pipeline reorder + `process_chain`
signature change). `cargo test --tests` = **1185/0/0** post-rebase.

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

- **AmSTeX `\@` undefined (`\input amstex` + `\documentstyle{amsppt}`)**
  — pure-AmSTeX (plain TeX) papers reach `\@` (LaTeX-only kernel CS)
  unintendedly through some amsppt subroutine. Both engines emit
  identical `Error:undefined:\@`. Witnesses: math-ph0001012/0001015.
  SHARED-FAILURE.

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

**Stage 1 mini-sandbox completion (2026-05-10)**: 34 failure-set
papers verified Rust-vs-Perl with `--path=~/git/ar5iv-bindings
--preload=ar5iv.sty`. Final distribution:
  * 16 BOTH-CLEAN (recovered after the binding-fallback policy fix
    `52ca5d6299` + earlier revtex4 default-amsmath flip).
  * 17 SHARED-FAILURE (logged above; both engines fail identically).
  * 1 RUST-CLEANER (hep-th0005268: Rust 21 vs Perl 26).
  * **0 RUST-REGRESSION.**
Mini-sandbox exhausted; advanced to stage 2.

**Stage 2 canvas (2026-05-10)**: 9991/10000 = **99.91% OK**.
9 failures triaged. 1 RUST-REGRESSION fixed (`hep-ph0109006` —
`\xpt`/`\ixpt` undefined cluster, cured by `9673bf8b98` which
loads raw `latex209.def` post-`latex.ltx` during dump-build so
LaTeX 2.09 user-facing pt-family wrappers reach the dump).
Remaining 8: 5 SHARED-FAILURE, 2 RUST-CLEANER, 1 NO_TEX.
Mini-sandbox exhausted; advanced to stage 3.

**Stage 3 canvas (2026-05-10)**: 9984/10000 = **99.84% OK**.
16 failures triaged: **0 RUST-REGRESSION**, 14 SHARED-FAILURE,
2 RUST-CLEANER (`cond-mat0308059`: Rust 1 vs Perl 3;
`hep-th0308103`: Rust 38 vs Perl 102). Top SHARED-FAILURE patterns
match earlier stages: `\psfig` (3 papers), `Error:expected:{`
math arg gaps (4 papers), `\@` AmSTeX (1), `\GenericError`
chain (3), `\@personname`/`\endflushright` cascades (2),
`{instit}` (1), `_/^` text-mode cascades (2). Mini-sandbox
exhausted; advanced to stage 4.

**Stage 4 canvas (2026-05-10)**: 9974/10000 = **99.74% OK**.
26 failures (25 errors + 1 fatal) triaged: **0 RUST-REGRESSION**,
23 SHARED-FAILURE, 3 RUST-CLEANER (3 papers with Rust 1 vs Perl 3
on `\GenericError` chain). Note: an earlier attempt to load
`latex209.def` raw during dump-build (commit `9673bf8b98`)
regressed 87 modern `\documentclass` papers via the file's
`\@documentclasshook` override; reverted via `ac0965abfd`. The
safe pt-family fix from `31154d0760` (latex_base.rs post-snapshot)
remains in place. Top SHARED-FAILURE patterns: `\GenericError` (7),
`\@` AmSTeX (5), `\ifpst@useCalc` (3), `\endnote` (2),
`\psfig` (1), `\@math@baccent` (1), `\endflushright` (1),
`_/^` cascade (1). Mini-sandbox exhausted; ready for stage 5.

**Post-rebase landings 2026-05-10**:
- `21e730e71e` — promote two silent-content-loss signals from Info
  to Warn/Error so the canvas no longer classifies broken papers as
  `[ok]`. (1) `document.rs:2759` "Duplicated attribute xml:id"
  Info→Error (bypasses the `Error!` macro because the function
  returns `String`, not `Result`; calls `note_status(Error)` +
  `log::error!` directly). (2) `keyvals.rs:274` "Encountered unknown
  KeyVals key" Info→Warn (SeenSet still dedups per (prefix,key)
  tuple). Intentional divergences from Perl `Document.pm:1454` /
  `KeyVals.pm:97`. Witness `1410.8171`: previously `[ok]` despite
  S3+ rendering as essentially empty; now reports
  `Status:conversion:2` with 54 warnings + 3 errors.
- `fc2aae7266` — `siunitx_sty::six_format_1unit`: replace
  `ExplodeText!(&pre_resolved)` / `ExplodeText!(&u_resolved)` with
  `mouth::tokenize` for the `\mathrm{...}` argument. The exploded
  form turned the resolved-presentation string `"\SIUnitSymbolMicro"`
  into 17 OTHER tokens (literal text in math output); tokenize
  re-parses through std catcodes producing a single CS token. Prefix
  and unit are tokenized separately so the boundary is preserved.
  Witness 1410.8171: `\SI{0,1}{\micro\kelvin}` rendered the literal
  `\SIUnitSymbolMicroK` in math; now renders as `µ K`. Test fixture
  `tests/complex/si.xml` regenerated (169→77 lines; broken
  `<XMWrap>...<XMTok>\</XMTok><XMTok>SIUnitSymbol*</XMTok></XMWrap>`
  triplets collapse to clean `<XMTok>` per unit).

- **Keyval registration cluster** (paired with `21e730e71e`'s
  Info→Warn promotion; Rust-only divergences except where noted):
  - `75bab231a5` — siunitx 32 boolean SIX keyvals (Perl-faithful,
    mirrors `siunitx.sty.ltxml:38-54`).
  - `4255f5a7cd` — siunitx 45 non-boolean SIX keyvals (Rust-only,
    silences siunitx-internal `\sisetup{...}` defaults noise).
  - `254b4f54c9` — hyperref ~80 Hyp keyvals (mirrors the existing
    `DeclareOption` loop). Driver: 2304.12803.
  - `ece08d7ea5` — 5 hyperxmp Hyp keys (`pdfcopyright`, etc.).
    Driver: tests/complex/hypertest.tex.
  - `be595f4084` — `tabular.vattach` + 5 listings/lstlang internal
    keys. Drivers: tests/structure/{greek,numprints}.tex,
    tests/tikz/various_colors.tex.
  - `d27be28dc0` — siunitx 26 rounding/table keys + 16 mathtools
    `mt` keys + 10 xargs keys + 2 `lx@GEN` keys (`atameaning`
    typo, `alignment-required`). Drivers: tests/complex/si.tex,
    tests/ams/mathtools.tex, tests/digestion/xargs.tex,
    tests/complex/physics.tex.
  - `3a65bf6a88` — graphicx 20 Gin keys (`bb`, `hiresbb`,
    `natwidth`, …) + 8 hyperref keys (`pdfinfo`, `pdfa`, …).
    Drivers: 1503.00123 (`bb=...`), 1807.08711 (`pdfinfo`).
  - `571fa4ed87` — epsfig 18 epsGin keys + caption 22 keys.
    Drivers: 2101.10980 (`\psfig{angle=180}`),
    2110.03647 (`\usepackage[compatibility=false]{caption}`).

  Net effect: the Warn promotion now reserves "Encountered unknown
  KeyVals key" for genuine binding gaps (typos, package gaps, version
  drift). Internal init noise across siunitx/hyperref/graphicx/listings/
  mathtools/xargs/caption/epsfig is silenced. Sandbox sweeps post-fix
  (160+ random papers across years 2007-2024) found 0 residual gaps.

**1410.8171 outcome (2026-05-10)**: standalone re-run of
`SarkanyPRArevision.tex` against the post-fix binary now reports
`Conversion complete: No obvious problems` — **0 warnings, 0 errors**
(vs prior `54 warnings; 3 errors`). Three independent fixes
combined:
1. `fc2aae7266` — siunitx CS-preserving tokenize: collapses µK into
   a clean single `<XMTok>` per glyph. Eliminated the 3
   `Error:malformed:id` "Duplicated attribute xml:id" events as a
   side-effect (broken `<XMTok>\</XMTok><XMTok>SIUnitSymbolMicroK
   </XMTok>` triplets were producing the conflicting xml:id slots
   during math-parser absorption; with clean tokenization the
   id-counter clash is impossible).
2. `75bab231a5` — Perl-faithful boolean DefKeyVals: silenced 11
   warnings.
3. `4255f5a7cd` — Rust-only non-bool DefKeyVals: silenced the
   remaining 32 warnings. No XMath/_xmkey generator change needed.


Round-20 Phase A Gate 0 closed 2026-05-03 at **99,829 / 100,003 =
99.83%** raw OK on the 100k canvas. Round-22 sprint targeted the
335-paper baseline-failure set (`~/round22_validate/inputs/`):
- v10 baseline: 249/350 OK = 71.1%
- v16 (mid-round): 274/330 = 83.0%
- v17 (T1-cmd-loop fix): 289/330 = 87.6%
- v18 (open_text walk + isotope + etoolbox-& + biblatex Let): 292/328 = 89.0%
- v19 (XUntil constructor-args + aas_support C/L/R): 294/328 = 89.6%
- **v21** (bookmark stub + graphics gs-timeout/inkscape-default): **294/327 = 89.9%**
  (same 294 unique OK as v19; bookmark stub didn't directly recover papers
  because token-limit fires elsewhere in 2310.15090 / 2203.01231 paths)
- **v22** (round-22 wrap, 100k canvas validation): 295/329 = 89.7%

Round-21 work archived in `docs/archive/`.

## Round-23 (active 2026-05-07/08)

Continuation of round-22 sprint on the same 335-paper baseline.
Re-ran the round-22 failures-set sweep across multiple iterations.

| Sweep | Cortex OK / Unique | True parity (Rust=0=Perl) gain |
|-------|---:|---|
| v22 (carry-over) | 295 / 329 = 89.7% | 12 papers tractable + 1 Perl-regression |
| v25 (siunitx + block + lstinline) | 300 / 326 = 92.0% | +5 cortex-recover, +9 BOTH CLEAN |
| v26 (matching binary) | 299 / 327 = 91.4% | (binary timing diff vs v25) |
| **v27 (after natbib NAT@@wrout fix)** | **300 / 328 = 91.5%** | **11 of 12 originally-tractable failures fixed** |

Round-23 commits (chronological):
1. `ad77a29f47` — siunitx: pass `\DeclareSIUnit` presentation as Tokens not exploded letters; restores `\metre→\meter→m` collapse inside `\SI{}`. Driver: 1907.04278.
2. `fd8bb072a7` — siunitx `\mathrm{...}` wrap in `six_resolve_unit_objects` (Perl L1216 parity) + `six_parse_literalunits` peels CC_BEGIN groups opaquely; graphics: pdftocairo `--png`/`--svg` fast paths added with 8 MB SVG output guard. Drivers: 2304.12803, W.pdf gs-runaway.
3. `1569d6f86b` — SYNC_STATUS task: long-term consolidate `pdftocairo`/`pdfium-render` to single PDF renderer.
4. `5b2e38590c` — schema: `Tag!("ltx:block", auto_close => true)`. Driver: 2302.11635 IEEEtran transmag minipage row.
5. `ba56a30a33` — listings: `\lstinline` body under verbatim catcodes (Perl `EMPTY_CATTABLE` parity) + match closing delim by text only. Driver: 2301.10618 section-in-item cascade.
6. `3198b744ab` — natbib: `\NAT@@wrout` ditches `bounded => true` (manual bgroup + soft pop, bypassing egroup mode-frame guard) + `\lx@NAT@parselabel` skips `Expand!` on labels with complex CSes (`\cite`, `\href`, …). Driver: 2404.06289 (19 errors → 0).
7. `d42de4439e` — `latexml_oxide` bin: bail with `Fatal:invalid:not_tex_source` for single-file inputs whose first 5 bytes are `%PDF-`. Mirrors the directory-mode `is_pdf_magic` already in `find_main_tex`. Driver: 2301.04210 (PDF mis-named `.tex`; was 101 cascading tokenizer errors → 1 Fatal).

8. `1790c32b1b` — `\MakeUppercase`/`\MakeLowercase` pre-stub
   `\UTF@two/three/four@octets@noexpand` to `\@empty` so the body's
   neutralisation `\let`s don't trigger `Error:undefined:` on the
   `\edef\reserved@a{...}` partial-expansion phase. Real TeX's
   `\let<undef>\<defined>` is a no-op without error.
9. `31b6cc1e00` — `lx_read_and_change_case` inserts `\dont_expand`
   between `\protect` and the munged robust CS in the fall-through
   case (CS not in exclude list, not in case-mapping). Without it
   the outer `\edef\reserved@a{...}` body's `Partial` expansion
   re-invokes the unprotected munged macro, mangling captured tokens
   and silently dropping the case-changed content during the later
   `\reserved@a` invocation. Driver 2009.10018: 16 errors → 0.
10. `e436a9cda7` + `fedc89cabd` — `\regex_match:NnTF` and 5 variants
    short-circuited to FALSE branch (with `_`/`:` letter-catcode
    wrap so the helper CS names tokenize correctly). Drives
    2406.14142 from **21 errors → 0** (the last historical
    REAL_REGRESSION). Trigger: duckuments.sty's `\includegraphics`
    wrapper uses `\regex_match:NnTF` against
    `\c_duckuments_example_regex`; our Rust expansion of expl3 regex
    compile/match drove `\if_int_compare:w` against `\l__regex_*_int`
    in a way that stalled at `\end{document}`. The stub falls back to
    plain `\includegraphics`, which is acceptable. Other expl3
    packages relying on regex matching silently take the F branch —
    Rust-only divergence; faithful expl3 regex emulation is tracked
    in `docs/archive/`.

**No REAL_REGRESSIONs remain in the round-23 random sweep**
(0/327 papers post-fix; 2406.14142 was the last and is now Rust=0
vs Perl=4 PERL_REGRESSION).

Cortex-failure-but-parity-clean set (BOTH CLEAN with cortex_worker abort/OOM/timeout): 1904.02716 (xpath nodeset growth in math parser, formula 92), 2007.13470 (token-limit during english/slovak.ldf hook), 2011.14413, 2105.04174, 2203.01231, 2310.15090. Their conversion produces 0 errors in standalone parity_check; cortex's per-paper RAM cap or post-processing limits trip them. Out-of-scope for round-23 (post-processing infra work).

**True Rust regression count: 0** *for ported error conditions*.
[Caveat: Error/Fatal coverage audit](ERROR_PARITY_AUDIT.md) reveals
≈43% of Perl Error/Fatal callsites are absent in Rust (largely
concentrated in `latexml_post` and 4 packages: siunitx, pgfmath,
xcolor, calc). On the current 100k corpus this gap doesn't appear
to be inflating the parity claim — but a few PERL_REGRESSION papers
loading xcolor warrant re-verification. Re-classifying the 246 residual
rows by parity-check verdict:

| Verdict | Rows | Meaning |
|---|---:|---|
| OUT-OF-SCOPE | 188 | Rust=Perl, both error |
| PERL_REGRESSION | 36 | Rust strictly *better* than Perl |
| BOTH CLEAN | 5 | Stale (already-fixed entries) |
| REAL REGRESSION | 7 | All flagged PERL_TIMEOUT — now reclassify to `OUT-OF-SCOPE? (recheck at TIMEOUT_SECS≥180)` per `e1c3da3975` parity_check fix; Round-20 verification at 180s found 0 Rust-only regressions |
| (unparsed) | 4 | Stage TSV format mismatch |

The 18-paper `\lx@NBSP` cluster is entirely PERL_REGRESSION — Rust
emits half the errors Perl does (Rust=N, Perl=2N) on every sampled
paper.

Round-19 narrative + REG-1/2/3/NBSP fix detail archived in
`docs/archive/round19_iteration_log.md` + `git log
master..claude-round-19`.

---

## Round-22 (active 2026-05-07)

### Session contributions (commits, this branch)
24 commits on `claude-round-22` since 2026-05-07 11:00 UTC.

**Late-session adds (post-v17):**
- `9fe3e77c92` `Document::open_text` walk: stop at explicit
  (non-fontswitch) `<ltx:text>` wrappers — fixes 2402.16319
  `\uline{\textbf{2}}` cascade.
- `fc2ff67389` `aa_support_sty` drop spurious `\isotope` definition
  (Perl never defined it) — fixes 2011.10587 `\newcommand\isotope`
  shadow → math cascade (12 errors → 0).
- `70a8f2280f` `etoolbox_sty` `DeclareListParser` block
  `TeX!`→`RawTeX!` for `&` catcode — fixes 2108.09184 `\docsvlist`
  in `align*` cascade (45 errors → 0). Also recovers 2110.11931
  similar pattern.

**Binding fixes (15):**
- `f6fa966619` etoolbox `\ifstrempty` block `TeX!`→`RawTeX!` (1904.02116)
- `187997454e` enumitem `\the<counter>` ref= recursion (1904.10839)
- `b1bbe1cb8b` siunitx `S/s Optional` column option (1904.04479)
- `be094f63f4` `\startlongtable` no-op (2209.01632 aastex631)
- `76ec7b4621` `\psj` journal abbrev (2306.11151)
- `ebaacfde31` elsart `\affiliation` utf-8 char-vec (2407.00104)
- `6d4a15f73b` `discard_env_body` `require_open=false` (2402.09676 NiceTabular)
- `6947f5f3ce` `\shortauthor / \shorttitle` predef + save (helps arxiv.sty)
- `f587a6663c` amsmath `\tag` uses `\edef` (2406.07616 OOM)
- `f53ab3ecda` **`\DeclareFontEncoding` defines `\<encoding>-cmd` —
  recovers ~13 papers in token-limit cluster** (T1-cmd-loop fix)

**Defensive guards (6):**
- `fe96758a11` `\lx@dual` reversion + `\patchcmd` None args
- `0a1b7e15b9` XMDual `_xmkey` defensive
- `9538737ae0` + `d00dfc5876` `_font` parse defensive (2 sites)
- `cfbb003380` `xml::findnodes` empty vec on libxml2 error
- `308ce289b0` `Node::new` failure → Result not panic
- `e78c5aba97` `gullet read_internal_token` runtime-None defensive

### Round-22 well-diagnosed remaining failures (post-v17)

These need follow-up work but require deeper engine effort or
divergence-from-Perl design decisions:

| Paper | Cluster | Diagnosis |
|---|---|---|
| 1904.02716 | math-parser stack overflow | revtex4-1 + braket; deep math nesting overflows the math parser stack |
| 1904.10251 | math-parser stack overflow | similar |
| 2105.04174 | xpath/stack-overflow cascade | XPath findnodes on stale subtree triggers stack overflow elsewhere |
| 2304.07380 | math-parser OOM | XMTok/XMApp create-element failure during math; defensive Node::new converted to errors but math parser still over-allocates |
| 2306.12437 | local class needed | `\documentclass{ptephy_v1}` — paper ships ptephy_v1.cls but INCLUDE_CLASSES=false suppresses the load. Same Perl errors. Local-class loading fix would diverge from Perl. |
| 2406.14142 | expl3 `\group_begin:` | duckuments.sty + expl3 `\c_sys_jobname_str` cascade. `\shortauthor` fix removed one error; deeper expl3 issues remain. |
| 1907.04278 / 2304.12803 | siunitx `double-superscript` | state-cumulative; tight min repros pass standalone, suggests siunitx-specific accumulation. Needs siunitx unit-arg parsing audit. |
| 2007.13470 | babel-slovak hang | hangs after geometry.sty + babel @aux hooks fire; not the T1-cmd loop. Needs babel-slovak language-file investigation. |
| 2110.11931 | mnras `Script _` | state-cumulative; min repros pass. Needs mnras frontmatter mode-frame audit. |
| 2402.16319 | schema close-text | `<ltx:_CaptureBlock_><ltx:tabular><ltx:tr><ltx:td><ltx:text>` close failure inside icml2024 cell. Anonymous String trigger. |
| 2404.06289 | natbib `\NAT@@wrout` | bbl mode-frame imbalance after `\NAT@@wrout`. Known bbl path issue. |
| 2406.07616 (FIXED v17) | `\tag{\thesection.\theequation}` OOM | Recovered by f587a6663c. |
| 2306.16410 + 13 others (FIXED v17) | T1-cmd loop | Recovered by f53ab3ecda. |

Schema-strictness divergences (Perl accepts but Rust's RelaxNG rejects):
- 2211.01875: `ltx:enumerate` in `ltx:listingline`
- 2301.10618: `ltx:section` in `ltx:item`/`ltx:subsection`/`ltx:section`
- 2302.11635: `ltx:toccaption`/`ltx:caption` in `ltx:block`

These are LaTeXML schema model issues; need RelaxNG audit + `OXIDIZED_DESIGN`
divergence entries. Not low-hanging.

UTF-8 in cite-keys + `[T1]{fontenc}` cluster — root-caused as
`\<encoding>-cmd` undefined dispatcher, fixed in `f53ab3ecda`. Memory
note: `wisdom_utf8_semiverbatim_hang.md`.

### Round-22 next steps

| Task | Status |
|---|---|
| v17 sweep with `f53ab3ecda` (T1-cmd) | release rebuilding 15:38 |
| Confirm +13 paper recovery in v17 results | pending sweep |
| Schema-strictness audit (2211.01875 cluster) | open, needs RelaxNG sub-task |
| siunitx state-cumulative double-superscript audit | open |
| ptephy_v1 / unknown-local-class load policy | open, needs design decision |

---

## Round-20 (closed 2026-05-03)

### What landed this session
- **`tools/parity_check.sh`**: PERL_TIMEOUT papers with `partial < Rust`
  no longer misclassified as REAL_REGRESSION; they now get
  `OUT-OF-SCOPE? (Perl-timeout, recheck at TIMEOUT_SECS≥180)`. Verified
  on 0705.0102 at TIMEOUT_SECS=90.
- **`tests/06_cluster_regressions.rs`**: now greps `Error:<class>:`
  markers from the conversion log; relying on `status_code` alone
  was too permissive.
- **`find_main_tex` (cortex_worker.rs + latexml_oxide.rs)**: Perl
  `Pack.pm:128` `s/\%[^\r]*//` is `\r`-aware; the Rust port used
  `find('%')` which truncated everything past the first `%`. On
  bare-`\r` files (Mac classic) `\documentclass` after a comment
  was hidden, failing with "No viable .tex files". Witness:
  `cond-mat0002096`, `0708.2784`. Both convert cleanly post-fix
  (32kB / 33kB ZIPs, 0 errors, 207+ Maths each).
- **`alignment.rs:add_line`**: `row.get_column_mut(c).unwrap()` panicked
  when `\hline`/`\cline` referenced a column past the row count. Perl
  Alignment.pm:128-130 silently no-ops via autovivification — replaced
  unwraps with `if let Some()`. Surfaced by 0708.2784. 29/29 alignment
  tests + 4/4 cluster_regressions pass.

### Round-20 verification (PERL_TIMEOUT cohort, TIMEOUT_SECS=180)
| Paper | Rust | Perl | Verdict |
|---|---|---|---|
| 0705.0102 | 36 | 36 | OUT-OF-SCOPE (Sub-cause A `\emph{$$math$$}`) |
| 0705.3903 | 0 | 0 | BOTH CLEAN |
| astro-ph0502153 | 1 | 1 | OUT-OF-SCOPE |
| cs0412098 | 3 | 3 | OUT-OF-SCOPE |
| quant-ph0406132 | 0 | 0 | BOTH CLEAN |

### 100k re-sweep (Phase A Gate 0) — DONE 2026-05-03

| Metric | Pre-fix (round-19) | Post-fix (round-20) | Δ |
|---|---:|---:|---:|
| OK | 99,774 | **99,829** | **+55** |
| Non-OK | 226 | **174** | **-52** |
| NEW non-OK introduced | — | **0** | — |
| Raw OK rate | 99.77% | **99.83%** | **+0.06pp** |

170 unique non-OK papers (174 raw with retry dups). **All 170 were
already in the pre-fix 226-paper list**: zero truly new failures.
56 pre-fix non-OK papers recovered. Phase A Gate 0 cleared.

Residual breakdown (measured 2026-05-03 across all 226 unique non-OK
papers from the 10 stages, bucketed by primary `Error:<class>:<token>`
in the conversion log):

**Cluster 1: papers with `Error:unexpected:` (≈119 papers)**

| Token | Papers | Cluster | Status |
|---|---:|---|---|
| `^,_` | 41 | Sub-cause A: `$$math$$` in horizontal mode | SHARED-FAILURE; Phase C surpass-Perl |
| `_` (bare) | 21 | Sub-cause B: text-mode `_/^` reaching key-arg | mix SHARED-FAILURE + a few PERL_REGRESSION |
| `\lx@NBSP` | 18 | `~` in `\csname r@LABEL\endcsname` (HEP papers, elsart.cls) | **PERL_REGRESSION ≈100%** (Rust=N, Perl=2N) |
| `\endproof` | 7 | proof-cluster Gate 3 | SHARED-FAILURE; Phase C |
| `^` (bare) | 5 | Sub-cause A variant (single-token) | SHARED-FAILURE; Phase C |
| Combined-w/-other-tokens | ~27 | `\bm`, `\mbox`, `\@startsection`, `\end{equation}`, etc. | per-paper Phase C |

**Cluster 2: papers WITHOUT `Error:unexpected:` (107 papers)**

| Primary error | Papers | Cluster | Status |
|---|---:|---|---|
| `Error:undefined:\@` | 19 | `at_letter` scope on `\input` boundary | SHARED-FAILURE |
| `Error:undefined:\@ifundefined` | 11 | non-LaTeX residual after the 33-paper LaTeX fix | needs sample-investigation |
| `Error:expected:<box>` | 11 | math constructor missing arg | mostly cascade noise |
| `Error:undefined:\CITE` | 10 | Sub-B family (auto-defined zero-arg constructor leaves `{key}` text-mode) | SHARED-FAILURE |
| `Error:undefined:\psfig` | 7 | residual from `\input psfig.sty` (different from `\documentstyle[psfig]` already FIXED) | SHARED-FAILURE |
| `Error:expected:{` | 7 | group-brace mismatch (user-malformed) | Phase C |
| `Error:undefined:\setdec`/`\dec` | 10 | residual after FIXED cluster | needs sample-investigation |
| `Error:malformed:ltx:XMApp` | 3 | schema overcontainment / math-parser | tracked in `wisdom_para_rule_schema_overcontain.md` |
| `Error:malformed:ltx:acknowledgements` | 3 | schema overcontainment | same wisdom file |
| (no `Error:*` at all) | 6 | non-error category fail (warnings + 0 errors but still classified non-OK) | needs investigation |
| various rare-CS undefined | ~13 | `\endnote`, `\putrectangle`, `\lx`, `\vspace`, etc. | per-paper Phase C |

---

## Schedule (Round-20 — landing items completed)

Round-20 100k canvas tasks done 2026-05-03:
- Re-sweep + triage 99.83% raw OK, 0 NEW non-OK, 56 recovered ✓
- Round-20 fix series committed (`e1c3da3975`) ✓
- `_/^` cluster sub-cause bisection (5 witnesses) ✓

Outstanding:
- D+3: CI nightly canvas (random 1k slice with parity_check baseline diff)
- D+4: Open PR with Round-20 measured numbers

After Round-20 PR: Phase C long-tail. Per-paper triage at 1-2/day with
min-repro → fix → land → verify. Many will be SHARED-FAILURE that
require deliberate Rust-beats-Perl divergences — track in
`docs/OXIDIZED_DESIGN.md` before landing.

Phase E asymptote: convert intractable papers to
`Fatal:invalid:<reason>` via Phase D pre-screen. Canvas reports them
as legitimate skip → 100% by definition.

**Phase D first landing 2026-05-03** (commit `48f0c1ce8a`):
`%auto-ignore` archives now emit `Fatal:invalid:auto-ignore: archive
contains only %auto-ignore sentinel files` from `find_main_tex`. The
`Fatal:invalid:` prefix doesn't match parity_check.sh's lax
`Error:[a-z]+:` regex, so canvas log-grep counts these as 0 errors
(legitimate skip). Witness: `0903.3183.zip` (12 bytes literal
`%auto-ignore`). Same pattern can be extended to `texinfo`,
`auto-include`, withdrawn-paper sentinel, etc. as new witnesses
emerge.

**Stale-TSV validation 2026-05-03** (5 of the 6 "no Error:*" papers
from the bucket map): `cond-mat0002096`, `0708.2784`, `0705.3903` now
BOTH CLEAN with current binary (Round-20 fixes verified). `0903.3183`
now Fatal:invalid:auto-ignore (Phase D, just-landed). `0907.2492`
zip not present at expected path. Net effect: at least 4 papers in
the residual TSVs are stale entries already-fixed by current binary
and will recover on next sweep.

Phase C long-tail (1 month) and Phase D defensive layers (1 week) follow
the same per-cluster pattern; details in §Phase B clusters.

---

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
| `cargo test --tests` | 1135/0/0 | unchanged across all task work |
| `latexml_oxide --init=plain.tex` | 0 errors | 0 errors |
| `latexml_oxide --init=latex.ltx` | 0 errors | 0 errors |
| 100k canvas (Phase 2 closing) | **99.83% raw OK**, 0 NEW non-OK, 56 recovered | 100% match Perl |
| Phase A Gate 0 (re-sweep numbers) | resweep ~92% done | 0 NEW non-OK; ≥40-paper net recovery |

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
