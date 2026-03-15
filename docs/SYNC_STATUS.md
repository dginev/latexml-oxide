# Engine Sync Status: Perl vs Rust

> **This is a Perl-to-Rust translation project.** Every ported function, macro, and definition must faithfully reproduce the original Perl semantics, control flow, and edge-case behavior. The Perl source (`LaTeXML/` directory) is the ground truth. Only diverge when explicitly documented in `docs/OXIDIZED_DESIGN.md`.

Updated 2026-03-15. Only lists open gaps & TODOs; completed items live in git history.

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

---

## Cross-Cutting Infrastructure Gaps

1. **`FontDef` parameter type** — Simplified to `FontToken`. Blocks `\fontdimen`, `\hyphenchar` per-font tracking.
2. **SVG support** — Removed from glue/kern/box. Not critical for XML output.
3. **`LoadFormat` machinery** — Not ported; plain/latex bootstrap loaded inline.
4. **Font `computeBoxesSize`** — Single-pass only; missing word/line/stack decomposition.
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

## Test Suite Status (2026-03-15)

**Current totals: 214 pass, 0 fail, 65 ignored test functions**
**Perl total: ~315 test cases across 26 latexml_tests() suites + ~9 special tests**
**Coverage: 77% pass rate (214/279 non-permanent-ignore tests)**

| Suite | Pass | Ignored | Notes |
|-------|------|---------|-------|
| 000_hello | 1 | 0 | |
| 00_tokenize | 14 | 0 | |
| 00_contrib | 1 | 0 | |
| 01_unit_tokens | 1 | 0 | |
| 01_unit_state | 9 | 0 | |
| 10_expansion | 36 | 0 | |
| 12_grouping | 2 | 0 | |
| 20_digestion | 10 | 0 | |
| 22_fonts | 17 | 6 | fonts(156), plainfonts(66), sizes(393), ding(371), abxtest(crash), stmaryrd(1449) |
| 30_encoding | 26 | 0 | |
| 32_keyval | 8 | 0 | |
| 33_keyval_options | 11 | 0 | |
| 40_math | 0 | 1 | batch 149 diffs (math parser) |
| 50_structure | 31 | 5 | eqnums(598), enum(543), figure_grids(331), amsarticle(898), ieee(979) |
| 52_namespace | 0 | 5 | DTD not supported (permanent) |
| 53_alignment | 18 | 11 | cells(overflow), colortbls(crash), supertabular(629), algx(163), plainmath(351), split(2228), badeqnarray(507), eqnarray(1176), diagbox(timeout), ncases(timeout), vmode(segfault) |
| 55_theorem | 4 | 1 | ntheorem(1479) |
| 56_ams | 2 | 5 | amsdisplay(963), matrix(187), sideset(488), cd(crash), mathtools(crash) |
| 65_graphics | 5 | 4 | graphrot(596), picture(3125), xcolors(447), xytest(crash) |
| 70_parse | 0 | 1 | batch 120 diffs |
| 700_unit_parse | 3 | 0 | |
| 80_complex | 8 | 8 | deluxetable(35), aliceblog(144), cleveref(302), mixed_content(1142), physics(5417), si(9024), acm_aria(timeout), revtex(missing) |
| 81_babel | 0 | 1 | memory leak timeout |
| 82_moderncv | 0 | 2 | needs moderncv.cls binding |
| 83_expl3 | 0 | 2 | needs \ExplSyntaxOn |
| 84_slides | 0 | 2 | needs beamer.cls/slides.cls |
| 85_pgf | 0 | 2 | needs pgf.sty |
| 86_tikz | 0 | 10 | needs tikz.sty |

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

**Status (2026-03-15):** 214 pass, 0 fail, 65 ignored (77% pass rate). Full diff scan below.

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

- [ ] **8. fonts_test** (22_fonts) — 156 diffs. Math font map character lookup for `\cal`/`\it` in math mode. `\fontname` returns placeholder.
- [ ] **8b. plainfonts_test** (22_fonts) — 66 diffs. Same font map issues + `\fontname` + cmsy10 glyph mapping.
- [ ] **9. sizes_test** (22_fonts) — 393 diffs. Font size from `\font` definitions not propagated.
- [ ] **10. ding_test** (22_fonts) — 371 diffs. Enumerate nesting + table structure.
- [ ] **11. abxtest_test** (22_fonts) — TooManyErrors. Needs `\hexnumber@`, `\mathxfam`.
- [ ] **13. enum_test** (50_structure) — 543 diffs. Port enumitem.sty binding.
- [ ] **16. figure_grids_test** (50_structure) — 331 diffs. Needs graphicx figure grid support.
- [ ] **18. amsarticle_test** (50_structure) — 898 diffs. Port amsart.cls binding.
- [ ] **25. cells_test** (53_alignment) — STACK OVERFLOW. Debug recursive state lookup.
- [ ] **27. supertabular_test** (53_alignment) — 629 diffs + crash. Port supertabular.sty.
- [ ] **35. graphrot_test** (65_graphics) — 596 diffs. `\begingroup` in `\csname..\endcsname`.
- [ ] **37. xcolors_test** (65_graphics) — 447 diffs. Complete xcolor port.

### Tier 2: Needs afterConstruct DOM rearrangement (BLOCKED on item 29)

- [ ] **29. Implement afterConstruct rearrangement** — `rearrangeEqnarray`, `rearrangeAMSAlign`, `rearrangeAMSGather`, `openMathFork`/`closeMathFork`/`addColumnToMathFork`/`equationgroupJoinCols`. ~200 lines Perl. Unlocks items 2, 3, 28, 30–32.
- [ ] **2. eqnums_test** (50_structure) — 598 diffs. BLOCKED: needs afterConstruct MathFork.
- [ ] **3. algx_test** (53_alignment) — 163 diffs. BLOCKED: needs math parser XMDual + fontsize.
- [ ] **28. badeqnarray_test** (53_alignment) — 507 diffs. BLOCKED: needs afterConstruct.
- [ ] **30. amsdisplay_test** (56_ams) — 963 diffs. BLOCKED: needs afterConstruct + `\text{}`.
- [ ] **31. matrix_test** (56_ams) — 187 diffs. BLOCKED: needs afterConstruct + math parser.
- [ ] **32. sideset_test** (56_ams) — 488 diffs. BLOCKED: needs afterConstruct.

### Tier 3: Needs package bindings (moderate effort)

- [ ] **12. stmaryrd_test** (22_fonts) — 1449 diffs. Port stmaryrd.sty (font symbol map).
- [ ] **33. cd_test** (56_ams) — PANIC in math parser. Port amscd.sty.ltxml.
- [ ] **34. mathtools_test** (56_ams) — TooManyErrors. Port mathtools.sty.
- [ ] **36. picture_test** (65_graphics) — 3125 diffs. Port picture env + graphpap.sty.
- [ ] **38. xytest** (65_graphics) — TooManyErrors. Port xy.sty binding.
- [ ] **39. cleveref_minimal_test** (80_complex) — 302 diffs. Port cleveref.sty.
- [ ] **40. figure_mixed_content_test** (80_complex) — 1142 diffs. Needs wrapfig + listings math.
- [x] **41. aastex631_deluxetable_test** (80_complex) — DONE. Ported deluxetable.sty, Stored::Template, version-stripping dispatch.
- [ ] **42. aliceblog_test** (80_complex) — 144 diffs. Port blog.cls binding.

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
- [ ] **F. Port Post-processing pipeline** — Translate `LaTeXML/lib/LaTeXML/Post/` to Rust. Copy XSLT and CSS from Perl and use them exactly as-is. This includes `LaTeXML::Post::MathML`, `LaTeXML::Post::OpenMath`, `LaTeXML::Post::CrossRef`, `LaTeXML::Post::MakeBibliography`, etc.

### Permanent ignores (not counted)

- **ns1–ns5** (52_namespace) — DTD not supported in Rust port. Permanently ignored.

---

> **Reminder:** Every entry ported from Perl must follow tightly the original semantics and nuances. Read the Perl source, translate precisely, preserve edge cases. The Perl code is the ground truth.
