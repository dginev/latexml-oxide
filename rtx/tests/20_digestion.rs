#![feature(macro_literal_matcher)]
///**********************************************************************
/// Test cases for rtx
///**********************************************************************
extern crate rtx;
use rtx::util::test::*;

#[test]
#[ignore]
fn can_digest() { rtx_tests("tests/digestion", None); }
