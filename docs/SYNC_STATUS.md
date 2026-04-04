# Engine Sync Status: Perl vs Rust

> **This is a Perl-to-Rust translation project.** Every ported function, macro, and definition must faithfully reproduce the original Perl semantics, control flow, and edge-case behavior. The Perl source (`LaTeXML/` directory) is the ground truth. Only diverge when explicitly documented in `docs/OXIDIZED_DESIGN.md`.

Updated 2026-04-03. Only lists open gaps & TODOs; completed items live in git history.

**Test inventory:** 407 tests pass (338 integration + 1 post + 39+7+6+15 latexml_post unit tests + 1 post integration). All integration tests at zero structural diff against Perl reference XMLs. 4 tolerated post-processing diffs in `simplemath_post_test`.

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
| base_xmath.rs | MINOR | Matrix/cases fully ported. DefMathLigature ported in plain.rs. Missing: `MathWhatsit()` |

### Phase 1: TeX Primitives (High-Gap)

| File | Status | Open Gaps |
|------|--------|-----------|
| tex_math.rs | OK | `\nonscript`, `\lx@dollar@default` ported. DefMathLigature in plain.rs. `adjustMathstyle` fully ported (recursive helper). |
| tex_box.rs | MINOR | `\leaders/cleaders/xleaders`, `\vrule/\hrule`, `\hbox/vbox/vtop`, SVG functions all ported. Minor: some box dimension edge cases |
| tex_fonts.rs | MINOR | `\fontname`, `\hyphenchar` ported. Ligatures in plain.rs. Missing: `\fontdimen` full array semantics, `getFontDimen()` helper |
| tex_tables.rs | MINOR | `\halign BoxSpecification` fully implemented. Minor: padding CSS classes |

### Phase 2+3: Remaining Primitives + Plain Format

| File | Status | Open Gaps |
|------|--------|-----------|

### Phase 4: LaTeX Chapters (GAPS only)

| File | Status | Open Gaps |
|------|--------|-----------|
| latex_ch4_sectioning_and_toc.rs | MINOR | `\format@title@*`, `\@@compose@title`, `\lx@tag` in base_utilities.rs. Missing: `\@@section` (unused legacy), `LABEL_MAPPING_HOOK` |
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
| rewrite.rs | MINOR | ~95% ported. `Regexp` operator works. Missing: `domToXPath` (TeX string → XPath), `digest_rewrite` helper (not exercised by tests) |
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
| amsmath.sty | MINOR | ~95% ported. Core complete: operators, text, subequations, matrices, align, cfrac, MultiIntegral, options. Minor: cfrac mathstyle tracking |
| listings.sty | MINOR | ~95% ported. caption/title, extendedchars, stringstyle/directivestyle all working. Missing: literate `*` (protected) flag enforcement |
| ntheorem.sty | OK | `\newshadedtheorem` + `\colorbox` shading fully ported |
| caption.sty | OK | DefKeyVal declarations + CAPTION_ value storage ported |

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

**Status (2026-04-03):** 407 pass, 0 fail, 0 ignored.

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
- [x] **P6. Graphics post-processor** — Implemented with `imagesize` crate for PNG/JPEG/GIF dimension reading + ImageMagick CLI (`convert`) for PDF/EPS→PNG format conversion. File copying, path resolution, search paths, graphicspath PI support. Wired into post-processing pipeline between CrossRef and Split.
- [x] **P7. MathML intent attribute** — `intent=":literal"` on all `<math>` elements when ar5iv.sty is preloaded. Auto-detected from `<?latexml package="ar5iv"?>` processing instruction. New builder: `MathML::with_intent_literal(true)`.
- [x] **P8. Plane 1 Unicode mapping** — Ported `LaTeXML::Util::Unicode` plane 1 character mapping. New `latexml_post/src/unicode.rs` module: `unicode_convert()` with all 14 mapping styles (bold, italic, script, fraktur, double-struck, etc.), `unicode_mathvariant()` with full 28-entry normalization table. Integrated into `pmml_token` and `stylize_content`: when plane1=true (default), text is converted to Mathematical Alphanumeric Symbols (U+1D400–U+1D7FF) and mathvariant attribute is cleared. Special character overrides (script B→ℬ, fraktur C→ℭ, double-struck R→ℝ, etc.) faithfully ported. CSS class fallbacks (ltx_font_mathcaligraphic, ltx_font_oldstyle, etc.) preserved per Perl. All-or-nothing conversion semantics match Perl.

### XSLT infrastructure

- [x] Covered by L3 above.

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

### Math parser ambiguity reduction (M-series)

Grammar restructuring tasks to reduce raw parse tree counts. Target: <200 raw trees for any 4-equation `\quad`-separated formula. Ordered by expected impact.

- [x] **M1. Canonical double-script rules** — **INVESTIGATED: NOT THE SOURCE.** Tokens come in source order (start_POSTSUBSCRIPT before start_POSTSUPERSCRIPT for `x_a^b`), so only one derivation path matches per input. The `scripted_factor_r12 postsuperarg` vs `scripted_factor_r11 postsubarg` paths are mutually exclusive by token ordering. The 2^N growth comes from other sources (bigop absorption, PUNCT competition, diffop speculation). No grammar change needed for script rules.

- [x] **M2. Restrict bigop argument absorption** — **INVESTIGATED: NOT FEASIBLE as described.** Changing `term` to `tight_term` breaks nested bigops: `∑∑∑ a_{ij}b_{jk}c_{ki}` becomes unparsable because each `∑` can't absorb the next `∑` (which is a `bigop_application` at `term` level, not `tight_term`). The 2x ambiguity from bigop absorption is handled correctly by semantic pruning. Keeping `term` for now. Bigops absorb only juxtaposed factors (tight_term), not full addop chains (term). `∑ a + b = ∑(a) + b` instead of `∑(a+b)`. The current grammar allows both, with semantic pruning selecting one — but Marpa explores both paths. **Expected impact:** ~2x reduction per bigop by eliminating the over-absorption path. **Risk:** Some formulas like `∑_{i=1}^n f(x_i) Δx` need tight_term absorption to work correctly. The delimited path `scripted_bigop fenced_factor` already handles `∑(a+b)`. **Investigation:** Audit all test cases containing SUMOP/INTOP/BIGOP to verify no regressions. The Perl grammar's `addOpArgs` absorbs `Factor moreOpArgFactors` (multiplicative chains), not full expressions — this change would match Perl more closely.

- [x] **M3. PUNCT-separator disambiguation** — **ALREADY RESOLVED.** Semantic filtering in `list_apply` and `formulae_apply` already eliminates competition: (1) list_apply rejects when any item is relational, (2) formulae_apply rejects when no item is relational. Mixed cases route through `formula relop formula_list` or single-handler paths. No 2x PUNCT duplication remains — remaining tree counts come from bigop absorption (M2 investigated, cannot restrict) and diffop speculation.

- [x] **M4. Diffop grammar-level filtering** — **IMPLEMENTED.** Lexer emits `XDIFFUNK`/`XDIFFID` tokens for "d" content (instead of `UNKNOWN`/`ID`). Grammar uses `diffunk`/`diffid` in diffop rule; these also appear in `factor_base` and `speculative_prefix_apply` as alternatives. Only "d" tokens enter the diffop path — all other UNKNOWNs skip it. **Results:** `unknown_letters_24` 5000→1, `intop_chain` 768→1, `attn_lrate` 5000→240, `attn_warmup_steps` 233→1. `esint_test` passes (d correctly parsed as diffop).

- [x] **M5. Consecutive UNKNOWN coalescing** — **RESOLVED by M4.** The diffop filtering eliminated the Catalan-number growth: `unknown_letters_24` went from 5000 raw trees to 1. The remaining invisible-times chain has only one derivation path (left-recursive). No pre-parser coalescing needed.

- [x] **M6. Script order canonicalization pragma** — **NOT NEEDED.** M1 investigation proved script ordering is not the source of ambiguity (tokens come in source order, only one derivation path per input). No canonicalization needed.

- [x] **M7. `function_call` nonterminal consolidation** — **PARTIALLY CONSOLIDATED.** Moved `apply_delimited` rules (6 rules: function/opfunction/trigfunction × lparen/lbracket) into `applied_func` for chaining support. Removed 6 duplicate scripted variant rules from `tight_term` (scripted_function/opfunction/trigfunction fenced_factor + lparen) — these were in BOTH `tight_term` and `applied_func`, creating 2x ambiguity per site. Kept base function type tight_term rules as "priority boosters" — removing them changes Marpa tree enumeration order. Also: max_consecutive_dupes 64→32, 200ms convergence budget after first unique parse. All 407 tests pass.

- [x] **M8. `fenced_factor` rule audit** — **PARTIALLY AUDITED.** Removed `langle_open expression rangle_close` (subsumed by `langle_open formula rangle_close` since every expression is a formula — 2x ambiguity for `⟨a⟩`). `langle_rel` vs `langle_open` are different lexer tokens — no overlap. `lparen term_list rparen` vs `lparen formula_list rparen` overlap for simple terms but produce different inner structures (term_list_apply vs formula_list_apply) — kept both as the semantic difference is intentional. All 407 tests pass.

- [x] **M9. `tight_opterm` vs `applied_func` interaction** — **RESOLVED via pragmatic.** Root cause: OPFUNCTION/TRIGFUNCTION are `factor`s (line 101), allowing invisible_times to consume them standalone. The `FunctionsPreferWiderAbsorption` pragmatic now rejects N-ary invisible_times chains with bare OPFUNCTION/TRIGFUNCTION in non-terminal positions, forcing prefix_apply absorption. FUNCTION excluded (only absorbs fenced args). Covers both diffd and trig cases: `f(x)*d*x→f(x)*d@(x)`, `2*sin*x→2*sin@(x)`.

- [x] **M10. Target: ≤10 unique parses** — **ACHIEVED.** All test formulas produce ≤10 unique parses after semantic pruning + deduplication. Worst case: `a|a|+b|b|+c|c|` with 10 unique (vertical bar inherent ambiguity). Common formulas: 1-3 unique. Raw tree counts remain high (up to 5000) but converge to ≤10 unique. The `FunctionsPreferWiderAbsorption` pragmatic (M9) further reduces unique counts by eliminating bare-function-in-chain parses.

- [x] **M11. Time-budgeted tree enumeration** — **PARTIALLY DONE.** `max_consecutive_dupes` reduced 64→32. Added 200ms convergence budget after finding first unique parse. These cap post-convergence latency but don't help when the valid parse appears late (e.g., tree #3585 of 3713 for 4-equation bigop formulas). The 5000-tree hard limit and 30s timeout remain. QUADPUNCT token splitting was investigated (43-71% reduction on regression tests) but reverted — too many edge cases at the expression level.

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

### Math parser performance — IN PROGRESS (2026-04-03)

**Problem:** Marpa grammar produces exponentially many parse trees for multi-equation formulas. Three multiplicative ambiguity sources compound, producing 5000+ raw trees for 4-equation `\quad`-separated formulas (800ms parse time in release mode).

**Ambiguity root causes (first principles analysis):**

**A. Script ordering duplication (2^N per formula).**
`scripted_factor_r2` and `scripted_bigop` allow both sub-then-super and super-then-sub orderings via chained intermediate nonterminals. For `x_a^b`: path 1 is `(factor_base postsubarg) postsuperarg` via `scripted_factor_r12 → r2`, path 2 is `(factor_base postsuperarg) postsubarg` via `scripted_factor_r11 → r2`. Both produce identical semantic trees. With N doubly-scripted terms, this creates 2^N derivation paths. For 4 equations of `∑_a^b V_i` (2 scripted terms each, 8 total): 2^8 = 256 duplicate derivations.

**B. Bigop argument absorption ambiguity (2x per bigop).**
For `∑_a^b V_i`, the grammar allows: (a) `bigop_application(∑_a^b, term(V_i))` — bigop absorbs argument, vs (b) `∑_a^b` as standalone statement + `V_i` as separate term joined by invisible-times. Both are valid grammar paths; semantic pruning selects the correct one, but Marpa explores both.

**C. PUNCT-separator competition (2x per separator).**
For `A \quad B`: both `statements(A, punct, B)` via `list_apply` and `formulae(A, punct, B)` via `formulae_apply` match. Semantic filtering resolves (formulae requires relational content, list rejects both-relational), but the grammar generates both derivation paths. Mixed-relational content (one relational, one not) passes BOTH filters.

**D. Diffop speculative parsing (additive overhead).**
`factor += unknown factor_base => diffop_apply` tries differential interpretation for EVERY `UNKNOWN` token followed by a factor. Semantic action rejects ("`d` check failed") but Marpa already explored the path. Adds ~50% pruned trees to bigop formulas.

**E. Consecutive UNKNOWN token explosion (Catalan-number growth).**
For N adjacent UNKNOWN tokens (`b l b l...`), the `tight_term factor => apply_invisible_times` rule creates Catalan(N) derivation paths for binary tree structures. 24 letters → 5000+ raw trees (all pruned since single-letter invisible-times chains are semantically rejected).

**Measured raw tree counts (regression test in `700_unit_parse.rs::parse_tree_count_limits`):**

| Formula pattern | Tokens | Raw trees | Time | Source |
|---|---|---|---|---|
| `V = ∑_a^b V_i` (1 eq) | 13 | 8 | 3ms | baseline |
| `V=∑V_i \quad X=∑X_i` (2 eq) | 27 | 192 | 5ms | mathtools |
| `V=∑V_i \quad X=∑X_i \quad Y=∑Y_i` (3 eq) | 38 | 1792 | 17ms | mathtools |
| `V=∑V_i \quad ... \quad Z=T Z_i` (4 eq) | 49 | 5000+ | 55ms | mathtools |
| `X=∑X_i, X=∑X_i, X=∑X_i, X=∑X_i` (4 eq, comma) | 43 | 3840 | 34ms | sampler |
| `{}^4_{12}C^{5+}_2 \quad ...` (5 pre-scripted) | 63 | 5000+ | 62ms | mathtools |
| `xy+xy+∫xy dx+xy+...` (28 tokens) | 28 | 768 | 6ms | mathtools |
| `blblblbllbblblblblblblbl` (24 UNKNOWN) | 24 | 5000+ | 17ms | mathtools |

Scaling: 1eq→8, 2eq→192 (24x), 3eq→1792 (9.3x), 4eq→5000+ (capped). Super-linear growth from multiplicative ambiguity sources.

**Fixes applied (session 84):**

1. **`formulae` nonterminal split.** Split `formula_list` into `formula_list` (expression-level, `formula_list_apply`) + `formulae` (statement-level, `formulae_apply`). Previously both shared the nonterminal name, creating massive cross-rule ambiguity. Pre-script formula: 5000→159 raw trees (31x reduction), now parses successfully (was 0 semantic trees).

2. **Online deduplication.** `parses.contains(&tree)` check during tree enumeration. Convergence after 64 consecutive non-novel trees (duplicates or pruned). Eliminates redundant 2^N enumeration once all unique parses found.

3. **Regression test.** `700_unit_parse.rs::parse_tree_count_limits` tracks raw tree counts for 8 problematic formulas. Prevents ambiguity regressions from grammar changes.

**Results:** Mathtools test: 5.35s → 3.47s (35% faster). Pre-script formula `{}^4_{12}C^{5+}_2\quad...`: 1.1s → 25ms (44x). Three previously-unparsable formulas now parse correctly.

**Grammar restructuring plan (TODO items M1–M10 below).**

### Preload + option handling — FIXED (2026-04-03)

**Problem:** `--preload=ar5iv.sty` had no effect — `INCLUDE_STYLES` never set to true, raw `.sty` files like `nips_2017.sty` couldn't load.

**Root causes found and fixed (session 85):**

1. **Preloads not loaded.** `Converter::initialize_session` passed only `vec!["TeX.pool"]` to `initialize_singletons`, ignoring user preloads from `CoreOptions::preload`. Fixed: extend preload list with user entries.

2. **Preload options not handled.** `initialize_singletons` used `InputDefinitionOptions::default()` (with `handleoptions: false`) for all preloads. Perl's `initializeState` extracts the extension, sets `handleoptions => true` for `.sty`/`.cls`. Fixed: parse extension and set `handleoptions` accordingly.

3. **Duplicate ar5iv_sty binding.** Both `latexml_package` and `latexml_contrib` had `ar5iv_sty.rs`, with the contrib version (stale) loaded first via `extra_bindings_dispatch`. The stale version didn't call `pass_options`. Fixed: removed `ar5iv_sty` from `latexml_package`, unified in `latexml_contrib` with full `pass_options` implementation.

4. **Missing early stubs.** `\@unknownoptionerror`, `\AtBeginDocument`, `\@addtofilelist` are defined in LaTeX.pool but needed during preload-time `ProcessOptions`/`handleoptions`. Added no-op stubs in `latex_hook.rs` (guarded by `IsDefined!`), overwritten when LaTeX.pool loads.

5. **`\@addtofilelist` guard.** `input_definitions` called `\@addtofilelist` unconditionally when `handleoptions: true`, failing before LaTeX.pool. Now guarded by `lookup_definition` check.

**Result:** `--preload=ar5iv.sty` now correctly sets `INCLUDE_STYLES=true`, enabling raw TeX loading for custom `.sty` files (nips_2017, etc.). Test: 1706.03762 "Attention Is All You Need" processes with natbib, geometry, raw nips_2017.sty.

### ar5iv example parity — 2502.04134

- [ ] **Compare latexml-oxide vs latexmlc output for `arxiv-examples/2502.04134`** (ICLR 2025 paper).

  **Session 86 fixes:**
  1. **`\NewCommandCopy`/`\DeclareCommandCopy`** ported — resolved tcolorbox fatal error (was hitting token limit due to missing L3 kernel command)
  2. **`\@onefilewithoptions`** ported — newer LaTeX kernel hook for package loading
  3. **`\setcitestyle` brace-aware parsing** — old version used naive comma-split that broke `aysep={,}` (inner braces around comma). New version handles nested braces correctly.
  
  **Result:** 72 errors + 1 fatal → **0 errors**. Tcolorbox loads, citations work, sections properly structured.
  
  **Remaining diff (1231 lines):** Perl: 862 HTML lines, Rust: 815 HTML lines. Differences include: XML comments (Rust includes, Perl strips), resource links, attribute ordering, some structural differences in tcolorbox theorem boxes.

### Kernel dump precompilation (E)

- [ ] **E1. Precompile kernel dumps on `cargo build`** — Currently, `latex_dump.rs` and `plain_dump.rs` are no-op stubs. The `build.rs` only ensures the stubs exist; it doesn't run `--init=latex.ltx`. Real dumps are generated manually via `cargo run --release --bin latexml_oxide -- --init=latex.ltx` (saves a 137-entry zero-regression dump). Automating this as a `build.rs` step would ensure the precompiled kernel is always available, significantly reducing startup time for tcolorbox-heavy documents. The dump bypasses runtime TeX loading of latex.ltx/plain.tex, which is critical for packages like tcolorbox that push the 30M token limit during raw loading.

### Permanent ignores (5)
- **ns1–ns5** (52_namespace) — DTD not supported in Rust port.

---

> **Reminder:** Every entry ported from Perl must follow tightly the original semantics and nuances. Read the Perl source, translate precisely, preserve edge cases. The Perl code is the ground truth.
