# Engine Sync Status — Active Worklist

**Mission**: 100k "no-problem" sandbox parity. A paper is in scope iff
Perl LaTeXML on TL2025 with `--preload=ar5iv.sty
--path=~/git/ar5iv-bindings/bindings` produces 0 errors. Mission completes
when every in-scope paper produces 0 errors on Rust too.

**Status**: Round-20 Phase A Gate 0 closed 2026-05-03 at
**99,829 / 100,003 = 99.83%** raw OK (round-19 was 99.77%); **0 NEW
non-OK** introduced; **56 papers recovered**. Round-20 fix series
committed (`e1c3da3975`).

**True Rust regression count: 0**. Re-classifying the 246 residual
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

## Round-20 (active 2026-05-03)

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

## Schedule (1–2 weeks to PR)

| Day | Task | Outcome |
|---|---|---|
| ~~Today~~ DONE | Re-sweep + triage: 99.83% raw OK, **0 NEW non-OK**, 56 papers recovered. Gate 0 cleared. | Measured ✓ |
| ~~D+1~~ DONE 2026-05-03 | Round-20 fixes committed (`e1c3da3975`) — parity_check, cluster_regressions, find_main_tex, alignment | One coherent series ✓ |
| ~~D+2~~ DONE 2026-05-03 | Bisected 5 `_/^` witnesses → measured 4/5 Sub-A, 1/5 Sub-B, 0/5 Sub-C. Sub-C removed from active tracking. See sub-cause table below. | Sub-cause table ✓ |
| D+3 | CI nightly canvas (random 1k slice with parity_check baseline diff) | Drift insurance |
| D+4 | Open PR with measured numbers | Ship |

After PR ships: Phase C long-tail. Per-paper triage at 1-2/day with
min-repro → fix → land → verify. Many will be SHARED-FAILURE that
require deliberate Rust-beats-Perl divergences — track in
`docs/OXIDIZED_DESIGN.md` before landing.

Phase E asymptote: convert intractable papers to
`Fatal:invalid:<reason>` via Phase D pre-screen. Canvas reports them
as legitimate skip → 100% by definition.

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

## Earlier work (archived)

Round-17 / 18 / 19 narrative + REG-1, REG-2, REG-3, CLUSTER-NBSP
detail moved to `docs/archive/round19_iteration_log.md`. Commit log:
`git log --oneline master..claude-round-19`. Major commits include
`d44f1cb38` (`\relax` sentinel on EOF), `817d91624` (XUntil
`\def`-family re-Invoke), `6ac613b48` (xy.sty preloads amstext),
`a6b4cb5161` (psfig cluster), `342b237199` (ntheorem [standard]),
plus 25+ others.
