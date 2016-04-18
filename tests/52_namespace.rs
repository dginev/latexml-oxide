///**********************************************************************
/// Test cases for rtx
///**********************************************************************
extern crate rtx;
use rtx::util::test::*;

#[test]
fn can_namespace() {
  rtx_tests("tests/structure", None);
}
