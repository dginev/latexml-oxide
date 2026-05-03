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
Final classification after branch `claude-round-19`:

| Verdict | Count | Notes |
|--------|------:|-------|
| BOTH CLEAN | many | both Rust and Perl now produce 0 errors |
| OUT-OF-SCOPE | many | Perl=Rust both >0; not Rust-only regressions |
| PERL_REGRESSION | many | Rust beats Perl (40+ across full sweep) |
| **REAL_REGRESSION** | **2** | REG-3 (0909.5169) fixed; REG-1 + REG-2 remain |

**Latest random-sample validation (150 papers, 100 ok + 50 failed)**:
108 BOTH CLEAN, 0 REAL_REGRESSION, 12 Rust-beats-Perl wins, no
regressions on previously-clean papers.

`cargo test --tests` 1124/0/0.

**REG-3 root cause (fixed)**: `digest_alignment_column`'s OUTER loop
in `latexml_engine/src/tex_tables.rs` did NOT reset `last_token`
across iterations. Perl's `digestAlignmentColumn`
(`LaTeXML/blib/lib/LaTeXML/Engine/TeX_Tables.pool.ltxml:367-396`)
sets `$token` per `readXToken(0)` call, so when the gullet returns
undef (mouth exhausted, e.g. mid-cell `\input` finishes) the
`if (!$token)` check terminates the column. In Rust the
`while let Some(xtoken) = read()` body only updates `last_token`
on `Some`; on the OUTER iter after INNER 1 / INNER 2 exhausted
the mouth, `last_token` still pointed at the previous content
token, the `last_token.is_none() || last_is_end` check skipped,
and the column re-fed `(column_before, marker, last_token)` into
an empty gullet — re-invoking `\begin{picture}` infinitely.
Fix: reset `last_token = None` at the start of each OUTER iter.
One-line addition + 9-line comment, see
`tex_tables.rs:711` (commit on top of round-19).

---

## Open work — the 2 remaining REAL_REGRESSIONs

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

**Bisected min repro** (verified on this branch, 2026-05-02):

```latex
\documentclass{article}\begin{document}
\begin{array}{c}\input test.pstex_t\end{array}
\end{document}
```
with `test.pstex_t` containing just `\begin{picture}(0,0)\end{picture}\n`.

* Perl: **0 errors** (clean).
* Rust: **501 errors** cascade starting with `Error:unexpected:&
  Extra alignment tab '&'` followed by `Error:expected:Pair Missing
  argument Pair for Constructor[\begin{picture}…]`.

Same content INLINE (no `\input`) is clean in Rust too. Same
content in `\input` but OUTSIDE `\begin{array}` is clean. So the
trigger is **`\begin{array}` + `\input` together**.

**Refined diagnosis** (further deep-dive 2026-05-02):

Backtrace from instrumented `picture.before_digest`:
```
0: closure at latex_constructs.rs:8443
1: Constructor::execute_before_digest
2: Constructor::invoke_primitive
3: stomach::invoke_token
4: tex_tables::digest_alignment_column at tex_tables.rs:809
5: tex_tables::digest_alignment_body at tex_tables.rs:631
6: closure for \@end@array's after_digest at tex_tables.rs:41
```

**The bug is in Rust's `\halign`/array cell re-digestion.** When
`\@end@array` fires its `after_digest`, it calls
`digest_alignment_body` which loops calling `digest_alignment_column`.
That re-digests cell tokens. Each re-digestion of `\begin{picture}`
creates a fresh picture invocation. The first iteration consumes
`(0,0)` correctly via Pair, but subsequent iterations (which
shouldn't be happening for a single-cell single-row array) keep
re-invoking picture, each time finding `\end{picture}` next and
firing "Missing argument Pair", cascading to 10001 errors.

**Why `\input` matters**: When the picture content is INLINE
(`\begin{array}{c}\begin{picture}(0,0)\end{picture}\end{array}`),
the env-machinery captures picture's body BEFORE array
sees the cell. Picture's body is closed before array starts cell
walking. Single, clean invocation.

When the content comes via `\input`, the file's tokens are pushed
to the gullet and read DURING array's cell digestion. Picture's
env-handler runs from inside `digest_alignment_column` (instead of
outside). Body capture inside this nested context doesn't bound
the alignment loop's iteration — picture's body tokens stay in the
input stream and get re-walked.

(An earlier candidate — make Pair use expanding peek
[`read_x_token` + unread] — was rejected: it's a divergence from
Perl's `ifNext`, and only reduced the cascade to 501 errors without
fixing the root cause.)

**Plan of attack:**

1. **Inspect `\input` token-streaming** —
   `latexml_engine/src/tex_file_io.rs:191`. When `\input` reads a
   file, does it preserve the surrounding alignment context? In
   particular, are the token-stream's pushback / catcode / align-state
   markers correctly maintained across the file-IO boundary? Compare
   with Perl's `\input` (TeX_FileIO.pool.ltxml).
2. **Inspect `\begin{array}` cell-template setup** — find where
   the `c`-column expands to `\hfil $\displaystyle ## $ \hfil &`
   (or equivalent) and ensure the trailing `&` is the `\halign`-machinery
   separator, not a user-visible gullet token.
3. **Trace the actual token sequence** — temporarily add `eprintln!`
   before `Pair`'s `ifNext` call to print the next 5 tokens. This
   confirms whether `&` is sitting between `\begin{picture}` and
   `(` (root cause is upstream) or whether `(` is consumed by Pair
   itself (root cause is in Pair / read_token).
4. **Diff with Perl trace** — `latexml --debug=tokens` on the
   same min repro and compare the token sequence at the picture-env
   boundary.
5. **Fix candidates (in order of root-cause-fidelity)**:
   * a. **Fix the `&` leak** — most likely a `\halign` /
        `\begin{array}` template setup divergence. The fix should
        match Perl's column-template emission so cell-separator
        tokens stay inside the alignment machinery, not visible to
        the gullet's `read_token`.
   * b. **Cap error cascade at first `Pair` failure** (pragmatic
        bound): when picture's required `Pair` reader returns
        `ArgWrap::None`, instead of erroring + cascading, gobble
        balanced text up to `\end{picture}` via XUntil and
        silently emit an empty `<ltx:picture>`. This bounds the
        damage at 1 error (or 0 if we Warn instead of Error).
        Acceptable as a defense-in-depth even after fix (a) lands.
6. **Verify**: 0909.5169 R=10001 → R=0 (== Perl) BOTH CLEAN under
   fix (a). Sweep round-19 + 100-paper random ok-status sample to
   confirm no regressions in any Pair-using construct (picture,
   pstricks, qbezier, etc.).

**Difficulty**: Hard — root cause is in `\halign` / `\input`
machinery, not the easy "Pair reader" surface. Fix (b) is a
low-risk fallback if (a) requires more iteration than available.

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
