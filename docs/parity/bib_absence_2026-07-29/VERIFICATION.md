# Re-conversion verification — the 533 known articles

Every article this PR claims to fix was **re-converted from its arXiv source
with the PR build** and re-examined. This file is the evidence; the per-paper
rows are in `verification_2026-07-29.tsv.gz`.

Cohort: all 533 papers the audit flagged as *wrongly missing* a bibliography
in the 2605 (sandbox-13) and 2606 (sandbox-14) reruns — HTML present, source
demonstrably asks for a bibliography, zero `class="ltx_bibitem"`. **Every one
of them had a baseline of 0 entries** (`baseline` column, verified 0/533).

Harness: `tools/bib_recheck.sh`, which reconverts through
`cortex_worker --standalone` — the same binary and ar5iv profile the fleet
uses — then counts `ltx_bibitem` against what the source implies and what
`MakeBibliography` reports it cited.

## Result

| | papers | entries |
|---|---|---|
| **Recovered** (0 → non-zero) | **189** | **14 715** |
| Complete (`now == cited`) | 182 OK | |
| Short of what was cited (THIN) | 7 | |
| Still empty | 342 | |
| No HTML | 2 | |

Regression control: 20 papers whose bibliographies already worked reconvert
with **identical** counts. `cargo test --tests`: **1767 passed, 0 failed**.

## Duplication audit

A fix that emits a bibliography twice would look like a win by entry count, so
every paper was checked for over-emission (`bibitems > 1.5 ×` the source's own
`\bibitem` count). Four are flagged and all four are **single-section** with
`N bibentries, N cited` — the `want` estimate undercounts them because it falls
back to counting distinct `\cite` keys for `.bib`-only papers:

| paper | now | want | why |
|---|---|---|---|
| 2605.03129 | 21 | 5 | 21 bibentries, 21 cited, 1 section |
| 2606.18009 | 86 | 18 | `.bbl` route, 1 section |
| 2606.26959 | 18 | 1 | 18 bibentries, 18 cited, 1 section |
| 2606.32016 | 25 | 11 | 25 bibentries, 25 cited, 1 section |

This audit is what caught the REVTeX `auto@bib` doubling (2605.27226 330 → 660,
2605.13984 88 → 176) that led to withdrawing that change — see the audit doc's
F3(b) entry.

## Content spot-checks

Counts alone do not prove a bibliography *reads* correctly, so entries were
read as rendered text:

- **2605.27226** (GWTC-5, 330 entries) — "Aasi et al. (2015) Aasi, J., Abbott,
  B. P., Abbott, R., et al. 2015, Classical and Quantum Gravity, 32, 074001,
  doi: 10.1088/0264-9381/32/7/074001"
- **2605.00125** (54) — full author lists, journal, volume, pages
- **2605.03129** (21) — numbered labels, notes, and `Cited by: §1` backrefs
- **2605.21570** (46) — 34 + 12 across its two bibunits, each matching that
  unit's own `\begin{thebibliography}{N}`

## What is still empty (342), by first error

`expected:\fi` 42 · *no error at all* 34 · `unexpected:\lx@begin@alignment` 28 ·
`unexpected:\endgroup` 8 · `unexpected:\@end@tabular` 7 ·
`Fatal:Stomach:Recursion` 6 · `unexpected:\usepackage` 6 ·
`undefined:\setboolean` 6 · long tail.

The largest single bucket is **F6/achemso** (`expected:\fi`, 42), reduced to an
8-line reproducer in `repros/f6_tocentry_conditional/` and not yet fixed. The
34 with no diagnostic at all are the next thing to mine — silence is the
defect class this PR's F2 fix exists to remove.

## Reproducing

```bash
cargo build --release --bin cortex_worker --no-default-features --features cortex
tools/bib_recheck.sh -j 28 --baseline .sandbox-13 --list <ids-from-2605.flagged.tsv>
tools/bib_recheck.sh -j 28 --baseline .sandbox-14 --list <ids-from-2606.flagged.tsv>
```
