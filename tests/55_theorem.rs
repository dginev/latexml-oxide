///**********************************************************************
/// Test cases for RusteXML
///**********************************************************************
extern crate rustexml;
use rustexml::util::test::*;
use std::collections::HashMap;

#[test]
fn can_theorem() {
  let mut requires = HashMap::new();
  requires.insert("ntheorem", "ntheorem.std");
  rustexml_tests("tests/theorem", Some(requires));
}
