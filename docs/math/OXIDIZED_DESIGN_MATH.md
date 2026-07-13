# Oxidized Design — Math Parser & Grammar

[← OXIDIZED_DESIGN.md](../parity/OXIDIZED_DESIGN.md) · Marpa-style ambiguous grammar design + the numbered grammar-rule divergences.

> **Numbering note:** these `### N` numbers (`#16`, and the grammar cluster `#7–#18`) are a SEPARATE sequence from the divergences in [OXIDIZED_DESIGN_DIVERGENCES.md](../parity/OXIDIZED_DESIGN_DIVERGENCES.md) and collide with them by value — kept verbatim because code refers to them (notably `#18` = "Speculative function application", the f(x)→apply decision). See also divergence #4 (Marpa parser) and #15 (improved parses) in the divergences file.

---

### 16. Math Parser Design Rules

**Rule 1: Prefer grammar rules over post-parse rewrites.** Do not create rewrite rules in `semantics.rs` if the behavior can be expressed as a token rule or grammar rule in Marpa. If Perl's `MathGrammar` hints a grammar-level rule, implement it as a grammar rule.

**Rule 2: Aggressive intermediate pruning.** Ambiguous parses should be pruned early via pragmatic semantic actions. The same atoms and sub-expressions must coordinate their meanings — a given subexpression should always produce the same parse and use the same meaning within a single expression.

**Rule 3: Value-specific tokens via Marpa terminals.** When matching specific token values (like `d` for DIFFOP), prefer value-specific terminal definitions (e.g., `token!(diffd = "UNKNOWN:d")`) over runtime string checks in semantic actions. Note: the current Marpa tree builder has a limitation where one lexeme cannot match two terminals simultaneously, so value-specific terminals that overlap with role-based terminals (e.g., `diffd` overlapping `unknown`) require workarounds until the tree builder is fixed.
### 7. Angle Bracket Inner Product Parsing

**Decision:** `<x,y>` with RELOP `<` and `>` is recognized as an inner product
(fenced expression with angle bracket delimiters), producing
`delimited-<>@(list@(x, y))`.

**Rationale:** Old typesetting conventions used `<` `>` instead of `\langle` `\rangle`
for operator delimiters such as inner products. Perl's parser leaves these expressions
unparsed (`ltx_math_unparsed`). We do better by recognizing the `<term, term>` pattern
as fenced content. The `<<` and `>>` two-part relops (much-less-than, much-greater-than)
still take priority via the `two_part_relop` grammar rule.

**Grammar:** `fenced_factor += langle_rel term_list rangle_rel => fenced`, where
`term_list = term punct term | term_list punct term` handles arbitrary-length
comma-separated term chains.

**Impact:** `ambiguous_relations_test` equations `0=<x,y>` and `0=<x,y>A` now parse
correctly instead of being marked `ltx_math_unparsed`. Test XMLs updated to match.

### 8. Broad Bigop Argument Absorption

**Decision:** Bigops (`\sum`, `\int`, etc.) absorb the full `term` (mulop/invisible-times
chain), not just the next `tight_term`.

**Rationale:** `\sum_{i=0}^{\infty} f_i x^i` should produce `∑(f_i * x^i)`, not
`∑(f_i) * x^i`. The summation variable `i` appears in both `f_i` and `x^i`, so the
entire product is the summand. Perl's `addOpArgs` (Parse::RecDescent) non-deterministically
selects narrow absorption for some expressions (documented in KNOWN_PERL_ERRORS #9).

**Grammar:** `bigop_application = bigop/scripted_bigop/composed_bigop term`, lifted to
`expression` level so bigops can't be followed by invisible-times on the right.

**Impact:** `declare_test` sum equations updated. `calculus_test` improved (331→273 diffs).

### 9. Document-Order xml:id Renumbering

**Decision:** After math parsing completes, xml:ids inside each XMath subtree are
renumbered to be sequential in document order (pre-order DFS). Perl's
Parse::RecDescent generates IDs in bottom-up parse order (tokens first, then
higher-level constructs).

**Rationale:** The Marpa grammar parser explores multiple parse alternatives
simultaneously, consuming ID counter slots for pruned nodes. This produced
non-sequential IDs like `m1.1, m1.7, m1.12` instead of `m1.1, m1.2, m1.3`.
Document-order assignment is predictable and deterministic regardless of
parser internals. It uses a pure post-processing pass in `core_interface.rs`
after all parsing and kludge processing, before `document.finalize()`.

**Implementation:** `renumber_math_ids()` performs a single DFS walk per XMath
subtree, collecting both xml:id and idref nodes. Parent prefixes are derived
via O(1) string parsing (rfind('.')) instead of DOM ancestor walks. IDs are
stripped in a batch pass before reassignment to avoid idstore collisions.

**Impact:** Test XMLs for mathaccents, esint, mathbbol, not, choose, declare,
sampler, amsarticle, latextheorem, amstheorem, genfracs, amsdisplay, sets,
multirelations, standalone_modifiers, sequences_and_lists, and compose were
updated to reflect document-order IDs. All structural content is identical
to Perl; only ID values differ.

### 10. Grammar: Two-level sequence semantics (formulae vs list)

**Decision:** The Marpa grammar distinguishes two levels of comma/punct-separated
sequences, matching Perl's `Formulae`/`extendFormula` distinction:

- **`formulae`** (formula level): Punct-separated COMPLETE relational formulas.
  `a=b, c=d` → `formulae@(a=b, c=d)`. Produced by `formula_list` rule via
  `formulae_apply` semantic action.

- **`list`** (expression level): Punct-separated expressions within a formula.
  `a, b, c` → `list@(a, b, c)`. Also used for RHS extension: `a=b, c` →
  `a = list(b, c)`. Produced by `statements` rule via `list_apply`.

**Disambiguation rules** (semantic pruning, since Marpa explores both paths):

1. `formulae_apply` rejects when NO items are relational → forces `list_apply`.
2. `list_apply` rejects when BOTH items are relational → forces `formulae_apply`.
3. `list_apply` rejects when either item is relational and left is not already a
   list/formulae Dual → forces `formulae_apply`.
4. `infix_relation` (multirelation extension) rejects when the left formula's
   last operand is a `list` Dual → prevents `a = list(b,c) = d`, forcing the
   comma to be a formula boundary instead.
5. Both `list_apply` and `formulae_apply` reject items with `absent` relop
   operands (equation fragments) — see rule 11.

**Rationale:** Perl's Parse::RecDescent resolves this structurally through rule
ordering (extendFormula consumes commas before moreFormulae can see them). Marpa
explores all alternatives simultaneously, so semantic pruning is needed. The
rules above create a clean partition: relational items go through formulae,
non-relational through list, with multirelation rejection preventing the
"comma inside formula RHS" misparse.

### 11. Grammar: Absent operands are formula-level only

**Decision:** The `absent` token (meaning="absent") represents a missing/implied
operand, typically from alignment cell boundaries in multi-line equations:

```latex
a(x) &= f(x) + g(x) + h(x) \\
     &= f(x) + \phantom{g(x)} + h(x)
```

The second row `= f(x) + \phantom{g(x)} + h(x)` has an absent LHS (the `a(x)`
from the row above). This is a single formula fragment: `absent = f(x) + ... + h(x)`.

**Rules:**
- `absent` as a relop operand is valid in a single **formula** (equation fragment).
- `absent` is NOT valid inside a **list** — `list_apply` rejects.
- `absent` is NOT valid inside a **formulae** collection — `formulae_apply` rejects.
- At the top level, a formula with `absent` is a standalone fragment, not part of
  a multi-formula collection.

**Open question:** `\phantom` creates intentional gap space that may need a
dedicated grammar rule. Currently, `\phantom{g(x)}` produces a box with
invisible content. When alignment cell boundaries split an expression containing
`\phantom`, the fragments become unparseable. The proper fix requires alignment
infrastructure to join cells before math parsing, or a dedicated phantom rule
that preserves expression continuity across cell boundaries.

### 12. Grammar: bigop_application at term level

**Decision:** `bigop_application` (e.g. `\neg b`, `\sum x dx`) is placed at the
`term` level in the grammar (`term += bigop_application`), not at the `expression`
level. This prevents exponential Marpa ambiguity when ADDOP precedes BIGOP
(e.g. `a + \neg b`).

**Rationale:** At expression level, `expression += bigop_application` combined
with `expression = term addop expression` created multiple derivation paths for
the same semantic result (e.g. `π + ¬a`). The Marpa Earley recognizer explored
all paths, causing exponential tree enumeration. At term level, the addop rule
handles the combination with a single derivation.

### 13. Grammar: Period and comma precedence in formulae

**Decision:** Period (`.`) and comma (`,`) are both formula/list separators at
the same grammar level (`statements`/`formula_list`). Comma after a relational
formula's RHS groups as a list (`a=b,c` → `a=list(b,c)`), while period always
creates a hard formula boundary (`a=b.c` → `formulae(a=b, c)`).

For `a=b.c,d=e`, the Rust parse is `formulae(a=b, c, d=e)` — three separate items.
Perl produces `formulae(a=b, list(c,d)=e)` — grouping `c,d` across the period as
a list LHS. The Rust parse is accepted as a valid alternative.

**Rationale — the long tail of rare mathematical notation:**

Mathematical notation is a natural language with centuries of accumulated conventions.
While common patterns (like `a=b,c=d` for parallel equations or `a=b,c` for a set-like
RHS list) appear frequently and have clear semantic intent, the interaction between
MULTIPLE separators in a single expression creates a combinatorial explosion of
edge cases that are vanishingly rare in practice.

Expressions like `a=b.c,d=e` (mixing period and comma with multiple relations)
essentially never appear in real mathematical writing. When they do, the intended
semantics are ambiguous even to human readers without surrounding context. Attempting
to match Perl's interpretation for every long-tail combination:
- Adds grammar complexity that risks regressions on common patterns
- Encodes arbitrary choices that may not reflect any real author's intent
- Cannot be validated against actual mathematical usage

The Rust port prioritizes:
1. **Correct handling of common patterns** (>99% of real math)
2. **Defensible alternatives** for rare patterns (valid parse, just different grouping)
3. **Grammar simplicity** to avoid Marpa ambiguity explosion

When the Rust parse differs from Perl on a rare notation, both parses are typically
valid mathematical interpretations. We accept the Rust parse as a documented
intentional divergence rather than adding complexity to match Perl exactly.

### 14. Grammar: Generic open/close fenced delimiters

**Decision:** Added `open expression close => fenced` rule for generic OPEN/CLOSE
delimiter pairs (e.g. `\lfloor...\rfloor`, `\lceil...\rceil`, `\Lbag...\Rbag`).
Previously, only specific delimiter pairs (parens, brackets, braces, vertbar)
had fenced rules. Added floor/ceiling/norm semantic meanings for known delimiter
pairs.

### 15. Grammar: Evaluated-at and norm patterns

**Decision:** Added `evaluated-at` pattern (`a|_∞` → `evaluated-at@(a, ∞)`)
and `norm` pattern (`||a||` → `norm@(a)` with ‖). These match Perl's
MathGrammar `evalAtOp`/`maybeEvalAt` and `SINGLEVERTBAR SINGLEVERTBAR`
rules respectively.

### 16. Grammar: Bigop argument scope after invisible times

**Decision:** Removed `any_bigop` from `scripted_factor_r11`/`scripted_factor_r12`
rules. Bigops now ONLY get scripts via `scripted_bigop`, ensuring
`bigop_application` always fires and absorbs the following term.

Before this change, `1/2∫_0^1 f dx` parsed as `(1/2)*(∫_0)^1*f*dx` because
the integral was treated as a scripted factor, preventing argument absorption.
After: `(1/2)*((∫_0)^1)@(f*dx)`.

**Note:** Explicit mulop (`\times`) between bigop and its argument still breaks
absorption: `∫ F×G dx` → `integral(F)*G*dx`. Both `∫(F)` and `∫(F×G×dx)` are
valid Marpa parses; tree selection currently prefers the shorter absorption.
This is a known limitation affecting rare explicit-mulop-in-integrand patterns.

### 17. Script content preservation (C5)

**Decision:** `faux_wrap` now returns `XM::Wrap([start_script_lexeme, parsed_content])`
instead of just the lexeme. `new_script_inner` detects this and uses the parsed
content directly, avoiding re-reading from DOM via `obtain_arg`.

This fixes empty XMRef for any parsed expression inside scripts:
- `f^{(n)}` → `f ^ n` (was `f ^ []` — fenced XMDual discarded)
- `q_{a,b}` → `q _ list(a,b)` (was `q _ list([], [])`)

The root cause was that `obtain_arg` re-read the original DOM, which still had
the raw tokens `(`, `n`, `)` — not the parsed `fenced@(n)` XMDual.

### 18. Speculative function application produces Apply, not invisible times

**Decision:** For any UNKNOWN token `f` followed by a fenced expression `(x)`,
Rust produces `f@(x)` (function application) rather than Perl's default
`f * x` (invisible-times multiplication). This is the *always-on* default,
not gated on any flag.

**Rationale.** Parse::RecDescent (Perl) can only commit to one parse. Its
`MaybeFunctions` mechanism was a workaround: mark the UNKNOWN token with
`possibleFunction="yes"` and then fail the production, yielding invisible-times
with an advisory attribute. Marpa (Rust) is an ambiguous CFG engine — the
grammar produces *both* interpretations in the forest, and the pragmatic layer
picks one. `FencedLettersAreFunctionArguments` is the authoritative selector:
when mathematical practice reads `f(x)` as function application (which it
always does for a letter `f` and any non-NUMBER content in the parens), that
is the tree we keep.

**Role of `MATHPARSER_SPECULATE`.** The flag no longer influences parse
structure. Its only remaining effect is to enable the `possibleFunction="yes"`
diagnostic attribute on UNKNOWN tokens that participate in such speculation.
`\usepackage[mathparserspeculate]{latexml}` is kept for backwards compatibility
but does not change which tree wins.

**Author override.** Authors who want `f(x) = f * x` can declare `f` as ID:
`\lxDeclare[role=ID]{f}`. With the ID role, the speculative grammar rule
`unknown fenced_factor` does not apply (it's gated on role UNKNOWN), so only
the invisible-times parse is produced.

**Affected tests:** 13 test XMLs updated session 107 (previously recorded
Perl's SPECULATE-off behavior; now record mathematically-consistent parses).

**Reaffirmed 2026-06-22 (user decision, AskUserQuestion "Keep f@(x) apply as
intentional divergence").** A survey of the apply-vs-multiply family confirmed the
clean split: KNOWN functions already match Perl (`\sin(x)`→`sine@(x)` in both);
only UNKNOWN symbols diverge (`f(x)`→Rust `f@(x)` vs Perl `f * x`;
`\Gamma(s)`→Rust `Gamma@(s)` vs Perl `Gamma * s`). The corpus-wide change to match
Perl (≈25 test fixtures / ≈150 single-letter applies flip to multiply) was
**declined**: `f@(x)` application is the better semantics for the common
function-call case, so it stays the intentional divergence above. **Distinct,
complementary fix (toward Perl, in progress):** the KNOWN-function multi-arg
*flattening* — `\max(a,b)` should be `max@(a,b)`, not `max@(vector@(a,b))` (Perl
`ApplyDelimited`/`extract_separators` spreads the comma-list items as direct
args). That is a parity bug, NOT a divergence; tracked in SYNC_STATUS
("`f(a,b)` multi-arg flattening"). It is scoped to FUNCTION/OPFUNCTION/
TRIGFUNCTION roles, so it does NOT touch the unknown-`f` apply preserved here.

**Re-affirmed 2026-07-02 — the strongest form of the decision.** The
toward-Perl flip was green-lit that morning, then FULLY IMPLEMENTED and
verified (12-formula witness set byte-identical to same-host Perl; ~22
fixtures re-blessed toward Perl; grammar productions + the
`FencedLettersAreFunctionArguments` pragma removed) — and then **reverted on
user review before pushing**: *"f(x) is almost always an application in
common STEM use."* The application reading is a deliberate beyond-Perl
quality choice (screen readers say "f of x", not "f times x"; U+2061 vs
U+2062), and it wins over strict Perl parity here. The reverted
implementation — including the finding that the pragma is load-bearing (its
deletion alone leaves `f(x)` unparseable) and the per-fixture toward-Perl
verification method — is preserved on branch
`archive/fx-perl-parity-attempt-2026-07-02` (commit `bcf88db280`). Do not
re-attempt the flip without a fresh explicit user decision.

---
