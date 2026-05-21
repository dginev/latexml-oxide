# ASF tiebreaking — operating notes

> Companion to [`MATH_PARSER_AND_ASF.md`](MATH_PARSER_AND_ASF.md). That
> doc covers *how* the math parser maps onto Marpa's ASF traversal.
> This one captures the **durable lever-selection discipline** for
> handling ambiguity, the **pragma catalog** for reference, and the
> **type-aware pruning** direction we eventually want to grow into.
>
> Status (2026-05-20): ASF integration closed; test suite at
> **1328/0/0** (both HYBRID default and `LATEXML_MARPA_ASF_ONLY=1`).
> The `modified_term` grammar refinement landed 2026-05-19 in Phase
> 1+2 (commits `a16cce3ddc` + `994cbcfa1a`) — see
> [`MATH_PARSER_AND_ASF.md`](MATH_PARSER_AND_ASF.md) for the
> implementation summary. The historical per-failure-case catalogs
> are removed; full versions recoverable from git history if
> needed.

---

## The setup

The math parser's grammar is intentionally **wide**: it admits many
parse trees for an ambiguous formula. Three layers prune:

1. **Stage 1 — grammar categories** (`factor`, `tight_term`,
   `statement`, …). Structural pruning at recognition.
2. **Stage 2 — per-tree actions** (`Actions::action_on`). Semantic
   pragmas inside each rule reduction; under HYBRID these run once
   per glade via the ASF callback in `asf_traverser.rs`.
3. **Stage 3 — `soft_prune_choices`** over the surviving forest.
   Final cross-tree filter.

After all three stages, if more than one tree survives, the consumer
picks `parses.remove(0)` — i.e., the first surviving tree wins.
**Order matters as the implicit tiebreaker.**

Tree iteration relies on libmarpa's natural depth-first traversal
for that order. ASF traversal uses our Cartesian-product expansion
at each glade. These orders are not the same — which is what makes
disambiguation work non-trivial.

### The two-pattern paradox

Two empirical cases prove that **enumeration order alone is not a
viable tiebreaker**:

| Case | Lexemes | Surviving parses | LEGACY picks | ASF picks |
|---|---|---|---|---|
| `g(x)` | `UNKNOWN:g, OPEN:(, UNKNOWN:x, CLOSE:)` | `g@(x)` (function-app) vs `g*x` (implicit-times) | `g@(x)` | `g*x` |
| `\int xy\,dx` (in a chain) | `INTOP, UNKNOWN:x, UNKNOWN:y, XDIFFUNK:d, UNKNOWN:x` | DIFFOP-app vs `integral@(x*y*d*x)` | DIFFOP-app | implicit-times |

Reversing the Cartesian product at every glade **fixes Case A but
breaks Case B**. There is no single per-glade ordering that
satisfies both, so we don't rely on order — we rely on the lever
ladder below.

---

## Lever selection discipline

When a parse-disambiguation case lands on the desk, work bottom-up
through the ladder and stop at the first viable layer:

```
1. grammar refinement       — narrowest, hardest, most principled
2. early action prune       — moderate cost, per-rule scope
3. tree-level pragma        — cheapest, broadest, latest
```

### When to use **early action pruning** (inside `semantics.rs`)

Use when the rejection is **purely local** to the reduction —
detectable from the rule's RHS components alone, without looking
elsewhere in the parse forest.

Examples that landed in the ASF wire-in:
* `compose` left-associativity reject in `infix_apply` — peek at
  the RHS; if it's another `compose` Apply, reject right-nesting.
* `OPERATOR` wider-absorption in `apply_invisible_times` and
  `infix_apply_nary` — when LHS Apply has `role="OPERATOR"`, absorb
  the chain into its args rather than emitting `OP * rest`.
* `bare_conditional` reject inside `list_apply` — if a list item is
  a bare `conditional@(…)` without parens-fence, reject it.

**Why prefer this layer**: Per-rule actions fire **once per glade**
under ASF (memoized), versus per-tree in `soft_prune_choices`. Cheap,
local, and fails fast.

Rule of thumb: *"not a pragma — you can detect this during the
action. Pragmas happen late and are less efficient — we only need
them for rules that require more global analysis of an
expression."*

### When to use **tree-level pragmas** (`pragmatics.rs`)

Use when the rejection criterion requires looking at the **whole
tree** — root shape, multi-tree comparison, or forest-wide count.

Examples that landed:
* `prefer_named_interval_at_root` — checks the root's Apply meaning.
* `prefer_non_self_wrapping_root` — checks `Apply@(Apply@(...))`
  redundancy at the root.
* `prefer_combined_relop_over_multirelation_with_absent` — root is
  a multirelation AND has interior `absent`.

Retired 2026-05-19 (commit `994cbcfa1a`):
* ~~`prefer_zero_absent_when_available`~~ — multi-tree count comparison.
  Its conceptual target (`<x|y>` bra-ket → `inner-product`) is now
  covered by qm-specific pragmas + the angle-bracket grammar rules
  after the `modified_term` Phase 1 grammar landing. Function body
  removed from `semantics/tree.rs`; tests stayed 1328/0/0 on both
  HYBRID and ASF.

**Why this layer exists**: some signals are only legible at the
forest level (e.g. "this candidate has 0 absent, that one has 2").
Per-rule actions can't see siblings.

### When to use **grammar refinement** (most principled)

Use when the ambiguity is **structural** — when the grammar admits
parses that mathematicians would never consider, regardless of
context.

**Why this layer is last**: most expensive (changes the grammar
and every action that produces XM for the affected rule); hard to
roll back; risk of regressions on the well-behaved cases.

Recent witness for this layer (open): `\Pi^N(p,q,…)` — capital
Greek letter followed by superscripts followed by parenthesised
comma-list. UNKNOWN-as-function vs implicit-multiplication is a
structural ambiguity whose right home is a grammar refinement
(or a lexer-level recognition pattern for "capital-Greek-letter
as function in math context"), not a per-action filter.

### Levers we considered but did not pick

* **Marpa `rule_rank`**: rule-rank ordering is hard to debug from
  XM output alone (the rank choice happens inside libmarpa, far
  from where we observe its effect).
* **Per-rule score return type**: invasive trait change; deferred
  until clear evidence that early-action prunes are insufficient.
* **Match Marpa tree-iter order in ASF**: proven not to be a
  uniform inversion (see the two-pattern paradox above).

---

## Pragma proposals catalog

For reference when similar patterns appear. Both P1 and P2 were
prototyped during the ASF wire-in; neither was kept as a default
because they over-prune on edge cases — but they remain useful
single-use levers when documented at the call site.

### P1 — "Prefer fewer `absent` tokens"

A parse that uses `absent` as a placeholder for a missing operand
is structurally weaker than a parse that doesn't need such
placeholders. When ranking surviving trees, prefer the tree with
**fewer** `absent` markers.

* Resolves cases like `0=<x,y>`: `0 = absent < list@(x, y) >
  absent` (2 absent) vs a no-absent reading (preferred).
* Does **not** resolve `\int xy\,dx`: both readings are
  absent-free.

**Implementation**: walk the XM tree, count `XM::Token` /
`XM::Lexeme` with content `"absent"`. Compare counts across
choices; prefer the lower count. Tiebreak by another lever.

**Scope**: applies to a *forest of survivors* — Stage 3. Costs
O(tree-size × surviving-count) per glade where the choice is made.

### P2 — "Smaller trees are usually better parses"

Among semantically equivalent parses, prefer the **shallower** or
**fewer-node** parse. Reasoning: a semantic operator
(`norm@(x)`, `differential-d@(x)`) compresses what would
otherwise be deeply nested or repeated structure.

* `||a||`: `absolute-value@(absolute-value@(a))` (depth 3) vs
  `norm@(a)` (depth 2) — prefer the latter.
* `\int x\,dx`: `integral@(x * differential-d@(x))` (3 atoms)
  vs `integral@(x * d * x)` (4 atoms) — prefer the former.

### Caveat — when P2 fights itself

P2 prefers smaller trees, but sometimes the **larger** tree is
semantically correct (e.g. `\sin x \cos y` should compose as
`sine(x) · cosine(y)` — explicit multiplication — not as a
fewer-token nonsense reading). P2 must be applied AFTER pragmatics
that establish semantic correctness, not as a sole filter.

### Combining P1 + P2

Reasonable ordering: pragmatics → P1 (drop absent-bearing if
others exist) → P2 (pick smallest survivor). Both are
**multi-tree comparisons** — they don't fit cleanly inside
`Actions::action_on` (per-rule), so they belong in
`pragmatics.rs` as `ValidationPragmatics` variants or in a new
post-pragmatics scoring step.

---

## Long-term direction: type-aware pruning

The pragmas in this doc are *shape-based* — they count node
patterns and pick by tree topology. That works as far as it goes,
but it misses the deepest mathematical signal: **operand types**.

When a mathematician reads `‖x‖ · a · ‖y‖` vs `‖x · ‖a‖ · y‖`,
they don't count nesting depth — they recognize that:

* `‖·‖` is a function `Vector → Scalar`.
* `·` (multiplication) usually requires compatible types.
* `‖x‖ · a · ‖y‖` is `Scalar · Vector · Scalar = Vector` — clean.
* `‖x · ‖a‖ · y‖` requires `Vector · Scalar · Vector` inside the
  norm — a mixed-type product whose result then gets normed.
  Unusual without explicit grouping.

The shape pragma `prefer_fewer_nested_same_fences` happens to
arrive at the right answer here, but the *reason* is type-driven.

### Why this matters

Many of our hardest disambiguation cases share the pattern:
"the grammar admits two readings; pick the one whose operator
arities, operand types, and result types form a coherent
algebraic expression". Examples encountered so far:

* `letter |x|` — K-12: `letter · |x|` (scalar/vector · scalar).
  QM bra-ket: `letter @ |x|` (operator applied to ket).
  Distinguished by whether the surrounding context establishes
  QM conventions.
* `<a|f|b>` vs `<a, f, b>` — bra-ket
  `quantum-operator-product` (scalar) vs relation chain
  (boolean). The presence of the `|` operator inside angle
  delimiters signals the QM type framework.
* `a|a|+b|b|+c|c|` — multiplication (`Vector · AbsScalar`)
  summed, vs nested-conditional (set-builder, which is a *set
  type*, not a scalar). The `+` separator forbids the set-type
  reading because sets don't sum.
* `(a, b)` — open-interval (`Interval[Scalar, Scalar]`) vs
  pair / tuple (`Tuple[T, T]`) vs cartesian point
  (`Point[Scalar, Scalar]`). The surrounding context (function
  application, set notation, geometry) determines the type.
* Function application vs implicit multiplication for `f(x)` —
  function-app requires `f : Domain → Range`; multiplication
  treats `f` as a scalar/element. Same pattern drives the
  `\Pi^N(p,q,…)` ambiguity explosion in 2310.16583.

### What "type-aware pruning" looks like

A future direction: assign each `XM::Apply` operator a **type
signature** (input arities + types, output type), and check
candidate parses for type coherence:

1. **Type tags on known operators**. E.g. `norm` is `Vector →
   Scalar`; `absolute-value` is `Real → NonNegReal` (or
   `Complex → Real`); `inner-product` is `(Vector, Vector) →
   Scalar`; `quantum-operator-product` is `(Bra, Operator, Ket)
   → Scalar`.
2. **Type-propagation rules** for compound operators: `times`,
   `plus`, etc. propagate operand types according to standard
   algebraic rules (scalar · vector → vector, etc.).
3. **Forest pragma**: for each candidate parse, attempt
   type-propagation from leaves to root. Candidates whose types
   fail to unify (or require improbable type coercions like
   "vector squared = scalar") are pruned. Among surviving
   candidates, prefer the one whose root type is simplest /
   most canonical for the surrounding context.
4. **Context-derived type frames**: a `delimited-⟨⟩` ancestor
   activates the QM type frame (bras/kets/operators); a
   `set-builder` ancestor activates set semantics; an
   `integral` ancestor activates measure-theoretic types.

This is a substantial undertaking — it pushes the math parser
from a pure syntactic disambiguator toward a *lightweight
semantic checker*. But many of our current pragmas (and known
remaining failures, including the `\Pi^N(p,q,…)` explosion)
are *proxies* for type checks. Replacing them with a real type
system would be more principled, more extensible, and would
naturally handle composite ambiguities that don't fit any
individual shape pragma.

### Practical sequencing

* **Short term**: continue with shape-based pragmas where they
  cleanly capture a single principle. The `count_*` / `prefer_*`
  family in `semantics/tree.rs` is reusable.
* **Medium term**: as more shape pragmas accumulate, factor out
  recurring sub-questions — e.g. "is this Apply a scalar?" —
  into a `Type::infer(&XM)` helper that returns a coarse type
  category. Convert individual shape pragmas to type-driven
  checks one at a time.
* **Long term**: a full type-propagation pass over the candidate
  tree that returns a type-coherence score. The forest pragma
  selects the candidate with the highest coherence score.
  Replaces most of the shape pragmas accumulated here, with
  one principled mechanism.

The arXiv-scale parsing goal in particular benefits from type
awareness: papers in different sub-disciplines (quantum
mechanics, statistics, category theory, differential geometry)
have very different type conventions, and a type-aware parser
can adapt to the surrounding context rather than relying on
shape heuristics that work for some domains but not others.
