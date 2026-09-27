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

*Last compaction: 2026-09-03 — 949 → ~320 lines. Lifted non-actionable upstream review R1, R9-BST historical narrative, ltx_env design notes, and m:menclose/glossaryref parity analyses to `archive/SYNC_SESSIONS_2026-09.md` and dedicated docs. Elevated high-impact fatal seeds (2605.22927/2606.11121 P0/R101 flood) and unblocking class fixes (sn-jnl.cls) to ranked active list. Prior: 2026-09-03 (1276 → 953 lines); 2026-08-18 (1462 → ~890 lines); 2026-07-25 (1979 → ~500 lines).*

---

## Ranked worklist — start here

Ordered by: **does it reproduce today** → **is a real user affected** → **is it unblocked** → **effort**.
High-impact fatal seeds and major publisher class fixes take priority.

| # | item | state | size | detail |
|---|---|---|---|---|
| **R1** | **Fatal-Seed: Perl-0 vs Rust-101 Error Floods** (`2605.22927`, `2606.11121`) | **OPEN**, fresh seed from rc4 60k run. Hits `TooManyErrors:MaxLimit(100)` fatal abort in Rust | medium | Open items §R1 |
| **R2** | **Springer Nature `sn-jnl.cls` Dependency Drop** (witness `2606.00121`) | **OPEN**; raw-load drops `\usepackage{booktabs}` and `\usepackage[title]{appendix}` in `content.rs:2429` | small-medium | Open items §R2 |
| **R3** | **Bibliography-absence campaign** (PR #444) — **291 recovered / 20 338 entries**. Remaining unblocked: **R3d tab-mark parameter scan vs cell read** | **R3d next** (12 papers left, unblocks alignment macro `&` splits) | medium | Open items §R3, [`RESIDUAL.md`](parity/bib_absence_2026-07-29/RESIDUAL.md) |
| **R4** | `--preload=<cls>` trips the LaTeX hook stack (`Extra \PopDefaultHookLabel`) | **OPEN**, re-verified (1 error with `--preload=article.cls`, 0 without). Pool load reordering | medium | Open items §R4 |
| **R5** | **Physical In-Place Image Cropping (`trim`/`clip`)** | **OPEN**, witness `2510.17772` Fig 7. Image metadata scaled but raster uncropped | small | Open items §R5 |
| **R6** | **`Collector::rescan` Refactor for Generated Backmatter** | **OPEN**; `Scan` owns `ObjectDB` by value; generated backmatter lacks full relations/labels | medium | Open items §R6 |
| **R7** | Presentation-MathML **F5** Linebreaker | **OPEN**, full linebreaker feature gap needing a port-or-drop scope decision | family | Open items §R7 |
| **R8** | **Generalized kernel-capability program** (branch `perfect_kernel`; user-approved 2026-09-05) — K1 definition provenance + overlay bindings, K3 lthooks store, K4 templates/sockets, K5 raw-line reader, K2 nest vs save stack (= R9), K6 font model, K7 file model, K8 runaway cap. **Continuation state 2026-09-22** (s108 248 invalid measured, ≈238 projected; next: memory lever B → raw `\author` surpass → singletons → s109): [`PERFECT_KERNEL.md` → Continuation](PERFECT_KERNEL.md#continuation--state-and-next-steps-2026-09-22) | **OPEN**, batches through 56fd landed | program | [`perfect_kernel/KERNEL_CAPABILITIES.md`](perfect_kernel/KERNEL_CAPABILITIES.md) |
| **R8b** | **forest.sty full support** (side goal, user 2026-09-05; heavily used on arXiv) — discard stub today; overlay-binding vs native-tree shape to be decided | **OPEN**, recorded | large | [`perfect_kernel/DIFFICULT_CASES.md` §D10](perfect_kernel/DIFFICULT_CASES.md) |

---

## Current status

- **2026-08-02 — rc4-recut full rerun of sandbox-arxiv-2605+2606 (60,505 docs):**
  - **Overall:** no_problem 6,078/6,359 · warning 19,744/19,724 · error 3,991/4,102 · fatal 266/241.
  - **Fatal clusters:**
    | cluster | size | verdict |
    |---|---|---|
    | `panic:caught` | 3 | **FIXED (PR #491)** — pooled-worker math parser `PENDING_DISCARDS` stale handle sweep on abort. |
    | `TooManyErrors:MaxLimit(100)` | 117 | **REAL seed** — 4/8 sampled REAL, led by **2605.22927 & 2606.11121** (Perl 0 vs Rust 101-flood). |
    | `Stomach:Recursion` | 55 | **MIXED** — 3/8 REAL-by-count (`2605.17696` R144/P56, `2606.05321` R35/P15, `2606.08524` R94/P50). |
    | `Timeout:PushbackLimit` | 120 | Environmental/budget caps, not conversion bugs. |
    | `Timeout:TokenLimit` | 88 | Performance ceiling; legitimate heavy papers. |

---

## Open items — detail for the ranked rows

### R1 — Fatal-Seed: Perl-0 vs Rust-101 Error Floods (`2605.22927`, `2606.11121`)
- **Symptom:** In the 60,505-paper rerun, 117 papers hit `Fatal:TooManyErrors:MaxLimit(100)`. On `2605.22927` and `2606.11121`, Perl converts cleanly with **0 errors**, while Rust cascades past 100 errors and fatally aborts (also `2606.01136` P63/R101, `2605.10685` P7/R101).
- **Action:** Bisect each paper with `latexml --verbose` to identify the initial diverging token/macro. Fixing these primary triggers will recover multiple papers from fatal abortion.

### R2 — Springer Nature `sn-jnl.cls` Dependency Drop (witness `2606.00121`)
- **Symptom:** Springer Nature's standard class `sn-jnl.cls` (1765 lines) raw-loads but drops `\usepackage{booktabs}` (:307) and `\usepackage[title]{appendix}` (:303), causing undefined `\toprule`/`\midrule`/`\bottomrule` cascades.
- **Root Cause:** `maybe_require_dependencies` in [`latexml_core/src/binding/content.rs:2429`](latexml_core/src/binding/content.rs#L2429) fails to extract or load dependencies declared mid-class during raw interpretation.
- **Action:** Trace dependency extraction in `content.rs` and ensure required packages are loaded.

### R3 — Bibliography-Absence Campaign (PR #444 Residuals)
- **R3d: Alignment Parameter Scan vs Cell Read Distinction (`suppressed_tab_marks`):**
  - *Symptom:* An unescaped `&` inside a delimiter-fenced macro argument splits the alignment row and truncates the document (and bibliography). 12 papers remain affected.
  - *Mechanism:* `tex.web` §394 `macro_call` suppresses tab marks while scanning parameters. `SuppressedTabMarks` in [`latexml_core/src/common/local_assignments.rs:194`](latexml_core/src/common/local_assignments.rs#L194) fixed `physics.sty`'s `\mqty` (14 papers), but applying it globally to `Parameters::read_arguments` regressed 5 tests (`cells_test`, `numprints_test`, `xytest_test`, `consort_flowchart_test`, `unit_tests_by_silviu_test`) because that path also reads alignment cell content.
  - *Action:* Distinguish macro parameter scanning from alignment cell reading in `Parameters::read_arguments` so tab marks inside `{...}` do not split outer cells.
- **R3b: No-Diagnostic Chase Candidates (~6 left):**
  - Remaining papers with real `\cite` calls but empty bibliography: `2605.14990`, `2606.05629` (math-in-body silent drop), `2606.10056`, `2606.17491`, `2606.00231`, `2605.29754`.
- **R3g: amsrefs Bare `\begin{biblist}` (4 papers):**
  - `\begin{biblist}` without `{bibdiv}` wrapper $\to$ `malformed:ltx:biblist`. Requires `BACKMATTER_ELEMENT` route.

### R4 — `--preload=<cls>` trips the LaTeX hook stack (`Extra \PopDefaultHookLabel`)
- **Symptom:** `--preload=article.cls` prints `LaTeX hooks Error: Extra \PopDefaultHookLabel` (clean without preload or with `LATEXML_NODUMP=1`).
- **Mechanism:** `\@pushfilename` changes meaning mid-load: `article` is pushed before `LaTeX.pool` loads, using a pre-pool `\@pushfilename` that does not touch `\g__hook_name_stack_seq`. The pool installs expl3's `\@popfilename`, which pops an empty seq and errors.
- **Resolution:** A TeX-side repair or re-synchronizing the sequence at the point `LoadPool('LaTeX')` executes.

### R5 — Physical In-Place Image Cropping (`trim` / `clip`)
- **Symptom:** In `\includegraphics[trim=..., clip]`, [`latexml_core/src/util/image.rs:433`](latexml_core/src/util/image.rs#L433) adjusts metadata dimensions but keeps the original uncropped image, causing browsers to squish the entire raster into the sub-box (witness `2510.17772` Fig 7).
- **Resolution:** Implement `crop_image_inplace` in [`latexml_post/src/graphics.rs`](latexml_post/src/graphics.rs) alongside `rotate_image_inplace` (:771) using `convert -crop`.

### R6 — `Collector::rescan` Refactor for Generated Backmatter
- **Symptom:** Generated subtrees (Bibliographies, Indexes, Glossaries) lose ObjectDB relations, labels, and fragids because `Scan` owns `ObjectDB` by value (`latexml_post/src/scan.rs:49`) and cannot rescan generated nodes.
- **Resolution:** Refactor `Scan` to borrow or take/restore `ObjectDB`, enabling clean rescanning of generated nodes and removing fragile ad-hoc registrations.

### R7 — Presentation-MathML F5 Linebreaker
- **Status:** The only remaining pMML gap from the MathML line audit (F17 is closed). A full linebreaker algorithm gap requiring a port-or-drop scope decision before coding.

---

## Secondary residuals & unranked active items

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
- **A paragraph's trailing whitespace: Perl and Rust split its text into different nodes** (56jy; RED repro `tools/perfect_kernel/repros/block-model/paragraph_trim_text_node_split.tex`): both trim with `s/\s+$//` (all Unicode whitespace; TeX_Paragraph.pool.ltxml:184-196, `document.rs` `trim_node_right_whitespace`) and only the last text node. Perl appends the space after a formula with XML::LibXML `appendChild` (Document.pm:1148-1153), which never merges, and a formula's own text with `appendTextNode` (Document.pm:2123, :2133), which merges into the text before it; Rust's `add_child` (xmlAddChild) merges the space, and `replace_node_as_tree` keeps the formula's text a node of its own. `\item $\,$ ` gives Perl `<p>U+2006</p>` (its golden `t/parse/artefacts.xml`), us `<p/>`; `\item A $\,$\par` Perl `A`, us `A `. `cleanup_Math` runs as the Math element closes in both (Document.pm:1239-1248, tex_math.rs:756), so ordering is not the cause. Cosmetic; a faithful fix appends text without merging, which touches many goldens.
- **An `&` in a glossary or index entry's display raises "Stray alignment" at the mark** (sweep #126, robustindex/robustsample 0 → 2 since 56js made `\glossary` an index mark; RED repro `tools/perfect_kernel/repros/index/glossary_ampersand_robustglossary.tex`): the display phrase is re-tokenized with `&` an alignment tab and digested at the mark, where TeX never typesets it (latex.ltx:17730-17739 writes the `.glo`/`.idx`). Perl: 0 errors for `\glossary` (it drops the entry), 1 for `\index{$x$&…}`. A first fix (56kd review, NO-SHIP) turned a top-level `&` into text but tracked `\begin`/`\(` — tokens already expanded away by `do_expand_partially` (mod.rs:3960), so arrays in index entries lost their tabs; a rework must track the surviving tokens (`\begin{…}`-named CS, `\begingroup`, `\lx@begin@inline@math`/`display@math`, `$$`), keep the tab character, and cover `|see{…}`. Open design point: `\printindex` re-reads the `.ind` with `&` a tab, so pdflatex reports the error when the index is printed — text at the mark would hide it there. Related: `\index{x@\begin{tabular}{cc}a&b\end{tabular}}` gives an empty `<indexmark/>` silently (both binaries); `\@makechapterhead` (book.cls:382) is still undefined (tocbibind's numbered index, biblatex's `refsection=chapter` patch).
- **biblatex `editortype` is not the editor's role** (56kc review; RED repro `tools/perfect_kernel/repros/index-bib/biblatex_editortype_role.tex`): every editor prints "(Ed.)" where biber prints "vocalist Billie Holiday" / "Kilo, Karl, comp." (biblatex.def:2875-2892 `editorstrg`). Giving the role in the BibTeX reader (tried in 56kc) broke the author-year label of author-less entries ("(Hitchcock, 2000)" → "(52)", cms-dates-sample; the fallbacks at make_bibliography.rs ~1076/1212/1354/3640 read only `@role='editor'`), changed `.bst` bibliographies, and exploded an undecoded (ctex) `editortype` byte string. Shape: in the biblatex binding, keep the editor for label/sort/head, carry the type separately, from the decoded field value.
- **A template with no column raises "Extra alignment tab" on every row** (56jz root-causer; SHARED with Perl, Alignment.pm:136-145/240): `\begin{tabular}{}` (or `{|}`) gives an error per row start even without `&`; TeX's preamble always has a column (tex.web:15367-15375, "Missing # inserted" :15468), array adds one for an empty preamble with "Empty preamble: `l' used" (array.sty:332, 412). Shape: in `read_alignment_template`, no columns and no repeated columns → one `l` column plus that error. Witnesses platex-tools/plextarray, plextcolortbl (plext's `\begin{tabular}<t>{…}` direction argument, pTeX family, parked).
- **Measured box sizes, K18 step 2** (step 1 landed in 56kj; repro `tools/perfect_kernel/repros/boxes-groups/box_dimensions_measured.tex` GREEN): `\raisebox`'s raise does not measure the box (`\raisebox{-.5\height}{icon}`, 13 papers): it waits for `yoffset` to render with reserved space, since LaTeXML-common.xsl's `position:relative; bottom:` lets a true lowered icon overflow into the next row (RED repro `boxes-groups/raisebox_raise_measures_the_box.tex`). A `\makebox` in script math measures its text at the script size (RED repro `boxes-groups/makebox_in_script_math_text_size.tex`, 2605.18608). `\resizebox{\width}{!}{…}` is `xscale="0"` and `\resizebox*` behaves as `\resizebox`: `\Gscale@@box`'s `GraphixDimension` sizes are not yet read with the box set, as 56kl's `TempboxaDimension` are (4 papers). `\includegraphics[height=1em]` measures 10.1178pt (pdflatex 10.00002pt): its size goes through whole pixels at 100 DPI (RED repro `graphics-tikz/includegraphics_size_not_pixel_rounded.tex`). An empty `\raisebox{d}[]{x}` reads 0pt with Missing number where latex.ltx treats it as not given (0 uses; RED repro `boxes-groups/raisebox_empty_height_is_not_given.tex`). `\resizebox{1em}{!}{x}` measures 10.00002pt, Perl's, where pdflatex gives 10.00081pt through graphics.sty's `\Gscale@div`. makecell (`makecell_sty.rs:192-193`), diagbox (`diagbox_sty.rs:288-289`) and rotate.sty's `\rotate[u|f]` (`rotate_sty.rs:43-45`, Perl rotate.sty.ltxml:44-46 stores Dimensions) still store string sizes, which `compute_size_and_cache`'s complete-size bypass (56kq) cannot read, so the box is measured and reports its unrotated content; rotating.sty shares `rotated_properties` and is typed. Still unported from Perl's `computeSizeStore` (Box.pm:266-297): a partly specified size keeping its given parts over the computed ones, and `isEmpty` (`docs/archive/UPSTREAM_SYNC_2767_to_2833_2026-06-26.md` U2/S5). `set_width`/`set_height`/`set_depth` (`latexml_core/src/lib.rs`) write only the declared value, where Perl's `setWidth` etc. (Box.pm:211-230) also update the computed copy, and `fobj_get_size` (`tex_box.rs:75-118`) prefers a stored computed size and never computes a missing one (Perl `$whatsit->getSize`, TeX_Box.pool.ltxml:410): a `\ht` assigned after a size was computed stays stale for a foreignObject unless the box's size is complete (56kq's bypass then applies). Needs a ruling: `\ ` is 0.5em wide (Perl's constant, TeX_Character.pool.ltxml:28-30) where TeX uses the font's interword space (`\spaceskip` if nonzero, tex.web §1041-1044): a node with a control space is 0.167em per `\ ` wider than pdflatex. Also open: a detector for a String-valued size key in `compute_size_and_cache`, and an audit of box constructors without a sizer (79 `sizer =>` sites against 945 `DefConstructor!`). See `docs/perfect_kernel/KERNEL_CAPABILITIES.md` K18.
- **`\fcolorbox` ends the running paragraph and loses the space after it** (56kb review; RED repro `tools/perfect_kernel/repros/boxes-groups/fcolorbox_splits_paragraph.tex`; SHARED): `X \fcolorbox{red}{blue}{Mid} et` gives `<p>X</p><p><text framed>Mid</text>et</p>`; pdflatex one paragraph "X Mid et" (color.sty:159-164 `\color@b@x` = `\leavevmode` + `\hbox`). Perl's constructor (color.sty.ltxml:107-115, xcolor the same) digests the text in `internal_vertical` mode, as ours (color_sty.rs, xcolor_sty.rs): the content opens a paragraph of its own.
- **babel-french high punctuation keeps the space before it** (56kb review; RED repro `tools/perfect_kernel/repros/babel-lang/french_highpunct_unskips_space.tex`): `Mid bold ; suite.` gives U+0020 U+2006 before `;`; pdflatex's active `;` removes the preceding space (`\unskip`) before its thin space. After a word as after a closed group or box (matapli-doc, 6 places).
- **A block box at a paragraph start splits the paragraph** (56kb review; SHARED): `\rule`, `\parbox`, `minipage`, `tabular` first in a paragraph (latex.ltx 16361, 16250, 16306, 16560 start with `\leavevmode`) come out as separate blocks, and the text after them ("et B.") in a paragraph of its own.
- **Cells at a column rule or `@{..}`: alignment and padding classes** (56jy review; RED repro `tools/perfect_kernel/repros/alignment/column_rule_edge_padding.tex`): a leading arydshln `:` right-aligns the first `c` cell (Perl `align="left"` with `ltx_nopad_l`; pdflatex centres it), and the cell keeps the rule's U+2002 U+200A gap as leading text, doubling the CSS padding; `c@{\,}c` marks no `ltx_nopad_l` on the right cell where Perl does. Shared with Perl: a `c` column before `:`/`;` reads `align="right"` `thead="row"` (Perl's right-to-left cell scan stops at the class command, TeX_Tables.pool.ltxml:505-520), and the `@{..}` material itself is dropped.
- **`newunicodechar` four-hex `^^^^` caret support** (~119 docs, e.g. `2606.00241`): Adding 4-hex/6-hex caret parsing to `mouth.rs:get_next_char` allows `newunicodechar` to take its Unicode branch cleanly.
- **`floatrow` raw-load (witness `2606.10047`)**: Floatrow reroutes subcaption placement, causing 18 `malformed` errors in Rust vs 0 in Perl.
- **Scanner-status residuals** (batch 56jg, DIVERGENCES #310): commands Rust implements as primitives/constructors (`\setcounter`, `\lstnewenvironment`, `\newtcbinputlisting`) need a primitive-level abort for TeX's matching recovery; `skipping` is not modelled.
- **A file end in a `GeneralText`/`XGeneralText` brace hunt is not an error** (batch 56jj residual, DIVERGENCES #313): `\uppercase`/`\lowercase`/`\detokenize`/`\message`/`\errmessage`/`\mark`/`\marks`/`\special` as a file's last token — pdflatex 2 errors ("File ended while scanning text of \uppercase", "Missing { inserted") and "A abc B", Rust and Perl 0 and "A ABC B". Order: stop setting `matching` around expandable primitives' typed parameters (`Expandable::read_call_arguments`, `definition/expandable.rs:325-340`; `\csname`, `\number`, `\readline`), then let `read_x_token` call `recover_at_file_end` at non-normal status; the brace hunt (`skip_filler`) also runs at the caller's status and skips no `\protected` expansion, where `read_tokens_value` hunts with protected expansion — one hunt for both. Red repro `tools/perfect_kernel/repros/expansion-primitives/uppercase_brace_hunt_file_end.tex`.
- **A braced typed argument's rest: the residuals** (batch 56jr resolved the silent loss, OXIDIZED_DESIGN_DIVERGENCES #317 / KNOWN_PERL_ERRORS #275: the rest re-enters the input after the command, assignments scan by the register's type, calc — loaded in most arXiv papers — evaluates a braced length whole; an undefined control sequence the scan met is discarded, as TeX does, and a rest met between alignment rows is dropped with a warning: the review regressions 2605.25073, 2605.08378, 2605.27476, 2605.25272 are fixed; batch 56ju fixed the TL-manual A/B regressions, each a binding that read less than TeX — multido's Number variable, nicematrix `X[<keys>]`, tabularray `\NewColumnType` and rule options, pstricks angles read whole, `\resizebox` lengths through calc, calc loaded by the pdfcomment/diagbox/animate/savetrees/breqn/jmlr bindings as by their packages, `\DeclareTextAccent`'s stored slot). Open: `\width`/`\height`/`\depth`/`\totalheight` are `0pt` inside `\resizebox` (sect12.rs:184-187, shared with Perl; perfectcut's delimiters get `xscale="0"`); tabularray's `cmd=` key is unapplied; a box or space command's tail comes one step late — `A\hspace{1em\foo}B` gives "A xB" with a warning (pdflatex "Ax B"), inside a `minipage`/`tabular*` body for those; `\vspace` is Perl's `\vskip #2\relax` (latex.ltx:9362-9372 sets `\sp@ce@skip` first); a keyval value's rest (`Parameter::reparse`, keyvals.rs) is still dropped; a column type's rest (`p{3cm\foo}`) is dropped with a warning where TeX typesets it in each cell (2605.19386 `;{1pt/1pt}` under the arydshln stub); a `DigestedBody` constructor's (`\@@tabularx`) comes after the whole body; `\lx@@genfrac`'s `after_digest` (amsmath_sty.rs) reads its numerator after a re-inserted rest. Repros `tools/perfect_kernel/repros/expansion-primitives/braced_*`, guards `braced_quantity_tail::*`.
- **A latexml.sty preload loads the LaTeX format before the document** (56jl triage; RED repro `tools/perfect_kernel/repros/loader/latexml_preload_keeps_plain.tex`): its body's `\AddToHook` (the dvips pagecount hook, `latexml_sty/mod.rs`) autoloads LaTeX.pool (`latex_kernel.rs`), as do `\ExplSyntaxOn`/`\NewDocumentCommand`/`\lua_*`/`install_unicode_format_encoding` under the luatex and xetex profiles. Right for a LaTeX document (pdflatex's model: `\newcount\foo \foo=7 \documentclass…` keeps 7, ar5iv's locked `\today` survives), wrong for a plain TeX one (`\@latexerr` defined: pstricks.tex:39-41 skips pstricks-tex.def, 3 errors in `graphics-tikz/pstricks_input_plain` under the raw preload). Deferring the hook to the "format loaded" seam alone moved every LaTeX document's format load to `\documentclass` (as Perl): ar5iv's `\today` printed the conversion date and pre-`\documentclass` assignments were lost to the dump replay — reverted. A faithful fix needs both: the lazy load must not clobber assignments made before it (a general capability; Perl and no-preload Rust lose them too), and ar5iv's `\today` at begin-document (Perl ar5iv.sty.ltxml). Side finding: the luatex/xetex profiles pick `l3backend-dvips.def`, because the profile's `\ExplSyntaxOn` raw-loads expl3.sty, whose `\sys_load_backend:n{}` (expl3.sty:153-154) runs before the engine identity is set (lualatex: l3backend-luatex, xelatex: l3backend-xetex) — needs a RED repro.
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
- **Binding closures that assign as they expand: the unflagged rows** (batch 56jw audit, KNOWN_PERL_ERRORS #286): a number scan still runs `\IfFileExists`/`\InputIfFileExists` (sect13.rs, robust in latex.ltx:9667/9710), `\AtEndOfPackage` (sect05.rs), `\ProvidesPackage`/`\ProvidesFile`, `\@startsection` (sect04.rs), the `\e@alloc` allocators, amsthm `\swapnumbers`, paralist `\setdefaultenum`/`\setdefaultitem`, xkeyval `\setrmkeys`/`\key@ifundefined`/`\disable@keys`/`\DeclareOptionX*`, IEEEtran `\IEEEQEDhere`, multido `\fpAdd`/`\fpSub`, contrib biblatex `\DeclareCiteCommand`/`\DeclareMultiCiteCommand`, tabularray `\SetTblrInner` and chemnum `\resetcmpd`, whose originals reach an unexpandable token before they act. None has a corpus witness; each wants its repro before `peeks_by_futurelet`. ntheorem's `\theoremheaderfont` and siblings are macros where ntheorem.sty:628-634 declares token registers.
- **A constructor template's `?#N` holds for "0"** (W12 review; `latexml_codegen/src/constructable.rs:301-310` `emit_bool`): Perl's conditional is the argument's string (Constructor/Compiler.pm:166), false for "0" and "". `\qbezier`/`\bezier` work around it (DIVERGENCES #316); the generic fix changes many goldens (its own batch).

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
