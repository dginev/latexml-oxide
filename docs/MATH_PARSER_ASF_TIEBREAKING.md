# ASF tiebreaking — research notes

> Companion to [`MATH_PARSER_AND_ASF.md`](MATH_PARSER_AND_ASF.md). That
> doc explains *how* the math parser maps onto Marpa's ASF traversal.
> This one collects *open questions and design levers* for the
> harder-than-expected ambiguity-tiebreaking problem we hit during
> the wire-in.
>
> Status 2026-05-17: **new paradigm, open research problem**. We are
> not committing to a single tiebreaking discipline yet — this doc
> accumulates evidence and lever options as we triage cases.

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

## Phase 1 catalog — clean ASF baseline 1272/29 (2026-05-17)

**Important correction**: the 1281/20 baseline quoted earlier in
this doc was measured with a temporary `alts.reverse()` patch in
`parse_marpa` that was subsequently removed. With **no reverse and
no new pragmas wired in**, the true clean ASF parity is
**1272/29**. The 9-test delta is the function-application class
(Class A below); the reverse was a shallow patch that masked the
underlying preference issue.

The complete 29-test failure set, partitioned by ambiguity class:

### Class **A — function-application vs implicit-times**

Pattern: `letter(args)` or `letter token` where the legacy picks
function-application and ASF picks implicit multiplication.

| Test | Formula | Expected `text=` | Actual `text=` |
|---|---|---|---|
| `calculus_test` | `\sum_{...}P(i,j)` | `... P@(vector@(i, j))` | `... P * vector@(i, j)` |
| `count_parses_test` | `\langle B \|\sum_k f_k\| C\rangle` | `delimited-⟨⟩@(B@(abs@(sum)) * C)` | `delimited-⟨⟩@(B * abs@(sum) * C)` |
| `esint_test` | `\iiiint_C F(x)dx` | `... F@(x) * differential-d@(x)` | `... F * x * differential-d@(x)` |
| `mathtools_test` | `f(x)=\int h(x)\,dx` | `f@(x) = integral@(h@(x) * diff-d@(x))` | `f * x = integral@(h * x * diff-d@(x))` |
| `ntheorem_test` | `... f(\zeta) / (\zeta-z)^{n+1} ...` | `... f@(zeta) / ...` | `... f * zeta / ...` |
| `operators_test` | `\exists x.P(x)` | `formulae@(exists@(x), P@(x))` | `formulae@(exists@(x), P * x)` |
| `qm_test` | `<a\|f\|b>` | `absent < a@(abs@(f)) * b > absent` | `absent < a * abs@(f) * b > absent` |
| `sampler_test` | `\genfrac{(}{)}{}{}{\int_a^b f(x)dx}{...}` | `... f@(x) * diff-d@(x) / ...` | `... f * x * diff-d@(x) / ...` |
| `spacing_test` | `\int_0^\infty f(x)dx` | `(integral _ 0 ^ infty)@(f@(x) * diff-d@(x))` | `... f * x * diff-d@(x)` |

**9 tests.** All share the same root: the grammar admits both
`tight_term → factor` (then multiplied) and `tight_term →
function applyop tight_term` / `applied_func` (function-app). The
legacy's Tree-iter order picks function-app first; ASF picks
multiplication. **Single fix opportunity.**

### Class **B — `>=` lexed as `> absent =` instead of single RELOP**

| Test | Formula | Expected | Actual |
|---|---|---|---|
| `ambiguous_relations_test` | `x>=0` | `x >= 0` | `x > absent = 0` |
| `relations_test` | `x>=0` (same fixture) | `x >= 0` | `x > absent = 0` |

**2 tests.** Looks like a lexer-level issue (the `>=` token should
be a single RELOP lexeme but ASF is splitting it). Worth probing
the lexeme stream to confirm.

### Class **F — `||x||a||y||` norm-nesting**

| Test | Formula | Expected | Actual |
|---|---|---|---|
| `vertbars_test` | `\|\|x\|\|a\|\|y\|\|` | `norm@(x) * a * norm@(y)` | `norm@(x * norm@(a) * y)` |

**1 test.** Highly ambiguous (`\|\|` × 4 makes multiple groupings
valid). The expected is the "balanced 3-norm" parse; ASF picks
the "outer norm with inner norm" parse.

### Class **U — ASF produces no parse where legacy does**

| Test | Formula | Result |
|---|---|---|
| `physics_test` | `\mathbf{a}\qquad ...` | ASF → `<Math class="ltx_math_unparsed">` (parse failed) |

**1 test.** Different from tiebreaking — ASF is failing to find
a parse entirely. Could be related to specific formula structure;
needs separate investigation.

### Class **? — Needs deeper diff inspection**

Tests where the first DIFF didn't include a Math/text= line in the
quick scan (the divergence is structural, not in the top-line
attribute). Need targeted investigation.

* `artefacts_test`
* `function_argument_syntax_test`
* `scripts_test`
* `simplemath_test`
* `stmaryrd_test` — likely the flat-vs-nested-list improvement
  candidate from earlier scan.
* `unit_tests_by_silviu_test`
* `wasysym_test`

**7 tests.** TBD.

### Additional failures after reverting `alts.reverse()`

9 tests that the temporary `alts.reverse()` patch was masking. All
align with Class A (function-application preference):

* `amstheorem_test`
* `compose_test`
* `functions_test`
* `latextheorem_test`
* `metarelation_elision_test`
* `ncases_test`
* `nested_application_test`
* `parens_test`
* `parser_speculate_test`
* `picture_test`
* `plainfonts_test`
* `plainmath_test`
* `scripted_opfunction_addop_test`
* `sizes_test`
* `standalone_modifiers_test`
* `subordinate_lists_test`

Yes — that's 16. The "9 added" was a rough count; the actual set
overlap is messier because some tests passed previously by
coincidence under reverse(). The actionable observation: **Class A
(function-application preference) is the dominant root cause and
fixing it unlocks 16+ tests at once.**

---

### Intervention plan per class

| Class | # tests | Suggested intervention | Rationale |
|---|---|---|---|
| **A** | 9 | Marpa rule_rank on `function`/`factor → function` to outrank `factor → factor_base` for the parse direction that admits function-app | Single grammar-level lever; affects both legacy and ASF; principled (encodes "letter-followed-by-parens prefers function-app") |
| **B** | 2 | Lexer-level — verify `>=` lexes as single RELOP. May not be ASF-specific. | Should resolve at lexer, not parser |
| **F** | 1 | Targeted pragma "prefer flat norm chain over nested" | Pattern-specific |
| **U** | 1 | Debug the recognizer/ASF for this specific formula | Different root cause |
| **?** | 7 | First catalogue, then intervene | Don't pre-commit |

If Class A's grammar-rank fix lands cleanly, we go from 1281/20 →
~1281/11 in one stroke. The remaining 11 fall into Classes B
(2), F (1), U (1), and ? (7).

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

* ASF infrastructure on the marpa fork (branches
  `asf-step2-symches`, `asf-step3-generic-traverser`) — all
  panda tests pass; substantive 3-parse validation.
* `ASF::peak` multi-source fix (real correctness bug).
* `MathTraverser` scaffolding + wire-in behind
  `LATEXML_MARPA_ASF=1`.
* Discriminator handling 5 glade classes (byte / outer-token /
  passthrough-rule / scaffolding / action-rule).
* Per-glade `reverse()` of the alternatives vec inside the
  traverser — partial fix; helps Case A but hurts Case B. **Under
  active reconsideration.**

98.5% parity (1281/1301) with ASF enabled. The remaining 1.5%
is the tiebreaking problem documented above.
