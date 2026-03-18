# Mini Plan: Round 4

## Status: 217 pass, 0 fail, 62 ignored

## Analysis
Most remaining tests need either:
1. Math parser improvements (text= attr) — affects ~20 tests
2. Package bindings (stmaryrd, mathtools, makecell, etc.) — affects ~10 tests
3. Equation numbering (tags) — affects ~8 tests
4. Specific code fixes (preamble PI, \underaccent, etc.) — scattered

## Three most connected work packets

### Selection rationale
Focus on **equation numbering** which affects 8+ tests (eqnums, badeqnarray, amsarticle,
ieee, ntheorem, amsdisplay, listing, split). The equation counter stepping and `<tags>`
element generation is a single infrastructure piece that unblocks many tests.

### Packet 1: Implement equation counter stepping
- **Problem**: `\refstepcounter{equation}` during eqnarray doesn't increment properly,
  leading to wrong/missing equation numbers in `<tags>` elements.
- **Compare**: Run `latexml --noparse` on a minimal numbered eqnarray in both Perl and Rust.
- **Fix**: Ensure counter stepping works in alignment context.

### Packet 2: Implement `<tags>` element generation for equations
- **Problem**: The `<tags>` element with `<tag>`, `<tag role="refnum">`, `<tag role="typerefnum">`
  is missing from equation/equationgroup output.
- **Compare**: Perl's `@equationgroup@numbering` macro and `stepping` hooks.
- **Fix**: Port the tags generation hooks.

### Packet 3: Fix badeqnarray xml:id numbering (.m1 vs .m4)
- **Problem**: MathFork Math gets xml:id S0.Ex1.m1 instead of S0.Ex1.m4.
- **Root cause**: generate_id counter doesn't account for MathBranch cells.
- **Fix**: Ensure counter is synced after MathFork construction.

### Execution order
1. Minimal test comparison: numbered eqnarray in Perl vs Rust
2. Fix counter stepping → tags elements appear
3. Fix id numbering → MathFork ids match
