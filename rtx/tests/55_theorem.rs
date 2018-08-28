#![feature(macro_literal_matcher)]
///**********************************************************************
/// Test cases for rtx
///**********************************************************************
extern crate rtx;
use rtx::util::test::*;
use std::collections::HashMap;

#[test]
#[ignore]
fn can_theorem() {
  let mut requires = HashMap::new();
  requires.insert("ntheorem", "ntheorem.std");
  rtx_tests("tests/theorem", Some(requires));
}
