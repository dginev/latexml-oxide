# `blkarray` `\halign`-in-math OOM/timeout — OPEN, HIGH DIFFICULTY, post-release

> **Status: OPEN. Difficulty: HIGH. Priority: post-release.**
> A second, **smaller** witness of the known `\lx@begin@alignment` /
> `\halign`-in-math frame-accounting cluster — the same root as
> [`../kbordermatrix_halign_math/`](../kbordermatrix_halign_math/README.md).
> Read that sibling's README for the full root-cause analysis
> (`stomach.rs::egroup` refuses to pop a frame that "switched to mode math" at
> an alignment close, then re-enters and spins). This file records the blkarray
> variant, its **4-line minimal reproducer**, and the one new fact it adds.

Witnesses: **arXiv:1811.10792** (ar5iv #594) and **arXiv:2310.17416** (ar5iv
#473), surfaced in the 2026-07-18 ar5iv mini-sprint diagnostic sweep. Both were
classified RUST-WORSE ("timeout"); the true failure mode is a fast **OOM**
(memory Fatal at ~12 s), not a wall-clock hang.

## TL;DR

`\begin{block}{(cc)}` (a `block` with a **paren-delimited** column spec) nested
inside `\begin{blockarray}` in display math makes raw-loaded `blkarray.sty`
spin its `\halign` machinery. Rust cascades into a runaway that grows the box
list to the 4500 MB memory cap → `Fatal:Timeout:MemoryBudget` (~12 s). This is
the same alignment × inline-math frame divergence as `\kbordermatrix`.

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

Entry points to resume are the same as the kbordermatrix sibling
(`stomach.rs::egroup`, the alignment cell/`\crcr` frame accounting), **plus**
the option of a dedicated blkarray binding modelling `blockarray`/`block`/
`\BlockArray*` on LaTeXML's alignment constructs.

## Cross-references

- [`../kbordermatrix_halign_math/README.md`](../kbordermatrix_halign_math/README.md) — the sibling witness + full root-cause analysis.
- [`../../performance/STABILITY_WITNESSES.md`](../../performance/STABILITY_WITNESSES.md) — Cluster H (`\lx@begin@alignment` digest-runaways).
- [`../../AR5IV_DIAGNOSTICS.md`](../../AR5IV_DIAGNOSTICS.md) — the ar5iv mini-sprint sweep that surfaced these two witnesses.
- Full-arXiv `\lx@begin@alignment` ~12.1k-fatal cluster (memory `full-arxiv-corpus-reference-2026-06-30`).
