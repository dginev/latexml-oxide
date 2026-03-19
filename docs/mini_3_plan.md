# Mini Plan: Round 8 (revised)

## Status: 224 pass, 0 fail, 95 ignored

## Three most connected work packets

### Packet 1: Fix \lx@ams@matrix@ reversion (matrix_test, cd_test)
- **Root cause**: `\lx@ams@matrix@` tex= shows internal CS name instead of `\begin{matrix}...\end{matrix}`
- **Fix**: Add reversion sub (same pattern as \lx@gen@plain@matrix@ fix from Round 7)
- **Impact**: Fixes tex= for matrix_test, cd_test, and potentially other alignment tests

### Packet 2: Fix cd_test fontsize 16% vs 160%
- **Root cause**: Arrow font scaling factor wrong (probably /10 instead of *10 or similar)
- **Fix**: Find and fix the scaling calculation in amscd_sty.rs
- **Impact**: Reduces cd_test diffs significantly

### Packet 3: Add \sideset macro definition (sideset_test)
- **Root cause**: `\sideset` produces ERROR class="undefined"
- **Fix**: Port \sideset from Perl amsmath.sty.ltxml
- **Impact**: Unlocks sideset_test (currently entirely broken)

### Expected outcome
- matrix_test closer to passing (reversion fix)
- cd_test diffs reduced
- sideset_test partially working
