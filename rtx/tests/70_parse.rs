#![feature(macro_literal_matcher)]
///**********************************************************************
/// Test cases for rtx
///**********************************************************************
extern crate rtx;
use rtx::util::test::*;

#[test]
#[ignore]
fn can_parse() { rtx_tests("tests/parse", None); }
