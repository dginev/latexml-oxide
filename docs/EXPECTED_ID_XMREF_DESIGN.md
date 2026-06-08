# `expected:id` dangling-XMRef — Design Scope

> **Status:** DESIGN (no code yet). Scopes the fix for the residual `expected:id
> Cannot find a node with xml:id='…'` cluster — the non-VERTBAR XMDual-dangling
> remainder after the bra-ket VERTBAR work (`50c8a6a35e`, `275d249acc`) landed.
> **Author handoff:** written 2026-06-08. **Witnesses:** `2311.01600` (App. eq.
> E66), `2207.08945` (E49), `2306.04445` (alg2.m13), `2307.02913` (Ex17). All
> **Perl = 0 errors** on these papers (Rust-only). Related history:
> [[project_xmref_dangling_split]], `MATH_PARSER_AND_ASF.md` (ASF + LOSTNODES).

## 1. Symptom

Post-processing (`latexml_post` `mark_xm_node_visibility_aux` / `realize_xm_node`,
faithful ports of Perl `Post::Document`) raises `Error:expected:id Cannot find a
node with xml:id='A1.E66.m1.1a'`. The idref lives in an **XMDual content branch**,
e.g. (intermediate XML of 2311.01600):

```
<XMApp><XMTok meaning="probability"/><XMRef idref="A1.E66.m1.1a"/></XMApp>
```

i.e. a custom `\Pr`-style function macro's dual. The conversion still completes
(rc=0) — the error is logged but output is produced. Each affected paper emits
2–12 such errors.

## 2. The id-management machinery (3 layers) and where E66 escapes all 3

A math `XMRef` gets its concrete `idref` from one of two resolvers, and absorbed
nodes are reconciled by a third:

| Layer | Where | Mechanism |
|---|---|---|
| **A. `createXMRefs`** | `base_xmath.rs` `ltx:XMDual` `after_close_late` | For `_xmkey`-tagged nodes: `generate_id` the target, set the XMRef `idref` to the target's **actual** id. |
| **B. `resolve_xmkeys`** | `parser.rs:2662` (post-parse, per XMath) | Same idea for parser-generated `_pxmkey`/`_xmkey` (apply_delimited / grammar duals). |
| **C. LOSTNODES** | `parser.rs:711` cleanup + `:1410` record | After a math's tree-replacement, any pre-snapshot id that no longer resolves is recorded `lost→__LOSTNODE__`; the cleanup walks `//ltx:XMRef[@idref]` and **rewrites** (transitive) or **unlinks** (orphan). Mirrors Perl `MathParser.pm` L287-297. |

`idref="…m1.1a"` is an **actual** id assigned by A or B (the `a`/`b` suffix is
`Document::modify_id` → `radix_alpha` collision disambiguation — two sub-nodes
both claimed base `…m1.1`). So at assignment time the target node existed. By the
**intermediate XML** (post-parse, pre-post) it is **gone**, and **no node ever
carries it again** (verified: `grep xml:id="A1.E66.m1.1a"` → 0 hits).

**The gap (root cause).** Layer C's orphan snapshot (`pre_replacement_ids`,
parser.rs:1383-1392) captures only ids present in the **OLD subtree BEFORE** the
parse. But `…m1.1a` is **created during the parse** (`generate_id` on the new
tree) and **lost during the parse / a subsequent rewrite** (the `\left.`/`\right.`
multi-line split + the function-macro dual restructure dissolves or re-ids the
node). A parse-time-created-then-lost id is in **neither** the pre-snapshot **nor**
the LOSTNODES map, so the cleanup's `resolve(idref)` returns `None` and the
dangling ref is left untouched → Post errors.

**Disproved hypothesis (don't re-try):** widening `Document::record_node_ids`'
idref redirect from subtree-scoped to document-wide changed **nothing** on the
witnesses — so this is **not** a `modify_id`-collision redirect-scope miss; it is
the parse-time create-then-lose path above.

## 3. Phase 0 — grounding still needed (do first, ~½ day)

The design below is robust to either sub-case, but pin the exact create/lose site
to choose Tier 2 well. Instrument under an env flag (`LXDBG_XMREF`):
1. In **A/B** when an idref is set, log `(idref, target xml:id, node ptr)`.
2. In the parser's node-removal/replace sites (`replace_tree` parser.rs:~456/690,
   `unbind_node` :639/856, `rewrite.rs:522`, semantics `into_xmath`/absorb) log
   removed `xml:id`s.
3. Cross the logs for `A1.E66.m1.1a`: which layer sets it, which site drops it,
   and whether `record_replacement` was reachable there. Confirm whether the node
   is **dissolved** (content merged up, no successor — Tier 1 territory) or
   **re-id'd / moved** (a successor exists — Tier 2 can redirect).

## 4. Design options

**Option A — Robust reconciliation (catch parse-time orphans).** *Low risk, broad,
clears the error cluster; mild fidelity cost.* Extend Layer C: after the existing
LOSTNODES cleanup, add a **final unresolved-idref sweep** — for every
`//ltx:XMRef[@idref]` whose `idref` does **not** `lookup_id`, treat it as an
orphan (the LOSTNODES sentinel path: unlink the XMRef, or replace with
`<XMTok meaning="absent"/>` so the content arity is preserved). This catches
create-then-lose-in-parse ids that the pre-snapshot misses. It is an extension of
an **already-established pattern** (the sentinel already unlinks orphans). Cost: a
dropped ref → the content arg renders "absent" (cf. the `[…, [], …]` seen during
the Dirac work) — **degraded vs Perl's correct content, but no error**, and only
on the arg that was already lost.

**Option B — Faithful preservation (Tier 2).** *Higher risk, per-construct.*
Use Phase-0 to find why the node is lost and either (i) record a proper
`record_replacement(lost_id, successor_id)` at that removal site (so C **redirects**
rather than drops — keeps content correct), or (ii) stop the dissolution (as the
Dirac prune did for bra-ket). This is the genuinely faithful fix but is bespoke to
the `\left.`/`\right.`-split + function-macro-dual interaction and risks the whole
math corpus.

**Option C — Defer concrete idref assignment to a single post-parse/post-rewrite
pass.** Keep `_xmkey` symbolic through ALL parsing + rewriting; resolve every
`_xmkey → final id` exactly once at the end. Avoids the create-then-lose-then-
dangle window **iff** the `_xmkey`-bearing node survives (it moves with its
attribute). If the node is fully dissolved, `_xmkey` is lost too → falls back to
Option A's orphan handling. Largest blast radius (reorders id resolution for every
dual); most "correct by construction" but most invasive.

## 5. Recommendation — phased

1. **Phase 0** grounding (above) — mandatory before B/C.
2. **Ship Option A first** as the safety net: it converts the whole residual
   `expected:id` cluster from **error → at-worst-degraded-content**, is additive to
   LOSTNODES, and is testable in isolation. Gate the "unlink vs absent-token"
   choice on whether the dual's content arity must be preserved (prefer the
   `meaning="absent"` token to keep XMApp arity, matching how empty args already
   render). Add an `Info:cleanup:xmref` count so silent drops are visible (canvas
   signal-integrity rule).
3. **Then Option B per Phase-0 finding** to recover *fidelity* on the
   highest-volume construct (record the real successor id so C redirects instead of
   dropping). Option C only if B proves whack-a-mole across many constructs.

## 6. Test plan

- **Unit/regression:** the 4 witnesses must go **error-free** (Option A) and, for
  Option B, the recovered arg must render (diff the affected `<XMApp>` content vs
  Perl's intermediate XML — `latexml … --noparse`? no: plain `latexml` emits the
  intermediate; compare `text=`/content args).
- **Guardrails:** full `cargo test --tests --no-fail-fast` (1390/0) — math parser
  changes are high-blast-radius; **clean-build** (`cargo clean`) per
  [[feedback_clean_rebuild_validation]]. Watch `parse/count_parses.xml` and every
  `*/math*.xml`, `ams/*.xml`, `parse/*.xml` for unintended structure changes.
- **Corpus:** re-run a sample of the `expected:id` canvas papers (the witnesses +
  ~15 more `Cannot find a node` papers) Rust-vs-Perl with the reliable harness
  ([[feedback_differential_perl_runner_rigor]]); confirm error count drops and no
  new regressions. Use `--release` for any timing claims
  ([[feedback_timeout_release_only]]).

## 7. Risks

- **Over-unlink (Option A):** dropping a *legitimately-resolvable-later* idref. Mitigate:
  the sweep runs only at the very end (after C), and only when `lookup_id` is None at
  that final point — there is no "later" after that.
- **Math-corpus regression (B/C):** id reordering / redirect changes XMDual content
  across many docs. Mitigate: clean-build full suite + corpus differential before
  commit; land A and B/C as separate commits so A's safety net can ship even if
  B/C is reverted.
- **Faithfulness:** Option A is *recovery*, not *parity* (Perl keeps the arg). Frame
  A as the stopgap and B as the parity goal; record the divergence in
  `OXIDIZED_DESIGN.md` if A ships standalone.
