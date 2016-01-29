///**********************************************************************
/// Test cases for RusteXML
///**********************************************************************
extern crate rustexml;
use rustexml::util::test::*;

#[test]
fn can_group() {
  rustexml_tests("tests/grouping", None);
}
