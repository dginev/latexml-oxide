# Engine Sync Status — Active Worklist

> **DO NOT downgrade Errors to cheat the task.** If Perl LaTeXML converts a paper
> without a downgrade, the Rust translation must match by improving the core
> engine — never by silencing diagnostics. New downgrades require explicit proof
> Perl emits the same severity on the SAME paper, else they hide a real gap.
> (User directive 2026-05-15.) Always classify with `latexml --verbose`, never
> `--quiet` (which hides Perl's `Error:` lines); cross-check pathological inputs
> with `pdflatex`.

## How to read this file

**Start at "The task list" below (row R): DG's campaign order, then the worklist.** It is the single entry for the work
lists (DG 2026-11-12: "We should have a single point of entry for work lists"). Everything after it is supporting detail.

- **The task list (row R)**: every open task, one line each; first, always.
- **Open-row detail**: [`perfect_kernel/OPEN_ROWS.md`](perfect_kernel/OPEN_ROWS.md) — the post-merge rows' detail under
  the same ids ("Post-merge rows moved from SYNC_STATUS (SYNC3)") and the older open rows.
- **Silent-loss inventory**: open losses of authored content or markup with 0 errors (blocks the 98 % claim); its own doc since 2026-11-09, [`perfect_kernel/SILENT_LOSS_INVENTORY.md`](perfect_kernel/SILENT_LOSS_INVENTORY.md).
- **Rust Error Fixes**: Rust-only defects fixed (with their KNOWN_PERL_ERRORS cross-reference); read before filing a defect as Perl's.
- **Current status**: where the corpus measures live.
- **Standing policies**: rules on *how* to fix (CLI flags, stubs, divergences).
- **Parked families**, **Reference**: pointers to dedicated docs; stable facts.
- **Closed history**: the month's `archive/SYNC_SESSIONS_YYYY-MM.md` — the landed bundles, the closed rows, the batch
  and integration narratives (63a–79b and int51–int73 in [`archive/SYNC_SESSIONS_2026-10.md`](archive/SYNC_SESSIONS_2026-10.md) § "Moved 2026-10-23").

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

*Last compaction: 2026-11-14 (DOCSPM, after the merge) — row R became the post-merge worklist: the pre-merge table (GOAL, MPR), its counters, the road's conclusions and the done WIN1 section moved to [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md); the PRs of 2026-11-14 added their residual lines. Before it, 2026-11-12 (SYNC3): the ranked rows folded into row R (detail in [`perfect_kernel/OPEN_ROWS.md`](perfect_kernel/OPEN_ROWS.md) "Post-merge rows moved from SYNC_STATUS (SYNC3)"); the closed rows (0, 0b, R8, XP, R3, R4), the old index table and row R's history in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-12-compaction-sync3), with the earlier compactions' log.*

---

## The task list — start here (row R, the post-merge worklist)

- **The campaign (DG 2026-11-14), in order:** (1) finish latexml-oxide's open PRs and issues and ALL the Bruce Q&A residuals (the QA lines below); (2) a cutoff commit — output-changing merges freeze; (3) rerun the 2605/2606 cortex sandboxes and review regressions against runs 375/376; (4) reconvert the whole arXiv corpus in cortex (ARXIV); (5) sync to ar5iv.
- **The goal now (DG 2026-11-23): [`GOAL_v_0_8_1.md`](GOAL_v_0_8_1.md).** Two timelines: v0.8.0 final on `main` by 2026-11-28 (small reviewed fixes only: regressions, crashes, rc2 tester reports); everything else in this list lands on `int/towards_v0_8_1` for v0.8.1. Each task ends fixed, ruled or moved (the goal page defines each).
- **Where it stands (2026-11-21):** the full arXiv run on the cutoff is run 397 (November 2026, 98.1 % of 2,962,950
  error-free); its regressions were classified and the clusters our changes caused fixed (#954-#964), the residue in
  row R. **0.8.0-rc1** is published as a prerelease (tag `f7f8120f349`, merged to main as #919); **0.8.0-rc2** is main
  `7c2b9ef3b76` (#965: Split frees its pages, release checks; #966: the version); its tag was cut and its release
  run cancelled (unused), and it is re-issued on the large-document perf work (DG 2026-11-21): #970 (STRMWARN,
  CORELEAK, POSTPERF, CHAPLINKS, ALLOCTRIM, BIGSPLIT, mimalloc 2.3.2) and SERSTREAM (the core result serialized to
  disk; issue 777's whole book converts, 1,549 s on main 4a5aae128b2 against lualatex's 3,523 s for two passes). Main stays the release line until 0.8.0
  final; the reviewed post-release branches (COPYROUTE, ENCURL) merge after it. The ar5iv sync of the run's bundles (every month, then a cache flush) is running.
- **Where it stood (2026-11-18):** the frontmatter campaign merged as PR #895 (`aeda3db4e47`, 2026-11-17; DG's 39-paper
  showcase review in runs 389–392, a 300-paper acceptance A/B, and a performance audit whose XSLT-key cost was cut to
  median −0.08 % / p90 +0.46 % vs main, the residual row R XSLTKEYc). **Cutoff = `aeda3db4e47`** (DG 2026-11-17): the
  fleet runs it, and the 2605/2606 reruns are runs 395/396, compared against 379/380. The Bruce Q&A lines below move to
  after v0.8.0 (DG 2026-11-17: "the 35 Q&A items will be carried to post-release v0.8.0"); they no longer gate steps
  (2)–(4). ar5iv-css 0.9.5 released (cortex serves it).
- **The merge (closed):** the perfect-kernel goal met on the final tree `803ce550a6`, merged into main on 2026-11-13 as `6df4632fb1` (a regular merge, DG 2026-11-13). The verdict's basis and the deferred list with each ruling: [`archive/PERFECT_KERNEL_2026-11-13.md`](archive/PERFECT_KERNEL_2026-11-13.md) "Close status"; the pre-merge table, its counters and the road's conclusions: [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md) "Moved from SYNC_STATUS by the 2026-11-14 post-merge reorg".
- **Done since the merge (2026-11-14/15):** PRs #826 #831 #832 #812 (portable tests, CI lint, environment writes before the first thread), #833 (WIN1), #828 #830 #834 #835 #836 #839 #840 #813 #821 #842 #844 (issues 736 760 772 773 778 785 787 794 795 803 806 807 782 …), #837 #838 #841 (issues 755 789 809 824's isotopes), #829 #843 (Rhai and the binding macros: issues 791 792 796 825), #846, #827, #800; 32 issues closed (`gh issue list --state closed`). Lines closed here: WIN1 (#833), CORERES's pathname.rs `TEXMFROOT` write (verified before the prewarm spawn and allowlisted by `tools/lint_env_writes.sh`, #812/#831), GUARDC's visible skip (#826). Each PR's recorded residual is a line below (WIN1R, RHAIR … GRMLINT). **2026-11-14/15:** #845 (the upstream sync, UPSYNC), #847 #848 #852 #855 #864 #865 (issues 808, 695, 810/475, 853, 857, 802), #849 #860 (Rhai round H, the cheat sheet), #850 (mhchem stage 1), #854 #856 #858 #859 #862 #867 #869 (Bruce Q&A QA26, QA47B, QA24/QA58/QA64, QA91 + DEPN, QA90), #866 (LOSTF), #868 (amsppt), #870 (ODR10's two hooks), #851 #861 #863 (docs, CI); lines closed: UPSYNC, QA24, QA26, QA47B, QA58, QA64, QA90, QA91, DEPN — their residuals are IFGEO, QA26R, QA47R, QA58R, QA90R, QA91R. **DG's 2026-11-15 docs rulings, applied by ODDOC-E (docs/dg-rulings-1115):** BIB — recorded as they stand, what the `.bib` session supplies (#72's url.sty, #78's catalog line, 62zr's MathSciNet macros, bibtex.rs:3044) a bullet of #58, #73 amended by #74; OD #615 — TeX wins, llncs's "X 11" (KPE #1111; the 2026-10-16 "X 1.1" ruling retired); CLAUDE.md — the Key Concepts row `LaTeXML::Package` → `latexml_core::binding` + `latexml_package` (ORGANIZATION.md:15, :74); ODDOC-D Q1 — #993 restated as today's code (the bindings' `split_tokens`/`\lx@splitting` cut by spelling, generic binding utilities; RED `article_author_tail_let_and_stays_a_row_red` stays); Q3 — old #916 retired (inked notes' drops accepted, the seven `kernel_thanks_*_red` `status: RETIRED`, #981); Q4 — CLSFM's 19 rows for bindings Perl lacks moved from #460 to "Bindings beyond Perl's tree" (#460 keeps svjour3, elsarticle/cas, quantumarticle/ptephy, REVTeX 2.1, AASTeX); Q5 — #918's bullet in #911 states #924's amendment. **2026-11-15/16 (the pre-cutoff close):** #885 (the perf pass: typical papers −10.4 % instructions, the issue-790 book −18 %, output byte-identical; the string arena made lend-sound; libxml 0.3.24 and libmarpa-asf-sys 0.3.1 from crates.io), #886 (mutool's page ≥ 2 SVG and the SVG cache salt; MUTOOLTMP), #887 (NOTCANCEL), #888 (FLEETTMP: no temporary file outlives a conversion in the fleet), and the Windows-suite PR (`fix/windows-suite`: the Windows full suite green, TL 2026's subcaption first aid and `\caption@subtypehook`; brief `release/WINDOWS_SUITE_TRIAGE_2026-11-16.md`); lines closed: NOTCANCEL, MUTOOLTMP; the cutoff follows these.
- **The BM Q&A rows** (QA01-QA93, `~/data/pk_agents/main/bruce_qa/ACTIONS.md`, which keeps each row's text and evidence; status and rank live here) are all catalogued: the open ones are the QA lines below (QA60 site 8 as SR4/SR5, QA27 within CSS, QA75's rest as QA75 and QA75M); their pre-merge work done (QA41 → RV1 and PSF1, QA46/QA47 → QA47A, QA60 site 2 → TP3, QA75's short look, the eight upstream patches QA08, QA35, QA36, QA39, QA50, QA56, QA62 and the Perl sides of QA30 and QA45 — submitted as QA80; QA92 awaits QA02's rest); not listed — 28 done (QA01, QA02, QA03, QA07, QA09, QA10, QA12, QA13, QA18, QA25, QA30, QA31, QA33, QA34, QA40, QA42, QA45, QA54, QA55, QA59, QA67, QA70, QA71, QA82, QA83, QA87, QA89, QA93) and 10 that need no work (QA04, QA16, QA21, QA38, QA49, QA65, QA68, QA72, QA74, QA78: confirmed or noted).
- **Detail per row:** the batch reports in `~/data/pk_agents/main/agent_reports/`, the BM Q&A rows in the Q&A tracker, each deferred finding's `_red.tex` (`moves:` names its row), the moved SYNC rows' detail in [`perfect_kernel/OPEN_ROWS.md`](perfect_kernel/OPEN_ROWS.md) "Post-merge rows moved from SYNC_STATUS (SYNC3)" under the same id. DG's pragmatic bar (2026-11-12) still sorts new findings: regressions and high-frequency / high-impact gaps first, new edge cases into this list.

  **The worklist** (557 items on 2026-11-23: 553 open, weight 1082, 59 without a weight — counted as the `  - ID` bullets from here to "Silent-loss inventory", a trailing `; N` the weight; id — task; weight — easy 1, medium 3, hard 5):


  - ODR1 — OD #973's pgfmath residuals (moved from OD by the 2026-11-14 rationalization): string operands, units after array primaries, grouping, out-of-bounds indexing (repro `graphics-tikz/pgfmath_string_and_units_in_arithmetic_red`; every case's pgf-vs-ours values: OPEN_ROWS ODR1); 3
  - ODR2 — OD #976's tabularray `preto`/`appto` unwarned losses (inline expl key, the inner key's two-argument form, empty `preto`/`appto`, the `\SetTblrInner` order; repros `alignment-bindings/tblr_inline_expl_command_in_a_key_red`, `tblr_preto_in_a_two_argument_cell_key_red`; detail OPEN_ROWS ODR2); 1
  - ODR3 — OD #1019's mark-led names: REDs `article_mark_led_unanswered_trailing_mark_uncounted_red` and the two siblings; 1
  - ODR4 — OD #982: a top-level `\institute` after the author block is not cleared (revDEL3b R1, RED); 1
  - ODR5 — KPE #1267 (formerly OD #327) box sizes, K18 step 2: `\raisebox`'s `[height][depth]` with `\height` a 0pt stub, `\resizebox{1em}` 10.00002pt vs pdflatex 10.00081pt, makecell/diagbox string sizes (RED repro `boxes-groups/raisebox_optionals_measure_the_box`; detail OPEN_ROWS ODR5); 3
  - ODR6 — OD ruling 12 (recorded in OD #449): `dump_reader.rs`'s "add-only" docstrings (:365, :574) corrected, then a removal trial of the skip lists against CLAUDE.md rule 2; 1
  - ODR7 — OD ruling 6: each "Bindings beyond Perl's tree" body (end of OD) into its binding module's `//!` doc comment, then trim the catalog to its table — the 19 CLSFM bodies ODDOC-E moved there from #460 (#412 … #912) included; 3
  - ODR8 — OD docs: #8 (the OML slot-127 claim unconfirmed in code), #26 (mdframed now inserts through `insert_block`, `mdframed_sty.rs`); 1
  - ODR9 — wire `tools/perfect_kernel/od_anchors.sh` into `tools/lint.sh`; 1
  - ODR10 — the two Rhai `Fatal()` paths #870 left dropped (OD #1028; RHFAT, revRHFAT): a register GETTER (`values.rs` `.ok()?`; `Definition::value_of` returns `Option`, 720 callers; Perl `<p>Before </p>` + 1 fatal) and a `fontTest` Fatal in `set_node`'s close (document.rs:2855; Perl's `setNode`, Document.pm:74, dies) — REDs `rhai-bindings/register_getter_fatal_continues_red.tex`, `rhai-bindings/fonttest_fatal_closed_by_set_node_red.tex`; DG 2026-11-15 (RHFAT): stash the getter's and `set_node`'s close Fatal as `take_raised_fatal` does and stop at the next digest step, no API change — post-cutoff; 3
  - OD83 — OD #83 (DG 2026-11-15): keep digesting acmart's `\Description`, but report an error inside it as a Warning (acmart.cls:895 discards the argument, so pdflatex never meets it; 2607.21760's `\D1 … \D5`; KNOWN_PERL_ERRORS #66 point 2) — post-cutoff; 1
  - PGF259 — OD #259 (DG 2026-11-15): one rule, pgf's `\divide`, for every native division by zero — `pgfmath_divisor` (pgfmath_code_tex.rs:208) returns the dividend as `pgfmath_divide` (:235) does, not EPSILON — post-cutoff code + guard rows; 1
  - PGF1007 — OD #1007 (DG 2026-11-15): follow pgf — an unparseable expression raises pgf's "Unknown function" Error and computes 0; `calc_compat_expand` (pgfmath_code_tex.rs:2247) retired — post-cutoff, with an A/B sizing the papers that lose a value; 3
  - DEFER — label forms deferred too (DG 2026-11-15): typeset where printed through `defer_print` (stomach.rs:2778, :2828), one rule for every deferred form (index, glossary, bibitem, label); retires #748's `\ensuremath` retry and the 2026-10-02 option A (#418) — post-cutoff, MED, with an A/B (#778, #845 shapes); 3
  - NATBIB — #123 (DG 2026-11-15): faithful to pdflatex's decision boundary — the citation mode decided ONCE, as early as our stages allow, and recorded as metadata (an XML attribute or a processing instruction) where post-processing needs it; replaces the per-cite post-pass (crossref.rs:2182 `make_bibcite`) — post-cutoff code target; 3
  - PANEL — #885 vs #138's `sole_graphic_width` (latex_constructs/mod.rs:5964) (DG 2026-11-15): wait for #193 — the entries stay separate as today's code; consolidate, retiring #529's and #339's workarounds, when #193 lands; 1
  - IOPEAD — old #808 kept as a target (DG 2026-11-15, ODDOC-D Q2): iopart's `\ead` goes to the author its `\address`'s mark names (RED `iopart_ead_under_its_address_mark_red`; #805's chain) — post-cutoff frontmatter work, by reach; 3
  - REGC — regression C (DG 2026-11-15, ruling (b)): GREEN since REGCDE r2 — at `\begin{document}` every `\@preamblecmds` command except Perl's 7 loaders is `\@notprerr` (latex.ltx:9521-9522; `loader/preamble_only_command_is_notprerr.tex`, OD #226). Left: two synthetic REDs, catalogue only by the pragmatic bar — `loader/loader_is_not_notprerr_red.tex` (the ruling's own cost: scrlttr2's `\ifx\RequirePackage\@notprerr` in the body) and `loader/declaremathoperator_is_preamble_only_red.tex` (amsopn's `\@onlypreamble\DeclareMathOperator`); 1
  - CT1b — CT1b; 1
  - CT1d — CT1d; 1
  - HG2 — HG2: 182 DEL1-kept guards + a no-content-assertion fence; 3
  - FNT1 — 2108.12618: each author split into two creators (nowfnt); 3
  - RB1h — RB1's hyperref condition; 1
  - SMB1 — Separator meaning B1: engine snapshot; 3
  - SMB2 — Separator meaning B2: contrib helper; 3
  - SMB3 — Separator meaning B3: emitter `before` from the ink; 3
  - SR2 — SR2: a marked net note on the author block keeps its marks/markup or routes to the authors (SR1b open a; 2405.12817; pre-existing); 3
  - SR3 — SR3: the block note keeps _net on the creator route (floor + Warning; SR1 open c); 1
  - SR5 — SR5 (QA60 site 8): registers set only at \maketitle lose their notes uncounted (RED net_title_note_on_the_authors_set_at_maketitle_is_lost_red); 3
  - TP2 — TP2: title_and for pasj00/interact/l_aa/jmlr; 3
  - ARAB — \arabic etc. on an undefined counter (DG ruling); 1
  - FOC1 — FOC1: 0-wide foreignObject (\rlap in a node) browser render check; CSS nowrap if text breaks (DTL1a Q7); 1
  - MATHR — The math silent-diagnostic RED row; 1
  - RAWS — On-disk raw-class dependency-scan RED; 1
  - RAWDEFR — raw `.sty`/`.cls` reading as the default (#910, OD #1080), papers whose output moved: 1502.07025
    (ut-thesis, a local stephenpack.sty, tikz/tikz-cd/ytableau) 123× `malformed:svg:g isn't allowed in <ltx:equation>`,
    21 unmatched `</svg:g>` and `{preliminary}` undefined, errors 7 → 147, though the output under Perl's loading was
    nearly empty (11 words → 37,878; RAWSVGEQ); 1401.8205 (copernicus.cls raw) loses its Received/Revised/Accepted dates
    (RAWCOPERNICUS); 0801.1806 (iaus.cls raw) warns `\end{document}` inside a group (level 1) and drops its `[Author
    (year)]` bibliography labels — check against the PDF (RAWIAUS); from the RAWDEF study: codedoc (a TL manual)
    Fatal:Mouth:Recursion, 2309.07511's ctex a new Error (the fandol fontset unavailable), circularglyphs' TikZ text on
    a path lost, 2312.09266's background.sty notice moved. Root-caused 2026-11-23: none of it reaches the fleet (`--preload=ar5iv.sty` already loads raw; rc2 ar5iv = default counts). RAWIAUS labels: faithful — iaus.cls:1221 `\@biblabel` is empty and the PDF prints none (drop it); its group warning and stray "n" are fixed by MTARG (v0.8.0, OD #265: the deposit and replay of an argument-taking `\@maketitle`; ~1,665 iaus.cls papers, 0801.1806 0 Errors / 4 Warnings). RAWCOPERNICUS: faithful too — copernicus.cls:493 discards `\received` etc. without the logo file, but FIDELITY_CHARTER §2 keeps authored text: a raw setter that discards its argument should hand it to the frontmatter under OmniBus's date roles (`title_page_check.rs` ~:440; HARD, ~411 copernicus papers). 1502.07025: a gain (11 → 37,878 words) whose 147 Errors are YTABMATRIX; 3
  - MTARGDELIM — (MTARG reviews, synthetic, no arXiv witness) residual shapes of an argument-taking class `\@maketitle`
    (OD #265), each losing title-page text that only an Info `frontmatter:class_text` counts, where the ruling wants a
    Warning: (a) a delimited `\@maketitle` (`\def\@maketitle#1.{…}`) whose replayed `\maketitle` is dropped for an
    error gets no deposit (repro `sectioning-frontmatter/maketitle_delimited_argument_class_keeps_its_text_red.tex`);
    (b) a class `\maketitle` that calls `\@maketitle` only when `\@title` is set replays on emptied fields, typesets
    nothing and is kept, so no deposit runs (fix: decide from the speculative run on the real fields,
    `note_title_page_print`, whether it called `\@maketitle`); (c) an `\@latex@error` inside `\@maketitle` on an empty
    title errs again in the fallback. Also: a second `\maketitle` prints nothing where pdflatex repeats the title page,
    and `\lx@saved@maketitle` stays defined after `\maketitle` (harmless). rc2 kept (a)-(c)'s text with the open-group
    Warning; 3
  - SVA — SV-A: \csname via \ifincsname; 3
  - TBLR — tabularray first-token, \g__tblr_table_count_int, tblr-only \hline REDs; 1
  - QA05 — Kernel-version switch + \@ifundefined per year (QA05/QA06); 1
  - QA11 — the native pgf/l3fp inventory's rest (done 2026-11-14: `agent_reports/report_QA11.tsv`, probes `scratch_QA11/`; l3fp native 0/72k mismatches and pgfkeys 0/33 KEEP, raw pgfkeys +5.6…+16.9 %; pgfmath 34/90 differ from pdflatex, 29 Perl-origin; Root A — expandable closures run inside TeX's optional-space scan after a unit, tex.web:8869, 2606.14412's 55 false E, repros `scratch_QA11/rc/rc.tex`, `rc2.tex` — is QA11A, PR #872 merged): Root B, a dimen register above `\maxdimen` raises no "Dimension too large" (gullet.rs:4719-4734 skips tex.web:8870-8872's attach_sign check; `scratch_QA11/k1/k3.tex`; Perl the same; an Error-boundary batch, census first); DROP-FOR-RAW after an A/B: `\pgfsetdash` (pgfsys_latexml_def.rs:996), `\pgfmathsincos`, candidate `\pgfmath@smuggleone`; native pgfmath misses TeX's Errors on `123456789/1000000`, `-1e5/10`, `1/0` (OD #259), `"a"=="a"` — fold into OD #1012; 3
  - QA11AR — QA11A's residuals (PR #872; QA11A, revQA11A): `\pgfmathsetlength{+…}`'s quick path `\unskip`s the space typeset before the call in pgf, ours and Perl keep it (RED `graphics-tikz/pgfmath_setlength_quick_path_unskips_red`); the same path skips spaces before putting back its leftover tokens, pgf the other way round, output identical (pgfmath_code_tex.rs:2783); `\pgfmathpostparse` not ported (the raw parse calls it before returning); a pgfmath primitive inside an `\edef` (`\edef\x{\pgfmathparse{1+2}}`) raises no Error where pdflatex errs and stops (Undefined + Incomplete `\iffalse`; base 1 E; RED `graphics-tikz/pgfmath_parse_in_edef_errs_red`; reach ≈ 0); 1
  - QA14 — the length census's K2 LIST/ADJW (done 2026-11-14: `agent_reports/report_QA14.tsv`, probes `scratch_QA14/probe/`; 918 cells, 523 exact, 395 wrong, all SHARED with Perl but two; K1 PARBOX is QA14A, PR #873 merged; K3 is QA14K3): `\list`'s register arithmetic (latex.ltx:15848-15869) and article.cls:323-339's `\leftmargini..vi`/`\labelsep` are 0 (sect06.rs:656; Perl :1303, :1649) — ≈530 papers sizing by `\linewidth` in a list 7-20 % too wide, ≈550 MDPI `adjustwidth{-\extralength}{0cm}` figures stay 345pt; 3
  - QA14K3 — twocolumn/multicols geometry (TWOCOL/MULTICOL): today one 345pt page for every class (Perl's halving, article.cls.ltxml:33, undone at `\begin{document}`, latex_constructs.pool.ltxml:321; ours sect02.rs:144; latex.ltx:9481-9485's `\if@twocolumn` dropped) where pdflatex's `\columnwidth` is 229.5/252/246pt (article/IEEEtran/revtex; ≈346k papers' column images 1.4-1.5× print size, `figure*` 0.67-0.73×; `{multicols}` keeps full width, multicol.sty:286-291, ≈1.25k papers, 2×). DG 2026-11-15: follow TeX's geometry, but "tricky, because of the impedance mismatch between TeX sizing for the PDF output and our expectations of theming and reflowing the HTML output on any device width" (ar5iv.css normalizes image sizes) — post-cutoff, design first: a study of ar5iv.css's normalization and 2-3 designs (e.g. sizes relative to the container, a fraction of `\columnwidth`/`\textwidth`, keeping TeX's proportions without breaking reflow) before any code (`agent_reports/report_QA14.tsv`); 5
  - QA14K4 — the length census's cosmetic K4: `\parindent` 20pt vs size10.clo:90's 15pt (tex_paragraph.rs:443), `\@arrayparboxrestore`'s zero; `\vsize` never `\textheight` (latex.ltx:9478); `\marginpar` keeps `\linewidth` (latex.ltx:17603); footnote `\baselineskip` 12pt; 1
  - FLTW — float widths not modelled as `\@xfloat` models them (pre-existing, base the same; revQA14A, revQA14Ab): `begin_float`/`before_xfloat` (latex_constructs/mod.rs) set `\hsize` but not `\@parboxrestore`'s `\linewidth\hsize` (latex.ltx:17482-17483, float.sty:85; `\@xdblfloat` :17556 `\linewidth\textwidth`) — `\parbox{0.5\textwidth}{\begin{figure}[H]…width=\linewidth…}` 172.5pt vs TeX's 345pt (`scratch_revQA14A/p/p1.tex` row T); wrapfigure's `\columnwidth` and sidewaysfigure's registers (`\linewidth` 345 vs 550pt; rows K, L); floatflt `\hsize` 345pt vs TeX's 4cm; twocolumn `figure*`/`table*`/`algorithm*` read 345pt vs `\textwidth` 469pt; IEEEtran `\subcaptionbox[0.48\columnwidth]{…width=0.45\linewidth…}` loses `panelline`/`panelwidth`, a `\columnwidth` graphic gets `panelwidth` 0.491 vs the PDF's 0.48 (`p5.tex`, `p6.tex`); `before_xfloat` callers that do not model `\@xfloat`: acmart teaserfigure/marginfigure/margintable (`\Collect@Body`, acmart.cls:991-1004, :2202), JHEP `@half@double*` sub-panels, JHEP floatingfigure vs floatflt_sty.rs's `before_float` (`scratch_revQA14Ab/p/q1-q7.tex`); 3
  - QA15 — TeX's two kinds of unread: RED + reach; 3
  - QA19 — the schema-extension inventory for BM (PR #874 merged, with QA85): two schema changes with no OD entry — abstract/acknowledgements `Labelled.attributes` (from the arXiv fork, `8d9189f7e46`) and `role_side` (planned internal, `docs/math/MATH_PARSE_RESIDUALS.md`) — and S9's, decided after the OD rewrite (SCHEMA_EXTENSIONS.md lists them); the compiled `LaTeXML.model` still lacks `role_side` on `XMHint`/`XMRef` (checked on main 2026-11-23) though the `.rnc`/`.rng` have it: recompile or explain (QA1985b; ODDOC-B2); 1
  - QA22 — Page furniture kept as metadata (QA22/QA23, census done): element designed with BM, implemented; 5
  - QA29 — xml:id nesting check; 1
  - QA43 — the binding-retirement census's rest (done 2026-11-14: `agent_reports/report_QA43.tsv`, probes `scratch_QA43/`; of 434 `*_sty.rs`, 173 keep (markup), 94 `DefMacro` copies (86 already raw-load), 60 stubs (41 with no TL 2025 raw file), 1 perf shim; 40 probed; the 16 RETIRE-SAFE kept — times 5101 users, mathrsfs 4426, xspace 3652, balance, placeins, lmodern, stfloats, helvet, fullpage, eucal, courier, indentfirst, euscript, etex, afterpage, overpic 376; raw better for xspace before `\footnote`, eucal/euscript, overpic; each binding ≈7-18M instructions on a ≈1.0G floor, 0.7-1.8 % — DG 2026-11-15 deferred their retirement until after the release, to revisit with real-paper cost measurements): 11 NEEDS-WORK — raw geometry `Error:unexpected:\fi` after "detected driver: pdftex" (8.3k users; `scratch_QA43/probes/geometry/raw2`, pdflatex clean; a kernel gap if a paper ever raw-loads geometry), microtype `\MT@setup@*` undefined, fancyhdr/textcase/footmisc/newtxtext cost; the newtxtext binding leaves `\textsu`/`\textlf`/`\textosf` undefined; 3
  - QA47R — the kernel hooks' residuals (#856, #862, #869; QAB1, revQAB2b): the label hook runs after the label is recorded (`macro-state/label_hook_sets_the_reference_text_red`); a DefEnvironment's begin hooks read the enclosing environment's name (`macro-state/environment_begin_hook_reads_its_environment_red`, 2001.02707; needs the end-side frame check too); `\@currenvline` never set (`currenvline_is_never_set_red`); `\ShowHook` needs `\tex_showstream:D` (`show_hook_needs_showstream_red`, Perl the same); `\emph` calls no `\selectfont` (`selectfont_hook_on_emph_red`); the kernel abstract never applies article's `\small` (`abstract_begin_hook_size_under_the_class_small_red`); a begin hook's typeset text joins the abstract's first word where amsart/revtex4-2/elsarticle print it before the title (`frontmatter_api.rs:606-615`; `macro-state/amsart_abstract_begin_hook_text_before_the_title_red`); a family register as an expanded value (`\csname textfont\endcsname 1`, `\def\tf{\textfont}…\tf1`) still prints its number (gullet.rs:4162-4164; `fonts-nfss/family_register_value_expanded_red.tex`, Perl the same); cost: initialize_all at `\begin{document}` (latex.ltx:15360) — every body `\UseHook` re-initializes (0.7M instructions; content.rs hooks 1-3M each), the unused undeclared one-time hook's clear code (OD #1048), 2,000 `\IfHookEmptyTF{label}` under cleveref +25.7 % (TeX's own cost); NIT `sect13.rs:1154-1156` `hook_use_tokens` re-checks the store; 3
  - QA61 — Unknown classes' title-page stores as frontmatter items — DG 2026-11-15: **D1, a labelled pubnote** (`<ltx:pubnote name="Course: ">MATH 410</ltx:pubnote>`, the centred body line removed; title/author stores under other names kept out of pubnotes — SelfArx `\PaperTitle`, smj, Epigenetics, ametsocV6.1 `\authors`; glued labels, fundam's DOI before `\publyear`, 2006.07415; HTML visibility through PNOTEA's titleless head block, #881); the batch in flight (study `agent_reports/report_QA61D.md`, 58 of 2,929 raw-class papers, ≈5.4k arXiv); 3
  - QA69 — an encoding with no fontmap decodes as OT1 (`tex_fonts.rs:228-261`; SHARED with Perl `TeX_Fonts.pool.ltxml:169-176`), told only by `Info:missing_font_encoding`, so a wrong glyph is uncounted: T5 (≈480 papers; a wrong `"`/`<`/`>`/`|` in ≈240: 2406.15888, 2511.17955, 2605.10376), L7x, IL2 (pdflatex agrees) and U (harmless); below the bar, no shipped map — plan (HARD): derive `<ENC>_fontmap` while `<enc>enc.def` is read (`\DeclareFontEncoding`, sect08.rs:1127), seeded with encguide §3.1's ASCII slots, symbols from bound LICRs or the inverse of `<enc>enc.dfu`, composites by NFC, U excluded (per font); RED `unicode-catcodes/qa69_t5_ascii_slots_red.tex` (pdflatex `an "overrider" a<b>c |x| "<| “q” "<`); 5
  - QA76 — the upstream-issues survey's post-merge rows (done 2026-11-14: `agent_reports/report_QA76.tsv`, repros `scratch_QA76/min/`; 314 open, 60 run: 45 fixed in oxide (7 only under raw loading), 12 same as Perl, 1 oxide-worse; #2263 llncs keywords and #711 apacite `\fullcite` are QA76A, PR #871 merged): upstream #2714 `\ExplSyntaxOn` across `\begin{document}` — sect02.rs:356-368 forces `\ExplSyntaxOff` at `\begin{document}` where latex.ltx:18364-18378 restores per file (E=7 vs Perl/pdflatex 0; 0 of 442 census users; RED `m2714a.tex`); #2797 `\cellcolor` in a nested tabular colours the outer cell (≤64k); #2417 listings `morestring=[b]` `\lstinline!B!` prints "BB" (invented, ≤12k); #2674 thmtools `sharenumber=`+`numberlike=` (≈230); #2084 combelow raw (≈1k); #2371 siunitx `\micro` invisible times (cosmetic); #2096 amsart `\thanks` → acknowledgements (markup); #1763 block content in `\marginpar`; 3
  - QA76AR — QA76A's review residuals (PR #871; revQA76A): apacite `\citeA`/`\citeNP`/`\citeyearNP`/`\shortcite*` defined in every citation mode where apacite.sty defines them in classic only (:258-584) — RED in QA76Ab; svmult v5.12 `\keywords{A \and B}` after `\maketitle`: pdflatex 19 `!` (kernel tabular `\and`), ours 0 E — RED; unchecked: svproc/webofc/wileymsp keyword `\and`, natbibemu mode; 1
  - QA90R — QA90's residuals (#867): latex.ltx's token preamble-only macros (`\makeindex`, `\makeglossary`, `\DeclareTextCommandDefault`, …) stay silent in the body (`loader/makeindex_in_the_body_red.tex`; the loader runs latex.ltx's filename-stack macros in the body; reach ~0), as do body token macros listed in `\@preamblecmds` (`\makeindex`, a document's `\@onlypreamble\myonly`: `loader/document_onlypreamble_macro_in_the_body_red.tex`); the walk's recovery (Perl's: Error, then run) is wrong for commands whose setting later commands read — `\includeonly` in the body drops `\include{cb}` where pdflatex ignores it and prints both (`loader/includeonly_in_the_body_red/`): (a) defer, (b) a small list of commands that only report the Error; 1
  - QA91R — QA91's and DEPN's residuals (#859): the wrap reads the title and author marks, one answer per block, so a class printing its own title store, or the title and the authors apart, is placed wrong (`sectioning-frontmatter/text_before_maketitle_where_a_class_prints_its_own_title_store_red`, `…_where_title_and_authors_print_apart_red`); `\date{Spring 2024}` printed twice in the deposit ("Spring 2024A=0", gullet.rs:2670 `unread(value)`); a float between the paragraphs before `\maketitle` splits them; `\maketitle` inside `{titlepage}` leaves the cover text after the title (base too); stray `*1`, `]`, `,` more visible at the top (QAB4b); 3
  - SR4 — SR4 (QA60 site 8): a title note held in \title (\titlenote) becomes a pubnote (1601.00073, 1805.02241, cs0405006) — with the page-furniture element (QA22/23); 1
  - QA17 — `\iffontchar`/`\fontchar*` read `<font>` as a plain Token (etex.rs:620, :253; tex.web §577): `\textfont1 "80` typesets `”80`, `\nullfont` answers Y; the NFSS current font has no file, so the answer is always TRUE, and the identifier's file ignores the encoding (tex_fonts.rs:858; latex.ltx `\get@external@font`:12538 gives `ecrm1000`) — fix: a `FontToken` + `\nullfont` = empty (EASY) together with the identifier's file from `\get@external@font` in `pickup_font` (MED), since the first alone makes T1 wrong; limit: the font-package family stays CM (sect13.rs:907); RED `fonts-nfss/qa17_iffontchar_red.tex` (synthetic; pdflatex `[a:N][b:N][c:N][d:Y][e:cmbxti10]`); reach 0 author uses in 35,942 papers (12 bundled package copies); 3
  - QA20 — Open/close vertical bars (VERTBAR read as MIDBAR): a math RED row (math parked); 1
  - QA26R — QA26's residuals (#854; QAB8): `\addvspace` is a no-op (`paragraph-mode/addvspace_between_paragraphs_takes_the_larger_red`); two `\vspace`s in a paragraph end it at the first (`paragraph-mode/two_vspaces_in_a_paragraph_stay_in_it_red`); a mid-paragraph `\vspace` splits the paragraph where TeX keeps one (Perl too; ~2 % of papers; its own mechanism); 3
  - QA27 — Lengths in em of the parent: the gradual migration after the sizing CSS; 3
  - QA28 — BM's question on our animation-frame / RDFa example clarified; 1
  - QA32 — Semantic SVG drawings: tikz-cd and xymatrix nodes and connectors labelled; 5
  - QA37 — Side-by-side figures (C1): BM's sibling figures vs ruling 57g (DG: deferred, "requires care"); 3
  - QA44 — Thin the remaining bindings family by family (raw package + semantic overrides); 5
  - QA48 — LaTeXML's own hooks onto LaTeX3 hooks (after QA47B); 5
  - QA51 — Macro faces: the `\meaning` census (sized: 241 macros read as primitives, 276 other bodies); 3
  - QA52 — Our reply to BM's "What do you think?" on macro faces; 1
  - QA53 — Macro faces in the definers, subsystem by subsystem, an A/B each, proposed upstream; 5
  - QA57 — Horizon: one binding language for Perl, compiled Rust and rhai; 1
  - QA58R — #858's residuals (QA24, QA58, QA64; QAB57): `\parbox[t][3cm]{4cm}{…}` and `minipage[t][2cm]` lose their height in the XML, Perl too (`block-model/parbox_and_minipage_height_dropped_red`); a framed box holding a `\section` draws its frame as an empty box above it (`block-model/framed_box_section_frame_collapses_red`; main too); a shipped `\usepackage{mydefs}` that defines `\email` still loses to OmniBus's guess; a document's hook definition refused under OmniBus (`singletons/omnibus_document_hook_definition_refused_red`); a latexml option in the document after `--preload=ar5iv.sty` (`backend-persona/latexml_option_after_ar5iv_preload_red`); 1
  - QA63 — XeTeX / LuaTeX emulation modules by the intended engine; 5
  - QA66 — Each Error's site marked in the output (`ltx:ERROR`), designed with BM; 3
  - QA73 — `ELIDEOP` renamed to an elision role, together with Perl; 1
  - QA77 — A Rust bst-driven BibTeX keeping each field's meaning (JATS, RDFa); 5
  - QA79 — LaTeXML 0.9 as the reference on its release: oracle re-baselined, parity re-run; 3
  - QA81 — A latexml-oxide home page (Sammy in the logo); 3
  - QA86 — A principled organization shared by both implementations, proposed to BM; 3
  - QA88 — Horizon: fully idiomatic Rust, the parity ties broken (DG's call); 1
  - P6TA — the token address readers (`line_addresses` at affiliations.rs:120/558/603, `run_in_author_order`, kind.rs's `prints_addresses_only` / `line_addresses`) onto the reading: give the emitter's per-item events to `\lx@frontmatter@itemkinded` (P6f's open item); 3
  - IEREG — IEEEtran `\IEEEauthorrefmark{1}\the\toks` / `\csname annmail\endcsname` address line: 1 Error, an invented e-mail "0", the address lost, counted (the token e-mail line reader; `ieeetran_authorblocka_register_address_line_red`; P6f); 1
  - BLET — beamer `\let\sep\and` inside the `\institute` list: 1 Error, a raw `\sep` left in the text, base and P6f2 alike (`beamer_institute_let_of_and_inside_the_list_red`; P6f2); 1
  - BAND — beamer's `\and` that pdflatex prints as a line break inside parentheses becomes the word "and", counted by its Warning (`\institute{\inst{1}Univ A (North \and South) …}`; revP6f p1); 1
  - ORCR — acmart `\orcid` between two `\author` calls ends P6e2's run of names, the shared block's first e-mail on the last name (`acmart_orcid_between_shared_names_email_on_last_red`; revP6e2 finding 1; no corpus witness); 1
  - P6E2G — a guard row for P6e2's note path (Wiley `\corres{…\email{…}}` after two names: a raw `ltx:note` holding `\email`; 8 corpus witnesses, no synthetic row; revP6e2 finding 2); 1
  - ACOE — the content check's false "placed more often than printed" Warning for an address printed under two own calls (`acmart_own_email_call_stays_whatever_it_spells` pins W=1 on int168a, int169a, P6e2): a RED catalog repro; 1
  - MARKTXT — a mark's text kept in a name: PoS's first name (`pos_leading_mark_kept_in_the_first_name_red`), an unanswered trailing mark uncounted (`article_mark_led_unanswered_trailing_mark_uncounted_red`), nosuchclass "1 Ann Able", `$^1$Ann \and $^1$Bob`, an unanswered mark moved after its name (CL170 R2); 3
  - ISO — an unrequested isotope line loses its mark (`article_unrequested_isotope_line_loses_its_mark_red`; MLN1); 1
  - ROW1 — a mark-led second names row of one name read as an affiliation, a silent partial loss (`article_mark_led_second_row_of_one_name_read_as_affiliation_red`; revMLN1 2); 1
  - GRPA — an unmarked name inside a mark-led row takes its group's affiliation by the declared convention (1405.2400; revMLN1, the charter's step 3); 1
  - PTPM — ptptex `\author` marks after a comma consumed, uncounted (CL170 R1; probe `scratch_CL170/probes/ptp_a.tex`); 1
  - UNWE — D2's unwrapped e-mail command never runs its other effects (`raw_class_store_email_command_effects_lost_red`; revP23e 3); 1
  - XYZC — `\author{Ann\inst{1}, Bob\inst{2} \and the XYZ Collaboration}`: the institutes' e-mails on a nameless creator (`aa_comma_names_and_collaboration_emails_on_the_block_red`; revP23b 3); 1
  - LLCM — llncs's comma-parted marked list: the e-mail line the block's (`llncs_comma_parted_marked_list_email_line_is_the_blocks_red`; P23b); 3
  - SVMH — svjour3: a macro holding names and the separator, the e-mail line the block's (`svjour3_marked_list_macro_holding_names_and_separator_red`; P23b); 1
  - RAWL — a raw class's in-place `\email` label repeated per contact / its prose (`raw_class_in_place_email_label_under_numbered_institute_red`; revP23c S1); 1
  - AACM — aa `Bob\inst{2}$^{,}$\inst{1}` keeps the comma in the name (revP23c l4); 1
  - LLGL — llncs `\institute` legend line glued into the last institute (revP23c l5); 1
  - AFNR — two identical `\affiliation{…\footnote{…}}` write two pubnotes, no Warning (`aastex6_footnote_in_repeated_affiliation_doubled_red`; AAS1); 1
  - AAGR — AAS1's guard: a repeated affiliation content that must NOT be reused (`\stepcounter`/`\footnotemark`, A A B A) (revAAS1 NIT); 1
  - CSSPORT — The ar5iv-css port: PR #54 merged 2026-11-05; the mirror of latexml-oxide's later `LaTeXML.css` fixes in ar5iv-css #55, #56 (#841's aligned columns), #57 (#844's verbatim, display runs, ORCID), open; 3
  - MHCHEM — a semantic mhchem binding (DG 2026-11-13: deferred, planned; Perl ships none; stage 1 merged as #850, its rest MHCHEM-BM; #837 prescripts and #838 arrow heads fixed the raw path's isotopes and `<=>`): option (a) of study `agent_reports/2026-11-13_MHCHEM0_report.md` —
    a thin binding over mhchem's output layer (`\__mhchem_output_coreFive:nnnnnnn`, mhchem.sty:2774; amounts, `+`, arrows, bonds), keeping its parser:
    element tokens (the run split on `[A-Z][a-z]*`, mhchem keeps "MoO" one slot), `mmultiscripts` for isotope/count/charge (charge lifted to the
    formula, minus U+2212), arrows as relations, one `<math>` per `\ce`; ~0.9 % of arXiv (~13k papers; 6,268 calls in the 30k census: 53 % counted
    formulas, 14 % isotope prescripts, 7 % charges); raw path costs ~0.22 s per `\ce` (148 → 34.8 s) — a perf item first; issue comments #824/#825
    record the detail; 5
  - MHCHEM-BM — the mhchem binding (stage 1 merged as #850, DG's tentative A0 markup) awaits Bruce Miller's feedback and refinements; stage 2 (MHCHEM4): arrows, `+` between species, bonds, amounts, the charge's meaning, MathML intent annotations on the presentation branches (DG 2026-11-14); a braced `{\mathop…}` typed as Op; signs in script style keep Core's Bin/Rel/Punct space (MSPC's `script_style_rel_has_no_thick_space_red`); 3
  - REL — A release; 3
  - G — Run 336's open residuals (the arXiv success-rate task); 5
  - G1 — Run 329's open residuals (PushbackLimit, TooManyErrors, the `handle_template` panic, …); 3
  - A2 — Fidelity audit round 2: the title-`\cite` tooltip (12); 1
  - A3 — arXiv html_feedback classes (HF1-HF15); 3
  - A4 — The full arXiv fidelity audit (user 2026-10-10: last); 5
  - AT — AutoTeX emulation (the main file, supplements); 5
  - TLY — A target TeX Live year per conversion (with QA05); 5
  - KC — Kernel capability program K20-K23, C3/C6/C7; 5
  - MS — TeX's math-group recovery, items (1)-(5); 3
  - BD — A `\DeclareBoldMathCommand` delimiter after `\left` (1205.4484, RED); 3
  - D1 — Rulings pending: `\obeylines` `^^M`, `{framed}` items, biblatex anchors, R7 port-or-drop; 1
  - T2 — Test API stage 2: 37 test files still spawn binaries; 3
  - GTXT — GeneralText/XGeneralText arguments revert without braces (`\mark{x}` → `\mark xb`; Perl `\mark{x}`; RED `reversion/mark_general_text_reverts_without_braces_red`); 1
  - SHIT — `\shortintertext` reverts as `\intertext` (mathtools `\let`; the macro-face design); 1
  - PSFR — psfrag replacements rendered into the figure (after PSF1's Warning); 5
  - PGRE — pgfmath's parse-failure Error (EB1, held by PGR1): fires on 3 pdflatex-clean papers by three mechanisms (1612.06541 tikz-cd anchors under its shipped pgf 3.0.0; 2006.16998 `\d` in `\pgfextra`; 2208.01735 collectcell + `\IfDecimal`); leftover text dropped silently until then; 3
  - PGU — pgfmath unit conversion in scaled points (`2bp` 2.0075 vs 2.00749), pdfTeX's `px`/`nd`/`nc` and `mu` units (RED `pgfmath_units_as_tex_reads_them_red`); `08+1` octal (RED `pgfmath_octal_bad_digit_red`); 3
  - PGCNT — a count register then a dimension (`2\myc\mydim`: pdflatex 15.0; RED `pgfmath_count_then_dimension_drops_the_dimension_red`) (revPGR1); 1
  - QAHK — a `cmd/` hook's owner read from its label: a package's body hook a Warning, a document's hook labelled with a loaded package an Info (REDs `command_hook_a_package_adds_in_the_body_red`, `command_hook_a_document_label_names_a_package_red`; one fix: record `\@currname` when the code is stored) (revQA47Ab); 1
  - P4bQ2 — `${\mathrm ^{1}}$` marks (erroneous TeX; pdflatex 4 `!` prints the 1): our recovery prints no superscript, so the API Cut finds no requested entry (RED `api_cut_erroneous_mathrm_marks_red`; cond-mat0206329) (P4b2); 1
  - DTLR — DTL1d's REDs: `\multiply` overflow keeps the register (`multiply_overflow_keeps_the_register_red`), pgfmath function results clamped silently (`pgfmath_function_overflow_clamped_silently_red`), acmart loads setspace outside `manuscript` (`acmart_setspace_only_in_manuscript_red`, 2409.07148; Perl the same) (DTL1d); 3
  - GPAR — group-paragraph geometry REDs: `\leavevmode{ \par}x` (P1), `\noindent{ }\par` (P2), a group's `\strut`/`\vspace`/leader or a blank piece after `\small` (P3, PE2), a glue-only formula (PE1), a forced break's last line (PE3) (RG1c, revRG1c); 3
  - PBXL — a marked list printed entirely inside one `\parbox` is one entry (`cut_list` splits at the top level only; RED `ieee_marked_list_inside_one_parbox_is_one_entry_red`) (revP4c S1); 1
  - AUMK — answered marks hidden on both sides: the kernel `\author` whose own lines answer its names' marks (`kernel_author_lines_answered_marks_hidden_red`, as Perl), its authblk sibling (`authblk_author_line_answers_one_mark_red`), authblk's own `\author[1]`/`\affil[1]` generated marks dropped (`authblk_labelled_marks_dropped_red`, as Perl) (P4b2b, revP4b2b); llncs `\inst` names whose marks a macro prints hide the institute's marks too (`llncs_inst_names_macro_marked_institute_marks_hidden_red`; P4b4a: 1803.02579, 2201.04974, and CL166's seven: 0807.2091, hep-th0502229, 2412.12966, 0706.0112, 0803.2400, 0808.1862, 2508.08560's name marks); 3
  - CALCR — calc's `read_term` does not skip `\relax` before `*`/`/`: a stray "2+1" printed (`\setcounter{a}{\value{b}\relax*2+1}`; revGA1e S2); 1
  - P4CRN — P4cR1's residuals: a name's mark in a box with a space lost uncounted (`ieee_name_mark_in_spaced_box_lost_uncounted_red`), apa6 entries' marks dropped (`apa6_marked_list_entry_marks_dropped_red`), an unmatched e-mail address on a nameless creator when local parts run out of name order (`ieee_email_run_unmatched_address_nameless_red`, 2409.10792, 2410.09414; a ruling first), `unanswered_mark_lost`'s wording for a `ref:` mark (revP4cR1); 3
  - FNMK — footnote marks: `[\relax 2]`/`[\value{nosuch}2]` read as digits, a braces-only tail silent (`footnotemark_relax_then_digits_reads_digits_red`), a blank `[]` (`footnotemark_blank_is_tex_zero_red`), `\ifnum` with a missing number (`ifnum_missing_number_inserts_equals_red`) (GA1f, revGA1f); 1
  - STRM2 — streamed-vs-eager differences under a forced switch (pre-existing; STRM): ids order (1705.02293, 2507.13358), whitespace (1005.3879, 2206.03430), footnote split (2010.06608), `toc` inlist (1907.01464), `\quad` reversion (2509.09797); and unforced: 2308.04512 streams under the production `--max-memory=8192` (PDFCMP 2026-11-15: RSS 2.09 → 0.97 GB, CPU 28.1 → 24.4 s) and its HTML differs from the eager run in 186 lines (proof ids S2.SS7.2 → S2.SS7.1, footnote class `ltx_role_footnotemark`, +26 W/E lines); the eager→streaming switch rate in production to measure; `finalize_context_font` DRY with `finalize_rec` (revSTRM); 5
  - OMNT — OmniBus/KOMA title notes step no counter and print arabic (2407.13730, 2606.19583 recalls "0" vs pdflatex ∗; `omnibus_title_footnotemark_prints_fnsymbol_red`; reach to measure) (FNR1); 3
  - FNRR — the rehearsal reads a printing definition's later use (the true signal behind FNR1's gate; `author_note_macro_resets_an_existing_counter_red`; the replay writes a note from a printing call's expansion with its markup: 2604.20516) (FNR1, FNR1c); 3
  - IEMP — a minipage's optional argument leaks into IEEEtran affiliations ("t].44", 1901.06399; invented text; int162a too; reach to measure) (P4fC); 1
  - DTLX — DTL1d's counted pre-existing overflows (varwidth → VARW): a subscript-only math box of height 0 (2503.14426; Perl too; HARD), pgfplots `y expr` (2407.10017), xymatrix/tikz shapes (1409.3605, 0704.0778, 2409.07148, 2408.05806, 2411.12316), 1103.1294's cap (TooManyErrors sooner; nicefrac in xymatrix) (CL164); CX171 (`2026-11-13_CX171_report.md`): 2606.14412 (IEEEtran + circuitikz: 55 E, pdflatex 0 `!`; QA11A, PR #872 merged: 55 → 0; a `bending` start tip on a degenerate bezier — the native `\pgfmathreciprocal@` (pgfmath_code_tex.rs:2623) reads its divisor as 0 where TeX has 22.76184, pgflibrarycurvilinear.code.tex:98; 4-line repro scratch_CX171/r/m1.tex: Rust 64 E, Perl 4 E, pdflatex 0), 2605.31088 (pgfplots log axis `declare function` at small x, 4 E); 5
  - P4FR — P4f's residuals: llncs `\inst` names + `$^n$` institute, beamer, unmarked names + marked institute (REDs `llncs_inst_names_marked_institute_marks_hidden_red`, `beamer_marked_institute_marks_hidden_red`, `llncs_unmarked_names_marked_institute_marks_lost_red`) (P4f); 1
  - P4B4R — P4b4's post-merge parts (P4b4C): Group A's other readers (`kind.rs:111/237`, `frontmatter_api.rs:935`, `authblk_sty.rs:485`, `relocate.rs:1409/1449/1626`; DRY, no shape found), Group B (what the names request: `place.rs:478/259`, `ijcai.rs:533`, `old_reader/marks.rs:148`, `frontmatter_fields.rs:641` → `queued_author_marks`; byte-identity), Group D (REVTeX "after the comma", hep-ph0301204), Group E (`MARK_SWITCHES`/`leading_line_label` read typed numbers: a typed-label event), `events/lexer.rs:529`'s stale `requested_labels` (the rehearsal memo), P4b2b's rendering residuals (math-wrapped symbol marks, two symbols before a number reordered, 2509.08362's plain-text name); 5
  - VWLD — varwidth measurement residuals: lists/displays/`\hfill` inside a varwidth (`varwidth_lists_and_displays_measure_red`), a leading `\color` keeps the given width, an inline box beside text, `\fcolorbox`/`\colorbox` measures, inline math no-wrap in a `p{}` cell (2502.13537), cryptocode text cells ~7-10 % wide (VARW, VARWb; REDs `varwidth_display_draws_at_given_width_red`, `varwidth_colour_at_its_start_keeps_the_given_width_red`, `varwidth_inline_box_beside_text_measures_the_line_red`, `varwidth_cell_beside_overflowing_math_header_red`, `colour_boxes_measure_their_frame_red`, `cryptocode_text_cells_measure_wide_red`); 3
  - CMPJR — cmpj residuals: renamed copies (cmpj2e, cmp2em: 0903.4566, cond-mat0511115) stay raw with "?"; txfonts `\re@DeclareMathSymbol` undefined; `\refaddr{nokey}` prints a dangling `\ref` where pdflatex prints "?" (CMPJ1; REDs `cmpj_renamed_copy_refaddr_prints_question_marks_red`, `txfonts_re_declaremathsymbol_undefined_red`); the address list's residuals — a group across items loses the number and bold, counted (`cmpj_address_list_group_across_items_red`), a tagged `\item[a]` keeps the list whole, counted (`cmpj_address_list_tagged_item_kept_whole_red`), `\ukrainianpart`'s second title page's names hold the first page's addresses (`cmpj_ukrainianpart_addresses_of_the_second_title_red`, 1212.6367, pre-existing), an item copied onto several names repeats its `labels=` and the HTML links go to the last copy (CMPJ2/3, revCMPJ); 3
  - MHAF — REVTeX: an `\affiliation` whose marked lines a document macro holds (`\affiliation{\Insts}`) is one nameless creator, where the inline twin reads the lines under their names (`revtex_macro_held_affiliation_lines_nameless_red`; CST2); 1
  - CSNL — a mark macro built by `\csname…\endcsname` in a name (`A.~Smith\csname mA\endcsname`): 2 Errors on REVTeX, 1 on IEEEtran ("\let should not appear between \csname and \endcsname") and the marks lost; pdflatex 0 (`revtex_ieeetran_csname_built_mark_macro_errs_red`; revCST2); 1
  - TYST — a typed `*` beside `\,` / `\ ` in a name is read as a mark by the rehearsal the `\,` triggers (nameless creator), where the gate reads typed characters as text; option (a): the rehearsal reads them as text too (`revtex_typed_star_beside_thin_or_control_space_nameless_red`; CST2b); 1
  - IEFL — IEEEtran: `\IEEEauthorblockN{A. Smith*, B. Jones}` over `$^1$`/`$^2$` lines gives A line 1 only, where the PDF prints one block under both names (`ieeetran_typed_star_name_takes_first_line_only_red`; CST2b); 1
  - P4LR — P4b4a's list-cut residuals: a `tabular`'s `\affmark` rows keep the base's owners, Ann all three lines (`llncs_institute_macro_marks_tabular_rows_red`); a box around part of a line leaves its comma and an empty `\mbox{}` (`llncs_institute_macro_marks_part_boxed_keeps_comma_red`, revP4b4a S1); `\textcolor{blue}{$^1$A, $^2$B}`'s second entry loses its colour (`llncs_institute_marks_in_textcolor_second_entry_color_red`, S2); 2608.21425's `nancyzhzhai@` e-mail loses its size (`llncs_institute_parbox_email_line_keeps_size_red`, the emitter's address cut); 3
  - P4DEC — a macro of font declarations (`\newcommand\affilfont{\small\itshape}`) is carried to the first cut piece only (int165a carried it to all; restoring that brings back the printing-macro duplication, revP4b4a2 B1) (`llncs_institute_marks_macro_declarations_lead_not_carried_red`); `FONT_DECLARATIONS` repeats LAYOUT's head and LAYOUT lacks `\tt`/`\LARGE`/`\huge`/`\Huge`/`\normalcolor` (revP4b4f NIT); 1
  - P4IAL — IEEEtran's `\IEEEauthorblockA{\centering…}` first cut piece keeps the alignment (`ieeetran_authorblocka_centering_marks_first_entry_keeps_alignment_red`); 1
  - P4SET — the lead scan stops at `\setlength{…}{…}`, a later `\footnotesize` reaching no piece (`llncs_institute_setlength_lead_stops_carried_declarations_red`); 1
  - P4ENV — an environment's size (`\begin{footnotesize}`, 2506.05954) on no piece of the list it spans (`revtex_affiliation_footnotesize_env_size_lost_red`); 1
  - P4OWN — a minipage's one marked line kept whole on a nameless creator (`llncs_institute_minipage_one_line_marks_owners_red`); 1
  - P4FNS — llncs.cls:391 `\let\footnotesize\small` missing from the binding (PDF 9pt, ours 80 %; Perl the same → KPE when fixed; `llncs_footnotesize_is_small_red`); 1
  - ACMDUP — acmart `\country{$^1$…}` with an entry no author requests: the requester of the one before it holds both, twice (`acmart_country_unrequested_entry_duplicates_red`; revP4b4f); 1
  - IEADR — short marked e-mails kept one affiliation of the first name (`ieeetran_authorblocka_marked_short_email_list_whole_on_first_red`; P4b4f); 1
  - IEHREF — marked `\href{mailto:}` e-mails lose their link (`ieeetran_authorblocka_marked_href_emails_keep_links_red`, 2505.22454); 1
  - LGADR — a grouped marked address line `\{$^{1}$ann, $^{2}$bob\}@x` cut apart (`llncs_institute_grouped_marked_addresses_cut_apart_red`, int165a too); 1
  - BSTN — `elsarticle-num` has no row in `lookup_bibstyle_params`: its list alphabetical where pdflatex lists by citation (`index-bib/elsarticle_num_lists_by_citation_red`; revGA3); 1
  - BLSN — biblatex `sorting=none` with no shipped `.bbl` lists alphabetically (`index-bib/biblatex_sorting_none_lists_by_citation_red`); 1
  - MBAL — every multibib list holds every cited entry, each numbered from [1] (`index-bib/multibib_lists_hold_every_entry_red`); 1
  - JURS — post's numeric gate: an author-year list under an unsorted `.bst` stays alphabetical (`index-bib/jurunsrt_lists_by_citation_red`; drop the gate); 1
  - BURL — `style_prints_urls`, post's last bibstyle-name list (fence pin 5): the engine writes the url fact on the node (GA3); 1
  - LBIB — the kernel `\bibitem` label path by `\NAT@ifcmd`: read whether `\@lbibitem` hands `#1` to `\bibcite` or runs it (`index-bib/lbibitem_parses_its_label_red`; GA5: acl2012/2015 refute the meaning test); 3
  - ACLB — acl's `\@lbibitem` prints a `[label]` at the entry where pdflatex prints none, Perl too (`index-bib/acl_lbibitem_prints_no_label_red`); 1
  - GA6 — cell padding by template spelling in five copies (`alignment.rs`, `normalize.rs`, `tex_tables.rs`; GA0 finding 8; `kernel-alignment/halign_hfil_template_pads_its_left_red`); 3
  - GEQB — (SUSPECTED) `tex_math.rs` EqnoTag predigest checks the stop set inside a braced tag where Perl reads it balanced (revGA5); 1
  - IEGN — a grade inside a note, biography or the body dropped, as Perl (`check_binding_macro_drops_inside_an_entry_red`); 1
  - IEGW — a grade in a plain list's `\IEEEauthorblockA` raises 2 false content-check Warnings (`ieee_membership_grade_in_a_plain_affiliation_call_red`); 1
  - IEGF — a font-wrapped name and grade print the grade inside the name and keep the contact (`ieee_membership_grade_inside_a_font_wrapped_name_red`); 1
  - PSTL — pstool's `\pstool{f}{\psfrag…}` fragment uncounted (1608.03745; `reversion/psfrag_pstool_fragment_uncounted_red`); 1
  - PSFM — a graphic measured in `\settowidth` then printed is counted twice (`reversion/psfrag_measured_graphic_counted_twice_red`); 1
  - GA4N — `\@tabularcr`, `\@arraycr`, `\LT@tabularcr` one meaning (`\lx@alignment@newline`): `\noalign` through a macro (`noalign_tabularcr_through_a_macro_red`) and a kernel tabular's `\hline\noalign\@arraycr` (pdflatex 10 errors, ours 0: `noalign_arraycr_in_a_kernel_tabular_red`) — give each row end its own meaning, collcell's test a meaning family (GA4, revGA4); 3
  - GA4C — a longtable caption row's other content: keep it as its own row and skip the column template in the caption's `\multicolumn` cell (LT@mcol), replacing GA4b's text-comparison exclusion (`longtable_caption_then_a_nested_tabular_red`, `longtable_caption_takes_the_column_template_font_red`; 1903.09068, 1009.4206); 3
  - P5BN — a framed collaboration call with its own note (`\fnref`/`\footnote`) a nameless creator or the collaboration's creator holding it (`elsarticle_author_framed_collaboration_with_a_note_red`; 2301.12970); 1
  - P5BA — a bracketed collaboration call or column with its own affiliation: the affiliation a nameless creator or the first author's (3 REDs, synthetic; the "for the" form pre-existing; revP5b); 1
  - P6BF — an inline "… and Bob Baker for the X Collaboration" remark: the emitter's plain-text fallback, 2 counted Warnings, content right (`author_names_inline_remark_markup_fallback_red`; P6b); 1
  - P5SV — svjour3's `queued_author_name_prints`/`queued_author_names_spelled` read the names by tokens (`svjour_institute_piece_naming_a_macro_held_author_red`; P5c); 1
  - OSAN — OSA classes: an author name by `\csname` lost as "_" with a false Error; a name macro with arguments loses its space (`osa_author_name_by_csname_is_lost_red`, `osa_author_name_macro_with_arguments_loses_its_space_red`; P5c); 1
  - IEGNM — a co-author's name written inside `\IEEEmembership{…}` (2507.19818) sits in the grade's contact: make it its own creator (`ieee_membership_holding_a_coauthor_name_red`; CL167); 1
  - UPCS — `\uppercase` over the last group's `\IEEEmembership` keeps its lowercase; an earlier group's prints uppercase with a counted markup Warning (2402.07183; `ieee_membership_inside_uppercase_keeps_lowercase_red`; CL167); 1
  - PBLX — post's `biblatex`-prefix test in `make_bibliography.rs`, uncounted by F1 (a post-crate fence); the easy `\ver@` debts `dgruyter_sty.rs:27`, `njdnatbib_sty.rs:59`, `ws_rv_har_sty.rs:22` → `class_loaded`, and `\lx@record@provided` (a query's `\relax` stops the version being recorded) (F14); 1
  - UNCR — a wordless `\thanks` written as its own binding command (REVTeX's after `\author`) goes the contact route and is lost uncounted (`revtex_wordless_thanks_after_author_uncounted_red`; revUNC); 1
  - UNCD — a wordless note counted twice when two readers meet it (`author_wordless_notes_counted_twice_red`; revUNC); 1
  - LOGOX — `\thanks{Hello \LaTeX}`: a false "excess" content-check Warning, the rehearsal's note text missing the logo (`author_thanks_logo_false_excess_warning_red`); 1
  - UNCW — wordless `\thanks` notes rendered (keyed by their place, as a picture's note), after measuring their real reach (UNC: lexically ≤ ~1,500 arXiv, most expanding to words); 3
  - TP3H — a raw class's title page reading > 8 M tokens per run: a store only the run decides lost on the walk's fallback (the `\thanks` label), only the budget Warning naming it (`title_page_over_the_read_budget_loses_a_store_label_red`; revTP3); 1
  - AMSV — amsart-derived classes: `\@setcopyright` unbound in our amsart, the class's store ("Course: MATH 410") lost, counted (`ams_class_maketitle_with_unbound_internal_loses_store_red`; TP3, BM probe `ams`); 1
  - AUTP — a document macro WITH parameters holding names stays by text, counted (the end of its arguments unknown when the call ends the field) (`article_author_macro_with_parameters_holding_names_red`; AUT1); 1
  - AUTL — a leading mark after a comma (`, $^{1}$Bob`) lost, counted (`omnibus_author_leading_mark_after_a_comma_is_lost_red`; AUT1); 1
  - AUTC — hep-ph0301204 +0.60 % (a 1,000-name probe +4.6 %): the cost of emitting the gap marks AUT1 keeps — make a shown mark's emission cheaper (AUT1b); 1
  - GAPC — a shown gap mark loses its trailing comma (0704.2093; CL168); 1
  - GAPS — `$^{1,*,**}$` after a name shows `1,∗`, the `∗∗` (the corresponding-author note's) not shown (1507.03321; CL168); 1
  - FNTN — `\fontname\font` under NFSS prints the CM name (Perl the same, `cmr10`); author reach 0 of 2,027 census papers; needs `.fd` → TFM in `\selectfont` (ROW0A, typog-example); 3
  - GINS — `\lx@Gin@include`/`\lx@Gin@setfile` local where `\includegraphics` is global (since batch 70a; graphicx loaded in the body or a group loses the graphic, Perl keeps it); EASY: `scope => Global` on both (ROW0A; the repro `gin_body` is not in the tree). Measured on rc2 2026-11-23: graphicx `\RequirePackage`d inside a body group — pdflatex itself stops ("Can be used only in preamble", an undefined control sequence) and we raise the same two Errors; `\AtBeginDocument{\RequirePackage{graphicx}}` converts clean. So no well-formed document loses its graphic: v0.8.1, low priority; 1
  - TUZK — an overflow `&` grows a column (Perl Alignment.pm:137-143) where TeX §792 ends the row, the cascade an Error per extra `&`; `\@nolnerr` silent (SHARED; tuzuk; ROW0A); 3
  - CTOV — CT1ar's count calls a space-separated repeat glued: `$^a$Dept of Physics $^a$Univ X` over-counted "a", XML right (`ctlabel_space_separated_repeat_over_counted_red`; DG accepted the over-count); 1
  - CTIC — an isotope after a comma (`, $^{3}$He Lab`) cut as a second entry of label 3, its "3" lost uncounted (`ctlabel_isotope_after_a_comma_uncounted_red`; revCT1ar); 1
  - CTSF — `$1$St Floor`: a math number leading a line read as a mark, its "1" lost uncounted (CT1a's St Floor repro; CT1ar); 1
  - P28G — guard rows for P28m2's generic take-back outside IEEEtran (an svjour3/kernel `\author` replaced: the nameless creator and its note dropped) and the `\let` copy defined only after `\author` (revP28m2's l1 probe); 1
  - P28C — `\ifblind Anonymous Authors\else <blocks>\fi` (literal or in a macro): a nameless creator with the words as its affiliation, where the base named it (`ieeetran_author_blind_conditional{,_in_a_macro}_red`; revP28m S2); 1
  - P28B — P2-8b's block-call fact (the whole IEEEtran `\author` one block the reading cuts; the grid rule in the reading; the binding's ~820 lines of token passes moved) — DG: after the merge (`agent_reports/2026-11-12_P28b_report.md` options B/C); 5
  - TRUTH — the bars' truth set (every paper the frozen scorer flags + ~500 it passes, two independent agent reads from the PDF and source, disagreements to DG) — DG 2026-11-12: after the merge; the per-bundle bars (content kept or counted; no paper worse) gate the merge; 5
  - HOLD4 — the fresh holdout v4 read once, 98/98 per creator and per paper — DG 2026-11-12: after the merge, after TRUTH; the holdout stays sealed until then; 3
  - PERF — the perf + RAM pass, remaining levers (#346) — DG 2026-11-12: after the merge (each bundle's cost bar ≤ +0.5 % blocks slowdowns); RAM witness 2605.17422 (4.04 GB peak RSS; Fatal:oom at the fleet worker's address-space limit in runs 368 and 375); 5
  - FMPL — the frontmatter perf levers (FMPERF 2026-11-13; after AAS1 the >1,000-author AASTeX papers stay +7.3 / +8.3 % vs the 2026-10-31 audit: the author blocks' rehearsals ≈0.33 G, `emit_commands` ≈0.53 G diffuse, entry digestion +0.3 G — DG: after the merge; P23's ~25M gap); 3
  - T1 — the repro suite move (`tools/perfect_kernel/repros`, 1,469 files → `latexml_oxide/tests/repros` header-contract trials; drain `tools/`; memory `project_repro_suite_move`) — DG 2026-11-12: after the merge; 3
  - P6R — P6's rest: the declared cut `cut_list` and P4b2's cost gate `may_open_with_mark` out of the moved directories (e0's MOVED_CUT_KIND_CODE_LINES, OLD_READER_EXPORTS, TOKEN_READS fall), further compaction (P6S) — DG 2026-11-12: P6 closes at P6f; + LST0's plan (`2026-11-13_LST0_report.md`; its 18 kind-deciding F/D lists are the special-casing debt): P6R1 name annotations by the binding's declaration (lists NAME_ANNOTATIONS, PRINTED_IN_PLACE, FRONTMATTER_SETTERS, MARK_NOTE/NOTE/NOTE_TEXT_COMMANDS), P6R2 font commands by `declared_font` (with P6S D3/P6c), P6R3 breaks/spacing by meaning, P6R4 symbols/accents by glyph; 3
  - QA80 — the upstream LaTeXML patches (8 prepared: QA08, QA30, QA35, QA36, QA39, QA50, QA56, QA62; `/home/deyan/data/pk_wt/latexml_upstream`, `~/data/pk_agents/main/upstream_patches/`; summary e-mailed to BM 2026-11-12) — HELD: BM asked us to wait; none submitted; each goes upstream at BM's go and DG's OK; no merge gate (DG 2026-11-12); 3
  - QA92 — the reply owed to BM (Q3.9, Q4.6, Q3.13; QA02's rest awaited from BM) — any time, no merge gate (DG 2026-11-12); 1
  - CSS — the sizing CSS (QA27: BM's em-of-parent lengths as input) — DG 2026-11-12: after the merge; + RBXB — the stock LaTeXML.css transform model lays resized tables over their captions (79 % of resizebox+tabular papers with the stock CSS, 0 with ar5iv.css; witnesses 2606.15594, scratch_RBX0/m/fl.tex; `2026-11-13_RBX0_report.md`); 3
  - QA84 — comment compaction across the crates — DG 2026-11-12: after the merge (on main); DG 2026-11-15: deferred until after the cutoff ("it can wait"); 5
  - QA75 — eyeball review of rendered HTML (a short look folded into SWEEP before the merge: QA75S, 2026-11-13 — LOGO1 and MEM1 fixed, the minor patterns QA75M) — DG 2026-11-12; 3
  - CLEAN — workspace cleanup (keep-list to DG) — DG 2026-11-12: after the merge; 1
  - ARXIV — L7, the full arXiv rerun on the merged head (stream G; ~2.96M papers, ~3 days; the last complete one run 397, November 2026, on the cutoff; run 336, 2026-10-10, before it) — DG 2026-11-12: after the merge, from main; its new Error/Fatal clusters classified into this list; 5
  - TABUN — tabu's private `|` let is inherited by a tabular nested in a tabu cell: `|[1pt]` accepted where TeX errs "Illegal pream-token" (`tabu_vline_style_in_a_nested_tabular_red`; revTABU2 N1); 1
  - TABUE — a `\tabularnewline` inside a list in a plain tabular cell: the recovery from the unbalanced group drops the table and every word after it (8 E, only "Before." kept; pdflatex errs but prints all) (`row_end_inside_a_list_in_a_cell_red`; revTABU2 N2); 3
  - TABUC — `{thebibliography}` in a p-column cell: `<ltx:bibliography> isn't allowed in <ltx:block>`, the content kept; pdflatex 0 errors (`block-model/bibliography_in_a_table_cell_red`; TABU2c); 1
  - TABUF — `{center}`/`{flushleft}` in a tabu cell keep their line breaks where TeX ends the row (tabu's `\centering` ends with `\arraybackslash`; pdflatex 104 `!`), 0 E (`tabu_center_env_backslash_red`; TABU2); 1
  - SUPIT — a floating superscript rewritten into text `<sup>` (TeX_Math.pool.ltxml:527-550, every `word$^{2}$` and `Name$^{1}$` mark) takes the math default font: `<sup><text font="italic">2</text></sup>` where pdflatex prints CMR7 upright; Perl-shared (KNOWN_PERL_ERRORS #1235), OD #928's frontmatter residual (`floating_superscript_rewrite_digit_italic_red`; P4b4r Q2; since AUT1 ~30 Frontiers papers' held-name marks too, as their inline twin: `omnibus_author_math_mark_in_the_name_run_is_italic_red`); 1
  - TAILN — a name ending in a bare accent command (`A.~Pompo\v`, 0705.2788) falls to the plain-text path and loses its no-break space, counted (PMK1's `tail_call`; CL165); `tail_call` on an operand (`authblk_mark_macro_ending_in_let_operand_red`) (revPMK1); 1
  - NOTEX — a machine without TeX Live (OXIDIZED_DESIGN_DIVERGENCES #1095; CI `notex` + `tools/notex_smoke.sh`): the test suite is a TeX Live suite by design — without TeX it fails 2883/8468 (2026-11-19, docker `latexml-notex:check`), all on files only TeX Live has (hyperref's raw keyval/kvoptions/… in 989, expl3 220, revtex4-2.cls's `\@FMN@list` 175, listings' language files, tikz, babel); a visible skip policy is open if wanted. Rust-only: `\usepackage{array}` warns `missing_file:expl3` there (array_sty.rs's `\ExplSyntaxOn` blocks fire the expl3 autoload; Perl's binding has none). The kpathsea crate's own one-time stderr notice (0.3.4 lib.rs:433-434) stays: rust-kpathsea is a general-purpose crate, not latexml-oxide's (DG 2026-11-19).
  - LOADFMT — the strict-`LoadFormat` audit's live residual: a ~72-CS Perl-only long tail (the 58 "misc atomics" + the investigate bucket, minus the ported `\bib@*` family; [`archive/PERL_LOADFORMAT_AUDIT.md`](archive/PERL_LOADFORMAT_AUDIT.md)) — demand-driven, bounded by the corpus success gate, not a release blocker; 3
  - KPEDUP — KNOWN_PERL_ERRORS numbers 80-86 are used twice (`parity/KNOWN_PERL_ERRORS.md`: entries 80-86 once from `pdfcol.sty` (80) and again from `\define@cmdkey` (80)), a pre-existing registry defect `next_entry_numbers.sh` surfaced (#846's body): renumber the later set to free numbers and carry every inbound `KPE #8x` reference; 1
  - BEAMSLIDE — beamer's faithful Perl frame model (`ltx:slide` in `ltx:slidesequence`, Perl `beamer.cls.ltxml`) in place of the document-top frontmatter fallback for `\titlepage`/`\maketitle` in a frame (`beamer_cls.rs` `\titlepage`; P12's follow-up, `perfect_kernel/archive/PLANS_DONE_PHASE56_2026-09-27.md`; witnesses beamerthemecelestia demos, beamerthemeNord, beamertheme-simpleplus); 5
  - IEEO — IEEEtran preprocessing residuals (P4cR2): a refmark inside a font switch (`ieee_nested_block_bold_refmark_spliced_red`), a mark after a box (`ieee_nested_block_mark_after_box_spliced_red`), marks after `\\[2pt]`/`\newline` (`ieee_nested_block_after_spaced_break_spliced_red`) (revP4cR2b); an inner block inside a group stays nested (`ieee_nested_affiliation_block_in_group_red`); the ordinal content-check false pair (`ieee_ordinal_floor_line_names_unnumbered`, 2 W); the Cut gives an unmatched address in an inline marked list to the last entry's author; two sibling blocks after one names block reach the last author only (revP4cR2 q14); a block inside the outer's `\thanks` read as an affiliation (q1); 1
  - CNTE — counter readers' leniency where pdflatex errs: `\newcounter{foo}[nosuch]`, `\setcounter{chapter}` in article, `\addtocounter{zzz}` (pdflatex 4 `!`; ours 0 E 5 W) (revGA1d); `\bibitem` does not step enumiv (`bibitem_steps_the_list_counter_red`); cleveref + varioref FIXED by VREFCLEV (OD #1144); 3
  - EBN — EB1's nits: the calc Warning's wording when calc is not loaded; the ifpdf Error row's message; the DTL1 row's whole `<para>`; one jing process per row; 1
  - MENC — `<m:menclose>` still emitted for `\cancel`/`\boxed` (MathML Core removed it; no mechanical replacement: a rendering + golden change on its own branch; deferred by DG 2026-07-30); 3
  - CI — CI under 30 minutes on `main` (workflow edits, a PR tier) and CIPERF's rest (DG 2026-11-12): TC0's test-cost levers (in-process for the non-CLI spawns, eager = streaming checked once per repro, batch68d's per-name `kpsewhich` loops, the long-pole tests, table-driven guards); CITLIMAGE (DG 2026-11-18: held after the numbers) — the Linux shards and asan in `ghcr.io/tkw1536/texlive-docker:<year>` instead of apt's Ubuntu TeX Live: a healthy apt install takes 76-210 s per shard, the image ~4-6 min (scheme-full with docs and sources, ~17 GB; a job-level `container:` runs out of disk, release-dumps.yml:150-171), a speedup only with a slim tag; if apt stalls persist after #907/#915, cache the `.deb`s in actions/cache first; 5
  - MACFLAKE — `perfect_kernel_batch56::inline_pictures_in_paragraphs_stay_memory_bounded`, a memory-bound guard, hit
    the 110 s wall clock (`Fatal:timeout:wallclock`) on a macOS shard (run 44309107329) and passed on the next PR: raise
    its timeout, or assert the memory bound without the wall clock; 1
  - PUSHPIPE — `git push` with its output redirected to a file sometimes exits 141 (SIGPIPE) right after the pre-push
    lint's "all checks passed", and nothing reaches the remote; a plain retry succeeds: find the pipe that closes early
    in `.githooks/pre-push` or `tools/lint.sh`; 1
  - TOKLOCLINT — under the `token-locators` feature (ar5iv-editor's build; CI never enables it)
    `latexml_core/src/definition/constructor.rs:49` trips `unused-qualifications` (`crate::token::TokenStart`): drop the
    qualification, and add a CI `cargo check --features token-locators`; 1
  - CORTEXTESTS — cortex_worker's own unit tests (`archive_tests` among them) run neither in CI nor in the gates, which
    run only `--test cortex_worker_exit_reports` under `--features cortex`: add `--bin cortex_worker`; and
    `cortex_worker.rs` keeps its own `find_main_tex`/`collect_tex_files` (:1158, :1190) beside
    `latexml::main_tex::find_main_tex` — call the library's; 1
  - TESTTMPLEAK — tests leak temp dirs: `latexml_post/src/graphics_cache.rs:807` makes a `gcache-shared-<nanos>` per
    test process (one per test under nextest; 12,427 found on the dev host) and never removes it;
    `latexml_core/src/common/model.rs` `schema_primary_ns_becomes_default_document_namespace` (~:1508) lacks its
    siblings' `remove_dir_all` (1,622 `lxo652out_*`): a per-run dir keyed on `NEXTEST_RUN_ID` (or self-cleaning of stale
    ones) and the missing remove; 1
  - F1 — Per-encoding text commands, the residual; 3
  - F2 — Small RED repros (`\vadjust` scope; OPEN_ROWS' RED catalogue); 1
  - FONT — Font-selection chain residuals 1-17; 5
  - FRAG — Fragid parity open items 1-4; 3
  - R6 — `Collector::rescan` for generated backmatter; 3
  - R7 — Presentation-MathML F5 linebreaker (port or drop); 5
  - R8b — forest.sty drawing and layout fidelity; 5
  - R3T — tlp.cls `\maketitle` leaves a group open (2606.10056; R3's residual); 1
  - 0c — Noticed in the perf work: `source3` output depends on RSS, `SASnRdisplay`, pgfmath's f64 functions, a Timeout's 39-byte XML; 1
  - S2610 — 2610.06709: the `\footnotetext` dagger, sideways scroll at ≤ 700px, a `<p>` in a transformed box's `<span>`; 1
  - OLDRES — Residuals of closed rows: R1's clusters and witnesses, equation layout (#334), cropping (#337); 1
  - AR5IV — ar5iv tracker residuals: the close-out list, #472/#556 Rust-only, the unclassified four; 3
  - RBXA — booktabs rules add no spacing to the alignment height (Perl the same, booktabs.sty.ltxml:23-39): resized tables 62 % > 10 % short (RED `booktabs_rules_add_their_spacing_to_the_table_height_red`; `2026-11-13_RBX0_report.md`); 2
  - STARTTOC — the kernel `\@starttoc` loses any raw list (Perl the same; 29 TL manuals): emit an untitled `<TOC lists=ext>`, audit its callers, offer to Perl (`2026-11-13_TOC0_report.md`); 3
  - MEMW — memoir: `\after<ext>title` text before the TOC title (RED `memoir_after_list_title_text_precedes_the_title_red`), `\cft<ext>beforelisthook` text after the list (RED `memoir_list_before_hook_text_red`), the `\newlistof` wrap not idempotent (an old ob-toclof), the chapter-style case not carried into list titles (MEM1; `2026-11-13_brief_revMEM1.md`); 1
  - LEGAB — author-block legend lines (`\textsuperscript{$^*$ prose}`, Perl the same: RED `author_line_superscript_wrapped_legend_lost_red`, 2502.05252) and a mark item's symbol tail not split before label matching (REDs `author_list_mixed_mark_note_becomes_nameless_creator_red`, `authblk_author_mark_list_mixed_symbol_note_becomes_nameless_creator_red`; 2509.15733, 2606.27251); reach 1 of 35,942, DG 2026-11-13 post-merge by reach (`2026-11-13_LEG0_report.md`); 2
  - BRKC — a breakable tcolorbox inside a minipage or float is not broken by TeX, yet its overflow is a Warning (REDs `tcolorbox_breakable_box_in_minipage_overflow_red`, `tcolorbox_breakable_box_in_float_overflow_red`; revSBX1); 1
  - FORC — article `\and for correspondence: …` read as both authors' affiliation, 0 E (revSBX1 probe n3; CAU1 too); 1
  - GATHER — linenoaa's `{gather}` on amsmath internals (RED `amsmath_start_gather_internals_red`; no sandbox witness; SBX1); 1
  - ABLK — authblk `\author[ ]` (a blank mark) given an affiliation the PDF does not print (RED `authblk_blank_mark_author_takes_no_affiliation_red`, 2605.24212; SBX1); 1
  - WRAPN — user wrappers around marked affiliation lines (`\inst`/`\myaff` around `$^n$` lines, `\textsc{\textsuperscript…}` in `\thanks`, `{array}` marks): lost, duplicated or misordered, counted (pre-existing; probes scratch_revSBX2F/p/u4_inst, x3_authblk, v8_tsup, t7_array, x2_myaffsuf; revSBX2F); 2
  - TKAR — `takes_arguments` reads "hands on an optional argument" from the spelling (RED `thanks_mark_list_parameterless_macro_ending_in_bracket_red`); FWEND — `Field::whole`'s `\end` spelling → the frames (SBX2F); 1
  - GRPRD — a call reading across `\and` moves words into the next name (RED `article_author_declaration_reading_across_and_moves_words_red`); a call defining then printing its argument leaves a later macro undefined (RED `article_author_call_defining_then_printing_its_argument_red`); `field_position_encloses` compares positions only (NIT) (revGRP1, `2026-11-13_brief_revGRP1.md`); 2
  - APCOL — `\author{X Project \and Ann Smith, Group}` makes Ann an affiliation (RED `author_project_column_takes_the_next_column_as_affiliation_red`; plain words on the base too); `\author{Ann \and Bob, \TeX\ Users Group \and Cat}` one creator (LOGO1, revLOGO1); 1
  - EMIT — 0906.2789's `{\em …}` affiliation loses its italic, uncounted (GRP1/LOGO1 lateral; CL171a); 1
  - QA75M — QA75S's minor patterns (`2026-11-13_QA75S_report.md`, gallery scratch_QA75S/gallery/): code listings one paragraph per line (the tabularray, thmtools, nicematrix docs), ACM boilerplate above the authors (2605.07722, 2403.05668), stray † marks and the running head in the body (2311.07985, 2407.08541), achemso citations glued to the word before (2606.06771), biblatex.tex's title lacks "Version 3.21"; 3
  - REPOTRIM — the repository's local-only files out of the tree (DG 2026-11-13: after the merge; "keep all repro files in the repository … The process and session history is likely not useful and can be local-only"): process and session docs (`docs/archive` session logs, `docs/perfect_kernel` archive and worklists, dated audits, ≈22.2k lines), `bin/ab_audit` + `bin/binding_audit` (5.7k), `tools/perfect_kernel` scripts and tsv with local paths (3.9k), `docs/parity/*_AT_EMIT2.tsv` (0.8k) → out of tree on this machine; every repro and NOTES.md stays (`2026-11-13_REPO0_report.md`); 3
  - GUARDC — the guards as table rows with one shared helper set (`cluster_package_guards` 177k lines, ~32 per test; `convert_clean` ×48, `convert_counted` ×32): −40..−60k lines and CI time (the visible skip done in #826: `util::test::Requirements`, one `skip()`; `2026-11-13_REPO0_report.md`); 3
  - CORERES — latexml_core residue (`2026-11-13_CORE0_report.md`): uncalled `dump_frame_bindings` (stomach.rs), `merge_selected_font` (content.rs; sect13.rs:2568's stale comment), `memory_fatal_seen` (watchdog.rs; check MEMORY_FATAL_SEEN's reader first); cache `LXML_TRACE_NODE_BOXES` / `LATEXML_EXPL_TRACE`; arena.rs `first_bytes` aliasing → through `with`; one frame-id counter (`NEXT_FRAME_ID` / `FRAME_SERIAL`); 2
  - CMTLEN — comment length: 4.8k lines of latexml_core sit in 6+-line comment blocks (BM's "short comments"); the narrative to OD entries, every witness id kept (`2026-11-13_CORE0_report.md`); 2
  - MATHAB — A/B analysis: skip parsed-math-only changes when the `tex=` attributes are equal — the math parser's limits are wall-clock, 25 of int171a's 81 changed papers flipped under load (`2026-11-13_CL171b_report.md`); 1

  - WIN1R — WIN1's post-merge rows (WIN1 done in #833: the suite green on TL 2025, the distro TL and TL 2026; its TL 2026 install recipe and resolution in `archive/SYNC_SESSIONS_2026-11.md` "WIN1"): `\pdftexrevision` is Perl's 19 where TL 2025 runs 1.40.28, so latex.ltx takes the `\everyjob` glyphtounicode branch (needs a full A/B); an undefined `\if…` recovery swallows the rest (`parameter-conditional/undefined_if_name_swallows_the_rest_red`); biblatex's `\blx@blxinit` hooks never run (sbl's `\lno` …); `\Umathcode` defined under the pdfTeX persona; asmejour's `\arabic*` "Missing number" our Warning, TeX's Error; a fill glue at a box's end prints a space (`boxes-groups/fill_glue_at_box_end_prints_a_space_red`); csvsimple 3.0's l3 code in an siunitx column (`package-bindings/csvsimple_l3/csvsimple_l3_siunitx_column_red`); keytheorems custom `thmstyle` → `\newtheoremstyle`; Verbatim `numbers=left` trailing spaces; `latexml_tests_internal` ignores `golden_variant`; the circuitikz predicate could key on `transistors/rounded caps`; 3
  - IFGEO — #845 (the upstream sync to `0db6f47e`) left as upstream ships it: `ifgeo10` also sizes some non-ifsym glyphs (wasysym `\CIRCLE`, marvosym `\SquareSteel`) at 1em (KPE #1245; RED `fonts-nfss/ifgeo_fallback_sizes_non_ifsym_glyphs_red`; a 20-paper census changed no output); 1
  - RHAIR — Rhai residuals (#829, #849, #860; OD #1030, #1031, #1035): `#tags` in a text-level template (`rhai-bindings/tags_in_text_red`); a gullet-only one-argument macro (`gullet_only_on_one_arg_macro_red`); `IfCondition` on an argument-taking conditional with too few arguments is silent where Perl dies (#849); `fontdecode_encoding_ignores_family_map_red`, `stringify_dimension_red`, `trimmedcommalist_trailing_comma_red` (CHEAT3b, #860); the `Invocation` error stub; `counter_value`'s Warning on an undefined counter (the imperative constructor's space and `afterDigest`'s `($stomach, $whatsit)` are green: #909 OD #1079, #908); 3
  - RHAIPOLR — the Rhai polish's residuals (#906, #908, #914): the locator walk ends an all-sourceless walk on the
    innermost mouth and gives none for a sourceless mouth alone, where Perl keeps the last mouth examined, the
    outermost, and prints its `String; line …` (`gullet.rs` `get_locator_opt`/`get_locator_from_start`,
    Gullet.pm:150-157; message text only; RHAILOCWALK); a TeX-level message raised by a `.rhai` binding's `RawTeX` names
    no binding file, where Perl's `Mouth::Binding` gives `at slp.sty.ltxml; line 2` (Mouth/Binding.pm:42-49; open a
    binding mouth while a `.rhai` file loads; RHAIRAWLOC); Perl stops after an initialization error, we go on
    (RHAIINITABORT); deprecated-alias Warnings and the option-key Error name the `.rhai` file but no line (no call
    context at the registrations; RHAIDEPLINE); `\usepackage{misn}` may load a bare `misn.rhai` (a class request too):
    accept only `<name>.sty.rhai`/`.cls.rhai`, or log an Info naming the file (RHAIBARE, unverified); two bindings with
    byte-identical source share one AST (`SCRIPT_CACHE` keyed by source), so one deprecation Warning for both
    (RHAIHOOKSRC); 3
  - DELC — `\delcode` holds 24 bits: the delcode table as an integer table (the dump's DELCODE_TABLE, tex_math.rs:2336's −1 reading) (`macro-state/delcode_holds_24_bits_red`; #829, W3RB); 1
  - REGW — the core register lookup could Warn on an undefined CS once the kernel's optional-register probes use the quiet lookup (a Warning there added geometry `\Gm@tw` Warnings to 27 of 200 papers; #843, OD #1036); 1
  - XYOPT — `\xyoption{<feature>}` without `[all]` loses one space mid-paragraph through `load_tex_definitions`; Perl keeps both (pre-existing; #843); 1
  - VERBR — verbatim CSS residuals (#844): a `numbers=left` line number overlaps the code (`listings-algorithms/fancyvrb_numbers_left_overlap_the_code_red`); a breaklines first line indented twice (fvextra's negative `\hspace` missing from the HTML); a breaklines Verbatim in a minipage in a `tabular` widens the page (pre-existing); `display: contents` on the id-bearing wrapper may remove an `#id` link's scroll target (suspected, unprobed); 2605.00468 widens a phone page through a fixed-width tcolorbox SVG; an xleftmargin Verbatim in a minipage or p-cell loses the margin from its second line on (#855, issue 853; I853); lstlisting's xleftmargin not rendered (Perl parity); 3
  - MODCV — moderncv 2.x: our binding, like Perl's, predates it — `\name` and `\makecvtitle` raise Errors, `\cventry` drops two fields (`class-bindings/moderncv_name_style_makecvtitle_red`; #844); 3
  - NICEO — nicematrix: 2206.12224's second trailing key list (`package-bindings/nicematrix_second_trailing_options_red`); NiceArray's `tex=` reverts to `\left.\begin{array}…\right.` and drops `\Block` (pre-existing) (#839, issue 736); 1
  - MSPC — math spacing (#841; OD #1032, #1033, both to offer upstream): `\mathbin{/}` and `\bigm/` lose their class in digestion (tex_math.rs:1194 → :2694), so no space where TeX gives medium or thick (`math-parse/mathbin_slash_keeps_binary_space_red`); the spacing walk (presentation.rs `adjust_pair`) ignores script style, `x^{a\mathrel{\star}b}` 0.278 em where TeX has none (`math-parse/script_style_rel_has_no_thick_space_red`); 3
  - PRESC — prescripts (#837, issue 824): a trailing float script after a prescripted factor, a prescripted bare OPFUNCTION (`scripted_factor_l11`), `{}^*x` (`math-parse/{trailing_float_script_after_a_prescripted_factor,prescript_on_a_bare_opfunction,prescript_star_on_a_bare_identifier}_red`); 3
  - ARRF — arrow heads (#838, OD #81): an uncomposed head pair or a `\let` alias of a head reads as a list (`arrowfill/arrowfill_uncomposed_pair_red`); mathtools' `\xLongleftarrow` undefined (`arrowfill/mathtools_xLongleftarrow_red`, pre-existing); 1
  - ARCR — colortbl and rules (#835, OD #1025): a `!{}` rule's own colour (`alignment-bindings/arrayrulecolor_bang_rule_own_colour_red`); the error boundary of colortbl's `\noalign` test (`arrayrulecolor_mid_cell_misplaced_noalign_red`, `arrayrulecolor_noalign_and_before_multicolumn_red`); `\hhline` colour codes; the kernel template reader drops a `|` directly before `!{…}` (Perl too; `tabular_bar_then_bang_rule_between_columns_red`); a math array's `\multicolumn` `tex=` loses the template (`math_array_multicolumn_tex_keeps_the_template_red`); 3
  - CAPJ — caption justification (#834, issue 787, OD #1024): sub-captions (`captions-floats/subcaption_justification_and_skip_red`); `centerlast`/`centerfirst`; a justification through a user macro or raw glue — read the digested body, not its spelling (`caption_justification_through_a_macro_red`, `caption_justification_by_glue_red`); a short multi-line caption under singlelinecheck (`caption_singlelinecheck_short_multiline_red`); 3
  - LWID — `\linewidth` residuals (#836, issue 778, OD #42): a tabularx `X` cell's width, which needs the X width computed (`kernel-alignment/tabularx_x_cell_linewidth_red`); `\linewidth` inside list items (`lists/list_item_linewidth_red`); 3
  - TXWD — class text widths (#830): the standard classes' and elsarticle's `twocolumn`, REVTeX's size options (`class-bindings/{standard_class_twocolumn,elsarticle_twocolumn,revtex_size_option}_textwidth_red`); 3
  - ETOCR — etoc (#830, issue 760): relative depth (`package-bindings/etoc_relative_depth_red`), `\etocsettocstyle` headings (`etoc_settocstyle_heading_red`); 1
  - IMPFC — import: a filecontents file under an import directory (`package-bindings/import_filecontents_file_under_an_import_dir_red`; #830, issue 772); 1
  - EPUBR — EPUB residuals (#840, issue 807; Perl the same): a document without sections has no navigation document, which epubcheck rejects (`epub/no_sections_no_nav_red`); `--splitnaming=label` with a label equal to the stem overwrites the root page (`epub/split_label_collides_with_root_red`); CI does not run epubcheck yet; 1
  - GRMLINT — the math grammar builder (`init_grammar`) shares the stack-frame lint `allow` with the converter; splitting them (#832); 1
  - MCSS — carry an author's row spacing (and other layout MathML Core dropped) through CSS instead of the unsupported MathML 3 `rowspacing` attribute; `\arraystretch{0.8}` writes a negative `rowspacing="-2.0pt"` today (DG 2026-11-14, PR #848 follow-up; low priority; after the cutoff); 3

  *Folded 2026-11-15 (ROWR1): the 2026-11-14/15 reviews and studies, and the merged PRs' residuals:*

  - MML729 — MathML Ord spacing (deferred past the cutoff, DG 2026-11-14; branch `fix/mathml-ord-spacing`): the lone-operator rule (a lone top-level `$\pm$` spaced as infix) and the ∇/¬/∀/∃/set-builder `\mid`/`a\|b`/`\colon` spacing (the walk types ∇, ¬, `\mid` by LaTeXML's role where TeX's class differs) await Bruce (QA94); `${}\pm{}$` empty-Ord evidence; `aligned`'s `&-x` prefix minus (medmuskip in TeX); REDs `math-parse/nabla_cdot_is_a_binary_operation_red` (on main; extended on the branch) and, on the branch only, `math-parse/lone_sign_is_an_ord.tex` cases, `sign_after_absent_operand_red.tex`; 3
  - MISSN — "Missing number, treated as zero" is a Warning where pdflatex errs (Perl Gullet.pm:904 `Warn('expected', '<number>', …)`, its Dimension :972 and Glue :1047 siblings; pre-existing, revQAB4): an Error-boundary batch, census first (WIN1R's asmejour `\arabic*` and FNMK's `ifnum_missing_number_inserts_equals_red` raise the same message); 3
  - XDEFP — a PERF lever (post-cutoff): our 1.45-1.75× excess over pdflatex on `\xdef`/`\cs_gset_nopar:Npx` with `\unexpanded` copies — `Tokens::pack_parameters` looks up a meaning per CS and goes through a VecDeque, and parameterless macro bodies are copied onto the pushback where TeX's back_list is O(1) (tex.web §323); HOOKINITb profile `/home/deyan/data/pk_wt/scratch_HOOKINITb/prof/`; options: a single-pass pack_parameters, a cheaper `\unexpanded` copy, a design study for shared token lists on the pushback (also OD #326's 5M pushback limit); 3
  - I777 — issue #777 (the slowdown as a document grows): profile the reporter's document now that #802 is in (#865; ISSTRI, `agent_reports/2026-11-14_report_ISSTRI.md`); 3
  - I802R — #802's follow-ups (#865): ~44 whole-document predicate XPaths in latexml_post (`document.rs:1450/1458/1634`, `math_processor.rs:209`, `crossref.rs:1863,2079`) and frontmatter `insert.rs`/`relocate.rs` each fail past libxml2's 10M node-set cap — reuse `xml::descendant_or_self_elements` / `Document::find_elements`; a no-namespace script XMTok keeps an unreferenced id (`math-parse/unparsed_script_token_outside_ltx_namespace_red`); kit `probe.sh` counts only `^Fatal:` at line start and passes no `--timeout` (a wallclock Fatal after "(Finalizing..." went uncounted); 3
  - PGTTL — the page `<title>` fallback reads a title's math as raw text (pre-existing; found reviewing the UnicodeMath PR): `crossref.rs` `generate_document_title`, used when the root has no `xml:id`, takes the first `//ltx:title` through `get_text_content_node` (`get_content`), so a document without `\title` that opens with `\section{On $x^2$, $a_{i+1}$}` gets `<title>1On x 2 , a + i 1</title>`, where Perl `generateDocumentTile` (CrossRef.pm:811-812) calls `getTextContent`, which honours the tag's `close` and reads an `ltx:Math` through `unicodemath`: `1 On 𝑥², 𝑎_{𝑖+1}` (vendored Perl latexmlc, 2026-11-19) — route it through `value_text`'s element path (`scan::title_node_text` plus the whitespace collapse); every untitled single-page HTML whose first heading has math; 1
  - I857R — #857's residuals (#864): a real-Safari check of ar5iv-css's WebKit rule; a `\bibitem[Smith et al.(2000)]` label with spaces classed numeric (2003.13016) may be spread by justification in WebKit; a nameless biblatex `\textcite` prints "[2]", classed numeric, where the PDF prints "No Author Misc [2]" (OD #786's open gap); natbib `\citet`'s "(author?)" placeholder lost, uncounted (`index-bib/natbib_citet_unknown_author_red`); lrec's `\protect\citename` bibitem loses its `)` (pre-existing); 1
  - I694 — tcolorbox: flow-content boxes degraded to a semantic CSS block in HTML, where the fixed-geometry SVG overflows (issue #694) — after the cutoff; 3
  - I783P — #783's post-merge author lines (revI783, revI783F; pre-existing unless noted): aistats address rows — Eve East's "Univ E" unclassed while Ann's "Univ A" is classed (77o), a shared one-column aistats address unclassed while two-column ones are; REDs `author_lines_783_center_first_line_red`, `author_lines_783_list_by_order_email_red`, `author_lines_783_declined_email_red`, `author_lines_783_ieee_authorblock_red` (IEEEtran `\IEEEauthorblockA` lines behind the pop-up); 0708.2995's author block shows the Fax line before the e-mail and drops the word "email:"; math0111266's second author "Moreno-Frías, M.A." stays a contact line, not a creator, and "F.J." loses its final dot (base the same); `cluster_cli::browser_render_author_lines` is opt-in (SKIP unless node + playwright-core + Chrome; `LATEXML_BROWSER_TESTS`), so CI enforces only its CSS-string and XML halves; 3
  - I783C — the author-note pop-up's CSS (revI783G; mirror into ar5iv-css): `LaTeXML.css:125` `contain:inline-size` + `overflow-wrap:anywhere` squeeze a long note into a narrow card (36 px × 696 px, words split; `scratch_revI783G/longname_li.png`) — add `min-width:min(16em,90vw)` to the hovered note (introduced, content kept); `:121-129` the hover sits on the inline `.ltx_author_notes`, so a 1-3 px gap closes the notes for a slow pointer (main loses the pop-up over 19-48 px) — hover on `.ltx_creator`, drop the vertical margin; pre-existing: no `tabindex`/`:focus-within` (keyboard and touch cannot open a note); 2507.06016 in Firefox at 375 px overlaps names before hover (main the same, the old pop-up path); 1
  - I790P — revI790A (issue #790, PR #876 merged): `keep_point_out_of` + the `text_run_font` walk cost +0.13 % on 0710.4421 — a text-node discard could compare with the point instead of walking (the replace-node coalesce and trailing-whitespace sites); 1
  - RAWREF — a `\label` after a step whose algorithm2e line was dropped or never built lands on `<document>`, where pdflatex prints the line number (synthetic, `scratch_RAWREF/p/p1,p5,p7.tex`); settled: `raw_refstep_objects` (document.rs:235, :7867-7869, :8010) is no dangling path (revI790A's suspicion) — the stored `Node` is libxml's cached wrapper, nulled by `free_subtree`, and `document.rs:7871` filters a null parent (an LD_PRELOAD quarantine shim over 48 algorithm2e papers: 351 listingline frees, 0 moves); 1
  - LOSTFR — LOSTF's residuals (#866): the pazh/astl `*` mark written as an e-mail contact "*" (`sectioning-frontmatter/mark_then_store_email_read_by_a_refused_abstract_red`; the observer reads the rerouted setter as the class's `\email`; reach ~880); astl address notes all go to the block's last name instead of following the marks (1206.0451 Khabibullin's "1" unanswered); `author_and_on_behalf_of_teams_red` (1002.4968), `rmaa_altaffiltext_marks_red`, `ptephy_author_given_twice_keeps_the_last_red`; `class-bindings/vmsta/vmsta_first_name_without_surname_red.tex`'s engines line is stale; 3
  - LOSTA — revLOSTF: `classify/title_code.rs:252` `reads_arguments` decides by syntax — `\let\address\textsuperscript` stays undefined in the rehearsal though it reads an argument (probe `scratch_revLOSTF/p3/astlet.tex`; rule: run a definition whose definee's MEANING reads arguments, `get_num_args() > 0`); symbol-only `\thanks` notes (`\thanks{$\star$}`, `\thanks{\dag}`) dropped by `note_key`'s `retain` (`emitter/plan.rs:45`; counted; apply `symbol_key` in the note arm; probe `p2/symnote.tex`); two rehearsals without `author_markup_begin` (`separators.rs:211` `read_argument`, `emitter/emit.rs:2787` `rehearsed_with_expansions`) — prefix or justify; the astlb repro's two residuals (notes on the last name; the notes' italic lost) lack a `_red.tex`, and its `holds` should pin the notes' owner; 3
  - AMSPR — amsppt's heads (#868; AMSPPT, revAMSPPT): a bare `\ref` invents a "References" title amsppt never prints (math0206022; `amstex/amsppt_bare_ref_invents_references_red`; Perl and base too); alg-geom9502009 nests 49 theorems where Perl nests 19 (base the same; not minimized); `\proclaim\nofrills{…}` titles "\nofrills" (alg-geom9502009, pre-existing); no `\add@missing` (amsppt.sty:574-576, :863): `amstex/amsppt_roster_without_endroster_before_head_red.tex` (2 E vs TeX's 1, section 2 inside an item), `amsppt_head_inside_proclaim_red.tex` (0 E vs TeX's 2); NIT: five theorem `DefConstructor!`s → one loop; 3
  - AMSRO — amsppt's `\roster` (#868; AMSPPTb, revAMSPPT): an unbraced nested roster gives the outer `\item` back where amsppt.sty:744 makes it plain's (invented "(3)"; `amstex/amsppt_unbraced_nested_roster_outer_item_red.tex`; synthetic); a `\runinitem` first item loses its run-in layout (no word lost); the `\endroster` inside an unclosed `{…}` (math0309287) is fixed (#963, OD #1117, guard `perfect_kernel_batchregfixp::endroster_ends_the_roster_inside_a_group`); 3
  - LOADFR — loader frames (revQAB3b): a non-Fatal `Err` mid-load (`load_tex_definitions(...)?`, `hook?` in content.rs) skips `pop_load_frame`, then `load_class` falls back to OmniBus with the frame still pushed (suspected; a drop-guard or a single exit); a package loaded ONLY in a subfiles sub-preamble loses its begin-document setup (synthetic: `\usepackage{cleveref,glossaries}` + `\newacronym` there); 1
  - RHBOR — a `fontTest` calling `AssignValue` panics "RefCell already borrowed" (state.rs:4087; document.rs:3355 runs the tests inside `state::with_value("TEXT_LIGATURES", …)` — clone the list first; probe `scratch_revRHFAT/p/ligsta.tex`, base too); NIT: the setter row's `kept: ""` checks nothing (revRHFAT); 1
  - ODDB — docs and dead code (ODDOC-B, ODDOC-B2, revQA1985): `tools/fontmap_drift.py:56-67`'s slot-94/126 "Bruce's accents" allowlist is dead (61c made them ASCII) — remove; stale comments at `float_description.rs:84` ("read Undigested"), `latexml_math_parser/src/grammar/builder.rs:2123` ((∂_t∇)·u) and `latexml_core/src/binding/content.rs:1089-1098` (says `.ltxml_loaded` is not set, then sets it); constructor `font='#x'` pairs ignored for the whatsit's `font` property (constructable.rs:214, replacement.rs:796; Perl Document.pm:1834 applies them; probe glossaries/amsrefs `font='#titlefont'`); `next_entry_numbers.sh` cannot see `kit/od_reservations.txt` (#1051) and counts MATH in the OD pool though MATH runs its own sequence (#63); for the OD PRs: OD #887, OD:5004, :21249 and :21538 cite `arrange_panels_and_breaks` at the stale Perl range 3229-3349 (now L3236-3356); 1
  - ODDC — ODDOC-C's rest (revODDOCC): STANDIN members 592 597 605 614 734 735 741 986 and #731's ~500-line body not merged; #216's closed-form `\pgfmathpointintersectionoflineandarc` is a WORKAROUND — principled: TeX-exact trig, so pgf's bisection terminates (zx-calculus, 2201.09268); 13 merged entries carry no "ex-#N" marker on the bullet holding their text (#20 40 41 87 310 323 338 340 516 1035; #732/733/736/744 named in text) — code cites of #87 (bibunits_sty.rs:125, 06_cluster_bibliography.rs:502) and #41 (cluster_xslt_split.rs:149) reach the primary but no labelled bullet; the LOCK/REPLAY one-rule restatement (ruling 8, state.rs:5715-5824) for an ODDOC-E docs pass; 1
  - ODDD — ODDOC-D's code residuals, after DG's Q1-Q5: one Kind pass + one Link chain (#805/#954), K11 → #767 (ruling 9) (the spelling splits under #993 stay: DG 2026-11-15, ODDOC-D Q1, generic binding utilities); 5

  *Folded 2026-11-15 (PERFFOLD): the perf pass's residuals (int/perf, rust-libxml 0.3.24, libmarpa-asf-sys 0.3.1, PDFCMP):*

  - NOTCANCEL — DONE (fix/not-cancel-move): math_common's `\not` rewrite MOVES its operand on both paths (Perl `appendChild`, `math_common.pool.ltxml:341/:354`) and gives the cancel `XMApp` the `\not`'s box and font (Perl `_box`): nested `\not` at any depth `≠`, `$\not\frac{\not\sqrt{a\not=b}}{c}$` 70 %, `\not\not=` `≠`, `\not\not\in` `∉`; the double registration stays, its second pass idle; arXiv 1009.4552, 1611.10101, 2001.07246, 1701.02906, 2209.02767 (A/B 5 of 314 changed, all toward Perl); guards `perfect_kernel_batchnotcancel::*`; 0
  - NOTRES — NOTCANCEL's residuals (revNOTCANCEL; no witness, pre-existing): `Document::insert_math_token` returns the parent (`document.rs:3046`) where Perl returns the new token (`Document.pm:673`) — a `\not` carrying an `xml:id` would hand it to the cancel `XMApp` (census the 3 callers that use the value first: `tbox.rs:253`, `document.rs:1728/:1734`, `math_common.rs:686`); `sideset_wrap` (`amsmath_sty.rs:3777-3784`, Perl `appendChild`) and `base_xmath.rs:1783` (its comment says "Move" but it calls `append_tree`) copy where Perl moves; Replace never frees the node it discards (WISDOM #79 — a future free must skip nodes the callback re-attached, `rewrite.rs:694`); math RED rows (parked): `\not\left(x\right)` unparsed (Perl `delimited-(̸)`), `\overset{\not}{…}` / `\mathop{\not}…` leave `／` unrewritten, `\not\xhookrightarrow` keeps `font="italic"`; 3
  - LEAD805 — #805's rest (closed 2026-11-15; PR 883 fixed the rule fillers): box/char fillers keep an `ltx:text class="ltx_leader"` wrapper (target: the class on the box's own element, no wrapper), and a vertical-box leader lands inside the next paragraph (REDs `leaders_vertical_box_starts_no_paragraph_red`, `leaders_box_filler_repeats_red`) — post-cutoff; the brucemiller 2882 patch offer waits with the UP items for BM's confirmation (DG 2026-11-15); 3
  - LXOOM — a libxml2 out-of-memory under `--max-memory=8192` aborts the process: 1111.1264 SIGABRTs on base and int/perf alike ("failed to create element td", then every XPath fails; 8 vs 18-19 Errors only because PERFE's walks replaced most XPath calls; no XML written) — it should end in a clean MemoryBudget Fatal (`Stomach:MemoryBudget`, stomach.rs:546) (combined A/B INTPERF5 vs MAIN884, 999/1000 identical) — post-cutoff. 2026-11-23: not reproducible within 180 s (wall-clock Fatal at fragment 2048, RSS 4.4 GB rising). Deferred, decided by difficulty (DG 2026-11-23); 3
  - DTDENT — XML input carrying a DTD/internal entities (`--whatsin=xml` / `is_xml_input`, `latexml_oxide/bin/latexml_oxide.rs:1175`): the entity's content is silently dropped from the HTML ("see  and …", no Warning; Perl XML::LibXML expands by default; revPERFD, base too) — DG 2026-11-15: expand at parse (libxml2 substitute-entities), so post never sees entity refs or a DTD; the PERFE walk's XPath fallback for DTD documents and revPERFE's (b)/(c) edges then go away — post-cutoff; 1
  - XMLATTR — serialization keys attributes by local name (first qname, last value; PERFG's R6 kept it exactly): `xml:lang="en" lang="de"` serializes as `xml:lang="de"` (en lost, wrong qname), the same for `id` + `xml:id`; `_a` + `xml:_a` leaks `xml:_a="2"` (the bookkeeping strip removes one attribute per local name) (revPERFG, pre-existing) — post-cutoff; 3
  - XMAPPNS — namespace-less math elements in core's live DOM: PERFK's round-trip inventory found 12,054 elements (XMApp, presumably from the math parser) in 31/50 papers that land in the ltx namespace only via the serialize→reparse, so an `ltx:XMApp` query on the live DOM misses them (latent; root-cause first; I802R's no-namespace XMTok RED `math-parse/unparsed_script_token_outside_ltx_namespace_red` is the same family); 3
  - COREPOST — R9, the core→post in-memory handoff (PERFK stopped, nothing landed; probe 9af0496b2ae "NOT FOR INTEGRATION" on `perf2/core-post-handoff`): DG's design rule (2026-11-15) is to keep both paths, serializing for core XML without post and passing the document object when post runs; the reparse normalizes 1.31M indentation text nodes, attribute order (27 % of elements), 28 text merges, circuitikzmanual's `xmlns:xlink` stored as a plain attribute, encoding NULL vs UTF-8, and the streaming spill placeholders need the string; in-memory normalization costs ~60 % of serialize+parse → best case ~0.5-1 % typical; needs an ordered attribute visitor + non-registering text creation in rust-libxml, and a DG ruling on post's `fallback()` echoing the core XML as the .html on a post failure (Perl: undef) — post-release; 5
  - SPINEG — the streamed spine's math parse builds its own grammar (`latexml_oxide/src/core_interface.rs:657` `MathParser::default()`) instead of sharing PERFA's per-conversion `math_grammar` (:886) — one more grammar build per streamed conversion, ≈0.02 % (revPERFA) — DG 2026-11-15: post-cutoff, low priority ("maybe we won't need that improvement at all"); 1
  - ARENAF2 — the string arena's F2: our own safe never-moving store (0 unsafe) in place of string-interner's BufferBackend; DG 2026-11-15 chose R+ now (ARENAR + ARENARb: unsafe in arena.rs 10 → 1, lends borrow the arena, an Rc<str> overflow page) and F2 later, measured per document on small papers (advisor #2: 2604.01119 makes 5.4 M arena entries in 0.58 s, so +1 ns/entry ≈ +0.9 %); 3
  - MODELWC — `model.rs:1034` `namespace_wildcard_sym` interns its `ns:*` probe (`arena::pin`, called at :1000/:1122 on every tag-frame miss), growing the arena permanently; `arena::get` answers the same in production, since a never-interned key has no frame — the test helper `wildcard()` (:1288-1290) expects `Some` for an uninterned wildcard and changes with it (pre-existing; ARENARb's item 7 covered only `set_allows`' probes, revARENARb); 1
  - POSTLTX — `latexml_post/src/document.rs:100` `is_ltx` wraps a Namespace and builds an owned href per call → `with_namespace_href` as the sibling sites do (:1035/:1066/:1087) (revMEMC, pre-existing) — post-cutoff; 1
  - XWALKRO — core `xml::descendant_or_self_elements` (`latexml_core/src/common/xml.rs:416`, behind `Document::find_elements`, PR 865) wraps a registered Node per element; test on RoNodes and wrap only the matches, as PERFD did in post (`latexml_post/src/document.rs` ~:340-430) (left by PERFD and PERFG; XML runs too); 1
  - RLXF — rust-libxml after 0.3.24 (`~/git/rust-libxml`): a rust-owned node still HELD when its document drops is a use-after-free (documented rule kept, LIBXML24b), and cached unlinked wrappers freed at drop (review B (6), deferred); the documented 120-byte LSan leak (unlinked-node case); `holds_property`'s guarded first-property read could go unguarded (sanity check: the guards cost `walk_node` +0.82/+0.86 %); 14 `cargo doc` warnings in files 0.3.24 did not edit (LIBXML24 open (5), recheck); 3
  - GENMARK — (post-run; DG 2026-11-17: "the square bracket fencing in [a] … may look unpleasant to a reader. Would it be easier to compute this in a binding and annotate this as an attribute, so that XSLT knows what value to pick?") Move mark generation upstream: one engine function in the frontmatter insert pass collects every provided mark (today's XSLT the `printed` node set of `f:generated-mark` (formerly the key the `printed` node set of `f:generated-mark`; since XSLTKEY it holds only what a bracketed-letter lookup can hit — a lookup without brackets must bring back the legend-opening and bracket-stripped alternatives of e130fb1c3f4) set: note marks, tags, superscripts, legend openings) and picks the first glyph of a pleasant series no provided mark uses (`∗ † ‡ § ¶ ‖`, then doubled, letters last); a binding may DECLARE its class's series (llncs ⋆, the document's `\@fnsymbol`) — micro declares, global computes; written as the shared note's `mark` with a generated flag (`class="ltx_mark_generated"`), the XSLT rendering it verbatim; PAGEGEN's fallback reuses it. Hairy: the engine must see legends in body/\date/tables; `mark` then holds provided and generated values; 5
  - RATCHETS-1117 — (DG 2026-11-17: "Accept raises, row R to lower"; and "I am not even sure if we have meaningful refactors towards managing digested data, given that your work found it natural to work on the token level. It is something to consider and rationalize, but possibly after the release") Three release branches raised the token-read/spelling ratchets: MTREDUCEE `latex_constructs/mod.rs` spellings 199 → 219 and two state keys (`lx_title_marks_read_closed`/`_declared`), DATELEG `frontmatter (rest)` reads 37 → 52 (the `\@date` store), IEEEROUTE `ieeetran_cls.rs` reads 104 → 125 (the `\thanks` sentences). After the run: classify every token-read site as necessary (the datum exists only as tokens: a macro's stored body, an undigested store) or avoidable (a decision on spelling where TeX reads a meaning, or a digested/event view exists), lower the avoidable ones, and reword the ratchets to tell the two apart.
  - PUBNOTEMARK — (DG 2026-11-17, OD #1069) a title note carries no `mark`: `.ltx_pubnotes` hides every title note behind one handle (ⓘ since #1069; Perl a CSS †), where the PDF prints each note's own mark (1612.03574's title ∗, 2605.10496's affiliation-line ‡ whose `\thanks` lands in the title's notes) or, unmarked, its text (#1063). Record the printed mark on `ltx:pubnote` as on a creator's note, render it as the note's handle (`ltx_note_mark`), an unmarked one in the open; then drop the `.ltx_pubnotes:before` handle in LaTeXML.css and ar5iv-css. Guard to flip: `xslt::witnessed_css_delta::title_pubnote_content_stays_collapsed`.
  - ACHEMSO-CALLS — the API's `annotate=` counts creators, not author calls: an `\alsoaffiliation` (or an `\email`) after `\author{Ann Able and Bob Baker}` reaches the last name only (RED row `fmmarks_achemso_alsoaffiliation_after_a_two_name_author_red`; 2209.04586, 1810.01505, 2008.03381 show it for the e-mail); an `annotate=` by author call; 3
  - ACHEMSO-ORDER — a waiting author's held pieces (alternative affiliation, e-mail) come before an affiliation given after the next `\author`, where the PDF lists the affiliation first (`\author{A}\altaffiliation{P}\author{B}\affiliation{X}`, revACHEMSO q14); order only; 1
  - ACHEMSO-BODY — achemso's `\author`/`\email`/`\phone`/`\fax` are `\@onlypreamble` (:346, :521, :530, :539): in the body pdflatex errs and prints the text inline; the binding attaches them at once with no Error, a phone even without an e-mail (revACHEMSOb h3/h6); whether to raise TeX's Error is a policy call; 1
  - PAGEGEN — the page's fallback glyph for a markless body note (`LaTeXML-meta-xhtml.xsl` note-mark, Perl's `†`, :429) is a mark the conversion generates; DG 2026-11-17: generated marks never equal a provided mark (author-, class- or package-printed). Where the document prints a provided `†`, the fallback must take `f:generated-mark`'s unique glyph instead (one check, the `printed` node set of `f:generated-mark`); after CONNFIX lands (same XSLT); red/green rows; 3
  - DEPS080 — before v0.8.0 (DG 2026-11-17): study `agent_reports/2026-11-18_DEPS080_dependency_review.md` §4-5. DONE (chore/deps080): A2 deny.toml notes (the distribution binary ships rhai/smartstring; chrono is direct and via pericortex), A4 `once_cell` → `std::sync::LazyLock` (direct dep dropped from 5 manifests). LEFT: A1 marpa fork on `bindgen 0.73` or pre-generated bindings, publish libmarpa-asf-sys 0.3.2 (−4-5 build crates); A3 refresh the fleet checkout's stale lock at the next fleet rebuild (clears the libmarpa-asf-sys license-exception warning); A5 clap `default-features = false` (~6 crates; CLI help loses colour and "did you mean" — needs DG); A6 size levers in our code (`build_model` as data, DEP-15 outlining of the top `load_definitions`); CHRONO after v0.8.0; 3
  - XSLTKEYc — (DG 2026-11-17: "Accept, row R") the author-note XSLT still costs 2405.13379 +1.00 % and 1706.03762 +0.59 % instructions vs main (PERFAUDIT basis: REGFIX vs the #895 head, html5 production recipe; 38/40 docs within +0.5 %, median −0.08 %, p90 +0.46 %): the note templates' own work and the per-note `use` of `f:creator-note`, chiefly kind O's scan of a note's text for its opening glyph (about half of 2405.13379's excess is outside the stylesheets). Options: compute a note's opening glyph once when the XML is built (an attribute), or look it up only within the note's own block. Also `f:glyph-elsewhere`'s third key makes 8 walks under the root's non-excluded children (~2M on 1706.03762): lazy like the `printed` node set, since it is read only when a shared mark's glyph may print elsewhere. WISDOM #105 has the method.
  - IEEEROUTE-OWNERCHECK — (revIEEEROUTEb) no post-placement check that each routed IEEE contact sits on its creator (correct by construction today); witness 2106.02811.
  - AUTOREFBACK — (HYPERPROV, OD #588) under the ar5iv first-use provide, an `\autoref` to a target numbered before the
    paper's first `\autoref` shows the target's title, or its enclosing unit's (an equation reads as its section's title),
    not hyperref's "Equation 1": 2 of 74 autorefs on 4 witnesses (0803.2070 `#Ch0.E1`, 1010.2520 `#S2.SS1`). Options: an
    eager internal tag formatter at preload (an autoref tag on every target of every paper) or a post-processor fallback
    to `typerefnum`. Post-release; 3
  - SHIPOUTNOTE — (HYPERPROV final review; synthetic shapes, no corpus witness — 2309.06611, 2403.15239, 2302.00362 and
    1612.04433 are right) the page-notice note of `ar5iv_sty.rs` `\lx@arxiv@shipout@note`: (a) material with a picture
    (a tikz node) is noted from `ink_text_printed`'s font-encoded strings, so `\url{…_…~…}`, `\_`, `\textbackslash`, math
    and fonts garble — take the text nodes' digested boxes and route on the digested result, not the source test; (b) a
    picture-mode `\put(20,-30){\makebox(0,0)[l]{…}}` still notes a positioned picture; (c) material registered in the
    preamble is digested there, so `\AtBeginShipoutFirst{\parbox{12cm}{Pre one\par Pre two}}` reads "Pre onePre two"
    with a false `frontmatter:unclaimed` Warning (base glues too; registered in the body it is right) — digest at
    registration only to test for text, emit the tokens; (d) a notice behind a macro of the paper's inside the hook's
    argument (`\AtBeginShipout{\mynotice}`) is not seen by the token scan. Post-release; 3
  - ROBUSTIFYERR — (CLUSTFIX3 review, OD #1092; synthetic, no corpus witness) etoolbox's `\robustify` is a no-op where
    TeX errs "not a macro" on a LaTeX `\mathchardef` that is a `DefMath` here (`\usepackage{etoolbox}\robustify{\alpha}`;
    `\infty`, `\sum`/`\int` without amsmath) and on e-TeX's/pdfTeX's own primitives (`\middle`): pdflatex 1 Error, base 1,
    now 0, output unchanged. Fix: add pdflatex's e-TeX/pdfTeX primitive names to `LATEX_PRIMITIVE_COMMANDS`
    (`etoolbox_sty.rs`, `\meaning`-checked), and keep the Error for a `DefMath` whose LaTeX meaning is `\mathchar"…`
    (fontmath.ltx), unless amsmath made it a macro. Post-release; 1
  - TITLESECRED — (TITLESEC reviews, OD #1143) titlesec's hold on `\section`…`\subparagraph` misses: `\titleclass` after a
    release does not take the command back (titlesec.sty:133-139); a release inside a group outlives it (the lock's
    `\lx@dropped@<sec>` is global); only a body naming `\@startsection` releases (`\renewcommand\section{\mysection}`, a
    `\let` restore and `\secdef` bodies do not; pre-existing); a hand-written redefinition copying the claimed body with
    tokens added still counts as titlesec's (needs provenance from the lock, not token containment); and sectsty under a
    class whose binding loads article (amsart, acmart, IEEEtran, mnras, revtex4-2, emulateapj, asmejour, subfiles)
    releases where pdflatex errs. RED repros in sectioning-frontmatter/ (`titlesec_*_red`, `sectsty_under_amsart_keeps_titlesec_red`); 3
  - READARG395 — (study, DG 2026-11-23) tex.web §395 ("Argument of \x has an extra }", `\par` inserted, the call
    ended) for a `}` read as a macro argument. Branch `pk/READARG` (382885e58aa, cb5d679984b, c3cc7f3ba48; OD #1140, KPE
    #1314 reserved). `A{a\x}b` with `\def\x#1{[#1]}`: pdflatex 2 errors ("Aa" / "b"), Perl and ours 0 ("Aa[]b"). Blockers
    found: 9 guards (batch77r nine-arguments and siunitx, batch66h caption and svjour3, pgfkeys `closing_brace_is_not_a_list`,
    batchregfixj `\@firstoffive`, batch56 noligs `<`, batchp26, batche0 pin) and cs0308004 (33 → 101 Errors + Fatal via
    fancyvrb's `\FancyVerbGetVerb`): each a binding that puts a macro before a `}` where TeX's code does not. Audit the
    bindings' token shapes (box family: `\@begin@tempboxa` ends with `\color@endgroup`, latex.ltx:16085-16147; ours
    `\relax` as Perl, TL 2025's `\endgraf\@endpefalse\endgroup`), then enable §395. Constructors' `{}` arguments too
    (READARGCTOR: `A{a\emph}b`, pdflatex 2 errors, ours 0). Repro `sectioning-frontmatter/titleformat_at_a_group_end_red.tex`; 5
  - ROBUSTIFYCELL — (VREFCLEV review, Perl the same) etoolbox's `\robustify` (any `\etb@resrvdb` meaning scan) inside an
    alignment cell garbles the input from that row on: "Paragraph ended before \etb@resrvdb" (pdflatex 0 `!`); 2606.11484
    (eczhandbook.cls:285 in a tabularx) Fatals after it, which VREFCLEV's `\fullref` → `\cref` path exposed (5,927 words
    on the base from error recovery; TL 2025 pdflatex Fatal too, a TL 2023-only source); 3
  - HYPERFAMILY — (INTERIM397g, OD #588) one family member's first use defines its whole family (`\phantomsection` then
    `\providecommand\nameref` is skipped) — arXiv's hyperref build behaves the same; the `ar5iv_sty.rs` comment "what
    pdflatex without the package sees" overclaims for the family's other names. Doc fix. With the provided anchor layer
    (#923, F1ANCHORTEST), a runtime `\@ifundefined{Hy@raisedlink}` / `\ifdefined\hyper@anchor` takes the hyperref branch
    in a paper without hyperref (suftesi.cls:3021 then skips its `\let\texorpdfstring\@firstoftwo`;
    interfaces-hypbmsec.sty:79), as arXiv's hyperref build does; 0 arXiv papers use them; a package testing
    `\@ifundefined{hyper@anchor}` in a later `\AtBeginDocument` hook sees hyperref loaded (no witness); 1
  - AAS5-HYPERREF — (LOOPFIX review, OD #1100) under AASTeX 5.x aas_support leaves hyperref marked loaded (as Perl), so
    after `aastex_cls.rs` restores the kernel's `\refstepcounter` a document's own `\usepackage{hyperref}` does not
    re-install the dispatch, and cleveref's hyperref branch (cleveref.sty:2044-2054) sets no `\cref@currentlabel`:
    `\documentclass{aastex}` + algorithm/algpseudocode + cleveref, `\label` in an `algorithmic` line, `\cref` → 2
    Errors (`\cref@currentprefix` expands into itself; base 0, pdflatex 0). aastex + cleveref in 0 of 35,942 census papers.
    Fix: `unmark_loaded("hyperref.sty")` for AASTeX 5.x after aas_support (A/B AASTeX 5 papers: `\@ifpackageloaded`
    turns false). Post-release; 3
  - PHYS-PROTECT — (LOOPFIX review) `\svmqty` (physics_sty.rs:1714, physics.sty:108) and `\trace`/`\Trace` (with the
    `\tr`/`\Tr` aliases, physics.sty:353-354/371-372) are `\DeclareDocumentCommand`s still unprotected here; no loop
    found (`\edef\x{\svmqty{a}}` clean). Protect them with the others. Post-release; 1
  - PHYS-COTH — physics.sty:346 redefines `\coth` as its other hyperbolic functions; doing so here moves the
    complex/physics golden (TRIGFUNCTION). Also undefined: `\homework`, `\braces`, `\fbraces`, `\argopen`,
    `\argclose`, `\opbraces`, `\trigbraces`. Post-release; 1
  - AMSTEX-ENUM — (LOOPFIX) inlined-AmS-TeX papers past the `\sum@` loop hit `malformed:ltx:enumerate` (#PCDATA, Math
    and text in an `enumerate`) from amsppt `\roster`/`\item` (math0612243, math0112238, math0303294, math0404149,
    math0404221, math0404272, math0405116, math0406482 TooManyErrors; 0705.4126, math0112286, math0211398, math0405158,
    math0404207); 3
  - LTXUTILH-OTHER — (LOOPFIX) shipped-ltxutil papers whose Fatal has another root: "Package loading depth exceeded
    500" (1408.3933, 1410.2241, 1511.02101, 1511.05750, 1602.08358), unexpected `\fi`/`\par` (1207.4642, 1211.3724,
    1211.6751, 1304.5586), `\errmessage` (1601.02763). rc2 2026-11-23: all ten still Fatal, and TL 2025 pdflatex is Fatal on every one run: shipped babel 3.9l inputs `switch.def`, TL 2025's stub of which inputs `babel.def` back (1408.3933, 1410.2241, 1511.02101, 1511.05750, 1602.08358, 1601.02763); shipped 2012 expl3 (`l3names.sty:48`) against the TL 2025 kernel (1207.4642, 1211.3724, 1211.6751, 1304.5586). DG 2026-11-23: older arXiv papers must convert even where TL 2025 fails — a strategy per family (binding over the stale shipped copy, a legacy-TL dump, …), study first; 3
  - ONLINECITE — (LOOPFIX review) `revtex4_support_sty.rs:430` `\onlinecite` → `\citealp` by user-level name, where
    revtex4-2.cls:7069 `\let\onlinecite\rev@citealp` (the OD #1100 class; no loop witness yet). Post-release; 1
  - CYCLE-WINDOW — `cycle_guard.rs` `MAX_WINDOW = 10`: a loop of 11+ tokens is not caught as a fast Fatal and runs to
    the 180 s wall clock (the apacite loop before LOOPFIX); raise it (DG call). Post-release; 3
  - TOKLIMFLOOR — a token budget below the 20G default never takes effect: `--token-limit` (`latexml_oxide.rs:734-736`)
    and latexml.sty's `tokenlimit` (`latexml_sty/mod.rs:730-731`) set it, then every file open (`mouth.rs:694`,
    `core_interface.rs:1276`) runs `scale_token_limit_to_source` (`gullet.rs:793-807`), which lifts any limit to at
    least `DEFAULT_TOKEN_LIMIT` unless `LATEXML_TOKEN_LIMIT` is set: scale the default only, never a user's limit; 1
  - IFUNDEFCS — (ruling needed) `\lx@ifundefined` reads its name with `Expand!` (as Perl's
    Base_Utility.pool.ltxml:23-31), where TeX's `\csname` stops at an unexpandable `\let` with "Missing \endcsname
    inserted" (`gullet.rs` `read_cs_name_inner` raises it); reading it as `\csname` does turns runaway loops (CCJNL397's
    author `\fmt` + `\@ifundefined`) into an honest Error, a divergence from Perl's deep recursion; since #924 such a
    loop ends in one counted `Fatal:Timeout:Recursion`; 3
  - C2FAITHFUL — #917's fallback for an alignment reverted while it is absorbed (`Whatsit::revert` try_borrows, Warning
    `unexpected:self_referential_alignment`) reverts from the definition, so the math's `tex`/alttext lose the inner
    alignment's cells: 1901.06584 reads `{T_{\Sigma,\vtop{\lxSVG@halign}}}` (an internal macro in a reversion), Perl
    `\vtop{\halign{#\cr$\vtop{\halign{#\cr L\crcr}}$\crcr}}`; release the borrow while `be_absorbed_mut` absorbs the
    cells (`mem::take` the rows, restore after); minimal `\begin{tikzpicture}\node
    {$\ooalign{$\ooalign{L}$}$};\end{tikzpicture}`. With it OOALIGNSVG: a nested `\ooalign` in tikz node math leaves
    `svg:g` open inside `ltx:XMath` (2 Errors an occurrence, 1901.06584's 57; Perl ~2 others, `svg:g` in `ltx:block`); 3
  - REGNOARGS — (no witness) `state::lookup_dimension_cs` (a single token), `lookup_register_quiet` and Rhai's
    `LookupRegister` can still run a getter-with-parameters register (`\fontcharwd`) without its arguments, the class
    #917 fixed in the panel-width scan and pgfmath: make the `\fontchar*` getters total (as `\dimexpr`) or have
    `Register::value_of` decline when arguments are missing; 1
  - TITLESECSTAR — starred (numberless) titlesec headings drop the before-code: titlesec applies `#4{#8}` to numberless
    titles too, our unnumbered path runs only the font and the after-code (`sect04.rs` ~423-431): pdflatex `[Starred D]`
    / `Star.`, ours `Starred D` / `Star` (brackets, a period, `\MakeUppercase` lost; synthetic); 1
  - ALGOVINTAGE — old shipped algorithm2e copies (#920; synthetic unless named): release 1.16's `ruled` test reads
    `\algostyle` at load (`algorithm2e_sty.rs` ~359-362), so a later `\restylealgo{ruled}` closes ": " where pdflatex
    prints "Algorithm 1 …" (decide at caption time); a 2003/10/10 copy under `french` prints `##1:` without
    `\algocf@typo`, ours " : "; old copies' `\ifalgocf@procnumbered` / "Boolean algocf@func undefined" (0810.3203,
    0911.2801); prooftrees' bare `\forest … \endforest` is still a stub, its trees not drawn, one counted Warning
    (2606.20837, 2412.10588; a forest batch); 3
  - RESETLIST — counter reset lists (#925, OD #1090): `sync_reset_list_macro` (`counter/dialect.rs:130`) rebuilds
    `\cl@<ctr>` from LaTeXML's stored list after every change, so a document's `\def\cl@X{}` / `\let\cl@chapter\@empty`
    comes back at the next `\@addtoreset`/`\numberwithin`/`\newcounter{…}[X]` and `tex_reset_list` can raise a false
    cycle Error: `\@cons` onto the macro's current body (latex.ltx:10156) and filter it for `\@removefromreset`
    (CLRESYNC; synthetic); `\stepcounter` into a cycle reports both sect08's Error and the cascade's Warning
    (CYCLEONCE); `\lx@stepcounter@within` reads LaTeXML's list, not TeX's macro (STEPWITHIN-CL; not probed); the
    two-counter id cycle (`\numberwithin{section}{equation}` where the class nests equation ids in section's →
    Fatal:Timeout:PushbackLimit; IDCYCLE, loud, no witness): record each counter's id-within and walk it in
    `define_counter_ids`; 3
  - CLUSTFIXR — run 397's Error clusters' leftovers (#921-#923): `\documentstyle[PASJadd]{PASJ95}`'s `\draft`
    (PASJadd.sty:46; astro-ph0003107, astro-ph0002344; ~10 census papers): a small PASJadd binding or the shipped files
    read raw; pasj00's `\Published{}` (pasj00.cls:113; 0712.0877, astro-ph0701328) → `\lx@add@date[role=published]` with
    its own A/B; the `\@part` lock (`latex_constructs_rust_only.rs:146`, for KOMA) refuses jpp.sty:608-609's
    `\gdef\@part{0}`, a stored number (astro-ph0207068; ATPARTLOCK); astro-ph0107268 picks the dvips l3 backend where a
    minimal psfig file picks pdftex (DVIPSBACKEND; bisect); an old aipproc `\author{A, B, and C}` with many names gives
    `merged_creators` (astro-ph0011502, astro-ph0604126; AIPPROC-MERGED, frontmatter splitting); `\introductionname`
    undefined in 79 papers — copernicus.cls:915-917 adds `english` to `\@classoptionslist` and TL 2025's babel loads
    nil.ldf (COPERNICUSINTRO; check a 5-line `\documentclass{copernicus}` + `\introduction`); 3
  - CLUST397T — run 397's next Error tier (CLUST397c; Perl-parity, pdflatex clean): `(LayAureo)` 47; dcolumn in a math
    `array` ~40; emulateapj's `\tabcaption` ~5; bisect `ltx:float` in `inline-block` (1110.3898) and
    `\lx@add@frontmatter@now` (astro-ph0010381); tabularht's `\endinput` after `\@array` (44, HARD); `(newunicodechar)`
    after CJKutf8 (40, HARD, defer); 3
  - VERBARGR — `\verb` in arguments after #922/#924: where a binding pushes an argument back without its braces
    (`\multicolumn`, `\textbf`) the verb scan still runs to the line's end — `\multicolumn{2}{l}{x \verb+a+ y}\\ c & d`
    (an extra `&`), `Pre \textbf{\verb|x| y} post.` (4 Errors, text moved); astro-ph0108286 keeps its text: end the verb
    where the pushback runs out, or push it back with braces (VERBARG-NOBRACE); `\mbox{\verb|m| after}` pdflatex 1
    Error, ours 0 (VERB-MBOX); a `\verb` in a bigfoot minipage footnote pdflatex 5 `!`, ours 0, the leniency of
    fancyvrb's `\VerbatimFootnotes` (BIGFOOTMINIPAGE; defer); 3
  - MANYFOOT — manyfoot has no binding: `\DeclareNewFootnote{B}\footnoteB{…}` is no `<note>` and "`\c@pchk@FN@totalid`
    is not a register" (MANYFOOTSERIES); every manyfoot or bigfoot load warns "`\endminipage` is not an expandable
    control sequence" (manyfoot.sty:390; MANYFOOTWARN): a small faithful binding; 3
  - CRASH397L — run-397 crashes left to root-cause (the same on every binary compared then; recheck on rc2): stack
    overflows 2209.08800, 2403.08513 (rc=134; since #924 an unbounded argument expansion is a Fatal); recursion
    2411.07443 (listings), 2411.07667 (`\collect@box`); math0012097's endless "Ignoring redefinition of \title" at
    CRclassif3.tex:40 (~3 GB stderr); 2101.07941's wall clock (LOOPFIX). rc2 2026-11-23: 2209.08800, 2403.08513 and math0012097 convert clean (math0012097 because `amsp2.sty` is now skipped as a missing file; recheck if raw loading under `\documentstyle` returns). 2411.07667 fixed by LISTINLINE (an inline listing runs `\lst@Init`/`\lst@DeInit`, OD #1139). Left: 2411.07443 (`\@listings@inline` nested 98 deep in the first `lstlisting`, Perl the same, unexplained), 2101.07941 (wall clock; deferred by difficulty, DG 2026-11-23); 3
  - REALBOXBG — (LISTINLINE review) realboxes' `\Colorbox`/`\Fcolorbox` keep their text but lose the background colour:
    collectbox digests the box before `\colorbox` (bound as `\color@box`, color.sty:158) sets the colour, so
    `\Colorbox{yellow}{x}` gives a plain `<text>` (2411.07667's ~390 `\myinline` boxes); a constructor around the
    collected box would carry `backgroundcolor`. Silent (no Warning); guard comments name it; 3
  - LSTDISPLAYINIT — (LISTINLINE) display listings (`lstlisting`, `\lstinputlisting`, `\lstnewenvironment`) never run
    listings.sty's `\lst@Init`/`\lst@DeInit`: `\g@addto@macro\lst@DeInit{!}` then `\begin{lstlisting}x\end{lstlisting}`,
    pdflatex prints "!" after the listing, we drop it; touches every display listing (own A/B); no arXiv witness; 3
  - LSTTEXRECON — (LISTINLINE review, pre-existing) an `\lstinline` in math reconstructs `tex=` from binding internals
    (`\mbox{\lstinline{{\lst@@@set@language…\@listingGroup{ltx_lst_identifier}…}}}`) where BM's rule is the author's
    user-level macros; 3
  - PICKILLGLUE — (PICNEST review, OD #47) in a picture the explicit-space keep (`document.rs` `open_text`) decides by
    the previous sibling, where TeX decides by the next item: a drawing object kills the glue before it (latex.ltx
    `\@killglue` :16785-16805, pstricks.tex:1564 `\pst@killglue`). So `\put(0,40){word\line(1,0){10}\ more}` drops the
    space TeX keeps, and `\rput(1,1){\psline(0,0)(1,1)}\ \rput(2,1){…}` keeps one TeX kills, nesting the next object
    (depth +2 per item, as on the base). Fix: drawing objects remove a trailing whitespace-only text before they insert
    (`\@killglue` in the bindings or at insertion), and the keep goes back to "always text"; RED repros for both
    shapes. No corpus witness in ~2,700 pstricks/tikz papers; 3
  - NULLDELIMPOS (after NULLDELIM, pre-existing since #373) — the fence retry (`balance_null_delimiters`) puts the
    stand-in for TeX's null delimiter at the formula's edge, not where the `.` stood, so `w\left.x\right)` reads
    `delimited-)@(w*x)` (the meaning spans `w`) and `\{x\mid x\left(y\right.\}` becomes `{ x | x ( y } )`, which
    fails to parse; when the retry still fails, its inserted nodes are unlinked but not freed (one small leak per such
    formula). Fix: place the stand-in at the null delimiter's `XMHint`; 3
  - EMPTYFENCE (low) — 1,269 empty, id-less OPEN/CLOSE tokens remain in a 635-paper sample from another mechanism (an
    absolute value inside `split`, 0803.0204; an aligned dual's content branch, `tests/parse/opfunction_arguments.xml`
    :1069 `|z_i|`); none is drawn (content side only). Find their source and drop or fill them; 1
  - PLACEB4 — (PLACEFIX, OD #1099) a block-local reference list (elsarticle CPC program summary in the abstract
    2503.17910; IEEE CSM sidebars 2402.17043, 2502.16577; hep-ex0405026's minipage) stays in place as DG ruled but keeps
    its "`<ltx:bibliography>` isn't allowed" Error: the schema allows a `biblist` only in back matter. Ruled design
    (DG 2026-11-19 "keep in place as a plain list"): a non-sectional list whose items Scan registers as BIBLABEL targets so
    citations still resolve. ~22 so far; 3
  - PLACEACK — `\acknowledgments` inside an open float keeps its Error (cond-mat0108156); the float-out of OD #1099 in the
    acknowledgements constructor would fix it; 1
  - PLACEG3 — a nomenclature inside an inline frontmatter container (`{IEEEkeywords}` 1606.00288; 2111.12783, 2402.06758)
    keeps its Error: a paragraph is not allowed there; ~8; 3
  - PLACEB1 — piparticle(-final).cls:648-666 `{theaffiliation}` does `\let\institution\bibitem` inside `\blfootnote`, so
    a whole `{thebibliography}` opens in the footnote (1001.2197, 1510.06283; 38 so far; Perl the same,
    latex_constructs.pool.ltxml:4116-4129). General fix: `\bibitem` in a `\list`-based environment other than
    thebibliography acts as `\item[\@biblabel{n}]` + anchor; keep astro-ph9806175, 2103.15495, 0801.2060; 3
  - PLACET3 — an abstract that swallows the main matter (math0006149's unbalanced `\abstract{`, 1108.5171's brace-less
    `\abstract` with no heading, hep-th0110297) keeps its theorems as `ltx:theorem`, each with its "isn't allowed in
    <ltx:abstract>" Error (OD #1097, DG 2026-11-20); ABSFIX (in progress) ends such an abstract after its paragraph
    (DG ruling), which moves those theorems to the body. Repros `block-model/theorem_in_unended_abstract{,_group}`; 3
  - ABSOWNDEF — a document's own `\long\def\abstract{…}` is refused by the kernel lock (as Perl,
    latex_constructs.pool.ltxml:1150), so 1209.6193's abstract swallows the body; DG 2026-11-20: keep the lock; 1
  - PLACENEST — (PLACEFIX final review; synthetic, 0 of 86 witnesses) a theorem dissolved into an abstract splices every
    non-`ltx:para` child as it is, so a nested `{proof}`/`{lemma}` or a `{figure}` inside it lands directly in
    `ltx:abstract` (schema-invalid, no Error; base raised "isn't allowed"). Fix in `document.rs`
    `dissolve_theorem_in_abstract`: dissolve nested theorems/proofs first; if a remaining child cannot go in the target
    (`can_contain_qsym`), keep the theorem with base's Error. Probes: an abstract holding a theorem with a proof inside,
    and a theorem with a figure inside (both pdflatex-clean); 3
  - REFMISSINGTEXT — an unresolved `\ref` renders an empty `ltx_missing_label` span ("See Theorem ."), where Perl prints
    the label text (Post/CrossRef.pm:346-348; ours `latexml_post/src/crossref.rs:1907-1910` skips it); PLACEFIX's
    dropped theorem labels add such refs (0801.2274, math0509535, math0511748, math0605416); 1
  - ABS-BLOCKPAR — (ABSFIX re-review; synthetic, no witness changed) a late-closed abstract's first paragraph is not
    ended by a block environment that follows without a blank line (`Short abstract text.` then `\begin{center}…`, or
    `\begin{theorem}` on the next line): the S1 display exemption narrowed `split_after_first_paragraph`'s `internal_par`
    arm to "followed by a vertical skip". Fix: an `internal_par` ends it unless it borders a display. Base is worse on
    all three probes (`\abstract`+center/theorem without a blank line, closed by the bibliography or the end); 3
  - ABS-SHORTFIRST — a short first line (a PACS line) set before the real abstract is kept as the whole abstract and the
    rest moved (`\abstract \noindent PACS: 11.15\par We show…`; no witness among the 33 swallows); 3
  - ABS-BIBFORMS — net A covers `\begin{thebibliography}`/`\bibliography` only: `\printbibliography`, a bare
    `\thebibliography`, `{references}` and a binding-constructor `thebibliography` do not end a command-opened abstract; 1
  - SVMULT-PAREND — modern svmult's `\abstract{` with its `}` lost: TeX errs "Paragraph ended before \@abstract was
    complete"; we give only the open-group Warning; 1
  - ABS-ENV-OPEN — a `\begin{abstract}` never ended keeps the whole paper with its theorems and its Error
    (cond-mat9410055); net B covers command forms only; 3
  - ABS-MOVECOUNT — the moved-words Warning glues words across block edges ("Body theorem." then "Text." counts 2); 1
  - UNSAFE937 — after v0.8.0 (DG 2026-11-20): move away from `unsafe` per issue #937 — 110 sites at #935 (~89 shipped,
    ~21 test-only), almost all at OS/C-library boundaries. Phases: an unsafe-count ratchet; Unix syscalls → `rustix`
    (a direct dependency since #935; `lsp_server/unix.rs`, `watchdog.rs`, getrusage/flock/geteuid/dup, `process_group(0)`
    for `setsid`); glibc `malloc_trim`/`mallinfo` → the allocator's safe API; no process-env mutation after startup;
    safe libxml/libxslt wrappers upstream in our crates; `unsafe impl Send/Sync`; isolate the rest (signal handlers,
    Windows/macOS queries). Target ≤ 15 shipped, 0 test-only; every PR byte-identical on a fleet A/B and perf-neutral; 5
  - SECR — the Lua sandbox's residuals (#911, #924, #935; OD #1081): carry the Lua protocol over a socket pair
    (`UnixStream::pair()` via `Stdio::from`; `/proc/self/fd/N` cannot reopen a socket, ENXIO), closing the race where a
    chunk reads its own pipe and swallows its refusal record (records are best-effort today); a newline in a refused
    name forges log lines (`Fatal:forged…` at column 0 of the `.latexml.log`; status and counts unaffected): escape
    control characters in `lua_bridge.rs` `shown()`/`error_text()`; a parent SIGKILLed from outside (the kernel's OOM
    killer) orphans a spinning texlua (`PR_SET_PDEATHSIG` needs `pre_exec`); defence in depth (LUAOSBOX): an OS sandbox
    of the texlua child — Linux Landlock (read texmf and the source, write the job dir) + seccomp (no
    execve/fork/socket), a macOS sandbox-exec profile, a Windows Job object (no child processes) + a restricted token; 5
  - PARANOIDIN — (DG 2026-11-18: "not now") a document reads any file the user can read (`\input{/abs}`, `\input{../…}`,
    `\verbatiminput`, `\openin`, `\includegraphics` copying it into the output, Lua `io.open`, `os.getenv`), as under
    TeX's default `openin_any=a` and in Perl; kept for later: a `--paranoid-input` (`openin_any=p` semantics: the
    unpacked source root, texmf and `--path` after symlink resolution), on in cortex_worker; 3
  - IMGINTENT — after v0.8.0, a dedicated sprint (DG 2026-11-20: "addressing the full challenge and complexity, and
    testing against the full diversity of arXiv"): image sizing that keeps the source's width intent (issue #721,
    ar5iv#83). Today `width=7in`, `0.8\linewidth`, a length register and a macro all reach the XML as pt
    (`options="width=505.89pt"`) and the HTML as a px `width`; LaTeXML.css caps every image to its minipage
    (`.ltx_minipage > .ltx_graphics{max-width:100%}`, as Perl) and ar5iv makes nested landscape images fluid
    (`width:auto`), so #721's 7in image in a `.5\textwidth` minipage shrinks where PDF and tex4ht keep 7in. Goal:
    relative widths (fractions of `\linewidth`/`\textwidth`/`\columnwidth`/`\hsize`) scale with their box, absolute
    ones keep their size capped only by the reading column and the screen, natural size (no width) stays fluid.
    Work: (1) classify while the width's dimension is evaluated (literal and macro widths; a length register already
    holds a plain length, so it keeps today's cap), on every sizing path (`width`/`height`/`totalheight`/`scale`,
    `\resizebox`, `\scalebox`, `\epsfig`/`\psfig`/`\epsfxsize`; `latexml_core/src/util/image.rs`), marked by a class
    on `ltx:graphics` (OD record; Perl marks none); (2) CSS in LaTeXML.css (local delta, offered upstream) and ar5iv-css:
    the box cap and ar5iv's fluid rule exempt absolute images, everything is fluid again below the reading width, and
    side-by-side panels never overlap (exempt only a box alone on its row, or take the row from the float panel grid,
    task #360); (3) test against all of arXiv: count the images whose rendering changes (stated width > box; alone on
    its row vs side by side; narrow screen), then a visual A/B over a sample stratified by class, column layout,
    subfigure grids, tables and natural-size rasters; acceptance bar: no new overlap or overflow. Risks: misread
    registers, panel overlap, phone overflow, two-stylesheet specificity. Witness: #721's MWE; 5
  - IEEEROUTE-ABBREV — an IEEE affiliation note with "St."/"Univ." + capital, or a "D.S. Name" sentence start, keeps the whole note unrouted; 2011.08553 (probe p2).
  - IEEEROUTE-CUTLIST — the content check does not read a note's routed contacts as one printed text (B3 cleared via B2c only).
  - IEEEROUTE-PARTIAL — (REVIEEEB S2) 2204.03507: Torrisi's "also with INSTM" is routed while his main note "A and B, are with …" stays whole; fix: leave an author's "also with" note whole when an earlier note of his stayed whole, or accept the comma.
  - BLOCKPLACE-ZONE — (DG 2026-11-17: general, DRY, reusable) the opt-in landed (`\lx@names@note`, OD #1074: a note moves under the names only where its binding declares it, so round 1's three `_zone=footer` declarations are gone); what remains is the engine rule for a names-declared command used inside an author's footnote (`\AUTHORS{Ann\footnote{Univ X \EMAIL{a@x}}}` shape): a note made while a name's note is digested is the footer's, as a contact inside a note already is (`frontmatter_api.rs`, the contact-in-note rule). No witness among the declared classes yet.
  - SPCONF-TYPEDNUM — a typed `2 Univ B` under a `$^2$` name continues the line above (synthetic only; probes `REVSPCONFb/{e,f,h}.tex`).
  - SPCONF-BLOCKEMAIL — e-mail lines inside spconf's `\address` keep the note whole; 2407.20176 (2-for-2 unplaced). (the fail-safe of c9b7845ce4d; placing them needs an owner rule stricter than `listed_address_owners`, which misread 1802.09371's "Aline, Roumy")
  - SPCONF-ISCLS — Interspeech.cls does not share spconf's `\maketitle` address decision: key-marked names `\name[affiliation=1]` are unseen by the line reader.
  - SPCONF-NAMEPLUS — spconf `\nameplus` leaves `<sup>1</sup>` in the affiliation; 1309.1501 (its doubled mark).
  - JHEP-ADDRLIST-ORDER — jheppub's `\lx@add@contact@list` order step leaves the unspelled third address of a 3-for-3 list unplaced (probe `\author{Ann Able,}\author{Bob Baker,}\author{Cy Coe}\emailAdd{ann.able@a.org, bb@b.org, zz@a.org}`); found by SPCONF round 1, whose nameless block creators SPCONFb removed.
  - DATELEG-LEGEND — (DATELEGb) the legend-note rescue was CUT: 1507.08926, 2508.05506, 2404.15649, 2602.04557 keep their legend notes unrescued.
  - DATEICML — DATELEG NOT DONE: notes the flush places — icml 2602.04557, MRM `\corres`.
  - DATEMACRO — DATELEG NOT DONE (the `\@date` store / date macros; detail not given beyond the ID).
  - AUTHORNLABEL — DATELEG NOT DONE (detail not given beyond the ID).
  - SETCOPY-FLAG — (OD #1078) a store the class's replayed setter changed, handed whole to a frozen setter copy (`\orig@author{\toc@authors}`), reads as no events of the copy's block: each name is written as plain text with the per-name Warning "a name its events don't spell" (names correct, markup lost): tocbase 1811.06821 ×3, 1811.11810 ×3, 1904.05269 ×5, 1905.07135 ×4, 2311.03341 ×4; `setter_copy/tocbase_shape.tex` pins W=3 (`perfect_kernel_batchsetcopy::a_copy_in_an_end_code_takes_the_class_list_once`); fix: the copy's call reads the store's digested names as its own events; 3
  - ANDSTORE — `\author{\store}` with a store opening with `\and` (`\newcommand\toc@authors{\and Christian Elbracht}`, no copy) invents the word "and": one creator "and Christian Elbracht", 1 Warning, where pdflatex sets the name alone (an empty first column); main 8b298f9c0d6 the same; RED catalog `sectioning-frontmatter/and_store/direct_store.tex`; fix in the author split: a leading empty name before `\and` is no name; 2
  - MARKORDER — notes' document order differs from the PDF's mark order ("Ann Able † ∗" vs PDF "∗ †"; HTML NEITHER ×5): `elsarticle_fntext_shared_by_two_names_is_their_marked_note`, `author_thanks_shared_by_footnotemark_marks_kept`, `ne2c_mark_after_footnote_glued`, `ne2b_counter_moved_between_notes`, `ne2e_mark_past_spacing_is_its_names` (+ revPRESBXb S2 RED, the WRONGFIX folded ∗ vs PDF text * order swap); engine order; low priority.
  - TABCELLMARK — a prefix mark opening the next tabular cell is carried back across the cell to Ann, silently (FMRG suite).
  - SEPTEXT — mark separators `;`/space are not read; with the `\lx@sup@request@affiliation` digestion-path record and authblk `\author[1,*,\textdagger]` † landing on the block.
  - MARKGONEXT — the reading side's `mark_goes_next` (`lines.rs`) still uses source distance where the emitter decides by TeX gap widths (nearer name, tie → next; DG ruling); converge later.
  - SHGIVEN-TEXTSC — `\textsc` next name (probe s4): a spacing-separated mark is a counted loss instead of carried.
  - UNMARKEDDUP — identical unmarked author-note copies print twice; wait for a real witness on the int head (2602.18802 is already marked); the exact fix is identity (issue #893 Option A idref), not text.
  - DOTCUT — int's generic cut bug behind AAAI dotted marks (revPRESBXb P2, "high"; RED rows c1/c7/n1/n2/n4/b1g + P1-P3); 2509.03329 went to base `\thanks` per use as the fail-safe.
  - OWNLEGGLYPH — an own-legend `\thanks` printing another counter's glyph is not kept by its author (b6; 12 corpus papers; pre-existing; OD #1068).
  - COMBLEG — a note that legends several of the names' marks should be the block's note (whole text, every carrier's mark linked, margin/after the block), never under one author nor split ("Correspondence to: qw281@" follows the ‡ clause but is Wei's); ‡ unlinked today; low priority, reach unmeasured (census 30k).
  - DOCROLES — OD #1063's document-level notes: dates (and other roles) shown behind a role census; role=altaffiliation is not in `f:document-note-shown` (post-merge).
  - AUTHBLKINK — (INVDAG S3) authblk ink residual; detail not given beyond the ID.
  - P1-LOGOS — (stretch, DG run 391) 2610.22523's `#p1` is left holding only the logo images once its affiliations are extracted — delete a paragraph emptied by extraction (or one of graphics only that the class's title block owned).
  - QA61-STORES — QA61 D1 post-merge: (a) K11's `store_lines` readers keep the top-level-line hole (RED `table_store_in_a_top_level_line_with_another_value_red`; base leaves "E-mail: , tel. 555"); (b) `\lx@replay@records` for K11's handed stores (`note_store_handed`) takes a label's first match; (c) RED `offtable_store_moved_leaves_the_rest_of_its_line_a_paragraph_red` ("(corresponding author)" left a paragraph); D2/D3 (unlabelled printed stores; keywords/classification as `ltx:keywords`/`ltx:classification`) await rulings.
  - CJKGB — GB2312 mojibake inside "(…" read as a mark (`lexer.rs:317`), ")" lost; CJK GB decoding is silent where Perl errs.
  - SPCONF-EMAIL — spconf `\email` needs an after-authors `\maketitle` hook; 2405.13379's e-mails stay a note (the note now sits under the names, `\lx@names@note`, af9dc7b87af).
  - BLOCKPLACE-DECL — `\lx@names@note` declared in jfm, mdpi, ieeeaccess, oup, autart, sigma, spconf, Interspeech only; the other elsart-family clones are unverified and ieeetaes stays undeclared.
  - BLOCKPLACE-AUTART — 19 autart papers: the comma-junk titlepage now follows the moved addresses (pre-existing junk).
  - AAAIAFFIL — AAAI affiliation marks residual (revPRESBXb); detail not given beyond the ID.
  - AAAILABEL — AAAI notes kept as moved (once-per-name `lx_aaai_marked@<kind>`); the label reading stays open; no aaai binding (OD note).
  - GTPART — gtpart.cls `\urladdr`, `\subject` undefined (MTREDUCE residual; `\corresponding` bound).
  - TITLENOTELAST — stopped, no binding, post-release: sig-alternate/acm_proc_article-sp title note order; 1003.2281 (70 census papers).
  - JASAPACS — stopped, no binding, post-release: JASA PACS notes; 0803.0204 (4 papers).
  - REVGRIDWARN — REVTeX twocolumn grid Warning wording; post-release.
  - NOMTHOOK — article with no `\maketitle`: we print † where pdflatex prints 2 (pre-existing; ≤ 56 papers); RED pinned.
  - TITLEPAGEMARKS — (post-release; ~9/30k papers) report/book/`[titlepage]` article print the arabic `\thefootnote` in force; we write none (fail-safe, MARKBINDd removed the fix); one line in `declare_fnsymbol_title_block`'s title-page branch, large reach → own A/B; RED rows report titlepage + titling fnsymbol, t1/t17.
  - AUTHRESET — `article_replaced_author_steps`: ‡ § where pdflatex prints * † (pre-existing wrong glyphs the old model hid).
  - ICMLSTY — the bundled icml sty notice text (INVDAG row; detail not given beyond the ID); icml `*` marks collide with LaTeXML.css's blue "*" author-notes trigger (S3).
  - BMCART — 2112.10068 bmcart (unbound): the corresponding link is lost, five `\author[email=]` keys (rubric "wrong connection"). (verify on the next sandbox run: the bmcart binding landed in 905b2578ae2)
  - REGCDE-RES — loader residuals: (a) a package reloaded in the body with NEW options gets no reopen of the preamble commands; (b) a body `\DeclareUnicodeAccent` drops "TU" that lualatex prints; (c) RED `loader/loader_is_not_notprerr_red.tex` (Perl's 7 loaders Error-then-load, scrlttr2.cls:2313); amsmath never adds `\DeclareMathOperator` to `\@preamblecmds` (RED `loader/declaremathoperator_is_preamble_only_red.tex`, pdflatex 3 `!`, ours 0); an undefined command used twice gives 1 Error (pdflatex 1 per use); a sub-file with no `\begin{document}` keeps its text with 0 E (pdflatex 1 Error, drops it; census first); an unknown textcomp option is `Info:unexpected` (pdflatex Error).
  - REGABBOLD — a `$` nested in an `\hbox` does not inherit `\bm`/`\boldsymbol`'s bold (`\ext@arrow` inside `\bm` loses `font="bold"`; RED `arrowfill/bold_ext_arrow_in_hbox_red.tex`); NIT: make the `ext_arrow_hmode_fill` guard row whole-element.
  - TITLE-NOTE-SPACE — the title's pubnotes span sits inside `<h1>` with no whitespace before it, so the heading text (copy/search/accessible name) reads "ActionsThe work of…" (1207.6084); add a space, or lift title notes out of the h1 as the `_meta` pubnotes are (html_feedback#6886).
  - CSSHL-scroll — copying at an off-screen margin note gives no hint (the shared-copy highlight, fix/shared-note-copy-highlight 466a511ac51; ar5iv-css c9b1e99).
  - PNOTE2 — (SHARED, ≈300 arXiv) a pubnote under a cover that stands for the title is lost in the HTML: cernphprep `\Collaboration{ALICE Collaboration\thanks{…}}` (1311.0214, 1406.3206, 1609.07104, 2309.03788); `LaTeXML-structure-xhtml.xsl:379` skips the title under an inked non-`_before`/`_continued` cover (Perl :359), so `maketitle`'s pubnotes never render (:530). Fix (EASY): one `f:titlepage-stands` used at :379 and in the `ltx:titlepage` template (:390-407) to call `pubnotes` with `../ltx:pubnote`; RED `scratch_PNOTE2/rep/titlepage_thanks_pubnote_red.tex`; side lead 1006.5322 (H1/ZEUS).
  - PNOTEAR — revPNOTEA: DRY `LaTeXML-structure-xhtml.xsl:710-728` (titleless head block) with `maketitle`'s tail (:472-498) as one named template; a titleless document's never-printed `\date{\today}` shows the conversion date (1504.03307, 2510.21368; titled-without-`\maketitle` the same); 1112.0709's two `role="journal"` pubnotes (`\jname` + raw IAU-TrA header); a titleless `--splitat=section` document shows "Date:" on every page and links the root as `TEMPORARY_DOCUMENT_ID` (base the same).
  - LEADPANEL — a rule leader between two panels on one line splits the float's panel row (RED `leader_between_panels_one_row_red`; LEAD805 covers the other two REDs).
  - CHRONO — AFTER v0.8.0 (DG 2026-11-18: not replaced for the release). `chrono` is unmaintained (RUSTSEC-2026-0543, chronotope/chrono#1768, no CVE, "no safe upgrade"), ignored with its reason in `deny.toml`; a direct dependency of latexml and latexml_engine for the local date/time (`identity.rs`, `source_date.rs`, `tex_job.rs`, `pdftex.rs`) and also reached through `pericortex` 0.2.9 (cortex feature, `logger.rs` `chrono::Local`), so the ignore goes only once both are replaced. Replacement: `jiff` (thread-safe local zone from TZ / /etc/localtime; never `time`'s `current_local_offset`, which refuses in a multi-threaded process → silent UTC); a trial (chore/deps080, not landed) measured, on the distribution recipe, Linux 120 → 120 crates (−chrono −iana-time-zone, +jiff +jiff-core), Windows +3 (bundled tzdb), macOS −1; note jiff hours are `i8` (`60 * hour` must widen) and `\pdffilemoddate` needs its own `±hh'mm'` formatter, and jiff's `Timestamp` holds years −9999..9999 only; with it PDFMODZ — `\pdffilemoddate` at UTC offset 0 gives `D:…+00'00'` where pdfTeX (utils.c `makepdftime`) writes `D:…Z` (`pdftex.rs:351-364`; trigger `TZ=UTC`, `\pdffilemoddate{x.tex}`); 3
  - WINSTORE — (DG 2026-11-18, issue 775) the Windows binary ships unsigned: SignPath Foundation declined our free-signing application (re-apply once in heavy mainstream use) and paid signing is out (DG 2026-11-16). Plan: a Microsoft Store MSIX package (Microsoft signs it at certification; registration is free) with a `windows.appExecutionAlias` so `latexml_oxide` runs from any terminal, built in `release.yml` from the same executable as the Releases zip; the publisher (arXiv or DG) is decided with the Store account; the code signing policy then names the signed channel. Still open on the issue: whether the reporter saw SmartScreen, Smart App Control or a Defender detection — post-v0.8.0 (0.8.x); 3
  - MARPALIC — replace libmarpa's LGPL C in marpa (`marpa_obs.[ch]`, GNU obstack, LGPL-2.1+, incl. PERFJ's modified `c/marpa_obs.c` overlay; `marpa_avl`/`marpa_tavl`, libavl, LGPL-3.0+) with MIT code, release libmarpa-asf-sys, then drop `deny.toml`'s per-crate exception (DG 2026-11-15 "exception now, rewrite later"; license field fixed in 0.3.1); with it the fork's pre-existing `cargo fmt` drift in 6 files (PERFJc) — post-v0.8.0; 5
  - MATHTAIL — 2106.09072 (PDFCMP X/P1 34.6; PDFX1): math parse = 96.6 % of instructions (`--nomathparse` 129.3 G → 4.4 G); one merged eqnarray (S2.E19, src l.191-196, ~24 |·| pairs, ~224 lexemes, 20,003 and-nodes) runs the tree iterator 3× through the bar-depth retry (`parser.rs:3220-3235`, Perl MathParser.pm:813-836), building 5000 trees whole and pruning them each pass = 4.5 s of 6.7 s, still `ltx_math_unparsed`; the ASF second chance misses by 3 and-nodes (`ASF_SECOND_CHANCE_AND_NODE_LIMIT` 20_000, parser.rs:287/:2655); control 2107.00424 tree iterator 1.6 % self — math parked: data-structure levers first (reuse the enumerated trees across depth retries if the forest is identical, verify; cheaper build/prune per tree), any threshold change only by DG ruling (it changes output); reach: count papers with > 1 s in tree iteration from a sweep's PARSE_AUDIT — post-cutoff; 5
  - PERFBR — mimalloc override (PERFB, branch `perf/mimalloc-override` 092c6abd1a2) DROPPED for this release (DG 2026-11-15), to revisit after round 2 since the node-registry lever changes locality: PERFBs (60 papers, ar5iv recipe) wall −1.30 % total / −4.23 % median, cycles +0.26 % weighted, RSS −1.29 %, gain in sub-second papers; book790 +2.8 % cycles (mimalloc's libxml2 placement, BIGDOC); 2309.12794 +8.7 % wall; if revisited: `cfg(all(target_os="linux", not(sanitize="address")))` (ASan blocker), the #790 guard → ASan is the Linux check (06_cluster_math.rs:348-363 perturb child, STABILITY_WITNESSES.md:583 caveat), `util/heap.rs` "whole heap" under jemalloc, recalibrate cortex_worker's RLIMIT_AS constants (cortex_worker.rs:1035/:1047); 3
  - BIGDOCR — BIGDOC's large-document levers (book790): `node_boxes` kept after build (~2 GB of the Rust side, document.rs:102; Perl the same); a failing formula's transient parse spike (one 73-lexeme formula 3.86 → 6.72 → 3.86 GB in 1.5 s; PERFI cut book790's peak to 4.5-4.6 GB; math parked); CLI-only skip of end-of-run frees ≤ 2.1 % (not the fleet: cortex_worker reuses its process, cortex_worker.rs:1004-1030); 3
  - MEMR — MEM1's levers not taken (`agent_reports/report_MEM1.md`): L13 marpa-asf `Rc<RefCell<Node>>` per step (b250 1.0 %, typical 0.16 %; hard), L14 append_tree attribute HashMap (document.rs:8408-8411, b250 0.26 %), L17 decode_qname Strings (model.rs:916/925/926, b250 0.15 %; MEMC's try reverted), L16 get_tag_action_list Vec (b250 0.11 %), L15 revert `tokens.clone()` (2308 0.11 %; hard), L4c/L4d/L4f (0.06-0.08 % each; L4d = `Rc<Font>`); the clippy allocation-lint inventory never run — post-cutoff, by reach; 3
  - PDFOUT — PDFCMP's unprofiled outliers (32 papers, X/P1 CPU median 1.24, X/PF 0.41; DG 2026-11-15: release-acceptable, perf no longer release-gating): 1609.09534 X/P1 6.5, 1506.03731 5.3 (post, images) — profile both, post-cutoff; 3
  - FLEETR — FLEETTMP's residuals (revFLEETTMPb; pre-existing or test-only): `.dims` sidecars whose entry is gone are never reclaimed — prune skips every `.dims` name (`latexml_post/src/graphics_cache.rs`); let it drop a `.dims` with no entry (the fleet cache already holds some); `leased_archive_dirs` (`latexml_oxide/tests/cortex_worker_exit_reports.rs:~249-257`) counts only dirs still holding `<TASK_ID>.zip` — assert `task_dirs(slot).is_empty()` before the respawn instead; the exit-sweep marker check has ~1 s of margin (pericortex's fixed 1 s respawn backoff) and fails loudly if a nextest thread stalls longer; pericortex: create the `cortex_task` TempDir after the lease and stop SIGKILLing idle children (`worker.rs:216`, `harness.rs:686-708`, a crate release); 3
  - GCACHECHURN — (DG 2026-11-18: not the source of the fleet's speed; an optional perf lever) the graphics cache
    (`latexml_post/src/graphics_cache.rs`, LRU cap `LATEXML_GRAPHICS_CACHE_MAX_MB`, default 2048) turns over in ~1.2 min
    at fleet rate, 0.2 % of its entries are hit again, and every insertion's `prune_if_over_cap` walks and stats the
    whole cache: measure off (`LATEXML_GRAPHICS_CACHE_OFF=1`) vs 2 GB vs 50 GB on ~200 papers and set the fleet's by
    throughput, output byte-identical; 1
  - SUBCAPLIST — subcaption's `\subcaptionlistentry` and `\subcaptiontext` (subcaption.sty:253-254) are unbound (undefined control sequences; 2404.12080, 2503.09244 use the first): bind them faithfully — enter the sub-type (running `\caption@subtypehook`) and `\captionlistentry[sub\@captype]` / `\captiontext` under `\@captype` = `sub<type>` (WINSUITEb; minimal: `\begin{figure}\begin{minipage}{.3\textwidth}\subcaptionlistentry{x}\end{minipage}\caption{F}\end{figure}`); 1
  - PHANTOM2 — a second `\phantomsubcaption`, or a `\subcaption` after one, in the same group (a minipage, or a figure with one image and (a)(b) labels) re-runs `\caption@subtypehook` where pdflatex runs it once (`\setcaptionsubtype*` sets `\@subcaptype` for the group, caption.sty:305-327; our gate reads `\@captype`, which `\phantomsubcaption` never sets) — fix: a group-local "sub-type entered" flag set by `\phantomsubcaption` and `\lx@subcaption@type`, read by `in_caption_subtype`, the `\phantomsubcaption` gate and `\subcaption` (not `\@captype`, which drives numbering), plus a two-`\phantomsubcaption` row in `subtype_hook_runs_once_per_sub_type` (revWINSUITEb, probes `/home/deyan/.claude/jobs/356a0c47/tmp/revWINSUITEb/p1.tex`, `p6.tex`); matters only for a non-idempotent hook (the first aid's `\let` and zref-clever's property are idempotent); `\subcaptionbox` runs it once where pdflatex runs it twice (its trial pass, caption.sty:465-489 — documented, not mirrored, OXIDIZED_DESIGN_DIVERGENCES #853); 1
  - TILDEFN — a `~` in a file name becomes `\lx@NBSP` (`latexml_engine/src/plain_constructs.rs:472` overrides latex.ltx's `~`, which is a literal `~` inside `\csname`, latex.ltx:9413-9419, and in file names, :19727-19730); `\input{a~1/child/body}` / `\subimport*{…/a~1/…}` → `Error:missing_file:a\lx@NBSP1/…` where pdflatex reads the file (Perl errs too); fix: define `~` per latex.ltx (protected; literal inside `\csname`) and have `\lx@set@path`/`\lx@append@path` use `gullet::expand_as_csname_text` — changes Linux output (`gullet.rs:3794` maps `\lx@NBSP` to U+00A0 in label names, CLUSTER-NBSP) — SHARED, MED (found by the Windows triage: 8.3 short names carry `~`); 3
  - CURCTRA — aliascnt: `\@currentcounter` gets the root counter (`thm`) where latex.ltx gives the alias (`lem`) (`latexml_core/src/binding/counter/dialect.rs:1150-1162`: when `is_counter_alias(ctype)`, define from `ctype`; pdflatex `[a:thm][b:lem]`, ours `[a:thm][b:thm]`); llncs `\@spothm` same path; readers zref-counter and zref-clever's type; NITs: `ExplodeText!` makes `@` catcode other where `\edef` keeps it a letter (`\ifx` on `ALG@line`-like names), the repro headers cite unbtex.cls:485-493 where the `\AddToHook{label}` is at :495 (revCURCTR); 1
  - MHCHEMPROT — REGCHEM's residual (revREGCHEM): a protected user macro hiding a math shift (`\NewDocumentCommand{\nd}{}{$\alpha$-\ce{SiC}}`, `\protected\def\pd{…}`, then `\ce{\nd}`) is not detected by `shifts_math` (`latexml_contrib/src/mhchem_sty.rs:~286` skips protected macros — load-bearing for `\ensuremath`), so it still errs "Missing $ inserted" (pdflatex 0); and the scan does not know the mode, so `$` inside `\ifmmode…\else$…$\fi` or a user `\mbox{$…$}` declines a formula that would have worked (content kept, 0 E); possible fix: follow protected macros taking only `\ifmmode`'s true branch (units are set in math) — probe `/home/deyan/.claude/jobs/356a0c47/tmp/revREGCHEM/p2/p.tex`; no witness yet; 1
  - REGFIX — DONE (int/regfix): the cutoff 999af99497d (fleet rebuilt, dumps on the distro TL) reran 2605 (run 377: F 8, E 881 vs
    run 375 9/875) and 2606 (run 378: F 13, E 873 vs run 376 15/863); no new Fatal, 27 papers newly in Error, all from PRs merged
    2026-11-14/15 (`agent_reports/report_CUTSBX_2026-11-16.md`). Fixed: (A, 17; F, 1) bindings define what their files define
    (`\@enhook`, `\TPT@defaults`, `\ps@opening`, the acmart/elsarticle/REVTeX/amsaddr/icml lists) and print the lists their classes
    print with the title page (`\lx@frontmatter@footnotes`, OD #1036; KNOWN_PERL_ERRORS #1283); pdfcomment's soulpos registers;
    (D, 1) cleveref's first-aid `\ltx@label` repair gated on v0.21.4, amsmath's `\ltx@label` reset at begindocument/end (OD #1048;
    2605.07565's silent key corruption); (B, 6) #866's rehearsal: a title-code command (klmm `\instref`) assigns in its own run
    (mouth-depth aware), a paper's macro around it still declares the rest, declarations replay under the author markup, OmniBus's
    `\orcid` bound (2606.13132's iD restored); (C, 1) the preamble walk moves a binding's lock to `\lx@onlypreamble@<name>` and back
    (2606.22730's body `\renewcommand{\doi}`, OD #226); (E, 1) mhchem declines to its own output when a macro expands to a math shift.
    Guards perfect_kernel_batchreg{bind,rehearse,chem}, error_boundary (REGLOCK rows).
  - FMLISTNOTE — REGFIX residual (revREGBINDb): plain (non-`\footnotetext`) text in a class's title-page list is typeset as a body
    paragraph after the frontmatter, where pdflatex prints it in the title block and FIDELITY_CHARTER.md:168 puts unclaimed title-page
    text at the block as an `ltx:note` (2606.06621 "contributed equally", 2605.22682's preprint box, elsarticle's cnf line; minimal
    acmart `\g@addto@macro\@authornotes{Plain text.}`). Fix: route it through `hand_net_stores`' block note
    (frontmatter_stores.rs:5617); a note holding `\par`/`\vbox` material is the design step. Content kept (formerly lost with an Error).
  - ICMLCORR — revREGBINDb NIT (synthetic): a document `\gdef`ing `\icmlcorrespondingauthor@text` without calling
    `\icmlcorrespondingauthor` leaves `\lx@icml@corr@built` undefined, so the line becomes a pubnote without pdflatex's
    "Correspondence to: " (icml2026.sty:530; icml_support_sty.rs:167). Fix: an undefined built-macro means the document replaced it.
  - REHEARSEARG — revREGREHEARSEb (synthetic): a run-time-built definition a paper's macro passes as a title-code command's ARGUMENT
    (`\instref{#1}{\expandafter\gdef\csname aff#1\endcsname{#2}}`) reads as inside the command's run and is not the block's; the
    body's `\csname aff1\endcsname` is empty, no diagnostic (RED `sectioning-frontmatter/title_code_command_argument_building_a_definition_red.tex`;
    a kernel run's argument loses the same way on CUT999). Fix: a title-code run drops only the names its own body names.
  - REHEARSER — REGREHEARSE residuals: a line-broken ORCID iD (`orcid.org/0000-0002-⏎9546-4195`) keeps the line end as a glyph
    (2503.20847: an honest 3-word missing + 3-word excess Warning) and its surnames' case (PDF fidelity); five creators sharing one iD
    in different spellings warn 6 lost words (synthetic, `revREGREHEARSE/p2/c.tex`: placed links count once per target). LOSTA's two
    rehearsals without `author_markup_begin` stand.
  - DOIURL — idea, not scheduled (DG 2026-11-16): auto-enrich URL markup that links to a DOI (`doi:` text, `https://doi.org/…`) into
    the DOI link a binding's `\doi` would make; witness 2606.22730 (imsart over OmniBus, whose body `\renewcommand{\doi}` keeps
    the text unlinked, as pdflatex prints it).
  - MUTOOLTMP — DONE (fix/mutool-page-svg): mutool's `%d` is its OUTPUT page counter, so every `\includegraphics[page=N≥2]` vector PDF fell to pdftocairo (a different intrinsic size from page 1) and left `.<stem>.<nanos>.mutool_svg1.svg` in the destination, hidden by the graphics cache on hits; the temp now lives in a private directory beside `dest`, and SVG cache keys are salted (`SVG_KEY_VERSION`) — guards `graphics::tests::vector_svg_of_page_two_is_page_two`, `graphics_cache::tests::svg_keys_are_salted_and_png_keys_are_not`; WISDOM #104; witnesses 2305.12780, 1906.10268, 2406.07459, 2406.14311, 1709.02311, 2512.04096 (its `.pdf_tex` overlay layers now align); 0
  - MUTOOLPCT — a `%` anywhere in the destination path is read by mutool's `-o` as its pattern (`pct%20draft` → `%20d`): every vector SVG, page 1 too, silently falls to pdftocairo (pre-existing; revMUTOOLFIX); fix: run mutool with `current_dir` = the private temp dir, `-o page%d.svg`, an absolute `source` — post-cutoff; 1
  - SVGSIZE — mutool's unitless SVG root size is read as CSS px (`latexml_core/src/util/image.rs:1784`) while pdftocairo's `pt` is 4/3 larger, so an unsized vector image shows at 595 px for A4 under mutool vs 794 under pdftocairo — which converter ran decides the intrinsic size (pre-existing; revMUTOOLFIX P2); decide the intended unit (TeX bp → CSS px at 96/72) and apply it to both — post-cutoff; 3


  - FMPOST — frontmatter marks: tricky cases declared out of scope for the shared-note campaign, post-release (DG
    2026-11-16: "add a red/green TDD test or declare them out of scope with a witness note for later work"). Each line:
    witness → what pdflatex prints / what we output. In-scope items are in `agent_reports/2026-11-16_brief_MARKKEY_R2.md`
    and the MARKBIND branch, not here.
    - FMPOST1 ASTERISKGLYPH — `Ann Able*\thanks{X}` (batchpure guard): pdflatex text `*` then math `∗`; we write the
      mark-only note `∗` and the counter note `*` (code points swapped; both asterisks, one charter key); 1
    - FMPOST2 SCALEBOXGAP — 2412.20504's title `\def\RETAKE{R\scalebox{0.8}{E}T\scalebox{0.8}{A}K\scalebox{0.8}{E}}`:
      pdflatex "RETAKE"; we render "R ET AK E" (inter-box gaps); 3
    - FMPOST3 REVALTWARN — `fmmarks_revtex_altaffiliation_then_shared_thanks_red`: a spurious content-check Warning for
      "Univ B" (printed as Ann's altaffiliation and the others' affiliation); 1
    - FMPOST4 MARKLEDCELL — `Ann\thanks{X} \and \footnotemark[1]Bob Baker` (`ne2i_bare_mark_opening_a_cell_writes_the_block_whole`):
      pdflatex Bob a name with ∗; we read Bob as an affiliation (as at the base); 3
    - FMPOST5 FILL0PT — `fmmarks_mark_then_hfill_before_next_red`: a mark beside `\hfill` in a kernel `\and` segment is
      0pt-wide only on the class's own `\maketitle` (article.cls:181-183); undecided today, waits on T1/SHAUTO; 3
    - FMPOST6 REFMIXSPELL — `\IEEEauthorblockN{Ann\IEEEauthorrefmark{1}\IEEEauthorrefmark{2}, Bob\authorrefmark{1}}\IEEEauthorblockA{\authorrefmark{1}Univ A}`
      (revPUREc q1, synthetic): Ann loses Univ A (relocation by mixed spellings; pre-existing); 3
    - FMPOST7 EMPTYEMPH — a copy cut from `\emph{$^{\diamond}$}` leaves an empty `<emph/>` (`drop_emptied_wrappers`
      handles `text`/`ref` only; cosmetic); 1
    - FMPOST8 ORCIDINNAME — `Ann Able\orcidlink{…}`: the `<ref class="ltx_orcid">` sits inside `<personname>`; decide
      whether an ORCID link is name ink or a contact (the ORCID `href`/`title` also hold `\lx@orcidlink{…}`, P2 of revPURE); 3
    - FMPOST9 SEPONCE — two notes with bodies under one mark on one author both render the mark (dropping either loses a
      link); SEPMULT — footmisc `multiple`'s `\multfootsep` commas are not known to the engine; 3
    - FMPOST10 ENDNOTEMARK (silent loss) — 1703.08733: every author's `\endnotemark` is missing with no Warning and Efim
      Zelmanov gets the mark ",,1" (pdflatex "1,2"-style endnote marks per author); 3
    - FMPOST11 AFFILLEGEND (silent loss) — 2201.10988: `\affil[$\dag$]{These authors have contributed equally}` appears
      nowhere in the XML, 0 Warnings; 2606.17809's `\affil[ ]{\textsuperscript{\textdagger}…}` legend goes to the block,
      not the names; 3
    - FMPOST12 MARKPAIRS — between names: `\textsuperscript{\footnotemark[1]}\footnotemark[2]` gives ∗ to Ann (pdflatex
      prints it 4pt from Bob) and counts † lost; `Ann Able\footnotemark[1] and \qquad \footnotemark[2] \ Bob` counts †
      lost (revSHGNEXTb, pre-existing, counted); 3
    - FMPOST13 TABCELLAMP — the rehearsal does not read `&` cell ends for a `\thefootnote` declaration's scope (MARKSAFEb);
      1
    - FMPOST14 SPCONFADDR — 2602.18802: spconf's `\address` becomes a document-level `note role="address"`, so the names'
      `1/2/3` labels are mark-only notes nothing answers; 3
    - FMPOST15 CLUMPED — 2511.10879: all authors merged into one creator (names, e-mails, `<break/>`s) plus 1 jing error,
      pre-existing on every binary; 3
    - FMPOST16 CSSBASE — ar5iv-css: an author block without marked notes falls back to the centred layout and its name
      rows lose the baseline alignment (1807.01940 after MARKSAFEb); an IEEE conference 3-author block overflows at 375 px
      (pre-existing); 1
    - FMPOST17 REVTITLETHANKS — REVTeX title `\thanks` contacts are not rendered on the page (FMRG; XML has them); 1
    - FMPOST18 SHAUTO/T1 — unbound classes' title glyphs (geophysics 2006.10886 and 2403.07393, scrartcl 2406.03281,
      mnras-like hooks) and `\NewCommandCopy` / `\global\let\maketitle\relax` wrappers (`fmmarks_own_maketitle_relax_red`,
      `maketitle_newcommandcopy_wrapper_red`): measure from the class's own `\maketitle` replay (T1); after DG's sandbox
      review; 5
  - BINDFID — the long-term binding standard (DG 2026-11-20: "we *eventually* want very principled and complete
    bindings for each .cls and .sty that requires non-trivial semantic mapping … incrementally, as our sprints allow,
    while working on improving arXiv's fidelity and satisfying user reports"): a binding that stands in for a real file
    reproduces the file's observable effects — the files it loads or marks loaded (thmtools' four subpackages), where
    each command is legal (amsfonts' `\newsymbol` is a body-legal `\global\mathchardef`, not the preamble-only
    `\DeclareMathSymbol`), each macro's shape (a stored value vs a macro with parameters: EDP's `\@subjclass`), the
    definitions of the mode it runs in (amstex.sty under `\documentstyle` has no `\matrix@check`), and its own package
    loads with their options (natbib from elsarticle/acmart/jmlr/jheppub; no invented loads, as cvpr's was). A binding
    that reuses another binding's definitions calls them as Rust code and never loads a file the real one does not load
    (DG 2026-11-21: "reuse as Rust code, without the side effects of declaring auxiliary package and class bindings as
    loaded"; Perl's emulateapj.cls → emulateapj.sty and svjour3 → svjour shortcuts read 1305.6912's and 2110.05134's
    shipped copies; REGFIXH), while every file the real one loads itself is loaded as a real load, with its options and
    a shipped copy read (DG: "we still want to faithfully load LaTeX dependencies that the original packages and classes
    load themselves"). Witnesses:
    the v0.8.0 regression check (PR #958 and the REGFIX-D/E batches, ~430 papers). Steps: extend
    `tools/perfect_kernel/binding_conformance.sh` to compare a binding's loaded-file marks and preamble-only commands
    with the real file's; then audit the bindings in order of arXiv reach; 5
  - HEALTHYAB — a standing healthy-paper A/B before each bundle merge or fleet swap: a fixed, holdout-free sample
    stratified over all arXiv years (~10k, `kit/pop.py`), base vs candidate on the error-free rate, content kept,
    semantic markup and byte identity. The 2605/2606 sandbox reruns cover only 2026 idioms and a batch's own A/B only
    its construct; the v0.8.0 regressions sat in older idioms (1990s AMS-TeX, amsppt copies, sig-alternate, easytable)
    and only the full run caught them; 3
  - LOCKSIDEFX — one principled rule for what a refused definition defines on the side (an `advisor` design study):
    the definition lock (D1; OD #482, #500, #568) refuses a paper's redefinition of a kernel or binding command, and
    with it the helpers or counts that redefinition sets — mlbib/astron/ACM `\citeauthoryear` inside their own `\cite`
    (per-shape rule `label_macros_from_refused_cite`, OD #88; PR #958, REGFIX-F), amsppt copies' self-redefining
    `\title` (outer accepted, inner refused: an endless recursion; REGFIX-D), a replay's local change to an existing
    switch (REPLAYSWITCH). (bullsrsl's author count was not this: the kernel's store overwrote the class's own
    `\@author` helper, REGFIXG.) Replace the per-shape rules with one mechanism; 5
  - FUPC — after rc1 (DG 2026-11-20): `$$` inside a display (branch `fix/fupc-display-dollar-followups`, `39321db12b4`,
    reviewed NO-SHIP): it reopens the display its opener opened (a bare `\equation` and aliases too) and carries `\tag`,
    `\nonumber` and `\label` to the second display. On main (TAGDOLLAR, content): `\lx@retract@eqnno`/`retract_equation`
    (`sect07.rs` ~163-165) drop the first half's tag — `\begin{equation*} d=e \tag{S} $$ $$ =f \end{equation*}` loses
    (S) with 0 Warnings, `{equation} a=b \tag{T} $$ $$ =c` numbers (1), and the `\label` stays on the first half; `$$`
    inside the other `before_equation` displays still closes `{equation}` (REG929-ENVS, Perl too: `{multline*}`, whose
    `tex` reads `a$$$$b` where pdflatex prints "ab", aa/mn2e eqnarray-likes, IEEEeqnarray; general fix:
    `before_equation` records the display its caller opens, read by `\lx@begin@fake@intertext`), and a bare `{\equation
    x=1 $$ $$ y=2 \endequation}` gives 1 Error (pdflatex 0). DG ruling (b): an untagged amsmath label prints the
    equation's number where TeX's `\ref` is empty — record an OD divergence. The fix round: labels kept when a
    retraction is never reopened; the token-spelling ratchet (`latex_constructs/mod.rs` 224→225, `sect07.rs` 29→30);
    corrected KPE #1295 / OD #1091 wording. Residuals: `\[ a $$ $$ c \tag{3}\]` (pdflatex errs "\tag not allowed here"),
    `{displaymath}` with `$$ $$` (1 Error, pdflatex 0); 3
  - REGTAIL — the v0.8.0 regression check's residue after REGFIX M-U (int/regfix6; run 372 → 399, tier 2): of the 497
    papers clean on run 372, 192 still err (343 before that round, 0 newly erring, 12 Fatal), ~45 classes of 2-9:
    `unexpected:\endgroup` 9 (2110.03080, 1907.13475, 2502.02871), `expected:<dimen>` 7 (xy `\ar` labels, 1504.04186,
    1809.00682), `Fatal:oom:alloc_failed` 6, `unexpected:}`/`&` 6 each (ANONSTR), `expected:Match` 5, `\end{center}` 5,
    `\lx@personname` 5 (1207.5684, 1906.11649), `toccaption` 5 (solarphysics, svjour3; 1510.03282, 1511.05217),
    `TooManyErrors` 5, `\ordinal` 5 (1112.4346), `\@metaauthor` 5 (snapshotmfo.cls), `\email` 4 (cccconf.cls), old pgf
    1.x trees' `expected:conditional` 4 (0704.2264, cond-mat0610559), pgfmath overflow (2007.09944, 2201.10213),
    `\@end@tabular` 4 (0708.3763, 1107.1010), `\omit`/`\noalign` (2601.10115, 2609.04552, 2408.07095), `\relax` in
    `\csname` (2205.09553, 2208.14960); traced: THEAPABCA, NOLTAAUTHOR, IEEEDVI, CREFCAPS, EXPANDCUT, NEURIPSACK,
    TBLRADJ. From REGFIX G-L still: aastex631 2211.00793, 2502.03242's `\the@toks`. Unexplained on rc1 code (T2R):
    jcappub + translations `\spacefactor` (2008.10659, 2008.12721), 2507.12047 (`keyvals.rs:313`), thmtools
    prelude.sty:664 (2110.06326), `\@no@sparg`/`\@citea` (2209.13209), TooManyErrors 1507.00508, 1210.5701 (xy frames,
    Dimension too large), 1310.2513 (extraipa), 1701.06055 (`\RIfM@`), 2406.05605 (`\enquote` in a `.bbl`),
    `\ifCLASSINFOpdf` (2105.10205, 2502.09129; before Oct 8), circuits' wall clock (2506.09686), astro-ph0012068,
    `\rcite` recursion (cond-mat9505034), 2402.13372 (flaky?); Theorems.sty's `\newtheorem{\theoremcommand}` (2); 5
  - COPYROUTE — (post-0.8.0 first, DG 2026-11-21) REGFIXQ's copy tests read the shipped file's text before TeX reads
    it (content.rs `loads_file`, `reads_as_the_file`, `names_read_as`; OD #1118); replace them by rules on the executed
    path (advisor study): P the copy read where the binding reads its file (exists), W a request for X during that read
    reads X's real file, M a substituted read does not mark X loaded (precedent content.rs `_unmarks_itself`).
    `\ProvidesPackage`/`\ProvidesClass` record `\ver@` under the requested name (latex.ltx:18471-18486; ours and Perl's
    the declared one, sect05.rs:260-271, latex_constructs.pool.ltxml:841-850 — a KNOWN_PERL_ERRORS entry) and warn
    "You have requested … but the package provides …" as a Warning (DG). Guards to add: `\ProvidesClass{smfart}
    \LoadClassWithOptions{smfart}` (3 Errors now), `\IfFileExists{X.sty}{\RequirePackage{X}}{}`, an author file of four
    names or fewer sharing one (COPYSMALL: `{tikz-cd}` with `\def\ar`, `{tcolorbox}` renewing `\arraystretch`), a copy
    then a real `\usepackage{X}`; 5
  - SHIPPEDPKGSCAN — dependency scans still made before a copy is read: `shipped_class.rs` `require_shipped_packages`
    (~213, it chooses the binding's defaults) and `require_package`'s after a raw read (content.rs ~4215, a near no-op
    by the executed-set gate ~4828). Perl never scans a file it reads (Package.pm:2696, :2732); sn-jnl and optica follow
    TeX's order since REGFIXU (OD #1130); 3
  - NOLTAAUTHOR — nolta.cls's `\AUTHOR{…}` stores `\author{Name}{1}\orcid{…}` in `\@Mauthor`, run only by its
    `\@maketitle`; the lock refuses the class's `\author`, so there is no creator (the name a block note) and
    `Error:undefined:\current@author` (2404.06828, 2509.07487, 2604.20333, 2605.00366, 2605.05978; census reach ≈0): a
    thin binding, or a kernel rule that a raw class's stored `\author` calls printed by its title page are the creators;
    3
  - EXPANDCUT — a parameterless macro reading through its expansion in an author (`\def\contact{\xdef\@thefnmark{}
    \@footnotetext}`) is cut from its argument: 1909.05295 (svmult) a creator "and \@undefined", Stefan Bruckner lost,
    "footnote" in an e-mail; `\author{\textsc{Ann Bee}\contact{a@b.org}, Univ A\\ \textsc{Cy Dee}\contact{c@d.org}, Univ
    B}` merges Cy Dee into Ann Bee's affiliation; rims02e's `\affil` reroute swallows `\lx@author@markup@run`'s
    `\let\mail\@undefined` tail (math0408269). A whatsit made by a macro's expansion should cover the invoking token; 3
  - IEEEDVI — IEEEtran papers with `\includegraphics{x.eps}` throughout: the figure cue (converter.rs ~305
    `figure_output_cue`) picks DVI, `\ifCLASSINFOpdf` is false (ieeetran_cls.rs ~170) and graphicx never loads (3
    papers; 2112.07932 ships `-eps-converted-to.pdf`, so arXiv ran pdflatex): the cue should not pick DVI where the DVI
    branch cannot load graphicx; 3
  - THEAPABCA — theapa.sty read raw: after a `\cite`, `\BCAY` calls `\BCA`, defined only inside `\@@cite`, as our
    `\bibitem` digests the `\protect\BCAY{…}` label theapa's `\@biblabel` gobbles (:429) (1109.6618, 1111.0053,
    1405.3608, 2111.08066, 2304.01300; Perl 8 errors, it never reads theapa.sty): honour a raw `\@biblabel`, or bind
    theapa; 3
  - CREFCAPS — (ruling pending; (a), build the capitalised name lazily on its first use, recommended) a cleveref
    capitalised name with a leading command (`\MakeUppercase{\color{black} Section}`) is typeset at every `\label`
    (cleveref_sty.rs ~913-1000, CREF_TAG_ROLES): "Can't find color BLACK" though `\Cref` is unused (2402.05459,
    2404.01185, 2605.26927, 2606.11815; 2212.00537 errs in pdflatex too); 3
  - NEURIPSACK — `\usepackage[preprint]{neurips_2025}` with `\begin{ack}Thanks $$x_i=1$$\end{ack}` (or `\ack` as a
    command) gives `_` in math mode, the TL file too and Perl's same `{ack}` (2510.08734): model environ's
    `\NewEnviron{ack}`; 3
  - TBLRADJ — a `tblr` inside `adjustbox` inside `table*` (2502.02871), untraced; 3
  - STARTALIGNGRP — `\start@align` under a document's renewed `{alignat}` ended by its own `\math@cr\black@\totwidth@
    \egroup\egroup` (eqndefns-left: 2601.10843, 2602.04048): Fatal Recursion → 3 Errors, still an empty document; open
    the alignment in a group a plain `\egroup` closes (repro alignment/start_align_under_redressed_alignat). rc2 2026-11-23: no longer Fatal, but 2601.10843 keeps 1,328 of pdflatex's 15,212 words (63 Errors). Deferred, decided by difficulty (DG 2026-11-23); 3
  - GATHERCAP — `\lx@ams@gather@close` scans 8 tokens, not items: `…\black@\totwidth@ \noalign{\vskip 4pt}\egroup $$`
    overflows into the raw cascade (6 Errors), `\iftag@\fi` before the `\egroup` gives a false Warning (synthetic; every
    TL and corpus form is `\math@cr \black@\totwidth@ \egroup`): scan balanced items, skip `\black@<arg>`, empty
    conditionals and `\relax` silently; 1
  - XARGSSPACE — an xargs later optional argument peeks past spaces and drops them where no `[` follows, which
    `\xargs@ifnextchar` (xargs.sty:64-87) puts back: `\ry{i} \ry{i}[j]` prints "SieSij", pdflatex "Sie Sij"; Perl the
    same (KNOWN_PERL_ERRORS #1304); 1
  - PDFSTRGAPS — hyperref PDF strings against pdfinfo: `\cite[p.~3]{k}` → `[cite][k, p. 3]`, `\footnote[3]{n}` →
    `3]n`, `\hspace{1em}` → `1em`, `\textcolor{red}` keeps `red`, `\fontsize{9}{10}` → `910`, `\raisebox{1pt}` keeps
    `1pt`; no "Token not allowed in a PDF string" Warnings (OD #824); a plain `\edef` of `\xrightarrow` still Fatal
    (pdflatex 3 `!`; KNOWN_PERL_ERRORS #1302); a macro used in hyperref's package options before its `\newcommand`
    stays an ERROR stub; 3
  - FMNOTESHAPES — frontmatter note shapes (synthetic unless named; REGFIXM2, P, S reviews): a list note with several
    marks (`\textsuperscript{1}Univ X\\ \textsuperscript{2}Univ Y` appended to `\@setauthors`; elsarticle's unlabelled
    `\elsaddress`, acmart's `\@authornotes`) loses its marks (`\lx@fm@list@note` is mark-led at one mark only); a
    stranded "by" from a document's `\@setauthors` printing `by \authors`; a `\footnote` in `\@setauthors` nests in a
    note; a mark-led note goes to every creator showing the mark (MARKLEDNOTE; 2610.02789: three copies; OD #1054;
    options: the first creator only, or drop an `\address` a document's emptied `\@setaddresses` never prints); a
    printed `$/\!\!/$` between names dropped with 0 Warnings (FMSEPDROP; 2401.17245; repro
    `sectioning-frontmatter/emitter_parameterless_macro_before_a_skip_keeps_the_name.tex`); `\let\mail\footnote` in
    title code still held undefined (LETFOOTNOTE); an EPL `\inst{2}` in a `\thanks` an italic digit where pdflatex's is
    upright (EPLINSTFONT; `\lx@title@inst` = `\unskip${}^{#1}$`); a raw amsart-based class with its own layout
    `\@setauthors` (ijmart.cls) would print leftover words ("by") as a note (no witness); amsart's `\@setthanks` is
    defined since #964 (guard `perfect_kernel_batchregfixx::amsart_document_setauthors_calls_setthanks`); 3
  - CAPTIONV1 — caption3 v1 residues (REGFIXR2 review): a consumed v1 option list applies `labelformat` only
    (`[subfloat]{labelformat={simple},labelsep=colon}` prints "a", pdflatex "a:"); a caption-less `\subfloat` under
    `[subfigure]` warns "Unused" (pdflatex not), and raw v1.8h errs on `\phantomcaption`; `\captionsetup[figure]`
    under raw floatrow warns "Unused"; a document's `\usepackage{caption3}` reaches the binding while a raw package's
    `\RequirePackage{caption3}` reads the shipped copy; 3
  - KERNELSMALL — kernel residuals found by this round's reviews: a `\let` copy of `\underline` prints as the primitive
    under `\meaning` (ULMEANING; pdflatex `macro:->\protect \underline  `; tex_debugging.rs ~286-290 keys on the token's
    name, so `\patchcmd\oldul` or `\ifdefmacro\oldul` misread a saved copy; fix: give `\@@underline`, latex_base.rs
    ~536, its own primitive stand-in); `\ignorespacesafterend` leaves the space after `\end{…}` (IGNSPACESEND:
    "AFTERTEXT  next."); a named-equation hack (`\eqname` through a patched `\maketag@@@`) prints nothing (EQNAMEHACK;
    pdflatex "NAMEA"); `\[…\]\noindent X` marks the whole paragraph `ltx_noindent`; a `\let` (or `\LetLtxMacro`) copy of
    an optional-argument command keeps the old body after the original's redefinition, where LaTeX's calls `\\cs` by
    name (TWOOPTLET: `\LetLtxMacro\twc\tw` then `\renewcommandtwoopt\tw`, pdflatex "Rpqr", ours "Tpqr"); the
    `if_next('[')` peeks other than `\@testopt`'s (`pdftex.rs:399`, `base_parameter_types.rs:2027` optional_key_vals) do
    not skip spaces (Perl-shaped, no witness); `\@testopt`, like `\@ifnextchar`'s closure, may not halve `##` in the
    chosen branch (suspected); 3
  - INLSCAN — OD #1079's rule (an inline constructor that sets authored text starts the paragraph, #909) is applied by a
    scan that cannot see imperative (`sub`-bodied) constructors or ones opening with a non-inline element: a follow-up
    scan for full coverage; algorithmic/algorithmicx's `\hfill` stand-ins do not enter horizontal mode as TeX's `\hfill`
    does (ALGHFILL; no loss seen); 3
  - PHANTOMPAR — (Perl-shared) `\phantom{Y} starts here.` / `\vphantom{Z} starts here.` glue the next word where TeX
    starts the paragraph (latex.ltx:15617 `\leavevmode@ifvmode`), and `enter_horizontal` alone would drop a paragraph a
    phantom alone holds (`isSpace` whatsits in `repack_paragraph`, Perl `repackHorizontal`): (a) `\phantom`/`\vphantom`
    dispatch like `\hphantom`, a restricted-horizontal text branch with enter_horizontal (the code of the 2004.10048,
    2605.21158 witnesses), or (b) the repack keeps a paragraph whose only content is a phantom box; RED
    `paragraph-mode/phantom_starts_the_paragraph`; 3
  - RESTATEKEY — thm-restate residuals: the `restate=` key's `\x*` prints the first number where pdflatex reads the
    last from the `.aux`; `restate=` on a wrapper environment unprobed; 2609.31596's `\renewcommand{\thecorollary}
    {\ref*{#1}.#2}` ignored (theorem numbering, "Corollary 9" for pdflatex's "8.1"); 1
  - INPUTNAMEBYTE — `\input{x<0x81>y}` under latin1 is a missing file whose name holds `\protect\GenericError…`
    (pdflatex: a garbled name, then Fatal); revtex4-1 without hyperref defines `\ifpdf`/`\ifxetex` (undefined in TeX);
    emulateapj answers `\@ifpackageloaded{iftex}`/`{hyperref}` yes (pdflatex no); an undefined byte inside `{verbatim}`
    drops its Error (in one shape the line too); inputenc's Errors come twice in `\caption`, natbib's `\bibitem[…]`,
    `\texorpdfstring` and `\nameref`, where pdflatex's first run gives one; 1
  - TIKZCDARROWS — a file declaring `{tikz-cd}` that loads `\usetikzlibrary{arrows}` gives 14 Errors at
    pgflibraryarrows.code.tex:36 ("I did not find the pgf library 's'"), base too, 0 under its own name; pdflatex
    clean; 1
  - SAFEACTIVES — the URL readers (`\url`'s, `HyperVerbatim`) do not set babel's safe actives, as `\hyper@normalise`
    does (`\Hy@safe@activestrue`, hyperref.sty:4627, defined :362): ngerman `A\footnote{\href{http://x.org/"o}{s}}`
    gives href `http://x.org/ö`, pdflatex `http://x.org/"o`; main the same (int/regfix7 review); 1
  - C1UTF8 — after `[utf8]{inputenc}` a C1 control code point (U+0080-U+009F) read fresh is catcode other
    (`utf8_def.rs:243`) and prints silently, where pdflatex errs "Unicode character (U+0085) not set up for use with
    LaTeX"; main the same; 1
  - INPENCLOOP — `set_input_encoding`'s loop, like Perl's (inputenc.sty.ltxml:48-51), skips 0x1F and 0x7F, which
    inputenc.sty:83-84 covers (`\^^N\^^_`, `\^^?\^^ff`); 1
  - AMSDEPOSIT — amsart's `\@settitle`, `\@setdate`, `\@setsubjclass`, `\@setkeywords`, `\@adminfootnotes`,
    `\@setcopyright`, `\@maketitle@hook`, `\@cleartopmattertags`, `\thanks@warning`, `\@setcontribs` are undefined in
    `ams_core_cls.rs` (Perl's neither), so a document's own `\@maketitle`/`\maketitle` deposit that calls them is
    dropped with an Info ("raised 4 error(s)"); in REGFIXX's 200-paper census ~128 redefinitions call `\@settitle`, ~70
    `\@setdate`, 84 `\@maketitle@hook`. Defining them faithfully changes output (an empty `\@date` gives "Date: ."):
    (a) define + A/B the deposit population, (b) define but keep the deposit dropped while the frontmatter owns the
    fields, (c) leave — DG ruling; 3
  - SPLITFREE — follow-ups of SPLITLEAK (fixed for 0.8.0-rc2, DG 2026-11-21: `Split::process_pages` frees each page's
    source subtree once copied, `PostDocument::discard_subtree`; a page holding an `ltx:navigation` stays, as the nav may
    use its namespaces; guards `112_fragment_yield::split_pages_leave_no_c_heap_residue`,
    `::split_keeps_pages_whose_namespaces_a_navigation_uses`): (a) the other `remove_nodes` callers that never reuse
    the node — `PostDocument::unwrap_node`, CrossRef `settle_cref_conjunctions`, the external-ref children
    (`crossref.rs` ~1923), `make_bibliography`'s bibentries and empty biblists (check moved vs copied first), svg's
    `drawn`; (b) `malloc_trim(0)` after Split as its own lever (C-free 954 MB vs 8 MB streaming at n1600, C-live 29 MB
    both); (c) with `--input xml`, Split's copies drop a nav's prefixed namespace and a page redeclaring the default
    namespace comes out `<default:section xmlns:default=…>` (base too, cosmetic, foreign XML only); 3
  - SERSTREAMUNSPLIT — an unsplit run and `--nopost` output still build the whole core XML as one String
    (`Document::serialize_to_string`, `converter.rs` `serialize_result`); the same sink (`serialize_to_file`) could write
    them to `--dest` or to post's file input. Witness: any unsplit document past ~1 GiB of core XML (issue 777's whole
    book unsplit); 2
  - RENDERFLEETBIG (post-0.8.0) — a render worker decodes the whole render database at attach: its peak is ~42 MB +
    4.85 × the db (measured on issue 777's k600/k2400 slices; the estimate is `64 MiB + 6 × db`, `render_workers.rs`
    `worker_estimate_bytes`). On issue 777's whole book (115,519 pages, db ≈ 600 MB) one worker needs ~3.7 GB, so under
    `--max-memory=8192` with the parent at ~4.3 GB the fleet affords 0 workers and the pages render serially. Lever:
    attach the db lazily (point lookups per page), so a worker's footprint follows its page range; an `ObjectDB`
    redesign — measure k2400 and the whole book before and after; 2
  - SPILLKEEP (post-0.8.0) — render workers keep their spilled pages (`remove_spilled: false`) until post ends, since
    the serial fallback re-renders the whole document: ~+2.7 GB of disk on issue 777's whole book. Freeing a finished
    worker's pages needs a fallback that re-renders only the failed ranges (`render_workers.rs` `parallel_render`'s
    fold); 1
  - RENDERPLATFORMS — DONE for 0.8.0-rc2 (2026-11-23, `release-platform-check.yml` run 45119655890): an 80-section
    split document (`tools/render_platform_check.sh`) was converted on the published macOS arm64, macOS Intel and
    Windows binaries three ways: with the automatic fleet (3/4/4 workers over 82 pages), serially, and with the fleet
    again over its own output (Windows' `writer::Staged` renames onto existing files). All runs exited 0 with no
    Error, the sites were byte-identical (84 files) and no spill or handoff file was left. Re-run the workflow on
    every later RC. The fleet stays the default on every platform. Residual, unchanged: macOS has no RSS reading, so
    a worker's watchdog memory share is not enforced there; 0
  - PANICSWEEP-MAXPERF (verify) — the abort-panic hook that removes the run's registered paths (`install_panic_sweep`,
    `bin/latexml_oxide.rs`) is guarded only through its wiring (test builds unwind; the test runs the hook in a child
    process): one maxperf build with a test-utils post panic, check no `latexml-post-handoff-*` stays; 1
  - RENDERPID (post-0.8.0) — `render_workers.rs` (the first-death kill) and `try_kill_registered_children` kill by raw
    pid: between the drain thread's `wait()` and `is_finished()` the pid is free and could be reused (a microsecond
    window, now on every abandoned fleet); signal through a pidfd opened at spawn. Also a child whose drain thread fails
    to spawn is killed and dropped unwaited, a zombie until the parent exits; 1
  - GFXTEMPS (post-0.8.0) — a SIGKILLed render worker leaves its pid-tagged `.…tmp.<pid>.…` staging files in a
    directory output (the cache's `STALE_TMP_AGE` sweep reclaims the cache's; zips skip dotfiles): the parent could
    sweep a dead worker's temps after abandoning the fleet. Also ImageMagick's PNG `date:*` chunks make raster bytes
    differ run to run; 1
  - SPILLSEG (post-0.8.0) — output can depend on where the streaming core spills: issue 777's 2,400-problem slice at
    `--max-memory=3700` (RSS ≈ the 925 MB watermark) wraps `\date` in an extra `<span id="id2" class="ltx_text">` when
    pass 1 stages 7 segments instead of 5; pinning `LATEXML_SPILL_AT_MIB` makes two builds agree. Find the yield-point
    dependence (an id counter or a frame crossing a segment boundary); 2
  - WORKERERRBASE (post-0.8.0) — a render worker counts errors from 0, while the serial render's cap also counts the core
    phase's errors, so a document near its cap can latch `TooManyErrors` serially but not in parallel: pass the
    parent's error count as a baseline in the `RenderManifest`; 1
  - LOGONBREACH (post-0.8.0) — on a wall-clock or memory breach the CLI writes no `--log` file (0.7.6 too): the
    watchdog thread cannot flush the converting thread's `LOG_BUFFER`. Options: open `--log` up front and mirror into
    it (`set_log_mirror`, as cortex harness children do; costs a write per log append, measure), or a one-line stub
    log from the hook; 2
  - CHROMEGATHER (post-0.8.0) — Chrome sizes a `display="block"` `<math>` to its ink alone, so `gather*` rows sit 11-13
    px apart (no line strut), inside or outside a cell; Firefox gives 19-21 px. Options: a minimum line height on
    equation cells holding block math, or `display="inline"` for per-row math in equation tables; 1
  - EQNSPANRESID (post-0.8.0, #972 residual, OXIDIZED_DESIGN_DIVERGENCES #1135) — in the span form (a display in a
    `p{}` cell, `\parbox`, minipage or note) `colspan`/`rowspan` do not span: an `\intertext` splits the column grid,
    and a `leqno` tag cell (`rowspan`) covers only row 1, shifting rows 2+ one column left (minimal: `align` with
    `leqno` in a minipage); a `::before` table-cell on the following rows is a cheap fix; 1
  - EQNUMTD (post-0.8.0; Perl too, KNOWN_PERL_ERRORS candidate) — `LaTeXML-block-xhtml.xsl:551/579` `eqnumtd` tests
    `ancestor-or-self::ltx:tr`, which also matches an enclosing tabular's row, so an `equation`+`aligned` in a tabular
    cell prints its tag on every row. Minimal: `\begin{tabular}{|p{3in}|}{\begin{equation}\begin{aligned} x &= 1\\
    y &= 2\end{aligned}\end{equation}}\\\end{tabular}`; test only for a row inside the equation's own MathFork; 1
  - ACMTEASER (post-0.8.0, fidelity, needs a ruling) — acmart's `teaserfigure`: pdflatex prints title, authors, teaser,
    abstract, keywords; ours (`acmart_cls.rs` ~455-476, Perl's Replace semantics) puts the teaser after the keywords,
    Perl LaTeXML before the title. A binding change with an OD entry and an A/B over the acmart papers; 1
  - TEMPSWEEP (after the v0.8.0 tag, DG 2026-11-22) — an abandoned render fleet leaves pid-tagged staging temps in the
    output: the stop-at-first-death loop (`render_workers.rs:682`, `kill_pid(pid, false)`) SIGKILLs the other workers mid-
    graphics, so they never remove their registered paths and their mutool/pdftocairo/gs children survive. Probe: a
    73-page `--split` with 9,000 math copies in section 1 and 10 tikz PDFs per later section, `--max-memory=1400`,
    graphics cache off: 6 hidden `..f_N_0.tmp.<pid>.5.svg.<pid>-3.mutool_svg/` dirs; the base before #973 left 0; pages
    byte-identical. Fix: workers as process-group leaders (`CommandExt::process_group(0)`) killed as a group, a parent
    sweep of the killed pids' staging names on abandon (output and graphics cache; matcher built from
    `unique_staging_tag`), and `install_panic_sweep` before the worker-mode return (`bin/latexml_oxide.rs:799/803`,
    maxperf `panic=abort`). Supersedes GFXTEMPS' "lever" line; 2
  - ALLOCOPTS (after 0.8.0, portable, DG 2026-11-22) — the fleet's children get `MALLOC_ARENA_MAX=2` and
    `MIMALLOC_ARENA_RESERVE=128MiB` by environment (PR fix/worker-malloc-arenas). The `latexml_oxide` CLI sets no
    address-space limit and does not need them (2605.22454: VmPeak 4.7-5.8 -> 2.1-2.4 GB, resident 472-479 MB either way,
    HTML byte-identical); under a user's `ulimit -v` they help the same way. Setting them from inside the binaries
    (glibc `mallopt(M_ARENA_MAX)`, mimalloc `mi_option_set`) needs a design for macOS and Windows too; a small `unsafe`
    at that boundary is acceptable if truly needed; 1
  - WORKERAS-RECAL (after 0.8.0) — `cortex_worker.rs` `FRESH_ADDRESS_SPACE_KB`, `ADDRESS_SPACE_HEADROOM_KB`,
    `RECYCLE_ADDRESS_SPACE_KB` (:1046-1062) were measured on mimalloc 3.3.2; re-measure a fresh child's address space and
    the recycle point under 2.3.2 with the new allocator settings; 1
  - SCANFLAGS-SUBTREE (after 0.8.0, nit) — in release builds the per-page index/bibliography flags come from Scan's own
    walk (the XPath probe is a `debug_assert`); Scan does not descend `ltx:Math`, `ltx:rawhtml`, `ltx:navigation`,
    `ltx:bibentry`, so an `ltx:bibliography`/empty `ltx:index` under one would skip MakeBibliography/MakeIndex on that
    page. No witness; document the limit by `last_page_placeholders` or fall back to the XPath; 1
  - LOCALREPLACE-ANCHOR (after 0.8.0, doc nit) — `rewrite.rs:806-813`: under the opt-in `local_replace`, a closure that
    moves the anchor sibling leaves the inserted nodes at the parent's end. None of the four opted-in rules touches
    siblings; one line in the `local_replace` doc; 1
  - ENVCONSIST (after 0.8.0) — the environment-variable inventory (docs/ENVIRONMENT.md) found inconsistent parsing: six
    flag conventions (presence: `LATEXML_NODUMP=0` still skips the dumps; exactly `1`; `1|true|yes`; `0` disables;
    clap bool; `LATEXML_POST_STREAM_SPLIT` any-non-0 forces on); `SOURCE_DATE_EPOCH` parsed as i64 (`tex_job.rs:17`,
    `identity.rs:64`) and u64 (`bin/latexml_oxide.rs:1829,2148`, `epub.rs:583`); `NO_COLOR` empty counts as set
    (`logger.rs:32`); a non-numeric `LATEXML_TOKEN_LIMIT` falls back yet still disables source scaling (`gullet.rs:794`);
    uncached hot-path reads (`LATEXML_MARPA_ASF_AUDIT` 6× per ambiguous formula, `parser.rs:2534-2863`;
    `LXML_TRACE_NODE_BOXES` per child, `document.rs:5509`); no-op `rerun-if-env-changed` for run-time dump variables
    (`latexml_engine/build.rs:326-328`); two git-sha inputs (`LATEXML_GIT_SHA` vs `LATEXML_GIT_SHA_OVERRIDE`); cache-home
    resolution differs (`graphics_cache.rs:157` has no `USERPROFILE` fallback); `LATEXML_RENDER_JOBS` ≥ 2 outside the
    `latexml_oxide` binary re-execs `current_exe()` as a render worker that only that binary serves (`render_workers.rs:493,611`;
    inferred, not run); `KPSEWHICH` not honoured by our own kpsewhich probes (`pathname.rs:377,412`). Stale docs:
    PERFORMANCE.md says `LATEXML_TELEMETRY_OUT` truncates (it appends), `graphics.rs:1422` says 100 dpi (120). One
    convention per kind, documented in ENVIRONMENT.md; 1
  - FMTCACHE-PRUNE (post-0.8.0) — the per-user format cache (`~/.cache/latexml-oxide/formats/<version>-<executable>-
    <tree>/`, ~4 MB an entry, OXIDIZED_DESIGN_DIVERGENCES #449) is never pruned: a developer host holds 1,843 entries
    (7 GB; earlier 6.6 GB in 1,734; `latexml_oxide/src/format_dumps.rs` ~:281-315; the fleet's dumps match its tree, so it does not use them). Drop entries whose executable hash no longer exists, or keep the newest few per tree, or document a cleanup (FMTCACHE folded in 2026-11-23); 1
  - DUMPTRIM (post-0.8.0, DG 2026-11-23) — embed one TeX Live year's formats instead of the 5-year window (2022–2026,
    `parity/DUMP_DESIGN.md` "Distribution"). An embedded format is used only with no TeX tree, or on a tree whose
    `latex.ltx` + `expl3-code.tex` (or `plain.tex`) match its `# source` stamps (DUMP_DESIGN "Tree match",
    OXIDIZED_DESIGN_DIVERGENCES #449); every other tree builds its own once. Measured with the 0.8.0-rc2 Linux binary:
    neither an official-installer TL 2025 last updated 2025-11 nor Ubuntu's `texlive-base 2025.20260124` matched the
    embedded TL2025 (both differ in both files) — first conversion 8.5 s including the build, 0.13 s cached. A year
    costs ~0.55 MB gzipped (latex 534 KB + plain 16 KB) and one pinned-TL container per release. Keep the newest
    frozen year (the no-TeX case); check whether the container image's tree matches an embedded year, else have the
    image build its format at image build time; before dropping all of them, measure the `LATEXML_NODUMP` branch
    (quality, start-up) as the no-TeX fallback. Touches CLAUDE.md's self-contained-binary rule and `INSTALL.md`; 2
  - MIMALLOC3 — back to mimalloc 3.x once `libmimalloc-sys` bundles a release with the 3.3.x heap-corruption fixes
    (≥ 3.5.3; microsoft/mimalloc issues 1287, 1430): 0.8.0-rc2 builds the bundled 2.3.2 (`latexml_oxide/Cargo.toml`
    `v2` feature). Witness: 1502.06095 crashed twice under full-width A/B load (mimalloc's small free list holding a
    Token-shaped value; an indirect call through a corrupted pointer), ~0.3 % of loaded runs, 0 of ~370 quiet runs and 0
    ASan reports on the system allocator; the fleet's only unexplained SIGSEGV in run 397 is 2212.05990 (never
    reproduced). A render worker's later SIGSEGV on 2.3.2 (issue 777's 2,400-problem slice under load; ip in
    the glibc heap) is a different class, fixed by the watchdog ending through `_exit` (#973); the 1502.06095 crash in
    mimalloc's own heap during digestion stays unexplained. Re-measure 3.x against 2.3.2 (instructions, peak RSS, a
    loaded 1502.06095 loop) before switching; 2
  - SPLITDOMNS — the whole-DOM split (`latexml_post/src/split.rs`) loses a page's non-`ltx` namespaces: xy-pic's svg
    inside math raises `malformed:namespace` Warnings (1502.06095: 344, 1609.07593: 1,377 under `--splitat=section`)
    and 2606.28974 loses an `aria-hidden`; the streamed split, the default for split runs since OXIDIZED_DESIGN_DIVERGENCES
    #1133, gives the unsplit pages. Reached now only with `LATEXML_POST_STREAM_SPLIT=0` and the streamed split's
    fallbacks (a late navigation, resource, root date or `<?latexml?>` PI; a nested `ltx:resource`); 2
  - PARTUP — a split `\part` book's chapter pages carry no `up` link to their part page, which Perl's carries
    (pre-existing, seen while pruning the head links, OXIDIZED_DESIGN_DIVERGENCES #1134); 2
  - POSTRESID — after the core, ~190 MB of C heap stays live into post (pass B 12.69 s vs 11.26 s on a clean heap at
    n1600): core nodes unlinked and never freed, ~98 KB a subsection (heaptrack, n100: `insert_math_token` ~52 %, the
    amsmath rearrangement `ams_rearrangeable_bindings`/`equationgroup_join_cols` ~15 %, `tex_math`, `sect07`,
    `mark_xmnode_visibility`; first suspect `latexml_math_parser/src/parser.rs` ~2190); constant: kpathsea's file
    database 12.9 MB, the XSLT cache 5 MB. CORELEAK (0.8.0-rc2) frees the MathFork's `_Capture_` cells and the folded
    XMHints (core orphans 1,402 roots → 2 on the n100 book); left: a constant `rewrite.rs` Replace residue (2 roots),
    and eqnarray leaves 4 stale `node_boxes` entries per environment, which the streaming backstop sweeps
    (`perfect_kernel_gemini::eqnarray_stale_node_boxes_are_swept`); 2
  - RUNAWAY395 — an undelimited macro argument that meets a closing `}` takes it (Perl `Gullet.pm:732` readArg;
    `gullet.rs` ~4048 `read_arg`), so an `\edef` runs to the end of the file (`\makeatletter\edef\x{\@firstoffive{a}}`):
    1012.2852 loses its last ~370 lines, base and Perl too; TeX inserts `\par` and keeps the `}` (tex.web §395).
    Fix through the `Runaway::Paragraph` path, as its own PR with an A/B. (`\end{document}` is protected since REGFIXJ3,
    so the runaway no longer loops on its tail; OD #1107); 3
  - ANONSTR — the "at Anonymous String" `&`/`}` error classes are separate mechanisms (REGFIXK sample): 2512.14155 (a
    stray `&`), 2107.00457 (an extra alignment tab), 2003.01824 (`Extra }` with arydshln), 2207.06210 (unclassified):
    one root-causer each; 3
  - FMREPLAY — the class-setter replay's residuals (REGFIXG, REGFIXI): a replay drops its local change to an existing
    switch (REPLAYSWITCH; bullsrsl `\setboolean{BSRSL@affilsempty}`: "Affiliation is empty", 19 papers, 2309.01084,
    2411.00793; clashes with `perfect_kernel_batch56::class_maketitle_replay_that_errors_is_dropped`); a replay's
    records held in toks registers survive the deposit (REPLAYTOKS: a class helper appending authors to a toks its
    `\maketitle` prints prints each twice; synthetic,
    `\def\@author[#1]#2{\global\auth@toks\expandafter{\the\auth@toks\par AUTH: #2}}` + `\the\auth@toks` in `\maketitle`;
    fix in `note_replay_records`, sect05.rs ~3345); bullsrsl surnames read as affiliations (BSRSLSURNAME;
    `\lx@author@trailing@mand`, author_macros.rs ~148, since Oct 8) and its "Corresponding author(s):" label not kept
    (BSRSLCORRESP; now counted: 38 papers' unclaimed Warning; option: carry `\BSRSL@CorrespIntro` into the frontmatter);
    a delimited-argument kept call makes the block decline (RED
    `sectioning-frontmatter/author_delimited_argument_kept_call_declines_red`); a note-printing call's globals read
    outside the block are still lost (no witness); with authblk a `\footnotemark` inside a robust command drops its mark
    (counted); contacts on a `\\` line land in a nameless creator; a counted `\orcidlink` wrapper gives Count 1
    (pdflatex 2); JATS/TEI `[@class='ltx_phantom']` misses `ltx_align_left ltx_phantom`; 5
  - ENDDOCRES — `\end{document}`'s residuals (REGFIXJ): an environment open at the end is Perl's Warning where TeX's
    `\@badend` errs (ENDBADEND; the standing rule says Error: count the corpus first); `enddocument/info` and
    `\AtEndAfterFileList` never run, nor a package's (non-top-level) late-hook code (ENDINFOHOOK); raw ltxgrid's
    output-routine messages (`\do@output@MVL`/`\do@output@cclv`) never fire (LTXGRIDMSG, LTXGRIDOUTPUT; 2401.03827:
    empty `ltx:p`s at the end, the document's own afterlastpage/afteraux code dropped; option: a no-output stand-in in a
    binding); a hook that detokenizes the grabbed tail prints `“lx@enddocument@tail` where pdflatex prints
    `“@kernel@after@enddocument` (ENDTAILNAME; name the slot after latex.ltx:15258's); a hook that drops what it grabbed
    before `\@checkend` drops `\lx@enddocument@tail` with it, so its `\clearpage` adds a `<pagination>` (ENDTAILDROP;
    synthetic, repro `expansion-primitives/enddocument_hook_scan_drops_the_tail.tex`, OD #1107); `\deadcycles\z@` before
    `\@@end` would let ~40 papers' `\patchcmd\enddocument{\deadcycles}` (quantumarticle.cls, cedram.cls) succeed
    (DEADCYCLES; its own A/B); 2502.03242, 2510.09575, 2603.11285 keep `\vcenter` end-mode Errors where the finalizer
    closes abandoned raw-array groups (was a Fatal); 3
  - BINDRES — the binding fixes' residuals (REGFIXH, REGFIXI, REGFIXK, REGFIXL): no harvard binding (HARVARDCITE; Perl
    none; 17 IUCr papers; 2504.07452's `\citeasnoun` prints a bold "?", "[]" before every bibblock, keyless bibitems;
    2505.20965; option: a thin binding routing `\citeasnoun`/`\citeyear` to the kernel cite); `\item[label]` in
    `{thebibliography}` prints the label twice (BIBITEMLABEL2; `[[1]]`, since Oct 8); emulateapj's `\textwidth` is 345pt
    where emulateapj.cls:334-335 sets 7.1in and `\columnsep` 0.3125in (EMULATEAPJWIDTH; 1612.09425's f1: pdflatex
    ≈231pt, ours 155pt; 0812.1578; its own A/B, it moves every emulateapj paper's widths); `[epj]{svjour2}` papers never
    get the EPJ panel page (SVJOUREPJPANEL; hep-ph0608235; svjour_cls.rs:82 `is_document_class("svjour")`); BINDFID's
    audit list: aas_support (aas_macros, deluxetable), svproc (aas_macros), paspconf (aaspp), emulateapj's base class
    aastex (the real one is REVTeX), sv_support's unconditional natbib (svjour3.cls:68 only under its option);
    `\meaning\text` shows `\lx@text` where pdflatex shows `\protect \text  ` (OD #1104; robust needs the lock to hold
    the inner `\text␣`, OD #568); `\bibitem[\lab]` with an argument-taking `\lab` (BIBLABELARG; pdflatex 2 errors, we 0
    and cite `[L]`; mnras's `\citeauthoryear` label: pdflatex 2 errors, one per `\write`, we 1; `$\frac\text{a}{b}$`:
    pdflatex 37 errors, we 0 with `tex="\frac{\text{}}{a}{b}"`); ulem's `\ul` runs through an `\end{…}`/`\endgroup` at
    its depth and past a closing `$`, and a cell-end-closed line loses the cell's right border (pdflatex fatal; RED
    repros under `loader/ulem_declaration_*`), and a document's own underline macro through `\useunder` in a
    `\multicolumn` still errs; breakurl's `\burl` URLs print as sans text without a link, as pdflatex's `.bbl` fallback
    since REGFIXU (2403.06633; a binding `\burl` = `\url` would be a surpass); 3
  - REGRES — the regression fixes' residuals: fdsymbol's ~165 `symNNN` glyphs, 6 extensible accents (2312.00399
    `\overlinesegment`) and `\ullcorner`/`\ulrcorner`; `\usepackage{amsmath}` after `\documentstyle{amsart}` does not
    restore the old-form error; natbib loaded before `\documentclass{elsarticle}` is only an Info (RED
    `natbib_before_elsarticle`); natbib residuals 2111.08230 (a standalone sub-document's preamble) and 2405.11676
    (imsart-altered passes); a binding-defined name silently ignores a document `\newcommand` (no "already defined"
    Error); `\usepackage{amstex}` under LaTeX2e raises no "obsolete" error; fdsymbol + amssymb gives no "already
    defined" errors; papers whose `aaai.sty` is missing (cmp-lg9406010, cmp-lg9704001, cs9811010, cs0003021) keep
    `undefined:\citeauthoryear`, the honest missing-file result (option: a small aaai.sty binding); an unmodified
    renamed natbib.sty after natbib gives pdflatex 80 errors, ours 0 (lenient); natbib's fullauthors tag is typeset
    (2408.03605, Perl too); 3
  - LASTBOXROW — `\lastbox` takes a whole alignment, not its last row (tex.web §1080; rows §799, §810): easytable's
    per-cell peel collapses into one cell holding the whole table, so its rules don't line up; 3
  - FUPA — after rc1 (DG 2026-11-20): the loader follow-ups of #945 and #947. A user `.rhai` for a name reached on the
    provides route (PROVROUTE: the paper ships `mylipics.cls` with `\ProvidesClass{lipics-v2021}`) still has the
    renamed copy read after it (`loading_for_a_provides_route`, `content.rs`, never asks `state::local_binding_file`);
    `\IfFileExists{mypkg.sty}` with only a user `mypkg.sty.rhai` answers false (`binding_counts_as_file`), where Perl's
    `FindFile` finds any `.ltxml` in the search paths; `DeferredNote` state unit tests (an empty load, log then decline,
    `catch_unwind` inside a printed and an unprinted note, silenced notes); a dispatcher that logs and then declines
    leaves a phantom `(Loading X... )` (Perl's `FindFile` runs before `ProgressSpinup`); a `.tex` binding's note names
    the raw file (`(Loading epsf.tex... )`); 3
  - AMSPPTRES — REGFIX-D's residuals (PR #960): math0610862 is back at its Oct-8 `Fatal:TooManyErrors` — the amsppt
    binding reads each `\ref` field expanding (`\@bibfield … XUntil`, `amsppt_sty.rs`; Perl amsppt.sty.ltxml:439,
    KNOWN_PERL_ERRORS #1298) where amsppt.sty boxes it (:956-965), so the paper's `\noOf` assignment misfires: read the
    field by digesting it (RED `amstex/amsppt_bibfield_expands_a_conditional_red`); a copy's additions over amsppt
    are lost (math0610862 `\Refst`, `\subrefhead`; 1003.5377 `\SecondPage`; RED
    `amstex/amstex_copy_additions_are_kept_red`); amsppt's self-redefining `\title` stays refused, 1 Error with every
    word kept where TeX prints "[T:Hello]" (RED `amstex/amstex_self_redefining_title_runs_red`; LOCKSIDEFX); 5
  - THMEDPSRES — REGFIX-E's residuals (PR #960): thm-listof's filters (`\showtheorems`, `\ignoretheorems`,
    `\onlynamedtheorems`, `\setlisttheoremstyle`) are undefined under the thmtools binding (2 of ~519 census papers;
    `\usepackage{thmtools}\ignoretheorems{x}`); edpsmath's `\keywords` stores into box `\@b@kwrds` and our
    `\maketitle` never runs `\@setkeywords`, so 1110.0287's keywords are missing (~11 of 60 edps papers; under
    `[proc]` the PDF empties them too: a FIDELITY_CHARTER §2 question); a stored `\@subjclass` without a year gets
    "1991 Mathematics Subject Classification" (Perl ams_support.sty.ltxml:131) where TL 2025's amsart prints "2020"
    (amsart.cls:555, :789); 1704.06431's `Fatal` enumerate/`\lx@tags` recursion (pre-existing on main); 3
  - LMCSQED — the lmcs binding's `\qEd` is `\ensuremath{\square}` (`lmcs_cls.rs` ~149), where LMCS.cls:551-553 draws its own
    box: a paper that redefines `\square` (1305.0576's ldiagxy.tex:207, a delimited diagram macro) breaks every `\qEd`
    display (217 errors, more since `\[` is `{equation*}`; 0 with `\qEd` overridden). Give `\qEd` the class's box under
    an internal name; repro `tmp/END399/T3W/repros/1305.0576/qEd_square.tex` (tier 3 A/B, pre-existing; 1202.6352); 1
  - TIKZCDHANG — a tikz-cd diagram whose cell has `\hspace{1em}\rar[mapsto]` loops under calc after an earlier cell's
    calc error (1611.01788 thesis.tex:5400/:5434; both the Nov 17 binary and the release candidate; pdflatex is fatal
    there too, 8 errors): find the loop site (`perf record` at 20 s), then end in a bounded Error. The wall-clock Fatal
    leaves no XML and no log, so the paper's 56k words are lost — a salvage question too. rc2 2026-11-23: still the wall clock at 180 s (59 Errors before it). Deferred, decided by difficulty (DG 2026-11-23); 5
  - BACKMATTERPART (after 0.8.0, needs DG's ruling; TeX fidelity, offer upstream) — LaTeXML nests `\backmatter` (the
    bibliography and index after it) inside the last `ltx:part`, while in TeX `\backmatter` belongs to no part. Since
    SPLITPARENT matches Perl's Split → Scan parents, a split `\part` book's Bibliography/Index footer buttons appear only
    on the last part's pages (KEPT_RELATIONS, OD #1134) and `bib.html`'s title ends "‣ Part II …"; identical in Perl.
    Fix in the XML nesting (close the part at `\backmatter`) or in the relation walk; 1
  - SCANINDEXXML (after 0.8.0, multi-document sites) — Rust's Scan parent fallback (`scan.rs` after `doc.parent_id`)
    lacks Perl's second guess, an `index.xml` in a containing directory (`Scan.pm:117-124`). Split's pages no longer
    use the fallback; 1
  - STREAMNODISPLAY (after 0.8.0, suspected) — `stream_split.rs` has no equivalent of `split.rs::get_pages`'s
    `ltx_nodisplay` filter, so a hidden bibliography (a `\fullcite`-only one under `--splitat`) may become a page only
    on the streamed path. Probe first; 1
  - ROOTTOCSPLIT (after 0.8.0, pre-existing) — a split root page's run TOC comes out as several `<nav class="ltx_TOC">`
    lists (Preface / Pt1 / Pt2) where Perl has one; both split paths, before and after SPLITPARENT; 1
  - LUAERRSEV (after 0.8.0) — a document Lua chunk's runtime error is an Info (`pdftex.rs:118-123`, `luacode_sty.rs:64`)
    while lualatex stops with rc=1; LUAEXEC2 escalates only a chunk that carried a sandbox refusal. Decide whether every
    chunk runtime error is an Error (texlua lacks LuaTeX's node/font libraries, so measure the Error flood on the manual
    sweep first); 1
  - LUAWINCOLON (after 0.8.0, Windows, suspected) — `lua_bridge.rs:200` `out_name_ok`: a `:` that is not a drive letter
    (`a:b`, an alternate data stream) gets lualatex's refusal values and a Warning; kpathsea's paranoid check rejects a
    `:` only in position 2, so lualatex probably writes it and the policy makes it an Error. Empty/NUL/non-string names
    get nil + Warning where lualatex returns fopen's 3 values or "bad argument". Needs a Windows lualatex; 1
  - LINENOAUTHORS (after 0.8.0, content) — lineno/lineno's two extra `\author` lines ("Uwe Lück", "Karl Wette") become
    affiliation contacts; pre-existing, separate from MANFM's `~` fix. Perl itself is not clean on this manual under the
    sweep preload (23 missing-argument Errors near the end); 1
  - AMSWRAPSETAUTHORS (after 0.8.0, synthetic only) — MANFM keeps the `\@setauthors` an amsart-based class ends its load
    with (`class/after`) and runs only what the document appends. A wrapper class that `\LoadClass{amsart}` and then
    appends to (`\g@addto@macro\@setauthors{\par Communicated by …}`) or redefines `\@setauthors` with extra text has
    that text dropped (pdflatex prints it; base kept it as a note). No TL or arXiv class seen (12 real-class papers: 8
    identical, 4 only lose a junk nameless creator). Route: keep the definition amsart's binding ends with (`\@empty`)
    and skip only a derived class's setter that fails a sanity check; 1
  - IJMEMAILCOLON (after 0.8.0, nit) — an ijmart e-mail contact gets `name="e-mail:: "`: the class's `\emailaddrname`
    already ends in ":" and the binding appends ": "; 1
  - LUAFONTIDS (after 0.8.0) — the texlua `font` mirror holds one font (the current one, id 1): `font.id(<real font>)`
    answers -1 with a Warning where lualatex gives its id (e.g. 2 for `\tenln`); hand out mirror ids per named font; 1
  - LUALANG (after 0.8.0) — `lang.new()` succeeds in lualatex (polyglossia.lua:99, luababel.def:274) but is an engine-gap
    Warning plus a Lua error under texlua; a `lang` mirror is its own batch; 1
  - LUAERRSEV2 (after 0.8.0) — answer C modules, network libraries and `package.loadlib` with lualatex
    `-no-shell-escape`'s values and a `W` frame (`loadlib` → no value; `require` of a C module/network library →
    `false, msg`; `luatex-core.lua` L195-200); decide how `G` engine gaps rank once chunk runtime errors are ruled; and a
    chunk with a `W` then an uncaught `D` logs two Errors (the `D` and the escalation) — skip the escalation when a `D`
    already flags the chunk; 1
  - LUASPRINTSPACE (after 0.8.0) — a trailing space in what a chunk `tex.sprint`s is dropped: `\luaexec{tex.sprint("[c] ")}d`
    gives `[c]d`, lualatex `[c] d` (also `luacode*`); the bridge's tokenizing of chunk output; 1
  - NBSPREVERT (after 0.8.0) — Rust's `\lx@nobreakspace` box (`general.rs:118`) reverts to itself; Perl's
    (Base_Utility.pool.ltxml:53-55) reverts to `\nobreakspace`. Since MANFM2 the 28 default frontmatter labels end in it;
    no output showed the difference. Align with Perl; 1
  - BABELTILDE (after 0.8.0, pre-existing, Perl too) — under babel, `x~y` gives "x y" with an ASCII space: babel's `~`
    shorthand `\leavevmode\nobreak\ ` (babel.sty:1504) digests its `\ ` to a plain space and the no-break is lost; TeX prints
    a no-break space (U+00A0). The default labels end in U+00A0 since MANFM2 (OD #1136); body text does not; 1
  - FMTIESTILDE (after 0.8.0, suspected) — ties LaTeXML adds still use the document's `~`: `frontmatter_api/email_lines.rs:103`
    (`named_email_calls` pushes `T_ACTIVE('~')`), `defs/frontmatter_fields.rs` `\lx@row@email@named` (`name={#2~}`),
    `frontmatter_stores.rs:4219` (the off-table label); under `\def~{\verb~}` they would meet OD #1136's Error; not
    triggered by `Emails: …` lines. Spell them with `\lx@nobreakspace` with a RED repro first; 1
  - LUAMKTEX (record, decided) — the Lua sandbox sets `MKTEXTFM`/`MKTEXPK`/`MKTEXMF`/`MKTEXTEX`/`MKTEXFMT`/`MKOCP`/`MKOFM`
    to 0 before kpathsea starts (LUAFONT2): with `MKTEXTFM=1` in the environment, `kpse.find_file` (since SECLUA) and
    `font.read_tfm` ran mktextfm/METAFONT and wrote into `TEXMFVAR`. Stricter than lualatex, which may run mktextfm for a
    missing TFM; fail-safe by choice. State it in SAFETY.md; 1
  - PDFEXTFONTID (after 0.8.0) — `\pdfextension fontattr`/`includechars` (and `\pdfincludechars`) read their font as one
    token, so a `\textfont1`-style font identifier leaks the "1"; add a font-identifier reader; 1
  - PDFCOLORSTACKNOACT (after 0.8.0) — `\pdfcolorstack<n>` with no action: TeX errors, we read nothing and say nothing;
    add the Error; 1
  - PDFFONTFEEDBACK (after 0.8.0) — pdfTeX's `\pdffontname`/`\pdffontobjnum`/`\pdffontsize` expand to nothing (as Perl,
    pdfTeX.pool:97-100), so `\count255=\pdffontobjnum\font\relax` gives "Missing number" where pdflatex gives a number;
    one shared answer for both these and `\pdffeedback fontname`/`fontobjnum`/`fontsize`, which answer 0 (lualatex
    typesets "10.0pt" for `fontsize`; a bare 0 is no dimension): an integer for name and objnum, the font's size with
    `pt` for size (pdftex.web:11359-11369). Reach: microtype-pdftex.def:254 if ever raw-loaded, pdfbase.sty:520-526; 1
  - PDFOPTSPACE (after 0.8.0, advisor 2026-11-22) — one optional-space helper (`skip_one_space(true)`, gullet.rs) at
    pdftex.web's four sites: obj `reserveobjnum` (:34321), annot `reserveobjnum` (:34900), dest, `newwindow`/`nonewwindow`
    (`A\pdf_object_new:n{foo}B` gives "A B", pdflatex "AB"; ordinary documents make 0 such calls, `\DocumentMetadata`
    tagging 23 in pdflatex, 0 here as our tagpdf stubs that path); `read_action_spec` (pdftex.rs) to `scan_action`
    (:34774: `struct`, page/name/num order) and `\pdfdest struct <n>`; `\pdffontattr` on the pdfTeX path (:35375); "Color
    stack action is missing" as an Error; `lastximagepages` before `lastximage`; a non-font operand of `read_font_ident`
    ("Missing font identifier" + back up, as TeX); 1
  - WINEXIT (after 0.8.0, Windows) — `watchdog::exit_now` is `_exit` on Unix only; on Windows it is
    `std::process::exit`, which runs thread-local destructors and flushes stdout, the hazard `_exit` avoids on Unix
    (libxml2's `.fini_array` under a running render thread, ELF-specific). A breach under a stuck thread could hang a
    Windows run. `TerminateProcess(GetCurrentProcess(), code)` is the equivalent (windows-sys, at the OS boundary);
    guard `watchdog::tests::a_breach_ends_the_process_without_exit_handlers` checks the destructor only on Unix; 1
  - WINCRASHSEV (after 0.8.0, Windows) — a render worker that crashes on Windows ends with an exit code (an NTSTATUS such
    as 0xC0000005), which the parent reports as an `Info` ("exited (code N)"), where a Unix signal death is a `Warning`.
    Treat NTSTATUS crash codes (>= 0xC0000000) as a crash Warning; guard `120_midscale_streaming` expects the Warning
    on Unix only; 1
  - *Open GitHub issues without a task line of their own (pointers, 2026-11-23):*
  - ISSUE683 — issue #683 (xworld21): an external SVG sized in em/ex should use `NOMINAL_FONT_SIZE`. The `<?latexml
    nominal-font-size?>` PI ships (`core_interface.rs` ~:719-734); post never reads it (`util/image.rs` `svg_len_px` gives
    `None` for em/ex; `svg.rs` `to_px` uses `em * 10`). Design request; the reporter asked to hold (2026-08-18); v0.8.1+; 3
  - ISSUE585 — issue #585 (nasser1): `\bar`'s `<mo>¯</mo>` looks small. Parity with Perl (`math_common.pool.ltxml`
    :522-523 non-stretchy; ours `math_common.rs` ~:944); the size is the browser's math font (w3c/mathml-core#254). DG
    2026-08-16: await MathML Core; 1
  - ISSUE716 — issue #716 (RFC): a `.rhai` binding API consistency pass before stabilization. Check what the Rhai
    faithfulness rounds (#829, #906, #914) already covered, then close or narrow it; 1
  - ISSUE715 — issue #715 (xworld21): telling Perl LaTeXML, latexml-oxide, TeX and the XSLT apart from inside a
    document. The reporter is decoupling his post-processing first (2026-08-22); answer with what exists today; 1
  - ISSUE670 — issue #670: minted Tier-2, pygmentize highlighting (surpass-Perl, shell-escape tooling); design and DG
    ruling first; 5
  - ISSUE143 — issue #143: Rust stable once `#[thread_local]` stabilizes; blocked upstream; 1
  - ISSUE80 — issue #80: space XMHints as elided arguments (math, parked); 3
  - ISSUE47 — issue #47: accurate LaTeX linting (feature, long-term; with issue 92's diagnostics); 5
  - YTABMATRIX — (RAWDEFR root cause, 2026-11-23) ytableau's `\ydiagram` inside a pgf matrix cell (`tikzcd`, `\matrix[matrix of math nodes]`) closes `svg:g` while the `\vtop` capture block is still open (ytableau.sty:190-206 `\vtop{\iffalse}\fi` … `\iffalse{\fi}` around an inner `\halign`): `\begin{tikzcd} \ydiagram{1} & B \end{tikzcd}` gives 2 Errors on rc2, pdflatex clean; a plain node, a tabular cell, `\vcenter` are clean. Witness 1502.07025 (123 × `svg:g isn't allowed in <ltx:equation>`). Start from `pgfsys_latexml_def.rs`'s matrix cell build; 3
---

## Silent-loss inventory — blocks the 98 % claim (user 2026-10-18)

Moved 2026-11-09 (SYNC1b) to its own living worklist, [`perfect_kernel/SILENT_LOSS_INVENTORY.md`](perfect_kernel/SILENT_LOSS_INVENTORY.md): 226 rows, 220 of them open (the rest closed, accepted or no action), one `###` section per row. A "SYNC row" citation of an inventory row (the numbered items #5-#27, the frontmatter residuals, 108b, Z14, the int153a flip row, "Before the flip", "Open residuals lifted from landed rows") resolves there by the same title.

## Rust Error Fixes

Rust-only defects (a port that diverged from the Perl it translates, not a Perl-origin defect), each with its
KNOWN_PERL_ERRORS cross-reference when one applies. Perl-origin defects go to `parity/KNOWN_PERL_ERRORS.md`.

Each entry's body (defect, cause, fix, guards, witnesses) is verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "From \"Rust Error Fixes\"". Titles here:

- **A `filecontents` capture ends at `\end{<current environment>}` (formerly OD #227)**
- **`{filecontents}` ends at its `\end{…}` anywhere in a line, as LaTeX's does (formerly OD #572)**
- **nicematrix's `\Block{i-j}<tokens>{content}` sets the tokens before the content (Perl: no nicematrix binding) (formerly OD #787)**
- **OD #785's title-font half (a port bug; the OD entry keeps its pointer)**
- **OD #503's font half (a port bug)**
- **OD #468's `content=` half (a port bug)**
- **OD #136's `insert_pi` fix (a port bug)**
- **OD #145's mix-weight fix (a port bug; the entry is in the binding catalog)**
- **OD #493's plain `\matrix` fix (a port bug; the entry is in the binding catalog)**
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
- **Split's pages were filed under a parent guessed from the id prefix (SPLITPARENT, #979)**: Scan now takes the parent Split computed (Split.pm:132-133, Scan.pm:99) on both split paths; guard `118_streaming_split_parity::split_pages_navigate_under_the_parent_split_gave_them` (issue 777's book: 9,262 pages).
- **Document Lua's refused program calls gave an Error and no value (LUAEXEC)**: they answer lualatex `-no-shell-escape`'s values with a counted Warning; a chunk failing after one is an Error, as lualatex errs (OD #1081; guards `perfect_kernel_batchseclua::batchluaexec_*`).
- **texlua's `font`/`lang`/`vf` crashed the Lua bridge (LUAFONT)**: a one-font mirror answers through `\iffontchar`, the rest is a counted `Warning:lua:engine` (OD #1081; guards `perfect_kernel_batchluafont::*`).
- **`\pdfextension <kw>` read one token and printed its operands (LUAFONT2)**: every keyword reads pdfTeX's operands (xfakebold-doc 58 leaks → 0; guards `perfect_kernel_batchluafont::batchluafont_pdfextension_*`).
- **An amsart-based class's own `\@setauthors` ran with the names emptied (MANFM)**: the ams bindings run only the document's additions (OD #1036; ijmart/ijmsample 2 Errors → 0; guard `perfect_kernel_batchmanfm::ams_based_class_setauthors_runs_only_the_document_additions`).

## Current status

The regression baseline for the campaign's sandbox rerun (row R step 3) is cortex runs 375/376 on `2a1fa68d95`, the pushed final tree (2605: F 9 / E 875;
2606: F 15 / E 863; `archive/SYNC_SESSIONS_2026-11.md` bundle int171a, CORTEX2); the last complete arXiv run is 336 (2026-10-10). The perfect-kernel
scoreboard ([`PERFECT_KERNEL_2026-11-13.md`](archive/PERFECT_KERNEL_2026-11-13.md), sweeps of the 2,374 TL manuals) and the per-batch arXiv A/B
measured the program; the cortex reruns of sandboxes 2605/2606 (runs 325-328, 2026-10-04) and run 329's validations are in
`archive/SYNC_SESSIONS_2026-10.md`; the last full sandbox rerun before them (2026-08-02, 60,505 docs) in
`archive/SYNC_SESSIONS_2026-09.md`.

---

## Standing policies & method

### Methodology & the cortex cross-join
- Working method: **re-triage LARGE-error papers** (the single-error tail is exhausted) → bisect the doc to the trigger line → verify Perl with `--verbose` → fix the divergence.
- Cortex API: `http://127.0.0.1:8000/api`. Endpoints: `/api/reports/<corpus>/oxidized-tex-to-html/<severity>` and `/api/corpus/<corpus>/tex_to_html/document/<id>`.

### CSS themes — `ar5iv.css` vs `LaTeXML.css`
- `ar5iv.css` is actively developed (`~/git/ar5iv-css`). Base `LaTeXML.css` starts from upstream Perl LaTeXML's default theme (synced with upstream master in #844).
- **A measured defect against the PDF is fixed in `LaTeXML.css` (DG 2026-11-14)**, mirrored into ar5iv-css, recorded in OD and offered upstream to `brucemiller/LaTeXML`.
- `ar5iv.css` is actively developed (`~/git/ar5iv-css`). Base `LaTeXML.css` follows upstream Perl LaTeXML's default theme; a defect measured against the PDF is fixed locally as a commented `latexml-oxide local delta` (recorded in OD, guarded by `witnessed_css_delta`), mirrored into ar5iv.css where ar5iv shows it, and offered upstream to `brucemiller/LaTeXML` (DG 2026-11-14).
- Sizing intent gap (witness #721, ar5iv#83): the post-v0.8.0 sprint IMGINTENT in row R.

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
