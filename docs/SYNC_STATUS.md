# Engine Sync Status — Task List

**Mission (active 2026-04-30):** 100k "no-problem" sandbox parity.
Phase 2 canvas at `/home/deyan/data/100k_noproblem_sandbox/`. Every
paper there is a Perl LaTeXML "no problem" conversion (zero errors,
zero warnings on TL2025 + ar5iv preset). Mission completes when
`latexml_oxide` matches that 100% clean rate paper-for-paper.

A sandbox paper is **in scope** iff Perl LaTeXML on TL2025 with
`--preload=ar5iv.sty --path=~/git/ar5iv-bindings/bindings` produces
0 errors on it. Mission completes when every in-scope paper also
produces 0 errors on Rust.

**Phase 1 (DONE):** 7898-paper canvas; 7731 OK = 97.89% (PR #220,
commit `71b0a3e82`).

Earlier per-iteration narrative: `docs/archive/`. Tactical insights:
`docs/WISDOM.md`. Upstream Perl bugs: `docs/KNOWN_PERL_ERRORS.md`.
Intentional divergences: `docs/OXIDIZED_DESIGN.md`. Branch fix list:
`git log master..claude-round-19`.

---

## Current state (2026-05-02 evening)

**Round-19 sandbox parity sweep**: 305 canvas-failed papers triaged.
Final classification after branch `claude-round-19` (30 commits):

| Verdict | Count | Notes |
|--------|------:|-------|
| BOTH CLEAN | many | both Rust and Perl now produce 0 errors |
| OUT-OF-SCOPE | many | Perl=Rust both >0; not Rust-only regressions |
| PERL_REGRESSION | many | Rust beats Perl (40+ across full sweep) |
| **REAL_REGRESSION** | **3** | true Rust-only gaps — see "Open work" below |

**Latest random-sample validation (150 papers, 100 ok + 50 failed)**:
108 BOTH CLEAN, 0 REAL_REGRESSION, 12 Rust-beats-Perl wins, no
regressions on previously-clean papers.

`cargo test --tests` 1124/0/0.

---

## Open work — the 3 remaining REAL_REGRESSIONs

Each requires dedicated multi-iteration architectural work. Bisections
and root causes are recorded; the steps below are the proposed line
of attack.

### REG-1: math0403005 — `\vtop` mode-frame leak (R=29, P=27, gap=2)

**Witness paper**: `/home/deyan/data/100k_noproblem_sandbox/arxmliv/0403/math0403005/math0403005.zip`
(file `scs8-for-arxiv.tex`, around lines 570-600).

**Symptom**: Both Rust and Perl error on the same `\noalign` /
`\end{center}` / `\end{table}` cascade caused by user-malformed
TeX inside `\vtop{...}`. But Rust emits **3 extra `\vtop`-labeled
frame errors** that Perl doesn't:

```
Error:unexpected:\@end@array Attempt to close … due to T_CS[\vtop]
Error:unexpected:\endgroup Attempt to close … due to T_CS[\vtop]
Error:unexpected:\lx@begin@alignment Attempt to close … due to T_CS[\vtop]
```

These are **recovery-cascade noise** — same root cause as Perl's 1
visible error, just multiplied by Rust's stricter mode-frame
discipline.

**Plan of attack:**

1. **Reproduce in min repro** — bisect scs8-for-arxiv.tex around the
   error site (lines 570-580 in the source) to find the smallest
   `\vtop{ … malformed … }` snippet that fires the same cascade.
2. **Instrument `\vtop`'s digestion** — temporarily add `eprintln!`
   in `latexml_engine/src/tex_box.rs:747` (`\vtop` DefConstructor
   `before_digest`/`after_digest`) and at `push_stack_frame`/
   `pop_stack_frame` to count frames opened/popped per `\vtop`
   invocation.
3. **Compare with Perl** — run min repro through `latexml --debug=stomach`
   (or equivalent verbose flag) and diff the frame-pop sequence.
4. **Identify the redundant Error firings** — likely `egroup` is
   raising `Error!("unexpected", ...)` 3× when the same broken state
   should be raised once and then suppressed/recovered until next
   well-formed token.
5. **Fix candidates (ordered by safety)**:
   * a. **Once-per-cascade gate**: in `latexml_core/src/stomach.rs`'s
        `egroup` (line 273) and `\@end@array` / `\lx@begin@alignment`
        primitives, debounce repeated mode-frame errors with the same
        `groupInitiator` token (only emit the first within a single
        digest call).
   * b. **Frame stripping on first error**: when egroup hits a
        mode-switch frame and errors, also pop the frame so subsequent
        egroups see a fresh state.
   * c. **Match Perl's swallow-and-recover semantics**: Perl emits 1
        error here and continues; Rust should match. The Perl path
        likely lives in `Stomach.pm` `egroup`/`endMode` — port the
        recovery branch verbatim.
6. **Verify**: math0403005 R=29 → R=27 (== Perl) reclassifies to
   OUT-OF-SCOPE. Sweep round-19 to confirm no regressions on the
   other 38 papers.

**Difficulty**: Medium — recovery-cascade noise rather than a hard
correctness bug. Low-risk because we're matching Perl, not changing
the digestion semantics.

---

### REG-2: math-ph0501074 — `\lefteqn{pmatrix}` in `\begin{align}` (R=15, P=0)

**Witness paper**: `/home/deyan/data/100k_noproblem_sandbox/arxmliv/0501/math-ph0501074/math-ph0501074.zip`
(file `Claeys-Kuijlaars.tex`, line 1832).

**Bisected min repro** (memory: `wisdom_lefteqn_pmatrix_align_leak.md`):

```latex
\documentclass{article}
\usepackage{amsmath}
\begin{document}
\begin{align} \nonumber
    \lefteqn{
    \begin{pmatrix} 1 & 2 \end{pmatrix} A^{-1}(y)} \\
    & = B
\end{align}
\end{document}
```

Triggers 10 errors; without `pmatrix` (e.g. `\lefteqn{\begin{array}
{c}1\end{array}}`) it's clean. So **pmatrix specifically** inside
`\lefteqn` inside `\begin{align}` leaks the `\lx@begin@inline@math`
mode-switch frame.

**Plan of attack:**

1. **Inspect Rust's pmatrix expansion** — `\pmatrix` in
   `latexml_package/src/package/amsmath_sty.rs:287` expands to
   `\lx@ams@matrix{name=pmatrix,…,left=\lx@left(,right=\lx@right)}`.
   Trace `\lx@ams@matrix` and see if/when it pushes/pops alignment
   frames inside an existing math context.
2. **Inspect Rust's `\lefteqn`** —
   `latexml_engine/src/latex_constructs.rs:5396` matches Perl exactly:
   `\rlap{\lx@begin@inline@math\displaystyle #1\lx@end@inline@math}`.
   The bug is NOT in `\lefteqn` itself.
3. **Inspect `\rlap`** — find the Rust binding (likely in
   `tex_box.rs` or similar) and verify it doesn't open a
   stack-frame that pmatrix's `\\` row-sep would close in the
   wrong order.
4. **Trace with `LXML_TRACE_BOUND_MODE=1`** on the min repro to see
   the exact sequence of begin_mode_opt calls and which `}` finally
   raises egroup.
5. **Compare to Perl** — Perl handles this paper cleanly. Diff
   how Perl's `\lx@ams@matrix` (in `LaTeXML/Engine/amsmath.sty.ltxml`
   or wherever) handles `\\` and column boundaries when the
   surrounding context is `\rlap{\lx@begin@inline@math …
   \lx@end@inline@math}`.
6. **Fix candidates**:
   * a. **Wrap `\lefteqn` body in extra group**: change Rust's
        `\lefteqn{#1}` expansion to wrap `#1` in `{ }` so pmatrix's
        `\\` is scoped to that group, preventing escape to the
        outer align row-sep. Test that this doesn't visually change
        the output.
   * b. **Make pmatrix's `\\` Let-bind only inside its own
        environment** — verify `\lx@ams@matrix`'s setup re-Lets
        `\\` to `\lx@alignment@newline` in the right scope and pops
        it on `\end{pmatrix}`.
   * c. **Diagnose mode-frame leak**: the trace will reveal which
        `\lx@begin@inline@math` frame isn't getting popped — fix
        the symmetry there.
7. **Verify**: math-ph0501074 R=15 → R=0 (== Perl) BOTH CLEAN.
   Run full test suite + round-19 sweep for regressions.

**Difficulty**: Hard — requires understanding the pmatrix /
inline-math / align frame interaction. Each component is correct in
isolation; the bug is in the cross-component composition.

---

### REG-3: 0909.5169 — pstex_t `\put(0,0)` Pair-param parse failure (R=10001, P=0)

**Witness paper**: `/home/deyan/data/100k_noproblem_sandbox/arxmliv/0909/0909.5169/0909.5169.zip`
(file `v-Dims.tex`, line 28: `\def\pstex#1{\begin{array}{c}\input
figs/#1.pstex_t \end{array}}`).

**Symptom**: A pstex_t file (`\input figs/RMoves.pstex_t` inside
math-mode `\begin{array}`) starts with `\begin{picture}(0,0)%` —
standard PiCTeX/pstex output. Rust's `{picture}` env signature is
`{picture} Pair OptionalPair` (Pair = `(x,y)`). Inside math-mode
array context, the Pair reader fails to consume `(0,0)`, fires
`Error:expected:Pair Missing argument Pair`, then mode-frame
cascade explodes to 10001 errors (Rust's MAX_ERRORS cap).

**Plan of attack:**

1. **Min repro** — verify that putting `\begin{array}{c}\begin{picture}
   (0,0)\end{picture}\end{array}` (no `\input`) inside `\[ ... \]`
   reproduces the `Pair` failure. Then add `\input` to confirm it's
   not file-IO related.
2. **Inspect `Pair` parameter type** — `latexml_engine/src/base_parameter_types.rs:193`
   reads via `gullet::if_next(T_OTHER!("("))`. In math mode, what
   catcode does `(` carry? It should be OTHER (math doesn't change
   catcodes), but verify with a debug print.
3. **Compare to Perl** — Perl handles this paper cleanly with 0
   errors. Run Perl's `latexml --debug=tokens` on the min repro to
   see how `(` reaches the picture's `Pair` reader.
4. **Likely root cause hypotheses**:
   * a. **Math-mode `\input` re-tokenizes** — when a file is `\input`'d
        inside math mode, perhaps catcodes differ. Check
        `latexml_core/src/binding/content.rs` `\input` impl.
   * b. **Array's column template emits a token before picture** —
        in math array, each cell may have `\hfil $\displaystyle` or
        similar prepended that happens BEFORE picture's parameters
        are read, eating something.
   * c. **`Pair`'s `if_next` doesn't expand** — perhaps an expandable
        token sits before `(` and Pair's peek doesn't expand it.
        Switch to `if_next_x` (expanding peek) and re-test.
5. **Fix candidates**:
   * a. **Make Pair's open-paren peek expand expandable tokens**:
        `gullet::if_next_x(T_OTHER!("("))` instead of `if_next`.
   * b. **Stub pstex_t `\begin{picture}` in math mode**: if the
        body is just `\includegraphics`, the picture env content
        adds no value in math mode — make it a Math-aware no-op
        that gobbles content via XUntil to `\end{picture}`.
   * c. **Cap the error cascade at first `Pair` failure**: have
        the picture env's failure path be a hard recovery — gobble
        balanced text up to `\end{picture}` and skip silently.
        This wouldn't fully fix the paper but would prevent the
        10000-error explosion (R=10001 → R=1 OUT-OF-SCOPE if Perl
        also has 1).
6. **Verify**: 0909.5169 R=10001 → R≤1. Even cap-only fix is a
   massive improvement (drops from "blows up MAX_ERRORS" to
   "1 user-attributable error").

**Difficulty**: Medium-Hard — likely tractable via fix-candidate (a)
[expanding `if_next`] but needs careful regression testing because
many other constructs use `Pair` (picture, pstricks, qbezier, etc.).

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
  where Perl errors): `1207.6068`, `0909.3444`, plus 40+ papers
  identified in round-19 sweep (memory:
  `project_rust_supersedes_perl.md`).
* **Unported pools:** `BibTeX.pool.ltxml` (skipped via `--nobibtex`).

---

## Acceptance gates

| Gate | Current | Target |
|---|---|---|
| `cargo test --tests` | 1124/0/0 | unchanged across all task work |
| `latexml_oxide --init=plain.tex` | 0 errors | 0 errors |
| `latexml_oxide --init=latex.ltx` | 0 errors | 0 errors |
| 10k canvas (Phase 1, complete) | 7731 / 7898 = 97.89% | n/a (canvas retired) |
| 100k "no-problem" canvas (Phase 2, active) | downloaded — sweep pending | 100% match Perl |
| Round-19 305-paper triage | 3 REAL_REGRESSION | 0 REAL_REGRESSION |

---

## Distribution follow-up

Once TL2025 dumps stay robust through a CI cycle: `include_bytes!`
`{plain,latex}.dump.txt` for TL2022 … TL2026 and select at runtime
by `kpsewhich --version`. Currently dumps load from
`resources/dumps/` on disk.

---

## Earlier work (archived)

Pre-round-19 fix history (Round-17 squashed master, plus rounds 18
and 19 in branch `claude-round-19`) is preserved in `git log` and
`docs/archive/`. Major commits in `claude-round-19` (30 total):

* `d44f1cb38` Token/XToken `\relax` sentinel on EOF (cleared 3 papers
  via cascade-removal: 0910.2125, hep-th0302065, gr-qc0304029)
* `817d91624` XUntil re-Invoke `\def`-family primitives (cleared
  0805.1712, cs0502037)
* `6ac613b48` xy.sty pre-loads amstext for in-math `\text`
  (math0211451)
* `799abc9e2` `\@trivlist → \relax` (Perl L1732, 0802.2207)
* `1e34cdd27` multirow brace-presence gate (3 papers)
* `a6b4cb5161` Pre-flight `\documentstyle` checks not eagerly load
  (12-paper psfig cluster)
* `5c1ec07da` IEEEtran `\if@twocolumn=true` journal default
* `0155b56c1` IEEEtran private `\if@technote`/`\if@confmode`
* `342b237199` ntheorem [standard] option triggers std raw load
* …plus 21 smaller package-binding bridges and stubs.

See `git log --oneline master..claude-round-19` for the full list.
