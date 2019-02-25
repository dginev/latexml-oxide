///**********************************************************************
/// Test cases for rtx
///**********************************************************************
use std::collections::HashMap;
use rtx_package::util::test::*;

#[test]
#[ignore]
fn can_structure() {
  let mut requires = HashMap::new();
  requires.insert("csquotes", "csquotes.sty");
  rtx_tests("tests/structure", Some(requires));
}
