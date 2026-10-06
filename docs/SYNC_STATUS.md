# Engine Sync Status — Active Worklist

> **DO NOT downgrade Errors to cheat the task.** If Perl LaTeXML converts a paper
> without a downgrade, the Rust translation must match by improving the core
> engine — never by silencing diagnostics. New downgrades require explicit proof
> Perl emits the same severity on the SAME paper, else they hide a real gap.
> (User directive 2026-05-15.) Always classify with `latexml --verbose`, never
> `--quiet` (which hides Perl's `Error:` lines); cross-check pathological inputs
> with `pdflatex`.

## How to read this file

**Start at "Ranked worklist" below and take the top unblocked row.** That is the
whole intent of this file; everything after it is supporting detail.

| section | what it is | when you read it |
|---|---|---|
| **Ranked worklist** | every open item, ordered, with size + where the detail lives | **first, always** |
| Current status | suite count, the last session, release state | to orient |
| Open items | the detail behind the ranked rows | when you pick that row |
| Standing policies | rules that constrain *how* you fix things | before adding a CLI flag, a stub, or a divergence |
| Parked families | pointers to extracted docs | only when starting that family |
| Reference | stable facts, not active work | when something surprises you |

Three rules that keep this file honest:

1. **Verify a status label before acting on it — and before deleting it.** Check the
   **named guard test** in the tree, or `gh issue view <N>` / `gh pr view <N>`.
   **SHA-ancestry does not work** as a check — the repo squash-merges, so a branch
   SHA quoted here is never an ancestor of `main`.
2. **This is the BRIEF ACTIONABLE LIST.** Day-by-day logs live in `git log` and
   `docs/archive/`. When you close an item, delete it here and lift anything
   worth re-reading into `docs/archive/SYNC_SESSIONS_YYYY-MM.md`.
3. **Keep it under ~500 lines.** When a section outgrows ~100 lines it has become
   its own subject — give it a doc under `docs/` and leave a one-line pointer.

*Last compaction: 2026-10-04 — the run-329 narratives (61r-62f) moved to `archive/SYNC_SESSIONS_2026-10.md`, the parked math rows to `math/MATH_PARSE_RESIDUALS.md`, landed rows removed; before: 2026-10-02 — the K13 stage-2 findings and the rc131/rc132 + 57cn–59l rows moved verbatim to `perfect_kernel/archive/LEDGER_PHASE57_58_2026-10-02.md`, their open residuals kept here. Prior: 2026-09-27 (closed R1, R2, R5 and the 2026-08-02 status snapshot to `archive/SYNC_SESSIONS_2026-09.md`); 2026-09-03 (949 → ~320 lines; 1276 → 953); 2026-08-18 (1462 → ~890); 2026-07-25 (1979 → ~500).*

---

## Ranked worklist — start here

Ordered by importance (2026-10-04 review): closing the perfect-kernel goal, then run 329 and its residuals (they seed
the coming arXiv success-rate task, not started), the rulings that block rows, the work safe to do while the run
finishes (T1, T2, F1, F2), then the older parity rows. Within a tier: **does it reproduce today** → **is a real user
affected** → **is it unblocked** → **effort**.

| # | item | state | size | detail |
|---|---|---|---|---|
| **R8** | **Perfect-kernel program** (branch `perfect_kernel`; the TL-manual corpus; generalized capabilities K1–K18 in [`perfect_kernel/KERNEL_CAPABILITIES.md`](perfect_kernel/KERNEL_CAPABILITIES.md)) — goal bar met at sweeps #146-148 (G1–G5); close status in [`PERFECT_KERNEL.md`](PERFECT_KERNEL.md) | **CLOSING**: bar confirmed (user 2026-10-04); ar5iv-css PR #54 open; run 329 completes | program | [`PERFECT_KERNEL.md`](PERFECT_KERNEL.md) |
| **G** | **Run 329** (full arXiv, cortex, 2.9M papers, on `cortex-worker-62c`) — compare to run 306 and cluster when complete; this opens the arXiv success-rate task | **PAUSED** 2026-10-05 02:29Z by the user at 263,636 done (Error/Fatal study → 62i-62n); fixes 62d-62n not deployed. Held until the ar5iv update finishes (user 2026-10-05); user to decide: resume 329 on 62c or a fresh run on a 62n worker, and whether to keep 62k's `localrawclasses` | program | [`PERFECT_KERNEL.md`](PERFECT_KERNEL.md) close status |
| **G1** | **Run-329 open residuals** — 17 PushbackLimit Fatals (csvsimple + siunitx `S`), TooManyErrors / never_completed, the `gullet.rs` `handle_template` latent panic, the runtime-bindings `.rhai` fallback, amsppt-as-plain, `.aux` re-read, eplain, calc's `!`, singles | **NEXT** (arXiv success-rate task) | medium each | §"Run-329 and sandbox open residuals" |
| **D1** | **Rulings**: RULED 2026-10-04 — implement tex.web §392 `\par` in a non-`\long` argument (1001.1670) and restore Perl's `Error:malformed` for DIVERGENCES #189 (prerex section-in-figure): 62g landed (#189: 7 in-scope manuals now report it — biblatex, keytheorems-doc, pdfmarginpar, phonrule-doc, prerex, srdp-mathematik, zx-calculus — so G1 counts 7), 62h landed (§392, DIVERGENCES #442 / KPE #472-473). Still pending: `\obeylines` `^^M`; `{framed}` item shape; biblatex `cite.<n>@<key>` surpass anchors (D15); R7 port-or-drop | RULING | small-medium | rows cited |
| **T1** | **Repro suite move**: `tools/perfect_kernel/repros` (1,469 files) → `latexml_oxide/tests/repros` header-contract trials (release CI + `LATEXML_FULL_TESTS=1`); drain `tools/` | **OPEN**, not started (user 2026-10-02) | large | memory `project_repro_suite_move` |
| **T2** | **Test API stage 2**: 37 test files still spawn binaries (`Command::new`; cluster_cli 33, cluster_xslt_split 13 …) → the in-process convert API; five `[[bin]]`s remain | **OPEN** (stage 1 landed 61a) | medium | `~/data/pk_agents/main/HANDOFF.md` (61a) |
| **F1** | **Per-encoding text commands**: textcomp's symbols now dispatch on the encoding (62r, KPE #489; repro `fonts-nfss/textmu_follows_the_encoding` GREEN). Residual: the kernel letters keep Perl's definedness gate (a T1 fontmap slot shared by `\DH`/`\DJ`), and `\<E>-cmd` chains ignore `\cf@encoding` | **PARTIAL** | medium | §Font-Selection Chain Residuals |
| **F2** | Small RED repros: hyperref pdfinfo values expanded as by `\pdfstringdef`; `\vadjust` queue scoped to its list (60k, TeX nest; low return); the rest under §"Perfect-kernel RED repros" | **OPEN**, safe now | small each | §Perfect-kernel RED repros |
| **R3** | **Bibliography-absence campaign** (PR #444) — **291 recovered / 20 338 entries**. Remaining unblocked: **R3d tab-mark parameter scan vs cell read** | **R3d next** (12 papers left, unblocks alignment macro `&` splits) | medium | Open items §R3, [`RESIDUAL.md`](parity/bib_absence_2026-07-29/RESIDUAL.md) |
| **R8b** | **forest.sty full support** (side goal, user 2026-09-05; heavily used on arXiv) — native semantic tree since 56iz (nested inline lists, labels as TeX, node keys; DIVERGENCES #302); open: drawing/layout fidelity | **OPEN**, recorded | large | [`perfect_kernel/DIFFICULT_CASES.md` §D10](perfect_kernel/DIFFICULT_CASES.md) |
| **R4** | `--preload=<cls>` trips the LaTeX hook stack (`Extra \PopDefaultHookLabel`) | **OPEN**, re-verified (1 error with `--preload=article.cls`, 0 without). Pool load reordering | medium | Open items §R4 |
| **R6** | **`Collector::rescan` Refactor for Generated Backmatter** | **OPEN**; `Scan` owns `ObjectDB` by value; generated backmatter lacks full relations/labels | medium | Open items §R6 |
| **R7** | Presentation-MathML **F5** Linebreaker | **OPEN**, full linebreaker feature gap needing a port-or-drop scope decision | family | Open items §R7 |

---

## Current status

The corpus measure is the perfect-kernel scoreboard ([`PERFECT_KERNEL.md`](PERFECT_KERNEL.md), sweeps of the 2,374
TL manuals) and the per-batch arXiv A/B; the cortex reruns of sandboxes 2605/2606 (runs 325-328, 2026-10-04) and run 329's validations are in
`archive/SYNC_SESSIONS_2026-10.md`; the last full sandbox rerun before them (2026-08-02, 60,505 docs) in
`archive/SYNC_SESSIONS_2026-09.md`.

---

## Open items — detail for the ranked rows

### R3 — Bibliography-Absence Campaign (PR #444 Residuals)
- **R3d: Alignment Parameter Scan vs Cell Read Distinction (`suppressed_tab_marks`):**
  - *Symptom:* An unescaped `&` inside a delimiter-fenced macro argument splits the alignment row and truncates the document (and bibliography). 12 papers remain affected.
  - *Mechanism:* `tex.web` §394 `macro_call` suppresses tab marks while scanning parameters. `SuppressedTabMarks` in [`latexml_core/src/common/local_assignments.rs:194`](../latexml_core/src/common/local_assignments.rs#L194) fixed `physics.sty`'s `\mqty` (14 papers), but applying it globally to `Parameters::read_arguments` regressed 5 tests (`cells_test`, `numprints_test`, `xytest_test`, `consort_flowchart_test`, `unit_tests_by_silviu_test`) because that path also reads alignment cell content.
  - *Action:* Distinguish macro parameter scanning from alignment cell reading in `Parameters::read_arguments` so tab marks inside `{...}` do not split outer cells.
- **R3b: No-Diagnostic Chase Candidates (~6 left):**
  - Remaining papers with real `\cite` calls but empty bibliography: `2605.14990`, `2606.05629` (math-in-body silent drop), `2606.10056`, `2606.17491`, `2606.00231`, `2605.29754`.
- **R3g: amsrefs Bare `\begin{biblist}` (4 papers):**
  - `\begin{biblist}` without `{bibdiv}` wrapper $\to$ `malformed:ltx:biblist`. Requires `BACKMATTER_ELEMENT` route.

### R4 — `--preload=<cls>` trips the LaTeX hook stack (`Extra \PopDefaultHookLabel`)
- **Symptom:** `--preload=article.cls` prints `LaTeX hooks Error: Extra \PopDefaultHookLabel` (clean without preload or with `LATEXML_NODUMP=1`).
- **Mechanism:** `\@pushfilename` changes meaning mid-load: `article` is pushed before `LaTeX.pool` loads, using a pre-pool `\@pushfilename` that does not touch `\g__hook_name_stack_seq`. The pool installs expl3's `\@popfilename`, which pops an empty seq and errors.
- **Resolution:** A TeX-side repair or re-synchronizing the sequence at the point `LoadPool('LaTeX')` executes.

### R6 — `Collector::rescan` Refactor for Generated Backmatter
- **Symptom:** Generated subtrees (Bibliographies, Indexes, Glossaries) lose ObjectDB relations, labels, and fragids because `Scan` owns `ObjectDB` by value (`latexml_post/src/scan.rs:49`) and cannot rescan generated nodes.
- **Resolution:** Refactor `Scan` to borrow or take/restore `ObjectDB`, enabling clean rescanning of generated nodes and removing fragile ad-hoc registrations.

### R7 — Presentation-MathML F5 Linebreaker
- **Status:** The only remaining pMML gap from the MathML line audit (F17 is closed). A full linebreaker algorithm gap requiring a port-or-drop scope decision before coding.

---

## Secondary residuals & unranked active items

### Open residuals of closed rows (R1, R2, R5; details in `archive/SYNC_SESSIONS_2026-09.md`)
- **R1:** the 117-paper `TooManyErrors:MaxLimit(100)` cluster the seeds came from (2026-08-02 rerun) has not been
  re-measured; its seeds `2605.22927`, `2606.11121`, `2606.01136`, `2605.10685` are clean since 56ks3/56kt. The same
  rerun's `Stomach:Recursion` cluster (55 papers, MIXED) had three REAL-by-count witnesses never followed up:
  `2605.17696` (Rust 144 / Perl 56 errors), `2606.05321` (35 / 15), `2606.08524` (94 / 50).
- **sn-jnl (DIVERGENCES #333):** newer class copies (2404+, e.g. 2605.00003 L1280-1321) define `\toprule`/`\midrule`/
  `\botrule`/`\cmidrule` themselves instead of loading booktabs; neither Perl nor the binding runs the class code, so a
  paper relying on them without its own `\usepackage{booktabs}` keeps `\toprule` undefined (parity).
- **Equation layout (DIVERGENCES #334):** `eqnarray` after a class's `fleqn` + amsmath is flush left in pdflatex, centred
  here (`ltx_fleqn` is document-wide). `document_class_filename` (content.rs) is overwritten by nested class loads, and
  xkeyval's fallback reads it (`xkeyval_sty.rs:1524`). elsarticle's `\ifpreprint` is always false here; elsarticle.cls:112
  makes `preprint` the default, and `5p`/`3p`/`1p`/`final` clear it (:71-87).
- **Cropping (DIVERGENCES #337):** `reflect` is not applied; a negative trim (padding) is clamped, as in Perl; anisotropic
  resolutions crop by the x one; the job key holds the full options, so `[trim=X,clip,width=3cm]` and `[…,width=5cm]`
  crop twice into identical files.


### Font-Selection Chain Residuals
1. **`\cal ABC` collapses to one `<mi>`**: Drops `class="ltx_font_mathcaligraphic"` (Perl emits three `<mi>` elements, Rust one containing `𝒜ℬ𝒞`). Token grouping and class diverge.
2. **`\DeclareTextCommand`/`\ProvideTextCommand` lack encoding-dispatch chain**: `latex_constructs.rs:6525/6544` bind bare `\cs` to first-encoding expansion permanently.
3. **Minor font registration gaps**: `\DeclareTextSymbol` decodes eagerly instead of installing deferred `CharDef`; `\DeclareErrorFont` is a bare no-op where Perl defines argument as `\relax`.
4. **A preamble font switch stays in force in the body** (W17, DIVERGENCES #315; shared with Perl): latex.ltx's `\@kernel@after@begindocument@before` also runs `\init@series@setup` (latex.ltx:13902-13920: `\reset@font`, `\mdseries`, `\let\seriesdefault\f@series`), which Rust does not run. `\documentclass{article}\itshape\bfseries\begin{document}Body text.` — pdflatex upright CMR10; Rust and Perl `<text font="bold italic">`.

### Fragid Parity Open Items
1. **`associateNode` (Post.pm L508-585) unported**: Generated MathML/OpenMath nodes carry no `xml:id`, so `convertedIDs` and pmml↔cmml parallel cross-linking do not exist.
2. **`in_page_id` lacks `labelids` and `split_from_id` branches**: Affects `--splitnaming=label*`.
3. **`strip_ref_display_fragids`** (crossref.rs:131): Matches `//ltx:ref//*[@fragid]` wholesale; narrow to IDs absent from the ObjectDB.
4. **`make_sub_collection_documents` returns `vec![]`** (collector.rs:141): `--splitindex`/`--splitbibliography` drop entries past first initial.

### Corpus Triage Quick Wins
- **A paragraph's trailing whitespace: Perl and Rust split its text into different nodes** (56jy; RED repro `tools/perfect_kernel/repros/block-model/paragraph_trim_text_node_split.tex`): both trim with `s/\s+$//` (all Unicode whitespace; TeX_Paragraph.pool.ltxml:184-196, `document.rs` `trim_node_right_whitespace`) and only the last text node. Perl appends the space after a formula with XML::LibXML `appendChild` (Document.pm:1148-1153), which never merges, and a formula's own text with `appendTextNode` (Document.pm:2123, :2133), which merges into the text before it; Rust's `add_child` (xmlAddChild) merges the space, and `replace_node_as_tree` keeps the formula's text a node of its own. `\item $\,$ ` gives Perl `<p>U+2006</p>` (its golden `t/parse/artefacts.xml`), us `<p/>`; `\item A $\,$\par` Perl `A`, us `A `. `cleanup_Math` runs as the Math element closes in both (Document.pm:1239-1248, tex_math.rs:756), so ordering is not the cause. Cosmetic; a faithful fix appends text without merging, which touches many goldens. Where the trim empties the last text node, Rust frees it and Perl keeps it empty, which Perl's `isVAttached` then counts (KNOWN_PERL_ERRORS #311).
- **Test-suite hygiene (57z)**: the logger echoes through `eprint!` (`logger.rs` `stderr_echo`), so `cargo test` captures every test's log lines with its output (a direct `std::io::stderr()` write bypassed libtest's capture and leaked all of them to the terminal); every passing test that prints `Error:`/`Fatal:` exercises an error path and pins it by count (inventory 2026-09-28: 84 of 3,634 tests; incidental fixture errors removed, loose `> 0`/`contains` checks made exact); the streaming sweeps require each swept fixture to convert error-free (the math sweep has its dispatcher; intentional-error and preload-needing fixtures are excluded); `latexml::util::test::error_count` counts every `Error:<category>:` (`missing_file`, `I/O` were missed). Found by the inventory: LANDED 57aa `expl3/tex_jobname_is_the_jobname` (the dump wrote `\tex_jobname:D`, a `\let` of `\jobname`, by value — empty; now, as Perl's `Lt`, a `\let` copy identical to its source is an alias record re-`\let` at load; copies a preload takes before the job's `\jobname` exists are re-pointed at install, `core_interface::install_jobname`: `\c_sys_jobname_str` is the job name with and without a preload; RUST-ONLY; DUMP_DESIGN "Format"); open: RED `boxes-groups/nicefrac_text_argument_is_text` (nicefrac's parts are text in `\mbox`, the binding digests them in math; PERL-ORIGIN); forest binding gaps `\@escapeifif` (forest.sty:203) and `\forest@file@copy` (:8430), used by forest-index.sty (RUST-ONLY, stub binding), and `\lst@InstallKeywords` (lstmisc.sty:618, lstdoc.sty:178; SHARED) — pinned by `perfect_kernel_batch56::forest_docinput_lstenv_writefile_gobbles_doc_percent`; the parked pTeX-format registers (`\hour`, `\jfam`, `\kanjiskip`, §D9) behind jsarticle/jarticle/otf; tests that still mutate the process env (race under `cargo test`): `117_post_2gib_handoff.rs:82` (`LATEXML_POST_MEM_PARSE_LIMIT`), `118_streaming_split_parity.rs:142-216` (`LATEXML_POST_STREAM_SPLIT`/`_THRESHOLD`), `identity.rs:113` (`SOURCE_DATE_EPOCH`) — migrate to thread-local overrides as `datatool_sty`/`pgfkeys_code_tex::set_native_override`; RED `loader/package_options_keep_protected_macros` (a robust or e-TeX `\protected` macro in a package option is expanded on the option path: pdflatex stores `k=\protect \lxr`, `j=\lxp`; Rust `k=\protect \protectR`, `j=P`; the chinesechess guard `package_options_are_stored_by_protected_xdef` no longer discriminates).
- **simplebnf-doc warns `unexpected:\fatslash … should only appear in math mode`** (sweep #130, 0 → 1; new since sweep #129): stmaryrd's `\fatslash` (loaded `\AtBeginDocument` with `[only,fatslash]`) inside the manual's tcolorbox `example` re-typesetting (`lx-example-1.tmp`); a minimal `bnf` + stmaryrd probe is clean in both builds and Perl, so the trigger is the example's re-read. Likely 57n's requireMath for every DefMath constructor meeting a text-mode re-read; confirm against Perl on the manual before fixing.
- **An index/glossary entry is digested at its mark** — the `&` case LANDED 59k (a literal `&`; KPE #423, DIVERGENCES #411; robustsample 2 → 0; `index/glossary_ampersand_robustglossary` GREEN). Open (ARCHITECTURE_THEMES §4a): TeX only writes the entry and re-reads it at `\printindex`; a phrase inspected after `do_expand_partially` (mod.rs) has lost `\begin`/`\(`, so arrays in index entries stay unhandled.
- **biblatex `editortype` is not the editor's role** (56kc review; RED repro `tools/perfect_kernel/repros/index-bib/biblatex_editortype_role.tex`): every editor prints "(Ed.)" where biber prints "vocalist Billie Holiday" / "Kilo, Karl, comp." (biblatex.def:2875-2892 `editorstrg`). Giving the role in the BibTeX reader (tried in 56kc) broke the author-year label of author-less entries ("(Hitchcock, 2000)" → "(52)", cms-dates-sample; the fallbacks at make_bibliography.rs ~1076/1212/1354/3640 read only `@role='editor'`), changed `.bst` bibliographies, and exploded an undecoded (ctex) `editortype` byte string. Shape: in the biblatex binding, keep the editor for label/sort/head, carry the type separately, from the decoded field value.
- **A template with no column raises "Extra alignment tab" on every row** (56jz root-causer; SHARED with Perl, Alignment.pm:136-145/240): `\begin{tabular}{}` (or `{|}`) gives an error per row start even without `&`; TeX's preamble always has a column (tex.web:15367-15375, "Missing # inserted" :15468), array adds one for an empty preamble with "Empty preamble: `l' used" (array.sty:332, 412). Shape: in `read_alignment_template`, no columns and no repeated columns → one `l` column plus that error. Witnesses platex-tools/plextarray, plextcolortbl (plext's `\begin{tabular}<t>{…}` direction argument, pTeX family, parked).
- **Measured box sizes, K18 step 2** (step 1 landed in 56kj; repro `tools/perfect_kernel/repros/boxes-groups/box_dimensions_measured.tex` GREEN): `\raisebox`'s raise measures the box since 61v, with `yoffset` rendered as `\raise` (KPE #466; repro `boxes-groups/raisebox_raise_measures_the_box.tex` GREEN). A `\makebox` in script math measures its text at the script size (RED repro `boxes-groups/makebox_in_script_math_text_size.tex`, 2605.18608). `\resizebox{\width}{!}{…}` is `xscale="0"` and `\resizebox*` behaves as `\resizebox`: `\Gscale@@box`'s `GraphixDimension` sizes are not yet read with the box set, as 56kl's `TempboxaDimension` are (4 papers). `\includegraphics[height=1em]` measures 10.1178pt (pdflatex 10.00002pt): its size goes through whole pixels at 100 DPI (RED repro `graphics-tikz/includegraphics_size_not_pixel_rounded.tex`). An empty `\raisebox{d}[]{x}` reads 0pt with Missing number where latex.ltx treats it as not given (0 uses; RED repro `boxes-groups/raisebox_empty_height_is_not_given.tex`). `\resizebox{1em}{!}{x}` measures 10.00002pt, Perl's, where pdflatex gives 10.00081pt through graphics.sty's `\Gscale@div`. makecell (`makecell_sty.rs:192-193`), diagbox (`diagbox_sty.rs:288-289`) and rotate.sty's `\rotate[u|f]` (`rotate_sty.rs:43-45`, Perl rotate.sty.ltxml:44-46 stores Dimensions) still store string sizes, which `compute_size_and_cache`'s complete-size bypass (56kq) cannot read, so the box is measured and reports its unrotated content; rotating.sty shares `rotated_properties` and is typed. Still unported from Perl's `computeSizeStore` (Box.pm:266-297): a partly specified size keeping its given parts over the computed ones, and `isEmpty` (`docs/archive/UPSTREAM_SYNC_2767_to_2833_2026-06-26.md` U2/S5). `set_width`/`set_height`/`set_depth` (`latexml_core/src/lib.rs`) write only the declared value, where Perl's `setWidth` etc. (Box.pm:211-230) also update the computed copy, and `fobj_get_size` (`tex_box.rs:75-118`) prefers a stored computed size and never computes a missing one (Perl `$whatsit->getSize`, TeX_Box.pool.ltxml:410): a `\ht` assigned after a size was computed stays stale for a foreignObject unless the box's size is complete (56kq's bypass then applies). Needs a ruling: `\ ` is 0.5em wide (Perl's constant, TeX_Character.pool.ltxml:28-30) where TeX uses the font's interword space (`\spaceskip` if nonzero, tex.web §1041-1044): a node with a control space is 0.167em per `\ ` wider than pdflatex. Also open: a detector for a String-valued size key in `compute_size_and_cache`, and an audit of box constructors without a sizer (79 `sizer =>` sites against 945 `DefConstructor!`). See `docs/perfect_kernel/KERNEL_CAPABILITIES.md` K18.
- **babel-french high punctuation** (residual of Gemini round 13, KPE #383): a space inside a closed group (`{\bfseries gras }?`) stays; a negative skip is removed where TeX removes only one above 1sp.
- **K13 stage-2 findings: open residuals** (57h audit, `tools/perfect_kernel/binding_conformance.sh`; the landed rounds
  57i-57y, 57ad, 57ah, 57aj and Gemini round 13 are verbatim in `perfect_kernel/archive/LEDGER_PHASE57_58_2026-10-02.md`;
  verify each against the real `.sty` before fixing). K13 stage 3 (environments' `\endX`, classes, the allowlist) not started.
  - *setspace/titlesec*: RED `block-model/setspace_display_math_in_doublespace`; titlesec — an alignment switch in the
    before-code aligns an inner `<text>` (#344), a `leftmargin` after-code ends the title where TeX starts the next
    paragraph with it, `explicit` as a global class option unseen, `uppercase` (`\ttl@case`) unread, keyed
    `\titleformat{name=\section,numberless}` defines a junk macro (13 A/B papers: googledeepmind.cls, mystyle.cls); RED
    `sectioning-frontmatter/unnumbered_section_takes_the_title_font` (`\@@unnumbered@section`, Perl
    latex_constructs.pool.ltxml:666-670; ~70 A/B papers; needs tufte's `\titleformat` `[after]` first).
  - *floats/captions*: RED `captions-floats/par_breaks_a_panel_row`; RED
    `captions-floats/continuedfloat_minipage_subcaption_continues_the_letters` (no `\caption@subtypehook` pre-step,
    caption.sty:642); under subfig a third continued part keeps the first continuation's sub-float id suffix (`S0.F1a.sf4`
    under `S0.F1b`); `[caption=false]{subfig}` still gets `\caption@ContinuedFloat`'s check; caption's
    `continuedfloat`/`continued<type>` option sets (caption.sty:526-528) unapplied; RED
    `captions-floats/longtable_starred_caption_keeps_its_text` (KPE #363); longtable ignores `\LTcaptype`; floatfig/JHEP
    `floatingfigure`/`floatingtable` and elsart `algorithm` typed without a source; RED `captions-floats/subref_prints_the_letter`;
    `subfloat_sty.rs:29-30` `\fnum@sub…` = `(\thesub…)` gives "((a))" under `labelformat=simple|empty` (2605.01267,
    2605.10017, 2605.05371; `subfig_sty.rs:128` moved to `\lx@subfig@fnum` in Gemini Q8 — re-verify which still
    double-wraps); jfm.cls/interact.cls `\doi` setters (KPE #326); the XSLT DOI-pubnote predicate (DIVERGENCES #345) has no
    HTML guard; `\rotcaption*`/`\rotcaptionof` undefined, `\controtcaption` unnumbered (57l); a `\rule{\linewidth}` panel's
    `width="100%"` is unmeasured by the panel rows (57by, 2605.18774; KPE #389).
  - *loader/kernel*: neither the loader nor `\@ifpackageloaded` honours a `\ver@<pkg>.sty` mark (2605.20200; memoir
    `\EmulatedPackage`; honouring it broke 2605.08402's `\listoffigures*` on the 57bu A/B); Rust's `\@addtoreset` omits
    Perl's `defCounterID … unless \the<ctr>@ID` (latex_constructs.pool.ltxml:3044; 57ac).
  - *bibliography*: natbib's `\bibpostamble`; KOMA's `\setbibpreamble` without natbib; apacite's citation layer is natbib's
    (`\cite` textual, `\citeA` abbreviates).
  - *anchors and hooks*: RED `block-model/hyperdef_starts_the_paragraph`, RED
    `block-model/hypertarget_heading_a_paragraph_keeps_the_space` (both shared); math in a new anchor takes ids chained from
    the anchor's (`m.m1`); RED `macro-state/afterendpreamble_code_is_followed_by_ignorespaces` (RUST-ONLY); RED
    `macro-state/atbegindocument_in_the_body_runs_now` (fire `\document`'s hooks with `\hook_use_once:n`, then drop 57r's
    `\AfterEndPreamble` flag).
  - *layout and lists*: RED `boxes-groups/rotatebox_starts_the_paragraph` (shared); RED `fonts-nfss/ulem_custom_mark_is_rendered`
    (41 of 30,079 2605 papers; the class mapping is a design call); RED `block-model/noindent_paragraph_in_a_minipage_takes_its_id`
    (same defect as 58g's RED `noindent_paragraph_in_minipage_numbers_inside` — merge the two); RED
    `fonts-nfss/textcircled_of_a_box_circles_its_text`; RED `list-structure/enumitem_ref_clears_the_parent_prefix`
    (enumitem.sty:551-556); RED `list-structure/endtrivlist_after_a_heading_starts_a_paragraph` (61p review; OD #435
    residual); RED `string-mouth/verbatim_sty_starred_shows_visible_spaces` (verbatim.sty's `{verbatim*}` without its
    visible-space setup; SHARED); level-only `\setlist[1]` (re-verify after 57u's KPE #354); `\optc[x]` in a message prints `\optc [x]` (KPE #381
    residual). Settled dead end: a separate counter family for kernel lists fixes the label but changes Perl's ids.
  - *57ah residuals*: svn-multi's keywords are pdflatex's pass-2 `.aux` values; savetrees does not load titlesec/geometry for
    `sections`/`margins`; `\MakeUppercase{\authors}` leaves the kernel `\and`'s " and " lowercase; varioref has no
    per-language range words; RED `sectioning-frontmatter/amsart_uppercasenonmath_textcase_keeps_the_title` (textcase
    `\altucnm`, SHARED); RED `macro-state/kernel_Ref_is_a_capitalised_reference`.
  - *math (PARKED 2026-10-02; verbatim in the archive)*: RED `math-parse/left_delimiter_outside_the_map` (~400 warnings, 7
    papers via `\delimiter`), RED `math-parse/differential_needs_its_integrand`, RED
    `math-parse/lxdeclare_matches_a_font_styled_symbol`; the 57ag route-dependence and grammar residuals (2605.11385 S2.E18,
    2605.02221, 2605.30618, `x_1,x_2,x_3\in X`, `\otimes_k E:=F`, `[n]\setminus\cup_k C_k` 71 formulas/12 papers,
    `<r_i>_n^*`, `x^{\prime,}`, `x_{a,\qquad}`, `f(x)+\phantom{g(x)}+h(x)`, `gathered` `vattach`, `ams/mathtools`
    S10.Ex76's `\qquad`, `\sum_k\beta\log\varepsilon_k`, `\max_i\max_j a_{ij}`, `\operatorname*{argmin}_z`); 57ae's
    `\DeclareMathOperator{\Tr}{Tr}` lacks Perl's `name="Tr"`.
  - Kept as is: MnSymbol/fdsymbol `\not` (2605.05506; `\mathrel` wrapping is a dead end: ≠ → `not@(=)`), fontawesome
    `\faicon`, the superset bindings (nameref `\Nameref`, subfigure `\subref` `*`, lineno `\linerefp`); walker limits
    (subfig `\DeclareSubrefFormat`, apacite `\BCAY`, soul `\caps`, eufrak `\mathfrak`).
- **A block box at a paragraph start splits the paragraph** (56kb review; SHARED): `\rule`, `tabular` first in a paragraph (latex.ltx 16361, 16560 start with `\leavevmode`) come out as separate blocks, and the text after them ("et B.") in a paragraph of its own. `\parbox` (16250) fixed in 57d and `minipage` (16306) in 57f (KNOWN_PERL_ERRORS #309, DIVERGENCES #338).
- **Cells at a column rule or `@{..}`: alignment and padding classes** (56jy review; RED repro `tools/perfect_kernel/repros/alignment/column_rule_edge_padding.tex`): a leading arydshln `:` right-aligns the first `c` cell (Perl `align="left"` with `ltx_nopad_l`; pdflatex centres it), and the cell keeps the rule's U+2002 U+200A gap as leading text, doubling the CSS padding; `c@{\,}c` marks no `ltx_nopad_l` on the right cell where Perl does. Shared with Perl: a `c` column before `:`/`;` reads `align="right"` `thead="row"` (Perl's right-to-left cell scan stops at the class command, TeX_Tables.pool.ltxml:505-520), and the `@{..}` material itself is dropped.
- **Scanner-status residuals** (batch 56jg, DIVERGENCES #310): commands Rust implements as primitives/constructors (`\setcounter`, `\lstnewenvironment`, `\newtcbinputlisting`) need a primitive-level abort for TeX's matching recovery; `skipping` is not modelled.
- **A file end in a `GeneralText`/`XGeneralText` brace hunt is not an error** (batch 56jj residual, DIVERGENCES #313): `\uppercase`/`\lowercase`/`\detokenize`/`\message`/`\errmessage`/`\mark`/`\marks`/`\special` as a file's last token — pdflatex 2 errors ("File ended while scanning text of \uppercase", "Missing { inserted") and "A abc B", Rust and Perl 0 and "A ABC B". Order: stop setting `matching` around expandable primitives' typed parameters (`Expandable::read_call_arguments`, `definition/expandable.rs:325-340`; `\csname`, `\number`, `\readline`), then let `read_x_token` call `recover_at_file_end` at non-normal status; the brace hunt (`skip_filler`) also runs at the caller's status and skips no `\protected` expansion, where `read_tokens_value` hunts with protected expansion — one hunt for both. Red repro `tools/perfect_kernel/repros/expansion-primitives/uppercase_brace_hunt_file_end.tex`.
- **A braced typed argument's rest: the residuals** (batch 56jr resolved the silent loss, OXIDIZED_DESIGN_DIVERGENCES #317 / KNOWN_PERL_ERRORS #275: the rest re-enters the input after the command, assignments scan by the register's type, calc — loaded in most arXiv papers — evaluates a braced length whole; an undefined control sequence the scan met is discarded, as TeX does, and a rest met between alignment rows is dropped with a warning: the review regressions 2605.25073, 2605.08378, 2605.27476, 2605.25272 are fixed; batch 56ju fixed the TL-manual A/B regressions, each a binding that read less than TeX — multido's Number variable, nicematrix `X[<keys>]`, tabularray `\NewColumnType` and rule options, pstricks angles read whole, `\resizebox` lengths through calc, calc loaded by the pdfcomment/diagbox/animate/savetrees/breqn/jmlr bindings as by their packages, `\DeclareTextAccent`'s stored slot). Open: `\width`/`\height`/`\depth`/`\totalheight` are `0pt` inside `\resizebox` (sect12.rs:184-187, shared with Perl; perfectcut's delimiters get `xscale="0"`); tabularray's `cmd=` key is unapplied; a box or space command's tail comes one step late — `A\hspace{1em\foo}B` gives "A xB" with a warning (pdflatex "Ax B"), inside a `minipage`/`tabular*` body for those; `\vspace` is Perl's `\vskip #2\relax` (latex.ltx:9362-9372 sets `\sp@ce@skip` first); a keyval value's rest (`Parameter::reparse`, keyvals.rs) is still dropped; a column type's rest (`p{3cm\foo}`) is dropped with a warning where TeX typesets it in each cell (2605.19386 `;{1pt/1pt}` under the arydshln stub); a `DigestedBody` constructor's (`\@@tabularx`) comes after the whole body; `\lx@@genfrac`'s `after_digest` (amsmath_sty.rs) reads its numerator after a re-inserted rest. Repros `tools/perfect_kernel/repros/expansion-primitives/braced_*`, guards `braced_quantity_tail::*`.
- **`\ifx` ignores `\protected`/`\long`/`\outer`** (W17 final review, pre-existing; shared with Perl `Core/Definition/Expandable.pm:116-120`): `latexml_core/src/definition/expandable.rs:107-110` `PartialEq` compares only parameters and expansion, where tex.web §507 also compares the command code and eTeX's `protected_token`. `\def\a{x}\protected\def\b{x}\long\def\c{x}` + `\ifx\a\b`, `\ifx\a\c`: pdflatex "FF", Rust and Perl "TT". RED repro `tools/perfect_kernel/repros/macro-state/ifx_ignores_macro_prefixes.tex`. The fix has to keep `x_equals` callers that compare binding copies (`\bibitem`/`\restoring@bibitem`, `\document`/`\lx@orig@document`) equal.
- **`\shipout`/`\setbox` residuals** (batch 56jk, DIVERGENCES #314): `read_box_operand` invokes a non-box operand (`\shipout A` typesets "A", `\shipout\hrule` a rule) where TeX §1084 reports "A <box> was supposed to be here" and backs the token up; a shipped `\hbox` in horizontal mode runs into the surrounding text (emit it as its own unit, as a `\vbox` yields an inline-block); `box_prefix_tokens` (`tex_box.rs`) reads `\everyhbox` before the `\afterassignment` token, where TeX reads the after-token first (§1083 then §1269 back_input; TeXbook p.279) — Perl the same (TeX_Box.pool.ltxml:170-174), a KNOWN_PERL_ERRORS candidate once a pdflatex probe pins it; `repros.sh:63` parses `% status: RED (…) -> GREEN (…)` as RED (98 repros use that form), so their regression check is off.
- **doc.sty's code-line index and the index-list residuals** (batch 56js, DIVERGENCES #318): a dtx manual's index lists only its `\index`-level (usage) entries — doc.sty's `\codeline@wrindex` (doc.sty:855-862), hypdoc's `\HD@codeline@wrindex` (hypdoc.sty:332-343) and l3doc's `\__codedoc_index_page_hc:nn`/`_codeline_hc:nn` (l3doc.cls:1899-1914) write `\indexentry{…}{…}` to `\@indexfile` themselves, which is allocated but never opened, so the text goes to the log (source2e: 44,246 `\indexentry` lines). A `\write` to an index stream could become an `ltx:indexmark` read by the same makeindex reader. Also open: imakeidx documents get no index at all (imakeidx.sty:147-156 redefines `\index` to write the file itself; e.g. assoccnt_doc, atableau); robustglossary's `&`-column `\glossary` entries raise `Stray alignment "&"` at the mark (robustindex/robustsample, 2 errors; pdflatex prints them through its own `\glossaryentry` table; RED repro `tools/perfect_kernel/repros/index/glossary_ampersand_robustglossary.tex`); xindy styles' separators are read as text (makeglos.xdy `:`); `\@SpecialIndexHelper@` keys of control symbols stay garbled (amsldoc `\cn{\\}`). The re-read of the written entry (a `\` plus letters make a control word when `\printindex` inputs the `.ind`) is modelled only for `\string\verb`: amsldoc.cls:99-103's `\string\texttt{#2}` prints `“texttt…` — RED repro `tools/perfect_kernel/repros/index/index_string_command_amsldoc.tex`; one re-read step after the entry's expansion (skipping the absorbed `\@internal@text@verb` arguments) would also fix doc.sty's `\LeftBraceIndex`/`\RightBraceIndex` displays. `\verb*` in an entry shows no visible spaces; under hypdoc the encap style is `hdpindex`/`hdclindex` rather than doc's `usage`/`main`. A list-of-figures entry loses the space after its number (`1Cap`; Perl and pdflatex `1 Cap`): Scan stores `caption`/`toccaption` as text (scan.rs `captioned_handler`), which drops the tag's `close`; storing the `cleanNode` copy as a node, as Perl does, would keep it (repro `index/indexmark_cleaned_from_titles_captions.tex`).
- **A `\gls`/`\glsadd` inside display math lists nothing** (batch 56jt residual, DIVERGENCES #320): the location-only `ltx:glossaryref` of a formula goes to the nearest enclosing level that admits it, after the formula; inline math has its `p`, but `ltx:equation` admits no inline element, so an entry named only in display math (`\begin{equation} \gls{E}=mc^2 \end{equation}`) is missing from the list. `\aftergroup` carries tokens past `$…$`/`\[…\]` (probed), but not safely out of an alignment cell.
- **`{filecontents}{name}` overrides a `name.tex` on disk** (batch 56jt residual, KNOWN_PERL_ERRORS #279): since `\openout`/`{filecontents}` name the file `name.tex`, the source's contents shadow a `name.tex` beside the document, where LaTeX's default (no `[overwrite]`) keeps the disk file ("File `name' already exists on the system. Not generating it from this source.", latex.ltx:19004). Probe: `democode.tex` = "Disk version." + `\begin{filecontents}{democode}Source version.\end{filecontents}` + `\input{democode}` — pdflatex and the pre-56jt binary print "Disk version.", now "Source version.". No fix yet: the store must consult the disk before a non-`[overwrite]` write.
- **Fixture exception to the 0-warning rule** (batch 56jt): `stream_a_recall::a_citation_of_another_lists_item_is_missing_here` asserts its one warning, "Missing bibkeys: supp" — the faithful diagnostic (Perl MakeBibliography.pm:342-343; bibtex warns "I didn't find a database entry"), since the key it guards is cited and absent from that bibliography's files by construction.
- **biblatex refsections share one citation list** (batch 56jt residual, KNOWN_PERL_ERRORS #277): each bibliography now reads its own `@files`, but every `\printbibliography` fallback lists every cited key its files hold and warns "Missing bibkeys" for the rest, where biber lists and checks only its refsection's citations (RED repro `index-bib/biblatex_refsections_share_citations.tex`: 2 warnings, Globalentry in the first list); `\printbibliography[section=N]` after the refsections reads the pooled resources (biblatex-apa6-test).
- **babel language tags are lost under the `luatex` profile** (batch 56ji residual): babel loads `luababel.def`, so `xml:lang` is not set; `babel_support_sty.rs:243-254` also forces LGR for Greek (latent). Red repro `tools/perfect_kernel/repros/babel-lang/luababel_language_tag_luatex.tex`.
- **Rust `replace_tree`/`replace_tree_detach` keep the replaced node's box in its parent** (W12 residual, document.rs): Perl's `replaceTree` (Document.pm:2079-2091) runs `removeNode` on the attached old node, so its box leaves the parent's (and each auto-opened ancestor's) box before `appendTree` adds the copy's; Rust's copy-then-detach adds the copy's box without removing the old one. Callers: the math parser (`replace_tree_deferred`), the `svg:foreignObject` cleanup (tex_box.rs). Only `cleanup_math` has Perl's `replaceTree` boxes (`Document::replace_node_as_tree`).
- **A text-only `svg:foreignObject` is not renamed to `svg:text`** (W12 review probe t5, pre-W12): Perl's foreignObject `afterClose` renames it (TeX_Box.pool.ltxml:386-389); the Rust port (tex_box.rs, its comment says it does) has no such branch. A tikz `\node[draw,align=center]{A\\BB}` gives Perl two `<svg:text>` lines, Rust a 10.38×9.46 and a 19.6×9.46 foreignObject, and its cells lack Perl's `ltx_nopad_l` class; `$\phantom{H}$` in a node gives Rust a foreignObject holding nullfont spaces.
- **The foreignObject width override is stale** (W12 review, `latexml_engine/src/tex_box.rs:885-918`): an auto-opened `svg:foreignObject` wrapping a width-bearing `ltx:inline-block` takes that block's `width` attribute in place of its box's width, on the premise "appendNodeBox creates Lists that sum widths incorrectly", which W12 retired (DIVERGENCES #319). It makes `\node[draw]{\fbox{\parbox{2cm}{Hello world}}}` 78.73 px wide (Perl 88.15). Retire it in its own A/B batch.
- **`\node{x $\text{ab}$ y}` is 37.08 px wide, Perl 49.63** (W12 review probe t1, pre-W12): Perl measures an undefined `\text` (no amsmath) as its literal name inside the Math.
- **The pgf shading node `\node{YAY!\pgfuseshading{msh2}}` is 227.16 px wide, Perl 230.62** (unit_tests_by_silviu): Perl keeps a space before the shading picture that pdflatex does not set (`\wd` of `\hbox{YAY!\pgfuseshading{msh2}}` = 123.99pt = "YAY!" 23.61pt + shading 100.38pt); the ~1.7pt Rust still lacks is the Y–A kerns across separate text boxes.
- **Figure panels under `geometry` are measured at the class `\textwidth` but arranged at `\Gm@tw`** (pre-existing; `latex_constructs/mod.rs` `after_float` floatwidth override, OXIDIZED_DESIGN #99): in 2605.03152 (`[margin=1in]{geometry}`, S3.F2) the `0.41\textwidth` subfigures are 141.5pt wide (0.41 × 345pt) against a 469.8pt row. The W12 fixup's panel-row change (KNOWN_PERL_ERRORS #274) turned its rows from 2 | 2 (56jk, pdflatex's layout, reached through Perl's row-width carry-over) into 3 | 1; pdflatex (0.41 × 469.75pt) sets two per row, separated by the source's blank line, which the rows ignore (Perl alike).
- **A merged number's box is wider than its glyphs** (W12 fixup review, pre-existing): the math ligature's composite box is a math-mode `List` of the merged tokens, which adds ~3mu. `\node{$2.414$}` is a 33.82 px foreignObject (Perl 31.52; pdflatex 22.78pt), `x \put(0,0){$2.414$} y` a 45.74 px picture (Perl 43.43).
- **A declared UTF-8 character is case-changed through its LICR** (batch 56jw residual): pdfTeX's `\MakeUppercase{ά}` keeps the character (its lead octet is `\protected`, utf8.def:180), maps the code point to U+0386 and typesets that LICR, "΄Α"; the case changer expands `ά`'s LICR `\ensuregreek{\acctonos\textalpha}` (lgrenc.dfu:125) and textalpha drops the tonos of `\acctonos`, "Α" (repro `unicode-catcodes/case_change_text_command_escape.tex`). Keeping the declared character and casing its code point (tried in 56jw) gives "΄Α" but loses the accent drop of the Greek locale, which l3text does on code points (`\__text_change_case_upper_el:nnnnn`, expl3-code.tex:37019-): char-list's babel-greek `\TestUppercase` lines lost 20 words (recall 74.9 → 71.6 %). Both belong together: the code-point path with the `el` rules and `\DeclareUppercaseMapping` (a no-op in sect08.rs).
- **Binding closures that assign as they expand: the unflagged rows** (batch 56jw audit, KNOWN_PERL_ERRORS #286): a number scan still runs `\IfFileExists`/`\InputIfFileExists` (sect13.rs, robust in latex.ltx:9667/9710), `\AtEndOfPackage` (sect05.rs), `\ProvidesPackage`/`\ProvidesFile`, `\@startsection` (sect04.rs), the `\e@alloc` allocators, amsthm `\swapnumbers`, paralist `\setdefaultenum`/`\setdefaultitem`, xkeyval `\setrmkeys`/`\key@ifundefined`/`\disable@keys`/`\DeclareOptionX*`, IEEEtran `\IEEEQEDhere`, multido `\fpAdd`/`\fpSub`, contrib biblatex `\DeclareCiteCommand`/`\DeclareMultiCiteCommand` and chemnum `\resetcmpd` (tabularray's `\lx@tblr@env`/`\lx@tblr@portrait` set the after-table state and raise the unknown-type/theme errors while expanding — internal, called only from the environment), whose originals reach an unexpandable token before they act. None has a corpus witness; each wants its repro before `peeks_by_futurelet`. ntheorem's `\theoremheaderfont` and siblings are macros where ntheorem.sty:628-634 declares token registers.
- **A constructor template's `?#N` holds for "0"** (W12 review; `latexml_codegen/src/constructable.rs:301-310` `emit_bool`): Perl's conditional is the argument's string (Constructor/Compiler.pm:166), false for "0" and "". `\qbezier`/`\bezier` work around it (DIVERGENCES #316); the generic fix changes many goldens (its own batch).

- **Residuals of batches 57cn–59l** (rc132 schema roots, then the scoreboard batches; LANDED prose verbatim in
  `perfect_kernel/archive/LEDGER_PHASE57_58_2026-10-02.md`, one row per 59 batch in `perfect_kernel/LEDGER.md`). Open:
  - *hyperref anchors* (57cn.1, 57cp, 58i): `\hyper@anchor`/`\hyper@anchorstart`/`\hyper@anchorend` still gobblers
    (hyperref_sty.rs:693; fdudoc.cls:921-922; multibibliography/amsrefs/bibtopicprefix `cite.<key>`); a vertical-mode
    `\Hy@raisedlink{\hypertarget{v}{}}` opens a paragraph (hyperref.sty:2098-2102; tudscrmanual.cls:524, reledmac
    `\edlabel`); an anchor in math loses its name (`$a\hypertarget{m}{}b$`); a stale `\@currentHref` names memoir pagenote
    anchors (`target-star-.3`, ucalgmthesis); glossaries-extra `\printunsrtglossary` re-digests a label's anchor into its
    `typerefnum` (`glo..fooa`); `\Hy@SaveSpaceFactor` family undefined; RED `singletons/anchor_at_paragraph_start_keeps_the_space`. A `\hypertarget` around a block in vertical mode (a `\parbox`, after `\item`, in `quote`/`minipage`/`p{}` cells) puts its destination in the paragraph after the block, one block late (59m, KPE #424: exact placement before it would need a vertical-mode destination, which `\hypertarget{n}{\section{T}}`'s in-title convention rules out).
  - *tabularray* (57cp; `evaluate=`/`expand=` landed 59n): rules (`hlines`, `vlines`, `hline{…}`) are not drawn, so an
    all-empty table is pruned whole (RED `alignment-bindings/tblr_all_empty_table_keeps_its_rules`; the manual's
    `\makeEmptyTable` demo, tabularray.tex:2859-2870); a math table without a colspec is sized before `evaluate=`/`expand=` run (RED `alignment-bindings/tblr_math_columns_follow_the_expanded_body`); custom `caption-tag`/`caption-sep`
    ("Quadro 2 —", tabularray-abnt); a long/tall table in a captioned `figure`/`subfigure` nests a `table`; document
    templates print as tabularray's own (tblr-extras head, `label=none`); a math table's `tex=` holds the reduction;
    `\SetCell[c=…,r=…]` spans, `rownum`/`colnum`/`rowcount`/`colcount`; `cmd=`/font styles; the LoT line of a caption-less
    long table. Settled (pdflatex errors too): `\caption{\url{…#1}}`, floats in minipages, `{l→}`.
  - *floats/captions*: RED `captions-floats/{two_captions_in_one_float_are_numbered_apart (KPE #396),
    subfig_subfloats_restart_in_each_figure, float_in_a_subfloat_row_keeps_the_parent_number (KPE #395 part),
    captionsetup_type_in_a_figure_minipage, captionless_longtable_is_not_listed, label_after_a_continued_caption,
    label_after_a_caption_with_a_note_names_the_float (SHARED), cleveref_optional_label_type_names_the_label (2605.13648),
    filecontents_graphic_has_its_size}`, RED `boxes-groups/caption_minipage_in_a_table_cell`; text a `\refstepcounter`
    rebinding typesets is dropped; floatrow — a footnote ignores `\def\@mpfn{mpfootnote}` (floatrow.sty:476-478), a
    `\floatfoot` in a plain `table` and a `\DeclareNewFloatType` float are lost (our float environments shadow
    `\float@end`, :114-127, :421-424), a floatbox caption lacks `ltx_centering` and a zero-size `<rule>` follows the body
    (:392); float placements are unchecked (`[!H]`); a tcolorbox listing's title/number is not printed (59i).
  - *box measurement* (58b-58e; K15/K18): `\vsplit` never splits a paragraph into lines (reledmac
    2-titles_in_line_numbering_with_notes 16 numbered lines vs 28); `\penalty` leaves no item, no interline glue or
    `\splittopskip`; `{x\par y}` measures one line more; `\lastbox` removes any last item; a note counts as a line break,
    plain `\footnote` is measured with its body, a minipage footnote's text is uncounted (DIVERGENCES #391); size10.clo
    leadings whatever the class size; a footnote keeps the surrounding leading; RED `alignment/booktabs_rules_add_their_spacing`;
    RED `boxes-groups/negative_kern_overprints` (2605.14271); 2605.02221's `\rTo{\pi}` sequences now parse as fragments;
    reledmac — `\patchcmd{\endminipage}` fails (reledmac.sty:7643-7654), `\footnote` in `\pstart` splits the line.
  - *fonts* (58f, 58g, 58k, 59d): a raw font's `\fontdimen` reads cmr10's, `\fontcharwd`/`\fontcharht` the undecoded char;
    the lcircle map is unchecked; `\ding` 7.5pt vs 5.71pt; a standard-metric font's missing char (KPE #408); `\f@encoding`
    inside a raw font's group; RED `fonts-nfss/math_char_without_glyph_keeps_its_box` (`t1_guillemet_ligatures` GREEN 60s); T2A ј/і as
    Latin j/i (surpass candidate); a redefined `\everymath` is not run (latex.ltx:10580).
  - *lists*: RED `list-structure/stale_list_counter_reset_gives_I0_ids` (Perl-origin; 2605.04012); a `\list` on the
    enclosing counter inside `\lx@trivlist@setup` gets `i0`; RED `block-model/noindent_paragraph_in_minipage_numbers_inside`
    (2605.21713; same defect as the K13 row's `noindent_paragraph_in_a_minipage_takes_its_id`).
  - *frontmatter* (58m, 58q, 59d-59g): RED `sectioning-frontmatter/{store_set_again_after_maketitle_keeps_the_title_page_value,
    deposit_of_only_punctuation_is_dropped, abstract_store_keeps_the_document_abstract,
    store_default_that_typesets_nothing_leaves_no_element, ijcai_markers_in_an_appending_author}`; bfh-ci
    Factsheet/ProjektProposal lose their department (a footer-layer semantic note, kept under the 2026-10-01 ruling);
    `\addresslink`-style marks `\let` to `\textsuperscript` are not author marks; dtk.cls `\Author`; RED
    `singletons/omnibus_alias_shadows_a_local_package`; g-brief (59o, #412): only a serial document's first letter gets
    its sender/addressee (RED `sectioning-frontmatter/gbrief_serial_letters_each_have_their_frontmatter`), a contact
    label's ligatures stay untypeset in `name=` (RED `sectioning-frontmatter/contact_name_label_keeps_its_ligatures`); acmart — the year without `\copyrightyear`/`\acmYear` is the
    conversion's, a document's own `\footnotetextcopyrightpermission` lands in the first paragraph, the cc logo/link and
    `authorversion` statement are unported, an unknown `\setcopyright` is silent.
  - *bibliography and notes*: `\printbibliography[type=…]`/`[check=…]` all print under the first call (biblatex_sty.rs
    `\biblatex@printbibliography`; `~/data/pk_agents/rc132/repro/typed_print.tex`; the biblatex manual's 244 missing
    words); endnotes `\theenmark` = `\theendnote` (PERL-ORIGIN endnotes.sty.ltxml:29; cms-noteref-demo "18" ×18); enotez
    `split=section` — a `\chapter` starts a split, a pre-heading mark's `\endnotetext` lists in the mark's split, 0.10d
    internals overridden; biblatex-ext-oa prints no open-access mark (RED `boxes-groups/xsavebox_saved_box_prints_its_content`);
    Ruled STAY (D15): biblatex `cite.<n>@<key>` (cms-dates-intro 80, cms-trad-appendix 4) and
    gost `back:` (770) dangling links — the surpass option needs a ruling
    (`~/data/pk_agents/main/agent_reports/2026-10-02_biblatex_idref_rootcause.md`).
  - *kernel*: RED `singletons/register_value_absorbs_as_its_text` (`digested.rs:378`; KPE #403); RED
    `macro-state/generic_command_hooks_run`; the loader ignores `\ver@<pkg>.sty` (memoir `\EmulatedPackage{setspace}`
    loads the setspace binding; see the K13 row); Perl's open-conditional reports at `\end{document}`
    (latex_constructs.pool.ltxml:357,374-377) and end of input (Core.pm:229-233) are unported (sect02.rs); the raw
    babel `.ldf` load stays parked on the `\mdqoff` note (latexml_package lib.rs).
  - *pgfmath/TikZ* (59a-59c): RED `graphics-tikz/{tikzmath_function_draws, pgfmath_width_of_formatted_text}` (both SHARED);
    `random(m,n)` integer, `.5`/`1.` pass through (DIVERGENCES #404); a missing `:` in a chained ternary is silent;
    `\pgfmathsetlength`'s leftover tokens run outside pgf's group; perf levers L1 and L6 landed (59q) and G4 is
    met since s136 (pgf-interference-en ~170 s alone); further levers are optional margin
    (`~/data/pk_agents/main/agent_reports/2026-10-03_g4_pgf_interference_levers.md`).
  - Out of scope (recorded): luatexja `\fontencoding{T1}` prints "T1" (`\luadef` predicate no-ops); biblatex-ext's last
    error, a bibliography inside a tcolorbox (user 2026-10-02); cms-notes-intro (lualatex-unclean).
- **Recall-tail residuals, sweep #130 (rc131; LANDED 57cl-57cm.2 verbatim in the archive)** — golden mismatches are
  curated in `tools/perfect_kernel/golden_reference.tsv` (dinbrief, geradwp, JACoW A4/Letter; `candidate-mismatch` is a
  flag only). Open: RED `graphics-tikz/xml_id_after_a_picture_repeats_its_svg_id` (one-way check); RED
  `graphics-tikz/eqnarray_copy_in_a_picture_keeps_its_references` (`append_clone_aux`, document.rs: copy the svg id as Perl,
  Document.pm:1942-1951, dropping `_svgid` from the clone so `record_svg_ids` renames it); **abntex2cite** (413 missing
  words at s134, the top in-scope S3 doc; -alf 102): `abnt-verbatim-entry` blocks (post, ~+11 %), `\autoref` babel names
  (`\HyLang@DeclareLang`), `reprinted-from`, `conference-*` (abntex2-num.bst:1003-1014), `\hiddenbibitem` (`@hidden`,
  abntex2-num.bst:1379-1390), URL/citation-order rules keyed on the style name (unbtex `unbtex-alf-pt` prints "Link" ×9:
  key them off the `.bst`'s `format.url`/`presort`), and the presentation list (upper-case `alf` author, "e" and ";"
  separators, a multi-key `\citeonline[note]`, authorless entries, `abnt-full-initials=yes`, num "(1)" brackets
  (`\citebrackets`, :213, :232-233), repeated authors (:995-999, :1037-1043), `alf` list labels, note field order, the math
  `tex=` brace group) — repros `~/data/pk_agents/rc131/abntex2/repro/r1–r8`, `review57cm/`; **jacow-collaboration**
  (92.7 %): author continuation lines glued at `\\` (`add_authors_calls`, base_utilities.rs:1235-1239 = Perl
  Base_Utility.pool.ltxml:702-703, PERL-ORIGIN; keep the delimiter as `<break/>`; `~/data/pk_agents/rc131/repro/author_glue.tex`).
  Ruled out: suanpan-l3's last jing line (a second `\title`/`\maketitle` replaces the first — double `\maketitle` keeps
  the latest title, user 2026-10-01).

### Perfect-kernel RED repros without a row elsewhere

The open RED repros of `tools/perfect_kernel/repros/` that no other row names; headers re-checked 2026-10-02 (scope:
PERFECT_KERNEL.md → Scope; a witness whose oracle is unclean is a crash canary). Each row: defect, classification, repro path.

- **`\multirow` in a `p{}` column spans no rows** (PERL-ORIGIN): the `p{0mm}` cell gets no `rowspan` where the `c`-column copy gets `rowspan="2"`, so the rotated label inflates its row (2605.23257 S5.T4 169.5pt vs pdflatex 116.2; A2.T7). Both bindings put `rowspan` on the alignment lookup's current column (`multirow_sty.rs:18-45`), which misses inside a p cell's box. Repro `alignment-bindings/multirow_in_p_column_spans.tex`.
- **`\resizebox` of a zero-size box** (PERL-ORIGIN, K18 step 2 residue): a 0-width box gets `xscale="6553600"` where graphics.sty's `\Gscale@div` reports "Division by 0" and uses 1. Repro `boxes-groups/resizebox_zero_size_division.tex`. (The `pspicture` half landed in 58i, DIVERGENCES #397: `graphics-tikz/pspicture_size_is_typed` GREEN.)
- **`\obeylines` makes the newline an `<ltx:break/>`, not `\par`** (PERL-ORIGIN; OUT OF SCOPE): Perl/Rust let the active `^^M` be `\@break` (`plain_base.rs:467-470`), latex.ltx:575-576 `\obeyedline` (= `\par`), so `\let\par=\cr \obeylines` ends no row. 6 errors, first `Error:unexpected:&`. A faithful fix changes goldens everywhere: ruling needed. Repro `alignment/metre_obeylines_halign.tex`.
- **A `\def` whose parameter text meets `}` swallows text to the next `{`** (SHARED; kernel): tex.web §474-475 inserts "Missing {" and ends the definition at the `}`; `\def … UntilBrace` (`tex_macro.rs:91`) reads on, so storecmd-guide loses two sentences. 2 errors vs pdflatex 6. Repro `expansion-primitives/def_param_text_right_brace_storecmd.tex`.
- **A `\lastbox` loop counts 2 title lines, pdflatex 1** (SHARED): `\lastbox` pops whatever item is last (`tex_box.rs`; Perl TeX_Box.pool.ltxml:596) and paragraphs have no line boxes; needs K15's list tail record. Repro `expansion-primitives/unpenalty_lastbox_sidenotes.tex`.
- **`\keys_set:nn {siunitx}` finds no l3keys module** (SHARED): the binding declares keyval `SIX` keys, so currency.sty:90's raw `\keys_set:nn {siunitx}{round-precision=2}` gets l3keys' unknown-key error (1 `Error:errmessage`; Perl prints it uncounted). Fix: declare the `siunitx` l3keys module over the binding's keys, routed as `\sisetup`. Repro `expl3/l3chk_siunitx_keys_set_l3keys_empty.tex`.
- **chessboard under beamer overlays** (RULED residue, DIFFICULT_CASES §D14, Perl's single-pass overlays kept): 3 errors + `Fatal:Mouth:EoF`, first `mainline: black, not white, to move (e4)`; in scope (sweep #130's non-clean in-scope doc). Kept. Repro `expl3/s41msg_chessboard_beamer_only_gamestate.tex`.
- **fourier-orns ornaments print their `futs` slot characters** (SHARED): no `futs` glyph map in `latexml_core/src/common/font.rs`, so the raw preload prints "A 1 B M C U D e". Fix: map `futs` to the Unicode ornaments. Repro `fonts-nfss/fourier_orns_ornaments_are_their_glyphs.tex`.
- **forest-doc's `\DocInput` example: binding gaps** (OUT OF SCOPE, R8b): 3 errors (`\lst@InstallKeywords` SHARED; `\@escapeifif`, `\forest@file@copy` from the forest binding, a native tree since 56iz). Repro `graphics-tikz/forest_docinput_lstenv_optarg_absent.tex`.
- **A raw `\item[…]` label is re-digested for the `typerefnum` tag** (PERL-ORIGIN, latex_constructs.pool.ltxml:1384-1386): `\typerefnum@<counter>` digests `Revert($tag)`, so `\item[\n]` gives `Error:undefined:\n` and `\item[\(]` opens math. 5 and 2 errors (Perl 61 and 43). Fix: build the typerefnum from the formatted tag. Repros `index-bib/item_tag_makelabel_typerefnum.tex`, `index-bib/item_tag_math_cascade_downstream.tex`.
- **An `lstlisting` in a `\parbox` argument is dropped silently** (CONTROL; RUST-ONLY): the pre-tokenized argument puts the body on the begin line, which the reader drops; 0 errors and an empty `<listing/>` (pdflatex 7 errors, no PDF). Needs a diagnostic only when the reader's source is a pre-tokenized argument. Repro `kernel-alignment/pcol_lstlisting_parbox_CONTROL.tex`.
- **`\RequirePackage` inside a group raises no error** (OUT OF SCOPE): pdflatex says "Loading a class or package in a group" (latex.ltx:18699-18704) and loses `{multicols}` at the group's end; Rust loses it silently. Fix: the group-level check in `\RequirePackage`/`\usepackage` (`latex_constructs/sect05.rs:180`). Repro `loader/multicols_afterpackage_group.tex`.
- **A product before a big operator stays unparsed** (RUST-ONLY): `a\cdot b\sum_i c_i`, `\frac1n\sum_i x_i\cdot\frac1m\sum_j y_j` — 4 `Warning:unparsed_math`; Perl parses all four. Not root-caused (the wide/narrow big-operator pragma or a missing `term mulop term bigop_application` path); a bare OPFUNCTION as the MulOp's right operand alike (`a\cdot\log\sum_i x_i`, `a\cdot\mathbb{E}\sum_i X_i`; Perl (a·log)·∑…, 57cf review), and `a/b\sum_i x_i`, `a\times b\int f\,dx`. Repro `math-parse/product_before_a_big_operator.tex`.
- **An `\index` phrase is re-tokenized live** (SHARED; with the `&` glossary row above): SanitizedVerbatim (`base_parameter_types.rs:692-716`) re-tokenizes `\index`'s argument with normal catcodes, so `\index{a\def{b}c}` runs `\def` at the mark (1 error). Repro `string-mouth/index_phrase_primitive_retokenize.tex`.
- **An \item inside {framed}** (RUST-ONLY shape; GREEN on errors): the boxed `\item` keeps the outer numbering (`S0.I1.i2`) but sits in a framed `<itemize>` inside item 1's paragraph, not as the list's own second `<item>` (Perl: 2 `Error:malformed:ltx:item`). Ruling needed on the shape: a framed sibling item or the nested framed list. Repro `boxes-groups/mal_item_framed_capture.tex` (colorframed-doc, OUT OF SCOPE).
- **hyperref's PDF information values are recorded unexpanded** (SHARED, Perl hyperref.sty.ltxml:133 `ToString`):
  `pdftitle=\mytitle` gives `<rdf content="\mytitle"/>`; hyperref expands with `\pdfstringdef` (resphilosophica.cls:311
  `pdftitle=\@title`). Digesting the value would duplicate frontmatter notes (`\thanks`); needs a `\pdfstringdef`-like
  expansion. RED `sectioning-frontmatter/hyperref_pdfinfo_expands_macros`.
- **Header re-grades pending**: `luatex-profile/xetexprobe_xevlna_shared` needs `% preload: [xetex]` (then libertinus-otf.sty:215 `\XeTeXtracingfonts` undefined is a new RED); `luatex-profile/babelmodifier_greek_polutoniko` drops its `[luatex]` line (lualatex fails too; pdfTeX 0 errors); `luatex-profile/zugferdtabular_loop` needs the witness's `unit=hour`; `graphics-tikz/calc_scbox_babel_frozen_bang_stale_oracle` becomes CONTROL.

### Run-329 and sandbox open residuals (61r-62f, 2026-10-04)

Open items from the cortex reruns of sandboxes 2605/2606 (61r) and from mining run 329 (62a-62f). The fixed narratives
(61r's validation, the stop of run 329 and the 62a-62f fixes with their numbers, witnesses and guards) are archived in
[`archive/SYNC_SESSIONS_2026-10.md`](archive/SYNC_SESSIONS_2026-10.md); first-error and Fatal cluster tables of run 329
in `~/data/pk_agents/main/scratch_g/run329/clusters62c/`.

From 61r (sandboxes 2605/2606): a tabular
`\\[<len>]` with a non-positive length is `\setlength`-calcified in latex.ltx (:16586-16602, `\@vspace@calcify`) but read
by TeX's scan here; latex.ltx 2025's `\vspace` is calcified too (:9254/9362) while ours is `\vskip#2\relax`. Open, each pdflatex-clean, ids in
`~/data/pk_agents/main/hostb_61r4.tsv`:
  - calc's `!` protocol (`\calc@next`, calc.sty:52/147/153): linegoal's `\LNGL@set!` runs inside the expression
    (2605.05695).
  - Pre-broken papers now Fatal: 2605.05881 (`\pgfutil@emu@@unpack` Mouth EoF; 83 errors on 56il), 2605.10232 (tikz-cd
    cells holding `tblr` tables with `\&`; PushbackLimit since 57cp; 23 errors on 56il).
  - Latent panic: `gullet.rs` `read_token`'s `handle_template(data.borrow_mut(), …)` (RefCell already borrowed), Fatal
    on 2606.05500 under both images until 61s (KPE #462) removed the makecell cascade that reached it; the full paper
    on `latexml_oxide.61r6-rel` still reproduces it, no truncation of it does.
  - `runtime-bindings` builds only: a versioned name whose stripped name has only a runtime `.rhai` binding is found by
    loading it (`find_file_fallback`), without the document's options; the caller's load with them is then skipped.
    Fix: ask the rhai tier whether `<name>.rhai` exists instead of loading it.
  - Singletons (1-8 errors each, 56il-57ck window unless noted): Missing control sequence inserted (2605.26277,
    2606.00866, 2606.20186); listings input files not found (2605.15569, 2605.12863); delimited user macros
    (`\edef\__figfile` read through a neutralized Semiverbatim, KPE #465, 2606.13798; `\def\\ell` reaching
    authblk's `\affil` text, which TeX typesets where `\\` is local — authblk.sty:156, `\@maketitle`'s tabular —
    2605.28517; `\\` Match:X in neurips' author tabular, minimal shape clean, 2605.12000); 2605.04969 (sectioning in a box, 59j); 2605.07684 (`\ContinuedFloat` after a table); 2605.10068 (`#`
    reaches the stomach, 61m); 2605.11285, 2605.31585 (58h counter steps: `\the\cmdKV@…`, `\iffirstchoice@`);
    2606.15122 (utf8 keyboard character, 58q); 2605.19122 (`_Capture_` close); 2605.29722 (`_` outside math);
    2606.30845 (`\capitalizethefirst`); 2606.11726 (2 undefined counters in a plain-TeX paper now converted whole).

62s: the 2609 templates' new author and title macros, from each template's source (2609 cluster study): neurips_2026
`\workshoptitle` + tracks (113 papers), acmart `\correspondingauthor` + `\if@ACM@anonymous` (76+8), aa `\corrauth` (58),
spconf `\sthanks` (45), revtex's `\move@AU`/`\move@AF`/`\@affiliation` for openjournal (33), wacv via the cvpr binding
with `\thetitle` the title's copy (21; wrapped once when cvpr and wacv both load, and for iccv.sty:477-479), natbib's
`\@ifxundefined`/`\NAT@sectionbib` for iau/JFM (24), acmart `\if@ACM@balance` (11), neurips' `preprint` switch and
tracks' `\@trackname` and `\@noticestring` with the year's own ordinal, location and track wording — also for a style
requested under a directory, whose fallback records the request (`content.rs`, Perl's `\@currname`); no venue note of
the binding's own, as the paper's style copy decides what its first page prints, so a camera-ready workshop paper's
workshop name is not output (DIVERGENCES #448; a workshop-only note awaits a ruling) — acmart's `balance`
options, aa `\aa@emailfont`, newtxmath's `\up<letter>` names (`\upmu`, 15). Guards
`perfect_kernel_batch61::templates_of_2609_keep_their_author_and_title_macros`,
`perfect_kernel_batch61::neurips_notice_names_the_year_of_its_style`. Known limit: a preamble `\newcommand`
of one of these names is ignored like any binding macro's (no 2609 bundle is hurt; older-template papers could be). The 2609 study's other findings:
the expl3 "Mismatched LaTeX support files" errors are the dump regression below (62u), not the harness; 945 EPS
failures: AppArmor `gs` denies `/opt/cortex-scratch` (open).

62t: REGRESSION since 62b fixed — a caption opening with a conditional (`\caption{\ifdefined\x …\fi …}`) ended in
"Extra \else" + Fatal EoF scanning for `\endcaption` (2609.27590, 2609.28438): `\@dblarg` always gives a `[short]` and
`\@caption@@@`'s `\ifx.#2.#3\else#2\fi` took the conditional as its second token; the list entry is now chosen by the
detokenized `[short]` without skipping either text, as are longtable's and listings' (KNOWN_PERL_ERRORS #468). The 2609
classes' commands: llncs `\doi` (a single `\url` link under hyperref; 2609.04690; #492), sagej `\affilnum` (2609.04585), the
appendices' `\Roman` numbering of ieeeconf (`\ifuseRomanappendices`, default on; 2609.16300) and IEEEtran
(`romanappendices`; KNOWN_PERL_ERRORS #491), subfigure's captions chosen the same way as the
kernel's (#468), acronym `\Acp`/`\Aclp`/`\Acfp`/`\Aclu`/`\Acfip` (let to the lower-case forms, the capital not
rendered) and `\acfip` (#490). Guard `perfect_kernel_batch61::captions_and_2609_class_commands_keep_their_text`.
OPEN (design, user 2026-10-05): a paper and its supplement are two top-level documents — revtex's supplement idiom
restarts the title block (`\frontmatter@init` + lets of `\title`/`\author`/`\maketitle` to `\frontmatter@…`,
revtex4-2.cls:2115-2140,3127-3140; 2609.07332, TooManyErrors Fatal) — and the frontmatter API has one title block, so
lets to the binding's `\title` replace the paper's title and append the supplement's authors (the draft is set aside
locally, recorded in the main handoff register); the frontmatter API needs extending for a second document.
62t (part 2): the 2609 classes load what their class files load — lipics-v2021 (array, subcaption, comment with the
`{CCSXML}` exclusion, multirow, tabularx, …, and at the document's start xcolor unless the paper has it plus the class's
named colours, soul's `\textsolittle`; 2609.13401 64 → 44 errors, 2609.10114, 2609.13485 13 → 0), fairmeta (2609.11172),
WileyNJDv5 (and the WileyNJD-v2/WileyASNA-v1 copies the binding serves, which load the same) with caption
(2609.06025 2 → 0), bmvc2k's xcolor and T1 with its `\addauthor` mail read verbatim and set in sans (2609.06007 4 → 0),
lmcs's theorem set with its `defC`/`thmC` styles, tikz, xparse, mathtools (2609.11893 10 → 0), informs3's
`\TheoremsNumbered*`/`\EquationsNumbered*`/`\ECSwitch` (2609.08001), MnSymbol `\llangle`/`\rrangle` (2609.07645);
`\tracingmacros`/`\tracingcommands` stored as numbers so `\the` reads them back (Perl TeX_Debugging.pool.ltxml:214-225;
was silently empty). Guard `perfect_kernel_batch61::the_2609_classes_load_their_packages_and_commands`. The bindings'
named witnesses, re-converted: 2502.11299 25 → 0, 2305.19985 14 → 2, 2305.14448 2 → 0, 2511.16624 1 → 0, the other fifteen
unchanged (lipics 2311.17226, 2211.04601, 2606.01187; Wiley 2203.16535, 2406.06228, 2407.00139, 2504.02281; fairmeta
2412.06264, 2508.07407, 2509.24704, 2605.29955; lmcs 1607.01886, 1607.04128, 1709.06170; bmvc2k's xcolor reload
2605.00310). `\ECSwitch` without a `\TheoremsNumbered*` is undefined, as under pdflatex (the
class defines no default). Residuals:
math-mode `^`/`_` in 2609.13401; a raw `_` in an email (`\lx@add@email`) prints OT1's dot accent (a catcode-12 `_` in
roman OT1, as pdflatex would — classes that set the address with url.sty, in typewriter or under T1 need the binding to
say so, as bmvc2k's now does); the amsthm binding ignores a `\newtheoremstyle` head spec (lmcs's `thmC` drops the
note's parentheses, ours keeps them) and informs3's `\mdseries\scshape` head font (ours bold, :1857-1859); thm-restate does not load thmtools' `\declaretheorem` (2606.01187); 2609.25833's
98 errors are arabtex (`\setcode`, `\RL`); amsart's `\@xsetfontsize` internals (2609.37833) landed in 62z. GENERALIZATION
(open): these bindings hand-copy their class's package list, which drifts by class version and drops options
(fairmeta's `[numbers,sort&compress]{natbib}`); Perl's own dependency scan of the shipped class
(`require_dependencies_except`, as sn_jnl_cls.rs uses it) with per-class exceptions is the general form — needs its own
A/B over each class's papers.

62zf: elsarticle declares the class's other conditionals (elsarticle.cls:40-45 `\iflongmktitle`, `\ifdoubleblind`,
`\ifnonatbib`, `\ifnopreprintline`, `\ifuseexplthreefunctions` — the last stays false, the class's expl3 helpers it
selects being unemulated) and their options: the Elsevier journal styles
(jasr.sty, cnf.sty, jcomp.sty) test them inside the `\if@twocolumn` branches of their `\maketitle`, and undefined they
unbalanced TeX's skip of the false branch, so the other branch's body ran at load (`\finalMaketitle` undefined, an
unbalanced `}`, stray `\else`/`\fi`). The class's `\emailauthor{<email>}{<name>}`/`\urlauthor` (:211-235) are a
frontmatter note "email (name)", as the first page prints them — `\ead`'s contact would land on whichever author
precedes the call; the styles' `\KWD` (defined in their own `\keyword`, which the binding's {keyword} bypasses) is a
no-op. 2609.23725, 23732, 27838, 39431, 09773, 31741, 39502: 4-6 errors → 0. `nonatbib` leaves natbib out, as the class
does (:1242; KNOWN_PERL_ERRORS #501; 2609.05849, 19225, 20719, 23461, 26003). Guards
`perfect_kernel_batch61::{elsarticle_journal_styles_load, elsarticle_nonatbib_leaves_natbib_out}`.

62ze: the mnras binding takes mnras.cls's own settings: its enumerate labels "(i)", "(a)" (:930-940), the bibliography
heading "REFERENCES" (:1312, 1339), with `usenatbib` the author-year punctuation "(Draine 2011)" (:1336 `\bibpunct`),
with `usedcolumn` the `d`/`.`/`,` columns on dcolumn (:1349-1354); the mn binding declared that option as `usedcolum`
(Perl's typo, KNOWN_PERL_ERRORS #500), so dcolumn never loaded. 216 mnras + 18 rasti papers in 2609. Guard
`perfect_kernel_batch61::mnras_lists_citations_and_columns`.

62zd: rasti.cls (RAS Techniques and Instruments, mnras.cls under another name: v3.0 defines the same commands) loads
as the mnras binding (rasti_cls.rs): all 18 rasti papers in 2609 at 0 errors (2609.08700, 09329, 27622, 29860, 35676
had 5-33). cas-common's `stm/mktitle` title-page keys are declared (2609.00281). The box capture's backmatter lift
stops at a float, as at an alignment (base_utilities.rs `insert_block`, OXIDIZED_DESIGN_DIVERGENCES #205): an appendix
section or a bibliography in a minipage in a table was lifted past it with no diagnostic, leaving the table empty and
its caption at the top level; it stays in the table and errors as Perl's does (A/B over 867 papers: 8 improved; 2609.05763,
a cas paper's abbreviation list in a framed table, now in its table with Perl's `ltx:glossary`-in-`ltx:block` error
instead of moved after an emptied table). OPEN (third stop on one climb —
consolidate): the climb's stops are per-tag (list, alignment, float), and a box in a `quote` still leaks — the lift
moves the insertion point past the open quote, so the text after the box escapes it (RED
repros/boxes-groups/bibliography_in_minipage_in_quote_keeps_the_quote.tex); Perl's `floatToElement` restores the
insertion point after floating past an open container, the likely single rule, but the xebaposter/juradiss hoists
(surpass) must keep working — needs its own A/B. The mnras-binding gaps rasti papers inherited landed in 62ze. NEEDS A RULING: a `\section` inside a float (38 papers in 2609, 117 errors; 29 have no
other error: 2609.03590, 00943, 05680) is valid TeX — `\@startsection` needs only vertical mode — and could open in an
`ltx:inline-sectional-block` in the float (schema-valid, the appendix as its `ltx:section` stand-in; prototyped, 0
errors, jing-clean), but the 2026-10-04 ruling restored Perl's `Error:malformed` for sectioning in items and figures
(OXIDIZED_DESIGN_DIVERGENCES #189, `perfect_kernel_batch54::sectioning_unit_inside_item_or_figure_errors`), so it is
not applied. Guards `perfect_kernel_batch61::{rasti_loads_as_mnras, appendix_in_a_minipage_stays_in_its_float,
cas_common_helpers_can_be_renewed}`.

62zc: class bindings that skipped what the class loads or defines. optica-article's binding runs the class file's own
dependency scan (`require_dependencies_except`, as sn-jnl's, without soul — the `\else` arm of its `\ifpdf`): array,
tabularx, multirow, newtxmath were missing (2609.00899, 05706, 06191, 10235, 01145; all 24 optica papers in 2609 at 0
errors); its `fontenc[T1]` now prints `"`, `<`, `>` as pdflatex does (9 papers' text, e.g. 2609.12212's bibliography). cas-dc/cas-sc define cas-common's
name parsers, printers and page styles (no-ops; the frontmatter keeps e-mails and notes itself), so a paper's
`\RenewDocumentCommand\firstname` or `\ps@cas` no longer errors (2609.16168, 16199, 36345, 00634, 20010). The general
form — every class binding scanning its class file — stays open (GENERALIZATION above). Guards
`perfect_kernel_batch61::{optica_article_loads_its_class_packages, cas_common_helpers_can_be_renewed}`.

62zb: three kernel gaps from the 2609 recheck. `\input@path` searched for files kpathsea does not find, after the local
paths and kpathsea, a `/` added as l3file does (l3file's `\file_full_name:n`, expl3-code.tex:12585-12612; content.rs
`find_on_input_path`, OXIDIZED_DESIGN_DIVERGENCES #454):
71 papers in 2609 set it, 2609.02998's class style (and the natbib its `\citep`s needed) was missing. `\pdfstartlink`
reads its rule, attr and action specs (pdftex.rs `LinkSpecification`, KNOWN_PERL_ERRORS #499): the bare no-op typeset
`attr {…} goto name {…}`, 57 papers in 2609 use it (AAAI forbids hyperref; 2609.00161). aa's `\pmatrix`/`\cases` take
amsmath's environment form when `\@currenvir` names them, as `\matrix@check` does (KNOWN_PERL_ERRORS #498; 2609.02726,
2609.04318). Guards `perfect_kernel_batch61::{aa_cases_and_pmatrix_take_the_environment_form,
pdfstartlink_consumes_its_spec, input_path_finds_a_style_in_a_subdirectory}`.

62za: `\minipage`/`\endminipage` locked against class and package files (`:locked@files`, state.rs `is_name_locked`;
sect12.rs, OXIDIZED_DESIGN_DIVERGENCES #453), the document's own definitions free as in Perl: iucr.cls's `\endminipage`
(kernel internals, to drop the footnote rule) broke every minipage and, through tikz's `\pgfutil@endminipage`, every
`text width` node: 2609.07722 and 2609.02106, 265 errors each, Fatal → 0; memoir's own save-and-restore is retired. From
the 2609 tally (cortex bundles of worker a01c8944b4: 3,374 Error, 64 Fatal), class gaps the class files fill:
aaai2027.sty's `\corresponding` and aaai's `\equalcontrib`, defined only inside their `\@maketitle` with their footnotes,
now carry the note on each author (kernel stubs, `DefinitionOrigin::Stub`, which the document's or a raw file's own
`\newcommand` replaces, as copernicus.cls:1673's; sect05.rs; 223 AAAI-27 papers); egpubl (argument-less
`\PrintedOrElectronic`/`\BibtexOrBiblatex`/`\biberVersion`, whose argument swallowed the paper's `\ifpdf`; `\ConfName`; the
remaining editor setters kept as the others; the paper-type, page-number and licence selectors; `\excludecomment{CCSXML}`;
`\teaser` a figure at `\maketitle`, as `\@maketitle` sets it with `\def\@captype{figure}`, after the abstract since the
schema keeps top matter ahead of figures; `\EGyear` only stores the year `\ConfYear` keeps), bmvc2k (geometry,
`\BMVA@blfootnote`, `\bmvaEtAl`, two-argument `\runninghead`), sn-jnl `\unnumbered`/`\numbered`, acmart's section fonts
(`\@secfont`, patched by pvldb.sty): 2609.00420/00555 1 → 0, 2609.00994 23 → 0, 2609.00732 13 → 0, 2609.00981/08090 2 → 0,
2609.05015/05615 1 → 0, 2609.00548/02328 1 → 0. Open: acmart's teaser rewrite (acmart_cls.rs `\lx@relocate@teaser`)
never matches, since the abstract precedes the teaser in the built document, and moving the teaser ahead of the abstract
would break the schema (`ltx:document` takes its top matter first) — retire or rethink; raw-loaded copernicus.cls drops `\affil` and its equal-contribution note, both typeset only by its
`\maketitle` body, which is not replayed (`\citati@nbyarticlenumber`): 2609.05557, 07363, 08383, 12496, 13482, 17175,
21633, 36077; 931 papers' first error
`imageprocessing:failed_to_convert` (not examined); `\cprime` in MathSciNet `MRREVIEWER`/`FJOURNAL` fields of cited
`.bib` entries (40 papers; Perl's MathReview synthesis digests the reviewer too; the always-on stub was retracted by
maintainer decision 2026-07-27, latex_constructs_rust_only.rs — needs a ruling); the tally's `_` cluster (216 papers) is
mostly gone on HEAD (splncs04 `\doi{…_14}` in the `.bbl`, now clean); Perl-origin, pdflatex-clean: `\lefteqn{…} & &` in
eqnarray (Perl's `\lefteqn` spans 3 columns, so the tabs the idiom writes are extra, 2609.03020) and an etoolbox
`\AtBeginEnvironment{table}{\begingroup…}` + `\AfterEndEnvironment{table}{\endgroup}` pair (Package.pm:1918-1925 binds the
environment's mode into the hook's group, 2609.01057). Guards `perfect_kernel_batch61::{class_endminipage_does_not_replace_the_environment,
document_endminipage_definition_is_kept, class_bindings_2609_clusters}`.

62z: amsart's size machinery in ams_support (amsart.cls:169-219, 258-296): `\@xsetfontsize`, `\@currsizeindex`,
`\@adjustvertspacing`, `\@mainsize`/`\@ptsize`, and each size option's `\@typesizes` (10pt the default), which a class
built on amsart redefines its size commands with (m2an.cls:241): 2609.37833 undefined `\@xsetfontsize` then Fatal
PushbackLimit → 0 errors; the class bindings declare the size options as the classes do (ams_support keeps them no-ops,
as Perl, so a package does not run the document's size again), so `\LoadClass[12pt]{amsart}` sets the 12pt sizes. `\label` no longer digests `\@currentlabel` into a `LABEL@` value nothing reads (sect11.rs):
read lazily after a list whose `label=` redefined `\theenumi`, it reached the document's self-referential `\theenumi`
(KNOWN_PERL_ERRORS #496, OXIDIZED_DESIGN_DIVERGENCES #452; Perl hangs): 2609.13327 Fatal Recursion → 0. mathtools' paired delimiters follow the package
(`\delimsize` in a group, `\<size>l`/`\<size>r`; KNOWN_PERL_ERRORS #497, OXIDIZED_DESIGN_DIVERGENCES #451, ams
`mathtools` golden: three `tex` attributes): 2609.17447 861 errors, Fatal → 0. A/B over 551 papers (62y5 → 62z10, before the `\@ptsize`
restore): no output changed. Open (RED `tools/perfect_kernel/repros/math-parse/sized_bars_do_not_merge_into_a_norm.tex`, math
parked): two adjacent sized bars of different sizes merge into one `‖` (`\Bigl\lvert\bigl\lvert y…`), which
`\abs[\bigg]{\abs[\big]{y}}` now reaches; `\reDeclarePairedDelimiterInnerWrapper` still writes wrappers the X forms do
not read. Guards
`perfect_kernel_batch61::{amsart_xsetfontsize_sizes, label_after_a_relabeled_list_does_not_loop,
paired_delimiter_size_and_delimsize}`.

62y: four 2609 Fatal witnesses. (1) aastex7/aastex701 load rotating before their own `\rotate` (aastex701.cls:11416,
12196; aastex631 and earlier do not); the binding now does too (aastex_cls.rs, by the requested name), so a
deluxetable's `\rotate` stays its no-op instead of rotating's `{rotate}` environment, which the versioned fallback's
dependency scan of the class file loaded after the binding (Perl's versioned fallback scans nothing, Package.pm:2226):
2609.09266 Fatal → 2 errors (its .bib's ADS `\lt`, `\CID`). `rotatetable(*)` (:12212) set as its body: 2609.01052 → 0.
(2) `\left`/`\right`/`\middle` read a delimiter character by its `\delcode` (tex.web §1160): `DelimiterToken` +
`stomach::digest_as_delimiter`, where a math-active `(` (`\mathcode`(="8000`, active meaning `\left(`) is the
parenthesis (KNOWN_PERL_ERRORS #494), for `\bigl(` too (`TeXDelimiter`, one token); a math-active character whose active
meaning is undefined self-inserts as the plain character, where it was dropped (Perl: Γ, KNOWN_PERL_ERRORS #495):
2609.40266 Fatal Recursion → 0. (3) autobreak bound (latexml_contrib): lines as the package reads them (a line end inside
braces stays in its line, a leading `,`/`.`/`;`/`:` joins the line before), the first non-empty one the left-hand
side, the rest after the tab, the width-driven breaks left out; the raw package needed
amsmath's `\start@align`/`\collect@body`, which the native `align` never runs: 2609.08470 Fatal → 0 (and 2609.04368,
which loads autobreak unused, loses csquotes' failed patch of the raw package's `\collect@body`, 1 → 0, as pdflatex).
A/B over 551 papers (the 157 aastex + every 20th of the first 8000 2609 ids, 62x8 → 62y2): 2 improved, 0 regressed,
1 output changed (2609.01052). Open:
2609.13328's `\def\year{\pdfprimitive\year}` (an ieeeaccess `\def\year#1` workaround) loops, as `\pdfprimitive` is `#1`
(pdftex.rs:611, Perl pdfTeX.pool.ltxml:208 likewise); a faithful one needs the engine's primitive meanings kept in
State past the TeX pools (before plain_bootstrap) — one paper, deferred. Remaining 2609 Fatal witnesses (root causes in
`~/data/pk_agents/main/HANDOFF.md`): 2609.07018 (nested tikzpicture under
`\widetilde`), 2609.14043 (tikz-timing). Guards
`perfect_kernel_batch61::{aastex7_rotate_and_rotatetable, math_active_delimiter_is_its_character,
math_active_character_without_meaning_is_itself, autobreak_lines_after_the_left_side}`.

62x: AASTeX's table columns (aas_support_sty.rs), ported from aastex701.cls (TL 2025 copy). `D` decimal columns
(:12010): after `\decimals` a cell's first word splits at its first `.` (`\lookfordecimal`, :11980), both parts in math
(a sign is a minus), the point only before a fraction, the rest of the cell kept; without `\decimals` the cell stays whole
(each deluxetable resets `\decimals`, :11330; Perl's split uses an undefined `\lx@alignment@align`, KNOWN_PERL_ERRORS
#493). `C`/`L`/`R` are the class's math cells
(:8857-8859), with `$` active in every table (:8849, tabular's and deluxetable's bindings) and doing nothing there, as in
the class's `\nodata` (:8268-8269); `\centerwidetable` (:13497) and the `\movetabledown`/`\movetableright` registers
(:11756, 13690) defined. OXIDIZED_DESIGN_DIVERGENCES #450. Kernel: a `$$` display opens and closes on a math shift by
meaning (tex.web §1138, §1197; tex_math.rs `next_is_math_shift`), so the active `$` pairs in a `p{}` cell; a
`split_tokens_delimited` math span closes as it opens, by meaning.
2609.05675 102 → 0 errors (was Fatal), 2609.00308 102 → 0 (was Fatal), 2609.06985 63 → 0, 2609.04324 12 → 0,
2609.06112 2 → 0; A/B over the 157 aastex papers in the first 8000 2609 ids (62w3 → 62x2): 5 improved, 0 regressed,
10 outputs changed — the 5, 3 more whose `C`/`L` cells are now one math expression (2609.00140, 2609.00283,
2609.07836: `-16.44$\pm$0.07` → `−16.44±0.07`), and 2 whose `\nodata` keeps the class's surrounding spaces. Open: a
`\colhead` over a `D` column covers its integer half only (the class spans both, :12027-12028 `\CheckNumberAndSwitch`;
no witness, `\twocolhead` is the documented form), and the class's `d` (:12011) is the hidden pair, which this binding,
like Perl's, prints as `D`. Guards `perfect_kernel_batch61::aastex_decimal_and_math_columns`,
`perfect_kernel_batch61::active_math_shift_pairs_for_display_math`.

62w: the ar5iv profile's `iflimit` 16M → 48M (ar5iv_sty.rs). Five 2609 pgfplots/tikz papers ended `Fatal:Timeout:IfLimit`
with no error before it; measured without the limit (a release probe build counting conditionals), three are finite and
complete: 2609.10563 (19M, 48 s), 2609.16075 (21M, 45 s), 2609.07725 (39M, 78 s); so do 2605.27177 (raw mhchem, 56 s)
and 2605.04377 (98 s). Any runaway still stops at the worker's 180 s per-document timeout (the dispatcher lease is
240 s); the limit only ends a faster one sooner with its label, and one between ≈89k and ≈267k cond/s now ends at
the timeout. Outcomes that change label: 2609.30783 (lirseg, a memory runaway) now ends at the box-list memory budget
(`Fatal:Stomach:MemoryBudget`, which rides the memory cap: 8 GB probe, 5.25 GiB in the cortex fleet), and 2609.08966 (pgfplots
`shader=interp`, slow) and 2605.12601 at the 180 s timeout — performance items (open). Guard
`perfect_kernel_batch61::ar5iv_profile_sets_its_runaway_limits`.

62u (REGRESSION 2026-10-05, critical): 12,144 of 38,624 2609 papers ended as Error with expl3's "Mismatched LaTeX
support files" + "Cannot run piped system commands" (expl3.sty:64-78). Cause: the worker's `LATEXML_DUMP_DIR`
(`resources/dumps`) held dumps built from `/usr/local/texlive/2025` (L3 2025-11-06), copied there from a gates dump
dir on 2026-10-04 21:28, while the worker reads packages from `/usr/share/texlive` (L3 2026-01-19); every engine
revision fails the same way with that dump (61m17..62s11), all pass with a dump built from the runtime tree. Fix: the
dumps regenerated by `tools/make_formats.sh` in the worker's environment; a dump now records the format files it was
built from (`# source` header: `latex.ltx` + `expl3-code.tex` / `plain.tex`, size + CRC-32) and every loader passes
over one this TeX tree does not have (`dump_paths::dump_matches_tree`: `resolve_versioned_in_dir`, the embedded
dumps, `LATEXML_DUMP_PATH`), the files looked up by tree subpath so a document's own `expl3-code.tex` never
counts; when none matches, `latexml::format_dumps` builds the format in process, once per tree and build, in a
directory with no TeX files, under a lock, into a per-user cache (`~/.cache/latexml-oxide/formats/`,
`$LATEXML_FORMAT_CACHE`), keeps it only if the build logged no error, and uses it, else the base branch (DUMP_DESIGN
"Tree match", DIVERGENCES #449). Dev dumps from before the stamps are passed over: regenerate them with
`tools/make_formats.sh`. Guard `dump_gate_init::a_dump_built_from_another_tree_is_not_used` (an older tree's forged
dumps — bogus stamps, L3 date 2000-01-01 — as the only dumps allowed: expl3's mismatch on the old engine; now both
formats built into the cache with this tree's stamps and loaded from there, on any TeX Live; the document's own
`expl3-code.tex` never read; matching dumps load and build nothing). Perf audit: the stamps cost a CLI process +0.6M
instructions (+0.09%, 78.2 → 78.9 ms on a one-line article, within noise) and `cortex_worker` once per child
(DUMP_DESIGN "Tree match"). 62v: per format, builds that have not succeeded are counted
(`.attempts.<kind>`, so a build dying with its process counts), two in a row mark it `.failed.<kind>` for a day, and a
process that waited while a build failed does not retry it at once; a dump on disk wins over the cache's, the embedded
one comes after it; a failed build is a counted `Warning:dump:build_failed` in each conversion's log; the embedded dumps'
`$TMPDIR` cache is per user and owner-checked (`latexml_core::util::private_files`, unit-tested).
Rerun: run 332 (error/errmessage), 1,532 of whose papers converted during a dump-less minute of a second regeneration
and need regenerating again (dumps must be swapped atomically under a running fleet).

62r: textcomp's symbols are TS1 text symbols that dispatch on the encoding (KPE #489): Greek `\textmu` and babel
greek's `\figurename` print μ, not µ; a document's per-encoding `\DeclareTextCommand` takes effect.

62q: a captioned minipage or parbox in a float is the float its caption numbers (DIVERGENCES #447, KPE #488): its
panel takes the caption's id, tags, list and `\label`s, a table's are tables — side-by-side captioned minipages read
"See 1 and 2", not "2 and 2" (shared with Perl; the common idiom, and 1601.03744's tabular cells).

62p: graphicx's pdftex driver chain at `\begin{document}` (pdftex.def → epstopdf-base → pdftexcmds → iftex, when
`\@curroptions` is not empty) defines `\ifpdf` afresh (KPE #487): a paper's `\let\ifpdf\relax` no longer leaves
JINST's `\label` a stray `\fi` (1310.6454, a Fatal). `\ProcessOptions` and the package loader keep `\@curroptions`
as latex.ltx does, key=value processors leave it (40 bindings say so: `ProcessKeyOptions!`/`key_options_processed`);
amsmath passes `namelimits` to amsopn. 31 synthetic probes agree with pdflatex on iftex/pdftexcmds loading.

62o: a captioned minipage in a tabular cell is a figure panel in the cell (KPE #486): the box placement no longer climbs
out of an alignment (it left `<td>` open in the figure, 5 malformed errors, 1601.03744), and caption material left in a
box in running text becomes a figure panel in an inline logical block (labels: 62q).

62n: tocloft keeps the kernel's lists (DIVERGENCES #446): interpreted raw for its `\cft…` parameters, its
`\tableofcontents`/`\listoffigures`/`\listoftables` replaced by the kernel's under its own condition and times —
they ran `\@starttoc` on a `.toc` LaTeXML never writes, losing the `<TOC>` (SciPost.cls; shared with Perl).

62m: the ACM SIG classes (sig-alternate, sigchi; shipped, run raw since 62k) keep their authors: `\alignauthor` is of the
`\and` family, its brace group and `\affaddr` read as their content (DIVERGENCES #445; open: 1609.00045, 1608.06253), and a missing `figure`/`table` counter is made
once the class has loaded (KPE #485, sig-alternate.cls:699 `\@ifundefined{figure}`). 1605.02827 18 → 1 errors,
2003.09061 24 → 0 (4 creators with affiliations, OmniBus merged them), 1607.07514 22 → 0, 1906.01122 17 → 0,
1707.05754 30 → 0.

62l: a PASJ binding (`latexml_contrib` `pasj00_cls.rs`, also pasj01/pasj02): the shipped class is interpreted raw and
the kernel `\caption` (`\lx@caption`) put back over PASJ's own, which bypassed `\@caption` (captions without number,
dangling labels since 62k); `\@maketitle` emptied; `\KeyWords`, `\affil`, `\altaffiltext`, `\email`, `\orcid`,
`\Received`/`\Accepted` go to the frontmatter; the binding splits the one `\author` itself (commas, `\&`, `\\`; a
name's marks follow its comma). 11 PASJ papers: 0 errors
but 1601.03744 (3, a shared kernel defect, RED), captions back (≥ 62j4), keywords and affiliations gained. RED:
`\caption[<number>]`, captions in minipages in a tabular.

62k: the arXiv profile interprets a shipped class without a binding raw (`localrawclasses` in ar5iv.sty; DIVERGENCES
#444; a binding route, Perl's prefix alternate or Rust's case/basename steps, still wins): OmniBus-class papers 1,151 →
220 errors and 2 → 89 error-free (127), random run-329 error papers 4,645 → 2,714 and 45 → 155 error-free (383),
general sample 752 → 691 errors (873), no body text lost. With it: extsizes bindings (`\@ptsize` in points), `\documentstyle` binding-first probes
and a raw `.cls` probe, tocbibind conditionals (KPE #482), ragged2e `\LaTeX*` saves under `newcommands` (KPE #481),
IEEEtran `\ifCLASSINFOpdf` from `\pdfoutput` (KPE #480, 22 papers), and four kernel paths raw classes reach: an empty
`\@startsection` type (KPE #483), a re-let `\@startsection` recursing through `\@sect` (`\lx@startsection`), the
`\@maketitle` deposit's diagnostics hold, the locked `\NAT@wrout` (KPE #484). Open, worse than OmniBus (DIVERGENCES
#444, RED repros): a paper's `\let\ifpdf\relax` that graphicx's driver undoes
in TeX (JINST, Fatal), JINST's locked-`\author` flag, raa crossed groups, `\alignauthor`/`\newauthor` vs
`\lx@personname`, `\@ifundefined{figure}` seeing the kernel's environment (sig-alternate).

62j: elsarticle `\jtype` (KPE #478) and txfonts `\varv`/`\varw`/`\vary` (KPE #479) fixed; the run's one panic,
1111.1991 (Rust-only: an auto-opened `ltx:picture` sized the `{picture}` whatsit being absorbed and stored the size
memo on it — "RefCell already borrowed"), fixed by `set_memo_property` (a busy box keeps no memo; any other re-entrant
property write is an Error, not a panic). Residuals: `Digested::compute_size` answers 0 for any box it cannot borrow
mutably — one being absorbed too, unless all three of its sizes are requested (1111.1991 passes because the
`{picture}` carries them), not only the astro-ph0310145 self-reference; a `\parbox` after a text `\put` sizes the
auto-opened picture 204.16×22.56 where Perl gives 7.3×5.96 (pre-62j); 0709.1145's `\LT@array`.

From the 62i review: amsmath `multline` drops a `\tag` and numbers the equation (pdflatex "(Q)"; the counter then
runs one ahead) — RED `repros/alignment-bindings/multline_tag_replaces_the_number.tex`.

From the 62h in-scope manual pass (each pdflatex-clean, so Rust-only; both were Fatals on 62g1 and stay failures):
- typog-example: Rust's `\fontname\font` gives the fallback `cmss9` for an NFSS family (`Inter-LF`, T1Inter-LF.fd)
  where pdflatex gives `Inter-Regular-lf-t1 at 9.0pt`, so the document's `\projectoutfontname#1-#2-#3\relax`
  (typog-example.tex:263-264, inside an `\edef`) finds no `-`. 62g1: the argument ran to the end of the file (Fatal);
  62h: "Paragraph ended before \projectoutfontname" ×11 (tex.web §395's "extra }" at the `\edef`'s close is not
  implemented), then the run times out near line 1813.
- chessboard_and_beamer: skak's `\mainline` under beamer `\only<n>` reports "black, not white, to move", then
  `\EatNumberA(#1.#2)` (skak.sty:1319) finds no `.` — 62g1 Fatal at the end of the file, 62h 4 errors.

From run 329 (62a-62c):
- `\jobname.aux` re-read: lamuphys.sty:1240-1247 and caosp.sty:916 redefine `\enddocument` to `\input \jobname.aux`
  under `\if@filesw` (true in Perl and Rust), a file pdflatex has written by then and LaTeXML never writes:
  `Error:missing_file` (cond-mat9607109, astro-ph9805185; 0.7.6 clean).
- Still Fatal after 62b, each erring under pdflatex too: 1409.3401 (pdflatex emergency stop), 2105.00771 (101 = 101), cs0702042 (citesort.sty missing from TL; Rust
  then 101 math-alignment errors, pdflatex 3).
- Rust-only, still open: CJK GB `\@inpenc@undefined` (1007.1512); singles 1205.5844 (`\@journal`), 1711.06710
  (`\@rticle@options`), 1811.00686, 1907.03566, 2004.12109 (memoir font command), 2105.02164, 2203.13766, 2207.02360,
  2306.10394, 2409.00304, astro-ph9805185 and cond-mat9607109 (`.aux` input).
- AMSTeX `\documentstyle{amsppt}` documents still get the LaTeX format (Perl reads them plain). Kept plain they measured
  worse: 64 sampled 1996-99 amsppt papers, 192 → 319 errors, 1 → 2 Fatal (math9806005 0 → `TooManyErrors`; plain
  AMSTeX lacks `\ams@return@opt@arg`, `\rightpoint`; math9603201, math9709201, math9801043, math9803037). Fix the
  plain-AMSTeX gaps first, then classify them plain (`converter.rs` `compile_route`); sample and A/B in
  `~/data/pk_agents/main/scratch_g/run329/amsppt/`.
- `\input eplain` (pre-existing, tex clean): `Error:undefined:\auxfile`, then `Fatal:ParamSpec` "Parameters for `\@`
  not in order" (Perl: the one error).

From 62d/62e: Open, pre-existing (review probes `scratch_g/review62e/r3/r4.tex`, `r5.tex`): with
`linesnumbered` the first statement right after `\caption` takes no number; `{quote}`/`{center}`/`{flushleft}` end no
algorithm line, so the text after them stays on their line; a statement inside `\href` breaks inside the link
(`ltx:ref` does not auto-close). Not fixed: missing journal classes (raa `\pagerange`/`\volnopage`, ws-* `\bodymatter`,
iopart `\ioptwocol`, old A&A `\thesaurus`), text-mode `_`/`&`/`^`, and the Fatals `TooManyErrors` (49),
`never_completed` (24), `PushbackLimit` (21 then, 86 by 126k papers; fixed for harvmac by 62f below).

From 62f: Side risk, open: any genuine `\@ifnextchar` autoload in a plain
  document still replaces the document's macros through the dump apply.
- 62i fixed six of the 17 PushbackLimit Fatals left after 62f: 2108.13640 (siunitx `S` cell read past csvsimple's
  `\relax`, KPE #474), 2407.10582 + 2505.05474 (Rust-only: a document's own `\emails`/`\affiliations` taken for an
  IJCAI author-block marker, `has_ijcai_section_marker`), 2309.08676 + 2502.21053 (`\DeclareEmphSequence`:
  `\selectfont` defined no font identifier, KPE #477), 2408.12869 (`\tag` text mentioning `\theequation`, KPE #475),
  2305.06365 (`\let\@@cite\cite`, KPE #476 / DIVERGENCES #443; its revtex `\@AF@join`/`\twocolumn@sw` stay
  undefined). 0902.2281 and 1007.3028 are AMSTeX documents (`\input amstex` + amsppt: label.def needs amstex.tex's
  `\W@`, `\eat@`), the amsppt-as-plain residual. 2408.12869 then shows 3 `<ltx:XMArray>` in `<ltx:figure>` errors.
- Open, still Fatal: 2302.07191, 2501.17908,
  2509.10120 (error before run 329, Fatal now); unclassified: 1807.10890, 1807.08405, 2107.07104, dg-ga9410001,
  2406.19307. A `\noexpand`-ed undefined control sequence raises "undefined" in csvsimple's
  space trim where TeX treats it as `\relax` (`g62f/c_m.tex`, a CSV cell `\foo` with `\foo` undefined).


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
