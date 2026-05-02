# Out of scope (moved from SYNC_STATUS.md 2026-05-01)

Empirically verified: Perl LaTeXML on TL2025 with --preload=ar5iv.sty --path=~/git/ar5iv-bindings/bindings does NOT produce 0 errors on this paper, so it fails the in-scope predicate ("in scope iff Perl produces 0 errors").

Original SYNC_STATUS.md task content preserved below for future reference.

### 0. math0606553 — `\CompileMatrices` + `\xy@@ix@` re-tokenization

Single-error paper (`undefined:\lx`); affects every paper using
`\usepackage{xy}` + `\CompileMatrices` whose matrix cells contain
`\lx@*` CSes (e.g. `\DeclareMathOperator` operators that expand into
`\lx@dual …`). Min repro at
`latexml_oxide/tests/graphics/xycompile.tex` (no `.xml` pair yet).

Root cause: xy.tex compile mode writes a `.xyc` file, then `\input`s
it back. `\xyxy@@ix@` body sets `@`=OTHER **before** `\toks9 =
{body}` reads, so `\lx@dual` re-tokenizes as `\lx`+`@dual`. Stored
in `\toks9`, later `\the\toks9` fires `\lx` undefined.

**New angle (2026-04-30 evening):** xy.tex L38-46 defines
`\xyreuncatcodes` to `\edef \xyuncatcodes {... \catcode64 \the\catcode 64
...}` — i.e. `\xyuncatcodes` is a *snapshot* of the current
catcode numbers, baked in at definition time, NOT a hard-coded
reset to OTHER. After lines 47-112's flow (initial `\xyreuncatcodes`
captures pre-state, then `\xycatcodes` sets `@`=LETTER, then
`\xyresetcatcodes` re-snapshots so the new `\xyuncatcodes`
contains `\catcode64 11`), `\xyuncatcodes` actually *preserves*
`@`=LETTER. SYNC_STATUS's earlier "sets `@` to OTHER" claim was
a misreading. The real question is whether our `\edef` correctly
evaluates `\the\catcode 64` at definition time so the snapshot
bakes in the right number — if not, `\xyuncatcodes` re-evaluates
at use time to whatever catcode is current, causing the cell-body
re-tokenization mismatch. **Next:** instrument
`\meaning\xyuncatcodes` after each capture point and confirm the
literal numbers in the body. Hypotheses 1-4 (Perl `\toks` reading
catcode snapshot, `UnTeX` write semantics, `xylatexml.tex.ltxml`
rerouting, one-step `remove_value` frame-collapse) remain open
secondary candidates. Empirical band-aid (NOT applied): force
`at_letter:true` on `.xyc` paths.
Acceptance: min repro + math0606553.zip → 0 errs, tests 1110+/0/0.

