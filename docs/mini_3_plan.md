# Mini Plan: Round 1 — COMPLETED

## Results
- **Before**: 215 pass, 0 fail, 64 ignored
- **After**: 216 pass, 0 fail, 63 ignored

### Packet 1: wasysym_test — DONE
- **Root cause found**: `create_xmrefs` in math parser called `generate_id` on XMTok nodes that didn't have xml:id. The `get_attribute("id")` check missed the NS attribute, so EVERY lexeme got a new id. After prune_xmduals collapsed the XMDual structures, the ids remained orphaned on XMTok elements.
- **Fix**: Added `cleanup_unreferenced_xmtok_ids()` to Document, called after finalize. Removes xml:ids from XMTok elements not referenced by any idref. Uses `remove_attribute_ns("id", XML_NS)` for proper namespace-aware removal.

### Packet 2: colors_test — BLOCKED
- **\pagecolor Tbox causes infinite loop**: When `\colorbox{red}{R}` expands to `\hbox{\pagecolor{red}R}`, returning a Tbox from `\pagecolor` creates an infinite absorption loop inside `\hbox`. The `\color` primitive also returns Tbox but works because it's not used inside `\hbox`.
- **Deferred**: Needs investigation into `\hbox` box content absorption path.

### Packet 3: Mark already-passing tests — DONE
- Marked items 10 (ding), 16 (figure_grids), 37 (xcolors), 42 (aliceblog) as [x] done.

---

# Mini Plan: Round 2

## Three most connected work packets

### Selection rationale
Focus on **crash/panic fixes** that could unlock tests with minimal code changes. Tests that crash/panic never produce diff output, so fixing the crash reveals whether the test is close to passing. The cd_test and mathtools_test both crash in the math parser/AMS area and share common infrastructure.

### Packet 1: cd_test (56_ams) — math parser panic in parse_rec
- **Problem**: `parse_rec` panics during tree replacement.
- **Fix**: Debug the panic, likely an unwrap on None or out-of-bounds access.
- **Expected**: Fix panic, see actual diffs to assess if test can pass.

### Packet 2: mathtools_test (56_ams) — MathPrimitive unhandled in is_defined_token
- **Problem**: `is_defined_token` doesn't handle MathPrimitive variant, causing a crash.
- **Fix**: Add MathPrimitive case to `is_defined_token`.
- **Expected**: Fix crash, see actual diffs.

### Packet 3: cells_test (53_alignment) — stack overflow in state.rs
- **Problem**: Recursive state lookup causes stack overflow.
- **Fix**: Debug the recursion, add depth limit or break cycle.
- **Expected**: Fix overflow, see actual diffs.

### Execution order
1. Fix mathtools_test crash (likely simplest — missing match arm)
2. Fix cd_test panic
3. Fix cells_test stack overflow
