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

## Test Suite Status (2026-03-20)

**Current totals: 245 pass, 0 fail, 74 ignored test functions (319 total)**
**Coverage: 79% pass rate (245/314 non-permanent-ignore tests)**
*Note: 40_math (14) and 70_parse (28) split into individual tests, adding 40 test functions.*

**Recent fixes (2026-03-20, session 12):**
- **Per-size font metrics**: Added cmm7/cmm5 entries to STDMETRICS (from cmmi7.tfm/cmmi5.tfm). Script/scriptscript style characters now use correct design-size metrics instead of always falling back to cmmi10. sizes_test: 4→1 diffs (3 script width diffs fixed).
- **Matrix delimiter absorption**: `get_value_digested()` for left/right keyvals in matrix/cases properties. matrix_test: 164→2 diffs.
- **Smallmatrix Perl typo**: Matched Perl's `atameaning` typo (not `datameaning`) to avoid XMDual wrapping.
- **\framebox math mode**: Added `?#mathframe` conditional template, IN_MATH detection, isMath child walk. terms_test: 12→11 diffs.
- **\boxed**: Ported from amsmath (boxed@math → XMArg enclose='box', boxed@text → Math framed='rectangle').
- **DefMath parity audit**: Ported `\And`, `\varlimsup/inf`, `\varinjlim`, `\varprojlim`, `\intop`, `\ointop`, `\iint/iiint/iiiint/idotsint`, `\varGamma...\varOmega` (11 italic Greeks), `\implies/\impliedby`, `\mod/\pod/\pmod/\bmod`, misc stubs.

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
- [ ] **50_structure** (39 pass, 3 ignored = 42 total)
  - [x] abstract, acro, app, apps, article, authors, autoref, badabstract, beforeafter, bibsect, book, changectr, columns, crazybib, csquotes, endnote, enum, epitest, faketitlepage, fancyhdr, figures, filelist, floatnames, footnote, glossary, hyperref, itemize, mainfile, natbib, options, paralists, para, plainsample, report, sec, subcaption, svabstract, titlepage
  - [x] amsarticle
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
- [ ] **56_ams** (3 pass, 4 ignored = 7 total)
  - [x] dots, genfracs
  - [x] amsdisplay
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

**Status (2026-03-21):** 255 pass, 0 fail, 66 ignored (321 total). Session 17 (32 commits): get_arg(0)→panic. Diaghead picture+line+g. {rotatebox} env. Multirow DefMacro+rowspan+vattach. Grammar: postfix_apply, elideop-as-term. colortbl overhaul (@ tokenization, xcolor stubs, overhang args, rowcolor propagation). dcolumn DC_started_math guard. Trailing newline fix. eqnarray 222→123, sideset 481→336, colortbls 220→63, cells ~143.

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
- [x] **26. colortbls_test** (53_alignment) — 63 diffs (was 220). Session 17: colortbl @ tokenization overhaul, backgroundcolor on td, overhang args, dcolumn DC_started_math fix. Remaining: preamble PIs (9), char alignment (8), multicolumn backgroundcolor (4).
- [x] **57–61. Test suite sync** — DONE. Copied pgf (2), tikz (10), moderncv/orc, expl3 (2), slides (2). All 18 new tests ignored.

### Tier 1: Actionable items (no infrastructure blockers)

- [x] **8. fonts_test** (22_fonts) — DONE. Fixed delimited-[] meaning attr (was content). Updated expected XML.
- [x] **8a. mixed_test** (22_fonts) — DONE. Fixed list_apply grammar rule for comma-separated lists.
- [x] **8b. mathaccents_test** (22_fonts) — DONE. Fixed create_xmrefs for Dual/Wrap + empty-arg absent token.
- [x] **8c. plainfonts_test** (22_fonts) — 62 diffs remaining. OMS `\cal` symbols with roles grammar can't handle (METARELOP prefix, empty fenced).
- [x] **9. sizes_test** (22_fonts) — DONE (was 313→181→19→6→4→1→0). Fixed: per-size font metrics, VBoxContents mode property fix, repackHorizontal in predigest_box_contents (matching Perl's readBoxContents endMode → leaveHorizontal_internal flow). 0 diffs.
- [x] **10. ding_test** (22_fonts) — DONE. Passing after cleanup_math + vbox fixes.
- [x] **11. abxtest_test** (22_fonts) — DONE (was TooManyErrors→29→0). Ported mathabx.sty, fixed DefPrimitive literal reversion (empty Tokens!() → CS token), added missing * mathcode (0x2203), mathabx scriptpos=>dynamic_scriptpos, empty element self-closing. 0 diffs.
- [x] **13. enum_test** (50_structure) — DONE. enumitem.sty fully ported.
- [x] **16. figure_grids_test** (50_structure) — DONE. Passing after previous fixes.
- [x] **18. amsarticle_test** (50_structure) — DONE (was 807). Ported rearrangeAMSSplit/rearrangeAMSMultirow, `\@ams@multirow@bindings`, multline tex= via setBody, prefix addop n-ary fix, XMRef resolution, append_tree xml:id preservation. 3 minor diffs accepted: lpadding from \quad, xml:ids on + operators.
- [ ] **25. cells_test** (53_alignment) — 85 diffs (was 102, 282, 300, 548, 780). Session 18-19: fixed trailing whitespace (skip empty text font wrappers in trim). Remaining: ltx_nopad_l on left-aligned @{} cells (4 diffs), rotation angle/dimensions missing on inline-block, paragraph width mismatch, diaghead structural diffs.
- [x] **27. supertabular_test** (53_alignment) — DONE. Ported supertabular.sty + alignment glue fix + right-trim fix.
- [ ] **35. graphrot_test** (65_graphics) — 25 diffs (was 127, 596). Fixed rowspan-cancelled thead→tfoot misclassification. Remaining: thead="column row" vs "column" (from guessHeaders), inline-block dimensions (font metrics), border "ll" vs "l".
- [x] **37. xcolors_test** (65_graphics) — DONE. Passing after previous fixes.

### Tier 2: Needs afterConstruct DOM rearrangement (BLOCKED on item 29)

- [x] **29. Implement afterConstruct rearrangement** — `rearrangeEqnarray`, `rearrangeAMSAlign`, `rearrangeAMSGather`, `openMathFork`/`closeMathFork`/`addColumnToMathFork`/`equationgroupJoinCols`. Done. Fixed _Capture_ XMArg wrapping issue (Perl model prevents XMArg inside _Capture_).
- [x] **2. eqnums_test** (50_structure) — DONE. MathFork xml:id, prefix_relop_apply, tex= mbox synthesis, tag font italic wrapping (XMArg box font family="math" detection in floating script rewrite). 0 diffs.
- [x] **3. algx_test** (53_alignment) — DONE. Infix modifierop grammar rule (expression modifierop expression => infix_apply). 0 diffs.
- [x] **28. badeqnarray_test** (53_alignment) — DONE. Fixed is_script regex, prefix_relop_apply grammar rule, displaystyle tex= spacing. 0 diffs.
- [x] **30. amsdisplay_test** (56_ams) — DONE (was 842). Ported subequations counter save/restore, multline tex= via setBody. 3 minor diffs accepted (same as amsarticle).
- [x] **31. matrix_test** (56_ams) — DONE. Fixed \| delimiter: OPEN/CLOSE role, U+2016 char, name="||", U+2225 char key. 0 diffs.
- [ ] **32. sideset_test** (56_ams) — 336 diffs (was 481, DOM corruption). Fixed append_tree. Grammar rules helped (-145). Remaining: "Classic" section ({}_a^b\sum) floating scripts + 8 unparsed expressions.

### Tier 3: Needs package bindings (moderate effort)

- [ ] **12. stmaryrd_test** (22_fonts) — 1007 diffs (was 1449). Ported stmaryrd.sty, fixed FontDirective Display. Remaining: mostly XMDual + math parser text= diffs.
- [ ] **33. cd_test** (56_ams) — 221 diffs (was 146, 175). XMCell structure, XMDual/XMWrap diffs.
- [ ] **34. mathtools_test** (56_ams) — Ported mathtools.sty. Test hits timeout in math parser (was TooManyErrors).
- [ ] **36. picture_test** (65_graphics) — 3125 diffs. Port picture env + graphpap.sty.
- [ ] **38. xytest** (65_graphics) — TooManyErrors. Port xy.sty binding.
- [x] **39. cleveref_minimal_test** (80_complex) — DONE. Ported cleveref.sty: \lx@cref constructor, crefMulti, type_tag_formatter mappings. 0 diffs.

### Infrastructure fixes (no test flip, but improve structure):

- **XMDual for \\choose/\\brace/\\brack**: Pre-digest left/right delimiter tokens during after_digest, store as Stored::Digested. Template `?#needXMDual` conditional now produces proper XMDual + XMWrap + OPEN/CLOSE structure.
- **Matrix delimiter Tokens**: left/right keyvals stored as Stored::Tokens for proper absorption in matrix template (matrix_test 173→164).
- **\\lx@begin@inmath@text mode fix**: Changed from 'text' to 'restricted_horizontal' matching Perl.
- **\\subjclass Default: fix**: Handle Default: parameter spec in closure.

### Alignment Faithful Translation Plan

**Priority: Complete and faithful translation from Perl. No gaps, no silent behavioral differences.**

**Completed:**
- [x] 2a. Preamble PI capture: `\newcolumntype` now calls `AddToPreamble` via `\lx@add@Preamble@PI`. colortbls 312→96, array 230→127.

**Remaining — Faithful Perl translation gaps (ordered by Perl file coverage):**
- [ ] A1. `alignment_skip_data` continuation-line check: Perl has `&& (($n < 2) || (empty_count <= 0.4 * cols))` guard. Adding it naively caused 173 regressions — needs careful integration with full Perl matching.
- [ ] A2. `normalize_prune_rows` empty-row preservation: Perl keeps empty rows that Rust prunes. Root cause unclear — possibly different `empty`/`skippable` computation for cells with only template fill.
- [ ] A3. Font wrapper `<text>` elements during alignment absorption: Rust creates spurious empty `<text _noautoclose>` wrappers. Perl doesn't create these. Root: different font change tracking.
- [ ] A4. `{turn}` rotation dimensions inside alignment: `after_digest_body` gets empty body for alignment-containing environments.
- [ ] A5. guessHeaders column characterization: Rust over-detects column headers vs Perl. Same threshold/validation, but different results. Needs instruction-level comparison.

**Remaining — Missing package bindings (faithful ports needed):**
- [ ] B1. diagbox.sty (164 lines Perl → ~150 lines Rust). Blocks diagboxtest_test (267 diffs).
- [ ] B2. Split/gather `$` mode: alignment depth guard (Perl #2775). Blocks split_test (1884 missing lines).
- [ ] B3. listings math: listings code blocks with math expressions. Blocks listing_test (2032 diffs).

**Remaining — Math parser faithful translation gaps:**
- [ ] C1. Empty XMRef idref: premature id generation during grammar actions conflicts with DOM installation. Needs architectural fix.
- [ ] C2. Font specialize for parser tokens: `_font` not fully specialized for operator symbols in nested script contexts.
- [ ] C3. Scripted operators: `\mathop{\mathop{A}\limits_{B}}\limits^{C}` produces different structure (XMWrap vs XMApp with scriptpos differences).
- [ ] C4. ltx_nopad_l on @{}l@{} columns: Perl doesn't add ltx_nopad_l, Rust does. Subtle lspaces difference.

### Math parser known limitations:

- **`2\sin(x)` doesn't parse** — `tight_term factor` is left-recursive only; `\sin(x)` as a tight_term can't appear as right of invisible_times. Perl handles this. Pre-existing limitation.
- **Division right-scoping** — `abc/de` = `(abc)/(de)` in Rust, `((abc)/d)*e` in Perl. Accepted as intentional: juxtaposition binds tighter than explicit `/`.
- **`f∘sin x` composition scoping** — Rust: `compose(f, sin(x))`, Perl: `(compose(f, sin))(x)`. Both mathematically equivalent.

### Incomplete stubs requiring full implementation:

- **\\sideset** (amsmath_sty.rs): Stub passes through base #3 only. Full Perl implementation (L1183-1234) needs sidesetWrap with individual pre/post sub/superscript handling, scriptpos calculation, and FLOATING detection.
- **\\lx@equationgroup@subnumbering@begin/end** (latex_ch7_math_mode_environments.rs): DONE. Full counter save/restore with RefStepCounter, ResetCounter, \theequation redefinition.
- **mathtools.sty** (mathtools_sty.rs): \\DeclarePairedDelimiter family, \\newtagform/\\renewtagform, \\newgathered, \\smashoperator all stubbed as DefMacro None. Full Perl implementations need runtime macro factory closures.
- **\\lxDeclare** (latexml_sty.rs): Simplified post-hoc matching vs Perl's full DeclarationRewrite system with XPath patterns, scope limiting, afterConstruct hook. Mathcode-decoded and name-attribute matching added as workarounds.
- **diagbox.sty** (diagbox_sty.rs): Stub with simple macros. Full Perl implementation (164 lines) has diagonal line drawing, SVG generation, width calculation.
- [ ] **40. figure_mixed_content_test** (80_complex) — 1142 diffs. Needs wrapfig + listings math.
- [x] **41. aastex631_deluxetable_test** (80_complex) — DONE. Ported deluxetable.sty, Stored::Template, version-stripping dispatch.
- [x] **42. aliceblog_test** (80_complex) — DONE. Passing after previous fixes.

### Tier 3b: Perl XML sync — code fixes needed (see docs/PERL_XML_DIFFS.md)

These are differences discovered by comparing `LaTeXML/t/*.xml` with `latexml_oxide/tests/*.xml`.
Tests currently pass against Rust expected XMLs, but Rust output diverges from updated Perl.

- [ ] **P1. guessTableHeaders differences** — Fixed pifont empty cells (was `?`), reducing ding.xml diffs 435→31. Remaining 31: `<thead>` wrapper not detected due to `is_numeric()` classifying circled digits (①-⑧) as Integer, causing row 21-22 comparison to exceed threshold. Cannot use `is_ascii_digit` because bbold header detection requires `is_numeric` for mathematical digits (𝟘-𝟟). Affects: `fonts/ding.xml`, `alignment/tabular.xml`, `graphics/xcolors.xml`. NOTE: Perl's continuation-line logic (L1336-1339) is dead code.
- [x] **P2. ltx_figure_panel CSS class** — DONE. `arrange_panels` now marks all non-metadata children. Synced `figure_grids.xml`.
- [x] **P3. DIFFOP recognition in math parser** — DONE. Grammar rule `factor += unknown factor_base => diffop_apply` with INTOP context check. Synced `dots.xml`.
- [x] **P4. Titled frame support** — DONE. Fixed `after_digest_begin` to use `gullet::unread`. Synced `framed.xml`.
- [ ] **P5. xcolors.xml fixes** — Color complement/wheel computation errors, missing `pt` units in calc output, `colortbl` row cycling broken (all "row 0"), missing `ltx_guessed_headers` class. Affects: `graphics/xcolors.xml` (~688 line diff). BLOCKED: needs color model + guessTableHeaders.
- [x] **P6. RDFa support** — DONE. Full lxRDFa.sty binding: DefKeyVal for RDFa family, `\lxRDFa`, `\lxRDFAnnotate`, `\lxRDF` preamble/body, `\lxRDFaPrefix`, `\ref` detection. aliceblog_test passes.
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

- [x] **46. choose_test** (40_math) — DONE. Fixed \lx@generalized@over delimiter digestion (rewrite \lx@right/\lx@left to \@right/\@left before stomach::digest to avoid egroup semantics). 0 diffs.
- [x] **47a. arrows_test** (40_math) — DONE. Fixed ARROW multirelation chain (was only RELOP). 0 diffs.
- [ ] **47. 40_math suite** (40_math) — niceunits 405→38 (nicefrac+units.sty ports, remaining: slash font). not 150 (\not\operatorname{R} sibling issue). testscripts 124 (nested \mathop Marpa RESET). ambiguous_relations 237 (structural diffs).
- [ ] **48. 70_parse suite** (70_parse) — algebraic_terms PASSES, terms PASSES (juxtaposition-binds-tighter accepted). compose 12 (OPFUNCTION barearg + lxDeclare fix; remaining: f∘sin x composition scoping). function_argument_syntax 27 (sin π×x multi-function chain — parser fails). scripts/testscripts/arrows/not ~74 each (parser structural).
- [ ] **49. plainmath_test** (53_alignment) — 351 diffs. Math parser XMDual structure.
- [ ] **50. split_test** (53_alignment) — 102 diffs (down from 2228). prefix_relop_apply fixed most diffs. Remaining: math parser.
- [x] **51. eqnarray_test** (53_alignment) — DONE (was 6→0, was 575→69→9→6). Fixed: postfix_apply n-ary, cdots ELIDEOP, lefteqn class, untex CS spacing, MathFork tex= join, xml:id multi-row pre-advancement. 0 diffs.
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

- [ ] **53. diagboxtest_test** (53_alignment) — 744 diffs (was TIMEOUT). No longer loops but has many structural diffs.
- [ ] **54. ncases_test** (53_alignment) — TIMEOUT: infinite loop in ncases.
- [ ] **55. vmode_test** (53_alignment) — SEGFAULT in vertical mode.
- [ ] **56. babel suite** (81_babel) — TIMEOUT: unbounded memory leak.

### Overarching projects

#### Critical engine gaps (blocks multiple tests, ~2500 lines):
- [ ] **G. latex_ch7_math_common_delimiters.rs** — File empty but `\big` etc. already work via plain.rs. May need LaTeX-specific sizing adjustments. Low priority.
- [ ] **H. tex_box.rs SVG + vrule/hrule** — SVG collapse, `\vrule/\hrule` alignment, BoxSpecification for `\halign`. ~300 lines.
- [ ] **I. tex_tables.rs \halign BoxSpecification** — Entirely commented-out section. ~200 lines.
- [ ] **J. Rewrite system** — rewrite.rs at ~20% (Select/Replace only). Missing: `attributes`, `regexp`, `action`, `on_match` clauses. ~400 lines. Affects \lxDeclare, math decoration.
- [ ] **K. Declaration system (\lxDeclare)** — Rust uses simplified post-hoc matching. Perl has full DeclarationRewrite with XPath patterns, scope limiting, semantic annotation. ~200 lines.

#### Important engine gaps (needed for test suites, ~1500 lines):
- [ ] **L. latex_ch8_defining_commands.rs** — Missing `\DeclareMathAccent`, `\DeclareFontShape/Family/Encoding`, `\SetSymbolFont`, `\SetMathAlphabet`. ~200 lines.
- [ ] **M. latex_ch14_pictures_and_color.rs** — Picture environment 0% ported (`\put`, `\multiput`, `\line`, `\vector`, `\oval`). ~300 lines. Blocks picture_test.
- [ ] **N. tex_fonts.rs** — `\fontname` scaling format, per-font `\hyphenchar`, `\fontdimen` (only 3/15 params), 7 ligatures. ~150 lines.
- [ ] **O. base_xmath.rs matrix/cases** — ~24 commented-out defs, `MathWhatsit()` missing. ~100 lines.
- [ ] **P. plain.rs missing macros** — `\alloc@{}{}{}{}{}`, `\@@oalign/@@ooalign`, `\multispan`, `\hglue`. ~100 lines.

#### Package binding parity (329/452 packages unported):
- [ ] **E. Translate ALL Perl binding files** — 123/452 ported (27%). Critical missing: physics.sty (800 lines), siunitx.sty (2000 lines), xy.sty (1000 lines), biblatex.sty (2000 lines), babel.sty (3000 lines). Mega-packages: beamer.cls (2000 lines), tikz.sty+pgf.sty (8000 lines), expl3.sty (4000 lines).

#### Post-processing pipeline (entire system missing, ~7000 lines):
- [ ] **F. Port Post-processing pipeline** — Fully translate `LaTeXML/lib/LaTeXML/Post/` to Rust. 25 modules, 0% ported.
  - **Resources (copy as-is):** XSLT stylesheets, CSS, JavaScript, RelaxNG schemas, Profiles.
  - **Core modules (priority order):** `Post::XMath` (math cleanup), `Post::Scan` (cross-refs), `Post::CrossRef` (navigation), `Post::MathML` (MathML generation), `Post::XSLT` (transforms), `Post::Writer` (serialization), `Post::Split` (multi-page), `Post::MakeBibliography`, `Post::MakeIndex`.
  - **Impact:** Tests 90_latexmlpost out-of-scope. No HTML5/EPUB/JATS output. `role="ID"` assignment (affects ~20 tests) comes from Post::XMath.

### Permanent ignores (not counted)

- **ns1–ns5** (52_namespace) — DTD not supported in Rust port. Permanently ignored.

---

> **Reminder:** Every entry ported from Perl must follow tightly the original semantics and nuances. Read the Perl source, translate precisely, preserve edge cases. The Perl code is the ground truth.
