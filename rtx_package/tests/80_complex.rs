///**********************************************************************
/// Test cases for rtx
///**********************************************************************
extern crate rtx_package;
use rtx_package::util::test::*;

#[test]
#[ignore]
fn can_complex() { rtx_tests("tests/complex", None); }
