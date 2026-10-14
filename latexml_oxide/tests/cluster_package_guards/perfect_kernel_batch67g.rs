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
  // Radii 10 and 5 in `\unitlength` = 1pt: the extremes of the axes, so operands read as 0 would fail.
  let corners: Vec<&str> = points.split(' ').collect();
  for extreme in ["13.84,0", "0,6.92", "-13.84,0", "0,-6.92"] {
    assert!(
      corners.contains(&extreme),
      "{extreme} missing from {points}"
    );
  }
}

/// ellipse.sty:300-302 `\ellipse*` and pict2e's `\polygon*` fill their path (`\pIIe@fillGraph`, `\fillpath`):
/// one filled, unstroked polyline each; the plain forms stay stroked and unfilled.
#[test]
fn pict2e_filled_shapes_are_filled() {
  let tex = "\\documentclass{article}\n\\usepackage{ellipse}\n\\begin{document}\n\\begin{picture}(40,40)\
             \\put(20,20){\\ellipse*{4}{2}}\\put(10,10){\\ellipse{4}{2}}\\polygon*(0,0)(5,0)(5,5)\
             \\polygon(10,0)(15,0)(15,5)\\end{picture}\n\\end{document}\n";
  let (log, xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let lines: Vec<&str> = xml
    .match_indices("<line ")
    .map(|(i, _)| &xml[i..i + xml[i..].find('>').unwrap_or(0)])
    .collect();
  assert_eq!(lines.len(), 4, "{xml}");
  let filled = lines
    .iter()
    .filter(|l| l.contains(r##"fill="#000000""##) && l.contains(r#"stroke="none""#))
    .count();
  let stroked = lines
    .iter()
    .filter(|l| !l.contains("fill=") && l.contains(r##"stroke="#000000""##))
    .count();
  assert_eq!((filled, stroked), (2, 2), "{lines:?}");
}
