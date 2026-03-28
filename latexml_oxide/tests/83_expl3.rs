use latexml::util::test::*;
const DIR: &str = "tests/expl3";

#[test]
fn tilde_tricks_test() {
  latexml_test_single("tests/expl3/tilde_tricks.tex", "tilde_tricks", DIR, None, None);
}

#[test]
fn xparse_test() {
  latexml_test_single("tests/expl3/xparse.tex", "xparse", DIR, None, None);
}
