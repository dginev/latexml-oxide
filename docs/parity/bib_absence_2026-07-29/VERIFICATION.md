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
| **Recovered** (0 → non-zero) | **256** | **17 937** |
| Complete (`now == cited`) | 248 OK | |
| Short of what was cited (THIN) | 8 | |
| Still empty | 272 | |
| No HTML | 5 | |

Regression control: 20 papers whose bibliographies already worked reconvert
with **identical** counts. `cargo test --tests`: **1777 passed, 0 failed**.

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

## What is still empty (272), by first error

*no error at all* 32 · `unexpected:\lx@begin@alignment` 28 ·
`unexpected:\endgroup` 8 · `unexpected:\@end@tabular` 7 ·
`Fatal:Stomach:Recursion` 6 · `unexpected:\usepackage` 6 ·
`undefined:\setboolean` 6 · `malformed:ltx:XMTok` 6 · long tail.

The `expected:\fi` bucket that led this list (42 papers) is **gone** — it was
achemso's `{tocentry}`, suppressed with an `\iffalse` whose `\fi` lived in a
macro body where conditional skipping can never see it. The silent bucket also
gave up `\captionof` opening a verbatim-bodied environment (OXIDIZED_DESIGN
#87). Both were OUR constructs manufacturing what looked like source defects.

**Three papers moved the wrong way**, and honestly so: 2606.08929, 2606.12056
and 2606.15422 went from EMPTY (1 error, truncated HTML) to NOHTML (513 errors,
fatal). They had 0 bibliography entries before and after; the `{tocentry}`
swallow had been hiding an unrelated SVG/math group leak in each, which now
surfaces and trips the error cap. That is the F2 pattern again — a real defect
becoming visible — but it costs the partial output, so the underlying leak is
worth its own fix.

`docmute` accounts for another 5 (a native binding; the raw `.sty` is inert
here because it patches the `document` environment rather than our
`\begin{document}` control sequence) — 2605.17865 0 → 69, 2606.21971 0 → 41.
`undefined:\setboolean` is also gone: all 6 were `\documentclass{pnas-new}`,
whose class declares four booleans our binding did not
(2606.29674 0 → 60, 2605.07504 0 → 53, 2606.02411 0 → 45).

The alignment bucket (28) is diagnosed but deliberately NOT fixed here — see
`repros/f7_alignment_fenced_amp/`. It is a 14-line reproducer, and the cause is
engine-level: alignment rows are split on the **unexpanded** `&`, so any macro
whose delimiter-fenced argument contains `&` leaks into the enclosing row. The
physics binding already applies the documented remedy and the leak persists, and
a plain user macro reproduces it, so the fix belongs to the alignment machinery —
its own branch and corpus measurement, not this PR. The 32 silent papers are the
other open front.

## Reproducing

```bash
cargo build --release --bin cortex_worker --no-default-features --features cortex
tools/bib_recheck.sh -j 28 --baseline .sandbox-13 --list <ids-from-2605.flagged.tsv>
tools/bib_recheck.sh -j 28 --baseline .sandbox-14 --list <ids-from-2606.flagged.tsv>
```
