# Engine Sync Status — Active Worklist

> **DO NOT downgrade Errors to cheat the task.** If Perl LaTeXML
> converts a paper without a downgrade, the Rust translation must
> match by improving the core engine — never by silencing
> diagnostics. Acceptable pre-existing exception:
> `is_typesetting_only_message` entries that match Perl's behavior
> on the SAME paper (e.g. "Running heading author exceeds size
> limitations" per WISDOM #50). Any NEW downgrade requires explicit
> proof Perl emits the same severity, otherwise it's hiding a real
> engine gap. User directive (2026-05-15): "downgrading errors is
> generally cheating at the task and must not be attempted."

---

## Active mission (Round-36, opened 2026-05-22): 1,000,000 error-free conversions on the arXiv "warning" corpus

**Goal.** Reach **1,000,000 successful conversions** with the Rust
translation (`cortex_worker --standalone`) on the 1,000,001-paper
subset of arxmliv where the original Perl LaTeXML emitted at least
one warning. This is the strongest practical regression harness we
have: every paper is a known stress case for the engine, and the
gap to 100% measures translation completeness more accurately than
any synthetic benchmark.

### Input corpus

* **Source list.** `~/data/all_warnings.txt` (psql dump, 1,551,853
  rows; 2 header lines + paths shaped as
  ` /data/arxmliv/YYMM/ID/ID.zip`).
* **Slice.** First **1,000,001** data rows (lines 3–1,000,003 of
  the file).
* **On disk.** 500,000 zips already rsync'd to
  `~/data/large_scale_canvas_3/data/arxmliv/`
  (181 YYMM subdirs `0001`..`1501`, 194 GB).
* **Pending rsync.** Second 500,000 zips
  (lines 500,003–1,000,002 of `all_warnings.txt`; YYMM `1501`..
  `2110`) — staged when the first 500K is fully through the canvas.

### Driver

`cortex_worker` in standalone mode is the harness:

```bash
ulimit -v 6291456                          # 6 GiB virtual-address cap
timeout 130 cortex_worker --standalone \   # 130s wall, 120s internal
  --timeout 120 \
  --input  $zip \
  --output $workdir/out.zip
```

Per-worker classification:

| Exit code | Class       | Meaning |
|----------:|-------------|---------|
| 0         | `OK`        | clean conversion (HTML ≥ 500 B), or `OK_EMPTY` for runaway-empty output |
| 124       | `TIMEOUT`   | wall-clock exhausted |
| 137       | `OOM`       | OS-killed via ulimit |
| 139       | `FATAL_139` | SIGSEGV (typically libxml2/libxslt under memory pressure) |
| 101       | `FATAL_101` | Rust panic |
| ≥3        | `FATAL_n`   | engine bailed with status code `n` (`Error::log_fatal` chain) |

Canvas is parallelised at 16–32 workers via `xargs -P` per stage of
10,000 papers, results land in `canvas/stage_NN/results.txt`.

### Iteration protocol

1. **Run a stage.** 10,000 zips per stage; ~16 workers; per-paper HTML
   written to `canvas/stage_NN/.work/<paper>/out.zip`.
2. **Conserve disk.** Once a stage closes, *delete* the per-paper
   output zips for `OK` papers. Failed-paper outputs (and logs)
   stay for triage. Each closed stage frees ~30–50 GB.
3. **Triage failures.** Group by status code first; within `FATAL_3`
   group by the last error line / cascade origin. New clusters of
   ≥3 papers usually share one engine root cause.
4. **Perl-parity check.** For each non-`OK` paper, run Perl LaTeXML
   `latexml --noparse --quiet --path=$HOME/git/ar5iv-bindings
   --preload=ar5iv.sty <main>.tex`. If Perl also fails, the paper
   is a **SHARED-FAILURE** — log it (below) and move on. Only
   Rust-only failures are R36 work.
5. **Fix the engine.** Land the smallest engine change that closes
   the cluster, with a regression test only when the fix is
   well-localised (large stubs ride on the canvas as their test).
   Commit per logical fix.
6. **Re-run the cluster.** After every commit batch, re-verify the
   newly-fixed witnesses (cheap), then re-queue the still-failing
   ones into the next canvas stage's tail (full re-run).
7. **Repeat** until each closed stage holds 0 non-`OK`.

### Sandboxes

* `~/data/large_scale_canvas_3/canvas/stage_NN/` — live canvas state.
* `~/data/canvas_3_failures_sandbox/` — frozen failure zips from
  the 150K canvas-3 baseline (kept as a regression-style witness
  pool even as the engine improves; do NOT regenerate the HTML).

### 🎯 500K MILESTONE REACHED (2026-05-23 08:30 local)

| | Value |
|---|---:|
| **Stages closed** | **50 of 50** (first 500K batch complete) |
| **Total papers** | **500,000** processed, **499,832 OK**, **99.9664%** |
| **Best stage** | stage_49 at **99.99% (9999/10000)** |
| Failure distribution | 126 FATAL_3, 16 OOM, 15 TIMEOUT, 4 FATAL_139, 3 FATAL_101, 3 FATAL_1, 1 FATAL_134 |
| Tests | **1,334 / 0 / 0** |
| Branch | `large-scale-testing-round-3`, 920+ commits ahead of `origin/master` |
| Second 500K rsync | 903,716 zips on disk (~403K of next 500K complete) |

**Cumulative-fix retest of all 168 failures (2026-05-23 update post
lstMakeShortInline-of-CS fix c78e0fe556)**: 47 PASS / 67 FAIL / 11
TIMEOUT / 24 MISSING-from-disk + 1 has-error. Of the 67 still
FATAL, **Perl also fails on 45** (SHARED-FAILUREs). Only 11 are
true PERL_OK_W_WARN (Rust-only) candidates:
* `1004.4538` — biblatex `\lossort\endlossort` PushbackLimit:
  triggered at ~20+ entries in `\thebibliography` expansion; root
  cause: `bib_as_thebibliography` emits all variants as Tokens in
  one shot, expansion cascades through `\par@in@bibliography`-style
  rebinds. Single-entry isolated repro: see `/tmp/u/biblat_min*`.
* `1012.1313`, `1012.1340` — `erics_preprints.sty` missing → both
  engines suffer undefined-macros, Perl tolerates 26/16 errors,
  Rust hits 100-cap. Higher error multiplier per cascade.
* `1301.0040` — `pst-all.sty` + `macros.sty` + `eptcs.cls`
  missing; same error-multiplier shape.
* `1207.2132` — `mhsetup.sty` raw load triggers PGF
  `\pgfutil@xifnch` undefined cascade (only **inside** pgfutil-
  common.tex line 174 `\expandafter\gdef\:` — needs deeper
  investigation of TL-2023 PGF token interaction).
* `1207.4709`, `1310.8644` — pb-diagram.sty / mathpartir.sty
  missing → diagram/halign cascade.
* `1307.0538`, `1402.6510`, `1403.5962`, `1408.2108` — pstricks /
  pst-all / curve2e / `\omit`-cascade.

**Random samples (2026-05-23) from the 1501-2110 second-500K corpus**:
* **500**: 290 PASS / 207 WARN / 3 errors / 0 FATAL.
* **1000**: 562 PASS / 435 WARN / 2 errors / 1 FATAL
  (arXiv:2103.03138 — chemnum, fixed by `be19874ba0`).
* **2000**: 1185 PASS / 808 WARN / 7 errors / 0 FATAL.
* **5000**: 2911 PASS / 2078 WARN / 11 errors / 0 FATAL —
  **99.78% non-fatal, 58.2% clean pass**.
* **10000 FINAL**: **5900 PASS / 4086 WARN / 10 errors / 4 FATAL**
  — 99.86% non-fatal, 59.0% clean pass. ALL 4 FATALs confirmed
  SHARED-FAILUREs (Perl also `too_many_errors`s on each):
  arXiv:1501.03690 (`\endcsname` extra at internal token),
  1512.05621 (text-mode cascade in `\text{Tr}^L_X` math),
  1502.06361 (text-mode cascade post-fullpage),
  1910.02237 (svjour3 text-mode cascade). The 100-error cap
  behavior matches Perl exactly.
* **1000 from early years (07-14)**: 696 PASS / 303 WARN /
  1 error / 0 FATAL.

Sampling-driven stubs landed:
* `3e4e0cc25d` — rotfloat (witnesses: arXiv:2101.12526, 1804.05845).
* `00412df771` — tabls (witness: arXiv:2003.12942).
* `be19874ba0` — chemnum (witness: arXiv:2103.03138).
* `edeb9b62f7` — pax (witness: arXiv:1512.06235).
* `fd85f769c9` — figcaps (witness: arXiv:1912.07260).
* `d0c5f760ed` — refstyle (witnesses: arXiv:1804.06350, 2009.10518).
* `7bc8a6cec9` — envmath (witness: arXiv:1501.05259, a real
  Rust-only PushbackLimit fatal).

Remaining sample failures are paper-local typos (`\lx`,
`\MedicalPrizeEditors`), `_` in text mode, refstyle's
`\eqref already defined` vendor error, tikz positioning — all
non-fatal, 0 FATALs at sample-2000 scale.

**Post-fix retest #3 (TeXDelimiter END-token fix)**: 70 PASS / 69
FATAL of 179 retested (+2 vs run #2). Newly passing:
arXiv:1207.4709, 1101.2531.

**Architectural investigation 2026-05-23 (mhsetup → tikz bleed)**:
Traced `\usepackage{mhsetup, mathtools}\usepackage{tikz}` cascade.
Root cause: `invoke_token`'s continuation read
(`gullet::read_x_token(None, ...)` in stomach.rs L1070-1081)
defaults to autoclose=true and pops past the mhsetup.sty mouth
boundary, pulling the user's NEXT `\usepackage{tikz}` token
into the raw-load loop. After tikz finishes loading, mhsetup's
`\AtEndOfPackage{\MHInternalSyntaxOff}` hook fires too late
(`:` was still at catcode 11 when pgfutil-common.tex parsed
`\:` — yielding a control word instead of the expected control
symbol). Defensive catcode reset in `mhsetup_sty.rs` only helps
the separate-line form; the digest auto-pop fix breaks
`csquotes_test` (digest IS expected to bleed in some contexts).
A proper fix needs scoped autoclose semantics — deferred.

**Post-fix retest #2 (6 fixes landed total: listings, mathpartir,
curve2e, pst-all, biblatex \verb, mhsetup)**: 68 PASS / 70 FAIL /
11 TIMEOUT / 24 MISSING of 179 retested. +21 papers recovered
vs previous retest snapshot. Of remaining 70 FATAL:
* **58 SHARED-FAILUREs** (Perl also fails — engine recovery
  ceiling reached).
* **12 PERL_OK_W_WARN** (Rust-only divergence). New ones surfaced
  beyond the earlier 11:
  * `0911.1590` — `\lx@equation@settag@` mode-switch (reverted
    fix would break eqnums_test).
  * `1102.2909` — xy-pic 8M conditional-limit infinite-`\if`.
  * `1305.0848` — tikz MemoryBudget exceeded.
  * `1402.7269` — pst-plot stub triggers PushbackLimit.
  * `1404.6225` — ctable "load after tikz" → Convert TIMEOUT.

**Post-fix retest #1 (4 stubs landed: mathpartir, curve2e, pst-all,
1105.4136 listings)**: 3 of 11 PERL_OK_W_WARN now PASS cleanly:
  * `1310.8644` — mathpartir stub: now 1 warning (was fatal)
  * `1402.6510` — pst-all stub: now 4 warnings (was fatal)
  * `1408.2108` — curve2e stub: now 1 warning (was fatal)
  * `1301.0040` — partial recovery (pst-node stubs help, but
    pspicture-with-math mode-switch still fatals).

Remaining 8 of 11 PERL_OK_W_WARN need engine-level work:
  * `1004.4538` — biblatex `\lossort\endlossort` PushbackLimit
    (>=20 entries trigger; root cause in `bib_as_thebibliography`
    bulk-token-injection path).
  * `1012.1313`, `1012.1340`, `1207.4709`, `1307.0538`, `1403.5962`
    — error-count multiplier vs Perl: missing-package or paper-
    local-macro cascades produce 100+ errors in Rust where Perl
    produces fewer than 100. Cross-cutting investigation needed.
  * `1207.2132` — PGF `\pgfutil@xifnch` undefined cascade
    (mhsetup + tikz interaction).

Projected rerun rate on the full 500K: ~99.974% OK (from 99.9664%
historical).

### Session R36 — 18 root-cause fixes landed, 28+ papers closed

**1207.4709 deep-dive (2026-05-23)**: Traced the `\smalltwomatrix`
cascade in align*. The user's `\newcommand{\smalltwomatrix[5]}{...}`
correctly defines a 5-arg macro (both Perl and Rust). The actual
paper invokes it with only 4 brace-groups: `\smalltwomatrix{B}{x}{}{t}\big|...`.
TeX reads `\big` as the 5th arg. In the body, the substituted `#5`
becomes `\big`, which is `\big TeXDelimiter` — our impl reads the
next token (`\end`) as the delimiter, swallowing the
`\end{smallmatrix}` close. The alignment env stays open → cascade.

Perl's `\big` is more lenient with non-delimiter follow-tokens
(emits a warning rather than swallowing). Fixing this requires
audit of our TeXDelimiter param reader vs Perl behavior.
Deferred.

**Latest sandbox retest (16 frozen failures, 2026-05-23)**:
* PASS: physics0003074, hep-th0009218, math0009192 (was FATAL_139);
  hep-ph0012156 (was FATAL_101); math0104252, gr-qc0209055,
  gr-qc0301024 (was TIMEOUT) — **7/16 historical failures
  auto-recovered**.
* Still fail: math0102053/.089, math0212126, math0402448,
  math0504436, math0506088, math0507219, math0604321 (all plain
  TeX MemoryBudget — paper-bundled `\catcode @=11`, `\magnification`,
  custom `\newcount` — no `\documentclass`); math0203082
  (tabular-only fragment).

### Session R36 — 17 root-cause fixes landed, 24+ papers closed

| Commit | Fix | Papers recovered |
|---|---|---:|
| `d167f86785` | `load_class`: defer deps-scan until AFTER alternate-class loads (OmniBus order) | 7 (statsoc/ectj/compositio/biom clusters) |
| `9c578bcaa9` | `ams_support`: gate `\pf`/`\pf*` env aliases on 2.09_COMPATIBILITY | 1 (1102.0135) |
| `a38d0db250` | `titleref.sty`: minimal stub binding (\titleref→\ref) | 1 (1103.2227) |
| `6a64259589` | `ccaption.sty`: minimal stub binding (extensions→\caption) | 1 (1105.3285) |
| `a900101da3` | `acronym.sty`: defer `\Ac`/`\Acf`/etc. via `\AtBeginDocument` | 1 (1102.0244) |
| `8f00710f64` | `backref.sty`: minimal stub binding (no-op back-refs) | 1 (1107.0498) |
| `585996033f` | `omnibus`: `\frontmatter`/`\mainmatter`/`\backmatter` as noop overrides | 2 (1102.3639, 1004.3619 — memo-l cluster) |
| `fbe8626c57` | `oldlfont.sty`: minimal stub (preserve kernel \mathit etc.) | 1 (1112.3561) |
| `684563dd12` | `digested.rs`: `try_borrow` defensive fix (prevent RefCell panic) | 1 (1205.0376) |
| `7598a82b32` | `graphics.rs`: UTF-8-safe slice (prevent SVG-preamble panic) | 1 (1307.4573) |
| `caaf1433c0` | `amsmath`: `\ext@arrow` 5th arg → `{}` for extpfeil-style braced calls | 1 (1308.1071) |
| `9ff8c22986` | `omnibus`: drop natbib-autoload global-clear (preserve natbib's local def) | 1 (1403.6801) |
| `3767609b46` | `nag.sty`: minimal stub (no-op obsolete-CS lints, preserve mode tracking) | 1 (1411.3836) |

### Retest of all 98 prior failures with latest binary

Of 98 papers that failed in earlier stages, **45 PASS** with the
current binary (cumulative effect of session fixes). Remaining 53
triaged against Perl:
* **Genuinely Rust-only (5 papers — all deep engine issues):**
  * `gr-qc0301024` — Perl 0.47s OK, Rust hangs in (Building...)
    phase. LaTeX 2.09 `\documentstyle{iopconf}` doc, pictex
    raw-load successful but XML-construction loops indefinitely.
    Deep schema-validation / build-phase perf gap (not digestion).
  * `math0504436` — Perl 0.22s OK, Rust Convert TIMEOUT. amsart
    + eucal + paper-bundled `treetex.tex` / `classes.tex`
    (custom `\newcount`/`\loop` low-level TeX). classes.tex
    digestion hangs on user-defined math binary-tree macros.
  * `1004.4538` — Perl 7 errors complete, Rust hits
    `PushbackLimit:650000` infinite loop in biblatex `.bbl`
    processing. Undefined `\mathbf`/`\emph`/`\mathbb` cascade
    inside the bbl entry body triggers runaway re-expansion.
  * ~~`1105.4136`~~ — **FIXED** (c78e0fe556). Root cause was
    `\lstMakeShortInline{\"}`: our Rust impl took the first char of
    a 2-char CS string (`\`), making backslash active and corrupting
    every subsequent `\foo`. Now matches Perl's no-op-for-CS
    behavior.
  * `math0507219` — Perl 5 errors complete, Rust fatal. Old TeX
    picture-style figure (`\put`/`\unitlength`/`\picture`)
    inside an obsolete user-defined `\droite` macro chain.
* **SHARED-FAILUREs (~48 papers):** Perl also fails or times
  out. Most underscore-catcode cascades from missing class/package,
  or pictex/pstricks raw-load slowness affecting both engines.

All 5 remaining Rust-only failures require dedicated engine-level
investigation (build-phase profiling, expansion-recovery overhaul,
catcode-leak tracing) beyond the tactical session-scope fixes.

Triage of stages 28-30 (10 FATAL_3 + 1 TIMEOUT, sampled with new
binary): **0 Rust-only** — all 11 are SHARED-FAILUREs (Perl also
fails) or auto-fixed by the OmniBus reorder:
* 4 auto-passed with new binary (`1003.4546`, `1004.0524`,
  `1005.4553`, `1008.3706`).
* 1 fatal in shared category at `Fatal:Timeout:PushbackLimit` cap
  (`1004.4538` — Perl produces 7 errors+complete, Rust fatals at 650K
  pushback safety net; borderline whether to count as Rust-only).
* 6 Perl-also-fails (1004.2276, 1004.3619, 1004.5482, 1006.3261,
  1006.5461, 1009.3622, 1009.4876, 1009.6139, 1010.5320; mostly
  underscore-catcode cascades from missing class/package).

### Stage 31 final (post-OmniBus-fix binary) — 99.94% OK

Stage 31: 9994 OK / 5 FATAL_3 / 1 TIMEOUT. Triaged:
* 3 SHARED-FAILUREs: 1012.2852 (TooManyErrors), 1101.2531 (pictex
  timeout — Perl also hangs), 1102.2909 (Perl also fatals).
* **Rust-only — closed by `ams_support`-`\pf`-env-gate fix
  (commit 9c578bcaa9, 2026-05-22):**
  * **`1102.0135`** ✓ — `\newcommand{\pf}{...}` AFTER
    `\begin{document}` was being silently ignored because our
    `\AtBeginDocument` block had pre-defined `\pf` as
    `\begin{@proof}`. Subsequent `$\pf$` expanded into proof env in
    math mode → `\itshape`/`\not@math@alphabet@@{\itdefault}`
    warning → cascading mode-mismatch errors. Fix: gate the alias
    on `2.09_COMPATIBILITY` like Perl does. Now "No obvious
    problems".
* **Open Rust-only:**
  * **`1102.0244`**: pstricks cluster (same as 0712.0243) — Perl
    converts in ~1 min, Rust times out. Engine-perf gap on pstricks
    raw-load chain.
  * **`1102.3639`**: missing `memo-l.cls` + missing user macros
    (`\Ext`, `\opH`, `\mathbb`, etc.). Perl handles with 14 errors
    "complete", Rust cascades to 101 errors + fatal via the
    underscore-catcode-in-text-mode path. Same shape as 1004.3619.
    Likely benefits from better undefined-macro recovery in math
    context.

Stage 32 (post-pf-gate-fix, in flight): 3977/3978 = 99.97% OK.

### R36 commits landed this session (6)

| Commit | Fix | Papers recovered |
|---|---|---:|
| `3b1024de83` | `delarray.sty` no-op binding (preserves binding-aware `\@@array`) | 8 |
| `17f587c0fe` | Merge `origin/master` (1M-arXiv PR + indexmap 2.14.0 + ProcessOptions keysets) | — |
| `a68505d52e` | `babel_lang_stubs`: `\expandafter\newlanguage\csname...` (16 stubbed langs) | 1 (brazil) |
| `fb588899df` | `trace.sty` no-op binding (bypasses `\frozen@everymath` self-reference) | 1 |
| `4a1b326151` | `let_i`: deep-copy robust-wrapper pair (Expandable+`\<cs><space>` body) | 1 |
| `ee92ead429` | `mdwtab.sty` + `mathenv.sty` no-op bindings (preserves binding-aware `\tabular`/`\eqnarray`) | 2 (stage-26+27) |

Stage 16-23 sandbox went **0/22 → 11/22 OK**. Stages 24-27 fresh
FATAL_3 cohort (26 papers): re-verified, **10/26 already fixed by
prior R36 commits** (mostly `delarray.sty` + `let_i` deep-copy);
remaining 16 split into 9 SHARED-FAILUREs + 7 Rust-only (5 Convert
TIMEOUTs + 2 mode-mismatch). `mdwtab.sty` commit then closed 2 of
the 7 Rust-only (0910.3293, 1002.3613).

Open Rust-only (post-R36 commits):

| Paper | Stage | Class | Notes |
|---|---|---|---|
| 0712.0243 | 20 | TIMEOUT | pstricks-heavy doc, hits 120 s ceiling — separate root cause |
| 0911.1590 | 26 | `\tag\textsc{…}` cascade | needs engine `Digested` parameter-type for `DefPrimitive` (see archive notes) |

**Recently closed (`OmniBus-load-order` fix, 2026-05-22):**
0809.4358, 0904.3132, 0904.3938, 0908.3882, 0912.1617, 1001.1919, 1001.5004 — all
**no-class-binding** cases where the alternate-class deps-scan (Perl's
`maybe_require_dependencies` analogue) used to fire BEFORE the OmniBus
fallback. natbib (or any `\RequirePackage{natbib}`-bearing deps-scan)
loaded its `Let('\bibitem', '\lx@nat@bibitem')` first; THEN OmniBus's
`Let('\lx@OmniBus@saved@bibitem', '\bibitem')` + `DefMacro('\bibitem',
...)` clobbered natbib's binding — infinite-loop chain on
`\bibitem[\protect\citeauthoryear{...}{...}{...}]{key}`. The fix
defers the deps-scan to AFTER the alternate-class load (matches Perl's
order: warn → OmniBus → deps-scan), and removes the `alternate.is_some()`
gate so the deps-scan also runs for the pure-OmniBus fallback path.
See `latexml_core/src/binding/content.rs::load_class` (commit landing
2026-05-22).

**Cluster hints (remaining):**
* **`0712.0243` (pstricks)** — heavy pstricks loadout. Not related
  to the OmniBus-order cluster. Profile pstricks chains for the slow
  expansion.
* **`0911.1590` (`Digested` parameter type)** — Perl's
  `latex_constructs.pool.ltxml L2053` uses `DefPrimitive('\lx@equation@settag@
  Digested', ...)`. Our `latex_constructs.rs::L5527` uses `{}` + manual
  `stomach::digest(content)?` inside `mode => "restricted_horizontal"`.
  Two divergences: (1) explicit `?` propagates digest errors instead
  of locally catching them, (2) wrong mode flips `\ifmmode` evaluation
  → orphan `\else`/`\fi` cascade. **Fix path**: add `Digested`
  parameter-type support to `DefPrimitive` (currently only
  `DefConstructor` accepts it). Engine work, deferred — needs broader
  audit of `DefPrimitive` call sites that might benefit.

### Open R36 tactical work

* **Rsync the second 500K** (in flight, PID 3557279; the local
  rsync 3.2.7 with a 500K `--files-from` is slow to start because
  the receiver-side `rsync --server --sender` has to stat every
  entry before transfer begins; first new file expected within
  another 5–15 min).
* **Stages 28–50** — let the canvas keep grinding while engine
  fixes accumulate; re-classify each new cluster.
* **Rust-only triage list above** — 5 of 9 are Convert TIMEOUT
  (group by what's slow); 2 are mode-mismatch (likely shared
  mode-stack invariants); 1 conditional issue (post-enumerate
  `\else` cascade).
* **mhchem 77-error cluster** — see "mhchem retirement" below;
  retire `latexml_contrib/src/mhchem_sty.rs` (~110 LoC stub) by
  closing the upstream `\int_value:w` mis-evaluation at the head
  of the cascade.

---

## SHARED-FAILURE log (Perl + Rust both fail identically)

These papers fail in both engines for the same reason. They count
as **out of scope** for R36 and should not be triaged repeatedly.

* **`\def\<one-letter-CS>` before `\documentclass`** — kernel
  redefines `\d`/`\th`/`\b` to text accents on load, then `$\d_x$`
  trips text-mode underscore. Witnesses: hep-th0005159, hep-th0010165,
  hep-ph0001306, cond-mat0102064, cond-mat0103632, hep-th0005268.
* **pstricks `\ifpst@useCalc`/`\ifpst@psfonts` undefined** —
  paper `\input`s `pstricks-dots.tex` before `pstricks-tex.def`
  runs, so the `\newif`-conditionals are missing. Witnesses:
  astro-ph0002346, astro-ph0002348.
* **amsart `_/^` cascade after `\maketitle` /
  `\numberwithin{equation}{section}`** — math0010241.
* **plain-TeX `\input psfig.sty` mid-document reload** —
  cond-mat0010356, cond-mat0101405.
* **Paul Taylor `diagrams.tex` time-bomb** — TL v3.96 L2630-2631
  `\ifnum\count@>24307 …\endinput\fi` expired July 2025. Re-evaluate
  when v3.97 ships.
* **xcolor double-load Option clash** — paper-local `.cls` runs
  bare `\usepackage{xcolor}` then user adds
  `\usepackage[svgnames,x11names]{xcolor}`. Witnesses: 2204.01429,
  2204.01753. Surpass-Perl path (not yet designed): when xcolor is
  re-loaded with new options, process them instead of suppressing
  the second `\usepackage`.
* **Canvas-3 stage 16–23 SHARED-FAILUREs (R36 verified 2026-05-22):**
  math0611010 (xy-pic OOM), hep-ph0612355 (feynmp SEGV),
  math0703454 (R35.A MoveableBox depth-cap), 0708.3218, 0708.3398
  (harvard.sty timeouts), 0809.3663 (memo-l.cls), 0809.3725
  (`\@math@baccent`), 0901.1928 (XMApp-in-emph).

---

## mhchem retirement (deferred R36 long-tail)

`latexml_contrib/src/mhchem_sty.rs` intercepts TL `mhchem.sty`
(~110 lines as of 2026-05-19). The raw chain is `chemgreek` →
`xparse` → expl3 (group machinery, `\__file_tmp:w`, l3regex,
l3tl-analysis). Driver: arXiv:1806.06448.

**Minimal repro** (`LATEXML_MHCHEM_NOLTXML=1` to bypass the stub):
`\documentclass{article}\usepackage[version=3]{mhchem}` +
`\ce{H}` → **77 errors** in Rust, 0 in Perl. Just
`\usepackage{mhchem}` without `\ce{...}`: 0 errors. So the 77-error
cascade is triggered specifically by the first `\ce{...}` call.

**First diagnostic anomaly:** the cascade begins with
`Warn:expected:<number> Missing number, treated as zero while
processing "\int_value:w", next token is Some(";")`. The
`\int_value:w` (PA→`\number`) is called and sees `;` directly with
no leading digit — the expected preceding digit-producing
expansion produced *no digits*. Every following expl3 token
(`\__int_eval_end:`, `\fi:`, `\else:`, `\s__tl`, `\tex_skip:D`, …)
shifts left by one slot and surfaces in `\csname...\endcsname`
reads where it shouldn't.

**Root-cause hypothesis** (2026-05-12 deep dive): `read_x_token`
returns PA-aliased CS tokens as opaque `Stored::Token(\let-target)`
and the csname-reader then errors because the let-target is itself
a CS, not a character.

**Next step:** instrument `read_x_token` to log token + meaning
class around line 6 col 1 in the minimal repro; narrow to the
first non-empty return that doesn't match the expected expansion.

---

## Permanent ignores

* **Sandbox out-of-scope**: ns1–ns5 (52_namespace, no DTD); 2402.03300,
  2410.10068, 2511.03798 (Perl also fails).
* **Rust supersedes Perl** (both in scope, Rust passes where Perl
  errors): `1207.6068`, `0909.3444`, plus 40+ in
  `memory/project_rust_supersedes_perl.md`.
* **Unported pools**: `BibTeX.pool.ltxml` (skip via `--nobibtex`).

---

## Acceptance gates

| Gate | Current (2026-05-22) | Target |
|---|---|---|
| `cargo test --tests` | **1334/0/0** | unchanged |
| `cargo clippy --workspace --all-targets` | 14 warnings (all in `latexml_math_parser`, post-ASF cleanup — collaborator's lane) | 0 warnings |
| `latexml_oxide --init=plain.tex` | 0 errors (dump + `LATEXML_NODUMP=1`) | 0 errors |
| `latexml_oxide --init=latex.ltx` | 0 errors (dump + `LATEXML_NODUMP=1`) | 0 errors |
| 1910.01256 mini-benchmark vs pdflatex×2 | **0.71 s** (release, full post-proc); pdflatex idle ~1.11 s | beat 2× pdflatex (met) |
| Distribution build size | release: **44.38 MB**; `--no-default-features --profile maxperf`: ~44.98 MB | met |

Distribution chain (LANDED 2026-05-15): versioned dump filenames
+ compile-time embedded fallback via `include_bytes!`; TL2023 +
TL2025 currently bundled. Resolution chain:
`$LATEXML_NODUMP` → `$LATEXML_DUMP_PATH` →
`$LATEXML_DUMP_DIR/<kind>.YYYY.dump.txt` → exe-relative → dev-tree
→ embedded fallback. IA consolidation (`81176ba689`) halved the
latex dump (~7.4 → ~3.7 MB).

---

## Tikz known diffs vs Perl (reference)

1. `foreignObject` transform Y / width/height.
2. Arrow-tip shape (different path data).
3. SVG viewBox / total width differs slightly.
4. matrix uses `<svg:g class="ltx_tikzmatrix">` (Rust) vs inline-blocks
   (Perl).

---

## Post-processing graphics renderer chain (LANDED 2026-05-12, reference)

Subprocess-only, no library linking — AGPL/GPL on the underlying C
libraries (MuPDF, poppler) does not propagate because we invoke
standalone binaries via `exec`. Required apt packages:
`poppler-utils` (mandatory), `mupdf-tools` (recommended optional,
~1.7× faster), `imagemagick + ghostscript` (last-resort), `inkscape`
(SVG last-resort).

PDF → PNG: `mutool draw` → `pdftocairo --png` → `convert + gs`
(60 s hard timeout). PDF → SVG: `mutool convert -F svg` →
`pdftocairo --svg` → `inkscape` (15 s hard timeout).

Rust-crate alternatives evaluated and rejected: `mupdf-rs` (AGPL),
`poppler-rs` (GPL), `pdfium-render` (license-clean but not
thread-safe — Mutex-serialising the 5-worker graphics phase wipes
out the in-process benefit).

---

## Performance follow-ups (separate track — see `PERFORMANCE.md`)

* **P1 graphics** — CLOSED 2026-05-12. Primary rasterizer optimization
  (`5244a5a4e2` → `feaf8bcd16`) brought graphics 1031 ms → ~480 ms on
  1910.01256. Content-identity conversion cache + cross-document
  duplicate coalescing landed in follow-ups.
* **P1 digest+build** — CLOSED 2026-05-19. Profile-driven sweep on
  `2305.06773`: residual cost is structural to the TeX
  read-then-invoke pattern; combining the two probes would require
  an API change on the gullet (out of scope per user directive
  2026-05-19). Internal wins landed: `Catcode::name_sym`, `has_meaning`
  migration, `Token::pin_cs_name`, plus 6 clippy-driven sweeps.
* **P1 math/large-doc** — open; `LATEXML_PARSE_AUDIT=1` on
  astro-ph0204009, 0911.0884, astro-ph0401354, 0809.5174,
  astro-ph0507615 when bandwidth allows.
* **P2 allocation/startup** — partial; reopen only when a fresh
  profile shows entries above the SwissTable-probe floor.

---

## Math parser ↔ Marpa ASF migration — CLOSED 2026-05-19

Multi-session ASF traversal migration is **landed**. Marpa is back
on master (`dginev/marpa` master, commit `0bf241116fcef…`,
PRs #3 + #4 merged). HYBRID is the default; `LATEXML_MARPA_ASF=1`
turns on the ASF traversal; `LATEXML_MARPA_ASF_ONLY=1` forces it
alone. Both modes: **1334/0/0** on this branch.

Full design + retro: [`docs/MATH_PARSER_AND_ASF.md`](MATH_PARSER_AND_ASF.md),
[`docs/MATH_PARSER_ASF_TIEBREAKING.md`](MATH_PARSER_ASF_TIEBREAKING.md),
and the ASF-fork retro at [`marpa/ASF_STATUS.md`](https://github.com/dginev/marpa/blob/asf-completion/ASF_STATUS.md).

---

## Distribution-readiness dependency cleanup — CLOSED 2026-05-19

Release binary **44.60 MiB stripped** (down from 57.12 MiB pre-audit);
.text ≈ 34.3 MiB, .rodata = 2.2 MiB (TL2023+TL2025 dumps gzipped).
The remaining .text is OUR macro-arm bindings (latexml_package 41%,
engine 16%, contrib 13%, core 10%) — i.e. payload, not dependencies.

**Settled lessons (do not retry):**

* Generic `T: Into<X>` helpers GROW the binary via per-call-site
  monomorphization
  ([[wisdom_helper_monomorphization_trap]]). Only concrete-value
  helpers shrink.
* Data-drive helpers need ≥5 dominant call-sites per file to
  net-shrink ([[wisdom_data_drive_min_call_sites]]).
* Helpers needing complex option structures (e.g. textcomp's
  `bounded => true, font => { encoding => "TS1" }`) cross the
  ergonomics-vs-savings line.

`panic = "abort"` is `maxperf`-only (NOT release — `cortex_worker`
per-paper isolation needs unwinding). Distribution build recipe:
`cargo build --no-default-features --profile maxperf --bin latexml_oxide`.

---

## Historical rounds (archived to git log)

Detailed narratives for Round-26 (100K warning subset, 99.44% close),
Round-27 (220-paper classified-cluster cohort, all clusters A–G
closed), Round-34 (surpass-Perl content-preservation pass), and
Round-35 (16-paper Canvas-3 failure sprint, R35.A safety nets +
R35.B/C/D investigations + R35.F stage-22/23 cluster) have been
folded into commit history. Run `git log --grep=Round-26 --oneline`
(or `R27`, `R35`, `R35\.F`) to recover the per-commit story when
needed.
