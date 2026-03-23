use latexml::util::test::*;
const DIR: &str = "tests/slides";

#[test]
#[ignore] // needs beamer.cls binding
fn beamer_test() {
  latexml_test_single("tests/slides/beamer.tex", "beamer", DIR, None, None);
}

#[test]
fn slides_test() {
  latexml_test_single("tests/slides/slides.tex", "slides", DIR, None, None);
}
