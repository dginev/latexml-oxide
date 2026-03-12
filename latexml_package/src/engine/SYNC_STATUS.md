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
- **Fixed:** Display math `$$` now injects `\everymath` and `\everydisplay` tokens in before_digest (was commented out TODO)
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
- **Fixed:** `\indent`/`\noindent` now have `isSpace` property (matches Perl)
- **Missing helpers:** `alignLine()`, `trimNodeLeftWhitespace()`, `trimNodeRightWhitespace()`
- **Rust-only:** `ltx:break` insertion in figures (not in Perl)

### tex_macro.rs (vs TeX_Macro.pool.ltxml) — MINOR
- **Fixed:** `\the \font` now returns current FontDef token (was `todo!()`)
- **Remaining:** FontDef case still commented out

### tex_logic.rs (vs TeX_Logic.pool.ltxml) — OK
- **Fixed:** `\ifvmode`, `\ifhmode`, `\ifinner` now check actual MODE state (was hardcoded `false`)
- `\ifmmode` works (checks `IN_MATH`)

### pdftex.rs (vs pdfTeX.pool.ltxml) — MINOR (96.7% complete)
- **116/120 definitions ported.** All 73 registers (integer, dimension, token, read-only) fully ported.
- **Missing (3):** `OpenAnnotSpecification` parameter type, `\pdfannot`, `\pdfobj` (PDF annotations, low priority)
- **Incomplete:** `\pdfcolorstack` (commented out TODO)

### etex.rs (vs eTeX.pool.ltxml) — MINOR
- **Fixed:** `\fontcharwd/ht/dp` removed 0-127 code restriction, added current font fallback (matches Perl `$font->merge` behavior)
- **Fixed:** `\currentgrouplevel` — `get_frame_depth()` now returns correct count (removed erroneous `saturating_sub(1)`)
- **Type mismatch:** `\parshapelength` returns `Dimension` in Rust, `Number` in Perl
- **Incomplete:** `etex_readexpr_i` has `todo!()` for missing close paren error

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
- **Fixed:** `\lx@end@document` now calls `leave_horizontal()` before `gullet::flush()` (matches Perl: `$stomach->leaveHorizontal; $stomach->getGullet->flush;`)
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

### latex_ch1_break_command.rs — OK
- **Fixed:** Implemented Perl's `\lx@newline` with `sub[document]` constructor: context-aware behavior (checks IN_MATH, _CaptureBlock_, ltx:p parent, canContain). `\\` is now Let'd to `\lx@newline` (was template-only `<ltx:break/>`)

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
- **Fixed:** `\@block@cr` now Let'd to `\lx@newline` (was separate constructor with `<ltx:break/>\n` — the `\n` caused spurious newlines in text nodes). Quote/quotation/verse environments no longer override `\\`/`\par` in before_digest (Perl doesn't either).
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
| `latex_constructs.pool.ltxml` | ~148 | Medium | 94.6% ported across latex_ch*.rs files; missing: `\hline`, `\@multicolumn`, `{figure*}`, `{table*}`, `\marginpar` |
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
- **Fixed:** `begin_semiverbatim()` now sets MODE to `restricted_horizontal` (was `text`), matching Perl
- **Fixed:** `begin_semiverbatim()` now calls `assign_mathcode('\'', 0x8000)` (was TODO)
- **Synced:** All value/catcode/mathcode/sfcode/lccode/uccode/delcode lookups and assignments
- **Synced:** All definition/meaning lookups (lookupDefinition, lookupExpandable, lookupConditional, isDontExpandable, lookupDigestableDefinition)
- **Synced:** Frame management (pushFrame, popFrame, getFrameDepth — fixed off-by-one, isValueBound)
- **Synced:** Scope management (activateScope, deactivateScope, getKnownScopes, getActiveScopes)
- **Synced:** Prefix management (setPrefix, getPrefix, clearPrefixes)
- **Synced:** Unit conversion (convertUnit)
- **Deferred:** `pushDaemonFrame`/`popDaemonFrame`/`daemon_copy` — commented out (daemon mode not yet needed)
- **Deferred:** `valueInFrame()` — frame-level positional lookup (debugging aid, rarely used)
- **Note:** Status tracking (`noteStatus`/`getStatus`/`getStatusMessage`/`getStatusCode`) lives in `error.rs` REPORT singleton rather than in State, already functional

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
| `\lx@end@document` | TeX_Job L67 | tex_job.rs | **DONE** — `leave_horizontal()` + `flush()` |

### leaveHorizontal_internal — needs `leave_horizontal_internal()`

| Definition | File (Perl) | File (Rust) | Status |
|---|---|---|---|
| `\begin@lx@document` afterDigest | latex_constructs L329 | latex_ch2_document.rs | **DONE** |
| `\@documentclasshook` | latex_constructs L84/128 | latex_ch2_document.rs | **DONE** |
| sectioning afterDigest | latex_constructs L616/653 | N/A | N/A — not in scope |
| `\usepackage`/`\RequirePackage` | latex_constructs L798/811 | N/A | N/A — not in scope |

---

## Core Modules Sync Status

### mouth.rs (vs Mouth.pm) — OK
- Synced 2026-03-11 (commit 727bd4f49, 94713d882)
- `at_letter` option, input encoding (latin-1 + FFFD→space), file validation, `note_message "w/@ other"`
- **Fixed:** `at_letter` restore now uses `unwrap_or(Catcode::OTHER)` — was silently leaving `@` as LETTER when saved catcode was None (not in table). Fixed meaning_test and inin_test.
- **EOL handling confirmed identical** (2026-03-11): state machine (N/M/S), blank line detection, trailing space removal, PRESERVE_NEWLINES, comment handling — all match Perl Mouth.pm. No catcode divergences for spaces/newlines.

### gullet.rs (vs Gullet.pm) — MINOR
- Synced 2026-03-11 (commit 3ca5b0198)
- `\special_relax` smuggling, `peekToken`, `readingFromMouth` skipSpaces, `skip_one_space_expanded`, backquote charcode fix
- **Deferred:** `readArg` isolation via `readingFromMouth(Tokens(...))`

### stomach.rs (vs Stomach.pm) — MINOR
- Synced 2026-03-11 (commit 9690d998e)
- `repackHorizontal`, BOUND_MODE checks, `current_frame_message` locator, math mode enter_horizontal skip
- `decode_math_char` hook infrastructure (cross-crate fn pointer pattern)
- **Deferred:** `everymath`/`everydisplay` injection consolidation (currently per-constructor in tex_math.rs)

### state.rs (vs State.pm) — OK
- Synced 2026-03-11 (commit 18eef452c)
- `beginSemiverbatim` MODE fix (text → restricted_horizontal), mathcode `'\''` = 0x8000

### document.rs (vs Document.pm) — MINOR
- Synced 2026-03-11
- **Fixed:** `modify_id()` rewritten with radix_alpha iteration + ID_SUFFIX support
- **Fixed:** `record_id()` now calls `modify_id()` for duplicates (was `todo!()`)
- **Fixed:** `float_to_element()` warning condition was inverted (warned when canContainSomehow=true, should be false)
- **Fixed:** `float_to_label()` now starts from lastChild of current node (matching Perl)
- **Fixed:** `is_open()` now checks all child nodes (was element-only)
- **Added:** `remove_ss_values()`, `remove_class()`, `float_to_attribute()`
- **Replaced:** `_pre_comment`/`_comment` `todo!()` with TODO comment (needs libxml create_comment)
- **Added:** TODO for comment-swapping in `open_text_internal` (Perl lines 1139-1144)
- **Deferred (stubs):** `compact_xmdual()` (commented out body), `autoCollapseChildren()` (inline in finalize_rec)
- **Deferred:** `mergeAttributes()`, `getInsertionContext()`, `doctest()` diagnostic suite
- **Deferred:** `finalize_rec` font element name extraction from pending_declaration (`element` key)
- **Deferred:** `insertElementBefore()`, `getNodeLanguage()`, `decodeFont()`

### token.rs (vs Token.pm) — MINOR
- Reviewed 2026-03-11
- **Cosmetic:** T_COMMENT doesn't prepend `%` (Perl `bless ['%' . ($c || ''), CC_COMMENT]`). No test impact.
- **Minor:** T_ARG has no 1-9 validation (Perl calls Fatal for out-of-range). Low priority.
- **OK:** `get_cs_name()`, `defined_as()`, `substitute_parameters()`, `neutralize()` all match Perl.

### tokens.rs (vs Tokens.pm) — OK
- Reviewed 2026-03-11
- **Intentional divergence:** `untex` omits `%\n` line-break insertion (documented design decision)
- **Minor:** `strip_braces()` always strips 1 layer (Perl supports `$layers` parameter). Low priority.
- **OK:** `revert()`, `neutralize()`, `equals()`, `unlist()`, `Explode`/`ExplodeText` all match Perl.

### comment.rs (vs Comment.pm) — OK
- Synced 2026-03-11 (commit c6871020b)
- **Fixed:** `get_property("isEmpty")` now returns `true` (matches Perl `Comment->getProperty('isEmpty')`)
- **Deferred:** `insert_comment` in document.rs mostly stubbed (needs libxml create_comment)

### list.rs (vs List.pm) — MINOR
- Reviewed 2026-03-11
- **Deferred:** `List()` factory function with single-box simplification and horizontal list flattening
  (attempted and reverted — caused percent_test regression; needs per-callsite integration)
- **OK:** `List::new()`, `is_empty()`, `BoxOps` trait implementation all match Perl core behavior.

### whatsit.rs (vs Whatsit.pm) — MINOR
- Synced 2026-03-11 (commit c6871020b)
- **Fixed:** `set_body` now reads mode from whatsit's "mode" property (was binary `is_math()` only)
- **Fixed:** Trailer property copying expanded to handle TBox and List variants (was Whatsit-only)
- **Deferred:** `toAttribute` with parameter substitution (`#1`, `#prop` patterns) — not yet used in Rust
- **Deferred:** Reversion caching disabled, `computeSize` missing sizer string patterns (`'0'`, `'#\w+'`)

### parameter.rs (vs Parameter.pm) — MINOR
- Synced 2026-03-11 (commit 8ad6d8072)
- **Fixed:** OptionalMatch space-skipping after successful read (Perl L98-100)
- **Deferred:** MODE preservation in `digest()` (Perl L122/139-141) — causes percent_test regression
  when `leave_horizontal_internal()` fires during parameter digestion. Needs mature MODE tracking.
- **OK:** `read()`, `digest()`, `revert()`, `setup_catcodes`/`revert_catcodes` all match Perl core flow.

### parameters.rs (vs Parameters.pm) — OK
- Reviewed 2026-03-11
- Faithful port of multi-parameter container. `read_arguments()`, `digest()`, `revert()` match Perl.

### definition/ (vs Definition.pm, Register.pm) — MINOR
- Reviewed 2026-03-11
- **OK:** `addValue` logic is handled inline by `\advance`/`\multiply`/`\divide` in tex_registers.rs
  (Perl has convenience method on Register, Rust does it at call site — functionally equivalent)
- **Deferred:** Profiling hooks (startProfiling/stopProfiling/showProfile) are stubs only. Low priority.
- **Deferred:** FontDef parameter type not implemented. Blocks proper math font selection.

### conditional.rs (vs Conditional.pm) — OK
- Synced 2026-03-11 (commit b701adc60)
- **Fixed:** `\ifcase` negative number handling: `num > 0` → `num != 0` (Perl L88). Negative values now correctly skip all `\or` branches to `\else`.
- **Implemented:** IF_LIMIT infinite loop protection (Perl L63-65, was TODO). Uses `state::lookup_int("if_limit")`.
- **Implemented:** ALIGN_STATE tracking in `read_next_conditional` for `{`/`}` tokens during conditional body skipping (Perl L128-130).
- **Fixed:** Error message for "conditional fell off end" now includes start location (Perl L154-155).
- **Verified:** `\fi` frame comparison logic is correct — Rust pop-then-check is equivalent to Perl peek-then-pop.
- **Verified:** `invoke_conditional`, `invoke_else`, `invoke_fi` all match Perl logic.
- **Minor:** `PartialEq` only compares `cs` (Perl `equals` also compares parameters and test). Low priority.

### alignment.rs (vs Alignment.pm) — MINOR
- Synced 2026-03-11 (commits 5baef91a3, 7c35d8765, e5220ef18)
- **Implemented:** `normalize_mark_spans` (was stubbed `Ok(())`). Marks cells covered by colspan/rowspan as skipped, copies rowspan from spanned columns, truncates rowspan for non-empty cells, copies bottom borders.
- **Implemented:** `Digested::is_skippable()` matching Perl's `isSkippable` (L484-508).
- **Implemented:** Cell `skippable` field set in `normalize_cell_sizes` (Perl L469).
- **Fixed:** Absorption uses `skippable` (not `empty`) for cell content check (Perl L362).
- **Fixed:** `nextColumn` adds fallback column with align=center for extra tabs (Perl L140-143, was returning None).
- **Fixed:** colspan/rowspan attributes now passed to cell constructor (Perl L353-354, was commented out).
- **Fixed:** Border normalization sorts chars before joining (matching Perl L330-331 sort+dedup pattern).
- **Fixed:** `normalize_prune_rows` uses `skippable` with bracket-check heuristic (Perl L742-753).
- **Deferred:** Padding CSS classes (`ltx_nopad_l`, `ltx_nopad_r`) — needs `lspaces`/`rspaces` on Cell.
- **Deferred:** ABSORB_LIMIT guard (Perl L302-307) — needs global limit infrastructure.
- **Deferred:** Sizing info not passed to open_container (Perl L306-314) — commented out.
- **Deferred:** `\lx@intercol` intercolumn handling in template construction (tex_tables.rs TODO).

### rewrite.rs (vs Rewrite.pm) — GAPS
- Reviewed 2026-03-11
- Only ~20% ported (Select and Replace operators)
- **Missing:** Pattern compilation, label resolution, most operators
- **Known:** Low priority until more tests exercise rewrite rules

---

## Package.pm Sync (Perl API layer)

Audited 2026-03-11 against Package.pm (5022 lines, ~120 subroutines).
The Rust port distributes these across multiple modules:
- `latexml_core/src/state.rs` — value/catcode/definition lookups
- `latexml_core/src/binding/content.rs` — digestion/expansion/option/file/color functions
- `latexml_core/src/binding/def/dialect.rs` — Def* functions (macro, primitive, constructor, etc.)
- `latexml_core/src/binding/counter/dialect.rs` — counter management
- `latexml_core/src/common/cleaners.rs` — string cleaning utilities
- `latexml_package/src/prelude/setup_binding_language.rs` — macro wrappers

### Implemented (110+ of ~120 subs)

All core APIs ported:
- **State access:** LookupValue, AssignValue, PushValue, PopValue, UnshiftValue, ShiftValue, LookupMapping, AssignMapping, LookupMappingKeys
- **Catcode/code:** LookupCatcode, AssignCatcode, LookupMathcode, AssignMathcode, LookupSFcode, AssignSFcode, LookupLCcode, AssignLCcode, LookupUCcode, AssignUCcode, LookupDelcode, AssignDelcode
- **Definitions:** LookupMeaning, LookupDefinition, InstallDefinition, XEquals, IsDefined
- **Def forms:** DefMacro/I, DefPrimitive/I, DefConstructor/I, DefEnvironment/I, DefConditional/I, DefRegister/I, DefMath/I
- **Digestion:** Let, Digest, DigestText, DigestLiteral, DigestIf, Expand, ExpandPartially, Invocation, RawTeX
- **Counters:** NewCounter, CounterValue, SetCounter, AddToCounter, StepCounter, RefStepCounter, RefStepID, ResetCounter, GenerateID, deactivateCounterScope
- **File loading:** FindFile, Input, InputContent, InputDefinitions, RequirePackage, LoadClass, LoadPool, loadTeXDefinitions, loadTeXContent
- **Options:** DeclareOption, PassOptions, ProcessOptions, resetOptions, AddToMacro
- **Model:** Tag, DocType, RelaxNGSchema, RegisterNamespace, RegisterDocumentNamespace
- **Font:** DeclareFontMap, LoadFontMap, decodeMathChar
- **Other:** StartSemiverbatim, EndSemiverbatim, Tokenize, TokenizeInternal, requireMath, forbidMath, LookupRegister, LookupDimension, AssignRegister, DefParameterType, DefColumnType, allocateRegister, roman, Roman, CleanID, CleanLabel, CleanBibKey, CleanURL, ComposeURL, DefRewrite, DefMathRewrite, DefLigature, DefMathLigature, RequireResource, LookupColor, MaybeNoteLabel, AtBeginDocument, AtEndDocument, dualize_arglist, getXMArgID, defRobustCS

### Newly implemented (2026-03-11)

- **`IfCondition`** — `if_condition()` in content.rs. Tests a conditional and returns boolean. Reads arguments, invokes test closure, handles `\iftrue`/`\iffalse` fallbacks. Added `get_test()`/`get_conditional_type()` to Definition trait.
- **`SetCondition`** — `set_condition()` in content.rs. Sets `\newif`-type conditionals by Let to `\iftrue`/`\iffalse`.
- **`RefCurrentID`** — `ref_current_id()` in counter/dialect.rs. Recycles last ID without incrementing (Perl L876-881).
- **`MaybePeekLabel`** — `maybe_peek_label()` in counter/dialect.rs. Peeks for following `\label{...}` to support label-derived reference numbers (Perl L818-833).
- **`ExecuteOptions`** — `execute_options()` in content.rs + fixed `\ExecuteOptions` primitive in latex_ch5_packages.rs (was TODO stub).
- **`FontDecode`** — `font_decode()` in content.rs. Decodes codepoint through fontmap with family-specific map support.
- **`FontDecodeString`** — `font_decode_string()` in content.rs. Decodes string through fontmap, supports implicit mode and UTF-8 input encoding awareness.
- **`DefColor`** — `def_color()` in content.rs. Stores color value and defines `\\color@{name}` macro (Perl L3003-3015).
- **`DefColorModel`** — `def_color_model()` in content.rs. Stores derived color model info (Perl L3021-3024).
- **`CleanIndexKey`** — `clean_index_key()` in cleaners.rs. NFC normalization + trailing punctuation removal (Perl L513-525).
- **`CleanClassName`** — `clean_class_name()` in cleaners.rs. NFD decomposition, non-alnum removal, NFC recompose (Perl L527-535).
- **`NormalizeBibKey`** — `normalize_bib_key()` in cleaners.rs. Lowercase of clean_bib_key (Perl L546-548).
- **`TrimmedCommaList`** — `trimmed_comma_list()` in cleaners.rs. Split on commas, trim each (Perl L570-575).
- **`CleanBibKey` fix** — Now removes ALL whitespace (Perl `s/\s//sg`), was only trimming edges.

### Still missing / deferred

| Perl Function | Status | Notes |
|---|---|---|
| `createXMRefs` | DEFERRED | Complex DOM manipulation; used by math parser for XMRef creation. Will port when math absorption is exercised. |
| `defmath_introspective` | N/A | Perl runtime introspection; not applicable in compiled Rust |
| `CheckOptions` | N/A | Compile-time type checking replaces this in Rust |
| `DefExpandable` | N/A | Deprecated in Perl; use DefMacro instead |
| `ClearAutoLoad` | DEFERRED | Autoload infrastructure not yet needed |
| `maybeRequireDependencies` | DEFERRED | Heuristic TeX source scanning; Rust binding dispatch handles differently |
| `maybeReportSearchPaths` | DEFERRED | Minor logging utility |
| `FindFile_fallback` | DEFERRED | arXiv version-suffix stripping logic, skeleton exists |
| `LoadFormat` | DEFERRED | `.pool`/`.fmt` bootstrap; handled inline in Rust |
| `processRewriteSpecs` | DEFERRED | Internal to DefRewrite; handled in Rewrite struct construction |
| `ProcessPendingResources` | EXISTS | Already `document.process_pending_resources()` |
| `maybePreemptRefnum` | PARTIAL | Skeleton exists with `todo!()` — needs LABEL_MAPPING_HOOK closure type |
| `IsEmpty` (standalone) | PARTIAL | Method on Digested exists; standalone variadic version not needed in Rust |

---

## Font.pm → font.rs Sync (2026-03-11)

### File: `latexml_core/src/common/font.rs` ↔ `LaTeXML/lib/LaTeXML/Common/Font.pm`

#### Static Data
| Perl | Rust | Status |
|---|---|---|
| `%font_family` | `FONT_FAMILY` | **SYNCED** — fixed cmm (math not italic), cmsy/cmex/msa/msb (encoding-only), added graphic/xy fonts |
| `%font_series` | `FONT_SERIES` | OK |
| `%font_shape` | `FONT_SHAPE` | OK |
| `%font_size` | `FONT_SIZE` | OK |
| `$FONTREGEXP` | `FONT_RE` | **SYNCED** — updated to match new font entries |
| `%metric_map` | `METRIC_MAP` | **NEW** — full family_series_shape → tfm mapping |
| `@metric_fallbacks` | `METRIC_FALLBACKS` | **NEW** — cmr/cmmi/cmsy/cmex/msam/msbm |
| `%mathatomtype` | `MATH_ATOM_TYPE` | **NEW** — 20 role→type mappings |
| `$mathbearings` | `MATH_BEARINGS` | **NEW** — 8×8 bearing table |
| `$mathbearingreg` | inline in `math_bearing` | **NEW** — thinmuskip/medmuskip/thickmuskip register lookup |
| `%baseline_map` | `BASELINE_MAP` | **NEW** (not yet used in compute_boxes_size) |
| `%scriptstylemap` | `SCRIPT_STYLE_MAP` | OK |
| `%fracstylemap` | `FRAC_STYLE_MAP` | **RENAMED** from `_FRAC_STYLE_MAP` |
| `%stylesize` | `STYLE_SIZE` | OK |
| `%mathstylestep` | `MATH_STYLE_STEP` | OK |
| `%stepmathstyle` | `STEP_MATH_STYLE` | **RENAMED** from `_STEP_MATH_STYLE` |
| `%mathstylesize` | `MATH_STYLE_SIZE` | **RENAMED** from `_MATH_STYLE_SIZE` |
| `$FLAG_*` | `FLAG_FORCE_FAMILY/SERIES/SHAPE`, `FLAG_EMPH` | **NEW** — were commented out |

#### Subroutines: Constructors & Accessors
| Perl | Rust | Status |
|---|---|---|
| `new` | `Font { ... }` / `fontmap!` | OK — Rust struct construction |
| `new_internal` | `Font { ... }` | OK — direct struct construction |
| `textDefault` | `Font::text_default()` | OK |
| `mathDefault` | `Font::math_default()` | OK |
| `getFamily..getFlags` (11 accessors) | `get_family()..get_flags()` | OK |
| `toString` | `fmt::Debug` | OK — `Font[fam,ser,shp,...]` format |
| `stringify` | `Font::stringify()` | **NEW** — condensed non-default components |
| `asFontinfo` | `Font::as_fontinfo()` | **NEW** — returns HashMap |
| `equals` | `PartialEq` derive | OK |

#### Subroutines: Font Lookup
| Perl | Rust | Status |
|---|---|---|
| `lookupFontFamily` | `lookup_font_family()` | OK |
| `lookupFontSeries` | `lookup_font_series()` | OK |
| `lookupFontShape` | `lookup_font_shape()` | OK |
| `lookupTeXFont` | `lookup_tex_font()` | **NEW** |
| `decodeFontname` | `decode_fontname()` | OK |
| `rationalizeFontSize` | `rationalize_font_size()` | OK |
| `relativeFontSize` | `relative_font_size()` | OK |

#### Subroutines: Font Comparison & Matching
| Perl | Rust | Status |
|---|---|---|
| `isDiff` | `is_diff()` + `is_diff_opt_str()` + `is_diff_f64()` | OK |
| `match` | `Font::font_match()` | **NEW** — wildcard matching |
| `makeConcrete` | `Font::make_concrete()` | **NEW** |
| `relativeTo` | `Font::relative_to()` | **FIXED** — added color/bg/opacity/encoding/language/emph diffs |
| `distance` | `Font::distance()` | **FIXED** — matches Perl (no encoding/mathstyle, FLAG_EMPH) |
| `match_font` | `match_font()` | **NEW** — regex-based string matching |
| `font_match_xpaths` | `font_match_xpaths()` | **NEW** — XPath generation |
| `isSticky` | `Font::is_sticky()` | OK |

#### Subroutines: Font Merging
| Perl | Rust | Status |
|---|---|---|
| `merge` | `Font::merge()` | **REWRITTEN** — now handles forcebold, fraction, emph/FLAG_EMPH, force-flag blocking, scale, specialize |
| `specialize` | `Font::specialize()` | **FIXED** — DEFSERIES→DEFSHAPE bug, forceshape for lowercase Greek |
| `purestyleChanges` | `Font::purestyle_changes()` | OK |
| `mergePurestyle` | `Font::merge_purestyle()` | **NEW** — uses STEP_MATH_STYLE |

#### Subroutines: Metrics & Sizing
| Perl | Rust | Status |
|---|---|---|
| `getMetric` | `Font::get_metric()` | **REWRITTEN** — uses METRIC_MAP for proper lookup |
| `getMetricForName` | `get_metric_for_name()` | **NEW** — with fallback chain |
| `getEMWidth` | `Font::get_em_width()` | **FIXED** — uses get_metric(None) |
| `getEXHeight` | `Font::get_ex_height()` | **FIXED** — uses get_metric(None) |
| `getMUWidth` | `Font::get_mu_width()` | **FIXED** — uses get_metric(None) |
| `computeStringSize` | `Font::compute_string_size()` | **FIXED** — kerning + italic correction |
| `getNominalSize` | `Font::get_nominal_size()` | OK |
| `math_bearing` | `Font::math_bearing()` | **IMPLEMENTED** — was stub returning 0.0 |
| `computeBoxesSize` | `Font::compute_boxes_size()` | PARTIAL — single-pass, no word/line/stack decomposition |
| `computeBoxesSize_box` | inline in compute_boxes_size | PARTIAL — no separate function |
| `computeBoxesSize_words` | — | MISSING — word-level sizing with space/break handling |
| `computeBoxesSize_lines` | — | MISSING — line breaking logic |
| `computeBoxesSize_stack` | — | MISSING — multi-line stacking with vattach |
| `_showsize` | — | N/A — debug helper |

#### Subroutines: Font Decoding
| Perl | Rust | Status |
|---|---|---|
| Font decode (implicit) | `font::decode()` | OK |
| Font decode_string | `font::decode_string()` | OK |

#### Known Remaining Gaps
1. `DEFSIZE` is a static `10.0` — Perl's `DEFSIZE()` consults `$STATE->lookupValue('NOMINAL_FONT_SIZE')`. This matters when documents declare `\documentclass[12pt]{article}`.
2. `computeBoxesSize` doesn't decompose into `_words`/`_lines`/`_stack` sub-functions. The current implementation is a single-pass that doesn't handle line breaking, word boundaries, or vertical stacking with `vattach`.
3. `BASELINE_MAP` is defined but not yet used (needed by `computeBoxesSize_stack`).
4. `FONT_FAMILY` map: some entries intentionally differ from Perl (e.g. `cmbrs => symbol`, `ccy => symbol` kept for compatibility even though Perl doesn't have them).

---

## Core Type Sync: Token.pm, Tokens.pm, Number.pm, Float.pm, Dimension.pm

Audited 2026-03-11 against current Perl HEAD.

### Token.pm → token.rs — OK

| Perl | Rust | Status |
|---|---|---|
| CC_ESCAPE..CC_ARG (constants) | `Catcode` enum | OK |
| T_BEGIN..T_SUB (constants) | `TOKEN_*` statics + `T_*!()` macros | OK |
| T_SPACE, T_CR | `TOKEN_SPACE`, `TOKEN_CR` | OK |
| T_LETTER($c) | `T_LETTER!()` macro | OK |
| T_OTHER($c) | `T_OTHER!()` macro | OK |
| T_ACTIVE($c) | `T_ACTIVE!()` macro | OK |
| T_COMMENT($c) | `T_COMMENT!()` macro | OK — Perl prepends '%', Rust doesn't. Comments are filtered in toString anyway. |
| T_CS($c) | `T_CS!()` macro | OK |
| T_MARKER($t) | `T_MARKER!()` macro | OK |
| T_ARG($v) | `T_ARG!()` macro | OK — Perl validates 1-9 range, Rust doesn't (panics later if bad) |
| Token($string, $cc) | `Token!()` macro / `Token::new()` | OK |
| Explode($string) | `Explode!()` macro | OK |
| ExplodeText($string) | `ExplodeText!()` macro | OK — Perl `/[a-zA-Z]/` (ASCII), Rust `is_alphabetic()` (Unicode). Intentional. |
| UnTeX($thing) | `Tokens::untex()` | OK — Rust intentionally omits `%\n` line-breaks (documented design decision) |
| @CATCODE_PRIMITIVE | `Catcode::is_primitive()` | OK |
| @CATCODE_EXECUTABLE | `Catcode::is_executable()` | OK |
| @CATCODE_STANDARDCHAR | Not ported as array | N/A — only used in commented-out code |
| @CATCODE_NAME | `Catcode::name()` | OK |
| @CATCODE_PRIMITIVE_NAME | `get_primitive_name()` | OK |
| @CATCODE_SHORT_NAME | `Catcode::short_name()` | OK |
| @CATCODE_NEUTRALIZABLE | `Catcode::is_neutralizable()` | OK |
| isaToken() | — | N/A — Rust has static typing |
| getCSName() | `Token::get_cs_name()` / `with_cs_name()` | OK |
| getExecutableName() | `Token::get_executable_name()` | OK |
| getString() | `Token::with_str()` | OK |
| getCharcode() | `Token::get_charcode()` | OK |
| getCatcode() | `Token::get_catcode()` | OK |
| isExecutable() | `Token::is_executable()` | OK |
| unlist() | `Into<Vec<Token>>` | OK |
| stripBraces() | — on Token | OK — Token returns self, braces handled at Tokens level |
| neutralize() | `Token::neutralize()` | OK |
| substituteParameters(@args) | `Token::substitute_parameters()` | OK |
| packParameters() | Returns self (no-op) | OK |
| revert() | `Token::revert()` | OK |
| toString() | `Display for Token` | OK |
| beDigested() | `Token::be_digested()` | OK |
| equals($a,$b) | `PartialEq for Token` | OK |
| defined_as($token) | `Token::defined_as()` | OK |
| stringify() | `Token::stringify()` | FIXED — was using decimal "U+{c}/..." instead of hex "U+{:04x}/..." |

### Tokens.pm → tokens.rs — OK

| Perl | Rust | Status |
|---|---|---|
| Tokens(@tokens) | `Tokens!()` macro / `Tokens::new()` | OK |
| TokensI(@tokens) | `Tokens::new()` (no flattening) | OK |
| unlist() | `Tokens::unlist()` / `unlist_ref()` / `unlist_mut()` | OK |
| clone() | Rust `Clone` derive | OK |
| revert() | `Tokens::revert()` | OK |
| toString() | `Display for Tokens` (skips COMMENT) | OK |
| equals($a,$b) | `Tokens::equals()` (filters COMMENT+MARKER) | OK |
| stringify() | `Tokens::stringify()` | OK |
| beDigested() | `Tokens::be_digested()` | OK |
| neutralize() | `Tokens::neutralize()` | OK |
| isBalanced() | `Tokens::is_balanced()` | OK |
| substituteParameters(@args) | `Tokens::substitute_parameters()` | OK |
| packParameters() | `Tokens::pack_parameters()` | OK |
| stripBraces($layers) | `strip_braces()` / `strip_braces_n(layers)` | FIXED — was buggy: didn't verify brace pairing, stripped ALL layers. Now uses Perl's balanced-pair algorithm with `layers` parameter (default 1). |

### Number.pm → number.rs + numeric_ops.rs — OK

| Perl | Rust | Status |
|---|---|---|
| Number($number) | `Number!()` macro | OK |
| new($class,$number) | `Number::new()` via NumericOps | OK — truncates, not rounds |
| valueOf() | `NumericOps::value_of()` | OK |
| toString() | `Display for Number` | OK |
| $EPSILON, $ROUNDING_HALF | `EPSILON`, `ROUNDING_HALF` constants | OK — Rust uses f32-precision constants (sufficient for TeX) |
| roundto($number,$prec) | `round_to()` | OK |
| kround($number) | `kround()` | OK |
| unlist() | Object trait | OK |
| revert() | `Object::revert()` | OK |
| smaller($other) | `NumericOps::smaller()` | OK |
| larger($other) | `NumericOps::larger()` | OK |
| absolute() | `NumericOps::absolute()` | OK |
| sign() | `NumericOps::sign()` | OK |
| negate() | `NumericOps::negate()` | OK |
| add($other) | `NumericOps::add()` | OK |
| subtract($other) | `NumericOps::subtract()` | OK |
| multiply($other) | `NumericOps::multiply()` | OK — integer mult is exact, no truncation needed |
| divide($other) | `NumericOps::divide()` | OK — truncating |
| divideround($other) | `NumericOps::divideround()` | OK — rounding |
| stringify() | — | MINOR — Perl returns "Number[N]", Rust uses Display format. Cosmetic only. |
| getStretch() | — | N/A — returns Dimension(0). Not needed in Rust: eTeX registers use direct Glue field access |
| getShrink() | — | N/A — same as above |
| getStretchOrder() | — | N/A — same as above |
| getShrinkOrder() | — | N/A — same as above |

### Float.pm → float.rs — OK

| Perl | Rust | Status |
|---|---|---|
| Float($number) | `Float::new_f64()` / `Float::new()` | OK |
| new($class,$number) | `Float::new_f64()` | OK — no truncation |
| toString() | `Display for Float` → `floatformat()` | OK |
| multiply($self,$other) | `NumericOps::multiply()` override | OK — f64 mult, no truncation |
| stringify() | `Object::stringify()` | OK — "Float[N]" |
| floatformat($n) | `floatformat()` / `custom_float_format()` | OK — Rust adds `tight` mode for integer-like output |

### Dimension.pm → dimension.rs — OK

| Perl | Rust | Status |
|---|---|---|
| Dimension($spec) | `Dimension::new()` / `new_f64()` + `spec_to_f64()` | OK |
| _unit() | `NumericOps::unit()` → `Some("pt")` | OK |
| new($class,$spec) | `new_f64()` + `spec_to_f64()` | OK |
| toString() | `Display for Dimension` → `fixedformat()` | OK |
| toAttribute() | `NumericOps::to_attribute()` → `attribute_format()` | OK |
| stringify() | — | MINOR — Perl returns "Dimension[N]", Rust uses Display. Cosmetic. |
| $UNITY | `UNITY` constant (65536) | OK |
| fixpoint($float,$unit) | `fixpoint()` | OK |
| fixedformat($s,$unit) | `fixedformat()` | OK — Knuth's print_scaled §103 |
| attributeformat($sp,$unit) | `attribute_format()` | OK |
| ptValue($self,$prec) | `NumericOps::pt_value()` | OK |
| pxValue($self,$prec) | `NumericOps::px_value()` | FIXED — was hardcoded DPI=100, now reads from state |
| spValue($self,$prec) | `NumericOps::value_of()` | OK — returns SP directly |
| emValue($self,$prec,$font) | `Dimension::em_value()` | NEW — was missing, now uses font's EM width |

#### Fixes Applied This Session
1. **Token::stringify** hex format: `U+{c}/...` → `U+{:04x}/...` to match Perl's `sprintf("%04x",...)`.
2. **Tokens::strip_braces** rewritten with Perl's balanced-pair algorithm. Old impl was buggy: `{a}{b}` → `a}{b` (wrong), `{{a}}` → `a` (should be `{a}`). Added `strip_braces_n(layers)`.
3. **keyvals.rs** updated to call `strip_braces_n(2)` matching Perl's `stripBraces(2)`.
4. **Dimension::em_value** added (was missing). Converts to em units using font EM width.
5. **NumericOps::px_value** now reads DPI from state instead of hardcoding 100.
