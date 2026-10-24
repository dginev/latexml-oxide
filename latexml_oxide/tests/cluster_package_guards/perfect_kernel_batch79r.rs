//! Red/green guards for batch 79r (complete breakage, continued: user 2026-10-24): `{numcases}`'s `\\` reads its
//! `*`/`[` with the alignment's tabs disabled, as latex.ltx's `\@eqncr` does. Each guard converts its repro under
//! `tools/perfect_kernel/repros/`, asserts the diagnostics by count and the elements.
use latexml::util::test::convert_files_with;

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// `{subnumcases}` and `{numcases}` rows after `\\` whose first cell is empty (`& $c \geq d$`): cases_sty.rs
/// `\@numcases@newline` reads the row end as `\lx@alignment@newline` does (tex_tables.rs `alignment_newline_then`), so
/// the next row's `&` is that row's; and a left-hand side holding `\\` (`{M_0=0\Rightarrow \\}`) is set with the outer
/// `\\`, as cases.sty sets it once before its `\let\\\@eqncr`, while a `{numcases}` with no `{…}` (its left-hand side
/// the next token, `\mathrm`) keeps its first row. The seven numbered equations keep their labels. Red (int88b):
/// "\lx@hidden@egroup Attempt to close a group that switched to mode math" and "\endgroup Attempt to close non-boxing
/// group"; int88c: the left-hand side's `\\` re-inserted the left-hand side (Fatal); int88d: `\mathrm` took the
/// group's closing token as its argument (8 errors, both rows lost). Witnesses 2510.01717, 1709.02454, 2202.06960,
/// 2412.05950. Repro alignment-bindings/numcases_row_with_empty_first_cell.
#[test]
fn a_numcases_row_with_an_empty_first_cell_ends_cleanly() {
  let (log, xml) = convert_files_with(
    repro!("alignment-bindings/numcases_row_with_empty_first_cell.tex"),
    &[],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(xml.matches("<equation ").count(), 8, "{xml}");
  assert!(xml.contains("is given."), "{xml}");
  for label in ["eq:a", "eq:b", "eq:c", "eq:d", "eq:e", "eq:f", "eq:g"] {
    assert_eq!(
      xml.matches(&format!("labels=\"LABEL:{label}\"")).count(),
      1,
      "{label}: {xml}"
    );
    assert!(
      xml.contains(&format!(r#"<ref labelref="LABEL:{label}"/>"#)),
      "{label}: {xml}"
    );
  }
}
