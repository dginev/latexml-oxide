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
- **Silent-loss inventory**: open losses of authored content or markup with 0 errors (blocks the 98 % claim); its own doc since 2026-11-09, [`perfect_kernel/SILENT_LOSS_INVENTORY.md`](perfect_kernel/SILENT_LOSS_INVENTORY.md).
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
   in at bundle time. The user (2026-11-09): "we may want to compact that file"; the reason is the main session's: its
   one-line rows conflicted on nearly every merge.
4. **This is the brief actionable list; landed work leaves.** Day-by-day logs live in `git log` and `docs/archive/`.
   When an item closes, move it, in conclusion form, to the month's `docs/archive/SYNC_SESSIONS_YYYY-MM.md`; an open
   residual stays here under its row.
5. **Keep it under ~120 KB.** When a section outgrows ~100 lines it has become its own subject: give it a doc under
   `docs/` and leave a one-line pointer.
6. **Compaction moves only closed history.** An open row stays whole and live (here or in its own doc); only its closed
   facts may go to the archive (SYNC1c, 2026-11-09).

*Last compaction: 2026-11-09 (SYNC1c) — every open row whole and live again: row G in the ranked worklist, the inventory's 84 long rows and the pre-flip leftovers row in [`perfect_kernel/SILENT_LOSS_INVENTORY.md`](perfect_kernel/SILENT_LOSS_INVENTORY.md), split at sentences only. 2026-11-09 (SYNC1b) — the silent-loss inventory moved to [`perfect_kernel/SILENT_LOSS_INVENTORY.md`](perfect_kernel/SILENT_LOSS_INVENTORY.md), with the open facts of its 33 long rows lifted from the archive. 2026-11-09 (SYNC1) — every table row became a `###` section of one-fact bullets; the landed rows (run 336, the bundle rows int119a-int152a, int133a, the pre-flip leftovers' history, XP, R4), the long rows' landed history and the Rust Error Fixes' bodies moved verbatim to [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1); before: 2026-10-23 — the batch and integration narratives (63a–int73, 7,464 lines) and 51 landed rows moved to `archive/SYNC_SESSIONS_2026-10.md`, the four open-row detail sections (751 lines) to `perfect_kernel/OPEN_ROWS.md` (8,677 → 426 lines); before: 2026-10-04 — the run-329 narratives (61r-62f) moved to `archive/SYNC_SESSIONS_2026-10.md`, the parked math rows to `math/MATH_PARSE_RESIDUALS.md`, landed rows removed; before: 2026-10-02 — the K13 stage-2 findings and the rc131/rc132 + 57cn–59l rows moved verbatim to `perfect_kernel/archive/LEDGER_PHASE57_58_2026-10-02.md`, their open residuals kept here. Prior: 2026-09-27 (closed R1, R2, R5 and the 2026-08-02 status snapshot to `archive/SYNC_SESSIONS_2026-09.md`); 2026-09-03 (949 → ~320 lines; 1276 → 953); 2026-08-18 (1462 → ~890); 2026-07-25 (1979 → ~500).*

---

## Ranked worklist — start here

Ordered by importance (2026-10-04 review): closing the perfect-kernel goal, then run 329 and its residuals (they seed
the coming arXiv success-rate task, not started), the rulings that block rows, the work safe to do while the run
finishes (T1, T2, F1, F2), then the older parity rows. Within a tier: **does it reproduce today** → **is a real user
affected** → **is it unblocked** → **effort**.

| # | item | state | size |
|---|---|---|---|
| R | Road to the main merge (user 2026-11-09: "keep planning out all steps … towards completing our main goal"; … | OPEN — step (1) **done in int155a**: `author_block/` deleted (Plan A: Z18k, Z18l reroutes, Z18m/Z18m2 the rest moved); left Z18n, the e0 fence hardening; step (2) B1/B2, CR10b, CR10c, pazh `\address`, `\@nil` done; neurips_2024, 77p, the image before a URL left | L |
| 0 | The TeX Live manuals: every regression addressed before the merge (user 2026-11-06: "This remains the primary … | met; margin batches landing | — |
| 0a | M3: the manuals' remaining regressions | F1-F10 done, 3 deferred | S |
| 0b | The frontmatter's old engine out of every decision before the merge, and all it leaves unused deleted (user … | **Route D (user 2026-11-06):** the experiment, then one switch for every paper with an author block; a failed … | about 12-15 batches |
| 0c | Noticed during the performance work, no fix yet. | open | S each |
| R8 | Perfect-kernel program | **CLOSING**: bar confirmed (user 2026-10-04); ar5iv-css PR #54 open; run 329 completes | program |
| G | Run 336 | TRIAGED; open residuals below | program |
| KC | Kernel capability program | **ORDERED**: K24 LANDED (`ab_audit`, p0 `1e47551e6c`); manual sweep #149 → K20 G4 → G2 → G3 → G1 → K21, K22's … | program |
| MS | TeX's math-group recovery | **PARTLY DONE 65n**; (1) next | M |
| XP | A detail-less XPath failure in post is a resource failure, not 100+ Errors | closed; archived | — |
| BD | A `\DeclareBoldMathCommand` delimiter after `\left`/`\right` | **OPEN**, RED only | S-M |
| A2 | Fidelity audit round 2 residuals | **OPEN** (12 only); re-read 2026-10-25: entics `\thanksref{ALL}` (10), `$(a)$` marks (15) and the llncs check … | small |
| A3 | arXiv html_feedback subgoal | **OPEN**; baseline sweep + 4 classes verified (raw-latex-leak 12/15 fixed, tcolorbox 6 fixed + 5 viewer, tikz … | medium each |
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
| R6 | `Collector::rescan` Refactor for Generated Backmatter | **OPEN**; `Scan` owns `ObjectDB` by value; generated backmatter lacks full relations/labels | medium |
| R7 | Presentation-MathML **F5** Linebreaker | **OPEN**, full linebreaker feature gap needing a port-or-drop scope decision | family |

### R — Road to the main merge (user 2026-11-09: "keep planning out all steps … towards completing our main goal"; inventory `agent_reports/2026-11-09_PREMERGE_inventory.md`, 30 …

- **Road to the main merge (user 2026-11-09: "keep planning out all steps … towards completing our main goal"; inventory `agent_reports/2026-11-09_PREMERGE_inventory.md`, 30 items; supersedes the 2026-11-05 close order)**
- Plan:
  - **Order, each step reviewed and bundled:** (1) **single engine clean** — `author_block/` gone (row 0b);
  - (1) done on `pk/int154a`: the deletion pass DEL1-DEL5 (tests −349, the switch, the old driver −5.8k, the S2b writer and census −12.2k; output byte-identical), Z18a the API's Cut/Kind out of `author_block/` (byte-identical), the llncs reroute Z18c-Z18i (OD #984; its first blockers in `agent_reports/2026-11-09_Z18c_report.md`), Z18j the API engine and field readers moved (Plan A batch 1, `256a8cd611`); history in `archive/SYNC_SESSIONS_2026-11.md` "Bundle int154a".
  - (1) **Plan A for the rest of Z18 (user 2026-11-10, "Plan A: move now, reroute in C"; plan `agent_reports/2026-11-10_Z18P_report.md`):** Z18k the omnibus `\authors` + eptcs reroute (done in int155a: Z18k/Z18k2/Z18k3, OD #991); Z18l aas; Z18m move the rest (the IJCAI chain, the authblk/ieeetran gates, the shared mark readers) and delete `author_block/`; Z18n the declined path's row sharing routed through the reading's helpers ("Route via reading helpers": keep its gains, call the reading's one copy, delete `shared_row/`'s token-level copy); authblk, IEEEtran, IJCAI, `@marked`, the stores and the Cut rerouted inside Option C P2-P4.
  - (1) State: **done in bundle int155a** — Z18j, Z18k, Z18l merged; Z18m/Z18m2 moved the rest unchanged (the IJCAI chain to `defs/author_macros/ijcai.rs`, the mark/authblk/IEEEtran readers to `frontmatter/classify/old_reader/`, ERR2's title-code reader to `classify/title_code.rs`) and deleted `author_block/`; Z18o fixed the REDs the bundle introduced. E0h hardened the e0 fence, Z18o2 the stopgap scope (int156a); Z18n (the declined path) folded into Option C P2 (authblk): the emitter writes the no-author-block flushes from the reading, then `shared_row/`'s token copy (836 lines) is deleted.
  - (2) **every RED seen on a real paper fixed** (user 2026-11-09; **narrowed for the merge 2026-11-11**, the user: "The 'fix them all' is a long-term goal for arXiv": before the merge a real-paper loss is fixed only at arXiv reach ≥ ~500, below it counted by a Warning — no silent loss — and kept as a catalogued RED; fixing them all is the arXiv goal after the merge): the silenced Errors FIRST (row 0b's item 39: an undefined command in an author field, 18 A/B papers), then the content residual (`FRONTMATTER_CONTENT_RESIDUAL_AT_EMIT2.tsv`: every word kept or counted), ejpecp, NE5's whole blocks, the footnote-counter whole block, the ORCID logo, 77p's grouping; synthetic-only stay REDs;
  - (2) done on `pk/int154a`: the silenced Errors (ERR2, OD #983); the content residual triaged (CR1, CR5, CR9) and its real classes fixed (CR2-CR4 OD #985/#986/#972, CR6 + CR10a OD #987, CR8 OD #989/#990); ejpecp and the ORCID logo closed (CR5); the footnote-counter block written as Perl's split (CR7-CR7d, OD #988); NE5's whole blocks split but 0809.5152, 1110.6375 (the fail-safe) and 0906.2789.
  - (2) State: done in int155a — CR10c (IEEEtran grades), CR10b (CR9's glued breaks, the `\fcolorbox` e-mail line, jheppub's abstract in `\author`), the push's B1 (1511.09009) and B2 for aaai2026/neurips_2022 (PB/PBb); left — pazh `\address` and 1802.01191 `\@nil` (RCcut), neurips_2024, 77p's grouping (2603.07345), the image before a URL (2503.08161); every open fact in `perfect_kernel/SILENT_LOSS_INVENTORY.md` "Bundle int154a — the residual".
  - (2) State in int157a: the image before a URL kept, or counted where no written item holds it (IMG1, OD #989); left — neurips_2024, 77p's grouping (2603.07345), aa `\mail{\email{x}}`; open facts in `perfect_kernel/SILENT_LOSS_INVENTORY.md` "Bundle int157a — the residual".
  - (2) State in int158a: a glued e-mail label names its contact, the address the content (GL1, OD #956/#992: hep-ph9606235, astro-ph0001190, hep-ph0411269, 2206.00543, astro-ph0107243, gr-qc0212108 — a broken `mailto:` before; Rust-only, ~380 arXiv); open facts in "Bundle int158a — the residual".
  - **Pushed:** `pk/int154a` → `perfect_kernel` `1969cb34cb` (user 2026-11-10 "Push now, fix next", REDs B1/B2); `pk/int155a` (PB, Z18k, CR10b, Z18l, PC3, Z18m, Z18o, PF10) → `perfect_kernel` `8670d33a6c` (chain: gates 8017/8017, A/B E 45,827 → 45,818 F 172 → 172, nets worse 0, sweep #169 G1-G4 0, cost +0.14 %); `pk/int156a` (E0h, RA0, Z18o2, MB1, MB2) → `perfect_kernel` `e25c1a4881` (chain: gates 8026/8026, A/B 23 changed E 45,818 → 45,818 F 172 → 172, nets worse 0, sweep #170 G1-G4 0, cost ≤ +0.21 %); `pk/int157a` (OC1, SEP1, OC1c, SEP1b, IMG1, P20) → `perfect_kernel` `fcf7747d29` (chain: gates 8047/8047, A/B 11 changed (worse 0) E 45,818 → 45,818 F 172 → 174 (two A/B timeouts, clean on a quiet core), nets worse 0, sweep #171 G1-G4 0, cost ≤ +0.37 %); `pk/int158a` (P21, GL1, P21b, P21c, P24b, P21d, P21e, EJ1) → `perfect_kernel` with this commit (chain: gates 8067/8067, A/B 25 changed (worse 0) E 45,818 → 45,818 F 172 → 172, nets worse 0, sweep #172 G1-G4 0, cost ≤ +0.06 %); each bundle's history in `archive/SYNC_SESSIONS_2026-11.md` "Bundle <label>".
  - (3) **Option C P1-P6** (all segmentation on the digested reading; `FRONTMATTER_SINGLE_ENGINE_PLAN.md`) after Z18, with the plan's DoD items, frontmatter code and test compaction (table-driven, one conversion per fixture);
  - (3) **Rulings 2026-11-11** (`FRONTMATTER_SINGLE_ENGINE_PLAN.md` "Segmentation is digested"): each port deletes the spelling path it replaces (P25 +112, P26 +44, P22 +123 net `src/` lines had not; the four bundles int156a-int159a +1,863), its output changing only where the PDF agrees (a condition; the goal stays every source word kept in high-quality semantic markup), every changed paper classified, none worse, its guards updated in the same commit (tests kept passing); the separator event carries its meaning (kernel / binding / raw), retiring the per-class `\and` lists and T8's spelling fallbacks, one shared helper — the next design batch, before P2-3. Each bundle row reports the spelling-split sites left, the net `src/` lines, the frontmatter line count and a dev re-score.
  - (3) State in int157a: **P1 done on both routes** — OC1 the block routes, OC1c the API route (the emitter writes a reading-cut creator call one call per piece, in no-block papers too), OD #993; separators bound by meaning (SEP1, SEP1b, OD #994); P2 mapped (`agent_reports/2026-11-10_P2P_report.md`: P2-0 … P2-8, trap T8); **P2-0 done** (P20, OD #995: the binding-callable separator query, the cut labelseq call).
  - (3) State in int158a: **P2-1 done outside llncs** — OmniBus, ptephy, webofc (P21), sv_support (P21b), aa (P21c, the events walker walking a paragraph's list as no group) hand the whole author list to the reading; P21d fixed P21's OmniBus regression (a name with an affiliation command after an `\and` read as the earlier names' affiliation; `column_own_text`), P21e scoped P21c's paragraph rule to the block's own level, EJ1 made a setter in an emitted author note write once (ejpecp `\EMAIL`); **P2-4 done** — eptcs's cells cut by the separator query's own run (P24/P24b, OD #991 (8): one `ArgSeparator::span` for both callers). OD #993 "P2-1 … P2-1d". Next: P2-2 (S2 institute splits), P2-3 (llncs), P2-5 … P2-8; BT1 parked (three NO-SHIPs; redesign BT2).
  - (4) **the bars**: truth set, then a fresh holdout v4 read once — 98/98 on dev3/devh/dev20k and v4, longtail ≥ off with no regression (user 2026-11-09);
  - (5) **non-frontmatter**: the generalization audit re-scoped and done before the merge (user 2026-11-09), T1 the repro suite move, the perf + RAM pass's remaining levers, row 0a's deferred manual REDs decided, workspace cleanup (keep-list to the user);
  - (6) **close**: goal docs current, the closing manual sweep on the final head (row 0's G1-G5), the 2605/2606 cortex reruns (G6; the fleet rebuild needs the user's OK), the merge PR (the user confirms).
  - **After the merge:** CI ≤ 30 min (user 2026-11-06), the arXiv rerun, A4, AutoTeX, forest, the ar5iv-css port, a release.
- State: OPEN — step (1) done (int155a: `author_block/` deleted; int156a the fence hardened; Z18n folded into Option C P2-7); step (2) left: neurips_2024, 77p, aa `\mail{\email{x}}`; step (3) P1, P2-0 (int157a), P2-1 outside llncs and P2-4 (int158a) done; P2-2, P2-3, P2-5 … P2-8 next
- Size: L
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
- Size: `pgf-interference-en/-de` pass with a thin margin until `pk/P2` (150 s to 33.6 s); clusters B, T and the `?` canaries left, row 0a
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
  - **The deletion pass done (DEL1-DEL5, bundle int154a):** no `LATEXML_FM_EMIT` switch, no old driver, no S2b writer or census.
  - **`author_block/`: DELETED in bundle int155a** (6,458 → 0 file lines: Z18a 6,458 → 4,637; Z18j → 3,364; Z18k2 → 2,552; Z18l → 2,069; Z18m/Z18m2 the rest moved unchanged — the old reader's code now in `classify/old_reader/` and `defs/author_macros/ijcai.rs`, 1,702 lines, fenced by e0's `MOVED_OLD_READER_FILES`).
  - What still holds old-reader code (to reroute in Option C P2-P4): the IJCAI chain and its `@marked`/`@shared` gates (sv, aistats, revtex, iopart, informs reach it), authblk's gates and label readers, IEEEtran's, `frontmatter_stores`' `class_numbered_institutes`, llncs's `affiliation_calls` for `\lx@llncs@institute@sup` — all in the fenced old-reader files.
  - Outside it, still an old leftover: the declined path's row sharing (`place.rs` `share_row_contacts` → `shared_row.rs` + `shared_row/emails.rs`, ≈730 lines, OD #901) — Z18n.
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

- **Open: TRIAGED; open residuals below.** The row whole and live but for its "Fixed in 64p" and "Fixed in 64u" facts, which are in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "Row G".
- **Run 336** (full arXiv, 2,947,191 papers; 62zu `c0e4393f39` to 2026-10-07 16:35Z, then `9569ca27a7`) — COMPLETE 2026-10-10 07:10Z; vs run 306 (snapshot 2026-09-17): error 12.86 % → 6.32 %, fatal 0.87 % → 0.27 %; 203,366 papers out of error, 20,570 out of fatal; 16,827 worse into error/fatal; no_problem → warning 290,846 (the warning-visibility shift); E/F rerun **run 338** (e57be39640, complete 2026-10-12): of run 336's 186,160 error / 8,019 fatal, 42,143 left error and 2,509 left fatal (corpus error 6.32 % → 4.89 %, fatal 0.27 % → 0.19 %); 142 error → fatal to triage; **run 340** (E/F rerun on 1bd0c44f59: 64zf + 65b + 65c, complete 2026-10-12 15:19Z): of run 338's 144,214 error / 5,652 fatal, 2,282 left error and 305 left fatal (corpus error 141,799, fatal 5,634); 287 error → fatal — 204 TooManyErrors (107 = longtable undefined under aa/pasj00/emulateapj, a 65b regression, batch 65h), 59 cortex never_completed (dispatcher lease clock started at batch fetch, fixed in CorTeX a5bbbee5, not yet deployed), 18 PushbackLimit; **runs 344/345** (E/F rerun on f1811254dc: 66b-66e, complete 2026-10-14 05:04Z): corpus error 141,799 → 105,835 (3.62 %), fatal 5,634 → 4,625 (0.158 %), OK + warning 96.22 % of 2,922,325; error → OK 5,966, error → warning 30,248, fatal → OK 81, fatal → warning 678, fatal → error 544; 294 error → fatal, of which a local reconversion confirms 118 (Fatal on 66e1, Error on 64zf: TooManyErrors 68, Mouth EoF 24, Recursion 14, PushbackLimit 9; `~/data/pk_agents/main/ef344/fatal118.tsv`) and the rest are fleet artifacts (OOM under host memory pressure, timeouts)
- State:
  - **TRIAGED** (2026-10-10): 1,571 clusters of the 16,827 (`~/data/pk_agents/main/run336/worse_306_336.tsv`, `agent_reports/2026-10-10_run336_*`, `_rc336_*`).
  - On head already clean: CCSXML EOF (64d), imageprocessing (AppArmor), most `never_completed`.
  - Faithful (pdflatex errs too, `\errmessage` an Error since 56g): ctex fandol 107, l3backend twice 71, acro `class` 52, `\DeclareMathOperator{\free_var}` 155, `\verb` misuse 183, JINST.cls under TL 2025 39 — NOT discounted (user 2026-10-17: TL 2025 build failure = skip PDF checks only), targets of the queued rc_harness study.
  - Fixed in 64h (~2,500 papers): `\hyper@makecurrent` patchable 129 (with the kernel's `\theH<counter>` names, OD #480), `pdftex` option / PostScript-only K6 cue 47, CJK stray bytes 746 + byte-mouth `.bib` 4, tcilatex `\thechapter` 537 + plain `\input #1` 2, aipproc `\tablehead` ~428, elsart wrappers 367, algorithmicx nested listings 134, spverbatim 107, nomencl prefix 12, `\diaghead` pair.
  - Fixed in 64k (OD #482): raw classes' refused `\author`/`\abstract` replayed for their flags, stores kept the kernel's (JINST/JHEP/CECS 340, iosart2x 71), the Foundation Compositio family (224) and JINST/CECS bindings (semantic contacts, keywords, MSC, dates; no second title page), each reading the class file the document asked for (a path-prefixed `JINST-Sample-files/JINST`, a renamed `JINST_mod`) — A/B over 2,918 papers (64h4 → 64k16, the 238 the email rounds touch re-run on the final 64k22): with errors 809 → 252, with Fatals 72 → 2, no creator or abstract lost; a labelled email line of a marked author block is now its author's contact, or the block's (OD #52(m); addresses on several authors 66 → 4).
  - Open: the 31 renamed JHEP3 copies (QCEMDA, AKRSX, jhepLike…) need a class route by content; `blkarray` `\begin{block}{[…]c}` raises `\else` errors (2112.09933, reached once its frontmatter stopped failing); a Semiverbatim URL contact's `~` prints as the OT1 tilde accent `˜` (`\urladdr{…/~y}`, Perl the same; RED sectioning-frontmatter/url_contact_keeps_tilde).
  - Fixed in 64qb (KPE #563-567, OD #483): paspconf bound to aaspp (+ its Q&A, `\deg`, keywords as RDFa) and RevTeX 3's keyless `\reference` (~860), a caption2 binding (843), amsmath's `\macc@` chain with `\skewchar` a font register (817), `\tablebreak` ends the row (38), `\sidecaption[pos]` (186), jmlr2e `\keywords{}` (7), `Digested::to_attribute` ported (a skip in an attribute is its spaces, not `\enskip` text; Perl List/Whatsit `toAttribute`); A/B 332 papers: with errors 301 → 37, none worse.
  - Still open in 64p's fixed fact (the fixed rest archived with it): its REDs (RED: a class end printing text after its list prints it before the entries; a class bibliography built on `{enumerate}` leaves a group open — both synthetic; OD #484, KPE #569); a document's `\mathaccentV` accent (21; its `tex` attribute now reads `\mathaccent 28766{G}` where 62d's named reading gave a `.bbl`'s `\hat{G}` — a `\mathaccentV` reversion is open).
  - RED: keyval runs a key given no value that has no default with an empty value (keyval.sty:52-56 errors and skips it; csquotes `[english]` loops, 5; `macro-state/keyval_bare_key_without_default`).
  - Fixed in 64qa (KPE #570-573, OD #485): World Scientific `\body`/`\bodymatter` and table rules (ws-procs9x6, ws-procs961x669, ws-rv9x6; ~1,078), iopart `\ioptwocol` (1,058), revtex `\rev@citealpnum` (1,023), svmult `\toctitle`/`\tocauthor` (960), mdpi `\history`/`\changeurlcolor` (765), WileyNJD `\state` (701), the ptephy author block (204 → 27 papers with errors), quantumarticle tikz (61 → 4).
  - Open: epsfig `\epsfbox` a no-op (KPE #574, its own batch with an A/B); mdpi's other history fields gated by status like `\history` (display-toggle rule); the ws classes' packages (rotating, dcolumn, url, chapterbib, cite[super,nospace]: bindings load what the class loads).
  - Fixed in 64s (OD #486, KPE #575): ar5iv's `\today` is the paper's source date (the newest TeX source entry time of its zip/tar), `\relax` only for an undated bundle; ptapap's `\today` parse 95 (1801.05985), `<date>` filled on 43 of 60 sampled `\today` papers; open: ptapap `\let\oldfnum@figure=\fnum@figure` undefined (92 papers).
  - Fixed in 64ra (KPE #576-581, OD #487-491): `$$` is display math in IEEEtran `{IEEEproof}`, lineno `{linenomath}`/`{linenumbers*}`, OmniBus `{frontmatter}` and autart `{pf}` (macros as the classes define them, not boxed DefEnvironments; ~1,600 IEEEproof, ~110 linenomath, ~200 frontmatter, ~90 autart papers), IEEEproof titled by its optional argument with IEEEtran's own QED switch (`\IEEEQEDhere`/`\IEEEQEDoff`/new `\IEEEQEDhereeqn`, one tag "∎(1)"), the class QED symbols sized as their boxes so a cell holding only the QED is kept, and `\IEEEeqnarray`/`IEEEeqnarraybox` columns built from the IEEEtrantools column table (≤1k papers); A/B: IEEEproof 60 → 1 papers with errors, linenomath 40 → 3, `\endlinenomath` 30 → 1, frontmatter 39 → 12, autart 26 → 4, IEEEeqnarray 80 → 8, 30 extra-tab eqnarray papers byte-identical.
  - IEEEtrantools' own IEEEproof/QED commands for non-IEEEtran classes landed in 67x (`block-model/ieeetrantools_ieeeproof_any_class` GREEN); open:, IEEEeqnarray `[decl]`/struts (its rule rows landed in 67x), 64 more mode-less `"#body"` DefEnvironments (census in `agent_reports/2026-10-10_payload_64ra.md`).
  - Fixed in 64w (OD #492): an axodraw v1 binding (not in TL; 1,507 papers with `\Text` undefined, 149 as the first error) evaluating v1's PostScript procedures (gluons, photons, arcs, arrows, blobs, boxes, ovals, curves, axes; PostScript-arithmetic arguments over dvips-header constants) into the picture model, `\Text` at `\unitlength`s unscaled, colordvi loaded as the colordvi-aware v1 does; it also answers papers shipping their own axodraw.sty (drawn now, errors equal), while axodraw2 (raw TL file, kept off the versioned fallback) and axodraw4j stay raw.
  - A/B vs e1ec58c24e: 149 witnesses with errors 149 → 10, errors 872 → 94; 63 shipping, 16 axodraw4j, 8 axodraw2 papers unchanged in errors and warnings.
  - Port gap closed alongside: the SVG of an `ltx:arc` keeps its stroke attributes, as SVG.pm `convertArc` copies them (L385-387) — every arc, pstricks' `\psarc` too, had been invisible.
  - Open RED: a picture label mixing text and math overprints in the SVG (graphics-axodraw/picture_label_mixed_text_and_math, hep-ph0509118); a math label in a `\vcenter{\hbox{picture}}` is a bare `<msub>` without `<math>` (graphics-axodraw/picture_math_label_in_vcenter_box); `\hspace*` before a picture is dropped (graphics-axodraw/picture_after_hspace_star); colordvi's `\Red{…}` sets no color (fonts-nfss/colordvi_named_text_color); an article `\author` block's second name after `\\[12pt]` is read as an affiliation (sectioning-frontmatter/article_author_block_second_author_after_spaced_break, gr-qc9906006; GREEN since 85a, OD #900).
  - No generic fallback for a missing `\usepackage` (user ruling 2026-10-10).
  - Open: under `\documentstyle[aps]{revtex}` the OmniBus overlay gives an `\author{A and B}` group's address to B only (RED `revtex3_omnibus_author_group_address`; ruling asked); `\documentstyle[l-aa]{article}`; marks that are a paper's own macros (1201.5125).
  - Fixed in 64y (OD #497-499, KPE #583-585): an environment frame closed by `}`/`\endgroup` (M1, ≈2,060 run-336 papers) ends as tex.web §1064 `off_save` does, the diagnostic following the group kind the begin code leaves (none | semi_simple | box; 71 environments audited); 17/17 witnesses clean (139 → 0 errors), 150 of the cluster 141 → 82 erroring (1172 → 642); 0 regressions over 300 random + 100 theorem/list-heavy + 70 reviewer papers; REVTeX 3 figure list entries (cond-mat9607130 1 → 0) and natbib `\bibpostamble` (2009.08029 1 → 0).
  - RED follow-ups in `boxes-groups/`: `atbegin_hook_group_above_env_frame` (2001.02707), `bare_theorem_ended_by_enclosing_end`, `amsthm_bare_proof_closed_by_brace`/`amsthm_proof_closed_by_endgroup` (the proof is a constructor, so a group closer silently swallows the document), `footnote_bare_quote`, `macro_env_ended_by_inner_bare_end`, `bare_doublespace_closed_by_brace` (2310.08233, setspace's no-op bare `\doublespace`); 2106.02397's float.sty binding no-op `\float@end` under floatrow; `osa,floats` REVTeX 3 figures.
  - Fixed in 64zb (OD #500): a locked binding owns its calls (D1, superseding the arity rule below) — a document's parameterless redefinition of a binding with a braced signature is the meaning for every use without an argument (pgffor `\foreach \i/\text`, `\@for\text:=`, `\def\title{…}` as a value), a `{`/`[`/`*` call still runs the binding (kept as `\lx@locked@<cs>`), a redefinition with parameters (robust and ltcmd forms included) stays refused, `@` names and 0-parameter readers stay absolute; A/B vs 8542e1b1a5: M5 cluster 369 papers errors 11,882 → 2,500 (140 fewer, 125 to zero, none more), census 314/320 identical, restyle 100/100, random 298/300 (2 fewer errors); witnesses 2405.18512 45 → 0, 1406.2544 31 → 0, 1511.05179 1 → 0.
  - Accepted residual: an unbraced parameterless forwarder runs the author's code (`\mathcal C` after `\def\mathcal{\mathscr}`; 1806.09639, 1904.10871 `\sqrt` hack).
  - RED: `macro-state/m5_locked_name_in_expansion_only_context` (`\lx@locked@<cs>` in `\meaning`/`\typeout`), `sectioning-frontmatter/paragraph_title_trailing_gobbler_open_group` (2604.23335, pre-existing); m5_e2/m5_d2 deferred (rulings 4/5).
  - Fixed in 64zc (OD #501-503, KPE #586): an email or web address keeps its `_` and `~` in the font its class sets it in — typewriter (`\lx@address@urlfont`: REVTeX 4.0, AASTeX 5.x, amsart/amsproc/amsbook, llncs, JHEP, jheppub, iopart, aa, elsarticle, cas, imsart) or roman (`\lx@address@textfont`, user ruling 2026-10-11: REVTeX 4.1/4.2, AASTeX 6+, emulateapj, svjour3/svmult, quantumarticle, jmlr, pasj00, ptephy), the author block's contacts made of a class's `\email` call too, `\~{}` in a typewriter address a `~` of its run, the link of a split address its whole text, `\i`/`\j` kept in the ASCII encoding (1001.1076 "aiwen˙fan"; RED `url_contact_keeps_tilde` green); A/B vs 64u over 175 sampled underscore-email papers (7 months of run 336), 100 random and 31 review papers: no Error/Fatal/Warning count changes, papers with a `˙` email/url contact 60 → 0, every XML change on those sets on an email/url contact (font, or `˙`/`˜`/`ˆ` → `_`/`~`/`^`), no address left split into a one-character link.
  - Beyond contacts, the accent of an empty letter is the current font's (OD #503): 6 of 13 sampled `\~{}` papers change in the body, each a run joined or a typewriter `~` (1001.1367, 1001.1793, 1301.6104, 1301.6221, 1901.07113, 2101.11048).
  - Open: a binding's Semiverbatim address freezes `$`/`^`, so an address's math prints as text and its link holds the `$`s (RED `sectioning-frontmatter/semiverbatim_address_freezes_math`; 1601.02576, 1601.02586, 1601.00888; Perl the same, the class of KPE #515); hyperref's `\url{http://x.org/\'{\i}}` prints the accent's expansion in its text and link (RED `sectioning-frontmatter/url_accent_command_leaks_expansion`).
  - Fixed in 64zd (OD #504-505, KPE #574 fixed, #587): REVTeX 3's own options declared in revtex3_support (revtex.sty v3.0/v3.1 `\ds@` and the distribution's option files; `eqsecnum`, `preprint`, `osa`), its numbering as aps.sty ("II A", "II A 1", appendix "A 1", A1, an unnumbered appendix stepping the counter; seg A-1, rmp "II.A") and marked author lists split (OD #52(n): `\cite`, math surnames, "by", bare asterisks); `\compat@loadpackages` loads OmniBus provide-only (`ProvideOnlyGuard`: every binding definition during the load — OmniBus's and the bindings it requires — declines a name defined before it, environments whole, article not re-run, `extends` wrappers such as its `\bibitem` install anyway, TeX `\def`/`\let` untouched); bindings for the 2.09 option styles OmniBus used to stand in for: World Scientific `sprocl`/`mprocl`/`stwol` (appending `\author`/`\address`, `\abstracts`) and AASTeX substyles `aas2pp4`/`aaspptwo`/`apjpt4`/`aj_pt4` → aaspp (AASTeX 3's `{planotable}` ending its own data, astro-ph9502038); author lists (OD #52(n)): glued initials before capitals surnames (`M.CACCIA`), a doubled `\& and` separator, a name line with a `\thanks` between affiliations opening the next author when the block's first names carry one (hep-lat9404004); sprocl addresses led by an author's mark go to that author; AASTeX's obsolete `\authoremail` before `\author` the email of the author it spells, not a creator of its own (astro-ph9803018); packages OmniBus requires under provide-only are unmarked when it ends, so a later `\usepackage` loads them whole; epsfig `\epsfbox` includes its graphic; aas `{references}` titled; A/B against 44ff2ac956 (64zc) over 150 REVTeX 3, 40 REVTeX 2 `{instit}`, 149 other-class missing-option papers, 400 more option-stratified and 46 of the bound style families in `agent_reports/2026-10-10_payload_64zd.md`.
  - Open: marked affiliations then an unmarked group's: the unmarked one reaches the marked authors too (RED `sectioning-frontmatter/marked_affiliations_then_unmarked_group_leaks`; revtex4 the same); an article author block's plain name line between plain affiliation lines stays an affiliation (RED `sectioning-frontmatter/article_plain_name_line_between_affiliations`); `\input` of a binding in a group a second time finds its definitions gone (RED `loader/input_binding_in_group_twice`, cond-mat9610182; TeX re-reads, Perl the same; low priority); hep-th9707026's glued `Sahng-kyoon.Yoo` list stays one creator; the ws classes' psfig/epsfig load (RED `singletons/ws_procs_psfig`, now unblocked by KPE #574).
  - Fixed in 64x (OD #506-511, KPE #588-597): titlepage `{centering}`; natbib `\bibitem` outside any list reports Lonely `\item` (Perl's `malformed:ltx:bibitem`, once per `\bibitem`; 1108.2593 40 Errors kept, structure recovered) and among a list's own items keeps its place (1411.7266 10 → 0); implicit-bibliography headings absorbed or untitled (no contents entry); biblist reopen with the phantom prune and emptied-list sweep at the bibliography's and document's close (147 → 6 phantoms over 127 papers, the 6 holding figure panels); only a TeX `\item`/`\bibitem` steps the counter (ACM `.bbl`-preamble ids now `bib.bibN`, user ruling); a `.bbl` bibliography after an implicit one prints again (KPE #596); AMS-TeX `\end`/`\predefine`; `\endabstract` locked against files; jheppub `\abstract`; PoS without JHEP journal macros + `\pos`.
  - A/B 127 papers vs 8c7ed11a23: Errors 1986 → 1276, none above base, every drop a shape pdflatex prints cleanly (`scratch_64x/ab/drops10.txt`).
  - Accepted residual: 2205.07302 keeps its words but loses its `\enquote` marks as `<ERROR>` (KPE #589, RED index-bib/bibitem_list_local_definition, deferred to K21).
  - Open RED: kernel `\bibitem` in the body silent (KPE #595; Warning per ruling, batch 64zf stage 2); natbib `\bibitem` among an enumerate's items (cond-mat0003103, 1 Error; K21); a paragraph after a lonely natbib `\bibitem` with no heading between becomes a keyless entry (pdflatex: a paragraph; RED index-bib/natbib_lonely_bibitem_then_paragraph).
  - Fixed in 64ze (OD #512-517, KPE #598-599; K22 census binding gaps): natbib entry interface (`\BibitemOpen`/`\BibitemShut`/`\bibinfo`/`\bibfield`; 80 → 2 papers with errors; a lonely bibliography spends no `\BibitemShut` stop); svmult/svmono (`{svgraybox}`, `\svhline`, `{prob}`/`{sol}`, `\motto`/`\chapauthor` via the after-heading hook, svmono as a book; 167 → 14); aipproc `\pacs`; svjour `\preprint`; interact/World Scientific `\bm`; IEEEAerospace; quantumarticle hyperref + REVTeX compatibility (PACS/preprint as RDFa, natbib numbers; 46 → 5); elsarticle bare `\frontmatter`; mn2e abbreviations per shipped copy (217 → 77); kernel: fallbacks yield to the author's shipped files, `\noalign\\` (2309.15248).
  - A/B 1,530 documents vs e57be39640: no rises, 679 papers fewer with errors.
  - Open: pasj00/PASJ95 absent-class emulation (~190 papers; own batch); quantumarticle `plainnat` default; svmult 2018 info boxes; RED `noalign_let_tabularcr`, `svmult_frontmatter_chapter_numbered`, `aipproc_nested_pacs` (0709.4622).
  - Queued from the run-336 Error/Fatal ranking (`agent_reports/2026-10-10_run336_error_fatal_ranking.md`, triage `rc_undefA`/`rc_undefB`/`rc_struct`): 64q binding gaps behind first-error undefined commands (ws `\bodymatter` ~1,078, iopart `\ioptwocol` ~1,058, revtex4 `\rev@citealpnum` ~1,023, svmult `\toctitle` ~786, mdpi `\history` ~765, paspconf ~860, caption2 ~843, amsmath `\macc@` ~817, WileyNJD `\state` ~701, ptephy ~204); 64r roots behind stray `&`/`_`/`^` (`\lefteqn` as a 3-column span ≤2.2k, `$$` in boxed binding environments ≤1.7k, `IEEEeqnarray` column spec ≤1k, redefined `\span`, a braced amsmath body, `\tablebreak`, `\sidecaption`, jmlr2e `\keywords`).
  - 64l pgfkeys native dispatch (~17), 64m ar5iv preload loads the format early (`\salta`, 84), 64n alignment cell-head peek (61), 64o delimited argument past an unmatched `}` (Fatal → Error), 64p loops (`\renewenvironment` lock 132, `\tag`/`\theequation` 76, `\let` wrapper 33, `\mathaccentV` 21, keyval 5).
  - Ruled 2026-10-10 (queued 64t-64w): `&` as text in stored-but-untypeset metadata, bindings for l-aa/laa, epl2, OSA opex3/ol2/osajnl2 and REVTeX 2 `{instit}` (plus a generic raw-loading improvement), `\left`/`\right`/`\middle` unexpandable (62 IfLimit in xymatrix), an axodraw v1 SVG binding (1,507).
  - Ruled later 2026-10-10: an environment frame closed by a plain `}`/`\endgroup` recovers as tex.web `off_save` does (M1, ~2,060, unparked: 64y); a locked binding yields only to a redefinition with a different number of mandatory arguments, after minimal repros show the rule well-founded (forwarding 0-argument redefinitions such as `\renewcommand\section{\@startsection…}` excluded; M5, ~160); a heading between `\bibitem`s is an inline sectional block inside the one bibliography (~750); a missing `\usepackage` file gets no OmniBus fallback, only `Warning:missing_file` (pdflatex fails too: 1210.7592 `opex3.sty` not found; shipped copies raw-load); REVTeX 3.0's real options declared in revtex3_support; an implicitly opened bibliography takes a preceding heading-like line as its title, else none (no `\refname`)
- Size: program
- Detail: [`PERFECT_KERNEL.md`](PERFECT_KERNEL.md) close status

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
- Facts carried (closed row; the residual note):
  - a fresh-process `ulimit -v` sweep of 2101.04438 (1.2-7.0 GB) never reproduces it (the Rust side fails first), so it needs a reused child.

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
  - acmart comma list (2b) 63x (KPE #549), an.cls `\abstract{}` (9) 77b ([silent-loss](perfect_kernel/SILENT_LOSS_INVENTORY.md) #23)
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

Moved 2026-11-09 (SYNC1b) to its own living worklist, [`perfect_kernel/SILENT_LOSS_INVENTORY.md`](perfect_kernel/SILENT_LOSS_INVENTORY.md): 226 rows, 220 of them open (the rest closed, accepted or no action), one `###` section per row. A "SYNC row" citation of an inventory row (the numbered items #5-#27, the frontmatter residuals, 108b, Z14, the int153a flip row, "Before the flip", "Open residuals lifted from landed rows") resolves there by the same title.

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
- **lamstex arrowheads print as letters** (PERL-ORIGIN, rev78b; OPEN): a `\newCD` diagram's arrowheads and dotted shafts are `\char`s of the lams1-lams5 fonts, which no font map covers, so they print as "w", "h", "ﬂ" (math0005042's pullback diagram).
  - Open: lams1-lams5 font maps (arrowheads and shafts to Unicode arrows and rules) or drop the glyphs.
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
