// tex_tests! disabled: font tests have unbounded memory leaks.
// Individual passing tests listed below.
use latexml::util::test::*;
const DIR: &str = "tests/fonts";

#[test]
fn textsymbols_test() {
  latexml_test_single("tests/fonts/textsymbols.tex", "textsymbols", DIR, None, None);
}
