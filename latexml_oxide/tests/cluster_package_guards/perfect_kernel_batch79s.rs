//! Red/green guards for batch 79s (complete breakage, continued: user 2026-10-24): a siunitx `S` column takes its
//! `>{…}` prefix as given, unexpanded, ahead of the cell, as siunitx's collector does. Each guard converts its repro
//! under `tools/perfect_kernel/repros/`, asserts the diagnostics by count and the elements.
use latexml::util::test::convert_files_with;

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// `>{\centering\arraybackslash}S[table-format=2.1]` columns: siunitx_sty.rs `read_si_cell` takes the prefix up to
/// the template's `\ignorespaces` unexpanded (siunitx.sty:5625 `\__siunitx_table_collect_begin:w #1 \ignorespaces`),
/// so `\arraybackslash`'s `\let\\\tabularnewline` is cell material, not a row end run inside the cell, and
/// `\centering`'s `\\` no longer opens the cell with a line break. Red (int88e): "Missing { inserted" in every row, the
/// table never closed (7 errors); `>{\centering}S` cells began with `<break/>`. Witness 2504.01955 (210 errors → 0).
/// Repro package-bindings/siunitx_s_column_prefix.
#[test]
fn an_s_column_prefix_is_taken_as_given() {
  let (log, xml) = convert_files_with(
    repro!("package-bindings/siunitx_s_column_prefix.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(xml.matches("<tabular ").count(), 2, "{xml}");
  assert_eq!(xml.matches("<tr ").count(), 3, "{xml}");
  for text in ["Road", "Wall", ">Ours<", ">12.5<", ">3.4<", ">7.25<"] {
    assert!(xml.contains(text), "{text}: {xml}");
  }
  assert!(!xml.contains("<break"), "{xml}");
}
