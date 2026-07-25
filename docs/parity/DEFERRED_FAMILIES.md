# Deep deferred families — parked work

> Lifted out of `docs/SYNC_STATUS.md` on 2026-07-25. Large or shared-machinery
> items, each needing a dedicated session. Parked deliberately: read the entry
> before starting, several carry explicit "do NOT start" directives.

## Deep deferred families (parked — large or shared; dedicated sessions)

- **Native `.bst` interpretation — DEFERRED (pending plan, ~a few months out; do NOT
  start work that requires reading `.bst`).** arXiv's bibliography convention is codified
  in `ar5iv.sty`: LaTeXML prefers a ready-made `.bbl` and, only if none is present,
  interprets the `.bib` itself into XML internally (its own `MakeBibliography` conventions).
  In production this is a non-issue — arXiv's AutoTeX runs `bibtex`, so a `.bbl` is present
  and the conversion reproduces the PDF. The gap only appears when a conversion sees
  `.bib` + `.bst` but **no** `.bbl` (e.g. a standalone/manual run that skips `bibtex`):
  the `.bib`-direct fallback cannot reproduce the document's `.bst` output, because we do
  not read `.bst` yet. **Witness: arXiv:2605.16562** (LNCS, `splncs04.bst`). With a
  `bibtex`-generated `main.bbl` present, the bibliography matches the PDF exactly — PDF sort
  order, inline `\url`/`\doi` links, no "External Links:" label, corporate author rendered
  "W3C Math Working Group". Without the `.bbl`, the `.bib`-direct path still diverges from
  the PDF in ways that genuinely require the `.bst` (DEFERRED): LaTeXML's own alphabetical
  sort (different order from splncs04), "External Links:" prefixes instead of inline links,
  and DOI shown as bare text (`10.48550/...`) rather than a `https://doi.org/...` link.
  These are inherent to synthesising a bibliography from `.bib` without the `.bst`, not
  formatting bugs. **Resolution:** until native `.bst` interpretation lands, rely on
  `bibtex`/AutoTeX producing the `.bbl` (production already does); no latexml-oxide change.
  To reproduce: `latex main && bibtex main`, add `main.bbl` to the source, re-convert →
  matches PDF; remove it → diverges as above.
  NOTE: two *native-pipeline* bib bugs surfaced by the same witness were genuine and have
  been FIXED (they did NOT need `.bst`): (1) the duplicate Note/External-Links bibblock
  (`8ffca54713`); (2) brace-protected corporate authors mis-split into initials
  ("{W3C Math Working Group}" → "W. M. W. Group") and the `@inproceedings` `booktitle`
  dropped to a "See ," artifact — both from the simplified `.bib` parser
  (`convert_bib_file_to_xml`) and the lightweight XPath matcher in `document.rs`
  (value-less `[@attr]` predicate treated as always-true; `split('/')` fragmenting a
  predicate's `../`). Fixed: corporate-author detection in `parse_bib_authors`, and a
  bracket-aware / existence-checking `findnodes_by_traversal`.

- **`Fatal:Stomach:Recursion` (43 cortex Rust-service fatals) — TRIAGED 2026-06-28,
  mostly SHARED / Rust-better; ~1 Rust-only over-fatal DEFERRED (deep core).** Two
  guards in `stomach.rs`: the box-cycle "Infinite digestion loop" (9 papers,
  stomach.rs:1040) and the token-stack-depth "Excessive recursion(?)" (28 pkg-loading
  + 6 box/thm, stomach.rs:1343, `MAXSTACK=200`). **Same-host Perl parity on an 11-paper
  sample: ~10/11 SHARED** — the box-cycle/digloop papers (1906.06902, 1810.02304,
  1911.00254, 1911.11563, 2605.27339) **HANG in Perl 50–94 s** while Rust fail-fasts in
  <1 s via the guard (**Rust strictly better**); others (1809.00641, 2103.12717,
  1409.4048, 2011.08422) fail in BOTH. **1804.01117 (svjour3) was thought Rust-only but
  is actually SHARED — see the corrected deep-dive below (Perl `--includestyles` hits the
  identical readBalanced failure).** Crucially the limit
  **matches Perl exactly** (`Stomach.pm:159 $MAXSTACK=200`, identical guard at L175) —
  so it is NOT a mis-set cap; do NOT raise `MAXSTACK` (diverges from Perl and lets genuine
  infinite recursion run). The guard is doing its job — this category is a Rust **stability
  win**, not a bug cluster.
  **DEEP-DIVE of the lone Rust-only case 1804.01117 (2026-06-28): it is NOT a
  stomach-accounting bug — it is a tikz/pgf cascade.** Full stack capture: the top ~170
  frames are `{ \bgroup { \bgroup …` piled up by **`\pgffor@expand@list`** (pgffor's
  `\foreach`), immediately after `Error:pushback_limit:Timeout … loading binding for
  'tikz.sty'`. Rust fails to load the `tikz.sty` binding (pushback-limit), leaving
  `\foreach` in a broken state that floods the digestion stack → `Stomach:Recursion`;
  Perl loads tikz fine and never gets there. (The earlier "Rust digests packages deeper"
  hypothesis was WRONG.) Minimal `\usepackage{tikz}`, the full preamble package set, and
  `tikz`+`\foreach` in the body all load CLEANLY — the binding-load pushback only triggers
  under the paper's specific complex state. **FULLY ROOT-CAUSED 2026-06-28 (a 2nd deep
  dive) — it is NOT tikz/pgf either; it is a Rust `read_balanced` bug in xint.** The
  trigger is **`--preload=ar5iv.sty` + `xintexpr` (loaded before pgfmath/tikz)**. ar5iv
  (INCLUDE_STYLES) RAW-loads xint; `xintexpr`'s load of its built-in float functions
  (`\xintdeffloatfunc`, e.g. xinttrig's `@sind`) runs `\xintexprSafeCatcodes` (a
  `\begingroup`) then `\XINT_NewFloatFunc`/`\XINT_NewExpr` (xintexpr.sty:4721) whose
  body-compilation does a balanced read that goes UNBALANCED ("readBalanced ran out of
  input in an unbalanced state" + "Attempt to close boxing group").
  **✅ SURPASS-PERL LANDED 2026-06-28: 1804.01117 now converts FULLY under
  `--preload=ar5iv.sty` (0 Error/Fatal, 423 KB HTML, renders cleanly with `--css=ar5iv.css
  --nodefaultresources --path=~/git/ar5iv-css/css`; 463 native MathML nodes, 0 degraded
  body nodes). Perl LaTeXML still DEGRADES to a 459-byte error stub here** (`latexml
  --includestyles` → 26 errors, the IDENTICAL `readBalanced ran out` at xinttrig.sty:350),
  so this is a genuine beyond-Perl win. The chain: ar5iv (INCLUDE_STYLES) raw-loads xint;
  `xintexpr` does `\edef\X{\scantokens{...}}` where `\scantokens` opens an autoclose
  "Anonymous String" mouth MID-`\edef`-body and the `\edef`'s closing `}` is in the PARENT
  file. The fix is two-part, both faithful to tex.web `get_next`/`get_x_token` §362-365:
  (1) **`read_balanced` now CROSSES autoclose mouths** (gullet.rs `None =>` arm: close the
  exhausted autoclose mouth and resume the parent instead of `break`-ing unbalanced — the
  same crossing `read_x_token` already does; dump-neutral, suite 1491/0). This kills the
  `\xintexprSafeCatcodes` `\begingroup` leak → no "Attempt to close boxing group" → no
  TokenLimit cascade. DELIBERATE divergence from Perl (Gullet.pm:466 `last`s here and so
  also fails this input). (2) the prior-committed transient-`\noexpand` arg-capture decode +
  per-token `\special_relax` family + native `\Ucharcat` (see
  [[ucharcat-char-generate-noexpand-2026-06-28]]) which eliminated the `\XINT_expr_var_!`
  expr-compiler cascade.
  **Residual (HARMLESS, package-load-time only): 112 `Warning:expected:<number>` during
  xinttrig's `\xintdeffloatfunc` compilation** (56× `\the` seeing `$`, 56× `\romannumeral`
  seeing the f-stop `\special_relax\XINTusefunc`, all inside the "Anonymous String"
  scantokens mouth). xint's compiled expression token-stream is slightly MISALIGNED vs real
  xint, so a number scan lands on the f-stop. **Zero body impact** — this paper only
  `\usepackage{xintexpr}` and never evaluates an expression in the body. Full xint
  expression *evaluation* fidelity (so a real `\xintthefloatexpr sind(30)` computes the
  correct value, not just "doesn't crash") is a deeper, separate surpass layer — **parked**.
  **LONG-TERM FIDELITY FOLLOW-UP (user-flagged 2026-06-28):** the ar5iv rendering is a fair,
  successful conversion but not yet pixel-perfect — improve the *fidelity* of **subfigures
  and listings (reflow)**. Tracked here as a long-term task (not a correctness bug; the page
  is far better than the prior broken/Fatal state). Repro + full bisection history in
  `docs/reproducers/xintexpr_pgfmath_ar5iv_pushback.tex`. The Stomach:Recursion category
  itself still has **zero genuine stomach bugs**.

- **1610.00974 step-3 (global p{}→VBox) + cluster-B — ✅ LANDED 2026-06-22, NO
  LONGER DEFERRED.** See "Landed this session" above. p{}/m{}/b{} columns now build
  the cell as Perl's `\lx@tabular@p` inline-block (VBoxContents); p/m/b `<td>`
  `align="left"`; **cluster-B FULLY RESOLVED**; fixes 1510.07685. Commits
  `f65b80c1c2` / `eb978df5a9` / `1867f17da9` (+ box-model `7545e07fd6`). NOTE: the
  `collcell`/`\collectcell` undefined seen in some table papers is PARITY (both
  engines default `notex=1`/`INCLUDE_STYLES=false`, so neither raw-loads
  `collcell.sty`; the `--quiet` Perl "0 errors" was a display-suppression artifact —
  use verbose Perl).
- **`expected:id` cmml dangling-XMRef tail** — MathFork/split content-arm xml:id
  duplication; the last live `expected:id` class. See
  `EXPECTED_ID_XMREF_DESIGN_2026-06-08.md`. **★ CANONICAL WITNESS FIXED AT THE ROOT
  (2026-06-26q, LANDED on `class-b-xmref`):** the grammar rule `statements punct
  statement vertbar statements => vertbar_modifier_listlhs` makes a comma-list left
  of a conditional bar parse (`a,b|c` → `list@(a, conditional@(b,c))`, Perl-exact),
  so the witness's aligned `\Pr(s_A,s_B|\Omega)` arg parses → refs RESOLVE, dual
  PRESERVED. cb_repro & full witness `2311.01600` → 0 danglers; suite 1470/0; also
  fixes the standalone `a,b|c` aside. **RESIDUAL CHARACTERIZED (2026-06-26r):** the
  fix closed the "No node found"/DANGLING sub-case (canonical witness). The
  DOMINANT remaining `warning/expected/id` cortex cluster (**370 tasks**) is a
  DISTINCT class — `Missing idref on ltx:XMRef … _xmkey is `` ` (keyless XMRef, no
  idref, document.rs:3238), NOT a dangling idref — Rust-only (0704.2334 Rust 2 /
  Perl 0), from `\quad`/`\;`-separated **formulae/lists** with function-fence
  applies; context-dependent; root = `formulae_apply` content ref whose key never
  reaches the presentation item's top node (structure captured 2026-06-26t: a
  `formulae@` dual with a trailing bare `XMRef _xmkey=XM291` and no presentation
  top carrying XM291; the extend path doesn't clone `right`, so it's a subtler
  nested-relation/`\lx@dual` interaction). **SEVERITY: content-MathML QUALITY gap,
  NOT corruption** — the keyless ref has no idref so the prune sweep skips it; it
  survives with the faithful `Missing idref` Warn, schema-valid, no content dropped.
  Lower-priority cMML-polish item for the deferred math-fork session; the two
  higher-severity sub-classes (Class-B dangling + content-corruption) are FIXED.
  **★ COMMON SUB-CAUSE FIXED (2026-06-26v):** the keyless bare ref is a
  distribute-dual extend interaction — `distribute_list_relation` makes a
  `formulae`-content dual with a relation-`Apply` (non-Wrap) presentation; the
  formulae/list extend paths then push a content ref but silently skip the non-Wrap
  presentation → bare ref. Fix = gate the extend on a Wrap presentation (fall
  through to a fresh dual otherwise). Witnesses 0704.2334/0705.0790/0707.1173 →
  0 Missing-idref; suite 1471/0; regression `cluster_formulae_distribute_no_bare_ref`.
  PARTIAL: 0707.1339 still emits 2 (a different sub-cause). **QUANTIFIED 2026-06-22 (pre-fix): this WAS the
  #1 remaining Rust-only divergence** — `warning/expected/id` is **1005 cortex
  tasks** ("Cannot find a node with xml:id='S…E…m1.N'" from
  `latexml_math_parser/src/parser.rs:2840`; math-node ids, so genuinely the
  content-arm/MathFork XMRef cluster). It's a large Rust-only WARNING excess vs
  Perl (e.g. 0704.3530 Rust 152 vs Perl 9 warnings) — NOT parity. The prime
  candidate for the deferred content-MathML dedicated session; do NOT pick at it
  piecemeal (user directive). **FULLY DIAGNOSED + DE-RISKED 2026-06-26** (branch
  `class-b-xmref`, research-only, no code): same-host confirmed (0803.3810 Rust 51
  vs Perl 0), exact 6-dangler witness `2311.01600` (now `/data/arxiv/2311/`),
  Perl's target tree captured, a ~15s repro, and ALL peripheral fixes (clone/move/
  `.mf`/combos) empirically RULED OUT — the sole fix is the core post-parse
  preserving the structural XMArg ids (it rebuilds a fresh result tree → fresh
  per-row `{group}X.m1.*` ids, stranding the build-time `{group}.m1.*` refs). The
  re-id is in a distributed parse/install path (the `parser.rs:1354` reinstall is
  NOT it). **PIN SHARPENED 2026-06-26 (notes 2026-06-26i/j) — full end-to-end
  runtime trace; exact unrecord site identified by backtrace.** The danglers are
  the `\Pr` (physics-pkg `I_dual`) CONTENT-arm arg refs; the arg material is still
  present (ref merely dangles → any prune/drop is content loss, RULED OUT as a
  cheat). The arg XMArg (`_xmkey="1"`, `xml:id`) is **swallowed by the
  `parse_single` reparse of its ancestor presentation XMWrap** (`unrecord_node_ids`
  ← `parser.rs:1501`), NOT parse_rec'd standalone — so the working `parse_rec`
  id-transfer (`:1136-1196`, which heals the sibling dual args keys 2,3,5,6,7,8)
  never applies. RULED OUT (all empirically): prune/drop, `XProps` xml:id capture
  (dual not ingested via `From<&Node>`), `_xmkey` re-resolution + remap (parser
  REGENERATES keys; `XM::Arg` drops the build key). LANDMINE: the reparse
  orphan-detection (`:1502-1528`) is dead-code via the `@xml:id` namespace footgun;
  naively fixing it ACTIVATES a content-losing `__LOSTNODE__` drop. Two viable fix
  designs (key-carrying `XM::Arg` + re-point handler; OR cross-recursion old↔new
  `_xmkey` snapshot) with failure modes in the design doc. **DEFINITIVE ROOT
  (2026-06-26k, proven vs Perl source):** the ASF-vs-RecDescent node-identity
  divergence — Perl `parse_rec` returns an array-tree EMBEDDING the real parsed
  child nodes, so `appendTree` preserves their `xml:id`; Rust's ASF `into_xmath`
  REBUILDS fresh nodes (XM::Apply), so a re-materialized (non-`XM::Lexeme`)
  referenced target loses its id and the content XMRef strands. Faithful fix =
  identity-preserving `into_xmath` for non-leaf referenced nodes (reuse the input
  DOM node, like the leaf `XM::Lexeme` arm); LOSTNODES re-point is the pragmatic
  alternative. **TRIGGER ISOLATED (2026-06-26l):** the dangler is a downstream
  symptom of a CONTEXT-DEPENDENT **parse FAILURE** of the `\Pr` argument
  (`s_A,s_B|Ω_{len=k}` → `parse_single` returns `None`), so the `parse_rec` id-transfer
  (which heals the args that DO parse) never runs and the ancestor reparse strands the
  ref. Confirmed: the SAME arg parses standalone (0 danglers) — only the paper's
  preamble makes it fail in-context. Two fix axes (both dedicated-session): (A)
  parse-coverage (make the in-context arg parse; relates to the open VERTBAR/comma-list
  asides); (B) failure-robust id preservation via reused-leaf correspondence
  (`record_replacement(oldXMArgId, newTopId)` re-point, content-preserving). Precise
  repro + ruled-out approaches in `EXPECTED_ID_XMREF_DESIGN_2026-06-08.md`
  (2026-06-26a–o). The dedicated session = fix axis A or B + full math-fixture/corpus
  validation. **PARTIAL FIX LANDED (2026-06-26o, `class-b-xmref`):** an
  operand-protection guard in `prune_dangling_split_xmrefs` stops the broad `^S\d+`
  sweep from DROPPING `\Pr` content-arm arg refs (which emitted a malformed
  `apply(probability)` = silent content loss for section-numbered aligned `\Pr`);
  it now PRESERVES the arg (dangling, closer to Perl). 1469/0, clippy clean, does
  NOT re-flood wp3, regression test `cluster_xmref_pr_arg_not_dropped`. Does NOT
  make refs resolve — that is still the dedicated session (the leaf-LCA re-point,
  design B, works mechanically but collapses the dual; the faithful fix needs a
  CONTENT-branch arg copy, Perl's `.mf` scheme, via `rearrange_lone_ams_aligned`).
  **ROOT CAUSE + EXACT FIX FOUND (2026-06-26p) — AXIS A now recommended.** Bisected:
  only `\Pr(a,b|c)` (comma-list-LHS conditional) dangles; `\Pr(x)/\Pr(a|b)/\Pr(a,b)`
  resolve. The grammar's lone VERTBAR-modifier rule is `statement vertbar statements`
  (single LHS, `builder.rs:447`), so `a,b|c` doesn't parse → arg fails → ref strands.
  ONE-LINE fix `statements vertbar statements` TESTED: standalone `a,b|c` parses
  (fixes the open VERTBAR aside), witness → 0 danglers, refs **RESOLVE**, dual
  PRESERVED (faithful, = Perl's path). BUT regresses abs-value (`a|a|` →
  `conditional@(a,a)` not `a*|a|`; abs-value-vs-conditional ambiguity defeats
  `prefer_fewer_conditionals`). Reverted. Targeted fix = a `comma_statements`
  nonterminal (≥1 comma, not subsumed by `statements`) so the rule fires only on
  genuine lists, OR a pruning tweak — dedicated math-parser session. Axis A produces
  the genuinely-correct tree; preferred over the deep rearrange materialization.
- **xy-pic `svg:path` / curve cluster** (1501.03690) — shifted-arrows `svg:path`
  in `ltx:text`; mode-frame cascade root.

**SHARED (both engines fail — match Perl; do NOT "fix" by downgrading):**
- **1804.01117 xint raw-load** — both raw-load xint and fail (plain: both stub,
  byte-identical). The Rust stack-overflow crash is FIXED (gullet `stack_guard`,
  configurable via `latexml_core::stack_guard`). Deep xint emulation parked.
- **mode-frame auto-close cluster** (1611.04940, 2009.05630, 1702.06692,
  1702.02037) — a theorem env opened via its bare begin-command with no matching
  `\end…` leaks the mode-switch frame; Perl `Stomach.pm:343-376` errors
  identically. A graceful auto-close would *surpass* Perl (beyond-parity R&D).

---
