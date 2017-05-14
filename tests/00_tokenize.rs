///**********************************************************************
/// Test cases for rtx
///**********************************************************************
extern crate rtx;
use rtx::util::test::*;

#[test] #[ignore]
fn can_tokenize() {
  let requires = None;

  rtx_tests("tests/tokenize", requires);
}
