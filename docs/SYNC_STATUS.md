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

#### In-scope worksheet (sandbox papers needing work — Perl=0, Rust>0)

- [ ] **math0010095** (R=11) — `\thesection\par` + `\par@ID` cluster.
  `\@@numbered@section`'s digested `args[0]` evaluates to `"section\par"`
  for sections after captioned figures, even though `\@startsection`'s
  emitted tokens are clean. Constructor `{}` arg digestion in vmode
  pulls trailing `\par` from outer stream after captions accumulate
  state. See `memory/project_section_par_contamination.md`. Fix locus:
  `latexml_core/src/binding/parameters.rs` `{}` parameter handler.

- [ ] **astro-ph0007367 / astro-ph0012401 / astro-ph9903386** (Cluster D)
  — aa-class `\institute{...\and...}` math-mode leak. Math frame opens
  during `\@new@institute XUntil:\@end@institute` arg reading (when
  `$^\S\,$` inside an `\and`-segment is consumed), BEFORE `\@add@institute`
  is invoked. Tried `mode => "restricted_horizontal"` on the
  constructor — made things worse (R=3→9 on astro-ph0012401). Need
  `XUntil`-parameter-reader audit OR `\institute@and` boundary
  explicitly closing any open math frame. Min repro:
  `memory/project_aa_institute_xuntil_math_mode.md`.

- [x] **astro-ph0107583** (Cluster E, R=2 → R=0) — **FIXED**: extended
  the Perl-faithful `T_ALIGN` self-deactivation guard to the
  `Stored::Constructor` branch in `stomach.rs:invoke_token` (mirror
  Perl `Stomach.pm:187-189`). The `&` char-token's meaning is
  Constructor-bound (TeX_Tables.pool L49), not Token, so the
  pre-existing guard at the Token branch never fired. Now: first stray
  `&` errors once and rebinds itself to `\relax` LOCAL; subsequent
  stray `&`s no-op silently. Witness: astro-ph0107583 2 → 0.

- [ ] **physics0002038 / cond-mat0011517** (Cluster H) — Rust emits a
  follow-up `Error:unexpected:} Attempt to close a group that switched
  to mode internal_vertical` after the underlying `\@personname` /
  `\@add@frontmatter@now` mode-mismatch that Perl suppresses. P=4-6,
  R=5-7 (single extra error per paper). Fix locus: error-tracker dedup
  at `stomach.rs:273` and `:363` egroup/endgroup error sites — needs
  per-position state to remember the last error.

- [ ] **hep-th0101146** (R=17 vs P=15) — `Error:malformed:ltx:XMApp`
  + `ltx:XMTok` "isn't allowed in <ltx:p>". Source has malformed
  `$$ ... \end{equation} \begin{equation} ...` mismatch. Both engines
  fail; Rust collapses 14 `_/^` errors into 2 malformed XML errors
  while Perl emits per-position. Mostly a verbosity divergence.

- [ ] **hep-th0010165** (R=206 vs P=101) — Big cascade. Perl truncates
  at 101; Rust doesn't. Likely single root unlocking the cascade.
  Triage: identify first error, find binding gap.

- [ ] **hep-ph0007044** (R=410 vs P=101) — Same big-cascade pattern as
  hep-th0010165. Triage needed.

- [ ] **quant-ph0109041** (R=67 vs P=9) — Real-but-large delta. Triage
  needed.

- [ ] **astro-ph0204393** (R=113 vs P=101) — Borderline; small delta
  over Perl's 101 truncation cap. Triage needed.

- [ ] **hep-ph0102192** (R=4 vs P=0) — Newly discovered. Triage needed.

- [ ] **math0004140** (R=1182 vs P=?) — High-error AmS-TeX paper.
  Triage to find single cascading root.

- [ ] **hep-th0005268** (R=1000001 vs P=26) — Runaway cascade.
  Termination-condition bug; identify recursion source.

- [ ] **hep-th0005159** (R=786478 vs P=101) — Same runaway cascade
  family.

#### Out-of-scope (Perl also fails — moved to `docs/out-of-scope/`)

| Paper | Reason |
|---|---|
| `0901.2408_emph_dollar` | `$$`-in-`\emph{}` — Perl=Rust |
| `cond-mat0003169` | `Unexpected:_` cluster — Perl=Rust=2 |
| `cond-mat0106160` | `\def\r\rho` BEFORE `\documentstyle` clobber family |
| `hep_ph0001306_documentstyle_clobber` | `\def`s before `\documentstyle` — broader family |
| `math0005251_math_parser_oom` | math-parser OOM — needs grammar work |
| `math0203148_amstex_endmatrix` | AmS-TeX `\matrix\endmatrix` mode mismatch |
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
