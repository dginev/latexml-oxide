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

## Test Suite Status (2026-03-14)

**Current totals: 207 pass, 0 fail, 54 ignored test functions (updated 2026-03-15)**
**Perl total: ~315 test cases across 26 latexml_tests() suites + ~9 special tests**
**Coverage: 60% of Perl test cases passing**

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
| 22_fonts | 17 | 6 | fonts, plainfonts, ding, stmaryrd, abxtest, sizes |
| 30_encoding | 26 | 0 | |
| 32_keyval | 8 | 0 | xkeyvalview now passing |
| 33_keyval_options | 11 | 0 | |
| 40_math | 0 | 1 | math parser deferred |
| 50_structure | 31 | 11 | package bindings needed |
| 52_namespace | 0 | 5 | DTD not supported (permanent) |
| 53_alignment | 18 | 11 | crashes + math parser |
| 55_theorem | 4 | 1 | ntheorem (math parser) |
| 56_ams | 2 | 5 | afterConstruct rearrangement |
| 65_graphics | 5 | 4 | package bindings + xcolor |
| 70_parse | 0 | 1 | math parser regression |
| 700_unit_parse | 3 | 0 | |
| 80_complex | 8 | 8 | labelled, tcilatex_minimal, hypertest newly passing |
| 81_babel | 0 | 1 | memory leak |

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

---

## Work Plan — Ordered TODO List

Follow this list in order. Work on the first unchecked `[ ]` item. Only investigate what to do next when all items are clearly completed.

### Tier 1: Small fixes to un-ignore existing tests (1–2 tests each)

- [x] **1. xkeyvalview_test** (32_keyval) — DONE. Ported `\xkvview` constructor with typewriter font, table counter, XKVVIEW_TRACKING.
- [ ] **2. eqnums_test** (50_structure) — 0 errors, 362 diffs. Fixed agc drift (read_match, read_until_brace, newline@noskip extra }). Remaining: equation numbering/tags/MathFork infrastructure for AMS align, font-italic in tag primes.
- [ ] **3. algx_test** (53_alignment) — 100 diffs. algorithmicx ported but `\csname` expansion errors in nested `\ALG@bl@...` macros. Fix `\csname`/`\edef` in gullet.rs.
- [x] **4. figures_test** (50_structure) — DONE. Ported subfigure.sty font options (DeclareOption handlers).
- [x] **5. floatnames_test** (50_structure) — DONE. Ported `\newfloat` (float.sty) and `\DeclareFloatingEnvironment` (newfloat.sty) with full float environment creation, beforeFloat/afterFloat, addFloatFrames.
- [x] **6. filelist_test** (50_structure) — DONE. Fixed `\@filelist` init to `\@gobble` (matching latex.ltx), added `\@addtofilelist` in `input_definitions`, added `RequirePackage!("textcomp")` to latex.rs engine.
- [x] **7. options_test** (50_structure) — DONE. Created myclass.cls + apackage.sty Rust bindings in latexml_contrib.

### Tier 2: Font system improvements (unlocks 2–6 font tests)

- [ ] **8. fonts_test + plainfonts_test** (22_fonts) — `\fontname` format: need "select font X at Ypt" instead of just filename. Store full description in fontinfo at `\font` definition time.
- [ ] **9. sizes_test** (22_fonts) — 377+ diffs. Font size from `\font` definitions not propagated to Font struct. `\font\myfont=cmr10 at 5pt` → `fontsize="50%"`.
- [ ] **10. ding_test** (22_fonts) — pifont.sty + pzd font map DONE. 877 diffs from enumerate nesting + table structure issues.
- [ ] **11. abxtest_test** (22_fonts) — needs `\hexnumber@`, `\mathxfam` (font allocation macros).
- [ ] **12. stmaryrd_test** (22_fonts) — needs stmaryrd.sty binding (font symbol map, same pattern as esint/marvosym).

### Tier 3: Package bindings for structure tests (unlocks 1–3 tests each)

- [ ] **13. enum_test** (50_structure) — port enumitem.sty binding (custom list environments).
- [x] **14. paralists_test** (50_structure) — DONE. Added `set_enumeration_style`/`set_itemization_style`, `afterDigestBegin` hooks, conditional stock enum/itemize redefinition.
- [x] **15. subcaption_test** (50_structure) — DONE. Ported subcaption.sty with beforeFloat/afterFloat preincrement, collapseFloat, \format@title@subfigure. Fixed rescue_caption_counters false-value pollution.
- [ ] **16. figure_grids_test** (50_structure) — needs graphicx figure grid support.
- [x] **17. natbib_test** (50_structure) — DONE. Full natbib.sty port: citation styles (authoryear/numbers/super), \cite/\citet/\citep/\citealt/\citealp/\citeauthor/\citeyear/\citeyearpar, \bibpunct, \setcitestyle, \bibstyle@* dispatchers, \NAT@wrout/\NAT@@wrout tag generation, \lx@NAT@parselabel bibitem parsing. Fixed Invocation! arg-shifting for None required params.
- [ ] **18. amsarticle_test** (50_structure) — port amsart.cls binding.
- [ ] **19. csquotes_test** (50_structure) — port csquotes.sty binding (context-sensitive quoting).
- [x] **20. svabstract_test** (50_structure) — DONE. Ported svjour.cls + sv_support.sty + inst_support.sty bindings.
- [ ] **21. ieee_test** (50_structure) — port IEEEtran.cls binding.
- [x] **22. acro_test** (50_structure) — DONE. Ported acronym.sty binding + addIndexPhraseKey afterClose hook.
- [x] **23. glossary_test** (50_structure) — DONE. Implemented glossaries.sty binding with \newglossaryentry, \longnewglossaryentry, \newacronym (state-stored entries), \gls/\Gls/\glspl/\Glspl/\glssymbol (runtime macros with first-use tracking), \printglossary/\printnoidxglossaries.
- [x] **24. bibsect_test** (50_structure) — DONE. Ported \lx@bibliography, \bibstyle constructors, bibunits.sty binding.
- [x] **24b. crazybib_test** (50_structure) — DONE. Implemented `\bibsection` parsing in `begin_bibliography_clean`: deciphers sectional unit from expansion tokens, updates BACKMATTER_ELEMENT mapping, extracts custom title.

### Tier 4: Alignment crashes and diffs (unlocks 3–8 tests)

- [ ] **25. cells_test** (53_alignment) — stack overflow in state.rs. Debug recursive state lookup.
- [ ] **26. colortbls_test** (53_alignment) — TooManyErrors. Port colortbl.sty binding.
- [ ] **27. supertabular_test** (53_alignment) — "alignment not active". Port supertabular.sty binding.
- [ ] **28. badeqnarray_test** (53_alignment) — 306 diffs. Needs afterConstruct `rearrangeEqnarray`.

### Tier 5: AMS math + afterConstruct DOM rearrangement (unlocks 5–7 tests)

- [ ] **29. Implement afterConstruct rearrangement** — `rearrangeEqnarray`, `rearrangeAMSAlign`, `rearrangeAMSGather`, `openMathFork`/`closeMathFork`/`addColumnToMathFork`/`equationgroupJoinCols`. ~200 lines Perl. Unlocks: amsdisplay, matrix, sideset, badeqnarray, split, eqnarray.
- [ ] **30. amsdisplay_test** (56_ams) — needs afterConstruct + `\text{}` + math operators.
- [ ] **31. matrix_test** (56_ams) — needs afterConstruct + math parser for XMDual wrapping.
- [ ] **32. sideset_test** (56_ams) — needs afterConstruct.
- [ ] **33. cd_test** (56_ams) — math parser panic in parse_rec. Port amscd.sty.ltxml.
- [ ] **34. mathtools_test** (56_ams) — TooManyErrors. Port mathtools.sty binding.

### Tier 6: Graphics + color (unlocks 4 tests)

- [ ] **35. graphrot_test** (65_graphics) — TooManyErrors. `\begingroup` in `\csname..\endcsname`.
- [ ] **36. picture_test** (65_graphics) — port picture environment (`\put`, `\line`, `\circle`, `\oval`). Port graphpap.sty.
- [ ] **37. xcolors_test** (65_graphics) — ~600 diffs. Complete xcolor port (color expressions, testbox).
- [ ] **38. xytest** (65_graphics) — port xy.sty binding.

### Tier 7: Complex integration tests (unlocks 8 tests)

- [ ] **39. cleveref_minimal_test** (80_complex) — 125 diffs. Port cleveref.sty binding.
- [ ] **40. figure_mixed_content_test** (80_complex) — 875 diffs. Needs listings + figure + wrapfig.
- [ ] **41. aastex_test** (80_complex) — port aastex631.cls binding.
- [ ] **42. acm_aria_test** (80_complex) — port acmart.cls binding.
- [ ] **43. aastex631_deluxetable_test** (80_complex) — port aastex.cls binding.
- [ ] **44. aliceblog_test** (80_complex) — port blog.cls binding.
- [ ] **45. physics_test** (80_complex) — port physics.sty binding.
- [ ] **46. si_test** (80_complex) — port siunitx.sty binding.

### Tier 8: Math parser tests (unlocks 1–28 tests, active research)

- [ ] **47. 40_math suite** (40_math) — 14 tests. Enable and categorize: parser bugs vs intentional Marpa divergence.
- [ ] **48. 70_parse suite** (70_parse) — 28 tests. Generate Rust-specific expected XMLs where Marpa diverges.
- [ ] **49. plainmath_test** (53_alignment) — 382 diffs. Math parser XMDual structure.
- [ ] **50. split_test** (53_alignment) — TooManyErrors. Amsmath split env + math parser.
- [ ] **51. eqnarray_test** (53_alignment) — 2698 diffs. afterConstruct + math parser.
- [ ] **52. ntheorem_test** (55_theorem) — 897 diffs. Math parser tree structure (873 diffs) + eqnarray (23 diffs).

### Tier 9: Infinite loops and timeouts (need deep debugging)

- [ ] **53. diagboxtest_test** (53_alignment) — infinite loop in diagbox processing.
- [ ] **54. ncases_test** (53_alignment) — infinite loop in ncases processing.
- [ ] **55. vmode_test** (53_alignment) — infinite loop in vertical mode.
- [ ] **56. babel_test** (81_babel) — unbounded memory leak. Port babel.sty + `.ldf` files.

### Tier 10: New test suites from Perl (not yet copied)

- [ ] **57. Copy + port pgf suite** (2 tests) — port pgf.sty binding.
- [ ] **58. Copy + port tikz suite** (10 tests) — port tikz.sty binding (depends on pgf).
- [ ] **59. Copy remaining complex tests** (15 tests) — copy .tex/.xml from Perl, port bindings.
- [ ] **60. Port moderncv suite** (1 test) — port moderncv.cls binding.
- [ ] **61. Copy keyvalemptyvalue** (1 test) — copy from Perl.

### Permanent ignores (not counted)

- **ns1–ns5** (52_namespace) — DTD not supported in Rust port. Permanently ignored.

### Infrastructure prerequisites (not tied to specific tests)

- [ ] **A. Port expl3 programming layer** — `\ExplSyntaxOn/Off`, `\cs_new:Npn`, `\tl_set:Nn`. Unlocks: soul, fontspec, unicode-math, beamer, and many modern packages.
- [ ] **B. Complete Document.pm audit** — afterConstruct hooks, insertElementBefore, compact_xmdual.
- [ ] **C. Port BibTeX.pool.ltxml** — bibliography infrastructure (~150 defs, ~9% ported).
- [ ] **D. Port AmSTeX.pool.ltxml** — legacy AMS macros (~112 defs, ~30% ported).

---

> **Reminder:** Every entry ported from Perl must follow tightly the original semantics and nuances. Read the Perl source, translate precisely, preserve edge cases. The Perl code is the ground truth.
