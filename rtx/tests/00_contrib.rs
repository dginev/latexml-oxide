///**********************************************************************
/// Test cases for rtx
///**********************************************************************
use rtx::util::test::*;
use std::rc::Rc;

#[test]
fn can_contrib() {
  let requires = None;

  rtx_tests_internal(
    "tests/contrib",
    requires,
    Some(Rc::new(rtx_contrib::dispatch)),
  );
}
