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
- **Done since 2026-11-11:** 102 tasks (weight 280), each in its bundle's entry in `archive/SYNC_SESSIONS_2026-11.md` (GA4, F1-4, P5, UNC, TP3, AUT1, ROW0A in bundle int168a, with P6a-c).
- **Before the merge:** 3 ongoing (11: P6, TABU2, QA80 held for BM's go), 2 done on their branches for int169a (6: P2-8b's P2-8a, CT1a-r), 17 to come (49), worked in Rank order — the critical path (1-12: the frontmatter's Option C, the losses, the bars), the parallel fillers (13-29), the close (30-34).
- **After the merge:** 175 (371; 28 of them the old SYNC rows moved here by SYNC3) — DG's pragmatic bar (2026-11-12): regressions and high-frequency / high-impact gaps before the merge; newly found edge cases after it, unless one invalidates the approach.
- **The BM Q&A rows** (QA01-QA93, `~/data/pk_agents/main/bruce_qa/ACTIONS.md`, which keeps each row's text and evidence; status and rank live here) are all catalogued: their PRE work in the table (QA41 → RV1 and PSF1, QA46/QA47 → QA47A, QA60 site 2 → TP3, QA27's input → CSS, QA75, QA80 with QA08, QA35, QA36, QA39, QA50, QA56, QA62 and the Perl sides of QA30 and QA45, QA84, QA92 with QA02's awaited rest), the POST / LATER / HZN rows and those DG moved post-merge in the list below (QA60 site 8 as SR4/SR5); not listed — 28 done (QA01, QA02, QA03, QA07, QA09, QA10, QA12, QA13, QA18, QA25, QA30, QA31, QA33, QA34, QA40, QA42, QA45, QA54, QA55, QA59, QA67, QA70, QA71, QA82, QA83, QA87, QA89, QA93) and 10 that need no work (QA04, QA16, QA21, QA38, QA49, QA65, QA68, QA72, QA74, QA78: confirmed or noted).
- **Detail per row:** the batch reports in `~/data/pk_agents/main/agent_reports/`, the BM Q&A rows in the Q&A tracker (`~/data/pk_agents/main/bruce_qa/ACTIONS.md`), each deferred finding's `_red.tex` (`moves:` names its row).

  | Rank | Id | Task | Step | Weight | Status |
  |---|---|---|---|---|---|
  | 4 | P6 | (incl. moving the declared cut `cut_list` out of the moved directories — the fences P4c3 raised: MOVED_CUT_KIND_CODE_LINES 2219 → 2236, OLD_READER_EXPORTS; P4b2's cost gate `may_open_with_mark` and its pins TOKEN_READS 28 → 33) Option C P6 + the frontmatter code/test compaction | 3 | 5 | ongoing (P6S study: 52,132 frontmatter code lines, P6a-e ≈ −275; P6a, P6b (one collaboration vocabulary), P6c (one font table) done in int168a; P6d (address-only lines on the reading, partial) and P6e (one marks walker, the DOM respell deleted: −0.56 / −0.41 %) reviewed SHIP, land in int169a; P6f (P6d's rest: `affil_splits` onto the separator query, the remaining address-only token readers) to come) |
  | 4 | P6f | P6f (P6d's rest, revP6de): `affil_splits` (`classify/separators.rs` `author_affil_splits`, 7 spelled commands) onto the separator query by meaning (own A/B); the remaining address-only token readers (`line_addresses` at affiliations.rs:120/558/603, kind.rs:74/299, email_lines.rs:436; `kind.rs:369 prints_addresses_only`) onto `only_addresses` | 3 | 1 | todo |
  | 18 | TABU2 | TABU2 (ROW0A's tabu find, TABU1): the tabu binding's three gaps — `\tabu@init`'s lets (`\centering` ends the row), `\tabu@trivlist` through a kernel `\lx@trivlist@hook`, the `[1pt]` rule-width specs after a column rule, `{tabu*}`/`{longtabu*}`; 10 witnesses 313 → 0 Errors (2210.07016 95 → 0); ~880 arXiv | 5 | 3 | ongoing (TABU2 + TABU2b reviewed; TABU2c: the hook on thebibliography and two proof bindings) |
  | 5 | P2-8b | P2-8b: IEEEtran block-call fact, grid rule in the reading | 3 | 5 | done in int169a (P2-8a merged as P28m + P28m2: routes and grid columns by meaning, the body read again at the flush; A/B 2,654: 0 changed; revP28m, revP28m2 SHIP; the block-call fact → post-merge P28B) — folds at the push |
  | 6 | P2-3b | P2-3 rest: llncs one-block list (DG's option a/b pending) | 3 | 3 | todo — DG 2026-11-12 option (a): renumber the queue's `author:N` positions and the resolver's author entries by the reading's cut for every whole-list binding (kernel, OmniBus, aa, sv, webofc, ptephy), then llncs's marked list as one block |
  | 10 | CT1a-r | CT1a count design (DG ruling pending) | 2 | 1 | done on its branch (CT1ar `68b18b34a6`, DG accepted the narrower rule: a mark glued to another same-label entry's text counted; 2204.02412 counted; XML byte-identical on 1,931; revCT1ar SHIP) — lands in int169a |
  | 11 | TRUTH | Bars: the truth set | 4 | 5 | todo |
  | 12 | HOLD4 | Bars: fresh holdout v4 read once; 98/98 | 4 | 3 | todo |
  | 20 | PERF | Perf + RAM pass, remaining levers (#346) | 5 | 5 | todo |
  | 21 | FMPERF | Frontmatter performance evaluation (#516), after P6 | 5 | 3 | todo |
  | 22 | T1 | T1: the repro suite move | 5 | 3 | todo |
  | 24 | QA80 | Upstream patch set prepared: QA08 equationgroup id, QA30 bibunits, QA35 subparagraph, QA36 abstract-heading XSLT, QA39 TOC copy, QA45 \list labels, QA50 \pretocmd on locks, QA56 \mbox, QA62 \bm | 5b | 3 | prepared — 8 of 9 patches as local branches of a LaTeXML clone (`/home/deyan/data/pk_wt/latexml_upstream`, `make test` 35/35 before, all eight together 481 tests pass; `.patch` files in `~/data/pk_agents/main/upstream_patches/`): QA08, QA30, QA35, QA36 (latexmlpost 101.8G → 18.6G instructions on a 1,500-section article), QA39, QA50, QA56, QA62; QA45 skipped (needs OD #180's `\@trivlist` opener ported — a larger patch: DG); a summary e-mailed to BM (DG, 2026-11-12); held until BM says go, then submitted at DG's OK |
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
  - AUMK — answered marks hidden on both sides: the kernel `\author` whose own lines answer its names' marks (`kernel_author_lines_answered_marks_hidden_red`, as Perl), its authblk sibling (`authblk_author_line_answers_one_mark_red`), authblk's own `\author[1]`/`\affil[1]` generated marks dropped (`authblk_labelled_marks_dropped_red`, as Perl) (P4b2b, revP4b2b); llncs `\inst` names whose marks a macro prints hide the institute's marks too (`llncs_inst_names_macro_marked_institute_marks_hidden_red`; P4b4a: 1803.02579, 2201.04974, and CL166's seven: 0807.2091, hep-th0502229, 2412.12966, 0706.0112, 0803.2400, 0808.1862, 2508.08560's name marks); 3
  - CALCR — calc's `read_term` does not skip `\relax` before `*`/`/`: a stray "2+1" printed (`\setcounter{a}{\value{b}\relax*2+1}`; revGA1e S2); 1
  - P4CRN — P4cR1's residuals: a name's mark in a box with a space lost uncounted (`ieee_name_mark_in_spaced_box_lost_uncounted_red`), apa6 entries' marks dropped (`apa6_marked_list_entry_marks_dropped_red`), an unmatched e-mail address on a nameless creator when local parts run out of name order (`ieee_email_run_unmatched_address_nameless_red`, 2409.10792, 2410.09414; a ruling first), `unanswered_mark_lost`'s wording for a `ref:` mark (revP4cR1); 3
  - FNMK — footnote marks: `[\relax 2]`/`[\value{nosuch}2]` read as digits, a braces-only tail silent (`footnotemark_relax_then_digits_reads_digits_red`), a blank `[]` (`footnotemark_blank_is_tex_zero_red`), `\ifnum` with a missing number (`ifnum_missing_number_inserts_equals_red`) (GA1f, revGA1f); 1
  - STRM2 — streamed-vs-eager differences under a forced switch (pre-existing; STRM): ids order (1705.02293, 2507.13358), whitespace (1005.3879, 2206.03430), footnote split (2010.06608), `toc` inlist (1907.01464), `\quad` reversion (2509.09797); the eager→streaming switch rate in production to measure; `finalize_context_font` DRY with `finalize_rec` (revSTRM); 5
  - OMNT — OmniBus/KOMA title notes step no counter and print arabic (2407.13730, 2606.19583 recalls "0" vs pdflatex ∗; `omnibus_title_footnotemark_prints_fnsymbol_red`; reach to measure) (FNR1); 3
  - FNRR — the rehearsal reads a printing definition's later use (the true signal behind FNR1's gate; `author_note_macro_resets_an_existing_counter_red`; the replay writes a note from a printing call's expansion with its markup: 2604.20516) (FNR1, FNR1c); 3
  - IEMP — a minipage's optional argument leaks into IEEEtran affiliations ("t].44", 1901.06399; invented text; int162a too; reach to measure) (P4fC); 1
  - DTLX — DTL1d's counted pre-existing overflows (varwidth → VARW): a subscript-only math box of height 0 (2503.14426; Perl too; HARD), pgfplots `y expr` (2407.10017), xymatrix/tikz shapes (1409.3605, 0704.0778, 2409.07148, 2408.05806, 2411.12316), 1103.1294's cap (TooManyErrors sooner; nicefrac in xymatrix) (CL164); 5
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
  - DEPN — the deposit empties `\@date`, a class reading it as a number prints "Exam session 0" (`deposit_emptied_date_read_as_number_red`; TP3, BM probe `session`); 1
  - AUTP — a document macro WITH parameters holding names stays by text, counted (the end of its arguments unknown when the call ends the field) (`article_author_macro_with_parameters_holding_names_red`; AUT1); 1
  - AUTL — a leading mark after a comma (`, $^{1}$Bob`) lost, counted (`omnibus_author_leading_mark_after_a_comma_is_lost_red`; AUT1); 1
  - AUTC — hep-ph0301204 +0.60 % (a 1,000-name probe +4.6 %): the cost of emitting the gap marks AUT1 keeps — make a shown mark's emission cheaper (AUT1b); 1
  - GAPC — a shown gap mark loses its trailing comma (0704.2093; CL168); 1
  - GAPS — `$^{1,*,**}$` after a name shows `1,∗`, the `∗∗` (the corresponding-author note's) not shown (1507.03321; CL168); 1
  - FNTN — `\fontname\font` under NFSS prints the CM name (Perl the same, `cmr10`); author reach 0 of 2,027 census papers; needs `.fd` → TFM in `\selectfont` (ROW0A, typog-example); 3
  - GINS — `\lx@Gin@include`/`\lx@Gin@setfile` local where `\includegraphics` is global (since batch 70a; graphicx loaded in the body or a group loses the graphic, Perl keeps it); EASY: `scope => Global` on both (ROW0A, repro `gin_body`); 1
  - TUZK — an overflow `&` grows a column (Perl Alignment.pm:137-143) where TeX §792 ends the row, the cascade an Error per extra `&`; `\@nolnerr` silent (SHARED; tuzuk; ROW0A); 3
  - CTOV — CT1ar's count calls a space-separated repeat glued: `$^a$Dept of Physics $^a$Univ X` over-counted "a", XML right (`ctlabel_space_separated_repeat_over_counted_red`; DG accepted the over-count); 1
  - CTIC — an isotope after a comma (`, $^{3}$He Lab`) cut as a second entry of label 3, its "3" lost uncounted (`ctlabel_isotope_after_a_comma_uncounted_red`; revCT1ar); 1
  - CTSF — `$1$St Floor`: a math number leading a line read as a mark, its "1" lost uncounted (CT1a's St Floor repro; CT1ar); 1
  - P28G — guard rows for P28m2's generic take-back outside IEEEtran (an svjour3/kernel `\author` replaced: the nameless creator and its note dropped) and the `\let` copy defined only after `\author` (revP28m2's l1 probe); 1
  - P28C — `\ifblind Anonymous Authors\else <blocks>\fi` (literal or in a macro): a nameless creator with the words as its affiliation, where the base named it (`ieeetran_author_blind_conditional{,_in_a_macro}_red`; revP28m S2); 1
  - P28B — P2-8b's block-call fact (the whole IEEEtran `\author` one block the reading cuts; the grid rule in the reading; the binding's ~820 lines of token passes moved) — DG: after the merge (`agent_reports/2026-11-12_P28b_report.md` options B/C); 5
  - SUPIT — a floating superscript rewritten into text `<sup>` (TeX_Math.pool.ltxml:527-550, every `word$^{2}$` and `Name$^{1}$` mark) takes the math default font: `<sup><text font="italic">2</text></sup>` where pdflatex prints CMR7 upright; Perl-shared (KNOWN_PERL_ERRORS #1235), OD #928's frontmatter residual (`floating_superscript_rewrite_digit_italic_red`; P4b4r Q2; since AUT1 ~30 Frontiers papers' held-name marks too, as their inline twin: `omnibus_author_math_mark_in_the_name_run_is_italic_red`); 1
  - TAILN — a name ending in a bare accent command (`A.~Pompo\v`, 0705.2788) falls to the plain-text path and loses its no-break space, counted (PMK1's `tail_call`; CL165); `tail_call` on an operand (`authblk_mark_macro_ending_in_let_operand_red`) (revPMK1); 1
  - IEEO — IEEEtran preprocessing residuals (P4cR2): a refmark inside a font switch (`ieee_nested_block_bold_refmark_spliced_red`), a mark after a box (`ieee_nested_block_mark_after_box_spliced_red`), marks after `\\[2pt]`/`\newline` (`ieee_nested_block_after_spaced_break_spliced_red`) (revP4cR2b); an inner block inside a group stays nested (`ieee_nested_affiliation_block_in_group_red`); the ordinal content-check false pair (`ieee_ordinal_floor_line_names_unnumbered`, 2 W); the Cut gives an unmatched address in an inline marked list to the last entry's author; two sibling blocks after one names block reach the last author only (revP4cR2 q14); a block inside the outer's `\thanks` read as an affiliation (q1); 1
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
