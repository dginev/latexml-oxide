# Mini Plan: Round 2

## Three most connected work packets

### Selection rationale
The **missing `tex=` attribute on Math elements inside MathFork** affects most Tier 2 tests (badeqnarray, eqnums, amsdisplay, matrix, sideset). Fixing this single issue could reduce diff counts across 5+ tests. The root cause is in the `add_body_TeX` afterClose hook not finding a `node_box` for Math elements that are inside equation arrays. Connected: the xml:id numbering (`.m1` vs `.m4`) is also a MathFork issue.

### Packet 1: Fix `tex=` attribute on MathFork Math elements
- **Problem**: `ltx:Math` afterClose hook at tex_math.rs:188 gets `node_box` to call `body.untex()` for the `tex` attribute. Inside MathFork (equation arrays), Math elements don't have their `node_box` set.
- **Debug approach**:
  1. Create minimal TeX: `\begin{eqnarray} a &=& b \end{eqnarray}`
  2. Compare Perl vs Rust: check if Perl's `add_body_TeX` fires for MathFork Math, when it fires, and what box it gets
  3. Add debug prints in the afterClose hook to see what `node_box` returns
- **Fix**: Ensure MathFork Math elements have their `node_box` set during rearrangement.

### Packet 2: Fix xml:id numbering for MathFork Math (`.m1` vs `.m4`)
- **Problem**: In Perl, equation array Math elements get id suffix `.m4` (after 3 alignment-related math elements). In Rust, they get `.m1`.
- **Root cause**: The `generate_id` counter for Math inside equations doesn't account for alignment-related Math elements that Perl creates.
- **Connected**: Same MathFork rearrangement code path.

### Packet 3: badeqnarray_test math tree (Ex10)
- **Problem**: Ex10 has flat XMath tree where Perl has nested XMApp. Math parser difference for `=e+f+g` without leading term.
- **Connected**: Same test file, can be investigated together.

### Expected outcome
- badeqnarray_test: reduce from 158 to <20 diffs
- Potentially unblock other MathFork tests
