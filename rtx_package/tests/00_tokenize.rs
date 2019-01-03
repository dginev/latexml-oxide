///**********************************************************************
/// Test cases for rtx
///**********************************************************************
extern crate rtx_package;
use rtx_package::util::test::*;

#[test]
fn can_tokenize() {
  let requires = None;

  rtx_tests("tests/tokenize", requires);
}

#[test]
#[ignore]
fn can_tokenize_todo() {
  let requires = None;

  rtx_tests("tests/tokenize_todo", requires);
}
