///**********************************************************************
/// Test cases for rtx
///**********************************************************************
extern crate rtx;
use rtx::util::test::*;

#[test]
fn can_theorem() {
  rtx_tests("tests/ams", None);
}
