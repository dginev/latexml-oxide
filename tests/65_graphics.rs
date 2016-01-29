///**********************************************************************
/// Test cases for RusteXML
///**********************************************************************
extern crate rustexml;
use rustexml::util::test::*;
use std::collections::HashMap;

#[test]
fn can_graphics() {
  let mut requires = HashMap::new();
  requires.insert("colors", "dvipsnam.def");
  requires.insert("xcolors", "dvipsnam.def");
  rustexml_tests("tests/graphics", Some(requires));
}
