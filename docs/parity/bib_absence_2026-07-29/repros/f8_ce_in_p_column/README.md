# F8 — `\ce` inside a `p{}` column truncates the document

7 papers in the residual, all led by
`Error:unexpected:\@end@tabular Attempt to close a group that switched to mode
internal_vertical` (or `… mode horizontal`). The tabular's group cannot close,
and the document is truncated from that point — bibliography included.

`ce_in_p_column.tex` is the reproducer, 7 lines. pdflatex renders it and raises
nothing.

## What decides it

Reduced from **2605.12186** (an RSC/PCCP template whose abstract sits in a
`\begin{tabular}{m{4.5cm} p{13.5cm}}` inside `\twocolumn[...]`, with `\ce{BN}`
throughout). Bisecting its 66-line preamble one `\usepackage` at a time pinned
it to line 9, `\usepackage[version=3]{mhchem}`:

| cell content | column | result |
|---|---|---|
| `\ce{BN} y` | `p{5cm}` | **error, tail lost** |
| `\ce{H2O} y` | `p{5cm}` | **error, tail lost** |
| `\ce{BN} y` | `l` | clean |
| plain text | `p{5cm}` | clean |
| `$x$ y` | `p{5cm}` | clean |
| `\ensuremath{x} y` | `p{5cm}` | clean |
| `\leavevmode x y`, `\mbox{x} y` | `p{5cm}` | clean |

So it is `\ce` specifically, and only inside a paragraph column — the `p{}`
column opens `internal_vertical` (`\lx@tabular@p@`) and `\ce` leaves a
`horizontal` frame inside it.

**Not fixable by wrapping the call site:** `\mbox{\ce{BN}}` and
`\ensuremath{\ce{BN}}` both still fail, so the leak is inside `\ce`'s own
expansion, not the surrounding mode.

## Where to look

`mhchem` is raw-loaded from TeX Live (`latexml_contrib/src/mhchem_sty.rs` is a
one-line `InputDefinitions(noltxml)`, matching Perl, which ships no
`mhchem.sty.ltxml` either). That binding's own header already records a sibling
residual — `\ce` inside an amsmath `align*` emitting
`\lx@begin@alignment`/`\lx@end@inline@math` — so this is the same family one
context over, and a fix should address both. `LXML_TRACE_BOUND_MODE=1` is the
tool: it prints every `begin_mode`/`set_mode` with its bound mode, which is how
the `$$`-in-`.bib` cascade (family F1) was pinned.

Perl raw-loads the same package, so this is very likely a shared limit rather
than a Rust-only regression — worth confirming with same-host Perl before
deciding how far to surpass.
