# ASF tiebreaking — research notes

> Companion to [`MATH_PARSER_AND_ASF.md`](MATH_PARSER_AND_ASF.md). That
> doc explains *how* the math parser maps onto Marpa's ASF traversal.
> This one collected *open questions and design levers* during the
> ASF wire-in.
>
> **Status (2026-05-18, ASF integration closed):** all 29 → 0 ASF
> failures resolved via per-pattern pragmas + early-action
> interventions; current test suite is **1309/0**. The
> `modified_term` grammar refinement proposal below was NOT needed
> in the end — its motivating failures were all closed by pragma
> work. It remains documented as a future refactor opportunity
> (consolidate per-pattern pragmas under one principled grammar
> rule) but is not actively pursued.
>
> What's still load-bearing in this doc:
> * "Lever selection discipline" — durable rule for choosing the
>   right layer (grammar / action / pragma) for new ambiguity
>   classes if they surface.
> * The pragma proposals catalog — useful reference if similar
>   patterns appear.
> * "Long-term direction: type-aware pruning" — forward-looking
>   architectural sketch.
>
> Doc was compacted on 2026-05-18: the two "Historical: Phase 1
> catalog" sections (~240 lines of superseded per-failure tables)
> were removed; full version recoverable from git history.

---

## The setup

The math parser's grammar is intentionally **wide**: it admits
many parse trees for an ambiguous formula. Three layers prune:

1. **Stage 1 — grammar categories** (`factor`, `tight_term`,
   `statement`, …). Structural pruning.
2. **Stage 2 — per-tree actions** (`Actions::action_on`). Semantic
   pragmas inside each rule reduction.
3. **Stage 3 — `soft_prune_choices`** over the surviving forest.
   Final cross-tree filter.

After all three stages, if **more than one** tree survives, the
legacy `to_xmath` consumer picks `parses.remove(0)` — i.e., the
first surviving tree wins. **Order matters as the implicit
tiebreaker**.

Under the legacy `Tree`-iteration path, that order came from
libmarpa's natural depth-first / first-alternative-first traversal.
**Under ASF, the order comes from our Cartesian-product expansion at
each glade.** These orders are not the same, and the math parser
has no explicit ranking — so we inherit a tiebreaker mismatch.

---

## Two concrete cases we have measured

### Case A — `g(x)` (function-app vs multiplication)

* Input lexemes: `UNKNOWN:g, OPEN:(, UNKNOWN:x, CLOSE:)`.
* Two surviving parses: `g@(x)` (function-app) and `g*x` (implicit
  multiplication of three factors with parens-around-x).
* Legacy order: `g@(x)` first → wins.
* ASF Cartesian-product order: `g*x` first → wins.
* Effect: every `f(x)` formula across the test suite flips
  interpretation. ~9 of 20 ASF failures.

### Case B — `\int xy\,dx` (DIFFOP-app vs implicit-multiplication of d)

* Input lexemes: `INTOP:integral, UNKNOWN:x, UNKNOWN:y, XDIFFUNK:d,
  UNKNOWN:x`.
* Two surviving parses: `integral@(x*y*differential-d@(x))` (DIFFOP-app)
  and `integral@(x*y*d*x)` (treats `d` as a plain factor).
* Legacy order: DIFFOP-app first → wins.
* ASF Cartesian-product order: depends on glade; in `\int x\,dx`
  alone the DIFFOP wins, but in `xy+xy+\int xy\,dx+xy+xy` the
  multiplication wins. **Context-sensitive.**
* Effect: every `\int … dx` in a multi-term context flips. ~9 of
  20 ASF failures.

### Why Case A and Case B disagree on direction

Reversing the Cartesian product at every glade **fixes Case A but
breaks Case B**. The two ambiguities prefer opposite tree-iteration
positions. There is no single per-glade order that satisfies both
— this is the strongest evidence that **enumeration order is
the wrong tiebreaker**.

---

## Levers we have available

The math parser sits between Marpa and the XMath output. Each
boundary is a place we can intervene.

### Lever 1 — Marpa rule ranking

libmarpa supports per-rule rank in the `Order` step. The math
parser's `init_grammar` could call `grammar.rule_rank(rule_id,
rank)` to bias certain reductions ahead of others. Under the
legacy `Tree`-iterator this would change emission order directly;
under ASF the ranking is carried into the bocage's and-node
ordering, which our `collect_factorings` could then honor.

* **Pros**: principled, infrastructure-level fix; affects both
  paths uniformly.
* **Cons**: needs to be specified per ambiguous rule pair (DIFFOP-
  app > multiplication; function-app > multiplication; …) — N
  cases, each requiring intent capture.
* **Open**: is the math grammar small enough to enumerate all
  ambiguity pairs? Probably yes (~dozens), but tedious.

### Lever 2 — Ranking pragma in `Actions::action_on`

Each rule's action returns `Ok(Some(XM))` on success. We could
extend the return type to include a **score** (`Ok(Some((XM,
i32)))`), and let the driver use the score as an explicit
tiebreaker after pragmatics.

* **Pros**: domain knowledge lives in `Actions` next to the
  per-rule code; easy to read and update.
* **Cons**: invasive trait change; needs every action updated;
  legacy path can't easily use it because it iterates tree by tree.

### Lever 3 — Post-pragmatics scoring in `XM::Choices`

After Stage 3 reduces to `XM::Choices(N)`, run a deterministic
scorer over the N candidates and pick the highest. Examples:

* prefer trees with fewer `XM::Lexeme` leaves (more semantic
  interpretation);
* prefer trees where `differential-d`, `integral`, `sine` etc.
  are recognized as named operators rather than generic letters;
* prefer trees where parenthesized expressions appear as function
  arguments rather than as factors of multiplication.

* **Pros**: keeps the grammar simple, ranking is centralized in
  one Rust function, both legacy and ASF paths benefit.
* **Cons**: heuristic; needs comprehensive coverage to avoid
  surprises; risk of being too clever.

### Lever 4 — Grammar restructuring to remove the ambiguity

Some Case-A/B style ambiguities exist because the grammar admits
both interpretations. Restricting `factor → diffunk` so `diffunk`
is **only** valid in DIFFOP-application contexts would eliminate
Case B. Similarly `factor_base → unknown` could exclude letters
adjacent to OPEN parens.

* **Pros**: makes the grammar say what it means; speedup as a
  bonus.
* **Cons**: violates the "wide grammar + semantic pruning" design
  philosophy; possible parse regressions on corner cases that
  legitimately need the broader interpretation.

### Lever 5 — Match Marpa's tree-iteration order in ASF

Make the ASF Cartesian product visit alternatives in exactly the
same order as libmarpa's `Tree` iterator. Cosmetic if it works,
since it preserves the implicit legacy tiebreaker.

* **Pros**: drop-in compatibility, no semantic decisions.
* **Cons**: hard to characterize Marpa's natural order at every
  glade; reverse-engineering libmarpa internals; brittle.
* **Empirical**: a top-level `reverse()` of the peak's
  alternatives matches Case A but not Case B; a per-glade
  `reverse()` is the opposite. **There is no single fixed
  reversal that matches both** — Marpa's natural order is not
  simply "first" or "last" of our Cartesian product.

---

## Lever selection discipline (durable rule, 2026-05-17)

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

Examples (all landed 2026-05-17):
* `compose` left-associativity reject in `infix_apply` — peek at
  the RHS; if it's another `compose` Apply, reject right-nesting.
* `OPERATOR` wider-absorption in `apply_invisible_times` and
  `infix_apply_nary` — when LHS Apply has `role="OPERATOR"`, absorb
  the chain into its args rather than emitting `OP * rest`.
* `bare_conditional` reject inside `list_apply` — if a list item is
  a bare `conditional@(…)` without parens-fence, reject it.

**Why prefer this layer**: Per-rule actions fire **once per glade**
in ASF (memoized), versus per-tree in `soft_prune_choices`. Cheap,
local, and fails fast.

User feedback verbatim (2026-05-17): *"not a pragma — you can detect
this during the action (early semantic pruning). Pragmas happen
late and are less efficient — we only need them for rules that
require more global analysis of an expression."*

### When to use **tree-level pragmas** (`pragmatics.rs`)

Use when the rejection criterion requires looking at the **whole
tree** (root shape, multi-tree comparison, or forest-wide count).

Examples (landed 2026-05-17):
* `prefer_named_interval_at_root` — checks the root's Apply meaning.
* `prefer_non_self_wrapping_root` — checks `Apply@(Apply@(...))`
  redundancy at the root.
* `prefer_combined_relop_over_multirelation_with_absent` — root is
  a multirelation AND has interior `absent`.
* `prefer_zero_absent_when_available` — multi-tree count comparison.

**Why this layer exists**: some signals are only legible at the
forest level (e.g. "this candidate has 0 absent, that one has 2").
Per-rule actions can't see siblings.

### When to use **grammar refinement** (most principled)

Use when the ambiguity is **structural** — when the grammar admits
parses that mathematicians would never consider, regardless of
context. The `modified_term` proposal (below) is the canonical
example: `tight_term ≡ EXPR ≥ BOUND` should ALWAYS parse as
"define-then-bound", never as a flat multi-relation chain that
needs `absent` markers.

**Why this layer is last**: most expensive (changes the grammar
and every action that produces XM for the affected rule); hard to
roll back; risk of regressions on the well-behaved cases.

### Levers we DIDN'T pick this session, and why

* **Lever 1 — Marpa `rule_rank`**: explored conceptually; deferred
  in favor of action/pragma layers because rule-rank ordering is
  hard to debug from XM output alone (the rank choice happens
  inside libmarpa, far from where we observe its effect).
* **Lever 2 — Per-rule score return type**: invasive trait change;
  deferred until clear evidence that early-action prunes are
  insufficient.
* **Lever 5 — Match Marpa tree-iter order in ASF**: proven not to
  be a uniform inversion. Two cases (function-app vs implicit-times,
  and DIFFOP vs letter-d) prefer **opposite** Cartesian-product
  orders. There is no single reversal that satisfies both.

---

## What landed in the 2026-05-17 push (1272/29 → 1292/9)

Eight interventions, in chronological landing order. Each cites
the file + function where the change lives.

| # | Layer | Mechanism | File / function | Tests gained |
|---|---|---|---|---|
| 1 | Pragma (student tier) | Dual-aware `FencedLettersAreFunctionArguments` recognising both `XM::Token` and `XM::Lexeme` OPEN/CLOSE inside a Dual's presentation | `pragmatics.rs::is_dual_fenced_rhs` | 12 |
| 2 | Pragma (forest) | `prefer_named_interval_at_root` — if root is `open-interval@(a,b)` in one candidate and `vector@(a,b)` in another, drop the vector | `semantics/tree.rs::prefer_named_interval_at_root` | 2 |
| 3 | Pragma (forest) | `prefer_non_self_wrapping_root` — drop `set@(set@(…))` when `set@(…)` exists | `semantics/tree.rs::prefer_non_self_wrapping_root` | 2 |
| 4 | Pragma (forest) | `prefer_combined_relop_over_multirelation_with_absent` — `x >= 0` over `x > absent = 0` when the multirelation has *interior* absent | `semantics/tree.rs::root_is_multirelation_with_interior_absent` | 1 |
| 5 | Early action | OPERATOR wider-absorption in `apply_invisible_times` and `infix_apply_nary` — `D x*y*z` → `D@(x*y*z)` | `semantics.rs::apply_invisible_times`, `infix_apply_nary` | 1 |
| 6 | Early action | Compose left-associativity prune | `semantics.rs::infix_apply` | 1 |
| 7 | Early action | Bare-conditional reject in list items, with parens-fence carve-out | `semantics.rs::list_apply` | 1 |
| 8 | Pragma (forest) | `prefer_zero_absent_when_available` — multi-tree count comparison, with `count_nodes_for_parse_ranking` following `XM::Ref` through `build_ref_index` | `semantics/tree.rs::prefer_zero_absent_when_available` | 1 (ncases, **blessed** — accepted ASF reading as new ground truth) |

### Key invariants captured along the way

* `count_nodes_for_parse_ranking`: Apply = `1 + sum(args)` (operator
  is intrinsic to the Apply, not a separate node). Dual = content
  count only (presentation is decoration). Ref = follow through
  idref into the resolved node. Established via user direct
  feedback after I double-counted Apply on first attempt.
* Forest pragma compares the **roots** of each candidate; for
  per-glade decisions use the action layer.
* Expert pragmas (e.g. `FencedLettersAreFunctionArguments` before
  it was demoted) only fire via `Lexeme::specialize` callsites.
  Actions don't call `.specialize()` on Apply nodes, so an expert
  pragma on a Dual-shaped Apply is a **no-op**. The fix was to
  move the pragma to the student tier (which fires via
  `validate_recursive` inside `soft_prune_choices`).
* The ncases test bless was the only "ASF ground-truth wins"
  decision this push. The original Perl XML and legacy ASF
  produced `cases @ (((w...|...)... ≥ |d|))` — a conditional
  inside a conditional. The new ASF reading is
  `w ≡ √|c| · √(…) · |c| ≥ |d|` (define-then-bound chain), which
  is the obvious mathematical reading. User sign-off recorded.

---

## Triage of the 20 failing tests (2026-05-17)

| Test | Class | Notes |
|---|---|---|
| calculus, count_parses, esint, mathtools, ntheorem, operators, sampler, spacing, unit_tests_by_silviu | Case B (DIFFOP vs multiplication-of-d) | 9 of 20. Same root cause. |
| function_argument_syntax | Case A (function-app vs multiplication) | 1 of 20. |
| ambiguous_relations, relations, qm | `<`/`>` as RELOP vs delimiter | 3 of 20. Possibly the same root pattern as Case A but at the delimiter level. |
| scripts, simplemath, artefacts | Script attachment ordering | 3 of 20. Different from A/B — about which side of a script position binds. |
| vertbars | `||a||` — abs-of-abs vs nested | 1 of 20. May be a structural-improvement candidate. |
| physics | `\sin[…]` with mixed delimiters | 1 of 20. Subtle, needs case-by-case look. |
| stmaryrd, wasysym | List flattening | 2 of 20. **ASF produces flatter lists; legacy has weird nesting.** May be structural improvements. Need user sign-off. |

---

## Decisions we need (open)

1. **Tiebreaker source**: pick one of Levers 1, 2, 3 (or a hybrid)
   as the primary mechanism. Each has different costs and
   reversibility.

2. **Per-case authority**: who decides "DIFFOP > multiplication"
   in `\int xy\,dx`? Is that universal or domain-dependent?

3. **Improvement gating**: when ASF reveals a cleaner parse than
   legacy (e.g. stmaryrd's flat list), do we adopt ASF's output as
   the new ground truth? Per the user's 2026-05-17 instruction:
   yes, with sign-off per case.

4. **Cap removal sequencing**: once parity is restored, when do we
   delete the 5 convergence caps in `parse_marpa`? They're
   harmless under ASF (each glade fires once) but dead-code-removal
   tightens the next contributor's mental model.

5. **Grammar simplification**: separable from this work, but if
   Lever 4 lands for some pairs, it may reveal that the grammar's
   ambiguity is broader than necessary.

---

## Open experiments to try (no commitment yet)

* **E1**: stat-dump the 20 failing tests' parse-count and per-pragma
  rejection histograms (PARSE_AUDIT + PARSE_PRUNE_REASONS) to see
  if there's a structural pattern beyond the case labels above.
* **E2**: implement Lever 3 (post-pragmatics scoring) for the
  two known cases (DIFFOP-app preferred, function-app preferred)
  and see how many of the 20 it fixes vs how many it breaks.
* **E3**: try Lever 1 (Marpa rank) on a single rule pair as a
  feasibility check.
* **E4**: study the bocage shape for `g(x)` and `\int xy\,dx` to
  understand WHY their natural orders disagree — is it about
  rule definition order in `init_grammar`, or about Aycock-
  Horspool internal-rule promotion?

---

## Pragma proposals (user-contributed, 2026-05-17)

### P1 — "Prefer fewer `absent` tokens"

A parse that uses `absent` as a placeholder for a missing operand
is structurally weaker than a parse that doesn't need such
placeholders. When ranking surviving trees, prefer the tree with
**fewer** `absent` markers.

**Worked example** — `\int xy\,dx`:
* `integral@(x * y * differential-d@(x))` — 0 `absent` tokens
* `integral@(x * y * d * x)` — 0 `absent` tokens
* Both tied; this pragma alone doesn't resolve.

**Worked example** — `0=<x,y>`:
* `0 = absent < list@(x, y) > absent` — 2 `absent` tokens
  (uses `<` and `>` as RELOP with missing operands)
* `0 = bra-ket-or-similar(...)` — 0 `absent`
* This pragma resolves correctly toward the no-`absent` parse.

**Implementation**: walk the XM tree, count `XM::Token` /
`XM::Lexeme` with content `"absent"`. Compare counts across
choices; prefer the lower count. Tiebreak via Lever-X.

**Scope**: applies to a *forest of survivors* — Stage 3. Costs
O(tree-size × surviving-count) per glade where the choice is made.

### P2 — "Smaller trees are usually better parses"

Among semantically equivalent parses, prefer the **shallower**
or **fewer-node** parse. Reasoning: a semantic operator
(`norm@(x)`, `differential-d@(x)`) compresses what would otherwise
be deeply nested or repeated structure.

**Worked example** — `||a||`:
* `absolute-value@(absolute-value@(a))` — depth 3, 3 atoms
* `norm@(a)` — depth 2, 2 atoms — **prefer this**.

**Worked example** — `\int x\,dx`:
* `integral@(x * differential-d@(x))` — depth 3, 3 atoms
* `integral@(x * d * x)` — depth 3, 4 atoms — **prefer the first**.

**Implementation**: walk XM, count nodes (or compute max depth).
Prefer the lower count. Same scope and cost as P1.

### Caveat — when P2 fights itself

P2 prefers smaller trees, but sometimes the **larger** tree is
semantically correct (e.g. `\sin x \cos y` should compose as
`(sine(x))(cosine(y))` — explicit multiplication — not as
`sin xc os y` which would be fewer tokens but nonsensical). P2
needs to be applied AFTER pragmatics that establish semantic
correctness, not as a sole filter.

### Combining P1 + P2

Reasonable ordering: pragmatics → P1 (drop absent-bearing if
others exist) → P2 (pick smallest survivor). Both are
**multi-tree comparisons** — they don't fit cleanly inside
`Actions::action_on` (per-rule), so they belong in
`pragmatics.rs` as new `ValidationPragmatics` variants or in a
new post-pragmatics scoring step (Lever 3).

---

## Empirical results from prototyping P1 and P2 (2026-05-17)

### P1 — `prefer_fewer_absent`

Tested standalone (without P2). **Result**: 1300 / 1 / 0 (vs 1301
baseline). Single regression on `latextheorem_test` — a boundary
case where both surviving trees had the same `absent` count and
the legacy already picked the right one; P1 was a no-op on it.

**Verdict**: safe to keep. `prefer_fewer_absent` is now wired into
`parse_marpa` after `student_pragmatics`.

### P2 — `prefer_smaller_tree`

Tested with TWO counting conventions:

**(a) naïve `count_nodes` (Apply = 1 + op + args):** regressed
13 legacy tests because the count over-weighted `Apply` nodes
(double-counting the operator).

**(b) corrected `count_nodes_for_parse_ranking` (Apply = 1 + sum
of args; Dual = content only; Ref = follow into presentation):**
9 legacy regressions, of which **3 are genuine
improvements** revealed by ASF:

| Test | Improvement |
|---|---|
| `count_parses_test` | `quantum-operator-product@(B, sum_k f_k, C)` instead of legacy's `delimited-⟨⟩@(B@(absolute-value@(sum_k f_k)) * C)` — the QM bracket notation is correctly recognized. |
| `mathtools_test` | Same QM bracket improvement. |
| `stmaryrd_test` | Flat 7-list `list@(a varominus b, …, a vartimes b)` instead of legacy's weird nested `list@(…, a varoslash list@(b, a multiplicative-conjunction b), …)`. |

And **6 regressions** that the legacy got right:

| Test | Regression |
|---|---|
| `function_argument_syntax_test` | `cosine@(2)` instead of `cosine@(2 * pi * y)` — drops `FunctionsPreferWiderAbsorption` |
| `standalone_modifiers_test` | `x@(absent > 0)` instead of `annotated@(x, absent > 0)` |
| `physics_test` | `sine@(X/Y)` instead of `sine@(delimited-[]@(X/Y))` |
| `ambiguous_relations_test` | `formulae@(2 < x, y >= z)` instead of `2 * delimited-<>@(list@(x, y)) = z` |
| `qm_test` | flips the bracket interpretation |
| `ncases_test` | drops a `conditional@` wrapper |

**Net: 3 improvements vs 6 regressions in legacy. Reverted.**

### What the failures tell us about the grammar

The 3 improvements all share a structural pattern: the **legacy
admitted two interpretations**, one shallow (the wrong one) and
one semantically deeper (the right one). Tree-iteration order
happened to pick the shallow one. Tree-size pragma picked the
deeper. So the right call for these is to PROMOTE the semantic
recognizer at the grammar / action level rather than rely on
post-hoc shape ranking.

The 6 regressions reveal that tree-size is a misleading proxy
when ground truth wraps semantics in explicit `delimited-X@(…)` or
`annotated@(…)` markers that LATER post-processing depends on.
Stripping the wrappers by size penalty corrupts downstream code.

---

## Updated decision matrix

| Lever | Cost | Coverage | Reversible | Verdict |
|---|---|---|---|---|
| 1 — Marpa rule_rank | High (per-pair grammar edits) | High (affects both paths) | Easy | **Pursue selectively** for the well-known cases (function-app > implicit-times, DIFFOP > letter-d, QM-bracket > abs-bracket). |
| 2 — Action-level scoring | Medium (trait change) | Per-rule | Easy | Defer — wait until the grammar is more stable. |
| 3a — `prefer_fewer_absent` | Low | Multi-tree | Easy | **Adopted.** Safe net positive. |
| 3b — `prefer_smaller_tree` (universal) | Low | Multi-tree | Easy | **Rejected as universal.** Too coarse. |
| 3c — Targeted post-pragmatics scorers (e.g. QM-bracket recognizer) | Low | Per-pattern | Easy | **Pursue** for each "shallow vs deep" pair we want to fix. |
| 4 — Grammar restructuring | Very high | Targeted | Hard | Defer to a separate research track. |
| 5 — Match Marpa tree-iter order | Hard / brittle | All cases | Easy | Don't pursue — proven not to be a uniform inversion. |

---

## First-principles framing — why tree-size alone fails

The math parser's grammar is **wide by design**: it admits many
parse trees per formula to maximize recall on arXiv-scale weird
math. Among multiple valid parses, the goal is to recover the
parse that best matches **standard mathematical conventions**:

* recognize named operators (`norm`, `differential-d`, `integral`,
  `quantum-operator-product`),
* prefer explicit semantic wrappers (`delimited-[]`, `annotated`,
  `conditional`) where they were authored,
* avoid filler `absent` tokens when avoidable.

**Tree-size is a proxy** for "preferred parse". It works when the
semantic wrapper is the SMALLER tree (e.g. `norm@(a)` 2-node beats
`abs(abs(a))` 3-node), and fails when the semantic wrapper is the
LARGER tree (e.g. `annotated@(x, expr)` beats `x@(expr)`).

Tree-size therefore correlates with semantic richness in only
half the cases. The other half need a different signal.

**What's the better signal?**

* **Named-operator recognition**: count `XM::Apply` with
  `meaning="quantum-operator-product"` / `"integral"` / `"norm"` /
  etc. as +1; count generic times as 0. Trees with more named
  operators win.
* **Domain-typed wrappers**: count `delimited-X`, `annotated`,
  `conditional` as +1 — these are explicit semantic markers
  authored by the latex source.
* **Coverage**: count covered tokens — a parse that produces
  `Apply` over 4 children covers all 4 tokens; one that drops a
  token to `absent` covers fewer.

These three give a more **semantic** ranking than raw size. Each
is computable as a tree-walk pragma, and they could be combined
linearly (or by ordered preference).

**Lever-3c proposal**: implement these as a new family of
`pragmatics::ranking` functions, applied AFTER `soft_prune_choices`
and `prefer_fewer_absent`. Three rounds:

1. `prefer_more_named_operators` — drop trees with the fewest
   named operators; keep the richer ones.
2. `prefer_more_domain_wrappers` — drop trees missing
   `delimited-X` / `annotated` / `conditional` when others have
   them.
3. `prefer_fewer_lexeme_leaves` — drop trees that have raw
   `XM::Lexeme` leaves where alternatives have `XM::Token` with
   `meaning` set (i.e. the lexeme was specialized into a known
   role).

Stop once a single tree survives, or yield `XM::Choices(N)` as
last resort.

This decomposes the "smaller is better" intuition into three
SEMANTIC checks, each of which can be reasoned about individually.

---

## Current 9 failures — canonical catalog (end of 2026-05-17 push)

**ASF parity: 1292/9. LEGACY: 1301/0.**

Each row links to (a) the dominant ambiguity class from earlier
sections, (b) the next lever we'd try, and (c) whether it falls
under the proposed `modified_term` umbrella (✅) or needs its own
intervention (─).

| # | Test | Pattern (compressed) | Class | Next lever | `modified_term`? |
|---|---|---|---|---|---|
| 1 | `ambiguous_relations_test` | `(<x,y>, …)` family | C / B | Either modified_term, or extending P4 to `formulae@(2 < x, y >= z)`-shape | ✅ |
| 2 | `count_parses_test` | `\langle B\|\sum f\|C\rangle` (function-app inside angle-delim Dual) | H | Either modified_term, or extending the Dual-aware pragma to recognise this shape | ✅ |
| 3 | `mathtools_test` | Same as #2 in a slightly different fixture | H | Same as #2 | ✅ |
| 4 | `metarelation_elision_test` | Multi-relation with elision marker | M | Probable modified_term beneficiary | ✅ (probably) |
| 5 | `physics_test` | ASF produces `<Math class="ltx_math_unparsed">` for some sub-formula | U | Separate — grammar coverage issue, NOT tiebreaking | ─ |
| 6 | `plainfonts_test` | TBD — needs case-by-case diff | ? | Catalog first | ? |
| 7 | `qm_test` | `<a\|f\|b>` bra-ket inside angle-delim | C | Either modified_term, or QM-bracket-specific pragma | ✅ |
| 8 | `standalone_modifiers_test` | `(<0)` — modifier-only term | K | **Direct fit for modified_term** | ✅ |
| 9 | `vertbars_test` | `\|\|x\|\|a\|\|y\|\|` — ambiguous bar pairing | F | Pragma + likely also helped by modified_term | ✅ (partly) |

### Read of the table

* **5-6 of 9** failures align with the `modified_term` grammar
  refinement. If that refinement is implemented and verifies as
  expected, ASF parity would jump to **1297-1298 / 3-4**.
* **`physics_test`** is the one true outlier — it's not a
  tiebreaking issue at all but a parse-coverage gap (ASF produces
  no parse). Different investigation track.
* **`plainfonts_test`** needs first-pass diff inspection before
  classifying.

### Next-session entry point (priority order)

1. **`modified_term` grammar refinement** — single principled change
   that should close 5-6 failures simultaneously. See proposal
   section.
2. **`physics_test`** — diagnose the parse-coverage gap. Likely a
   missing rule or a specific lexeme handling.
3. **`plainfonts_test`** — catalogue, then per-pattern intervention
   if not subsumed by modified_term.
4. **`vertbars_test`** — pattern-specific pragma if modified_term
   doesn't suffice.

---

## Grammar evolution — `modified_term` proposal (2026-05-17)

The pragma path closes individual ambiguity-class failures
one-at-a-time. A more principled refinement is to **evolve the
grammar categories** to express how mathematicians actually parse
expressions: definitions and constraints attached to a term.

### Proposed category

```
modifier      = relop expression
modified_term = tight_term modifier+        // 1+ modifiers attached
statement     = modified_term | formula | ...
```

A `modified_term` is:
* a base `tight_term` followed by one or more `modifier`s,
* the modifiers chain only with each other (no other operations
  intervene),
* the result lifts to top-level / `statement`.

### Why this matches the math reading

For `w ≡ √|c| · √((1+√(1+(d/c)²))/2) · |c| ≥ |d|` (the ncases
case):

* `tight_term = w`
* `modifier 1 = ≡ √|c| · √(…) · |c|` (definition)
* `modifier 2 = ≥ |d|` (constraint clarifying the definition)
* → `modified_term(w, [mod1, mod2])`

This is the only valid parse under the refinement. The vertical
bars inside `mod1` are unambiguously absolute-value because they
appear inside a `tight_term` context, NOT at a level where the
conditional-separator rule competes for them.

### Generalization

The pattern `IDENT ≡ EXPR (≥ | ≤ | < | > | =) BOUND` is one of the
most common idioms in mathematical writing — every Bourbaki-style
"define X as Y, which is bounded by Z" follows this shape. The
current grammar handles this via the loose `formula relop
expression` rule that admits any number of relops to interleave,
producing many parse trees per chain. `modified_term` constrains
the chaining to legitimate modifier sequences.

### Expected coverage

This refinement would directly resolve the following classes
(currently held together by pragma):

* **ncases-type definition-constraint chains** — handled today
  by `prefer_zero_absent_when_available`.
* **`<x, y> = 0` shape** (ambiguous_relations) — the `<x,y>`
  becomes an unambiguous tight_term and `= 0` is a modifier.
* **`<a|f|b>` QM bra-ket inside angle-delim** — the bra-ket
  becomes a tight_term; modifiers around it parse cleanly.
* **metarelation_elision** — likely (need to verify the failure
  shape).

That's ~5-6 of the remaining 9 ASF failures, addressed at the
grammar level rather than per-pattern pragmas.

### Per-failure coverage hypothesis

Walk through the 9 remaining ASF failures and predict which the
refinement subsumes:

| Test | Current ambiguity | Why modified_term resolves |
|---|---|---|
| `standalone_modifiers_test` | `(<0)` — bare `<0` parsed as `absent < 0` versus as a parenthesized modifier expression | `modified_term` legitimizes `<0` as a free-standing modifier inside parens — no `absent` introduced. **Direct fit.** |
| `ambiguous_relations_test` | `<x, y> = 0` — vector inside angle-bracket vs `absent < x, y > absent = 0` chain | `<x,y>` becomes an unambiguous `tight_term` (vector or delimited-pair). `= 0` attaches as a single `modifier`. No `absent` markers needed. |
| `qm_test` | `<a|f|b>` — bra-ket interpretation | The Dirac bracket `<…|…|…>` is recognised as a `tight_term` at the lexical level; surrounding context attaches `modifier`s rather than competing for the bars. |
| `count_parses_test`, `mathtools_test` | `\langle B|\sum f|C\rangle` — function-app `B(…)` not happening because Dual-aware pragma doesn't see it | When the angle-delim is at the `tight_term` level, the function-app pragma already fires correctly. The current failure is the Dual structure prevents the pragma from reaching inside. Modified_term moves the angle-delim to a category where the Dual is built differently. |
| `metarelation_elision_test` | Multi-relation with elision marker | The elision marker fits naturally as a non-relop modifier between relop modifiers. Need to verify. |
| `vertbars_test` | `\|\|x\|\|a\|\|y\|\|` | The bars don't directly map onto modifier syntax, but `modified_term` confines the `tight_term`'s internal ambiguity — bars inside tight_term unambiguously absolute-value. May not fully resolve, but reduces forest size. |
| `physics_test` | ASF produces no parse | NOT subsumed — separate parse-coverage gap. |
| `plainfonts_test` | TBD | TBD — catalog first. |

**Estimated subsumption: 5-6 of 9 failures.** Definitely:
standalone_modifiers, ambiguous_relations, qm. Likely:
count_parses, mathtools, metarelation_elision. Partial:
vertbars. Not: physics, plainfonts (TBD).

### Concrete BNF placement

In `latexml_math_parser/src/grammar/builder.rs` the current
relevant rules are roughly (see file for exact form):

```
statement      → formulae | formula
formulae       → formula (",", formula)+
formula        → expression (relop expression)+
              |  expression
expression     → tight_term (addop tight_term)*
tight_term     → factor (mulop factor)*
factor         → factor_base ("^" | "_" factor_base)*
factor_base    → number | letter | OPEN expression CLOSE | …
```

The refinement adds:

```
modifier       → relop expression                                      // NEW
modified_term  → tight_term modifier+                                  // NEW
statement      → formulae | formula | modified_term                   // ADD modified_term arm
```

Note the **ordering** matters: `modified_term` must be visible at
the `statement` level (top of the grammar), so a single tight_term
+ modifier chain doesn't fall through to `formula` and pick up the
flat multi-relation interpretation. With Marpa's left-to-right
SLIF semantics, both `formula` and `modified_term` may match for
chains of relops; the action layer should disambiguate by checking
whether the result is a "lifting" modifier sequence vs a true
relation chain.

### Edge cases and risks

* **`a < b < c` chain** — under the new grammar this could parse
  as `modified_term(a, [< b, < c])` OR as `formula(a < b < c)`. The
  flat form is what we want for true relation chains. The
  refinement needs an action-layer disambiguator: if all modifiers
  share the same relop AND there's no `=` or `≡`, prefer the
  formula reading.
* **`a = b = c = d`** — equality chains are usually relation chains
  ("all four are equal"), not "define a, then constrain b, then
  constrain c". Same disambiguator: chain of identical relops →
  formula; mixed (`≡` + `≥`) → modified_term.
* **Already-pragma'd cases** — `prefer_zero_absent_when_available`
  currently rescues ncases. If modified_term subsumes ncases the
  pragma becomes dead code; either delete it or keep it as a
  belt-and-suspenders for other patterns.

### Implementation cost

Medium-high. Requires:
1. Adding the `modifier`, `modified_term` rules and a
   `statement → modified_term` arm to
   `latexml_math_parser/src/grammar/builder.rs`.
2. Writing actions (`apply_modified_term`, `chain_modifier`) in
   `semantics.rs` to construct the right XM shape — likely
   `XM::Apply` with `meaning="modified-term"` or similar.
3. **Disambiguator at the action layer** to distinguish "true
   relation chain" from "definition+modifier" — see edge cases.
4. Validate that `prefer_zero_absent_when_available` and the other
   landed pragmas don't fight the new grammar — they likely become
   no-ops on modified_term roots but should still help in other
   contexts.
5. Test that all 1301 LEGACY tests still pass (the new rules
   should be ADDITIVE; demoting an existing rule risks regressions).

### Validation plan

1. Implement grammar + actions; run full `cargo test --tests` to
   measure both LEGACY and ASF impact.
2. If LEGACY regresses, the new rules are competing with existing
   ones. Either restrict the modified_term firing condition or
   add an action-layer disambiguator.
3. Once both paths green or improved, run 10k canvas to validate
   on real arXiv math.
4. Delete pragmas that have become dead code (verify by removing
   one at a time and checking neither path regresses).

### Sequencing

* **Short term (this session)**: pragma path — already at 1292/9
  with the zero-absent pragma. Continue with targeted pragmas for
  the bra-ket and `<x,y>` cases.
* **Medium term (separate session)**: implement `modified_term`
  refinement. Verify it subsumes the pragmas it supersedes; keep
  any pragmas that handle orthogonal cases.

---

## Arxiv-scale implications

The current state — **98.5% parity on a small test suite** — is a
solid foundation. The remaining 1.5% is concentrated in 20 well-
classified ambiguity classes, none of them about parse correctness
per se; they're about TIEBREAKING when the grammar admits multiple
valid interpretations.

For arXiv-scale (1B formulas) we need:

1. **Robustness on the long tail**: pragma misfires must not cause
   regressions on the well-behaved 99%+ of formulas. P1
   (`prefer_fewer_absent`) is safe in this sense; P2
   (`prefer_smaller_tree`) is not.

2. **Composability of pragmas**: each pragma should have a clear
   "I drop trees that lack X" semantic, never "I prefer the
   smaller". That way the union of pragmas is principled.

3. **Coverage-of-corpus rather than coverage-of-tests**: the test
   suite is small. The 10k canvas stage (the next gate after this
   work) will reveal pragma misfires we can't see here.

The ASF infrastructure is the load-bearing piece: it makes
exhaustive enumeration practically feasible (98.5% parity at 4×
speedup on ambiguous formulas, see `MATH_PARSER_AND_ASF.md`).
The pragmatic tiebreaking is the long-tail polish on top of it.

---

## What we have committed so far

### Marpa fork (`~/git/marpa`)
* Branch `asf-step3-generic-traverser` — Steps 2-6 of
  `ASF_STATUS.md` landed: `compute_symches` factoring loop,
  Glade query API, recursive memoized `ASF::traverse`, generic
  `&mut TR Traverser` trait, substantive 3-parse panda test. 17
  marpa tests pass.
* `ASF::peak` multi-source fix (real correctness bug discovered
  during wire-in).

### latexml-oxide (this branch)
* `MathTraverser` wired into `parse_marpa` behind
  `LATEXML_MARPA_ASF=1`.
* Discriminator handling 5 glade classes (byte / outer-token /
  passthrough-rule / scaffolding / action-rule).
* **8 pragma + early-action interventions** (see "What landed in
  the 2026-05-17 push" section above) — 1272/29 → **1292/9**.
* `count_nodes_for_parse_ranking` with Ref-following via
  `build_ref_index` in `semantics/tree.rs`.
* Per-glade `reverse()` of the alternatives vec — **removed** this
  push (proved not to be a uniform inversion; replaced with the
  pragma/action ladder).
* `ncases_test` XML reblessed — accepted ASF reading as new
  ground truth with user sign-off (define-then-bound chain is
  the obvious mathematical reading).

### Numbers
* **LEGACY**: 1301/0
* **ASF**: 1292/9
* Parity gap: 9 tests (0.69%).

### What's NOT yet committed
* `modified_term` grammar refinement (next-session priority).
* Marpa PR merge to dginev/marpa master + dep switch back to
  master branch.
* 10k canvas validation under ASF.
* Removal of 5 convergence caps in `parse_marpa` (only safe
  once ASF parity is 100%).

---

## Long-term direction: type-aware pruning

The pragmas in this doc are *shape-based* — they count node
patterns and pick by tree topology. That works as far as it goes,
but it misses the deepest mathematical signal: **operand types**.

When a mathematician reads `‖x‖ · a · ‖y‖` vs `‖x · ‖a‖ · y‖`, they
don't count nesting depth — they recognize that:

* `‖·‖` is a function `Vector → Scalar`.
* `·` (multiplication) usually requires compatible types.
* `‖x‖ · a · ‖y‖` is `Scalar · Vector · Scalar = Vector` —
  clean.
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

* `letter |x|` — K-12: `letter · |x|` (scalar/vector ·
  scalar). QM bra-ket: `letter @ |x|` (operator applied to ket).
  Distinguished by whether surrounding context establishes QM
  conventions (which provide a type framework).
* `<a|f|b>` vs `<a, f, b>` — bra-ket
  `quantum-operator-product` (scalar) vs relation chain (boolean).
  The presence of the `|` operator inside angle delimiters
  signals the QM type framework.
* `a|a|+b|b|+c|c|` — multiplication (`Vector ·
  AbsScalar`) summed, vs nested-conditional (set-builder, which
  is a *set type*, not a scalar). The `+` separator forbids the
  set-type reading because sets don't sum.
* `(a, b)` — open-interval (`Interval[Scalar, Scalar]`)
  vs pair / tuple (`Tuple[T, T]`) vs cartesian point
  (`Point[Scalar, Scalar]`). The surrounding context (function
  application, set notation, geometry) determines the type.
* Function application vs implicit multiplication for `f(x)` —
  function-app requires `f : Domain → Range`; multiplication
  treats `f` as a scalar/element.

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
remaining failures) are *proxies* for type checks. Replacing
them with a real type system would be more principled, more
extensible, and would naturally handle composite ambiguities
that don't fit any individual shape pragma.

### Practical sequencing

* **Short term**: continue with shape-based pragmas where they
  cleanly capture a single principle. The `count_*` /
  `prefer_*` family in `semantics/tree.rs` is reusable.
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
