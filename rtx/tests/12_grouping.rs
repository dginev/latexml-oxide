///**********************************************************************
/// Test cases for rtx
///**********************************************************************
use rtx::util::test::*;
use std::sync::Arc;

#[test]
fn can_group() {
  rtx_tests(
    "tests/grouping",
    None,
    Some(Arc::new(rtx_contrib::dispatch)),
  );
}
