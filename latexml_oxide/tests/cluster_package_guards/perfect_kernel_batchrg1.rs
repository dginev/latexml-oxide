//! Guards for RG1: a `\par` inside a text group opened mid-paragraph keeps the paragraph, as Perl's open-brace box does
//! (TeX_Box.pool.ltxml:30-42, Stomach.pm:463-465; `stomach::digest_group_body_in_paragraph`). One table: each repro
//! converted once in-process, its exact Error and Warning counts, and the selected elements whole, in order.

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batchcr2::elements,
};

macro_rules! repro {
  ($name:literal) => {
    (
      $name,
      include_str!(concat!(
        "../../../tools/perfect_kernel/repros/boxes-groups/",
        $name,
        ".tex"
      )),
    )
  };
}

/// (repro, Errors, Warnings, the tag selected — `document` for the body from the first `<para` — those elements whole).
type Row = (
  (&'static str, &'static str),
  usize,
  usize,
  &'static str,
  &'static [&'static str],
);

const ROWS: &[Row] = &[
  // Each phantom before a `\par` in a group opened mid-paragraph, in its paragraph with the x before it, as Perl: in body
  // text, `\em`, nested and `\bgroup` groups, a second paragraph in the group, a minipage, a footnote, a `p` cell, a
  // `\parbox`, an `\item` (int162a lost Ca, Ea Eb, Ja, Aa, Pa, Qa, Ra, Sa).
  (
    repro!("group_par_mid_paragraph_keeps_its_paragraph"),
    0,
    0,
    "document",
    &[
      "<para xml:id=\"p1\">\n    <p xml:id=\"p1.1\">x<text class=\"ltx_phantom\" xml:id=\"p1.1.1\">Ca</text></p>\n  </para>\n  <para xml:id=\"p2\">\n    <p xml:id=\"p2.1\">Cb</p>\n  </para>\n  <para xml:id=\"p3\">\n    <p xml:id=\"p3.1\">x<text class=\"ltx_phantom\" xml:id=\"p3.1.1\">Da</text></p>\n  </para>\n  <para xml:id=\"p4\">\n    <p xml:id=\"p4.1\">Db</p>\n  </para>\n  <para xml:id=\"p5\">\n    <p xml:id=\"p5.1\">x<text class=\"ltx_phantom\" font=\"italic\" xml:id=\"p5.1.1\">Ea</text><text class=\"ltx_phantom\" font=\"italic\" xml:id=\"p5.1.2\">Eb</text></p>\n  </para>\n  <para xml:id=\"p6\">\n    <p xml:id=\"p6.1\"><text font=\"italic\" xml:id=\"p6.1.1\">Ec</text></p>\n  </para>\n  <para xml:id=\"p7\">\n    <p xml:id=\"p7.1\">x<text class=\"ltx_phantom\" xml:id=\"p7.1.1\">Fa</text></p>\n  </para>\n  <para xml:id=\"p8\">\n    <p xml:id=\"p8.1\">Fb\u{2003}x<text class=\"ltx_phantom\" xml:id=\"p8.1.1\">Ja</text></p>\n  </para>\n  <para xml:id=\"p9\">\n    <p xml:id=\"p9.1\"><text class=\"ltx_phantom\" xml:id=\"p9.1.1\">Jb</text></p>\n  </para>\n  <para xml:id=\"p10\">\n    <p xml:id=\"p10.1\">Jc</p>\n  </para>\n  <para xml:id=\"p11\">\n    <p xml:id=\"p11.1\"><inline-block class=\"ltx_minipage\" vattach=\"middle\" width=\"142.3pt\" xml:id=\"p11.1.1\">\n        <p xml:id=\"p11.1.1.1\">x<text class=\"ltx_phantom\" xml:id=\"p11.1.1.1.1\">Aa</text></p>\n        <p xml:id=\"p11.1.1.2\">Ab</p>\n      </inline-block></p>\n  </para>\n  <para xml:id=\"p12\">\n    <p xml:id=\"p12.1\">x<note mark=\"1\" role=\"footnote\" xml:id=\"footnote1\"><tags>\n          <tag>1</tag>\n          <tag role=\"refnum\">1</tag>\n          <tag role=\"typerefnum\">footnote 1</tag>\n        </tags>y<text class=\"ltx_phantom\" xml:id=\"footnote1.1\">Pa</text>Pb</note>\n<tabular vattach=\"middle\" xml:id=\"p12.1.1\">\n        <tbody>\n          <tr xml:id=\"p12.1.1.1\">\n            <td align=\"left\" vattach=\"top\" xml:id=\"p12.1.1.1.1\"><inline-block vattach=\"top\" width=\"85.4pt\" xml:id=\"p12.1.1.1.1.1\">\n                <p xml:id=\"p12.1.1.1.1.1.1\">x<text class=\"ltx_phantom\" xml:id=\"p12.1.1.1.1.1.1.1\">Qa</text></p>\n                <p xml:id=\"p12.1.1.1.1.1.2\">Qb</p>\n              </inline-block></td>\n          </tr>\n        </tbody>\n      </tabular>\n<inline-block class=\"ltx_parbox\" vattach=\"middle\" width=\"85.4pt\" xml:id=\"p12.1.2\">\n        <p xml:id=\"p12.1.2.1\">x<text class=\"ltx_phantom\" xml:id=\"p12.1.2.1.1\">Ra</text></p>\n        <p xml:id=\"p12.1.2.2\">Rb</p>\n      </inline-block></p>\n    <itemize xml:id=\"S0.I1\">\n      <item xml:id=\"S0.I1.i1\">\n        <tags>\n          <tag>•</tag>\n          <tag role=\"typerefnum\">1st item</tag>\n        </tags>\n        <para xml:id=\"S0.I1.i1.p1\">\n          <p xml:id=\"S0.I1.i1.p1.1\">x<text class=\"ltx_phantom\" xml:id=\"S0.I1.i1.p1.1.1\">Sa</text></p>\n        </para>\n        <para xml:id=\"S0.I1.i1.p2\">\n          <p xml:id=\"S0.I1.i1.p2.1\">Sb</p>\n        </para>\n      </item>\n    </itemize>\n  </para>\n",
    ],
  ),
  // The witness's shape, a tikz node's `\\` (2410.06985: int162a's first node lost the Cpd phantom).
  (
    repro!("group_par_in_tikz_node_keeps_its_paragraph"),
    0,
    0,
    "inline-block",
    &[
      "<inline-block class=\"ltx_minipage\" vattach=\"top\" width=\"18.49em\" xml:id=\"p1.pic1.1\">\n                <p xml:id=\"p1.pic1.1.1\"/>\n                <p xml:id=\"p1.pic1.1.2\"><text class=\"ltx_phantom\" xml:id=\"p1.pic1.1.2.1\">Cpd</text></p>\n                <p xml:id=\"p1.pic1.1.3\"><text class=\"ltx_phantom\" xml:id=\"p1.pic1.1.3.1\">pt</text></p>\n              </inline-block>",
      "<inline-block class=\"ltx_minipage\" vattach=\"top\" width=\"18.49em\" xml:id=\"p1.pic1.2\">\n                <p xml:id=\"p1.pic1.2.1\"/>\n                <p xml:id=\"p1.pic1.2.2\">Cpd</p>\n                <p xml:id=\"p1.pic1.2.3\">pt</p>\n              </inline-block>",
    ],
  ),
  // The safe side: a group's paragraph of nothing or a space adds no empty <p> (pdflatex and Perl the same).
  (
    repro!("group_par_space_only_paragraph_makes_none"),
    0,
    0,
    "document",
    &[
      "<para xml:id=\"p1\">\n    <p xml:id=\"p1.1\">x</p>\n  </para>\n  <para xml:id=\"p2\">\n    <p xml:id=\"p2.1\">Hb</p>\n  </para>\n  <para xml:id=\"p3\">\n    <p xml:id=\"p3.1\">x</p>\n  </para>\n  <para xml:id=\"p4\">\n    <p xml:id=\"p4.1\">Ib</p>\n  </para>\n  <para xml:id=\"p5\">\n    <p xml:id=\"p5.1\">x</p>\n  </para>\n  <para xml:id=\"p6\">\n    <p xml:id=\"p6.1\">Kb</p>\n  </para>\n",
    ],
  ),
];

/// The selected elements of `xml`: `document` is the body from its first `<para` to `</document>`.
fn selected(xml: &str, tag: &str) -> Vec<String> {
  if tag == "document" {
    let start = xml.find("<para").unwrap_or(xml.len());
    let end = xml.rfind("</document>").unwrap_or(xml.len()).max(start);
    vec![xml[start..end].to_string()]
  } else {
    elements(xml, tag)
  }
}

#[test]
fn batchrg1_group_paragraph_rows() {
  let mut failures = Vec::new();
  for &((name, tex), errors, warnings, tag, expected) in ROWS {
    let (log, xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
    let got = selected(&xml, tag);
    if (error_count(&log), warning_count(&log)) != (errors, warnings) || got != expected {
      failures.push(format!(
        "{name}: E/W {:?} (expected {:?})\n{got:#?}\n{log}",
        (error_count(&log), warning_count(&log)),
        (errors, warnings)
      ));
    }
  }
  assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}
