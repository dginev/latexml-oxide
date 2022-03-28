///**********************************************************************
/// Test cases for rtx
///**********************************************************************
use rtx::util::test::*;
use std::sync::Arc;

#[test]
fn can_contrib() {
  let requires = None;

  rtx_tests_internal("tests/contrib", requires, Some(Arc::new(rtx_contrib::dispatch)));
}
