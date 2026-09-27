# `blkarray` `\halign`-in-math OOM/timeout — ✅ FIXED via binding (2026-07-18)

> **Status: FIXED (binding).** The *blkarray* variant is resolved: a Rust
> `blkarray` binding (`latexml_package/src/package/blkarray_sty.rs`) now shadows
> the raw `.sty`, so the pathological `\halign`-in-math is never digested.
> 1811.10792 (#594) went **OOM → 0 errors**; 2310.17416 (#473) **OOM → 9**
> (residual is a separate math-mode issue, not blkarray). Guard:
> `latexml_oxide/tests/graphics/blkarray.{tex,xml}`.
>
> **UPDATE 2026-07-20 — the "underlying `stomach.rs::egroup` bug" framing below is
> RETRACTED.** The `kbordermatrix` sibling is now FIXED, and *not* by touching
> `egroup`: its root was an inherited kernel `\@arraycr` that Perl never had
> ([`../kbordermatrix_halign_math/`](../kbordermatrix_halign_math/README.md)).
> Whether any `egroup` frame-accounting defect is reachable at all is therefore
> **unproven** — do not treat it as a known open bug. blkarray is shadowed by a
> binding either way. NOTE: `blkarray_min.tex` below no longer reproduces (the
> binding wins over `--includestyles`); to exercise the core bug use the
> kbordermatrix reproducer. This file is kept as the record of the analysis and
> the binding's faithfulness simplification.
>
> Original diagnosis (pre-binding) follows.

Witnesses: **arXiv:1811.10792** (ar5iv #594) and **arXiv:2310.17416** (ar5iv
#473), surfaced in the 2026-07-18 ar5iv mini-sprint diagnostic sweep. Both were
classified RUST-WORSE ("timeout"); the true failure mode is a fast **OOM**
(memory Fatal at ~12 s), not a wall-clock hang.

## TL;DR

`\begin{block}{(cc)}` (a `block` with a **paren-delimited** column spec) nested
inside `\begin{blockarray}` in display math makes raw-loaded `blkarray.sty`
spin its `\halign` machinery. Rust cascades into a runaway that grows the box
list to the 4500 MB memory cap → `Fatal:Timeout:MemoryBudget` (~12 s). This was
*believed* to be the same alignment × inline-math frame divergence as
`\kbordermatrix` — that turned out to be wrong for the sibling (see the update
above), so the two share a symptom, not a proven root.

## The one new fact vs the kbordermatrix sibling

For `\kbordermatrix`, **Perl completes** (0.4 s) and only Rust loops — a clean
GENUINE-RUST-ONLY. For **blkarray, Perl ALSO fails**: same-host Perl (with
`--includestyles`) runs **~90 s then rc=124 (terminated)**. So blkarray is a
*shared* catastrophic failure of both LaTeXML engines on the raw `.sty`, where
Rust's failure mode (OOM in 12 s) merely differs from Perl's (hang in 90 s).
**pdflatex renders the same input cleanly** — a parenthesised 2×2 matrix — so
the golden LaTeX behaviour is well-defined; both LaTeXML engines are wrong.

## Reduction map (what triggers, what does NOT) — 2026-07-18

All Rust runs under `--includestyles` (or ar5iv.sty preload) with `--timeout`;
`blkarray.sty` is in TeX Live so nothing is bundled.

| Construct | Rust | Note |
|---|---|---|
| `block{(cc)}` in `blockarray` in `\[..\]` (**blkarray_min.tex**) | **OOM 4.5 GB / 12 s** | ★ the minimal repro (this dir) |
| same but `block{cc}` (no `(`/`)` delimiter) | clean 0.2 s | the **delimiter** is required |
| `block{(cc)}` **without** the `blockarray` wrapper | clean 0.2 s | the **blockarray wrapper** is required |
| bare `blockarray{cc}` one row, no `block` | clean 0.2 s | `block` is required |
| load blkarray, no usage | clean 0.2 s | not the package load |

So the minimal trigger is precisely **a paren-delimited `block` nested in a
`blockarray`** — much smaller than the kbordermatrix repro (whose reduction
below the whole macro "was not achieved").

## The safe sidestep (a binding), and why it is not trivial

Because the OOM comes entirely from *raw-loading* `blkarray.sty` (without the
raw load the envs are merely "undefined", 0.2 s), a proper LaTeXML **binding**
for `blkarray` — one that shadows the raw `.sty` — would sidestep the deep
`egroup` bug and match pdflatex, surpassing Perl (which also has no binding and
hangs). Feasibility probe (2026-07-18):

- `\newenvironment{blockarray}[1]{\begin{array}{#1}}{\end{array}}` alone (no
  `block`) → **clean, correct `<ltx:XMArray>`** with all cells. So the outer
  environment maps trivially to `array`.
- The hard part is **`block`**: in blkarray it is *not* a nested array but a
  continuation of the same alignment whose `(`/`[` spec adds delimiters spanning
  a group of data rows. Making `block` transparent breaks the alignment
  grouping (`\begin`/`\end` inject a `\begingroup`/`\endgroup` that crosses `\\`
  row boundaries → `\lx@begin@alignment Attempt to close boxing group`), and
  mid-array `\left(`…`\right)` around a row group is not expressible in
  LaTeXML's `array`. A faithful binding must model the block delimiters as
  array-spanning delimiters — non-trivial, hence the HIGH-DIFFICULTY marking.

Entry points to resume: the raw `blkarray.sty` `\halign` machinery itself, and a
dedicated blkarray binding modelling `blockarray`/`block`/`\BlockArray*` on
LaTeXML's alignment constructs (the route actually taken). The old pointer at
`stomach.rs::egroup` is withdrawn — see the update at the top; it was inferred
from the kbordermatrix sibling, whose real root was elsewhere.

## Cross-references

- [`../kbordermatrix_halign_math/README.md`](../kbordermatrix_halign_math/README.md) — the sibling witness + full root-cause analysis.
- [`../../performance/STABILITY_WITNESSES.md`](../../performance/STABILITY_WITNESSES.md) — Cluster H (`\lx@begin@alignment` digest-runaways).
- [`../../AR5IV_DIAGNOSTICS.md`](../../AR5IV_DIAGNOSTICS.md) — the ar5iv mini-sprint sweep that surfaced these two witnesses.
- Full-arXiv `\lx@begin@alignment` ~12.1k-fatal cluster (memory `full-arxiv-corpus-reference-2026-06-30`).
