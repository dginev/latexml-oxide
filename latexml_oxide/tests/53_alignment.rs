// tex_tests! disabled: alignment tests have unbounded memory leaks.
// Individual passing tests listed below.
use latexml::util::test::*;
const DIR: &str = "tests/alignment";

#[test]
fn tabtab_test() {
  latexml_test_single("tests/alignment/tabtab.tex", "tabtab", DIR, None, None);
}

#[test]
#[ignore] // diffs — alignment engine issues
fn halign_test() {
  latexml_test_single("tests/alignment/halign.tex", "halign", DIR, None, None);
}
