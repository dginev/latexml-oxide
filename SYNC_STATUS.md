# Sync Status — 2026-03-12

**134 pass, 16 fail, 4 ignored** (was 107/40/6 before encoding & math fixes)

## Recent Fixes (2026-03-12, session 2)

### Dynamic mathstyle/scriptpos for variable-size operators
- Added `dynamic_mathstyle` and `dynamic_scriptpos` bool flags to `MathPrimitiveOptions`
- `def_math_primitive` now computes mathstyle/scriptpos at invocation time from current font
- Perl `doVariablesizeOp`: "display" in display mode, "text" in inline → `dynamic_mathstyle`
- Perl `doScriptpos`: "mid" in display mode, "post" in inline → `dynamic_scriptpos`
- Key finding: `\int`/`\oint` have only `mathstyle` (no scriptpos); `\sum`/`\prod`/etc have both
- `\smallint` has dynamic scriptpos but STATIC `mathstyle => "text"`
- Added `mathstyle` handler to `defi_opts!` macro for `Option<String>` literal values
- Files: `math_primitive.rs`, `dialect.rs`, `plain.rs`, `setup_binding_language.rs`
- **Fixed applemac_test** (was the last encoding failure, all 26 encoding tests now pass)

### Encoding test fixes (session 1, carried over)
- vrule/hrule whatsit sizing with cached_width/cached_height/cached_depth
- Dimension `to_attribute()` for 1-decimal-place formatting
- isVerticalRule logic corrected (only set when height dominates)
- Float superscript italic font preservation via `document.set_attribute`
- `\dots` sizer added for non-zero computed width in tabulars
- Fixed 6 cp* encoding tests, 3 ansi/cp12* tests, 1 applemac test

### Test status breakdown
| Suite | Pass | Fail | Ignored |
|-------|------|------|---------|
| hello | 1 | 0 | 0 |
| contrib | 1 | 0 | 0 |
| unit_state | 9 | 0 | 0 |
| unit_tokens | 1 | 0 | 0 |
| tokenize | 14 | 0 | 0 |
| expansion | 36 | 0 | 0 |
| grouping | 2 | 0 | 0 |
| digestion | 10 | 0 | 0 |
| fonts | 1 | 0 | 0 |
| encoding | 26 | 0 | 0 |
| math | 2 | 12 | 0 |
| structure | 24 | 0 | 0 |
| namespace | 0 | 0 | 1 |
| alignment | 0 | 2 | 0 |
| theorem | 4 | 0 | 0 |
| ams | 0 | 0 | 1 |
| graphics | 0 | 1 | 0 |
| unit_parse | 3 | 0 | 0 |
| parse | 0 | 0 | 1 |
| complex | 0 | 1 | 0 |
| babel | 0 | 0 | 1 |

### Pre-existing failures (not caused by recent changes)
- **12 math tests**: math parser role/structure issues (UNKNOWN vs ID, function application)
- **2 alignment tests**: nested tabular layout (tabtab, halign)
- **1 graphics test**: infinite recursion in `\usepackage{color}`
- **1 complex test**: aastex631 class attribute/resource issues

## Recent Gap Fix Progress (2026-03-12)

### todo!()/unported!() elimination
- **0 `unported!()` calls remain** (was 7)
- **1 `todo!()` remains in latexml_package** (was 12+), down to a DefKeyVal macro branch
- **67 `todo!()` remain in latexml_core** — mostly trait defaults and register arithmetic

### Items implemented this session:
| Item | File | Change |
|------|------|--------|
| `\hglue Glue` | plain.rs | DefPrimitive with dimension_to_spaces |
| Fill operations | plain.rs | `\hrulefill/\dotfill` → DefMacro; arrows/braces → DefMath |
| `\makeindex`, `\makeglossary`, `\indexspace` | latex_ch11 | Stub primitives |
| `\index {}` | latex_ch11 | Stub primitive (discards arg) |
| `augment_delimiter_properties` | plain.rs | Full Perl-matching upgrade with DELIM_CHAR_MAP |
| `From<Glue> for Stored` | store.rs | Missing impl added |
| eTeX expr close-paren error | etex.rs | Error message instead of todo!() |
| `\arrowvert`, `\Arrowvert` | latex_ch15 | DefMath (was commented) |
| `\mapstochar`, `\owns` | latex_ch15 | DefMath (was commented) |
| `\cdotp`, `\ldotp` | latex_ch15 | DefMath (was commented) |
| shortstack properties | latex_ch14 | align + vattach properties |
| table layout property | latex_ch9 | `layout => "vertical"` |
| `\@@caption` / `\@@toccaption` | latex_ch9 | Added `^^` float-up prefix |
| `revert_spec()` | base_functions.rs | Implemented (Explode keyword + revert value) |
| `adjust_box_color_rec()` | base_functions.rs | Stub no-op (was todo!()) |
| tabular→XMArray conversion | base_functions.rs | Stub no-op (was todo!()) |
| `add_meaning_rec()` | base_xmath.rs | Full implementation |
| `\lx@xmDual` reversion | base_xmath.rs | "dual" + context-dependent branches |
| `\lx@intercol` + text/math variants | tex_tables.rs | DefMacro + DefConstructors |
| `svg:g` tag handler | tex_box.rs | Stub no-op (was unported!()) |
| `\accent` | plain.rs | Stub no-op (was unported!()) |
| `\joinrel` + `\@@joinrel` | plain.rs | Simplified stubs (was unported!()) |
| `\@add@to@frontmatter` | base_utilities.rs | Stub no-op (was unported!()) |
| `DirectoryList` param type | base_parameter_types.rs | Returns empty (was unported!()) |
| `CommaList` param type | base_parameter_types.rs | Returns empty (was unported!()) |
| Variable reversion | base_parameter_types.rs | Token passthrough (was todo!()) |
| parseParameters unreachable | base_functions.rs | unreachable!() (was todo!()) |
| Constructor Display | constructor.rs | Proper fmt (was todo!()) |
| RelaxNG load_schema | relaxng.rs | Stub no-op (was todo!()) |
| HTTP/HTTPS mouth | mouth.rs | Warning stubs (was todo!()) |
| Namespace attributes | document.rs | Set directly (was todo!()) |
| Postponed+KeyVals+RegisterValue absorption | document.rs | Graceful handling (was todo!()) |
| checkin_value edge cases | state.rs | Warning instead of panic (was todo!()) |
| Aligned equation row tagging | latex_ch7_math | Stub with TODO (was todo!()) |

---

## Engine .ltxml Audit (Perl → Rust) — Cross-Verified

### Fully Synced (no actionable gaps)
| Perl File | Rust File | Notes |
|-----------|-----------|-------|
| TeX_Character | tex_character.rs | Complete. `\accent` intentionally uses `\lx@applyaccent` |
| TeX_Logic | tex_logic.rs | Complete. `\ifmmode` uses IN_MATH flag (deliberate) |
| TeX_Penalties | tex_penalties.rs | Complete |
| TeX_Page | tex_page.rs | Complete |
| TeX_Marks | tex_marks.rs | Complete |
| TeX_Inserts | tex_inserts.rs | Complete |
| TeX_Debugging | tex_debugging.rs | Complete (logging uses eprintln vs NoteLog) |
| TeX_Registers | tex_registers.rs | Complete |
| eTeX | etex.rs | Complete (43/43 defs) |
| Base_Schema | base_schema.rs | Complete (15/15 defs) |
| latex_bootstrap+LaTeX | latex.rs | Complete (10/10 defs) |
| latex_base | latex_ch*.rs + appendices | Complete (~160/160 defs, distributed across 36 ch files) |

### Gaps Found — Ranked by Impact

#### 1. Base_XMath (base_xmath.rs) — ~15 constructors commented out
- `\lx@apply OptionalKeyVals:XMath {}{}` — semantic function application
- `\lx@symbol OptionalKeyVals:XMath {}` — math symbol with attributes
- `\lx@wrap OptionalKeyVals:XMath {}` — semantic wrapping
- `\lx@superscript`/`\lx@subscript OptionalKeyVals:XMath {} InScriptStyle` — semantic sub/superscript
- `\lx@padded[MuDimension]{MuDimension}{}` — padded math content
- `\lx@math@tweaked`/`\lx@text@tweaked RequiredKeyVals {}` — adjustments
- `\lx@gen@matrix@bindings`/`\lx@gen@plain@matrix@`/`\lx@ams@matrix@` — matrix environments
- `\lx@cases@condition`/`\lx@cases@end@condition`/`\lx@gen@plain@cases@` — cases environment
- `\lx@gen@cases@bindings` — cases setup
- DefRewrite for mixed fractions — completely missing
- ~~`\lx@dual` reversion closure — `todo!()` stub~~ DONE: dual + context branches
- ~~`add_meaning_rec()` function — `todo!()` stub~~ DONE: full implementation

#### 2. math_common → Delimiters (19 missing)
File `latex_ch7_math_common_delimiters.rs` is **empty** — all sized delimiters unimplemented:
- `\big`, `\Big`, `\bigg`, `\Bigg` TeXDelimiter
- `\bigl`/`\bigm`/`\bigr`, `\Bigl`/`\Bigm`/`\Bigr`
- `\biggl`/`\biggm`/`\biggr`, `\Biggl`/`\Biggm`/`\Biggr`

#### 3. TeX_Tables (tex_tables.rs) — partially addressed
- `\halign BoxSpecification` constructor (infrastructure exists, constructor not wired)
- ~~`\lx@intercol`, `\lx@text@intercol`, `\lx@math@intercol`~~ DONE
- `\lx@alignment@ncolumns`, `\lx@alignment@column` registers
- `\lx@alignment@multicolumn` macro
- `\lx@alignment@bindings` primitive
- `beforeCellUnlist`/`afterCellUnlist` helpers

#### 4. TeX_Fonts (tex_fonts.rs) — 12+ gaps
- FontDef parameter type (stub, not full implementation)
- `\font` primitive (simplified, no metrics/at/scaled)
- `\fontname FontDef` (returns "not implemented")
- `\fontdimen` getter/setter (hardcoded stubs)
- 6 `DefLigature` calls not ported
- `$nominal_fontinfo` array not ported
- Default font initialization (`\font\lx@default@font=cmr10`)

#### 5. plain.rs — reduced to 5 commented out items
- `\@@oalign`/`\@@ooalign` constructors (alignment-based)
- `\@math@daccent`/`\@math@baccent` DefConstructor (math/text accent constructors)
- `\lx@hack@bordermatrix` constructor
- `\@@eqalign`/`\@@eqalignno`/`\@@leqalignno` constructors (display math alignment)
- `\displaylines{}` — commented out
- ~~`\hglue Glue`~~ DONE
- ~~fill operations~~ DONE
- ~~`\accent`~~ DONE (stub)
- ~~`\joinrel`/`\@@joinrel`~~ DONE (stub)

#### 6. latex_constructs — eqnarray + picture + index + misc
- `\eqnarray` environment + `\@eqnarray@bindings` — NOT FOUND in any Rust file
- ~~`\rule` command~~ was already implemented in latex_ch13_boxes.rs
- `\index`/`\@index` — index system partially stubbed (full process_index_phrases deferred)
- ~~`\makeindex`~~ DONE
- Picture environment (`\line{}`, `\vector{}`, `\circle`, `\oval`, `\@bezier`) — NOT FOUND
- `\@xargdef`/`\@yargdef`/`\@reargdef` — NOT FOUND
- `\DeclareMathAccent` — commented out in latex_ch8_defining_commands.rs

#### 7. TeX_Math (tex_math.rs) — verified after cross-check
Most items are in other files. Genuinely missing:
- `\lx@delimiterdot` (handled inline in `\@left`/`\@right` via hint property)
- 2 DefMathLigature: `···→⋯`, `...→…`

#### 8. TeX_Box (tex_box.rs) — leaders + SVG
- `\leaders`/`\cleaders`/`\xleaders` (stub no-ops, need full constructor)
- `\lx@math@nounicode`, `\lx@text@nounicode` (commented out)
- ~~SVG foreignObject sizing, group collapsing~~ svg:g stub done
- `insertBlock`, `hackVBoxAttachment` helpers incomplete
- ~~`adjustBoxColor`~~ DONE (stub)
- `\setbox` missing `SkipSpaces` parameter

#### 9. TeX_Glue (tex_glue.rs) — reversion + features
- `revertSkip()` subroutine entirely missing
- `\hskip` missing: reversion property, SVG handling, isMath/XMHint
- `\vskip` missing: `height` property
- `\qquad` spacing entry missing from unicode table

#### 10. TeX_FileIO (tex_file_io.rs) — graphics
- `\lx@special@graphics` constructor (commented out, ~50 lines)
- 7 `DefKeyVal` entries for SpecialPS (commented out)
- `\openin`/`\openout` missing first `SkipSpaces` parameter
- `\special` uses `{}` instead of `XGeneralText`

#### 11. Base_ParameterTypes — mostly addressed
- `ScriptscriptStyle` parameter type — missing
- ~~`DirectoryList`~~ DONE (stub)
- ~~`CommaList`~~ DONE (stub)

#### 12. Base_Utility (base_utilities.rs) — addressed
- ~~`\@add@to@frontmatter@now`~~ DONE (stub)
- `\lx@frontmatter@fallback` — returns None (incomplete)
- Reference formatting macros (`lx@the@@`, `lx@fnum@@`, etc.) are all present

#### 13. Minor gaps (low priority)
- TeX_Macro: `\the` missing FontDef case
- TeX_Job: `DumpFile()` intentionally deferred
- pdfTeX: 4 missing (`\pdfannot`, `\pdfcolorstack`, `\pdfobj`, OpenAnnotSpecification)
- TeX_Paragraph: `alignLine()` helper missing
- TeX_Kern: `raisedSizer()` helper (logic inlined)
- TeX_Hyphenation: `\hyphenchar` getter/setter incomplete

### Specialized Packages (low coverage, lower priority)

| Perl File | Rust Coverage | Key Missing Items |
|-----------|--------------|-------------------|
| Base_Deprecated (77 defs) | ~16% | Mostly deprecated compat shims (`\@@BEGININLINEMATH`, etc.). Port on-demand. |
| AmSTeX (459 lines, ~112 defs) | ~30% | Format control, sp-accents, display environments, cfrac. Port on-demand. |
| BibTeX (956 lines, ~150 defs) | ~9% | Almost entirely unimplemented. Entry processing, field handlers, name constructors. |

### Audit Summary

| File Group | Perl Defs | Rust Coverage | Status |
|------------|-----------|---------------|--------|
| TeX_* Engine (12 files) | ~350 | ~96% | Mostly complete, minor gaps |
| eTeX | 43 | 100% | Complete |
| pdfTeX | 20 | ~80% | 4 missing |
| plain_* (3 files) | ~110 | ~92% | 5 items commented out (was 10) |
| Base_Schema | 15 | 100% | Complete |
| Base_ParameterTypes | 59 | ~97% | 1 unported type (was 3) |
| Base_Utility | 41 | ~95% | 1 stub (was 2) |
| Base_XMath | 64 | ~55% | Largest gap — constructors commented |
| latex_bootstrap+LaTeX | 10 | 100% | Complete |
| latex_base | ~160 | ~100% | Complete |
| latex_constructs (6013 lines) | ~843 | ~91%+ | eqnarray + defining cmds missing |
| math_common | 312 | ~87% | Delimiters empty, accents/phantoms present |
| Base_Deprecated | 77 | ~16% | Low priority |
| AmSTeX | 112 | ~30% | Low priority |
| BibTeX | 150 | ~9% | Low priority |

### Remaining todo!()/unported!() inventory
- **latexml_package**: 1 todo!() (DefKeyVal macro branch, compile-time only)
- **latexml_core**: 67 todo!() — breakdown by file:
  - `definition/register.rs` (16): Token/Tokens RegisterValue arithmetic (add/sub/mul/div/etc.)
  - `lib.rs` (7): BoxOps trait defaults (unlist, be_absorbed, get_tokens, etc.)
  - `definition/argument.rs` (7): AlignmentTemplate/RegisterDefinition edge cases
  - `digested.rs` (6): Unhandled DigestedData variant defaults
  - `definition/expandable.rs` (5): Profiling hooks (not ported from Perl)
  - `definition.rs` (4): Register trait stubs
  - `alignment.rs` (4): compute_size, get_font, get_string, be_absorbed
  - `rewrite.rs` (3): Pattern matching edge cases
  - `keyvals.rs` (3): set_property, compute_size, set_keys_expansion variant
  - `common/object.rs` (2): Trait defaults (intentional — catch missing impls)
  - Other (10): 1 each in tokens, stomach, state, list, primitive, conditional, numeric_ops, error, macros, counter/dialect
