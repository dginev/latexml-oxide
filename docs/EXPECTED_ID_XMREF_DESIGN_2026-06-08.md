# `expected:id` dangling-XMRef — Design Scope

> **★ 2026-06-26v — LANDED: the "Missing idref" keyless-ref bug is a
> distribute-dual extend interaction; partial fix for the #1 cluster.** Root nailed
> + fixed (`class-b-xmref`): `distribute_list_relation` builds a dual with
> `meaning="formulae"` content but a RELATION-`Apply` presentation (`(a,b)=c`), not
> an `XMWrap`. When a further `\quad`/comma formula extends that dual, the
> `formulae_apply` / `list_apply_core` extend paths pushed a content ref but only
> updated the presentation `if let XM::Wrap(items)=pres` — which SILENTLY FAILS on
> the relation-Apply, stranding the ref as a keyless bare `<XMRef/>` →
> `Missing idref` (document.rs:3238). Minimal repro: `a,\quad b=c,\quad d=e` →
> `formulae@(a=c, b=c, [])` (the `[]` is the bare ref). **FIX:** gate both extend
> paths on `matches!(**pres, XM::Wrap(..))`; a non-Wrap (distribute) left falls
> through to `list_or_formulae_create`, which builds a fresh dual whose refs ALL
> resolve (output nests: `formulae@(formulae@(a=c,b=c), d=e)` — no bare ref,
> content preserved, schema-valid). Validated: minimal repros 1→0 bare; witnesses
> `0704.2334`/`0705.0790`/`0707.1173` Missing-idref 2/1/1 → **0**; full suite
> **1470/0**, clippy clean; regression `cluster_formulae_distribute_no_bare_ref`.
> **PARTIAL:** `0707.1339` still emits 2 — a DIFFERENT "Missing idref" sub-cause
> (not the distribute-extend path); the cluster has multiple roots, this closes the
> common distribute-extend one. Next: trace `0707.1339`'s remaining bare-ref origin.
> **COVERAGE MEASURED:** of a 16-paper sample of the 370 cortex `expected/id`
> papers (10 with usable local sources), **9 → 0 Missing-idref, 1 still**
> (`0707.1339`) → **≈90% of the cluster resolved by this single fix** (sample
> estimate; a fresh cortex rerun gives the exact number). The distribute-extend
> path was the DOMINANT root of the #1 `expected/id` cluster.
> **COVERAGE FIRMED (2 samples, 37 papers w/ usable sources): 36 → 0 Missing-idref,
> 1 still (`0707.1339`) ≈ 97%.** Safety re-checked: spot-checked fixed papers are
> fully clean (0 errors, 0 Missing-idref); the few errors in the broader sample are
> concentrated/background (undefined macros/env), NOT fix-induced (full suite
> 1471/0 confirms no regression). So the single distribute-extend guard resolves
> ~97% of the 370-paper #1 `expected/id` cluster, safely.
> **Remaining tail characterized:** `0707.1339`'s sub-cause is NICHE — minimal
> repro `{}^{++}_{bkg}` (EMPTY base `{}` + `++` operator-list superscript + a
> subscript) → Rust 2 bare refs / Perl 0; the simpler variants (`{}^{++}`,
> `x^{++}`, `{}^{+}_{b}`) are clean in BOTH, so it needs the full empty-base +
> operator-list-script + subscript combination. It builds a `list@(+,+)` with bare
> content refs (relates to the open "N-ary bare-operator listing" math-aside).
> Low-value tail (rare construct); deferred. The high-value root (distribute-extend,
> ~90%) is FIXED; the remaining ~10% is a handful of niche script/operator-list
> sub-causes like this, for the deferred math-fork session.
> **Structure (2026-06-26w):** `{}^{++}_{bkg}` →
> `apply(SUBSCRIPTOP, apply(SUPERSCRIPTOP, absent, DUAL), b*k*g)` where the `++`
> superscript DUAL is `list@(<XMRef/>, <XMRef/>)` / presentation `XMWrap[+, PUNCT, +]`
> — i.e. the two `+` ADDOPs parsed as a bare-operator `list@` whose content refs
> never got keyed. **This is the OPEN "N-ary bare-operator listing" math-aside**
> (SYNC_STATUS: `+ - \times \div` → `list@(+,-,*,/)`; ambiguity-sensitive, PARKED).
> So the `expected/id` niche tail ≡ that parked aside. It is SAFE today (the
> operand-protection guard keeps the bare refs → faithful Warn, no corruption,
> schema-valid). Fixing it = keying bare-operator-list refs in the N-ary aside work
> — low value (rare), ambiguity-risky; leave parked.

> **★ 2026-06-26u — CORPUS SPLIT MEASURED: the 10k `expected/id` cluster is 100%
> "Missing idref" (keyless), 0% "No node found" (dangling).** Classified all 884
> cortex `warning/expected/id` messages (370 papers): **884 "Missing idref", 0 "No
> node found".** Consequences for prioritization:
> * The 2026-06-22 framing ("1005 tasks, *Cannot find a node*, the #1 divergence")
>   was the PARSE-TIME warning (`parser.rs:2840`), SUPPRESSED 2026-06-25. The
>   remaining POST-processing cluster is the faithful `Missing idref` (document.rs
>   :3238, keyless), NOT `No node found` (:3247, dangling).
> * The landed comma-list-conditional fix (2026-06-26q) addresses the DANGLING
>   class (the design-doc Class-B witness `2311.01600`, which has post-proc
>   danglers) — it is CORRECT & faithful (witness 4→0, Perl-exact, fixes the
>   standalone `a,b|c` aside) but the dangling class is **~0 in the 10k corpus**, so
>   its 10k impact is negligible. (2311.01600 is evidently not in the shuffle, or
>   its construct is rare.) Still a legitimate fix — other corpora/papers hit it.
> * **The genuine #1 remaining `expected/id` divergence is the "Missing idref"
>   keyless class (370 papers / 3.7% of the 10k)** — reframed UP from "low-priority
>   polish": low-severity per paper (faithful Warn, no corruption, schema-valid) but
>   HIGH breadth. Root = `formulae_apply` content ref whose key never reaches the
>   presentation item's top (26t). This is now the top math-fork target.

> **★ 2026-06-26r — the DOMINANT residual `expected/id` cluster is a DISTINCT
> class: "Missing idref" (bare keyless XMRef), NOT Class B's "No node found"
> (dangling idref).** Queried the live cortex `warning/expected/id` report (pre-fix
> 10k run): **370 tasks / 884 msgs**, and they are overwhelmingly
> `Missing idref on ltx:XMRef … _xmkey is `` ` (document.rs:3238 — the XMRef has
> NEITHER an idref NOR a key), NOT the `No node found with id=…` (document.rs:3247,
> dangling idref) that Class B / the comma-list-conditional fix (2026-06-26q)
> addresses. Confirmed on `0704.2334` (Rust 2 bare `<XMRef/>`, **Perl 0** — Rust-
> only) and `0705.0790`/`0707.1173` (still emit it with the FIXED binary → separate
> root). The output shape is `apply(op, apply(…), <XMRef/>)` — an XMApp whose last
> operand is a keyless XMRef. The triggering source: `\quad`/`\;`-separated
> **formulae/lists** with function-fence applies, e.g.
> `0,\quad\mathrm{pgh}\left(\eta_\mu\right)=1,\quad\mathrm{pgh}(\Phi)=\mathrm{pgh}(\eta)=0`
> and `…,\;V^{…}=V^{…},\;…`. **CONTEXT-DEPENDENT** (like Class B's `\appendix`): the
> formula extracted standalone does NOT reproduce the bare XMRef — needs the full
> paper context. So the dominant residual is the **`create_xmrefs`/`formulae_apply`
> key path emitting a keyless content ref** (the key is empty/lost so it never
> resolves to an idref) — a separate dedicated investigation (same methodology as
> Class B: find the context that drops the key, then the targeted fix). The
> 2026-06-26q comma-list fix correctly closed the "No node found"/dangling sub-case
> (the canonical witness); this "Missing idref"/keyless class is the next target.
>
> **ROOT PIN (2026-06-26s) — key MISALIGNMENT in the formulae/list dual, resolved
> by the base_xmath build-time resolver.** Traced on `0704.2334` (reverted, clean):
> * `XM::Ref` materialization (tree.rs:2058) emits **0** bare refs — so it is NOT
>   `into_xmath`/`create_xmrefs` (rules out the earlier 26r "clone arm" hypothesis).
> * The bare refs come from the **Base_XMath `Tag!("ltx:XMDual", after_close_late)`
>   resolver** (base_xmath.rs:502-513): a content `XMRef` has `_xmkey="XM291"` but
>   the dual's collected targets are keyed `["XM287","XM288"]` — the content ref's
>   key has **NO matching presentation target**, so `ids.get(key)` misses, idref
>   stays unset, and L513 removes the now-unresolved `_xmkey` → BARE ref →
>   `Missing idref … _xmkey is `` ` at finalize. (`resolve_xmkeys` leaves 0 bare —
>   it's purely the base_xmath dual resolver.) The keys are close-numbered
>   (287/288 vs 291) ⇒ a RE-KEYING: the content ref and its presentation target
>   were assigned DIFFERENT `get_xmarg_id()` keys instead of a shared one.
> * **Narrowed further (2026-06-26s):** `list_or_formulae_create` (semantics.rs:680)
>   AND the `list_apply_core` extend path both align keys CORRECTLY in isolation —
>   `create_xmrefs(&mut [&mut left,&mut right])` MUTATES each arg's `props.xmkey`
>   AND returns a ref with the SAME key, so content-ref key == presentation-arg key.
>   Therefore the double-keying is UPSTREAM of these: a CALLER that clones or
>   re-keys an already-keyed item between the content-ref creation and the
>   presentation insertion, so the content ref (XM291) and its presentation target
>   (re-keyed to XM287/288) diverge. Close-numbered keys ⇒ two `get_xmarg_id` calls
>   for the same logical item.
> * **STRUCTURE captured (2026-06-26t).** Dumped the offending dual: it is a
>   `formulae@` dual (`formulae_apply`). Content arm =
>   `apply(formulae, apply(=, refs…), apply(=, refs…), <XMRef _xmkey="XM291"/>)`;
>   the first refs (`S2.E11.m3.1..4`) RESOLVE, only the trailing `XM291` is bare.
>   The presentation arm carries `_xmkey` only on NESTED nodes (XM287 = the `η_μ`
>   subscript-app, XM288 = the `pgh(...)` app) — **no presentation node's TOP has
>   `_xmkey="XM291"`**. So `formulae_apply`'s extend path
>   (`create_xmrefs(&mut [&mut right])` mutates `right`'s key + returns a matching
>   ref, then pushes `right` to presentation) produced a content ref keyed XM291
>   whose presentation item's TOP node does NOT carry XM291 — the key mutation on
>   the `right` XM did not reach the presentation item's top on materialization
>   (a propagation/clone gap, or `right`'s top is an XM variant that drops `_xmkey`).
> * **Next session entry point:** in `formulae_apply`/`list_apply_core` extend
>   paths, after `create_xmrefs(&mut [&mut right])`, verify `right`'s TOP carries the
>   same `_xmkey` the returned ref got, and that it survives `into_xmath`
>   materialization (compare to `list_or_formulae_create`, which keys both arms in
>   one call and works). NOTE the extend path does NOT clone `right`, so the bug is
>   subtler than a single site — likely a nested-relation ref or a build-time
>   `\lx@dual` interacting with the formulae content arm. Trace each formulae_apply
>   call on `0704.2334` to catch the exact XM291-producing call. Fix = ensure the
>   presentation item's top keeps the content ref's key. Validate 2→0, Perl 0, +
>   math fixtures.
>
> **SEVERITY REFRAME (2026-06-26t):** this "Missing idref" residual is a content-
> MathML **QUALITY gap, NOT corruption/validity.** The bare `<XMRef/>` has no idref,
> so the broad prune sweep SKIPS it (nothing to match) → it survives to output and
> the FAITHFUL `Missing idref` Warn fires (Perl Document.pm:1548). No content is
> dropped and the XML is schema-valid; only the operand's content-↔-presentation
> link is degraded (Rust emits a Warn Perl doesn't). So it is lower-priority than
> the (now-fixed) Class-B dangling/corruption classes — a cMML-quality polish item
> for the deferred dedicated math-fork session, not a release blocker. The landed
> comma-list fix (dangling) + operand guard (corruption) closed the two
> higher-severity sub-classes; this keyless-quality class is the remaining tail.

> **★ 2026-06-26q — CANONICAL CLASS-B WITNESS FIXED AT THE ROOT (LANDED, axis A).**
> The faithful fix landed (`class-b-xmref`, commit "comma-list-LHS conditional"):
> grammar rule `statements punct statement vertbar statements =>
> vertbar_modifier_listlhs` (+ extracted `list_apply_core` to bypass list_apply's
> Rule-4 for the legitimate bar-after-comma item). A comma-list left of a
> conditional bar now PARSES — `a,b|c` → `list@(a, conditional@(b,c))` (Perl-exact,
> `|` binds the LAST item). The Class-B witness's aligned `\Pr(s_A,s_B|\Omega)`
> argument therefore parses, the native id-transfer runs, the dual content refs
> **RESOLVE** and the dual is **PRESERVED** — the genuinely-correct tree. cb_repro
> 4 danglers → **0**; full original witness `2311.01600` → **0**. Validated vs Perl
> (exact): `a,b|c`, `a,b,c|d`, `x|y,z`, abs-value `a|a|+b|b|+c|c|`. Full suite
> **1469/0**, clippy clean, regression tests `cluster_comma_list_conditional` +
> `cluster_xmref_pr_arg_not_dropped`. Also fixes the open standalone-`a,b|c` VERTBAR
> math-aside. **RESIDUAL:** this fixes the `a,b|c` arg-parse gap (the canonical
> witness + that pattern). The broader `warning/expected/id` cluster (~1005 cortex
> tasks) may include OTHER unparseable-arg shapes that strand content refs the same
> way — each is the same axis-A pattern (find the arg-parse gap, add the targeted
> grammar rule). The landed operand-protection guard (2026-06-26o) remains the
> general safety net (content-loss → honest dangling) for any residual. The
> FENCED `(a,b|c)` conditional (`P(a,b|c)`) is a separate pre-existing divergence.
> A fresh cortex rerun would quantify residual `expected:id` after this fix.

> **UPDATE 2026-06-26 — Class B re-grounded for the dedicated attempt (branch
> `class-b-xmref`); FIX NOT YET WRITTEN.** Picked this up under explicit sign-off
> (the fresh 10k re-mine flagged it the #1 remaining genuine Rust-only divergence:
> 0803.3810 = Rust 51 vs Perl 0 same-host). Findings that change how to approach it:
> * **The cluster is MULTI-ROOT — do NOT conflate witnesses.** `0803.3810` (the
>   WORST by warning count, 51) is **NOT** Class B: it has **zero** align-family
>   environments; its `\Pr`/array refs come from `\left(\begin{array}…\right)` +
>   `\frac` in plain `equation`s — a *different* (Class-A-ish / general
>   createXMRefs) resolution gap. The design-doc Class B (lone-aligned `\Pr`) is
>   `2311.01600` (now at `/data/arxiv/2311/2311.01600.zip`), which still reproduces
>   the **exact 6 documented danglers** (`A1.E66.m1.{1a,1b,4a,4b}`,
>   `A1.E68.m1.{1a,1b}`) on current HEAD.
> * **The bug is CONTEXT-DEPENDENT, not construct-local.** The same `eq:statedefs`
>   equation, extracted standalone (becomes `S0.E1`), resolves with **0 danglers**;
>   it only dangles deep in the document. The `1a/1b` are `modify_id` **collision**
>   suffixes (step 5 below) — they only arise when the cloned content's ids collide
>   with pre-existing ids, i.e. need a multi-equation document + the appendix id
>   scheme. So a clean construct-only minimal fixture does NOT exist.
> * **Faster repro (the iteration unlock, ~15 s vs the ~4 min full paper):**
>   `head -150 adaptivepaper.tex` (the full preamble — a minimal preamble does NOT
>   trigger it, so something in those macros is load-bearing) + a `\section` with a
>   few `\begin{equation}`s + `\appendix` + the `eq:statedefs` block (lines
>   1196–1206) → **4 `No node found` danglers**. Use this to iterate the fix.
> * **The warning IS faithful.** `document.rs:3238` ("Missing idref on ltx:XMRef",
>   no idref) and `:3247` ("No node found with id", dangling idref) both mirror Perl
>   `Document.pm:1548/1553` (Warnings). So Rust hitting them while Perl doesn't =
>   genuinely incomplete `_xmkey`→idref resolution (degraded content MathML), not
>   noise. The final HTML has 0 `<XMRef>` (consumed into MathML) so the damage is in
>   content-MathML quality, invisible to a plain idref-dangling grep — diff MathML.
> * **Fix remains the design-doc multi-part structural change** (single
>   group-content-Math: main Math id from GROUP not inner-X equation + MOVE not
>   clone + parse-reinstall id preservation) across `open_math_fork`/
>   `rearrange_lone_ams_aligned`/parser-reinstall — the "~full day, 4 files, 3 prior
>   reverts" effort. Groundwork done (repro + root re-confirmed, branch
>   `class-b-xmref` clean); the structural change itself is the next dedicated sitting.
>
> **2026-06-26b — Perl's exact target tree CAPTURED + the fix decomposed into A/B,
> Part A implemented-tested-REVERTED.** On the ~15s repro, dumped Perl's structure:
> the equationgroup's per-row `equation`s have **per-row main Math *elements***
> (`A1.E4X.m1`, `A1.E4Xa.m1`, …) — same as Rust — BUT their content *children* keep
> the original **group scheme `A1.E4.m1.*`** (deep dotted, e.g. `A1.E4.m1.1.1`),
> with presentation clones suffixed `.mf` (`A1.E4.m1.1.mf`). So Perl does NOT build
> a literal "one group Math element"; it **moves** the parsed content (children keep
> their ids) and its reinstall **does not re-id existing-id nodes**.
> * **Part A (rearrange MOVE not clone)** — `amsmath_sty.rs` main append
>   `append_clone(stuff_children)` → `unlink_node()`+`add_child` move (Perl L671).
>   **Implemented, FULL SUITE GREEN (1468/0), but REVERTED.** Faithful, but
>   insufficient ALONE: danglers stay 4 (set shifts `1a/1b/4a/4b`→`1/1a/4/4a`) AND
>   it's behavioral churn (move vs clone changes content structure on real papers)
>   with no user-facing fix → not committed without Part B. (This reconfirms the
>   design doc's earlier move-revert, now with full-suite + mechanism evidence.)
> * **Part B (THE BLOCKER) = the math-parse reinstall re-ids the moved content.**
>   After the move, the parsed content STILL lands at per-row `A1.E4Xa.m1.*` (zero
>   `A1.E4.m1.*` survive). `document.generate_id` (document.rs:4565) correctly
>   PRESERVES existing ids, so the loss is NOT there — it's that the ASF/grammar
>   parse builds a **fresh** result tree (new leaf nodes, no ids) for the per-row
>   main `<Math A1.E4Xa.m1>`, so `generate_id` assigns fresh per-row ids; the
>   `\Pr` content XMRefs (minted `A1.E4.m1.*`) then strand. Perl's parse reuses the
>   input-lexeme nodes (keeping `A1.E4.m1.*`). Fixing this = make the math-parse
>   reinstall preserve the input-lexeme xml:ids on the parsed tree's leaves — deep
>   in the ASF→XMath reinstall (`parser.rs` ~1340-1369 install + the leaf-node id
>   derivation), the 3-revert sensitive area. **The dedicated sitting is Part A
>   (re-apply) + Part B together; validate the ~15s repro → 0 danglers + the
>   full equation/equationgroup/MathFork fixture set + `split.xml` vs Perl.**
>
> **2026-06-26c — full id-lifecycle TRACE (instrumented generate_id + the
> createXMRefs idref set; instrumentation reverted, branch clean).** Sequence on
> the repro: (1) Build: original `<Math A1.E4.m1>`; createXMRefs sets XMArg ids +
> XMRef idrefs all to `A1.E4.m1.1..8` (consistent). (2) Rearrange creates per-row
> mains `A1.E4X.m1`..`A1.E4Xd.m1` and CLONES content in (clone re-ids →
> `A1.E4.m1.1a` collisions). (3) Post math-parse re-ids each per-row main
> **INCONSISTENTLY**: some rows → `A1.E4X.m1.*` with freshly-rebuilt XMDuals whose
> refs ARE consistent (`xmref idref = A1.E4X.m1.7.1`, resolve); other rows →
> `A1.E4.m1.2b.*` (group scheme, b-suffixed). The danglers (`A1.E4.m1.1a/4a/…`)
> are **leftover cloned XMRefs from step 2 that the step-3 rebuild neither consumed
> nor redirected**. So the defect is the INTERACTION of clone-re-id (2) + a
> non-uniform post-parse rebuild (3), not a single id derivation. Faithful fix =
> Part A (move-not-clone) PLUS Part B (post-parse preserves moved leaf ids
> uniformly); the trace shows step-3's per-row inconsistency is the concrete thing
> to make uniform. Focused structural sitting; groundwork (trace + repro + Perl
> target) complete.
>
> **Part-B starting point for the next session.** The lexeme→input-node reuse
> infrastructure already exists (`semantics.rs` `lookup_lex_node`,
> `data.rs` `MATH_IDSTORE` frozen snapshot) — so leaves can carry input ids. The
> change is making the XMArg/XMRef **structural** nodes preserve their pre-rearrange
> ids through the result construction/reinstall (so the per-row main content keeps
> `{group}.m1.*`, uniformly across rows). This is central to the parser (every
> formula) → full `*/math*.xml` + equationgroup/MathFork fixture validation +
> corpus differential mandatory; land as its own revertible commit. Pair with Part A
> (move) and the Perl `ID_SUFFIX='.mf'` presentation-branch clone (amsmath.sty.ltxml
> L664) which Rust currently omits.
>
> **2026-06-26d — ALL peripheral approaches empirically EXHAUSTED.** Tested on the
> repro (each reverted, branch clean): move-alone → 4 danglers; **move + `.mf`
> presentation clone → 4 danglers** (`A1.E4.m1.1` + `A1.E4.m1.1.mf`). The post-parse
> re-ids the per-row main content to `A1.E4Xa.m1.*` *regardless* of clone/move/`.mf`,
> so the refs (any `A1.E4.m1.*` scheme) strand. **CONCLUSIVE: the ONLY fix is the
> core Part-B change** — the post-parse must assign the per-row main *content* ids in
> the original group `{group}.m1.*` scheme (continuing the `.1..8` numbering ACROSS
> the per-row main elements, since the original single `<Math {group}.m1>` spanned
> all rows). That cross-row content-id-scheme coordination in the math-parse result
> construction is the dedicated-session change; no drive-by (move/`.mf`/clone) helps.
>
> **Narrowing (2026-06-26e).** `create_xmrefs` (`util.rs:459`) already reuses input
> LEAF nodes via `lookup_lex_node` (preserving ids) — leaf preservation works. The
> gap is the **structural XMArg** (`\lx@xmarg`, the `\Pr` argument container): its id
> is set during BUILD by `createXMRefs` (→ `A1.E4.m1.1`) and the XMRef idref set to
> it; the post-parse then RE-IDS the XMArg to `A1.E4Xa.m1.N` without updating the ref
> → strand. Part-B's precise target: the post-parse must **preserve the
> rearrange-moved XMArg ids** (they already carry a build-time `{group}.m1.*` id),
> OR re-id refs in lockstep — still in the core post-parse id pass.
>
> **PIN (2026-06-26f) — exact code site located via sustained step-through.** The
> re-id is NOT in `append_tree` (document.rs:4649 — it RE-CREATES with the same
> xml:id, preserving) nor `generate_id` (preserves existing). It is the XM→Node
> materialization `XM::into_xmath` (`semantics/tree.rs`): the **`XM::Arg` arm
> (tree.rs:2076-2082)** does `Node::new("XMArg")` with **NO attributes/id**, and the
> `XM::Arg(Vec<XM>)` variant **carries no `props`/id** — so the input `\lx@xmarg`
> node's build-time `{group}.m1.N` id (set by `createXMRefs`, and the target of the
> `\Pr` content XMRefs) is dropped at materialization; the fresh XMArg then gets a
> per-row `{group}X.m1.*` id and the refs strand. (XM::Apply/Dual/Wrap/Tok DO carry
> ids via `props.into_attributes()`; only XM::Arg and XM::Ref use bare `Node::new`.)
> **So Part-B's minimal change is: make `XM::Arg` carry the input node's id
> (add an id/props field to the variant + populate it at parse-construction from the
> input XMArg, and set `xml:id` in the tree.rs:2077 materialization).** This is a
> central XM-enum change (ripples to every `XM::Arg` construction site) → full
> `*/math*.xml` + structural-diff-vs-Perl validation; but it is now a ONE-variant,
> ONE-site change, not an open-ended parse rewrite. Repro: the 15s recipe above
> (→ should reach 0 danglers, content ids `{group}.m1.*`).
>
> **ATTEMPTED + FAILED (2026-06-26g) — the pinned fix is necessary but NOT
> sufficient; the id is lost POST-materialization.** Implemented it end-to-end
> (reverted, branch clean): `XM::Arg(Vec<XM>)` → `XM::Arg(Vec<XM>, XProps)` (~19
> match sites + construction), captured the input `<XMArg>`'s `xml:id` at
> construction (`tree.rs` `XMArg` arm — `XProps::from` does NOT read xml:id, so set
> `props.id = n.get_attribute_ns("id", XML_NS)` explicitly), and set it at the
> `XM::Arg` materialization (`set_attribute(arg_node, "xml:id", id)`). Result on the
> repro: **STILL 4 danglers, ZERO `A1.E4.m1.*` content ids** — the materialized
> XMArg ends up at the per-row `{group}X.m1.*` scheme anyway. So the captured id
> does NOT persist: the `XM::Arg` node is created **detached** via `Node::new`
> (unlike `XM::Apply` which uses `open_element_at`), and a `set_attribute` xml:id on
> a detached node does not survive the subsequent attach/`append_tree`/`generate_id`
> flow → the node is re-id'd. **Next attempt must set the XMArg id AFTER it is
> attached to the tree (in the caller / via `open_element_at`-style creation), or
> ensure `append_tree`/`generate_id` see and preserve it.** So Part-B is: (1) XM::Arg
> carries the id [done-and-works structurally] + (2) make that id survive
> attach/append_tree/generate_id [the remaining unknown — a detached-node /
> id-recording-order issue]. Deeper than one site; the id-LIFECYCLE through
> attach+append_tree is the real remaining puzzle.
>
> **RULED OUT (2026-06-26h) — XM::Arg materialization is NOT the fix site (3
> variants all failed, reverted, branch clean).** Tried: (a) Node::new + set_attribute
> xml:id; (b) explicit `props.id = n.get_attribute_ns("id", XML_NS)` capture at the
> `XMArg` construction; (c) materialize via `open_element_at(owner,"ltx:XMArg",attrs)`
> (attached+recorded, with `into_attributes()` emitting the captured id — confirmed
> it emits xml:id at tree.rs into_attributes). **ALL three → still 4 danglers, 0
> `A1.E4.m1.*` ids.** So the id is lost BEFORE/ELSEWHERE than the XM::Arg path: either
> the input XMArg at parse time no longer carries `A1.E4.m1.*` (stripped by the
> rearrange clone/unrecord), or the dangler-targets aren't routed through
> `XM::from("XMArg")→XM::Arg` at all (likely the XMDual content arm / `create_xmrefs`
> path). **Next session's FIRST step must be a targeted runtime trace of the SPECIFIC
> dangler node `A1.E4.m1.1a`'s lifecycle** (where its xml:id is set, stripped, and
> re-assigned) — NOT another XM::Arg attempt. The pinned-site hypotheses are
> exhausted; the real site is in the dangler-target's own id lifecycle through the
> rearrange-clone + the XMDual content resolution.

> **2026-06-26i — FULL runtime trace done; mechanism nailed end-to-end; TWO more
> approaches RULED OUT; the real fix site is now PRECISE.** (Instrumented
> `XProps::from`, `generate_id` w/ backtrace, the XMDual resolver, `parse_rec`'s
> XMArg branch, and the reparse orphan-snapshot — all on the 15 s `cb_repro.tex`;
> every probe reverted, branch clean, baseline re-confirmed = 4 danglers
> `A1.E4.m1.{1a,1b,4a,4b}`.)
> * **The 4 danglers are the `\Pr` XMDual CONTENT-arm arg refs**
>   (`<XMApp><XMTok meaning="probability"/><XMRef idref="A1.E4.m1.1a"/></XMApp>`).
>   Their resolve target is the presentation arg, re-id'd to `A1.E4X.m3.5`.
>   **The arg material IS still present** in the baseline output (the ref merely
>   dangles) — so any fix that DROPS/PRUNES the ref is **content loss** (the `\Pr`
>   loses its argument in content MathML). RULED OUT as a cheat.
> * **The breaking id `A1.E4X.m3.5` is born in `createXMRefs`** — the Base_XMath
>   `Tag!("ltx:*", after_open_late)` `_xmkey` handler (`base_xmath.rs:477`) calls
>   `generate_id` while `into_xmath` (XM::Apply, `tree.rs:2026`) re-materializes
>   the presentation arm during the math parse (backtrace confirmed). `generate_id`
>   correctly PRESERVES existing ids — the node is FRESH (no id) at that point, so
>   it gets a per-row id; the content ref keeps the stale build/clone idref.
> * **The pre-built `\Pr` dual is NOT ingested via `From<&Node>`/`XProps::from`**
>   (ZERO trace hits for the E4 ids) — so "make `XProps::from` capture xml:id" is a
>   non-starter for this path (it's a different construction route). `get_attributes()`
>   DOES return xml:id under key `"id"` (document.rs:2051), but the dual never goes
>   through it.
> * **`_xmkey` re-resolution RULED OUT (empirically, still 4 danglers).** The
>   parser-side `resolve_xmkeys` (`parser.rs:2859`, called at `:1519`) strips ALL
>   `_xmkey` after matching; the re-materialized dual has **0 `_xmkey` at the
>   Base_XMath `after_close_late`**. Re-opening the Base_XMath resolver to
>   re-point refs-with-stale-idref (keep `_xmkey`, conservative `lookup_id`-gated
>   overwrite) was implemented + tested → **0 effect** (the markers aren't there
>   on the 2nd pass).
> * **`_xmkey` old→new REMAP RULED OUT — the parser REGENERATES keys.** Traced:
>   the orphaned old XMArg carries the BUILD key `_xmkey="1"` (from `getXMArgID`);
>   the re-materialized nodes carry FRESH parser keys `"XM45".."XM48"`. No shared
>   key → no correspondence to remap through. (Implemented the remap at the
>   reparse site + the libxml-footgun snapshot fix below; it found no mapping and
>   fell back to the sentinel drop = content loss. Reverted.)
> * **NEW landmine found + must-NOT-fix: the reparse orphan-detection
>   (`parser.rs:1502-1528`) is DEAD CODE.** Its snapshot uses
>   `descendant-or-self::*[@xml:id]` + `get_attribute("xml:id")` — the libxml
>   namespace footgun, so it captures **0 ids** (verified: 35/74 reparses had >0
>   id-nodes yet snapshotted 0). "Fixing" the footgun (use `@*[local-name()='id']`
>   + `get_attribute_ns`) **ACTIVATES** the existing `record_replacement(pre_id,
>   "__LOSTNODE__")` path → the LOSTNODES cleanup then UNLINKS the dangling `\Pr`
>   content refs → **content loss**. So the footgun is currently *masking* a
>   content-losing drop; do not naively repair it without first switching the
>   orphan handler from drop to re-point.
> * **THE crux, now precise.** `parse_rec`'s XMArg/XMWrap branch (`parser.rs:1136-1196`)
>   ALREADY does the right thing — it transfers the old node's xml:id onto the
>   parsed result AND re-points refs (`resultid→newid`, `:1189-1196`). It heals
>   the *other* dual args (keys 2,3,5,6,7,8 — confirmed parse_rec'd in the trace,
>   they resolve). **Only the `\Pr` (OPFUNCTION) ARGUMENT XMArgs (keys 1,4) BYPASS
>   `parse_rec`** — they are consumed by the function-application path
>   (`parse_single`→`into_xmath`), so the id-transfer never runs for them and the
>   content ref strands. **Next step (precise): determine why the OPFUNCTION-arg
>   XMArg is consumed by `parse_single` instead of being `parse_rec`'d, then either
>   (a) route it through the same id-transfer, or (b) capture its old-id→new-id at
>   the consume site and `record_replacement(old,new)` (re-point, NOT drop) so the
>   existing LOSTNODES cleanup heals it content-preservingly.** This is contained to
>   the parser reparse/apply path — no XM-enum or Base_XMath change needed.
>
> **2026-06-26j — EXACT unrecord site pinned by backtrace; the consume path is the
> ancestor `parse_single` reparse, not a standalone `parse_rec`.** Dumped the
> pre-parse `\Pr` dual (it's the **physics package** `\lx@physics@operatorP`, an
> `I_dual`: content `apply(wrap(probability), wrap(XMRef[idref]))`, presentation
> `XMWrap[ XMWrap[ Pr ( XMArg[_xmkey=1, xml:id=A1.E4.m1.1b] ) ] ]`). **At parse
> entry the arg XMArg carries BOTH `_xmkey="1"` AND `xml:id="A1.E4.m1.1b"`, and the
> content ref resolves** — the strand is created DURING the parse. Instrumented
> `unrecord_id` (backtrace, watch=`A1.E4.m1.1b`): it is unrecorded **exactly once**,
> via `unrecord_node_ids` ← `parse_single` reparse (parser.rs `:1501`) ← nested
> `parse_rec`/`parse_children`. So the arg XMArg is **swallowed by the
> `parse_single` reparse of an ANCESTOR** (the presentation `XMWrap`), NOT
> parse_rec'd standalone — which is exactly why the `:1136-1196` id-transfer (that
> heals keys 2,3,5,6,7,8) never applies to it. The reparse unrecords every
> descendant id and rebuilds via `into_xmath` with fresh ids; the `|`-conditional
> inside the arg spawns NEW parser duals with regenerated `_xmkey` (`XM45..`), and
> `XM::Arg` ingestion drops the original `_xmkey`/id — so NO old↔new correspondence
> survives for a remap.
> * **Convergence of all prior pins:** this is the SAME structural defect the
>   2026-06-08 trace and the XM::Arg notes (2026-06-26f/g/h) circled — `XM::Arg`
>   dropping the build `_xmkey`/id is *one* link in the chain, but the id is
>   ultimately lost at the ancestor `parse_single` reparse (`:1502-1528` block),
>   whose orphan-detection is dead (the `@xml:id` footgun) and, if revived,
>   content-losing.
> * **The fix is genuinely the "dedicated session" multi-level reparse work**, not
>   a one-site patch. Concretely it requires an old→new id correspondence that
>   survives the nested `parse_single` rebuild. Two viable designs, both non-trivial:
>   (A) make `XM::Arg` carry the build `_xmkey` (NOT just the id — the prior
>   attempts carried the *id*, which the rebuild discards anyway; the *key* is what
>   the orphan-detection could match on) AND switch the reparse orphan handler from
>   sentinel-drop to `_xmkey`-keyed re-point; OR (B) at the `parse_single` reparse,
>   snapshot `xml:id → _xmkey` of the OLD subtree before unrecord and `_xmkey →
>   xml:id` of the NEW subtree after `into_xmath` (BOTH within the same reparse
>   scope, before `resolve_xmkeys` strips keys), then `record_replacement(old,new)`
>   for the LOSTNODES cleanup to re-point. (B) was prototyped but failed because the
>   key snapshots were taken at the WRONG nesting level (the orphaning reparse and
>   the rebuild-with-key happen in DIFFERENT nested `parse_single` calls) — the fix
>   must thread the correspondence across the recursion, or hoist it to a single
>   post-parse pass keyed on a marker that `XM::Arg` preserves. **Groundwork now
>   complete: exact unrecord site + the four ruled-out approaches + the two viable
>   designs with their failure modes are documented.**
>
> **2026-06-26k — DEFINITIVE ROOT: the ASF-vs-RecDescent node-identity divergence
> (confirmed by side-by-side Perl read).** Perl `MathParser.pm:309-378` `parse_rec`
> is structurally IDENTICAL to the Rust port — same XMath branch
> (`unRecordNodeIDs`+`unbindNode`+`appendTree`), same XMArg/XMWrap attribute/id
> copy, same `resultid→newid` ref re-point (L366-371). So the divergence is NOT in
> parse_rec's logic. **It is in what `$result` IS:** Perl's `parse_single` returns
> an **array-tree that embeds the ACTUAL parsed child nodes** (incl. the reused
> XMArg-result subtrees), so `appendTree($node,$result)` re-attaches those real
> nodes and `appendTree` PRESERVES their `xml:id` — the referenced target keeps its
> id, the content XMRef resolves. **Rust's ASF `parse_tree.into_xmath` REBUILDS
> fresh nodes** (XM::Apply/Dual/Wrap materialization via `open_element_at`), so a
> referenced target that is re-materialized (rather than reused as an `XM::Lexeme`)
> gets a fresh id and the pre-existing content XMRef strands. This EXACTLY explains
> the split: simple atom args (kept as `XM::Lexeme` → `into_xmath` reuses
> `nodes[id]`, id preserved) resolve (keys 2,3,5,6,7,8); the complex `\Pr` arg
> (`(s_A,s_B|Ω=…)`, re-parsed into a fresh conditional dual → XM::Apply, NOT a
> Lexeme) is rebuilt fresh → id lost (keys 1,4). **This is the design-doc's
> long-standing "ASF materializes a fresh tree" hypothesis, now PROVEN against the
> Perl source.** The faithful fix is therefore architectural: `into_xmath` must
> REUSE the input DOM node (preserving its `xml:id`/attrs, like the `XM::Lexeme`
> arm) whenever an XM structural node corresponds to a pre-existing referenced
> input node — i.e., the ASF→DOM materialization needs an identity-preserving path
> for non-leaf nodes, not just leaves. That is the genuine dedicated-session scope;
> the LOSTNODES re-point (design B above) is the pragmatic alternative if full
> node-reuse is too invasive. **Not blind-attemptable in a loop tick.**
>
> **2026-06-26l — THE TRIGGER, isolated: a CONTEXT-DEPENDENT parse FAILURE of the
> `\Pr` argument (not an id-machinery bug per se).** Instrumented `parse_rec`
> entry + the `parse_single` outcome for the dangler XMArgs (reverted, clean):
> `[parse_rec ENTER] tag=XMArg id=A1.E4.m1.1b _xmkey=1` → `parse_single -> None`.
> **The `\Pr` argument `s_A,s_B|Ω_{len=k}` FAILS to parse** (returns `None`), so the
> `parse_rec` XMArg id-transfer (`:1136-1196`, which heals the args that DO parse —
> keys 2,3,5,6,7,8) is never reached; the XMArg is left transparent and the ancestor
> presentation-`XMWrap` `parse_single` reparse then decomposes/re-ids it, stranding
> the content ref. **Confirmed against the minimal standalone case:**
> `\[ \Pr(s_A,s_B|\Omega_{len=k}) \]` ALONE → the arg PARSES (→ `<XMDual
> xml:id="S0.Ex1.m1.1">` `apply(open-interval,…)`, the XMArg id transfers, the
> content XMRef resolves, **0 danglers**), matching the design-doc's long-standing
> "standalone resolves" note. So the failure is **context-dependent** — something in
> the paper's ~150-line preamble changes the parse so the SAME arg fails in-context.
> (Separately confirmed `a,b|c` standalone is a genuine Rust-only grammar gap:
> `ltx_math_unparsed` in Rust, 0 in Perl — the VERTBAR comma-list family, a known
> open math-parser aside.)
> * **This UNIFIES the whole investigation:** the dangling content-XMRef is a
>   *downstream symptom* of an unparsed (failed) dual argument. When the arg parses
>   (standalone, and for keys 2,3,5,6,7,8 in-context), the id-transfer preserves the
>   target id and the ref resolves — exactly Perl's behavior. When it fails, the id
>   is stranded.
> * **Two fix axes, both dedicated-session:** (A) **Parse-coverage** — make the
>   in-context arg parse (find the preamble-induced divergence; relates to the open
>   VERTBAR/comma-list grammar asides). Faithful and would fix the ROOT, but
>   context-dependent and grammar-sensitive. (B) **Failure-robust id preservation**
>   — when a referenced dual-arg XMArg fails to parse, preserve its id through the
>   ancestor reparse so the content ref resolves regardless. The robust correspondence
>   is the **reused-leaf identity**: the arg's leaf XMToks are reused (`XM::Lexeme`)
>   in the ancestor's re-parsed tree, so the new arg-top = the new node whose reused-
>   leaf-set equals the old XMArg's leaf-set; `record_replacement(oldXMArgId,
>   newTopId)` then re-points the ref content-preservingly (this is design B/the
>   LOSTNODES path done with leaf-correspondence instead of the regenerated `_xmkey`).
>   (B) is more general (heals ANY unparseable dual arg) and avoids grammar churn,
>   but the leaf-set correspondence + Perl-granularity matching needs the full
>   math-fixture validation budget. **Groundwork COMPLETE: the trigger (arg parse
>   failure), the standalone-vs-context split, and both fix axes with the concrete
>   leaf-correspondence mechanism are all documented.**
>
> **2026-06-26m — 9-LINE MINIMAL REPRO + the prune sweep is CORRUPTING content
> (new bug).** Reduced the witness from 167 lines to 9
> (`docs/reproducers/class_b_aligned_pr_xmref.tex`, converts <1 s — iterate the fix
> on THIS): `\appendix\section` + `\begin{equation}\begin{aligned} \rho &=
> \Pr(s_A,s_B|\Omega)\\ \sigma&=x \end{aligned}\end{equation}`. Bisection result:
> * Trigger = `aligned` (≥2 rows) + `\Pr(a,b|c)`. A SIMPLE (non-aligned) equation
>   resolves (0 danglers); `align` (not `equation`+`aligned`) also resolves. So it
>   is specifically the `equation{aligned}` → `rearrange_lone_ams_aligned` path.
> * **`\appendix` is the dangle-vs-corrupt discriminator — via the id SCHEME.** With
>   `\appendix` the ids are `A1.E1.m1.*`; the `\Pr` content refs `A1.E1.m1.1a/1b`
>   **dangle** (2 danglers). With a plain `\section` the ids are `S1.E1.m1.*` and the
>   SAME refs are **silently DROPPED** by `prune_dangling_split_xmrefs`'s broad
>   `^S\d+\.E\d+\.m\d+\.` sweep → the `\Pr` argument **VANISHES from content MathML**
>   (verified: the `apply(probability,…)` has NO argument child). **So the prune
>   sweep is actively CORRUPTING content for section-numbered aligned `\Pr` across
>   the corpus** — not just hiding a warning. The appendix case only "looks worse"
>   (a visible dangler) because the `A`-scheme dodges the `^S`-anchored regex; it is
>   actually the LESS-destructive of the two (ref structure preserved, just broken).
> * **Perl target (captured on the minimal repro):** the `\Pr` content arm has TWO
>   resolving refs — `A1.E1.m1.1` AND `A1.E1.m1.1.mf` — i.e. content-branch arg +
>   the presentation-branch `.mf` clone (`amsmath.sty.ltxml` L664
>   `local $ID_SUFFIX='.mf'`), both present in the tree. The faithful fix must
>   reproduce this dual (content + `.mf`) id scheme through the rearrange/MathFork +
>   parse, so both refs resolve.
> * **Implication for the prune sweep:** removing/narrowing the broad `^S\d+` sweep
>   would stop the content-corruption but re-expose the dangling-ref warning flood
>   (the reason it was added). The right end state is the faithful fix (refs
>   resolve) → then the broad sweep can be deleted. Until then the sweep trades a
>   warning for silent content loss — a worse deal than the doc's original "content-
>   preserving" claim assumed (which held only for the genuinely-redundant
>   `_split_ref`/MULOP case, NOT for `\lx@dual` content-arm arg refs).
>
> **2026-06-26n — leaf-LCA re-point (design B) IMPLEMENTED + TESTED → eliminates
> danglers but REGRESSES (reverted, clean). Key insight: must re-point to a
> CONTENT-branch node, not presentation.** Implemented the reused-leaf-LCA
> correspondence at the `parse_single` reparse (footgun-fixed snapshot of `id →
> descendant XMTok leaf identities`; after `into_xmath`, for each orphaned id,
> `record_replacement(old, LCA_of_surviving_leaves)` for the LOSTNODES cleanup to
> re-point; helper `lca_of_reattached`). Result on the minimal repro AND full
> witness: **0 danglers** (down from 2 / 4) — the re-point mechanism WORKS. **BUT**
> `cargo test` regresses `tests/ams/mathtools.xml` (equation/equationgroup nesting
> diff) and the `\Pr` XMDual **collapses** to a single `<XMTok
> meaning="probability" role="OPFUNCTION">Pr</XMTok>` instead of Perl's parallel
> `apply(probability, ref)` + presentation. **Root of the regression:** the per-row
> MathFork main IS the presentation branch, so the LCA of the reused leaves is a
> PRESENTATION node; re-pointing the *content*-arm XMRef into the presentation arm
> makes `mark_xmnode_visibility`/`prune_xmduals` treat the dual as single-branch and
> COLLAPSE it — restructuring every aligned equation. **So re-pointing to the
> presentation arm is wrong; Perl keeps a CONTENT-branch copy of the arg (id
> `…m1.1`) plus a `.mf` presentation clone, and the content ref targets the
> content-branch copy.** This sharpens the faithful fix: the rearrange/MathFork must
> MATERIALIZE a content-branch arg (so the content ref resolves WITHIN the content
> branch) rather than the content ref borrowing a presentation node — i.e. the
> Perl `local $ID_SUFFIX='.mf'` dual-copy scheme, applied through
> `rearrange_lone_ams_aligned` + the parse reinstall. The leaf-LCA machinery is a
> sound *mechanism* but needs the CONTENT-branch target; that target doesn't exist
> in the current Rust tree (only the presentation arg does), so producing it is the
> dedicated-session change. (Negative result; `lca_of_reattached` + the
> footgun-fixed snapshot are documented here for reuse.)
>
> **2026-06-26n-corr — the leaf-LCA regression is COMPACTION, not collapse.**
> Traced the regression mechanism: `mark_xmnode_visibility` correctly marks the
> content branch `_cvis` (the content-arm XMRef node itself is marked at
> document.rs:3211 before following its idref), so the dual does NOT collapse in
> `prune_xmduals`. Instead, `compact_xmdual` (the both-branches-visible path) MERGES
> the content op `probability` + the presentation op `Pr` into a single
> `<XMTok meaning="probability" role="OPFUNCTION">Pr</XMTok>` — because re-pointing
> the content ref into the presentation arm made the dual "compactable." Perl keeps
> it a PROPER dual (separate content `apply(probability, ref→m1.1)` + `.mf`
> presentation clone) and does NOT compact. So the faithful fix must keep the dual
> un-compactable by giving the content arm its OWN arg target (the `.mf`/content-
> branch scheme) — re-pointing to presentation is fundamentally wrong because it
> triggers compaction. Final confirmation that axis B (re-point) cannot be faithful;
> only axis A (parse-coverage so the arg parses → native id-transfer) or the deep
> rearrange content-branch materialization will match Perl.
>
> **2026-06-26o — LANDED: operand-protection guard stops the content-MathML
> CORRUPTION (the first real Class-B code fix; partial — corruption→honest
> dangling, not full resolution).** Added a safety guard to the broad sweep in
> `Document::prune_dangling_split_xmrefs` (document.rs): **never drop a dangling
> ref that is an OPERAND (non-first element child) of an `XMApp`.** Such refs are
> essential shared content links (e.g. an `\lx@dual` content arm
> `apply(probability, XMRef)`); dropping one emitted a malformed `apply(probability)`
> with no operand = silent content loss. The genuinely-redundant MathFork/`_split_ref`
> mirrors the sweep targets live in `XMWrap`/`XMArray` (NOT as `XMApp` operands) and
> are handled by the MARKED sweep (untouched), so the guard CANNOT re-flood the
> ~1500-paper wp3 cluster (verified: an `align`+`\mathcal{L}\rho` MULOP-absorption
> case stays 0 danglers / 0 warnings). Effect: section-numbered aligned `\Pr` now
> PRESERVES its arg (dangling ref) instead of dropping it — moving Rust *closer* to
> Perl (arg present) and unifying the appendix/section behavior. Validation:
> `cargo test --tests` 1468→1468/0 (zero fixture changes — no fixture witnessed the
> guarded case), clippy clean, + new regression `cluster_xmref_pr_arg_not_dropped`
> (asserts no malformed `apply(probability)`). **This does NOT make the ref resolve**
> (that is still the dedicated rearrange/content-branch-materialization session per
> 2026-06-26n) — it stops the strictly-worse CORRUPTION. Trade-off: surfaces a few
> genuine "No node found" Warns (honest danglers Perl resolves) that the corrupting
> drop was hiding — aligned with the signal-fidelity principle (surface genuine,
> suppress spurious). NOTE: the broad sweep's comment claiming it targets unmarked
> add_column refs is STALE — `add_column_to_math_fork` DOES `_mf_ref`-mark its refs
> now, so the broad sweep's real catch is these `\lx@dual` content-arm refs; once
> the faithful fix lands (refs resolve) the broad sweep can likely be deleted.
>
> **2026-06-26p — AXIS A ROOT CAUSE + EXACT FIX FOUND (one line); blocked only by a
> known VERTBAR ambiguity (dedicated-session pruning work).** Bisected the trigger to
> a SPECIFIC arg shape: in an aligned `\Pr(…)`, `\Pr(x)`/`\Pr(a|b)`/`\Pr(a,b)` all
> resolve (0 danglers); ONLY `\Pr(a,b|c)` dangles. So Class B (this witness) is a
> **comma-list-LHS conditional parse gap**: the grammar's only VERTBAR-modifier rule
> is `statement vertbar statements` (`builder.rs:447`, single LHS), so `a,b | c`
> (list LHS) has no rule → the arg fails to parse → no native id-transfer → the dual
> content ref strands. **The fix is ONE line:** generalize the LHS to `statements`
> (`statements vertbar statements`; `statements` subsumes `statement`). TESTED:
> standalone `a,b|c` now parses (was `ltx_math_unparsed`; fixes the open VERTBAR
> math-aside too), M1 → 0 danglers, full witness → 0 danglers, and the refs
> **RESOLVE** with the **dual PRESERVED** (not merged) — the FAITHFUL fix (arg
> parses natively, exactly Perl's path). **BUT it regresses absolute-value parsing:**
> `a|a|+b|b|+c|c|` parses as `conditional@(conditional@(a,a),…)` instead of `a *
> absolute-value@(a) + …` — the classic abs-value-vs-conditional VERTBAR ambiguity.
> The list-LHS rule perturbs the parse forest so `prefer_fewer_conditionals`
> (`parser.rs:2167`) no longer selects the 0-conditional abs-value parse over the
> 2-conditional reading. Reverted (`qm_test` + `parse_tree_count_limits` restored).
> **Targeted-fix direction (dedicated math-parser session):** restrict the new rule
> to GENUINE comma-lists — a separate `comma_statements` nonterminal (≥1 punct) that
> does NOT reduce to plain `statements`, so `comma_statements vertbar statements`
> fires on `a,b|c` but never on the comma-less `a|a|` abs-value — OR a pruning tweak
> that prefers the abs-value (fewer-conditionals / vertbar-pairing) reading. Either
> is grammar/ambiguity work needing the full math-fixture budget. **This is the most
> concrete Class-B fix to date: the exact one-line grammar change + the exact
> regression + the exact targeted direction are all pinned. Axis A (parse-coverage)
> is now the recommended path over the deep rearrange materialization — it produces
> the genuinely-correct tree (refs resolve, dual intact), needing only the
> ambiguity-scoping above.**

> **UPDATE 2026-06-25 — parse-time warning SUPPRESSED (signal-fidelity), Class B
> structural fix still deferred.** The dominant emitter of this cluster was the
> math-parser's *parse-time* `realize_xmnode` (`parser.rs:2840`, ~128.9k of the
> ~130.8k `warning:expected:id` messages in the 10k sandbox). It is a **benign
> transient false-positive**: it consults the LIVE `document.lookup_id` mid-parse
> while XMath elements reinstall (a Rust/ASF artifact Perl lacks), but the FINAL
> tree is clean — empirically verified on the heaviest witness `0704.2400`: of 98
> transient misses, 85 ids are present in the output and the other 13 leave **0
> dangling `<XMRef idref>`** (the output has 0 dangling idrefs of 2597). The whole
> 10k run has **0 `error:expected:id`**, i.e. the authoritative post-processing
> check (`latexml_post` `realize_xm_node` / `mark_xm_node_visibility_aux`, faithful
> Perl `Post.pm:1444/1456` Error) flags no genuine output danglers. So the
> parse-time Warn was made SILENT (`parser.rs`), output byte-identical, genuine
> danglers still caught downstream. The Rust-only `base_xmath.rs` createXMRefs
> "Unresolved _xmkey" Warn (Perl `Base_XMath.pool.ltxml:306-308` is silent) was
> also removed. **The Class B structural divergence below (equation→equationgroup
> refnum-id loss) is NOT fixed** — it simply no longer floods the cortex signal;
> when a genuine dangler reaches output, the faithful post-Error reports it.
>
> **CLASS B RE-CONFIRMED + GUARANTEE CLARIFIED 2026-06-25.** Re-ran the
> fully-traced witness `2311.01600` on current HEAD. Class B is **still live** but
> the picture has moved on from the 2026-06-08 trace:
> * The **container-id half IS fixed** — the group is `A1.E66` (NOT the old
>   `A1.p10.1`), with per-row presentation equations `A1.E66X`/`A1.E66Xa`/`A1.E66Xb`
>   and math under `A1.E66Xa.m1.*` (the X-equation scheme).
> * **6 residual danglers**: `A1.E66.m1.1a/1b/4a/4b`, `A1.E68.m1.1a/1b` — the `\Pr`
>   XMDual **content** refs, minted against the *group* Math scheme `A1.E66.m1.*`
>   (pre-rearrange) while the parsed content lands under the *per-row* X scheme
>   `A1.E66Xa.m1.*`. So `A1.E66.m1.*` never materializes → dangling.
> * **Current root** (`rearrange_lone_ams_aligned`, `amsmath_sty.rs:1703`): Rust
>   builds *per-row* equations each with their **own** MathFork main, id'd off the
>   inner X equation (`A1.E66Xa.m1.*`). **Perl builds ONE group-level content Math
>   `A1.E66.m1` + per-row presentation equations.** Closing it = restructure
>   rearrange to Perl's single-content-Math shape (main Math id derived from the
>   GROUP, one content Math, parse-reinstall id preservation). Multi-part,
>   high-fixture-churn — the "3 prior reverts" area. **Deferred to a dedicated,
>   carefully-validated effort.**
>   * **MOVE-only approach EMPIRICALLY EXHAUSTED 2026-06-25.** Changed the
>     main-fork content append from `append_clone` (re-ids) to a true MOVE
>     (`add_child`, keeping the originals' `A1.E66.m1.*` ids). Built clean, but the
>     witness STILL danglers 6 (set shifted to `A1.E66.m1.{1,1a,4,4a}` /
>     `A1.E68.m1.{1,1a}`). Dumped ids: the moved content's defs are re-assigned to
>     `A1.E66X/Xa/Xb.m1.*` (ZERO `A1.E66.m1.*` defs). So a re-id pass keyed off the
>     **main Math's id** — `open_math_fork` (`base_xmath.rs:1397`) ids the main
>     `<ltx:Math>` off the enclosing **inner X equation** (`A1.E66Xa` → `A1.E66Xa.m1`),
>     and `finalize_rec`/`generate_id` (`document.rs:377/4565`) + the parse reinstall
>     overwrite the moved ids — confirming §3e is LIVE on current code, not stale.
>     **Reverted (witness gate failed).** The actionable next step: make
>     `open_math_fork` (or rearrange) derive the MathFork main content Math id from
>     the **GROUP** (`{group}.m1`), build it **ONCE per group** (per-row mains
>     collide on `{group}.m1`), MOVE all rows' originals into it, and ensure the
>     parse reinstall **preserves** pre-existing ids. Core-builder + math-parser
>     change — validate every `MathFork`/`equation`/`equationgroup` fixture vs Perl
>     incrementally.
>
> **GUARANTEE (important correction).** These danglers are **NOT** unflagged: core
> `markXMNodeVisibility` (`latexml_core/document.rs:3250`) fires **`Warning:expected:node`
> "No node found with id='…' (referred to from ltx:XMRef)"** — one per dangling ref,
> reaching the XMDual **content** branch — faithful to Perl `Document.pm:1553` (a
> Warning). The 2026-06-25 parse-time `realize_xmnode` suppression did **not**
> create a false negative; the earlier "0 errors / 6 danglers" reading was a grep
> artifact (only `expected:id` was counted, not the `expected:node` Warning that
> actually fires). So the id-message guarantee for digestion-created danglers is
> already met by the faithful `expected:node` signal. (A genuinely-final-tree
> fresh-scan check was prototyped and **reverted** — it merely duplicated the
> `expected:node` Warning as an `expected:id` Error, double-fired, and escalated
> severity, which Perl does not.) The remaining hardening target is *post-created*
> danglers (a target renamed during a post phase), where the post
> `markXMNodeVisibility` uses the possibly-stale `idcache` — verify before adding
> any new signal.
>
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

> **IMPLEMENTATION UPDATE (2026-06-08): root cause found + container-id half
> FIXED.** Phase-0b pinned the drop to a **libxml string-accessor footgun**, not
> a provisional-id reassignment: `xml:id` is stored namespaced (local name
> `"id"`), so `rename_node_internal`'s `key == "xml:id"` capture never matched
> (id lost across the equation→equationgroup rename) and
> `rearrange_lone_ams_aligned`'s `get_attribute("xml:id")` always read empty.
> Both fixed (see [`XMLID_ACCESSOR_AUDIT_2026-06-08.md`](archive/XMLID_ACCESSOR_AUDIT_2026-06-08.md)).
> Result: the equationgroup now keeps its refnum id and the inner equation gets
> the Perl `{id}X` suffix — **`split.tex` is now full Perl parity** (re-blessed),
> and 2311.01600's containers are correct (`A1.E66`/`A1.E66X`, the
> presentation-branch refs resolve). **Residual:** 2311.01600's `\Pr` *content*
> refs (`A1.E66.m1.1a/1b…`) still dangle — they were minted against the
> pre-rearrange Math/XMArg id scheme and need the MathFork **content-branch**
> id reconciliation (the shallow-XMArg-ref nuance below; a distinct, deeper
> sub-issue, still open).
>
> **STRUCTURAL DIAGNOSIS (Phase-0c, 2026-06-08) — deeper than an id fix.**
> Dumping both trees for the eq:statedefs `\begin{equation}\begin{aligned}` (3
> rows of `\Pr(\fullaccevent{i})…`) shows Rust and Perl build *structurally
> different* equationgroups:
> * **Perl:** group `A1.E66` → **5 per-row presentation equations**
>   (`A1.E66X`, `A1.E66Xa`–`Xd`) **+ a separate CONTENT `<Math A1.E66.m1>`**
>   carrying the deep semantic tree (`A1.E66.m1.1.1.…`, with a `.mf` MathFork
>   marker). The `\Pr` content refs resolve into that content Math — 360
>   `A1.E66.m1.*` nodes exist.
> * **Rust:** group `A1.E66` → **one** inner equation `A1.E66X` with three math
>   elements (`A1.E66X.m1/m2/m3`); **no separate content Math**, so the
>   digestion-minted `A1.E66.m1.*` refs have no target.
>
> So this is **not** an id-preservation fix: `split.tex` (a lone-aligned
> equation with NO content-bearing duals) has Perl `Ch0.Ex2X.m1` — the X-on-Math
> scheme Rust already matches — and re-blessing confirmed it. There is **no
> single "keep the Math id" rule**; the behaviour forks on whether the equation
> carries content-bearing XMDual macros (`\Pr`-style) that Perl splits into a
> dedicated content Math + per-row presentation equations. Closing the residual
> requires porting that **content/presentation MathFork split** in
> `rearrange_lone_ams_aligned` (a structural rebuild, high fixture-churn risk —
> validate every `equationgroup`/MathFork fixture vs Perl). **Deferred as a
> dedicated effort**, not a drive-by — same discipline as the `rewrite.rs:1242`
> lesson ([`XMLID_ACCESSOR_AUDIT_2026-06-08.md`](archive/XMLID_ACCESSOR_AUDIT_2026-06-08.md)).

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

## 3b. The `S7.E46`/`E48`/`E50` cluster (Phase-0d, 2026-06-08) — benign warnings + a separate deep mis-parse

Investigated as the "more tractable" sibling cluster. It splits into two
non-fixes:

1. **27 `expected:id` warnings are benign.** Every target *exists in the final
   output* (verified); they're transient parse-time misses because
   `realize_xmnode` consults the **live** `document.lookup_id`, which is mutated
   as each XMath element reinstalls (old ids unrecorded, new registered), while
   the grammar's role path (`data::resolve_xmref`) uses the **frozen
   `MATH_IDSTORE` snapshot**. WARN-level, no rc/canvas-signal impact. **Routing
   `realize_xmnode` through `resolve_xmref` to silence them DUPLICATES content**
   (`\choose` → `a + ba + b binomial c + dc + d`; regresses
   `choose`/`declare`/`sampler`) — callers rely on an unresolved ref returning
   the XMRef itself. Left as-is with a do-not-fix comment at the call site.
2. **`S7.E46` is genuinely mis-parsed** (Rust leaves one `XMWrap rule="Anything,"`
   with empty `[]` superscripts; Perl parses fully, `rho^[virtual]_(K_A…)`).
   But this is a *deep* parse/expansion issue on a complex lone-aligned
   `\begin{equation}\begin{aligned}` with paper macros (`\suplabelsbrkt` (empty),
   `\Cfull`, `\tracenorm`), entangled with the §3 MathFork structural divergence
   — **not** the ref-resolution warnings, and not a drive-by. **Deferred** with
   the same discipline as §3.

Net: the residual `expected:id` work is **not** more tractable than §3; both the
`\Pr` content-branch and the `S7.E46` mis-parse are deep math-parser/MathFork
work for a dedicated session.

### 3c. Attempted the §3 fix (Phase-0e, 2026-06-08) — it's multi-part, not one line

Pinned the exact divergence: Perl `rearrangeLoneAMSAligned` (amsmath.sty.ltxml
L657-671) **MOVES** the original cell nodes into the MathFork MAIN/content
branch (`appendChild`, keeping their `<group>.m1.*` ids); the Rust port
(`amsmath_sty.rs` ~L1835) **clones** them (`append_clone`), re-id'ing to the
X-equation scheme so the `\Pr` refs strand. **Switched it to a move and it
changed NOTHING** (witness still 27 `expected:id` / 6 `expected:node`, `…m1.1a`
still absent) — because the subsequent **math parse re-ids the content branch**
from the inner-equation-derived main Math id (`<group>X.m1`) regardless of
move-vs-clone. Reverted (no behaviour change), left a comment at the site.

So closing §3 is a **multi-part** change, not a one-liner:
1. the MathFork **main Math id must derive from the GROUP** (`<group>.m1`), not
   the inner `X` equation (`<group>X.m1`) — so parsed content lands on the
   scheme the digestion-minted refs expect;
2. **move** (not clone) the originals into main; and
3. the math-parse reinstall must **preserve** those ids for the main branch.
Plus the presentation branch needs the Perl `.mf` `ID_SUFFIX`. This touches the
MathFork id derivation in the core builder + the parse reinstall — genuinely a
dedicated effort, and the third core-math change in this area to be reverted
(after `rewrite.rs:1242` and the `realize_xmnode` snapshot route). **Deferred.**

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
