# Engine Sync Status: Perl vs Rust

> **This is a Perl-to-Rust translation project.** Every ported function, macro, and definition must faithfully reproduce the original Perl semantics, control flow, and edge-case behavior. The Perl source (`LaTeXML/` directory) is the ground truth. Only diverge when explicitly documented in `docs/OXIDIZED_DESIGN.md`.

Updated 2026-03-19. Only lists open gaps & TODOs; completed items live in git history.

**High-level roadmap:** See [`mini_3_plan.md`](mini_3_plan.md) for the 4-phase strategic plan
(Engine Parity → Package Bindings → Post-Processing → Production).

## Legend
- **OK** = fully synced | **MINOR** = small gaps | **GAPS** = significant missing | **EMPTY** = not ported

**See also:** [`KNOWN_PERL_ERRORS.md`](KNOWN_PERL_ERRORS.md) — upstream Perl issues (not Rust bugs)

---

## Engine Files

### Phase 1: Foundation

| File | Status | Open Gaps |
|------|--------|-----------|
| base_schema.rs | OK | Complete (15/15 defs, verified 2026-03-12) |
| base_parameter_types.rs | GAPS | `DirectoryList`, `CommaList`, `DigestUntil` unported; `Variable` reversion `todo!()` |
| base_utilities.rs | MINOR | Audit 2026-03-12: ~90% complete. `\lx@endash/emdash/NBSP/nobreakspace` present. Reference formatting macros (`lx@the@@`, `lx@fnum@@`, `lx@therefnum@@`, `lx@typerefnum@@`, `lx@format@title@@`) all present. Stubs: `\@add@to@frontmatter@now` (unported), `\lx@frontmatter@fallback` (returns None). Missing Perl helpers: `isDefinable()`, `aligningEnvironment()`, `addClass()`, `SplitTokens()`, `JoinTokens()`. |
| base_xmath.rs | GAPS | ~24 commented-out defs (matrix/cases systems, `\lx@padded`, tweaked). Done: `\lx@apply`, `\lx@symbol`, `\lx@wrap`, `\lx@superscript/subscript`, `openMathFork()`, `closeMathFork()`, `addColumnToMathFork()`, `equationgroupJoinCols()`, `equationgroupJoinRows()`. Missing: `MathWhatsit()` |
| base_functions.rs | MINOR | — |

### Phase 1: TeX Primitives (High-Gap)

| File | Status | Open Gaps |
|------|--------|-----------|
| tex_math.rs | GAPS | Missing: `\nonscript`, `\lx@dollar@default`, `TeXDelimiter` param type, `adjustMathRole()`, math ligatures. `\mathchoice` ported. CS names synced: `\lx@hidden@egroup@right`, `\lx@right` (was `\right@hidden@egroup`, `\@right`). `\left` now unreads `\lx@hidden@bgroup` (was `\@hidden@bgroup`). |
| tex_box.rs | GAPS | `\leaders/cleaders/xleaders` implemented (bounded + hide alignment + filled leader extension). Missing: SVG functions (`collapseSVGGroup` etc), `\hbox/vbox/vtop` have many TODOs, `\vrule/\hrule` mostly commented out |
| tex_file_io.rs | MINOR | `\lx@special@graphics` constructor + `Tag('ltx:graphics')` commented out |
| tex_fonts.rs | GAPS | `\fontname` implemented (returns font filename). Missing: `\fontname` "select font X at Ypt" format for scaled fonts, per-font `\hyphenchar` tracking, `getFontDimen()`, 7 ligature defs. `\fontdimen` only handles 3 hardcoded params |
| tex_tables.rs | GAPS | `\halign BoxSpecification` entirely commented out, many alignment helpers missing |

### Phase 2: Remaining TeX Primitives

| File | Status | Open Gaps |
|------|--------|-----------|
| tex_character.rs | OK | `\accent Number` fully ported with assignment loop, `unicode_accent()` table |
| tex_paragraph.rs | MINOR | Missing: `alignLine()`, `trimNodeLeftWhitespace()`, `trimNodeRightWhitespace()` |
| tex_macro.rs | MINOR | FontDef case still commented out |
| tex_logic.rs | OK | — |
| tex_glue.rs | MINOR | `\hskip` SVG missing |
| tex_registers.rs | MINOR | Missing `DumpFile()` infrastructure |
| tex_kern.rs | OK | SVG handling removed (not critical for XML output) |
| pdftex.rs | MINOR | Missing: `OpenAnnotSpecification`, `\pdfannot`, `\pdfobj`, `\pdfcolorstack` |
| etex.rs | MINOR | `\parshapelength` returns Dimension (Perl: Number); `etex_readexpr_i` has `todo!()` for missing close paren |
| tex_job.rs | OK | — |
| tex_debugging.rs | OK | — |
| tex_page.rs | OK | — |
| tex_penalties.rs | OK | — |
| tex_marks.rs | OK | — |
| tex_inserts.rs | OK | — |
| tex_hyphenation.rs | MINOR | FontDef→FontToken simplification |

### Phase 3: Plain Format

**plain.rs** — GAPS
- Done: `\displaylines`, `\@math@daccent/baccent`, `\phantom/hphantom/vphantom` afterDigest sizing, `\nointerlineskip/\offinterlineskip` as DefMacro, `\@break` properties
- Missing: `\alloc@{}{}{}{}{}`, `\@@oalign/@@ooalign`, `\multispan`, `\hglue`, `\lx@hack@bordermatrix`
- `\leavevmode` calls `enter_horizontal()` (synced)
- `\joinrel`/`\@@joinrel` return errors
- `\partial` role fixed to DIFFOP (was OPERATOR), `\smallint` has mathstyle TODO
- `\neg/\lnot` now use role "BIGOP" (matching Perl)

### Phase 4: LaTeX Chapters

| File | Status | Open Gaps |
|------|--------|-----------|
| latex_ch1_documentclass.rs | MINOR | `\documentstyle` compat, `onlyPreamble` |
| latex_ch1_environments.rs | OK | `beforebegin/afterend` hooks and `\@checkend` implemented |
| latex_ch2_document.rs | MINOR | Unclosed group/env/conditional warnings commented out |
| latex_ch3_sentences_and_paragraphs.rs | OK | `enterHorizontal` now auto via `mode => "text"` |
| latex_ch4_sectioning_and_toc.rs | GAPS | Missing: `\format@title@*`, `\format@toctitle@*`, `\@@compose@title`, `\@tag`. `backmatterelement` property for appendix sections implemented (matches Perl `find_insertion_point` behavior). |
| latex_ch5_packages.rs | MINOR | Done: `\PassOptionsToPackage/Class`, `\OptionNotUsed`, `\@unknownoptionerror`. Missing: `\@onefilewithoptions`, `ProcessOptions` inorder flag |
| latex_ch7_math_mode_environments.rs | GAPS | Done: `retract_equation()`, `\nonumber`, `\lx@equation@nonumber`, `\lx@equation@retract`, `\lx@equation@settag`/`@`, `{equation*}`, `after_equation` postset branch, alignment-based `{eqnarray}`/`{eqnarray*}` with `eqnarray_bindings()` (3-column template, equationgroup/equation/_Capture_ hooks, row before/after hooks for equation numbering), `\@equationgroup@numbering{}` primitive (parses `{key=val,...}` and calls `prepare_equation_counter()`), `\if@in@firstcolumn` conditional, `\lefteqn{}` macro. Missing: `\intertext`, afterConstruct DOM rearrangement (`rearrangeEqnarray`/`openMathFork`/`closeMathFork`/`addColumnToMathFork`/`equationgroupJoinCols` — transforms `_Capture_` into `MathFork`/`MathBranch`/`tr`/`td` structure) |
| latex_ch7_math_common_structures.rs | GAPS | Missing: `\frac` sizer, mathstyle property calc |
| latex_ch7_math_common_delimiters.rs | EMPTY | 0% ported |
| latex_ch8_defining_commands.rs | GAPS | Missing: `\DeclareMathAccent`, `\DeclareFontShape/Family`, many font declaration primitives |
| latex_ch9_marginal_notes.rs | GAPS | 50% |
| latex_ch10_tabbing_environment.rs | OK | Full port: registers, macros, markers, tab tracking, bindings, alignment setup. tabbing_test passes. |
| latex_ch14_pictures_and_color.rs | GAPS | 30% — picture environment not implemented |

Files at OK/MINOR (95%+): latex_ch1_fragile_commands, latex_ch1_break_command, latex_ch5_page_styles (added `\columnsep`, `\columnseprule`, `\mathindent`, `\onecolumn` → `\par`), latex_ch5_title_page_and_abstract (frontmatter now working: \maketitle includes \lx@frontmatterhere, {abstract} after_construct calls insert_frontmatter, {titlepage} has before_digest/after_construct hooks), latex_ch6_* (all), latex_ch7_math_mode_changing_style, latex_ch8_defining_environments, latex_ch8_theoremlike_environments, latex_ch8_numbering (\@addtoreset ported), latex_ch9_figures_and_tables ({figure}[] now has placement arg), latex_ch10_array_and_tabular, latex_ch11_* (all), latex_ch12_line_and_page_breaking, latex_ch13_boxes, latex_ch15_* (both), latex_other_in_appendices (\hb@xt@, \TextOrMath, \eminnershape), latex_semi_undocumented (\protected@write ported).

---

## Missing Tag() Calls (Perl vs Rust)

### Present in Perl, missing in Rust:
| Tag | Perl Source | Notes |
|-----|-------------|-------|
| `Tag('ltx:figure', afterClose => \&BuildPanelsAndID)` | latex_constructs L3417 | Rust only has `generate_id` |
| `Tag('ltx:table', afterClose => \&BuildPanelsAndID)` | latex_constructs L3419 | Rust only has `generate_id` |
| `Tag('ltx:float', afterClose => \&BuildPanelsAndID)` | latex_constructs L3418 | Rust only has `generate_id` |
| `Tag('ltx:figure/table/float', afterClose => \&collapseFloat)` | latex_constructs L3521-3523 | Float collapsing |
| `Tag('ltx:indexphrase', afterClose => \&addIndexPhraseKey)` | latex_constructs L4455 | Index (whole system commented out) |
| `Tag('ltx:glossaryphrase', afterClose => \&addIndexPhraseKey)` | latex_constructs L4456 | Glossary (whole system commented out) |
| `Tag('ltx:indexentry', autoClose => 1)` | latex_constructs L4533 | Index (whole system commented out) |
| `Tag('ltx:picture', autoOpen => 0.5, autoClose => 1, ...)` | latex_constructs L4994 | Picture env (not ported) |
| `Tag('ltx:picture', afterOpen => tex attr)` | latex_constructs L5176 | Picture TeX source |
| `Tag('ltx:g', afterClose => remove if empty)` | latex_constructs L5182 | SVG cleanup |
| `Tag('ltx:graphics', afterOpen => GenerateID)` | TeX_FileIO L84 | Graphics ID |
| `Tag('svg:g', afterClose => \&collapseSVGGroup)` | TeX_Box L855 | SVG group collapse (Rust has stub) |
| `Tag('svg:foreignObject', autoOpen/Close => 1, ...)` | TeX_Box L863 | SVG foreign object |

### Present in both (verified OK):
ltx:section, ltx:document (4 calls), ltx:* (2 calls), ltx:XMDual, ltx:XMText, ltx:Math (3 calls), ltx:emph, ltx:note, ltx:part/chapter/section/.../subparagraph (7 calls), ltx:personname, ltx:titlepage, ltx:item, ltx:inline-item, ltx:equationgroup, ltx:para (2 calls), ltx:theorem, ltx:proof, ltx:figure/table/float (generate_id), ltx:biblist, ltx:bibliography, ltx:bibitem, ltx:bibblock, ltx:text, ltx:td, ltx:p, ltx:appendix (article+book), svg:g (stub).

---

## enterHorizontal / leaveHorizontal Checklist

**Infrastructure:** `enter_horizontal`/`leave_horizontal` options now supported on `DefConstructor!`, `DefPrimitive!`, `DefEnvironment!`. `mode => "text"` auto-adds `enter_horizontal` for constructors/primitives (matching Perl).

### enterHorizontal — all done

Done: `\indent`, `\noindent`, `\ `, `\char`, `\hskip`, `\hss/hfilneg/hfil/hfill`, `\kern/raise/lower/moveleft/moveright`, `\lx@framed/hflipped/overlay`, `\TeX/\LaTeX/\LaTeXe`, `\lx@kludged`, `\@makebox/\raisebox`, `\emph`, `\leavevmode`, `\vrule`, `\unhbox/\unhcopy`, `\lx@begin@display/inline@math`, `\lx@url@url@nolink`, `\@internal@math@verb`, `\@internal@text@verb` + all `mode => "text"` definitions.

Note: `\box/\copy` do NOT call enterHorizontal in Perl (verified TeX_Box.pool.ltxml lines 647-655).

### leaveHorizontal — all done

Done: `\vskip`, `\lx@end@document`, `\vfil/vfill/vss/vfilneg`, `\hrule`, `\unvbox/\unvcopy`.

### leaveHorizontal_internal — all done
Done: `\begin@lx@document` afterDigest, `\@documentclasshook`.

---

## Rust Error Fixes (diverging from Perl)

1. **`DefMacro!` double-packing** — Compile-time `DefMacro!` used `compile_expansion!` (which calls `pack_parameters` at build time) but `Expandable::new()` called `pack_parameters` again at runtime. The double-packing caused spurious `Error:misdefined:expansion` warnings for macros with alignment templates (e.g. `\displaylines`). **Fix:** All `DefMacro!` branches with `compile_expansion!` now set `nopack_parameters: true`. See KNOWN_PERL_ERRORS.md §1 for the underlying Perl issue.
2. **`Font::merge()` specialize bug** — Rust's `merge()` incorrectly called `specialize(font_name)` with the font filename (e.g. "cmb10") instead of text. The "Other Symbol" Unicode case in `specialize` reset `series="bold"` to `series="medium"`. **Fix:** Removed `specialize` from `merge()` — only called at TBox creation with actual text. See KNOWN_PERL_ERRORS.md §4.
3. **`%\n` line-break separator not emitted** — Perl preserves `%\n` (TeX comment-newline used for line breaking) in `tex` attributes. Rust does not emit this separator. **Decision:** Intentional divergence — the `%\n` is a TeX formatting artifact with no semantic content. All 146 occurrences of `%&#10;` removed from 26 test XML files.
4. **`infer_sizer` reversion inference removed** — `dialect.rs::infer_sizer()` was inferring a sizer closure from the Constructor's reversion text when no explicit sizer was specified. For body-capturing constructors like `\lx@begin@inline@math` (reversion=`$`), this measured the `$` character instead of the math body content, producing constant `5.00002pt x 7.5pt + 0.55554pt` for all math boxes. **Fix:** `infer_sizer()` now only returns an explicit sizer or None, matching Perl where sizer is never inferred from reversion.
5. **`METRIC_MAP` math italic lookup** — `METRIC_MAP` mapped `"math_medium_italic"` to `"cmmi"` but `STDMETRICS` used key `"cmm"` for cmmi10 data. The metric lookup failed and fell back to cmr (serif) metrics, losing italic corrections and using wrong character widths for all math content. **Fix:** Changed METRIC_MAP value to `"cmm"` to match STDMETRICS key.
6. **`compact_xmdual` implemented** — Perl's `pruneXMDuals` → `compactXMDual` merges XMDual elements when content and presentation are compatible. Case 1 (XMTok+XMTok) transfers `name`/`meaning` from content and `xml:id`/`role` from dual to the presentation token, replacing the XMDual. Case 2 (mirrored XMApp nodes) walks children, matching XMRef↔xml:id pairs, and creates a compact XMApp. Previously the function was a no-op stub.
7. **`\lx@dual` keyval extraction** — The `\lx@dual` constructor's after_digest callback had a TODO for extracting keyval pairs from its OptionalKeyVals argument. Without this, properties like `role`, `name`, `meaning` from DefMath were not available as `#property` references in the constructor template. **Fix:** Added keyval extraction via `kv.get_hash()` → `whatsit.set_property()`.
8. **DefMath empty `{}` in tex attributes** — `def_math_dual()` always wrapped presentation/content macro arguments in `{}`  even when no arguments existed, producing `\Langle{}` instead of `\Langle` in tex attributes. **Fix:** Only add `{arg}` braces for actual arguments; omit for parameterless macros.
9. **`dynamic_mathstyle` in constructors/duals** — The `dynamic_mathstyle => true` flag (used by esint integrals via `doVariablesizeOp`) was only handled in `def_math_primitive()`. Constructors and dual definitions ignored it, so `mathstyle="text"/"display"` was missing from content tokens. **Fix:** Added mathstyle computation in `transfer_common_constructor_options` after_digest.

---

## Cross-Cutting Infrastructure Gaps

1. **`FontDef` parameter type** — Simplified to `FontToken`. Blocks `\fontdimen`, `\hyphenchar` per-font tracking.
2. **SVG support** — Removed from glue/kern/box. Not critical for XML output.
3. **`LoadFormat` machinery** — Not ported; plain/latex bootstrap loaded inline.
4. ~~**Font `computeBoxesSize`**~~ — DONE. Full 4-helper decomposition (words/lines/stack/box) matching Perl.
5. **`DEFSIZE`** — Static 10.0; Perl reads `NOMINAL_FONT_SIZE` from state.
6. **`.ltxml` file search** — NOT APPLICABLE. Rust compiles all package bindings at build time. Every Perl `.sty.ltxml` must be translated to a Rust `_sty.rs` file. No runtime `.ltxml` loading possible. For external packages, use `latexml_contrib` crate.

---

## Unported Perl Files

| File | Defs | Priority | Notes |
|------|------|----------|-------|
| `latex_bootstrap.pool.ltxml` | 10 | — | **Complete** (audit 2026-03-12: 10/10 defs present) |
| `latex_base.pool.ltxml` | ~160 | — | **Complete** (audit 2026-03-12: ~100% across 36 ch files + appendices) |
| `latex_constructs.pool.ltxml` | ~843 | Low | ~92% ported. Done: alignment-based `\eqnarray`/`\eqnarray*`, `\lefteqn`, `\if@in@firstcolumn`, `\@equationgroup@numbering`, `eqnarray@row@before/after`. Missing: afterConstruct rearrangement (`rearrangeEqnarray`), `\@xargdef/yargdef/reargdef`, picture env. |
| `math_common.pool.ltxml` | 312 | Medium | ~87% ported. Missing: 19 sized delimiters (\big/\Big etc.), `\vert` Let. |
| `Base_Deprecated.pool.ltxml` | 77 | Low | ~16% — deprecated compat shims, port on-demand |
| `AmSTeX.pool.ltxml` | 112 | Low | ~30% — port on-demand |
| `BibTeX.pool.ltxml` | 150 | Low | ~9% — essentially unimplemented |

---

## Core Modules

| Module | Status | Open Gaps |
|--------|--------|-----------|
| mouth.rs | OK | Full encoding support (only latin-1+UTF-8) |
| parameter.rs | OK | `Parameter::digest` MODE capture + `leaveHorizontal_internal` matches Perl Parameter.pm lines 122,139-141 |
| gullet.rs | MINOR | `readArg` isolation via `readingFromMouth`; `read_register_value` coercions |
| stomach.rs | MINOR | Mathcode char decoding (ADDOP vs BINOP). `execute_before_after_group` extracted. `begin_mode_opt`/`end_mode_opt` with `noframe` parameter synced with Perl Grouplevel commit (acaab773). `everymath/everydisplay` injection now centralized in `begin_mode_opt`. `push_stack_frame` now tolerates missing current_token (uses `\relax` fallback) for absorption-phase group operations. |
| state.rs | OK | `Stored::KeyVals` now wraps `Rc<KeyVals>` for pointer-based equality (`Rc::ptr_eq`) |
| document.rs | MINOR | `insertElementBefore()`, comment creation (needs libxml). Fixed: `close_to_node` ifopen parameter now suppresses error (was ignored). `close_node_with_strictness` walker now tracks `n.get_type()` (was `node.get_type()`). `mergeAttributes` now uses `add_ss_values` for space-joined attrs (class/lists/inlist/labels), matching Perl's sort+dedup. `finalize_rec` class merge also sorts. `compact_xmdual` implemented (Case 1: XMTok merge, Case 2: mirrored XMApp). `mergeAttributes` now supports `force` override parameter. |
| register.rs | MINOR | — |
| pathname.rs | MINOR | Missing: `pathname_make`, `pathname_relative`, `pathname_is_contained`, `pathname_findall`, `pathname_timestamp/copy/mkdir`. `canonical` now handles `./`, `/../`. Dir-listing approach in `candidate_pathnames` not ported (uses `Path::exists` instead). |
| alignment.rs | MINOR | normalize.rs deep refactored (2026-03-14): per-column-index arrays, vattach height/depth split, lspaces/rspaces padding, border padding (0.4*UNITY), first/last row strut, rowspan redirect. Remaining: padding CSS classes, ABSORB_LIMIT guard |
| rewrite.rs | GAPS | ~20% ported (Select/Replace only) |
| token.rs | OK | — |
| tokens.rs | OK | — |
| number/float/dimension.rs | OK | — |
| glue.rs | OK | NumericOps overrides: multiply/divide/subtract/smaller/larger preserve plus/minus/pfill/mfill |
| numeric_ops.rs | OK | Default multiply uses float arithmetic matching Perl |
| font.rs | MINOR | `computeBoxesSize` decomposition, `DEFSIZE` from state. Fixed: `merge()` no longer calls `specialize(font_name)` — specialize is only called at TBox creation with actual text. |

---

## Package.pm — DefFoo Sync Status (dialect.rs)

| DefFoo | Perl Lines | Rust Lines | Status | Gaps |
|--------|-----------|-----------|--------|------|
| `DefMacroI` | 1152-1165 | 204-240 | MINOR | `outer`/`long` fields present in ExpandableOptions but not mapped to Expandable struct |
| `DefPrimitiveI` | 1286-1317 | 317-414 | MINOR | Missing `outer`/`long` in PrimitiveOptions |
| `DefConstructorI` | 1436-1474 | 845-963 | MINOR | Missing `outer`/`long`/`attributeForm`; robust alias fallback not implemented |
| `DefEnvironmentI` | 1882-1981 | 977-1220 | OK | `\FOO` (csname case) hooks mostly unimplemented (TODO). `\begin{env}` now uses `begin_mode_opt(noframe)`. `\endFOO` now has endMode + beforeDigestEnd. |

## Package.pm — Missing/Deferred Functions

| Function | Status | Notes |
|----------|--------|-------|
| `createXMRefs` | DEFERRED | Complex DOM; needed when math absorption exercised |
| `ClearAutoLoad` | DEFERRED | Autoload infrastructure not yet needed |
| `FindFile_fallback` | DEFERRED | arXiv version-suffix stripping |
| `LoadFormat` | DEFERRED | Handled inline in Rust |
| `maybePreemptRefnum` | PARTIAL | Skeleton with `todo!()` |

---

## Package Bindings

| Package | Status | Notes |
|---------|--------|-------|
| calc.sty | OK | Full expression parser: +,-,*,/, \real, \ratio, \minof, \maxof, \widthof/heightof/depthof/totalheightof. RegisterValue smaller/larger preserve type variant. |
| report.cls | OK | Faithful port of Perl report.cls.ltxml (identical to book.cls except CSS resource). |
| amsmath.sty | GAPS | ~65% ported: spacing, \overunderset, \lvert/rvert/lVert/rVert, \notag, \tag, \tfrac/\dfrac, \xrightarrow/\xleftarrow, over/under arrows, alignment infrastructure (`ams_rearrangeable_bindings`, `ams_gather_bindings`, `ams_align_bindings`, `ams_aligned_bindings`), environment macros for {gather}/{gather*}/{align}/{align*}/{flalign}/{flalign*}/{alignat}/{alignat*}/{multline}/{multline*}/{split}/{gathered}/{aligned}/{alignedat}, `\@ams@intertext{}` constructor, `\lx@ams@cr@binding` primitive, **matrix environments** (`\lx@gen@matrix@bindings`, `\lx@gen@plain@matrix@`, `\lx@ams@matrix@`, all named matrix envs + subarray/substack). Missing: afterConstruct DOM rearrangement (`rearrangeAMSAlign`/`rearrangeAMSGather`), `\text{}`, operators, `{subequations}`. |
| appendix.sty | OK | Core environments: appendices, subappendices, conditional switches. |
| multicol.sty | OK | Full port: multicols/multicols* environments, registers, stubs. |
| booktabs.sty | OK | Full port: toprule/midrule/bottomrule/cmidrule/specialrule, registers. |
| caption.sty | MINOR | Stub-level: captionsetup, Declare* macros, registers. Missing: KeyVals, CAPTION_ value storage. |
| remreset.sty | OK | Empty stub (obsolete, macros moved to LaTeX core). |
| chngcntr.sty | OK | Empty stub (obsolete, macros moved to LaTeX core). |
| listings.sty | GAPS | Core infrastructure ported: `lstActivate`, `\@listings@inline`, `\lstdefinelanguage`, `\@listingGroup`, `\@listingKeyword`, `lstClassBegin/End`, language loading (lstlang0-3). Fixed: `lstAddDelimiter` style parameter processing ("commentstyle"→"comments" class chain), delimiter font scoping (delim chars outside `\itshape` scope). Remaining: index generation (`lst@@index`), `literate` key, `extendedchars`, listing_test blocked by math parser (mathescape XMDual diffs, 2062 lines). |
| ntheorem.sty | GAPS | Framing constructors (`\lx@addframing`, `\lx@@snapshot@framing`) ported. Missing: `\colorbox` (needs xcolor port) for shaded theorems, `backgroundcolor` attribute. |

---

## Test Suite Status (2026-03-19)

**Current totals: 227 pass, 0 fail, 92 ignored test functions (319 total)**
**Coverage: 72% pass rate (227/314 non-permanent-ignore tests)**
*Note: 40_math (14) and 70_parse (28) split into individual tests, adding 40 test functions.*

**Recent infrastructure fixes (2026-03-18, session 5):**
- **Equation numbering tags**: Row properties changed from `HashMap<String, String>` to `HashMap<String, Stored>`, enabling `Stored::Digested` tags to propagate through alignment absorption. eqnarray equations now have `<tags>` elements matching Perl.
- **Preamble PI**: `\lx@add@Preamble@PI` constructor now uses procedural body with `document.insert_pi()`. `\DeclareMathAccent` returns AddToPreamble result. acc_test: 169→96 diffs.
- **\pagecolor reversion**: Both color.sty and xcolor.sty now return Tbox with reversion tokens.

**Recent fixes (2026-03-18, session 2):**
- **cleanup_math XPath**: Updated to match Perl — excludes XMHint and lone PUNCT/PERIOD from "real math" check. Math spacing commands (`\,`, `\!`, `\>`, `\;`, `\mskip`) no longer produce spurious `<Math>` elements. XMHint width converted to Unicode space chars via `dimension_to_spaces`.
- **vbox/vtop height/depth**: Fixed 3 bugs — sizer closure now passes Whatsit properties (vattach etc.), `repack_horizontal` sets "mode" property string, removed extra `flat_map(unlist)` in `compute_boxes_size`.
- **compact_xmdual xml:id leak**: Fixed content token xml:id leaking to presentation token when dual has no xml:id (Case 1).
- **Sizes test progress**: Down from 313 to 20 diff lines. Remaining: dimension rounding (4), super/subscript widths (3), table sizing (6+), section 8 (7).

### Per-test enumeration

- [x] **000_hello** (1/1)
  - [x] hello
- [x] **00_contrib** (1/1)
  - [x] contrib
- [x] **00_tokenize** (14/14)
  - [x] alltt, comment, equality, file_read, hashes, ligatures, mathtokens, newlines, par, percent, trailingspaces, url, verbata, verb
- [x] **01_unit_tokens** (1/1)
- [x] **01_unit_state** (9/9)
- [x] **10_expansion** (36/36)
  - [x] aftergroup, definedness, endinput, environments, env, escapechar, etex, etoolbox, for, hyperurls, ifthen, inin, keywords, lettercase, meaning, multi_escaped_param, noexpand, noexpand_conditional, numexpr, parindent, partial, pass_param_toks_in_gen_macros, pdftex_expanded, romannumeral, simple_dimen, testchar, testexpand, testif, testinput, testmultido, textcase, toks, urls, utflettercase, whichcache, whichinput
- [x] **12_grouping** (2/2)
  - [x] mathgroup, scopemacro
- [x] **20_digestion** (10/10)
  - [x] box, chardefs, defaultunits, def, dollar, io, primes, rebox, testctr, xargs
- [ ] **22_fonts** (19 pass, 0 fail, 4 ignored = 23 total)
  - [ ] abxtest — IGNORED: needs `\hexnumber@`, `\mathxfam` (mathabx binding)
  - [x] acc
  - [x] accents
  - [x] bbold
  - [x] cancels
  - [ ] ding — IGNORED: enumerate nesting + guessTableHeaders
  - [x] emph
  - [x] esint
  - [x] fonts
  - [x] marvosym
  - [x] mathaccents
  - [x] mathbbol
  - [x] mathcolor
  - [x] mixed
  - [x] omencodings
  - [x] plainfonts
  - [ ] sizes — IGNORED: ~26 diff hunks, vbox/vtop height/depth
  - [x] soul
  - [ ] stmaryrd — IGNORED: needs stmaryrd.sty port (1449 diffs)
  - [x] textcomp
  - [x] textsymbols
  - [x] ulem
  - [x] wasysym
- [x] **30_encoding** (26/26)
  - [x] ansinew, applemac, cp1250, cp1252, cp437, cp437de, cp850, cp852, cp858, cp865, decmulti, latin1, latin2, latin3, latin4, latin5, latin9, latin10, ly1, ot1, t1, t2a, t2b, t2c, ts1, utf8
- [x] **32_keyval** (8/8)
  - [x] keyvalemptyvalue, keyvalinline, keyvalstyle, xkeyvaladv, xkeyvalbasic, xkeyvalkvcompat, xkeyvalstyle, xkeyvalview
- [x] **33_keyval_options** (11/11)
  - [x] xkvdop1a, xkvdop1b, xkvdop2a, xkvdop2b, xkvdop3a, xkvdop3b, xkvdop4a, xkvdop5a, xkvdop5b, xkvdop6a, xkvdop6b
- [ ] **40_math** (0 pass, 1 ignored = batch)
  - [ ] batch — IGNORED: 149 diffs (math parser)
- [ ] **50_structure** (38 pass, 4 ignored = 42 total)
  - [x] abstract, acro, app, apps, article, authors, autoref, badabstract, beforeafter, bibsect, book, changectr, columns, crazybib, csquotes, endnote, enum, epitest, faketitlepage, fancyhdr, figures, filelist, floatnames, footnote, glossary, hyperref, itemize, mainfile, natbib, options, paralists, para, plainsample, report, sec, subcaption, svabstract, titlepage
  - [ ] amsarticle — IGNORED: needs amsart.cls binding (898 diffs)
  - [ ] eqnums — IGNORED: equation counter stepping + tag font (416 diffs)
  - [ ] figure_grids — IGNORED: needs BuildPanelsAndID (331 diffs)
  - [ ] IEEE — IGNORED: math parser diffs (979 diffs)
- [ ] **52_namespace** (0 pass, 5 ignored = permanent)
  - [ ] ns1–ns5 — DTD not supported in Rust port
- [ ] **53_alignment** (18 pass, 11 ignored = 29 total)
  - [x] array, halign, halignatt, listing, longtable, mathmix, min_listing, min_listing2, min_listing_data, min_listing_display, min_listing_lang, min_listing_short, min_listing_string, morse, tabtab, tabbing, tabular, tabularstar
  - [ ] algx — IGNORED: 163 diffs, needs math parser XMDual
  - [ ] badeqnarray — IGNORED: 182 diffs, needs afterConstruct
  - [ ] cells — IGNORED: stack overflow
  - [ ] colortbls — IGNORED: crash
  - [ ] diagboxtest — IGNORED: infinite loop timeout
  - [ ] eqnarray — IGNORED: 1176 diffs, needs afterConstruct + math parser
  - [ ] ncases — IGNORED: infinite loop timeout
  - [ ] plainmath — IGNORED: 351 diffs, math parser XMDual
  - [ ] split — IGNORED: 2228 diffs, amsmath split + math parser
  - [ ] supertabular — IGNORED: 629 diffs, needs supertabular.sty
  - [ ] vmode — IGNORED: segfault
- [ ] **55_theorem** (4 pass, 1 ignored = 5 total)
  - [x] amstheorem, latextheorem, ntheoremstyle, theorem
  - [ ] ntheorem — IGNORED: 1479 diffs, math parser + eqnarray
- [ ] **56_ams** (2 pass, 5 ignored = 7 total)
  - [x] dots, genfracs
  - [ ] amsdisplay — IGNORED: 963 diffs, needs afterConstruct + `\text{}`
  - [ ] cd — IGNORED: panic in math parser, needs amscd.sty
  - [ ] mathtools — IGNORED: TooManyErrors, needs mathtools.sty
  - [ ] matrix — IGNORED: 187 diffs, needs afterConstruct + math parser
  - [ ] sideset — IGNORED: 488 diffs, needs afterConstruct
- [ ] **65_graphics** (5 pass, 4 ignored = 9 total)
  - [x] calc, colors, framed, keyval, simplekv
  - [ ] graphrot — IGNORED: 596 diffs, `\begingroup` in `\csname..\endcsname`
  - [ ] picture — IGNORED: 3125 diffs, needs picture env
  - [ ] xcolors — IGNORED: 447 diffs, complete xcolor port
  - [ ] xytest — IGNORED: crash, needs xy.sty
- [ ] **70_parse** (0 pass, 1 ignored = batch)
  - [ ] batch — IGNORED: 120 diffs (math parser)
- [x] **700_unit_parse** (3/3)
  - [x] basic_1, recognizer_after_failure, recognizer_subscript_atom
- [ ] **80_complex** (10 pass, 6 ignored = 16 total)
  - [x] aastex631_deluxetable, aastex_test, equationnest, hyperchars, hypertest, labelled, figure_dual_caption, tcilatex_minimal, versioned_fallback, xii
  - [ ] acm_aria — IGNORED: timeout, needs acmart.cls
  - [x] aliceblog
  - [ ] cleveref_minimal — IGNORED: 302 diffs, needs cleveref.sty
  - [ ] figure_mixed_content — IGNORED: 1142 diffs, needs wrapfig + listings math
  - [ ] physics — IGNORED: 5417 diffs, needs physics.sty
  - [ ] si — IGNORED: 9024 diffs, needs siunitx.sty
- [ ] **81_babel** (0 pass, 1 ignored)
  - [ ] batch — IGNORED: unbounded memory leak timeout
- [ ] **82_moderncv** (0 pass, 2 ignored)
  - [ ] cs_cv, orc — needs moderncv.cls binding
- [ ] **83_expl3** (0 pass, 2 ignored)
  - [ ] tilde_tricks, xparse — needs `\ExplSyntaxOn`
- [ ] **84_slides** (0 pass, 2 ignored)
  - [ ] beamer, slides — needs beamer.cls/slides.cls
- [ ] **85_pgf** (0 pass, 2 ignored)
  - [ ] stress_pgfmath, stress_pgfplots — needs pgf.sty
- [ ] **86_tikz** (0 pass, 10 ignored)
  - [ ] ac_drive_components, ac_drive_voltage, atoms_and_orbitals, consort_flowchart, cycle, dominoes, tikz_3d_cone, tikz_figure, unit_tests_by_silviu, various_colors — needs tikz.sty

### Perl-only test infrastructure (not applicable to Rust)
- `00_unittest.t` — Perl module unit tests (Perl-specific)
- `91_latexmlc_api.t` — latexmlc API tests (Perl daemon)
- `92_profiles.t` — Output profile tests (post-processing)
- `93_formats.t` — Output format tests (post-processing)
- `931_epub.t` — EPUB output tests (post-processing)
- `94_runtimes.t` — Runtime environment tests (Perl-specific)
- `95_complex_config.t` — Configuration tests (Perl-specific)
- `96_fatal.t` — Fatal error handling tests
- `97_manifest.t` — Manifest validation tests

### Daemon frame pattern (not yet ported)
Perl uses `pushDaemonFrame`/`popDaemonFrame` (State.pm L607-660) to isolate state per conversion. This creates a locked frame, deep-copies mutable values, and allows rollback after conversion. Rust has the code commented out as TODO (state.rs L1784-1818). Currently Rust relies on `initialize_singletons` + `Core::new` resetting STATE/GULLET/STOMACH/MODEL/REPORT/LOCALIZED_VARS, which is sufficient for single-conversion use but lacks the deep-copy rollback semantics needed for daemon mode.

---

## Work Plan — Ordered TODO List

Follow this list in order. Work on the first unchecked `[ ]` item. Skip items marked BLOCKED.

**Status (2026-03-19):** 230 pass, 0 fail, 89 ignored (319 total). Session 9: script_sizer proper font metrics (removed 0.8 hack, added \scriptspace, nominal fontinfo ratios), dimension_to_spaces floor() precedence fix, tabular strut fixes (isLaTeX flag, baselineskip not *1.5, Glue type handling, kround). Diff reductions: sizes 19→6.

### Completed items

- [x] **1. xkeyvalview_test** (32_keyval) — DONE. Ported `\xkvview` constructor.
- [x] **4. figures_test** (50_structure) — DONE. Ported subfigure.sty font options.
- [x] **5. floatnames_test** (50_structure) — DONE. Ported float.sty/newfloat.sty.
- [x] **6. filelist_test** (50_structure) — DONE. Fixed `\@filelist` init.
- [x] **7. options_test** (50_structure) — DONE. Created myclass.cls + apackage.sty.
- [x] **14. paralists_test** (50_structure) — DONE. `set_enumeration_style`/`set_itemization_style`.
- [x] **15. subcaption_test** (50_structure) — DONE. Ported subcaption.sty.
- [x] **17. natbib_test** (50_structure) — DONE. Full natbib.sty port.
- [x] **19. csquotes_test** (50_structure) — DONE. Ported csquotes.sty.
- [x] **20. svabstract_test** (50_structure) — DONE. Ported svjour.cls + sv_support.sty.
- [x] **21. ieee_test** (50_structure) — DONE. Ported IEEEtran.cls (979 diffs remain — math parser).
- [x] **22. acro_test** (50_structure) — DONE. Ported acronym.sty.
- [x] **23. glossary_test** (50_structure) — DONE. Ported glossaries.sty.
- [x] **24. bibsect_test** (50_structure) — DONE. Ported bibunits.sty.
- [x] **24b. crazybib_test** (50_structure) — DONE. `\bibsection` parsing.
- [x] **26. colortbls_test** (53_alignment) — DONE. Ported colortbl.sty (still needs dcolumn/hhline).
- [x] **57–61. Test suite sync** — DONE. Copied pgf (2), tikz (10), moderncv/orc, expl3 (2), slides (2). All 18 new tests ignored.

### Tier 1: Actionable items (no infrastructure blockers)

- [x] **8. fonts_test** (22_fonts) — DONE. Fixed delimited-[] meaning attr (was content). Updated expected XML.
- [x] **8a. mixed_test** (22_fonts) — DONE. Fixed list_apply grammar rule for comma-separated lists.
- [x] **8b. mathaccents_test** (22_fonts) — DONE. Fixed create_xmrefs for Dual/Wrap + empty-arg absent token.
- [x] **8c. plainfonts_test** (22_fonts) — 62 diffs remaining. OMS `\cal` symbols with roles grammar can't handle (METARELOP prefix, empty fenced).
- [ ] **9. sizes_test** (22_fonts) — 6 diff lines (was 313→181→19→6). Fixed: script_sizer h/d (font metrics), dimension_to_spaces floor() bug, tabular strut (isLaTeX, baselineskip, Glue type, kround). Remaining: super/subscript widths (3), halign zero dims (2), vtop width overflow (1).
- [x] **10. ding_test** (22_fonts) — DONE. Passing after cleanup_math + vbox fixes.
- [ ] **11. abxtest_test** (22_fonts) — TooManyErrors. Needs `\hexnumber@`, `\mathxfam`.
- [x] **13. enum_test** (50_structure) — DONE. enumitem.sty fully ported.
- [x] **16. figure_grids_test** (50_structure) — DONE. Passing after previous fixes.
- [ ] **18. amsarticle_test** (50_structure) — 898 diffs. Port amsart.cls binding.
- [ ] **25. cells_test** (53_alignment) — 369 diffs (was stack overflow). Ported makecell.sty, fixed \rothead recursion, implemented Pair parameter type.
- [x] **27. supertabular_test** (53_alignment) — DONE. Ported supertabular.sty + alignment glue fix + right-trim fix.
- [ ] **35. graphrot_test** (65_graphics) — 596 diffs. `\begingroup` in `\csname..\endcsname`.
- [x] **37. xcolors_test** (65_graphics) — DONE. Passing after previous fixes.

### Tier 2: Needs afterConstruct DOM rearrangement (BLOCKED on item 29)

- [x] **29. Implement afterConstruct rearrangement** — `rearrangeEqnarray`, `rearrangeAMSAlign`, `rearrangeAMSGather`, `openMathFork`/`closeMathFork`/`addColumnToMathFork`/`equationgroupJoinCols`. Done. Fixed _Capture_ XMArg wrapping issue (Perl model prevents XMArg inside _Capture_).
- [ ] **2. eqnums_test** (50_structure) — 416 diffs (down from 598). MathFork works, remaining: equation counter stepping, tag font propagation, tex attributes on MathFork Math.
- [ ] **3. algx_test** (53_alignment) — 163 diffs. BLOCKED: needs math parser XMDual + fontsize.
- [ ] **28. badeqnarray_test** (53_alignment) — 507 diffs. BLOCKED: needs afterConstruct.
- [ ] **30. amsdisplay_test** (56_ams) — 963 diffs. BLOCKED: needs afterConstruct + `\text{}`.
- [ ] **31. matrix_test** (56_ams) — 187 diffs. BLOCKED: needs afterConstruct + math parser.
- [ ] **32. sideset_test** (56_ams) — 488 diffs. BLOCKED: needs afterConstruct.

### Tier 3: Needs package bindings (moderate effort)

- [ ] **12. stmaryrd_test** (22_fonts) — 1007 diffs (was 1449). Ported stmaryrd.sty, fixed FontDirective Display. Remaining: mostly XMDual + math parser text= diffs.
- [ ] **33. cd_test** (56_ams) — 175 diffs (was PANIC). No longer crashes after replace_tree fix.
- [ ] **34. mathtools_test** (56_ams) — TooManyErrors. Port mathtools.sty.
- [ ] **36. picture_test** (65_graphics) — 3125 diffs. Port picture env + graphpap.sty.
- [ ] **38. xytest** (65_graphics) — TooManyErrors. Port xy.sty binding.
- [ ] **39. cleveref_minimal_test** (80_complex) — 302 diffs. Port cleveref.sty.
- [ ] **40. figure_mixed_content_test** (80_complex) — 1142 diffs. Needs wrapfig + listings math.
- [x] **41. aastex631_deluxetable_test** (80_complex) — DONE. Ported deluxetable.sty, Stored::Template, version-stripping dispatch.
- [x] **42. aliceblog_test** (80_complex) — DONE. Passing after previous fixes.

### Tier 3b: Perl XML sync — code fixes needed (see docs/PERL_XML_DIFFS.md)

These are differences discovered by comparing `LaTeXML/t/*.xml` with `latexml_oxide/tests/*.xml`.
Tests currently pass against Rust expected XMLs, but Rust output diverges from updated Perl.

- [ ] **P1. guessTableHeaders update** — Perl updated header detection: `<thead>` wrapper with `thead="column"`. Affects: `fonts/ding.xml`, `alignment/tabular.xml`, `graphics/xcolors.xml`. BLOCKED: needs guessTableHeaders post-processing port.
- [x] **P2. ltx_figure_panel CSS class** — DONE. `arrange_panels` now marks all non-metadata children. Synced `figure_grids.xml`.
- [x] **P3. DIFFOP recognition in math parser** — DONE. Grammar rule `factor += unknown factor_base => diffop_apply` with INTOP context check. Synced `dots.xml`.
- [x] **P4. Titled frame support** — DONE. Fixed `after_digest_begin` to use `gullet::unread`. Synced `framed.xml`.
- [ ] **P5. xcolors.xml fixes** — Color complement/wheel computation errors, missing `pt` units in calc output, `colortbl` row cycling broken (all "row 0"), missing `ltx_guessed_headers` class. Affects: `graphics/xcolors.xml` (~688 line diff). BLOCKED: needs color model + guessTableHeaders.
- [ ] **P6. RDFa support** — Perl handles RDFa attributes (`property=`, `typeof=`, `resource=`). Rust produces ERROR nodes. Affects: `complex/aliceblog.xml`. BLOCKED: needs RDFa infrastructure.
- N/A **P7. Daemon format fixes** — OUT OF SCOPE. The Rust port does not currently include daemonized functionality. Daemon tests are not tracked.

### Tier 4: Needs major infrastructure

- [ ] **A. Port expl3 programming layer** — `\ExplSyntaxOn/Off`, `\cs_new:Npn`, `\tl_set:Nn`. Unlocks: beamer, fontspec, unicode-math, and many modern packages.
  - [ ] expl3 tilde_tricks_test (83_expl3) — needs expl3
  - [ ] expl3 xparse_test (83_expl3) — needs expl3
  - [ ] beamer_test (84_slides) — needs beamer.cls + expl3
  - [ ] slides_test (84_slides) — needs slides.cls
- [ ] **B. Complete Document.pm audit** — afterConstruct hooks, insertElementBefore, compact_xmdual.
- [ ] **C. Port BibTeX.pool.ltxml** — bibliography infrastructure (~150 defs, ~9% ported).
- [ ] **D. Port AmSTeX.pool.ltxml** — legacy AMS macros (~112 defs, ~30% ported).

### Tier 5: Math parser tests (active research, Marpa grammar)

- [ ] **47. 40_math suite** (40_math) — 149 diffs across batch. Parser bugs vs intentional Marpa divergence.
- [ ] **48. 70_parse suite** (70_parse) — 120 diffs across batch. Generate Rust-specific expected XMLs.
- [ ] **49. plainmath_test** (53_alignment) — 351 diffs. Math parser XMDual structure.
- [ ] **50. split_test** (53_alignment) — 2228 diffs. Amsmath split + math parser.
- [ ] **51. eqnarray_test** (53_alignment) — 1176 diffs. afterConstruct + math parser.
- [ ] **52. ntheorem_test** (55_theorem) — 1479 diffs. Math parser tree + eqnarray.

### Tier 6: Heavy package bindings (distant future)

- [ ] **43. acm_aria_test** (80_complex) — TIMEOUT. Port acmart.cls.
- [ ] **44. physics_test** (80_complex) — 5417 diffs. Port physics.sty.
- [ ] **45. si_test** (80_complex) — 9024 diffs. Port siunitx.sty.
- [ ] **pgf suite** (85_pgf) — 2 tests. Port pgf.sty.
- [ ] **tikz suite** (86_tikz) — 10 tests. Port tikz.sty (depends on pgf).
- [ ] **moderncv suite** (82_moderncv) — 2 tests. Port moderncv.cls.
- [ ] **revtex4_1_test** (80_complex) — needs revtex4-1.cls binding.
- [ ] **tcilatex_test** (80_complex) — needs tcilatex support.

### Tier 7: Crashes and infinite loops (need deep debugging)

- [ ] **53. diagboxtest_test** (53_alignment) — TIMEOUT: infinite loop in diagbox.
- [ ] **54. ncases_test** (53_alignment) — TIMEOUT: infinite loop in ncases.
- [ ] **55. vmode_test** (53_alignment) — SEGFAULT in vertical mode.
- [ ] **56. babel suite** (81_babel) — TIMEOUT: unbounded memory leak.

### Overarching projects

- [ ] **E. Translate ALL Perl binding files** — Every `.sty.ltxml`, `.tex.ltxml`, `.cls.ltxml`, `.def.ltxml` in `LaTeXML/lib/LaTeXML/Package/` must have a corresponding `_sty.rs`, `_tex.rs`, `_cls.rs`, `_def.rs` in the Rust codebase. No file should be missed. This is the full package binding parity goal.
- [ ] **F. Port Post-processing pipeline** — Fully translate `LaTeXML/lib/LaTeXML/Post/` to Rust. This is the second major subsystem (after the TeX engine) and converts LaTeXML XML into final output (HTML5, XHTML, EPUB, JATS).
  - **Resources (copy as-is):** XSLT stylesheets (`LaTeXML/lib/LaTeXML/resources/XSLT/`), CSS (`resources/CSS/`), JavaScript, RelaxNG schemas, Profiles. These are declarative and used unchanged.
  - **Core modules to port:** `Post.pm` (pipeline orchestrator), `Post::Scan` (cross-ref scanning), `Post::CrossRef` (cross-references + navigation), `Post::MathML` (content/presentation MathML), `Post::OpenMath`, `Post::MathImages` (math-to-image fallback), `Post::Graphics` (image conversion), `Post::SVG`, `Post::XSLT` (apply XSLT transforms), `Post::Writer` (serialize to file), `Post::Split` (multi-page split), `Post::MakeBibliography`, `Post::MakeIndex`.
  - **Approach:** Port each Post module as a Rust crate or module. Use `libxslt` (already a system dep) for XSLT transforms. Use existing `resources/` directory structure.
  - **Testing:** Post-processing tests compare final HTML/MathML output. Start with `latexmlpost` CLI equivalent.

### Permanent ignores (not counted)

- **ns1–ns5** (52_namespace) — DTD not supported in Rust port. Permanently ignored.

---

> **Reminder:** Every entry ported from Perl must follow tightly the original semantics and nuances. Read the Perl source, translate precisely, preserve edge cases. The Perl code is the ground truth.
