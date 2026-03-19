# Mini Plan: Round 11

## Status: 227 pass, 0 fail, 92 ignored

## Completed (Round 10)
- \lxDeclare fast-path: init_flag fix, DefKeyVal family, apply_lx_declarations
- Grammar: trigfunction factor, compound_operator, APPLYOP rules
- nested_application_test passing

## Three most connected work packets

### Packet 1: Sync Perl expected XMLs for parse tests & un-ignore close ones
- **Impact**: Several parse tests may be close to passing with grammar improvements
- **Approach**: Check function_argument_syntax (55 diffs), sequences_and_lists (86),
  sets (86), algebraic_terms (119) — update expected XMLs where Rust parses correctly
- **Files**: tests/parse/*.xml, 70_parse.rs

### Packet 2: Tier 1 work plan items — sizes_test and eqnums_test
- **Impact**: sizes_test has ~20 diffs remaining, eqnums_test 416 diffs
- **Approach**: Fix remaining dimension rounding and tabular sizing issues
- **Files**: font.rs, normalize.rs, test XMLs

### Packet 3: Tier 2 — badeqnarray_test (afterConstruct already done)
- **Impact**: afterConstruct rearrangement was already implemented (item 29 done)
- **Approach**: Try un-ignoring badeqnarray, fix remaining diffs
- **Files**: 53_alignment tests

### Expected outcome
- +3-5 passing tests
