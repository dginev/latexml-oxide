# BibTeX.pool.ltxml — Rust port plan

**Status (2026-05-15):** scoping. `latexml_engine/src/bibtex.rs` is a
37-line skeleton; the Perl pool is 956 lines. Per the
[strict-parity audit](PERL_LOADFORMAT_AUDIT.md), the 116-CS `\bib@*`
family is the largest single block of Perl-only CSes (~62% of the
remaining engine-wide Perl-only gap). This document scopes the port
into discrete phases so future sessions can pick up a phase without
re-deriving the dependency graph.

## What BibTeX.pool.ltxml does

Defines the bibliographic-entry processing pipeline for any pool
that chains in via `LoadPool('BibTeX')`. Primary client:
`amsrefs.sty.ltxml` (the AMS `\bib{key}{type}{keyval-pairs}` form
common in math-paper bibliographies). Direct users also include
DLMF-style entries with `\MR{...}` / `\Zbl{...}` cross-references.

Output is `<ltx:bibentry>` / `<ltx:bib-name>` / `<ltx:bib-title>` etc.
schema nodes, downstream consumed by `latexml_post::make_bibliography`.

## Why direct porting needs infrastructure first

The Perl pool is ~70% Perl-`sub { ... }` closures over a **BibEntry**
data structure that doesn't exist on the Rust side. Even the
"simple" pure-string `DefMacro`s call into closure-backed
`DefConstructor`s (`\bib@addto@related`, `\bib@@field`, `\bib@@names`)
which need state lookup, document-node insertion, and name-list
processing helpers.

Verbatim port without supporting helpers ships dead-end stubs that
silence "undefined CS" errors but produce wrong output (lost
metadata). Audit principle (CLAUDE.md): "preserve original
semantics, control flow, edge cases" — that requires the helpers
first.

## Dependency graph — what to port first

### Phase 1: Foundation helpers (no bindings, just plumbing)

These are the Perl support routines that every binding eventually
hits. They live as `sub` definitions inside the pool but are
top-level helpers in spirit.

| Perl helper | Purpose | Rust target |
|---|---|---|
| `currentBibEntry` | Look up the entry currently being processed | `latexml_engine::bibtex::current_entry()` — reads `BIBENTRY@<normkey>` from State Value table |
| `currentBibEntryField(field)` | Get a field's *processed* token value | `current_entry().get_field(field)` |
| `currentBibEntryRawField(field)` | Get a field's *raw* string value | `current_entry().get_raw_field(field)` |
| `copyCrossrefFields(@fields)` | Pull listed fields from the crossref'd parent entry | `copy_crossref_fields(&[&str])` |
| `bibAddToContainer(doc, tag, data, %attr)` | Insert into a `<ltx:bib-related>` container, deduplicating by tag+attrs | function on `Document` (or a free fn) |
| `processBibNameList(string)` | Parse "Smith, John and Doe, Jane" into a list of name tokens | new module `latexml_engine::bibtex::names` |
| `NormalizeBibKey` | Already in `latexml_core::common::cleaners::cleaners.rs:125` | reuse |
| `CleanBibKey` | Currently TODO | sibling of `NormalizeBibKey` |
| `ProcessBibTeXEntry` | Currently TODO; orchestrates the per-entry pipeline | new top-level function |

The **BibEntry** itself needs a Rust representation. Sketch:

```rust
pub struct BibEntry {
  pub key: String,
  pub entry_type: String,                  // "article", "book", ...
  pub fields: Vec<(String, Tokens)>,       // processed
  pub raw_fields: Vec<(String, String)>,   // raw verbatim string
}
```

Lookup by `BIBENTRY@<normalized-key>` in the State Value table
(custom `Stored::BibEntry(Rc<BibEntry>)` variant, similar to how
`Stored::Font` rides through value entries).

### Phase 2: Core constructors (5-7 bindings)

Once the foundation is in, port the constructors that move data
from the entry into XML. These build on `bibAddToContainer` and
`current_entry`:

- `\bib@@field {} OptionalKeyVals Digested` (L230) — inserts a
  single field node into the entry.
- `\bib@addto@related {}{} Digested` (L261) — inserts into a
  `<ltx:bib-related>` sub-element with type/role.
- `\bib@@@name{}{}` (L266) — emits `<ltx:bib-name>`.
- `\bib@@@names{}` (L270) — wrapper for a name list.
- `\bib@@names{}{}` (L271) — token-list processor that calls
  `processBibNameList`.

### Phase 3: Title-case logic and field helpers

- `\bib@@title{}{}{}` (L293) — title-case normalization. Five
  modes (asis / capitalize1 / capitalize / uppercase / lowercase).
  Distinct Rust function: `recase_title(text, mode) -> String`.
- `\bib@field@@ignore` (L343), `\bib@field@default@default` (L346),
  `\bib@field@unknownasdata` (L347) — fallback handlers.
- `\bib@@booktitle{}{}` (L337) — booktitle shortcut.

### Phase 4: Entry-type prepare/complete + field-type aliases

These are mostly pure-string `DefMacro`s that route per-(entry,
field) to the core constructors. They were what the original
"first batch" attempt aimed at — they're the *largest* chunk of
the CS-count (~80 of the 116 `\bib@*`), but each one needs Phase 2
infrastructure to do anything useful.

- `\bib@entry@<type>@prepare` for each of article, book, booklet,
  inbook, incollection, inproceedings, mastersthesis, phdthesis,
  manual, misc, online, patent, proceedings, techreport, thesis,
  unpublished. ~16 entries × `copyCrossrefFields` calls.
- `\bib@entry@<type>@complete` (~16 entries).
- `\bib@entry@<type>@alias` (a few; `conference -> inproceedings`,
  `mastersthesis -> thesis`, `phdthesis -> thesis`).
- `\bib@field@<type>@<field>` mappings: ~60 macros across the
  entry types (inbook/incollection/inproceedings each have ~8).

### Phase 5: Cross-references and identifiers

- `\bib@synthesize@mr` (L803) + `\bib@@mr` (L812) — MathReviews
  identifier synthesis from `mrnumber`/`mrreviewer` fields.
- `\bib@synthesize@zbl` + `\bib@@zbl` — Zentralblatt equivalent.
- `\bib@@origbibentry` — raw verbatim entry capture for the
  `<ltx:bibentry>` `original` attribute.

### Phase 6: BibTeX special-character handling (~L860-955)

Standalone-ish section dealing with diacritic and accented-letter
mappings for BibTeX-format author/title strings. Could be ported
in isolation as a Rust `char_replacements` table.

## Acceptance criteria

Per phase:
- **Phase 1:** Unit tests covering `current_entry` lookup,
  crossref-copy, NormalizeBibKey↔CleanBibKey round-trip,
  bibAddToContainer deduplication. No bindings ship yet.
- **Phase 2-3:** A minimal amsrefs paper with one `\bib{key}{article}{
  author={X}, title={Y}, journal={Z}, year={2020}}` produces the
  same `<ltx:bibentry>` XML as Perl LaTeXML (up to known XML
  divergences in `docs/OXIDIZED_DESIGN.md`).
- **Phase 4-5:** All 16 entry types + MR/Zbl synthesis produce
  Perl-equivalent XML.
- **Phase 6:** BibTeX-format `@author={Š{m}́{i}th, J.}` accented
  inputs decode to the same Unicode as Perl.

Overall: when the engine-wide CS-name diff (audit § Engine-wide
CS-name diff refresh) shows the `\bib@*` family count dropping
from 116 toward 0, the port is converging. Final acceptance: a
sandbox paper using amsrefs converts with Rust=Perl error counts
(SHARED-FAILURE at worst).

## Test corpus

Suitable arxiv papers:
- DLMF-derived papers in the math-classification corpus (use
  `\MR{...}` extensively).
- AMS journal-template papers (use amsrefs `\bib{...}{...}{...}`).
- Pick witnesses from sandbox sweeps showing
  `Error:undefined:\bib@*` first-errors once the work starts.

## Effort estimate

Rough phase sizing:
- Phase 1: ~300-400 LOC + tests. Foundation; biggest unknowns.
- Phase 2: ~150 LOC across 5-7 DefConstructors.
- Phase 3: ~100 LOC; title-case logic is the main piece.
- Phase 4: ~250 LOC (mostly DefMacro one-liners once helpers exist).
- Phase 5: ~120 LOC.
- Phase 6: ~80 LOC.

Total: ~1000-1100 LOC + tests. Likely 4-6 focused sessions.

## What this document is NOT

A binding-by-binding line-by-line translation guide. The point of
the plan is to make the work decomposable — each phase ships
testable code on its own, without leaving the engine in a
half-broken state.

## Related

- Skeleton: `latexml_engine/src/bibtex.rs`
- amsrefs stub: `latexml_package/src/package/amsrefs_sty.rs`
- Cross-engine audit: `docs/PERL_LOADFORMAT_AUDIT.md` § Engine-wide CS-name diff refresh
- Memory: `memory/wisdom_hub_kernel_dump.md`,
  `memory/wisdom_intarray_fontdimen_storage.md`
- Post-processing consumer: `latexml_post::make_bibliography`
