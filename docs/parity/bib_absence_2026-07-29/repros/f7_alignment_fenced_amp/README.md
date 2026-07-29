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

Fixing it means changing when alignment rows are split relative to expansion —
wide blast radius across every `array`/`align`/`tabular`, and it needs its own
branch plus a full-corpus measurement. Deliberately left out of the
bibliography-absence PR.

## Bisect notes

Cut only at **safe** points — blank lines where every `\begin`/`\beq` is
balanced. Truncating anywhere else manufactures its own alignment error and the
signal goes non-monotonic (through=520 errored while 550 and 600 were clean).
And keep the preamble's `\def\beq{\begin{eqnarray}}` in every slice: dropping
it leaves `\beq` undefined, the equation never runs, and the result reads as a
false "clean".
