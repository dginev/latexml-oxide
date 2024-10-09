///**********************************************************************
/// Test cases for rtx
///**********************************************************************
use latexml::util::test::*;
use std::rc::Rc;

#[test]
fn can_contrib() {
  let requires = None;

  latexml_tests_internal(
    "tests/contrib",
    requires,
    Some(Rc::new(latexml_contrib::dispatch)),
  );
}
