# Mini Plan: Round 9

## Status: 225 pass, 0 fail, 94 ignored

## Three most connected work packets

### Packet 1: Fix CD arrow fontsize "16%" → "160%" (cd_test)
- **Root cause**: fontsize calculation divides by 10 somewhere, producing "16%" instead of "160%"
- **Fix**: Find the scaling factor in amscd_sty.rs and fix
- **Impact**: Many of cd_test's 177 diffs trace to this

### Packet 2: Fix sizes_test thin/medium/thick spaces + rounding (19 diffs)
- **Root cause**: Space characters not converting to Unicode, dimension rounding, tabular sizing
- **Fix**: Known issue from prior sessions — check space char emission and dimension formatting
- **Impact**: sizes_test very close to passing (19 diffs)

### Packet 3: Investigate subordinate_lists_test VERTBAR→MODIFIEROP (14 diffs)
- **Root cause**: `|` not morphed to MODIFIEROP with meaning="conditional"
- **Fix**: Grammar rule or morphing logic for VERTBAR in conditional context
- **Impact**: subordinate_lists_test (14 diffs)

### Expected outcome
- sizes_test close to passing if rounding/space fixes land
- cd_test diffs reduced significantly
- subordinate_lists_test may pass with VERTBAR morphing
