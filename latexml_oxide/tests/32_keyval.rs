// Keyval tests
// keyvalinline passes, keyvalstyle has diffs
// xkeyval* tests loop infinitely — needs xkeyval package ported
use latexml::util::test::*;
const DIR: &str = "tests/keyval";

#[test]
fn keyvalinline_test() {
  latexml_test_single("tests/keyval/keyvalinline.tex", "keyvalinline", DIR, None, None);
}

#[test]
fn keyvalstyle_test() {
  latexml_test_single("tests/keyval/keyvalstyle.tex", "keyvalstyle", DIR, None, None);
}

#[test]
#[ignore] // infinite loop — needs xkeyval package
fn xkeyvaladv_test() {
  latexml_test_single("tests/keyval/xkeyvaladv.tex", "xkeyvaladv", DIR, None, None);
}

#[test]
#[ignore] // infinite loop — needs xkeyval package
fn xkeyvalbasic_test() {
  latexml_test_single("tests/keyval/xkeyvalbasic.tex", "xkeyvalbasic", DIR, None, None);
}

#[test]
#[ignore] // infinite loop — needs xkeyval package
fn xkeyvalkvcompat_test() {
  latexml_test_single("tests/keyval/xkeyvalkvcompat.tex", "xkeyvalkvcompat", DIR, None, None);
}

#[test]
#[ignore] // infinite loop — needs xkeyval package
fn xkeyvalstyle_test() {
  latexml_test_single("tests/keyval/xkeyvalstyle.tex", "xkeyvalstyle", DIR, None, None);
}

#[test]
#[ignore] // infinite loop — needs xkeyval package
fn xkeyvalview_test() {
  latexml_test_single("tests/keyval/xkeyvalview.tex", "xkeyvalview", DIR, None, None);
}
