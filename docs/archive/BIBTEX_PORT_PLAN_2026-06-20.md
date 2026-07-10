# BibTeX.pool.ltxml — Rust port plan

**Status (2026-05-16):** Phases 1-7 shipped; Phase 8 (Pre::BibTeX
parser + `--bibtex` CLI mode) shipped. End-to-end `.bib → .xml`
flow lands the bibentries inside a `<ltx:bibliography>` element
with macro expansion (`@string`/`@preamble`) and faithful field
dispatch. `latexml_engine/src/pre_bibtex.rs` is ~700 lines (17
parser unit tests); `bibtex.rs` is ~1750 lines (BibEntry + name
parser + title-case + 16 bindings including `{bibtex@bibliography}`
+ 31 unit tests); the Perl pool is 956 lines, the Pre::BibTeX
module is 439 lines. The remaining gap is the long tail of Phase
4-5 field handlers/MR-Zbl synthesis polish.

## Coverage audit (2026-05-15)

Cross-checked the Perl pool entity-by-entity against bibtex.rs.

**Translated faithfully (18 entities):**
- `LoadPool('LaTeX')` (L19); `currentBibEntry`/`Field`/`RawField`
  helpers (L195-204); `bibAddToContainer` (L242); `processBibNameList`
  (L872); `splitWords` (L921).
- DefConstructors: `\bib@@field`, `\bib@addto@related`, `\bib@@@name`,
  `\bib@@@names`, `\bib@surname`/`given`/`lineage`,
  `\bib@field@unknownasdata`.
- DefMacros: `\bib@addtype`, `\bib@@names`, `\bib@@title`,
  `\bib@@booktitle`, `\bib@field@@ignore`,
  `\bib@field@default@default`.

**Translation divergences from Perl-actual:**

| # | Issue | Severity | Fix priority |
|---|-------|----------|--------------|
| B1 | `CURRENT@BIBKEY` Perl: AssignValue/LookupValue (group-scoped); Rust: thread-local `Option<String>` (NOT group-scoped). Nested `\bibentry@prepare` won't restore on `\egroup`. | Medium | Revisit when DefPrimitive prepares ship in Phase 4. |
| B2 | `copy_crossref_fields` uses raw-field for `crossref` key lookup; Perl uses processed `getField` Tokens. Subtle if key contains macros. | Low | Acceptable. |
| B3 | `\bib@field@unknownasdata` digested-vs-raw field value. | **Medium** | ✅ RESOLVED `07c928d370`: now prefers `current_entry_field` (digested, Perl `currentBibEntryField`), falls back to raw. While fixing, found a WORSE bug: unknown fields (and `\bib@@origbibentry`) emitted EMPTY `<ltx:bib-data/>` — value set in `after_digest` (too late) as `Stored::Tokens` (which the constructor's `#prop` CONTENT-insertion silently drops; only `Stored::String` works in content). Fixed via `properties` + `Stored::String`. Also aligned `pretty_print` to Perl's `=`-aligned shape. |
| B4 | `recase_title` word-regex narrower than Perl `\w` for non-ASCII + `\<digit>+` escapes. | Low | Acceptable for ASCII titles. |
| B5 | `recase_title` math-group splits at first `$` (no backslash-escape recognition like Perl `Text::Balanced::extract_delimited`). | Very low | Acceptable. |
| B6 | `TitleCaseMode::parse` unknown values fall back to `Capitalize1`; Perl propagates unknown string as-is (no match → leaves alone). | Very low | Acceptable; stricter is safer. |

**Note on Perl `ucfirst`**: `Capitalize1` mode calls Perl `ucfirst` —
this uppercases the first char and leaves the REST of the word
untouched (it does NOT downcase). The Perl docstring at L286
("downcase all, then Capitalize 1st word") is misleading; the
implementation does not downcase. Rust matches Perl-actual:
`Capitalize1("ON THE…") = "ON the…"`, not `"On the…"`.

**Missing entirely (Phase 4+ work):**

- ~~Public `current_bib_key()` helper (Perl `currentBibKey`, L192).~~ Landed.
- ~~Orchestration: `\ProcessBibTeXEntry` (L111), `\bibentry@prepare`
  DefPrimitive (L114), `\bibentry@create` DefPrimitive (L135).~~ Landed.
- `\the@lx@xmarg@ID` (L173) — still missing.
- ~~Environments: `{bibtex@bibliography}` (L175), `{bib@entry}` (L185).~~
  Both landed in Phase 7 + Phase 8.

**Phase 8 — Pre::BibTeX parser (landed 2026-05-16):**
The low-level `.bib` file parser
(`LaTeXML::Pre::BibTeX` + `LaTeXML::Pre::BibTeX::Entry`, total 506
Perl lines) is now ported to `latexml_engine/src/pre_bibtex.rs`
(~700 Rust lines, 17 unit tests + 1 e2e test). The CLI binary
auto-detects `.bib` extensions and `literal:@` sources and routes
them through `DigestionMode::BibTeX`; converter preloads
`BibTeX.pool` instead of `TeX.pool`; `core_interface.rs::digest`
drains the gullet mouth via `PreBibTeX::new_from_gullet`, registers
entries in the thread-local `BIB_ENTRIES` map, and pushes back a
`literal:\begin{bibtex@bibliography}…\end{bibtex@bibliography}`
wrapper. See `latexml_oxide/tests/55_bibtex.rs` for the end-to-end
fixture.
- Default entry handlers: `\bib@entry@default@prepare` (L207),
  `\bib@entry@default@complete` (L210).
- 12 per-entry `@prepare` + 4 `@complete` + 9 `@alias` (article,
  book, booklet, inbook, incollection, inproceedings, manual,
  thesis, mastersthesis, phdthesis, proceedings, techreport, report,
  unpublished, online/electronic/www/webpage→website,
  conference→inproceedings).
- ~30 per-(type,field) field handlers (inbook×8, incollection×8,
  inproceedings×11, proceedings×1, article×1).
- ~28 `\bib@field@default@<field>` handlers (L553-783).
- `\bib@@pages` constructor (L670); `processIdentifier` helper
  (L784).
- Phase 5: `\bib@synthesize@mr`/`zbl`, `\bib@@mr`/`@@zbl`/`@@origbibentry`
  (L803-870).

**Phase 6 line was wrong** — there's no separate special-character
phase in the Perl source. `splitWords` + name constructors at
L920-953 are already covered by Phase 1 + Phase 2. See the
"Phases" section below for the corrected layout.

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

**Status (2026-05-15): complete except `ProcessBibTeXEntry` / `currentBibKey`** —
helpers + name parser + `bibAddToContainer` shipped through
commits `977426ea81` (initial), `e67c928912` (name list),
`dc1c99872e` (bibAddToContainer).

| Perl helper | Purpose | Rust target | Status |
|---|---|---|---|
| `currentBibEntry` | Look up the entry currently being processed | `latexml_engine::bibtex::current_entry()` | ✓ shipped |
| `currentBibEntryField(field)` | Get a field's *processed* token value | `current_entry_field(name)` | ✓ shipped |
| `currentBibEntryRawField(field)` | Get a field's *raw* string value | `current_entry_raw_field(name)` | ✓ shipped |
| `currentBibKey` (L192) | Get the current normalized bibkey | (only `CURRENT_ENTRY_KEY` internal; no public helper) | ⚠ no `current_bib_key()` accessor — add when first needed in Phase 4 |
| `copyCrossrefFields(@fields)` | Pull listed fields from the crossref'd parent entry | `copy_crossref_fields(&[&str])` | ✓ shipped (handles missing crossref + self-loop). Divergence B2: uses RAW crossref key, Perl uses DIGESTED. |
| `bibAddToContainer(doc, tag, data, %attr)` | Insert into a `<ltx:bib-related>` container, deduplicating by tag+attrs | `bib_add_to_container(doc, tag, data, attrs)` | ✓ shipped (commit `dc1c99872e`) |
| `processBibNameList(string)` | Parse "Smith, John and Doe, Jane" into a list of name tokens | `process_bib_name_list(s) -> BibNameList` | ✓ shipped (commit `e67c928912`) |
| `NormalizeBibKey` | Already in `latexml_core::common::cleaners.rs:125` | reused | ✓ existed |
| `CleanBibKey` | Already in `latexml_core::common::cleaners.rs:119` | reused | ✓ existed |
| `ProcessBibTeXEntry` (orchestrator, L111) | Drives entry prepare/create cycle | top-level function + DefPrimitives | ⚠ Phase 4 — needs entry-type aliases first |

**BibEntry storage strategy chosen**: thread-local registry
(`HashMap<NormalizeBibKey(key), Rc<RefCell<BibEntry>>>`) + separate
`Option<String>` "current entry" pointer. Avoids threading a
custom `Stored::BibEntry` variant through the dump pipeline, since
BibEntries don't round-trip dumps (created and consumed within
one conversion).

7 unit tests cover round-trips, case-insensitive key lookup,
crossref copy with missing/self-loop edges, and outside-block
None behaviour.

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

**Status (2026-05-15): ✓ shipped** in commits `0b2b39c2bf`
(BibTeX-pool wiring into BINDINGS + amsrefs `LoadPool!`) and
`b6b84478e4` (constructors).

Once the foundation is in, port the constructors that move data
from the entry into XML. These build on `bibAddToContainer` and
`current_entry`:

- `\bib@@field {} OptionalKeyVals Digested` (L230) — inserts a
  single field node into the entry. ✓
- `\bib@addto@related {}{} Digested` (L261) — inserts into a
  `<ltx:bib-related>` sub-element with type/role. ✓
- `\bib@@@name{}{}` (L266) — emits `<ltx:bib-name>`. ✓
- `\bib@@@names{}` (L270) — wrapper for a name list. ✓
- `\bib@@names{}{}` (L271) — token-list processor that calls
  `processBibNameList`. ✓
- `\bib@addtype{}` (L235) — conditional type-emit. ✓
- Name-component constructors `\bib@surname`/`\bib@given`/
  `\bib@lineage` (L951-953). ✓

### Phase 3: Title-case logic and field helpers

**Status (2026-05-15): ✓ shipped.** Title-case `recase_title` +
`TitleCaseMode` + 4 fallback bindings. See coverage-audit
divergences B3/B4/B5/B6 above; B3 is the only one worth fixing
before Phase 4.

- `\bib@@title{}{}{}` (L293) — title-case normalization. Five
  modes (asis / capitalize1 / capitalize / uppercase / lowercase).
  Pure Rust helper: `recase_title(text, mode) -> String`. ✓
- `\bib@field@@ignore` (L343), `\bib@field@default@default` (L346),
  `\bib@field@unknownasdata` (L347) — fallback handlers. ✓
  (B3: unknownasdata uses raw chars, Perl uses digested Tokens.)
- `\bib@@booktitle{}{}` (L337) — booktitle shortcut, aliases
  `\bib@@field` NOT `\bib@@title` per Perl L335 comment. ✓

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

### Phase 6 — REMOVED (was misread)

Earlier drafts of this plan listed a "Phase 6: BibTeX
special-character handling" pointing at Perl `L860-955`. On
re-reading the Perl source, that range is **`splitWords` +
name-component constructors**, both of which Phase 1 / Phase 2
already cover. There is no separate diacritic table to port.

## Acceptance criteria

Per phase:
- **Phase 1:** Unit tests covering `current_entry` lookup,
  crossref-copy, NormalizeBibKey↔CleanBibKey round-trip,
  process_bib_name_list parsing edges. No bindings ship yet.
  **Status (2026-05-15)**: 19 unit tests in
  `latexml_engine::bibtex::tests`, including the
  Perl-faithful `et al.` (multi-word ≠ etal marker) edge.
  `bibAddToContainer` unit tests defer to Phase 2 (needs the
  Document-API helper to be in place).
- **Phase 2-3:** A minimal amsrefs paper with one `\bib{key}{article}{
  author={X}, title={Y}, journal={Z}, year={2020}}` produces the
  same `<ltx:bibentry>` XML as Perl LaTeXML (up to known XML
  divergences in `docs/parity/OXIDIZED_DESIGN.md`).
- **Phase 4-5:** All 16 entry types + MR/Zbl synthesis produce
  Perl-equivalent XML. Port the Perl end-to-end tests
  `LaTeXML/t/structure/{bibsect,natbib,crazybib}.tex+.xml`
  (+ `lit.bib` data fixture) into `latexml_oxide/tests/structure/`
  and let the existing `tex_tests!` macro auto-discover them. These
  are the canonical integration tests for the whole BibTeX
  pipeline (TeX → XML); failures at Phase 4-5 will surface
  through those.

## Testing strategy

Two layers run in parallel:

1. **Rust unit tests** (`#[cfg(test)] mod tests` in
   `latexml_engine::bibtex`) directly probe each helper. Perl
   LaTeXML has no equivalent unit-level coverage — Perl tests
   the helpers transitively through end-to-end TeX→XML runs. The
   unit tests catch helper regressions before they propagate up.
2. **End-to-end TeX→XML tests** (port Perl's bibsect/natbib/
   crazybib pairs to `latexml_oxide/tests/structure/`, Phase 4+).
   These are the binding-layer integration check; they validate
   the whole pipeline against Perl's reference output.

Phase 1 ships #1 only. #2 comes online when Def\* bindings exist.

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
- Phase 1: ~300-400 LOC + tests. Foundation; biggest unknowns. ✓ shipped
- Phase 2: ~150 LOC across 5-7 DefConstructors. ✓ shipped
- Phase 3: ~100 LOC; title-case logic is the main piece. ✓ shipped
- Phase 4: ~250 LOC (mostly DefMacro one-liners once helpers exist).
- Phase 5: ~120 LOC.

Total: ~820-920 LOC + tests. Phase 1-3 = ~700 LOC, 30 unit tests.
Phases 4-5 remaining: ~370 LOC. Likely 2-3 more focused sessions.

## What this document is NOT

A binding-by-binding line-by-line translation guide. The point of
the plan is to make the work decomposable — each phase ships
testable code on its own, without leaving the engine in a
half-broken state.

## Source-locator tracking for `.bib` — combine with the `.bst` engine (directive, 2026-05-25)

`--source-map` / `token-locators` currently does **not** locate `.bib` entries.
The BibTeX port parses `.bib` via its own string parser (`BibEntry::parse(&str)`)
plus `\ProcessBibTeXEntry Semiverbatim`, *not* the TeX `read_token` path — so
`.bib` tokens never receive origin handles and `<bibentry>` carries no
`data:sourcepos` (the source table stays empty for a direct `.bib` conversion).
By contrast `.bbl` *does* track, because it is read via the TeX mouth (see
`SOURCE_PROVENANCE.md` §3.1.3; the user-source filter now passes `.bbl`/`.bib`).

**Directive:** locating `.bib` is a natural fit to combine with translating the
`.bst` emulation engine from brucemiller/LaTeXML#1955
(<https://github.com/brucemiller/LaTeXML/pull/1955>). Both touch the same `.bib`
parsing/processing path, so thread byte-offset → `(line, col)` source positions
through the `.bib`/`.bst` parser and attach them to the `bibentry`/field
constructs *as part of that port*, rather than as a separate retrofit.

## Related

- Skeleton: `latexml_engine/src/bibtex.rs`
- amsrefs stub: `latexml_package/src/package/amsrefs_sty.rs`
- Cross-engine audit: `docs/archive/PERL_LOADFORMAT_AUDIT.md` § Engine-wide CS-name diff refresh
- Memory: `memory/wisdom_hub_kernel_dump.md`,
  `memory/wisdom_intarray_fontdimen_storage.md`
- Post-processing consumer: `latexml_post::make_bibliography`
