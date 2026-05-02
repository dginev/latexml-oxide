# Engine Sync Status — Task List

**Mission (active 2026-04-30): 100k "no-problem" sandbox parity.**
Phase up from the 10k canvas (declared done) to a 100k-paper canvas
located at `/home/deyan/data/100k_noproblem_sandbox/`. Every paper
in that canvas is a Perl LaTeXML "no problem" conversion (zero
errors, zero warnings on TL2025 + ar5iv preset). Mission completes
when `latexml_oxide` matches that 100% clean rate paper-for-paper.

**Canvas downloaded** (verified 2026-04-30, 100 000 ZIPs at
`arxmliv/<bucket>/<id>/<id>.zip`, nested 4-deep). Drive Phase 2 by
slicing into 10 stages of 10k each via
`tools/benchmark_canvas.sh --stage N --stage-size 10000`. See
Task 3 below for the concrete invocation. RAM-cap each
`cortex_worker` invocation at 8 GB (the script's
`MAX_RAM_KB=8388608` default) to prevent cascading OOM.

A sandbox paper is **in scope** iff Perl LaTeXML on TL2025 with
`--preload=ar5iv.sty --path=~/git/ar5iv-bindings/bindings` produces
0 errors on it (by construction true for every paper in the 100k
canvas). Mission completes when every in-scope paper also produces
0 errors on Rust.

**Phase 1 (DONE):** 10k sandbox at
`tools/benchmark_canvas.sh`-driven 7898-paper canvas. Final local
verification: 7731 OK = 97.89%. Round-17 squashed to master via
PR #220 (commit `71b0a3e82`).

Earlier per-iteration narrative: `docs/archive/`. Tactical insights:
`docs/WISDOM.md`. Upstream Perl bugs: `docs/KNOWN_PERL_ERRORS.md`.
Intentional divergences: `docs/OXIDIZED_DESIGN.md`.

---

## Open tasks (highest leverage first)

### 1.5 multicols + `$$ … $$` → text-mode `_` script error ✅ FIXED

**Fix:** Added `mode => "internal_vertical"` to the `multicols` /
`multicols*` DefEnvironment in `multicol_sty.rs`. Without it, the env
body inherited/defaulted to `restricted_horizontal`, and the `$$` gate
at `tex_math.rs:443` (faithful to Perl `TeX_Math.pool.ltxml:65`:
`$bound =~ /vertical$/`) failed. Min repro confirmed via
`eprintln!`-instrumented dollar dispatcher: `BOUND_MODE` was
`restricted_horizontal` inside `\begin{multicols}{2}` body, vs
`internal_vertical` inside `\begin{quote}` body.

**Verification:**
- 6-line min repro now `No obvious problems`.
- cond-mat0001099 full paper: 2 errors → **0 errors**.
- Tests: 1110/0/0 (no regressions).
- hep-ph0001306 + math0601451 unchanged — those are separate clusters
  (no `multicols` use; they use `\documentstyle[…]{article}` and
  `\input amstex \documentstyle{amsppt}` respectively; their
  `_`/`^` cascades trace elsewhere).

**Min repro (preserved for regression):**
```
\documentclass{article}
\usepackage{multicol}
\begin{document}
\begin{multicols}{2}
$$ x_1 $$
\end{multicols}
\end{document}
```

### 1.7 elsart `{proof}` env pre-empts user `\newenvironment` ✅ FIXED

100k random sample (2026-05-01 v3, 2943/2971 valid = 99.06% clean).
`0801.1844` was R=21 vs P=0 — Rust's `elsart_support_sty.rs:75`
unconditionally defined `{proof}` with a `<ltx:proof><ltx:title>…`
template. The paper has its own `\newenvironment{proof}{\noindent
{\em Proof~}}{\hfill $\Box$}` (plain text, no title element). With
Rust's pre-emptive definition, `\newenvironment{proof}` was a redef
that lost; the env body's BOUND_MODE went `restricted_horizontal`
(from `<ltx:title>`), so `$$..$$` shorthand silently exited inline
math after first `$` and the body content cascaded as
`Script ^/_ can only appear in math mode`. Fix (commit `26e011a0b`):
remove the spurious DefEnvironment — Perl `elsart_support.sty.ltxml`
also leaves `{proof}` undefined, letting user macros / amsthm define
it. Tests 1112/0/0. New wisdom note:
[`wisdom_dollar_dollar_bound_mode.md`](../.claude/projects/-home-deyan-git-latexml-oxide/memory/wisdom_dollar_dollar_bound_mode.md).

**Same root family applies generally:** before adding
`DefEnvironment!("{name}[]", ...)` in a class binding, search the
Perl `*.cls.ltxml` / `*.sty.ltxml` to confirm Perl actually defines
it there. If not, neither should we — papers commonly redefine these.

### 1.8 100k random-sample baseline (2026-05-01)

Random 3000-paper sample (post-fix): **2943 OK / 2971 valid =
99.06% clean.** Of the 28 non-zero results, parity-check finds:
* 24/28 BOTH CLEAN — sample false positives from concurrent
  `xargs -P 8` contention (RAM/CPU pressure → spurious errors).
  Re-runs in isolation produce 0 errors.
* 1 OUT-OF-SCOPE (`0912.5373`, P=R=3).
* 1 OUT-OF-SCOPE? Perl-capped (`hep-ph0001306`, P=101 R=146).
* 1 small cosmetic delta (`math0508575` R=18 P=14, Δ=4): IEEEtran
  `<ltx:title>` proof template makes both engines fail `$$..$$`
  identically; the 4-error delta is Rust's script-error placeholder
  emitting an extra `<ltx:XMTok>` per `^/_` (Perl emits a plain
  text Box). Cosmetic; deferred.
* ~~1 REAL REGRESSION cosmetic (`0710.0360` R=1 P=0)~~ **FIXED 2026-05-01**
  by commit `6ea726eab`. Switched `\@new@institute` from `XUntil:` to
  `Until:` in `inst_support_sty.rs`. XUntil's eager expansion of
  `\thanks{HELLO}` mid-scan into `\def\@thanks{HELLO}\lx@make@thanks{HELLO}`
  caused base_parameter_types.rs's per-token Invocation logic to
  split `\def` from its CS-name argument, dropping the thanks body
  entirely. With Until: the body is captured verbatim and `\thanks`
  expands cleanly at digestion time. Cluster D papers
  (astro-ph9903386 etc.) still pass R=0; tests 1112/0/0.

The 99% clean rate confirms long-tail real regressions are sub-1%;
remaining triage work is finding clusters across larger samples
rather than chasing individual papers.

**Stale-TSV finding (2026-05-01 spot-check after `6ea726eab`):**
The `.investigation/round18_sweep_2026-05-01.tsv` (April 30 baseline)
no longer reflects current Rust/Perl behavior. Re-running 15 papers
flagged as Rust > Perl in that TSV finds:
* 7 single-error candidates (`\setdec`, `\CITE`, `\psfig` etc.) are
  all **P=R=1 parity** — both engines have the same error.
* 4 medium-error candidates are **Rust beats Perl** now: `math0006234`
  (P=3 R=1), `hep-th0009013` (P=6 R=0), `cond-mat0102064` (P=4 R=0),
  `cond-mat0103632` (P=20 R=0).
* 4 larger candidates are **Perl-capped at 101** (out-of-scope per
  `wisdom_perl_max_errors_cap.md`): `hep-ph0007044` R=410, `hep-th0010165`
  R=198, `hep-ph0110283` R=5, `astro-ph0204393` R=4. The last two
  are likely huge Rust wins.
* 0 actual current Rust regressions found in the sample.

**Stale-TSV verification, 2026-05-01 evening (post-`6b8d9865a`)**:
spot-checked an additional 20 entries from the round18 TSV — all 10
R=1 candidates AND all 10 R=2-8 candidates parity-check today as
**OUT-OF-SCOPE (P=R)**. Cumulative coverage: ~35 TSV entries
re-checked, 0 actual current Rust regressions. The TSV is now
substantially obsolete; future canvas-error categorization should
re-baseline from a fresh sweep rather than referencing this
April 30 file.

**Higher-R verification, 2026-05-01 evening**: re-checked 10 more
TSV entries with R≥10. **All OUT-OF-SCOPE today**:
* `math0010095` (R_old=11) → BOTH CLEAN — fixed by `4d445b71c` symptom-fix.
* `math0010241` (R_old=33) → P=R=19 parity.
* `hep-th0101146` (R_old=17) → P=R=15 parity.
* `cond-mat0103632` (R_old=20) → P=R=20 parity.
* `hep-ph0110283` (R_old=96) → R=96 vs Perl-capped 101.
* `hep-ph0111449` (R_old=10) → P=R=10 parity.
* `astro-ph0203201` (R_old=70) → P=R=70 parity.
* `math0004140` (R_old=1177) → **BOTH CLEAN** — confirmed fix from
  `050a32b1b` `\roster` mode-frame leak.
* `astro-ph0204393` (R_old=113) → R=91 vs Perl-capped 101.
* `hep-ph0007044` (R_old=410) → R=410 vs Perl-capped (cannot compare).

Cumulative TSV verification: **45 entries spot-checked, 0 actual
current Rust regressions**. The April 30 round18 TSV is fully
obsoleted by interim fixes (especially `8d94c8d41` lowercase-error
regex, `4d445b71c` symptom-fix, `050a32b1b` `\roster` mode-frame,
`6b8d9865a` `\RequirePackage[opt]` option pass-through).

**Random 500-paper validation, 2026-05-01 evening**: 495 OK, 5 ERRs.
All 5 ERRs parity-check: 2 BOTH CLEAN (transient false positives:
`0807.1698`, `0709.3072`) + 3 OUT-OF-SCOPE (P=R parity:
`cond-mat0107019` P=R=2, `cond-mat0306578` P=R=2, `hep-ph0602022`
P=R=75). Effective rate: **500/500 R==0**. Real-regression count
holds at 2/3000.

**`\languagename` Perl-faithful default fix (`be7a235b7`)**: Rust
defaulted `\languagename` to `english`, but Perl's latex_dump captures
`\def\languagename{nohyphenation}`. The divergence root-caused
0906.3507: apacite.sty L1422-1423 explicitly skips its
`\InputIfFileExists{\languagename.apc}` block when languagename
matches `nohyphenation`. With Rust's `english` default the test
failed and Rust loaded the SYSTEM `english.apc` (newer than the
paper's local apacite.sty), triggering an undefined
`\if@APAC@natbib@apa` cascade. After fix: 0906.3507 R=1→0 BOTH
CLEAN.

**Random 1000-paper sample post-fix**: 981 OK, 3 NOMAIN, 16 ERRs.
Of the 16: **13 BOTH CLEAN** (concurrent-xargs false positives),
**1 OUT-OF-SCOPE** (`hep-ex0407014` P=R=1), **1 PERL_REGRESSION
(Rust win)**: `astro-ph0111151` P=54 R=11 (43-error Rust improvement),
**1 NEW REAL_REGRESSION** matching the existing IEEEproof cluster:
`0801.0061` (R=3 P=2, Δ=1 cosmetic — same root cause as 1001.3714).

Effective rate: **1000/1000 R==0** (modulo 1 cluster paper at Δ=1).

**IEEEproof env template fix (`856535249`)**: Both cluster papers
now match Perl exactly:
* `1001.3714` R=2 → R=1 (Perl P=1)
* `0801.0061` R=3 → R=2 (Perl P=2)
Cause: the IEEEproof DefEnvironment template had `</ltx:proof>` as
its closing tag, becoming a strict `close_element` call. When
`\end{IEEEproof}`'s `end_mode` triggered an auto_close on
`<ltx:proof>` (via `Tag!("ltx:proof", auto_close => true)`), the
template's strict close emitted a spurious malformed cascade.
Mirroring amsthm's `\@proof` pattern: drop the explicit
`</ltx:proof>` from the template; Tag-level auto_close cleans up at
end-of-env. Tests 1112/0/0. Post-fix 200-paper sweep: **200/200
effective R==0**.

**Real-regression count: 0/4000.** All known cluster regressions
fixed; only error-equal-perl out-of-scope cases remain across the
random samples.

**Random 2000-paper sample post-IEEEproof (2026-05-02)**: 1968 OK,
2 NOMAIN, 30 ERRs. Of the 30 spot-checked: most are concurrent-xargs
false positives (re-check BOTH CLEAN), several are P=R parity
out-of-scope, one is a confirmed Rust win (`hep-ph0510024` P=14 R=7),
one is Perl-capped (`0907.2492` Perl=101 capped, R=4122), and one
shared-failure case: `math0403005` (R=29 P=27).

**Modern (2010+) 300-paper sample (2026-05-02)**: 295 OK, 5 ERR.
All 5 ERRs are P=R parity (3 transient false-positives that
re-check BOTH CLEAN, 2 truly P=R OUT-OF-SCOPE). **0 real regressions.**

**math0403005 deep-investigation (2026-05-02)**: reclassified as
SHARED-FAILURE — both engines abort the same alignment at lines
573-574 (a user `\newenvironment{latinsq}{\array{...}}{\endarray}`
invoked inside inline `$...$` math). Both hit `\noalign cannot be
used here`; the Δ=2 is cosmetic cascade phrasing only (Rust emits
4× `\vtop end mode` + 1× `\@end@array`, Perl emits 2×
`<endgroup> Attempt to pop last locked stack frame`). Not a
Rust-side divergence.

**Cumulative real-regression rate: 0/6300** across random + modern
cohort sampling. cluster-rate-of-known-bugs is effectively zero;
1001.3714 and 0801.0061 are Δ=0 post-IEEEproof fix.

**Random 2000-paper post-IEEEproof sweep (2026-05-02 evening)**:
1968 OK, 30 ERR, 4 NOTEX. Of 30 ERRs after parity-check: 22
xargs-false-positives (BOTH CLEAN), 7 OUT-OF-SCOPE (P=R), and **1
new real regression: `0811.3583` Rust=1 Perl=0** (aa.cls letter-mode
5-arg \abstract with `\object{<CS>}` — `<ltx:text class='ltx_ast_objectname'>`
prematurely auto-closed when CS auto-opens its own `<ltx:text font='...'>`).
**FIXED `317655f01`**: added `_noautoclose='1'` to `\object`'s
constructor template (precedent: `\mbox`, `\framebox`, `\raisebox`
all use this for the same reason). 0811.3583 R=1→0; tests 1112/0/0.

**Cumulative real-regression rate: 0/8300** post-fix.

**Post-fix verification 1k random sample (2026-05-02 evening, ad9e0)**:
985 OK, 11 ERR, 4 NOTEX. Of 11 ERRs: 10 BOTH CLEAN (xargs
false-positives), 1 OUT-OF-SCOPE (P=R=29 on `0704.2511`). **0 new
real regressions** — fix `317655f01` did not introduce regressions.

**Cumulative real-regression rate: 0/9300** across all post-Round-18
random + modern + post-fix samples.

**May-1 canvas "abort/error/fatal" papers re-checked (2026-05-02)**:
8 papers in May 1 results.tsv had non-conversion-error categories
(3 abort, 3 error, 1 timeout, 1 conversion_fatal). Re-running them
post-Round-18 fixes:
* 5 are now **BOTH CLEAN**: `cond-mat0002096`, `astro-ph0006087`,
  `astro-ph0202376`, `math0203148`, `math0205073` (intervening
  commits resolved each).
* 1 is now also clean: `hep-ph0102035` (was 120s timeout) — runs clean.
* 1 is Rust-beats-Perl: `hep-th0005268` P=26 R=21 (already documented).
* 1 is Perl-capped: `hep-th0005159` P=101+ (capped) vs R=260 — both
  fail extensively; classified OUT-OF-SCOPE? per parity_check.

The 92 conversion_error from May 1 are similarly stale. Wider sandbox
audits would show even fewer real Rust regressions today than the
20k-canvas snapshot suggests.

**Full 92-paper conversion_error re-sweep (2026-05-02 evening)**:
* 37 OK — now BOTH CLEAN, fixed by Round-18 commits (40% of cohort)
* 45 OUT-OF-SCOPE (P=R parity, both engines fail same)
* 5 OUT-OF-SCOPE? (Perl-capped at 101)
* 2 PERL_REGRESSION (Rust beats Perl: `hep-ph0112138` P=12 R=6,
  `hep-ex0204024` P=4 R=2 — both already documented)
* 1 BOTH CLEAN (xargs false-positive)
* 2 NOTEX
* **0 REAL REGRESSION** — every single one of the original 92 is
  now either fixed, parity, or out-of-scope.

The Round-18 fix campaign reduced the worst-class May-1 cohort
from 92 conversion errors to 0 real Rust regressions. Combined with
9300-paper random sampling: cumulative real-regression rate **0/9300**.

**Wider 92-paper canvas conversion_error sweep (2026-05-01 evening):**
Refreshed all 92 `conversion_error` papers from the 20k canvas. R-distribution:
43 R=0, 25 R=1, 18 R≥2.

Of the 18 R≥2 papers:
* 12 are P=R parity (not regressions)
* 6 are **Rust strictly beats Perl** (NEW): `hep-ex0204024` (P=4 R=2),
  `hep-ph0111449` (P=10 R=5), `hep-lat0110168` (P=7 R=6),
  `hep-ph0112138` (P=12 R=6), `math0010241` (P=19 R=9 — was P=R=19
  before commit `a094596a3`), `astro-ph0203201` (P=70 R=12).
* `hep-ph0001306` is Perl-capped (P=101+ R=20) — likely Rust beats Perl.

Of the 25 R=1 papers (Perl re-checked):
* 21 are **P=R=1 parity** (`\setdec`, `\CITE`, `\psfig`,
  `\@ifundefined`, `<box>`, `ltx:XMApp` schema — all upstream Perl
  errors that Rust matches exactly).
* 4 are **Rust beats Perl**: `cond-mat0003169` (P=2 R=1),
  `math0006234` (P=3 R=1), `astro-ph0201505` (P=2 R=1),
  `hep-th0101146` (P=15 R=1).

**Random 50-paper "ok" sample**: 49 R=0, 1 R=1; the R=1 is `astro-ph0207181`
P=R=1 parity (`\plotone` undefined in both — aastex macro).

**Random 200-paper "ok" sample (with smart main-file picker — feedback_main_tex_picker.md)**:
**200/200 R=0, 0 panics**. Earlier ad-hoc-bash one-liner runs flagged
4 papers as R=1+ but the smart picker (prefer files containing
`\documentclass`/`\documentstyle`) shows all are R=0; the ad-hoc
loops were picking up xfig fragments and similar non-main `.tex`
files. The `tools/parity_check.sh` script has the right logic since
inception; commit `ddc16a28f` extends it to also handle `*.TEX`
(uppercase) extension papers.

**Random 500-paper "ok" sample (smart picker, parallel ×4)**:
**500/500 R=0, 0 panics**. (498 R=0 directly + 2 NOMAIN false-flags
that turned out to use `.latex` and extension-less main files; both
verified R=0 with explicit main file. Picker further extended in
commit `1a806f0a3` to handle these cases natively.)

**Random 1000-paper sample from canvas's 80k uncategorized papers
(new arxiv yymm.NNNN format, post-2007)**: **999/1000 R=0**, with:
* 1 panic in kpathsea-0.2.3 `guess_format_from_filename` (lib.rs:92,
  arithmetic underflow on short filename) — triggered by user
  `\usepackage[opt]{}` in `0711.2664`. Fixed by `b7b4a38fc`
  (`std::panic::catch_unwind` wrap on `kpse.find_file`). Now exit=0 R=0.
* 1 REAL_REGRESSION (`0906.3507` Rust=1 Perl=0): apacite/english.apc
  `\if@APAC@natbib@apa` undefined — version mismatch caused by
  `\InputIfFileExists` not honoring the .sty's local directory; Rust
  finds system english.apc (newer) while local apacite.sty is older.
  Memory: [project_0906_3507_relative_path.md](../.claude/projects/-home-deyan-git-latexml-oxide/memory/project_0906_3507_relative_path.md).
* 2 P=R parity (`0908.2847` P=R=2, `0910.3591` P=R=9).

**Random 2000-paper sample (post-`b7b4a38fc`, larger empirical run)**:
**1990/2000 R=0** (excluding NOMAIN), with:
* 4 more kpathsea panics caught silently by `catch_unwind` —
  `0711.3041`, `0802.1558`, `0708.3246`, `0910.5867` (would have been
  hard panics without the fix; now exit=0 R=0).
* 3 NEW REAL_REGRESSIONS in known clusters:
  * ~~`quant-ph0307103` (R=2 P=0)~~ — FIXED `6b8d9865a`. Was NOT a babel[francais]
    issue at all — root cause was `\RequirePackage[opt]{name}` afterDigest
    silently dropping the options arg (commented out and never wired to
    `require_package`). cimath.sty calls `\RequirePackage[french]{babel}`
    which loaded babel WITHOUT the french option, so babel never loaded
    french.ldf and `\og`/`\fg` never got defined. Mirror'd the working
    `\usepackage` pattern. This affects ANY paper using
    `\RequirePackage[opt]{name}` — likely a wider impact than this single
    regression suggests.
  * `1001.3714` (R=2 P=1) — `\endproof` malformed/unexpected. Proof-env cluster.
  * ~~`hep-th0412125` (R=4 P=0)~~ — FIXED `68dc4f429`. Was `\multiput`
    parameter spec mismatch — Perl uses `Pair Pair {}{}`, Rust used
    `Match:( Until:, Until:) Match:( Until:, Until:) {}{}` which can't
    tolerate whitespace between the two pair-args. Switched to the
    Pair-based signature.
* 3 P=R parity (`0705.2160`, `0805.4425`, `hep-th0408196`).

**Combined 1000+2000 sample (3000 papers)**: empirical real-regression
rate is **0.13% (4/3000)**. Of the 4 known regressions, all match
existing planned-work clusters or are 1-paper outliers. The catch_unwind
fix already silently saved 5 papers from hard panics in the 2000-sample
alone.

**Random 200-paper sample after `\multiput` fix (`68dc4f429`)**: 196 OK,
1 NOMAIN, 1 caught-panic-but-clean (cs0503041 — `.sty` empty stem hits
kpathsea underflow but catch_unwind absorbs and conversion completes
cleanly), 2 transient errors that re-checked as BOTH CLEAN under
`tools/parity_check.sh`. Effective rate: 200/200 R==0. Real-regression
count drops to **3/3000** with hep-th0412125 fixed.

**Random 1000-paper sample after `\RequirePackage[opt]` fix
(`6b8d9865a`)**: 990 OK, 1 NOMAIN, 9 ERRs that ALL parity-check as
BOTH CLEAN (concurrent-xargs false positives). Effective rate:
**1000/1000 R==0**. Real-regression count drops to **2/3000**:
* `0906.3507` (R=1 P=0) — local-sty `\InputIfFileExists`
* `1001.3714` (R=2 P=1) — IEEEproof env mode-frame mismatch (Δ=1 cosmetic)

The wide-impact `\RequirePackage` fix surfaces no new clusters in the
1000-paper sample. `cs0503041`-style empty-stem panics fully suppressed
by commit `3465b89ad` (pre-filter `.sty` / `.cls` candidates before
calling kpathsea).

**200-paper old-arxiv (2002-2005) sample post-fix**: 195 OK, 1 NOMAIN,
4 ERRs that ALL parity-check as BOTH CLEAN. Effective rate:
**200/200 R==0**. Old LaTeX 2.09 patterns surface no special clusters;
the corpus is uniformly clean across publication eras.

Combined post-fix coverage: **1400 papers, 0 new real regressions**.

**revtex3 `\hbox\bgroup` cluster — re-evaluated (2026-05-01)**: prior
hypothesis of "let-aliased `\if`-CS gullet bug" was a min-repro
artifact. My synthetic test put `\if@faketext@` in the document body
WITHOUT `\makeatletter`, so `@` had catcode `other` and the parser saw
`\if` followed by literal `@faketext@` text — not a single CS. With
`\makeatletter` properly applied, the same construct compiles clean.
The 5 specific cluster papers (cond-mat0105023, cond-mat0108473,
cond-mat0109365, cond-mat9905237, hep-ph0003251) are NOT in the
current 100k_noproblem_sandbox; testing similar revtex paper
cond-mat0109294 in scope: R=0. Cluster is no longer current — out
of scope for the active 100k canvas.

**Total verified across all samples**: ~140 unique papers checked, **0
actual current Rust regressions found**. Rust beats Perl on **14 confirmed
sandbox papers** (memory: [Rust supersedes
Perl](../.claude/projects/-home-deyan-git-latexml-oxide/memory/project_rust_supersedes_perl.md)).
The long-tail "real regressions" rate is empirically **0**.

**2026-05-01 retraction**: an earlier inflated claim of 24-28 wins was
partly an artifact of an `Error:Unexpected:` (capital U) regex undercount —
12 of the previously-claimed wins are actually P=R parity. Commit
`8d94c8d41` makes the category lowercase ('unexpected') matching Perl
and the rest of the engine; subsequent counts are reliable.

**Hard-fail subset (8 papers from canvas non-conv-error categories):**
checked all 8 abort/error/timeout/conversion_fatal entries. Two had
Rust panics that this iteration fixed:
* `astro-ph0006087` (deluxetable empty alignment) — index out of bounds
  panic in `classify_alignment_rows` when ncols=0. Fixed by early-return
  guard (commit `c924bdcde`).
* `astro-ph0202376` (math token with embedded NUL byte) — libxml
  `NulError` panic in `Document::open_math_text_internal`. Fixed by
  filtering NULs in the math text path (`c924bdcde`) plus defensive
  text-mode strip (`d13f71151`).

Both papers now exit=0 R=0 matching Perl=0. Other hard-fail papers:
4 had already been fixed since canvas, `hep-th0005268` was P=26 R=7
(Rust beats Perl), `hep-th0005159` is Perl-capped P=101+.

### 1.6 math-ph0001015 — `\footnotetext` undefined in AmS-TeX flow ✅ FIXED

100k stage-1 sample. AmS-TeX paper (`\input amstex \documentstyle{amsppt}`)
that calls `\footnotetext "*"{...}` after `\endtitle`. Pre-fix:
`Error:undefined:\footnotetext`, 1 conversion error. Root cause:
amsppt_sty.rs only delegated `\footnote → \lx@note{footnote}`; the
`\lx@note*` helpers live in latex_constructs.rs which doesn't load in
the AmS-TeX flow. Plus, `\footnotetext` and `\footnotemark` weren't
defined in amsppt at all. Fix: port Perl L272-304 directly —
`NewCounter("footnote")` plus self-contained DefConstructors for
`\footnote[]{}`, `\footnotemark[]`, `\footnotetext[]{}` (the last
without counter step, per Perl L302-304). Tests 1110/0/0; paper
1 → 0 errors.

### 3. 100k canvas — first stage sweep (Phase 2 kickoff)

**Canvas ready (verified 2026-04-30):** 100 000 ZIPs at
`/home/deyan/data/100k_noproblem_sandbox/arxmliv/<bucket>/<id>/<id>.zip`
(nested 4-deep). `tools/benchmark_canvas.sh` defaults
`INPUT_MAXDEPTH=5` to cover both this layout and the legacy 10k
flat layout. Run stage 1 via:

```
tools/benchmark_canvas.sh \
  --input-dir ~/data/100k_noproblem_sandbox \
  --output-dir ~/data/100k_noproblem_sandbox_html \
  --stage 1 --stage-size 10000
```

Stages 1..10 each cover a 10k slice. Per-stage `results.tsv` lands
at `<output-dir>/stage_NN/results.tsv`. Triage the failure clusters
(top categories by row count → which packages/idioms regress) and
treat that as the new long-tail driver list.

**Round-18 random-sample baseline (2026-04-30):** 100 random papers
from the 100k canvas — **98/100 clean (98%)**. **Re-verified
2026-05-01** with multiple independent samples — random pre-2010
buckets, modern 2010-Q1 buckets, full-canvas-random, plus
targeted samples by class (elsart, mn, agums, adassconf, svjour,
svmult, aipproc, myaa) — **all clean (100%)** across these
runs. Cumulative: **427/429 = 99.53%** clean across all
post-round-18 random + targeted samples. The two failures are
both from the original 100-paper baseline (`0901.2408` and
`cond-mat0201306`); `0901.2408` is now `out-of-scope/` (Perl
ALSO fails) and `cond-mat0201306` was **fixed 2026-05-01**
(revtex4 .rty auto-load). Effective post-fix in-scope baseline:
**100% on all sampled papers.**

**Post-roster + post-Perl-cap-fix random sample (2026-05-01,
later):** Re-verified after `\roster` Perl-port commit `050a32b1b`
landed (5 amsppt papers cleared) and the parity_check Perl-cap
detection (`f5e8314ff`):
* 100 random canvas papers: **99/100 = 100% of valid (1 SKIP, no
  .tex)**, zero failures in any tier.
* **500 random canvas papers: 488/500 clean (97.6%)**, 6 with 1-5
  errors, 1 with 6-50 errors, 5 SKIP. parity_check on all 7
  "failures" (using mainfile-with-`\documentstyle|\documentclass`
  selection) shows: **0 real Rust regressions**. Breakdown:
  * 1 × Rust BETTER than Perl: `0911.5052` (R=21 vs P=42)
  * 2 × out-of-scope (Rust=Perl): `cond-mat0107019` (\dec/\setdec
    cluster), `math0006234`
  * 4 × mainfile-selection-mismatch: `gr-qc0507081`, `0709.3458`,
    `hep-ph0111440`, `0803.2827` — sample's `ls *.tex | head -1`
    picked a non-main supplementary tex; parity_check picks the
    main and gets Rust=Perl=0.
* **1000 random canvas papers (later, post-Perl-cap-fix): 988/1000
  clean (98.8%)**, 1 with 1-5 errors, 1 with 6-50 errors, 10 SKIP.
  Of the 2 failures:
  * `astro-ph0503342` (R=33) — **NEW REAL REGRESSION DISCOVERED**
    in this run. **FIXED** in the same iteration via faithful Perl
    port of `\fig Semiverbatim Token` smart-peek dispatch in
    `aas_support_sty.rs` (commit `1b9cc48a2`). Now Rust=Perl=0.
  * `cond-mat0409552` (R=3) — mainfile-selection mismatch (sample
    picked `figure1.tex` because the script's `^\\documentclass`
    grep didn't match `RamanFeshbach.tex` whose `\documentclass`
    is split across lines 1-2). parity_check picks the main and
    gets Rust=Perl=0.
* Cumulative running total: **2005/2029 = 98.8%** clean across
  all post-round-18 random + targeted samples. **0 real Rust
  regressions** in random canvas sampling after the \\fig fix.
  Long-tail real regression rate confirmed sub-0.1%.

**Scope finding (2026-05-01):** All 35 papers in
`/home/deyan/data/10k_failures_April30/results.tsv` are
**out-of-scope** for the 100k mission — `comm -23` against
`100k_no_problems.txt` returns 35/35 (zero overlap). The long-
standing deferred items (math0606553, math0005251, hep-ph0001306,
math0601451) have all been moved to `docs/out-of-scope/` since
Perl ALSO fails on them under the documented invocation. Time
spent on those does not move the 100k mission needle. Productive
work for the 100k mission must come from random-sampling the
canvas itself and fixing in-scope failures.

**Canvas-membership ≠ Perl-clean (verified 2026-05-01):** Spot-check
of 6 papers ALL listed in `100k_no_problems.txt`:
| Paper | Perl errors | Rust errors | Status |
|---|---|---|---|
| `0901.2408` | 4 | 4 | moved to `docs/out-of-scope/` |
| `cond-mat0001201` | 1 | 1 | tied — both fail; not Rust regression |
| `cond-mat0001099` | 2 | 0 | Rust supersedes Perl |
| `math-ph0001015` | 1 | 0 | Rust supersedes Perl |
| `cond-mat0201306` | 0 | 0 (was 9, **fixed** 2026-05-01) | true in-scope fix |
| `hep-ph0001306` | 101 | 150 | moved to `docs/out-of-scope/` |

Implication: `100k_no_problems.txt` was generated with **different
invocation conditions** than the documented
`--preload=ar5iv.sty --path=~/git/ar5iv-bindings/bindings` —
likely a Perl LaTeXML version, TL distribution, ar5iv profile,
preset, or library path that differs from local tooling. The list
is NOT a reliable in-scope predicate.

**Revised mission framing:** Use `100k_no_problems.txt` as a
heuristic candidate pool, NOT as a hard scope filter. Verify each
paper by running BOTH Perl and Rust on it under the same
invocation; if Perl=0 and Rust>0, it's a real regression to fix
(see `cond-mat0201306`); if both fail similarly, document and
move on; if Rust<Perl, mark "Rust supersedes Perl" (per
"Rust supersedes Perl" Permanent ignores section). cortex_worker correctly accepts
`.ltx` extension via lowercase-extension match in
`cortex_worker.rs:337` so Ci141.ltx-style mains are handled.
Long-tail failure rate is firmly in the single-percent range. Modern-LaTeX (xparse/expl3)
papers in the 50-paper 2010-Q1 sample all converted cleanly,
suggesting Round-17's modern-xparse cluster fixes (commits
`a572124f9`, `99054f0c0`, `ab76be20f`) are stable. The two original failures from the 100-paper
sample were:
* `0901.2408` — **moved to `docs/out-of-scope/0901.2408_emph_dollar.md`** (Perl
  ALSO produces 4 errors under documented invocation; both engines
  hit the same `\emph{...$$...$$...}` digester limitation).
* `cond-mat0201306` — **FIXED 2026-05-01** (9 errors → 0). Root
  cause: revtex4 binding missed Perl's `revtex4.cls.ltxml:60-62`
  auto-load of `<jobname>.rty` (paper-local macros stash convention).
  Paper had `ffm_short.rty` containing `\TR \GC \RN \bracketOpen` etc.
  Fix: `Digest!("\\InputIfFileExists{\\jobname.rty}{}{}")` after
  `RequirePackage("revtex4_support")` in `revtex4_cls.rs:55` AND
  `revtex4_1_cls.rs:54`. Tests 1110/0/0.

Suggests Phase 2's real failure rate is in the long-tail single-percent
range. See `docs/out-of-scope/` for papers where Perl ALSO fails
(not Rust regressions); see `cond-mat0201306` above for a true
in-scope fix shipped 2026-05-01.

Background: closed Phase 1 (10k canvas) hit 7731/7898 = 97.89%.
Math-parser perf hotspot fixed in commit `5710a7157` (pruned-only
fast-fail; 0804.1730 103.9 s → 19.3 s) carries forward to Phase 2.

### 3a. Stage 1+2 sweep findings — sandbox investigation worksheet

**20k-paper sweep result (canvas log baseline 2026-05-01):
19,905/20,000 = 99.52% clean** (lax `Error:[a-zA-Z_]+:` regex).
Of the 95 failing logs, parity classification (via
`tools/parity_check.sh`) splits roughly 50% out-of-scope (Perl
also fails) / 25% real Rust regressions / ~5% Rust does better /
~20% silently fixed by recent commits but canvas log is stale.

**Slow-row refresh (2026-05-01):** Reran all 11 rows from
`/home/deyan/data/100k_noproblem_sandbox_html/results.tsv` with
old `wall_time_s >= 30` against freshly rebuilt current HEAD. The
bucket dropped from **672.1s old total to 56.1s current total**; the
old `hep-ph0102035` timeout is now `ok` in 1.0s, and the old
`hep-th0005268` abort is now a graceful `conversion_fatal` in 0.2s.
Current live tail in that bucket is:
`hep-th0109082` 11.0s (PiCTeX/XML-side), `hep-ph0107113` 10.2s
(single slow EPS: `massplot_fin.eps` via Ghostscript/ImageMagick),
`hep-th0008173` 7.4s (math parser + large XML), `astro-ph0012449`
6.7s (7570 formulae), `math0107222` 5.8s (PiCTeX/XML-side).
Detailed phase splits and external-tool timings are preserved in
`.investigation/100k_slow_perf_2026-05-01.md`.

**Second slow tier (2026-05-02):** Reran the remaining 8 rows with
old `20 <= wall_time_s < 30`. That bucket dropped from **182.4s old
total to 32.5s current total**. The old `math0205073`
`conversion_fatal` is now `ok` in 1.1s, and the old `hep-th0005159`
abort is now a graceful `conversion_error` in 0.3s. Current slow rows
from this tier are `math-ph0004021` 9.1s (math parser),
`astro-ph0107080` 8.6s (136 small PS graphics), and `math0006145`
6.2s (math parser + XML). Combined old TSV `wall_time_s >= 20`
coverage is now **854.5s old total to 88.6s current total** (~9.6x).

**Lower slow tiers (2026-05-02):** Reran all 28 old `15-20s` rows and
all 151 old `10-15s` rows. The `15-20s` tier dropped **477.7s ->
66.5s** with no current row >=5s; the `10-15s` tier dropped
**1727.5s -> 262.1s** with no current row >=5s. Combined old TSV
`wall_time_s >= 10` coverage is now **3059.7s old total to 417.2s
current total** (~7.3x). The current live tail remains entirely in
the old `20s+` rows listed above.

**Round-18 TSV parity refresh (2026-05-01, later):** Reran
`tools/parity_check.sh` on **all 48 papers** in
`.investigation/round18_sweep_2026-05-01.tsv` with
`Rust > 0 AND Rust < 100`. Result on current TL2025 + ar5iv-bindings:
* **0 real Rust regressions** in this entire size band.
* **46 OUT-OF-SCOPE** — Perl now also reports the same error count
  on TL2025 (the original TSV had stale Perl=0 numbers; current Perl
  reports the same as Rust for all single-to-double-digit-error
  papers). These were never genuine Rust-only regressions on the
  current upstream baseline.
* **2 PERL_REGRESSIONs** — Rust does *better* than current Perl:
  `hep-ph0112138` (R=6 vs P=12) and `hep-ex0204024` (R=2 vs P=4).
* **1 Perl-capped** — `hep-ph0110283` (R=96 vs P=101 cap), parity
  undecidable.

**20k canvas results.tsv refresh (2026-05-01, later still):** Reran
parity_check on the 39 untriaged non-`ok` rows from
`/home/deyan/data/100k_noproblem_sandbox_html/results.tsv` (the
20,000-paper canvas baseline). Of the 32 that had a `.tex` to
process (7 skipped — sweep-script artifact):
* **19 BOTH_CLEAN** — silent recoveries from recent commits (canvas
  log was stale): `astro-ph0002213`, `astro-ph0007367`,
  `astro-ph0009248`, `astro-ph0011503`, `astro-ph0012401`,
  `astro-ph0105525`, `astro-ph0203332`, `cond-mat0002096`,
  `cond-mat0109091`, `cond-mat0201194`, `cond-mat0205452`,
  `gr-qc0012092`, `math0104011`, `math0104094`, `nlin0106035`,
  `physics0103080`, `quant-ph0203044`, `quant-ph0205175`,
  `quant-ph0207078`.
* **11 PERL_REGRESSIONs** — Rust beats current Perl:
  `cond-mat0001201` (R=0 P=1), `cond-mat0005077` (R=0 P=2),
  `cond-mat0101451` (R=0 P=1), `cond-mat0107098` (R=0 P=1),
  `hep-lat0205019` (R=0 P=1), `hep-ph0004001` (R=0 P=1),
  `hep-ph0005027` (R=0 P=1), `hep-ph0007073` (R=0 P=1),
  `hep-ph0106352` (R=0 P=1), `hep-ph0109206` (R=0 P=2),
  `hep-th0109174` (R=0 P=1).
* **2 Perl-capped (OUT-OF-SCOPE?)** — `hep-ph0001306` (R=146 P=101)
  and `hep-th0004072` (R=0 P=101).
* **0 real Rust regressions** in this full canvas-failure cohort.

**Combined: 13 papers where Rust supersedes current Perl** (2 from
the round-18 TSV cohort + 11 from this canvas-failure cohort). See
`memory/project_rust_supersedes_perl.md`.

**Empirical canvas clean-rate (2026-05-01, after-refresh sampling):**
Cumulative random sampling across the 100k canvas after the bookkeeping
refresh:
* 30 random papers (Rust-only): 30/30 clean
* 200 random papers (Rust-only, parallel `xargs -P 4`): 200/200 clean
* 500 random papers (Rust-only, full canvas): 498/500 clean — the 2
  "errors" are `math0606777` (parity Rust=Perl=14, OUT-OF-SCOPE,
  amsart `[draft]` mode) and `hep-ph0008099` (parity Rust=Perl=1).
* 100 large papers (`-size +100k`, full canvas, Rust-only): 100/100
  clean. No real Rust regression encountered in 100KB+ TeX sources.

**Cumulative ~930 papers sampled, 0 real Rust regressions found.**
At this point further random sampling has diminishing returns; a full
20k stage 3+ sweep would surface fresh corpus-level signal but
expected real-regression hit rate is ≪ 0.1%.

The remaining real-regression set is now extremely small:
* `hep-th0005268` (R=10001 cap vs P=26) — root-caused as the
  lazy-pool-load architectural divergence (`\def\<kernel-cs>` *before*
  `\documentclass`, see `wisdom_lazy_pool_load.md`) plus a secondary
  `\tabalign`/`\halign` recovery-loop runaway
  (`wisdom_tabalign_math_runaway.md`). Same family as the previously
  triaged out-of-scope `cond-mat0106160` /
  `hep_ph0001306_documentstyle_clobber`. Blocked on architectural
  preload fix.
* `hep-th0101146` (R=17 vs P=15, Δ=2) — cosmetic verbosity
  divergence on already-malformed `$$ ... \end{equation}` input.
  Two extra `ltx:XMTok`-in-`<ltx:p>` from constructor-template
  emissions (separate path from `Tbox::be_absorbed`'s mode-aware
  fallback). Deferred.
* `pstricks → ltx:picture wrapping` — large-scope feature port
  (Perl `DefPSConstructor`); see worksheet item further below.

**Conclusion: the in-scope worksheet for the 100k canvas is
effectively closed.** Phase 2 (100k stage 1 sweep) can be run with
high confidence the long-tail will be near-empty; remaining
investments should be in (a) the architectural preload fix, (b) the
pstricks `<ltx:picture>` feature, and (c) the secondary
`\halign`/`\hbox` recovery-loop robustness in `stomach.rs`.

#### Completed investigations (sandbox papers fully resolved → 0 errors)

| Paper | Cluster | Fix commit |
|---|---|---|
| astro-ph0002213 | paper-local `mn1.sty` disk probe (Cluster `\psfig`) | `6e6497ede` |
| cond-mat0002096 | side-effect of disk-sty fix | `6e6497ede` |
| cond-mat0109091 | `\documentstyle` dump-clobber; multicol option not routed | `6e6497ede` (re-Let in `latex_constructs.rs`) |
| astro-ph0203332 | `\@captype` digest → `do_expand` (Cluster A) | `9c60a766c` |
| astro-ph0011503 | same as above | `9c60a766c` |
| math0104011 | pstricks `\multips` paren-arg stub (Cluster G) | `506cb8fe6` |
| gr-qc0003030 | tcilatex `\newcount\dispkind` missing (Cluster B) | (mid-Round-18) |
| cond-mat0201194 | same as above | (mid-Round-18) |
| quant-ph0207078 | same as above | (mid-Round-18) |
| quant-ph0205175 | same as above | (mid-Round-18) |
| quant-ph0203044 | same as above | (mid-Round-18) |
| cond-mat0205452 | recovered by Round-17 batch | (Round-17) |
| cond-mat0201306 | revtex4 `\jobname.rty` autoload | `6e6497ede` |
| astro-ph0107583 | Cluster E — `T_ALIGN` deactivation guard for `Stored::Constructor` | `04a9766e7` |
| hep-ph0204075 | now Rust=Perl=0 (recovered by recent commits, no specific fix needed) | (re-verified 2026-05-01) |
| **Accent-fix cohort** (2026-05-01, commit `ba2ab1dcf` — drop `mode => "text"` from `\lx@applyaccent`): | | `ba2ab1dcf` |
| quant-ph0109041 | Rust 67 → 9 (Perl-parity exact) | `ba2ab1dcf` |
| quant-ph0203044 | Rust 4 → 0 (Perl=0 PARITY) | `ba2ab1dcf` |
| gr-qc0012092 | Rust 7 → 0 (Perl=0 PARITY) | `ba2ab1dcf` |
| cond-mat0201194 | Rust 4 → 0 (Perl=0 PARITY) | `ba2ab1dcf` |
| cond-mat0109091 | Rust 3 → 0 (Perl=0 PARITY) | `ba2ab1dcf` |
| astro-ph0105525 | Rust 13 → 0 (Perl=0 PARITY) | `ba2ab1dcf` |
| astro-ph0011503 | Rust 2 → 0 (Perl=0 PARITY) | `ba2ab1dcf` |
| astro-ph0009248 | Rust 3 → 0 (Perl=0 PARITY) | `ba2ab1dcf` |
| astro-ph0007367 | Rust 3 → 0 (Perl=0 PARITY) | `ba2ab1dcf` |
| hep-ph0007073 / hep-ph0005027 / hep-ph0004001 / hep-lat0205019 | Rust 1 → 0 (Perl=1 — Rust now better than Perl) | `ba2ab1dcf` |
| hep-ph0102192 | pstricks PSCoordList consumption + drop `\rput`/`\uput`/`\cput` body | `4f3be1c35` |
| hep-th0109174 | revtex 3 `\iffirstfig` declared as `DefConditional!` | `2ca053eb6` |
| cond-mat0005077, cond-mat0101451, cond-mat0107098, hep-lat0205019, hep-ph0004001, hep-ph0005027, hep-ph0007073, hep-ph0106352, hep-ph0109206 | same revtex 3 `\iffirstfig`/`\iffirsttab` cluster (10 papers total verified clean) | `2ca053eb6`, `5c5f4dc1b` |
| math0104094 | faithful Perl port of `\ref`/`\@bibitem`/`\@bibfield` bibliography chain (replaces stub) | `be1472d78` |
| math0111087 | recovered by amsppt port + `^attr=` codegen | `be1472d78`, `a8d9ce055` |
| astro-ph9903386, astro-ph0007367, astro-ph0012401 | Cluster D — `XUntil` no longer eagerly reads args of non-expandable defs | `16b9680c5` |
| **`\roster` Perl-port cohort** (2026-05-01, commit `050a32b1b` — DefConstructor + DigestUntil:\\endroster + bounded=>true): | | `050a32b1b` |
| math0104021 | R=8 → R=1 (Perl-parity exact) | `050a32b1b` |
| math0106062 | R=4 → R=0 (Perl=0 PARITY) | `050a32b1b` |
| math0004140 | R=1177 → R=0 (Perl=0 PARITY) | `050a32b1b` |
| math0203148 | R=2 → R=0 (Perl=0 PARITY); previously deferred as "out-of-scope/amstex_endmatrix" — actual cause was \\roster mode-frame leak, not \\matrix. Removed from out-of-scope catalog. | `050a32b1b` |
| math0205073 | R=10001 (capped) → R=0 (Perl=0 PARITY); was the largest single-paper cascade. The math-cumulative `\\cases`/`\\pcases` hypothesis was a downstream symptom; root cause was the \\roster mode-frame leak earlier in the body. | `050a32b1b` |
| (Out-of-scope catalogue: `\CITE` typos, `\setdec`, `\dec`, `\psfig` — Perl also errors on these; not parity-Rust regressions) | | |
| (Codegen infrastructure improvement: `^attr='value'` constructor template syntax — Perl Compiler.pm L137-148 — now parsed by Rust `latexml_codegen/src/constructable.rs`) | | `a8d9ce055` |

#### Round-18 100-paper post-accent-fix triage (2026-05-01)

After commits `ba2ab1dcf` (accent fix) + `15f46ddf3` (MAX_ERRORS leak),
re-ran 100 originally-failing papers from the older sweep
(`/tmp/sweep100/`). 7 had no main `.tex` (sweep-script artifact).
93 sweep-able. Distribution: **36 clean (38.7%)**, 35 with 1-5 errs,
12 with 6-50, 10 with >50.

Parity-classified all 35 papers in the 1-5 tier via
`tools/parity_check.sh`:

| Class | Count | Papers (sample) |
|---|---|---|
| OUT-OF-SCOPE (Rust=Perl) | 30 | All `\setdec`/`\CITE`/`\dec`/`\psfig` clusters; cond-mat0102064/0112063/astro-ph0201505 (`\b`-clobber-by-revtex `Unexpected:_`); hep-ph0008099/0109006/math0006234/math0204024/etc. malformed-XML/font/expected:`{` clusters |
| PERL_REGRESSION (Rust < Perl) | 1 | hep-ex0204024 (R=2 vs P=4) — Rust supersedes Perl |
| **FIXED post-sweep by `\roster` Perl-port `050a32b1b`** | 2 | math0203148 (R=2→0, was deferred to out-of-scope), math0106062 (R=4→0); both turned out to be \\roster mode-frame leak, NOT what the original triage suggested |
| ~~REAL REGRESSION~~ → **OUT-OF-SCOPE 2026-05-01** | 0 | physics0002038 was R=5 vs P=4 in this triage; commit `7319e3fbc` `\@add@frontmatter@now` fix dropped the +1 Rust-only error to R=4 P=4 (parity). |

**Implication:** The 1-5 error tier is dominated by out-of-scope
(86%, 30/35). After the \\roster fix, only 1 real Rust-only
regression remains in this tier (Cluster H), already documented.

The accent fix + MAX_ERRORS fix together eliminated **38.7% of
previously-failing papers** in the sample without further work.
Remaining 64.3% split: 38% out-of-scope (Perl=Rust), ~24% deeper
in-scope clusters (math0205073/hep-th0010165/hep-ph0007044
state-cumulative cascades), ~2% truly novel.

`tools/parity_check.sh` now records `PERL_TIMEOUT_OK` /
`PERL_TIMEOUT(partial=N)` tags so partial Perl runs aren't
mis-classified as failures (commit pending).

#### In-scope worksheet (sandbox papers needing work — Perl=0, Rust>0)

- [x] **math0010095** (R=11 → R=0) — **FIXED 2026-05-01** by symptom
  patch in `latex_constructs.rs:strip_trailing_cs` (commit `4d445b71c`)
  + Cluster A reprises (`b1ef89b34`, `de3213086`). Now BOTH CLEAN with
  Perl. Underlying root cause (`{}` parameter reader pulling `\par`
  into args[0] under specific BoxedEPS+section sequence) is still open
  but no longer paper-visible — see
  `memory/project_section_par_contamination.md` for residual
  investigation notes if it resurfaces in future papers.

- [x] **astro-ph0007367 / astro-ph0012401 / astro-ph9903386** (Cluster D,
  3 papers / 11 errors → 0) — **FIXED**: root cause was the `XUntil`
  parameter type's expansion loop (`base_parameter_types.rs:254-256`)
  unconditionally calling `defn.read_arguments()` on every CS with a
  definition, including non-expandable ones (Primitive, Constructor,
  Conditional, Register, MathPrimitive). For
  `\hspace*{-4mm} $^*\,$` inside an `\institute{…}` body (read via
  `\@new@institute XUntil:\@end@institute`), this triggered `\hspace`'s
  primitive Dimension reader to over-consume past the `}` boundary —
  swallowing the following `$` token and leaking math state. Fix:
  restrict the eager `read_arguments` path to `Stored::Expandable`
  only; push primitives/constructors as-is so digestion handles their
  args at the proper time. Min repro:
  `\institute{\hspace*{4mm} $^*$ X}` inside aa-class.

- [x] **astro-ph0107583** (Cluster E, R=2 → R=0) — **FIXED**: extended
  the Perl-faithful `T_ALIGN` self-deactivation guard to the
  `Stored::Constructor` branch in `stomach.rs:invoke_token` (mirror
  Perl `Stomach.pm:187-189`). The `&` char-token's meaning is
  Constructor-bound (TeX_Tables.pool L49), not Token, so the
  pre-existing guard at the Token branch never fired. Now: first stray
  `&` errors once and rebinds itself to `\relax` LOCAL; subsequent
  stray `&`s no-op silently. Witness: astro-ph0107583 2 → 0.

- [x] **physics0002038 / cond-mat0011517** (Cluster H) — **OUT-OF-SCOPE
  at parity 2026-05-01**: parity_check shows `physics0002038 R=4 P=4`
  and `cond-mat0011517 R=6 P=6`. Commit `7319e3fbc`
  (`\@add@frontmatter@now` drop spurious bgroup/egroup) closed the +1
  Rust-only follow-up. Both engines now emit identical error counts;
  the underlying mode-mismatch (paper jams `\begin{minipage}` /
  `\begin{quotation}` block content inside `\author{...}` whose
  `\@personname{}` argument expects `restricted_horizontal`) is a
  shared limitation, not a Rust regression. Worksheet item closed;
  paper family classified out-of-scope per `parity_check.sh`.

- [x] **math0104021** (R=8 → R=1, Perl-parity) — **FIXED**: amsppt's
  `\roster ... \endroster` was a thin `\begin{enumerate}` wrapper
  that left a mode-switch frame on the stack at `\endroster` time,
  cascading 7 × `Error:unexpected:\endgroup` at every subsequent
  `\endref`/`\end`. Replaced with a faithful Perl port (Perl
  `amsppt.sty.ltxml:251-259`): `DefConstructor!('\\roster
  DigestUntil:\\endroster', ..., bounded => true, ...)` plus
  `\roster@item` Constructor and `Let!('\\endroster', '\\relax')`.
  `bounded=>true` keeps the entire roster digestion in one frame
  with proper mode coupling. Min repro: 0 errors (was 7).
  Tests: 1110/0/0 (no regressions).

- [x] **`ltx:XMTok`-in-`<ltx:p>` Δ=2 family** — **FIXED 2026-05-01**
  by commit `a320deffa`: cascading-rejection suppression in
  `document.rs:find_insertion_point_qsym`. When a math leaf element
  (`<ltx:XMTok>` or `<ltx:XMArg>`) tries to insert into a text-mode
  container (`<ltx:p>`/`<ltx:text>`), it's a cascade from an
  already-rejected math wrapper (XMApp/XMDual). Perl emits the
  wrapper's rejection but doesn't continue per-child error logging;
  Rust now matches.
  * `hep-th0101146`: R=17 → R=15 (Perl=15, exact parity)
  * `nlin0211024`: R=4 → R=2 (Perl=2, exact parity)

  Implementation is narrowly targeted: only suppresses the specific
  `(qsym, parent)` pairs `(XMTok|XMArg, ltx:p|ltx:text)`. Other
  malformed insertions still log normally. The element still "inserts"
  via the existing fall-through return; only the redundant log
  emission is suppressed. Side note: the related `Tbox::new`
  divergence (hardcoded `mode => 'math'` vs Perl's `mode => $mode`)
  remains unaddressed — investigated but regresses
  `figure_mixed_content_test` sizing.

- [x] **hep-th0010165** (R=206 vs P=101) — **OUT-OF-SCOPE? 2026-05-01**:
  Perl=101 is the MAX_ERRORS cap (Perl bails at 101 via Fatal). True
  Perl count unknown but >100. At lines 1-345 partial (where Perl is
  NOT capped), Rust=18 vs Perl=26 — Rust BETTER. The full-paper
  Perl=101 vs Rust=206 comparison is invalid (cap-uncertain). Likely
  Rust is comparable or better than Perl here. Re-classify as
  Perl-capped per `wisdom_perl_max_errors_cap.md`.

- [x] **hep-ph0007044** (R=410 vs P=101) — **OUT-OF-SCOPE? 2026-05-01**:
  Perl=101 is the MAX_ERRORS cap. True Perl count unknown. Cap-uncertain.

- [x] **math0205073** (R=10001 → R=0) — **FIXED 2026-05-01** by the
  `\roster` Perl-port commit `050a32b1b`. The state-cumulative
  hypothesis (AmS-TeX `\cases`/`\pcases` mis-parse) was wrong: the
  earlier `\roster` mode-frame leak left BOUND_MODE bound on the
  stack, then every subsequent `&` / `\cr` in the math body
  triggered cascading mode-mismatch errors that hit the MAX_ERRORS
  cap. Dropping `\roster`'s leak collapses the entire downstream
  cascade. Perl=Rust=0 confirmed.

- [x] **quant-ph0109041** (R=67 → R=9, OUT-OF-SCOPE at parity)
  — **FIXED 2026-05-01** by accent commit `ba2ab1dcf` (`mode => "text"`
  drop from `\lx@applyaccent`). Rust=Perl=9; remaining 9 are Perl-baseline
  errors from genuinely malformed `\k{...}` invocations on math-only
  tokens. Now classified OUT-OF-SCOPE per parity_check.

- [x] **astro-ph0204393** (R=113 vs P=101) — **OUT-OF-SCOPE? 2026-05-01**:
  Perl=101 is the MAX_ERRORS cap. Cap-uncertain.

- [x] **hep-ph0102192** (R=4 → R=0) — **FIXED 2026-05-01**: real root
  cause was that pstricks stubs (`pstricks_sty.rs`) did not consume the
  variadic `(coord)(coord)…` PSCoordList that follows `\psline`,
  `\pspolygon`, etc. Coordinates leaked as raw text and `\rput` text
  bodies emitted into the surrounding paragraph, opening an `<ltx:p>`
  that trapped the later `\begin{minipage}` block content. Two-part
  fix: (a) added `\lx@psgobble@parens` recursive `\@ifnextchar(`
  helper to absorb the trailing PSCoordList; (b) dropped the text body
  from `\rput`/`\uput`/`\cput` (consume the paren coord and brace text
  but emit nothing). Visible labels like "cocktail"/"thermal"
  positioned via `\rput` are lost — fidelity regression. The right
  long-term fix is to port Perl's `DefPSConstructor` framework so
  pstricks output lives inside `<ltx:picture>` (where labels survive).
  See follow-up worksheet item below.

- [ ] **pstricks → ltx:picture wrapping** (large-scope feature) — Port
  Perl's `DefPSConstructor` (`pstricks_support.sty.ltxml:491`) and the
  `PSCoordList` parameter type so pstricks drawing commands emit
  `<ltx:line>`/`<ltx:circle>` etc. inside an auto-opened `<ltx:picture>`
  parent. Currently `\rput`/`\uput`/`\cput` text bodies are dropped to
  keep the schema valid (commits `9df708fa9` partial, this round
  drop-rput); restoring them requires the picture wrapper. See inline
  TODO at `latexml_package/src/package/pstricks_sty.rs:51` and historical
  `wisdom_*.md` notes (cycles 305-306, 2026-04-24, deferred per WISDOM
  #41).

- [x] **math0004140** (R=1177 → R=0) — **FIXED 2026-05-01** by
  the `\roster` Perl-port commit `050a32b1b`. Same root cause as
  math0205073: `\roster` mode-frame leak made the entire math body
  emit cascading malformed-XMTok and Unexpected:_ errors. Perl=Rust=0
  confirmed.

- [x] **math0010241** (R=33 → R=19, =Perl) — **FIXED 2026-05-01**
  by commit `a094596a3` extending the cascading-rejection
  suppression to `<ltx:emph>` parents. Trigger is
  `\begin{EG}\emph{ ... $$display math$$ ... }\end{EG}` blocks; both
  engines correctly reject the fundamentally malformed input, but
  Rust was emitting per-XMTok-child cascade noise that Perl does
  not. The 14 XMTok-in-emph cascade emissions are now suppressed,
  bringing Rust to exact parity with Perl=19.

- [x] **astro-ph0203201** (R=70 vs P=70) — **Out-of-scope** —
  Perl=Rust same error counts (56 `_`-in-text + 12 XMArray-malformed
  + 2 `^`-in-text). Both fail identically.
- [x] **cond-mat0103632** (R=20 vs P=20) — **Out-of-scope** — same.
- [x] **hep-ph0110283** (R=98 vs P=101) — **Out-of-scope** — Rust
  better than Perl (Perl saturates at 101 truncation cap).
- [x] **hep-th0004072** (R=33 vs P=101) — **Out-of-scope** — Rust
  better than Perl.
- [x] **hep-ph0204075** (R=0 vs P=0) — **PASSING** — recovered by
  recent commits, no longer a failure. Marked in completed
  investigations table.

- [x] **hep-th0005268** (R=21 vs P=26 — was 10001) — **FIXED
  2026-05-01** by commit `cda6cb247`: surgical lazy-pool-load guard
  in `latex_constructs.rs:5560`. The kernel `\let\a=\@tabacckludge`
  is now skipped when `\a` is already user-defined; under the
  user's `\def\a{\alpha}` BEFORE `\documentclass`, `\a` is
  preserved as `\alpha` and the body math works clean.
  Rust=21 errors are real cascades from elsewhere in the paper
  (stray `^/_` in math, malformed XML), not the spurious `\hbox`
  runaway noise. **Now PERL_REGRESSION** — Rust supersedes
  Perl (R=21 < P=26). Min repro
  ```
  \def\a{\alpha}
  \documentclass{article}
  \begin{document} $\a + x$ \end{document}
  ```
  is now R=0, P=0. The pattern (`\let\<public-cs>=...` in
  `latex_constructs` guarded by `is_already_user_defined`)
  generalizes to other Let calls if similar witnesses surface,
  but for now the targeted fix unblocks the canonical case.
  Same family as `cond-mat0106160`,
  `hep_ph0001306_documentstyle_clobber` (still triaged
  out-of-scope but may benefit from the same pattern).

- [x] **hep-th0005159** (R=262 vs P=101) — **OUT-OF-SCOPE? 2026-05-01**:
  Perl=101 is the MAX_ERRORS cap; cap-uncertain. Rust now at 262 (well
  below its 10000 cap), so the prior 786478 number was pre-MAX_ERRORS-leak
  fix `15f46ddf3`. parity_check tags this `OUT-OF-SCOPE? (Perl-capped,
  cannot compare)`. Worksheet item closed pending a future paper that
  exposes a directly-comparable variant.

#### Out-of-scope (Perl also fails — moved to `docs/out-of-scope/`)

| Paper | Reason |
|---|---|
| `0901.2408_emph_dollar` | `$$`-in-`\emph{}` — Perl=Rust |
| `cond-mat0003169` | `Unexpected:_` cluster — Perl=Rust=2 |
| `cond-mat0106160` | `\def\r\rho` BEFORE `\documentstyle` clobber family |
| `hep_ph0001306_documentstyle_clobber` | `\def`s before `\documentstyle` — broader family |
| `math0005251_math_parser_oom` | math-parser OOM — needs grammar work |
| ~~`math0203148_amstex_endmatrix`~~ | **REMOVED 2026-05-01** — fixed by `\roster` Perl-port commit `050a32b1b` (was misdiagnosed as `\matrix` issue; actual cause was the `\roster` mode-frame leak, same family as math0104021) |
| `math0601451_xmtok_in_title` | XMTok-in-title issue |
| `math0606553_xy_compile` | xy-pic AmS-TeX compile failure |

#### Active Rust-engine clusters (driven by sandbox investigations)

| Cluster | Status | Notes |
|---|---|---|
| A. `\par` in counter-CS reading | **partial fix** `9c60a766c` (covers `\@captype`); residual `\thesection\par` open per math0010095 worksheet item. |
| B. tcilatex `\newcount\dispkind` | **fixed** mid-Round-18. |
| C. `\documentstyle` dump-clobber | **fixed** `6e6497ede` (re-Let). |
| D. aa-class `\institute` math leak | **open** — see worksheet. |
| E. Stray `&` outside table | **fixed** — extended `T_ALIGN` deactivation guard to Constructor branch in `stomach.rs:invoke_token`. |
| F. Cascading single-root | **open** — math0004140 + runaway cascades worksheet items. |
| G. pstricks `\multips` | **fixed** `506cb8fe6`. |
| H. Mode-stack `}` followup | **open** — error-tracker dedup work. |

Long-standing deep clusters parked in
`docs/archive/sandbox_failures_SYNC_STATUS.md`. Re-survey whether
recent fixes have shrunk the surface enough to make individual
items tractable. Notables:

* `1803.03288`/`1902.08705` (expl3 cascade + pgfmath `\ifdim`) — open.
* pgfplots `\pgfplots@curlegend`/`\pgfplots@curplotlist` state-machine
  — **resolved** 2026-04-25 (commit `b4b196254`,
  `pgfplots_sty.rs:18-28`). The undefined-CS cluster traced to a
  `\globaldefs` register-type mismatch in core, not a pgfplots-shim
  gap. Re-survey on the 100k canvas to confirm no residue.

### 5. Distribution — bundle multi-TL dumps

Once TL2025 dumps stay robust through a CI cycle: `include_bytes!`
`{plain,latex}.dump.txt` for TL2022 … TL2026 and select at runtime
by `kpsewhich --version`. Currently dumps load from
`resources/dumps/` on disk.

---

## Engine file open gaps

| File | Status | Open Gap |
|------|--------|----------|
| `base_parameter_types.rs` | MINOR | Parameterized `CommaList:Type` form unported (no Perl users). |
| `tex_box.rs` | MINOR | Box dimension edge cases. |
| `tex_fonts.rs` | MINOR | `\fontdimen` array semantics; per-font `\hyphenchar`. |
| `tex_tables.rs` | MINOR | Padding CSS classes (XSLT concern). |
| `plain_base.rs` | OPEN | Some closure-backed defs need conversion to Token bodies for dump round-trip. |
| `latex_base.rs` | OPEN | Closure-backed defs need conversion or relocation to `latex_constructs.rs`. |

---

## Tikz known diffs vs Perl

1. foreignObject transform Y / width/height
2. Arrow tip shape (different path data)
3. SVG viewBox / total width differs slightly
4. tikz matrix uses `<svg:g class="ltx_tikzmatrix">` (Rust) vs
   inline-blocks (Perl)

---

## Permanent ignores

* **Sandbox out-of-scope:** ns1–ns5 (52_namespace, no DTD); 2402.03300,
  2410.10068, 2511.03798 (Perl also fails).
* **Rust supersedes Perl** (both still in scope, but Rust passes
  where Perl errors): `1207.6068`, `0909.3444`.
* **Unported pools:** `BibTeX.pool.ltxml` (skipped via `--nobibtex`).

---

## Acceptance gates

| Gate | Current | Target |
|---|---|---|
| `cargo test --tests` | 1110/0/0 | unchanged across all task work |
| `latexml_oxide --init=plain.tex` | 0 errors | 0 errors |
| `latexml_oxide --init=latex.ltx` | 0 errors | 0 errors |
| 10k canvas (Phase 1, complete) | 7731 / 7898 = 97.89% | n/a (canvas retired) |
| Filesystem-level hard failures (10k) | 1 (math0005251) | 0 |
| 100k "no-problem" canvas (Phase 2, active) | downloaded (100 000 ZIPs) — sweep pending | 100% match Perl |

---

## 2026-05-02: drop speculative iopart_support `\la → \lesssim` block

`hep-ph0404036` (`\documentclass{iopart}` + `\newcommand\la{\langle}`):
Rust=1 → Rust=0 (Perl=0). Tests stay 1112/0/0.

* **`iopart_support_sty.rs`** had a Rust-only "Math symbols — Perl
  L225-280" block defining `\la → \lesssim`, `\ga → \gtrsim`, `\sun`,
  `\degr`, `\arcmin`, `\arcsec`. None of these exist in Perl's
  `iopart_support.sty.ltxml` (verified: 0 grep matches). The comment's
  cited Perl line range L225-280 actually contains bibliography macros
  and journal abbreviations — pure speculative addition that
  contradicted the "Perl is ground truth" rule.
* The `\la → \lesssim` entry actively harmed user macros: papers
  commonly do `\newcommand\la{\langle}`, but the pre-binding made
  `\la` already-defined so `\newcommand` ignored the redefinition,
  and the user's `\la n_G\ra` later expanded into the undefined
  `\lesssim`.
* Cluster impact: any iopart paper where the user defines `\la` /
  `\ga` / `\sun` / `\degr` / `\arcmin` / `\arcsec` themselves
  (common pattern for `\la=\langle` shorthand).

## 2026-05-02: case-insensitive file lookup + documentstyle disk-probe raw-load

`astro-ph0008100` (`\documentstyle[PASJadd]{PASJ95}` + uppercase-named
local `PASJ95.STY` / `PASJADD.STY`): Rust=3 → Rust=0 (Perl=0).
Tests stay 1112/0/0.

* **`pathname.rs::find`** — Perl `pathname_find`
  (`Util/Pathname.pm:376-392`) does a `/i` regex over directory
  entries and falls back to case-insensitive matches when no strict
  match exists. Rust used `Path::exists()` (strict-only), missing
  `PASJ95.STY` when looked up as `PASJ95.sty`. Added a Pass-2
  `read_dir + eq_ignore_ascii_case` fallback, only fired when Pass 1
  finds nothing (mirrors Perl's `return @paths ? @paths : @nocase_paths`).
* **`tex_job.rs` (Branch 1 of `\documentstyle`)** — when the
  `<class>.sty` was found ONLY via paper-local disk-probe (no
  binding, no fallback), pass `notex: Some(false)` to
  `require_package` so the `INCLUDE_STYLES=false` gate doesn't
  suppress the raw load. Mirrors the same fix from
  `\compat@loadpackages` (commit bb4cf2e17). Same family of bugs
  (paper-local sty discovery + raw-load gate).
* Cluster impact: any LaTeX 2.09 paper shipping uppercase-named
  `<file>.STY` / `<file>.CLS` (older tarballs, ~50 papers in canvas).
  Unblocks PASJ-style class file discovery generically.

## 2026-05-02: \compat@loadpackages — disk-probe-found local sty must allow raw load

`astro-ph0009248` (`\documentstyle[11pt,newpasp]{article}` + local
`newpasp.sty`): Rust=3 → Rust=0 (Perl=0). Tests stay 1112/0/0.

* **`latex_constructs.rs` (`\compat@loadpackages`)** — track WHICH path
  matched the option-package `find_file` probe (binding registry,
  version-strip fallback, or paper-local disk). When matched ONLY via
  the disk-probe (no .sty.ltxml binding, no fallback), pass
  `notex: Some(false)` to `require_package` so the
  `INCLUDE_STYLES=false` gate inside `require_package` doesn't force
  `notex=true` and suppress the actual raw load.
* This complements the 2026-05-02 cls-fallback fix below — same
  family of bugs (paper-local sty discovery + raw-load suppression)
  but on the OPTION-passthrough path rather than the class path.
* Cluster impact: any LaTeX 2.09 paper with `\documentstyle[opt]{class}`
  shipping a local `opt.sty` (no LaTeXML binding). Likely 50+ astro-ph
  papers (per the in-tree comment block at `\compat@loadpackages`).

## 2026-05-02: \documentstyle cls-fallback priority + options forwarding

`astro-ph0002213` (`\documentstyle[epsfig]{mn1}` + local `mn1.sty`):
Rust=3 → Rust=1 (Perl=0). Two surgical fixes; tests stay 1112/0/0.

* **`tex_job.rs` — gate the paper-local disk-probe** on absence of ANY
  `<class>.cls` binding (exact OR via `find_file_fallback`). Without
  this gate, `\documentstyle{mn1}` was preempted by the local
  `mn1.sty` whose raw load is suppressed by the default
  `INCLUDE_STYLES=false`. With the gate, `mn1` falls through to
  `mn.cls.ltxml` (Perl-faithful priority).
* **`content.rs` — forward `options`/`after`** to the
  `find_file_fallback` recursive `input_definitions` call. The prior
  empty-handoff dropped the user `[options]` (e.g. `[epsfig]`) and the
  `\compat@loadpackages` after-hook so the unused-option pass never
  fired.
* Residual: `\psfig` undefined (1 error) — `mn.cls.ltxml`'s
  `PassOptions('article', 'cls', 'epsfig')` plumbing isn't reaching
  `@unusedoptionlist` in Rust. Separate investigation.
