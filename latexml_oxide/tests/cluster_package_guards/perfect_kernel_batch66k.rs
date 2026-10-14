//! Red/green guards for batch 66k: wrong-mode roots of K1's word losses (root-causer report
//! `agent_reports/2026-10-14_k1_wordloss_triggers.md`): a `_`/`^` our engine read in text where pdflatex reads it in math.
use super::perfect_kernel_batch46::{error_count, warning_count};

fn convert_counted(tex: &str) -> (String, String) {
  latexml::util::test::convert_with(tex, Some("ar5iv.sty"))
}

/// cancel.sty's `\cancel` is robust, so an aligned cell's start does not expand it before the template's math
/// (1801.01765 and 4 more).
#[test]
fn cancel_at_aligned_cell_start() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/alignment/cancel_at_aligned_cell_start.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(r#"enclose="updiagonalstrike" font="italic" meaning="cancel" role="ENCLOSE""#),
    "{xml}"
  );
  assert!(xml.contains("AFTERWORDS survive."), "{xml}");
}

/// pstricks keeps graphics' boxing `\scalebox`: an inner `$…$` is a formula in the box (2104.00361).
#[test]
fn pstricks_keeps_graphics_scalebox() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/pstricks_keeps_graphics_scalebox.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains(r#"\scalebox{1}{$b_{1}$}"#), "{xml}");
  assert!(xml.contains("AFTERWORDS survive."), "{xml}");
}

/// scalerel's `\scaleobj` over graphics' `\scalebox` with pstricks loaded, in a tabular cell (2504.03865).
#[test]
fn pstricks_scalerel_scaleobj_in_tabular() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/pstricks_scalerel_scaleobj_in_tabular.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(xml.contains("AFTERWORDS survive."), "{xml}");
}

/// graphicx's `alt` is a string, never typeset: a formula in it stays out of the alignment cell's math (2605.23217).
#[test]
fn includegraphics_alt_is_a_string() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/includegraphics_alt_is_a_string.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    xml.contains(r#"description="\Psi' = (\mathrm{id}_A \otimes U)(\Psi)""#),
    "{xml}"
  );
  assert!(xml.contains("AFTERWORDS survive."), "{xml}");
}
