# Oxidized Design — Math Parser & Grammar

[← OXIDIZED_DESIGN.md](../parity/OXIDIZED_DESIGN.md) · Marpa-style ambiguous grammar design + the numbered grammar-rule divergences.

> **Numbering note:** these `### N` numbers (`#16`, the design rules, and the grammar cluster `#7–#15`, `#17`, `#18` and `#63`, which held a second `#16` until 2026-11-15) are a SEPARATE sequence from the divergences in [OXIDIZED_DESIGN_DIVERGENCES.md](../parity/OXIDIZED_DESIGN_DIVERGENCES.md) and collide with them by value — kept verbatim because code refers to them (notably `#18` = "Speculative function application", the f(x)→apply decision). `#19`–`#62` are the math-parse entries moved here from the divergences file on 2026-11-14, each headed "formerly OD #N" (the code's `OXIDIZED_DESIGN #N` citations resolve through the stub there). See also divergence #4 (Marpa parser) and #15 (improved parses) in the divergences file.

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

### 63. Grammar: Bigop argument scope after invisible times (renumbered from the grammar cluster's second #16, 2026-11-15)

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

---

## Math-parse divergences moved from OXIDIZED_DESIGN_DIVERGENCES (2026-11-14)

Each keeps its old number as "formerly OD #N" (code comments cite `OXIDIZED_DESIGN #N`; the stub there points here).
These numbers continue this file's own sequence after #18. Inside a moved entry a bare `#N` (or "divergence #N") is
an OXIDIZED_DESIGN_DIVERGENCES number, as it was written; one of this file's says so ("this file's #18").

### 19. `\cdots` Role: ELIDEOP Instead of ID (formerly OD #3)

**Decision:** `\cdots` uses `role="ELIDEOP"` (Perl uses `role="ID"`).

**Rationale:** This enables dedicated grammar rules in the Marpa math parser
(e.g. `term mulop tight_term elideop => infix_apply_and_elide`) for better-structured
parse trees. The ID role is too generic for ellipsis operators.

**Impact:** Test XMLs must use `role="ELIDEOP"` for `\cdots`.

An ELIDEOP is also a factor (57bs, user ruling 2026-09-29), as Perl's ID is: it stands among juxtaposed factors and
as a MulOp/BinOp operand — `a_1a_2\cdots a_n` a₁·a₂·⋯·aₙ, `a\times\cdots\times b`, `x_{i_1\cdots i_k}`,
`(1-x)\cdots(1-x^n)` (1,035 formulas in 236 A/B papers were unparsed; 2605.10811, 2605.31453). Not a `factor_base`:
a trig bare argument, a limit-from and a differential take none; an OPFUNCTION's or operator's bare argument does, as
Perl's `aBarearg` takes an ID (`\nabla u\cdots` ∇@(u·⋯)) (57bx: `\max_{m}|a||b|\cdots|c|` max_m@(|a|·|b|·⋯·|c|), 2605.23673; `\log\cdots` log@(⋯)
was unparsed). A product ending in a bare ellipsis is no
right operand of a visible operation (`infix_apply_nary`), so an operation elided at the end keeps its elision reading
(`a\cdot b\cdots` a cdot b cdot cdots). Golden `tests/parse/ellipsis_products.tex`.

### 20. Marpa-Style Math Parser (formerly OD #4)

**Decision:** The math parser uses a highly ambiguous Marpa grammar instead of Perl's
hand-coded recursive descent parser.

**Rationale:** This is the primary research contribution of the Rust rewrite. The
approach is to be highly ambiguous in parsing but aggressively prune in semantics rules,
minimizing final parse count. This produces better-structured parse trees for complex
mathematical expressions.

**Impact:** Math parse trees differ structurally from Perl. This is active research;
math tests are deferred until the core engine is solid.

### 21. Improved Math Parses Over Perl (formerly OD #15)

**Decision:** When the Rust Marpa grammar successfully parses an expression that Perl's
Parse::RecDescent left unparsed, the Rust output is preferred if the parse is mathematically
correct. The expected test XML is updated to match Rust's improved output.

**Rationale:** The Marpa grammar is more powerful than Parse::RecDescent and can handle
expressions that Perl gives up on. Matching Perl's *failure* modes is not a goal — matching
Perl's *success* modes is. When Rust produces a better parse, that's an improvement.

**Process:** When a test fails because Rust produces a parsed structure where Perl had flat
unparsed tokens, the developer asks the user to confirm whether the Rust XML should be updated.

### 22. A fence split by TeX's null delimiter is re-balanced before it is given up on (formerly OD #67)

**Perl.** `\left( a+b \right.` is a fence whose right delimiter is *empty*, and
digestion emits no token for the `.` at all. When such a fence is split across an
alignment break — the standard way to break a long parenthesised expression:

```latex
\begin{align}
  H & = \frac{\hbar}{2} \left( \Omega_{IX} IX + \ldots \right. \nonumber \\
    & \quad \left. + \Omega_{ZX} ZX + \ldots \right).
\end{align}
```

each cell is left holding one unmatched delimiter. Perl's MathGrammar has no rule
for that, so it reports `not_parsed` (four warnings on the minimal case) and the
formula degrades to unstructured markup.

**Rust.** The XMath we build is byte-identical to Perl's — same `XMWrap` with an
unmatched `OPEN`, same bare `CLOSE` — so this is purely a parser difference. When
the grammar returns **zero** derivations, `parser.rs::balance_null_delimiters`
re-supplies the delimiter TeX says is there and retries once. The synthesized
`XMTok` carries the fence `role` but empty text, so it renders as nothing, which
is exactly `\right.`'s meaning; `tex=` reversions stay verbatim
(`\left(a+b\right.`). The cells then parse through the *existing* open/close
fence semantics as `delimited-(@(…)` / `delimited-)@(…)`.

**Why this is safe rather than a parity break.** It is gated on total parse
failure, so no formula that already parses can be re-read — the repair is
unreachable for them. That gate is load-bearing, not incidental: an earlier
unconditional version read the `⟩` of a ket `|f⟩` as a dangling close (its partner
is a `VERTBAR`, not an `OPEN`), prepended a bogus `(`, and broke formulae that had
been fine. Guarded by `tests/math/split_fence.tex`, which pins the ket case
alongside the split fence. Emitting *more* structure than Perl on input Perl
cannot parse never yields fewer errors than Perl. Witness: arXiv 2606.13010.

**What did NOT loosen.** Parser recovery stays OFF in both attempts. libxml's
recovery mode silently destroys author content — measured: `<b>a</b> <i>b</i>`
salvaged to just `<b>a</b>`, `a&nbsp;b` to `ab`, `a & b` to `a  b` — so malformed
markup is still rejected outright rather than quietly mangled. Perl agrees here:
`XML::LibXML->parse_string` defaults to `recover => 0`. Empty markup is not a parse
failure (it parses to zero nodes); `insert_xml` is the layer that reports it.

**The wrapper is invisible.** It is never returned, and never reachable: walking
UP from a top-level parsed node reports NO parent (`common::xml::is_parse_artifact`
covers both the wrapper and the parsed document node), so a script can neither see
`_lxfragment` nor splice it into the page with `insertXML(n.parent())`.

**Guards:** `common::xml::parse_fragment_tests::*` (siblings kept whole, bare text,
malformed still rejected, wrapper never returned, empty → zero nodes) and the
end-to-end `30_script_bindings::script_binding_macro_and_constructor_convert`
(`\rhfragment` parses two siblings, edits them while detached, inserts both, and
asserts `data-top="detached"` plus a blanket `!contains("_lxfragment")`).

### 23. A comma between digit groups is a thousands separator, not a list separator (formerly OD #70)

**Perl behaviour.** `$50,000$` parses as a two-item LIST — `list(50, 000)` (or,
after a relation, `>(absent, list(50,000))`). The number ligature that would join
them (`Base_XMath.pool.ltxml` L506-508) demands `$r ne 'PUNCT'` on the separator
— "Be paranoid about lists" — and a math-mode comma is ALWAYS `role="PUNCT"`, so
for `en`, where the thousands separator *is* the comma, that arm is unreachable
dead code. The content arm is then nonsense and even the presentation grouping is
wrong: `mrow(mn 50, mo ",", mn 000)` instead of a single `mn`.

**Rust behaviour (owner-directed 2026-07-25).** `$50,000$` is ONE number, 50000.
The default reading is **US** (comma groups digits); the **European** reading
(comma as DECIMAL separator, `$3,14$` = 3.14) stays available and is selected the
way it already was — by the document language, through the `DECIMAL_SEP` /
`THOUSANDS_SEP` maps, whose decimal arm never had the role guard and so always
worked. Real corpora contain both conventions, hence a default rather than a
single answer.

Merged: `50,000`→`50000`, `1,234,567`→`1234567`, `12,345,678,901`, `1,234.56`.
Left alone: `3,14` (two digits — the European decimal shape), `50,0001`,
`f(x,000)` (no NUMBER left of the comma), `(1, 2024)`, `(12, 3456)`.
`$(12, 345)$` DOES merge — genuinely ambiguous, and this is what "default US"
decides. Presentation becomes a single `<mn>50,000</mn>`.

**Where, and why not elsewhere.** A `DefRewrite` in the post-build `Rewriting`
phase (`base_xmath.rs`, `THOUSANDS_SELECT_XPATH`), not the ligature. Ligatures run
per-token from `Document::open_math_text_internal` while the document is being
built, so `get_next_sibling()` is None for every node — there is no right context
at all, and a merge-on-three-digits rule fires before a fourth digit can arrive.
Measured: it turned `$(1, 2024)$` into `12024` and `$(12, 3456)$` into `123456`.
By the Rewriting phase the ligature has already collapsed each digit run into ONE
token, so the group length is directly testable and those cases are rejected by
construction. Package-built number markup is excluded (`not(parent::ltx:XMWrap)`)
— siunitx/numprint construct their own `XMDual(semantic, XMWrap[...])` and are
already correct.

Golden updated: `tests/complex/si.xml` (three author-typed `$3,762$` cells).
Guards `cluster_thousands_separator_us_default`, `cluster_thousands_separator_eu`.
Witness: arXiv 2605.17646 (`population $ > 50,000$`).

### 24. Meta content in math text floats out of the Math, and an equation's footnotes render in HTML (Perl: stranded in `XMText`; equation-level notes not rendered) (formerly OD #272)

**Perl**: `\footnote`/`\index`/`\gls`/`\nomenclature` inside `\text{}` in math construct in the `ltx:text` that `\text` opens. That element admits them, so their `^` float (`floatToElement`, Document.pm:1052) stops there. `cleanup_XMText` (TeX_Math.pool.ltxml) then unwraps the `ltx:text` and leaves the marker as a direct `XMText` child. `XMText_model = (text | Inline.class | Misc.class)*` has no `Meta.class`, so the document is schema-invalid. The marker is also read into the formula: the footnote enters `text=` (`[b11footnote 1fn]`), and ryethesis's `\nomenclature` entries parse as a factor, `c² × [glossary]`. In an `align` cell, `addColumnToMathFork` (Base_XMath.pool.ltxml:880) clones any non-math element into the main branch's `XMText`, so the fork holds the footnote twice (a `.mf` copy). Separately, the XSLT renders "all of equation_model EXCEPT Meta" (LaTeXML-block-xhtml.xsl), so a footnote at equation level, a bare `\footnote` in `\[…\]` included, never reaches the HTML.
**Rust** (batch 56gx, user ruling 2026-09-23 "move them out of math"):
- `cleanup_math` first runs `float_meta_out_of_math` (`base_utilities/math_cleanup.rs`). Every outermost `Meta.class` element (LaTeXML-meta.rnc:17) inside the Math's `XMText` moves, in document order, to just after the Math, at the nearest ancestor level that admits it: the `p` for inline math, the `equation` for a display, the `td` for an align cell. This is where a bare `\footnote` in math floats. A `text`/`XMText` wrapper the move leaves empty is removed, so it does not parse as an empty term.
- `add_column_to_math_fork` does not clone Meta content into the main branch.
- The XSLT's two equation-content selects add `ltx:note`, and an aligned row's own notes render in its right padding cell (`eq-right`'s `notes` parameter).

Measured on the 11 affected s117 manuals: schema errors 7 → 0 (ribbonproofs, sidenotesplus, ryethesis), 0 core-XML words lost, HTML 0 words lost and 9 docs gained (the pdfcomment manuals' equation annotations, LaTeX_for_Undergraduates, ryethesis). **Guards**: `cluster_schema::meta_in_math_text_floats_out_of_the_math`, `footnote_in_display_math_text_floats_to_the_equation`, `footnote_in_display_math_renders_in_html`.

### 25. Letters after leading digits join into one identifier (formerly OD #348)

Perl's defect: KNOWN_PERL_ERRORS #364.

Perl's letters ligature (Base_XMath.pool.ltxml:443-458) reads back from the last letter while the fonts are `equals` and the text is `/^[0-9a-zA-Z]+$/`, and joins only when the run starts with a letter. A digit before `\mathrm` letters has an equal font, so it is read too and the whole run is refused: `10\mathrm{log}_{10}` is `l * o * g`, `2\mathrm{skew}` `s * k * e * w` (KNOWN_PERL_ERRORS #364) — the rendering is the same, the semantics (and a screen reader's reading) are not.

**Rust** (batch 57ae): the fonts compare as Perl's (`Font::perl_equals`), and when the run read back starts with digits, the letters after them join, the digits staying a number: `10`, `log`; `2`, `KL`; `5`, `mm`. A run starting with a letter joins whole, as Perl (`\mathrm{ab}2` is `ab2`, `\mathrm{a2b}` `a2b`). In the 3,003-paper A/B this keeps 71 formulas in 30 papers joined.

**Guard**: golden `tests/parse/math_lexemes.tex#letters_ligature_reads_back_through_a_digit`.

### 26. A spacing hint after a punctuation mark in a gathered row is padding (formerly OD #349)

Perl's defect: KNOWN_PERL_ERRORS #365.

Perl's `filter_hints` (MathParser.pm:417-491) makes a hint of 10pt or more (`\quad`) a virtual PUNCT unless the node before it is a PUNCT, reading that node's own `role` (:482-483), and passes a hint after an OPEN on to the next node, again by the node's own `role` (:445). In a gathered/split/multline row's content branch the nodes are XMRefs without a role, so `a=b,\quad c=d` there has two punctuations in a row and `(\quad x)` a punctuation inside its fence; Perl leaves both unparsed, where inline they are `formulae@(a = b, c = d)` and `x` (KNOWN_PERL_ERRORS #365).

**Rust** (batch 57af): the OPEN and PUNCT tests read the realized role (`get_grammatical_role`, an XMRef's target's), so the row reads as it does inline and the `\quad` is the comma's `rpadding` (on the XMRef too, as Perl pads a content XMRef). The rest follows Perl: the phantom gate (:443), APPLYOP taking no space (:458-461), a pending space — negative too — as the next node's `lpadding` (:466). Not ported: keeping an XMHint that an XMRef references (:453-457), carrying XML comments onto the neighbouring node (:427-437), and Perl's precision in summed widths (Rust records each hint's width at 0.1pt, `\!` as `width="-1.7pt"`, so `\!\!` sums to `-3.4pt` where Perl's is `-3.3pt`).

**Guard**: golden `tests/parse/aligned_content_branch.tex#content_branch_reads_its_delimiters`.

### 27. An evaluation bar inside `\|…\|`, `\left|…\right|` or a nested group (formerly OD #350)

Perl's `absExpression : <rulevar: local $forbidEvalAt = 1>` (MathGrammar:410) forbids the `f|_{x=0}`
evaluation bar (`moreFactors`, :259-262) anywhere inside an absolute value or norm, nested groups
included — a guard against reading one of a pair of identical bars as the evaluation bar
(`|\nabla a|_L|\nabla b|_L`).

**Rust** (batch 57am): the guard holds for the single-bar pairs where a bar can be misread (`|…|`,
`||…||`, `\big|…\big|`), and stops at a nested group, which pairs its own bars; `\|…\|` (one
token) and `\left|…\right|` (paired by the lexer) take an evaluation bar inside:
`\|u|_{\Gamma}\|`, `\left\|\left.\frac{\partial y}{\partial Z}\right|_{Z=0}\right\|`,
`\left|\mathcal{M}|_S\right|`, `|g(f|_{x=0})|` parse (2605.01526, 2605.04708, 2605.07463), where
Perl fails or reads `delimited-‖|`. And it is a preference, not a prune: the single-bar fences mark
the reading (`Meta::evaluation_bar_inside`, latexml_math_parser/src/semantics.rs) and the forest
keeps the trees with the fewest marks (`XM::prefer_fewest_evaluation_bars_inside`, semantics/tree.rs,
before the student pragmas), so a pair whose every reading nests the bar keeps it while the other
pairs of the formula read Perl's way — `|f(x)|_{0}^{1}|`, `\bigg|g\big|_{t=1}-h\bigg|\le C` is
|eval(g, t=1) − h| ≤ C, `|\nabla a|_L|\nabla b|_L+|f(x)|_0^1|` keeps |∇a|_L·|∇b|_L (2605.26054,
2605.02499; Perl fails). Only where no reading without the mark exists: before `+ c` the bar pair is
a conditional, as in Perl — `|f(x)|_0^1|+c` is `conditional@(((|f(x)|)_0)^1, + c)` (57al read
|eval(f(x), 0, 1)| + c), and `|f(x)|_0^1|+|\nabla a|_L|\nabla b|_L`
`conditional@(((|f(x)|)_0)^1, + |∇a|_L·|∇b|_L)` (Perl fails) — a residual, the price of Perl's rule. A conditional bar takes no condition with a bar reading of its own (Perl's
`ExpressionsNoBars`, MathGrammar:267-268) nor a relation missing an operand (`vertbar_modifier`).

**Guard**: `perfect_kernel_batch57::evaluated_at_stays_outside_absolute_bars` (repro
`math-parse/evaluated_at_stays_outside_absolute_bars`).

### 28. An OPFUNCTION applied to a group takes the scripts after it (formerly OD #351)

Perl's `addEasyArgs` (MathGrammar:571-576) ends an OPFUNCTION's Factor at the group's close; a
script after it has no base, and the formula is unparsed (`\log(n)^2`, `\operatorname{Var}(X)_k`,
`\max(a,b)^2`, `\exp(x)^{-1}`).

**Rust** (batch 57am): the group application takes the scripts
(`scripted_opfunction_application`, latexml_math_parser/src/grammar/builder.rs): (log@(n))²,
(Var@(X))_k, (max@(a,b))², also after a factor (`\lambda^2\log(|A|)^2` λ²·(log@(|A|))²); a bare
OPFUNCTION before a group always takes it (`addEasyArgs`), never multiplies it. The rule it
replaces (`opfunction tight_term`) read `\log(n)^2` as log@(n²). About 359 formulas in 112 of
the 3,003 A/B papers (2605.24357, 2605.01408). Only an unscripted head: a scripted one reads as a
limit, its operand the scripted group however many scripts (`scripted_group_apply`) —
`\min_w(y-w)^2` is min_w@((y−w)²), `\min_j(y_j)_{j=1}^{n}` min_j@(((y_j)_{j=1})^n) (2605.04340,
2605.23087, 2605.19263) — a base subscript too: `\log_2(n)^2` is (log₂)@(n²), as before 57am (Perl
fails). Not a bar pair, which is no `addEasyArgs` group but an `aBarearg` (:330) that starts the
greedy bare argument: `\log|z|^{2}dz` is log@(|z|²·dz), as Perl (`group_apply`; `||x||` too,
#353); `\lvert…\rvert` is an OPEN/CLOSE pair, so `\log\lvert x\rvert^2` is (log@(|x|))². Not a
TRIGFUNCTION, whose argument is `trig_arg`: `\sin(x)^2` stays sin@(x²). Known cost: a power meant
for the group reads outside it — `\log(2\pi e)^{n}` (the Gaussian entropy's ½log((2πe)ⁿ|Σ|)) is
(log@(2πe))ⁿ, was log@((2πe)ⁿ) (Perl fails both).

**Guard**: golden `tests/parse/opfunction_arguments.tex#opfunction_group_application_takes_its_scripts`.

### 29. A differential anywhere inside an integral's operand (formerly OD #352)

Perl licenses a differential `d` (MathGrammar `IntFactor`'s `diffd`, :643-646) only among the
top-level factors of an integral's operand (`addIntOpArgs`/`moreIntOpArgFactors`, :633-638): in a
fenced integrand, or under a big operator nested in the integral, the `d` is a plain unknown —
`\oint(P\,dx+Q\,dy)` is `contour-integral@(P * d * x + Q * d * y)`, `\int\sum_i f_i\,dx`
`integral@((sum _ i)@(f _ i * d * x))`.

**Rust** (user ruling 2026-09-28: "a clear semantic improvement"): a `d` in a clear integration
context reads as the differential wherever it stands in the integral's operand —
`contour-integral@(P * differential-d@(x) + Q * differential-d@(y))`,
`integral@((sum _ i)@(f _ i * differential-d@(x)))`. Outside an integrand it is to stay a plain `d`,
as Perl (`\int f=1,\qquad dx=2` is `d * x = 2`; Rust still reads `differential-d@(x) = 2` there). The faithful licensing port (repro
`math-parse/differential_needs_its_integrand`, RED) keeps this scope.

**Guard**: golden `tests/parse/integrals_and_differentials.tex#differential_is_no_big_operator` (the kept
differentials); the licensing port adds the fenced and nested-sum rows.

### 30. A norm written `||x||` is a bare item, as `\|x\|` is (formerly OD #353)

Perl's `SINGLEVERTBAR SINGLEVERTBAR absExpression SINGLEVERTBAR SINGLEVERTBAR` norm (MathGrammar:294)
is a Factor, not an `aBarearg` (:329-330, only `VERTBAR absExpression VERTBAR`), so an OPFUNCTION
before it multiplies it: `\log||x||_2^2` is `logarithm * ((norm@(x)) _ 2) ^ 2`,
`\min_x||Ax-b||_2^2` `minimum _ x * …`, while `\log\|x\|^2` is `logarithm@((norm@(x)) ^ 2)` — the
same norm, split by the spelling of its bars.

**Rust** (batch 57am): every fence built from VERTBAR tokens — `|…|`, `\|…\|`, `\left|…\right|`,
the stretchy norms (kerned or not, `\log\left|\left|x\right|\right|_2^2` is `logarithm@(((norm@(x)) _ 2) ^ 2)`),
and `||…||` — is marked (`Meta::bar_fence`, latexml_math_parser/src/
semantics.rs) and is a bare item of a function's argument, never the group of its easy arguments:
`\log||x||_2^2` is `logarithm@(((norm@(x)) _ 2) ^ 2)`, `\min_x||Ax-b||_2^2`
`(minimum _ x)@(((norm@(A * x - b)) _ 2) ^ 2)`, `\arg\min_w||y-Xw||^2` likewise — as 57al read them;
an operator's bare argument too (`bare_abs`): `\nabla u||v||` is `nabla@(u * norm@(v))`, as
`\nabla u\|v\|` is (Perl `nabla@(u) * norm@(v)`).
Four single bars around a term are the norm, never nested absolute values (Perl's ordered choice,
:294 before :299). The mark is the fence's own, so a `split` row, whose bars read through XMRefs,
reads as the inline formula. A `\left`/`\right` double bar (`\left\|`, `\left\Vert`,
`\left\lVert`) is a bar pair as in Perl, which re-roles its ∥ a VERTBAR (TeX_Math.pool.ltxml:755):
the markup keeps its OPEN/CLOSE role, the lexer reads it as the stretchy bars (latexml_math_parser/
src/util.rs), so `\exp\left\|x\right\|^2` is `exponential@((norm@(x)) ^ 2)`, as Perl. A bare
`\lvert…\rvert`, `\lVert…\rVert` are OPEN/CLOSE delimiters, groups as in Perl
(`\log\lvert x\rvert\,y` is `logarithm@(absolute-value@(x)) * y`). The bar pairs are a nonterminal
of their own (`bare_abs`, grammar/builder.rs), which a function's group application
(`group_factor`) does not include, so each has one derivation after a function.

**Guard**: goldens `tests/parse/opfunction_arguments.tex#opfunction_group_application_takes_its_scripts`,
`tests/parse/bar_pairs.tex#left_double_bar_is_a_bar_pair`.

### 31. Three kerned `\left|` bars are one operator-norm delimiter (formerly OD #354)

Perl reads stacked `\left|` bars (`\left\vert`, the `\vertii`/`\vertiii` idioms) with its bar grammar:
two around two are `SINGLEVERTBAR SINGLEVERTBAR absExpression …` (MathGrammar:294), `\vertii{x}` is
`norm@(x)` with a merged `‖`; three are a norm around an absolute value, `\vertiii{x}` is
`norm@(absolute-value@(x))`, kerned or not. (The `\left\lvert` spelling lexes by its name, no
`SINGLEVERTBAR`, and nests absolute values.)

**Rust** (task #263): two `\left|` around two `\right|` are one norm as Perl, kerned or not
(`stretchy_norm_fenced`, latexml_math_parser/src/semantics.rs; until 57am the pair needed the kern and
read `\left|\left|x\right|\right|` as |(|x|)|). Three bars stacked with a negative kern each side (the
kern folded into the bar's `rpadding`) are one delimiter pair, `operator-norm@(x)` with `⦀`
(`stretchy_triple_norm_fenced`) — the divergence; unkerned they are Perl's
`norm@(absolute-value@(x))`. The `\left\lvert` spelling lexes as the same stretchy bar (util.rs reads
its glyph), so it stacks too — a divergence: `\left\lvert\left\lvert x\right\rvert\right\rvert` is
`norm@(x)` (Perl `absolute-value@(absolute-value@(x))`), three unkerned `norm@(absolute-value@(x))`
(Perl three absolute values); the stack renders as ‖x‖, and |(|x|)| is no one's intent. A merged glyph takes the padding after its stack's last bar,
the kerns between the bars being the glyph's own (57am review round 7). Only single bars stack: a
`\left\|` inside a stack is a bar pair of its own (`\left|\!\left\|A\right\|\!\right|` is
`absolute-value@(norm@(A))`, as Perl). One level of Perl's `absExpression` nesting.

**Guard**: golden `math/norm_kerned_delims` (2211.13044 §S4.Ex17; `\vertiii{x}\le C\vertiii{y}` is
`operator-norm@(x) <= C * operator-norm@(y)`), golden `tests/parse/bar_pairs.tex#left_double_bar_is_a_bar_pair`.

### 32. A placeholder between `\left`/`\right` bars is its norm (formerly OD #355)

Perl's `absExpression` needs a term (MathGrammar:410-412), so `\left\|\cdot\right\|_\infty` — the norm
of an argument slot — leaves the formula unparsed.

**Rust** (batch 57am): a placeholder between `\left`/`\right` bars is the norm or absolute value of
the slot, as the placeholders fenced in parentheses are (`f(\cdot)`, `\langle\cdot,\cdot\rangle`):
`\left\|\cdot\right\|_\infty` is `(norm@(cdot)) _ infinity`, `(A,\left\|\cdot\right\|_\infty)`
`vector@(A, (norm@(cdot)) _ infinity)` (a normed space, #371) — what 57al read through the OPEN/CLOSE placeholder rule
before the lexer made a `\left\|` pair bars (#353); a stack around the slot is its norm, kerned or not
(`\vertii{\cdot}` and `\left|\left|\cdot\right|\right|` are `norm@(cdot)`), three kerned bars its operator norm
(#354). The slot is a MulOp or BinOp (`\cdot`,
`\bullet`; `\left|\times\right|` is `absolute-value@(times)`) — `\left|+\right|`, `\left|=\right|` stay
unparsed, as in Perl. Sided bars only: between unsided ones, `|x|\cdot|y|` would offer a `|\cdot|`
reading of its own, so `\|\cdot\|` and `|\cdot|` stay unparsed, as in Perl.

**Guard**: golden `tests/parse/bar_pairs.tex#stretchy_bars_around_a_placeholder` (57am7 A/B: 78 formulas,
30 papers; 2605.00709, 2605.05645, 2605.07463).

### 33. A sided bra and ket around an operator holding bars are one quantum-operator-product (formerly OD #356)

Perl's `ketExpression` forbids bars (MathGrammar:401), so between `\left\langle a\right|` and
`\left|e\right\rangle` an operator holding bars cannot be the middle of a braket: Perl reads
`\left\langle a\right|b\left|c\right|d\left|e\right\rangle` as `bra@(a) * b * absolute-value@(c) * d * ket@(e)`
and `\left\langle\Psi\right|\left|\Phi\right|\left|\Psi\right\rangle` as `delimited-⟨⟩@(Psi * norm@(Phi) * Psi)` —
a norm of two bars TeX paired apart.

**Rust** (batch 57ap): the sided bars fix the pairing — the `\right|` closes the bra, the last `\left|` opens the
ket, the pairs between are absolute values — and the bracket's middle is any expression, so both are the matrix
element Dirac notation writes: `quantum-operator-product@(a, b * absolute-value@(c) * d, e)`,
`quantum-operator-product@(Psi, absolute-value@(Phi), Psi)`. Unsided bars are unchanged.

**Guard**: golden `tests/parse/bar_pairs.tex#stretchy_bar_divides_a_conditional` (the "divergence #356" rows).

### 34. In braces, a `\left|` pairs its `\right|` evaluation bar (formerly OD #358)

Perl's `scriptFactorOpen` tries the set-builder bar before the bar pair in braces (MathGrammar:486-491, `FormulaNOBar
suchThatOp Formulae`), so `\left\{a\left|f\right|_{x=0}\right\}` is `conditional-set@(a, evaluated-at@(f, x = 0))` — the
`\left|` that TeX paired with the `\right|` read as the set-builder bar.

**Rust** (57ao and before; kept deliberately since 57ap/57ar): TeX's pairing — `set@(a * (absolute-value@(f)) _ (x = 0))`;
a `\left|` divider whose next item evaluates at a stretchy bar is refuted (`left_bar_pairs_an_evaluation_bar`). In
parentheses and brackets Perl pairs them too (`Factor` before `evalAtOp`, :257-263).

**Guard**: golden `tests/parse/bar_pairs.tex#stretchy_bar_divides_a_conditional` (the brace row).

### 35. A `\middle|` divides a conditional (formerly OD #359)

Perl's conditional reads a VERTBAR (`evalAtOp ExpressionsNoBars`, MathGrammar:261-268); a `\middle|` is role MIDDLE
(etex `\middle`), so `P\left(A\middle|B\right)` is unparsed.

**Rust** (before 57ap; recorded 57ar): the `middle_bar` fence rules read it — `P@(conditional@(A, B))`, the `\middle|`
the author wrote to divide.

**Guard**: golden `tests/parse/bar_pairs.tex#stretchy_bar_divides_a_conditional` (the `\middle|` rows).

### 36. A closing mark outside a script is presentation (formerly OD #361)

Perl's `Subscript`/`Superscript` start rules alone end in `endPunct(?)` (MathGrammar:84-92), which wraps a
script's content in a one-item list (`NewList`, MathParser.pm:1463-1475): `x_{a,}` x _ (list@(a)). A formula
parsed with `Anything,` has its closing marks set aside first (MathParser.pm:657-670); an `XMArg` or `XMWrap`
parsed with plain `Anything` (MathGrammar:64-82, MathParser.pm:384-388) has no closing mark, so `\boxed{a+b.}`, `\frac{a,}{b}`,
`k\xrightarrow{a.}o` and `a\mathrel{=,}b` are unparsed in Perl.

**Rust** (rc59b; landed with the batch that records it): the grammar admits a closing mark in every container
(`statements end_punct => postfix_embellished`); a mark that closes a script builds Perl's list, any other is
presentation — `\boxed{a+b.}` a + b, `a\mathrel{=,}b` a = b — where the mark's value once chose `list@` for any
container (`list@(a + b)`; 763 formulas in 82 papers of the 57ar A/B, 709 of them `\boxed{… .}`). A mark followed by a
wide space reads `fragments@(…)` where the accidental one-item list swallowed what followed it (`a=b.\quad c=d`,
`\boxed{x=0,}\qquad y`: was `list@(…)`; Perl unparsed; `a=b\quad c=d` reads `formulae@(…)`, the `statements
end_punct` result feeding no `formulae` rule).

**Guard**: golden `tests/parse/trailing_punctuation.tex`.

### 37. A reversed-bracket pair is an open interval (formerly OD #362)

Perl's grammar has no rule for a reversed bracket pair: `]0,1[` and `]0;1[` are unparsed (`isMatchingClose`,
MathParser.pm:1379-1384, over `%balanced` :1348-1356, pairs `[` with `]` only), and its `%enclose2` table
(:1368-1373) names comma pairs alone.

**Rust**: `interval_term` admits `rbracket expression punct expression lbracket` (`grammar/builder.rs`, the French notation
for an open interval; expression endpoints since 57bj, `]-1,1[`), named `open-interval` with a comma or a `;` — the reversed pair marks the interval, and `;` is the
French separator (`]0;1[`). Every other pair is named as Perl's tables name it: a comma pair an interval, any other
separator a `list` (57ba) — `[0;1]` is a closed interval in French papers but a vertical concatenation in machine-
learning ones (`[\mathbf{a};\mathbf{b}]`, 2605.00423), so the neutral `list`.

**Guard**: golden `tests/parse/fenced_lists.tex` (`(a;b]`, the `interval` action's `list`).

### 38. A function of a named fence takes the fence as one argument (formerly OD #363)

Perl's grammar reads a function's arguments before it names any fence: `addEasyArgs` (MathGrammar:570-577) takes
`OPEN Argument (argPunct Argument)* balancedClose` for any balanced pair (`isMatchingClose`, MathParser.pm:1379, over
`%balanced` :1348-1356) and `ApplyDelimited` drops the delimiters. A delimiter pair that names a function of its
content is dropped with them, and a set-builder's bar or colon splits it:

| Input | Perl | Rust |
|---|---|---|
| `\log\lfloor x\rfloor` | `logarithm@(x)` | `logarithm@(floor@(x))` |
| `\log\lceil x\rceil` | `logarithm@(x)` | `logarithm@(ceiling@(x))` |
| `\log\lvert x\rvert` | `logarithm@(x)` | `logarithm@(absolute-value@(x))` |
| `\log\lVert x\rVert` | `logarithm@(x)` | `logarithm@(delimited-∥∥@(x))` |
| `\max\{x\mid x>0\}` | `maximum@(x * ket@(x) * 0)` | `maximum@(conditional-set@(x, x > 0))` |
| `\max\{x:x>0\}` | `maximum@(x colon x > 0)` | `maximum@(conditional-set@(x, x > 0))` |
| `\Pr\left(A\middle\|B\right)` | `Pr@(A, B)` | `Pr@(conditional@(A, B))` |
| `\max\langle a,b\rangle`, `\max_m\langle t,b_m\rangle` | `maximum@(a, b)` | `maximum@(delimited-⟨⟩@(list@(a, b)))` (an inner product; 57bf) |
| `\log\langle Z\rangle`, `\max\langle x\rangle` | `logarithm@(Z)` | `logarithm@(delimited-⟨⟩@(Z))` (an average; 57bf) |

**Rust** (57bb; `fenced_tuple_items`, semantics.rs): a function takes the arguments between a matched pair of grouping
delimiters — parens, brackets, braces; angle brackets are an inner product or an average, one argument whatever
they hold (57bf; 95 formulas in 38 papers of the A/B, 2605.02896 `\max\langle Q,X\rangle`, 2605.20551) — around a
grouping fence (`delimited-…`, `list`, `set`, an
interval, a paren `vector`); any other fence is one argument. `|x|` agrees with Perl either way (bars are an
`aBarearg`: `logarithm@(absolute-value@(x))`). An operator head, bare, scripted or nested, takes them alike (57bf, 57bh):
`\nabla_X[Y,Z]` — a covariant derivative of a Lie bracket — reads Perl's `(nabla _ X)@(Y, Z)`, two arguments where
57be kept `closed-interval@(Y, Z)` one; neither is a commutator. Kept as Perl's though not the intent (SYNC): a function
before an operator's application multiplies it, `\operatorname{stf}\nabla(a-b)` `stf * nabla@(a - b)`,
`\min_h\operatorname*{R}(h,P)` `minimum _ h * R@(h, P)` (57bh restored the parse 57bf lost).

**Guard**: golden `tests/parse/fenced_lists.tex` ("A function takes the arguments between any delimiters":
`\log\lfloor x\rfloor`, `\max\{x\mid x>0\}`, `\log\langle Z\rangle`), `tests/parse/operator_application.tex`
("An operator takes the arguments between delimiters").

### 39. A fenced conditional's given part and its joint part are lists (formerly OD #364)

Perl's grammar reads a paren group's comma list before the conditional bar: `p(x,y|z)` is
`p * open-interval@(x, conditional@(y, z))` — the bar binds only the last item, and the pair is named an interval.
Its brace form reads the joint whole: `\{x,y|z\}` `conditional-set@(list@(x, y), z)`.

**Rust**: a fenced conditional takes the whole list on either side of the bar, `conditional@(list@(x, y), z)` — the
joint of x and y given z, as the notation means — and the given part is a list too, `p(x|y,z)`
`conditional@(x, list@(y, z))` as Perl (57bd: both had been renamed `open-interval` by `rename_fenced_lists`). An
unfenced `a,b|c` still reads `list@(a, conditional@(b, c))`, as Perl (`vertbar_modifier_listlhs`). A list on
both sides at once reads the joint given both, `p(x,y|z,w)` `conditional@(list@(x, y), list@(z, w))` (57bh; Perl
`open-interval@(x, conditional@(y, list@(z, w)))`); the brace form `\{x,y|z\}` as Perl. A list holding a relation
is Perl's `formulae` beside the bar as in a fence (`FormulaNOBar suchThatOp Formulae`, MathGrammar:487-491):
`\{x : a<1, b<2\}` `conditional-set@(x, formulae@(a < 1, b < 2))`; `\mathbb{E}[Y\mid A=1,X]`
`E@(delimited-[]@(conditional@(Y, formulae@(A = 1, X))))` (`E` an unknown applied, #18), which Perl garbles; a
paren conditional's list too, `\Pr(X=1,Y=2|Z=3)` `Pr@(conditional@(formulae@(X = 1, Y = 2), Z = 3))` (was `list@`).
Brackets and braces take a list on either side, and a brace's colon only (Perl's `suchThatOp`, MathGrammar:499-501):
`\{x,y|z,w\}` `conditional-set@(list@(x, y), list@(z, w))` and `\{x,y : x<y\}` as Perl, `[a,b|c]`
`delimited-[]@(conditional@(list@(a, b), c))` (Perl `closed-interval@(a, conditional@(b, c))`); `\{\Gamma\vdash A,B\}` stays
unparsed (Perl `set@(Gamma proves list@(A, B))`). Open: `\middle|` sets with a list (`\left\{x,y\middle|z\right\}`), a
multirelation item (`\{i : 1\le i\le n, i\ne j\}`, a `formula_list` limit), and a plain paren list holding a relation,
still `vector@(…)` where Perl says `formulae@(…)` (SYNC, "Application residuals after 57bf–57bh").

**Guard**: goldens `tests/parse/bar_pairs.tex` ("A list beside a conditional bar is the conditional's"),
`tests/parse/functions.xml` (`p(x,y|\Theta)`).

### 40. A script that is one float script alone is a script of an absent base (formerly OD #365)

A script whose argument is a lone float script — `F^{{}^{\prime}}`, `H_{{}_{\mathrm{I}}}`, `(a-b)^{{}^{\prime}}` —
reaches Perl's parser as a one-node argument, which `parse_single` passes through unparsed (MathParser.pm:680-681): the
output keeps the raw `<XMApp role="FLOATSUPERSCRIPT">`, text `F ^ ^prime`. A float followed by more
(`x^{{}^{\prime}\prime}`) fails Perl's `rule 'Superscript'` (TeX_Math.pool.ltxml:501-516).

**Rust** (57bj; `lone_float_script`, grammar/builder.rs): the script argument may be one float script, built as
`NewScript(absent, ′)` (the shape of MathGrammar:79-80), so `F^{{}^{\prime}}\leq F` reads `F ^ ^ prime absent <= F` —
a parsed structure where Perl keeps raw markup, the float nested as written; a float followed by more stays unparsed, as
in Perl.

**Guard**: golden `tests/parse/rust_parse_additions.tex` ("A script may be a float script alone").

### 41. A colon list between delimiters is a list (formerly OD #366)

Perl reads a colon between a fence's items as a metarelation over the whole content (`OPEN Formulae CLOSE` with a
`metarelopFormula`, MathGrammar:69, :118-125; one item fenced, MathParser.pm:1412-1415): `[a:b]`
`delimited-[]@(a colon b)`, `(x:y:z)` `x colon y colon z`; a fenced metarelation is a statement, so `[K:\mathbb{Q}]=2`
and `x\in[a:b]` are unparsed.

**Rust** (57bm; `colon_list`/`colon_terms`, grammar/builder.rs): a colon list between brackets (expression items) or of
three or more terms between parentheses is a `list`, as the two-item `(a:b)` already was (`fence` names a non-comma
separator `list`, the default of Perl's `Fence` tables, MathParser.pm:1405-1409): `[a:b]` `list@(a, b)`,
`[x_0:\dots:x_n]` `list@(x _ 0, dots, x _ n)`, `(x:y:z)` `list@(x, y, z)`, and a factor — `list@(K, Q) = 2`,
`x element-of list@(a, b)`, `c ^ (list@(1, t))`. Nothing in the syntax tells an index `[G:H]` from a range `[1:t]` or
a projective point, so the neutral `list`; a function takes the list whole, `\max[a:b]` `maximum@(list@(a, b))`
(Perl `maximum@(a colon b)`). A paren item may carry a sign, `(1:-1:0)` `list@(1, - 1, 0)`, but no sum: a sum between
colons in parentheses is a tensor double contraction (`(\nabla x:\nabla y-\nabla z:\nabla w)`, 2605.21445, 2605.01156,
2605.09779), no list; it stays unparsed, as does `(a+b:c:d)`, which Perl reads `a + b colon c colon d`. Witnesses
2605.14715, 2605.08808, 2605.01646, 2605.25087, 2605.00473 (~216 formulas / 52 papers were unparsed).

**Guard**: golden `tests/parse/fenced_lists.tex` ("A colon list between delimiters is a list").

### 42. Explicit space or a differential `d` ends a trig function's bare argument (formerly OD #367)

Perl's trig bare argument is greedy (`trigBarearg : aTrigBarearg moreTrigBareargs`, MathGrammar:340-357): every
following atom, identifier, unknown or number joins it, whatever spacing the author wrote — `\sin\theta\,d\theta`
`sine@(theta * d * theta)`, `\sin\theta_W\,C_{uB}` `sine@(theta _ W * C _ (u * B))`, `\sinh r_u\,\mathbf a_u`,
`d\cos\theta_1 d\cos\theta_2` `d * cosine@(theta _ 1 * d) * cosine@(theta _ 2)`.

**Rust** (57bo; `trig_argument_juxtaposition`, `ends_with_space`, `is_differential_d`, semantics.rs): a juxtaposed
item does not join the argument after an item carrying explicit space — a positive `rpadding`, which
`filter_hints` folds from `\,`, `\:`, `\;`, `~`, `\ ` or a `\kern`/`\hspace` under 10pt onto the token or script
before it (a wider space, `\quad`, splits the formula as a PUNCT; a net zero or negative one, `\!`, is none) — nor
when it is the differential letter `d`; the greedy-argument prune (`leaves_a_trig_bare_argument`) does not fire
there: `sine@(theta) * d * theta` (with or without the space), `sine@(theta _ W) * C _ (u * B)`, `sine@(x) * y`
for `\sin x\;y`, `d * cosine@(theta _ 1) * d * cosine@(theta _ 2)`, `dA=\sin\theta d\theta d\phi` sin@(θ)·dθ·dφ.
Otherwise the argument stays Perl's greedy chain (`\sin 2x` sine@(2x)); `\sin kd` reads sin@(k)·d. Witnesses
2605.12033, 2605.03429, 2605.00265, 2605.10520, 2605.26406 (~90 formulas read wide across a space).

57bq (user direction 2026-09-29: prune a compound trig argument by the types inside it): one predicate,
`ends_trig_argument`, answers both actions, so exactly one reading survives. Besides space and `d` (a space before a
number sits in the number token's own text, `\cos\theta\;1`), an item ends the argument when it carries a mark no
scalar angle does (`non_scalar_mark`, read from the token's decoded `_font` and the built atom's shape): bold; a
calligraphic, script, blackboard, fraktur or sans-serif family; an upright Latin letter; a `\vec`/`\overrightarrow`,
`\hat`/`\widehat` or `\dot`/`\ddot` accent; a Leibniz derivative; a built atom holding a trig function; an adjoint
or transpose superscript (`\dagger`, `\top`, `\intercal`, upright or sans-serif `T`) — unless the argument so far
is coefficients only — numbers, π, a fraction or root of numbers (`\sin 2\mathcal P` sine@(2·P),
`\cos 2\pi\mathbf k\cdot\mathbf r`) — or already holds an item of the same mark (`\cosh\mathcal K\mathcal S`, an
all-upright document). A Leibniz fraction is read from its parsed parts (`\frac{d\theta}{dt}`,
`\frac{\partial^2u}{\partial x^2}`; its denominator a product or application, not `\frac{d_1}{d_2}`); a split
row's items through their XMRefs. `\cos\phi_m\vec e_{x_m}` cos@(φ_m)·e⃗_{x_m}, `\sin\theta\mathrm P_1` sin@(θ)·P_1,
`\sin\theta\dfrac{\partial Y}{\partial\theta}` sin@(θ)·∂Y/∂θ, `\cos\theta A^\dagger` cos@(θ)·A†; `\tilde`, `\bar`,
Greek and capital italic letters name angles too and join (`\cos\omega\tilde t`, `\cos\omega T`). Witnesses
2605.31180, 2605.28946, 2605.14528, 2605.15451, 2605.20574, 2605.09444, 2605.14924, 2605.26410. Known splits against
intent: `\cos\Omega\hat t` (2605.03417, a dimensionless t̂); a font switch around the whole formula,
`\mathbf{\cos\omega t}`, `\mathrm{\cos\omega t}` (the function keeps its operator font and Greek letters keep
theirs, so the DOM cannot tell it from `\cos\omega\mathbf t`). No evidence, greedy as Perl: `\cos\theta\sigma_x`,
`\sin\theta_k D_{x_k}f`, `(r\sin\theta\Omega)^2`, `\cos\omega_1X_1`.

57cj.7, 57cj.8 (the 57cj.6 and 57cj.7 reviews; `ends_with_space` → `trailing_padding`, `trig_letter_application`): the
space after the argument so far is its last token's, found down its right edge — through a product, a derivative's
operand and a bare application's argument however deep (`\sin\partial_x u\,v` sin@(∂_x u)·v, `\sin\partial_x 2\,\partial_y
u\,v` sin@(∂_x(2·∂_y u))·v, `\sin\log u\,v` sin@(log u)·v), and through a group, or an application to one, to its closing
delimiter: `\sin f(x)\,y` sin@(f(x))·y, `\sin\Gamma(x)\,y`, `\sin\log(x)\,y` sin@(log(x))·y, `\sin\exp\bigl(x\bigr)\,y`,
`\sin\log\left(x\right)\,y`, `\sin\log[x]\,y`, `\sin\max(a,b)\,y`, `\sin\partial_x\log(u)\,v`, where the argument took
y before; a space inside the delimiters ends nothing (`\sin\log(x\,)y` sin@(log(x)·y)). Formulas unparsed before now
parse: `\sin\log|x|\,y`, `\sin\log\|x\|\,y`, `\sin\exp x\,y`, `\sin\arcsin x\,y`, `\sin\operatorname{tr}A\,B`,
`\sin\ln\frac{x}{2}\,y`, `e^{i\sin\log x\,t}`. A letter's application to a group (#18) does not reach across the space
that ends the argument: `\cos\phi\,(1-x)` cos@(φ)·(1−x), `\sin k\,(x-y)` sin@(k)·(x−y), `\cos\omega\,(k)\,t` cos@(ω)·k·t
(`\cos\phi(1-x)` stays cos@(φ(1−x)), `\cos\omega(k)\,t` cos@(ω(k))·t — a dispersion phase may mean cos@(ω(k)t), but the
rule reads the space). Witness 2605.29683 A1.E17 `a\cos\phi\,\bigl(10-\cos(6\theta)\bigr)\,r^{6}`: cos@(φ@(10−cos 6θ)·r⁶)
before 57cj.7, cos@(φ@(10−cos 6θ))·r⁶ in 57cj.7, a·cos@(φ)·(10−cos(6θ))·r⁶ now; its delta A/B's other readings, all this
class: `\sin^{2}\beta\,\big(F(\dots)-F(\dots)\big)` (sin²)@(β)·(F−F) (2605.11097), `-q\cos\theta~(\bar{\mathcal F}=\dots)`
(2605.15566), `\cosh^{2}Z\,(dZ^{2}+d\varphi^{2})` (cosh²)@(Z)·(dZ²+dφ²) (2605.27600). A derivative inside the argument ends
where the argument ends (#374). Open (awaiting a ruling): a plain argument's coefficient run before a space ends there,
`\cos 2\,\theta` cos@(2)·θ, `\sin 2\pi\,ft` sin@(2π)·f·t, while inside a derivative it does not (#374).

57cj.9 (the 57cj.8 review; latent, its probes and Perl's own readings): the argument takes an OPFUNCTION's application as
an OPFUNCTION's argument does — Perl's `aTrigBarearg : preScripted['OPFUNCTION'] addOpFunArgs` (MathGrammar:341-343),
where the old item was one unscripted OPFUNCTION applied to one factor (`opfunction factor`), so Rust left unparsed what
Perl parses (`trig_arg += opfunction_application | bound_application | scripted_opfunction_application =>
trig_function_item`): `\sin\log_2 x` sin@(log₂(x)), `\sin\log^2 x`, `\cos\ln^2 x`, `\sin\max_i x_i` sin@(max_i(x_i)), `\sin\log 2x`
sin@(log(2x)), `\sin\log uv`, `\sin\operatorname{tr}AB`, `\sin\ln\frac{x}{2}y`, `\sin\log\log x`, `\sin\log\max(a,x)`,
`\sin\log\partial_x u`, `\sin\log|x|y`, and `\sin\log_2(x)` sin@(log₂(x)), which read sin@(log₂)·x (an unapplied OPFUNCTION
ending the argument takes what it applies to, `leaves_a_trig_bare_argument`); a group application takes its scripts
(#351: `\sin\log(x)^2` sin@((log x)²), was sin@(log(x²))). Its bare argument ends where the trig argument ends
(`crosses_within`, `ends_a_trig_argument_within`): `\sin\log u\,v` sin@(log u)·v, `\sin\log_2 x\,y`, and before a trig
function, as juxtaposed trig functions are separate factors (`\sin\log x\cos y` sin@(log x)·cos y, where alone `\log x\cos y`
is log@(x·cos y)); a mention of a bound head's variable continues it (`\sin\max_i u_i\,v_i` sin@(max_i(u_i·v_i)), the head's
scope following its variable, user ruling 2026-09-30). A letter's postfixed application to a group does not reach across
the space either (`trig_letter_postfixed`: `\cos\phi\,(1-x)!` cos@(φ)·(1−x)!, `\sin^2\phi\,(1-x)!`). A limit-type
operator's application the trig function takes (#390) keeps Perl's product when its operand crosses the argument's end
(`\sin\det A\,y` sin·det@(A·y)).

57cj.10 (the 57cj.9 review): the OPFUNCTION's bare chain in a trig argument is built without a trig function after its
first item (`trig_op_bare_arg`, `op_bare_plain_next`), so no tree derives log@(x·cos y) to refuse it — refused trees per
site multiplied: `\sin\log x\cos y+\cos\log x\sin y` 25 trees and an `ambiguous_math` warning, eight such terms the
5,001-tree cap; now one tree (`parse_tree_count_limits`). Space, `d` and type-mark ends stay refusals in the actions (the
grammar cannot see them): `\sin\log x\,dx+\cos\log x\,dx` enumerates 36 trees (9 before 57cj.9), a residual; the
parallel trig chain also grows the bocage, so `\sin^2\log\log x\,dy` (491 or-, 503 and-nodes) takes the tree route and warns
(14 trees, the same reading; 8 again in 57cj.11). Until 57cj.13 an argument
list kept a letter's application across the space (`\cos\phi\,(x,y)` read cos@(φ(x,y)); the #18 row's "an argument list keeps
the application", 2605.24758) — narrowed in 57cj.11 to a parenthesized list of variables; since 57cj.13 a tuple after a space
in a trig argument is a vector (`\cos\phi\,(x,y)` cos φ·(x,y), below), the argument list a derivative operand's rule (#374). The OPFUNCTION before a big operator is an item (#390: `\sin\log\det A`). Readings 57cj.9
changed by these rules: `\sin\log x\cdot y` sin@(log(x·y)), as `\log x\cdot y`; `\cos 2\pi i\mathbf k` cos@(2πi·k) and
`\cos\log 2\mathbf k` cos@(log(2k)) — a coefficients-only argument takes a marked item (the type-mark rule's exemption, with
the imaginary unit and a function of a constant now constants). With the physics package loaded `\sin` is an OPFUNCTION
(physics' own operator), so none of these trig rules apply there (`\sin x\,y` sin@(x·y)) — a residual (SYNC).

57cj.11 (the 57cj.10 review; latent, its probes): an argument list is a parenthesized comma list of variables only —
letters, scripted letters (not to a constant power), ellipses: `(x,t)`, `(x_1,\ldots,x_n)`, `(x',y)` (`is_an_argument_list`,
`is_a_variable`). Numbers, applications, powers, sums, brackets or another separator make a vector, interval or list that
the letter multiplies, as Perl reads it: `\cos\theta\,(1,0)+\sin\theta\,(0,1)` cos θ·(1,0)+sin θ·(0,1),
`\hat r=\sin\theta\,(\cos\phi,\sin\phi,0)+\cos\theta\,(0,0,1)`, `\cos\phi\,[x,y]`, `\cos\phi\,(x;y)`, `\cos\phi\,(x^2,y)` (57cj.10
read φ applied to each). A trig function's argument composes another trig function's application
(`trig_composed_arg`; Rust-only unparsed before — Perl's `aTrigBarearg` has no TRIGFUNCTION, MathGrammar:340-356, so Perl
multiplies, sine·cosine@(x)): `\sin\cos x` sin@(cos@(x)), `\cos\sin x+1`, `\sin^2\cos x` (sin²)@(cos@(x)), `\sin\cos^{-1}x`,
`\int\sin\cos x\,dx` ∫(sin(cos x)·dx), `\partial_x\sin\cos u`; the inner argument is the one that goes on and ends
(`\sin\cos xy` sin@(cos@(x·y)), `\sin\cos x\,y` sin@(cos@(x))·y, `\sin\cos x\cos y` sin@(cos@(x))·cos@(y)); an inner trig
function before a big operator takes a limit-type operator's application and multiplies a sum (#390: `\sin\cos\det A`
sin@(cos@(det A)), `\sin\sin\sup_x f`, `\sin\tan\lim_n a_n`, `\sin\cos\sum_i x_i` sin@(cos·∑…)). A function before an
operator's application inside the argument multiplies it, as outside (`\log\nabla u` log·∇u): `\sin\log\nabla u`
sin@(log·∇u), `\sin\cos\nabla u` sin@(cos·∇u) (Rust-only unparsed before; Perl sine@(logarithm)·∇u). The OPFUNCTION chain's
first item is built without a trig function at every nesting (`trig_op_bare_item`, `trig_opfunction_item_application`):
`\sin\log\log x\cos y` 7 → 3 trees (the 3 of `\sin\log\log x` alone; `\log\log x` has 2), four sites 3,182 → 81, five
under the cap (243); `\sin^2\log\log x\,dy` 14 → 8 trees, no warning. Why composition: sine·cosine@(x) multiplies a bare
head, which has no value; composition is the only meaningful reading (user ruling 2026-09-29: the mathematically correct
reading in context); latent, no corpus witness in the 3,003 A/B sources (the 57cj.11 review's mining: the only hits are
the operator-name lists of 2605.21236).

57cj.12 (the 57cj.11 review; latent, its probes): an argument list holds at least one variable and may hold numbers
beside it — an evaluation point (`(0,t)`, `(x,0)`, `(x,-1)`, `(x,\tfrac12)`) — and an accented letter is a variable
(`is_accented_letter`: an OVERACCENT or UNDERACCENT over a letter, which lexes as one ATOM: `\hat x`, `\dot q`, `\vec r`),
on all three paths (trig argument, derivative operand, postfix) until 57cj.13, which leaves the trig path a vector (below):
`\cos\phi\,(x,0)` read cos@(φ(x,0)) and `\cos\phi\,(\hat x,y)` cos@(φ(x̂,y)) in 57cj.12, cos φ·(…) since; `\partial_x u\,(0,t)=0` ∂_x(u(0,t))=0, `\partial_q L\,(q,\dot q,t)`, `\partial_t\psi\,(\vec r,t)` — as the bare
letter's `u\,(x,0)=g(x)` u@(x,0), `L\,(q,\dot q,t)` (57cj.11 read these products, an m13 → m14 regression; Perl multiplies
them all). Numbers alone (π among them) stay a vector: `\cos\theta\,(1,0)`, `\cos\phi\,(0,\pi)`; so do applications, sums
and powers (`\sin\theta\,(\cos\phi,\sin\phi,0)`). Residual: a numeric superscript is a power, so contravariant coordinates
read as a vector (`\cos\phi\,(x^1,x^2)` cos φ·(x¹,x²), `\partial_t u\,(x^1,x^2)`; telling an index from a power needs
the whole tuple). An ellipsis opens a trig argument, as it opens an OPFUNCTION's (`trig_arg += elideop`; `\ldots` already
did, a `factor_base` ID): `\cos\cdots x` cos@(⋯·x), `\sin\,\cdots`, `f(\sin\cdots)`, `\sin\cos\cdots x` sin@(cos@(⋯·x))
(Rust-only unparsed before; Perl cosine@(cdots·x), sine·cosine@(cdots·x)); after an item it still ends the argument
(`\cos\theta_1\cos\theta_2\cdots\cos\theta_n`, divergence #3). `\cos\cdots x` kept its twin cos@(⋯)·x as a second reading
(two unique parses; `\cos\ldots x` has one), refused since 57cj.13.

57cj.13 (the 57cj.12 review; latent, its probes; no corpus witness in the 3,003 A/B sources): a trig argument is an angle,
so a tuple after a space is a vector the trig value scales, whatever its items (`trig_letter_application`; decided
2026-09-30 by the main loop under the user's ruling "the mathematically correct reading in context", 2026-09-29, for the
user's review): `\mathbf v=\cos\alpha\,(v_x,0)+\sin\alpha\,(0,v_y)` cos α·(v_x,0) + sin α·(0,v_y), `\cos\theta\,(a,0)`,
`\cos\phi\,(x,0)`, `\cos\phi\,(x,y)` cos φ·(x,y), `\cos\phi\,(x,0)!` cos φ·(x,0)!, as Perl (57cj.10-57cj.12 applied φ to the
argument lists and evaluation points above). The argument list is a derivative operand's rule only (#374:
`\sin\partial_x u\,(x,0)` sin@(∂_x(u(x,0)))); without the space the letter applies (#18: `\cos\phi(x,y)` cos@(φ(x,y))). An
ellipsis opening the argument lets it go on as any first item does, a run of ellipses too (`trig_ellipses`), and its twin
cos@(⋯)·x is refused (`leaves_a_trig_bare_argument`, `is_an_ellipsis_run`): `\cos\cdots x\,y` cos@(⋯·x)·y, `\sin^2\cdots x`
(sin²)@(⋯·x), `\sin\cdots\cdots x` sin@(⋯·⋯·x), `\tan\cdots\cdots\cdots x` (57cj.12 read cos@(⋯)·x·y, (sin²)@(⋯)·x,
sin@(⋯)·⋯·x; Perl as now, but for the space: cos@(⋯·x·y)); `\cos\cdots x` one parse, four sites one unique of 16
enumerated (was 16 unique), as `\cos\ldots x` and `\cos a x`. After an addop or a MulOp the ellipsis is a bare head's
argument, not an elided operation's operand (`infix_apply_and_elide` refuses a trig function, OPFUNCTION or operator
before it): `a+\cos\cdots` a + cos@(⋯), `\sin\cdots-\cos\cdots` sin@(⋯) − cos@(⋯), `a+2\cos\cdots`, `a+\log\cdots`,
`a+\nabla\cdots`, as Perl (57cj.12 read a + cos + ⋯, which rendered a second `+` the source does not have). Residual: an
elided operation shows its operator before the ellipsis (`a+b\cdots` a + b + ⋯, `b_1+b_2\cdots+b_j`; Perl a + b·⋯; SYNC row
"Math-parse residuals of the 57cj train" (9)).

57cj.14 (the 57cj.13 review; latent, its probes; no corpus witness in the 3,003 A/B sources): the elided operation is
refused after every head with no argument of its own and after its bare application (`reaches_a_following_ellipsis`):
a trig function, OPFUNCTION or operator (57cj.13), and a big, limit-type or differential operator, bare or scripted
(`is_bare_big_operator`: SUMOP, INTOP, BIGOP, LIMITOP, DIFFOP) — `a+\sum\cdots` a + ∑@(⋯), `a+\sum_n\cdots`,
`x_1+\sum\cdots+x_n`, `\sin\cdots+\det\cdots` sin@(⋯) + det@(⋯), `a+\int_0^1\cdots`, `a+\oint\cdots`, `a+\prod\cdots`,
`a+\bigcup\cdots`, `a+\lim\cdots`, `a-\inf\cdots`, `a+\sup\cdots`, `a+\partial_x\cdots`, as Perl (57cj.13 read a + ∑ + ⋯).
After a head's bare application the ellipsis reads as it does with nothing before it (`ends_in_a_bare_ellipsis` no longer
refuses that product): it goes on the argument — `a+\cos\cdots\cdots` a + cos@(⋯·⋯), `a+\det A\cdots` a + det@(A·⋯),
`a+\sum_n a_n\cdots` a + ∑_n(a_n·⋯), as Perl — or, where a trailing ellipsis leaves the bare argument (user ruling
2026-09-29), multiplies the application: `a+\sin x\cdots` a + sin@(x)·⋯, `a+\log x\cdots`, `a+\nabla u\cdots`,
`a\cdot\log x\cdots` (a·log@(x))·⋯ (Perl a + sin@(x·⋯)); never a + sin x + ⋯, whose `+` the source does not have. A run of
ellipses opening a trig argument is one whatever the macros (`is_an_ellipsis_run` takes an ellipsis ID):
`\sin\ldots\cdots x` sin@(…·⋯·x) as `\sin\cdots\ldots x`, `\cos\ldots\cdots` cos@(…·⋯), `a+\cos\ldots\cdots`, as Perl
(57cj.13 read sin@(…)·⋯·x); across a space it ends (`\sin\ldots\,\cdots x` sin@(…)·⋯·x). Residual (SYNC (9)): a closed
factor keeps the elision (`a+b\cdots`, `a+\sin(x)\cdots`, `a+\Delta\cdots` a + … + ⋯); a limit-type operator's operand
that opens with a group is not one — it takes what follows, `a+\det(A)\cdots` a + det@((A)·⋯), as Perl and as
`\det(A)\cdots`, `\det(A)B` det@(A·B) alone (a LIMITOP is a big operator, Perl `bigop`, MathGrammar:717), where
`a+\gcd(a,b)\cdots`, `a+\max(a)\cdots`, `a+\log(x)\cdots` (OPFUNCTIONs) keep the elision.

57cj.15 (the 57cj.14 review; latent, its probes; no corpus witness in the 3,003 A/B sources): 57cj.14's grammar derived
a run with one leading ellipsis ID only (`trig_ellipses += ellipsis_id elideop`) while `is_an_ellipsis_run` accepted any
mix, so the refusal of sin@(run)·⋯ left runs with a later ID before an ELIDEOP no tree — `\cos\ldots\ldots\cdots`,
`\sin\ldots\ldots\cdots x`, `\sin\cdots\ldots\cdots x`, `a+\cos\ldots\ldots\cdots`, `\sin^2\ldots\ldots\cdots x`,
`\sin\dots\cdots\ldots\cdots x` were unparsed (57cj.13 read sin@(…·…)·⋯·x). `trig_ellipses` now derives every run that
ends in an ELIDEOP (`trig_ellipsis_ids`: a run ending in an ID, which an ELIDEOP continues), the IDs after its last
ELIDEOP going on as `trig_arg`'s bare items — one derivation per run: `\sin\ldots\ldots\cdots x` sin@(…·…·⋯·x),
`\cos\ldots\ldots\cdots` cos@(…·…·⋯), `\sin\cdots\ldots\cdots\ldots x` sin@(⋯·…·⋯·…·x), `\sin\cos\ldots\ldots\cdots x`, as
Perl. An OPFUNCTION's or operator's bare argument takes a run between two items likewise (#376), joined by MulOps
too, and so does an OPFUNCTION's chain inside a trig argument (`\sin\log x\cdots y` sin@(log@(x·⋯·y)), 57cj.16). A run
after an addop or a MulOp is one unit, a product of its ellipses, as here (57cj.16, `semantics::ends_an_elided_run`):
the elided operation (`infix_apply_and_elide`) takes one ellipsis, never a run's last, so `a+x\cdots\cdots+c` is
a + x·⋯·⋯ + c, `a+\cdots\cdots\cdots+c` a + ⋯·⋯·⋯ + c, `a+x\ldots\cdots+c` a + x·…·⋯ + c and `a+\sin x\cdots\cdots+c`
a + sin@(x)·⋯·⋯ + c, as each reads alone (the first two had no parse, refused both as a bare ellipsis and as the elided
addition; the last two read a + x·… + ⋯ + c, a + sin@(x)·⋯ + ⋯ + c; latent, no corpus witness).

57cj.17 (the 57cj.16 review; latent, no corpus witness in the 3,003 A/B sources): an ELIDEOP run between two items stays
inside a trig argument, as an ellipsis ID does and as in an OPFUNCTION's (`trig_elided`, `trig_elidable_arg`,
`semantics::trig_argument_elision`; the twin refused in `semantics::leaves_a_bare_argument`): `\sin x\cdots y` sin@(x·⋯·y),
as `\sin x\ldots y` sin@(x·…·y) — 57cj.16 split it, sin@(x)·⋯·y, so the reading hung on the macro (`\sin x\dotsb y` stayed
inside, an ID) and on the head (`\log x\cdots y` and `\sin\log x\cdots y` kept the run inside). Whatever the macros, joins and
head, as Perl: `\sin^2 x\cdots y`, `\sin a_i\cdots y`, `\cos_a x\cdots y`, `\sin x\cdot\cdots\cdot y`, `\sin x\times\cdots\times y`,
`\sin x\cdots\cdots y`, `\sin x\ldots\cdots y`, `\sin x\cdots y\cdots z`, `\sin\cdots x\cdots y`, `\sin x!\cdots y`, and a run
opening the argument joined by a MulOp (`\sin\cdots\cdot\cdots\cdot y` sin@(⋯·⋯·y), was sin@(⋯)·⋯·y); through a composition,
a differential operator and an OPFUNCTION's argument (`\sin\cos x\cdots y` sin@(cos@(x·⋯·y)), `\partial\sin x\cdots y`,
`\log\sin x\cdots y` log@(sin@(x·⋯·y)), `\log\sin x\cdot\cdots\cdot\cdots\cdot y`). What ends a trig argument ends it here: a
space before or after the run (`\sin x\,\cdots y`, `\sin x\cdots\,y`), a `d`, a symbol of another type; a trig function's
application after the run is a new factor (`\cos\theta_1\cdots\cos\theta_n`, divergence #3), a trailing ELIDEOP run leaves
(`\sin x\cdots`, `\sin x\cdots+b` sin@(x)·⋯) as user ruling 15 (2026-09-29) has it — "a trailing ellipsis leaves any bare
argument … an ellipsis stays inside only between two items" — while a trailing ellipsis ID stays (`\sin x\ldots` sin@(x·…), as
Perl), which departs from that ruling pending the user's answer to Q10 (does ruling 15 cover trig arguments, reversing Perl's
sin@(x·…)?; SYNC_STATUS "Math-parse residuals of the 57cj train" (10)), so an ID after the run is an item
(`\sin x\cdots\ldots` sin@(x·⋯·…), as `\sin x\ldots` sin@(x·…)), and an argument ending in an OPFUNCTION's bare application leaves the run to that chain (no
`trig_elidable_arg`: `\sin\log x\cdots y` sin@(log@(x·⋯·y)), `\sin\log x\cdots\dots` sin@(log@(x))·⋯·…, its trailing run);
an application to a group closes and the run goes on (`\sin\log(x)\cdots y` sin@(log(x)·⋯·y)), and so does an OPFUNCTION before
a big operator, whose operand takes it (no twin refused: `\sin\log\det A\cdots y` keeps sin@(log@(det A))·⋯·y, SYNC (13)). A
run holding an ellipsis ID before an ELIDEOP is the whole argument only at its end (`trig_mixed_ellipses`: `\cos\ldots\cdots`
cos@(…·⋯)); an item after it goes on through `trig_elided`, so no ID before an ELIDEOP has a second derivation
(`\sin\ldots\cdots\ldots\cdots\ldots\cdots\ldots\cdots x` 9 trees, as 57cj.16). Two refusals that saw no
twin now do, so one reading survives at every number of sites (SYNC_STATUS "Math-parse residuals of the 57cj train"
(10)): a trig function ending an OPFUNCTION's or operator's argument keeps its argument's chain where the OPFUNCTION's
does not go on (`semantics::trig_application_ending`, read by `leaves_a_trig_bare_argument`): `\log\sin x\ldots\ldots`
log@(sin@(x·…·…)), `\log\sin\cdots\cdots\cdots` log@(sin@(⋯·⋯·⋯)), `\exp\sin x\ldots\ldots`, `\log\cos x\ldots\ldots`, as Perl
(log@(sin@(x))·…·… alone, Perl's at a second site); and a MulOp continues a trig argument (`semantics::trig_argument_across_a_mulop`,
`trig_arg mulop factor_base`): `\sin x\cdot y` sin@(x·y) as before but one tree, `\sin\log\ldots\ldots\ldots\cdot c`
sin@(log@((…·…·…)·c)), `\sin\log a_i\cdot\ldots\cdot\cdots\cdot\ldots` read as `\sin\log x\cdot\ldots\cdot\cdots\cdot\ldots`
(the pragmas chose between two trees by the letters). The ellipsis grid (`~/data/pk_agents/math/grids/ellipsis_grid/`,
77,250 formulas): 0 lost against 57cj.13-57cj.16, 3,263 readings changed to Perl's, 160 changed otherwise — each the
reading of a shorter form (`\sin\log x\ldots\cdots\ldots` sin@(log@(x)·…·⋯·…) as `\sin\log x\ldots` sin@(log@(x)·…), `\log\sin
x\ldots\cdots` log@(sin@(x·…))·⋯ as `\sin x\ldots\cdots`); formulas with more than one surviving reading 14,598 → 5,570, every
`\log\sin`/`\sin\log` run-length and context inconsistency gone (`byhead.py`).

57cj.18 (the 57cj.17 review; latent, no corpus witness in the 3,003 A/B sources): a run a MulOp or BinOp opens before a
juxtaposed item is the run between two items too, so it stays inside the bare argument — trig function, OPFUNCTION and
operator alike: `\sin x\cdot\cdots y` sin@((x·⋯)·y), `\sin\ldots\cdot\cdots x`, `\cos\dots\times\cdots y`, `\sin x\ldots\cdot\cdots y`,
`\log\sin x\ldots\cdot\cdots y`, `\sin\log x\cdot\cdots y`, `\log x\cdot\cdots y` log@((x·⋯)·y), `\nabla u\cdot\cdots v`,
`\max_i a_i\times\cdots b`, `\sin x\cdot\cdots\ldots` sin@((x·⋯)·…), as Perl. A MulOp's right operand is a juxtaposed product
(`term mulop tight_term`), so the twin (sin@(x)·⋯)·y was built by `infix_apply_nary` with `⋯ y` as one operand, a split
`leaves_a_bare_argument` never saw: both readings survived, 2^n at n sites (`\sin\ldots\cdot\cdots x` 256 at eight), the forest's
order choosing. `semantics::leaves_a_bare_argument` now asks its question (`a_run_stays_inside`) at every junction the
juxtaposed product would have — the split, and each one inside the right's opening run up to its first other factor. A
trailing run after a trig argument that ends in an OPFUNCTION's or operator's bare application leaves both bare
arguments, whatever its first macro (`semantics::leaves_a_trig_argument_s_chain`; user ruling 15, 2026-09-29: "a trailing
ellipsis leaves any bare argument … an ellipsis stays inside only between two items"): `\sin\log x\ldots` sin@(log@(x))·…, as
`\sin\log x\cdots` sin@(log@(x))·⋯ and `\log x\ldots` log@(x)·…; `\sin\log x\ldots\cdots\ldots`, `\sin\log x\ldots\ldots`,
`\sin\exp x\ldots\cdots`, `\cos\log_2 x\ldots`, `\sin\max_i a_i\ldots\cdots`, `\sin\log x\cdot\ldots`, `\sin\log\ldots\ldots`
sin@(log@(…))·… as `\log\ldots\ldots` log@(…)·…, `\sin\cos\log x\ldots` sin@(cos@(log@(x)))·… (57cj.17 kept an ID-led run in
the trig argument, sin@(log@(x)·…·⋯·…), while a `\cdots`-led one left; Perl keeps both in the OPFUNCTION's, sin@(log@(x·…))). The
trig actions ask it of the same pair — `trig_argument_juxtaposition` and `trig_argument_across_an_operator` (`trig_arg mulop
factor_base`) refuse to join, `leaves_a_trig_bare_argument` and `trig_argument_across_a_mulop` refuse no stop. Between two
items the chain still takes the run (`\sin\log x\ldots y` sin@(log@(x·…·y))); after a big operator its operand takes it
(`\sin\log\det A\ldots`, SYNC (13), Q9) and after an application to a group the trig argument's trailing-run reading holds
(`\sin\log(x)\ldots` as `\sin x\ldots`, pending Q10). This change already applies a "yes" to Q10 (does ruling 15 cover
trig arguments?) for a trig argument ending in an OPFUNCTION's or operator's chain: a "no", which keeps `\sin x\ldots`
sin@(x·…), reverts `\sin\log x\ldots` to 57cj.17's sin@(log@(x)·…), and the 2,952 grid rows below with it. The ellipsis grid,
extended by 79,425 mixed-join rows (the joins before, inside and after the run differ; 156,675 formulas): 0 lost against 57cj.13-57cj.17, 1,076 readings changed to Perl's (the
forest-order picks of the twin), 2,952 changed otherwise — every one a trailing run after `\sin\log`/`\cos\log_2` reading by
ruling 15; no formula from one reading to more (`ambig.py`), rows with more than one reading 22,835 → 16,160, every one left a
big operator's (`\det`, `\sum_i`; SYNC (15)).

**Guard**: golden `tests/parse/rust_parse_additions.tex` ("Explicit space ends a trig function's bare argument",
"A symbol of another type ends a trig function's bare argument", its 57cj.7-review rows, "A trig function's argument
takes an OPFUNCTION's application", its 57cj.10- to 57cj.14-review rows); `tests/parse/ellipsis_products.tex` ("An
ellipsis after a function head", "An ellipsis after an operator head or a head's application", "An ellipsis in an
operator function's bare argument" 57cj.14-review rows, "A run of ellipses in a trig function's argument", "A run of
ellipses that ends a trig function's argument", "A run of ellipses after a trig function in an OPFUNCTION's argument", "A run a MulOp opens before a juxtaposed item", "A trailing run after an OPFUNCTION's chain in a trig function's argument"); `tests/parse/count_parses.tex` (57cj.7 to 57cj.17 reviews); `parse_tree_count_limits`
(`trig_composition`, `trig_argument_nested_opfunction_*`, `trig_argument_opened_by_an_ellipsis*`,
`ellipsis_after_a_trig_head_in_a_sum`).

A coefficient run — numbers, π, constants (`constant_run`) — ends nothing at a space, the angle it scales following (user
ruling Q5, 2026-10-01, 57cj.22): `\cos 2\,\theta` cos(2θ), `\sin 2\pi\,\nu t` sin(2πνt), `\partial_x\cos 2\pi\,u` ∂_x cos(2πu),
`\cos\frac{\pi}{4}\,x` cos(π/4·x) (were cos@(2)·θ, …); an item past the coefficients still ends at a space (`\sin 2x\,y`
sin(2x)·y), and so does a differential (`\cos 2\,d\theta`). A trailing ellipsis leaves the argument, whatever its macro (user ruling
Q10, 2026-10-01, extending ruling 15; `trig_bare_application`, `leaves_a_trailing_ellipsis`): `\sin x\ldots` sin(x)·… (Perl
sin@(x·…)), `\sin x\ldots\cdots` sin(x)·…·⋯, `\log\sin x\ldots\ldots` log(sin x)·…·…; between two items it stays
(`\sin x\ldots y` sin@(x·…·y)), and a run that is the whole argument is it (`\sin\ldots` sin@(…)). Both latent in the 3,003
A/B papers. Goldens `tests/parse/rust_parse_additions.tex` ("A coefficient run ends no trig argument at a space"),
`tests/parse/ellipsis_products.tex` ("A trailing ellipsis leaves a trig function's argument").

A BINOP and a large MULOP (⊗ ⊙ and their family, #396) end the argument too (user ruling 2026-10-01, 57cj.22;
`semantics::ends_a_trig_bare_argument`), where Perl's `moreTrigBareargs` goes on across any `MulOp`, its `MulOp : BINOP`
(MathGrammar:688) included: an operator that keeps its juxtaposed operand whole (#393, #396) joins two operands, the trig
application one of them — `\int\sin x\boxast y z\,dx` ∫(⧆(sin x, y z)·dx) (Perl and 57cj.21 ∫(sin@(⧆(x, y)·z)·dx)),
`\sin x\otimes y z` (sin x)⊗(y z) (57cj.21 sin@(x⊗(y z)); Perl sin@((x⊗y)·z)), `\sin\log x\otimes y` (sin(log x))⊗y — an
OPFUNCTION's chain inside the argument ends with it, where `\log x\otimes y` keeps log@(x⊗y) —, `\partial\sin x\otimes y`
(∂ sin x)⊗y, `\sin x\boxast\cdots\boxast y` ⧆(sin x, ⋯, y). The grammar's trig chains take no BINOP; a large MULOP lexes as
any MULOP, so the chains' actions refuse it (`trig_argument_across_an_operator`, `trig_chain_across_an_operator`,
`trig_argument_elision`), and an ellipsis run across either is no trig argument's
(`ends_a_trig_bare_argument_before_its_run`). `\cdot`, `\times`, `/` go on. Latent: no bare trig argument before such an
operator in the 3,003 A/B papers (only grouped ones, `\Lambda\sin(\Theta)\otimes J_2`, 2605.26317). Golden
`tests/parse/product_chains.tex` ("A trig function's bare argument ends at a BINOP or a large MULOP").

### 43. A nested group inside a Dirac label keeps its own bars (formerly OD #368)

Perl's `ketExpression` and `maybeBra` set `$forbidVertBar` with `local` (MathGrammar:373, :401), a dynamic scope that
nothing resets, so inside a bra's, ket's or bracket's label no bar reads as a ket, a bar pair, an evaluation bar or a
conditional — not even inside parentheses: `\langle a|(1-|u\rangle\langle u|)|b\rangle` reads `bra@(a) * (1 - ket@(u)
* bra@(u)) * ket@(b)`, the projector's bars ending the bracket.

**Rust** (57bp; `is_forbidden_dirac_label`, `holds_ket`, `holds_open_bra`, `holds_bar_reading`, semantics.rs): the
label check stops at a nested group — a fence or Dirac bracket pairs its own bars — so the parenthesized projector
stays inside the operator product: `quantum-operator-product@(a, 1 - ket@(u) * bra@(u), b)`, the matrix element the
author wrote (2605.05292, 2605.20326).

**Guard**: golden `tests/parse/bar_pairs.tex` ("A bra that a bracket closes is the bracket's").

### 44. A two-item paren pair is a vector unless a set is asked for (formerly OD #371)

Perl's `Fence` (MathParser.pm:1390-1417) names a two-item comma pair between parentheses by `%enclose2` alone
(`'(@,@)' => 'open-interval'`, :1369), whatever surrounds it: `G=(V,E)`, `\xi=(\xi_1,\xi_2)`, a whole-formula
`(x,y)`, `a^{(i,j)}` and `\{(x_i,y_i)\}` all read `open-interval@(…)`.

**Rust** (57br, user ruling 2026-09-29; `rename_fenced_lists`, `operand_slots`, `paren_pair_meaning`,
semantics.rs): the pair is a `vector` — a pair, a point — unless the slot it fills holds a set: the right operand of
`∈`/`∉` (not a pair right of a pair: `(x,a)\in(\mathcal X,\mathcal A)` is componentwise, 2605.06977; `(t,x)\in(0,T)\times\Omega`
keeps its set slot), the left
of `∋`, either operand of a subset or superset relation (in a multirelation, by the relation beside it; a decorated
one by its base, `\in_{\mathcal A}`), and the operands of `×` (as written), `∪`, `∩`, `∖` (big or scripted,
`\bigcup_n`) or a power's base when that application fills such a slot — `x\in(0,1)`, `A\subset(0,1)`,
`x\in(0,1)^d`, `t\in(-\delta,\delta)\setminus\{0\}`, `x\in\bigcup_n(a_n,b_n)` read `open-interval`,
`(x,y)\in\mathbb R^2` reads `vector`. A function's argument pair is a `vector` (`u(x,\infty)`,
`\Pi(M^2,\infty)`, 2605.28015), unless it runs from −∞ to ∞ or to a number — the interval the function takes
(`S(-\infty,\infty)`, `f(-\infty,-3)`, 57bx; 2605.16086, 2605.01702; `u(-\infty,t)` stays a vector). Elsewhere an infinite endpoint on its own side makes an interval — `-\infty` (or
`\pm`/`\mp`) first, `\infty` (`+`, `\pm`, `\mp`) last: `(0,\infty)`, `C^1((0,\infty))`, `\mu((x,\infty))`, `B=(0,\infty)\times\mathbb R` — while
`(\infty,1)` (an (∞,1)-category, 2605.30648) and `(0,-\infty)` stay vectors. The name is given once, on the chosen parse,
reading relations and delimiters through XMRefs (Perl `p_getTokenMeaning(realizeXMNode(…))`, MathParser.pm:1090), so
it no longer depends on which derivation Marpa enumerated first (before: `G=(V,E)` vector but `\xi=(\xi_1,\xi_2)`
open-interval). Brackets, half-open pairs and `]a,b[` (#362) are unchanged. Accepted, read `vector`: a function's argument pair,
bare-letter function spaces among them (`C(0,\infty)`, `\mathcal L(0,\infty)`, `\mu(0,\infty)`; one from −∞ reads an
interval since 57bx, `C(-\infty,0)`, so `G(-\infty,0)=G(0,\infty)` names its two pairs differently; the number test
reads a single token, so `F(-\infty,\tfrac12)` stays a vector; a scripted or accented
head parses as a product, so `\Pi_1(M^2,\infty)` reads an interval); a set outside any relation and without an infinite endpoint — `\mu((a,b))`, a `∪` outside a relation, an integration
domain `\int_{(a,b)}` (2605.02556), an indicator's subscript `\mathbbm{1}_{(0,\delta_x)}` (2605.01709), a map's
codomain `\to(0,1)` (2605.01729) — a pair set equal to a pair `(p,q)=(1,\infty)`, and a pair in a gathered or split
row whose rows the parser joins by invisible times (the relation's operand is then a product). Witnesses: 2605.01424, 2605.00899, 2605.00335, 2605.03082, 2605.01633, 2605.00581.

**Guard**: golden `tests/parse/paren_pairs.tex`.

### 45. A missing comma beside an ellipsis is supplied (formerly OD #373)

Perl reads an item that starts (or ends) with a bare ellipsis as a product with it: `\{l_1,\cdots l_k\}`
`set@(l _ 1, cdots * l _ k)`, `k=0,1,\cdots L-1` `list@(k = 0, 1, cdots * L - 1)`, `\{30,40\ldots,80\}`
`set@(30, 40 * ldots, 80)` — the author's missing comma read literally.

**Rust** (57bv; user ruling 2026-09-29, split it off; `separate_ellipsis_items`, semantics.rs): in a comma container
(a list or formulae, a Fence-named set or pair, a head's arguments `\max(a_1,\ldots a_n)`), an item whose spine —
through unfenced invisible products, sums and relations — reaches an invisible product that starts with an ellipsis
gives it up as an item before it, and one ending in an ellipsis before a comma gives it up after it, with an
invisible comma (U+2063, Perl's `InvisibleComma` for a missing separator) between. Each reads as its comma'd twin
(`set@(l _ 1, cdots, l _ k)`, `k = list@(0, 1, cdots, L - 1)` once the enumeration attaches, a pair grown to three
items named as Fence names three). Not before a visible operator (`x_1,\cdots +x_n` elides a sum); a trailing
ellipsis only where one factor remains (`p_1p_2\cdots, q` elides the product) and never a decimal's digits
(`x=0.325\ldots`, `x, 0.3\ldots`). Residual: the distributed shape `x_1,\dots x_n\in\{0,1\}` (3 A/B
formulas; SYNC). Witnesses 2605.12837, 2605.12555, 2605.02211, 2605.00390, 2605.25695 (~288 formulas / 130 papers
of the 57bs A/B). **Guard**: `tests/parse/enumerations.tex` "A missing comma beside an ellipsis" (each row beside
its twin).

### 46. A differential operator takes one factor; a Leibniz quotient is one derivative (formerly OD #374)

Perl's grammar lists DIFFOP among the big operators (`bigop : BIGOP | SUMOP | INTOP | LIMITOP | DIFFOP`,
MathGrammar:717), and `addOpArgs`/`moreOpArgFactors` (:605-618) take every factor after one, across a MulOp too:
`\partial F/\partial T` reads ∂@(F/∂@(T)), `\partial\Omega\times(0,T]` ∂@(Ω·(0,T]), `\nu\partial_x u\cdot\partial_x v`
ν·∂_x(u·∂_x v), `\partial^\rho G(x-y)c(y)` ∂^ρ@(G(x−y)·c(y)) (KNOWN_PERL_ERRORS #387).

**Rust** (57bx, then 57cj and 57cj.1; user ruling 2026-09-29: ∂ takes one factor, ∇ keeps its greedy bare argument): a
DIFFOP is no `bigop_application` head. It takes one factor (`diffop_application = diffop_head diffop_operand`, builder.rs;
`diffop_operand` is Perl's `Factor` list: a factor, an application, another differential operator's application, a
postfixed or interval term, a number with the factors juxtaposed after it — `numeric_monomial`, `\partial_x\frac12 u^2`
∂_x(½u²), `\partial_t 2\pi iu` — and an ellipsis before a differential operator's application, `\partial_i\ldots\partial_j u`
∂_i(…·∂_j u)), built by `differential_operator_apply` as a finished factor (`Meta::differential`, as `d x` already was):
`\partial_x u\cdot v` (∂_x u)·v, `g^{\mu\nu}\partial_\mu\varphi\partial_\nu\varphi` g·∂_μφ·∂_νφ, `\partial\Omega\times(0,T]` ∂Ω·(0,T],
`\partial^\rho G(x-y)c(y)` ∂^ρ(G(x−y))·c(y), `\partial_x\partial_y f(x)g(x)` ∂_x(∂_y(f(x)))·g(x) (`diffop_group_application`
nests), `\int\partial_x u\,dx` ∫(∂_x u·dx); `\partial\partial f`, `\partial\sin x\cdot y` keep their operand whole, and so does an ellipsis run between two
items in it (57cj.18a, the 57cj.18 review: `\partial\log x\ldots y` ∂(log(x·…·y)), `\partial\nabla u\cdots v` ∂(∇(u·⋯·v)),
`\partial\sin\log x\cdots y`, as Perl and as `\partial\log x\cdot y` ∂(log(x·y)); were ∂(log x)·…·y —
`semantics::a_run_stays_inside` now looks through the differential, as `leaves_a_bare_argument` does; a trailing run
still leaves, `\partial\log x\ldots` ∂(log x)·…; latent, no corpus witness). What no
factor may follow it takes at term level (`diffop_term`, a `bigop_operand`), so a factor-level operand cannot split it:
a big operator's application (`\partial_t\int_\Omega u\,dx` ∂_t(∫…); 57cj's factor-level route gave
`\partial_t\int u(y)v(y)w(y)\,dy` 42 trees, 8 now as before), a function before one (`\partial\log\sum_i x_i` ∂(log·∑…), ∂(log(∑…)) since 57cj.22's Q7,
2605.30560), a bare function (`\partial\operatorname{\mu}`, 2605.29136), a bare operator before a big operator (`\partial_t\nabla\cdot\sum_i u_i`
(∂_t∇)·∑; a divergence or a curl is one factor, below) and a bare differential operator, a chain (`\partial_x\partial_y` (∂_x)@(∂_y),
`\eta^{\alpha\beta}\partial_\alpha\partial_\beta`, `(\partial_t\partial_s-\partial_s\partial_t)\Phi=0`, `D^\alpha=\partial_1^{\alpha_1}\cdots
\partial_n^{\alpha_n}`; 2605.12948, 2605.21314, 2605.22252, 2605.26285, 2605.27600, 2605.28314, 2605.29990). A partial
derivative is the first item of an operator's, an OPFUNCTION's and a trig function's bare argument (Perl's `aBarearg`
has no DIFFOP): `\nabla\partial_x u` ∇@(∂_x u), `\sin\partial_x u` sin@(∂_x u), `\operatorname{dom}\partial\psi` dom@(∂ψ); a later
item ends it, `\nabla u\partial_x v` ∇@(u)·∂_x v, as Perl. An accented DIFFOP is one (util.rs lexes `\bar\partial` a
DIFFOP; Perl an ATOM): `\partial\bar\partial u=0` ∂(∂̄u) (2605.01526, 2605.15276, 2605.01646). A Leibniz slash quotient
is one derivative (`leibniz_quotient` in `infix_apply_nary`): the numerator is the last differential operator with the
factors after it, the denominator the leading differentials of the numerator's kind (∂ or d: `\int_0^1\partial^n
f/\partial x^n\,dx` keeps its `dx`) its order asks for (the numeric exponent on ∂, else 1; each denominator variable's
exponent; a symbolic order takes the run), and the rest multiplies the quotient — `T\,\partial F/\partial T` T·(∂F/∂T),
`h\,\partial f_A/\partial p_j` h·(∂f_A/∂p_j), `\partial^2 f/\partial x\partial y` ∂²f/(∂x·∂y), `\partial u/\partial x\,v`
(∂u/∂x)·v, `\partial\rho u/\partial t` ∂(ρu)/∂t; a `\frac` numerator over a differential reads the same way
(`regroup_leibniz_numerator`, parser.rs), and a `\frac` denominator under one gives each differential the factors up to
the next (`regroup_leibniz_denominator`: `\frac{\partial u_0}{\partial\Delta\psi}` ∂u_0/∂(Δψ), 2605.23203, 2605.28495,
2605.05995). A bare DIFFOP with nothing to take stays an operand (`\partial/\partial t`, `\partial_x+\partial_y`); a
pre-scripted one keeps Perl's greedy operand (no witness); `\nabla` is an OPERATOR and keeps its bare argument
(`\nabla u\cdot v` ∇@(u·v)). Witnesses: 2605.03741, 2605.08634, 2605.24774, 2605.00580, 2605.00105, 2605.29990, 2605.15405,
2605.21149, 2605.00581 (`\partial_{11}l(F(x),Y)f(x)`), and 57bx's 2605.13790, 2605.22408, 2605.24151
(`\partial\Omega\times(0,T)`), 2605.28300; the 57bw A/B XML applied a ∂ to a greedy slash quotient in 370 formulas / 123
papers, a product of derivatives in 807 / 90, a MulOp chain in 42 / 24, another product in 916 / 94. Residuals: a
scripted letter's group (`\partial_t\rho_t(\bm x)` (∂_tρ_t)·x, ~775 formulas) follows #18's scripted-letter reading; a
braced compound (`\partial_z{\delta B_y}` ∂_z(δ)·B_y, 2605.22729) and an order mismatch in a slash quotient
(`\partial f/\partial x\partial y` (∂f/∂x)·∂y) stay; a letter constant is no number (`\partial_x\pi r^2` ∂_x(π)·r²,
`\partial_\theta\frac1N\sum_n\ell_n` ∂_θ(1/N)·∑…, 57cj.1 review); iopart's `\rmd^n f/\rmd x^n\,\rmd x`
takes the integral's `\rmd x` (the d kind matches); a `\frac` denominator's constant joins the variable
(`\frac{\partial^2\phi}{\partial t^2c^2}` ∂(t²c²)); `\partial_t\nabla\cdot u` is ∂_t(∇·u), as Perl and 57ci read
it (user ruling 2026-10-01, below; `semantics.rs:5439` `divergence_or_curl`). 57cj.1's numeric operands
(`numeric_monomial`, a fraction of numbers lexed `ATOM_NUMBER`, util.rs), the operand before a big operator
(`numeric_big_operand`, before a function before one too) and the unsubscripted ∂ a `\frac` denominator regroups
were the 57cj.1 and 57cj.2 reviews' (57cj.2, 57cj.3; latent, the reviews' probes); a derivative right after the number
joins it (`\partial_x 2\,\partial_y u` ∂_x(2·∂_y u), as Perl).
Residuals: a factor after an integral's differential stays in the integrand, as Perl (`\partial_t\int_0^x u(y)\,dy\,v`
∫(u·dy·v)); an accented letter is never applied (`\partial\tilde s(u,j)` ∂(s̃)·(u,j); Perl ∂(s̃·(u,j)), s̃ unapplied as well); `\partial_x 2\frac12 u`
reads (∂_x(2+½))·u through the mixed-number rule; `\nabla\partial_x 2\sum_i a_i` ∇@(∂_x 2)·∑; `\partial_x 2\nabla\sum_i a_i`
reads (∂_x 2)·∇·∑ (Perl ∂_x(2·∇·∑); no numeric route before an operator before a big operator);
`\mathbb{E}\partial_t\sum_i X_i` E·∂_t(∑…). OPEN, pending a ruling (57cj.3 review): 𝔼 before a derivative is a letter
since 57cj.3 (`\mathbb{E}\partial_t u\,v` E·∂_t u·v, `\nabla\mathbb{E}\partial_t u` ∇@(E)·∂_t u, as Perl); typed (57cj.2), an
OPFUNCTION's bare argument ended at the first derivative, so the Fisher information `\mathbb{E}\partial_\theta\log p\,
\partial_\theta\log p^\top` read 𝔼@(∂_θ log p)·∂_θ…; the question is whether an expectation's bare argument runs on over
later derivatives. Further residuals: before a big operator a number's operand is a term no bare argument takes, so
`\nabla\partial_x 2\sum_i a_i` is ∇@(∂_x 2)·∑… and `\sin\partial_x 2\sum_i a_i` sin·∂_x(2·∑…) (the function before a big
operator a factor of its own, as `\sin\sum_i a_i` sin·∑…); physics' braced `\dd{x}` lexes one ATOM a monomial takes
(`\int\partial_x 2uv\,\dd{x}` ∫(∂_x(2uv·dx))); `\partial_x 2\pi\,\partial_y u` is (∂_x(2π))·∂_y u (Perl ∂_x(2π·∂_y u)). The
differentiated number's preference is a soft pragma (`DifferentiatedNumbersTakeTheirFactors`, 57cj.4), counting its
sites (57cj.5): a refusal in `apply_invisible_times` (57cj.1–57cj.3) killed the last parse where the monomial cannot hold
what follows (`\partial_t 2\,\partial_x u\,\nabla\cdot v`, 57cj.3 review), and a pass-or-fail pragma let that one site make
every other split (`\partial_t 2\,\partial_x u\,\nabla\cdot v+\partial_y 3w` read (∂_y 3)·w, 57cj.4 review). A bare operator or
function after the monomial is none it takes, so no violation (57cj.6): `\partial_x 2u\,\nabla\cdot v` (∂_x(2u)·∇)·v,
`\partial_t 2\,\partial_x u\,\nabla\cdot\mathbf{v}=0` (∂_t(2·∂_x u)·∇)·v = 0 — counted as one, the two readings tied and the
tie-break chose by what stood before (57cj.5 review: the split after `\sin x`, `\log x`, `\nabla f`, the monomial after
`a`); and every differentiated number on the left factor's right edge counts, through a differential operator's
operand and a function's or operator's bare argument (`right_edge`): `\partial_x\partial_y 2u` ∂_x(∂_y(2u)),
`\partial_x 2\,\partial_y 3w\,\nabla` ∂_x(2·∂_y(3w))·∇, `\sin\partial_x 2\,\partial_y u` sin@(∂_x(2·∂_y u)),
`\nabla\partial_x 2\,\partial_y u` ∇@(∂_x(2·∂_y u)). A derivative in a bare argument ends where the argument ends (57cj.7;
57cj.6 review): its numeric monomial crosses no end the enclosing argument obeys — in a trig function's, explicit
space, a `d`, a type mark (`ends_trig_argument`, #367) or a function or operator after it (juxtaposed trig functions are
separate factors), in an OPFUNCTION's or operator's an OPFUNCTION or operator, unless a bound head's variable is
mentioned (`trig_derivative_item`, `operator_bare_apply`, `crosses_a_bare_argument_end`): `\sin\partial_x 2u\,v`
sin@(∂_x(2u))·v as `\sin\partial_x u\,v` sin@(∂_x u)·v, `\sin\partial_t 2\pi u\,v`, `\sin\partial_x 2u\mathbf v`, `\sin\partial_x 2u\cos v`,
`\log\partial_x 2u\log v` log@(∂_x(2u))·log v, `\nabla\partial_x 2u\,\log v`, `\max_i\partial_x 2u_i\log v` (but
`\max_i\partial_x 2u_i\log v_i` takes it); a function's or operator's bare argument in it crosses none of a trig
argument's own ends either, and ends where its own head ends it (57cj.8; 57cj.7 review): `\sin\partial_x\log u\,v`
sin@(∂_x(log u))·v, `\sin\partial_x 2\log u\,v`, `\sin\partial_x\log u\,v\,w` (one parse each, where
sin@(∂_x(log(u·v))) survived to the pragma, and a four-term sum held 16) — a mention of the variable a bound head binds
continuing it across the space (57cj.9; 57cj.8 review: `\sin\partial_x\max_i u_i\,v_i` sin@(∂_x(max_i(u_i·v_i))), the head's
scope following its variable, user ruling 2026-09-30, before #367's space). The principle: no end where everything under
the derivative before it is constant ends anything — ending there would differentiate a constant — so
`\sin\partial_x 2\,\partial_y u` is sin@(∂_x(2·∂_y u)), `\sin\partial_x 2\,u\,v` sin@(∂_x(2u))·v, and (57cj.8; 57cj.7 review)
`\sin\partial_x 2\pi\mathbf v` sin@(∂_x(2πv)), `\sin\partial_x 2\pi\,u\,v` sin@(∂_x(2πu))·v, `\log\partial_x 2\pi\log v`
log@(∂_x(2π·log v)), `\nabla\partial_x 2\pi\,\log v`, `\partial_x 2\pi\,\partial_y u` ∂_x(2π·∂_y u); judged down the whole path,
through a function's argument too (57cj.9; 57cj.8 review): `\sin\partial_x\log 2\,u\,v` sin@(∂_x(log(2u)))·v,
`\sin\partial_x 2\log 2\pi\,u\,v` sin@(∂_x(2·log(2πu)))·v, `\sin\partial_x\log\sqrt2\,u\,v`. A constant (`is_constant`, one notion
with the lexer's `is_numeric_constant`, 57cj.9) is a number; π unscripted or raised to a constant power (`\pi^2`, not
`\pi^a` or `\pi_a`: a field or a policy — `\partial_\mu 2\pi^a\partial^\mu\pi^a` (∂_μ(2πᵃ))·∂^μπᵃ, a regression of 57cj.8,
`\sin\partial_x 2\pi_a\,u\,v` sin@(∂_x(2π_a))·u·v); a sum, quotient, product, constant power, root or nth root of
constants; the imaginary unit `i` (`\mathrm{i}`, `\imath`) right after a constant in a product (57cj.9; 57cj.8 review: 25 of
25 sampled `2\pi i` in the delta A/B's 373 are the unit — `\sin\partial_t 2\pi i\,u\,v` sin@(∂_t(2πiu))·v, `\partial_x\frac{1}{2\pi
i}u` ∂_x(1/(2πi)·u); a leading `i` stays a letter, `\sin\partial_x i\,u\,v` sin@(∂_x i)·u·v); a group holding one; a function
applied to one (`\log 2`). A derivative of a bare π or π-power counts as no differentiated number (`differentiates_a_number`):
an integral's measure `\int f\,d\pi(x,y)` keeps dπ (2605.00545, 2605.28589; a first cut of 57cj.9 read d·π·(x,y) in 65
A/B formulas). The lexer reads a constant atom as the number `\frac12` is (`ATOM_NUMBER`: `\sqrt2`, `\frac{\pi}{2}`,
`\frac{\pi^2}{6}`, `\sqrt[3]{2}`, `\frac{1}{2\pi i}`; a Dual by its presentation, a reference through the idstore only;
`\frac{1}{\text{n}}` is none), and a number raised to a constant power leads a monomial (`numeric_power`: `\partial_x 2^3u`
∂_x(2³u), `\partial_x\sqrt2^3u`), so `\partial_x\sqrt2 u` is ∂_x(√2·u), `\partial_x\sqrt[3]{2}u` ∂_x(∛2·u) (were ∂_x(√2)·u). The count
reads into a function's or operator's bare argument only for a differentiated constant (an argument that has ended takes
nothing more). In a trig argument a derivative of a numeric constant — a number, a constant monomial a number leads, a
function's bare application to a constant — takes what follows it: neither the chain nor the product goes on after it
(`ends_in_a_differentiated_constant`, 57cj.9; 57cj.8 review NIT 9: `\sin\partial_t 2\pi\,f t\,v`, `\sin\partial_x 2\pi\,u\,v` one
parse, where 2–4 survived to the pragma). A bare π leads no monomial — `\partial_x\pi u` stays ∂_x(π)·u, ∂ taking one factor:
π is also a field (`\partial_\mu\pi^a\partial^\mu\pi^a`; all 77 ∂π formulas of the 9,420 corpus ∂ formulas are a field or a
policy, 57cj.8 review; Perl ∂_x(π·u)). A letter's application to a group across explicit space is no operand of a
derivative, nor a factor of its monomial (57cj.9; 57cj.8 review, latent: `\partial_x u\,(1-x)` (∂_x u)·(1−x),
`\sin\partial_x 2u\,(1-x)` sin@(∂_x(2u))·(1−x), `\partial_z\rho\,(\tfrac12|\nabla\theta|^2)` (∂_z ρ)·(½|∇θ|²) — 6 formulas in the
delta A/B, 2605.24151 and 2605.18945's measure `\differential^4\mathsf y\ (1-\dots)`; `\partial_x u(1-x)` stays ∂_x(u(1−x)), this file's #18; outside a derivative's operand and a trig argument the
letter still applies, `k\,(x-y)` k@(x−y), a ruling pending). A trig argument's trailing space is its last token's (#367).
Residuals: `u\,\nabla\cdot v` groups as (u·∇)·v, where ∂_x(2u)·(∇·v) is intended (Perl ∂_x((2·u·∇)·v) and
∂_t(2·∂_x((u·∇)·v)) = 0, its greedy `bigop`); an argument list keeps a letter's application across the space
(57cj.10: `\partial_t u\,(x,t)` ∂_t(u(x,t)); 2605.24758) — a parenthesized list holding a variable (57cj.11, 57cj.12:
constants beside it an evaluation point, `\partial_x u\,(0,t)=0` ∂_x(u(0,t))=0, `\partial_q L\,(q,\dot q,t)`; 57cj.13, the
57cj.12 review: ∞ and a signed one, `\partial_x u\,(x,\infty)` (was (∂_x u)·open-interval), a lone `\cdot` or `\bullet` a
variable slot, the standard `\partial_x u\,(\cdot,t)`, a number approached from one side `(0^+,t)`, an accent over a
scripted letter `\partial_t f\,(\hat{x_1},t)` (`\hat{x^2}` a power); an unscripted e or i is a constant as π is, so
`\partial_t f\,(i,0)` is a vector and `(x,i)`, `(i,j)` hold a variable; 57cj.14, the 57cj.13 review: a subscripted or
primed e or i is a letter, an index or a basis vector, `\partial_t a\,(i_1,\ldots,i_k)` ∂_t(a(i_1,…,i_k)), `(i',t)`,
`(e_1,t)`, `(\mathbf e_1,t)` apply (57cj.13 read them vectors); raised to any other power it is the exponential or a power
of the imaginary unit, no variable (57cj.15, the 57cj.14 review, which admitted any script): `(e^{x},t)`, `(e^{i\theta},t)`,
`(\mathrm{e}^{x},t)`, `(i^n,t)`, `(i^j,t)`, `(e^2,t)` are vectors, as Perl's open intervals — the whole script chain
deciding, not its outer layer, so script orders TeX renders alike read alike (57cj.16, the 57cj.15 review): a letter if
any layer is a subscript or a script opening with a prime and no layer a constant power, `(e^x_k,t)` as `(e_k^x,t)`,
`(e'^x,t)` (TeX's one superscript `e^{\prime x}`) as `({e'}^x,t)` apply, `(e^2_k,t)`, `(e_1^2,t)` are vectors — and a prime
raised to a constant is that power (57cj.17, the 57cj.16 review; `semantics::raises_a_prime_to_a_constant`: `e'^2` is TeX's
`e^{\prime 2}`, (e′)², which the constant test could not see through the prime): `(e'^2,t)`, `(x'^2,t)`, `(e'^{-1},t)`,
`(i'^2,t)` are vectors as `(e_1^2,t)` and `(x^2,y)` are (57cj.16 applied them; Perl multiplies, f·open-interval), `(e'^x,t)`,
`(x'^n,t)` apply; latent, no primed-power tuple in the 3,003 A/B sources; an accent over a power takes a bare power's constant test,
`\hat{x^{-1}}`, `\hat{x^{1/2}}`, `\hat{x^\pi}` powers as `x^{-1}` is, `\hat{x^n}` a variable; the lone slot is `\cdot`,
`\bullet` or the wildcard `\ast` (`\Delta(\ast,\ast)`, 2605.02499, 2605.18079 — the only operator alone in a tuple in the
3,003 A/B sources, 52 hits), so `(\otimes,t)`, `(\star,t)`, `(\times,t)` are vectors as Perl reads every one — Perl's
`AnyOp` takes any operator alone before a PUNCT or CLOSE, MathGrammar:204-206, and singles none out as a slot); constants alone or a bracketed pair multiply: `\partial_x u\,(1,0)`
(∂_x u)·(1,0), `\partial_t u\,[a,b]` (∂_t u)·[a,b] — Perl multiplies inside the operand, ∂_x(u·(1,0)), and the one-factor
rule moves the tuple out. Intended but not done (latent, no corpus witness): a signed or scaled letter, an expression
item and a semicolon parameter list are evaluation points too — `\partial_x u\,(-x,t)` ∂_x(u(−x,t)), `\partial_x u\,(2x,t)`
∂_x(u(2x,t)), `\partial_x u\,(e^x,t)` (an application while e was a letter, 57cj.12), `\partial_x u\,(x,t;\lambda)`
∂_x(u(x,t;λ)) — and read (∂_x u)·(…) today. The rule is the derivative operand's: in a trig
argument a tuple after a space is a vector (#367, 57cj.13); a constant group a number leads, raised to a constant power, leads a
monomial (`numeric_group`: any scripted group there added a refused tree, two new `ambiguous_math` warnings in the corpus
∂ set), and a group's delimiters are no factor of a constant atom (57cj.10: `\partial_x(2\pi)^{-3}u` ∂_x((2π)⁻³·u),
`\partial_x\frac{1}{(2\pi)^3}u`); a function of a constant under ∂ before an OPFUNCTION or operator,
`\partial_x\log 2\log v` (∂_x log 2)·log v — ∂ takes one factor and the function's argument ends at a later OPFUNCTION, so no
reading takes both (the grid's 136 constant readings, `grid9/`); the reviewers' formulas `\sin\partial_x 2\log 2\pi\,u\,v`,
`\sin\partial_t 2\pi i\,u\,v`, `\sin\partial_t 2\pi\,f t\,v` read as intended but enumerate over 10 derivations (an
`ambiguous_math` warning).
Each differentiated number's two readings reach the root on the tree route, 2^n trees for n sites (six in a sum,
64; latent, the review's probe; a per-glade preference would drop the split a parent needs, so none); per-site
counting is unobservable in today's grammar (every forced site the reviews probed has a 0-violation reading) and is kept
for correctness. A `d` outside
an integral: `\partial_x 2u\,dx` ∂_x(2u·d·x) (as Perl) while `\partial_x u\,dx` is (∂_x u)·d·x. A `\frac` denominator holding a
subscripted ∂ followed by a non-∂ is a derivative, neither argument regrouped (57cj.6: `\frac{\partial fg}{\partial_x\partial_y u}`,
`\frac{\partial fg}{\partial x\,\partial_y u}`); a subscripted ∂ before another ∂ or at the end names no operand
(`\frac{\partial^2 fg}{\partial_x\partial_y}`, `\frac{\partial fg}{\partial_x\partial y}` regroup ∂(fg)). **Guards**: `tests/parse/integrals_and_differentials.tex` ("A
differential operator takes one factor", "A Leibniz quotient is one derivative", "A differential operator's term
operands", "A differential operator's numeric and fenced operands"), `tests/parse/ellipsis_products.tex` ("A run of
ellipses in the bare argument a differential operator takes"), `tests/parse/bigop_operands.tex`, `parse_tree_count_limits` ∂ rows.

A divergence or a curl is one factor (user ruling 2026-10-01, 57cj.22; `divergence_or_curl`, grammar `divergence_operand`): an
operator's head joined by `\cdot` or `\times` to the one factor after it — `\partial_t\nabla\cdot u` ∂_t(∇·u),
`\partial_t\nabla\cdot(\rho u)`, `\partial_t\nabla\times E=0` ∂_t(∇×E) = 0, `\partial_t\nabla_x\cdot u` (were (∂_t∇)·u; Perl and
57ci ∂_t(∇·u)); the derivative's factor ends after it (`\partial_t\nabla\cdot u\,v` ∂_t(∇·u)·v), and before a big operator the
head alone stays (`\partial_t\nabla\cdot\sum_i u_i` (∂_t∇)·∑). Guard `tests/parse/integrals_and_differentials.tex` ("A
derivative takes a divergence or a curl").

### 47. Juxtaposed operator functions are separate factors; `\mathbb{E}` and `\mathbb{P}` are operator functions (formerly OD #376)

Perl's bare argument (`barearg`, MathGrammar:553-558, for an OPFUNCTION through `addOpFunArgs`, and for an operator)
is greedy: it runs to the next relation or punctuation and takes every OPFUNCTION it meets with it, so
`\log x\log y\log z` is log@(x·log@(y·log@(z))), `\max_i a_i\log b_i` max_i@(a_i·log@(b_i)) and `\nabla x\log y`
∇@(x·log@(y)). A second function juxtaposed after the first's argument begins a new factor, as it is read (user rulings
2026-09-29). Perl's `\mathbb{E}` and `\mathbb{P}` are UNKNOWN letters: `\mathbb{E}X` is E·X, `\mathbb{E}[X|Y]` E·[X|Y],
`\mathbb{P}(A)` P·(A).

**Rust** (57cb): a bare argument's first item may be a function (`op_bare_item`: `\log\log x` log@(log@(x)),
`\max_x\log p(x)`, `\arg\min_w`); a later item may not be an OPFUNCTION (`op_bare_next`, builder.rs), so
`\log x\log y\log z` is log@(x)·log@(y)·log@(z), `\max_i a_i\log b_i` max_i@(a_i)·log@(b_i), `\ln x\ln(1+x)`
ln@(x)·ln@(1+x), `\log^3\omega\log y` (2605.16034), `\nabla x\log y` ∇@(x)·log@(y) (2605.11994). A trig function,
bare or applied, still continues the argument (`\log x\sin y` log@(x·sin@(y)), `\max_j 2\sin\frac{\theta}{2}`,
`\nabla u\sin v`, as Perl), and a bare function head before a bare item takes it (`\tau\log\mathbb{E}_{y'}\exp(x)`
τ·log@(𝔼_{y'}@(exp@(x)))). An ellipsis stays inside only between two items (`op_bare_elided`; an ID one,
`\ldots`/`\dots`, through `op_bare_next`, refused at the end by `operator_bare_apply`): `\log x\cdots y`
log@(x·⋯·y), `\log x_1\cdots\log x_n` and `\log x_1\ldots\log x_n` log@(x_1)·⋯·log@(x_n), and a trailing one leaves,
`\nabla u\cdots` ∇@(u)·⋯, `\nabla u\ldots` ∇@(u)·… (Perl ∇@(u·⋯)). A run of them is one ellipsis here, whatever the
macros (57cj.15, the 57cj.14 review; latent, no corpus witness): `op_bare_elided` goes on over an ELIDEOP and
`semantics::leaves_a_bare_argument` looks past the whole run to the application before it, so `\log x\cdots\cdots y`
log@(x·⋯·⋯·y), `\log x\ldots\cdots y`, `\log\ldots\ldots\cdots x` log@(…·…·⋯·x), `\max_i a_i\cdots\cdots a_n`,
`\nabla u\cdots\cdots v`, as Perl (57cj.14 read log@(x)·⋯·⋯·y); a trailing run leaves (`\log x\cdots\cdots`
log@(x)·⋯·⋯). The run may be joined by MulOps or BinOps, any mix with juxtaposition (57cj.16, the 57cj.15 review:
`op_bare_elided mulop elideop`, `binop`, and the twin `trig_op_bare_elided`, so the grammar derives every run the
refusal looks past): `\log x\cdot\cdots\cdot\cdots\cdot y` log@(x·⋯·⋯·y), `\max_i a_i\times\cdots\times\cdots\times b`,
`\exp x\otimes\cdots\otimes\cdots\otimes y`, `\log x\cdots\cdot\cdots y` log@(((x·⋯)·⋯)·y), as Perl (unparsed on 57cj.15,
log@(x)·⋯·⋯·y on 57cj.14). An OPFUNCTION's chain ending a trig function's argument goes on over the run the same way
(`semantics::trig_argument_ending_in_an_opfunction_application`): `\sin\log x\cdots y` sin@(log@(x·⋯·y)), `\cos\log_2 x\ldots\cdots y`,
as Perl, where both trees used to survive and the forest's order picked one per formula — a second site flipped
`\sin\log x\cdots y` to sin@(log@(x))·⋯·y; a space, a trig function's application after the run and a trailing run
leave it, as they end a trig argument (`\sin\log x\,\cdots\,y`, `\sin\log x\cdots\cos y`, `\sin\log x\cdots` keep
sin@(log@(x))·⋯…). A run a MulOp opens before a juxtaposed item is the same run (57cj.18, the 57cj.17 review): `\log x\cdot\cdots y`
log@((x·⋯)·y), `\nabla u\cdot\cdots v`, `\max_i a_i\times\cdots b`, where (log@(x)·⋯)·y survived beside it (the MulOp's right operand
`⋯ y` hid the junction; `semantics::a_run_stays_inside` at every junction), and a trailing run leaves the chain ending a trig
argument whatever its first macro (`\sin\log x\ldots` sin@(log@(x))·…, #367). Latent: none among the 951 two-ellipsis formulas of the 3,003 A/B sources; the 57cj.16 ellipsis
grid (`~/data/pk_agents/math/grids/ellipsis_grid/`, 77,250 formulas: 15 heads × 4 first items × 6 joins × 5 ellipsis
mixes × run lengths 1-3 × 3 tails × 5 contexts) parses every formula. A bare OPFUNCTION ending a product is a term
(`bare_opfunction_term`), not a factor another factor follows, and a bare head is a MulOp's or BinOp's operand
(`a\cdot\log^2`, `\log\circ\exp`, Perl-unparsed before): the product `tight_term opfunction` let every junction of an
n-function chain be split and refused (2^(n−1) trees; `semantics::leaves_a_bare_argument` still refuses the one split
left, a product whose last application's argument the right side would continue). A blackboard `E`/`P` — `\mathbb{E}`
by the document's decoded font, or the glyphs 𝔼/ℙ (`parser.rs::expectation_operators`, whose lexemes
`type_expectation_lexemes` relabels after lexing) — lexes as an OPFUNCTION: `\mathbb{E}X` 𝔼@(X),
`\mathbb{E}[X|Y]` 𝔼@(X|Y), `\mathbb{E}_{x\sim p}[f(x)]`, `\mathbb{P}(A)` ℙ@(A), `\mathbb{P}\{X\leq t,Y\leq s\}`
ℙ@(X ≤ t, Y ≤ s). Only the lexeme is typed; the token keeps its role, so a set (`x\in\mathbb{E}^3`,
`\sum_{\Gamma\in\mathbb{E}_h}`) or a lone letter carries no function role or operator spacing into the output, and a
numeric power among its scripts names a space and stays a letter (`\mathbb{P}^1\times\mathbb{P}^1`,
`\mathbb{P}^2\#\overline{\mathbb{P}^2}`, `\mathbb{P}_{\mathbb{C}}^2(V)`, `ℙ^{2}(\mathbb{C})`, `\mathbb{R}\times\mathbb{E}^3`). An
evaluation bar evaluates a bare or scripted function or operator head, alone or ending a product, as it does a tight
term (`eval_operand`: `\log^2|_{x=0}`, `\left.\max_i\right|_{i=1}`, `a\log^2|_{x=0}`, `\sin^2|_0^\pi`,
`\left.\nabla^2\right|_{x=0}`, `a\nabla|_0`, as Perl; `\log|_{x=0}`, `\sin|_0` and the `\left.…\right|` operator
forms were Rust-unparsed before 57cb). The standalone `latexmlmath_oxide` types 𝔼/ℙ too
(`type_expectation_operators`, from the serialized `font` of its finalized document). Residuals: a big operator after 𝔼 was a separate factor, as after any OPFUNCTION
(`\mathbb{E}\sum_i X_i` 𝔼·∑…; since 57cf 𝔼 takes it, divergence #379; `\log\sum_i` keeps Perl's reading); LIMITOP heads keep Perl's greedy nest (`\det A\det B`
det@(A·det@(B)), `\liminf`, `\limsup`); a letter-indexed power still reads 𝔼 as a function where a bare item follows
(`\mathbb{E}^n X`; ℙ only where applied since 57cd, `\mathbb{P}^n X` P^n·X), and a numeric one never does (`\mathbb{E}^2[X]` 𝔼²·[X]); a scripted 𝔼 before a scripted group
takes the script inside (`\mathbb{E}_x[X]^2` 𝔼_x@([X]²), #351's scripted-head limit, where `\mathbb{E}[X]^2` is
(𝔼@(X))²); `\lxDeclare[role=UNKNOWN]` does not opt a blackboard E/P out (`role=ID` does); a scripted ELIDEOP ending a bare
argument stays unparsed (`\log x\cdots^2`, `\nabla u\cdots_n`; Perl log@(x·⋯²), as 57ca5), where the ID one leaves
(`\log x\ldots^2` log@(x)·…²); a differential head takes no evaluation bar (`\partial|_0`, `\partial_x|_0`; ∂ is a
DIFFOP big operator, task #185); on the guard's 500-alternative test budget the sixteen-log formula keeps the cut
sample's reading, whose bars misread an evaluation bar (57ca5 left it unparsed; the production budget reads it). 57cd (the regressions of the 57cb train A/B): ℙ is an operator only where it is applied — a group after it, past its
scripts, an OPEN or a group built whole (physics `\qty(A)`, `parser::opens_a_group`; user ruling 2026-09-30):
`\mathbb{P}(A)`, `\mathbb{P}_x[\tau<\infty]`, `\mathbb{P}\{…\}`; otherwise a named matrix or space — `\mathbb{P}^{-1}\mathbb{Q}`
(2605.04101), `\mathbb{P}_{1,1}\mathbb{P}_{1,2}` (57cb: (P_{1,1})@(P_{1,2})) and `\mathbb{P}_1^\intercal\mathbb{P}_1\odot\beta`
(57cb: unparsed; both 2605.01199), `\mathbb{P}\text{-a.s.}` P·[-a.s.] (2605.20593), `\mathbb{P}X` P·X. A ℙ after an integral's differential is
the measure, a letter (`\int u\,\mathrm{d}\mathbb{P}(\omega)` differential-d@(P)·ω as `\mathrm{d}\mu(x)`, 2605.24620):
a `d` counts only in a formula with an INTOP (the lexer's XDIFFUNK rule), a DIFFOP except ∂, a token or built (physics
`\dd`) — `d\,\mathbb{P}(A)` is d·ℙ@(A), `\partial\mathbb{P}(A)` ∂@(ℙ@(A)). 𝔼 with nothing to take — the end, a
relation, punctuation, a close or an infix operator after its scripts, or an operator, which an OPFUNCTION's bare
argument does not take (`parser::is_an_argument`; 57cd review) — is a name: `\nabla\mathbb{E}=0` ∇@(E) = 0,
`\nabla\mathbb{E}\nabla f` ∇@(E)·∇@(f). (A big operator it takes since 57cf, divergence #379.) The lexeme is an EXPECTATION (`EXPECTATION:𝔼:5`, `type_expectation_lexemes`), an OPFUNCTION of its own
category: the grammar's `opfunction` is `plain_opfunction | expectation_head`, and the actions read it as an OPFUNCTION
(`semantics::lexeme_category`; so do CorTeX warning keys). The typing spells the lexeme from the token's
content, never a `meaning` or `name` (`\lxDefMath{\EE}{𝔼}[meaning=expectation]` expectation@(X)). An operator nests over
plain OPFUNCTIONs only (`plain_opfunction`, `scripted_plain_opfunction`) and takes an expectation's application whole,
an operator or a nest of operators only (`open_op_head expectation_application`; a nest over a function takes it as
any function's application): one derivation (57cd reviews: a refused nest, then a refused closed nest, per site made
`\nabla\mathbb{E}[X]+\nabla\mathbb{E}[Y]+\nabla\mathbb{E}[Z]+\nabla\mathbb{E}[W]` and `\nabla\log\mathbb{E}[X]+…` 16 trees
and a warning; 1 now):
`\nabla_{\mathbf x_t}\mathbb{E}[\mathbf x_1\mid\mathbf x_t]` (∇_x)@(𝔼@(x_1 | x_t)) (2605.00941),
`\operatorname*{argmax}_\theta\mathbb{E}(X_\theta)` (2605.03240), were (∇_x 𝔼)@(…); the application forms are an
OPFUNCTION's — a group with its scripts (`\nabla\mathbb{E}[X]^2` ∇@((𝔼@(X))²)), a scripted head's group or scripted
group, a bare argument, a postfixed group's operand (`\nabla\mathbb{E}(X)!` ∇@(𝔼@(X!))), a postfixed scripted group application
(`\nabla\mathbb{E}[X]^2!` ∇@((𝔼@(X))²!)). A group application ends
the operator's argument, as a group does (`is_closed_expectation_application`): `\alpha_t\nabla^{\top}_{x_t}
\mathbb{E}[x|x_t]\Sigma_t^{-1}` α_t·∇@(𝔼@(x | x_t))·Σ_t⁻¹ (2605.20593); a bare one keeps its greedy argument
(`\nabla\mathbb{E}XY` ∇@(𝔼@(X·Y))). A closed nest takes it as any bare argument and goes on:
`\nabla\log\mathbb{E}[X]Y` (∇@log)@(𝔼@(X)·Y), as `\log\mathbb{E}[X]Y` (`nest_is_open` in
`is_bare_operator_application`). Residuals (SYNC "𝔼/ℙ follow-ups"): before a factor the bare route is a refused
twin, as for any OPFUNCTION after an operator, and so is a scripted group's bare-item reading, as for any OPFUNCTION
(`\log[X]^2` 2 trees, four in a sum 16 and a warning, 57cc too) (`parse_tree_count_limits` rows
`operator_before_an_expectation_*`: 1, 1, 2, 2, 1 closed nest, and 1 for three sites in a sum); a scripted 𝔼's group
with scripts then a postfix after an operator is unparsed (`\nabla\mathbb{E}_x(X)^2!`), as a paren group holding a
fenced modifier (`\nabla\mathbb{E}(\to x)`, `\nabla\mathbb{E}(>0)`); a measure's scripts hide the differential before it
(`\int u\,d\mathbb{P}_x(\omega)` u·d·P_x·ω; 57cc d·(P_x)@(ω), neither the measure); a letter after the application multiplies (`\nabla_\theta\mathbb{E}_x[f(x)]g(y)`
…·g·y, where 57bl applies it after `\nabla(X)`); ℙ applied in one place and a matrix in another of the same formula is
read by place (`\mathbb{P}(\mathbb{P}^{-1}\mathbb{Q}-zI)^{-1}` (ℙ@(P⁻¹·Q − z·I))⁻¹, 2605.04101, a matrix product;
direction: one reading per formula); `\mathbb{P}\abs{X}` applies ℙ where `\mathbb{P}|X|` multiplies; a declared operator (`\DeclareMathOperator{\E}{\mathbb{E}}`) is no expectation and nests
(`\nabla\E[X]` (∇@E)@(X)); 𝔼 takes no operator (`\mathbb{E}\nabla f` E·∇@(f), an OPFUNCTION's `aBarearg`). 𝔼 takes a
following big operator since 57cf (divergence #379). Queued by the same rulings (SYNC): a scripted head's scope follows its bound variable
(`\min_{\mu}\tau\mathbb{E}_z\phi(…\mu…)`, 2605.02116, 2605.26653). A blackboard alphabet row
(`\mathbb{ABC…Z}`, short-math-guide:1131) enumerates 21 trees for its E (57cc: 100), a warning, reading
A·B·C·D·𝔼@(F·…·Z).
**Guards**: golden
`tests/parse/opfunction_arguments.tex#juxtaposed_operator_functions_are_separate_factors` (with the 57cb review rows) and its 57cd rows; `perfect_kernel_batch57::probability_applies_to_a_physics_group`;
`tests/parse/postfix_operands.tex` (`a\log\nabla^{2}!` a·log·(∇²)!, one product); `parse_tree_count_limits` rows
`three_logs`, `seven_logs`, `scripted_max_then_log`, `logs_across_an_ellipsis`,
`operator_before_an_expectation_{group,bare,then_a_factor}`, `operator_before_expectations_in_a_sum`;
`perfect_kernel_batch57::opfunction_chain_parses_past_the_tree_sampler` (repro
`math-parse/opfunction_chain_parses_past_the_tree_sampler`, now one reading per chain),
`perfect_kernel_batch56::{varmathbb_is_its_own_alphabet, varmathbb_survives_an_earlier_renewcommand}` (the role stays).

### 48. A bare conditional keeps its events in the condition; any other relation relates the conditional (formerly OD #377)

Perl's bare bar is a factor continuation, `moreFactors : evalAtOp ExpressionsNoBars` (MathGrammar:252-268): the condition
is a bar-free list of plain expressions, and a relation after it relates the whole conditional — `y|x\sim N(0,1)` is
conditional@(y, x) ∼ N@(0, 1), but also `Y\mid X=x` conditional@(Y, X) = x and `\sum_{j|j=1,\ldots,n}` a relation
over a conditional. Rust's bar was statement-level only (`statement vertbar statements`): the condition took every
relation after it, `y|x\sim N` conditional@(y, x ∼ N), and after 57bv.1's refusal of a statement-list condition the
index-list shapes were unparsed (2605.05396, 2605.13128, 2605.19519, 2605.03152).

**Rust** (57cc; user ruling 2026-09-29): an *event* — an undecorated equality, inequality, order, membership or
inclusion (`is_event_relation`: `=`, `≠`, `<`, `>`, `≤`, `≥`, `≪`, `≫`, `∈`, `∉`, `∋`, the subset/superset family) —
states what is given and stays in the condition, as the statement-level bar reads it: `Y\mid X=x` conditional@(Y, X = x),
`T-s\mid T>s`, `y|x_1=a_1,\ldots,x_n=a_n`, `\sum_{j|j=1,\ldots,n}`. Any other relation (`\sim`, `\approx`, `\propto`,
`\equiv`, an arrow, a decorated relation) relates the whole conditional through the factor-level `conditional_head` /
`conditional_formula` (builder.rs; Perl's reading): `y|x\sim N(0,1)`, `\theta|y\propto p(y|\theta)p(\theta)`,
`\gamma_g\mid\theta\stackrel{\mathrm{iid}}{\sim}\mathrm{Bernoulli}(\theta)`, `x|y\to 0`, and mixed ones keep the event
inside, `WT\mid W=w\sim\mathrm{Gamma}(a,b)` conditional@(W·T, W = w) ∼ Gamma@(a, b), `U\mid A=a,B=b\sim V`
conditional@(U, formulae@(A = a, B = b)) ∼ V (2605.18724; Perl splits the condition at the comma,
formulae@(conditional@(U, A) = a, B = b ∼ V)); the conditioned is an expression (`W_t-W_s\mid\mathcal F_s\sim\mathcal
N(0,t-s)`, `X-\mu\mid\sigma\sim t_\nu`; Perl binds the term, minus@(W_t, conditional@(W_s, F_s))), the chain goes on
through any relation (`X_n\mid\mathcal F\sim\mu_n\to\mu`), a METARELOP after it is a statement (`\Gamma\mid\Delta\vdash e:\tau`
conditional@(Γ, Δ) proves e colon τ, `a\mid b\iff a\le b`, `0\mid n\leftrightarrow n=0` 2605.29001, as Perl), and a
conditional on each side of a relation or METARELOP is two, an event between them too (`x|y\sim z|w` (x|y) ∼ (z|w),
`3\mid n^2\Rightarrow 3\mid n`, `a|b=c|d`, `a\mid b\iff c\mid d`, as Perl; `vertbar_modifier` refuses a relation over a
conditional as its left side), and a list before a relation over a conditional is no distributed relation
(`a,b\mid c\sim d` formulae@(a, conditional@(b, c) ∼ d), `\Delta;\Gamma\mid\Phi\vdash P`). The two readings complement
each other — `vertbar_modifier` refuses a condition holding a non-event (`condition_holds_a_non_event`),
`conditional_relation` an event relation, `conditional_over_a_list` a relation list that `relation_pairs` owns — one
derivation each. The factor-level conditional is no `formula`, so fences and set-builders keep their bar (`P(A|B=b,C=c)`,
`\{x\mid x>0\}`), and it enters lists (`y_i|\theta_i\sim P,\quad i=1,\ldots,n` formulae@(conditional@(y_i, θ_i) ∼ P, i =
list@(1, …, n)), `y|x\sim N,\ z|w\sim M`). Changes: divisibility before `\equiv` reads the relation over a conditional (`n\mid m\equiv 0`). Residuals (SYNC_STATUS
"Enumeration residuals"): every bare conditional before a relation keeps a refused twin in the forest — an event the
factor-level one, a non-event the statement-level one — so raw tree counts roughly double per conditional (`Y|X=x` 2,
`y|x_1=a_1,\ldots,x_n=a_n` 3 → 6, `x|y(x)\sim p(x|y(x))` 8 → 16, three subscript conditionals 144 → 2,304) and some pass
the 10-tree `ambiguous_math` threshold with their reading unchanged (a conditional in a list of three or more items,
`y_i|\theta_i\sim P,\ i=1,\ldots,n` 11); fix direction: split events from other relations in the grammar so neither twin
is built. Unparsed, where 57cbb read the statement-level bar over them (nonsense readings): a conditional after a
relation (`x=y|z\sim N`, `y^{(k)}_t:=y_t|z_t=k\sim\mathrm{Beta}(…)`, `A\perp B\mid C\sim D`, `\alpha\Rightarrow y|x\sim N`,
`\Gamma,x:A\mid\Delta\vdash e:B`), a bare conditional as a list's first item (`y|x=1,\ z|w\sim N`: `list_apply` refuses a
conditional item, as it does for `Y\mid X=x,\quad i=1,\ldots,n`, unparsed before 57cc too), a placeholder condition
(`\Gamma\mid\cdot\vdash e:\tau`). **Guards**: golden
`tests/parse/bar_pairs.tex#a_bare_bar_binds_its_adjacent_factors`;
`perfect_kernel_batch57::conditional_ends_before_the_next_statement` (repro
`math-parse/conditional_ends_before_the_next_statement`).

### 49. A plain run beside an attached enumeration attaches (formerly OD #378)

Perl's `maybeColRHS` (MathGrammar:165-172) attaches every comma run after a relation: `a=b,c,d` is a = list@(b, c, d),
which KNOWN_PERL_ERRORS #37 refuses — a bare list is no relation operand — keeping the run as items,
except an enumeration holding an ellipsis (57bv, `i=1,\ldots,n` i = list@(1, …, n)). So a plain run beside an attached
enumeration read as loose items: `i=1,\ldots,4,\,j=0,1,2` list@(i = list@(1, …, 4), j = 0, 1, 2), where Perl reads
formulae@(…, j = list@(0, 1, 2)).

**Rust** (57cc; user ruling 2026-09-29, both directions; `attach_in_container`, semantics.rs): a plain run attaches as
the values of the relation before it in its own segment when another relation of the container took an enumeration on
its right, the relation takes a right run (`takes_right`), the run bridges no chain to the next relation, and each item
continues the relation's value (`continues_progression`: a number after a number, the same letter under scripts, a plain
letter after plain letters): `i=1,\ldots,4,\,j=0,1,2` formulae@(i = list@(1, …, 4), j = list@(0, 1, 2)),
`i=1,\ldots,n;\ j=1,2`, `i=1,\ldots,n,\ \sigma=x,y,z`, and before the enumeration `d=0,1,\,k=1,\ldots,K` formulae@(d =
list@(0, 1), k = list@(1, …, K)) (2605.18167, ~10 formulas), `\kappa_j^k\sim U[-1,1],\quad j=1,2,3,\quad k=1,\ldots,K`.
A signed number continues numbers (`i=1,\ldots,n,\ j=-1,0,1` j = list@(−1, 0, 1)). The run attaches whole or not at all:
kept as items when any item continues nothing (`i=1,\ldots,n,\ x=0, y`, `i=1,\ldots,n, j=1,2, k`), without an
enumeration beside it (`a=b,c,d`, `i=1,2`), and beside a left-attached one, which ranges no relation (`x_1,\ldots,x_n\ge
0,\ k=1,2` keeps `k = 1, 2` loose; Perl k = list@(1, 2); no corpus case). **Guards**: golden
`tests/parse/enumerations.tex#a_plain_run_beside_an_enumeration_attaches`.

### 50. An expectation takes the big operator after it (formerly OD #379)

Perl's `\mathbb{E}` is an UNKNOWN letter, and an OPFUNCTION's bare argument takes no big operator (`aBarearg`,
MathGrammar:323-331), so `\mathbb{E}\frac1n\sum_i X_i` is E·(1/n)·∑… (latexmlc, 0 errors); since 57cb (divergence #376)
Rust read 𝔼@(1/n)·∑…, the expectation of the coefficient alone.

**Rust** (57cf; user ruling 2026-09-30, "follow the mathematical meaning": an expectation of a sum written without
parentheses is of the whole sum): an expectation takes the summation-like big operator after it (BIGOP, SUMOP, INTOP,
LIMITOP — Perl's `bigop`, MathGrammar:717, less DIFFOP, so a physics `\dd t` stays outside) — alone,
`\mathbb{E}_{x\sim\rho}\sum_a` (2605.06977 A3.Ex64.m2), `\varepsilon\mathbb{E}\int_0^T` (2605.13204 S2.Ex6.m1),
`\mathbb{E}\sup_g\int` (2605.03300 S3.E17.m2), `\mathbb{E}\prod_{s=1}^4` (2605.22025 A3.Ex75.m1), `\mathbb{E}_t\inf_z`
(2605.07165 S3.Ex134.m2); after its bare argument's coefficients, `\mathbb{E}\frac1n\sum_{i=1}^n[X_i]` 𝔼@((1/n)·∑…)
(2605.02116 A3.Ex163.m2), `\eta\mathbb{E}_{x\sim\rho}\gamma\sum_a` η·𝔼@(γ·∑…) (2605.06977 A3.Ex74.m1); as a function's
or operator's whole bare argument, `\max_\pi\mathbb{E}_{\tau\sim\pi}\sum_t` max_π@(𝔼_τ@(∑…)) (2605.11975 S6.E24.m1,
2605.11742 S4.E12X.m1), `\operatorname{Tr}\mathbb{E}\prod` (2605.02768 S1.Ex1.m1), `\nabla\mathbb{E}\frac1n\sum_i X_i`
∇@(𝔼@((1/n)·∑…)). The grammar derives these shapes once, as the product they are for any OPFUNCTION, and the product's
action reads it (`product_before_a_big_operator`, `function_before_a_big_operator` →
`expectation_takes_the_big_operator`: the deepest expectation the factor before the big operator ends in takes it), so
no rule and no refused tree is added; after an operator, which nests over no expectation, one term-level rule takes it,
where the big operator's operand runs over the term (`open_op_head expectation_before_a_big_operator bigop_operand`,
`operator_takes_an_expectation_s_big_operator`: `\nabla_\theta\mathbb{E}_x\sum_i f_i` (∇_θ)@(𝔼_x@(∑…)),
`\nabla_\theta\mathbb{E}_{x\sim\mathcal{D}}\int\pi_\theta(y\mid x)\cdot\frac{…}{…}dy`, 2605.09853; a first cut inside
the tight-term `expectation_application` split the operand at every factor, `\nabla\mathbb{E}\int_0^T u(t)v(t)\,dt` 21
trees and a warning; 57cf review). Where the grammar has no reading for an expectation — before a big operator in a trig
function's argument, after a closed nest or a MulOp in a bare argument (`\sin\mathbb{E}\sum_i X_i`,
`\nabla\log\mathbb{E}\sum_i X_i`, `\log x\cdot\mathbb{E}\sum_i X_i`) — it reads as the letter Perl reads, rather than
leaving the formula unparsed: a formula no typed stream parses is parsed again with every expectation also read as
its letter (M3, parser.rs `spell_letter_readings`: the lexeme `EXPECTATION:𝔼.letter`, whose grammar category
`expectation_letter` presents the UNKNOWN lexeme the lexer gave it, `semantics::expectation_as_letter`), and the
counting soft prune `ExpectationLettersAreFallbacks` keeps the readings with the fewest letter readings — per glade in
the ASF traverser (`keep_fewest_letter_readings`), at the root on every route — one expectation at a time:
`\sin\mathbb{E}\sum_i X_i+\mathbb{E}Y` keeps 𝔼@(Y), `\log x\cdot\mathbb{E}\sum_i X_i=\mathbb{E}X` 𝔼@(X) (the untyped retry
of 57cf.1–57ch.1 read every expectation of the formula a letter). As lexed first, then with the fence retry's null
delimiters (`\sin\mathbb{E}\sum_i X_i|0\rangle`, 57ch review). A stream a typed reading parses takes no letter twin:
offered in every formula, the twin doubles the tree iterator's trees per expectation (probed:
`\nabla\mathbb{E}[X]+…+\nabla\mathbb{E}[W]` 16 trees); offered only before a big operator (M3 as committed), it still
added 7 `ambiguous_math` in the delta A/B with no reading changed (2605.02116, 2605.03300, 2605.13204, 2605.29267) and
lost the retry's parse in contexts no list foresees (`\nabla_\theta\mathbb{E}_x\log\sum_y p_\theta(x,y)`,
`\sin\mathbb{E}_x f(x)`; M3 review). The gate is exact, not only cheaper: `ExpectationLettersAreFallbacks` ranks before
every other pragma, so a typed reading beats every letter reading whenever one exists — computing the letter readings
only when none does is the ungated category's result, evaluated lazily (M3.1 A/B: 0 readings changed over the 3,003
papers). Residuals: inside a letter retry each expectation still doubles the tree iterator's trees
(`\sin\mathbb{E}\sum_i X_i=\mathbb{E}[A]\mathbb{E}[B]\mathbb{E}[C]\mathbb{E}[D]` 16 trees and a warning; 0 corpus hits);
the retry's letter readings are Perl's, not always good (`\sin\mathbb{E}_x f(x)` sine@(E_x·f)·x splits f(x),
`\sin\nabla\mathbb{E}\sum_i X_i` sine·∇@(E)·∑…, and against the rulings: `\cos\mathbb{E}\mathbb{E}\sum_i X_i` cos@(E@(E))·∑…,
intended cos@(𝔼@(𝔼@(∑…))) — 𝔼 before ∑ takes the sum; `\nabla_\theta\mathbb{E}_x\log\sum_y p_\theta(x,y)` (∇_θ)@(E_x)·log·∑…,
intended (∇_θ)@(𝔼_x@(log ∑…)); `\log x\cdot\mathbb{E}\mathbb{P}(A)\sum_i X_i` log@(x·E)·P@(A)·∑…, intended log(x)·𝔼@(ℙ(A)·∑…)
— golden rows pinning today's retry readings, M3.2); `latexmlmath_oxide` runs no fence or letter retry.
`\nabla\mathbb{E}_x\mathbb{E}_y\sum_i Y_i` reads ∇@(𝔼_x@(E_y·∑…)) (the retry: ∇@(E_x·E_y)·∑…); both lack the nested
𝔼_x@(𝔼_y@(∑…)) the formula means, which needs an operator's route over a chain of expectations. The pragma
`BigopPreferWiderAbsorption` reads through an unfenced bare OPFUNCTION or operator application
(`pragmatics::ends_in_a_bigop_application`), so `\mathbb{E}\sum_i X_i\cdot c` is 𝔼@(∑(X_i·c)), as `\log\sum_i x_i\cdot c`
log·∑(x_i·c). As they read: a group closes the expectation (`\mathbb{E}[X]\sum_i Y_i` 𝔼@(X)·∑…), a sum ends the big
operator (`\mathbb{E}\sum_i X_i+c`), a MulOp is no juxtaposition (`\mathbb{E}X\cdot\sum_i Y_i`, 2605.28612 keeps a sum
outside with `\times`), another OPFUNCTION kept Perl's product (`\log\sum_i x_i`, `\min_\theta\frac1n\sum_i`, and inside
the argument `\mathbb{E}_{z_j}\min_\mu\frac{\tau}{m}\sum_j` 𝔼@(min_μ@(τ/m)·∑…), 2605.02116 A5.Ex283.m1) — it takes it since 57cj.22/
57cj.23 (#390: Q7 and the ruling of 2026-10-01c, after its coefficients too, 𝔼@(min_μ@((τ/m)·∑…))), an unapplied ℙ is
a letter (`\mathbb{P}\sum_i X_i`). **Guards**: golden `tests/parse/opfunction_arguments.tex` (the 57cf block);
`parse_tree_count_limits` rows `expectation_before_a_big_operator`, `expectation_coefficient_before_a_big_operator`,
`expectation_midterm_before_a_big_operator`, `expectation_nested_before_a_big_operator`,
`operator_before_an_expectation_before_a_big_operator`.

An expectation takes the derivatives after it into its argument too (user ruling 2026-10-01, 57cj.22; `expectation_bare_arg`,
parser.rs `expectation_operators` types 𝔼 before a DIFFOP): the Fisher information `\mathbb{E}\partial_\theta\log p\,
\partial_\theta\log p^\top` 𝔼(∂_θ log p·∂_θ log p^⊤), `\mathbb{E}\partial_x u\cdot\partial_y v` 𝔼(∂_x u·∂_y v), `\nabla\mathbb{E}\partial_x
u\,\partial_y v` ∇(𝔼(∂_x u·∂_y v)) (were E·∂_θ log p·∂_θ log p^⊤, the letter, as Perl); a sign or a group application ends it
(`\mathbb{E}\partial_x u+\partial_y v`, `\mathbb{E}[X]\partial_t u`); every other OPFUNCTION's bare argument still ends at a
derivative after its first item (`\log\partial_x u\,\partial_y v` log@(∂_x u)·∂_y v, as Perl's `aBarearg`, which has no
DIFFOP). Guard `tests/parse/integrals_and_differentials.tex` ("An expectation takes the derivatives after it").

An operator takes a chain of expectations before a big operator, each expectation taking the rest (user ruling 2026-10-01,
57cj.22; `expectation_chain_big_operator`): `\nabla\mathbb{E}_x\mathbb{E}_y\sum_i Y_i` ∇(𝔼_x(𝔼_y(∑ Y_i))) (was ∇@(𝔼_x@(E_y·∑…)), the
letter), `\nabla_\theta\mathbb{E}_x\mathbb{E}_y\mathbb{E}_z\sum_i Y_i`; without the operator the chain nested already
(`\mathbb{E}\mathbb{E}X` 𝔼(𝔼(X)), `\mathbb{E}_x\mathbb{E}_y\sum_i Y_i`). Left: a function between (`\nabla_\theta\mathbb{E}_x\log\sum_y
p_\theta(x,y)` (∇_θ)@(E_x)·log(∑…)). Guard `tests/parse/opfunction_arguments.tex` ("An operator takes a chain of expectations").

### 51. A limits head's bare argument takes a later application that mentions its bound variable (formerly OD #380)

Perl's bare argument is greedy (`barearg`, MathGrammar:321-337, :553-558): `\max_i a_i\log b_i` is max_i@(a_i·log@(b_i)),
but so is `\max_x f(x)\log y`; divergence #376 (57cb) made every later OPFUNCTION a separate factor, splitting both.

**Rust** (57cg; user ruling 2026-09-30, "a scripted operator's scope follows its bound variable"): the bare argument of
a head whose subscript binds a variable takes a later OPFUNCTION's application that mentions one free
(`bound_application`, `semantics::bound_argument_extends`; the product that would leave it outside is refused,
`bound_head_takes` in `leaves_a_bare_argument`): `\max_i a_i\log b_i` max_i@(a_i·log@(b_i)),
`\min_{\mu\in\mathbb R}\tau\mathbb{E}_z\phi((z-\mu)/\tau)+\mu` min_μ@(τ·𝔼_z@(φ(…)))+μ (2605.02116 A5.Ex275.m1 and 28 more),
`\min_{\eta>0}\eta\ln\eta` (2605.18694), `\max_{\boldsymbol\gamma\in H}4n\exp(\ldots\boldsymbol\gamma\ldots)` (2605.26653),
`\max_{S\ni M}e\ln|S|` (2605.09953), `\min_{1\le i\le N}2\arccos(q_i)` (2605.17033). A head binds its subscript when its
scripts are limits — TeX's `\mathop` without `\nolimits` (`\max`, `\min`, `\gcd`, `\Pr`: plain.tex:1073-1074, :1083-1084;
LaTeXML's `scriptpos`, math_common.pool.ltxml:742, :759-764), a starred operator name (`\operatorname*`,
`\DeclareMathOperator*`, amsopn.sty.ltxml:23-29, :50-53) — and, by the same ruling's meaning, the named `argmin`/`argmax`
and an expectation (`\mathbb{E}_x f(x)\log g(x)` 𝔼_x@(f(x)·log g(x))); a `\nolimits` head's script is a base or a
parameter (`\log_a b`, `\operatorname{rank}_K`), an operator's its variable of differentiation (`\nabla_x`): neither
binds. The subscript binds a letter, a chain's interior operands (`\max_{a\le x\le b}` x, `\max_{1\le i<j\le n}` i and
j; 57cg.1), a single relation's first operand holding one (`\max_{i\ne j}` i), each item of a list, a scripted letter by
its base (`\max_{x'}f(x')\log x'`); a mention is a free occurrence in the same font (a script's too, `\log b_i`), not
where an inner head or big operator rebinds the plain letter (`\max_{z\in A}|f(z)|\max_{z\in A}|g(z)|`, 2605.24231); a
scripted inner binder rebinds nothing (`\mathbb{E}_{y\sim p}\tau\log\mathbb{E}_{y'}\exp(y\cdot y')` mentions y,
2605.02116). With no mention the argument ends as #376 reads it (`\max_x f(x)\log y`, `\max_{k\notin\{i,j\}}\frac{\pi_k}{\pi_d}
\exp(-\kappa)`, 2605.06831). The innermost head binding a mentioned variable takes it (`\max_i a_i\max_j b_{ij}\log c_j`
max_i@(a_i·max_j@(b_ij·log@(c_j)))), through an operator's argument too (`\operatorname*{argmin}_w a_w\cdot\max_i
b_{iw}\cdot\log c_i` argmin_w@(a_w·max_i@(b_iw·log c_i)), 57cg.1). Each limits head's argument has its own start
(`limits_bare_application`), so a formula with no candidate item derives once; k OPFUNCTIONs after one head's argument
are k+1 derivations, limits heads in a row compound (Catalan: three 5, four 14), and separate sites multiply — n terms
each with a head and a later OPFUNCTION are 2^n trees whether or not anything is mentioned (`x+\max_i a_i\log c+…` with 4
terms 16 trees and an `ambiguous_math` warning; 12 terms 4,096 in 0.76 s; 14+ reach the 5,000-tree cap, still read
right), and every subscripted operator pays a refused twin (`limits_operator_head` takes any `operator postsubarg`:
`\nabla_x f(x)\log x` 2 → 3 trees). The direction is the one recorded for #377's conditional twins: split the grammar so
the refused reading is never derived. Residuals: the scope ends at the first later OPFUNCTION that does not mention the
variable, so a later mention stays outside (`\max_i a_i\log c\log b_i` max_i@(a_i)·log c·log b_i — the scope should reach
the last mention, a formula-level check); a variable is its base letter (`\max_{x'}f(x')\log x` extends; the whole
scripted identifier — base, script, font — should bind and rebind). **Guards**: golden `tests/parse/opfunction_arguments.tex` ("A scripted operator's
scope follows its bound variable"); `parse_tree_count_limits` rows `scripted_max_then_log`, `bound_variable_*`,
`limits_head_*`, `nolimits_scripted_chain`, `starred_operator_bound`, `bound_head_as_a_first_item`, `nested_bound_heads`,
`bound_witness_min_mu`.

### 52. (withdrawn in 57ch.1) A `<` relation in a `\langle…\rangle` Dirac label (formerly OD #381)

57ch let a plain `<`/`>` relation stand in a `\langle…\rangle` label, where Perl's `$forbidLRAngle` (MathGrammar:402,
:708) forbids it; its one witness, `\langle p_\alpha\mid\alpha<\gamma\rangle` (2605.09161), is an ordinal-indexed
sequence, not an inner product. Withdrawn on review: the label refuses the relation again
(`semantics::holds_plain_angle_relation`), as Perl, and the formula stays unparsed; a sequence-builder reading is open
(SYNC_STATUS "Dirac shapes"). The rest of 57ch's Dirac work is Perl parity, not a divergence: labels are Perl's
`ketExpression : Formulae` (:401-404; `|n,l\rangle` ket@(list@(n, l)), `|F=1,m_F=1\rangle` ket@(formulae@(…))), and a bra
directly before a ket stays a bra, Perl's `maybeBra` fallback (:373-379; `\langle x||y\rangle` bra@(x)·ket@(y), 2605.29622).
**Guards**: golden `tests/parse/bar_pairs.tex` ("A Dirac label is a formula or a list", "A bra directly before a ket
stays a bra").

### 53. amsmath's `\lvert` opens a ket and `\rvert` closes a bra (formerly OD #382)

amsmath's `\lvert`/`\rvert` are an OPEN and a CLOSE (amsmath.sty.ltxml:1150-1152), so Perl reads `\lvert\psi\rangle` as a
group, delimited-|⟩@(ψ), and leaves `\langle\psi\rvert H\lvert\psi\rangle` unparsed.

**Rust** (57ch; 2605.02211, ~49 formulas): they are a ket's opening and a bra's closing bar (`ket_bar`, `bra_bar`,
`braket_bar`): `\lvert\psi\rangle` ket@(ψ), `\langle\psi\rvert` bra@(ψ), `\langle\psi\rvert H\lvert\psi\rangle`
quantum-operator-product@(ψ, H, ψ), `\langle\phi\rvert\psi\rangle` inner-product@(φ, ψ), `\lvert\psi\rangle\langle\psi\rvert`
ket@(ψ)·bra@(ψ); never a conditional's divider, and a group of its own (`\lvert x\rvert\lvert y\rvert` |x|·|y|) as before.
**Guards**: golden `tests/parse/bar_pairs.tex` (the `\lvert` rows).

### 54. A function takes a limit-type operator's application (formerly OD #390)

Perl's OPFUNCTION takes no big operator as its argument (`aBarearg`, MathGrammar:323-331), and a LIMITOP — `\det`,
`\sup`, `\inf`, `\lim`, `\dim`, `\liminf` — is one (`bigop`, :717), so a function before one multiplies it:
`\log\det A` reads logarithm * determinant@(A), `\sin\det A` sine * determinant@(A) (KNOWN_PERL_ERRORS #400).

**Rust** (57cj.9; 57cj.8 review; `function_takes_a_limit_operator`, semantics.rs): an OPFUNCTION or trig function, bare or
scripted, or an operator's nest over one, right before a limit-type operator's application — or a derivative's of one —
takes it as its argument: `\log\det\Sigma` log@(det@(Σ)), `\frac12\log\det(2\pi e\Sigma)` (1/2)·log@(det@(2πeΣ)),
`\cos\sup_t u` cos@(sup_t(u)), `\log\inf_x u`, `\sin\partial_x\det A` sin@(∂_x(det A)), `\nabla_x\log\det(A)`
((∇_x)@(log))@(det@(A)), `\operatorname*{arg\,max}_s\log\det(L_s)`, `\mathbb{E}\log\det\Sigma` 𝔼@(log@(det Σ)),
`\max_{|\alpha|\le m}\ \sup_{x,v}(\dots)` (max_α)@(sup_{x,v}(…)), `\Im\lim_{\varepsilon\to0}` Im@(lim…). Delta A/B 57cj.8 →
57cj.9: 243 formulas in 55 of the 3,003 papers read so, most `\log\det\Sigma` in ML papers (2605.00130, 2605.26554, 2605.02883
`\log{\det}_\epsilon`, 2605.05081 and 2605.28059 max-before-sup, 2605.28932 `\Im\lim`; earlier witnesses 2605.03984,
2605.24401, 2605.25592, 2605.14289; 4 of the 9,420 corpus ∂ formulas). The grammar derives the product once, as for any big operator, and the action reads it — no rule
is added (the precedent of `expectation_takes_the_big_operator`, #379). A sum or an integral stayed a factor of its own
(`\log\sum_i x_i` log·∑…, as Perl) until 57cj.22 (user ruling Q7, 2026-10-01: every function before a summation-like
operator — ∑ ∏ ∫ ⋃, `is_a_summation_like_application` — takes it with its operand: `\log\sum_i x_i` log(∑…), `\sin\log\sum_i
x_i` sin(log(∑…)), `\exp\int_0^t a(s)\,ds`, `\max_\theta\sum_i\ell_i`; and since 57cj.23 after the coefficients of the function's
bare argument too, as an expectation (user ruling 2026-10-01c: `\min_\theta\frac1n\sum_i\ell_i` min_θ((1/n)·∑…); a trig function's
after constants only, `\sin\theta\sum_l P_l` sin θ·∑…), and down an operator's nest (`\nabla\nabla\log\sum_i p_i` ∇(∇(log(∑…))), the
limit-type operators alike: `\nabla_x\log\det(A)` ∇_x(log(det A)), was ((∇_x)@(log))@(det A)); witnesses 2605.05133, 2605.30325, 2605.31599, 2605.03750,
2605.09119; guard `tests/parse/bigop_operands.tex` "A function takes a summation-like operator's application"), and a trig function keeps Perl's product when the operator's operand crosses its
argument's end (`\sin\det A\,y` sin·det@(A·y), `\cos\sup_t u\,v`, `\sin\lim_n a_n\,b`; #367). 57cj.10 (57cj.9 review): a
function's bare application to a function takes it too, inside (`take_a_limit_operator`): `\log\log\det A` log@(log@(det A)),
`\min_\theta\log\det\Sigma_\theta` min_θ@(log@(det Σ_θ)), `\log\max_i\det A_i`, `\log\exp\sup_x f` (2605.14289); a trig
function's argument takes an OPFUNCTION before a big operator (`\sin\log\det A` sin@(log@(det A)); a sum or an integral the
product, `\sin\log\sum_i x_i` sin@(log·∑…) until 57cj.22's Q7, sin@(log(∑…)); Rust-only unparsed before, Perl sine@(logarithm)·…). Not a word that qualifies
the operator (`qualifies_the_limit_operator`, 57cj.11 narrowing 57cj.10's `is_a_limit_qualifier`): `\arg` — or
`\operatorname{arg}`, its spelling (57cj.13, the 57cj.12 review: `\operatorname{arg}\sup_x f` arg·sup_x f as Perl, was
arg@(sup…)) — right before an infimum or a supremum — `\arg\inf f(\theta)` means one arg-inf, so it keeps Perl's argument·inf@(f(θ)) (2605.30648,
2605.16560; the take read the complex argument of an infimum) — and `\operatorname{ess}` before every limit-type operator
(`\operatorname{ess}\sup`, `\operatorname{ess}\limsup`, and since 57cj.12 `\operatorname{ess}\lim_{x\to a}f(x)`, `\operatorname{ess}\det A`
ess·det(A)): ess is never a function of a value, only a qualifier (the 57cj.11 review; 57cj.11 read ess@(lim…), ess@(det A)).
Before any other limit-type operator `\arg`, the complex argument, is a function and takes it: `\bar\theta=\theta-\arg\det M_q`
argument@(det M_q) (the strong-CP phase), `\exp(i\arg\det U)`, `\arg\lim_{z\to0}f(z)` (57cj.10 read products). Before a
minimum or a maximum — OPFUNCTIONs, no limit-type operator — both words apply, as Perl: `\operatorname{ess}\max_x f`
ess@(max_x f), `\arg\min_x f(x)`. The qualifier is judged at the head right before
the operator, so an outer one does not stop the take: `\arg\min_x\log\det\Sigma_x` argument@(min_x@(log@(det Σ_x))) (57cj.10
argument@(min_x@(log))·det Σ_x). Residual (SYNC): one arg-min/arg-inf operator is not modelled — `\arg\min_x f(x)` reads
argument@(min_x(f(x))) (older; 1,824 Math `tex=` in the 3,003 A/B papers by the 57cj.10 review's count, 2605.31100).

**Guard**: golden `tests/parse/bigop_operands.tex` ("A function takes a limit-type operator's application"),
`tests/parse/operator_application.tex`, `tests/parse/opfunction_arguments.tex` (`\log\det A`).

### 55. A BINOP keeps its juxtaposed operand whole (formerly OD #393)

Perl reads a BINOP as a MulOp (`MulOp : BINOP addOpDecoration`, MathGrammar:688), so it takes one factor on its right
and a juxtaposition after it multiplies the result: `a\boxast b c` ⁢(⧆(a,b),c), `KX\mathbin{\|}(I-K)X` ‖(KX, I−K)·X. A
BINOP is a binary operator of unknown meaning: `\mathbin{…}`, the stmaryrd box operators, every
`\DeclareMathSymbol{…}{\mathbin}` symbol of a raw-loaded package (sect08.rs), a raw `\mathchar`/`\mathchardef` of
class 2 (latexml_core common/mathchar.rs) and a `\Udelimiter` of class 2 (latex_constructs_rust_only.rs).

**Rust** (57cj.19.2-57cj.19.8, `infix_apply_nary`): a BINOP's right operand is the juxtaposed product whole — ⧆(a, b c),
‖(KX, (I−K)X) — as before 57cj.19 (the one-factor regroup is a MULOP's only: `a\cdot b c` (a·b)·c, as Perl). Perl's own
grammar says "BINOP can never really be satisfactory; it comes from something marked as \mathbin; we don't know any more
about it" (MathGrammar:690-691), and the corpus agrees: the 57cj.19.1 A/B (km191, m23 → m24, 3,003 papers) that gave a
BINOP the one-factor rule changed 11 formulas in 6 papers, every one to a wrong reading — concatenations
`KX\mathbin{\|}(I-K)X` (2605.31129) and `[WX_i\mathbin{\|}WX_j\mathbin{\|}w_{ij}]` ‖(‖(WX_i, W)·X_j, w_ij) (2605.31315,
2605.08689), a map arrow set as a `\mathbin` (2605.26237), an inner product (2605.25490), temporal operators (2605.30618).
Before an operand that opens with an integrand — a factor that is no differential — and holds an integral's differential
after it — a `d`-kind differential's application, not a differential operator's (`KX\mathbin{\|}Y\partial_t u` stays ‖(KX, Y·∂_t u), `\int f\boxast g\,\partial_t u`
⧆(f, g·∂_t u)): a bare `d`, which the lexer offers as one only in an integral's operand (`util::in_an_integral_operand`;
`a\boxast b\,dx` stays ⧆(a, b·d·x)), or a bound differential, one anywhere — iopart's `\rmd`, elsart's `\d` (meaning `differential-d`),
physics' `\dd`/`\differential` (meaning `differential`), braced or raised too (`\dd{x}`, `\dd[3]{x}`, `\dd^2 x`; not a
variation, `\variation x`, which stays in the operand where Perl splits it off) — a BINOP takes the integrand's factors
before the differentials, its juxtaposed operand, as a large MULOP does (#396; user ruling 2026-10-01, which replaced
57cj.19.3-57cj.19.8's one factor, Perl's): `\int f\boxast g\,dx` ∫((f⧆g)·dx), `\int f\boxast g h\,dx` ∫((f⧆(g h))·dx) (was
∫((f⧆g)·h·dx), Perl's), `\int f\mathbin{\#}g h\,dx\,dy` ∫(#(f, g h)·dx·dy), `\int_X f\boxast g\,d\mu(x)` ∫((f⧆g)·dμ·x), `\int f\boxast g\,dx\,(1+h)`
(`integrand_split`, `integrand_before_differentials`; 57cj.21; A/B km21: 0 corpus readings changed, latent); outside an integral a bound differential closes the operand too,
`a\mathbin{\#}b c\,\dd x` #(a, b c)·dx (was (a#b)·c·dx). An operand that opens with a differential closes no integrand
and stays whole, a bare `d` or a bound one (57cj.19.6-57cj.19.8): `a\mathbin{\#}\dd\omega\,\eta` #(a, dω·η) (an exterior
derivative; Perl #(a, d(ω·η))), `a\mathbin{\#}\dd\omega\,\eta\,\dd x` #(a, dω·η·dx) (Perl #(a, d(ω·η·dx))),
`\int f\mathbin{\#}\dd x\,g\,\dd y` ∫(#(f, dx·g·dy)) (Perl ∫(#(f, d(x·g·dy)))), `a\mathbin{\#}\dd x\,\dd y` #(a, dx·dy), where
Perl's one-factor rule splits the braced and `\qty`-fenced physics forms (`\int f\mathbin{\#}\dd{x}\,\dd{y}` ∫(#(f, dx·dy)), Perl
∫((f#dx)·dy); `\int f\mathbin{\#}\qty(\dd x)\,g` ∫(#(f, dx·g)), Perl ∫((f#dx)·g); `\dd[3]{x}\,g\,\dd y`, `\dd{x}\,g\,\dd{y}`,
`\qty(\dd x)\,g\,\dd y` read Perl's split on 57cj.19.7) and
a bare `d`: `\int f\boxast dx\,g` ∫(⧆(f, dx·g)), `\int f\boxast dx\,dy` ∫(⧆(f, dx·dy)), `\int f\boxast dx\,g\,dy`
∫(⧆(f, dx·g·dy)) (Perl ∫((f⧆dx)·g), ∫((f⧆dx)·dy), ∫((f⧆dx)·g·dy)); an integrand before the BINOP is no matter,
`\int dx\,f\boxast dy\,g` ∫(⧆(dx·f, dy·g)) (Perl ∫((dx·f⧆dy)·g)), and `\int dx\,f\boxast g\,dy` ∫((dx·f⧆g)·dy), as Perl. A raised bare `d` before its variable is
the volume element since 57cj.20 (#395), a differential as `\dd^2` and iopart's `\rmd^2` are: `\int f\boxast g\,d^2x`
∫((f⧆g)·(d²)@(x)), and an operand that opens with it stays whole, `\int f\boxast d^2x\,g\,dy` ∫(⧆(f, (d²)@(x)·g·dy)); Perl's
`IntFactor` has no raised `diffd` (MathGrammar:643-651: `diffd ATOM_OR_ID`, the atom right after a bare `d`) and reads
∫((f⧆g)·d²·x), ∫(⧆(f, d²)·x·g·dy) (57cj.19 read ∫(⧆(f, g·d²·x)) and Perl's split).
Around a big operator a BINOP is a MulOp, as Perl: right
before one (`a\boxast\sum_i b_i c_i` ⧆(a, ∑(b_i c_i)); unparsed before 57cj.19.2) and in its wider operand (`\sum_i a_i\boxast b_i`
∑(a_i⧆b_i), was ⧆(∑a_i, b_i); `pragma_bigop_prefer_wider_absorption`'s `is_product`). A trailing ellipsis after a trig
argument leaves it (user ruling 15): `\sin x\boxast\cdots` ⧆(sin x, ⋯), Perl sin@(x⧆⋯), as `\sin x\cdot\cdots` sin@(x)·⋯.
A juxtaposed product that ends in a big operator is its whole operand too (57cj.22, the large-MULOP plan's step C2,
`juxtaposed_term`): `a\boxast 2b\sum_i c_i` ⧆(a, 2b·∑c_i), Perl ⁢(⧆(a, 2), b, ∑c_i) (unparsed before 57cj.22).
Q11 (user ruling 2026-10-01) gives the large MULOPs (⊗, ⊙ and their circled and boxed family) the same whole operand, #396.
Guards `tests/parse/product_chains.tex`
("A BINOP, a different operator of the same meaning, a coloured operator", "A product ending in a big operator after a
MulOp"), `tests/parse/physics_package.tex` (the physics
differentials).

### 56. An integral's raised `d` before its variable is the volume element (formerly OD #395)

Perl's `IntFactor` reads a differential only from a bare `d` and the atom after it, scripts on the atom included
(`diffd ATOM_OR_ID addScripts`, MathGrammar:643-647), so a raised `d` is a letter's power: `\int d^3x\,f` integral@(d³·x·f),
`\int d^nx\,f(x)`, `\int\mathrm{d}^2r` — the volume element of physics papers (`d^4x\sqrt{-g}`, `d^dx`, `d^3k`), read as a
power of a variable `d` times `x`. A bound differential raised the same way is one (iopart's `\rmd^3x` (differential-d ^ 3)@(x),
physics' `\dd[3]x`), as are Perl's.

**Rust** (57cj.20, 57cj.20.1; `raised_differential_d`, `differential_d_power`, `differential_power_apply`): an integral's `d`
(the lexer's `XDIFFUNK`, only in an integral's operand, `util::in_an_integral_operand`) raised to a count before a bare or post-scripted variable is the
differential's power applied to it, `(differential-d ^ 3)@(x)`, the markup of `\rmd^3x`: `\int\!d^3x\,f` ∫((d³)@(x)·f),
`\int d^2x\,d^2y\,f`, `\int d^nx_1\,f`, and as an integrand's last factor `\int f\,d^2\mathbf{r}`. A count
(`is_a_power_count`) is a number, a single-character letter other than the transpose `T` (a named one is not: `d^\alpha x` stays d^α·x), or a sum, difference or product of counts
(`d^{d-1}x`, `d^{2N}x`); not a power: a prime (`d'x`), a star (the codifferential `d^*`), a dagger, a negative number
(`d^{-1}x`), a group (`d^{(2)}x`, an order as `x^{(2)}` is), a symbol (`d^\infty`, the Haar measure's `\mathrm{d}^\natural h`,
2605.01646), an accented letter (`d^{\hat n}`), the transpose (`\exp(-d^{\mathsf T}x)`), a subscript (`d_1x`, `d^2_xy`), and — a residual — a
subscripted count, whose script the lexer spells as one atom (`\int{\rm d}^{n_{\mathrm{lin}}}\theta_{\mathrm{lin}}` d^{n_lin}·θ,
2605.21436), and a decorated measure in an operand (the Haar measure `\int…\,\mathrm{d}^\natural h` d^♮·h, 2605.01646: the integral
keeps no differential), as Perl reads both. Only in
an integral's operand as the tokens show it — an INTOP before the `d` at its level with no relation, arrow or wide
punctuation between (`util::in_an_integral_operand`, the lexer's test for every letter `d` since 57cj.21, SYNC (17); where Perl reads
`diffd` at all, `moreIntOpArgFactors`, MathGrammar:633-638): a dimension stays a letter's power before the
integral sign, `L_3^2d^2h^2\int_t^{t+h}…` d²·h² (2605.07939), `\leq 9\tilde L_f^2d^2h\sum\int…` (2605.26800), and after a
relation, `\int f\le C d^2 n`, `\int f=O(d^2n)`; the 57cj.20 review found the corpus rows read (d²)@(…) on 57cj.20. A relation
inside the integral's limits or a script is no end (the lexer makes it an atom): `\int_{s<u<t}d^2x\,f` keeps (d²)@(x); nor is one
inside a group closed before the `d` (`\int\mathbb{1}\{x\le y\}\,d^2x` (d²)@(x) since 57cj.21, was d²·x). Ranked with the bare differential
(`LetterDsBeforeVariablesAreDifferentials`). Corpus: 660 formulas in 79 of the 3,003 A/B papers read a `d ^ n * x` product
(m58c; 2605.29990 `\int d^{4}y`, 2605.21314 `\int\!d^{4}x`, 2605.23046 `\int\mathrm{d}^{2}r=\int_{-\infty}^{+\infty}\mathrm{d}^{2}r`,
2605.30421 `\int d^{d}xd\tau`); the km20 A/B (m58g → m35) changed 476 readings in 51 papers this way. A fraction's numerator is
a formula of its own with no INTOP, so `\int\frac{d^3k}{(2\pi)^3}` keeps d³·k, as `\frac{dk}{2\pi}` keeps d·k. Guard
`tests/parse/integrals_and_differentials.tex` ("An integral's differential takes a scripted variable").

### 57. A large product operator keeps its juxtaposed operand whole (formerly OD #396)

Perl reads every MULOP with `moreFactors` (MathGrammar:252-258: `MulOp Factor`, each `ApplyNary`d on the product so far), so
an explicit MulOp takes one factor on its right and a juxtaposition after it multiplies the result: `a\otimes 2b` (a⊗2)·b,
`2\Lambda_1\otimes\Lambda_1\otimes 2\Lambda_1\otimes\Lambda_1` (((2Λ₁⊗Λ₁⊗2)·Λ₁)⊗Λ₁, `g\otimes w\otimes\sigma'(\gamma_i)` (g⊗w⊗σ′)·γᵢ,
`a\otimes_k DB` (a⊗_k D)·B. Rust read so from 57cj.19 (SYNC "Math-parse residuals" (14)) until the ruling.

**Rust** (user ruling Q11, 2026-10-01: juxtaposition binds tighter than a large MULOP; 57cj.20.Q11, `is_a_large_mulop`,
`infix_apply_nary`, `apply_invisible_times`): a large product operator — ⊗ `\otimes` (tensor-product), ⊙ `\odot`
(direct-product) and the circled and boxed operators of the same size, ⊘ `\oslash`, ⊚ `\circledcirc`, ⊛ `\circledast`,
⊠ `\boxtimes`, ⊡ `\boxdot`, stmaryrd's ⦸ `\varobslash`, and (the Q11 scope ruling, 2026-10-01; 57cj.21) the semidirect products
⋉ ⋊ ⋋ ⋌, the coproduct ∐ (`\amalg`, U+2210), the circles ○ `\bigcirc` and ◯ `\varbigcirc`, mathabx's box product □ `\square` and
`\pluscirc` (2605.11552 `\pi_1(T^3)\rtimes\pi_1(N_0)` (π₁T³)⋊(π₁N₀), 2605.15276, 2605.27086 `\operatorname{Diff}(M)\ltimes\Omega^2_{\mathrm{sym}}(M)`;
2605.12221 a control; A/B km21: 9 readings in 7
papers, 7 better, 1 neutral, 1 worse where a `\text{ and }` connective joins the operand, 2605.22466), bare or decorated — keeps
its juxtaposed operand whole, as a BINOP does (#393):
(2Λ₁)⊗Λ₁⊗(2Λ₁)⊗Λ₁ (2605.17901), g⊗w⊗(σ′·γᵢ) (2605.01702), `P\odot P\odot P_\theta(x|y)` P⊙P⊙(P_θ·(x|y)) (2605.00423),
`c\boxtimes T^{(r,n)}c` (2605.29990), `R\otimes_{\mathbb C}\mathbb C G` R⊗_ℂ(ℂG) (2605.14864), `a\otimes_k DB` a⊗_k(DB). In a bare
argument's chain, where the grammar joins the next item to the chain so far (`trig_arg factor_base`, `op_bare_arg
op_bare_next`), the large MULOP's application takes the juxtaposed item into its last operand: `\sin x\otimes y z`
sin@(x⊗(y z)), `\log a\otimes b c`, `\nabla u\otimes v w`, `\sin x\otimes 2\pi t` sin@(x⊗(2πt)) (Perl sin@((x⊗y)·z)) — not a
differential, and an ellipsis only between two items (`\log x\otimes y\cdots z` log@(x⊗(y·⋯·z)); a trailing run leaves,
`\log x\otimes y\cdots` log@(x⊗y)·⋯, user ruling 15).
Before an integral's differentials — or the letter `d` of one, so the letter twin meets the differential's reading at the
same shape (`\int_0^1 f\otimes g h\,dx` read ∫(f⊗(g·h·d·x)) in the first cut) — it takes the integrand's factors before them,
one juxtaposed operand:
`\int f\otimes g h\,dx` ∫((f⊗(g h))·dx) (Perl ∫((f⊗g)·h·dx); a BINOP too since 57cj.21, #393); an operand that opens
with a differential stays whole, `\int_X F(x)\otimes d\nu(x)` ∫(F(x)⊗(dν·x)) (2605.25146). `\cdot` keeps Perl's one-factor
chain grouping (57cj.19), `\circ` (f∘g)·x, and `\times`, `\star`, `\ast` their one factor (the semidirect products, ∐, the circles,
mathabx's □ and `\pluscirc` took one factor too until the Q11 scope ruling, 57cj.21: the 57cj.20.Q11 plan check counted 4
corpus formulas with a semidirect product that read better whole, 2605.11552, 2605.12221, 2605.15276, 2605.27086). The plan check's corpus count (km201 B outputs, 3,003 papers):
⊗ 344 formulas in 43 papers and ⊙ 60 in 25 split a juxtaposed operand at a term's level, ⊠ 35 in 2, and 11 inside a bare
argument (2605.17112, 2605.25134). Accepted ambiguity: an operator before a trailing group, `{\mathrm{tr}}\otimes
I_{\mathcal X}(a)` (2605.21982), now reads tr⊗(I_X·(a)); a Kronecker product before a vector, `A\otimes B\,x` A⊗(B·x), where
(A⊗B)x may be meant (latent, no corpus formula); math braces are transparent, so `{a\otimes b}c` and `\boldsymbol{a\otimes b}c`
read a⊗(b·c) (`\mathord{a\otimes b}c` keeps (a⊗b)·c); a mixed number in a bare argument's chain joins as a product,
`\log x\otimes 2\frac34` log(x⊗(2·¾)) beside `x\otimes 2\frac34` x⊗(2+¾). A juxtaposed product that ends in a big operator is the operand too (57cj.22, `juxtaposed_term`): `a\otimes b\sum_i c_i`
a⊗(b·∑c_i), Perl (a⊗b)·∑c_i (unparsed before 57cj.22). Guards `tests/parse/product_chains.tex` ("A large product operator keeps its
juxtaposed operand", "The semidirect products, the coproduct and the circles", "A product ending in a big operator after a
MulOp"), `tests/parse/large_mulops_mathabx.tex`.

### 58. An integral's letter `d` stays a differential past a sign, a colon and narrow punctuation (formerly OD #398)

Perl reads a letter `d` as a differential only among an INTOP's arguments (`addIntOpArgs`/`moreIntOpArgFactors`,
MathGrammar:620-647): a product of `IntFactor`s, which ends at an additive operator, a relation, any punctuation and never
enters a group. So `\int_M a+b\,dV` reads b·d·V, `\int|\mathrm dP-\mathrm dQ|` d·P−d·Q, `\int f\,P(y,dx')` and
`\int K\tilde\mu(ds,de)` read their measures' `d`s as letters.

**Rust** (user ruling (17), 2026-10-01; 57cj.21, `util::in_an_integral_operand`, applied by the lexer to every letter `d`, which
then offers its differential lexeme `XDIFFUNK`/`XDIFFID` only there): an INTOP before the `d` at its level, with no relation,
arrow or wide punctuation (`,\quad`, a `\qquad`) between at that level. The `d` keeps its differential reading — as before 57cj.21, whatever the grammar's tree for the integral (`\int f-g\,dx` reads
integral@(f) − g·dx, the Frobenius integral (∫H) : Q·dx: a differential outside its integral's tree, a residual) — past a sign (697 sum-integrand
sites in 129 of the 3,003 A/B papers: `\int_{\bar M}Q_t(w-1)+Qw_t\,\mathrm{d}V_{0,0}` 2605.17001, `\int|\mathrm{d}P-\mathrm{d}Q|`
2605.15207, `\int_s^t-V\,\mathrm{d}W_\tau-\frac12V^2\,\mathrm{d}\tau` 2605.17850), past a colon, a product in an operand (the Frobenius
`\int_\Omega\mathbf{H}:\mathbf{Q}_t\,d\mathbf{x}`, 2605.24758, 2605.12883, 2605.19415), past narrow punctuation (mostly a comma typed for `\,`,
`\frac1L\int_0^L\mathbf u\cdot\mathbf e_x,dx`, 2605.21567, 2605.04013), and into groups (the measures `\int f\,P(y,dx')` 2605.08485,
`\int_E K\tilde\mu(ds,de)` 2605.20593). A group closed before the `d` is skipped whole (parentheses, brackets, a `\left|…\right|` pair; a plain `|…|` does not pair
by position, `|\int f\,dx|\,d\pi` keeps differential-d@(π)): `(\int_0^1 f\,dx)\,d\pi` d·π, and a
relation inside it ends nothing (`\int\mathbb{1}\{x\le y\}\,d^2x` (d²)@(x)). Letters: a `d` before the integral sign, after a
relation, arrow or wide punctuation at the integral's level, in a script, or bound by a big operator in between until a sign
ends that operator's operand — the letter, a list item, or a relation's, relation chain's or condition's operand other than
its last (`\int f\,dx+\sum_{d=1}^D d\,w_d` d·w_d, `\sum_{1\le d\le D}`, `\sum_{d\mid n}`, `\sum_{d,e}`; `\sum_{i\le d}` binds `i`). A/B km21 (m58i → m49): 120 readings in 35 of the 3,003 papers lost a differential — 11 right, 92 ruling-conformant,
17 residual (SYNC (17)). The design review found the ADDOP stop of Perl wrong in
all but one corpus case (2605.26800's second `dh`, pinned) and the colon stop wrong in all 28. Guard
`tests/parse/integrals_and_differentials.tex` ("A letter d is a differential only inside an integral's operand").

### 59. A letter `d` outside an integral is a differential where the formula shows one (formerly OD #401)

**Perl** reads a letter `d` as a differential only among an integral's arguments (`diffd`, `moreIntOpArgFactors`,
MathGrammar:633-638): outside an integral `dX_t=\mu\,dt+\sigma\,dW_t` is d·X_t = μ·d·t + σ·d·W_t, a measure's `\pi(du)` π@(d·u), a
metric `ds^2=dx^2+dy^2` d·s² = d·x² + d·y².

**Rust** (57cj.22, user ruling 2026-10-01 widening SYNC (17); `util::DifferentialEvidence`, `util::continues_an_open_integral`):
the lexer offers the differential reading of a `d<var>` outside an integral's operand where the formula shows evidence of
it — the grammar keeps both readings and `LetterDsBeforeVariablesAreDifferentials` prefers the differential — and the plain
letter elsewhere. The evidence, each a corpus shape (A/B km21's lost readings; precision sampled over the 3,003 A/B papers by
the design review, `~/data/pk_agents/math/r22/rc1/`): an upright `d` (`\mathrm d`, `{\rm d}`; since 57cj.23.6 only through the
document evidence below), two or more such `d`s in the formula with no other `d` a variable there (an SDE, a differential form, a line element, a
Leibniz quotient `dJ/dK`), a whole item of a group applied to a letter (a measure's set, `\pi(du)`, `F(du\mid x)`,
`\widetilde N(ds,dz)`; not after an order symbol `O`, `\mathcal O`, Θ, Ω, `C`), the opening of a formula's first side before a
relation with an integral in the formula (`dU(z)=-\int…`), a wedge (`dx\wedge dy`), and a continuation row of an alignment
that continues an integral the row before it left open (57cj.23, the merge review: only a continuation row — its column
pair's first cell empty, or opening with a sign or a MulOp — in an alignment, not a gather, and only the row right before it,
the same column pair's; each INTOP opens one, each differential closes one; inside it up to the row's first relation; asked
only of a `d` the formula leaves open at its start, so the cost is one row's scan — a 1,200-row alignment 59 s on 57cj.22,
7 s now, `tools/perfect_kernel/repros/math-parse/continuation_rows_scale_linearly.tex`). No `d` reads so
that heads an italic word (`dist`) or is Pearl's `do(`, nor, upright or opening a side, one whose variable runs on unspaced
into a letter (`1+\mathrm dt^2L_R^2`); a raised order is an integer or a letter. A dimension (`\leq d\pi^d`, 2605.03853), a
coefficient (`\mathcal O(dh)`, `Cdh^3`, 2605.26800), a distance `d(x,y)` and a lone `d<var>` stay letters. Witnesses
2605.13204, 2605.14643, 2605.20593, 2605.27600, 2605.26640, 2605.04766, 2605.02070, 2605.18724, 2605.17850, 2605.26170,
2605.15926, 2605.26008, 2605.30839. Known imprecision: two dimension coefficients in one formula (`O(dn+dm)`) read as
differentials (~8 % of the sampled two-`d` formulas); an upright `\mathrm dt` used as a time step read a differential
(2605.29194, `x+\mathrm dt\,b(x)+\sqrt{\mathrm dt}\,\sigma\xi`) until the document evidence below. Guard `tests/parse/integrals_and_differentials.tex` ("A
letter d outside an integral is a differential by evidence"); `tests/theorem/{amstheorem,latextheorem}.xml` (the line
elements `ds^2=h(z)|dz|^2`, `\{ds_1^2,\dots,ds_k^2\}`, letters in Perl).

**Document evidence** (57cj.23.6, user rulings 2026-10-01c/d/e, superseding the upright font as evidence on its own:
in an intent study of the 377 formulas it alone made differentials, 42 of 50 sampled were differentials, 6 time steps).
Before any formula is parsed, `util::DifferentialMap` reads every formula of the document for what it shows of its `d`
tokens — a token's identity is its font's family, series and shape (`d_identity`) —: which identities are ever a scalar
(a `d<var>` in a scalar context; a lone `d` gives no scalar identity — an italic one is every document's default
letter, recorded only for the `\in` veto below unless it is a Leibniz quotient's operator; an upright one an operator
or a label, `\frac{\mathrm d}{\mathrm dt}`, `\mathrm d^2=0`, `M_{\rm d}`), and which
`(identity, variable)` pairs are a scalar or a differential somewhere — a differential where an integral reaches the `d` cleanly, in any container (an exponent's
`e^{-\int r\,\mathrm ds}`), with no sign, punctuation or plain bar between (past those the walk alone puts it in the
operand, `\int f\,dx;\,d\pi`, and the reading would spread), or a Leibniz quotient's part. A pair is keyed by the
variable's own token (`dW_t` by `W`). A `d<var>` then reads, in this order: a letter in a scalar context of its own formula
— a root, a script, a denominator whose numerator holds no `d` anywhere (`\sqrt{\mathrm dt}`, `x_{t+\mathrm dt}`,
2605.09779, 2605.29194), `d<var>\to 0`, a side assigned a nonzero number when no other `d<var>` is in the formula
(`\mathrm dt=1/12`; an equation between differentials outranks it, 2605.09717), an order symbol's argument
(`\mathcal O(dh)`, 2605.00250) —; a differential on its formula's shape — a Leibniz or Radon-Nikodym quotient's part,
italic too, whose numerator is a `d` (with its power) alone or before a variable, an accented letter, a function or a
`d`-free group (`numerator_of_a_quotient`: `\frac{d\vec r}{d\tau}`, `\frac{d\ln Z}{d\beta}`, `\frac{d(uv)}{dz}`) and
whose denominator's `d` takes a variable (`\frac{d\mathbf v}{dt}`; not `\frac{dn}{d+2}`, `\frac{d-1}{dk}`), two or more
`d<var>` and no other `d` a variable (an SDE, a form, an increment), a measure's item, a wedge, the first side before
a relation with an integral, a Fréchet derivative applied to a direction (`\mathrm dH(u)[\psi]`, 2605.04766) —; a letter
when its variable runs on, a bold `d` is bolder than its variable (`\mathbf{d}x`: the bold font is not evidence), or the
formula itself uses the same `d` token as a scalar (`\int_{[-1,1]^d}f\,dx\leq d\pi^d`, 2605.03853; a big operator's
index, `\sum_{d=1}^D d\,w_d`; a lone italic `d`, or one in a script outside an integrand); a set `\in d<var>`, an
infinitesimal set (`\tau_1\in dt`), a differential — not before a set's letter (blackboard, calligraphic, bold or a
capital: `k\in d\mathbb Z`, `y\in dZ`), and not where the document uses the same `d` as a number (`d\geq 2`, `q\in dn`)
unless the set is a measure's argument (`P(X_t\in dx)`, `\Pr(Z\in dz)`, `\mathbb P\{Y\in dv\}`; whether a set-builder
event `\{\omega: X(\omega)\in dv\}` is one awaits a ruling); then the document: the same pair a scalar elsewhere, a letter; a differential elsewhere,
a differential (`\mathrm dS` beside `\int_\Sigma f\,\mathrm dS`, 2605.16486); otherwise an upright `d` the document never
uses as a scalar is a differential (2605.18213 `0=\mathrm dL+\tfrac12[L,L]`) — the ruling's population, a document's
convention token —, and an italic `d`, every document's default letter as well, stays a letter (`\int f\,dx\le d^2n`,
`abcde`). The evidence is read before the document's rewrites, over the whole document in both modes: eagerly before
`finish_document`, and under `--streaming` from each subtree as it spills (`Document::spill_observer`) and from the spine
before pass 2, so a segment parsed in pass 2 reads the same evidence as the eager parse
(`latexml_math_parser::read_differential_evidence`); a reference resolves within its own formula only, in both modes. Guards
`tests/parse/differential_*.tex` (17 documents), `tests/parse/{integrals_and_differentials,
differential_only_in_the_operand, continuation_row_with_its_own_side}.tex`,
`113_streaming_core::streaming_differential_evidence_is_the_whole_documents`,
`stream_a_recall::gls_in_math_typesets_the_term_alone` (the Leibniz quotient). Known imprecision: two `\mathrm dt` Euler
steps in one formula read a form (2605.29194). Open: the numerator `d` of an operator quotient `\frac{d}{dt}` stays a
letter; `d\frac{…}{…}` as an operator; a decorated variable (`d\tilde x`).

**One local verdict, the precedence completed** (57cj.23.7, after the 57cj.23.6 arXiv A/B — 2,253 formulas gained a
differential, 148 lost one, 41 of them against the rulings — and user rulings 2026-10-02). The lexer and the document map
read a `d<var>`'s own formula through one verdict (`DifferentialEvidence::local`: a differential, a scalar context, a
letter for this formula, or open), so the map records a differential pair from every local evidence — an equation
between differentials, a form, a measure, a wedge, a Fréchet direction —, not only from integrands and `\frac` Leibniz
quotients (2605.00250's lone `\mathrm d\bm f` beside `\mathrm ds^2=\frac{\mathrm dt}{g^2}\|\mathrm d\bm f\|^2`). In order:
a Leibniz quotient's part, a fraction's or a slash's (`\frac{\sigma(E')}{(dE'/dx)}`, 2605.21289; `\frac{1}{\mathrm
ds/\mathrm d\lambda}`, 2605.29065; `_{=\mathrm dh(s)/\mathrm ds}`, 2605.09779), outranks any scalar container, and its `d` is
no scalar use of the token in the document (2605.29065's `\mathrm du_K`, `\langle\mathrm dg^i\rangle`); a measure's
argument outranks a script (`\|F\|_{L^1(d\mu_X)}`, 2605.02556 ×13; `L^1(\mathbb T,dm)`, 2605.02034); an under- or
overbrace's label is no script, it restates the braced term (2605.15276); an order symbol's argument yields to the same
`d<var>` a differential elsewhere in its formula (`\phi+d\phi … o(\|d\phi\|^2)`, 2605.00265); a relation's first side
is a differential with a wedge past the relation as with an integral (Maurer–Cartan `\mathrm d\eta^a=-\frac12
f\eta^b\wedge\eta^c`, 2605.15276); and a root's or a fraction part's content inside an integrand is in it, though parsed
on its own (`in_an_enclosing_integrand`: `\int\sqrt{dP\,dQ}`, 2605.09119; `\int\frac{dx_1}{x_1}`, ruling 2026-10-01c). A
root outside an integrand stays a scalar context (`\sqrt{-g\,dx^\mu dx^\nu}`, `\sqrt{du+dv}`; ruling 2026-10-02).
Guards `tests/parse/differential_{leibniz_beats_its_container, leibniz_denominator_is_no_scalar_use,
integrand_through_a_root, wedge_relation, document_pair_from_an_equation, measure_in_a_script, brace_label_restates,
same_pair_in_a_landau_argument}.tex`.

**A `d` inside a word is a letter** (57cj.23.8, after the 57cj.23.7 arXiv A/B km27 — 843 formulas in 135 papers gained a
differential, words among them). A `d` with two or more single Latin letters of its own font run into it unspaced before
it is a letter, as a `d` heading a word, whatever its formula or the document says of its pair (`util::inside_a_word`):
a word repeated in a formula had read as a form — `MidX`, `OldY` (2605.14140), `D(node)` (2605.00265), `node.child`
(2605.00751), `selected\_node`, `candidate\_terms`, `PruningMode` (2605.29184) —, and an acronym took the document's
pair (`qKdS_3`, 2605.02523). One letter before stays a product (`TdS`, `pdV`, `\mu dN`), and so do a `d<var>` before it
(`dxdy`, 2605.01526) and letters of another font (`\mathrm d\bm x\mathrm dt`, 2605.00545). A/B km29 (n14 → n17,
`~/data/pk_agents/math/abm29/`): every tally identical but warnings −2 (2605.14140's word formulas, no longer
ambiguous); 56 readings in 7 papers, every one a `d` back to a letter and right — 48 words, the acronym ×6, two products
of scalars (`2Bndm` 2605.00392; `uLdn` 2605.11864, `d` a model width). Not taken: keeping a relation's first side beside
an integral local to its formula — its witness's spread came from a repeated pair, not a first side, and the shape is
the document's evidence for a force element (`dF_{\rm add}(z)=-c\int_0^{R_p}\rho\,d\rho`, 2605.26170, four lone `dF_{\rm
add}(z)` kept). Open, the repeated-pair form: one `d<var>` twice in a formula reads as a form and, through the document
map, spreads to every like `d<var>` of the document (a dimension `d`: 2605.02317 `-dN^2\sum_k\|…\|_1-dN^2(…)` and its
every `dN^2`; 2605.11864 `c_{\rm att}dn^2+c_{\rm ffn}d^2n`) — the known imprecision of two dimension coefficients above,
widened by the map. Guard `tests/parse/differential_inside_a_word_is_a_letter.tex`.

A differential's numeric power is the differential's, an index the variable's, and the variable takes its argument group
(57cj.23, the merge review; `diffop_apply`, grammar `applied_differential_variable`): a line element `ds^2=dX^2+dY^2` (ds)² =
(dX)² + (dY)², `\frac{d^2y}{dx^2}`'s and `d^2x/dt^2`'s denominator (dt)², `dx^\mu` d(x^μ); `dU(z)` d(U(z)), `\mathrm dX(s)` d(X(s)),
`d\mu(x)` d(μ(x)), `\mathrm dH(u)[\psi]` d(H(u))·[ψ] (were d(s²), d(U)·z — the reading Perl's IntFactor gives an integral's
`d x^2`, `d\mu (x)`). Guards `tests/parse/integrals_and_differentials.tex`, `tests/theorem/{amstheorem,latextheorem}.xml`.

### 60. A closed group holding only integrals is an integral operator (formerly OD #402)

**Perl** reads a bracketed sum of integral signs as a group multiplying what follows it, and the `d` after it a letter
(the integral signs inside take no operand; `diffd` is an INTOP's argument only, MathGrammar:633-638):
`\left[\int_G+\sum_{i=1}^6\int_{a_i}^{b_i}\right]f(\theta)\,\mathrm d\theta` delimited-[]@(∫_G + ∑∫)·f·θ·d·θ.

**Rust** (57cj.22, user ruling 2026-10-01; `util::integral_operator_group`, the grammar's `integral_operator_group`): a group
whose own level holds only integral signs — each an INTOP, or a big operator before one, with their scripts — joined by signs,
no operand, is an integral operator: the lexer marks its OPEN (`INTOP_GROUP_OPEN`), the grammar applies it to the integrand
after it as an INTOP (`summation_bigop`), and the integrand's `d`s are differentials (`in_an_integral_operand`):
(delimited-[]@(∫_G + ∑∫))@(f(θ)·dθ). Witnesses 2605.15451 (12 readings), 2605.02925 (`\left(\int_{-\infty}^{-\varepsilon}+
\int_\varepsilon^\infty\right)…\mathrm dz`). A group with an operand (`[\int_a^b f\,dx+\int_c^d]`), a lone big operator
(`[\sum_i]`) or no integral stays a group — an operand being any node but a big operator's scripts, a fraction, root or
accent too (`\left(\int_\Omega\frac{|f|^2}{w}\right)^{1/2}\left(\int_\Omega\frac{|g|^2}{v}\right)^{1/2}` stays the product of two
integrals, `2\left(\int_0^1\sqrt{x}\right)y`; 57cj.23, the merge review) — and only a parenthesis, bracket or brace group is
one (`\langle\int_a+\int_b\rangle` stays a fence). Guard `tests/parse/integrals_and_differentials.tex` ("A closed group holding only
integrals is an integral").

### 61. mathtools' sized paired delimiters are `\<size>l`/`\<size>r` (Perl: the size itself) (formerly OD #451)

Perl's defect: KNOWN_PERL_ERRORS #497.

The sizing and grouping of `\DeclarePairedDelimiter` and `\DeclarePairedDelimiterX`/`XPP` follow mathtools.sty:874-990
(not its `\mathopen`/`\mathclose` wrappers or its scaled/unscaled wrapper split): a sized delimiter
`\abs[\big]{x}` is `\bigl\lvert x\bigr\rvert` (`\@nameuse{\MH_cs_to_str:N ##1 l}`), the X forms in a group with
`\delimsize` set before the delimiters. Perl (mathtools.sty.ltxml:677-692) writes `\big\lvert x\big\rvert`, and for the X
forms outside a group (KNOWN_PERL_ERRORS #497). The ams `mathtools` golden changes in three `tex` attributes only
(`\bigl`/`\Bigl`/`\Biggl` for `\big`/`\Big`/`\Bigg`); the parsed math is the same. Witness 2609.17447.

**Guard**: `perfect_kernel_batch61::paired_delimiter_size_and_delimsize`, `56_ams::mathtools_test`.

### 62. `\left`, `\middle` and `\right` scan their delimiter as tex.web §1160 does, and `\middle`/`\right` require the `\left` group (Perl: `TeXDelimiter` invokes any token, no group check) (formerly OD #530)

tex.web §1188 makes `\left`, `\middle` and `\right` one unexpandable command, `left_right`; §1160 `scan_delimiter`
reads its delimiter when it executes; §1191 opens a `math_left_group` for `\left` and requires one for `\middle` and
`\right`. Until 65d the port deviated (Rust-only): `\left` was an expandable macro reading its delimiter as an XToken
and trampolining to `\lx@delim@left` + `\lx@hidden@bgroup`, `\lx@right` a macro of the same shape, `\middle` unchecked.

**Delimiter scan** (`tex_math::scan_delimiter`, the `DelimiterToken` parameter; Perl `TeXDelimiter`,
TeX_Math.pool.ltxml:712-724). `get_x_token` past blanks and `\relax` (protected macros expand, as at execution in TeX),
then by meaning:
- a letter/other character, or a `\let` copy of one, with a `\delcode` ≥ 0 or in the delimiter map (the base route
  loads no `\delcode` table; a Unicode fence typed directly is what unicode-math's `\Udelcode` gives one);
- `\delimiter` (§1160 `delim_num`) and `\Udelimiter`, or a `\let` copy, matched by the defining cs: the code is scanned
  right there and becomes its glyph's character token (the map's character when the glyph is in the map, else the
  glyph — the null delimiter before);
- a LaTeXML math symbol: a `DefMath` symbol, or a parameterless primitive/constructor other than `\left`/`\middle`/
  `\right` that typesets one — digested in a group of its own it gives a glyph box or a construct with a math role
  (`\lmoustache`, `\lx@math@lbrace`; `typesets_a_symbol`) — LaTeXML's stand-ins for TeX's `\delimiter` macros; TeX
  would refuse a non-delimiter symbol such as `\alpha`, which these cannot tell apart. `\displaystyle`, `\nonumber`,
  `\par`, `\hfill`, `\,` typeset nothing to fence with and are "Missing delimiter". The trial digestion reports
  nothing (Perl's `IGNORE_ERRORS` probing scope, `set_ignore_diagnostics`): the command reports its diagnostics once,
  where it is digested;
- an undefined control sequence passes through (its digestion reports it);
- the input ending (an argument's own tokens, `\frac{a\left}{b}`): the null delimiter, with the Warning below;
- anything else — `{`, `}` (or `\bgroup`/`\egroup`), `$`, `&`, a command reading arguments (`\big`, `\mathopen`, `\vphantom`), a register — is
  TeX's "Missing delimiter (. inserted)": the token put back, the null delimiter returned. A Warning (user ruling
  2026-10-11: a TeX-only error whose recovery is semantic — pdflatex errs, Perl is silent, the token is still typeset).

In text mode (a `\left` TeX would have refused after inserting `$`, reached where a beamer frame stripped the `$`) every
token is taken, as Perl takes it. A brace takes TeX's path (§1160-1166), not Perl's unwrap (`readBalanced` on `{`, which
turns the typo `\left{ x \right}` into a delimiter `x`) nor 65a's reading as `\{`: a brace cannot tell the typo from a
brace-bodied macro (`\newcommand\lb{{[}}`, LaTeX's `\dag`), and the PDF shows no brace either way; the typo then reads
as pdflatex shows it — null delimiters, `x` in a group, the trailing `}` deleted (16 of 3,194 sampled papers hold
`\left{`/`\right}`, nearly all `\newcommand{\lt}{\left}`-style definitions). This closes both gaps WISDOM #41 recorded for `\left`: the
single-token read is §1160's loop, and the number of `\delimiter` is scanned at the token level, so Perl's
`undigested => 1` invoke (an `ArgWrap::Digested` or `Parameter.undigested`) is not needed. A delimiter outside the map
is emitted as digested and made stretchy, as Perl's `augmentDelimiterProperties` makes it (:800-823) — before, it
warned "Missing delimiter; '.' inserted" and kept `stretchy="false"` (`\llbracket`, `\lparen`, the corners,
`\lmoustache`, a right `\backslash`).

**Commands.** `\left` is `DefConstructor('\left DelimiterToken')` as in Perl, its after-digest unreading
`\lx@hidden@bgroup \lx@math@left@group`; the marker, the first thing digested in that group, writes the K9 group code
`lx@group@code = \left`. `\lx@left`/`\lx@right` are `\let` copies of the private `\lx@delim@left`/`\lx@delim@right`
constructors (alias `\left`/`\right`). `\right` keeps Perl's dispatching constructor (`\lx@hidden@egroup@right
\lx@right`); `\middle` becomes a primitive dispatching to `\lx@delim@middle` (alias `\middle`; Perl's `\middle Token` is
the constructor itself), so that a rejected `\middle` leaves nothing behind. When the innermost group is not the
`\left` group (`tex_math::left_right_group`):
- the formula's own group (a frame that switched mode — §1192 `math_shift_group`): "Extra \right." / "Extra
  \middle.", an Error as in Perl ("Unbalanced \right, no balancing \left."; `\middle` unchecked there). TeX drops the
  delimiter; it is kept, typeset where it stands without closing any group (user ruling 2026-10-12: author content
  first — the delimiter is the author's);
- a group the document opened with `{`/`\bgroup` or `\begingroup` (by `groupInitiator`): §1064 `off_save` — "Missing }
  inserted" / "Missing \endgroup inserted", the closer inserted and the command read again; a Warning (TeX-only,
  semantic recovery). The author's own closer has been used: the group `off_save` closed owes it (frame values
  `lx@owed@braces` / `lx@owed@endgroups`), and at every pop the debt moves to the group then innermost (stomach.rs
  `pop_stack_frame`), so it follows the stack to where the author's closer arrives. There a `}` meeting a group `}`
  cannot close — any group not opened by a begin-group token: the formula's math shift, the `\left` group, a
  `\begingroup` group, and the argument groups the engine opens to digest a constructor's argument (they carry a K9
  group code; the style parameters `DisplayStyle` … `InFractionStyle` write it, base_parameter_types.rs
  `mark_argument_group`) — is deleted as §1069 `extra_right_brace` deletes it ("Extra }, or forgotten $" / "\right." /
  "\endgroup", a Warning; stomach.rs `delete_extra_brace`, before the digestion loops could take it for a group end).
  TeX has no argument group of ours: LaTeX's `\frac` digests its argument in `\begingroup…\endgroup`, so the message
  for those is TeX's "forgotten \endgroup" (`\frac{\left( {a \right)}}{b}`, pdflatex the same), where for `\sqrt` or
  `\mathbf` TeX's group is a math group its `}` would close; and an owed `\endgroup` meeting a
  group it cannot close is deleted ("Extra \endgroup."); a closable group closes normally and passes the debt on.
  Before, the `}` met the formula's frame, inserted `$` and set the rest of the paragraph, and the paragraphs after
  it, as math (review r1/r3 of 65d; 1703.04019);
- any other group (one LaTeXML's machinery opened): Perl's recovery — `\right` closes it, `\middle` goes on.

**Group ends inside the `\left` group** (Warnings: TeX-only errors with a semantic recovery, user ruling 2026-10-11):
- a `}` closing it (stomach.rs `egroup`): "Extra }, or forgotten \right." — TeX deletes the brace (§1069); the group
  closes here, as Perl closes it silently. Deleting it as TeX does needs our `$` to close the `{` group it leaves open
  (§1065 inserts the `}`), so `$ {\left( a } b \right) $` keeps an "Extra \right." Error pdflatex does not give — +1
  Error vs 65a (review5 d2: 5 → 6), the content intact (pdftotext "A (ab) B. / Next.") (SYNC_STATUS row MS). Since 65n
  the `$` does insert that `}` (#553), so the deletion is unblocked; not done there (RED
  paragraph-mode/brace_meeting_left_group_in_inline_math_is_deleted);
- the primitive that meets it (§1065 `off_save` at the mismatching primitive): a formula's end (`$`, `$$`, `\]`, `\)`:
  `\lx@dollar@default`, `tex_math::close_open_left`), an `\endgroup` (stomach.rs `endgroup`), a DefEnvironment's own
  end constructor `\end{X}` (`\begin{equation}\left( x \end{equation}`; stomach.rs `is_environment_end` in
  `digest_next_body` — no TeX-level `\endX` exists to read again), each read again after the `\right.`, or the end
  of an argument's input (`\frac{\left( a}{b}`; stomach.rs `close_left_group_at_input_end` in `digest_next_body`): "Missing \right.
  inserted" and a null `\right.`. Only the end of the input the `\left` was read in closes its group: `\lx@math@left@group`
  records the reading context (`gullet::reading_context`, the serial and depth of the `reading_from_mouth` context, 0
  for the document's own input) on the group's frame, and an input's end closes the group when that context is the one
  ending or one nested in it that has already ended. A `\left` read outside the ending input stays open, as tex.web
  §1191-1192 keeps a `math_left_group` until its `\right` or an error: for TeX an argument's end is no end of input, the
  argument being a `{…}` group inside the `\left` group. A binding's `\left( \lx@xmarg{…}{#1} \right)` (physics.sty's
  `\left(#1\right)`) digests its argument from a mouth of its own, and from 65d to 66g that argument's end closed the
  binding's own `\left`: "Extra \right." and a dangling XMRef for `\cosh(a\over 2)` (2605.02768), an `\expval{a \over b}`
  in `aligned` whose `\right\rangle` met the row's `\begingroup` and lost the equation (2606.01486). Guards
  `perfect_kernel_batch66g::*`. A LaTeX environment whose end code holds the `\right` (`\newenvironment{paren}
  {\left(}{\right)}`) closes the group itself, unchecked (round 3's `\end`-time check broke it and looped in an
  `\edef`). Before, the formula ran on to the next `$` and swallowed the
  paragraphs after it. A math ender read again after the inserted `\right.` leaves an empty whatsit in the group that
  reverts as nothing (state value `lx@math@ender@reread`, so no thread-local outlives a document).

**The `}` a delimiter scan put back** (`$\left\{ x \right}$`, `\mbox{$\left\{ x \right}$}`): after "Missing
delimiter" the `}` is read again (§1160); meeting the formula's own math shift it is deleted per §1069, "Extra }, or
forgotten $" — an Error, as TeX and Perl err (a Warning when it pays debt) — whatever group is outside the formula
(tex_math.rs `missing_delimiter` → stomach.rs `mark_put_back_brace`). The mark lives as long as its brace: a serial
bound on the frame innermost when the brace was put back (inside the formula, so it dies with the formula at the latest),
valid only while it equals the live serial `lx@putback@live`, which the next `}` digested anywhere clears — in the
digestion loops' `delete_extra_brace` and at `invoke_token`, the choke point: a cell's `}` and any `}` inside a
`\noalign` body pass it; a loop's own closing `}` (the `\noalign` loop, tex_tables.rs:1291; the box-body loop,
base_utilities/boxes_lists.rs:120) leaves the live serial set, which is harmless because the frame holding the mark closes with it
(probe B1) (review r6: a global count outlived a cell's `}` and deleted a later, unrelated `}`; guard
`perfect_kernel_batch65d::put_back_mark_lives_with_its_brace`). pdflatex: the same two
diagnostics; pdftotext "Before {x c." for the `\mbox` shape. 65a closed the math at that `}` with 1 Error. The
general §1069 deletion needed §1046-1047 (`\par` or `\end{X}` in inline math inserts `$`) first, without which a
deleted `}` left the formula running on through the paragraphs; both landed in 65n (#553). Guards
`perfect_kernel_batch65d::{argument_group_owes_the_brace, stray_brace_at_the_formula_level_is_deleted}`.

**Residuals.** The `\right` delimiter is digested after the group's `\aftergroup` tokens run (TeX scans it before
`unsave`; Perl the same order as here), so mathtools' documented `\def\right{\aftergroup\egroup\@right}` leaves the
delimiter outside the `\mathclose` group (unparsed, as before). A `\right` inside a constructor's argument
(`\frac{a}{b \right)}`, `\mathbf{x \right)}`) drops the argument and the `)` silently (RED
boxes-groups/right_inside_an_argument_drops_the_argument; 65a the same). A stray `}` at a display formula's frame, or at
an inline one whose `{` partner lies outside the formula (a box body, an argument's braces, an enclosing group), ends
the formula there (#553).

Witnesses: run 336's X1 (48 Fatal:Timeout:IfLimit with first error `unexpected:\right`, an xy cell scanner expanding
`\left\{`: 0705.0452 0802.0663 0804.1292 0809.0095 0903.4903 …, all 48 converting, errors 3541 → 826 over 109
witnesses); 2605.21750 (`\let\@left\left`); tests/parse/parens.tex:43 (`\left\delimiter"426830A`, byte-identical) and
its `\let\mydelim\delimiter` twin (before: the code digested as math); 1207.4709 (byte-identical); 2008.11762 (an
etoolbox `\robustify\left` now errs "\left not a macro" as in pdflatex), zhlineskip.tex:234 (`\env@cases`); 2605.02768 and 2606.01486 (physics' `\left(#1\right)` around an isolated argument,
66g). Guards `perfect_kernel_batch65d::*`, `perfect_kernel_batch66g::*`, golden
`tests/parse/left_right_delimiters`, `perfect_kernel_batch56::{let_at_left_left_does_not_loop,
left_right_in_text_mode_are_plain_text}`.

Since 66h the `\right.` an input's end inserts into a `\left` group it opened is inserted once per group: a `\left` read
in text (a document's `\newcommand\at[2]{\left.#1\right|_{#2}}` met outside math) opens a group the inserted `\right.`
cannot close, which then met every later end of input and inserted it again without end (1404.4148 and 8 more svjour3
papers: Fatal `Stomach:Recursion` after 25,640 Warnings). stomach.rs `close_left_group_at_input_end`, the frame value
`lx@left@right@inserted`; repro `sectioning-frontmatter/svjour3_doubled_at_left_in_text_closes_once` (an `\at` the svjour3
sentinel strip of KPE #652 leaves), guard `perfect_kernel_batch66h::svjour3_doubled_at_left_in_text_closes_once`.
