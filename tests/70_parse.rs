///**********************************************************************
/// Test cases for RusteXML
///**********************************************************************
extern crate rustexml;
use rustexml::util::test::*;

#[test]
fn can_parse() {
  rustexml_tests("tests/parse", None);
}
