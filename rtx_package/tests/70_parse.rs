///**********************************************************************
/// Test cases for rtx
///**********************************************************************
extern crate rtx_package;
use rtx_package::util::test::*;

#[test]
#[ignore]
fn can_parse() { rtx_tests("tests/parse", None); }
