# Sync Status — 2026-03-13

**153 pass, 0 fail, 28 ignored**

## Recent Changes
- **color.sty PORTED** — `\definecolor`, `\color`, `\textcolor`, `\colorbox`, `\fcolorbox`, `\pagecolor`, `\normalcolor`, color group macros, default colors (black/white/red/green/blue/cyan/magenta/yellow). Supports rgb/cmyk/cmy/gray/hsb models with hex conversion.
- **cancel.sty PORTED** — `\cancel`, `\bcancel`, `\xcancel`, `\cancelto` with math/text dispatch and `cancelColorProperties` font capture. 14 diffs remain due to `_force_font`/`_font` property handling (text wrapper not created when font unchanged).
- Keyval options tests — all 11 enabled and passing
- `\LoadClass` afterDigest implemented (was commented out)

## Test Status Breakdown

| Suite | Pass | Fail | Ignored | Notes |
|-------|------|------|---------|-------|
| hello | 1 | 0 | 0 | |
| contrib | 1 | 0 | 0 | |
| tokenize | 14 | 0 | 0 | |
| expansion | 36 | 0 | 0 | |
| grouping | 2 | 0 | 0 | |
| digestion | 10 | 0 | 0 | |
| fonts | 7 | 0 | 16 | 16 ignored need packages |
| encoding | 26 | 0 | 0 | |
| keyval | 4 | 0 | 3 | 3 ignored (xkeyvalstyle/view) |
| keyval_options | 11 | 0 | 0 | **ALL PASS** |
| math | 0 | 0 | 1 | deferred per CLAUDE.md |
| structure | 24 | 0 | 0 | 18 more as .todo files |
| namespace | 0 | 0 | 1 | needs .latexml bindings |
| alignment | 4 | 0 | 2 | halign/tabtab need fixes |
| theorem | 3 | 0 | 0 | |
| ams | 0 | 0 | 1 | math parser diffs |
| graphics | 0 | 0 | 1 | |
| parse | 0 | 0 | 1 | math parser |
| complex | 0 | 0 | 1 | |
| babel | 0 | 0 | 1 | |
| unit_parse | 4 | 0 | 0 | |

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
| TeX_Math | tex_math.rs | **80%** | Missing: math atom adjusters (see below) |
| TeX_Tables | tex_tables.rs | **95%** | Missing: advanced alignment templates |
| eTeX | etex.rs | **98%** | Missing: `\directlua` (LuaTeX only) |
| pdfTeX | pdftex.rs | **60%** | Many PDF-specific primitives stubbed |
| Base_Schema | base_schema.rs | **100%** | All 15 definitions |
| Base_XMath | base_xmath.rs | **90%** | Missing: matrix/cases bindings (see below) |
| Base_Functions | base_functions.rs | **95%** | Core constructor logic |
| plain.tex | plain.rs | **95%** | Missing: `\beginsection`, `\lx@centerline/leftline/rightline` |
| LaTeX bootstrap | latex.rs | **100%** | All 10 definitions |

### TeX_Math — Missing Definitions (HIGH PRIORITY)

**Math atom adjusters (8 definitions — needed for semantic math markup):**
- `\mathrel`, `\mathbin`, `\mathord`, `\mathop`, `\mathopen`, `\mathclose`, `\mathpunct`, `\mathinner`
- These wrap content and set `role` attribute for math parsing

**Floating scripts (4 definitions):**
- `\lx@floating@superscript`, `\lx@floating@subscript`
- `\lx@post@superscript`, `\lx@post@subscript`

**Delimiter handling (3 definitions):**
- `\left TeXDelimiter`, `\lx@left`, `\lx@right`
- `\lx@delimiterdot`

### Base_XMath — Missing/Deferred Definitions

**Commented out in base_xmath.rs:**
- `\lx@cases@condition`, `\lx@cases@end@condition` — cases environment conditions
- `\lx@gen@cases@bindings`, `\lx@gen@matrix@bindings` — complex binding machinery
- `\lx@ams@matrix@` — AMS matrix variant constructor
- `compact_xmdual()` — document.rs no-op stub
- DefRewrite for mixed fractions — completely missing

**Sized delimiters (`latex_ch7_math_common_delimiters.rs` is EMPTY):**
- `\big`, `\Big`, `\bigg`, `\Bigg` + l/m/r variants (19 total)

---

## latex_constructs.pool.ltxml — Missing Definitions (~101 unported of ~957)

### Bibliography/Citations (HIGH IMPACT — blocks 3+ structure .todo tests)
| CS Name | Type | Perl Line | Notes |
|---------|------|-----------|-------|
| `\@cite` | DefConstructor | ~4237 | Core citation rendering |
| `\@@cite` | DefConstructor | ~4245 | Internal cite variant |
| `\nocite` | DefPrimitive | ~4260 | No-cite marker |
| `\bibcite` | DefPrimitive | ~4270 | BibTeX cite entry |
| `\citation` | DefMacro | ~4250 | Citation macro |
| `\bibdata` | DefMacro | ~4275 | BibTeX data |
| `\lx@bibliography` | DefConstructor | ~4280 | Bibliography environment |
| `\lx@mark@nocite` | DefPrimitive | ~4265 | No-cite tracking |
| `\restoring@bibitem` | DefConstructor | ~4290 | Bib item restoration |

### Equations/Alignment (MEDIUM IMPACT)
| CS Name | Type | Perl Line | Notes |
|---------|------|-----------|-------|
| `\@eqnarray@bindings` | DefPrimitive | ~2250 | Eqnarray alignment setup |
| `\@@eqnarray` | DefConstructor | ~2260 | Core eqnarray env |
| `\eqnarray*` | DefConstructor | ~2280 | Starred variant |
| `\@eqnarray@label` | DefMacro | ~2300 | Label in eqnarray |
| `\lx@eqnarray@save@label` | DefPrimitive | ~2310 | Label saving |
| `\eqnarray@row@before` | DefPrimitive | ~2270 | Row hooks |
| `\eqnarray@row@after` | DefPrimitive | ~2275 | Row hooks |

### Caption/Float (MEDIUM-HIGH IMPACT — blocks figure/table .todo tests)
| CS Name | Type | Perl Line | Notes |
|---------|------|-----------|-------|
| `\@caption` | DefMacro | ~1015 | Caption chain entry |
| `\@caption@` | DefConstructor | ~1020 | Caption constructor |
| `\@caption@@@` | DefPrimitive | ~1025 | Caption finalization |
| `\caption` | DefMacro | ~1010 | User-facing caption |
| `\@caption@postlabel` | DefMacro | ~1030 | Post-label hook |
| `\@@toccaption` | DefConstructor | ~1035 | TOC caption entry |
| `\format@title@figure` | DefMacro | ~1040 | Figure title format |
| `\format@title@table` | DefMacro | ~1045 | Table title format |

### Float Placement System (MEDIUM IMPACT)
| CS Name | Type | Perl Line | Notes |
|---------|------|-----------|-------|
| `\@topnewpage` | DefMacro | ~1050 | Float at top of page |
| `\@freelist` | DefMacro | ~1055 | Free float list |
| `\@toplist` | DefMacro | ~1060 | Top float list |
| `\@botlist` | DefMacro | ~1065 | Bottom float list |
| `\@midlist` | DefMacro | ~1070 | Mid-page float list |

### Sectioning/TOC (HIGH IMPACT — blocks structure .todo tests)
| CS Name | Type | Perl Line | Notes |
|---------|------|-----------|-------|
| `\addcontentsline` | DefPrimitive | ~3800 | Add TOC entry |
| `\contentsline` | DefPrimitive | ~3810 | TOC line rendering |
| `\tableofcontents` | DefConstructor | ~3820 | TOC generation |
| `\listoffigures` | DefConstructor | ~3830 | LOF generation |
| `\listoftables` | DefConstructor | ~3840 | LOT generation |

### Font Declarations (MEDIUM IMPACT)
| CS Name | Type | Notes |
|---------|------|-------|
| `\DeclareSymbolFont` | DefPrimitive | Symbol font registration |
| `\DeclareSymbolFontAlphabet` | DefPrimitive | Symbol font alphabet |
| `\DeclareTextFontCommand` | DefPrimitive | Text font command factory |
| `\DeclareOldFontCommand` | DefPrimitive | Old NFSS compatibility |
| `\bfseries` | DefPrimitive | Bold series switch |
| `\mdseries` | DefPrimitive | Medium series switch |

### Input/File Handling (LOW-MEDIUM IMPACT)
| CS Name | Type | Notes |
|---------|------|-------|
| `\@input` | DefPrimitive | Internal file input |
| `\@input@` | DefPrimitive | File input variant |
| `\InputIfFileExists` | DefPrimitive | Conditional file input |

### Picture Environment (~25 defs — LOW IMPACT)
- `\circle`, `\oval`, `\line`, `\vector`, `\put`, `\multiput`, `\qbezier`, `\bezier`
- `\dashbox`, `\thicklines`, `\thinlines`, `\arrowlength`, `\linethickness`
- Various `\pic@*` box variants

### Index/Glossary (~11 defs — LOW IMPACT)
- `\glossary`, `\@index`, `\@indexphrase`, `\@indexsee`, `\@indexseealso`
- `\index@done`, `\index@item`, `\index@subitem`, `\index@subsubitem`

---

## Package Binding Coverage

### Summary Table

| Package | Perl Defs | Rust Defs | Coverage | Test Impact |
|---------|-----------|-----------|----------|-------------|
| color.sty | 148 | ~130 | **~90%** | **JUST PORTED** |
| xkeyval.sty | 674 | 930 | **100%+** | Best coverage |
| article.cls | 120 | 118 | **98%** | Near-complete |
| amsthm.sty | 197 | 230 | **~90%** | Good |
| hyperref.sty | 90 | 92 | **~65%** | Key features missing |
| graphicx.sty | 75 | 52 | **~70%** | Sizer/properties stubs |
| amsmath.sty | 162 | 17 | **~10%** | CRITICAL gap |
| natbib.sty | 674 | 57 | **~8%** | CRITICAL gap |

### Tier 1 — Critical Missing (high-frequency packages)
| Package | Perl Lines | Status | Test Impact |
|---------|-----------|--------|-------------|
| xcolor.sty | 1200 | **NOT PORTED** | Blocks abxtest |
| array.sty | 650 | **NOT PORTED** | Enhanced tables |
| geometry.sty | 300 | **NOT PORTED** | Page layout |
| babel.sty | 400 | **NOT PORTED** | Blocks babel test |
| keyval.sty | 120 | **NOT PORTED** | Upstream of graphicx |

### Tier 2 — Important Missing
| Package | Perl Lines | Status |
|---------|-----------|--------|
| soul.sty | 180 | NOT PORTED |
| stmaryrd.sty | 290 | NOT PORTED |
| wasysym.sty | 200 | NOT PORTED |
| pifont.sty | 140 | NOT PORTED |
| accents.sty | 50 | NOT PORTED |
| mathabx.sty | 600 | NOT PORTED |
| amscd.sty | 130 | NOT PORTED |

### Tier 3 — Ported but Critically Incomplete

**amsmath.sty (10% coverage — 145 definitions missing)**
Missing major features:
- ALL alignment environments: `\align`, `\align*`, `\alignat`, `\flalign`, `\xalignat`, `\xxalignat`
- ALL gather environments: `\gather`, `\gather*`, `\gathered`
- `\split`, `\multline`, `\multline*`
- `\aligned`, `\alignedat`
- `\cases`, `\boxed`
- Matrix environments: `\matrix`, `\pmatrix`, `\bmatrix`, `\Bmatrix`, `\vmatrix`, `\Vmatrix`
- `\subequations`, `\intertext`, `\eqref`
- `\smash`, `\cfrac`, `\genfrac`
- ALL alignment binding primitives: `\lx@ams@align@bindings`, `\lx@ams@gather@bindings`, etc.
- DeclareOption: `reqno`, `leqno`, `fleqn`

**natbib.sty (8% coverage — ~600 definitions missing)**
Missing major features:
- Complex `\cite` macro with style-dependent logic
- `\citet`, `\citep`, `\citealt`, `\citealp` with multiple arguments
- `\citeauthor`, `\citeyear` with complex Perl logic
- `setCitationStyle()` function calls (all stubbed)
- Options with `setCitationStyle()` callbacks

**hyperref.sty (65% coverage — key features missing)**
Missing:
- `\hypersetup` — incomplete (KeyVals handling stubbed)
- `\hyperref@@ii`, `\hyperref@@iv` — constructor variants
- `\hyperlink{}{}`, `\hypertarget{}{}`
- PDF form fields: `\TextField`, `\CheckBox`, `\ChoiceMenu`, `\PushButton`
- Metadata RDFa support

**graphicx.sty (70% coverage — sizer callbacks stubbed)**
Missing:
- `\includegraphics` actual multi-phase logic
- `image_candidates()` function
- `graphicX_options()` function
- Complex `sizer` and `properties` callbacks

---

## Ignored Tests — Root Causes

### Font tests (16 ignored)
| Test | Packages Needed | Root Cause |
|------|----------------|------------|
| abxtest | mathabx, xcolor | Missing packages |
| acc | accents | `\mathgroup` undefined, alignment crash |
| bbold | bbold (have binding) | 676 diffs — table + math structure |
| cancels | ~~color~~, ~~cancel~~ | **Both ported** — 14 diffs: `_force_font`/`_font` property eliding `<text>` wrapper |
| ding | pifont | Missing pifont (pzd font map) |
| esint | esint (have binding) | Math parser `todo!()` panic |
| fonts | (article only) | 999 diffs — massive font table |
| mathaccents | (article only) | 403 diffs — math structure |
| mathbbol | mathbbol (have binding) | 109 diffs — math structure |
| mathcolor | ~~color~~, amsmath | **color.sty NOW PORTED** — needs `\ExplSyntaxOn` (LaTeX3) |
| mixed | (article only) | 36 diffs — math parser XMDual |
| plainfonts | (plain TeX) | 73 diffs — font tables |
| sizes | (article only) | 376 diffs — many sizing |
| soul | soul, ~~color~~ | **color.sty NOW PORTED** — needs soul.sty |
| stmaryrd | stmaryrd | Missing stmaryrd package |
| wasysym | ~~color~~, wasysym | **color.sty NOW PORTED** — needs wasysym.sty |

### Keyval tests (3 ignored)
| Test | Root Cause |
|------|------------|
| keyvalstyle | xkeyval style environments |
| xkeyvalstyle | xkeyval view/style system |
| xkeyvalview | xkeyval view handling |

### Other ignored (9)
| Test | Root Cause |
|------|------------|
| can_mathl | Math parser |
| can_namespace | Needs .latexml document-level bindings |
| halign_test | Missing `ltx_nopad_r`, nested tabular bracket |
| tabtab_test | Nested tabular missing, column misalignment |
| can_theorem (ams) | Math parser diffs in amsmath environments |
| can_graphics | Missing graphics features |
| can_parse | Math parser |
| can_complex | Complex document features |
| can_babel | Missing babel package |

---

## Core Infrastructure Gaps

### todo!() Inventory (latexml_core: 67)
- `register.rs` (16): Token/Tokens RegisterValue arithmetic
- `lib.rs` (7): BoxOps trait defaults
- `argument.rs` (7): AlignmentTemplate/RegisterDefinition edge cases
- `digested.rs` (6): Unhandled DigestedData variants
- `expandable.rs` (5): Profiling hooks
- `definition.rs` (4): Register trait stubs
- `alignment.rs` (4): compute_size, get_font, get_string, be_absorbed
- `rewrite.rs` (3): Pattern matching edge cases
- `keyvals.rs` (3): set_property, compute_size, set_keys_expansion variant
- Other (12): scattered across tokens, stomach, state, etc.

### Deferred Items
- `compact_xmdual()` body — document.rs
- `mergeAttributes()` — document.rs
- `\fontname` full format ("select font X at Ypt")
- `\font` primitive (metrics/at/scaled)
- `\fontdimen` getter/setter (hardcoded stubs)
- `\leaders`/`\cleaders`/`\xleaders` (stub no-ops)
- `\lx@special@graphics` constructor
- BibTeX entry processing

---

## Perl Commit Sync Status

| Commit | Description | Synced? |
|--------|-------------|---------|
| 7119a535 | do not double-escape spec for dumped parameters | N/A (Dumper.pm) |
| acaab773 | Correct Grouplevel | YES |
| 5082b034 | kernel upgrades for CI in texlive 2025 | YES |
| 3a89a24d | Lrgroup | YES |
| d81e955b | add \overunderset to amsmath.sty | YES |
| e577cbd3 | marvosym binding typo | YES |
| 3875cd64 | bibconfig option for latexml.sty | YES |
| e6db0871 | add missing refactor for textunderscore | N/A (underscore.sty not ported) |

---

## Recommended Work Order

### Phase 1 — Enable structure .todo tests (highest ROI)
1. ~~Port `color.sty` binding~~ **DONE**
2. Implement `\eqnarray` environment (latex_constructs ~L2250)
3. Implement `\addcontentsline`/`\tableofcontents` (latex_constructs ~L3800)
4. Complete `\caption` chain (latex_constructs ~L1010)
5. Port bibliography basics (`\bibcite`, `\@cite`, `\nocite`)
6. Implement `\bfseries`/`\mdseries` font series commands

### Phase 2 — Complete amsmath for AMS tests
1. Port remaining ~90% of amsmath.sty.ltxml (alignment envs, matrices, cases)
2. Implement `\big`/`\Big`/`\bigg`/`\Bigg` delimiters (19 definitions)
3. Implement Base_XMath commented constructors (matrix/cases bindings)
4. Port math atom adjusters: `\mathrel`, `\mathbin`, `\mathord`, `\mathop`, etc.
5. Complete `compact_xmdual()`

### Phase 3 — Enable more font tests
1. Port `cancel.sty` (80 lines — unblocks cancels test with color.sty done)
2. Port `soul.sty` (180 lines — unblocks soul test with color.sty done)
3. Port `pifont.sty` (140 lines)
4. Port `stmaryrd.sty`, `wasysym.sty`, `accents.sty`
5. Fix alignment padding for font table tests

### Phase 4 — Broader coverage
1. Port `array.sty` (650 lines), `geometry.sty` (300 lines)
2. Port `babel.sty` basics (400 lines)
3. Implement picture environment (~25 defs)
4. Port `keyval.sty` (120 lines)
5. Complete natbib.sty citation logic

### Phase 5 — Polish
1. Eliminate todo!() in latexml_core (67 items)
2. Complete `\font` primitive
3. Port BibTeX.pool basics
4. Address remaining alignment issues
5. Complete hyperref.sty (forms, metadata, hypersetup)
