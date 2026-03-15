use latexml::util::test::*;
const DIR: &str = "tests/moderncv";

#[test]
#[ignore] // needs moderncv.cls binding
fn cs_cv_test() {
  latexml_test_single("tests/moderncv/cs_cv.tex", "cs_cv", DIR, None, None);
}

#[test]
#[ignore] // needs moderncv.cls binding
fn orc_test() {
  latexml_test_single("tests/moderncv/orc.tex", "orc", DIR, None, None);
}
