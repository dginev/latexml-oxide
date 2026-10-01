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

**Relations, not angles (57ch, 57ch.1):** the signs are relations — Perl's reading, which parses most of these
(`a<b,c>d` `formulae@(a < b, c > d)`) — where an operand ends before the `<` and one starts after the `>`
(`c_1<c_D,c_2>c_D`), or where the `<` continues an inequality whatever follows the `>` (`0<s<1,\ t>-1` formulae@(0 < s
< 1, t > −1), `\mathbb{P}(m<S<m+\delta\mid S>m)` P@(conditional@(…)); `semantics::angle_signs_are_relations`,
`continues_an_inequality`). A plain Dirac bracket `<a|b>`, `<a|f|b>` is refuted only by the inequality rule: a factor on
either side keeps it (`c_m^*<m|H|n>c_n`, as Perl). `2<x,y>=z` keeps its angles (this golden); Perl joins `>` `=` into one
relation (`TwoPartRelop`, MathGrammar:711) and reads `formulae@(2 < x, y >= z)`. Guards: `tests/parse/fenced_lists.tex`
("Plain angle signs between operands are relations"), `tests/math/ambiguous_relations`.

### 8. Broad Bigop Argument Absorption

**Decision:** Bigops (`\sum`, `\int`, etc.) absorb the full `term` (mulop/invisible-times
chain), not just the next `tight_term`.

**Rationale:** `\sum_{i=0}^{\infty} f_i x^i` should produce `∑(f_i * x^i)`, not
`∑(f_i) * x^i`. The summation variable `i` appears in both `f_i` and `x^i`, so the
entire product is the summand. Perl's `addOpArgs` (Parse::RecDescent) non-deterministically
selects narrow absorption for some expressions (documented in KNOWN_PERL_ERRORS #9).

**Grammar:** `bigop_application = any_bigop/scripted_bigop/prescripted_bigop term`, lifted to
term level so bigops can't be followed by invisible-times on the right. Adjacent bigops apply in
turn (`\partial\partial f` = ∂@(∂@(f))); a bare bigop is an operand (`bigop_operand`); only a
FLOAT script starts a pre-scripted bigop; a function or operator that starts a term before a bigop
is a factor (`function_factor bigop_operand`: `\min_\theta\sum` = min_θ * ∑…) (57w/57x).

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

**In a bare argument (batch 57am).** Perl's `aBarearg` (MathGrammar:327) is
`preScripted['UNKNOWN'] doubtArgs`, and `doubtArgs` leaves the `(`: Perl reads
`\log f(x)` as log@(f)·x. Rust's bare argument item (`speculative_item`, for an
operator and an OPFUNCTION alike, alone or in the greedy chain) applies the
unknown to its group wherever it stands in the chain: `\log f(x)` log@(f@(x)),
`\nabla f(x)` ∇@(f@(x)), `a\log f(x)` a·log@(f@(x)), `\eta\nabla L(\theta)`
η·∇@(L@(θ)), `\log\lambda g(x)` log@(λ·g@(x)), `\log 2y(1+x)` log@(2·y@(1+x)); a
juxtaposed product that leaves the group outside the argument's last item, through
nested bare applications, is pruned (`leaves_a_bare_argument`, `takes_the_group`,
`last_bare_leaf`). A top-level product still goes by the student pragmas. Since 57av
`FencedLettersAreFunctionArguments` checks every factor beside the one before it and
ranks readings by its violation count, so an applied reading wins wherever the grammar
offers one: `f(x)(a+b)` f@(x)·(a+b), `(f(x)+1)(g(x)+1)`, `k(x-y)(x+y)` k@(x−y)·(x+y),
`L(f)(x)` L@(f)·x. A letter whose own group holds it as a plain value multiplies it, as Perl (57bz,
`letter_recurs_in_its_group`; 2605.28300, 2605.02279, 2605.04013, 2605.30479, 2605.00539, 2605.31439):
`x(x+1)` x·(x+1), `n(n-1)(N-n)` n·(n−1)·(N−n), `\lambda(\lambda I-A)`, `x(x-(y+z))`; a letter only, in the
same font, not a head (`f(f(x))`), not in function position (a differential `d` before a factor: `d(x\,dy)`
d@(x·d·y), 2605.21794), not scripted (`x(1+x^2)` keeps the application, Perl multiplies), not before an argument list (`u(x,u)`, `E(Y|X,E)`,
`I(a;I\mid q)`, `u(x,u>0)` — the sign of a function Perl's `forbidArgs` flags under MaybeFunctions,
MathGrammar:524-525, where default Perl multiplies every unknown), not in a named font
(`\mathbb E[\mathbb E[X]]`, `{\cal V}(|{\cal V}|=K)`). It holds in a bare argument too, which then ends
before the group: `\sin x(x+1)` sin@(x)·(x+1), `\log 2x(1+x)` log@(2·x)·(1+x) as Perl, but also `\max_x
x(1-x)` max_x@(x)·(1−x) and `\ker R(R+\mathrm{id})` (2605.21992) where the argument is the whole product. The
chain after an application does not start at a letter times its own group: `p(p+2)\Gamma(p/2)` p·(p+2)·Γ·(p/2)
(2605.26653, 21 formulas), under the open coefficient question. Known misses, no structural cue: an operator
on its own image (`T(Tx)`, `V(UVx)`, 2605.01968), a measure of a set naming it (`\gamma(\ldots\setminus
\operatorname{spt}\gamma)`, 2605.10491), italic expectation shorthand (`E[X-EX]`), name clashes (`G[V(G)\setminus
S]`, 2605.11288), a pairing `s\left(a,b\right)`, a nested application (`f(\min(\operatorname{dom}(f)))`,
2605.21142; `m(\mathrm{id}\otimes m)`, 2605.11903) — SYNC_STATUS "A letter times its own group". After a visible operator the letter
applies too, the fenced factor being the last operand's (57az: `\lambda\cdot g(x)`
λ·g@(x), `\Omega(n\cdot f(n))`, `v(O)-\beta\cdot c(O)`; 2605.00201, 2605.00411,
2605.00423). After another application — a group its function closed — the letter applies
too (user ruling 2026-09-29; 57bl, `application_before_a_letter`): `f(x)g(y)` f@(x)·g@(y),
`P(A|B)P(B|C,D)`, `\Gamma(s)\zeta(s)`, `\log(x)f(y)` (1,607 formulas / 373 papers read
f@(x)·g·y; 2605.09849, 2605.08899, 2605.05133) — a group in parentheses or brackets only, as at the start
of a product: `U(t)H|\psi\rangle` U@(t)·H·ket, `P(A)P\{X>0\}` P@(A)·P·set (57bn). An ellipsis is transparent to the chain (57bv): a letter after it reads as it would with the ellipsis removed —
`g(1)g(2)\cdots g(n)` g@(1)·g@(2)·⋯·g@(n), `\cdots g(n)` ⋯·g@(n), `a\cdots g(n)` a·⋯·g·n (after a coefficient);
2605.23467, 2605.10016, 2605.05078 (`ellipsis_after_an_application`, golden `tests/parse/ellipsis_products`). After a juxtaposed coefficient the grammar
offers none, so Perl's reading stays: `\lambda g(x)` λ·g·x, `2x(1+y)` 2·x·(1+y),
`(a+b)g(x)` (repro `math-parse/application_after_a_leading_factor`, which asks whether #18
should reach there) — so a bare argument applies more often than a top-level product does. #18
applies letters, never a pre-built atom: a role-less XMDual (`\binom`, a matrix, `cases`,
physics `\abs`) lexes as ATOM, as Perl's `getGrammaticalRole` gives it (MathParser.pm:851-854),
and stays a product (`\binom{n}{2}(x+1)` binomial@(n, 2)·(x+1); 57be). With Perl's
greedy chain this reads `\nabla f(x)\cdot d` as ∇@(f@(x)·d) (Perl's greed gives
`\nabla u\cdot d` ∇@(u·d) too). A scripted group stays outside (`\nabla f(x)^T d`
∇@(f)·x^T·d, as `f(x)^2` is f·x²). A postfix after a letter's application (57bw) takes the application where
#18 applies the letter without one — first in a product, after another application, in a bare argument
(`letter_postfixed`) — beyond Perl, whose `addScripts` runs before a head's arguments (MathGrammar:419-424,
:545-558): `x(n+1)!` (x@(n+1))!, `f(n)g(n)!` f@(n)·(g@(n))! (Perl x·(n+1)!, f·n·g·n!); after a juxtaposed coefficient
the letter multiplies its postfixed group as it does without the postfix, `2n(n-1)!` 2·n·(n−1)! (the coefficient
question left open). A function's application takes it too, `f(x)!` (f@(x))!; an OPFUNCTION's argument takes it,
bare or a group, `\log n!` log@(n!), `\log(n-k)!` log@((n−k)!) (Perl unparsed). A trig function's and an operator's
argument take it too (57ca, user ruling 2026-09-29: factorial is ill-defined on the reals a trig function returns;
every POSTFIX alike, `\sin 30\%` sin@(30%)), `\sin x!` sin@(x!), `\nabla f!` ∇@(f!), as Perl's `aTrigBarearg`/`aBarearg`
with `addScripts`, and beyond Perl a group, `\sin(n)!` sin@(n!), `\nabla(f)!` ∇@(f!) (Perl unparsed); a
group's one operand (floor, ceiling, angle, conditional) takes it, `\log\lfloor n/2\rfloor!` log@(⌊n/2⌋!), and a list's
application the whole, `\max(a,b)!` (max@(a, b))! (`group_factor` = `operand_group` | `list_group`). Golden
`tests/parse/postfix_operands`.

**Juxtaposed functions (57cb, divergence #376).** A bare argument's later items take no OPFUNCTION
(`op_bare_next`): `\log f(x)\log g(x)` is log@(f@(x))·log@(g@(x)), each unknown still applied inside its own
argument, and `\nabla f(x)\log y` ∇@(f@(x))·log@(y) (user ruling 2026-09-29; Perl's greedy chain nests the second
function inside the first). A first-item function still nests (`\log\log x`), and a trig function still continues
the argument. Golden `tests/parse/opfunction_arguments#juxtaposed_operator_functions_are_separate_factors`.

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

**Explicit space before a group multiplies it** (user ruling Q6, 2026-10-01, 57cj.22; `semantics::space_before_a_group`):
a letter followed by `\,`, `\;`, `\ `, `~` before a group that is no argument list does not apply to it, everywhere — as
trig arguments (#367) and derivative operands (#374) already read — and the space ends an OPFUNCTION's or operator's bare
argument there too: `k\,(x-y)` k·(x−y), `\log k\,(x-y)` log(k)·(x−y), `\exp\phi\,(1-x)` exp(φ)·(1−x), `\nabla\phi\,(1-x)`
∇(φ)·(1−x), `a\,[b+c]` (were k@(x−y), log@(k@(x−y)), …; the 57cj.8 review's mine: 429 formulas in 140 of the 3,003 A/B
papers; 2605.09037 `r=a\,(1-e\cos f_e)`, 2605.25633 `c_1\,S\,(L+1)`). An argument list keeps the application, an evaluation point (`u\,(x,t)`, `u\,(x,0)=g(x)`, as the derivative
operand's `\partial_x u\,(0,t)`), and so does a name, a function a space does not part from its argument (`\mathrm{Unif}\,[0,1]`,
`\mathrm{sigmoid}\,(x)`, `{\rm Pr}\,[A]`, `\mathrm{cos}\,(\Omega t)`: 50 of the km24 A/B's first-cut changes; `is_a_one_letter_head`),
and so does a group whose content is only ever an argument — a condition, a relation, a semicolon parameter list (`p\,(x\mid y)`,
`P\,(X\in A)`, `\phi\,(x;\theta)`; `holds_an_argument_s_content`, 57cj.23, the merge review);
without the space nothing changes (`k(x-y)` k@(x−y)). km24 A/B (m60 → m63): 375 one-letter readings, e.g. 2605.00580
`\alpha\,(1-2\,s_i)` α·(1−2s_i), 2605.00224 `\beta\,[\Delta\log\pi_\theta-…]`. Guard
`tests/parse/rust_parse_additions.tex` ("Explicit space before a group multiplies it").

---
