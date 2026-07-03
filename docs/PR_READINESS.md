# PR readiness review — `ar5iv-2606-prep` → `main`

> Point-in-time critical review (2026-07-02) of the 105-commit branch before
> the PR to main and the July-5 fleet rebuild. Two phases: (1) a one-by-one
> risk filter over all commit descriptions; (2) three parallel reviewer
> agents over the flagged clusters, findings verified against current code.
> Companion to the per-commit landing evidence (witnesses, fixture parity,
> suite runs) recorded in `SYNC_STATUS.md` / `MATHML_POST_LINE_AUDIT.md`.

## Phase 1 — risk filter (all 105 commits)

Verdict rules: docs/chore/clippy/fmt = SKIP; strong landing-time verification
(byte-identical Perl witnesses, upstream-identical fixture diffs, corpus A/B,
fleet deployment) + small size = PASS; flagged HIGH-RISK when a commit has
subtle semantics, mechanical sweeps, cache invalidation, heuristics over
corpus input, concurrency/pipes, hand-written parsers, manual save/restore
disciplines, or zero-fixture-coverage behavioral surface.

- **SKIP (docs/chore, no runtime code): 45 commits.**
- **PASS with landing evidence (33 commits)** — highlights of why:
  - `179d445955`+`6d82d2eb06` (#2829 port): critically audited same-day;
    8-element witness battery byte-identical vs reference-tree Perl; the one
    real bug found (keyvals double-digest panic) is fixed.
  - `e577613fb1` (opdict tables): machine-generated verbatim from Perl range
    strings, non-overlap asserted at generation, regression tests pin the
    fixed codepoints. (Note: sortedness is asserted in the generator, not by
    a Rust debug_assert — acceptable, tables are static.)
  - `d5f8df16d2` (math over-parse prune), `da74f6ecfe`/`383f9d6517` (XSLT
    O(n²) fixes): verified output-neutral per-paper (isolated-dir byte-diff
    protocol) / byte-identical.
  - `191d8e26d2` (depth guard): fired 0× across 10k-paper validation.
  - `57d28a633c`/`5b6fdcb2c7` (graphics chain): fleet-deployed + verified.
  - `f3fab341d7` (iflimit raise): corpus-validated (fatal 281→269).
  - `be167b33ef` (`\+` retraction): user-reviewed twice at landing, witness.
  - `cb431a2f74` (`\Ucharcat`): dump-parity verified (246→0 special_relax).
  - upstream syncs `83b05eea5e`/`e5b77edd91`/`20adb952e7`/`c2e885e10b`:
    upstream-fixture parity.
  - llncs/wrapfig/marginpar/booktabs/bibblock/mhchem-contrib singles: small,
    fixture- or KPE-documented.
- **FLAGGED HIGH-RISK: 27 commits in 3 clusters** (reviewed in Phase 2):

| Cluster | Commits | Risk theme |
|---|---|---|
| A. Core engine/gullet | `6ac88769eb` `c84a615755` `2335c78c44` `68769f7ce1` `8646ec32cb` `7b64a48ad1` `915e02aa07` `da2515e3a8` `123d03b3ed` `f178702f11` | noexpand semantics, mouth-boundary crossing (surpass-Perl), UTF-8 truncation, arena-reset discipline, the mechanical pin! sweep, kpsewhich miss-caching, raw-loading real mhchem, global natbib hoist |
| B. MathML post | `856de84a10` `2c02795251` `ff87a841e5` `24def068cd` `3b20c4f399` `8074ef8e0a` `3ab9ce3cb3` `e6347fd880` | path-based tree mutation, thread-local style/context save-restore, share-id minting into the live DOM, \let trampoline grouping, zero fixture churn (~30-formula witnesses vs corpus millions) |
| C. Bindings/guards/infra | `226d3bfa51` `797e1ca6d5` `456dd9f654` `1ec4fb3643` `6e02b6a57b` `dd226d1973` `d74529d9eb` `786d9ed89d` `adc26fbc59` `45dc1d26af` `75c452843d` `f0a8847c07` `5326c4505d` `cb8b648784` `c422c64937` `5f3c4a0566` `d63beb0d99` `83a7dc89a4` | hand-written colspec parser, lock-free diagnostics routing, subprocess pipes, corpus heuristics (title promotion), DOM restructuring, rewrite-rule matching, panic→graceful conversions, BibTeX braces, log-format changes vs fleet signal anchoring |

## Phase 2 — reviewer findings

(populated below by the three parallel cluster reviews)

### Cluster A — Core engine / gullet / tokenizer

| Commit | Verdict summary |
|---|---|
| `6ac88769eb` special_relax noexpand | encoding sound; **BUG-RISK**: `\string` of a live family token leaks a raw 0x01 (illegal XML char) into text where Perl leaks a valid `\special_relax`; active-`\` decode fabricates an empty-name CS (pathological); `meaning_key` prefix check sits on the hottest lookup path (profile-verify); MISSING-TEST: no unit round-trip for the family encode/decode |
| `c84a615755`+`2335c78c44` noexpand capture/edef | correct on their paths; **CORNER-CASE**: delimited-arg capture (`read_until` content loop) does NOT collapse family tokens — the transient model covers 2 of 3 capture paths; MISSING-TEST: neither kernel reproducer (`noexpand_transient_capture.tex`, `noexpand_edef_collapse.tex`) was promoted into the suite |
| `68769f7ce1` read_balanced crossing | **BUG-RISK (top of cluster)**: crossing gates on the `autoclose` bit, but `\input` file mouths are opened `autoclose=true` — a truncated/unbalanced `\input`ed file (or raw-loaded .sty) now SILENTLY absorbs parent-document content into the argument where TeX and Perl both error loudly. Recommend gating on mouth KIND (token/string injection) or emitting the parity error for file-backed mouths. MISSING-TEST: no fixture for either direction |
| `8646ec32cb` graceful read_token + UTF-8 | OK — boundary-safe truncation, all degradation paths verified, zero remaining `read_token()?.unwrap()` sites; cosmetic: `\font` label hard-coded for textfont/scriptfont variants |
| `7b64a48ad1` REPORT arena hardening | fix correct; **CORNER-CASE (class not closed)**: `DEFERRED_COMMANDS`, `CURRENT_LOAD_CTX`, siunitx cache, and every `pin!` OnceCell survive `arena::reset()` — safe only via the implicit "no thread reuse after reset" contract; recommend a poison flag/debug_assert or documenting at `reset_thread_engine` |
| `915e02aa07` pin! sweep | OK — wrong-value hazard impossible by construction (`$s:literal` matcher); widens the reset-hazard surface above (note in arena docs) |
| `da2515e3a8` kpsewhich memoize | sound for harness mode; **CORNER-CASE**: persistent (non-harness) cortex_worker never clears the memo across papers — a cached cwd-relative MISS has no exists() re-check; cheap fix: clear in prepare_session |
| `123d03b3ed` mhchem raw-load | **MISSING-TEST (high value)**: no `\ce{H2O}` fixture guards the whole chemistry corpus against TL/expl3 drift; no missing-file fallback shim; chemformula's version=4 pin blocks a later explicit `[version=3]{mhchem}` |
| `f178702f11` natbib hoist | OK — reachable only from the OmniBus sniff, idempotent, global promotion matches top-level semantics; two documented blind spots (pre-bound frame keys; Values not hoisted) |

**Cluster A top risks:** (1) read_balanced `\input`-boundary silent absorption; (2) mhchem corpus flip unguarded by any `\ce` fixture; (3) 0x01 leak via `\string`; (4) noexpand model missing on delimited-arg path + unpromoted reproducers; (5) pin!/SymStr reset contract undocumented.

### Cluster B — MathML post-processing

| Commit | Verdict summary |
|---|---|
| `856de84a10` spacewalk | **BUG-RISK (live-confirmed)**: `atompair_spacing` is missing Perl's `(Inner, Punct) => -1` cell — `$\begin{array}...\end{array},x$` loses Perl's `lspace="0.167em"` on the comma; matrix-then-punctuation is ubiquitous. All other 63 cells + walk structure verified correct (rewrap/path-invalidation proven sound; script prefix-match safe). CORNER: childless-script `children[0]` panic (unreachable from own trees); mathscript min 65536-vs-65535 sp; `get_xm_hint_spacing` returns 0 for raw glue strings; `perl_num` is 15-decimals not %.15g (contract comment wrong) |
| `2c02795251` mathstyle/styler | stylemap tables + needs_mathstyle + save/restore discipline all verified exact (live witnesses byte-identical). CORNER: U+2062→ZWSP replacement happens before the invisible-styling suppression (only under --noinvisibletimes); `op_base_is_mo` descends mstyle where Perl's generic-apply stops (⁡ divergence on rare shapes); `pmml_summation` runs the is_mo check at all where Perl never emits ⁡; NOMOVABLELIMITS unported (documented F17) |
| `ff87a841e5` context bindings | ctx save/restore + per-formula re-seed verified sound; **BUG-RISK (live-confirmed)**: the commit message claims FRACOP/ENCLOSE consume CTX_COLOR — they do NOT (an edit-script abort silently dropped those hunks; only sqrt/mroot landed): `{\color{red}$\frac{a}{b}$}` renders a BLACK fraction bar (Perl: red). Stale in-code comments mask the gap. CORNER: XMRef color precedence inverted vs Perl `_getattr`; `color=""` emits empty mathcolor |
| `24def068cd` cmml | **BUG-RISK**: `convert_to_cmml` lacks Perl `cmml_top`'s context bindings (STYLE='text' + FONT/COLOR/…) — decorated-symbol ci interiors render at stale/Display style; `stylize_ci_content` misses the CTX_FONT fallback (`<ci>v</ci>` vs Perl `<ci>𝐯</ci>` in styled context); `cmml_shared` can mint duplicate bare `sh1` xml:ids on no-ancestor-id formulas and never registers ids in the idcache. CORNER: redundant loop clause; even-arity degrades silently; ASCII-only integer test |
| `3b20c4f399` cfrac | faithful (trampoline scoping, phases, all three pull-up branches verified vs Perl); CORNER: `\cfrac[l]` optional-arg tolerance dropped — exact Perl parity but a regression vs the previous Rust binary (KNOWN_PERL_ERRORS candidate); forced `mathstyle='text'` where Perl stores none (outside math only). MISSING-TEST: zero \cfrac fixtures |
| `8074ef8e0a` wrapper/arms | class-merge, FRACOP verbatim thickness, double-enclose rarity all verified; the FRACOP/ENCLOSE color gap is recorded under ff87a841e5 |
| `3ab9ce3cb3` padding carry | round-trip consistent; CORNER: Rust SUMS outer padding where Perl assigns-if-nonzero — XMDual-with-padding double-counts (rare) |
| `e6347fd880` degenerate parses | **BUG-RISK**: both re-entrant Alignment arms degrade SILENTLY (empty revert / dropped cells, no Error!) — violates the fail-toward-flagging directive; triage reads missing-content papers as clean. Ligature guard fixed only the underflow half. Parser-side guards verified content-preserving. MISSING-TEST: none of the 4 guards has a fixture |

**Cluster B top risks:** (1) `(Inner,Punct)` spacing cell — one-line fix, big formula-class churn; (2) FRACOP/ENCLOSE context color — the "black fraction bar in colored text" class survives, masked by an overclaiming commit message; (3) cmml context bindings + share-id integrity; (4) silent Alignment drop; (5) no fixture pins post-processed MathML at all — a small golden set (colored frac/cancel, array-comma, 3-deep cfrac, one --contentmathml pair) would have caught 1-3.

### Cluster C — Bindings / panic-guards / post-infra

| Item | Verdict summary |
|---|---|
| `226d3bfa51` tabularray | parser can't loop, unknown→stub fallback holds; CORNER: bare-colspec shorthand `{Q[c]Q[c]}` + longtblr/talltblr NOT covered (cluster residue — size via corpus grep); naive `contains('c')` misreads `bg=cyan` as center; nested `*{}{}` multiplies past the per-level 1000 cap (OOM/stack); LOW BUG-RISK: fallback re-tokenization of `%` can comment out the closing brace |
| `797e1ca6d5` diagnostics routing | mechanism verified genuinely lock-free (per-thread capture, fold-by-value on join); anchoring + Status:conversion:N intact, regression-tested; CORNER: a panicking worker loses its pre-panic log text/counts (status still non-clean via worker_panicked); capture() reuse invariants unenforced |
| `456dd9f654` stderr capture | no pipe deadlock (dedicated drain thread, keeps draining past the 8KiB kept-cap), non-UTF8 safe, child always reaped; CORNER: unbounded reader join if a descendant survives killpg (timeout no longer strict); EINTR treated as terminal; multi-line gs stderr can inflate line-anchored ^Error: counts (over-count only — fail-safe) |
| `1ec4fb3643` title promotion | **BUG-RISK ×2**: committed debug `eprintln!("PROMOTE2: no first_body")` fires on common paper shapes straight to fleet stderr; the second-pass DFS can STEAL a centered `\Large` block from INSIDE §1 as the paper title (nothing constrains the anchor to document level). CORNER: `\Large` epigraph on abstract-only papers passes all gates; `\\`-joined author lines absorbed into title |
| `6e02b6a57b` ORCID unwind | logic sound (no detached-node trap, no empty-sup); **BUG-RISK (portability)**: `add_child` without `unlink()` — internally-unlinking only on libxml2 ≥2.13; on 2.9-2.12 (RHEL8/9) it corrupts both child chains → double-free. One-line fix |
| lxDeclare trio | replace= unwrap + font-CLASS check faithful; **BUG-RISK ×3**: compiled XPaths still carry dead `@font`/`@meaning` predicates → whole declaration families silently match NOTHING (golden 51 vs Perl 84 decl_id — the historical vanish mode, live); replace-rules bypass declare-side filtering → latent delete-the-wrong-sibling; untagged `scope=section` applies document-globally. CORNER: unrecognized patterns drop with no Warn |
| panic-guard group | **BUG-RISK**: re-entrant-Alignment guard silently skips absorption — a whole table vanishes with NO diagnostic (violates the emit-an-Error directive); 4 unguarded `Node::new().unwrap()` remain on the exact hardened paths (parser.rs:1320/2506/939/946) — next fleet pressure panics there. CORNER: inconsistent loudness in alignment.rs; None-parent walk → silent mis-nesting on degenerate trees |
| `5326c4505d` .bib parsing | **BUG-RISK ×3**: `" and "` split is brace-blind (`{Barnes and Noble}` → 2 authors); quote-delimited fields strip the protective braces (`author="{W3C ...}"` defeats the fix); one unbalanced brace silently swallows every later entry (no resync, no Warn); newline-wrapped `and` merges authors |
| `cb8b648784` bbl-first | precedence matrix verified; **BUG-RISK (load-bearing, pre-existing)**: `find_file` extension test uses `ends_with("bib")` without the dot → `\bibliography{mybib}` never retries `mybib.bib`, silently disabling the new fallback. CORNER: multi-bib `all()` gate declines on partial availability; unspaced-split; 0-byte .bbl still wins (Perl parity) |
| `c422c64937` fvextra | display-path neutralization correct and output-safe; **BUG-RISK (live-confirmed)**: the INLINE `\Verb` + breakanywhere path still hangs to Fatal:Timeout on HEAD (Perl clean) — direct residue of the 121-paper cluster; each occurrence burns a fleet timeout slot |
| logger group | anchoring invariants verified end-to-end (notes cannot inject mid-Error-line; Status emitted exactly once, last, max(core,post)); CORNER: doubled `\n` cosmetic; gs stderr interplay above |
| `83a7dc89a4` xml:id | verified truly dead on all call paths (setter side included); ~30 sibling always-None reads remain as cleanup candidates |

**Cluster C top risks:** (1) fvextra inline path still hangs (live repro); (2) title-promotion debug eprintln + §1-stealing heuristic; (3) .bib parser corner cases now fleet-reachable + the find_file missing-dot bug; (4) silent Alignment drop + 4 remaining unguarded Node::new; (5) lxDeclare silent no-match class.

## Combined verdict and action plan

The branch is in **good shape for its size** — the heavy MathML/#2829 work verified
byte-identical against Perl on its witnesses, the infra changes hold their
invariants (log anchoring, status emission, no deadlocks), and no finding
invalidates a whole commit. But the review surfaced a clear pre-PR fix list,
dominated by two themes: **silent-degradation paths** (against the project's
fail-toward-flagging rule) and **live-confirmed one-line output bugs**.

> **STATUS 2026-07-03: ALL items below are COMPLETE** — must-fix 1-7,
> should-fix 8-13, and the actionable backlog (fixture set, reproducer
> promotion, KPE entries, contract docs, corner-case batches) landed as the
> `a22780aceb`..HEAD commit series; suite green + workspace clippy clean
> throughout. Remaining OPEN (deliberately): the lxDeclare dead-predicate
> class (own SYNC_STATUS worklist item), the bbl/bib precedence fixture and
> a cyclic-box unit test (low-value residual test debt), the F5 linebreaker
> release-time decision, F19 (math parser), and the corpus-scale 1k/10k A/B
> (run post-fleet). Notable extras found while fixing: the cluster-test
> harness lacked the binaries' contrib dispatcher (mhchem et al. were
> invisible to tests); `touch latexml_oxide/build.rs` re-runs fixture
> discovery without a full cargo clean.

**Must fix before the PR / July-5 rebuild (small, high-confidence):**
1. `atompair_spacing`: add the missing `("Inner","Punct") => -1` cell (+ unit-test the full Inner row) — live-confirmed churn on matrix-then-punctuation.
2. FRACOP/ENCLOSE: wire the CTX_COLOR fallback that the F8b commit message claims (an edit-script abort silently dropped those hunks); fix the stale comments. Live-confirmed black-fraction-bar-in-red-text.
3. Delete the committed debug `eprintln!` in the title-promotion fallback; constrain the second-pass anchor to non-sectional context.
4. Emit `Error!` in the silent degradation arms: re-entrant Alignment absorb-skip (document.rs:674) and the e6347fd880 revert/absorb arms.
5. Guard the 4 remaining `Node::new().unwrap()` sites in parser.rs (same pressure class as the fleet panics f0a8847c07 fixed).
6. `find_file`: `ends_with(".bib")` dot fix (unlocks cb8b648784's own fallback).
7. ORCID unwind: `unlink()` before `add_child` (legacy-libxml2 double-free).

**Should fix before July-5 (bounded, medium):**
8. fvextra INLINE `\Verb` breakanywhere guard (or STABILITY_WITNESSES entry) — live repro hangs on HEAD.
9. read_balanced crossing: gate on mouth KIND, not the autoclose bit (truncated `\input` currently silently absorbs parent content).
10. `.bib` author split: brace-depth-aware + newline-tolerant; Warn + resync on unbalanced entry.
11. cmml: bind Perl's cmml_top context in `convert_to_cmml`; CTX_FONT fallback in `stylize_ci_content`; register/uniquify minted share ids.
12. kpsewhich memo: clear in prepare_session (persistent-worker cwd staleness).
13. tabularray: bare-colspec shorthand arm + total-column/depth caps.

**Backlog (document, fixture, or upstream):**
- Missing-test debt called out in all three clusters — highest value: a MathML golden set (colored frac/cancel, array-comma, cfrac, --contentmathml pair), a `\ce{H2O}` fixture, noexpand reproducers promoted to suite, bbl/bib precedence fixture, cyclic-box guard test.
- lxDeclare dead-predicate class (pre-branch, golden blesses 51 vs Perl 84) — reopen as its own worklist item.
- `\string` of noexpand-family tokens leaks 0x01 (illegal XML); KNOWN_PERL_ERRORS candidates: `\cfrac[l]` (Perl drops it too), LookupDimension macro-path (already #41).
- pin!/SymStr arena-reset contract: document at `reset_thread_engine` + consider a poison debug_assert.
- Corpus-scale differential A/B (1k now at low nice, 10k post-fleet) remains the strongest whole-branch check — none of the above findings would be visible to the fixture suite.
