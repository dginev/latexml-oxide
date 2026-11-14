//! Red/green guards for batch 66n: an `\hrule` is an alignment's border only between its rows (root-causer report
//! on 2308.07459's raw ytableau, 2026-10-14).
use super::perfect_kernel_batch46::{error_count, warning_count};

fn convert_counted(tex: &str) -> (String, String) {
  latexml::util::test::convert_with(tex, Some("ar5iv.sty"))
}

/// A rule inside a cell's box is the box's own (ytableau's box frames): every row keeps all its cells, each a formula,
/// and no border row is added (2308.07459).
#[test]
fn rule_in_a_cell_box_keeps_the_row() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/alignment/rule_in_a_cell_box_keeps_the_row.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(xml.matches("<td").count(), 11, "{xml}");
  assert!(!xml.contains("border="), "{xml}");
  for formula in [
    "tex=\"a\"",
    "tex=\"b\"",
    "tex=\"c\"",
    "tex=\"d\"",
    "tex=\"p_{1n}\"",
    "tex=\"Z\"",
  ] {
    assert!(xml.contains(formula), "{formula}: {xml}");
  }
  assert!(xml.contains("AFTERWORDS survive."), "{xml}");
}
