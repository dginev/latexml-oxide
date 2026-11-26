//! Batch ARYDSHLN: arydshln's dashed rules reach the HTML as `ltx_border_<side>_dashed` beside their `ltx_border_<side>`
//! (upstream LaTeXML PR 2889, drawn dashed: OD #1025). The core XML is the `tests/contrib/arydshln` golden; this guard
//! is its HTML, the one place the XSLT folds `border` and the dashed class into a cell's classes (td and mtd); the
//! colortbl rows are repros of their own.
use latexml::util::test::convert_with;

use super::perfect_kernel_batch46::{error_count, warning_count};

/// Every element `<tag …>…</tag>` of `html` (not nested), in order.
fn elements<'a>(html: &'a str, tag: &str) -> Vec<&'a str> {
  let (open, close) = (format!("<{tag} "), format!("</{tag}>"));
  html
    .match_indices(&open)
    .filter_map(|(i, _)| {
      html[i..]
        .find(&close)
        .map(|e| &html[i..i + e + close.len()])
    })
    .collect()
}

#[test]
fn arydshln_dashed_borders_reach_the_html() {
  let (log, _, (html, post_log)) = latexml::util::test::convert_bytes_files_then(
    include_bytes!("../contrib/arydshln.tex"),
    &[],
    None,
    |xml, source_dir| {
      super::perfect_kernel_batch64zf::post_logged(xml, source_dir.to_str().expect("a UTF-8 path"))
    },
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(error_count(&post_log), 0, "{post_log}");
  let cells: Vec<&str> = ["td", "th", "mtd"]
    .iter()
    .flat_map(|tag| elements(&html, tag))
    .collect();
  for cell in [
    // `c:c|c` after `\hdashline`: the `:` an `r` border, dashed; the dashed top on every cell
    r#"<td class="ltx_td ltx_border_r_dashed ltx_border_t_dashed ltx_align_center ltx_border_r ltx_border_t">d</td>"#,
    r#"<td class="ltx_td ltx_border_t_dashed ltx_align_center ltx_border_r ltx_border_t">e</td>"#,
    // `\cdashline{1-2}` stops before the third column
    r#"<td class="ltx_td ltx_align_center">i</td>"#,
    // `::` is a double right border, dashed; the closing `\lasthdashline` the bottom
    r#"<td class="ltx_td ltx_border_b_dashed ltx_border_r_dashed ltx_border_t_dashed ltx_align_center ltx_border_b ltx_border_rr ltx_border_t">p</td>"#,
    // `\hline\hdashline`: a side mixing a solid and a dashed rule is solid; the rule colour stays
    r#"<td class="ltx_td ltx_border_r_dashed ltx_align_center ltx_border_r ltx_border_tt" style="border-top-color:#FF0000;border-right-color:#FF0000;">t</td>"#,
    // the math array's `:` and `\hdashline`
    r#"<mtd class="ltx_border_r ltx_border_r_dashed"><mn>2</mn></mtd>"#,
    r#"<mtd class="ltx_border_t ltx_border_t_dashed"></mtd>"#,
  ] {
    assert!(cells.contains(&cell), "missing {cell} in {cells:#?}");
  }
  // A leading `:` is the first column's left border, and no cell keeps the rule's spacing as text (was `  x`).
  let x = cells
    .iter()
    .find(|c| c.ends_with(">x</th>"))
    .expect("the x cell");
  assert!(
    x.contains("ltx_border_l_dashed") && x.contains("ltx_border_l "),
    "{x}"
  );
  assert!(
    !cells
      .iter()
      .any(|c| c.contains('\u{2002}') || c.contains('\u{200A}')),
    "{cells:#?}"
  );
}

/// The colortbl shapes, each repro's cells whole (its header has the pdflatex reference): the `:` takes
/// `\arrayrulecolor`, and the gap colour setters exist only under colortbl (arydshln.sty:1053-1125).
#[test]
fn arydshln_colortbl_rows() {
  let rows: [(&str, &str, &[&str], &[&str]); 3] = [
    // the `:` is red, as pdflatex draws it, beside the `|` and the dashed rows
    (
      "colour",
      include_str!(
        "../../../tools/perfect_kernel/repros/alignment-bindings/arydshln_dash_column_colour.tex"
      ),
      &[],
      &[
        r#"<td align="center" border="r t" class="ltx_border_r_dashed ltx_border_t_dashed" cssstyle="border-top-color:#FF0000;border-right-color:#FF0000" xml:id="p1.1.1.1">a</td>"#,
        r#"<td align="center" border="r t" class="ltx_border_t_dashed" cssstyle="border-top-color:#FF0000;border-right-color:#FF0000" xml:id="p1.1.1.2">b</td>"#,
        r#"<td align="center" border="b r t" class="ltx_border_b_dashed ltx_border_r_dashed" cssstyle="border-top-color:#FF0000;border-bottom-color:#FF0000;border-right-color:#FF0000" xml:id="p1.1.2.1">d</td>"#,
      ],
    ),
    // without colortbl, `\dashgapcolor` and `\nodashgapcolor` are undefined: an Error each, the colour spec kept as text
    (
      "no colortbl",
      include_str!(
        "../../../tools/perfect_kernel/repros/alignment-bindings/arydshln_dashgapcolor_needs_colortbl.tex"
      ),
      &["\\dashgapcolor", "\\nodashgapcolor"],
      &[
        r#"<p xml:id="p1.2">[rgb]1,0,0"#,
        r#"<td align="center" border="b r t" class="ltx_border_b_dashed ltx_border_r_dashed ltx_border_t_dashed" xml:id="p1.2.1.1.1">a</td>"#,
      ],
    ),
    // under colortbl they read their arguments and draw nothing
    (
      "colortbl",
      include_str!(
        "../../../tools/perfect_kernel/repros/alignment-bindings/arydshln_dashgapcolor_under_colortbl.tex"
      ),
      &[],
      &[
        r#"<td align="center" border="b r t" class="ltx_border_b_dashed ltx_border_r_dashed ltx_border_t_dashed" xml:id="p1.1.1.1">a</td>"#,
        r#"<td align="center" border="b t" class="ltx_border_b_dashed ltx_border_t_dashed" xml:id="p1.1.1.2">b</td>"#,
      ],
    ),
  ];
  for (name, tex, undefined, cells) in rows {
    let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (undefined.len(), 0),
      "{name}: {log}"
    );
    for cs in undefined {
      assert!(
        log.contains(&format!("Error:undefined:{cs} ")),
        "{name}: {cs} {log}"
      );
    }
    for cell in cells {
      assert!(xml.contains(cell), "{name}: missing {cell}\n{xml}");
    }
    if undefined.is_empty() {
      assert!(
        !xml.contains("rgb") && !xml.contains("1,0,0"),
        "{name}: {xml}"
      );
    }
  }
}
