///**********************************************************************
/// Test cases for RusteXML
///**********************************************************************
extern crate rustexml;
use rustexml::util::test::*;

#[test]
fn can_tokenize() {
  let requires = None;
  
  rustexml_tests("tests/tokenize", requires);
}
