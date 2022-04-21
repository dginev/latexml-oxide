use rtx::util::test::*;
///**********************************************************************
/// Test cases for rtx
///**********************************************************************
use std::collections::HashMap;

#[test]
fn can_structure() {
  let mut requires = HashMap::new();
  requires.insert("csquotes", "csquotes.sty");
  rtx_tests("tests/structure", Some(requires), None);
}
