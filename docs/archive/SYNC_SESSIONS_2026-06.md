# SYNC_STATUS archive — completed 2026-06 session logs & mined-out triage history

> Moved out of [`../SYNC_STATUS.md`](../SYNC_STATUS.md) in the 2026-07-02 docs
> consolidation. Everything here is COMPLETED work or mined-out triage history;
> live residuals were lifted into SYNC_STATUS before archiving. The upstream-sync
> U1–U11 details complement the catalog in
> [`UPSTREAM_SYNC_2767_to_2833_2026-06-26.md`](UPSTREAM_SYNC_2767_to_2833_2026-06-26.md).

---
  (guarded on the body still being plain's bare `\tabalign`; new
### Landed this session (2026-07-01, on `ar5iv-2606-prep`) — frontmatter title fidelity (no-`\maketitle` papers)

Two fixes to the post-PR#2767 Frontmatter API for papers that hand-format their
title as a leading `\begin{center}{\Large ...}` block and declare only an abstract
(no `\title`/`\author`/`\maketitle`), driver arXiv 1609.07638. Suite **1504/0**,
clippy + fmt clean; pushed (`437a8745`, `1ec4fb36`).

- **Ordering — keep the title block above the abstract (`437a8745`).** The
  abstract-only `\lx@frontmatter@fallback` flushed the abstract at the document TOP
  (after `ltx:resource`), floating it ABOVE the hand-formatted title block deposited
  before it. Scoped a rescue to the abstract-only case (exactly what
  `insert_frontmatter` already flags for deferral): insert at the CURRENT position so
  preceding body stays in place. Preamble-`\title` (no `\maketitle`,
  `tests/digestion/rebox`) still goes to the top.
- **Promotion — synthesize `<ltx:title>` from the first display block (`1ec4fb36`).**
  With no `\title` there was no `ltx:title` frontmatter, so the doc rendered
  titleless (Perl LaTeXML wished for this in its `{titlepage}` "how could we guess?"
  note but never built it). In the abstract-only fallback, promote the first
  non-resource body element's leading centered, larger-than-body paragraph to a real
  default-namespace `<title>` (children MOVED, xml:ids preserved; empty wrappers
  pruned). Conservative gates reject epigraphs / normal-size blocks / `\maketitle`
  papers. New fixture `tests/structure/promote_center_title`. The three
  construction-time DOM traps (relative-context `findnode` detachment, `_font` vs
  resolved `fontsize` timing, default-namespace element creation) are in
  `WISDOM.md` §41.

### Landed this session (2026-06-30, cont.) — pre-rerun audits: perf/stability hardening + gullet cleanup

Three requested pre-rerun audits (performance, stability, deep dives). All
changes suite-green (**1503/0**), clippy + fmt clean, `--release` re-validated.

- **Math over-parse — differential-`d` lexer gating (perf, output-neutral).** The
  lexer emitted the diffop-competing `XDIFFUNK`/`XDIFFID` terminal for *every* `d`;
  outside integrals `diffop_apply` always prunes it, so Marpa built a ~71-and-node
  branch per `d<var>` only to reject it. `util.rs::node_to_grammar_lexemes_from` now
  downgrades `XDIFFUNK→UNKNOWN`/`XDIFFID→ID` when the formula has no `INTOP` (same
  predicate + node list `diffop_apply` uses → **byte-identical output**), killing the
  over-parse on every non-integral `d` (high volume — differentials are everywhere).
  Full ranked lever list in `docs/math/MATH_OVERPARSE_DEEP_DIVE_2026-06-30.md` (which also
  corrects stale `MATH_AMBIGUITY_AUDIT` claims: `\Pi^N(p,q,r)` and simple `|x|≤|y|`
  are now unambiguous).
- **Stability — process-crash surface hardening (defensive, output-neutral).**
  Extended the base_xmath re-entrant-borrow fix (`75c452843d`) to the two remaining
  infallible Alignment arms in `digested.rs` (`revert` `:277`, `be_absorbed` `:315`)
  → `try_borrow` + graceful default. Hardened three math-parser panic sites the
  crash-surface audit ranked highest: `semantics.rs` `postfix_embellished`
  (empty-xmref `remove(0)` → bare Wrap, mirrors the landed `fenced` guard) and
  `fence` (degenerate `stuff[0]`/`len-2` underflow → prune), and `semantics/tree.rs`
  `get_value` (`.expect()`→`?` propagation on bad-lexeme-id). Fixed the
  `apply_math_ligature` `0..nmatched-1` usize underflow (`document.rs`, `saturating_sub`).
- **Stability — Cluster F fast-fail expansion-depth guard (lean).** A ~30-line
  thread-local `ExpandDepthGuard` at the top of `read_x_token` caps expansion-recursion
  DEPTH (= re-entrancy, covering all edges) at 12_000 (env `LATEXML_EXPAND_DEPTH_LIMIT`,
  0 disables) → graceful `Fatal:Timeout:Recursion` in ms instead of the watchdog/RSS
  grind. Measured: a `\csname a\a\endcsname` runaway 6.5 GB/1.5 s/exit-137 → 145 MB/0.10 s.
  Legit depth ~6–20 on real docs (incl. `NODUMP` expl3 raw-load) → 12_000 is ~600×
  margin. See `STABILITY_WITNESSES.md` Cluster F. (Full guard/crash-surface audit
  recorded; the `catch_unwind` per-formula-isolation option was left as a follow-up.)
- **Docs.** `PERFORMANCE.md` compressed 974→~380 lines (new Principle 6: no per-node
  whole-tree `//`/`preceding::` XSLT scans; FxHash node-cache shipping status
  corrected). `gullet.rs` verbose-comment cleanup 3023→2906 lines (essays condensed to
  their essential facts; no code change).
- **Validation — `sandbox-arxiv-10k-shuffle` rerun (2026-06-30, 9905 tasks).**
  Full cortex fleet rerun (72-worker harness, release binary) vs the 2026-06-29
  baseline. Aggregate: no_problem 6931→6928, warning 1756→1754, error 1166→1172,
  **fatal 52→51**, invalid 95. Transition audit (`/runs/…/diff` + per-paper
  before-vs-after `cortex_worker` repro): **3 genuine `fatal→error` wins** (the
  Alignment `try_borrow` / math-panic guards recovered hard-failing papers:
  0803.1344, 1205.0533, 1406.0085). The 2 "new fatals" (1506.09203, 1511.07586)
  were `cortex:never_completed_with_retries` **fleet-contention nondeterminism** —
  both convert to their baseline status standalone, before==after. The 3
  "error-regressions" (astro-ph9701035, physics0408020, 1408.6720) likewise
  before==after (multi-file main-selection nondeterminism). **Depth guard fired 0×
  across 10k** (the sole `Timeout:Recursion` fatal was the pre-existing cycle_guard
  "repeated token window", not the new `ExpandDepthGuard`) → zero false positives.
  Output-neutrality independently confirmed byte-identical on 145 sampled
  math-bearing papers (isolated-dir before/after).
- **PERF (2026-06-27): OmniBus natbib-autoload reload loop — FIXED.** The dominant
  arXiv slow/timeout cluster (~50 `sn-jnl` + Wiley/`sagej`/`wlpeerj`/… papers, all
  unbound classes → OmniBus fallback) hung ~90 s in digest: OmniBus's hand-rolled
  natbib autoload re-loaded natbib on every cite-CS re-emit. Routed through the
  canonical loop-safe `def_autoload` (clear-trigger-globally-before-load + hoist).
  2603.06884: 90 s→fatal ⇒ **0.5 s, 0 errors**. Regression witnesses 1403.6801 +
  2207.14344 both green. Full root-cause + breadth in `docs/performance/ARXIV_PERFORMANCE.md`.
- **PERF (2026-06-27): XSLT `f:seclev-aux` O(n²) — FIXED (output-neutral).** The
  second arXiv perf cluster (14 XSLT-dominated papers, ~133–167 s in XSLT) was an
  O(headings × tree-size) heading-level computation in
  `resources/XSLT/LaTeXML-structure-xhtml.xsl` (whole-tree `//` descendant scans
  per `ltx:title`). Memoized to per-name global `<xsl:variable>`s (O(n)).
  2404.12418: 179 s fatal ⇒ **34.7 s**; XSLT @99k 21.2 s → 5.3 s (below Perl's 8.7 s
  — Perl keeps the O(n²)). Byte-identical output, suite 1480/0. Shared upstream XSLT
  issue (candidate to upstream); see `docs/parity/OXIDIZED_DESIGN.md` #37 +
  `docs/performance/ARXIV_PERFORMANCE.md` Hotspot #2.
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

### Landed this session (2026-06-30, on `ar5iv-2606-prep`) — conditional-accounting root-cause + iflimit raise

**Combined session impact (final cortex rerun, all 30,079 sandbox-arxiv-2605 papers):
fatal 281→268, `Timeout/IfLimit` 32→4.** Three fixes: iflimit 8M→16M (`f3fab341`, the −12 fatal
mover + IfLimit collapse), base_xmath re-entrant-borrow panic (`75c452843d`, full-arXiv panic
cluster — 0 papers in 2605, broad value elsewhere), and tabularray `tblr` colspec
(`226d3bfa51`, fidelity + the compound TokenLimit runaway, e.g. 2605.06284 fatal→complete). The
error count is flat (±5 variance + a fatal→error shift as previously-fatal papers now complete
with their residual errors). NOTE: 2605's `\lx@begin@alignment` cluster stayed ~163 — it is
heterogeneous `\begingroup` group-leaks, NOT mostly tabularray, so the tblr fix's error-count
impact is small; its value is correct tblr column rendering + the runaway fix.

- **`iflimit` 8M→16M — Rust counts conditionals more comprehensively than Perl (FIXED, `f3fab341`).**
  Root-caused the long-standing "2× iflimit vs Perl" gap: it is NOT an accounting bug to
  tighten away, but *deliberate, more-comprehensive runaway counting*. Rust defines
  `\ifx`/`\ifcsname` (and the low-level TeX conditionals) as real `DefConditional`s that
  feed the global `if_count` runaway guard; **Perl does NOT count `\ifx`/`\ifcsname` toward
  its `if_count` at all**. On pgfkeys-driven tikz/pgfplots input (both engines raw-load the
  real `pgfkeys.code.tex` — Perl's native pgfkeys override is `__END__`-disabled), a
  controlled 2-plot figure that BOTH engines render identically (≈86 vs 87 graphic nodes)
  counts **148,078** conditionals in Rust vs **<200** in Perl (≈740×), dominated by `\ifx`
  (63%) + `\ifcsname` (15%) inside the pgfkeys key-dispatch. Counting these is *correct* — a
  `\ifx`/`\ifcsname` runaway is invisible to Perl's counter but caught by Rust's. So the
  right response is to raise the limit (per user directive 2026-06-30: "count as many as we
  can … doubling the limit is justified"), not count less. Real *finite* heavy docs measure
  ≈10–15M conditionals and complete in 24–43 s; a genuine runaway still trips well before the
  180 s worker lease (≈350k cond/s ⇒ 16M ≈ 46 s) and the RSS fuse. **Recovers coverage on
  12/32 `Timeout/IfLimit` papers** (the `\tikz@dashphase` pgfplots cluster), which Perl
  cannot convert at all (chokes on their expl3 first). Output byte-identical, suite 1502/0.
  **Corpus-wide cortex rerun CONFIRMS: fatal 281→269 (−12) on all 30,079 papers**
  (Timeout 148→136, the IfLimit cluster; no_problem/warning stable, error ±4 variance) —
  exactly matching the local 12/32 IfLimit measurement.

- **base_xmath re-entrant Alignment-reversion panic — FIXED (`75c452843d`).** Surfaced from the
  FULL arXiv corpus (<https://corpora.latexml.rs/.../arXiv/oxidized_tex_to_html>, 76 panics):
  reproducing one witness per panic *location* on current HEAD found **14/15 already fixed**
  locally; the one live cluster (2 papers, witness `hep-ph9806263`) was
  `base_xmath.rs` `\lx@gen@matrix@` reversion `al.borrow()` re-entering an Alignment RefCell
  left mutably borrowed by a broken `\matrix`/`\pmatrix` (mode-group mismatch). `try_borrow` +
  graceful fallback (mirrors `digested.rs:447`); Perl has no borrow-checker so its
  `$alignment->revert` never crashes. Witness now completes rc=1, 0 panics, genuine mode-group
  Errors intact (parity). So **all 76 public-corpus panic clusters are now resolved on
  `ar5iv-2606-prep`** (the public corpus runs an older binary; a re-run would clear them).

- **TokenLimit hot-loop root-cause (task #20) — heterogeneous; the tabularray one FIXED.**
  After the iflimit raise, 16/32 of the IfLimit cluster flip to `Timeout/TokenLimit`. Per user
  directive (root-cause the hot loops, then shortcut via native bindings or fix translation
  bugs), traced 3 witnesses via a sampled macro-expansion histogram (`EXP_TRACE`, reverted):
  the cluster is **heterogeneous** — three different hot loops:
  - **(1) 2605.06284 = `\@for` (`\@iforloop`, 62%) driven by tabularray — ✅ FIXED (`226d3bfa51`).**
    Rust's `tabularray_sty.rs` mapped `\tblr`→`\tabular` WITHOUT translating the `tblr` colspec;
    `{colspec={Q[c]…},hlines}` leaked into the classic alignment template parser
    (`alignment.rs:986`) → char-explosion ("Unrecognized tabular template" per char, the
    `\lx@begin@alignment` leak = a **12,100-paper** error cluster on the full arXiv corpus), and
    on big multi-table + tikz-cell-color docs the leak compounds into the `\@for`/pgfkeys runaway.
    `\tblr` now parses the inner spec and translates the `Q[…]`/`X[…]`/`c`/`l`/`r`/`p|m|b{…}`/`|`/
    `*{n}{…}` colspec mini-language to a classic `\tabular` template (correct column count +
    alignment), bailing to the unchanged stub on any unhandled construct (never worse). Witness
    2605.06284: **TokenLimit FATAL → completes rc=0**, 0 "Unrecognized" warnings (was 57+), 0
    `alignment.rs:986` errors (was 208). Surpass-Perl (Perl's ar5iv binding is the identical
    stub). Suite 1502/0 + unit test `colspec_translation`.
  - **(2) 2605.29738 = babel** modern `.ini` loading (`\usepackage[ukrainian,english]{babel}` +
    `\foreignlanguage{lithuanian}`; `\languagename` 52M× + `\bbl@ifunset`) — deep expl3/`.ini`
    interaction, NOT yet fixed.
  - **(3) 2605.05840 = expl3** l3kernel internals (`\__kernel_exp_not:w`/`\use_none:n`) — deep,
    NOT yet fixed. (2) and (3) remain dedicated deep efforts; the method + per-witness findings
    are the actionable record.

### Landed this session (2026-06-29, on `ar5iv-2606-prep`) — sandbox-arxiv-2605 prep for the July-5 2606 ar5iv run

- **fvextra `breakanywhere` PushbackLimit loop — FIXED (commit `c422c64937`), the #1
  genuine Rust-only fatal cluster.** `breakanywhere=true` installs a recursive
  char-scanner (`\FancyVerbBreakStart`) that measures every character by boxing a
  line-prefix through `predigest_box_contents`, growing the gullet pushback until the
  650k guard fatals; Perl converts cleanly. Fix routes the breaking line-processor to
  the non-breaking one (`\let\FV@ListProcessLine@Break\FV@ListProcessLine@NoBreak`,
  `fvextra_sty.rs`) — output-faithful (`font="typewriter"` preserved, browser-handled
  wrapping). **Official cortex before/after on all 30 079 papers: fatal 415→284
  (−31.6 %), PushbackLimit 185→63, OK+warn 82.4→82.7 %.** Regression
  `cluster_fvextra_breakanywhere`. Witness 2605.01024. See memory
  `sandbox-2605-error-landscape-2026-06-29`, `ar5iv-preload-required-for-sandbox-repro`.
- **Triage conclusion: the 2605 error/fatal tail is at a PARITY CEILING.** Two
  specific findings recorded for future agents (do NOT chase): (1) `undefined:\cellcolor`
  (483 papers) is PARITY — same-host Perl also leaves it undefined (a shared LaTeX
  option-clash from `\usepackage{xcolor}` then `\usepackage[table]{xcolor}`; the re-load
  is a no-op in both engines, `colortbl` never loads). (2) The `\lx@begin@alignment`/
  `\Gscale@@box` cluster (~164/129) is a heterogeneous `\begingroup`-leak mechanism, NOT
  tabularray-driven (minimal `tblr` passes) — and Rust frequently already BEATS Perl
  there (2605.00025: Perl fatals/timeouts, Rust completes with errors). No second clean
  Rust-only error fix exists in this corpus's tail.
- **SPEED on large math books — 3rd XSLT O(n²) FOUND & FIXED.** Witness 2605.01585 (a
  multi-chapter physics book, 2000+ formulae, 512 titles): `maketitle`'s per-title
  `//ltx:navigation` full-tree scan was 22.7 s of 24.9 s of XSLT. Memoized to a global
  → **XSLT 24.94 s → 2.15 s, output byte-identical, suite 1502/0** (Open task §3 below,
  OXIDIZED_DESIGN #41). The fleet `phase_xslt_us` on this paper (65.7 s) collapses to
  ~2 s; large books that were near/at the 180 s timeout should flip fatal→complete.

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
`docs/performance/ARXIV_PERFORMANCE.md` (Hotspot #3).

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


## Methodology history — autonomous-mining taper (2026-06-21) + phantom clusters


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
- Remaining candidates: `jpconf` class → iopart (18+ IOP-conf papers).
  (theorem/mdframed-in-figure schema `figure_mixed_content` — ✅ FIXED 2026-06-27,
  float content models accept `theorem`/`proof`; OXIDIZED_DESIGN #38.)

---

