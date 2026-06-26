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

- `cargo test --tests`: **1468 / 0 / 0**.
- `cargo clippy --workspace --all-targets -- -D warnings`: **clean**.
- `--init=plain.tex` / `--init=latex.ltx`: **0 errors** (with dump and `LATEXML_NODUMP=1`).
- Distribution build (`maxperf`): ~45 MB; beats 2× pdflatex on the mini-benchmark.

### Landed this session (2026-06-25, on `post-processing-signal-fidelity`)

**Signal-fidelity pass — ~200.7k spurious `warning` messages eliminated from the
10k sandbox, all faithful to Perl, ZERO output change.** Triaged the dominant
post-processing/digestion warning clusters in the cortex 10k run; each was a
Rust-only divergence where Perl is silent (verified against the Perl source per
fix):

- **`expected:id` parse-time transient (128.9k msgs / 1142 tasks)** — the
  math-parser `realize_xmnode` (`parser.rs`) warned "Cannot find a node with
  xml:id" on a LIVE-`lookup_id` miss mid-reinstall (a Rust/ASF artifact Perl's
  `MathParser::realizeXMNode` lacks). Empirically benign: on the heaviest witness
  `0704.2400`, 85/98 warned ids are present in the output and the other 13 leave
  **0 dangling `<XMRef>`** (0 dangling of 2597 idrefs); the whole 10k has **0
  `error:expected:id`**. Made silent; genuine danglers still caught by the
  faithful post-Error (`latexml_post`, Perl `Post.pm:1444/1456`). Output
  byte-identical. See `EXPECTED_ID_XMREF_DESIGN_2026-06-08.md` (2026-06-25 banner).
- **`expected:register \tabcolsep`/`\arraycolsep` (43.8k msgs)** — `\lx@text@intercol`
  / `\lx@math@intercol` used the warning `lookup_register`; Perl
  (`TeX_Tables.pool.ltxml:639/646`) uses a silent `isRegister ? valueOf :
  Dimension(0)` inline guard (a document may `\renewcommand` the length register
  into a macro). Added `state::lookup_register_quiet` (no warn) and used it.
- **`expected:register \fam` (27.1k msgs)** — `decode_math_char` (`mathchar.rs`)
  read `\fam` via warning `lookup_register`; Perl `decodeMathChar`
  (`Package.pm:2928`) reads `lookupValue('fontfamily')` DIRECTLY. Switched to a
  direct `fontfamily` read (the `\fam` register's own getter already does this) —
  no warn, and correct even when `\fam` is shadowed (matches Perl). Normal
  (non-shadowed) `\fam` is unaffected (suite unchanged).
- **`expected:id` createXMRefs (900 msgs)** — `base_xmath.rs` XMDual
  `after_close_late` warned "Unresolved _xmkey"; Perl (`Base_XMath.pool.ltxml:306-308`)
  silently does `setAttribute(idref => undef)`. Removed the Rust-only warning.

NON-fixes (confirmed PARITY — Perl warns too via `LookupRegister`, left as-is):
`\tikz@dashphase`/`\cmdGR@*` (pgfmath `pgfmath_register`→`LookupRegister`), `\c@*`
counters (`CounterValue`→`LookupRegister`). `eqnarray` `\arraycolsep` at
`latex_constructs.rs:971` is a minor remaining divergence (Perl `LookupDimension`
reads the macro body; 354 msgs / 2 tasks) — deferred, needs a `LookupDimension` port.

Suite 1468/0, clippy clean, fmt clean.

**10k-sandbox rerun validation** (maxperf-cortex, 72-worker fleet, vs the PR#269
snapshot, true per-task transition matrix from `historical_tasks`):
- **no_problem 6219 → 6982 (+763)**, warning 2446 → 1683 (−763) — the +763 are
  papers that were `warning` SOLELY from the removed spurious diagnostics, now
  correctly clean (matching Perl's clean output).
- **ZERO clean→hard regressions**: 0 no_problem→{warning,error,fatal}, 0
  warning→error. Only transitions: `warning→no_problem` 763, plus `error↔fatal`
  ±1 boundary noise (2 papers `never_completed_with_retries` = cortex
  timeout/retry infra, unrelated to the warning-suppression code).
- **Total warning messages 262,986 → 62,106 (−200,880, −76%)**; `expected:id`
  130,814 → 1,011 (only the faithful "Missing idref" Warn remains); `expected:register`
  74,790 → 3,865 (only the parity pgfmath/counter ones remain). error 1175→1174,
  fatal 65→66 (within run-to-run variance).

**Error-category triage (2026-06-25, same fresh rerun).** After the warning
de-noising, re-mined the ERROR/FATAL cross-join (Rust error/fatal vs Perl
no_problem/warning) for genuine Rust-only regressions. **Mined out** — the strict
filter (Rust error/fatal AND Perl `no_problem`, excluding cyrillic phantoms)
returns a single missing-macro singleton (`\textcjheb`). Specifics:
- `unexpected:<char>` inputenc (20 papers) + babel (13) = **re-confirmed env-artifact
  phantoms** (host lacks the `cyrillic` collection; same-host Perl fails identically).
- `Attempt to end mode` box-recovery (73 papers) = **71/73 Perl-parity**.
- `malformed:ltx:XMTok` (3 papers) = parity/Rust-better (Perl error/fatal).
- The only GENUINE Rust-only finding is the **`{\input file}` box cascade**
  (math0701308: same-host Perl 0, Rust 90; `TeXFileName` consumes the `}` —
  shared Perl parity — then Rust's stricter box recovery cascades into text-mode
  `_`/`^` errors). **Low-breadth (3 papers); deferred** — the faithful fix is a
  deep stomach-recovery change, the easy fix (`TeXFileName` stop-at-`}`) diverges
  from Perl-LaTeXML and has broad blast radius. Characterized for a dedicated
  effort. Reconfirms: the cortex `Perl=clean` baseline is unreliable (env
  artifacts) — verify every cross-service delta with same-host Perl.

### Landed earlier (2026-06-22, on `further-stability-coverage`, pushed)

Two genuine Rust-only bugs fixed + the full p/m/b table-column parity arc:
- **Cluster G hang** `1707.02464` — `\hsize`-aware vbox paragraph wrapping (faithful
  Perl `readBoxContents`); the `\narrow` `\hsize`-shrink loop now terminates
  (hang → ~4.8s, 10 errors = Perl). `7545e07fd6`. See `STABILITY_WITNESSES.md`
  Cluster G (FIXED).
- **p{} block-content** `1510.07685` — `\begin{itemize}` in a `p{}` cell (3→0
  errors); global p{} → Perl `\lx@tabular@p` VBox form (1610.00974 step-3).
  `f65b80c1c2`.
- **array.sty m{}/b{} → `\lx@tabular@p`** (`eb978df5a9`) and **p/m/b `<td>`
  `align="left"`** (`1867f17da9`) → **cluster-B FULLY RESOLVED**; table fixtures
  near-/exact-to-Perl (array_newline_math Perl-exact); rotfloat2 sidewaystable
  innerheight 69.1→98.6 vs Perl 98.5.
- Validated regression-free: 12 table-stressed papers + a fresh 24-paper same-host
  sweep (0 Rust>Perl; Rust at-or-better everywhere) + class-level cross-join.
- (Earlier this session: tasks 5 & 6 below — post-processing log parity + graphics
  never-ship-raw-.eps — also landed.)

## Upstream sync — translate brucemiller/LaTeXML PRs since #2767 (NEW MISSION, opened 2026-06-25)

> **Mission.** The Rust port mirrors upstream Perl LaTeXML through commit
> `23f3acfa` (#2767 "Frontmatter refactor"; record in
> `docs/archive/frontmatter_api_refactor.md`). Upstream `master` has since
> advanced **9 commits** to `cb455179` (#2783). Translate each PR **in merge
> order, as a separate sub-task**, faithfully — `perl-port` discipline: read the
> Perl diff first (`git -C LaTeXML show <commit>`), place per `ORGANIZATION.md`,
> obey the divergence policy (`OXIDIZED_DESIGN.md`). Check an item off here when
> it lands (`git log` keeps the record); archive this catalog when the whole
> mission completes.
>
> **Commit granularity (user directive 2026-06-25): each sub-task = its own
> self-contained commit** — one feature/patch deliverable per commit, never a
> batched mega-commit. The #2798 sub-mission lands as several commits (one per
> sub-step). All work on the **`upstream-sync-prs`** feature branch — never
> FF-push to `main`; open a PR (per `branch-further-stability-coverage-workflow`).
>
> **The `LaTeXML/` checkout is already AT `cb455179`**, so the *new* Perl is the
> live reference for every file; diff against the per-PR commit to isolate a
> single PR's change. Each landed sub-task needs: faithful port + ported test
> fixture(s) (`cargo clean` after adding a `.tex`/`.xml` pair so the test plugin
> rediscovers them — see CLAUDE.md) + `cargo test --tests` green + clippy clean,
> and re-confirm on the current binary.
>
> **Sizing.** 8 of 9 are small/contained (new bindings, listings tweaks, a proof
> fix, a mostly-already-present residual). **#2798 "Leavehorizontal" is the one
> large core-engine refactor** (Font.pm +303, latex_constructs +174, Box/List/
> Stomach/Whatsit/Alignment + ~15 packages) — stage it as its own sub-mission
> with a dedicated regression budget, NOT a single commit.
>
> **Recommended execution order** (the user's "in order" = merge order is the
> default; the independent small wins can land first to build momentum):
> ① #2737 causets → ② #2806 dirtytalk → ③ #2814 proof-punct → ④ #2783
> quantikz2-residual (all independent, **S**) → the **listings cluster**
> #2819 → #2818 → #2824, then #2828 (shared file `listings_sty.rs` + shared
> `listing` fixture — sync the fixture ONCE at the end) → **⑤ #2798
> Leavehorizontal LAST** (largest; its own listings.sty + table/box touches may
> reshuffle the listings fixture again, so doing it after the listings cluster
> avoids double fixture churn).

### U1. ✅ PR #2806 "Add dirtytalk binding" (`51fea96a`) — LANDED
- **What:** `dirtytalk.sty` — `\say{…}` context-aware quotation marks with a
  nesting-depth counter (`dirtytalk@qdepth`): outer level uses
  `\textquotedblleft/right`, nested uses `\textquoteleft/right`. Four KeyVals
  (`left`/`right`/`leftsub`/`rightsub`) let the user override each symbol.
- **Perl:** new `lib/LaTeXML/Package/dirtytalk.sty.ltxml` (54 lines): 4
  `DefMacro` symbol defaults + 4 `DefKeyVal('dirtytalk', …, 'UndigestedKey', …,
  code => setDirtytalkSymbol)` + `ProcessOptions(inorder=>1, keysets=>…)` + a
  `RawTeX` block (`\newcounter`, `\dirtytalk@lsymb/rsymb` `\ifnum`, `\say`).
- **Rust target:** new `latexml_package/src/package/dirtytalk_sty.rs` (register
  in the package module list). `\say` is currently only in `revtex4_support_sty`
  (unrelated). The `\say` core + 4 symbol-default `DefMacro`s + the
  `\newcounter`/`\say` `RawTeX!` block are straightforward (`raw_tex`,
  `Tokens::is_empty` for `IsEmpty` all exist). **The 4 keyval overrides
  (`left`/`right`/`leftsub`/`rightsub`) need the runtime
  `keyval::define(KeyvalConfig { code: Some(…), .. })` directly** — the
  `DefKeyVal!` macro has NO `code`-callback arm and Rust has no `UndigestedKey`
  type, so map Perl's `UndigestedKey` + `code => sub` onto the config's `code`
  field (verified `KeyvalConfig.code: Option<ExpansionBody>` exists).
- **Complexity:** **M** (core `\say` is S; the keyval-override callbacks add the work).
- **Tests:** ported `t/structure/dirtytalk.{tex,xml}` → `latexml_oxide/tests/structure/`
  — Rust output **byte-identical to Perl** (nested `\say` curly quotes), error-clean,
  `dirtytalk_test` green; keyval override (`[left={«},right={»}]` → `«hi»`) smoke-validated
  via the faithful `ExpansionBody::Closure` (incl. the `IsEmpty` guard); clippy clean.

### U2. ⬜ PR #2798 "Leavehorizontal" (`24d39b55`) — LARGE CORE REFACTOR (XL; stage as a sub-mission)
- **What:** two coupled rewrites + a wide application layer (75 files,
  +1172/−902; ignore the CI-only `windows.yml`):
  - **(A) TeX-faithful mode / `leaveHorizontal`.** In real TeX, beginning a
    vertical/display construct while in horizontal mode first ends the paragraph
    (`\par`); an inline `\hbox`/block does not. LaTeXML scattered
    `leaveHorizontal`/`enterHorizontal`/`\par` inconsistently. Now `beginMode`
    itself calls `leaveHorizontal` when entering a vertical/display bindable mode
    **unless the user mode name contains `inline`**, and a new pseudo-mode
    **`inline_internal_vertical`** (→ bound `internal_vertical`, suppressing the
    auto-leave) marks inline blocks (`\vbox`/`\vtop`/`\parbox`/`minipage`/
    `picture`/footnotes). `digestNextBody` splits into `digestUntil` (digests
    onto the *current* `@LIST` without rebinding) + a thin wrapper; `T_BEGIN`
    only builds a fresh `List` in math mode, else digests onto the ambient list;
    `executeBeforeDigest` pushes results onto `@LIST` instead of returning them;
    `repackHorizontal` records `\hsize`+`\baselineskip` on the finished paragraph
    (and `\hsize` is recorded ONLY there now).
  - **(B) Box/Font sizing rewrite** (the +303 Font.pm is the largest piece).
    Box.pm separates *requested* (`width/height/depth`) from *computed*
    (`cwidth/cheight/cdepth`); `getWidth/…` return only `c*`; new `getSPSize`
    (raw scaled-point triple); `computeSizeStore` bypasses fully-specified sizes,
    marks `isEmpty`, adds padding (`padtop/padbottom/padleft/padright`,
    `totalheight`). Font.pm rewrites `computeBoxesSize` to dispatch by ref-type
    and thread real `baseline`/`totalheight`/math-axis, replacing the old
    `_box/_words/_lines/_stack` helpers with `linebreak_paragraph`/
    `flatten_paragraph`/`split_words`/`collect_lines`/`stack_lines`; CJK
    (`\p{Ideographic}`) counts as `isIdeographic`. Whatsit gains
    `flattenForSizing` (a horizontal whatsit with a pure `#arg`/`#prop` sizer is
    flattened so `\emph` etc. line-break across the paragraph).
  - **(C) Application (~75 files):** new `mode`/`sizer`/padding props across the
    engine pools + ~31 package/class bindings; `\begin{document}` sets
    `\columnwidth=\hsize=\linewidth=\textwidth`; `\emph` → `bounded`+`sizer=>#1`
    (drops `restricted_horizontal`); itemize envs LOSE the mistaken `\par` and
    gain real `\topsep/\parsep/\partopsep/\itemsep` glue + padding; captions →
    new `PBoxContents` param (caption arg processed as a horizontal paragraph);
    `\@framebox` padding from `\fboxsep+\fboxrule`; parbox/minipage/picture →
    `inline_internal_vertical`; + 24 regenerated `t/*.xml` fixtures.
- **Rust state:** core is at the **pre-PR shape** — `stomach.rs` `bindable_mode`
  lacks `inline_internal_vertical`, `begin_mode_opt` does not `leave_horizontal`;
  `executeBeforeDigest` still returns `@pre`; `digest_next_body` is monolithic;
  `T_BEGIN` (`tex_box.rs`) always builds a List; `list.rs` infers mode/width
  (pre-PR); `tbox.rs`/`lib.rs` size-getters use the old `width||cwidth` fallback
  (no `get_sp_size`/padding/`isEmpty`/full-spec bypass); `common/font.rs`
  `compute_boxes_size`+`_words/_lines/_stack/_box` is the pre-PR algorithm;
  `whatsit.rs` has no `flatten_for_sizing`; alignment lacks `replace_column`.
  **Already done in Rust:** the `\lx@add@thanks`/`\person@thanks` removal
  (`base_utilities.rs:210`, `latex_constructs.rs:4505`) + `\lx@personname` sizer
  → S0 is verify-only. **Known overlap:** the p/m/b + `\multicolumn` rework (S9)
  intersects landed Rust array work (memory
  `genuine-rust-only-unexpected-clusters-2026-06-21`, `array_pcolumn`
  reproducers) — reconcile, don't re-port. **`Package.pm`** adds a `$noerror`
  param to `LookupDimension`; Rust's `lookup_dimension` (`state.rs:1613`) is a
  value-cast helper, NOT a faithful `LookupDimension` (it lacks the
  register→macro-`readingFromMouth`→warn-on-undefined logic), so reconcile that
  alongside the `noerror` add (small, but a real semantic gap; see also the
  parked `lookup_register`→`lookup_dimension` eqnarray/cases cleanup).
- **Complexity:** **XL** — two deep foundational rewrites on the hottest
  digestion/sizing paths + wide fixture churn. **Land as separate commits (one
  per sub-step), never one commit.**
- **Ordered sub-steps** (two foundations first; the app layer needs both):
  - **S1 (M, FOUNDATION — namesake)** core mode mechanism: `bindable_mode +=
    inline_internal_vertical`; `begin_mode_opt` calls `leave_horizontal` on
    vertical/display entry when the user mode lacks `inline`. `stomach.rs` + mode
    mapping in `binding/def/dialect.rs`. Keystone; very high blast radius.
  - **S2 (M)** `beforeDigest` pushes onto the active list (not return):
    `primitive.rs`+`constructor.rs` invoke + drop the dialect prepend.
  - **S3 (M, high risk)** extract `digest_until` from `digest_next_body`;
    rewrite `T_BEGIN` (math-only List). Depends S2. (`{…}` grouping is everywhere.)
  - **S4 (M)** `list.rs` `reqmode` + `baseline` on vertical lists; stop setting
    per-List `width`; `repack_horizontal` records `width=\hsize`,
    `baseline=\baselineskip`. Depends S1.
  - **S5 (L, FOUNDATION)** Box.pm rework: `@sizing_properties`; getters return
    `cached_*` only; setters set both; `get_sp_size`; `compute_size_store` with
    full-spec bypass, `isEmpty`, padding/`totalheight`. `tbox.rs` + size trait in
    `lib.rs`. Very high blast radius (every size query).
  - **S6 (L, critical-path risk)** Font.pm rework: rewrite `compute_boxes_size`;
    add `linebreak_paragraph`/`flatten_paragraph`/`split_words`/`collect_lines`/
    `stack_lines`; baseline/totalheight/math-axis; per-box-font kern. `common/
    font.rs`. Depends S5/S4. The +303 change — port behind the `size` debug
    instrumentation and diff line/word/stack vs Perl.
  - **S7 (M)** Whatsit `flatten_for_sizing` + CJK `isIdeographic`. Depends S5/S6.
  - **S8 (M)** Alignment `replace_column` (template+alignment) + 3-tuple
    `normalize_cell_sizes` + `extractAlignmentColumn` PUT/USED split. Depends S5/S6.
  - **S9 (L)** TeX_* pools: `PBoxContents` + `\lx@enterhorizontal`; vbox/vtop →
    inline_internal_vertical; `\vskip` pure-height; display-math pad +
    under/overline; p/m/b inline-block cols + `\lx@tabular@p` + `\multicolumn`
    (overlaps landed work). Depends S1–S8.
  - **S10 (L, highest fixture impact)** latex_base+latex_constructs: `\baselinestretch`;
    moved `\addvspace/\addpenalty/\@endparenv`; `\begin{document}` widths; `\emph`
    bounded+sizer; itemize `\par`-removal + glue + padding + `\preitem@par`;
    caption `PBoxContents`; `\@framebox` padding; parbox/minipage/picture
    inline_internal_vertical. Depends S1–S9.
  - **S11 (M)** ~31 package/class bindings — aas_support, acmart, alltt,
    amsrefs, amstext, array, bbold, beamer, cancel, elsarticle, enumerate,
    enumitem, epsf, fancyvrb, frenchb, glossaries, graphics, hyperref, IEEEtran,
    JHEP, listings, natbib, ntheorem, numprint, paralist, pgfsys-latexml,
    setspace, soul, sv_support, xcolor, xy — mostly mechanical mode/`sizer`/
    property edits + `LaTeXML.css` (`ltx_verbatim` nowrap, overline/underline
    classes) + `LaTeXML-picture-xhtml.xsl`. Depends S1–S10.
  - **S0 (S)** verify the already-done thanks/`\lx@personname` bit — expect no-op.
- **Tests:** no new `.tex`; 24 regenerated `t/*.xml` (alignment
  array/cells/colortbls/halignatt/tabular; math array_newline_math; complex
  figure_dual_caption/figure_mixed_content/cleveref_minimal/equationnest;
  structure authors/autoref/enum/figure_grids/figures; fonts marvosym/sizes;
  digestion dollar; babel numprints; ams mathtools; graphics
  graphrot/picture/xytest; tokenize alltt). All exist in `latexml_oxide/tests/`
  at pre-PR output — **regenerate each from same-host Perl `cb455179` and diff;
  never hand-edit** (legit size/paragraph-structure churn; a few intentional
  divergences per `OXIDIZED_DESIGN.md`). Gate each fixture on its sub-step.
- **Empirical coupling (measured 2026-06-25 — S1 prototyped then reverted):**
  S0 verified (thanks/`\lx@personname` already in Rust — `base_utilities.rs:215`,
  `:807`; no-op). A standalone **S1** prototype (added `inline_internal_vertical`
  to `bindable_mode` + the `leave_horizontal()`-on-vertical/display-entry guard
  in `begin_mode_opt`, `stomach.rs:663,681`) **breaks exactly 11 tests**, which
  split cleanly:
  - **4 genuine regressions** — `footnote`, `endnote`, `etoolbox`, `fancyhdr`
    — are **NOT** in Perl's 24-fixture regen list, so Perl's output for them is
    unchanged by #2798. They break only because the inline blocks (footnotes
    etc.) aren't yet reclassified to `inline_internal_vertical`, so they wrongly
    `leaveHorizontal`. **Fix = the S9/S10 inline reclassification**, after which
    their existing fixtures pass again.
  - **7 sizing-coupled regens** — `dollar`, `autoref`, `enum`, `equationnest`,
    `figure_mixed_content`, `picture`, `sizes` — **are** in Perl's regen list
    (legit #2798 churn). Their regenerated Perl fixtures carry the new
    paragraph-structure **and** the new `width/height/depth`/padding/`baseline`
    size attributes, so Rust cannot match them until the **S5/S6 Box.pm/Font.pm
    sizing rewrite** lands. Regenerating them before S5/S6 just trades one red
    for another (Rust's intermediate sizes ≠ Perl's).
  - **Conclusion: U2 is test-atomic.** No proper subset of S1–S11 reproduces the
    combined-state fixtures — the +303 Font.pm rewrite (S6) is on the critical
    path for 7 of the 11. Land U2 as **one coherent push** (S1→S11 + reclassify
    inline blocks + regenerate all 24 fixtures from same-host Perl `cb455179`),
    reaching green only at the end; do **not** commit a partial S1 (it leaves
    the tree red). The S1 diff is small and known (above) — re-apply it first
    when the coherent push begins. Prototype reverted; branch stays green.

### U3. ✅ PR #2819 "listings: create group around identifiers" (`0d748100`) — LANDED (absorbs U5)
- **What:** in `lstSetClassStyle`, a TeX-style class now wraps its styling in a
  brace group — `begin => Tokens($style, T_BEGIN)`, `end => T_END` (was
  `begin => $style` only). And in `lstProcess_internal`, index emission uses
  `lstRescan($index{begin})…lstRescan($index{end})` (was bare `T_BEGIN … T_END`).
  Net effect: identifier/keyword styling is grouped (e.g. `\bfseries\underbar`
  applies as a group so the underline spans the whole keyword).
- **Perl:** `lib/LaTeXML/Package/listings.sty.ltxml` (`lstSetClassStyle` ~L496;
  `lstProcess_internal` ~L1413).
- **Rust target:** `latexml_package/src/package/listings_sty.rs` —
  `lst_set_class_style` (TeX branch, ~L450) + `lst_class_end` (~L956).
- **Rust port (this branch):**
  - `lst_set_class_style` TeX branch: `begin = Tokens(style, T_BEGIN)`, add
    `end = T_END` — faithful to the Perl diff.
  - `lst_class_end`: changed from **leaf-only** end collection to walking the
    **full class chain** (push order: leaf close-delims first, parent styling
    group-closers last), so the `T_END` added to a parent styling class
    (comments/strings) matches the `T_BEGIN` in its `begin`. Pre-#2819 only leaf
    delimiter classes carried an `end`, so this is a no-op extension there;
    keyword/identifier classes are themselves the leaf, so their `T_END` was
    already collected. Verified faithful for both keyword-leaf and
    delimiter→styling chains (matches Perl's `@close` order exactly).
  - The `lstProcess_internal` **index** line-change has **no functional Rust
    target**: Rust's index branch is a no-op stub (`// Index generation
    (simplified)`) that emits nothing, so the bare-`T_BEGIN/T_END` → `index{end}`
    swap is moot until the index feature is implemented (left as-is).
- **Complexity:** **M.**
- **Tests:** resynced `t/alignment/listing.{tex,xml}` (added `\underbar` to the
  bingo keywordstyle). Rust output for the grouped keyword now renders
  `<text class="ltx_lst_keyword ltx_underline" font="bold">foo</text>` —
  **matching upstream HEAD `cb455179` (post-#2828) exactly**; the only delta vs
  Perl is the missing `<indexmark>` blocks (pre-existing index-stub divergence,
  not introduced here). `53_alignment` suite (9 tests incl. `listing_test`)
  green; error-clean; clippy clean. **#2828 (U5) is absorbed**: Rust's
  `\underbar` renders natively as `ltx_underline` (the settled form), so the
  #2819→#2828 underline transition happens in one step here.

### U4. ✅ PR #2818 "listings: do not look up ltxml files when reading raw files" (`41bd31e8`) — LANDED
- **What:** `listingsReadRawFile` now calls `FindFile($filename, noltxml => 1)`
  so `\lstinputlisting{foo.sty}` reads the raw source, never a `.ltxml` binding.
- **Perl:** `listings.sty.ltxml` `listingsReadRawFile` (~L320).
- **Rust target:** `listings_sty.rs` `listings_read_raw_file` (L234) — pass the
  `noltxml` flag to the find-file call.
- **Complexity:** **S** (one-flag change). Rust spells `noltxml` as `forbid_ltxml`
  in `FindFileOptions` (matches `tex_job.rs:252`).
- **Tests:** none new; listing suite (9 tests) green — output unchanged, so it's a
  clean independent commit.

### U5. ✅ PR #2828 "Resync listings test for change to underline" (`39f319bd`) — LANDED (absorbed into U3)
- **What:** test-only follow-up to #2819 — the underline styling settled to
  `class="ltx_lst_keyword ltx_underline"` (from the intermediate
  `framed="underline"`).
- **Perl:** `t/alignment/listing.xml` only.
- **Rust outcome:** **absorbed into U3** — Rust's `\underbar` renders natively as
  `ltx_underline`, so the U3 fixture regen produced the final post-#2828 form
  (`ltx_lst_keyword ltx_underline`) directly, in one step. No separate commit.
- **Complexity:** **S** (fixture resync).

### U6. ✅ PR #2814 "Fix 2240 proof title punct" (`01b8d651`) — LANDED
- **What:** the amsthm `proof` env stops double-punctuating — append the trailing
  period only when the (optional) title doesn't already end in `.!?:;,` (mimics
  LaTeX `\@addpunct`). `\begin{proof}[x.]` → "x." not "x..".
- **Perl:** `lib/LaTeXML/Package/amsthm.sty.ltxml` `\@proof` properties (~L155):
  inspect `$content[-1]->toString` and conditionally add `T_OTHER('.')`.
- **Rust target:** `latexml_package/src/package/amsthm_sty.rs` — the `\@proof`
  `properties` closure currently does an **unconditional** `title_tokens.push(
  T_OTHER!("."))` at **L188**; gate it on the last content token's last char.
- **Complexity:** **S.**
- **Tests:** ported `t/theorem/proofpunct.{tex,xml}` → `latexml_oxide/tests/theorem/`
  — Rust output **byte-identical to Perl**, error-clean, `proofpunct_test` green.

### U7. ✅ PR #2737 "Added bindings for causets (TikZ extension)" (`eb08bd7f`) — LANDED
- **What:** `causets.sty` binding = a raw-load passthrough:
  `InputDefinitions('causets', type => 'sty', noltxml => 1)`.
- **Perl:** new `lib/LaTeXML/Package/causets.sty.ltxml` (24 lines, body is the
  one `InputDefinitions` call).
- **Rust target:** new `latexml_package/src/package/causets_sty.rs` that
  raw-loads `causets.sty` with `noltxml`. Binding itself is trivial; actual
  rendering depends on the host TikZ machinery (out of scope for the binding).
- **Complexity:** **S.**
- **Tests:** none added upstream. Smoke-validated: `\usepackage{causets}` loads
  error-clean and raw-loads the host `causets.sty` (`Loading causets.sty…`),
  body renders; clippy clean.

### U8. ✅ PR #2824 "do not add frame and background to inline listings" (`a6f6316f`) — LANDED
- **What:** `\lstinline` / `\begin{lstinline}` now set `LISTINGS_INLINE => 1`;
  `\lst@@@set@frame` and `\lst@@@set@background` skip the frame/background when
  `LISTINGS_INLINE` is set (inline listings shouldn't get a box frame/bg).
- **Perl:** `listings.sty.ltxml` — `\lx@lstinline` (~L58), `\begin{lstinline}`
  (~L94), `\lst@@@set@frame` (~L929), `\lst@@@set@background` (~L958).
- **Rust port (this branch):** set `LISTINGS_INLINE` (local to the inline
  bgroup) in `\lx@lstinline` (after `lst_activate`, before reading the
  delimiter) and the `lstinline` environment; `\lst@@@set@frame` returns an
  empty `frame` prop when inline (constructor already skips an empty `framed=`).
  For `\lst@@@set@background`, guarded the **entire** body on `!inline` — not
  just the `merge_font`. Perl never clears `LISTINGS_BACKGROUND`; Rust's
  `assign_value(None)` is a workaround so a *block* listing doesn't leak its bg
  to later listings. Guarding the clear too keeps that workaround for block
  listings while leaving the global value intact when an inline listing runs
  first, so a **following block listing still renders its background** (verified
  by repro: global `\lstset{frame=single,backgroundcolor=\color{yellow}}` →
  inline gets neither, the next block listing gets both — matching Perl). A
  naive merge-only guard regressed that case.
- **Complexity:** **S–M.**
- **Tests:** none new — `53_alignment` `listing` suite (9 tests) unchanged-green
  (matches upstream: #2824 had no Perl fixture change); error-clean; clippy
  clean; behaviour confirmed by the inline-vs-block repro above.

### U9. ✅ PR #2783 "quantikz2 raw interpretation" (`cb455179`) — LANDED (color-macro residual)
- **What:** four fixes (this PR was authored by us and partly upstreamed
  Rust-discovered fixes): (a) `\AtBeginDocument[]{}` / `\AtEndDocument[]{}`
  optional `[label]`; (b) `color.sty` defines `\current@color`/`\default@color`/
  `\reset@color` with safe DVI defaults; (c) `tcolorbox.sty` pre-defines
  `\tcb@use@autoparskip` (drops the `expl3`/`xparse` RequirePackage); (d)
  `\hphantom` math/text split (`\lx@math@hphantom` / `\lx@text@hphantom` with
  `restricted_horizontal`) to stop display-math leaks. Plus pure whitespace
  realignment in `math_common.pool` (no semantics).
- **Rust state (verified 2026-06-25):**
  - (a) `\AtBeginDocument[]{}` — **ALREADY in Rust** (`latex_constructs.rs:3073`).
  - (c) `\tcb@use@autoparskip` — **ALREADY in Rust** (`tcolorbox_sty.rs:17`).
  - (b) color macros — **✅ DONE** (this branch): added `\current@color`/
    `\default@color`/`\reset@color` to `color_sty.rs` (`'0 0 0'`/`'0 0 0'`/empty,
    in Perl order before `\set@color`) + updated the comment. Validated:
    `\makeatletter` smoke error-clean; graphics (8) + tikz (10) suites green; no
    regression. (This was the only real porting work in #2783.)
  - (d) hphantom split — **INTENTIONALLY DIVERGED in Rust** (`math_common.rs:1037`):
    the `restricted_horizontal` wrapping was reverted because it FATALs on
    `\minipage…\hphantom\endminipage` (2004.10048), and the `$$`-leak it guards
    errors in installed Perl too. The new #2783 form uses the same mechanism →
    re-evaluate only if the mode-end fatal is solved; otherwise keep the
    divergence and document it against the new upstream form.
- **Complexity:** **S** (color macros; the rest is verify/divergence-note).
- **Tests:** none new; ensure tcolorbox/quantikz/color regressions stay green.

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

**State of the autonomous methods (2026-06-21) — all tapered; a FRESH cortex
rerun is the clear next step:**
- *Stale 10k error cross-join*: **mined out** — every remaining apparent
  "Rust-only" cluster traced to a SHARED cause (third-party class/pkg neither
  engine binds; author errors; stale pre-fix run). **2026-06-21 re-check via the
  live cortex `document/<id>` API (not the stale ad-hoc join):** the last two
  candidates were BOTH phantom — `1308.2655` "Extra alignment tab" on
  `\lefteqn`/`\multicolumn{N>cols}` is **parity** (Perl 1 error, Rust 1 error —
  Perl's `nextColumn` errors on column overflow too, `Alignment.pm:136-144`); and
  `0710.5692` `equationgroup isn't allowed in <ltx:p>` is **parity** (Perl 2,
  Rust 2). An ad-hoc same-tree cross-join had falsely reported both as "Perl 0";
  the stable cortex DB is authoritative. **Lesson: confirm every cross-join
  "Rust-only" read against the live cortex `document/<id>` API before chasing —
  do not trust a bespoke join's Perl column.** (One genuine *minor* residual on
  `0710.5692`: Rust reports the equationgroup location as `Anonymous String` vs
  Perl's `cosmo_sing_iwa.tex; line 1124` — a source-locator gap, belongs to the
  #47/#92 source-map track, NOT a parity/correctness bug.)
- *Diagnostic-message faithfulness*: **exhausted** — a systematic batch
  comparison (undefined CS/env, missing-number, group/mode close, malformed,
  close-environment) shows all primary messages matching Perl.
- *Structural-skeleton diff on Perl-clean papers* (the silent-divergence method
  that found the REVTeX/OmniBus `\references` fix): now consistently surfaces
  only the DEFERRED families — MathFork/content-MathML (`equation > tags`) and
  document-builder block/paragraph auto-wrap — plus cosmetic/niche cases.
- *Binding-completeness set-diff*: too noisy to be useful — it misses every
  macro defined via `TeX!(r"…")` raw-TeX blocks (single-backslash), so its
  flagged "gaps" are mostly false positives (verified: longtable `\LTcapwidth`
  etc. ARE defined). OmniBus was confirmed structurally complete this way.
- *fatal/TooManyErrors mining (2026-06-22)*: **mined out — ZERO genuine
  Rust-only bugs.** Of 35 `MaxLimit(100)` papers: 24 Perl-fatal (parity), **9 a
  `cp1251`/Cyrillic env artifact** (all `[cp1251]{inputenc}`+`[T2A]{fontenc}`+
  russian babel → ~100 `unexpected:<char>` each; the `cyrillic`/`t2` TeX package
  is missing on this host so `cp1251.def`/`t2aenc.def` are absent — **local Perl
  fails identically**, the cortex Perl=clean came from a host WITH the package),
  2 stale/marginal. Same env-artifact family as the isolatin phantom. **Cyrillic
  coverage fix is host-side (`tlmgr install cyrillic cm-super`), not a code bug;
  an optional surpass-Perl charset-decode fallback for missing inputenc `.def`s
  would convert them without the host package (needs authorization).**
- *fatal/Timeout mining (2026-06-22)*: 18 papers → 16 Perl-fatal (parity), 2
  candidates. `1506.09195` = missing custom `my_paper.sty` + deep expl3/datatool/
  l3fp (local Perl also fatals; Rust runs the conditional runaway to the IfLimit
  guard). **`1707.02464` = the ONE genuine Rust-only bug from all 53 fatal papers:
  Perl completes in 11.76s, Rust hangs to the 60s watchdog** — a custom
  `\narrow` macro's `\hsize`-shrink loop never terminates because Rust's vbox
  `\ht` is `\hsize`-invariant (Perl models paragraph height ∝ `\hsize`). Recorded
  as `STABILITY_WITNESSES.md` Cluster G (open; box-model fix, regression-risky,
  warrants a focused session).
- *error-severity sweep (2026-06-22)*: full cross-join of the cortex `error`
  severity (1189 tasks) on the **same local host** (env-artifact discipline).
  **Parity/env-artifact dominated; ONE genuine Rust-only correctness bug.**
  - `malformed` (162): all parity except **`ltx:itemize` in a `p{}` cell** — the
    p{}-block-content bug (1510.07685), root = **1610.00974 step-3**, now
    **✅ FIXED 2026-06-22** (`f65b80c1c2`, the p{}→VBox port, unblocked by the
    Cluster G box-model fix `7545e07fd6`). `_CaptureBlock_`/listing errors are
    Perl-identical (parity).
  - `latex` (31): all parity. Every package `\PackageError` (`\GenericError`,
    `(ifthen)`, `(newunicodechar)` 189, `(etoolbox)` 187, `(glossaries)` 224,
    `(pgfkeys)`) is shared. The `(babel)` `Unknown option 'russian'`/`'ukrainian'`
    cluster (11 papers, cortex Perl=warning) is a **babel-VERSION env artifact**:
    local babel.sty ≥3.9 (locale-based) errors on the `russian` *option*
    (`russianb.ldf` absent), and **local Perl emits the IDENTICAL single error**
    (0709.3796: Rust==Perl==1). The cortex Perl=warning host had pre-3.9 babel.
    Same class as the isolatin/cp1251 phantoms; not a code bug (a `babel_lang_stubs`
    russian/ukrainian stub would surpass local-Perl + overlap the Cyrillic
    host-side decision → left as-is).
  - `missing_file` (31), `misdefined` (3), `document` (2), `xpath` (2): all parity.
  - `undefined` (890): top-20 whats all parity — the `imsart` bib cluster
    (`\bauthor`/`\bfnm`/`\btitle`/… + `{barticle}`, 16 papers) and `{diagram}`
    (17/19) are **Perl-also-undefined** (Perl LaTeXML ships no imsart/diagram
    binding either). Confirms "undefined = shared third-party CS".
  - `unexpected` (268): the big "Script `_`/`^` can only appear in math mode" +
    "Misplaced alignment tab `&`" clusters are **100% parity** under a FULLY
    PAGINATED cross-join (`_` 109/109, `^` 45/45, `&` 51/51 papers — no math-mode
    detection divergence; these are genuinely-malformed unescaped inputs both
    engines flag). The only "candidates" were the `<char>` inputenc Cyrillic/latin
    env-artifact cluster (0802.1123 isolatin, 1008.0492/1011.5076 babel-russian,
    1009.2998 `[cp866]`+`[T2A]` — host missing the `.def`; same class as Clusters
    A/C/E) and `\end{table}`/1805.00875 (**already FIXED** — see next).
  - **META (2026-06-22): the cortex Rust service data is STALE** (predates recent
    branch fixes). 1805.00875 (dcolumn) shows `unexpected/\end{table}` in the
    cortex report but converts **0 errors on the current binary** (the 2026-06-21
    dcolumn fix is in). So a flagged "Rust-only candidate" may already be fixed —
    **always re-confirm on the current binary** (the genuine finds 1510.07685 /
    1707.02464 were). A **fresh cortex Rust rerun built from this branch** is the
    real prerequisite for surfacing NEW genuine Rust-only correctness bugs; the
    stale data is still authoritative for *parity* and *env-artifact* classes
    (those don't change). **Conclusion: the entire `error` severity is mined out —
    parity + env-artifacts; the one genuine find (p{} block content, 1510.07685) is
    now ✅ FIXED (1610.00974 step-3 port + Cluster G box-model fix, 2026-06-22).**
  - **Current-binary same-host sweep (2026-06-22):** a fresh 24-paper deterministic
    corpus sample (LaTeX2e + LaTeX 2.09 `\documentstyle` + revtex, multi-domain),
    current Rust vs **verbose** local Perl (avoiding the `--quiet` trap). 21 real
    TeX papers (3 were `\documentstyle` 2.09, 1 a misnamed PostScript file):
    **ZERO Rust>Perl divergences — Rust is at-or-better than Perl on every paper.**
    Parity on most (0/0, 33/33); **Rust BEATS Perl on 4** (1509.03503 Perl
    timeout→Rust clean; 1604.03906 3 vs 101; astro-ph0210479 18 vs 101; 1712.01466
    2 vs 3). Confirms the stale-DB mining: no genuine Rust-only bugs findable on the
    current binary without a fresh cortex rerun. Sweep harness:
    `tools/`-style `/tmp/sweep.sh` (grep `\documentclass` misses 2.09
    `\documentstyle` — heuristic note, not a Rust gap).
  - **`warning` severity mined too (2026-06-22) → nothing new actionable.** Of 2208
    warning tasks, the bulk is **user-deferred math** (`ambiguous` 1348 + `expected`
    1181 = `not_parsed` "MathParser failed to match" — content-MathML) + env
    (`missing_file` 590). The small non-math categories are niche graceful-recovery
    warnings, all parity/faithful: `unsupported/multirow` ("Negative row sizes … not
    yet supported") is a **line-for-line mirror of Perl `multirow.sty.ltxml`:27-28**
    (Perl doesn't support it either — implementing would surpass Perl);
    `malformed/{_CaptureBlock_,labels,ltx:Proof}` (1-5 tasks) are graceful fallbacks
    for custom/edge constructs. **All cortex severities (error/fatal/warning) are now
    mined; no unblocked, in-scope, non-surpass Rust-only bug remains findable on the
    current binary.**

**NEXT: a FRESH cortex Rust rerun built from this branch** (needs
`X-Cortex-Token`) is the prerequisite for mining genuine Rust-only *correctness*
wins now that the diagnostic messages are faithful; always re-confirm a flagged
paper on the CURRENT binary before chasing it. Otherwise, the highest-value work
is the DEFERRED focused sessions below (content-MathML, document-builder).

> **2026-06-21 update — reruns IN PROGRESS, first cortex cross-check done.** A
> fresh Rust rerun (`019eea79…`) AND a fresh Perl rerun (started 03:51) are both
> live on `sandbox-arxiv-10k-shuffle`, so per-paper status is in flux (many show
> transient `todo`). A first cortex-grounded cross-check of the **`error/malformed`
> tail** (the richest vein for Rust-only document-builder bugs) — filtered to
> papers where BOTH services are terminal AND Perl lacks the exact `what` —
> surfaced **zero genuine Rust-only structural regressions**. Every apparent
> candidate is either still `todo` in the Perl rerun, or a paper where **Rust is
> at-or-better than Perl**: e.g. `0905.3143` Perl 101 errors→FATAL vs Rust 6
> errors/no-fatal; `1710.08311` Perl FATAL vs Rust survives. (Method script
> pattern: `reports/.../error/malformed/<what>` → per-paper
> `corpus/<c>/tex_to_html/document/<id>`, require Perl status ∈ terminal AND no
> `malformed/<what>` message.) **Re-run the clean full cross-join once both reruns
> COMPLETE** — only then is a Perl=`no_problem`/`warning` vs Rust=`error` signal
> trustworthy.

> **2026-06-21 (later) — reruns now COMPLETE; cross-join reopened.** Rust service
> `oxidized-tex-to-html` on `sandbox-arxiv-10k-shuffle` is 100 % terminal
> (todo=0); Perl `tex_to_html` is 99.77 % terminal (23/9849 `todo`). The
> small-category sweep (xpath/document/misdefined, fully enumerated + per-paper
> cross-checked against the live `document/<id>` API) found:
> - **`1506.09203` — STALE signal, already FIXED on current HEAD.** The cortex
>   DB shows Perl=`warning`, Rust=`error` (`error|xpath|findnodes|()` at
>   `xml.rs:46`), but that Rust status is from the rerun binary `019eea79`. A
>   local repro on current HEAD (`/data/arxiv/1506/1506.09203/`,
>   `Subrepresentation_book_6tag3.tex`, TCI/Scientific-Word + `tcilatex.tex`,
>   ar5iv profile) converts **clean: 0 errors / 0 fatals, no xpath failure, 52
>   warnings** — matching Perl. An intervening branch commit (after the rerun
>   snapshot) resolved the eqnarray/MathFork `findnodes` invalid-context failure.
>   **Lesson reaffirmed: always re-confirm a flagged paper on the CURRENT binary
>   before chasing.** Landed regardless: `xml.rs` `findnodes`/`findvalues` now
>   include the failing XPath string + context-node presence in the error (the
>   old message was just `{:?}` → empty `()`), so any future xpath failure is
>   diagnosable.
> - `0803.1344` (document/open_element_internal): Perl `fatal` vs Rust `error` →
>   Rust at-or-better, not a regression.
> - `1608.07271`, `1802.04240` (misdefined `#`), `hep-th9207093`
>   (misdefined `\list`): Perl=`error` = Rust=`error` → parity (shared cause).

> **2026-06-21 (later still) — the existing rerun (`019eea79`) is now STALE; a
> NEW rerun is required before further mining.** The Rust `oxidized-tex-to-html`
> error data predates this session's fixes (m{}/b{} `\multicolumn`, dcolumn
> empty-todelim, the over-parse/grammar work, etc.), so per-`what` mining keeps
> surfacing already-fixed leads. This iteration checked the highest-cascade
> `error/latex` clusters and ALL were stale/parity/Perl-worse on the CURRENT
> binary: `(newunicodechar)` 1704.05587 (cortex "ASCII character requested" ×63 →
> now PARITY: `\newunicodechar` simply undefined in both, 22=22 identical);
> `(etoolbox)` 1604.02419 (cortex Rust=error but Perl=**fatal** → Rust at-or-
> better); `(babel)` `Unknown option 'russian'` ×11 (witness 0709.3796 now
> Rust=0=Perl=0; minimal `[russian]{babel}` is Rust=1 / Perl=3, the option error
> emitted by BOTH → parity-or-better). **Do not mine `019eea79` further — request
> a fresh Rust rerun on current HEAD first** (needs `X-Cortex-Token`); only then
> is a Perl=clean / Rust=error signal trustworthy. Reliable interim method: a
> direct LOCAL both-engines diff on a small paper sample (ground truth, not the
> stale DB).
>
> **`1506.03557` (`ESSS_2015.tex`) — Rust 49 / Perl 2, PARTIALLY addressed
> (math session, 2026-06-21).** Two distinct roots:
> - **WIDE_PUNCT threshold — FIXED.** A fenced comma-list with an interword
>   control space `\ ` before a signed term (`(3,\ -5)`, `(300,\ -50,\ +50)`,
>   `\textit{Held\_For}\;(300,\ -50,\ +50)`) fell to `ltx_math_unparsed`: the `\ `
>   put 5.0pt `rpadding` on the comma, and `punct_followed_by_wide_space`'s ≥5pt
>   threshold mis-tagged it `WIDE_PUNCT` (a `\quad`-class formula-separator routed
>   through `formulae_apply`, which fails inside a fence). Raised the threshold to
>   ≥10pt (only `\quad`+; matches `filter_hints`). Now parses, matches Perl
>   `vector@(300,-50,+50)`. Regression test in `parse/sequences_and_lists`.
> - **The 42× `XMWrap isn't allowed in <ltx:p>` residual is a WRAPPING leak
>   triggered by the `program` package — ROOT LOCALIZED 2026-06-21, still OPEN
>   (niche, deferred).** Bisection: the 42 leaks come from 3 sections
>   (preliminaries=18, trip_sealin=12, pushbutton=12), and preamble bisection pins
>   the enabling factor to **`\usepackage{program}`** (commenting it → 0 leaks).
>   `program.sty` makes `_`/`;`/`` ` `` ACTIVE in math (`\catcode\_=\active
>   \def_{\ifmmode\sb\else\p@sb\fi}`, lines 535/67-75) and redefines `\(`; the
>   preliminaries math is subscript-heavy (`t_n`, `t_{now}`, …), so under the
>   active-`_` Rust produces unparsed inline math whose bare `<XMWrap>` leaks into
>   `<ltx:p>` while Perl (which has NO program.sty.ltxml — it raw-loads) keeps it
>   `<Math>`-wrapped. Rust loads `program` via the **contrib binding**
>   (`latexml_contrib/src/program_sty.rs`), so the divergence is contrib-binding
>   vs Perl-raw-load. NOT reproducible from `program` + the snippet alone — needs
>   the full preliminaries context (accumulated state). Both the unparsed Z-math
>   AND the leak are recovered in the final output; these are build-time errors.
>   Niche (`program` is rare on arXiv); for a future contrib-binding session —
>   fix in `program_sty.rs` (match Perl's raw-load active-`_` behavior) and/or the
>   document-builder unparsed-math wrapping. The WIDE_PUNCT fix above was the
>   general, landable win from this witness. (Same scan: `1705.04022`
> 16 err `_`/`^`-in-text — re-verify vs Perl before chasing.)
>
> **`1704.05644` (`Paperling_revu.tex`) — CONFIRMED Rust-only (Rust 17 / Perl 0)
> but DEEP/tangled; deferred.** Root: `shadethm.sty` (raw-loaded, no binding in
> either engine) fails to define `\newshadetheorem` in Rust in this paper's
> context → cascade of undefined `{theorem}`/`{hyp}`/`{propgrise}` envs +
> `\shadebox*`/`\shadedtextwidth` `expected:<variable>`. KEY: the *minimal*
> `\usepackage{shadethm}\newshadetheorem{thm}{Theorem}` is **parity-broken** (BOTH
> engines: `\newshadetheorem` undefined) — so shadethm's raw-load is incompletely
> emulated in both, and only the full paper's preamble context makes Perl's
> shadethm work while Rust's still fails. Not cheaply isolatable (bisection of the
> preamble/`\input{macropulko}` did not localize a single culprit; the apparent
> "`\input` breaks it" lead was a red herring — minimal no-`\input` is equally
> broken). The `\Vertex`/gastex errors in this paper are SHARED (gastex depends on
> pstricks/pst-pdf; both engines fail identically in isolation). A proper
> `shadethm` binding (which neither engine has) would be the real fix — surpass-
> Perl R&D, not strict parity. Do not chase piecemeal.

**Beyond-parity coverage candidates (#2 track, surpass-Perl — defer while
strict-parity is #1):** `arximspdf`/`imsart` support (16+ IMS papers aop/aos;
needs a bundled imsart.sty since the host lacks it); `jpconf` class → iopart
(18+ IOP-conf papers); theorem/mdframed-in-figure schema (`figure_mixed_content`,
Open task §1).

---

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
  > toward-Perl), it needs explicit scope sign-off before undertaking; B (below)
  > is the contained first step (~5 fixtures).
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
- **N-ary bare-operator listing** (content-loss already FIXED `a75fbf17ed`):
  `\[ + - \times \div \]` → Perl `list@(+,-,*,/)`; Rust now marks unparsed with
  ALL tokens preserved (the coverage guard rejects the exhausted-early prefix
  parse). Remaining = the N-ary upgrade: `anyop anyop` → recursive
  `compound_operator_2` list (its own `// TODO`). Ambiguity-sensitive. (Root
  cause was the marpa fork's `Parser::read` breaking on `is_exhausted()` before
  the token source drained — `marpa/src/parser/mod.rs:130`.)
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
  `a \le b \quad \stackrel{?}{\ge} \quad c` → Perl `a <= list@(b, >=^?, c)`, Rust
  unparsed. The scripted-relop atomic fix (`4a5ebf29f7`) cleared standalone list
  items but not a relop-item inside a relation's list-RHS.
- **`\underset`/`\overset` over an ARROW with a multi-token script**:
  `x \underset{n\to\infty}{\to} y` — the under-script reads `n@to@infinity`
  (apply) where Perl groups `(n to infinity)`. Same ARROW-as-applied-function
  family as `f(a,b)`.

CAUTION: new VERTBAR/fence grammar rules can collide with package-built
structures — always cross-check the affected fixture against Perl before
assuming a regression (the norm rule "regressed" physics_test, but Perl matched
the new output, so it was a parity *fix*).

## DefMathRewrite `\WildCard` subscript bug (focused-session item)

`DefMathRewrite` with a `\WildCard` SUBSCRIPT pattern doesn't demote the match
(witness `math/simplemath`): `f_\WildCard → role=ID` should make `f_1(a+b)` =
`f _ 1 * (a+b)` (Perl), but Rust produces `Unknown@() * (a + b)` — the
`f_\WildCard` rewrite isn't firing (or loses to the sibling `f → FUNCTION`
rewrite), so `f_1` stays a FUNCTION and gets APPLIED. The non-wildcard
`f_D → DIFFOP` works, so it's the `_\WildCard`-subscript match/ordering in
`latexml_package/.../latexml_sty.rs` (`compile_declare_pattern`). Niche
(binding-author feature, rare in real arXiv); the fixture encodes the buggy
output.

---

## Open tasks (actionable)

### 1. `ERROR_DEBT` test-gate drain
The harness error-gate (`latexml_oxide/src/util/test.rs`) fails a test at zero
debt to force removal once fixed. Remaining:
- **`figure_mixed_content`** — `ltx:theorem` not allowed in `ltx:figure` (Perl
  also errors 1). True fix = **schema expansion** (theorems/mdframed in figures).

### 2. `\gls`/`\acrshort` in MATH mode (1705.10306) — suspected Rust gap, UNVERIFIED vs Perl
293 errors `ltx:XMTok isn't allowed in <ltx:glossaryref>` (the "Perl 1" figure is
**unverifiable** — 1705.10306 is in NO cortex corpus and Perl 0.8.8 times out on
glossaries on this host, so it cannot be cross-checked; treat as suspected, not
confirmed): a glossary
command in math mode forces the `glossaryref` content (#3, the link display
text) as math → bare `<XMTok>`, which `Inline.model` rejects. **Diagnosis
re-narrowed 2026-06-21** (earlier "document-builder / Math-not-auto-openable"
theory DISPROVEN): on the SAME host tree the current binary is **byte-identical
to Perl** for `\textbf`/`\emph`/`\href` in math (general math-in-text is
faithful); `ltx:Math`/`ltx:XMath` are **not** autoOpen in either engine (so no
auto-open path), and `ltx:glossaryref` has **no** autoClose in either (faithful,
so it can't float its content out like `emph` does). Most likely root: Perl's
**raw-loaded `glossaries.sty`** typesets the term as TEXT (`\glstextformat`/
`\mbox`), so Perl's #3 is PCDATA — the Rust divergence is in the raw-load
display chain, **not** the document builder. **STILL BLOCKED** on a runnable
Perl reference: glossaries times out in Perl 0.8.8 on this host (datatool/
l3regex) even without `\makeglossaries`; the `glossary.{tex,xml}` fixture has no
math case; witness 1705.10306 is not in the local corpus. Repro + full notes:
`docs/reproducers/glossaryref_math_xmtok.tex`.

### 3. PR #248 B1 — re-entrant `&mut Document` UB (runtime-bindings), accepted caveat
The Rhai constructor trampoline re-mints `&mut Document` (Stacked/Tree-Borrows UB
under a re-entrant `\wrap{\myemph{..}}`). Consolidated to one audited
`script_bindings/mod.rs::with_doc` site + documented; the review's checked-guard
fix **deadlocks** `Document::absorb`. **Optional future work:** make re-entrancy
sound-while-succeeding (interior-mutable `Document` or a core handle around
`do_absorption`). Not a blocker; `runtime-bindings` stays on by default.

### 4. 0.7.0 release — release-prep LANDED; tag pending
Version bumped, `runtime-bindings` in the artifact, `.deb` deps, CHANGELOG/README
done. **Remaining:** tag `0.7.0` on `main` → `release.yml` runs the TL-window
`dumps` + macOS arm64 leg + publish (each first-exercised on that tag).

### 5–6. LANDED 2026-06-22 (see "Landed this session" above)
- **Post-processing log parity** (`512dbc1ba2`, `9524d2e179`): `cortex.log` carries
  core+post. **Residual (cortex-side owner):** wire `cortex_worker.rs::convert_archive`
  to `run_post_processing_logged` + fold `max(core, post.status_code)` into
  `Status:conversion` (Perl `LaTeXML.pm` L631-634).
- **Graphics never ships a raw `.eps`/`.pdf`** (`80b4438385`, `604951c232`): three
  guards → a `<graphics>` without `@imagesrc` renders `ltx_missing_image`. Known
  post-orchestration deltas (not blocking, broader parity): `PictureImages` absent
  (Rust = regex inline-SVG), `SVG` regex extractor, no `prescan`.

---

## Deep deferred families (parked — large or shared; dedicated sessions)

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
  `EXPECTED_ID_XMREF_DESIGN_2026-06-08.md`. **QUANTIFIED 2026-06-22: this is the
  #1 remaining Rust-only divergence** — `warning/expected/id` is **1005 cortex
  tasks** ("Cannot find a node with xml:id='S…E…m1.N'" from
  `latexml_math_parser/src/parser.rs:2840`; math-node ids, so genuinely the
  content-arm/MathFork XMRef cluster). It's a large Rust-only WARNING excess vs
  Perl (e.g. 0704.3530 Rust 152 vs Perl 9 warnings) — NOT parity. The prime
  candidate for the deferred content-MathML dedicated session; do NOT pick at it
  piecemeal (user directive).
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
    Rust is error-clean and schema-valid, so this is a COSMETIC structural
    divergence, not a validity bug. Same root: Perl's builder opens a block for
    a horizontal run inside a block-context element; Rust doesn't.
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
`pdftocairo`→`inkscape`. Subprocess `exec` (no GPL linking). Apt: `poppler-utils`
(req), `mupdf-tools` (rec), `imagemagick+ghostscript`, `inkscape`.

### Other tracks (separate docs)
- Performance: `PERFORMANCE.md` (P1 math/large-doc open; P2 allocation partial).
- Release gates: `RELEASE_CRITERIA.md`. Releasing: `RELEASING.md`.
- Completed missions (archived): strict-LoadFormat dump parity, Marpa ASF
  migration, distribution-readiness, the 500K/1M warning-corpus mission, and the
  diagnostic-message faithfulness pass (2026-06-20) — see `docs/archive/` and
  `git log`.
