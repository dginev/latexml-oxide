//! Red/green guards for batch 79q (complete breakage, continued: user 2026-10-24): soul's commands set a `\\` in their
//! argument as a line break, never the end of an alignment row. Each guard converts its repro under
//! `tools/perfect_kernel/repros/`, asserts the diagnostics by count and the elements.
use latexml::util::test::convert_files_with;

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// `\hl{line \\ line}` and `\ul{line \\ line}` in a `p{\linewidth}` cell: soul_sty.rs `soul_line_breaks` lets `\\`
/// be `\newline` while the argument digests (soul.sty:263/348), so each highlight keeps both lines with a break
/// between them and the table, its caption and its label survive. Red (int88a): 10 errors ("\lx@texthl@color Attempt
/// to close a group that switched to mode horizontal"), the second lines and the table lost. Witnesses 2310.07652,
/// 2406.17962. Repro package-bindings/soul_newline_in_table_cell.
#[test]
fn a_soul_line_break_in_a_table_cell_is_a_line_break() {
  let (log, xml) = convert_files_with(
    repro!("package-bindings/soul_newline_in_table_cell.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for line in [
    "First line of the prompt.",
    "Second line of the prompt.",
    "Single-column perspective.",
    "Under the line.",
  ] {
    assert_eq!(xml.matches(line).count(), 1, "{line}: {xml}");
  }
  let cell = xml
    .split("Prompt:")
    .nth(1)
    .and_then(|c| c.split("</tabular>").next())
    .unwrap_or_default();
  assert_eq!(cell.matches("<break").count(), 2, "{xml}");
  assert!(
    xml.contains(r#"<ref labelref="LABEL:tab:prompt"/>"#),
    "{xml}"
  );
}
