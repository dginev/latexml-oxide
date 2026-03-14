# Engine Sync Status: Perl vs Rust

Updated 2026-03-14. Only lists open gaps & TODOs; completed items live in git history.

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
| base_xmath.rs | GAPS | ~24 commented-out defs (matrix/cases systems, `\lx@padded`, tweaked). Done: `\lx@apply`, `\lx@symbol`, `\lx@wrap`, `\lx@superscript/subscript`. Missing: `openMathFork()`, `closeMathFork()`, `MathWhatsit()`, equation group helpers |
| base_functions.rs | MINOR | — |

### Phase 1: TeX Primitives (High-Gap)

| File | Status | Open Gaps |
|------|--------|-----------|
| tex_math.rs | GAPS | Missing: `\nonscript`, `\lx@dollar@default`, `TeXDelimiter` param type, `adjustMathRole()`, math ligatures. `\mathchoice` ported. CS names synced: `\lx@hidden@egroup@right`, `\lx@right` (was `\right@hidden@egroup`, `\@right`). `\left` now unreads `\lx@hidden@bgroup` (was `\@hidden@bgroup`). |
| tex_box.rs | GAPS | Missing: `\leaders/cleaders/xleaders` (needed for tabbing), SVG functions (`collapseSVGGroup` etc), `\hbox/vbox/vtop` have many TODOs, `\vrule/\hrule` mostly commented out |
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
| latex_ch10_tabbing_environment.rs | EMPTY | 0% |
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

---

## Cross-Cutting Infrastructure Gaps

1. **`FontDef` parameter type** — Simplified to `FontToken`. Blocks `\fontdimen`, `\hyphenchar` per-font tracking.
2. **SVG support** — Removed from glue/kern/box. Not critical for XML output.
3. **`LoadFormat` machinery** — Not ported; plain/latex bootstrap loaded inline.
4. **Font `computeBoxesSize`** — Single-pass only; missing word/line/stack decomposition.
5. **`DEFSIZE`** — Static 10.0; Perl reads `NOMINAL_FONT_SIZE` from state.
6. **`.ltxml` file search** — Commented out in `content.rs` (line 946-959). Perl searches for `.ltxml` files in search paths; Rust only searches for raw `.sty`/`.tex` files. Blocks loading test-local package bindings.

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
| document.rs | MINOR | `compact_xmdual()`, `insertElementBefore()`, comment creation (needs libxml). Fixed: `close_to_node` ifopen parameter now suppresses error (was ignored). `close_node_with_strictness` walker now tracks `n.get_type()` (was `node.get_type()`). `mergeAttributes` now uses `add_ss_values` for space-joined attrs (class/lists/inlist/labels), matching Perl's sort+dedup. `finalize_rec` class merge also sorts. |
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
| amsmath.sty | GAPS | ~55% ported: spacing, \overunderset, \lvert/rvert/lVert/rVert, \notag, \tag, \tfrac/\dfrac, \xrightarrow/\xleftarrow, over/under arrows, alignment infrastructure (`ams_rearrangeable_bindings`, `ams_gather_bindings`, `ams_align_bindings`, `ams_aligned_bindings`), environment macros for {gather}/{gather*}/{align}/{align*}/{flalign}/{flalign*}/{alignat}/{alignat*}/{multline}/{multline*}/{split}/{gathered}/{aligned}/{alignedat}, `\@ams@intertext{}` constructor, `\lx@ams@cr@binding` primitive. Missing: afterConstruct DOM rearrangement (`rearrangeAMSAlign`/`rearrangeAMSGather`), `\text{}`, operators, `{subequations}`. |
| appendix.sty | OK | Core environments: appendices, subappendices, conditional switches. |
| multicol.sty | OK | Full port: multicols/multicols* environments, registers, stubs. |
| booktabs.sty | OK | Full port: toprule/midrule/bottomrule/cmidrule/specialrule, registers. |
| caption.sty | MINOR | Stub-level: captionsetup, Declare* macros, registers. Missing: KeyVals, CAPTION_ value storage. |
| remreset.sty | OK | Empty stub (obsolete, macros moved to LaTeX core). |
| chngcntr.sty | OK | Empty stub (obsolete, macros moved to LaTeX core). |
| listings.sty | GAPS | Core infrastructure ported: `lstActivate`, `\@listings@inline`, `\lstdefinelanguage`, `\@listingGroup`, `\@listingKeyword`, `lstClassBegin/End`, language loading (lstlang0-3). Remaining: index generation (`lst@@index`), `literate` key, `extendedchars`, display listings have many diffs. |
| ntheorem.sty | GAPS | Framing constructors (`\lx@addframing`, `\lx@@snapshot@framing`) ported. Missing: `\colorbox` (needs xcolor port) for shaded theorems, `backgroundcolor` attribute. |

---

## Test Suite Status (2026-03-14)

**Current totals: 185 pass, 0 fail, 56 ignored test functions**
**Perl total: ~315 test cases across 26 latexml_tests() suites + ~9 special tests**
**Coverage: 59% of Perl test cases passing**

| Suite | Pass/Total | Notes |
|-------|-----------|-------|
| 000_hello | 1/1 | |
| 00_tokenize | 14/14 | All pass |
| 00_contrib | 1/1 | All pass |
| 01_unit_tokens | 1/1 | |
| 01_unit_state | 9/9 | |
| 10_expansion | 36/36 | All pass (Rust 48 .tex, Perl 47) |
| 12_grouping | 2/2 | All pass |
| 20_digestion | 10/10 | All pass |
| 22_fonts | 10/23 | 10 pass; 13 ignored (cancels, soul, mathcolor un-ignored) |
| 30_encoding | 26/26 | All pass |
| 32_keyval | 7/8 | 7 pass; 1 ignored (xkeyvalview) |
| 33_keyval_options | 11/11 | All pass |
| 50_structure | 24/24 | All pass (18 .todo disabled at build level) |
| 52_namespace | 0/5 | **All ignored** — DTD not supported in Rust port |
| 53_alignment | 12/28 | halign, tabtab, tabularstar, morse, mathmix, halignatt, longtable, min_listing, min_listing_data, min_listing_lang, min_listing_short, min_listing_string pass; 16 ignored |
| 55_theorem | 4/4 | All pass (ntheorem disabled) |
| 56_ams | 1/7 | genfracs pass; 6 ignored (need afterConstruct DOM rearrangement for MathFork/MathBranch) |
| 65_graphics | 5/9 | 5 pass; 4 ignored |
| 70_parse | 0/1 | **All ignored** — math parser regression tests |
| 700_unit_parse | 3/3 | |
| 80_complex | 8/16 | xii, figure_dual_caption, hyperchars, hypertest, versioned_fallback, equationnest, tcilatex_minimal, labelled pass; 8 ignored |
| 81_babel | 0/1 | **All ignored** — needs babel language `.ldf` files |

### Perl-only tests (not yet copied to Rust)

| Suite | Perl-only tests | Notes |
|-------|----------------|-------|
| graphics | xytest | 1 missing .tex/.xml |
| keyval | keyvalemptyvalue | 1 missing .tex/.xml |
| moderncv | orc | 1 missing .tex/.xml |
| complex | 15 tests | .tex/.xml not copied; 4 Perl-new (no .xml in Perl either) |
| pgf | 2 tests | Entirely unported suite |
| tikz | 10 tests | Entirely unported suite |

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

### ntheorem test gap analysis
- **Math parser tree structure** (873/896 diffs): XMApp/XMTok nesting differs due to Marpa-based parser architecture. Not fixable without parser changes (active research).
- **eqnarray** (23/896 diffs): Now uses alignment-based `eqnarray_bindings()` with 3-column `$\displaystyle` template and equationgroup/equation/_Capture_ hooks. Still missing: afterConstruct `rearrangeEqnarray` post-processing (~200 lines Perl) that transforms `_Capture_` into `MathFork`/`MathBranch`/`tr`/`td` structure.
- **Equation numbering**: Offset by ~1 due to simplified eqnarray not splitting rows.
- **Shaded theorems**: `backgroundcolor` attribute missing (needs `\colorbox` from xcolor.sty).

---

## Roadmap to Full Parity — Ambitious Plan

### Philosophy
We aim for **complete Perl test parity**. No test is too hard — some just need prerequisite infrastructure first. This plan attacks root causes that block the most tests, not just easy wins.

### Phase 1: Infrastructure Unlocks (target: 139 → ~180)

These are foundational fixes that unblock large batches of tests across multiple suites.

#### 1A. Port xkeyval.sty.ltxml — unlocks 17 tests
**Blocked tests:** 5 keyval tests + 11 keyval_options tests + keyvalstyle_test
**Root cause:** No Rust binding → raw TeX fallback → `\input xkeyval.sty` → recursive chain → infinite loop.
**Work:** Port `xkeyval.sty.ltxml` (~300 lines Perl). Key constructs: `\define@key`, `\setkeys`, `\presetkeys`, `\savekeys`, option processing with `\XKV@` prefix. Also need test-local `.sty.ltxml` loading for `mykeyval`/`myxkeyval` test stubs.
**Effort:** Medium — systematic macro definitions, well-understood keyval semantics.
**Yield:** +17 tests

#### 1B. Port local .sty.ltxml loading — unlocks keyvalstyle + structure .todo tests
**Root cause:** Tests with custom `.sty.ltxml` files in the test directory can't load them because the package loader only searches compiled-in packages.
**Work:** Add test-directory search path to `FindFile`/`RequirePackage`. When processing `\usepackage{foo}`, also check the directory of the `.tex` file being processed for `foo.sty.ltxml`.
**Effort:** Small — pathname.rs + package loader change.
**Yield:** Unlocks keyvalstyle_test + many structure tests that use custom class files.

#### 1C. Port missing structure packages — unlocks 18 .todo tests
**Blocked tests:** acro, amsarticle, bibsect, crazybib, csquotes, enum, eqnums, figure_grids, figures, filelist, floatnames, glossary, IEEE, natbib, options, paralists, subcaption, svabstract
**Work per test (roughly grouped):**
- **Easy (stub-only):** options, filelist, floatnames — just need option processing
- **Medium:** enum/paralists (enumitem.sty), figures/figure_grids/subcaption (subfigure/subcaption.sty), svabstract (svjour.cls)
- **Hard:** csquotes (context-sensitive quoting), natbib (bibliography), amsarticle (amsart.cls), IEEE (IEEEtran.cls), glossary (glossaries.sty), acro (acro.sty), eqnums (equation numbering), bibsect/crazybib (bibliography)
**Effort:** High aggregate — ~15 new package stubs/ports. But each is independent work.
**Yield:** +18 tests

#### 1D. Fix "misdefined expansion" warning
**Impact:** Appears in almost every test (non-fatal but noisy). Some macro definition is producing `#` in expansion text without proper `#1`-`#9` or `##` form.
**Work:** Track down the source — likely a recently-added macro with bare `#` in its body.
**Effort:** Small — diagnostic + fix.

### Phase 2: Font System Completion (target: ~180 → ~195)

#### 2A. Per-font \hyphenchar tracking
**Blocked tests:** fonts_test, plainfonts_test, sizes_test
**Root cause:** `\hyphenchar` is global — should be per-font. `\font\myfont=cmr10` stores hyphenchar=45, `\hyphenchar\myfont=99` should update only that font's hyphenchar.
**Work:** Store hyphenchar in the per-font `fontinfo_\cs` state. `\hyphenchar` getter/setter reads/writes from fontinfo.
**Effort:** Small-medium.

#### 2B. \fontname "select font X at Ypt" format
**Blocked tests:** plainfonts_test (all font descriptions wrong format)
**Root cause:** `\fontname` returns just the font filename, not "select font cmr10 at 5.0pt" for scaled fonts.
**Work:** Store the full font description string (including `at Xpt` / `scaled Y`) in fontinfo at `\font` definition time. `\fontname` returns this stored string.
**Effort:** Small.

#### 2C. Font size effect on text rendering
**Blocked tests:** plainfonts_test, sizes_test (376 diffs)
**Root cause:** `\font\myrmfiveA=cmr10 at 5pt` should produce `<text fontsize="50%">` wrapping. Currently the font size from `\font` definitions isn't propagated to the Font struct's `size` field.
**Work:** In `decode_fontname`, extract size from the font command parameters. Store in Font struct. `relative_to()` then produces correct fontsize attribute.
**Effort:** Medium — touches font pipeline.

#### 2D. Port missing font package bindings
**Blocked tests:** soul (needs expl3), stmaryrd, wasysym, cancels, mathcolor (need expl3/graphics), pifont (ding), abxtest (font allocation)
**Work:** Each needs its `.sty.ltxml` port. stmaryrd is a font symbol map (like esint/marvosym — already done pattern). pifont needs pzd font map.
**Effort:** Medium per package. stmaryrd/pifont are tractable. soul/wasysym/cancels/mathcolor blocked on expl3.
**Yield:** +3–7 font tests

### Phase 3: Alignment Engine (target: ~195 → ~215)

#### 3A. ~~Fix nested tabular handling — unlocks tabtab_test~~ DONE
**Fixed:** Sizer string parsing (`IntoOption<SizingClosure> for &str`) only handled `#digit` patterns, not `#property_name`. The `\lx@begin@alignment` constructor uses `sizer => "#alignment"` which was being parsed as arg 1 instead of property lookup. Fixed in `traits.rs` to match Perl's `$sizer =~ /^(#\w+)*$/` pattern.

#### 3B. ~~Fix nested halign alignment — unlocks halign_test~~ DONE
**Fixed:** Two bugs in `align_group_count` (`$ALIGN_STATE`) tracking:
1. `unread_one()` didn't adjust agc when unreading `{`/`}` tokens (Perl's `unread()` always adjusts).
2. `stomach::bgroup()/egroup()` incorrectly adjusted agc (Perl only tracks at scan level in Gullet).
These caused `handle_template` not to fire for outer alignment columns after nested `\vbox{\halign{...}}`.

#### 3C. Port alignment packages — unlocks remaining 20 alignment tests
**Packages needed:** longtable.sty, multirow.sty, supertabular.sty, colortbl.sty, rotating.sty, tabbing environment, listings.sty
**Work:** Each is a separate port. longtable and multirow are the most common. listings is large (~500 lines Perl).
**Effort:** High aggregate.
**Yield:** +5–20 tests

### Phase 4: Math Parser + Math Tests (target: ~215 → ~260)

#### 4A. Math parser regression tests — 28 tests
**Work:** Run all 28 70_parse tests. Categorize: (a) bugs in Rust code feeding parser, (b) intentional Marpa divergence. For (a), fix. For (b), generate Rust-specific expected XMLs by running `latexmlmath_oxide` and validating output manually.
**Effort:** High — each test needs individual analysis.
**Yield:** +10–28 tests

#### 4B. Math pipeline tests — 14 tests
**Work:** Enable 40_math tests. Many will have non-parser issues (missing `\DeclareMathOperator`, delimiter macros, spacing). Fix those first, then address parser tree diffs.
**Effort:** Medium-high.
**Yield:** +5–14 tests

#### 4C. AMS math tests — 7 tests
**Work:** Complete amsmath.sty port: `{align}`, `{gather}`, `{multline}`, `{split}`, `\text{}`, `\intertext`, math operators. Port `amscd.sty.ltxml` for CD diagrams.
**Effort:** High — `{align}` needs full alignment-based equation groups.
**Yield:** +3–7 tests

#### 4D. eqnarray with MathFork/MathBranch
**Work:** Implement full 3-column eqnarray with `$\displaystyle` template, `rearrangeEqnarray` post-processing. Needed by ntheorem and several structure tests.
**Effort:** High (~200 lines Perl to port).
**Yield:** Unlocks ntheorem + several equation tests.

### Phase 5: Graphics + Color (target: ~260 → ~275)

#### 5A. Port color.sty.ltxml — unlocks graphics tests + downstream
**Root cause:** `\usepackage{color}` causes infinite recursion.
**Work:** Full port of `color.sty.ltxml` (~200 lines). Key: `\color`, `\textcolor`, `\colorbox`, `\fcolorbox`, color models (rgb, cmyk, named). Also port `dvipsnam.def.ltxml` for named colors.
**Effort:** Medium.
**Yield:** Unlocks graphics tests + shaded theorem backgrounds + many structure tests.

#### 5B. Port xcolor.sty.ltxml
**Work:** Extended color package — `\definecolor`, `\colorlet`, color mixing, tints. Used by many modern packages.
**Effort:** Medium-high.
**Yield:** Unlocks packages that depend on xcolor (tikz, beamer, many structure tests).

#### 5C. Port graphicx improvements + picture environment
**Work:** `\includegraphics` options, picture environment (`\put`, `\line`, `\circle`, `\oval`).
**Effort:** High for picture env.
**Yield:** +3–8 graphics tests

#### 5D. Port pgf/tikz — 12 new tests
**Work:** PGF is a massive library (~5000 lines in Perl bindings). TikZ builds on PGF. Both need `\pgfkeys` (key-value on steroids), coordinate parsing, path construction.
**Effort:** Very high. Consider stub approach first.
**Yield:** +2–12 tests (pgf 2, tikz 10)

### Phase 6: Language + Internationalization (target: ~275 → ~285)

#### 6A. Port babel.sty.ltxml — unlocks 6 babel tests
**Work:** Language selection system, `\selectlanguage`, `\foreignlanguage`, `\otherlanguage`. Port frenchb, germanb, greek `.ldf` files.
**Effort:** Medium — well-defined structure.
**Yield:** +2–6 babel tests

#### 6B. Port csquotes.sty.ltxml
**Work:** Context-sensitive quotation marks. Used by some structure tests.
**Effort:** Medium.
**Yield:** Unlocks csquotes structure test.

### Phase 7: ~~Namespace + Document Bindings~~ REMOVED

DTD support removed from Rust port (decision 2026-03-13). Only RelaxNG schemas supported.
Namespace tests (ns1–ns5) permanently ignored. xii.tex converted to use standard LaTeXML elements.

### Phase 8: Complex Integration + New Suites (target: ~290 → ~320+)

#### 8A. Port complex tests incrementally
**Easy tier:** tcilatex_minimal, versioned_fallback, hypertest — small, few dependencies.
**Medium tier:** labelled, aastex_test, hyperurls — need cleveref, aastex631.cls.
**Hard tier:** physics, si, figure_mixed_content — massive output, many packages (siunitx, physics, wrapfig, algorithm).
**Yield:** +3–16 tests

#### 8B. Port expl3 programming layer — unlocks 2 expl3 tests + many packages
**Work:** LaTeX3 `\ExplSyntaxOn/Off`, `\cs_new:Npn`, `\tl_set:Nn`, etc. This is architecturally significant — many modern packages (soul, fontspec, unicode-math) depend on expl3.
**Effort:** Very high — new programming paradigm within TeX.
**Yield:** +2 direct tests, unlocks soul/wasysym/cancels/mathcolor/fontspec and modern packages.

#### 8C. Port beamer.cls.ltxml — unlocks 2 slides tests
**Work:** Presentation class with frames, overlays, themes. Complex but well-structured.
**Effort:** High.
**Yield:** +2 tests

#### 8D. Port moderncv.cls.ltxml — unlocks 2 moderncv tests
**Work:** CV/resume class with custom layout.
**Effort:** Medium.
**Yield:** +1–2 tests

### Phase 9: Post-Processing Pipeline (stretch goal)

#### 9A. Port post-processing (XSLT → Rust)
**Work:** The Perl pipeline has post-processing stages: bibliography resolution, cross-referencing, MathML generation, HTML5 output. Currently Rust only produces the intermediate XML.
**Effort:** Very high — entire new subsystem.
**Yield:** Enables format/profile/EPUB tests.

### Summary: Expected Progression

| Phase | Cumulative Tests | Delta | Key Infrastructure |
|-------|-----------------|-------|--------------------|
| Current | 185 | — | — |
| Phase 1 (infrastructure) | ~190 | +35 | local .ltxml, structure packages |
| Phase 2 (fonts) | ~205 | +15 | per-font hyphenchar, fontname format, font sizes |
| Phase 3 (alignment) | ~225 | +20 | nested tabular, alignment packages |
| Phase 4 (math) | ~270 | +45 | parser tests, amsmath, eqnarray |
| Phase 5 (graphics) | ~285 | +15 | color, xcolor, graphicx, pgf/tikz |
| Phase 6 (languages) | ~295 | +10 | babel, csquotes |
| Phase 7 (namespace) | — | — | REMOVED (DTD not supported) |
| Phase 8 (integration) | ~325 | +30 | complex tests, expl3, beamer |
| Phase 9 (post-proc) | ~354 | +29 | XSLT, MathML, HTML5 output |
| **Full Perl parity** | **~354** | | **all suites, all tests** |

### Critical Path Dependencies

```
expl3 ──→ soul, wasysym, cancels, mathcolor, fontspec, beamer
color.sty ──→ xcolor ──→ tikz, beamer, colortbl, ntheorem backgrounds
alignment engine ──→ eqnarray ──→ ntheorem, amsmath {align}
local .ltxml loading ──→ keyvalstyle, structure .todo tests
```

### Ignored Tests — Ranked Priority (fewest diffs → most diffs)

| Priority | Test | Diffs | Blocker |
|----------|------|-------|---------|
| 1 | tcilatex_minimal | 7 | `\TEXUX` undefined — needs tcilatex binding |
| 2 | hypertest | 16 | prefix= namespace decls, color wrapping |
| 3 | xkeyvalstyle | 16 | `\ProcessOptionsX` style handler |
| 4 | aastex_test | 18 | Output truncates early (missing aastex.cls) |
| 5 | ns1–ns5 | 19 each | DTD not supported (permanent ignore) |
| 6 | keyvalstyle | 29 | Keyval style attributes mishandled |
| 7 | aastex631_deluxetable | 31 | `\deluxetable*` undefined |
| 8 | longtable | 59 | longtable.sty incomplete |
| 9 | morse | 103 | Large diffs |
| 10 | aliceblog | 144 | blog.cls missing, large diffs |
| 11 | tabbing | 162 | Tabbing environment unported |
| 12 | algx | 168 | algorithmic package |
| 13 | xkeyvalview | 178 | Large diffs |
| 14 | supertabular | 315 | `\@makecaption` undefined |
| 15 | tabular | 369 | Deep tabular issues |
| 16 | xcolors | 652 | color system diffs |
| 17 | picture | 3124 | Picture environment unported |
| 18 | physics | 5417 | Massive diffs (was crash, now runs) |

**Crashes (need code fixes):**
- ~~halignatt~~: FIXED — hackVBoxAttachment now walks Lists to find \halign alignment
- colortbls: normalize.rs crash fixed; now hits TooManyErrors (colortbl.sty not ported)
- cells + listing + graphrot + xytest: TooManyErrors (>100 undefined errors)
- figure_mixed_content: `\lstKV@SetIf@` param spec error (listings.sty)
- cd_test: math parser `replacing tree should always work`
- babel: infinite loop (timeout)

### Immediate Next Actions (prioritized)
1. ~~Fix alignment.rs:317 crash~~ DONE — halignatt now shows 2 vattach diffs (needs insert_block refactor)
2. ~~Fix colortbls normalize.rs:403 crash~~ normalize.rs rewritten — retest needed
3. Complete Document.pm audit (10-part sub-audit in progress)
4. Port tcilatex binding (7 diffs → pass)
5. Fix hypertest namespace/color issues (16 diffs)
6. Implement local .sty.ltxml loading from test directories
7. Port color.sty.ltxml (unlocks graphics + downstream)
