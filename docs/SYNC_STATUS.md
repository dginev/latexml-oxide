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
- **The arXiv long tail (v0.8.2+)**: [`LONG_TAIL.md`](LONG_TAIL.md) — every task affecting fewer than 200 arXiv
  articles, or synthetic-only (DG 2026-11-23); out of scope for v0.8.0 and v0.8.1.
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
- **The goal now (DG 2026-11-23): [`GOAL_v_0_8_1.md`](GOAL_v_0_8_1.md).** Two timelines: v0.8.0 final on `main` by 2026-11-30 (small reviewed fixes only: regressions, crashes, rc2 and rc3 tester reports); everything else in this list lands on `int/towards_v0_8_1` for v0.8.1. Each task ends fixed, ruled or moved (the goal page defines each).
- **Where it stands (2026-11-25):** the full arXiv run on the cutoff is run 397 (November 2026, 98.1 % of 2,962,950
  error-free); its regressions were classified and the clusters our changes caused fixed (#954-#964), the residue in
  row R. **0.8.0-rc1** is published as a prerelease (tag `f7f8120f349`, merged to main as #919). **0.8.0-rc2** is
  tag `f219a9e9d68` (#981), published as a prerelease on 2026-11-23 (release run 45107500545, 17 assets). **0.8.0-rc3**
  is tag `bb903363a6b`, published as a prerelease on 2026-11-25 UTC (release run 45533293760, 17 assets): #987-#1003
  (issue 995's resource cache, the CLI options, the CLI hardening, the cortex worker's runtime-binding refusal, the
  `unsafe` ratchet) and rust-libxml 0.3.27, which removes 0.3.25's 2.1x slowdown on deeply nested TikZ pictures.
  **0.8.0-rc4** is tag `9fa31b9a7ad`, published as a prerelease on 2026-11-25 UTC (release run 45664468839, 17
  assets): #1007 (a journal class's `\jyear{…}` printed as a body paragraph; ar5iv issues 353 and 301), #1008 (issue
  1006: split pages' navigation under `--sitedirectory`) and #1009 (a site directory named through a symbolic link,
  every macOS temporary directory, was refused since rc3; KNOWN_PERL_ERRORS #1334). **0.8.0-rc5** is cut from main on
  2026-11-25: #1012 (a cache closing beside another left `-wal`/`-shm` side files), #1013 (a negative LaTeX argument
  count exhausted memory; non-BMP characters kept), #1015 (issue 1011: books of thousands of `\input`/`\subimport`
  files abort at the string arena's halved ceiling; a crash now leaves its log), #1014 (Windows: the first
  conversion's format build off the `--timeout` clock; figure paths of 260+ characters) and #1016 (release builds
  bundle ar5iv.css; a stylesheet already in the destination is used as-is, OD #1165). It is tag `408aea99958`,
  published as a prerelease on 2026-11-26 UTC (release run 45764381757, 17 assets; platform check 45774240976), with
  #1019 added (a `00README` main file and a stylesheet's copies stay inside their directories). **0.8.0-rc6** is rc5's
  code plus issue 972's Grow in the base LaTeXML.css, the bundled ar5iv-css 0.9.6 and the documentation pass; 0.8.0
  then promotes rc6, reusing its build inputs (DG 2026-11-26: a change after an RC takes a new RC). Before it: the rc1 → rc5 health A/B of the published binaries (10,536
  arXiv papers, 1991–2026) found no content regression and fewer Errors, and the 2605 rerun on rc5 (run 417) matched
  rc4's run 415 on all 30,079 papers.
  rc2 was re-issued on the large-document perf work (DG 2026-11-21): #970 (STRMWARN,
  CORELEAK, POSTPERF, CHAPLINKS, ALLOCTRIM, BIGSPLIT, mimalloc 2.3.2) and SERSTREAM (the core result serialized to
  disk; issue 777's whole book converts, 1,549 s on main 4a5aae128b2 against lualatex's 3,523 s for two passes). Main stays the release line until 0.8.0
  final; the reviewed post-release branches (COPYROUTE, ENCURL) are on `int/towards_v0_8_1`. The ar5iv sync of the run's bundles (every month, then a cache flush) is running.
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

  **The worklist** (437 items on 2026-11-26 on `int/towards_v0_8_1`: 437 open, weight 957, 40 without a weight — counted as the `  - ID` bullets from here to "Silent-loss inventory", a trailing `; N` the weight; the arXiv long tail — items under 200 articles or synthetic-only — is in [`LONG_TAIL.md`](LONG_TAIL.md), v0.8.2+; id — task; weight — easy 1, medium 3, hard 5):


  - ODR6 — OD ruling 12 (recorded in OD #449): `dump_reader.rs`'s "add-only" docstrings (:365, :574) corrected, then a removal trial of the skip lists against CLAUDE.md rule 2; 1
  - ODR7 — OD ruling 6: each "Bindings beyond Perl's tree" body (end of OD) into its binding module's `//!` doc comment, then trim the catalog to its table — the 19 CLSFM bodies ODDOC-E moved there from #460 (#412 … #912) included; 3
  - ODR8 — OD docs: #8 (the OML slot-127 claim unconfirmed in code), #26 (mdframed now inserts through `insert_block`, `mdframed_sty.rs`); 1
  - ODR9 — wire `tools/perfect_kernel/od_anchors.sh` into `tools/lint.sh`; 1
  - DIGFATAL-CLOSE — (DIGFATAL residual; post-merge) a digestion Fatal now keeps every pending level (OD #251;
    68 of the 159 CorTeX Fatal papers keep their text), but the salvage splices an unfinished construct's boxes into the
    running paragraph: a footnote's words fuse with the sentence (`BeforeNote`), an unfinished `\author{…}` becomes a
    body paragraph, unfinished math becomes italic text, an unfinished list leaves rootless items (4 of the 68
    schema-invalid: 1402.0613, cond-mat9706088, 2512.12535, 2302.01695). Cause: `stomach.rs:2351`
    `salvage_pending_box_lists` flattens the levels without their owners. Plan: tag each level with its kind; keep
    body and group levels; close an argument level as its owner's container (a note, the frontmatter store), else a
    separate paragraph, never fused; math as an unparsed math list; a word still unplaced is counted by a Warning. REDs
    `rhai-bindings/digest_fatal_in_{an_unfinished_list,a_footnote,an_author,inline_math,display_math}_red.tex`; 2
  - DIGFATAL-EOF — the 81 arXiv `Mouth:EoF` Fatals, by Until delimiter: `\endsubheading` 10 (amsppt
    `\subheading\nofrills`; alg-geom9709005 now an empty `<document>`), `\NOT` 9, ` at ` 7, `\__prop_pair:wn` 6,
    `\LT@err` 5 (a preamble longtable patch: RED `package-bindings/longtable_patched_at_its_lt_err_red.tex`, 1409.6572,
    1503.02372), `\lx@jmlr@endaddr` 4; Perl recovers some with an Error where we Fatal (a run-off `\grab` in the
    preamble); a root-cause batch; 3
  - DIGFATAL-MUTE — after a Fatal the build's malformed-markup Errors are muted for a `Mouth:EoF` (the Until reader
    latches TooManyErrors) but printed for a binding's Fatal (E=2 in the list RED); one rule for both; 1
  - RHAISTRBODY — a Rhai `DefMacro` whose string body is unbalanced (`"{"`) is a Fatal here; Perl checks the balance
    only for a Tokens body (Expandable.pm:30-35); 1
  - OD83 — OD #83 (DG 2026-11-15): keep digesting acmart's `\Description`, but report an error inside it as a Warning (acmart.cls:895 discards the argument, so pdflatex never meets it; 2607.21760's `\D1 … \D5`; KNOWN_PERL_ERRORS #66 point 2) — post-cutoff; 1
  - PGF259 — OD #259 (DG 2026-11-15): one rule, pgf's `\divide`, for every native division by zero — `pgfmath_divisor` (pgfmath_code_tex.rs:208) returns the dividend as `pgfmath_divide` (:235) does, not EPSILON — post-cutoff code + guard rows; 1
  - PGF1007 — OD #1007 (DG 2026-11-15): follow pgf — an unparseable expression raises pgf's "Unknown function" Error and computes 0; `calc_compat_expand` (pgfmath_code_tex.rs:2247) retired — post-cutoff, with an A/B sizing the papers that lose a value; 3
  - DEFER — label forms deferred too (DG 2026-11-15): typeset where printed through `defer_print` (stomach.rs:2778, :2828), one rule for every deferred form (index, glossary, bibitem, label); retires #748's `\ensuremath` retry and the 2026-10-02 option A (#418) — post-cutoff, MED, with an A/B (#778, #845 shapes); 3
  - NATBIB — #123 (DG 2026-11-15): faithful to pdflatex's decision boundary — the citation mode decided ONCE, as early as our stages allow, and recorded as metadata (an XML attribute or a processing instruction) where post-processing needs it; replaces the per-cite post-pass (crossref.rs:2182 `make_bibcite`) — post-cutoff code target; 3
  - PANEL — #885 vs #138's `sole_graphic_width` (latex_constructs/mod.rs:5964) (DG 2026-11-15): wait for #193 — the entries stay separate as today's code; consolidate, retiring #529's and #339's workarounds, when #193 lands; 1
  - IOPEAD — old #808 kept as a target (DG 2026-11-15, ODDOC-D Q2): iopart's `\ead` goes to the author its `\address`'s mark names (RED `iopart_ead_under_its_address_mark_red`; #805's chain) — post-cutoff frontmatter work, by reach; Reach (census 2026-11-24): 101/30k ≈ 9393 papers; 3
  - CT1b — CT1b; 1
  - CT1d — CT1d; 1
  - HG2 — HG2: 182 DEL1-kept guards + a no-content-assertion fence; 3
  - RB1h — RB1's hyperref condition; 1
  - SMB1 — Separator meaning B1: engine snapshot; 3
  - SMB2 — Separator meaning B2: contrib helper; 3
  - SMB3 — Separator meaning B3: emitter `before` from the ink; 3
  - SR2 — SR2: a marked net note on the author block keeps its marks/markup or routes to the authors (SR1b open a; 2405.12817; pre-existing); 3
  - SR3 — SR3: the block note keeps _net on the creator route (floor + Warning; SR1 open c); 1
  - TP2 — TP2: title_and for pasj00/interact/l_aa/jmlr; Reach (census 2026-11-24): 36/30k ≈ 3348 papers; 3
  - ARAB — \arabic etc. on an undefined counter (DG ruling); 1
  - FOC1 — FOC1: 0-wide foreignObject (\rlap in a node) browser render check; CSS nowrap if text breaks (DTL1a Q7); 1
  - RAWS — On-disk raw-class dependency-scan RED; 1
  - RAWDEFR — raw `.sty`/`.cls` reading as the default (#910, OD #1080), papers whose output moved: 1502.07025
    (ut-thesis, a local stephenpack.sty, tikz/tikz-cd/ytableau) 123× `malformed:svg:g isn't allowed in <ltx:equation>`,
    21 unmatched `</svg:g>` and `{preliminary}` undefined, errors 7 → 147, though the output under Perl's loading was
    nearly empty (11 words → 37,878; RAWSVGEQ); 1401.8205 (copernicus.cls raw) loses its Received/Revised/Accepted dates
    (RAWCOPERNICUS); 0801.1806 (iaus.cls raw) warns `\end{document}` inside a group (level 1) and drops its `[Author
    (year)]` bibliography labels — check against the PDF (RAWIAUS); from the RAWDEF study: codedoc (a TL manual)
    Fatal:Mouth:Recursion, 2309.07511's ctex a new Error (the fandol fontset unavailable), circularglyphs' TikZ text on
    a path lost, 2312.09266's background.sty notice moved. Root-caused 2026-11-23: none of it reaches the fleet (`--preload=ar5iv.sty` already loads raw; rc2 ar5iv = default counts). RAWIAUS labels: faithful — iaus.cls:1221 `\@biblabel` is empty and the PDF prints none (drop it); its group warning and stray "n" are fixed by MTARG (v0.8.0, OD #265: the deposit and replay of an argument-taking `\@maketitle`; ~1,665 iaus.cls papers, 0801.1806 0 Errors / 4 Warnings). RAWCOPERNICUS: faithful too — copernicus.cls:493 discards `\received` etc. without the logo file, but FIDELITY_CHARTER §2 keeps authored text: a raw setter that discards its argument should hand it to the frontmatter under OmniBus's date roles (`title_page_check.rs` ~:440; HARD, ~411 copernicus papers). 1502.07025: a gain (11 → 37,878 words) whose 147 Errors are YTABMATRIX; 3
  - SVA — SV-A: \csname via \ifincsname; 3
  - QA05 — Kernel-version switch + \@ifundefined per year (QA05/QA06); 1
  - QA11 — the native pgf/l3fp inventory's rest (done 2026-11-14: `agent_reports/report_QA11.tsv`, probes `scratch_QA11/`; l3fp native 0/72k mismatches and pgfkeys 0/33 KEEP, raw pgfkeys +5.6…+16.9 %; pgfmath 34/90 differ from pdflatex, 29 Perl-origin; Root A — expandable closures run inside TeX's optional-space scan after a unit, tex.web:8869, 2606.14412's 55 false E, repros `scratch_QA11/rc/rc.tex`, `rc2.tex` — is QA11A, PR #872 merged): Root B, a dimen register above `\maxdimen` raises no "Dimension too large" (gullet.rs:4719-4734 skips tex.web:8870-8872's attach_sign check; `scratch_QA11/k1/k3.tex`; Perl the same; an Error-boundary batch, census first); DROP-FOR-RAW after an A/B: `\pgfsetdash` (pgfsys_latexml_def.rs:996), `\pgfmathsincos`, candidate `\pgfmath@smuggleone`; native pgfmath misses TeX's Errors on `123456789/1000000`, `-1e5/10`, `1/0` (OD #259), `"a"=="a"` — fold into OD #1012; 3
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
  - SR4 — SR4 (QA60 site 8): a title note held in \title (\titlenote) becomes a pubnote (1601.00073, 1805.02241, cs0405006) — with the page-furniture element (QA22/23); Reach (census 2026-11-24): 54/30k ≈ 5022 papers; 1
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
  - QA63 — XeTeX / LuaTeX emulation modules by the intended engine; 5
  - QA66 — Each Error's site marked in the output (`ltx:ERROR`), designed with BM; 3
  - QA73 — `ELIDEOP` renamed to an elision role, together with Perl; 1
  - QA77 — A Rust bst-driven BibTeX keeping each field's meaning (JATS, RDFa); 5
  - QA79 — LaTeXML 0.9 as the reference on its release: oracle re-baselined, parity re-run; 3
  - QA81 — A latexml-oxide home page (Sammy in the logo); 3
  - QA86 — A principled organization shared by both implementations, proposed to BM; 3
  - QA88 — Horizon: fully idiomatic Rust, the parity ties broken (DG's call); 1
  - P6TA — the token address readers (`line_addresses` at affiliations.rs:120/558/603, `run_in_author_order`, kind.rs's `prints_addresses_only` / `line_addresses`) onto the reading: give the emitter's per-item events to `\lx@frontmatter@itemkinded` (P6f's open item); 3
  - P6E2G — a guard row for P6e2's note path (Wiley `\corres{…\email{…}}` after two names: a raw `ltx:note` holding `\email`; 8 corpus witnesses, no synthetic row; revP6e2 finding 2); 1
  - GRPA — an unmarked name inside a mark-led row takes its group's affiliation by the declared convention (1405.2400; revMLN1, the charter's step 3); 1
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
  - BD — A `\DeclareBoldMathCommand` delimiter after `\left` (1205.4484, RED); Reach (census 2026-11-24): 3/30k ≈ 279 papers; 3
  - D1 — Rulings pending: `\obeylines` `^^M`, `{framed}` items, biblatex anchors, R7 port-or-drop; 1
  - T2 — Test API stage 2: 37 test files still spawn binaries; 3
  - SHIT — `\shortintertext` reverts as `\intertext` (mathtools `\let`; the macro-face design); 1
  - PSFR — psfrag replacements rendered into the figure (after PSF1's Warning); 5
  - PGRE — pgfmath's parse-failure Error (EB1, held by PGR1): fires on 3 pdflatex-clean papers by three mechanisms (1612.06541 tikz-cd anchors under its shipped pgf 3.0.0; 2006.16998 `\d` in `\pgfextra`; 2208.01735 collectcell + `\IfDecimal`); leftover text dropped silently until then; 3
  - P4bQ2 — `${\mathrm ^{1}}$` marks (erroneous TeX; pdflatex 4 `!` prints the 1): our recovery prints no superscript, so the API Cut finds no requested entry (RED `api_cut_erroneous_mathrm_marks_red`; cond-mat0206329) (P4b2); 1
  - DTLR — DTL1d's REDs: `\multiply` overflow keeps the register (`multiply_overflow_keeps_the_register_red`), pgfmath function results clamped silently (`pgfmath_function_overflow_clamped_silently_red`), acmart loads setspace outside `manuscript` (`acmart_setspace_only_in_manuscript_red`, 2409.07148; Perl the same) (DTL1d); 3
  - AUMK — answered marks hidden on both sides: the kernel `\author` whose own lines answer its names' marks (`kernel_author_lines_answered_marks_hidden_red`, as Perl), its authblk sibling (`authblk_author_line_answers_one_mark_red`), authblk's own `\author[1]`/`\affil[1]` generated marks dropped (`authblk_labelled_marks_dropped_red`, as Perl) (P4b2b, revP4b2b); llncs `\inst` names whose marks a macro prints hide the institute's marks too (`llncs_inst_names_macro_marked_institute_marks_hidden_red`; P4b4a: 1803.02579, 2201.04974, and CL166's seven: 0807.2091, hep-th0502229, 2412.12966, 0706.0112, 0803.2400, 0808.1862, 2508.08560's name marks); 3
  - P4CRN — P4cR1's residuals: a name's mark in a box with a space lost uncounted (`ieee_name_mark_in_spaced_box_lost_uncounted_red`), apa6 entries' marks dropped (`apa6_marked_list_entry_marks_dropped_red`), an unmatched e-mail address on a nameless creator when local parts run out of name order (`ieee_email_run_unmatched_address_nameless_red`, 2409.10792, 2410.09414; a ruling first), `unanswered_mark_lost`'s wording for a `ref:` mark (revP4cR1); 3
  - STRM2 — streamed-vs-eager differences under a forced switch (pre-existing; STRM): ids order (1705.02293, 2507.13358), whitespace (1005.3879, 2206.03430), footnote split (2010.06608), `toc` inlist (1907.01464), `\quad` reversion (2509.09797); and unforced: 2308.04512 streams under the production `--max-memory=8192` (PDFCMP 2026-11-15: RSS 2.09 → 0.97 GB, CPU 28.1 → 24.4 s) and its HTML differs from the eager run in 186 lines (proof ids S2.SS7.2 → S2.SS7.1, footnote class `ltx_role_footnotemark`, +26 W/E lines); the eager→streaming switch rate in production to measure; `finalize_context_font` DRY with `finalize_rec` (revSTRM); 5
  - OMNT — OmniBus/KOMA title notes step no counter and print arabic (2407.13730, 2606.19583 recalls "0" vs pdflatex ∗; `omnibus_title_footnotemark_prints_fnsymbol_red`; reach to measure) (FNR1); Reach (census 2026-11-24): 24/30k ≈ 2232 papers; 3
  - FNRR — the rehearsal reads a printing definition's later use (the true signal behind FNR1's gate; `author_note_macro_resets_an_existing_counter_red`; the replay writes a note from a printing call's expansion with its markup: 2604.20516) (FNR1, FNR1c); 3
  - IEMP — a minipage's optional argument leaks into IEEEtran affiliations ("t].44", 1901.06399; invented text; int162a too; reach to measure) (P4fC); 1
  - DTLX — DTL1d's counted pre-existing overflows (varwidth → VARW): a subscript-only math box of height 0 (2503.14426; Perl too; HARD), pgfplots `y expr` (2407.10017), xymatrix/tikz shapes (1409.3605, 0704.0778, 2409.07148, 2408.05806, 2411.12316), 1103.1294's cap (TooManyErrors sooner; nicefrac in xymatrix) (CL164); CX171 (`2026-11-13_CX171_report.md`): 2606.14412 (IEEEtran + circuitikz: 55 E, pdflatex 0 `!`; QA11A, PR #872 merged: 55 → 0; a `bending` start tip on a degenerate bezier — the native `\pgfmathreciprocal@` (pgfmath_code_tex.rs:2623) reads its divisor as 0 where TeX has 22.76184, pgflibrarycurvilinear.code.tex:98; 4-line repro scratch_CX171/r/m1.tex: Rust 64 E, Perl 4 E, pdflatex 0), 2605.31088 (pgfplots log axis `declare function` at small x, 4 E); 5
  - P4B4R — P4b4's post-merge parts (P4b4C): Group A's other readers (`kind.rs:111/237`, `frontmatter_api.rs:935`, `authblk_sty.rs:485`, `relocate.rs:1409/1449/1626`; DRY, no shape found), Group B (what the names request: `place.rs:478/259`, `ijcai.rs:533`, `old_reader/marks.rs:148`, `frontmatter_fields.rs:641` → `queued_author_marks`; byte-identity), Group D (REVTeX "after the comma", hep-ph0301204), Group E (`MARK_SWITCHES`/`leading_line_label` read typed numbers: a typed-label event), `events/lexer.rs:529`'s stale `requested_labels` (the rehearsal memo), P4b2b's rendering residuals (math-wrapped symbol marks, two symbols before a number reordered, 2509.08362's plain-text name); 5
  - VWLD — varwidth measurement residuals: lists/displays/`\hfill` inside a varwidth (`varwidth_lists_and_displays_measure_red`), a leading `\color` keeps the given width, an inline box beside text, `\fcolorbox`/`\colorbox` measures, inline math no-wrap in a `p{}` cell (2502.13537), cryptocode text cells ~7-10 % wide (VARW, VARWb; REDs `varwidth_display_draws_at_given_width_red`, `varwidth_colour_at_its_start_keeps_the_given_width_red`, `varwidth_inline_box_beside_text_measures_the_line_red`, `varwidth_cell_beside_overflowing_math_header_red`, `colour_boxes_measure_their_frame_red`, `cryptocode_text_cells_measure_wide_red`); 3
  - CMPJR — cmpj residuals: renamed copies (cmpj2e, cmp2em: 0903.4566, cond-mat0511115) stay raw with "?"; txfonts `\re@DeclareMathSymbol` undefined; `\refaddr{nokey}` prints a dangling `\ref` where pdflatex prints "?" (CMPJ1; REDs `cmpj_renamed_copy_refaddr_prints_question_marks_red`, `txfonts_re_declaremathsymbol_undefined_red`); the address list's residuals — a group across items loses the number and bold, counted (`cmpj_address_list_group_across_items_red`), a tagged `\item[a]` keeps the list whole, counted (`cmpj_address_list_tagged_item_kept_whole_red`), `\ukrainianpart`'s second title page's names hold the first page's addresses (`cmpj_ukrainianpart_addresses_of_the_second_title_red`, 1212.6367, pre-existing), an item copied onto several names repeats its `labels=` and the HTML links go to the last copy (CMPJ2/3, revCMPJ); Reach (census 2026-11-24): 12/30k ≈ 1116 papers; 3
  - P4LR — P4b4a's list-cut residuals: a `tabular`'s `\affmark` rows keep the base's owners, Ann all three lines (`llncs_institute_macro_marks_tabular_rows_red`); a box around part of a line leaves its comma and an empty `\mbox{}` (`llncs_institute_macro_marks_part_boxed_keeps_comma_red`, revP4b4a S1); `\textcolor{blue}{$^1$A, $^2$B}`'s second entry loses its colour (`llncs_institute_marks_in_textcolor_second_entry_color_red`, S2); 2608.21425's `nancyzhzhai@` e-mail loses its size (`llncs_institute_parbox_email_line_keeps_size_red`, the emitter's address cut); Reach (census 2026-11-24): 11/30k ≈ 1023 papers; 3
  - P4ENV — an environment's size (`\begin{footnotesize}`, 2506.05954) on no piece of the list it spans (`revtex_affiliation_footnotesize_env_size_lost_red`); 1
  - P4FNS — llncs.cls:391 `\let\footnotesize\small` missing from the binding (PDF 9pt, ours 80 %; Perl the same → KPE when fixed; `llncs_footnotesize_is_small_red`); 1
  - IEHREF — marked `\href{mailto:}` e-mails lose their link (`ieeetran_authorblocka_marked_href_emails_keep_links_red`, 2505.22454); 1
  - BSTN — `elsarticle-num` has no row in `lookup_bibstyle_params`: its list alphabetical where pdflatex lists by citation (`index-bib/elsarticle_num_lists_by_citation_red`; revGA3); 1
  - BLSN — biblatex `sorting=none` with no shipped `.bbl` lists alphabetically (`index-bib/biblatex_sorting_none_lists_by_citation_red`); 1
  - JURS — post's numeric gate: an author-year list under an unsorted `.bst` stays alphabetical (`index-bib/jurunsrt_lists_by_citation_red`; drop the gate); 1
  - BURL — `style_prints_urls`, post's last bibstyle-name list (fence pin 5): the engine writes the url fact on the node (GA3); 1
  - LBIB — the kernel `\bibitem` label path by `\NAT@ifcmd`: read whether `\@lbibitem` hands `#1` to `\bibcite` or runs it (`index-bib/lbibitem_parses_its_label_red`; GA5: acl2012/2015 refute the meaning test); 3
  - ACLB — acl's `\@lbibitem` prints a `[label]` at the entry where pdflatex prints none, Perl too (`index-bib/acl_lbibitem_prints_no_label_red`); 1
  - GA6 — cell padding by template spelling in five copies (`alignment.rs`, `normalize.rs`, `tex_tables.rs`; GA0 finding 8; `kernel-alignment/halign_hfil_template_pads_its_left_red`); 3
  - GEQB — (SUSPECTED) `tex_math.rs` EqnoTag predigest checks the stop set inside a braced tag where Perl reads it balanced (revGA5); 1
  - GA4N — `\@tabularcr`, `\@arraycr`, `\LT@tabularcr` one meaning (`\lx@alignment@newline`): `\noalign` through a macro (`noalign_tabularcr_through_a_macro_red`) and a kernel tabular's `\hline\noalign\@arraycr` (pdflatex 10 errors, ours 0: `noalign_arraycr_in_a_kernel_tabular_red`) — give each row end its own meaning, collcell's test a meaning family (GA4, revGA4); 3
  - GA4C — a longtable caption row's other content: keep it as its own row and skip the column template in the caption's `\multicolumn` cell (LT@mcol), replacing GA4b's text-comparison exclusion (`longtable_caption_then_a_nested_tabular_red`, `longtable_caption_takes_the_column_template_font_red`; 1903.09068, 1009.4206); 3
  - IEGNM — a co-author's name written inside `\IEEEmembership{…}` (2507.19818) sits in the grade's contact: make it its own creator (`ieee_membership_holding_a_coauthor_name_red`; CL167); 1
  - UPCS — `\uppercase` over the last group's `\IEEEmembership` keeps its lowercase; an earlier group's prints uppercase with a counted markup Warning (2402.07183; `ieee_membership_inside_uppercase_keeps_lowercase_red`; CL167); 1
  - PBLX — post's `biblatex`-prefix test in `make_bibliography.rs`, uncounted by F1 (a post-crate fence); the easy `\ver@` debts `dgruyter_sty.rs:27`, `njdnatbib_sty.rs:59`, `ws_rv_har_sty.rs:22` → `class_loaded`, and `\lx@record@provided` (a query's `\relax` stops the version being recorded) (F14); 1
  - UNCW — wordless `\thanks` notes rendered (keyed by their place, as a picture's note), after measuring their real reach (UNC: lexically ≤ ~1,500 arXiv, most expanding to words); 3
  - AUTC — hep-ph0301204 +0.60 % (a 1,000-name probe +4.6 %): the cost of emitting the gap marks AUT1 keeps — make a shown mark's emission cheaper (AUT1b); 1
  - GAPC — a shown gap mark loses its trailing comma (0704.2093; CL168); 1
  - GAPS — `$^{1,*,**}$` after a name shows `1,∗`, the `∗∗` (the corresponding-author note's) not shown (1507.03321; CL168); 1
  - TUZK — an overflow `&` grows a column (Perl Alignment.pm:137-143) where TeX §792 ends the row, the cascade an Error per extra `&`; `\@nolnerr` silent (SHARED; tuzuk; ROW0A); 3
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
  - SUPIT — a floating superscript rewritten into text `<sup>` (TeX_Math.pool.ltxml:527-550, every `word$^{2}$` and `Name$^{1}$` mark) takes the math default font: `<sup><text font="italic">2</text></sup>` where pdflatex prints CMR7 upright; Perl-shared (KNOWN_PERL_ERRORS #1235), OD #928's frontmatter residual (`floating_superscript_rewrite_digit_italic_red`; P4b4r Q2; since AUT1 ~30 Frontiers papers' held-name marks too, as their inline twin: `omnibus_author_math_mark_in_the_name_run_is_italic_red`); 1
  - TAILN — a name ending in a bare accent command (`A.~Pompo\v`, 0705.2788) falls to the plain-text path and loses its no-break space, counted (PMK1's `tail_call`; CL165); `tail_call` on an operand (`authblk_mark_macro_ending_in_let_operand_red`) (revPMK1); 1
  - NOTEX — a machine without TeX Live (OXIDIZED_DESIGN_DIVERGENCES #1095; CI `notex` + `tools/notex_smoke.sh`): the test suite is a TeX Live suite by design — without TeX it fails 2883/8468 (2026-11-19, docker `latexml-notex:check`), all on files only TeX Live has (hyperref's raw keyval/kvoptions/… in 989, expl3 220, revtex4-2.cls's `\@FMN@list` 175, listings' language files, tikz, babel); a visible skip policy is open if wanted. Rust-only: `\usepackage{array}` warns `missing_file:expl3` there (array_sty.rs's `\ExplSyntaxOn` blocks fire the expl3 autoload; Perl's binding has none). The kpathsea crate's own one-time stderr notice (0.3.4 lib.rs:433-434) stays: rust-kpathsea is a general-purpose crate, not latexml-oxide's (DG 2026-11-19).
  - LOADFMT — the strict-`LoadFormat` audit's live residual: a ~72-CS Perl-only long tail (the 58 "misc atomics" + the investigate bucket, minus the ported `\bib@*` family; [`archive/PERL_LOADFORMAT_AUDIT.md`](archive/PERL_LOADFORMAT_AUDIT.md)) — demand-driven, bounded by the corpus success gate, not a release blocker; 3
  - KPEDUP — KNOWN_PERL_ERRORS numbers 80-86 are used twice (`parity/KNOWN_PERL_ERRORS.md`: entries 80-86 once from `pdfcol.sty` (80) and again from `\define@cmdkey` (80)), a pre-existing registry defect `next_entry_numbers.sh` surfaced (#846's body): renumber the later set to free numbers and carry every inbound `KPE #8x` reference; 1
  - BEAMSLIDE — beamer's faithful Perl frame model (`ltx:slide` in `ltx:slidesequence`, Perl `beamer.cls.ltxml`) in place of the document-top frontmatter fallback for `\titlepage`/`\maketitle` in a frame (`beamer_cls.rs` `\titlepage`; P12's follow-up, `perfect_kernel/archive/PLANS_DONE_PHASE56_2026-09-27.md`; witnesses beamerthemecelestia demos, beamerthemeNord, beamertheme-simpleplus); 5
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
  - FONT — Font-selection chain residuals 1-17; 5
  - FRAG — Fragid parity open items 1-4; 3
  - R6 — `Collector::rescan` for generated backmatter; 3
  - R7 — Presentation-MathML F5 linebreaker (port or drop); 5
  - R8b — forest.sty drawing and layout fidelity; 5
  - R3T — tlp.cls `\maketitle` leaves a group open (2606.10056; R3's residual); Reach (census 2026-11-24): 5/30k ≈ 465 papers; 1
  - 0c — Noticed in the perf work: `source3` output depends on RSS, `SASnRdisplay`, pgfmath's f64 functions, a Timeout's 39-byte XML; 1
  - S2610 — 2610.06709: the `\footnotetext` dagger, sideways scroll at ≤ 700px, a `<p>` in a transformed box's `<span>`; 1
  - OLDRES — Residuals of closed rows: R1's clusters and witnesses, equation layout (#334), cropping (#337); 1
  - AR5IV — ar5iv tracker residuals: the close-out list, #472/#556 Rust-only, the unclassified four; 3
  - RBXA — booktabs rules add no spacing to the alignment height (Perl the same, booktabs.sty.ltxml:23-39): resized tables 62 % > 10 % short (RED `booktabs_rules_add_their_spacing_to_the_table_height_red`; `2026-11-13_RBX0_report.md`); 2
  - STARTTOC — the kernel `\@starttoc` loses any raw list (Perl the same; 29 TL manuals): emit an untitled `<TOC lists=ext>`, audit its callers, offer to Perl (`2026-11-13_TOC0_report.md`); 3
  - ABLK — authblk `\author[ ]` (a blank mark) given an affiliation the PDF does not print (RED `authblk_blank_mark_author_takes_no_affiliation_red`, 2605.24212; SBX1); 1
  - EMIT — 0906.2789's `{\em …}` affiliation loses its italic, uncounted (GRP1/LOGO1 lateral; CL171a); 1
  - QA75M — QA75S's minor patterns (`2026-11-13_QA75S_report.md`, gallery scratch_QA75S/gallery/): code listings one paragraph per line (the tabularray, thmtools, nicematrix docs), ACM boilerplate above the authors (2605.07722, 2403.05668), stray † marks and the running head in the body (2311.07985, 2407.08541), achemso citations glued to the word before (2606.06771), biblatex.tex's title lacks "Version 3.21"; 3
  - REPOTRIM — the repository's local-only files out of the tree (DG 2026-11-13: after the merge; "keep all repro files in the repository … The process and session history is likely not useful and can be local-only"): process and session docs (`docs/archive` session logs, `docs/perfect_kernel` archive and worklists, dated audits, ≈22.2k lines), `bin/ab_audit` + `bin/binding_audit` (5.7k), `tools/perfect_kernel` scripts and tsv with local paths (3.9k), `docs/parity/*_AT_EMIT2.tsv` (0.8k) → out of tree on this machine; every repro and NOTES.md stays (`2026-11-13_REPO0_report.md`); 3
  - GUARDC — the guards as table rows with one shared helper set (`cluster_package_guards` 177k lines, ~32 per test; `convert_clean` ×48, `convert_counted` ×32): −40..−60k lines and CI time (the visible skip done in #826: `util::test::Requirements`, one `skip()`; `2026-11-13_REPO0_report.md`); 3
  - CORERES — latexml_core residue (`2026-11-13_CORE0_report.md`): uncalled `dump_frame_bindings` (stomach.rs), `merge_selected_font` (content.rs; sect13.rs:2568's stale comment), `memory_fatal_seen` (watchdog.rs; check MEMORY_FATAL_SEEN's reader first); cache `LXML_TRACE_NODE_BOXES` / `LATEXML_EXPL_TRACE`; arena.rs `first_bytes` aliasing → through `with`; one frame-id counter (`NEXT_FRAME_ID` / `FRAME_SERIAL`); 2
  - CMTLEN — comment length: 4.8k lines of latexml_core sit in 6+-line comment blocks (BM's "short comments"); the narrative to OD entries, every witness id kept (`2026-11-13_CORE0_report.md`); 2
  - MATHAB — A/B analysis: skip parsed-math-only changes when the `tex=` attributes are equal — the math parser's limits are wall-clock, 25 of int171a's 81 changed papers flipped under load (`2026-11-13_CL171b_report.md`); 1

  - WIN1R — WIN1's post-merge rows (WIN1 done in #833: the suite green on TL 2025, the distro TL and TL 2026; its TL 2026 install recipe and resolution in `archive/SYNC_SESSIONS_2026-11.md` "WIN1"): `\pdftexrevision` is Perl's 19 where TL 2025 runs 1.40.28, so latex.ltx takes the `\everyjob` glyphtounicode branch (needs a full A/B); an undefined `\if…` recovery swallows the rest (`parameter-conditional/undefined_if_name_swallows_the_rest_red`); biblatex's `\blx@blxinit` hooks never run (sbl's `\lno` …); `\Umathcode` defined under the pdfTeX persona; asmejour's `\arabic*` "Missing number" our Warning, TeX's Error; a fill glue at a box's end prints a space (`boxes-groups/fill_glue_at_box_end_prints_a_space_red`); csvsimple 3.0's l3 code in an siunitx column (`package-bindings/csvsimple_l3/csvsimple_l3_siunitx_column_red`); keytheorems custom `thmstyle` → `\newtheoremstyle`; Verbatim `numbers=left` trailing spaces; `latexml_tests_internal` ignores `golden_variant`; the circuitikz predicate could key on `transistors/rounded caps`; 3
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
  - REGW — the core register lookup could Warn on an undefined CS once the kernel's optional-register probes use the quiet lookup (a Warning there added geometry `\Gm@tw` Warnings to 27 of 200 papers; #843, OD #1036); 1
  - VERBR — verbatim CSS residuals (#844): a `numbers=left` line number overlaps the code (`listings-algorithms/fancyvrb_numbers_left_overlap_the_code_red`); a breaklines first line indented twice (fvextra's negative `\hspace` missing from the HTML); a breaklines Verbatim in a minipage in a `tabular` widens the page (pre-existing); `display: contents` on the id-bearing wrapper may remove an `#id` link's scroll target (suspected, unprobed); 2605.00468 widens a phone page through a fixed-width tcolorbox SVG; an xleftmargin Verbatim in a minipage or p-cell loses the margin from its second line on (#855, issue 853; I853); lstlisting's xleftmargin not rendered (Perl parity); 3
  - NICEO — nicematrix: 2206.12224's second trailing key list (`package-bindings/nicematrix_second_trailing_options_red`); NiceArray's `tex=` reverts to `\left.\begin{array}…\right.` and drops `\Block` (pre-existing) (#839, issue 736); 1
  - MSPC — math spacing (#841; OD #1032, #1033, both to offer upstream): `\mathbin{/}` and `\bigm/` lose their class in digestion (tex_math.rs:1194 → :2694), so no space where TeX gives medium or thick (`math-parse/mathbin_slash_keeps_binary_space_red`); the spacing walk (presentation.rs `adjust_pair`) ignores script style, `x^{a\mathrel{\star}b}` 0.278 em where TeX has none (`math-parse/script_style_rel_has_no_thick_space_red`); 3
  - PRESC — prescripts (#837, issue 824): a trailing float script after a prescripted factor, a prescripted bare OPFUNCTION (`scripted_factor_l11`), `{}^*x` (`math-parse/{trailing_float_script_after_a_prescripted_factor,prescript_on_a_bare_opfunction,prescript_star_on_a_bare_identifier}_red`); 3
  - CAPJ — caption justification (#834, issue 787, OD #1024): sub-captions (`captions-floats/subcaption_justification_and_skip_red`); `centerlast`/`centerfirst`; a justification through a user macro or raw glue — read the digested body, not its spelling (`caption_justification_through_a_macro_red`, `caption_justification_by_glue_red`); a short multi-line caption under singlelinecheck (`caption_singlelinecheck_short_multiline_red`); 3
  - LWID — `\linewidth` residuals (#836, issue 778, OD #42): a tabularx `X` cell's width, which needs the X width computed (`kernel-alignment/tabularx_x_cell_linewidth_red`); `\linewidth` inside list items (`lists/list_item_linewidth_red`); 3
  - TXWD — class text widths (#830): the standard classes' and elsarticle's `twocolumn`, REVTeX's size options (`class-bindings/{standard_class_twocolumn,elsarticle_twocolumn,revtex_size_option}_textwidth_red`); 3
  - ETOCR — etoc (#830, issue 760): relative depth (`package-bindings/etoc_relative_depth_red`), `\etocsettocstyle` headings (`etoc_settocstyle_heading_red`); 1
  - IMPFC — import: a filecontents file under an import directory (`package-bindings/import_filecontents_file_under_an_import_dir_red`; #830, issue 772); 1
  - EPUBR — EPUB residuals (#840, issue 807; Perl the same): a document without sections has no navigation document, which epubcheck rejects (`epub/no_sections_no_nav_red`); CI does not run epubcheck yet; 1
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
  - LOSTFR — LOSTF's residuals (#866): the pazh/astl `*` mark written as an e-mail contact "*" (`sectioning-frontmatter/mark_then_store_email_read_by_a_refused_abstract_red`; the observer reads the rerouted setter as the class's `\email`; reach ~880); astl address notes all go to the block's last name instead of following the marks (1206.0451 Khabibullin's "1" unanswered); `author_and_on_behalf_of_teams_red` (1002.4968), `rmaa_altaffiltext_marks_red`, `ptephy_author_given_twice_keeps_the_last_red`; `class-bindings/vmsta/vmsta_first_name_without_surname_red.tex`'s engines line is stale; 3
  - AMSPR — amsppt's heads (#868; AMSPPT, revAMSPPT): a bare `\ref` invents a "References" title amsppt never prints (math0206022; `amstex/amsppt_bare_ref_invents_references_red`; Perl and base too); alg-geom9502009 nests 49 theorems where Perl nests 19 (base the same; not minimized); `\proclaim\nofrills{…}` titles "\nofrills" (alg-geom9502009, pre-existing); no `\add@missing` (amsppt.sty:574-576, :863): `amstex/amsppt_roster_without_endroster_before_head_red.tex` (2 E vs TeX's 1, section 2 inside an item), `amsppt_head_inside_proclaim_red.tex` (0 E vs TeX's 2); NIT: five theorem `DefConstructor!`s → one loop; Reach (census 2026-11-24): 46/30k ≈ 4278 papers; 3
  - RHODRRES — (RHODR re-review, post-merge) the end-of-digestion stash check covers `digest_internal` only
    (`core_interface.rs:2153-2157`): its callers then differ — `converter.rs:981-996` salvages, `bib_session.rs:300-302`
    loses the bibliography with `Error:bibliography:convert`, `lsp_server/unix.rs:643` logs and falls back — and the
    CLI's `digest_adaptive` (`:1984-2083`) has no end check (the build's first `check_timeout` raises it); one rule for
    both loops. Streaming does not end the rewrites: `apply_rewrite_rules` returns `Ok` per fragment, so later fragments
    and the spine still run every rule (latch a flag, or narrow OD #1028 to eager mode). A dropped stash should not be
    marked printed (`:631`, `:1786` use `take_stashed_fatal`; use `set_aside_stashed_fatal`). A Rhai math matcher
    returning only `name`/`role` drops the merged token's `meaning` (Perl keeps `meaning="equals"` on `≔`;
    `MathLigatureOptions` cannot tell absent from undef) — keep ours or copy Perl? After a getter's Fatal the step goes
    on, so twoget prints 2 `Fatal:` lines where Perl prints 1; 2
  - ODDB — docs and dead code (ODDOC-B, ODDOC-B2, revQA1985): `tools/fontmap_drift.py:56-67`'s slot-94/126 "Bruce's accents" allowlist is dead (61c made them ASCII) — remove; stale comments at `float_description.rs:84` ("read Undigested"), `latexml_math_parser/src/grammar/builder.rs:2123` ((∂_t∇)·u) and `latexml_core/src/binding/content.rs:1089-1098` (says `.ltxml_loaded` is not set, then sets it); constructor `font='#x'` pairs ignored for the whatsit's `font` property (constructable.rs:214, replacement.rs:796; Perl Document.pm:1834 applies them; probe glossaries/amsrefs `font='#titlefont'`); `next_entry_numbers.sh` cannot see `kit/od_reservations.txt` (#1051) and counts MATH in the OD pool though MATH runs its own sequence (#63); for the OD PRs: OD #887, OD:5004, :21249 and :21538 cite `arrange_panels_and_breaks` at the stale Perl range 3229-3349 (now L3236-3356); 1
  - ODDC — ODDOC-C's rest (revODDOCC): STANDIN members 592 597 605 614 734 735 741 986 and #731's ~500-line body not merged; #216's closed-form `\pgfmathpointintersectionoflineandarc` is a WORKAROUND — principled: TeX-exact trig, so pgf's bisection terminates (zx-calculus, 2201.09268); 13 merged entries carry no "ex-#N" marker on the bullet holding their text (#20 40 41 87 310 323 338 340 516 1035; #732/733/736/744 named in text) — code cites of #87 (bibunits_sty.rs:125, 06_cluster_bibliography.rs:502) and #41 (cluster_xslt_split.rs:149) reach the primary but no labelled bullet; the LOCK/REPLAY one-rule restatement (ruling 8, state.rs:5715-5824) for an ODDOC-E docs pass; 1
  - ODDD — ODDOC-D's code residuals, after DG's Q1-Q5: one Kind pass + one Link chain (#805/#954), K11 → #767 (ruling 9) (the spelling splits under #993 stay: DG 2026-11-15, ODDOC-D Q1, generic binding utilities); 5

  *Folded 2026-11-15 (PERFFOLD): the perf pass's residuals (int/perf, rust-libxml 0.3.24, libmarpa-asf-sys 0.3.1, PDFCMP):*

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
  - RLXF — rust-libxml after 0.3.24 (`~/git/rust-libxml`): a rust-owned node still HELD when its document drops is a use-after-free (documented rule kept, LIBXML24b), and cached unlinked wrappers freed at drop (review B (6), deferred); the documented 120-byte LSan leak (unlinked-node case); `holds_property`'s guarded first-property read could go unguarded (sanity check: the guards cost `walk_node` +0.82/+0.86 %); 14 `cargo doc` warnings in files 0.3.24 did not edit (LIBXML24 open (5), recheck). 0.3.25/0.3.26 (2026-11-23,
    now required) say they fix the first two (a document's drop neutralises its wrappers and frees a rust-owned node;
    an unlinked-and-relinked node keeps its wrapper): re-run the LIBXML24b repro before closing them; 3
  - GENMARK — (post-run; DG 2026-11-17: "the square bracket fencing in [a] … may look unpleasant to a reader. Would it be easier to compute this in a binding and annotate this as an attribute, so that XSLT knows what value to pick?") Move mark generation upstream: one engine function in the frontmatter insert pass collects every provided mark (today's XSLT the `printed` node set of `f:generated-mark` (formerly the key the `printed` node set of `f:generated-mark`; since XSLTKEY it holds only what a bracketed-letter lookup can hit — a lookup without brackets must bring back the legend-opening and bracket-stripped alternatives of e130fb1c3f4) set: note marks, tags, superscripts, legend openings) and picks the first glyph of a pleasant series no provided mark uses (`∗ † ‡ § ¶ ‖`, then doubled, letters last); a binding may DECLARE its class's series (llncs ⋆, the document's `\@fnsymbol`) — micro declares, global computes; written as the shared note's `mark` with a generated flag (`class="ltx_mark_generated"`), the XSLT rendering it verbatim; PAGEGEN's fallback reuses it. Hairy: the engine must see legends in body/\date/tables; `mark` then holds provided and generated values; 5
  - RATCHETS-1117 — (DG 2026-11-17: "Accept raises, row R to lower"; and "I am not even sure if we have meaningful refactors towards managing digested data, given that your work found it natural to work on the token level. It is something to consider and rationalize, but possibly after the release") Three release branches raised the token-read/spelling ratchets: MTREDUCEE `latex_constructs/mod.rs` spellings 199 → 219 and two state keys (`lx_title_marks_read_closed`/`_declared`), DATELEG `frontmatter (rest)` reads 37 → 52 (the `\@date` store), IEEEROUTE `ieeetran_cls.rs` reads 104 → 125 (the `\thanks` sentences). After the run: classify every token-read site as necessary (the datum exists only as tokens: a macro's stored body, an undigested store) or avoidable (a decision on spelling where TeX reads a meaning, or a digested/event view exists), lower the avoidable ones, and reword the ratchets to tell the two apart.
  - PUBNOTEMARK — (DG 2026-11-17, OD #1069) a title note carries no `mark`: `.ltx_pubnotes` hides every title note behind one handle (ⓘ since #1069; Perl a CSS †), where the PDF prints each note's own mark (1612.03574's title ∗, 2605.10496's affiliation-line ‡ whose `\thanks` lands in the title's notes) or, unmarked, its text (#1063). Record the printed mark on `ltx:pubnote` as on a creator's note, render it as the note's handle (`ltx_note_mark`), an unmarked one in the open; then drop the `.ltx_pubnotes:before` handle in LaTeXML.css and ar5iv-css. Guard to flip: `xslt::witnessed_css_delta::title_pubnote_content_stays_collapsed`.
  - ACHEMSO-CALLS — the API's `annotate=` counts creators, not author calls: an `\alsoaffiliation` (or an `\email`) after `\author{Ann Able and Bob Baker}` reaches the last name only (RED row `fmmarks_achemso_alsoaffiliation_after_a_two_name_author_red`; 2209.04586, 1810.01505, 2008.03381 show it for the e-mail); an `annotate=` by author call; 3
  - PAGEGEN — the page's fallback glyph for a markless body note (`LaTeXML-meta-xhtml.xsl` note-mark, Perl's `†`, :429) is a mark the conversion generates; DG 2026-11-17: generated marks never equal a provided mark (author-, class- or package-printed). Where the document prints a provided `†`, the fallback must take `f:generated-mark`'s unique glyph instead (one check, the `printed` node set of `f:generated-mark`); after CONNFIX lands (same XSLT); red/green rows; 3
  - DEPS080 — before v0.8.0 (DG 2026-11-17): study `agent_reports/2026-11-18_DEPS080_dependency_review.md` §4-5. DONE (chore/deps080): A2 deny.toml notes (the distribution binary ships rhai/smartstring; chrono is direct and via pericortex), A4 `once_cell` → `std::sync::LazyLock` (direct dep dropped from 5 manifests). LEFT: A1 marpa fork on `bindgen 0.73` or pre-generated bindings, publish libmarpa-asf-sys 0.3.2 (−4-5 build crates); A3 refresh the fleet checkout's stale lock at the next fleet rebuild (clears the libmarpa-asf-sys license-exception warning); A5 clap `default-features = false` (~6 crates; CLI help loses colour and "did you mean" — needs DG); A6 size levers in our code (`build_model` as data, DEP-15 outlining of the top `load_definitions`); CHRONO after v0.8.0; 3
  - XSLTKEYc — (DG 2026-11-17: "Accept, row R") the author-note XSLT still costs 2405.13379 +1.00 % and 1706.03762 +0.59 % instructions vs main (PERFAUDIT basis: REGFIX vs the #895 head, html5 production recipe; 38/40 docs within +0.5 %, median −0.08 %, p90 +0.46 %): the note templates' own work and the per-note `use` of `f:creator-note`, chiefly kind O's scan of a note's text for its opening glyph (about half of 2405.13379's excess is outside the stylesheets). Options: compute a note's opening glyph once when the XML is built (an attribute), or look it up only within the note's own block. Also `f:glyph-elsewhere`'s third key makes 8 walks under the root's non-excluded children (~2M on 1706.03762): lazy like the `printed` node set, since it is read only when a shared mark's glyph may print elsewhere. WISDOM #105 has the method.
  - IEEEROUTE-OWNERCHECK — (revIEEEROUTEb) no post-placement check that each routed IEEE contact sits on its creator (correct by construction today); witness 2106.02811.
  - AUTOREFBACK — (HYPERPROV, OD #588) under the ar5iv first-use provide, an `\autoref` to a target numbered before the
    paper's first `\autoref` shows the target's title, or its enclosing unit's (an equation reads as its section's title),
    not hyperref's "Equation 1": 2 of 74 autorefs on 4 witnesses (0803.2070 `#Ch0.E1`, 1010.2520 `#S2.SS1`). Options: an
    eager internal tag formatter at preload (an autoref tag on every target of every paper) or a post-processor fallback
    to `typerefnum`. Post-release; 3
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
  - BABELINPUT — (LTXOWN review, pre-existing) `\input babel.def` in a LaTeX document with no copy raises 9 Errors
    (`\bbl@trace`, `\bbl@ifshorthand`, `\bbl@bidimode` …) where pdflatex has 0; and aa.cls with
    `\usepackage[english]{babel}` (1004.1914) gets no document `xml:lang` where a plain article gets `en`; 1
  - HYPREFSTEPRES — (HYPREFSTEP review, pre-existing) a `\label` after a bare or typed `\refstepcounter{myc}` under
    cleveref attaches to the last labelled object ("lemma 1", pdflatex "mything 1"/"mything 2";
    `package-bindings/cleveref_label_after_bare_refstepcounter_red.tex`); our `hyperref/fixcleveref` plug wraps only
    `\cref@old@refstepcounter`, where hyperref's sits on the kernel's `refstepcounter` socket and also covers
    `\@kernel@refstepcounter` callers (`\@thm`, latex.ltx:17197; `\H@refstepcounter`), hidden today by the cleveref
    binding's native label typing; 2
  - CREFOLDPLUG — (ruling needed) our plug sets the label data for every cleveref copy, where pdflatex's first aid works
    only for v0.21.4 and stops on an undefined `\firstaid@cref@updatelabeldata` for any other
    (latex2e-first-aid-for-external-files.ltx:194, 238-247; v0.19 with hyperref first: pdflatex 2 `!`, ours 0 Errors; OD
    #1100): an unrecorded surpass under the "Error where TeX errs" rule; 1
  - CREFPREFIX — (perf) cleveref's own `\refstepcounter` costs ~1.1M instructions per step, ~89 % of it in
    `\cref@constructprefix` (1811.08194 +7.3 % since HYPREFSTEP, which runs it as pdflatex does; DG 2026-11-25: the cost
    is accepted for the correctness): a native `\cref@constructprefix` in `cleveref_sty.rs`, which every cleveref
    document pays; 2
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
  - TITLESECSTAR — starred (numberless) titlesec headings drop the before-code: titlesec applies `#4{#8}` to numberless
    titles too, our unnumbered path runs only the font and the after-code (`sect04.rs` ~423-431): pdflatex `[Starred D]`
    / `Star.`, ours `Starred D` / `Star` (brackets, a period, `\MakeUppercase` lost; synthetic); 1
  - ALGOVINTAGE — old shipped algorithm2e copies (#920; synthetic unless named): release 1.16's `ruled` test reads
    `\algostyle` at load (`algorithm2e_sty.rs` ~359-362), so a later `\restylealgo{ruled}` closes ": " where pdflatex
    prints "Algorithm 1 …" (decide at caption time); a 2003/10/10 copy under `french` prints `##1:` without
    `\algocf@typo`, ours " : "; old copies' `\ifalgocf@procnumbered` / "Boolean algocf@func undefined" (0810.3203,
    0911.2801); prooftrees' bare `\forest … \endforest` is still a stub, its trees not drawn, one counted Warning
    (2606.20837, 2412.10588; a forest batch); 3
  - CLUSTFIXR — run 397's Error clusters' leftovers (#921-#923): `\documentstyle[PASJadd]{PASJ95}`'s `\draft`
    (PASJadd.sty:46; astro-ph0003107, astro-ph0002344; ~10 census papers): a small PASJadd binding or the shipped files
    read raw; pasj00's `\Published{}` (pasj00.cls:113; 0712.0877, astro-ph0701328) → `\lx@add@date[role=published]` with
    its own A/B; the `\@part` lock (`latex_constructs_rust_only.rs:146`, for KOMA) refuses jpp.sty:608-609's
    `\gdef\@part{0}`, a stored number (astro-ph0207068; ATPARTLOCK); astro-ph0107268 picks the dvips l3 backend where a
    minimal psfig file picks pdftex (DVIPSBACKEND; bisect); an old aipproc `\author{A, B, and C}` with many names gives
    `merged_creators` (astro-ph0011502, astro-ph0604126; AIPPROC-MERGED, frontmatter splitting); `\introductionname`
    undefined in 79 papers — copernicus.cls:915-917 adds `english` to `\@classoptionslist` and TL 2025's babel loads
    nil.ldf (COPERNICUSINTRO; check a 5-line `\documentclass{copernicus}` + `\introduction`); 3
  - VERBARGR — `\verb` in arguments after #922/#924: where a binding pushes an argument back without its braces
    (`\multicolumn`, `\textbf`) the verb scan still runs to the line's end — `\multicolumn{2}{l}{x \verb+a+ y}\\ c & d`
    (an extra `&`), `Pre \textbf{\verb|x| y} post.` (4 Errors, text moved); astro-ph0108286 keeps its text: end the verb
    where the pushback runs out, or push it back with braces (VERBARG-NOBRACE); `\mbox{\verb|m| after}` pdflatex 1
    Error, ours 0 (VERB-MBOX); a `\verb` in a bigfoot minipage footnote pdflatex 5 `!`, ours 0, the leniency of
    fancyvrb's `\VerbatimFootnotes` (BIGFOOTMINIPAGE; defer); 3
  - PERPAGE — (from MANYFOOT) `\usepackage{perpage}\MakePerPage{footnote}` warns "`\c@pchk@footnote` is not a register"
    at every step: perpage.sty:68-75 makes `\c@pchk@<ctr>` a macro that LaTeX's `\@stpelt` runs, and our `\stepcounter`
    resets by register (Perl's `ResetCounter` alike). (a) a perpage binding with `\MakePerPage`/`\MakeSorted` as counter
    no-ops, as footmisc's `perpage` option; or (b) a kernel reset that runs a macro `\c@` the way `\@stpelt` does. Repro
    `perpage_make_per_page_reset_red.tex`; census 16 perpage, 14 `\MakePerPage` per 30k; 1
  - MFTITLE — (from MANYFOOT) a series note in the title block: `\title{T\footnoteB{x}}` becomes a `<pubnote>`, so the
    series counter does not step and the body's series marks run one short (pdflatex 1, then 2); a series note in
    `\author` becomes a `\thanks` and shifts the article's own marks (Bob's `\thanks` "‡" where pdflatex prints "†").
    The frontmatter honours `\@mpfn`, or series notes get a name it treats as a note but not the title's own. Repro
    `manyfoot_series_note_in_title_block_red.tex`; 1
  - MFNUMERAL — (from MANYFOOT; NCCNUMERAL) a given numeral mark is printed through the main `\thefootnote`
    (`numbered_footnote_mark`, `latex_constructs/mod.rs`): under `\fnsymbol` a series text marked 1 shows "∗" and
    nccfoots' `\Footnotetext{5}` shows "¶". It needs a kernel route for "a mark given, not a number", and for a series
    whether the class's `\@makefntext` prints `\@makefnmark` (2609.01974: nimeart.cls:2209 `\let\@makefntext\noindent`;
    a first attempt invented a "0" mark there). Repros `manyfoot_series_numeral_marks_under_fnsymbol_red.tex`,
    `nccfoots_given_numeral_mark_under_fnsymbol_red.tex`; 1
  - MFMATHMARK — a footnote text whose mark is set in math keeps the mark's TeX source as `mark` and gets no `<tags>`
    (`mark="$\dagger$"`, the page shows "$\dagger$" where pdflatex shows "†"); kernel-wide and Perl alike
    (`\renewcommand\thefootnote{$\dagger$}`, nccfoots' `\Footnote{$\dagger$}`). (a) deluxetable's `shown_mark` in
    `make_note_tags` (a Perl divergence, KPE entry) or (b) `<tags>` for `\lx@notetext` built from its mark. Repro
    `footnotetext_math_mark_shows_its_source_red.tex`; 1
  - MFTOCSTEP — (MANYFOOT2 review, pre-existing) a series `\footnotemark<X>` in a `\caption` or section title steps the
    series counter twice: `\lx@toc@copy@neutralize` neutralizes only the kernel's note constructors, so manyfoot's
    `\MFL@makemark` `\stepcounter` runs again in the toc copy ("Cap2" over a toccaption "Cap1"; since MANYFOOT the next
    `\footnotetextA` is marked "2"); pdflatex "Figure 1: Cap1", "1 SERIES-CAP". Any package mark macro that steps a
    counter in a moving argument; needs a `_red.tex`; 1
  - MFMARKSUP — (MANYFOOT2 review, base alike) a series `\footnotemark<X>` prints a plain baseline "1" (the kernel's
    `\@footnotemark`) where the PDF sets a superscript, and nothing links it to its series text; 1
  - FMTSIGNAL — after a first-run format build, a later SIGTERM/SIGHUP still exits 130, not the true signal status:
    re-exec after the build, or build in a child process (Gemini's CLIHARD review, GEMINI_ARCHIVE_2026-11-25.md L367); 1
  - XHTMLEXT — `--format=xhtml` without `--dest` gives `out_extension` "xml", not "xhtml" (`post.rs:1225-1234`; Perl
    `Config.pm` L408-420; GEMINI_ARCHIVE_2026-11-25.md L316, L347-348); 1
  - LSTDISPLAYINIT — (LISTINLINE) display listings (`lstlisting`, `\lstinputlisting`, `\lstnewenvironment`) never run
    listings.sty's `\lst@Init`/`\lst@DeInit`: `\g@addto@macro\lst@DeInit{!}` then `\begin{lstlisting}x\end{lstlisting}`,
    pdflatex prints "!" after the listing, we drop it; touches every display listing (own A/B); no arXiv witness; 3
  - NULLDELIMPOS (after NULLDELIM, pre-existing since #373) — the fence retry (`balance_null_delimiters`) puts the
    stand-in for TeX's null delimiter at the formula's edge, not where the `.` stood, so `w\left.x\right)` reads
    `delimited-)@(w*x)` (the meaning spans `w`) and `\{x\mid x\left(y\right.\}` becomes `{ x | x ( y } )`, which
    fails to parse; when the retry still fails, its inserted nodes are unlinked but not freed (one small leak per such
    formula). Fix: place the stand-in at the null delimiter's `XMHint`; 3
  - EMPTYFENCE (low) — 1,269 empty, id-less OPEN/CLOSE tokens remain in a 635-paper sample from another mechanism (an
    absolute value inside `split`, 0803.0204; an aligned dual's content branch, `tests/parse/opfunction_arguments.xml`
    :1069 `|z_i|`); none is drawn (content side only). Find their source and drop or fill them; 1
  - PLACEACK — `\acknowledgments` inside an open float keeps its Error (cond-mat0108156); the float-out of OD #1099 in the
    acknowledgements constructor would fix it; 1
  - PLACET3 — an abstract that swallows the main matter (math0006149's unbalanced `\abstract{`, 1108.5171's brace-less
    `\abstract` with no heading, hep-th0110297) keeps its theorems as `ltx:theorem`, each with its "isn't allowed in
    <ltx:abstract>" Error (OD #1097, DG 2026-11-20); ABSFIX (in progress) ends such an abstract after its paragraph
    (DG ruling), which moves those theorems to the body. Repros `block-model/theorem_in_unended_abstract{,_group}`; 3
  - ABSOWNDEF — a document's own `\long\def\abstract{…}` is refused by the kernel lock (as Perl,
    latex_constructs.pool.ltxml:1150), so 1209.6193's abstract swallows the body; DG 2026-11-20: keep the lock; 1
  - REFMISSINGTEXT — an unresolved `\ref` renders an empty `ltx_missing_label` span ("See Theorem ."), where Perl prints
    the label text (Post/CrossRef.pm:346-348; ours `latexml_post/src/crossref.rs:1907-1910` skips it); PLACEFIX's
    dropped theorem labels add such refs (0801.2274, math0509535, math0511748, math0605416); 1
  - ABS-ENV-OPEN — a `\begin{abstract}` never ended keeps the whole paper with its theorems and its Error
    (cond-mat9410055); net B covers command forms only; 3
  - UNSAFE937 — after v0.8.0 (DG 2026-11-20): move away from `unsafe` per issue #937 — 110 sites at #935 (~89 shipped,
    ~21 test-only), almost all at OS/C-library boundaries. Phases: an unsafe-count ratchet; Unix syscalls → `rustix` (a
    direct dependency since #935; `lsp_server/unix.rs`, `watchdog.rs`, getrusage/flock/geteuid/dup, `process_group(0)`
    for `setsid`); glibc `malloc_trim`/`mallinfo` → the allocator's safe API; no process-env mutation after startup;
    safe libxml/libxslt wrappers upstream in our crates; `unsafe impl Send/Sync`; isolate the rest (signal handlers,
    Windows/macOS queries). Target ≤ 15 shipped, 0 test-only; every PR byte-identical on a fleet A/B and perf-neutral.
    Done: phase 0 and part of 1 (PR 1000), phase 5 on int (UNSAFE5: the 3 `unsafe impl Send/Sync` markers dropped,
    `NO_PROPERTIES` a const empty map; ratchet 80 → 77 blocks, 83 → 80 keywords), phase 1 rest and phase 3 on int
    (UNSAFE6: log mirror, cause record, `getpid`, stderr through rustix; one `getrusage` helper; `--init` writes no
    environment; 80 → 69 keywords, 77 → 66 blocks). Phase 2 step 1 on int (UNSAFE7: `mallinfo2` ×4 and `mi_collect` ×2
    behind `heap_trim::c_heap_mb`/`collect_rust_heap`, one block each; 69 → 65 keywords, 66 → 62 blocks; byte-identical,
    perf-neutral). Left: phase 2 `malloc_trim` (UNSAFE937.2, a fleet measurement first) and the test-only `mallinfo2` in
    `tests/112_fragment_yield.rs` (a `c_heap_bytes` would take it, 65 → 64; phase 6), phase 4 libxslt (2 shipped;
    rust-libxslt 0.1.6 `set_max_depth`, publish approved after both SHIPs, D-18), phase 6 (signal handlers, macOS 4 /
    Windows 9 queries, `_exit`, script bindings 4, tests 29 + test-utils 5); kept with contracts: `getrusage`,
    `sysconf(_SC_PHYS_PAGES)`, `TEXMFROOT` (libkpathsea reads `environ`; written before a second thread unless a library
    host resolves before `init_kpathsea()`), `fork`, `pre_exec`. Every batch shows no slowdown: user and kernel
    instructions, task-clock and `strace -c` counts, base vs candidate (DG 2026-11-25). Known trade-off: the cortex
    worker's signal handler relies on `OnceLock::get` being one acquire load (`exit_report.rs:212`);
    `rust-toolchain.toml` floats on nightly, so its "re-check on a toolchain bump" has no trigger; kept, since a
    replacement needs a new `unsafe` (Gemini R4, 2026-11-25); 5
  - SECR — the Lua sandbox's residuals (#911, #924, #935; OD #1081): carry the Lua protocol over a socket pair
    (`UnixStream::pair()` via `Stdio::from`; `/proc/self/fd/N` cannot reopen a socket, ENXIO), closing the race where a
    chunk reads its own pipe and swallows its refusal record (records are best-effort today); a newline in a refused
    name forges log lines (`Fatal:forged…` at column 0 of the `.latexml.log`; status and counts unaffected): escape
    control characters in `lua_bridge.rs` `shown()`/`error_text()`; a parent SIGKILLed from outside (the kernel's OOM
    killer) orphans a spinning texlua (`PR_SET_PDEATHSIG` needs `pre_exec`); defence in depth (LUAOSBOX): an OS sandbox
    of the texlua child — Linux Landlock (read texmf and the source, write the job dir) + seccomp (no
    execve/fork/socket), a macOS sandbox-exec profile, a Windows Job object (no child processes) + a restricted token; 5
  - PARANOIDIN — (DG 2026-11-25: v0.8.1; "not now" 2026-11-18) a document reads any file the user can read, as under
    TeX's default `openin_any=a`; confirmed for an archive (CLI `.zip`, and `cortex_worker` by its code): all
    19 read paths leak a file outside the unpacked tree by `../` or an absolute name with Status OK (`\input`,
    `\include`, `\IfFileExists`, `\openin`, verbatim/listings/fancyvrb, `\includegraphics` copying it, `\graphicspath`,
    `.bib`/biblatex, `\import`, `\usepackage{../x}`, `\pdfmdfivesum`, Lua `io.open`), and a bare name is answered from
    the process cwd (`SEARCHPATHS` puts it first, `state.rs:800-812`; kpathsea's `.`). DG rulings: archives always
    confined, `--paranoid-input` opt-in for plain `.tex`; `Error:refused_file` once per name, one Warning per
    out-of-tree `\graphicspath`/`\input@path` directory. Rule study (advisor, `agent_reports/2026-11-25_ARCHJAIL_rule_study.md`):
    roots = the unpack root (lexical then resolved), `--path` and the TeX installation from kpathsea's `$TEXMF`
    expansion + `OSFONTDIR` (lexical + same-file, not resolved: the distro tree's symlinks point outside it), never cwd (unless it
    is one of the roots), `$HOME` or `SELFAUTO*`; checked inside `pathname::find`/`pathname::kpsewhich` before any stat (no
    existence oracle); post `graphics.rs:495`'s cwd try; Lua's whole read surface (`io.lines`, `loadfile`, `dofile`,
    `package.path`, `lfs.dir`). DG 2026-11-26: a confined conversion reads from the source directory,
    the whole destination and site directories, the TeX installation (read-only) and the `--path` directories given
    on the command line or in the fleet config (never ones a document adds); everything else, system directories
    included, is refused (one Error per name); writes go only into the destination and site. User trees as `$TEXMF`
    expands them; env `TEXINPUTS` directories outside `$TEXMF` refused (the message points to `--path`); Lua reads
    mirror the write rule. Repro probes `~/data/pk_wt/scratch_PARANOID/`; 5
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
  - SPCONF-BLOCKEMAIL — e-mail lines inside spconf's `\address` keep the note whole; 2407.20176 (2-for-2 unplaced). (the fail-safe of c9b7845ce4d; placing them needs an owner rule stricter than `listed_address_owners`, which misread 1802.09371's "Aline, Roumy"); Reach (census 2026-11-24): 31/30k ≈ 2883 papers
  - SPCONF-ISCLS — Interspeech.cls does not share spconf's `\maketitle` address decision: key-marked names `\name[affiliation=1]` are unseen by the line reader.; Reach (census 2026-11-24): 23/30k ≈ 2139 papers
  - DATELEG-LEGEND — (DATELEGb) the legend-note rescue was CUT: 1507.08926, 2508.05506, 2404.15649, 2602.04557 keep their legend notes unrescued.
  - DATEICML — (root-caused 2026-11-26) no words lost: icml's starred names carry a mark-only `footnotemark` note
    while `\printAffiliationsAndNotice{\icmlEqualContribution}` becomes an unowned block `affiliationnotice`
    (2602.04557; 6 of 20 sampled icml papers ≈ 6.9k corpus-wide; Perl drops the notice, `icml_support.sty.ltxml:52`).
    Fix in the icml binding (`\lx@frontmatter@namemark` for the symbol, per-piece labelled contacts from the notice,
    then the existing kind-legend pass); in progress on `pk/DATEICML`; 1
  - WILEYCORRES — WileyNJD-v2/WileyASNA/MRM `\corres{*Name, address, \email{…}}` stays a free `note role=corresponding`
    beginning with the typed `*` (1903.08928, 2003.04316, 2102.05937, 2201.01977, 2307.09820, 2412.06516, MRM
    2509.13644; ≈ 1.6k corpus-wide); the binding shape of DATEICML with the glyph as label. DG 2026-11-26: the
    starred author's corresponding note (mark ∗, the typed `*` as its label; the e-mail on that author by spelling),
    on DATEICML's legend mechanism; 1
  - MRMAUTHLABEL — MRM `\author[1,2]{…}` drops its labels: all three `\address[n]` and an e-mail land on the last
    author, and `\presentaddress` is absent (2506.14995); 3
  - MLSYSNOCREATOR — an mlsys paper emits 0 creators (2601.11589); 3
  - DATEMACRO — DATELEG NOT DONE (the `\@date` store / date macros; detail not given beyond the ID).
  - AUTHORNLABEL — DATELEG NOT DONE (detail not given beyond the ID).
  - SETCOPY-FLAG — (OD #1078) a store the class's replayed setter changed, handed whole to a frozen setter copy (`\orig@author{\toc@authors}`), reads as no events of the copy's block: each name is written as plain text with the per-name Warning "a name its events don't spell" (names correct, markup lost): tocbase 1811.06821 ×3, 1811.11810 ×3, 1904.05269 ×5, 1905.07135 ×4, 2311.03341 ×4; `setter_copy/tocbase_shape.tex` pins W=3 (`perfect_kernel_batchsetcopy::a_copy_in_an_end_code_takes_the_class_list_once`); fix: the copy's call reads the store's digested names as its own events; Reach (census 2026-11-24): 3/30k ≈ 279 papers; 3
  - SEPTEXT — mark separators `;`/space are not read; with the `\lx@sup@request@affiliation` digestion-path record and authblk `\author[1,*,\textdagger]` † landing on the block.
  - MARKGONEXT — the reading side's `mark_goes_next` (`lines.rs`) still uses source distance where the emitter decides by TeX gap widths (nearer name, tie → next; DG ruling); converge later.
  - UNMARKEDDUP — identical unmarked author-note copies print twice; wait for a real witness on the int head (2602.18802 is already marked); the exact fix is identity (issue #893 Option A idref), not text.
  - DOTCUT — int's generic cut bug behind AAAI dotted marks (revPRESBXb P2, "high"; RED rows c1/c7/n1/n2/n4/b1g + P1-P3); 2509.03329 went to base `\thanks` per use as the fail-safe.
  - OWNLEGGLYPH — an own-legend `\thanks` printing another counter's glyph is not kept by its author (b6; 12 corpus papers; pre-existing; OD #1068).
  - COMBLEG — a note that legends several of the names' marks should be the block's note (whole text, every carrier's mark linked, margin/after the block), never under one author nor split ("Correspondence to: qw281@" follows the ‡ clause but is Wei's); ‡ unlinked today; low priority, reach unmeasured (census 30k).
  - DOCROLES — OD #1063's document-level notes: dates (and other roles) shown behind a role census; role=altaffiliation is not in `f:document-note-shown` (post-merge).
  - AUTHBLKINK — (INVDAG S3) authblk ink residual; detail not given beyond the ID.
  - P1-LOGOS — (stretch, DG run 391) 2610.22523's `#p1` is left holding only the logo images once its affiliations are extracted — delete a paragraph emptied by extraction (or one of graphics only that the class's title block owned).
  - QA61-STORES — QA61 D1 post-merge: (a) K11's `store_lines` readers keep the top-level-line hole (RED `table_store_in_a_top_level_line_with_another_value_red`; base leaves "E-mail: , tel. 555"); (b) `\lx@replay@records` for K11's handed stores (`note_store_handed`) takes a label's first match; (c) RED `offtable_store_moved_leaves_the_rest_of_its_line_a_paragraph_red` ("(corresponding author)" left a paragraph); D2/D3 (unlabelled printed stores; keywords/classification as `ltx:keywords`/`ltx:classification`) await rulings.
  - CJKGB — GB2312 mojibake inside "(…" read as a mark (`lexer.rs:317`), ")" lost; CJK GB decoding is silent where Perl errs.
  - BLOCKPLACE-DECL — `\lx@names@note` declared in jfm, mdpi, ieeeaccess, oup, autart, sigma, spconf, Interspeech only; the other elsart-family clones are unverified and ieeetaes stays undeclared.; Reach (census 2026-11-24): 121/30k ≈ 11253 papers
  - BLOCKPLACE-AUTART — 19 autart papers: the comma-junk titlepage now follows the moved addresses (pre-existing junk).; Reach (census 2026-11-24): 18/30k ≈ 1674 papers
  - AAAILABEL — (DG 2026-11-26: the aaai slot of the v0.8.1 frontmatter reopening) aaai's labelled `\affiliations`
    lines stay an unowned title-page block while the names carry their marks (20 of 20 sampled aaai21+ papers; ≈ 95 %
    of modern aaai). Two gates keep aaai off the displaced-setter route (OD #955): `affiliations` is not in
    `STORE_SETTERS`, and the copies' `_`-delimited self-redefining store is skipped by `assigned_parts_holding`. In
    progress on `pk/AAAILABEL`; the AAAI notes kept as moved (once-per-name `lx_aaai_marked@<kind>`) stay as they are; 3
  - ARYDSHLN — (DG 2026-11-26) translate upstream LaTeXML PR 2889 (the arydshln binding, by Christopher Smolen) with
    dashed borders in the HTML: `ltx_border_{t,b,l,r}_dashed` on a per-side style channel beside `border_color`, a
    `:`/`;{}` column rule that leaves nothing in the cell, `\arrayrulecolor` kept as a colour setter (arydshln.sty
    :1119-1123), no `\xleaders` no-op, the upstream test as a golden pair; its own PR into `int/towards_v0_8_1`; in
    progress on `pk/ARYDSHLN`; 3
  - GTPART — gtpart.cls `\urladdr`, `\subject` undefined (MTREDUCE residual; `\corresponding` bound).; Reach (census 2026-11-24): 9/30k ≈ 837 papers
  - TITLENOTELAST — stopped, no binding, post-release: sig-alternate/acm_proc_article-sp title note order; 1003.2281 (70 census papers).
  - JASAPACS — stopped, no binding, post-release: JASA PACS notes; 0803.0204 (4 papers).; Reach (census 2026-11-24): 3/30k ≈ 279 papers
  - REVGRIDWARN — REVTeX twocolumn grid Warning wording; post-release.
  - TITLEPAGEMARKS — (post-release; ~9/30k papers) report/book/`[titlepage]` article print the arabic `\thefootnote` in force; we write none (fail-safe, MARKBINDd removed the fix); one line in `declare_fnsymbol_title_block`'s title-page branch, large reach → own A/B; RED rows report titlepage + titling fnsymbol, t1/t17.
  - ICMLSTY — the bundled icml sty notice text (INVDAG row; detail not given beyond the ID); icml `*` marks collide with LaTeXML.css's blue "*" author-notes trigger (S3).
  - BMCART — 2112.10068 bmcart (unbound): the corresponding link is lost, five `\author[email=]` keys (rubric "wrong connection"). (verify on the next sandbox run: the bmcart binding landed in 905b2578ae2); Reach (census 2026-11-24): 5/30k ≈ 465 papers
  - TITLE-NOTE-SPACE — the title's pubnotes span sits inside `<h1>` with no whitespace before it, so the heading text (copy/search/accessible name) reads "ActionsThe work of…" (1207.6084); add a space, or lift title notes out of the h1 as the `_meta` pubnotes are (html_feedback#6886).
  - CSSHL-scroll — copying at an off-screen margin note gives no hint (the shared-copy highlight, fix/shared-note-copy-highlight 466a511ac51; ar5iv-css c9b1e99).
  - PNOTE2 — (SHARED, ≈300 arXiv) a pubnote under a cover that stands for the title is lost in the HTML: cernphprep `\Collaboration{ALICE Collaboration\thanks{…}}` (1311.0214, 1406.3206, 1609.07104, 2309.03788); `LaTeXML-structure-xhtml.xsl:379` skips the title under an inked non-`_before`/`_continued` cover (Perl :359), so `maketitle`'s pubnotes never render (:530). Fix (EASY): one `f:titlepage-stands` used at :379 and in the `ltx:titlepage` template (:390-407) to call `pubnotes` with `../ltx:pubnote`; RED `scratch_PNOTE2/rep/titlepage_thanks_pubnote_red.tex`; side lead 1006.5322 (H1/ZEUS).
  - PNOTEAR — revPNOTEA: DRY `LaTeXML-structure-xhtml.xsl:710-728` (titleless head block) with `maketitle`'s tail (:472-498) as one named template; a titleless document's never-printed `\date{\today}` shows the conversion date (1504.03307, 2510.21368; titled-without-`\maketitle` the same); 1112.0709's two `role="journal"` pubnotes (`\jname` + raw IAU-TrA header); a titleless `--splitat=section` document shows "Date:" on every page and links the root as `TEMPORARY_DOCUMENT_ID` (base the same).
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
    at fleet rate and 0.2 % of its entries are hit again (an insertion no longer walks the cache: GFXHARDLINK's usage
    tally walks it only past the cap and prunes to 90 %): measure off (`LATEXML_GRAPHICS_CACHE_OFF=1`) vs 2 GB vs 50 GB on ~200 papers and set the fleet's by
    throughput, output byte-identical; 1
  - TILDEFN — a `~` in a file name becomes `\lx@NBSP` (`latexml_engine/src/plain_constructs.rs:472` overrides latex.ltx's `~`, which is a literal `~` inside `\csname`, latex.ltx:9413-9419, and in file names, :19727-19730); `\input{a~1/child/body}` / `\subimport*{…/a~1/…}` → `Error:missing_file:a\lx@NBSP1/…` where pdflatex reads the file (Perl errs too); fix: define `~` per latex.ltx (protected; literal inside `\csname`) and have `\lx@set@path`/`\lx@append@path` use `gullet::expand_as_csname_text` — changes Linux output (`gullet.rs:3794` maps `\lx@NBSP` to U+00A0 in label names, CLUSTER-NBSP) — SHARED, MED (found by the Windows triage: 8.3 short names carry `~`); 3
  - CURCTRA — aliascnt: `\@currentcounter` gets the root counter (`thm`) where latex.ltx gives the alias (`lem`) (`latexml_core/src/binding/counter/dialect.rs:1150-1162`: when `is_counter_alias(ctype)`, define from `ctype`; pdflatex `[a:thm][b:lem]`, ours `[a:thm][b:thm]`); llncs `\@spothm` same path; readers zref-counter and zref-clever's type; NITs: `ExplodeText!` makes `@` catcode other where `\edef` keeps it a letter (`\ifx` on `ALG@line`-like names), the repro headers cite unbtex.cls:485-493 where the `\AddToHook{label}` is at :495 (revCURCTR); 1
  - FMLISTNOTE — REGFIX residual (revREGBINDb): plain (non-`\footnotetext`) text in a class's title-page list is typeset as a body
    paragraph after the frontmatter, where pdflatex prints it in the title block and FIDELITY_CHARTER.md:168 puts unclaimed title-page
    text at the block as an `ltx:note` (2606.06621 "contributed equally", 2605.22682's preprint box, elsarticle's cnf line; minimal
    acmart `\g@addto@macro\@authornotes{Plain text.}`). Fix: route it through `hand_net_stores`' block note
    (frontmatter_stores.rs:5617); a note holding `\par`/`\vbox` material is the design step. Content kept (formerly lost with an Error).
  - REHEARSER — REGREHEARSE residuals: a line-broken ORCID iD (`orcid.org/0000-0002-⏎9546-4195`) keeps the line end as a glyph
    (2503.20847: an honest 3-word missing + 3-word excess Warning) and its surnames' case (PDF fidelity); five creators sharing one iD
    in different spellings warn 6 lost words (synthetic, `revREGREHEARSE/p2/c.tex`: placed links count once per target). LOSTA's two
    rehearsals without `author_markup_begin` stand.
  - DOIURL — idea, not scheduled (DG 2026-11-16): auto-enrich URL markup that links to a DOI (`doi:` text, `https://doi.org/…`) into
    the DOI link a binding's `\doi` would make; witness 2606.22730 (imsart over OmniBus, whose body `\renewcommand{\doi}` keeps
    the text unlinked, as pdflatex prints it).
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
  - SHIPPEDPKGSCAN — dependency scans still made before a copy is read: `shipped_class.rs` `require_shipped_packages`
    (~213, it chooses the binding's defaults) and `require_package`'s after a raw read (content.rs ~4215, a near no-op
    by the executed-set gate ~4828). Perl never scans a file it reads (Package.pm:2696, :2732); sn-jnl and optica follow
    TeX's order since REGFIXU (OD #1130); 3
  - STARTALIGNGRP — `\start@align` under a document's renewed `{alignat}` ended by its own `\math@cr\black@\totwidth@
    \egroup\egroup` (eqndefns-left: 2601.10843, 2602.04048): Fatal Recursion → 3 Errors, still an empty document; open
    the alignment in a group a plain `\egroup` closes (repro alignment/start_align_under_redressed_alignat). rc2 2026-11-23: no longer Fatal, but 2601.10843 keeps 1,328 of pdflatex's 15,212 words (63 Errors). Deferred, decided by difficulty (DG 2026-11-23); Reach (census 2026-11-24): 3/30k ≈ 279 papers; 3
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
  - TITLEPAGEALIGN (after ACMTEASER) — a cover `wrap_as_titlepage` synthesizes still demotes a centred `logical-block`
    to `block` (`normalize_titlepage_layout_children`, #244), so its paragraphs lose `align`, while a real
    `{titlepage}` now keeps the logical-block (S14). Make the synthesized path keep it; needs its own A/B; 3
  - COMPILESCHEMA (tooling) — `tools/compileschema.sh` stage 2 (`--dump-model`) reloads the committed `LaTeXML.model`
    instead of compiling the `.rng`, so a schema change does not reach the model (S14's model line was hand-edited).
    Check: widen any `.rnc`, run the script, `--dump-model` shows the element unchanged; 3
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
  - SEGVTEST — test processes, which run on glibc (`#[global_allocator]` is set only in the `bin/*.rs`), took a SIGSEGV
    twice in a full nextest run (`-j 64`), both on 2026-11-24: 0 in the 471 gate runs before rust-libxml 0.3.25 arrived
    (2026-11-23 18:10), 2 in the 49 since. Witnesses: gates UNSAFE3 (`114_streaming_complex
    streaming_matches_eager_on_complex`, many conversions in one process, libxml 0.3.26) and LIBXML027b on `ee0484cb1ac`
    (`perfect_kernel_batchfmmarks::achemso_pieces_no_author_holds_are_unlinked_notes`: conversions 1 and 2 of the test
    completed, the 3rd died while digesting `l3backend-pdftex.def`; libxml 0.3.27). Both trees hold #1000 (rustix in
    place of raw syscalls) as well as libxml 0.3.26, so either may be the cause. 0 of 400 standalone repeats of the
    achemso test. Kernel records (`journalctl -k`): the achemso fault reads through an empty-Vec pointer (8) with a
    non-zero length in `invoke_token`'s `result.clear()` (`latexml_core/src/stomach.rs:4390`), a state the function
    cannot reach, and the streaming one returns to 0x15e, so a live stack slot was overwritten on the one running
    thread. Not reproduced: valgrind memcheck on both tests (0 errors, 4 runs), 160 `malloc.perturb` stress runs, 18
    traced full-suite loops. Not seen in production: sandbox run 413 (rc3 candidate `d8e0d3c253`, built with its
    gated lock: libxml 0.3.27, #1000; 30,079 papers) had 0 crash fatals, 0 crash-signal worker exits and 0 kernel
    segfaults, and 0 papers worse than run 411 (DG lifted the rc3 block on it, 2026-11-24). The same wild-jump
    form hit an experimental mimalloc build on 1502.06095 on 2026-11-21, before both changes. Open leads: a kernel
    core dump under load (a single flipped bit would mean hardware), CPU boost on/off, and whether nextest's
    256 MiB thread stacks matter; 3
  - PATHAPI — (DG 2026-11-25, v0.8.1 only; main/v0.8.0 takes no refactor) one pathname API used everywhere, so a path
    bug is fixed in one file: helpers sized to the business needs and to Perl `Util/Pathname.pm`'s 25 operations (ours
    has 47 public functions, while ~24 `canonicalize(`, 3 `std::path::absolute`, 3 `current_dir()`, ~90 `.parent()`
    and ~210 `Path::new(` sit outside `pathname.rs`); two path families, lexical by default and resolved only for
    containment and identity, never compared across (the rc3 Windows 8.3 and rc4 macOS symlink bugs, KPE #1334); a
    raw-path-op ratchet test (counts outside `pathname.rs` only fall); migration by business area in byte-identical
    batches. Design study: `agent_reports/2026-11-25_PATHAPI_design_study.md` (advisor, running). Absorbs UTILDRY's path
    part and PATHFAMILY; 5
  - UTILDRY — shared helpers live in `latexml_core::util` and the whole workspace reuses them (DG ruling and standing
    refactor permission, 2026-11-24); paths first, through `util::pathname`. Rule: a path comparison takes both sides
    from one family, lexical vs lexical (Perl's naming and relative-URL semantics) or resolved vs resolved (where bytes
    land), never a fallback across them (`canonicalize(x).map_or_else(|_| absolute(x), …)`). Witness: the rc3 Windows
    release guards, fixed by `pathname::is_contained_resolved` in `Graphics::resolves_inside_site_or_dest`, guard
    `batchfix995_paths_spelled_through_a_link_keep_their_names`; on Unix `getcwd()` resolves links, so only absolute
    paths spelled through a link reproduce the class, and Windows tests run on PRs only as `cargo check`. Backlog:
    Gemini's U1–U3 study (GEMINI_ARCHIVE_2026-11-25.md L1309-1414): real-vs-lexical sites (`content.rs` 332/2502/2759,
    `title_page_check.rs` 1561-1591, `xslt.rs` 461/613/915, `post.rs` 952 `--sitedirectory`), Windows separator and
    drive-letter sites (`graphics_sty.rs` 633, `svg_sty.rs` 214, 12 `rsplit('/')` basenames), then byte-identical
    DRY steps (`radix`, hex, whitespace collapse, `Staged` into `util::fs`, XML escaping, number formatting), A/B-gated
    where output can move. Each path fix needs a red test first (Windows CI dispatch or a Unix link test). Done on int: F4, the drive-letter sites (`graphics_sty.rs`, `svg_sty.rs` ask
    `pathname::is_absolute`; guard `perfect_kernel_batch79g::an_absolute_graphics_path_is_searched_as_written`, red only
    on Windows). Next (Gemini 2026-11-25 06:59): `converter.rs:399` `outside_the_tree` misses `C:/…` and drive-relative
    `C:foo` on Windows (one `pathname` helper testing `Component::Prefix | RootDir | ParentDir`), `xslt.rs:664`
    (defensive); 3
  - SPLITDOMNS — the whole-DOM split (`latexml_post/src/split.rs`) loses a page's non-`ltx` namespaces: xy-pic's svg
    inside math raises `malformed:namespace` Warnings (1502.06095: 344, 1609.07593: 1,377 under `--splitat=section`)
    and 2606.28974 loses an `aria-hidden`; the streamed split, the default for split runs since OXIDIZED_DESIGN_DIVERGENCES
    #1133, gives the unsplit pages. Reached now only with `LATEXML_POST_STREAM_SPLIT=0` and the streamed split's
    fallbacks (a late navigation, resource, root date or `<?latexml?>` PI; a nested `ltx:resource`); 2
  - PARTUP — a split `\part` book's chapter pages carry no `up` link to their part page, which Perl's carries
    (pre-existing, seen while pruning the head links, OXIDIZED_DESIGN_DIVERGENCES #1134); 2
  - SPLITFIXR — SPLITFIX's residuals after SPLITFIX2 (pages named in document order on both paths, a climbing
    `labelrelative` name resolved lexically, messages relative to the destination; KNOWN_PERL_ERRORS #1329 names the
    split trigger; witness 1502.05726): (d′) under EPUB the outside-the-destination Error still prints the sandbox
    `/tmp/.tmpXXXX/OPS` as the directory: print it in the package's frame (`OPS`), not by making the sandbox the site
    directory as Perl's archive mode does (`LaTeXML.pm` L206/L210), which would move the boundary to the sandbox root;
    (f) SPLITCASE: on a case-insensitive filesystem `\label{Intro}`/`\label{intro}` still collide (the issued-path
    key is case-sensitive; unprobed); (g) SPLITRELDOTS: `\label{sec:../../x}` under `labelrelative` gives
    `out/sec/x.html`, Perl `out/x.html` (the name is collapsed before the parent's is stripped; links resolve);
    (i) 1801.04036 under `--splitnaming=label`: whole-DOM W=1548 (1531 `malformed:namespace`) vs streamed W=17 (base
    alike); (j) `\label{..}` is flagged as climbing though its page `out/S1/...html` climbs nowhere (harmless; the
    `...html` name is Perl's `pathname_make` too); (k) Perl 0.8.8 stalls past 200 s on `--splitpath='//ltx:p |
    //ltx:p/ltx:text'` after its default-ID conflict Warning (a suspected Perl hang, unconfirmed). The index and
    bibliography pages go through the namer (checked; no per-letter pages exist); 2
  - PATHFAMILY — `pathname::absolute` is not one path family: it joins a relative path onto `current_dir()`, which the
    OS reports resolved (macOS `/var` is `/private/var`), while an absolute argument stays lexical, against DG's rule
    (2026-11-24: never compare a resolved path with a lexical one). It made #1001's site-directory check refuse a site
    named through a link (KNOWN_PERL_ERRORS #1334; rc4 PR #1009 accepts "inside as written OR inside resolved") and it
    still makes post's site-relative URLs mix the families: from `link/book`, `--sitedirectory=…/link` gives
    `../../link/LaTeXML.css` where `../LaTeXML.css` is meant (rc1 alike). Complete a relative path from the logical
    working directory (`$PWD` when it names the same directory as `current_dir()`, as `pwd -L`; `current_dir()`
    otherwise, e.g. a child started with a working directory), then grep every `absolute(` caller for a comparison with
    a resolved path; drop #1009's resolved fallback only if the rule then holds for programmatic callers too; 3
  - LTBODYCAP — a longtable `\caption{Bottom}` in the body after the head's `\caption` is dropped without a Warning
    (pdflatex prints it as a row, with a List of Tables entry): `longtable_sty.rs` `\@@longtable` after_digest takes
    the head's captions before the body's (Perl `longtable.sty.ltxml` `HEAD_CAPTIONS || CAPTIONS` alike). Keep the
    body caption where it stands; repro `alignment-bindings/longtable_body_caption_after_a_head_caption_red.tex`
    (synthetic, found probing LTCAPDUP's siblings in FILL1); 1
  - LTFILLCAP — a longtable caption pdflatex prints only on continuation pages still shows when the group that is
    shown has none: a `\caption` only in `\endhead` with an uncaptioned `\endfirsthead`, or a `\endfoot` caption
    with an uncaptioned `\endlastfoot`, gives "Table 2: …" where a one-page pdflatex table has no caption
    (`longtable_sty.rs` ~150-169, the fill-in from another group; base and Perl alike). Fill in only when the shown
    group is absent, not when it is present without a caption. Found by the FILL1 review and Gemini Neon (R3,
    2026-11-25); probe `lt2.tex` tables 2 and 3; 1
  - NOTETAGSWS — a footnote pop-up's text starts with an extra space: the core XML is re-parsed before post (the spill
    handoff) and the indentation inside `<ltx:tags>` survives `xsl:preserve-space elements="ltx:note"`
    (`LaTeXML-meta-xhtml.xsl:37`); Perl's own two-step `latexmlpost` on the same XML does the same, only `latexmlc`'s
    in-memory path avoids it. `<xsl:strip-space elements="ltx:tags"/>` (tag elements only); every footnote's HTML
    changes; 1
  - IDXSTARPART — an index entry inside a `\chapter*` of a book with parts reads its locator as "Part I" (Perl alike;
    Nasser workflow check 2026-11-25, drained repro `idx_unnumbered_chapter_tag`); 1
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
    2505.20965; option: a thin binding routing `\citeasnoun`/`\citeyear` to the kernel cite); emulateapj's `\textwidth`
    is 345pt where emulateapj.cls:334-335 sets 7.1in and `\columnsep` 0.3125in (EMULATEAPJWIDTH; 1612.09425's f1:
    pdflatex ≈231pt, ours 155pt; 0812.1578; its own A/B, it moves every emulateapj paper's widths); `[epj]{svjour2}`
    papers never get the EPJ panel page (SVJOUREPJPANEL; hep-ph0608235; svjour_cls.rs:82 `is_document_class("svjour")`);
    BINDFID's audit list: aas_support (aas_macros, deluxetable), svproc (aas_macros), paspconf (aaspp), emulateapj's
    base class aastex (the real one is REVTeX), sv_support's unconditional natbib (svjour3.cls:68 only under its
    option); `\meaning\text` shows `\lx@text` where pdflatex shows `\protect \text ` (OD #1104; robust needs the lock to
    hold the inner `\text␣`, OD #568); `\bibitem[\lab]` with an argument-taking `\lab` (BIBLABELARG; pdflatex 2 errors,
    we 0 and cite `[L]`; mnras's `\citeauthoryear` label: pdflatex 2 errors, one per `\write`, we 1;
    `$\frac\text{a}{b}$`: pdflatex 37 errors, we 0 with `tex="\frac{\text{}}{a}{b}"`); ulem's `\ul` runs through an
    `\end{…}`/`\endgroup` at its depth and past a closing `$`, and a cell-end-closed line loses the cell's right border
    (pdflatex fatal; RED repros under `loader/ulem_declaration_*`), and a document's own underline macro through
    `\useunder` in a `\multicolumn` still errs; breakurl's `\burl` URLs print as sans text without a link, as pdflatex's
    `.bbl` fallback since REGFIXU (2403.06633; a binding `\burl` = `\url` would be a surpass); 3
  - BIBLISTRES — (BIBITEMLABEL2 review, all pre-existing; base the same or worse) thebibliography residuals: asl
    citations print their keys ("[Ku:Book]" where the PDF prints "[Woo10A]"; 1005.4193, 0907.5495, 2108.03574) and a
    custom `\my@lbibitem` writing `\bibcite` leaves its cites unresolved (1211.3435 "[Oseledets]", PDF "[Os]");
    emulateapj's `\def\@biblabel#1{\relax}` entries get [1], [2] where the PDF prints none (1503.02542); mnras prints
    labels 1/2/3 and "[1]" cites where the PDF has none and "(1)"; a bare `\item` after natbib's `\bibitem` is [2],
    pdflatex [1]; math in a tag label is upright text and `\color` in it is lost (1609.04957, 2402.14816; lists alike);
    a bibblock keeps the label's trailing space where `\@item` ends in `\ignorespaces`; 2108.03574 gains stray
    asl-internal entries; 1
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
  - AMSPPTRESRES — (AMSPPTRES done on int: each `\ref` field digested as `\makerefbox` boxes it; math0610862 101 Errors
    + Fatal → 2 Errors, +1,985 words; math0205237 +2,144) residuals: a copy's additions over amsppt are lost
    (math0610862 `\Refst`, `\subrefhead`; 1003.5377 `\SecondPage`; RED `amstex/amstex_copy_additions_are_kept_red`);
    amsppt's self-redefining `\title` stays refused, 1 Error with every word kept where TeX prints "[T:Hello]" (RED
    `amstex/amstex_self_redefining_title_runs_red`; LOCKSIDEFX); post-merge REDs from the round-2 review, all synthetic
    (0 of the 182 `\endref` papers): a blank line inside a field glues its rest to the next field in an invented
    untagged bibitem, and one before `\moreref` splits the translation off
    (`amstex/amsppt_blank_line_inside_a_field_red`; fix: a marker with no field open ends the paragraph, as
    `\makerefbox`'s `\endgraf`, amsppt.sty:956); a blank line after `\ref` gives a bibitem tagged "0"
    (`amstex/amsppt_keyless_ref_is_numbered_red`, Perl the same for keyless refs); `\endref` inside a box
    (`amstex/amsppt_endref_inside_a_box_red`); a kern-only field (`\by\kern3em`) is dropped with its comma, where TeX
    prints blank width (PDF-only); cost ~30k instructions per reference (math0610862 +0.24 %; four per-reference group
    assignments, a Rust primitive would take them); Reach (census 2026-11-24): 38/30k ≈ 3534 papers; 3
  - THMEDPSRES — REGFIX-E's residuals (PR #960): thm-listof's filters (`\showtheorems`, `\ignoretheorems`,
    `\onlynamedtheorems`, `\setlisttheoremstyle`) are undefined under the thmtools binding (2 of ~519 census papers;
    `\usepackage{thmtools}\ignoretheorems{x}`); edpsmath's `\keywords` stores into box `\@b@kwrds` and our
    `\maketitle` never runs `\@setkeywords`, so 1110.0287's keywords are missing (~11 of 60 edps papers; under
    `[proc]` the PDF empties them too: a FIDELITY_CHARTER §2 question); a stored `\@subjclass` without a year gets
    "1991 Mathematics Subject Classification" (Perl ams_support.sty.ltxml:131) where TL 2025's amsart prints "2020"
    (amsart.cls:555, :789); 1704.06431's `Fatal` enumerate/`\lx@tags` recursion (pre-existing on main); 3
  - MSCINFER-LATER — the MSC edition follows the codes (OD #1168) only for the amsart family and amsppt; elsarticle
    `\MSC` (its name "MSC: " drops the year it prints; 1405.7537), svjour `\subclass` "(2000)", informs, compositio
    and iopart `\ams` still print their own year unchecked: each needs its label-year fix through `\lx@msc@infer`; 2
  - MSCINFER-AMSPPT — amsppt keeps one global `\lx@ppt@year`: a second topmatter re-stamps the first classification
    after its Info was logged (RED `sectioning-frontmatter/amsppt_msc_edition_two_topmatters_share_one_year_red`), and
    two `\subjclass` in one topmatter are both kept where TeX prints the last; one year macro per call; 1
  - MSCINFER-COUNT — when the class keeps the author's renewed label, the whole label is passed as the print, so
    `note_info_counted` counts its words as accounted-for losses though they still print (it could mask a real
    lost-word Warning); count nothing when the name is kept, and say in the Info that only the scheme follows; 1
  - MSCINFER-RENEW — a `\subjclassname` renewed after the call names what `\maketitle` prints, but the scheme keeps the
    call's edition (RED `msc_edition_renewed_after_the_call_names_the_scheme_red`, 0811.1789); 1
  - MSCINFER-DATE — arXiv single-file sources arrive undated (an unzip-restored "no time" entry reads as 1980 and is
    dropped below 1991, `source_date.rs`), so the era rule seldom applies and the corpus takes DG's oldest-edition
    fallback (2026-11-26): math9909024 → 1991, hep-th0307062 → 1991 (its era's 2000), 1504.02453 → 2000 (its era's
    2010); `source_date` should treat a 1980 zip entry as undated rather than rely on the 1991 floor; 2
  - HALIGNREVERT — no guard reaches an alignment reverted during its own absorption (`self_referential_alignment`,
    tex_tables.rs ~933-941; the old panic-recovery path, whatsit.rs ~355-362): a shape that still does is owed (OD #1167
    Residual); 1
  - YTABNODE — a Young diagram set in a pgf node renders as the running-text diagram, boxes 21.86×25.32 against
    pdflatex's 15.53×15.6, so 2509.03390 Figs. 1–2 and 1909.01257 Fig. 1.2 overlap their labels (content kept; OD
    #1167); owned by the ytableau binding batch; 2
  - QA14K2-FMT — enumitem sets a description's `font`/`format` inside `\descriptionlabel` and sends every label through
    `\makelabel` (enumitem.sty:1470-1480); ours wraps the key around the whole label from outside, so a keyed `\newlist`
    description is not bold, a bare `\item` prints its number, and `{description}`'s `\normalfont` undoes the key (RED
    `list-structure/enumitem_description_format_inside_the_label_red`; 1810.05092, 2609.31516, 2212.04966). The general
    rule: every item label through `\makelabel{\@itemlabel}`, with enumitem's format inside; 2
  - BAREITEMTAG — in a keyed `{description}` a bare `\item` keeps an empty `<tag><text/></tag>` (the netted `\labelsep`
    skip carries the key's font; 1305.3417; same RED repro as QA14K2-FMT); 1
  - WIDETABLELEFT — base CSS: a centred table wider than the screen starts past the page's left edge, where no scroll
    reaches (issue 972's longtable 17 px at 375 px, 1805.00336 126 px, 2001.07938 220 px; RED
    `block-model/wide_longtable_past_the_left_edge_red`, from 0.8.0-rc6's GROWBASE review); 1
  - FLEXCELLGROW — base CSS: a minipage figure panel in `.ltx_flex_cell` grows to its display but the flex cell does not,
    so a wide display overprints the neighbour panel's caption (pre-existing, not a Grow regression; the flex cell holding
    such a box needs `min-width:min-content`; GROWBASE review probe `fig.tex`); 1
  - SCHEMAMODEL — a Rust tool that generates `LaTeXML.model` from the `.rng` schema, with a drift guard (DG 2026-11-26),
    the first milestone of a Rust RelaxNG and validation toolbox: rnc → rng, rng → model, and HTML-dialect validation in
    place of the W3C validator; 2
  - RUSTONLY — load-bearing tools move to Rust (DG 2026-11-26: "tools can survive as python if they are exploratory, but
    long-term we really want to be Rust-only"): `tools/*.py` holds 10 scripts; port the ones CI, release or the guards
    depend on first; 1
  - ITEMTHEBLANK (needs a ruling) — `\item[…]` lets `\the<ctr>` be empty (Perl latex_constructs.pool.ltxml:1390), so
    `\item[(\theenumi)] B\label{y}` prints "()" and "See ." where TeX prints "(1)" and "See 1."; `\item[\fnum@fc]` gives
    "Fig. :" (TeX "Fig. 1:"); a nested list reusing the counter labels its inner item "Fig. 0:" (TeX "Fig. 1:"). Keep
    Perl's design or blank only when the tag does not mention the counter; 3
  - ESCAPECAT — no token carries `Catcode::ESCAPE`, so tests on it are dead: `siunitx_sty.rs:488` never adds Perl's
    `T_SPACE` after a matched control sequence (siunitx.sty.ltxml:207; fix with a `\num` A/B, `tex=` spacing moves);
    `amsmath_sty.rs:1105`/`:1114` redundant, `tex_character.rs:232`, `latexml_sty/mod.rs:199` unreachable arms; 1
  - ROUTESCAN (after REGTAILC) — `converter.rs::begins_document_in_pdf_branch` (OD #1154) misses a switch right after
    any control word (`\makeatother⏎\ifpdf⏎\begin{document}`, likely IEEEtran's `\IEEEoverridecommandlockouts⏎
    \ifCLASSINFOpdf`): block only after definers and prefixes (`\newif`, `\let`, `\def`…, `\global`, `\expandafter`,
    `\providecommand`/`\newcommand`); `uncommented_code` does not strip comment.sty `comment` environments, so a
    `\begin{comment}\ifpdf\end{comment}` flips the route (also the figure cue, `has_latex_cue`); two nested unclosed true
    conditionals before `\begin{document}` give `Error:expected:\fi` + Fatal where pdflatex typesets with two
    "incomplete" warnings (pre-existing). Probes `tmp/rv/route/t07, t08, t15`; 3
  - ROOTTOCSPLIT (after 0.8.0, pre-existing) — a split root page's run TOC comes out as several `<nav class="ltx_TOC">`
    lists (Preface / Pt1 / Pt2) where Perl has one; both split paths, before and after SPLITPARENT; 1
  - LUAERRSEV (after 0.8.0) — a document Lua chunk's runtime error is an Info (`pdftex.rs:118-123`, `luacode_sty.rs:64`)
    while lualatex stops with rc=1; LUAEXEC2 escalates only a chunk that carried a sandbox refusal. Decide whether every
    chunk runtime error is an Error (texlua lacks LuaTeX's node/font libraries, so measure the Error flood on the manual
    sweep first); 1
  - LUAERRSEV2 (after 0.8.0) — answer C modules, network libraries and `package.loadlib` with lualatex
    `-no-shell-escape`'s values and a `W` frame (`loadlib` → no value; `require` of a C module/network library →
    `false, msg`; `luatex-core.lua` L195-200); decide how `G` engine gaps rank once chunk runtime errors are ruled; and a
    chunk with a `W` then an uncaught `D` logs two Errors (the `D` and the escalation) — skip the escalation when a `D`
    already flags the chunk; 1
  - BABELTILDE (after 0.8.0, pre-existing, Perl too) — under babel, `x~y` gives "x y" with an ASCII space: babel's `~`
    shorthand `\leavevmode\nobreak\ ` (babel.sty:1504) digests its `\ ` to a plain space and the no-break is lost; TeX prints
    a no-break space (U+00A0). The default labels end in U+00A0 since MANFM2 (OD #1136); body text does not; 1
  - LUAMKTEX (record, decided) — the Lua sandbox sets `MKTEXTFM`/`MKTEXPK`/`MKTEXMF`/`MKTEXTEX`/`MKTEXFMT`/`MKOCP`/`MKOFM`
    to 0 before kpathsea starts (LUAFONT2): with `MKTEXTFM=1` in the environment, `kpse.find_file` (since SECLUA) and
    `font.read_tfm` ran mktextfm/METAFONT and wrote into `TEXMFVAR`. Stricter than lualatex, which may run mktextfm for a
    missing TFM; fail-safe by choice. State it in SAFETY.md; 1
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
  - YTABMATRIX — (RAWDEFR root cause, 2026-11-23) ytableau's `\ydiagram` inside a pgf matrix cell (`tikzcd`, `\matrix[matrix of math nodes]`) closes `svg:g` while the `\vtop` capture block is still open (ytableau.sty:190-206 `\vtop{\iffalse}\fi` … `\iffalse{\fi}` around an inner `\halign`): `\begin{tikzcd} \ydiagram{1} & B \end{tikzcd}` gives 2 Errors on rc2, pdflatex clean; a plain node, a tabular cell, `\vcenter` are clean. Witness 1502.07025 (123 × `svg:g isn't allowed in <ltx:equation>`). Start from `pgfsys_latexml_def.rs`'s matrix cell build; Reach (census 2026-11-24): 12/30k ≈ 1116 papers; 3
  - EMPTYGROUPUNARY — (issue 991 review, pre-existing; math parked) `${}-b$` drops the `{}` and makes the `−` unary (no
    space); TeX's empty group is an Ord atom, so the `−` is a binary operator with \medmuskip on each side; 3
  - NOTEMARKXMATH — a note mark taken from a formula tag reads the XMath tokens and the MathML text together:
    2509.25166's marks read "∗ † ∗†"; `\renewcommand\thefootnote{\ensuremath{=1}}` gives "= 1 =1"; Perl's `note-mark`
    template does the same (shared); 3
  - TEXTOPCHROME — Chrome sets `\text{x}+\text{y}` tight (`\frac{\text{TP}}{\text{TP}+\text{FP}}`): Chrome 158 gives
    `<mtext>x</mtext><mo>+</mo><mtext>y</mtext>` 0px each side, 3.55px when either neighbour is not an `mtext`; MathJax
    3.2.2 spaces it. Only a sibling that is not space-like fixes it; the Math WG is narrowing `mtext` space-likeness.
    Measure reach first; 3
  - CELLLEADSPACE — a leading `&\quad g` with no operator after it still loses its `\quad`: an author space is applied
    only between two row neighbours (`presentation.rs` `adjust_pair`); 1
  *Folded 2026-11-24 (CLIAUDIT, CACHEAUDIT): the command-line options audit (every option traced to its algorithm; the easy fixes are the CLI-wiring PR) and the post-processing cache audit (SQLite only for small keyed metadata shared across processes or runs — the resource cache, `LaTeXML-oxide.cache`, in the issue 995 PR; blob caches stay content-addressed files; memos stay maps):*

  - CORTEXEAGER — (DG 2026-11-25: keep for now) `cortex_worker` converts eager-first and reruns under streaming only
    on a memory stop at the restart watermark, where the CLI streams from the start when the source projects over the
    fuse (`streaming_restart.rs:76`): the eager pass costs 46 s, 37 %, on NLCT (CORTEXRSS round 1). With it: the worker's
    box-cap and fuse Fatals say "raise that ceiling" where its watchdog line names `--max-rss-mb` (record the flag's
    name beside `MEMORY_CEILING_MIB` in `arm_memory_fuse`, CORTEXRSS review); 3
  - CONFIGPREP — (CLI audit, advisor; DG 2026-11-24) one `prepare()` porting `Config.pm` `_prepare_options` in
    `latexml_oxide/src/config.rs`, used by the CLI, cortex_worker, the server, `api.rs` and the render children through
    one serialized post plan; output byte-identical; 5
  - OPTGUARDS — (same) exhaustive option transfers (no `..Default::default()` between option structs) and a
    `cluster_cli` effect table with one row per `--long` option or a named exemption; 3
  - OPTPREFIX — (same) Getopt::Long's unique-prefix abbreviations (`--destin` for `--destination`) through clap's
    `infer_long_args`; 1
  - OPTMISSING — Perl options we reject (probe 2026-11-24: `latexml_oxide --<name>` → "unexpected argument", 49 names
    with negations), by group. Post toggles: `--crossref`, `--scan`, `--index`, `--permutedindex`, `--splitindex`,
    `--splitbibliography`, `--validate`, `--omitdoctype`, `--graphicsmap`, `--svg`, `--pictureimages`, `--mathimages`,
    `--mathsvg`, `--mathimagemagnification`, `--linelength`, `--parallelmath`, `--openmath`/`--om`. Input and output:
    `--bibliography`, `--tex`, `--box`, `--output` (Perl's alias of `--dest`), `--parse=<grammar>`, `--profile`
    (latexmlc's presets), `--VERSION`. Persistent object DB: `--dbfile`, `--prescan` (on demand, DG 2026-11-24).
    latexmls's daemon options (`--mode`, `--port`, `--address`, `--expire`, `--autoflush`, `--cache_key`, `--base`):
    out of scope (our server is the LSP), say so in the docs. Decide per group: wire, alias, or reject with a pointer; 5
  - CLIFORMATS — `--format`: the core's `OutputFormat::TeX`/`Box` (`converter.rs:983-1006`) is unreachable from the
    CLI (Perl's `--tex`/`--box`; a `.tex`/`.box` `--dest` makes Perl infer the format, where we write XML into
    `o.tex`); html4 and jats (their sheets are embedded, `xslt.rs:1006-1015`); `--xml` and `--format` are not
    last-wins; 3
  - CLIPOSTDEF — an explicit `--post` turns pMML on for non-HTML formats (latexmlpost's default, not latexmlc's), and
    `--split` implies post (Perl's splitxml writes the split XML without post): align with Perl or record each in OD
    (OD #1093's math-format model stays, DG 2026-11-24); 1
  - CLIPATHPI — `--path`: we never emit `<?latexml searchpaths?>` (`INCLUDE_PATH_PIS` set at `state.rs:912`, never read)
    and post's `PostDocument` gets only the source directory, so post-time lookups (graphics, bibliographies) ignore
    `--path`, which Perl passes on; 3
  - LSPOPTS — `--server` (the LSP) silently ignores every other option (`--preload`, `--path`, `--nocomments`, …):
    honour the conversion options in the server's per-document settings, or reject them as a usage error; 3
  - CORTEXPOOL — `cortex_worker --pool-size` threads are pericortex `thread::spawn` threads (≈2 MiB stack, not the
    256 MiB of `cortex_worker.rs:~1855`, and without its 5M pushback thread-local, :~1858); `--limit` counts per
    thread (N × limit tasks per process); `--path` is appended after the source directory though its help says
    prepended (`cortex_worker.rs:384-385`), and post gets no search paths. The fleet runs pool 1, so no fleet effect; 3
  - CLINITS — small CLI items: `--source` silently wins over positionals (warn); several positionals join into one
    document (Rust-only; document it); telemetry's `timeout_s` is always 0 (`telemetry.rs:143`, `set_timeout_s` never
    called) and its `git_sha` (from `latexml_core/build.rs`) can differ from `--version`'s revision;
    `resolve_max_memory` runs three times (its Debug line twice under `-v`); `--inputencoding` survives `utf8.def`
    here (`utf8_def.rs:297-300`) where Perl's resets it (low impact); 3
  - RENDERDBRO — (DG 2026-11-24: v0.8.1, measured first) write the parallel render's ObjectDB snapshot
    (`LaTeXML-render.db` after the issue 995 PR; `render_workers.rs:~504`) with `journal_mode=OFF` and open it
    `immutable=1` in the workers, with the planned lazy read-only attach; it stays a file separate from
    `LaTeXML-oxide.cache` (a per-run read-only snapshot in the private spill directory vs a persistent read-write
    name table in the destination; Perl keeps ObjectDB and `LaTeXML.cache` apart too); 3
  - TMPFILES — the intermediate-files inventory's defects (2026-11-24, from an Explore sweep; verify each before fixing;
    the page is `docs/INTERMEDIATE_FILES.md`): (a) the default log `<stem>.latexml.log` lands in the current directory
    under a fixed name, so two concurrent runs of `main.tex` from one directory collide and the second deletes the
    first's log up front (`latexml_oxide.rs:1047-1053`, `:1188-1193`); (b) Ctrl+C / SIGTERM run no cleanup in a normal
    conversion, and the format build's handler exits 130 without `remove_registered_paths` (`format_dumps.rs:94-104`);
    (c) never pruned: the unpacked embedded dumps (`$TMPDIR/latexml-oxide-dumps-<hash>-<uid>/`, one per build) and the
    Lua preludes (one per hash), beside FMTCACHE-PRUNE's format cache; (d) name races: the embedded dumps'
    `.tmp.<pid>` is shared by a process's threads and opened truncating (`embedded_dumps.rs:200-218`), the post handoff
    is named by pid only (`post.rs:~692`; two in-process conversions into one directory collide), `.fonts.eps` by
    nanoseconds only (`postscript_fonts.rs:205`), and the format build's `set_current_dir` moves every thread's
    relative paths (`format_dumps.rs:527`, `:557`); (e) `.fonts.eps` is not registered with the watchdog's
    remove-on-exit; (f) the source directory is written (streaming spill, the writability probe, Lua `io.open`), against
    the note at `virtual_files.rs:7-9`; (g) final outputs are written in place, not staged (`pack.rs:82`, `:142`;
    `writer.rs:~410`), so a crash leaves a truncated output; 5
  - SQLITESTORE — (DG 2026-11-24) harden and productize the SQLite store that issue 995 introduced
    (`latexml_post/src/sqlite_store.rs`, `resource_cache.rs`) into the one shared cache layer, following Perl
    latexmlpost's `LaTeXML.cache` (names stay the same across a reconversion of the same file): safe for parallel
    threads in one process (a connection per thread or a small pool; no connection in a static, since the LSP forks)
    and for many processes where appropriate. Uses: (a) the resource-name cache per destination (as issue 995 ships
    it); (b) the image-conversion cache's metadata, host-wide: an indexed table replaces the `.dims` sidecars and the
    per-insert prune walk (LRU by last use, atomic claims so concurrent conversions do not convert one image twice),
    while the converted images stay content-addressed files (with GFXHARDLINK's copy delivery; GCACHECHURN, FLEETR);
    (c) consider the format-dump cache's index (FMTCACHE-PRUNE). Schema versions with migration; one failure policy
    (a cache problem is a Warning and an in-memory fallback, never a Fatal); guards for N threads × M processes;
    per-image cost measured (≤ +0.5 % per document). Design decisions from the 2026-11-24 adversarial review (GEMINI.md
    in the kit): a normalized `LaTeXML-oxide.cache` schema (`resources` + `counters` + `kv` for Perl's generic cache only, a
    `user_version` bump with migration; retires the `|` delimiter, orphaned `.svg` claims, stale rows); for the image index no
    SQLite lease rows (kernel `flock` per key if duplicate conversions prove worth it), read-only hits with a coarse
    best-effort LRU touch, generation-tagged blob names against the prune race; no connection in a `static`/`thread_local!`
    (a pool per post session, opened after any fork); the format-dump cache stays files + `flock` (no `rusqlite` in
    `latexml_core`; recommendation, DG to confirm); one open layer shared with `ObjectDB` (with RENDERDBRO); 5
  - AR5IVBUGS — latexml-oxide defects the 2026-11-24 ar5iv issue audits found. Each was seen in ar5iv's cached
    0.7.6 HTML, so re-check it on main before fixing. Every one is silent, against the every-loss-counted rule:
    (a) nicematrix `\Block[fill=[RGB]{243,240,255}]{3-3}{…}`: an option with nested brackets leaks `]{3-3}` into the math, and
    the block gets no span or fill (2405.21060 `structure/efficient.tex:83`; ar5iv #497, #516); `\Block` without options spans
    correctly; (b) fontawesome5 icons are empty `fas fa-*` spans and no FontAwesome CSS is loaded, so they are invisible
    (2401.00905 Table 1; ar5iv #512); `fontawesome5_sty.rs:34-35` leaves out Perl's CDN `RequireResource`; (c) emulateapj's
    `\tablecomments` inside a plain `table` is dropped (1310.5776 `ms.tex:646`; ar5iv #637); (d) blkarray `block`
    delimiters are lost: `\begin{block}{c (cccccc)}` renders without its parentheses (1811.10792 `A5.Ex17.m1`; ar5iv #638),
    and the brackets are dropped too (2310.17416; ar5iv #473); (e) framed's `\MakeFramed`/`\endMakeFramed`/`\FrameRestore`
    are no-ops (`framed_sty.rs:332-334`), so an author's framed or shaded environment loses its box (2003.04755 `formal`;
    ar5iv #269); (f) titletoc's `\printcontents` prints nothing (2310.11511; ar5iv #429). Leads from Gemini's study,
    unverified: ACL/COLM parenthetical citations rendering bare, 1908.05686 losing 36 % of its words at 0 errors, empty
    `{forest}`/`tcolorbox` figures (`/home/deyan/data/ar5iv_triage/master_open_witnesses.tsv`); 4
  - NATBIBRES — residuals of the natbib apalike-label fix (OD #1161; review 2026-11-24). (a) An authoryear label with an
    empty author, `\bibitem[, 1998]{l}`, prints the key as the author: "(l, 1998)" / "l (1998)", where pdflatex prints "(, 1998)" /
    "(1998)". It is the general fallback for an entry with no authors; synthetic, with no corpus witness; RED repro first.
    (b) A `(year)` label with no closing `)` at depth 0 repeats the year text as the full authors (`rest_idx` stays after the
    `(`); Perl writes no full authors there. Read from the code, not probed. (c) `[{Maes}(1995)]` keeps its braces on the
    author and year, where TeX strips them; nothing visible; 1
  - SVGCOLLAPSE — (DG 2026-11-24: performance, "keep or improve rc2's speed") `collapse_svg_group`
    (`latexml_engine/src/tex_box.rs:525`, `Tag!("svg:g", after_close)` :1202) is quadratic in the depth of a transient
    `svg:g` chain. On arXiv 2605.19500 it makes 46.8 M node moves through a chain 9,642 deep (the final XML is 34 deep),
    about 46.5 M of them in Step 3 (`add_next_sibling` of every already-hoisted trailing group at every level), plus
    per-child attribute `HashMap`s. That cost is inside rc2's 916 G too, so fixing it can beat rc2 on heavy TikZ papers.
    Design (Gemini Neon's sketch, GEMINI_ARCHIVE_2026-11-25.md L812-824; check it against the code first): collapse once
    at the outermost `svg:g` close; the 11 COLLAPSIBLE keys as a bitmask read once per node; a post-order pass whose
    hoisted leading and trailing runs move in O(1) while the masks allow, materialized in the DOM once. Verify the
    premise that nothing between an inner and the outermost close reads the closed groups (`record_svg_ids` at the
    `svg:svg` close, the `svg:foreignObject` cleanup at :1400). A hazard is confirmed in the code: `\lxSVG@closescope`
    (`pgfsys_latexml_def.rs:1528-1532`) closes `svg:g`s until one carries `_scopebegin`, and Step 5 (`tex_box.rs:647`)
    copies `_`-prefixed attributes (`_scopebegin` too) from a merged single child onto the group as it closes. A
    deferred collapse would move where that loop stops, so collapse at once when the closing group's single child
    carries `_scopebegin`, or keep the loop's check on the unmerged child. Bar: byte-identical output on every pgf/TikZ
    witness, plus the instructions:u gain on 2605.19500 and a heavy-picture population; 3
  - ARENAQUAD — (FIX1011 residual, pre-existing) the issue-1011 pattern — `lookup_string(K)` + push + `assign_value(K,
    String)` — interns a fresh copy of a whole list per call (`Stored::from(String)`, `store.rs:720`), so the
    never-freed arena grows with the square of the list. LTLABEL: `longtable_sty.rs:292-303` re-interns the space-joined
    label list at each `\label` in one longtable (30 MB at 2,000 labelled rows, ~750 MB at 10,000; synthetic witness).
    FMACC: per-author accumulators `ELS_AUTHORS` (`elsart_support_core_sty.rs:147`), `NAME_LABEL_MARKS` and
    `note_labels` (`authblk_sty.rs:370,486`), `AUTHOR_KEYS` (`frontmatter_stores.rs:8631`) — a few MB at ~1,000 authors
    (Gemini: 45-90 MB at 3,000). Fix: the mapping-or-list change `note_content_file` got (PR 1015); 3
  - ARENANITS — (FIX1011 review nits, post-merge) `latexml_oxide.rs:665/936` a caught-and-recovered panic leaves
    `PANIC_LINE` set, so a later `resume_unwind` from another thread would log the earlier panic (not reachable today:
    the only `resume_unwind`, `xslt.rs:176`, re-raises on the hooked thread) — use it only when its reason equals the
    payload's; `:1716` clones the core log for the whole post phase (move it in, take it back) and leaves a gap between
    `Converter::flush_log` and the arming; `:1245` a panic with an archive `--dest` writes `<stem>.latexml.log` beside
    the zip where a normal run packs it; `content.rs:3476` a non-mapping `lx_content_files` (a cached dump from
    2026-11-04..14) makes `content_files()` silently empty; 1
  - INTWCASE — (INTWIDTH #4a, Gemini 2026-11-25, CONFIRMED) `\lccode`/`\uccode`/`\sfcode` keep 16 bits
    (`tex_character.rs:265,273,282` cast `i64 as u16`): `\lowercase{𐐀}` gives `Ш` (U+10400 → U+0400), a non-BMP case map
    is silently rewritten; hold the full code point; 1
  - INTWHOSTILE — (INTWIDTH #4b/#5/#6, hostile-only, CONFIRMED) out-of-range codes and integer edges pass silently where
    TeX errs: `$\mathchar"18041$` dropped with no message (`tex_math.rs:986,1020,1045,1071,1118`, "Bad mathchar" in
    TeX); `\endlinechar` as `i16` wraps 65536+13 → 13 (`mouth.rs:1847`) and `^^` casts via `i16`/`u8`
    (`mouth.rs:1134-1136`); `numeric_ops.rs:170-179,221-230` add/subtract skip TeX's `\maxdimen`/2^31 overflow checks
    (shared with Perl; SIGFPE CONTRADICTED — Rust panics, `divideround` makes d positive). Raise TeX's error and clamp,
    never wrap; 3
  - CHARNONBMP — (INTW5 review P2, PARITY with Perl's silence) `\char"20AC`, `\char"20041` and `\symbol{"10000}` vanish
    with 0 E/0 W; pdflatex says "Bad character code". A silent content loss: at least a Warning naming the code (or keep
    the character, as XeTeX would); RED post-merge; 1
  - YARGDEFCOUNT — (INTW5 review P1, pre-existing) `\@yargdef` parses its count as a string where latex.ltx:1281 uses
    `\number #3` (`sect08.rs:129-131`): `\@yargdef\fh\@ne{\numargs}{(#1#2)}` gives 2 Errors ("#1/#2 should never reach
    Stomach") and `J[(12)mn]`, pdflatex `J[(mn)]` clean. Reachable from raw callers that pass a macro count
    (etoolbox.sty:81, elsarticle.cls:609, biditools.sty:631, mhsetup.sty:163, tocbasic.sty:2006); 1
  - XARGSCOUNT — (INTW5 review, KPE #1336) xargs declarators do not go through `latex_arg_count`, so a bad count there
    is not checked (Error + clamp 0..9) like every kernel declarator; read the count as a number
    (`package-bindings/xargs_argument_count_unparsed_red.tex`); 1
  - MATHTOOLSX0 — (INTW5 leftover) `\DeclarePairedDelimiterX\ci[0]{(}{)}{i}` then `$\ci$`: `mathtools_sty.rs` `n.max(1)`
    reads an argument the command does not take (the closing `$`: Missing $ inserted, unparsed math), and mathtools's
    own "Macro need 1 or more arguments" is not raised (pdflatex 1 `!`, "(i)"; Perl 0 errors, "(i)");
    `package-bindings/mathtools_paired_delimiter_x_zero_arguments_red.tex`; 1
  - TESTTEXTFOLD — (TESTHARDEN1 round 2, deferred) the 84b `text` helper still reads markup by its own rule; folding it
    into `markup_text` moves 131 `cluster_package_guards` expectations (`<sup>1</sup>Univ` reads `1Univ`, U+2003 not
    collapsed; largest: 94b 12, 104c 8, ieeeroute 7, c1 7, 97f 6). Needs a ruling on whether a mark/text boundary reads
    as a space; also `markup_text` parses `<script>`/`<style>` content as markup (skip to the close tag); 3
  - TESTHARDEN2 — (TESTHARDEN burn-down) 76 guards still assert with prefix-ambiguous `contains("<tag")` (`<note` also
    matches `<notes`); move each to the whole-element helpers in `util/test.rs` with the shadow hook
    (`LATEXML_TEST_SHADOW`) showing 0 flips; 3

  - WINLONGOUT — (PR #1014 residual, pre-existing) on Windows, `Graphics::reachable_source` gives the converters a short
    alias of a figure whose source path reaches MAX_PATH, but the staged output path (`writer.rs` `staged_path`, `<dest
    dir>/.<stem≤64>.tmp.<pid>.<seq>.<ext>`) is not aliased: a deep destination directory with a short figure name can
    still pass 260 characters, and ImageMagick/mutool/Ghostscript then cannot create the output (rc4 failed the same
    way). Windows CI has no converters, so `a_long_figure_name_converts_as_a_short_one` skips there; render to a
    temp-dir name and install with copy-then-rename when the stage path reaches 260; 1
  - CSSSEARCHROOT — (AR5IVCSS review, pre-existing) `latexml_oxide/src/post.rs:1360-1366` puts the binary's third
    ancestor (exe/../../..) and the cwd on the resource search path, which Perl does not: a stray `ar5iv.css` or
    `CSS/LaTeXML.css` there (for an unpacked release, plausibly `~`) wins over the embedded copy and is copied over a
    user's non-empty destination file. Also: an empty file of a bundled name next to the `.tex` (issue 312's leftover)
    is copied over a non-empty file in a separate destination with 0 Warnings (DG 2026-11-25 left the source→destination
    copy unchanged; record, and revisit with Perl's `pathname_copy` date rule, `Pathname.pm:266`); 1

---

## Silent-loss inventory — blocks the 98 % claim (user 2026-10-18)

Moved 2026-11-09 (SYNC1b) to its own living worklist, [`perfect_kernel/SILENT_LOSS_INVENTORY.md`](perfect_kernel/SILENT_LOSS_INVENTORY.md): 226 rows, 220 of them open (the rest closed, accepted or no action), one `###` section per row. A "SYNC row" citation of an inventory row (the numbered items #5-#27, the frontmatter residuals, 108b, Z14, the int153a flip row, "Before the flip", "Open residuals lifted from landed rows") resolves there by the same title.

## Rust Error Fixes

Rust-only defects (a port that diverged from the Perl it translates, not a Perl-origin defect), each with its
KNOWN_PERL_ERRORS cross-reference when one applies. Perl-origin defects go to `parity/KNOWN_PERL_ERRORS.md`.

Each entry's body (defect, cause, fix, guards, witnesses) is verbatim in [`archive/SYNC_SESSIONS_2026-11.md`](archive/SYNC_SESSIONS_2026-11.md#moved-from-sync_status-by-the-2026-11-09-compaction-sync1) "From \"Rust Error Fixes\"". Titles here:

- **A Rhai `fontTest` or math-ligature matcher that assigns no longer panics on a held state borrow (RHBOR; `apply_text_ligatures`/`apply_math_ligatures` copy the closures out first, Perl `Document.pm:1174-1183, 1225-1228`)**
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
