//! Cluster-regression integration tests — package and engine clusters.
//!
//! Pins the surpass-Perl wins from the post-100k cluster work (NBSP,
//! `\@ifundefined`, setdec/dec, `\CITE`) as 0-error. If a future change
//! re-introduces the cluster errors, CI fails before the PR can land.
//!
//! Siblings, all sharing [`mod cluster`](cluster): `06_cluster_math`,
//! `06_cluster_bibliography`, `06_cluster_frontmatter`,
//! `cluster_xslt_split` (was `06_cluster_toc_navigation`), `06_cluster_standalone_subfiles`.

mod cluster;
use cluster::{
  convert_and_post_clean, convert_and_post_pmml_clean, convert_and_post_pmml_contrib_clean,
  convert_clean, convert_expecting_errors, convert_log, convert_to_xml, convert_to_xml_contrib,
  convert_to_xml_contrib_clean,
};

#[test]
fn cluster_nbsp_csname() { convert_clean("tests/cluster_regressions/nbsp_csname.tex"); }
#[test]
fn cluster_at_ifundefined() { convert_clean("tests/cluster_regressions/at_ifundefined.tex"); }
#[test]
fn cluster_setdec_dec() { convert_clean("tests/cluster_regressions/setdec_dec.tex"); }
#[test]
fn cluster_cite_uppercase() { convert_clean("tests/cluster_regressions/cite_uppercase.tex"); }
/// `\let\cline\cmidrule` (a common booktabs idiom) must NOT create a
/// `\cmidrule`->`\cline`->`\cmidrule` infinite expansion. LaTeXML's booktabs
/// binding defines `\cmidrule` via `\cline`, so the `\let` would loop until the
/// 8M-conditional IfLimit fatal unless `\cmidrule` routes through a private
/// saved `\cline` (`booktabs_sty.rs` `\ltx@saved@cline`). Shared with Perl
/// LaTeXML (which hangs); Rust surpasses. Witnesses: arXiv 2506.23179, 2511.17056.
#[test]
fn cluster_cmidrule_cline_let() {
  convert_clean("tests/cluster_regressions/cmidrule_cline_let.tex");
}
/// fvextra's `breakanywhere=true` installs a recursive char-by-char break
/// scanner that measures every character by boxing a line-prefix. In our
/// engine that recursed through `predigest_box_contents_in_mode` and grew the
/// gullet pushback until the 650000 `Timeout/PushbackLimit` Fatal — where Perl
/// converts cleanly. The `fvextra_sty` binding routes the breaking
/// line-processor to the non-breaking one (line wrapping is a PDF-visual
/// concern with no HTML semantics), so the verbatim completes with the
/// `font="typewriter"` styling preserved. Drove 119/121 fatal papers in the
/// sandbox-arxiv-2605 corpus (witness arXiv 2605.01024).
#[test]
fn cluster_fvextra_breakanywhere() {
  convert_clean("tests/cluster_regressions/fvextra_breakanywhere.tex");
}
/// fvextra loaded after fancyvrb must NOT strip the `ltx_verbatim` css class
/// from `Verbatim` lines. `fancyvrb_sty.rs` installs the class by wrapping
/// `\FancyVerbFormatLine` (`\lx@add@cssclass{ltx_verbatim}…`); fvextra.sty L2249
/// then `\def\FancyVerbFormatLine#1{#1}` overwrites that wrapper, so every line
/// loaded after fvextra lost `class="ltx_verbatim"` (and its `white-space:pre`)
/// — the verbatim collapsed to ordinary typewriter text, silently, 0 errors.
/// `fvextra_sty.rs` re-installs the hook over fvextra's redefinition. Witness:
/// issue #502 (fancyvrb + fvextra; pre-fix 0 `ltx_verbatim`, post-fix one per line).
/// arXiv/html_feedback#6903: two-column subfigure panels with no explicit
/// `{width}` — `\subcaptionbox` and subfig `\subfloat` — are sized to the full
/// `\hsize` and were stacked one-per-row (each full text width). `arrange_panels`
/// now sizes such a panel to its sole graphic, so panels narrower than the column
/// share a row like an explicit-width `\begin{subfigure}{0.48\linewidth}` already
/// does. The fix only ever SHRINKS a full-width panel to a narrower graphic, so a
/// panel whose content is genuinely full width must still stack — guarded here so
/// no future change makes the reflow over-eager.
///
/// Read from the core XML via `<break>`, the "start a new row" marker: two narrow
/// figures (fig 1-2) with NO break between their panels = shared row; one
/// full-width figure (fig 3) with exactly ONE break = still stacked.
#[test]
fn cluster_subfigure_panels_share_a_row_6903() {
  let xml = convert_to_xml("tests/cluster_regressions/subfigure_panel_wrapping_6903.tex");
  assert_eq!(
    xml.matches("ltx_figure_panel").count(),
    6,
    "expected 6 subfigure panels marked ltx_figure_panel (2 per figure):\n{xml}"
  );
  // arrange_panels inserts one `<break>` per stacked row transition. Only the
  // full-width-content figure (3) stacks; the two narrow figures share a row.
  assert_eq!(
    xml.matches("<break").count(),
    1,
    "narrow subcaptionbox/subfloat panels must share a row (0 breaks), and the \
     full-width-content figure must still stack (1 break) — got a different \
     break count, so the reflow is either not firing or too eager:\n{xml}"
  );
}
/// brucemiller/LaTeXML#2563: loading `svg` (which does `RequirePackage('subfig')`)
/// after `subcaption` must not break subfig's `\subfloat`. Perl 0.8.8 gates
/// subfig's `\lx@subfloat@figure` behind `\@ifundefined{c@subfigure}{\newsubfloat
/// {figure}}{}` (subfig.sty.ltxml:114) — a guard on the COUNTER, which subcaption
/// already defined. So Perl skips the definition and `\subfloat` leaks its args as
/// literal text: `<p>[This is a caption.]This is a figure.</p>`. Rust's
/// `subfig_sty.rs` defines the subfloat macros unconditionally (NewCounter is
/// idempotent), so the panel + its subcaption survive. Guards that surpass-Perl win.
#[test]
fn cluster_svg_subfloat_survives_subcaption_2563() {
  let xml = convert_to_xml("tests/cluster_regressions/svg_subfloat_2563.tex");
  // The Perl breakage signature: the optional arg dumped verbatim, unparsed.
  assert!(
    !xml.contains("[This is a caption.]"),
    "svg+subcaption broke \\subfloat — its optional arg was dumped as literal text \
     (#2563 Perl breakage). Expected a figure panel, not a raw `[...]`:\n{xml}"
  );
  // The subcaption must render as an actual caption, and the body as the panel.
  assert!(
    xml.contains("<caption") && xml.contains("This is a caption."),
    "\\subfloat's subcaption was lost — expected a <caption> carrying \
     'This is a caption.':\n{xml}"
  );
  assert!(
    xml.contains("This is a figure."),
    "\\subfloat's body content was lost:\n{xml}"
  );
}
/// arXiv/html_feedback#6895: `\RequirePackage{scalerel}` leaves `\scalerel`
/// undefined (no binding in Perl or Rust; the raw `.sty` load fails to define it),
/// so an inline icon built with `\scalerel*` (the `\orcidicon` of arXiv:2608.12272)
/// raised `Error:undefined:\scalerel` and rendered its picture unscaled. The
/// `scalerel_sty` binding defines `\scalerel`/`\stretchrel` so the object scales to
/// the reference's height. Beyond Perl 0.8.8 (which errors identically). Witness:
/// 2608.12272.
#[test]
fn cluster_scalerel_defined_6895() {
  convert_clean("tests/cluster_regressions/scalerel_icon_6895.tex");
}
/// arXiv/html_feedback#6909 (witness 2606.08266): Pandoc's default relative-width
/// table column `p{(\columnwidth - N\tabcolsep) * \real{X}}` is a calc infix
/// expression the base dimension reader could not evaluate, so every column
/// collapsed to `width="0.0pt"` and the cell text wrapped one character per line
/// ("a river of characters"). Column widths now route through the calc expression
/// parser when calc is loaded. Surpasses Perl 0.8.8 (which emits the same 0pt +
/// `Missing number` warning); pdflatex renders the real widths. OXIDIZED_DESIGN #141.
#[test]
fn cluster_pandoc_calc_colwidth_6909() {
  let xml = convert_to_xml("tests/cluster_regressions/pandoc_calc_colwidth_6909.tex");
  // Canary: a zero-width p{} column is the corruption.
  assert!(
    !xml.contains(r#"width="0.0pt""#),
    "a Pandoc calc column width collapsed to 0pt (#6909) — the calc expression \
     `(\\columnwidth - N\\tabcolsep) * \\real{{X}}` was not evaluated:\n{xml}"
  );
  // The two \real factors (0.30 / 0.70) of (345pt - 4*6pt) = 321pt give distinct,
  // proportional widths: 96.3pt and 224.7pt.
  assert!(
    xml.contains(r#"width="96.3pt""#) && xml.contains(r#"width="224.7pt""#),
    "Pandoc calc column widths are not the expected proportional 96.3pt / 224.7pt \
     (30%/70% of 321pt):\n{xml}"
  );
}
/// Issue #719 (witness: user MWE): with `\setlength{\parindent}{0pt}`, the FIRST
/// paragraph must be marked `ltx_noindent` so the stylesheet's default 2em
/// first-line indent (`ltx-article.css` `.ltx_para > .ltx_p:first-child`) does
/// not apply — pdflatex shows no indent. Perl LaTeXML emits byte-identical XML
/// (only the 2nd+ paragraphs carry the class, since `\par` records it for the
/// NEXT paragraph and the first has no prior `\par`); Rust surpasses by also
/// stamping the first paragraph from the live `\parindent`. OXIDIZED_DESIGN #143.
#[test]
fn cluster_first_para_noindent_719() {
  let xml = convert_to_xml("tests/cluster_regressions/first_para_noindent_719.tex");
  // Canary: the FIRST <para> must now carry ltx_noindent (it did not before).
  assert!(
    xml.contains(r#"<para class="ltx_noindent" xml:id="p1">"#),
    "first paragraph is not marked ltx_noindent despite \\parindent=0pt (#719):\n{xml}"
  );
  // The 2nd paragraph keeps its class (deferred mechanism, unchanged).
  assert!(
    xml.contains(r#"<para class="ltx_noindent" xml:id="p2">"#),
    "second paragraph lost its ltx_noindent class (#719 regression):\n{xml}"
  );
  // Control: DEFAULT (nonzero) \parindent must leave NO paragraph noindent —
  // the surpass fires only on a genuine zero \parindent, never by default.
  let def = convert_to_xml("tests/cluster_regressions/first_para_indent_default_719.tex");
  assert!(
    !def.contains("ltx_noindent"),
    "a paragraph was wrongly marked ltx_noindent under the DEFAULT \\parindent \
     (#719 over-application):\n{def}"
  );
}
/// Issue #719, no-dump path. The first landing keyed the first-paragraph stamp on
/// a `seen_first_para` state one-shot, which a begin-document `\par` consumed
/// before the first content paragraph under the no-dump sequence — so the fix
/// silently reverted there (and in CI, whose freshly-generated dump takes the same
/// path). The stamp is now structural (first `ltx:para` of its parent), robust to
/// stray `\par` timing. This subprocess drives the binary with `LATEXML_NODUMP=1`
/// (env-isolated to dodge the shared-env race) to guard exactly that path.
#[test]
fn cluster_first_para_noindent_nodump_719() {
  use std::process::Command;
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  let dir = tempfile::tempdir().expect("tempdir");
  std::fs::write(
    dir.path().join("m.tex"),
    "\\documentclass[12pt]{article}\n\\setlength{\\parindent}{0pt}\n\
     \\begin{document}\nFirst line\n\nsecond lines\n\\end{document}\n",
  )
  .expect("write m.tex");
  let status = Command::new(bin)
    .args(["m.tex", "--dest", "m.xml", "--nocomments"])
    .env("LATEXML_NODUMP", "1")
    .current_dir(dir.path())
    .output()
    .expect("spawn latexml_oxide");
  let xml = std::fs::read_to_string(dir.path().join("m.xml")).unwrap_or_default();
  assert!(
    status.status.success() && xml.contains(r#"<para class="ltx_noindent" xml:id="p1">"#),
    "no-dump first paragraph is not marked ltx_noindent despite \\parindent=0pt (#719):\n{xml}"
  );
}
/// Issue #722: the optional `[dimen]` of `\\[20pt]` (extra vertical space at a
/// forced line break) is preserved as a themeable CSS custom property
/// `--ltx-break-space` on `ltx:break`. Perl parses the glue and drops it
/// (`ltx:break` has no spacing slot); we surpass it, default-inert. Plain `\\` and
/// `\\[0pt]` stay bare. OXIDIZED_DESIGN #142.
#[test]
fn cluster_break_optional_glue_722() {
  let xml = convert_to_xml("tests/cluster_regressions/break_optional_space_722.tex");
  assert!(
    xml.contains(r#"<break cssstyle="--ltx-break-space:20.0pt"/>"#),
    "\\\\[20pt] should preserve its optional glue as --ltx-break-space on ltx:break:\n{xml}"
  );
  // Exactly one break carries the variable — plain \\ and \\[0pt] must stay bare.
  assert_eq!(
    xml.matches("--ltx-break-space").count(),
    1,
    "only the \\\\[20pt] break should carry --ltx-break-space (plain \\\\ and \\\\[0pt] \
     stay attribute-free):\n{xml}"
  );
}
/// arXiv/html_feedback#6924 (witness arXiv 2608.10928): the paper sets `\title{}`
/// (structured `<ltx:title>`) but never calls `\maketitle`; it hand-typesets the
/// title (and authors) as a leading centered display-font block. LaTeXML captured
/// the structured title AND kept the ink → the title rendered twice. We prioritize
/// the structured metadata: the redundant leading title-ink is removed, the
/// semantic `<ltx:title>` kept, and the author ink (no structured counterpart) is
/// preserved.
#[test]
fn cluster_frontmatter_title_ink_dedup_6924() {
  let xml = convert_to_xml("tests/cluster_regressions/frontmatter_title_ink_dedup_6924.tex");
  // The structured title survives.
  assert!(
    xml.contains("<title>"),
    "the structured <ltx:title> must remain:\n{xml}"
  );
  // The title text now appears exactly ONCE (structured only; the body ink is gone).
  assert_eq!(
    xml.matches("My Great Title").count(),
    1,
    "title should appear once (structured), not duplicated as body ink:\n{xml}"
  );
  assert_eq!(
    xml.matches("A Longer Subtitle").count(),
    1,
    "subtitle line once:\n{xml}"
  );
  // The hand-typeset AUTHOR block has no structured counterpart, so it stays.
  assert!(
    xml.contains("Jane Q. Author"),
    "author ink must be preserved:\n{xml}"
  );
}
/// arXiv/html_feedback#6569 (witness arXiv 2410.00317): a nicematrix
/// `bNiceMatrix[first-row,first-col]` with a `\CodeBefore … \Body` cell-coloring
/// block. Beyond-Perl (no Perl `nicematrix.sty.ltxml`): the family reduces to a real
/// bracketed math array (`ltx:XMArray`), each `\rectanglecolor{blue!15}{i-j}{k-l}`
/// fills its mapped `XMCell`s with `backgroundcolor`, and the first-row/first-col
/// label cells are marked `thead` — NOT a discarded placeholder + `Error:undefined`.
/// The four rects color exactly the 6 nonzero rigidity-matrix entries
/// (`{1-1},{1-3},{2-1..2-2},{3-2..3-3}` = 1+1+2+2). See `nicematrix_sty.rs`.
#[test]
fn cluster_nicematrix_codebefore_6569() {
  let xml =
    convert_to_xml_contrib_clean("tests/cluster_regressions/nicematrix_codebefore_6569.tex");
  // Stage 1: the matrix renders as a real array, no placeholder leak.
  assert!(
    xml.contains("<XMArray"),
    "matrix should render as a real ltx:XMArray:\n{xml}"
  );
  assert!(
    !xml.contains("nicematrix-placeholder"),
    "no nicematrix placeholder note should remain:\n{xml}"
  );
  // Stage 3: the \CodeBefore rects color exactly the 6 blue!15 cells.
  let bg = xml.matches("backgroundcolor=").count();
  assert_eq!(
    bg, 6,
    "expected 6 blue!15 backgroundcolor cells, got {bg}:\n{xml}"
  );
  // Stage 2: first-row/first-col labels carry thead.
  assert!(
    xml.contains("thead="),
    "first-row/first-col label cells should be marked thead:\n{xml}"
  );
  // End-to-end: the cell colors must survive MathML post-processing onto the
  // `m:mtd` (as `mathbackground`, which the XSLT turns into the `--ltx-bg-color`
  // theming variable the CSS paints). Guards the pmml `pmml_array` carry.
  let pmml =
    convert_and_post_pmml_contrib_clean("tests/cluster_regressions/nicematrix_codebefore_6569.tex");
  let mtd_bg = pmml.matches("mathbackground=").count();
  assert_eq!(
    mtd_bg, 6,
    "expected 6 m:mtd carrying mathbackground, got {mtd_bg}:\n{pmml}"
  );
}
/// arXiv/html_feedback#6569 (PR-review regression): two Nice matrices sharing ONE
/// display must each paint their OWN array. The `\lx@nicematrix@applycolors`
/// color-walk targeted the FIRST matrix-XMDual under the shared `ltx:XMath`, so
/// the second matrix's `\CodeBefore` color leaked onto the first (and could even
/// miscolor an adjacent plain `pmatrix`). Fixed by selecting the LAST
/// matrix-XMDual (the just-closed one). RED before the fix: the first array held
/// BOTH colored cells.
#[test]
fn cluster_nicematrix_multi_matrix_no_color_leak_6569() {
  let xml =
    convert_to_xml_contrib_clean("tests/cluster_regressions/nicematrix_multi_display_6569.tex");
  // Two matrices, one colored cell each → exactly two backgrounds total.
  assert_eq!(
    xml.matches("backgroundcolor=").count(),
    2,
    "each of the two matrices colors exactly one cell:\n{xml}"
  );
  // The leak painted both onto the FIRST matrix's array; assert the first
  // `<XMArray>` holds exactly ONE colored cell, not both.
  let start = xml.find("<XMArray").expect("an XMArray must be present");
  let first_array = &xml[start..];
  let end = first_array.find("</XMArray>").expect("a closed XMArray");
  assert_eq!(
    first_array[..end].matches("backgroundcolor=").count(),
    1,
    "the first matrix must hold only its own colored cell, not the second's:\n{xml}"
  );
}
/// dginev/latexml-oxide#740 + #742 (reporter nasser1): vertical rules in a math
/// `array`, two independent defects fixed to byte-for-byte Perl parity.
///
/// #740 (POST): `cc|c`, framed `|c|c|`, and colortbl `\arrayrulecolor` rendered as
/// bare `<m:mtd>` — the rule vanished. The CORE XML already carried the correct
/// `border="r"`/`border="b l r"` on each `XMCell` (identical to Perl); the loss was
/// in `pmml_array`, which never read `border`. Ported Perl `MathML.pm` L456-475:
/// `border` → space-joined `ltx_border_*` classes (and `thead` → `ltx_th_*`, folded
/// with any explicit `class`).
///
/// #742 (CORE): the `array`-package `!{|}` (`@{}cc!{|}c@{}`) inserts its filler past
/// the column's trailing `\hfil`, defeating the centering, so Perl right-aligns that
/// cell (`align="right"`). Rust's `expected_from_template` fallback in
/// `extract_alignment_column` — a Rust-only patch for trailing fills lost in nested
/// `\hbox` digestion — wrongly restored Center. `template_after_fill_defeated` now
/// detects the defeated fill (real content after the last `\hfil` in the `after`
/// template; a `\vrule` rule is skippable and does NOT defeat it) and the fallback
/// leaves the Perl-faithful Right. RED before the fixes: zero `ltx_border_` classes,
/// and the `!{|}` cell centered.
#[test]
fn cluster_array_vertical_rule_border_740() {
  let pmml = convert_and_post_pmml_clean("tests/cluster_regressions/array_vertical_rule_740.tex");
  // #740 — `cc|c` right rule on the middle column of each of two rows.
  assert_eq!(
    pmml.matches("ltx_border_r").count(),
    4,
    "expected 4 m:mtd with a right rule (2 from cc|c, 2 from the framed array):\n{pmml}"
  );
  // #740 — framed `|c|c|` + `\hline` corner cell folds three borders into one class.
  assert!(
    pmml.contains(r#"class="ltx_border_b ltx_border_l ltx_border_r""#),
    "framed corner cell should fold b/l/r into one class attribute:\n{pmml}"
  );
  // #740 — `\hline` top rule.
  assert!(
    pmml.contains("ltx_border_t"),
    "the \\hline should surface as a top-rule class:\n{pmml}"
  );
  // #742 — the two `!{|}` cells (columns q,t) are right-aligned in the core XML …
  let xml = convert_to_xml("tests/cluster_regressions/array_vertical_rule_740.tex");
  assert_eq!(
    xml.matches(r#"<XMCell align="right">"#).count(),
    2,
    "the two `!{{|}}` cells must be right-aligned like Perl, not centered:\n{xml}"
  );
  // … and that survives into the rendered MathML as `ltx_align_right`.
  assert_eq!(
    pmml.matches("ltx_align_right").count(),
    2,
    "the two `!{{|}}` cells should render `ltx_align_right`:\n{pmml}"
  );
}
/// arXiv/html_feedback#6681 (reporter younesmouhib, paper 2606.22155v1): "does
/// not compile properly: half missing". The deployed page (LaTeXML oxide 0.7.5)
/// dumped LaTeXML-internal constructors (`\@@listings@block`,
/// `\@@numbered@section`, `\lx@bibliography`) as literal text from the second
/// `lstlisting` onward, swallowing the whole document tail — Open questions,
/// bibliography, and the Verification appendix all became macro soup. The fix
/// landed after that build; current main converts the paper end-to-end. This
/// pins the fixed behaviour on the paper's construct: `listings` blocks with
/// `breaklines=true`/`columns=fullflexible` inside `table`s, followed by more
/// sectioning and an appendix. A regression would re-leak the internal
/// constructors and drop the tail. (Any `@@…` in the output is a leak: the
/// verbatim listing source lives base64-encoded in `data=`, whose alphabet
/// excludes `@`, so it can never false-match.)
#[test]
fn cluster_listings_tail_leak_6681() {
  let xml = convert_to_xml("tests/cluster_regressions/listings_tail_leak_6681.tex");
  // No LaTeXML-internal constructor may leak into the serialized body as text.
  for marker in ["@@listings@block", "@@numbered@section", "lx@bibliography"] {
    assert!(
      !xml.contains(marker),
      "internal constructor `{marker}` leaked as text — the listings tail regressed:\n{xml}"
    );
  }
  // Both witness-table listings survive as real listings, not swallowed text.
  assert_eq!(
    xml.matches(r#"class="ltx_lstlisting""#).count(),
    2,
    "both lstlisting blocks must render as listings:\n{xml}"
  );
  // The document tail after the second listing must render: the second section,
  // the appendix, and their body paragraphs.
  for needle in [
    "Open questions",
    "Verification",
    "must render as ordinary text",
    "must survive as a real appendix",
  ] {
    assert!(
      xml.contains(needle),
      "tail content `{needle}` missing — the listing swallowed the rest:\n{xml}"
    );
  }
  assert!(
    xml.contains("<appendix"),
    "the Verification appendix must survive:\n{xml}"
  );
}
/// arXiv/html_feedback#6873 (reporter tdsmith, paper 2601.13118v1): Table 2 — a
/// `tabular` inside a `tcolorbox` `enhanced` skin — rendered vertically upside
/// down. The box is drawn as SVG and the table sits in an `<svg:foreignObject>`
/// inside a TeX-y-up (flipped) `<svg:g>`; the fo needs a counter-flip
/// `transform="matrix(1 0 0 -1 0 h)"` (set by its size-dependent afterClose,
/// `tex_box.rs` / Perl `TeX_Box.pool.ltxml` L407-423) or it renders upside down.
/// `insert_block` renames a `_CaptureBlock_` — which carries the block's box —
/// to `svg:foreignObject`; `rename_node_internal` now carries the node box
/// across (Perl copies it via the `_box` attribute), so the fo's afterClose
/// finds the size and sets the flip. RED before the fix: the tabular's
/// foreignObject had no `transform`, so the height was 0 and it drew flipped.
#[test]
fn cluster_tcolorbox_tabular_not_flipped_6873() {
  let xml = convert_to_xml("tests/cluster_regressions/tcolorbox_tabular_flip_6873.tex");
  // The <svg:foreignObject> that wraps the <tabular> must carry the y-flip
  // transform — otherwise the table draws upside down inside the flipped group.
  let tab = xml
    .find("<tabular")
    .unwrap_or_else(|| panic!("a tabular should render inside the SVG picture:\n{xml}"));
  let fo_start = xml[..tab]
    .rfind("<svg:foreignObject")
    .unwrap_or_else(|| panic!("the tabular must be wrapped in an svg:foreignObject:\n{xml}"));
  let fo_open = &xml[fo_start..tab];
  assert!(
    fo_open.contains(r#"transform="matrix(1 0 0 -1"#),
    "the foreignObject wrapping the tabular must carry the y-flip transform \
     (else the table renders upside down):\n{fo_open}"
  );
}
/// Issue #723 (reporter xworld21): a Rhai binding's `HyperVerbatim` argument
/// under T1 fontencoding produced non-ASCII `~`/`^`, breaking URLs. The T1
/// fontmap deliberately maps slots 94/126 to accent glyphs U+02C6/U+02DC (Bruce
/// Miller, LaTeXML #2435) — we keep that. Instead, `Verbatim`/`HyperVerbatim`
/// now hold an identity ASCII fontmap THROUGH digestion (a `before_digest`
/// `MergeFont(encoding => "ASCII")`, mirroring `Semiverbatim`), so a verbatim/URL
/// argument stays ASCII while normal T1 text still follows Bruce's mapping.
/// Surpasses Perl (which loses ASCII the same way). OXIDIZED_DESIGN #144.
/// Driven by the binary so the runtime Rhai binding loads (the reported path).
#[test]
fn cluster_t1_hyperverbatim_ascii_723() {
  use std::process::Command;
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  let dir = tempfile::tempdir().expect("tempdir");
  std::fs::write(
    dir.path().join("myhyper.sty.rhai"),
    "DefConstructor(\"\\\\myhyper HyperVerbatim\", \"<ltx:ref class=\\\"myhyper\\\" href=\\\"#1\\\">#1</ltx:ref>\");\n",
  )
  .expect("write rhai");
  std::fs::write(
    dir.path().join("m.tex"),
    "\\documentclass{article}\n\\usepackage[T1]{fontenc}\n\\usepackage{myhyper}\n\
     \\begin{document}\n\\myhyper{http://x/a~b^c}\n\\end{document}\n",
  )
  .expect("write m.tex");
  let out = Command::new(bin)
    .args(["m.tex", "--dest", "m.xml", "--nocomments"])
    .current_dir(dir.path())
    .output()
    .expect("spawn latexml_oxide");
  let xml = std::fs::read_to_string(dir.path().join("m.xml")).unwrap_or_default();
  // Canary: the HyperVerbatim `~`/`^` (href AND text) must be ASCII, not U+02DC/U+02C6.
  assert!(
    out.status.success()
      && xml.contains("http://x/a~b^c")
      && !xml.contains('\u{02DC}')
      && !xml.contains('\u{02C6}'),
    "T1 HyperVerbatim `~`/`^` not ASCII (#723):\n{xml}"
  );
}
/// Issue #723, extended scope: the same rule applies to `\verb`, the `verbatim`
/// environment, and `\url`/`\path` (all select the identity ASCII fontmap for
/// their run while keeping the typewriter family). `~`/`^` stay ASCII in the
/// DISPLAYED text, not Bruce's accent glyphs. `\url`/`\path` were missed by the
/// first pass (#727): their display is `\UrlFont`-wrapped (a plain, non-verbatim
/// arg), so the reader's ASCII fontmap didn't reach it — `\UrlFont` now selects
/// `\fontencoding{ASCII}`, matching pdflatex (Perl shows the accents there).
/// OXIDIZED_DESIGN #144.
#[test]
fn cluster_t1_verbatim_ascii_723() {
  let xml = convert_to_xml("tests/cluster_regressions/t1_ascii_tilde_circumflex_723.tex");
  // \verb|a~b^c| and the verbatim env `d~e^f` — both ASCII, both typewriter.
  assert!(
    xml.contains("a~b^c") && xml.contains("d~e^f"),
    "T1 \\verb / verbatim env did not keep `~`/`^` ASCII (#723):\n{xml}"
  );
  // \url{http://g/~h^i} and \path|j~k^l| — the DISPLAYED url text (the href
  // attribute was always ASCII via reversion) must stay ASCII too.
  assert!(
    xml.contains("http://g/~h^i") && xml.contains("j~k^l"),
    "T1 \\url / \\path displayed text did not keep `~`/`^` ASCII (#723):\n{xml}"
  );
  assert!(
    !xml.contains('\u{02DC}') && !xml.contains('\u{02C6}'),
    "T1 verbatim/url still emits accent U+02DC/U+02C6 for a literal `~`/`^` (#723):\n{xml}"
  );
  assert!(
    xml.matches(r#"font="typewriter""#).count() >= 4,
    "verbatim/url runs lost the typewriter family (#723 — encoding must not clobber font):\n{xml}"
  );
}
/// Issue #723 (reporter xworld21 / Vincenzo Mantova), follow-up rebuttal: a
/// non-expandable control sequence inside a hyperref `\url`/`\href` (e.g. `\def`)
/// was DIGESTED — it executed, consumed the following tokens, truncated the URL
/// and raised errors (`\url{…q=\def}` → `href="…q="` + 2 errors), whereas
/// pdflatex/url.sty keep it as literal href text. url.sty stringifies the URL via
/// `\meaning` (all leftover control sequences become inert characters); we now
/// mirror that faithfully: after the semiverbatim read, any surviving control
/// sequence is recatcoded to `other` instead of being handed to digestion. The
/// url.sty escapes (`\_`, `\%`, `\^`, `\textasciitilde`, `\textbackslash`, …)
/// still RESOLVE to their character (they expand during the read), so realistic
/// URLs are unchanged. Surpasses Perl (same digest bug). Distilled reproductions
/// cover Vincenzo's reported cases; the full escape matrix is in `hyperurls`.
#[test]
fn cluster_url_cs_verbatim_723() {
  // `convert_to_xml` gates on ZERO `Error:` markers — RED while `\def` digests.
  let xml = convert_to_xml("tests/cluster_regressions/url_cs_verbatim_723.tex");
  // (a)+(b) Canary: the leftover `\def` survives as literal href text in BOTH
  // `\url` and `\href`, rather than executing and truncating the URL.
  assert!(
    xml.contains(r#"href="http://x/q=\def""#),
    "\\url with a trailing \\def did not keep it as literal href text (#723):\n{xml}"
  );
  assert!(
    xml.contains(r#"href="http://x/h=\def""#),
    "\\href with a trailing \\def did not keep it as literal href text (#723):\n{xml}"
  );
  // (c) Regression guard: resolving control sequences stay resolved and a literal
  // `~` passes through — \textasciitilde -> ~ , \textbackslash -> \ .
  assert!(
    xml.contains(r#"href="http://x/a~b~c\d""#),
    "`\\textasciitilde`/`\\textbackslash`/literal `~` regressed in a URL (#723):\n{xml}"
  );
  // (d) Regression guard: `\href` keeps a literal `~`.
  assert!(
    xml.contains(r#"href="http://x/~u""#),
    "`\\href` dropped a literal `~` (#723):\n{xml}"
  );
  // (e) Regression guard: url.sty escapes still resolve to their character.
  assert!(
    xml.contains(r#"href="http://x/a_b%c""#),
    "url.sty escapes `\\_`/`\\%` no longer resolve to `_`/`%` (#723 regression):\n{xml}"
  );
}
#[test]
fn cluster_fvextra_preserves_ltx_verbatim() {
  let xml = convert_to_xml("tests/cluster_regressions/fvextra_ltx_verbatim.tex");
  assert!(
    xml.contains(r#"class="ltx_verbatim""#),
    "fvextra clobbered the ltx_verbatim class — its `\\def\\FancyVerbFormatLine` \
     overwrote the fancyvrb hook, so verbatim lines lose white-space:pre:\n{xml}"
  );
}
/// brucemiller/LaTeXML#2709: `arrange_panels`' merge heuristic (a >8x size
/// disparity, a tiny joint width, or a zero-width sibling) grouped two panels
/// into an `ltx:block`. For `ltx:figure`/`ltx:table` panels that yields a
/// schema-invalid `<block><figure/></block>` — a block cannot contain a float.
/// Float panels must stay siblings (the merge is only for small inline content).
/// Present in both engines (Perl 0.8.8 has the identical `wrapNodes('ltx:block')`
/// at `latex_constructs.pool.ltxml:3322`); recorded in `KNOWN_PERL_ERRORS.md`.
#[test]
fn cluster_panel_merge_never_wraps_a_figure_in_a_block_2709() {
  let xml = convert_to_xml("tests/cluster_regressions/panel_block_invalid_2709.tex");
  assert!(
    !xml.contains("<block"),
    "arrange_panels wrapped disparate/tiny figure panels in an invalid <block> \
     (#2709) — a block cannot contain a float; the panels must stay siblings:\n{xml}"
  );
  // Both figures keep their two subfigure panels as marked siblings.
  assert_eq!(
    xml.matches("ltx_figure_panel").count(),
    4,
    "expected 4 subfigure panels (2 per figure) as siblings:\n{xml}"
  );
}
/// An unbound class (->OmniBus) whose `.bbl` `\bibitem[\protect\citeauthoryear…]`
/// side-loads natbib must not leave a body `\citep` looping. The side-load runs
/// inside the `thebibliography` group, so natbib's `\citep` would be popped on
/// `\end{thebibliography}` and revert to its (now `sty_loaded`) `def_autoload`
/// trigger, whose already-loaded re-emit then loops to the token limit. Fixed by
/// hoisting the side-loaded package's defs to global (`\lx@late@usepackage`,
/// omnibus_cls.rs). Witness: arXiv 2209.11799 (200s TokenLimit fatal -> 1s/0err).
#[test]
fn cluster_omnibus_natbib_bbl_sideload() {
  convert_clean("tests/cluster_regressions/omnibus_natbib_bbl_sideload.tex");
}
/// A bare `\url` at end-of-input previously panicked: `\url`'s reader did
/// `read_token()?.unwrap()` and the `None` (input exhausted) hit the `.unwrap()`.
/// Real TeX raises a clean "Emergency stop" ("File ended while scanning use of
/// \url"); now `read_token_required` emits that parity Error and the macro
/// degrades (closes its group) instead of crashing. Guards the whole
/// `read_token_required` family (hyperref/url.sty `\url`, `\path`, amscd `\cd@`,
/// `\textfont`). Witnesses: 1401.5000, 1502.05051, 2204.10457. The specimen
/// truncates `\url` at EOF, so the ONE intentional `expected:` Error (input
/// ended while scanning use of `\url`) is the correct outcome — Perl emits the
/// same. `convert_expecting_errors(…, 1)` asserts EXACTLY that error (not merely
/// "non-empty output"): a drift to 0 means we stopped detecting the truncation,
/// >1 or a Fatal means the graceful recovery regressed.
#[test]
fn cluster_url_at_eof_no_panic() {
  let xml = convert_expecting_errors("tests/cluster_regressions/url_eof_no_panic.tex", 1);
  assert!(
    !xml.is_empty(),
    "url-at-EOF conversion produced empty output"
  );
}
/// Twemoji-style csname construction with accent macros (`\'`, `\^`, `\~`)
/// and `\textquoteright` apostrophe — must produce 0 errors after the
/// csname-stream soft-substitute fixes for `\lx@applyaccent`, the canonical
/// `\text…` primitives, and the NFSS `\<encoding>\i`/`\j` glyphs.
/// Pinned by stage-1..3 of the 100k warning corpus (arXiv:2603.22193,
/// 2603.23433, 2604.20621 — twemoji St. Barthélemy / Côte d'Ivoire / São Tomé).
#[test]
fn cluster_csname_accent() { convert_clean("tests/cluster_regressions/csname_accent.tex"); }
/// Legacy `\documentstyle[…]{amsart}` (LaTeX 2.09 compat) must auto-load
/// the AmS-TeX `\Sb` / `\Sp` substack environments via
/// `RequirePackage('amstex') if LookupValue('2.09_COMPATIBILITY')`.
/// Witnesses: arXiv:alg-geom9208004, arXiv:alg-geom9202004.
#[test]
fn cluster_amstex_2_09_sb() { convert_clean("tests/cluster_regressions/amstex_2_09_sb.tex"); }
/// AmSTeX `\input amstex` + `\documentstyle{amsppt}` papers must
/// stub `\vspace` / `\hspace` / `\scriptsize` / other LaTeX2e
/// typesetting CSes as no-ops (the AmSTeX pool path doesn't load
/// latex_constructs.rs). Witnesses: arXiv:funct-an9211012,
/// funct-an9211013, funct-an9211011, funct-an9312004.
#[test]
fn cluster_amsppt_vspace() { convert_clean("tests/cluster_regressions/amsppt_vspace.tex"); }
/// Picture-environment `\multiput(x,{y})` with the second coordinate
/// braced. Pair parameter reader must look through BEGIN…END groups
/// before reading the float. Witnesses: arXiv:hep-th9610147,
/// hep-th9703142.
#[test]
fn cluster_multiput_braced_pair() {
  convert_clean("tests/cluster_regressions/multiput_braced_pair.tex");
}
/// `\thechapter` autoload from `omnibus_cls.rs` must autoload the
/// `book.cls` BINDING, not `book.sty`. The obsolete `book.sty` shim
/// in TeXLive fires `\LoadClass{book}` immediately — by the time
/// `\thechapter` triggers (inside the document body), we're past
/// the preamble and `\LoadClass`'s preamble guard errors. Perl
/// avoids this by using `DefAutoload('thechapter', 'book.cls.ltxml')`
/// (cls extension, not sty). Witness: arXiv:2602.10407.
#[test]
fn cluster_omnibus_chapter_book_autoload() {
  convert_clean("tests/cluster_regressions/omnibus_chapter_book_autoload.tex");
}
/// Tolerant `Pair` parameter reader: malformed `(3.2,3,8)` (three
/// comma-separated values where Pair expects two) must consume the
/// trailing `,8` silently so the next Pair argument can read its `(`.
/// Mirrors Perl `ReadPair`'s `readUntil(',')`/`readUntil(')')`.
/// Witness: arXiv:physics/9709007.
#[test]
fn cluster_pair_tolerant_trailing() {
  convert_clean("tests/cluster_regressions/pair_tolerant_trailing.tex");
}
/// `\newpsobject{name}{old}{keyval}` must dynamically define
/// `\<name>` as a forwarder to `\<old>[<keyval>]`. Earlier stub
/// no-op'd, leaving the defined CS undefined. Mirrors Perl
/// `pstricks_support.sty.ltxml` L849-861. Witness:
/// arXiv:physics/9710028 (10 errors → 0 with this fix).
#[test]
fn cluster_newpsobject_forward() {
  convert_clean("tests/cluster_regressions/newpsobject_forward.tex");
}
/// JHEP.cls override of `\href` must use `Semiverbatim Semiverbatim`
/// (NOT hyperref's `HyperVerbatim {}`) so the BODY arg's `^`/`_`
/// are neutralized to OTHER catcode and don't fire `script_handler`
/// when digested in math mode. Affects all `\@spires`-style journal
/// citation macros (`\am`, `\ap`, `\np`, `\pl`, …). Mirrors Perl
/// `JHEP.cls.ltxml` L133-136. Witness: arXiv:2602.22473.
#[test]
fn cluster_jhep_href_semiverbatim() {
  convert_clean("tests/cluster_regressions/jhep_href_semiverbatim.tex");
}
/// An eqnarray reading a `\def`-ized `\arraycolsep` (a plain macro, not a length
/// register) must NOT emit the Rust-only `expected:register` warning — Perl's
/// `LookupDimension` reads the macro body silently (verified same-host: Perl
/// 0.8.8 is silent; Rust used to warn 1×). Fixed by `state::lookup_dimension_cs`.
/// See docs/SYNC_STATUS.md.
#[test]
fn cluster_eqnarray_arraycolsep_macro_no_register_warning() {
  let log = convert_log("tests/cluster_regressions/eqnarray_arraycolsep_macro.tex");
  assert!(
    !log.contains("is not a register"),
    "spurious expected:register warning on a \\def-ized \\arraycolsep (LookupDimension regressed):\n{log}"
  );
}
/// Same as above for the `cases` package `numcases` environment (Perl
/// cases.sty.ltxml L82 also reads `\arraycolsep` via `LookupDimension`). A
/// `\def`-ized `\arraycolsep` must not produce the Rust-only `expected:register`
/// warning. See docs/SYNC_STATUS.md.
#[test]
fn cluster_numcases_arraycolsep_macro_no_register_warning() {
  let log = convert_log("tests/cluster_regressions/numcases_arraycolsep_macro.tex");
  assert!(
    !log.contains("is not a register"),
    "spurious expected:register warning on a \\def-ized \\arraycolsep in numcases:\n{log}"
  );
}
/// A `\label` placed right after `\begin{eqnarray}` whose first row is
/// `\nonumber` must make `\ref` render the equation number ("1"), not the
/// document title. LaTeX steps the `equation` counter once at `\begin`, so
/// `\label` captures "1" before the `\nonumber` row retracts its display;
/// LaTeXML binds the label to that unnumbered row (no refnum) and `\ref` fell
/// through to the document title (SHARED bug with Perl 0.8.8 — verified same
/// host; surpass-Perl per html_feedback#94). Fixed in Scan: a labelled equation
/// row with no refnum inherits its group's number from a numbered sibling.
/// Witness arXiv 2308.06222. pdflatex ground truth: `\newlabel{eqx}{{1}{1}…}`.
#[test]
fn cluster_eqnarray_nonumber_label_ref_is_the_number() {
  let x = convert_and_post_clean("tests/cluster_regressions/eqnarray_nonumber_label_ref.tex");
  // The in-text \ref renders the number "1" as an ltx_ref_tag (not a title).
  assert!(
    x.contains(r#"<text class="ltx_ref_tag">1</text>"#),
    "eqnarray \\ref did not resolve to the equation number \"1\":\n{x}"
  );
  // The distinctive title word must NOT leak into a reference as its text (it may
  // still appear once in the real <title> element and as the standard breadcrumb
  // tooltip, exactly as a normal numbered-equation ref does).
  assert!(
    !x.contains(r#"ltx_ref_title">Distinctive"#),
    "the document title leaked into the equation \\ref link text:\n{x}"
  );
}
/// floatflt `floatingfigure` must compute the `width` percentage from its
/// `{Dimension}` arg (Perl `toPercent`: `int(100*dim/\textwidth)`). The args are
/// only on the BEGIN whatsit (after_digest_begin); the prior code read them in
/// `after_digest` (args=None) → `width="0%"`. Default \textwidth=345pt + a 3cm
/// figure → `width="24%"` (matches Perl 0.8.8). See docs/SYNC_STATUS.md.
#[test]
fn cluster_floatflt_pctwidth() {
  let xml = convert_to_xml("tests/cluster_regressions/floatflt_pctwidth.tex");
  assert!(
    xml.contains(r#"width="24%""#),
    "floatflt floatingfigure width != 24% (pctwidth/args regressed)"
  );
  assert!(
    !xml.contains(r#"width="0%""#),
    "floatflt floatingfigure width=\"0%\" — Dimension arg not read (after_digest args=None)"
  );
}
/// jcappub (JCAP's SISSA/IOP class) is the JCAP sibling of jheppub with the same
/// accumulating `\author[affil]{name}` + `\affiliation` + `\emailAdd`. Unbound, its
/// `\author`s fell through to article's (which overwrites), so only the LAST author
/// survived and `\affiliation`/`\emailAdd`/`\keywords` were undefined. Routing
/// jcappub to the jheppub binding accumulates every author. SHARED gap with Perl
/// (also truncates + `missing file[jcappub.sty]`); surpass-Perl. html_feedback
/// #6884, witness arXiv 2404.03569 (63 authors rendered, previously 1).
#[test]
fn cluster_jcappub_accumulates_authors() {
  let xml = convert_to_xml("tests/cluster_regressions/jcappub_authors.tex");
  assert_eq!(
    xml.matches(r#"role="author""#).count(),
    3,
    "jcappub did not accumulate all 3 authors (routed to the jheppub binding):\n{xml}"
  );
  for n in ["Alpha Author", "Beta Author", "Gamma Author"] {
    assert!(xml.contains(n), "jcappub author `{n}` missing:\n{xml}");
  }
  assert!(
    xml.contains("Institute Two"),
    "\\affiliation was undefined / not rendered:\n{xml}"
  );
  assert!(
    xml.contains("alpha@example.org"),
    "\\emailAdd was undefined / not rendered:\n{xml}"
  );
}
/// Same fix for the `floatfig` package: a 4cm figure → `width="32%"`.
#[test]
fn cluster_floatfig_pctwidth() {
  let xml = convert_to_xml("tests/cluster_regressions/floatfig_pctwidth.tex");
  assert!(
    xml.contains(r#"width="32%""#),
    "floatfig floatingfigure width != 32% (pctwidth/args regressed)"
  );
}
/// The arXiv IMS journal class (`arximspdf`/`arxstspdf`, used by Annals of
/// Probability/Statistics — aop/aos) must convert with 0 errors AND preserve
/// frontmatter metadata via the standard `\lx@add@*` API. Neither Perl LaTeXML nor
/// Rust bound this self-contained ~3000-line class, so papers cascaded into dozens
/// of undefined errors (`\b*` structured bib, `{barticle}`, `\operatorname`/`\tfrac`,
/// plain-TeX `\matrix`); the binding loads `article` + defines the IMS macros.
/// Surpasses Perl (which fails outright — both engines lack the class). Witness
/// cluster: 0910.0069 + 15 aop/aos papers. See docs/SYNC_STATUS.md.
#[test]
fn cluster_arximspdf_imsart() {
  convert_clean("tests/cluster_regressions/arximspdf_imsart.tex");
  let xml = convert_to_xml("tests/cluster_regressions/arximspdf_imsart.tex");
  // Frontmatter metadata preserved (standard frontmatter API).
  assert!(xml.contains("A Sample IMS Paper"), "title metadata missing");
  assert!(
    xml.contains("Doe"),
    "author (creator/personname) metadata missing"
  );
  assert!(xml.contains("probability"), "keywords metadata missing");
  // Structured \b* bibliography passes through as readable text.
  assert!(
    xml.contains("Smith") && xml.contains("On examples"),
    "structured \\b* bibliography content missing"
  );
}
/// A plain DefMath symbol (`\rightarrowfill`, a DefMath ARROW) used in TEXT mode
/// must NOT emit the Rust-only `unexpected:mode` "should only appear in math mode"
/// warning. Perl (Package.pm:1304) adds the requireMath beforeDigest only for
/// `requireMath => 1` bindings; Rust's `transfer_common_constructor_options` added
/// it unconditionally for every DefMath (broad over-emission; 0802.3360 Rust 3 /
/// Perl 0). See docs/SYNC_STATUS.md.
#[test]
fn cluster_defmath_textmode_no_mode_warning() {
  let log = convert_log("tests/cluster_regressions/defmath_textmode_no_mode_warning.tex");
  assert!(
    !log.contains("should only appear in math mode"),
    "spurious unexpected:mode warning for a DefMath symbol in text mode (requireMath over-applied):\n{log}"
  );
}
/// A `feynmp` (Feynman-diagram, MetaPost) document must convert with 0 errors —
/// feynmp shares feynmf's macros but had no Rust binding, so `\fmf{...label=$$}`
/// cascaded into `expected:$` display-math errors and `{fmfgraph*}`/`\fmfleft`/…
/// were undefined (witness 1003.1620: Rust 28 / Perl 0). The feynmp binding +
/// shared diagram-macro stubs absorb them. See docs/SYNC_STATUS.md.
#[test]
fn cluster_feynmp_fmf() { convert_clean("tests/cluster_regressions/feynmp_fmf.tex"); }
/// Issue #531 (secondary): faithful `\everyjob` emulation (beyond Perl). l3sys
/// defers `\sys_if_shell:*` / `\c_sys_shell_escape_int` / date-time ints into
/// `\g__sys_everyjob_tl`, which `\__kernel_sys_everyjob:` runs at job start via
/// `\everyjob`. Perl LaTeXML never fires `\everyjob`, so those constants were
/// undefined until a package loaded expl3 — and NEVER on the dump/short-circuit
/// path a texmf expl3.sty newer than the embedded dump takes (it skips `\input
/// expl3-code.tex`), where the newer expl3.sty USES `\sys_if_shell:TF` in its
/// support-file check → `Error:undefined:\sys_if_shell:TF` (reporter's TL2026
/// case; reproduced in the `texlive-docker:2026` container with l3kernel
/// 2026-07-20 over a 2026-01-19 dump). `latex.rs` now fires
/// `\__kernel_sys_everyjob:` at `LoadFormat('latex')` completion, so the family
/// is defined with LIVE values before the preamble. The fixture PROBES
/// `\sys_if_shell:TF` in the preamble (before any `\usepackage`) — RED
/// (`EVERYJOB-MISSING`) without the fix, GREEN (`EVERYJOB-PRESENT`) with it —
/// then uses it (LaTeXML has no shell → FALSE branch). OXIDIZED_DESIGN #113.
#[test]
fn cluster_everyjob_defines_l3sys_shell() {
  let xml = convert_to_xml("tests/cluster_regressions/everyjob_sys_shell.tex");
  assert!(
    xml.contains("EVERYJOB-PRESENT") && !xml.contains("EVERYJOB-MISSING"),
    "\\everyjob must fire at LoadFormat so \\sys_if_shell:TF is defined in the \
     preamble:\n{xml}"
  );
  assert!(
    xml.contains("SHELL-NO") && !xml.contains("SHELL-YES"),
    "\\sys_if_shell:TF must take the FALSE branch (LaTeXML has no shell):\n{xml}"
  );
}
/// Issue #531: `pdfcol.sty` (a PDF colour-stack manager, pulled in transitively
/// by tcolorbox's `breakable` library) had no binding, so `\pdfcolInitStack` and
/// its four siblings raised `undefined` errors and leaked their args as text.
/// SHARED-FAILURE (same-host Perl errors identically — neither engine shipped a
/// pdfcol binding). LaTeXML emits no PDF colour stack, so `pdfcol_sty.rs` ports
/// the package's own "disabled" fallback (all commands no-op; `\pdfcolIfStackExists`
/// takes the false branch). OXIDIZED_DESIGN #112.
#[test]
fn cluster_pdfcol_stub_no_undefined() {
  let xml = convert_to_xml("tests/cluster_regressions/pdfcol_stub.tex");
  // No `undefined` ERROR nodes for any pdfcol command (`convert_to_xml` already
  // gates 0 errors; this pins the shape). The four no-op commands emit nothing
  // and `\pdfcolIfStackExists{main}{STACK-YES}{STACK-NO}` runs the FALSE branch
  // (the disabled fallback: a stack is never registered), so the whole body
  // collapses to exactly `STACK-NO` — no leaked `main`/`STACK-YES` args.
  assert!(
    !xml.contains("<ERROR"),
    "pdfcol commands must no-op, not <ERROR>:\n{xml}"
  );
  assert!(
    xml.contains("<p>STACK-NO</p>") && !xml.contains("STACK-YES"),
    "\\pdfcolIfStackExists must take the false branch and siblings must no-op:\n{xml}"
  );
}
/// arXiv/html_feedback#970 (paper 2312.06275): a siunitx unit declared with an
/// empty symbol — `\DeclareSIUnit{\nothing}{\relax}` — must render INVISIBLY, not
/// as the literal word "nothing". The core emits an empty `<XMTok
/// class="ltx_unit" meaning="nothing"/>`; the MathML empty-content fallback
/// (`presentation.rs`) used the `meaning` attribute, producing `<mi
/// class="ltx_unit">nothing</mi>` (Perl does the same, in red — SHARED-FAILURE).
/// An empty *unit* now renders as an invisible placeholder. OXIDIZED_DESIGN #114.
#[test]
fn cluster_siunitx_empty_unit_renders_invisible() {
  let post = convert_and_post_pmml_clean("tests/cluster_regressions/siunitx_nothing_unit.tex");
  // The presentation MathML must NOT render the unit's name as visible text.
  assert!(
    !post.contains(">nothing<"),
    "empty siunitx unit leaked its `meaning` as visible MathML text:\n{post}"
  );
  // It renders as an invisible placeholder instead, and the quantity 5 stays.
  assert!(
    post.contains("mphantom") && post.contains(">5<"),
    "empty unit must be an <m:mphantom> and the quantity 5 must still render:\n{post}"
  );
}
/// An UNBOUND journal class (`sn-jnl`, `wlpeerj`, `sagej`, Wiley, …) falls back
/// to the OmniBus class, whose lazy natbib autoload triggers (`\citep`/`\citet`/
/// `\citeyear`/…) must load natbib EXACTLY ONCE and resolve to natbib's real
/// definition. The hand-rolled OmniBus autoload (require_package → re-emit, no
/// clear) re-fired its own stub on every re-emit — fully RE-loading natbib each
/// iteration until the wall-clock watchdog (~60s+ digest hang). This was the
/// dominant slow/timeout cluster in the arXiv perf testbed (~50 sn-jnl + Wiley/
/// sagej/wlpeerj papers; witness 2603.06884: 90s digest → fatal timeout). Routing
/// through the canonical loop-safe `def_autoload` (clear trigger globally BEFORE
/// the load, hoist natbib's fresh defs to global, then re-emit) fixes the hang
/// while keeping `\citep` defined — the 1403.6801 (wlpeerj) regression that the
/// clear-AFTER-load attempt broke. See docs/performance/ARXIV_PERFORMANCE.md.
#[test]
fn cluster_omnibus_natbib_autoload_no_reload_loop() {
  let src = "tests/cluster_regressions/omnibus_natbib_autoload.tex";
  // Completes (no hang/timeout) and renders the citations — natbib's real
  // \citep/\citet resolved, producing the `ltx_cite` citation groups.
  let html = convert_to_xml(src);
  assert!(
    html.contains("ltx_cite"),
    "OmniBus natbib autoload: citations did not resolve to natbib's \\citep/\\citet \
     (expected an ltx_cite group in the output):\n{html}"
  );
  // The cite trigger must NOT have reverted to undefined after natbib loaded
  // (the clear-after-load failure mode, 1403.6801).
  let log = convert_log(src);
  let undef_cite = log
    .lines()
    .any(|l| l.contains("undefined") && l.contains("cite"));
  assert!(
    !undef_cite,
    "OmniBus natbib autoload: a cite trigger reverted to undefined after the load:\n{log}"
  );
}
/// The mhchem stub must NOT clobber an author's own `\cf` ("cf.") macro.
/// `\cf`/`\cee` are mhchem LEGACY (`version < 4`) commands; real mhchem
/// resolves the default version to 4 and leaves them undefined, so Perl
/// (raw-load) lets `\newcommand{\cf}` succeed as text. Defining them
/// unconditionally made `\newcommand` error "already defined" and left
/// `\cf` an `\ensuremath` math macro, so "cf." text leaked into math mode
/// ("Script _ can only appear in math mode" → `<ltx:XMTok>` cascade).
/// Mirrors mhchem.sty L3430 `\int_compare:nT { version < 4 }`. Witness:
/// arXiv:1901.08894 (chemformula + revtex4-1): 1002 errors / Fatal → 0.
#[test]
fn cluster_mhchem_cf_author_macro() {
  convert_clean("tests/cluster_regressions/mhchem_cf_author_macro.tex");
}
/// The flagship raw-load guard: \ce{H2O}/\ce{SO4^2-} must convert cleanly
/// through the real mhchem.sty + expl3 pipeline (PR_READINESS review — the
/// chemistry corpus had no fixture at all).
#[test]
fn cluster_mhchem_ce_subscripts() {
  convert_clean("tests/cluster_regressions/mhchem_ce_subscripts.tex");
}
/// Multi-level `theindex` (`\item`/`\subitem`/`\subsubitem`) must build nested
/// `<ltx:indexlist>`/`<ltx:indexentry>` cleanly. Requires (1) `Tag('ltx:indexentry',
/// autoClose=>1)` — Perl `latex_constructs.pool.ltxml` L4477 — so a new entry
/// auto-closes its open sibling and indexlist unwinds its entry children; and (2)
/// the theindex `beforeDigestEnd` must RETURN the digested `\index@done` whatsit so
/// it is constructed and unwinds the trailing indexphrase/indexlist. Without these
/// the builder errors "indexentry isn't allowed in indexentry" / "Closing ltx:index
/// whose descendents do not auto-close". Witness: arXiv:1205.0533 (102 errors /
/// Fatal → 1, the residual `\hyperpage` shared with Perl).
#[test]
fn cluster_theindex_nested_autoclose() {
  convert_clean("tests/cluster_regressions/theindex_nested_autoclose.tex");
}
/// `\verb` inside `\index{…}` must render its body as typewriter verbatim, not
/// vanish. `\index` reads its argument `SanitizedVerbatim`, which re-tokenizes it —
/// collapsing `\verb`'s raw body back into control sequences and leaving `\verb`
/// with no mouth to scan a delimiter from. In BOTH engines this produced an empty
/// `<verbatim/>` with the body leaking out mis-tokenized (`\delta` → math-italic δ),
/// and a `|` delimiter additionally collided with makeindex's encap separator
/// (`Error:expected:delimiter Verbatim argument lost`, phrase lost into a bogus
/// `style=`). Rust surpasses (OXIDIZED_DESIGN #119): `process_index_phrases`
/// consumes a `\verb<D>body<D>` run atomically — before the `!`/`@`/`|` split can
/// see the delimiter — and emits `\@internal@text@verb`, so the body renders as
/// typewriter. Shared with Perl LaTeXML 0.8.8; issue #354.
#[test]
fn cluster_verb_in_index_renders_typewriter() {
  // convert_to_xml gates on 0 errors — the `|` form used to emit
  // `Error:expected:delimiter Verbatim argument lost`.
  let xml = convert_to_xml("tests/cluster_regressions/verb_in_index.tex");
  // Four `\verb` bodies (`\verb+..+`, `\verb|..|`, `\verb*|..|`, and the
  // `\verb|sub|` subentry), each a typewriter verbatim; `\index{plain}` and the
  // `grp` head are plain phrases.
  assert_eq!(
    xml.matches(r#"<verbatim font="typewriter""#).count(),
    4,
    "each \\verb in \\index must render one typewriter <verbatim>; xml=\n{xml}"
  );
  // The body survives as literal typewriter text …
  assert!(
    xml.contains(r"\delta"),
    "the \\verb body did not survive as literal text; xml=\n{xml}"
  );
  // … and is NOT leaked out and re-digested as a math-italic δ (U+03B4).
  assert!(
    !xml.contains('\u{03B4}'),
    "the \\verb body leaked out and digested as math δ; xml=\n{xml}"
  );
  // The `|` delimiter must not be mistaken for the encap separator.
  assert!(
    !xml.contains("style=\"\u{201c}"),
    "a \\verb `|` delimiter leaked into a bogus indexmark style=; xml=\n{xml}"
  );
  // `grp!\verb|sub|` composes with the `!` subentry split: a plain `grp` head and
  // a verbatim `sub` subentry — the `\verb` `|` delimiters consumed, not split on.
  assert!(
    xml.contains(
      r#"<indexphrase key="sub"><verbatim font="typewriter">sub</verbatim></indexphrase>"#
    ),
    "the \\verb subentry did not compose with the `!` split; xml=\n{xml}"
  );
}
/// aa.cls (Astronomy & Astrophysics) does `\RequirePackage[T1]{fontenc}`
/// (real aa.cls L154), so a literal text-mode `>`/`<` renders as itself in the
/// PDF. Both LaTeXML engines' aa binding loaded `fontenc` WITHOUT the `[T1]`
/// option, so the document stayed OT1 and `>` decoded as ¿ (U+00BF), `<` as ¡
/// (U+00A1) — the arXiv HTML diverged from the arXiv PDF (html_feedback#84,
/// arXiv:2308.06236v1 Fig 6 caption `masses > 0.1~M_\oplus`). `aa_support_sty.rs`
/// now loads `fontenc` with `[T1]` like the real class (and like the
/// acmart/elsarticle/moderncv bindings). Shared with Perl 0.8.8 (its aa_support
/// dropped the same option).
#[test]
fn cluster_aa_class_t1_fontenc_angle_brackets() {
  let xml = convert_to_xml("tests/cluster_regressions/aa_class_t1_fontenc.tex");
  // The canary: no OT1 ¡/¿ decode of `<`/`>` under aa.cls's T1 encoding.
  assert!(
    !xml.contains('\u{00BF}') && !xml.contains('\u{00A1}'),
    "aa.cls forces [T1]{{fontenc}}, so `>`/`<` must not decode as OT1 ¿/¡; xml=\n{xml}"
  );
  // …and the greater/less signs survive as themselves.
  assert!(
    xml.contains("masses &gt; 0.1") && xml.contains("less &lt; 0.5"),
    "literal `>`/`<` must render as themselves under aa.cls's T1 encoding; xml=\n{xml}"
  );
}
/// Regression guard for a bug that self-resolved via cumulative digestion fixes.
/// arXiv:1806.08417's `\DeclarePairedDelimiterXPP\seq` used inside `\genfrac` in
/// display math once errored (`\delimsize` + `\seq@after` undefined, ~3 errors)
/// because the constructor's arg-digestion sub-frame dropped the XPP closure's
/// local `\def`s. `convert_to_xml` gates 0 errors; the asserts prove the paired
/// delimiter + FRACOP actually rendered (guards against an error-free-but-empty
/// pass). Was tracked in memory `project_1806_08417_seq_in_body` (now retired).
#[test]
fn cluster_seq_paired_delim_in_genfrac_display_math() {
  let xml = convert_to_xml("tests/cluster_regressions/seq_paired_delim_genfrac.tex");
  assert!(
    xml.contains("role=\"FRACOP\""),
    "genfrac must render a FRACOP; xml=\n{xml}"
  );
  assert!(
    xml.matches("role=\"OPEN\"").count() >= 2 && xml.matches("role=\"CLOSE\"").count() >= 2,
    "both \\seq paired delimiters must render their big-paren OPEN/CLOSE; xml=\n{xml}"
  );
  assert!(
    !xml.contains("delimsize") && !xml.contains("seq@after"),
    "no leftover undefined \\delimsize/\\seq@after; xml=\n{xml}"
  );
}
/// Regression guard (self-resolved). `$$\begin{tabular}…$$` — the old astro/aa
/// display-math centering idiom — under `aa.cls` once produced a
/// `malformed:ltx:text` close-without-open cascade (3 errors) from the XMText⇄
/// tabular interplay. `convert_to_xml` gates 0 errors; the asserts prove the
/// tabular nested cleanly in the equation. Was `project_dollar_dollar_tabular`.
#[test]
fn cluster_dollar_dollar_tabular_in_aa_class() {
  let xml = convert_to_xml("tests/cluster_regressions/dollar_dollar_tabular.tex");
  assert!(
    xml.contains("<tabular class=\"ltx_markedasmath\""),
    "the $$…$$ tabular must render as a math-marked tabular; xml=\n{xml}"
  );
  assert!(
    xml.contains(">A</td>") && xml.contains(">B</td>"),
    "both tabular cells must render; xml=\n{xml}"
  );
  assert!(
    !xml.contains("malformed"),
    "no malformed close-cascade; xml=\n{xml}"
  );
}
/// Regression guard (self-resolved). `\[ … \]` (single-CS display math) with a
/// body constructor, followed by `\begin{equation}`, once leaked
/// `malformed:ltx:XMApp "isn't allowed in <ltx:text>"` during digestion (a
/// Rust-only cluster ~50 papers; the second equation came out `ltx_math_unparsed`).
/// `convert_to_xml` gates 0 errors; the asserts prove BOTH equations math-parse.
/// Was `project_math_leak_em_in_equation`.
#[test]
fn cluster_display_math_then_equation_no_xmapp_leak() {
  let xml = convert_to_xml("tests/cluster_regressions/display_math_then_equation.tex");
  assert!(
    xml.matches("<equation").count() >= 2,
    "both equations must render; xml=\n{xml}"
  );
  assert!(
    xml.contains("role=\"ADDOP\""),
    "the A+B equation must math-parse to an ADDOP application; xml=\n{xml}"
  );
  assert!(
    !xml.contains("ltx_math_unparsed") && !xml.contains("isn't allowed"),
    "no math-parse failure / XMApp-into-text leak; xml=\n{xml}"
  );
}
/// subcaption loaded AFTER subfigure.sty must not clobber subfigure.sty's
/// self-contained `\subfigure[][]{}` macro with its own `{subfigure}[]{Dimension}`
/// environment. The two have incompatible contracts: the macro consumes a
/// balanced body and closes itself; the environment reads a `{Dimension}` and
/// opens a group closed only by `\end{subfigure}`. A document using the macro
/// form (`\subfigure[]{\includegraphics{...}}`) would then reparse it as an
/// environment — the `{\includegraphics{...}}` misread as a Dimension and the
/// group left open — swallowing the rest of the document (figures, sections,
/// bibliography). Real LaTeX's `\newenvironment` refuses to redefine an existing
/// `\subfigure`; we mirror that guard. Witness 2507.21938 (Perl times out on it).
#[test]
fn subcaption_does_not_clobber_subfigure_macro() {
  let x = convert_to_xml("tests/cluster_regressions/subcaption_subfigure_conflict.tex");
  // Content after the figure survived => no leaked, unclosed group.
  assert!(
    x.contains("must survive"),
    "subcaption clobbered subfigure.sty's \\subfigure; content after the figure was lost:\n{x}"
  );
  // The bibliography (document tail) is present => no truncation.
  assert!(
    x.contains("<bibitem") && x.contains("representative title"),
    "bibliography lost — the subfigure/subcaption clash leaked a group and truncated the document:\n{x}"
  );
}
/// Brace-less `\hphantom` immediately followed by `\endminipage` (the low-level
/// minipage primitive, no braces): upstream #2783's `\hphantom{}` grabs `#1`
/// unconditionally, so it would swallow `\endminipage` into the phantom's
/// `restricted_horizontal` frame — the minipage never closes and every element
/// after it (the "After" section and the bibliography) is absorbed and LOST.
/// The brace-guard (`\@ifnextchar\bgroup`) emits an empty phantom that consumes
/// nothing, so `\endminipage` closes its minipage in the ambient mode.
/// Witness 2004.10048 (`\minipage…\hphantom\endminipage`).
#[test]
fn hphantom_braceless_minipage_does_not_swallow_endminipage() {
  let x = convert_to_xml("tests/cluster_regressions/hphantom_braceless_minipage.tex");
  // Content after the figure survived => the minipage closed.
  assert!(
    x.contains("must survive"),
    "brace-less \\hphantom swallowed \\endminipage; content after the minipage was lost:\n{x}"
  );
  // The bibliography (last thing in the document) is present => no truncation.
  assert!(
    x.contains("<bibitem") && x.contains("representative title"),
    "bibliography lost — the minipage leaked and truncated the document:\n{x}"
  );
}
/// Witness 2605.11619: `\end{lstlisting}` preceded by content on the same line
/// (`</body></html> \end{lstlisting}`). Perl anchors the terminator regex at the
/// line start (listings.sty.ltxml L316), so the reader ran to EOF and swallowed
/// the rest of the document — Conclusion, `\bibliography` and appendix — with NO
/// error at all. Real `listings` terminates there (pdflatex renders the leading
/// text as the final listing line and continues), so both LaTeXML engines were
/// wrong vs the PDF. OXIDIZED_DESIGN #61 / KNOWN_PERL_ERRORS #51.
#[test]
fn inline_end_lstlisting_does_not_swallow_the_document() {
  let x = convert_to_xml("tests/cluster_regressions/lstlisting_inline_end.tex");
  assert!(
    x.contains("AFTER-THE-LISTING-MARKER"),
    "inline \\end{{lstlisting}}: the rest of the document was swallowed:\n{x}"
  );
  // The text before the terminator is still the listing's last line (pdflatex
  // renders exactly "hello world" there).
  assert!(
    x.contains("hello") && x.contains("world"),
    "inline \\end{{lstlisting}}: the listing body was lost:\n{x}"
  );
}

/// minted's `escapeinside=!!` must let a `\label` inside the code attach to its
/// listing line, so `\ref{line:...}` resolves and links.
///
/// Witness arXiv:2308.03276 (html_feedback#1028). Our `minted` binding
/// (`minted_contrib`) parsed `\begin{minted}[opts]{lang}` but *dropped* the
/// options — calling `lst_process_display` without activating them — so
/// `escapeinside`/`mathescape` never reached the listings tokenizer. The
/// `!$\label{line:world}$!` was emitted as literal code, `\label` never ran, the
/// line label was never registered, and `\ref{line:world}` rendered as an empty
/// `ltx_missing_label`. The fix forwards the options through `lst_activate` (as
/// `lstlisting` does). Perl has no minted binding (it processes the body as raw
/// LaTeX), so this completes our richer binding — surpass-Perl / OXIDIZED_DESIGN
/// #127.
#[test]
fn minted_escapeinside_label_registers_on_the_code_line() {
  let x = convert_to_xml_contrib("tests/cluster_regressions/minted_escapeinside_label.tex");
  // The escaped `\label` now runs and registers its line label (before the fix
  // there was no `labels="LABEL:line:world"` anywhere — the label was lost).
  assert!(
    x.contains(r#"labels="LABEL:line:world""#) && x.contains(r#"labels="LABEL:line:road""#),
    "escapeinside \\label did not register on the code line:\n{x}"
  );
  // ...and the escape markers / raw `\label` are consumed, not left as literal
  // code characters in the listing.
  assert!(
    !x.contains(r"\label{line:world}"),
    "the escapeinside `\\label` leaked into the listing as literal text:\n{x}"
  );
}
/// `\usepackage{xparse}` (or `expl3`) must not clobber LaTeX's cedilla accent.
///
/// `expl3_sty.rs` used to `\edef` the `\c_sys_*` system constants through
/// `raw_tex`, which tokenizes with the AMBIENT catcodes. After the expl3 load
/// the document regime has `_` = SUB, so `\edef\c_sys_shell_escape_int{0}`
/// parsed as `\edef\c` with parameter text `_sys_shell_escape_int` and body `0`:
/// it rebound LaTeX's cedilla accent `\c` (`\meaning\c` =
/// `macro:_sys_shell_escape_int->0`) and defined none of the constants. Every
/// later `Fran\c cois` then rendered "Fran0cois" — silently, with 0 errors —
/// where Perl LaTeXML renders "François" (GENUINE-RUST-ONLY; issue 421, witness
/// arXiv 2605.11579's `MRREVIEWER = {Fran\c cois\ Digne}`). The block was dead
/// code — expl3 defines those constants itself, with live values — so it was
/// deleted rather than re-tokenized.
///
/// Two properties, because the accent is only the symptom: the accents
/// round-trip, AND the document catcode regime that made the corruption
/// possible is intact after the expl3/xparse load (`_` = 8, `:` = 12, `~` = 13
/// — a leaked `\ExplSyntaxOn` regime would show up here first).
/// (Post-processing is not involved: the corruption happens in the gullet, so
/// the engine XML is the right layer.)
#[test]
fn expl3_load_does_not_clobber_cedilla_accent() {
  let x = convert_to_xml("tests/cluster_regressions/expl3_accent_catcode.tex");
  assert!(
    x.contains("François") && x.contains('Ç') && x.contains('Ş') && x.contains('ţ'),
    "\\usepackage{{xparse}} clobbered the \\c cedilla accent:\n{x}"
  );
  assert!(
    !x.contains("Fran0cois"),
    "\\c expanded to a `\\c_sys_…` macro body — expl3-syntax raw TeX was \
     tokenized outside the expl3 catcode regime:\n{x}"
  );
  assert!(
    x.contains("catcodes: [8][12][13]"),
    "the document catcode regime did not survive \\usepackage{{xparse}} \
     (expected `_`=8, `:`=12, `~`=13):\n{x}"
  );
}

/// An author's `\fnum@<type>` redefined to TAKE AN ARGUMENT — the widely-copied
/// "Fig. 1: → Fig. 1." hack. Real `\fnum@<type>` takes none, but LaTeX's
/// `\@makecaption` is `\sbox\@tempboxa{#1: #2}`, so a one-argument version eats
/// the `:` that follows; pdflatex accepts it (measured: 0 errors, renders
/// "Figure 1. A caption.").
///
/// LaTeXML has no `:` TOKEN to eat — the separator is a tag ATTRIBUTE
/// (`\lx@tag[][: ]`) — so the argument scan ran past the hook and swallowed the
/// caption group's closing brace: the `<figure>` never closed and **every
/// following section, the bibliography included, was absorbed into it**. That is
/// the truncation mechanism behind witnesses 2605.01731 (18 figures x 3 errors)
/// and 2605.12842 (10 x 3), both confirmed live on the fleet run. A 2026-07-14
/// note put the breadth at 18 papers via a `grep 'lx@tag@intags'` proxy; that
/// proxy re-measures to 23 papers over 2605+2606, only 2 of which carry this
/// cause's signature — the symptom has several causes.
///
/// SHARED-FAILURE, not a Rust-only defect: same-host Perl 0.8.8 raises 9 errors
/// on the two-hook form and Rust raised 7, so fixing it is a deliberate
/// surpass — **OXIDIZED_DESIGN #85**, `KNOWN_PERL_ERRORS #68`. Pre-fix this
/// fixture raised **10** errors.
///
/// All three `\csname fnum@…\endcsname` hooks are exercised: the caption one
/// (`\lx@fnum@@`), the toc one (`\lx@fnum@toc@@`) and the theorem-header one
/// (`latex_constructs.rs`).
#[test]
fn cluster_fnum_arg_hook() {
  // `convert_to_xml` is the strict 0-error gate AND returns the XML.
  let x = convert_to_xml("tests/cluster_regressions/fnum_arg_hook.tex");
  // The figure closes, so what follows it is a SIBLING, not a descendant.
  // This is the property that actually matters — an unclosed <figure> is what
  // swallowed the rest of the document.
  let bib = x
    .find("<bibliography")
    .expect("no bibliography element:\n{x}");
  assert!(
    x[..bib].contains("</figure>"),
    "the <figure> must close before the bibliography — an open one absorbs it:\n{x}"
  );
  assert_eq!(
    x.matches("<section").count(),
    2,
    "both sections should be siblings at top level:\n{x}"
  );
  assert!(
    x.contains("Text after must survive"),
    "the body after the figure was lost:\n{x}"
  );
  // Each arg-taking hook actually supplied the label it defines: the caption's
  // `\figurename~\thefigure.`, the toccaption's bare `\thefigure.`, and the
  // theorem's `\thethmx.` — none of them the default that fires when no
  // `\fnum@<type>` exists.
  // NBSP, not a space: the hook body is `\figurename~\thefigure.`, so the `~`
  // is itself evidence that the author's definition ran rather than the default.
  assert!(
    x.contains("<tag close=\": \">Figure\u{a0}1.</tag>"),
    "\\fnum@figure did not supply the caption tag:\n{x}"
  );
  assert!(
    x.contains(r#"<tag close=" ">1.</tag>"#),
    "\\fnum@toc@figure did not supply the toccaption tag:\n{x}"
  );
  assert!(
    x.contains("<theorem") && x.contains(r#"<tag>1.</tag>"#),
    "\\fnum@thmx did not supply the theorem header tag:\n{x}"
  );
}

/// A `robust` DefConstructor must revert under its ORIGINAL control sequence,
/// not the munged one.
///
/// Perl `Package.pm` L1480-1481 gives a `robust` DefConstructor
/// `alias => $cs` whenever the caller supplied no explicit alias. `robust`
/// installs the real definition under `\ref` + a literal trailing SPACE —
/// LaTeX2e's `\DeclareRobustCommand` idiom, where `\ref` expands to
/// `\protect\ref␣` and `\ref␣` holds the body — so without that alias the
/// whatsit reverts as `\ref␣` and the space rides into the `ltx:Math` `tex=`
/// attribute, and from there into the MathML `alttext`, which is the
/// screen-reader / no-MathML fallback. `\ref` is LaTeXML's only `robust`
/// DefConstructor (the `robust` DefMath entries pass an explicit alias, and
/// Perl deliberately does NOT apply this fallback to `DefPrimitiveI`, L1318).
///
/// Ground truth: same-host Perl LaTeXML 0.8.8 on this exact fixture emits all
/// three `tex=` attributes byte-identically to the assertions below, with zero
/// errors in both engines.
#[test]
fn cluster_robust_cs_reverts_unmunged() {
  let x = convert_to_xml("tests/cluster_regressions/robust_cs_reversion.tex");
  assert!(
    !x.contains(r"\ref {"),
    "a robust constructor reverted under its munged `\\ref ` name, so the \
     trailing space leaked into tex= (and thence the MathML alttext):\n{x}"
  );
  // `\pageref` is `\let` to `\ref`, so it reverts as `\ref` too — both
  // formulas therefore carry the identical reversion.
  assert_eq!(
    x.matches(r#"tex="x+\ref{sec:one}""#).count(),
    2,
    "expected \\ref and \\pageref to both revert as `x+\\ref{{sec:one}}`:\n{x}"
  );
  // And inside \text, where the reversion is nested one level deeper.
  assert!(
    x.contains(r#"tex="x+\text{see \ref{sec:one}}""#),
    "the nested \\ref reversion inside \\text does not match Perl:\n{x}"
  );
}

/// CrossRef must resolve an RDFa `aboutidref` into a real `about`.
///
/// `lxRDFa.sty` records an intra-document RDFa subject as `aboutidref` rather
/// than a URL, because the URL is not knowable until the document has been split
/// — `LaTeXML-common.rnc` L301: "it will be converted to `aboutidref` and `about`
/// during post-processing". Rust had no port of Perl's
/// `CrossRef.pm::fill_in_RDFa_refs` (L372-398), so that conversion never
/// happened: the `aboutidref` survived into the output where nothing reads it,
/// and the RDFa triple lost its SUBJECT entirely.
///
/// The companion half — `outerWrapper` copying `about`/`property`/`typeof`/… onto
/// `<m:math>` — is guarded by `90_latexmlpost::mathouter_post_test` against a
/// Perl golden. Together they make `\lxRDFa[//ltx:Math]{about=#thm1,…}`
/// byte-identical to same-host Perl 0.8.8 end to end, `about="#thm1"` included.
#[test]
fn cluster_rdfa_math_subject() {
  let x = convert_and_post_clean("tests/cluster_regressions/rdfa_math_subject.tex");
  assert!(
    x.contains(r##"about="#thm1""##),
    "CrossRef did not resolve `aboutidref` into `about`, so the RDFa subject is \
     lost (no port of fill_in_RDFa_refs):\n{x}"
  );
  // The author-visible attributes that ride along must survive too.
  for attr in [r#"property="ex:formula""#, r#"typeof="ex:Eq""#] {
    assert!(x.contains(attr), "missing RDFa attribute {attr}:\n{x}");
  }
}

/// fvextra `backgroundcolor` on a framed `Verbatim` paints the frame box (issue
/// #525, sub-problem 3). Asserted as a substring rather than an exact golden
/// because fvextra's own `backgroundcolor` machinery is version-dependent (a
/// newer host fvextra paints it per line via `\FV@BGColor@List`, which we
/// neutralize in `fvextra_sty.rs`; the older TL fvextra lacks the key, which we
/// port there): either way the colour lands as `background:<hex>` on the single
/// `ltx_framed_verbatim` box (`fancyvrb_sty.rs`), stable across fvextra versions.
#[test]
fn cluster_fvextra_backgroundcolor_paints_the_frame_box() {
  let x = convert_to_xml("tests/cluster_regressions/fancyvrb_bgcolor.tex");
  assert!(
    x.contains("ltx_framed_verbatim") && x.contains("background:#FFFF00"),
    "fvextra backgroundcolor should paint the frame box yellow (issue #525):\n{x}"
  );
  // The per-line strip must NOT survive as nested background boxes (version noise).
  assert!(
    !x.contains(r##"backgroundcolor="#FFFF00""##),
    "the per-line fvextra background strip should be neutralized, not captured:\n{x}"
  );
}

/// fvextra `breaklines=true` on a framed `Verbatim` surfaces the wrapping
/// directive as a stable `ltx_break` css class on the frame box, so a stylesheet
/// can style wrapping vs non-wrapping verbatims apart WITHOUT reaching for the
/// accidental `:has(.ltx_parbox)` selector (issue #702). Beyond-Perl, same spirit
/// as the `frame=single`->`ltx_framed_verbatim` remap (OXIDIZED_DESIGN #111): both
/// engines emit no breaklines hook natively. The class rides `\lx@fv@breakclass`,
/// which `fancyvrb_sty.rs` defaults to empty (plain fancyvrb never wraps) and
/// `fvextra_sty.rs` redefines to fire off `\ifFV@breaklines`. Only the break box
/// gets it: exactly one `ltx_break` against two `ltx_framed_verbatim` boxes.
#[test]
fn cluster_fvextra_breaklines_class() {
  let x = convert_to_xml("tests/cluster_regressions/fvextra_breaklines_class.tex");
  assert_eq!(
    x.matches("ltx_framed_verbatim").count(),
    2,
    "both Verbatim boxes should frame (issue #702):\n{x}"
  );
  assert_eq!(
    x.matches("ltx_break").count(),
    1,
    "only the breaklines=true box should carry the ltx_break hook (issue #702):\n{x}"
  );
}

/// A faked space's glyph run must be sized by the font the skip was DIGESTED
/// in, not by whatever font happens to be current when the document is BUILT.
///
/// `tex_glue::dimension_to_spaces` expresses a width in **ems** and picks
/// Unicode space glyphs to match, so the answer depends entirely on which font
/// supplies the em. Perl `TeX_Glue.pool.ltxml` L44 reads the live
/// `LookupValue('font')` — inside a constructor that is the font at CONSTRUCTION
/// time, i.e. whatever the document ends in, since Perl builds only after the
/// whole document is digested. Appending `\small` before `\end{document}`
/// therefore changes the glyph chosen for a skip that occurred pages earlier.
///
/// That ambient read made the result depend on WHEN the build ran, which broke
/// the eager/streaming byte-identity invariant outright: streaming builds
/// mid-document, so it read the local font. Rust now uses the whatsit's own
/// font (OXIDIZED_DESIGN #96, KNOWN_PERL_ERRORS #74) — deterministic, and the
/// glyph run finally approximates the true pt width in the font that actually
/// renders it.
///
/// The witness is fancyvrb's line number: the `numbers=left` skip is digested
/// inside the NUMBER's font (`fontsize="56%"`), while `fontsize=\small` makes
/// the surrounding verbatim differ from both that and the document default —
/// three distinct sizes, so every candidate font gives a different answer.
/// `06_cluster_regressions` fixtures are also swept by
/// `114_streaming_cluster_regressions`, which is what pins eager == streaming;
/// this test pins the VALUE, so a future "match Perl here" cannot quietly
/// restore the ambient read while keeping the sweep green.
#[test]
fn faked_space_is_sized_by_the_font_it_was_digested_in() {
  let xml = convert_to_xml("tests/cluster_regressions/fancyvrb_fontsize_numbers.tex");
  // Two em-quads + a three-per-em, measured in the 56% line-number font.
  // Perl 0.8.8 emits `1\u{2003}\u{2009}` here — one em-quad + a thin space,
  // measured in the document's FINAL font, which is not the font that renders
  // these glyphs. The divergence is intentional; see the doc comment.
  assert!(
    xml.contains("1\u{2003}\u{2003}\u{2004}</text>"),
    "the line-number skip must be sized by the line-number font:\n{xml}"
  );
}

/// `\mathversion{bold}` switches the MATH font, exactly like `\boldmath`
/// (`plain_base.rs`) — Perl `latex_constructs.pool.ltxml` L5290-5297,
/// `AssignValue(mathfont => LookupValue('mathfont')->merge(forcebold => N))`. It
/// was using `MergeFont!`, which merges the current *text* `font` value, so
/// `\mathversion{bold}` never reached the math font; and an unknown version fell
/// to a silent `_ => {}` where Perl raises `Error('unexpected', …)`. The `{...}`
/// groups isolate the `local` assignment so `p`/`s` stay plain math italic.
#[test]
fn mathversion_switches_the_mathfont_like_boldmath() {
  let xml = convert_to_xml("tests/cluster_regressions/mathversion_bold_sets_mathfont.tex");
  // q (grouped \mathversion{bold}) must match r (grouped \boldmath): both bold.
  assert!(
    xml.contains(r#"<XMTok font="bold italic" role="UNKNOWN">q</XMTok>"#),
    "\\mathversion{{bold}} did not bold the math font — MergeFont! merges the \
     TEXT font, leaving math un-bold:\n{xml}"
  );
  assert!(
    xml.contains(r#"<XMTok font="bold italic" role="UNKNOWN">r</XMTok>"#),
    "\\boldmath reference is not bold — the test premise is broken:\n{xml}"
  );
  // p and s, outside any bold group, must NOT be bold (clean local-scope reset).
  assert!(
    xml.contains(r#"<XMTok font="italic" role="UNKNOWN">p</XMTok>"#)
      && xml.contains(r#"<XMTok font="italic" role="UNKNOWN">s</XMTok>"#),
    "plain math outside the bold groups must stay italic, not bold:\n{xml}"
  );

  // An unknown version raises Error('unexpected', …) (Perl L5297); pre-fix the
  // `_ => {}` arm swallowed it (0 errors), so "exactly 1 error" is the guard.
  let _ = convert_expecting_errors(
    "tests/cluster_regressions/mathversion_unknown_version_errors.tex",
    1,
  );
}

/// arXiv/html_feedback#6638 (arXiv:2511.14625v1): the `\twocolumn[...]` optional
/// argument is a one-column-spanning header that real LaTeX typesets in a box
/// (`\@topnewpage`), so a font declaration inside it (`\Large`) is scoped to the
/// header and must not leak into the body. Perl's simplified `\twocolumn` splices
/// the argument unscoped and leaks — cvpr's `\maketitlesupplementary` does
/// `\twocolumn[\centering\Large … Supplementary Material …]`, which made the whole
/// Supplementary section render oversized. Our `\twocolumn` now groups the header.
#[test]
fn twocolumn_optional_header_font_does_not_leak_into_body() {
  let x = convert_to_xml("tests/cluster_regressions/twocolumn_optarg_font_scope.tex");
  // The spanning header itself IS \Large (the fix must not swallow that).
  assert!(
    x.contains(r#"fontsize="144%""#),
    "the \\twocolumn spanning header lost its \\Large size:\n{x}"
  );
  // But the section heading and body after it must be normal size — no leak.
  let after = &x[x.find("Body Section").unwrap_or(0)..];
  assert!(
    !after.contains("fontsize="),
    "the header's \\Large leaked past \\twocolumn into the body:\n{x}"
  );
}

/// arXiv/html_feedback#6876 (arXiv:2311.15365v3): the `derivative` package's
/// differential/derivative operators leaked as undefined control sequences.
/// physics.sty covers the `\dv`/`\pdv`/`\dd` overlap but NOT derivative-only
/// commands like `\mdif` (material differential, 36 uses on the witness). There
/// is no Perl binding; the faithful fix force-raw-loads the real `derivative.sty`
/// (`derivative_sty.rs`, `texlive-science`), which our expl3 support executes.
///
/// Red→green signal: with `\mdif` DEFINED it expands during digestion, so the
/// operators vanish from the output entirely; UNDEFINED they stay as raw `\mdif`
/// source plus an `Error:undefined`.
#[test]
fn derivative_package_defines_its_operators() {
  let x = convert_to_xml_contrib("tests/cluster_regressions/derivative_operators.tex");
  for leak in [r"\mdif", r"\odif", r"\odv", r"\pdv"] {
    assert!(
      !x.contains(leak),
      "derivative operator leaked as undefined raw CS `{leak}` (binding not loaded?):\n{x}"
    );
  }
  // The operators produced real math structure rather than nothing.
  assert!(
    x.contains("<XMApp") && x.contains("<Math"),
    "derivative operators produced no math content:\n{x}"
  );
}

/// arXiv/html_feedback#6876: `\cref` inside math `\text{}` reverted its
/// inter-word tie to the internal `\lx@tilde` CS in the `tex=` attribute, where
/// Perl reverts a plain `~`. The `show=` attribute was already `creftype~refnum`
/// in both engines; only the Semiverbatim `tex=` reversion leaked the CS. Now
/// byte-matches Perl (`cleveref_sty.rs` emits a catcode-OTHER `~`).
#[test]
fn cleveref_cref_in_math_reverts_tie_as_plain_tilde() {
  let x = convert_to_xml("tests/cluster_regressions/cref_in_math_tilde.tex");
  assert!(
    x.contains("creftype~refnum"),
    "the cref show/reversion tie is missing:\n{x}"
  );
  assert!(
    !x.contains(r"\lx@tilde"),
    "the internal \\lx@tilde CS leaked into the tex= reversion:\n{x}"
  );
}

/// arXiv/html_feedback#140: a bare `\newtheorem{arch}{Architecture}` (no explicit
/// `\crefname`) referenced with `\cref` rendered only the number ("1") instead of
/// "Architecture 1" as in the PDF. Real cleveref auto-names such theorems by their
/// heading; LaTeXML's native `\newtheorem` bypasses cleveref's `\@ynthm` patches, so
/// both engines dropped the type name. The creftype formatter now falls back to the
/// theorem heading `\lx@name@<type>` (surpass-Perl, OXIDIZED_DESIGN #131).
#[test]
fn cleveref_custom_theorem_cref_shows_heading_name() {
  let x = convert_and_post_clean("tests/cluster_regressions/cleveref_newtheorem_crefname.tex");
  // The \cref now resolves creftype → the theorem heading "Architecture" AND the
  // refnum "1" — both as ref tags, so the link reads "Architecture 1".
  assert!(
    x.contains(r#"<text class="ltx_ref_tag">Architecture</text>"#),
    "\\cref did not render the custom theorem's type name \"Architecture\":\n{x}"
  );
  assert!(
    x.contains(r#"<text class="ltx_ref_tag">1</text>"#),
    "\\cref lost the reference number \"1\":\n{x}"
  );
  // The type name is surfaced at core time as a creftype tag on the theorem.
  assert!(
    x.contains(r#"<tag role="creftype">Architecture</tag>"#),
    "the theorem is missing its creftype type-tag:\n{x}"
  );
}

/// html_feedback#861 (arXiv:2403.15796, neurips_2023 preprint): "everything after
/// the abstract missing". The paper defines its own brace-gobbling `\newcommand
/// {\hide}[1]{}` and comments blocks out with `\hide{ … }`. The neurips binding
/// defined the `{hide}` environment UNCONDITIONALLY, so `\hide` was already a CS,
/// the author's `\newcommand` was ignored as a redefinition, and `\hide{` opened a
/// runaway environment that ran to `\end{document}` looking for `\endhide` —
/// swallowing the whole body. The real neurips_2023.cls only defines `{hide}` in
/// SUBMISSION mode (not preprint/final), so the binding now gates on it. Perl 0.8.8
/// drops the body identically; this surpasses it. Canary: the visible sections
/// AFTER the hidden block must survive, and the hidden content must stay gone.
#[test]
fn neurips_hide_preprint_preserves_body() {
  let x = convert_to_xml("tests/cluster_regressions/neurips_hide_preprint_body.tex");
  // The body after the abstract survives — the reported symptom.
  assert!(
    x.contains("Visible Introduction") && x.contains("The body after the abstract must survive."),
    "neurips preprint: body after the abstract was swallowed:\n{x}"
  );
  assert!(
    x.contains("Conclusion") && x.contains("Concluding remarks that must not vanish."),
    "neurips preprint: sections after the hidden block are missing:\n{x}"
  );
  // …and `\hide{…}` still hid its argument (the author's intent).
  assert!(
    !x.contains("Hidden paragraph") && !x.contains("Hidden Introduction"),
    "neurips preprint: the \\hide{{…}} block leaked into the output:\n{x}"
  );
}

/// Companion to the above (html_feedback#140): an explicit `\crefname{widget}{gadget}
/// {gadgets}` must WIN over the theorem-heading fallback — `\cref` renders the explicit
/// "gadget", not the heading "Widget". `\crefname` is now a real definition (a faithful
/// port of cleveref's `\@crefname`), not a no-op stub, so it populates `\cref@widget@name`
/// and the primary branch takes precedence over `\lx@name@widget`.
#[test]
fn cleveref_explicit_crefname_overrides_heading() {
  let x = convert_and_post_clean("tests/cluster_regressions/cleveref_newtheorem_crefname.tex");
  assert!(
    x.contains(r#"<text class="ltx_ref_tag">gadget</text>"#),
    "explicit \\crefname name \"gadget\" did not reach the \\cref link:\n{x}"
  );
  // The heading "Widget" must NOT be used as this ref's type name (the explicit
  // \crefname overrides it). It still appears in the theorem's own <tag>/title.
  assert!(
    !x.contains(r#"<text class="ltx_ref_tag">Widget</text>"#),
    "the heading \"Widget\" leaked past the explicit \\crefname into the ref:\n{x}"
  );
}
