# Mini Plan: Round 7

## Status: 221 pass, 0 fail, 98 ignored

## Analysis
Three tests each have exactly 1 diff line — the lowest-hanging fruit in the entire suite.

## Three most connected work packets

### Packet 1: Fix array_math_test (1 diff)
- **Diff**: `tex=` shows `\lx@gen@plain@matrix@{name,datameaning}{...}` instead of `\matrix{...}`
- **Root cause**: Matrix reversion not generating proper `\matrix` command
- **Files**: Likely `base_xmath.rs` or `plain.rs` (matrix reversion)

### Packet 2: Fix parser_speculate_test (1 diff)
- **Diff**: Missing `possibleFunction="yes"` attribute on XMTok
- **Root cause**: Math parser not setting possibleFunction attribute
- **Files**: `latexml_math_parser/` — grammar or semantic rules

### Packet 3: Fix prescripted_test (1 diff)
- **Diff**: `scriptpos="pre1"` vs `scriptpos="pre2"` — off-by-one index
- **Root cause**: Prescripted script position indexing
- **Files**: `base_xmath.rs` or math parser prescripted handling

### Expected outcome
- +3 passing tests → 224 pass, 0 fail, 95 ignored
