use std::rc::Rc;

///**********************************************************************
/// Test cases for latexml_oxide
///**********************************************************************
use latexml::util::test::*;

#[test]
fn can_contrib() {
  let requires = None;

  latexml_tests_internal(
    "tests/contrib",
    requires,
    Some(Rc::new(latexml_contrib::dispatch)),
  );
}
