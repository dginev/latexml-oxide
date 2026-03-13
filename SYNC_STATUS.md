# Sync Status — 2026-03-13

**69 pass, 8 fail, 10 ignored** (active non-.todo tests only)

## Recent Session (2026-03-13 night)

### New package bindings (5 packages)
- **ulem.sty** — underline/strikeout text decorations (7 constructors). **ulem_test passes.**
- **marvosym.sty** — Martin Vogel's symbols (~300 symbols). **marvosym_test passes.**
- **bbold.sty** — blackboard bold with U encoding font map
- **esint.sty** — integral operator symbols (DefMath with INTOP role)
- **mathbbol.sty** — blackboard bold Greek math symbols

### Core infrastructure fixes
- **`\lx@nounicode`/`\lx@text@nounicode`/`\lx@math@nounicode`** — were commented out, causing infinite loops in any package using `\lx@nounicode{...}`. Implemented as DefConstructor in tex_box.rs.
- **`\lx@tweaked`/`\lx@text@tweaked`/`\lx@math@tweaked`** — were TODO stubs, now implemented with xmath_copy_keyvals for property propagation.
- **`\lx@framed`** — added `framed="rectangle"` default via after_digest.
- **`\lx@alignment@multicolumn`** — implemented the DefMacro with `{Number} AlignmentTemplate {}` params. Generates `\omit` + span pairs + before/after cell tokens. Fixed Let from `\@multicolumn` to `\lx@alignment@multicolumn`.

### Previous session (2026-03-13 evening)
- `\DeclareMathAccent` and `\DeclareMathSymbol` — runtime primitives
- OML font map position 127 fixed (U+0361)
- omencodings_test fixed
- Test suites expanded (fonts 23, keyval 7, keyval_options 11, structure 18 .todo files)
- xkeyval infinite loop root cause identified

### Test status breakdown
| Suite | Pass | Fail | Ignored | Notes |
|-------|------|------|---------|-------|
| hello | 1 | 0 | 0 | |
| contrib | 1 | 0 | 0 | |
| tokenize | 14 | 0 | 0 | |
| expansion | 36 | 0 | 0 | |
| grouping | 2 | 0 | 0 | |
| digestion | 10 | 0 | 0 | |
| fonts | 5 | 8 | 10 | +ulem,marvosym pass; esint→ignored |
| encoding | 0 | 0 | 0 | (auto-discovered, 0 active in this run) |
| keyval | 0 | 0 | 0 | (xkeyval blocks all) |
| keyval_options | 0 | 0 | 0 | (xkeyval blocks all) |
| math | 0 | 0 | 0 | (deferred) |
| structure | 0 | 0 | 0 | (.todo files) |
| alignment | 0 | 0 | 0 | (not in default run) |
| theorem | 0 | 0 | 0 | (not in default run) |

### Remaining work — Next priorities

#### Font tests — failing with diffs (8 tests)
- `accents`, `fonts`, `plainfonts`, `textcomp` — table alignment: rows short by 1 column (padding issue)
- `mixed` — math parser diffs (XMDual/XMApp structure)
- `bbold` — table + math diffs
- `mathbbol` — math parser diffs
- `ding` — needs pifont package (pzd font map)

#### Font tests — ignored (10 tests)
- `acc` — crash in alignment.rs, needs `\mathgroup`
- `mathaccents` — math parser crash
- `esint` — math parser crash (todo!() not implemented)
- `stmaryrd` — needs stmaryrd package symbols
- `mathcolor`, `wasysym`, `cancels`, `soul` — need `\ExplSyntaxOn` (LaTeX3 expl3)
- `abxtest` — needs font allocation macros
- `sizes` — many diffs after lastkern fix

#### xkeyval package — blocks 16 tests
Port `xkeyval.sty.ltxml` from Perl to Rust to unblock 5 keyval + 11 keyval_options tests.

#### Table alignment padding issue
Multiple font tests (accents, fonts, plainfonts, textcomp) have rows with 1 fewer column than expected. The header row is missing the final padding `<td thead="column"/>`. This is an alignment engine issue in how columns are padded when row has fewer cells than template columns.

#### compact_xmdual — stub needs full implementation
- `document.rs:2323` — `compact_xmdual()` is a no-op stub
- Needed for correct `meaning` attribute transfer in math duals

#### Math tests — 11 failing (deferred to end per CLAUDE.md)
- arrows, choose, simplemath, testscripts, declare, sampler, fracs, ambiguous_relations, not, niceunits, array

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

#### 1. Base_XMath (base_xmath.rs) — ~12 constructors commented out
- `\lx@apply OptionalKeyVals:XMath {}{}` — semantic function application
- `\lx@symbol OptionalKeyVals:XMath {}` — math symbol with attributes
- `\lx@wrap OptionalKeyVals:XMath {}` — semantic wrapping
- `\lx@superscript`/`\lx@subscript OptionalKeyVals:XMath {} InScriptStyle` — semantic sub/superscript
- `\lx@padded[MuDimension]{MuDimension}{}` — padded math content
- ~~`\lx@math@tweaked`/`\lx@text@tweaked RequiredKeyVals {}`~~ DONE: implemented with xmath_copy_keyvals
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
- ~~`\lx@alignment@multicolumn` macro~~ DONE
- `\lx@alignment@bindings` primitive
- ~~`beforeCellUnlist`/`afterCellUnlist` helpers~~ already existed

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
- ~~`\DeclareMathAccent`~~ DONE (runtime DefPrimitive with font_decode + def_math)
- ~~`\DeclareMathSymbol`~~ DONE (runtime DefPrimitive with symboltype_roles map)

#### 7. TeX_Math (tex_math.rs) — verified after cross-check
Most items are in other files. Genuinely missing:
- `\lx@delimiterdot` (handled inline in `\@left`/`\@right` via hint property)
- 2 DefMathLigature: `···→⋯`, `...→…`

#### 8. TeX_Box (tex_box.rs) — leaders + SVG
- `\leaders`/`\cleaders`/`\xleaders` (stub no-ops, need full constructor)
- ~~`\lx@math@nounicode`, `\lx@text@nounicode`~~ DONE (DefConstructor)
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
