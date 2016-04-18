///**********************************************************************
/// Test cases for rtx
///**********************************************************************
extern crate rtx;
use rtx::util::test::*;

#[test]
fn can_complex() {
  rtx_tests("tests/complex", None);
}
