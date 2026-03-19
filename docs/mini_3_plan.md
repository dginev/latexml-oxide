# Mini Plan: Round 5

## Status: 217 pass, 0 fail, 62 ignored

## Three most connected work packets

### Selection rationale
The **equation numbering tags** blocker affects 8+ tests and is the single highest-impact
infrastructure fix available. The root cause is that alignment Row properties use
`HashMap<String, String>`, losing `Stored::Digested` tags during conversion. All three
packets below are steps toward fixing this single issue.

### Packet 1: Change Row properties from `HashMap<String, String>` to `SymHashMap<Stored>`
- **Files**: `latexml_core/src/alignment/template.rs` (Template struct),
  `latexml_core/src/alignment.rs` (OpenRowFn, be_absorbed_mut)
- **Change**: Template.properties → `SymHashMap<Stored>`,
  OpenRowFn signature → accepts `SymHashMap<Stored>`,
  after_equation stores tags directly (no .to_string())
- **Impact**: All 4 open_row implementations need updating

### Packet 2: Update open_row callbacks to handle Stored properties
- **Files**: `latex_ch7_math_mode_environments.rs` (eqnarray), `amsmath_sty.rs` (align etc.),
  `base_xmath.rs` (equationgroupJoinCols)
- **Change**: Extract `id` as string from `Stored::String`, extract `tags` as `Stored::Digested`,
  absorb tags into the equation element after opening it

### Packet 3: Verify equation numbering works end-to-end
- **Test**: Run mini eqnarray test, compare with Perl output
- **Verify**: Tags appear on eqnarray equations
- **Un-ignore**: Any tests that now pass

### Execution order
1. Change Template.properties type + OpenRowFn signature
2. Update all open_row callbacks
3. Update after_equation to store Stored directly
4. Test, verify, un-ignore
