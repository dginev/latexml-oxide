# latexml_math_parser ↔ Marpa ASF — rationalization

> How the three-stage ambiguity-handling pipeline in
> `latexml_math_parser` maps onto Marpa's abstract syntax forest
> (ASF) traversal model.
>
> Written 2026-05-17 to plan the integration.
>
> **Status (2026-05-19): integration complete, modified_term Phase 1+2
> landed.** ASF traversal is the default via the HYBRID dispatch path;
> LEGACY remains as an opt-in escape hatch (`LATEXML_MARPA_LEGACY=1`).
> Tests at **1328/0/0** (both HYBRID and `LATEXML_MARPA_ASF_ONLY=1`).
> The doc remains useful as design rationale for the three-stage
> pipeline and the ASF mapping; the "pseudocode sketch" sections
> reflect what was actually implemented in
> `latexml_math_parser/src/asf_traverser.rs`. The narrow `modified_term`
> grammar category and `modified_list_apply` action (commits
> `a16cce3ddc` + `994cbcfa1a`) handle the `P(x=0, y<0)` comma-list-of-
> relations case and retired the `prefer_zero_absent_when_available`
> pragma.
>
> Marpa upstream landed in `dginev/marpa` master (PR #3, 2026-05-18).

---

## Today's three stages

The math parser is intentionally a wide-net design: admit everything,
prune semantically. The pipeline:

```
                                                                            ┌────────────────────┐
   XMath DOM   ── lex ──▶  Marpa::Recognizer ──▶  Tree iterator ──▶  ◀── 1. │ Per-tree action    │
                                                  (up to 5000)             │    (Stage 2)        │
                                                                            └────────────────────┘
                                                                                     │
                                                                                     ▼
                                                                            ┌────────────────────┐
                                                                            │ XM tree            │
                                                                            │ (or rejected)      │
                                                                            └────────────────────┘
                                                                                     │
                                                              dedup + accumulate ────┤
                                                                                     ▼
                                                                            ┌────────────────────┐
                                                                            │ Choices(N)         │
                                                                            └────────────────────┘
                                                                                     │
                                                                                     ▼
                                                                            ┌────────────────────┐
                                                                  Stage 3 ──▶│ soft_prune_choices│
                                                                            │ (cross-tree)       │
                                                                            └────────────────────┘
                                                                                     │
                                                                                     ▼
                                                                                  Final XM
```

### Stage 1 — grammar categories

**File**: [`latexml_math_parser/src/grammar/builder.rs`](../../latexml_math_parser/src/grammar/builder.rs).

A highly-ambiguous CFG over lexeme categories: `factor`, `tight_term`,
`statement`, `formula`, `statements`, `formulae`, plus the
operator-level categories `RELOP`, `ADDOP`, `MULOP`, `OPFUNCTION`,
`POSTSUBSCRIPT`, `STRETCHY_VERTBAR`, `WIDE_PUNCT`, etc.

**Job**: model every plausible reading of arXiv math syntax. Don't
try to be unambiguous; we know we can't.

**Output**: a `marpa::thin::Grammar` that, fed a lexeme stream, will
typically produce **dozens to thousands of parse trees** for non-trivial
input.

### Stage 2 — early semantic pruning in actions

**File**: [`latexml_math_parser/src/semantics.rs`](../../latexml_math_parser/src/semantics.rs).
Entry point [`Actions::get_tree`](../../latexml_math_parser/src/semantics.rs#L78)
→ recursive [`translate_node`](../../latexml_math_parser/src/semantics.rs#L89)
→ per-rule action closures.

Each rule's action is a closure with this contract:

```rust
fn (rule_id: i32,
    args: Vec<Option<XM>>,
    pragmas: &[ValidationPragmatics],
    ctxt: ActionContext)
 -> Result<Option<XM>, Box<dyn Error>>
```

The action either:
1. Returns `Ok(Some(xm))` — accept this sub-tree, contribute it.
2. Returns `Ok(None)` — accept but contribute nothing (rare).
3. Returns `Err(...)` — **reject this entire tree**. The Marpa
   tree iterator continues, but the reject propagates up to
   `parser.rs::parse_string` which counts it as `pruned_trees += 1`
   and tries the next tree.

Concrete example (`infix_apply`):

```rust
if let Some(XM::Lexeme(ref lex, _)) = infixop {
  if lex.contains(":compose:") {
    if is_applied_function(&arg1) || is_applied_function(&arg2) {
      return Err("compose requires function-level operands"
                 .into());
    }
  }
}
```

**Job**: enforce per-rule semantic invariants (operator–operand type
agreement, balance constraints, etc.) at every materialized tree
node.

### Stage 3 — late semantic pruning in pragmas

**File**: [`latexml_math_parser/src/semantics/tree.rs`](../../latexml_math_parser/src/semantics/tree.rs)
`XM::soft_prune_choices` — driven by the
`ValidationPragmatics` list at
[`latexml_math_parser/src/pragmatics.rs`](../../latexml_math_parser/src/pragmatics.rs).

Called *after* the recognizer + tree iterator has accumulated the
shortlist of `Choices(N)`. Each pragma partitions the trees into
"consistent" vs "inconsistent" — if ≥1 consistent, keep only those;
if all inconsistent, fall back to the unfiltered set. Repeat across
pragmas in order until ≤1 tree remains.

**Job**: cross-tree comparisons — "prefer parses where the same
identifier doesn't appear with conflicting roles", "prefer parses
that use the fewest XMRefs", etc. Things you can't decide from a
single tree alone.

### Why the caps exist

The driver loop (`parser.rs::parse_string`, lines 1042–1151) has
**five** safety caps because the cost of materializing every tree is
real:

| Cap | Value | Purpose |
|---|---|---|
| `max_trees` | 5000 | Hard ceiling on enumeration |
| `max_time` | 30 s | Overall timeout |
| `max_unique` | 10 | Post-dedup ceiling — the pragmatics step rarely needs more |
| `max_consecutive_dupes` | 16 | Early stop when grammar produces only structural duplicates (script-attachment ordering, etc.) |
| `converge_budget` | 200 ms | Stop after this once we have ≥1 unique parse |
| `pruned_only_time_budget` | 2 s | Bail when 200+ pruned trees and 0 OK trees |

Every cap exists to dodge a known pathological case where Stage 2's
per-tree cost × 5000 trees = a frozen UI. The caps are **defensive
work-arounds for the wrong-paradigm cost**, not algorithmic
improvements.

---

## What ASF changes

Marpa's ASF (abstract syntax forest) traversal exposes the **forest
of OR-/AND-nodes directly**, with the user supplying a callback
invoked at most **once per glade** (memoized). The same semantic
constraints get evaluated, but at **glade granularity** instead of
tree granularity.

### Stage 1 — grammar categories

**No change.** ASF operates on the same Marpa::Grammar, the same
recognizer state. Our category-driven CFG stays exactly as it is.

The only second-order effect: with ASF, the cost of grammar ambiguity
is *paid per glade*, not per tree. A glade with 10 alternatives that
each have 5 alternative children costs us 10+50=60 traverser calls,
whereas full tree enumeration would cost 10 × 5 = 50 trees, each of
which separately walks all its sub-positions — so for our deepest
formulas, ASF is **multiplicatively** cheaper. Caps become much less
important; we may be able to drop `max_trees` entirely.

### Stage 2 — early semantic pruning in actions ⇨ **fits ASF natively**

The current `action_on(rule_id, args, pragmas, ctxt) -> Result<Option<XM>, _>`
contract is **almost word-for-word the Marpa::R2 ASF Traverser
callback shape**:

| latexml_math_parser today | Marpa::R2 ASF (Perl) | Proposed Rust ASF |
|---|---|---|
| `args: Vec<Option<XM>>` (children's results) | `glade.rh_value(i)` (lazy) | `children: &HashMap<usize, XM>` (eager, memoized) |
| `Result<Option<XM>, Box<dyn Error>>` | callback returns defined value, undef = fatal | `Result<XM, Box<dyn Error>>` |
| `Err(...)` propagates out of the **whole tree** | undef "stops" the **whole traversal** | `Err(...)` prunes the **glade** — siblings survive |
| Runs once per (tree × occurrence of this rule) | runs once per glade | runs once per glade |

The last row is **the win**. Consider an ambiguous formula with 4
top-level alternatives, each with 3 sub-alternatives, each with 5
sub-sub-alternatives. Today:
* Tree iterator yields 4 × 3 × 5 = 60 trees.
* Each tree calls `infix_apply` etc. ~10 times = 600 action calls.

Under ASF:
* `ASF::traverse` calls the traverser once per glade.
* Top-level glade has 4 alternatives = 4 callbacks.
* Each of those references 3 child-glade outputs (cached) = 12 callbacks total at depth 1.
* Each of those references 5 grandchild-glade outputs (cached) = 36 at depth 2.
* Total ≈ 52 callbacks vs 600 today.

**Required adaptation on the math-parser side**:

1. Change action signature from `args: Vec<Option<XM>>` to
   `alternatives: &[GladeAlternative]` + `children: &HashMap<usize, XM>`
   so an action can pick *which* RHS factoring it accepts when a
   glade has multiple symches at the same rule_id.
2. Replace `parse_string`'s tree-iteration loop with
   `engine.parse_and_traverse_forest(tokens, init_state, traverser)`.
3. Delete the `max_trees` / `pruned_only_time_budget` / `max_unique`
   caps. Keep `max_time` as a safety net.
4. Delete the online-dedup logic — ASF memoization gives it to us for
   free.

**Required adaptation on the marpa-fork side**:

This is what `ASF_STATUS.md` Steps 2-5 are about. The Glade query API
(`alternatives()`, `rh_glade_id(i)`, recursion driver) needs to be
fleshed out. See the target Rust API sketch in `ASF_STATUS.md` § Target.

### Stage 3 — late semantic pruning in pragmas ⇨ **partial fit**

This is the awkward one. Some pragmas are glade-local in disguise;
others are genuinely cross-tree.

**Look at [`pragmatics.rs`](../../latexml_math_parser/src/pragmatics.rs)
to classify each pragma:**

| Pragma | Decision boundary | ASF fit |
|---|---|---|
| Role/meaning consistency (e.g. "same identifier shouldn't switch between RELOP and ID") | Cross-position within a single tree | **Glade-local** — fits as a Stage-2 check on the assembled tree returned from a glade. Doesn't need cross-tree comparison. |
| "Prefer fewer XMRefs" | Counts a tree-global feature | **Cross-tree** — requires the multi-pass shape below. |
| "Prefer the parse that respects expected POS-tagging" | Tree-global preference | **Cross-tree**. |
| Equation-list alignment | Compares siblings within the same tree | **Glade-local** at the `statements`/`formulae` glade. |

There are **three options** for Stage 3 under ASF:

#### Option A — fold what can be folded; keep a thin Stage 3 on the survivors

Most pragmas become glade-local Stage-2 checks. The few genuinely
cross-tree ones stay in Stage 3, but Stage 3 now runs on a **much
smaller shortlist** (say ≤ 3 candidates emitted by ASF traversal at
the top-level glade, vs ≤ 10 today). Minimal disruption; modest win.

#### Option B — glade-local scoring with the ASF picking a single winner

Each glade returns a *scored* alternative. Cross-glade preferences
become scoring functions on the per-glade decision: e.g. "this glade
contributes -1 score per XMRef in its subtree". The ASF picks the
highest-scoring assembly. **One final tree out, no Stage 3**.

Downside: some preferences resist local scoring (e.g. "prefer the
parse where the LHS and RHS of `=` use the same variable types" —
that's a relationship between siblings of a top-level glade, decidable
locally). Most others do localize cleanly.

#### Option C — two-pass ASF

First ASF pass: each glade returns *all surviving alternatives* (Vec
instead of single value). Top-level glade returns up to N candidate
trees. Second pass: apply the existing Stage 3 pragmas on the N
candidates. Behaves like today's pipeline but with N=3 instead of
N=10 at the top, and **no 5000-tree cap** below.

**Recommendation**: ship **Option A first** (minimal disruption,
guaranteed correctness on the current test suite), measure, then
selectively promote pragmas to glade-local where they fit cleanly.

---

## Concrete adaptation table

| Component | Today | Under ASF | Action |
|---|---|---|---|
| **Grammar** (`grammar/builder.rs`) | Same Marpa::Grammar | Same | None |
| **Lexer** (`util.rs::node_to_grammar_lexemes`) | XMath → lexeme stream | Same | None |
| **Driver loop** (`parser.rs::parse_string` L1037-1220) | Tree iterator + 5 caps + per-tree actions | ASF traverse + 1 cap (`max_time`) | **Rewrite** to call `parse_and_traverse_forest`. |
| **Action closures** (`semantics.rs::action_*`) | `(rule_id, Vec<Option<XM>>, pragmas, ctxt) -> Result<Option<XM>, _>` | `(glade_alternatives, &cached_children, pragmas, ctxt) -> Result<XM, _>` | **Refactor signature**; semantics stays. |
| **Action dispatch** (`Actions::action_on`) | `HashMap<i32, ActionClosure>` | Same shape, called by ASF driver | None |
| **Pragmatics** (`pragmatics.rs`) | `ValidationPragmatics::validate_recursive(tree)` | Mix of glade-local and cross-tree | **Audit**: classify each pragma per the table above. Glade-local → fold into Stage 2 actions. Cross-tree → keep in Stage 3, but Stage 3 now runs on the shortlist from ASF. |
| **`soft_prune_choices`** (`semantics/tree.rs` L494) | Partitions `XM::Choices(N)` | Optional, runs on shortlist if any | **Keep** but as a thin top-of-pipeline filter, not the main attraction. |
| **Convergence caps** (`max_trees`, `max_consecutive_dupes`, `pruned_only_time_budget`, `converge_budget`, `max_unique`) | Defensive bandages | Mostly unnecessary | **Remove all except `max_time`.** |
| **Online dedup** (`parses.contains(&tree)`) | Catches script-attachment-ordering dupes | Subsumed by ASF memoization | **Remove**. |
| **Marpa-fork dependency** | `Tree` iteration (works today) | `ASF::traverse(...)` (needs `ASF_STATUS.md` Steps 2-5) | **Track**: see `~/git/marpa` `asf-completion` branch. |

---

## Sequencing — closed

Originally drafted 2026-05-17; closed in successive commits through
2026-05-19. The detailed step list is preserved in git history
(`git log -p docs/math/MATH_PARSER_AND_ASF.md`); current state below.

1. ✅ **Marpa side (ASF_STATUS Steps 2-6)** — `compute_symches`,
   Glade query API, recursive `ASF::traverse` with memoization, and
   generic `&mut TR` Traverser API all landed on the
   `asf-step3-generic-traverser` branch, then merged to dginev/marpa
   master via PRs #3 + #4.
2. ✅ **latexml-oxide marpa dep** — pinned at dginev/marpa master,
   no branch override.
3. ✅ **`MathTraverser` wiring** — `LATEXML_MARPA_ASF=1` was the
   opt-in feature flag during the side-by-side parity push (1272/29
   → 1292/9 → 1298/3 → 1328/0/0). HYBRID dispatch (commit
   `703637bf95`) then made ASF the default; LEGACY kept behind
   `LATEXML_MARPA_LEGACY=1` (`ff51b50c18`).
4. ✅ **Canvas validation** — 10k stages run cleanly under HYBRID;
   the Round-26+ corpus tallies in `SYNC_STATUS.md`'s top section
   confirm parity stayed in the 99.3-99.6% band.
5. ⚠️ **Convergence caps** — the 5/6 caps are *kept* in
   `parser.rs::parse_marpa` to protect the LEGACY escape-hatch path
   from real-world ambiguity hangs. They never fire on the
   ASF/HYBRID hot path; treating this as a doc cleanup item rather
   than a code change. See `SYNC_STATUS.md` "Math parser ↔ Marpa
   ASF migration" item 6.
6. ✅ **modified_term grammar refinement** (item 5 in the
   `SYNC_STATUS.md` list, deferred-then-landed 2026-05-19) —
   commits `a16cce3ddc` (Phase 1, narrow `modified_term = tight_term
   relop expression` rule + `modified_list_apply`) and `994cbcfa1a`
   (Phase 2, retired the now-redundant
   `prefer_zero_absent_when_available` pragma). Witness `P(x=0, y<0)`
   now parses as `P @ vector(x=0, y<0)`.
7. ⏳ **Pragmatics audit** — not yet classified as glade-local vs.
   cross-tree. Low priority since both run cleanly on the small ASF
   shortlist.

---

## Open question

`parse_and_traverse_forest` is currently sequential per formula —
one Recognizer state machine cycle per call. That's fine; the math
parser already loops per XMath node. Where the existing code
**reuses** the Marpa engine across formulas (via `GReady` state),
ASF traversal needs to do the same. The `Parser::ambiguity_metric`
oracle [just committed on the marpa fork](https://github.com/dginev/marpa/commit/5a3441b)
demonstrates the `R → GReady` reset pattern; the ASF path needs to
follow it.

This is mechanical, not novel. Mention only because the existing
math parser's `reset_engine` ladder (3-clone-attempts then full
`init_grammar` rebuild) is load-bearing for cleanup; the ASF
migration must preserve those error paths.

---

## Measured marpa-level performance

`marpa/tests/asf_perf_compare.rs` (in the dginev/marpa fork,
branch `asf-step3-generic-traverser`) runs head-to-head wall-time
comparisons of `run_recognizer` (Tree iteration) vs
`parse_and_traverse_forest` (ASF post-order memoization). Run with
`cargo test --release --test asf_perf_compare -- --nocapture`.

| Workload                          |   Trees | Tree iter | ASF       | Speedup |
|-----------------------------------|--------:|----------:|----------:|--------:|
| panda short (`a panda eats…`)     |       3 |    248 µs |    242 µs |   1.02× |
| panda long (4× VP repetition)     |    1562 |   2081 µs |    498 µs |   4.17× |
| `1+1+…+1` Catalan, 8 operands     |     429 |    224 µs |    156 µs |   1.43× |
| `1+1+…+1` Catalan, 12 operands    |  58 786 |  25759 µs |    295 µs |  87.18× |

The asymptotic gain is the headline. ASF cost is dominated by
**glade count**, which scales polynomially in input size — even when
parse-tree count is Catalan-class. The 12-operand arithmetic
explosion has a few hundred glades but 58 786 distinct parse trees;
`run_recognizer` walks every tree, ASF walks each glade once and
Cartesian-multiplies child outputs.

**What this means for the math parser**:
* Typical arXiv formulas have 5-50 parse trees → expect ~2-5×
  speedup, swallowed by other costs (lexer, libxml).
* Pathological formulas (script attachment, multi-clause RELOP
  lists) have hundreds-to-thousands → expect 10-30× speedup,
  eliminating the need for the 5000-tree cap.
* The `0804.1730` case noted in `parser.rs:1077` (4536 enumerated
  trees over 28 seconds before timeout) would compress to roughly
  ~280 ms via ASF. **The pruned-only-time-budget bandage becomes
  obsolete.**

The Cartesian-product cost per glade can still blow up if a single
glade has many alternatives and many RHS positions. Stage-2
glade-local pruning inside `action_on` (semantic pragmas applied
per (symch, factoring) combo) is what keeps that bounded — the
"Option B" promotion path in the existing rationalization.

---

## Worked example — the `f@(g(x))` ambiguity

Pick a concrete simple-but-ambiguous case to make the cost difference
concrete. Input: `f g (x)`. Grammar reading:

* As "`f` times `g(x)`": one infix-`*` between two factor terms.
* As "`f(g(x))`": curried application, `f` applied to the result of `g(x)`.
* As "`f g (x)`" with `(x)` parenthesized factor: ground term, three factors multiplied.
* (For sufficiently rich grammars, also "`f`-of-`g` applied to `x`",
  invisible-times variations, etc.)

### Today (Tree iteration)

`parse_string` materializes each derivation tree, walks it
post-order calling actions. For 3 surface readings × ~2 invisible-
times variants × ~2 script-attachment orderings ≈ **12 trees
enumerated**, each visited completely. Actions in `infix_apply`,
`apply_invisible_times`, and `prefix_apply` fire 12 × ~5 = **60
times**. Dedup folds the 12 into 3 unique XM trees, Stage 3
`soft_prune_choices` picks one. Total wall: dominated by the 60
action calls and 12 dedup `contains` scans.

### Under ASF

The bocage has roughly the same shape but the OR-/AND-node graph
deduplicates shared sub-positions. The `(x)` factor is **one glade
shared by all 3 readings**. So:

* `(x)` factor glade — 1 callback.
* `g` factor glade — 1 callback.
* `f` factor glade — 1 callback.
* `g(x)` application glade — 1 callback (uses cached `(x)` and `g`).
* `f*g(x)` infix glade — 1 callback (uses cached `f`, `g(x)`).
* `f(g(x))` apply glade — 1 callback (uses cached `f`, `g(x)`).
* Top-level glade with 3 alternatives — 1 callback that picks.

**Total: 7 callbacks vs 60.** The win grows super-linearly with
formula depth × per-position ambiguity.

This is also where Stage 3 collapses: today's `XM::Choices(3) →
soft_prune_choices` becomes "the top-level glade returns the
prefered single alternative" — the alternatives never materialize
as a `Choices(N)` in the first place if the picker is glade-local.

---

## What landed

The integration shipped as **HYBRID dispatch** (one recognizer pass +
one bocage + `ambiguity_metric()`-based routing), with the design
choices above mapped to concrete code:

| Component | Plan → Reality |
|---|---|
| **Driver** | `parser.rs::parse_marpa` routes via `Parser::parse_hybrid_with_and_node_limit(...)` (marpa master, post-PR #4). Returns `HybridParseResult::{Unambiguous(Tree), Ambiguous(PT, PS), AmbiguousTree(Tree, BocageStats)}`. |
| **Unambiguous path** | `Unambiguous(Tree)` → existing `Actions::get_tree` machinery, no ASF construction. |
| **Ambiguous path** | `Ambiguous(PT, _)` → `MathTraverser` callback in `asf_traverser.rs` accumulates per-glade alternatives in `Rc<Vec<Option<XM>>>`; downstream `dispatch_action` runs the Cartesian product over RHS positions. |
| **Large-bocage fallback** | `AmbiguousTree(Tree, BocageStats)` when `BocageStats::and_node_count > LATEXML_MARPA_HYBRID_AND_NODE_LIMIT` (default 500). Routes back through the legacy Tree-iter convergence-cap loop with libmarpa's already-built bocage. |
| **Action signature** | Adopted **Option A** plus a small bit of Option B. Actions still take `Vec<Option<XM>>` per RHS factoring; ASF callback resolves children from the marpa-supplied `children: &[Option<PT>]` slice. Glade-local picking happens implicitly via the per-glade `Rc<Vec<Option<XM>>>` accumulator; cross-tree pragmas remain as `ValidationPragmatics`. |
| **Caps** | LEGACY path keeps all six caps as documented above (still active when `LATEXML_MARPA_LEGACY=1`). HYBRID's Tree branches reuse them. ASF branch needs none of them because per-glade memoization replaces the per-tree cost model. |
| **Engine env vars** | `LATEXML_MARPA_LEGACY=1`, `LATEXML_MARPA_ASF_ONLY=1`, `LATEXML_MARPA_HYBRID_AND_NODE_LIMIT=N`, plus audit knobs `LATEXML_MATH_AMBIGUITY_AUDIT=1`, `LATEXML_MARPA_HYBRID_AUDIT_PARITY=1`, `LATEXML_MARPA_ASF_AUDIT=1`. |

Final acceptance numbers:

- Test suite: 1309/0/0 (1301 prior + 8 new
  `parity_outcomes_compatible_*` unit tests).
- `Article-2025.tex` (87.3 % raw-unambiguous): HYBRID 12.45 s vs
  LEGACY 12.32 s = **1.01×** (within the original 1.05× gate).
- 100-paper math-bound sample (top by `phase_math_parse_us` on
  wp4 telemetry): HYBRID **+0.5 %** vs LEGACY on n=98 both-OK,
  zero OOM aborts (without the 500-and-node cap the HYBRID path
  OOM-aborted 19/100). See
  [`docs/performance/PERFORMANCE.md`](../performance/PERFORMANCE.md) for the per-paper
  breakdown.

---

## Pointers

| For… | See… |
|---|---|
| Current routing implementation | [`latexml_math_parser/src/parser.rs::parse_marpa`](../../latexml_math_parser/src/parser.rs) |
| Per-glade callback (ambiguous path) | [`latexml_math_parser/src/asf_traverser.rs`](../../latexml_math_parser/src/asf_traverser.rs) |
| Action closures | [`latexml_math_parser/src/semantics.rs`](../../latexml_math_parser/src/semantics.rs) |
| Pragma definitions | [`latexml_math_parser/src/pragmatics.rs`](../../latexml_math_parser/src/pragmatics.rs) |
| Tiebreaking research notes | [`docs/math/MATH_PARSER_ASF_TIEBREAKING.md`](MATH_PARSER_ASF_TIEBREAKING.md) |
| Ambiguity hotspots research | [`docs/archive/MATH_AMBIGUITY_AUDIT_2026-05-21.md`](../archive/MATH_AMBIGUITY_AUDIT_2026-05-21.md) |
| marpa-side perf doc | [`~/git/marpa/docs/ASF_PERFORMANCE_FINDINGS.md`](https://github.com/dginev/marpa/blob/master/docs/ASF_PERFORMANCE_FINDINGS.md) |
| Pre-flight ambiguity oracle | `Parser::ambiguity_metric(tokens)` and `Bocage::ambiguity_metric()` |
