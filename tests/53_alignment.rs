///**********************************************************************
/// Test cases for RusteXML
///**********************************************************************
extern crate rustexml;
use rustexml::util::test::*;
use std::collections::HashMap;

#[test]
fn can_align() {
  let mut requires = HashMap::new();
  requires.insert("listing", "listings.cfg");
  rustexml_tests("tests/alignment", Some(requires));
}
