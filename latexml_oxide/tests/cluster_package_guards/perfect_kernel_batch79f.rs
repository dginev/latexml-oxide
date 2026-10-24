//! Red/green guards for batch 79f (the panel page model; user rulings 2026-10-22/23, FIDELITY_CHARTER §2 "Float
//! panels": a float's rows are set against the class's real line, its sizes fractions of the PDF's running-text column
//! at the float): the two-column classes declare their pages (`declare_panel_page`) — article, report and book under
//! `twocolumn` (their size files' page), acmart's two-column formats, elsarticle's `5p` and two-column `3p`/`preprint`,
//! emulateapj, mn2e and mn, svjour3 `twocolumn`, svjour `epj`, ieeeconf, REVTeX 3, AASTeX 5.x, and iopart's one
//! column; `\onecolumn`/`\twocolumn` set the declared page's column setting; a declared page's separation holds only
//! while `\columnsep` is the binding's; a float held in a minipage is a fraction of the page outside it; and subfig's
//! sub-float is its body's width, the glue the body states counted; a `\parbox` reads the width its source states,
//! outside the box. Each guard converts its repro under
//! `tools/perfect_kernel/repros/captions-floats/` (pdflatex-checked: `\the\columnwidth`/`\the\textwidth` in each
//! float), asserts the diagnostics by count and the floats whole where the markup is the class's own.
//! OXIDIZED_DESIGN_DIVERGENCES #885, #887.
use latexml::util::test::{assert_element, convert_with, xml_element};

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

/// A grid attribute's value as `annotate_panel_grid` writes it: 3 decimals, trailing zeros dropped.
fn decimal(value: f64) -> String {
  let text = format!("{value:.3}");
  text.trim_end_matches('0').trim_end_matches('.').to_string()
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

/// A centred `\rule` panel `width` wide and 1cm high, stating `panelwidth`.
fn rule_panel(width: &str, panelwidth: &str) -> String {
  format!(
    "<rule align=\"center\" class=\"ltx_figure_panel\" height=\"28.5pt\" panelwidth=\"{panelwidth}\" \
     width=\"{width}\"/>"
  )
}

/// A float's opening tag of a gallery: its rows, columns, line and fill.
fn gallery(id: &str, rows: u32, cols: u32, line: &str, fill: &str) -> String {
  format!(
    "<figure class=\"ltx_panels_gallery\" inlist=\"lof\" panelcols=\"{cols}\" panelfill=\"{fill}\" \
     panelline=\"{line}\" panelrows=\"{rows}\" xml:id=\"{id}\">"
  )
}

/// Each class's page as pdflatex gives it (`\the\columnwidth`, `\the\textwidth` in a float; the class's own file, TL
/// 2025's or the copy papers ship): the class line put in the repro's place, the column and the text width in points.
/// The figure of two 100pt rules states 100pt of the column, the `figure*` of the text, and its line the text over the
/// column. Red (int75a: every class the registers' 345pt line, a two-column one 345pt as the column and 690pt as the
/// text, `figure*` `panelline` 2, or 1 where the binding keeps `\if@twocolumn` false — acmart, elsarticle `5p`,
/// emulateapj, mn2e; ieeeconf IEEEtran's 252/516pt). Witnesses: review rev79c2 finding 1 (acmart `[sigconf]` 30 papers /
/// 90 floats, article `[twocolumn]` 21 / 84, of popint70a), 2109.08525 (iopart). Repro
/// captions-floats/declared_class_pages.
#[test]
fn each_class_declares_its_page() {
  const PAGES: &[(&str, f64, f64)] = &[
    (r"\documentclass[twocolumn]{article}", 229.5, 469.0),
    (r"\documentclass[a4paper,twocolumn]{article}", 221.0, 452.0),
    (
      r"\documentclass[12pt,a4paper,twocolumn]{article}",
      221.0,
      452.0,
    ),
    (
      r"\documentclass[landscape,twocolumn]{article}",
      320.0,
      650.0,
    ),
    (r"\documentclass[twocolumn]{report}", 229.5, 469.0),
    (r"\documentclass[twocolumn]{book}", 229.5, 469.0),
    (r"\documentstyle[twocolumn]{article}", 200.0, 410.0),
    (r"\documentclass[sigconf]{acmart}", 241.14749, 506.295),
    (r"\documentclass[sigplan]{acmart}", 240.94499, 505.89),
    (r"\documentclass[acmtog]{acmart}", 243.14749, 510.295),
    (r"\documentclass[5p]{elsarticle}", 252.0, 522.0),
    (r"\documentclass[3p,twocolumn]{elsarticle}", 222.0, 468.0),
    (
      r"\documentclass[preprint,twocolumn]{elsarticle}",
      229.5,
      469.0,
    ),
    (r"\documentclass{emulateapj}", 245.26653, 513.11743),
    (
      r"\documentclass[onecolumn]{emulateapj}",
      513.11743,
      513.11743,
    ),
    (r"\documentclass{mn2e}", 240.0, 504.0),
    (r"\documentclass[onecolumn]{mn2e}", 504.0, 504.0),
    (r"\documentclass{mn}", 240.0, 504.0),
    (r"\documentclass[twocolumn]{svjour3}", 238.96417, 495.0),
    (r"\documentclass[epj]{svjour}", 250.30945, 512.0),
    (r"\documentclass{iopart}", 372.0, 372.0),
    (r"\documentclass[12pt]{iopart}", 446.39996, 446.39996),
    (
      r"\documentclass[conference,a4paper]{ieeeconf}",
      245.71811,
      505.89,
    ),
    (r"\documentclass[aps,twocolumn]{revtex}", 246.0, 510.0),
    (
      r"\documentstyle[aps,preprint,twocolumn]{revtex}",
      229.0,
      468.0,
    ),
    (r"\documentclass[preprint2]{aastex}", 216.81, 469.75499),
    (r"\documentclass[preprint]{aastex}", 469.75499, 469.75499),
  ];
  let template = repro!("captions-floats/declared_class_pages.tex");
  for (class, column, text) in PAGES {
    let xml = convert_clean(&template.replace(r"\documentclass[twocolumn]{article}", class));
    // (`S0.F1`, or a class with chapters' `Ch0.F1`)
    for (id, line) in [(".F1", column), (".F2", text)] {
      let figure = xml_element(&xml, "figure", &[&format!("{id}\"")])
        .unwrap_or_else(|| panic!("{class}: no {id}\n{xml}"));
      let start = &figure[..=figure.find('>').unwrap()];
      assert!(
        start.contains(&format!(" panelline=\"{}\"", decimal(line / column))),
        "{class} {id}: line {line} of the {column}pt column\n{figure}"
      );
      assert_eq!(
        figure
          .matches(&format!(" panelwidth=\"{}\"", decimal(100.0 / line)))
          .count(),
        2,
        "{class} {id}: two 100pt panels of the {line}pt line\n{figure}"
      );
    }
  }
}

/// article `[twocolumn]` sets its rows against its size file's page (`standard_class_twocolumn_page`): two 120pt
/// panels and 4pt of glue are two rows of the 229.5pt column, two 200pt ones one row of the 469pt text, its `figure*`
/// 2.044 of the column (pdflatex). Red (int75a: one row each, the registers' 345pt line as the column and twice it as
/// the text, `panelline` 2). Repro captions-floats/standard_class_twocolumn_rows.
#[test]
fn a_twocolumn_article_sets_its_rows_against_its_page() {
  let xml = convert_clean(repro!("captions-floats/standard_class_twocolumn_rows.tex"));
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F1\""],
    &format!(
      "{}{}{}<break class=\"ltx_break\"/>{}{}</figure>",
      gallery("S0.F1", 2, 1, "1", "0.523"),
      figure_tags(1),
      rule_panel("120.0pt", "0.523"),
      rule_panel("120.0pt", "0.523"),
      figure_captions(1, "Two 120pt panels in a column.")
    ),
  );
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F2\""],
    &format!(
      "{}{}{}{}{}</figure>",
      gallery("S0.F2", 1, 2, "2.044", "0.852"),
      figure_tags(2),
      rule_panel("200.0pt", "0.426"),
      rule_panel("200.0pt", "0.426"),
      figure_captions(2, "Two 200pt panels across the page.")
    ),
  );
}

/// `\onecolumn` in a `[twocolumn]` article sets its floats against the text (latex.ltx:20430-20437
/// `\columnwidth\textwidth`): the figure after it is one row of two 120pt panels, 0.256 of the 469pt line, which is
/// the running-text column there (pdflatex). Red (int75a: `panelwidth` 0.348 of the registers' 345pt). Repro
/// captions-floats/onecolumn_in_a_twocolumn_article.
#[test]
fn onecolumn_sets_the_declared_page_in_one_column() {
  let xml = convert_clean(repro!(
    "captions-floats/onecolumn_in_a_twocolumn_article.tex"
  ));
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F1\""],
    &format!(
      "{}{}{}<break class=\"ltx_break\"/>{}{}</figure>",
      gallery("S0.F1", 2, 1, "1", "0.523"),
      figure_tags(1),
      rule_panel("120.0pt", "0.523"),
      rule_panel("120.0pt", "0.523"),
      figure_captions(1, "Two 120pt panels in a column.")
    ),
  );
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F2\""],
    &format!(
      "{}{}{}{}{}</figure>",
      gallery("S0.F2", 1, 2, "1", "0.512"),
      figure_tags(2),
      rule_panel("120.0pt", "0.256"),
      rule_panel("120.0pt", "0.256"),
      figure_captions(2, "Two 120pt panels in one column.")
    ),
  );
}

/// A style that sets its own page over a declared one (2008.06630's 3dv.sty over `[twocolumn]{article}`: `\textwidth
/// 6.875in`, `\columnsep 0.3125in`) sets its separation too: two 117pt panels and 4pt of glue are two rows of its
/// 237.136pt column (pdflatex), not one of 243.4pt, the declared page's 10pt separation. Green before (int75a, no page
/// declared); guards the declared page. Repro captions-floats/a_style_sets_its_own_columns.
#[test]
fn a_style_sets_its_own_column_separation() {
  let xml = convert_clean(repro!("captions-floats/a_style_sets_its_own_columns.tex"));
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F1\""],
    &format!(
      "{}{}{}<break class=\"ltx_break\"/>{}{}</figure>",
      gallery("S0.F1", 2, 1, "1", "0.493"),
      figure_tags(1),
      rule_panel("117.0pt", "0.493"),
      rule_panel("117.0pt", "0.493"),
      figure_captions(1, "Two 117pt panels in a column.")
    ),
  );
}

/// A float held in a `{minipage}{0.5\textwidth}` is set against the minipage's line, half the running-text column
/// outside it: `panelline` 0.5, two `0.45\linewidth` panels 0.45 of it (pdflatex: `\hsize` = `\textwidth` =
/// `\columnwidth` = 172.5pt inside, the page's column 345pt). Red (79c r2: `panelline` 1, the minipage's `\textwidth`
/// taken as the page; review rev79c2 finding 2). Repro captions-floats/panel_line_of_a_float_in_a_minipage.
#[test]
fn a_float_in_a_minipage_is_a_fraction_of_the_page_outside_it() {
  let xml = convert_clean(repro!(
    "captions-floats/panel_line_of_a_float_in_a_minipage.tex"
  ));
  let panel = "<rule align=\"center\" class=\"ltx_figure_panel\" height=\"28.5pt\" panelwidth=\"0.45\" \
               width=\"77.6pt\"/>";
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F1\""],
    &format!(
      "<figure class=\"ltx_minipage ltx_panels_gallery\" inlist=\"lof\" panelcols=\"2\" panelfill=\"0.9\" \
       panelline=\"0.5\" panelrows=\"1\" placement=\"H\" vattach=\"middle\" width=\"172.5pt\" \
       xml:id=\"S0.F1\">{}{panel}{panel}{}</figure>",
      figure_tags(1),
      figure_captions(1, "Two panels in a half-width minipage.")
    ),
  );
}

/// subfig's `\subfloat` is its body's width with the glue the body states (subfig.sty:375-376 measures the body's
/// `\hbox`; :424 `\ignorespaces` leaves no glue between sub-floats): three `\rule{120pt}{1cm}\hspace{-8pt}` bodies,
/// 112pt each, are one row of the 345pt line, each 0.325 of it (pdflatex). Red (int75a: three rows, each sub-float its
/// box, which takes the line; no `panelwidth`). Witness 2008.06630 Figs 1, 7. Repro
/// captions-floats/a_subfig_body_states_its_glue.
#[test]
fn a_subfig_body_states_its_glue() {
  let xml = convert_clean(repro!("captions-floats/a_subfig_body_states_its_glue.tex"));
  let sub = |n: u32, letter: char| {
    format!(
      "<figure align=\"center\" class=\"ltx_figure_panel\" panelwidth=\"0.325\" xml:id=\"S0.F1.sf{n}\"><tags><tag>\
       ({letter})</tag><tag role=\"refnum\">1{letter}</tag></tags><rule height=\"28.5pt\" width=\"120.0pt\"/></figure>"
    )
  };
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F1\""],
    &format!(
      "{}{}{}{}{}{}</figure>",
      gallery("S0.F1", 1, 3, "1", "0.975"),
      figure_tags(1),
      sub(1, 'a'),
      sub(2, 'b'),
      sub(3, 'c'),
      figure_captions(1, "Three sub-floats.")
    ),
  );
}

/// A `\parbox` is set at the width the source states, read where latex.ltx:16252 reads it (`\setlength\@tempdima{#4}`,
/// outside the box): against a two-column page a one-column table's `\parbox{\linewidth}` note is a whole column and
/// `\parbox{0.5\columnwidth}` half of it (pdflatex: 229.5pt and 114.75pt in the letter page's 229.5pt column). Red
/// (int78a: 1.503 and 0.752, the macro handing `\lx@parbox` the value of Perl's 345pt registers, no tokens to read at
/// the declared page). Repro captions-floats/a_parbox_states_its_width_against_the_page.
#[test]
fn a_parbox_states_its_width_against_the_page() {
  let xml = convert_clean(repro!(
    "captions-floats/a_parbox_states_its_width_against_the_page.tex"
  ));
  for (id, panelwidth, width, text) in [
    ("S0.T1", "1", "345.0pt", "A note across the column."),
    ("S0.T2", "0.5", "172.5pt", "Half."),
  ] {
    assert_element(
      &xml,
      "p",
      &[&format!("xml:id=\"{id}.2\"")],
      &format!(
        "<p class=\"ltx_figure_panel ltx_parbox\" panelwidth=\"{panelwidth}\" vattach=\"middle\" width=\"{width}\" \
         xml:id=\"{id}.2\">{text}</p>"
      ),
    );
  }
}
