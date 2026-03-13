// Keyval options tests — all depend on xkeyval package which is not yet ported.
// xkeyval loading causes infinite loop due to unported package.
use latexml::util::test::*;
const DIR: &str = "tests/keyval_options";

#[test]
#[ignore] // needs xkeyval package (infinite loop)
fn xkvdop1a_test() {
  latexml_test_single("tests/keyval_options/xkvdop1a.tex", "xkvdop1a", DIR, None, None);
}

#[test]
#[ignore] // needs xkeyval package (infinite loop)
fn xkvdop1b_test() {
  latexml_test_single("tests/keyval_options/xkvdop1b.tex", "xkvdop1b", DIR, None, None);
}

#[test]
#[ignore] // needs xkeyval package (infinite loop)
fn xkvdop2a_test() {
  latexml_test_single("tests/keyval_options/xkvdop2a.tex", "xkvdop2a", DIR, None, None);
}

#[test]
#[ignore] // needs xkeyval package (infinite loop)
fn xkvdop2b_test() {
  latexml_test_single("tests/keyval_options/xkvdop2b.tex", "xkvdop2b", DIR, None, None);
}

#[test]
#[ignore] // needs xkeyval package (infinite loop)
fn xkvdop3a_test() {
  latexml_test_single("tests/keyval_options/xkvdop3a.tex", "xkvdop3a", DIR, None, None);
}

#[test]
#[ignore] // needs xkeyval package (infinite loop)
fn xkvdop3b_test() {
  latexml_test_single("tests/keyval_options/xkvdop3b.tex", "xkvdop3b", DIR, None, None);
}

#[test]
#[ignore] // needs xkeyval package (infinite loop)
fn xkvdop4a_test() {
  latexml_test_single("tests/keyval_options/xkvdop4a.tex", "xkvdop4a", DIR, None, None);
}

#[test]
#[ignore] // needs xkeyval package (infinite loop)
fn xkvdop5a_test() {
  latexml_test_single("tests/keyval_options/xkvdop5a.tex", "xkvdop5a", DIR, None, None);
}

#[test]
#[ignore] // needs xkeyval package (infinite loop)
fn xkvdop5b_test() {
  latexml_test_single("tests/keyval_options/xkvdop5b.tex", "xkvdop5b", DIR, None, None);
}

#[test]
#[ignore] // needs xkeyval package (infinite loop)
fn xkvdop6a_test() {
  latexml_test_single("tests/keyval_options/xkvdop6a.tex", "xkvdop6a", DIR, None, None);
}

#[test]
#[ignore] // needs xkeyval package (infinite loop)
fn xkvdop6b_test() {
  latexml_test_single("tests/keyval_options/xkvdop6b.tex", "xkvdop6b", DIR, None, None);
}
