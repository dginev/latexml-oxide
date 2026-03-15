# Sync Status — 2026-03-13 (evening)

**144 pass, 0 fail, 27 ignored**

## Recent Changes (this session)
- **cancel.sty FULLY WORKING** — All 14 diffs eliminated. Key insight: Perl's `Digest()` always returns truthy object, so `forcefont`/`cancelfont` are always set. Uses `_force_font='#forcefont'` on math XMTok for font computation via `finalize_rec`, `color_to_hex()` for named→hex color conversion.
- **Font defaults fixed** — `DEFBACKGROUND` → None (was "white"), `DEFLANGUAGE` → None (was "en"), matching Perl's `undef`. `rationalize_font_size` bug fixed (now parses numeric strings).
- **color.sty `lookup_color` made pub** — for cross-package color normalization (cancel.sty uses it).
- **soul.sty, stmaryrd.sty, wasysym.sty, accents.sty, xkeyval.sty** — Package bindings created and registered.
- **marvosym.sty, mathbbol.sty, bbold.sty, esint.sty, ulem.sty** — Package bindings created and registered.
- **`\DeclareMathAccent` + `\DeclareMathSymbol`** — Runtime primitives implemented with font_decode + def_math.
- **OML font map position 127** — Fixed to U+0361 (was U+0311). omencodings_test passes.
- **Theorem tests expanded** — 4 tests (3 pass, 1 ignored for math parser diffs).

## Test Status Breakdown

| Suite | Pass | Fail | Ignored | Notes |
|-------|------|------|---------|-------|
| hello | 1 | 0 | 0 | |
| contrib | 1 | 0 | 0 | |
| tokenize | 14 | 0 | 0 | |
| expansion | 36 | 0 | 0 | |
| grouping | 2 | 0 | 0 | |
| digestion | 10 | 0 | 0 | |
| fonts | 8 | 0 | 15 | cancels/marvosym now pass |
| encoding | 26 | 0 | 0 | |
| keyval | 4 | 0 | 3 | xkeyvalstyle/view need xkeyval features |
| keyval_options | 11 | 0 | 0 | |
| math | 0 | 0 | 1 | math parser research |
| structure | 24 | 0 | 0 | 18 more as .todo files |
| namespace | 0 | 0 | 1 | needs .latexml doc-level bindings |
| alignment | 0 | 0 | 2 | halign/tabtab need fixes |
| theorem | 4 | 0 | 1 | ntheorem: 897 math parser diffs |
| ams | 0 | 0 | 1 | math parser diffs |
| graphics | 0 | 0 | 1 | dvipsnam.def colors missing |
| parse | 0 | 0 | 1 | math parser |
| complex | 0 | 0 | 1 | needs aastex631.cls |
| babel | 0 | 0 | 1 | hangs (infinite loop, no Rust binding) |
| unit_parse | 3 | 0 | 0 | |

---

## Ignored Tests — Root Causes (27 total)

### Math parser issues (7 tests — deferred per CLAUDE.md)
| Test | Diffs | Root Cause |
|------|-------|------------|
| mixed_test | 37 | XMDual/XMApp parse tree structure |
| mathbbol_test | 110 | Math parse tree differences |
| mathaccents_test | 404 | Math structure diffs |
| ntheorem_test | 897 | Marpa grammar divergence |
| ams_test | 178 | equationgroup/subequations structure |
| parse_test | 118 | Algebraic term grouping |
| math_test | 109 | Relation chains, `<<`/`>>` operators |

### Missing packages/features (9 tests)
| Test | Diffs | Root Cause |
|------|-------|------------|
| stmaryrd_test | crash | `parse_kludge` todo!() in math parser |
| esint_test | crash | `parse_kludge` todo!() in math parser |
| acc_test | 163 | `\mathgroup` undefined, alignment crash |
| ding_test | - | Needs pifont.sty (pzd font map) |
| abxtest_test | - | Needs `\hexnumber@`, `\mathxfam` (font alloc) |
| soul_test | - | Needs `\ExplSyntaxOn` (LaTeX3/expl3) |
| wasysym_test | - | Needs `\Gin` (graphics), `\ExplSyntaxOn` |
| mathcolor_test | - | Needs `\Gin`, `\ExplSyntaxOn` |
| babel_test | hang | Infinite loop — no Rust babel binding |

### Large diff counts (6 tests)
| Test | Diffs | Root Cause |
|------|-------|------------|
| fonts_test | 1001 | `\fontname` not implemented |
| plainfonts_test | - | `\fontname` not implemented |
| sizes_test | 377 | Many sizing/layout diffs |
| bbold_test | 677 | Table + math structure |
| complex_test | 19 | Needs aastex631.cls binding |
| namespace_test | 13 | Custom .latexml doc-level bindings |

### xkeyval feature tests (3 tests)
| Test | Diffs | Root Cause |
|------|-------|------------|
| keyvalstyle_test | 26 | xkeyval style environments |
| xkeyvalstyle_test | 13 | xkeyval style handling |
| xkeyvalview_test | 9 | xkeyval view + tabular |

### Alignment (2 tests)
| Test | Diffs | Root Cause |
|------|-------|------------|
| halign_test | 51 | Missing `class="ltx_nopad_r"`, bracket in cell |
| tabtab_test | 11 | Nested tabular not processed |
| graphicx_test | 144 | dvipsnam.def colors all #000000 |

---

## Comprehensive Perl→Rust Audit

### TeX Engine Pools — Coverage by File

| Perl File | Rust File | Coverage | Notes |
|-----------|-----------|----------|-------|
| TeX_Character | tex_character.rs | **100%** | All character handling, accents, case conversion |
| TeX_Logic | tex_logic.rs | **100%** | All conditionals |
| TeX_Macro | tex_macro.rs | **100%** | def/edef/gdef/xdef/let/futurelet/expandafter/the |
| TeX_Paragraph | tex_paragraph.rs | **100%** | Line/paragraph breaking |
| TeX_Registers | tex_registers.rs | **100%** | Register allocation |
| TeX_Kern | tex_kern.rs | **100%** | Kerning and movement |
| TeX_Glue | tex_glue.rs | **100%** | Glue/spacing |
| TeX_Job | tex_job.rs | **100%** | jobname/day/month/year/time/mag |
| TeX_Penalties | tex_penalties.rs | **100%** | All penalties |
| TeX_Page | tex_page.rs | **100%** | Page layout |
| TeX_Marks | tex_marks.rs | **100%** | Mark commands |
| TeX_Inserts | tex_inserts.rs | **100%** | Insert commands |
| TeX_Debugging | tex_debugging.rs | **100%** | Debug commands |
| TeX_FileIO | tex_file_io.rs | **98%** | Missing: `\lx@special@graphics` |
| TeX_Box | tex_box.rs | **95%** | Missing: `\leaders` body, SVG collapse ops |
| TeX_Fonts | tex_fonts.rs | **90%** | Ligatures compiled to static data (architectural diff) |
| TeX_Math | tex_math.rs | **80%** | Missing: math atom adjusters |
| TeX_Tables | tex_tables.rs | **95%** | Missing: advanced alignment templates |
| eTeX | etex.rs | **98%** | Missing: `\directlua` (LuaTeX only) |
| pdfTeX | pdftex.rs | **60%** | Many PDF-specific primitives stubbed |
| Base_Schema | base_schema.rs | **100%** | All 15 definitions |
| Base_XMath | base_xmath.rs | **90%** | Missing: matrix/cases bindings |
| Base_Functions | base_functions.rs | **95%** | Core constructor logic |
| plain.tex | plain.rs | **95%** | Missing: `\beginsection` |
| LaTeX bootstrap | latex.rs | **100%** | All 10 definitions |

### Package Binding Coverage

| Package | Coverage | Status |
|---------|----------|--------|
| xkeyval.sty | **100%+** | Complete |
| article.cls | **98%** | Near-complete |
| color.sty | **~90%** | Ported — missing dvipsnam.def |
| cancel.sty | **100%** | Fully working |
| amsthm.sty | **~90%** | Good |
| hyperref.sty | **~65%** | Key features missing |
| graphicx.sty | **~70%** | Sizer/properties stubs |
| amsmath.sty | **~10%** | CRITICAL gap |
| natbib.sty | **~8%** | CRITICAL gap |
| marvosym.sty | **new** | Basic binding |
| mathbbol.sty | **new** | Basic binding |
| bbold.sty | **new** | Basic binding |
| esint.sty | **new** | Basic binding |
| ulem.sty | **new** | Working |
| soul.sty | **new** | Basic binding |
| stmaryrd.sty | **new** | Basic binding |
| wasysym.sty | **new** | Basic binding |
| accents.sty | **new** | Basic binding |

---

## Recommended Work Order

### Phase 1 — Most accessible improvements
1. Fix alignment engine — `halign_test` (51 diffs), `tabtab_test` (11 diffs)
2. Port `dvipsnam.def` color definitions — unblocks graphicx_test colors
3. Fix `namespace_test` (13 diffs) — custom .latexml loading
4. Fix `complex_test` (19 diffs) — needs aastex631.cls or ERROR tolerance

### Phase 2 — Enable structure .todo tests (highest ROI)
1. Implement `\eqnarray` environment (latex_constructs ~L2250)
2. Implement `\addcontentsline`/`\tableofcontents` (~L3800)
3. Complete `\caption` chain (~L1010)
4. Port bibliography basics (`\bibcite`, `\@cite`, `\nocite`)
5. Implement `\bfseries`/`\mdseries` font series commands

### Phase 3 — Complete amsmath for AMS tests
1. Port remaining ~90% of amsmath.sty.ltxml
2. Implement `\big`/`\Big`/`\bigg`/`\Bigg` delimiters (19 definitions)
3. Port math atom adjusters: `\mathrel`, `\mathbin`, `\mathord`, `\mathop`, etc.
4. Complete `compact_xmdual()`

### Phase 4 — Package gaps
1. Port `pifont.sty` (140 lines) — unblocks ding_test
2. Port `babel.sty` basics — stops infinite loop
3. Port `array.sty` (650 lines) — enhanced tables
4. Complete natbib.sty citation logic

---

## Deferred Items
- `compact_xmdual()` body — document.rs
- `mergeAttributes()` — document.rs
- `\fontname` full format ("select font X at Ypt")
- `\font` primitive (metrics/at/scaled)
- `\fontdimen` getter/setter (hardcoded stubs)
- `\leaders`/`\cleaders`/`\xleaders` (stub no-ops)
- `\lx@special@graphics` constructor
- BibTeX entry processing
- `parse_kludge` in math parser (blocks stmaryrd/esint)
