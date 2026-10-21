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
- **Ownership by evidence, never by position**: an e-mail goes to the author whose name it spells, else the group's only
  author, else the block (jinstpub OD #801, 2026-10-21; jheppub to follow if measured better). An address that spells no
  one is not guessed onto an author.
- **The marks shown on a name are its requests for labelled affiliations**; a name's label is the name alone
  (2026-10-21, rank 93).
- **A footnote-list symbol (`\dag` included) that starts a line labels that line**; the line goes to the authors who
  carry the symbol, affiliation or legend alike (2026-10-21).
- **A second `\maketitle` keeps the latest title** (2026-10-01); multi-contribution volumes are deferred (KPE #1026).

## 5. Bindings and shipped copies

- **A binding defines a macro where the original package or class defines it**, under its conditions (2026-10-17).
- **A binding wins over a shipped copy** (2026-10-17); a copy may override **text names** (`\refname`, caption labels,
  journal abbreviations), never **structure** (`\appendix`, `\frontmatter`) — standard-class derivatives (OD #731
  rule 13) and, since 2026-10-21, copies routed to a binding by their `\Provides` line (1402.7103).
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
