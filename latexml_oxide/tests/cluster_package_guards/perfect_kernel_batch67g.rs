//! Red/green guards for batch 67g: pict2e's TeX Live-era registers and dependencies (run 349 Error→Fatal witness
//! 2109.06028).
use super::perfect_kernel_batch46::{error_count, warning_count};

/// ellipse.sty (and longfbox.sty, which loads it) uses `\@tempdimd`, which pict2e allocated before v0.4b, and trig's
/// `\CalculateSin`/`\UseCos`, which pict2e.sty:123 loads (2109.06028).
#[test]
fn pict2e_ellipse_has_its_old_registers_and_trig() {
  let (log, xml) = latexml::util::test::convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/stub-dependencies/pict2e_ellipse_old_tempdimd.tex"
    ),
    Some("ar5iv.sty"),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(xml.matches("<picture ").count(), 1, "{xml}");
  // The ellipse's four Bézier arcs, sampled into one closed polyline (pict2e.sty:295 `\pIIe@addtoGraph\pIIe@curveto@op`).
  let picture = xml.split("<picture ").nth(1).unwrap_or_default();
  let points = picture
    .split("<line ")
    .nth(1)
    .and_then(|line| line.split("points=\"").nth(1))
    .and_then(|rest| rest.split('"').next())
    .unwrap_or_else(|| panic!("no ellipse line: {xml}"));
  assert!(points.split(' ').count() >= 32, "{points}");
}
