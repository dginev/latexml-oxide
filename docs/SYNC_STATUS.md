# Engine Sync Status — Active Worklist

> **DO NOT downgrade Errors to cheat the task.** If Perl LaTeXML converts a paper
> without a downgrade, the Rust translation must match by improving the core
> engine — never by silencing diagnostics. New downgrades require explicit proof
> Perl emits the same severity on the SAME paper, else they hide a real gap.
> (User directive 2026-05-15.) Always classify with `latexml --verbose`, never
> `--quiet` (which hides Perl's `Error:` lines); cross-check pathological inputs
> with `pdflatex`.

## How to read this file

**Start at "Ranked worklist" below and take the top unblocked row.** Everything after it is supporting detail.

- **Ranked worklist**: every open item, ordered; first, always.
- **Silent-loss inventory**: open losses of authored content or markup with 0 errors (blocks the 98 % claim).
- **Rust Error Fixes**: Rust-only defects fixed (with their KNOWN_PERL_ERRORS cross-reference); read before filing a defect as Perl's.
- **Current status**: where the corpus measures live.
- **Open items**: detail behind some ranked rows; secondary residuals.
- **Standing policies**: rules on *how* to fix (CLI flags, stubs, divergences).
- **Parked families**, **Reference**: pointers to dedicated docs; stable facts.

Rules that keep this file honest and mergeable:

1. **Verify a status label before acting on it, and before deleting it.** Check the **named guard test** in the tree,
   or `gh issue view <N>` / `gh pr view <N>`. **SHA-ancestry does not work**: the repo squash-merges, so a branch SHA
   quoted here is never an ancestor of `main`.
2. **One row, one `###` section, one fact per bullet line**, so two edits to different facts merge cleanly. No table
   row longer than ~300 characters.
3. **Writers never edit this file.** A payload writer hands the main session a "SYNC note"; the main session folds it
   in at bundle time (user 2026-11-09: its rows conflicted on every merge).
4. **This is the brief actionable list; landed work leaves.** Day-by-day logs live in `git log` and `docs/archive/`.
   When an item closes, move it, in conclusion form, to the month's `docs/archive/SYNC_SESSIONS_YYYY-MM.md`; an open
   residual stays here under its row.
5. **Keep it under ~120 KB.** When a section outgrows ~100 lines it has become its own subject: give it a doc under
   `docs/` and leave a one-line pointer.

*Last compaction: 2026-11-09 (SYNC1) — every table row became a `###` section of one-fact bullets; the landed rows (run 336, the bundle rows int119a-int152a, int133a, the pre-flip leftovers' history, XP, R4), the long rows' landed history and the Rust Error Fixes' bodies moved verbatim to [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1); before: 2026-10-23 — the batch and integration narratives (63a–int73, 7,464 lines) and 51 landed rows moved to `archive/SYNC_SESSIONS_2026-10.md`, the four open-row detail sections (751 lines) to `perfect_kernel/OPEN_ROWS.md` (8,677 → 426 lines); before: 2026-10-04 — the run-329 narratives (61r-62f) moved to `archive/SYNC_SESSIONS_2026-10.md`, the parked math rows to `math/MATH_PARSE_RESIDUALS.md`, landed rows removed; before: 2026-10-02 — the K13 stage-2 findings and the rc131/rc132 + 57cn–59l rows moved verbatim to `perfect_kernel/archive/LEDGER_PHASE57_58_2026-10-02.md`, their open residuals kept here. Prior: 2026-09-27 (closed R1, R2, R5 and the 2026-08-02 status snapshot to `archive/SYNC_SESSIONS_2026-09.md`); 2026-09-03 (949 → ~320 lines; 1276 → 953); 2026-08-18 (1462 → ~890); 2026-07-25 (1979 → ~500).*

---

## Ranked worklist — start here

Ordered by importance (2026-10-04 review): closing the perfect-kernel goal, then run 329 and its residuals (they seed
the coming arXiv success-rate task, not started), the rulings that block rows, the work safe to do while the run
finishes (T1, T2, F1, F2), then the older parity rows. Within a tier: **does it reproduce today** → **is a real user
affected** → **is it unblocked** → **effort**.

| # | item | state | size |
|---|---|---|---|
| R | Road to the main merge (user 2026-11-09: "keep planning out all steps … towards completing our main goal"; … | **Order, each step reviewed and bundled:** (1) **single engine clean** — the deletion pass DEL1-DEL4 merged … | OPEN — step (1) in progress |
| 0 | The TeX Live manuals: every regression addressed before the merge (user 2026-11-06: "This remains the primary … | met; |  |
| 0a | M3: the manuals' remaining regressions | F1-F10 done, 3 deferred | S |
| 0b | The frontmatter's old engine out of every decision before the merge, and all it leaves unused deleted (user … | **Route D (user 2026-11-06):** the experiment, then one switch for every paper with an author block; | about 12-15 batches |
| 0c | Noticed during the performance work, no fix yet. | open | S each |
| R8 | Perfect-kernel program | **CLOSING**: bar confirmed (user 2026-10-04); | program |
| G | Run 336 | closed; archived | — |
| KC | Kernel capability program | **ORDERED**: K24 LANDED (`ab_audit`, p0 `1e47551e6c`); | program |
| MS | TeX's math-group recovery | **PARTLY DONE 65n**; | M |
| XP | A detail-less XPath failure in post is a resource failure, not 100+ Errors | closed; archived | — |
| BD | A `\DeclareBoldMathCommand` delimiter after `\left`/`\right` | **OPEN**, RED only | S-M |
| A2 | Fidelity audit round 2 residuals | **OPEN** (12 only); | small |
| A3 | arXiv html_feedback subgoal | **OPEN**; | medium each |
| A4 | Full arXiv fidelity audit | **QUEUED, LAST**: after the run-336 Error/Fatal subgoal and the queued payloads reach diminishing returns | large |
| G1 | Run-329 open residuals | **NEXT** (arXiv success-rate task) | medium each |
| D1 | Rulings | RULING | small-medium |
| T1 | Repro suite move | **OPEN**, not started (user 2026-10-02) | large |
| T2 | Test API stage 2 | **OPEN** (stage 1 landed 61a) | medium |
| CI | After the merge into `main`, not before (user 2026-11-06). | **OPEN** — localhost study | medium |
| TLY | A target TeX Live year per conversion | **OPEN**, design (needs the year source: submission date vs a CLI flag) | family |
| AT | AutoTeX emulation | **QUEUED** (near the end) | family |
| F1 | Per-encoding text commands | **PARTIAL** | medium |
| F2 | Small RED repros | **OPEN**, safe now | small each |
| R3 | Bibliography-absence campaign | **R3d next** (12 papers left, unblocks alignment macro `&` splits) | medium |
| R8b | forest.sty full support | **OPEN**, recorded | large |
| R4 | `--preload=<cls>` trips the LaTeX hook stack (`Extra \PopDefaultHookLabel`) | closed; archived | — |
| R6 | `Collector::rescan` Refactor for Generated Backmatter | **OPEN**; | medium |
| R7 | Presentation-MathML **F5** Linebreaker | **OPEN**, full linebreaker feature gap needing a port-or-drop scope decision | family |

### R — Road to the main merge (user 2026-11-09: "keep planning out all steps … towards completing our main goal"; inventory `agent_reports/2026-11-09_PREMERGE_inventory.md`, 30 …

- **Road to the main merge (user 2026-11-09: "keep planning out all steps … towards completing our main goal"; inventory `agent_reports/2026-11-09_PREMERGE_inventory.md`, 30 items; supersedes the 2026-11-05 close order)**
- State:
  - **Order, each step reviewed and bundled:** (1) **single engine clean** — the deletion pass DEL1-DEL4 merged on `pk/int154a` (tests −349, the switch, the old driver −5.8k, the S2b writer and census −12.2k; output byte-identical), DEL5 truthful names, Z18a the API's Cut/Kind out of `author_block/` (byte-identical), Z18 one batch per binding family (llncs, eptcs, omnibus, authblk, aas, ieeetran, sv, aistats, stores/fields, IJCAI; A/B each) (Z18c: the emitter's rules the llncs reroute needs, OD #984; the reroute itself next, its blockers in `agent_reports/2026-11-09_Z18c_report.md`) → `author_block/` gone (row 0b);
  - (2) **every RED seen on a real paper fixed** (user 2026-11-09): the silenced Errors FIRST (row 0b's item 39: an undefined command in an author field, 18 A/B papers), then the content residual (`FRONTMATTER_CONTENT_RESIDUAL_AT_EMIT2.tsv`: every word kept or counted), ejpecp, NE5's whole blocks, the footnote-counter whole block, the ORCID logo, 77p's grouping;
  - synthetic-only stay REDs;
  - (3) **Option C P1-P6** (all segmentation on the digested reading; `FRONTMATTER_SINGLE_ENGINE_PLAN.md`) after Z18, with the plan's DoD items, frontmatter code and test compaction (table-driven, one conversion per fixture);
  - (4) **the bars**: truth set, then a fresh holdout v4 read once — 98/98 on dev3/devh/dev20k and v4, longtail ≥ off with no regression (user 2026-11-09);
  - (5) **non-frontmatter**: the generalization audit re-scoped and done before the merge (user 2026-11-09), T1 the repro suite move, the perf + RAM pass's remaining levers, row 0a's deferred manual REDs decided, workspace cleanup (keep-list to the user);
  - (6) **close**: goal docs current, the closing manual sweep on the final head (row 0's G1-G5), the 2605/2606 cortex reruns (G6; the fleet rebuild needs the user's OK), the merge PR (the user confirms).
  - **After the merge:** CI ≤ 30 min (user 2026-11-06), the arXiv rerun, A4, AutoTeX, forest, the ar5iv-css port, a release.
- Detail: OPEN — step (1) in progress
- Detail: L
- Detail: this row; the inventory

### 0 — The TeX Live manuals: every regression addressed before the merge (user 2026-11-06: "This remains the primary goal of this branch").

- **The TeX Live manuals: every regression addressed before the merge (user 2026-11-06: "This remains the primary goal of this branch").** **Met at sweep #152 on `pk/int142a` and held at #154 on bundle int143b and #156 on bundle int144b: goal set 1,552 of 1,552 pass G1-G4, 100 %; no new Fatal or timeout against #149 (10 gone); manuals over 120 s 2 → 0.** Open under this row: the deferred REDs of row 0a.
- Sweep #150 on `d6e175bacb`: goal set 1,531 of 1,552 pass G1-G4, 98.65 %, against 99.81 % at #149;
- G1 10, G2 6, G3 5, G4 0;
- in scope clean 1,582 of 1,602.
- None is from Bundle int140a (21 run on int139c: equal).
- `chain/bundle_chain.sh` now runs the sweep.
- Batches M1, M2 fix clusters A, C1-C9 (the worklist's `Done` column; guards `perfect_kernel_batchm1`): sweep 902 on M2 gives 1,550 of 1,552, G1 0, G2 0, G3 0, G4 2
- State: met; margin batches landing
- Detail: `pgf-interference-en/-de` pass with a thin margin until `pk/P2` (150 s to 33.6 s); clusters B, T and the `?` canaries left, row 0a
- Detail: [`perfect_kernel/MANUAL_REGRESSIONS.md`](perfect_kernel/MANUAL_REGRESSIONS.md)

### 0a — M3: the manuals' remaining regressions

- **M3: the manuals' remaining regressions**, F1-F10 fixed (`perfect_kernel/MANUAL_REGRESSIONS.md`, guards `perfect_kernel_batchm3`).
- **Deferred, each with its evidence:** F11 `\fontname\font` under NFSS (typog-example: `cmr10` vs `LinLibertineT-tlf-t1`, tex_fonts.rs:150, `\projectoutfontname##1-##2-##3` runs away; needs `\font@name` to TFM through `.fd`; `\fontname` in 2,027 census papers);
- `\includegraphics` global vs `\lx@Gin@include` local (graphicx_sty.rs:133, :163; draftfigure);
- tuzuk's 2 Errors vs pdflatex's 1.
- **floatrow-rus is no deadline defect:** a TeX loop in xkeyval's `\XKV@f@r` under a `\the` its own cascade redefined (doc.sty v3 has no `\Describe@Macro`, so `\DescribeMacro{\DeclareNewFloatType}` runs the command; pdflatex stops at the absent picins.sty, l.95).
- `--timeout` stops it (Fatal:Timeout at 20, 30, 170 s);
- the sweep's 124 is run_doc.sh's outer `timeout` of the same length winning the race.
- Open on it: `\def\a}` (KPE #1216, in gullet.rs) and a Timeout's 39-byte XML (`digest_internal`, "not recovering", R35.A).
- Left of F4: an `\item`'s `\everypar` (KPE #1214), tabularx's X, tabulary's columns, wrapfig's body (RED `boxes-groups/everypar_in_tabularx_tabulary_wrapfig_red`).
- **The review's fix round (M3r2):** `lstlisting` declared at `\begin{document}` (KPE #1217; 2405.15073), `\end`'s own test of `\protect` with a siunitx `S` cell's read of it (2505.21533, 2105.06936), the case changer's `\begin`, `\end`, pgfmath's array text (OD #973), tabularray's `preto`, `appto` (OD #976).
- **REDs:** `graphics-tikz/pgfmath_string_and_units_in_arithmetic_red`, `graphics-tikz/pgfmath_string_keeps_protected_commands_red`, `expansion-primitives/afterassignment_through_a_toks_assignment_red` (KPE #1215's sibling, nothing relies on it), `alignment-bindings/tblr_inline_expl_command_in_a_key_red`, `listings-algorithms/listings_numberbychapter_red`.
- **Post-release (user 2026-11-06):** Timeout handling: `run_doc.sh`'s outer `timeout` margin, `status=124` on the Timeout Fatal, `scoreboard.py` and `sweep.sh`'s reading of it, a Timeout's salvage.
- **Left by the re-review (revM3r2P1), synthetic, each a RED repro or an OD #973/#976 line:** `width({"a"})` 0.0 with no Error, a sublist of one number returned as text, a spanned cell's `preto` lost with no Warning, an `S` cell ended by a `\let` copy of `\end`.
- `\end` now loses its meaning too if a spurious message under `\globaldefs` leaves a display `\protect` global (no oracle: pdflatex stops; `\begingroup\globaldefs\@ne\PackageWarning{p}{note}\PackageInfo{p}{n}\endgroup`).
- **Noticed by M3r2, no fix yet:** `xpeekahead`'s lost "something else" lines and `pkginfograb`'s missing paragraph (both match the base, so not M3's), and book-class listings print "Listing 1" where pdflatex prints "Listing 1.1" (`listings_numberbychapter_red`; Perl shares it, a fix changes `xml:id`s).
- State: F1-F10 done, 3 deferred
- Size: S
- Detail: `perfect_kernel/MANUAL_REGRESSIONS.md`

### 0b — The frontmatter's old engine out of every decision before the merge, and all it leaves unused deleted (user 2026-11-06)

- **The frontmatter's old engine out of every decision before the merge, and all it leaves unused deleted (user 2026-11-06)**
- State:
  - **Route D (user 2026-11-06):** the experiment, then one switch for every paper with an author block;
  - a failed experiment means both engines fenced at the merge.
  - Z0-Z3 done.
  - **The experiment passed its score gates** and showed what a score can't see: creator XML changed on 47 % of agreeing papers, 805 tests move, whole-dev3 98.31 per creator before repairs (`agent_reports/2026-11-07_S0_report.md`, the plan's S0 row).
  - Z4a, the emitter with its markup, and Z14, the reading's repairs, landed (int144b);
  - today's state, the batches in flight and the next steps are the plan's "Where it stands".
  - **Then all token segmentation onto the digested reading before the merge (user 2026-11-07, option C; the plan's "Segmentation is digested", P1-P6)**
- Size: about 12-15 batches
- Detail: `parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md` "The path"

### 0c — Noticed during the performance work, no fix yet.

- **Noticed during the performance work, no fix yet.** `source3` output depends on RSS (`--max-memory=8192` gives two hashes on the base, `</p>` closing newlines, W 65 vs 61; the native evaluator isn't involved).
- `SASnRdisplay` is 1.04× from listings 79z/80b and gains a Warning nobody has root-caused.
- pgfmath's native functions are f64 by Perl's design (`pgfmath.code.tex.ltxml:182-218`): 65 of 549 exact against pdflatex, 22 of 54 boundary probes differ, and 11 `int`/`floor`/`ceil` probes land on another integer (`int(100*sin(30))` 50 vs 49, `floor(log2(8))` 2 vs 3);
- no paper is known to depend on it, and a study waits for one.
- **Post-release (user 2026-11-06):** a Timeout's 39-byte XML and `run_doc.sh`'s outer `timeout`;
- no work on timeout cases before the release.
- State: open
- Size: S each
- Detail: `performance/PERFORMANCE.md`

### R8 — Perfect-kernel program

- **Perfect-kernel program** (branch `perfect_kernel`; the TL-manual corpus; generalized capabilities K1–K18 in [`perfect_kernel/KERNEL_CAPABILITIES.md`](perfect_kernel/KERNEL_CAPABILITIES.md)) — goal bar met at sweeps #146-148 (G1–G5);
- close status in [`PERFECT_KERNEL.md`](PERFECT_KERNEL.md)
- State: **CLOSING**: bar confirmed (user 2026-10-04); ar5iv-css PR #54 open; run 329 completes
- Size: program
- Detail: [`PERFECT_KERNEL.md`](PERFECT_KERNEL.md)

### G — Run 336

- Closed (**TRIAGED** (2026-10-10): 1,571 clusters of the 16,827 (`~/data/pk_agents/main/run336/worse_306_336.tsv`, …); the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Row G".
- Open facts carried from the row:
  - 287 error → fatal — 204 TooManyErrors (107 = longtable undefined under aa/pasj00/emulateapj, a 65b regression, batch 65h), 59 cortex never_completed (dispatcher lease clock started at batch fetch, fixed in CorTeX a5bbbee5, not yet deployed), 18 PushbackLimit;
  - a Semiverbatim URL contact's `~` prints as the OT1 tilde accent `˜` (`\urladdr{…/~y}`, Perl the same; RED sectioning-frontmatter/url_contact_keeps_tilde).
  - Fixed in 64p (run 336's loops, KPE #475/#568, OD #315): a class's `\renewenvironment{thebibliography}` that a binding's hook runs unlocked is refused, the lock read as a value as Perl reads it (wiley2sp & co, 132), and a class's own bibliography list — `{thebibliography}` `\let` to it (ajour, cjour, IEEE-Con-Sys-mag) or its `{references}` — gives way to the kernel's at the first `\bibitem` (RED: a class end printing text after its list prints it before the entries; a class bibliography built on `{enumerate}` leaves a group open — both synthetic; OD #484, KPE #569);
  - a document's `\mathaccentV` accent (21; its `tex` attribute now reads `\mathaccent 28766{G}` where 62d's named reading gave a `.bbl`'s `\hat{G}` — a `\mathaccentV` reversion is open).
  - RED: keyval runs a key given no value that has no default with an empty value (keyval.sty:52-56 errors and skips it; csquotes `[english]` loops, 5; `macro-state/keyval_bare_key_without_default`).
  - open: ptapap `\let\oldfnum@figure=\fnum@figure` undefined (92 papers).
  - open:, IEEEeqnarray `[decl]`/struts (its rule rows landed in 67x), 64 more mode-less `"#body"` DefEnvironments (census in `agent_reports/2026-10-10_payload_64ra.md`).
  - Open RED: a picture label mixing text and math overprints in the SVG (graphics-axodraw/picture_label_mixed_text_and_math, hep-ph0509118);
  - Fixed in 64u (OD #493-496, KPE #582): bindings for journal styles neither in TL nor shipped — l-aa/laa/l-aa-ps as the `aa` class with `\thesaurus` codes, argument-less `\keywords` and `$^n$`-marked institutes (843), epl2/epl (740), OSA opex3/ol2/osajnl/osajnl2 with appending authors and email lines and `\homepage` placed by mark, spelled name or group through the new kernel `\lx@add@contact@placed` (759), REVTeX 2.1 `{title}`/`{instit}`/`\receipt`/`\nonum` (750);
  - No generic fallback for a missing `\usepackage` (user ruling 2026-10-10).
  - Open: under `\documentstyle[aps]{revtex}` the OmniBus overlay gives an `\author{A and B}` group's address to B only (RED `revtex3_omnibus_author_group_address`; ruling asked);

### KC — Kernel capability program

- **Kernel capability program** (user 2026-10-11): the open gaps grouped by the missing mechanism behind several of them — K24 one audited A/B comparator (P0), K20 gullet expansion fidelity (four tex.web algorithms: column ends by command code, `\left`/`\middle`/`\right` primitives, §395 extra `}`, peeking macros as one unexpandable step; absorbs queued 64n/64o/64v), K21 environments as TeX does them (`Streamed` vs `Captured`; absorbs the `#body` audit), K22 binding provenance and ownership (FileIdentity, owned sets, corpus-copy audit; version drift and author-shipped copies), K23 the author-block event model;
- next tier C3 placement policy (11,012 `malformed:` first errors), C6 content-aware emptiness, C7 string values for attributes
- State: **ORDERED**: K24 LANDED (`ab_audit`, p0 `1e47551e6c`); manual sweep #149 → K20 G4 → G2 → G3 → G1 → K21, K22's read-only census/audit in parallel → K23 shadow mode
- Size: program
- Detail: [`perfect_kernel/KERNEL_CAPABILITIES.md`](perfect_kernel/KERNEL_CAPABILITIES.md) K20–K24; evidence [`KERNEL_CAPABILITY_GAPS_2026-10-11.md`](perfect_kernel/KERNEL_CAPABILITY_GAPS_2026-10-11.md)

### MS — TeX's math-group recovery

- **TeX's math-group recovery** (main session 2026-10-12, task #204) — **65n landed for inline formulas outside alignments** (OD #553): a math shift meeting a `{`/`\begingroup` group opened in the formula inserts its closer and ends the formula there (§1193 + §1064-1065; 2007.11971 13 → 1 errors, 179 → 0 run-together runs, text words 5,268 → 9,865), a `\par` or a DefEnvironment's end in it is "Missing $ inserted" (§1046-1047), a `}` at its math shift ends it when it closes a construct digested as a unit (a box body, an argument digested on its own — content first by main-session decision: `\mbox{$x} after` keeps "after" as text where pdflatex swallows it into the box) and is deleted otherwise (a plain `{…}` group included, as TeX: `{\bf $x} y$ after.` keeps "after." text), one Error "Extra }, or forgotten $" either way (§1069; 1710.02165 29 → 21);
- display math stays tolerant of `\par` (no §1047, by decision);
- since 66w (user ruling 2026-10-14) the `\par` keeps an inline formula open when a math-only token precedes the next `$` (a formula split by a blank line, Perl's content; hep-th9402135).
- REDs dollar_closes_open_script_group, par_in_inline_math_inserts_dollar, stray_brace_then_dollar_keeps_text GREEN.
- **Open**: (1) a formula in an alignment keeps the old recoveries — a cell's `$` may be its template's (math9805003's undefined `{Sb}` in an `align*` lost the alignment when its row `$` closed the `\begingroup`);
- an author's cell formula needs the template's `$` told from the author's (RED paragraph-mode/cell_end_in_inline_math_inserts_dollar; 1210.0368 and 1907.09124, whose deferrals sit in alignments, unchanged);
- run 363 added 1906.07043 (iopart + breqn, which loads amsmath: the "Old form `\cases`" error left `$x>l_0$` inside `{equation}`'s alignment, where a lone `$` does not end the display as §1197 "Display math should end with $$" does — 101 Errors + TooManyErrors where pdflatex gives 9 and a PDF; since 77q the old form is Plain TeX's `\cases` after the error, 1 Error, KNOWN_PERL_ERRORS #993);
- (2) display math has no §1047 (a blank line in `equation` is common), so its stray `}` keeps 65a's inserted `$` (f2 `\[ x } y \]`, a13);
- (3) DONE: boxes-groups/mbox_math_stray_brace_ends_box_not_math GREEN by the unit-construct rule (round 3; round 2's "partner outside" rule lost a1/a2/a16, reverted);
- its residual, a forgotten `$` in a plain group (`Before {\bf see $x} more text.`) runs "more text." as math to the `\par`, as pdflatex — possible refinement by `$`-parity in the rest of the paragraph (odd: delete; even: end at the `}`) (RED paragraph-mode/forgotten_dollar_in_a_plain_group_keeps_the_text);
- (4) the `}`-meets-`\left`-group deletion in inline formulas (65d round 7, h1/d2/e1-e4) is unblocked by (§1065 at `$`) and not done (RED paragraph-mode/brace_meeting_left_group_in_inline_math_is_deleted);
- (5) hep-lat9811036 / nucl-th0401019 run text together from another open construct, not this one
- State: **PARTLY DONE 65n**; (1) next
- Size: M
- Detail: OD #553, #530; `agent_reports/2026-10-12_payload_65n.md`, `_65d.md` rounds 5-7

### XP — A detail-less XPath failure in post is a resource failure, not 100+ Errors

- Closed (**FIXED 80m**); the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Row XP".
- Open facts carried from the row:
  - Fix: route it to the same cause-carrying `Fatal:oom` path (one Fatal, the child recycled, retried per the 2026-10-15 reused-child ruling) instead of an Error per node;
  - a fresh-process `ulimit -v` sweep of 2101.04438 (1.2-7.0 GB) never reproduces it (the Rust side fails first), so it needs a reused child.
  - Guard `latexml_post document::tests::libxml_oom_is_told_from_an_xpath_error` (the fleet condition itself needs a reused child under RLIMIT_AS and is not reproduced in a test) **FIXED 80m**

### BD — A `\DeclareBoldMathCommand` delimiter after `\left`/`\right`

- **A `\DeclareBoldMathCommand` delimiter after `\left`/`\right`** (65j round 3 review): bm.sty's `\bm@define` (:307-316) `\xdef`s `\blangle` to a bold `\delimiter` (`\bm@delimiter`, :416), so `\DeclareBoldMathCommand\blangle{\langle}` then `$\left\blangle x\right\rangle$` is a stretchy bold ⟨ in pdflatex (0 errors);
- ours defines `\blangle` as `\bm{\langle}`, a constructor `\left`'s delimiter scan cannot read: Warning "Missing delimiter (. inserted)" (naming `\bm` on the 65 bundle, `\lx@bm@bold` on 65j), a null `\left` and a non-stretchy bold ⟨ after it.
- Witness 1205.4484 (`\boldlangle`/`\boldrangle`/`\boldlvert`/`\boldrvert`).
- RED `boxes-groups/bm_declared_bold_delimiter_after_left`
- State: **OPEN**, RED only
- Size: S-M
- Detail: OD #544; `agent_reports/2026-10-12_payload_65j.md` round 3

### A2 — Fidelity audit round 2 residuals

- **Fidelity audit round 2 residuals** (silent content/markup losses in "successful" run-336 papers): only the title-`\cite` tooltip (12, low) remains
- State:
  - **OPEN** (12 only);
  - re-read 2026-10-25: entics `\thanksref{ALL}` (10), `$(a)$` marks (15) and the llncs check (16) FIXED 77t, sectsty (11) FIXED 78a;
  - 63j-63m landed 10 classes (63m: group affiliations 14, names past `\\` 13);
  - acmart comma list (2b) 63x (KPE #549), an.cls `\abstract{}` (9) 77b (silent-loss #23)
- Size: small
- Detail: §"Fidelity audit round 2"

### A3 — arXiv html_feedback subgoal

- **arXiv html_feedback subgoal** (user 2026-10-08): 5,970 reader reports → 4,819 papers → 44 problem classes;
- each class verified on head, persisting defects ranked HF1-HF13 — tcolorbox listings one line high (HF1, 37 papers; root-caused: the listings block has no sizer), merged authors of the report set (HF2, 118), binding gaps vs shipped classes (HF3), acmart `\Description` raw TeX and labels on its hidden note (HF4), box font sizes (HF5), deferred typesetting (HF6), error clusters (HF7), pgf text anchors vs amstext (HF11), `\includestandalone` and TikZ residuals (HF12), table pruning (HF13)
- State:
  - **OPEN**;
  - baseline sweep + 4 classes verified (raw-latex-leak 12/15 fixed, tcolorbox 6 fixed + 5 viewer, tikz 6 fixed + 1 viewer, table 10 fixed + 3 viewer, of 15 each);
  - 63m landed HF2 shapes;
  - 63n landed HF1 (listing sizes) and HF4 (`\Description` digested, labels on the float);
  - **refresh 2026-10-19** (user: continue this work under the quality focus): 51 new reports, 33 papers re-converted on int31a — conference-template frontmatter persists (HF14: aero.cls, aistats2027, iclr2027; title from the first heading, merged or missing creators);
  - **refresh 2026-10-21** (HF15): 7 reports (#7162-#7168), 5 papers — 2 fixed on head, 2 fixed with a residual, classes C1 (webofc side caption, FIXED 78e) and C2 (a subfigure panel keeps the page's `\linewidth`, 78d)
- Size: medium each
- Detail: [`perfect_kernel/HTML_FEEDBACK.md`](perfect_kernel/HTML_FEEDBACK.md)

### A4 — Full arXiv fidelity audit

- **Full arXiv fidelity audit** (user 2026-10-10): a high-performance Rust analysis script over the whole arXiv conversion output for silent content/markup losses (empty frontmatter elements, words lost against the source, ERROR nodes, unparsed-math ratio, misplaced elements), ranked into fix classes — A2's method at corpus scale
- State: **QUEUED, LAST**: after the run-336 Error/Fatal subgoal and the queued payloads reach diminishing returns
- Size: large
- Detail: —

### G1 — Run-329 open residuals

- **Run-329 open residuals** — 17 PushbackLimit Fatals (csvsimple + siunitx `S`), TooManyErrors / never_completed, the `gullet.rs` `handle_template` latent panic, the runtime-bindings `.rhai` fallback, amsppt-as-plain, `.aux` re-read, eplain, calc's `!`, singles
- State: **NEXT** (arXiv success-rate task)
- Size: medium each
- Detail: [archive](archive/SYNC_SESSIONS_2026-10.md) §"Run-329 and sandbox open residuals (61r-62f, 2026-10-04)"

### D1 — Rulings

- **Rulings**: RULED 2026-10-04 — implement tex.web §392 `\par` in a non-`\long` argument (1001.1670) and restore Perl's `Error:malformed` for DIVERGENCES #189 (prerex section-in-figure): 62g landed (#189: 7 in-scope manuals now report it — biblatex, keytheorems-doc, pdfmarginpar, phonrule-doc, prerex, srdp-mathematik, zx-calculus — so G1 counts 7), 62h landed (§392, DIVERGENCES #442 / KPE #472-473).
- Still pending: `\obeylines` `^^M`;
- `{framed}` item shape;
- biblatex `cite.<n>@<key>` surpass anchors (D15);
- R7 port-or-drop
- State: RULING
- Size: small-medium
- Detail: rows cited

### T1 — Repro suite move

- **Repro suite move**: `tools/perfect_kernel/repros` (1,469 files) → `latexml_oxide/tests/repros` header-contract trials (release CI + `LATEXML_FULL_TESTS=1`);
- drain `tools/`
- State: **OPEN**, not started (user 2026-10-02)
- Size: large
- Detail: memory `project_repro_suite_move`

### T2 — Test API stage 2

- **Test API stage 2**: 37 test files still spawn binaries (`Command::new`; cluster_cli 33, cluster_xslt_split 13 …) → the in-process convert API;
- five `[[bin]]`s remain
- State: **OPEN** (stage 1 landed 61a)
- Size: medium
- Detail: `~/data/pk_agents/main/agent_reports/` (61a)

### CI — After the merge into `main`, not before (user 2026-11-06).

- **After the merge into `main`, not before (user 2026-11-06).** **CI under 30 minutes** (user 2026-10-22): the suite is 7,000 tests (≈3.8× v0.7.6's ~1,835, whose last full `main` CI took 22-23 min on the 4-vCPU runner, 2026-08-23) and `perfect_kernel` pushes do not run CI.
- **No CI-workflow edits or PRs until the user's November measurements; localhost measurements only.** Localhost baseline (int64a gates, 32 cores, test profile): 118 s wall, 53.5 core-min;
- p50 0.28 s, p90 0.90 s, p99 2.58 s;
- 17 tests over 10 s = 498 s (15 %: the two `114_streaming_cluster_regressions` shards 72/70 s, `dump_gate_init` 50/26/24 s, `batch68d` census 50/33 s);
- `cluster_package_guards` 4,474 tests / 2,613 s (81 %).
- A release trivial-article conversion takes 0.11 s / 140 MB;
- one guard test run alone in the test profile costs a flat 0.17 s / 107 MB whichever guard it is (three measured), so ≈7,000 × 0.17 s ≈ 1,190 of the 3,208 core-s (≈37 %) is per-process fixed cost (nextest runs each test as its own process).
- Options to measure, cheapest first: (1) shard the full suite over parallel runners (`cargo nextest archive` once, `--partition hash:k/N`);
- (2) a PR tier (fast unit + changed-area guards) with the full suite on `main`/release;
- (3) the per-conversion fixed cost (process start, dump load, post) and the >10 s tail;
- (4) the `ci` profile's opt levels for the workspace crates (the 2026-08 measurement: test CPU 88 → 23.8 min for +46 s build)
- State: **OPEN** — localhost study
- Size: medium
- Detail: memory `project_ci_time_budget_2026_10_22`

### TLY — A target TeX Live year per conversion

- **A target TeX Live year per conversion** (user 2026-10-25, long-term): what a paper finds depends on the TeX Live of its day, and today every answer is the ambient year's.
- Three facets, one missing input (the paper's year → its TL year, bounded by the dumps shipped; never newer than the ambient dump, `project_no_forward_time_machine_dump_oracle`): (1) **kernel** — a command newer than the paper refuses the paper's own definition (`\Ref`, `\clap`: 1307.5170, 1012.2852, 1102.2873, 1308.3100);
- ruled 2026-10-25: legacy-TL compatibility mode from that year's dump, no `\newcommand` exception;
- (2) **file presence** — `latexml_core/src/binding/author_files.rs` is year-blind: `FORMER_TEX_LIVE` answers `\IfFileExists` as present in every year (its doc comments hold the years, from the TL 2007-2024 file lists), a binding for a file TeX Live added later counts as present for a paper older than the file, and `AUTHOR_FILE_STAND_INS` holds only never-in-TL files;
- each binding needs its file's TL year range, read against the target year (`binding_counts_as_file`);
- (3) **binding behaviour** — a class or package whose versions differ by year (the family rule: one binding, union of features, dispatch only on conflict).
- Dump selection today: `dump_paths` prefers the ambient year, else the most recent (`docs/parity/DUMP_DESIGN.md`)
- State: **OPEN**, design (needs the year source: submission date vs a CLI flag)
- Size: family
- Detail: this row

### AT — AutoTeX emulation

- **AutoTeX emulation** (user 2026-10-25, queued near the end, a task of its own): arXiv-specific damage from emulating AutoTeX — (1) the wrong main `.tex` file converted, the whole document wrong;
- (2) an article with a supplement ("multiple top-level `.tex` files"), which classic arXiv turns into one PDF each and concatenates in the order AutoTeX's file-type guessing gives.
- The official arXiv PDFs guide the expected behaviour;
- the arXiv/html_feedback issues labelled `autotex` are the test harness (data `~/data/html_feedback/`).
- Related: a supplement is a second document whose frontmatter does not replace the main one's
- State: **QUEUED** (near the end)
- Size: family
- Detail: this row

### F1 — Per-encoding text commands

- **Per-encoding text commands**: textcomp's symbols now dispatch on the encoding (62r, KPE #489; repro `fonts-nfss/textmu_follows_the_encoding` GREEN).
- Residual: the kernel letters keep Perl's definedness gate (a T1 fontmap slot shared by `\DH`/`\DJ`), and `\<E>-cmd` chains ignore `\cf@encoding`
- State: **PARTIAL**
- Size: medium
- Detail: §Font-Selection Chain Residuals

### F2 — Small RED repros

- Small RED repros: ~~hyperref pdfinfo values expanded as by `\pdfstringdef`~~ (FIXED 78i, OD #824);
- `\vadjust` queue scoped to its list (60k, TeX nest; low return);
- the rest under §"Perfect-kernel RED repros"
- State: **OPEN**, safe now
- Size: small each
- Detail: §Perfect-kernel RED repros

### R3 — Bibliography-absence campaign

- **Bibliography-absence campaign** (PR #444) — **291 recovered / 20 338 entries**.
- Remaining unblocked: **R3d tab-mark parameter scan vs cell read**
- State: **R3d next** (12 papers left, unblocks alignment macro `&` splits)
- Size: medium
- Detail: Open items §R3, [`RESIDUAL.md`](parity/bib_absence_2026-07-29/RESIDUAL.md)

### R8b — forest.sty full support

- **forest.sty full support** (side goal, user 2026-09-05; heavily used on arXiv) — native semantic tree since 56iz (nested inline lists, labels as TeX, node keys; DIVERGENCES #302);
- open: drawing/layout fidelity
- State: **OPEN**, recorded
- Size: large
- Detail: [`perfect_kernel/DIFFICULT_CASES.md` §D10](perfect_kernel/DIFFICULT_CASES.md)

### R4 — `--preload=<cls>` trips the LaTeX hook stack (`Extra \PopDefaultHookLabel`)

- Closed (**RESOLVED** (re-verified on int101a 2026-10-25: `--preload=` article, report, book, amsart, revtex4-2 all 0 …); the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Row R4".

### R6 — `Collector::rescan` Refactor for Generated Backmatter

- **`Collector::rescan` Refactor for Generated Backmatter**
- State: **OPEN**; `Scan` owns `ObjectDB` by value; generated backmatter lacks full relations/labels
- Size: medium
- Detail: Open items §R6

### R7 — Presentation-MathML **F5** Linebreaker

- Presentation-MathML **F5** Linebreaker
- State: **OPEN**, full linebreaker feature gap needing a port-or-drop scope decision
- Size: family
- Detail: Open items §R7

---

## Silent-loss inventory — blocks the 98 % claim (user 2026-10-18)

"We should address all of [the High Priority items that were not Error status] before claiming 98 % is reached": a
conversion that loses authored content or its markup with 0 errors is a false success. Source:
`agent_reports/2026-10-18_silent_loss_inventory.md` (27 items, numbered as there); each row is fixed, or at least made
an honest Error, before the 98 % report or the E/F rerun. Measured on all three targets (error-free log, authored text
present, semantic element) — `feedback_three_metrics_honest_tests`.

**Status 2026-10-19 (int31, 9482a8e7db):** every numbered row is fixed (#21 in part: what remains prints extra
entries, it loses none), closed, or an honest Error; the unnumbered
rows left open below lose no authored text (duplication, mis-split, hover-only placement, a wrong reference number) and
do not block the E/F rerun.

**Moved 2026-11-08 (DOC1):** the landed frontmatter rows of this table are in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md), verbatim; a landed row with a live residual left a one-line row here holding only the residual, marked "the residual".

### 5 — raw-class off-table title-page stores (#5) — the residual

- **raw-class off-table title-page stores (#5) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): the vmsta 2018+ record ruling (75a row (e));
- nowfnt's cover author block beside the creators (OD #767, row "nowfnt creators").
- State: OPEN (residual)

### 18 — amsrefs `\bib` keys differing in case merge, one entry lost

- amsrefs `\bib` keys differing in case merge, one entry lost
- State: honest Error int27 (`scan_bibentries`); keeping both open

### ptptex `[preprint]` double abstract

- ptptex `[preprint]` double abstract: the class's own `[preprint]` `\maketitle` runs (ptp-prep.clo:111-119), prints `\@abst` in its block and clears it (:27-34);
- State: open residual (not new in 77i; pinned by count in `perfect_kernel_batch77i::abstract_store_cleared_by_the_class_title_page_is_kept`, repro `sectioning-frontmatter/abstract_store_cleared_by_the_class_title_page_is_kept.tex`): the store is handed on, but the block the class's running `\maketitle` typesets is not the deposit's, so its label and store are not taken out with it
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "ptptex `[preprint]` double abstract".

### #6a labels

- #6a labels: a label read off the class's code only when it is plain text — one with an accent (`R\'esum\'e : \@resumen`, `\textbf{Mots-cl\'es :} \@keywords`: `label_item` refuses the accent macro) stays an orphan paragraph and the entry takes the default name ("Abstract", "Keywords: ");
- State: see the archive
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "#6a labels".

### nowfnt creators

- nowfnt creators: `\author[n]{Haikin, Marina}` (nowfnt.cls:581, BibTeX name order "Surname, First", split by the class's `\nowfnt@splitauthor`) becomes two creators "Haikin" and "Marina" (8 creators for 4 authors), and `\affil[n]{…}` (:599) reaches no creator — 2108.12618, 2308.02698 (base too);
- the cover's `\maintitleauthorlist` block is dropped when it only repeats the creators' names (2308.02698) and kept as an `ltx:pubnote` with line breaks between the authors when it carries more (2108.12618's affiliations and e-mails; main-session decision, OD #767, 77i r3)
- State: open

### bullsrsl correspondence

- bullsrsl correspondence: the corresponding-author e-mail (`\correspondance`, bullsrsl.cls:475-477, printed after `\BSRSL@CorrespIntro`, a label macro the class defines only inside its `\maketitle`, :747-765 "Correspondence to:") reaches the frontmatter as an `ltx:pubnote` without that label, which renders as a hover-only popup (LaTeXML.css:107-120) — 2401.09121, 2410.19120 (77i r2/r3; base lost it).
- Needs: a label read from a macro the title page defines before printing the store, and routing to the corresponding creator's `ltx:contact role=email` (the creator carrying `[corresponding]`): not a brief patch with the existing store techniques (77i r3)
- State: open

### 21 — biblatex `\printbibliography` filters

- biblatex `\printbibliography` filters: every list printed every entry
- State: see the archive
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "biblatex `\printbibliography` filters".

### an IEEE affiliation block whose refmarked entries a tie run parts is one entry

- an IEEE affiliation block whose refmarked entries a tie run parts is one entry: `\IEEEauthorblockA{\IEEEauthorrefmark{1}Silicon Austria Labs … \\\IEEEauthorrefmark{2}Johannes Kepler University, Linz, Austria~~~\IEEEauthorrefmark{3}Infineon Technologies …}` gives "Johannes Kepler University, Linz, Austria 3Infineon Technologies" one affiliation and the mark-3 authors none (2603.03918; int34a the same), 0 Errors
- State:
  - open (77p r3, review item 7): reach 1 of 84,990 papers of 2603-2605 (3 blocks part refmarked entries by a run; the `\;\;` ones, 2603.12023 and 2604.07032, already split);
  - the run is inside the `\IEEEauthorblockA` argument, which `normalize_hspace_separators` (the top level of `\author`) does not read — the IEEE affiliation reader would take the mark-led rule

### the aero.cls variant whose `\maketitle` prints `\@title`/`\@author` (2509.08607, 2511.17720) keeps its `\cor` glyph in the personname, "Dario Izzo(🖂)", and the paper's …

- the aero.cls variant whose `\maketitle` prints `\@title`/`\@author` (2509.08607, 2511.17720) keeps its `\cor` glyph in the personname, "Dario Izzo(🖂)", and the paper's e-mail and address stay body paragraphs (`\noindent\cor{} dario.izzo@esa.int`), as int34a
- State: open (77p r3, OD #775): the kernel `\author` path is left as it is (review item 1); reading the variant's `\cor` as a request needs the e-mail the paper sets in its body

### `Error:frontmatter:merged_creators` separators (77d, 77n; OD #765) — the residual

- **`Error:frontmatter:merged_creators` separators (77d, 77n; OD #765) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): open S6 shapes (class list `\author`s: aipproc 0911.5408, 1411.0869, physics0611155; acmart 2105.00164; quantumarticle 2404.17677; IEEE `\IEEEauthorblockN` with `\IEEEauthorrefmark` 1509.06333; fairmeta 2509.15733), 1706.05664 (pasj01 `\author`), 1911.02873's first two names (a mark and jfm `\ns`, flagged `merged_creators` again).
- State: OPEN (residual)

### `\@@underline` is defined under the dump (record `M \@@underline PA \underline`, latex.ltx:16368's `\let`) but undefined under `LATEXML_NODUMP=1`

- `\@@underline` is defined under the dump (record `M \@@underline PA \underline`, latex.ltx:16368's `\let`) but undefined under `LATEXML_NODUMP=1`: the two `LoadFormat` branches disagree on that name (77g review finding 8)
- State: see the archive
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "`\@@underline` is defined under the dump (record `M \@@under".

### merges now flagged by 77d's mark-as-comma re-read (`flag_merged_creators`; XML unchanged, the content was merged silently before)

- merges now flagged by 77d's mark-as-comma re-read (`flag_merged_creators`; XML unchanged, the content was merged silently before): binding nets 67int34 vs 67int29 13 documents 0 → 1-2 `Error:frontmatter:merged_creators` — names merged with each other (2508.17434 aaai2026/listings, 1804.05141 cryptocode, 1706.10121 and 2207.11582 gretsi, 2112.09858 jfm, 2602.02920 listings, 1912.09565 sagej, population 1906.03608 `\And`-less list), a name merged with its affiliation (1102.4258, 2608.09604 egpubl: "V. Zobel INRIA Grenoble Rhône-Alpes") or with a note ("José C. Príncipe Equal Contribution", 2410.14697 neurips_2024)
- State: open — split by shape (egpubl affiliation lines, neurips equal-contribution notes, gretsi/aaai separators)

### author names parted by `\ ` alone with a `\\` line of more names (`\author{Ann Able \ Bob Baker \\ Cat Cole \ Dan Dorn}`, the article `\author` and every per-author …

- author names parted by `\ ` alone with a `\\` line of more names (`\author{Ann Able \ Bob Baker \\ Cat Cole \ Dan Dorn}`, the article `\author` and every per-author split): "Ann Able Bob Baker" is one creator and the second line its affiliation, two authors lost with 0 Errors;
- ecai's 1606.03244 has the shape (ecai keeps such a block whole since 77n r2, one creator holding the five names)
- State: open (77n r1 review finding 1): a `\\` line that reads as names after a line of `\ `-parted names continues them (`names_continue`), and `\ ` between two name-shaped runs separates them

### an author's mark is shown after its ORCID link

- an author's mark is shown after its ORCID link: `\author{A$^{1}$\orcidlink{…} and B$^2$\orcidlink{…}}` (2405.16615, authblk; the article `\author` the same) gives "A[iD]1" where pdflatex prints "A1[iD]"
- State:
  - open (77n r1 review finding 2; RED repro sectioning-frontmatter/author_mark_before_orcid_link): a split name's marks are read as affiliation requests (`\lx@sup@request@affiliation`) and an unanswered one is appended to the personname (`show_unanswered_marks`);
  - keeping its place needs a slot where the request was read (after the math it sits in: `\aftergroup`), filled or removed at insertion, without spending an `xml:id`

### a split name's mark that a line or other text follows

- a split name's mark that a line or other text follows: the same placement on every name the author-line splitter reads — `Ann Able$^{a}$ (ann@x.org)` gives "Ann Able (ann@x.org)ᵃ", and one before a `\\` is dropped (`\author{Ann Able$^{1}$\\ ann@x.org \and Bob Baker$^{2}$}`: "Ann Able", int32a too)
- State: see the archive
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "a split name's mark that a line or other text follows".

### a combined legend mark is labelled by its first label only

- a combined legend mark is labelled by its first label only: `\author{Ann Able$^{1}$and Bob Baker$^{1,2}$\thanks{$^{1}$…}\thanks{$^{1,2}$Bob Baker is with Univ Y.}}` (2604.27101, ieeeconf) gives Ann Able Bob Baker's legend too (`\lx@sup@setlabel@affiliation` keeps the first of `clean_frontmatter_labels`; int32a the same)
- State: open (77n r4; authblk's `\affil{$^{1,2}$…}` leaves the marked path for authblk's run instead, `queued_authors_carry_entry_mark`)

### names parted by a separator inside a layout or declaring group are one creator

- names parted by a separator inside a layout or declaring group are one creator: `{\large \bf Yuxuan Yang$^{1,2}$ \quad Dugang Liu$^{1*}$ \quad …}` (2603.20775, flagged `merged_creators`), `{\centering\bfseries A\textsuperscript{…}\quad B…\\ C…\par}{\centering\small $^1$…\par}` (2610.21682: one creator holding every name and affiliation, 0 Errors — silent), `\textbf{Alain Rakotomamonjy\up{2}\hfill Patrick Gallinari\up{1,2}}` (2209.14855, silent), `{David McAllister$^{1}$… ~ Songwei Ge$^{2*}$ ~ …}` (2406.09417: one tie between spaces, no run, flagged)
- State: see the archive
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "names parted by a separator inside a layout or declaring gro".

### a row of addresses parted by space under a row of names parted by space (`\author{Cat Cole \quad Dan Dorn \\ Univ C \quad Univ D}`, each address printed under the name …

- a row of addresses parted by space under a row of names parted by space (`\author{Cat Cole \quad Dan Dorn \\ Univ C \quad Univ D}`, each address printed under the name above it) gives Univ C to the row's last name and makes "Univ D" a creator, 0 Errors (Perl the same)
- State: see the archive
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "a row of addresses parted by space under a row of names part".

### names parted by horizontal space under symbol-labelled address lines (77p-77y; OD #52(o)) — the residual

- **names parted by horizontal space under symbol-labelled address lines (77p-77y; OD #52(o)) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): a legend's "Corresponding author:" before a bare address is no e-mail label (2609.24299);
- Also open (recorded 77p r4-77y, not fixed since): a starred legend in a numbered block goes to the last author (2608.22149's `{\small $^{*}$Equal contribution.}` on Yuchen Cui, 2605.03042's `$^\dagger$Project Leader \quad $^*$Corresponding Author` on Shuai Li);
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "names parted by horizontal space under symbol-labelled addre".

### a linked two-word text whose last word is a link word, alone in its `\and` column, reads as a name

- a linked two-word text whose last word is a link word, alone in its `\and` column, reads as a name: `\author{Ann Able \\ Univ A \and Bob Baker \\ Univ B \and \href{…}{Source Code}}` makes a creator "Source Code" (no corpus witness, probes a5, a6 — "Online Demo" too; and a three-word one whose last word is a link word that is also a surname, "Full Source Code", "Google Scholar Page", "Hugging Face Link", the link column first or last: probes s1, s3, s4, s7 of the rev77y3 re-review's finding 5 — or its plural, "Other Useful Links", the rev77y4 re-review's probe s20 — no author block linking such a text among 519,672 papers of 2025-2026, int38b the same, 77y r2 URL contacts; 2606.06924 prints "Datasets  Source Code" on one line under its e-mail line, both URL contacts of the shared creator as its PDF shows, the rev77y2 re-review's finding 6; int38b the same), 0 Errors
- State: open (77y r2, the 77y review's finding 1): `link_words_beyond_a_surname` keeps a two-word text a name for "Lucy Page" alone in its column (`perfect_kernel_batch77p`'s linked-names guard) and cannot tell it from "Source Code" without a word list; options: a column holding a linked text alone is the block's link unless the text spells the link's target (`x.org/lp` for Lucy Page), or accept.
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "a linked two-word text whose last word is a link word, alone".

### a name alone under a row's merged symbol lines over a one-word generic institution noun or a connective-led institution of its own is lost

- a name alone under a row's merged symbol lines over a one-word generic institution noun or a connective-led institution of its own is lost: `Ann Able$^{\dagger}$ \quad Bob Baker$^{\ddagger}$ \\ $^\dagger$Univ A \quad $^\ddagger$Univ B \\ Cat Cole \\ Physics` gives "Cat Cole" to the shared creator as an affiliation and "Physics" to Ann Able and Bob Baker (pdflatex prints it under Cat Cole only);
- State: open (77y, accepted cost of `continues_the_line`, which reads `Shanghai Jiao Tong \\ University` as one institution, probe k12; the hyphenated `Max-Planck-Institut` FIXED in int41, guard `perfect_kernel_batch77y::names_split_under_symbol_lines_reads_the_line_under`).
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "a name alone under a row's merged symbol lines over a one-wo".

### in a per-author `\author` that adds to its block, a link column (`\href{…}{Project Page} \\ Univ B \\ contact@x.org`) goes to the current author

- in a per-author `\author` that adds to its block, a link column (`\href{…}{Project Page} \\ Univ B \\ contact@x.org`) goes to the current author: a raw class appending to `\author` puts the URL, Univ B and the e-mail on Ann Able, where pdflatex prints the column beside her (the rev77y4 re-review's finding 3, probes x7, x8; int38b and 77y r2 a creator "Univ B"), no corpus witness, 0 Errors
- State: open (77y r4): `LinkOwner::Current`/`\lx@add@email` for `block_links`/`block_lines` when `replace` is false; option: the block's (`LinkOwner::Block`, `AddressPlacement::Shared`) once a per-author block forms a shared creator.

### an organization link whose name is exactly an author's spelling reads as that author's handle

- an organization link whose name is exactly an author's spelling reads as that author's handle: `Hua Wei \\ \href{https://www.huawei.com}{Huawei}` and `Hua Wu \\ {HW}` give a URL contact on the name, not an affiliation (the rev77y4 re-review's finding 4, probes h20, h21; int38b and 77y r2 affiliations), the link still on the name it prints under, 0 Errors
- State: open (77y r4, `link_prints_a_handle`'s exact-spelling test; no corpus witness).

### a name alone under a row's merged symbol lines with a place under it is lost

- a name alone under a row's merged symbol lines with a place under it is lost: `Ann Able$^{\dagger}$ \quad Bob Baker$^{\ddagger}$ \\ $^\dagger$Univ A \quad $^\ddagger$Univ B \\ Cat Cole \\ Palo Alto, CA` gives "Cat Cole" and "Palo Alto, CA" to the shared creator (the rev77y2 re-review's probe k10; int38b and 77y r2 the same; 77y r1 an author), 0 Errors
- State: see the archive
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "a name alone under a row's merged symbol lines with a place ".

### a place line with no comma under a welded symbol row's labelled institution goes to the shared creator

- a place line with no comma under a welded symbol row's labelled institution goes to the shared creator: `$^\dagger$Department of Physics \\ University A \\ Palo Alto \\ $^\ddagger$School of Law …` gives "Palo Alto" to the block, not to the † name (the rev77y2 re-review's probe w9; 77y r1 Ann Able's, int38b the block's), 0 Errors
- State: accepted (77y r3, the rev77y2 re-review's finding 7): `reads_as_place` needs a comma (`Muncie, IN 47306`), so a bare place is no `weldable_line`, and an unreadable owner goes to the block (the accepted fallback).

### a brace-list address group and a later address under three names swap two owners

- a brace-list address group and a later address under three names swap two owners: 2606.06924's `\{laign,yehj\}@…, huhr@…` gives `yehj@` (Han-Jia Ye's) to Haoran Hu and `huhr@` (Haoran Hu's) to Han-Jia Ye (int38b and 77y r1-r3 the same), 0 Errors
- State: open (pre-existing; the rev77y2 re-review's finding 8): `spelled_owners` scores "yehj" and "huhr" against the three names' keys — not yet root-caused; the swap puts each address on a name the PDF does not print it under.

### in an unmarked block a row of names after an affiliation line stays an affiliation line of the names above

- in an unmarked block a row of names after an affiliation line stays an affiliation line of the names above: `Ann Able \\ Univ A \\ Bob Baker~~~~Cat Cole \\ Univ B` gives Ann Able the affiliations "Univ A", "Bob Baker Cat Cole", "Univ B" (two authors lost; 0 Errors; int34a and Perl the same, the r3 re-review's probes p19, p27, where r3 had made Cat Cole alone a creator)
- State: see the archive
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "in an unmarked block a row of names after an affiliation lin".

### an author mark with a footnote symbol before the number, `$^{*1}$`, requests no affiliation

- an author mark with a footnote symbol before the number, `$^{*1}$`, requests no affiliation: `Ann Able$^{*1}$, Bob Baker$^{1}$ \\ $^1$Univ A` gives Ann Able none (the 77p r4 re-review's probe k1);
- State: see the archive
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "an author mark with a footnote symbol before the number, `$^".

### a horizontal skip that opens a line of an unmarked block parts it as a separator

- a horizontal skip that opens a line of an unmarked block parts it as a separator: `The University of Hong Kong\\ \hspace{-12pt}\texttt{\small \{bhzhang, liujh, xjqi, hso\}@eee.hku.hk}` makes the e-mail line a names line, its pieces creators "{bhzhang", "liujh", "xjqi", "hso}@eee.hku.hk" (2603.27558; int34a the same), 0 Errors
- State:
  - open (77p r5 A/B, recorded): OD #52(i)'s `\hspace` → `\quad` rewrite reads a skip at a line's start as a group separator;
  - a negative or line-opening skip separates nothing, but a positive one can stand between author groups (`Ann Able \\ Univ A \\ \hspace{2cm} Bob Baker \\ Univ B`), so the rule needs care

### a mark wrapped in a user macro is not read as a mark

- a mark wrapped in a user macro is not read as a mark: `\newcommand*{\affmark}[1][*]{\textsuperscript{#1}}` then `Thanh-Tung Nguyen\affmark[1]~~ Viktor Schlegel\affmark[1,3]~~ …\\ \affaddr{\affmark[1]AICS Singapore / TaiPei}` (2304.13998): the block reads as unmarked, "3]" and "2]" junk creators (the comma in the optional argument), the affiliations not by the marks, 0 Errors;
- 2602.23952's `\affmark[1,2,3]` the same, invented creators "2", "3]", "2]" (int34a too; 77p r5 recovers about six of its authors; the r5 re-review)
- State: open (77p r1 question 3; main-session decision 2026-10-19: a later follow-up): a macro whose expansion is exactly `\textsuperscript{#1}` (its optional default the mark too) reads as a mark, as a `\let` alias does (`mark_command_meant`, OD #52(o))

### \footnotemark[1] after an author name in an ICLR block prints "1 footnote 1 footnote Footnote footnotes Footnotes 1 footnote 1" inside the personname (\textbf{Sungil …

- `\footnotemark[1]` after an author name in an ICLR block prints "1 footnote 1 footnote Footnote footnotes Footnotes 1 footnote 1" inside the personname (`\textbf{Sungil Son}\textsuperscript{1}\footnotemark[1]`, 2610.20409; `\textbf{Ying-Tu Chen\textsuperscript{\normalfont{1},}\footnotemark[2]}`, 2604.24532; 2603.12193, 2509.10377, 2403.00801; int33a the same, the names merged then)
- State: open (77p A/B)

### an e-mail that spells no surname (77p r3-r11, 77y; OD #52(o), user ruling R10) — the residual

- **an e-mail that spells no surname (77p r3-r11, 77y; OD #52(o), user ruling R10) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): an icon-only link (2604.10930);
- an address printed twice is two contacts (2604.10667, a ruling if one should go).
- Also: the trade's second witness 2609.02349's `xiahou@` (on Junping Zhang; accepted residuals, user ruling 2026-10-20);
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "an e-mail that spells no surname (77p r3-r11, 77y; OD #52(o)".

### an unmarked author block's brace-list emails (`\{nlp2ct.guanhua, nlp2ct.chuyue, …, xqsongangie\}@gmail.com\\ \{derekfw, lidiasc\}@um.edu.mo`, 2605.15019) go to the last …

- an unmarked author block's brace-list emails (`\{nlp2ct.guanhua, nlp2ct.chuyue, …, xqsongangie\}@gmail.com\\ \{derekfw, lidiasc\}@um.edu.mo`, 2605.15019) go to the last author, but for an address that spells a name
- State: open, an accepted trade-off (77n r5 brief; r3: `NLP\textsuperscript{2}CT Lab` is the word's since `rewrite_in_word_superscripts`, so the block is no longer a marked one — its four junk creators gone and the affiliation on every author, the marked branch's per-position emails lost)

### authblk `\affil{1}` inside `\author` (`\author{Khalid Mahmud Labib\affil{1}, Saad Waheed\affil{1}, …}` then `\affil{Dynamical Systems and Signals Lab (DSSL)\\ …}`, …

- authblk `\affil{1}` inside `\author` (`\author{Khalid Mahmud Labib\affil{1}, Saad Waheed\affil{1}, …}` then `\affil{Dynamical Systems and Signals Lab (DSSL)\\ …}`, 2605.01689): pdflatex prints no "1" and the DSSL block under every author;
- we give each author a "1" contact and the DSSL block to the last author only (int32a's and Perl's `annotate=new`, authblk.sty.ltxml:57)
- State: open (77n r4; guard `perfect_kernel_batch77n::authblk_affil_after_affils_in_author_reaches_them` pins it): read an `\affil` inside `\author` that prints nothing as no affiliation, so the run reaches every author — a further departure from Perl's `annotate=new`, a ruling first

### a capitalised "And"/"AND" between two names is no separator

- a capitalised "And"/"AND" between two names is no separator: `\author{Ann Able And Bob Baker}` is one creator with 0 Errors — the merge check reads "And" as a name word (int32a and Perl the same; Perl's only word separator is " and ", Base_Utility.pool.ltxml:678);
- glued after a mark, `Ann Able$^{1}$And Bob Baker$^{2}$` (77n r4/r5 review probes a1, a6), it is one creator flagged `Error:frontmatter:merged_creators`
- State: open (77n r6 keeps the flag, fail toward flagging — r5 read the "And" as a separator after the mark and merged the line silently; guard `perfect_kernel_batch77n::mark_glued_to_capital_and_is_flagged`)

### a run always reaches its last author (Perl's `new`, Base_Utility.pool.ltxml:530), answered or not — the label-answered break applies only once the run has reached an …

- a run always reaches its last author (Perl's `new`, Base_Utility.pool.ltxml:530), answered or not — the label-answered break applies only once the run has reached an unanswered author (77n r6, so a run answered throughout is the whole run's, as ptephy prints it): `\author{Craig M$^{\mathrm{c}}$Rae}\author{Ann Able$^{1}$}\affil{$^1$Univ X}\affil{Univ Y}` (2505.09041's shape) gives the answered last author Ann Able Univ Y as well as Univ X
- State: open (77n r5; guard `perfect_kernel_batch77n::authblk_superscript_inside_a_name_stays` pins it). The first half of this row closed in 77n r6: an authblk name kept on `\lx@add@creator` (`Ann Able$^{1}$\textsuperscript{equal contrib.}`) requests the affiliations its marks pick (`author_label_marks`), so the run passes it over (guard `authblk_textual_superscript_after_mark_is_shown`)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "a run always reaches its last author (Perl's `new`, Base_Uti".

### authblk's note numbering (`\AB@thenote`, authblk.sty:114-167) is not modelled

- authblk's note numbering (`\AB@thenote`, authblk.sty:114-167) is not modelled: an unlabelled `\author` after a labelled one shares its note (`\author[1]{Ann}\author{Bob}\affil[1]{Univ X}\affil{Univ Y}`: pdflatex "Ann1 and Bob1 / 1Univ X / 1Univ Y", Univ X under Bob too; we give Bob Univ Y only), and an `\affil` inside a later `\author` steps it (`\author[1]{Ann}\affil[1]{Univ X}\author{Bob \affil{Univ Y}}`: pdflatex "Ann1 Bob2 / 1Univ X / 2Univ Y"; we give Ann Univ Y as well, 77n r4 review probe e3);
- State: open (77n r6; int32a and Perl's `annotate=new`, authblk.sty.ltxml:57, the same; guard `perfect_kernel_batch77n::authblk_labelled_author_shares_the_run` pins it): number the notes as authblk does — an `\author` steps `\AB@thenote` after an `\affil`, an unlabelled one takes it — and label each unlabelled `\author`/`\affil` by it
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "authblk's note numbering (`\AB@thenote`, authblk.sty:114-167".

### a symbol legend after a run reaches authors who do not show its symbol

- a symbol legend after a run reaches authors who do not show its symbol: `\author{Ann$^{1}$}\author{Bob}\author{Cat$^{\dagger}$}\affil{$^1$X}\affil{Y}\affil{W}\affil{$^\dagger$Corresponding author}` — pdflatex shows † on Cat only, int32a and Perl's `annotate=new` give the legend to Cat only, 77n r6 to Bob and Cat, r7 to Ann, Bob and Cat (77n r7 review probe x7; s1, z1, g05 since r6, the ∗ legend on Ann; guard `perfect_kernel_batch77n::authblk_symbol_mark_then_text_shown_once` pins it).
- No paper in 823 A/B conversions or the 2603-2605 scans
- State: open (77n r7; reach 0): when an author of the run shows the legend's symbol, the legend is those authors' only (`base_utilities/defs/frontmatter_api.rs` legend read ~:596, extent ~:719)

### authblk

- authblk: an affiliation a mark picks (`\affil{$^1$X}`) is listed on its authors after the run's unlabelled ones (`\author{Ann$^1$ and Bob$^2$}\affil{$^1$X}\affil{$^3$Z}` gives Ann Z, X): the marked contact is placed at the end, when the labels are matched (`relocate_annotations`), the run's at once
- State: open (77n r3; guard `perfect_kernel_batch77n::authblk_typo_mark_affil_runs` pins the order): place a label-matched contact at its queue position among its author's contacts

### author names parted by a mark and spacing alone, with no separator (pre-77d, synthetic)

- author names parted by a mark and spacing alone, with no separator (pre-77d, synthetic): `Ann Able$^{1,}$ Bob Baker$^{2}$` is one creator "Ann Able Bob Baker", both marks after it, with 0 Errors (1911.02873's shape, there flagged as its marks stay between the names);
- `Ann Able\hspace{1em}and\hspace{1em}Bob Baker` makes "and" an affiliation of Ann (the `\hspace` read as an author/affiliation line break before the name split);
- a pair merged with only glued text to show it ("Ann Ableand Bob Baker", a shape no rule splits) is not flagged — "Ableand Bob" reads as "Ferdinand Smith"
- State: open

### HF14 conference-template frontmatter (77o; OD #772) — the residual

- **HF14 conference-template frontmatter (77o; OD #772) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): open (HTML_FEEDBACK HF14): M2 spacing-run separators / `\let`-alias marks, M4 authblk replacing authors, aero.cls, 2609.12818.
- State: OPEN (residual)

### aistats paired column

- aistats paired column: an address line starting "and" is read as an author separator by the kernel `\author` line reading, so 2402.12508's `Department of Mathematics \\ and Computer Science \\ University of Basel` makes a creator "Computer Science" (both rows since 77o r3; up to 32 paired papers carry such a line; 2310.02702 reads right);
- State: open (77o r2 review finding 6, r3 finding 1). Fix when taken: the binding knows line 1 of each column is the name — add the name, then each address line as an affiliation or e-mail explicitly, instead of the kernel's guess
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "aistats paired column".

### aistats in-place reading and e-mail splitting, small (77o r3 review nits 2-4; corpus reach 0 for each)

- aistats in-place reading and e-mail splitting, small (77o r3 review nits 2-4; corpus reach 0 for each): a column holding a test primitive (`\ifx\Uni\relax …`) is not read in place, so a macro defined in the block is undefined (probe c_ifx; r2 printed it by luck);
- a mark glued to an address in a block line (`ann@a.org$^*$, bob@b.org`) stays in the e-mail contact ("ann@a.org*"; probe e_mark);
- a `\gdef` after the block wins over the one before the row (probe c_gdef; deferred frontmatter digestion, r2 the same)
- State: open: protect only a test primitive's operands; reject an `email_addresses` item with a skipped superscript inside it; digest the row in place (FM S5)

### aistats

- aistats: a marked first row followed by an unmarked row leaks the second row's address upward (review probe `r_secondrow_marked`: Ann gets "Univ C" before "Univ A"; her marked affiliation is attached later, so `annotate=run` still counts her new)
- State: open, 0 of 2,843 aistats papers (77o r2 review finding 7)

### aistats `in_place` residuals

- aistats `in_place` residuals: a macro with parameters defined inside the `\twocolumn[…]` block (`\def\Uni#1{…}`, probe `g_args`: Error undefined, pdflatex "Univ of Here") or a local `\let` of a primitive (`\let\Uni\relax`, probe `g_letprim`: Error) is not read in place
- State: open (77o r2 review finding 4); no corpus witness

### aip-cp `\author[aff1]`/`\affil[aff1]` marks are not linked

- aip-cp `\author[aff1]`/`\affil[aff1]` marks are not linked: the authors are creators without affiliations and the `\affil` list reaches the page as the class title page's deposit paragraph (1506.04050, 1604.08112; int37a the same)
- State: open: needs a mechanism for a class's mark-keyed author/affiliation lists (`\g@addto@macro` accumulators, not K11 stores)

### natbib forks that declare their own names stay read raw (`hbnatbib` 16, `ctextemp_natbib` 13, `NAR-natbib` 9, `natbib_padeu` 8, `rnatbib` 7, …

- natbib forks that declare their own names stay read raw (`hbnatbib` 16, `ctextemp_natbib` 13, `NAR-natbib` 9, `natbib_padeu` 8, `rnatbib` 7, `wsnatbib`/`igsnatbib`/`dmnatbib` 6 each, …) and so do the 399 shipped natbib copies that declare nothing: their citations still print "?" unless the fork has a binding (NJDnatbib, bmvc2k_natbib, ws-rv-har)
- State: open (77w residual; census `~/data/pk_agents/main/scratch_356a0c47/natbib_copies/natbib_copies_all.tsv`): a route by content (rule 0 (b)'s command-set test) is the candidate, a user ruling first

### author-block gaps that renamed copies reach (int40) — the residual

- **author-block gaps that renamed copies reach (int40) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): aamas and spie FIXED;
- egpubl (2312.16264's arxiv.cls) and bytedance_seed open: binding work in each family.
- State: OPEN (residual)

### caption separator vs a copy's own label

- caption separator vs a copy's own label: the separator is the binding's model of the class's `\@makecaption` (`\format@title@<type>`'s close), so a routed copy's (or a raw class's) `\fnum@<type>` that carries its class's punctuation meets the binding's (or the kernel's ": "): "Fig. 1.—.— " (1106.2565 emulateapj-rtx4.cls:911 + the emulateapj binding's ".— ", astro-ph0204283), "Figure 1..— " (1505.06804), "Table 1:: " (astro-ph0601469, astro-ph0603093), "Figure 1.: " (astro-ph0101121), "Fig. 1.—: " (1402.7103, PDF "Fig. 1.— " from proto.cls:1226 `#1\ #2`) — 7 of the 78h A/B's 322 papers, worse than int47a's binding label
- State: RULED 2026-10-21: the separator is the binding's (default ": " as Perl), and a copy's caption format (`\fnum@<type>`) is the binding's too — only its declared text names (`\figurename`) reach the label (OD #731 rule 14); no raw `\@makecaption` is read. The 7 doubled separators of 78h r1 are gone (78h r2) — closed
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "caption separator vs a copy's own label".

### a stand-in's copy under the binding's own name (rule 0: a shipped svjour3.cls, aas_macros.sty) keeps the binding's text names (OD #731 rule 1); the 2026-10-21 ruling's …

- a stand-in's copy under the binding's own name (rule 0: a shipped svjour3.cls, aas_macros.sty) keeps the binding's text names (OD #731 rule 1);
- the 2026-10-21 ruling's yielding is the route's (rule 14), though TeX prints the copy's text there too
- State: RULED 2026-10-21: a copy under the binding's own name overrides nothing (rule 1 unchanged; guard `perfect_kernel_batch78h::self_named_copy_supplies_no_text_names`) — closed

### the aas_macros binding's journal names lag AASTeX 7's

- the aas_macros binding's journal names lag AASTeX 7's: it prints "ApJ" for `\apjl`, "J. Quant.
- Spec.
- Radiat.
- Transf." for `\jqsrt`, "Acta Astron." for `\actaa` (aas_macros_sty.rs:25/70/16), where TL aastex701.cls:8314/8352/8359 prints "ApJL", "JQSRT", "AcA" — a class-family binding gap (one binding serves every version: union of features);
- routed AASTeX 6.1+ copies print the class's names since 78h (2107.03400, 2007.00749), an AASTeX document under the binding does not
- State: open (found 78h r2 review)

### the aa binding's itemize labels are article's "•", where aa.cls:2054-2056 sets a bold "–" at the first two levels and `$\bullet$` at the third (2503.15310's aaEC.cls and …

- the aa binding's itemize labels are article's "•", where aa.cls:2054-2056 sets a bold "–" at the first two levels and `$\bullet$` at the third (2503.15310's aaEC.cls and 2505.14783's copies printed it under 78h r1) — the llncs gap 78a closed (KNOWN_PERL_ERRORS #1017)
- State: RULED 2026-10-21: typography, the binding's default stays (list labels are the binding's, OD #731 rule 14) — closed

### the svjour3 binding numbers the class's predefined theorems within sections ("Theorem 5.1") without the `envcountsect` option, where svjour3.cls:902-911 numbers them …

- the svjour3 binding numbers the class's predefined theorems within sections ("Theorem 5.1") without the `envcountsect` option, where svjour3.cls:902-911 numbers them consecutively unless it is given (1803.06644's PDF "Theorem 1"; its routed svjour3-arxiv.cls copy's `\@thmcountersep` stays the binding's since 78h r2, OD #731 rule 14)
- State: open (78h r2, the rev78h review's finding 1): check sv_support_sty.rs's predefined theorems (Perl sv.cls.ltxml L189-223) against svjour3.cls's `envcountsect`/`envcountsame` branches

### pazh_engl.cls:467's title-page journal line ("Astronomy Letters, 2004, accepted") lost under the aa binding's locked `\maketitle` (astro-ph0402416, routed by the copy's …

- pazh_engl.cls:467's title-page journal line ("Astronomy Letters, 2004, accepted") lost under the aa binding's locked `\maketitle` (astro-ph0402416, routed by the copy's `\ProvidesClass{aa}`)
- State: open, minor: a publication note

### a jinstpub/jheppub copy whose own `\maketitle` adds lines loses them

- a jinstpub/jheppub copy whose own `\maketitle` adds lines loses them: 2103.11983's copy prints "Published in JINST as DOI: 10.1088/1748-0221/16/07/P07016" from its edited title page;
- the binding owns `\maketitle` (jheppub's the same exposure; of 121 jinstpub copies the rev78e review read, 4 edit what the title page prints: 3 the header line, kept since 78e r2 as a preprint pubnote, OD #801, and this one)
- State: open, minor (78e r2 residual, the rev78e review's finding 3): a copy-aware rule would read the copy's `\maketitle` for lines the stock one lacks

### jinstpub's (and jheppub's, as Perl's) `\collaboration` and `\proceeding` are pubnotes, which LaTeXML.css shows only on hover (LaTeXML.css:107-119)

- jinstpub's (and jheppub's, as Perl's) `\collaboration` and `\proceeding` are pubnotes, which LaTeXML.css shows only on hover (LaTeXML.css:107-119): 2311.16063's "on behalf of the DUNE collaboration" and 1610.01994's Pixel2016 conference line were visible lines read raw (the rev78e review's finding 10)
- State: open, presentation (CSS; review with the ar5iv-css port at the goal's end)

### `\thanks{\label{fn:c}\email{x}}` (aa probe)

- `\thanks{\label{fn:c}\email{x}}` (aa probe): the `\label` relabels the note, and the e-mail lands on a creator without a name instead of its author (as int45b; rev78e2)
- State:
  - **STOPPED 78f, not brief and no target**: pdflatex (TL 2025) errs on the shape itself — `\thanks{\label{…}…}` in an `\author` under aa.cls (41 Errors), article (101) and llncs: "Use of \@xfootnotemark doesn't match its definition", no PDF — and 0 of the 26,323 sel78d census papers write `\thanks{\label`;
  - left as is

### iopart `\ead` (78f; OD #806, #808, #809, KPE #1031) — the residual

- **iopart `\ead` (78f; OD #806, #808, #809, KPE #1031) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): two `\ead`s inside an `\address` unchanged;
- `\AA rrestad` (2311.12551's own `\AA`, read by the mark reader) open, recorded.
- State: OPEN (residual)

### tlp

- tlp: a `\documentclass{tlp}` (and a renamed copy routed to it, new_tlp.cls: 1405.1833, 1405.2564, 2008.02015, 2008.04008, 2008.06692) reads the author block's affiliation lines after `\\` as more names — 1405.1833 4 → 6 creators, "KU Leuven E-Mail: …" and "transnational University of Limburg E-Mail: …";
- the probe also raises a false "Can't find binding or file for 'tlp.cls'" Warning, OmniBus loaded before tlp_cls.rs (rev77w probe_tlp)
- State: open (77w review finding 2): the raw tlp.cls `\author` lines go through the kernel author splitter; not a few lines on tlp_cls.rs

### 2209.10785: cidr.cls (an acmart copy) comments out acmart's `natbib=true` default; the acmart binding's natbib meets the document's `\usepackage[numbers]{natbib}`, …

- 2209.10785: cidr.cls (an acmart copy) comments out acmart's `natbib=true` default;
- the acmart binding's natbib meets the document's `\usepackage[numbers]{natbib}`, LaTeX's "Option clash" Error where pdflatex loads natbib once
- State: open, stays an Error (main-session decision 2026-10-20; 77w residual)

### adassconf.sty (astro-ph0101121) declares nothing of its own; its first declaration is html.sty's embedded `\ProvidesPackage{html}` (:1374, after 391 definitions), so it …

- adassconf.sty (astro-ph0101121) declares nothing of its own;
- its first declaration is html.sty's embedded `\ProvidesPackage{html}` (:1374, after 391 definitions), so it routes to the html binding;
- converts cleanly (+17 words)
- State: open, left as is (main-session decision 2026-10-20; review finding 7)

### 1802.07182 (aistats)

- 1802.07182 (aistats): the affiliations written as a second `\aistatsauthor` row become creators "University of Cambridge1", "Invenia Labs2", "Cambridge", "UK" (the author's misuse of the setter; its e-mail list is the block's, one contact an address since 77o r3)
- State: open, author misuse (77o r2 review finding 3)

### 1202.1568 (aistats2eM)

- 1202.1568 (aistats2eM): an e-mail list with the domain after the braces (`{seungyeon.kim@, fli@cc., …}gatech.edu`) is not read as an e-mail list and stays inside the affiliation (content kept)
- State: open (77o r2 report)

### 2203.06997 (aistats, paired)

- 2203.06997 (aistats, paired): a `~\\` spacer line before a column's e-mail line makes the kernel `\author` line reading take the e-mail for an affiliation ("Richard E. Turner: University of Cambridge; ret26@cam.ac.uk" both affiliations)
- State: open (77o r3 A/B; content kept)

### `\hyperref[s1]{}` with empty text

- `\hyperref[s1]{}` with empty text: a class deposit holding only it is kept (a `<ref labelref>`, which post fills with "1");
- pdflatex prints nothing.
- Perl does the same in body text
- State: open (77o r2 review finding 10)

### `\shortstack` eats the space after its argument (`A \shortstack[r]{x} Body.` → `…</inline-block>Body.`)

- `\shortstack` eats the space after its argument (`A \shortstack[r]{x} Body.` → `…</inline-block>Body.`): the trailing `OptionalMatch:* [Dimension]` of the prototype reads past it;
- Perl the same
- State: open, KNOWN_PERL_ERRORS #996

### a `{titlepage}` whose class `\@maketitle` prints a paragraph after the title and author

- a `{titlepage}` whose class `\@maketitle` prints a paragraph after the title and author: the paragraph is re-homed before the title (`<para>…Printed cover text…</para><title>T</title>`), the PDF prints it after them;
- base, 77o r1-r3 alike
- State: open (77o r2 review finding 9; guard `perfect_kernel_batch56::unwound_titlepage_rehomes_a_printed_paragraph` asserts the paragraph alone)

### K11 counts a page style the class defines as printing its stores though the class never selects it (`ClassReaders::printed`)

- K11 counts a page style the class defines as printing its stores though the class never selects it (`ClassReaders::printed`): zzfmb.cls's `\def\ps@jhead{\def\@oddhead{\hfil\@journal}…}`, never `\pagestyle{jhead}`, makes the class's `\@journal` default ("Journal of Default Placeholders") a `<pubnote role="journal">` pdflatex never prints — invented visible content (int68a the same).
- Direction: a page style prints only when `\pagestyle`/`\thispagestyle` select it (the class's, its `\maketitle`'s, or the document's) or it is the one the class starts with.
- Repro class-bindings/title_page_typesetting/address_in_a_vbox_with_a_class_line (zzfmb.cls).
- Review rev79a finding 4
- State: open

### K11's line rule (OD #662) after 79a r2 — a store printed in one line with another value of the class's, or glued to its text, stays the class's

- K11's line rule (OD #662) after 79a r2 — a store printed in one line with another value of the class's, or glued to its text, stays the class's: (a) a store kept the class's when the class is loaded (another value in its line printing text then) is not judged again at `\maketitle`, so a document that empties that value still leaves the store in the class's line;
- State: open (accepted unless a paper shows (a) or (d))
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "K11's line rule (OD #662) after 79a r2 — a store printed in ".

### Design

- Design: decide a raw class's title-page line membership from TeX's own output — the speculative title-page run (`before_title_page_runs`) — instead of reading the class's code (`prints_in_a_line_with_another_value`, frontmatter_stores.rs).
- The code-reading rule keeps accreting exceptions (68x's shape test; 79a r1's printed-text rule; r2's sentence full stop, skip dimensions, running-head arguments; r3's head definitions; rev79a3's head optional arguments).
- Needs `harvest_stores` after the speculative run (an ordering change in sect05.rs's `\maketitle`).
- Principled-before-heuristics (user 2026-10-16);
- schedule with FM S2b (the observer reads print events), not as a one-off.
- 79a r2 writer's pattern report
- State: OPEN (design; task #367)

### natbib merged citations (`\cite{a,*b}` under `mcite`/`merge`/`elide`, REVTeX 4.1/4.2 unless `nomerge`)

- natbib merged citations (`\cite{a,*b}` under `mcite`/`merge`/`elide`, REVTeX 4.1/4.2 unless `nomerge`): since 78n the keys are a and b, but the merged entry is not folded into the one before it (natbib.sty:841-860 `\NAT@b*@key`), so the citation prints "[1, 2]" for pdflatex's "[1]", the bibliography keeps b as an entry of its own and every later entry is numbered one higher per merge;
- the `[pre][post]` notes are not added to the entry (`\nat@keyopt@open`/`\nat@keyopt@shut`).
- 1808.00247, 1509.05768, 1206.6986, 1209.2422;
- KPE #1081, guard `perfect_kernel_batch78n::revtex_merged_citation`
- State: open (78n r1-r3 residual; a wrong reference number, content kept)

### `\nonumber` inside an amsmath inner block of an `{equation}` (`aligned`, `split`) removes the equation's number; amsmath's `\let\print@eqnum\@empty` is local to the …

- `\nonumber` inside an amsmath inner block of an `{equation}` (`aligned`, `split`) removes the equation's number;
- amsmath's `\let\print@eqnum\@empty` is local to the block, so pdflatex prints it (2407.10017 `eq:chainMapHam`, PDF 41; int54a the same; found by 78r's `.aux` check, OD #865)
- State: open

### IEEEeqnarray

- IEEEeqnarray: a `\label` written before `\IEEEyessubnumber` on its row names the pre-advanced number in pdflatex (`\@currentlabel` as it is then), ours the row's sub-equation — common where the label opens the environment and the text writes `(\ref{X}a)`, which prints "(23aa)" here for the PDF's "(23a)" (r1→r2 A/B: 1608.03745 `Eq:MMSE_Qvar`, 1705.04514 `modulation`, 1705.10635, 1806.01087, 2003.10007, 2012.02697 `eq:dss`; probe `scratch_79d/r2/eq/lb.tex`).
- Fix direction: such labels name the environment's parent number — the `ltx:equationgroup` labelled with that number's reference forms — needing the label keys a row holds before the back-out (sect07.rs `\lx@eqnarray@label`) and `rearrange_eqnarray` to move them;
- not bounded for 79d r2.
- State: OPEN

### Cases.sty's `{subnumcases}` leaves the numbering in sub-equations (2001.04863: every number from (3) on "3x", r1 and r2 alike); pre-existing (int72a the same)

- Cases.sty's `{subnumcases}` leaves the numbering in sub-equations (2001.04863: every number from (3) on "3x", r1 and r2 alike);
- pre-existing (int72a the same)
- State: OPEN

### IEEEeqnarray*: a `\nonumber` *before* `\IEEEyesnumber`/`\IEEEyessubnumber` on a row still wins (`sect07.rs:270-272`

- IEEEeqnarray*: a `\nonumber` *before* `\IEEEyesnumber`/`\IEEEyessubnumber` on a row still wins (`sect07.rs:270-272`: the starred environment retracts the row at once, so the tags lose `preset` and the later command takes its non-IEEE branch);
- pdflatex prints (1) (2) (2a), ours (1) with the labels unresolved (review rev79d3 finding 2, probe `rev79d3/eq/v.tex`);
- fix: defer the retraction in an IEEE row of the starred environment too;
- pre-existing (r2 the same), 0 corpus uses found
- State: OPEN

### 1806.02261 (106 IEEEeqnarrays)

- 1806.02261 (106 IEEEeqnarrays): 5 printed equation numbers where the PDF has ~114 — the log's Errors start at `\smaller` undefined, then "Missing {" and a non-boxing group; flagged, not silent, but large (79d r3 report); pre-existing (r2 the same)
- State: OPEN

### REVTeX 3 (aps.sty:473-487)

- REVTeX 3 (aps.sty:473-487): `\nonumber` and `eqnarray*` redefine `\label` to an error or a warning, so pdflatex leaves such labels "??";
- ours since 78r resolves an `eqnarray*` label to the next number (the label's content kept, a reference pdflatex does not print; review rev78r finding 6)
- State: open — main-session decision 2026-10-22: keep the resolved number (no visible loss), not PDF-literal "??"

### `\RequirePackage{aas_macros}` before an AASTeX 6+ `\documentclass`

- `\RequirePackage{aas_macros}` before an AASTeX 6+ `\documentclass`: pdflatex 55 errors (an unclean oracle);
- ours prints the class's journal names (review rev78r finding 6)
- State: no action (main-session decision 2026-10-22: the oracle is unclean)

### algorithm2e

- algorithm2e: an algorithm's last line, closed by the environment's end rather than by `\;`, ignores the `\Indp`/`\Indm` offset (`\lx@prepend@indentation` runs only from `\lx@algo@endline`; algorithm2e_sty.rs env template): `\Indp Bb\; Cc \end{algorithm}` or `\DontPrintSemicolon … \Return Gg \end{algorithm}` sets Cc / "return Gg" at the margin, pdflatex one level in;
- 2508.06088's three algorithms end with `\Return` at the margin (pdflatex x 165.1, one level in).
- int54a and 78r alike;
- review rev78r2 finding 1.
- Fix: give the open last line its indentation before the environment's group ends (a no-op on an empty last line)
- State: open

### algorithm2e `\DontPrintSemicolon` / `\PrintSemicolon` are assigned globally (algorithm2e_sty.rs, Perl algorithm2e.sty.ltxml:131-136 the same) where algorithm2e.sty's are …

- algorithm2e `\DontPrintSemicolon` / `\PrintSemicolon` are assigned globally (algorithm2e_sty.rs, Perl algorithm2e.sty.ltxml:131-136 the same) where algorithm2e.sty's are a local `\renewcommand`, so a later algorithm in the same document loses every printed ";" (pdflatex "Hh;", ours "Hh"; int54a, 78r alike; review rev78r2 finding 4; SHARED)
- State: open

### cleveref `\label[type]{…}` after a `\caption` inside a minipage panel of a figure loses its type

- cleveref `\label[type]{…}` after a `\caption` inside a minipage panel of a figure loses its type: the panel (or the folded figure) carries the label but keeps `creftype` "fig." — `\begin{figure}\begin{minipage}{.4\linewidth}\caption{A}\label[appendix]{mp}\end{minipage}…` gives "fig.
- 1" where pdflatex prints "appendix 1" (0 errors; a subfigure's works, "appendix 2a");
- the type's tags have no target when `\lx@cleverref@labeltype` runs (cleveref_sty.rs:61-72 `continue`s) and the panel fold moves the labels, not the type's tags.
- int54a and 78j alike;
- review rev78j5 finding 2.
- Fix: carry the type's tags with the labels when the panel/fold merges them, or settle the type once its owner exists
- State: open

### A `\begingroup` left open in held text raises an Error TeX does not

- A `\begingroup` left open in held text raises an Error TeX does not: an `\autoref` name `\begingroup Fig`, `\creflabelformat{equation}{\begingroup(#2#1#3)}` — pdflatex 0 errors (only "\end occurred inside a group at level 1"), ours 1 ("Attempt to end mode restricted_horizontal");
- `\crefformat{figure}{\begingroup#2Fig.~#1#3}` — pdflatex 0 errors and the author's "Fig. 1", ours 1 ("leaves 1 group(s) open") and cleveref's default "fig.
- 1".
- 78j r5 (r4 raised 2);
- review rev78j5 finding 3.
- Direction: when only `\begingroup` frames are left open, `digest_in_own_group` unwinds them without an Error and keeps the form
- State: open

### subfigure's `\subref{b}` prints "1(b)" where pdflatex prints "(b)"

- subfigure's `\subref{b}` prints "1(b)" where pdflatex prints "(b)": subfigure.sty:412-420 `\subref` reads the `sub@b` label its `\label` writes (the panel's own number), the binding's `\subref` is `\ref` (subfigure_sty.rs, Perl subfigure.sty.ltxml:181 the same);
- since 78t numbers every panel more papers reach it (review rev78t finding 7).
- Fix: `\subref` shows the panel's own tag (its `ltx:tag` without the figure's number)
- State: open

### A `\label` after a raw `\refstepcounter` in a paragraph (a user environment's counter, 1505.02540: 8 references print "2", the section's) names the enclosing unit

- A `\label` after a raw `\refstepcounter` in a paragraph (a user environment's counter, 1505.02540: 8 references print "2", the section's) names the enclosing unit: no schema element there can carry the label and the number (`ltx:para`/`ltx:p` have neither `labels` nor `ltx:tags`).
- Proposed: `ltx:para` takes `labels` and an optional leading refnum-only `ltx:tags` (the 78g `ltx:picture` mechanism; the XSLT drops `ltx:para/ltx:tags`);
- reach ≥ 24 papers (78t item 5)
- State: open, awaiting the schema ruling

### algorithm2e sets a caption's text in a box placed at the style's caption position (`\global\sbox\algocf@capbox`, algorithm2e.sty:2305-2336: the bottom for …

- algorithm2e sets a caption's text in a box placed at the style's caption position (`\global\sbox\algocf@capbox`, algorithm2e.sty:2305-2336: the bottom for `plain`/`boxed`, the top for the ruled family), so a `\label` in the caption's text is written there, not at its source position: a key labelled in a `plain` algorithm's caption text and again after its last line names the algorithm in pdflatex, the line here (2511.10625 `alg:general_astar`: pdflatex 1, 78t 13; int58a 1).
- Fix: a caption-text `\label` of an algorithm2e float counts as written at the caption's position (OD #877 residual)
- State: open

### Sub-figures in a `tabular` cell (`\begin{tabular}{ccc}\subfigure[]{…} & \subfigure[]{…} \\ …`) are lifted out of the table and their rows guessed from their widths, not …

- Sub-figures in a `tabular` cell (`\begin{tabular}{ccc}\subfigure[]{…} & \subfigure[]{…} \\ …`) are lifted out of the table and their rows guessed from their widths, not the table's rows (1309.3581, astro-ph0205132: neither the PDF's grid; int58a and 78t alike);
- their stated widths are not evaluated at the class's real geometry either (`note_stated_widths` does not enter an alignment's cells: 1911.11519 S3.F3, `{subfigure}{\hsize}` in `p{0.45\textwidth}` cells under elsarticle's geometry, 4+3 in the PDF, 3+1+3 here; 2502.05562 S5.F4)
- State: open

### An adjustbox's (or `\resizebox`'s) shell around sub-floats renders a blank oversized row

- An adjustbox's (or `\resizebox`'s) shell around sub-floats renders a blank oversized row: the sub-floats leave the box (`absorb_figure_material`), and the box stays behind as an `ltx:p` panel holding `<inline-block width="1393.3pt">` with an empty `ltx:p` (review rev78x3 finding 4, probe `q4.tex` Figs 1-2: `\begin{adjustbox}{width=\linewidth,center}` of four sub-figures; the XSLT gives it `style="width:1393.3pt"`, `.ltx_inline-block` is `display:inline-block`, LaTeXML.css:250).
- Pre-existing (int68a the same for the environment form);
- 78x keeps the four sub-floats one row after it (`note_panel_hbox`).
- Direction: drop a box left with no content once its floats are lifted out, or fold its scale onto them
- State: open

### A raster graphic's explicit size comes out at 72/72.27 of it, rounded up to a whole pixel

- A raster graphic's explicit size comes out at 72/72.27 of it, rounded up to a whole pixel: `image_graphicx_sizer`'s pixel branch (`latexml_core/src/util/image.rs`, `apply_graphicx_ops` with `units_per_bp = DPI/72.27`) multiplies graphicx's bp values (`parse_graphicx_options`, `to_bp`) by pixels per pt, as Perl does (Util/Image.pm:162-163 `to_bp`, :237 `ceil($a1 * $dppt)` with `$dppt = DPI/72.27`; a Perl-origin defect, KNOWN_PERL_ERRORS #1142): `\includegraphics[width=114.6pt,height=1cm]{x.png}` is 114.187pt (158px) where TeX's box is 114.6pt, so two such panels stay one row of REVTeX `[preprint,twocolumn]`'s 229pt column where pdflatex breaks them (review rev78x3 finding 6; probes `rvpt.tex`, `rvpt2.tex`).
- State: open
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "A raster graphic's explicit size comes out at 72/72.27 of it".

### `\adjustbox{max width=\columnwidth}{\subfloat[a]{…}\subfloat[b]{…}}` in a float (revtex4-1 twocolumn)

- `\adjustbox{max width=\columnwidth}{\subfloat[a]{…}\subfloat[b]{…}}` in a float (revtex4-1 twocolumn): the two sub-figures land outside the scaled `ltx:inline-block`, which keeps an empty `<p><text/></p>`, and come out `P \| PP` where pdflatex sets one line holding one adjustbox;
- the figure counter steps twice, so the next float is numbered one too high (refnums `5`, `5a`… where pdflatex's `.aux` reads `\numberline {4}`).
- int69b, 78x r4 and r5 alike (pre-existing): adjustbox digests its content twice — fix in the adjustbox/subcaption interaction.
- Review rev78x5 finding 1, probe `rev78x5/p/m3.tex` (third figure)
- State: OPEN (RED)

### Sub-figures lifted out of their `figure*` into a paragraph of floats settled into a row (`settle_float_paragraph`) beside the float's minipage remnant (2202.03884 …

- Sub-figures lifted out of their `figure*` into a paragraph of floats settled into a row (`settle_float_paragraph`) beside the float's minipage remnant (2202.03884 S4.SS1.fig1: `{subfigure}{0.33\columnwidth}`s with `{center}` and `\resizebox` bodies around pgfplots `\input`s; the PDF 2+1, int67a 2+2, 78x r2 one per row, 78x r3 as int68a): the lift is the defect (the sub-figures belong to S4.F5);
- the row is set at the registers' widths (OD #885)
- State: open

### Rows TeX keeps tight or overfull (a `~` tie, `}\subfloat` with no breakpoint, tight TikZ, two 4.5cm sub-floats in a 252pt column, a nested `\subfloat` beside a 4.5cm one …

- Rows TeX keeps tight or overfull (a `~` tie, `}\subfloat` with no breakpoint, tight TikZ, two 4.5cm sub-floats in a 252pt column, a nested `\subfloat` beside a 4.5cm one with ~1pt to spare in a 246pt column — base the same, review rev78x5 finding 2, `rev78x5/p/m3.tex` fourth figure): 1108.5543 Fig 9, 2009.04593 Fig 5, 1712.01536 Fig 2, 1703.09165 (2 floats), 2406.05437 Fig 12;
- State: accepted residual (user 2026-10-22: overflow)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Rows TeX keeps tight or overfull (a `~` tie, `}\subfloat` wi".

### Panel grid markup

- **Panel grid markup** (grid design payload 3): the XML says a float's rows only implicitly (the runs between `ltx:break`s);
- State: see the archive
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Panel grid markup".

### The natural `cssstyle` is the size pdflatex prints a graphic at (`first_natural_size`, util/image.rs; 79a), but post renders the candidate it finds most desirable …

- The natural `cssstyle` is the size pdflatex prints a graphic at (`first_natural_size`, util/image.rs; 79a), but post renders the candidate it finds most desirable (`latexml_post/src/graphics.rs:321-371`), which may be another file of another shape: 2107.02597's gnuplot figures (`\DeclareGraphicsExtensions{.pdf,.eps,.png,.jpg,.jpeg}`, a 640x480 png beside each eps) render the png (4:3) at the eps's `36.135em x 25.294em` (aspect 1.43), about 7 % out of proportion (int68a the same, `cssstyle` A = B).
- Options: post renders the candidate the box is sized from when it can convert it (an eps through gs), or the `cssstyle` keeps the box's width and lets the height follow the rendered file.
- Review rev79a finding 5
- State: open

### `\Gin@extensions` is seeded with pdftex.def's list (`graphics_sty.rs`, pdftex.def:597-600) under every driver option, though `\Gin@driver` names the option's driver …

- `\Gin@extensions` is seeded with pdftex.def's list (`graphics_sty.rs`, pdftex.def:597-600) under every driver option, though `\Gin@driver` names the option's driver (`graphics_driver()`): `\usepackage[dvips]{graphicx}` loads dvips.def in TeX, whose list is `.eps,.ps,.eps.gz,.ps.gz,.eps.Z,.mps` (dvips.def:202), so pdflatex takes `q.eps` (40.15pt) beside `q.png` where ours (int68a, 79a) takes the png (144.54pt);
- likewise `[dvipdfmx]`/`[xetex]`.
- Fix: seed by `graphics_driver()` from each driver file's list.
- A corpus-wide shift for every `[dvips]` paper with a raster and a vector of one basename: its own A/B. Review rev79a2 finding 3, probe `/home/deyan/.claude/jobs/356a0c47/tmp/rev79a2/g/gh.tex`
- State: open (ranked: after the panel grid payloads)

### graphicx's natural-size keys are not read by the sizer

- graphicx's natural-size keys are not read by the sizer: `bb=`/`bbllx=`…, `natwidth=`/`natheight=` and `hiresbb` set the natural size in TeX (Perl reads none of them either): `\includegraphics[bb=0 0 10 10]{e.eps}` of a 40bp EPS is 10.03749pt in pdflatex, ours 40.15pt (int67a 28.908pt).
- Witness cond-mat0610021 (`bb=0in 0in 2in 2in`; Fig 11's sub-figure then breaks where TeX sets one overfull `\hbox`).
- Review rev78z finding 5
- State: OPEN

### A float LaTeX defers writes its `\newlabel` when placed, so a later `\label` of the same key loses to it

- A float LaTeX defers writes its `\newlabel` when placed, so a later `\label` of the same key loses to it: `\begin{figure}[p]\caption{Deferred}\label{k}\end{figure}\section{Two}\label{k}See \ref{k}.` — pdflatex "See 1.", ours, int58a and Perl "See 2." (review rev78t2's lb2; OD #877, KPE #1131);
- rare
- State: open

### An author's mark whose label holds any control sequence prints nothing

- An author's mark whose label holds any control sequence prints nothing: the mark reader reads the label from the operand's unexpanded text (`author_macros.rs:198` `clean_frontmatter_labels(&operand.to_string(),…)`), which no list answers, so `\textsuperscript{\relax 3}` loses its "3" (pdflatex "Ann Alpha3"), `\textsuperscript{\one}` and `\textsuperscript{\protect 2}` their "1" and "2", `\textsuperscript{\relax 1\thanks{a}}` and `\textsuperscript{\protect 2\thanks{b}}` keep the note but lose the label, as do `1\protect\protect\thanks{b}`, `{\protect}1\thanks{f}` and `\protect\protect\thanks{c}`, and `1,\protect\one\thanks{n}` gives `<sup>1</sup>` where pdflatex prints "1,1".
- More open facts: the archived record of this row.
- State: open
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "An author's mark whose label holds any control sequence prin".

### A note macro reached through four macros (`\fnM→\fnN→\fnO→\fnP{8\footnotemark}`) inside an author's mark loses both its "8" and the footnotemark (pdflatex "Di Delta8†")

- A note macro reached through four macros (`\fnM→\fnN→\fnO→\fnP{8\footnotemark}`) inside an author's mark loses both its "8" and the footnotemark (pdflatex "Di Delta8†"): `makes_a_note`/`prints_beside_a_note` (frontmatter/classify/preprocess.rs) look three deep at most, so the macro is neither taken out nor read in place and stays in a label nothing answers.
- int62a, int63a and 78v alike;
- synthetic.
- Fix: past the depth bound, fall back to taking the macro out whole (the int62a behaviour for a note macro).
- Review rev78v2 finding 6b;
- RED repro `sectioning-frontmatter/author_mark_four_deep_note_macro_chain`
- State: open

### Author marks

- Author marks: inside a note macro's body, a `\protect` before another note-making macro (`\newcommand\fnL{\footnotemark}`, `\newcommand\fnB{7\protect\fnL}`, `Lu L\textsuperscript{\fnB}`) is not label-safe, so the macro is taken out whole and its "7" lands in the name text: `L7[mark]` where 78v r2 gave `L[mark]<sup>7</sup>` (pdflatex "L7§").
- State: open
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Author marks".

### A formula silently degrades to `ltx_math_unparsed` under load

- A formula silently degrades to `ltx_math_unparsed` under load: 1001.0023 `S9.E2.m1.pic1.m2` once in a census run (FM r9 writer's set-mode census), never in 3+1+2+2 reruns;
- the log identical, so no warning — the math parser's 2 s pruned-only budget (`parser.rs:235`) is the only time-based path found (revfms2a2_9 finding 7).
- A time budget that changes output without a diagnostic hides regressions in A/Bs: emit a Warning when the budget cuts a parse, or make the cut deterministic
- State: open

### Complete breakage at 100 % (user 2026-10-24)

- **Complete breakage at 100 % (user 2026-10-24)**: a missing image, a `\ref` or `\cite` with no target, where pdflatex resolves it.
- Not ours (pdflatex or the arXiv PDF breaks too): a file not shipped, xr without the other `.aux`, a stale shipped `.bbl` (ruling 2026-10-24: 2509.06285, 2205.10368, 2402.06576, 2311.01729, 2205.00263, 2004.00833, 2507.01293, 1806.02261), the harness\'s main-file guess, a document\'s own `\def\label`, a shipped namesake of a TL package (ruling 2026-10-17 H1: 0802.3014\'s and 1210.0342\'s pretex `units.sty`).
- More open facts: the archived record of this row.
- State: OPEN
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Complete breakage at 100 % (user 2026-10-24)".

### FM S2b: the observer's writer on by default for revtex4_support (OD #837 "S2b r1") — the residual

- **FM S2b: the observer's writer on by default for revtex4_support (OD #837 "S2b r1") — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): left 2608.23944 (an uncarried symbol line under ruling 2 in a one-author group: a ruling) and 2602.16003;
- open (ruling): the mark-picked path does not show a line's mark (219 guards).
- State: OPEN (residual)

### K11 store residuals after 95d

- **K11 store residuals after 95d** (OD #939-#941; the scorer's four lost-words papers 2607.06973, 2608.26053, 2307.16262, 1202.2889 fixed; 95d-r2: a setter's own store only when no running code reads it, a date's label in an `\else` branch, an unlabelled date unnamed, fancyhdr styles read not run; 95d-r3: a date's label read in any code the title page reaches, a label phrase ending at a `;` or `,`, the label before a store's emptiness test, a role's default only where its word is printed right before the store): unsupported shapes, each a one-line input — (a) a footer token register selected only at the class's top level or in `\AtBeginDocument` (`\pagestyle{firststyle}` outside `\maketitle`), or set by `\fancyfoot[L]{\the\correspondingauthor}` under `\pagestyle{fancy}` outside `\fancypagestyle` (fancyhdr's fields stay no-ops), stays unread;
- More open facts: the archived record of this row.
- State: open
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "K11 store residuals after 95d".

### 95d r3 review should-fixes (rev95d-r3 SHIP, 2026-10-31; synthetic only, no census paper changed this way)

- **95d r3 review should-fixes (rev95d-r3 SHIP, 2026-10-31; synthetic only, no census paper changed this way)** — (S1, a content loss on its shape) the `\else`/`\or` whole-test removal (`frontmatter_stores.rs` `line_label_before` ~:5302-5315, `end = past` ~:5349-5353, `enclosing_conditional` ~:5492) checks only that nothing precedes the store in its branch, so text printed after the store inside the test leaves the title page with it and is printed nowhere: `Received: \if@twocolumn\else\@received\ (preprint version)\fi` loses "(preprint version)" (base and r2 keep it; a static scan of the 9,502 class-shipping census zips found no such shape) — fix: take the whole test only when the store's branch holds just the store and layout and the test is the store's own emptiness test, else the r2 path (the `\ifcase`/`\or` variant too);
- More open facts: the archived record of this row.
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "95d r3 review should-fixes (rev95d-r3 SHIP, 2026-10-31; synt".

### 95e residuals (OD #938)

- **95e residuals (OD #938)** — the several-address `\email` give-back (2109.07363 three creators again, 0 E) leaves: (a) `\address{$1.$ …}` labels unread, so the reading's upfront grid gives the entries by position, its pile lacking the listed call (`answer_listed_emails` runs after `Resolver::distribute_upfront`): 2109.07363 Liu {1}, Tao {2} where the PDF prints {1,2} for both (RED `amsart_dollar_numbered_addresses_under_shared_marks`), synthetic reversed marks both wrong where int123a had one wrong (RED `amsart_dollar_numbered_addresses_under_reversed_marks`, rev95e S1);
- More open facts: the archived record of this row.
- State: open
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "95e residuals (OD #938)".

### Unlanded payload branches (inventory 2026-10-31; all work reintegrates into perfect_kernel, user 2026-10-31)

- **Unlanded payload branches (inventory 2026-10-31; all work reintegrates into perfect_kernel, user 2026-10-31)** — each kept on its branch for a recorded reason, none silently: `pk/66p` 41122534fa (a sub/superscript mark outside math, tex.web §1046; ruled unlanded, `feedback_rulings_error_fatal_policy`);
- if not, retire with its residual rows.
- **RETIRED 2026-11-01** (row "K9 row F residual": 264 Errors in 29 of its 62 witnesses on the pushed tip).
- State: open
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Unlanded payload branches (inventory 2026-10-31; all work re".

### K9 row F residual: a macro that closes and reopens the author block's environment (66u RETIRED 2026-11-01, user ruling 2026-10-31)

- **K9 row F residual: a macro that closes and reopens the author block's environment (66u RETIRED 2026-11-01, user ruling 2026-10-31)** — `\author{\begin{tabular}{c}Ann\end{tabular}\hskip1em\begin{tabular}{c}Bob\end{tabular}}`-style blocks (sig-alternate `\alignauthor`, hand-made `\end{tabular}…\begin{tabular}` breaks) read as `\and` by pdflatex leave a run of Errors: on 66u's 62 code/test witnesses (`pk/66u` `perfect_kernel_batch66u`; job tmp `w66u.txt`, `m66u/run.sh`) the pushed tip (int125c `8a87df771c`) gives 264 Errors in 29 papers, 33 error-free (int123a 272 / 32; the bundle fixed 2405.18462) — `unexpected:\hbox` 66, `\lx@personname` 52, `\endgroup` 52, `\@@tabular` 52, `\@end@tabular` 39, `&` 3;
- State: open
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "K9 row F residual: a macro that closes and reopens the autho".

### Bundle int126a/int126b (97c-98e) — the residual

- **Bundle int126a/int126b (97c-98e) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): a REVTeX `superscriptaddress` `\affiliation` with an unmarked first line lands at the block (1507.03321, 2312.15705);
- 2506.19040's `drohr@` copied to the other `*` authors;
- BESIII `(A)` sub-entries (R6);
- no Mark event for nucleus formulas;
- 2112.13115's leading "and";
- sp2 (an unanswered line's label dropped);
- 1609.03822's `\pounds` label unsplit.
- State: OPEN (residual)

### Frontmatter on the digested representation — the freeze and the refactor's batches 90a-95c (task #423; now `parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`, row 0b) — the …

- **Frontmatter on the digested representation — the freeze and the refactor's batches 90a-95c (task #423; now `parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`, row 0b) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): REDs `amsart_email_of_a_two_name_call_spelling_an_accented_name` (2112.10604) and `revtex_email_spelling_its_holders_given_name`;
- REDs' witnesses 0711.3633, 0809.3090 (`santos@` off Santos Bravo Yuste);
- More open facts: the archived record of this row.
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Frontmatter on the digested representation — the freeze and ".

### A routed amsart note belongs to the last live author at `\maketitle`; needs a change tag and a ruling

- **A routed amsart note belongs to the last live author at `\maketitle`; needs a change tag and a ruling** (92a review rev92a_r3 s1; main-session decision: defer to the ams written-family work): amsart.cls:516 keeps each `\thanks` in `\thankses`, run at `\maketitle`, and its `\email` (:508) appends to `\addresses`, so a `\thanks{Correspondence: \email{x}}` between two authors, or before every `\author`, prints x after the last author's address.
- RED catalog repros `class-bindings/amsart_thanks_between_authors_email`, `amsart_thanks_before_every_author_email`
- State: DEFERRED (ams written family)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "A routed amsart note belongs to the last live author at `\ma".

### Compound marks: one rule for the superscript path, the formula path's runs and the legend's label (94a r9 residual; OD #928 round 9, OD #927)

- **Compound marks: one rule for the superscript path, the formula path's runs and the legend's label (94a r9 residual; OD #928 round 9, OD #927)**: the three readers part a mark of several symbols differently — the superscript path at commas only (`apply::show_legend_mark`'s `ltx:sup` loop: `Ann Able$^{*\dagger}$` under a `$^{*}$Corresponding author` legend shows "*†" beside the note's ∗, the star twice, once as ASCII `*`; w01), the formula path by `token_runs` and `repeated_symbols` (`$^{1,*\spadesuit}$`'s ∗ taken with the comma before it, "1♠"), the reading's labels by `Entry::compound` and `mark:parted` (`**` read as `*,*`, so a `$^*$` legend is the `**` carrier's, w11; the kernel then hides the `**`: `\author{Ann Able$^{1,**}$, Bob Baker$^{1}$\\ $^1$Univ X\\ $^*$Corresponding author}` prints "1,∗∗", ours no mark, as 93a), and a `$^{*\dagger}$Corresponding author` legend reaches Bob Baker, who carries only `*` (`Ann Able$^{1,*\dagger}$ and Bob Baker$^{1,*}$`, w10, since 93a).
- More open facts: the archived record of this row.
- State: OPEN (refactor)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Compound marks: one rule for the superscript path, the formu".

### A trailing answered name takes the unmarked in-place line, before its own (94a r9 residual P3)

- **A trailing answered name takes the unmarked in-place line, before its own (94a r9 residual P3)**: `ANN ABLE$^1$ and BOB BAKER and CID COLE$^1$ \affil{$^1$Univ A} \affil{Univ B}` (a class printing `\affil` in place) gives Univ B to BOB BAKER and CID COLE, CID COLE's Univ B before his Univ A;
- RED catalog repro `raw_class_affil_run_trailing_answered_name_takes_the_unmarked_line`
- State: OPEN
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "A trailing answered name takes the unmarked in-place line, b".

### IEEEtran's `annotate=new` affiliation block after a marked one, contact order (94a r9 residual P4; since 93a)

- **IEEEtran's `annotate=new` affiliation block after a marked one, contact order (94a r9 residual P4; since 93a)**: `\IEEEauthorblockN{Ann Able$^1$ and Bob Baker}\IEEEauthorblockA{$^1$Univ X}\IEEEauthorblockA{Univ Y}` gives Ann Able Univ Y before Univ X (the print's order X, Y);
- the same with `\IEEEauthorrefmark{1}` (review rev94a r8 z11, z07).
- RED catalog repro `ieee_affil_new_block_after_a_marked_name_contact_order`
- State: OPEN

### A raw class's line whose mark only a later `\author` requests stays the class's line (94b r7 residual; it was 94b r4's S1)

- **A raw class's line whose mark only a later `\author` requests stays the class's line (94b r7 residual; it was 94b r4's S1)**: in an appending class (capa-like `\let\@author\@empty\renewcommand\author[2][]{\g@addto@macro\@author{#2 }}`), `\author{Ann Able\inst{1}}\address[\inst{2}]{Univ B, City}\author{and Bob Baker\inst{2}}\maketitle` prints "(2) Univ B, City" where it is called and Bob Baker gets no contact: at the call no author requests 2, so the line is in doubt — the class's, digested once (`frontmatter_stores.rs` `class_contact`, OD #936 round 7).
- More open facts: the archived record of this row.
- State: OPEN (deferred)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "A raw class's line whose mark only a later `\author` request".

### Per-entry wraps of nested containers, and an `\href` around entries (94b r7 review residuals; synthetic, triage 2026-10-31: RED catalog, no fix round)

- **Per-entry wraps of nested containers, and an `\href` around entries (94b r7 review residuals; synthetic, triage 2026-10-31: RED catalog, no fix round)**: (1) a container inside a kernel text font command with more entries after its group loses the container's font (`\textbf{\emph{$^1$…\\ $^2$…}} $^3$…`: bold kept, italic lost; `emitter/plan.rs` `entry_forms`/`container_wrap`);
- More open facts: the archived record of this row.
- State: OPEN
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Per-entry wraps of nested containers, and an `\href` around ".

### A `\thanks` opening with `\label` sits on a nameless creator (94a r9 residual P5; 94b r4 and on)

- **A `\thanks` opening with `\label` sits on a nameless creator (94a r9 residual P5; 94b r4 and on)**: llncs's `\author{Ann Able\thanks{\protect\label{fn1}Funded.}\inst{1} \and Bob Baker\inst{2}}` keeps "Funded." on a creator of no name after the two authors, not on Ann Able: inside a frontmatter item `\label` is `\lx@set@frontmatter@label` (Base_Utility.pool.ltxml:330, :565), so the note is an annotation labelled `LABEL:fn1` that no entry requests.
- Perl drops the text with a Warning (KNOWN_PERL_ERRORS #1204).
- Review rev94a r8 pl/p02.
- RED catalog repro `llncs_thanks_label_on_a_nameless_creator`
- State: OPEN

### A `\footnotemark` after a name's mark invents a superscript (94a r9 residual P6)

- **A `\footnotemark` after a name's mark invents a superscript (94a r9 residual P6)**: a class printing `\affil[2]{…}` in place, `BOB BAKER\textsuperscript{2}\footnotemark \affil[2]{Univ B}` with a later `\footnotetext`: the name holds the footnote then `<sup>∗</sup>`, the 2 lost and a ∗ nothing prints;
- unlabelled `\affil{Univ B}` `<sup>2,∗</sup>`, `\footnotemark[3]` `<sup>‡</sup>` (review rev94a r8 x02, x13, x14).
- RED catalog repro `raw_class_affil_footnotemark_after_a_mark_invents_a_sup`
- State: OPEN

### Marks a document macro prints, or picture keys, are not split (94a r9 residual P7; content kept)

- **Marks a document macro prints, or picture keys, are not split (94a r9 residual P7; content kept)**: `\newcommand\sa[1]{\textsuperscript{#1}}` and `\author{ANN ABLE\sa{1} \affil[1]{Univ A} BOB BAKER\sa{2} \affil[2]{Univ B}}` (a class printing `\affil[k]` in place) stay one creator holding both lines;
- the names boxed the same;
- a tikz circled key `\affil[\ctwo]{…}` (review rev94a r8 y01, y05, x10).
- RED catalog repro `raw_class_affil_marks_by_macro_not_split`
- State: OPEN

### A mark formula rewritten into a superscript keeps its italic digits (94a r9 residual P8; since 93a)

- **A mark formula rewritten into a superscript keeps its italic digits (94a r9 residual P8; since 93a)**: `Ann Able$^{1,\dagger\dagger}$` → `<sup><text font="italic">1</text></sup>`, pdflatex's digits upright (math digits are upright).
- Pinned as the known residual by the rv11 and rv15 rows of `perfect_kernel_batch94a::a_legend_of_several_symbols_shows_its_mark` and the rows of `a_repeated_symbol_beside_a_legend_is_its_own_mark` (update them with the fix);
- no catalog repro of its own, since those guard rows pin it
- State: OPEN

### just.cls binding (91a; cpc-hepnp.cls under the PhiPsi15 name) — the residual

- **just.cls binding (91a; cpc-hepnp.cls under the PhiPsi15 name) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): 91c open: the cpc-hepnp binding's house mark convention (an item ending in `)` a footnote mark, `;` a separator; the `;n)` residual, 341 papers; RED `class-bindings/just_semicolon_email_marks`), then a structural fail-safe for an unmarked multi-line address under marked names and the 611-paper A/B.
- State: OPEN (residual)

### nature-pre / nature_mod binding: the class's own list environments (92f) — the residual

- **nature-pre / nature_mod binding: the class's own list environments (92f) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): open: a paragraph a relaxed setter prints after `\maketitle` (article.cls:199/:233 let `\date`, `\author` `\relax`) between the list and the abstract keeps the abstract above the list (2608.12563, 2102.01782, 2111.09589).
- Also 2412.11966's author line (the same shape).
- Pre-existing (rev92f): 1408.2522's copy prints "Received …;
- Accepted …" from its own `\@maketitle` (nature_mod.cls:59-63), lost;
- `\author{…$^{1,2 \ast}$}` under article or TL nature keeps only `<sup>1</sup>`.
- State: OPEN (residual)

### A body `\ref` to an `\institute` entry's `\label` prints nothing (found 2026-10-30)

- **A body `\ref` to an `\institute` entry's `\label` prints nothing (found 2026-10-30)**: aa's `\institute{Univ X\label{ia} \and Univ Y\label{ib}}` keys `\inst{\ref{ib}}` correctly (the observer's `ref:` keys, OD #913), but a body "Univ Y is institute `\ref{ib}`." gives `<ref labelref="LABEL:ib"/>` with no target — our HTML an empty `ltx_missing_label` span, Perl's the red key "LABEL:ib";
- pdflatex prints 2 (the label is the institute counter's, which is why the same `\ref` inside `\inst` prints the number).
- The affiliation contact carries no label or refnum.
- Content loss shared with Perl;
- reach to be sized (body refs to institutes).
- RED repro `tools/perfect_kernel/repros/class-bindings/aa_body_ref_to_institute_label.tex`.
- State: OPEN

### frontmatter notes read as the class prints them (92e, 92g) — the residual

- **frontmatter notes read as the class prints them (92e, 92g) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): the version-dependent shapes left are elsarticle's `\journal` (copies up to v3.3 print "Preprint submitted to X", 2609.40131; TL 2025's v3.4c none) and elsart's ("Article published in …" when `\volume` is set);
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "frontmatter notes read as the class prints them (92e, 92g) —".

### An unmarked address after a name list goes to the last name only (iopart, informs4; pre-existing, Perl's default)

- **An unmarked address after a name list goes to the last name only (iopart, informs4; pre-existing, Perl's default)**: `\author{A, B and C}\address{X}` (iopart) and `\AUTHOR{A, B}\AFF{X}` (informs4) give X to C (B) alone, where the print sets X under all of them.
- For iopart this is Perl's: `\lx@add@affiliations` (Base_Utility.pool.ltxml:742) attaches to the last creator by default.
- Candidate: the same default 91a's just binding opts into (`annotate=all`/`run`: an unmarked address belongs to the run of names it follows), declared per binding;
- found by 91a r2's sibling audit (the other `\lx@add@affiliation@marked` callers already annotate or reach it only through a marked branch)
- State: DEFERRED to frontmatter refactor stage 6/7

### Frontmatter correctness scorer (2026-10-27)

- **Frontmatter correctness scorer (2026-10-27)**: our creators against the paper's own print (pdflatex TL 2025, pages 1-2, sources and outputs only — no arXiv API, user 2026-10-27), on 3,000 random arXiv papers (`~/data/pk_agents/main/fm_scorer/`: `score2.py`, `conv2.sh <id> <bin> <outroot>`, `view.py`; seed 20261027).
- a hand-typeset title page with no `\author` and no creators 6.9 % (178; all 8 sampled print their authors; Perl the same — a beyond-Perl capability, needs a ruling);
- residuals in `parity/DEFERRED_FAMILIES.md` "Abstracts a class stores or queues";
- More open facts: the archived record of this row.
- State: OPEN (rulings)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Frontmatter correctness scorer (2026-10-27)".

### dev giant-list census (2026-11-01) — the residual

- **dev giant-list census (2026-11-01) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): BESIII `(A)`/`(C)` sub-labels deferred (R6, the difficulty rule);
- M3 PoS's one `\@author` (1711.04132, R9, settled by Z14);
- scorer limits L1-L4 (1201.5677, 1904.03257, 2510.23732).
- Also L4 math in a source name (2310.11912) and L5 a colon heading kept in the source name (2206.00684);
- aa's `X Collaboration\thanks{…}: R.~Abuter\inst{8}` keeps the collaboration an author (2206.00684, 97f optional).
- State: OPEN (residual)

### Holdout10k v3 (2026-11-01; spent: any further read is a fresh holdout v4, read once at the end — the plan's rule, user 2026-11-06; the rules below stay the judge's)

- **Holdout10k v3 (2026-11-01; spent: any further read is a fresh holdout v4, read once at the end — the plan's rule, user 2026-11-06; the rules below stay the judge's)**: **Papers with an Error or Fatal in their frontmatter are excluded from every count (user 2026-11-01: "completely exclude papers with errors in frontmatter from this work, they confound the results. Errors elsewhere are OK to keep")** — `score_tiers.py` `frontmatter_error`, status `fm_error`: an Error/Fatal located in the main file before the first sectioning command, in a class or package the preamble loads, or in a file input before that point, or an `ltx:ERROR` inside a frontmatter element;
- More open facts: the archived record of this row.
- State: OPEN (the exit bar's judge)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Holdout10k v3 (2026-11-01; spent: any further read is a fres".

### a marked author list merged by a name the split test does not know (97a) — the residual

- **a marked author list merged by a name the split test does not know (97a) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): a `NON_NAME_WORDS` word in a surname keeps a mark-covered list whole (1212.4143);
- reversed marked names `Surname, I.\altaffilmark{1}` (1903.10014);
- a `(B)`-led institute makes a nameless extra creator.
- State: OPEN (residual)

### dev20k `aff_missing` census (int125b, 2026-11-01; 451 creators / 146 papers: 390 / 114 classed, 61 / 32 long tail, ≈43 scorer limits)

- **dev20k `aff_missing` census (int125b, 2026-11-01; 451 creators / 146 papers: 390 / 114 classed, 61 / 32 long tail, ≈43 scorer limits)** — **F1** (110 / 29) the family's writer gate is shut while the reading is right (inst_support 69 / 14 — the names census's M3; the jheppub family jheppub/jcappub/jinstpub 30 / 9; undeclared policies 11 / 6: aas_support, spie, emulateapj, PoS; `inst_support_sty.rs:11`, `jheppub_sty.rs:11`, `latexml_contrib/src/jcappub_sty.rs:14`, `jinstpub_sty.rs:18`, `policy.rs:122-127`, gate `writer/mod.rs:45-71`; witnesses 2302.00492, 2303.05792, 1607.07434, 1509.06963, 0807.0164; fix: each family through the OD #837 S2b gate, then `written: true`);
- More open facts: the archived record of this row.
- State: open (worklist; 97c takes W1, L1, SV1)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "dev20k `aff_missing` census (int125b, 2026-11-01; 451 creato".

### dev20k `name:unmatched` census (int125b, 2026-11-01; 284 creators / 103 papers: 198 / 44 ours, 86 / 59 scorer limits)

- **dev20k `name:unmatched` census (int125b, 2026-11-01; 284 creators / 103 papers: 198 / 44 ours, 86 / 59 scorer limits)** — by creators: **M1** (61 / 10, EASY) a whole-block `tabular`/`minipage`/`\parbox` author block with e-mail lines stays ONE creator, mostly silent: the re-part cannot write an e-mail item (`observer/emitter/repart.rs:183-215` `item_source`, declines :449/:522; fix `Made::Email` at `plan.rs:46` + an `apply.rs` arm; witnesses 2605.09461, 2507.07465, 2609.36484, 2603.10436, 2608.20401; repro `\author{\begin{tabular}{c}Ann Able$^{1}$ \quad Bob Baker$^{2}$\\ $^{1}$Univ A \quad $^{2}$Univ B\\ \texttt{ann.able@a.org} \quad \texttt{bob.baker@b.org}\end{tabular}}`; an address no name spells needs a ruling: a nameless creator) — FIXED 100c for the addresses a name spells (OD #958, `Made::Email`: 2507.07465, 2603.10436, 2608.20401, 2609.39183);
- More open facts: the archived record of this row.
- State: open (worklist)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "dev20k `name:unmatched` census (int125b, 2026-11-01; 284 cre".

### dev lost-words census (int126a-pre, 2026-11-01; dev20k + devh10k, 171 flagged papers: 101 real, 69 scorer artefacts, 1 deferred)

- **dev lost-words census (int126a-pre, 2026-11-01; dev20k + devh10k, 171 flagged papers: 101 real, 69 scorer artefacts, 1 deferred)** — every paper loses exactly the same words on int125b, 97b-rel and int126a-pre (97c-97f neither fixed nor broke one).
- → the 2026-11-01 ruling: unclaimed printed text at the block as a note with a counted Warning (FIDELITY_CHARTER §4), plus API routing for the large families;
- **F** smaller (daj, quantum e-mails, 1605.04229 deferred as pathological, 2603.24729 `$^{\textrm{…}}$` lines, INTERSPEECH 2103.17142).
- More open facts: the archived record of this row.
- State: open (98b; A needs the net)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "dev lost-words census (int126a-pre, 2026-11-01; dev20k + dev".

### the unclaimed-text net (int127a; FIDELITY_CHARTER §4) — the residual

- **the unclaimed-text net (int127a; FIDELITY_CHARTER §4) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): deferred cccconf 1702.07637 and the 1-2-word losses below the threshold;
- stores at their class default (tADR2e 1509.08973, cup-journal 2302.11911) no candidates, the placeholder ruling pending.
- Open: R, routing a class carrying authblk's code without the package through the authblk binding (copernicus 0712.1312, esapub2005 1401.4706: the net's notes today);
- N3, note lines split only at direct-child breaks (2607.26558);
- left: a `\email` the class sends to an unprinted `\@thanks` (1304.5354, astro-ph0609644, astro-ph0612749).
- State: OPEN (residual)

### 99b: a raw class's title-page affiliation or address (OD #955; int127a, 100a) — the residual

- **99b: a raw class's title-page affiliation or address (OD #955; int127a, 100a) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): RED `run_in_lines_a_run_places_keep_their_order`;
- the "Deceased" ruling and rev99b2 F2 (its item (f), int127a's next round).
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "99b: a raw class's title-page affiliation or address (OD #95".

### 100a: hand-printed letter notes and unmarked note lines after a marked list (99a's Cut)

- **100a: hand-printed letter notes and unmarked note lines after a marked list (99a's Cut)** — OPEN, RED.
- Tentative ruling (main session 2026-11-02): a hand-printed `x)` some name's mark requests is that line's entry label, read by the Cut's entry reader (marks win);
- Deferred from 100a r2: only `a)` leads a line — `b)`…`g)` stand mid-line (source newlines), so the reading is of mid-line hand-printed letter labels, a new reader beside the Cut's line-leading one, in `author_block/affiliations.rs` (batch 100b's file).
- More open facts: the archived record of this row.
- State: OPEN (RED)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "100a: hand-printed letter notes and unmarked note lines afte".

### 100c: two Link fixes the observer read right and its writer declined (census127 clusters 6 and 3; prep `agent_reports/2026-11-02_prep99d_link.md`)

- **100c: two Link fixes the observer read right and its writer declined (census127 clusters 6 and 3; prep `agent_reports/2026-11-02_prep99d_link.md`)** — FIXED: (6) a line of marked notes only after a marked block's names line and its `\\` (`\\ \thanks{$^{1}$…}`, also `\\[3.0ex]`, `\\*[len]`, a spacer `\\`) joins the names line, its notes the names' (`author_block/preprocess.rs` `notes_line_joins_names`, `notes.rs` `notes_only_line`; IEEEtran's `grade_only_line` now "one grade, then `notes_only_line`"; OD #957, KNOWN_PERL_ERRORS #1206 — Perl shares it; witnesses 2609.17106, 2411.05212, 2312.10807).
- More open facts: the archived record of this row.
- State: open (3B, cells, exposed, Kind, S2, q10, S3, 6', 3C; held FIXED 101b)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "100c: two Link fixes the observer read right and its writer ".

### Z14's REDs and follow-ups (revZ14r2, SHIP)

- **Z14's REDs and follow-ups (revZ14r2, SHIP)** — OPEN.
- repro `sectioning-frontmatter/z14_affiliation_label_word_left_red.tex`.
- (2) **The narrowed heading row needs a person cue on every piece** (`labels.rs` `later_line_is_names`): `Cat Cole$^1$\\ Authors: Ann Able and Bob Baker\\ $^1$Univ A` and `Editor: A. Able, Bob Baker` read one creator with the row as an affiliation, as int143b;
- repro `z14_heading_row_plain_names_red.tex`.
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Z14's REDs and follow-ups (revZ14r2, SHIP)".

### Z14r3's REDs (revZ14r3, SHIP; the reading's repairs had their two fix rounds, so these go to the next reading batch)

- **Z14r3's REDs (revZ14r3, SHIP; the reading's repairs had their two fix rounds, so these go to the next reading batch)** — OPEN.
- repro `sectioning-frontmatter/z14r3_address_glued_to_a_surname_red.tex`.
- repros `plain_address_glued_by_a_dash_red.tex`, `plain_address_glued_by_a_comma_red.tex`.
- repro `z14r3_same_line_text_under_two_marks_red.tex`, with the OPEN `entry_address_owners` row (2403.12766).
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Z14r3's REDs (revZ14r3, SHIP; the reading's repairs had thei".

### Z14: the reading's repairs before the flip (int144b) — the residual

- **Z14: the reading's repairs before the flip (int144b) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): left by creators on dev3: 2305.13191 -10 (flexisym's digits are `\mathchar`s), 1509.04564 -4, 0807.4720, 1404.5693;
- RED `a_mark_past_a_marked_lists_leading_run_is_an_event_red` (2411.03172, 2510.06608, 2602.11758);
- the role heading's REDs.
- Also left at -2 each: 1901.10843, 2311.16860 (`Alexander New`: a surname read as an institution word), 2409.13798, 2510.01302, cs0011034.
- Also 2003.02512: the reading has its four authors, the writer declines, today's merged creator and its honest Error kept.
- State: OPEN (residual)

### Z4a: the emitter behind its switch (int144b; OD #980) — the residual

- **Z4a: the emitter behind its switch (int144b; OD #980) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): left for Z17: an address in an entry's run with no separator (astro-ph0610446, cond-mat0610708), a legend behind a printed label (2107.01943);
- a conditional a cut ends inside;
- a document note's own API calls (0807.4720).
- State: OPEN (residual)

### Z4b: `=2` reaches `=1`, the safety net (int145b; OD #980) — the residual

- **Z4b: `=2` reaches `=1`, the safety net (int145b; OD #980) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): left before the flip: a block past the positions or the read budget scores as Perl's (dev3 -513 creators, devh10k -607 and -7).
- State: OPEN (residual)

### Z3: spans from the digested list, the rehearsal's isolation (int143b; OD #978) — the residual

- **Z3: spans from the digested list, the rehearsal's isolation (int143b; OD #978) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): RED `writer_second_digestion_says_an_error_twice_red`;
- `#[ignore]`d REDs of `perfect_kernel_batchz3`: `affiliation_cut_environments_by_spelling_red`, `affiliation_cut_group_siblings_stay_uncut_red`, `rehearsal_no_lua_loses_a_cut_red`.
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Z3: spans from the digested list, the rehearsal's isolation ".

### Z2: event gaps 3, 4, 5, 7, 10 (int143b) — the residual

- **Z2: event gaps 3, 4, 5, 7, 10 (int143b) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): REDs with no guard `ordinal_in_a_group_or_with_a_font_keeps_its_letters`, `ordinal_names_on_two_lines_are_two_names`, `event_a_par_by_another_name_is_a_par_break`, `event_declaration_a_kernel_macro_building_a_name`, `event_declaration_through_ifthenelse`;
- REDs with no code: a package's macro defined before `\AtBeginDocument` unread, an unbalanced Declaration span.
- RED, no guard: `ieee_author_after_maketitle_prints_nothing` (`\author{…}\maketitle\author{…}`: the PDF and Perl print the first, ours both).
- State: OPEN (residual)

### Z1: event gaps 1, 2, 6, 8, 9 — the residual

- **Z1: event gaps 1, 2, 6, 8, 9 — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): for Z17: most held papers are the observer's own errors (dev3 120 held errors in 32 papers; about 630 API calls in 353 papers);
- a block's AASTeX 7 `\email` still a printing Contact (gap 6);
- a raw class's font switch read as a declaration.
- The study's held-error papers (21 `Script _ can only appear in math mode`, the rehearsal digesting the API's private options in text mode): 2603.28168, 1405.3507, 2201.04449, 2309.13984.
- State: OPEN (residual)

### A native answer for l3fp's doors, `pk/P2` (user 2026-11-06, as P1)

- **A native answer for l3fp's doors, `pk/P2` (user 2026-11-06, as P1)** — `latexml_engine/src/l3fp_native/`: `\__fp_parse_operand:Nw` at the doors' shape answers with what its run leaves, from a port of l3fp's arithmetic, or runs as defined (a flag, a tuple, `nan`, `rand`, a conditional in the argument, a tab mark that ends a cell, any l3fp name changed since the install, an expression past 400 levels);
- State: [`performance/PERFORMANCE.md`](performance/PERFORMANCE.md)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "A native answer for l3fp's doors, `pk/P2` (user 2026-11-06, ".

### l3kernel/source3: the XML varies with the streaming yield (found by P2's comparison, not caused by it)

- **l3kernel/source3: the XML varies with the streaming yield (found by P2's comparison, not caused by it)** — a big document's yields fall by RSS (`stomach.rs` soft-RSS branch), so 7, 8 or 9 segments are staged from run to run under load, and the output moves with them: a paragraph's closing newline (`types of functions.\n</p>`) and a `\,`'s `rpadding` (`1.5pt` or `1.7pt`, `1\,\text{in}`, l3fp.dtx:1570).
- Base P1 alone, 8 runs under load: 2 hashes (3 and 5);
- P2 8 runs: 1 of those 2;
- idle, one hash.
- Text and errors equal | open, not a regression | a yield must not change the output: what crosses the boundary (a font's size, a trailing space)
- State: [`perfect_kernel/MANUAL_REGRESSIONS.md`](perfect_kernel/MANUAL_REGRESSIONS.md)

### Interpreter hot paths, `pk/P1` (user 2026-11-06: "dramatic speedups for pgf-interference and other pgf work, spanning the entire latexml-oxide")

- **Interpreter hot paths, `pk/P1` (user 2026-11-06: "dramatic speedups for pgf-interference and other pgf work, spanning the entire latexml-oxide")** — byte-identical on the benchmarks and on 1,873 arXiv papers: fp1000 -22.1 %, fill1000 -19.1 %, addplot500 -22.9 %, loading pgf -24.5 %.
- pgf-interference-en/-de are still marginal: 171 s and 181.7 s on loaded SMT siblings, so the native l3fp evaluator (`pk/P2`) is what closes them.
- Each lever, its figures and the measured dead ends: `performance/PERFORMANCE.md` 2026-11-06 | levers in, gates pending | a native l3fp evaluator (`agent_reports/2026-11-06_brief_P2_l3fp.md`);
- `-Zplt=yes` to measure on idle cores
- State: [`performance/PERFORMANCE.md`](performance/PERFORMANCE.md)

### R1: the title-page check's cost, the last regression of #151 (user 2026-11-06; OD #953 "R1"; `performance/PERFORMANCE.md` "the pk/P1 levers")

- **R1: the title-page check's cost, the last regression of #151 (user 2026-11-06; OD #953 "R1"; `performance/PERFORMANCE.md` "the pk/P1 levers")** — DONE: no run of the check's reads more than 8 M tokens (`gullet::read_budget`), what it leaves out named in a counted `Warning:frontmatter:check_budget`;
- RED `check_store_argument_loop_times_out_red` green (`perfect_kernel_batchr1::a_looping_stored_argument_is_budgeted`).
- SASnRdisplay's `Warning:unexpected:\end{document}`, a `\ifx` left open since between #149 and P1, not root-caused
- State: [`performance/PERFORMANCE.md`](performance/PERFORMANCE.md)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "R1: the title-page check's cost, the last regression of #151".

### A second `\author` in article keeps the first call's contact

- **A second `\author` in article keeps the first call's contact** (Z2's review) — `\author{Ann$^1$ \\ $^1$Univ A\thanks{First note.}}\author{Bob}`: the PDF has Bob alone;
- ours Bob with the affiliation Univ A and the pubnote First note, on the base too.
- Synthetic;
- no arXiv witness.
- RED `sectioning-frontmatter/article_second_author_call_keeps_a_block_of_the_first_red`, no guard | RED | Before the frontmatter's single engine reads it (plan Z-series) or on a witness
- State: `tools/perfect_kernel/repros/sectioning-frontmatter/`

### Z19 list setters (DEL3, OD #982)

- **Z19 list setters (DEL3, OD #982)** — the S2b writer's last `=2` path closed: a block calling a binding's list setter (`\institute` of sv_support, inst_support, aa, llncs inside `\author`) is the emitter's — the setter's call written as it is after the items, its entries the list the names' marks request (`emitter/emit.rs` `list_setter_runs`, `without_list_setters`);
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Z19 list setters (DEL3, OD #982)".

### Bundle int153a — THE FLIP: `LATEXML_FM_EMIT=2` (the single frontmatter engine) is the default; `=0` keeps the old driver until the deletion pass (FLIP1; user 2026-11-09 …

- **Bundle int153a — THE FLIP: `LATEXML_FM_EMIT=2` (the single frontmatter engine) is the default; `=0` keeps the old driver until the deletion pass (FLIP1; user 2026-11-09 "i accept the flip"). The no-harm pass (OD #981): NE1 across-cell decline, NE2/NE2b/NE2c/NE2d counter-moved and counter-stepping-mark declines, NE3 `&` only with digested cells, NE4 annotations from the call's tokens, NE5 `declaration_reads_past` + `conditional_cut` (the Fatals 2505.04436, 1511.09009), NE6 a mark's span takes its arguments + `reads_past` over optional/verbatim parameters (2603.11001), NE7 a copied picture keeps its SVG ids (Perl `appendClone`; tikz-shields-doc jing 8→0), NE8 an IJCAI `\emails` section ends the reading + symmetric `conditional_cut`, NE9 the IJCAI flag only for a block holding its markers + an address opening with a call ends the affiliation before it; CL6-CL10 the content no-harm fixes; PF9/PF9b the `=2` checks proportional to what they inspect; reviews revX4-revX7 SHIP; code `e537b16abf`)** — gates 8,311 of 8,311 (4 skipped), cortex 10 of 10, fmt, clippy and doc 0, dumps plain 0 / latex 0 errors.
- **A/B** 28,354, int152c (off) vs int153m (`=2`): 5,397 changed (frontmatter 4,254), E 46,080 → 45,800 (116 better, 1 worse: cs0607047 1→2, its `\def\\KL` erring at each `\\`, faithful), F 172 → 171 (none worse);
- 1710.01722 a timeout equal alone (base, `=2` and off all 180 s under load);
- the large word drops are deduplication (0 distinct words lost: hep-ex0107028, nucl-ex0002003, 1001.4730, 1204.5689, 2406.08304, 1811.08969, hep-ex0106079, 2109.09437);
- 2606.14105 (BESIII, 770 creators off) written whole since int153d ("unstamped: out of positions", counted).
- **The `=2` stage** (1,434): 4 creators changed vs int152c at `=2`;
- 0 newly erroring at `=2` vs off;
- the keep-better net's 6 "equal off" the same as int153d's (1307.5879 ruling, 1409.0847 deferred, the rest scorer-wrong);
- nextest off 8,311 of 8,311.
- **The 37 `=2`-only Error papers**: 0 still newly erroring (bundle dumps).
- **Cost** vs int152c: `=2` −1.10 % to +0.42 % (2103.04797), off −0.12 % to +0.36 %.
- **Nets** 136, 1 worse: ejpecp (3 of 30, the class's queued e-mail notes lost at `=2`, counted — RED).
- **Sweep #167**: G1-G4 0 (G2 1→0 by NE7), G5 58.
- **Re-measure rd8** (int153m, `=2`, per creator / per paper): devh10k 98.60/98.05;
- dev3_10k 99.08/98.73;
- dev20k 98.64/98.32;
- longtail3k 94.21/93.75 vs off 93.05/91.81;
- full suite on the pushed tree (`631dfd6c86`, the 77p pin and docs after the chain's gate tree) 8,311 of 8,311.
- **flip200, judged once** (hand labels, `=2` vs off): MAIN per creator 81.64 % vs 75.72 %, per paper 69.21 vs 68.83 %, fully right 124 vs 122 of 200;
- STRICT 76.89 vs 71.05 %;
- extra creators 54 vs 60.
- **REDs at the flip** (SYNC "Before the flip" item 39, `FRONTMATTER_CONTENT_RESIDUAL_AT_EMIT2.tsv`): the content residual (248 rows incl. 2406.07496's Correspondence line and the three ejpecp nets);
- NE5's decline writing six A/B blocks whole (0809.5152, 0906.2789, 1110.6375, 1404.5602, 2404.18592, 2503.21719);
- the IJCAI `@` cut;
- the textless TikZ drop;
- 77p's ruled grouping (Dan Dorn at UBC alone, user 2026-11-09).
- The push gate's cargo-deny met RUSTSEC-2026-0459 (bundled SQLite in `libsqlite3-sys` 0.38.2, published the day of the flip): `latexml_post` takes `rusqlite` 0.41.1 (`libsqlite3-sys` 0.39.1), full suite 8,311 of 8,311.
- Method notes: probes compare on the bundle's own dumps;
- a writer touching shared frontmatter code runs the whole guards binary.

### Bundle int139c (Bundle 5: 107a-107g) — the residual

- **Bundle int139c (Bundle 5: 107a-107g) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): scorer false losses left unfixed: 1808.00588, 2410.15681.
- State: OPEN (residual)

### Bundle 5's plan (landed as int139c) — the residual

- **Bundle 5's plan (landed as int139c) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): side RED `email_list_unspelled_entries_legend_becomes_creator_red`.
- State: OPEN (residual)

### 100b: one Kind for the addresses inside affiliation entries, one Link chain (OD #956; int129a) — the residual

- **100b: one Kind for the addresses inside affiliation entries, one Link chain (OD #956; int129a) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): HARD, deferred: an address inside a line of other words (about 151 dev20k papers);
- 2009.04283 pathological (DEFERRED_FAMILIES);
- TENTATIVE ruling E and the only-carrier step (2403.17706, 2411.17073);
- RED an `@` in `\href`'s URL read as a printed font;
- More open facts: the archived record of this row.
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "100b: one Kind for the addresses inside affiliation entries,".

### Cortex sandbox reruns on `3e4819bfee` (user 2026-11-02; runs 368 sandbox-arxiv-2605, 369 sandbox-arxiv-2606)

- **Cortex sandbox reruns on `3e4819bfee` (user 2026-11-02; runs 368 sandbox-arxiv-2605, 369 sandbox-arxiv-2606)** — error-free coverage did not regress: 2605 95.47 → 96.88 % (E 1,332 → 923, F 29 → 16), 2606 95.64 → 96.98 % (E 1,298 → 897, F 27 → 22), baselines runs 354/355 (2026-10-15, `1a95291ef6`) snapshotted first.
- State: OPEN (triage)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Cortex sandbox reruns on `3e4819bfee` (user 2026-11-02; runs".

### Author blocks routed through user macros — example collection (user 2026-11-02: no approach decided; collect examples, data and tests first)

- **Author blocks routed through user macros — example collection (user 2026-11-02: no approach decided; collect examples, data and tests first)** — name formatter wrapping a nested `tabular`: 2609.37429 `\FormatAuthor` (two calls in one cell, printed side by side; 100c declines it to base);
- mark macros: 2304.13998, 2602.23952 `\affmark[..]`, `\sa{1}` (RED `raw_class_affil_marks_by_macro_not_split`);
- affiliation macros: nucl-th0303060, 2405.10528, 1307.5171 `\affiliation{\LPC}`.
- Add witnesses as they appear.
- State: COLLECTING

### 101b: an address line carrying a note (2605.23553) — the residual

- **101b: an address line carrying a note (2605.23553) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): RED: an address line's notes are appended after the block's calls (`dispatch.rs:1844`, `affiliations.rs:200`).
- Repro `address_line_note_order_after_a_later_note`.
- State: OPEN (residual)

### 102a: a copy of authblk's `\affil` no longer replaces the binding's (OD #959) — the residual

- **102a: a copy of authblk's `\affil` no longer replaces the binding's (OD #959) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): open variant (iii), an ACL/NAACL `\email` built from authblk's `\affil` code (2406.11632, 1904.03969; RED `authblk_email_from_affil_code`), and variant (ii);
- RED a `\let`-alias wrapper loses its typography (`authblk_affil_copy/wrapper_keeps_its_font`).
- RESIDUAL (the binding's): 2604.08855's `\author[2\ensuremath\mathparagraph]{…}` asks for no `¶` label, so `\affil[$\mathparagraph$]{Corresponding author}` goes to a nameless creator (OD #159).
- State: OPEN (residual)

### 103b: name tails inside `personname` — the residual

- **103b: name tails inside `personname` — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): RED (S) `\samethanks`/`\footnotemark[n]` keeps a mark-only note;
- left: a remark inside the name's own group (2406.15355, 1206.2546), an iD printed as digits after a name's comma;
- RED an icon glued to a bare comma (OD #960 "Open").
- REDs: (S) `author_samethanks_mark_is_the_same_note`;
- the bare-comma icon `author_orcid_after_bare_comma_is_the_names`;
- the leading-icon test reading only the first name `author_orcid_leading_a_later_name_is_its_own`.
- State: OPEN (residual)

### 103b residual: a collaboration heading over an `\and` list covers only the first name

- **103b residual: a collaboration heading over an `\and` list covers only the first name** — `\author{GRAVITY+ Collaboration\thanks{…}: R.~Abuter\inst{8} \and …}` (aa, 2206.00684, the 98a witness), `Euclid Collaboration: M.~Walmsley\orcid{…}\thanks{…}\inst{…} \and …` (aa 2503.15310, aaEC 2507.12116): the binding split reads the heading as the remark of the first piece (`split_author_calls`, OD #946), so only the first name carries `contact role="collaboration"` (2206.00684: 1 of 115);
- More open facts: the archived record of this row.
- State: OPEN (review rev103b S3, record only)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "103b residual: a collaboration heading over an `\and` list c".

### 103c: a legend line read as an affiliation is its carriers' note (census129 cluster 6 / mechanism 4, Bundle 1c; OD #961)

- **103c: a legend line read as an affiliation is its carriers' note (census129 cluster 6 / mechanism 4, Bundle 1c; OD #961)** — FIXED: at the frontmatter flush, after placement, the Link step and the observer's writing (`insert::kind_legend_contacts`), a creator's affiliation contact whose whole text is legends — correspondence, a contribution, a death — names and addresses, no institution (`notes::entry_legend_kind`: pieces parted at marks, brackets, `,:;` and line breaks), is renamed in place to `<note role="thanks" class="ltx_note_frontmatter ltx_thanks_<kind>">`, carriers and e-mails unchanged, its `mark` one that prints — the printed superscript, else a label key a printed mark gave (`affiliation:K`, not an entry's place: `_seqlabel`), a carrier's name shows or requests, or its binding prints before the line (`_labelprints`: authblk, jheppub/jcappub, cas), the carrier's own among several;
- More open facts: the archived record of this row.
- State: open
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "103c: a legend line read as an affiliation is its carriers' ".

### 104b: a raw class's `\and` institute store (OD #791, #955) — the residual

- **104b: a raw class's `\and` institute store (OD #791, #955) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): OPEN, RED catalog repros, one paper each: comsis2 2207.09249, caosp 1401.6818 (`displaced_numbered_institutes_also_printed_after_the_bibliography`), a numbered store whose last entry carries the names' legend lines.
- RED `raw_class_numbered_institutes_with_legend_lines` (2207.09249).
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "104b: a raw class's `\and` institute store (OD #791, #955) —".

### int131fix: the Bundle 1 A/B's regressions — the residual

- **int131fix: the Bundle 1 A/B's regressions — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): left: a recovered Fatal in a field leaves `place_keeper` (`\date{\deep}`), 2110.07418;
- RED `author_orcid_link_wrapped_comma_parts_no_names`.
- State: OPEN (residual)

### Token-stack RAII (int131fix follow-up)

- **Token-stack RAII (int131fix follow-up)** — `invoke_token` (stomach.rs) pushes `token_stack` and pops it on its normal exits only;
- State: OPEN
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Token-stack RAII (int131fix follow-up)".

### `\string` of the null control sequence (int131fix follow-up)

- **`\string` of the null control sequence (int131fix follow-up)** — tex.web §263 prints `\csname\endcsname` (with `\escapechar`) for the null CS;
- ours prints `\` (`\edef\x{\expandafter\string\csname\endcsname}\show\x` gives `\x=macro:->\`).
- `define_optional_internal` names its internal as TeX's `\string` does by its own test (int131fix, OD #567);
- every other `\string`/`\meaning`/`\detokenize` of the null CS is still the divergence.
- State: OPEN

### 104a: IEEE address blocks — the residual

- **104a: IEEE address blocks — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): REDs `ieee_shared_block/email_line_marked_addresses`, `ieee_shared_block/explicit_empty_block`, `ieee_shared_block/mixed_grid_column_major`;
- not this row: 0907.0548, 2402.00243.
- State: OPEN (residual)

### 104c: marked runs the Cut left whole (OD #964) — the residual

- **104c: marked runs the Cut left whole (OD #964) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): RED `revtex_nucleus_marks_spaced_in_one_line_red`;
- open (rev104c3): a nucleus mark after "and", an isotope inside a line;
- deferred: an unmarked line before the marked ones (2205.14113).
- Also open, readings before 104c (recorded, no guard): a tabular of marked entries then `\\ Joint Lab` gives "Joint Lab" to the last entry only;
- More open facts: the archived record of this row.
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "104c: marked runs the Cut left whole (OD #964) — the residua".

### 104d: link lists — the residual

- **104d: link lists — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): residual 2008.13733 (a correspondent of several names), L3's other witnesses (2607.00453);
- RED L4, a `|` between addresses (1507.04180), L5, ijcai `\emails` with a TeX group (`ijcai_emails_group_list_is_spelled`, 2403.12370), P1, aastex631's `\correspondingauthor` placement;
- OPEN P2, cas's `{augroup}` undefined.
- P1's RED `aastex631_correspondingauthor_after_authors_email_is_the_named_authors`.
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "104d: link lists — the residual".

### 105a: the net's lines below its floor (OD #953) — the residual

- **105a: the net's lines below its floor (OD #953) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): RED: class-default placeholders no candidates (2302.11911, 1509.08973; the placeholder ruling);
- code the net does not walk (cccconf 2408.14758, 2206.02389, 2512.09276, 1702.07637 deferred; basm 1601.01922).
- REDs `net_class_default_placeholder_is_counted_red`, `net_non_english_month_line_red`;
- More open facts: the archived record of this row.
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "105a: the net's lines below its floor (OD #953) — the residu".

### 106b: IEEEtran's `\IEEEauthorrefmark{n}`, URL and grade lines — the residual

- **106b: IEEEtran's `\IEEEauthorrefmark{n}`, URL and grade lines — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): a late revtex `\author{A}\affiliation{X}` after `\maketitle`;
- Also open (one-line inputs): amsart `\author{Name\\ \today}` dateline as an affiliation (1910.12289);
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "106b: IEEEtran's `\IEEEauthorrefmark{n}`, URL and grade line".

### 106a: an address line kept as affiliation text (OD #966) — the residual

- **106a: an address line kept as affiliation text (OD #966) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): open: 2410.09681's template line, a line break inside a font wrapper (2311.04121), a phrase-led `\altaffiliation`;
- RED (rev106a2) the older route places by order.
- REDs: `addr_line_authblk_two_carriers_line_marks_red` (the older route), `addr_line_authblk_thanks_requested_mark_red` (a `\thanks`-requested mark unread by `author_mark_keys`);
- left: an address inside a legend note (2312.10429, 2501.04244).
- More open facts: the archived record of this row.
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "106a: an address line kept as affiliation text (OD #966) — t".

### 106s: the title-page check (OD #953) — the residual

- **106s: the title-page check (OD #953) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): RED: a raw class printing its title page from `\begin{document}` (cup-journal.cls:1013-1014, 2302.11911);
- REDs (rev106s2) `\let\thanks\@gobble` a false alarm, `\let\thanks\footnote` (talk.cls:413) a silent loss;
- a `\thanks` in the third of several still recurses;
- a class word equal to a document word counted the document's;
- the placeholder ruling.
- Those REDs: `check_class_let_thanks_gobble_red`, `check_class_let_thanks_footnote_red`, `check_shipped_class_text_is_not_the_authors_red`, `check_document_sty_macro_body_counts_red`.
- The sibling: opendatalab's `\let\oldmaketitle\maketitle` (2601.09733).
- State: OPEN (residual)

### 106c: a tail inside the personname (OD #967) — the residual

- **106c: a tail inside the personname (OD #967) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): left: `\samethanks`/`\footnotemark[n]` keeps a mark-only note (2110.07431, 2402.06922);
- RED `grade_given_back_per_creator_red`, `grade_macro_names_and_membership_one_block_red` and kin.
- The kin: `grade_macro_block_beside_written_grade_red`;
- a lone acronym a collaboration contact, `collab_for_lone_acronym_is_no_group_red`;
- the `\samethanks` repro `author_samethanks_mark_is_the_same_note`.
- State: OPEN (residual)

### 106t: the title-page check's second source (OD #953) — the residual

- **106t: the title-page check's second source (OD #953) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): open: quantumarticle's `\ead` (KPE #1208), ceurart's `\orcidauthor`;
- RED (rev106t2 S1) a binding selector's undeclared dropped branch;
- That RED: `check_binding_selector_untaken_branch_unwanted` (fancyhdr's `\iffloatpage`).
- dmtcs `\publicationdata`'s dates counted, the content fix open;
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "106t: the title-page check's second source (OD #953) — the r".

### 107a: an address spells its author by transliteration — the residual

- **107a: an address spells its author by transliteration — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): left: a list after a depth-0 break inside one font wrapper (2408.05517, 2609.38967, 1805.08616).
- RED `email_list_unspelled_entries_legend_becomes_creator_red` unchanged.
- State: OPEN (residual)

### 107b: a names list not parted (OD #897 "107b") — the residual

- **107b: a names list not parted (OD #897 "107b") — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): `\mbox{S.~Burke-Spolaor,$^{12,13}$}` keeps the comma (1809.10744);
- a `\shortstack` and the tabular/minipage blocks stay one author;
- REDs `author_row_of_name_shaped_institutions_red`, `author_space_tie_between_two_names_red`.
- RED `author_sub_lettered_mark_shown_on_name_red` (1202.2048).
- Also 1505.05811's empty-mark `${}^{\mathsf{}}$` continuation lines go to the block creator, no word lost (F4).
- State: OPEN (residual)

### 107c: an address, legend or URL line kinded affiliation (OD #966, #961, #969) — the residual

- **107c: an address, legend or URL line kinded affiliation (OD #966, #961, #969) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): open: a run a role tag follows (2503.14354, 2210.14319).
- Also open (one line each): a template line `Emails: name.surname@polito.it, …` (2401.15236);
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "107c: an address, legend or URL line kinded affiliation (OD ".

### 107d: the title-page check where no `\maketitle` of ours runs (OD #953) — the residual

- **107d: the title-page check where no `\maketitle` of ours runs (OD #953) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): REDs `check_title_page_relaxed_maketitle_no_false_warning_red` (S1), a sibling-directory `\input` read as a package's (S2).
- Open, one dev3 paper: nj_style's `\ead` written to the `.aux`, printed on pdflatex's second run (1401.7920).
- State: OPEN (residual)

### 107e: a marked run cut whatever parts it (OD #954) — the residual

- **107e: a marked run cut whatever parts it (OD #954) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): open: 2605.25195's legend (the writer declines its URL line's owner);
- REDs `cut_run_wrapped_entry_then_comma_address_red`, `cut_run_group_entry_then_comma_address_red`.
- Also `cut_run_legend_after_wrapped_entry_ending_in_a_group_red`, `cut_run_nested_wrapper_per_entry_red`.
- Also open: the kernel alone welds a legend line into the entry above it after any run (base the same);
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "107e: a marked run cut whatever parts it (OD #954) — the res".

### 107f: e-mail ownership (OD #970, #972, #966) — the residual

- **107f: e-mail ownership (OD #970, #972, #966) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): open: 2511.18493's `minhnnhat@` goes to Anh-Minh Phan;
- RED `addr_line_sole_author_symbol_series_orphan_is_blocks_red` (`mark_series`' symbol branch does not fire).
- Also open: a note holding a list for its carriers is not parted;
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "107f: e-mail ownership (OD #970, #972, #966) — the residual".

### 107g: the Bundle 5 fix round (OD #970, #966, #897) — the residual

- **107g: the Bundle 5 fix round (OD #970, #966, #897) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): REDs `email_list_mark_line_names_non_carriers_no_spelling_red`, `email_list_mark_line_spelled_out_of_place_no_order_red`, `email_own_call_printed_under_both_stays_on_both_red` (2502.15891, 2307.00704, 2405.01799), `author_block_repeated_affiliations_land_on_the_last_red` (2307.07324, 2001.07542).
- More open facts: the archived record of this row.
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "107g: the Bundle 5 fix round (OD #970, #966, #897) — the res".

### 108a: authors merged under authblk (2610.06709; OD #52(h)) — the residual

- **108a: authors merged under authblk (2610.06709; OD #52(h)) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): left: `\footnote` under a sole authblk author (2601.20363, 2605.13069; RED `authblk_footnote_line_under_sole_author_red`);
- 2505.05785 (RED `email_spelled_part_of_a_hyphen_name_red`);
- REDs `authblk_institution_line_before_affil_red`, `authblk_legend_swallows_contact_lines_red`, `authblk_title_suffix_one_name_red`, `authblk_small_hspace_in_name_red`.
- More open facts: the archived record of this row.
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "108a: authors merged under authblk (2610.06709; OD #52(h)) —".

### 108b: the silent lost words counted (OD #953) — the residual

- **108b: the silent lost words counted (OD #953) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): open RED catalog `check_raw_setter_input_own_file`, `check_store_read_only_in_first_page_foot`;
- a glued `\textbf{Lunar}\includegraphics{logo}` fails the source gate.
- State: OPEN (residual)

### 108b replay gate: a discarded print need not obey it

- **108b replay gate: a discarded print need not obey it** — OPEN.
- `sect05.rs` "not replayed: it uses …, which is undefined here" refuses a class's `\maketitle` for the kept replay and for the check's print;
- 61 of 10,000 dev3 logs (~17,000 arXiv papers).
- Letting the discarded print run ignore the gate recovers the print (untested: 2408.05088, `check_raw_setter_replay_refused`).
- Counted meanwhile by `unclaimed_store`

### 108b content: keep what mechanism A only counts

- **108b content: keep what mechanism A only counts** — OPEN, a ruling.
- Content losses against Perl (it keeps them, with Errors, apparently by not reading the raw class): 2305.00867 (Perl E=14), 1011.2770 (E=58), 2605.02209 (E=11), repro `check_raw_setter_store_gated` (E=0).
- Rust's run is clean and loses them, now counted, not kept

### 108b `\newcommand`: a class's `\newcommand{\@maketitle}` ignored under the kernel's empty one

- **108b `\newcommand`: a class's `\newcommand{\@maketitle}` ignored under the kernel's empty one** — OPEN, found here.
- cjpsuppl.cls:199 (hep-ph0402063): "Ignoring redefinition (\newcommand) of `\@maketitle`", no `\lx@dropped@` record, so the class's title page (`\@authori`, `\@addressi`) never runs: the authors lost;
- the words reach only `Info:frontmatter:unread_store`.
- In pdflatex the class loads no article and the name is free.
- Minimal: a class with no `\LoadClass` and `\newcommand{\@maketitle}{Lunar}`

### 97a review should-fixes (rev97a, SHIP, 2026-10-31)

- **97a review should-fixes (rev97a, SHIP, 2026-10-31)** — (S1, synthetic; FIXED 97b, OD #944: a piece that is an address and no name is no marked name, GREEN `perfect_kernel_batch97b::marked_address_piece_is_no_person`) a marked e-mail piece is made a person: `names_split.rs` `marked_names` counts any piece `reads_as_affiliation` declines as a marked name, and `frontmatter/classify/marks.rs:8-11` declines every text holding `@`, so `\author{A.~Able$^{1}$, B.~in der Baker$^{2}$, b.baker@two.edu$^{*}$}` (revtex4-2) gives three creators, the third `<personname>b.baker@two.edu<sup>*</sup></personname>` (95b r3: one creator);
- More open facts: the archived record of this row.
- State: open
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "97a review should-fixes (rev97a, SHIP, 2026-10-31)".

### 97b: a take-out removes what it read and nothing else TeX prints

- **97b: a take-out removes what it read and nothing else TeX prints** (OD #944; the int125b A/B's `fm_checks.py` losses against int123a) — FIXED: aa's `\mail` labelled as the shipped copy prints it (a copy naming `\mailname`, v4.03-v7.0, keeps "Correspondence to "; a storing copy, v8 on, none; `\mailname`'s `\it` grouped, no italic on later items);
- Residuals, each a one-line input: (a) a shape-split names line keeps the binding's `before` — `\documentclass{revtex4}…\author{Ann Able, Bob Baker and Cat Cole}` before="  " (int123a the same; "and" printed nowhere);
- More open facts: the archived record of this row.
- State: open
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "97b: a take-out removes what it read and nothing else TeX pr".

### 97b review should-fixes (rev97b, SHIP, 2026-11-01)

- **97b review should-fixes (rev97b, SHIP, 2026-11-01)** — (S1, synthetic) the store-label reader knows which arguments print only for the commands its table names (`frontmatter_stores.rs` `argument_of`, `PRINTED_ARGUMENTS`), so a printed label is still lost through a raw file's own wrapper (`\newcommand{\mymailto}[1]{\href{mailto:#1}{#1}}`, `Correspondence: \mymailto{\@corremail}`), `\hyperlink{corr}{\@corremail}` or `{\color{black}\@corremail}` (int125b the same);
- More open facts: the archived record of this row.
- State: open
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "97b review should-fixes (rev97b, SHIP, 2026-11-01)".

### 97c: an affiliation the reading already has reaches its author

- **97c: an affiliation the reading already has reaches its author** (OD #945; dev20k's largest archival shape `own:aff_missing`, 364 creators / 119 papers) — FIXED: (W1) a names block's mark-led entry of several events is cut by its whole span (resolver/blocks.rs `labelled_entries`; `$^{1}$Univ of Science and Technology~~~~$^{3}$Univ C` was Ann's whole: 2111.07950, 1503.06795, 2504.05692, 2008.02300, 2404.01856; W1b 2212.01433, 2608.06741), a digit mark glued inside a word only when the source glues it (events/lexer.rs `glued_in_source`: `NLP$^2$CT` the word's, `Russia\break $^2$Nuclear` an entry, hep-ph9809222);
- More open facts: the archived record of this row.
- State: open
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "97c: an affiliation the reading already has reaches its auth".

### 97d: a collaboration named in the author line is a contact (OD #946) — the residual

- **97d: a collaboration named in the author line is a contact (OD #946) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): left, one-line inputs: (a) aa's own `\and` split;
- (g) RED `author_collaboration_name_with_and_stays_one`;
- (j)'s RED `author_team_piece_mark_labels_its_own_line`.
- Also left: (a) 2512.08803 (`XYZ Collaboration: A.~Able\inst{1}` one name);
- (h) `\affiliation{(WA98 Collaboration)}` an affiliation (0708.2630), awaiting a ruling;
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "97d: a collaboration named in the author line is a contact (".

### 97f: giant lists in a size environment; AASTeX 6.x's shared affiliations — the residual

- **97f: giant lists in a size environment; AASTeX 6.x's shared affiliations — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): left: (a) sub-labelled marks (RED `author_sub_labelled_marks_read_as_digits`, ruling deferred);
- RED `aa_collaboration_heading_before_colon_is_a_name`;
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "97f: giant lists in a size environment; AASTeX 6.x's shared ".

### 99a: one Cut (OD #954) — the residual

- **99a: one Cut (OD #954) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): an unrequested entry once at the block (hep-ph9805510, 2608.11300);
- IEEEtran's symbol-labelled lines (2308.05546);
- REDs `cut_acmart_uncut_call_after_a_cut_list_red`, `cut_acmart_closing_part_composes_unprinted_text_red`;
- N-b a conjunction dropped between entries;
- 2009.04283;
- 2204.02412.
- Pathological, deferred: REDs `cut_acmart_template_address_line_red`, `inblock_glued_repeated_label_is_entry_text_red`;
- int127a's RESIDUAL: a cut entry shows no label once its name's mark is hidden as answered (`hide_answered_marks`; 2406.07529, 2207.02356, 2309.15702, 2405.17878).
- State: OPEN (residual)

### 98b: four lost-words mechanisms (OD #949-#952) — the residual

- **98b: four lost-words mechanisms (OD #949-#952) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): left: pathological 2008.12867 (RED `svjour3_institute_environment_before_at_is_lost`).
- Also left: an affiliation-only `\IEEEauthorblockA{X, Y\\ …}` parted at its first line's commas (words kept);
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "98b: four lost-words mechanisms (OD #949-#952) — the residua".

### field stamps overflow on a large block (2410.11607) — the residual

- **field stamps overflow on a large block (2410.11607) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): 1810.03464 is a rehearsal over its read budget (vldb-modify.cls's `\normalsize`; the leftovers row's item 6).
- State: OPEN (residual)

### R2 binding-API data: ijcai `\emails`, cas `\address[label]` (88b; OD #908, KPE #1200) — the residual

- **R2 binding-API data: ijcai `\emails`, cas `\address[label]` (88b; OD #908, KPE #1200) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): residuals with options in OD #908: marked groups kept whole on the block (2105.02711), `wulk@` unspelled (2308.08563), a group in markup on one author (2501.02356).
- Also: the one leftover address to the corresponding author (2304.11989 `scs_lzg@`, 2505.12894's plural legend; a user-ruling step, kept; two narrower refinements for the user);
- ijcai's one unmarked `\affiliations` line on the first author only (Perl's `labelseq=author`).
- State: OPEN (residual)

### amsart: the frontmatter setters after `\maketitle` are still the binding's (pre-existing on every binary; 92a review rev92a_r2 P2)

- **amsart: the frontmatter setters after `\maketitle` are still the binding's (pre-existing on every binary; 92a review rev92a_r2 P2)**: amsart.cls:621-629 `\@cleartopmattertags` lets `\maketitle`, `\title`, `\author`, `\address`, `\email` and the other setters `\relax` after the first `\maketitle`, so `\title{Old}\maketitle Text. \title{New}\author{Ann Able}\address{University X, Town}\email{ann@x.org}\maketitle More.` prints "Text.
- New Ann Able University X, Town ann@x.org More." as body text under the title "OLD".
- We print "New Ann Able" in the body and put the address and the e-mail on a nameless creator;
- Perl prints the same body text and loses them.
- RED catalog repro `class-bindings/amsart_setters_after_maketitle_print_as_text`;
- not 92a's mechanism, so no guard
- State: OPEN

### FM S2b: ams_support written (81a-81f, 82a-82e; OD #837 "S2b next family") — the residual

- **FM S2b: ams_support written (81a-81f, 82a-82e; OD #837 "S2b next family") — the residual** (landed: 82b `written: true`, 82e; its Bundle A/B and nets superseded by the later bundles; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): 1610.06222 open (marks in an address's own names line).
- State: OPEN (residual)

### FM S2b: IEEEtran written (95b, OD #942) — the residual

- **FM S2b: IEEEtran written (95b, OD #942) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): left 2602.04239 (an ORCID line kept as an affiliation), 2603.10979 (affiliations in `\thanks` legends), 2311.14729.
- State: OPEN (residual)

### FM S2b: acmart written (95b, OD #942) — the residual

- **FM S2b: acmart written (95b, OD #942) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): left 2010.12245, 2607.06101.
- State: OPEN (residual)

### FM S2b: authblk written (95b, OD #942) — the residual

- **FM S2b: authblk written (95b, OD #942) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): left 2609.33086, 1208.5147, 2603.11001.
- State: OPEN (residual)

### 97e: every remaining observer-writer family written (OD #948) — the residual

- **97e: every remaining observer-writer family written (OD #948) — the residual** (landed; the row verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md)): left 1509.06963 (jcappub, declined), 2009.04166.
- Also: the svjour/elsart collaboration headings wait on OD #946's contact reaching the per-author bindings' `\and` pieces;
- a re-parted entry's `\textit` mark kept in its text where the source typed `//` (1306.1933).
- State: OPEN (residual)

### 97e r2 residuals (review rev97e; OD #948)

- **97e r2 residuals (review rev97e; OD #948)** — (N1) 1108.2557 (ws-ijmpd, inst_support): the e-mail line printed under the addresses is read into entry b, so all three addresses go to Geng (97c: on no one) — the reading's line cut;
- More open facts: the archived record of this row.
- State: OPEN
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "97e r2 residuals (review rev97e; OD #948)".

### int126a review (rev126a; OD #947, #948)

- **int126a review (rev126a; OD #947, #948)** — B1 and B3 FIXED in 98a (a collaboration heading a printed separator parts from the first name is an entry, 2206.00684; a control word in an `\altaffilmark` list is no request label, astro-ph0105133; guards `perfect_kernel_batch98a`).
- Open for a ruling: the heading `XYZ Collaboration\thanks{…}: Ann Able` is written as a creator of its own (Perl's and the old reader's `\and` entry, 97b's output), not OD #946's collaboration contact — svjour's, llncs's and aa's `\and` pieces never reach the author block's collaboration reader (row 97d (a));
- More open facts: the archived record of this row.
- State: OPEN
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "int126a review (rev126a; OD #947, #948)".

### authblk: a label holding a command before its number

- **authblk: a label holding a command before its number** (`\author[$\dagger$1]{Ann Able}`, pdflatex "Ann Able†1") — `labels::parted_label` reads a source-spelled label to its first command (2405.16051), so the label is nothing: Ann Able holds no affiliation, entry 1 and the `$\dagger$` entry stay in the block (int124b alike; `$^{\dagger}$1` alike);
- `\author[1$\dagger$]` keeps `1` from 95b on, the dagger's entry in the block.
- Synthetic (review rev95b);
- repro class-bindings/`authblk_label_opening_with_a_command` (RED, no guard).
- Options: read each label piece a command prints as a mark (the `\affil[$\dagger$]` label's own reading)
- State: OPEN

### The take-out reads a replaced `\author` call's block items

- **The take-out reads a replaced `\author` call's block items** (`Resolver::take_out_entry_addresses`, post.rs ~1664): the creators' prints are the live creators' (`self.live()`), the block's every item, a replaced call's (IEEEtran's anonymous-review `\author{…}` before the real one, 2602.12633's shape) included — an address in such a call's block entry would be taken out and owned.
- Output identical to int124b on the census;
- no repro (a replaced call holding a block e-mail is not cheap to build).
- Fix: skip the block items whose source is a replaced call
- State: OPEN (review rev95b)

### authblk residuals read on the 95b census (output identical to int124b)

- **authblk residuals read on the 95b census (output identical to int124b)**: 2106.15893 — `\author[$^{1,2}$]{…}`, `\author[$^{\star,1}$]{…}`, `\affil[$^1$]{…}`: a label spelled as a formula's script is not read, every entry in the block;
- 1411.2438 — one `\affil[ ]{…}` shared by all three authors sits on the last only;
- 2008.01011 — `\affil[$\S$]{…}` stays in the block.
- Each a one-line input of its source's shape
- State: OPEN (review rev95b)

### Writer cosmetics on the 95b census (review rev95b N2)

- **Writer cosmetics on the 95b census (review rev95b N2)**: 2507.03771 — authblk `\affil[*]{Corresponding authors: \color{blue}yujiewang@sjtu.edu.cn\color{black}, …}`: the take-out leaves "Corresponding authors", its colon trimmed with the cut addresses (pdflatex prints "Corresponding authors: …");
- 2603.23991 — IEEEtran `Qiong Wu$^{1,2,}$$^\ast{}$`: the name keeps "1,2," as the source prints it, its `∗` hidden as answered by the legend now hers ("∗Corresponding author is Qiong Wu.").
- Neither is a separator-trim one-liner of the 95c F6 rule: the first needs the cut to keep a label's own colon, the second is the source's comma
- State: OPEN

### Unobserved frontmatter calls under a written author block (95b r3 residual, OD #942)

- **Unobserved frontmatter calls under a written author block (95b r3 residual, OD #942)** — the writer tells an unwritten family's call apart only when a watched setter's expansion made it (`ledger::FRONTMATTER_SETTERS`);
- More open facts: the archived record of this row.
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Unobserved frontmatter calls under a written author block (9".

### svjour3 `\institute`: the names before `\at` are no entry (pre-existing; 95b r3)

- **svjour3 `\institute`: the names before `\at` are no entry (pre-existing; 95b r3)** — svjour3.cls:506 `\def\at{\\\let\and=\iand}` prints the names before `\at` over the institute they share;
- State: OPEN (the reading; the writer declines)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "svjour3 `\institute`: the names before `\at` are no entry (p".

### 95b r3 review should-fixes (rev95b-r3 SHIP, 2026-10-31; none a regression or a loss)

- **95b r3 review should-fixes (rev95b-r3 SHIP, 2026-10-31; none a regression or a loss)** — (S1, moot since DEL5 deleted the setter watch) a setter's input mark can stay open after its expansion ends: `state.rs` `input_unread_since` (~:2436-2449) compares the mouth's depth and read position but not which mouth, and token-list digestion runs on `Mouth::default()` (position (0,0,0,0), mouth.rs:164-175), so a mark taken at the end of a digested list reads open in a later default-mouth context;
- More open facts: the archived record of this row.
- State: OPEN (residual)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "95b r3 review should-fixes (rev95b-r3 SHIP, 2026-10-31; none".

### AMS contact setters under raw amsart derivatives (next batch; 86c r1, reverted)

- **AMS contact setters under raw amsart derivatives (next batch; 86c r1, reverted)**: a raw class read over the AMS binding re-dresses `\address`, `\curraddr`, `\email`, `\urladdr` as stores only its own `\@maketitle` prints (stacs.cls:294-298, stacs_proc.cls, rmmcart.cls; m2an.cls, edpsmath.cls, pcms-l.cls:481 `\csname @address\number\addresscount@`), and our deposit of that `\@maketitle` is dropped, so the addresses are printed nowhere (0902.1737, 0802.2833).
- Repros (no guard): `sectioning-frontmatter/ams_raw_class_address_setters_stored` (RED) and the six above;
- More open facts: the archived record of this row.
- State: open
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "AMS contact setters under raw amsart derivatives (next batch".

### Frontmatter e-mail ↔ name spelling

- Frontmatter e-mail ↔ name spelling: an address that spells an author with an umlaut written out (`ü` as `ue`, `duerrschnabel@` for Dürrschnabel, 1911.11496) is not matched, so it goes unspelled (to the block).
- The spelling helper is shared by today's output and the FM S2 shadow.
- Fix: fold the German umlaut transliterations (ä→ae, ö→oe, ü→ue, ß→ss) into the comparison when the plain fold fails.
- FM S2a.2 r6 decision 3 sample
- State: open

### Graphics search order

- Graphics search order: LaTeX's `\IfFileExists` tries the bare name in the working directory first (kpathsea, latex.ltx:1095-1102) and only then the `\input@path`/`\graphicspath` entries (`\@iffileonpath`, latex.ltx:9698), where `image_candidates` (`util/image.rs:36-41`, Perl Image.pm:46-48 alike) lists `\graphicspath` entries first and SOURCEDIRECTORY last: with `\graphicspath{{../up/}{./}}` and a `u.pdf` in both, pdflatex reads `./u.pdf`, ours `../up/u.pdf` (for the extensionless name and the `.eps`-retry alike).
- Older than 78v (int63a the same).
- Also `graphics_type_rank` ranks `.MPS` among the uppercase types, which pdftex.def:599 does not list (no practical effect).
- Review rev78v3 findings 2 and 4 (probe `rev78v3/g/sub/g3.tex`)
- State: open

### float.sty's `\restylefloat{<type>}` of a kernel float (figure, table) records the style only (float_sty.rs `record_float_style`), so without caption the caption keeps …

- float.sty's `\restylefloat{<type>}` of a kernel float (figure, table) records the style only (float_sty.rs `record_float_style`), so without caption the caption keeps the format it had: a ruled figure "Fig. 1: text" under IEEEtran (kernel ": " under article) where float.sty's `\floatc@ruled` prints "Fig. 1 text" (review rev78y probe3/rs.tex);
- State: OPEN (pre-existing; found at 78y r2)
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "float.sty's `\restylefloat{<type>}` of a kernel float (figur".

### acmart copies older than TL 2025's v2.16 (v1.55-era, review rev78y acmcls/cs12.cls) set sigchi's small caption fonts on figures only, not tables; acmart_cls.rs ports the …

- acmart copies older than TL 2025's v2.16 (v1.55-era, review rev78y acmcls/cs12.cls) set sigchi's small caption fonts on figures only, not tables;
- acmart_cls.rs ports the TL version's.
- Fonts only, so behind the class caption fonts knob (caption_sty.rs `CLASS_CAPTION_FONTS`, off by default): no output difference unless the knob is set.
- 78y r2
- State: OPEN (knob-only; per-version port not planned)

### A preload's options do not reach a package another preload already loaded, in either order, so until 78z the class caption fonts knob could not be combined with ar5iv.sty

- A preload's options do not reach a package another preload already loaded, in either order, so until 78z the class caption fonts knob could not be combined with ar5iv.sty: `--preload=[classcaptionfonts]latexml.sty --preload=ar5iv.sty` sets the knob but loses every option ar5iv.sty passes to latexml.sty (`ids`, `rawstyles`, `rawclasses`, `magnify`/`zoomout`, the limits — `\documentclass{paper}` falls to OmniBus with `Warning:missing_file` where ar5iv alone reads paper.cls raw; the magnify PIs and `<p>`/autoref ids gone), and `--preload=ar5iv.sty --preload=[classcaptionfonts]latexml.sty` sets no knob (no `\ds@classcaptionfonts` runs).
- State: the class caption fonts knob FIXED 78z (guards `perfect_kernel_batch78z::{ar5iv_classcaptionfonts_option_sets_the_knob, ar5iv_classcaptionfonts_option_keeps_ar5iv_options}`); the general repeat-preload behaviour OPEN
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "A preload's options do not reach a package another preload a".

### Determinism (det354 B2)

- Determinism (det354 B2): the `xmlns:*` declarations on the root `<document>` are in the order of the model's namespace table (`common/model.rs:1244-1254`, a `SymHashMap` at `:55`, → `document.rs:774-830`), not the order the document first uses them, as Perl declares them (Document.pm:1393-1394,1874): `\usepackage{fontawesome,tikz}` then `\faTimes … \tikz\draw (0,0)--(1,1);` declares `xmlns:svg` before `xmlns:aria`, though `aria` is used first (trigger det354 `repro/ns.tex`; Perl declares `svg` only).
- State: open
- Landed history of this row: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Determinism (det354 B2)".

### appendix.sty

- appendix.sty: `\appendix` inside `{appendices}` does not restart the letters (pdflatex "A" again for the next section, ours "B": the kernel's `\@appendix` is `\relax` there; 2112.09636);
- pinned as ours by `perfect_kernel_batch79d::cleveref_appendix_inside_appendices_names_appendix` (review rev79d2 finding 8)
- State: OPEN (pre-existing)

### cleveref's `\appendix` type under TeX Live's LaTeX first aid

- cleveref's `\appendix` type under TeX Live's LaTeX first aid: TL 2025's first aid (`\firstaid@cref@updatelabeldata`) drops cleveref's appendix aliasing, so TL 2025 pdflatex prints "section" after `\appendix` where first-aid-off pdflatex prints "appendix" (the oracle: the witnesses refer by `\ref`, so their PDFs never print the type; 2503.13751, pdfTeX-1.40.25 / TL 2023, only dates the kernel);
- 79d follows the latter (main-session decision 2).
- Decide by the build's TeX Live year if arXiv's builds move to a first-aid kernel (79d r2 writer's question 2)
- State: open, low

### Two adjacent `\vec`s (and `\mathbf`s) merge into one bold identifier by the kernel's `\mathbf` merge rule

- Two adjacent `\vec`s (and `\mathbf`s) merge into one bold identifier by the kernel's `\mathbf` merge rule: 0812.2480 `e^{-i\vec{k}\vec{r}}` → one `<XMTok font="bold">kr</XMTok>`, 2305.06325 `\vec{g}\vec{z}` → "gz", `\vec{z}\vec{p}` → "zp";
- `$\mathbf{k}\mathbf{r}$` gives "kr" in base (probe `rev79b/kr/`).
- Unlike the accepted `\vec{kr}` ruling, two vectors written separately lose their implied product;
- renders the same, a parser-semantics question (review rev79b (`agent_reports/extracted_356a0c47/2026-10-23T024112_a8b3383dff3c35201.md`, probes `~/.claude/jobs/356a0c47/tmp/rev79b/`), finding 3)
- State: OPEN, needs a ruling; math parsing parked (no change)

### euler.sty:47-57 and fourier.sty:77-87/104-114 declare the capital Greek letters `\mathord` (class 0) from their upright fonts, so in TeX `\mathbf{\Gamma}` keeps the …

- euler.sty:47-57 and fourier.sty:77-87/104-114 declare the capital Greek letters `\mathord` (class 0) from their upright fonts, so in TeX `\mathbf{\Gamma}` keeps the medium-weight Euler/`futm` letter (and `\mathit{\Gamma}` stays upright; mathptmx's, mathpazo's and eulervm's `\mathord` `\up…` the same);
- ours follows the kernel's `\mathalpha` rule (`declare_capital_greek(…, None)`: `\mathbf` bold, `\mathit` italic), as Perl and int75a do — no regression, no fixed-upright `LettersClass` (review rev79e finding 3, probes `~/.claude/jobs/356a0c47/tmp/rev79e/`)
- State: open, low (not a regression)

### mathastext's Greek options (`symbolgreek`, `eulergreek`, `LGRgreek`, `selfGreek`, `itGreek`) are not modelled — its binding is a no-op for the capital Greek letters (79e …

- mathastext's Greek options (`symbolgreek`, `eulergreek`, `LGRgreek`, `selfGreek`, `itGreek`) are not modelled — its binding is a no-op for the capital Greek letters (79e r2 report);
- no census paper combines mathastext with the italic classes;
- a `\mathord` upright letter (mathptmx's `\up…`, amstex, euler) under `\mathit` turns italic here where TeX keeps it upright (no fixed-upright rule; the row above's gap)
- State: open, low

### The kernel's `\DeclareMathSymbol` gives a `\mathalpha` (class 7) symbol role `ID`, where Perl's (latex_constructs.pool.ltxml:2697-2705) gives none

- The kernel's `\DeclareMathSymbol` gives a `\mathalpha` (class 7) symbol role `ID`, where Perl's (latex_constructs.pool.ltxml:2697-2705) gives none: a raw class/package copy's capital Greek letters (and any `\mathalpha` glyph) parse as `ID`, a binding's (`declare_capital_greek`, the kernel's `\Gamma`) as `UNKNOWN`.
- Predates 79e (55h);
- a parser role, math parsing parked (79e r1 writer's question 2, review rev79e decision 2: leave it, track here)
- State: open, math parked

### 25, 26, 27 — `--splitindex`/`--splitbibliography` (non-default), alignment open at `\end{document}`, lstlisting in `\parbox`

- `--splitindex`/`--splitbibliography` (non-default), alignment open at `\end{document}`, lstlisting in `\parbox`
- State:
  - not silent, re-checked on head 2026-10-19: #26 is an honest Error (a `tabular` left open, `c & d`: 4 Errors; an `align`, `c&=d`: 3 — "Attempt to close a group that switched to mode internal_vertical"; invalid input, pdflatex errs);
  - #27 has raised 1 Error since 64zf (CONTROL `kernel-alignment/pcol_lstlisting_parbox_CONTROL`, pdflatex 7);
  - #25 needs a non-default option the arXiv runs never pass — open, low reach, not blocking

### Before the flip: the switch-on leftovers — open items (the row's history, items 1-39, in the archive)

- The whole row (DEL1/DEL2, LO1-LO4, E1, the content check, the readiness re-measures, the flip's gate and no-harm pass) verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Before the flip: the switch-on leftovers".
- Left: (3) **1812.07638's `Contributors:` heading** (a line between two names rows) read as the six editors' affiliation: a role heading is its names row's label, which the API has no kind for yet (options: a note of the block, the row's creators' contact of a heading role, or left as today).
- (3c) an in-block `$^1$Univ (ECT$^*$) and …` splits the entry at the symbol with either switch (a symbol glued inside open brackets in the lexer's `glue_in_word_marks` fixes it, and moves the switch-off output).
- (5) **BESIII's sub-labels** (R6, tentatively ruled): 2204.09614, 2404.06718 leave 48 and 51 words silent (`68A,68C` answered by entry 68).
- (6) **1810.03464's rehearsal loops** past its read budget (vldb's `\normalsize` in `\email{…}`; trigger in its own row).
- (7) **The tabular grade** `…, Fellow, IEEE\end{tabular}` reads `\end{tabular}` into the grade (E=2, off too).
- (9) open part: the switch-off cost +0.18 to +0.22 % `instructions:u` over LO1 on 1507.02068, 2103.04797, 1901.01540 (from LO2) — to bisect (`kit/perf_bisect.sh` over 3205f4ba81..df0926bd9b); the rest of (9), the tests under `=2`, done before the flip.
- **The residual, flipped with as named REDs (user 2026-11-09):** `docs/parity/FRONTMATTER_CONTENT_RESIDUAL_AT_EMIT2.tsv`.
- Open beside it (REDs): the author-field Errors silenced since the flip (the PRIORITY RED: 18 A/B papers, revDEL4b) are raised again by ERR2 (below; OD #983);
- a block whose `\footnote` counter steps inside `\author` (14 A/B papers) goes whole as one creator "Ann Able  and Bob Baker" where Perl's article writes two, so that Warning's "as Perl's whole-block creator" is false there;
- a `\stepcounter` of another counter inside `\author` is lost (0 vs Perl/pdflatex 1; 0 A/B papers; repro `observer_rehearsal_loses_an_author_stepcounter.tex`);
- DEL3's list-setter port (OD #982), synthetic only (2 of 73,251 papers have the shape, neither these cases; probes `scratch_revDEL3b/rp/`): R1 a top-level svjour `\institute` *after* the block takes over — the block's Univ X / Lab Y lost with 2 orphan Warnings, where pdflatex prints the block's list (`\maketitle` sets it last; fix: clear the epoch's affiliation calls up to the flush), R2 an entry of the written setter call no mark asks for is dropped with an orphan Warning (`sv_unref_in`: pdflatex prints "2 Lab Y, City"; base and off keep it on a nameless creator; `sv_typed_sup` the same);
- the ORCID logo of an icon-only `\href` is lost at `=2` (OD #960; int153m `=2` vs off: 2507.12319 4→0 logos, 2406.08258 2→0; pre-existing at the flip, not DEL3);
- the ejpecp e-mail notes the class queues besides its creators are lost at `=2` ("holds no place … (ltx:note); it is lost", counted; binding net ejpecp 3 of 30 worse: 1809.03082, 1310.4722, 2609.22952 — rows in the residual TSV);
- NE5's `declaration_reads_past` writes six A/B papers' blocks whole that the candidate before it split (0809.5152, 0906.2789, 1110.6375, 1404.5602, 2404.18592, 2503.21719; 0 distinct words lost, three also drop a queued `ltx:date`, counted) — narrow it to a definition whose replay actually reads past;
- `resolver/mod.rs` takes every held Error in an IJCAI block as a section marker, not only those at `\affiliations`/`\emails` (revX7, unprobed);
- NE8's IJCAI cut is picked by an `@` in a held section's printed text, not by the split's own delimiter (`resolver/blocks.rs` `address_section`; revX5, synthetic `scratch_revX5/syn/a1.tex`: e-mails inside `\affiliations` with no `\emails` → at `=2` the held `\affiliations` Error off also raises, and "Univ B \affiliations"; fix: cut at the marker whose held token is `\lx@ijcai@affilsplit`'s `Until:\emails` delimiter, read from its definition; no stage2 paper reaches it);
- a frontmatter TikZ picture with no text inside `\href` (`\author{Ann\\ \href{https://e.org}{\tikz\fill (0,0) circle (0.1);}}`, or in `\thanks`) is dropped with no Warning, off and `=2` (NE7; needs a kept contact/note or a counted Warning);
- 1602.03850-shaped names-only macro overlaps written as each name's text;
- two unrecorded calls in one flush;
- 1901.02012's sequenced e-mails counted lost, not owned;
- AIMS's `\email` label printed at the end (1903.05770);
- an `\emph`-wrapped class label leaves an empty wrapper;
- a label's font not carried into `name`;
- `E-mail: \email{…}` with a class `\email` printing "E-mail:" loses one;
- `content::reads_past`'s end-of-run rule (a run ending at a call none of whose arguments it holds) has no guard — no input reaching it was found (a piece a macro prints spans its arguments, `\newcommand\place[1]{Princeton}` + `$^1$Univ \place{x}, $^2$Univ B`; a mark's span is grown by `arguments_end` first), so it stays a backstop (NE8);
- since the flip a `\stepcounter` in an author field is lost: `\author{Ann Able\stepcounter{probe}\ifx\foo\undefined\gdef\foo{ONE}\else\gdef\foo{TWO}\fi \and Bob}` with `Value \foo{} and \theprobe.` in the body gives "Value ONE and 0." at `=2` (0 errors, 0 warnings: the `\gdef` kept, the step lost), where Perl, pdflatex and off print "1" (revDEL4 P1; repro sectioning-frontmatter/`observer_rehearsal_loses_an_author_stepcounter`, pinned as today's in `perfect_kernel_batchfms2a1::the_rehearsal_leaves_nothing_behind`);
- an author field's undefined commands, whose Errors the rehearsal held and discarded, are raised once since ERR2 (OD #983): in place where a piece holds them (after a name, past its marks and notes; an address whose local part an undefined command glues a word's ink onto — ink of the same word on both sides of it before the `@`, a Name the ink table cannot align failing closed on its span — is never read as one, any other read as without the Error, so `\author{Ann Able\undefinedprobecs \and Bob\email{b@x.org}}` gives Perl's 2 Errors and creators Ann Able and `Bob<ERROR>\email</ERROR>b@x.org`), else by the flush's net (a piece written as text);
- the flip A/B's 19 papers: every lost `Error:undefined` back except the artefacts off raised and pdflatex does not (kr.sty's `\affiliations` in 2405.06973, 2602.18351, 2604.26688; 2405.13308's `\@thefnmark`/`\@undefined` from a digestion that lost both names), creators never fewer than DEL5's, 0 words lost, stage-2 1,434 unchanged;
- the `\fi` chains of 1603.08231 (18) and 2112.06251 (12) are Perl's split at `\and` (KNOWN_PERL_ERRORS #1218; pdflatex 0), correctly absent.
- ERR2's REDs left by its last review (revERR2c, SHIP): one undefined command after `\footnotemark` writes two `<ERROR>` stubs while logging its one Error (`\author{Ann Able\footnotemark\foo{} \and Bob Baker}`; Perl and pdflatex one; ERR2-introduced, 0 of the 358 sample papers; suspected `emit.rs` `block_names` taking the held command into `apart` while the note piece grows over it by `content.rs` `past_held_undefined`) — the next batch;
- the span fallback's `@` test is end-exclusive (`b\foo@x.org` keeps the invented `Baker1b@x.org`, `b\foo{}@x.org` declines);
- pre-existing on the base: a superscript mark glued before an address is read into it (`Bob Baker\textsuperscript{1}b@x.org` → e-mail `Baker1b@x.org`), and `\author{Ann Able, ann@x.org\footnotemark \and Bob Baker}` gives one creator (TeX 2, Perl 3; no word lost).
- Reviews: revNE1, revNE2, revNE2r2, revX1, revX2, revCL9b, revCL10 (and the fix rounds' re-reviews).
- Reports: `agent_reports/2026-11-07_{F1,F1r2,revF1,revF1r2,N1,revN1,LO1,revLO1,N2,revN2,N2r2,LO2,revLO2,LO2r2,revLO2r2,tests_sort}_report.md`, `2026-11-09_{NE1,NE2,NE1w,NE2w,NE2b,NE2c,NE3,CL5,CL6,CL7,CL8,CL8b,CL9,CL9b,CL9c,CL10,revNE1,revNE2,revNE2r2,revX1,revX2,revCL9b,revCL10}_report.md`, `2026-11-08_{T1,revT1,N2r3,revN2r3,LO3,revLO3,LO4,revLO4,RD1,P2,revP2,P2r2,revP2r2,RC1,revRC1,revG1,revG2,revG3,revBC1,revG5,revH1,revH2,revH3,revI1,revI2,revI3,revI4,revI5,revI6,revI7,revI8,cw78,ts9}_report.md`

### Open residuals lifted from landed rows (2026-11-09)

Each landed row is verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1); these are its open facts.

- **Bundle int147b (LO2 + LO2r2, the second switch-on leftovers; T1, the tests at `=2` part 1; N2r2 + N2r3, the content check's fix rounds; each reviewed SHIP (revT1 NO-SHIP …**:
  - Readiness (`=2` vs off on LO2r2-rel, scored; dev20k pending): dev3 per creator 98.89 / 98.88 %, per paper 98.51 / 98.57 %;
- **Bundle int133a (98d + 105a; code `9a716bc81b`)**:
  - the 9 frontmatter changes all 105a gains (e-mail contacts 1305.0854, 2001.07747; PACS codes 1311.3004; Chinese address lines 2206.01007, 2601.14329) but 2108.12618, whose kept template note ("Instructions for Creating an FnT Article") gains its "2018" (placeholder ruling pending).
- **98d: `\end{X}` over a `\begingroup` opened inside X (kernel; SYNC "The unclaimed-text net" R0; OD #965, KNOWN_PERL_ERRORS #1207)**:
  - More in the archived row.

## Rust Error Fixes

Rust-only defects (a port that diverged from the Perl it translates, not a Perl-origin defect), each with its
KNOWN_PERL_ERRORS cross-reference when one applies. Perl-origin defects go to `parity/KNOWN_PERL_ERRORS.md`.

Each entry's body (defect, cause, fix, guards, witnesses) is verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "From \"Rust Error Fixes\"". Titles here:

- **An `\xymatrix` cell opening with `\xy` lost the wrapper's conditional (82f)**
- **A display's `$$` deferred past an author's open group (82g)**
- **`\newcommand{\cs[n]}{…}` (82h)**
- **`\pdftexbanner` empty (82i)**
- **A package by its installed path missed its binding (82j)**
- **An xargs optional-argument command expanded outside typesetting (82c)**
- **A LaTeX picture inside PiCTeX kept PiCTeX's `\put` (82d)**
- **hyperref's autoref tag ran the document's `\null` (81b)**
- **A creator's fuzzy name label leaked a mark's comma (80v)**
- **The FM writer left a lone suffixed id (80u)**
- **cleveref `\crefname`/`\Crefname` derivation (77s)**
- **An unnumbered heading's title font (78a)**
- **Text merged into a text node is ligated (78a)**
- **`\halign` inserts `\everycr`; AMS-TeX's active `@` (78b)**
- **thmtools' unnamed theorems (78k)**
- **subfig's `\newsubfloat` was a no-op (78s)**
- **A class copy with classic Mac line ends was one comment (79b)**
- **lamstex arrowheads print as letters**
- **A pgfkeys key that expands to one brace group (79w)**
- **A package's own `\zap@space` around a load (79x)**
- **An argument a template's cell brace closes (79y)**
- **ctable's options after a line end (80d)**
- **caption's `\captionbox` over a class's box (80i)**
- **nicematrix's `\tabularnote[label]` (80i)**
- **A binding's guessed theorem refused the document's `\newcommand` (80k)**
- **The conditional guard's Fatal kept nothing (80k)**
- **hyperref's `\Hy@driver` was `hpdftex` whatever the output mode (101a)**
- **A recovered Fatal left the Stomach's token stack full (int131fix)**
- **The null control sequence's optional-argument internal was `\\` (int131fix)**

## Current status

The corpus measure is the perfect-kernel scoreboard ([`PERFECT_KERNEL.md`](PERFECT_KERNEL.md), sweeps of the 2,374
TL manuals) and the per-batch arXiv A/B; the cortex reruns of sandboxes 2605/2606 (runs 325-328, 2026-10-04) and run 329's validations are in
`archive/SYNC_SESSIONS_2026-10.md`; the last full sandbox rerun before them (2026-08-02, 60,505 docs) in
`archive/SYNC_SESSIONS_2026-09.md`.

---

## Open items — detail for the ranked rows

### R3 — Bibliography-Absence Campaign (PR #444 Residuals)
- **R3d — RE-MEASURED + CLOSED int26 (agent_reports/2026-10-18_r3d_remeasure.md):** of the 28 papers
  (`docs/parity/bib_absence_2026-07-29/residual_characterization.tsv.gz`, `\lx@begin@alignment` rows) 27 convert to the
  end on head with their bibliographies; the last, 2606.03447, was physics' unguarded `[…]` read (KNOWN_PERL_ERRORS
  #981), fixed in int26 (`phys_read_optional`). The mechanism below was wrong: TeX's `macro_call` does not disable tab
  marks (that is `read_toks`, tex.web §482); ltcmd's `\group_align_safe_begin:` (latex.ltx:2103-2110) does, so a global
  guard in `Parameters::read_arguments` would be unfaithful (settled dead end; the 5 regressions were correct).
- **(historical) R3d: Alignment Parameter Scan vs Cell Read Distinction (`suppressed_tab_marks`):**
  - *Symptom:* An unescaped `&` inside a delimiter-fenced macro argument splits the alignment row and truncates the document (and bibliography). 12 papers remain affected.
  - *Mechanism:* `tex.web` §394 `macro_call` suppresses tab marks while scanning parameters. `SuppressedTabMarks` in [`latexml_core/src/common/local_assignments.rs:194`](../latexml_core/src/common/local_assignments.rs#L194) fixed `physics.sty`'s `\mqty` (14 papers), but applying it globally to `Parameters::read_arguments` regressed 5 tests (`cells_test`, `numprints_test`, `xytest_test`, `consort_flowchart_test`, `unit_tests_by_silviu_test`) because that path also reads alignment cell content.
  - *Action:* Distinguish macro parameter scanning from alignment cell reading in `Parameters::read_arguments` so tab marks inside `{...}` do not split outer cells.
- **R3b — RE-MEASURED + CLOSED 2026-10-18 (agent_reports/2026-10-18_r3b_rootcause.md):** the six "real `\cite`, empty
  bibliography" papers all keep their bibliographies on 67int27 — 2605.14990 53, 2606.05629 19, 2606.10056 26,
  2606.17491 30, 2606.00231 31 bibitems, each = pdflatex+bibtex's `.bbl`, 0 E/F (count them in the post-processed
  output: the core XML's `<bibliography files=…>` is a placeholder latexml_post fills from the `.bib`). 2605.29754
  comments out biblatex's `\printbibliography` and types its list by hand (kept); its `\cite`s render the keys with
  `Warning:expected:ids Missing Entry`, as Perl (SHARED, honest; biblatex labels without an `ltx:bibliography` would be
  a 1-paper surpass, not pursued). Side finding, open: tlp.cls `\maketitle` leaves a group open — a Rust-only
  `Warning:unexpected:\end{document} … inside a group at level 1` (2606.10056; content intact; suspect the locked
  `\maketitle` replaying tlp.cls:519-536 `\@ifnextchar [{\m@ketitleone}…` through `\lx@deposit@maketitle`; repro
  scratch_356a0c47/r3b/repro_tlp/a.tex).
- **R3g: amsrefs bare `\begin{biblist}` — FIXED (71a):** a `{biblist}` outside `{bibdiv}` opens its own bibliography
  through the backmatter route (amsrefs_sty.rs; KNOWN_PERL_ERRORS #939).

### R4 — `--preload=<cls>` trips the LaTeX hook stack (`Extra \PopDefaultHookLabel`)

- RESOLVED (re-verified on int101a 2026-10-25); the detail verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "R4 detail".

### R6 — `Collector::rescan` Refactor for Generated Backmatter
- **Symptom:** Generated subtrees (Bibliographies, Indexes, Glossaries) lose ObjectDB relations, labels, and fragids because `Scan` owns `ObjectDB` by value (`latexml_post/src/scan.rs:49`) and cannot rescan generated nodes.
- **Resolution:** Refactor `Scan` to borrow or take/restore `ObjectDB`, enabling clean rescanning of generated nodes and removing fragile ad-hoc registrations.

### R7 — Presentation-MathML F5 Linebreaker
- **Status:** The only remaining pMML gap from the MathML line audit (F17 is closed). A full linebreaker algorithm gap requiring a port-or-drop scope decision before coding.

---

## Secondary residuals & unranked active items

### Seen on 2610.06709, not investigated (carried from the CorTeX session's report, 2026-11-05)
- A `\footnotetext` note ("Senior coauthors") carries a stray dagger above its rule in the wide layout, from `<sup class="ltx_note_mark">†</sup>` on `ltx_role_footnotetext` (`\lx@note` in `sect03.rs`, `LaTeXML-meta-xhtml.xsl:103`: a mark-less note falls back to †). DIVERGENCES line 1740 records the fallback as out of scope.
- The page scrolls sideways at widths of 700px and below (882px of content), the overflow including `figure.ltx_float.ltx_algorithm` and its listing. Markup or CSS: unknown.
- [dginev/ar5iv-css#55](https://github.com/dginev/ar5iv-css/pull/55), open at the time, fixes the wrapped-note stacking by `.ltx_creator`, `.ltx_role_thanks` and their sibling order: keep that structure stable, or update the CSS with it, when note markup changes.

### A `<p>` inside a transformed box's `<span>` (open; seen on 2610.06709, not root-caused)
- `\resizebox`-style display math in an appendix prints `<div class="ltx_inline-block ltx_markedasmath ltx_transformed_outer"><span class="ltx_transformed_inner"><p>…`: 4 of the paper's 4 inner spans (`A6.Ex179.1`, `.Ex180.1`, `.Ex183.1`, `.Ex184.1`, on int139c). A `<p>` is not valid inside a `<span>`, and the outer box is already a `<div>`. Perl's output and the reach are unmeasured. Separate from the frontmatter nesting report (batch 108a), whose authors block is well-formed.

### Open residuals of closed rows (R1, R2, R5; details in `archive/SYNC_SESSIONS_2026-09.md`)
- **R1:** the 117-paper `TooManyErrors:MaxLimit(100)` cluster the seeds came from (2026-08-02 rerun) has not been
  re-measured; its seeds `2605.22927`, `2606.11121`, `2606.01136`, `2605.10685` are clean since 56ks3/56kt. The same
  rerun's `Stomach:Recursion` cluster (55 papers, MIXED) had three REAL-by-count witnesses never followed up:
  `2605.17696` (Rust 144 / Perl 56 errors), `2606.05321` (35 / 15), `2606.08524` (94 / 50).
- **sn-jnl (DIVERGENCES #333):** newer class copies (2404+, e.g. 2605.00003 L1280-1321) define `\toprule`/`\midrule`/
  `\botrule`/`\cmidrule` themselves instead of loading booktabs. CLOSED (verified 77v): since 67z (3a3cbdfd01) the
  binding loads booktabs as the class as distributed does (sn_jnl_cls.rs), so the rules are defined without the paper's
  own `\usepackage{booktabs}`; guard `perfect_kernel_batch77v::sn_jnl_table_rules_without_booktabs`.
- **Equation layout (DIVERGENCES #334):** `eqnarray` after a class's `fleqn` + amsmath is flush left in pdflatex, centred
  here (`ltx_fleqn` is document-wide). `document_class_filename` (content.rs) is overwritten by nested class loads, and
  xkeyval's fallback reads it (`xkeyval_sty.rs:1524`). elsarticle's `\ifpreprint` follows elsarticle.cls:112 (`preprint`
  the default, `5p`/`3p`/`1p`/`final` clear it, :71-87) since 62j (verified on int40c in 78a: "PREPRINT"; guard
  `perfect_kernel_batch61::elsarticle_journal_layout_is_the_jtype_macro`).
- **Cropping (DIVERGENCES #337):** `reflect` is not applied; a negative trim (padding) is clamped, as in Perl; anisotropic
  resolutions crop by the x one; the job key holds the full options, so `[trim=X,clip,width=3cm]` and `[…,width=5cm]`
  crop twice into identical files.

### Font-Selection Chain Residuals
1. **`\cal ABC` collapses to one `<mi>`**: Drops `class="ltx_font_mathcaligraphic"` (Perl emits three `<mi>` elements, Rust one containing `𝒜ℬ𝒞`). Token grouping and class diverge.
2. **`\DeclareTextCommand`/`\ProvideTextCommand` lack encoding-dispatch chain**: `latex_constructs.rs:6525/6544` bind bare `\cs` to first-encoding expansion permanently.
3. **Minor font registration gaps**: `\DeclareTextSymbol` decodes eagerly instead of installing deferred `CharDef`; `\DeclareErrorFont` is a bare no-op where Perl defines argument as `\relax`.
4. ~~A preamble font switch stays in force in the body~~ FIXED 77u (W17, DIVERGENCES #315 addendum): the begin-document constructor runs latex.ltx's `\init@series@setup` (:13902-13920, `\reset@font`, `\mdseries`, `\let\seriesdefault\f@series`) after `\reinstall@nfss@defs`, so a preamble or class-body font switch does not reach the body (guards `perfect_kernel_batch77u::{preamble_font_switch_does_not_reach_the_body, class_body_font_does_not_reach_the_body}`). ~~Residual: `\normalfont` sets `\rmdefault`, not `\familydefault`~~ FIXED 77u r2 (user ruling 2026-10-20, DIVERGENCES #779): `\normalfont`/`\reset@font` are latex.ltx's (:14113-14122, `\encodingdefault`, `\familydefault`, `\seriesdefault`, `\shapedefault`) and the neutral font of notes and tags takes the same defaults, so a `\renewcommand\familydefault{\sfdefault}` document is sans throughout, as its PDF (guards `perfect_kernel_batch77u::{family_default_sets_the_body, family_default_survives_a_preamble_switch}`). Residual: row 6.
5. **`.bib` author names and year are not sans in a sans document** (77u r2/r3): `\renewcommand\familydefault{\sfdefault}` + a `.bib` bibliography — the title and publisher spans are `font="sansserif"` (the session digests in the defaults where `\bibliography` stands, `lx@bibliography@normalfont`), the `ltx_bib_author` and `ltx_bib_year` spans, which the post stage builds, carry no font; pdflatex sets the whole entry in CMSS10. Fixture `latexml_oxide/tests/cluster_regressions/bib_font_defaults_sans_document.tex`; `06_cluster_bibliography::bib_entries_take_the_defaults_where_the_bibliography_stands` asserts the title and publisher only. Open.
6. **`\textnormal` in maths keeps a fixed serif font** (77u r2): `\textnormal@math` (`$\textnormal{x}$` in a `\familydefault{\sfdefault}` document) is serif, pdflatex CMSS10; the text-mode `\textnormal` follows `\familydefault` (OD #779). Open.
7. **A `\familydefault` the font table does not know falls back to serif only in `\normalfont`** (77u r2, kept by the main session): `\lx@normalfont@base` (sect13.rs) puts serif in force before `\normalfont`'s `\selectfont`; elsewhere `\selectfont` keeps the font in force for an unknown family (`{\sffamily\fontfamily{droidserif}\selectfont x}` stays sans). The neutral font of notes and tags (`font_code_named`) reads only a literal or a chain of parameterless macros: opensans.sty's `[default]` (:160-161, `\familydefault` = `\opensansfamily` = `\opensans@family{…}`, "opensans-TLF", not in font.rs's table either) stays serif where pdflatex sets Open Sans. Open.
8. **Math a package makes sans stays serif** (77u r4, review probe sfm; base and r3 alike): sfmath.sty (:329-344,
   `\SetSymbolFont{operators}`, the `SFMath` letters), arev.sty, cmbright.sty (:29-33, the `cmbrm`/`cmbrs` symbol fonts)
   and beamer's default sans math (beamerbasefont.sty:205-222, `\SetSymbolFont{operators}{normal}{OT1}{cmss}…` unless
   `professionalfonts`) set a formula's letters and digits in a sans font (pdflatex `pdffonts` CMSS/HFBR); here the
   letters are `<mi>` italic serif. A formula's token fonts are the
   text default's (OD #780), and none of these bindings declares a sans math version (`\DeclareSymbolFont`/
   `\SetSymbolFont` for `operators`/`letters`). Fix: a binding-level math default (the family of the `operators` and
   `letters` symbol fonts) read where `finalize_rec` resets an `ltx:Math`. Open.
9. **Math glyphs of a raw math-font package's own fonts** (89a residuals; FIXED there: a symbol-font name given by a
   macro, mathdesign — and a kernel or AMS name at its fontmath.ltx/amssymb slot in a font with no map, now the token
   LaTeXML's own binding makes of it, kpfonts, raw newtxsf, eulerpx; newpxmath and newtxmath are bound and untouched;
   OXIDIZED_DESIGN_DIVERGENCES #910, the msam/msbm/cmsy/cmmi map slots KNOWN_PERL_ERRORS #1202). Also open: a raw
   kernel `\mathchardef` LaTeXML keeps prints the low byte of its code (fontmath.ltx:268-269 `\varbigtriangleup`
   `\mathchar"2234` as `4`, `\meaning` `\char"8756`; pdflatex △), and a symbol whose font HAS a map takes the map's
   glyph and the class's role, not the binding's (oz.sty's `U/msa` `\Cap` ⋒ BINOP where kpfonts' is ⋒ ADDOP
   `double-intersection`; a raw OMS `\mid` `|` RELOP), and mdsymbol's `\SetSymbolFont{symbols}{normal}{U}{MdSymbolA}`
   for an already declared font (mdsymbol.sty:383-390) is not modelled: its symbols decode through cmsy's OMS layout
   or print a stand-in (`\Cap` ⊥, `\blacktriangledown` ⊎; 339 of the 402 reference names in a probe), and a binding's
   limits placement is not carried (a redeclared `\sum`/`\bigcup` sets display limits as scripts; no corpus witness;
   RED repro fonts-nfss/redeclared_mathop_takes_display_limits). FIXED 89b (B0, user ruling 2026-10-29): an
   undecodable slot is U+FFFD, never the slot as a wrong ASCII stand-in (`\neq` as `,`, Ω as a newline), and each symbol
   font that loses glyphs says so in one `Warning:missing_font_encoding:<enc>/<family>`, where such a symbol is first
   printed — raised when the document absorbs its box, so a symbol only digested (program.sty's savebox, a
   `\settowidth`, a discarded hold) says nothing (OD #27; guards
   `perfect_kernel_batch89b::*`; 1502.03923, 2402.09138, 2008.13643, 1604.04618, 2307.11186). Open, the text side's
   `\lx@unmapped@text@symbol` (a macro expanding to nothing) still warns at expansion and sets its key there, so a
   text symbol expanded only under a discarded hold or in a box never printed warns or is silenced alike as 89b r1's
   math symbols were (minimal: fonts-nfss/text_symbol_of_an_unmapped_encoding_warns's `XX` preamble, then
   `\sbox0{\textxxa}Hello.` → 1 Warning, pdflatex prints "Hello."); a box of its own would change its expansion. A kernel accent a
   package redeclares at another slot of an undecodable font is U+FFFD over its base since 89b. Before, its token was
   empty and MathML filled it with the accent's `name` (`<mo>hat</mo>`, presentation.rs:1186; Perl MathML.pm:711), and
   an align cell holding only the accent and its base was dropped whole (1902.03183 eq. 9 `\bar{u}`, restored by 89b;
   guard `perfect_kernel_batch89b::undecodable_math_accent_is_fffd_and_warns`): newtxsf's `\hat`/`\bar`/`\tilde`/`\dot` in `lettersA` and `\vec` in `symbols`
   (newtxsf.sty:301-313, :1332), and mathabx's `\widehat`/`\widecheck`. These are 1,093 tokens in 16 papers of pop89a:
   2106.08363, 1808.01384, 2406.11942 and 2607.15091 among them. FIXED 89c (OD #27): a kernel accent (fontmath.ltx's
   thirteen) undecoded in a declared font with no font map is the kernel binding's token for its name, at any slot
   (`reference_accent`, checked by rendering newtxsf, asmeconf, mathabx.dcl copies), and `U/eur` decodes through OML at
   the 111 slots eurm10 shares with cmmi10 (program.sty's `\powersetsymbol` ℘); the 17 it lacks are gaps of the font
   (`U_eur_fontmap_lacks`), where neither a kernel accent nor a kernel symbol decodes by its name (`\vec` "7E, `\star`
   "3F: U+FFFD and the font's Warning, as pdflatex prints nothing), as are msbm's five ("5F, "62-"65: `\Cap` declared
   in `AMSb` at its `AMSa` slot "65; review 89c r2 S1), every mapped `U` family's gaps generated by tftopl
   (`U_FONT_GAPS`, gen_reference_math_slots.py `--u-font-gaps`), while the msam/msbm maps' empty slots that hold a
   glyph with no single character (msbm's negated pairs, msam's `\dabar@`; oz.sty, zed-csp.sty) still decode by name
   (`symbol_font_lacks_slot`); guards `perfect_kernel_batch89c::*`. FIXED 89d
   (OD #27): an over accent LaTeXML's own package bindings define, where every binding defining it makes the same
   token, decodes by its name too (mathabx's `\widecheck` ˇ, 2607.15091 88 tokens; twenty names, generated,
   `\overleftrightarrow` left out as amsmath's and mathabx's bindings disagree, which TeX does not settle: tex.web §738
   `make_math_accent` walks the accent's charlist to the widest successor that fits, so a `\mathaccent` can grow and
   "not stretchy in TeX" is no premise (review 89d r1 open question 2); a name some definition makes other than as
   such an accent is left out too, none today (gen_reference_math_slots.py `binding_definitions_otherwise`); guard
   `perfect_kernel_batch89d::package_accent_decodes_by_its_name`). Open: a letter
   `\DeclareMathSymbol` puts in an upright font is italic (`\DeclareMathSymbol{\myA}{\mathalpha}{operators}{"41}` →
   `font="italic"` A, pdflatex upright; Euler's α alike; deferred: styling, no content lost; RED repro
   fonts-nfss/declare_math_symbol_letter_in_an_upright_font_is_upright); a raw `\mathaccent` in a family with no font
   map is an empty `<XMTok/>` with no Warning, on int120a and 89c alike (newtxsf, `$\mathaccent\numexpr"7000+
   \symlettersA*256+223\relax{x}$` → pdflatex x̂; OD #27 covers only `\Declare*`; deferred; RED repro
   fonts-nfss/raw_mathaccent_in_an_unmapped_family_is_counted); mdsymbol's `\widehat`/`\widetilde` in its
   `largesymbols` (`U/MdSymbolE` "72/"78) decode through cmex's OMX map as ⎷ and ↑, and its `\vec` at "84 — outside
   cmex's slots — is U+FFFD with the `OMX/cmex` Warning (the `\SetSymbolFont` gap above). Open, each a slot
   `\DeclareMathSymbol` declares in a font with no map, now U+FFFD and counted: (B3, HARD, deferred) the layout from the TFM's
   coding scheme: `U/fsy`'s `zplmr7t` says "TEX TEXT" (OT1) and `zplmr7m` "TEX MATH ITALIC" (OML), trustworthy only for
   those two strings — msam10/msbm10/esint10 claim false ones (heppennames2.sty:117-151 via cms-tdr.cls:167;
   2307.11186 147 tokens, 2507.13469; minimal: `\DeclareFontFamily{U}{fsy}{}\DeclareFontShape{U}{fsy}{m}{n}{<->s*[1.0]zplmr7t}{}`
   `\DeclareSymbolFont{ugrfr}{U}{fsy}{m}{n}\DeclareMathSymbol{\Uplambda}{\mathord}{ugrfr}{"03}` → pdflatex Λ, ours U+FFFD);
   package data, a binding row each: esvect's `\vv` (`\fldr`/`\montraita` are arrow fragments, no Unicode; esvect.sty:28-29,
   1502.03923), cmll's `U/cmllr` table (⅋ `\invamp` at 0, cmll.sty:32/87; 2402.09138), kpfonts' `symbolsC` (150
   symbols at txfonts' `symbolsC` slots, no `txsyc` table to share; `\neq` at 44).
10. **A mapped font's gaps are recorded only for the `U` families** (89c r3, review 89c r2 S1 "same pattern"; no
   witness): a reference name declared at a slot whose map entry is empty decodes by its name (#910) unless the slot is
   a recorded gap (`symbol_font_lacks_slot`), and gaps are recorded, from tftopl, only for `U/msa`, `U/msb` and `U/eur`
   (u_fontmap.rs `U_FONT_GAPS`). The same holds for every other encoding map with empty slots at a reference slot:
   OMX "75-"77 and "7A-"7D (cmex10 has glyphs there, the kernel's `\braceld`…`\braceru`, so no gaps for cmex, but an
   OMX font of another family may lack them), TS1's gaps (a TS1 family without the full set), and OML families other
   than cmm/eur (a font lacking cmmi's slots). The general mechanism: read the actual TFM's character info at conversion
   time for any mapped symbol font (any encoding, cm included) — a slot the font in force lacks is a gap, U+FFFD with
   "has no character at slot" — in place of the static table. Deferred: no TeX Live package or corpus paper declares a
   reference name at such a slot (review 89c r2 scan); minimal:
   `\usepackage{amsfonts}\DeclareMathSymbol{\Cap}{\mathbin}{AMSb}{"65}` `$a\Cap b$` (fixed by the table, the shape
   the general mechanism would cover for any font). ~~The generator's `U_FAMILIES` font list is typed by hand~~ FIXED
   89d (review 89c r3 N1): it reads each `u<family>.fd`'s `\DeclareFontShape` fonts through kpsewhich and stops on a
   difference from `U_FAMILIES`, takes the TL year from `tftopl --version`, and stops with a message on a TFM with no
   characters.
11. ~~**bbold's map is encoding-wide, so it decodes every unmapped `U` family**~~ FIXED 89d except the bbm/bbmss
   alias (row 12) (review 89c r3 P1; Perl the
   same, bbold.sty.ltxml:30, KNOWN_PERL_ERRORS #1203; OXIDIZED_DESIGN_DIVERGENCES #914): `bbold_sty.rs` registered
   `U_fontmap`, so beside `\usepackage{bbold}` every `U` font without a map of its own (stmaryrd, mathabx copies,
   newtxsf; and `U/msa`/`U/msb`/`U/eur`, whose u_fontmap.rs never loaded) decoded through the blackboard map, silently
   (`\widehat` 𝕡, 0 Warnings). The map is now bbold's own families' (`U_blackboard`, `U_bbold`), read by `decode`,
   `decode_string`, math characters (`decode_in_font`) and `\usefont{U}{bbold}` without a `U` map; guard
   `perfect_kernel_batch89d::bbold_map_is_its_own_familys`. No other binding registers an encoding map under a shared
   encoding.
12. **bbold's map reaches bbm's and bbmss's fonts through the shared `blackboard` alias** (review 89d r1 S1; the same
   on 89c): font.rs's family table (font.rs:184-186) names `bbm`, `bbold` and `bbmss` alike `blackboard`, the family
   bbold_sty.rs's `U_blackboard` map is keyed to, so beside bbold `{\usefont{U}{bbm}{m}{n}R\char"0B\char"00}` is
   ℝαℾ, where bbm10 (58 characters) has nothing at "0B or "00 and pdflatex prints ℝ alone (four "Missing character"
   lines for bbm10 and bbmss10); without bbold it is RﬀΓ (OT1, row 13). Fix: key the map to the NFSS family before
   font.rs normalises it, and give bbm/bbmss their own tables (tftopl) or their gaps. RED repro
   fonts-nfss/bbold_map_reaches_bbm_through_the_blackboard_alias. Open, a later batch.
13. **Text in a `U` font without a map decodes through OT1, silently** (review 89d r1 open question 1; #645's
   `\lx@fontencoding` fall-back): 2003.08144's fourier-orns `\leafNE` (`{\FourierOrns G}`, `U/futs` "47) prints `G`,
   pdflatex fourier-orns' leaf; minimal `\usepackage{fourier-orns}` `A\leafNE B` → AGB, 0 Warnings. Direction (the
   reviewer's and 89b's math rule, user ruling 2026-10-29): U+FFFD and one Warning per font, unless a table gives the
   glyph. It touches every text `U` font and the `\cf@encoding` dispatch, so its own batch with an A/B. RED repro
   fonts-nfss/text_in_an_unmapped_u_font_is_not_ot1.
14. **The AMS fonts are an encoding to font.rs, so a family change does not leave them and NFSS reports the alias**
   (review 89d r1 P1, P2; the same on 89c): font.rs maps `msa`/`msb` to the encodings `AMSa`/`AMSb`
   (font.rs:190-191), so `{\usefont{U}{msa}{m}{n}A\fontfamily{cmr}\selectfont B}` is ⊐⊳ (pdflatex ⊐B) and
   `{\usefont{U}{msb}{m}{n}R\rmfamily S}` ℝ𝕊 (pdflatex RS), and inside `\usefont{U}{msa}` `\f@encoding` and
   `\cf@encoding` expand to `AMSa` where TeX says `U`. Open.
15. **A raw `\mathchar` in an AMS family, and `U/msb`'s multichar slots** (review 89d r1 P4, P5; the same on 89c):
   `$\mathchar"0\A04$` (amssymb, `\edef\A{\hexnumber@\symAMSa}`) decodes through OT1 as Ξ (pdflatex msam10's ■), and
   `"2\A01` as Δ (pdflatex ⊞); and `U/msb` has no family multichar table, so AMSb's negated pairs (the empty msbm map
   slots holding two-character glyphs) are not reached through it, a sibling of 89d's family multichar reading. Open.
16. **`\Pisymbol{psy}{97}` is an empty box with 0 Warnings** (review 89d r1 P3; Perl the same, pifont.sty.ltxml:21-24):
   pdflatex prints α (Symbol's "61). A silent loss: the box should carry the psy glyph (a `psy` table) or U+FFFD with
   the font's Warning. Open.
17. **(perf) `fontmap_family_key_syms` allocates two `String`s per math character** (review 89d r1 N3, SUSPECTED
   negligible): its memo is keyed by `(String, String)` (content.rs `FONTMAP_FAMILY_KEY_MEMO`, the lookup at
   content.rs:6346), so every decode in a family pays the allocation the R6 audit removed elsewhere; key it by
   `SymStr`. For a perf batch, measured with `perf stat -e instructions:u`.

### Fragid Parity Open Items
1. **`associateNode` (Post.pm L508-585) unported**: Generated MathML/OpenMath nodes carry no `xml:id`, so `convertedIDs` and pmml↔cmml parallel cross-linking do not exist.
2. **`in_page_id` lacks `labelids` and `split_from_id` branches**: Affects `--splitnaming=label*`.
3. **`strip_ref_display_fragids`** (crossref.rs:131): Matches `//ltx:ref//*[@fragid]` wholesale; narrow to IDs absent from the ObjectDB.
4. **`make_sub_collection_documents` returns `vec![]`** (collector.rs:141): `--splitindex`/`--splitbibliography` drop entries past first initial.

### Batch and integration narratives — archived 2026-10-23

The landed batches 63a–79b and integrations int51–int73 (with the run-329 and run-336 residual notes they grew out of)
moved verbatim to [`archive/SYNC_SESSIONS_2026-10.md`](archive/SYNC_SESSIONS_2026-10.md) § "Moved 2026-10-23"; search it
by the batch name. Their open residuals are rows of [`perfect_kernel/OPEN_ROWS.md`](perfect_kernel/OPEN_ROWS.md)
§ "Archived-narrative residuals (digest 2026-10-23)" (292 items as 209 rows). New batch and integration narratives go to the month's `archive/SYNC_SESSIONS_YYYY-MM.md`
directly; this file keeps rows only. Archived names: 65c, 64zf, 65d, 65b, 65e, 65a, 65h, 65i, 65m, 65f, 65l, 65j, 65r, 65k, 65p, 65n, 65o, 65s, 65t, 65v, 65w, 65x, 65y, 65z, 66b, 66c, 66d, 66e, 66f, Runs 346/347, 67d, Fleet TeX tree, 67g, 67j, 67l, 67m, Runs 366/367, Runs 364/365, Runs 362/363, Runs 360/361, Runs 358/359, 68f, 68c, Run 357, Run 353, 67o, Run 351, 67n, Runs 354/355, 67q, 67r, 67p, 67s, 67u, Residual of 67u — our empty `\Gin@driver` sends trimclip to its PGF fall-back, 67ps, 68d, 67x, Fleet packaging (68d study, not a loader bug), 67y, Residual of 67y — a font-size change after a formula rewrites its `tex=`, Residual of 67y — a `\par` in a frontmatter note pulls the formula after it into the styled text before it, 67z, 68k, 68n, 68g, 68a, 67w, 68q, 68r, 68s, 68t, 68v, 68w, 68x, int17, 68y, 68z, 69a, 69b, 72a, int20, 70b, 69c, int19, 70a, 71a, fms1, 73a, 72b, 73b, 70c, 74a, int25, int26, 76a, 75a, 77j, 77k, 77s, 77u, 78a, 78b, 78c, 78d, 78e, 78m, 78p, 78i, 78l, 78k, 78o, 78q, 78j, 78g, 78f, 78n, 78r, 78s, 78u, 78t, 78v, 78y, 78w, 78z, 79a, 78x, 79b, split114, FM S2a.1, FM S2a.2, 77x, 77p, 77v, 77w, 77y, 77t, 68i, 68j, 68b, 68h, subfig passes a key's value unexpanded, A stored `\ProcessOptions` list drops `\protect` before a robust or optional-argument command, caption v1 keywords given by a protected or robust macro, `\ProcessKeyOptions` sees an expanded option list, A local `my<binding>.sty` loads the binding instead, Residual of 68h — trimclip's PGF fall-back under `[dvipdfmx]`/`[dvipdf]`, 67v, 68u, 67b, 67a, 66z, 66y, 66x, 66w, 66v, 66t, 66s, 66r, 66q, 66o, 66n, 66m, 66l, 66k, 66j, 66g, 66h, 63b, 63l, 63k, 63j, 63i, 63h, 63g, 63f, 63e, 63d, Fleet address-space ceiling → `Fatal:TooManyErrors`, Mouth EoF, Document Malformed, never_completed, IMS bindings.

### Open-row detail — moved 2026-10-23

"Corpus Triage Quick Wins", "Perfect-kernel RED repros without a row elsewhere", "Fidelity audit round 2" and "Open rows
from the main-stream handoff register" moved verbatim to [`perfect_kernel/OPEN_ROWS.md`](perfect_kernel/OPEN_ROWS.md) (rule 3:
a section past ~100 lines is its own doc). Every row there is open unless it says otherwise.

### ar5iv tracker residuals (frozen sweep: `archive/AR5IV_DIAGNOSTICS_2026-08-14.md`)
- **Close-out pending:** the screened issues (the ~48 already 0-error + the 16 fixed by PR #306) are still OPEN on
  dginev/ar5iv (2026-09-27) but for #503 and #555 (closed 2026-07-19); #546, #550, #598 went 0-error on 2026-07-20; close via a maintainer batch list after an ar5iv redeploy, spot-checking each reported
  symptom — never post unilaterally.
- **Rust-only:** #472 `2311.06609` siamart paper-local `code` env (`list`+`tabbing`+`\mathcode`+`\mynewline`, inline
  `$…$`), 82 vs Perl 39 (the 07-18 reclassification; 17 in the archive's 10×-parallel table); #556 `2508.07407` tikz `calc`-coordinate cloud loop, contained (31 KB salvage;
  `docs/reproducers/tikz_calc_node_recursion_2508.07407.tex`).
- **Unclassified since 2026-07-20:** #520 `2412.06264` 337 `\or` at `\end{document}` (deferred-content `\ifcase` leak);
  `2305.05665` 33 macro-generated `unexpected:_` (not axessibility, not ar5iv-only); #558 `2301.12995` 16
  `\@end@tabular`; #473 `2310.17416` 9 errors after the blkarray binding, whose `block` `(`/`[` sub-region delimiters are
  dropped (blkarray_sty.rs:14-21).
- **Grew since the sweep:** #576 `2511.16624` — 4 → 2 errors after the fairmeta binding, now 9 in cortex 2026-08-26 (8×
  `\noalign`, `{subfigure}` undefined): a new cause, untriaged.
- **Shared, surpass candidate:** `2604.16007` acmart `\@ACM@balancefalse`/`\@ACM@pbalancefalse` undefined (acmart.cls:123
  `\define@boolkey`; neither binding defines them).

---

## Standing policies & method

### Methodology & the cortex cross-join
- Working method: **re-triage LARGE-error papers** (the single-error tail is exhausted) → bisect the doc to the trigger line → verify Perl with `--verbose` → fix the divergence.
- Cortex API: `http://127.0.0.1:8000/api`. Endpoints: `/api/reports/<corpus>/oxidized-tex-to-html/<severity>` and `/api/corpus/<corpus>/tex_to_html/document/<id>`.

### CSS themes — `ar5iv.css` vs `LaTeXML.css`
- `ar5iv.css` is actively developed (`~/git/ar5iv-css`). Base `LaTeXML.css` is a faithful copy of upstream Perl LaTeXML's default theme; bugs in base CSS route upstream to `brucemiller/LaTeXML`.
- Sizing intent gap (witness #721, ar5iv#83): Preserve absolute (`7in`) vs relative (`\textwidth`) intent in `image.rs:186` instead of flattening both to `pt`.

### Algorithm Markup + CSS Unification (0.7.7 Target)
- Part 2 landed (line numbers, captions, CSS mirrors).
- Shared markup class: Assign every algorithm listing a shared marker class (regardless of surrounding env) so one generic CSS rule targets it in both `LaTeXML.css` and `ar5iv.css`. Details in [`parity/ALGORITHM_RENDERING.md`](parity/ALGORITHM_RENDERING.md).

### CLI options policy (Option-C) + `validate()`
- Wire only options whose engine feature works end-to-end; strict clap parser (no accept-and-warn stubs).
- `--validate`: STUB today (`latexml_post/src/document.rs:1717`). Requires safe RelaxNG bindings in `rust-libxml` published to crates.io before wiring.

---

## Parked families — pointers to dedicated docs

| family | doc |
|---|---|
| Environment markup class (`ltx_env_<name>`) | [`parity/ENV_MARKUP_DESIGN.md`](parity/ENV_MARKUP_DESIGN.md) (Phase 2 / Post-Release) |
| Beyond-Perl performance levers (BP-1…BP-6) | [`performance/BEYOND_PERL_LEVERS.md`](performance/BEYOND_PERL_LEVERS.md) |
| Content-MathML & math parser gaps | [`math/CONTENT_MATHML_GAPS.md`](math/CONTENT_MATHML_GAPS.md) |
| Deep deferred families (`.bst`, xy-pic, etc.) | [`parity/DEFERRED_FAMILIES.md`](parity/DEFERRED_FAMILIES.md) |
| Stage 4 WASM bring-up plan | [`release/WASM_COMPATIBILITY_PLAN.md`](release/WASM_COMPATIBILITY_PLAN.md) |
| Streaming core DOM design | [`archive/STREAMING_CORE_DESIGN_2026-07-29.md`](archive/STREAMING_CORE_DESIGN_2026-07-29.md) |
| Two-pass streaming split | [`archive/STREAMING_POST_DESIGN_2026-07-06.md`](archive/STREAMING_POST_DESIGN_2026-07-06.md) |
| Multi-document streaming post-join | [`performance/MULTIDOC_JOIN.md`](performance/MULTIDOC_JOIN.md) |

---

## Reference & stable notes

- **SVG picture path:** `<ltx:picture>` is converted on the live page DOM by `latexml_post::svg::SVG` (the faithful `SVG.pm` port) before MathML/XSLT — Perl's chain order — since batch 56bz (2026-09-17); the former regex fragment table + post-XSLT placeholder splice in `post.rs` is deleted. The libxml2 `PostDocument` use-after-free that kept the DOM port benched was fixed by the idcache relinking in `PostDocument::drop` (#480) on libxml 0.3.21; `crossref.rs`'s live-DOM `replace_node` was the production precedent. Guards `cluster_xslt_split::picture_svg_on_the_live_dom::*`.
- **Picture `\unitlength` sizing:** Inkscape `.pdf_tex` pictures ignore `\unitlength` during core `{picture}` sizing, producing degenerate (sub-pixel) outer SVGs; the picture-nested figure still renders because the XSLT emits it inside an `overflow="visible"` `<foreignObject>` at its own size (the former string-path `DEGENERATE_SVG_PX` special case went with batch 56bz). The core-side `\unitlength` gap is open.
- **Primitive layer:** Audited faithful (2026-06-20); core arithmetic, glue, conditionals, and token tables match Perl byte-for-byte.
