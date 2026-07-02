# MathML post-processors — line audit (exhaustive-port verification)

> **Living worklist** (opened 2026-07-02, user-commissioned: "the Rust
> translation wants to be an exhaustive port"). Walk every Perl sub AND every
> `DefMathML` registration in the MathML post-processing stack, verdict each
> against the Rust port, and — critically — verify the **wiring** (producer →
> consumer chains), not just function existence. When complete: date + move to
> `docs/archive/`, lifting residuals into `SYNC_STATUS.md`.

## Motivation (the charter bugs)

The astro-ph0001001 `S9.Ex4.m1` lost-spaces bug (fixed 2026-07-02,
`3ab9ce3cb3`) required BOTH halves of a wiring gap: Perl MathML.pm L344-348
(attach `lpadding`/`rpadding` → `_lpadding`/`_rpadding`) was never ported —
the consumer existed with no producer — AND the live `adjust_pair` dropped
Perl's author-spacing term from `target` and could not materialize onto an
invisible operator. Neither is findable by name-matching functions. Also
discovered: a **dead duplicate spacewalk** in `mod.rs` (unused, but it HAD
the padding term the live one lacked), and a Linebreaker port ratio (Perl
1053L/58 subs vs Rust 378L/7 fns) that suggests wholesale stubs.

## Method

- Perl side: `LaTeXML/lib/LaTeXML/Post/MathML.pm` (60 subs + 197 `DefMathML`
  registrations), `MathML/{Presentation,Content,OperatorDictionary,
  Linebreaker}.pm`, plus the `MathProcessor.pm` parallel-markup layer.
- Rust side: `latexml_post/src/mathml/{presentation,mod,content,
  operator_dictionary,linebreaker}.rs`.
- Verdicts: **PORTED** (where) / **PARTIAL** (what's missing) / **MISSING** /
  **N-A** (infrastructure Perl-ism). Every PARTIAL/MISSING becomes a numbered
  Finding; fixes ship individually with a witness formula compared against
  Perl-generated MathML (`latexmlc --format=html5`; NB it segfaults at exit
  on this host AFTER writing complete output — exit 139 is not a failure).
- Regenerate inventories:
  `grep -n '^sub ' LaTeXML/lib/LaTeXML/Post/MathML.pm` and
  `grep -oE "DefMathML\(...\"" …` (keys below).

## Internal-attribute wiring table (the bug class)

| attr | Perl producer | Perl consumer | Rust producer | Rust consumer | status |
|---|---|---|---|---|---|
| `_role` | stylizeContent L821+ / pmml L355 | adjust_pair (atom types) | presentation.rs stylize port (L642) | get_node_role | ✅ wired |
| `_lspace`/`_rspace` | stylizeContent (opdict) | adjust_pair defaults | presentation.rs (opdict>0 only) | get_node_attr_f64 | ⚠️ verify: Perl stores 0-values? check `>0` gate |
| `_lpadding`/`_rpadding` | pmml L344-348 | adjust_pair L1228-9 | pmml wrapper (`attach_source_padding`, 2026-07-02) | adjust_pair | ✅ FIXED `3ab9ce3cb3` |
| `_largeop` | stylizeContent | pmml_bigop?/display sizing | 4 sites | ? | ⬜ trace |
| `_ignorable` | stylizeContent (mo ignorables) | linebreaker? | 1 site | ? | ⬜ trace |
| cleanup | (attrs dropped on serialize — Perl arrays never serialize _keys) | | `clean_internal_attrs` | | ✅ (verify runs AFTER spacewalk in all pipelines) |

## Findings

- **F1 ✅ FIXED (`3ab9ce3cb3`)** — `_lpadding`/`_rpadding` producer missing +
  `adjust_pair` author-spacing term dropped + no invisop materialization.
- **F2 ⬜ dead duplicate** — `mod.rs::adjust_spacing`/`space_walk`/
  `get_internal_attr` (mod.rs ~1444-1560) is an unused second port with
  *different* behavior. Reconcile into the live presentation.rs one, delete.
- **F3 ⬜ adjust_pair unported branches** — Perl L1255-1262: `target < 0`
  → mpadded width rewrap (needs `compute_size` L1135, MISSING); prev/next
  `m:mspace` → width merge with `getXMHintSpacing`. TODO comments in code.
- **F4 ⬜ fmt_em format divergence** — Perl `sprintf("%.3fem")` → `0.330em`;
  Rust trims trailing zeros → `0.33em`. Semantically equal; decide byte-parity.
- **F5 ⬜ Linebreaker coverage** — Perl 1053L/58 subs vs Rust 378L/7 fns.
  Enumerate what `convertNode_linebreak` needs; likely large PARTIAL/MISSING.
- **F6 ⬜ space_walk simplifications** — Perl space_walk descends into
  scripts (L1124) and unwraps rows; embellisher descent for atom-typing
  exists in Rust (`get_node_role`) but the script-descent recursion needs
  verification.

## MathML.pm named subs (60)

| Perl sub (line) | verdict | notes |
|---|---|---|
| `preprocess` (L66) | ⬜ | |
| `outerWrapper` (L77) | ⬜ | |
| `rawIDSuffix` (L109) | ⬜ | |
| `combineParallel` (L113) | ⬜ | |
| `getQName` (L147) | ⬜ | |
| `addCrossref` (L154) | ⬜ | |
| `realize` (L163) | ⬜ | |
| `getOperatorRole` (L173) | ⬜ | |
| `DefMathML` (L199) | ⬜ | |
| `lookupPresenter` (L205) | ⬜ | |
| `lookupContent` (L212) | ⬜ | |
| `pmml_top` (L273) | ⬜ | |
| `find_inherited_attribute` (L291) | ⬜ | |
| `pmml_smaller` (L303) | ⬜ | |
| `pmml_scriptsize` (L311) | ⬜ | |
| `pmml` (L318) | ⬜ | |
| `first_element` (L359) | ⬜ | |
| `_getattr` (L367) | ⬜ | |
| `_getspace` (L371) | ⬜ | |
| `getXMHintSpacing` (L380) | ⬜ | |
| `pmml_internal` (L387) | ⬜ | |
| `needsMathstyle` (L512) | ⬜ | |
| `pmml_maybe_resize` (L525) | ⬜ | |
| `filter_row` (L577) | ⬜ | |
| `pmml_row` (L581) | ⬜ | |
| `pmml_unrow` (L586) | ⬜ | |
| `pmml_parenthesize` (L594) | ⬜ | |
| `pmml_punctuate` (L611) | ⬜ | |
| `pmml_infix` (L626) | ⬜ | |
| `stylizeContent` (L672) | ⬜ | |
| `pmml_mi` (L830) | ⬜ | |
| `pmml_mn` (L836) | ⬜ | |
| `pmml_mo` (L842) | ⬜ | |
| `pmml_bigop` (L847) | ⬜ | |
| `pmml_script` (L876) | ⬜ | |
| `pmml_script_mid_layout` (L893) | ⬜ | |
| `pmml_scriptsize_padded` (L926) | ⬜ | |
| `pmml_script_multi_layout` (L936) | ⬜ | |
| `pmml_script_decipher` (L963) | ⬜ | |
| `pmml_text_aux` (L1029) | ⬜ | |
| `adjust_spacing` (L1079) | ⬜ | |
| `space_walk` (L1096) | ⬜ | |
| `compute_size` (L1135) | ⬜ | |
| `adjust_pair` (L1220) | ⬜ | |
| `fmt_em` (L1285) | ⬜ | |
| `cmml_top` (L1290) | ⬜ | |
| `cmml` (L1301) | ⬜ | |
| `cmml_internal` (L1311) | ⬜ | |
| `cmml_contents` (L1350) | ⬜ | |
| `cmml_unparsed` (L1360) | ⬜ | |
| `cmml_leaf` (L1377) | ⬜ | |
| `cmml_decoratedSymbol` (L1396) | ⬜ | |
| `cmml_not` (L1406) | ⬜ | |
| `cmml_synth_not` (L1410) | ⬜ | |
| `cmml_synth_complement` (L1415) | ⬜ | |
| `cmml_shared` (L1420) | ⬜ | |
| `cmml_share` (L1426) | ⬜ | |
| `cmml_or_compose` (L1436) | ⬜ | |
| `pmml_summation` (L1796) | ⬜ | |
| `do_cfrac` (L1931) | ⬜ | |

## Sibling files

| Perl sub | verdict | notes |
|---|---|---|
| Presentation.pm: preprocess / convertNode_simple / convertNode_linebreak / convertNode / rawIDSuffix / associateNodeHook / preprocess_linebreaking | ⬜ | linebreaking entry points — pairs with F5 |
| Content.pm: convertNode / rawIDSuffix / canConvert | ⬜ | |
| OperatorDictionary.pm: opdict_lookup / lookup_category / decode_ranges | ⬜ | Rust `operator_dictionary.rs` (11 fns) — verify table completeness |
| MathProcessor.pm (parallel markup / associateNode / crossref) | ⬜ | scope: only the parts MathML.pm calls |

## DefMathML registrations (197 calls, 196 unique keys)

Verdict ⬜/✅ per key: does the Rust dispatch (presentation.rs match arms +
content.rs) produce the same markup for the same role/meaning?

- ⬜ `Apply:?:?`
- ⬜ `Apply:ADDOP:?`
- ⬜ `Apply:?:annotated`
- ⬜ `Apply:ARROW:?`
- ⬜ `Apply:BIGOP:?`
- ⬜ `Apply:BINOP:?`
- ⬜ `Apply:?:closed-interval`
- ⬜ `Apply:?:closed-open-interval`
- ⬜ `Apply:COMPOSEOP:?`
- ⬜ `Apply:?:contains`
- ⬜ `Apply:?:continued-fraction`
- ⬜ `Apply:ENCLOSE:?`
- ⬜ `Apply:?:formulae`
- ⬜ `Apply:FRACOP:?`
- ⬜ `Apply:?:hack-definite-integral`
- ⬜ `Apply:INTOP:?`
- ⬜ `Apply:?:less-than-or-approximately-equals`
- ⬜ `Apply:?:limit-from`
- ⬜ `Apply:?:list`
- ⬜ `Apply:METARELOP:?`
- ⬜ `Apply:MIDDLE:?`
- ⬜ `Apply:MODIFIEROP:?`
- ⬜ `Apply:MULOP:?`
- ⬜ `Apply:?:multirelation`
- ⬜ `Apply:?:not-approximately-equals`
- ⬜ `Apply:?:not-contains`
- ⬜ `Apply:?:nth-root`
- ⬜ `Apply:?:open-closed-interval`
- ⬜ `Apply:?:open-interval`
- ⬜ `Apply:OVERACCENT:?`
- ⬜ `Apply:POSTFIX:?`
- ⬜ `Apply:RELOP:?`
- ⬜ `Apply:?:set`
- ⬜ `Apply:?:square-root`
- ⬜ `Apply:SUBSCRIPTOP:?`
- ⬜ `Apply:SUMOP:?`
- ⬜ `Apply:SUPERSCRIPTOP:?`
- ⬜ `Apply:?:superset-of`
- ⬜ `Apply:?:superset-of-and-not-equals`
- ⬜ `Apply:?:superset-of-or-equals`
- ⬜ `Apply:UNDERACCENT:?`
- ⬜ `Apply:?:vector`
- ⬜ `Array:?:?`
- ⬜ `Array:?:cases`
- ⬜ `Hint:?:?`
- ⬜ `Token:?:?`
- ⬜ `Token:?:absent`
- ⬜ `Token:?:absolute-value`
- ⬜ `Token:ADDOP:?`
- ⬜ `Token:ADDOP:minus`
- ⬜ `Token:ADDOP:plus`
- ⬜ `Token:?:and`
- ⬜ `Token:APPLYOP:?`
- ⬜ `Token:?:approximately-equals`
- ⬜ `Token:?:argument`
- ⬜ `Token:ARROW:?`
- ⬜ `Token:BIGOP:?`
- ⬜ `Token:BINOP:?`
- ⬜ `Token:?:cardinality`
- ⬜ `Token:?:cartesian-product`
- ⬜ `Token:?:ceiling`
- ⬜ `Token:CLOSE:?`
- ⬜ `Token:?:codomain`
- ⬜ `Token:?:compose`
- ⬜ `Token:COMPOSEOP:?`
- ⬜ `Token:?:conjugate`
- ⬜ `Token:?:cosecant`
- ⬜ `Token:?:cosine`
- ⬜ `Token:?:cotangent`
- ⬜ `Token:?:curl`
- ⬜ `Token:?:determinant`
- ⬜ `Token:?:differential`
- ⬜ `Token:DIFFOP:?`
- ⬜ `Token:?:divergence`
- ⬜ `Token:?:divide`
- ⬜ `Token:?:domain`
- ⬜ `Token:?:element-of`
- ⬜ `Token:?:equals`
- ⬜ `Token:?:equivalent-to`
- ⬜ `Token:?:exists`
- ⬜ `Token:?:exponential`
- ⬜ `Token:?:factorial`
- ⬜ `Token:?:factor-of`
- ⬜ `Token:?:floor`
- ⬜ `Token:?:forall`
- ⬜ `Token:?:gcd`
- ⬜ `Token:?:gradient`
- ⬜ `Token:?:greater-than`
- ⬜ `Token:?:greater-than-or-equals`
- ⬜ `Token:?:hyperbolic-cosecant`
- ⬜ `Token:?:hyperbolic-cosine`
- ⬜ `Token:?:hyperbolic-cotantent`
- ⬜ `Token:?:hyperbolic-secant`
- ⬜ `Token:?:hyperbolic-sine`
- ⬜ `Token:?:hyperbolic-tangent`
- ⬜ `Token:ID:circular-pi`
- ⬜ `Token:ID:complexes`
- ⬜ `Token:ID:empty-set`
- ⬜ `Token:?:identity`
- ⬜ `Token:ID:Euler-constant`
- ⬜ `Token:ID:exponential-e`
- ⬜ `Token:ID:false`
- ⬜ `Token:ID:imaginary-i`
- ⬜ `Token:ID:infinity`
- ⬜ `Token:ID:integers`
- ⬜ `Token:ID:notanumber`
- ⬜ `Token:ID:numbers`
- ⬜ `Token:ID:primes`
- ⬜ `Token:ID:rationals`
- ⬜ `Token:ID:reals`
- ⬜ `Token:ID:true`
- ⬜ `Token:?:image`
- ⬜ `Token:?:imaginary-part`
- ⬜ `Token:?:implies`
- ⬜ `Token:?:integral`
- ⬜ `Token:?:intersection`
- ⬜ `Token:INTOP:?`
- ⬜ `Token:?:inverse`
- ⬜ `Token:?:inverse-cosecant`
- ⬜ `Token:?:inverse-cosine`
- ⬜ `Token:?:inverse-cotangent`
- ⬜ `Token:?:inverse-hyperbolic-cosecant`
- ⬜ `Token:?:inverse-hyperbolic-cosine`
- ⬜ `Token:?:inverse-hyperbolic-cotangent`
- ⬜ `Token:?:inverse-hyperbolic-secant`
- ⬜ `Token:?:inverse-hyperbolic-sine`
- ⬜ `Token:?:inverse-hyperbolic-tangent`
- ⬜ `Token:?:inverse-secant`
- ⬜ `Token:?:inverse-sine`
- ⬜ `Token:?:inverse-tangent`
- ⬜ `Token:?:lambda`
- ⬜ `Token:?:laplacian`
- ⬜ `Token:?:lcm`
- ⬜ `Token:?:less-than`
- ⬜ `Token:?:less-than-or-equals`
- ⬜ `Token:?:limit`
- ⬜ `Token:LIMITOP:?`
- ⬜ `Token:?:logarithm`
- ⬜ `Token:?:matrix`
- ⬜ `Token:?:maximum`
- ⬜ `Token:?:mean`
- ⬜ `Token:?:median`
- ⬜ `Token:METARELOP:?`
- ⬜ `Token:MIDDLE:?`
- ⬜ `Token:?:minimum`
- ⬜ `Token:?:minus`
- ⬜ `Token:?:mode`
- ⬜ `Token:MODIFIEROP:?`
- ⬜ `Token:?:moment`
- ⬜ `Token:MULOP:?`
- ⬜ `Token:?:natural-logarithm`
- ⬜ `Token:?:not`
- ⬜ `Token:?:not-element-of`
- ⬜ `Token:?:not-equals`
- ⬜ `Token:NUMBER:?`
- ⬜ `Token:OPEN:?`
- ⬜ `Token:OPERATOR:?`
- ⬜ `Token:?:or`
- ⬜ `Token:?:outer-product`
- ⬜ `Token:OVERACCENT:?`
- ⬜ `Token:?:partial-differential`
- ⬜ `Token:PERIOD:?`
- ⬜ `Token:?:plus`
- ⬜ `Token:POSTFIX:?`
- ⬜ `Token:?:power`
- ⬜ `Token:?:prod`
- ⬜ `Token:PUNCT:?`
- ⬜ `Token:?:quotient`
- ⬜ `Token:?:real-part`
- ⬜ `Token:RELOP:?`
- ⬜ `Token:?:remainder`
- ⬜ `Token:?:scalar-product`
- ⬜ `Token:?:secant`
- ⬜ `Token:?:selector`
- ⬜ `Token:?:set-minus`
- ⬜ `Token:?:sine`
- ⬜ `Token:?:standard-deviation`
- ⬜ `Token:SUBSCRIPTOP:?`
- ⬜ `Token:?:subset-of`
- ⬜ `Token:?:subset-of-and-not-equals`
- ⬜ `Token:?:subset-of-or-equals`
- ⬜ `Token:?:sum`
- ⬜ `Token:SUMOP:?`
- ⬜ `Token:SUPERSCRIPTOP:?`
- ⬜ `Token:SUPOP:?`
- ⬜ `Token:?:tangent`
- ⬜ `Token:?:tends-to`
- ⬜ `Token:?:times`
- ⬜ `Token:?:transpose`
- ⬜ `Token:?:uminus`
- ⬜ `Token:UNDERACCENT:?`
- ⬜ `Token:?:union`
- ⬜ `Token:?:variance`
- ⬜ `Token:?:vector-product`
- ⬜ `Token:VERTBAR:?`
- ⬜ `Token:?:xor`
