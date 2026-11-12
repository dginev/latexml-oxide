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
  // The safe side: a group's paragraph of nothing or a space adds no empty <p> (pdflatex and Perl the same), nor a
  // `<text/>` for a font-switched blank (1602.08755; tex.web §816, where Perl leaves an empty `<text>`).
  (
    repro!("group_par_space_only_paragraph_makes_none"),
    0,
    0,
    "document",
    &[
      "<para xml:id=\"p1\">\n    <p xml:id=\"p1.1\">x</p>\n  </para>\n  <para xml:id=\"p2\">\n    <p xml:id=\"p2.1\">Hb</p>\n  </para>\n  <para xml:id=\"p3\">\n    <p xml:id=\"p3.1\">x</p>\n  </para>\n  <para xml:id=\"p4\">\n    <p xml:id=\"p4.1\">Ib</p>\n  </para>\n  <para xml:id=\"p5\">\n    <p xml:id=\"p5.1\">x</p>\n  </para>\n  <para xml:id=\"p6\">\n    <p xml:id=\"p6.1\">Kb</p>\n  </para>\n  <para xml:id=\"p7\">\n    <p xml:id=\"p7.1\">Ma</p>\n    <p align=\"center\" xml:id=\"p7.2\"><text fontsize=\"80%\" xml:id=\"p7.2.1\">Mb</text></p>\n  </para>\n",
    ],
  ),
  // revRG1 B1: a group's blank paragraph measures no line of its own; x's line is outside the group (pdflatex 4.30554pt;
  // rg1 16.30554pt for `x{ \par}`, `x{ \par\par}`, `x{{ \par}}`; 12pt more after it, 4.2pt more in a p-cell).
  (
    repro!("group_par_blank_paragraph_measures_no_line"),
    0,
    0,
    "document",
    &[
      "<para xml:id=\"p1\">\n    <p xml:id=\"p1.1\">H1=4.30554pt, D1=0.0pt.</p>\n  </para>\n  <para xml:id=\"p2\">\n    <p xml:id=\"p2.1\">H2=4.30554pt, D2=0.0pt.</p>\n  </para>\n  <para xml:id=\"p3\">\n    <p xml:id=\"p3.1\">H3=4.30554pt, D3=0.0pt.</p>\n  </para>\n  <para xml:id=\"p4\">\n    <p xml:id=\"p4.1\">H4=4.30554pt, D4=0.0pt.</p>\n  </para>\n  <para xml:id=\"p5\">\n    <p xml:id=\"p5.1\">H5=4.30554pt, D5=0.0pt.</p>\n  </para>\n  <para xml:id=\"p6\">\n    <p xml:id=\"p6.1\">H6=16.30554pt, D6=1.94444pt.</p>\n  </para>\n  <para xml:id=\"p7\">\n    <p xml:id=\"p7.1\">H7=8.5pt, D7=3.5pt.</p>\n  </para>\n",
    ],
  ),
  // Its safe twin (revRG1b S1): after `\noindent` the paragraph outside the group holds nothing and makes no line; the
  // trailing x discriminates (rg1: 12pt more on A1, A3, an empty top line in a minipage, `\parbox`, `\vtop`).
  (
    repro!("group_par_blank_paragraph_after_noindent_makes_no_line"),
    0,
    0,
    "document",
    &[
      "<para xml:id=\"p1\">\n    <p xml:id=\"p1.1\">A1=4.30554pt, D1=0.0pt.</p>\n  </para>\n  <para xml:id=\"p2\">\n    <p xml:id=\"p2.1\">A2=4.30554pt, D2=0.0pt.</p>\n  </para>\n  <para xml:id=\"p3\">\n    <p xml:id=\"p3.1\">A3=4.30554pt, D3=0.0pt.</p>\n  </para>\n  <para xml:id=\"p4\">\n    <p xml:id=\"p4.1\">A4=4.30554pt, D4=0.0pt.</p>\n  </para>\n  <para xml:id=\"p5\">\n    <p xml:id=\"p5.1\">A5=4.30554pt, D5=0.0pt.</p>\n  </para>\n  <para xml:id=\"p6\">\n    <p xml:id=\"p6.1\">A6=4.30554pt, D6=0.0pt.</p>\n  </para>\n  <para xml:id=\"p7\">\n    <p xml:id=\"p7.1\">A7=4.30554pt, D7=0.0pt.</p>\n  </para>\n",
    ],
  ),
  // B1 in a tikz text-width node: one line, as pdflatex (rg1: height 32.34, depth 1.2em); the second node a group of
  // glue (revRG1b B1: rg1b 32.34, 1.2em). Each node's empty first <p/> is int162a's too, not this mechanism.
  (
    repro!("group_par_blank_in_tikz_node_measures_no_line"),
    0,
    0,
    "picture",
    &[
      "<picture height=\"15.73\" width=\"127.89\" xml:id=\"p1.pic1\">\n      <svg:svg height=\"15.73\" overflow=\"visible\" version=\"1.1\" viewBox=\"0 0 127.89 15.73\" width=\"127.89\">\n        <svg:g fill=\"#000000\" stroke=\"#000000\" stroke-width=\"0.4pt\" transform=\"translate(0,15.73) matrix(1 0 0 -1 0 0) translate(63.94,0) translate(0,7.87)\">\n          <svg:path d=\"M -63.67 -7.59 h 127.33 v 15.18 h -127.33 Z\" style=\"fill:none\"/>\n          <svg:g fill=\"#000000\" stroke=\"#000000\" transform=\"matrix(1.0 0.0 0.0 1.0 -59.06 -2.98)\">\n            <svg:foreignObject height=\"5.96\" overflow=\"visible\" style=\"--ltx-fo-width:8.54em;--ltx-fo-height:0.43em;--ltx-fo-depth:0em;font-size:10pt;\" transform=\"matrix(1 0 0 -1 0 5.96)\" width=\"118.17\"><inline-block class=\"ltx_minipage\" vattach=\"top\" width=\"8.54em\" xml:id=\"p1.pic1.1\">\n                <p xml:id=\"p1.pic1.1.1\"/>\n                <p class=\"ltx_align_left\" xml:id=\"p1.pic1.1.2\">x</p>\n              </inline-block></svg:foreignObject>\n          </svg:g>\n        </svg:g>\n      </svg:svg>\n    </picture>",
      "<picture height=\"15.73\" width=\"127.89\" xml:id=\"p2.pic1\">\n      <svg:svg height=\"15.73\" overflow=\"visible\" version=\"1.1\" viewBox=\"0 0 127.89 15.73\" width=\"127.89\">\n        <svg:g fill=\"#000000\" stroke=\"#000000\" stroke-width=\"0.4pt\" transform=\"translate(0,15.73) matrix(1 0 0 -1 0 0) translate(63.94,0) translate(0,7.87)\">\n          <svg:path d=\"M -63.67 -7.59 h 127.33 v 15.18 h -127.33 Z\" style=\"fill:none\"/>\n          <svg:g fill=\"#000000\" stroke=\"#000000\" transform=\"matrix(1.0 0.0 0.0 1.0 -59.06 -2.98)\">\n            <svg:foreignObject height=\"5.96\" overflow=\"visible\" style=\"--ltx-fo-width:8.54em;--ltx-fo-height:0.43em;--ltx-fo-depth:0em;font-size:10pt;\" transform=\"matrix(1 0 0 -1 0 5.96)\" width=\"118.17\"><inline-block class=\"ltx_minipage\" vattach=\"top\" width=\"8.54em\" xml:id=\"p2.pic1.1\">\n                <p xml:id=\"p2.pic1.1.1\"/>\n                <p class=\"ltx_align_left\" xml:id=\"p2.pic1.1.2\">x</p>\n              </inline-block></svg:foreignObject>\n          </svg:g>\n        </svg:g>\n      </svg:svg>\n    </picture>",
    ],
  ),
  // revRG1b B1: a group of glue, kerns or penalties adds no line (rg1b 16.30554pt for `\quad`, `\hskip1cm`, `\kern1pt`
  // and their mix; 12pt more before y; a p-cell 4.2pt more; a minipage's depth 12pt). Controls already one line on
  // rg1b: `\hfill`, `\,`, `\hspace`, `\hspace*`, `\nobreak\hskip0pt`, `\phantom{}` (a box: kept), `\penalty`.
  (
    repro!("group_par_glue_kern_paragraph_measures_no_line"),
    0,
    0,
    "document",
    &[
      "<para xml:id=\"p1\">\n    <p xml:id=\"p1.1\">H1=4.30554pt, D1=0.0pt.</p>\n  </para>\n  <para xml:id=\"p2\">\n    <p xml:id=\"p2.1\">H2=4.30554pt, D2=0.0pt.</p>\n  </para>\n  <para xml:id=\"p3\">\n    <p xml:id=\"p3.1\">H3=4.30554pt, D3=0.0pt.</p>\n  </para>\n  <para xml:id=\"p4\">\n    <p xml:id=\"p4.1\">H4=16.30554pt, D4=1.94444pt.</p>\n  </para>\n  <para xml:id=\"p5\">\n    <p xml:id=\"p5.1\">H5=8.5pt, D5=3.5pt.</p>\n  </para>\n  <para xml:id=\"p6\">\n    <p xml:id=\"p6.1\">H6=4.30554pt, D6=0.0pt.</p>\n  </para>\n  <para xml:id=\"p7\">\n    <p xml:id=\"p7.1\">H7=4.30554pt, D7=0.0pt.</p>\n  </para>\n  <para xml:id=\"p8\">\n    <p xml:id=\"p8.1\">H8=4.30554pt, D8=0.0pt.</p>\n  </para>\n  <para xml:id=\"p9\">\n    <p xml:id=\"p9.1\">H9=4.30554pt, D9=0.0pt.</p>\n  </para>\n  <para xml:id=\"p10\">\n    <p xml:id=\"p10.1\">H10=4.30554pt, D10=0.0pt.</p>\n  </para>\n  <para xml:id=\"p11\">\n    <p xml:id=\"p11.1\">H11=4.30554pt, D11=0.0pt.</p>\n  </para>\n  <para xml:id=\"p12\">\n    <p xml:id=\"p12.1\">H12=4.30554pt, D12=0.0pt.</p>\n  </para>\n  <para xml:id=\"p13\">\n    <p xml:id=\"p13.1\">H13=4.30554pt, D13=0.0pt.</p>\n  </para>\n  <para xml:id=\"p14\">\n    <p xml:id=\"p14.1\">H14=4.30554pt, D14=0.0pt.</p>\n  </para>\n",
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
