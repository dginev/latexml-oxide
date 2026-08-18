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

## CONFIRMED SHARED-FAILURE (2026-08-04)

Re-triaged on today's binaries. The earlier "very likely shared" is now
**confirmed**: the valid comparison needs `--includestyles` on the Perl side,
because Perl does **not** interpret a raw `.sty` without it (bare `latexml`
reports `missing file[mhchem.sty]` + `undefined:\ce`, so `\ce` is inert and the
tabular closes — a false "Perl is clean"). With `latexml --includestyles`
Perl 0.8.8 raw-loads mhchem exactly as Rust does and produces the
**byte-identical cascade**: 7 errors, `\@end@tabular` (horizontal) → `\endgroup`
→ `\lx@begin@alignment` (internal_vertical ×3) → `\@@tabular`
(restricted_horizontal), TAILTEXT lost. Same error count, same order.

Verdict: **SHARED-FAILURE, surpass-only.** Any fix (mode-robust `\ce`, or an
`mhchem.sty.ltxml`-equivalent binding handling `\ce` cleanly) is a surpass-Perl
divergence needing the three qualifying tests + user escalation — NOT a straight
parity fix. Deprioritized until a surpass decision is taken. The sibling
`\ce`-in-`align*` residual noted in `mhchem_sty.rs` is the same family and a fix
should address both.

## Root cause isolated: expl3 regex-REPLACE, not mhchem (2026-08-17)

`regex_replace_in_p_column.tex` (beside this file) strips the mystery to its
core: **`\regex_replace_all:nnN` inside a `p{}` column, with NO chem package
loaded** — expl3 is base LaTeXML, so it reproduces on the plain binary and on
**bare** Perl 0.8.8 (no `--includestyles` needed, unlike the `\ce` form). Both
engines give the byte-identical 7-error cascade. `\regex_match` (read-only) in
the same column is clean; a `c`/`l` column is clean. So the trigger is
regex-**REPLACE** in a **paragraph** column corrupting the deferred-alignment
mode/group state — a shared property of LaTeXML's alignment reimplementation,
independent of chemistry.

This also generalises the witness set: **chemformula `\ch` hits the identical
cascade** (witness **2606.04125**, `\ch{Li+}` in `tabularx` `X`/`P{5.5cm}`
columns), because `\ch` and mhchem `\ce` both route through `\regex_replace_all`.
mhchem `\ce` witnesses remain 2605.12186 and 2606.00894 (`\ce` in a `p{3cm}`
column of a `sidewaystable`).

**False-clean trap (cost a triage pass 2026-08-17):** a same-host Perl run that
*omits* `--includestyles` reports `Can't find binding for package
mhchem/chemformula` and leaves `\ce`/`\ch` **undefined** — the table then closes
cleanly and Perl looks correct while Rust looks broken, i.e. a spurious
"Rust-only" verdict. The mhchem-free `\regex_replace_all` repro sidesteps the
trap entirely and is the fastest way to re-confirm the shared nature.
