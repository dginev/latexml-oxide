// Keyval tests
use latexml::util::test::*;
use std::rc::Rc;
const DIR: &str = "tests/keyval";

#[test]
fn keyvalinline_test() {
  latexml_test_single("tests/keyval/keyvalinline.tex", "keyvalinline", DIR, None, None);
}

#[test]
fn keyvalstyle_test() {
  latexml_test_single("tests/keyval/keyvalstyle.tex", "keyvalstyle", DIR, None,
    Some(Rc::new(latexml_contrib::dispatch)));
}

#[test]
fn xkeyvaladv_test() {
  latexml_test_single("tests/keyval/xkeyvaladv.tex", "xkeyvaladv", DIR, None, None);
}

#[test]
fn xkeyvalbasic_test() {
  latexml_test_single("tests/keyval/xkeyvalbasic.tex", "xkeyvalbasic", DIR, None, None);
}

#[test]
fn keyvalemptyvalue_test() {
  latexml_test_single("tests/keyval/keyvalemptyvalue.tex", "keyvalemptyvalue", DIR, None, None);
}

#[test]
fn xkeyvalkvcompat_test() {
  latexml_test_single("tests/keyval/xkeyvalkvcompat.tex", "xkeyvalkvcompat", DIR, None, None);
}

#[test]
fn xkeyvalstyle_test() {
  latexml_test_single("tests/keyval/xkeyvalstyle.tex", "xkeyvalstyle", DIR, None,
    Some(Rc::new(latexml_contrib::dispatch)));
}

#[test]
#[ignore] // needs myxkeyval.sty Rust binding (xkeyval view handling)
fn xkeyvalview_test() {
  latexml_test_single("tests/keyval/xkeyvalview.tex", "xkeyvalview", DIR, None, None);
}
