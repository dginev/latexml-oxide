//! Red/green guards for batch GA5: decisions on a token that TeX takes by its meaning, read by meaning (GA0 finding
//! 10). One table, a row per changed code path whose output differed, each the `\let`-copy shape that was wrong (oracle
//! in the repro header). A module of its own: no table of meaning-for-spelling rows existed; later GA batches add rows.
use latexml::util::test::{assert_element, convert_files_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

/// One repro: its source, its exact (errors, warnings), whole elements `(tag, attributes, element)` and text the
/// output holds.
struct Row {
  name:     &'static str,
  tex:      &'static str,
  counts:   (usize, usize),
  elements: &'static [(&'static str, &'static str, &'static str)],
  holds:    &'static [&'static str],
}

const ROWS: &[Row] = &[
  // tex_math.rs `eqno_stops_at`: `\let\dollar=$` ends the tag, as Perl's `defined_as`
  Row {
    name:     "eqno_tag_ends_at_a_let_dollar",
    tex:      include_str!(
      "../../../tools/perfect_kernel/repros/kernel-alignment/eqno_tag_ends_at_a_let_dollar.tex"
    ),
    counts:   (0, 0),
    elements: &[(
      "p",
      "xml:id=\"p1.1\"",
      r#"<p xml:id="p1.1">Text after.</p>"#,
    )],
    holds:    &[r#"tex="e=f""#, r#"<XMTok meaning="3" role="NUMBER""#],
  },
  // tex_tables.rs `before_cell_unlist`/`after_cell_unlist`, `is_fill`: `\let\myhfil\hfil` in a math template
  Row {
    name:     "halign_let_hfil_in_math_template",
    tex:      include_str!(
      "../../../tools/perfect_kernel/repros/kernel-alignment/halign_let_hfil_in_math_template.tex"
    ),
    counts:   (0, 0),
    elements: &[(
      "td",
      "xml:id=\"p1.1.1.1\"",
      r#"<td align="center" class="ltx_nopad_l ltx_nopad_r" xml:id="p1.1.1.1"><Math mode="inline" tex="x" text="x" xml:id="p1.m1"><XMath xml:id="p1.m1.1"><XMTok font="italic" role="UNKNOWN">x</XMTok></XMath></Math></td>"#,
    )],
    holds:    &[],
  },
  // tex_tables.rs `template_after_fill_defeated`: the after template's last fill by the same reading (`is_fill`) as
  // `has_after_fill`, so content after a `\myhfil` defeats it as after `\hfil` (pinned, the same on GA5base)
  Row {
    name:     "halign_let_hfil_before_content_after",
    tex:      include_str!(
      "../../../tools/perfect_kernel/repros/kernel-alignment/halign_let_hfil_before_content_after.tex"
    ),
    counts:   (0, 0),
    elements: &[(
      "td",
      "xml:id=\"p1.1.1.1\"",
      r#"<td align="right" class="ltx_nopad_l ltx_nopad_r" xml:id="p1.1.1.1">a  <Math mode="inline" tex="|" text="|" xml:id="p1.m1"><XMath xml:id="p1.m1.1"><XMTok role="VERTBAR" stretchy="false">|</XMTok></XMath></Math></td>"#,
    )],
    holds:    &[],
  },
];

/// Every row converted once, in-process: exact counts, the whole elements.
#[test]
fn batchga5_meaning_rows() {
  for row in ROWS {
    let (log, xml) = convert_files_with(row.tex, &[], Some("ar5iv.sty"));
    let name = row.name;
    assert_eq!(error_count(&log), row.counts.0, "{name}: errors\n{log}");
    assert_eq!(warning_count(&log), row.counts.1, "{name}: warnings\n{log}");
    for (tag, attrs, element) in row.elements {
      assert_element(&xml, tag, &[attrs], element);
    }
    for text in row.holds {
      assert!(xml.contains(text), "{name}: no {text:?}\n{xml}");
    }
  }
}
