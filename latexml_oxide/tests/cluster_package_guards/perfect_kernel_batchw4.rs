//! Red/green guards for batch W4 (#773): colortbl's `\arrayrulecolor` colours each border side with the colour
//! current when its rule is drawn (colortbl.sty:119-192), as `cssstyle="border-<side>-color:…"` on the cell; OD #1025.
//! Each repro's pdflatex reference is in its header.
use latexml::util::test::convert_with;

use super::perfect_kernel_batch46::{error_count, warning_count};

/// The XML with every whitespace run collapsed to one space.
fn squash(xml: &str) -> String { xml.split_whitespace().collect::<Vec<_>>().join(" ") }

/// Every repro's bordered cells, whole and in order; no other element carries a `cssstyle`.
#[test]
fn arrayrulecolor_colours_the_rules_drawn_under_it() {
  let rows: [(&str, &str, &[&str]); 6] = [
    // the preamble's colour on every rule (#773's shape)
    (
      "borders",
      include_str!(
        "../../../tools/perfect_kernel/repros/alignment-bindings/arrayrulecolor_colours_the_borders.tex"
      ),
      &[
        r#"<td align="left" border="b l r t" cssstyle="border-top-color:#BFBFBF;border-bottom-color:#BFBFBF;border-left-color:#BFBFBF;border-right-color:#BFBFBF" xml:id="p1.1.1.1">1</td>"#,
        r#"<td align="left" border="b r t" cssstyle="border-top-color:#BFBFBF;border-bottom-color:#BFBFBF;border-right-color:#BFBFBF" xml:id="p1.1.1.2">a</td>"#,
      ],
    ),
    // mid-table changes: `|` per row, `\hline`, `\cline`, a `\multicolumn`'s `|c|`, `[rgb]`, black adds nothing
    (
      "mid-table",
      include_str!(
        "../../../tools/perfect_kernel/repros/alignment-bindings/arrayrulecolor_mid_table_changes.tex"
      ),
      &[
        r#"<td align="left" border="l r t" thead="row" xml:id="p1.1.1.1">1</td>"#,
        r#"<td align="left" border="r t" xml:id="p1.1.1.2">a</td>"#,
        r#"<td align="left" border="l r t" cssstyle="border-top-color:#FF0000;border-left-color:#FF0000;border-right-color:#FF0000" thead="row" xml:id="p1.1.2.1">2</td>"#,
        r#"<td align="left" border="r t" cssstyle="border-top-color:#FF0000;border-right-color:#FF0000" xml:id="p1.1.2.2">b</td>"#,
        r#"<td align="left" border="l r" cssstyle="border-left-color:#FF0000;border-right-color:#FF0000" thead="row" xml:id="p1.1.3.1">3</td>"#,
        r#"<td align="left" border="r t" cssstyle="border-top-color:#FF0000;border-right-color:#FF0000" xml:id="p1.1.3.2">c</td>"#,
        r#"<td align="center" border="b l r t" cssstyle="border-top-color:#0000FF;border-left-color:#0000FF;border-right-color:#0000FF" thead="row" xml:id="p1.1.4.1">d</td>"#,
        r#"<td align="left" border="b r t" cssstyle="border-top-color:#0000FF;border-right-color:#0000FF" xml:id="p1.1.4.2">e</td>"#,
      ],
    ),
    // restored at a nested tabular's and a math array's end; booktabs' rules
    (
      "restored",
      include_str!(
        "../../../tools/perfect_kernel/repros/alignment-bindings/arrayrulecolor_restored_at_the_table_end.tex"
      ),
      &[
        r#"<td align="left" border="b l r t" cssstyle="border-top-color:#0000FF;border-bottom-color:#0000FF;border-left-color:#0000FF;border-right-color:#0000FF" xml:id="p1.1.1.1">1</td>"#,
        r#"<td align="left" border="b r t" cssstyle="border-top-color:#0000FF;border-bottom-color:#0000FF;border-right-color:#0000FF" xml:id="p1.1.1.2"><tabular"#,
        // the `|` frozen in the inner preambles (blue, teal), the `\hline`s red: pdflatex (820dd03806 had them red)
        r#"<td align="center" border="b l r t" cssstyle="border-top-color:#FF0000;border-bottom-color:#FF0000;border-left-color:#0000FF;border-right-color:#0000FF" xml:id="p1.1.1.2.1.1.1">x</td>"#,
        r#"<XMCell border="t" cssstyle="border-top-color:#FF0000" xml:id="p2.m1.1.1.1.1"/>"#,
        r#"<XMCell align="center" border="b l r" cssstyle="border-bottom-color:#FF0000;border-left-color:#008080;border-right-color:#008080" xml:id="p2.m1.1.1.2.1">"#,
        r#"<td align="left" border="tt" cssstyle="border-top-color:#008080" thead="row" xml:id="p3.1.1.1">2</td>"#,
        r#"<td align="left" border="tt" cssstyle="border-top-color:#008080" xml:id="p3.1.1.2">b</td>"#,
        r#"<td align="left" border="bb t" cssstyle="border-top-color:#FF8000;border-bottom-color:#FF8000" thead="row" xml:id="p3.1.2.1">3</td>"#,
        r#"<td align="left" border="bb t" cssstyle="border-top-color:#FF8000;border-bottom-color:#FF8000" xml:id="p3.1.2.2">c</td>"#,
      ],
    ),
    // restored after the alignment closes, a last row without `\\` included; no restore for IEEEeqnarraybox/longtable
    (
      "last-row",
      include_str!(
        "../../../tools/perfect_kernel/repros/alignment-bindings/arrayrulecolor_restored_after_the_last_row.tex"
      ),
      &[
        r#"<td align="center" border="l r t" xml:id="p1.1.1.1">a</td>"#,
        r#"<td align="center" border="l r t" cssstyle="border-top-color:#FF0000;border-left-color:#FF0000;border-right-color:#FF0000" xml:id="p1.1.1.2.1.1.1">x</td>"#,
        r#"<td align="center" border="l r t" cssstyle="border-top-color:#FF0000;border-left-color:#FF0000;border-right-color:#FF0000" xml:id="p1.1.1.2.1.2.1">y</td>"#,
        r#"<td align="center" border="b l r t" xml:id="p1.1.2.1">b</td>"#,
        r#"<XMCell border="t" cssstyle="border-top-color:#0000FF" xml:id="p1.m1.1.1.1.1"/>"#,
        r#"<XMCell align="center" border="l r" cssstyle="border-left-color:#0000FF;border-right-color:#0000FF" xml:id="p1.m1.1.1.2.1">"#,
        r#"<td align="center" border="l r t" cssstyle="border-top-color:#00FF00;border-left-color:#00FF00;border-right-color:#00FF00" xml:id="p2.1.1.1">c</td>"#,
        r#"<td align="center" border="l r t" cssstyle="border-top-color:#00FF00;border-left-color:#00FF00;border-right-color:#00FF00" xml:id="p2.1.2.1">k</td>"#,
        r#"<td align="left" border="b l r t" cssstyle="border-top-color:#FF0000;border-bottom-color:#FF0000;border-left-color:#FF0000;border-right-color:#FF0000" xml:id="p2.2.1.1">"#,
        r#"<td align="center" border="b l r t" xml:id="p2.3.1.1">e</td>"#,
        r#"<td align="center" border="l r t" cssstyle="border-top-color:#0000FF;border-left-color:#0000FF;border-right-color:#0000FF" xml:id="p3.1.1.1">f</td>"#,
        r#"<td align="center" border="r t" cssstyle="border-top-color:#0000FF;border-right-color:#0000FF" xml:id="p3.1.1.2">"#,
        r#"<td align="center" border="b l r t" cssstyle="border-top-color:#0000FF;border-bottom-color:#0000FF;border-left-color:#0000FF;border-right-color:#0000FF" xml:id="p3.1.2.1">g</td>"#,
        r#"<td align="center" border="b r t" cssstyle="border-top-color:#0000FF;border-bottom-color:#0000FF;border-right-color:#0000FF" xml:id="p3.1.2.2">h</td>"#,
        r#"<td align="center" border="b l r t" cssstyle="border-top-color:#FF0000;border-bottom-color:#FF0000;border-left-color:#FF0000;border-right-color:#FF0000" xml:id="S0.T1.1.1.1">i</td>"#,
        r#"<td align="center" border="b l r t" cssstyle="border-top-color:#FF0000;border-bottom-color:#FF0000;border-left-color:#FF0000;border-right-color:#FF0000" xml:id="p4.1.1.1">j</td>"#,
      ],
    ),
    // a `|` colour frozen in the preamble when set, else read as the rule is drawn (a left one as its cell starts)
    (
      "vline",
      include_str!(
        "../../../tools/perfect_kernel/repros/alignment-bindings/arrayrulecolor_vline_colour_at_preamble_or_cell_start.tex"
      ),
      &[
        r#"<td align="left" border="l r t" cssstyle="border-right-color:#FF0000" thead="row" vattach="top" xml:id="p1.1.1.1">"#,
        r#"<td align="left" border="r t" cssstyle="border-right-color:#FF0000" xml:id="p1.1.1.2">b</td>"#,
        r#"<td align="left" border="l r t" cssstyle="border-top-color:#FF0000;border-left-color:#FF0000;border-right-color:#FF0000" thead="row" vattach="top" xml:id="p1.1.2.1">"#,
        r#"<td align="left" border="r t" cssstyle="border-top-color:#FF0000;border-right-color:#0000FF" xml:id="p1.1.2.2">d</td>"#,
      ],
    ),
    // no `\arrayrulecolor`: the ambient text colour is no rule colour (the base's output)
    (
      "ambient",
      include_str!(
        "../../../tools/perfect_kernel/repros/alignment-bindings/arrayrulecolor_ambient_colour_is_not_the_rule_colour.tex"
      ),
      &[
        r##"<td align="left" border="b l r t" xml:id="p1.1.1.1"><text color="#FF0000" xml:id="p1.1.1.1.1">1</text></td>"##,
        r##"<td align="left" border="b r t" xml:id="p1.1.1.2"><text color="#FF0000" xml:id="p1.1.1.2.1">a</text></td>"##,
      ],
    ),
  ];
  for (name, tex, cells) in rows {
    let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{name}: {log}"
    );
    let xml = squash(&xml);
    let mut from = 0;
    for cell in cells {
      let at = xml[from..]
        .find(cell)
        .unwrap_or_else(|| panic!("{name}: {cell}\n{xml}"));
      from += at + cell.len();
    }
    let colored = cells
      .iter()
      .filter(|cell| cell.contains(" cssstyle="))
      .count();
    assert_eq!(xml.matches(" cssstyle=").count(), colored, "{name}: {xml}");
  }
}
