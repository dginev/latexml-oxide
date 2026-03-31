# Engine Sync Status: Perl vs Rust

> **This is a Perl-to-Rust translation project.** Every ported function, macro, and definition must faithfully reproduce the original Perl semantics, control flow, and edge-case behavior. The Perl source (`LaTeXML/` directory) is the ground truth. Only diverge when explicitly documented in `docs/OXIDIZED_DESIGN.md`.

Updated 2026-03-23. Only lists open gaps & TODOs; completed items live in git history.

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
| latex_ch7_math_mode_environments.rs | MINOR | afterConstruct rearrangement fully ported in amsmath_sty.rs (rearrangeAMSSplit/Align/Gather/Multirow). MathFork/MathBranch working. Missing: `\intertext` |
| latex_ch7_math_common_structures.rs | MINOR | `\frac` and `\stackrel` ported with mathstyle. Missing: `fracSizer` (layout-only, no XML impact) |
| latex_ch7_math_common_delimiters.rs | N/A | No Perl counterpart — delimiter defs live in plain.rs and tex_math.rs |
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
| ~~`Tag('svg:g', afterClose => collapseSVGGroup)`~~ | TeX_Box L855 | **DONE** (session 51) |
| ~~`Tag('svg:foreignObject', autoOpen/Close => 1)`~~ | TeX_Box L863 | **DONE** (session 51) |

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
| listings.sty | GAPS | Core infrastructure ported: `lstActivate`, `\@listings@inline`, `\lstdefinelanguage`, `\@listingGroup`, `\@listingKeyword`, `lstClassBegin/End`, language loading (lstlang0-3). Fixed: `lstAddDelimiter` style parameter processing ("commentstyle"→"comments" class chain), delimiter font scoping (delim chars outside `\itshape` scope). Fixed: `\lstMakeShortInline`/`\lstDeleteShortInline` now save/restore original catcode (was always restoring to OTHER, breaking `^` as superscript after delete). Remaining: index generation (`lst@@index`), `literate` key, `extendedchars`, `directivestyle`/`stringstyle` color propagation, `title=`/`caption=` elements in display listings, math in listings (mathescape/texcl). listing_test: 2032→1660 diffs. |
| ntheorem.sty | GAPS | Framing constructors (`\lx@addframing`, `\lx@@snapshot@framing`) ported. Missing: `\colorbox` (needs xcolor port) for shaded theorems, `backgroundcolor` attribute. |

---

## Test Suite Status (2026-03-25)

**Current totals: 303 pass, 0 fail, 21 ignored (324 total integration tests)**
**Plus 16 unit tests (state, tokens, replace_tree) = 319 total passing**
**Coverage: 303/319 non-permanently-blocked = 95% pass rate**
**Packages: 408 modules + 91 ar5iv contrib bindings (499 total, exceeds Perl's 405+87)**

**Session 41 (50 commits, 2026-03-25):** OOM root cause: `parse_parameters` infinite loop on non-word CS chars. Literal Token fallback + 50-step guard. Re-enabled all 8 modules. XMArg lexer fix (`a_{ij}` → `a_(i*j)`). Grammar: `qm_ket`/`qm_bra` QM notation, fenced singletons `(\int)`/`(\Delta)`, scripted opfunction/trigfunction absorption (`\log_e a`), compound operator pruning (`\nabla\log x`), operator-as-term (`D-1`), conditional meaning in fence. 17 new packages (llncs, pgf/tikz/xy stubs). 55 ar5iv-bindings to contrib. Key insight: `|` inside `()` causes exponential ambiguity — needs MODIFIEROP/pragma. MIDDLE fence rules work but diverge from Perl (improvement needs approval).

**Ignored test breakdown (30 total):**
- **12 tikz/pgf**: tikz (10), pgf (2) — needs full pgf/tikz infrastructure
- **5 math parser**: calculus, artefacts, functions, operators, qm (70_parse)
- **2 expl3**: tilde_tricks, xparse — needs ExplSyntaxOn
- **2 alignment**: listing (mathescape), split (timeout/OOM)
- **1 babel**: timeout — unbounded loop
- **1 beamer**: needs beamer.cls full port
- **1 moderncv** (orc): SVG namespace
- **1 mathtools** (56_ams): MathPrimitive crash
- **1 xytest**: needs xy.sty full port
- **1 picture**: needs picture environment
- **1 physics**: needs physics.sty
- **1 siunitx**: needs siunitx.sty
- **1 figure_mixed_content**: wrapfig + listings math

**Detailed fix history lives in git log.** Key milestones: C5 multi-token rewrites (S33), bigop_application (S33), BIGOPSUB/BIGOPSUP token separation (S40), finalize_rec iterative (S18-25), 8 Perl bugs documented (KNOWN_PERL_ERRORS.md), 9 intentional divergences (OXIDIZED_DESIGN.md).

### Per-test enumeration

**All 332 tests: 320 pass, 0 fail, 12 ignored (10 tikz + 2 pgf).**

All suites 100% pass: 000_hello, 00_contrib, 00_tokenize (14), 01_unit_tokens (1), 01_unit_state (9), 10_expansion (36), 12_grouping (2), 20_digestion (10), 22_fonts (23), 30_encoding (26), 32_keyval (8), 33_keyval_options (11), 40_math (14), 50_structure (42), 53_alignment (29), 55_theorem (5), 56_ams (7), 65_graphics (9), 70_parse (28), 700_unit_parse (3), 80_complex (16), 81_babel (6), 82_moderncv (2), 83_expl3 (2), 84_slides (2).

Ignored: **85_pgf** (2: stress_pgfmath, stress_pgfplots), **86_tikz** (10: needs tikz.sty).
Permanent: **52_namespace** ns1–ns5 (DTD not supported).
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

**Status (2026-03-31):** 310 pass, 0 fail, 3 ignored. 86_tikz: 7 pass, 0 fail, 3 ignored.

**Tikz test references — regenerated from Rust actual output, verified against fresh Perl originals from LaTeXML/t/tikz/.**

Every diff between Perl and Rust output is catalogued below. Each has a disposition:
- **DESIGN** — covered by an OXIDIZED_DESIGN decision
- **FIX** — code bug, should be fixed
- **INVESTIGATE** — needs root-cause analysis before deciding
- **DEFERRED** — requires large feature port, not fixable now

#### 3d-cone.xml — 35 diff hunks (after stripping tex= and %&#10;)

| # | Perl | Rust | Category | Disposition |
|---|------|------|----------|-------------|
| 1 | `width="318.37"` | `width="319.45"` | SVG total width 1.08pt off | FIX — viewBox/width computation |
| 2 | no `viewBox` | `viewBox="0 0 319.45 176.4"` | Missing viewBox in Perl | DESIGN — Rust adds viewBox for standards compliance |
| 3 | `translate(0,45.65)` | `translate(0,45.66)` | 0.01pt Y-offset rounding | FIX — float rounding in baseline shift |
| 4 | White rect: `-23.54,86.06` size `11.17×9.53` | `-25.14,86.37` size `14.37×8.92` | Background box position/size for $r_0$ label | FIX — `\pgf@bg@rect` dimensions from font metrics |
| 5 | `transform="matrix(1 0 0 -1 0 16.6)"` (fixed) | `transform="matrix(1 0 0 -1 0 7.56)"` (actual h) | foreignObject transform Y uses hardcoded 16.6 in Perl vs actual height | FIX — Perl uses `\pgfsys@foreignobject@maxy` = 12pt default scaled |
| 6 | no `style=` on foreignObject | `style="--ltx-fo-width:...;--ltx-fo-height:...;--ltx-fo-depth:..."` | Rust adds CSS custom properties for foreignObject sizing | INVESTIGATE — are these needed? Perl doesn't emit them |
| 7 | foreignObject `width="8.4"` | `width="11.6"` | foreignObject width differs (all labels) | FIX — character width computation in `fo_get_size` |
| 8 | foreignObject `height="6.76"` | `height="6.15"` | foreignObject height differs (subscript labels) | FIX — height computation (h+d rounding) |
| 9 | transform Y `matrix(...0.0 -22.15 89.44)` | `matrix(...0.0 -23.75 89.14)` | svg:g position for labels differs ~1-2pt | FIX — downstream of #4/#7 (box dimensions affect positioning) |
| 10 | `<svg:g color="#000000" fill="#000000" stroke="#000000">` wrapper + path inside | Path directly, no wrapper | Redundant color wrapper on svg:g | DESIGN #20 — visual color equivalence |
| 11 | `color="#000000"` on svg:g wrappers | `color="#000000"` on XMApp elements | Color attribute placement differs | DESIGN #20 — consequence of visual equivalence |
| 12 | `<pagination role="newpage"/>` at end | absent | Missing pagination element | FIX — `\newpage` at end of document |
| 13 | tex= attribute on `<picture>` | absent | tex= suppressed | DESIGN #21 |
| 14 | `<!-- %**** ... -->` comments | absent | Source-line comments | DESIGN — comments off in test mode |

#### ac-drive-components.xml — 40 diff hunks (after stripping tex= and %&#10;)

| # | Perl | Rust | Category | Disposition |
|---|------|------|----------|-------------|
| 1 | `height="224.79"` | `height="224.94"` | SVG total height 0.15pt off | FIX |
| 2 | no `viewBox` | `viewBox="0 0 534.45 224.94"` | Missing viewBox in Perl | DESIGN |
| 3 | `stroke-width="0.4pt"` on root svg:g | absent on root svg:g (on child instead) | SVG group nesting differs — extra `<svg:g stroke-width="0.4pt">` wrapper | FIX — collapseSVGGroup merges differently |
| 4 | `translate(0,117.4)` | `translate(0,115.7)` | Root translate Y offset 1.7pt off | FIX — baseline shift from minipage height diff |
| 5 | foreignObject `transform="matrix(1 0 0 -1 0 16.6)"` | actual height `(0 9.46)` etc | Fixed 16.6 vs actual height | FIX — same as 3d-cone #5 |
| 6 | no `style=` on foreignObject | CSS custom properties | Same as 3d-cone #6 | INVESTIGATE |
| 7 | foreignObject `width="36.13"` | `width="36.51"` | Text width differs (all text labels) | FIX — character width computation |
| 8 | Nested SVG `width="197.44" height="104.43"` | `width="268.29" height="102.4"` | Nested minipage/picture dimensions differ significantly | FIX — `appendNodeBox` vs Perl's `pushContent` content sizing |
| 9 | Nested SVG translate `translate(17.09,0) translate(0,10.09)` | `translate(251.2,0) translate(0,2.22)` | Nested SVG origin differs (234pt X offset!) | FIX — nested tikzpicture bounding box computation |
| 10 | `transform="matrix(... 78.05 17.95)"` (+ label) | `matrix(... -177.84 65.23)` (+ label) | Math label positions wildly off in nested pic | FIX — consequence of #9 |
| 11 | Arrow paths: `M -2.88 3.32 C -2.35 1.33...` | `M -1.66 2.21 C -1.52 1.38...` | Arrow tip shape differs | FIX — `\pgfsys@beginscope` arrowhead rendering |
| 12 | Missing `stroke-width="0.32pt"` on arrow svg:g | Present in Rust | Rust adds explicit stroke-width on arrows | INVESTIGATE |
| 13 | Arrow transform positions differ (e.g., `-59.61` vs `-59.97`) | Downstream of height diffs | FIX |
| 14 | Bold text: `width="74.99"` | `width="87"` | Bold "Control unit" text width differs | FIX — bold font metrics |
| 15 | `<pagination role="newpage"/>` | absent | Missing pagination | FIX |
| 16 | tex= on `<picture>` | absent | DESIGN #21 |
| 17 | `<!-- ... -->` comments | absent | DESIGN |

#### various_colors.xml — 25 diff hunks (after stripping tex= and %&#10;)

| # | Perl | Rust | Category | Disposition |
|---|------|------|----------|-------------|
| 1 | `<text color="#000000"></text>` trailing empty text | absent | Rust omits empty text wrappers | INVESTIGATE — may be Perl bug or needed for CSS |
| 2 | `text="...list@(T, w)..."` | `text="...delimited-⟨⟩@(list@(T, w))..."` | Math text attribute: angle-bracket fencing | DESIGN — intentional divergence (⟨a,b⟩ parsing) |
| 3 | `<XMTok meaning="list"/>` | `<XMTok meaning="delimited-⟨⟩"/>` | XMTok meaning for angle-bracket delimited list | DESIGN — intentional |
| 4 | Flat `T, w` in XMWrap | Wrapped in XMDual with list semantics | XMDual nesting for delimited expressions | DESIGN — intentional |
| 5 | `<XMRef idref="S2.E1.m1.m1.3"/>` | `<XMRef idref="S2.E1.m1.m1.4"/>` | xml:id renumbering | DESIGN #9 |
| 6 | `<para><p>2mm</p></para>` section between listings | absent | Missing `\vspace{2mm}` output | FIX — `\vspace` in vertical mode |
| 7 | Listings: `color="#FF0000"` on code tokens | `color="#000000"` | Listings escapechar: Perl renders red, Rust renders black | FIX — `escapechar=@` color scoping in listings |
| 8 | Listings: single `<text>` span for code line | Multiple `<text>` spans with identifier/keyword classes | Listings tokenization differs — Rust adds ltx_lst_keyword etc. | INVESTIGATE — Rust may be MORE correct (finer classification) |
| 9 | Listings: `//` comment detection produces `color="#FF0000"` | `(*{\color{red}//}*)` literal — escapechar not processed | FIX — `escapechar` combined with `\color{red}` inline not processed |
| 10 | Listings line 397: `color="#FF0000"` on `%[0:3]` | `color="#000000"` | Listings: moredelim color scoping differs | FIX — `moredelim` color not applied |
| 11 | tcolorbox `height="435.48"` | `height="291.77"` | tcolorbox box height 144pt off | DEFERRED — tcolorbox package port needed |
| 12 | tcolorbox title foreground: `color="#FFFFFF"` on foreignObject | `color="#FFFFFF"` on inner text | tcolorbox title color propagation differs | DEFERRED |
| 13 | tcolorbox minipage `width="40.23em"` | `width="402.3pt"` | Width units differ (em vs pt) | FIX — minipage width unit preservation |
| 14 | tcolorbox content height: `242.07` vs `225.47` | Downstream of #11 | DEFERRED |
| 15 | tex= on picture | absent | DESIGN #21 |
| 16 | `<!-- ... -->` comments | absent | DESIGN |

### Summary of dispositions

| Disposition | Count | Action |
|---|---|---|
| **DESIGN** | 16 | Already documented, no action needed |
| **FIX** | 22 | Code bugs to resolve |
| **INVESTIGATE** | 4 | Need root-cause analysis |
| **DEFERRED** | 4 | Require large feature ports (tcolorbox) |

### Priority FIX items (shared across tests)

1. **foreignObject transform Y=16.6** (3d-cone #5, ac-drive #5) — Perl uses fixed 12pt maxy; Rust uses actual height. Need to match Perl's `\pgfsys@foreignobject@maxy`.
2. **foreignObject width/height** (3d-cone #7/#8, ac-drive #7/#14) — Character width computation in `fo_get_size` differs from Perl.
3. **Nested minipage/SVG sizing** (ac-drive #8/#9/#10) — `appendNodeBox` content accumulation produces different dimensions than Perl's `pushContent`.
4. **Arrow tip shape** (ac-drive #11) — Different arrowhead path data.
5. **`<pagination role="newpage"/>`** (3d-cone #12, ac-drive #15) — Missing `\newpage` handling.
6. **SVG viewBox/width computation** (3d-cone #1, ac-drive #1) — Total dimensions differ slightly.
7. **Listings escapechar + color** (various_colors #7/#9/#10) — `escapechar=@` processing with `\color{red}` inline.
8. **Missing `\vspace{2mm}` output** (various_colors #6) — `\vspace` in vertical mode.

### Package bindings

- [x] physics.sty (800 lines, COMPLETE: 0 diffs, all meaning attrs match)
- [ ] siunitx.sty (2000 lines), xy.sty (1000 lines)
- [ ] tikz.sty+pgf.sty (8000 lines, 12 tests)
- [ ] expl3.sty (4000 lines, unlocks beamer/fontspec/unicode-math)
- [ ] babel.sty (3000 lines), biblatex.sty (2000 lines)
- [ ] moderncv.cls (2 tests), beamer.cls (2000 lines)

### Infrastructure projects

- [ ] **F. Post-processing pipeline** — 25 modules, 0% ported (~7000 lines). Prototype in worktree `latexml-post-first-prototype`.
- [x] **G. Codegen: `Until:` parameter type** — Added `Until` → `Tokens` mapping to `parameter_rust_type!` macro. `DefMacro!/DefPrimitive!` closures now work with `Until:\cs` parameter specs. pgfcircutils.tex updated to use proper DefPrimitive instead of RawTeX workaround.
- [x] **H. pgfsys pattern system** — 7 pattern definitions ported (declarepattern, setpattern, colored/uncolored pattern constructors). Remaining: tikz matrix alignment (`\lxSVG@halign` + 5 helper subs).

### Permanent ignores (5)

- **ns1–ns5** (52_namespace) — DTD not supported in Rust port.

---

> **Reminder:** Every entry ported from Perl must follow tightly the original semantics and nuances. Read the Perl source, translate precisely, preserve edge cases. The Perl code is the ground truth.
