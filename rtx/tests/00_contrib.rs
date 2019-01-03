///**********************************************************************
/// Test cases for rtx
///**********************************************************************
extern crate rtx_package;
use rtx_package::util::test::*;
use std::rc::Rc;

#[test]
fn can_contrib() {
  let requires = None;

  rtx_tests_internal("tests/contrib", requires, Some(Rc::new(rtx_contrib::dispatch)));
}
