//! Red/green guards for issue 736: nicematrix's `{NiceTabular}` keeps its column and row features — the `p[keys]`
//! columns, `\Block` as a spanning cell (and its `\\` lines), the body's `\rowcolors` — with no `&`/`\noalign` errors.
//! Each repro's pdflatex reference text is in its header.
use latexml::util::test::convert_with;

use super::perfect_kernel_batch46::{error_count, warning_count};

/// One row per mechanism: the repro (under `package-bindings/`, 0 errors and 0 warnings) and whole squashed elements of
/// its table. `p[c]{2em}` (2607.15452); `\Block{i-j}` spanning, stopping at a covered cell with text (2206.12224);
/// `\Block{2-1}{Building\\Block}` (2206.12224); the body's two-color `\rowcolors` before a `$` cell (2602.03088).
#[test]
fn nicematrix_body_features_keep_the_table() {
  let rows: [(&str, &str, &[&str]); 4] = [
    (
      "nicematrix_p_column_keys.tex",
      include_str!(
        "../../../tools/perfect_kernel/repros/package-bindings/nicematrix_p_column_keys.tex"
      ),
      &[
        r#"<td align="left" vattach="top" xml:id="p1.1.1.1"><inline-block vattach="top" width="20.0pt" xml:id="p1.1.1.1.1"> <p align="center" xml:id="p1.1.1.1.1.1">1</p> </inline-block></td>"#,
        r#"<td align="left" vattach="middle" xml:id="p1.1.1.2"><inline-block vattach="middle" width="30.0pt" xml:id="p1.1.1.2.1"> <p class="ltx_align_right" xml:id="p1.1.1.2.1.1">2</p> </inline-block></td>"#,
        r#"<td align="left" vattach="top" xml:id="p1.1.1.3"><inline-block vattach="top" width="30.0pt" xml:id="p1.1.1.3.1"> <p class="ltx_align_left" xml:id="p1.1.1.3.1.1">3</p> </inline-block></td> </tr>"#,
      ],
    ),
    (
      "nicematrix_block_spans_cells.tex",
      include_str!(
        "../../../tools/perfect_kernel/repros/package-bindings/nicematrix_block_spans_cells.tex"
      ),
      &[
        r#"<tr xml:id="p1.1.1"> <td align="center" colspan="2" rowspan="2" xml:id="p1.1.1.1">Big</td> <td align="center" xml:id="p1.1.1.2">C</td> <td align="center" xml:id="p1.1.1.3">D</td> </tr> <tr xml:id="p1.1.2"> <td align="center" xml:id="p1.1.2.1">F</td> <td align="center" xml:id="p1.1.2.2">G</td> </tr>"#,
        r#"<tr xml:id="p1.1.3"> <td align="center" xml:id="p1.1.3.1">E</td> <td align="center" colspan="2" xml:id="p1.1.3.2">Wide</td> <td align="center" xml:id="p1.1.3.3">H</td> </tr>"#,
        r#"<tr xml:id="p1.1.4"> <td align="center" colspan="2" xml:id="p1.1.4.1">Long</td> <td align="center" xml:id="p1.1.4.2">x</td> <td align="center" xml:id="p1.1.4.3">I</td> </tr>"#,
      ],
    ),
    (
      "nicematrix_block_newline_body.tex",
      include_str!(
        "../../../tools/perfect_kernel/repros/package-bindings/nicematrix_block_newline_body.tex"
      ),
      &[
        r#"<td align="center" border="tt" rowspan="2" thead="row" xml:id="p1.1.1.1"><tabular vattach="middle" xml:id="p1.1.1.1.1"> <tr xml:id="p1.1.1.1.1.1"> <td align="center" xml:id="p1.1.1.1.1.1.1">Building</td> </tr> <tr xml:id="p1.1.1.1.1.2"> <td align="center" xml:id="p1.1.1.1.1.2.1">Block</td> </tr> </tabular></td> <td align="center" border="tt" xml:id="p1.1.1.2">Head</td> </tr>"#,
        r#"<tr xml:id="p1.1.2"> <td align="center" border="t" xml:id="p1.1.2.1">Sub</td> </tr>"#,
      ],
    ),
    (
      "nicematrix_body_rowcolors_arity.tex",
      include_str!(
        "../../../tools/perfect_kernel/repros/package-bindings/nicematrix_body_rowcolors_arity.tex"
      ),
      &[
        r#"<td align="center" thead="row" xml:id="p1.1.2.1"><Math mode="inline" tex="x_{1}" text="x _ 1" xml:id="p1.m1">"#,
        r#"<td align="center" xml:id="p1.1.2.2">c</td> </tr> <tr xml:id="p1.1.3"> <td align="center" thead="row" xml:id="p1.1.3.1">d</td> <td align="center" xml:id="p1.1.3.2">e</td> </tr>"#,
      ],
    ),
  ];
  for (name, tex, parts) in rows {
    let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{name}\n{log}"
    );
    let xml = xml.split_whitespace().collect::<Vec<_>>().join(" ");
    for part in parts {
      assert!(xml.contains(part), "{name}: {part}\n{xml}");
    }
  }
}
