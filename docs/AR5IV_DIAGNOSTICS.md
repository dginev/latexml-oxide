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

> ## ✅ SPRINT CLOSED (2026-07-18)
>
> The mini-sprint's landable engine value is complete: **13 issues fixed** on PR
> #306 (xcolor global `dvipsnames`, `\sidecaptionvpos`, `\newtcblisting` verbatim
> close, three meta-class frontmatter bindings, agujournal end-matter, the
> `_load_attempted` deferred-load parity fix, `\ifmmode` coverage) — all
> CI-green. Every **remaining** deep issue was minimally reproduced and
> cross-checked vs pdflatex **and** same-host Perl (see the Diagnostic update
> below); none has a shallow faithful fix. They resolve three ways:
> **(1)** parity / Rust-better (author-malformed input — not bugs);
> **(2)** the KNOWN, post-release `\lx@begin@alignment`-in-math cluster (blkarray
> + code-env — deep core work, now with a 4-line repro in
> `docs/known_crashes/blkarray_halign_math/`);
> **(3)** genuine-but-CONTAINED deep bugs (tikz `calc`-coord recursion 2508.07407;
> deferred `\or` 2412.06264 — graceful, post-release).
> The **~48 already-CLEAN** papers (sample re-verified 0-error) close on the ar5iv
> **redeploy** from the current binary + a maintainer batch-comment — no code
> change. No further mini-sprint engine work is planned.

# Implementation plans — remaining deep issues (2026-07-18)

The self-contained wins are landed (13 issues on PR #306; see the "Ranked
worklist" and per-cluster notes below). What remains is deep or shared-with-Perl.
Each plan below is written to be picked up cold: symptom + evidence, a root-cause
hypothesis, the concrete approach, the files, the traps, and the test. Ordered by
value × tractability. **Golden rules still bind:** Perl is ground truth, classify
vs same-host Perl **verbose**, cross-check pathological inputs with `pdflatex`,
never downgrade an Error to "pass", diverge only when `OXIDIZED_DESIGN.md` (or an
explicit surpass-Perl decision) sanctions it, and add a red/green guard per fix.

> ## ⚠️ Diagnostic update (2026-07-18) — P1/P2/P3 investigated → RECLASSIFIED
>
> The P1 "highest value genuine Rust cluster" hypothesis was **wrong**. Each
> witness was minimally reproduced and cross-checked against **both** pdflatex
> and same-host Perl. Findings (repros saved under `docs/reproducers/` and
> `docs/known_crashes/blkarray_halign_math/`):
>
> | issue | paper | verdict | root (all cross-checked vs pdflatex) |
> |---|---|---|---|
> | #568 | 2309.16609 | **PARITY / Rust-better** | fragile `\lstinline{...{9}...}` (brace delim reads to 1st `}`); Rust 18 = Perl 18 **+ a Perl Fatal**. pdflatex "Too many }'s" (recovers). `docs/reproducers/lstinline_brace_2309.16609.tex` |
> | #477 | 2310.07298 | **PARITY** | author typo `$\boldsymbol{\Delta$}` (misplaced `$`); Rust 24 = Perl 24. Rust flags it like pdflatex; Perl lenient. `docs/reproducers/misplaced_dollar_boldsymbol_2310.07298.tex` |
> | #497/#516 | 2405.21060 | **Rust-BETTER** | same misplaced-`$` family; Rust **26** vs Perl **102 + Fatal** |
> | #472 | 2311.06609 | **Rust-worse, deferred** | custom `code` env (tabbing+`$…$`); Rust 82 vs Perl 39. Removing the code env → **Rust 0**, Perl 14: Rust converts the *rest* perfectly. The amplifier is non-minimizable and `egroup` recovery is a faithful port. `docs/reproducers/tabbing_math_code_env_2311.06609.tex` |
> | #594/#473 | 1811.10792, 2310.17416 | **known cluster (both engines fail)** | **`blkarray`** `block{(cc)}` in `blockarray` in display math → Rust OOM (4.5 GB/12 s), **Perl also hangs** (90 s/rc=124), pdflatex clean. 4-line repro. |
>
> **The unifying root is the KNOWN, documented, HIGH-DIFFICULTY, post-release
> `\lx@begin@alignment` / `\halign`-in-math cluster** (`stomach.rs::egroup`
> refuses to pop a per-cell inline-math frame at an alignment close; ~12.1k
> full-arXiv fatals). See `docs/known_crashes/{kbordermatrix,blkarray}_halign_math/`.
> This mini-sprint's contribution: a **much smaller** repro (4-line blkarray) and
> new witnesses (blkarray degrades **both** engines, unlike kbordermatrix). The
> deep core fix is out of mini-sprint scope; a `blkarray` binding is the safe
> sidestep but needs non-trivial `block`-delimiter modelling (probed, deferred).
> Net: **P1 is not a fixable Rust-only cluster** — Rust is at parity-or-better on
> every shared construct. The P1–P8 plans below are kept for provenance but P1/P2/
> P3 are superseded by this update.

## P1 — Alignment env inside a restricted-horizontal box (GENUINE, highest value)

**Issues:** #568 (2309.16609, 31), #497/#516 (2405.21060, 26), #477 (2310.07298,
24), and the RUST-WORSE #594 (1811.10792, timeout) / #472 (2311.06609, 82) share
the same machinery. This is the single largest *genuine* Rust cluster.

**Symptom.** `Error:unexpected:\lx@begin@alignment Attempt to close a group that
switched to mode restricted_horizontal` (15× on 2309.16609), plus
`\lx@end@inline@math`, `\hbox Attempt to end mode restricted_horizontal`,
`\endgroup Attempt to close non-boxing group`. Errors fire at "Anonymous String"
/ macro-expanded locations, not source lines.

**Evidence / hypothesis.** An amsmath alignment (`align`/`cases`/`split`/`aligned`)
or an inline `$…$` is being digested **inside a restricted-horizontal box**
(`\hbox`/`\mbox`/`\parbox`/`\vcenter`, or a CJK box on 2309.16609 which loads
`CJKutf8`; 2405.21060 uses `mathtools` `\DeclarePairedDelimiter`). The alignment's
`\lx@begin@alignment` opens an alignment group, but the surrounding box already
switched the mode to `restricted_horizontal`, so when the alignment tries to
close/realign it "closes a group that switched mode" → cascade. Perl keeps a
looser mode stack here (17 vs Rust 82 on 2311.06609), so Rust is stricter-wrong.

**Approach.**
1. Build the minimal repro from the smallest witness: try `\mbox{$\begin{aligned}
   a&=b\\ c&=d\end{aligned}$}` and `\hbox{\begin{cases}…\end{cases}}`, and the
   CJK case `\begin{CJK*}…$…$…\end{CJK*}`. One will reproduce `Attempt to close a
   group that switched to mode restricted_horizontal`.
2. Trace with `LXML_TRACE_BOUND_MODE=1` (see mhchem memory) around
   `\lx@begin@alignment` / the mode stack (`latexml_core::stomach` mode
   transitions; `latexml_engine` alignment constructs `\halign`/`\lx@…@alignment`).
   Find where Rust pushes `restricted_horizontal` but should allow an alignment to
   open a nested math/alignment group (Perl's `beginMode`/`endMode` pairing).
3. The fix is almost certainly in the **mode/group pairing** when an alignment or
   inline-math opens inside a box: permit the alignment group to nest (open its
   own mode frame) rather than asserting the box's mode. Cite the Perl
   `Stomach`/`Gullet` alignment source for the faithful pairing.

**Files.** `latexml_core/src/stomach*.rs` (mode stack), `latexml_engine/src/*`
alignment constructs (`\lx@begin@alignment`, `\halign`, `\lx@end@inline@math`),
`latexml_core::stack_guard`. **Traps:** don't loosen the mode check globally (it
guards real malformed input); scope to the alignment/inline-math-in-box case.
Re-run every alignment test (`tests/alignment`, `tests/ams`, `tests/math`) — this
is core machinery.

**Test.** `.tex`+`.xml` pair under `tests/alignment` with align/cases/aligned
inside `\mbox`/`\hbox` and (separately) a CJK box; assert 0 errors and correct
`<ltx:XMArray>` nesting. Value: 4–5 issues, and it de-risks the two RUST-WORSE
timeouts (which begin with the same `\lx@end@inline@math` cascade before looping).

## P2 — 2311.06609 siamart paper-local `code` env (#472, RUST-WORSE 82 vs 17)

**Root.** A paper-local `\newenvironment{code}` = `list` + `tabbing` + `\mathcode
\`\:` remap + custom `\mynewline`, holding inline `$…$` cells. Raw `tabbing`+`$…$`
is clean in BOTH engines (verified), so it is the **custom-env composition** that
breaks group/mode balance (same family as P1 — `\lx@begin@alignment`/
`\lx@end@inline@math`). **Approach:** land P1 first, then re-measure; the residual
is likely the `list`+`tabbing`+`\mynewline` interaction — min-repro by peeling the
env to the smallest failing combo (start from `\begin{list}{}{}\item[]
\begin{tabbing}\>$a=b$\\ \end{tabbing}\end{list}`). **Files:** tabbing constructs +
mode stack. **Test:** the min-repro as a `tests/alignment` pair.

## P3 — Rust-only timeouts (#594 1811.10792, #473 2310.17416)

**Symptom.** Rust hits the 60 s wall-clock timeout; Perl completes (with 101
errors → its `too_many_errors` fatal, so Perl is not "clean" either, but it
terminates). Both begin with the P1 cascade (`\lx@end@inline@math`, `\halign`,
`_`) then loop. **Hypothesis.** The P1 group-mode cascade drives error-recovery
that re-digests the same tokens → a grind, not a true infinite loop (the box list
grows). **Approach.** (a) land P1 — likely removes the cascade that feeds the
loop; (b) if a loop remains, sample it with the `EXP_TRACE` histogram (see
`limit-counting-raise-not-reduce` memory) to find the hot re-digest site;
**RAISE** the relevant guard limit rather than reduce counting, or fix the
recovery to not re-enqueue. **NEVER** downgrade to a cap that hides the cascade.
**Files:** `latexml_core::stack_guard`, the recovery path in
`core_interface::digest_internal`. **Test:** a bounded min-repro that converts
under the timeout with the expected (small) error count.

## P4 — Shared-with-Perl timeouts (tikz / tcolorbox / forest / pgf)

**Issues:** #599 (1802.01134), #598 (1611.02087), #596 (2505.01658), #471
(2308.04512), #522 (2405.19920), #533 (2406.15882), #546 (2504.07033), #550
(2501.09223), #551 (2501.10235). **Both engines time out** (Perl `rc=124` at 1×
too — re-verify each at 1× first, the sweep ran 10× parallel). These are the
heavy graphics stacks (tikz pictures, pgfplots, tcolorbox, forest). **Approach.**
Per-paper: (1) confirm Perl also times out at 1× (if so → parity, and the lever is
performance not correctness — see `docs/performance/ARXIV_PERFORMANCE.md`, the
17% math over-parse + tikz-cd digest levers); (2) locate the hot construct
(`--timeout` + the sampled histogram) — usually one runaway tikz/pgfmath loop or a
tcolorbox `most`-library expansion; (3) either bind the offending construct to a
placeholder (the `discard_env_body` pattern, as nicematrix/forest do) or fix the
specific pgfmath/tikz loop. **Do not** blanket-raise the timeout. **Files:**
`pgfmath*`, `tikz*`, `tcolorbox_sty.rs`, contrib graphics stubs. **Test:** the
paper converts under a fixed timeout; guard the specific construct with a min-repro
if a real fix (not a stub) lands. Lower priority than P1–P3 (mostly parity).

## P5 — `_` / `^` "script can only appear in math mode" cascade

**Issues:** #601 (2604.16007, 60), #557 (2305.05665, 33), #597 (1404.3143,
ytableau — Perl fatals worse), #483 (2312.11805), #523 (2408.15403), #585
(1802.09089). **Errors are macro-generated** ("Anonymous String"), NOT source
lines. **Ruled out:** the Rust `axessibility` binding is a faithful, identical
port of Perl's (parity) — not the cause. **Mixed deep roots:** (a) `ytableau`
`\none[\textstyle …]` cells in `align*` (1404.3143 — shared Perl limit, PARITY);
(b) `axessibility[accsupp]` ActualText re-digesting the math source in text mode
(2305.05665 — needs the accsupp alt-text treated as an opaque string, not
re-tokenized); (c) table-cell `_` (2604.16007). **Approach.** Split by root: for
(b), make the axessibility accsupp injection wrap its argument as verbatim/string
(don't re-digest); classify (a)/(c) vs Perl first — likely PARITY (record in
`KNOWN_PERL_ERRORS.md`), fix only where Rust > Perl. **Value:** moderate; several
are parity. **Test:** per-root min-repro.

## P6 — Residuals on already-improved papers

- **2412.06264 (#520) `\or` flood (337, all at `\end{document}`).** The fairmeta
  frontmatter is fixed; the residual is 337 `\or` fired at end-of-document →
  DEFERRED content (floats/endnotes) carrying an unbalanced `\ifcase`/`\or`, or
  nicematrix/luabridge expl3 the stub doesn't balance. **Approach:** bisect the
  deferred content; find the `\or`-emitting construct (likely a nicematrix table or
  an expl3 `\int_case:nn`). Likely parity-ish. Lower priority.
  **2026-07-18 update:** confirmed the 337 `\or` fire in DEFERRED content — the
  first is at `paper; line 3820`, one line PAST `\end{document}` (3819). No literal
  `\ifcase`/`\or` exists in the source, so a package macro (nicematrix/expl3) leaks
  `\or` from an unbalanced `\ifcase` in a float/output-routine flush. Deep +
  deferred; deferred to post-release. Frontmatter (the reported issue) is fixed.
- **2508.07407 (#556) `Stomach:Recursion` — ROOT CORRECTED 2026-07-18.** NOT a
  resizebox/minipage box-loop (that minimal is clean in all three engines). Bisected
  to an inline **`\tikz{…}` picture whose nodes are placed at `calc` coordinates**
  (`($(env.west)+(10mm,6mm)$)`) relative to a sized `cloud` shape, with `\draw`
  arrows — Rust's tikz/pgf coordinate machinery loops (~3-box window) past the
  50000-box guard → `Fatal:Stomach:Recursion`, **caught gracefully** (conversion
  COMPLETES; only the one tikz table is dropped). **GENUINE-RUST-ONLY**: Perl
  completes, pdflatex renders cleanly. Bare Rust reproduces (its OWN tikz binding,
  not a raw-load). Minimal repro:
  `docs/reproducers/tikz_calc_node_recursion_2508.07407.tex`. DEEP tikz-subsystem
  work → deferred, like the `\lx@begin@alignment` cluster; already contained
  (graceful, fidelity-only loss). Frontmatter (the reported issue) is fixed.

## P7 — Parity / shared-Perl singletons (document, don't force)

Both engines fail identically (verify each vs Perl, then record in
`KNOWN_PERL_ERRORS.md`; fix in Rust only if cheap and Perl-shared):
`2602.15902` (#591, `\mintinline`→`\verb` `\ifmmode` — already documented as
parity), `{forest}` stubs (#476/#573), `{pNiceMatrix}` stub (#499), `\filledstar`
(author-undefined, 2402.13846 residual), `\BibSpecAlias` (#485), biblatex
`{refsegment}`/`\defbibfilter` (#580), `\bfR` (#484), `malformed:ltx:bibitem`
(#482), `malformed:ltx:listing` (#554), `\@end@tabular` (#558, 2301.12995 —
check if genuine). **Content-bearing deferrals:** `\titlehead` (#498, scrartcl/KOMA
— needs `\maketitle` integration, not a no-op).

## P8 — Verify + close the already-CLEAN batch (~48 issues)

The largest bucket: ~48 issues already convert **0-error** in latexml-oxide (see
the CLEAN table). They were filed against old Perl output. **Approach:** they close
once the ar5iv corpus is re-served from the current binary (a redeploy, not a code
change). Before closing each, spot-verify the specific reported symptom is gone
(not just 0 errors — e.g. the missing section/figure the user named renders).
Batch a maintainer-facing list; do not post to the tracker unilaterally.

---

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
| **RUST-WORSE** | 3 | genuine Rust regression — top priority (was 4; `2602.15902` reclassified → parity, see cluster 5) |
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
2. **`fairmeta.cls` / ML meta-class cluster** — `2509.24704` (#567) / `2511.16624`
   (#576) / `2412.06264` (#520). **Root (general): an unknown `.cls` body is NOT
   raw-loaded** — OmniBus extracts its `\RequirePackage` dependencies but does not
   execute the body, so every class-defined command (`\metadata`, `\contribution`,
   `\beginappendix`, …) is `Error:undefined` (a `.sty` DOES raw-load; a `.cls`
   does not). **LANDED** `fairmeta_cls.rs` binding (bytedance/fcs pattern): routes
   the frontmatter through `\@add@frontmatter`/`\lx@add@author`/`\lx@add@abstract`
   (title, both authors, affiliations, contribution, metadata, correspondence,
   abstract all captured), pulls in the real deps, and loads `tcolorbox[most]` via
   `pass_options("tcolorbox","sty",["most"])` **before** the require (Perl idiom,
   mirrors `ar5iv.sty.ltxml`) so the enhanced/breakable/skins keys resolve.
   2509.24704 **5→2**, 2511.16624 **4→2** (residual = the paper's own `luabridge`
   expl3, separate). 2412.06264's `\or`/`\f`-fragment flood (483) is a distinct
   paper-specific issue, not frontmatter. Two near-identical sibling classes got
   the same treatment: **`selfevolagent.cls`** (2508.07407/#556 — frontmatter
   captured; residual `Stomach:Recursion` is a box-loop in the paper's
   `paradigms.tex`, separate) and **`openmoss.cls`** (2605.12090/#605 — 14→1: the
   `\definecolor{openmossblue}` set + `\checkdata` frontmatter now defined,
   residual `{forest}` is the parity stub). All three `\RequirePackage[latin,
   english]{babel}` — openmoss's `\addto\extrasenglish` needed babel required in
   the binding.
   **Core parity fix (general — benefits far beyond fairmeta):** the class loads
   `nicematrix` (→ `\RequirePackage{pgfcore}`, faithful to nicematrix.sty:23) then
   `tcolorbox[most]` (whose `skins` library also needs pgfcore). pgfcore has no
   binding, so nicematrix's bare require missed and — via the Rust-only
   `_load_attempted` guard — permanently STARVED tcolorbox's later pgfcore
   raw-load (49 spurious pgf errors); pdflatex loads pgfcore fine in either order.
   Fix (`content.rs`): set `_load_attempted` only when raw-loading was actually
   POSSIBLE (`INCLUDE_STYLES` on / `noltxml`). A miss while raw loading is OFF is a
   deferral, not a genuine "file absent", so a later load once INCLUDE_STYLES turns
   on (inside another package's raw read) may retry — matching pdflatex. The guard
   still fires where its loop-prevention is needed (a raw read is itself
   INCLUDE_STYLES=true). So fairmeta needs **no** `--includestyles`/ar5iv preload:
   49 → 0. No new flag; restricts the existing Rust-only guard to its real case.
   **Harness note:** the guard is a fresh-process **binary-driven** test
   (`92_fairmeta_frontmatter`), not an in-process `tests/contrib` fixture — loading
   a `LoadClass!("OmniBus")` `.cls` then `reset_thread_engine`-ing between files
   (as `can_contrib` does) reads a pre-reset `SymStr` from an unresettable `pin!`
   cache and aborts (the documented one-conversion-per-thread contract). Fresh
   process = how production runs, and why the ~100 other contrib `.cls` bindings
   carry no in-process fixture.
3. **`\lx@begin@alignment` / `\lx@end@inline@math` grouping** — `2311.06609`
   (**82**, RUST-WORSE), `2405.21060` (26), `2310.07298` (24), `2309.16609` (31),
   `1811.10792` (RUST-WORSE timeout). Inline-math / amsmath alignment group
   balance. `2311.06609` root: a **paper-local `code` environment** (`list` +
   `tabbing` + `\mathcode`\`\:` + custom `\mynewline`) whose inline `$…$` cells
   break group balance in Rust more than Perl (raw `tabbing`+math is clean in
   both — verified — so it's the custom-env interaction, not general). Deep,
   paper-specific.
4. **`_` / `^` "script can only appear in math mode" cascade** — `2604.16007`
   (60), `2305.05665` (33), `1404.3143` (ytableau, **parity** — Perl fatals
   worse), `2312.11805`, `2408.15403`. Errors originate at "Anonymous String"
   (macro-generated), not source lines. **Not** axessibility — the Rust
   `axessibility` binding is a faithful, identical port of Perl's (parity); ruled
   out. Mixed deep roots (table cell / pgfplots / ytableau); largely parity.
5. **`\ifmmode` double-`\else` flood** — `2602.15902` (783). **RECLASSIFIED →
   PARITY.** Minimal repro: `\textbf{\mintinline{latex}{CD}}`. Both LaTeXML
   bindings map `\mintinline` → fragile `\verb` (Rust `minted_sty.rs:108`, Perl
   `minted.sty.ltxml:107`); `\verb` inside `\textbf`'s `\ifmmode\else\fi`
   corrupts the conditional **identically in both engines** (verified — Perl
   emits the same `Extra \else` / `\fi` / `\lx@hidden@egroup` cascade). The
   783-vs-101 gap is only Perl bailing at `too_many_errors:100`; per-occurrence
   behaviour is the same. A real-minted `\mintinline` is robust (works in an
   arg), but pdflatex here can't run minted v3 (executable absent) so there is no
   local oracle. **No faithful engine change** (both track Perl); a surpass-both
   "robust `\mintinline`" is possible but out of mini-sprint scope. Direct
   `\ifmmode` coverage added: `tests/expansion/ifmmode`, `.../ensuremath_mode`
   (every branch selection verified to match pdflatex).
6. **`malformed:ltx:subsection/section` nesting** — `2402.13846` (#504),
   `2507.00833` (#569/#570). **LANDED.** Root: a **`\newtcblisting` (tcolorbox
   `listings` library) box captured its body verbatim but never CLOSED** at
   `\end{name}` — the raw library's body reader didn't integrate with LaTeXML's
   verbatim reader, so the listing swallowed following content and a later
   `\section` nested inside `<ltx:verbatim>`. Fix (`tcolorbox_sty.rs`): delegate
   `\newtcblisting{name}[N][d]{tcb-opts}` → listings' `\lstnewenvironment{name}
   [N][d]{}{}` (drop the visual box options), whose verbatim reader terminates
   correctly; `locked` so the raw `\tcbuselibrary{listings}` can't clobber it.
   2507.00833 **8→0**, 2402.13846 **16→1** (residual `\filledstar` = a genuine
   author-undefined macro, not tcblisting). Guard: `95_newtcblisting_verbatim`.
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
  - [x] **LANDED** `\sidecaptionvpos` no-op in sidecap (2408.08435, **1 → 0**,
    #555). Layout hint, no logical output. Guard: `tests/graphics/sidecap_vpos`.
  - [x] **LANDED** `\newtcblisting` verbatim close (#504/#569/#570) — delegate to
    listings' `\lstnewenvironment`. 2507.00833 8→0.
  - [x] **LANDED** three meta-class frontmatter bindings — fairmeta/selfevolagent/
    openmoss (#520/#567/#576/#556/#605).
  - [x] **LANDED** agujournal2019.cls end-matter (#538, 2003.03231 **7 → 0**):
    extend the existing binding with `\RequirePackage{rotating}` (sideways floats)
    + the `{acronyms}`/`{notation}` description-list envs (`\acro`/`\notation` →
    `\item[]`). Guard: `tests/97_agujournal_acronyms`.
  - Deferred (content-bearing, not a safe no-op): `\titlehead` (2305.01582,
    scrartcl/KOMA) needs `\maketitle` integration to render the header text.
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
