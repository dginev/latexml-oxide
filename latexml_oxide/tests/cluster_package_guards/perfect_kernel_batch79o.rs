//! Red/green guards for batch 79o (complete breakage, continued: user 2026-10-24): a nicematrix row that starts with
//! `&` inside `align` is the matrix's, as ltcmd's alignment-safe peeks keep it in pdflatex. Each guard converts its
//! repro under `tools/perfect_kernel/repros/`, asserts the diagnostics by count and the elements.
use latexml::util::test::convert_files_with;

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// `\begin{pNiceArray}{cc}[first-row]`, `\begin{pNiceMatrix}` and `\begin{bNiceMatrix}[r]`, each followed by a
/// newline and a row `& entry \\`, inside `align`: the `\CodeBefore` test (nicematrix_sty.rs
/// `nice_code_before_follows`) and the `!O{}` options (`OptionalBalancedNoSkip`) peek, so the `&` opens the matrix's
/// second column — three matrices of two rows (each twice, in the equation's `MathFork`), every first row an empty
/// cell then its entry. Red (int86b): 34 errors ("\end@amsalign Attempt to close a group that switched to mode
/// restricted_horizontal"), no matrix. Witnesses 2302.00928, 2406.05750, 2207.07825. Repro
/// alignment-bindings/nicematrix_row_leading_tab_in_align.
#[test]
fn a_matrix_row_that_starts_with_a_tab_is_the_matrixs() {
  let (log, xml) = convert_files_with(
    repro!("alignment-bindings/nicematrix_row_leading_tab_in_align.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(xml.matches("<XMArray ").count(), 6, "{xml}");
  assert_eq!(xml.matches("<XMRow ").count(), 12, "{xml}");
  assert_eq!(xml.matches(">first</XMText>").count(), 2, "{xml}");
  for array in xml.split("<XMArray ").skip(1) {
    let first_row = array.split("</XMRow>").next().unwrap_or_default();
    let cell = first_row.split("<XMCell").nth(1).unwrap_or_default();
    assert!(
      cell.split('>').next().unwrap_or_default().ends_with('/'),
      "first cell empty: {first_row}"
    );
  }
  assert_eq!(xml.matches("After the display.").count(), 1, "{xml}");
}
