# Engine Sync Status: Perl vs Rust

> **This is a Perl-to-Rust translation project.** Every ported function, macro, and definition must faithfully reproduce the original Perl semantics, control flow, and edge-case behavior. The Perl source (`LaTeXML/` directory) is the ground truth. Only diverge when explicitly documented in `docs/OXIDIZED_DESIGN.md`.

Updated 2026-04-03. Only lists open gaps & TODOs; completed items live in git history.

**Test inventory:** 391 tests pass (338 integration + 1 post + 39+7+6 latexml_post unit tests). ~298 paired tests at effectively zero-diff (remaining diffs are intentional math parser divergences).

**Production-ready:** Full CorTeX ZIP-to-ZIP pipeline operational. All legacy production options supported:
```
latexml_oxide --whatsin=archive --format=html5 --pmml --mathtex --noinvisibletimes \
  --nodefaultresources --preload=ar5iv.sty --timeout=2700 --log=log.txt \
  --dest=output.zip input.zip
```
Features: clap CLI (30+ options), OmniBus fallback, DefAutoload, \index/\glossary, error recovery, log capture, status messages.

**High-level roadmap:** See [`mini_3_plan.md`](mini_3_plan.md) for the 4-phase strategic plan
(Engine Parity → Package Bindings → Post-Processing → Production).

## Legend
- **OK** = fully synced | **MINOR** = small gaps | **GAPS** = significant missing | **EMPTY** = not ported

**See also:** [`KNOWN_PERL_ERRORS.md`](KNOWN_PERL_ERRORS.md) — upstream Perl issues (not Rust bugs)

---

## Engine Files — Open Gaps Only

Only files with GAPS or significant MINOR issues listed. OK files omitted (see git history).

### Phase 1: Foundation

| File | Status | Open Gaps |
|------|--------|-----------|
| base_parameter_types.rs | GAPS | `DirectoryList`, `CommaList`, `DigestUntil` unported; `Variable` reversion `todo!()` |
| base_utilities.rs | MINOR | Missing: `isDefinable()`, `aligningEnvironment()`, `addClass()`, `SplitTokens()`, `JoinTokens()` |
| base_xmath.rs | MINOR | Matrix/cases fully ported. Missing: `DefMathLigature` rules (cdots/ldots ligatures), `MathWhatsit()` |

### Phase 1: TeX Primitives (High-Gap)

| File | Status | Open Gaps |
|------|--------|-----------|
| tex_math.rs | MINOR | `\nonscript`, `\lx@dollar@default` ported. Missing: `DefMathLigature` (cdots/ldots), `adjustMathstyle` recursive helper |
| tex_box.rs | MINOR | `\leaders/cleaders/xleaders`, `\vrule/\hrule`, `\hbox/vbox/vtop`, SVG functions all ported. Minor: some box dimension edge cases |
| tex_fonts.rs | GAPS | Missing: `\fontname` scaled format, per-font `\hyphenchar`, `getFontDimen()`, 7 ligature defs |
| tex_tables.rs | MINOR | `\halign BoxSpecification` fully implemented. Minor: padding CSS classes |

### Phase 2+3: Remaining Primitives + Plain Format

| File | Status | Open Gaps |
|------|--------|-----------|
| plain.rs | GAPS | Missing: `\alloc@{}{}{}{}{}`, `\@@oalign/@@ooalign`, `\multispan`, `\hglue`, `\lx@hack@bordermatrix` |

### Phase 4: LaTeX Chapters (GAPS only)

| File | Status | Open Gaps |
|------|--------|-----------|
| latex_ch4_sectioning_and_toc.rs | MINOR | `\format@title@*`, `\@@compose@title`, `\lx@tag` in base_utilities.rs. Missing: `\@@section` (unused legacy), `LABEL_MAPPING_HOOK` |
| latex_ch8_defining_commands.rs | OK | `\DeclareMathAccent` fully implemented; `\DeclareFontShape/Family` as proper stubs |
| latex_ch9_marginal_notes.rs | GAPS | 50% |
| latex_ch14_pictures_and_color.rs | GAPS | 30% — picture environment not implemented |

---

## Missing Tag() Calls

| Tag | Perl Source | Notes |
|-----|-------------|-------|
| `Tag('ltx:picture', autoOpen => 0.5, autoClose => 1, ...)` | latex_constructs L4994 | Picture env |

**Completed:** `ltx:figure/table/float` afterClose hooks (BuildPanelsAndID + collapseFloat) fully ported in `latex_ch9_figures_and_tables.rs`.

---

## Cross-Cutting Infrastructure Gaps

1. **`FontDef` parameter type** — Simplified to `FontToken`. Blocks `\fontdimen`, `\hyphenchar` per-font tracking.
2. ~~**`DEFSIZE`**~~ — **FIXED**: Now reads `NOMINAL_FONT_SIZE` from state via `defsize()` function (was static 10.0).

---

## Unported Perl Files

| File | Defs | Priority | Notes |
|------|------|----------|-------|
| `latex_constructs.pool.ltxml` | ~843 | Low | ~92% ported. Missing: picture env, `\@xargdef/yargdef/reargdef` |
| `math_common.pool.ltxml` | 312 | Medium | ~95% ported. Sized delimiters and `\vert` in plain.rs. Missing: `DefMathLigature` rules |
| `Base_Deprecated.pool.ltxml` | 77 | Low | ~16% — deprecated compat shims |
| `AmSTeX.pool.ltxml` | 112 | Low | ~30% |
| `BibTeX.pool.ltxml` | 150 | Low | ~9% |

---

## Core Modules (MINOR+ only)

| Module | Status | Open Gaps |
|--------|--------|-----------|
| gullet.rs | MINOR | `readArg` isolation (type ergonomics) |
| stomach.rs | OK | Mathcode decoding fully implemented (MATH_CLASS_ROLE matches Perl) |
| document.rs | MINOR | XML comment creation (needs libxml2 FFI) |
| alignment.rs | OK | Padding CSS classes and ABSORB_LIMIT guard both implemented |
| rewrite.rs | MINOR | ~90% ported. Missing: `compile_regexp`, `digest_rewrite` |
| pathname.rs | MINOR | Missing: `pathname_make`, `pathname_relative`, `pathname_findall` |
| font.rs | OK | `defsize()` reads NOMINAL_FONT_SIZE from state |

---

## Package.pm — DefFoo Sync Status (dialect.rs)

| DefFoo | Status | Gaps |
|--------|--------|------|
| `DefMacroI` | MINOR | `outer`/`long` not mapped |
| `DefPrimitiveI` | MINOR | Missing `outer`/`long` |
| `DefConstructorI` | MINOR | Missing `outer`/`long`/`attributeForm`; robust alias fallback |
| `DefEnvironmentI` | OK | — |

---

## Rust Error Fixes (9 total — see git history for details)

1. `DefMacro!` double-packing, 2. `Font::merge()` specialize bug, 3. `%\n` not emitted (intentional),
4. `infer_sizer` reversion removed, 5. `METRIC_MAP` italic lookup, 6. `compact_xmdual` implemented,
7. `\lx@dual` keyval extraction, 8. DefMath empty `{}` in tex, 9. `dynamic_mathstyle` in constructors.

---

## Package Bindings (open gaps only)

| Package | Status | Notes |
|---------|--------|-------|
| amsmath.sty | MINOR | ~90% ported. Core complete: operators, text, subequations, matrices, align, cfrac, MultiIntegral, options. Missing: cfrac mathstyle tracking |
| listings.sty | GAPS | Missing: `literate`, `extendedchars`, `directivestyle`/`stringstyle` propagation, `title=`/`caption=` |
| ntheorem.sty | GAPS | Missing: `\colorbox` for shaded theorems |
| caption.sty | MINOR | Missing: KeyVals, CAPTION_ value storage |

All other packages OK: calc, report, appendix, multicol, booktabs, remreset, chngcntr, physics (0 diffs), siunitx (1817 lines), tikz+pgf (7/7 pass), expl3 (37K lines load), babel (6 pass), moderncv (2 pass), beamer (2 pass). txfonts: ~130 symbols ported.

---

## Test Suite Status

**391 pass, 0 fail, 0 ignored** (338 integration + 1 post + 39+7+6 latexml_post unit tests). Effectively zero-diff on all paired tests.

**Permanent ignores:** ns1–ns5 (DTD not supported).

---

## Tikz Test References

XML files in `LaTeXML/t/tikz/` are OUTDATED. Always regenerate fresh Perl output.

Fresh Perl diffs (after stripping tex= and %&#10;):
- 3d-cone: **29 lines** (mostly DESIGN decisions)
- ac-drive: **225 lines** (nested SVG sizing)
- various_colors: **85 lines** (listings + tcolorbox)

### Priority FIX items (shared across tikz tests)

1. **foreignObject transform Y=16.6** — Perl uses fixed 12pt maxy; Rust uses actual height
2. **foreignObject width/height** — `fo_get_size` differs from Perl
3. **Nested minipage/SVG sizing** — `appendNodeBox` vs Perl's `pushContent`
4. **Arrow tip shape** — Different arrowhead path data
5. **`<pagination role="newpage"/>`** — Missing `\newpage` handling
6. **SVG viewBox/width** — Total dimensions differ slightly
7. **Listings escapechar + color** — `escapechar=@` with `\color{red}` inline
8. **Missing `\vspace{2mm}` output** — `\vspace` in vertical mode

---

## Work Plan — Ordered TODO List

Follow this list in order. Work on the first unchecked `[ ]` item. Skip items marked BLOCKED.

**Status (2026-04-02):** 391 pass, 0 fail, 0 ignored.

### Completed infrastructure
- [x] **F. Post-processing pipeline** — `latexml_post` crate (12,300+ lines, 25 modules). MathML Presentation+Content, XSLT via libxslt FFI.
- [x] **G. Codegen: `Until:` parameter type**
- [x] **H. pgfsys pattern system**
- [x] **I. Unified CLI** — `--post --pmml --cmml --keepXMath --stylesheet --format=html5 --dest`

### Post-processing tasks

- [x] **P1. Scan post-processor** — Port `LaTeXML::Post::Scan`. Populates ObjectDB with IDs, labels, titles, parent-child relationships. All handler methods implemented (section, captioned, labelled, anchor, note, bibitem, ref, bibref, glossary, indexmark, declare, rdf). DB entries store text content (not XML node refs) to avoid dangling pointers.
- [x] **P2. CrossRef post-processor** — Port `LaTeXML::Post::CrossRef`. Resolves `\ref{label}` → `<a href="#id">3.3</a>`, `\cite{key}` → `<a href="#bib.bib18">18</a>`. Fills in refs, bibrefs, glossaryrefs, TOC generation, navigation links, fragment IDs, math declaration links. Integrated into unified CLI pipeline: Scan→CrossRef→MathML→XSLT.
- [x] **P3. MakeBibliography post-processor** — Port `LaTeXML::Post::MakeBibliography` (818 lines). Full FMT_SPEC table (article/book/incollection/report/thesis/website/software), citation style detection (numbers/author-year/alpha), getBibliographies (.bib.xml loading), referrer tracking with parent-chain filtering, bibreferrer cross-links, suffix assignment for duplicate author+year, cited-by blocks, META_BLOCK (notes + external links), bibentry/biblist cleanup. Works from both bibentry XML nodes and ObjectDB metadata fallback.
- [x] **P4. Split post-processor** — Port `LaTeXML::Post::Split` (~300 lines). Full implementation: page tree building, recursive naming (id/label/relative strategies), document surgery (node extraction, sibling removal/re-add), TOC generation, navigation distribution, `PostDocument::new_document()` sub-document creation. All split naming strategies supported.
- [x] **P5. Writer post-processor** — Port `LaTeXML::Post::Writer`. TEMPORARY_DOCUMENT_ID removal, HTML vs XML serialization (`as_html` SaveOptions for `toStringHTML` parity), file output with directory creation. Integrated as Processor in the pipeline.

### XSLT infrastructure

- [ ] Covered by L3 above.

### Library improvements (KWARC/rust-libxml, KWARC/rust-libxslt)

- [ ] **L1. Deep clone for `rust-libxml`** — Add `xmlCopyNode` FFI wrapper. Current `Node::clone()` is a reference copy (same ptr). Need `node.deep_copy()` that calls `xmlCopyNode(ptr, 1)` for proper DOM cloning. Required for: Scan storing XML node values, Perl's `cloneNode(1)` pattern. Without this, we store text content (get_content()) instead of XML nodes, losing inline markup in titles and descriptions.
- [ ] **L2. `get_attribute("xml:id")` for `rust-libxml`** — `Node::get_attribute("xml:id")` returns None on some builds. Workaround: `get_property("id")`. The xml: prefix is a built-in namespace that should be handled transparently.
- [x] **L3. Migrate to `rust-libxslt` crate** — Done. Replaced raw FFI with `libxslt = "0.1.2"` crate. Only `exsltRegisterAll()` still uses FFI (not yet in the crate).
- [ ] **L4. Default namespace handling in `rust-libxml`** — `Node::new_child(ns, localname)` always uses the explicit prefix from the Namespace object, even when the element's namespace matches the document's default namespace. This creates `<ltx:ref>` instead of `<ref>` when the default xmlns is already the LaTeXML namespace. Workaround: check if default namespace matches before looking up prefixed namespace.

### ar5iv conversion parity

Target: full support for the ar5iv production command:
```
latexmlc $1 --dest=html/$1.html --css=ar5iv.css --css=ar5iv-fonts.css \
  --preload=ar5iv.sty --path=.../bindings --path=.../supported_originals \
  --format=html5 --pmml --mathtex --noinvisibletimes --timeout=2700 --nocomments
```

- [x] **A1. `--dest`** — destination output path.
- [x] **A2. `--format=html5`** — HTML5 output via XSLT stylesheet.
- [x] **A3. `--pmml`** — Presentation MathML generation.
- [x] **A4. `--nodefaultresources`** — suppress built-in CSS/JS resource copying.
- [x] **A5. `--css=<file>`** — inject additional CSS `<link>` elements into the HTML output. Repeatable. Passed as XSLT parameter.
- [x] **A6. `--preload=<file>`** — preload a `.sty` file before processing. Repeatable. Passed to `CoreOptions::preload`.
- [x] **A7. `--path=<dir>`** — add search paths for finding packages and inputs. Repeatable. Passed to `CoreOptions::search_paths`.
- [x] **A8. `--mathtex`** — wraps PMML in `<m:semantics>` with `<m:annotation encoding='application/x-tex'>` containing the formula's TeX source.
- [x] **A9. `--noinvisibletimes`** — replaces invisible times (U+2062) with zero-width space (U+200B) in MathML. Thread-local flag in presentation.rs.
- [x] **A10. `--timeout=<seconds>`** — conversion timeout via thread-local deadline checked in the digest loop. Fatal error on timeout.
- [x] **A11. `--nocomments`** — omit XML comments from output. Maps to `CoreOptions::include_comments = Some(false)`.
- [x] **A12. `--javascript=<url>`** — inject `<script>` elements. Repeatable. Passed as XSLT parameter `JAVASCRIPT`.
- [x] **A13. `--source=<file>`** — specify source file (overrides positional argument).
- [x] **A14. `--log=<path>`** — write conversion log to file after processing.
- [x] **A15. `--navigationtoc=context`** — navigation TOC. Passed as XSLT parameter `NAVIGATIONTOC`.
- [x] **A16. `--whatsin=directory`** — directory mode: adds source dir to search paths. Auto-detected from trailing `/`.

### Diff reduction tasks

- [x] **D1. Header guessing row headers** — Already working: bold cells get `thead="column"` in `<thead>`.
- [x] **D2. Equation numbering** — Already working: `(1)`, `(2)` tags produced for equation/align envs.
- [x] **D3. Listings escapechar + color** — moredelim style markup ported, escapeinside delimiter registration fixed. various_colors: 85→75 Perl diffs.

### SVG color groups — FIXED (2026-04-01)

**Root causes found and fixed:**
1. Missing combined color macros (`\pgfsys@color@gray`, `\pgfsys@color@cmyk`, `\pgfsys@color@cmy`) — tikz calls `\pgfsetcolor{gray}` which resolves to `\csname pgfsys@color@gray\endcsname` (combined, no @fill/@stroke suffix). We only had `\pgfsys@color@rgb` combined.
2. Whatsit timing: DefConstructor Whatsits created during tikz option processing were lost before document construction. Fix: store hex colors in pgf state (`pgf@svg@fillcolor`, `pgf@svg@strokecolor`), read via properties closure in `\lxSVG@drawpath@unclipped`.

**Result:** dominoes + unit_tests_by_silviu un-ignored. 390 pass, 0 ignored.

### Math parser dedup fix — FIXED (2026-04-02)

`XProps::PartialEq` (derived) compared internal `xmkey`/`id`/`idref` bookkeeping fields, preventing deduplication of structurally identical parse trees. Custom `PartialEq` now skips those fields. `\ltx@count@parses` diagnostic now reports post-dedup count.

**Result:** Parse counts dropped from 32–1280 to 1–3 for all test formulas. `∑∑ f_a(c^a) g_b(c^b)` went from 1024 distinct parses to 1.

### CLI directory creation — FIXED (2026-04-02)

`--dest=html/paper.html` now creates parent directories recursively via `ensure_parent_dir()`. Applied to output file, ZIP archive, and log file paths.

### Permanent ignores (5)
- **ns1–ns5** (52_namespace) — DTD not supported in Rust port.

---

> **Reminder:** Every entry ported from Perl must follow tightly the original semantics and nuances. Read the Perl source, translate precisely, preserve edge cases. The Perl code is the ground truth.
