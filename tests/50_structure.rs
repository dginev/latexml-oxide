///**********************************************************************
/// Test cases for RusteXML
///**********************************************************************
extern crate rustexml;
use rustexml::util::test::*;

#[test]
fn can_structure() {
  rustexml_tests("tests/structure", None);
}
