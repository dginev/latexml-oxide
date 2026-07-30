# F7 — a fenced macro argument containing `&` leaks into the outer alignment

Largest remaining bucket of the bibliography-absence residual: **28 papers**
whose first error is
`Error:unexpected:\lx@begin@alignment Attempt to close a group that switched to
mode restricted_horizontal` (a few say `mode math`). The document is truncated
at that point, so the bibliography goes with it.

`mqty_in_eqnarray.tex` is the reproducer (14 lines). pdflatex renders it and
raises nothing; latexml-oxide loses everything after the equation.

## What decides it

Witness **2605.05903** reduced to its preamble plus one equation. `\mqty` is
the physics package's matrix macro, taking a *delimiter-fenced* argument:

| body inside `\beq …\eeq` | errors | tail survives |
|---|---|---|
| `S = \mqty( a &0 \\ 0 &b )` | 6 | no |
| `S = \mqty{ a &0 \\ 0 &b }` — braces | 0 | yes |
| `S = \mqty( a \\ b )` — no `&` | 0 | yes |
| `S &=& \mqty( a &0 \\ 0 &b )` | 0 | yes |
| `S = \pmqty( a &0 \\ 0 &b )` | 0 | yes |

So it needs all three of: the **fenced** `(…)` form, at least one **`&`** inside
it, and an enclosing alignment row that has **no `&` of its own**. The brace
form is safe because alignment cell scanning skips balanced groups — `(…)` is
not a group.

## Why this is not a physics-binding fix

`physics_sty.rs` already documents this exact hazard and its remedy: make
`\lx@physics@mat` expandable (a `DefMacro`, not a `DefPrimitive`) so it grabs
`(…)` before the alignment sees the inner `&`/`\\` — witness 2007.06211. That
remedy is in place, and the leak still happens.

A plain user macro with a delimited argument leaks identically:

```tex
\def\myfence(#1){\left(\begin{array}{cc}#1\end{array}\right)}
\beq  S = \myfence( a &0 \\ 0 &b )  \eeq
```

So the cell split is happening on the **unexpanded** `&`, before macro
expansion gets a chance to consume it — the divergence is in the alignment
machinery, not in `\mqty`. In TeX, reading a delimited macro argument does not
break an alignment cell, which is why pdflatex is fine.

## FIXED (2026-07-29) — and Perl was never the oracle here

**This write-up originally recorded our loss without running Perl.** Perl raises
the *identical* error, 11 of them on this reproducer, and loses TAILTEXT too. So
the class is a shared limitation, and `pdflatex` — which renders the reproducer
silently — is the ground truth.

`tex.web` §394 `macro_call` says what to do: `align_state:=1000000; {disable tab
marks, etc.}` while a macro's parameters are scanned. A `SuppressedTabMarks` RAII
guard (`common/local_assignments.rs`), armed **only inside an alignment** so
ordinary macro calls keep their hot path, now wraps physics.sty's
`phys_read_arg` — where `\lx@physics@mat` consumes its fenced `(…)` body.

**Partial by necessity.** Arming the same guard at
`Parameters::read_arguments` — TeX's real `macro_call` site — also cures the
plain `\myfence` form, but regresses **5 tests**: `cells_test` (17 errors),
`numprints_test`, `xytest_test`, `consort_flowchart_test`,
`unit_tests_by_silviu_test`. That path is *also* how an alignment reads its own
cell content, so suppressing tab marks across it stops cells terminating. The
original "wide blast radius" warning was correct on this point. A general fix
needs to distinguish a parameter scan from a cell-content read.

A/B with a temporary kill-switch (both sites armed):

| | guard off | guard on |
|---|---|---|
| plain delimited macro | 12 errors, tail lost | 0 errors — but costs the 5 tests |
| `\mqty` (physics.sty) | 11 errors, tail lost | **0 errors, tail survives** — shipped |

`\mqty` now yields a correct 4×4 MathML table — 4 rows, 16 cells, `b_0..b_3` on
the diagonal — and the equation keeps its number. Divergence #90, guard
`alignment_fenced_amp_does_not_split_a_row`.

## Bisect notes

Cut only at **safe** points — blank lines where every `\begin`/`\beq` is
balanced. Truncating anywhere else manufactures its own alignment error and the
signal goes non-monotonic (through=520 errored while 550 and 600 were clean).
And keep the preamble's `\def\beq{\begin{eqnarray}}` in every slice: dropping
it leaves `\beq` undefined, the equation never runs, and the result reads as a
false "clean".
