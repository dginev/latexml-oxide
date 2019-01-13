///**********************************************************************
/// Test cases for rtx
///**********************************************************************
use rtx_package::util::test::*;
use std::collections::HashMap;

#[test]
#[ignore]
fn can_expand() {
  let mut requires = HashMap::new();
  requires.insert("meaning", "t1enc.def");
  requires.insert("ifthen", "ifthen.sty");

  rtx_tests("tests/expansion", Some(requires));
}
