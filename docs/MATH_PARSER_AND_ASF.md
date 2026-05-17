# latexml_math_parser ↔ Marpa ASF — rationalization

> How the three-stage ambiguity-handling pipeline in
> `latexml_math_parser` maps onto Marpa's abstract syntax forest
> (ASF) traversal model, and what would change on each side if we
> switch to ASF-driven pruning.
>
> Written 2026-05-17 in dialog with the [`asf-completion` branch
> of dginev/marpa](https://github.com/dginev/marpa/tree/asf-completion)
> and its [`ASF_STATUS.md`](https://github.com/dginev/marpa/blob/asf-completion/ASF_STATUS.md).

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

**File**: [`latexml_math_parser/src/grammar/builder.rs`](../latexml_math_parser/src/grammar/builder.rs).

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

**File**: [`latexml_math_parser/src/semantics.rs`](../latexml_math_parser/src/semantics.rs).
Entry point [`Actions::get_tree`](../latexml_math_parser/src/semantics.rs#L78)
→ recursive [`translate_node`](../latexml_math_parser/src/semantics.rs#L89)
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

**File**: [`latexml_math_parser/src/semantics/tree.rs`](../latexml_math_parser/src/semantics/tree.rs)
`XM::soft_prune_choices` — driven by the
`ValidationPragmatics` list at
[`latexml_math_parser/src/pragmatics.rs`](../latexml_math_parser/src/pragmatics.rs).

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

**Look at [`pragmatics.rs`](../latexml_math_parser/src/pragmatics.rs)
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

## Sequencing

Updated 2026-05-17 after ASF_STATUS Steps 2-6 landed on marpa branch
`asf-step3-generic-traverser`.

1. **Marpa side (ASF_STATUS Steps 2-6)** — ✅ **LANDED.** Branch
   `asf-step3-generic-traverser` on dginev/marpa carries:
   * Ported `compute_symches` with Perl-faithful glade unification.
   * Full Glade query API (`rule_id`, `rh_length`, `rh_glade_id`,
     `next`, `cursor`, `symch_count`, `factor_count`, `is_factored`,
     `is_token`).
   * Recursive `ASF::traverse` with post-order memoization.
   * Generic `&mut TR` Traverser API (no `Box<dyn>` — allows
     borrowing).
   * Substantive 3-parse test on the panda grammar.
   * 17 marpa tests pass.
2. **latexml-oxide Cargo.toml** — ✅ **LANDED.** marpa dep switched
   to the new branch; full test suite 1301/0/0 against it.
3. **`MathTraverser` scaffolding** — ✅ **LANDED**, file
   `latexml_math_parser/src/asf_traverser.rs`. Compiles, handles
   byte/lexeme-rule/standard glades, but **not yet wired into
   `parse_marpa`**.
4. **⏳ Wire `MathTraverser` behind `LATEXML_MARPA_ASF=1`.** Side-by-
   side parity check against the legacy tree-iteration path on the
   full test suite. Expect 0 regressions. Iterate on edge cases.
5. **⏳ Validate on the 10k canvas stage.** Sandbox parity should
   stay above the current 97.4-99.5% baseline.
6. **⏳ Delete 5 of the 6 convergence caps** in `parser.rs::parse_marpa`.
   Only `max_time` stays. Delete online `parses.contains(&tree)`
   dedup (memoization renders it pointless).
7. **⏳ Audit `pragmatics.rs`** — classify each pragma as glade-local
   (promote into Stage 2 inside `action_on`) or cross-tree (keep on
   the now-small ASF shortlist).
8. **⏳ Merge marpa PR to dginev/marpa master**, switch
   latexml-oxide's marpa dep back to default-branch.

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

## Pseudocode sketch of the new driver

The current loop (heavily reduced):

```rust
let parse_result = self.engine.run_recognizer(...)?;
for val in parse_result {
  if caps_exceeded() { break }
  match self.actions.get_tree(builder, val, pragmas, ctx) {
    Ok(Some(tree)) => { dedup_and_collect(tree) }
    Err(_) => pruned_trees += 1,
    Ok(None) => {}
  }
}
match parses.len() {
  0 => Err(...), 1 => Ok(parses.pop()),
  _ => Ok(soft_prune_choices(Choices(parses), pragmas)),
}
```

becomes:

```rust
struct MathTraverser<'a> {
  actions:  &'a Actions,
  pragmas:  &'a [ValidationPragmatics],
  context:  ActionContext<'a>,
}

impl Traverser for MathTraverser<'_> {
  type Output = XM;
  fn traverse_glade(
    &mut self,
    glade_id: usize,
    alternatives: &[GladeKind],
    children: &HashMap<usize, XM>,
  ) -> Result<XM> {
    // For each alternative (already a single rule_id + child-id list),
    // try the corresponding action with children resolved from cache.
    // Collect the survivors; ask the glade-local scoring to pick.
    let mut candidates = Vec::with_capacity(alternatives.len());
    for alt in alternatives {
      let xm_args = collect_args(alt, children);
      match self.actions.action_on(alt.rule_id(), xm_args, self.pragmas, ...) {
        Ok(Some(xm)) => candidates.push(xm),
        Ok(None) | Err(_) => {}  // pruned at this glade only — siblings live
      }
    }
    match candidates.len() {
      0 => Err("no surviving action at this glade".into()),
      1 => Ok(candidates.pop().unwrap()),
      _ => Ok(glade_local_pick(candidates, self.pragmas)),
    }
  }
}

let traverser = MathTraverser { actions: &self.actions, pragmas: &self.expert_pragmatics, context: ... };
let final_tree = self.engine.parse_and_traverse_forest(tokens, traverser)?;
```

Where `glade_local_pick` is either:

* Option A — just `XM::Choices(candidates)`, deferring to top-level Stage 3.
* Option B — `self.student_pragmatics.iter().fold(...)` applied locally.

The bocage memoization is the ASF layer's responsibility — actions
need not check whether a sibling glade has already been visited.

### What `parser.rs::parse_string` looks like after the cut

Reduced from ~200 lines to roughly:

```rust
fn parse_string(&mut self, input: &str, nodes: &[Node], doc: &mut Document) -> Result<XM> {
  let traverser = MathTraverser { actions: &self.actions, pragmas: &self.expert_pragmatics,
                                  context: ActionContext { nodes, document: doc } };
  match self.engine.parse_and_traverse_forest_with_timeout(
    ByteScanner::new(Cursor::new(input)),
    traverser,
    Duration::from_secs(30),  // only surviving cap
  ) {
    Ok(xm) => Ok(self.student_pragmatics.iter().fold(xm, XM::soft_prune_choices)),
    Err(e) if e.is_timeout() => { self.reset_engine(); Err("math parse: timeout".into()) }
    Err(e)                   => { self.reset_engine(); Err(e) }
  }
}
```

Six caps → one (`max_time`). The `reset_engine` ladder stays.

---

## Test plan for the migration

The migration is invariant-preserving by design. The test gates:

1. **Existing unit tests** (`tests/700_unit_parse.rs`,
   `tests/701_unit_footnote.rs`) must stay 100 % green at every
   step — these are the math-parser regression fixtures.
2. **Full test suite** `cargo test --tests` must stay at **1301/0/0**
   (current baseline as of 2026-05-17).
3. **Sandbox 10k stage** (`tools/staged_canvas_sweep.sh` or
   equivalent) — current baseline 97.4–99.5 % OK per
   [`docs/SYNC_STATUS.md`](SYNC_STATUS.md). ASF should match or
   exceed this. Any drop is a regression and must be root-caused.
4. **Per-formula wall time** — `LATEXML_PARSE_AUDIT=1` on the
   astro-ph corpus (5 formulas listed in `docs/SYNC_STATUS.md` §
   Performance follow-ups). Expect ~order-of-magnitude reduction
   on highly-ambiguous formulas.

Pin-down tests to add **before** the migration:

* Snapshot the current `_parsetrees` count on a small handful of
  pathological formulas (e.g. `{}^4{}_{12}C^{5+}`,
  `\displaystyle\frac{1}{2\pi i}\int...`). After migration the count
  semantics changes — capture the pre-migration values as ASF-era
  expected upper bounds.
* Add a `LATEXML_PARSE_PARADIGM=tree|asf` env var so we can A/B test
  on the same input.

---

## Quick reference for the next session

| Want to know… | Look at… |
|---|---|
| The full audit of where the math parser is today | This doc + [`docs/MATH_AMBIGUITY_AUDIT.md`](MATH_AMBIGUITY_AUDIT.md) |
| What ASF traversal looks like target-side | [`marpa/ASF_STATUS.md`](https://github.com/dginev/marpa/blob/asf-completion/ASF_STATUS.md), § Target Rust API |
| Background on Marpa's algorithms | [`marpa/background/`](https://github.com/dginev/marpa/tree/asf-completion/background) — Kegler 2023 papers |
| What's left to build on the marpa side | [`marpa/ASF_STATUS.md`](https://github.com/dginev/marpa/blob/asf-completion/ASF_STATUS.md), § Completion plan Steps 2-5 |
| Current caps in the math parser driver | [`latexml_math_parser/src/parser.rs::parse_string`](../latexml_math_parser/src/parser.rs#L1037-L1220), lines 1053-1077 |
| Action closures (Stage 2 today) | [`latexml_math_parser/src/semantics.rs`](../latexml_math_parser/src/semantics.rs) |
| Pragma definitions (Stage 3 today) | [`latexml_math_parser/src/pragmatics.rs`](../latexml_math_parser/src/pragmatics.rs) |
| Pre-flight ambiguity oracle (already landed) | [`marpa Parser::ambiguity_metric`](https://github.com/dginev/marpa/commit/5a3441b) — usable today via the standard marpa API |
