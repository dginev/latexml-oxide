# Engine Sync Status — Active Worklist

> **DO NOT downgrade Errors to cheat the task.** If Perl LaTeXML converts a paper
> without a downgrade, the Rust translation must match by improving the core
> engine — never by silencing diagnostics. New downgrades require explicit proof
> Perl emits the same severity on the SAME paper, else they hide a real gap.
> (User directive 2026-05-15.) Always classify with `latexml --verbose`, never
> `--quiet` (which hides Perl's `Error:` lines); cross-check pathological inputs
> with `pdflatex`.

> **This file is the BRIEF ACTIONABLE LIST.** The day-by-day fix log and
> completed-task records are NOT kept here — they live in `git log` and
> `docs/archive/`. **When you close an item, delete it here** (git keeps the
> record). Last compaction: 2026-06-21.

## Current status

- `cargo test --tests`: **1481 / 0 / 0** (on `class-b-xmref`; +13 vs main). The +13
  regression tests: content-corruption guard, comma-list-conditional,
  formulae-distribute, partial-over-partial (earlier), plus this session's
  eqnarray/numcases `\arraycolsep`-macro, floatflt/floatfig pctwidth, DefMath
  textmode-no-mode-warning, feynmp_fmf, arximspdf_imsart,
  omnibus_natbib_autoload_no_reload_loop, and seclev_heading_levels_stable
  (07_xslt_seclev_levels) — see the PERF notes below.
- **PERF (2026-06-27): OmniBus natbib-autoload reload loop — FIXED.** The dominant
  arXiv slow/timeout cluster (~50 `sn-jnl` + Wiley/`sagej`/`wlpeerj`/… papers, all
  unbound classes → OmniBus fallback) hung ~90 s in digest: OmniBus's hand-rolled
  natbib autoload re-loaded natbib on every cite-CS re-emit. Routed through the
  canonical loop-safe `def_autoload` (clear-trigger-globally-before-load + hoist).
  2603.06884: 90 s→fatal ⇒ **0.5 s, 0 errors**. Regression witnesses 1403.6801 +
  2207.14344 both green. Full root-cause + breadth in `docs/ARXIV_PERFORMANCE.md`.
- **PERF (2026-06-27): XSLT `f:seclev-aux` O(n²) — FIXED (output-neutral).** The
  second arXiv perf cluster (14 XSLT-dominated papers, ~133–167 s in XSLT) was an
  O(headings × tree-size) heading-level computation in
  `resources/XSLT/LaTeXML-structure-xhtml.xsl` (whole-tree `//` descendant scans
  per `ltx:title`). Memoized to per-name global `<xsl:variable>`s (O(n)).
  2404.12418: 179 s fatal ⇒ **34.7 s**; XSLT @99k 21.2 s → 5.3 s (below Perl's 8.7 s
  — Perl keeps the O(n²)). Byte-identical output, suite 1480/0. Shared upstream XSLT
  issue (candidate to upstream); see `docs/OXIDIZED_DESIGN.md` #37 +
  `docs/ARXIV_PERFORMANCE.md` Hotspot #2.
- **PERF (2026-06-28): XSLT `head-keywords` index-dedup O(n²) — FIXED (output-neutral).**
  The slowest-100 follow-up batch (#201–300) re-run on HEAD was 81/100 already <5 s
  (natbib fix) but left an XSLT survivor tier; root-caused via `xsltproc --profile`
  to `head-keywords` in `resources/XSLT/LaTeXML-webpage-xhtml.xsl` building
  `<meta name="keywords">` with `//ltx:indexphrase[not(.=preceding::ltx:indexphrase)]`
  — O(indexphrases² × tree). Replaced with a Muenchian `xsl:key` (O(n)).
  2208.07515 95→**33 s** (xslt 71.5→11.8); 1802.06435 (the prior campaign's *deferred*
  large-index witness) 78→**17 s**; 0807.4838 78→**13 s**. Byte-identical output
  (xsltproc full-HTML diff + historical-bundle keywords-meta diff), suite **1488/0**
  + guard `08_xslt_head_keywords.rs`. See the #201–300 4-cluster triage below,
  `OXIDIZED_DESIGN.md` #40, `ARXIV_PERFORMANCE.md` Hotspot #3.
- **PERF/parity (2026-06-28): booktabs `\cmidrule` infinite loop under
  `\let\cline\cmidrule` — FIXED (surpass-Perl).** Triaging the slowest-100 batch
  **#301-400** (parallel 50-worker re-run: 74/100 <5 s, ~24 known Cluster-C
  math-heavy theses, **0 timeouts**) surfaced 2 `Fatal:Timeout:IfLimit` papers
  (2506.23179, 2511.17056, both sn-jnl). Root cause: LaTeXML's booktabs binding
  defines `\cmidrule`→`\cline`, so a document `\let\cline\cmidrule` makes
  `\cmidrule`→`\cline`→`\cmidrule` loop forever. **Shared with Perl** (Perl *hangs*
  90 s+; Rust's 8M-`IfLimit` guard fatals at ~12 s — already better). Fixed by
  routing `\cmidrule` through a private `\ltx@saved@cline` captured at booktabs-load
  (`booktabs_sty.rs`): 2506.23179 →**3 s/0 err**, 2511.17056 →**1 s/0 err**.
  Output-neutral for ordinary `\cmidrule`; guard
  `06_cluster_regressions.rs::cluster_cmidrule_cline_let`; `KNOWN_PERL_ERRORS.md` #39.
- **PERF/parity (2026-06-28): OmniBus bbl-side-load natbib loop (2209.11799) — FIXED.**
  The lone HEAD timeout from the #201-300 batch (sn-jnl, 200 s `Fatal:Timeout:TokenLimit`):
  an unbound class's `.bbl` `\bibitem[\protect\citeauthoryear…]` side-loads natbib INSIDE
  the `thebibliography` group, so natbib's `\citep` is popped on group exit and reverts to
  its `def_autoload` trigger, whose already-loaded re-emit then loops. Fixed by hoisting the
  side-loaded defs to global in `\lx@late@usepackage` (`omnibus_cls.rs`) — localized to
  OmniBus, NOT the regression-trap-heavy shared `def_autoload` path. 2209.11799 →**1 s/0 err**;
  brings Rust to parity with Perl. Witnesses clean (2310.13684/1403.6801/2207.14344); guard
  `cluster_omnibus_natbib_bbl_sideload`. See the #201-300 Cluster D triage below.
- **Broad regression + health sweep (2026-06-27):** ~140 diverse random corpus
  papers (two samples of 40 + 100, NOT the perf testbed) on the current binary →
  **0 crashes, 0 fatals, 0 hangs** across all conversions; unbound-class (OmniBus
  natbib path) papers — aastex/revtex4/llncs/IEEEtran/elsarticle — all clean + fast.
  The single highest-error paper (1908.08787, 35 errors) is **Rust-BETTER than Perl**
  (same-host: Perl 101 errors + `too_many_errors` FATAL abort vs Rust 35 errors,
  completes) — shared root (`tabu.sty`/`arxiv.sty` missing in both, `\keywords`); Rust
  degrades gracefully. Undefined-CS discovery surfaced only package-specific gaps
  (svjour/babel/marvosym), **no engine long-tail-CS witnesses**. Confirms the
  session's broad changes (omnibus autoload, XSLT seclev, float-schema) introduce no
  regressions and there is no hidden fixable cluster in the sample.
- **Same-host parity sweep (2026-06-27):** 30 OLD papers (pre-expl3 YYMM dirs, where
  Perl 0.8.8 completes fast), Rust vs same-host Perl. Of the 8 where BOTH completed:
  **8/8 perfect parity (`rust=0 errors = perl=0`)** — zero Rust-worse cases. (1 Perl
  timeout; 21 no-top-level-main, a subdir sampling artifact.) Re-confirms the standing
  lesson that "Rust worse" deltas are rare/parity; on the Perl-comparable subset Rust
  matches Perl exactly.
- `cargo clippy --workspace --all-targets -- -D warnings`: **clean**; `cargo fmt --check`: clean.
- `--init=plain.tex` / `--init=latex.ltx`: **0 errors** (with dump and `LATEXML_NODUMP=1`).
- Distribution build (`maxperf`): ~45 MB; beats 2× pdflatex on the mini-benchmark.

### Landed this session (2026-06-25, on `post-processing-signal-fidelity`)

**Signal-fidelity pass — ~200.7k spurious `warning` messages eliminated from the
10k sandbox, all faithful to Perl, ZERO output change.** Triaged the dominant
post-processing/digestion warning clusters in the cortex 10k run; each was a
Rust-only divergence where Perl is silent (verified against the Perl source per
fix):

- **`expected:id` parse-time transient (128.9k msgs / 1142 tasks)** — the
  math-parser `realize_xmnode` (`parser.rs`) warned "Cannot find a node with
  xml:id" on a LIVE-`lookup_id` miss mid-reinstall (a Rust/ASF artifact Perl's
  `MathParser::realizeXMNode` lacks). Empirically benign: on the heaviest witness
  `0704.2400`, 85/98 warned ids are present in the output and the other 13 leave
  **0 dangling `<XMRef>`** (0 dangling of 2597 idrefs); the whole 10k has **0
  `error:expected:id`**. Made silent; genuine danglers still caught by the
  faithful post-Error (`latexml_post`, Perl `Post.pm:1444/1456`). Output
  byte-identical. See `EXPECTED_ID_XMREF_DESIGN_2026-06-08.md` (2026-06-25 banner).
- **`expected:register \tabcolsep`/`\arraycolsep` (43.8k msgs)** — `\lx@text@intercol`
  / `\lx@math@intercol` used the warning `lookup_register`; Perl
  (`TeX_Tables.pool.ltxml:639/646`) uses a silent `isRegister ? valueOf :
  Dimension(0)` inline guard (a document may `\renewcommand` the length register
  into a macro). Added `state::lookup_register_quiet` (no warn) and used it.
- **`expected:register \fam` (27.1k msgs)** — `decode_math_char` (`mathchar.rs`)
  read `\fam` via warning `lookup_register`; Perl `decodeMathChar`
  (`Package.pm:2928`) reads `lookupValue('fontfamily')` DIRECTLY. Switched to a
  direct `fontfamily` read (the `\fam` register's own getter already does this) —
  no warn, and correct even when `\fam` is shadowed (matches Perl). Normal
  (non-shadowed) `\fam` is unaffected (suite unchanged).
- **`expected:id` createXMRefs (900 msgs)** — `base_xmath.rs` XMDual
  `after_close_late` warned "Unresolved _xmkey"; Perl (`Base_XMath.pool.ltxml:306-308`)
  silently does `setAttribute(idref => undef)`. Removed the Rust-only warning.

- **`expected:register \arraycolsep` eqnarray (354 msgs / 2 tasks) — ✅ FIXED
  2026-06-27 (`db1b879e03`, on `class-b-xmref`).** eqnarray read `\arraycolsep`
  via the warning `lookup_register`; when a document `\def`s `\arraycolsep` into a
  plain macro, Perl's `LookupDimension` (`Package.pm` L1371-1383) reads the macro
  body AS A DIMENSION instead of warning. Ported it as
  `state::lookup_dimension_cs` (register→valueOf; macro→`readingFromMouth`+
  `readDimension`; undefined→warn-unless-noerror + `Dimension(0)`) — distinct from
  the value-casting `lookup_dimension`. Same-host verified: Perl 0.8.8 silent on
  the repro, Rust was 1×. **Bonus parity from extending the same helper to its
  sibling sites:** the `\jot`→rowsep emission gap — Perl emits `rowsep` on
  alignment containers when `\jot`≠default, Rust emitted none (eqnarray, align,
  gather) or wrote it to the wrong channel (`ams_alignment_bindings` → alignment
  `properties`, which never reach openContainer). Fixed across eqnarray +
  `ams_alignment_bindings` (→`xml_attributes`) + `ams_rearrangeable_bindings`
  (added). `mathtools.xml` regenerated: +8 `rowsep`, now byte-for-byte Perl on all
  9 rowsep sites. Suite 1472/0.
  - **Follow-up: `cases` `numcases`/`subnumcases` (`cases_sty.rs`) — same fix,
    2026-06-27.** A full audit of every Perl `LookupDimension` caller vs its Rust
    site found `numcases` was the ONE remaining `lookup_register` (warning) site
    for a `LookupDimension` CS (`\arraycolsep`, cases.sty.ltxml L82) — same class
    as eqnarray. Switched to `lookup_dimension_cs` + added its `\jot`→rowsep block
    (L83-85). Same-host verified (Perl silent; `\jot=8pt`→`rowsep="8.0pt"`
    identical). The other `LookupDimension` sites are already faithful: the
    `@@tabular` `\tabcolsep`→colsep uses the silent `lookup_dimension` (no warn);
    the `strut`←`\baselineskip` sites (`latex_constructs.rs:394`, `tex_tables.rs:771`)
    are Perl `LookupRegister` (parity); list-spacing/List/Stomach baseline/hsize
    don't warn. Regression tests `cluster_{eqnarray,numcases}_arraycolsep_macro_no_register_warning`.
  - **Follow-up: `floatflt`/`floatfig` `floatingfigure` width/float — 3 bugs fixed,
    2026-06-27.** Mining the lone non-parity `expected:register` CS (`\textwidth`,
    3 papers) uncovered a deeper break: the bindings computed `float`/`pctwidth`
    from the env args in `after_digest`, but env args live on the BEGIN whatsit
    (reachable in `after_digest_begin`, cf. minipage `get_arg(4)`) — in
    `after_digest` every `get_arg` is `None`. So (a) `width` was ALWAYS `"0%"` (the
    `{Dimension}` arg read as 0; affected every floatflt/floatfig use, not just the
    3 warning papers), (b) the optional `[l]`/`[r]` float direction was ignored
    (always fell back to `floatfltpos`), and (c) `\textwidth` was read via the
    warning `lookup_register` instead of Perl's silent `LookupValue` (`toPercent`,
    floatflt.sty.ltxml L57). Fix: move float/pctwidth to `after_digest_begin`; read
    the arg via `value_of()` and `\textwidth` via `lookup_dimension`
    (=`LookupValue`). Same-host verified vs Perl 0.8.8: `floatingfigure{3cm}`→
    `width="24%"` (was `0%`), `floatfig{4cm}`→`width="32%"`, `[l]`→`float="left"`,
    `\def\textwidth` macro → no warning. Regression tests
    `cluster_float{flt,fig}_pctwidth`. **Bug-class audit (2026-06-27):** a precise
    codebase scan for `DefEnvironment!` blocks reading `get_arg` in `after_digest`
    (where env args are `None` — they live on the BEGIN whatsit) found **zero**
    remaining after the fix — floatflt/floatfig were the only two affected envs.
    (DefConstructor `after_digest` is fine: single whatsit, args present.)

- **`unexpected:mode` "should only appear in math mode" — broad Rust-only
  over-emission FIXED 2026-06-27.** Rust's `transfer_common_constructor_options`
  (`dialect.rs`) added the `requireMath` beforeDigest UNCONDITIONALLY for every
  `DefMath`, so any plain math symbol (e.g. `\rightarrowfill`, a DefMath ARROW)
  used in TEXT mode warned — Perl (`Package.pm:1304`) adds it ONLY for
  `requireMath => 1` bindings (`$options{requireMath} ? (sub {...}) : ()`); plain
  DefMath auto-enters math, no warning. Gated the Rust closure on
  `options.require_math` (matching the already-correct sibling paths at
  `dialect.rs:368/995/1118`). `requireMath!` is warning-only (no output effect), so
  zero XML change. Same-host verified: `\rightarrowfill` text → Rust 0 = Perl 0;
  explicit `\bm` (requireMath) text → Rust 1 = Perl 1 (preserved); witness
  `0802.3360` Rust 3→0. Cortex cluster: `unexpected:mode` 767 msgs / 28 papers.
  Regression test `cluster_defmath_textmode_no_mode_warning`. **Deep re-validation
  2026-06-27** (both directions, all Rust=Perl): 15 plain DefMath in text mode
  (`\alpha`/`\infty`/`\sum`/`\int`/`\rightarrow`/`\to`/`\leq`/`\partial`/`\nabla`/
  `\otimes`/`\forall`/`\Re`/`\aleph`/`\hbar`/`\ell`) → 0 (no over-emission), and the
  `requireMath=>1` families (`\bm`/`\pmb`/`\boldsymbol`/`\mathbb`/`\mathfrak`) → 1
  (no under-emission, genuine warnings preserved).
- **`feynmp` package unbound → `error:expected:$` cascade FIXED 2026-06-27.** The
  `error/expected/$` cluster (116 msgs / 9 papers) was dominated by one feynmp
  paper (`1003.1620`, 28 of them). feynmp (the MetaPost/PDF variant of feynmf,
  IDENTICAL user macros) had a Rust binding for **feynmf** but NOT feynmp, so
  `{fmfgraph*}`/`\fmf`/`\fmfleft`/… were undefined and `\fmf{...label=$$}`
  digested the empty `$$` → 28 `expected:$` "Missing $ closing display math"
  errors. (The general `{$$}` display-math case is Rust at-or-better: minimal
  `before{$$}after` is Rust 1 / Perl 5 — ruled out as a Rust bug.) Fix: a feynmp
  binding mirroring feynmf, sharing an extracted `feynmf_diagram_stubs()` helper
  (extended to stub `\fmfleft`/`\fmfright`/`\fmftop`/`\fmfbottom`/`\fmfsurround`/
  `\fmfdot`/`\fmfblob`/`\fmffreeze`/`\fmfcmd`/`\fmfpen` as arg-absorbing no-ops).
  Same-host: `1003.1620` had **28 Rust-only** `expected:$` errors (Perl 0 of
  these); after the fix Rust has **0 total** errors on the paper vs **Perl 17**
  (Perl has no feynmp binding either — no `feynmp.sty.ltxml`, and `feynmp.sty`
  absent on this host — so Perl also struggles, just with different residual
  errors; the `$$` cascade was specifically Rust-only). Rust now SURPASSES Perl
  here. Genuine text-mode `_` still flagged (minimal `a_b` Rust 1 = Perl 1).
  Regression test `cluster_feynmp_fmf`. Suite 1478/0.

NON-fixes (confirmed PARITY — Perl warns too, left as-is): `expected:<number>`
"Missing number (Dimension)" (3148 msgs, ~half from one paper `1408.6720`'s
`\dimen@`/`\R@`/`\A@`/`\B@`) is the faithful TeX dimension-recovery warning Perl
also emits (Gullet.pm:972); same-host confirmed Rust 1384 ≈ Perl 1392 on
`1408.6720`. Via `LookupRegister`:
`\tikz@dashphase`/`\cmdGR@*` (pgfmath `pgfmath_register`→`LookupRegister`), `\c@*`
counters (`CounterValue`→`LookupRegister`), tabular/array `strut`←`\baselineskip`
(`LookupRegister`). Also parity in `warning/unexpected`: `\end{document}` "open
groups…" (271 papers, latex_constructs.pool:379), `annotation` "Orphaned
frontmatter annotation" (130 papers, Base_Utility.pool:907; same-host 0705.4287
Rust 2 = Perl 2) — faithful ports both engines emit. **The whole `warning`
severity is now mined: the only Rust-only over-emissions were `expected:register`
(eqnarray/numcases/floatflt) and `unexpected:mode` (DefMath requireMath), all
fixed; the rest are parity / env / the deferred `expected:id` math-fork.**

**`error` severity re-mined 2026-06-27 (stale data, but parity/static classes hold):**
- `undefined` (890): big clusters are the **imsart/arximspdf** journal class (aop/aos:
  `\bauthor`/`\btitle`/`{barticle}`/`\operatorname`/`\Name`/`\REVIEW`/`\pagerange`/… ,
  16+ papers) and `{diagram}` — **both engines lacked the class**. The
  **arximspdf/arxstspdf half is now ✅ FIXED** (the `arximspdf_cls.rs` binding —
  see "Beyond-parity coverage" below — Rust now surpasses Perl on the 16-paper
  cluster). `{diagram}` remains both-undefined (parity). `\citen` is parity
  (works with `cite` loaded; undefined without = Perl undefined too).
- `malformed` (151): largest is `ltx:XMApp` (28, content-MathML → deferred math-fork,
  don't pick piecemeal); the rest (`ltx:section`/`caption`/`bibitem`/…) are
  document-structure parity.
- `expected` (50): `$` feynmp FIXED (above); `<number>` parity; `{`/`}`/`\fi` are TeX
  syntax errors on malformed input (parity). The ONE static-gap find was feynmp.
- `fatal` (51) re-cross-joined vs Perl: **ZERO Rust-only fatals.** All 26
  `TooManyErrors`/`MaxLimit(100)` papers are Perl-fatal (24, parity) or Perl-error
  (2: `math0701308` = known input-brace cascade [[input-brace-filename-box-cascade-2026-06-25]];
  `1607.08317` = parity, same-host Rust 102 = Perl 102 identical errors — the
  fatal-vs-error status is just Rust's 100-error cap escalating where Perl's
  effective cap is higher, a classification artifact, not a content divergence).
  `Timeout` (17) was prior-triaged (16 Perl-fatal + the FIXED 1707.02464).
- `info` (550k msgs) checked too: dominated by Rust-infrastructure
  (`loaded_file`/`dump_reader`/`cleanup`/`cortex` — informational, not divergences)
  and parity ports — the biggest non-infra cluster `ignored:special` (34.8k msgs,
  "Unrecognized TeX Special") is a faithful port of Perl's `Info('ignored',
  'special', …)` (`TeX_FileIO.pool.ltxml:193`), parity. No Rust-only info
  over-emission.
**Conclusion: ALL cortex severities (info/warning/error/fatal) re-mined this session
+ the package-binding audit + a (corrected) structural skeleton diff done —
cortex-mineable parity wins are exhausted. New Rust-only correctness divergences
need a FRESH rerun (dynamic); the big remaining clusters (imsart, content-MathML,
figure-panel box-metric) are deferred #2-track / math-fork / box-session.**

Suite 1468/0→1478/0, clippy clean, fmt clean.

**10k-sandbox rerun validation** (maxperf-cortex, 72-worker fleet, vs the PR#269
snapshot, true per-task transition matrix from `historical_tasks`):
- **no_problem 6219 → 6982 (+763)**, warning 2446 → 1683 (−763) — the +763 are
  papers that were `warning` SOLELY from the removed spurious diagnostics, now
  correctly clean (matching Perl's clean output).
- **ZERO clean→hard regressions**: 0 no_problem→{warning,error,fatal}, 0
  warning→error. Only transitions: `warning→no_problem` 763, plus `error↔fatal`
  ±1 boundary noise (2 papers `never_completed_with_retries` = cortex
  timeout/retry infra, unrelated to the warning-suppression code).
- **Total warning messages 262,986 → 62,106 (−200,880, −76%)**; `expected:id`
  130,814 → 1,011 (only the faithful "Missing idref" Warn remains); `expected:register`
  74,790 → 3,865 (only the parity pgfmath/counter ones remain). error 1175→1174,
  fatal 65→66 (within run-to-run variance).

**Error-category triage (2026-06-25, same fresh rerun).** After the warning
de-noising, re-mined the ERROR/FATAL cross-join (Rust error/fatal vs Perl
no_problem/warning) for genuine Rust-only regressions. **Mined out** — the strict
filter (Rust error/fatal AND Perl `no_problem`, excluding cyrillic phantoms)
returns a single missing-macro singleton (`\textcjheb`). Specifics:
- `unexpected:<char>` inputenc (20 papers) + babel (13) = **re-confirmed env-artifact
  phantoms** (host lacks the `cyrillic` collection; same-host Perl fails identically).
- `Attempt to end mode` box-recovery (73 papers) = **71/73 Perl-parity**.
- `malformed:ltx:XMTok` (3 papers) = parity/Rust-better (Perl error/fatal).
- The only GENUINE Rust-only finding is the **`{\input file}` box cascade**
  (math0701308: same-host Perl 0, Rust 90; `TeXFileName` consumes the `}` —
  shared Perl parity — then Rust's stricter box recovery cascades into text-mode
  `_`/`^` errors). **Low-breadth (3 papers); deferred** — the faithful fix is a
  deep stomach-recovery change, the easy fix (`TeXFileName` stop-at-`}`) diverges
  from Perl-LaTeXML and has broad blast radius. Characterized for a dedicated
  effort. Reconfirms: the cortex `Perl=clean` baseline is unreliable (env
  artifacts) — verify every cross-service delta with same-host Perl.

### Landed earlier (2026-06-22, on `further-stability-coverage`, pushed)

Two genuine Rust-only bugs fixed + the full p/m/b table-column parity arc:
- **Cluster G hang** `1707.02464` — `\hsize`-aware vbox paragraph wrapping (faithful
  Perl `readBoxContents`); the `\narrow` `\hsize`-shrink loop now terminates
  (hang → ~4.8s, 10 errors = Perl). `7545e07fd6`. See `STABILITY_WITNESSES.md`
  Cluster G (FIXED).
- **p{} block-content** `1510.07685` — `\begin{itemize}` in a `p{}` cell (3→0
  errors); global p{} → Perl `\lx@tabular@p` VBox form (1610.00974 step-3).
  `f65b80c1c2`.
- **array.sty m{}/b{} → `\lx@tabular@p`** (`eb978df5a9`) and **p/m/b `<td>`
  `align="left"`** (`1867f17da9`) → **cluster-B FULLY RESOLVED**; table fixtures
  near-/exact-to-Perl (array_newline_math Perl-exact); rotfloat2 sidewaystable
  innerheight 69.1→98.6 vs Perl 98.5.
- Validated regression-free: 12 table-stressed papers + a fresh 24-paper same-host
  sweep (0 Rust>Perl; Rust at-or-better everywhere) + class-level cross-join.
- (Earlier this session: tasks 5 & 6 below — post-processing log parity + graphics
  never-ship-raw-.eps — also landed.)

## arXiv slowest-100 batch #201–300 — 4-cluster triage (2026-06-28, on `followups-2026-06-27`)

The follow-up perf batch (ranks #201–300; historically **173.2–175.5 s**, all pinned
near the 180 s cortex watchdog). Re-run on HEAD (current binary, *after* the 2026-06-27
natbib + seclev fixes): **81/100 already <5 s**, **1 still timing out**, ~18 XSLT/math
survivors at 13–95 s. Triaged into 4 clusters (the last 4 investigated in parallel);
**one new shared O(n²) fix landed, one genuine Rust-only timeout root-caused.**
Methodology: in-zip `telemetry.json` phase split (`scratchpad/triage_slow.py`) → re-run
on HEAD (`rerun.sh`, `--preload=ar5iv.sty`) → for survivors, `xsltproc --profile` +
gdb worker-thread sampling + (Cluster D) Perl-parity check. Full analysis in
`docs/ARXIV_PERFORMANCE.md` (Hotspot #3).

### Cluster A — OmniBus/natbib digest (≈77 papers) — ✅ CLEARED
Unbound journal classes → OmniBus → natbib (sn-jnl ×52, Wiley/sagej/ws-procs/lmcs/oup/
ecai/…). Already fixed by the loop-safe `def_autoload` (2026-06-27). Verified on HEAD:
every class now **0.3–0.9 s**, natbib loads exactly once, citations render (196–321
ltx_cite/ref), zero undefined-`\citep`. **No action.** Only OmniBus paper still slow is
2209.11799 → Cluster D (a *different* loop). Guard `omnibus_natbib_autoload_no_reload_loop`.

### Cluster B — index `head-keywords` XSLT O(n²) — ✅ FIXED this session
`resources/XSLT/LaTeXML-webpage-xhtml.xsl` `head-keywords` built `<meta name="keywords">`
via `//ltx:indexphrase[not(.=preceding::ltx:indexphrase)]` — distinct-by-value over the
`preceding::` axis = **O(indexphrases² × tree)**. `xsltproc --profile` pinned it (145 s of
a ~150 s transform on 2208.07515, 1 call). Replaced with a Muenchian hashed `xsl:key`
(O(n), identical first-occurrence + `xsl:sort` ⇒ byte-identical output). Cluster-wide:
2208.07515 95→**33 s** (xslt 71.5→11.8); 1802.06435 78→**17 s**; 0807.4838 78→**13 s**;
2403.19732 68→29 s; math0206203 50→30 s; 1807.02129 50→27 s. Output-neutral (xsltproc
full-HTML diff + historical-bundle keywords-meta diff, byte-identical), suite **1488/0** +
guard `08_xslt_head_keywords.rs`. **Supersedes** the prior campaign's *deferred* large-index
witness (1802.06435 — the real root was head-keywords, not the index-render templates).
Surpass-Perl; candidate to upstream. See `OXIDIZED_DESIGN.md` #40.

### Cluster C — math-heavy large-doc residual (6 papers) — ⏸ DEFERRED (P1 large-doc lever)
0707.3572 (20894 formulae), 2310.07949, 2106.02143, 2406.00467, 1708.01795, 1912.06823 —
~33–50 s, **no significant index** (head-keywords didn't help). Cost is **distributed**
across build + math_parse + xslt + mathml_pres, scaling with formula volume; **O(n²) ruled
out** by element-scaling on 0707.3572 (xslt ms/formula 0.43→0.83 across 25→100 % ⇒ ~**n^1.45
mild superlinear**, not the ~4× of O(n²)). gdb localizes the mild superlinearity to diffuse
per-element descendant-axis `xsl:if`/`xsl:choose` in the math/structural templates
(`xsltCallTemplate→xsltIf→xmlXPathNextDescendant`), worse on libxml2 2.16. No single hoistable
template (unlike seclev/head-keywords). All complete under the 180 s budget (max ~50 s).
**Second-XSLT-win hunt (deep dive, 2026-06-28): NEGATIVE.** Static enumeration of the
per-element high-frequency templates (`add_id` common.xsl:469, `add_classes`:511, `add_style`:572,
`base-classes`/`base-styling`/`add_RDFa`) found **no `//`/`descendant::`/`generate-id`/`count()`/
`key()`** — genuinely O(1)/element. The only per-element descendant-axis predicates are in
equation handling (`LaTeXML-block-xhtml.xsl:279/287/549`, `not(descendant::ltx:equation[ltx:tags])`
etc.), which scan an **equationgroup's own subtree, not the whole tree** — group-size-bounded, the
source of the n^1.45, NOT whole-tree O(n²). Memoizing them saves <~15 % of the ~16–22 s xslt only on
align-heavy docs, for real output-divergence risk ⇒ not worth it. (The `xsltproc --profile`
`maketitle` 66 s #1 is an inclusive-time + core.xml-XMath artifact — core.xml carries the giant
XMath trees the real compact-MathML input lacks — not a real leaf hotspot.)
**Digest/math-parse half (deep dive, 2026-06-28): also inherent O(formulae), no fixable hotspot.**
On the math-heavy papers build (~11 s) + math_parse (~12 s) are large but **not** a re-parse blowup:
`math_parse_count/formulae` ≈ **1.2** (0707.3572: 20894 formulae → 25086 parses; buckets
[13954,4004,1768,710,132,21,0,0,0] decay cleanly — Marpa healthy, no ambiguity blowup), and
build+math_parse µs/formula is **FLAT** across a 0707.3572 truncation (916.8 → 923.2 → 1085.5 at
25/50/100 %, +18 % at 4× = cache/working-set, not quadratic) ⇒ **O(n), ~1 ms/formula**. The only
lever is shrinking that per-formula constant (math-recognizer / document-builder micro-opt) = the
deep **P1 large-doc track**. **Action:** defer to P1; keep 0707.3572 as a STABILITY_WITNESS.

**Isolated outlier (not Cluster C): 2112.14457 — DIGEST-bound (7.1 s of 16.5 s), not math/xslt.**
`lipics-v2021` + `tikz`/`pgfplots`/`pgfplotstable`/`algorithm2e` (206 package-load lines); the
digest cost points to **pgfplots picture digestion** (known-heavy). Not pinned to a frame (stripped
release binary; needs a symbols build). **Low priority** — 16.5 s, well under the 180 s budget; no
shared breadth. Noted, not actioned.

### Cluster D — 2209.11799 token-limit recursion — ✅ FIXED 2026-06-28 (Rust-only, parity)
The lone HEAD timeout (sn-jnl, 200 s, `Fatal:Timeout:TokenLimit`). NOT the natbib reload —
a **follow-on of the `def_autoload` fix**. When natbib is side-loaded via a
non-autoload path (OmniBus's redefined `\bibitem` sees `[\protect\citeauthoryear` in the
`.bbl` → `\lx@late@usepackage{natbib}`, `omnibus_cls.rs:80`), the `\citep`/`\citet`
`def_autoload` triggers are **never cleared**; the "already-loaded → re-emit `cs_for_closure`
without clearing" branch (`latexml_engine/src/tex.rs:60-62`, added for the `\let`-alias case
2310.13684) re-fires the same closure → infinite token recursion (gdb: tight `__memcpy` +
repeating return-address cycle). **Perl completes** (repro 1.0 s, full paper 11.7 s) ⇒
genuine Rust-only. Minimal repro: `\documentclass{sn-jnl}` + `\begin{thebibliography}` with one
`\bibitem[\protect\citeauthoryear{Foo et~al.}{2020}]{a}` + a top-level `\citep{a}`
(saved at `scratchpad/cluster_D/repro.tex`). gdb on a debug build confirms a gullet
token-expansion spin (`read_x_token`/`read_internal_token` under `digest_next_body`) —
the `\citep` re-emit re-firing the autoload closure.
**Likely deeper mechanism (refined 2026-06-28):** natbib's real `\citep` is installed at
the `\bibitem`/bbl group frame during the side-load and **popped on group exit**, reverting
`\citep` to the global autoload trigger; the body `\citep{a}` then re-fires it. The
non-early-return path already guards this via `hoist_top_frame_meaning_delta`, but the
side-load (`\lx@late@usepackage{natbib}`) bypasses that hoist. (`require_package` is
**idempotent** — skips when `<pkg>.sty_loaded` — so a naive "clear + re-`require_package`"
in the shared early-return branch would NOT reinstall `\citep`; the correct fix hoists the
already-installed defs to global.)
**Fix (landed 2026-06-28):** `\lx@late@usepackage` (`omnibus_cls.rs`) now snapshots the
calling frame's meanings, loads, then `hoist_top_frame_meaning_delta` — exactly mirroring
`def_autoload` — so the side-loaded natbib's `\citep`/`\citet` are hoisted to GLOBAL and
survive `\end{thebibliography}` (matching a top-level `\usepackage{natbib}`). The fix is
**localized to OmniBus**, NOT the regression-trap-heavy shared `def_autoload` closure (so it
sidesteps the `ARXIV_PERFORMANCE.md` L420 "don't fix speculatively in the shared path"
hazard). 2209.11799: 200 s TokenLimit fatal ⇒ **1 s, 0 errors** (cite renders). Brings Rust
to **parity with Perl** (Perl already handled this — repro 1.0 s, full 11.7 s). Regression
witnesses all clean: 2310.13684 (`\varmathbb`) 0.3 s/0, 1403.6801 (wlpeerj) 0.6 s/0,
2207.14344 0.4 s/0. Guard: `06_cluster_regressions.rs::cluster_omnibus_natbib_bbl_sideload`.
Also resolves the analogous earlier-deferred 2602.15365 (informs4) class of secondary loop.

## Upstream sync — translate brucemiller/LaTeXML PRs since #2767 (NEW MISSION, opened 2026-06-25)

> **Mission.** The Rust port mirrors upstream Perl LaTeXML through commit
> `23f3acfa` (#2767 "Frontmatter refactor"; record in
> `docs/archive/frontmatter_api_refactor.md`). Upstream `master` has since
> advanced **9 commits** to `cb455179` (#2783). Translate each PR **in merge
> order, as a separate sub-task**, faithfully — `perl-port` discipline: read the
> Perl diff first (`git -C LaTeXML show <commit>`), place per `ORGANIZATION.md`,
> obey the divergence policy (`OXIDIZED_DESIGN.md`). Check an item off here when
> it lands (`git log` keeps the record); archive this catalog when the whole
> mission completes.
>
> **Commit granularity (user directive 2026-06-25): each sub-task = its own
> self-contained commit** — one feature/patch deliverable per commit, never a
> batched mega-commit. The #2798 sub-mission lands as several commits (one per
> sub-step). All work on the **`upstream-sync-prs`** feature branch — never
> FF-push to `main`; open a PR (per `branch-further-stability-coverage-workflow`).
>
> **The `LaTeXML/` checkout is already AT `cb455179`**, so the *new* Perl is the
> live reference for every file; diff against the per-PR commit to isolate a
> single PR's change. Each landed sub-task needs: faithful port + ported test
> fixture(s) (`cargo clean` after adding a `.tex`/`.xml` pair so the test plugin
> rediscovers them — see CLAUDE.md) + `cargo test --tests` green + clippy clean,
> and re-confirm on the current binary.
>
> **Sizing.** 8 of 9 are small/contained (new bindings, listings tweaks, a proof
> fix, a mostly-already-present residual). **#2798 "Leavehorizontal" is the one
> large core-engine refactor** (Font.pm +303, latex_constructs +174, Box/List/
> Stomach/Whatsit/Alignment + ~15 packages) — stage it as its own sub-mission
> with a dedicated regression budget, NOT a single commit.
>
> **Recommended execution order** (the user's "in order" = merge order is the
> default; the independent small wins can land first to build momentum):
> ① #2737 causets → ② #2806 dirtytalk → ③ #2814 proof-punct → ④ #2783
> quantikz2-residual (all independent, **S**) → the **listings cluster**
> #2819 → #2818 → #2824, then #2828 (shared file `listings_sty.rs` + shared
> `listing` fixture — sync the fixture ONCE at the end) → **⑤ #2798
> Leavehorizontal LAST** (largest; its own listings.sty + table/box touches may
> reshuffle the listings fixture again, so doing it after the listings cluster
> avoids double fixture churn).

### U1. ✅ PR #2806 "Add dirtytalk binding" (`51fea96a`) — LANDED
- **What:** `dirtytalk.sty` — `\say{…}` context-aware quotation marks with a
  nesting-depth counter (`dirtytalk@qdepth`): outer level uses
  `\textquotedblleft/right`, nested uses `\textquoteleft/right`. Four KeyVals
  (`left`/`right`/`leftsub`/`rightsub`) let the user override each symbol.
- **Perl:** new `lib/LaTeXML/Package/dirtytalk.sty.ltxml` (54 lines): 4
  `DefMacro` symbol defaults + 4 `DefKeyVal('dirtytalk', …, 'UndigestedKey', …,
  code => setDirtytalkSymbol)` + `ProcessOptions(inorder=>1, keysets=>…)` + a
  `RawTeX` block (`\newcounter`, `\dirtytalk@lsymb/rsymb` `\ifnum`, `\say`).
- **Rust target:** new `latexml_package/src/package/dirtytalk_sty.rs` (register
  in the package module list). `\say` is currently only in `revtex4_support_sty`
  (unrelated). The `\say` core + 4 symbol-default `DefMacro`s + the
  `\newcounter`/`\say` `RawTeX!` block are straightforward (`raw_tex`,
  `Tokens::is_empty` for `IsEmpty` all exist). **The 4 keyval overrides
  (`left`/`right`/`leftsub`/`rightsub`) need the runtime
  `keyval::define(KeyvalConfig { code: Some(…), .. })` directly** — the
  `DefKeyVal!` macro has NO `code`-callback arm and Rust has no `UndigestedKey`
  type, so map Perl's `UndigestedKey` + `code => sub` onto the config's `code`
  field (verified `KeyvalConfig.code: Option<ExpansionBody>` exists).
- **Complexity:** **M** (core `\say` is S; the keyval-override callbacks add the work).
- **Tests:** ported `t/structure/dirtytalk.{tex,xml}` → `latexml_oxide/tests/structure/`
  — Rust output **byte-identical to Perl** (nested `\say` curly quotes), error-clean,
  `dirtytalk_test` green; keyval override (`[left={«},right={»}]` → `«hi»`) smoke-validated
  via the faithful `ExpansionBody::Closure` (incl. the `IsEmpty` guard); clippy clean.

### U2. ✅ PR #2798 "Leavehorizontal" (`24d39b55`) — COMPLETE (2026-06-26, 1470/0; merged to `upstream-sync-prs`)
- **What:** two coupled rewrites + a wide application layer (75 files,
  +1172/−902; ignore the CI-only `windows.yml`):
  - **(A) TeX-faithful mode / `leaveHorizontal`.** In real TeX, beginning a
    vertical/display construct while in horizontal mode first ends the paragraph
    (`\par`); an inline `\hbox`/block does not. LaTeXML scattered
    `leaveHorizontal`/`enterHorizontal`/`\par` inconsistently. Now `beginMode`
    itself calls `leaveHorizontal` when entering a vertical/display bindable mode
    **unless the user mode name contains `inline`**, and a new pseudo-mode
    **`inline_internal_vertical`** (→ bound `internal_vertical`, suppressing the
    auto-leave) marks inline blocks (`\vbox`/`\vtop`/`\parbox`/`minipage`/
    `picture`/footnotes). `digestNextBody` splits into `digestUntil` (digests
    onto the *current* `@LIST` without rebinding) + a thin wrapper; `T_BEGIN`
    only builds a fresh `List` in math mode, else digests onto the ambient list;
    `executeBeforeDigest` pushes results onto `@LIST` instead of returning them;
    `repackHorizontal` records `\hsize`+`\baselineskip` on the finished paragraph
    (and `\hsize` is recorded ONLY there now).
  - **(B) Box/Font sizing rewrite** (the +303 Font.pm is the largest piece).
    Box.pm separates *requested* (`width/height/depth`) from *computed*
    (`cwidth/cheight/cdepth`); `getWidth/…` return only `c*`; new `getSPSize`
    (raw scaled-point triple); `computeSizeStore` bypasses fully-specified sizes,
    marks `isEmpty`, adds padding (`padtop/padbottom/padleft/padright`,
    `totalheight`). Font.pm rewrites `computeBoxesSize` to dispatch by ref-type
    and thread real `baseline`/`totalheight`/math-axis, replacing the old
    `_box/_words/_lines/_stack` helpers with `linebreak_paragraph`/
    `flatten_paragraph`/`split_words`/`collect_lines`/`stack_lines`; CJK
    (`\p{Ideographic}`) counts as `isIdeographic`. Whatsit gains
    `flattenForSizing` (a horizontal whatsit with a pure `#arg`/`#prop` sizer is
    flattened so `\emph` etc. line-break across the paragraph).
  - **(C) Application (~75 files):** new `mode`/`sizer`/padding props across the
    engine pools + ~31 package/class bindings; `\begin{document}` sets
    `\columnwidth=\hsize=\linewidth=\textwidth`; `\emph` → `bounded`+`sizer=>#1`
    (drops `restricted_horizontal`); itemize envs LOSE the mistaken `\par` and
    gain real `\topsep/\parsep/\partopsep/\itemsep` glue + padding; captions →
    new `PBoxContents` param (caption arg processed as a horizontal paragraph);
    `\@framebox` padding from `\fboxsep+\fboxrule`; parbox/minipage/picture →
    `inline_internal_vertical`; + 24 regenerated `t/*.xml` fixtures.
- **Rust state:** core is at the **pre-PR shape** — `stomach.rs` `bindable_mode`
  lacks `inline_internal_vertical`, `begin_mode_opt` does not `leave_horizontal`;
  `executeBeforeDigest` still returns `@pre`; `digest_next_body` is monolithic;
  `T_BEGIN` (`tex_box.rs`) always builds a List; `list.rs` infers mode/width
  (pre-PR); `tbox.rs`/`lib.rs` size-getters use the old `width||cwidth` fallback
  (no `get_sp_size`/padding/`isEmpty`/full-spec bypass); `common/font.rs`
  `compute_boxes_size`+`_words/_lines/_stack/_box` is the pre-PR algorithm;
  `whatsit.rs` has no `flatten_for_sizing`; alignment lacks `replace_column`.
  **Already done in Rust:** the `\lx@add@thanks`/`\person@thanks` removal
  (`base_utilities.rs:210`, `latex_constructs.rs:4505`) + `\lx@personname` sizer
  → S0 is verify-only. **Known overlap:** the p/m/b + `\multicolumn` rework (S9)
  intersects landed Rust array work (memory
  `genuine-rust-only-unexpected-clusters-2026-06-21`, `array_pcolumn`
  reproducers) — reconcile, don't re-port. **`Package.pm`** adds a `$noerror`
  param to `LookupDimension`; Rust's `lookup_dimension` (`state.rs:1613`) is a
  value-cast helper, NOT a faithful `LookupDimension` (it lacks the
  register→macro-`readingFromMouth`→warn-on-undefined logic), so reconcile that
  alongside the `noerror` add (small, but a real semantic gap; see also the
  parked `lookup_register`→`lookup_dimension` eqnarray/cases cleanup).
- **Complexity:** **XL** — two deep foundational rewrites on the hottest
  digestion/sizing paths + wide fixture churn. **Land as separate commits (one
  per sub-step), never one commit.**
- **Ordered sub-steps** (two foundations first; the app layer needs both):
  - **S1 (M, FOUNDATION — namesake)** core mode mechanism: `bindable_mode +=
    inline_internal_vertical`; `begin_mode_opt` calls `leave_horizontal` on
    vertical/display entry when the user mode lacks `inline`. `stomach.rs` + mode
    mapping in `binding/def/dialect.rs`. Keystone; very high blast radius.
  - **S2 (M)** `beforeDigest` pushes onto the active list (not return):
    `primitive.rs`+`constructor.rs` invoke + drop the dialect prepend.
  - **S3 (M, high risk)** extract `digest_until` from `digest_next_body`;
    rewrite `T_BEGIN` (math-only List). Depends S2. (`{…}` grouping is everywhere.)
  - **S4 (M)** `list.rs` `reqmode` + `baseline` on vertical lists; stop setting
    per-List `width`; `repack_horizontal` records `width=\hsize`,
    `baseline=\baselineskip`. Depends S1.
  - **S5 (L, FOUNDATION)** Box.pm rework: `@sizing_properties`; getters return
    `cached_*` only; setters set both; `get_sp_size`; `compute_size_store` with
    full-spec bypass, `isEmpty`, padding/`totalheight`. `tbox.rs` + size trait in
    `lib.rs`. Very high blast radius (every size query).
  - **S6 (L, critical-path risk)** Font.pm rework: rewrite `compute_boxes_size`;
    add `linebreak_paragraph`/`flatten_paragraph`/`split_words`/`collect_lines`/
    `stack_lines`; baseline/totalheight/math-axis; per-box-font kern. `common/
    font.rs`. Depends S5/S4. The +303 change — port behind the `size` debug
    instrumentation and diff line/word/stack vs Perl.
  - **S7 (M)** Whatsit `flatten_for_sizing` + CJK `isIdeographic`. Depends S5/S6.
  - **S8 (M)** Alignment `replace_column` (template+alignment) + 3-tuple
    `normalize_cell_sizes` + `extractAlignmentColumn` PUT/USED split. Depends S5/S6.
  - **S9 (L)** TeX_* pools: `PBoxContents` + `\lx@enterhorizontal`; vbox/vtop →
    inline_internal_vertical; `\vskip` pure-height; display-math pad +
    under/overline; p/m/b inline-block cols + `\lx@tabular@p` + `\multicolumn`
    (overlaps landed work). Depends S1–S8.
  - **S10 (L, highest fixture impact)** latex_base+latex_constructs: `\baselinestretch`;
    moved `\addvspace/\addpenalty/\@endparenv`; `\begin{document}` widths; `\emph`
    bounded+sizer; itemize `\par`-removal + glue + padding + `\preitem@par`;
    caption `PBoxContents`; `\@framebox` padding; parbox/minipage/picture
    inline_internal_vertical. Depends S1–S9.
  - **S11 (M)** ~31 package/class bindings — aas_support, acmart, alltt,
    amsrefs, amstext, array, bbold, beamer, cancel, elsarticle, enumerate,
    enumitem, epsf, fancyvrb, frenchb, glossaries, graphics, hyperref, IEEEtran,
    JHEP, listings, natbib, ntheorem, numprint, paralist, pgfsys-latexml,
    setspace, soul, sv_support, xcolor, xy — mostly mechanical mode/`sizer`/
    property edits + `LaTeXML.css` (`ltx_verbatim` nowrap, overline/underline
    classes) + `LaTeXML-picture-xhtml.xsl`. Depends S1–S10.
  - **S0 (S)** verify the already-done thanks/`\lx@personname` bit — expect no-op.
- **Tests:** no new `.tex`; 24 regenerated `t/*.xml` (alignment
  array/cells/colortbls/halignatt/tabular; math array_newline_math; complex
  figure_dual_caption/figure_mixed_content/cleveref_minimal/equationnest;
  structure authors/autoref/enum/figure_grids/figures; fonts marvosym/sizes;
  digestion dollar; babel numprints; ams mathtools; graphics
  graphrot/picture/xytest; tokenize alltt). All exist in `latexml_oxide/tests/`
  at pre-PR output — **regenerate each from same-host Perl `cb455179` and diff;
  never hand-edit** (legit size/paragraph-structure churn; a few intentional
  divergences per `OXIDIZED_DESIGN.md`). Gate each fixture on its sub-step.
- **✅ U2 COMPLETE 2026-06-26 (loop session 2) — `u2-leavehorizontal` at
  1470/0, clippy clean** (was 1466/4). All 4 residual failures resolved with
  faithful #2798 ports + principled regenerations. The branch is ready to merge
  into `upstream-sync-prs` for the single combined PR (resolve the SYNC_STATUS
  conflict in favour of this branch's final state — the 21 docs-only
  investigation commits on `upstream-sync-prs` are superseded; notably the old
  "various_colors DEEP, no clean fix / do NOT regenerate" note is WRONG, see
  below). Landed on the branch (each a self-contained WIP commit):
  - **`\phantom` h/d via single-box short-circuit** (`font.rs`
    `compute_boxes_size`): port Perl `computeBoxesSize` L646-647 — a single bare
    Box/Whatsit (not a List) returns `get_size` directly, ahead of the
    List/split_words path (which discards height/depth for an `isSpace` box).
    `sizes` math-strut `\phantom{g(x)}` 18.62×0+0 → 18.62×7.5+2.5 = cb455179
    Perl. Also moved `consort-flowchart` picture height 782.9 → **825.33 =
    Perl exactly** (tikz node content lost h/d the same way); regenerated.
  - **`\framebox` pad{top,bottom,left,right}=`\fboxsep+\fboxrule`** (3.4pt;
    `latex_constructs.rs` `\@framebox` properties): Perl sets these alongside
    `sizer=>#3`; the S5 padding slice (`compute_size_and_cache`) adds them.
    `graphrot` rotated `\framebox{\usebox}` inner dims 72.3×24+19 →
    79.1×27.4+22.4 = Perl; whole `\testrot` cluster aligns; Rust-vs-Perl
    26→8 lines. Full suite: no ripple to other fbox/framebox tests.
  - **Regenerated (Rust-specific fixtures, #2798 moved them toward Perl):**
    `sizes` (vbox 469.755→345 = Perl; phantom; residual g@(x) math-parser +
    `\vtop{tabular}` 37.055 — see below), `graphrot`, `consort-flowchart`,
    `figure_mixed_content` (subfigure scale 1.131→0.900≈Perl 0.881, width
    156.1→124.2 = Perl).
  - **Documented intentional/pre-existing divergences (kept, not "fixed"):**
    (a) `sizes` `\vtop{tabular}` width 37.055 (natural) vs Perl 345=`\hsize`:
    Perl repacks restricted_horizontal `\@@tabular` into a width=`\hsize` List
    inside the vtop's vertical list; Rust marks `\@@tabular`/`\halign`
    internal_vertical so repack SKIPS it — the [[box-model-hsize-frame-ordering-fix]]
    that cured the Cluster-G `\narrow`-hang (1610.00974). Matching Perl risks
    that hang. (b) `sizes` g(x) `g@(x)` (function-application) vs Perl `g * x` —
    pre-existing math-parser semantics. (c) `figure_mixed_content` panel
    `<break>` divergence — Rust's simplified `arrange_panels` + load-bearing
    `\par`-in-figure break (used by article/book/report/amsarticle/tikz_figure)
    vs Perl's width-based `breakIntoPanels`.
  - **✅ `various_colors` RESOLVED — stale fixture, regenerated.** The tcolorbox
    inner width 40.23em (6in-based) was a STALE RUST-BUG artifact, NOT a Perl
    divergence (correcting the prior session's "DEEP / do NOT regenerate" note).
    Proof: `tcolorbox.sty` defaults `width=\linewidth` (L2947), evaluated LAZILY
    at box-build (body) time via `\tcbdimto` (L1267); body `\linewidth`=345 in
    BOTH Perl (measured v0.8.8 + cb455179: preamble 433.62/6in → body 345) and
    Rust → both engines yield 31.37em. Old Rust never reset body `\linewidth`
    from its 6in default, baking 40.23em into the fixture; #2798's
    `\begin{document}` width-consistency (FAITHFUL — Perl matches) fixed that
    latent bug. Regenerated from Rust (16 lines, all inside the tcolorbox picture
    region). cb455179 Perl's own tcolorbox run HANGS in pgf-Perl (>5 min CPU on
    a minimal doc), which is why the value was settled by analysis + the lazy
    `\linewidth` proof rather than a direct Perl diff.
- **Empirical coupling (measured 2026-06-25 — S1 prototyped then reverted):**
  S0 verified (thanks/`\lx@personname` already in Rust — `base_utilities.rs:215`,
  `:807`; no-op). A standalone **S1** prototype (added `inline_internal_vertical`
  to `bindable_mode` + the `leave_horizontal()`-on-vertical/display-entry guard
  in `begin_mode_opt`, `stomach.rs:663,681`) **breaks exactly 11 tests**, which
  split cleanly:
  - **4 genuine regressions** — `footnote`, `endnote`, `etoolbox`, `fancyhdr`
    — are **NOT** in Perl's 24-fixture regen list, so Perl's output for them is
    unchanged by #2798. They break only because the inline blocks (footnotes
    etc.) aren't yet reclassified to `inline_internal_vertical`, so they wrongly
    `leaveHorizontal`. **Fix = the S9/S10 inline reclassification**, after which
    their existing fixtures pass again.
  - **7 legit regens** — `dollar`, `autoref`, `enum`, `equationnest`,
    `figure_mixed_content`, `picture`, `sizes` — **are** in Perl's regen list.
    Per-fixture root cause (Perl diffs are tiny, 4–14 lines each), which
    refines the earlier "all need S6" claim:
    - `dollar` (4), `equationnest` (8): **pure spacing** — the `leaveHorizontal`
      paragraph effect drops a space (`</equation> t`→`</equation>t`). S1 only.
    - `autoref` (4), `figure_mixed_content` (10): **width consistency** —
      minipage/figure width `433.6pt`→`345.0pt`. Root cause: Rust DefRegisters
      `\columnwidth`/`\linewidth` = `6in` (=**433.62pt**) and never resets them;
      #2798's `\begin{document}` adds `\columnwidth=\hsize=\linewidth=\textwidth`
      (=`345pt`). Found: Perl `latex_constructs.pool` `\begin{document}` handler
      (diff L139–142); Rust target `latex_constructs.rs:3107` (right after the
      `\everypar` clear). The figure_mixed inline-block scale change cascades
      from this width (124.2/156.1 = 345/433.6). **Width fix only, no Font.pm.**
    - `sizes` (8): widths→`345.0` (width consistency) **plus** one genuine
      math-axis change (`18.62154pt x 0.0pt + 0.0pt`→`x 7.5pt + 2.5pt`) — that
      one needs **S6** math-axis height/depth.
    - `enum` (14): tags gain `cssstyle="padding:3.0pt"` — needs **S5** Box pad
      props + `\lx@tag` sizing.
    - `picture` (14): `innerwidth` text-label widths shrink ~8% — needs **S6**
      text word-width measurement.
  - **So the dependency is narrower than first stated:** S1 + the inline
    reclassification + the `\begin{document}` width-consistency clear 4–6 of the
    11 on their own; only `picture`, the `sizes` math-axis line, and `enum`
    padding truly require the S5/S6 sizing work. **But it is still test-atomic
    for COMMIT purposes** — applying any one piece (even the width clear alone)
    turns its fixtures red until regenerated, and regenerating before the
    matching code lands trades one red for another. Land U2 as **one coherent
    push** (S1 → inline reclassification → `\begin{document}` widths → S5/S6
    sizing → S2–S4/S7–S11 → regenerate all 24 fixtures from same-host Perl
    `cb455179`), green only at the end. Known diffs to re-apply first:
    `bindable_mode`+`begin_mode_opt` (S1, above) and the 4-line
    `\begin{document}` width block. Prototype reverted; branch stays green.

### U3. ✅ PR #2819 "listings: create group around identifiers" (`0d748100`) — LANDED (absorbs U5)
- **What:** in `lstSetClassStyle`, a TeX-style class now wraps its styling in a
  brace group — `begin => Tokens($style, T_BEGIN)`, `end => T_END` (was
  `begin => $style` only). And in `lstProcess_internal`, index emission uses
  `lstRescan($index{begin})…lstRescan($index{end})` (was bare `T_BEGIN … T_END`).
  Net effect: identifier/keyword styling is grouped (e.g. `\bfseries\underbar`
  applies as a group so the underline spans the whole keyword).
- **Perl:** `lib/LaTeXML/Package/listings.sty.ltxml` (`lstSetClassStyle` ~L496;
  `lstProcess_internal` ~L1413).
- **Rust target:** `latexml_package/src/package/listings_sty.rs` —
  `lst_set_class_style` (TeX branch, ~L450) + `lst_class_end` (~L956).
- **Rust port (this branch):**
  - `lst_set_class_style` TeX branch: `begin = Tokens(style, T_BEGIN)`, add
    `end = T_END` — faithful to the Perl diff.
  - `lst_class_end`: changed from **leaf-only** end collection to walking the
    **full class chain** (push order: leaf close-delims first, parent styling
    group-closers last), so the `T_END` added to a parent styling class
    (comments/strings) matches the `T_BEGIN` in its `begin`. Pre-#2819 only leaf
    delimiter classes carried an `end`, so this is a no-op extension there;
    keyword/identifier classes are themselves the leaf, so their `T_END` was
    already collected. Verified faithful for both keyword-leaf and
    delimiter→styling chains (matches Perl's `@close` order exactly).
  - The `lstProcess_internal` **index** line-change has **no functional Rust
    target**: Rust's index branch is a no-op stub (`// Index generation
    (simplified)`) that emits nothing, so the bare-`T_BEGIN/T_END` → `index{end}`
    swap is moot until the index feature is implemented (left as-is).
- **Complexity:** **M.**
- **Tests:** resynced `t/alignment/listing.{tex,xml}` (added `\underbar` to the
  bingo keywordstyle). Rust output for the grouped keyword now renders
  `<text class="ltx_lst_keyword ltx_underline" font="bold">foo</text>` —
  **matching upstream HEAD `cb455179` (post-#2828) exactly**; the only delta vs
  Perl is the missing `<indexmark>` blocks (pre-existing index-stub divergence,
  not introduced here). `53_alignment` suite (9 tests incl. `listing_test`)
  green; error-clean; clippy clean. **#2828 (U5) is absorbed**: Rust's
  `\underbar` renders natively as `ltx_underline` (the settled form), so the
  #2819→#2828 underline transition happens in one step here.

### U4. ✅ PR #2818 "listings: do not look up ltxml files when reading raw files" (`41bd31e8`) — LANDED
- **What:** `listingsReadRawFile` now calls `FindFile($filename, noltxml => 1)`
  so `\lstinputlisting{foo.sty}` reads the raw source, never a `.ltxml` binding.
- **Perl:** `listings.sty.ltxml` `listingsReadRawFile` (~L320).
- **Rust target:** `listings_sty.rs` `listings_read_raw_file` (L234) — pass the
  `noltxml` flag to the find-file call.
- **Complexity:** **S** (one-flag change). Rust spells `noltxml` as `forbid_ltxml`
  in `FindFileOptions` (matches `tex_job.rs:252`).
- **Tests:** none new; listing suite (9 tests) green — output unchanged, so it's a
  clean independent commit.

### U5. ✅ PR #2828 "Resync listings test for change to underline" (`39f319bd`) — LANDED (absorbed into U3)
- **What:** test-only follow-up to #2819 — the underline styling settled to
  `class="ltx_lst_keyword ltx_underline"` (from the intermediate
  `framed="underline"`).
- **Perl:** `t/alignment/listing.xml` only.
- **Rust outcome:** **absorbed into U3** — Rust's `\underbar` renders natively as
  `ltx_underline`, so the U3 fixture regen produced the final post-#2828 form
  (`ltx_lst_keyword ltx_underline`) directly, in one step. No separate commit.
- **Complexity:** **S** (fixture resync).

### U6. ✅ PR #2814 "Fix 2240 proof title punct" (`01b8d651`) — LANDED
- **What:** the amsthm `proof` env stops double-punctuating — append the trailing
  period only when the (optional) title doesn't already end in `.!?:;,` (mimics
  LaTeX `\@addpunct`). `\begin{proof}[x.]` → "x." not "x..".
- **Perl:** `lib/LaTeXML/Package/amsthm.sty.ltxml` `\@proof` properties (~L155):
  inspect `$content[-1]->toString` and conditionally add `T_OTHER('.')`.
- **Rust target:** `latexml_package/src/package/amsthm_sty.rs` — the `\@proof`
  `properties` closure currently does an **unconditional** `title_tokens.push(
  T_OTHER!("."))` at **L188**; gate it on the last content token's last char.
- **Complexity:** **S.**
- **Tests:** ported `t/theorem/proofpunct.{tex,xml}` → `latexml_oxide/tests/theorem/`
  — Rust output **byte-identical to Perl**, error-clean, `proofpunct_test` green.

### U7. ✅ PR #2737 "Added bindings for causets (TikZ extension)" (`eb08bd7f`) — LANDED
- **What:** `causets.sty` binding = a raw-load passthrough:
  `InputDefinitions('causets', type => 'sty', noltxml => 1)`.
- **Perl:** new `lib/LaTeXML/Package/causets.sty.ltxml` (24 lines, body is the
  one `InputDefinitions` call).
- **Rust target:** new `latexml_package/src/package/causets_sty.rs` that
  raw-loads `causets.sty` with `noltxml`. Binding itself is trivial; actual
  rendering depends on the host TikZ machinery (out of scope for the binding).
- **Complexity:** **S.**
- **Tests:** none added upstream. Smoke-validated: `\usepackage{causets}` loads
  error-clean and raw-loads the host `causets.sty` (`Loading causets.sty…`),
  body renders; clippy clean.

### U8. ✅ PR #2824 "do not add frame and background to inline listings" (`a6f6316f`) — LANDED
- **What:** `\lstinline` / `\begin{lstinline}` now set `LISTINGS_INLINE => 1`;
  `\lst@@@set@frame` and `\lst@@@set@background` skip the frame/background when
  `LISTINGS_INLINE` is set (inline listings shouldn't get a box frame/bg).
- **Perl:** `listings.sty.ltxml` — `\lx@lstinline` (~L58), `\begin{lstinline}`
  (~L94), `\lst@@@set@frame` (~L929), `\lst@@@set@background` (~L958).
- **Rust port (this branch):** set `LISTINGS_INLINE` (local to the inline
  bgroup) in `\lx@lstinline` (after `lst_activate`, before reading the
  delimiter) and the `lstinline` environment; `\lst@@@set@frame` returns an
  empty `frame` prop when inline (constructor already skips an empty `framed=`).
  For `\lst@@@set@background`, guarded the **entire** body on `!inline` — not
  just the `merge_font`. Perl never clears `LISTINGS_BACKGROUND`; Rust's
  `assign_value(None)` is a workaround so a *block* listing doesn't leak its bg
  to later listings. Guarding the clear too keeps that workaround for block
  listings while leaving the global value intact when an inline listing runs
  first, so a **following block listing still renders its background** (verified
  by repro: global `\lstset{frame=single,backgroundcolor=\color{yellow}}` →
  inline gets neither, the next block listing gets both — matching Perl). A
  naive merge-only guard regressed that case.
- **Complexity:** **S–M.**
- **Tests:** none new — `53_alignment` `listing` suite (9 tests) unchanged-green
  (matches upstream: #2824 had no Perl fixture change); error-clean; clippy
  clean; behaviour confirmed by the inline-vs-block repro above.

### U9. ✅ PR #2783 "quantikz2 raw interpretation" (`cb455179`) — LANDED (color-macro residual)
- **What:** four fixes (this PR was authored by us and partly upstreamed
  Rust-discovered fixes): (a) `\AtBeginDocument[]{}` / `\AtEndDocument[]{}`
  optional `[label]`; (b) `color.sty` defines `\current@color`/`\default@color`/
  `\reset@color` with safe DVI defaults; (c) `tcolorbox.sty` pre-defines
  `\tcb@use@autoparskip` (drops the `expl3`/`xparse` RequirePackage); (d)
  `\hphantom` math/text split (`\lx@math@hphantom` / `\lx@text@hphantom` with
  `restricted_horizontal`) to stop display-math leaks. Plus pure whitespace
  realignment in `math_common.pool` (no semantics).
- **Rust state (verified 2026-06-25):**
  - (a) `\AtBeginDocument[]{}` — **ALREADY in Rust** (`latex_constructs.rs:3073`).
  - (c) `\tcb@use@autoparskip` — **ALREADY in Rust** (`tcolorbox_sty.rs:17`).
  - (b) color macros — **✅ DONE** (this branch): added `\current@color`/
    `\default@color`/`\reset@color` to `color_sty.rs` (`'0 0 0'`/`'0 0 0'`/empty,
    in Perl order before `\set@color`) + updated the comment. Validated:
    `\makeatletter` smoke error-clean; graphics (8) + tikz (10) suites green; no
    regression. (This was the only real porting work in #2783.)
  - (d) hphantom split — **INTENTIONALLY DIVERGED in Rust** (`math_common.rs:1037`):
    the `restricted_horizontal` wrapping was reverted because it FATALs on
    `\minipage…\hphantom\endminipage` (2004.10048), and the `$$`-leak it guards
    errors in installed Perl too. The new #2783 form uses the same mechanism →
    re-evaluate only if the mode-end fatal is solved; otherwise keep the
    divergence and document it against the new upstream form.
- **Complexity:** **S** (color macros; the rest is verify/divergence-note).
- **Tests:** none new; ensure tcolorbox/quantikz/color regressions stay green.

### U10. ✅ PR #2833 "Remove \@ifnext, use \@ifnextchar" (`346279c9`) — LANDED (2026-06-26)
- **What:** `\@ifnext` was a kernel alias `Let('\@ifnext','\@ifnextchar')` (with a
  `# ????` comment). The PR drops the alias and calls `\@ifnextchar` directly in
  its two use sites: `\@caption` (`latex_constructs.pool`) and `\@captionof`
  (`caption.sty`). Behaviour-preserving (they were aliased).
- **Rust port (this branch):** 3 edits — `latex_constructs.rs` `\@caption`
  body `\@ifnext`→`\@ifnextchar` + removed `Let!("\\@ifnext","\\@ifnextchar")`;
  `caption_sty.rs` `\@captionof` body `\@ifnext`→`\@ifnextchar`. Also updated the
  `semantic_sty.rs` header comment: it pre-undefines `\@ifnext` (semantic.sty's
  `\TestForConflict` errors on kernel-defined CSes, witness 2403.04708) — with the
  alias gone that pre-undefine is now a defensive no-op (kept). No other bare
  `\@ifnext` users (`\@ifnext@n` is a separate macro, retained).
- **Complexity:** **S** (mechanical, behaviour-preserving).

### U11. ✅ PR #2832 "initial \multicolumn content starts the new column" (`bc90e36c`) — N/A in Rust (verified)
- **What:** Perl inserts a `\relax` between `\lx@alignment@altcolumn{template}`
  and the cell `$tokens` in `\lx@alignment@multicolumn` (`TeX_Tables.pool`), so an
  initial `\multicolumn`'s content starts the new column rather than being absorbed
  by the template scan in `\halign`'s `\span`/`\omit` mechanics.
- **Rust state:** **no code change needed.** Rust's `\lx@alignment@multicolumn`
  (`tex_tables.rs:570`) is structurally divergent — it does NOT use
  `\lx@alignment@altcolumn`; it inlines the column template directly (p/m/b → VBox
  via `\lx@tabular@p`; normal → `before_cell` + body + `after_cell`) and digests
  via the custom `alignment_bindings` processor, not real `\halign` `\span`
  mechanics. The `\relax` boundary the PR adds guards a TeX gullet-scanning
  subtlety that does not exist in that path. **Verified:** Rust output is byte-
  identical to cb455179 Perl (which has the fix) across initial-`\multicolumn`
  edge cases (bordered bold head, empty cell, 2-col span, math content). Adding a
  `\relax` would risk regressing the matched output for zero benefit. Documented
  as a structural divergence; revisit only if a real initial-multicolumn
  divergence surfaces.
- **Complexity:** **S** (verify + divergence-note; no code).

## Methodology & the cortex cross-join

Working method (2026-06): **re-triage LARGE-error papers** (the single-error tail
is exhausted) → bisect the doc to the trigger line → verify Perl with `--verbose`
→ fix the divergence. Random sweeps are low-yield.

**Cortex agentic API (reads open, no token):** `http://127.0.0.1:8000/api`.
Recipe: `GET /api/reports/<corpus>/oxidized-tex-to-html/<severity>` → categories;
`…/<severity>/<category>` → per-`what`; `…/<category>/<what>` → paper list. Then
`GET /api/corpus/<corpus>/tex_to_html/document/<id>` for Perl status — a Rust-only
win is **Perl=no_problem/warning but Rust=error/fatal**. Corpus
`sandbox-arxiv-10k-shuffle`. URL-encode `\`→`%5C`, `^`→`%5E`.

**State of the autonomous methods (2026-06-21) — all tapered; a FRESH cortex
rerun is the clear next step:**
- *Stale 10k error cross-join*: **mined out** — every remaining apparent
  "Rust-only" cluster traced to a SHARED cause (third-party class/pkg neither
  engine binds; author errors; stale pre-fix run). **2026-06-21 re-check via the
  live cortex `document/<id>` API (not the stale ad-hoc join):** the last two
  candidates were BOTH phantom — `1308.2655` "Extra alignment tab" on
  `\lefteqn`/`\multicolumn{N>cols}` is **parity** (Perl 1 error, Rust 1 error —
  Perl's `nextColumn` errors on column overflow too, `Alignment.pm:136-144`); and
  `0710.5692` `equationgroup isn't allowed in <ltx:p>` is **parity** (Perl 2,
  Rust 2). An ad-hoc same-tree cross-join had falsely reported both as "Perl 0";
  the stable cortex DB is authoritative. **Lesson: confirm every cross-join
  "Rust-only" read against the live cortex `document/<id>` API before chasing —
  do not trust a bespoke join's Perl column.** (One genuine *minor* residual on
  `0710.5692`: Rust reports the equationgroup location as `Anonymous String` vs
  Perl's `cosmo_sing_iwa.tex; line 1124` — a source-locator gap, belongs to the
  #47/#92 source-map track, NOT a parity/correctness bug.)
- *Diagnostic-message faithfulness*: **exhausted** — a systematic batch
  comparison (undefined CS/env, missing-number, group/mode close, malformed,
  close-environment) shows all primary messages matching Perl.
- *Structural-skeleton diff on Perl-clean papers* (the silent-divergence method
  that found the REVTeX/OmniBus `\references` fix): now consistently surfaces
  only the DEFERRED families — MathFork/content-MathML (`equation > tags`) and
  document-builder block/paragraph auto-wrap — plus cosmetic/niche cases.
  **Re-run 2026-06-27 with current code** on 22 both-clean papers (0704 + 1401/1501).
  **CORRECTION (self-caught):** my first-pass harness matched only `<(ltx|m|svg):`
  *prefixed* elements, but the core XML uses the DEFAULT namespace
  (`xmlns="http://dlmf.nist.gov/LaTeXML"`, so `<break/>`/`<graphics/>`/`<block>` —
  no prefix), so it compared near-empty sets and falsely reported "identical." The
  CORRECTED all-element diff shows real silent divergences on both-clean papers —
  **all in the known DEFERRED families**, re-confirming the assessment above:
  - **document-builder figure-panel block-wrap** (0704.0001, 0704.0017): Perl wraps a
    consecutive `\includegraphics` run in `<block class="ltx_figure_panel">`; Rust
    emits the graphics bare with `class="…ltx_figure_panel"` — AND the panel sizing
    differs (Rust `width=303.5pt` vs Perl `241.5pt`, a consistent ~1.257×, a
    downstream consequence of the missing panel block). This is the "broad/risky"
    document-builder item below — now FRESHLY witnessed on clean papers; it has real
    visual impact (figure widths), not just structure.
  - equation `<tags>`/`<tag role="refnum">` (1501.00006), `<p>` paragraph auto-wrap
    (0704.0007), and content-MathML (`\qquad`→`formulae@`/`fragments@`,
    `ltx_math_unparsed`, XMDual) — deferred math-fork.
  - cosmetic serialization (trailing space `<XMArray … >`) and a Rust-BETTER case
    (1401.0003: Perl italicizes empty-base math-superscript digits `$^{12}$`→
    `<text font="italic">12`, Rust keeps them upright — correct for math digits).
  No NEW bounded *non-deferred* gap; the one real-impact divergence (figure-panel
  block-wrap) is the deferred broad/risky document-builder item.
- *Binding-completeness set-diff*: too noisy to be useful — it misses every
  macro defined via `TeX!(r"…")` raw-TeX blocks (single-backslash), so its
  flagged "gaps" are mostly false positives (verified: longtable `\LTcapwidth`
  etc. ARE defined). OmniBus was confirmed structurally complete this way.
- *PACKAGE-level binding diff (2026-06-27)*: cleaner than the macro set-diff —
  Perl 408 `*.{sty,cls}.ltxml` vs Rust's 389-entry registry. After excluding
  classes handled via `*_support`/omnibus (elsart/revtex/aastex/JHEP/emulateapj/mn…)
  and packages aliased/partially-handled (the algorithm family WORKS: `algorithm`
  registered + `algorithmic`/`\STATE`/`\IF` → Rust 0 = Perl 0), only **2** are
  genuinely absent from Rust: `causets` (niche physics) and `dirtytalk` (`\say{}`)
  — and BOTH are **unwitnessed** in the 10k sandbox (0 cortex `undefined` entries),
  so per the demand-driven rule they're left unported. Rust package coverage is
  complete for what arXiv witnesses.
- *fatal/TooManyErrors mining (2026-06-22)*: **mined out — ZERO genuine
  Rust-only bugs.** Of 35 `MaxLimit(100)` papers: 24 Perl-fatal (parity), **9 a
  `cp1251`/Cyrillic env artifact** (all `[cp1251]{inputenc}`+`[T2A]{fontenc}`+
  russian babel → ~100 `unexpected:<char>` each; the `cyrillic`/`t2` TeX package
  is missing on this host so `cp1251.def`/`t2aenc.def` are absent — **local Perl
  fails identically**, the cortex Perl=clean came from a host WITH the package),
  2 stale/marginal. Same env-artifact family as the isolatin phantom. **Cyrillic
  coverage fix is host-side (`tlmgr install cyrillic cm-super`), not a code bug;
  an optional surpass-Perl charset-decode fallback for missing inputenc `.def`s
  would convert them without the host package (needs authorization).**
- *fatal/Timeout mining (2026-06-22)*: 18 papers → 16 Perl-fatal (parity), 2
  candidates. `1506.09195` = missing custom `my_paper.sty` + deep expl3/datatool/
  l3fp (local Perl also fatals; Rust runs the conditional runaway to the IfLimit
  guard). **`1707.02464` = the ONE genuine Rust-only bug from all 53 fatal papers:
  Perl completes in 11.76s, Rust hangs to the 60s watchdog** — a custom
  `\narrow` macro's `\hsize`-shrink loop never terminates because Rust's vbox
  `\ht` is `\hsize`-invariant (Perl models paragraph height ∝ `\hsize`). Recorded
  as `STABILITY_WITNESSES.md` Cluster G (open; box-model fix, regression-risky,
  warrants a focused session).
- *error-severity sweep (2026-06-22)*: full cross-join of the cortex `error`
  severity (1189 tasks) on the **same local host** (env-artifact discipline).
  **Parity/env-artifact dominated; ONE genuine Rust-only correctness bug.**
  - `malformed` (162): all parity except **`ltx:itemize` in a `p{}` cell** — the
    p{}-block-content bug (1510.07685), root = **1610.00974 step-3**, now
    **✅ FIXED 2026-06-22** (`f65b80c1c2`, the p{}→VBox port, unblocked by the
    Cluster G box-model fix `7545e07fd6`). `_CaptureBlock_`/listing errors are
    Perl-identical (parity).
  - `latex` (31): all parity. Every package `\PackageError` (`\GenericError`,
    `(ifthen)`, `(newunicodechar)` 189, `(etoolbox)` 187, `(glossaries)` 224,
    `(pgfkeys)`) is shared. The `(babel)` `Unknown option 'russian'`/`'ukrainian'`
    cluster (11 papers, cortex Perl=warning) is a **babel-VERSION env artifact**:
    local babel.sty ≥3.9 (locale-based) errors on the `russian` *option*
    (`russianb.ldf` absent), and **local Perl emits the IDENTICAL single error**
    (0709.3796: Rust==Perl==1). The cortex Perl=warning host had pre-3.9 babel.
    Same class as the isolatin/cp1251 phantoms; not a code bug (a `babel_lang_stubs`
    russian/ukrainian stub would surpass local-Perl + overlap the Cyrillic
    host-side decision → left as-is).
  - `missing_file` (31), `misdefined` (3), `document` (2), `xpath` (2): all parity.
  - `undefined` (890): top-20 whats all parity — the `imsart` bib cluster
    (`\bauthor`/`\bfnm`/`\btitle`/… + `{barticle}`, 16 papers) and `{diagram}`
    (17/19) are **Perl-also-undefined** (Perl LaTeXML ships no imsart/diagram
    binding either). Confirms "undefined = shared third-party CS".
  - `unexpected` (268): the big "Script `_`/`^` can only appear in math mode" +
    "Misplaced alignment tab `&`" clusters are **100% parity** under a FULLY
    PAGINATED cross-join (`_` 109/109, `^` 45/45, `&` 51/51 papers — no math-mode
    detection divergence; these are genuinely-malformed unescaped inputs both
    engines flag). The only "candidates" were the `<char>` inputenc Cyrillic/latin
    env-artifact cluster (0802.1123 isolatin, 1008.0492/1011.5076 babel-russian,
    1009.2998 `[cp866]`+`[T2A]` — host missing the `.def`; same class as Clusters
    A/C/E) and `\end{table}`/1805.00875 (**already FIXED** — see next).
  - **META (2026-06-22): the cortex Rust service data is STALE** (predates recent
    branch fixes). 1805.00875 (dcolumn) shows `unexpected/\end{table}` in the
    cortex report but converts **0 errors on the current binary** (the 2026-06-21
    dcolumn fix is in). So a flagged "Rust-only candidate" may already be fixed —
    **always re-confirm on the current binary** (the genuine finds 1510.07685 /
    1707.02464 were). A **fresh cortex Rust rerun built from this branch** is the
    real prerequisite for surfacing NEW genuine Rust-only correctness bugs; the
    stale data is still authoritative for *parity* and *env-artifact* classes
    (those don't change). **Conclusion: the entire `error` severity is mined out —
    parity + env-artifacts; the one genuine find (p{} block content, 1510.07685) is
    now ✅ FIXED (1610.00974 step-3 port + Cluster G box-model fix, 2026-06-22).**
  - **Current-binary same-host sweep (2026-06-22):** a fresh 24-paper deterministic
    corpus sample (LaTeX2e + LaTeX 2.09 `\documentstyle` + revtex, multi-domain),
    current Rust vs **verbose** local Perl (avoiding the `--quiet` trap). 21 real
    TeX papers (3 were `\documentstyle` 2.09, 1 a misnamed PostScript file):
    **ZERO Rust>Perl divergences — Rust is at-or-better than Perl on every paper.**
    Parity on most (0/0, 33/33); **Rust BEATS Perl on 4** (1509.03503 Perl
    timeout→Rust clean; 1604.03906 3 vs 101; astro-ph0210479 18 vs 101; 1712.01466
    2 vs 3). Confirms the stale-DB mining: no genuine Rust-only bugs findable on the
    current binary without a fresh cortex rerun. Sweep harness:
    `tools/`-style `/tmp/sweep.sh` (grep `\documentclass` misses 2.09
    `\documentstyle` — heuristic note, not a Rust gap).
  - **`warning` severity mined too (2026-06-22) → nothing new actionable.** Of 2208
    warning tasks, the bulk is **user-deferred math** (`ambiguous` 1348 + `expected`
    1181 = `not_parsed` "MathParser failed to match" — content-MathML) + env
    (`missing_file` 590). The small non-math categories are niche graceful-recovery
    warnings, all parity/faithful: `unsupported/multirow` ("Negative row sizes … not
    yet supported") is a **line-for-line mirror of Perl `multirow.sty.ltxml`:27-28**
    (Perl doesn't support it either — implementing would surpass Perl);
    `malformed/{_CaptureBlock_,labels,ltx:Proof}` (1-5 tasks) are graceful fallbacks
    for custom/edge constructs. **All cortex severities (error/fatal/warning) are now
    mined; no unblocked, in-scope, non-surpass Rust-only bug remains findable on the
    current binary.**
  - **`ambiguous/math` triaged 2026-06-27 — NOT a suppression target.** The
    biggest warning category (977 tasks / 10713 msgs) is the `log_math_warn!`
    "Ambiguous math: N enumerated (… pruned … unique)" message (`parser.rs:2030`),
    which fires when a formula enumerates >10 Marpa parse trees. It is an
    INTENTIONAL Rust-internal diagnostic (the comment: "Diagnostic only — high
    ambiguity isn't a Perl-side Error") — Perl's RecDescent can't emit it (no tree
    enumeration), so Perl=0, but it was deliberately KEPT in the 2026-06-25
    signal-fidelity pass. Do NOT downgrade it (violates the don't-silence directive
    + hides the parser-struggle signal; it's a parser-dev metric, also available via
    `LATEXML_PARSE_AUDIT`). The UNDERLYING unparsed math (`ltx_math_unparsed`) is
    MOSTLY PARITY — hard physics math both engines fail on (`0705.2208` Rust 320 /
    **Perl 273** of 1734 Math; `\mathfrak{su}(4)\oplus_s`, `{}^{*}G_{…}`). The modest
    Rust-EXCESS (+47 here) is the genuine but DEFERRED open-ended math-coverage gap
    (each a distinct grammar rule; the deferred math-fork session, not loop-tick).
    **★ One common Rust-excess gap FIXED (2026-06-27):** a BARE bigop as a `/`-frac
    numerator — `\partial/\partial t` (Leibniz partial-derivative, pervasive in
    physics) — was `ltx_math_unparsed` (a bigop must apply, so `bigop /` had no
    rule). Added the divide-scoped rule `any_bigop divide term` (`builder.rs`),
    matching Perl `partial-differential / partial-differential@(t)`. Scoped to the
    `/` divide-MULOP (NOT all `mulop`) so it doesn't fire on `\partial \times B`
    (which regressed mathtools). 0705.2208 unparsed 320→315; suite 1472/0; clippy
    clean; regression `cluster_partial_over_partial`. The other excess gaps remain
    deferred (each a distinct, more-specialized grammar fix): **`\star_N`** =
    SCRIPTED PREFIX MULOP (`\star_3 x` lexes `MULOP start_POSTSUBSCRIPT 3
    end_POSTSUBSCRIPT UNKNOWN`; `\star x` parses but the scripted prefix form has
    no rule; Perl `absent star _ 3 x`). `scripted_mulop` exists (`builder.rs:852`)
    but is INFIX-only. ATTEMPTED 2026-06-27 (reverted): `scripted_mulop tight_term
    => prefix_apply` makes it parse BUT as `(star)@(x)` (apply) — DIVERGES from Perl
    `absent star x` (binary-with-absent-left). Needs the absent-binary prefix action
    the bare `\star x` uses (mechanism not located — likely a kludge/fallback, not a
    clean grammar rule); `prefix_apply` is wrong. The clean fix is the focused
    math-fork session. **`\Omega_{+,+-}`** = a comma-list of OPERATORS in a
    subscript (`+,+-` → Perl `list@(+, absent + -)`) — plain subscript-lists
    (`\Omega_{a,b}`, `\Omega_{p,1\bar 1}`) ALREADY work/match Perl; only the
    operator-list form fails = the PARKED "N-ary bare-operator listing" aside.
    **prefix `{}^{*}[…]`** = empty-base prefixed-star (the empty-base-script family,
    cf 0707.1339 `{}^{++}`). **CONCLUSION (2026-06-27): every remaining
    math-coverage gap connects to a known finicky/parked area** (scripted-prefix
    operators, N-ary operator-lists, empty-base scripts) — all ambiguity-sensitive
    → the deferred focused math-fork session, NOT loop-tick work. The clean common
    gap (`\partial/\partial t`) is landed.

**NEXT: a FRESH cortex Rust rerun built from this branch** (needs
`X-Cortex-Token`) is the prerequisite for mining genuine Rust-only *correctness*
wins now that the diagnostic messages are faithful; always re-confirm a flagged
paper on the CURRENT binary before chasing it. Otherwise, the highest-value work
is the DEFERRED focused sessions below (content-MathML, document-builder).

> **2026-06-21 update — reruns IN PROGRESS, first cortex cross-check done.** A
> fresh Rust rerun (`019eea79…`) AND a fresh Perl rerun (started 03:51) are both
> live on `sandbox-arxiv-10k-shuffle`, so per-paper status is in flux (many show
> transient `todo`). A first cortex-grounded cross-check of the **`error/malformed`
> tail** (the richest vein for Rust-only document-builder bugs) — filtered to
> papers where BOTH services are terminal AND Perl lacks the exact `what` —
> surfaced **zero genuine Rust-only structural regressions**. Every apparent
> candidate is either still `todo` in the Perl rerun, or a paper where **Rust is
> at-or-better than Perl**: e.g. `0905.3143` Perl 101 errors→FATAL vs Rust 6
> errors/no-fatal; `1710.08311` Perl FATAL vs Rust survives. (Method script
> pattern: `reports/.../error/malformed/<what>` → per-paper
> `corpus/<c>/tex_to_html/document/<id>`, require Perl status ∈ terminal AND no
> `malformed/<what>` message.) **Re-run the clean full cross-join once both reruns
> COMPLETE** — only then is a Perl=`no_problem`/`warning` vs Rust=`error` signal
> trustworthy.

> **2026-06-21 (later) — reruns now COMPLETE; cross-join reopened.** Rust service
> `oxidized-tex-to-html` on `sandbox-arxiv-10k-shuffle` is 100 % terminal
> (todo=0); Perl `tex_to_html` is 99.77 % terminal (23/9849 `todo`). The
> small-category sweep (xpath/document/misdefined, fully enumerated + per-paper
> cross-checked against the live `document/<id>` API) found:
> - **`1506.09203` — STALE signal, already FIXED on current HEAD.** The cortex
>   DB shows Perl=`warning`, Rust=`error` (`error|xpath|findnodes|()` at
>   `xml.rs:46`), but that Rust status is from the rerun binary `019eea79`. A
>   local repro on current HEAD (`/data/arxiv/1506/1506.09203/`,
>   `Subrepresentation_book_6tag3.tex`, TCI/Scientific-Word + `tcilatex.tex`,
>   ar5iv profile) converts **clean: 0 errors / 0 fatals, no xpath failure, 52
>   warnings** — matching Perl. An intervening branch commit (after the rerun
>   snapshot) resolved the eqnarray/MathFork `findnodes` invalid-context failure.
>   **Lesson reaffirmed: always re-confirm a flagged paper on the CURRENT binary
>   before chasing.** Landed regardless: `xml.rs` `findnodes`/`findvalues` now
>   include the failing XPath string + context-node presence in the error (the
>   old message was just `{:?}` → empty `()`), so any future xpath failure is
>   diagnosable.
> - `0803.1344` (document/open_element_internal): Perl `fatal` vs Rust `error` →
>   Rust at-or-better, not a regression.
> - `1608.07271`, `1802.04240` (misdefined `#`), `hep-th9207093`
>   (misdefined `\list`): Perl=`error` = Rust=`error` → parity (shared cause).

> **2026-06-21 (later still) — the existing rerun (`019eea79`) is now STALE; a
> NEW rerun is required before further mining.** The Rust `oxidized-tex-to-html`
> error data predates this session's fixes (m{}/b{} `\multicolumn`, dcolumn
> empty-todelim, the over-parse/grammar work, etc.), so per-`what` mining keeps
> surfacing already-fixed leads. This iteration checked the highest-cascade
> `error/latex` clusters and ALL were stale/parity/Perl-worse on the CURRENT
> binary: `(newunicodechar)` 1704.05587 (cortex "ASCII character requested" ×63 →
> now PARITY: `\newunicodechar` simply undefined in both, 22=22 identical);
> `(etoolbox)` 1604.02419 (cortex Rust=error but Perl=**fatal** → Rust at-or-
> better); `(babel)` `Unknown option 'russian'` ×11 (witness 0709.3796 now
> Rust=0=Perl=0; minimal `[russian]{babel}` is Rust=1 / Perl=3, the option error
> emitted by BOTH → parity-or-better). **Do not mine `019eea79` further — request
> a fresh Rust rerun on current HEAD first** (needs `X-Cortex-Token`); only then
> is a Perl=clean / Rust=error signal trustworthy. Reliable interim method: a
> direct LOCAL both-engines diff on a small paper sample (ground truth, not the
> stale DB).
>
> **`1506.03557` (`ESSS_2015.tex`) — Rust 49 / Perl 2, PARTIALLY addressed
> (math session, 2026-06-21).** Two distinct roots:
> - **WIDE_PUNCT threshold — FIXED.** A fenced comma-list with an interword
>   control space `\ ` before a signed term (`(3,\ -5)`, `(300,\ -50,\ +50)`,
>   `\textit{Held\_For}\;(300,\ -50,\ +50)`) fell to `ltx_math_unparsed`: the `\ `
>   put 5.0pt `rpadding` on the comma, and `punct_followed_by_wide_space`'s ≥5pt
>   threshold mis-tagged it `WIDE_PUNCT` (a `\quad`-class formula-separator routed
>   through `formulae_apply`, which fails inside a fence). Raised the threshold to
>   ≥10pt (only `\quad`+; matches `filter_hints`). Now parses, matches Perl
>   `vector@(300,-50,+50)`. Regression test in `parse/sequences_and_lists`.
> - **The 42× `XMWrap isn't allowed in <ltx:p>` residual is a WRAPPING leak
>   triggered by the `program` package — ROOT LOCALIZED 2026-06-21, still OPEN
>   (niche, deferred).** Bisection: the 42 leaks come from 3 sections
>   (preliminaries=18, trip_sealin=12, pushbutton=12), and preamble bisection pins
>   the enabling factor to **`\usepackage{program}`** (commenting it → 0 leaks).
>   `program.sty` makes `_`/`;`/`` ` `` ACTIVE in math (`\catcode\_=\active
>   \def_{\ifmmode\sb\else\p@sb\fi}`, lines 535/67-75) and redefines `\(`; the
>   preliminaries math is subscript-heavy (`t_n`, `t_{now}`, …), so under the
>   active-`_` Rust produces unparsed inline math whose bare `<XMWrap>` leaks into
>   `<ltx:p>` while Perl (which has NO program.sty.ltxml — it raw-loads) keeps it
>   `<Math>`-wrapped. Rust loads `program` via the **contrib binding**
>   (`latexml_contrib/src/program_sty.rs`), so the divergence is contrib-binding
>   vs Perl-raw-load. NOT reproducible from `program` + the snippet alone — needs
>   the full preliminaries context (accumulated state). Both the unparsed Z-math
>   AND the leak are recovered in the final output; these are build-time errors.
>   Niche (`program` is rare on arXiv); for a future contrib-binding session —
>   fix in `program_sty.rs` (match Perl's raw-load active-`_` behavior) and/or the
>   document-builder unparsed-math wrapping. The WIDE_PUNCT fix above was the
>   general, landable win from this witness. (Same scan: `1705.04022`
> 16 err `_`/`^`-in-text — re-verify vs Perl before chasing.)
>
> **`1704.05644` (`Paperling_revu.tex`) — CONFIRMED Rust-only (Rust 17 / Perl 0)
> but DEEP/tangled; deferred.** Root: `shadethm.sty` (raw-loaded, no binding in
> either engine) fails to define `\newshadetheorem` in Rust in this paper's
> context → cascade of undefined `{theorem}`/`{hyp}`/`{propgrise}` envs +
> `\shadebox*`/`\shadedtextwidth` `expected:<variable>`. KEY: the *minimal*
> `\usepackage{shadethm}\newshadetheorem{thm}{Theorem}` is **parity-broken** (BOTH
> engines: `\newshadetheorem` undefined) — so shadethm's raw-load is incompletely
> emulated in both, and only the full paper's preamble context makes Perl's
> shadethm work while Rust's still fails. Not cheaply isolatable (bisection of the
> preamble/`\input{macropulko}` did not localize a single culprit; the apparent
> "`\input` breaks it" lead was a red herring — minimal no-`\input` is equally
> broken). The `\Vertex`/gastex errors in this paper are SHARED (gastex depends on
> pstricks/pst-pdf; both engines fail identically in isolation). A proper
> `shadethm` binding (which neither engine has) would be the real fix — surpass-
> Perl R&D, not strict parity. Do not chase piecemeal.

**Beyond-parity coverage (#2 track, surpass-Perl):**
- **`arximspdf`/`arxstspdf` (arXiv IMS journal classes, aop/aos/aap/aoas) — ✅
  LANDED 2026-06-27** (user-directed). New `arximspdf_cls.rs` binding (one binding
  serves both — identical `\b*` bib macros): loads `article`, defines the IMS
  macros, and PRESERVES frontmatter metadata via the standard `\lx@add@*` API
  (title/creator+personname/contact/keywords/abstract/date all emitted). Key
  pivots: arximspdf does NOT load amsmath (defines `\tfrac`/`\dfrac`/`\operatorname`
  itself) — so we load `amsopn` not full `amsmath`, whose env-form `\matrix`
  override otherwise broke the plain-TeX `\matrix{…\cr…}` the papers use (Perl is
  parity-broken there too); the structured `\b*` bib is PASSTHROUGH text (the
  `ltx:bib-*` vocabulary is schema-valid only in the BibTeX `bibentry` path, not in
  `\bibitem`'s `bibblock`); `{keyword}` uses the COLLECTING
  `\lx@begin@keywords`/`\lx@end@keywords` form (`\lx@add@keywords` alone clears on
  each call). Both engines previously FAILED outright (cascade) — Rust now
  SURPASSES Perl: aop632 (0910.0069) 28→**1** vs Perl 17; the 16-paper aop/aos
  cluster all convert (1–11 errors, metadata preserved). Suite 1479/0; regression
  `cluster_arximspdf_imsart`. (Residual: the unusual `\ead`-inside-`\author`
  nesting leaves 1 frame-balance artifact; email still captured as a contact.)
- Remaining candidates: `jpconf` class → iopart (18+ IOP-conf papers);
  theorem/mdframed-in-figure schema (`figure_mixed_content`, Open task §1).

---

## Math-parser / content-MathML gaps — DEFERRED to a dedicated session

> **User directive 2026-06-20: defer ALL content-MathML items to a dedicated
> session** (the math parser is a full Marpa-vs-RecDescent rewrite; these touch
> the parse-tree / content-MathML structure and want a focused regression
> budget). Notes kept here; do NOT pick at them piecemeal.

- **`f(a,b)` multi-arg flattening — FIXED 2026-06-22.** A KNOWN function applied
  to a paren comma-list now flattens: `\max(a,b)`→`maximum@(a,b)` (was
  `maximum@(vector@(a,b))`), matching Perl `ApplyDelimited`/`extract_separators`.
  Implementation was simpler than the planned grammar-rule approach: a post-parse
  spread in the `prefix_apply` ACTION (`semantics.rs`, helper `vector_tuple_items`)
  — when a function-role op (FUNCTION/OPFUNCTION/TRIGFUNCTION) applies to a
  `Dual` whose content is `Apply(vector, [refs])`, spread the items as direct
  operands instead of wrapping. No grammar/pruning change → NOT pruning-sensitive,
  zero fixture regressions. Scoped to known function roles, so unknown-`f` apply
  (`f(a,b)`→`f@(vector@(a,b))`) is untouched — the intentional divergence #18.
  Verified Perl-identical: `\max(a,b)`/`\gcd(a,b)`/`\min(x,y,z)`/`g(a,b,c)` +
  nesting/`\frac`/trailing-ops; suite 1466/0; regression test in
  `parse/functions`. (Known pre-existing aside: juxtaposed `\max(a,b)\min(c,d)`
  greedily reads `\max` over the product — a separate function-juxtaposition
  pruning issue, not this flatten.)
- **`f(x)` single-arg apply-vs-multiply** (most PERVASIVE divergence): for an
  UNKNOWN/undeclared symbol + paren arg, Rust reads *application*, Perl reads
  *multiplication* — `\Gamma(s)`→Rust `Gamma@(s)` vs Perl `Gamma * s` (likewise
  `\zeta(s)`, `\Phi(x)`, `f(x)`). A real fix must respect Perl's "only declared
  FUNCTION/known-operator names apply; bare letters multiply" rule; heavily
  pruning-sensitive.
  > **SURVEY 2026-06-22 (current-state + blast radius — groundwork, NOT yet
  > changed):** confirmed the split cleanly — KNOWN functions ALREADY match Perl
  > (`\sin(x)`/`\log(x)` → `sine@(x)`/`logarithm@(x)` in both); only UNKNOWN
  > symbols diverge (`f(x)`/`g(x)`/`P(x)`/`\Gamma(s)`/`\zeta(s)`/`\phi(x)` →
  > Rust `X@(x)` vs Perl `X * x`; `f(x+1)` → Rust `f@(x+1)` vs Perl `f * (x+1)`).
  > LEXER ROLE: unknown `f` = `role="UNKNOWN"`, `\max` = `role="OPFUNCTION"` — so
  > the apply-of-UNKNOWN (A) is separable from the known-fn flatten (B). BLAST
  > RADIUS of A is corpus-wide: 25 test fixtures, ~150 single-letter applies
  > (`f@(`×57, `d@(`×51, `g@(`×13, …) would flip to multiply — a sweeping change
  > that reshapes all math output. Because A is corpus-wide (even though
  > toward-Perl), it needs explicit scope sign-off before undertaking; B (below)
  > is the contained first step (~5 fixtures).
- **`[a|b]` / `[a \mid b]` bracket-conditional — FIXED 2026-06-22.** Was unparsed
  in Rust; now `delimited-[]@(conditional@(a,b))` matching Perl (`E[X|Y]` etc.).
  Root: the bare `a|b` conditional reduces only at statement level (not as an
  `expression`), so `[a|b]` had no fence rule — though `[(a|b)]` already worked.
  Fix: a surgical grammar rule `lbracket formula singlevertbar formula rbracket =>
  bracket_conditional` (`singlevertbar` also covers `\mid`) + a `bracket_conditional`
  action (semantics.rs) that builds the inner `conditional@(a,b)` (delimiter-less
  presentation) and wraps it in `delimited-[]` via the same `fenced` path
  `[(a|b)]` uses (ctxt reborrow for the two ref levels). Suite 1466/0, clippy
  clean, zero other-fixture changes; regression test in `parse/vertbars`. (The
  `E` in `E[X|Y]` stays `E@(…)` apply vs Perl `E * …` — divergence #18, preserved.)
- **`⁡` DecorateOperator over-insertion — FIXED 2026-06-22.** Presentation MathML
  emitted `⁡` (U+2061 FUNCTION APPLICATION) after operators that render as
  `<m:mo>` — `\nabla \phi`→`∇⁡ϕ`, `\partial f`→`∂⁡f`, and (pre-existing) `\sum_i
  a_i`→`∑⁡a_i`, `\int f`→`∫⁡f` — where Perl juxtaposes (∇ϕ/∂f/∑a/∫f). Perl's rule
  (MathML.pm `Apply:?:?`): insert `⁡` only when the op base is NOT an `<m:mo>` (a
  function identifier `f`/`\sin`/`\max` IS `<m:mi>` → keeps `⁡`). FIX
  (`latexml_post/.../presentation.rs`): new `op_base_is_mo` helper (descends
  msub/msup/munder/mover to the base); applied at the generic-apply site AND in
  `pmml_summation`; and removed `DIFFOP` from the big-op→`pmml_summation` route
  (Perl MathML.pm:702 `# Not DIFFOP`). Suite 1466/0, clippy clean; verified
  Perl-identical for ∇/∂/∑/∫/∏/⋃/lim + `\sin`/`\max`/scripted forms; only residual
  diff is the `f(x)` apply-vs-multiply (`f⁡(` vs `f⁢(`) — divergence #18,
  preserved. Regression test in `tests/post/opdecoration`.
- **wide-space PUNCT XMDual content-arm XMRef ordering**: `x^2\quad y` — the
  `\quad` (≥10pt) becomes a virtual PUNCT through `formulae_apply`, producing an
  XMDual whose content-arm XMRef siblings emit one slot off from Perl. Same
  MathFork/split content-arm xml:id family as the `expected:id` tail
  (`EXPECTED_ID_XMREF_DESIGN_2026-06-08.md`). NOT the rpadding path (thin spaces
  `\,` are Perl-faithful incl. NewScript transfer, `005716ff66`).
- **`\DeclareMathOperator` cluster — INVESTIGATED 2026-06-22, LOW-VALUE metadata,
  deprioritized** (`text=` and cMML already match): (a) Perl splits Math attrs
  `tex="\operatorname{Tr}…"` vs `content-tex="\Tr…"` (Perl defines `\Tr` *via*
  `Invocation(\operatorname,…)` + `revert_as=>'context'`); Rust defines it
  directly so `tex` keeps the user macro `\Tr` (arguably MORE source-faithful) and
  emits no `content-tex`. Matching Perl needs the deep `revert_as=>context`
  content-tex mechanism — high effort, metadata-only value. (b) The `name="Tr"`
  "gap" is NOT a bug: `def_math` (dialect.rs:1567) DOES infer `name` from the CS
  but DROPS it when `name == presentation` (line ~33) — a deliberate
  redundant-attr cleanup. `\Tr` (name "Tr" == content "Tr") drops it; `\argmax`
  (name ≠ "arg max") keeps it. Perl always emits it. Changing this touches the
  GENERAL def_math path (every math token) for cosmetic value → not worth it.
  (c) `\DeclareMathOperator*` `scriptpos` in display mode — the remaining
  candidate if revisited, but mode-dependent and niche. Whole cluster parked.
- **N-ary bare-operator listing — ✅ NOW WORKS (verified 2026-06-27); note was
  STALE.** `+,-,\times,\div` → `list@(+,-,*,/)` (Perl-exact); `+,-`, `+,+`, `a,+,b`,
  `++`, `+-` all parse and match Perl. An intervening fix (likely the comma-list /
  marpa-drain work) closed this. NOT an open gap anymore. The truly-remaining
  operator-script cases are narrower and finicky/context-dependent: `\Omega_{+,+-}`
  (a comma-list-of-operators in a SUBSCRIPT — Perl's subscript grammar parses it as
  `list@(+, absent + -)`, Rust's doesn't; note `+,+-` STANDALONE is PARITY-unparsed
  in BOTH), and operator-scripts where both parse but DIVERGE structurally
  (`a^{++}`: Rust `a^(list@(+,+))` vs Perl `a^(absent + +)`). These are the deferred
  math-fork session (subscript-content grammar + scripted-operator structure).
- **comma-list LEFT of a relation `a,b \in A` — FIXED 2026-06-22 (2-item path).**
  Was the wrong `formulae@(a, b∈A)` (∈ binding only `b`). Now the user-specified
  surpass-Perl **XMDual**: content **DISTRIBUTES** — `formulae@(∈(a,A), ∈(b,A))`,
  sharing XMRefs to the relop and RHS — presentation wraps the list as the
  relation's LHS — `Apply(∈, XMWrap(a,',',b), A)`. Implemented as a scoped
  transform at the end of `formulae_apply` (semantics.rs): when `left` is a bare
  (non-relational, non-Dual) item and `right` is a binary RELOP relation
  `Apply(R,[lhs,rhs])` under a comma, `distribute_list_relation` builds the dual.
  `x,y \le z`→`formulae@(x≤z, y≤z)`. The list-RIGHT `0<x,y`→`list@(0<x,y)`,
  all-relational `a=b,c=d`→`formulae@`, and bare `a,b`→`list@` all stay. Full suite
  1466/0, clippy clean, zero other-fixture changes; regression test in
  `parse/relations`. **Remaining (follow-up):** the 3+-item `a,b,c \in S` goes
  through `list_apply` (not `formulae_apply`) → still `list@(a,b,c∈S)`; the same
  distribution needs porting to that path.
- **relation with a list-RHS that itself contains a scripted relop**:
  `a \le b \quad \stackrel{?}{\ge} \quad c` → Perl `a <= list@(b, >=^?, c)`.
  **UPDATED 2026-06-27: no longer `ltx_math_unparsed` (stale)** — Rust now PARSES
  it as `fragments@(a <= b, >= ^ ?, c)` (the `\quad`-WIDE_PUNCT routes it through
  `formulae_apply`→`fragments@` rather than the relation-with-list-RHS shape). So
  it's now a STRUCTURAL divergence (fragments@ vs `a <= list@(…)`), not a parse
  failure. Lower-severity (renders) cMML-structure item; the scripted-relop atomic
  fix (`4a5ebf29f7`) cleared standalone list items.
- **`\underset`/`\overset` over an ARROW with a multi-token script**:
  `x \underset{n\to\infty}{\to} y` — the under-script reads `n@to@infinity`
  (apply) where Perl groups `(n to infinity)`. Same ARROW-as-applied-function
  family as `f(a,b)`.

CAUTION: new VERTBAR/fence grammar rules can collide with package-built
structures — always cross-check the affected fixture against Perl before
assuming a regression (the norm rule "regressed" physics_test, but Perl matched
the new output, so it was a parity *fix*).

## DefMathRewrite `\WildCard` subscript bug (focused-session item)

`DefMathRewrite` with a `\WildCard` SUBSCRIPT pattern doesn't demote the match
(witness `math/simplemath`): `f_\WildCard → role=ID` should make `f_1(a+b)` =
`f _ 1 * (a+b)` (Perl), but Rust produces `Unknown@() * (a + b)` — the
`f_\WildCard` rewrite isn't firing (or loses to the sibling `f → FUNCTION`
rewrite), so `f_1` stays a FUNCTION and gets APPLIED. The non-wildcard
`f_D → DIFFOP` works, so it's the `_\WildCard`-subscript match/ordering in
`latexml_package/.../latexml_sty.rs` (`compile_declare_pattern`). Niche
(binding-author feature, rare in real arXiv); the fixture encodes the buggy
output.

---

## Open tasks (actionable)

### mhchem-manual fidelity mission (2026-06-27, on `followups-2026-06-27`) — LANDED
Driven by a manual review of `~/Downloads/mhchem.tex` (the mhchem package manual)
rendered with `--preload=ar5iv.sty --css=ar5iv.css --nodefaultresources
--path=~/git/ar5iv-css/css` (glowup branch), examined via playwright + Chrome.

1. **7 new `latexml_contrib` package bindings** for the manual's missing packages
   (errors 10→0): `fancyvrb-ex`, `rsphrase`, `hpstatement`, `tgpagella`,
   `sourcecodepro`, `AlegreyaSans` (raw-load real `.sty` where installed, per the
   user directive that raw-loading `.sty` is encouraged; fonts no-op where absent),
   and `scrreprt` (OmniBus `.cls` stub like `scrbook_cls`, + `\minisec`/`addmargin`/
   `\addtokomafont`). Perl ships no binding for any of these, so they are surpass-Perl
   contrib additions. `pstricks` already bound (its warning is a transitive
   fancyvrb-ex dep-scan artifact when the raw `pstricks.sty` is absent — benign).
2. **`\marginpar` font-leak fix** (`latex_constructs.rs`, `bounded => true`) — the
   manual's `\marginpar{\Large !}` leaked `\Large` document-wide (1388 `144%` nodes →
   4). PARITY bug (Perl 0.8.8 leaks identically); fixed surpass-Perl. OXIDIZED_DESIGN
   #39, KNOWN_PERL_ERRORS #38. Output-neutral (suite 1487/0).
3. **mhchem stub RETIRED → raw-load real `mhchem.sty`.** The engine's expl3/xparse/
   chemgreek support is now mature enough that `\usepackage{mhchem}` raw-loads the
   genuine package: chemistry renders with proper digit subscripts (`\ce{H2O}`→H₂O),
   charge superscripts, reaction arrows (`->`/`<=>`/`->[..]`), bonds, states,
   `\cesplit`. Simple `\ce` is 0 errors + correctly formatted (the old stub rendered
   formulae FLAT). chemformula stub updated to require mhchem with `version=4` (the
   real package warns without it; the old stub was silent). **Residual:** the full
   manual still emits ~69 edge-case errors under raw-load (`\ce` inside `align*` →
   `\lx@begin@alignment`/`\end@amsalign`; ~56 `\lx@end@inline@math` from specific
   `$`-toggle / `\cesplit`-derived example patterns). Basic `SideBySideExample`+`\ce`
   is clean. **TODO (this branch):** debug the align*/`\lx@end@inline@math` edge
   cases toward 0 errors; validate the corpus mhchem witnesses via cortex (the flip
   is corpus-wide).

### `ltx_env_<name>` env-markup class — PLANNED, separate branch (churns every test XML)
**User-requested generic enhancement** (2026-06-27): tag environment wrapper markup
with `class="ltx_env_<name>"` so custom/minipage-like envs (e.g. `SideBySideExample`)
become responsively styleable in CSS instead of fixed-width minipages. **MUST be on a
dedicated branch** — it changes nearly every test XML (additive class on every env
element), so the golden-suite update is large and must be done in isolation.
Two implementations, same markup outcome:
- **Binding side (`DefEnvironment!`):** the constructor guarantees exactly one element,
  so unconditionally add `ltx_env_<name>` (via an `@ADDCLASS`/`add_class` after the
  begin constructor opens). Applies to ALL DefEnvironments (`figure`, `table`,
  `theorem`, `minipage`, …) — user chose full scope.
- **Raw side (`\newenvironment`/`\renewenvironment`):** arm at env start; at `\begin`
  construction record `{name, anchor = globally-unique gid of current node, mark}`; at
  `\end` afterConstruct, if EXACTLY ONE element was deposited under the anchor since
  the mark → tag it; zero (font/text-only) or >1 (siblings, e.g. SideBySideExample's
  parboxes) → nothing. **Needs a globally-unique monotonic node gid** (verify/ add;
  `record_node_ids` exists but is xml:id-oriented).
- **SideBySideExample:** keep the working `fancyvrb-ex` raw-load (correct source+result)
  + drive responsive layout from the resulting `ltx_minipage`/`ltx_env_*` hooks in
  `ar5iv.css`; do NOT re-implement the verbatim+render dual capture.

### 1. `ERROR_DEBT` test-gate drain — ✅ DRAINED 2026-06-27 (now empty)
The harness error-gate (`latexml_oxide/src/util/test.rs`) fails a test at zero
debt to force removal once fixed.
- **`figure_mixed_content`** — ✅ FIXED: `ltx:theorem`/`ltx:proof` were rejected in
  `ltx:figure`/`ltx:table`/`ltx:float` (both engines errored — parity). A boxed
  theorem/proof inside a float is valid LaTeX, so expanded the schema model
  (`resources/RelaxNG/LaTeXML.model` + `LaTeXML-para.{rng,rnc}`: added `theorem`,
  `proof` to the three float content models). **Output-neutral** (the builder already
  placed the theorem inside the figure; only the spurious malformed-error is gone —
  golden XML byte-identical). Suite 1481/0; `ERROR_DEBT` is now empty. Surpass-Perl,
  monotonic (strictly more permissive — cannot invalidate any prior-valid doc).
  See OXIDIZED_DESIGN #38.

### 2. `\gls`/`\acrshort` in MATH mode (1705.10306) — RE-CLASSIFIED 2026-06-27: almost certainly PARITY (source-confirmed), blocked on unrunnable Perl
293 errors `ltx:XMTok isn't allowed in <ltx:glossaryref>`: a glossary command in
math mode digests the link display text (#3, the literal acronym term) as math →
bare per-letter `<XMTok>`, which the `glossaryref` content model rejects.
**Source-confirmed 2026-06-27 that this is most likely PARITY (NOT a Rust-only
gap — the cortex "Perl 1" is stale/unreliable, per `use-cortex-for-parity-work`):**
- Perl `Stomach.pm::enterHorizontal` (L422-434) is a **no-op in math** (`$mode
  =~ /math$/ => {}`) — Rust's `enter_horizontal` matches faithfully. So the
  `enterHorizontal => 1` on the shared `\lx@glossaries@gls@link` constructor does
  NOT switch #3 to text in math in EITHER engine.
- BOTH engines raw-load the SAME `glossaries.sty` (`InputDefinitions(noltxml=>1)`)
  with the SAME override constructor → both digest #3 in the ambient math mode →
  both produce `glossaryref > XMTok` → both hit the same schema rejection.
- `\ref`/`\cite` in math do NOT error (verified) — their content is STRUCTURED
  (bibref / ref-number), not a literal term; only `\gls`/`\acrshort` emit raw
  letter-XMToks. So glossaryref is specific, but the mechanism is shared with Perl.
- **The earlier "Perl raw-loads glossaries.sty and typesets as TEXT" hypothesis is
  weakened:** Rust raw-loads the identical `.sty`, so if it typeset the term as
  text, Rust would too. It doesn't (output: italic letter-XMToks) → so the `.sty`
  display chain does NOT force text in math.
**Perl confirmed UNRUNNABLE here (2026-06-27):** `latexml glx.tex` → `Fatal:terminate`
in `expl3-code.tex` (l3kernel) at 150 s — glossaries pulls in expl3 which is
pathologically slow in Perl 0.8.8 on this host; cannot capture ground truth.
**Fixing is therefore deferred as a likely non-bug.** If pursued, it parallels the
figure_mixed_content surpass-Perl pattern (a monotonic schema expansion to accept
the math content the builder already produces) — BUT the correct structure is
genuinely uncertain without Perl (XMTok directly? XMText-wrapped? operator-token
for the `\DeclareMathOperator` case? text PCDATA?), and there is **no precedent**
for `XMTok` in any inline element's model, so a speculative change risks an
unfaithful divergence. Repro + full notes:
`docs/reproducers/glossaryref_math_xmtok.tex`.

### 3. PR #248 B1 — re-entrant `&mut Document` round-trip (runtime-bindings) — ✅ RESOLVED 2026-06-27 (verified SOUND, was a misanalysis)
The Rhai constructor trampoline re-mints `&mut Document` from a thread-local
`*mut` for a nested `\wrap{\myemph{..}}` construct. The earlier B1 review feared
this was Stacked/Tree-Borrows **aliasing UB**; a careful reborrow analysis shows
it is **sound** — the nested pointer is a reborrow **descendant** of the outer
one (the core threads a reborrow of `absorb`'s `&mut self` down to the nested
constructor via `be_absorbed(self)`), and `with_doc` always re-mints from the
**innermost** published pointer (`CTOR_CTX` is a stack), so every re-mint is a
genuine descendant of all parked outer `&mut`s — a descendant reborrow never
invalidates its ancestors. **VERIFIED:** the exact pattern (thread-local `*mut`
stack + RAII guard + `with_doc` re-mint + nested `absorb` reborrowing down) is
modeled libxml2-free in `latexml_core::runtime_bindings_reentrancy_model` and
passes **Miri under both Stacked and Tree Borrows, 0 UB** (the real path is
libxml2/FFI, which Miri can't execute — hence the model). `tools/miri_check.sh`
runs it (stacked + tree) in CI. The checked-guard "fix" was correctly rejected:
there is no UB to guard, and it would deadlock `Document::absorb`'s loop (which
needs the nested construction to SUCCEED). No architectural change needed; the
single audited `with_doc` `unsafe` stays, now documented as verified-sound.
`runtime-bindings` stays on by default. **Sibling site audited too (2026-06-27):**
the `WHATSIT_CTX` re-mint (`engine.rs` `setProperty` `&mut *ptr`; `argString`/
`propertyString` are read-only `&*`) is sound — after-digest hooks run one-pass/
sequentially on a fresh-local whatsit (`definition.rs::execute_after_digest`) and
never re-enter on the SAME whatsit, so it's always the single-body re-mint pattern
the Miri model already covers. The runtime-bindings unsafe re-mint sites are now
fully audited.

### 4. 0.7.0 release — release-prep LANDED; tag pending
Version bumped, `runtime-bindings` in the artifact, `.deb` deps, CHANGELOG/README
done. **Remaining:** tag `0.7.0` on `main` → `release.yml` runs the TL-window
`dumps` + macOS arm64 leg + publish (each first-exercised on that tag).

### 5–6. LANDED 2026-06-22 (see "Landed this session" above)
- **Post-processing log parity** (`512dbc1ba2`, `9524d2e179`): `cortex.log` carries
  core+post. **Residual (cortex-side owner):** wire `cortex_worker.rs::convert_archive`
  to `run_post_processing_logged` + fold `max(core, post.status_code)` into
  `Status:conversion` (Perl `LaTeXML.pm` L631-634).
- **Graphics never ships a raw `.eps`/`.pdf`** (`80b4438385`, `604951c232`): three
  guards → a `<graphics>` without `@imagesrc` renders `ltx_missing_image`. Known
  post-orchestration deltas (not blocking, broader parity): `PictureImages` absent
  (Rust = regex inline-SVG), `SVG` regex extractor, no `prescan`.

---

## Deep deferred families (parked — large or shared; dedicated sessions)

- **`Fatal:Stomach:Recursion` (43 cortex Rust-service fatals) — TRIAGED 2026-06-28,
  mostly SHARED / Rust-better; ~1 Rust-only over-fatal DEFERRED (deep core).** Two
  guards in `stomach.rs`: the box-cycle "Infinite digestion loop" (9 papers,
  stomach.rs:1040) and the token-stack-depth "Excessive recursion(?)" (28 pkg-loading
  + 6 box/thm, stomach.rs:1343, `MAXSTACK=200`). **Same-host Perl parity on an 11-paper
  sample: ~10/11 SHARED** — the box-cycle/digloop papers (1906.06902, 1810.02304,
  1911.00254, 1911.11563, 2605.27339) **HANG in Perl 50–94 s** while Rust fail-fasts in
  <1 s via the guard (**Rust strictly better**); others (1809.00641, 2103.12717,
  1409.4048, 2011.08422) fail in BOTH. Only **1804.01117 (svjour3) is genuine Rust-only**
  (Perl completes in 32 s; Rust hits the token-stack guard). Crucially the limit
  **matches Perl exactly** (`Stomach.pm:159 $MAXSTACK=200`, identical guard at L175) —
  so it is NOT a mis-set cap; do NOT raise `MAXSTACK` (diverges from Perl and lets genuine
  infinite recursion run). The guard is doing its job — this category is a Rust **stability
  win**, not a bug cluster.
  **DEEP-DIVE of the lone Rust-only case 1804.01117 (2026-06-28): it is NOT a
  stomach-accounting bug — it is a tikz/pgf cascade.** Full stack capture: the top ~170
  frames are `{ \bgroup { \bgroup …` piled up by **`\pgffor@expand@list`** (pgffor's
  `\foreach`), immediately after `Error:pushback_limit:Timeout … loading binding for
  'tikz.sty'`. Rust fails to load the `tikz.sty` binding (pushback-limit), leaving
  `\foreach` in a broken state that floods the digestion stack → `Stomach:Recursion`;
  Perl loads tikz fine and never gets there. (The earlier "Rust digests packages deeper"
  hypothesis was WRONG.) Minimal `\usepackage{tikz}`, the full preamble package set, and
  `tikz`+`\foreach` in the body all load CLEANLY — the binding-load pushback only triggers
  under the paper's specific complex state (late in-body `\usepackage`/`\@iinput` mixed
  with nested tables/cases), so it is not minimally reducible. ⇒ The real root is the
  **known deep tikz/pgf binding-load lever**, not the stomach; the Stomach:Recursion
  category has **zero genuine stomach bugs**.

- **1610.00974 step-3 (global p{}→VBox) + cluster-B — ✅ LANDED 2026-06-22, NO
  LONGER DEFERRED.** See "Landed this session" above. p{}/m{}/b{} columns now build
  the cell as Perl's `\lx@tabular@p` inline-block (VBoxContents); p/m/b `<td>`
  `align="left"`; **cluster-B FULLY RESOLVED**; fixes 1510.07685. Commits
  `f65b80c1c2` / `eb978df5a9` / `1867f17da9` (+ box-model `7545e07fd6`). NOTE: the
  `collcell`/`\collectcell` undefined seen in some table papers is PARITY (both
  engines default `notex=1`/`INCLUDE_STYLES=false`, so neither raw-loads
  `collcell.sty`; the `--quiet` Perl "0 errors" was a display-suppression artifact —
  use verbose Perl).
- **`expected:id` cmml dangling-XMRef tail** — MathFork/split content-arm xml:id
  duplication; the last live `expected:id` class. See
  `EXPECTED_ID_XMREF_DESIGN_2026-06-08.md`. **★ CANONICAL WITNESS FIXED AT THE ROOT
  (2026-06-26q, LANDED on `class-b-xmref`):** the grammar rule `statements punct
  statement vertbar statements => vertbar_modifier_listlhs` makes a comma-list left
  of a conditional bar parse (`a,b|c` → `list@(a, conditional@(b,c))`, Perl-exact),
  so the witness's aligned `\Pr(s_A,s_B|\Omega)` arg parses → refs RESOLVE, dual
  PRESERVED. cb_repro & full witness `2311.01600` → 0 danglers; suite 1470/0; also
  fixes the standalone `a,b|c` aside. **RESIDUAL CHARACTERIZED (2026-06-26r):** the
  fix closed the "No node found"/DANGLING sub-case (canonical witness). The
  DOMINANT remaining `warning/expected/id` cortex cluster (**370 tasks**) is a
  DISTINCT class — `Missing idref on ltx:XMRef … _xmkey is `` ` (keyless XMRef, no
  idref, document.rs:3238), NOT a dangling idref — Rust-only (0704.2334 Rust 2 /
  Perl 0), from `\quad`/`\;`-separated **formulae/lists** with function-fence
  applies; context-dependent; root = `formulae_apply` content ref whose key never
  reaches the presentation item's top node (structure captured 2026-06-26t: a
  `formulae@` dual with a trailing bare `XMRef _xmkey=XM291` and no presentation
  top carrying XM291; the extend path doesn't clone `right`, so it's a subtler
  nested-relation/`\lx@dual` interaction). **SEVERITY: content-MathML QUALITY gap,
  NOT corruption** — the keyless ref has no idref so the prune sweep skips it; it
  survives with the faithful `Missing idref` Warn, schema-valid, no content dropped.
  Lower-priority cMML-polish item for the deferred math-fork session; the two
  higher-severity sub-classes (Class-B dangling + content-corruption) are FIXED.
  **★ COMMON SUB-CAUSE FIXED (2026-06-26v):** the keyless bare ref is a
  distribute-dual extend interaction — `distribute_list_relation` makes a
  `formulae`-content dual with a relation-`Apply` (non-Wrap) presentation; the
  formulae/list extend paths then push a content ref but silently skip the non-Wrap
  presentation → bare ref. Fix = gate the extend on a Wrap presentation (fall
  through to a fresh dual otherwise). Witnesses 0704.2334/0705.0790/0707.1173 →
  0 Missing-idref; suite 1471/0; regression `cluster_formulae_distribute_no_bare_ref`.
  PARTIAL: 0707.1339 still emits 2 (a different sub-cause). **QUANTIFIED 2026-06-22 (pre-fix): this WAS the
  #1 remaining Rust-only divergence** — `warning/expected/id` is **1005 cortex
  tasks** ("Cannot find a node with xml:id='S…E…m1.N'" from
  `latexml_math_parser/src/parser.rs:2840`; math-node ids, so genuinely the
  content-arm/MathFork XMRef cluster). It's a large Rust-only WARNING excess vs
  Perl (e.g. 0704.3530 Rust 152 vs Perl 9 warnings) — NOT parity. The prime
  candidate for the deferred content-MathML dedicated session; do NOT pick at it
  piecemeal (user directive). **FULLY DIAGNOSED + DE-RISKED 2026-06-26** (branch
  `class-b-xmref`, research-only, no code): same-host confirmed (0803.3810 Rust 51
  vs Perl 0), exact 6-dangler witness `2311.01600` (now `/data/arxiv/2311/`),
  Perl's target tree captured, a ~15s repro, and ALL peripheral fixes (clone/move/
  `.mf`/combos) empirically RULED OUT — the sole fix is the core post-parse
  preserving the structural XMArg ids (it rebuilds a fresh result tree → fresh
  per-row `{group}X.m1.*` ids, stranding the build-time `{group}.m1.*` refs). The
  re-id is in a distributed parse/install path (the `parser.rs:1354` reinstall is
  NOT it). **PIN SHARPENED 2026-06-26 (notes 2026-06-26i/j) — full end-to-end
  runtime trace; exact unrecord site identified by backtrace.** The danglers are
  the `\Pr` (physics-pkg `I_dual`) CONTENT-arm arg refs; the arg material is still
  present (ref merely dangles → any prune/drop is content loss, RULED OUT as a
  cheat). The arg XMArg (`_xmkey="1"`, `xml:id`) is **swallowed by the
  `parse_single` reparse of its ancestor presentation XMWrap** (`unrecord_node_ids`
  ← `parser.rs:1501`), NOT parse_rec'd standalone — so the working `parse_rec`
  id-transfer (`:1136-1196`, which heals the sibling dual args keys 2,3,5,6,7,8)
  never applies. RULED OUT (all empirically): prune/drop, `XProps` xml:id capture
  (dual not ingested via `From<&Node>`), `_xmkey` re-resolution + remap (parser
  REGENERATES keys; `XM::Arg` drops the build key). LANDMINE: the reparse
  orphan-detection (`:1502-1528`) is dead-code via the `@xml:id` namespace footgun;
  naively fixing it ACTIVATES a content-losing `__LOSTNODE__` drop. Two viable fix
  designs (key-carrying `XM::Arg` + re-point handler; OR cross-recursion old↔new
  `_xmkey` snapshot) with failure modes in the design doc. **DEFINITIVE ROOT
  (2026-06-26k, proven vs Perl source):** the ASF-vs-RecDescent node-identity
  divergence — Perl `parse_rec` returns an array-tree EMBEDDING the real parsed
  child nodes, so `appendTree` preserves their `xml:id`; Rust's ASF `into_xmath`
  REBUILDS fresh nodes (XM::Apply), so a re-materialized (non-`XM::Lexeme`)
  referenced target loses its id and the content XMRef strands. Faithful fix =
  identity-preserving `into_xmath` for non-leaf referenced nodes (reuse the input
  DOM node, like the leaf `XM::Lexeme` arm); LOSTNODES re-point is the pragmatic
  alternative. **TRIGGER ISOLATED (2026-06-26l):** the dangler is a downstream
  symptom of a CONTEXT-DEPENDENT **parse FAILURE** of the `\Pr` argument
  (`s_A,s_B|Ω_{len=k}` → `parse_single` returns `None`), so the `parse_rec` id-transfer
  (which heals the args that DO parse) never runs and the ancestor reparse strands the
  ref. Confirmed: the SAME arg parses standalone (0 danglers) — only the paper's
  preamble makes it fail in-context. Two fix axes (both dedicated-session): (A)
  parse-coverage (make the in-context arg parse; relates to the open VERTBAR/comma-list
  asides); (B) failure-robust id preservation via reused-leaf correspondence
  (`record_replacement(oldXMArgId, newTopId)` re-point, content-preserving). Precise
  repro + ruled-out approaches in `EXPECTED_ID_XMREF_DESIGN_2026-06-08.md`
  (2026-06-26a–o). The dedicated session = fix axis A or B + full math-fixture/corpus
  validation. **PARTIAL FIX LANDED (2026-06-26o, `class-b-xmref`):** an
  operand-protection guard in `prune_dangling_split_xmrefs` stops the broad `^S\d+`
  sweep from DROPPING `\Pr` content-arm arg refs (which emitted a malformed
  `apply(probability)` = silent content loss for section-numbered aligned `\Pr`);
  it now PRESERVES the arg (dangling, closer to Perl). 1469/0, clippy clean, does
  NOT re-flood wp3, regression test `cluster_xmref_pr_arg_not_dropped`. Does NOT
  make refs resolve — that is still the dedicated session (the leaf-LCA re-point,
  design B, works mechanically but collapses the dual; the faithful fix needs a
  CONTENT-branch arg copy, Perl's `.mf` scheme, via `rearrange_lone_ams_aligned`).
  **ROOT CAUSE + EXACT FIX FOUND (2026-06-26p) — AXIS A now recommended.** Bisected:
  only `\Pr(a,b|c)` (comma-list-LHS conditional) dangles; `\Pr(x)/\Pr(a|b)/\Pr(a,b)`
  resolve. The grammar's lone VERTBAR-modifier rule is `statement vertbar statements`
  (single LHS, `builder.rs:447`), so `a,b|c` doesn't parse → arg fails → ref strands.
  ONE-LINE fix `statements vertbar statements` TESTED: standalone `a,b|c` parses
  (fixes the open VERTBAR aside), witness → 0 danglers, refs **RESOLVE**, dual
  PRESERVED (faithful, = Perl's path). BUT regresses abs-value (`a|a|` →
  `conditional@(a,a)` not `a*|a|`; abs-value-vs-conditional ambiguity defeats
  `prefer_fewer_conditionals`). Reverted. Targeted fix = a `comma_statements`
  nonterminal (≥1 comma, not subsumed by `statements`) so the rule fires only on
  genuine lists, OR a pruning tweak — dedicated math-parser session. Axis A produces
  the genuinely-correct tree; preferred over the deep rearrange materialization.
- **xy-pic `svg:path` / curve cluster** (1501.03690) — shifted-arrows `svg:path`
  in `ltx:text`; mode-frame cascade root.

**SHARED (both engines fail — match Perl; do NOT "fix" by downgrading):**
- **1804.01117 xint raw-load** — both raw-load xint and fail (plain: both stub,
  byte-identical). The Rust stack-overflow crash is FIXED (gullet `stack_guard`,
  configurable via `latexml_core::stack_guard`). Deep xint emulation parked.
- **mode-frame auto-close cluster** (1611.04940, 2009.05630, 1702.06692,
  1702.02037) — a theorem env opened via its bare begin-command with no matching
  `\end…` leaks the mode-switch frame; Perl `Stomach.pm:343-376` errors
  identically. A graceful auto-close would *surpass* Perl (beyond-parity R&D).

---

## Reference (stable — not active work)

### Engine file open gaps (MINOR, demand-driven)
- `tex_box.rs` box-dimension edges; `tex_fonts.rs` `\fontdimen` array + per-font
  `\hyphenchar`; `tex_tables.rs` padding CSS (XSLT concern).
- **Document-builder block/paragraph auto-wrap of inline content** (core,
  broad/risky family — two witnesses):
  - **`\fcolorbox` inline paragraph-grouping**: an inline `\fcolorbox`
    mid-paragraph — Perl breaks the `<p>` (its `internal_vertical` block ends
    it), Rust keeps it inline. SAME flags on both; Rust's inline reading
    arguably matches real LaTeX's `\mbox`-based `\fcolorbox`. (`\colorbox`
    matches.)
  - **bare `\includegraphics` run in a figure** (witness 1108.0198, found
    2026-06-21 via skeleton diff — a clean, error-free reproducer): a
    `\begin{figure*}` with several consecutive `\includegraphics` (no blank
    line) — Perl wraps the inline run in a `<ltx:block>` (`figure > tags >
    block > graphics×N`), Rust emits the graphics bare (`figure > graphics×N`).
    Rust is error-clean and schema-valid. **Re-witnessed + root-confirmed
    2026-06-27** (0704.0001, 0704.0017 via the corrected structural diff): NOT
    merely cosmetic — the panel `<graphics>` WIDTHS also diverge (Rust 303.5pt vs
    Perl 241.5pt, ~1.257×), so figure sizing is visibly affected. Root: Perl's
    `arrange_panels_and_breaks` (`latex_constructs.pool.ltxml:3229-3295`) does a
    full box-metric panel layout — it inserts `<break class="ltx_break">` and wraps
    panels using `getNodeBox($child)->getWidth` vs `float_width`; Rust's
    counterpart (`latex_constructs.rs:1784-1869`) is explicitly **"Simplified: mark
    panel children with the class"** and skips the break/block arrangement. A
    faithful port DEPENDS on matching box widths → the deep box session (sibling of
    the `\resizebox` panel-width item below), not a loop-tick fix.
- **`\resizebox` panel scale-VALUE divergence**: in `complex/figure_mixed_content`
  two panels get a different computed natural width (xscale 1.13 vs 0.88). The
  construct in ISOLATION matches exactly (both xscale=1.9685); the divergence
  only appears inside the paper's `\footnotesize` + `table*` + `\subfloat` panel
  context → a font-size/box-context interaction. Scale *formatting* (%.15g) is
  already Perl-faithful (`551c5286ba`); missing-image candidates too
  (`64dd30b284`). Deep box-metric; for the focused box session.
- **~72-CS Perl-only long tail** (from the archived LoadFormat audit): misc
  atomics (`\@charlb`, point-size CSes, `\batchmode`, …) Perl defines, Rust does
  not. Investigate a CS only when a real paper witnesses it; refresh the CS-name
  diff before quoting counts (predates the BibTeX port).

### Primitive layer — AUDITED FAITHFUL (2026-06-20)
Probe-based Rust-vs-Perl audit found the core primitive layer byte-identical
(arithmetic, dimensions, glue, conditionals, string/token, case tables). Don't
re-audit without a witnessing paper. Shared-with-Perl quirks (NOT Rust bugs):
`\numexpr` divideround round-half-toward-+∞ (KNOWN_PERL_ERRORS #33); `\the\skip`
drops stretch/shrink to bare pt.

### Permanent ignores
- **Out-of-scope**: ns1–ns5 (`52_namespace`, no DTD support); 2402.03300,
  2410.10068, 2511.03798 (Perl also fails).
- **Rust supersedes Perl**: `1207.6068`, `0909.3444`, + 40 more in
  `memory/project_rust_supersedes_perl.md`.
- **BibTeX**: `BibTeX.pool.ltxml` ported (Phases 1–8; remaining B1–B6 polish in
  `BIBTEX_PORT_PLAN.md`). `--nobibtex` is opt-out, not default.

### Tikz known diffs vs Perl
`foreignObject` transform; arrow-tip path data; SVG viewBox/width; matrix
`<svg:g class="ltx_tikzmatrix">` vs inline-blocks; **bare `svg:g` in `<ltx:block>`**
(tikz-cd) trips a core-XML validity error but post-processing recovers (witness
2006.12702) — Rust-only, low priority (output recovered).

### Graphics renderer chain (subprocess-only; LANDED)
PDF→PNG `mutool draw`→`pdftocairo`→`convert+gs`; PDF→SVG `mutool convert`→
`pdftocairo`→`inkscape`. Subprocess `exec` (no GPL linking). Apt: `poppler-utils`
(req), `mupdf-tools` (rec), `imagemagick+ghostscript`, `inkscape`.

### Other tracks (separate docs)
- Performance: `PERFORMANCE.md` (P1 math/large-doc open; P2 allocation partial).
- Release gates: `RELEASE_CRITERIA.md`. Releasing: `RELEASING.md`.
- Completed missions (archived): strict-LoadFormat dump parity, Marpa ASF
  migration, distribution-readiness, the 500K/1M warning-corpus mission, and the
  diagnostic-message faithfulness pass (2026-06-20) — see `docs/archive/` and
  `git log`.
