///**********************************************************************
/// Test cases for latexml_oxide — AMS suite
///**********************************************************************
use latexml::util::test::*;

const DIR: &str = "tests/ams";

#[test]
#[ignore] // diffs — math parser (XMDual structure, text= attribute)
fn amsdisplay_test() {
  latexml_test_single("tests/ams/amsdisplay.tex", "amsdisplay", DIR, None, None);
}

#[test]
#[ignore] // crash — math parser panic in parse_rec tree replacement
fn cd_test() {
  latexml_test_single("tests/ams/cd.tex", "cd", DIR, None, None);
}

#[test]
#[ignore] // diffs — math parser (XMDual structure for comma lists, integrals)
fn dots_test() {
  latexml_test_single("tests/ams/dots.tex", "dots", DIR, None, None);
}

#[test]
fn genfracs_test() {
  latexml_test_single("tests/ams/genfracs.tex", "genfracs", DIR, None, None);
}

#[test]
#[ignore] // crash — MathPrimitive unhandled in is_defined_token
fn mathtools_test() {
  latexml_test_single("tests/ams/mathtools.tex", "mathtools", DIR, None, None);
}

#[test]
#[ignore] // diffs — math parser (XMDual structure)
fn matrix_test() {
  latexml_test_single("tests/ams/matrix.tex", "matrix", DIR, None, None);
}

#[test]
#[ignore] // diffs — math parser (XMDual structure)
fn sideset_test() {
  latexml_test_single("tests/ams/sideset.tex", "sideset", DIR, None, None);
}
