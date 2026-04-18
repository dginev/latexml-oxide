# Translation Gaps: Perl → Rust Function-Level Comparison

Generated 2026-03-15. Each `[ ]` marks an under-translated function. `[x]` when updated.

## 1. Font.pm — compute_boxes_size helpers (DONE)

Perl `computeBoxesSize` dispatches to 4 helper functions (159 lines total).
Rust now has full implementation matching Perl semantics.

- [x] `computeBoxesSize_box` (Perl L683-702, 20 lines) — single box sizing with vattach
- [x] `computeBoxesSize_words` (Perl L705-746, 42 lines) — paragraph line-breaking layout
- [x] `computeBoxesSize_lines` (Perl L749-766, 18 lines) — multi-line stacking with linewidth
- [x] `computeBoxesSize_stack` (Perl L769-801, 33 lines) — vertical stacking for vbox

## 2. math_common.pool.ltxml (803 lines Perl → ~1500 lines Rust in plain.rs)

Substantially ported (278 DefMath! definitions). Most core math symbols present.

- [x] Greek letters — DefMathI for α-ω, Γ-Ω
- [x] Misc math symbols — \aleph through \clubsuit
- [x] Variable-sized operators — \sum, \prod, \int, etc. with scriptpos
- [x] Binary operations — \pm through \amalg
- [x] Relation symbols — \leq through \propto
- [x] Arrow symbols
- [x] Ellipsis/dots — \ldots, \cdots, \vdots, \ddots, \dots
- [x] Math accents — \hat through \widetilde
- [x] Phantom/strut/smash — \phantom, \hphantom, \vphantom (with sizing afterDigest)
- [x] Roots — \sqrt, \root
- [x] Log-like functions — \log, \sin, \lim, etc.
- [x] Active prime — `\active@math@prime` at math_common.rs L397, `Let!("'", ...)` at L422
- [x] `\not` negation — ported as simplified Unicode `U+FF0F` (math_common.rs L598); full Perl DefRewrite chain over `\not\X` pairs not ported but the common case works
- [x] `\joinrel` relation combining — math_common.rs L667-695 (primitive + `@@joinrel` whatsit)
- [x] Delimiters — `\big`/`\Big`/`\bigg`/`\Bigg` sized variants at math_common.rs L894-903 (and `\bigl`/`\biggl` family at L922-942)
- [x] Modulo — `\pmod` / `\bmod` at math_common.rs L1108-1109
- [ ] XMWrap cleanup rewrite (Perl L769-798, 30 lines) — deliberately commented out in tex.rs L174; keeping unchecked as a reminder of the intentional divergence

## 3. TeX_Math.pool.ltxml — missing functions

- [ ] `scriptHandler()` — T_SUPER/T_SUB handling (Perl L353-426, 75 lines)
- [ ] `cleanup_Math()` / `cleanup_XMText()` (Perl L190-311, 122 lines)
- [ ] FLOATING script DefRewrite (Perl L527-551, 25 lines)
- [ ] `adjustMathStyle_internal()` + %mathstyle_adjust_map (Perl L1024-1052, 29 lines)
- [ ] `fracSizer()` (Perl L1054-1059, 6 lines)
- [ ] `scriptSizer()` (Perl L460-493, 34 lines)
- [ ] `revertScript()` (Perl L438-450, 13 lines)
- [ ] DefMathLigature rules (Perl L1271-1274)
- [ ] \skewchar register getter/setter (Perl L1202-1211, 10 lines)

## 4. Rewrite.pm (560 Perl → 350 Rust, mostly stubs)

- [ ] `compileClause()` — 6 clause types (Perl L284-331, 48 lines)
- [ ] `compile_match()` + `compile_match1()` (Perl L334-372, 39 lines)
- [ ] `compile_replacement()` (Perl L375-391, 17 lines)
- [ ] `compile_regexp()` (Perl L393-401, 9 lines)
- [ ] `domToXPath()` + `domToXPath_rec()` + `domToXPath_seq()` (Perl L416-532, 117 lines)
- [ ] `applyClause()` — operators beyond Select/Replace (Perl L78-181, 104 lines)
- [ ] Wildcard handling — markSeen, markWildcards, set_wildcard_ids (Perl L219-282, 64 lines)
- [ ] `setAttributes_encapsulate()` + `setAttributes_wild()` (Perl L184-231, 48 lines)

## 5. Package.pm — missing exported functions

- [ ] Counter system — NewCounter, StepCounter, RefStepCounter, etc. (Perl L654-949, ~200 lines)
- [ ] String cleaning — CleanID, CleanLabel, CleanClassName, etc. (Perl L482-585, ~100 lines)
- [ ] Register codes — LookupMathcode, AssignMathcode, etc. (Perl L~50 lines)
- [ ] DefConditionalI + IfCondition + SetCondition (Perl L1189-1279, ~90 lines)
- [ ] RawTeX (Perl L976-995, 20 lines)

## 6. Box.pm — missing methods

- [ ] `isMath()` for Tbox (Perl L79-81, 3 lines)
- [ ] `setProperties()` batch setter (Perl L171-175, 5 lines)
- [ ] `getTotalHeight()` (Perl L202-210, 9 lines)

## 7. Number.pm — missing methods

- [ ] `getStretch()`, `getShrink()`, `getStretchOrder()`, `getShrinkOrder()` stubs (Perl L125-128, 4 lines)

## 8. Color.pm — gaps (693 Perl → 339 Rust)

- [ ] Full color model support (CMYK, HSB conversions)
- [ ] Color mixing operations

## 9. pdfTeX.pool.ltxml — gaps (284 Perl → 254 Rust)

- [ ] Check for any missing register/primitive definitions

## Priority Order (by test impact)

1. **math_common** (Section 2) — blocks most math tests
2. **compute_boxes_size helpers** (Section 1) — blocks sizes_test
3. **TeX_Math scriptHandler** (Section 3) — blocks superscript/subscript tests
4. **Package.pm counters** (Section 5) — blocks structured documents
5. **Rewrite.pm** (Section 4) — blocks \not and XMWrap cleanup
