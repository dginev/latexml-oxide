///**********************************************************************
/// Test cases for rtx
///**********************************************************************
extern crate rtx;
use rtx::util::test::*;

#[test] #[ignore]
fn can_theorem() {
  rtx_tests("tests/ams", None);
}
