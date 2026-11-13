//! Red/green guards for batch 79c (grid design payload 3, the panel grid markup; user rulings 2026-10-22,
//! FIDELITY_CHARTER §2 "Float panels", grid study option (a)): a float `arrange_panels` set in rows says them —
//! `panelrows`, `panelcols` and `ltx_panels_gallery` or `ltx_panels_matrix` (a matrix only on strong signals: an
//! author-built `tabular` of panel-sized graphics, or 2 or more rows of the same columns and one panel width whose
//! sub-captions are none or exactly one whole row's) — and each panel its width as a fraction of its row's line
//! (`panelwidth`, none where the float could not take it at the real geometry) and, in a matrix, the columns it spans
//! (`panelspan`); where every panel states its width, the float its line as a fraction of the PDF's running-text column
//! (`panelline`) and its widest row's fill (`panelfill`; r2, user ruling 2026-10-23); and a `\label` makes no box
//! (TeX's `\label` is a `\write`). Round 2 answers review rev79c findings 1-5. Each guard converts its repro under
//! `tools/perfect_kernel/repros/captions-floats/` (pdflatex-checked: rows from the float boxes' `\showbox`), asserts
//! the diagnostics by count and the floats whole.
//! OXIDIZED_DESIGN_DIVERGENCES #887, KNOWN_PERL_ERRORS #1144.
use latexml::util::test::{assert_element, convert_with, rng_error_count};

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// Convert `tex`: 0 errors, 0 warnings; the XML with each graphic's `candidates` (a TeX tree path) left out.
fn convert_clean(tex: &str) -> String {
  let (log, xml) = convert_with(tex, Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let mut out = String::with_capacity(xml.len());
  let mut rest = xml.as_str();
  while let Some(at) = rest.find(" candidates=\"") {
    out.push_str(&rest[..at]);
    let value = &rest[at + " candidates=\"".len()..];
    rest = &value[value.find('"').map_or(value.len(), |end| end + 1)..];
  }
  out.push_str(rest);
  out
}

/// A float's tags: `Figure n`.
fn figure_tags(n: u32) -> String {
  format!(
    "<tags><tag>Figure {n}</tag><tag role=\"refnum\">{n}</tag><tag role=\"typerefnum\">Figure {n}</tag></tags>"
  )
}

/// Its captions, centred.
fn figure_captions(n: u32, text: &str) -> String {
  format!(
    "<toccaption class=\"ltx_centering\"><tag close=\" \">{n}</tag>{text}</toccaption><caption \
     class=\"ltx_centering\"><tag close=\": \">Figure {n}</tag>{text}</caption>"
  )
}

/// A `\rule` panel `width` wide and 1cm high, centred, stating `panel` (its grid attributes, or none).
fn rule_panel(width: &str, panel: &str) -> String {
  format!(
    "<rule align=\"center\" class=\"ltx_figure_panel\" height=\"28.5pt\"{panel} width=\"{width}\"/>"
  )
}

/// A centred `\\` between rows.
const CENTRED_BREAK: &str = "<break class=\"ltx_centering\"/>";

/// A gallery: four sub-captioned `{subfigure}{0.45\textwidth}` panels, two rows of two (pdflatex). Their row and
/// column are not what they mean — each has its own caption — so the float is `ltx_panels_gallery` though its rows
/// are uniform; each panel states its width, 0.45 of the line; the document is schema-valid. Red (int71a: no grid
/// markup). Repro captions-floats/panel_grid_markup (F1).
#[test]
fn a_gallery_states_its_rows_columns_and_panel_widths() {
  let xml = convert_clean(repro!("captions-floats/panel_grid_markup.tex"));
  let panel = |n: u32, letter: char, text: &str| {
    format!(
      "<figure align=\"center\" class=\"ltx_figure_panel\" inlist=\"lof\" panelwidth=\"0.45\" \
       xml:id=\"S0.F1.sf{n}\"><tags><tag><text fontsize=\"90%\" xml:id=\"S0.F1.sf{n}.1\">({letter})</text></tag><tag \
       role=\"refnum\">1{letter}</tag></tags><rule height=\"28.5pt\" width=\"155.3pt\"/><toccaption><tag close=\" \
       \">{letter}</tag>{text}</toccaption><caption><tag close=\" \"><text fontsize=\"90%\" \
       xml:id=\"S0.F1.sf{n}.2\">({letter})</text></tag><text fontsize=\"90%\" \
       xml:id=\"S0.F1.sf{n}.3\">{text}</text></caption></figure>"
    )
  };
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F1\""],
    &format!(
      "<figure class=\"ltx_panels_gallery\" inlist=\"lof\" panelcols=\"2\" panelfill=\"0.9\" panelline=\"1\" \
       panelrows=\"2\" xml:id=\"S0.F1\">{}{}{}<break class=\"ltx_break\"/>{}{}{}</figure>",
      figure_tags(1),
      panel(1, 'a', "A"),
      panel(2, 'b', "B"),
      panel(3, 'c', "C"),
      panel(4, 'd', "D"),
      figure_captions(1, "A gallery: sub-captioned panels, two rows of two.")
    ),
  );
  if let Some(errors) = rng_error_count(&xml) {
    assert_eq!(errors, 0, "{xml}");
  }
}

/// A matrix: three rows of three uncaptioned `0.3\textwidth` panels (pdflatex 3+3+3) are `ltx_panels_matrix`, 3 x 3,
/// each panel 0.3 of the line. Red (int71a: no grid markup). Repro captions-floats/panel_grid_markup (F2).
#[test]
fn equal_rows_of_equal_panels_are_a_matrix() {
  let xml = convert_clean(repro!("captions-floats/panel_grid_markup.tex"));
  let row = rule_panel("103.5pt", " panelwidth=\"0.3\"").repeat(3);
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F2\""],
    &format!(
      "<figure class=\"ltx_panels_matrix\" inlist=\"lof\" panelcols=\"3\" panelfill=\"0.9\" panelline=\"1\" \
       panelrows=\"3\" xml:id=\"S0.F2\">{}{row}{CENTRED_BREAK}{row}{CENTRED_BREAK}{row}{}</figure>",
      figure_tags(2),
      figure_captions(
        2,
        "A matrix: three rows of three equal panels, one caption."
      )
    ),
  );
}

/// Ragged rows: three `0.3\textwidth` panels, then two `0.45\textwidth` ones (pdflatex 3+2) are a gallery of 2 rows,
/// 3 columns (the most a row takes), each panel its own width. Red (int71a: no grid markup). Repro
/// captions-floats/panel_grid_markup (F3).
#[test]
fn ragged_rows_are_a_gallery() {
  let xml = convert_clean(repro!("captions-floats/panel_grid_markup.tex"));
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F3\""],
    &format!(
      "<figure class=\"ltx_panels_gallery\" inlist=\"lof\" panelcols=\"3\" panelfill=\"0.9\" panelline=\"1\" \
       panelrows=\"2\" xml:id=\"S0.F3\">{}{}{CENTRED_BREAK}{}{}</figure>",
      figure_tags(3),
      rule_panel("103.5pt", " panelwidth=\"0.3\"").repeat(3),
      rule_panel("155.3pt", " panelwidth=\"0.45\"").repeat(2),
      figure_captions(3, "Ragged rows: three, then two wider.")
    ),
  );
}

/// A matrix row may hold a panel a whole number of its columns wide: two rows of three `0.3\textwidth` panels, then a
/// `0.6\textwidth` and a `0.3\textwidth` one (pdflatex 3+3+2), are a 3 x 3 matrix whose wide panel spans 2 columns
/// (`panelspan`). Red (int71a: no grid markup). Repro captions-floats/panel_grid_markup (F4).
#[test]
fn a_matrix_panel_spans_whole_columns() {
  let xml = convert_clean(repro!("captions-floats/panel_grid_markup.tex"));
  let row = rule_panel("103.5pt", " panelwidth=\"0.3\"").repeat(3);
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F4\""],
    &format!(
      "<figure class=\"ltx_panels_matrix\" inlist=\"lof\" panelcols=\"3\" panelfill=\"0.9\" panelline=\"1\" \
       panelrows=\"3\" xml:id=\"S0.F4\">{}{row}{CENTRED_BREAK}{row}{CENTRED_BREAK}{}{}{}</figure>",
      figure_tags(4),
      rule_panel("207.0pt", " panelspan=\"2\" panelwidth=\"0.6\""),
      rule_panel("103.5pt", " panelwidth=\"0.3\""),
      figure_captions(4, "A matrix whose last row holds a panel two columns wide.")
    ),
  );
}

/// An author-built `tabular` of graphics (a header row, then 2 x 2 graphics; pdflatex one box) is a matrix of its
/// rows and columns — the table keeps its cells as they are, no panel states a width — and a float of one panel
/// states no grid at all. Red (int71a: no grid markup on the tabular's float). Witness 1403.6637 S4.F6. Repro
/// captions-floats/panel_grid_markup (F5, F6).
#[test]
fn a_tabular_of_panels_is_a_matrix_and_one_panel_no_grid() {
  let xml = convert_clean(repro!("captions-floats/panel_grid_markup.tex"));
  let graphic = |row: u32, cell: u32, n: u32| {
    format!(
      "<td align=\"center\" xml:id=\"S0.F5.1.{row}.{cell}\"><graphics graphic=\"example-image\" \
       options=\"width=137.9979pt,keepaspectratio=true\" xml:id=\"S0.F5.g{n}\"/></td>"
    )
  };
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F5\""],
    &format!(
      "<figure class=\"ltx_panels_matrix\" inlist=\"lof\" panelcols=\"2\" panelrows=\"3\" xml:id=\"S0.F5\">{}<tabular \
       class=\"ltx_centering ltx_guessed_headers\" vattach=\"middle\" xml:id=\"S0.F5.1\"><tbody><tr \
       xml:id=\"S0.F5.1.1\"><td xml:id=\"S0.F5.1.1.1\"/><td align=\"center\" thead=\"column\" \
       xml:id=\"S0.F5.1.1.2\"><text font=\"bold\" xml:id=\"S0.F5.1.1.2.1\">after</text></td></tr><tr \
       xml:id=\"S0.F5.1.2\">{}{}</tr><tr xml:id=\"S0.F5.1.3\">{}{}</tr></tbody></tabular>{}</figure>",
      figure_tags(5),
      graphic(2, 1, 1),
      graphic(2, 2, 2),
      graphic(3, 1, 3),
      graphic(3, 2, 4),
      figure_captions(5, "A matrix the author built as a tabular.")
    ),
  );
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F6\""],
    &format!(
      "<figure inlist=\"lof\" xml:id=\"S0.F6\">{}<graphics class=\"ltx_centering\" graphic=\"example-image\" \
       options=\"width=172.5pt,keepaspectratio=true\" xml:id=\"S0.F6.g1\"/>{}</figure>",
      figure_tags(6),
      figure_captions(6, "One panel.")
    ),
  );
}

/// A width the float cannot take at the real geometry is never guessed: two `width=\figw` panels, `\figw` assigned
/// without `\setlength` in a two-column document (its value the registers' geometry's), are a 1 x 2 gallery whose
/// panels state no `panelwidth`; the same two panels stating `0.45\columnwidth` state 0.45 of the 229.5pt column
/// (pdflatex: one line each). Red (int71a: no grid markup). Repro captions-floats/panel_grid_markup_of_an_unevaluable_width.
#[test]
fn an_unevaluable_width_states_no_panel_width() {
  let xml = convert_clean(repro!(
    "captions-floats/panel_grid_markup_of_an_unevaluable_width.tex"
  ));
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F1\""],
    &format!(
      "<figure class=\"ltx_panels_gallery\" inlist=\"lof\" panelcols=\"2\" panelrows=\"1\" \
       xml:id=\"S0.F1\">{}{}<toccaption class=\"ltx_centering\"><tag close=\" \">1</tag>Two panels whose width is a \
       register the document assigned without <text font=\"typewriter\" \
       xml:id=\"S0.F1.1\">\\setlength</text>.</toccaption><caption class=\"ltx_centering\"><tag close=\": \">Figure \
       1</tag>Two panels whose width is a register the document assigned without <text font=\"typewriter\" \
       xml:id=\"S0.F1.2\">\\setlength</text>.</caption></figure>",
      figure_tags(1),
      rule_panel("155.3pt", "").repeat(2),
    ),
  );
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F2\""],
    &format!(
      "<figure class=\"ltx_panels_gallery\" inlist=\"lof\" panelcols=\"2\" panelfill=\"0.9\" panelline=\"1\" \
       panelrows=\"1\" xml:id=\"S0.F2\">{}{}{}</figure>",
      figure_tags(2),
      rule_panel("155.3pt", " panelwidth=\"0.45\"").repeat(2),
      figure_captions(2, "The same two panels, their width stated.")
    ),
  );
}

/// A `\label` makes no box (TeX's is a `\write`): four `\subfigure[]{\fbox{\label{…}\includegraphics[width=0.23
/// \textwidth]{…}}}` in REVTeX's 246pt column, `\fboxsep` 0.3mm, are two rows of two 119.8pt boxes (pdflatex). Red
/// (int71a: one per row, each `\fbox` measured with its label's key text, as Perl's `\lx@label`, Whatsit.pm:253-265).
/// Witness 0809.3275 Figs 2-6. Repro captions-floats/a_label_makes_no_box.
#[test]
fn a_label_makes_no_box() {
  let xml = convert_clean(repro!("captions-floats/a_label_makes_no_box.tex"));
  let panel = |n: u32, letter: char, label: &str| {
    format!(
      "<figure class=\"ltx_figure_panel\" labels=\"LABEL:{label}\" panelwidth=\"0.488\" xml:id=\"S0.F1.sf{n}\"><tags><tag><text \
       fontsize=\"80%\" xml:id=\"S0.F1.sf{n}.1\">({letter})</text></tag><tag role=\"autoref\">1({letter})<text \
       xml:id=\"S0.F1.sf{n}.2\"/></tag><tag role=\"refnum\">1({letter})</tag></tags><p xml:id=\"S0.F1.sf{n}.3\"><text \
       cssstyle=\"padding:0.9pt\" framecolor=\"#000000\" framed=\"rectangle\" xml:id=\"S0.F1.sf{n}.3.1\"><graphics \
       graphic=\"example-image\" options=\"width=79.34853pt,keepaspectratio=true\" \
       xml:id=\"S0.F1.sf{n}.g1\"/></text></p><toccaption><tag close=\" \">({letter})</tag></toccaption><caption><tag \
       close=\" \"><text fontsize=\"80%\" xml:id=\"S0.F1.sf{n}.4\">({letter})</text></tag></caption></figure>"
    )
  };
  let text = "Panels <ref labelref=\"LABEL:fig:110raw-base\"/> and <ref labelref=\"LABEL:fig:110teApp1-1-pos\"/>.";
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F1\""],
    &format!(
      "<figure class=\"ltx_panels_gallery\" inlist=\"lof\" labels=\"LABEL:fig:mutualK1\" panelcols=\"2\" \
       panelfill=\"0.976\" panelline=\"1\" panelrows=\"2\" xml:id=\"S0.F1\"><tags><tag>Figure 1</tag><tag role=\"autoref\">Figure\u{a0}1<text \
       xml:id=\"S0.F1.1\"/></tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Figure \
       1</tag></tags>{}{}<break class=\"ltx_break\"/>{}{}<toccaption><tag close=\" \">1</tag>{text}</toccaption><caption><tag \
       close=\": \">Figure 1</tag>{text}</caption></figure>",
      panel(1, 'a', "fig:110raw-base"),
      panel(2, 'b', "fig:110mi1-pos"),
      panel(3, 'c', "fig:110teComp1-1"),
      panel(4, 'd', "fig:110teApp1-1-pos"),
    ),
  );
}

/// A sub-float holding nothing but its number — `\subfloat{\label{…}}`, a label's anchor — is no panel: three such
/// `\labelphantom`s before the one graphic that draws every panel are one line, the graphic (pdflatex). They keep
/// their numbers and labels (`\ref{fig:setupc}` is 1c) and take no place in a row; the float has one panel, so no
/// grid. Red (int71a: the 2nd and 3rd panels beside the graphic; Perl: each a row of its own). Witness 2208.08007
/// Figs 1, 2, S2, S5. Repro captions-floats/a_label_only_sub_float_is_no_panel.
#[test]
fn a_label_only_sub_float_is_no_panel() {
  let xml = convert_clean(repro!(
    "captions-floats/a_label_only_sub_float_is_no_panel.tex"
  ));
  let anchor = |n: u32, letter: char| {
    format!(
      "<figure align=\"center\" labels=\"LABEL:fig:setup{letter}\" xml:id=\"S0.F1.sf{n}\"><tags><tag>({letter})</tag><tag \
       role=\"autoref\">1{letter}<text xml:id=\"S0.F1.sf{n}.1\"/></tag><tag role=\"refnum\">1{letter}</tag></tags></figure>"
    )
  };
  let text = "A figure whose panels are drawn in its one graphic.";
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F1\""],
    &format!(
      "<figure inlist=\"lof\" placement=\"htb!\" xml:id=\"S0.F1\"><tags><tag>Figure 1</tag><tag role=\"autoref\">Figure\u{a0}1<text xml:id=\"S0.F1.1\"/></tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Figure \
       1</tag></tags>{}{}{}<graphics class=\"ltx_centering\" graphic=\"example-image\" \
       options=\"width=345.0pt,keepaspectratio=true\" xml:id=\"S0.F1.g1\"/>{}</figure>",
      anchor(1, 'a'),
      anchor(2, 'b'),
      anchor(3, 'c'),
      figure_captions(1, text)
    ),
  );
}

/// A graphic panel of a figure: `example-image` at `width`, stating `attrs` (its classes and grid attributes).
fn graphic_panel(id: &str, width: &str, attrs: &str) -> String {
  format!(
    "<graphics{attrs} graphic=\"example-image\" options=\"width={width},keepaspectratio=true\" xml:id=\"{id}\"/>"
  )
}

/// A `{subfigure}` panel of `float` (number `n`), its `k`-th, `letter`, holding `example-image` at `width` and its
/// sub-caption `text`, stating `panelwidth`.
fn sub_figure_panel(
  n: u32,
  k: u32,
  letter: char,
  width: &str,
  panelwidth: &str,
  text: &str,
) -> String {
  format!(
    "<figure align=\"center\" class=\"ltx_figure_panel\" inlist=\"lof\" panelwidth=\"{panelwidth}\" \
     xml:id=\"S0.F{n}.sf{k}\"><tags><tag><text fontsize=\"90%\" xml:id=\"S0.F{n}.sf{k}.1\">({letter})</text></tag><tag \
     role=\"refnum\">{n}{letter}</tag></tags>{}<toccaption><tag close=\" \">{letter}</tag>{text}</toccaption><caption><tag \
     close=\" \"><text fontsize=\"90%\" xml:id=\"S0.F{n}.sf{k}.2\">({letter})</text></tag><text fontsize=\"90%\" \
     xml:id=\"S0.F{n}.sf{k}.3\">{text}</text></caption></figure>",
    graphic_panel(&format!("S0.F{n}.sf{k}.g1"), width, "")
  )
}

/// A data table is no matrix: a `tabular` of text whose two cells hold an 8pt-high icon states no grid (an icon is no
/// panel: a graphic of a panel's size is 24pt or more, `is_panel_graphic`); a `tabular` of four 72pt graphics stays a
/// 2 x 2 matrix. Red (79c r1: the data table `ltx_panels_matrix` 3 x 3). Review rev79c finding 1; witnesses
/// 2608.05898 S4.T2, 2411.16612 S7.T1/T2, 2501.18280 S4.T3, 2506.23665 Ch3.T4, 1509.07152 S10.T23 (no grid); 1403.6637
/// S4.F6/F8, 2008.05524 T11-16, 0908.0534, 1104.5469 (matrices). Repro captions-floats/panel_grid_matrix_signals (T1, F1).
#[test]
fn a_data_table_is_no_matrix_and_a_tabular_of_panels_is() {
  let xml = convert_clean(repro!("captions-floats/panel_grid_matrix_signals.tex"));
  let icon = |n: u32| {
    format!(
      "<graphics graphic=\"example-image\" options=\"height=8.0pt,keepaspectratio=true\" xml:id=\"S0.T1.g{n}\"/>"
    )
  };
  let cell = |row: u32, col: u32, align: &str, body: &str| {
    format!(
      "<td align=\"{align}\"{} xml:id=\"S0.T1.1.{row}.{col}\">{body}</td>",
      if col == 1 { " thead=\"row\"" } else { "" }
    )
  };
  let text = "A data table with two small icons.";
  assert_element(
    &xml,
    "table",
    &["xml:id=\"S0.T1\""],
    &format!(
      "<table inlist=\"lot\" xml:id=\"S0.T1\"><tags><tag>Table 1</tag><tag role=\"refnum\">1</tag><tag \
       role=\"typerefnum\">Table 1</tag></tags><toccaption class=\"ltx_centering\"><tag close=\" \">1</tag>{text}</toccaption><caption \
       class=\"ltx_centering\"><tag close=\": \">Table 1</tag>{text}</caption><tabular class=\"ltx_centering \
       ltx_guessed_headers\" vattach=\"middle\" xml:id=\"S0.T1.1\"><tbody><tr xml:id=\"S0.T1.1.1\">{}{}{}</tr><tr \
       xml:id=\"S0.T1.1.2\">{}{}{}</tr><tr xml:id=\"S0.T1.1.3\">{}{}{}</tr></tbody></tabular></table>",
      cell(1, 1, "left", "Tool"),
      cell(1, 2, "center", "Correct"),
      cell(1, 3, "center", "Time"),
      cell(2, 1, "left", "A"),
      cell(2, 2, "center", &icon(1)),
      cell(2, 3, "center", "1.2"),
      cell(3, 1, "left", "B"),
      cell(3, 2, "center", "17"),
      cell(3, 3, "center", &icon(2)),
    ),
  );
  let graphic = |row: u32, col: u32, n: u32| {
    format!(
      "<td align=\"center\" xml:id=\"S0.F1.1.{row}.{col}\">{}</td>",
      graphic_panel(&format!("S0.F1.g{n}"), "72.0pt", "")
    )
  };
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F1\""],
    &format!(
      "<figure class=\"ltx_panels_matrix\" inlist=\"lof\" panelcols=\"2\" panelrows=\"2\" xml:id=\"S0.F1\">{}<tabular \
       class=\"ltx_centering\" vattach=\"middle\" xml:id=\"S0.F1.1\"><tbody><tr xml:id=\"S0.F1.1.1\">{}{}</tr><tr \
       xml:id=\"S0.F1.1.2\">{}{}</tr></tbody></tabular>{}</figure>",
      figure_tags(1),
      graphic(1, 1, 1),
      graphic(1, 2, 2),
      graphic(2, 1, 3),
      graphic(2, 2, 4),
      figure_captions(1, "A tabular of four 72pt graphics.")
    ),
  );
}

/// Sub-captions on exactly one whole row label its columns (main-session decision 3, 2026-10-23): a row of three
/// `0.3\textwidth` graphics over a row of three `{subfigure}{0.3\textwidth}`s, each sub-captioned (pdflatex 3+3), is a
/// 2 x 3 matrix. Red: none (79c r1 a matrix too: not every panel captioned); the guard pins the rule's other side.
/// Review rev79c finding 2; witnesses 2209.01559 Figs 3, 5; 1404.0431 Fig 1; 2212.03198 Figs 7, 10, 13, 14;
/// 1811.02385 S4.F3-F6. Repro captions-floats/panel_grid_matrix_signals (F2).
#[test]
fn captions_on_one_whole_row_label_a_matrixs_columns() {
  let xml = convert_clean(repro!("captions-floats/panel_grid_matrix_signals.tex"));
  let width = "103.50105pt";
  let top = |n: u32| {
    graphic_panel(
      &format!("S0.F2.g{n}"),
      width,
      " class=\"ltx_centering ltx_figure_panel\" panelwidth=\"0.3\"",
    )
  };
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F2\""],
    &format!(
      "<figure class=\"ltx_panels_matrix\" inlist=\"lof\" panelcols=\"3\" panelfill=\"0.9\" panelline=\"1\" \
       panelrows=\"2\" xml:id=\"S0.F2\">{}{}{}{}{CENTRED_BREAK}{}{}{}{}</figure>",
      figure_tags(2),
      top(1),
      top(2),
      top(3),
      sub_figure_panel(2, 1, 'a', width, "0.3", "First"),
      sub_figure_panel(2, 2, 'b', width, "0.3", "Second"),
      sub_figure_panel(2, 3, 'c', width, "0.3", "Third"),
      figure_captions(
        2,
        "Two rows of three, the sub-captions of the second label the columns."
      )
    ),
  );
}

/// Sub-captions on some panels of a row make a gallery: (a) (b) over (c) and an uncaptioned legend, each
/// `0.45\textwidth` (pdflatex 2+2), are independent panels, a 2 x 2 gallery though their rows are uniform. Red (79c
/// r1: `ltx_panels_matrix`, its veto only when every panel is captioned). Review rev79c finding 2; witnesses
/// 2503.15158 S4.F7, 2404.12123 S3.F6, 2106.04682 A2.F4, 1808.10020 chapter5.figure11, 2602.17978 Ch4.F2/F3. Every
/// panel captioned: `a_gallery_states_its_rows_columns_and_panel_widths`. Repro captions-floats/panel_grid_matrix_signals (F3).
#[test]
fn captions_on_part_of_a_row_make_a_gallery() {
  let xml = convert_clean(repro!("captions-floats/panel_grid_matrix_signals.tex"));
  let width = "155.24895pt";
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F3\""],
    &format!(
      "<figure class=\"ltx_panels_gallery\" inlist=\"lof\" panelcols=\"2\" panelfill=\"0.9\" panelline=\"1\" \
       panelrows=\"2\" xml:id=\"S0.F3\">{}{}{}{CENTRED_BREAK}{}{}{}</figure>",
      figure_tags(3),
      sub_figure_panel(3, 1, 'a', width, "0.45", "A"),
      sub_figure_panel(3, 2, 'b', width, "0.45", "B"),
      sub_figure_panel(3, 3, 'c', width, "0.45", "C"),
      graphic_panel(
        "S0.F3.g1",
        width,
        " class=\"ltx_centering ltx_figure_panel\" panelwidth=\"0.45\""
      ),
      figure_captions(3, "Three sub-captioned panels and an uncaptioned legend.")
    ),
  );
}

/// A float's line as a fraction of the PDF's running-text column (`panelline`) and its widest row's fill
/// (`panelfill`; user ruling 2026-10-23): in REVTeX's two columns (`\textwidth` 510pt, `\columnwidth` 246pt) a column
/// `figure` of two `0.45\columnwidth` panels is line 1, fill 0.9; a `figure*` of two `0.45\textwidth` panels line 2.073
/// (510/246), fill 0.9. A one-column `figure` is line 1 (`a_gallery_states_its_rows_columns_and_panel_widths`), a
/// wrapfigure under 1 (`a_wrapfigure_line_is_under_one_column`); the document is schema-valid (`panelline`,
/// `panelfill`, `panelwidth` typed `xsd:decimal`). Red (79c r1: neither attribute). Repro
/// captions-floats/panel_grid_line_and_fill.
#[test]
fn a_float_states_its_line_of_the_column_and_its_fill() {
  let xml = convert_clean(repro!("captions-floats/panel_grid_line_and_fill.tex"));
  for (n, line, text) in [
    (1, "1", "A column figure of two panels."),
    (2, "2.073", "A page-wide figure of two panels."),
  ] {
    assert_element(
      &xml,
      "figure",
      &[&format!("xml:id=\"S0.F{n}\"")],
      &format!(
        "<figure class=\"ltx_panels_gallery\" inlist=\"lof\" panelcols=\"2\" panelfill=\"0.9\" panelline=\"{line}\" \
         panelrows=\"1\" xml:id=\"S0.F{n}\"><tags><tag>Figure {n}</tag><tag role=\"autoref\">Figure\u{a0}{n}<text \
         xml:id=\"S0.F{n}.1\"/></tag><tag role=\"refnum\">{n}</tag><tag role=\"typerefnum\">Figure \
         {n}</tag></tags>{}{}</figure>",
        rule_panel("155.3pt", " panelwidth=\"0.45\"").repeat(2),
        figure_captions(n, text)
      ),
    );
  }
  // (the grid's numbers are the schema's decimals)
  if let Some(errors) = rng_error_count(&xml) {
    assert_eq!(errors, 0, "{xml}");
  }
}

/// A wrapfigure's line is its own width: a `{wrapfigure}{r}{0.5\textwidth}` of two `0.45\linewidth` panels (pdflatex
/// one line) is line 0.5 of the one-column page, fill 0.9. Red (79c r1: neither attribute). Repro
/// captions-floats/panel_grid_markup (F7).
#[test]
fn a_wrapfigure_line_is_under_one_column() {
  let xml = convert_clean(repro!("captions-floats/panel_grid_markup.tex"));
  let text = "A wrapped figure of two panels.";
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F7\""],
    &format!(
      "<figure class=\"ltx_panels_gallery\" float=\"right\" inlist=\"lof\" panelcols=\"2\" panelfill=\"0.9\" \
       panelline=\"0.5\" panelrows=\"1\" width=\"50%\" xml:id=\"S0.F7\">{}{}{}</figure>",
      figure_tags(7),
      rule_panel("77.6pt", " panelwidth=\"0.45\"").repeat(2),
      figure_captions(7, text)
    ),
  );
}

/// An inner float beside content of no width leaves its panels no width of its line: a `{subfigure}{0.6\textwidth}`
/// of two `0.45\linewidth` rules beside an empty `\makebox[0pt]` or `\raisebox` collapses into its float, whose rules
/// then state no `panelwidth` (0.45 was of the sub-figure's line, 0.27 of the float's). Red (79c r1: 0.45 kept, the
/// strip only for an inner that was a panel of its float). Review rev79c finding 3. Repro
/// captions-floats/panel_grid_collapse_drops_inner_widths.
#[test]
fn a_collapsed_inner_beside_empty_content_leaves_no_widths() {
  let xml = convert_clean(repro!(
    "captions-floats/panel_grid_collapse_drops_inner_widths.tex"
  ));
  let rule = "<rule class=\"ltx_figure_panel\" height=\"28.5pt\" width=\"93.2pt\"/>";
  for (n, empty, text) in [
    (
      1,
      "<text align=\"center\" width=\"0.0pt\" xml:id=\"S0.F1.1.1\"/>",
      "An inner sub-figure beside an empty box.",
    ),
    (
      2,
      "<text xml:id=\"S0.F2.1.1\" yoffset=\"0.0pt\"/>",
      "An inner sub-figure beside an empty raised box.",
    ),
  ] {
    assert_element(
      &xml,
      "figure",
      &[&format!("xml:id=\"S0.F{n}\"")],
      &format!(
        "<figure align=\"center\" class=\"ltx_panels_gallery\" inlist=\"lof\" panelcols=\"2\" panelrows=\"1\" \
         xml:id=\"S0.F{n}\">{}{rule}{rule}<break class=\"ltx_break\"/><p align=\"center\" \
         xml:id=\"S0.F{n}.1\">{empty}</p>{}</figure>",
        figure_tags(n),
        figure_captions(n, text)
      ),
    );
  }
}
