///**********************************************************************
/// Test cases for RusteXML
///**********************************************************************
extern crate rustexml;
use rustexml::util::test::*;

#[test]
fn can_namespace() {
  rustexml_tests("tests/structure", None);
}
