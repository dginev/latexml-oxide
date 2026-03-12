# Sync Status — 2026-03-12

**100 pass, 38 fail, 7 ignored** + 18 structure .todo disabled

## Failing Tests (ranked by fix ease)

| Rank | Test(s) | Diffs | Root Cause | Fix |
|------|---------|-------|------------|-----|
| 1 | All 26 encoding (t1, ot1, latin*, cp*, ly1, t2*...) | 23–600 | Tabular `{cc}` → `align="right"` not `"center"` | Fix `extract_alignment_column` fill logic in `tex_tables.rs` |
| 2 | compact_dual | 38 | `compact_xmdual()` unimplemented | Implement in document builder |
| 3 | array | 53 | Math array alignment + features | Alignment fix (Rank 1) + math array |
| 4 | arrows | 78 | `stretchy="true"` mismatch, reversion | Update expected XML or attr emission |
| 5 | choose | 80 | Binomial constructs | Missing choose/binom support |
| 6 | fracs | 89 | Nested mathstyle cascade | Fix InFractionStyle propagation |
| 7 | ambiguous_relations | 109 | Math parser ambiguity | Parser work |
| 8 | niceunits, simplemath, testscripts | 127–164 | Various math | Multiple issues |
| 9 | not | 229 | `\not` negation | Negation operator handling |
| 10 | sampler | 1012 | Comprehensive math | Many missing pieces |
| 11 | declare | 0 (crash) | Math parser `_WildCard_` | `semantics/tree.rs` unimplemented |

## Structure .todo (18 disabled, need .todo→.tex + cargo clean)

| Test | Diffs | Blocker |
|------|-------|---------|
| filelist, options | 6 each | Local `.ltxml` discovery (content.rs:945-956 commented out) |
| svabstract | 22 | Missing `svjour` class |
| floatnames | 25 | Missing `float`/`newfloat` packages |
| bibsect | 27 | Missing `bibunits` package |
| subcaption | 37 | Missing `caption`/`subcaption` packages |
| glossary | 66 | Missing `glossaries` package |
| figures | 88 | Figure environment improvements |
| natbib | 91 | Missing `natbib` package |
| eqnums | 97 | Equation numbering in amsmath |
| IEEE | 111 | Missing `IEEEtran` class |
| crazybib | 115 | natbib + inputenc |
| acro | 203 | Missing `acronym` package |
| csquotes | >100 err | Missing `csquotes` package |
| paralists | 254 | Missing `paralist` package |
| amsarticle | 481 | Missing `amsart` class |
| enum | 530 | Missing `enumitem` features |
| figure_grids | 1137 | Missing figure/subfigure support |

## Disabled Suites (memory leaks)
- **22_fonts.rs** / **53_alignment.rs**: Unbounded memory leaks. Only `textsymbols_test` and `tabtab_test` kept individually.
