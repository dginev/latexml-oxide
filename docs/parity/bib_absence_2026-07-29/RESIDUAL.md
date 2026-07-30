# The remaining failures, fully characterized

Every paper of the 533-article known cohort that still has **zero**
`class="ltx_bibitem"` after this PR. **Refreshed after the xpatch binding.** 270 are recovered; **263** are not (258 EMPTY + 5 NOHTML).
One row per paper in `residual_characterization.tsv.gz` —
`id, first_error_class, nerrors, status, html_bytes, html_pct_of_src,
source_bib_signal, bib_machinery` — every field derived from the paper's own run
artifacts, so **no paper is unaccounted for** (one row per residual paper).

Regenerate with `characterize.sh` (beside this file) over a `bib_recheck.sh`
output directory.

> Label hygiene: extract error classes with `printf`, never `echo -e`. `\e` in
> `\endgroup`/`\noalign` becomes an ESC byte and the label silently truncates to
> "ndgroup"/"oalign" — which corrupted an earlier version of this table.

## The headline: truncation is no longer the main story

| document state | papers |
|---|---|
| looks complete (HTML ≥60% of source bytes) | **222** |
| severely truncated (<15%) | 39 |
| partial (15–60%) | 22 |
| no HTML at all | 9 |

And bibliography markup is almost never even *attempted*:

| in the HTML | papers |
|---|---|
| no bibliography markup at all | **270** |
| a References heading with an empty list | 22 |

269 of 292 produced **no `MakeBibliography` line whatsoever** — the
bibliography machinery never ran. Only 8 show the `\bibliography` call
swallowed into visible verbatim text; for the other 284 it was simply never
reached.

## Four families, all 292 assigned

**A. Complete document, tail intact, bibliography still absent — 97.**
The strongest bucket, and the one to work next: acknowledgements (which sit
immediately before the references in most templates) *do* render, so this is
not collateral truncation but a bibliography-specific failure. 18 raise **no
error at all**. By source mechanism: **29 biblatex**, 23 plain `.bib`, 17
`.bbl`, 6 `thebibliography`, 12 with no bibliography source at all. The
biblatex sub-bucket is the single largest actionable group left and matches the
audit's still-open F4(c)/(d) (a resource declared in an unbound `.cls` is never
registered; OmniBus dep-mining taking a branch the document did not).

**B. Complete document, tail already gone — 125.** A mode/group leak starts
after the last visible section and before the references, so byte count still
looks healthy. Led by `\lx@begin@alignment` (20) and 12 with no error at all.
Same mechanism as family C, later in the document.

**C. Truncated — 61** (39 severe + 22 partial). Collateral: an earlier leak
swallows the rest of the file. This is what the landed fixes drained; what is
left is dominated by the alignment cluster.

**D. No output — 9.** Fatals: `Fatal:Stomach:Recursion` 6,
`Fatal:Timeout:TokenLimit` 3, `Fatal:Stomach:MemoryBudget` 3 across the set.
Belongs to the general fatal-mining mission, not to bibliography work.

Cross-cutting, **22 papers** have a References heading and an empty list —
the machinery ran and selected nothing (`N bibentries, 0 cited`). 14 of them
ship only a `.bib`. This is the `\cite`-record family; the raw-`\cite`-clobber
half of it was fixed here (divergence #88), so what remains is a narrower
selection defect worth its own reproducer.

**17 papers ship no bibliography source at all** — no `.bbl`, no `.bib`, no
`thebibliography`, no biblatex resource. They are candidates for exclusion as
faithful-to-broken-source, but **verify each against its own PDF first**: see
the corrected rule in `repros/README_residual_triage.md`, because we read
`.bib` directly and can legitimately have a bibliography where the toolchain
has none.

## Error-class shape

123 distinct first-error classes, **96 of them singletons** — the residual is a
long tail, not a few big causes. The clusters worth naming:

| class | papers | status |
|---|---|---|
| *no error at all* | 32 | open — the honest signal gap |
| `unexpected:\lx@begin@alignment` | 28 | diagnosed, engine-level, `repros/f7_alignment_fenced_amp/` |
| `unexpected:\endgroup` | 8 | open |
| `unexpected:\@end@tabular` | 7 | diagnosed to `\ce` in a `p{}` column, `repros/f8_ce_in_p_column/` |
| `Fatal:Stomach:Recursion` | 6 | general fatal mission |
| `malformed:ltx:XMTok` | 6 | open |
| `unexpected:\lx@end@inline@math` | 5 | open |
| `undefined:\volumeheader` | 4 | open, all one template |
| `malformed:ltx:biblist` | 4 | open — amsrefs bare `biblist` (F12) |
| `latex:(newunicodechar)` | 4 | open |

## Reading order for the next session

1. **Family A's 29 biblatex papers** — largest actionable group, and F4(c)/(d)
   already names the mechanism.
2. **The 32 with no diagnostic** — the class this whole audit exists to expose;
   family A's 18 and family B's 12 overlap here.
3. **The alignment cluster (28)** only with an engine-level change in its own
   branch.

## Characterized single-witness cases (mechanism known, not fixed)

**2606.01320 — a bibliography gated on a citation counter that our CS lock keeps
at zero.** `ncpds.tex` L27-28 does `\newcounter{cite}` +
`\pretocmd{\cite}{\stepcounter{cite}}{}{}`, then L2845 emits the bibliography
only inside `\ifnum\value{cite}>0`. Our core `\cite` is `locked => true` (this
PR's own #88 fix, commit `6f0e29477d`, which stopped raw conference styles —
aaai, iccc, flairs, kr, achicago, harvard, fixbib — from clobbering it), so the
hook's `\edef\cite{…}` is refused; the counter never leaves 0 and the whole
bibliography is skipped. **etoolbox still reports the patch as succeeding**
(measured: `\pretocmd`'s success branch runs, `\arabic{cite}` = 0) because
`\ifdefmacro{\cite}` is true here and only the assignment is refused.

The document is NOT truncated — L3972 is its last content line, followed only by
`\end{example}`/`\end{appendices}`/`\end{document}`. Only 1 residual paper
patches `\cite` this way and only 1 gates on the counter, so this was left
unfixed rather than risk the clauses that recovered several papers.

Remedy if it recurs at scale: let the etoolbox *hooks* (`\pretocmd`/`\apptocmd`,
which embed the original via `\expandonce` and so are non-destructive) assign
through the lock, while plain `\def`/`\renewcommand` from raw source stays
refused. That needs an unlocked-assignment window at the `\etb@hooktocmd` site
(`etoolbox_sty.rs` L1411) — `state_is_unlocked()` already exists as the gate.
Do NOT simply unlock `\cite`.

**Open, separate from the bibliography mission:** the diagnostic for a refused
redefinition (`state.rs` L1169-1184, Perl `State.pm` L509-515
`Info('ignore', …, "Ignoring redefinition of \cite")`) could not be made to
appear in any output at `--verbosity=1..3`, though the lock demonstrably holds
(a `\renewcommand{\cite}[1]{CLOBBERED}` in document source leaves no
`CLOBBERED` in the result). `SOURCEFILE` *is* assigned
(`core_interface.rs::establish_source_context`) and stores `Stored::String`, and
the `\.(tex|bib)$` predicate should match, so the gap is in whether this path is
reached at all for `\renewcommand`/`\edef` — worth a look, since a refused
redefinition silently losing document behaviour is the failure mode
CLAUDE.md's signal-integrity rule exists to prevent.

**2605.08378 — submission missing its own class.** cortex converts `thesis.tex`,
which is `\documentclass{PurdueThesis}`, and `PurdueThesis.cls` is **not in the
zip** (the zip ships `packages/neurips_2025.sty` and a decoy `ap-mathematics.tex`
that also has a `\begin{document}`). Hence `\ConfigureBibliography` undefined.
Source incomplete; no local PDF to confirm against — check the arXiv PDF before
counting it either way.
