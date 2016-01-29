///**********************************************************************
/// Test cases for RusteXML
///**********************************************************************
extern crate rustexml;
use rustexml::util::test::*;

#[test]
fn can_theorem() {
  rustexml_tests("tests/ams", None);
}
