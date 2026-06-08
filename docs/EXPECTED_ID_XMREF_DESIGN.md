# `expected:id` dangling-XMRef — Design Scope

> **Status:** DESIGN + PHASE-0 GROUNDED (no fix code yet). Scopes the fix for
> the residual `expected:id Cannot find a node with xml:id='…'` cluster — the
> non-VERTBAR XMDual-dangling remainder after the bra-ket VERTBAR work
> (`50c8a6a35e`, `275d249acc`) landed. **Phase 0 was run (2026-06-08) and
> changed the root-cause picture — see §3.** **Witnesses:** `2311.01600` (App.
> eq. E66 — *fully traced*), `2207.08945` (E49), `2306.04445` (alg2.m13),
> `2307.02913` (Ex17). All **Perl = 0 errors** (Rust-only). Related history:
> [[project_xmref_dangling_split]], [[project_dirac_balanced_delimiter_prune]],
> `MATH_PARSER_AND_ASF.md` (ASF + LOSTNODES).

## 1. Symptom

Post-processing (`latexml_post` `realize_xm_node` / `document.rs:1261`) raises
`Error:expected:id Cannot find a node with xml:id='…'`; the core math stage
already warns the same at `parser.rs:2644` (`realize_xmnode`) and
`util.rs:411` (`create_xmrefs`). The idref lives in an **XMDual content
branch** — typically a function-macro application such as `\Pr(…)`:

```
<XMApp><XMTok meaning="probability"/><XMRef idref="A1.E66.m1.1a"/></XMApp>
```

Conversion completes (rc=0) — the error is logged but output is produced. Each
affected paper emits 2–12 such errors.

## 2. The original hypothesis (now PARTLY DISPROVED)

The first draft of this doc blamed the **math-parser LOSTNODES pre-snapshot
gap**: a parse-time-created-then-lost id (`…m1.1a`) escaping the orphan
snapshot at `parser.rs:1383`. Phase 0 (§3) shows that mechanism is **real but
is NOT what strands the refs in the fully-traced witness**. Keep this section
as the description of **Class A** (below); it is the correct story for the
already-fixed VERTBAR/bra-ket family, not for the dominant remainder.

### The id-management machinery (3 layers — Class A context)

| Layer | Where | Mechanism |
|---|---|---|
| **A. `createXMRefs`** | `base_xmath.rs` `ltx:XMDual` `after_close_late` | For `_xmkey`-tagged nodes: `generate_id` the target, set the XMRef `idref` to the target's actual id. |
| **B. `resolve_xmkeys`** | `parser.rs:2662` (post-parse, per XMath, scoped to the mathnode) | Same for parser-generated `_pxmkey`/`_xmkey`. |
| **C. LOSTNODES** | `parser.rs:711` cleanup + `:1408` record | After a math's tree-replacement, any **pre-snapshot** id that no longer resolves is recorded `lost→__LOSTNODE__`; the cleanup rewrites or unlinks `//ltx:XMRef[@idref]`. Mirrors Perl `MathParser.pm` L287-298 — **but only the `ReplacedBy(lost, undef)` half** (drop), never the `ReplacedBy(lost, keep)` half (redirect). |

## 3. Phase 0 RESULTS — two distinct classes

Phase 0 fully traced witness **`2311.01600`** (instrumented `set_attribute`
for the id lifecycle + `rearrange_lone_ams_aligned`; instrumentation reverted,
tree clean). It splits the cluster into **two unrelated root causes.**

### Class A — math-parser within-parse absorption *(the original story)*

A node is **absorbed into another during the parse** (n-ary flatten, operator
concatenation), its id is lost, and the XMRef is **dropped** rather than
**redirected**. This is the VERTBAR/bra-ket family, already fixed
([[project_dirac_balanced_delimiter_prune]]). The residual here is a
**faithfulness gap**: Perl's `ReplacedBy` records the **successor** and the
top-level loop redirects (transitively); Rust records only `__LOSTNODE__` and
**drops**. See §4 *Option B* for the faithful fix. **No confirmed non-VERTBAR
witness is currently in this class** — classify each remaining witness in its
own Phase 0 before assuming it is.

### Class B — document-builder equation→equationgroup refnum-id loss *(DOMINANT; 2311.01600)*

**This is not a math-parser bug at all.** The source is the classic
lone-aligned equation:

```latex
\begin{equation} \label{eq:statedefs}
  \begin{aligned}
    \rho^{(k)}_{…} &= \sum_{s_A,s_B} \Pr(s_A,s_B | …) \\ …
  \end{aligned}
\end{equation}
```

Traced lifecycle (Rust):

1. **Building:** `\begin{equation}` opens `<ltx:equation xml:id="A1.E66">`
   (equation-refnum id, prefix `E`, matches Perl). Children get
   `<Math A1.E66.m1>`, `<XMArg A1.E66.m1.1 … .8>`. The `\Pr` XMDual content
   `XMRef`s are minted against this scheme (`A1.E66.m1.1` …).
2. **The equation's `xml:id` is absent by the time its `afterClose` fires**
   (confirmed: assigned `A1.E66` at open, yet every one of the 42
   `rearrange_lone_ams_aligned` calls in the doc reads `eq_id_before == ""`,
   including this equation — the E66-filtered probe never fires at rearrange
   time). Whether this is an explicit *remove* or a *deferred (late)
   reassignment* of the refnum id is the one open question for Phase-0b; the
   *consequence* — no id at the rename — is certain.
3. **`afterClose` → `rearrange_lone_ams_aligned`** (faithful port of Perl
   `rearrangeLoneAMSAligned`, amsmath.sty.ltxml:632) renames the equation →
   `<ltx:equationgroup>` and emits a MathFork — **but with an empty id**, so
   `modify_id("{eq_id}X")` never runs and the group is left id-less.
4. **`after_close_late` / `finalize`** then hit the generic
   *labeled-node-without-id* fallback (`latex_constructs.rs:7740`,
   `Tag!("ltx:*", after_close_late)` → `generate_id(node, "")`) and the
   `finalize_rec` id pass, giving the group a **paragraph-derived** id
   `A1.p10.1` and the inner equation `A1.p10.1.1`, math nodes `A1.p10.1.m*`.
5. The `\Pr` `XMRef`s still carry `A1.E66.m1.1a/1b/4a/4b` (the `a`/`b` are
   `modify_id` collision suffixes added when the refs were cloned during the
   math-parse reinstall) → **no node ever carries those ids** → `expected:id`.

**Perl ground truth** (same paper, 0 errors): the equation's refnum id is
**stable**, so `rearrangeLoneAMSAligned`'s `renameNode($equation,
'ltx:equationgroup')` **preserves `xml:id="A1.E66"`** on the group, the inner
equation becomes `A1.E66X`, and every intra-math `\Pr` ref resolves. Both
engines run the same MathFork rearrange path (verified: `MathFork` present in
both outputs). **The only difference is the container id**: Perl `A1.E66`, Rust
`A1.p10.1`.

**The refs themselves are already correct — Rust mints the *same scheme* as
Perl.** Perl's `\Pr` content ref is `idref="A1.E66.m1.1.1.…"` and resolves
(the `A1.E66.m1.*` target nodes exist because the group kept `A1.E66`). Rust's
refs are `A1.E66.m1.1a/1b/…` — the **same `A1.E66.m1` prefix** — but the target
nodes were re-homed under `A1.p10.1`, so they don't exist. This is the cleanest
possible confirmation that the divergence is **purely the container id**, not
the ref-construction logic. (One residual nuance for Phase-0b: Perl refs the
*parsed sub-node* — a deep dotted id — while Rust refs the *pre-parse XMArg* —
a shallow `…1a` collision id; B-fix must confirm those XMArg ids survive the
math-parse reinstall once the container id is stable, else a Class-A redirect
is also needed.)

**Root cause (Class B):** Rust **does not keep the equation's refnum id stable**
through the `afterClose` window (it removes or late-defers it — Phase-0b
pins which), so the id is absent where the equation→equationgroup rename would
carry it forward. Normal (non-rearranged)
numbered equations are unaffected — `A1.E53 … A1.E64` survive in the same
document — because there is no rename competing with the late id pass; only the
lone-aligned rearrange reads the id mid-window.

## 4. Design options

### For Class B (dominant) — faithful: keep the equation refnum id stable

The fix lives in the **document builder**, not the math parser. Direction
(pin the exact removal site first — see §5 Phase-0b):

* **B-fix (faithful):** the equation's refnum `xml:id` must be present and
  stable from open through `afterClose`, so `rearrange_lone_ams_aligned`'s
  rename carries it onto the `equationgroup` (then inner equations get
  `{id}X`, matching Perl). Concretely: find and remove the
  *remove-then-reassign-late* of the equation id (step 2 above), or — if the
  id is genuinely provisional until the number is final — have
  `rearrange_lone_ams_aligned` **re-derive and set the refnum id on the group
  before renaming** (mirror Perl, where the id is simply already there). This
  recovers full content parity (refs resolve to the right node).
* Once the group keeps `A1.E66`, the intra-math `\Pr` refs (`A1.E66.m1.*`)
  resolve with no math-parser change at all.

### Option B (Class A) — faithful port of Perl `ReplacedBy` via Meta-carried redirect

*This is the genuinely faithful fix for the math-parser absorption class.* It
does **not** help Class B.

**What Perl does.** `ReplacedBy(lost, keep)` (MathParser.pm:1562) records
`LOSTNODES{lostid} = keepid` — a **successor**, never a drop — at each
absorption site (`CatSymbols` L1167, `LeftRec` L1488, `ApplyNary` L1511),
recursing into equivalent subtrees when `isdup` (L1574-1586). The top-level
loop (L287-298) then **redirects** every `XMRef[@idref=lost] → keep`,
transitively (`while (my $reprepid = …)`). A drop is the *defect* path
(`Warn "LOST $lostid but no replacement!"`), not the norm. **Rust currently
records only `__LOSTNODE__` and drops — strictly less faithful even on the easy
case.**

**Why the naive port fails under ASF.** Recording `ReplacedBy` at the
absorption action (`infix_apply_nary` semantics.rs:1242, `cat_symbols`
semantics.rs:4389) would fire for **every candidate in the ASF
Cartesian-product**, including discarded parses, polluting LOSTNODES (this is
exactly why the port deferred to the post-commit snapshot-diff — see the note
at semantics.rs:1231-1241). The snapshot-diff, lacking Perl's parallel
old/new `isdup` walk, can only express "id vanished" (drop), not "absorbed
into Y" (redirect).

**The faithful, ASF-safe mechanism — carry the redirect in the chosen tree,
harvest at commit:**

1. **Add a field to `Meta`** (`semantics/metadata.rs`), e.g.
   `absorbed_ids: Vec<Rc<str>>`. `Meta` already rides per-node parse state
   (`syntax_trace`, `fenced`, curry, `bumplevel`) into the committed tree, and
   its `PartialEq` is a no-op (metadata.rs:28-32) so an added field won't
   perturb ASF tree-dedup.
2. **At each absorption action**, when an id-bearing operand/operator is
   discarded, capture its **source-node xml:id** (Lexeme → `lookup_lex_node`;
   Token/Apply → `XProps.id`) and push it onto the **kept** node's
   `Meta.absorbed_ids`. For `isdup`-style flattening, recurse the
   structurally-corresponding subtrees as Perl does (the subtle part — verify
   in Phase-0a).
3. **At commit** — after `into_xmath` + `append_tree` + `resolve_xmkeys`
   assign the final ids (`parser.rs:~1400`) but **before** the LOSTNODES
   cleanup (`parser.rs:711`) — walk the committed tree; for each node with
   `absorbed_ids`, call `record_replacement(absorbed_id, node_final_id)`. Only
   the chosen tree is materialized, so **no discarded-parse pollution**.
4. The existing cleanup then **redirects** the XMRefs to a real successor
   (Perl behavior) instead of dropping (current behavior).

This literally reproduces Perl's per-site `ReplacedBy` + transitive redirect,
commit-scoped to dodge the ASF re-firing. Risk: touches hot semantic actions +
the commit path + `Meta`; high blast radius → clean-build full suite + corpus
differential mandatory.

### Option A — generic unresolved-idref reconciliation (safety net; NOT parity)

*Papers over both classes; faithful to neither.* Extend Layer C: after the
LOSTNODES cleanup, sweep every `//ltx:XMRef[@idref]` whose idref does not
`lookup_id` and either unlink it or replace with `<XMTok meaning="absent"/>`
(preserving XMApp arity). This converts the whole `expected:id` cluster from
**error → at-worst-degraded-content** at low, isolated risk. It is what the
current Rust LOSTNODES path *already does* for pre-snapshot orphans, extended
to the ones the snapshot misses. **Cost:** the dropped arg renders "absent"
vs Perl's correct content — for Class B that means the `\Pr` argument silently
disappears, which is worse than the targeted B-fix. Add an
`Info:cleanup:xmref` count (signal-integrity rule) so silent drops are visible.

### Option C — defer all idref assignment to one post-parse pass

Keep `_xmkey` symbolic through all parsing + rewriting; resolve once at the
end. Largest blast radius; only consider if Option B becomes whack-a-mole
across constructs. Does **not** address Class B (whose ids are document-builder
ids, not math `_xmkey`s).

## 5. Recommendation — phased, by class

1. **Class B first (the dominant, fully-grounded win).** Phase-0b: pin the
   exact site that removes the equation's refnum id between open and
   `afterClose` (instrument id removal / the provisional-id reassignment).
   Then apply **B-fix**: keep the refnum id stable so
   `rearrange_lone_ams_aligned` carries it to the equationgroup (Perl parity,
   full content fidelity). Expected to clear `2311.01600` and sibling
   lone-aligned-equation papers — **pending the Phase-0b check** that the
   pre-parse XMArg ids survive the math-parse reinstall once the container id
   is stable (if not, pair with a Class-A redirect for the shallow→deep ref
   gap).
2. **Re-classify the other witnesses** (`2207.08945`, `2306.04445`,
   `2307.02913`) with the same id-lifecycle trace. Algorithm/example witnesses
   may be a *third* container class (algorithm-line / example id schemes) —
   do not assume B-fix covers them without tracing.
3. **Option B (Class A) only for genuine math-parser absorption residual.**
   Faithful, but currently has **no confirmed non-VERTBAR witness** — defer
   until one is classified into Class A.
4. **Option A as a last-resort safety net**, gated behind an
   `Info:cleanup:xmref` count, if a residual proves un-rootcausable in a
   reasonable budget. Frame as recovery, not parity; record in
   `OXIDIZED_DESIGN.md` if it ships.

## 6. Test plan

- **Class B:** `2311.01600` → **0 `expected:id`**; the equationgroup tagged
  `(66)` gets `xml:id="A1.E66"` (diff vs Perl `perl_out.xml`); the four `\Pr`
  refs resolve. Watch the existing equation/equationgroup id fixtures
  (`tests/**/eq*.xml`, `ams/*.xml`, anything with `equationgroup`).
- **Option B (Class A):** the recovered arg must render (diff the affected
  `<XMApp>` content vs Perl's intermediate XML); `parse/count_parses.xml` and
  every `*/math*.xml` unchanged.
- **Guardrails:** full `cargo test --tests --no-fail-fast` (1390/0) on a
  **clean build** (`cargo clean`) per [[feedback_clean_rebuild_validation]] —
  both areas are high-blast-radius.
- **Corpus:** re-run the witnesses + ~15 more `Cannot find a node` papers
  Rust-vs-Perl with the reliable harness
  ([[feedback_differential_perl_runner_rigor]]); confirm the error count drops
  with no new regressions. `--release` for any timing claim
  ([[feedback_timeout_release_only]]).

## 7. Risks

- **Class B id stabilization** could shift equation/equationgroup ids
  document-wide (every lone-aligned equation). Mitigate: diff a sample of
  equationgroup ids vs Perl before/after; land as its own commit.
- **Option B (Class A)** id reordering / redirect changes XMDual content across
  many docs → clean-build full suite + corpus differential before commit; land
  separately from any safety net so each can be reverted independently.
- **Option A** over-unlink of a legitimately-resolvable idref — mitigated by
  running only at the very end, after C, when `lookup_id` is None with no
  "later" to resolve in. But it is *recovery, not parity* — prefer the targeted
  per-class fixes.
- **Faithfulness framing:** B-fix (Class B) and Option B (Class A) are parity;
  Option A is a stopgap. Do not let the stopgap mask an un-fixed class —
  the `Info:cleanup:xmref` count is the tripwire.
