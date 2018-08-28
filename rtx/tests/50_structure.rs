#![feature(macro_literal_matcher)]
///**********************************************************************
/// Test cases for rtx
///**********************************************************************
extern crate rtx;
use rtx::util::test::*;

#[test]
#[ignore]
fn can_structure() { rtx_tests("tests/structure", None); }
