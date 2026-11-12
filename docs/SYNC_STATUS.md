# Engine Sync Status — Active Worklist

> **DO NOT downgrade Errors to cheat the task.** If Perl LaTeXML converts a paper
> without a downgrade, the Rust translation must match by improving the core
> engine — never by silencing diagnostics. New downgrades require explicit proof
> Perl emits the same severity on the SAME paper, else they hide a real gap.
> (User directive 2026-05-15.) Always classify with `latexml --verbose`, never
> `--quiet` (which hides Perl's `Error:` lines); cross-check pathological inputs
> with `pdflatex`.

## How to read this file

**Start at "The task list" below (row R) and take the top unblocked row by Rank.** It is the single entry for the work
lists (DG 2026-11-12: "We should have a single point of entry for work lists"): the pre-merge table ranked, the
after-merge list, then the road's conclusions. Everything after it is supporting detail.

- **The task list (row R)**: every open task — before the merge ranked, after the merge one line each; first, always.
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

*Last compaction: 2026-11-12 (SYNC3) — one list: the ranked rows folded into row R's task list (post-merge rows one line each, their detail whole in [`perfect_kernel/OPEN_ROWS.md`](perfect_kernel/OPEN_ROWS.md) "Post-merge rows moved from SYNC_STATUS (SYNC3)"); the closed rows (0, 0b, R8, XP, R3, R4), the old index table and row R's history moved to [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-12-compaction-sync3), with the earlier compactions' log.*

---

## The task list — start here (row R, the road to the main merge)

- **The task list — the single entry for the work until the merge (DG 2026-11-12: "We should have a single point of entry for work lists").** Updated at every decision and every bundle fold; status counts are read from these two tables. Weights: easy 1, medium 3, hard 5.
- **Done since 2026-11-11:** 66 tasks (weight 182), each in its bundle's entry in `archive/SYNC_SESSIONS_2026-11.md` (P4c, PGR1, FMK1, QA47A in bundle int163a).
- **Before the merge:** 8 ongoing (24: 4 at work — P4b2's and P4cR2's fix rounds, PGR1b, CAP1; GA1 in review; ST3b's fix round queued; P4b, ST3 ready), 36 to come (112); 2 done in int164a (6, RG1, DTL1d — counted at its fold), worked in Rank order — the critical path (1-12: the frontmatter's Option C, the losses, the bars), the parallel fillers (13-29), the close (30-34).
- **After the merge:** 101 (255; 28 of them the old SYNC rows moved here by SYNC3) — DG's pragmatic bar (2026-11-12): regressions and high-frequency / high-impact gaps before the merge; newly found edge cases after it, unless one invalidates the approach.
- **The BM Q&A rows** (QA01-QA93, `~/data/pk_agents/main/bruce_qa/ACTIONS.md`, which keeps each row's text and evidence; status and rank live here) are all catalogued: their PRE work in the table (QA41 → RV1 and PSF1, QA46/QA47 → QA47A, QA60 site 2 → TP3, QA27's input → CSS, QA75, QA80 with QA08, QA35, QA36, QA39, QA50, QA56, QA62 and the Perl sides of QA30 and QA45, QA84, QA92 with QA02's awaited rest), the POST / LATER / HZN rows and those DG moved post-merge in the list below (QA60 site 8 as SR4/SR5); not listed — 28 done (QA01, QA02, QA03, QA07, QA09, QA10, QA12, QA13, QA18, QA25, QA30, QA31, QA33, QA34, QA40, QA42, QA45, QA54, QA55, QA59, QA67, QA70, QA71, QA82, QA83, QA87, QA89, QA93) and 10 that need no work (QA04, QA16, QA21, QA38, QA49, QA65, QA68, QA72, QA74, QA78: confirmed or noted).
- **Detail per row:** the batch reports in `~/data/pk_agents/main/agent_reports/`, the BM Q&A rows in the Q&A tracker (`~/data/pk_agents/main/bruce_qa/ACTIONS.md`), each deferred finding's `_red.tex` (`moves:` names its row).

  | Rank | Id | Task | Step | Weight | Status |
  |---|---|---|---|---|---|
  | 1 | RG1 | RG1 — regression in int162a (RV1 exposed it; 2410.06985): a paragraph inside a group opened mid-paragraph is kept, as Perl's open-brace box keeps it (a `\phantom` line in a tikz node) | 2 | 3 | done in int164a (revRG1c SHIP) |
  | 1 | P4cR | P4cR — regressions in int163a (P4c; CL163): an IEEEtran e-mail on a nameless creator (2307.12121, 2311.01786, 1405.1593), apa6's list collapsed (2508.05776), a name read as the affiliation with words dropped (1806.11431), mark-only contacts (1710.07993) | 3 | 3 | ongoing — three mechanisms (P4cR): P4cR2 at work (IEEEtran: a nested `\IEEEauthorblockA` spliced in, a bare ordinal name line a names block); P4cR1 after P4b2b (the marks gate counts a mark in a box of its own and words each in a box; ~700 `\raisebox` refmark papers) |
  | 1 | PGR1b | PGR1b — regression in int163a (PGR1; sweep #178 G1 2): a `\dimexpr` operand inside pgfmath holding a `\csname`-built name with digits (pagelayout.cls:244-252 `\pal@get{\the\looprows H}` → `\pal@data1H`) splits into `\pal@data` + `1H` on the text round trip — 165 + 9 Errors on pagelayout's manuals (pdflatex clean; int162a 0) | 3 | 3 | ongoing |
  | 1 | CAP1 | CAP1 — regression in int163a (QA47A × caption; CL163): caption's binding lets `\@makecaption` to `\caption@makecaption` as caption.sty:270, so a `cmd/@makecaption` hook under babel-spanish patches caption's body (11 asmeconf papers, +2 Errors each: 2401.04032, 2505.14646, …) | 5b | 1 | ongoing |
  | 1 | FNR1 | FNR1 (GA1d/GA1e Q2, ~480 papers): `\footnoteremember`/`\footnoterecall` across three authors — the emitter writes an earlier author's call as a note and never replays its `\newcounter` before the later creators (`declaration_reads_past` declines only an unspaced body; RED `footnoterecall_counter_defined_by_an_earlier_creator_red`; 2407.13730, 2606.19583: an empty mark), and `\footnotemark[\relax]` is silent where pdflatex reports Missing number (RED `footnotemark_missing_number_is_silent_red`); on P4b2's emitter after P4b2b | 3 | 3 | todo |
  | 1 | ST3 | ST3 — a study: state carried between conversions in one process (P4c3: 90a's elsarticle row green in-process, wrong from the CLI on the base); the cortex worker converts many documents per process | 5 | 3 | ready (revST3 SHIP; `c3528bf1d2`: keyvals' report-once set reset per conversion; the 90a row is ISO1's merge, not state); lands with ST3b |
  | 2 | P4b | P4b part 1: the bindings' `leads_with_mark` sites on the digested reading (`leads_with_label`; 23 jfm `\aff` papers better) | 3 | 3 | ready (revP4b SHIP); lands with P4b2 in int164a |
  | 2 | P4b2 | P4b2 (+ 2410.01932's lost name marks, a P4b part 1 regression, revP4b): the API Cut's "marked" decision (`requested_marked_entries_in`, `holds_requested_marked_entries`) ported WITH `place_uncut_copies`'s copy rule (one fact, every reader); 2608.21425's llncs hand-marked block checked | 3 | 5 | ongoing (fix round P4b2b after revP4b2 SHIP; `5e4324996e`, A/B 1,433: 1 changed; decisions: 2410.01932 → an authblk name keeps a mark it printed (the fix round), cond-mat0206329 accepted, the cost-gate pins accepted (P6)) |
  | 2 | P4b3 | P4b3: REVTeX's group-rule mark readers (`requested_marks_held`, `requested_mark_lines`, `requested_marks_unprinted`, `content_marks`, `leading_macro_mark`) on the digested reading | 3 | 3 | todo |
  | 2 | P4b4 | P4b4: the remaining per-line token cutters — `marker_leads` (~13 sites), `author_mark_position` (~15), `author_mark_operands` (7), `MARK_SWITCHES`, relocate.rs 1312/1352/1529 — each with its red shape; `events::lexer.rs:529`'s stale `names_requested` (P4b2 Q4); REDs `api_cut_macro_marks_unrequested_line_red`, `llncs_institute_hand_marks_cut_red` | 3 | 5 | todo |
  | 2 | P4f | P4f: the second cutter (`affiliation_calls` for `\lx@add@store@affiliations`, `\lx@add@store@authorblock`, `row_calls`, `\lx@add@affiliations` — aa, epl, ptephy, interspeech, ijcai, llncs stores) through the declared `cut_list`, so its printed marks show (Q1; revP4c S2); a reach census first | 3 | 3 | todo |
  | 3 | P5 | Option C P5: new facts (IEEEmembership grade, "on behalf of", surname match) | 3 | 5 | todo |
  | 4 | P6 | (incl. moving the declared cut `cut_list` out of the moved directories — the fences P4c3 raised: MOVED_CUT_KIND_CODE_LINES 2219 → 2236, OLD_READER_EXPORTS; P4b2's cost gate `may_open_with_mark` and its pins TOKEN_READS 28 → 33) Option C P6 + the frontmatter code/test compaction | 3 | 5 | todo |
  | 5 | P2-8b | P2-8b: IEEEtran block-call fact, grid rule in the reading | 3 | 5 | todo |
  | 6 | P2-3b | P2-3 rest: llncs one-block list (DG's option a/b pending) | 3 | 3 | todo |
  | 7 | TP3 | TP3: merge the walks; QA60 site 2; BM probes' losses (ams, session) | 3 | 3 | todo |
  | 8 | AUT1 | AUT1: macro-held names (Frontiers `\author{\Authors}` and the like, ~2,900-3,400 arXiv papers): no word lost on 36 real witnesses, but the "markup lost" Warning over-counts — count only real losses (study AUT0) | 3 | 3 | todo |
  | 9 | UNC | Uncounted losses: IEEEtran grade drops, vmsta e-mails, kernel \thanks shapes | 2 | 3 | todo |
  | 10 | CT1a-r | CT1a count design (DG ruling pending) | 2 | 1 | todo |
  | 11 | TRUTH | Bars: the truth set | 4 | 5 | todo |
  | 12 | HOLD4 | Bars: fresh holdout v4 read once; 98/98 | 4 | 3 | todo |
  | 16 | DTL1d | DTL1: out-of-range numbers → Error (OD #519 revised) | 5 | 3 | done in int164a (revDTL1d SHIP) |
  | 17 | GA1 | GA1 + the \@listctr fix (thebibliography's enumiv) | 5 | 3 | in review (GA1d `c4c7d2c6f8` revGA1d SHIP + GA1e `054244fa83`: `\@listctr` = enumiv under `\@nmbrlist`, varioref's `\c@vrcnt`, `\value` as `\csname`; revGA1e next) |
  | 18 | GA2 | GA2 | 5 | 3 | todo |
  | 18 | GA3 | GA3 | 5 | 3 | todo |
  | 18 | GA4 | GA4 | 5 | 3 | todo |
  | 18 | GA5 | GA5 | 5 | 3 | todo |
  | 18 | F1-4 | Fences F1-F4 | 5 | 1 | todo |
  | 19 | PSF1 | PSF1: `\psfrag` fragments dropped silently (98 of 30k ≈ 9,100 arXiv papers; Perl too for graphicx; latex+dvips prints them): a counted Warning per dropped fragment before the merge (RED `reversion/psfrag_fragment_dropped_silently_red`); rendering them after it | 5 | 3 | todo |
  | 19 | ST3b | ST3b (ST3's rule): one per-conversion reset (`reset_conversion_state`) that session start and `reset_thread_engine` both call — refstep `PENDING_STEPS`/`REFSTEP_PROPS`, the subfile load count, `FALLBACK_NAMED`, log suppression — with rows in `perfect_kernel_batchst3`; before CORTEX (a persistent worker converts many papers per thread; the third such miss) | 5 | 3 | fix round ST3c queued after revST3b SHIP (MOUTH_CLOSED_ERRORS never reset — a fleet thread goes Fatal after one bad paper; predecessor checks; `e2f4fe7e79`: `reset_conversion_state` in core, called by `initialize_report` and `reset_thread_engine`; refstep after a caught panic) |
  | 19 | STRM | STRM (ST3b): a conversion switched from eager to streaming (`stomach.rs spill_watermark_bytes`, read from the PROCESS's RSS) gives different XML for babel papers (`xml:lang` on every streamed section; 2507.13358; RED `streaming/babel_language_marks_each_streamed_section_red`) — a persistent cortex worker's RSS grows, so output depends on paper order: (1) streaming byte-identical to eager (its segment finalize's language context), (2) the watermark measured from the conversion's own starting RSS; before CORTEX | 5 | 3 | todo |
  | 20 | PERF | Perf + RAM pass, remaining levers (#346) | 5 | 5 | todo |
  | 21 | FMPERF | Frontmatter performance evaluation (#516), after P6 | 5 | 3 | todo |
  | 22 | T1 | T1: the repro suite move | 5 | 3 | todo |
  | 23 | ROW0A | Row 0a deferred manual REDs decided | 5 | 3 | todo |
  | 24 | QA80 | Upstream patch set prepared: QA08 equationgroup id, QA30 bibunits, QA35 subparagraph, QA36 abstract-heading XSLT, QA39 TOC copy, QA45 \list labels, QA50 \pretocmd on locks, QA56 \mbox, QA62 \bm | 5b | 3 | todo |
  | 25 | QA92 | Reply owed to BM (Q3.9, Q4.6, Q3.13; QA02's rest awaited from BM) | 5b | 1 | todo |
  | 26 | CSS | The sizing CSS (goal end; QA27: BM's em-of-parent lengths as input) | 6 | 3 | todo |
  | 27 | QA84 | Comment compaction across the crates (the last code pass) | 5b | 5 | todo |
  | 28 | QA75 | Eyeball review of rendered HTML at the close | 5b | 3 | todo |
  | 29 | CLEAN | Workspace cleanup (keep-list to DG) | 5 | 1 | todo |
  | 30 | DOCS | Goal docs current | 6 | 3 | todo |
  | 31 | SWEEP | Closing manual sweep (G1-G5) | 6 | 1 | todo |
  | 32 | CORTEX | 2605/2606 cortex reruns (fleet rebuild: DG's OK) | 6 | 3 | todo |
  | 33 | CIPERF | CI performance work, last before the merge | 6 | 5 | todo |
  | 34 | MPR | The merge PR (DG confirms) | 6 | 1 | todo |

  **After the merge** (id — task; weight; a moved SYNC row's detail is in `perfect_kernel/OPEN_ROWS.md` "Post-merge rows moved from SYNC_STATUS (SYNC3)" under its id):

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
  - SVA — SV-A: \csname via \ifincsname; 3
  - TBLR — tabularray first-token, \g__tblr_table_count_int, tblr-only \hline REDs; 1
  - QA05 — Kernel-version switch + \@ifundefined per year (QA05/QA06); 1
  - QA11 — Native pgf/l3fp vs raw TeX inventory; 3
  - QA14 — Untracked length parameters census; 3
  - QA15 — TeX's two kinds of unread: RED + reach; 3
  - QA19 — Schema-extension inventory for BM; 1
  - QA22 — Page furniture kept as metadata (QA22/QA23, census done): element designed with BM, implemented; 5
  - QA24 — Minipage width on a section from a minipage; 1
  - QA29 — xml:id nesting check; 1
  - QA43 — Bindings with no semantics census; 3
  - QA47B — Hooks B-G: env begin/end, enddocument/*, file/after order, include/*, fonts, label; 3
  - QA61 — Unknown classes' title-page stores as frontmatter items; 3
  - QA64 — \pdfoutput latexml.sty option (DG decides timing); 1
  - QA69 — 8-bit fontmap fallback census; 3
  - QA76 — brucemiller/LaTeXML issue survey; 3
  - QA85 — ORGANIZATION.md extended to every crate; 1
  - QA90 — The 54 preamble-only commands after \begin{document}; 1
  - QA91 — Text before \maketitle: order (DG + BM); 1
  - SR4 — SR4 (QA60 site 8): a title note held in \title (\titlenote) becomes a pubnote (1601.00073, 1805.02241, cs0405006) — with the page-furniture element (QA22/23); 1
  - QA17 — `\iffontchar` outside Computer Modern: a K-row with a repro; 1
  - QA20 — Open/close vertical bars (VERTBAR read as MIDBAR): a math RED row (math parked); 1
  - QA26 — Large separating spaces kept as a class or role; 3
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
  - QA58 — OmniBus definitions as guesses: `\newcommand` may redefine without complaint (input to SV-D); 1
  - QA63 — XeTeX / LuaTeX emulation modules by the intended engine; 5
  - QA66 — Each Error's site marked in the output (`ltx:ERROR`), designed with BM; 3
  - QA73 — `ELIDEOP` renamed to an elision role, together with Perl; 1
  - QA77 — A Rust bst-driven BibTeX keeping each field's meaning (JATS, RDFa); 5
  - QA79 — LaTeXML 0.9 as the reference on its release: oracle re-baselined, parity re-run; 3
  - QA81 — A latexml-oxide home page (Sammy in the logo); 3
  - QA86 — A principled organization shared by both implementations, proposed to BM; 3
  - QA88 — Horizon: fully idiomatic Rust, the parity ties broken (DG's call); 1
  - ARXIV — The arXiv rerun on the merged head; 5
  - CSSPORT — The ar5iv-css port (PR #54); 3
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
  - IEEO — IEEEtran preprocessing residuals (P4cR2): an inner block inside a group stays nested (`ieee_nested_affiliation_block_in_group_red`); the ordinal content-check false pair (`ieee_ordinal_floor_line_names_unnumbered`, 2 W); the Cut gives an unmatched address in an inline marked list to the last entry's author; two sibling blocks after one names block reach the last author only (revP4cR2 q14); a block inside the outer's `\thanks` read as an affiliation (q1); 1
  - CNTE — counter readers' leniency where pdflatex errs: `\newcounter{foo}[nosuch]`, `\setcounter{chapter}` in article, `\addtocounter{zzz}` (pdflatex 4 `!`; ours 0 E 5 W) (revGA1d); `\bibitem` does not step enumiv (`bibitem_steps_the_list_counter_red`); cleveref + varioref internals (`cleveref_vref_with_varioref_red`, ~1,680 papers, errs on the base) (GA1d); 3
  - EBN — EB1's nits: the calc Warning's wording when calc is not loaded; the ifpdf Error row's message; the DTL1 row's whole `<para>`; one jing process per row; 1
  - MENC — `<m:menclose>` still emitted for `\cancel`/`\boxed` (MathML Core removed it; no mechanical replacement: a rendering + golden change on its own branch; deferred by DG 2026-07-30); 3
  - CI — CI under 30 minutes on `main` (workflow edits, a PR tier); its localhost study feeds CIPERF; 3
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

### The road — conclusions (row R; its history in the archive's SYNC3 section)

- **Order (DG 2026-11-09, the inventory `agent_reports/2026-11-09_PREMERGE_inventory.md`), each step reviewed and bundled:** (1) single engine clean; (2) every RED seen on a real paper; (3) Option C P1-P6; (4) the bars; (5) non-frontmatter; (5b) the BM Q&A work; (6) the close. The table's Rank is the order work is taken in.
- (1) **Done:** `author_block/` deleted (int155a); the e0 fence hardened (int156a); Z18n folded into P2-7 (`shared_row/` deleted, P27b). The old reader's code left — `classify/old_reader/` and `defs/author_macros/ijcai.rs`, fenced by e0's `MOVED_OLD_READER_FILES` (the IJCAI chain and its `@marked`/`@shared` gates, authblk's and IEEEtran's readers, llncs's `affiliation_calls`) — is rerouted in P4b-P6.
- (2) **Narrowed for the merge (DG 2026-11-11: "The 'fix them all' is a long-term goal for arXiv"):** a real-paper loss is fixed before the merge only at arXiv reach ≥ ~500; below it, counted by a Warning (no silent loss) and kept as a catalogued RED. Left: neurips_2024, 77p's grouping (2603.07345), aa `\mail{\email{x}}` ([`perfect_kernel/SILENT_LOSS_INVENTORY.md`](perfect_kernel/SILENT_LOSS_INVENTORY.md) "Bundle int157a — the residual").
- (3) **Option C** ([`parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`](parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md)): done — P1, P2-0, P2-1 outside llncs, P2-2, P2-4, P2-5/P2-6 gates, P2-7, P3, P2-3 half, one title-page reading (TP1/RK1), F0 + P4a, P4d; left — ranks 1-8; after the merge — TP2, the separator's print-time meaning (SMB1-SMB3).
- (3) **Rulings 2026-11-11** (the plan's "Segmentation is digested"): each port deletes the spelling path it replaces; its output changes only where the PDF agrees (a condition; the goal stays every source word kept in high-quality semantic markup); every changed paper classified, none worse; its guards updated in the same commit; a separator carries its meaning (kernel / binding / raw), never a per-class spelling list. Each bundle reports the spelling-split sites left, the net `src/` lines, the frontmatter line count and a dev re-score.
- (4) **The bars** (DG 2026-11-09): a truth set, then a fresh holdout v4 read once — 98/98 on dev3/devh/dev20k and v4, longtail ≥ off with no regression.
- (5) **Non-frontmatter:** the generalization audit (GA1-GA5) before the merge; FMPERF at the end of the frontmatter work (the rehearsal, the title-page runs, the separator query, the emitter; `instructions:u` against the 2026-10-31 audit's 18 documents; one lever per batch, byte-identical); PERF; T1; ROW0A; CLEAN (keep-list to DG).
- (5b) **The BM Q&A work** (DG 2026-11-11: "I approve your plan, let us proceed with full speed ahead"; BM's answers of 2026-11-05): done in int161a — QA45 (ML1, OD #180), QA54 fix (b) (PI1, KPE #1222), QA30 (BU1, KPE #1223), QA10 (DTL1a+b, OD #1004, KPE #1221); 2026-11-12 — QA41's audit, QA89 (OD #1005); QA60 site 8 (SR1) parked after DG's "author's group" ruling (SR4/SR5 after the merge).
- (6) **The close:** DOCS; SWEEP (row 0's G1-G5 on the final head); CORTEX (G6, the fleet rebuild needs DG's OK); CIPERF, the last step before the merge (DG 2026-11-11: "once we merge we will start working on public issues and PRs, and start running CI instantly"; releases run the whole suite on Linux, macOS and Windows within each runner's timeout: dump formats built once per run, shard balance, the long-pole tests, table-driven guards; study `agent_reports/2026-11-11_TC0_report.md`); MPR (DG confirms).
- **Pushed bundles:** int154a → int162a, each chain's figures in its "Bundle <label>" entry of [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md); the last, int161a: gates 8041/8041, A/B 28,354 → 86 changed (0 worse), E 45,813 → 45,788, F 172 → 172.

### Detail for the live pre-merge rows

#### ROW0A (rank 23) — M3: the manuals' remaining regressions

- Row 0 (the TeX Live manuals, every regression addressed before the merge: met at sweep #152, held through #156; its record in the archive's SYNC3 section) leaves only these deferred REDs.
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

#### T1 (rank 22) — Repro suite move

- **Repro suite move**: `tools/perfect_kernel/repros` (1,469 files) → `latexml_oxide/tests/repros` header-contract trials (release CI + `LATEXML_FULL_TESTS=1`);
- drain `tools/`
- State: **OPEN**, not started (user 2026-10-02)
- Size: large
- Detail: memory `project_repro_suite_move`

#### CIPERF (rank 33) — the CI performance work

- Study TC0 (`agent_reports/2026-11-11_TC0_report.md`): the per-batch guard modules are 72 % of the suite's time, about half of it per-test fixed cost; the localhost CI baseline (int64a) is the post-merge CI row in `perfect_kernel/OPEN_ROWS.md`.

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
