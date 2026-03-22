///**********************************************************************
/// Test cases for latexml_oxide — AMS suite
///**********************************************************************
use latexml::util::test::*;

const DIR: &str = "tests/ams";

#[test]
fn amsdisplay_test() {
  latexml_test_single("tests/ams/amsdisplay.tex", "amsdisplay", DIR, None, None);
}

#[test]
#[ignore] // 221 diffs: XMCell structure, XMDual/XMWrap diffs
fn cd_test() {
  latexml_test_single("tests/ams/cd.tex", "cd", DIR, None, None);
}

#[test]
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
fn matrix_test() {
  latexml_test_single("tests/ams/matrix.tex", "matrix", DIR, None, None);
}

#[test]
fn sideset_test() {
  latexml_test_single("tests/ams/sideset.tex", "sideset", DIR, None, None);
}
