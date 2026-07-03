# Engine Sync Status — Active Worklist

> **DO NOT downgrade Errors to cheat the task.** If Perl LaTeXML converts a paper
> without a downgrade, the Rust translation must match by improving the core
> engine — never by silencing diagnostics. New downgrades require explicit proof
> Perl emits the same severity on the SAME paper, else they hide a real gap.
> (User directive 2026-05-15.) Always classify with `latexml --verbose`, never
> `--quiet` (which hides Perl's `Error:` lines); cross-check pathological inputs
> with `pdflatex`.

> **This file is the BRIEF ACTIONABLE LIST.** The day-by-day fix log and
> completed-task records are NOT kept here — they live in `git log` and
> `docs/archive/`. **When you close an item, delete it here** (git keeps the
> record). Last compaction: 2026-06-21.

## Current status

- `cargo test --tests`: **1506 / 0 / 0** (on `ar5iv-2606-prep`; see "Landed this
  session" entries below).

### Landed this session (2026-07-02, on `ar5iv-2606-prep`) — upstream PR #2829 "Framing" ported in full

Faithful translation of brucemiller/LaTeXML#2829 (merged upstream 2026-07-02,
ref `d666adf8` — post-dates the archived U1-U11 sync window):

- `LookupDimension` coercion widening (`state.rs::lookup_dimension_cs` →
  `Option<Dimension>`): obvious-dimension strings parse directly, tokenized
  specs resolve registers or read multi-token dimensions. **KNOWN_PERL_ERRORS
  #41**: the upstream rewrite unintentionally LOST the macro-body-read path
  for `\def`-ized lengths (warns + 0 now) — Rust keeps it (deliberate
  divergence, covered by the arraycolsep cluster regressions).
- `framedProperties` helper (`tex_box.rs`, pub via prelude): consistent
  framed/framecolor/cssstyle attributes + pad* Dimension properties;
  `\lx@framed` now takes `OptionalKeyVals:framed` (margin/rule).
- `insertBlock` filters properties to the ltx:figure attribute set.
- `\makebox`: alignment map gains `c`→center, `s`→stretched (was
  "justified"); width without explicit alignment defaults to center.
  `\@framebox` properties now via framedProperties (this RESOLVES the
  KNOWN_PERL_ERRORS #35 `$sep ne '3.0pt'` bug upstream — entry updated), and
  the single-child unwrap is skipped when an explicit width was given.
- framed.sty ({framed}/{oframed}/{shaded}/{shaded*}/{snugshade} via
  framedProperties — shaded family now carries framed=rectangle+framecolor;
  {leftbar} direct properties, its `color` filtered out by insertBlock;
  {titled-frame} margin 0pt/rule 2pt), ntheorem `\lx@addframing` (copies
  framed/framecolor/cssstyle, cssstyle SET not merged), soul `\textul`
  (framedProperties with font-color fallback → framecolor always present).
- CSS: `.ltx_framed { padding:3pt; }` default removed (padding now supplied
  per-construct via framedProperties).
- All 6 upstream fixture updates mirrored: our re-blessed
  tabbing/mathtools/marvosym/soul/framed/ntheorem golden diffs are
  **byte-identical to the upstream fixture diffs**. Suite 1506/0.

### Landed this session (2026-07-02, on `ar5iv-2606-prep`) — MathML-post exhaustive line audit, wave 1+2

User-commissioned ("the Rust translation wants to be an exhaustive port") audit
of the MathML post-processors, opened as the living worklist
[`docs/MATHML_POST_LINE_AUDIT.md`](MATHML_POST_LINE_AUDIT.md) (verdicts for all
60 MathML.pm subs + 197 DefMathML registrations + sibling files; F-numbered
findings). Ten findings LANDED same-day, each witness-verified byte-identical
against same-host Perl (commits `3ab9ce3cb3`…`e577613fb1` + cfrac):

- **F1** author-spacing (lpadding/rpadding) carry into the spacewalk
  (astro-ph0001001 witness — the user-reported lost-spaces bug).
- **F2** dead duplicate spacewalk deleted from `mathml/mod.rs` (three-way
  table verification first).
- **F10/F12/F13** pmml-wrapper parity (menclose/class/_role), Apply:ENCLOSE →
  `m:menclose` (`\cancel`), FRACOP verbatim linethickness/mathcolor/bevelled.
- **F18** `nth-root` arg order: all THREE consumers had (degree, radicand)
  swapped — `<mroot>` was spec-backwards, cmml `<degree>` wrapped the
  radicand, unicode_math used the radicand as index.
- **F7** mathstyle→`m:mstyle` propagation (stylemap tables, needsMathstyle,
  XMApp/XMArray/bigop/script wraps, mode-sensitive entry baseline
  display↔text) — `\tfrac`/`\dfrac`/`\displaystyle` sizing.
- **F8** faithful mo styler (opdict xor-emission, largeop/movablelimits/
  symmetric, minsize/maxsize stretchyhack, %→em size resolution, mathsize on
  all token types). **F9** `pmml_maybe_resize` wired at all five call sites.
  **F4** `fmt_em` byte-parity (`%.3f`, trailing zeros kept).
- **F3/F6/F11** spacewalk rewritten as Perl's stream algorithm (mrow/script
  unwinding into the pair stream, negative-target mpadded via string metrics,
  mspace merge, both-mo target/2 split; Hint widths normalized to em,
  `_ignorable` + filter_row).
- **F14** content-MathML: multirelation → `m:and`-chained pairwise applies
  with `m:share` (generateNodeID port), or-composition, and STYLIZED ci
  content (plane1: `<ci>𝑥</ci>` — formerly raw ASCII on every cmml
  identifier), decorated symbols, Perl-regex integer test.
- **F16** OperatorDictionary Content_form/fence tables REGENERATED verbatim
  from Perl's range strings (machine-parsed, non-overlap asserted) — closes
  the arrow/negation codepoint holes and the U+2A50 misclassification.
- **F15** `do_cfrac` unrolling behind the `cfrac-inline` gate + the amsmath
  `\cfrac` binding rewritten to Perl's trampoline (capture surrounding
  mathstyle once, nested reuse, no size compounding).

- **F8b** inherited-context bindings (same day): `{\color{red}$a+b$}` math
  now colors its tokens (was black — visible arXiv bug class); + a latent
  rust-libxml misaligned-ns-read crash fixed in `find_inherited_attribute`.

- **lxDeclare dead-predicate class — CORE FIXED 2026-07-03** (PR_READINESS
  cluster C): dead `@font`/`@meaning` XPath predicates replaced by Rust-side
  font-CLASS filtering (`_declare_font` → declare_node_matches) and
  `(@meaning|@name)` predicates; replace-rules now carry the same
  declare-side filter (new `declare_filter` rewrite option — kills the
  latent delete-the-wrong-sibling); untagged `scope=section` gates the fast
  path via an explicit scope_prefix field. declare.xml golden: 51 → 67
  decl_id, strictly additive (0 lost marks), vs Perl's 84. RESIDUAL: the
  remaining ~17 belong to OTHER pattern families (S4/S6/S7: literal-base
  variants, replace-related XMDuals) that need their own compile arms in
  `compile_declare_pattern` — each now Warns as unrecognized instead of
  silently skipping.

Open queue lives in the audit doc: F17 misc, F14 share-suffix wiring,
**F5** linebreaker decision (Perl gates on `--linelength`, default OFF →
feature gap, not production divergence), **F19** math-parser
`\mathop{=}\limits^{def}` mis-parse (XMWrap vs script application — math-
parser worklist). Method traps recorded in the doc (installed Perl 0.8.8 lags
the reference tree; trace producer-vs-consumer before patching post).

### Landed this session (2026-07-02, on `ar5iv-2606-prep`) — live-run fatal mining: 2 panic sites, `\dabar@`, plain-`\+` retraction

Mining the in-flight full-arXiv rerun's fresh fatals (6.9k at ~32% corpus) produced
four fixes, each witness-verified against same-host Perl:

- **Graphics worker-join panic (15 papers)** — `graphics.rs` `join().unwrap()`
  escalated a pressure-induced worker-thread panic into a whole-conversion
  `Fatal:panic`. Now degrades per the function's own design: payload surfaced as
  `Error:imageprocessing:worker_panicked`, survivors' outcomes kept. Witness
  1811.01777 converts clean standalone (pressure-dependent, not paper-dependent).
- **`parser.rs` `Node::new().unwrap()` panic (2 papers)** — allocation failure in
  kludge-script restructuring now records `Error:misc:allocation` and returns the
  base un-scripted (`Result` threaded `new_script_node` → `kludge_scripts_rec` →
  `parse_kludge`); a genuine OOM then dies via the designed RSS watchdog.
- **`\dabar@` runaway (31 papers)** — KNOWN_PERL_ERRORS #40: real `amsfonts.sty`
  defines the dash glyph; both bindings omitted it, and author copies of the
  `\xdashrightarrow` snippet `\@whiledim`-loop on a 0-width `\sbox` of it forever
  (Rust's real label widths → `Fatal:Timeout:TokenLimit`; Perl escapes only via
  all-zero box widths). Binding now defines it (`╌`); witness 1705.09248
  180s-Fatal → completes with 1 error (same class as Perl's 2); pdflatex ground
  truth compiles. Reproducer `docs/reproducers/xdasharrow_dabar_whiledim_loop.tex`.
- **plain-`\+` retraction (Rust-only fix; part of the 516-paper
  `\lx@begin@alignment` TooManyErrors family)** — real LaTeX (INITEX-based) never
  defines plain.tex's `\+` (= `\tabalign`), but Rust's latex layer inherited it
  from the plain dump, so an author typo `\+` in math expanded into `\halign` and
  detonated a 102-error mode cascade (witness cond-mat0001412; Perl: 1 undefined
  error). The latex format loader (`latex.rs`, at the "kernel layer complete"
  seam after the dump/base branch) now retracts the inherited definition
  (guarded on the body still being plain's bare `\tabalign`; new
  `state::remove_meaning_global`). Witness now: exactly 1 error `undefined \+`
  — byte-parity with Perl. Watch the same class for other plain-only macros
  (`\tabalign` invoked directly, etc.) if cascade signatures persist in the
  next run.

Triage byproducts: `\tikzcdmatrixname` PushbackLimit cluster (345 papers) verified
**PARITY** (witness 1304.2913: Perl `Fatal:too_many_errors` in pgfmath, 44 s) —
known tikz-cd deep-divergence territory, not chaseable; `never_completed` (1,069)
spread evenly across months (governor sheds/hangs, overlaps STABILITY_WITNESSES).

**Plain-layer leakage audit (same day, follow-up to the `\+` fix).** The
layering is Perl-identical (Perl's `TeX.pool.ltxml:23` also runs
`LoadFormat('plain')` under LaTeX.pool); the divergence is content — Perl's
plain layer is the hand-curated `plain_base.pool.ltxml`, Rust's is the dump of
REAL `plain.tex`. Name-diff (plain-dump CSes − CSes mentioned anywhere in
Perl's engine pools − latex-dump-redefined CSes, `LC_ALL=C`): **55 survivors**,
coherently two subsystems plus stragglers: (1) the plain **tabbing machinery**
(`\tabalign`, `\settabs`/`\sett@b`/`\s@tt@b`/`\s@tcols`, `\cleartabs`, `\tabs`,
`\tabsdone`, `\tabsyet`, `\t@bbox`/`\t@bb@x`, `\m@ketabbox`, `\us@*`, `\if@cr`+
friends) — the `\+` family; (2) the plain **output routine** (`\plainoutput`,
`\pagebody`, `\pagecontents`, `\makeheadline`, `\makefootline`,
`\dosupereject`, `\@ins`, `\if@mid`/`\ifp@ge`/`\ifr@ggedbottom` + setters);
(3) inert stragglers (`\Orb`, `\oldstyle`, `\preloaded`, `\getf@ctor`, `\m@g`,
`\p@renwd`, `\if@`, `\@nother`) and record-format artifacts (`%NN`,
`\skewchar\<font>`, `count/dimen/skip254`). Live-run evidence: zero errors key
on any of these names (only `\+` was a typo-magnet; the rest execute only when
intentionally invoked and are silent if they work). **DECIDED 2026-07-02
(user): keep and watch** — the remaining tabbing entry points stay defined as
beneficial plain coverage; revisit only if next-run cascade signatures
implicate them. Regenerate the list with the three-set diff above.

**Resolved en passant (2026-07-01 lxDeclare session):** the long-standing
"DefMathRewrite `\WildCard` subscript bug" (wildcard-subscript rewrite not
firing; `math/simplemath` fixture encoded the buggy `Unknown@()` output) was
fixed by the `\lxDeclare` B+C parity work (`dd226d1973`, `d74529d9eb`,
`786d9ed89d`) — simplemath is now byte-identical to same-host Perl and the
golden was re-blessed.

### Older session logs (2026-06-22 … 2026-07-01) — ARCHIVED

Completed session entries, the slowest-100 batch triage, the finished
upstream-sync U1–U11 mission log, and the mined-out 2026-06 methodology
history now live in
[`archive/SYNC_SESSIONS_2026-06.md`](archive/SYNC_SESSIONS_2026-06.md)
(upstream-sync catalog also at
[`archive/UPSTREAM_SYNC_2767_to_2833_2026-06-26.md`](archive/UPSTREAM_SYNC_2767_to_2833_2026-06-26.md)).

## Methodology & the cortex cross-join

Working method (2026-06): **re-triage LARGE-error papers** (the single-error tail
is exhausted) → bisect the doc to the trigger line → verify Perl with `--verbose`
→ fix the divergence. Random sweeps are low-yield.

**Cortex agentic API (reads open, no token):** `http://127.0.0.1:8000/api`.
Recipe: `GET /api/reports/<corpus>/oxidized-tex-to-html/<severity>` → categories;
`…/<severity>/<category>` → per-`what`; `…/<category>/<what>` → paper list. Then
`GET /api/corpus/<corpus>/tex_to_html/document/<id>` for Perl status — a Rust-only
win is **Perl=no_problem/warning but Rust=error/fatal**. Corpus
`sandbox-arxiv-10k-shuffle`. URL-encode `\`→`%5C`, `^`→`%5E`.
## Math-parser / content-MathML gaps — DEFERRED to a dedicated session

> **User directive 2026-06-20: defer ALL content-MathML items to a dedicated
> session** (the math parser is a full Marpa-vs-RecDescent rewrite; these touch
> the parse-tree / content-MathML structure and want a focused regression
> budget). Notes kept here; do NOT pick at them piecemeal.

- **`f(a,b)` multi-arg flattening — FIXED 2026-06-22.** A KNOWN function applied
  to a paren comma-list now flattens: `\max(a,b)`→`maximum@(a,b)` (was
  `maximum@(vector@(a,b))`), matching Perl `ApplyDelimited`/`extract_separators`.
  Implementation was simpler than the planned grammar-rule approach: a post-parse
  spread in the `prefix_apply` ACTION (`semantics.rs`, helper `vector_tuple_items`)
  — when a function-role op (FUNCTION/OPFUNCTION/TRIGFUNCTION) applies to a
  `Dual` whose content is `Apply(vector, [refs])`, spread the items as direct
  operands instead of wrapping. No grammar/pruning change → NOT pruning-sensitive,
  zero fixture regressions. Scoped to known function roles, so unknown-`f` apply
  (`f(a,b)`→`f@(vector@(a,b))`) is untouched — the intentional divergence #18.
  Verified Perl-identical: `\max(a,b)`/`\gcd(a,b)`/`\min(x,y,z)`/`g(a,b,c)` +
  nesting/`\frac`/trailing-ops; suite 1466/0; regression test in
  `parse/functions`. (Known pre-existing aside: juxtaposed `\max(a,b)\min(c,d)`
  greedily reads `\max` over the product — a separate function-juxtaposition
  pruning issue, not this flatten.)
- **`f(x)` single-arg apply-vs-multiply** (most PERVASIVE divergence): for an
  UNKNOWN/undeclared symbol + paren arg, Rust reads *application*, Perl reads
  *multiplication* — `\Gamma(s)`→Rust `Gamma@(s)` vs Perl `Gamma * s` (likewise
  `\zeta(s)`, `\Phi(x)`, `f(x)`). A real fix must respect Perl's "only declared
  FUNCTION/known-operator names apply; bare letters multiply" rule; heavily
  pruning-sensitive.
  > **SURVEY 2026-06-22 (current-state + blast radius — groundwork, NOT yet
  > changed):** confirmed the split cleanly — KNOWN functions ALREADY match Perl
  > (`\sin(x)`/`\log(x)` → `sine@(x)`/`logarithm@(x)` in both); only UNKNOWN
  > symbols diverge (`f(x)`/`g(x)`/`P(x)`/`\Gamma(s)`/`\zeta(s)`/`\phi(x)` →
  > Rust `X@(x)` vs Perl `X * x`; `f(x+1)` → Rust `f@(x+1)` vs Perl `f * (x+1)`).
  > LEXER ROLE: unknown `f` = `role="UNKNOWN"`, `\max` = `role="OPFUNCTION"` — so
  > the apply-of-UNKNOWN (A) is separable from the known-fn flatten (B). BLAST
  > RADIUS of A is corpus-wide: 25 test fixtures, ~150 single-letter applies
  > (`f@(`×57, `d@(`×51, `g@(`×13, …) would flip to multiply — a sweeping change
  > that reshapes all math output. Because A is corpus-wide (even though
  > toward-Perl), it needed explicit scope sign-off; B (below) was the
  > contained first step (~5 fixtures).
  > **DECISION FINAL 2026-07-02: divergence #18 STANDS — `f(x)` leans toward
  > function application.** The toward-Perl flip was green-lit earlier the
  > same day, fully implemented (12/12 witness parity with Perl, ~22 fixtures
  > verified toward-Perl), and then **REVERTED on user review**: "f(x) is
  > almost always an application in common STEM use." The apply-of-UNKNOWN
  > reading is the settled intentional divergence (OXIDIZED_DESIGN #18,
  > re-affirmed). The reverted implementation is preserved on branch
  > `archive/fx-perl-parity-attempt-2026-07-02` (local) for reference — do
  > NOT re-attempt the flip without a fresh explicit user decision.
- **`[a|b]` / `[a \mid b]` bracket-conditional — FIXED 2026-06-22.** Was unparsed
  in Rust; now `delimited-[]@(conditional@(a,b))` matching Perl (`E[X|Y]` etc.).
  Root: the bare `a|b` conditional reduces only at statement level (not as an
  `expression`), so `[a|b]` had no fence rule — though `[(a|b)]` already worked.
  Fix: a surgical grammar rule `lbracket formula singlevertbar formula rbracket =>
  bracket_conditional` (`singlevertbar` also covers `\mid`) + a `bracket_conditional`
  action (semantics.rs) that builds the inner `conditional@(a,b)` (delimiter-less
  presentation) and wraps it in `delimited-[]` via the same `fenced` path
  `[(a|b)]` uses (ctxt reborrow for the two ref levels). Suite 1466/0, clippy
  clean, zero other-fixture changes; regression test in `parse/vertbars`. (The
  `E` in `E[X|Y]` stays `E@(…)` apply vs Perl `E * …` — divergence #18, preserved.)
- **`⁡` DecorateOperator over-insertion — FIXED 2026-06-22.** Presentation MathML
  emitted `⁡` (U+2061 FUNCTION APPLICATION) after operators that render as
  `<m:mo>` — `\nabla \phi`→`∇⁡ϕ`, `\partial f`→`∂⁡f`, and (pre-existing) `\sum_i
  a_i`→`∑⁡a_i`, `\int f`→`∫⁡f` — where Perl juxtaposes (∇ϕ/∂f/∑a/∫f). Perl's rule
  (MathML.pm `Apply:?:?`): insert `⁡` only when the op base is NOT an `<m:mo>` (a
  function identifier `f`/`\sin`/`\max` IS `<m:mi>` → keeps `⁡`). FIX
  (`latexml_post/.../presentation.rs`): new `op_base_is_mo` helper (descends
  msub/msup/munder/mover to the base); applied at the generic-apply site AND in
  `pmml_summation`; and removed `DIFFOP` from the big-op→`pmml_summation` route
  (Perl MathML.pm:702 `# Not DIFFOP`). Suite 1466/0, clippy clean; verified
  Perl-identical for ∇/∂/∑/∫/∏/⋃/lim + `\sin`/`\max`/scripted forms; only residual
  diff is the `f(x)` apply-vs-multiply (`f⁡(` vs `f⁢(`) — divergence #18,
  preserved. Regression test in `tests/post/opdecoration`.
- **wide-space PUNCT XMDual content-arm XMRef ordering**: `x^2\quad y` — the
  `\quad` (≥10pt) becomes a virtual PUNCT through `formulae_apply`, producing an
  XMDual whose content-arm XMRef siblings emit one slot off from Perl. Same
  MathFork/split content-arm xml:id family as the `expected:id` tail
  (`EXPECTED_ID_XMREF_DESIGN_2026-06-08.md`). NOT the rpadding path (thin spaces
  `\,` are Perl-faithful incl. NewScript transfer, `005716ff66`).
- **`\DeclareMathOperator` cluster — INVESTIGATED 2026-06-22, LOW-VALUE metadata,
  deprioritized** (`text=` and cMML already match): (a) Perl splits Math attrs
  `tex="\operatorname{Tr}…"` vs `content-tex="\Tr…"` (Perl defines `\Tr` *via*
  `Invocation(\operatorname,…)` + `revert_as=>'context'`); Rust defines it
  directly so `tex` keeps the user macro `\Tr` (arguably MORE source-faithful) and
  emits no `content-tex`. Matching Perl needs the deep `revert_as=>context`
  content-tex mechanism — high effort, metadata-only value. (b) The `name="Tr"`
  "gap" is NOT a bug: `def_math` (dialect.rs:1567) DOES infer `name` from the CS
  but DROPS it when `name == presentation` (line ~33) — a deliberate
  redundant-attr cleanup. `\Tr` (name "Tr" == content "Tr") drops it; `\argmax`
  (name ≠ "arg max") keeps it. Perl always emits it. Changing this touches the
  GENERAL def_math path (every math token) for cosmetic value → not worth it.
  (c) `\DeclareMathOperator*` `scriptpos` in display mode — the remaining
  candidate if revisited, but mode-dependent and niche. Whole cluster parked.
- **N-ary bare-operator listing — ✅ NOW WORKS (verified 2026-06-27); note was
  STALE.** `+,-,\times,\div` → `list@(+,-,*,/)` (Perl-exact); `+,-`, `+,+`, `a,+,b`,
  `++`, `+-` all parse and match Perl. An intervening fix (likely the comma-list /
  marpa-drain work) closed this. NOT an open gap anymore. The truly-remaining
  operator-script cases are narrower and finicky/context-dependent: `\Omega_{+,+-}`
  (a comma-list-of-operators in a SUBSCRIPT — Perl's subscript grammar parses it as
  `list@(+, absent + -)`, Rust's doesn't; note `+,+-` STANDALONE is PARITY-unparsed
  in BOTH), and operator-scripts where both parse but DIVERGE structurally
  (`a^{++}`: Rust `a^(list@(+,+))` vs Perl `a^(absent + +)`). These are the deferred
  math-fork session (subscript-content grammar + scripted-operator structure).
- **comma-list LEFT of a relation `a,b \in A` — FIXED 2026-06-22 (2-item path).**
  Was the wrong `formulae@(a, b∈A)` (∈ binding only `b`). Now the user-specified
  surpass-Perl **XMDual**: content **DISTRIBUTES** — `formulae@(∈(a,A), ∈(b,A))`,
  sharing XMRefs to the relop and RHS — presentation wraps the list as the
  relation's LHS — `Apply(∈, XMWrap(a,',',b), A)`. Implemented as a scoped
  transform at the end of `formulae_apply` (semantics.rs): when `left` is a bare
  (non-relational, non-Dual) item and `right` is a binary RELOP relation
  `Apply(R,[lhs,rhs])` under a comma, `distribute_list_relation` builds the dual.
  `x,y \le z`→`formulae@(x≤z, y≤z)`. The list-RIGHT `0<x,y`→`list@(0<x,y)`,
  all-relational `a=b,c=d`→`formulae@`, and bare `a,b`→`list@` all stay. Full suite
  1466/0, clippy clean, zero other-fixture changes; regression test in
  `parse/relations`. **Remaining (follow-up):** the 3+-item `a,b,c \in S` goes
  through `list_apply` (not `formulae_apply`) → still `list@(a,b,c∈S)`; the same
  distribution needs porting to that path.
- **relation with a list-RHS that itself contains a scripted relop**:
  `a \le b \quad \stackrel{?}{\ge} \quad c` → Perl `a <= list@(b, >=^?, c)`.
  **UPDATED 2026-06-27: no longer `ltx_math_unparsed` (stale)** — Rust now PARSES
  it as `fragments@(a <= b, >= ^ ?, c)` (the `\quad`-WIDE_PUNCT routes it through
  `formulae_apply`→`fragments@` rather than the relation-with-list-RHS shape). So
  it's now a STRUCTURAL divergence (fragments@ vs `a <= list@(…)`), not a parse
  failure. Lower-severity (renders) cMML-structure item; the scripted-relop atomic
  fix (`4a5ebf29f7`) cleared standalone list items.
- **`\underset`/`\overset` over an ARROW with a multi-token script**:
  `x \underset{n\to\infty}{\to} y` — the under-script reads `n@to@infinity`
  (apply) where Perl groups `(n to infinity)`. Same ARROW-as-applied-function
  family as `f(a,b)`.

CAUTION: new VERTBAR/fence grammar rules can collide with package-built
structures — always cross-check the affected fixture against Perl before
assuming a regression (the norm rule "regressed" physics_test, but Perl matched
the new output, so it was a parity *fix*).

## Open tasks (actionable)

### July-5 arXiv run — prep checklist (drafted 2026-07-02, user-approved sequence)

Ordered; items 1–3 are cross-repo and REQUIRED (user, 2026-07-02):

1. **ar5iv-css `glowup`**: merge the `glowup` branch (currently checked out at
   `~/git/ar5iv-css`, HEAD `3542c57` "ship committed lightningcss min bundles +
   automate the release") and **release a new ar5iv-css version**.
2. **Propagate ar5iv-css** to **ar5iv** (`~/git/ar5iv`) and **cortex**
   (`~/git/cortex`) — bump/vendor the released CSS in both.
3. **PR `ar5iv-2606-prep` → `main`** (user: later today, 2026-07-02) — parity
   fixes, perf audit + pin! sweep, fatal-mining fixes, docs consolidation.
4. ~~`f(x)` apply-vs-multiply dedicated session~~ — **CANCELLED 2026-07-02**:
   built, verified vs Perl, then reverted on user review; divergence #18
   (f(x) → function application) re-affirmed and stands. No math-output
   change ships in the July-5 binary from this item.
5. **After the current full-arXiv run finishes (~2026-07-04)**: rebuild
   `target/maxperf-cortex/cortex_worker` from merged `main` (fleet binary was
   deliberately NOT swapped mid-run).
6. **Smoke canvas** on the new binary (a few hundred mixed papers via
   `tools/benchmark_canvas.sh`; verify fatal classes vs the known list, spot
   HTML with the new CSS).
7. **Corpus/service setup** for the July-5 (2606) run; verify the harness
   watchdog + memory-governor settings match `CORTEX_WORKER_HARNESS.md`.
8. Post-run: idle standing-corpus perf re-baseline (PERFORMANCE.md audit-log
   follow-up), then **tag 0.7.0** from post-run `main` (decided 2026-07-02).

### Large arXiv corpus troubleshooting (2026-06-30, user-requested) — IN PROGRESS
**User directive 2026-06-30:** after the 2605 (10k/sandbox) troubleshooting, also troubleshoot
the **full arXiv corpus** at
<https://corpora.latexml.rs/corpus/arXiv/oxidized_tex_to_html>. **First pass done 2026-07-02**
(see the session entry above): live-run fatal mining at ~32% corpus produced 4 landed fixes
(2 panic sites, `\dabar@`, plain-`\+`) + PARITY verdicts for `\tikzcdmatrixname`/tikz-cd.
**Remaining threads for the next pass** (fresh fatals accrue as the run completes, ~2026-07-04;
fleet binary intentionally NOT swapped mid-run — rebuild only for the July-5 run):
- the residual `\lx@begin@alignment`/group-leak TooManyErrors family (516 papers; `\+` covered
  one driver, scalebox `\Gscale@@box` (~129, 2605 numbers) still open, others unidentified);
- the generic `_`/`^` math-mode cascade families (1.7k/1.4k papers — need sub-clustering by
  first-error);
- `never_completed_with_retries` (1,069) — sample for OOM/hang/crash witnesses
  (STABILITY_WITNESSES overlap);
- plain-layer leakage decision (55-name audit in the 2026-07-02 session entry): retract
  remaining tabbing entry points vs keep (user call pending).
Method: DB signature-clustering + `cortex_worker --standalone` (exact fleet binary) +
same-host Perl verbose; the canvas-triage skill encodes the rules.

### TokenLimit `tblr` colspec binding — ✅ DONE 2026-06-30 (`226d3bfa51`)
The cleanest fixable thread from the TokenLimit root-cause: `\tblr` now parses its inner spec,
extracts `colspec`, and translates the column mini-language to a classic `\tabular` template
(see the 2026-06-30 "Landed this session" TokenLimit note). **Remaining tabularray follow-ups
(not done):** the `colspec` translation drops X-column stretch (maps `X→l`) and ignores the
non-`colspec` keys (cell/row coloring, spans via `\SetCell`, `hlines`/`vlines` are no-ops) —
those are fidelity polish, not the alignment-leak/runaway bug (which is fixed). The babel-`.ini`
and expl3 TokenLimit hot loops (witnesses 2605.29738 / 2605.05840) remain deep open efforts.

### mhchem-manual fidelity mission (2026-06-27, on `followups-2026-06-27`) — LANDED
Driven by a manual review of `~/Downloads/mhchem.tex` (the mhchem package manual)
rendered with `--preload=ar5iv.sty --css=ar5iv.css --nodefaultresources
--path=~/git/ar5iv-css/css` (glowup branch), examined via playwright + Chrome.

1. **7 new `latexml_contrib` package bindings** for the manual's missing packages
   (errors 10→0): `fancyvrb-ex`, `rsphrase`, `hpstatement`, `tgpagella`,
   `sourcecodepro`, `AlegreyaSans` (raw-load real `.sty` where installed, per the
   user directive that raw-loading `.sty` is encouraged; fonts no-op where absent),
   and `scrreprt` (OmniBus `.cls` stub like `scrbook_cls`, + `\minisec`/`addmargin`/
   `\addtokomafont`). Perl ships no binding for any of these, so they are surpass-Perl
   contrib additions. `pstricks` already bound (its warning is a transitive
   fancyvrb-ex dep-scan artifact when the raw `pstricks.sty` is absent — benign).
2. **`\marginpar` font-leak fix** (`latex_constructs.rs`, `bounded => true`) — the
   manual's `\marginpar{\Large !}` leaked `\Large` document-wide (1388 `144%` nodes →
   4). PARITY bug (Perl 0.8.8 leaks identically); fixed surpass-Perl. OXIDIZED_DESIGN
   #39, KNOWN_PERL_ERRORS #38. Output-neutral (suite 1487/0).
3. **mhchem stub RETIRED → raw-load real `mhchem.sty`.** The engine's expl3/xparse/
   chemgreek support is now mature enough that `\usepackage{mhchem}` raw-loads the
   genuine package: chemistry renders with proper digit subscripts (`\ce{H2O}`→H₂O),
   charge superscripts, reaction arrows (`->`/`<=>`/`->[..]`), bonds, states,
   `\cesplit`. Simple `\ce` is 0 errors + correctly formatted (the old stub rendered
   formulae FLAT). chemformula stub updated to require mhchem with `version=4` (the
   real package warns without it; the old stub was silent). **Residual = SHARED Perl
   limitation, NOT a Rust gap (re-classified 2026-06-27):** the full manual still
   emits ~69 edge-case errors under raw-load (`\ce` inside `align*` →
   `\lx@begin@alignment`/`\end@amsalign`; ~56 `\lx@end@inline@math`). The minimal
   reduction `\begingroup$a$\endgroup` inside `align*` errors **IDENTICALLY in Rust
   AND same-host Perl** — deferred-alignment can't clean the cell `$`-frame across an
   intervening `\begingroup`. Nothing to fix for parity; a fix would be a deliberate
   deep surpass-Perl core divergence (not autonomous work). Basic
   `SideBySideExample`+`\ce` is clean. See memory `mhchem-ce-amsmath-alignment-2026-06-27`.

### `ltx_env_<name>` env-markup class — PLANNED, separate branch (churns every test XML)
**User-requested generic enhancement** (2026-06-27): tag environment wrapper markup
with `class="ltx_env_<name>"` so custom/minipage-like envs (e.g. `SideBySideExample`)
become responsively styleable in CSS instead of fixed-width minipages. **MUST be on a
dedicated branch** — it changes nearly every test XML (additive class on every env
element), so the golden-suite update is large and must be done in isolation.
Two implementations, same markup outcome:
- **Binding side (`DefEnvironment!`):** the constructor guarantees exactly one element,
  so unconditionally add `ltx_env_<name>` (via an `@ADDCLASS`/`add_class` after the
  begin constructor opens). Applies to ALL DefEnvironments (`figure`, `table`,
  `theorem`, `minipage`, …) — user chose full scope.
- **Raw side (`\newenvironment`/`\renewenvironment`):** arm at env start; at `\begin`
  construction record `{name, anchor = globally-unique gid of current node, mark}`; at
  `\end` afterConstruct, if EXACTLY ONE element was deposited under the anchor since
  the mark → tag it; zero (font/text-only) or >1 (siblings, e.g. SideBySideExample's
  parboxes) → nothing. **Needs a globally-unique monotonic node gid** (verify/ add;
  `record_node_ids` exists but is xml:id-oriented).
- **SideBySideExample:** keep the working `fancyvrb-ex` raw-load (correct source+result)
  + drive responsive layout from the resulting `ltx_minipage`/`ltx_env_*` hooks in
  `ar5iv.css`; do NOT re-implement the verbatim+render dual capture.

### 1. `\gls`/`\acrshort` in MATH mode (1705.10306) — RE-CLASSIFIED 2026-06-27: almost certainly PARITY (source-confirmed), blocked on unrunnable Perl
293 errors `ltx:XMTok isn't allowed in <ltx:glossaryref>`: a glossary command in
math mode digests the link display text (#3, the literal acronym term) as math →
bare per-letter `<XMTok>`, which the `glossaryref` content model rejects.
**Source-confirmed 2026-06-27 that this is most likely PARITY (NOT a Rust-only
gap — the cortex "Perl 1" is stale/unreliable, per `use-cortex-for-parity-work`):**
- Perl `Stomach.pm::enterHorizontal` (L422-434) is a **no-op in math** (`$mode
  =~ /math$/ => {}`) — Rust's `enter_horizontal` matches faithfully. So the
  `enterHorizontal => 1` on the shared `\lx@glossaries@gls@link` constructor does
  NOT switch #3 to text in math in EITHER engine.
- BOTH engines raw-load the SAME `glossaries.sty` (`InputDefinitions(noltxml=>1)`)
  with the SAME override constructor → both digest #3 in the ambient math mode →
  both produce `glossaryref > XMTok` → both hit the same schema rejection.
- `\ref`/`\cite` in math do NOT error (verified) — their content is STRUCTURED
  (bibref / ref-number), not a literal term; only `\gls`/`\acrshort` emit raw
  letter-XMToks. So glossaryref is specific, but the mechanism is shared with Perl.
- **The earlier "Perl raw-loads glossaries.sty and typesets as TEXT" hypothesis is
  weakened:** Rust raw-loads the identical `.sty`, so if it typeset the term as
  text, Rust would too. It doesn't (output: italic letter-XMToks) → so the `.sty`
  display chain does NOT force text in math.
**Perl confirmed UNRUNNABLE here (2026-06-27):** `latexml glx.tex` → `Fatal:terminate`
in `expl3-code.tex` (l3kernel) at 150 s — glossaries pulls in expl3 which is
pathologically slow in Perl 0.8.8 on this host; cannot capture ground truth.
**Fixing is therefore deferred as a likely non-bug.** If pursued, it parallels the
figure_mixed_content surpass-Perl pattern (a monotonic schema expansion to accept
the math content the builder already produces) — BUT the correct structure is
genuinely uncertain without Perl (XMTok directly? XMText-wrapped? operator-token
for the `\DeclareMathOperator` case? text PCDATA?), and there is **no precedent**
for `XMTok` in any inline element's model, so a speculative change risks an
unfaithful divergence. Repro + full notes:
`docs/reproducers/glossaryref_math_xmtok.tex`.

### 2. 0.7.0 release — release-prep LANDED; tag AFTER the July-5 run (user, 2026-07-02)
Version bumped, `runtime-bindings` in the artifact, `.deb` deps, CHANGELOG/README
done. **Remaining:** tag `0.7.0` on `main` → `release.yml` runs the TL-window
`dumps` + macOS arm64 leg + publish (each first-exercised on that tag).
**Timing decided:** tag from post-run `main`, folding the fatal-mining + perf
fixes and the July-5 run's validation into the release.

### 3. Speed: residual XSLT cost on large math books — ✅ FIXED 2026-06-29 (3rd O(n²) found)
After the seclev (`1172569034`) and head-keywords (`da74f6ecfe`) O(n²) XSLT fixes, the
slowest 2605 papers were multi-chapter math books where XSLT still dominated. Profiled
witness **2605.01585** ("From Qubit to Qubit", 2000+ formulae, 512 titles): `xsltproc
--profile` pinned **`maketitle` at 22.7 s of 24.9 s self-time (95 %)** — the inline
`not(//ltx:navigation/ltx:ref[@rel='up'])` full-tree scan, re-run **per title** =
O(titles × tree). Fixed by memoizing the document-global check into the global
`$maketitle_has_up_nav` (`LaTeXML-structure-xhtml.xsl`), same shape as the seclev fix.
**XSLT 24.94 s → 2.15 s (11.6×); maketitle self 22.7 s → 0.004 s; output byte-identical**
(`cmp` clean, 25 MB Core XML). Suite **1502/0** + guard `09_xslt_maketitle_navscan.rs`.
OXIDIZED_DESIGN #41, ARXIV_PERFORMANCE Hotspot #4. The three XSLT O(n²) templates on
large arXiv docs (seclev / head-keywords / maketitle) are now all O(n).

---

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

## Reference (stable — not active work)

### Engine file open gaps (MINOR, demand-driven)
- `tex_box.rs` box-dimension edges; `tex_fonts.rs` `\fontdimen` array + per-font
  `\hyphenchar`; `tex_tables.rs` padding CSS (XSLT concern).
- **Document-builder block/paragraph auto-wrap of inline content** (core,
  broad/risky family — two witnesses):
  - **`\fcolorbox` inline paragraph-grouping**: an inline `\fcolorbox`
    mid-paragraph — Perl breaks the `<p>` (its `internal_vertical` block ends
    it), Rust keeps it inline. SAME flags on both; Rust's inline reading
    arguably matches real LaTeX's `\mbox`-based `\fcolorbox`. (`\colorbox`
    matches.)
  - **bare `\includegraphics` run in a figure** (witness 1108.0198, found
    2026-06-21 via skeleton diff — a clean, error-free reproducer): a
    `\begin{figure*}` with several consecutive `\includegraphics` (no blank
    line) — Perl wraps the inline run in a `<ltx:block>` (`figure > tags >
    block > graphics×N`), Rust emits the graphics bare (`figure > graphics×N`).
    Rust is error-clean and schema-valid. **Re-witnessed + root-confirmed
    2026-06-27** (0704.0001, 0704.0017 via the corrected structural diff): NOT
    merely cosmetic — the panel `<graphics>` WIDTHS also diverge (Rust 303.5pt vs
    Perl 241.5pt, ~1.257×), so figure sizing is visibly affected. Root: Perl's
    `arrange_panels_and_breaks` (`latex_constructs.pool.ltxml:3229-3295`) does a
    full box-metric panel layout — it inserts `<break class="ltx_break">` and wraps
    panels using `getNodeBox($child)->getWidth` vs `float_width`; Rust's
    counterpart (`latex_constructs.rs:1784-1869`) is explicitly **"Simplified: mark
    panel children with the class"** and skips the break/block arrangement. A
    faithful port DEPENDS on matching box widths → the deep box session (sibling of
    the `\resizebox` panel-width item below), not a loop-tick fix.
- **`\resizebox` panel scale-VALUE divergence**: in `complex/figure_mixed_content`
  two panels get a different computed natural width (xscale 1.13 vs 0.88). The
  construct in ISOLATION matches exactly (both xscale=1.9685); the divergence
  only appears inside the paper's `\footnotesize` + `table*` + `\subfloat` panel
  context → a font-size/box-context interaction. Scale *formatting* (%.15g) is
  already Perl-faithful (`551c5286ba`); missing-image candidates too
  (`64dd30b284`). Deep box-metric; for the focused box session.
- **~72-CS Perl-only long tail** (from the archived LoadFormat audit): misc
  atomics (`\@charlb`, point-size CSes, `\batchmode`, …) Perl defines, Rust does
  not. Investigate a CS only when a real paper witnesses it; refresh the CS-name
  diff before quoting counts (predates the BibTeX port).

### Primitive layer — AUDITED FAITHFUL (2026-06-20)
Probe-based Rust-vs-Perl audit found the core primitive layer byte-identical
(arithmetic, dimensions, glue, conditionals, string/token, case tables). Don't
re-audit without a witnessing paper. Shared-with-Perl quirks (NOT Rust bugs):
`\numexpr` divideround round-half-toward-+∞ (KNOWN_PERL_ERRORS #33); `\the\skip`
drops stretch/shrink to bare pt.

### Permanent ignores
- **Out-of-scope**: ns1–ns5 (`52_namespace`, no DTD support); 2402.03300,
  2410.10068, 2511.03798 (Perl also fails).
- **Rust supersedes Perl**: `1207.6068`, `0909.3444`, + 40 more in
  `memory/project_rust_supersedes_perl.md`.
- **BibTeX**: `BibTeX.pool.ltxml` ported (Phases 1–8; remaining B1–B6 polish in
  `BIBTEX_PORT_PLAN.md`). `--nobibtex` is opt-out, not default.

### Tikz known diffs vs Perl
`foreignObject` transform; arrow-tip path data; SVG viewBox/width; matrix
`<svg:g class="ltx_tikzmatrix">` vs inline-blocks; **bare `svg:g` in `<ltx:block>`**
(tikz-cd) trips a core-XML validity error but post-processing recovers (witness
2006.12702) — Rust-only, low priority (output recovered).

### Graphics renderer chain (subprocess-only; LANDED)
PDF→PNG `mutool draw`→`pdftocairo`→`convert+gs`; PDF→SVG `mutool convert`→
`pdftocairo`→(raster PNG fallback). EPS/PS→`gs` direct→`convert+gs`. Subprocess
`exec` (no GPL linking). Apt: `poppler-utils` (req), `mupdf-tools` (rec),
`imagemagick+ghostscript`. A heavyweight inkscape third resort for PDF→SVG was
removed 2026-06-29 (GTK stack, 20–40× slower, timeout-prone, no coverage over the
raster fallback).

### Other tracks (separate docs)
- Performance: `PERFORMANCE.md` (P1 math/large-doc open; P2 allocation partial).
- Release gates: `RELEASE_CRITERIA.md`. Releasing: `RELEASING.md`.
- **BibTeX (plan archived 2026-07-02 →
  [`archive/BIBTEX_PORT_PLAN_2026-06-20.md`](archive/BIBTEX_PORT_PLAN_2026-06-20.md)):**
  Phases 1–8 shipped; live residuals = the Phase 4–5 field-handler/MR-Zbl
  long tail, divergences B1–B6 noted in `bibtex.rs`, and the deferred
  **native `.bst` interpretation** (witness 2605.16562, `f65cf7d6dc`) —
  demand-driven, pick up on corpus evidence.
- Completed missions (archived): strict-LoadFormat dump parity, Marpa ASF
  migration, distribution-readiness, the 500K/1M warning-corpus mission, the
  diagnostic-message faithfulness pass (2026-06-20), and the upstream-sync
  PR translation U1–U11 (2026-06-26) — see `docs/archive/` and `git log`.
