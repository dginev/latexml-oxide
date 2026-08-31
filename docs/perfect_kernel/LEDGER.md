# Perfect Kernel — progress ledger (living)

Protocol and status legend: [README.md](README.md). Newest sweep first.
Per-document artifacts live under `~/data/perfect_kernel/<bundle>/<name>/`
(out of repo); the sweep tally file is `~/data/perfect_kernel/sweep_verdicts.tsv`.

## Sweep history

| Date | Binary (commit, profile) | Corpus | Timeout | S0 fail (3/124/137) | status 2 | status 1 | status 0 | Notes |
|---|---|---|---|---|---|---|---|---|
| _pending_ | | 2374 docs (TL2025) | 120s | | | | | baseline sweep |

## Fix log

One row per landed kernel/engine fix attributable to this mission. Guard test
names are the durable part.

| Date | Fix | Cluster addressed | Guard test |
|---|---|---|---|
| | | | |

## Named exemplar: nicematrix (the mission's reference bundle)

`nicematrix.tex` (6,954 lines, LuaLaTeX-authored manual; tikz, siunitx,
tcolorbox, enumitem, fancyvrb, titlesec, varwidth, adjustbox…).

| Date | Binary | status | errors | fatals | warnings | secs | Note |
|---|---|---|---|---|---|---|---|
| 2026-08-31 | 1ef264a2bb test | 2 | 102 (capped) | 0 | 79,166 | 8.5 | first baseline; dominant noise: `Unrecognized tabular template` from `{NiceTabular}`-family preambles hitting the alignment template reader (`alignment.rs:997`); error heads: `Extra alignment tab '&'` ×26, `\noalign cannot be used here` ×11, nested-sectioning schema errors |
