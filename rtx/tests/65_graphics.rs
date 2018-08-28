#![feature(macro_literal_matcher)]
///**********************************************************************
/// Test cases for rtx
///**********************************************************************
extern crate rtx;
use rtx::util::test::*;
use std::collections::HashMap;

#[test]
#[ignore]
fn can_graphics() {
  let mut requires = HashMap::new();
  requires.insert("colors", "dvipsnam.def");
  requires.insert("xcolors", "dvipsnam.def");
  rtx_tests("tests/graphics", Some(requires));
}
