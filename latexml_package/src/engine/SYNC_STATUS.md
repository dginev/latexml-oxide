# Engine Sync Status: Perl vs Rust

Generated 2026-03-11. Covers all engine files.

## Legend
- **OK** = fully synced, no gaps
- **MINOR** = small differences (stubs, style)
- **GAPS** = significant missing definitions
- **EMPTY** = placeholder file, not ported

---

## Phase 1: Foundation Layer (Base_* files)

### base_schema.rs (vs Base_Schema.pool.ltxml) — MINOR
- **Missing:** `RegisterNamespace(aria => "http://www.w3.org/ns/wai-aria")`
- Everything else matches.

### base_parameter_types.rs (vs Base_ParameterTypes.pool.ltxml) — GAPS
- **Missing (unported):** `DirectoryList`, `CommaList`, `DigestUntil` — all have `unported!()`/`todo!()` stubs
- **Commented out:** `TextStyle`, `ScriptStyle`, `ScriptscriptStyle`
- **Incomplete:** `Variable` reversion has `todo!()`
- **Architectural:** Rust uses `predigest` callback pattern instead of Perl's flags (`undigested => 1`, `semiverbatim => 1`)

### base_utilities.rs (vs Base_Utility.pool.ltxml) — GAPS
- **Missing primitives:** `\lx@endash`, `\lx@emdash`, `\lx@NBSP`, `\lx@nobreakspace`, `\lx@frontmatter@fallback`
- **Missing helpers:** `isDefinable()`, `aligningEnvironment()`, `addClass()`, `setAlignOrClass()`, `SplitTokens()`, `JoinTokens()`, `insertFrontMatter()` (stub), `removeEmptyElement()`, `AddToPreamble()`
- **Incomplete:** `\@add@to@frontmatter` / `\@add@to@frontmatter@now` — `unported!()` stubs

### base_xmath.rs (vs Base_XMath.pool.ltxml) — GAPS
- **Commented out (~29 defs):**
  - `\lx@apply`, `\lx@symbol`, `\lx@wrap` constructors
  - `\lx@superscript`, `\lx@subscript` constructors
  - `\lx@padded`, `\lx@math@tweaked`, `\lx@text@tweaked`
  - Matrix system: `\lx@gen@matrix@bindings`, `\lx@gen@plain@matrix@`, `\lx@ams@matrix@`
  - Cases system: `\lx@cases@condition`, `\lx@cases@end@condition`, `\lx@gen@cases@bindings`, `\lx@gen@plain@cases@`
- **Missing helpers:** `openMathFork()`, `closeMathFork()`, `MathWhatsit()`, `addColumnToMathFork()`, `equationgroupJoinRows()`, `equationgroupJoinCols()`
- **Infrastructure blocker:** `enterHorizontal` not supported

---

## Phase 1: High-Gap TeX Primitives

### tex_math.rs + tex_scripts.rs (vs TeX_Math.pool.ltxml) — GAPS
- **Fixed:** `\lx@generalized@over Undigested RequiredKeyVals` constructor with full afterDigest (regurgitate numerator, digestNextBody denominator, keyvals extraction)
- **Fixed:** All 6 fraction macros: `\above`, `\abovewithdelims`, `\atop`, `\atopwithdelims`, `\over`, `\overwithdelims`
- **Fixed:** `\lx@left`/`\lx@right` — aliased to `\@left`/`\@right` (Perl uses TeXDelimiter, Rust uses Token)
- **TODO:** `adjustMathstyle` — recursive mathstyle adjustment on already-digested boxes (cosmetic)
- **TODO:** `fracSizer` — sizer callback for fractions
- **Missing (~14 defs):**
  - `\nonscript`, `\lx@dollar@default`
  - `TeXDelimiter` parameter type (Rust uses simpler `Token` approach)
  - `\lx@delimiterdot`
  - `adjustMathRole()` subroutine — Rust `\mathord`/`\mathop`/etc use simple XMWrap instead
  - `\lx@math@overline`, `\lx@text@overline`, `\lx@math@underrightarrow`, `\lx@math@underleftarrow`
  - `\lx@math@overbrace`, `\lx@math@underbrace`
  - `\eqno`, `\leqno` — **Fixed:** renamed `\@@eqno` to `\lx@eqno` to match Perl
  - Math ligatures for `\cdots`/`\ldots`
- **Unported:** `\mathchoice` returns `Err(unported!())`

### tex_box.rs (vs TeX_Box.pool.ltxml) — GAPS
- **Fixed:** Added `\lx@overlay{}{}` constructor for accent overlay fallback
- **Fixed:** `\unhbox`, `\unhcopy`, `\unvbox`, `\unvcopy` — implemented with mode-aware unlisting
- **Missing:** `\leaders`/`\cleaders`/`\xleaders` full implementation (stub `None`)
- **Missing helpers:** SVG functions (`collapseSVGGroup`, `addSVGDebuggingBox`, `isVAttached`, `insertBlock`, `hackVBoxAttachment`, `adjustBoxColor`, `_color_adjust`)
- **Simplified:** `\hbox`, `\vbox`, `\vtop` constructors have many TODOs
- **Simplified:** `\vrule`/`\hrule` constructors mostly commented out

### tex_file_io.rs (vs TeX_FileIO.pool.ltxml) — MINOR
- **Commented out:** `DefKeyVal`s for SpecialPS, `\lx@special@graphics` constructor, `Tag('ltx:graphics')`
- **Differs:** `\write` and `\special` use `{}` instead of `XGeneralText`; `\openin` has `forbid_ltxml=true` (Perl doesn't)

### tex_fonts.rs (vs TeX_Fonts.pool.ltxml) — GAPS
- **Missing:** `getFontDimen()` helper, all 7 ligature definitions (moved to `plain.rs`)
- **Broken:** `\fontname` always returns "fontname not implemented"
- **Broken:** `\fontdimen` getter only handles 3 hardcoded params (2,5,6), no setter
- **Simplified:** `FontDef` parameter type → `FontToken` (reduced functionality)
- **Known:** OML font map pos 127 uses U+0311 instead of U+0361 (documented limitation)

---

## Phase 2: Remaining TeX Primitives

### tex_character.rs (vs TeX_Character.pool.ltxml) — MINOR
- **Commented out:** `\accent Number` primitive (core TeX primitive, complex)
- **Differs:** `\char` doesn't preserve adjusted font; `apply_accent` uses hardcoded Unicode ranges vs dynamic lookup
- Note: `\k` accent (ogonek) added to `latex_ch15_special_symbol.rs`

### tex_paragraph.rs (vs TeX_Paragraph.pool.ltxml) — MINOR
- **Fixed:** `\indent`/`\noindent` now call `enter_horizontal()` in before_digest (matches Perl `enterHorizontal => 1`)
- **Fixed:** `\lx@normal@par` `beforeDigest` checks MODE/BOUND_MODE and calls `assign_value_inplace` to resume vertical mode (matches Perl's repackHorizontal logic)
- **Missing properties:** `\indent`/`\noindent` still lack `isSpace`
- **Missing helpers:** `alignLine()`, `trimNodeLeftWhitespace()`, `trimNodeRightWhitespace()`
- **Rust-only:** `ltx:break` insertion in figures (not in Perl)

### tex_macro.rs (vs TeX_Macro.pool.ltxml) — MINOR
- **Fixed:** `\the \font` now returns current FontDef token (was `todo!()`)
- **Remaining:** FontDef case still commented out

### tex_logic.rs (vs TeX_Logic.pool.ltxml) — OK
- **Fixed:** `\ifvmode`, `\ifhmode`, `\ifinner` now check actual MODE state (was hardcoded `false`)
- `\ifmmode` works (checks `IN_MATH`)

### pdftex.rs (vs pdfTeX.pool.ltxml) — GAPS
- **Missing registers (6):** `\knaccode`, `\knbccode`, `\knbscode`, `\shbscode`, `\stbscode`, `\tagcode`
- **Missing:** `OpenAnnotSpecification` parameter type, `\pdfannot`, `\pdfobj` primitives
- **Incomplete:** `\pdfcolorstack` (commented out), `\pdffilesize` (has `todo!()`)
- **Wrong default:** `\pdftexversion` should be `140`, not `0`

### etex.rs (vs eTeX.pool.ltxml) — MINOR
- **Type mismatch:** `\parshapelength` returns `Dimension` in Rust, `Number` in Perl
- **Incomplete:** `etex_readexpr_i` has `todo!()` for missing close paren error
- **Bounds:** `\fontchar*` enforces 0-127 in Rust; Perl allows any value

### tex_debugging.rs — OK
### tex_page.rs — OK
### tex_penalties.rs — OK
### tex_marks.rs — OK
### tex_inserts.rs — OK
### tex_hyphenation.rs — MINOR (FontDef → FontToken simplification)
### tex_kern.rs — MINOR (SVG handling removed)
### tex_registers.rs — MINOR
- **Fixed:** Added `\lx@alloc@` register allocation macro and `\lx@counter@arabic` counter display
- **Fixed:** Added `SkipSpaces` to all `\*def` parameter signatures to match Perl
- **Remaining:** Missing `DumpFile()` infrastructure

### tex_job.rs — OK
- **Fixed:** Added `today()` helper and `MONTH_NAMES` array
- **Fixed:** Added `SOURCE_DATE_EPOCH` env var support for reproducible builds
- **Remaining:** `DumpFile()` infrastructure not ported (low priority)

### tex_glue.rs — MINOR
- **Fixed:** `\vskip` now matches Perl thresholds (`<= 0` / `< 4.0` / else) with `leave_horizontal()?` in before_digest
- **Fixed:** `\unskip` comment restoration bug (comments collected but never pushed back)
- **TODO:** `\hskip` `enter_horizontal` commented out (causes test failures)
- **TODO:** `\hfil`, `\hfill`, `\hss`, `\hfilneg` `enter_horizontal` commented out
- **TODO:** `\vfil`, `\vfill`, `\vss`, `\vfilneg` `leave_horizontal` commented out
- **Missing:** `\hskip` SVG handling (low priority, not needed for XML output)

### tex_tables.rs — GAPS
- **Missing:** `\halign BoxSpecification` constructor entirely commented out
- **Missing:** Many alignment helpers (`parseHAlignTemplate`, `beforeCellUnlist`, `afterCellUnlist`, etc.)
- **Renamed:** Some `\lx@alignment@*` macros renamed in Rust

---

## Phase 3: Plain Format

### plain.rs (vs plain_bootstrap + plain_base + plain_constructs + math_common) — GAPS

**From plain_bootstrap:**
- **Fixed:** `\ch@ck{}{}{}` macro (no-op, matches Perl)
- Missing: `\alloc@{}{}{}{}{}` macro
- `\leavevmode` is no-op (Perl calls `enterHorizontal`)
- `\TeX` constructor missing `enterHorizontal` property

**From plain_base:**
- **Fixed:** mathcode assignments block (20+ assignments for punctuation/operators)
- Missing: `\@@oalign{}`, `\@@ooalign{}` constructors (commented out)
- Missing: `\multispan{Number}` (commented out)
- Missing: `\hglue Glue` (commented out)
- Missing: `\displaylines{}` (commented out)
- `\joinrel` / `\@@joinrel{}{}` return errors (unported)
- Typo: `\downbracefil` should be `\downbracefill`
- Duplicate `\tenrm` font declaration

**From plain_constructs:**
- Missing: `\@math@daccent {}`, `\@math@baccent {}` constructors (commented out)
- Missing: `\lx@hack@bordermatrix{}` constructor (commented out)
- **Fixed:** `\@ringaccent`/`\r` standalone char now `U+02DA` (was `"o"`); `\t` now `NBSP+U+0361` (was `"-"`)

**From math_common:**
- **Fixed:** `scriptpos` and `mathstyle` closures now use `need_scriptpos`/`need_mathstyle` flags resolved at invocation time via `MathCharProps` in `base_functions.rs`
- **Fixed:** All `\big`/`\Big`/`\bigg`/`\Bigg` with l/m/r suffixes — 12 sized delimiter constructors with `augment_delimiter_role` after_construct
- Missing: `\smallint` `font => { size => 9 }` property
- Missing: `\phantom`/`\hphantom`/`\vphantom` afterDigest size computation
- Role difference: `\neg`/`\lnot` use `"FUNCTION"` in Rust vs `"BIGOP"` in Perl — CHECK if intentional
- XMWrap cleanup rewrite rule not in plain.rs (may be elsewhere)

---

## Phase 4: LaTeX Chapters (C.1–C.6)

### latex_ch1_documentclass.rs — MINOR
- Missing: `\documentstyle` compatibility, `onlyPreamble` enforcement

### latex_ch1_environments.rs — GAPS
- Missing: `beforebegin`/`afterend` environment hooks, `\@checkend`

### latex_ch1_fragile_commands.rs — OK (intentionally minimal)

### latex_ch1_break_command.rs — MINOR
- Differs: `\lx@newline` variant missing, different context checking

### latex_ch2_document.rs — MINOR
- **Fixed:** `\begin{document}` sets `BOUND_MODE=internal_vertical` and `set_mode("internal_vertical")`, with `leave_horizontal_internal()` in afterDigest
- **Fixed:** `\end{document}` calls `leave_horizontal_internal()` in beforeDigest (matches Perl)
- Missing: Full unclosed group/environment/conditional warnings at document end (commented out)

### latex_ch3_sentences_and_paragraphs.rs — MINOR
- Missing: `\@@par`, `\@par`, `\@restorepar` macros
- Missing: `enterHorizontal` property on `\emph`

### latex_ch4_sectioning_and_toc.rs — GAPS
- Missing: `\format@title@*`, `\format@toctitle@*`, `\@@compose@title{}{}`, `\@tag[][]{}` (all TODO)

### latex_ch5_packages.rs — GAPS
- **Fixed:** `\@ifl@aded` now checks `.ltxml_loaded` suffix (was `_binding_loaded`)
- **Missing:** `\PassOptionsToPackage`, `\PassOptionsToClass`, `\ExecuteOptions` (full), `\@onefilewithoptions`, `\OptionNotUsed`, `\@unknownoptionerror`, `\@if@ptions`

### latex_ch5_page_styles.rs — MINOR
- **Fixed:** Added `\ps@empty`, `\ps@plain`, `\@mkboth`, `\@leftmark`, `\@rightmark`
- **Fixed:** Dimension defaults now match Perl: `\paperheight`→11in, `\paperwidth`→8.5in, `\textheight`→550pt, `\textwidth`→345pt, `\columnwidth`→345pt, `\linewidth`→345pt

### latex_ch5_title_page_and_abstract.rs — MINOR

### latex_ch6_displayed_paragraphs.rs — MINOR
- Missing: `\raggedright`/`\raggedleft` implementations (commented out)

### latex_ch6_quotations_and_verse.rs — OK
### latex_ch6_list_making_environments.rs — MINOR
### latex_ch6_list_and_trivlist_environments.rs — OK
### latex_ch6_verbatim.rs — MINOR (different architecture but functional)

---

## Phase 4: LaTeX Chapters (C.7–C.15)

### latex_ch7_math_mode_environments.rs — GAPS (60% coverage)
- Missing: Full equation numbering (`prepareEquationCounter`, `beforeEquation`, `afterEquation`)
- Missing: `\nonumber`, `\tag`, `\lefteqn{}`, `\intertext{}`
- Missing: Full eqnarray machinery, sub-numbered equations

### latex_ch7_math_common_structures.rs — GAPS (50%)
- Missing: `\frac` sizer callback, mathstyle property calculation

### latex_ch7_math_common_delimiters.rs — EMPTY (0%)
### latex_ch7_math_mode_changing_style.rs — OK (95%)

### latex_ch8_defining_commands.rs — GAPS (50%)
- **Fixed:** Added `\CheckCommand` and `\DeclareEncodingSubset` stubs
- **Present:** `\DeclareTextCommand`, `\ProvideTextCommand`, `\DeclareTextSymbol`, `\DeclareSymbolFont`, `\DeclareSymbolFontAlphabet`, `\DeclareFontEncoding`, font encoding macros
- **Missing:** `\DeclareMathAccent`, `\DeclareFontShape`, `\DeclareFontFamily`, many more font declaration primitives

### latex_ch8_defining_environments.rs — OK (95%)
### latex_ch8_theoremlike_environments.rs — MINOR (75%)
### latex_ch8_numbering.rs — OK (95%)

### latex_ch9_figures_and_tables.rs — MINOR (85%)
- Missing: `\listoffigures`, `\listoftables`, double-column float variants

### latex_ch9_marginal_notes.rs — GAPS (50%)
### latex_ch10_tabbing_environment.rs — EMPTY (0%)
### latex_ch10_array_and_tabular.rs — OK (90%)

### latex_ch11_moving_information.rs — MINOR (85%)
### latex_ch11_splitting_the_input.rs — OK
### latex_ch11_index_and_glossary.rs — MINOR
### latex_ch11_terminal_io.rs — OK

### latex_ch12_line_and_page_breaking.rs — OK (90%)
### latex_ch13_boxes.rs — MINOR (85%)
### latex_ch14_pictures_and_color.rs — GAPS (30%) — picture environment not implemented
### latex_ch15_font_selection.rs — OK (90%)
### latex_ch15_special_symbol.rs — OK (90%)
### latex_other_in_appendices.rs — MINOR (85%)
### latex_semi_undocumented.rs — MINOR (85%)

---

## Unported Perl Files (no Rust counterpart)

| File | Defs | Priority | Notes |
|------|------|----------|-------|
| `latex_bootstrap.pool.ltxml` | 9 | Medium | LoadFormat machinery needed |
| `latex_base.pool.ltxml` | 168 | High | Largest unported file |
| `latex_constructs.pool.ltxml` | ~84 | Medium | Some content in chapter files |
| `Base_Deprecated.pool.ltxml` | ~20 | Low | |
| `AmSTeX.pool.ltxml` | ~50 | Low | |
| `BibTeX.pool.ltxml` | ~30 | Low | |

---

## Core Engine Sync (non-engine files)

### mouth.rs (vs Mouth.pm + Mouth/file.pm) — MINOR
- **Fixed:** `at_letter` option now separate from `fordefinitions` (Perl keeps them independent)
- **Fixed:** `note_message` includes "w/@ other" suffix per Perl
- **Fixed:** File validation: readable check (PermissionDenied) and binary file detection
- **Fixed:** Input encoding: latin-1 support, `\u{FFFD}`→space replacement (Perl's Encode::FB_DEFAULT behavior)
- **Fixed:** `load_tex_definitions` now passes `at_letter: true` (matches Perl `$loadtexdefinitions_options`)
- **Fixed:** `raw_tex()` uses `at_letter: true` instead of manual catcode save/restore
- **Remaining:** Full encoding support (only latin-1 handled; other encodings fall back to UTF-8 lossy)
- **Remaining:** `FoodType::Binding` variant commented out
- **Remaining:** Token caching optimization not implemented (performance only, low priority)

### gullet.rs (vs Gullet.pm) — MINOR
- **Fixed:** `\special_relax` smuggling — unexpanded token stored in thread-local Cell (Perl: slot [2])
- **Fixed:** `read_match()` checks smuggled `\special_relax` token (Perl line 612)
- **Fixed:** `read_until()` checks smuggled `\special_relax` in single-token case (Perl line 662)
- **Fixed:** `peek_token()` added — reads+unreads with ALIGN_STATE suppression (Perl line 331-337)
- **Fixed:** `show_unexpected()` added — debug message for error reporting (Perl line 185-193)
- **Fixed:** `read_value(Token)` handles `\csname...\endcsname` (Perl line 770-775)
- **Fixed:** `unread_vec()` adjusts ALIGN_STATE by counting BEGIN/END tokens (Perl line 352-358)
- **Remaining:** `read_arg()` uses direct unread instead of `reading_from_mouth` for expanded single tokens (subtle semantic difference, low impact)
- **Remaining:** `read_register_value()` missing coercion types (Number→Dimension, Dimension→Glue, etc.)
- **Remaining:** `show_pushback()` debug utility not implemented (low priority)

### stomach.rs (vs Stomach.pm) — MINOR
- **Fixed:** `enter_horizontal`, `leave_horizontal`, `leave_horizontal_internal` implemented
- **Fixed:** `bindable_mode()` mapping, `BOUND_MODE` tracking
- **Fixed:** MODE initialized to "vertical" (was "text")
- **Fixed:** `repack_horizontal()` implemented (Perl lines 440-454) — pops horizontal items from box_list, packs into List
- **Fixed:** `egroup()`/`endgroup()` now check `BOUND_MODE` before allowing close (Perl lines 334, 354)
- **Fixed:** `current_frame_message()` now includes `groupInitiatorLocator` (Perl line 314)
- **Fixed:** `invoke_token_simple()` skips `enter_horizontal()` in math mode (Perl lines 248-255)
- **Fixed:** `decode_math_char` hook infrastructure — callback registered from `latexml_package` at init
- **Deferred:** `\everymath`/`\everydisplay` injection in `begin_mode()` — already handled per-constructor in tex_math.rs, consolidation deferred to avoid double injection
- **Deferred:** Full mathcode-based char decoding in `invoke_token_simple()` — `Tbox::new` already handles math props via `math_token_attributes`; hook available but disabled pending role reconciliation (ADDOP vs BINOP)
- **Remaining:** Test regressions from MODE changes need one-by-one investigation

### state.rs (vs State.pm) — MINOR
- **Fixed:** `assign_value_inplace` implemented (Perl's 'inplace' scope)
- Remaining gaps tracked separately

### register.rs (vs Definition/Register.pm) — MINOR
- **Fixed:** `new_math_chardef` with `chardef_props` HashMap
- Remaining gaps tracked separately

---

## Cross-Cutting Infrastructure Gaps

1. **`enterHorizontal`/`leaveHorizontal`** — Core infrastructure now implemented in `stomach.rs`:
   - `enter_horizontal()`: switches MODE from vertical→horizontal via `assign_value_inplace`
   - `leave_horizontal()`: fires `\par` to return to vertical mode
   - `leave_horizontal_internal()`: resets mode without `\par` (used by endMode for vertical modes)
   - `BOUND_MODE` tracking: `set_mode`/`begin_mode`/`end_mode` all set BOUND_MODE
   - `assign_value_inplace()` in state.rs: modifies value without undo recording
   - `bindable_mode()` mapping: text→restricted_horizontal, vertical→internal_vertical, etc.
   - `invoke_token_simple()`: spaces suppressed in math/vertical modes, `enter_horizontal` called for non-math chars
   - **STATUS:** Core working. Most engine-level calls still commented out (cause test regressions, need one-by-one investigation). See checklist below.
2. **`LoadFormat` machinery** — Not ported. `plain_bootstrap/base/constructs` and `latex_bootstrap/base/constructs` loaded inline instead.
3. **TeX mode tracking** — ~~`\ifvmode`/`\ifhmode`/`\ifinner` hardcoded false~~ **FIXED**: Now check actual MODE state.
4. **`FontDef` parameter type** — Simplified to `FontToken`. Blocks proper `\fontdimen`, `\fontname`, `\hyphenchar` font-aware getters.
5. **SVG support** — Removed from `\hskip`, `\kern`, `\raise`, `\lower`, `\hbox`. Not critical for XML output.
6. **Fraction/over system** — ~~`\lx@generalized@over` commented out~~ **FIXED**: Core fraction system implemented. `adjustMathstyle` still TODO (cosmetic).
7. **`new_math_chardef`** — **FIXED**: Register struct now has `chardef_props` field for extra math properties (meaning, stretchy, scriptpos, mathstyle, etc.). `\mathchardef` uses it.

---

## enterHorizontal / leaveHorizontal Checklist

In Perl's `Package.pm`, `enterHorizontal => 1` and `leaveHorizontal => 1` properties
on `DefConstructor`/`DefPrimitive` are converted to `beforeDigest` callbacks that call
`$stomach->enterHorizontal` / `$stomach->leaveHorizontal`. Some definitions call these
directly in their body or afterDigest instead.

Rust equivalents: `enter_horizontal()` / `leave_horizontal()?` / `leave_horizontal_internal()`.

### enterHorizontal — needs `enter_horizontal()` in before_digest or body

| Definition | File (Perl) | File (Rust) | Status |
|---|---|---|---|
| `\ ` (ctrl space) | TeX_Character L28 | tex_character.rs | **TODO** — body calls `enterHorizontal` |
| `\char Number` | TeX_Character L35 | tex_character.rs | **TODO** — body calls `enterHorizontal` |
| `\accent Number {}` | TeX_Character L135 | tex_character.rs | N/A — `\accent` not ported |
| `\indent` | TeX_Paragraph L60 | tex_paragraph.rs | **DONE** |
| `\noindent` | TeX_Paragraph L73 | tex_paragraph.rs | **DONE** |
| `\hskip Glue` | TeX_Glue L86 | tex_glue.rs | **TODO** |
| `\hss` | TeX_Glue L124 | tex_glue.rs | **TODO** |
| `\hfilneg` | TeX_Glue L125 | tex_glue.rs | **TODO** |
| `\hfil` | TeX_Glue L129 | tex_glue.rs | **TODO** |
| `\hfill` | TeX_Glue L134 | tex_glue.rs | **TODO** |
| `\kern` | TeX_Kern L98 | tex_kern.rs | **TODO** |
| `\raise` | TeX_Kern L109 | tex_kern.rs | **TODO** |
| `\lower` | TeX_Kern L98 | tex_kern.rs | **TODO** |
| `\moveleft` | TeX_Kern L124 | tex_kern.rs | **TODO** |
| `\moveright` | TeX_Kern L129 | tex_kern.rs | **TODO** |
| `\lx@framed` | TeX_Box L63 | tex_box.rs | **TODO** |
| `\lx@hflipped` | TeX_Box L67 | tex_box.rs | **TODO** |
| `\lx@overlay` | TeX_Box L73 | tex_box.rs | **TODO** |
| `\lx@text@nounicode` | TeX_Box L95 | tex_box.rs | **TODO** |
| `\box`/`\copy` | TeX_Box L663/673 | tex_box.rs | **TODO** — body calls `enterHorizontal` |
| `\vrule` | TeX_Box L756 | tex_box.rs | **TODO** — commented out |
| `\leavevmode` | plain_bootstrap L43 | plain.rs | **TODO** — body is `None` |
| `\TeX` | plain_bootstrap L27 | plain.rs | **TODO** |
| `\lx@begin@display@math` | TeX_Math L134 | tex_math.rs | **TODO** — beforeDigest |
| `\lx@begin@inline@math` | TeX_Math L150 | tex_math.rs | **TODO** — beforeDigest |
| `\lx@text@overline` | TeX_Math L956 | tex_math.rs | N/A — not ported yet |
| `\lx@text@underline` | TeX_Math L962 | tex_math.rs | N/A — not ported yet |
| `\lx@kludged` | Base_XMath L195 | base_xmath.rs | **TODO** — beforeDigest |
| `\vdots` | math_common L472 | plain.rs | N/A — not ported yet |
| `\dots` | math_common L488 | plain.rs | N/A — not ported yet |
| `\@makebox` | latex_constructs L4719 | latex_ch13_boxes.rs | **TODO** |
| `\raisebox` | latex_constructs L4853 | latex_ch13_boxes.rs | **TODO** |
| `\emph` | latex_constructs L407 | latex_ch3 | **TODO** |
| `\LaTeX` | latex_bootstrap L32 | plain.rs | **TODO** |
| `\LaTeXe` | latex_bootstrap L44 | N/A | N/A — not ported |

### leaveHorizontal — needs `leave_horizontal()?` in before_digest or body

| Definition | File (Perl) | File (Rust) | Status |
|---|---|---|---|
| `\vskip Glue` | TeX_Glue L100 | tex_glue.rs | **DONE** |
| `\vfil` | TeX_Glue L147 | tex_glue.rs | **TODO** |
| `\vfill` | TeX_Glue L148 | tex_glue.rs | **TODO** |
| `\vss` | TeX_Glue L149 | tex_glue.rs | **TODO** |
| `\vfilneg` | TeX_Glue L150 | tex_glue.rs | **TODO** |
| `\hrule` | TeX_Box L791 | tex_box.rs | **TODO** — commented out |
| `\unvbox`/`\unvcopy` | TeX_Box L683/693 | tex_box.rs | **TODO** — body calls `leaveHorizontal` |
| `\halign` | TeX_Tables L168 | tex_tables.rs | N/A — not ported |
| `\lx@end@document` | TeX_Job L67 | tex_job.rs | **TODO** — body calls `leaveHorizontal` |

### leaveHorizontal_internal — needs `leave_horizontal_internal()`

| Definition | File (Perl) | File (Rust) | Status |
|---|---|---|---|
| `\begin@lx@document` afterDigest | latex_constructs L329 | latex_ch2_document.rs | **DONE** |
| `\@documentclasshook` | latex_constructs L84/128 | latex_ch2_document.rs | **DONE** |
| sectioning afterDigest | latex_constructs L616/653 | N/A | N/A — not in scope |
| `\usepackage`/`\RequirePackage` | latex_constructs L798/811 | N/A | N/A — not in scope |
