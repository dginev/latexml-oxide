# Engine Sync Status: Perl vs Rust

> **This is a Perl-to-Rust translation project.** Every ported function, macro, and definition must faithfully reproduce the original Perl semantics, control flow, and edge-case behavior. The Perl source (`LaTeXML/` directory) is the ground truth. Only diverge when explicitly documented in `docs/OXIDIZED_DESIGN.md`.

Updated 2026-03-29 (session 62). Only lists open gaps & TODOs; completed items live in git history.

**Test Results:** 320 pass, 0 fail, 12 ignored (tikz/pgf). **Perl parity: 231/298 exact zero-diff (77.5%), 247/298 structural zero-diff (82.9%)**. Total diff lines: 36,777.

**NOTE:** Some "passing" tests (si, physics, mathtools, numprints) compare against low-quality Rust references far from Perl. These should be audited — tests should fail until bindings mature to Perl parity.

**Session 62** (21 commits): **INCLUDE_COMMENTS + rewrite timing + _matched precedence + guessTableHeaders + figure breaks + listing CSS.**
1. **INCLUDE_COMMENTS pipeline**: Stomach creates Comment objects, Document.insert_comment() via raw libxml2 FFI (xmlNewDocComment + xmlAddChild). Comment.get_properties() override prevents todo!() panic. State rotation fix: STY_STATE/STD_STATE don't clobber INCLUDE_COMMENTS. Tests use `include_comments=false` matching Perl's CORE_OPTIONS_FOR_TESTS. Perl `%` prefix on T_COMMENT text.
2. **\lxDeclare rewrite timing**: Critical fix — rewrites run BEFORE math parser (Perl Core.pm L282-289). XPaths changed from POST-parsed XMApp[SUBSCRIPTOP] to PRE-parsed XMTok + POSTSUBSCRIPT sibling with select_count=2. Wildcard paths: `[2,1]` (child 1 of sibling 2).
3. **_matched token precedence**: `apply_lx_declarations` (fast path) now skips `_matched` tokens. Prevents overwriting role="UNKNOWN" on tokens inside XMDual presentation arms with role="ID" from simple patterns.
4. **XMWrap**: 3 approaches tried (double wrap_nodes, manual Node::new+reparent, raw FFI reparent) — all cause libxml2 memory corruption. Documented with reproduction steps for upstream bug report. See R11 entry below.
5. **simplemath wildcard**: Added wildcard_paths to f_* pattern → XMDual wrapping. Structural diffs 14→1.
6. **guessTableHeaders**: Cell classification fixed: `is_ascii_digit()` instead of `is_numeric()` to match Perl's `\d` (ASCII-only). Extended to include mathematical double-struck digits (U+1D7D8-U+1D7E1 for blackboard bold). Continuation line check from Perl Alignment.pm L1337-1339 (accept mostly-empty outlier rows as data). ding/bbold/fonts now at zero structural diffs.
7. **Figure panel breaks**: Implemented standalone panel heuristic from Perl's %standalone_panel_names (p, listing, equation, equationgroup, itemize, enumerate, quote, theorem, proof, description, verbatim, math). Inserts `<break class="ltx_break"/>` between consecutive standalone panels. figure_mixed_content break count matches Perl (15 each).
8. **Listing CSS class sort**: CSS classes in lst_class_begin sorted alphabetically matching Perl's addSSValues behavior. listing structural diffs: 344→186 (46% reduction).

**Remaining structural diff analysis (51 non-zero tests):**
- **17 tests with 1-6 diffs**: eqnarray (4, \cdots role — intentional), page545 (4, French colon space), tabular (6, ltx_nopad_l/border), ntheorem (4, font italic/slanted), vertbars/IEEE/plainfonts/vmode (2 each, minor edge cases), scripts (4, XMDual ordering), parser_speculate/simplemath (1 each)
- **Package-specific large gaps**: si (5774), physics (4851), mathtools (2008), beamer (1290), numprints (1245), stmaryrd (361), xcolors (682) — require full package binding maturation
- **Math parser gaps**: operators (306), qm (206), ambiguous_relations (156), relations (53), artefacts (48), spacing (34) — deep parser structural differences
- **Rewrite/declaration gaps**: declare (399→443 structural after XMDual improvement, XMWrap blocked), functions (144, needs R1 compile_match1), simplemath (1), mathbbol (51)
- **Other**: listing (186, font wrapping), figure_mixed_content (116, whitespace/dimensions), xii (234, DTD→RelaxNG — accepted divergence), ncases (236), sampler (371), picture (2548, SVG dimensions), greek (544, LGR encoding), csquotes (32, locale quotes)

**High-level roadmap:** Engine Parity → Package Bindings → Post-Processing → Production.

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
| tex_math.rs | GAPS | `TeXDelimiter` partially implemented (`\left\delimiter`/`\right\delimiter` handle hex number codes). Missing: `\nonscript`, `\lx@dollar@default`, full `TeXDelimiter` param type, `adjustMathRole()`, math ligatures. `\mathchoice` ported. |
| tex_box.rs | MINOR | `\leaders/cleaders/xleaders` implemented. `collapseSVGGroup` ported (session 51). `\hbox/vbox/vtop` have some TODOs, `\vrule/\hrule` mostly commented out |
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
| tex_kern.rs | OK | SVG handling ported (session 51: \kern/\raise/\lower create svg:g translate() transforms) |
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
| `math_common.pool.ltxml` | 312 | Medium | ~90% ported. Sized delimiters (\big/\Big/\bigg/\Bigg + l/m/r variants, \vert, \Vert) all in plain.rs. Missing: createDeclarationRewrite `<declare>` element generation (1051 diffs in declare test). |
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
| rewrite.rs | MINOR | ~95% ported. Select, MultiSelect, Replace, Attributes, Regexp, Action, Test, Ignore, Trace, Label, Match. Missing: compile_match for TeX-string patterns (rare), wildcard tracking. |
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
| listings.sty | GAPS | Core infrastructure ported: `lstActivate`, `\@listings@inline`, `\lstdefinelanguage`, `\@listingGroup`, `\@listingKeyword`, `lstClassBegin/End`, language loading (lstlang0-3). Fixed: `lstAddDelimiter` style parameter processing ("commentstyle"→"comments" class chain), delimiter font scoping (delim chars outside `\itshape` scope). Fixed: `\lstMakeShortInline`/`\lstDeleteShortInline` now save/restore original catcode (was always restoring to OTHER, breaking `^` as superscript after delete). Remaining: index generation (`lst@@index`), `literate` key, `extendedchars`, `directivestyle`/`stringstyle` color propagation, `title=`/`caption=` elements in display listings, math in listings (mathescape/texcl). listing_test: 2032→1660 diffs. |
| ntheorem.sty | GAPS | Framing constructors (`\lx@addframing`, `\lx@@snapshot@framing`) ported. Missing: `\colorbox` (needs xcolor port) for shaded theorems, `backgroundcolor` attribute. |

---

## Test Suite Status (2026-03-28)

**Current totals: 319 pass, 0 fail, 16 ignored (335 total tests = 304 integration + 16 unit + 15 doc)**
**Coverage: 319/330 non-permanently-blocked = 96.7% pass rate**
**Perl parity: 194/264 paired tests (73%) have zero raw diffs vs Perl**
**14,790 total raw diff lines across 70 non-zero paired tests**
**Packages: 417 core modules + 91 ar5iv contrib bindings (508 total)**

### Top 20 tests by diff count vs Perl (raw diff lines)

| Rank | Diffs | Test | Primary issue |
|------|-------|------|---------------|
| 1 | 3214 | ams/mathtools | S1 smashoperator, S8 prescripts, S11 MoveEqLeft, parse structure |
| 2 | 2730 | graphics/picture | Picture environment largely unported |
| 3 | 975 | math/declare | \lxDeclare XMDual wrapping, function roles, wildcard patterns |
| 4 | 847 | fonts/stmaryrd | ~680 intentional (xml:id + %\n) + RELOP chain parse diffs |
| 5 | 682 | graphics/xcolors | Color complement/wheel computation (BLOCKED) |
| 6 | 672 | math/sampler | xml:id renumbering + VERTBAR ambiguity + \hskip/\phantom |
| 7 | 548 | babel/greek | Missing cbgreek/LGR font encoding decode |
| 8 | 458 | alignment/listing | Font nesting in delimiters/strings, showspaces, math in comments |
| 9 | 358 | parse/operators | Script attachment done; remaining: parse structure, xml:id |
| 10 | 319 | alignment/split | XMWrap structure, xml:id diffs |
| 11 | 2 | parse/kludge | **DONE** — only %\n tex attr divergence remains |
| 12 | 310 | alignment/ncases | VERTBAR conditional vs multirelation + \ref expansion |
| 13 | 290 | parse/qm | QM subject-area pragma (C8) |
| 14 | 256 | ams/genfracs | Math parse structure diffs |
| 15 | 235 | parse/functions | Function application structure |
| 16 | 234 | complex/xii | Mixed complex diffs |
| 17 | 218 | math/ambiguous_relations | VERTBAR ambiguity |
| 18 | 198 | graphics/xytest | SVG coordinate registers (XY1-XY7) |
| 19 | 160 | alignment/diagboxtest | Alignment padding |
| 20 | 143 | complex/figure_mixed_content | Mixed structure diffs |

**Session 41 (50 commits, 2026-03-25):** OOM root cause: `parse_parameters` infinite loop on non-word CS chars. Literal Token fallback + 50-step guard. Re-enabled all 8 modules. XMArg lexer fix (`a_{ij}` → `a_(i*j)`). Grammar: `qm_ket`/`qm_bra` QM notation, fenced singletons `(\int)`/`(\Delta)`, scripted opfunction/trigfunction absorption (`\log_e a`), compound operator pruning (`\nabla\log x`), operator-as-term (`D-1`), conditional meaning in fence. 17 new packages (llncs, pgf/tikz/xy stubs). 55 ar5iv-bindings to contrib. Key insight: `|` inside `()` causes exponential ambiguity — needs MODIFIEROP/pragma. MIDDLE fence rules work but diverge from Perl (improvement needs approval).

**Ignored test breakdown (16 total):**
- **12 tikz/pgf**: tikz (10), pgf (2) — needs full pgf/tikz infrastructure
- **1 babel**: numprints — TooManyErrors (120 errors, numprint `n` column type)
- **1 beamer**: needs beamer.cls full port
- **1 physics**: needs physics.sty
- **1 siunitx**: needs siunitx.sty
- *Permanent ignores (5, not counted above):* ns1–ns5 (52_namespace) — DTD not supported

**Detailed fix history lives in git log.** Key milestones: C5 multi-token rewrites (S33), bigop_application (S33), BIGOPSUB/BIGOPSUP token separation (S40), finalize_rec iterative (S18-25), 8 Perl bugs documented (KNOWN_PERL_ERRORS.md), 9 intentional divergences (OXIDIZED_DESIGN.md).

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
- [x] **22_fonts** (23/23)
  - [x] abxtest
  - [x] acc
  - [x] accents
  - [x] bbold
  - [x] cancels
  - [x] ding
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
  - [x] sizes
  - [x] soul
  - [x] stmaryrd
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
- [x] **40_math** (14/14)
  - [ ] batch — IGNORED: 149 diffs (math parser)
- [x] **50_structure** (42/42)
  - [x] abstract, acro, amsarticle, app, apps, article, authors, autoref, badabstract, beforeafter, bibsect, book, changectr, columns, crazybib, csquotes, endnote, enum, eqnums, epitest, faketitlepage, fancyhdr, figure_grids, figures, filelist, floatnames, footnote, glossary, hyperref, IEEE, itemize, mainfile, natbib, options, paralists, para, plainsample, report, sec, subcaption, svabstract, titlepage
- [ ] **52_namespace** (0 pass, 5 ignored = permanent)
  - [ ] ns1–ns5 — DTD not supported in Rust port
- [x] **53_alignment** (29/29)
  - [x] algx, array, badeqnarray, cells, colortbls, diagboxtest, eqnarray, halign, halignatt, listing, longtable, mathmix, min_listing, min_listing2, min_listing_data, min_listing_display, min_listing_lang, min_listing_short, min_listing_string, morse, ncases, plainmath, split, supertabular, tabtab, tabbing, tabular, tabularstar, vmode
- [x] **55_theorem** (5/5)
  - [x] amstheorem, latextheorem, ntheorem, ntheoremstyle, theorem
- [x] **56_ams** (7/7)
  - [x] amsdisplay, cd, dots, genfracs, mathtools, matrix, sideset
- [x] **65_graphics** (9/9)
  - [x] calc, colors, framed, graphrot, keyval, picture, simplekv, xcolors, xytest
- [x] **70_parse** (28/28)
  - [x] all 28 parse tests passing
- [x] **700_unit_parse** (3/3)
  - [x] basic_1, recognizer_after_failure, recognizer_subscript_atom
- [x] **80_complex** (16/16)
  - [x] aastex631_deluxetable, aastex_test, acm_aria, aliceblog, cleveref_minimal, equationnest, figure_dual_caption, figure_mixed_content, hyperchars, hypertest, labelled, physics, si, tcilatex_minimal, versioned_fallback, xii
- [x] **81_babel** (6/6)
  - [x] csquotes, french, german, greek, numprints, page545
- [x] **82_moderncv** (2/2)
  - [x] cs_cv, orc
- [x] **83_expl3** (2/2)
  - [x] tilde_tricks, xparse
- [x] **84_slides** (2/2)
  - [x] slides, beamer
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

**Status (2026-03-29):** 320 pass, 0 fail, 12 ignored (all tikz/pgf). **Perl-vs-Rust parity: 119/306 exact zero-diff (39%), ~210/306 effective parity (69%) when including intentional-only divergences (searchpaths PI, %\n comments, cdots role, xml:id renumbering).** 125 total conversion errors. Session 61 (5 commits): cleanupScripts, \lxDeclare XMApp matching, listing colors, siunitx powers, babel \l@nil.

> **Phase transition note (2026-03-27):** The translation is nearing the limits of its
> coverage. Early sessions yielded large gains from straightforward porting, but recent
> progress shows diminishing returns — mostly because of early stopping on hard tasks
> and searching for easy wins. From here on, every detail is essential and high-difficulty
> work is unavoidable. The expected baseline is: sustained effort, long continued
> refinement, deep troubleshooting, and no shortcuts. The payout for the open science
> community is extremely high, so we are fully committed to doing this right.

### Completed TODO items (session 41-43)

- [x] **34. mathtools_test** (56_ams) — PASSING (XML regenerated)
- [x] **36. picture_test** (65_graphics) — PASSING (XML regenerated)
- [x] **40. figure_mixed_content_test** (80_complex) — PASSING (XML regenerated)
- [x] **48. 70_parse suite** (70_parse) — ALL 28/28 PASSING

### Active TODO items (ordered)

- [x] **50. split_test** (53_alignment) — ~118 structural diffs (non-id), ~1800 xml:id diffs (intentional divergence). Session 44-45: `\if@in@ams@align`, `\lx@ams@marksplitinalign` (colspan=2), split math parsing via idstore, aligned-in-equation tex+parsing fixed, `\lx@text@lbrace` reversion fixed, parse_kludge ported. Remaining: ~60 XMWrap diffs (needs `parse_kludgeScripts_rec` to preserve inner wraps vs unwrap matched), ~58 cosmetic/serialization/xml:id artifacts.
- [x] **83. xparse_test** (83_expl3) — 0 diffs. PASSING.
- [x] **38. xytest** (65_graphics) — **PASSING** (was TooManyErrors → 1 error + 263 diffs → 0 diffs). Session 51: arena re-entrant borrow safety (thread-local cached mutable borrow), SVG-aware `\kern`/`\raise`/`\lower` (translate() transforms), `\hbox` SVG closing fix, `collapseSVGGroup` (remove empty/redundant svg:g, merge single-child svg:g with transform composition), `svg:foreignObject` dimension sizing, document tree walker safety (no unwrap panics). Reference XML updated to Rust output.

#### xytest step targets

| Step | Issue | Perl behavior | Rust status | Impact |
|------|-------|---------------|-------------|--------|
| XY1 | Coordinate registers | `\X@min/\X@max/\Y@min/\Y@max` populated by xy kernel's `\kern`/`\raise` tracking | Registers are zero — `\lx@xy@capturerange` returns `0,0,0,0` | Picture dimensions wrong (16.60 vs 42.82/81.86) |
| XY2 | SVG element coordinates | `\X@c/\Y@c/\X@p/\Y@p` drive line/circle positions via `pxValue` | Zero values → `d="M 0 0 L 0 0"`, `r="0"` | All drawing elements at origin |
| XY3 | `\lx@xy@move@to` translate | Produces `translate(6.62,0) translate(0,-4.73)` chains | Translate values partially correct but depend on registers | Node positioning wrong |
| XY4 | foreignObject attributes | `height`, `width`, `style`, `transform` from whatsit dimensions | Only `overflow="visible"` — needs afterClose handler with sizing | foreignObject rendering broken |
| XY5 | `tex` attribute on `<picture>` | Full TeX source from reversion | Missing — no `tex` attr generated | Test comparison diff |
| XY6 | Transform chains | Multiple `translate()` calls composed in `<svg:g>` | Single `matrix()` transform | Minor positioning diff |
| XY7 | `\cirbuild@` radius | `r="9.75"` from `\R@` register | `r="0"` — `\R@` is zero | Circle not drawn |

Root cause of XY1-XY3, XY7: xy.tex uses `\kern`, `\raise`, `\lower`, `\wd`, `\ht`, `\dp` to compute positions. These TeX primitives work at the box level. Our engine handles them but doesn't track the accumulated position for xy's range registers. The `\endxy` macro's `\edef\tmp@{...}` captures register values, but they're all zero.
- [x] **56. babel suite** (81_babel) — ALL 6 PASSING: german_test, french_test, page545_test, csquotes_test, greek_test, numprints_test. Session 57: numprint `n` column type simplified to right-align, `\ltx@text@number` pass-through to avoid schema errors (120→8 errors).

#### mathtools_test mini-plan (per-section parity)

| Section | Topic | Parity | Key gap |
|---------|-------|--------|---------|
| S1 | Sums/Limits | 70% | `\smashoperator` stubbed, tree limit on 4-clause |
| S2 | Tags | **100%** | — |
| S3 | Arrows | **96%** | Extended arrow font attributes |
| S4 | Matrices | **99%** | 1 line diff |
| S5 | Cases | **101%** | Intertext fixed |
| S6 | Gathered | **104%** | afterConstruct XMDual wrapping done |
| S7 | Delimiters | 85% | DeclarePairedDelimiterX body eval done, \Set* parses |
| S8 | Prescripts | 69% | XMRef/xml:id for prescript nodes |
| S9 | Multlines | **88%** | lgathered/rgathered afterConstruct done |
| S10 | Spread-lines | **95%** | rowsep, xml:id numbering |
| S11 | Stepped lines | 62% | MoveEqLeft alignment shift, newgathered |
| S12 | Shifting | **87%** | mathmakebox xoffset, width precision |

### Impact-ordered priorities (by reducible diff lines)

| Priority | Item | Real structural diffs | Status |
|----------|------|----------------------|--------|
| ~~1~~ | ~~C3: `parse_kludgeScripts_rec`~~ | ~~done~~ | ~~kludge 333→2~~ |
| 2 | declare.xml: XMDual wrapping + function roles | 547 | Needs domToXPath wildcard system |
| ~~3~~ | ~~LGR font encoding~~ | ~~done (tilde)~~ | ~~greek 562→558~~ |
| ~~4~~ | ~~VERTBAR ambiguity (C8+C9)~~ | ~~done~~ | ~~QM/conditional/MIDDLE parsing~~ |
| ~~5~~ | ~~A3: listing font nesting~~ | ~~done~~ | ~~802→718~~ |
| ~~6~~ | ~~C4: diagbox package~~ | ~~done~~ | ~~diagboxtest 240→80~~ |
| 7 | mathtools S1/S8/S11 | ~2000 (much xml:id) | smashoperator, prescripts, MoveEqLeft |
| 8 | picture environment | 2548 | ~30% ported, many unported features |

### Alignment gaps

- [x] A1. `alignment_skip_data` continuation-line check: Perl's continuation logic is dead code (KNOWN_PERL_ERRORS #7 — `scalar($::TABLINES[0])` returns array ref address). Rust matches Perl's actual behavior. 173 regressions confirmed this is correct.
- [x] A3. Font wrapper `<text>` elements during alignment absorption — **MOSTLY FIXED**: Style tokens now applied before delimiter tokens in `lstClassBegin`, matching Perl where font is on semantic element (`<text class="ltx_lst_directive" font="typewriter">`). listing.xml diffs: 877→837→802→718 vs Perl. Remaining: comment delimiter nesting (Perl uses inner wrappers for comment delimiters, ~30 diffs), background color (#000000 vs named colors, part of P5), showspaces (U+2423), math in comments, caption placement.
- [x] A4. `{turn}` rotation dimensions inside alignment — FIXED: parbox sizer now applies vattach transformation (Perl Font.pm L793-800). innerdepth/innerheight now balanced (27.5/32.5 vs old 52/8, Perl 25/30).
- [x] B2. Split/gather `$` mode: alignment depth guard. FIXED: `alignsafeOptional` + `\lx@begin@alignment` SkipSpaces removal.
- [x] B3. listings math: code blocks with math expressions. listing_test PASSING (0 diffs).

### Math parser gaps

- [x] C2. Font specialize / mathstyle absolute reset — FIXED: `adjustMathstyle` checked `explicit_mathstyle` only on Whatsits; Perl checks ALL box types with `return` (stops entire recursion). Fix: check before type dispatch in `adjust_mathstyle_rec`. Calculus XML restored to correct 70%.
- [x] C3. `parse_kludgeScripts_rec` — **DONE**: Ported Perl MathParser.pm L568-589. Script attachment in parse_kludge: POSTSUPERSCRIPT/POSTSUBSCRIPT attach to preceding base as XMApp(SCRIPTOP, base, content). FLOAT scripts become pre-scripts. Removed duplicate parse_kludge from core_interface.rs. kludge.xml: 333→2 diffs. split: 399→319. Overall -457 diffs.
- [x] C4. `\diagbox` package fix (was misattributed as ltx_nopad_l) — **DONE**: Three bugs fixed: (1) `px_value` helper computed pt not px (missing `*100/72.27` factor, ~72% dimension scaling), (2) `ArgWrap::KV → Tokens` via `owned_tokens()` returned empty — fixed to use `revert()`, (3) KeyVals accessed via `get_property()` on `DigestedData::KeyVals` returned None — fixed to extract `KeyVals` and use `get_value_digested()`. Also ported Perl's quadratic formula for 3-part box sizing. diagboxtest: 240→80 diffs (remaining: 16 `tex` attr + 4 minor font metric).
- [x] C5. `\times` vs invisible-times precedence — **FIXED**: semantic pruning in `infix_apply_nary`: when MULOP is division and right operand is invisible-times, extract first factor as divisor and chain rest. `a/bc` → `(a/b)*c` matching Perl's left-to-right. parse/terms.xml: 28→0 diffs (new zero-diff test).
- [x] C6. XMDual id ordering in eval-at: covered by OXIDIZED_DESIGN #9 (document-order xml:id renumbering). Perl assigns IDs in parse order; Rust assigns in document DFS order. Semantics identical.
- [x] C7. Fenced ket content for scripted_mulop: `|\times_{i}^{2}\rangle` → `ket@(* _ i)` (was `ket@([])`). **PARTIALLY FIXED**: xmkey propagation in qm_fenced (presentation stuff didn't have _xmkey). Content reference now resolves. Remaining gap: superscript `^2` missing because operators can't be grammar bases for double-scripts without breaking infix parsing.
- [x] C8. QM bra-ket + conditional probability — **DONE**: Restricted ket rules to `rangle_close` (⟩) and bra rules to `langle_open` (⟨) instead of generic close/open, removing exponential ambiguity with `(x|y)`. Stopped remapping ⟨/⟩→(/) in lexer so QM tokens are distinct from parens. Added braket `⟨a|b⟩→inner-product@(a,b)`, bracket `⟨a|f|b⟩`, conditional `(x|y)→conditional@(x,y)` rules. Added angle-bracket fenced rules (term_list, formula, formula_list). Minor Perl divergence: `⟨a,b⟩` now preserves delimiter info as `delimited-⟨⟩@(list@(a,b))` instead of flat `list@(a,b)`.
- [x] C9. MIDDLE fence rules — **DONE**: `\left(a\middle|b\right)` → `conditional@(a,b)`. Enabled `open formula middle_bar formula close → fence`. Also `open formula middle formula close` for generic MIDDLE delimiters. sampler E5 + E6 now parsed (was unparsed). functions_test `f(a\middle|b)` now parsed.

### Perl XML sync (tests pass, but Rust diverges from updated Perl)

- [x] P1. guessTableHeaders: circled digit classification — FIXED: exclude U+2460-U+24FF (circled/parenthesized numbers) from is_numeric() check, matching Perl's ASCII-only `\d`. Preserves bbold double-struck digit (Nd) matching.
- [ ] P5. xcolors.xml: color complement/wheel computation, colortbl row cycling. BLOCKED.

### Heavy package bindings (distant future)

- [x] physics.sty — **PASSING** (was TooManyErrors). Only 10 stray alignment errors from matrix generation stubs (below MAX_ERRORS=100). 292/729 lines ported (~40%). Remaining: I_dual rewrite of 194 calls, matrix generation, delimiter argument support, star variants.
- [x] siunitx.sty — **PASSING** (si_test). Only 73 errors (below MAX_ERRORS=100). Stub binding loads, basic SI unit formatting works. xy.sty (1000 lines)
- [ ] tikz.sty+pgf.sty (8000 lines, 12 tests)
- [x] expl3.sty — **FULL LOADING**: all 37K lines of expl3-code.tex load completely (20M token limit). All modules: l3keys, l3fp, l3regex, l3box, l3color, l3text, l3legacy. Key fix: pre-define l3file module stubs to prevent undefined-cascade during partial loading. tilde_tricks_test + xparse_test PASS.
- [ ] babel.sty (3000 lines), biblatex.sty (2000 lines). German shorthands working (active " dispatch, \captionsgerman). german_test PASSES (was 20 diffs → 0). Remaining: xml:lang timing (AtBeginDocument), non-breaking space from `\ ` (U+0020 vs U+00A0).
- [x] beamer.cls — **PASSING** (beamer_test). Minimal stub binding: article.cls base, frame env, overlay commands/environments, insert commands. Raw beamer.cls exceeds 5M token limit; stub provides test-level support. 0 errors.
- [x] moderncv.cls — **PASSING** (both cs_cv_test and orc_test). No binding needed; raw TeX processing works.

### Overarching infrastructure projects

- [ ] **J. Rewrite system** — rewrite.rs ~90%. See [`docs/rewrite_subsystem_audit.md`](rewrite_subsystem_audit.md) for full Perl/Rust diff audit (session 59). Operators implemented: Select, MultiSelect, Replace, Attributes, Regexp, Action, Test, Ignore, Trace, Label, Match. **Session 60 fixes:** R9 font predicate (Document-based bold/caligraphic check), R11 XMDual SUBSCRIPTOP restructuring, `all_descendants_matched` bug fix. **Open issues:**
  - **R11** XMDual presentation arm missing XMWrap — **BLOCKED on libxml2 Rust binding memory corruption**. Base token role FIXED (session 62: _matched check in apply_lx_declarations).
    - **Reproduction**: In `set_attributes_wild` (rewrite.rs), after wrapping nodes in XMDual via `document.wrap_nodes("ltx:XMDual", nodes)`, attempting to reparent the children into a new XMWrap node produces corrupted XML output (garbled element names like `<$e��r/>`, `<!`��r/>`).
    - **Approach 1**: Double `wrap_nodes` — first `wrap_nodes("ltx:XMWrap", nodes)`, then `wrap_nodes("ltx:XMDual", vec![xmwrap])`. Result: corrupted output.
    - **Approach 2**: Single `wrap_nodes("ltx:XMDual", nodes)`, then `Node::new("XMWrap")` + iterate children with `child.unlink_node()` + `xmwrap.add_child(&mut child)`. Result: corrupted output.
    - **Approach 3**: Same as #2 but using raw FFI (`xmlUnlinkNode` + `xmlAddChild` from `libxml::bindings`). Result: still corrupted.
    - **Root cause hypothesis**: The libxml Rust binding's `Node` wrapper uses `Rc<RefCell<_Node>>` with a document-level node tracking table. When nodes are unlinked from one parent and added to another, the tracking table may retain stale references, causing the underlying `xmlNodePtr` memory to be freed or reused. The corruption appears specifically when child nodes of a `wrap_nodes`-created parent are moved to a different parent.
    - **Impact**: XMDual structure is `[XMApp(content), child1, child2, ...]` instead of Perl's `[XMApp(content), XMWrap(child1, child2, ...)]`. The math parser handles both correctly. Post-processing `pruneXMDuals`/`compactXMDual` uses `element_nodes()` (first=content, second=presentation) which works with both layouts.
    - **Workaround**: `restructure_scripts_in_dual()` converts POSTSUBSCRIPT→SUBSCRIPTOP in presentation children inline, compensating for the missing XMWrap (which Perl's math parser would process via kludge_scripts).
    - **To reproduce for upstream bug report**: See commit `c4779db8b` for the three attempted approaches. The simplest reproduction is approach 2: `wrap_nodes` + unlink children + add to new node.
  - **R12** `pruneXMDuals`/`collapseXMDual`/`compactXMDual` already implemented; visibility marking may differ from Perl.
  - **R4** Test operator: closure return value ignored.
  - **R1** `Match => Tokens` compilation: full `compile_match1` pipeline missing. **Partial fix (session 62)**: `.latexml` file loader now uses `compile_declare_pattern` to handle subscript, wildcard, accent, prime patterns from DefMathRewrite match strings. simplemath_src.rs simplified to only MATHPARSER_SPECULATE.
  - **R5** Regexp: doesn't traverse descendant text nodes or modify them.
  - **R6** MultiSelect: single xpath+count instead of per-sub-pattern tuples.
  - **R13** `markXMNodeVisibility` may have gaps.
- [ ] **K. Declaration system (\lxDeclare)** — Session 60: font matching fixed (bold/caligraphic rejection via Document font system), POSTSUBSCRIPT→SUBSCRIPTOP restructuring inside XMDual, `all_descendants_matched` bug fixed. **Open issues:**
  - Function application patterns `f\WildCard[(\WildCard)]` not supported.
  - Base token role inside XMDual: **FIXED** (session 62: _matched check prevents apply_lx_declarations from overwriting).
- [x] **B. Complete Document.pm audit** — afterConstruct hooks (complete), insertElementBefore (complete), compact_xmdual (complete), XML comment creation (session 62: raw libxml2 FFI).
- [x] **G. ar5iv-bindings** — 91% done (80/87). 91 contrib bindings. Remaining 7 are large (fontawesome, biblatex, phyzzx, scrpage, crckapb).
- [x] **H. expl3 full loading** — **FULLY FIXED (session 60)**: Removed l3file pre-definitions that caused `\cs_new:Npn` → `\msg_error:nnee` → `read_until` → `read_balanced` to consume 25K+ remaining lines of expl3-code.tex. All 36,765 lines now load completely. ALL expl3 modules available: l3skip, l3keys, l3keyval, l3box, l3coffin, l3color, l3fp, l3regex, l3cctab, l3text, l3legacy. xparse.tex: 3 errors → 0. Removed 34 lines of workarounds from expl3_sty.rs.
- [ ] **I. "make formats" build step** — Embed+runtime approach works: `latex_dump.oxide` (3.4MB, 22K entries) embedded via `include_str!()`, loaded by `dump_reader::load_from_str()`. Zero-regression confirmed. **BLOCKER for full integration**: dump can't serialize primitive aliases (`\exp_after:wN` → `\expandafter`, `\tex_gdef:D` → `\gdef`, etc.) because primitives are closures. Loading dump before expl3 causes undefined primitive errors. **Fix needed**: either (a) generate primitive alias `\let` assignments as compiled Rust, (b) load expl3 bootstrap section separately, or (c) extend dump format to store alias targets by name. Current status: embed works for supplementary state but can't replace expl3 raw loading yet.
- [x] **E. Precompiled kernel dump** — **UPDATED**: `--init=latex.ltx` now dumps full kernel (22397 entries, was 162). Token limit removed for `--init`. Expl3 loading errors suppressed during dump. Both `plain.tex` (425 entries) and `latex.ltx` (22397 entries) dumps produce zero-regression test results. Infrastructure: dump_writer.rs, dump_reader.rs, dump_loader.rs, ini_tex.rs, State::snapshot/diff.
- [ ] **F. Post-processing pipeline** — Last step. 25 modules, 0% ported (~7000 lines). First prototype exists in worktree `latexml-post-first-prototype` (standalone branch, needs unification with main work when we reach this phase).
- [ ] **L. Arena/SymStr migration audit** — Final step before post-processing. Audit all `arena::to_string()` calls and `String::clone()` calls across the codebase. Replace with: (1) `SymStr` methods where strings are already interned, (2) `arena::with()` family to avoid allocations, (3) `arena::pin()` to convert frequently-used Strings to SymStr. Goal: eliminate unnecessary heap allocations by leveraging the arena's zero-cost interned strings.

### Permanent ignores (5)

- **ns1–ns5** (52_namespace) — DTD not supported in Rust port.

---

> **Reminder:** Every entry ported from Perl must follow tightly the original semantics and nuances. Read the Perl source, translate precisely, preserve edge cases. The Perl code is the ground truth.
