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

## Active mission (Round-37, opened 2026-05-26): 1,000,000 error-free conversions on the arXiv "warning" corpus

> **⚠ METHODOLOGY CORRECTION (2026-05-29) — Perl-gating path.** For most of the
> 2026-05-28/29 sessions, Perl parity runs used the WRONG `--path`
> (`~/git/ar5iv-bindings`, the PARENT) instead of `~/git/ar5iv-bindings/bindings`.
> With the wrong path Perl **silently fails to load `ar5iv.sty.ltxml`**
> (`Can't find package ar5iv`), so `INCLUDE_STYLES` never turns on and Perl
> **does NOT raw-load** un-bound `.sty`/`.cls` packages — it reports them
> "missing" and skips them, appearing falsely CLEAN. This produced
> **false "Perl-clean" verdicts** on every candidate whose failure came from
> raw-loading a package Perl couldn't find. The memory
> [[feedback_perl_parity_options]] already specifies the correct
> `.../bindings` path — it was not followed. **ALWAYS use
> `--path=$HOME/git/ar5iv-bindings/bindings --preload=ar5iv.sty`.**
>
> Re-gated all 8 session fix-witnesses with the CORRECT path:
> * **6 GENUINE Rust-only wins** (Perl clean, Rust was failing → now fixed):
>   2007.04819 (`\?`), 1911.07001 (`\@classoptionslist`), 2006.10240 (babel
>   `strings`), 2006.06087 (elsart `\note`), 2004.07710 (`\preitem@par`),
>   2002.09766 (algorithm2e env names).
> * **2 were SHARED — Rust now SURPASSES Perl** (Perl ALSO fails; my fix handles
>   valid TeX that Perl mishandles): 2006.02269 (pack_parameters halign `#` —
>   Perl 2 errors; this is exactly KNOWN_PERL_ERRORS item 1, a sanctioned
>   beneficial divergence) and 1910.09629 (hyperref `\url` active-`"` — Perl 5
>   errors; URLs-are-verbatim neutralization matches real-LaTeX url robustness).
>   Both pass `cargo test 1344/0` and are faithful to valid TeX — KEPT, but
>   re-labeled here as beneficial divergences, NOT Perl-clean wins.
> * The deferred "META-pattern" candidates (fontaxes 2005.05941, betababel
>   2003.05608, pstricks 1910.10243, mdwmath 2008.05168) are **SHARED** with the
>   correct path (Perl fails identically) — NOT Rust-only. They are real
>   parity-gap / beyond-Perl raw-load-robustness work, but not "wins to claim".

**Status.** Round-36 closed via PR #238 (merged as `9723f4f242`) —
500K first-batch at 99.9968% projected. Round-37 continues on
`large-scale-testing-round-4` branch: drive stages 51-100 (second
500K) and address remaining 5 deep Rust-only failures.

**2026-05-29 state.** 8 faithful fixes landed this session (6 genuine
Rust-only conversions + 2 valid Rust-surpasses-Perl divergences — see the
correction box above), all at `cargo test 1344/0`. After fixing the
Perl-gating path, corrected scans show the binary is at **high parity**:
genuine Rust-only failures are now *rare* (~0; a 117-paper correct-path scan
found only SHARED). The stale `resweep_fresh.tsv` (err=1..10) is exhausted for
clean single-root wins. Pivoted to a **release-binary large scan** (4000+ fresh
papers, correct path + largest-`\begin{document}` main-detection) to surface
the rare remaining genuine Rust-only candidates efficiently; low-error
Rust-failures (nerr 1-3) are Perl-gated as the likeliest genuine engine bugs.
One confirmed-genuine DEEP residual: 1911.01815 (listing/verbatim inside
`\hbox`/`\colorbox` — `\lx@algo@endline` closes a listingline over the hbox's
open `_noautoclose` text; whatsit-construction-order divergence; non-fatal).

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
* **On disk.** Both 500K subsets present in
  `~/data/large_scale_canvas_3/data/arxmliv/`.
* **First 500K (canvas_3 stages 01–50)** DONE — see Round-36 section.
* **Second 500K (canvas_3 stages 51–100)** IN PROGRESS — runner
  `run_stage_second.sh <offset>`; chain scripts at
  `/tmp/chain_stages.sh` (52–60) and `/tmp/chain_61_100.sh` (61–100).
* **OK-output HTML deleted** 2026-05-26 to reclaim disk (saved ~245 GB);
  failed paper IDs preserved at `.session_state/canvas3_failed.txt`
  + `.session_state/wp5_sample_*_failed.txt`. Re-run sandbox is
  the input zips in `~/data/large_scale_canvas_3/data/arxmliv/`.

### Round-37 progress so far (stages 51–55, 50,000 papers)

| Stage | OK | FATAL | Rate | Notes |
|---|---:|---:|---|---|
| 51 | 9996 | 4 | 99.96% | 1501.03690, 1502.06361, 1503.04558 SHARED with Perl; 1503.03906 FATAL_139 was concurrency artifact (re-runs clean, 6.3 MB HTML) |
| 52 | 9998 | 2 | 99.98% | 1503.05439 corpus PDF (not engine); 1504.00185 SHARED with Perl (missing `\cdot` → 101-cap) |
| 53 (v1, killed @1186) | 1186 | 2 FATAL_134 + 0 TIMEOUT | — | 2 stack-overflows in MathML[Content] post (1505.06709, 1505.06978) exposed by deferred-XMath-unlink — fix landed `18fe803244` (cmml depth cap 4096) |
| 53 (v2, complete) | 9928 | 0 FATAL_134, 2 TIMEOUT, 2 FATAL_3 (TooManyErrors) | 99.28% | TIMEOUTs: 1506.02567, 1506.03337(OOM); FATAL_3: 1506.06377/1506.06446 (101-error caps from `_`/`^`-in-text and `\noalign`/`&` cascades — likely SHARED). CONVERR cluster: 145× `_`, 107× `}`, 61× `^`, 33× `&`, 33× XMApp-in-text |
| 54 | 9939 | 1 FATAL_3, 1 TIMEOUT, 1 OOM | 99.39% | OOM (1508.06324) was cyclic-XMRef in cmml — fix landed `81061469fc` (cycle-detection + cap→256); other 2 likely SHARED |
| 55 | 9929 | 1 FATAL_3 (1510.03740), 1 TIMEOUT (1510.04225) | 99.29% | First full stage with cycle-guard binary; 0 stack-overflow, 0 OOM |
| 56 | 9943 | 7 FATAL_3, 4 TIMEOUT, 1 OOM (1511.09288 — `\scalefont Float` param-type bug, fix `56dc9497fc`) | 99.43% | Bisected `\scalefont{0.9}{\hspace…}` runaway-pushback to wrong DefPrimitive arg shape; brace-strip via `{Float}` mirrors Perl `'\scalefont{}'` |
| 57 | 9930 | 1 FATAL_3 (1601.06795, 101× `&`), 0 TIMEOUT, 0 OOM | 99.30% | First stage with scalefont fix; only 1 hard fail (alignment `&` cascade — likely SHARED) |
| 58 | 9930 | 3 FATAL_3, 1 FATAL_134, 1 OOM, 1 TIMEOUT | 99.30% | OOM: 1603.08483 babel/scrextend KOMA `draft=false` error-recovery runaway (deferred); FATAL_134: 1603.07517 XSLT OOM on 10420 maths (deferred); FATAL_3 all likely SHARED `&`/cascade |
| 59 | 9939 | **0 hard fails** | 99.39% | Cleanest stage of Round-37 so far |
| 60 | 9931 | 1 FATAL_3 (1609.00560, likely SHARED), 1 FATAL_1 (1609.01972, corpus-PDF-masquerade — not engine) | 99.31% | Only true engine hard fail = 1× shared `&` cascade |
| 61 | 9935 | 2 FATAL_3 (1609.08897 + 1610.04342, both `_`/`^` cascades) | 99.35% | 0 stack-overflow, 0 OOM, 0 TIMEOUT |
| 62 | 9938 | 4 FATAL_3, 2 OOM (1611.06630 post-after-Timeout 1.5 GB cascade; 1612.04716 xy-pic xymatrix 3.5 GB), 1 TIMEOUT | 99.38% | 1611.06630 = `Fatal:Timeout:Convert` then post-OOM (engine still post-processes timed-out partial); 1612.04716 = xy-pic deep matrix compile; both shared-mode risks |
| 63 | 9927 | 3 FATAL_3, 1 TIMEOUT, 1 FATAL_1 (corpus PDF) | 99.27% | 0 stack-overflow, 0 OOM |
| 64 | 9925 | 1 FATAL_1 (corpus PDF) | 99.25% | **Zero engine hard fails** |
| 65 | 9940 | 1 FATAL_3 (1705.01081), 1 TIMEOUT (1705.01885) | 99.40% | 0 stack-overflow, 0 OOM |
| 66 (v1, killed @7110) | — | hundreds of FATAL_1 (disk full) | — | DISK FULL on 1.9TB filesystem at stage_66 paper ~3500; OK outputs (~8 GB/stage × 15 = ~120 GB) had accumulated. Cleared OK outputs from stages 51-65 (`canvas3_round37_failed.txt` saved), restarted stage_66 |
| 66 (v2) | 9927 | 1 FATAL_134 (1706.06621 — deterministic math-parser abort at math 374; deferred), 2 FATAL_3, 1 TIMEOUT | 99.27% | OK outputs auto-purged after stage |
| 67 | 9943 | 1 TIMEOUT, 1 OOM (1708.06009 — second xy-pic xymatrix 12x11 OOM after 1612.04716), 1 FATAL_3 | 99.43% | xy-pic xymatrix-deep cluster confirmed |
| 68 | 9934 | 4 FATAL_3 (incl. 1711.02043 SHARED PushbackLimit) | 99.34% | 0 OOM/TIMEOUT/SO |
| 69 | 9932 | 1 FATAL_3 | 99.32% | 0 OOM/TIMEOUT/SO |
| 70 | 9932 | 4 FATAL_3 (incl. 1802.02070 revtex4-1 known SHARED) | 99.32% | 0 OOM/SO |
| 71 | 9931 | 2 FATAL_1 (corpus PDFs), 2 FATAL_3 | 99.31% | 0 OOM/TIMEOUT/SO |
| 72 | 9929 | 2 FATAL_3 | 99.29% | 0 OOM/TIMEOUT/SO |
| 73 | 9937 | 2 FATAL_3 | 99.37% | 0 OOM/TIMEOUT/SO |
| 74 (killed @4819) | 4786/4819 | 1 FATAL_3 (real); 5181 FATAL_127 (SIGKILL aftermath, not real) | 99.32% (excl. SIGKILL) | Stage killed during disk-cleanup pivot; uncounted papers go to remaining list |
| **Combined (real attempts)** | **229490/231222** | **73 hard / ~1330 CONVERR** | **99.25%** | **231K papers; mission switched to remaining-list canvas** |

### Remaining-list canvas (Round-37 phase 2)

After stage_74 cleanup, switched from raw-master slicing to processing
the **270,510-paper remaining list** at
`.session_state/canvas3_round37_remaining.txt`. The remaining list is
exactly `master_500K \ ok_ids` — every paper not yet converted to a
clean HTML in stages 51-74. Stages named `stage_R<NN>` (NN=01-28).
Runner: `canvas/run_stage_remaining.sh <offset>`. The remaining list
includes:

* ~7K real failures from stages 51-74 (CONVERR, FATAL_3, TIMEOUT, OOM)
* ~5.2K from stage_74's SIGKILL aftermath
* ~3.6K from stage_52's never-processed slice
* ~255K from stages 75-100 (un-touched papers)

Progress files preserved at `.session_state/`:
  * `canvas3_round37_progress.txt` — per-stage summary
  * `canvas3_round37_ok_ids.txt` — 229,490 papers not to redo
  * `canvas3_round37_done_ids.txt` — every paper any stage touched
  * `canvas3_round37_remaining.txt` — 270,510 to process

| Stage | OK | Hard fails | Rate | Notes |
|---|---:|---:|---|---|
| R01 | 8410/10000 | ~65 (FATAL_3/TIMEOUT — most are SHARED retries) | 84.1% | Dense-failure-front: retries of stages 51-74 known fails + ~5K stage_74 SIGKILL aftermath. Climbed from ~70% to 84% within slice as we entered fresh papers in mid-stage |
| R02 | 9931/10000 | ~6 (FATAL_3/TIMEOUT) | 99.31% | Back to typical rate; dense-failure-front cleared in R01 |
| R03 | 9945/10000 | 1 FATAL_3, 1 FATAL_1 (corpus PDF) | 99.45% | 0 OOM/TIMEOUT/SO |
| R04 | 9916/10000 | 2 FATAL_3, 1 FATAL_139 (1901.10171, 127s before SEGV — concurrency artifact per #232 notes) | 99.16% | 0 OOM/TIMEOUT |
| R05 | 9941/10000 | 1 FATAL_3, 1 TIMEOUT, 1 FATAL_139 | 99.41% | 0 OOM |
| R06 | 9946/10000 | 1 FATAL_3, 1 FATAL_1 (corpus PDF), 1 TIMEOUT | 99.46% | 0 OOM/SO |
| R07 | 9934/10000 | 1 TIMEOUT (1905.07341) | 99.34% | 0 OOM/SO/FATAL_3 |
| R08 | 9916/10000 | 4 FATAL_3 | 99.16% | 0 OOM/TIMEOUT/SO. **Disk full alert resolved**: discovered `/tmp/cortex_output_<pid>.zip` leak in cortex_worker standalone mode (947K files, 685 GB). Fixed `e522358d8f` — `fs::remove_file(&result_path)` after consuming. R09+ uses leak-free binary |
| R09 | 9935/10000 | 1 TIMEOUT (1908.05420) | 99.35% | 0 OOM/SO/FATAL_3. **yfonts fix** (`af19245b58`): `\textfrak`/`\textswab`/`\textgoth`/`\textinit` now defined in the binding (both Perl and Rust binding skipped them in favour of raw-load); witness 1907.06086 CONVERR_1→OK |
| R10 | 9928/10000 | 2 FATAL_3, 1 FATAL_134 (1910.03312 — deep math-parser abort at math 11550), 1 TIMEOUT, 1 OOM, 1 TIMEOUT | 99.28% | Per-paper bisect produced 3 fixes this session: yfonts text-font commands; epstopdf `\epstopdfDeclareGraphicsRule`/`\epstopdfcall` no-ops (`ea4b5c2f13`); babel-spanish trig aliases `\sen`/`\tg`/`\cotg`/`\arcsen`/etc. (`3f3f62fdf2`); listings aspect machinery `\lst@RequireAspects`/`\lst@EndWriteFile`/`\lstKV@OptArg` (`b63e1c73f0`) reducing showexpl-papers CONVERR_7→CONVERR_3 |
| R11 | 9943/10000 | 2 FATAL_3, 3 TIMEOUT | 99.43% | 5 more session fixes: babel-english variants `\dateUSenglish`/`\captionsenglish`/etc. (`9deebb239e`), inputenc `\@inpenc@test` (`38a1fdcb70`), epstopdf `\OutputFile` (`eee60929b9`), KOMA `\headmark`/`\pagemark` (`89b84ffb5a`), caption internals `\DeclareCaptionOptionNoValue` + `\SetCaptionDefault` + `\caption@ifundefined`/`\caption@ExecuteOptions` (`3e17ce9735`) |
| R12 | 9937/10000 | 60 (CONVERR + 2 FATAL_3 + 1 FATAL_1 + 2 TIMEOUT) | 99.37% | 3 more session fixes during R12 run: tikz-timing.sty no-op stub matching Perl missing-file behavior (`676be9cf53`, 8 papers cleaned); caption3 bootstrap chain `\caption@SetupOptions`/`\caption@ProcessOptions`/`\caption@IfPackageLoaded` (`85f8c87e96`, 4 of 5 papers cleaned); ctable.sty no-op stub matching Perl missing-file (`56e018b648`, 6 papers cleaned — none invoke `\ctable` in body) |
| R13 | 9938/10000 | 62 (CONVERR + 5 FATAL_3 + 5 TIMEOUT) | 99.38% | 5 more session fixes during R13 run: babel `\shorthandoff`/`\shorthandon` no-ops (`7099448f93`, 6 papers); typearea.sty no-op stub + `\areaset` (`69aa20604f`, 3 papers — scrbase `unknown option` cluster); ctable deps fix pulling in booktabs/array/tabularx etc. (`8fb3915f0c`, 4 papers — `\toprule`/`\midrule`/`\bottomrule` via transitive dep); expl3 `\hbox_unpack_clear:N`→`\hbox_unpack_drop:N` deprecated alias (`ae90d88ec8`, 8 papers — mmacells.sty); tocbibind all 5 `\if@dotoc*` conditionals (`fae578be43`, 1 paper); mdframed `\newmdenv`/`\renewmdenv` faithful definer (`473cd8af66`, surpass-Perl, witness 2002.06879) |
| R14 | 9955/10000 | 45 (CONVERR + 2 FATAL_3 + 1 TIMEOUT) | 99.55% | 6 more session fixes during R14 run: showexpl.sty stub w/ real deps + no-op API (`2e57ac693a`, 15 papers — `\SX@put@code@result`); mdpi.cls deps natbib/multirow/tabularx/makecell/colortbl + `\tablesize`/`\fulllength`/`\endnote` (`e31810aaf1`, witness 2003.10420); vntex.sty→T5 Vietnamese encoding (`96aec2dfc8`, 3 papers — `\ecircumflex`/`\h`); **constants.sty no-op stub — 70-paper cluster** (`0302a3292c`, raw `\input\jobname.aux` with no runtime `\@mainaux`); amsmath `\tagform@` faithful surpass-Perl (`8710ae735a`, witness 2004.10115); physics `\dmat`/`\admat` token-level split (`9e5ab794e1`, witness 2004.07845 — `\vbh`/`\tildeN` from string round-trip). 3 SHARED-FAILUREs logged (2003.13371/2004.03095/2003.12614). |

### OmniBus class stubs: a TOLERATED SHORTCUT, not a refactor target (task #273, refined 2026-05-28)

**Decisive audit finding.** All 51 `_cls.rs` files doing
`LoadClass!("OmniBus")` are for classes Perl LaTeXML has **no binding
for** (zero `*.cls.ltxml` matches). Perl handles every one via its
automatic fallback (`Package.pm:LoadClass` L2700-2716): warn
`missing_file` → load OmniBus → `maybeRequireDependencies($class,'cls')`
(dep-scan the raw `.cls`, load each `\RequirePackage`/`\usepackage`
binding). Rust mirrors this exactly in `binding/content.rs::load_class`
(L1962-2067). So a hand-rolled stub that just does `LoadClass!("OmniBus")`
is functionally what Rust does anyway *without* the file — except
registering the stub SKIPS the raw-`.cls` dep-scan (the
`<name>.cls.ltxml_loaded` flag short-circuits L2009), usually a
regression vs. the fallback.

**User guidance (2026-05-28, refined — supersedes the earlier
"switch every stub to article + support" plan).** Codifying
"no binding → OmniBus stub" is a **shortcut**: OK to lean on today, NOT
acceptable long-term. Converting the stubs to `LoadClass!("article")` +
hand-derived specifics is *also* a shortcut (still a hand-rolled binding
for a class Perl has no binding for). **The principled fix: add NO new
binding files; improve the raw interpretation of reading the original
`.sty`/`.cls`** so the automatic OmniBus+dep-scan+raw-read fallback works.

**What this means concretely:**
  * **Do NOT** build a `journal_support` mega-helper (it entrenches the
    shortcut). Plan cancelled.
  * The svproc→`article`+`sv_support` conversion (`ce6ecb16c7`) stays —
    `sv_support` is a *real* Perl support pkg — but it is NOT a template
    to replicate across the other 50 stubs.
  * Existing OmniBus stubs: tolerated as-is short-term. Bounded
    de-risking is fine (e.g. dropping eager `RequirePackage!("amsthm")`,
    which breaks `\let\proof\relax`+`\usepackage{amsthm}`: the paper's
    load no-ops → `\proof` stays `\relax` → `{proof}` undefined; witness
    1707.03222 svproc, 1612.03054 imsart, both clean in Perl. svproc +
    imsart already fixed, `10e819ea1b`; ~38 more stubs still carry it).
  * For a NEW class-related error: prefer avoiding a stub — fix the raw
    `.cls`/`.sty` read path so the fallback covers it. Keep/extend a stub
    only when raw interpretation genuinely can't yet.

See WISDOM #55 for the full rationale. Long-term north star: shrink the
51-stub set by making raw `.cls`/`.sty` interpretation robust enough that
the automatic fallback subsumes each one.

### stage_R15 batch triage (2026-05-28) — 2 Rust-only DEEP candidates isolated

Re-tested stage_R15 CONVERR_1 + Perl-gated (Perl as ground truth). Recovered
(stale, now 0-err): 2005.01533, 03899, 06712, 04818. **SHARED** (Perl errors
identically — skip): 2005.04134 (`svg:g isn't allowed in <ltx:block>`),
2005.07432 (`_` math-mode), 2005.07785 (`}` group-mode), plus 2006.13706
(`\SetCustomStyle` — paper `\renewcommand`s 3 undefined glossaries custom-style
cmds), 2006.10842 (`\AR` Arabic — babel[arabic] w/o arabtex), 2005.05903
(`\endkeywords` — OmniBus section-hook mode quirk, Rust `\keywords` sub is
byte-identical to Perl `auto_keywords`).

**Two GENUINE Rust-only candidates (Perl 0 errors), both DEEP — next session:**
* **2005.06787 — xint raw-load `readBalanced ran out of input`.** Minimal
  repro: `\usepackage{xintexpr}` → Rust 1 error (during `xinttrig.sty`
  raw-load), Perl 0. NUANCE: Perl reports `xintexpr.sty` as a MISSING file and
  SKIPS xint entirely (it's a `tex/generic/` package Perl's raw-load doesn't
  pull in); Rust DOES find+raw-load it and then hits the bug in xinttrig.sty's
  catcode-heavy block (`\catcode61\catcode48\catcode32=10` idiom, `~`-as-escape
  shorthands `~expanded`/`~unexpanded`, `\xintdefvar @Pi := float(...)`
  multi-line exprs, `\XINT_tmpa#1#2#3.#4.` delimited defs). Fail is EARLY
  (right after "Processing definitions xinttrig.sty"), locator "Anonymous
  String line 2". Likely a catcode/tokenizer divergence on xint's special
  chars. Faithful fix = the readBalanced root cause (NOT skipping xint).
  **Traced (LXML_RB at gullet.rs readBalanced):** the accumulated tokens are a
  letter-string `{ $noexpand$expanded { $noexpand$unexpanded { … ` — i.e.
  xinttrig's `~expanded`/`~unexpanded`/`~expandafter` shorthands (xint uses `~`
  as a placeholder-escape, materialised later via `\scantokens` with `~`→
  catcode-0 and `\escapechar` set to `$`, so `\string`/`\detokenize` of a CS
  prints `$cs`) are being mis-expanded into literal letters with UNBALANCED
  braces instead of the live `\expanded` primitive. So the gap is Rust's
  handling of xint's `~`-escape + `\escapechar='$'` + `\scantokens`/
  `\detokenize` build-then-rescan idiom (`\XINT_tmpa#1#2#3.#4.` defs, L118+).
  Deep xint-specific tokenizer interaction.
* **2005.04851 — pgfplots/tikz `grid style=dashed` → `_` in math mode.** Perl
  loads tikz + converts clean (0 err); Rust 1 error `Script _ can only appear
  in math mode` at "Anonymous String" + `Warn \tikz@dashphase is not a
  register`. pgfplots dashed-grid dash-pattern rendering. Deep tikz/pgf
  internals.

### R-stage stale-data re-run + cluster triage (2026-05-28, cont.)

* **R01 stage was STALE (pre-stub binary).** R01 (`stage_R01`) showed an
  anomalous 1590 non-OK vs ~60/stage for R02–R17. Cause: R01 ran with an
  older release `cortex_worker` (before the R12 ctable / R13 deps stubs).
  Re-ran all 1590 R01 non-OK papers with the current release binary:
  **289 recovered to OK**, 1225 CONVERR (output produced), 60 FATAL, 16
  TIMEOUT. Lesson reaffirmed ([[project_canvas_stage_v6_recovery]]):
  re-test stale stage data with the current binary BEFORE investigating.
* **ctable cluster ALREADY resolved.** The "181 CONVERR_1 `Package ctable
  Error: You must load ctable after tikz`" finding was entirely stale-R01
  data. Re-ran 30 ctable papers with current binary → **29 OK, 0 ctable
  errors** (1 CONVERR_23 with SHARED errors). Confirmed the R12 ctable
  stub (`56e018b648`) handles them; **0 of 181 invoke `\ctable`**, so the
  no-op stub costs no content. No work needed.
* **FATAL_3 `_`/`^`-in-text cluster is 100% SHARED.** Batch-ran Perl
  (ar5iv flags) on all 30 FATAL_3 papers whose first error is
  `unexpected:_/^ Script … can only appear in math mode`. **All 30: Perl
  rc=1, 101 errors, 0 bytes** — identical failure (malformed source, e.g.
  1502.06361 paoli.tex has stray `}` / unbalanced math around `example`
  envs). Not Rust-only. The one Rust-vs-Perl diff is locator quality
  (Rust "Anonymous String" vs Perl "paoli.tex; line 598").
* **SHARED clusters confirmed 2026-05-28 (NOT Rust-only — do not chase):**
  `malformed:ltx:XMApp "isn't allowed in <ltx:emph>"` (2007.01660 → Perl
  also 1 error, same line 315); `unexpected:\endproof Attempt to end mode`
  (2007.07553 → Perl 11 errors, paper redefines `\proof` brokenly);
  `malformed:ltx:p Attempt to close </ltx:p>` (1804.10191 → Perl also 1
  error, same line 229). All three are source-structure / content-model
  issues both engines reject identically; Rust often has FEWER errors than
  Perl on these. Recorded so they're skipped in future triage.
* **FRESH RE-SWEEP DONE (2026-05-28) — CONVERR_1 corpus is at near-parity.**
  Re-ran the recent-stage CONVERR_1 IDs with the current release binary
  (`/tmp/resweep.sh`): **~51% already recovered to 0 errors (stale**, fixed
  by this session + prior). Of the genuine residual, clustering the current
  first-errors shows it is **dominated by SHARED cases** (Perl errors
  identically, often MORE than Rust): `unexpected:}`/`_`/`&`/`^` (malformed
  source), `\GenericError` (moot vendor errors, WISDOM #50),
  `malformed:ltx:p`/`ltx:XMApp` (verified SHARED), `\gtrless`/`\definecolor`
  (paper omits the defining package — Perl also errors), plus deferred
  ar5iv-specific `\autrun`/`\crvi`/`\dq`/`{diagram}`. **Lesson: trust the
  re-sweep, not the stale stage logs.**
* **PRIMARY remaining GENUINE Rust-only cluster (NOT fixed — delicate core,
  next session): `\endgroup` mode-switch frame leak (~17 papers).**
  `Error:unexpected:\endgroup Attempt to close a group that switched to mode
  restricted_horizontal … due to T_CS[<env>]` (stomach.rs:478). Confirmed
  Rust-only (1505.07999 → Perl rc=0). The leaked mode-switch (BOUND_MODE)
  frames come from ENVIRONMENTS — theorem-like (`\proof`/`\thm`/`\lem`/
  `\remark`/`\step`/`\IEEEproof`), `\microtypecontext`, and `pspicture`.
  Critically, Perl's `Core/Stomach.pm` egroup/endgroup is BYTE-IDENTICAL to
  Rust's (same BOUND_MODE error) — so the bug is NOT egroup; **Rust leaks an
  env's mode-switch frame upstream where Perl balances/pops it.** Fix at the
  env push/pop site (NOT by blanket-suppressing egroup). pspicture also has
  a signature mismatch (Rust `[]{}`/`[][]` vs Perl `PSCoord`). High blast
  radius (a 2026-04-25 band-aid `3088dbd17` already suppresses the strict
  check during raw load). Full analysis + reproducer:
  [[project_endgroup_modeswitch_frame_leak]].
  * **FIX LANDED (2026-05-28, round 4) — `\rput`/`\cput` delimited-`(`
    runaway.** The active `\rput` was the RawTeX redef
    (`\def\lx@rput@parens(#1)#2{}` + `\@ifnextchar[`), whose **delimited
    `(#1)` parameter** requires a literal `(`. For the braced-angle/no-coords
    form `\rput{angle}{body}` there is no `(`, so TeX SCANNED FORWARD eating
    tokens — including `\end{pspicture}` — until the next `(` anywhere later.
    That swallowed the env end, so pspicture's `end_mode` never fired and its
    mode-switch frame leaked → the later `\endgroup` (from `\end{proof}`)
    tripped the BOUND_MODE check. (Confirmed via mode-frame instrumentation:
    pspicture `begin_mode` 11× vs `end_mode` 10×.) Fix: replaced the
    delimited-`(` defs with a runaway-safe `\@ifnextchar(` gobbler shared by
    `\rput`/`\cput` (PEEK for `(` instead of requiring it; Perl handles this
    via `OptionalBracketed`+`ZeroPSCoord`). Flips **1505.07999** → rc=0,
    0 errors, 1.73 MB HTML. `cargo test --tests`: 1344 passed, 0 failed.
    Body still dropped (the <ltx:p>-cascade workaround); faithful
    `<ltx:g>`-with-body (port `\rput@start`/`\put@end`/`<ltx:picture>`)
    remains a TODO.
  * **Cluster now at NEAR-PARITY (Perl comparison, round 5).** Of the ~12
    remaining `\endgroup` papers, MOST are **SHARED** — Perl errors
    identically (1510.07020 `\IEEEproof` Perl 13; 1709.00807 & 1612.02968
    same `\endgroup`; 1611.05278 Perl 101; 1512.03809 Perl 2; 1610.05482/
    1611.04940/1702.02037/1702.06692 Perl ≥1). The ONE clearly Rust-only
    case left is **1606.03691** (`amsart` + bare-used `\newtheorem{rem}`:
    `\rem text` w/o `\end{rem}` → unclosed theorem's internal_vertical mode
    frame leaks to `\end{sloppypar}`'s `\endgroup`; Perl rc=0). Root cause:
    XML `Tag autoClose` closes `<ltx:theorem>` but the STOMACH `begin_mode`
    frame isn't popped when an open env is auto-closed by a sibling/enclosing
    block (`\begin{thebibliography}`). Delicate (touches all theorem/list
    envs) + 1-paper malformed-usage edge → deferred (needs Perl-tracing of
    where the rem frame is popped). Memory has the full analysis.
* **FIX LANDED — `{keywords} environment is not defined` (fundam.cls
  cluster) by DELETING the `fundam_cls.rs` stub (Perl-faithful).** The
  earlier characterization was wrong on the root cause: there WAS a
  `fundam_cls.rs` binding (contrib lib.rs), a hand-rolled stub doing
  `LoadClass!("article")` + amsmath/amssymb/amsthm/fancyhdr/xcolor/
  hyperref + `\publyear`/`\papernumber`/`\volume`/`\issue`/`\runninghead`/
  theorem-envs — but it OMITTED `\keywords`, and its `article` base (vs.
  OmniBus) means no generic `{keywords}` env → `\begin{keywords}` errors.
  Instrumented `load_class`: `is_binding=true` (the stub), so `will_fallback`
  is false and OmniBus never loads. **Perl has NO fundam binding** → it
  falls back to OmniBus (`Warn:missing_file:fundam … using OmniBus`),
  whose generic `{keywords}` env resolves the env (verified: Perl rc=0,
  159 KB). Per user guidance (2026-05-28: no new binding files, OmniBus is
  the last-resort fallback, Perl is ground truth) the fix is to **delete
  the stub** so Rust falls back to OmniBus exactly like Perl. The stub's
  `\publyear`/`\papernumber` were *papering over a SHARED Perl limitation*
  — Perl ALSO errors `undefined:\publyear`/`\papernumber` under OmniBus
  (verified via minimal probe), so per [[feedback_no_papering]] they must
  not be Rust-only-defined. Flips **all 9 local cluster papers** (1810.10529,
  1901.04983 [fundam-stef], 1901.08246, 1904.07445, 1904.07480
  [fundam-arxiv], 1906.04897, 1911.05801, 1911.07591, 2005.04818) → rc=0,
  **0 errors** (only the `missing_file:fundam` warning, matching Perl);
  1810.10529 → 188 KB HTML with keywords classification + issue note
  preserved. Also auto-fixes the `fundam-stef`/`fundam-arxiv` *variants*
  that previously prefix-matched the stub. `cargo test --tests` green. This
  is the first concrete win of the task #273 north star (shrink the
  OmniBus-stub set via the Perl-faithful fallback, not new bindings).
  Memory: [[project_keywords_env_binding_less_cls]] (now resolved),
  [[feedback_raw_interpretation_over_bindings]].
* **FIX LANDED — `\bysame` undefined (mcom-l/proc-l/tran-l) by DELETING
  the `mcom_l_cls.rs` stub (same fundam pattern).** mcom-l.cls (AMS journal
  letters class) does `\LoadClass{amsart}` (L42); the stub instead did
  `LoadClass!("OmniBus")` + amsmath/amsthm/… + hand-rolled AMS frontmatter
  macros — but NOT `ams_support`, so `\bysame` (ams_support.sty.ltxml L215)
  was undefined. Perl has no mcom-l binding → OmniBus fallback + dep-scan
  finds `\LoadClass{amsart}` → loads amsart → ams_support → `\bysame`
  (verified: Perl rc=0, 0 errors, 812 KB). Deleted `mcom_l_cls.rs` + its 3
  registrations (mcom-l/proc-l/tran-l) so Rust falls back identically;
  amsart/ams_support now also cover the stub's hand-rolled macros
  (`\commby`/`\copyrightinfo`/`\subjclass`/… all in ams_support). Flips
  **1706.00540** → rc=0, 0 errors, 400 KB HTML (25-bibitem bibliography);
  the multi-error mcom-l papers (1608.08766 CONVERR_23, 1707.04919 _27,
  2006.16729 _11) are unchanged (other SHARED issues, not regressed).
  `cargo test --tests`: 1344 passed, 0 failed.
* **FIX LANDED — `\bysame` undefined (birkjour) by DELETING the
  `birkjour_cls.rs` stub (autoload-shadowing root cause).** birkjour papers
  (1503.01760, 1904.09833, CONVERR_1 `\bysame`) use `birkjour.cls`
  (amstex-based). The stub did `LoadClass!("OmniBus")` + amsmath/amsthm/…
  AND hand-rolled `\subjclass{}` as a frontmatter macro. That hand-rolled
  `\subjclass` **shadowed OmniBus's lazy `\subjclass`→ams_support autoload**
  (omnibus_cls.rs L557-558), so the paper's `\subjclass` only added
  frontmatter and never triggered `ams_support` → `\bysame` stayed
  undefined. Perl has no birkjour binding → OmniBus fallback (autoload
  intact) → paper's `\subjclass` loads ams_support → `\bysame` (verified:
  bare-birkjour probe ALSO errors in Perl; the full paper is clean ONLY
  because `\subjclass` fires the autoload). Deleted `birkjour_cls.rs` + its
  registration → OmniBus's autoload restored. Flips **1503.01760 +
  1904.09833** → rc=0, 0 errors (1503.01760 → 152 KB HTML, 17 bibitems,
  `ams_support` loads via the `\subjclass` autoload). General lesson
  (WISDOM #55): a stub that hand-rolls a CS which OmniBus uses as a lazy
  AUTOLOAD TRIGGER (`\subjclass`/`\curraddr`→ams_support, `\citet`→natbib,
  `\begin{theorem}`→amsthm, `\mathfrak`→amsfonts) **breaks the autoload
  chain** — a strong reason to delete these stubs rather than extend them.
* **FOLLOW-ON (same AMS-family pattern, NOT yet fixed): `conm-p-l`.**
  1603.00667 (`\copyrightinfo`) uses conm-p-l (Contemporary Math
  proceedings); it has NO Rust stub (already OmniBus-fallback), so the
  `\copyrightinfo` gap is a different shape — `\copyrightinfo` is an
  ams_support macro, so it likely needs an OmniBus lazy-autoload trigger
  for `\copyrightinfo` (like `\subjclass`), OR the paper doesn't use
  `\subjclass`/`\curraddr` to trigger ams_support. Needs its own
  Perl-ground-truth check — deferred.
* **FIX LANDED — undefined counter-register `\c@<ctr>` read is an ERROR in
  Rust but a WARNING in Perl (general engine-faithfulness fix).** When code
  reads an undefined counter register in a number context (e.g.
  `\algrestore`/`\ContinuedFloat` → `\c@subalgorithm@save`, or tikz-timing
  → `\c@tikztimingtrans`), Rust's `read_x_token` expanded the bare undefined
  `\c@<ctr>` through `state::generate_error_stub`'s generic
  `<ltx:ERROR/>` path → 1 spurious error. Perl never errors here: its
  counter machinery warns "Counter '<ctr>' was not defined; assuming 0"
  (Package.pm L712) and treats it as 0 (verified: 1910.02851 Perl rc=0,
  0 errors; bare-counter probe warns only). `\c@<ctr>` is, by LaTeX
  convention, ALWAYS the count register backing counter `<ctr>`, so an
  undefined one is unambiguously "counter not defined". Fix: in
  `generate_error_stub`, special-case `\c@<ctr>` early — warn (same
  category/message as `counter::dialect::counter_value`) and define it as a
  count register 0, then return — instead of the hard undefined-CS error.
  General (not a stub): fixes ANY undefined-`\c@` read. Flips the
  `\c@subalgorithm@save` cluster (1711.05152, 1809.10982, 1810.07730,
  1904.07131, 1910.02851) AND the `\c@tikztimingtrans` cluster (1807.08647,
  1912.11312, …) → rc=0, 0 errors (1910.02851 → 735 KB HTML, 71 algorithm
  blocks). `cargo test --tests`: 1344 passed, 0 failed.
* **FIX LANDED — `\DeclareMathOperator` undefined → 101-error FATAL
  (myclass test-fixture name collision).** Papers using `\documentclass
  {myclass}` (a common tutorial/template name) that bundle their OWN
  myclass.cls (`\usepackage{amsmath}` + `\LoadClass{article}`) FATAL'd: the
  Rust contrib test fixture `myclass_cls.rs` was registered GLOBALLY under
  the literal name `myclass`, intercepting these real papers. It loads
  article + DeclareOptions but NOT amsmath → `\DeclareMathOperator`
  undefined → every operator cascade → 101-error FATAL. Perl has NO myclass
  binding → OmniBus fallback + dep-scan of the bundled myclass.cls (loads
  amsmath) → clean (rc=0). Fix: re-register the options fixture under a
  deliberately-unique name `lxtestclass` (used only by
  tests/structure/options.tex), so real `myclass` papers fall back to
  OmniBus+dep-scan like Perl. Same class of bug as the fundam/mcom-l/birkjour
  OmniBus-stub interceptions, but for a TEST fixture. Flips **1710.04325 +
  1802.01751** FATAL → rc=0 (1710.04325 → 561 KB HTML). options test still
  passes; `cargo test --tests`: 1344 passed, 0 failed.
  * **Same fix for `mytemplate`** (test fixture for `\hw`, renamed →
    `lxtesttemplate`; tests/contrib/hw.tex). Witness **1810.07512**: bundles
    its own mytemplate.sty defining `\F`/`\eps`/`\sig`/… via `\newcommand`;
    the global `mytemplate` fixture shadowed it → 22 undefined → 101-error
    FATAL. Un-shadowed, Rust RAW-LOADS the bundled mytemplate.sty (under
    INCLUDE_STYLES) → all macros defined → rc=0, 0 errors, 441 KB HTML —
    *surpassing* Perl (Perl dep-scans only, doesn't run the `\newcommand`s →
    19 errors). (Remaining fixtures — apackage, filelistclass, mykeyval,
    myxkeyval, xkvdop* — keep generic names but are far less likely to
    collide; revisit if witnessed.)
* **FATAL triage (2026-05-28): `not_tex_source` cluster = correct PDF-only
  rejection (SHARED).** All 9 `Fatal:invalid:not_tex_source "PDF magic
  detected"` papers (1812.00352, 1503.05439, …) are genuinely arXiv
  PDF-only submissions (the `.tex` IS a PDF — `%PDF-1.4`, 0 TeX content).
  Perl can't convert PDFs either; the worker's rejection is correct. Not a
  target.
* **NEW Rust-only bug ISOLATED (NOT fixed — delicate, broad): robust CS in a
  `\usepackage[…]` brace-group option → infinite-loop FATAL.** Witness
  **2004.08143** (`\usepackage[…,pdfauthor={… Mar{\'\i}n},…]{hyperref}`):
  the `\i` (dotless-i, a `robust` primitive) in the option value loops
  (pushback 650000 → `Fatal:Timeout:PushbackLimit`) so hyperref never loads
  → `\href` undefined → cascade (Rust 4 err + FATAL; Perl rc=0). Minimal
  repro `\usepackage[x={\relax\i}]{hyperref}`. Mechanism: under semiverbatim
  the robust space-form `\i␣` in `\i`'s expansion (`\protect\i␣`) collapses
  back to `\i` (no-space) → `\i`→`\protect\i`→… loop (ITRACE-confirmed).
  `\i` in plain body is fine. Broad impact (accented pdfauthor/pdftitle in
  package options are common). Full analysis + minimal repro + next-step
  (trace `Expandable::invoke` body re-tokenization under semiverbatim):
  [[project_robust_cs_semiverbatim_loop]].
* **FIX LANDED — `\bookmarksetupnext` undefined (bookmark.sty stub gap).**
  Rust deliberately stubs bookmark.sty (raw-load hits the token-limit via
  its driver-file dispatch — documented in `bookmark_sty.rs`), no-opping the
  bookmark public API. But it covered every public macro EXCEPT
  `\bookmarksetupnext` (bookmark.sty L134, `\newcommand*{...}[1]`, sets
  options for the next bookmark — cosmetic PDF-outline, no HTML analogue).
  Perl raw-loads bookmark.sty so it has the macro (Perl rc=0). Added the
  matching `def_macro_noop("\\bookmarksetupnext{}")`. Flips **1707.07002**
  (→ 0 errors, 1.3 MB HTML; residual 1260 warnings are the separate
  `expected:id` cluster) **+ 1902.06453** → rc=0. `cargo test --tests`:
  1344 passed, 0 failed.
* **FIX LANDED — svproc/spie `\cellcolor` undefined (xcolor `table`
  option-clash).** Root cause: `svproc_cls.rs` and `spie_cls.rs` had a
  Rust-only `RequirePackage!("xcolor")` (no options); the real svproc.cls
  /spie.cls do NOT load xcolor (only `\LoadClass…{article}` + kernel
  `\normalcolor`). So a paper's `\usepackage[table]{xcolor}` option-clashed
  against the already-loaded optionless xcolor → `table` dropped → colortbl
  never loaded → `\cellcolor` undefined. Perl raw-loads the class (no xcolor
  preload) so the user's `[table]{xcolor}` is the first load and colortbl
  comes in. Fix: preload xcolor with `[dvipsnames, table]` in both class
  bindings (same sanctioned anti-clash pattern as mnras_cls /
  quantumarticle_cls). Flips **1706.04315 (svproc) + 1807.04749 (spie)**
  → rc=0, 0 errors, HTML produced. (1804.09301 also `\cellcolor` but uses
  `article`+bundled `naaclhlt2018.sty` — separate, not this fix.)
  * **Remaining sub-case (deferred):** the same `[table]{xcolor}` clash also
    fires when an *already-loaded optionless xcolor* comes from a
    paper-bundled `.sty` with no Rust binding (e.g. naaclhlt2018.sty L101
    `\usepackage{xcolor}` loaded before the main file's `[table,xcdraw]{xcolor}`
    → 1804.09301). No class binding to patch. The GENERAL Perl-faithful fix
    is: on a re-`\usepackage` of an already-loaded package with NEW options,
    run those options' `DeclareOption` handlers (Perl reprocesses `table` →
    colortbl; real LaTeX would "option clash" error, but Perl is lenient).
    Broader/riskier engine change to option-clash handling — deferred.
* **FIX LANDED — babel `\dateUSenglish` undefined (english≡USenglish
  canonical date hook).** ar5iv raw-loads babel.sty + the real
  babel-english.ldf, which builds `\date<CurrentOption>` via
  `\@namedef{date\CurrentOption}` — so option `english` creates only
  `\dateenglish`, not the canonical `\dateUSenglish` that modern babel's
  babel-en.ini machinery then calls → `Error:undefined:\dateUSenglish`.
  Plain CLI doesn't hit it (uses the babel binding, not raw-load); only
  ar5iv/INCLUDE_STYLES does. Perl raw-loads the same files yet converts
  clean. Fix: in `english_sty.rs`, after the english.ldf raw-load, alias
  each canonical english-variant date hook (USenglish/UKenglish/american/
  british/canadian/australian/newzealand) to the real `\dateenglish` when
  undefined (`\@for` + `\@ifundefined` guard). Consistent with
  babel_lang_stubs' typesetting-only-date philosophy, but aliasing keeps
  `\today` faithful. Flips **1503.02002, 1608.02901, 1707.06505,
  1808.10359** → rc=0, 0 errors (1503.02002 → 1.08 MB HTML). cargo test
  --tests: 1344 passed, 0 failed.
* **FIX LANDED — rotfloat `\restylefloat` undefined (missing float dep).**
  `rotfloat_sty.rs` was a stub loading only `rotating`, but rotfloat.sty
  L24 does `\RequirePackage{float}` (which defines `\restylefloat`/
  `\newfloat`/`\floatstyle`). Papers call `\restylefloat{figure}` directly
  → undefined without float. The stub's "Perl skips it (INCLUDE_STYLES=
  false)" rationale doesn't hold under the canvas ar5iv profile
  (INCLUDE_STYLES=true → Perl raw-loads rotfloat→float). Fix: add
  `RequirePackage!("float")` (before rotating, matching real load order) —
  loads float's binding (clean, no raw-load `\fi` errors). Flips
  **1604.07054, 1808.04014** (both `\documentclass{mnras}`+rotfloat) →
  rc=0, 0 errors. Perl baseline: rc=0, 485 KB / 605 KB. cargo test
  --tests: 1344 passed, 0 failed.
* **FIX LANDED — pstricks `\Cnode`/`\cnode` undefined (pst-all missing
  pst-node dep + wrong node signatures).** Two-part fix for the
  `\usepackage{pst-all}` + `\Cnode(x,y){name}` cluster (6 papers:
  1509.04932, 1509.06412, 1604.02906, 1604.02908, 1705.00191, 1809.03593).
  (1) `pst_all_sty.rs` did `RequirePackage!("pstricks")` and inlined a few
  nc-* macros but never loaded pst-node — yet pst-all.sty L23 does
  `\RequirePackage{pst-node}`. Added `RequirePackage!("pst-node")` (loads
  our clean binding, not raw .sty). (2) `pst_node_sty.rs` `\cnode[]{}` /
  `\Cnode[]{}` had the WRONG signature — PSTricks node macros take a
  `(coord)` (Perl: `\cnode='ZeroPSCoord {PSDimension} {}'`,
  `\Cnode='ZeroPSCoord {}'`), so the old `[]{}` left `(x,y)` unconsumed
  and `\Cnode(1,1){000}` leaked "1,1)". Fixed to
  `\cnode * [] () {} {}` / `\Cnode * [] () {}`. All 6 → rc=0, 0 errors;
  Perl baseline also rc=0 (1509.04932: 334 KB). cargo test --tests: 1344
  passed, 0 failed. NOTE: triage gotcha — these papers are single-file
  ISO-8859 `.tex`; `grep -l documentclass` treats them as binary and
  returns empty, so always pass the explicit filename to Perl (a bogus
  rc=1/empty-output is a harness artifact, not a Perl failure).
* **FIX LANDED — getfiledate `\gfd@width@tmp` undefined (ltxnew `\new`
  raw-load gap).** getfiledate.sty L20 `\RequirePackage{ltxnew}`, then L29
  `\new\dimen\gfd@width@tmp\gfd@width@tmp=\z@`. ltxnew builds its `\new`
  allocator via a `\futurelet`-based prefix scanner (`\ifx\x\dimen →
  \newdimen` chain) that our raw-load doesn't execute faithfully, so
  `\new\dimen` never allocates the register and the immediate
  `\gfd@width@tmp=\z@` errors at LOAD time (papers never even call
  `\getfiledate`). Perl reports getfiledate as missing-file and only
  deps-scans it (never runs the body) → clean. Fix: new contrib stub
  `getfiledate_sty.rs` — no-op `\getfiledate[]{}` + `RequirePackage!(
  "xcolor"[table])` (getfiledate.sty L22 dep; without it the stub drops
  xcolor and `\textcolor` goes undefined, witness 1803.07118). Flips
  **1503.08338, 1503.08341, 1709.04899, 1803.07118** → rc=0, 0 errors
  (Perl baseline 1503.08338: 3.3 MB). cargo test --tests: 1344 passed, 0
  failed. (Deeper ltxnew `\new` futurelet-scanner fix deferred — Perl
  skips getfiledate anyway, so the stub IS the Perl-faithful match.)
* **FIX LANDED — floatrow "Undefined object setting `centering`/
  `raggedright`" (caption `\caption@hj@*` missing).** floatrow.sty L1169
  probes `\@ifundefined{caption@hj@<name>}` for `objectset=centering`/
  `raggedright` (from `\floatsetup`); the `\caption@hj@<name>` macros are
  created by caption3.sty's `\DeclareCaptionJustification` (L955-969).
  Rust's `caption_sty.rs` stubbed `\DeclareCaptionJustification` as a pure
  no-op → those macros never existed → floatrow's `\flrow@error`. Both
  engines raw-load floatrow (no binding either side); Perl raw-loads
  caption3 which defines the hj macros, so Perl is clean. Fix: make
  `\DeclareCaptionJustification[<pkg>]{<name>}{<body>}` faithfully
  `\@namedef{caption@hj@<name>}{<body>}` (+ `caption@justification@<name>`),
  and seed the 6 standard justifications caption3 declares at load
  (justified/centering/centerfirst/centerlast/raggedleft/raggedright).
  Flips **1504.02564, 1608.07117, 1704.01862, 1708.07230, 1712.06479** →
  rc=0, 0 errors (Perl baseline 1504.02564: 1.7 MB). cargo test --tests:
  1344 passed, 0 failed.
* **FIX LANDED — morefloats "Too many floats requested" (moot float-
  register capacity).** morefloats.sty raises LaTeX's unprocessed-float
  limit; its body (L76+) computes "free" classic-TeX registers as
  `234 - max(\count10,\count11,\count12,\count14)` (L271-274) and errors
  when the requested float count exceeds it. That assumes the pre-eTeX
  256-register pool; our XML pipeline has NO float-box register pool
  (floats → `<ltx:float>`, unlimited), and our `\count10` allocation
  high-water-mark (~214 after the kernel — verified, vs ~30-90 in real
  LaTeX article; possible deeper over-allocation, noted for future) makes
  `234 - \count10` tiny → any `maxfloats=N` trips the cap. Pure
  typesetting-capacity `\PackageError`, moot (WISDOM #50). Perl skips
  morefloats (missing_file, deps-scan only) → clean. morefloats exports NO
  user macro (the 3 papers call none), so a no-op preserves all content.
  Fix: new contrib stub `morefloats_sty.rs` replicating morefloats.sty
  L63-74 (kvoptions option-handling prefix, so `[maxfloats=120]` is
  consumed) but OMITTING the moot capacity body (L76+). Flips
  **1504.06174, 1605.06159, 1607.05324** → rc=0, 0 errors (Perl baseline
  1504.06174: 1.18 MB). cargo test --tests: 1344 passed, 0 failed.
* **DEFERRED — `\crvi` cluster (3 papers: 1603.04650, 1704.02401,
  1804.00017; xy-pic curved arrows).** `\usepackage[all,tips]{xy}` +
  `\ar@/^15pt/[rr]^{...}` (the `@/.../` curve modifier) → `\crvi`
  undefined. `\crvi` is defined in xycurve.tex L69 (`\xydef@\crvi#1#{...}`,
  an advanced `#{`-delimited param). Rust's xy_sty.rs DOES raw-load
  xycurve.tex for `[all]` (and `\crvi` IS defined in a minimal repro), but
  the real papers hit a STATEFUL divergence: a minimal `\ar@/.../` gives
  `Info:xy:error Forms @/.../… only available when curve` (curve module's
  runtime flag not set) while the real papers reach `\crvi` undefined. The
  curve-module activation (`\ifxy@curve@`-style flag) + `\crvi`'s `#{`
  param are the deep issue. Perl raw-loads xycurve.tex and renders curves
  (2.9 MB). Deep xy-pic curve-machinery — deferred.
* **DEFERRED — `\dq` cluster (2 papers: 1602.07073, 1804.06196;
  babel-german double-quote).** `\usepackage[german,english]{babel}` +
  `\dq` → undefined. germanb.ldf L173 `\def\dq{"}`. german_sty.rs ports
  germanb but omits `\dq`. ROOT MYSTERY: adding `\dq` to german_sty.rs
  (any form — DefMacro, early `\gdef\dq{ZZQUOTE}`) does NOT make it stick —
  `\dq` is **actively undefined** after german_sty.rs runs (verified:
  `\captionsgerman`/`\mdqon` survive but a global `\gdef\dq` does not, and
  `\bbl@allowhyphens` (L60, late) is also UNDEF → the load also truncates
  somewhere past L57). Nothing in babel-german.tex/babel.sty/babel_support
  explicitly `\let\dq\@undefined`s it, so the clearer is elsewhere in the
  modern-babel `.ini` activation / `[german,english]` main-lang switch.
  Note 1602.07073 ALSO has 2 Perl errors (`\printbibliography`/biblatex),
  so it's only marginally Rust-only. Deferred — modern-babel state
  management.
* **DEFERRED — `\autrun` cluster (4 papers, ar5iv-specific, elusive).**
  1509.01533/1509.04088/1602.03020/1804.10461 redefine `\author` to set
  `\autrun` as a side-effect (`\def\author#1{\gdef\autrun{...}...}`), then
  use `\autrun` in `\markboth` (via a redefined `\address`). Rust ignores
  the `\def\author` (correctly — `\author` is `locked`, matching Perl
  LaTeX.pool L1210 `locked=>1`), so `\autrun` is never set → undefined
  error. Perl ALSO locks `\author` (so also never sets `\autrun`) yet
  converts clean (3.6 MB) — so it tolerates/gobbles the undefined `\autrun`
  where Rust expands it. The error is **ar5iv-only** (plain CLI clean) and
  needs the FULL real preamble (L1-220) × ar5iv to reproduce — NOT
  reproducible with minimal preamble + exact frontmatter, NOT from
  `\markboth` alone (it's a noop that gobbles). Trigger is a paper-specific
  preamble×ar5iv interaction that redefines `\markboth` or expands the
  stored mark. Deferred — low ROI / hard to isolate.
* **XMRef `expected:id` over-warning: mid-parse suppression is a DEAD END.**
  Tried a non-consuming `data::resolve_lost(id)` consulted in
  `realize_xmnode` before warning — warnings stayed at 9 on the minimal
  split repro because `LOST_NODES` is empty when `realize_xmnode` runs
  (populated only by the end-of-parse sweep). Resolve-to-survivor variant
  also BROKE the conversion (empty 39-byte output). Reverted. The real fix
  remains parser-side id preservation ([[project_xmref_dangling_split]]).

### R19 fixes (2026-05-28)

* **1302.3919 deep perf analysis — SHARED-slow, NOT a Rust-only failure
  (and a `expected:id` over-warning follow-up).** Localized the only
  "genuinely slow" timeout: EMDerivation.tex is math-VERY-heavy (182
  `equation` + **124 `\begin{split}`** + 6 align + 4 gather). The 60 s
  isolation-test failure was the CLI's *default* 60 s wall-clock guard;
  with `--timeout 240` Rust **completes in 119 s → 6.8 MB** — actually
  *faster* than Perl's 137 s. Both exceed the 120 s canvas budget, so it's
  SHARED-slow at the margin (not Rust-only). Phase timeline: Digest+Build
  fast, 4 rewrite rules fast (<70 ms total), then **~112 s in the Marpa
  math-parse** of the 340+ math envs. Rust emits **7009 warnings (4620
  `expected:id` + 2389 `expected:node`) vs Perl's 103** — these are the
  `rearrange_ams_split` dangling-XMRef cascade ([[project_xmref_dangling_split]]):
  `prune_dangling_split_xmrefs` (document.rs finalize) cleans the *output*
  (no post-process Error) but runs AFTER `parse_math`, so the parser's
  `realize_xmnode` (parser.rs:2576) still warns on each ref whose target
  cell it absorbed mid-parse. `--quiet` (suppress warning logging) does NOT
  speed it up (119 s), so the cost is the Marpa parse itself, not the
  warning I/O — the dangling refs can't be pruned *before* parse (cells
  still hold their ids until the parser drops them). **Follow-up (deferred,
  not a failure):** the 4620-warning over-emission is a Rust-only quality
  gap vs Perl; eliminating it needs deep math-parser split-absorption
  changes (memory approach #3, regression risk on declare_test). Marpa
  perf on 300+-math-env docs is the broader limiter; both engines strain
  the 120 s budget here.

* **First-500K canvas failure list (`.session_state/canvas3_failed.txt`,
  168 papers) re-tested: 82 recovered, 86 residual all SHARED.** This is
  the full 150K-run failure list (the `canvas_3_failures_sandbox` was just
  a 16-paper subset). Current binary: **82/168 now convert** (recovered by
  R19 + intervening fixes). The 86 still-failing break down as: **76
  rc=3 TooManyErrors** — dominated by the `_`/`^`-in-text 100-error-cap
  cluster (~36 papers, e.g. 0906.1913/0903.4689/0901.1928; verified SHARED
  — Perl also hits 101 errors + fatal, no output, these are math papers
  with stray `_`/`^` in text), plus stray-`&` (~8, verified SHARED —
  1107.0383 JHEP3: Perl also 101 errors + fatal), `malformed:ltx:XMApp`
  (4, e.g. 1006.5461/1111.1008 — SHARED, Perl fatals), `\displaylines`
  Cluster A (SHARED), and a few singletons; **7 rc=124** timeouts (some
  CPU-contention false positives); **3 rc=1** (`not_tex_source` PDF / empty
  source — correct rejects). Spot-checked 6+ across patterns: every one is
  SHARED (Perl empty/over-cap/hangs). No new Rust-only conversion failure.
  **Timeout (rc=124) triage (isolation re-test of the 7 cases):** 4 are
  CPU-contention false timeouts (0708.3398 21 s, 1009.3622 43 s,
  1210.6239 13 s complete standalone; 1001.3154 = empty input;
  hep-ph0012156 = 12.7k math, slow-but-completes). The 2 genuinely slow
  (>60 s standalone) are SHARED: 1202.2643 (Rust >300 s, but **Perl also
  fails** — 1 fatal at 20 s, no output) and 1302.3919 (Rust >300 s; **Perl
  completes but in 137 s**, itself over the 120 s canvas budget — both
  time out in the canvas). So no clean Rust-only-timeout-where-Perl-
  converts-in-time. The first-500K + round-37 failure space is fully
  triaged: **every residual is SHARED, a degenerate input, or a
  contention artifact — zero actionable Rust-only conversion failures.**
  (Note: 1302.3919 shows Rust ~2× slower than Perl on a math-heavy doc —
  a perf gap, not a correctness bug, and SHARED-slow at the 120 s budget.)

* **Early-years (first-500K) fresh sweep: clean.** A 2491-paper sample
  (every 200th `.zip` across year dirs 0001–1412) found **1 failure**:
  math0506088 (rc=3), which is a known-SHARED Cluster-A `\displaylines`
  `\raise`/`\hbox` recursion (Perl also terminates). So the early-years
  region is as clean as round-37. Cumulative this session: ~13k papers
  sampled (round-37 ~6.6k + failed-list 1164 + sandbox 16 + early-years
  2491) → **all genuine Rust-only conversion failures found are fixed**
  (5 R19 fixes); every fresh-sweep residual is SHARED (Perl also
  empty/over-cap/hangs) or a degenerate/`%auto-ignore` input. `~/data/
  all_warnings.txt` is just a 1.5M paper-PATH list (no messages), not a
  targeted error signal.

* **xy `\CompileMatrices` memory OOM — RESOLVED** (commit `45290a23e7`).
  Investigated the preserved `canvas_3_failures_sandbox` (16 cases from an
  old round-3 binary). Re-test with the current binary: **9 of 16 now
  pass** — Clusters C (extreme-math post-proc), D (rewriting-phase
  timeout) and E (the 3 FATAL_139 "segfaults", which were environmental)
  all recovered by intervening fixes. **Cluster B (xymatrix OOM, 2
  papers: math0203082, math0402448) fixed here:** `\usepackage[all]{xy}` +
  `\CompileMatrices` routed each `\xymatrix` through xy-pic's `.xyc`
  disk-cache compile/re-input cycle (xymatrix.tex L91
  `\let\xymatrix=\xymatrixcompile`); the cycle's unbounded `\global\toks9=`
  accumulation blew RSS past the 4.5 GB budget → `Fatal:Timeout:MemoryBudget`
  (Perl converts to ~5.86 MB). `\CompileMatrices` is a pure TeX-runtime
  speed optimization (output-identical), pointless in single-pass XML — the
  deprecated ar5iv binding likewise `DefMacro('\CompileMatrices','')`.
  No-op'd it in the `\xyoption` handler right after xymatrix.tex loads
  (a no-op before option-processing is clobbered; doc-preamble
  `\CompileMatrices` runs before `\begin{document}` so at_begin_document
  is too late). math0203082: OOM(4.6 GB)→2.1 MB main.html/652 MB RSS/1953
  svg; math0402448: OOM→4.2 MB/960 MB RSS/6224 svg. 53 binaries green.
  **Cluster A (`\displaylines` `\raise`/`\hbox` recursion, 7 papers:
  math0102053/089, math0212126, math0504436/06088/07219, math0604321) is
  SHARED** — the line-712 `$$\displaylines{…}$$` recurses through nested
  `\raise\hbox{\lower\hbox{…}}` box-stacking in BOTH engines; Perl *also*
  fails (terminated at the 250 s timeout, no output; backtrace shows the
  same `\raise…\hbox…\lower…\setbox` chain), Rust OOMs at 4.5 GB. Not
  Rust-only; a `MoveableBox::predigest` depth-1000 cap already exists
  (base_parameter_types.rs) but the blowup is gradual accumulation below
  that depth. Left as SHARED.

* **Round-37 Rust-only conversion failures: EXHAUSTED.** After the four
  R19 fixes below, three fresh `cortex_worker` sweeps of distinct slices
  of `canvas3_round37_remaining` (1500 + 3005 + 2081 ≈ **6.6k papers**)
  plus the 1164-paper failed-list re-test surfaced **zero remaining clean
  Rust-only conversion failures** (Perl-succeeds / Rust-fails). Every
  residual flagged failure is one of: (a) **SHARED** — Perl also fails
  with empty/over-cap output (catoptions raw-load, xint+tikz pgf runaway,
  deep_recursion, and the recurring **`_`/`^`-in-text** 100-error-cap
  cluster: 1510.03740, 1711.05610, 2001.01049 — both engines error-cap on
  stray `_`/`^` in text mode); (b) **degenerate input** — e.g. 1906.01445
  is a 12-byte `%auto-ignore` stub (no TeX source; both engines correctly
  emit an empty 39-byte doc); (c) **`not_tex_source`** PDF-as-tex; or (d)
  **CPU-oversubscription false timeouts** (see sweep note below). Triage
  rule reaffirmed: classify by *Perl output byte-size*, not its
  complete/failed status (see [[feedback_perl_baseline_output_size]]), and
  `%auto-ignore`/empty-source inputs are not bugs. Net: the round-37
  corpus is clean of actionable Rust-only conversion failures; remaining
  work is SHARED-limit hardening (lower priority, won't yield successful
  HTML since Perl is also empty) or scaling to new corpus regions.

* **`\kill` in `p{}` longtable locked-frame FATAL — RESOLVED** (commit
  `6e5f29a2a9`). Witness 2010.09763 (Perl: 1.94 MB / 140 `<tr>` / 0 errors;
  Rust was `Fatal:TargetUnexpected:Endgroup`, empty). `\kill` was `Let` to
  the bare `\lx@longtable@kill@marker` constructor whose afterDigest fired
  `Alignment->removeRow` **mid-cell**, while the `p{}` column's
  `\vtop{\hbox{…` boxing/mode frame (TeX_Tables L67-69) was still open →
  frame leak → at `\end{longtable}` the `\endgroup`/`\@end@tabular`
  mismatched it → `pop last locked stack frame` FATAL. (Perl's column
  scanner is *also* incremental — verified — so Perl ALSO has the box open;
  Perl just tolerates the desync where our stricter mode/frame checks
  abort.) **Fix = faithful to real `\LT@kill` = `\LT@echunk` (end the
  chunk/row, measure, discard):** route `\kill → \lx@longtable@kill@flag\crcr`.
  The `\crcr` ends the row through the NORMAL cr path (closing the column
  boxes exactly like `\\`, no leak); `\lx@longtable@kill@flag` sets
  `LONGTABLE_KILL_NEXT`, and the alignment driver
  (`digest_alignment_body`, tex_tables.rs) drops the just-ended,
  box-balanced row when that flag is set. Avoids BOTH earlier dead-ends
  (bare-marker FATAL; `\crcr\noalign{marker}` popping the noalign
  pseudo-row and leaving `KILLED` visible). Result: 701 KB main.html, 5/5
  deterministic, **0 errors**, tr=140 / Math=852 / 132 data rows ALL match
  Perl, killed rows removed (0 `KILLED` garbage). Full suite green (53).
  Same locked-frame *class* as the arydshln fix below + 1510.04473, but a
  distinct trigger. The `current_frame_message` readable-locator
  (committed with arydshln) is what localized it.

* **arydshln: stop noop'ing `\endlongtable`** (commit `42bcc87de0`) —
  Rust-only locked-frame FATAL on `arydshln` + `longtable` with `p{}`
  columns (1510.04473, single clear Rust-only case in the round-37
  failed-list re-test). The stub copied ar5iv `arydshln.sty.ltxml` L45's
  `DefMacro('\endlongtable', Tokens())` noop, but the REAL `arydshln.sty`
  SAVES+RESTORES longtable's original `\endlongtable`
  (`\let\endlongtable\adl@org@endlongtable`, L796), not neutralizes it. Our
  longtable relies on `\endlongtable=\lx@end@alignment\@end@tabular` to
  close the alignment boxing group; noop leaks the `{`-group → env
  `\endgroup` mismatch → mode cascade → `pop last locked stack frame`
  FATAL. Perl-ar5iv recovers (9 errors); we abort. Keeping `\endlongtable`
  functional matches the real package: 1510.04473 → 716 KB main.html, 5/5
  deterministic, 18 tables / 101 rows / 896 math (Perl=9 err, so we surpass
  it). Also made `current_frame_message` render the initiator locator
  readably (was redacted) — this localized the leak.

* **`Font::to_hashable` determinism** (commit `4dfc877ade`) — used
  `RandomState::new()` (fresh random seed per call), so the same Font
  hashed differently each call/run; it keys the `_font` node attribute and
  `node_fonts` map (set/get_node_font), making font dedup and
  font-identity-dependent layout nondeterministic. Manifested as
  intermittent FATALs flipping pass/abort across runs of the SAME binary on
  the SAME input. Switched to `FxHasher` (fixed seed). This was masking the
  arydshln bug above: 1510.04473 alternated complete/FATAL until the hash
  was made deterministic, then reproduced reliably for root-causing.

* **Round-37 failed-list re-test (1164 papers, current binary).** Only
  ~58–62 of 1164 prior failures genuinely still fail; the rest were
  recovered by landed work (re-test BEFORE investigating — canvas failed
  lists go stale fast). Genuine residue is dominated by **SHARED**
  Perl/Rust limits, NOT Rust-only: (a) catoptions/keyval2e raw-load (4
  papers: 1501.07012, 1502.01082, 1507.04637, 1512.01732) — Perl
  `--includestyles` ALSO FATALs (`too_many_errors:100` at catoptions.sty
  L6362; see KNOWN_PERL_ERRORS); (b) `\deep_recursion` 1612.06222 — Perl
  FATALs identically; (c) `not_tex_source` PDF-as-tex (4) — correctly
  rejected; (d) `TooManyErrors` rc=3 (~34) — spot-checked, Perl also
  hits its 100-error cap. 1510.04473 was the lone clear Rust-only case
  (now fixed above).

* **CORRECTION: 1804.01117 is SHARED, not Rust-only.** Perl reports
  `Conversion complete: 39 errors` but its **output is 39 bytes (empty
  document)** — "complete" in Perl does NOT mean real output; Perl can
  finish with errors and an empty `<document/>`. So neither engine
  produces usable HTML for this paper (Rust FATALs at the 100-error cap;
  Perl emits 39 errors + empty doc). **Triage lesson:** when checking
  Perl as ground truth, verify Perl's **output byte size / element
  count**, not just its `complete` vs `failed` status — otherwise an
  empty-but-"complete" Perl run masquerades as a Rust-only win. (Both
  fresh sweeps ultimately found **zero clean Rust-only failures**: every
  genuine residual converts to empty/failed in Perl too.) The pgffor
  analysis below is retained because the *runaway* is still a Rust
  reliability quirk worth hardening, but fixing it would NOT make the
  paper succeed (Perl doesn't either).

  **pgffor `\pgffor@values` self-ref cascade (deep, low priority).**
  `\usepackage{tikz}` with a complex/malformed `\foreach` (source typo at
  main.tex L83: `…/\colorII\shapeIII/…`, a MISSING `/`). Our cluster: 90×
  `\lx@end@inline@math`, 83× `Error:recursion:\pgffor@values`, 25× `fi`.
  Trace
  (`DBG_RECUR_GUARD` in expandable.rs): the guard fires because
  `\pgffor@values`'s body is *genuinely* `\pgffor@values, \pgffor@stop,`
  (self-referential first token) under full expansion — so the guard is
  CORRECT (prevents an infinite loop); the real bug is **upstream**:
  pgffor's `\pgffor@expand@list` (pgffor.code.tex L89) /the L90
  `\expandafter\def\expandafter\pgffor@values\expandafter{\pgffor@values,…}`
  leaves `\pgffor@values` self-referential when parsing 1804.01117's
  complex bracketed `\foreach` values, whereas Perl builds it correctly
  (verified: `\pgffor@expand@list` on a *well-formed* list works in our
  engine too — `\pgffor@values`→`1,2,3` — and a minimal malformed
  `\foreach` does NOT reproduce; the cascade needs the full tikzpicture
  with custom `regular polygon` shapes + `\includegraphics` nodes).
  Needs a dedicated pgffor value-parser raw-interp session; deep tikz,
  single rare paper, source-typo-triggered, Perl also degrades (39 err),
  so low priority. Do NOT weaken the recursion guard (it's faithful to
  Perl Expandable.pm L81-89 and is correctly catching a real self-ref).

* **1910.03372 — SHARED (tikz-load runaway vs Perl empty).** scrartcl +
  `xint`/`xinttools`/`xintexpr` + `braket`/`bbold`/`txfonts` + tikz. Rust:
  `Error:pushback_limit:Timeout (650000 exceeded, infinite loop?)` while
  loading `tikz.sty` → 60 s wall-clock FATAL, no output. Perl: 87 errors,
  13 undefined pgf macros (`\pgfkeys`, `\pgfmath@def`, `\pgffor@var`…),
  **39-byte empty output**. Neither produces HTML — the xint/pgf package
  combo defeats both engines' tikz raw-interp. Rust's runaway/timeout is a
  reliability quirk worth eventual hardening (pushback-limit instead of
  graceful degradation), but it is NOT a Rust-only win.

* **Fresh sweep of unseen `remaining` papers (current binary): clean.** A
  1500-paper sample (every 180th of `canvas3_round37_remaining.txt`)
  produced **zero genuine failures** — the only `rc=124` (1902.03551)
  was a CPU-contention **false timeout**: on a quiet CPU the release CLI
  converts it in **14.7 s** → 14.7 MB XML, 6122 `<Math>`, 0 errors.
  **Methodology lesson:** running the sweep at `-P $(nproc)` (=20)
  oversubscribes the box (each `cortex_worker` is itself multi-threaded
  for post-processing), so large math-heavy docs blow past the 120 s
  worker timeout under contention even though they finish in seconds
  alone. Use `-P ~10` and/or a higher `--timeout` for sweeps, or the
  TIMEOUT column will be inflated with non-bugs. (28 `rc=143` in that run
  were SIGTERM from stopping the sweep — re-tested clean: 2.8/3.6/1.3 MB.)

* **alignment noalign recursion: save `\lx@label` not mutable `\label`**
  (`<this commit>`) — Root cause of the deferred `\lx@hidden@noalign`
  `Stomach:Recursion` cluster (2008.13358 amsgather, 2009.09721
  amsalign). Both `eqnarray_bindings` (latex_constructs.rs) and
  `ams_rearrangeable_bindings` (amsmath_sty.rs) did
  `Let('\lx@eqnarray@save@label', '\label')` with **GLOBAL** scope, then
  `Let('\label', '\lx@eqnarray@label')` locally. Perl instead saves the
  **immutable canonical** `\lx@label` (latex_constructs.pool L2323:
  `Let('\lx@eqnarray@save@label','\lx@label')`). Under nested
  align/gather, the inner binding re-runs while the OUTER `\label` is
  already `\lx@eqnarray@label` (the noalign wrapper); the GLOBAL save
  then captures the wrapper, making `\lx@eqnarray@save@label` globally =
  `\lx@eqnarray@label` = `\lx@hidden@noalign{\lx@eqnarray@save@label{#1}}`
  → digesting that arg re-emits itself → unbounded `invoke_token` nesting
  (this is why MAXSTACK=5000 still overflowed, and why it was
  accumulation-dependent: it needs ≥2 nested rearrangeable scopes before
  the GLOBAL save captures the wrapper). Backtrace-confirmed: at every
  recursion depth the noalign `#1` arg was
  `\lx@eqnarray@save@label{sec:properties-traces}`. **Fix** = faithful
  Perl translation: rename the `\label` DefConstructor to `\lx@label`,
  add `Let('\label','\lx@label')` (Perl L3862), and save `\lx@label`
  (immutable) in both bindings. `\lx@label` registers in
  `latex_constructs` (post-dump), so it overrides the dumped kernel
  `\label` macro exactly as the old `\label` DefConstructor did. Witness
  2008.13358 via `latexml_oxide` (CLI): `Fatal:Stomach:Recursion` → 546 KB
  XML / 175 KB HTML, 23 errors (Perl=64 err, 8 missing files — we beat
  Perl on missing-package count). Full test suite green (53 binaries).
  **`cortex_worker` (canvas) now also fully succeeds: 286 KB main.html,
  10/10 deterministic, valid content (3 sections, 419 math nodes, 12
  equationgroups).** 2008.13358 *also* uses `\usepackage[all,cmtip,2cell]{xy}`;
  the transient xy.sty "re-entrance → empty XML" seen on early post-fix
  worker runs was a CONSEQUENCE of the corrupted global state left by the
  recursion FATAL (it does NOT reproduce on the clean fixed binary), NOT
  an independent xy blocker. With the recursion fixed, xy loads cleanly in
  the worker too.

* **xy: guard `\lx@xy@original` capture against double-load**
  (`<this commit>`) — xy_sty's SVG-wrapper overlay (Perl xy.tex.ltxml
  L148-151: save real `\xy`→`\lx@xy@original`, install wrapper `\xy`)
  was applied on EVERY entry of the binding. The binding is entered
  twice: once via `\usepackage{xypic}`→`RequirePackage("xy")`, and again
  because the real xy.tex (raw-loaded at L36) issues `\input xy.tex`,
  which our `\input` resolves to the `("xy","tex")` Rust binding (= xy_sty)
  instead of the self-guarding real file. On the 2nd entry `\xy` was
  ALREADY the wrapper, so `Let('\lx@xy@original','\xy')` captured the
  wrapper — making `\lx@xy@original` self-recursive and (since the real
  xy processing that sets `\xy@`≠`\xyinitial@` never runs) `\inxy@` always
  reports "not nested", so every internal `\xy` re-enters `\lx@xy@svg`
  UNBOUNDEDLY → `Fatal:Stomach:Recursion`. Guarded the overlay with
  `if !is_defined("\\lx@xy@original")` so it applies exactly once
  (matching Perl's idempotent package load). Witness 2009.05542
  (`\xymatrix` in an equation): via `latexml_oxide` FATAL → clean
  (5.2 MB HTML, 88 `svg:svg` diagrams rendered, matching Perl's 0
  errors). Full test suite green (53 binaries).

  **RESOLVED ON RE-TEST (2026-05-28).** The earlier-documented
  "`cortex_worker`-only xy blocker → 0-byte HTML, `\xymatrix` undefined,
  candidate (i) insufficient" no longer reproduces. On the current clean
  binary `cortex_worker` converts 2009.05542 to **2.8 MB main.html with
  41 `svg`/`svg:svg` diagrams, 3/3 deterministic**. Two findings closed
  the prior open questions:
  (a) the claim "`\xyloaded` is undefined in the worker but defined in the
  CLI" was **wrong** — a trace (`is_defined("\\xyloaded")`) shows it is
  `false` in BOTH the CLI and the worker at feature-load time, and the CLI
  succeeds anyway. So candidate (i) ("make `\xyloaded` survive") was
  chasing a non-difference; it was correctly abandoned.
  (b) the "0-byte worker" observations (here and on early post-`\lx@label`-
  fix 2008.13358 worker runs) were **stale-state / not-cleanly-rebuilt
  artifacts**, not a live code bug: a from-clean `cortex_worker` rebuild
  makes both papers succeed deterministically. Lesson for future triage:
  when a worker "0-byte" diverges from a clean CLI success, FIRST rebuild
  the worker from clean and re-run several times before theorizing a
  worker-vs-CLI dispatch divergence. The `xy_sty.rs` `\xyoption`
  feature-file direct-`input_definitions` workaround is unchanged and is
  NOT the cause.

### R18 fixes (2026-05-28)

* **IEEEproof: drop surpass-Perl `mode => internal_vertical`**
  (`<this commit>`) — Perl `IEEEtran.cls.ltxml` L206 declares
  `{IEEEproof}` with NO `mode`, leaving it in the ambient
  restricted_horizontal. Our binding had added `mode => internal_vertical`
  so `$$..$$` inside `\begin{IEEEproof}` would parse as display math.
  But Perl's dollar handler is identical (`$$` is display only in a
  vertical bound mode), and Perl does NOT treat such `$$` as display —
  it emits the cascading "Script _/^ can only appear in math mode" errors
  (verified on a synthetic IEEEproof+`$$`). So the tweak was a surpass-
  Perl divergence. Worse, the vertical mode made `\endIEEEproof` end
  `internal_vertical`, which matches the BOUND_MODE that
  `\begin{document}` binds on the LOCKED frame — so a *bare*
  `\endIEEEproof` (author error, no matching begin) popped the locked
  frame → `Fatal:TargetUnexpected:Endgroup`, aborting the run with empty
  HTML. Removing the mode: `\endIEEEproof` ends restricted_horizontal
  (never matches the locked frame) → Perl's recover-branch fires →
  completes. Witness 2009.01572 (bare `\endIEEEproof` at L570): FATAL /
  0-byte HTML → "Conversion complete: 1 error" with 367 KB HTML, matching
  Perl exactly; `$$`-in-IEEEproof now matches Perl's 3-error output.
  Full test suite green (53 binaries). Resolves the R18-DEFERRED
  2009.01572 entry below.
* **`read_normal_integer`: empty octal/hex → 0 (not fatal)**
  (`<this commit>`) — `gullet::read_normal_integer` parsed `'`/`"`
  number prefixes via `i64::from_str_radix(&read_digits(...)?, N)?`,
  which **fatally propagated** a `ParseIntError` (→
  `Fatal:Document:Generic(ParseIntError)`, aborting the whole run) when
  the prefix was followed by no octal/hex digit. Perl uses
  `Number(oct(...))` / `Number(hex(...))`, and Perl's `oct("")`/`hex("")`
  are 0 (TeX's "Missing number, treated as zero"). Mirror that: empty
  digit string → 0; valid → parsed; overflow → clamp to i64::MAX (as the
  decimal arm already does). Witness 2008.10843 (`mdwmath.sty` raw-load
  reads a bare `"`: FATAL → "Conversion complete" with HTML output;
  remaining 43 errors are SHARED mdwtab/`\tab@*` issues — Perl FATALs at
  101 errors on this paper, so we now surpass it). Valid hex/octal
  (`\char"41`→A, `"FF`→255, `'17`→15) verified unchanged.

**R18/R19 DEFERRED — Rust-only `Stomach:Recursion` FATAL cluster (Perl
completes).** Fresh offset-18/19 sampling (~6000 papers) surfaced a
cluster of infinite-recursion FATALs, all where Perl converts fine.
Two distinct mechanisms, both genuine *unbounded* recursion (NOT
low-MAXSTACK — verified 5000 still overflows for the noalign case):
  1. ~~**alignment `\lx@hidden@noalign`** — 2008.13358 (amsgather),
     2009.09721 (amsalign).~~ **RESOLVED (R19 fixes above):** root cause
     was `\lx@eqnarray@save@label` GLOBAL-saving the mutable `\label`
     (which is the noalign wrapper under nested align/gather) instead of
     the immutable canonical `\lx@label`. Not an alignment-internals
     nesting issue. **2009.09721 re-tested on the current binary: full
     `cortex_worker` success — 583 KB main.html, 0 errors (528 warnings),
     no recursion.** 2008.13358 (amsgather + `\usepackage[all,cmtip,2cell]{xy}`)
     is ALSO a full `cortex_worker` success now (286 KB main.html, 10/10
     deterministic) — its xy path loads cleanly once the recursion is gone.
  2. ~~**xypic `\lx@xy@svg`** — 2009.05542 (Perl=0, clean Rust-only).~~
     **RESOLVED:** the CLI recursion was fixed by the committed
     `\lx@xy@original` double-load guard (R19 fix above), and the
     previously-documented "`cortex_worker`-only 0-byte / `\xymatrix`
     undefined" blocker does NOT reproduce on a clean rebuild — re-tested
     2026-05-28, worker → 2.8 MB main.html, 41 svg diagrams, 3/3
     deterministic. (The `\xy@`/`\xyinitial@`-nesting recursion theory and
     the `\xyloaded` worker-vs-CLI-difference theory were both
     superseded; see the resolved note under the `\lx@xy@original` R19
     entry above.)
  Other R19 FATALs (classify before fixing): 2009.05276
  (`TooManyErrors`: `\GenericError` runaway 501×, likely vendor/SHARED),
  2009.09806 (`Timeout:MemoryBudget` RSS>4500MB — OOM, separate class).
Found via a fresh sample of the offset-18 remaining slice.
  * ~~**2009.01572** — RESOLVED~~ (see R18 fixes above: the locked-frame
    pop was a *symptom* of IEEEproof's surpass-Perl `internal_vertical`
    mode, NOT a deep mode-stack divergence. The earlier
    `end_mode`-guard attempt was the wrong layer; removing the env-mode
    fixed it at the source. Lesson: when a bare env-end CS pops the
    locked frame, suspect the env's `mode =>` matching the
    document-body bound mode before suspecting `pop_frame`/`end_mode`.)
  * ~~**2008.13358 `main.tex` (eptcs + mathpartir) —
    `Fatal:Stomach:Recursion`.**~~ **RESOLVED (R19 fixes above).** The
    backtrace WAS the smoking gun, but the cause was not
    alignment-internals: at every recursion depth the noalign `#1` arg
    was `\lx@eqnarray@save@label{sec:properties-traces}`, i.e.
    `\lx@eqnarray@save@label` had become the self-recursive
    `\lx@eqnarray@label` wrapper. The accumulation-dependence (only after
    ~8 gathers + the L321 label) was exactly the nested-rearrangeable-
    scope GLOBAL-save capturing the wrapper. Fixed by saving the
    immutable `\lx@label` (Perl parity). CLI now clean (546 KB / 23 err).
    `cortex_worker` ALSO fully succeeds (286 KB main.html, 10/10
    deterministic) — the recursion was the sole blocker; the early
    post-fix "xy worker re-entrance → empty" was a stale-state artifact of
    the caught FATAL, not reproducible on the clean binary.

### Round-37 err=3-5 gate sweep (2026-05-29): 4 fresh PCLEAN, 1 fixed, 3 deferred-deep

Gated 17 err=3-5 candidates (excluding shared clusters) from the resweep TSV →
4 Perl-clean+Rust-fail. **2002.09766 FIXED** (algorithm2e+algorithm combo, see
below). Remaining 3 are all the SAME META-PATTERN: *Perl's kpathsea doesn't
find the package* (reports it "missing" → skips it), while *Rust's kpathsea
finds it on the shared TL tree and raw-loads it*, then hits deep package
internals. (The two engines diverge on file lookup despite the same texmf;
`kpsewhich <pkg>.sty` succeeds, so these would cascade in Perl too in an env
where Perl finds them — i.e. they are "beyond-Perl" raw-load-robustness work,
not Perl-parity gaps.)
* **2005.05941** — `\fontaxes@code`/`\fontaxes@edoc` undefined (XCharter font →
  fontaxes). fontaxes' `\fontaxes@encode@[2]` dispatcher (fontaxes.sty L245)
  does `\@ifundefined{fontaxes@encode@#1#2}` where `#2` is `{w}{a}` (BRACES) —
  a `\csname`-with-braces read that diverges in Rust so the `@default` branch
  never defines `\fontaxes@code` ("readBalanced ran out"). Same hard
  csname-protocol territory as [[project_mhchem_csname_protocol_deepdive]].
* **2003.05608** — betababel.sty (Greek beta-babel) → teubner → Greek babel
  cascade: `\bbl@attributes` undefined + "Greek language unknown" + "attribute
  polutoniko unknown" + `\Greeknumeral`. Multi-error Greek-typography stack.
* **1910.10243** — pstricks/pst-plot → `\ifpst@useCalc`/`\ifpst@psfonts`/
  `\colorlet`/`\ifluatex` undefined. pstricks cascade.

### Round-37 release-binary fresh scan (2026-05-29): high parity confirmed

Built a fresh `--release` binary (all 8 session fixes) and scanned ~2500+
NOT-previously-gated fresh papers (correct path `.../bindings`, largest-
`\begin{document}` main-detection). Failure rate ~0.5%, and EVERY failure falls
into a known category — confirming the binary is at **high parity** with Perl
and genuine Rust-only candidates are now *rare*:
* **SHARED** (Perl fails identically): `Error:unexpected:_` math-mode cluster
  (×4); `Error:misdefined:#` = mdwmath.sty (×4+). Root of the mdwmath `#`-leak:
  a catcode-trick `\def\sq@readrad#1"#2\#3\relax{…}` then
  `\expandafter\sq@readrad\meaning\sqrtsign\relax` (mdwmath.sty L48-52) that
  parses `\meaning\sqrtsign` expecting a `\mathchar"XXXX` hex with a literal
  `"`. BOTH LaTeXML ports' `\meaning` of `\sqrtsign` lacks that `"`-hex form, so
  the `Until:"` delimited read fails → "Missing argument" → leaked `#`s reach
  the Stomach. SHARED `\meaning`-format gap (KNOWN_PERL_ERRORS-class), not
  Rust-only.
* **Main-detection artifacts**: multi-file papers where the heuristic picks a
  fragment (e.g. 1902.00014 → `appendix-2.tex`/tikz `standalone` → `\section`
  undefined; Perl ALSO errors on the fragment). Scan tooling, not engine.
* **GENUINE Rust-only cluster** = `Error:malformed:ltx:listingline` — now 2
  instances (1911.01815, 1903.04631 `supplement.tex`): listing/verbatim inside
  `\hbox`/`\colorbox`. CONFIRMED genuine (Perl clean, correct path). Deep
  whatsit/absorb-order root (see 1911.01815 note) — top remaining genuine
  Rust-only work; needs a focused engine-tracing session (instrumenting the
  HBoxContents-predigest → absorb → `\lx@algo@@endline` construction order, the
  one place Perl and Rust diverge despite byte-identical `\hbox`/`closeElement`/
  `canAutoClose`).

**Net:** the session's 9 fixes brought the binary to near-parity; the genuine
Rust-only wins are exhausted. Remaining work is SHARED beyond-Perl gaps (mdwmath
`\meaning`, revtex4 `@sw`, pushback-loop hangs, fontaxes/betababel/pstricks
raw-loads, table-in-algorithm2e-listingline).

**2026-05-29 (cont. 2) — NEAR-PARITY CONFIRMED across ~7000 sampled papers.**
After fixing 1911.01815 (`2627ed999a`), ran a second cleaner release scan
(cortex-style main-detection: require an UN-commented `\documentclass`, prefer
the `.tex` with a sibling `.bbl`) on a fresh 3000-paper batch + gated the
`_`/`^` cluster. Verdicts: 1904.10409 SHARED (`_` text-mode = genuine LaTeX
error, both fail 3/3); the cleaner scan's failures are still mdwmath `#`
(SHARED) + package-source artifacts (e.g. `fancyhdr/fancyhdr.tex` — a bundled
package source, NOT the paper; needs root-dir/`.bbl` preference to suppress).
**No genuine Rust-only candidate found** across ~7000 papers sampled this
session (4000 + 3000, multiple offsets, recent years). The genuine Rust-only
error class is EXHAUSTED for the discoverable cases; the residual is SHARED
beyond-Perl gaps (where Perl also fails: mdwmath `\meaning`-hex, firstaid,
revtex `@sw`, pushback hangs, table-in-listingline) or text-mode-`_` paper-bugs.
Next genuine-candidate hunting needs either a much larger sample (genuine rate
≈ 0.01%) or scanning a different ecosystem; SHARED gaps are beyond-Perl
"surpass" work, not faithful-parity Rust-only fixes.

**2026-05-29 (cont.) — fresh-scan candidates all SHARED.** Perl-gated (CORRECT
path) every low-error failure from the release scan: 1903.04631 SHARED (8/8,
`\tabular` inside an algo2e listingline — both fail), 1907.06165 SHARED
(achemso, Rust 1 < Perl 5 — Rust ahead), 1907.10053 SHARED (firstaid
`latex2e-first-aid-for-external-files.ltx`, Rust 2 < Perl 12+fatal — Rust
ahead), 2001.01248 SHARED + artifact (commented `\documentclass` in an
ieeeconf template fragment), 1901.08873 SHARED (pushback infinite-loop, Perl
also at 599999). The dominant failure types in the scan are `Error:unexpected:_`
/`^` (text-mode `_`/`^` — genuine LaTeX errors, both engines correctly fail) and
`Error:misdefined:#` (mdwmath `\meaning`, SHARED). **No genuine Rust-only
candidate found in the 4000-paper release scan** (the 1911.01815 listing-in-box
was the last genuine one, fixed `2627ed999a`). SCANNING-TOOLING note: the
largest-`\begin{document}` main-detection still picks fragments/templates in
multi-file submissions (commented `\documentclass`, `appendix-*.tex`, ACM/IEEE
sample files), producing false candidates — a future scan should prefer the
`.tex` with a sibling `.bbl` (cortex_worker's heuristic) to cut artifact noise.

### Round-37 err=6-10 gate sweep (2026-05-29): stale-TSV clean wins EXHAUSTED

Gated the remaining 12 err=6-10 candidates (excluding shared clusters). 3
Perl-clean+Rust-fail, ALL deep and ALL the same META-PATTERN as the err=3-5
batch (Perl's kpathsea misses the package → skips it; Rust raw-loads it → deep
cascade) or known mode-frame-leak territory:
* **2004.03193** — `\lx@hidden@egroup Attempt to close boxing group` (CJK.sty
  raw-load). The boxing-group variant of [[project_endgroup_modeswitch_frame_leak]]
  — a known-hard remaining root cause.
* **2004.03970** — `Extra alignment tab '&'` ×8 (ifacconf.cls missing → fallback
  class table-column cascade).
* **1910.14035** — `Error:unexpected:` ×10 (arydshln.sty raw-load, dashed-rule
  arrays).

**Conclusion:** the clean, low-error, single-root Rust-only wins in
`resweep_fresh.tsv` (err=1..10) are now EXHAUSTED — 8 fixed this session
(2007.04819, 1911.07001, 2006.02269, 1910.09629, 2006.10240, 2006.06087,
2004.07710, 2002.09766). The residual splits into: (a) the META-pattern
"Perl-can't-find-pkg → Rust-raw-loads → deep cascade" (fontaxes, betababel/Greek,
pstricks, CJK, arydshln, ifacconf) — beyond-Perl raw-load-robustness work, NOT
parity gaps; (b) deep document-builder cases (1911.01815 hbox+algo2e, boxing-
group mode-leak); (c) Perl-FATAL non-wins. Next productive step is a FRESH
canvas re-sweep with the current binary (the TSV predates the session's 8 fixes
+ general engine improvements, so it under-counts what now converts) rather than
further mining this stale list.

### FIXED: ifacconf stub eager-loaded xcolor → `\usepackage[table]{xcolor}` no-op → `m{}` column "Extra alignment tab" (2026-05-29)

**Witness 2004.03970** (`\documentclass{ifacconf}` + `\usepackage[table]{xcolor}`
+ a `{p{..}p{..}p{..}m{0.10\textwidth}l}` table). GENUINE Rust-only: Perl 2
warn / 0 errors / 850 KB; Rust 8 errors `Extra alignment tab '&'` / 716 KB.
Long root-cause chain (fully bisected):
1. Errors are "Extra alignment tab" in a 5-col table whose 4th col is
   `m{0.10\textwidth}` (array's vertically-centred para column).
2. The `m`/`b` column types live ONLY in `array.sty` (true in Perl too —
   `array.sty.ltxml` L29/L35, not core `TeX_Tables.pool`). Without `array`
   loaded, `m` is `Unrecognized tabular template` and the parser mis-recovers
   over `0.10\textwidth` char-by-char → spurious alignment tab PER such column.
3. `array` is normally pulled by `\usepackage[table]{xcolor}` → colortbl →
   `\RequirePackage{array}`. Under `article` this chain fires; under
   `ifacconf` it did NOT — colortbl never loaded.
4. Why: the Rust-only contrib stub `ifacconf_cls.rs` (Perl has NO ifacconf
   binding — it uses OmniBus) eagerly `\RequirePackage{xcolor}` at load time.
   So by the time the document's `\usepackage[table]{xcolor}` runs, xcolor is
   already loaded → the `table` option is silently dropped (LaTeX "options on
   already-loaded package" semantics — SHARED with Perl, verified) → colortbl
   never loads. Perl never preloads xcolor for ifacconf, so its document-level
   `\usepackage[table]{xcolor}` loads fresh WITH `table` → colortbl → array →
   `m` recognised → clean.

Fix: remove the eager `\RequirePackage{xcolor}` from `ifacconf_cls.rs` (the
document loads xcolor itself, with its own options — matching Perl). Eager
`color` (from the stub's `hyperref` → hyperref.rs L44) is harmless: color and
xcolor coexist and a later `xcolor[table]` still processes. 8 errors → 0
("No obvious problems", 718 KB). cargo test 1344/0. (commit `711306e9ab`).

**BROADER CLUSTER:** ~39 contrib class stubs eager-`RequirePackage` xcolor.
Verified: ALL sampled (ceurart/cas_dc/sagej/mdpi/sn_jnl/jmlr/lipics/
interspeech/scipost) have NO Perl binding (Perl uses OmniBus, never preloads
xcolor) → systematic Rust-only divergence. Risk audit: all 39 also load
hyperref/color (so `\color`/`\definecolor` stay available) and only 2
(scipost, bytedance_seed) use xcolor-specific commands internally. The minimal
bare `\usepackage{xcolor}` + `[table]` double-load is SHARED with Perl (LaTeX
"already-loaded → drop options"), so the fix is per-binding: drop the eager
xcolor preload (the document loads xcolor with its own options).

* **LANDED 2026-05-29 (batch 1):** lipics, jmlr, sn_jnl, sagej.
* **LANDED 2026-05-29 (batch 2, +24 classes):** pnas_new, ecai, gretsi,
  egpubl, ptephy, nature_pre, cimart, ejpecp, asme2ej, achemso, wlscirep,
  sigma, agujournal2019, tac, wileymsp_template, aomart, optica_article,
  bmvc2k, interspeech, interact, combine, lmcs, wlpeerj, siamart. Each first
  CONFIRMED to exhibit the bug (`\documentclass{CLS}\usepackage[table]{xcolor}`
  + `m{0.10\textwidth}` table → extra-tab, colortbl not loaded), then the
  eager xcolor preload removed, then re-verified: 23/24 → 0 tabs + colortbl
  loads + plain `\textcolor`/`\definecolor` doc clean (no color regression —
  color stays via each stub's hyperref→color). cargo test 1344/0.
  * **pnas_new** — eager xcolor removed (Perl-faithful, no regression) but
    STILL preloads xcolor via a separate transitive path (one of its many
    deps: authblk/fancyhdr/titlesec/caption/booktabs raw-load chain pulls
    xcolor after hyperref→color). Residual `m{}`+`xcolor[table]` bug remains
    for pnas-new pending that second path — tracked as a sub-follow-up.
* **Intentionally KEPT eager xcolor:** scipost, bytedance_seed (use the
  xcolor-only `HTML` color model — `\definecolor{...}{HTML}{...}` /
  `\color[HTML]{...}` — which color.sty can't provide; xcolor is a genuine
  dependency for their styling).
* **Lower-impact (deferred):** mdpi, cas_dc, uai2025, wileynjd, ws_journal —
  these ALSO eager-load colortbl, so their `m{}`/`b{}` columns already work;
  only non-`table` xcolor options (`dvipsnames`, …) would drop. Not yet
  touched. ceurart/scis2024/fcs/oup_authoring_template did not reproduce the
  m-table bug in probing (handle the preamble differently).

### FIXED: extsizes `extbook`/`extreport` mis-bound to `article` → `\thechapter` undefined (2026-05-29)

**Witness 1904.08040** (`\documentclass[14pt,oneside,english]{extbook}` +
`\chapter{...}`; LyX-exported, latin9-encoded). GENUINE Rust-only: Perl clean
(16 warn, 1 missing file[extbook.cls], **0 errors**), Rust 1 error
`\thechapter undefined`. Root cause was a Rust-only paper-over: a contrib stub
`extarticle_cls.rs` routed ALL five extsizes classes (extarticle / **extbook** /
**extreport** / extletter / extproc) to plain `article` via `LoadClass{article}`.
But `article` has no `chapter` counter, so the book-like members
(extbook/extreport, which define `\chapter`/`\thechapter`) lost `\thechapter`
entirely. Perl ships **no** binding for any extsizes class, so
`\documentclass{extbook}` falls through to `OmniBus.cls.ltxml`, whose
`DefAutoload('thechapter', 'book.cls.ltxml')` (omnibus_cls.rs L559) defines
`\thechapter` on first `\chapter` use → 0 errors. Fix (per
[[project_keywords_env_binding_less_cls]] / [[feedback_no_papering]]): **deleted
the stub** and all 5 registry entries so every extsizes class falls to OmniBus
exactly like Perl. `elife.cls`/`pnas-new.cls` bindings `\LoadClass{extarticle}`
which now likewise resolves via OmniBus (article-base superset), layout
preserved. All 5 siblings verified `\chapter`+`\thechapter` → 0 errors;
1904.08040 1 error → 0 (Rust 286 KB, Perl 346 KB, both "using OmniBus").
cargo test green, 0 failed. (commit pending).

### FIXED: algorithm2e `algorithm*`/`algorithm2e` via `\let` broke `algorithm`+algo2e combo (2026-05-29)

**Witness 2002.09766** (`\usepackage{algorithm,algorithmic}` +
`\usepackage[algo2e]{algorithm2e}`, `\begin{algorithm*}`). Found via err=3-5
gate sweep (genuine Rust-only: Perl clean, only ar5iv missing — NOT a
package-availability case). Perl algorithm2e.sty.ltxml L62-64 loops a FULL
`DefEnvironment` over `algorithm2e`/`algorithm`/`algorithm*`; Rust only
DefEnvironment'd `{algorithm}` and `\let`-aliased the rest. When the `algorithm`
floats package is also loaded it raw-defines a `{algorithm*}` two-column float;
`\let\algorithm*\algorithm` leaves the env NAME registered as `algorithm`, so
`\begin{algorithm*}` opened the float package's `<ltx:p>` paragraph wrapper with
algorithm2e's listing machinery inside → listinglines mis-nested in
`<ltx:float><ltx:p><ltx:text>` → "ltx:listingline isn't allowed in <ltx:text>"
(4 errors). Fix: a local `macro_rules!` applies the shared listing-env body as a
full `DefEnvironment` to all four names (matching Perl's loop). 4 errors → 0,
1.67 MB. cargo test 1344/0. (commit `7d0b8c88cf`). NOTE: the deferred 1911.01815
(`\colorbox`/`\hbox` + algorithm2e) is a DIFFERENT listingline-in-text root.

### FIXED: `\@classoptionslist` clobbered on nested `\LoadClass` → global babel langs (2026-05-28)

**Witness 1911.07001** (`\documentclass[oneside,french,titlepage]{amsart}` +
bare `\usepackage{babel}`, `\og`/`\fg` via `\addto\extrasfrench`). HIGH-VALUE,
GENERAL. Every BOUND class (amsart, amsbook, elsarticle, revtex, …) does a
nested `load_class_with_options(base, Tokens!())` with EMPTY options; the cls
path in `content.rs` unconditionally redefined `\@classoptionslist` to the
joined options on every load, so the nested empty-options load clobbered the
document class's option list to `""`. (Standard unbound `article`/`report`/
`book` were unaffected — no nested load.) babel iterates `\@classoptionslist`
(`\bbl@foreach`, babel.sty L4270) to find a GLOBAL language option like
`[french]`, then `\DeclareOption{french}{\bbl@load@language{french}}` +
`\ProcessOptions*` loads french.ldf. With the list clobbered empty, the global
language was silently dropped → french never activated → `\extrasfrench` never
ran → `\og`/`\fg` undefined. Fix: match Perl Package.pm L2561 (`if ($astype eq
'cls' and $options{options})` — set only when options non-empty); retain the
Rust empty-define divergence for an option-less document class (2504.00009
csname guard) but gate on "no class options recorded yet" so it never clobbers
a populated list on nested loads. 2 errors → 0, 3.26 MB (Perl 2.89 MB). Fixes
global babel language for ALL bound classes. cargo test 1344/0. (commit
`ac55fdfeb5`)

### FIXED: `pack_parameters` error+drop on halign-template `#` (2026-05-28)

**Witness 2006.02269** (`\documentclass{amsart}` + easyeqn.sty `{MATRIX}` env,
`$\mathstrut##$` `\halign` template). `pack_parameters` (tokens.rs) packs
`#<digit>`→ARG and `##`→`#`, but `#` followed by anything else (CS, `{`, `$`)
hit a *counted* `Error!` that ALSO dropped both tokens — corrupting the valid
`\halign`/`\valign` alignment-cell marker (or `#{` delimiter). Perl's
packParameters (Tokens.pm L139) does the same error+drop but rarely reaches it
(can't find the package, skips raw load); we DO raw-load. Now preserve both
tokens + log at Info (non-counted) — strictly more faithful to TeX than
erroring+dropping. KNOWN_PERL_ERRORS item 1 (beneficial divergence). 2 errors
→ 0, 6.5 MB. cargo test 1344/0. (commit `0d7e142da0`)

### Round-37 Perl-clean gate sweep (2026-05-28): 15 candidates, 3 fixed, 13 triaged

Perl-gated 65 low-error (err=1/2) candidates from the stale resweep TSV;
isolated 15 Perl-clean+Rust-fail. Fixed this session: **1911.07001** (babel
global french, see above), **2006.02269** (halign template, see above),
**2007.04819** (babel-french `\?`, see below). Remaining 13 triaged:
* **Vendor `\GenericError` (Perl skips MISSING pkg)** — 2001.04856 (pb-lams),
  2001.09580 (embedfile "Missing pdfTeX/luaTeX"). Perl reports the pkg missing
  and never raw-loads it; Rust finds it on TL and hits its vendor guard.
  Candidate for vendor-error downgrade (moot-in-XML class) OR raw-load
  robustness. **2006.10240 FIXED** (commit `d0a59bf42d`): was NOT a vendor
  error — `\usepackage[english,strings]{babel}`; Rust's
  `\lx@babel@activate@mainlang` treated the bare babel KEYWORD option `strings`
  as the main language → `\selectlanguage{strings}` → "haven't defined the
  language 'strings'". Excluded babel's bare keyword options (`strings`, `base`,
  `showlanguages`, `KeepShorthandsActive`, `activeacute`, `activegrave`,
  `debug`, `noconfigs`, `silent`, `nocase`, `leqno`, `fleqn`) from the
  language-candidate filter. 1 error → 0, xml:lang="en".
* **Content-model malformed** — 1911.01815 (`ltx:listingline`, 333 warns,
  statsoc.cls) DEFERRED-DEEP (see below). **2004.07710 FIXED** (commit `8bd255a982`): `Attempt to close
  </ltx:itemize>, which isn't open` — Rust's `\preitem@par` closed `ltx:p`/
  `ltx:para` unconditionally, missing Perl L1505's guard (`!inPreamble &&
  current element != ltx:itemize`). A `\trivlist`-based `{proofof}` env inside
  an itemize had its (para-wrapped) trivlist itemize closed by the para-close,
  so its `\item` escaped to the outer list → later `\end{itemize}` found
  nothing open. Added the guard; trivlist now nests correctly (matches Perl
  tree). 1 error → 0. **2006.06087 FIXED** (commit
  `66c623aeea`): `ltx:theorem isn't allowed in <ltx:note>` — elsart_support_core
  mistranslated Perl L189's `DefMacro('\note{}', "<ltx:note>#1</ltx:note>")`
  (token expansion → LITERAL TEXT, error-free but buggy, flagged `# ?` in Perl)
  as a `DefConstructor` (real `<ltx:note>` element). A paper's
  `\note{\begin{remark}…}` (with its own ignored `\newcommand\note`) then put a
  `\newtheorem` env inside a real ltx:note → error. Reverted to DefMacro;
  Rust `\note` output now byte-identical to Perl. 1 error → 0.
* **Engine** — 1910.09629 (`\iffalse` expected:i), 2005.09884 (pgf 'sequence'
  arg).
* **Perl-FATAL (NOT real wins)** — 2001.04466, 2005.08257 (ebproofs `\else`,
  Rust-AHEAD).
* **Deferred** — 2005.06787 (xint, beyond-parity), 1911.03214 (babel
  double-load: bibgerm→german loads babel first, second `[UKenglish]{babel}`
  ignored as option-clash → UKenglish.ldf never processed), 2006.11831
  (`\varleftarrow`/`\varlongleftarrow` from old-arrows.sty, missing in BOTH).
  **1910.09629 FIXED** (see below).

#### FIXED (2026-05-29, commit `2627ed999a`): 1911.01815 — algorithm2e listinglines inside `\hbox`/`\colorbox`

**Root cause + fix:** Rust had stubbed `\lx@prepend@indentation@{}` as an EMPTY
constructor (it emits indentation at `\lx@algo@startline` instead), which
DROPPED Perl's `$doc->floatToElement('ltx:tags')` cursor-reposition that
`\lx@algo@endline` runs right before `\lx@algo@@endline`. Without it, a listing
wrapped in `\colorbox`/`\hbox` left the cursor inside the box's `_noautoclose`
`<ltx:text>`, so `</ltx:listingline>` errored. Restored the `float_to_element(
"ltx:tags", false)` reposition (keeping Rust's startline-indentation, so #1 is
not re-absorbed). 1911.01815: 1 error → 0 (Perl clean). cargo test 1344/0.
NEW residual: 1903.04631 (`supplement.tex`) had this PLUS a separate
`ltx:tabular`/`ltx:tr`/`ltx:td` "isn't allowed in `<ltx:listingline>`" issue (a
`\tabular` inside an algorithm2e line) — now its top failure; a fresh candidate.
Full diagnostic history below (kept for reference):

#### [history] DEFERRED-DEEP: 1911.01815 — algorithm2e listinglines inside `\hbox`/`\colorbox`

Root-caused to a minimal repro:
```tex
\documentclass{article}
\usepackage[ruled]{algorithm2e}
\begin{document}
\begin{algorithm}
\hbox{\For{$t=1$ \KwTo $T$}{ \# Init \\ $x=0$; \\ }}
\caption{Test}
\end{algorithm}
\end{document}
```
→ `Error:malformed:ltx:listingline Closing tag "ltx:listingline" whose open
descendents do not auto-close. Descendants are "text"` (NON-fatal — it still
closes, but counts as 1 error). The real paper wraps an algorithm2e body in
`\colorbox{gray!25}{\parbox{…}{ … }}`; `\colorbox` → `\hbox{…#3}` (color.sty
L105, matched faithfully in color_sty.rs), so the listinglines render in the
`\hbox`'s restricted-horizontal (`ltx:text`) mode. `\parbox` alone is clean;
the `\hbox` (from colorbox) is the trigger. In that mode Rust emits an
`<emph font="italic">` wrapper and stray `<break/>`s inside the listinglines
that Perl does NOT (Perl's listinglines hold plain `<text>` runs and close
cleanly). The leftover `ltx:text`/`emph` is not auto-closeable, so closing the
listingline reports the malformed-descendant error. Affects "verbatim/listing
inside a colored box" broadly.

**Refined root cause (2026-05-29):** `\hbox` opens its box element with
`_noautoclose='true'` on the `ltx:text` (tex_box.rs:558 — BYTE-IDENTICAL to
Perl TeX_Box.pool.ltxml L313 `openElement($newtag,_noautoclose=>1,…)`; both pick
`ltx:text` here since vmode is false). The `\hbox` sits inside the algorithm
float's first `<ltx:listingline>`, so its `_noautoclose` text is a DESCENDANT of
that listingline. While `\hbox` is still absorbing its content, algorithm2e's
`\For` block machinery (`\algocf@Vline`/`\algocf@@@block` → `\lx@algo@endline
\lx@algo@startline`, algorithm2e_sty.rs L121-123) fires `\lx@algo@@endline`
(`</ltx:listingline>`). closeElement then walks up and finds the `_noautoclose`
hbox `ltx:text` as a non-auto-closeable descendant → the malformed error.
closeElement is also byte-identical to Perl (Document.pm L804-829, same
`Error('malformed',…)`), so Perl would error TOO *if it reached this state* —
Perl is clean only because its digestion of the `\For` block machinery inside a
restricted-horizontal `\hbox` does NOT leave the listingline-close straddling
the open hbox text. The divergence is therefore in the EXPANSION/DIGESTION
ORDER of the algorithm2e `\For` block macros inside an hbox (not in `\hbox`,
`\lx@algo@@endline`, or closeElement, which all match). Next step: trace the
`\algocf@@@block`/`\algocf@Vline` digestion sequence inside `\hbox`'s `absorb`
vs Perl, and ensure the listingline-close fires AFTER the hbox text closes (or
that the hbox content's listingline boundaries are handled before the inner
`\lx@algo@endline`). DEEP — needs a focused digestion-tracing session.

**CONFIRMED genuine Rust-only (2026-05-29, CORRECT Perl path).** Re-gated with
`--path=.../bindings`: Perl converts CLEAN (3 warnings; statsoc.cls missing in
both), Rust = 1 error. Structural diff on the minimal repro: Perl emits
`<listingline><text(hbox)> for <emph>…</emph> do </text></listingline>` — the
`\hbox`'s `_noautoclose` `ltx:text` is nested INSIDE listingline-1 and CLOSES
cleanly at the listingline boundary. Rust opens the SAME `<listingline><text>
for <emph>…` but then the `\For` block-open's `\lx@algo@endline` (closeElement
'ltx:listingline'), firing while still inside the hbox text, hits the
`_noautoclose` text as a non-auto-closeable DESCENDANT → error. Both engines'
`\hbox` (sets `_noautoclose`, opens in current tree, absorbs) and `closeElement`
(errors on non-auto-closeable descendant) are byte-identical — so Perl must
close the hbox text BEFORE the `\lx@algo@endline` fires (digestion/whatsit
order), which Rust does not. Pure root-cause is the `\For`/`\algocf@@@block`
digestion-vs-tree-mutation ordering inside an `\hbox` body. NON-FATAL (output
still produced). Worth a focused session: it generalizes to any listing/verbatim
inside `\hbox`/`\colorbox`/`\fbox` (2nd instance found 2026-05-29: 1903.04631
`supplement.tex`).

**Instrumented trace (2026-05-29).** Added temporary `LX_TRACE_LL` prints to
`open_element`/`close_element` and ran the minimal repro. Rust open/close order:
`OPEN listing → OPEN listingline-1 → OPEN text(hbox, _noautoclose) → OPEN
text/emph (for…do) → CLOSE listingline-1 (cursor inside text(hbox),
cant_close=["text"]) ⇒ ERROR → OPEN listingline-2 → CLOSE listingline-1 clean …`.
So `\lx@algo@@endline` (the For header→body split) closes listingline-1 while
the cursor is still inside the hbox's `_noautoclose` text; the hbox's own
`maybe_close_node(text)` only runs AFTER the whole For body (i.e. after the
split close). Perl's output has the SAME nesting (hbox `<text>` inside
listingline-1, closed there) yet does NOT error — and Perl's `closeElement`/
`canAutoClose`/`\hbox`(`_noautoclose=1`) are byte-identical to Rust, so Perl
must reach the split-close with the hbox text ALREADY closed. The remaining
unknown is purely Perl's digestion/absorb ORDER (why the hbox text closes before
the For's split-close in Perl but after in Rust) — needs PERL-SIDE
instrumentation; a Rust-only force-close of `_noautoclose` inline descendants at
listingline boundaries would match Perl's *output* but not its *mechanism* (a
stopgap), so deferred rather than patched speculatively.

#### FIXED: 1910.09629 — hyperref `\url` + active-`"` conditional leak (2026-05-28)

Root-caused to a minimal repro:
```tex
\documentclass[aps,pra]{revtex4}
\usepackage{quotes}        % bosisio quotes.sty: makes " ACTIVE → \@VIRGOLETTE
\begin{document}
\url{"abc"}                % the .bbl had \urlprefix\url{"http://…983"}
\end{document}
```
→ `Error:expected:\fi \iffalse` (conditional fell off end) + `readBalanced ran
out of input`. quotes.sty makes `"` active → `\@VIRGOLETTE`, which is a
`\newif\if@virgolette` conditional (`\if@virgolette…\else…\fi`). Inside
revtex4's `\url`, that conditional's `\fi` is lost during Rust digestion.

Environmental trigger: Perl reports `quotes.sty` MISSING (can't find
`tex/latex/bosisio/quotes.sty`) so it never makes `"` active → Perl clean. We
raw-load it. **Real url.sty also doesn't neutralize `"`**, so in real
TeX/Perl-with-quotes the conditional would simply *complete* — this is a RUST
conditional-handling bug, NOT a begin_semiverbatim divergence (verified:
`state.rs::begin_semiverbatim` matches Perl `State.pm::beginSemiverbatim`
byte-for-byte; both reset only `SPECIALS`/`\dospecials`, not `"`).

**Pinned to `\lx@hyper@url`** (hyperref's `\url`, hyperref_sty.rs:319), NOT
url.sty's `\lx@url@url`: `\documentclass{article}\usepackage{url,quotes}
\url{"abc"}` is CLEAN (renders literal `"abc"`), but under revtex4 `\meaning\url`
= `\begingroup\lx@hyper@url\url` (revtex4_support pulls in hyperref). The
divergence: `\lx@hyper@url` reads its arg with `read_balanced(ExpansionLevel::
Partial, …)` ("Expand as we go!", hyperref_sty.rs:325) whereas `\lx@url@url`
reads it un-expanded. Under partial expansion the active `"` expands to
`\@VIRGOLETTE`'s `\newif\if@virgolette…\else…\fi` mid-read, and
`read_balanced(Partial)` mishandles the conditional — skips/consumes past the
closing `}` so the `\fi` is lost (→ "conditional fell off end" + "readBalanced
ran out of input"). Two **Fix (commit pending):** SCOPED neutralization — after `begin_semiverbatim`,
`\lx@hyper@url` now resets the common shorthand-active chars (`" : ; ! ? ' \``)
to catcode-OTHER *iff* they are currently ACTIVE, mirroring the existing `~`
neutralization one line above (hyperref_sty.rs:322). URLs are verbatim, so an
active char must render literally — and `:` is doubly important (French babel
makes it active; it's ubiquitous in `http://…`). The deeper root (make
`read_balanced(Partial)` correctly bracket `\if…\else…\fi`) is left for later —
riskier and not needed for this class. Witness 1910.09629: 2 errors → 0
(`\url{"http://…"}` now renders literal quotes). cargo test 1344/0.

### FIXED: babel-french bare `\?` undefined (initiate@active@char side-effect) (2026-05-28)

**Witness 2007.04819** (`\usepackage[frenchb,english]{babel}`). The paper has a
stray set-builder `D([0,T];\R^k):\? u_C=v_C` in display math. Perl converts
clean (0 errors); Rust errored `Error:undefined:\?`. Root cause: babel.def's
`\initiate@active@char{?}` (TL `babel/babel.def` L1372) runs
`\bbl@add@special\csname?\endcsname`; expanding `\csname?\endcsname` on the
undefined escaped `\?` turns it into `\relax` (TeX's csname rule) — a permanent,
global, **language-independent** side-effect of *loading* french (catcode-flip to
active `?` is separate, in `\extrasfrench`). `\:`/`\;`/`\!` are already
math-spacing commands so only `\?` is affected → bare `\?` silently vanishes
under Perl. Rust skips the raw french.ldf load, so `\?` stayed undefined. Added
`\@ifundefined{?}{\let\?\relax}{}` at french_ldf.rs load time (covers `french`
AND `frenchb`). Verified: text `[\? Q]`→`[ Q]`, math `a\?b`→`ab`, both engines
identical; real paper 1 error→0, 3.1 MB HTML (Perl 3.2 MB). cargo test 1344/0.
(commit `58e40e1691`)

### FIXED: siunitx `\DeclareSIPrefix{\cs}` (braced) cs-arg mis-read (2026-05-28)

**Witness 1811.03510** (siunitx). The paper does `\DeclareSIPrefix{\million}
{\text{M}}{2}` (BRACED first arg — the siunitx `m`-arg form) then
`\SI{185}{\million rays/s}`. Rust's `\DeclareSIPrefix` (and `\DeclareSIPrePower`/
`\DeclareSIPostPower`/`\DeclareSIQualifier`/`\DeclareBinaryPrefix`) hand-rolled
`gullet::read_token()` for the cs, which reads `{` (catcode BEGIN) for a braced
arg → the real cs (`\million`) was never registered → undefined → 1 error.
`\DeclareSIUnit` already used the `DefToken` param spec (brace-aware) and was
fine; Perl uses `DefToken` for all (0 errors). Added `read_si_declare_cs()`
(via `read_arg`, strips optional braces, handles bare `\yocto` AND braced
`{\million}`) and routed all 5 through it. 1 error → 0, 1.05 MB HTML; built-in
`\DeclareSIPrefix \yocto {y} {-24}` (bare) still works. cargo test 1344/0.
(commit `<this commit>`)

### R10-R16 cluster characterization — many are Rust-AHEAD, not clean Rust-only (2026-05-28)

Sampled the big still-failing clusters + Perl-gated. KEY meta-finding: most are
NOT "Perl-clean, Rust-fails" — Perl fails too, often WORSE. Rust is at/above
parity on these; the clean Rust-only wins are largely exhausted in these stages.
* **`\else` "not in a conditional" cluster (7 papers) = ebproofs, Rust-AHEAD.**
  2005.08257 (`\documentclass{acmart}` + `\usepackage{ebproofs}`, proof trees):
  Rust raw-loads the bundled ebproofs.sty → `\prooftree`/`\Hypo`/`\Infer`
  DEFINED, only 2 errors from ebproofs' deep internal `\if/\expandafter/\else`
  box-stacking machinery (ebproofs.sty L285/339/390/… — `\expandafter\pop
  \ebproof@stack \else …`). **Perl 101 errors** (ebproofs.sty "missing" in Perl
  → `\prooftree`/`\Infer` undefined → cascade). So Rust FAR ahead; the residual
  `\else` is a deep conditional/`\expandafter` interaction, not a parity gap.
  Don't chase the `\else` cluster as Rust-only. (No easy minimal repro:
  ebproofs only raw-loads under ar5iv INCLUDE_STYLES.)
* **abntex2cite `\abntnextkey` (1910.04251) root-caused, tangled.** Rust 1 err
  (abntnextkey) vs Perl 3 (address/keywords — abntex2cite "missing" in Perl).
  Rust's `\bibitem` → `\lx@bibitem` DefConstructor BYPASSES abntex2cite's
  `\def\@lbibitem[#1]#2{\gdef\abntnextkey{#2}}` redef, so `\abntnextkey` is
  never set when the .bbl's `\bibciteEXPL{\abntnextkey}` reads it. Fix needs a
  bib-mechanism change (Rust `\bibitem` honoring a redefined `\@lbibitem`) or an
  abntex2cite shim — deferred (deep / new-binding).
* Content-model `isn't allowed` (svg:g-in-block, XMApp-in-emph) = mostly SHARED
  (verified earlier). **`_` math-mode cluster (~80) = SHARED** — Perl-gated 4
  low-error samples (1910.00659/08936, 1912.03473/13019), ALL hit the SAME
  `_ Script _ can only appear in math mode` in Perl too (source-level: math
  content outside `$…$`, or `$$`-in-environment like linenomath). `}` mode-close
  (~66) = endgroup/mode-leak, mostly SHARED (docs).
**⇒ To find genuine Rust-only regressions, gate on Perl being CLEAN. The
fresh-worker re-sweep + Perl-gate is the way (slow). Most R10-R16 stage
failures are Rust-ahead/both-fail.**

### R10-R16 candidate triage + bib-setup-macro pattern (2026-05-28)

Re-tested promising 1-error candidates (fresh worker). Classifications:
* **SHARED / missing-class** (skip): 2006.15136 (`\orcid`, compositionalityarticle.cls
  missing), 2007.04509 (`\pagerange`, biom.cls missing), 2006.16481
  (`\papertitle` — Perl ALSO errors; multi-line `\def\papertitle{…}` is
  paper-buggy), 1910.04679 `\lpb` (used as Polish ł, likely a typo for `\l`).
* **HIGH-LEVERAGE Rust-only pattern — bibliography setup macros not run.**
  Journal bib styles define per-entry commands (`\betal`/`\byear`/`\bpages`/
  `\bmisc`/`\bnote`/… as `\@firstofone`) INSIDE a setup macro that the
  entry-type env/command calls. LaTeXML's bib handling doesn't run that setup,
  so they're undefined in the .bbl:
  - **imsart** (1912.11583): FIXED (`<this commit>`). imsart_cls.rs already
    hoists `\common@pub@types`'s identity `\let`s as a `def_macro_identity`
    list (`\bauthor`/`\byear`/`\bpages`/`\btitle`/`\bnote`/… = `\@firstofone`)
    but had OMITTED `\betal` (+ `\banumber`) — so a `.bbl` using
    `\begin{barticle}…\betal{…}` (bold-"et al." separator) saw only `\betal`
    undefined (siblings present). Added them. 1 error → 0 (Perl 5 errors here —
    imsart.cls "missing" in Perl — so Rust now far ahead). Binding-completeness
    pattern (chemformula/aas_support).
  - **abntex2cite** (1910.04251): same class — `\@bibitem`/`\@lbibitem` redef
    `\gdef\abntnextkey{#1}`, but LaTeXML's `\bibitem` DefConstructor bypasses
    `\@bibitem` → `\abntnextkey` undefined in `\bibciteEXPL{\abntnextkey}`.
  This bib-setup-macro-not-run pattern likely spans many journal-class papers
  → high-leverage dedicated-session target (investigate LaTeXML bib-env /
  `\bibitem` dispatch vs the class's `\@b*`/setup macros).
* `\pgfpl@@` (2005.10228): pgfplots internal — deep pgf gap.

### R10-R16 re-sweep + cortex_worker-staleness METHODOLOGY LESSON (2026-05-28)

**LESSON: `cortex_worker` is a SEPARATE binary — rebuild it (`cargo build
--bin cortex_worker --features cortex`) before ANY canvas sweep.** A
`cargo build --bin latexml_oxide` (or `cargo test`) does NOT rebuild
cortex_worker, so a sweep silently uses a stale worker. First R10-R16 sweep
(435 papers) used a 19:31 worker (latest commit 21:16) → wrongly reported
known-FIXED papers as failing (2001.10284 jmlr2e \BlackBox, 2001.03244
xwatermark→href, 1504.05963 dep-scan all showed "still-failing" but a FRESH
worker converts them clean: 18/44/300 warnings, 0 errors). Re-launched the
sweep with a freshly-built worker → `/tmp/resweep_fresh.tsv`.

**Cluster breakdown (stale-worker sweep, counts inflated but shape useful):**
dominant first-error clusters among still-failing were `_ Script _ … math
mode` (~65), `} Attempt to close a group that switched to mode …` (the
endgroup/mode-leak cluster, ~66 combined), `^ … math mode` (~15),
content-model `X isn't allowed` (~16), `\else`/conditional (~7),
`readBalanced ran out of input` (~3, xint-class). NEXT FIRE: read
`/tmp/resweep_fresh.tsv` (accurate), pick a still-failing 1-error paper,
Perl-gate, fix. The math-mode and mode-close clusters are large — if a
SUBSET shares a Rust-only root, high-leverage (but both clusters are known
to mix Rust-only + shared; sample several + Perl-gate to find the common
Rust-only trigger). `\abntnextkey` (1910.04251, abntex2cite .bbl) is a
known tangled one: LaTeXML's `\bibitem` DefConstructor bypasses
abntex2cite's `\@bibitem` redef (`\gdef\abntnextkey`), so `\abntnextkey`
is undefined when the .bbl's `\bibciteEXPL{\abntnextkey}` reads it; Perl
skips abntex2cite entirely (treats .sty missing). Would need an
abntex2cite init or a bibitem-mechanism change.

### FATAL/cascade triage + stale-canvas finding (2026-05-28) — recommend fresh re-sweep

Sampled the FATAL/TIMEOUT/OOM logs across stage_R10-R17 (the CONVERR_1
binding-gap candidates are largely exhausted/stale). Findings:
* **2006.12945** (`PushbackLimit` loop): STALE-recovered → 0 errors.
* **1910.03312** (runaway page-shipout, 5000+ `[N]` pages; 18829-line
  `hornshaw_qot_*` doc, heavy `\BeforeBeginEnvironment`/`\AfterEndEnvironment`):
  **SHARED** — Perl also times out (28 errors + fatal). Not a Rust-only win.
* **2003.02873** (`Timeout:TokenLimit` loop): **SHARED** — Perl
  `Fatal:too_many_errors:100`.
* `TooManyErrors:MaxLimit(100)` cascades (2008.00562 `\the$`, 2006.03833
  `\FirstAidNeededT` already-defined, 2006.01613 mode-close, 2005.12856 math
  `^`, 2005.10370 `\noalign`): all still cascading, varied roots, cascade-class
  (not clean Rust-only wins).
* **2001.10605** `not_tex_source` / `not_tex_source` PDF-magic: correct
  rejection (SHARED).

**Fresh re-sweep of stage_R17 (66 failures, current binary) — confirms massive
staleness:** **64/66 are now FATAL-free** (~97% recovered). Only 2 still FATAL:
* **2008.00562** — `\the$` cascade (`You can't use $ after \the`) → 101 errors
  → `TooManyErrors`. Source-level / cascade-class.
* **2008.07966** — `Fatal:Timeout:MemoryBudget` (RSS 4521 MB > 4500 cap), ZERO
  regular errors. Cause: `\input{dalpha-plot.tex}` = 809 KB of pgfplots
  `\addplot` data. **SHARED** — Perl also fails (times out at 180 s, 2 fatal
  "terminated"), BUT Perl peaks at only 1.28 GB RSS vs Rust's 4.5 GB. So a
  3.5× Rust memory-bloat EFFICIENCY gap on huge pgfplots data (a PERFORMANCE
  item, not a correctness parity gap — both engines fail). Not a Rust-only
  correctness win.

**⇒ stage_R17 has ZERO clean Rust-only failures** (64 recovered, 2 shared).
The Rust-only work in this stage is exhausted; the next fire should re-sweep
OTHER stages (corrected error-count: strip ANSI / parse the "Conversion
complete: N errors" line, my `^Error:` grep missed ANSI-prefixed lines) or
take a documented deep Rust-only candidate (xint/pgfplots).

**Stale-canvas reality:** across this campaign the MAJORITY of re-tested
CONVERR_1 candidates were already-fixed on the current binary (\lx ×10,
\c@tikztimingtrans ×9, \c@subalgorithm@save ×4, {mdfigure}, \specialrule,
\tagform@, \@inpenc@test, 2006.12945, …). The stage failure logs predate ~8
landed fixes this session. **Recommended next high-value step: a fresh
cortex_worker re-sweep of a recent-month chunk (release build) to surface the
TRUE current Rust-only failure set** — grinding the stale stage logs has
diminishing returns. The two known live Rust-only DEEP candidates remain
(2005.06787 xint `~`-escape, 2005.04851 pgfplots `_`; doc commit b2616561f8).

### FIXED: dep-scan skips packages required with conflicting options (`\def`-body false-positives) (2026-05-28)

**Witness 1504.05963** (`\documentclass[]{myaa}`, the A&A class family): Rust 1
error `unexpected:<char> Keyboard character used is undefined in inputencoding
ascii` (PB_ms:3150, a UTF-8 `ç` in "François") → **0 errors** (`<this commit>`,
772 KB HTML, "François" renders correctly); **Perl 0 errors**. FULLY
root-caused (traced via `LXML_DO` on `\DeclareOption`/`execute_option_internal`
+ both engines' logs):
* Both engines version-fall-back `myaa` → the generic `aa` binding
  (Perl log: `Info:fallback:myaa.cls Interpreted my as a versioned … falling
  back to generic aa.cls`; Rust loads aa_support, same `ExecuteOptions(…utf8
  hideoverfull…)` "unexpected" Info in BOTH — so utf8/hideoverfull being
  unhandled is SHARED and NOT the bug).
* The ONLY divergence: after the version-fallback, **Rust ALSO runs
  `maybe_require_dependencies(myaa, "cls")`** (content.rs:2010; the fallback
  path at 498-524 deliberately leaves `myaa.cls.ltxml_loaded` unset, comment
  525-529, to pick up the renamed class's bundled deps — helps e.g.
  `myclass`→caption, witness 2202.11535). That raw-text regex scan extracts
  `\RequirePackage[ascii]{inputenc}` from INSIDE the `\def\aa@inputenc{
  \RequirePackage[ascii]{inputenc}}` BODY (myaa.cls L93 — a deferred/conditional
  define, never actually executed) → loads `inputenc[ascii]` → the UTF-8-decoded
  `ç` (codepoint 231, in inputenc[ascii]'s 128-255 "undefined" range) errors.
* **Perl's version-fallback path does NOT call maybeRequireDependencies on the
  raw .cls** (it loads aa.cls.ltxml and stops) → never loads inputenc → `ç`
  stays a plain UTF-8 letter → clean. (NB: under an EXPLICIT
  `\usepackage[ascii]{inputenc}` BOTH engines error on UTF-8 chars — that part
  is correct/shared; the bug is the spurious inputenc[ascii] load.)
* **FIX LANDED (Option B):** `maybe_require_dependencies` (content.rs:1714)
  now pre-scans all `\RequirePackage`/`\usepackage` matches and SKIPS any
  package required with MULTIPLE CONFLICTING option sets (myaa requires
  inputenc with ascii/latin1/latin9/ansinews/applemac/utf8 — a clear
  "conditional `\def`-body" signature; only one is ever executed via
  `\aa@inputenc`). A genuine single-option require is unaffected, so real
  bundled deps (e.g. `myclass`→caption, 2202.11535) still load. Considered
  Option A (don't dep-scan after version-fallback, Perl-faithful) but it risks
  losing the `myclass`→caption pickup; Option B is more robust and keeps real
  deps. `cargo test --tests` 1344/0. Broad impact: A&A class family (myaa/aa)
  + any renamed class with `\def`-body `\RequirePackage`.

### R12/R16/R17 fixes (2026-05-28)

* **xwatermark stub: pull in hyperref (+ catoptions) like the real .sty**
  (`<this commit>`) — `xwatermark.sty` L31/L52 does `\RequirePackage{catoptions}`
  + `\usepackage{hyperref}`, so loading xwatermark makes hyperref's `\href`/
  `\url`/etc. available document-wide. Perl has no xwatermark binding → raw-
  loads it → gets hyperref. Our `xwatermark_sty.rs` stub (created to dodge the
  catoptions raw-load OOM cascade) no-opped the watermark API but OMITTED the
  hyperref dependency, so a paper loading xwatermark but not hyperref directly
  saw `\href` undefined — and since the only `\href` was in a `plainurl` .bbl
  (`\href{doi}{\path{…}}` DOI links), the WHOLE `<ltx:bibliography>` failed.
  Added `RequirePackage!("hyperref")` + `catoptions` (the safe Rust stub, not
  the cascade-prone raw load). **Witness 2001.03244** (`\usepackage[printwatermark]
  {xwatermark}`, `\bibliographystyle{plainurl}`, no direct hyperref): 1 error +
  empty bib → **0 errors**, full bibliography with 30 DOI links. Diagnosis hinge:
  Perl log showed `Loading dependencies … xwatermark.sty: catoptions,hyperref`.
  cargo test --tests 1344/0.
* **aas_support: add `\floattable` no-op (aastex62 layout macro)**
  (`<this commit>`) — `aastex62.cls` L4574
  `\def\floattable{\global\deluxestartrue\global\floattrue}` makes the next
  deluxetable a full-width float (two-column PDF layout). Neither our
  `aas_support_sty.rs` nor Perl's `aas_support.sty.ltxml` provided it (both
  route `aastex62` through the aas_support path, not a raw `.cls` load). Pure
  page-layout → moot for HTML (WISDOM #50), added as a no-op alongside
  `\placetable`/`\platewidth`. **Witness 1909.08916** (`\documentclass{aastex62}`,
  `\floattable` before deluxetables): 1 error → **0**. NOTE: Perl ALSO errors
  here (same aas_support gap — documented in KNOWN_PERL_ERRORS); this is a
  both-bindings-incomplete real-package macro, so Rust now converts where Perl
  still errors. cargo test --tests 1344/0.
* **More stale-log recoveries confirmed this triage (already 0-err):** the
  entire `\lx`-undefined cluster (1501.07631, 1505.07819, 1601.07412/07836,
  1602.03564, 1603.00071, 1703.08918, 1705.01609, … 10+ papers — all recovered),
  plus 2002.06879 (`{mdfigure}`). The stage logs were stale for these.
* **jmlr2e stub: add `\BlackBox` (end-of-proof QED box)** (`<this commit>`) —
  `jmlr2e.sty` (JMLR template, not in TeX Live — shipped with submissions)
  defines `\newcommand{\BlackBox}{\rule{1.5ex}{1.5ex}}`. Our
  `jmlr2e_sty.rs` stub provided the author-block + frontmatter macros
  (`\editor`, `{keywords}`, `\ShortHeadings`, `\firstpageno`, …) but omitted
  `\BlackBox`, so a JMLR paper ending proofs with `\hfill\BlackBox` saw it
  undefined. Mirror the real def (`\rule{1.5ex}{1.5ex}`). **Witness
  2001.10284** (`\usepackage{jmlr2e}`, `\hfill\BlackBox`): 1 error → **0**,
  644 KB HTML (Perl has 5 errors here — missing jmlr2e.sty → all 5 JMLR cmds
  undefined; Rust's stub handles 4, now 5/5). NOTE: 2001.07861 also uses
  `\BlackBox` but does NOT load jmlr2e → still undefined (buggy/SHARED, Perl
  errors too). cargo test --tests 1344/0.
* **Stale-recovered this triage (already 0 errors on current binary):**
  2002.04989 (`\specialrule` — ctable→booktabs require predates the stage
  log), 2004.10115 (`\tagform@` — amsmath). Confirmed via re-test; the
  stage_R12/R13/R14 logs were stale for these.
* **changepage: define `{adjustwidth*}` (separate env, was missing)**
  (`<this commit>`) — `changepage.sty` L122 has `\newenvironment{adjustwidth*}[2]`
  as a SEPARATE environment from `{adjustwidth}` (the `*` is part of the env
  NAME, not a `*`-argument). Our `changepage_sty.rs` stub defined only
  `{adjustwidth} OptionalMatch:* []{}{}` — and that `OptionalMatch:*` never
  matches `\begin{adjustwidth*}` because LaTeX dispatches the starred form as
  the env named `adjustwidth*`. Perl has no changepage binding → raw-loads the
  real .sty (both envs). Added a sibling `DefEnvironment!("{adjustwidth*}
  []{}{}", …, mode => internal_vertical)` (odd/even-page margin logic in the
  real def is moot — both branches just set list margins we ignore). **Witness
  2006.09676** (`\begin{adjustwidth*}{0.0in}{0pt}`): 1 error → **0** (588 KB
  HTML). cargo test --tests 1344/0.
* **chemformula stub now mirrors `\RequirePackage{...xfrac,nicefrac}`**
  (`<this commit>`) — the real `chemformula.sty` L29 does
  `\RequirePackage{tikz,amsmath,xfrac,nicefrac}`, so loading chemformula makes
  `\sfrac` (from xfrac) available to the document. Perl has NO chemformula
  binding: it raw-loads `chemformula.sty` and pulls in xfrac → `\sfrac`. Rust's
  `chemformula_sty.rs` stub (which maps `\ch`→mhchem `\ce`) preloaded only
  mhchem/l3keys2e/xparse and OMITTED xfrac, so a paper that loads chemformula
  and then uses `\sfrac` in *plain math* (not inside `\ch`) saw `\sfrac`
  undefined where Perl had it. Added `RequirePackage!("xfrac")` +
  `nicefrac` to the stub (NOT tikz — the stub renders via `\ce`, not
  chemformula's tikz arrows). **Witness 2006.07679** (siunitx + chemformula,
  no `\ch`; `\sfrac{\theta}{2}` in math): 1 error → **0** (667 KB HTML).
  Considered deleting the stub for a full raw-load (Perl-faithful — Perl
  *also* errors on `\ch`, both engines fail chemformula's expl3 body), but
  that regressed the `chemformula_raw_l3keys` trip test (which intentionally
  keeps `\ch`→`\ce` content-preserving, surpassing Perl). Extending the stub's
  RequirePackage to match the package's own declared deps is faithful and
  keeps both behaviors. `cargo test --tests` 1344/0.
* **`\catcode`/`\lccode`/`\uccode`/`\sfcode` Unicode codepoint truncation**
  (`<this commit>`) — these char-code registers (`tex_character.rs`)
  converted their numeric char-code argument with `(value_of() as u8) as
  char`, **truncating any codepoint > 255 to 8 bits**. So
  `\catcode`‹=\active` (U+2039 = 8249) set the catcode of `8249 & 0xFF = 57`
  = `'9'` instead of `‹` — silently making `'9'`/`':'` active+undefined.
  csquotes `\MakeAutoQuote*{‹}{›}` does exactly this, so any later literal
  `9` or `:` in the body raised `Error:undefined:9 T_ACTIVE[9]` /
  `T_ACTIVE[:]`. LaTeXML is Unicode-aware and Perl keys its catcode table on
  `chr($charcode)` with no truncation. Added `charcode_to_char()`
  (`char::from_u32`, 8-bit fallback only for out-of-range/surrogate codes)
  and routed all four registers through it. **Witness 2007.09691**
  (`\MakeAutoQuote*{‹}{›}` + biblatex): `2 errors + 2 undefined macros[9,:]`
  → **0 errors**, 2.5 MB HTML (Perl baseline had 2 errors — Rust now
  cleaner). Minimal repro: `\usepackage{csquotes}\MakeAutoQuote*{‹}{›}` then
  `Page 9 at 10:30.`. Broad impact: any package activating a >255-codepoint
  char (csquotes inner quotes, babel shorthands on Unicode chars, …).
* **`ltx:_CaptureBlock_` content-model parity** (`<commit 1cf95cb583>`) — our
  `Model::load_internal_extensions` synthesized `_CaptureBlock_` from only
  4 sources (`ltx:block`, `ltx:logical-block`, `ltx:sectional-block`,
  `Caption`); Perl `Common/Model.pm` L96-97 uses 6, also including
  `FrontMatter` and `BackMatter`. Added the two missing sources so a
  captured box holding frontmatter/backmatter content is modelled as
  permissively as Perl. (Parity correction; does not by itself resolve
  the 2007.07021 listingline close-recovery divergence below.)
* **thmtools: drop divergent native `restatable`, require thm-restate**
  (`<this commit>`) — Perl `thmtools.sty.ltxml` defines no `restatable`
  env (it comes solely from `thm-restate.sty`), and the real
  `thmtools.sty` L47-49 does `\RequirePackage{thm-patch, thm-kv,
  thm-restate}`. Our `thmtools_sty.rs` had added a native
  `DefEnvironment!("{restatable}…")` that (a) diverged from Perl and (b)
  blocked thm-restate's clean `\newenvironment{restatable}` — LaTeX's
  `\newenvironment` refuses to redefine an existing env, so loading
  thmtools-then-thm-restate left the buggy native version active. That
  version digested the store-name arg (3rd) in text mode, so a name with
  `_` (e.g. `two_var_indp`) raised `unexpected:_ Script _ can only appear
  in math mode` once per use. Removed the native env and added
  `RequirePackage!("thm-restate")` so `\usepackage{thmtools}` still
  provides `restatable` (matching the real package). Witness 2007.12335
  (`\begin{restatable}{theorem}{two_var_indp}`: 9 errors → clean,
  matching Perl).
* **`\@checkend` stray-brace removal** (`8ca20da419`) — Perl
  `latex_constructs.pool.ltxml` L190 transcribed the LaTeX-kernel
  `\def\@checkend#1{…\fi}` *including* the `\def`'s closing `}` into the
  `DefMacro` body, so every `\@checkend{env}` emits an unmatched `}`.
  LaTeXML's own `\begin`/`\end` skip `\@checkend`, so it's normally
  invisible — but a package that redefines `\end` the kernel way to call
  `\@checkend` (e.g. `extract.sty`'s `AfterEndEnv` machinery, pulled in
  by `\usepackage{extract}`) runs the stray `}` inside its wrapping
  `\begingroup`, yielding one `Error:unexpected:} Attempt to close
  boxing group … due to \begingroup` per environment. Perl's gullet
  tolerates the extra brace; ours errored. Dropped the artifact (matches
  standard-LaTeX semantics). Witness 2007.09971 (IEEEtran+extract under
  ar5iv: 41 errors → clean / 9 warnings, matching Perl). See
  KNOWN_PERL_ERRORS.md #25.
* **biblatex `\providetoggle` for `blx@` toggles** (`ba35223039`) —
  bundled `mybiblatex.sty` wrappers re-enter biblatex init via a path
  the `_loaded` guard doesn't cover, so the 56 `\newtoggle{blx@…}`
  allocations hard-errored 55× on already-defined toggles. Switched
  the trailing RawTeX block to `\providetoggle` (idempotent
  define-if-absent). Witness 2007.06815 (55 errors→clean), verified
  2007.13391/.13597/.13644/.13719.
* **microtype `\microtypecontext` declaration vs environment**
  (`<this commit>`) — microtype defines BOTH the `\microtypecontext{settings}`
  scoped *declaration* (no body) and a `{microtypecontext}` environment.
  Our `DefEnvironment!("{microtypecontext}")` defines the bare
  `\microtypecontext` CS as the env-begin, clobbering the declaration;
  a bare `\begingroup\microtypecontext{expansion=sloppy}…\endgroup`
  (common around `\bibliography`) then treated `\microtypecontext` as
  an unclosed env-begin, opening a restricted_horizontal mode-switch
  group the `\endgroup` couldn't close (`unexpected:\endgroup`).
  Defining the env FIRST and the no-op declaration macro AFTER lets
  `\microtypecontext{…}` resolve to the harmless declaration while
  `\begin{microtypecontext}` still finds the env (env lookup is
  independent of the CS). Witness 2007.06927 (CONVERR_1→clean), both
  declaration and environment forms verified.

**R17 SHARED (not actionable) — `^`/`_`/XMApp-in-emph cluster.** The
remaining R17 CONVERR tail is dominated by `Error:unexpected:^`/`_
Script can only appear in math mode` plus `malformed:ltx:XMApp/XMWrap
isn't allowed in <ltx:emph>` — all **SHARED with Perl** (author errors:
real `^`/`_` math symbols typed in text/emph without `$…$`). Verified
Perl produces the *identical* error counts on the worker's actual main
file (NB: several of these zips ship multiple `\documentclass` files —
classify against the LARGEST/worker-selected main, not the first
alphabetically): 2007.06816 (Perl 9), 2008.00074 (12), 2007.09876 (11),
2008.00163 (15, `FUSED_JMLR_Omni_arxiv_June16.tex` not `jmlr_sample.tex`),
2007.07599 (11, svjour3), 2007.15143 (10). Do **not** re-investigate as
Rust-only. Also SHARED (verified Perl=identical): 2008.01188
(figure-in-quote, Perl 9), 2007.15203 (bibitem-in-itemize, 7), 2008.01181
(`\fi`/`\else` outside conditional, 6), 2007.15479 (5), 2008.00502
(natbib `\NAT@citetp`/`\NAT@parfalse`/`\NAT@swafalse` undefined +
`\lx@note` mode errors, 7).

**R17 DEFERRED — document-builder close-recovery divergences (Rust-only,
high-risk core).** Two remaining failures are genuinely Rust-only but
both stem from how the document builder closes/recovers open
boxes/blocks when an ancestor closes — a core area the user flagged as
sensitive (math-id/ASF). Defer pending careful, well-tested work:
  * **2008.00562 `IAIPAL-SIAM-Ver6.tex` (siamart190516) — FATAL
    TooManyErrors, Perl=0.** Root cause: ntheorem's binding (loaded as a
    siamart dependency in BOTH engines; ntheorem `RequirePackage`s
    amsthm) sets `\qed`=`\@qedbox{\the\qedsymbol}` with `\qedsymbol` a
    toks-register. The paper does `\renewcommand\qedsymbol{${\small
    \blacksquare}$}`, turning `\qedsymbol` into a *macro*; amsthm's
    `proof` env auto-inserts `\qed` at `\end{proof}`, so `\the\qedsymbol`
    expands `\qedsymbol`→`$…$` and `\the$` errors, leaving an unclosed
    inline-math `$` inside the `\@qedbox{…}` arg. Perl recovers at the
    group boundary (auto-closes the leaked math); OUR stomach raises
    `unexpected:\endgroup Attempt to close a group that switched to mode
    math` and the leaked math corrupts all following content → 100+
    `_`/`^`/`\lx@end@inline@math` errors → FATAL. The single-proof case
    is SHARED (Perl also emits 2-8); the FATAL *cascade* is the Rust-only
    amplifier. Fix needs Perl-like math-mode group-close recovery in the
    stomach — broad/risky, NOT a per-paper shim. (Confirmed: removing
    amsthm from `siamart_cls.rs` does NOT help — ntheorem loads amsthm
    regardless, matching Perl.)
  * **2007.07021 `SSGL_GLM.tex` (amsart) — Perl 4, ours 6.** The 4
    shared errors (2× `_CaptureBlock_` "isn't open", 2× `enumerate` not
    allowed in `listingline`) are author/structural (enumerate inside an
    algorithmic listing line). Our 2 EXTRA: `ltx:listingline Closing tag
    whose open descendents do not auto-close. Descendants are
    _CaptureBlock_` — a close-SEQUENCE divergence (`_CaptureBlock_` has
    no `autoClose` in either engine, but Perl closes it before the
    `listingline` close is reached). Same close-recovery class as above.

### R15–R16 fixes (2026-05-28)

Engine/binding fixes landed driving the remaining-list canvas through
the 2005-2007 range (all verified Perl-faithful):
* **siamart `{@abssec}`/`{@doisec}`** (`c5f3e7eca2`) — titled-section
  envs (inline-logical-block); 2005.11911 (surpass-Perl, Perl has 24
  errors).
* **autofe.sty no-op stub** (`6a571cfb42`) — ucs/utf8x's autofe
  activated LGR and transliterated Latin→Greek in CS-name building
  (`\thesection`→`\theςεςτιον`); Perl skips ucs as missing-file.
  Witness 1701.05945, 1703.07562, 1702.05510.
* **revtex4-1 `\doi` HyperVerbatim** (`d82ad1e6ba`) — `\doi{…%2F…}`'s
  `%` (not in Semiverbatim's SPECIALS) commented out the closing `}`,
  causing readBalanced-to-EOF + infinite pushback FATAL. HyperVerbatim
  does `begin_semiverbatim(['%'])` (the `\@sanitize@url` analogue).
  Witness 2006.12945 (FATAL→OK), bisected to one bibliography DOI.
* Earlier in the run: vntex/babel-vietnamese T5 (`96aec2dfc8`/`52a3a72ff4`),
  `\textviet` (`ac3df4d520`), apacite `\BOthersPeriod` (`14db9baf64`),
  amsmath subequations token-level `\theparentequation` (`6fb7e001c7`),
  named-color dvipsnam lazy-load (`b6f8117a94`).

Remaining R13-R16 tail confirmed dominated by SHARED (glossaryref+math,
braced `\be/\ee` equations, bundled custom classes, display-math-in-
caption) and the deferred ASF `expected:id` MathFork tail (see deferred
sections below). Easy package/binding wins exhausted in this range.

### Audit findings (2026-05-27)

**Branch-commit audit completed.** 33 commits since master, 7 touch
engine code, 26 are pure-doc updates. Code-commit summary:

| Commit | Status | Notes |
|---|---|---|
| `66effc0157` (logger \n) | harness | canvas Error-line counter |
| `5d78ca1325` (LOSTNODES port) | root cause | Perl `MathParser.pm` parity |
| `d46541f60c` (xml_safe_char + ASF) | mixed → **intentional divergence #27** | xml_safe_char marked in OXIDIZED_DESIGN.md; ASF half is correctness |
| `1625353bd9` (defer XMath unlink) | root cause | Perl `Post.pm` L373-393 parity |
| `18fe803244` (cmml depth cap 4096) | shortcut (superseded) | bug locus identified, deferred |
| `81061469fc` (cmml cycle guard) | shortcut | confirmed SHARED with Perl |
| `56dc9497fc` (`\scalefont {Float}`) | root cause | Perl `\scalefont{}` parity |

**cmml cycle bug locus** (witness arXiv:1505.06709, math `S4.E82.m1`):
Traced via `LATEXML_CMML_TRACE_CYCLE=1` (added in `01e5b04a24`).
The XMath emitted by `amsmath_sty.rs::rearrange_ams_split` (the AMS
`split`/`gather` rearrange that wraps a parsed XMArray in an XMDual
whose content-arm is `XMWrap rule="Anything,"` containing
`createXMRefs(cells)`) sometimes produces an XMRef whose idref
resolves back to the wrapping XMDual itself — i.e. one of the
`cells` had the same xml:id as the wrapping XMDual eventually got
assigned. cmml then follows the XMRef → XMDual → content-arm-XMRef
→ XMDual → ... in an infinite loop.

**This is SHARED with Perl**: `LaTeXML/lib/LaTeXML/Package/amsmath.sty.ltxml`
L302-306 and L368-372 build the *exact same* tree (`replaceTree(['ltx:XMDual', {},
['ltx:XMWrap', { rule => 'Anything,' }, createXMRefs(...)], $array], $array)`).
Perl just doesn't OOM/abort on the cycle because Perl's interpreter
stack is much deeper than Rust's 256 MB worker stack — cmml-as-defined
walks the self-reference indefinitely, but Perl `no warnings 'recursion'`
absorbs the warning and presumably finishes (slowly) in some cases or
silently fails in others. Cycle guard remains the correct defensive
measure; the actual root cause (rearrange-arm's XMDual id colliding
with an inner cell's id) requires careful `createXMRefs` / id-collision
handling in the rearrange pass.

Recommendation: keep cycle guard, file follow-up to fix
`rearrange_ams_split` so the XMDual-vs-cell id collision can't occur
(then cycle becomes dead code, depth cap stays as truly-deep safety
floor).

**⚠ Canvas harness fix (2026-05-26):** the `run_one.sh` Error-line
counter used `grep -cE $'^\\x1b\\[31mError:'` — the `^` anchor never
matched because the engine writes Error lines mid-line after content
+ `\r` + ANSI escape, not at line start. Result: papers with non-fatal
errors were silently classified `OK` instead of `CONVERR_N`. Fixed by
removing the `^` anchor. Stage_53+ will produce accurate CONVERR
classifications; stages 01-52 stats may overcount OK (logs for OK
papers were deleted, so retro-classification not possible).

**Dominant CONVERR cluster — fix landed 2026-05-26 (`1625353bd9`).**
With the new error-line counter applied to a stage_51-fresh sample
(2026-05-26), ~63% of CONVERR papers were emitting `Error:expected:id
Cannot find a node with xml:id=...` from the post-processing
`mark_xm_node_visibility` walk. Root cause: `process_math_node`
unlinked XMath eagerly after the first math-format processor (PMML).
The second processor (CMML) then dereferenced live XMRef idrefs into
the freed subtree, and `find_node_by_id` returned None for every
target id. Perl `Post.pm` L373-393 marks ids reusable but defers the
actual `unlink` ("XMath will be removed (LATER!)"). Rust now mirrors
that: `PostDocument::defer_xmath_unlink` queues the subtree;
`Post::process_chain` calls `drain_pending_xmath_unlinks` once after
every processor in the chain has run. `DocOwnedNode` wrapping is
preserved in the drain pass (cycle-236's `$X$` + ar5iv SIGSEGV
reproducer remains green). Two witnesses confirmed clean:
arXiv:1503.05614 (was CONVERR_1) and 1501.05180 (was CONVERR_1;
combined with the `xml_safe_char` U+FFFD fallback from `d46541f60c`).
Tests: 1344 passed / 0 failed (mathtools.xml re-blessed: 2 XMRef
idrefs now match ASF-correct LOSTNODES output).

### Driver

Beyond-Perl showcase (issues #47/#92): live source↔preview + linting via
source locators. Full design in
[`SOURCE_PROVENANCE.md`](SOURCE_PROVENANCE.md).

**Scope:** line-level, block/inline-element granularity, **math opaque**
(= SyncTeX granularity). Columns, per-leaf char-offset maps, and in-equation
provenance are deferred. **Parity-neutral and off by default** — a normal
conversion (switch off) must stay byte-identical to today; build on the
existing `Locator` model (`common/locator.rs`) **unchanged**.

**Attribute contract (decided 2026-05-24, web-ecosystem audit — see
SOURCE_PROVENANCE §0/§0.1/§2):** attribute name **`data-sourcepos`** (the
cmark-gfm/GitHub/GitLab convention; *not* `data-src`, which is the lazysizes
lazy-load idiom). Value `tag:l:c-tag:l:c` — file **first-class** in each
endpoint, integer `tag` = index into a doc-level `sources` table
(Source-Map-v3 `sources`/`sourceRoot`/`sourcesContent` flavour: compact,
anonymisable, no inlined paths). Serialise via a new compact
`Locator::to_sourcepos()`; the latent XPointer `Locator::to_attribute()` is
**not** used (zero web-platform support). Rung-2 char map keeps `data-srcmap`.

Engine-substrate checklist:

- [x] `--source-map` flag (+ `LATEXML_SOURCE_MAP` env), off by default,
      gating *both* tracking and emission via the `State.source_map` field
      (`state::source_map_enabled()`); threaded Config → CoreOptions →
      StateOptions, mirroring `nomathparse`. Scaffold test
      `tests/52_source_map.rs` pins off-by-default (no `data-sourcepos`) +
      ON-currently-inert (byte-identical). Verified: corpus binary path
      (`cortex_worker`) keeps `source_map: None`.
- [ ] Start-*line* capture in `mouth.rs::read_token` (`:628`), after
      inter-token skips; range open→close at the digestion frame via
      `Locator::new_range` (`locator.rs:80`). Gated by `source_map_enabled()`
      and cached into the Mouth so the hot path is zero-cost when off.
- [x] Stamp elements with `data-sourcepos` in **`open_element_at`** (the
      shared element-creation primitive — covers plain `open_element`, math,
      and alignment uniformly), via `Locator::to_sourcepos(tag)` (integer
      `sources`-table tag, no paths). Box locator captured as a `Copy`
      `Locator` at `set_box_to_absorb` time (`current_box_locator`) to avoid
      the `RefCell` re-borrow panic mid-`be_absorbed`. Gated.
      - **Deferred:** the `ltx:Math` *wrapper* is stamped at digestion but the
        Marpa math parser rebuilds the subtree (`base_xmath.rs:1410`) and
        discards it (§7 A.3 — math-parse provenance). Math stays opaque;
        equations inherit the container's locator client-side. Math internals
        (`ltx:XM*`) are skipped by design.
- [x] Propagate `data:sourcepos` through the post XSLT into HTML
      `data-sourcepos`. Done via **Perl parity**: emit in LaTeXML's `data:`
      namespace; `Document::set_attribute` now mirrors Perl's
      `getDocumentNamespacePrefix($ns,1)` — it **promotes a namespaced
      attribute's namespace to a document namespace** on first use, so finalize's
      `apply_document_namespace_declarations` declares `xmlns:data` on the root,
      the literal `data:sourcepos` resolves into that namespace on serialize, and
      the existing `copy_foreign_attributes` (`LaTeXML-common.xsl`) converts
      `data:` → `data-` (`USE_DATA_ATTRIBUTES` = HTML5). No XSLT change — same
      path `aria:` already uses. General fix (any namespaced attr; implements the
      long-standing `decodeQName` TODO); verified parity-neutral on
      structure/complex(aria)/tikz(xlink). See [[refcell-digestion-debt]] sibling
      `WISDOM.md` note.
- [x] User-vs-foreign source: stamp only into editable user docs
      (`.tex`/`.ltx`). This skips both synthetic default locators (source =
      `locator.rs` from `Locator::default()`'s `file!()`) and foreign
      `.cls`/`.sty`/dump files; foreign/unstamped elements inherit the nearest
      user-source ancestor client-side. (MVP extension heuristic; a tracked
      user-input set would be more precise.) Verified on `article.tex`:
      265 → 53 stamps, all `tag 0 = article.tex`, real line:col positions.
- [x] **MVP locator test** (`tests/52_source_map.rs`, 3/3): off-by-default
      emits no locator; ON emits `data:sourcepos` in core (user-source only,
      math-opaque, shape `tag:l:c[-tag:l:c]`); ON round-trips to HTML
      `data-sourcepos` (the XSLT pass-through). Future hardening (not blocking
      MVP): pin an exact `data-sourcepos` golden; corpus round-trip (literal
      range substring == visible text; range ⊆ parent; within file bounds) +
      debug-assert invariants. Self-contained (no SyncTeX dependency).
- [x] **Coverage:** constructor-built elements now capture a real locator.
      `Definition/Constructor.pm` L106 parity — `constructor.rs` sets
      `whatsit.locator = gullet::get_locator()` (gated on `source_map_enabled()`
      so the corpus path pays nothing and stays byte-identical; the whatsit
      locator only feeds source-map + untested error messages). Previously every
      `DefConstructor` whatsit got `Locator::default()` and was dropped by the
      user-source filter. Result on `article.tex`: **53 → 128** stamps with real
      line:col ranges (e.g. `\section` line, equation lines). Full suite green.
- [x] **Cleanup: `Option<Locator>`.** Replaced the `Locator::default()`
      `file!()/line!()` *sentinel* with an honest `Option<Locator>`:
      `Object::get_locator -> Option<Locator>`; `Whatsit`/`Tbox`/`List.locator:
      Option<Locator>`; `List::new` → `find_map`. The free fn
      `gullet::get_locator() -> Locator` is unchanged (the "where the parser is
      now" workhorse for errors + box creation). Cross-cutting (17 files: trait +
      all box types + ~21 call sites); full suite green, parity-neutral. Aligns
      with the "meaningful Rust types" goal. (Rejected: a stateful gated
      `Whatsit::default()` — `Default` must stay pure.)
- [~] **Column precision — needs Tier B, NOT a quick fix (attempted + reverted
      2026-05-24).** Tried Bruce #101's proposed fix: `read_token` token-start
      (`last_token_start`, the `from` of `get_locator`) + capturing the
      construct's open locator in `Constructor::invoke_primitive` *before* args.
      **Empirically REGRESSED** the common cases: `section` `12:1`→`12:9` (the
      `{`), `itemize` `40:1`→`40:15` (the `}`). Reason: `\section`/`\begin{…}`
      reach their element constructor via **expansion** (`\@startsection`,
      `\begin`), so `invoke_primitive` fires *after* the user's keyword — the
      open locator is the post-keyword position, not the command start. This is
      **Bruce's #3 (invocation-span vs macro-origin)** — accurate construct-start
      needs **expansion-provenance** (tag expansion frames with the invocation
      locator; propagate to the constructor) = the deferred **Tier B**
      (`SOURCE_PROVENANCE §3`), genuinely hard, no clear bounded change. Do NOT
      re-attempt the naive `invoke_primitive` capture. **LINE accuracy already
      meets the MVP bar** (every construct on its correct source line, verified
      on `article.tex`); the ar5iv-editor scrolls by line, so columns are a
      post-MVP refinement gated on Tier B.

Next phase (after substrate): warm-state conversion server (full-doc
reconvert MVP) → ar5iv-editor + VSCode-extension clients. Deferred to
post-MVP: columns/`data-srcmap` (§6 rung 2), in-equation/math-parser
provenance (§7 A.3), Tier B expansion provenance.

## Round-27 parity clusters

### Handoff — `ar5iv.sty` package-option keyvals (`tokenlimit` etc.)

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
| **Total papers** | **500,000** processed |
| **Recorded result** | 499,832 OK = **99.9664%** (canvas time, 2026-05-15..22) |
| **Post-fix projection** | **499,984 / 500,000 = 99.9968%** (per the 2026-05-26 retest of all 168 historical fatals: 152 now produce HTML output; only 16 NO_HTML — 3 corpus-invalid, 8 SHARED-FAILURE timeouts, 4 OOM, 1 Rust-only timeout) |
| **Best stage** | stage_49 at **99.99% (9999/10000)** |
| Failure distribution (recorded) | 126 FATAL_3, 16 OOM, 15 TIMEOUT, 4 FATAL_139, 3 FATAL_101, 3 FATAL_1, 1 FATAL_134 |
| Tests | **1,344 / 0 / 0** (post-merge with master) |
| Branch | `large-scale-testing-round-3`, 960+ commits ahead of `origin/master` (post 2026-05-26 merge) |
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

* **25000**: 14674 PASS / 10290 WARN / 27 errors / 9 FATAL —
  **99.964% non-fatal, 58.7% clean pass**. ALL 9 FATALs accounted
  for: 7 SHARED with Perl + 2 fixed Rust-only (envmath, maketitle).
* **50000 (interim, 1387 processed)**: 1385 OK / 2 "FATAL_1". Both
  "FATAL_1" are *driver-level* `pack_archive` errors after a
  successful conversion — `Info:latexml::converter Conversion
  complete: N warnings` then `Error: No such file or directory
  (os error 2)` from `add_dir_to_zip`'s `File::open(&path)?` (a
  TOCTOU on mutool-generated PDF→PNG intermediates). **Zero engine
  fatals at 50K-sample scale.** Post-processing driver issue,
  not conversion correctness.

* **arXiv:1711.02043 confirmed SHARED-FAILURE (2026-05-26)**:
  Earlier R36 bisection bottomed out at preamble
  `\def\docAuthor{M. Sezer Erk{\i}l{\i}nc{c}}` combined with
  hyperref `pdfauthor=\docAuthor`. Re-tested Perl on the same
  minimal article — Perl also infinite-loops, allocating
  2.35 GB+ at 99% CPU until killed. Our 650K-PushbackLimit
  safety net trips at ~3s; Perl has no comparable cap and just
  consumes memory. **Pinned as SHARED-FAILURE, not Rust-only.**
* **arXiv:1802.02070 (revtex4-1) — still timing out**: 180s
  budget, package loading completes (`hhline.sty` is last preamble
  closure), then digestion of the body times out at
  `Timeout/Convert`. Not yet bisected to a specific construct.

Sampling-driven stubs landed:
* `3e4e0cc25d` — rotfloat (witnesses: arXiv:2101.12526, 1804.05845).
* `00412df771` — tabls (witness: arXiv:2003.12942).
* `be19874ba0` — chemnum (witness: arXiv:2103.03138).
* `edeb9b62f7` — pax (witness: arXiv:1512.06235).
* `fd85f769c9` — figcaps (witness: arXiv:1912.07260).
* `d0c5f760ed` — refstyle (witnesses: arXiv:1804.06350, 2009.10518).
* `7bc8a6cec9` — envmath (witness: arXiv:1501.05259, a real
  Rust-only PushbackLimit fatal).
* `44e1097eef` — maketitle fatal-flag restoration (witness:
  arXiv:1903.01633, a sneaky silent fatal — the deferred
  frontmatter digest was swallowing Err but leaving fatal=true).

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

**Re-retest 2026-05-26 (current binary, properly exit-captured)**:
7/16 PASS, 9/16 still FATAL — confirming the earlier 2026-05-23
classification holds. PASS: hep-th0009218, physics0003074,
math0009192, gr-qc0209055, math0104252, gr-qc0301024, hep-ph0012156
(0.5–51s). Still FATAL with `Fatal:Timeout:MemoryBudget`:
math0102053, math0102089, math0212126, math0402448, math0504436,
math0506088, math0507219, math0604321, math0203082 — all plain-TeX
papers (no `\documentclass`, `\catcode @=11`, `\magnification`,
custom `\newcount`/`\loop`). The "plain TeX MemoryBudget" cluster
remains an open Rust-vs-Perl perf gap: Perl converts each in ~0.2-30s,
Rust exceeds the 4.5 GB RSS cap. Engine work for memory-efficient
plain-TeX digestion is deferred.

(A 2026-05-26 retest claiming "all 16 recovered" was retracted —
the test script captured `$?` after a `| tail` pipe, so every exit
code read as 0 regardless of cortex_worker's outcome.)

### Full 168-paper canvas_3 FATAL retest (2026-05-26, current binary)

Re-ran the 168 papers that fataled across canvas_3 stages 01–50
against the current binary (post-merge with master) using a
proper output-classifier (`HTML_OK` if `Output written to`
appears in log; `NO_HTML` otherwise).

**Result: 152/168 now produce HTML output (90.5% recovery).**

| Category | Count | Note |
|---|---:|---|
| `HTML_OK` (success) | **152** | conversion produces HTML, exit-code may still be 3 if 100-error cap tripped |
| `NO_HTML` total | 16 | |
| ↳ corpus-only (PDF/empty zip) | 3 | 0901.2851, 1201.2466, 1407.7289 — not engine bugs |
| ↳ wallclock timeout (120s) — SHARED with Perl | 8 | 0708.3218, 0708.3398, 1001.3154, 1009.3622, 1101.2531, 1202.2643, 1302.3919, 1407.1983 — Perl also times out (60s budget Terminated each time, pictex/heavy-graphics chains) |
| ↳ wallclock timeout — Rust-only | 1 | 1404.6225 — Perl completes in 23.6s with 11 warnings + 1 error; Rust hits 120s cap (heavy elsarticle + tikz + many missing-style packages) |
| ↳ SIGKILL=137 (OOM during build) | 4 | 1106.3552 (Scientific Word bbl), 1304.5520 (hypcap raw-load), 1405.5891 (algorithmic env in spconf context), 1406.4689 (tikz/pgfplots) |

**Updated 500K canvas_3 success projection.**
Original recorded: 499,832 OK / 500,000 = **99.9664%**.
Plus 152 recovered: **499,984 OK / 500,000 = 99.9968%**.

After Perl-parity verification on the 9 wallclock cases:
**Only 5 true Rust-only failures remain** (4 OOM + 1 wallclock),
plus 3 corpus-only and 8 SHARED-FAILURE timeouts.

**Open follow-up clusters (no fix yet):**
- 1404.6225 (Rust-only) — heavy elsarticle preamble (tikz +
  todonotes + soul + ctable + many missing-style packages).
  Perl 24s vs Rust 120s+ timeout. Perf gap in package-load and/or
  per-CS expansion. Even at 300s timeout, Rust produces 0-byte HTML.
- OOM during XML build (4 papers) — each fails via a different
  combinatorial path:
  * 1405.5891 — `abstract end + algorithmic env` in full paper
    context.
  * 1106.3552 (bisected 2026-05-26) — triggered by
    `\appendix\setstretch{1} \scalefont{0.8}\newpage` at line 2002
    of the body in the full 2001-line prelude. Minimal repro of the
    same constructs converts cleanly. RSS jumps from <1 GB to 60 GB
    in 30s after this line. State accumulation interacts with the
    `\scalefont` font-merge in some unidentified way.
  * 1304.5520 (hypcap) and 1406.4689 (tikz/pgfplots) — similar
    "minimal repro fine, full paper OOMs" pattern.
- SHARED-FAILURE timeouts (8 papers) — engine recovery ceiling,
  Perl also fails. Mostly pictex / pst-all chains.

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

### Deferred Rust-only: `expected:id` MathFork/split dangling-XMRef tail (R16 analysis 2026-05-28)

The residual post-processing `Error:expected:id Cannot find a node with
xml:id='…'` cluster (witnesses 2006.06709 `A8.Ex87.m1.5`, and R10-R16
shapes `S6.E4.m1.2.mf`, `S2.Ex12.m1.2`, `A2.E12.1.m1.2.mf`) is the
MathFork (`.mf`) / `rearrange_ams_*` XMRef-cloning issue. `append_clone`
(document.rs L3869) suffixes cloned ids via `ID_SUFFIX=".mf"`
(base_xmath.rs L1580) and rewrites idrefs through `id_map`, but some
cloned XMRefs end up referencing an id that no node finally carries.
`prune_dangling_split_xmrefs` (document.rs L3173) already removes such
refs in two sweeps: (1) any `@_split_ref`/`@_mf_ref`-marked ref whose
idref doesn't resolve; (2) a broader regex sweep — **but restricted to
`^S\d+\.E\d+\.m\d+\.`** (numbered `\begin{equation}` form) so it
explicitly EXCLUDES the `Ex<digit>` (unnumbered display) and `A<digit>`
(appendix) forms, because declare_test has legitimate renamed-id refs
(`S1.Ex1.m1.1`→`.1a`) that a naive broadening would wrongly prune.
So the surviving danglers are precisely the `Ex`-form / `A`-prefix
*unmarked* refs that fall outside both gates. Safe fix needs a
provenance marker (extend `_mf_ref`/`_split_ref` tagging to ALL
MathFork/rearrange-cloned refs) so sweep (1) catches them without
touching declare_test — careful ASF work, do WITH the user (per their
"be careful with the abstract syntax forests / math id" guidance), not
an autonomous patch.

### Deferred Rust-only: `to_string`-space-loss class (control-word + letter merge)

A recurring engine pattern: code that serializes a token list to a
string and re-tokenizes drops the inter-token space that separates a
control word from a following letter, merging e.g. `\rm S`→`\rmS`,
`\vb h`→`\vbh`. Confirmed instances:
* **physics `\dmat`/`\admat`** — FIXED `9e5ab794e1` by splitting at the
  token level (`split_tokens`) instead of `to_string()`+re-`Tokenize!`.
* **`\rmS` via `\renewcommand{\theequation}{{\rm S}\arabic{equation}}`
  in subequations** — FIXED `6fb7e001c7`. Witness 2005.06712. The site
  was `\lx@equationgroup@subnumbering@begin` (latex_constructs.rs
  ~L5689) `.to_string()`+re-tokenize of the expanded `\theequation`
  when fixating `\theparentequation`; now keeps the token list (Perl
  `\protected@edef`). Subequation tags `(S15a)` now correct.

### Deferred Rust-only investigation: fundam.cls raw-load drops late `\def`s

Witness 2005.04818 (`\documentclass{fundam}`, Fundamenta Informaticae,
bundled `.cls`). fundam.cls is raw-loaded (not OmniBus — no fallback
warning). Probe of `\@ifundefined` after load shows a *non-contiguous*
DEF/UNDEF pattern: `\issue`(L38)/`\papernumber`(L40)/`\runninghead`
(L42)/`\abstract`(L83)/`\and`(L81) are **defined**, but `\@maketitle`
(L47) and `\keywords`(L91) are **undefined**. In a plain `article`,
`\def\@maketitle{…}`+`\def\keywords{…}` both define fine — so it's
specific to the raw-`.cls`-load path. Prime suspects: the long
`\def\@maketitle{…}` body (L47-79, contains
`\begin{tabular}…\end{tabular}`, `\iffour…\else…\fi`, `\pageref`) or
`\AtEndDocument{\label{::last page of FI article:\jobname::}}` (L45,
colons + `\jobname` in a `\label`) mis-consuming following tokens.
`\keywords` is the user-visible error. Not yet confirmed vs Perl
(Perl times out >160s on this paper). Deferred — needs a focused
raw-load `\def`-body / `\AtEndDocument`-arg trace.

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
* **math tokens inside `ltx:glossaryref`** — a glossary reference
  (`\gls`/`\glspl`) whose displayed content includes math (or is used
  in a math/algorithm context) emits bare `ltx:XMTok`/`ltx:XMApp`
  directly under `ltx:glossaryref`, whose content model is
  `Inline.model` (no bare math tokens). Verified 2005.04232 (R15):
  Perl emits byte-identical `ltx:XMTok isn't allowed in
  <ltx:glossaryref>` at source line 1011. Cluster: 114 occurrences
  across 5 R13-R16 papers (2003.03080, 2004.07271, 2004.09272,
  2005.04232, 2006.10102). SHARED schema limitation in both engines.
* **`{… \be \int_{…} … \ee}` braced custom begin/end-equation
  shorthand** — author wraps a `\be…\ee` display equation in an extra
  brace group; both engines fail to enter display math through the
  brace, so the `\int_{…}` subscripts trip `unexpected:_ Script _ can
  only appear in math mode`. Verified 2006.05110 (R16): Perl emits 21
  errors at the same source line (305) vs our 13 — we're strictly
  better. The R16 math `unexpected:_`/`^` "Anonymous String" cluster is
  predominantly this class.
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
* **`{\theoremcmd …}` theorem-as-declaration misuse** — paper uses
  `{\assumption text}` / `{\corollary text}` etc. (where `\assumption`
  was `\newtheorem`-defined, so it's the env-*begin*, not a font
  declaration). The theorem-begin opens a mode-switch frame and the
  group-closing `}` then hits it: `Attempt to close a group that
  switched to mode horizontal due to T_CS[\assumption]`. Byte-identical
  errors in Perl (verified 2026-05-27, same source lines). Witness
  2003.13371 (R14, CONVERR_13). Also explains the recurring
  `\lem`/`\prop`/`\thm`/`\example` mode-switch-close cluster.
* **Unknown bundled `.cls` → OmniBus fallback, body not raw-interpreted**
  — `\documentclass{<custom>}` with the `.cls` bundled but no LaTeXML
  binding: both engines use OmniBus + dependency-scan only; the class's
  own `\newtheorem`/`\def` body is NOT executed, so its theorem
  environments + metadata macros stay undefined. Byte-identical 7-error
  set in Perl (verified 2026-05-27). Witness 2004.03095
  (`artjlt.cls` → `{Theorem}`/`{Lemma}`/`\lastname`/`\msc` undefined).
* **`\filenamebase`-driven multi-file build** — paper does
  `\input{\filenamebase.settings}` etc. where `\filenamebase` is meant
  to be defined externally (build wrapper / command line). Undefined in
  a standalone run → cascade of `\filenamebase.*` missing-file +
  `\setboolean` undefined (the ifthen-loading settings file never
  runs). Byte-identical in Perl (verified 2026-05-27). Witness
  2003.12614 (R14, CONVERR_12).

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

## Engine file open gaps (MINOR)

- ~~`base_parameter_types.rs` — `CommaList:Type` parameterised
  form unported.~~ **CLOSED 2026-05-15** (commit `bb17c1adb0`).
  Reads each item through the inner-type Parameter via
  `Parameters::reparse_argument`, mirroring Perl
  `$typedef->reparseArgument`. Tests 1220/0/0 (no Perl users
  in current corpora; pure parity infrastructure).
- `tex_box.rs` — box dimension edge cases.
- `tex_fonts.rs` — `\fontdimen` array semantics; per-font `\hyphenchar`.
- `tex_tables.rs` — padding CSS classes (XSLT concern).
- `plain_base.rs` / `latex_base.rs` — NON-BLOCKING. Closures kept in
  memory before dump; PA aliases capture `\let` round-trips.
  Architecturally documented in
  `latexml_core/src/state.rs::is_serializable`.
- **~72-CS Perl-only long tail** (from the completed LoadFormat audit,
  `archive/PERL_LOADFORMAT_AUDIT.md`). Engine union has ~72 CSes that Perl
  defines and Rust does not, *excluding* the now-ported `\bib@*` family —
  mostly "misc atomics" (`\@charlb`, point-size CSes, `\batchmode`, …) plus
  the stable 45-CS same-file relocation set. Demand-driven: investigate a
  CS only when a real paper witnesses it; bounded by the corpus-success
  gate, not a release blocker. Refresh the engine-wide CS-name diff (it
  predates the BibTeX port) before quoting exact counts.

## Tikz known diffs vs Perl (reference)

1. `foreignObject` transform Y / width/height.
2. Arrow-tip shape (different path data).
3. SVG viewBox / total width differs slightly.
4. matrix uses `<svg:g class="ltx_tikzmatrix">` (Rust) vs inline-blocks
   (Perl).
5. **`svg:g` directly in `<ltx:block>` core-XML validity error** —
   tikz-cd diagrams emit a bare `svg:g` into an `ltx:block` without the
   wrapping `svg:svg`, tripping `malformed:svg:g isn't allowed in
   <ltx:block>` during core conversion. Post-processing recovers (final
   HTML has well-formed `<svg>`), so the conversion still produces
   output — but the intermediate XML is schema-invalid. Witness
   2006.12702 (`\usepackage{tikz-cd}`, CONVERR — 6 occurrences).
   Rust-only core-construction issue in the tikz→SVG path; lower
   priority since output is recovered.

## Permanent ignores

- **Sandbox out-of-scope**: ns1–ns5 (52_namespace, no DTD); 2402.03300,
  2410.10068, 2511.03798 (Perl also fails).
- **Rust supersedes Perl** (both in scope, Rust passes where Perl
  errors): `1207.6068`, `0909.3444`, plus 40+ in
  `memory/project_rust_supersedes_perl.md`.
- **Unported pools**: none outstanding. (`BibTeX.pool.ltxml` is **ported** —
  Phases 1–8 landed, see [`BIBTEX_PORT_PLAN.md`](BIBTEX_PORT_PLAN.md). The
  remaining B1–B6 / Phase 4–5 polish is tracked there as product
  correctness, not a permanent ignore. `--nobibtex` is an opt-out, not the
  default escape hatch — see [`RELEASE_CRITERIA.md`](RELEASE_CRITERIA.md) §10.)

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

---

## Math parser ↔ Marpa ASF migration — CLOSED 2026-05-19


A multi-session effort to swap the math parser's Tree-iteration
+ per-tree-pruning loop for ASF-driven traversal.

**Working docs**:
* [`docs/MATH_PARSER_AND_ASF.md`](MATH_PARSER_AND_ASF.md) — full
  rationalization: where the existing three stages (grammar
  categories, early semantic pruning in actions, late semantic
  pruning in pragmas) map onto ASF, a worked example, pseudocode
  for the new driver, and a four-gate test plan. **Read first.**
* [`marpa/ASF_STATUS.md`](https://github.com/dginev/marpa/blob/asf-completion/ASF_STATUS.md)
  on the `asf-completion` branch of dginev/marpa — what's
  scaffolding vs functional on the marpa side, with a 7-step
  completion plan and the target Rust API sketch.

**Status snapshot 2026-05-17 (end of session)**:
* Marpa fork `asf-step3-generic-traverser` branch — **Steps 2-6
  LANDED**:
  * `compute_symches` ported (Perl `ASF.pm`-faithful: contiguous
    same-predecessor and-nodes unify into multi-source glades).
  * `Glade` query API: `rule_id`, `symch_count`, `factor_count`,
    `is_factored`, `rh_length`, `rh_glade_id`, `next`, `rewind`,
    `is_token`, `cursor`, `symches()`. (`literal()` deferred —
    needs SLR; math parser is a token-stream consumer, doesn't
    need text spans.)
  * `ASF::traverse` is now a post-order recursive driver with
    per-glade `HashMap<usize, PT>` memoization. Cycle-safe via
    `visited` flag.
  * `Traverser` trait: generic + `&mut TR` (no `Box<dyn>`). Allows
    borrowing traversers like `MathTraverser<'a>` that hold
    `&'a mut Document` + `&'a Actions`. Single-threaded by design.
  * `asf_three_parses_via_exhaustive_traverser` substantive test:
    panda grammar produces exactly 3 distinct Penn-tagged strings
    via post-order memoized traversal — the substantive end-to-end
    validation.
  * 17 marpa tests pass (was 13 before this session).
* latexml-oxide:
  * Cargo.toml marpa dep switched to
    `branch = "asf-step3-generic-traverser"`.
  * Full test suite (1301/0/0) passes against the new marpa branch.
  * `latexml_math_parser/src/asf_traverser.rs` — **scaffolding
    landed**: `MathTraverser` struct implementing
    `marpa::asf::Traverser`. Handles byte glades, lexeme-rule glades
    (matches `TreeBuilder::rollup_token` semantics), standard rule
    glades (Cartesian product + `Actions::action_on`).
    **Not yet wired into `parse_marpa`** — that's the next-session
    task.

**Remaining sequence**:
1. ✅ **LANDED**: `MathTraverser` wired behind `LATEXML_MARPA_ASF=1`.
   Side-by-side runs validated.
2. ✅ **MOSTLY LANDED**: pragma/action prunes for ambiguity classes
   (1272 → 1292 ASF; LEGACY 1301/0 preserved).
3. ⏳ Validate on the 10k canvas stage. Expect 0 test regressions,
   measurable perf gain on ambiguous formulas.
4. ✅ **CLOSED 2026-05-19**: the 9-test list referenced below
   was already obsolete (down to 1 — `physics_test`); the residual
   `physics_test` failure under `LATEXML_MARPA_ASF_ONLY=1` is now
   resolved. Both `cargo test --tests` (HYBRID, default) and
   `LATEXML_MARPA_ASF_ONLY=1 cargo test --tests` report
   **1328/0/0** on this branch.
   Root cause: the grammar had two rules matching `\sin[arg]` in
   `applied_func` — `opfunction tight_term => prefix_apply` AND
   `opfunction lbracket formula rbracket => apply_delimited`
   (`[arg]` is also a `fenced_factor` → `tight_term` via
   `lbracket formula rbracket => fenced`). HYBRID's Tree-iter
   landed on `prefix_apply` and capped via `max_unique`; ASF's
   Cartesian-product enumeration ran BOTH rules. `apply_delimited`
   eagerly XMRefs its `func` operand through `create_xmrefs` →
   `Document::generate_id`, bumping `_ID_counter_` on the math
   ancestor for a tree that's then pruned in favor of
   `prefix_apply`'s output. The wasted xml:id slot shifted
   surviving lexemes' IDs by +1 (`S1.Ex14.m1.15` vs expected
   `S1.Ex14.m1.14`).
   Fix: removed the redundant `opfunction lbracket formula
   rbracket => apply_delimited` rule in
   `latexml_math_parser/src/grammar/builder.rs`. Both modes now
   converge on `prefix_apply` for `OPFUNCTION+[…]`, eliminating
   the spurious action call. The paren variant
   (`opfunction lparen formula rparen => apply_delimited`)
   remains — `\sin(x)` is the canonical function-call notation
   that warrants the XMDual structure. `function lbracket`
   and `trigfunction lbracket` rules left intact for now (their
   rule-id signatures didn't fire on the failing case; revisit
   if a future witness emerges). Test fixture
   `tests/complex/physics.xml` re-blessed (23 xml:id
   renumberings; tighter contiguous numbering — closer to
   Perl's `t/complex/physics.xml` ID pattern, no structural
   changes).
   Historical context: the old 9-test list was
   `ambiguous_relations, count_parses, mathtools,
   metarelation_elision, physics, plainfonts, qm,
   standalone_modifiers, vertbars` — those were the ASF failures
   as of 2026-05-17 / 2026-05-18; subsequent landings (pragma
   refinements documented in `MATH_PARSER_ASF_TIEBREAKING.md`)
   closed all but `physics`, which this fix addresses.
5. ✅ **LANDED 2026-05-19**: `modified_term` grammar category
   (Phase 1 + Phase 2). Concrete witness `P(x = 0, y < 0)` —
   previously `ltx_math_unparsed`, now parses cleanly as
   `P @ vector(x = 0, y < 0)`.
   * **Phase 1 (a16cce3ddc):** narrow grammar additions —
     `modified_term = tight_term relop expression =>
     infix_relation` (single-relop only; multi-relop chains keep
     the existing multirelation path) plus
     `formula_list += modified_term punct modified_term |
     formula_list punct modified_term => modified_list_apply`.
     Early-action prune in `infix_relation` rejects `Apply(relop,
     lhs, list@(…))` when the list contains a relational item,
     forcing Marpa to commit to the modified_term + fenced path.
     `cargo test --tests` and `LATEXML_MARPA_ASF_ONLY=1 cargo
     test --tests` both **1328/0/0**.
   * **Phase 2 (994cbcfa1a):** retired the now-redundant
     `prefer_zero_absent_when_available` pragma (no dedicated
     test witness; conceptual target already covered by qm
     pragmas + angle-bracket grammar). Function body removed
     from `semantics/tree.rs`; placeholder comment in
     `parser.rs::parse_marpa` references the commit.
   * **Discipline notes:** the earlier (deferred) additive
     prototype broke 8 tests because it added a wider
     `modified_term` form at the `statement` level alongside the
     `formula relop expression` chain — additive co-existence
     multiplied ambiguity. Phase 1 stays narrow (all-modified-
     terms list variants only); mixed-content variants
     (`modified_term punct expression`, etc.) deferred until a
     witness justifies them. `parse_tree_count_limits` regression
     test is the canary.
6. ⏳ Delete 5 of the 6 convergence caps in `parser.rs` (only
   `max_time` stays). Delete online `parses.contains(&tree)` dedup.
   **Note (refreshed 2026-05-19):** the code comment at
   `parser.rs::parse_marpa` line ~1576-1589 explicitly keeps the
   caps as the LEGACY-path debug-escape-hatch protection — without
   them the legacy escape would hang on real ambiguous inputs.
   The intent of this item was the ASF/HYBRID hot path, where
   the caps don't fire anyway. Treat as a documentation cleanup
   rather than a code change.
7. ✅ **CLOSED**: marpa dep is on `dginev/marpa` master
   (`Cargo.toml` shows `git = "https://github.com/dginev/marpa"`
   with no branch; commit `0bf241116fcef…` in `Cargo.lock`).
   The asf-step3-generic-traverser branch was merged via marpa
   PRs #3 + #4 (`cdb5fa5f99` "marpa back to master (PR #4 merged,
   large-bocage fallback landed)").

**Session progress (2026-05-17, second push)**: ASF parity
**1272/29 → 1292/9** (20 tests fixed) via:
* `FencedLettersAreFunctionArguments` Dual-aware + tier move (12)
* `prefer_named_interval_at_root` for `(a,b)`, `[a,b]` (2)
* `prefer_non_self_wrapping_root` for `set@(set@(...))` (2)
* `prefer_combined_relop_over_multirelation_with_absent` (subcase fix)
* Early-action prune for `Apply(OPERATOR, [single]) * simple_RHS` (1)
* Compose left-associativity in `infix_apply` (1)
* `bare_conditional` reject in `list_apply` (1)
* `prefer_zero_absent_when_available` + ncases.xml bless (1)

**The win**: eliminates the 5000-tree cap. Per-formula action cost
drops from O(trees × occurrences) to O(glades). Removes the five
convergence bandages (`max_trees`, `max_consecutive_dupes`,
`pruned_only_time_budget`, `converge_budget`, `max_unique`) that
exist purely to dodge the wrong-paradigm cost. `max_time` is the
only cap that needs to stay.

---

## Release-readiness & issue-tracker context (consolidated 2026-05-24)

This file stays the **engine-sync log**. The public-release contract moved
out so it doesn't crowd the parity worklist:

- **[`RELEASE_CRITERIA.md`](RELEASE_CRITERIA.md)** — pre-1.0 gates: size,
  portability, license audit, safety, tail-latency, surpass-Perl policy,
  and the source-provenance / VSCode-synced-preview track (#47/#92).
- **[`ISSUE_AUDIT.md`](ISSUE_AUDIT.md)** — open GitHub issues mirrored
  locally (refresh before milestone planning).

These replace the inline 2026-05-24 codex "public-quality gaps" pass; its
errors are corrected in `RELEASE_CRITERIA.md` §10. The parity mission is
unchanged: ~99.4% on the 100k warning subset, no error-downgrading.
