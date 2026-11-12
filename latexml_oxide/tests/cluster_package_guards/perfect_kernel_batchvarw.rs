//! VARW: a varwidth box is as wide as its widest line, at most the width given (varwidth.sty `\endvarwidth`;
//! `minipage_takes_natural_width`; KNOWN_PERL_ERRORS #1234, OD #1014). Witnesses algxpar-doc, 2401.12710.
use latexml::util::test::assert_element;

use super::{
  perfect_kernel_batch46::{convert_with, error_count, warning_count},
  svg_nested_picture::post_html,
};

/// The `[\the\wd0]` rows of a repro, in order.
fn rows(xml: &str) -> Vec<&str> {
  xml
    .split("<p xml:id=\"")
    .skip(1)
    .filter_map(|p| p.split_once('>')?.1.split_once("</p>"))
    .map(|(text, _)| text)
    .collect()
}

/// One row per shape, pdflatex's widths (repro header): a line, two lines, a tabular, a display in text, a broken
/// paragraph, two paragraphs, `\centering`, a rule, a minipage inside, a varwidth in a minipage, nested varwidths, a bare
/// `\varwidth` in a group then a minipage (the flag taken by its own box), `[t]`, `\hspace`, empty, `\large`,
/// `\settowidth`, array's `V` column, a minipage (its given width); the box in the text is its measured width.
/// Then algxpar's measured comment (algxpar-doc, 2401.12710): no overflow, one "⊳" per comment.
#[test]
fn varwidth_takes_its_widest_line() {
  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/boxes-groups/varwidth_measures_its_widest_line.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let widths = rows(&xml);
  assert_eq!(
    widths[..19],
    [
      "[22.3056pt]",
      "[86.19458pt]",
      "[44.00003pt]",
      "[41.62253pt]",
      "[80.02792pt]",
      "[115.13911pt]",
      "[22.3056pt]",
      "[142.26378pt]",
      "[113.81102pt]",
      "[170.71652pt]",
      "[10.5556pt]",
      "[79.2111pt]",
      "[45.58342pt]",
      "[62.1833pt]",
      "[0.0pt]",
      "[51.14253pt]",
      "[5.2778pt]",
      "[47.02785pt]",
      "[284.52756pt]",
    ],
    "{xml}"
  );
  // VARWb: the XML width is the box's measured one (was the given 284.5pt), its HTML following its content up to it.
  assert!(
    xml.contains(
      r#"<inline-block class="ltx_minipage" cssstyle="width:max-content;max-width:284.5pt" vattach="middle" width="22.3pt" xml:id="p20.1.1">"#
    ),
    "{xml}"
  );

  let (stderr, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/listings-algorithms/algxpar_comment_width_measures_the_text.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches(">⊳</XMTok>").count(), 2, "{xml}");
  assert!(!xml.contains("width=\"-"), "{xml}");
  for comment in [
    "<text font=\"slanted\" xml:id=\"algx1.l1.2.2.1\">a comment</text>",
    "<text font=\"slanted\" xml:id=\"algx1.l2.3.1.1.1.2.1\">Select method to solve, select equations to solve and \
     more words here to make it long</text>",
  ] {
    assert!(xml.contains(comment), "{comment}: {xml}");
  }
  assert!(
    xml.contains(
      "<p class=\"ltx_align_left\" xml:id=\"algx1.l2.3.1.1\">a long statement text that goes on and on and on and on \
       and on and on and on and on and on and on\n"
    ),
    "{xml}"
  );
}

/// The HTML draws a varwidth box as its measurement (OD #1014): `width` its widest line, the style
/// `width:max-content;max-width:<given>`. Each row a reader of that width: a TikZ node's foreignObject (its frame and
/// box 4.31em, was the given 17.07em, revVARW `tz.tex`), cells under a `\resizebox` scaling them 8.2 (each cell was
/// 345pt, off-screen, revVARW `rs1.tex`), and 2507.12476's Table 1 shape (cells in a float's resized tabular).
#[test]
fn varwidth_html_follows_its_content() {
  let html = |repro: &str| {
    let (stderr, xml) = convert_with(repro, Some("ar5iv.sty"));
    assert_eq!(error_count(&stderr), 0, "{stderr}");
    assert_eq!(warning_count(&stderr), 0, "{stderr}");
    post_html(&xml)
  };
  let tz = html(include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/varwidth_box_in_a_tikz_node_sizes_its_frame.tex"
  ));
  assert!(
    tz.contains(r#"d="M -34.46 -9.42 h 68.92 v 18.83 h -68.92 Z""#),
    "{tz}"
  );
  assert_element(
    &tz,
    "foreignObject",
    &[r#"width="59.64""#],
    r#"<foreignObject style="--ltx-fo-width:4.31em;--ltx-fo-height:0.69em;--ltx-fo-depth:0em;font-size:10pt;" height="9.61" overflow="visible" transform="matrix(1 0 0 -1 0 9.61)" width="59.64"><span class="ltx_foreignobject_container"><span class="ltx_foreignobject_content"><span id="p1.pic1.1" class="ltx_inline-block ltx_minipage ltx_align_middle" style="width:4.31em;width:max-content;max-width:170.7pt;"><span id="p1.pic1.1.1" class="ltx_p ltx_align_center">short text</span></span></span></span></foreignObject>"#,
  );

  let rs = html(include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/varwidth_cells_under_a_resizebox.tex"
  ));
  assert!(
    rs.contains("[345.0pt]") && rs.contains("[42.0556pt]"),
    "{rs}"
  );
  assert_element(
    &rs,
    "div",
    &[r#"id="p2.1""#],
    r#"<div id="p2.1" class="ltx_inline-block ltx_transformed_outer" style="width:345.0pt;height:155.4pt;vertical-align:-57.2pt;"><span class="ltx_transformed_inner" style="transform:translate(151.5pt,-68.2pt) scale(8.2034253503793,8.2034253503793) ;"><table id="p2.1.1" class="ltx_tabular ltx_align_middle"><tbody class="ltx_tbody"><tr id="p2.1.1.1" class="ltx_tr"><td id="p2.1.1.1.1" class="ltx_td ltx_align_center"><span id="p2.1.1.1.1.1" class="ltx_inline-block ltx_minipage ltx_align_middle" style="width:10.6pt;width:max-content;max-width:345.0pt;"><span id="p2.1.1.1.1.1.1" class="ltx_p">ab</span><span id="p2.1.1.1.1.1.2" class="ltx_p">cd</span></span></td><td id="p2.1.1.1.2" class="ltx_td ltx_align_center"><span id="p2.1.1.1.2.1" class="ltx_inline-block ltx_minipage ltx_align_middle" style="width:7.5pt;width:max-content;max-width:345.0pt;"><span id="p2.1.1.1.2.1.1" class="ltx_p">ef</span></span></td></tr></tbody></table></span></div>"#,
  );

  let t1 = html(include_str!(
    "../../../tools/perfect_kernel/repros/boxes-groups/varwidth_cells_resized_in_a_float.tex"
  ));
  assert!(t1.contains("[T461.91748pt]"), "{t1}");
  assert!(
    t1.contains("scale(0.746886650944443,0.746886650944443)"),
    "{t1}"
  );
  for (cell, width) in [(1, "64.5pt"), (2, "40.5pt"), (3, "53.1pt"), (4, "187.4pt")] {
    let style = format!(r#"style="width:{width};width:max-content;max-width:345.0pt;""#);
    assert!(
      t1.contains(&format!(
        r#"<span id="S0.T1.1.1.2.{cell}.1" class="ltx_inline-block ltx_minipage ltx_align_top" {style}>"#
      )),
      "cell {cell}: {t1}"
    );
  }
}
