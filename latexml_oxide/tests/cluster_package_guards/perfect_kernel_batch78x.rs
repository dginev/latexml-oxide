//! Red/green guards for batch 78x (grid design payload 2, the panel grid rule; user rulings 2026-10-22,
//! FIDELITY_CHARTER §2 "Float panels"): a float's panels are set in rows from what the source states — an explicit
//! break (a blank line or `\par`, `\\` in a `{center}`, a `{center}` boundary, `\vskip`, `\linebreak`) always breaks,
//! the stated widths and the stated glue between panels fill a row up to the class's real line (a two-column `figure`
//! the column, a `figure*` the text width, the geometry's width), the registers keeping Perl's values. Each guard
//! converts its repro under `tools/perfect_kernel/repros/captions-floats/` (pdflatex-checked: rows from the float
//! boxes' `\showbox`), asserts the diagnostics by count and the floats whole. OXIDIZED_DESIGN_DIVERGENCES #885.
use latexml::util::test::{assert_element, convert_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// Convert `tex`: 0 errors, 0 warnings.
fn convert_clean(tex: &str) -> String {
  let (log, xml) = convert_with(tex, Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  xml
}

/// A float's tags: `Figure n`.
fn figure_tags(n: u32) -> String {
  format!(
    "<tags><tag>Figure {n}</tag><tag role=\"refnum\">{n}</tag><tag role=\"typerefnum\">Figure {n}</tag></tags>"
  )
}

/// Its captions, `class` their alignment class (` class="ltx_centering"` or empty).
fn figure_captions(n: u32, class: &str, text: &str) -> String {
  format!(
    "<toccaption{class}><tag close=\" \">{n}</tag>{text}</toccaption><caption{class}><tag close=\": \">Figure {n}</tag>\
     {text}</caption>"
  )
}

/// A `\rule` panel `width` wide and 1cm high, centred, `fraction` of its row's line (`panelwidth`, batch 79c; none
/// where empty).
fn rule_panel(width: &str, fraction: &str) -> String {
  let panel = if fraction.is_empty() {
    String::new()
  } else {
    format!(" panelwidth=\"{fraction}\"")
  };
  format!(
    "<rule align=\"center\" class=\"ltx_figure_panel\" height=\"28.5pt\"{panel} width=\"{width}\"/>"
  )
}

/// The `ltx:break`s directly in the float `id` (the rows' separators), in order, as their markup.
fn float_breaks(xml: &str, id: &str) -> Vec<String> {
  let start = xml
    .find(&format!("xml:id=\"{id}\">"))
    .unwrap_or_else(|| panic!("no {id}: {xml}"));
  let (mut depth, mut rest, mut breaks) = (1, &xml[start..], Vec::new());
  while let Some(at) = rest.find('<') {
    rest = &rest[at..];
    let tag = &rest[..rest.find('>').map_or(rest.len(), |end| end + 1)];
    rest = &rest[tag.len()..];
    if tag.starts_with("</") {
      depth -= 1;
      if depth == 0 {
        break;
      }
    } else if !tag.ends_with("/>") {
      depth += 1;
    } else if depth == 1 && tag.starts_with("<break") {
      breaks.push(tag.to_string());
    }
  }
  breaks
}

/// A source paragraph break between a float's panels ends their row: our panels are digested in vertical mode, where
/// TeX's are in a paragraph, so the `\par` left nothing in the tree (`note_panel_separator`). The two
/// [label minipage + panel] rows stay two rows (pdflatex). Red (57o..int65a: "(b)" joined the first row). Witnesses
/// 1110.0084 Figs 1, 12-16, 27, 29. Repro captions-floats/par_breaks_a_panel_row.
#[test]
fn par_breaks_a_panel_row() {
  let xml = convert_clean(repro!("captions-floats/par_breaks_a_panel_row.tex"));
  let row = |n: u32, label: &str| {
    format!(
      "<block class=\"ltx_figure_panel\" panelwidth=\"0.94\" xml:id=\"S0.F1.{n}\"><p class=\"ltx_minipage\" vattach=\"middle\" \
       width=\"13.8pt\" xml:id=\"S0.F1.{n}.1\">({label})</p><rule class=\"ltx_minipage\" height=\"56.9pt\" \
       vattach=\"middle\" width=\"310.5pt\"/></block>"
    )
  };
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F1\""],
    &format!(
      "<figure class=\"ltx_panels_gallery\" inlist=\"lof\" panelcols=\"1\" panelfill=\"0.94\" panelline=\"1\" panelrows=\"2\" xml:id=\"S0.F1\">{}{}<break class=\"ltx_break\"/>{}{}</figure>",
      figure_tags(1),
      row(1, "a"),
      row(2, "b"),
      figure_captions(1, "", "Text label minipages.")
    ),
  );
}

/// The glue the source states between two panels counts towards their row: `\hspace{1in}` between two
/// `0.45\textwidth` panels is past the line (two rows), `\hfill` and plain spaces count nothing, two `\quad`s between
/// three `0.3\textwidth` panels fit (pdflatex). Red (int65a: one row in all three). Witnesses 0706.4410 Figs 3, 5,
/// 1312.0675 Figs 5, 7. Repro captions-floats/glue_between_panels_counts_towards_their_row.
#[test]
fn glue_between_panels_counts_towards_their_row() {
  let xml = convert_clean(repro!(
    "captions-floats/glue_between_panels_counts_towards_their_row.tex"
  ));
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F1\""],
    &format!(
      "<figure class=\"ltx_panels_gallery\" inlist=\"lof\" panelcols=\"1\" panelfill=\"0.45\" panelline=\"1\" panelrows=\"2\" xml:id=\"S0.F1\">{}{}<break class=\"ltx_break\"/>{}{}</figure>",
      figure_tags(1),
      rule_panel("155.3pt", "0.45"),
      rule_panel("155.3pt", "0.45"),
      figure_captions(1, " class=\"ltx_centering\"", "Stated glue past the line.")
    ),
  );
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F3\""],
    &format!(
      "<figure class=\"ltx_panels_gallery\" inlist=\"lof\" panelcols=\"3\" panelfill=\"0.9\" panelline=\"1\" panelrows=\"1\" xml:id=\"S0.F3\">{}{}{}{}{}</figure>",
      figure_tags(3),
      rule_panel("103.5pt", "0.3"),
      rule_panel("103.5pt", "0.3"),
      rule_panel("103.5pt", "0.3"),
      figure_captions(3, " class=\"ltx_centering\"", "Quads within the line.")
    ),
  );
  assert!(float_breaks(&xml, "S0.F2").is_empty(), "{xml}");
  // No row separator is left in the tree.
  assert!(!xml.contains("_panelsep"), "{xml}");
}

/// A two-column document's `figure` sets its rows against the column, a `figure*` against the text width, while
/// `\columnwidth` keeps Perl's value (= `\textwidth`, latex_constructs.pool.ltxml:321) for everything else: two
/// `0.74\columnwidth` panels share a starred float's row, two `0.45\textwidth` ones do not share a column's, two
/// `0.45\linewidth` graphics do, and a width key given twice counts its last value (pdflatex). Red (int65a: Figure 1
/// two rows, Figure 2 one). Witnesses 1707.00190 Fig 2, nucl-ex0511052 Fig 13, 2502.05562 Fig 6, astro-ph0205132,
/// 1705.10798, 2601.01508. Repro captions-floats/two_column_floats_set_rows_against_the_column.
#[test]
fn two_column_floats_set_rows_against_the_column() {
  let xml = convert_clean(repro!(
    "captions-floats/two_column_floats_set_rows_against_the_column.tex"
  ));
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F1\""],
    &format!(
      "<figure class=\"ltx_panels_gallery\" inlist=\"lof\" panelcols=\"2\" panelfill=\"0.74\" panelline=\"2\" panelrows=\"1\" xml:id=\"S0.F1\">{}{}{}{}</figure>",
      figure_tags(1),
      rule_panel("255.3pt", "0.37"),
      rule_panel("255.3pt", "0.37"),
      figure_captions(
        1,
        " class=\"ltx_centering\"",
        "Column-wide panels in a starred float."
      )
    ),
  );
  // The panels keep Perl's sizes: `0.45\textwidth` is 155.3pt, the register at 345pt.
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F2\""],
    &format!(
      "<figure class=\"ltx_panels_gallery\" inlist=\"lof\" panelcols=\"1\" panelfill=\"0.9\" panelline=\"1\" panelrows=\"2\" xml:id=\"S0.F2\">{}{}<break class=\"ltx_break\"/>{}{}</figure>",
      figure_tags(2),
      rule_panel("155.3pt", "0.9"),
      rule_panel("155.3pt", "0.9"),
      figure_captions(
        2,
        " class=\"ltx_centering\"",
        "Text-wide panels in a column."
      )
    ),
  );
  assert!(float_breaks(&xml, "S0.F3").is_empty(), "{xml}");
  assert!(float_breaks(&xml, "S0.F4").is_empty(), "{xml}");
  assert_eq!(xml.matches("ltx_figure_panel").count(), 8, "{xml}");
}

/// A `{center}`'s lines are rows of their own: a `\\` between sub-figures in it, a blank line in it, and its
/// boundary end a row (pdflatex 2+1, 2+1, 1+2). Red (int65a: one row in all three; the `\\` skipped in the centre's
/// capture). Witnesses 0912.4528 Fig 4, 1206.5015 Fig 3. Repro captions-floats/a_center_and_its_breaks_end_a_panel_row.
#[test]
fn a_center_and_its_breaks_end_a_panel_row() {
  let xml = convert_clean(repro!(
    "captions-floats/a_center_and_its_breaks_end_a_panel_row.tex"
  ));
  let centred = |width: &str| {
    format!(
      "<rule align=\"center\" class=\"ltx_figure_panel\" height=\"28.5pt\" panelwidth=\"0.3\" width=\"{width}\"/>"
    )
  };
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F2\""],
    &format!(
      "<figure class=\"ltx_panels_gallery\" inlist=\"lof\" panelcols=\"2\" panelfill=\"0.6\" panelline=\"1\" panelrows=\"2\" xml:id=\"S0.F2\">{}{}{}<break class=\"ltx_break\"/>{}{}</figure>",
      figure_tags(2),
      centred("103.5pt"),
      centred("103.5pt"),
      centred("103.5pt"),
      figure_captions(2, "", "A blank line inside a centre.")
    ),
  );
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F3\""],
    &format!(
      "<figure class=\"ltx_panels_gallery\" inlist=\"lof\" panelcols=\"2\" panelfill=\"0.6\" panelline=\"1\" panelrows=\"2\" xml:id=\"S0.F3\">{}{}<break class=\"ltx_break\"/>{}{}{}</figure>",
      figure_tags(3),
      centred("103.5pt"),
      centred("103.5pt"),
      centred("103.5pt"),
      figure_captions(3, "", "Two centres.")
    ),
  );
  // Figure 1: (a) (b) | (c).
  let figure1 = &xml[xml.find("xml:id=\"S0.F1\">").unwrap()..];
  let (sf2, brk, sf3) = (
    figure1.find("xml:id=\"S0.F1.sf2\"").unwrap(),
    figure1.find("<break").unwrap(),
    figure1.find("xml:id=\"S0.F1.sf3\"").unwrap(),
  );
  assert!(sf2 < brk && brk < sf3, "{xml}");
  assert_eq!(float_breaks(&xml, "S0.F1").len(), 1, "{xml}");
}

/// Between a float's panels a `\vskip` ends the row (in TeX it ends their paragraph) and a forced `\linebreak` ends
/// it, where a `\vspace` and `\medskip` do not (latex.ltx:9361-9370 `\@vspace`: `\vadjust` outside vertical mode).
/// pdflatex: one row of three, then one panel per row. Red (int65a: one row in both). Repro
/// captions-floats/vskip_ends_a_panel_row_vspace_does_not.
#[test]
fn vskip_ends_a_panel_row_vspace_does_not() {
  let xml = convert_clean(repro!(
    "captions-floats/vskip_ends_a_panel_row_vspace_does_not.tex"
  ));
  assert!(float_breaks(&xml, "S0.F1").is_empty(), "{xml}");
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F2\""],
    &format!(
      "<figure class=\"ltx_panels_gallery\" inlist=\"lof\" panelcols=\"1\" panelfill=\"0.3\" panelline=\"1\" panelrows=\"3\" xml:id=\"S0.F2\">{}{}<break class=\"ltx_break\"/>{}<break class=\"ltx_break\"/>{}{}\
       </figure>",
      figure_tags(2),
      rule_panel("103.5pt", "0.3"),
      rule_panel("103.5pt", "0.3"),
      rule_panel("103.5pt", "0.3"),
      figure_captions(2, " class=\"ltx_centering\"", "A skip and a line break.")
    ),
  );
}

/// The geometry's text width (geometry_sty.rs `\Gm@tw`, 469.75pt for letter paper with 1in margins) is the line the
/// rows are set against, the stated widths taken at it: four `0.3\textwidth` panels are three and one, two `3in`
/// graphics share a row (pdflatex). Red (int65a: one row of four, Perl's 345pt widths against the geometry's line).
/// Witnesses 1508.01788 Figs 19-22, 1411.5428 Figs 6-7, 2111.05508. Repro
/// captions-floats/geometry_text_width_sets_the_panel_rows.
#[test]
fn geometry_text_width_sets_the_panel_rows() {
  let xml = convert_clean(repro!(
    "captions-floats/geometry_text_width_sets_the_panel_rows.tex"
  ));
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F1\""],
    &format!(
      "<figure class=\"ltx_panels_gallery\" inlist=\"lof\" panelcols=\"3\" panelfill=\"0.9\" panelline=\"1\" panelrows=\"2\" xml:id=\"S0.F1\">{}{}{}{}<break class=\"ltx_break\"/>{}{}</figure>",
      figure_tags(1),
      rule_panel("103.5pt", "0.3"),
      rule_panel("103.5pt", "0.3"),
      rule_panel("103.5pt", "0.3"),
      rule_panel("103.5pt", "0.3"),
      figure_captions(
        1,
        " class=\"ltx_centering\"",
        "Four panels of three tenths."
      )
    ),
  );
  assert!(float_breaks(&xml, "S0.F2").is_empty(), "{xml}");
}

/// An uncaptioned sub-figure is as wide as its body's material, its spaces and its `\label` counting nothing: two
/// `\subfigure{⏎\rule{0.45\textwidth}{1cm}⏎\label{…}}` share a row (pdflatex; "See 1(a) and 1(b)."). Red (int65a: one
/// per row, each measured with its spaces and the label's key text). Witness 1007.2355 Figs 1, 3. Repro
/// captions-floats/an_uncaptioned_subfigure_is_as_wide_as_its_body.
#[test]
fn an_uncaptioned_subfigure_is_as_wide_as_its_body() {
  let xml = convert_clean(repro!(
    "captions-floats/an_uncaptioned_subfigure_is_as_wide_as_its_body.tex"
  ));
  let panel = |n: u32, letter: &str, label: &str| {
    format!(
      "<figure align=\"center\" class=\"ltx_figure_panel\" labels=\"LABEL:fig:{label}\" panelwidth=\"0.45\" \
       xml:id=\"S0.F1.sf{n}\">\
       <tags><tag><text fontsize=\"80%\" xml:id=\"S0.F1.sf{n}.1\">({letter})</text></tag>\
       <tag role=\"refnum\">1({letter})</tag></tags><rule height=\"28.5pt\" width=\"155.3pt\"/></figure>"
    )
  };
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F1\""],
    &format!(
      "<figure class=\"ltx_panels_gallery\" inlist=\"lof\" panelcols=\"2\" panelfill=\"0.9\" panelline=\"1\" panelrows=\"1\" xml:id=\"S0.F1\">{}{}{}{}</figure>",
      figure_tags(1),
      panel(1, "a", "left"),
      panel(2, "b", "right"),
      figure_captions(1, " class=\"ltx_centering\"", "Left and right.")
    ),
  );
}

/// The `ltx_figure_panel`s in the float `id`, its sub-floats' included.
fn float_panels(xml: &str, id: &str) -> usize {
  let start = xml
    .find(&format!("xml:id=\"{id}\">"))
    .unwrap_or_else(|| panic!("no {id}: {xml}"));
  let (mut depth, mut rest, mut panels) = (1, &xml[start..], 0);
  while let Some(at) = rest.find('<') {
    rest = &rest[at..];
    let tag = &rest[..rest.find('>').map_or(rest.len(), |end| end + 1)];
    rest = &rest[tag.len()..];
    if tag.starts_with("</") {
      depth -= 1;
      if depth == 0 {
        break;
      }
    } else {
      panels += usize::from(tag.contains("ltx_figure_panel"));
      depth += usize::from(!tag.ends_with("/>"));
    }
  }
  panels
}

/// A construct's stated width is the argument or key its binding declares (`declare_panel_width`), evaluated again at
/// the class's real geometry: four `width=0.23\linewidth,height=0.23\linewidth` graphics and two
/// `\resizebox{0.45\textwidth}{!}{\includegraphics[width=\textwidth]{…}}` in a two-column `figure*` are one row each
/// (pdflatex). Red (78x r1: 2+2 and 1+1, the height key and the box's body counted as width). Witnesses 1703.09474
/// Fig 13, 2503.09651 Fig 1; review rev78x finding 1. Repro captions-floats/a_panel_states_its_width_not_its_height_or_body.
#[test]
fn a_panel_states_its_width_not_its_height_or_body() {
  let xml = convert_clean(repro!(
    "captions-floats/a_panel_states_its_width_not_its_height_or_body.tex"
  ));
  for (id, panels) in [("S0.F1", 4), ("S0.F2", 2)] {
    assert!(float_breaks(&xml, id).is_empty(), "{xml}");
    assert_eq!(float_panels(&xml, id), panels, "{xml}");
  }
  // The sizes stay Perl's: the graphics at Perl's 345pt line.
  assert_eq!(
    xml
      .matches("options=\"width=79.34853pt,height=79.34853pt\"")
      .count(),
    4,
    "{xml}"
  );
}

/// subcaption's `\subfloat` boxes its body at its natural width; the binding's `{\columnwidth}` is no width the source
/// states (`\lx@panel@unstated`), nor an unknown one (`STATED_LENGTHS_WITHHELD`): every sub-float stays a panel, the
/// three of the `figure*` one row, the two 4.5cm ones of a 247pt column two (pdflatex). Red (78x r1: the first
/// `\subfloat` 0 wide against the real column, out of the panel list; 78x r4: Figure 2 one row, the float unevaluable).
/// Witnesses 2602.06569 S4.F1/F2, 1902.05392 S3.F3; review rev78x finding 1, rev78x4 finding 3. Repro
/// captions-floats/a_subfloat_states_no_width.
#[test]
fn a_subfloat_states_no_width() {
  let xml = convert_clean(repro!("captions-floats/a_subfloat_states_no_width.tex"));
  assert!(float_breaks(&xml, "S0.F1").is_empty(), "{xml}");
  assert_eq!(
    float_breaks(&xml, "S0.F2"),
    ["<break class=\"ltx_break\"/>"],
    "{xml}"
  );
  // (each the width of its one graphic, `sole_graphic_width`, a fraction of its line: batch 79c)
  for (float, n, label, fraction) in [
    (1, 1, "a", "0.229"),
    (1, 2, "b", "0.229"),
    (1, 3, "c", "0.229"),
    (2, 1, "d", "0.508"),
    (2, 2, "e", "0.508"),
  ] {
    assert!(
      xml.contains(&format!(
        "<figure align=\"center\" class=\"ltx_figure_panel\" inlist=\"lof\" labels=\"LABEL:fig:{label}\" \
         panelwidth=\"{fraction}\" xml:id=\"S0.F{float}.sf{n}\">"
      )),
      "{xml}"
    );
  }
  assert_eq!(last_row_panels(&xml, "S0.F2"), 1, "{xml}");
}

/// The glue the source states between panels is evaluated at the real geometry as their widths are: `0.48 +
/// 3×0.24\textwidth` sub-figures with `−0.25\textwidth` of `\hspace` under geometry are one row of four (pdflatex). Red
/// (78x r1: 3+1, the glue at Perl's 345pt). Witness 2311.18531 Fig 1; review rev78x finding 3. Repro
/// captions-floats/stated_glue_at_the_real_geometry.
#[test]
fn stated_glue_at_the_real_geometry() {
  let xml = convert_clean(repro!(
    "captions-floats/stated_glue_at_the_real_geometry.tex"
  ));
  assert!(float_breaks(&xml, "S0.F1").is_empty(), "{xml}");
  assert_eq!(float_panels(&xml, "S0.F1"), 4, "{xml}");
}

/// The geometry's `\Gm@tw` is the page only while `\textwidth` is the class's (`\Gm@doctw`): a later
/// `\setlength{\textwidth}{15cm}` is the page, and two `width=7cm` graphics share its row (pdflatex). Red (78x r1: two
/// rows against elsarticle's 384pt `\Gm@tw`). Witness 1704.03351 Figs 16, 35, 36; review rev78x finding 2. Repro
/// captions-floats/a_later_text_width_is_the_page.
#[test]
fn a_later_text_width_is_the_page() {
  let xml = convert_clean(repro!("captions-floats/a_later_text_width_is_the_page.tex"));
  assert!(float_breaks(&xml, "S0.F1").is_empty(), "{xml}");
  assert_eq!(float_panels(&xml, "S0.F1"), 2, "{xml}");
}

/// The glue `\hspace` states between a float's panels is the kernel's glue (`\lx@panel@hskip`, absorbed as the kernel's
/// box), its separator noted as it is absorbed: an algorithm2e line keeps its indentation and `\unskip` removes the
/// glue whole (pdflatex: "LeftRight"). Red (78x r1: "Compute" and "Left   Right", the `\lx@panel@glue` marker).
/// Witnesses 1610.04467, 1712.10197, 1901.07422, 1901.07434, 2009.06170, 2011.13396, 2405.17108, 2410.15995,
/// 2605.09653; review rev78x finding 4. Repro captions-floats/panel_glue_is_the_kernels_glue.
#[test]
fn panel_glue_is_the_kernels_glue() {
  let xml = convert_clean(repro!("captions-floats/panel_glue_is_the_kernels_glue.tex"));
  // `\hspace{1.5mm}`'s spaces (a three-per-em and a hair space), as the kernel's `\hspace` sets them
  assert!(
    xml.contains("</tags>\u{2004}\u{200a}Compute the thing.;</listingline>"),
    "{xml}"
  );
  assert_element(
    &xml,
    "p",
    &["xml:id=\"S0.F1.1\""],
    "<p xml:id=\"S0.F1.1\">LeftRight</p>",
  );
}

/// A float's begin lets `\linebreak`, `\vspace` and `\hspace` to the panel rule's only while they have the kernel's
/// meaning: the author's own stand (pdflatex "A[V]B[H]C"). Red (78x r1: the `\vspace` let over the author's). Review
/// rev78x finding 11. Repro captions-floats/an_authors_spacing_commands_stand_in_a_float.
#[test]
fn an_authors_spacing_commands_stand_in_a_float() {
  let xml = convert_clean(repro!(
    "captions-floats/an_authors_spacing_commands_stand_in_a_float.tex"
  ));
  assert_element(
    &xml,
    "p",
    &["xml:id=\"S0.F1.1\""],
    "<p xml:id=\"S0.F1.1\">A[V]B[H]C</p>",
  );
}

/// TeX breaks no line in an `\hbox`: two `width=200pt` graphics in one subfigure.sty `\subfigure` (its body one `\hbox`,
/// subfigure.sty:232) are one overfull row, and four sub-figures in one adjustbox one row (pdflatex). Red (int67a: the
/// sub-figure's graphics two rows; 78x r1: the adjustbox's 3+1). Witnesses cond-mat0610021 Fig 11, 2311.18531
/// A7.F5/F6. Repros captions-floats/panels_in_one_hbox_share_a_row, subfigures_in_an_adjustbox_share_a_row.
#[test]
fn panels_in_one_hbox_share_a_row() {
  let xml = convert_clean(repro!("captions-floats/panels_in_one_hbox_share_a_row.tex"));
  assert!(float_breaks(&xml, "S0.F1.sf1").is_empty(), "{xml}");
  assert_eq!(
    xml
      .matches("class=\"ltx_figure_panel\" graphic=\"example-image\"")
      .count(),
    2,
    "{xml}"
  );
  let xml = convert_clean(repro!(
    "captions-floats/subfigures_in_an_adjustbox_share_a_row.tex"
  ));
  // the adjustbox's own paragraph, then the four sub-figures in one row
  assert_eq!(float_breaks(&xml, "S0.F1").len(), 1, "{xml}");
  assert_eq!(float_panels(&xml, "S0.F1"), 5, "{xml}");
  let row = &xml[xml.find("<break class=\"ltx_break\"/>").unwrap()..];
  assert!(
    row.find("S0.F1.fig1").unwrap() < row.find("S0.F1.fig4").unwrap(),
    "{xml}"
  );
}

/// A class binding declares the page its floats' panels are set in rows against (`declare_panel_page`), the registers
/// keeping the binding's 345pt: REVTeX's two-column 246pt column and 510pt text set three `1.5in` graphics 2+1 in a
/// `figure` and three `2.2in` ones in one row of a `figure*` (pdflatex). Red (78x r1: one row each, the 345pt line taken
/// as the column). Witnesses 2308.02289, 1104.5578, 2005.13806; review rev78x §3. Repro
/// captions-floats/a_class_page_sets_the_panel_rows.
#[test]
fn a_class_page_sets_the_panel_rows() {
  let xml = convert_clean(repro!(
    "captions-floats/a_class_page_sets_the_panel_rows.tex"
  ));
  assert_eq!(
    float_breaks(&xml, "Sx1.F1"),
    ["<break class=\"ltx_break\"/>"],
    "{xml}"
  );
  assert!(float_breaks(&xml, "Sx1.F2").is_empty(), "{xml}");
  assert_eq!(float_panels(&xml, "Sx1.F1"), 3, "{xml}");
  assert_eq!(float_panels(&xml, "Sx1.F2"), 3, "{xml}");
}

/// The declared page follows the class's one-column option: IEEEtran `onecolumn` (whose `\if@twocolumn` the binding
/// keeps true) sets two `3.3in` graphics in one row of its 516pt page, and an author's `\textwidth 7.2in` is the page
/// (pdflatex). Red (int67a and 78x r1: two rows in the first; 78x r1 two rows in the second). Witnesses 2211.03584,
/// 1604.07648, 1404.3152, 1809.02562, 2103.06618; review rev78x §3. Repros
/// captions-floats/a_one_column_class_option_sets_the_panel_rows, an_authors_text_width_is_the_class_page.
#[test]
fn a_one_column_class_option_sets_the_panel_rows() {
  for xml in [
    convert_clean(repro!(
      "captions-floats/a_one_column_class_option_sets_the_panel_rows.tex"
    )),
    convert_clean(repro!(
      "captions-floats/an_authors_text_width_is_the_class_page.tex"
    )),
  ] {
    assert!(float_breaks(&xml, "S0.F1").is_empty(), "{xml}");
    assert_eq!(float_panels(&xml, "S0.F1"), 2, "{xml}");
  }
}

/// A graphic that states only its height keeps its aspect ratio, so the height at the real geometry scales its natural
/// width (`declare_panel_width`'s height keys): four `height=0.21\linewidth` sub-figures in a two-column column under
/// geometry are 3+1 (pdflatex). Red (int67a: one row of four). Witness 2209.01559 Fig 6. Repro
/// captions-floats/a_graphics_height_scales_its_width.
#[test]
fn a_graphics_height_scales_its_width() {
  let xml = convert_clean(repro!(
    "captions-floats/a_graphics_height_scales_its_width.tex"
  ));
  assert_eq!(
    float_breaks(&xml, "S0.F1"),
    ["<break class=\"ltx_break\"/>"],
    "{xml}"
  );
  let row = &xml[xml.find("<break class=\"ltx_break\"/>").unwrap()..];
  assert!(
    row.contains("S0.F1.sf4") && !row.contains("S0.F1.sf3"),
    "{xml}"
  );
}

/// The `ltx_figure_panel`s directly in the float `id` after its last `ltx:break` (its last row).
fn last_row_panels(xml: &str, id: &str) -> usize {
  let start = xml
    .find(&format!("xml:id=\"{id}\">"))
    .unwrap_or_else(|| panic!("no {id}: {xml}"));
  let (mut depth, mut rest, mut panels) = (1, &xml[start..], 0);
  while let Some(at) = rest.find('<') {
    rest = &rest[at..];
    let tag = &rest[..rest.find('>').map_or(rest.len(), |end| end + 1)];
    rest = &rest[tag.len()..];
    if tag.starts_with("</") {
      depth -= 1;
      if depth == 0 {
        break;
      }
      continue;
    }
    if depth == 1 {
      if tag.starts_with("<break") {
        panels = 0;
      } else {
        panels += usize::from(tag.contains("ltx_figure_panel"));
      }
    }
    depth += usize::from(!tag.ends_with("/>"));
  }
  panels
}

/// The widths a float states are evaluated inside its group, at its end: a width macro defined in the float (`\def`,
/// `\newcommand`) or one a later `\renewcommand` changes is read with the meaning it has there — two `0.48\columnwidth`
/// graphics are one row of REVTeX's 246pt column in each figure (pdflatex). Red (78x r2: two rows in Figures 1-2, the
/// widths evaluated as the element opened, after the float's group ended). Review rev78x2 finding 1 (`q1.tex`,
/// `q2.tex`). Repro captions-floats/stated_widths_are_evaluated_in_the_float.
#[test]
fn stated_widths_are_evaluated_in_the_float() {
  let xml = convert_clean(repro!(
    "captions-floats/stated_widths_are_evaluated_in_the_float.tex"
  ));
  for n in 1..=4 {
    let id = format!("S0.F{n}");
    assert!(float_breaks(&xml, &id).is_empty(), "{id}: {xml}");
    assert_eq!(float_panels(&xml, &id), 2, "{id}: {xml}");
  }
}

/// A width read through a length register is what the register's `\setlength` stated, evaluated at the class's real
/// geometry (`note_setlength`): `\figw` set to `0.48\columnwidth` in the body is one row of REVTeX's column, `\figv`
/// set to `0.45\textwidth` in the float two rows; a register a primitive assignment set (`\figu=0.48\columnwidth`) holds
/// a value of the registers' geometry, and its float keeps the registers' line, one row (pdflatex: one, two, one row).
/// Red (78x r2: two rows each, the register's value against the real column; int68a: Figure 2 one row). Review
/// rev78x2 finding 1 (`q2.tex` Figs 1, 3). Repro captions-floats/a_width_read_through_a_length_register.
#[test]
fn a_width_read_through_a_length_register() {
  let xml = convert_clean(repro!(
    "captions-floats/a_width_read_through_a_length_register.tex"
  ));
  for (n, breaks) in [(1, 0), (2, 1), (3, 0)] {
    let id = format!("S0.F{n}");
    assert_eq!(float_breaks(&xml, &id).len(), breaks, "{id}: {xml}");
    assert_eq!(float_panels(&xml, &id), 2, "{id}: {xml}");
  }
}

/// A transformed panel's width at the real geometry is its content's, transformed: four `\scalebox{0.5}` of
/// `0.6\columnwidth` graphics, four `width=0.35\columnwidth,angle=90` graphics and four `\rotatebox{90}` of
/// `0.35\columnwidth` graphics are 3+1 each in REVTeX's 246pt column (pdflatex). Red (78x r2: one row of four each, the
/// graphic's real-minus-ours width added unscaled to its box). Review rev78x2 finding 2 (`q3.tex`). Repro
/// captions-floats/a_transformed_panel_scales_its_width.
#[test]
fn a_transformed_panel_scales_its_width() {
  let xml = convert_clean(repro!(
    "captions-floats/a_transformed_panel_scales_its_width.tex"
  ));
  for n in 1..=3 {
    let id = format!("S0.F{n}");
    assert_eq!(
      float_breaks(&xml, &id),
      ["<break class=\"ltx_break\"/>"],
      "{id}: {xml}"
    );
    assert_eq!(float_panels(&xml, &id), 4, "{id}: {xml}");
    assert_eq!(last_row_panels(&xml, &id), 1, "{id}: {xml}");
  }
}

/// subcaption's `\subfloat`s in one adjustbox or `\resizebox` are one TeX `\hbox`, one row, though each begins in a
/// group of its own: the row is the group that set the box's mode (`note_panel_hbox`). Red (78x r2: 3+1 after the box's
/// paragraph). Review rev78x2 finding 3 (`q4.tex` Figs 2-3); 2311.18531 A7.F5/F6. Repro
/// captions-floats/subfloats_in_a_box_share_a_row.
#[test]
fn subfloats_in_a_box_share_a_row() {
  let xml = convert_clean(repro!("captions-floats/subfloats_in_a_box_share_a_row.tex"));
  for n in 1..=2 {
    let id = format!("S0.F{n}");
    // the box's own paragraph, then the four sub-floats in one row
    assert_eq!(float_breaks(&xml, &id).len(), 1, "{id}: {xml}");
    assert_eq!(last_row_panels(&xml, &id), 4, "{id}: {xml}");
  }
}

/// `\hspace` between panels in a `{center}` directly in the float counts towards their row as `\hskip` does: two
/// `0.45\textwidth` graphics with `0.2\textwidth` between them are two rows either way (pdflatex). Red (78x r2: one row
/// with `\hspace`, the kernel's there). Review rev78x2 finding 6. Repro
/// captions-floats/an_hspace_in_a_center_counts_towards_its_row.
#[test]
fn an_hspace_in_a_center_counts_towards_its_row() {
  let xml = convert_clean(repro!(
    "captions-floats/an_hspace_in_a_center_counts_towards_its_row.tex"
  ));
  for n in 1..=2 {
    let id = format!("S0.F{n}");
    assert_eq!(float_breaks(&xml, &id).len(), 1, "{id}: {xml}");
    assert_eq!(float_panels(&xml, &id), 2, "{id}: {xml}");
  }
}

/// A float whose binding sets its own `\hsize` (a wrapfigure's width) keeps it as its panels' line: two
/// `0.25\textwidth` graphics in a `0.3\textwidth` wrapfigure are two rows (pdflatex). Red (78x r2: one row, against the
/// page's line). Witness 2609.34643 S3.SS3.fig1. Repro captions-floats/a_wrapfigure_keeps_its_own_line.
#[test]
fn a_wrapfigure_keeps_its_own_line() {
  let xml = convert_clean(repro!(
    "captions-floats/a_wrapfigure_keeps_its_own_line.tex"
  ));
  assert_eq!(float_breaks(&xml, "S0.F1").len(), 1, "{xml}");
  assert_eq!(float_panels(&xml, "S0.F1"), 2, "{xml}");
}

/// A class binding declares the page its options give (`declare_panel_page`, values from pdflatex's `\showthe` in a
/// float): IEEEtran `[conference,compsoc]` 243.911pt columns (IEEEtran.cls:1785-1787), `[draftcls,a4paper]` 220.484pt
/// (A4 less 2in), REVTeX `[preprint,twocolumn]` 229pt — two panels just wider than the column are two rows, two just
/// narrower one (pdflatex). Red (78x r2: one row each, its 252.945pt, 228.878pt and one-column 468pt). Review rev78x2
/// finding 4. Repros captions-floats/ieeetran_compsoc_conference_page, ieeetran_draftcls_a4paper_page,
/// revtex_preprint_twocolumn_page.
#[test]
fn class_pages_follow_their_options() {
  for xml in [
    convert_clean(repro!(
      "captions-floats/ieeetran_compsoc_conference_page.tex"
    )),
    convert_clean(repro!("captions-floats/ieeetran_draftcls_a4paper_page.tex")),
    convert_clean(repro!("captions-floats/revtex_preprint_twocolumn_page.tex")),
  ] {
    assert_eq!(float_breaks(&xml, "S0.F1").len(), 1, "{xml}");
    assert!(float_breaks(&xml, "S0.F2").is_empty(), "{xml}");
    for id in ["S0.F1", "S0.F2"] {
      assert_eq!(float_panels(&xml, id), 2, "{id}: {xml}");
    }
  }
}

/// A row that begins a paragraph keeps the glue stated before its first panel, as TeX keeps glue at a paragraph's start:
/// `\hspace*{-0.5cm}` before three `0.35\textwidth` graphics pulls them into one row; after a `\\` TeX discards a row's
/// leading `\hspace`, and the same three are 2+1, but keeps an `\hspace*` (pdflatex: one row; 1 | 2 | 1; 1 | 3). Red
/// (78x r2: 2+1, the leading glue dropped). Witness 2212.03198 Figs 6, 10-12, 15. Repro
/// captions-floats/glue_at_a_paragraph_start_counts.
#[test]
fn glue_at_a_paragraph_start_counts() {
  let xml = convert_clean(repro!(
    "captions-floats/glue_at_a_paragraph_start_counts.tex"
  ));
  assert!(float_breaks(&xml, "S0.F1").is_empty(), "{xml}");
  assert_eq!(float_panels(&xml, "S0.F1"), 3, "{xml}");
  assert_eq!(float_breaks(&xml, "S0.F2").len(), 2, "{xml}");
  assert_eq!(float_panels(&xml, "S0.F2"), 4, "{xml}");
  assert_eq!(last_row_panels(&xml, "S0.F2"), 1, "{xml}");
  // (an `\hspace*` after the `\\` is kept: its three graphics one row)
  assert_eq!(float_breaks(&xml, "S0.F3").len(), 1, "{xml}");
  assert_eq!(last_row_panels(&xml, "S0.F3"), 3, "{xml}");
}

/// A stated width the float cannot read again at its end — a macro of a group that has ended (`{\def\wa{…}…}`, a
/// `\newcommand` in a `{center}`), or one an outer meaning shadows (`\def\w` before the document, `{\def\w{…}…}` in the
/// float) — does not give its box's width again, so its float keeps the line before the rule: two `0.48\columnwidth`
/// graphics one row in REVTeX's column (pdflatex). An `\hbox`'s content is set at the enclosing line, not at the box's
/// width: two `\resizebox{4.5cm}{!}` of `0.5\linewidth` graphics are evaluated and two rows (pdflatex). Red (78x r3: two
/// rows in Figures 1-3, the registers' widths against the real column). Review rev78x3 finding 1 (`p1.tex` F1-F2,
/// `p8.tex` F1). Repro captions-floats/a_width_the_float_cannot_read_again_keeps_the_line_before_the_rule.
#[test]
fn a_width_the_float_cannot_read_again_keeps_the_line_before_the_rule() {
  let xml = convert_clean(repro!(
    "captions-floats/a_width_the_float_cannot_read_again_keeps_the_line_before_the_rule.tex"
  ));
  for (n, breaks) in [(1, 0), (2, 0), (3, 0), (4, 1)] {
    let id = format!("S0.F{n}");
    assert_eq!(float_breaks(&xml, &id).len(), breaks, "{id}: {xml}");
    assert_eq!(float_panels(&xml, &id), 2, "{id}: {xml}");
  }
}

/// A float whose stated widths cannot be evaluated at the real geometry (a `\newdimen` register assigned `\figw=3.8cm`,
/// an `\addtolength` register) is arranged as before the rule, the geometry's text width included (OXIDIZED_DESIGN #99):
/// four 3.8cm panels one row of `[a4paper,margin=1in]{geometry}`'s 452.97pt (pdflatex). Red (78x r3: 3+1 each, against
/// the 345pt `\hsize`). Review rev78x3 finding 2 (`p2.tex` F2-F4). Repro
/// captions-floats/an_unevaluable_width_keeps_the_geometry_line.
#[test]
fn an_unevaluable_width_keeps_the_geometry_line() {
  let xml = convert_clean(repro!(
    "captions-floats/an_unevaluable_width_keeps_the_geometry_line.tex"
  ));
  for n in 1..=3 {
    let id = format!("S0.F{n}");
    assert!(float_breaks(&xml, &id).is_empty(), "{id}: {xml}");
    assert_eq!(float_panels(&xml, &id), 4, "{id}: {xml}");
  }
}

/// Under `\centering`, `\raggedright` and in a `{center}`, `\\` is `\@centercr`, whose `\par` ends the paragraph: the
/// `\hspace{-0.05\columnwidth}` after it begins the next, and its row keeps the glue — three `0.34\columnwidth` graphics
/// fit the second row (pdflatex 1 | 3); after a justified `\\` the glue is discarded and the three do not fit (TeX's
/// overfull line, ours 2+1 as base). Red (78x r3: 1 | 2 | 1 each). Review rev78x3 finding 3 (`p7.tex` F1). Repro
/// captions-floats/an_hspace_after_a_centred_newline_begins_its_row.
#[test]
fn an_hspace_after_a_centred_newline_begins_its_row() {
  let xml = convert_clean(repro!(
    "captions-floats/an_hspace_after_a_centred_newline_begins_its_row.tex"
  ));
  for n in [1, 3, 4] {
    let id = format!("S0.F{n}");
    assert_eq!(float_breaks(&xml, &id).len(), 1, "{id}: {xml}");
    assert_eq!(last_row_panels(&xml, &id), 3, "{id}: {xml}");
  }
  assert_eq!(float_breaks(&xml, "S0.F2").len(), 2, "{xml}");
  assert_eq!(last_row_panels(&xml, "S0.F2"), 1, "{xml}");
}

/// REVTeX's floats are centred (`\appdef\@xfloat@prep{\appdef\@parboxrestore{\centering}}`), so a `\\` in one is
/// `\@centercr` and the `\hspace` after it begins the next row, which keeps it (pdflatex 1 | 3). Red (78x r3: 1 | 2 |
/// 1). Review rev78x3 finding 3 (`p1.tex` F10). Repro captions-floats/revtex_floats_are_centred.
#[test]
fn revtex_floats_are_centred() {
  let xml = convert_clean(repro!("captions-floats/revtex_floats_are_centred.tex"));
  assert_eq!(float_breaks(&xml, "S0.F1").len(), 1, "{xml}");
  assert_eq!(float_panels(&xml, "S0.F1"), 4, "{xml}");
  assert_eq!(last_row_panels(&xml, "S0.F1"), 3, "{xml}");
}

/// A sub-float body's stated glue counts towards its width, as glue between panels does (its spaces still count 0):
/// two `\subfigure{\hspace{-0.4cm}\includegraphics[width=175pt]{…}}` with `\hspace{0.2cm}` between them are one row of
/// 345pt, the same without the `\hspace{-0.4cm}` two (pdflatex). Red (int69b: two rows each, the body's glue left out).
/// Witness 1307.3388 Fig 3 (2+2 in the PDF). Repro captions-floats/a_sub_float_bodys_stated_glue_counts.
#[test]
fn a_sub_float_bodys_stated_glue_counts() {
  let xml = convert_clean(repro!(
    "captions-floats/a_sub_float_bodys_stated_glue_counts.tex"
  ));
  for (n, breaks) in [(1, 0), (2, 1)] {
    let id = format!("S0.F{n}");
    assert_eq!(float_breaks(&xml, &id).len(), breaks, "{id}: {xml}");
    assert_eq!(float_panels(&xml, &id), 2, "{id}: {xml}");
  }
}

/// latex.ltx:9256 `\newline` is `\@normalcr` in every context, a justified break: under `\centering` and in a
/// `{center}` the `\hspace{0.15\columnwidth}` after it is discarded at the line's start, and three `0.3\columnwidth`
/// graphics fit the second line (pdflatex 1 | 3); after a `\\` there (`\@centercr`, whose `\par` ends the paragraph)
/// the glue begins the next paragraph and its row keeps it (pdflatex 1 | 2 | 1). The markup is the author's. Red (78x
/// r4: Figures 1-2 1 | 2 | 1, the `\newline`'s break read by its alignment class). Review rev78x4 finding 1 (`n3.tex`
/// F4, `n7.tex` F1). Repro captions-floats/a_newline_is_a_justified_break.
#[test]
fn a_newline_is_a_justified_break() {
  let xml = convert_clean(repro!("captions-floats/a_newline_is_a_justified_break.tex"));
  for n in [1, 2] {
    let id = format!("S0.F{n}");
    assert_eq!(
      float_breaks(&xml, &id),
      ["<break class=\"ltx_centering\"/>"],
      "{id}: {xml}"
    );
    assert_eq!(float_panels(&xml, &id), 4, "{id}: {xml}");
    assert_eq!(last_row_panels(&xml, &id), 3, "{id}: {xml}");
  }
  assert_eq!(
    float_breaks(&xml, "S0.F3"),
    [
      "<break class=\"ltx_centering\"/>",
      "<break class=\"ltx_break\"/>"
    ],
    "{xml}"
  );
  assert_eq!(
    float_breaks(&xml, "S0.F4"),
    [
      "<break class=\"ltx_break\"/>",
      "<break class=\"ltx_break\"/>"
    ],
    "{xml}"
  );
  for id in ["S0.F3", "S0.F4"] {
    assert_eq!(float_panels(&xml, id), 4, "{id}: {xml}");
    assert_eq!(last_row_panels(&xml, id), 1, "{id}: {xml}");
  }
  assert!(!xml.contains("_normalcr"), "{xml}");
}

/// No AASTeX's floats are centred, where REVTeX's are: AASTeX 6.x and 7.x `\def\@xfloat@prep` again after revtex4-1,
/// without its `\centering` (aastex701.cls:8066), as emulateapj.cls:949 does, so a `\\` in them is `\@normalcr` and
/// the `\hspace` after it is discarded: three `0.3\columnwidth` graphics fit the second line (pdflatex 1 | 3, in
/// aastex701 and in emulateapj). Red (78x r4: 1 | 2 | 1, REVTeX's declaration inherited). Review rev78x4 finding 2
/// (`n5.tex`). Repro captions-floats/aastex_floats_are_not_centred.
#[test]
fn aastex_floats_are_not_centred() {
  let tex = repro!("captions-floats/aastex_floats_are_not_centred.tex");
  for class in ["aastex701", "emulateapj"] {
    let xml = convert_clean(&tex.replace("{aastex701}", &format!("{{{class}}}")));
    assert_eq!(float_breaks(&xml, "S0.F1"), ["<break/>"], "{class}: {xml}");
    assert_eq!(float_panels(&xml, "S0.F1"), 4, "{class}: {xml}");
    assert_eq!(last_row_panels(&xml, "S0.F1"), 3, "{class}: {xml}");
  }
}

/// subcaption's `\subfloat` gives its `{subfigure}` the binding's `{\columnwidth}`, which the source does not state
/// (`\lx@panel@unstated`, `STATED_LENGTHS_WITHHELD`): the float's rows are set against REVTeX's real 246pt column, two
/// 4.5cm sub-floats one each (pdflatex two lines). Red (78x r4: one row, the float unevaluable and set at the base's
/// line). Review rev78x4 finding 3 (`n6.tex` F1). Repro captions-floats/subfloats_of_a_stated_width_in_a_revtex_column.
#[test]
fn subfloats_of_a_stated_width_in_a_revtex_column() {
  let xml = convert_clean(repro!(
    "captions-floats/subfloats_of_a_stated_width_in_a_revtex_column.tex"
  ));
  assert_eq!(
    float_breaks(&xml, "S0.F1"),
    ["<break class=\"ltx_break\"/>"],
    "{xml}"
  );
  for n in [1, 2] {
    assert!(
      xml.contains(&format!(
        "<figure class=\"ltx_figure_panel\" inlist=\"lof\" panelwidth=\"0.52\" xml:id=\"S0.F1.sf{n}\">"
      )),
      "{xml}"
    );
  }
  assert_eq!(last_row_panels(&xml, "S0.F1"), 1, "{xml}");
}

/// captions-floats/an_hspace_star_opens_the_row_paragraph: `\hspace*{5pt}` (Fig 2: `\hskip 5pt`) first in a float's
/// `{center}`, then two `\parbox` panels holding a `\par` — the leading glue's row separator (`note_panel_separator`) is
/// a mark for `arrange_panels`, not the capture's content (`insert_block_as`): the parboxes stay inline blocks of one
/// row and each caption is its figure's (pdflatex one line per float). Red (int70a, 78x r5; the ws-procs961x669 binding
/// net's 1710.06503 Fig 1, 0 → 8 Errors): the separator chose the capture's container, the parboxes were hoisted out as
/// vertical `ltx:logical-block`s and the caption kept inside the row's `ltx:block`.
#[test]
fn an_hspace_star_opens_the_row_paragraph() {
  let xml = convert_clean(repro!(
    "captions-floats/an_hspace_star_opens_the_row_paragraph.tex"
  ));
  assert!(!xml.contains("<logical-block"), "{xml}");
  for (n, labels, width, text) in [
    (1, " labels=\"LABEL:ICHEP\"", "200.0pt", "Cap."),
    (2, "", "100.0pt", "Cap two."),
  ] {
    let panel = |m: u32, label: char| {
      format!(
        "<block class=\"ltx_parbox\" vattach=\"middle\" width=\"151.8pt\" xml:id=\"S0.F{n}.1.{m}\">\
         <rule height=\"10.0pt\" width=\"{width}\"/><p align=\"center\" xml:id=\"S0.F{n}.1.{m}.1\">\
         <text fontsize=\"80%\" xml:id=\"S0.F{n}.1.{m}.1.1\">({label})</text></p></block>"
      )
    };
    let first = if n == 1 { 'a' } else { 'c' };
    let second = if n == 1 { 'b' } else { 'd' };
    assert_element(
      &xml,
      "figure",
      &[&format!("xml:id=\"S0.F{n}\"")],
      &format!(
        "<figure inlist=\"lof\"{labels} placement=\"t\" xml:id=\"S0.F{n}\">{}<block xml:id=\"S0.F{n}.1\">{}{}</block>{}\
         </figure>",
        figure_tags(n),
        panel(1, first),
        panel(2, second),
        figure_captions(n, "", text)
      ),
    );
  }
}
