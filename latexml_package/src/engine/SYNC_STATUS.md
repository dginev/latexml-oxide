# Engine Sync Status: Perl vs Rust

Updated 2026-03-12. Only lists open gaps & TODOs; completed items live in git history.

## Legend
- **OK** = fully synced | **MINOR** = small gaps | **GAPS** = significant missing | **EMPTY** = not ported

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
| tex_math.rs | GAPS | Missing: `\nonscript`, `\lx@dollar@default`, `TeXDelimiter` param type, `adjustMathRole()`, math ligatures. `\mathchoice` ported. Done: `\lx@math@overline/underline/overbrace/underbrace`, `\lx@text@overline/underline`, `\lx@math@over/underleft/rightarrow`, `operator_stretchy` on all. |
| tex_box.rs | GAPS | Missing: `\leaders/cleaders/xleaders`, SVG functions (`collapseSVGGroup` etc), `\hbox/vbox/vtop` have many TODOs, `\vrule/\hrule` mostly commented out |
| tex_file_io.rs | MINOR | `\lx@special@graphics` constructor + `Tag('ltx:graphics')` commented out |
| tex_fonts.rs | GAPS | Missing: `getFontDimen()`, 7 ligature defs. `\fontname` always returns placeholder. `\fontdimen` only handles 3 hardcoded params |
| tex_tables.rs | GAPS | `\halign BoxSpecification` entirely commented out, many alignment helpers missing |

### Phase 2: Remaining TeX Primitives

| File | Status | Open Gaps |
|------|--------|-----------|
| tex_character.rs | MINOR | `\accent Number {}` not ported |
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
- Missing: `\alloc@{}{}{}{}{}`, `\@@oalign/@@ooalign`, `\multispan`, `\hglue`, `\displaylines`, `\@math@daccent/baccent`, `\lx@hack@bordermatrix`
- `\leavevmode` calls `enter_horizontal()` (synced)
- `\joinrel`/`\@@joinrel` return errors
- `\partial` role fixed to DIFFOP (was OPERATOR), `\smallint` has mathstyle TODO
- Missing: `\smallint` font size, `\phantom/hphantom/vphantom` afterDigest sizing
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
| latex_ch7_math_mode_environments.rs | GAPS | Done: `retract_equation()`, `\nonumber`, `\lx@equation@nonumber`, `\lx@equation@retract`, `\lx@equation@settag`/`@`, `{equation*}`, `after_equation` postset branch, simplified `{eqnarray}`/`{eqnarray*}` (single-equation per group, no alignment). Missing: `\lefteqn`, `\intertext`, full alignment-based eqnarray (MathFork/MathBranch/rearrangeEqnarray), `{align}`/`{gather}`/`{multline}` |
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

## Cross-Cutting Infrastructure Gaps

1. **`FontDef` parameter type** — Simplified to `FontToken`. Blocks `\fontdimen`, `\fontname`, `\hyphenchar`.
2. **SVG support** — Removed from glue/kern/box. Not critical for XML output.
3. **`LoadFormat` machinery** — Not ported; plain/latex bootstrap loaded inline.
4. **Font `computeBoxesSize`** — Single-pass only; missing word/line/stack decomposition.
5. **`DEFSIZE`** — Static 10.0; Perl reads `NOMINAL_FONT_SIZE` from state.

---

## Unported Perl Files

| File | Defs | Priority | Notes |
|------|------|----------|-------|
| `latex_bootstrap.pool.ltxml` | 10 | — | **Complete** (audit 2026-03-12: 10/10 defs present) |
| `latex_base.pool.ltxml` | ~160 | — | **Complete** (audit 2026-03-12: ~100% across 36 ch files + appendices) |
| `latex_constructs.pool.ltxml` | ~843 | Low | ~90% ported. Missing: `\eqnarray` (no alignment), `\@xargdef/yargdef/reargdef`, picture env. |
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
| stomach.rs | MINOR | Mathcode char decoding (ADDOP vs BINOP). `execute_before_after_group` extracted. `begin_mode_opt`/`end_mode_opt` with `noframe` parameter synced with Perl Grouplevel commit (acaab773). `everymath/everydisplay` injection now centralized in `begin_mode_opt`. |
| state.rs | OK | — |
| document.rs | MINOR | `compact_xmdual()`, `mergeAttributes()`, `insertElementBefore()`, comment creation (needs libxml) |
| register.rs | MINOR | — |
| pathname.rs | MINOR | Missing: `pathname_make`, `pathname_relative`, `pathname_is_contained`, `pathname_findall`, `pathname_timestamp/copy/mkdir`. `canonical` now handles `./`, `/../`. Dir-listing approach in `candidate_pathnames` not ported (uses `Path::exists` instead). |
| alignment.rs | MINOR | Padding CSS classes, ABSORB_LIMIT guard, sizing info |
| rewrite.rs | GAPS | ~20% ported (Select/Replace only) |
| token.rs | OK | — |
| tokens.rs | OK | — |
| number/float/dimension.rs | OK | — |
| glue.rs | OK | NumericOps overrides: multiply/divide/subtract/smaller/larger preserve plus/minus/pfill/mfill |
| numeric_ops.rs | OK | Default multiply uses float arithmetic matching Perl |
| font.rs | MINOR | `computeBoxesSize` decomposition, `DEFSIZE` from state |

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
| amsmath.sty | GAPS | ~25% ported: spacing, \overunderset, \lvert/rvert/lVert/rVert, \notag, \tag, \tfrac/\dfrac, \xrightarrow/\xleftarrow, over/under arrows. Missing: equation environments ({align},{gather},{multline},{split}), \text, \intertext, operators. |
| appendix.sty | OK | Core environments: appendices, subappendices, conditional switches. |
| multicol.sty | OK | Full port: multicols/multicols* environments, registers, stubs. |
| booktabs.sty | OK | Full port: toprule/midrule/bottomrule/cmidrule/specialrule, registers. |
| caption.sty | MINOR | Stub-level: captionsetup, Declare* macros, registers. Missing: KeyVals, CAPTION_ value storage. |
| remreset.sty | OK | Empty stub (obsolete, macros moved to LaTeX core). |
| chngcntr.sty | OK | Empty stub (obsolete, macros moved to LaTeX core). |
| ntheorem.sty | GAPS | Framing constructors (`\lx@addframing`, `\lx@@snapshot@framing`) ported. Missing: `\colorbox` (needs xcolor port) for shaded theorems, `backgroundcolor` attribute. |

---

## Test Suite Status (2026-03-12)

**Current totals: 103 pass, 0 fail, 9 ignored test functions**

| Suite | Pass/Total | Notes |
|-------|-----------|-------|
| 000_hello | 1/1 | |
| 00_tokenize | 14/14 | All pass |
| 00_contrib | 1/1 | All pass |
| 01_unit_tokens | 1/1 | |
| 01_unit_state | 9/9 | |
| 10_expansion | 36/36 | All pass (whichcache/whichinput now pass) |
| 12_grouping | 2/2 | All pass |
| 20_digestion | 10/10 | All pass |
| 22_fonts | 0/0 | **Disabled** (commented out) — 22 .tex/.xml pairs ready |
| 30_encoding | 0/26 | **Ignored** — 26 pairs ready, needs encoding `.def` loading |
| 40_math | 0/14 | **Ignored** — 14 pairs ready, math parser divergence |
| 50_structure | 21/42 | 21 pass; 21 disabled (.todo) — regressed from pathname cwd change |
| 52_namespace | 0/5 | **Ignored** — needs `.latexml` document-level bindings |
| 53_alignment | 0/22 | **Ignored** — needs alignment system, longtable, listings |
| 55_theorem | 4/5 | 4 pass; ntheorem ignored (897 math parser diffs) |
| 56_ams | 0/7 | **Ignored** — needs amsmath environments, CD diagrams |
| 65_graphics | 0/8 | **Ignored** — needs color/graphicx/picture, keyval |
| 70_parse | 0/28 | **Ignored** — math parser regression tests |
| 700_unit_parse | 3/3 | |
| 80_complex | 1/1 | xii passes; 11 disabled (.todo), 4 Perl-only (no .tex/.xml yet) |
| 81_babel | 0/6 | **Ignored** — needs babel language `.ldf` files |

### Perl-only tests (not yet copied to Rust)

| Suite | Perl-only tests | Notes |
|-------|----------------|-------|
| fonts | plainfonts | 1 missing .tex/.xml |
| structure | 21 tests | .tex/.xml not copied (match .todo list) |
| graphics | xytest | 1 missing .tex/.xml |
| keyval | keyvalemptyvalue | 1 missing .tex/.xml |
| moderncv | orc | 1 missing .tex/.xml |
| complex | 15 tests | .tex/.xml not copied; 4 Perl-new (no .xml in Perl either) |

### Perl daemon frame pattern (not yet ported)
Perl uses `pushDaemonFrame`/`popDaemonFrame` (State.pm L607-660) to isolate state per conversion. This creates a locked frame, deep-copies mutable values, and allows rollback after conversion. Rust has the code commented out as TODO (state.rs L1784-1818). Currently Rust relies on `initialize_singletons` + `Core::new` resetting STATE/GULLET/STOMACH/MODEL/REPORT/LOCALIZED_VARS, which is sufficient for single-conversion use but lacks the deep-copy rollback semantics needed for daemon mode.

### ntheorem test gap analysis
- **Math parser tree structure** (873/896 diffs): XMApp/XMTok nesting differs due to Marpa-based parser architecture. Not fixable without parser changes (active research).
- **eqnarray** (23/896 diffs): Simplified `DefEnvironment` produces single equation per group instead of full alignment with MathFork/MathBranch/tr/td restructuring. Full impl needs: `eqnarray_bindings`, 3-column `$\displaystyle` template, `rearrangeEqnarray` post-processing (~200 lines in Perl).
- **Equation numbering**: Offset by ~1 due to simplified eqnarray not splitting rows.
- **Shaded theorems**: `backgroundcolor` attribute missing (needs `\colorbox` from xcolor.sty).

---

## Incremental Test Plan — Ranked Priority Order

Ranked by: (1) fewest new dependencies, (2) highest test yield per effort, (3) unlocks downstream work. First item is first to try.

### Rank 1 — Re-enable structure .todo tests (21 tests)
**Target:** 50_structure, the 21 `.todo` files
**Why first:** These tests *used to pass*. They regressed from the `candidate_pathnames` cwd-only-as-fallback change. Likely a single pathname/search-path fix re-enables all 21.
**Work:** Debug why `.cls`/`.sty` resolution fails when cwd isn't always in search paths. Copy 21 `.tex`/`.xml` from Perl, rename `.todo` back to `.tex`.
**Dependencies:** pathname.rs fix only — no new packages or engine features.
**Yield:** +21 tests (103 → 124)

### Rank 2 — Enable 22_fonts (up to 22 tests)
**Target:** 22_fonts.rs — currently commented out
**Why:** Test data already present (22 `.tex`/`.xml` pairs). Font system is partially implemented. Copy `plainfonts.tex/.xml` from Perl for the 23rd test.
**Work:** Uncomment `tex_tests!("tests/fonts")`. Run, triage failures. Likely issues: specialty font packages (soul, stmaryrd, wasysym, marvosym, esint, bbold) need `.ltxml` bindings or stubs.
**Dependencies:** Font subsystem (mostly done), possibly new `.sty.ltxml` stubs.
**Yield:** +10–22 tests (estimated; basic font tests likely pass, specialty packages may need stubs)

### Rank 3 — Enable 56_ams (up to 7 tests)
**Target:** 56_ams.rs — currently ignored
**Why:** amsmath is partially ported (~12%). Simpler AMS tests (dots, genfracs, sideset, matrix) may pass or need small fixes. CD diagrams need `amscd.sty` binding.
**Work:** Remove `#[ignore]`, run, triage. Implement missing amsmath environments incrementally. Port `amscd.sty.ltxml` for CD test.
**Dependencies:** amsmath.sty binding (partial), amscd.sty binding (new).
**Yield:** +3–7 tests

### Rank 4 — Enable 40_math (up to 14 tests)
**Target:** 40_math.rs — currently ignored
**Why:** Math pipeline is the core feature. 14 diverse tests covering arrays, fractions, scripts, delimiters, spacing. Some will pass with current engine; others expose parser divergence.
**Work:** Remove `#[ignore]`, run, triage. Fix non-parser issues (missing macros, attributes). Accept parser tree diffs where Rust intentionally diverges — update expected `.xml` for those.
**Dependencies:** Math parser (exists, divergent), amsmath (partial), `\DeclareMathOperator` machinery.
**Yield:** +5–14 tests (math parser divergence may block some)

### Rank 5 — Enable 30_encoding (up to 26 tests)
**Target:** 30_encoding.rs — currently ignored
**Why:** Encoding tables are systematic. Each test exercises one input encoding with `inputenc`. All 26 `.tex`/`.xml` pairs present.
**Work:** Remove `#[ignore]`, run. Fix encoding `.def` file loading — may need `inputenc.sty.ltxml` improvements or encoding table stubs.
**Dependencies:** inputenc system, encoding `.def` files. fontenc T1/TS1/OT1 support (partial).
**Yield:** +10–26 tests (encoding tests tend to pass/fail in batches)

### Rank 6 — Enable 55_theorem ntheorem (1 test)
**Target:** ntheorem test in 55_theorem.rs
**Why:** Single test, but 897 diffs — mostly math parser tree structure (873) + eqnarray (23) + shaded theorems. Low ROI unless math parser stabilizes.
**Work:** If parser converges, update expected XML. Implement full eqnarray with MathFork/MathBranch. Add `\colorbox` for shaded theorems.
**Dependencies:** Math parser convergence, eqnarray alignment, xcolor `\colorbox`.
**Yield:** +1 test (but validates theorem+math integration)

### Rank 7 — Enable 52_namespace (5 tests)
**Target:** 52_namespace.rs — currently ignored
**Why:** Only 5 tests, but requires `.latexml` document-level bindings infrastructure for custom DTDs/namespaces.
**Work:** Port the document-class `.latexml` binding system. Create Rust equivalents of ns1–ns5 `.latexml` files.
**Dependencies:** `.latexml` binding loader (new infrastructure).
**Yield:** +5 tests

### Rank 8 — Enable 65_graphics (up to 8 tests)
**Target:** 65_graphics.rs — currently ignored
**Why:** Needs color package, graphicx improvements, picture environment, keyval parsing. Copy `xytest.tex/.xml` from Perl.
**Work:** Port `color.sty.ltxml`, improve `graphicx.sty.ltxml`, implement keyval parameter type. Picture environment (EMPTY) is the hardest part.
**Dependencies:** color.sty, graphicx.sty, keyval.sty (partial), picture environment (new), dvipsnam.def.
**Yield:** +3–8 tests (picture and xy tests may remain blocked)

### Rank 9 — Enable 53_alignment (up to 22 tests)
**Target:** 53_alignment.rs — currently ignored
**Why:** Alignment is a major subsystem. `\halign` mostly commented out. Needs array, longtable, multirow, supertabular, tabbing, listings.
**Work:** Implement `\halign BoxSpecification`, port longtable/multirow/supertabular `.ltxml` bindings. Port tabbing environment.
**Dependencies:** `\halign` (GAPS), array.sty, longtable.sty, multirow.sty, supertabular.sty, listings.sty, colortbl.sty, rotating.sty.
**Yield:** +5–22 tests (incremental; simple tables first, then longtable/multirow)

### Rank 10 — Enable 81_babel (up to 6 tests)
**Target:** 81_babel.rs — currently ignored
**Why:** Requires full babel language loading infrastructure and per-language `.ldf` files.
**Work:** Port babel.sty.ltxml, implement language selection, port frenchb/germanb/greek `.ldf` bindings.
**Dependencies:** babel.sty (new), language `.ldf` files (new), numprint.sty, csquotes.sty.
**Yield:** +2–6 tests

### Rank 11 — Enable 70_parse (up to 28 tests)
**Target:** 70_parse.rs — currently ignored
**Why:** Pure math parser regression tests. 28 tests that validate parse tree structure. Parser is in active research with intentional Marpa divergence.
**Work:** Run all 28, categorize failures as (a) fixable Rust bugs vs (b) intentional divergence. Update expected XML for intentional divergence. Fix actual bugs.
**Dependencies:** Math parser stability. `\lxDeclare` role system.
**Yield:** +10–28 tests (depends on parser maturity)

### Rank 12 — Re-enable complex .todo tests (up to 11 tests)
**Target:** 80_complex .todo files
**Why:** Complex integration tests requiring many packages. Copy `.tex`/`.xml` from Perl for the 15 missing. Ranges from trivial (tcilatex_minimal, versioned_fallback, hypertest) to extreme (physics: 279KB, si: 472KB).
**Work:** Start with small tests (tcilatex_minimal, versioned_fallback, hypertest, labelled, aastex_test). Leave physics/si/figure_mixed_content for last.
**Dependencies:** Many packages (cleveref, siunitx, physics, wrapfig, algorithm, icml2016). Custom class bindings (aastex631, acm).
**Yield:** +3–11 tests (tiered approach)

### Rank 13 — Create new Rust test suites for Perl-only suites
**Target:** 32_keyval, 33_keyval_options, 82_moderncv, 83_expl3, 84_slides
**Why:** Test data mostly already copied. Need new `.rs` test files and package bindings.
**Work per suite:**
- **32_keyval** (7 tests ready + 1 to copy): keyval/xkeyval infrastructure, custom `.sty.ltxml` bindings for `mykeyval`/`myxkeyval`.
- **33_keyval_options** (11 tests): xkeyval document-class option handling, 6 custom `.sty.ltxml` stubs.
- **82_moderncv** (1 test ready + 1 to copy): moderncv class, complex layout package.
- **83_expl3** (2 tests): LaTeX3 expl3 programming layer — major new infrastructure.
- **84_slides** (2 tests): beamer class — complex presentation package.
**Dependencies:** Heavy. Each needs new package bindings. expl3 and beamer are architecturally significant.
**Yield:** +5–25 tests (but very high effort per test)

### Summary: Expected progression

| Milestone | Cumulative Tests | Key unlocks |
|-----------|-----------------|-------------|
| Start | 103 | — |
| After Rank 1 (structure .todo) | ~124 | pathname fix |
| After Rank 2 (fonts) | ~140 | font system validated |
| After Rank 3 (ams) | ~145 | amsmath basics |
| After Rank 4 (math) | ~155 | math pipeline validated |
| After Rank 5 (encoding) | ~175 | encoding tables |
| After Ranks 6–8 | ~190 | namespace, graphics, ntheorem |
| After Ranks 9–11 | ~230 | alignment, babel, parse |
| After Ranks 12–13 | ~260 | complex integration, new suites |
| Full Perl parity | ~354 | all suites |
