# Mini Plan: Round 6

## Status: 217 pass, 0 fail, 62 ignored

## Analysis
Most remaining tests are blocked by math parser text= diffs or package bindings.
The batch suites (40_math, 70_parse) contain 42 individual test files, of which
4 pass perfectly (array_newline_math, compact_dual, testover, standalone_equations).
These are currently locked behind all-or-nothing batch test runners.

## Three most connected work packets

### Packet 1: Split 40_math batch into individual tests
- **Current**: Single test `can_mathl()` runs ALL 14 tests in tests/math/
- **Change**: Create individual test functions for each .tex/.xml pair
- **Benefit**: 3 tests (array_newline_math, compact_dual, testover) will immediately pass
- **Files**: `latexml_oxide/tests/40_math.rs`

### Packet 2: Split 70_parse batch into individual tests
- **Current**: Single test `can_parse()` runs ALL 28 tests in tests/parse/
- **Change**: Create individual test functions, ignore ones that don't pass
- **Benefit**: standalone_equations will immediately pass
- **Files**: `latexml_oxide/tests/70_parse.rs`

### Packet 3: Update memory and plan with new counts
- Mark completed items
- Update test counts
- Record blockers for remaining tests

### Expected outcome
- +4 passing tests (array_newline_math, compact_dual, testover, standalone_equations)
- 221 pass, 0 fail, ~96 ignored (more tests counted individually)
