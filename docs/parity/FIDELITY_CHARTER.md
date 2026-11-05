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
one-line SYNC rows, not payloads. Complete breakage counts as readability and is held to 100 % too (user 2026-10-24): a
missing image, a broken `\ref` or `\cite`, and the like — the fixes are usually coverage gaps.

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
  became contacts (OD #803, 2026-10-21). An author's own empty `\thanks{}` is not emitted either (2026-10-30, superseding
  2026-10-21's "stays": pdflatex prints a bare mark, and an empty note only adds a "Thanks:" label over nothing).
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
- **An author's own call holding one address keeps it** (2026-10-29, #420): when an `\email` (or the class's
  equivalent) belongs to one author's own call — amsart's `\author{A}\address{…}\email{x}`, one address — the class's
  structure says one address per author, and spelling does not move it unless the address spells another author. An
  address that spells another author as printed does move (`\author{Anna Smith}…\email{mueller-group@x.de}` beside
  `\author{Jorg Mueller}` moves to him; amended to match the data, user 2026-10-31). **A transliterated spelling is a
  spelling as printed** (user 2026-11-05: "joerg.mueller@ clearly only matches the Jörg Müller (also written Joerg
  Mueller) author"): ä/ö/ü/ß/å/ø/æ/œ → ae/oe/ue/ss/aa/oe/ae/oe, so the same address beside `\author{J\"org M\"uller}`
  moves too (the 2026-10-29 example that kept it on Anna Smith is superseded), and an AASTeX
  `\author{J\"org M\"uller and Ann Able}\email{joerg.mueller@x.de}` is Müller's. A literal and a transliterated
  spelling are equal evidence (user 2026-11-05: "we can't be confident that is the correct author due to the umlaut
  not being part of email addresses at all"): `mueller@` beside Hans Mueller and Jörg Müller spells both, a tie, so no
  author by spelling — the order below decides (author-order list, elimination, else the block): measured 2026-10-31 (brief 95c r3, OD #938), the corresponding author's address set after the last
  `\author` or `\correspondingauthor` is the author it spells on 22 real papers of a 347-paper respell population
  (1303.5460, 2005.06651, 1511.04148, 2408.09634, …; 2011.01984 the batch63 guard's), the call's author on 2
  (0711.3633, 0809.3090: the holder's given name). A call holding several addresses is not
  that evidence: spelling decides among them, as above (1204.1484: `montaldo@`, `piu@` under Caddeo's call).
- **The marks shown on a name are its requests for labelled affiliations**; a name's label is the name alone
  (2026-10-21, rank 93).
- **All asterisks are one label key** (2026-11-02, user: "all asterisks are the same key, they are visually identical
  to readers"): `*`, `\ast`, `\textasteriskcentered`, ∗ U+2217, ⁎ U+204E, ✱ U+2731 and ＊ U+FF0A key the same label, for a
  name's request and a line's label alike (`clean_frontmatter_labels`); the printed glyph stays as written. Labels key by
  what a reader sees, not by the TeX spelling.
- **"Deceased" is an annotation on the author, of the same kind as "Corresponding author"** (2026-11-02): a legend
  line `$^\dagger$Deceased` is a note of the names carrying its symbol, never an affiliation, read by the shared kind
  reader (`classify_thanks`) alongside correspondence, contribution and address change.
- **A footnote-list symbol (`\dag` included) that starts a line labels that line**; the line goes to the authors who
  carry the symbol, affiliation or legend alike (2026-10-21). An e-mail address on such a line that spells none of the
  carriers is every carrier's — the symbol-line rule wins over "not guessed onto an author" there (2026-10-30, "every
  symbol carrier"); one that spells a carrier is that carrier's; and a spelled address beside it does not send the
  unspelled one to the block (`Ann Able$^\dagger$, Cat Cole$^\dagger$ … $^\dagger$\texttt{alpha@x.org,
  beta@y.org}`: both addresses Ann's and Cat's).
- **The frontmatter exit bar's two rulings** (2026-10-29): a family deferred with its witnesses in
  `DEFERRED_FAMILIES.md` (hand-typeset title pages, ifacconf's `{frontmatter}` box, names stacked under names, …) is
  excluded from the readability bar — its witnesses stay recorded for later work, not dropped; and the archival tier's
  provenance (how each creator and contact was decided) is recorded as a **schema attribute**, not a class token.
- **An abstract's place is the schema's** (2026-10-30, superseding the same day's "a late abstract keeps print
  order"): "it is more important to follow the latexml schema than to get the exact abstract placement of the PDF".
  `ltx:abstract` is frontmatter, and the document model puts frontmatter and titlepages before any body element
  (LaTeXML-structure.rnc:34-36), so an abstract printed after body text — nature-pre's `\date{\today}` set after
  `\maketitle` between `{affiliations}` and `{abstract}` (2608.12563, 2102.01782, 2111.09589, 2412.11966) — stays in the
  frontmatter, before it. The abstract's position relative to the PDF is not a readability defect; its content is.
- **Frontmatter label keys are compared by their printed text** (2026-10-29, refactor stage 4): the marks that link
  authors to affiliations (`\inst{…}`, `$^{…}$`, `\textsuperscript{…}`) are keyed by the printed text of the digested
  operand, not Perl's source-string ToString — aa's `\inst{\ESO}` with `\newcommand{\ESO}{2}` prints 2 and is entry 2's;
  `\dag` and `$\dagger$` are one label. A documented divergence from Perl's frontmatter API (OD entry with the stage).
- **A frontmatter note reads as the class prints it** (2026-10-30): its label is the class's own printed label — aa's
  and svjour's `\offprints{O. Kochukhov, \email{…}}` is "Send offprint requests to: O. Kochukhov, …" (aa.cls,
  svjour3.cls:371-374), never a role name the class does not print ("Preprint: "); a note we cannot give its printed
  reading is better hidden than shown mislabelled ("it adds confusion"). Where the class prints no label, the note
  is kept, with its role, and has none (2026-10-30, "keep every note with the label the class prints, which here is
  none"): the kernel's title `\thanks{X}` (pdflatex: a "∗ X" footnote) is X, not "Thanks: X"; aa's `\dedication`,
  aastex631's `\reportnum`, emulateapj's `\slugcomment`/`\journalinfo` likewise. Perl's role defaults
  (Base_Utility.pool.ltxml:768-787 `\lx@pubnote@<role>@name`) stand only where the class prints that label; a label
  the binding writes into the content (IEEEtran `\IEEEpubid`'s "pubid: ") goes too. The role stays semantic: `role=` in
  the XML, the `ltx_role_<role>` class in the HTML (LaTeXML-common.xsl:538), which a CSS theme may style or label
  (2026-10-30); only the `ltx_note_name` label span goes.
- **A note our own extraction empties is not emitted** (2026-10-30, generalising OD #803 from thanks notes to every
  frontmatter note): `\offprints{\email{x}}` whose only content became a contact leaves no "Send offprint requests to:"
  shell; a note whose remainder still informs stays with its printed label (`\offprints{O. Kochukhov, \email{…}}` reads
  "Send offprint requests to: O. Kochukhov," — the address is in the author list).
- **A collaboration or team named in the author line is a contact on its authors** (2026-11-01): `\author{A$^1$, B$^2$ and the GAPS Team}`, a `(XYZ Collaboration)` line under the names, BESIII's `\author{\center BESIII Collaboration\\ … (BESIII Collaboration)}` — the names split into their creators and the collaboration is a `<contact role="collaboration">` on every name of the line it covers (Perl's aastex convention, aas_support.sty.ltxml:126 `\collaboration` → `\lx@add@contact[role=collaboration]`), not a creator of its own and not welded into the last name (dev census M4: 1708.04166, 1711.07097, 1311.7305, 1905.11825, hep-ph0502212; 2212.06489).
- **An address no author's name spells, in a boxed author block, is a block-level contact** (2026-11-01): a shared `contact@lab.org` line in a `tabular`/`minipage` block goes to no person; it is kept once at the block level (dev census M1, 2605.09461).
  **Amended 2026-11-05 (user: "block addresses may live on the #159 block creator"):** the block level is the shared
  nameless creator below the authors (OD #159, `label=addresses:block`) — the one nameless creator allowed, holding
  only the block's contacts (step 4 of the placement order below: an address, e-mail or URL no author's name spells,
  and whatever else no mark, spelling or convention places). No other nameless creator is made: a legend line whose
  mark the names carry is their note, never a creator of its own (html_feedback #7233's one-line variant,
  `repros/sectioning-frontmatter/email_list_unspelled_entries_legend_becomes_creator_red.tex`).
- **One placement order for every frontmatter contact** (2026-11-01; `parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`): an affiliation, address, e-mail, note, collaboration, ORCID or URL goes (1) by its labels and marks — `label=`, a declared `labelseq`, its `\label`, its leading printed mark, a hand-printed leading number where the names request numbers — to every name requesting them; else (2) to the one author it spells as printed; else (3) by the class's declared convention for unmarked items; else (4) to the block, once. **Marks win (R1, R3):** a convention never gives an item printed apart from the block to an author whose own marks request items of that role (unmarked lines directly under the names row stay with that row), and a class's printed numbering is a label, so no convention outranks a mark — aastex's run gave every author every `\affil` line where marks decided (1810.01941: 39 → 364 affiliations). **Position (R2):** hand-printed numbers (`1 Univ A`, `2) Univ B`) pair as labels wherever names request numbers; bare position pairs only under a declared numbered sequence. **Licence (R4):** a difference from the old reader's output that follows this order needs no per-change tag; the comparator stays the census gate and every A/B change is still read against pdflatex. **Order (R8):** a creator's contacts follow the order the PDF prints them.
- **No printed frontmatter word is lost** (2026-11-01, on the dev lost-words census): printed frontmatter text that no owner claims — a raw class's own stores that only its title-page code prints (raa's `\abstract`, sig-alternate's `\titlenote`, copernicus's `\AB@affillist`) — is written at the block as an `ltx:note` with a counted Warning, only for lines with several unmatched words; the large families also route through the API to their owners. Printed template placeholders ("dd Mmm YYYY", "00 Month 201X") are words like any other. The zero-lost-words bar covers the abstract and keywords against pdflatex's text.
  **Amended 2026-11-04 (user, after the diminishing-returns analysis):** the bar is now **every residual loss counted by a Warning**, not zero lost-words papers — one generic self-check compares the title page's printed words with the output and emits a counted `Warning:frontmatter:unclaimed` for each word lost; a loss is fixed only when its mechanism reaches about 500 arXiv papers or more. Shaped short lines below the note floor (addresses, codes, dates, labelled datelines, batch 105a) are kept.
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
- **A shipped copy is routed to the binding whose command set it defines** (2026-11-01): a renamed copy of a binding's style (`arxiv2025.sty`, `icml2021_oppo.sty` defining the icml binding's `\icmlauthor`/`\icmlaffiliation`) is read through that binding, not raw — the binding owns markup and typography (dev census M9; 2502.01411, 2209.13569).

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
