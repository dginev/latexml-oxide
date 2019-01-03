///**********************************************************************
/// Test cases for rtx
///**********************************************************************
extern crate rtx_package;
use rtx_package::util::test::*;

#[test]
#[ignore]
fn can_digest() { rtx_tests("tests/digestion", None); }
