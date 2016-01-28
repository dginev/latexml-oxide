///**********************************************************************
/// Test cases for RusteXML
///**********************************************************************
extern crate rustexml;
use rustexml::util::test::*;
use std::collections::HashMap;

#[test]
fn can_tokenize() {
  let mut requires = HashMap::new();
  requires.insert("meaning", "t1enc.def");
  requires.insert("ifthen", "ifthen.sty");

  rustexml_tests("tests/expansion", Some(requires));
}
