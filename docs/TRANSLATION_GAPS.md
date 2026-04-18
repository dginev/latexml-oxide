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

- [x] `scriptHandler()` — `tex_math.rs::script_handler` L71
- [x] `cleanup_Math()` / `cleanup_XMText()` — `base_utilities.rs::cleanup_math` L1415, `cleanup_xmtext` L1513, `cleanup_xmtext_outer` L1506
- [x] FLOATING script dispatch — `tex_math.rs` L43 (inline script-position detection rather than a DefRewrite chain, but covers the same cases)
- [x] `adjustMathStyle_internal()` + `%mathstyle_adjust_map` — `tex_math.rs::mathstyle_adjust` L1854, `adjust_mathstyle_internal` L1877
- [ ] `fracSizer()` (Perl L1054-1059, 6 lines) — **still missing**; used by `\over`/`\atop` to size vertical fractions. Low impact on current tests.
- [x] `scriptSizer()` — `tex_math.rs::script_sizer` L226
- [x] `revertScript()` — `tex_math.rs::revert_script` L210 (callers at L1507, 1519, 1531, 1541)
- [x] `DefMathLigature` — macro at `prelude/setup_binding_language.rs::DefMathLigature` L186
- [x] `\skewchar` — `tex_math.rs` L1395 `DefRegister!("\\skewchar{}", ...)`

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

- [x] Counter system — `counter/dialect.rs::new_counter` L53, `step_counter` L286, `maybe_preempt_refnum` L413 (RefStepCounter equivalent). Substantively ported.
- [x] String cleaning — `common/cleaners.rs::clean_id` L58, `clean_label` L77, `clean_class_name` L98.
- [x] Register codes — `state.rs::lookup_mathcode` L1549, `assign_mathcode` L1569.
- [x] `DefConditional` family — macro at `prelude/setup_binding_language.rs::DefConditional` L237 covers both `DefConditional` and `DefConditionalI` use cases; `set_condition` at `content.rs` L1562.
- [x] `RawTeX` — `stomach.rs::raw_tex` L725.

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
