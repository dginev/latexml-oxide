# Translation Gaps: Perl → Rust Function-Level Comparison

> **Status (2026-04-26): SUBSTANTIALLY RESOLVED.** This document
> snapshotted the state on 2026-03-15. All major sections are now
> closed; the only remaining `[ ]` items are intentional Perl
> divergences or comment-only Perl entries (see "Priority Order"
> at the bottom). Future translation-gap audits should be performed
> with a fresh Perl↔Rust diff, not by extending this file.
>
> Current priority is tracked in [`SYNC_STATUS.md`](../SYNC_STATUS.md)'s
> dashboard.

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
- [x] `fracSizer()` — `tex_math.rs::frac_sizer` L235; wired as `sizer` on `\lx@generalized@over` so `\over`/`\atop`/`\above*` variants get TeX-faithful width/height/depth.
- [x] `scriptSizer()` — `tex_math.rs::script_sizer` L226
- [x] `revertScript()` — `tex_math.rs::revert_script` L210 (callers at L1507, 1519, 1531, 1541)
- [x] `DefMathLigature` — macro at `prelude/setup_binding_language.rs::DefMathLigature` L186
- [x] `\skewchar` — `tex_math.rs` L1395 `DefRegister!("\\skewchar{}", ...)`

## 4. Rewrite.pm — substantively ported

(The "560 Perl → 350 Rust, mostly stubs" header from 2026-03-15 is now
inaccurate — `rewrite.rs` is ~1200 lines of functioning code.)

- [x] `compileClause()` — `rewrite.rs::compile_clause` L225 (handles all 6 clause types inline rather than dispatching to compile_match/compile_replacement helpers; the structural equivalent)
- [x] `compile_match*` / `compile_replacement` / `compile_regexp` — absorbed into `compile_clause`'s arm dispatch; `CompiledMatch` type at L724
- [x] `domToXPath*` — `rewrite.rs::dom_to_xpath` L728, `dom_to_xpath_rec` L742, `dom_to_xpath_seq` L859
- [x] `applyClause()` with all operators — `rewrite.rs::apply_clause` L379
- [x] Wildcard handling — `mark_seen` L1186, `mark_wildcards` L934, `unmark_wildcards` L963, `set_wildcard_ids` L979
- [x] `setAttributes_wild()` — `rewrite.rs::set_attributes_wild` L1023 (setAttributes_encapsulate use cases folded into the same function)

## 5. Package.pm — missing exported functions

- [x] Counter system — `counter/dialect.rs::new_counter` L53, `step_counter` L286, `maybe_preempt_refnum` L413 (RefStepCounter equivalent). Substantively ported.
- [x] String cleaning — `common/cleaners.rs::clean_id` L58, `clean_label` L77, `clean_class_name` L98.
- [x] Register codes — `state.rs::lookup_mathcode` L1549, `assign_mathcode` L1569.
- [x] `DefConditional` family — macro at `prelude/setup_binding_language.rs::DefConditional` L237 covers both `DefConditional` and `DefConditionalI` use cases; `set_condition` at `content.rs` L1562.
- [x] `RawTeX` — `stomach.rs::raw_tex` L725.

## 6. Box.pm — ported

- [x] `isMath()` — `tbox.rs::is_math` (checks `mode` property against `MATH_SYM`)
- [x] `setProperties()` batch setter — `tbox.rs::set_properties`
- [x] `getTotalHeight()` — `tbox.rs::total_height` (sums height + depth)

## 7. Number.pm — N/A

Perl's `Number.pm` has stub methods `getStretch`/`getShrink`/`getStretchOrder`/
`getShrinkOrder` that always return 0 — polymorphism shims for when a
`Number` is used where a `Glue` is expected. Rust's type system makes this
unnecessary: `Number` is `i64`-backed with no stretch/shrink fields, and
the type system prevents calling glue-specific accessors on a number.
`common/glue.rs::Glue { skip, plus, minus, pfill, mfill }` carries the
stretch/shrink data as public fields (L106-109); direct field access
replaces the Perl getters. No port needed.

## 8. Color.pm — substantively ported

(The "693 Perl → 339 Rust" header from 2026-03-15 is stale.
`common/color.rs` has all core models and conversions.)

- [x] Full color model support — `common/color.rs::to_cmyk` L110, `to_hsb` L130, `convert` L172 (dispatches across rgb, cmy, cmyk, hsb, gray, HTML, RGB, Hsb, HSB, Gray, tHsb, wave)
- [x] Color mixing operations — `common/color.rs::mix` L235 (linear interpolation across color models); `xcolor_sty.rs::apply_mix_expr` L232 (parses !pct!name chains)

## 9. pdfTeX.pool.ltxml — 20 of 138 primitives still missing

Audit (2026-04-18, refined): 118 of 138 Perl CSes have Rust
equivalents. Of the other 20, most were "Perl comment only" (documented
but not actually defined in pdfTeX.pool.ltxml) or defined in a
different engine file:

  [x] \lpfcode, \rpfcode                  — pdftex.rs DefRegister
  [x] \pdfsavepos                         — pdftex.rs no-op stub (Perl: comment only)
  [x] \pdfstartthread, \pdfendthread      — pdftex.rs no-op stubs (Perl: comment only)
  [x] \pdfnoligatures                     — pdftex.rs stub (Perl: comment only)
  [x] \pdfsetrandomseed                   — pdftex.rs stub (Perl: comment only)
  [x] \special                            — tex_file_io.rs L210 (not in pdftex pool either)
  [x] \vadjust                            — tex_paragraph.rs L23 (not in pdftex pool either)

Perl-comment-only (documented but not actually Perl-defined — N/A for
pure parity):

  \pdfdest, \pdfthread, \pdfoutline       — all comment-only at Perl L179-184
  \pdfximage, \pdfrefximage               — comment-only at Perl L144-145
  \pdfcolorstackinit                      — commented-out DefMacro at Perl L125
  \pdffontattr, \pdffontexpand            — comment-only at Perl L194-195

Genuinely Perl-defined but Rust-missing — now 0 items remaining:

  [x] \pdfannot                           — pdftex.rs DefPrimitive + OpenAnnotSpecification parameter type
  [x] \pdfobj                             — pdftex.rs DefPrimitive (shares OpenAnnotSpecification)
  [x] \pdfcolorstack                      — pdftex.rs DefPrimitive with 4 OptionalMatch flags; consumes GeneralText unless action is `pop`

Section 9 is fully resolved for the Perl-defined set. The
comment-only items (L179-184, L125, L194-195 etc.) remain
intentionally undefined — Perl documents but doesn't define them.

## Priority Order (by test impact) — UPDATED 2026-04-18

The 2026-03-15 priority list is largely resolved. Current summary:

1. ~~math_common (Section 2)~~ — substantively ported (commit a0d8848 marked
   5/6 items; only `XMWrap cleanup rewrite` remains, deliberately skipped).
2. ~~compute_boxes_size helpers (Section 1)~~ — ported (already [x]).
3. ~~TeX_Math scriptHandler (Section 3)~~ — ported in full (all 9 items
   marked `[x]`, including `fracSizer` at `tex_math.rs::frac_sizer` L242
   wired via `sizer` on `\lx@generalized@over`).
4. ~~Package.pm counters (Section 5)~~ — ported (commit c1ee42b marked 5/5).
5. ~~Rewrite.pm (Section 4)~~ — ported (commit 921f179 marked all 8 items).

**Remaining gaps worth tracking:**

- XMWrap cleanup rewrite (Section 2) — deliberately commented out.
- pdfTeX stubs (Section 9) — comment-only in Perl source, intentionally
  undefined in Rust for parity.

All current [ ] items are either intentional divergences or low-priority
stubs. The significant translation-gap work is done; future items
should be added with fresh Perl↔Rust diff audits rather than the
2026-03-15 snapshot this file was born from.
