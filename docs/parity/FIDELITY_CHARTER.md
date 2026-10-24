# Fidelity charter — the standing rules for conversion decisions

The user's rulings, distilled into the rules a writer or reviewer applies **without asking**. Each rule names the
ruling it comes from (date, and the OD/KPE entry where one records it). Ask the user only when two rules conflict,
when no rule covers the case, or when a change departs from Perl for a large population and no rule below already
licenses it.

## 1. The order of priorities (user 2026-10-06)

1. **The author's content** — every word and structure the author wrote survives.
2. **Semantic markup** — each construct that has a LaTeXML-schema element is that element (frontmatter elements,
   classes, links, theorem-likes, floats, cross-references, MathML Core).
3. **Fidelity to the PDF** — what pdflatex (the source's intended engine) prints. PDF fidelity gives way first.

When two rules below pull apart, the higher priority wins.

**Where to stop** (user 2026-10-24): a completely readable HTML rendering is essential to a reader, so priorities 1 and 2
— content preservation and the semantic XML and HTML markup — are held to 100 %. Secondary details of priority 3 (sizing:
widths, heights, panel fractions, box dimensions) are done once 90-95 % of their issues are handled; the rest stay
one-line SYNC rows, not payloads.

## 2. Content

- **Authored text is kept, even where the class or the PDF discards it.** IEEEtran's `\@IEEEdestroythesectionargument`
  throws away a `\section` title after `\appendix`; ours keeps it as a heading (OD #793, 2026-10-21).
- **Page furniture is dropped; semantic notes are kept** (2026-10-01): running heads/feet, page numbers, letterheads,
  decorative template labels go, as in Perl; footnotes, frontmatter notes, and corresponding-author lines stay even
  when a class prints them in a header, footer or margin.
- **Generated content follows Perl**: a table of contents a class's `\maketitle` prints (JHEP/JINST) is not emitted;
  the reader's navigation comes from the HTML (2026-10-21).
- **Label text is markup, never stray content and never dropped**: "E-mail:", "Keywords:", "Require:" become an
  element or attribute CSS can style or hide (2026-10).
- **An empty presentational shell left by our own restructuring is not emitted**: a thanks note whose entire content
  became contacts (OD #803, 2026-10-21). An author's own empty `\thanks{}` is the author's and stays.
- **Duplicates the author printed stay duplicated**: an address in the author list and again after "Correspondence:"
  is kept twice (2026-10-21).
- **Display toggles are honored as the PDF shows them**: blind review, anonymous, final/draft options (2026-10).
- **Metadata the class only writes to PDF metadata** (e.g. `\icmlkeywords` → pdfkeywords) becomes RDFa, not a displayed
  block (2026-10-09).
- **Float panels: the grid first, and the LaTeX sizes kept** (2026-10-22, at 78t). A group of sub-figures/sub-floats is a
  logical grid: rows × columns, as many panels per row as the PDF sets, and for advanced grids the cells a panel spans.
  Capture that structure; do not re-run TeX's line breaking against one page geometry (the browser width varies with
  the theme — ar5iv uses 52rem unconditionally — and one class's exact page width is not the information to model).
  Derive the rows from what the source states (explicit breaks; widths as fractions of the line width). The grid is not
  the whole story: HTML needs well-sized images, so keep the widths the LaTeX side states (as for the PDF) as the
  starting point for sizing. ar5iv-css sizes panels by the grid and by the image's shape (landscape, portrait, square);
  the final sizing design is open (design study task #360, `agent_reports/2026-10-22_grid_design.md`).
  Two abstractions (user 2026-10-22): a **gallery** — independent, usually sub-captioned panels whose order matters but
  whose position does not, so reflow by whole rows on a narrow screen is harmless — and a **matrix** — panels whose row
  and column carry meaning (uniform rows/columns, row or column labels, one caption, a `tabular`-built grid), which keep
  their columns at any width (cells shrink, then scroll) and never reflow.
  Ruled 2026-10-22 (grid study questions): the markup is **today's elements plus attributes** — `panelwidth`/`panelspan`
  on each panel, `panelrows`/`panelcols` and a class `ltx_panels_gallery`/`ltx_panels_matrix` on the float, mapped by the
  XSLT to CSS custom properties (no row wrapper, no table for non-`tabular` grids); rows are computed against the class's
  **real line width** (a two-column `figure` → the column, `figure*` → the text width), the `\columnwidth` register
  staying Perl's for everything else; a group is a matrix only on **strong signals** (an author-built `tabular`, or ≥ 2
  rows each with the same column count and equal panel widths), else a gallery. **Sizing keeps the panels' relative
  sizes within one figure**: capping each image separately at its LaTeX size skews the read-out against the PDF, so
  the group scales as a whole. Ruled 2026-10-23: **sizes are fractions of the PDF's running-text column at the float**
  (`\columnwidth` in two-column mode, `\textwidth` in one) — the float states `panelline` (its line ÷ that column: 1 for a
  column figure, ≈ 2.07 for a `figure*`, < 1 for a wrapfigure) and `panelfill` (the widest row's Σ `panelwidth`); the
  theme maps that column to its own width, so the converter emits no rem/pt constant (OD #887's rendering rule). A
  deliberate overfill pulled together by negative glue (2301.02117) is an accepted
  residual.

## 3. Numbering, labels and references

- **Numbers, counters and `\ref` text follow pdflatex**: the class's own `\the<counter>`, `\p@<counter>` chains,
  `secnumdepth` (REVTeX's PRL unnumbered headings, KPE #1027), heading forms (`\fnum@<level>`, IEEEtran KPE #1022).
  For a beyond-Perl call pdflatex is the oracle, not a reporter's stated engine.
- **`\autoref`/`\cref` names are bound at the target** (2026-10-02, ruling 7e).
- **Dangling `\hyperlink` targets stay** (2026-09-18); never strip links for schema validity.
- Where keeping authored content forces a reference to differ from the PDF (OD #793's later appendices "B", "C"),
  record the difference in the OD entry.

## 4. Frontmatter

- **One dialect**: class setters capture metadata and rewire to the `\lx@add@*` API; no presentational artifacts
  (tabulars, minipages) inside a `personname` (2026-10).
- **Ownership by evidence, never by bare position**: an e-mail goes to the author whose name it spells; else, in a list
  with exactly one address per author that every spelled address agrees with, to the author at its place (the consistent
  author-order list is evidence, 2026-10-21, OD #805); else to the group's only author (jinstpub OD #801); else, for the
  one address left on the block of a multi-author paper, to the one author marked as the corresponding author (a
  "Corresponding author" note, or a mark whose legend says so; 2026-10-21); else to the block. Otherwise an address that
  spells no one is not guessed onto an author.
- **The marks shown on a name are its requests for labelled affiliations**; a name's label is the name alone
  (2026-10-21, rank 93).
- **A footnote-list symbol (`\dag` included) that starts a line labels that line**; the line goes to the authors who
  carry the symbol, affiliation or legend alike (2026-10-21).
- **A second `\maketitle` keeps the latest title** (2026-10-01); multi-contribution volumes are deferred (KPE #1026).

## 5. Bindings and shipped copies

- **A binding defines a macro where the original package or class defines it**, under its conditions (2026-10-17).
- **Math letters follow the class's declarations** (2026-10-23): where a class declares the capital Greek letters
  `\mathalpha` from `letters` (llncs, svmult, svjour3) they print italic, as Latin letters do, and a math alphabet still
  overrides (`\mathrm{\Omega}` upright); a package that declares the same (fixmath; mathptmx/newtxmath/cmbright under
  `slantedGreek`) follows the same rule. Keyed on the declaring command, not the character (svmult's `\var<Greek>` stay
  upright).
- **A binding wins over a shipped copy, for markup and for typography** (2026-10-17, 2026-10-21): bindings are the ground
  truth for the XML. A copy may supply only the **text names its binding declares** — the kernel's default list of the
  standard `\…name` macros (`\figurename`, `\tablename`, `\abstractname`, `\refname`, …), extended per binding (journal
  abbreviations) — and only a standard-class derivative (OD #731 rule 13) or a copy routed to a binding by its `\Provides`
  line (rule 14, 1402.7103). Caption formats (`\fnum@…`), separators, counter separators, list labels and structure
  (`\appendix`, `\frontmatter`) stay the binding's; a copy shipped under the binding's own name overrides nothing.
- **Typographic variation in raw class or package code is ignorable**; where it is worth keeping it becomes a setting the
  binding makes, with a default, never a reading of the raw definitions (2026-10-21: the caption separator stays the
  binding's, default ": " as in Perl).
- **One binding per class or package family** supports the union of its versions' features; a feature is validated
  against the version that defines it (2026-10-09).
- **A stub loads the real dependencies** its original `\RequirePackage`s (2026-10-12).

## 6. Diagnostics

- **Every Warning, Error and Fatal is emitted and counted, once**; recovery never silences one; the only sanctioned
  silence is `IgnoreDiagnosticsScope` (2026-09-20, reaffirmed 2026-10-11).
- **Class and package errors are reported as the class reports them** (2026-10-17), with ruled exceptions: caption's v1
  options on a repeat load stay an unknown-key Warning (2026-10-21).
- **A Fatal stays Fatal**; a panic fix emits an error with its reason.

## 7. How to decide and how to verify

- **Mechanisms before heuristics**: TeX/LaTeX semantics (print events, provenance, the class's own code) first; text
  matching only as a last resort that keeps content in place (2026-10-16). A third patch of the same shape becomes a
  shared mechanism (2026-10-18).
- **Record every divergence**: beyond-Perl → `OXIDIZED_DESIGN_DIVERGENCES.md`; Perl-origin defect →
  `KNOWN_PERL_ERRORS.md`.
- **Three metrics, honestly**: error-free logs, authored content preserved, semantic markup; a green test asserts all
  three; an honest success drop is acceptable (2026-10-18).
- **Reach-complete A/Bs** (2026-10-21): a payload's A/B covers every sample paper that uses each macro, environment,
  class or package it touches (the census tool in `~/data/pk_agents/main/census/`), and checks the changed outputs
  against the PDF (reference numbers, heading numbers, e-mail and affiliation owners), not only Error/Fatal counts.
- **Red/green TDD**: a minimal pdflatex-checked repro under `tools/perfect_kernel/repros/` and a whole-element guard,
  red before the fix.
- **Repro stand-ins** (2026-10-21): a shortened copy of an LPPL file keeps its notice and says it is modified; a file
  whose source states no licence is rewritten from scratch as a minimal stand-in citing the source's line numbers.
