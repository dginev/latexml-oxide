# Triaging the residual — method notes and two traps

Written for whoever picks this up next. The residual after this PR is 292 of
the 533 known articles; `verification_2026-07-29.tsv.gz` has the per-paper rows
and the audit doc has the first-error clustering.

## Trap 1 — the first error is often incidental

Clustering by first error found 6 papers led by
`Error:unexpected:\usepackage The current command '\usepackage' can only appear
in the preamble`. That error is **not** why they lose their bibliography: a
minimal document with `\usepackage` after `\begin{document}` recovers cleanly
here (tail and bibliography both survive, 3 errors raised). The papers are
multi-file submissions whose `\input`ed files are each standalone — own
`\documentclass`, own `\usepackage` block, own `\end{document}` — so the
`\usepackage` errors are a symptom of the shape, not the cause.

Always confirm a cluster's first error actually *causes* the loss before
chasing it. The cheap check is a minimal document with just that construct.

## Trap 2 — get pdflatex ground truth before calling something a bug

Two papers from that cluster, same first error, opposite verdicts:

- **2605.17865 — faithful, not a bug.** Its `combined.tex` saves
  `\bibliography` to `\origbibliography`, no-ops the original so the `\input`ed
  files cannot emit a list, then calls `\origbibliography` at the end. But
  `main.tex` carries its own `\end{document}`, which fires first, so the saved
  call is never reached. Running pdflatex on the real `combined.tex` produces a
  17.7 MB PDF with **zero** occurrences of "references"/"bibliography". The
  author's wrapper is broken; we match it.
- **2606.09184 — genuine loss, still open.** Same shape (five `\input`ed
  standalone files, each with `\end{document}`), but pdflatex renders
  **17 bracketed entries under "References"** from the `thebibliography` at
  `main.tex` L65. We stop after the abstract (5.5 KB of text) and emit 0. Note
  the PDF does *not* contain text from the last `\input`ed file either, so
  pdflatex truncates somewhere too and still gets the reference list out —
  the mechanism is not yet understood and is the thing to chase.

The method that settles it, and is worth using on every remaining cluster:

```bash
unzip -qo /data/arxiv/<yymm>/<id>/<id>.zip -d s && cd s
pdflatex -interaction=nonstopmode <main>.tex >/dev/null 2>&1
pdftotext <main>.pdf - | grep -coE '^\[[0-9]+\]'   # bracketed reference count
pdftotext <main>.pdf - | grep -n -iA 6 references  # and read them
```

A paper whose own PDF has no reference list is faithful-to-broken-source, and
belongs in the excluded pile rather than the worklist.

## Bisecting a truncation

Both hard-won and repeatedly relearned in this campaign:

* Delete **balanced** regions from the complete file. Truncating a prefix cuts
  inside a `figure`/`minipage`/`align` and manufactures its own error — the
  signal goes non-monotonic (one bisect showed cut-at-520 failing while 550 and
  600 were clean).
* Keep the preamble definitions the body needs in every slice. Dropping
  `\def\beq{\begin{eqnarray}}` leaves `\beq` undefined, the equation never
  runs, and a broken slice reads as a false "clean".
* Never probe with a marker word placed after the runaway. It survives
  *inside* the swallowed listing/verbatim, so `grep` finds it either way. Probe
  with something that only exists when the construct really executed —
  `\bibliographystyle` appearing as literal text, or the bibitem count.
* Red/green is the bibliography length: 0 is red, the full list is green.
